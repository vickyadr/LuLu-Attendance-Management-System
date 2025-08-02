use crate::{
    models::m_schedule::*, receiver::r_schedule::*, utility::{authorization::is_login, stor::{AppState, GenericResponse, KeyValResponse, NoDataResponse}}
};
use actix_web::{get, post, web::{self, ReqData}, HttpResponse, Responder};
use std::collections::HashMap;

#[get("/schedule/list")]
pub async fn schedule_list(pool: web::Data<AppState>, bearer: Option<ReqData<String>>) -> impl Responder {

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

    match sqlx::query_as::<_, Schedules>(r#"SELECT * FROM schedules INNER JOIN shifts ON shifts.shift_id = schedules.schedule_shift_id ORDER BY schedule_parrent"#)
    .fetch_all(&pool.db)
    .await
    {
        Ok(data) => return HttpResponse::Ok().json(GenericResponse::<Schedules>::ok(
                                data,
                                format!("OK")
                            )),
        Err(e) =>
        {
            println!("E: {:?}", e);
            return HttpResponse::Ok().json(NoDataResponse::ok (
                                format!("Internal server error"),
                            ))
        }
    }
        
}

#[get("/schedule/delete/{id}")]
pub async fn schedule_delete(path: web::Path<i32>, pool: web::Data<AppState>, bearer: Option<ReqData<String>>) -> impl Responder {
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

    match sqlx::query(r#"DELETE FROM schedules WHERE schedule_parrent=$1"#)
    .bind(id)
    .execute(&pool.db)
    .await
    {
        Ok(_) => return HttpResponse::Ok().json(NoDataResponse::ok (
                                format!("Schedule deleted !!!")
                            )),
        Err(_) => return HttpResponse::Ok().json(NoDataResponse::new (
                                format!("Schedule not found"),
                                404
                            ))
    }
        
}

#[post("/schedule/add")]
pub async fn schedule_add(pool: web::Data<AppState>, body: web::Json<ReceiverSchedule>, bearer: Option<ReqData<String>>) -> impl Responder {
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

    match body.name.clone() {
        Some(o) => {
            if o.len().lt(&3) {
                data.insert("name", format!("Too short"));
            }
        },
        None => {
            data.insert("name", format!("Field required"));
        }
    }

    match body.shift_id.clone() {
        Some(o) => {
            if o.len().lt(&1) {
                data.insert("shift", format!("Please select"));
            }
        },
        None => {
            data.insert("name", format!("Field required"));
        }
    }
    
    match body.pattern.clone() {
        Some(_) => (),
        None => {
            data.insert("name", format!("Field required"));
        }
    }

    if data.len() > 0 {
        return HttpResponse::Ok().json(KeyValResponse::<&str, String>::new(
            data,
            format!("There's was error while adding schedules"),
            403
        ));
    }

    println!("SHIFT : {:?}", body.shift_id.clone().unwrap());

    match sqlx::query_scalar::<_, i32>(r#"INSERT INTO schedules (schedule_name, schedule_shift_id, schedule_dom, schedule_parrent) VALUES ($1, $2, $3, $4) RETURNING schedule_id"#)
                .bind(body.name.clone())
                .bind(body.shift_id.clone().unwrap().get(0).unwrap())
                .bind(1)
                .bind(0)
                .fetch_one(&pool.db)
                .await
        {
            //) UPDATE schedules SET schedules.parrent = query WHERE schedules.schedule_id = query RETUNING *
            Ok(parrent_id) => {
                    println!("PARRENT : {:?}", parrent_id);
                match sqlx::query("UPDATE schedules SET schedule_parrent = $1 WHERE schedule_id = $1")
                    .bind(parrent_id)
                    .execute(&pool.db)
                    .await{
                        Ok(_)=>(),
                        Err(e)=> println!("ERROR : {:?}", e)
                    }
                
                let mut i = 0;
                for shift_id in body.shift_id.clone().unwrap()
                {
                    if i == 0 {
                        i += 1;
                        continue;
                    }

                    match sqlx::query("INSERT INTO schedules (schedule_name, schedule_shift_id, schedule_dom, schedule_parrent) VALUES ($1, $2, $3, $4)")
                    .bind(body.name.clone())
                    .bind(shift_id)
                    .bind(1)
                    .bind(parrent_id)
                    .execute(&pool.db)
                    .await{
                        Ok(_)=>(),
                        Err(_)=>()
                    }

                    i += 1;
                }
                
                return HttpResponse::Ok().json(NoDataResponse::ok(
                    format!("Schedule added !!!")
                ));

                /*return HttpResponse::Ok().json(GenericResponse::<Schedules>::single(
                                                data,
                                                format!("Schedule added !!!")
                                            ))*/
            },
            Err(e) => {
                println!("{}", e);
                let code =  e.as_database_error().unwrap().message();
                if code.contains("duplicate key value violates") {
                    return HttpResponse::Ok().json(NoDataResponse::new(
                        format!("This shift name has been use!"),
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

#[post("/schedule/edit")]
pub async fn schedule_edit(pool: web::Data<AppState>, body: web::Json<ReceiverSchedule>, bearer: Option<ReqData<String>>) -> impl Responder {
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

    match body.id.clone() {
        Some(o) => {
            if o.lt(&1) {
                data.insert("id", format!("Id not found"));
            }
        },
        None => {
            data.insert("id", format!("Required"));
        }
    }

    match body.name.clone() {
        Some(o) => {
            if o.len().lt(&3) {
                data.insert("name", format!("Too short"));
            }
        },
        None => {
            data.insert("name", format!("Field required"));
        }
    }

    if data.len() > 0 {
        return HttpResponse::Ok().json(KeyValResponse::<&str, String>::new(
            data,
            format!("There's was error while edit shift"),
            403
        ));
    }


    match sqlx::query_as::<_, Schedules>(r#"UPDATE schedules SET shift_name=$1, shift_start_time=$2, shift_end_time=$3, shift_start_enroll=$4, shift_end_enroll=$5, shift_passday=$6 WHERE shift_id=$7 RETURNING *"#)
                .bind(body.name.clone())
                .bind(body.pattern)
                //.bind(body.shift_id)
                .fetch_one(&pool.db)
                .await
        {
            Ok(data) => return HttpResponse::Ok().json(GenericResponse::<Schedules>::single(
                                    data,
                                    format!("Data shift has been update !")
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