use crate::{
    receiver::r_cdata::*,
    utility::{
        stor::{AppState, IntoResponse, KeyValResponse},
        wshub::WsOut,
    },
};
use actix_web::{
    HttpResponse, Responder, get, post,
    web::{self, Bytes},
};
use anyhow::{Result, bail};
use chrono::{NaiveDateTime, Utc};
use std::collections::HashMap;

#[get("/cdata")]
pub async fn get_cdata(pool: web::Data<AppState>, data: web::Query<GetCData>) -> impl Responder {
    let sn = data.sn.as_deref().unwrap_or("");
    log::info!("CDATA: {:?}", if sn.is_empty() { "No SN" } else { sn });

    // ADMS config only: a ZKNET device (handler_id=5) must never receive this
    // payload. Realtime=1 tells the machine to push its punches to us, which
    // is the opposite of how ZKNET works (we pull) and would let a ZKNET device
    // bypass the pull isolation entirely.
    //
    // An SN we do not know is refused too: handing out the config would let any
    // machine on the network enrol itself into the attendance feed.
    let handler_id: Option<i32> =
        match sqlx::query_scalar::<_, i32>(r#"SELECT device_handler_id FROM devices WHERE device_sn = $1"#)
            .bind(sn)
            .fetch_optional(&pool.db)
            .await
        {
            Ok(v) => v,
            Err(e) => {
                log::error!("cdata handler lookup failed for {sn}: {e}");
                return HttpResponse::InternalServerError()
                    .content_type("text/plain")
                    .body("");
            }
        };

    let Some(handler_id) = handler_id else {
        log::warn!("cdata refused: {sn} is not a registered device");
        return HttpResponse::Forbidden().content_type("text/plain").body("");
    };

    if handler_id == 5 {
        return HttpResponse::Forbidden().content_type("text/plain").body("");
    }

    let tz =
        sqlx::query_scalar::<_, i32>(r#"SELECT device_timezone FROM devices WHERE device_sn = $1"#)
            .bind(sn)
            .fetch_one(&pool.db.clone())
            .await
            .unwrap_or(0);

    // DeviceTalker option payload. `concat!` keeps each line free of the 24
    // leading spaces the old `\`-continued literals leaked into the payload.
    let text = format!(
        concat!(
            "GET OPTION FROM: {}\r\n",
            "Stamp=0\r\n",
            "OpStamp=0\r\n",
            "ErrorDelay=60\r\n",
            "Delay=30\r\n",
            "TransTimes=00:00;14:05\r\n",
            "TransInterval=1\r\n",
            "TransFlag=1000000000\r\n",
            "TimeZone={}\r\n",
            "Realtime=1\r\n",
            "Encrypt=0"
        ),
        data.sn.as_deref().unwrap_or(""),
        tz
    );

    HttpResponse::Ok().content_type("text/plain").body(text)
}

#[post("/cdata")]
pub async fn post_cdata(
    pool: web::Data<AppState>,
    get: web::Query<ReceiverCData>,
    body: Bytes,
) -> impl Responder {
    let mut data: HashMap<String, String> = HashMap::new();

    // A device body is not guaranteed to be UTF-8: a template or name in a legacy
    // single-byte encoding makes this decode fail, and the old `unwrap()` panicked
    // the worker -- every later request answered 500 until a restart. Recover the
    // text instead: the wire format is line/field oriented, so lossy decoding keeps
    // the structure and only mangles the offending character.
    let data_body = String::from_utf8_lossy(&body).into_owned();
    log::info!("POST CDATA: {:?}", data_body);

    if get.sn.is_none() {
        data.insert("sn".to_string(), "Serial Number cant be empty".to_string());
    }

    if get.table.is_none() {
        data.insert("table".to_string(), "Data table not found".to_string());
    }

    if get.stamp.is_none() && get.op_stamp.is_none() {
        data.insert("stamp".to_string(), "Timestamp cant be empty".to_string());
    }

    if !data.is_empty() {
        return KeyValResponse::<String, String>::new(data, "Post Error".to_string(), 400)
            .into_response();
    }

    // ADMS push only: reject ZKNET devices (handler_id=5), and reject SNs we do
    // not know. An unknown machine must not be able to inject punches, and a
    // failed lookup must fail closed rather than let the batch through.
    let push_handler: Option<i32> =
        match sqlx::query_scalar::<_, i32>(r#"SELECT device_handler_id FROM devices WHERE device_sn = $1"#)
            .bind(get.sn.as_deref().unwrap_or(""))
            .fetch_optional(&pool.db)
            .await
        {
            Ok(v) => v,
            Err(e) => {
                log::error!("cdata push lookup failed for {:?}: {e}", get.sn);
                return HttpResponse::InternalServerError()
                    .content_type("text/plain")
                    .body("");
            }
        };

    match push_handler {
        Some(5) => {
            return HttpResponse::Forbidden()
                .content_type("text/plain")
                .body("");
        }
        None => {
            log::warn!("cdata push refused: {:?} is not a registered device", get.sn);
            return HttpResponse::Forbidden()
                .content_type("text/plain")
                .body("");
        }
        Some(_) => {}
    }

    // An administratively disabled device (state 3) must stay dark: it may
    // still be talking to us, but nothing it sends is attendance data now.
    let disabled: Option<i32> =
        match sqlx::query_scalar::<_, i32>(r#"SELECT device_state FROM devices WHERE device_sn = $1"#)
            .bind(get.sn.as_deref().unwrap_or(""))
            .fetch_optional(&pool.db)
            .await
        {
            Ok(v) => v,
            Err(e) => {
                log::error!("cdata state lookup failed for {:?}: {e}", get.sn);
                return HttpResponse::InternalServerError()
                    .content_type("text/plain")
                    .body("");
            }
        };

    if disabled == Some(3) {
        return HttpResponse::Forbidden()
            .content_type("text/plain")
            .body("");
    }

    // A batch the server understood but could not store is a 400, not a 500. The
    // old code built the 400 inside `map_err` and then threw it away, falling
    // through to the InternalServerError arm below - so any upload whose rows
    // were all rejected answered 500, and a device that retries on 5xx would
    // resend the same bad buffer forever. `Err` is a handled outcome here, so
    // return the response it mapped to instead of discarding it.
    let (text, is_operlog) = if get.table.as_ref().unwrap() == "OPERLOG" {
        (data_operlog(pool, get, data_body).await, true)
    } else if get.table.as_ref().unwrap() == "ATTLOG" {
        (data_attlog(pool, get, data_body).await, false)
    } else {
        // A table this build does not implement. Tell the device plainly rather
        // than falling into the 500, so it stops resending it.
        return HttpResponse::BadRequest()
            .content_type("text/plain")
            .body("");
    };

    match text {
        Ok(body) => HttpResponse::Ok().content_type("text/plain").body(body),
        Err(e) => {
            let table = if is_operlog { "OPERLOG" } else { "ATTLOG" };
            log::error!("Error processing {}: {}", table, e);
            HttpResponse::BadRequest()
                .content_type("text/plain")
                .body("")
        }
    }
}

/// Push a "controller changed state" event to every live client. Best effort by
/// design: a failed broadcast must never turn a device upload into an error.
fn notify_device(pool: &AppState, message: &str) {
    let receivers = pool.ws.broadcast(WsOut {
        event: "device",
        message: message.to_owned(),
        enroll_id: 0,
    });
    log::info!("ws broadcast device -> {receivers} client(s)");
}

/// Mark a device as having just contacted the server.
///
/// An upload is proof of life for controllers that never poll `getrequest`, so
/// without this touch they would read as offline forever despite having just
/// reported data. State is set here rather than left to the sweeper so a
/// controller shows up as online immediately instead of up to a minute later.
///
/// Returns `true` when this contact flipped the device from offline to online,
/// which is the only transition worth pushing to the dashboard.
async fn touch_device(pool: &AppState, sn: &str) -> bool {
    let flipped: Option<(i64,)> = sqlx::query_as(
        r#"UPDATE devices
              SET device_online = EXTRACT(EPOCH FROM NOW())::BIGINT, device_state = 1
            WHERE device_sn = $1 AND device_state <> 3 AND device_state IS DISTINCT FROM 1
        RETURNING device_online"#,
    )
    .bind(sn)
    .fetch_optional(&pool.db)
    .await
    .ok()
    .flatten();

    // No row means either the device is unknown, disabled, or already online.
    // Only a real flip counts as news.
    let came_online = flipped.is_some();

    // Always refresh the timestamp, even when already online: the sweeper
    // decides online vs offline from `device_online`, not from the state alone.
    let _ = sqlx::query(
        r#"UPDATE devices SET device_online = EXTRACT(EPOCH FROM NOW())::BIGINT WHERE device_sn = $1 AND device_state <> 3"#,
    )
    .bind(sn)
    .execute(&pool.db)
    .await;

    came_online
}

/// Push a "new enroll" event to every live client. Best effort by design: a
/// failed broadcast must never turn a device upload into an error response.
fn notify_enroll(pool: &AppState, enroll_id: i32, recorded: i32) {
    let receivers = pool.ws.broadcast(WsOut {
        event: "enroll",
        message: format!("{recorded} new enroll record(s)"),
        enroll_id,
    });
    log::info!("ws broadcast enroll {enroll_id} -> {receivers} client(s)");
}

async fn data_attlog(
    pool: web::Data<AppState>,
    get: web::Query<ReceiverCData>,
    data_body: String,
) -> Result<String> {
    let log_data = data_body.lines();
    let mut log_recorded: i32 = 0;
    let mut log_failed: i32 = 0;

    for logs in log_data {
        if logs.is_empty() {
            continue;
        }

        // ATTLOG wire layout (ZKTeco PUSH SDK):
        //   UserID<HT>Timestamp<HT>Status<HT>VerifyMode<HT>WorkCode
        // so the canonical row is FIVE columns, e.g.
        //   `123<HT>2024-01-01 08:00:00<HT>0<HT>1<HT>0`
        // The old `log.len() < 6` guard rejected that canonical row outright, so
        // every real ATTLOG upload was discarded. Four columns is the true
        // minimum here: this handler needs UserID, Timestamp and VerifyMode.
        let log: Vec<&str> = logs.split("\t").collect();
        if log.len() < 4 {
            continue;
        }

        // VerifyMode (column 3) per the ADMS protocol table. The old mapping was
        // transposed - it read 2 as Password and 3 as Card, while the protocol
        // defines 2 as legacy Card and 3 as alternative Password - so card and
        // password enrollments were stored under the wrong type. Modes 5-8 are
        // multi-factor rows; the protocol lists the modalities in priority
        // order, so the first one named wins rather than dropping the row.
        //
        // The raw code is kept in `enroll_op_code` and an unrecognised one is
        // kept too (mapped to the "other" display type) rather than dropped:
        // the protocol defines codes this table does not enumerate, and a
        // dropped row is a lost punch that nothing downstream can diagnose.
        let op_code: Option<i32> = log[3].trim().parse::<i32>().ok();
        let enroll_type: i32 = match log[3].trim() {
            "0" => 2,  // Password
            "1" => 1,  // Fingerprint
            "2" => 3,  // Card (legacy)
            "3" => 2,  // Password (alternative)
            "4" => 3,  // Card
            "5" => 1,  // Fingerprint+Card
            "6" => 1,  // Fingerprint+Password
            "7" => 3,  // Card+Password
            "8" => 3,  // Card+Fingerprint+Password
            "15" => 4, // Face
            // Palm shares the biometric template storage path with fingerprint.
            "25" => 1,
            other => {
                // Visible, not silent: a code outside the table is recorded and
                // reported so the real device can be checked.
                log::warn!(
                    "attlog unmapped verify mode {} device={} user={}",
                    other,
                    get.sn.as_deref().unwrap_or(""),
                    log[0]
                );
                5 // Other / unmapped
            }
        };

        // Column 1 is the timestamp, and the protocol allows TWO formats for it:
        // `YYYY-MM-DD HH:MM:SS` in device-local time, or a Unix epoch. The old
        // `unwrap_or_default()` turned an epoch row into 1970-01-01 and INSERTED
        // it as a genuine punch, which is worse than dropping it - so an
        // unparseable timestamp must skip the row. An epoch is already UTC and
        // must not be shifted by the device offset; only the wall-clock form is
        // expressed in device-local time.
        let (mut dt, is_device_local) =
            match NaiveDateTime::parse_from_str(log[1], "%Y-%m-%d %H:%M:%S") {
                Ok(naive) => (naive.and_utc().timestamp(), true),
                Err(_) => match log[1].trim().parse::<i64>() {
                    Ok(epoch) => (epoch, false),
                    Err(_) => continue,
                },
            };

        if is_device_local
            && let Ok(device_timezone) = sqlx::query_scalar::<_, i32>(
                r#"SELECT device_timezone FROM devices WHERE device_sn = $1"#,
            )
            .bind(get.sn.as_ref().unwrap().to_string())
            .fetch_one(&pool.db.clone())
            .await
        {
            dt -= (3600 * device_timezone) as i64;
        }

        // Fingerprint template data arrives as tab-separated columns; a device
        // that sends a short or malformed row must be skipped, never panic -
        // an unwrap here would kill the worker process.
        let user_id: i32 = match log[0].parse() {
            Ok(id) => id,
            Err(_) => continue,
        };
        if user_id <= 0 {
            continue;
        }

        // Count each row exactly once and tell a real failure from a tolerated
        // duplicate. The old `.map_err().map()` chain counted a duplicate
        // twice, counted a successful insert once, and silently dropped a
        // genuine error (type mismatch, FK violation) with no bump at all - so
        // a batch where every row failed still answered "OK 0" and the missing
        // punches were never noticed by the device.
        let row_result = sqlx::query(r#"INSERT INTO enrolls (enroll_device_sn, enroll_employee_id, enroll_type, enroll_time, enroll_status, enroll_op_code) VALUES ($1, $2, $3, $4, $5, $6)"#)
                .bind(get.sn.as_ref().unwrap_or(&String::new()).to_string())
                .bind(user_id)
                .bind(enroll_type)
                .bind(dt)
                .bind(1)
                .bind(op_code)
                .execute(&pool.db)
                .await;

        match row_result {
            Ok(_) => {
                log_recorded += 1;
            }
            Err(e) => {
                // A device that resends its buffer after a dropped connection
                // must not be scolded, so the unique-constraint case still
                // counts as recorded. Anything else is a real loss.
                if e.to_string()
                    .contains("duplicate key value violates unique constraint")
                {
                    log_recorded += 1;
                } else {
                    log_failed += 1;
                    log::error!("enroll insert failed employee={} err: {}", user_id, e);
                }
            }
        }
    }

    if log_recorded == 0 {
        if log_failed > 0 {
            bail!(
                "None of the {} uploaded row(s) could be stored - check the server log",
                log_failed
            );
        }
        bail!("Some data error: {}", log_recorded);
    }

    // Contact proof: an upload means the device is alive right now. Devices
    // that only POST cdata (never GET getrequest) would stay offline forever
    // without this touch, even though they just talked to the server.
    if touch_device(&pool, get.sn.as_deref().unwrap_or("")).await {
        notify_device(&pool, "controller checked in");
    }

    // One event per upload, not per row: a device posts a batch every few
    // seconds and a fan-out per row would flood the feed with duplicates.
    if let Ok(newest) =
        sqlx::query_scalar::<_, i32>("SELECT COALESCE(MAX(enroll_id), 0) FROM enrolls")
            .fetch_one(&pool.db)
            .await
    {
        notify_enroll(&pool, newest, log_recorded);
    }

    Ok(format!("OK {}", log_recorded))
}

/// A row decoded from an OPERLOG/OPLOG upload, before any database work.
///
/// Parsing is split out from the insert path so the wire format can be tested
/// without a database. Every bug fixed on this endpoint so far was a column
/// index or a token split, and none of it had a test: the column layout is the
/// contract with the device, so it needs one.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum OperRow {
    /// An attendance punch from the OPLOG table.
    Punch {
        user_id: i32,
        /// Mapped display type (1 Finger, 2 PIN, 3 Card, 5 unmapped).
        enroll_type: i32,
        /// Raw wire code, kept so an unmapped code can be identified from data.
        op_code: i32,
        /// Raw timestamp field exactly as the device sent it.
        raw_time: String,
    },
    /// A user record from the USER table - carries no timestamp of its own.
    User {
        user_id: i32,
        /// Name value, already stripped of its `Name=` label.
        name: String,
    },
    /// A fingerprint template from the FP table.
    Finger {
        user_id: i32,
        /// Finger index, already stripped of its `FID=` label.
        finger_id: String,
        /// Template payload, already stripped of its `TMP=` label.
        template: String,
    },
}

/// Table token at the head of a row.
///
/// Devices send both spellings: `OPLOG 70` (space) and `USER=7` / `FP=7`
/// (equals). Splitting on whitespace alone turned every `=`-form row into an
/// unknown type, so device renames and fingerprint uploads were dropped.
fn row_table(row: &str) -> &str {
    row.split(|c: char| c == '=' || c.is_whitespace())
        .next()
        .unwrap_or("")
}

/// Value after the first `=` in a labelled field.
///
/// The label is a single `=`: `Name=…` splits cleanly, but a base64 template
/// may itself end in `=`, and taking the last segment truncates it.
fn labelled(field: &str) -> &str {
    field.split_once('=').map_or(field, |(_, v)| v)
}

/// Parse one OPLOG attendance row.
///
/// Layout is `OPLOG<SP><OpType><HT><flag><HT><OpTime><HT><OpWho><HT>…`, so the
/// timestamp is column 2 and the user id column 3. Reading them a column earlier
/// parsed the flag (`0`) as the user id and rejected every punch on its
/// `user_id <= 0` guard.
pub(crate) fn parse_punch_row(log: &[&str]) -> Option<OperRow> {
    if log.len() < 4 {
        return None;
    }

    let op_code: i32 = log[0]
        .split_whitespace()
        .last()
        .unwrap_or("")
        .parse::<i32>()
        .unwrap_or(0);

    // 6/7/8 are the documented biometric operation codes. The protocol defines
    // others (the vendor's own server also accepts 9 and 13) whose meaning this
    // table does not enumerate, so an unrecognised code keeps its raw value and
    // reports the "other" type instead of being dropped.
    let enroll_type: i32 = match op_code {
        6 => 1, // Fingerprint
        7 => 2, // Password
        8 => 3, // Card
        _ => 5, // Other / unmapped
    };

    let user_id = log[3].trim().parse::<i32>().ok()?;
    if user_id <= 0 {
        return None;
    }

    Some(OperRow::Punch {
        user_id,
        enroll_type,
        op_code,
        raw_time: log[2].trim().to_string(),
    })
}

/// Parse one USER row: `USER<SP>PIN=<id>\tName=<n>\tPri=<n>\tPasswd=…`.
pub(crate) fn parse_user_row(log: &[&str]) -> Option<OperRow> {
    if log.len() < 3 {
        return None;
    }

    let user_id: i32 = labelled(log[0]).trim().parse::<i32>().ok()?;
    if user_id == 0 {
        return None;
    }

    Some(OperRow::User {
        user_id,
        name: labelled(log[1]).to_string(),
    })
}

/// Parse one FP row: `FP<SP>PIN=<id>\tFID=<n>\tSize=<n>\tValid=<n>\tTMP=<base64>`.
pub(crate) fn parse_finger_row(log: &[&str]) -> Option<OperRow> {
    if log.len() < 5 {
        return None;
    }

    // Split the label only, and require a non-empty payload: a short row must
    // be skipped, never sliced blindly.
    let template = labelled(log[4]);
    if template.is_empty() {
        return None;
    }

    let user_id: i32 = labelled(log[0]).trim().parse::<i32>().ok()?;
    if user_id == 0 {
        return None;
    }

    Some(OperRow::Finger {
        user_id,
        finger_id: labelled(log[1]).to_string(),
        template: template.to_string(),
    })
}

/// Resolve a device timestamp field to a UTC epoch.
///
/// Devices send either device-local wall clock (`%Y-%m-%d %H:%M:%S`) or a Unix
/// epoch. The wall-clock form needs the device offset applied once; an epoch is
/// already UTC and must not be shifted. An unparseable value returns `None` so
/// the row is dropped rather than stored with a guessed time - `unwrap_or_default`
/// once wrote 1970-01-01 punches into the report as if they were real.
pub(crate) fn resolve_op_time(raw: &str, device_timezone: i32) -> Option<i64> {
    match NaiveDateTime::parse_from_str(raw.trim(), "%Y-%m-%d %H:%M:%S") {
        Ok(naive) => Some(naive.and_utc().timestamp() - (3600 * device_timezone) as i64),
        Err(_) => raw.trim().parse::<i64>().ok(),
    }
}

async fn data_operlog(
    pool: web::Data<AppState>,
    get: web::Query<ReceiverCData>,
    data_body: String,
) -> Result<String> {
    let log_data = data_body.lines();
    let mut log_recorded: i32 = 0;

    // The device offset is fixed for the whole upload, so read it once instead
    // of per row.
    let device_timezone: i32 =
        sqlx::query_scalar::<_, i32>(r#"SELECT device_timezone FROM devices WHERE device_sn = $1"#)
            .bind(get.sn.as_deref().unwrap_or(""))
            .fetch_one(&pool.db)
            .await
            .unwrap_or(0);

    for logs in log_data {
        if logs.is_empty() {
            continue;
        }

        let log: Vec<&str> = logs.split("\t").collect();
        if log.len() < 2 {
            continue;
        }

        // A row's type token is either space-separated (`OPLOG 6 …`) or `=`-separated
        // (`USER=7`, `FP=7`). Splitting on whitespace alone turned every `=`-form
        // row into "Unknown data type", so device renames and fingerprint uploads
        // were dropped silently.
        let data_type = row_table(log[0]);

        match data_type {
            "OPLOG" => {
                let Some(OperRow::Punch {
                    user_id,
                    enroll_type,
                    op_code,
                    raw_time,
                }) = parse_punch_row(&log)
                else {
                    continue;
                };

                // `enroll_time` is a `bigint` epoch and `enroll_employee_id` an
                // `integer`. The old code bound a `NaiveDateTime` and a `String`,
                // which sqlx rejects at execution; the error was then dropped on
                // the floor, so every OPERLOG row was reported as dropped with no
                // trace.
                let Some(dt) = resolve_op_time(&raw_time, device_timezone) else {
                    continue;
                };

                if !matches!(op_code, 6..=8) {
                    log::warn!(
                        "operlog unmapped operation code {} device={}",
                        op_code,
                        get.sn.as_deref().unwrap_or("")
                    );
                }

                let row_result = sqlx::query(r#"INSERT INTO enrolls (enroll_device_sn, enroll_employee_id, enroll_type, enroll_time, enroll_status, enroll_op_code) VALUES ($1, $2, $3, $4, $5, $6)"#)
                    .bind(get.sn.as_deref().unwrap_or(""))
                    .bind(user_id)
                    .bind(enroll_type)
                    .bind(dt)
                    .bind(1)
                    .bind(op_code)
                    .execute(&pool.db)
                    .await;

                match row_result {
                    Ok(_) => log_recorded += 1,
                    Err(e) => {
                        // A resend after a dropped connection is not a loss, so
                        // the unique-constraint case still counts as recorded.
                        if e.to_string()
                            .contains("duplicate key value violates unique constraint")
                        {
                            log_recorded += 1;
                        } else {
                            log::error!("operlog enroll insert failed user={} err: {}", user_id, e);
                        }
                    }
                }
            }
            "USER" => {
                // Wire layout (ZKTeco PUSH SDK, OPERLOG/USER):
                //   USER<SP>PIN=<id>\tName=<n>\tPri=<n>\tPasswd=…\tCard=…\tGrp=…
                // Two spellings reach this handler, `USER=<id>` and
                // `USER<SP>PIN=<id>`; both leave the id and name after their
                // label.
                let Some(OperRow::User { user_id, name }) = parse_user_row(&log) else {
                    continue;
                };

                // A USER row carries no timestamp of its own - column 2 is `Pri=`,
                // not a date. Parsing it as one failed every time and dropped the
                // rename silently; stamp the row with arrival time.
                let dt: i64 = Utc::now().timestamp();

                let check: bool = sqlx::query_scalar(
                    r#"SELECT EXISTS(SELECT 1 FROM employee WHERE employee_id=$1)"#,
                )
                .bind(user_id)
                .fetch_one(&pool.db)
                .await
                .unwrap_or(false);

                if !check {
                    continue; // Skip if the user does not exist
                }

                // `update_at` is a `bigint` epoch and the name column is `employee_fname`.
                let query = sqlx::query(
                    r#"UPDATE employee SET employee_fname=$1, update_at=$2 WHERE employee_id=$3"#,
                )
                .bind(&name)
                .bind(dt)
                .bind(user_id)
                .execute(&pool.db)
                .await
                .map_err(|e: sqlx::Error| e.to_string());

                if query.is_ok() {
                    // Increment the log_recorded counter
                    log_recorded += 1;
                }

                if let Err(_e) = query {
                    //if e.contains("Duplicate entry") {
                    //    return HttpResponse::BadRequest();
                    //}
                    log::error!("Error inserting log: {}", _e);
                }
            }
            "FP" => {
                // Wire layout (ZKTeco PUSH SDK, OPERLOG/FP):
                //   FP<SP>PIN=<id>\tFID=<n>\tSize=<n>\tValid=<n>\tTMP=<base64>
                // The template sits in the last column behind a `TMP=` label.
                let Some(OperRow::Finger {
                    user_id,
                    finger_id,
                    template,
                }) = parse_finger_row(&log)
                else {
                    continue;
                };

                let check: bool = sqlx::query_scalar(r#"SELECT EXISTS(SELECT 1 FROM fingers WHERE finger_employee_id=$1 AND finger_id=$2)"#)
                    .bind(user_id)
                    .bind(&finger_id)
                    .fetch_one(&pool.db)
                    .await
                    .unwrap_or(false);

                if check {
                    sqlx::query(r#"UPDATE fingers SET finger_code=$1, update_at=$2 WHERE finger_employee_id=$3 AND finger_id=$4"#)
                        .bind(&template)
                        .bind(Utc::now().timestamp())
                        .bind(user_id)
                        .bind(&finger_id)

                        .execute(&pool.db)
                        .await
                        .map(|_| {
                            log_recorded += 1;
                        })?;
                } else {
                    sqlx::query(r#"INSERT INTO fingers (finger_employee_id, finger_code, finger_id) VALUES ($2, $3, $1)"#)
                        .bind(&finger_id)
                        .bind(user_id)
                        .bind(&template)
                        .execute(&pool.db)
                        .await
                        .map(|_| {
                            log_recorded += 1;
                        })?;

                    //if let Err(_e) = query {
                    //if e.contains("Duplicate entry") {
                    //    return HttpResponse::BadRequest();
                    //}
                    //    log::error!("Error inserting log: {}", _e);
                    //}
                }
            }
            _ => {
                log::info!("Unknown data type: {}", data_type);
                continue; // Skip unknown types
            }
        }
    }

    // Contact proof, same as ATTLOG: any upload from this device proves it is
    // alive, whether or not it also polls getrequest.
    let contact_sn = get.sn.as_deref().unwrap_or("");
    if log_recorded > 0 && touch_device(&pool, contact_sn).await {
        notify_device(&pool, "controller checked in");
    }

    // OPERLOG carries user/fingerprint updates, not attendance punches, but the
    // feed still benefits from knowing something landed.
    if log_recorded > 0
        && let Ok(newest) =
            sqlx::query_scalar::<_, i32>("SELECT COALESCE(MAX(enroll_id), 0) FROM enrolls")
                .fetch_one(&pool.db)
                .await
    {
        notify_enroll(&pool, newest, log_recorded);
    }

    // Return success message with the number of logs recorded
    Ok(format!("OK {}", log_recorded))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Split a raw wire row the way the upload loop does.
    fn cols(row: &str) -> Vec<&str> {
        row.split('\t').collect()
    }

    // The canonical seven-column OPLOG row from the ZKTeco PUSH SDK. Every
    // earlier fix on this endpoint parsed a hand-written three-column row
    // instead, so the real payload was rejected and the E2E still passed.
    const PUNCH: &str = "OPLOG 70\t0\t2024-08-05 16:08:50\t1157\t0\t0\t0";

    #[test]
    fn punch_row_reads_timestamp_and_user_from_the_right_columns() {
        let row = cols(PUNCH);
        assert_eq!(
            parse_punch_row(&row),
            Some(OperRow::Punch {
                user_id: 1157,
                enroll_type: 5, // OPLOG 70 is not a documented biometric code
                op_code: 70,
                raw_time: "2024-08-05 16:08:50".to_string(),
            })
        );
    }

    /// The original off-by-one read the flag (`0`) as the user id and then
    /// dropped the punch on `user_id <= 0`.
    #[test]
    fn punch_row_never_takes_the_flag_column_as_the_user() {
        let row = cols(PUNCH);
        match parse_punch_row(&row) {
            Some(OperRow::Punch { user_id, .. }) => assert_ne!(user_id, 0),
            other => panic!("expected a punch, got {other:?}"),
        }
    }

    #[test]
    fn punch_row_maps_the_documented_operation_codes() {
        for (code, expected) in [(6, 1), (7, 2), (8, 3)] {
            let raw = format!("OPLOG {code}\t0\t2024-08-05 16:08:50\t1157\t0\t0\t0");
            let row = cols(&raw);
            assert_eq!(
                parse_punch_row(&row),
                Some(OperRow::Punch {
                    user_id: 1157,
                    enroll_type: expected,
                    op_code: code,
                    raw_time: "2024-08-05 16:08:50".to_string(),
                })
            );
        }
    }

    /// OPLOG 9 and 13 are accepted by the vendor's own server and have no
    /// documented meaning here. The row must survive with its raw code.
    #[test]
    fn unknown_operation_code_is_kept_not_dropped() {
        for code in [9, 13, 77] {
            let raw = format!("OPLOG {code}\t0\t2024-08-05 16:08:50\t1157\t0\t0\t0");
            let row = cols(&raw);
            assert_eq!(
                parse_punch_row(&row),
                Some(OperRow::Punch {
                    user_id: 1157,
                    enroll_type: 5,
                    op_code: code,
                    raw_time: "2024-08-05 16:08:50".to_string(),
                })
            );
        }
    }

    #[test]
    fn short_or_invalid_punch_rows_are_rejected_without_panicking() {
        for row in [
            vec!["OPLOG 70"],
            vec!["OPLOG 70", "0", "2024-08-05 16:08:50"],
            // The user column is non-numeric.
            cols("OPLOG 70\t0\t2024-08-05 16:08:50\tN/A\t0\t0\t0"),
            // A zero user id is not a punch.
            cols("OPLOG 70\t0\t2024-08-05 16:08:50\t0\t0\t0\t0"),
        ] {
            assert_eq!(parse_punch_row(&row), None);
        }
    }

    #[test]
    fn user_row_reads_the_pin_and_name_labels() {
        // `USER<SP>PIN=…` is what a device sends; `USER=7` is the other form.
        assert_eq!(
            parse_user_row(&cols("USER PIN=1157\tName=Bagus\tPri=0\tPasswd=1234")),
            Some(OperRow::User {
                user_id: 1157,
                name: "Bagus".to_string(),
            })
        );
        assert_eq!(
            parse_user_row(&cols("USER=1157\tName=Bagus\tPri=0")),
            Some(OperRow::User {
                user_id: 1157,
                name: "Bagus".to_string(),
            })
        );
    }

    /// A name may itself contain `=`; taking the last segment truncated it.
    #[test]
    fn user_row_keeps_an_equals_sign_inside_the_name() {
        assert_eq!(
            parse_user_row(&cols("USER PIN=1157\tName=Bagus=Setiawan\tPri=0")),
            Some(OperRow::User {
                user_id: 1157,
                name: "Bagus=Setiawan".to_string(),
            })
        );
    }

    #[test]
    fn user_row_rejects_zero_id_and_short_rows() {
        assert_eq!(parse_user_row(&cols("USER PIN=0\tName=X\tPri=0")), None);
        assert_eq!(parse_user_row(&cols("USER PIN=1157\tName=X")), None);
        assert_eq!(parse_user_row(&cols("USER PIN=abc\tName=X\tPri=0")), None);
    }

    /// Base64 padding means the payload often ends in `=`; `split("=").last()`
    /// truncated exactly those templates, silently storing a broken template.
    #[test]
    fn finger_row_keeps_base64_padding_intact() {
        let tpl = "c3RhdGVsAAAAF1JJZDUwOQ==";
        assert_eq!(
            parse_finger_row(&cols(&format!(
                "FP PIN=1160\tFID=0\tSize=1608\tValid=1\tTMP={tpl}"
            ))),
            Some(OperRow::Finger {
                user_id: 1160,
                finger_id: "0".to_string(),
                template: tpl.to_string(),
            })
        );
    }

    #[test]
    fn finger_row_rejects_short_rows_and_empty_templates() {
        assert_eq!(
            parse_finger_row(&cols("FP PIN=1160\tFID=0\tSize=1608\tValid=1")),
            None
        );
        assert_eq!(
            parse_finger_row(&cols("FP PIN=1160\tFID=0\tSize=1608\tValid=1\tTMP=")),
            None
        );
        assert_eq!(
            parse_finger_row(&cols("FP PIN=0\tFID=0\tSize=1\tValid=1\tTMP=x")),
            None
        );
    }

    /// Both token spellings occur on the wire. Whitespace-only splitting
    /// classified every `=`-form row as an unknown type and dropped it.
    #[test]
    fn row_table_accepts_both_token_spellings() {
        assert_eq!(row_table("OPLOG 70"), "OPLOG");
        assert_eq!(row_table("USER PIN=1157"), "USER");
        assert_eq!(row_table("USER=1157"), "USER");
        assert_eq!(row_table("FP PIN=1160"), "FP");
        assert_eq!(row_table("FP=7"), "FP");
        assert_eq!(row_table("GARBAGE"), "GARBAGE");
        assert_eq!(row_table(""), "");
    }

    #[test]
    fn wall_clock_is_shifted_by_the_device_offset() {
        let naive = "2024-08-05 16:08:50";
        let utc = resolve_op_time(naive, 0).unwrap();
        // GMT+7 local time is 7 hours ahead of UTC.
        assert_eq!(resolve_op_time(naive, 7).unwrap(), utc - 7 * 3600);
        assert_eq!(resolve_op_time(naive, -7).unwrap(), utc + 7 * 3600);
    }

    /// An epoch is already UTC, so it must NOT be shifted by the device offset -
    /// the two branches used to be easy to confuse and were the cause of
    /// seven-hour-off punches.
    #[test]
    fn epoch_is_not_shifted_by_the_device_offset() {
        let epoch = 1_722_875_330;
        assert_eq!(resolve_op_time(&epoch.to_string(), 0), Some(epoch));
        assert_eq!(resolve_op_time(&epoch.to_string(), 7), Some(epoch));
        assert_eq!(resolve_op_time(&epoch.to_string(), -7), Some(epoch));
    }

    /// The regression that mattered: `unwrap_or_default()` once turned an
    /// unparseable timestamp into epoch 0, storing a 1970 punch as if it were
    /// real. An unusable value must drop the row instead.
    #[test]
    fn unparseable_timestamp_yields_none_not_the_epoch() {
        for bad in ["notatimestamp", "", "2024-13-45 99:99:99", "2024/08/05"] {
            assert_eq!(resolve_op_time(bad, 7), None, "accepted {bad:?}");
        }
    }

    #[test]
    fn surrounding_whitespace_does_not_break_a_timestamp() {
        let utc = resolve_op_time("2024-08-05 16:08:50", 0).unwrap();
        assert_eq!(resolve_op_time(" 2024-08-05 16:08:50 ", 0), Some(utc));
    }
}
