use std::{collections::HashMap};
use crate::{
    models::{m_report::*, m_schedule::ScheduleHelper}, utility::{
        authorization::is_login, helper::{ts_at_zero, ts_dow, ts_week}, stor::{AppState, GenericResponse, NoDataResponse}
    }
};
use actix_web::{get, web::{self, ReqData}, HttpResponse, Responder};
use chrono::Utc;

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
    
    let schedule_list = match sqlx::query_as::<_, ScheduleHelper>(r#"SELECT * FROM schedules INNER JOIN shifts ON shifts.shift_id = schedules.schedule_shift_id"#)
    .fetch_all(&pool.db)
    .await {
        Ok(res)=> res,
        Err(_)=> vec![]
    };

    let today = ts_at_zero(Utc::now().timestamp(), 0);

    match sqlx::query_as::<_, ReportsHelper>(r#"SELECT device_timezone, device_name, device_location, enroll_device_sn, enroll_type, enroll_status, enroll_time, employee_id, employee_fname, employee_lname, employee_address, employee_departement, employee_status, employee_schedule_id, schedule_type
		FROM (((enrolls INNER JOIN employee ON employee.employee_id = enrolls.enroll_employee_id)
		INNER JOIN devices ON enrolls.enroll_device_sn = devices.device_sn)
		INNER JOIN schedules ON schedules.schedule_id = employee.employee_schedule_id) WHERE enroll_time > $1 ORDER BY enroll_time ASC"#)
    .bind(today - 86400)
    .fetch_all(&pool.db)
    .await
    {
        Ok(datas) => {
            return HttpResponse::Ok().json(GenericResponse::<Reports>::ok(
                get_enroll_data(today, today, datas, schedule_list),
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

#[get("/report/range/{start}/{end}")]
pub async fn report_range(path: web::Path<(i64, i64)>, pool: web::Data<AppState>, bearer: Option<ReqData<String>>) -> impl Responder {
    let (start, end) = path.into_inner();

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
    
    let schedule_list = match sqlx::query_as::<_, ScheduleHelper>(r#"SELECT * FROM schedules INNER JOIN shifts ON shifts.shift_id = schedules.schedule_shift_id"#)
    .fetch_all(&pool.db)
    .await {
        Ok(res)=> res,
        Err(_)=> vec![]
    };

    let start_range = ts_at_zero(start, 0);
    let end_range = ts_at_zero(end, 0);
    println!("S {}", start_range);
    println!("E {}", end_range);
    match sqlx::query_as::<_, ReportsHelper>(r#"SELECT device_timezone, device_name, device_location, enroll_device_sn, enroll_type, enroll_status, enroll_time, employee_id, employee_fname, employee_lname, employee_address, employee_departement, employee_status, employee_schedule_id, schedule_type
		FROM (((enrolls INNER JOIN employee ON employee.employee_id = enrolls.enroll_employee_id)
		INNER JOIN devices ON enrolls.enroll_device_sn = devices.device_sn)
		INNER JOIN schedules ON schedules.schedule_id = employee.employee_schedule_id) WHERE enroll_time > $1 AND enroll_time <= $2 ORDER BY enroll_time ASC"#)
    .bind(start_range - 86400)
    .bind(end_range + 86400)
    .fetch_all(&pool.db)
    .await
    {
        Ok(datas) => {
            return HttpResponse::Ok().json(GenericResponse::<Reports>::ok(
                get_enroll_data(start_range, end_range, datas, schedule_list),
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

fn get_enroll_data(start: i64, end: i64, collection: Vec<ReportsHelper>, schedule_list: Vec<ScheduleHelper>) -> Vec<Reports>{

    let mut schedule = HashMap::<(i32, i32), ScheduleHelper>::new();
    
    for d in schedule_list
    {
        schedule.insert((d.schedule_parrent, d.schedule_dom), d);
    }

    let mut start_date : i64 = start;
    let end_date : i64 = end;//ts_at_zero(1754179200, 0);
    let mut data_filtered: Vec<Reports> = Vec::<Reports>::new();
    let mut results: HashMap::<(i32, i64),Reports> = HashMap::new();
    while start_date <= end_date {
        for d in &collection {
            let pattern:i32 = d.schedule_type / 7;
            let dow: i32 = (ts_dow(start_date, d.device_timezone) as i32 % if d.schedule_type < 7 {1}else{7}) +1;
            let week: i32 = ts_week(start_date, d.device_timezone) as i32 % if pattern < 1 {1}else{pattern};
            let shift = schedule.get(&(d.employee_schedule_id, dow + (week * 7))).unwrap();
            let end_enroll = start_date + shift.shift_end_enroll - (d.device_timezone * 3600) as i64;
            let start_enroll = start_date - (d.device_timezone * 3600) as i64 + if shift.shift_prevday == 1 {
                shift.shift_start_enroll - 86400
            }else{
                shift.shift_start_enroll
            };
            
            if !(d.enroll_time >= start_enroll && d.enroll_time <= end_enroll) {
                continue;
            }

            if results.contains_key(&(d.employee_id, start_date)) {
                match results.get_mut(&(d.employee_id, start_date)) {
                    Some(p) =>{
                        p.end_enroll = Some(d.enroll_time);
                    }
                    None=> ()
                }
            }else{
                results.insert((d.employee_id, start_date), Reports { 
                    employee_id: d.employee_id,
                    employee_fname: d.employee_fname.clone(),
                    employee_lname: d.employee_lname.clone(),
                    employee_departement: d.employee_departement.clone(),
                    enroll_dow: Some((dow + (week * 7)).into()),
                    enroll_date: Some(start_date),
                    device_timezone: Some(d.device_timezone),
                    device_name: d.device_name.clone(),
                    device_location: Some(d.device_location.clone()),
                    shift_start_time: Some(shift.shift_start_time),
                    shift_end_time: Some(shift.shift_end_time),
                    shift_name: Some(shift.shift_name.clone()),
                    schedule_name: Some(shift.schedule_name.clone()),
                    start_enroll: Some(d.enroll_time),
                    end_enroll: None,
                    working_time: 0,
                    late_time:0 });
            }
        }

        start_date += 86400;
    }

    for d in results {
        let mut chk = d.1;
        let end_working = chk.enroll_date.unwrap() + chk.shift_end_time.unwrap() - (chk.device_timezone.unwrap() * 3600) as i64;
        let start_working = chk.enroll_date.unwrap() - (chk.device_timezone.unwrap() * 3600) as i64 + if chk.shift_start_time >= chk.shift_end_time {
                chk.shift_start_time.unwrap() - 86400
            }else{
                chk.shift_start_time.unwrap()
            };
        let half = (end_working - start_working) / 2;

        match chk.start_enroll {
            Some(i) => {
                chk.late_time = 0;
                if i > start_working {
                    chk.late_time = i - start_working;
                }
                if chk.late_time < 60 {
                    chk.late_time = 0;
                }
                if chk.late_time > end_working - start_working {
                    chk.late_time = end_working - start_working;
                }
            },
            None => ()
        }

        if chk.start_enroll.unwrap() > start_working + half && chk.end_enroll.is_none() {
            chk.end_enroll = chk.start_enroll;
            chk.start_enroll = None;
        }

        match chk.end_enroll {
            Some(i) => chk.working_time = i - start_working,
            None => ()
        }

        data_filtered.push(chk);
    }
    
    data_filtered.sort_by(|a, b| a.enroll_date.cmp(&b.enroll_date));
    
    return data_filtered

}

