use std::collections::HashMap;
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

    let today = ts_at_zero(Utc::now().timestamp(), 0);

    // Get schedules first
    let schedule_list = match get_schedules(&pool.db).await {
        Ok(schedules) => schedules,
        Err(_) => vec![]
    };

    // Then get reports data
    match get_reports_data(&pool.db, today, today).await {
        Ok(datas) => {
            let reports = get_enroll_data(today, today, datas, schedule_list);
            HttpResponse::Ok().json(GenericResponse::<Reports>::ok(
                reports,
                "OK".to_string(),
            ))
        }
        Err(e) => {
            println!("Database error: {}", e);
            HttpResponse::Ok().json(NoDataResponse::new(
                "Internal server error".to_string(),
                500
            ))
        }
    }
}
#[get("/report/range/{start}/{end}")]
pub async fn report_range(
    path: web::Path<(i64, i64)>, 
    pool: web::Data<AppState>, 
    bearer: Option<ReqData<String>>
) -> impl Responder {
    let (start, end) = path.into_inner();

    // Quick permission check
    match is_login(pool.db.clone(), bearer).await {
        Some(level) => {
            if level == 0 {
                return HttpResponse::Ok().json(NoDataResponse::new(
                    "Sorry, doesn't have permission".to_string(),
                    403
                ));
            }
        }
        None => return HttpResponse::Ok().json(NoDataResponse::new(
            "Session token invalid".to_string(),
            401
        ))
    }
    
    let start_range = ts_at_zero(start, 0);
    let end_range = ts_at_zero(end, 0);

    // Get schedules first
    let schedule_list = match get_schedules(&pool.db).await {
        Ok(schedules) => schedules,
        Err(_) => vec![]
    };

    // Then get reports data
    match get_reports_data(&pool.db, start_range, end_range).await {
        Ok(datas) => {
            let reports = get_enroll_data(start_range, end_range, datas, schedule_list);
            HttpResponse::Ok().json(GenericResponse::<Reports>::ok(
                reports,
                "OK".to_string(),
            ))
        }
        Err(e) => {
            println!("Database error: {}", e);
            HttpResponse::Ok().json(NoDataResponse::new(
                "Internal server error".to_string(),
                500
            ))
        }
    }
}

// Separate async functions for better concurrency
async fn get_schedules(db: &sqlx::PgPool) -> Result<Vec<ScheduleHelper>, sqlx::Error> {
    sqlx::query_as::<_, ScheduleHelper>(
        r#"SELECT schedules.schedule_name, COALESCE(shifts.shift_name,'Libur') as shift_name, COALESCE(shifts.shift_id,0) as shift_id, schedules.schedule_dom, schedules.schedule_parrent, schedules.schedule_type, schedules.schedule_hols, COALESCE(shifts.shift_start_time,0::BIGINT) as shift_start_time, COALESCE(shifts.shift_end_time,0::BIGINT) as shift_end_time, COALESCE(shifts.shift_start_enroll,0::BIGINT) as shift_start_enroll, COALESCE(shifts.shift_end_enroll,0::BIGINT) as shift_end_enroll, COALESCE(shifts.shift_passday,0::SMALLINT) as shift_passday, COALESCE(shifts.shift_prevday,0::SMALLINT) as shift_prevday FROM schedules 
           LEFT JOIN shifts ON shifts.shift_id = schedules.schedule_shift_id"#
    )
    .fetch_all(db)
    .await
}

async fn get_reports_data(
    db: &sqlx::PgPool, 
    start_range: i64, 
    end_range: i64
) -> Result<Vec<ReportsHelper>, sqlx::Error> {
    sqlx::query_as::<_, ReportsHelper>(
        r#"SELECT device_timezone, device_name, device_location, enroll_device_sn, 
                  enroll_type, enroll_status, enroll_time, employee_id, employee_fname, 
                  employee_lname, employee_address, employee_departement, employee_status, 
                  employee_schedule_id, schedule_type
           FROM enrolls e
           INNER JOIN employee emp ON emp.employee_id = e.enroll_employee_id
           INNER JOIN devices d ON e.enroll_device_sn = d.device_sn
           INNER JOIN schedules s ON s.schedule_id = emp.employee_schedule_id
           WHERE e.enroll_time BETWEEN $1 AND $2
           ORDER BY e.enroll_time ASC"#
    )
    // Pad 2 days on each side. One day covers a `prevday` shift whose enroll window opens on
    // the previous calendar day; the second covers a device whose timezone places local
    // midnight more than 24h away from the UTC midnight of the requested range. Without it,
    // punches at the range edges were never fetched and the report came back empty.
    .bind(start_range - (2 * 86400))
    .bind(end_range + (2 * 86400))
    .fetch_all(db)
    .await
}

fn get_enroll_data(
    start: i64,
    end: i64,
    collection: Vec<ReportsHelper>,
    schedule_list: Vec<ScheduleHelper>
) -> Vec<Reports> {
    if collection.is_empty() {
        return Vec::new();
    }

    // Pre-build optimized lookup structures
    let schedule_map: HashMap<(i32, i32), ScheduleHelper> = schedule_list
        .into_iter()
        .map(|d| ((d.schedule_parrent, d.schedule_dom), d))
        .collect();

    // Group enrollments by employee_id and date for faster processing
    let mut employee_enrollments: HashMap<i32, Vec<&ReportsHelper>> = HashMap::new();
    for enrollment in &collection {
        employee_enrollments
            .entry(enrollment.employee_id)
            .or_insert_with(Vec::new)
            .push(enrollment);
    }

    let mut results = Vec::new();

    // Process each employee's enrollments.
    //
    // The day loop deliberately stays in the shared UTC frame: `enroll_date` is a UTC
    // midnight and the frontend renders it as a plain calendar date, so the bucket identity
    // must not move when a device timezone changes. All timezone awareness lives in
    // get_shift_for_date / calculate_enrollment_window, which convert the local wall-clock
    // shift offsets to UTC epochs.
    for (_employee_id, enrollments) in employee_enrollments {
        let employee_reports = process_employee_enrollments(
            start,
            end,
            enrollments,
            &schedule_map
        );
        results.extend(employee_reports);
    }

    // Sort once at the end
    results.sort_unstable_by_key(|a| a.enroll_date);
    results
}

fn process_employee_enrollments(
    start: i64,
    end: i64,
    enrollments: Vec<&ReportsHelper>,
    schedule_map: &HashMap<(i32, i32), ScheduleHelper>
) -> Vec<Reports> {
    let mut daily_reports: HashMap<i64, Reports> = HashMap::new();

    // Process each day in the range. `current_date` is a UTC midnight and doubles as the
    // bucket key in `enroll_date`.
    for current_date in (start..=end).step_by(86400) {
        if let Some(enrollment) = enrollments.first() {
            if let Some(shift) = get_shift_for_date(current_date, enrollment, schedule_map) {
                let (start_window, end_window) = calculate_enrollment_window(
                    current_date, enrollment, &shift
                );

                // Find all enrollments for this day
                let day_enrollments: Vec<&ReportsHelper> = enrollments
                    .iter()
                    .filter(|e| e.enroll_time >= start_window && e.enroll_time <= end_window)
                    .copied()
                    .collect();

                if !day_enrollments.is_empty() {
                    if let Some(report) = create_daily_report(
                        current_date,
                        enrollment,
                        &shift,
                        &day_enrollments
                    ) {
                        daily_reports.insert(current_date, report);
                    }
                }
            }
        }
    }

    // Convert to vector and calculate working times
    daily_reports
        .into_values()
        .map(|mut report| {
            calculate_working_time(&mut report);
            report
        })
        .collect()
}


fn get_shift_for_date<'a>(
    current_date: i64,
    enrollment: &ReportsHelper,
    schedule_map: &'a HashMap<(i32, i32), ScheduleHelper>
) -> Option<&'a ScheduleHelper> {
    let schedule_type = enrollment.schedule_type;
    let pattern = if schedule_type < 7 { 1 } else { schedule_type / 7 };
    let dom_span = if schedule_type < 7 { 1 } else { 7 };

    let iso_dow = ts_dow(current_date, 0) as i32;
    let dow = (iso_dow - 1) % dom_span + 1;
    let week = (ts_week(current_date, 0) as i32) % if pattern < 1 { 1 } else { pattern };

    let key = (enrollment.employee_schedule_id as i32, (dow + (week * 7)) as i32);
    if let Some(s) = schedule_map.get(&key) {
        if s.shift_id == 0 || s.shift_name == "Libur" {
            return None;
        }
    }
    schedule_map.get(&key)
}


fn calculate_enrollment_window(
    current_date: i64,
    enrollment: &ReportsHelper,
    shift: &ScheduleHelper
) -> (i64, i64) {
    let timezone_offset = (enrollment.device_timezone as i64) * 3600;

    // Local midnight of the named calendar day, as a UTC epoch.
    let local_midnight = current_date - timezone_offset;

    // A prevday shift opens its enroll window on the PREVIOUS calendar day.
    let start_window = local_midnight + if shift.shift_prevday == 1 {
        shift.shift_start_enroll - 86400
    } else {
        shift.shift_start_enroll
    };

    let end_window = local_midnight + shift.shift_end_enroll;

    (start_window, end_window)
}

fn create_daily_report(
    current_date: i64,
    enrollment: &ReportsHelper,
    shift: &ScheduleHelper,
    day_enrollments: &[&ReportsHelper]
) -> Option<Reports> {
    // Find first and last enrollments for the day
    let first_enrollment = day_enrollments.first()?.enroll_time;
    let last_enrollment = if day_enrollments.len() > 1 {
        Some(day_enrollments.last()?.enroll_time)
    } else {
        None
    };

    let schedule_type = enrollment.schedule_type;
    let pattern = if schedule_type < 7 { 1 } else { schedule_type / 7 };
    let dom_span = if schedule_type < 7 { 1 } else { 7 };

    // Same frame as get_shift_for_date: the weekday of the named calendar day, mapped onto
    // dom without wrapping, so `enroll_dow` agrees with the schedule row that was selected.
    let iso_dow = ts_dow(current_date, 0) as i32;
    let dow = (iso_dow - 1) % dom_span + 1;
    let week = (ts_week(current_date, 0) as i32) % if pattern < 1 { 1 } else { pattern };

    Some(Reports {
        employee_id: enrollment.employee_id,
        employee_fname: enrollment.employee_fname.clone(),
        employee_lname: enrollment.employee_lname.clone(),
        employee_departement: enrollment.employee_departement.clone(),
        enroll_dow: Some((dow + (week * 7)).into()),
        enroll_date: Some(current_date),
        device_timezone: Some(enrollment.device_timezone),
        device_name: enrollment.device_name.clone(),
        device_location: Some(enrollment.device_location.clone()),
        shift_start_time: Some(shift.shift_start_time),
        shift_end_time: Some(shift.shift_end_time),
        shift_name: Some(shift.shift_name.clone()),
        schedule_name: Some(shift.schedule_name.clone()),
        start_enroll: Some(first_enrollment),
        end_enroll: last_enrollment,
        working_time: 0,
        late_time: 0,
    })
}

fn calculate_working_time(report: &mut Reports) {
    let Some(enroll_date) = report.enroll_date else { return };
    let Some(device_timezone) = report.device_timezone else { return };
    let Some(shift_start) = report.shift_start_time else { return };
    let Some(shift_end) = report.shift_end_time else { return };

    let timezone_offset = (device_timezone * 3600) as i64;

    // `enroll_date` is a UTC midnight; the shift's own start/end are wall-clock durations
    // from LOCAL midnight. Convert the anchor to local midnight once, then add the shift
    // offsets in that same frame. The original code added the offset to the start bound and
    // subtracted it from the end bound, which stretched or compressed the shift span by
    // twice the offset and pushed the reference point a day off for non-zero timezones.
    let local_midnight = enroll_date - timezone_offset;
    let start_working = local_midnight +
        if shift_start >= shift_end { shift_start - 86400 } else { shift_start };
    let end_working = local_midnight + shift_end;
    
    let shift_duration = end_working - start_working;
    let half_shift = shift_duration / 2;

    // Calculate late time
    if let Some(start_enroll) = report.start_enroll {
        if start_enroll > start_working {
            let late = start_enroll - start_working;
            report.late_time = if late < 60 { 0 } else { late.min(shift_duration) };
        }
        
        // Handle case where employee arrived after half shift without checkout
        if start_enroll > start_working + half_shift && report.end_enroll.is_none() {
            report.end_enroll = report.start_enroll;
            report.start_enroll = None;
        }
    }

    // Calculate working time: actual presence between first IN and last OUT.
    // The old formula measured from shift start (`end_enroll - start_working`),
    // which inflated the number whenever someone arrived late and hid early
    // leave entirely (OUT before shift end still showed a full shift).
    if let (Some(start_enroll), Some(end_enroll)) = (report.start_enroll, report.end_enroll) {
        report.working_time = end_enroll.saturating_sub(start_enroll);
    } else if let Some(end_enroll) = report.end_enroll {
        // Single-punch corner (late arrival reclassified as checkout-only above):
        // fall back to time since shift start so the row still carries a duration.
        report.working_time = end_enroll.saturating_sub(start_working);
    }
}