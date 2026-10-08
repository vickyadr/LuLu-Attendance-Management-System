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
    log::info!("CDATA: {:?}", data.sn.as_deref().unwrap_or("No SN"));

    let tz =
        sqlx::query_scalar::<_, i32>(r#"SELECT device_timezone FROM devices WHERE device_sn = $1"#)
            .bind(data.sn.as_deref().unwrap_or(""))
            .fetch_one(&pool.db.clone())
            .await
            .unwrap_or(0);

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


    let (text, is_operlog) = if get.table.as_ref().unwrap() == "OPERLOG" {
        (data_operlog(pool, get, data_body).await, true)
    } else if get.table.as_ref().unwrap() == "ATTLOG" {
        (data_attlog(pool, get, data_body).await, false)
    } else {

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

fn notify_device(pool: &AppState, message: &str) {
    let receivers = pool.ws.broadcast(WsOut {
        event: "device",
        message: message.to_owned(),
        enroll_id: 0,
    });
    log::info!("ws broadcast device -> {receivers} client(s)");
}

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

    let came_online = flipped.is_some();

    let _ = sqlx::query(
        r#"UPDATE devices SET device_online = EXTRACT(EPOCH FROM NOW())::BIGINT WHERE device_sn = $1 AND device_state <> 3"#,
    )
    .bind(sn)
    .execute(&pool.db)
    .await;

    came_online
}

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


        let log: Vec<&str> = logs.split("\t").collect();
        if log.len() < 4 {
            continue;
        }


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

        let user_id: i32 = match log[0].parse() {
            Ok(id) => id,
            Err(_) => continue,
        };
        if user_id <= 0 {
            continue;
        }

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
                "None of the {} uploaded row(s) could be stored — check the server log",
                log_failed
            );
        }
        bail!("Some data error: {}", log_recorded);
    }

    if touch_device(&pool, get.sn.as_deref().unwrap_or("")).await {
        notify_device(&pool, "controller checked in");
    }

    if let Ok(newest) =
        sqlx::query_scalar::<_, i32>("SELECT COALESCE(MAX(enroll_id), 0) FROM enrolls")
            .fetch_one(&pool.db)
            .await
    {
        notify_enroll(&pool, newest, log_recorded);
    }

    Ok(format!("OK {}", log_recorded))
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum OperRow {
    /// An attendance punch from the OPLOG table.
    Punch {
        user_id: i32,
        enroll_type: i32,
        op_code: i32,
        raw_time: String,
    },

    User {
        user_id: i32,
        name: String,
    },

    Finger {
        user_id: i32,
        finger_id: String,
        template: String,
    },
}

fn row_table(row: &str) -> &str {
    row.split(|c: char| c == '=' || c.is_whitespace())
        .next()
        .unwrap_or("")
}

fn labelled(field: &str) -> &str {
    field.split_once('=').map_or(field, |(_, v)| v)
}

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

    let contact_sn = get.sn.as_deref().unwrap_or("");
    if log_recorded > 0 && touch_device(&pool, contact_sn).await {
        notify_device(&pool, "controller checked in");
    }

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

    fn cols(row: &str) -> Vec<&str> {
        row.split('\t').collect()
    }

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

    #[test]
    fn epoch_is_not_shifted_by_the_device_offset() {
        let epoch = 1_722_875_330;
        assert_eq!(resolve_op_time(&epoch.to_string(), 0), Some(epoch));
        assert_eq!(resolve_op_time(&epoch.to_string(), 7), Some(epoch));
        assert_eq!(resolve_op_time(&epoch.to_string(), -7), Some(epoch));
    }

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
