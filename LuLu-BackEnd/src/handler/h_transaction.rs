use crate::{
    models::m_transaction::*,
    utility::{
        authorization::{Action, check_action_access},
        stor::{AppState, GenericResponse, IntoResponse, NoDataResponse},
    },
};
use actix_web::{
    Responder, get,
    web::{self, ReqData},
};

#[get("/transaction/live")]
pub async fn transaction_live(
    pool: web::Data<AppState>,
    bearer: Option<ReqData<String>>,
) -> impl Responder {
    if let Ok(resp) = check_action_access(pool.db.clone(), bearer, "dashboard", Action::View).await
    {
        return resp;
    }

    //match sqlx::query_as::<_, LiveTransaction>(r#"WITH query_data AS (SELECT (EXTRACT(EPOCH FROM enrolls.enroll_time)::INTEGER) AS enroll_time, enrolls.enroll_type, enrolls.enroll_status, employee.employee_fname, employee.employee_lname, devices.device_location, enrolls.enroll_id FROM (( enrolls INNER JOIN employee ON employee.employee_id = enrolls.enroll_employee_id) INNER JOIN devices ON enrolls.enroll_device_sn = devices.device_sn ) ORDER BY enrolls.enroll_time DESC LIMIT 100) SELECT * FROM query_data"#)
    //.bind(as_epoch("enrolls.enroll_time"))
    match sqlx::query_as::<_, LiveTransaction>(r#"SELECT enrolls.enroll_time, enrolls.enroll_type, enrolls.enroll_op_code, enrolls.enroll_status, employee.employee_fname, employee.employee_lname, employee.employee_id, devices.device_location, devices.device_timezone, employee.employee_departement, enrolls.enroll_id FROM (( enrolls INNER JOIN employee ON employee.employee_id = enrolls.enroll_employee_id) INNER JOIN devices ON enrolls.enroll_device_sn = devices.device_sn ) ORDER BY enrolls.enroll_time DESC LIMIT 100"#)
    .fetch_all(&pool.db)
    .await
    {
        Ok(data) => {
            let ids = match data.first(){
                Some(id)=> id.enroll_id,
                None => 0
            };
            GenericResponse::<LiveTransaction>::ok(
                data.clone(),
                format!("{}", ids),
            ).into_response()
        },
        Err(e) => {
            log::error!("Error: {}", e);

            NoDataResponse::new(
                "Internal server error".to_string(),
                500
            ).into_response()
        }
    }
}
