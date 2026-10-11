use crate::utility::stor::AppState;
use crate::utility::zkpull::ZkPull;
use actix_web::web;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio::time::Duration;

pub type PullRegistry = Arc<Mutex<HashMap<i32, JoinHandle<()>>>>;

// Pause between pulls. This is measured AFTER the read finishes, not as a fixed
// cadence: the device serves one client at a time, so a tick that fires while the
// previous read is still holding the slot gets refused and the next read starts
// from a connection that never got a session.
const IDLE_GAP: Duration = Duration::from_secs(30);
// Matches the handler's read timeout: a full table read needs well over a minute.
const READ_TIMEOUT: Duration = Duration::from_secs(180);

// Devices in live mode stream punches and hold the device's only client slot for
// as long as they run, so the poller must leave them alone. Live ids live in a
// constant list for now; add a `device_mode` column when a device actually needs
// switching at runtime.
pub const LIVE_DEVICE_IDS: &[i32] = &[26];

pub fn is_live_device(device_id: i32) -> bool {
    LIVE_DEVICE_IDS.contains(&device_id)
}

pub async fn init_pull_scheduler(pool: web::Data<AppState>, registry: web::Data<PullRegistry>) {
    // Only poll devices that are not streaming.
    match sqlx::query_scalar::<_, i32>(
        "SELECT device_id FROM devices WHERE device_handler_id = 5 AND device_id <> ALL($1)",
    )
    .bind(LIVE_DEVICE_IDS)
    .fetch_all(&pool.db)
    .await
    {
        Ok(device_ids) => {
            let count = device_ids.len();
            for device_id in device_ids {
                spawn_pull_task(device_id, pool.clone(), registry.clone()).await;
            }
            log::info!("Pull scheduler: {count} ZKNet devices queued");
        }
        Err(e) => {
            log::error!("Pull scheduler init failed: {e}");
        }
    }
}

pub async fn spawn_pull_task(
    device_id: i32,
    pool: web::Data<AppState>,
    registry: web::Data<PullRegistry>,
) {
    cancel_pull_task(&registry, device_id).await;

    let pool_clone = pool.clone();
    let handle = tokio::spawn(async move {
        loop {
            let row: Option<(Option<String>, String, i32)> =
                match sqlx::query_as(
                    "SELECT device_ip, device_sn, device_timezone FROM devices WHERE device_id = $1 AND device_handler_id = 5 AND device_state <> 3",
                )
                .bind(device_id)
                .fetch_optional(&pool_clone.db)
                .await
                {
                    Ok(r) => r,
                    Err(e) => {
                        log::error!("pull scheduler device {device_id}: db lookup failed: {e}");
                        tokio::time::sleep(IDLE_GAP).await;
                        continue;
                    }
                };

            let (ip, sn, tz) = match row {
                Some(r) => r,
                // Device deleted, moved to ADMS, or disabled: keep the task cheap
                // and idle instead of reconnecting to a slot nobody wants.
                None => {
                    log::debug!(
                        "pull scheduler device {device_id}: no longer pullable, idling"
                    );
                    tokio::time::sleep(IDLE_GAP).await;
                    continue;
                }
            };

            let Some(raw_ip) = ip.filter(|v| !v.trim().is_empty()) else {
                tokio::time::sleep(IDLE_GAP).await;
                continue;
            };

            let addr = match raw_ip.trim().parse::<std::net::IpAddr>() {
                Ok(a) => std::net::SocketAddr::from((a, 4370)),
                Err(_) => {
                    log::error!(
                        "pull scheduler device {device_id}: '{raw_ip}' is not an IP, skipping"
                    );
                    tokio::time::sleep(IDLE_GAP).await;
                    continue;
                }
            };

            // Connect, pull, disconnect per cycle. The reader is blocking std::net
            // and holds the device for over a minute, so it must not sit on a
            // runtime worker thread.
            let sn_for_log = sn.clone();
            let read = tokio::task::spawn_blocking(move || {
                let mut zk = ZkPull::connect(&addr, READ_TIMEOUT)?;
                zk.pull_attendance()
            })
            .await;

            match read {
                Ok(Ok((payload, record_count))) => {
                    // The table is prefixed with a 4-byte record-byte count, so
                    // the raw length over the record count yields a width that
                    // matches no layout. Skip the word and use what follows.
                    if payload.len() < 4 {
                        log::error!(
                            "pull scheduler device {device_id} ({sn_for_log}): {} byte payload has no size header",
                            payload.len(),
                        );
                        tokio::time::sleep(IDLE_GAP).await;
                        continue;
                    }
                    let announced = u32::from_le_bytes([
                        payload[0],
                        payload[1],
                        payload[2],
                        payload[3],
                    ]) as usize;
                    if announced + 4 > payload.len() {
                        log::error!(
                            "pull scheduler device {device_id} ({sn_for_log}): truncated, announced {announced} in {}",
                            payload.len(),
                        );
                        tokio::time::sleep(IDLE_GAP).await;
                        continue;
                    }
                    let body = &payload[4..];
                    let step = crate::handler::h_zkpull::record_width(body.len(), record_count);
                    let Some(step) = step else {
                        log::error!(
                            "pull scheduler device {device_id} ({sn_for_log}): {} bytes over {record_count} records, unreadable",
                            body.len(),
                        );
                        tokio::time::sleep(IDLE_GAP).await;
                        continue;
                    };
                    let records = crate::utility::zkpull::decode_records(body, step);
                    let (stored, duplicate, failed) =
                        crate::handler::h_zkpull::store_attendance(&pool_clone, &sn, tz, &records)
                            .await;
                    log::info!(
                        "pull scheduler device {device_id} ({sn_for_log}): {record_count} records, {stored} stored, {duplicate} duplicate, {failed} failed"
                    );
                }
                Ok(Err(e)) => {
                    log::error!(
                        "pull scheduler device {device_id} ({sn_for_log}) at {addr} pull failed: {e}"
                    );
                }
                Err(e) => {
                    log::error!("pull scheduler device {device_id} worker panicked: {e}");
                }
            }

            tokio::time::sleep(IDLE_GAP).await;
        }
    });

    registry.lock().await.insert(device_id, handle);
}

pub async fn cancel_pull_task(registry: &web::Data<PullRegistry>, device_id: i32) {
    if let Some(handle) = registry.lock().await.remove(&device_id) {
        handle.abort();
    }
}
