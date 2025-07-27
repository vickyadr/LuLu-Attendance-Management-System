use crate::{
    utility::{
        stor::{AppState, GenericResponse, NoDataResponse},
        authorization::is_login,
    },
    models::{m_transaction::*},
};
use actix_web::{get, web::{self, ReqData}, HttpResponse, Responder};

#[get("/transaction/live")]
pub async fn transaction_live(pool: web::Data<AppState>, bearer: Option<ReqData<String>>) -> impl Responder {
    match is_login(pool.db.clone(), bearer).await {
    Some(level) =>{
        if level == 0 {
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
    
    //match sqlx::query_as::<_, LiveTransaction>(r#"WITH query_data AS (SELECT (EXTRACT(EPOCH FROM enrolls.enroll_time)::INTEGER) AS enroll_time, enrolls.enroll_type, enrolls.enroll_status, employee.employee_fname, employee.employee_lname, devices.device_location, enrolls.enroll_id FROM (( enrolls INNER JOIN employee ON employee.employee_id = enrolls.enroll_employee_id) INNER JOIN devices ON enrolls.enroll_device_sn = devices.device_sn ) ORDER BY enrolls.enroll_time DESC LIMIT 100) SELECT * FROM query_data"#)
    //.bind(as_epoch("enrolls.enroll_time"))
    match sqlx::query_as::<_, LiveTransaction>(r#"SELECT enrolls.enroll_time, enrolls.enroll_type, enrolls.enroll_status, employee.employee_fname, employee.employee_lname, devices.device_location, enrolls.enroll_id FROM (( enrolls INNER JOIN employee ON employee.employee_id = enrolls.enroll_employee_id) INNER JOIN devices ON enrolls.enroll_device_sn = devices.device_sn ) ORDER BY enrolls.enroll_time DESC LIMIT 100"#)
    .fetch_all(&pool.db)
    .await
    {
        Ok(data) => {
            let ids = match data.first(){
                Some(id)=> id.enroll_id,
                None => 0
            };
            return HttpResponse::Ok().json(GenericResponse::<LiveTransaction>::ok(
                data.clone(),
                format!("{}", ids),
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