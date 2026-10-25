use crate::{
    service::{live_scheduler, pull_scheduler::is_live_device},
    utility::{
        authorization::{check_action_access, Action},
        stor::{AppState, GenericResponse, IntoResponse, KeyValResponse, NoDataResponse},
        zkpull::{AttendanceRow, ZkPull, decode_device_time},
        wshub::WsOut,
    },
};
use actix_web::{post, web, web::ReqData, Responder};
use std::time::Duration;

/// Restarts a live session once a manual pull is done with the device.
///
/// The pull has to stop the live session first, because a device serves one
/// client at a time. This guard puts the session back on every exit path -
/// including the error returns and an unreadable payload - without each one
/// having to remember to do it. A left-cancelled device would never come back
/// on its own, because the reconnect loop belongs to the task that was removed.
struct ResumeLive {
    registry: Option<web::Data<crate::service::live_scheduler::LiveRegistry>>,
    pool: web::Data<AppState>,
    device_id: i32,
}

impl Drop for ResumeLive {
    fn drop(&mut self) {
        if let Some(registry) = &self.registry {
            let registry = registry.clone();
            let pool = self.pool.clone();
            let device_id = self.device_id;
            // Drop runs outside the async context, so the task is scheduled instead
            // of awaited. spawn_live_task reconnects on its own if the device is down.
            actix_web::rt::spawn(async move {
                live_scheduler::spawn_live_task(device_id, pool, registry).await;
            });
        }
    }
}

/// Pull the attendance table off a device and store it in `enrolls`.
///
/// The device already pushes over ADMS, but a firmware that lost its server
/// address keeps its 9000 historical punches to itself forever. Pulling is the
/// recovery path, so this is a manual operator action rather than a scheduled
/// job: the read takes about a minute and the device refuses sessions while it
/// is busy.
///
/// Everything stored here matches the push path's contract exactly, so a report
/// cannot tell the two apart: `enroll_employee_id` is the device-side user id
/// (an ORPHAN, not a foreign key - the device holds more users than the
/// employee table has), `enroll_time` is a UTC epoch built by subtracting the
/// device offset once, `enroll_type` is 1 (Fingerprint), and `enroll_op_code`
/// keeps the raw wire code. The pull payload carries no verify mode, so the
/// type is a constant rather than a guess from a field that does not exist.
#[post("/device/pull/{id}")]
pub async fn device_pull(
    path: web::Path<i32>,
    pool: web::Data<AppState>,
    live_registry: web::Data<crate::service::live_scheduler::LiveRegistry>,
    bearer: Option<ReqData<String>>,
) -> impl Responder {
    let id = path.into_inner();

    if let Ok(resp) = check_action_access(pool.db.clone(), bearer, "controller", Action::Edit).await
    {
        return resp;
    }

    let (ip, sn, timezone, handler_id) = match sqlx::query_as::<_, (Option<String>, String, i32, i32)>(
        "SELECT device_ip, device_sn, device_timezone, device_handler_id FROM devices WHERE device_id = $1",
    )
    .bind(id)
    .fetch_optional(&pool.db)
    .await
    {
        Ok(Some(row)) => row,
        Ok(None) => {
            return NoDataResponse::new("Device not found".to_string(), 404).into_response();
        }
        Err(e) => {
            log::error!("device {id} lookup failed: {e}");
            return NoDataResponse::new("Internal error, please try again!".to_string(), 500)
                .into_response();
        }
    };

    // ZKNet pull only: reject ADMS devices (handler_id=4)
    if handler_id == 4 {
        return NoDataResponse::new("Device is configured for ADMS, not ZKNet".to_string(), 403).into_response();
    }

    // A live device serves one client at a time, so a manual pull cannot just open
    // a second socket: the device would refuse it and the request would hang for
    // the full read timeout. Hand the socket over instead - stop the live session,
    // take the backlog, then bring the live session back.
    //
    // This is the recovery path, not a convenience: when the network is down for a
    // long time the device keeps its punches to itself and streaming delivers
    // nothing, so pulling is the only way to get that backlog.
    //
    // Live mode is `LIVE_DEVICE_IDS`, not `handler_id`: a ZKNet device that is not
    // streaming has no live task to stop, and treating it as live would start one,
    // leaving the poller and a new live session fighting over the same socket.
    // Held until the end of the handler on purpose: its Drop restarts the live
    // session, so it must not be dropped here.
    let _resume_live = if is_live_device(id) {
        live_scheduler::cancel_live_task(&live_registry, id).await;
        ResumeLive { registry: Some(live_registry.clone()), pool: pool.clone(), device_id: id }
    } else {
        ResumeLive { registry: None, pool: pool.clone(), device_id: id }
    };

    // An administratively disabled device (state 3) must not be pulled from.
    let state: Option<i32> = match sqlx::query_scalar::<_, i32>(
        "SELECT device_state FROM devices WHERE device_id = $1",
    )
    .bind(id)
    .fetch_optional(&pool.db)
    .await
    {
        Ok(v) => v,
        Err(e) => {
            log::error!("device {id} state lookup failed: {e}");
            return NoDataResponse::new("Internal error, please try again!".to_string(), 500)
                .into_response();
        }
    };

    if state == Some(3) {
        return NoDataResponse::new("Device is disabled".to_string(), 403).into_response();
    }

    let Some(raw_ip) = ip.filter(|v| !v.trim().is_empty()) else {
        return KeyValResponse::<&str, String>::new(
            [("ip", "Device IP not set".to_string())].into(),
            "Cannot pull device".to_string(),
            400,
        )
        .into_response();
    };

    let addr = match raw_ip.trim().parse::<std::net::IpAddr>() {
        Ok(ip) => std::net::SocketAddr::from((ip, 4370)),
        Err(_) => {
            return KeyValResponse::<&str, String>::new(
                [("ip", "Not a valid IP address".to_string())].into(),
                "Cannot pull device".to_string(),
                400,
            )
            .into_response();
        }
    };

    // The client is blocking std::net, and a full table read holds the device for
    // about a minute, so it must not run on a worker thread the runtime needs.
    let timeout = Duration::from_secs(180);
    let read = web::block(move || {
        let mut zk = ZkPull::connect(&addr, timeout)?;
        zk.pull_attendance()
    })
    .await;

    let (payload, records) = match read {
        Ok(Ok(pair)) => pair,
        Ok(Err(e)) => {
            log::error!("device {id} at {addr} pull failed: {e}");
            return NoDataResponse::new(format!("Device did not answer: {e}"), 502)
                .into_response();
        }
        Err(e) => {
            log::error!("device {id} pull worker failed: {e}");
            return NoDataResponse::new("Internal error, please try again!".to_string(), 500)
                .into_response();
        }
    };

    // The attendance payload starts with a 4-byte total_size word, then the records
// back to back. Skip that word and trust it over the byte count: the payload is
// 4 bytes longer than the records it announces, and dividing the full length by
// the record count gives a width that fits no known layout.
let body = match payload.len() >= 4 {
    true => {
        let announced = u32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]) as usize;
        // The word counts record bytes only, so the body is everything after it.
        if announced + 4 <= payload.len() {
            &payload[4..]
        } else {
            // Announced more than arrived: a truncated read.
            log::error!(
                "device {id} at {addr} truncated: announced {announced}, got {}",
                payload.len()
            );
            return NoDataResponse::new("Device sent a truncated payload".to_string(), 502)
                .into_response();
        }
    }
    false => {
        return NoDataResponse::new("Device sent a payload we cannot read".to_string(), 502)
            .into_response();
    }
};

    // Derive the record width from the data rather than assuming the 40-byte
    // layout: bytes-per-record must divide the byte count, otherwise the slice
    // below either panics on the tail or silently mis-reads every row.
    let step = match record_width(body.len(), records) {
        Some(step) => step,
        None => {
            log::error!(
                "device {id} at {addr} unreadable: {} bytes over {records} records (per={}/{})",
                body.len(),
                if records > 0 { body.len() / records as usize } else { 0 },
                if records > 0 { body.len() % records as usize } else { 0 }
            );
            return NoDataResponse::new("Device sent a payload we cannot read".to_string(), 502)
                .into_response();
        }
    };

    let rows = crate::utility::zkpull::decode_records(body, step);
    if rows.is_empty() {
        return NoDataResponse::ok("Device holds no attendance record".to_string()).into_response();
    }

    let (stored, duplicate, failed) =
        store_attendance(&pool, &sn, timezone, &rows).await;

    log::info!(
        "device {id} at {addr} pulled {records} records: {stored} stored, {duplicate} already known, {failed} failed"
    );

    // Contact proof, same reason as the push path: a successful read means the
    // machine is alive right now, and a device that only POSTs cdata would
    // otherwise stay marked offline.
    if let Ok(Some(id_row)) = sqlx::query_scalar::<_, i32>(
        "UPDATE devices SET device_state = 1, device_online = EXTRACT(EPOCH FROM NOW())::BIGINT WHERE device_id = $1 RETURNING device_id",
    )
    .bind(id)
    .fetch_optional(&pool.db)
    .await
    {
        let _ = id_row;
    }

    // Same signal the push path sends: a successful read means the machine is
    // alive right now. Best effort - a failed broadcast must never fail the pull.
    let _ = pool.ws.broadcast(WsOut {
        event: "device",
        message: "controller checked in".to_owned(),
        enroll_id: 0,
    });

    GenericResponse::ok(
        vec![PullReport {
            device: id,
            records,
            stored,
            duplicate,
            failed,
        }],
        format!(
            "Pulled {records} records: {stored} stored, {duplicate} already known, {failed} failed"
        ),
    )
    .into_response()
}

/// Width of one attendance record in bytes.
///
/// Only the widths this device family is known to emit are accepted, and the
/// byte count must divide exactly, so a truncated read is refused rather than
/// decoded into plausible-looking rubbish.
pub fn record_width(bytes: usize, records: i32) -> Option<usize> {
    if records <= 0 {
        return Some(0); // Device empty, no records to parse
    }
    let per = bytes / records as usize;
    let remainder = bytes % records as usize;
    // Allow tail padding up to 8 bytes (common in device protocols)
    if remainder > 8 {
        return None;
    }
    match per {
        8 | 16 | 40 => Some(per),
        _ => None,
    }
}

#[derive(serde::Serialize)]
pub struct PullReport {
    pub device: i32,
    pub records: i32,
    pub stored: i32,
    pub duplicate: i32,
    pub failed: i32,
}

/// Insert pulled punches, one transaction for the whole batch.
///
/// A single statement rather than a per-row loop: 9000 rows would otherwise
/// mean 9000 round trips, and the device is held for the whole read anyway.
/// The primary key is `(enroll_device_sn, enroll_employee_id, enroll_time)`, so a
/// re-pull of an overlapping window is a no-op rather than a duplicate.
///
/// `enroll_op_code` is left NULL on purpose: the pull payload carries no verify
/// mode, and inventing a 0 there would claim a fingerprint match that the
/// device never reported. `enroll_status` is 1 for the same reason the push path
/// uses it - a stored record is a valid record.
/// Set the device clock to the current wall-clock time.
///
/// The device keeps its own RTC and a wrong clock stamps every future punch
/// with a skewed timestamp, so this is the recovery path for a clock that
/// drifted. Uses the same socket-handover as a manual pull: a live device
/// serves one client at a time, so the live session is stopped first and
/// restarted on drop.
#[post("/device/settime/{id}")]
pub async fn device_set_time(
    path: web::Path<i32>,
    pool: web::Data<AppState>,
    live_registry: web::Data<crate::service::live_scheduler::LiveRegistry>,
    bearer: Option<ReqData<String>>,
) -> impl Responder {
    let id = path.into_inner();

    if let Ok(resp) = check_action_access(pool.db.clone(), bearer, "controller", Action::Edit).await
    {
        return resp;
    }

    let (ip, sn, timezone, handler_id) = match sqlx::query_as::<_, (Option<String>, String, i32, i32)>(
        "SELECT device_ip, device_sn, device_timezone, device_handler_id FROM devices WHERE device_id = $1",
    )
    .bind(id)
    .fetch_optional(&pool.db)
    .await
    {
        Ok(Some(row)) => row,
        Ok(None) => {
            return NoDataResponse::new("Device not found".to_string(), 404).into_response();
        }
        Err(e) => {
            log::error!("device {id} lookup failed: {e}");
            return NoDataResponse::new("Internal error, please try again!".to_string(), 500)
                .into_response();
        }
    };

    if handler_id == 4 {
        return NoDataResponse::new("Device is configured for ADMS, not ZKNet".to_string(), 403)
            .into_response();
    }

    let _resume_live = if is_live_device(id) {
        live_scheduler::cancel_live_task(&live_registry, id).await;
        ResumeLive { registry: Some(live_registry.clone()), pool: pool.clone(), device_id: id }
    } else {
        ResumeLive { registry: None, pool: pool.clone(), device_id: id }
    };

    let state: Option<i32> = match sqlx::query_scalar::<_, i32>(
        "SELECT device_state FROM devices WHERE device_id = $1",
    )
    .bind(id)
    .fetch_optional(&pool.db)
    .await
    {
        Ok(v) => v,
        Err(e) => {
            log::error!("device {id} state lookup failed: {e}");
            return NoDataResponse::new("Internal error, please try again!".to_string(), 500)
                .into_response();
        }
    };

    if state == Some(3) {
        return NoDataResponse::new("Device is disabled".to_string(), 403).into_response();
    }

    let Some(raw_ip) = ip.filter(|v| !v.trim().is_empty()) else {
        return KeyValResponse::<&str, String>::new(
            [("ip", "Device IP not set".to_string())].into(),
            "Cannot set time".to_string(),
            400,
        )
        .into_response();
    };

    let addr = match raw_ip.trim().parse::<std::net::IpAddr>() {
        Ok(ip) => std::net::SocketAddr::from((ip, 4370)),
        Err(_) => {
            return KeyValResponse::<&str, String>::new(
                [("ip", "Not a valid IP address".to_string())].into(),
                "Cannot set time".to_string(),
                400,
            )
            .into_response();
        }
    };

    let timeout = Duration::from_secs(30);
    let tz = timezone;
    let result = web::block(move || {
        let local = chrono::Utc::now()
            .naive_utc()
            + chrono::Duration::hours(tz as i64);
        let encoded = ZkPull::encode_device_time(&local);
        ZkPull::set_device_time(&addr, timeout, encoded)
    })
    .await;

    match result {
        Ok(Ok(())) => GenericResponse::<&str>::single(
            "ok",
            "Device time set".to_string(),
        )
        .into_response(),
        Ok(Err(e)) => {
            log::error!("device {id} set_time failed: {e}");
            NoDataResponse::new(format!("Set time failed: {e}"), 500).into_response()
        }
        Err(e) => {
            log::error!("device {id} set_time join failed: {e}");
            NoDataResponse::new("Internal error".to_string(), 500).into_response()
        }
    }
}

pub async fn store_attendance(
    pool: &web::Data<AppState>,
    sn: &str,
    timezone: i32,
    rows: &[AttendanceRow],
) -> (i32, i32, i32) {
    let offset = (timezone as i64) * 3600;

    let mut stored = 0i32;
    let mut duplicate = 0i32;
    let mut failed = 0i32;
    let mut query = sqlx::QueryBuilder::<sqlx::Postgres>::new(
        r#"INSERT INTO enrolls (enroll_device_sn, enroll_employee_id, enroll_type, enroll_time, enroll_status, enroll_op_code)
           VALUES "#,
    );

    let mut bound = 0usize;
    let mut first = true;
    for row in rows {
        // Same as the push path: a non-numeric id is machine noise, not a punch.
        if row.user_id.parse::<i32>().unwrap_or(0) <= 0 {
            failed += 1;
            continue;
        }
        let (year, month, day, hour, minute, second) = decode_device_time(row.device_time);
        // A device time that decodes outside a plausible year means the layout
        // or the codec is wrong, not that the machine punched in 1990. Storing it
        // would fabricate history; dropping it with a count keeps the loss visible.
        if !(2000..=2038).contains(&year) {
            failed += 1;
            continue;
        }
        let local = chrono::NaiveDate::from_ymd_opt(year, month, day)
            .and_then(|d| d.and_hms_opt(hour, minute, second));
        let Some(local) = local else {
            failed += 1;
            continue;
        };
        // The device sends wall-clock local time; shift to UTC exactly once.
        let epoch = local.and_utc().timestamp() - offset;
        
        if !first {
            query.push(", ");
        }
        query.push("(");
        query.push_bind(sn.to_string());
        query.push(", ");
        query.push_bind(row.user_id.parse::<i32>().unwrap_or(0));
        query.push(", ");
        query.push_bind(1i32);
        query.push(", ");
        query.push_bind(epoch);
        query.push(", ");
        query.push_bind(1i32);
        query.push(", ");
        query.push_bind(None::<i32>);
        query.push(")");
        first = false;
        bound += 1;
    }

    // Nothing to insert (every row was implausible or non-numeric) - do not
    // send empty VALUES.
    if bound == 0 {
        return (0, 0, failed);
    }

    query.push(" ON CONFLICT (enroll_device_sn, enroll_employee_id, enroll_time) DO NOTHING");

    match query.build().execute(&pool.db).await {
        Ok(res) => {
            stored = res.rows_affected() as i32;
            // The unique key absorbs re-pulls, so rows_affected counts only new
            // rows. Everything else bound was a duplicate by definition.
            duplicate = (bound as i32 - stored).max(0);
        }
        Err(e) => {
            log::error!("device {sn} bulk insert failed: {e}");
            failed += bound as i32;
        }
    }
    (stored, duplicate, failed)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Strip the 4-byte total_size word the device prefixes to the table.
    ///
    /// Kept as a helper so the header rule has one home and one test. The word
    /// counts the record bytes only, so the payload is four bytes longer and
    /// dividing the raw length by the record count yields a width that matches no
    /// known layout.
    fn body_of(payload: &[u8]) -> Option<&[u8]> {
        let announced =
            u32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]) as usize;
        (payload.len() >= 4 && announced + 4 <= payload.len()).then(|| &payload[4..])
    }

    #[test]
    fn size_header_is_four_bytes_not_a_record() {
        // 4 records x 40 bytes = 160, so the payload is 164 bytes.
        let mut payload = 160u32.to_le_bytes().to_vec();
        payload.extend(std::iter::repeat(0u8).take(160));

        let body = body_of(&payload).expect("header announces a complete table");
        assert_eq!(body.len(), 160);
        assert_eq!(record_width(body.len(), 4), Some(40));
    }

    #[test]
    fn raw_length_over_record_count_is_the_wrong_width() {
        // The bug this guards: dividing the full payload gave 41, which matches
        // no layout and refused a perfectly good read.
        assert_eq!(record_width(164, 4), None);
    }

    #[test]
    fn announced_longer_than_received_is_truncated() {
        let payload = 900u32.to_le_bytes().to_vec();
        assert!(body_of(&payload).is_none());
    }
}