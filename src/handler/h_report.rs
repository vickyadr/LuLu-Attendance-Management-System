/*
// ---------------- QUERY W/O PASS DAY
WITH query AS (
SELECT enroll_employee_id, to_timestamp(enroll_time)::DATE AS enroll_date, to_timestamp(enroll_time)::TIME AS enroll_time FROM enrolls WHERE to_timestamp(enroll_time)::TIME < '18:00' AND to_timestamp(enroll_time)::TIME > '6:00'
)

SELECT enroll_employee_id, enroll_date, MIN(enroll_time) AS start_enroll, MAX(enroll_time) As end_enroll
FROM query
GROUP BY enroll_employee_id, enroll_date

// ---------------  QUERY W/O PASS DAY via VIEWER
 WITH query_sch AS (
         SELECT devices.device_timezone,
            devices.device_name,
            devices.device_location,
            enrolls.enroll_device_sn,
            enrolls.enroll_type,
            enrolls.enroll_status,
            enrolls.enroll_time,
            employee.employee_id,
            employee.employee_fname,
            employee.employee_lname,
            employee.employee_address,
            employee.employee_departement,
            employee.employee_status,
            schedules.schedule_name,
            schedules.schedule_dom,
            schedules.schedule_type,
            schedules.schedule_hols,
            shifts.shift_start_time,
            shifts.shift_end_time,
            shifts.shift_start_enroll,
            shifts.shift_end_enroll,
            shifts.shift_passday,
            shifts.shift_name
           FROM ((((enrolls
             JOIN employee ON ((employee.employee_id = enrolls.enroll_employee_id)))
             JOIN schedules ON ((schedules.schedule_parrent = employee.employee_schedule_id)))
             JOIN shifts ON ((shifts.shift_id = schedules.schedule_shift_id)))
             JOIN devices ON (((enrolls.enroll_device_sn)::text = (devices.device_sn)::text)))
        ), query_filter_range AS (
         SELECT query_sch.device_timezone,
            query_sch.device_name,
            query_sch.device_location,
            query_sch.enroll_device_sn,
            query_sch.enroll_type,
            query_sch.enroll_status,
            query_sch.enroll_time,
            query_sch.employee_id,
            query_sch.employee_fname,
            query_sch.employee_lname,
            query_sch.employee_address,
            query_sch.employee_departement,
            query_sch.employee_status,
            query_sch.schedule_name,
            query_sch.schedule_dom,
            query_sch.schedule_type,
            query_sch.schedule_hols,
            query_sch.shift_start_time,
            query_sch.shift_end_time,
            query_sch.shift_start_enroll,
            query_sch.shift_end_enroll,
            query_sch.shift_passday,
            query_sch.shift_name,
            (date_part('dow'::text, (to_timestamp(((query_sch.enroll_time + (query_sch.device_timezone * 3600)))::double precision))::date) + (1)::double precision) AS enroll_dow,
            date_part('epoch'::text, (to_timestamp(((query_sch.enroll_time + (query_sch.device_timezone * 3600)))::double precision))::date) AS enroll_date
           FROM query_sch
          WHERE (((query_sch.schedule_type = 1) AND (query_sch.enroll_time >= ((date_part('epoch'::text, (to_timestamp((query_sch.enroll_time)::double precision))::date))::integer + (query_sch.shift_start_enroll - (query_sch.device_timezone * 3600)))) AND (query_sch.enroll_time <= ((date_part('epoch'::text, (to_timestamp((query_sch.enroll_time)::double precision))::date))::integer + (query_sch.shift_end_enroll - (query_sch.device_timezone * 3600))))) OR ((query_sch.schedule_type = 7) AND ((date_part('dow'::text, (to_timestamp(((query_sch.enroll_time + (query_sch.device_timezone * 3600)))::double precision))::date) + (1)::double precision) = (query_sch.schedule_dom)::double precision) AND (query_sch.enroll_time >= ((date_part('epoch'::text, (to_timestamp((query_sch.enroll_time)::double precision))::date))::integer + (query_sch.shift_start_enroll - (query_sch.device_timezone * 3600)))) AND (query_sch.enroll_time <= ((date_part('epoch'::text, (to_timestamp((query_sch.enroll_time)::double precision))::date))::integer + (query_sch.shift_end_enroll - (query_sch.device_timezone * 3600))))))
        ), query_group_day AS (
         SELECT query_filter_range.employee_id,
            query_filter_range.employee_fname,
            query_filter_range.employee_lname,
            query_filter_range.employee_departement,
            query_filter_range.enroll_dow,
            query_filter_range.enroll_date,
            query_filter_range.device_timezone,
            query_filter_range.device_name,
            query_filter_range.device_location,
            query_filter_range.shift_start_time,
            query_filter_range.shift_end_time,
            query_filter_range.shift_name,
            query_filter_range.schedule_name,
            min(query_filter_range.enroll_time) AS start_enroll,
            max(query_filter_range.enroll_time) AS end_enroll
           FROM query_filter_range
          GROUP BY query_filter_range.employee_id, query_filter_range.employee_fname, query_filter_range.employee_lname, query_filter_range.employee_departement, query_filter_range.enroll_dow, query_filter_range.enroll_date, query_filter_range.device_timezone, query_filter_range.device_name, query_filter_range.device_location, query_filter_range.shift_start_time, query_filter_range.shift_end_time, query_filter_range.shift_name, query_filter_range.schedule_name
        )
 SELECT query_group_day.employee_id,
    query_group_day.employee_fname,
    query_group_day.employee_lname,
    query_group_day.employee_departement,
    query_group_day.enroll_dow,
    query_group_day.enroll_date,
    query_group_day.device_timezone,
    query_group_day.device_name,
    query_group_day.device_location,
    query_group_day.shift_start_time,
    query_group_day.shift_end_time,
    query_group_day.shift_name,
    query_group_day.schedule_name,
    query_group_day.start_enroll,
    query_group_day.end_enroll
   FROM query_group_day;

// ---------------  QUERY DYNAMIC SCHEDULE
ON RESEARCH

*/

use crate::{
    utility::{
        stor::{AppState, GenericResponse, NoDataResponse},
        authorization::is_login,
    },
    models::{m_report::*},
};
use actix_web::{get, web::{self, ReqData}, HttpResponse, Responder};

#[get("/report/today")]
pub async fn report_today(pool: web::Data<AppState>, bearer: Option<ReqData<String>>) -> impl Responder {
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
    
    /*match sqlx::query_as::<_, Reports>(r#"WITH query AS (SELECT enroll_employee_id, to_timestamp(enroll_time)::DATE AS enroll_date_naive, date_part('epoch'::text, to_timestamp(enroll_time)::DATE)::INTEGER AS enroll_date, enroll_time FROM enrolls WHERE enroll_time >= (date_part('epoch'::text, to_timestamp(enroll_time)::DATE)::INTEGER + $1) AND enroll_time <= (date_part('epoch'::text, to_timestamp(enroll_time)::DATE)::INTEGER + $2)),
	                                        query_g AS (SELECT enroll_employee_id, MIN(enroll_date) AS enroll_date, MIN(enroll_time) AS start_enroll, MAX(enroll_time) As end_enroll FROM query GROUP BY enroll_employee_id, enroll_date_naive)
                                            SELECT employee_id, employee_fname, employee_lname, enroll_date, employee_departement, start_enroll, end_enroll FROM query_g INNER JOIN employee ON employee_id = enroll_employee_id ORDER BY enroll_date DESC"#)
    */
    match sqlx::query_as::<_, Reports>(r#"SELECT * FROM view_report ORDER BY enroll_date DESC"#)
    .fetch_all(&pool.db)
    .await
    {
        Ok(data) => {
            return HttpResponse::Ok().json(GenericResponse::<Reports>::ok(
                data,
                format!("OK"),
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