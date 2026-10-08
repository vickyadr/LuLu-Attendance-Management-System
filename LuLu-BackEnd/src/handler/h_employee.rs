use std::collections::HashMap;

use crate::{
    models::m_employee::*,
    receiver::r_employee::*,
    utility::{
        authorization::{Action, check_action_access},
        stor::{AppState, GenericResponse, IntoResponse, KeyValResponse, NoDataResponse},
    },
};

use actix_web::{
    Responder, get, post,
    web::{self, ReqData},
};

#[get("/employee/list")]
pub async fn employee_list(
    pool: web::Data<AppState>,
    bearer: Option<ReqData<String>>,
) -> impl Responder {
    if let Ok(resp) = check_action_access(pool.db.clone(), bearer, "employee", Action::View).await {
        return resp;
    }

    //match sqlx::query_as::<_, Employee>(r#"SELECT * FROM employee INNER JOIN schedules ON schedules.schedule_id = employee.employee_schedule_id ORDER BY employee.employee_fname ASC"#)
    match sqlx::query_as::<_, Employee>(
        r#"SELECT * FROM employee ORDER BY employee.employee_fname ASC"#,
    )
    .fetch_all(&pool.db)
    .await
    {
        Ok(data) => GenericResponse::<Employee>::ok(data, "ok".to_string()).into_response(),
        Err(e) => {
            log::error!("Error: {}", e);

            NoDataResponse::new("Internal server error".to_string(), 500).into_response()
        }
    }
}

#[post("/employee/add")]
pub async fn employee_add(
    pool: web::Data<AppState>,
    body: web::Json<ReceiverEmployee>,
    bearer: Option<ReqData<String>>,
) -> impl Responder {
    let mut data: HashMap<&str, String> = HashMap::new();

    if let Ok(resp) = check_action_access(pool.db.clone(), bearer, "employee", Action::Add).await {
        return resp;
    }

    match body.fname.clone() {
        Some(o) => {
            if o.len().lt(&3) {
                data.insert("first_name", "Too short".to_string());
            }
        }
        None => {
            data.insert("first_name", "Field required".to_string());
        }
    }

    match body.departement.clone() {
        Some(o) => {
            if o.len().lt(&3) {
                data.insert("departement", "Too short".to_string());
            }
        }
        None => {
            data.insert("departement", "Field required".to_string());
        }
    }

    match body.address.clone() {
        Some(o) => {
            if o.len().ge(&160) {
                data.insert("address", "Max 160 character".to_string());
            }
        }
        None => {
            data.insert("location", "Field required".to_string());
        }
    }

    if !data.is_empty() {
        return KeyValResponse::<&str, String>::new(
            data,
            "There's was error while adding employee".to_string(),
            403,
        )
        .into_response();
    }

    //match sqlx::query_as::<_, Employee>(r#"WITH query AS (INSERT INTO employee (employee_fname, employee_lname, employee_departement, employee_address, employee_status, employee_schedule_id) VALUES ($1, $2, $3, $4, $5, $6) RETURNING *) SELECT * FROM query INNER JOIN schedules ON schedules.schedule_id = employee_schedule_id"#)
    match sqlx::query_as::<_, Employee>(r#"INSERT INTO employee (employee_fname, employee_lname, employee_departement, employee_address, employee_status, employee_schedule_id) VALUES ($1, $2, $3, $4, $5, $6) RETURNING *"#)
                .bind(body.fname.clone())
                .bind(body.lname.clone())
                .bind(body.departement.clone())
                .bind(body.address.clone())
                .bind(body.status)
                .bind(body.schedule_id)
                .fetch_one(&pool.db)
                .await
        {
            Ok(data) => GenericResponse::<Employee>::single(
                                    data,
                                    "New employee registered !!!".to_string()
                                ).into_response(),
            Err(e) => {
                log::error!("{}", e);
                let code =  e.as_database_error().unwrap().message();
                if code.contains("duplicate key value violates") {
                    return NoDataResponse::new(
                        "This employee have registered!".to_string(),
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

#[get("/employee/delete/{id}")]
pub async fn employee_delete(
    path: web::Path<i32>,
    pool: web::Data<AppState>,
    bearer: Option<ReqData<String>>,
) -> impl Responder {
    let id = path.into_inner();

    if let Ok(resp) = check_action_access(pool.db.clone(), bearer, "employee", Action::Delete).await
    {
        return resp;
    }

    match sqlx::query(r#"DELETE FROM employee WHERE employee_id=$1"#)
        .bind(id)
        .execute(&pool.db)
        .await
    {
        Ok(_) => NoDataResponse::ok("Employee deleted".to_string()).into_response(),
        Err(_) => NoDataResponse::new("Employee not found".to_string(), 404).into_response(),
    }
}

#[post("/employee/edit")]
pub async fn employee_edit(
    pool: web::Data<AppState>,
    body: web::Json<ReceiverEmployee>,
    bearer: Option<ReqData<String>>,
) -> impl Responder {
    let mut data: HashMap<&str, String> = HashMap::new();

    if let Ok(resp) = check_action_access(pool.db.clone(), bearer, "employee", Action::Edit).await {
        return resp;
    }

    match body.fname.clone() {
        Some(o) => {
            if o.len().lt(&3) {
                data.insert("first_name", "Too short".to_string());
            }
        }
        None => {
            data.insert("first_name", "Field required".to_string());
        }
    }

    match body.departement.clone() {
        Some(o) => {
            if o.len().lt(&3) {
                data.insert("departement", "Too short".to_string());
            }
        }
        None => {
            data.insert("departement", "Field required".to_string());
        }
    }

    match body.address.clone() {
        Some(o) => {
            if o.len().ge(&160) {
                data.insert("address", "Max 160 character".to_string());
            }
        }
        None => {
            data.insert("location", "Field required".to_string());
        }
    }

    if !data.is_empty() {
        return KeyValResponse::<&str, String>::new(
            data,
            "There's was error while adding controller".to_string(),
            403,
        )
        .into_response();
    }

    match sqlx::query_as::<_, Employee>(r#"UPDATE employee SET employee_fname = $1 , employee_lname =$2 , employee_departement = $3, employee_address = $4, employee_status = $5, employee_schedule_id = $6 WHERE employee_id = $7 RETURNING *"#)
                .bind(body.fname.clone())
                .bind(body.lname.clone())
                .bind(body.departement.clone())
                .bind(body.address.clone())
                .bind(body.status)
                .bind(body.schedule_id)
                .bind(body.id)
                .fetch_one(&pool.db)
                .await
        {
            Ok(data) => GenericResponse::<Employee>::single(
                                    data,
                                    "Employee data updated !!!".to_string()
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
