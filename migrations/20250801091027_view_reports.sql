DROP TABLE IF EXISTS "view_report";
CREATE VIEW "view_report" AS  WITH query_sch AS (
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
            shifts.shift_prevday,
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
            query_sch.shift_prevday,
            query_sch.shift_name,
            (date_part('dow'::text, (to_timestamp(((((query_sch.enroll_time + (query_sch.shift_prevday * 86400)) - (query_sch.shift_passday * 86400)) + (query_sch.device_timezone * 3600)))::double precision))::date) + (1)::double precision) AS enroll_dow,
            date_part('epoch'::text, (to_timestamp(((((query_sch.enroll_time + (query_sch.shift_prevday * 86400)) - (query_sch.shift_passday * 86400)) + (query_sch.device_timezone * 3600)))::double precision))::date) AS enroll_date
           FROM query_sch
          WHERE (((query_sch.schedule_type = 1) AND (query_sch.enroll_time >= ((date_part('epoch'::text, (to_timestamp((query_sch.enroll_time)::double precision))::date))::integer + (query_sch.shift_start_enroll - (query_sch.device_timezone * 3600)))) AND (query_sch.enroll_time <= ((date_part('epoch'::text, (to_timestamp((query_sch.enroll_time)::double precision))::date))::integer + (query_sch.shift_end_enroll - (query_sch.device_timezone * 3600))))) OR ((query_sch.schedule_type = 7) AND ((date_part('dow'::text, (to_timestamp(((query_sch.enroll_time + (query_sch.device_timezone * 3600)))::double precision))::date) + (1)::double precision) = (query_sch.schedule_dom)::double precision) AND (query_sch.enroll_time >= ((date_part('epoch'::text, (to_timestamp((query_sch.enroll_time)::double precision))::date))::integer + (query_sch.shift_start_enroll - (query_sch.device_timezone * 3600)))) AND (query_sch.enroll_time <= ((date_part('epoch'::text, (to_timestamp((query_sch.enroll_time)::double precision))::date))::integer + (query_sch.shift_end_enroll - (query_sch.device_timezone * 3600))))))
        ), query_group_day AS (
         SELECT query_filter_range_1.employee_id,
            query_filter_range_1.employee_fname,
            query_filter_range_1.employee_lname,
            query_filter_range_1.employee_departement,
            query_filter_range_1.enroll_dow,
            query_filter_range_1.enroll_date,
            query_filter_range_1.device_timezone,
            query_filter_range_1.device_name,
            query_filter_range_1.device_location,
            query_filter_range_1.shift_start_time,
            query_filter_range_1.shift_end_time,
            query_filter_range_1.shift_name,
            query_filter_range_1.schedule_name,
            min(query_filter_range_1.enroll_time) AS start_enroll,
            max(query_filter_range_1.enroll_time) AS end_enroll
           FROM query_filter_range query_filter_range_1
          GROUP BY query_filter_range_1.employee_id, query_filter_range_1.employee_fname, query_filter_range_1.employee_lname, query_filter_range_1.employee_departement, query_filter_range_1.enroll_dow, query_filter_range_1.enroll_date, query_filter_range_1.device_timezone, query_filter_range_1.device_name, query_filter_range_1.device_location, query_filter_range_1.shift_start_time, query_filter_range_1.shift_end_time, query_filter_range_1.shift_name, query_filter_range_1.schedule_name
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
;
