// Field attendance: the employee declares an event with a photo, the admin
// approves the day. One day row per employee, one photo row per event, so a
// day can be office-punch + field-photo on either side, or field on both.
//
// No assignment table on purpose: employees decide when they are in the field.
// No automatic daily_absences write either: a missing side stays visible on the
// day and the existing manual lupa_absen flow stays the only writer.

use crate::utility::{
    authorization::{Action, check_action_access, get_session_user},
    stor::{AppState, GenericResponse, IntoResponse, NoDataResponse},
};
use actix_web::{
    HttpRequest, HttpResponse, Responder, get, post, put,
    web::{self, ReqData},
};
use chrono::{DateTime, NaiveDate, Utc};
use ring::digest::{SHA256, digest};
use serde_json::{Value, json};
use sqlx::{PgPool, Row};
use std::path::PathBuf;
use uuid::Uuid;

const MAX_PHOTO_BYTES: usize = 2 * 1024 * 1024;
const MAX_EVENTS_PER_DAY: i64 = 10;
const DEFAULT_RETENTION_DAYS: i64 = 60;
const DELETE_WINDOW_SECS: i64 = 10 * 60;

fn upload_root() -> PathBuf {
    PathBuf::from(std::env::var("FIELD_UPLOAD_DIR").unwrap_or_else(|_| "./uploads".to_string()))
}

/// Office date, not the device date: a trip across timezones must not split
/// one working day into two rows.
async fn office_today(pool: &PgPool) -> Result<NaiveDate, sqlx::Error> {
    sqlx::query_scalar::<_, NaiveDate>("SELECT (NOW() AT TIME ZONE 'Asia/Jakarta')::date")
        .fetch_one(pool)
        .await
}

async fn employee_for_user(pool: &PgPool, nickname: &str) -> Option<i32> {
    sqlx::query_scalar::<_, i32>(
        "SELECT employee_id FROM employee WHERE employee_user_nickname = $1",
    )
    .bind(nickname)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
}

/// Resolve the caller to an employee. `Ok(None)` means the page gate passed but
/// the account is not linked to an employee row.
async fn caller_employee(
    pool: &PgPool,
    bearer: Option<ReqData<String>>,
) -> Result<Option<(String, i32)>, HttpResponse> {
    let nickname = match get_session_user(pool.clone(), bearer).await {
        Some(nickname) => nickname,
        None => {
            return Err(NoDataResponse::new("Session token invalid".to_string(), 401).into_response());
        }
    };
    Ok(employee_for_user(pool, &nickname)
        .await
        .map(|employee_id| (nickname, employee_id)))
}

/// True when the office-time clock has passed the shift end. A night shift ends
/// after midnight, so "past the end" means between end and start, not simply
/// `now >= end`.
fn after_shift_end(now_secs: i64, start_secs: i64, end_secs: i64) -> bool {
    // Only day shifts: a night shift that ends after midnight cannot be told
    // apart from "before the shift starts" by clock alone, and guessing "out"
    // at 20:00 for a 22:00 shift would be wrong. Those days rely on a device
    // punch-in instead.
    end_secs > start_secs && now_secs >= end_secs
}

fn photo_kind(bytes: &[u8]) -> Option<(&'static str, &'static str)> {
    if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        Some(("image/jpeg", "jpg"))
    } else if bytes.starts_with(&[0x89, b'P', b'N', b'G']) {
        Some(("image/png", "png"))
    } else {
        None
    }
}

fn day_payload(row: &sqlx::postgres::PgRow) -> Value {
    json!({
        "id": row.get::<Uuid, _>("id"),
        "employee_id": row.get::<i32, _>("employee_id"),
        "work_date": row.get::<NaiveDate, _>("work_date").to_string(),
        "status": row.get::<String, _>("status"),
        "note": row.try_get::<Option<String>, _>("note").ok().flatten(),
        "review_reason": row.try_get::<Option<String>, _>("review_reason").ok().flatten(),
        "reviewed_by": row.try_get::<Option<String>, _>("reviewed_by").ok().flatten(),
    })
}

async fn photos_for(pool: &PgPool, attendance_id: Uuid) -> Vec<Value> {
    let rows = sqlx::query(
        r#"SELECT id, kind, lat, lng, accuracy_m, tz_offset_minutes, device_at, server_at, sha256, bytes
           FROM field_photo WHERE attendance_id = $1 ORDER BY server_at"#,
    )
    .bind(attendance_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();
    rows.into_iter()
        .map(|row| {
            json!({
                "id": row.get::<Uuid, _>("id"),
                "kind": row.get::<String, _>("kind"),
                "lat": row.try_get::<Option<f64>, _>("lat").ok().flatten(),
                "lng": row.try_get::<Option<f64>, _>("lng").ok().flatten(),
                "accuracy_m": row.try_get::<Option<f32>, _>("accuracy_m").ok().flatten(),
                "tz_offset_minutes": row.try_get::<Option<i32>, _>("tz_offset_minutes").ok().flatten(),
                "device_at": row.try_get::<Option<DateTime<Utc>>, _>("device_at").ok().flatten().map(|d| d.to_rfc3339()),
                "server_at": row.get::<DateTime<Utc>, _>("server_at").to_rfc3339(),
                "sha256": row.get::<String, _>("sha256"),
                "bytes": row.get::<i32, _>("bytes"),
            })
        })
        .collect()
}

async fn day_row(pool: &PgPool, id: Uuid) -> Option<sqlx::postgres::PgRow> {
    sqlx::query("SELECT * FROM field_attendance WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
}

async fn ensure_today(pool: &PgPool, employee_id: i32) -> Result<Uuid, sqlx::Error> {
    let day = office_today(pool).await?;
    sqlx::query_scalar::<_, Uuid>(
        r#"INSERT INTO field_attendance (employee_id, work_date)
           VALUES ($1, $2)
           ON CONFLICT (employee_id, work_date) DO UPDATE SET updated_at = NOW()
           RETURNING id"#,
    )
    .bind(employee_id)
    .bind(day)
    .fetch_one(pool)
    .await
}

fn retention_days_setting() -> i64 {
    DEFAULT_RETENTION_DAYS
}

/// Opportunistic cleanup: at most a handful of expired photos per upload, so
/// there is no scheduler to run and no unbounded delete inside a request.
async fn purge_expired(pool: &PgPool) {
    let days = sqlx::query_scalar::<_, i64>(
        "SELECT setting_value FROM settings WHERE setting_key = 'field_photo_retention_days'",
    )
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
    .filter(|days| *days > 0)
    .unwrap_or(retention_days_setting());

    let rows = sqlx::query_as::<_, (String,)>(
        r#"DELETE FROM field_photo WHERE id IN (
               SELECT id FROM field_photo
               WHERE server_at < NOW() - ($1::int * INTERVAL '1 day')
               LIMIT 50
           ) RETURNING file_path"#,
    )
    .bind(days as i32)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    for (path,) in rows {
        let _ = std::fs::remove_file(path);
    }
}

// GET /api/field/me
#[get("/me")]
pub async fn field_me(
    pool: web::Data<AppState>,
    bearer: Option<ReqData<String>>,
) -> impl Responder {
    if let Ok(resp) = check_action_access(pool.db.clone(), bearer.clone(), "field", Action::Add).await {
        return resp;
    }

    let (_, employee_id) = match caller_employee(&pool.db, bearer).await {
        Ok(Some(found)) => found,
        Ok(None) => {
            return NoDataResponse::new(
                "Akun ini belum terhubung ke data karyawan".to_string(),
                403,
            )
            .into_response();
        }
        Err(resp) => return resp,
    };

    let profile = sqlx::query(
        "SELECT employee_id, employee_fname, employee_lname, employee_department FROM employee WHERE employee_id = $1",
    )
    .bind(employee_id)
    .fetch_optional(&pool.db)
    .await;

    let mut payload = json!({ "employee_id": employee_id });
    if let Ok(Some(row)) = profile {
        payload["name"] = json!(format!(
            "{} {}",
            row.get::<String, _>("employee_fname"),
            row.try_get::<Option<String>, _>("employee_lname")
                .ok()
                .flatten()
                .unwrap_or_default()
        )
        .trim()
        .to_string());
        payload["department"] = json!(row
            .try_get::<Option<String>, _>("employee_department")
            .ok()
            .flatten());
    }

    let day = match office_today(&pool.db).await {
        Ok(day) => day,
        Err(error) => {
            log::error!("field: office date failed: {error}");
            return NoDataResponse::new("Database error".to_string(), 500).into_response();
        }
    };
    let today = sqlx::query(
        "SELECT * FROM field_attendance WHERE employee_id = $1 AND work_date = $2",
    )
    .bind(employee_id)
    .bind(day)
    .fetch_optional(&pool.db)
    .await
    .ok()
    .flatten();

    let mut photos: Vec<Value> = Vec::new();
    payload["today"] = match today {
        Some(row) => {
            let id = row.get::<Uuid, _>("id");
            let mut day_json = day_payload(&row);
            photos = photos_for(&pool.db, id).await;
            day_json["photos"] = json!(photos);
            day_json
        }
        None => Value::Null,
    };

    // The next capture is decided from the day itself: alternate after an
    // existing photo, otherwise once the shift has ended the only missing side
    // is the checkout, so "pulang" must be the default instead of "datang".
    let shift = sqlx::query(
        r#"SELECT sh.shift_start_time AS start_secs, sh.shift_end_time AS end_secs
           FROM employee e
           LEFT JOIN schedules s ON s.schedule_id = e.employee_schedule_id
           LEFT JOIN shifts sh ON sh.shift_id = s.schedule_shift_id
           WHERE e.employee_id = $1"#,
    )
    .bind(employee_id)
    .fetch_optional(&pool.db)
    .await
    .ok()
    .flatten();
    let start_secs = shift
        .as_ref()
        .and_then(|row| row.try_get::<Option<i64>, _>("start_secs").ok().flatten())
        .unwrap_or(8 * 3600);
    let end_secs = shift
        .as_ref()
        .and_then(|row| row.try_get::<Option<i64>, _>("end_secs").ok().flatten())
        .unwrap_or(17 * 3600);
    let now_secs: i64 = sqlx::query_scalar(
        "SELECT (EXTRACT(EPOCH FROM (NOW() AT TIME ZONE 'Asia/Jakarta'))::bigint % 86400)",
    )
    .fetch_one(&pool.db)
    .await
    .unwrap_or(0);

    // Evidence beats the clock: if the device already recorded the arrival and
    // no checkout, the employee is here for the checkout photo even mid-shift.
    let punch = sqlx::query(
        "SELECT punch_in, punch_out FROM attendance_daily WHERE employee_id = $1 AND work_date = $2",
    )
    .bind(employee_id)
    .bind(day)
    .fetch_optional(&pool.db)
    .await
    .ok()
    .flatten();
    let has_punch_in = punch
        .as_ref()
        .and_then(|row| row.try_get::<Option<DateTime<Utc>>, _>("punch_in").ok().flatten())
        .is_some();
    let has_punch_out = punch
        .as_ref()
        .and_then(|row| row.try_get::<Option<DateTime<Utc>>, _>("punch_out").ok().flatten())
        .is_some();

    let suggested_kind = match photos.last().and_then(|p| p["kind"].as_str()) {
        Some("in") => "out",
        Some(_) => "in",
        None if (has_punch_in && !has_punch_out) || after_shift_end(now_secs, start_secs, end_secs) => "out",
        None => "in",
    };
    payload["suggested_kind"] = json!(suggested_kind);
    payload["shift_end_secs"] = json!(end_secs);
    payload["max_events"] = json!(MAX_EVENTS_PER_DAY);

    GenericResponse::single(payload, "OK".to_string()).into_response()
}

// POST /api/field/attendances
#[post("/attendances")]
pub async fn field_open_day(
    pool: web::Data<AppState>,
    bearer: Option<ReqData<String>>,
    body: web::Json<Value>,
) -> impl Responder {
    if let Ok(resp) = check_action_access(pool.db.clone(), bearer.clone(), "field", Action::Add).await {
        return resp;
    }

    let (_, employee_id) = match caller_employee(&pool.db, bearer).await {
        Ok(Some(found)) => found,
        Ok(None) => {
            return NoDataResponse::new(
                "Akun ini belum terhubung ke data karyawan".to_string(),
                403,
            )
            .into_response();
        }
        Err(resp) => return resp,
    };

    let id = match ensure_today(&pool.db, employee_id).await {
        Ok(id) => id,
        Err(error) => {
            log::error!("field: open day failed: {error}");
            return NoDataResponse::new("Database error".to_string(), 500).into_response();
        }
    };

    if let Some(note) = body.get("note").and_then(Value::as_str) {
        let note = note.trim();
        // A note edit must not change an approved day in place: the reviewer
        // approved what was there. Moving evidence (a new photo) already
        // re-opens the day.
        let updated = sqlx::query(
            "UPDATE field_attendance SET note = $1, updated_at = NOW() WHERE id = $2 AND status <> 'approved'",
        )
        .bind(note)
        .bind(id)
        .execute(&pool.db)
        .await;
        match updated {
            Ok(result) if result.rows_affected() == 0 => {
                let approved = sqlx::query_scalar::<_, bool>(
                    "SELECT status = 'approved' FROM field_attendance WHERE id = $1",
                )
                .bind(id)
                .fetch_optional(&pool.db)
                .await
                .ok()
                .flatten()
                .unwrap_or(false);
                if approved {
                    return NoDataResponse::new(
                        "Hari yang sudah disetujui tidak bisa diubah catatannya".to_string(),
                        409,
                    )
                    .into_response();
                }
            }
            Ok(_) => {}
            Err(error) => {
                log::error!("field: note update failed: {error}");
                return NoDataResponse::new("Database error".to_string(), 500).into_response();
            }
        }
    }

    match day_row(&pool.db, id).await {
        Some(row) => {
            let mut payload = day_payload(&row);
            payload["photos"] = json!(photos_for(&pool.db, id).await);
            GenericResponse::single(payload, "Day opened".to_string()).into_response()
        }
        None => NoDataResponse::new("Day not found".to_string(), 404).into_response(),
    }
}

// POST /api/field/attendances/{id}/photo?kind=in|out&lat=..&lng=..&accuracy=..&tz=..&device_at=epoch
#[post("/attendances/{id}/photo")]
pub async fn field_upload_photo(
    pool: web::Data<AppState>,
    bearer: Option<ReqData<String>>,
    id: web::Path<Uuid>,
    query: web::Query<FieldPhotoQuery>,
    body: web::Bytes,
) -> impl Responder {
    if let Ok(resp) = check_action_access(pool.db.clone(), bearer.clone(), "field", Action::Add).await {
        return resp;
    }

    let (_, employee_id) = match caller_employee(&pool.db, bearer).await {
        Ok(Some(found)) => found,
        Ok(None) => {
            return NoDataResponse::new(
                "Akun ini belum terhubung ke data karyawan".to_string(),
                403,
            )
            .into_response();
        }
        Err(resp) => return resp,
    };

    let attendance_id = id.into_inner();
    let day = match day_row(&pool.db, attendance_id).await {
        Some(row) => row,
        None => return NoDataResponse::new("Day not found".to_string(), 404).into_response(),
    };
    if day.get::<i32, _>("employee_id") != employee_id {
        return NoDataResponse::new("Day not found".to_string(), 404).into_response();
    }

    if query.kind != "in" && query.kind != "out" {
        return NoDataResponse::new("kind must be in or out".to_string(), 400).into_response();
    }
    if body.is_empty() || body.len() > MAX_PHOTO_BYTES {
        return NoDataResponse::new(
            format!("Photo must be 1 byte to {} MiB", MAX_PHOTO_BYTES / 1024 / 1024),
            400,
        )
        .into_response();
    }
    let Some((mime, extension)) = photo_kind(&body) else {
        return NoDataResponse::new("Photo must be JPEG or PNG".to_string(), 400).into_response();
    };

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM field_photo WHERE attendance_id = $1")
        .bind(attendance_id)
        .fetch_one(&pool.db)
        .await
        .unwrap_or(0);
    if count >= MAX_EVENTS_PER_DAY {
        return NoDataResponse::new(
            format!("Maksimal {MAX_EVENTS_PER_DAY} foto per hari"),
            400,
        )
        .into_response();
    }

    let now = Utc::now();
    let dir = upload_root()
        .join("field")
        .join(now.format("%Y").to_string())
        .join(now.format("%m").to_string());
    if let Err(error) = std::fs::create_dir_all(&dir) {
        log::error!("field: cannot create upload dir {:?}: {error}", dir);
        return NoDataResponse::new("Storage error".to_string(), 500).into_response();
    }
    let photo_id = Uuid::new_v4();
    let path = dir.join(format!("{photo_id}.{extension}"));
    if let Err(error) = std::fs::write(&path, &body) {
        log::error!("field: cannot write {:?}: {error}", path);
        return NoDataResponse::new("Storage error".to_string(), 500).into_response();
    }
    let checksum = hex::encode(digest(&SHA256, &body).as_ref());

    let inserted = sqlx::query(
        r#"INSERT INTO field_photo
              (id, attendance_id, kind, file_path, sha256, bytes, lat, lng, accuracy_m, tz_offset_minutes, device_at, server_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)"#,
    )
    .bind(photo_id)
    .bind(attendance_id)
    .bind(&query.kind)
    .bind(path.to_string_lossy().to_string())
    .bind(&checksum)
    .bind(body.len() as i32)
    .bind(query.lat)
    .bind(query.lng)
    .bind(query.accuracy)
    .bind(query.tz)
    .bind(query.device_at.map(|epoch| DateTime::<Utc>::from_timestamp(epoch, 0)).flatten())
    .bind(now)
    .execute(&pool.db)
    .await;

    if let Err(error) = inserted {
        log::error!("field: insert photo failed: {error}");
        let _ = std::fs::remove_file(&path);
        return NoDataResponse::new("Database error".to_string(), 500).into_response();
    }

    // A new photo on an already reviewed day re-opens it: the admin sees the
    // whole day again instead of an approval that silently ignores evidence.
    let _ = sqlx::query(
        r#"UPDATE field_attendance
           SET status = 'pending', reviewed_by = NULL, reviewed_at = NULL, review_reason = NULL, updated_at = NOW()
           WHERE id = $1 AND status <> 'pending'"#,
    )
    .bind(attendance_id)
    .execute(&pool.db)
    .await;
    let _ = sqlx::query("UPDATE field_attendance SET updated_at = NOW() WHERE id = $1")
        .bind(attendance_id)
        .execute(&pool.db)
        .await;

    purge_expired(&pool.db).await;

    GenericResponse::single(
        json!({
            "id": photo_id,
            "kind": query.kind,
            "mime": mime,
            "bytes": body.len(),
            "sha256": checksum,
        }),
        "Photo uploaded".to_string(),
    )
    .into_response()
}

#[derive(Debug, serde::Deserialize)]
pub struct FieldPhotoQuery {
    pub kind: String,
    pub lat: Option<f64>,
    pub lng: Option<f64>,
    pub accuracy: Option<f32>,
    pub tz: Option<i32>,
    pub device_at: Option<i64>,
}

// GET /api/field/attendances?status=&from=&to=
#[get("/attendances")]
pub async fn field_list_days(
    pool: web::Data<AppState>,
    bearer: Option<ReqData<String>>,
    query: web::Query<FieldListQuery>,
) -> impl Responder {
    if let Ok(resp) = check_action_access(pool.db.clone(), bearer, "field", Action::View).await {
        return resp;
    }

    let to = query.to.unwrap_or_else(|| chrono::Utc::now().date_naive() + chrono::Days::new(1));
    let from = query.from.unwrap_or_else(|| to - chrono::Days::new(30));
    let status = query.status.as_deref().filter(|value| !value.is_empty());

    let rows = sqlx::query(
        r#"SELECT a.id, a.employee_id,
                  trim(e.employee_fname || ' ' || COALESCE(e.employee_lname, '')) AS employee_name,
                  a.work_date, a.status, a.note, a.review_reason, a.reviewed_by, a.reviewed_at,
                  COALESCE((
                      SELECT json_agg(json_build_object(
                                 'id', p.id, 'kind', p.kind, 'lat', p.lat, 'lng', p.lng,
                                 'accuracy_m', p.accuracy_m, 'tz_offset_minutes', p.tz_offset_minutes,
                                 'device_at', p.device_at, 'server_at', p.server_at,
                                 'sha256', p.sha256, 'bytes', p.bytes)
                             ORDER BY p.server_at)
                      FROM field_photo p WHERE p.attendance_id = a.id
                  ), '[]'::json) AS photos,
                  ad.punch_in, ad.punch_out
           FROM field_attendance a
           JOIN employee e ON e.employee_id = a.employee_id
           LEFT JOIN attendance_daily ad
                  ON ad.employee_id = a.employee_id AND ad.work_date = a.work_date
           WHERE ($1::text IS NULL OR a.status = $1)
             AND a.work_date BETWEEN $2 AND $3
           ORDER BY a.work_date DESC, a.updated_at DESC
           LIMIT 200"#,
    )
    .bind(status)
    .bind(from)
    .bind(to)
    .fetch_all(&pool.db)
    .await;

    match rows {
        Ok(rows) => {
            let data: Vec<Value> = rows
                .into_iter()
                .map(|row| {
                    json!({
                        "id": row.get::<Uuid, _>("id"),
                        "employee_id": row.get::<i32, _>("employee_id"),
                        "employee_name": row.get::<String, _>("employee_name"),
                        "work_date": row.get::<NaiveDate, _>("work_date").to_string(),
                        "status": row.get::<String, _>("status"),
                        "note": row.try_get::<Option<String>, _>("note").ok().flatten(),
                        "review_reason": row.try_get::<Option<String>, _>("review_reason").ok().flatten(),
                        "reviewed_by": row.try_get::<Option<String>, _>("reviewed_by").ok().flatten(),
                        "reviewed_at": row.try_get::<Option<DateTime<Utc>>, _>("reviewed_at").ok().flatten().map(|d| d.to_rfc3339()),
                        "photos": row.get::<Value, _>("photos"),
                        "punch_in": row.try_get::<Option<DateTime<Utc>>, _>("punch_in").ok().flatten().map(|d| d.to_rfc3339()),
                        "punch_out": row.try_get::<Option<DateTime<Utc>>, _>("punch_out").ok().flatten().map(|d| d.to_rfc3339()),
                    })
                })
                .collect();
            GenericResponse::ok(data, "OK".to_string()).into_response()
        }
        Err(error) => {
            log::error!("field: list failed: {error}");
            NoDataResponse::new("Database error".to_string(), 500).into_response()
        }
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct FieldListQuery {
    pub status: Option<String>,
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
}

// PUT /api/field/attendances/{id}/review
#[put("/attendances/{id}/review")]
pub async fn field_review_day(
    pool: web::Data<AppState>,
    bearer: Option<ReqData<String>>,
    id: web::Path<Uuid>,
    body: web::Json<Value>,
) -> impl Responder {
    let reviewer = match get_session_user(pool.db.clone(), bearer.clone()).await {
        Some(nickname) => nickname,
        None => return NoDataResponse::new("Session token invalid".to_string(), 401).into_response(),
    };
    if let Ok(resp) = check_action_access(pool.db.clone(), bearer, "field", Action::Edit).await {
        return resp;
    }

    let action = body.get("action").and_then(Value::as_str).unwrap_or("");
    let status = match action {
        "approve" => "approved",
        "reject" => "rejected",
        _ => return NoDataResponse::new("action must be approve or reject".to_string(), 400).into_response(),
    };
    let reason = body
        .get("reason")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty());
    if status == "rejected" && reason.is_none() {
        return NoDataResponse::new("Alasan wajib diisi saat menolak".to_string(), 400).into_response();
    }

    let updated = sqlx::query(
        r#"UPDATE field_attendance
           SET status = $1, reviewed_by = $2, reviewed_at = NOW(), review_reason = $3, updated_at = NOW()
           WHERE id = $4
           RETURNING id, employee_id, work_date, status"#,
    )
    .bind(status)
    .bind(&reviewer)
    .bind(reason)
    .bind(id.into_inner())
    .fetch_optional(&pool.db)
    .await;

    match updated {
        Ok(Some(row)) => GenericResponse::single(
            json!({
                "id": row.get::<Uuid, _>("id"),
                "employee_id": row.get::<i32, _>("employee_id"),
                "work_date": row.get::<NaiveDate, _>("work_date").to_string(),
                "status": row.get::<String, _>("status"),
            }),
            format!("Day {status}"),
        )
        .into_response(),
        Ok(None) => NoDataResponse::new("Day not found".to_string(), 404).into_response(),
        Err(error) => {
            log::error!("field: review failed: {error}");
            NoDataResponse::new("Database error".to_string(), 500).into_response()
        }
    }
}

// GET /api/field/photos/{id} - admin with field View, or the employee who owns it.
#[get("/photos/{id}")]
pub async fn field_photo_file(
    pool: web::Data<AppState>,
    bearer: Option<ReqData<String>>,
    id: web::Path<Uuid>,
    _req: HttpRequest,
) -> impl Responder {
    let nickname = match get_session_user(pool.db.clone(), bearer.clone()).await {
        Some(nickname) => nickname,
        None => return NoDataResponse::new("Session token invalid".to_string(), 401).into_response(),
    };

    let row = sqlx::query(
        r#"SELECT p.file_path, a.employee_id
           FROM field_photo p JOIN field_attendance a ON a.id = p.attendance_id
           WHERE p.id = $1"#,
    )
    .bind(id.into_inner())
    .fetch_optional(&pool.db)
    .await
    .ok()
    .flatten();

    let Some(row) = row else {
        return NoDataResponse::new("Photo not found".to_string(), 404).into_response();
    };

    let is_admin = check_action_access(pool.db.clone(), bearer.clone(), "field", Action::View)
        .await
        .is_err();
    let is_owner = employee_for_user(&pool.db, &nickname)
        .await
        .map(|employee_id| employee_id == row.get::<i32, _>("employee_id"))
        .unwrap_or(false);
    if !is_admin && !is_owner {
        return NoDataResponse::new("Photo not found".to_string(), 404).into_response();
    }

    let path = row.get::<String, _>("file_path");
    match std::fs::read(&path) {
        Ok(bytes) => {
            let mime = photo_kind(&bytes).map(|(mime, _)| mime).unwrap_or("application/octet-stream");
            HttpResponse::Ok()
                .content_type(mime)
                .insert_header(("Cache-Control", "private, max-age=3600"))
                .body(bytes)
        }
        Err(error) => {
            log::error!("field: photo {} unreadable: {error}", path);
            NoDataResponse::new("Photo not found".to_string(), 404).into_response()
        }
    }
}

// ---------------------------------------------------------------------------
// Account provisioning. The admin creates the login and receives the random
// password exactly once; the hash is the only copy kept, so nobody can read it
// back later. Resetting issues a new password and drops open sessions.
// ---------------------------------------------------------------------------

fn generate_password() -> Option<String> {
    // No O/0/I/l: these passwords are read off a screen and typed on a phone.
    const CHARSET: &[u8] = b"ABCDEFGHJKMNPQRSTUVWXYZabcdefghjkmnpqrstuvwxyz23456789";
    let mut bytes = [0u8; 12];
    getrandom::getrandom(&mut bytes).ok()?;
    Some(
        bytes
            .iter()
            .map(|byte| CHARSET[*byte as usize % CHARSET.len()] as char)
            .collect(),
    )
}

fn valid_nickname(nickname: &str) -> bool {
    (3..=30).contains(&nickname.len())
        && nickname
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
}

// POST /api/field/accounts
#[post("/accounts")]
pub async fn field_create_account(
    pool: web::Data<AppState>,
    bearer: Option<ReqData<String>>,
    body: web::Json<Value>,
) -> impl Responder {
    if let Ok(resp) = check_action_access(pool.db.clone(), bearer, "users", Action::Add).await {
        return resp;
    }

    let Some(employee_id) = body.get("employee_id").and_then(Value::as_i64) else {
        return NoDataResponse::new("employee_id wajib diisi".to_string(), 400).into_response();
    };
    let nickname = body
        .get("nickname")
        .and_then(Value::as_str)
        .map(str::trim)
        .unwrap_or("");
    if !valid_nickname(nickname) {
        return NoDataResponse::new(
            "Nickname 3-30 karakter, hanya huruf, angka, titik, garis bawah, strip".to_string(),
            400,
        )
        .into_response();
    }

    let employee = sqlx::query(
        "SELECT employee_id, employee_fname, employee_lname, employee_user_nickname FROM employee WHERE employee_id = $1",
    )
    .bind(employee_id as i32)
    .fetch_optional(&pool.db)
    .await;
    let employee = match employee {
        Ok(Some(row)) => row,
        Ok(None) => return NoDataResponse::new("Karyawan tidak ditemukan".to_string(), 404).into_response(),
        Err(error) => {
            log::error!("field account: employee lookup failed: {error}");
            return NoDataResponse::new("Database error".to_string(), 500).into_response();
        }
    };
    if employee
        .try_get::<Option<String>, _>("employee_user_nickname")
        .ok()
        .flatten()
        .is_some()
    {
        return NoDataResponse::new(
            "Karyawan ini sudah punya akun, pakai reset password".to_string(),
            409,
        )
        .into_response();
    }

    let taken = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM users WHERE user_nickname = $1)",
    )
    .bind(nickname)
    .fetch_one(&pool.db)
    .await
    .unwrap_or(true);
    if taken {
        return NoDataResponse::new("Nickname sudah dipakai".to_string(), 409).into_response();
    }

    let Some(password) = generate_password() else {
        return NoDataResponse::new("Gagal membuat password".to_string(), 500).into_response();
    };
    let hash = crate::utility::hash::hash_password(&password).await;

    let mut tx = match pool.db.begin().await {
        Ok(tx) => tx,
        Err(error) => {
            log::error!("field account: begin failed: {error}");
            return NoDataResponse::new("Database error".to_string(), 500).into_response();
        }
    };

    let created = sqlx::query_as::<_, (i32,)>(
        r#"INSERT INTO users (user_nickname, user_password, user_fname, user_lname, user_field_only)
           VALUES ($1, $2, $3, $4, TRUE) RETURNING user_id"#,
    )
    .bind(nickname)
    .bind(&hash)
    .bind(employee.get::<String, _>("employee_fname"))
    .bind(employee.try_get::<Option<String>, _>("employee_lname").ok().flatten())
    .fetch_one(&mut *tx)
    .await;

    let user_id = match created {
        Ok((user_id,)) => user_id,
        Err(error) => {
            log::error!("field account: insert user failed: {error}");
            return NoDataResponse::new("Database error".to_string(), 500).into_response();
        }
    };

    let linked = sqlx::query("UPDATE employee SET employee_user_nickname = $1 WHERE employee_id = $2")
        .bind(nickname)
        .bind(employee_id as i32)
        .execute(&mut *tx)
        .await;
    if let Err(error) = linked {
        log::error!("field account: link failed: {error}");
        return NoDataResponse::new("Database error".to_string(), 500).into_response();
    }

    if let Err(error) = sqlx::query(
        r#"INSERT INTO user_page_permissions (user_nickname, page_key, can_view, can_add, can_edit, can_delete)
           VALUES ($1, 'field', false, true, false, false)"#,
    )
    .bind(nickname)
    .execute(&mut *tx)
    .await
    {
        log::error!("field account: permission failed: {error}");
        return NoDataResponse::new("Database error".to_string(), 500).into_response();
    }

    if let Err(error) = tx.commit().await {
        log::error!("field account: commit failed: {error}");
        return NoDataResponse::new("Database error".to_string(), 500).into_response();
    }

    GenericResponse::single(
        json!({
            "employee_id": employee_id,
            "user_id": user_id,
            "nickname": nickname,
            "password": password, // shown once; only the hash is stored
        }),
        "Akun dibuat. Catat password sekarang, tidak bisa dilihat lagi.".to_string(),
    )
    .into_response()
}

// POST /api/field/accounts/{employee_id}/reset
#[post("/accounts/{employee_id}/reset")]
pub async fn field_reset_password(
    pool: web::Data<AppState>,
    bearer: Option<ReqData<String>>,
    employee_id: web::Path<i32>,
) -> impl Responder {
    if let Ok(resp) = check_action_access(pool.db.clone(), bearer, "users", Action::Edit).await {
        return resp;
    }

    let employee_id = employee_id.into_inner();
    let nickname = sqlx::query_scalar::<_, String>(
        "SELECT employee_user_nickname FROM employee WHERE employee_id = $1 AND employee_user_nickname IS NOT NULL",
    )
    .bind(employee_id)
    .fetch_optional(&pool.db)
    .await
    .ok()
    .flatten();
    let Some(nickname) = nickname else {
        return NoDataResponse::new("Karyawan ini belum punya akun".to_string(), 404).into_response();
    };

    let Some(password) = generate_password() else {
        return NoDataResponse::new("Gagal membuat password".to_string(), 500).into_response();
    };
    let hash = crate::utility::hash::hash_password(&password).await;

    let updated = sqlx::query(
        "UPDATE users SET user_password = $1, update_at = date_part('epoch', CURRENT_TIMESTAMP)::integer WHERE user_nickname = $2 RETURNING user_id",
    )
    .bind(&hash)
    .bind(&nickname)
    .fetch_optional(&pool.db)
    .await;

    let user_id = match updated {
        Ok(Some(row)) => row.get::<i32, _>("user_id"),
        Ok(None) => return NoDataResponse::new("Akun tidak ditemukan".to_string(), 404).into_response(),
        Err(error) => {
            log::error!("field account: reset failed: {error}");
            return NoDataResponse::new("Database error".to_string(), 500).into_response();
        }
    };

    // The old password must stop working everywhere, including phones already
    // signed in; dropping the sessions is the only way to do that.
    let _ = sqlx::query("DELETE FROM sessions WHERE session_user_id = $1")
        .bind(user_id)
        .execute(&pool.db)
        .await;

    GenericResponse::single(
        json!({ "employee_id": employee_id, "nickname": nickname, "password": password }),
        "Password baru dibuat. Sesi lama sudah dikeluarkan.".to_string(),
    )
    .into_response()
}

// GET /api/field/accounts?q=&limit=&offset=
#[derive(Debug, serde::Deserialize)]
pub struct FieldAccountQuery {
    pub q: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[get("/accounts")]
pub async fn field_list_accounts(
    pool: web::Data<AppState>,
    bearer: Option<ReqData<String>>,
    query: web::Query<FieldAccountQuery>,
) -> impl Responder {
    if let Ok(resp) = check_action_access(pool.db.clone(), bearer, "users", Action::View).await {
        return resp;
    }

    // One page at a time: the account tab is a lookup tool, not a directory.
    let q = query.q.as_deref().unwrap_or("").trim().to_string();
    let limit = query.limit.unwrap_or(50).clamp(1, 100);
    let offset = query.offset.unwrap_or(0).max(0);

    let rows = sqlx::query(
        r#"SELECT e.employee_id,
                  trim(e.employee_fname || ' ' || COALESCE(e.employee_lname, '')) AS employee_name,
                  e.employee_department, e.employee_status,
                  e.employee_user_nickname, u.user_nickname IS NOT NULL AS has_user
           FROM employee e
           LEFT JOIN users u ON u.user_nickname = e.employee_user_nickname
           WHERE $1 = ''
              OR trim(e.employee_fname || ' ' || COALESCE(e.employee_lname, '')) ILIKE '%' || $1 || '%'
              OR COALESCE(e.employee_department, '') ILIKE '%' || $1 || '%'
           ORDER BY e.employee_fname, e.employee_lname, e.employee_id
           LIMIT $2 OFFSET $3"#,
    )
    .bind(&q)
    .bind(limit)
    .bind(offset)
    .fetch_all(&pool.db)
    .await;

    let total = sqlx::query_scalar::<_, i64>(
        r#"SELECT count(*) FROM employee e
           WHERE $1 = ''
              OR trim(e.employee_fname || ' ' || COALESCE(e.employee_lname, '')) ILIKE '%' || $1 || '%'
              OR COALESCE(e.employee_department, '') ILIKE '%' || $1 || '%'"#,
    )
    .bind(&q)
    .fetch_one(&pool.db)
    .await;

    match (rows, total) {
        (Ok(rows), Ok(total)) => {
            let data: Vec<Value> = rows
                .into_iter()
                .map(|row| {
                    json!({
                        "employee_id": row.get::<i32, _>("employee_id"),
                        "employee_name": row.get::<String, _>("employee_name"),
                        "department": row.try_get::<Option<String>, _>("employee_department").ok().flatten(),
                        "employee_status": row.get::<i32, _>("employee_status"),
                        "nickname": row.try_get::<Option<String>, _>("employee_user_nickname").ok().flatten(),
                        "has_user": row.get::<bool, _>("has_user"),
                    })
                })
                .collect();
            GenericResponse::<serde_json::Value>::ok(
                vec![serde_json::json!({ "items": data, "total": total })],
                "OK".to_string(),
            )
            .into_response()
        }
        (Err(error), _) | (_, Err(error)) => {
            log::error!("field account: list failed: {error}");
            NoDataResponse::new("Database error".to_string(), 500).into_response()
        }
    }
}

// DELETE /api/field/photos/{id} - the employee retakes a blurry photo. Removing
// evidence re-opens the day: an approved day cannot stay approved after its
// photo disappears.
#[actix_web::delete("/photos/{id}")]
pub async fn field_delete_photo(
    pool: web::Data<AppState>,
    bearer: Option<ReqData<String>>,
    id: web::Path<Uuid>,
) -> impl Responder {
    if let Ok(resp) = check_action_access(pool.db.clone(), bearer.clone(), "field", Action::Add).await {
        return resp;
    }

    let (_, employee_id) = match caller_employee(&pool.db, bearer).await {
        Ok(Some(found)) => found,
        Ok(None) => {
            return NoDataResponse::new(
                "Akun ini belum terhubung ke data karyawan".to_string(),
                403,
            )
            .into_response();
        }
        Err(resp) => return resp,
    };

    let photo_id = id.into_inner();
    let row = sqlx::query(
        r#"SELECT p.file_path, p.server_at, a.id AS attendance_id, a.employee_id, a.status
           FROM field_photo p JOIN field_attendance a ON a.id = p.attendance_id
           WHERE p.id = $1"#,
    )
    .bind(photo_id)
    .fetch_optional(&pool.db)
    .await
    .ok()
    .flatten();
    let Some(row) = row else {
        return NoDataResponse::new("Foto tidak ditemukan".to_string(), 404).into_response();
    };
    if row.get::<i32, _>("employee_id") != employee_id {
        return NoDataResponse::new("Foto tidak ditemukan".to_string(), 404).into_response();
    }

    // Ten minutes is long enough to notice a blurry shot and short enough that a
    // deletion cannot erase evidence after the fact. The client hides the button,
    // this check is what actually enforces it.
    let age = Utc::now().signed_duration_since(row.get::<DateTime<Utc>, _>("server_at"));
    if age.num_seconds() > DELETE_WINDOW_SECS {
        return NoDataResponse::new(
            "Foto sudah lebih dari 10 menit, tidak bisa dihapus. Hubungi admin.".to_string(),
            409,
        )
        .into_response();
    }

    let attendance_id = row.get::<Uuid, _>("attendance_id");
    if let Err(error) = sqlx::query("DELETE FROM field_photo WHERE id = $1")
        .bind(photo_id)
        .execute(&pool.db)
        .await
    {
        log::error!("field: delete photo failed: {error}");
        return NoDataResponse::new("Database error".to_string(), 500).into_response();
    }
    let _ = std::fs::remove_file(row.get::<String, _>("file_path"));

    let remaining: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM field_photo WHERE attendance_id = $1")
        .bind(attendance_id)
        .fetch_one(&pool.db)
        .await
        .unwrap_or(0);
    if remaining == 0 {
        let _ = sqlx::query("DELETE FROM field_attendance WHERE id = $1")
            .bind(attendance_id)
            .execute(&pool.db)
            .await;
    } else {
        let _ = sqlx::query(
            r#"UPDATE field_attendance
               SET status = 'pending', reviewed_by = NULL, reviewed_at = NULL, review_reason = NULL, updated_at = NOW()
               WHERE id = $1"#,
        )
        .bind(attendance_id)
        .execute(&pool.db)
        .await;
    }

    NoDataResponse::new("Foto dihapus".to_string(), 200).into_response()
}

// GET /field-app - the employee app is one static file, served from the same
// origin as the API so the PWA needs no separate build or CORS entry.
#[get("/field-app")]
pub async fn field_app_page() -> impl Responder {
    let path = std::env::var("FIELD_PWA_FILE")
        .unwrap_or_else(|_| "../LuLu-Field/index.html".to_string());
    match std::fs::read_to_string(&path) {
        Ok(html) => HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .insert_header(("Cache-Control", "no-store"))
            .body(html),
        Err(error) => {
            log::error!("field app: cannot read {path}: {error}");
            NoDataResponse::new("Field app not deployed".to_string(), 404).into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::photo_kind;

    #[test]
    fn checkout_is_suggested_after_the_shift_ends() {
        // Day shift 08:00-16:00.
        assert!(!super::after_shift_end(9 * 3600, 8 * 3600, 16 * 3600));
        assert!(super::after_shift_end(21 * 3600, 8 * 3600, 16 * 3600));
        // Night shifts are left to the punch evidence, never to the clock.
        assert!(!super::after_shift_end(23 * 3600, 22 * 3600, 6 * 3600));
        assert!(!super::after_shift_end(7 * 3600, 22 * 3600, 6 * 3600));
        assert!(!super::after_shift_end(20 * 3600, 22 * 3600, 6 * 3600));
    }

    #[test]
    fn accepts_only_real_images() {
        assert_eq!(photo_kind(&[0xFF, 0xD8, 0xFF, 0xE0]), Some(("image/jpeg", "jpg")));
        assert_eq!(photo_kind(&[0x89, b'P', b'N', b'G', 0x0D]), Some(("image/png", "png")));
        assert_eq!(photo_kind(b"<html>"), None);
        assert_eq!(photo_kind(&[]), None);
    }
}
