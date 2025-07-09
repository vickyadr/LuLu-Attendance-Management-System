use crate::handler::{
    adms::{h_cdata::*, h_getrequest::*, h_devicecmd::*},
    h_login::*
    };
use actix_web::web;

pub fn config(conf: &mut web::ServiceConfig) {
    // Frontend API
    let scope = web::scope("/api").service(login_check)
                                  .service(login_act);

    // ADMS API
    let iclock = web::scope("/iclock").service(get_cdata)
                                      .service(post_cdata)
                                      .service(get_request)
                                      .service(post_device_cmd);

    conf.service(scope).service(iclock);
}
