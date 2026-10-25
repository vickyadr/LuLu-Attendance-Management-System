use actix_web::body::MessageBody;
use actix_web::dev::ServiceResponse;
use actix_web::http::{StatusCode, header};
use actix_web::middleware::{ErrorHandlerResponse, ErrorHandlers};
use actix_web::{App, HttpResponse, HttpServer, middleware, web};
use actix_web_httpauth::extractors::bearer;

use lulu_attendance_server::utility::router;
use lulu_attendance_server::utility::{
    cors, db, device_sweeper, hash, shutdown, stor::AppState, stor::NoDataResponse, wshub,
};
use lulu_attendance_server::service::pull_scheduler::{self, PullRegistry};
use lulu_attendance_server::service::live_scheduler::{self, LiveRegistry};

/// Message shown for a status raised outside any handler.
fn message_for(status: StatusCode) -> &'static str {
    match status {
        StatusCode::UNAUTHORIZED => "Session token invalid",
        StatusCode::FORBIDDEN => "Sorry, doesnt have permission",
        StatusCode::NOT_FOUND => "Route or resource not found",
        StatusCode::PAYLOAD_TOO_LARGE => "Payload too large",
        _ => "Internal error, please try again!",
    }
}

/// Actix answers an empty body for errors raised outside a handler: a missing
/// `Authorization` header (from the bearer extractor), an unrouted path, or a
/// malformed JSON body. `$fetch` throws on those and the frontend finds no
/// `err.data`, so rewrite the body as the same JSON envelope the handlers use.
/// That keeps `message`/`code` readable on every failure, whatever raised it.
fn json_error<B: MessageBody + 'static>(
    status: StatusCode,
) -> impl Fn(ServiceResponse<B>) -> Result<ErrorHandlerResponse<B>, actix_web::Error> {
    move |res: ServiceResponse<B>| {
        // Handlers serialize their own message (e.g. the login failure text)
        // and always send `Content-Type: application/json`. Errors raised by
        // the framework itself arrive with no Content-Type at all. Only fill
        // in the untyped ones, so handler text survives. (Content-Length is
        // not usable here: Actix adds it after the middleware chain runs.)
        let is_json = res
            .response()
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .is_some_and(|v| v.starts_with("application/json"));
        if is_json {
            let (req, response) = res.into_parts();
            return Ok(ErrorHandlerResponse::Response(ServiceResponse::new(
                req,
                response.map_into_left_body(),
            )));
        }

        let (req, response) = res.into_parts();
        let response = response
            .set_body(
                HttpResponse::build(status)
                    .json(NoDataResponse::new(
                        message_for(status).to_string(),
                        status.as_u16() as i32,
                    ))
                    .into_body(),
            )
            .map_into_boxed_body()
            .map_into_right_body();
        Ok(ErrorHandlerResponse::Response(ServiceResponse::new(
            req, response,
        )))
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // Fail fast at boot rather than inside whichever worker serves the first
    // authenticated request. Without SECRET_KEY the session-token hash panics,
    // and the resulting 500s look like an auth problem rather than a config one.
    if let Err(err) = hash::ensure_secret_configured() {
        log::error!("{err}");
        std::process::exit(1);
    }

    // Once here, not per worker: `policy()` is built inside the factory closure.
    cors::log_scope();

    // Bind address comes from the environment so the same binary runs in
    // development, LAN, and production without recompiling.
    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(4343);

    let pool = match db::initialize_db_pool().await {
        Ok(pool) => {
            log::info!("database connected, migrations applied");
            pool
        }
        Err(err) => {
            log::error!("failed to connect to database: {err:?}");
            std::process::exit(1);
        }
    };

    log::info!("server starting on http://{host}:{port}");

    // Actix installs its own SIGTERM handler and drains in-flight requests on
    // shutdown, but it cannot stop the ZKNet tasks: those run on the tokio
    // runtime, whose blocking socket readers keep it alive past the drain. Take
    // the signal over so one flag reaches both the server and the device tasks.
    shutdown::install();

    // One hub for the whole server, shared by every worker: a hub created inside
    // the HttpServer factory would give each worker its own channel, so a client
    // on worker 3 would never see an event broadcast from worker 7.
    let ws_hub = wshub::WsHub::new();

    // Pull task registry: track active ZKNet device pull loops
    let pull_registry: PullRegistry = std::sync::Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new()));

    // Live task registry: one long-lived streaming session per ZKNet device.
    //
    // A device serves one client at a time, so a device in live mode must not
    // also be polled: the poller would be refused, and worse, the live socket
    // would be the one dropped. Mode is chosen per device below.
    let live_registry: LiveRegistry = std::sync::Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new()));

    // Device states go stale on their own: nothing ever flipped a controller
    // back to offline, so the dashboard's counter was stuck. The sweeper
    // compares each device's last contact against the 5-minute timeout once a
    // minute and pushes a `device` event when a state flips.
    actix_web::rt::spawn(device_sweeper::run_sweeper(pool.clone(), ws_hub.clone()));

    let app_state = web::Data::new(AppState {
        db: pool.clone(),
        ws: ws_hub.clone(),
    });

    // Initialize ZKNet pull scheduler: spawn background tasks for all ZKNET devices
    pull_scheduler::init_pull_scheduler(app_state.clone(), web::Data::new(pull_registry.clone())).await;

    // Initialize ZKNet live scheduler: stream punches instead of polling. Devices
    // listed here are excluded from the poller above, because both would contend
    // for the same single client slot on the device.
    live_scheduler::init_live_scheduler(app_state.clone(), web::Data::new(live_registry.clone())).await;

    let server = HttpServer::new(move || {
        let cors = cors::policy();

        App::new()
            // add DB pool handle to app data; enables use of `web::Data<DbPool>` extractor
            .app_data(web::Data::new(AppState {
                db: pool.clone(),
                ws: ws_hub.clone(),
            }))
            .app_data(web::Data::new(pull_registry.clone()))
            // The pull endpoint has to hand the device socket over: a manual pull
            // stops the live session first and restarts it afterwards, so it needs
            // the same registry the scheduler owns.
            .app_data(web::Data::new(live_registry.clone()))
            // 1 MiB: the salary CSV import travels inside a JSON string, and a
            // 300-employee export is ~73 KiB - the old 32 KiB cap rejected real
            // files with a 413 before the handler ran. The import handler also
            // caps the row count, so this is not an open door.
            .app_data(web::JsonConfig::default().limit(1024 * 1024))
            .app_data(
                bearer::Config::default()
                    .realm("restricted area")
                    .scope("login user"),
            )
            // Give every framework-raised error a JSON body matching the
            // handlers' envelope, so the frontend never gets an empty body.
            .wrap(
                ErrorHandlers::new()
                    .handler(
                        StatusCode::UNAUTHORIZED,
                        json_error(StatusCode::UNAUTHORIZED),
                    )
                    .handler(StatusCode::FORBIDDEN, json_error(StatusCode::FORBIDDEN))
                    .handler(StatusCode::NOT_FOUND, json_error(StatusCode::NOT_FOUND))
                    .handler(
                        StatusCode::PAYLOAD_TOO_LARGE,
                        json_error(StatusCode::PAYLOAD_TOO_LARGE),
                    )
                    .handler(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        json_error(StatusCode::INTERNAL_SERVER_ERROR),
                    ),
            )
            // add request logger middleware
            // `/api/ws` carries the session token in its query string; the default
    // logger prints the whole request line.
    .wrap(middleware::Logger::default().exclude("/api/ws"))
            // add authentication middleware
            //.wrap(auth)
            // add route handlers
            .wrap(cors)
            .configure(router::config)
    })
    //.bind(("10.55.54.145", 8080))?
    .bind((host.as_str(), port))?
    // Actix's own SIGTERM handling stops the server but leaves the ZKNet
    // background tasks running on the tokio runtime, and the process outlives
    // the server waiting for them. `shutdown::install` owns the signal instead,
    // so this server is stopped from the shutdown task below.
    .disable_signals()
    .run();

    let server_handle = server.handle();

    // On SIGTERM: stop accepting requests, let the device tasks drop their
    // sockets, then exit. Actix's graceful stop waits for in-flight requests,
    // and a dashboard holding an open WebSocket would hold it open indefinitely,
    // so the deadline below is the only thing that guarantees a prompt exit.
    actix_web::rt::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
            if shutdown::triggered() {
                break;
            }
        }

        log::info!("shutdown signal received, closing server");
        // Non-graceful: a graceful stop waits for every open worker connection,
        // and the device sockets are held open by the blocking readers, so the
        // wait would never end. Device sockets are released by the schedulers
        // seeing the flag; the HTTP server has no state worth draining.
        server_handle.stop(false).await;

        // Give the schedulers time to see the flag and close their device
        // sockets. A live session blocks in a socket read and only notices
        // between polls, so this wait is bounded by the read timeout rather
        // than by how long the device takes to answer.
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        log::info!("shutdown complete");
        log::logger().flush();
        std::process::exit(0);
    });

    server.await
}
