use crate::{
    models::m_shift::*,
    receiver::r_shift::*,
    utility::{
        authorization::{Action, check_action_access},
        stor::{AppState, GenericResponse, IntoResponse, KeyValResponse, NoDataResponse},
    },
};
use actix_web::{
    Responder, get, post,
    web::{self, ReqData},
};
use std::collections::HashMap;

#[get("/shift/list")]
pub async fn shift_list(
    pool: web::Data<AppState>,
    bearer: Option<ReqData<String>>,
) -> impl Responder {
    if let Ok(resp) = check_action_access(pool.db.clone(), bearer, "shift", Action::View).await {
        return resp;
    }

    match sqlx::query_as::<_, Shifts>(r#"SELECT * FROM shifts ORDER BY shift_name"#)
        .fetch_all(&pool.db)
        .await
    {
        Ok(data) => GenericResponse::<Shifts>::ok(data, "OK".to_string()).into_response(),
        Err(_) => NoDataResponse::ok("No list".to_string()).into_response(),
    }
}

#[get("/shift/delete/{id}")]
pub async fn shift_delete(
    path: web::Path<i32>,
    pool: web::Data<AppState>,
    bearer: Option<ReqData<String>>,
) -> impl Responder {
    let id = path.into_inner();

    if let Ok(resp) = check_action_access(pool.db.clone(), bearer, "shift", Action::Delete).await {
        return resp;
    }

    match sqlx::query(r#"DELETE FROM shifts WHERE shift_id=$1"#)
        .bind(id)
        .execute(&pool.db)
        .await
    {
        Ok(r) if r.rows_affected() == 1 => {
            NoDataResponse::ok("Shift deleted !!!".to_string()).into_response()
        }
        // Either the id never existed, or the row was already gone in another
        // tab. Either way there is nothing to report success about.
        Ok(_) => NoDataResponse::new("Shift not found".to_string(), 404).into_response(),
        Err(_) => NoDataResponse::new("Shift not found".to_string(), 404).into_response(),
    }
}

#[post("/shift/add")]
pub async fn shift_add(
    pool: web::Data<AppState>,
    body: web::Json<ReceiverShift>,
    bearer: Option<ReqData<String>>,
) -> impl Responder {
    let mut data: HashMap<&str, String> = HashMap::new();

    if let Ok(resp) = check_action_access(pool.db.clone(), bearer, "shift", Action::Add).await {
        return resp;
    }

    match body.name.clone() {
        Some(o) => {
            if o.len().lt(&3) {
                data.insert("name", "Too short".to_string());
            }
        }
        None => {
            data.insert("name", "Field required".to_string());
        }
    }

    match body.start_time {
        Some(o) => {
            if o.lt(&0) || o.gt(&86400) {
                data.insert("start_time", "Must be between 0 and 86400".to_string());
            }
        }
        None => {
            data.insert("start_time", "Field required".to_string());
        }
    }

    match body.end_time {
        Some(o) => {
            if o.lt(&0) || o.gt(&86400) {
                data.insert("end_time", "Must be between 0 and 86400".to_string());
            }
        }
        None => {
            data.insert("end_time", "Field required".to_string());
        }
    }

    match body.start_enroll {
        Some(o) => {
            if o.lt(&0) || o.gt(&86400) {
                data.insert("start_enroll", "Must be between 0 and 86400".to_string());
            }
        }
        None => {
            data.insert("start_enroll", "Field required".to_string());
        }
    }

    match body.end_enroll {
        Some(o) => {
            if o.lt(&0) || o.gt(&86400) {
                data.insert("end_enroll", "Must be between 0 and 86400".to_string());
            }
        }
        None => {
            data.insert("end_enroll", "Field required".to_string());
        }
    }

    if !data.is_empty() {
        return KeyValResponse::<&str, String>::new(
            data,
            "There's was error while adding shift".to_string(),
            403,
        )
        .into_response();
    }

    //let start_enroll = body.start_time.unwrap() - body.start_enroll.unwrap();
    //let end_enroll = body.end_time.unwrap() + body.end_enroll.unwrap();
    let start_enroll = body.start_time.unwrap() - body.start_enroll.unwrap();
    let end_enroll = body.end_time.unwrap() + body.end_enroll.unwrap();
    let prevday = if start_enroll < 0 || start_enroll > body.end_time.unwrap() {
        1
    } else {
        0
    };
    let passday = if end_enroll > 86400 { 1 } else { 0 };

    match sqlx::query_as::<_, Shifts>(r#"INSERT INTO shifts (shift_name, shift_start_time, shift_end_time, shift_start_enroll, shift_end_enroll, shift_passday, shift_prevday) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING *"#)
                .bind(body.name.clone())
                .bind(body.start_time)
                .bind(body.end_time)
                .bind(start_enroll)
                .bind(end_enroll)
                .bind(passday)
                .bind(prevday)
                .fetch_one(&pool.db)
                .await
        {
            Ok(data) => GenericResponse::<Shifts>::single(
                                                data,
                                                "Shift added !!!".to_string()
                                            ).into_response(),
            Err(e) => {
                log::error!("{}", e);
                let code = e.as_database_error().map(|d| d.message()).unwrap_or("");
                if code.contains("duplicate key value violates") {
                    return NoDataResponse::new(
                        "This shift name has been use!".to_string(),
                        405
                    ).into_response();
                }
                NoDataResponse::new(
                    "Internal error, please try again!".to_string(),
                    500
                ).into_response()
            }
        }
}

#[post("/shift/edit")]
pub async fn shift_edit(
    pool: web::Data<AppState>,
    body: web::Json<ReceiverShift>,
    bearer: Option<ReqData<String>>,
) -> impl Responder {
    let mut data: HashMap<&str, String> = HashMap::new();

    if let Ok(resp) = check_action_access(pool.db.clone(), bearer, "shift", Action::Edit).await {
        return resp;
    }

    match body.id {
        Some(o) => {
            if o.lt(&1) {
                data.insert("id", "Id not found".to_string());
            }
        }
        None => {
            data.insert("id", "Required".to_string());
        }
    }

    match body.name.clone() {
        Some(o) => {
            if o.len().lt(&3) {
                data.insert("name", "Too short".to_string());
            }
        }
        None => {
            data.insert("name", "Field required".to_string());
        }
    }

    match body.start_time {
        Some(o) => {
            if o.lt(&0) || o.gt(&86400) {
                data.insert("start_time", "Must be between 0 and 86400".to_string());
            }
        }
        None => {
            data.insert("start_time", "Field required".to_string());
        }
    }

    match body.end_time {
        Some(o) => {
            if o.lt(&0) || o.gt(&86400) {
                data.insert("end_time", "Must be between 0 and 86400".to_string());
            }
        }
        None => {
            data.insert("end_time", "Field required".to_string());
        }
    }

    match body.start_enroll {
        Some(o) => {
            if o.lt(&0) || o.gt(&86400) {
                data.insert("start_enroll", "Must be between 0 and 86400".to_string());
            }
        }
        None => {
            data.insert("start_enroll", "Field required".to_string());
        }
    }

    match body.end_enroll {
        Some(o) => {
            if o.lt(&0) || o.gt(&86400) {
                data.insert("end_enroll", "Must be between 0 and 86400".to_string());
            }
        }
        None => {
            data.insert("end_enroll", "Field required".to_string());
        }
    }

    if !data.is_empty() {
        return KeyValResponse::<&str, String>::new(
            data,
            "There's was error while edit shift".to_string(),
            403,
        )
        .into_response();
    }

    let start_enroll = body.start_time.unwrap() - body.start_enroll.unwrap();
    let end_enroll = body.end_time.unwrap() + body.end_enroll.unwrap();
    let prevday = if start_enroll < 0 || start_enroll > body.end_time.unwrap() {
        1
    } else {
        0
    };
    let passday = if end_enroll > 86400 { 1 } else { 0 };

    match sqlx::query_as::<_, Shifts>(r#"UPDATE shifts SET shift_name=$1, shift_start_time=$2, shift_end_time=$3, shift_start_enroll=$4, shift_end_enroll=$5, shift_passday=$6, shift_prevday=$7 WHERE shift_id=$8 RETURNING *"#)
                .bind(body.name.clone())
                .bind(body.start_time)
                .bind(body.end_time)
                .bind(start_enroll)
                .bind(end_enroll)
                .bind(passday)
                .bind(prevday)
                .bind(body.id)
                .fetch_one(&pool.db)
                .await
        {
            Ok(data) => GenericResponse::<Shifts>::single(
                                    data,
                                    "Data shift has been update !".to_string()
                                ).into_response(),
            Err(e) => {
                log::error!("{}", e);
                NoDataResponse::new(
                    "Internal error, please try again!".to_string(),
                    500
                ).into_response()
            }
        }
}
