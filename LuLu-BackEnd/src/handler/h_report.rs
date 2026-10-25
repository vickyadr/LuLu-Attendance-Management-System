// Reports are bucketed per calendar day, and the timestamp carried in `enroll_date` is a
// UTC midnight (the frontend renders it as a plain date). The original code, however,
// derived the schedule day-of-week from `ts_dow(current_date, device_timezone)` - i.e. it
// shifted the UTC-midnight anchor forward by the offset before taking the weekday. For a
// +07:00 device that turned the anchor into 07:00 local, so the schedule key was read off
// the NEXT local day; for a negative offset it became the PREVIOUS local day. Either way
// the key no longer matched the day the punch landed on, the shift resolved to a different
// schedule row (often "Libur"), and the whole day vanished from the report without any
// error. The enrollment window had the mirror-image bug: it subtracted the offset to build
// the base, then re-added it on the end bound, so the window came out 24h wide and shifted
// by the offset instead of spanning the shift.
//
// Fix: keep every day anchor in UTC (unchanged public contract) and derive the weekday,
// week and enrollment window from that SAME anchor, with the offset applied exactly once and
// only where it genuinely belongs - converting local wall-clock shift offsets into UTC
// epochs. Day buckets are then correct for any device timezone.
use crate::{
    models::{m_report::*, m_schedule::ScheduleHelper},
    utility::{
        authorization::{Action, check_action_access},
        helper::{ts_at_zero, ts_dow, ts_week},
        stor::{AppState, GenericResponse, IntoResponse, NoDataResponse},
    },
};
use actix_web::{
    Responder, get,
    web::{self, ReqData},
};
use chrono::Utc;
use sqlx::Row;
use std::collections::{HashMap, HashSet};

/// Tolerances (in seconds) applied when classifying a day as late or overtime.
/// Read from the `settings` table so an admin can change them from the Settings page
/// and the values take effect for every report without a rebuild.
#[derive(Clone, Copy)]
pub struct Tolerances {
    pub late_sec: i64,
    pub overtime_sec: i64,
}

pub async fn load_tolerances(db: &sqlx::PgPool) -> Tolerances {
    let row = sqlx::query(
        r#"SELECT
             COALESCE(MAX(setting_value) FILTER (WHERE setting_key='late_tolerance_sec'), $1::BIGINT) AS late_tol,
             COALESCE(MAX(setting_value) FILTER (WHERE setting_key='overtime_tolerance_sec'), $2::BIGINT) AS overtime_tol
           FROM settings"#
    )
    .bind(60i64)
    .bind(60i64)
    .fetch_one(db)
    .await;

    match row {
        Ok(r) => Tolerances {
            late_sec: r.get::<i64, _>("late_tol"),
            overtime_sec: r.get::<i64, _>("overtime_tol"),
        },
        // Settings table missing (migration not applied) - keep the historical 60s behaviour.
        Err(_) => Tolerances {
            late_sec: 60,
            overtime_sec: 60,
        },
    }
}

/// The set of non-working days covering one report request.
///
/// A day is a holiday when the `holiday` table has a row for that calendar date, whatever the
/// kind (decree holiday, collective leave, or a company closure). The set is resolved ONCE per
/// request rather than per day: the table is small (a few dozen rows per year) and the loop
/// below walks every day of the requested range, so a per-day query would multiply a single
/// report into hundreds of round trips.
///
/// Dates are collected as epoch seconds at UTC midnight, which is exactly how the day loop
/// names a day (`current_date`), so membership is a plain integer comparison. That matters:
/// the day loop deliberately keeps every anchor in the shared UTC frame (see the module
/// comment), so converting these to device-local midnights here would reintroduce the
/// timezone-dependent bucketing this file was rewritten to remove.
#[derive(Clone, Debug, Default)]
pub(crate) struct HolidaySet {
    /// Every holiday in range: these days produce no row unless also included.
    days: HashSet<i64>,
    /// The subset that must still be counted, plus the name for labelling.
    included: HashMap<i64, String>,
}

impl HolidaySet {
    fn contains(&self, day_utc_midnight: i64) -> bool {
        self.days.contains(&day_utc_midnight)
    }

    fn included_name(&self, day_utc_midnight: i64) -> Option<&str> {
        self.included.get(&day_utc_midnight).map(String::as_str)
    }
}

/// The approved absences covering one report request, keyed per employee.
///
/// Leave is per PERSON, so it cannot live in the shared `HolidaySet`: two people
/// can be on the same roster row, and only one of them is on leave. The set is
/// loaded once per request for the same reason as holidays - the day loop below
/// walks every day of the range, so a per-day query would multiply one report
/// into hundreds of round trips.
///
/// Only 'approved' rows are collected. A 'pending' request has not been granted
/// yet and must not erase real attendance, and a 'rejected' one is a decision not
/// to excuse the day. Both belong in the panel, not in the report.
#[derive(Clone, Debug, Default)]
pub(crate) struct LeaveSet {
    days: HashMap<i32, HashSet<i64>>,
}

impl LeaveSet {
    fn contains(&self, employee_id: i32, day_utc_midnight: i64) -> bool {
        self.days
            .get(&employee_id)
            .is_some_and(|set| set.contains(&day_utc_midnight))
    }
}

/// Load every APPROVED leave falling inside the requested range.
pub(crate) async fn load_leaves(db: &sqlx::PgPool, start: i64, end: i64) -> LeaveSet {
    let rows = sqlx::query(
        r#"SELECT leave_employee_id, (EXTRACT(EPOCH FROM leave_date))::BIGINT AS day_epoch
           FROM leave_requests
           WHERE leave_status = 'approved'
             AND leave_date >= to_timestamp($1) AT TIME ZONE 'UTC'
             AND leave_date <= to_timestamp($2) AT TIME ZONE 'UTC'"#,
    )
    .bind(start)
    .bind(end)
    .fetch_all(db)
    .await;

    match rows {
        Ok(rs) => {
            let mut days: HashMap<i32, HashSet<i64>> = HashMap::new();
            for r in rs {
                days.entry(r.get::<i32, _>("leave_employee_id"))
                    .or_default()
                    .insert(r.get::<i64, _>("day_epoch"));
            }
            LeaveSet { days }
        }
        // Degrade to "nobody is on leave" rather than failing the report: a
        // missing table must not blank every attendance row.
        Err(e) => {
            log::error!("failed to load leave requests: {}", e);
            LeaveSet::default()
        }
    }
}

/// Load every holiday whose date falls inside the requested range.
///
/// On a query error the set is empty rather than absent: a missing table (migration not
/// applied) or a database hiccup must degrade to the historical behaviour of ignoring
/// holidays, never to a report that hides every day it could not classify.
pub(crate) async fn load_holidays(db: &sqlx::PgPool, start: i64, end: i64) -> HolidaySet {
    let rows = sqlx::query(
        r#"SELECT (EXTRACT(EPOCH FROM holiday_date))::BIGINT AS day_epoch,
                  holiday_name, COALESCE(holiday_included, FALSE) AS holiday_included
           FROM holiday
           WHERE holiday_date >= to_timestamp($1) AT TIME ZONE 'UTC'
             AND holiday_date <= to_timestamp($2) AT TIME ZONE 'UTC'"#,
    )
    .bind(start)
    .bind(end)
    .fetch_all(db)
    .await;

    match rows {
        Ok(rs) => {
            let mut days = HashSet::new();
            let mut included = HashMap::new();
            for r in rs {
                let day: i64 = r.get("day_epoch");
                days.insert(day);
                // An old database that has not run the holiday_included migration has
                // no such column: try_get fails closed to "excluded", which is the
                // historical behaviour, rather than failing the whole report.
                let flag: bool = r.try_get("holiday_included").unwrap_or(false);
                if flag {
                    included.insert(day, r.get::<String, _>("holiday_name"));
                }
            }
            HolidaySet { days, included }
        }
        Err(e) => {
            log::error!("failed to load holidays: {}", e);
            HolidaySet::default()
        }
    }
}

#[get("/report/today")]
pub async fn report_today(
    pool: web::Data<AppState>,
    bearer: Option<ReqData<String>>,
) -> impl Responder {
    if let Ok(resp) = check_action_access(pool.db.clone(), bearer, "report", Action::View).await {
        return resp;
    }

    let today = ts_at_zero(Utc::now().timestamp(), 0);

    // Get schedules first
    let schedule_list = get_schedules(&pool.db).await.unwrap_or_default();

    let tolerances = load_tolerances(&pool.db).await;

    // A holiday set for today alone; the gate below drops the day, so a day that is a
    // non-working day produces no report row regardless of what was punched on it.
    let holidays = load_holidays(&pool.db, today, today).await;
    let leaves = load_leaves(&pool.db, today, today).await;

    // Then get reports data
    match get_reports_data(&pool.db, today, today).await {
        Ok(datas) => {
            let reports = get_enroll_data(today, today, datas, schedule_list, tolerances, &holidays, &leaves);
            GenericResponse::<Reports>::ok(reports, "OK".to_string()).into_response()
        }
        Err(e) => {
            log::error!("Database error: {}", e);
            NoDataResponse::new("Internal server error".to_string(), 500).into_response()
        }
    }
}
#[get("/report/range/{start}/{end}")]
pub async fn report_range(
    path: web::Path<(i64, i64)>,
    pool: web::Data<AppState>,
    bearer: Option<ReqData<String>>,
) -> impl Responder {
    let (start, end) = path.into_inner();

    // Quick permission check
    if let Ok(resp) = check_action_access(pool.db.clone(), bearer, "report", Action::View).await {
        return resp;
    }

    let start_range = ts_at_zero(start, 0);
    let end_range = ts_at_zero(end, 0);

    // Get schedules first
    let schedule_list = get_schedules(&pool.db).await.unwrap_or_default();

    let tolerances = load_tolerances(&pool.db).await;

    let holidays = load_holidays(&pool.db, start_range, end_range).await;
    let leaves = load_leaves(&pool.db, start_range, end_range).await;

    // Then get reports data
    match get_reports_data(&pool.db, start_range, end_range).await {
        Ok(datas) => {
            let reports = get_enroll_data(start_range, end_range, datas, schedule_list, tolerances, &holidays, &leaves);
            GenericResponse::<Reports>::ok(reports, "OK".to_string()).into_response()
        }
        Err(e) => {
            log::error!("Database error: {}", e);
            NoDataResponse::new("Internal server error".to_string(), 500).into_response()
        }
    }
}

// Separate async functions for better concurrency
pub(crate) async fn get_schedules(db: &sqlx::PgPool) -> Result<Vec<ScheduleHelper>, sqlx::Error> {
    sqlx::query_as::<_, ScheduleHelper>(
        r#"SELECT schedules.schedule_name, COALESCE(shifts.shift_name,'Libur') as shift_name, COALESCE(shifts.shift_id,0) as shift_id, schedules.schedule_dom, schedules.schedule_parent, schedules.schedule_type, schedules.schedule_hols, COALESCE(shifts.shift_start_time,0::BIGINT) as shift_start_time, COALESCE(shifts.shift_end_time,0::BIGINT) as shift_end_time, COALESCE(shifts.shift_start_enroll,0::BIGINT) as shift_start_enroll, COALESCE(shifts.shift_end_enroll,0::BIGINT) as shift_end_enroll, COALESCE(shifts.shift_passday,0::SMALLINT) as shift_passday, COALESCE(shifts.shift_prevday,0::SMALLINT) as shift_prevday FROM schedules
           LEFT JOIN shifts ON shifts.shift_id = schedules.schedule_shift_id"#
    )
    .fetch_all(db)
    .await
}

pub(crate) async fn get_reports_data(
    db: &sqlx::PgPool,
    start_range: i64,
    end_range: i64,
) -> Result<Vec<ReportsHelper>, sqlx::Error> {
    sqlx::query_as::<_, ReportsHelper>(
        r#"SELECT device_timezone, device_name, device_location, enroll_device_sn,
                  enroll_type, enroll_status, enroll_time, employee_id, employee_fname,
                  employee_lname, employee_address, employee_department, employee_status,
                  employee_schedule_id, schedule_type
           FROM enrolls e
           INNER JOIN employee emp ON emp.employee_device_uid = e.enroll_employee_id
           INNER JOIN devices d ON e.enroll_device_sn = d.device_sn
           INNER JOIN schedules s ON s.schedule_id = emp.employee_schedule_id
           WHERE e.enroll_time BETWEEN $1 AND $2
           UNION ALL
           -- Approved field photos are punches too: the day row already treats
           -- them as in/out, so the report must show the same evidence instead
           -- of an empty day while payroll credits it. Device name stays "Field"
           -- so the operator can tell where the punch came from.
           SELECT COALESCE(p.tz_offset_minutes, 420) / 60 AS device_timezone,
                  'Field'::text AS device_name,
                  CASE WHEN p.lat IS NOT NULL AND p.lng IS NOT NULL
                       THEN p.lat::text || ',' || p.lng::text END AS device_location,
                  NULL::varchar AS enroll_device_sn,
                  0 AS enroll_type,
                  0 AS enroll_status,
                  EXTRACT(EPOCH FROM p.server_at)::bigint AS enroll_time,
                  emp.employee_id, emp.employee_fname, emp.employee_lname,
                  emp.employee_address, emp.employee_department, emp.employee_status,
                  emp.employee_schedule_id, s.schedule_type
           FROM field_attendance a
           INNER JOIN field_photo p ON p.attendance_id = a.id
           INNER JOIN employee emp ON emp.employee_id = a.employee_id
           INNER JOIN schedules s ON s.schedule_id = emp.employee_schedule_id
           WHERE a.status = 'approved'
             AND EXTRACT(EPOCH FROM p.server_at) BETWEEN $1 AND $2
           ORDER BY enroll_time ASC"#,
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

pub(crate) fn get_enroll_data(
    start: i64,
    end: i64,
    collection: Vec<ReportsHelper>,
    schedule_list: Vec<ScheduleHelper>,
    tolerances: Tolerances,
    holidays: &HolidaySet,
    leaves: &LeaveSet,
) -> Vec<Reports> {
    if collection.is_empty() {
        return Vec::new();
    }

    // Pre-build optimized lookup structures
    let schedule_map: HashMap<(i32, i32), ScheduleHelper> = schedule_list
        .into_iter()
        .map(|d| ((d.schedule_parent, d.schedule_dom), d))
        .collect();

    // Group enrollments by employee_id and date for faster processing
    let mut employee_enrollments: HashMap<i32, Vec<&ReportsHelper>> = HashMap::new();
    for enrollment in &collection {
        employee_enrollments
            .entry(enrollment.employee_id)
            .or_default()
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
            &schedule_map,
            tolerances,
            holidays,
            leaves,
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
    schedule_map: &HashMap<(i32, i32), ScheduleHelper>,
    tolerances: Tolerances,
    holidays: &HolidaySet,
    leaves: &LeaveSet,
) -> Vec<Reports> {
    let mut daily_reports: HashMap<i64, Reports> = HashMap::new();

    // Process each day in the range. `current_date` is a UTC midnight and doubles as the
    // bucket key in `enroll_date`.
    for current_date in (start..=end).step_by(86400) {
        // A non-working day produces no report row at all UNLESS the operator marked
        // that date as included: an included holiday is real work at a different pay
        // rate, so it is counted against the roster shift for that weekday below.
        // The gate sits BEFORE the schedule lookup in both cases, so inclusion never
        // resurrects a roster row that says nobody works.
        if holidays.contains(current_date) && holidays.included_name(current_date).is_none() {
            continue;
        }

        // An approved leave excuses exactly ONE person on exactly one day. The id
        // comes from the enrollment rows this group was built from, so the gate
        // cannot drift onto a different employee.
        if let Some(enrollment) = enrollments.first()
            && leaves.contains(enrollment.employee_id, current_date)
        {
            continue;
        }

        if let Some(enrollment) = enrollments.first()
            && let Some(shift) = get_shift_for_date(current_date, enrollment, schedule_map)
        {
            let (start_window, end_window) =
                calculate_enrollment_window(current_date, enrollment, shift);

            // Find all enrollments for this day
            let day_enrollments: Vec<&ReportsHelper> = enrollments
                .iter()
                .filter(|e| e.enroll_time >= start_window && e.enroll_time <= end_window)
                .copied()
                .collect();

            if !day_enrollments.is_empty()
                && let Some(mut report) =
                    create_daily_report(current_date, enrollment, shift, &day_enrollments)
            {
                // Payroll needs to tell this row apart from a normal day: it was
                // worked on a holiday, at a different rate. The label is exactly
                // the name the operator gave the date, or nothing on normal days.
                if let Some(name) = holidays.included_name(current_date) {
                    report.holiday_name = Some(name.to_string());
                }
                daily_reports.insert(current_date, report);
            }
        }
    }

    // Convert to vector and calculate working times
    daily_reports
        .into_values()
        .map(|mut report| {
            calculate_working_time(&mut report, tolerances);
            report
        })
        .collect()
}

/// Resolve the schedule row governing `current_date`.
///
/// `current_date` is a UTC midnight and its only job here is to NAME a calendar day: the
/// frontend renders it as a plain date, so the weekday used to look up the schedule must be
/// the weekday of that named day - for every device, regardless of its timezone.
///
/// The original computed the weekday as `ts_dow(current_date, device_timezone)`, which first
/// shifts the instant by the device offset. For a negative offset that lands on the PREVIOUS
/// day, and the offset then re-enters the lookup through `(dow % dom_span) + 1`, which wraps
/// Sunday (7) onto Monday (1) whenever the shifted weekday reaches 7. The net effect is that
/// the SAME weekday maps to a DIFFERENT schedule row depending on the device timezone: at
/// +07:00 a Saturday resolved to dom 7 (the "Shift Siang" row) while at -05:00 it correctly
/// resolved to dom 6. When a day's shifted key landed on a "Libur" row the day was dropped
/// entirely - that is the missing-report class this fixes.
///
/// The fix is to make the key a function of the day name alone: read the weekday from the
/// anchor with no timezone shift, and map it onto `dom` preserving its value (1..7) instead of
/// wrapping, so `(dow - 1) % dom_span + 1`.
fn get_shift_for_date<'a>(
    current_date: i64,
    enrollment: &ReportsHelper,
    schedule_map: &'a HashMap<(i32, i32), ScheduleHelper>,
) -> Option<&'a ScheduleHelper> {
    let schedule_type = enrollment.schedule_type;
    let pattern = if schedule_type < 7 {
        1
    } else {
        schedule_type / 7
    };
    let dom_span = if schedule_type < 7 { 1 } else { 7 };

    // `current_date` already IS midnight of the named calendar day, so read the weekday and
    // week from it directly, with no timezone shift. `ts_dow`/`ts_week` return 1..=7 and 1..=52,
    // so shift the weekday down by one before folding it into 1..=dom_span - that keeps the
    // mapping an identity for a 7-day schedule instead of wrapping Sunday onto Monday.
    let iso_dow = ts_dow(current_date, 0) as i32;
    let dow = (iso_dow - 1) % dom_span + 1;
    let week = (ts_week(current_date, 0) as i32) % if pattern < 1 { 1 } else { pattern };

    let key = (enrollment.employee_schedule_id, (dow + (week * 7)));
    if let Some(s) = schedule_map.get(&key)
        && (s.shift_id == 0 || s.shift_name == "Libur")
    {
        return None;
    }
    schedule_map.get(&key)
}

/// Enrollment window for `current_date` (a UTC midnight naming a calendar day), as epochs.
///
/// A shift's enroll offsets are wall-clock durations measured from the device's LOCAL
/// midnight, so the anchor is converted into that local midnight once and both bounds are
/// expressed from it. The original code subtracted the offset to build the base but then
/// re-added it on the end bound, so the window was one day short and displaced by the
/// offset: a shift whose checkout falls after local midnight (every night shift) had its
/// last punch excluded, and the report came back without it.
fn calculate_enrollment_window(
    current_date: i64,
    enrollment: &ReportsHelper,
    shift: &ScheduleHelper,
) -> (i64, i64) {
    let timezone_offset = (enrollment.device_timezone as i64) * 3600;

    // Local midnight of the named calendar day, as a UTC epoch.
    let local_midnight = current_date - timezone_offset;

    // A prevday shift opens its enroll window on the PREVIOUS calendar day.
    let start_window = local_midnight
        + if shift.shift_prevday == 1 {
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
    day_enrollments: &[&ReportsHelper],
) -> Option<Reports> {
    // Find first and last enrollments for the day
    let first_enrollment = day_enrollments.first()?.enroll_time;
    let last_enrollment = if day_enrollments.len() > 1 {
        Some(day_enrollments.last()?.enroll_time)
    } else {
        None
    };

    // Device identity belongs to the punch that opened the day, not to whichever
    // punch happened to come first in the whole range: a day whose only evidence
    // is a field checkout must read "Field", not the office reader from yesterday.
    let device_source = day_enrollments.first().copied().unwrap_or(enrollment);

    let schedule_type = enrollment.schedule_type;
    let pattern = if schedule_type < 7 {
        1
    } else {
        schedule_type / 7
    };
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
        employee_department: enrollment.employee_department.clone(),
        enroll_dow: Some((dow + (week * 7)).into()),
        enroll_date: Some(current_date),
        device_timezone: Some(device_source.device_timezone),
        device_name: device_source.device_name.clone(),
        device_location: device_source.device_location.clone(),
        shift_start_time: Some(shift.shift_start_time),
        shift_end_time: Some(shift.shift_end_time),
        shift_name: Some(shift.shift_name.clone()),
        schedule_name: Some(shift.schedule_name.clone()),
        start_enroll: Some(first_enrollment),
        end_enroll: last_enrollment,
        working_time: 0,
        late_time: 0,
        overtime_time: 0,
        holiday_name: None,
    })
}

fn calculate_working_time(report: &mut Reports, tolerances: Tolerances) {
    let Some(enroll_date) = report.enroll_date else {
        return;
    };
    let Some(device_timezone) = report.device_timezone else {
        return;
    };
    let Some(shift_start) = report.shift_start_time else {
        return;
    };
    let Some(shift_end) = report.shift_end_time else {
        return;
    };

    let timezone_offset = (device_timezone * 3600) as i64;

    // `enroll_date` is a UTC midnight; the shift's own start/end are wall-clock durations
    // from LOCAL midnight. Convert the anchor to local midnight once, then add the shift
    // offsets in that same frame. The original code added the offset to the start bound and
    // subtracted it from the end bound, which stretched or compressed the shift span by
    // twice the offset and pushed the reference point a day off for non-zero timezones.
    let local_midnight = enroll_date - timezone_offset;
    let start_working = local_midnight
        + if shift_start >= shift_end {
            shift_start - 86400
        } else {
            shift_start
        };
    let end_working = local_midnight + shift_end;

    let shift_duration = end_working - start_working;
    let half_shift = shift_duration / 2;

    // Calculate late time. `late_tolerance_sec` is the configured grace window:
    // an arrival within that many seconds of shift start is not counted as late.
    if let Some(start_enroll) = report.start_enroll {
        if start_enroll > start_working {
            let late = start_enroll - start_working;
            report.late_time = if late <= tolerances.late_sec {
                0
            } else {
                late.min(shift_duration)
            };
        }

        // Handle case where employee arrived after half shift without checkout
        if start_enroll > start_working + half_shift && report.end_enroll.is_none() {
            // A lone punch that late can only be a checkout. It is not a late
            // arrival, so the late counter must be cleared with the reclassify.
            report.end_enroll = report.start_enroll;
            report.start_enroll = None;
            report.late_time = 0;
        }
    }

    // Calculate working time: actual presence between first IN and last OUT.
    // The old formula measured from shift start (`end_enroll - start_working`),
    // which inflated the number whenever someone arrived late and hid early
    // leave entirely (OUT before shift end still showed a full shift).
    if let (Some(start_enroll), Some(end_enroll)) = (report.start_enroll, report.end_enroll) {
        report.working_time = end_enroll.saturating_sub(start_enroll);
    }
    // Checkout-only days keep working_time 0: inventing a span from the shift
    // start reported 9 hours for a single 17:00 punch, which the payslip never
    // paid. The row still carries the punch and the overtime below.

    // Overtime: seconds worked past the configured shift end. Gated by
    // `overtime_tolerance_sec` - leaving within that many seconds after shift end
    // is treated as an on-time checkout, not overtime.
    if let Some(end_enroll) = report.end_enroll
        && end_enroll > end_working
    {
        let over = end_enroll - end_working;
        report.overtime_time = if over <= tolerances.overtime_sec {
            0
        } else {
            over
        };
    }
}
