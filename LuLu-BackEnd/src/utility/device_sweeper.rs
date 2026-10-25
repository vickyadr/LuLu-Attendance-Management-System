use sqlx::PgPool;
use std::time::Duration;

use crate::utility::authorization::SESSION_IDLE_SECS;
use crate::utility::wshub::{WsHub, WsOut};

/// A device is considered offline after this long without contact.
const OFFLINE_AFTER_SECS: i64 = 300; // 5 minutes
/// How often the sweeper runs.
const SWEEP_EVERY: Duration = Duration::from_secs(60);

/// Keep `devices.device_state` in sync with `devices.device_online`.
///
/// Nothing in the codebase ever set a device back to offline: `getrequest` and
/// `cdata` only ever mark a device as online, so a controller that was unplugged
/// kept reporting "Online" forever and the dashboard's device counter was stuck.
///
/// This compares the last contact timestamp against a timeout, which is what
/// "online" actually means for a device that only signals by talking to the
/// server. Devices in state 3 (disabled) are left alone - an administrator
/// turned them off on purpose and a heartbeat must not switch them back on.
pub async fn run_sweeper(pool: PgPool, hub: WsHub) {
    let mut ticker = actix_web::rt::time::interval(SWEEP_EVERY);
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

    loop {
        ticker.tick().await;

        // Only devices whose derived state differs from the stored one are
        // updated, so `rows_affected` doubles as "how many flipped".
        let flipped: Result<u64, sqlx::Error> = sqlx::query(
            r#"UPDATE devices
                  SET device_state = CASE
                        WHEN device_online > EXTRACT(EPOCH FROM NOW())::BIGINT - $1 THEN 1
                        ELSE 2
                      END
                WHERE device_state <> 3
                  AND device_state IS DISTINCT FROM CASE
                        WHEN device_online > EXTRACT(EPOCH FROM NOW())::BIGINT - $1 THEN 1
                        ELSE 2
                      END"#,
        )
        .bind(OFFLINE_AFTER_SECS)
        .execute(&pool)
        .await
        .map(|res| res.rows_affected());

        match flipped {
            Ok(0) => {}
            Ok(n) => {
                log::info!("device sweeper: {n} controller(s) changed state");
                // Tell the dashboard so the device card updates without a reload.
                let receivers = hub.broadcast(WsOut {
                    event: "device",
                    message: format!("{n} controller(s) changed state"),
                    enroll_id: 0,
                });
                log::info!("ws broadcast device -> {receivers} client(s)");
            }
            Err(err) => log::error!("device sweeper failed: {err}"),
        }

        // Drop session rows that can no longer authenticate anything.
        //
        // The lookup rejects them now, but nothing deleted them, so the table
        // only ever grew and kept a hashed token for every login ever made.
        // The grace window is three times the idle timeout, so a session that
        // expired a moment ago is still recoverable if the sweeper is what
        // noticed - there is no user-visible difference either way, and deleting
        // early could race a request that is mid-refresh.
        match sqlx::query(
            r#"DELETE FROM sessions
               WHERE update_at < EXTRACT(EPOCH FROM NOW())::BIGINT - $1"#,
        )
        .bind(SESSION_IDLE_SECS * 3)
        .execute(&pool)
        .await
        {
            Ok(result) => {
                let n = result.rows_affected();
                if n > 0 {
                    log::info!("session sweeper: {n} expired session(s) removed");
                }
            }
            Err(err) => log::error!("session sweeper failed: {err}"),
        }
    }
}
