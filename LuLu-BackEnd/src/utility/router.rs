use crate::handler::{
    adms::{h_cdata::*, h_devicecmd::*, h_getrequest::*},
    h_device::*,
    h_employee::*,
    h_holiday::*,
    h_leave::*,
    h_login::*,
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
        .service(user_permissions_list);

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

    conf.service(api).service(iclock);
}
