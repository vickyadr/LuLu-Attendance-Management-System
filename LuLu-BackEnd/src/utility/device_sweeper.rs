use sqlx::PgPool;
use std::time::Duration;

use crate::utility::authorization::SESSION_IDLE_SECS;
use crate::utility::wshub::{WsHub, WsOut};

/// A device is considered offline after this long without contact.
const OFFLINE_AFTER_SECS: i64 = 300; // 5 minutes
/// How often the sweeper runs.
const SWEEP_EVERY: Duration = Duration::from_secs(60);

pub async fn run_sweeper(pool: PgPool, hub: WsHub) {
    let mut ticker = actix_web::rt::time::interval(SWEEP_EVERY);
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

    loop {
        ticker.tick().await;

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
