use std::collections::HashMap;

use actix_web::{HttpResponse, http::StatusCode};
use serde::Serialize;
use sqlx::PgPool;

use crate::utility::wshub::WsHub;

pub struct AppState {
    pub db: PgPool,
    /// Live-feed broadcast hub. Held here so every worker shares one hub: a
    /// `web::Data<WsHub>` added separately would work too, but AppState is
    /// already threaded through every handler, so enrolling handlers stay free
    /// of an extra extractor.
    pub ws: WsHub,
}

/// Map an application `code` onto the matching HTTP status so the API answers
/// with a real status line instead of `200 OK` for every error.
///
/// The `code` field stays in the JSON body: the frontend reads it directly, and
/// `$fetch` throws on non-2xx, so callers must read `err.data` for the message.
fn status_for(code: i32) -> StatusCode {
    match code {
        c if (200..300).contains(&c) => StatusCode::OK,
        c if (300..400).contains(&c) => StatusCode::SEE_OTHER,
        400 => StatusCode::BAD_REQUEST,
        401 => StatusCode::UNAUTHORIZED,
        403 => StatusCode::FORBIDDEN,
        404 => StatusCode::NOT_FOUND,
        409 => StatusCode::CONFLICT,
        c if (500..600).contains(&c) => StatusCode::INTERNAL_SERVER_ERROR,
        _ => StatusCode::BAD_REQUEST,
    }
}

/// Build the `HttpResponse` for a response envelope, carrying the real HTTP
/// status for `code`. Replaces the bare `...into_response()` pattern so no
/// handler can silently report an error with a 200.
pub fn respond<T: Serialize>(code: i32, body: &T) -> HttpResponse {
    HttpResponse::build(status_for(code)).json(body)
}

/// Implement `into_response()` on each envelope so handlers read as
/// `NoDataResponse::new(...).into_response()`.
pub trait IntoResponse {
    fn into_response(self) -> HttpResponse;
}

impl IntoResponse for NoDataResponse {
    fn into_response(self) -> HttpResponse {
        respond(self.code, &self)
    }
}

impl<T: Serialize> IntoResponse for GenericResponse<T> {
    fn into_response(self) -> HttpResponse {
        respond(self.code, &self)
    }
}

impl<K: Serialize, V: Serialize> IntoResponse for KeyValResponse<K, V> {
    fn into_response(self) -> HttpResponse {
        respond(self.code, &self)
    }
}

impl<K: Serialize, V: Serialize> IntoResponse for WildData<K, V> {
    fn into_response(self) -> HttpResponse {
        respond(self.code, &self)
    }
}
#[derive(Serialize)]
pub struct NoDataResponse {
    pub message: String,
    pub data: Vec<i8>,
    pub code: i32,
    /// i18n key of this error, e.g. `payroll.payslipsExistPeriod`. The frontend
    /// passes it straight to `$t`, so a gate never needs an English message and
    /// no mapping table: one name, one place. `None` serializes as `null`, which
    /// the client treats as "no translation".
    pub error: Option<String>,
}

#[derive(Serialize)]
pub struct GenericResponse<T> {
    pub message: String,
    pub data: Vec<T>,
    pub code: i32,
}

#[derive(Serialize)]
pub struct KeyValResponse<K, V> {
    pub message: String,
    pub data: HashMap<K, V>,
    pub code: i32,
}

#[derive(Serialize)]
pub struct WildData<K, V> {
    pub wild: K,
    pub value: V,
    pub code: i32,
}

impl NoDataResponse {
    pub fn new(msg: String, e: i32) -> NoDataResponse {
        NoDataResponse {
            message: msg,
            data: Vec::new(),
            code: e,
            error: None,
        }
    }

    /// Error the frontend can translate: `error` carries the i18n key, `message`
    /// stays as the English fallback for logs and API clients.
    pub fn coded(key: &str, msg: String, e: i32) -> NoDataResponse {
        NoDataResponse {
            message: msg,
            data: Vec::new(),
            code: e,
            error: Some(key.to_string()),
        }
    }

    pub fn ok(msg: String) -> NoDataResponse {
        NoDataResponse {
            message: msg,
            data: Vec::new(),
            code: 200,
            error: None,
        }
    }
}

impl<T> GenericResponse<T> {
    pub fn new(dt: Vec<T>, msg: String, e: i32) -> GenericResponse<T> {
        GenericResponse::<T> {
            message: msg,
            data: dt,
            code: e,
        }
    }

    pub fn ok(dt: Vec<T>, msg: String) -> GenericResponse<T> {
        GenericResponse::<T> {
            message: msg,
            data: dt,
            code: 200,
        }
    }

    pub fn single(dt: T, msg: String) -> GenericResponse<T> {
        let mut data_single = Vec::<T>::new();
        data_single.insert(0, dt);
        GenericResponse::<T> {
            message: msg,
            data: data_single,
            code: 200,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{status_for, NoDataResponse};
    use actix_web::http::StatusCode;

    #[test]
    fn maps_conflict_envelopes_to_http_409() {
        assert_eq!(status_for(409), StatusCode::CONFLICT);
    }

    /// The frontend translates `error` by passing it straight to `$t`, so the
    /// key must round-trip and `None` must stay falsy on the wire.
    #[test]
    fn coded_error_serializes_key_and_plain_responses_omit_it() {
        let coded = NoDataResponse::coded("payroll.payslipsExistPeriod", "raw".into(), 409);
        let json = serde_json::to_value(&coded).expect("serializes");
        assert_eq!(json["error"], "payroll.payslipsExistPeriod");
        assert_eq!(json["code"], 409);

        let plain = serde_json::to_value(NoDataResponse::new("raw".into(), 500)).expect("serializes");
        assert!(plain["error"].is_null(), "no key when untranslated");

        let ok = serde_json::to_value(NoDataResponse::ok("done".into())).expect("serializes");
        assert!(ok["error"].is_null(), "success carries no error key");
    }
}

impl<K, V> KeyValResponse<K, V> {
    pub fn new(dt: HashMap<K, V>, msg: String, e: i32) -> KeyValResponse<K, V> {
        KeyValResponse::<K, V> {
            message: msg,
            data: dt,
            code: e,
        }
    }

    pub fn ok(dt: HashMap<K, V>, msg: String) -> KeyValResponse<K, V> {
        KeyValResponse::<K, V> {
            message: msg,
            data: dt,
            code: 200,
        }
    }
}
