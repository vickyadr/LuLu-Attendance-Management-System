use crate::handler::h_zkpull::store_attendance;
use crate::utility::stor::AppState;
use crate::utility::zkpull::{AttendanceRow, ZkPull};
use actix_web::web;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

/// Live capture tasks, one per device. A live session owns the device socket for
/// as long as it runs, so there is at most one per device and it is what owns the
/// device's single client slot.
use std::sync::atomic::{AtomicBool, Ordering};

pub struct LiveTask {
    handle: tokio::task::JoinHandle<()>,
    stop: Arc<AtomicBool>,
}

pub type LiveRegistry = Arc<Mutex<std::collections::HashMap<i32, LiveTask>>>;

// Gap before a dropped live session is retried. The device needs a moment to
// release its client slot after a socket closes, and reconnecting too eagerly
// just gets refused.
const RECONNECT_GAP: Duration = Duration::from_secs(10);

// Upper bound on the backoff gap. A device that is genuinely gone must not push
// the retry out so far that it stays dark for minutes after the network returns.
const RECONNECT_GAP_MAX: Duration = Duration::from_secs(60);

// Socket timeout for the live socket. Must exceed the idle window so a quiet
// device reads as a timeout we handle, not as a dead connection.
const LIVE_READ_TIMEOUT: Duration = Duration::from_secs(180);

use tokio::time::Duration;

pub async fn init_live_scheduler(pool: web::Data<AppState>, registry: web::Data<LiveRegistry>) {
    match sqlx::query_scalar::<_, i32>(
        "SELECT device_id FROM devices WHERE device_handler_id = 5 AND device_id = ANY($1)",
    )
    .bind(crate::service::pull_scheduler::LIVE_DEVICE_IDS)
    .fetch_all(&pool.db)
    .await
    {
        Ok(device_ids) => {
            let count = device_ids.len();
            for device_id in device_ids {
                spawn_live_task(device_id, pool.clone(), registry.clone()).await;
            }
            log::info!("Live scheduler: {count} ZKNet devices queued");
        }
        Err(e) => {
            log::error!("Live scheduler init failed: {e}");
        }
    }
}

pub async fn spawn_live_task(
    device_id: i32,
    pool: web::Data<AppState>,
    registry: web::Data<LiveRegistry>,
) {
    cancel_live_task(&registry, device_id).await;

    let pool_clone = pool.clone();
    let stop = Arc::new(AtomicBool::new(false));
    let stop_task = stop.clone();
    let handle = tokio::spawn(async move {
        // Grows while the device stays unreachable, collapses the moment a
        // session actually opens. Without this a device that is off for an hour
        // would be retried every 10s forever and a device that blipped would
        // wait out a stale long delay.
        let mut backoff = RECONNECT_GAP;
        let mut was_connected = false;
        loop {
            // If a prior session was cancelled, reset the flag for the next attempt.
            stop_task.store(false, Ordering::Relaxed);

            // A blocked socket read cannot be interrupted from outside, and the
            // reader only checks its flag between frames, so the process would
            // otherwise wait out the device's idle timeout before the socket is
            // released. On shutdown there is no next attempt, so leave here.
            if crate::utility::shutdown::triggered() {
                log::info!("live scheduler device {device_id}: shutdown, stopping");
                return;
            }

            let row: Option<(Option<String>, String, i32)> = match sqlx::query_as(
                "SELECT device_ip, device_sn, device_timezone FROM devices WHERE device_id = $1 AND device_handler_id = 5 AND device_state <> 3",
            )
            .bind(device_id)
            .fetch_optional(&pool_clone.db)
            .await
            {
                Ok(r) => r,
                Err(e) => {
                    log::error!("live scheduler device {device_id}: db lookup failed: {e}");
                    tokio::time::sleep(backoff).await;
                    continue;
                }
            };

            let (ip, sn, tz) = match row {
                Some(r) => r,
                None => {
                    log::debug!("live scheduler device {device_id}: no longer pullable, idling");
                    // Nothing to connect to until someone re-enables it, so do not
                    // climb the backoff: an operator fixing the row should see the
                    // session come back on the very next tick.
                    backoff = RECONNECT_GAP;
                    tokio::time::sleep(RECONNECT_GAP).await;
                    continue;
                }
            };

            let Some(raw_ip) = ip.filter(|v| !v.trim().is_empty()) else {
                tokio::time::sleep(backoff).await;
                continue;
            };

            let addr = match raw_ip.trim().parse::<std::net::IpAddr>() {
                Ok(a) => SocketAddr::from((a, 4370)),
                Err(_) => {
                    log::error!(
                        "live scheduler device {device_id}: '{raw_ip}' is not an IP, skipping"
                    );
                    backoff = (backoff * 2).min(RECONNECT_GAP_MAX);
                    tokio::time::sleep(backoff).await;
                    continue;
                }
            };

            let sn_for_log = sn.clone();

            // The reader is blocking std::net, so it runs on a blocking thread and
            // hands rows back over a channel. The async side owns the database.
            let (tx, mut rx) = mpsc::channel::<AttendanceRow>(256);
            // Aborting this task cannot stop a blocking thread, so the reader
            // needs its own stop flag: cancel signals it and the loop checks it
            // before each read. A process-wide shutdown is a second reason to
            // stop, and a blocking read cannot be cancelled from outside, so the
            // flag has to carry it or the socket is held until the poll expires.
            let stop_reader = stop_task.clone();
            let reader = tokio::task::spawn_blocking(move || {
                let mut zk = ZkPull::connect(&addr, LIVE_READ_TIMEOUT)?;
                // Both closures borrow `stop_reader`/`tx`; the reader thread owns
                // them for the whole session and the channel closes when it returns.
                let send = tx.clone();
                zk.live_capture(
                    || !stop_reader.load(Ordering::Relaxed) && !crate::utility::shutdown::triggered(),
                    |row| {
                        let _ = send.blocking_send(row);
                    },
                )
            });

            // Drain punches while the reader runs. `false` means the reader ended
            // and the session is gone. Once we start receiving, the device was reachable,
            // so reset backoff: transient failures on an otherwise live device do not grow
            // the wait time.
            was_connected = true;
            
            // The device just connected: refresh its last-contact timestamp so the
            // sweeper does not flip it to offline after 5 minutes of silence (even
            // though the socket is open). Every punch will refresh it again.
            let _ = sqlx::query(
                "UPDATE devices SET device_online = EXTRACT(EPOCH FROM NOW())::BIGINT, device_state = 1 WHERE device_id = $1 AND device_state <> 3",
            )
            .bind(device_id)
            .execute(&pool_clone.db)
            .await;
            
            let mut stored_total = 0u64;
            while let Some(row) = rx.recv().await {
                let (stored, _duplicate, failed) =
                    store_attendance(&pool_clone, &sn, tz, std::slice::from_ref(&row)).await;
                stored_total += stored as u64;
                // A live ZKNet device never phones home, so nothing else refreshes
                // its `device_online`. Without this the sweeper flipped it to
                // offline after 5 minutes even though punches were streaming in,
                // which is why the device page said "Offline" during live capture.
                let _ = sqlx::query(
                    "UPDATE devices SET device_online = EXTRACT(EPOCH FROM NOW())::BIGINT, device_state = 1 WHERE device_id = $1 AND device_state <> 3",
                )
                .bind(device_id)
                .execute(&pool_clone.db)
                .await;
                // Tell the dashboard a punch landed. Without this the live table
                // only ever changes on a manual reload: the pull path broadcasts
                // and the live path did not, so every real-time punch was silent.
                // Best effort - a failed broadcast must never fail the ingest.
                if stored > 0 {
                    // Info, not debug: a silent reader and a stopped reader look
                    // identical from the outside, and every "my punch did not
                    // arrive" question starts here. One line per punch is not
                    // flood on a real attendance device.
                    log::info!(
                        "live device {device_id} ({sn}): punch from uid {}, stored",
                        row.user_id
                    );
                    pool_clone.ws.broadcast(crate::utility::wshub::WsOut {
                        event: "enroll",
                        message: format!("punch from uid {}", row.user_id),
                        enroll_id: 0,
                    });
                }
                if failed > 0 {
                    // `failed` counts punches store_attendance refused: a
                    // non-numeric user id, or a device time outside 2000..2038.
                    // It does NOT mean the employee is unregistered, which reads
                    // as a much more alarming problem than it is.
                    log::warn!(
                        "live scheduler device {device_id} ({sn_for_log}): rejected punch from uid {} ({} unreadable)",
                        row.user_id,
                        failed
                    );
                }
            }

            match reader.await {
                Ok(Ok(delivered)) => log::info!(
                    "live scheduler device {device_id} ({sn_for_log}): session ended after {delivered} events, {stored_total} stored"
                ),
                Ok(Err(e)) => log::error!(
                    "live scheduler device {device_id} ({sn_for_log}) at {addr} live session failed: {e}"
                ),
                Err(e) => {
                    log::error!("live scheduler device {device_id} reader panicked: {e}");
                }
            }

            // The socket opened if we got past `connect`, which is the only thing
            // that distinguishes a device that is unreachable from a session that
            // merely dropped. Back off only on the former, and log the wait so a
            // device that stays dark is explainable from the log alone.
            if was_connected {
                log::info!(
                    "live scheduler device {device_id} ({sn_for_log}) at {addr}: reconnecting in {}s",
                    backoff.as_secs()
                );
                tokio::time::sleep(backoff).await;
            } else {
                backoff = (backoff * 2).min(RECONNECT_GAP_MAX);
                log::warn!(
                    "live scheduler device {device_id} ({sn_for_log}) at {addr} unreachable, next attempt in {}s",
                    backoff.as_secs()
                );
                tokio::time::sleep(backoff).await;
            }
            // A cancelled sleep means the process is stopping: the loop head
            // checks the shutdown flag and returns, so nothing reconnects after
            // the socket has been handed back.
        }
    });

    registry.lock().await.insert(device_id, LiveTask { handle, stop });
}

pub async fn cancel_live_task(registry: &web::Data<LiveRegistry>, device_id: i32) {
    if let Some(task) = registry.lock().await.remove(&device_id) {
        // Signal first: the blocking reader owns the socket and only checks this
        // flag between reads, so aborting alone would leave the session held.
        task.stop.store(true, Ordering::Relaxed);
        task.handle.abort();
    }
}