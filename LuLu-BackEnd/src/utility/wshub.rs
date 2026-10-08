use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use actix_web::rt::time::interval;
use actix_web::{HttpRequest, HttpResponse, web};
use actix_ws::Message;
use futures_util::StreamExt;
use tokio::sync::broadcast;
use tokio::time::MissedTickBehavior;

#[derive(Clone, serde::Serialize)]
pub struct WsOut {
    /// `connected` on open, `enroll` when a device posts a new record.
    pub event: &'static str,
    pub message: String,
    /// Newest `enroll_id` so a client can ignore an event it already has.
    pub enroll_id: i32,
}

/// Broadcast hub shared by every worker.
#[derive(Clone)]
pub struct WsHub {
    tx: broadcast::Sender<WsOut>,
    sessions: Arc<AtomicU32>,
}

impl WsHub {
    pub fn new() -> Self {
        let (tx, _rx) = broadcast::channel(32);
        Self {
            tx,
            sessions: Arc::new(AtomicU32::new(0)),
        }
    }

    pub fn count(&self) -> u32 {
        self.sessions.load(Ordering::Relaxed)
    }

    /// Fan a message out to every connection. Returns the number of receivers
    /// that were live at send time (0 means nobody is listening).
    pub fn broadcast(&self, out: WsOut) -> usize {
        self.tx.send(out).unwrap_or(0)
    }
}

impl Default for WsHub {
    fn default() -> Self {
        Self::new()
    }
}

/// GET /api/ws?token=... — the live transaction feed.
#[actix_web::get("/ws")]
pub async fn ws_endpoint(
    req: HttpRequest,
    stream: web::Payload,
    state: web::Data<crate::utility::stor::AppState>,
    query: web::Query<WsAuth>,
) -> Result<HttpResponse, actix_web::Error> {
    match crate::utility::authorization::is_login_token(
        state.db.clone(),
        // Same transformation the bearer middleware applies to an
        // `Authorization` header: sessions store the hash, not the raw token.
        crate::utility::hash::encode_256(&query.token).await,
    )
    .await
    {
        Some(_user_nickname) => {}
        _ => {
            return Ok(HttpResponse::Unauthorized().json(
                crate::utility::stor::NoDataResponse::new("Session token invalid".to_string(), 401),
            ));
        }
    }
    ws_handler(req, stream, state.ws.clone())
}

#[derive(serde::Deserialize)]
pub struct WsAuth {
    #[serde(default)]
    pub token: String,
}

fn ws_handler(
    req: HttpRequest,
    stream: web::Payload,
    hub: WsHub,
) -> Result<HttpResponse, actix_web::Error> {
    let (response, session, msg_stream) = actix_ws::handle(&req, stream)?;

    hub.sessions.fetch_add(1, Ordering::Relaxed);
    log::info!("ws client connected (total {})", hub.count());

    actix_web::rt::spawn(async move {
        if let Err(err) = session_task(session, msg_stream, hub.clone()).await {
            log::warn!("ws session ended: {err}");
        }
        hub.sessions.fetch_sub(1, Ordering::Relaxed);
        log::info!("ws client disconnected (total {})", hub.count());
    });

    Ok(response)
}

async fn session_task(
    mut session: actix_ws::Session,
    mut msg_stream: actix_ws::MessageStream,
    hub: WsHub,
) -> Result<(), actix_ws::Closed> {
    let mut events = hub.tx.subscribe();

    let welcome = WsOut {
        event: "connected",
        message: "live feed ready".to_string(),
        enroll_id: 0,
    };
    // Complete the welcome frame before the loop starts, otherwise it races the
    // first select arm.
    session
        .text(serde_json::to_string(&welcome).unwrap_or_else(|_| "{}".to_string()))
        .await?;

    let mut heartbeat = interval(std::time::Duration::from_secs(30));
    heartbeat.set_missed_tick_behavior(MissedTickBehavior::Delay);

    // `Session::close` consumes the session, so the loop records only the
    // client's own close reason and the actual teardown happens once, after.
    let mut close_reason = None;

    loop {
        tokio::select! {
            // Fan-out from the device ingest path.
            event = events.recv() => match event {
                Ok(out) => {
                    let text = serde_json::to_string(&out).unwrap_or_else(|_| "{}".to_string());
                    if session.text(text).await.is_err() {
                        break;
                    }
                }
                // Lagged behind: the hub kept newer events, skip the missed ones
                // instead of closing an otherwise healthy client.
                Err(broadcast::error::RecvError::Lagged(_)) => continue,
                Err(broadcast::error::RecvError::Closed) => break,
            },

            // Heartbeat: keep proxies and the browser from idling the socket out.
            _ = heartbeat.tick() => {
                if session.ping(b"hb").await.is_err() {
                    break;
                }
            },

            // Client traffic. Browsers answer our pings for us, so the read side
            // only has to honour a close.
            incoming = msg_stream.next() => match incoming {
                Some(Ok(Message::Close(reason))) => {
                    close_reason = reason;
                    break;
                }
                Some(Err(_)) | None => break,
                Some(Ok(_)) => {}
            },
        }
    }

    session.close(close_reason).await
}
