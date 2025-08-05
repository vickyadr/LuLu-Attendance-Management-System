use std::collections::HashMap;

use crate::{
    models::m_employee::*,
    receiver::r_employee::*,
    utility::{
        authorization::is_login, 
        stor::{
            AppState, GenericResponse, NoDataResponse, KeyValResponse
        }
    }
};

use actix_web::{get, post, web::{self, ReqData}, HttpResponse, Responder};

#[get("/employee/list")]
pub async fn employee_list(pool: web::Data<AppState>, bearer: Option<ReqData<String>>) -> impl Responder {
    
    match is_login(pool.db.clone(), bearer).await {
        Some(level) =>{
            if level.lt(&100) {
                return HttpResponse::Ok().json(NoDataResponse::new(
                    format!("Sorry, doesnt have permission"),
                    403
                ))
            }
        },
        None => return HttpResponse::Ok().json(NoDataResponse::new(
                    format!("Session token invalid"),
                    401
                ))
    }


    //match sqlx::query_as::<_, Employee>(r#"SELECT * FROM employee INNER JOIN schedules ON schedules.schedule_id = employee.employee_schedule_id ORDER BY employee.employee_fname ASC"#)
    match sqlx::query_as::<_, Employee>(r#"SELECT * FROM employee ORDER BY employee.employee_fname ASC"#)
    .fetch_all(&pool.db)
    .await
    {
        Ok(data) => {
            return HttpResponse::Ok().json(GenericResponse::<Employee>::ok(
                data,
                format!("ok"),
            ));
        },
        Err(e) => {
            println!("Error: {}", e);

            return HttpResponse::Ok().json(NoDataResponse::new(
                format!("Internal server error"),
                500
            ));                            
        }
    }
}

#[post("/employee/add")]
pub async fn employee_add(pool: web::Data<AppState>, body: web::Json<ReceiverEmployee>, bearer: Option<ReqData<String>>) -> impl Responder {
    let mut data: HashMap<&str, String> = HashMap::new();
    
    match is_login(pool.db.clone(), bearer).await {
        Some(level) =>{
            if level.lt(&100) {
                return HttpResponse::Ok().json(NoDataResponse::new(
                    format!("Sorry, doesnt have permission"),
                    403
                ))
            }
        },
        None => return HttpResponse::Ok().json(NoDataResponse::new(
                    format!("Session token invalid"),
                    401
                ))
    }

    match body.fname.clone() {
        Some(o) => {
            if o.len().lt(&3) {
                data.insert("first_name", format!("Too short"));
            }
        },
        None => {
            data.insert("first_name", format!("Field required"));
        }
    }

    match body.departement.clone() {
        Some(o) => {
            if o.len().lt(&3) {
                data.insert("departement", format!("Too short"));
            }
        },
        None => {
            data.insert("departement", format!("Field required"));
        }
    }

    match body.address.clone() {
        Some(o) => {
            if o.len().ge(&160) {
                data.insert("address", format!("Max 160 character"));
            }
        },
        None => {
            data.insert("location", format!("Field required"));
        }
    }

    if data.len() > 0 {
        return HttpResponse::Ok().json(KeyValResponse::<&str, String>::new(
            data,
            format!("There's was error while adding employee"),
            403
        ));
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
            Ok(data) => return HttpResponse::Ok().json(GenericResponse::<Employee>::single(
                                    data,
                                    format!("New employee registered !!!")
                                )),
            Err(e) => {
                println!("{}", e);
                let code =  e.as_database_error().unwrap().message();
                if code.contains("duplicate key value violates") {
                    return HttpResponse::Ok().json(NoDataResponse::new(
                        format!("This employee have registered!"),
                        405
                    ));
                }
                return HttpResponse::Ok().json(NoDataResponse::new(
                    format!("Internal error, please try again!"),
                    500
                ));
            }
        }
}

#[get("/employee/delete/{id}")]
pub async fn employee_delete(path: web::Path<i32>, pool: web::Data<AppState>, bearer: Option<ReqData<String>>) -> impl Responder {
    let id = path.into_inner();
    
    match is_login(pool.db.clone(), bearer).await {
        Some(level) =>{
            if level.lt(&100) {
                return HttpResponse::Ok().json(NoDataResponse::new(
                    format!("Sorry, doesnt have permission"),
                    403
                ))
            }
        },
        None => return HttpResponse::Ok().json(NoDataResponse::new(
                    format!("Session token invalid"),
                    401
                ))
    }

    match sqlx::query(r#"DELETE FROM employee WHERE employee_id=$1"#)
    .bind(id)
    .execute(&pool.db)
    .await
    {
        Ok(_) => return HttpResponse::Ok().json(NoDataResponse::ok(
                                format!("Employee deleted")
                            )),
        Err(_) => return HttpResponse::Ok().json(NoDataResponse::new(
                                format!("Employee not found"),
                                404
                            ))
    }
        
}

#[post("/employee/edit")]
pub async fn employee_edit(pool: web::Data<AppState>, body: web::Json<ReceiverEmployee>, bearer: Option<ReqData<String>>) -> impl Responder {
    let mut data: HashMap<&str, String> = HashMap::new();
    
    match is_login(pool.db.clone(), bearer).await {
        Some(level) =>{
            if level.lt(&100) {
                return HttpResponse::Ok().json(NoDataResponse::new(
                    format!("Sorry, doesnt have permission"),
                    403
                ))
            }
        },
        None => return HttpResponse::Ok().json(NoDataResponse::new(
                    format!("Session token invalid"),
                    401
                ))
    }

    match body.fname.clone() {
        Some(o) => {
            if o.len().lt(&3) {
                data.insert("first_name", format!("Too short"));
            }
        },
        None => {
            data.insert("first_name", format!("Field required"));
        }
    }

    match body.departement.clone() {
        Some(o) => {
            if o.len().lt(&3) {
                data.insert("departement", format!("Too short"));
            }
        },
        None => {
            data.insert("departement", format!("Field required"));
        }
    }

    match body.address.clone() {
        Some(o) => {
            if o.len().ge(&160) {
                data.insert("address", format!("Max 160 character"));
            }
        },
        None => {
            data.insert("location", format!("Field required"));
        }
    }

    if data.len() > 0 {
        return HttpResponse::Ok().json(KeyValResponse::<&str, String>::new(
            data,
            format!("There's was error while adding controller"),
            403
        ));
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
            Ok(data) => return HttpResponse::Ok().json(GenericResponse::<Employee>::single(
                                    data,
                                    format!("Employee data updated !!!")
                                )),
            Err(e) => {
                println!("{}", e);
                return HttpResponse::Ok().json(NoDataResponse::new(
                    format!("Internal error, please try again!"),
                    500
                ));
            }
        }
}