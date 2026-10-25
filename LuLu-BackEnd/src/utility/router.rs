use crate::handler::{
    adms::{h_cdata::*, h_devicecmd::*, h_getrequest::*},
    h_device::*,
    h_employee::*,
    h_employee_deductions::*,
    h_field::*,
    h_holiday::*,
    h_leave::*,
    h_login::*,
    h_payroll::*,
    h_report::*,
    h_schedule::*,
    h_settings::*,
    h_shift::*,
    h_transaction::*,
    h_user::*,
    h_zkpull::*,
};
use actix_web::{error::Error, web};

use crate::utility::hash::encode_256;
use actix_web::HttpMessage;
use actix_web::dev::ServiceRequest;
use actix_web_httpauth::extractors::bearer::BearerAuth;
use actix_web_httpauth::middleware::HttpAuthentication;

use crate::utility::wshub::ws_endpoint;

pub async fn validate(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let token = encode_256(credentials.token()).await;
    req.extensions_mut().insert(token);
    Ok(req)
}

pub fn config(conf: &mut web::ServiceConfig) {
    let auth = HttpAuthentication::bearer(validate);

    // Single /api scope: public login + protected sub-scope
    let protected = web::scope("")
        .wrap(auth)
        .service(login_check)
        .service(logout_act)
        .service(transaction_live)
        .service(device_add)
        .service(device_delete)
        .service(device_disable)
        .service(device_edit)
        .service(device_list)
        .service(device_pull)
        .service(device_set_time)
        .service(schedule_add)
        .service(schedule_delete)
        .service(schedule_edit)
        .service(schedule_list)
        .service(shift_add)
        .service(shift_delete)
        .service(shift_edit)
        .service(shift_list)
        .service(report_today)
        .service(report_range)
        .service(settings_list)
        .service(settings_edit)
        .service(holiday_list)
        .service(holiday_edit)
        .service(holiday_delete)
        .service(holiday_import)
        .service(leave_list)
        .service(leave_employee_options)
        .service(leave_edit)
        .service(leave_decision)
        .service(leave_cancel)
        .service(employee_add)
        .service(employee_delete)
        .service(employee_edit)
        .service(employee_list)
        .service(employee_unregistered)
        .service(employee_import)
        .service(user_list)
        .service(user_add)
        .service(user_edit)
        .service(user_delete)
        .service(user_permissions_update)
        .service(user_permissions_list)
        .service(get_config)
        .service(update_config)
        .service(create_salary_template)
        .service(list_salary_templates)
        .service(get_salary_template)
        .service(update_salary_template)
        .service(assign_employee_salary)
        .service(update_salary_rules)
        .service(delete_employee_salary)
        .service(list_employee_salaries)
        .service(import_employee_salaries)
        .service(copy_salary_rules)
        .service(get_employee_salary)
        .service(create_public_holiday)
        .service(list_public_holidays)
        .service(delete_public_holiday)
        .service(create_deduction_template)
        .service(list_deduction_templates)
        .service(get_deduction_template)
        .service(update_deduction_template)
        .service(delete_deduction_template)
        .service(assign_employee_deduction)
        .service(list_employee_deductions)
        .service(update_employee_deduction)
        .service(delete_employee_deduction)
        .service(upsert_employee_statutory)
        .service(get_employee_statutory)
        .service(crate::handler::payroll_api::generate_payslips)
        .service(crate::handler::payroll_api::list_payslips)
        .service(crate::handler::payroll_api::get_payslip)
        .service(crate::handler::payroll_api::lock_payslip)
        .service(crate::handler::payroll_api::delete_payslip)
        .service(crate::handler::payroll_api::delete_draft_payslips_bulk)
        .service(crate::handler::payroll_api::approve_daily_absence)
        .service(crate::handler::payroll_api::reject_daily_absence)
        .service(crate::handler::payroll_api::list_daily_absences)
        // Field attendance: employee PWA + admin review, same page key `field`.
        // Raw photo bodies are posted here, so the payload cap sits on this scope.
        .service(
            web::scope("/field")
                .app_data(web::PayloadConfig::new(2 * 1024 * 1024 + 1024))
                .service(field_me)
                .service(field_open_day)
                .service(field_upload_photo)
                .service(field_list_days)
                .service(field_review_day)
                .service(field_photo_file)
                .service(field_create_account)
                .service(field_delete_photo)
                .service(field_reset_password)
                .service(field_list_accounts),
        );

    let api = web::scope("/api")
        .service(login_act)
        // Outside `protected` on purpose: the browser WebSocket API cannot send
        // an Authorization header, so this route validates ?token= itself.
        .service(ws_endpoint)
        .service(protected);

    // ADMS API (device callbacks) - no auth
    let iclock = web::scope("/iclock")
        .service(get_cdata)
        .service(post_cdata)
        .service(get_request)
        .service(post_device_cmd);

    conf.service(api).service(iclock).service(field_app_page);
}
