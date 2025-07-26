use crate::handler::{
    adms::{h_cdata::*, h_getrequest::*, h_devicecmd::*},
    { h_login::*, h_transaction::*, h_device::* }
};
use actix_web::{
    web,
    error::Error,
};

use actix_web::dev::ServiceRequest;
use actix_web_httpauth::extractors::bearer::BearerAuth;
use actix_web_httpauth::middleware::HttpAuthentication; 
use actix_web::HttpMessage;
use crate::utility::hash::encode_256;

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

    // Frontend API
    let frontend = web::scope("/api").service(login_check)
                                        .service(logout_act)
                                        .service(transaction_live)
                                        .wrap(auth.clone())
                                        .service(login_act)
                                        .service(device_list);

    // ADMS API
    let iclock = web::scope("/iclock").service(get_cdata)
                                      .service(post_cdata)
                                      .service(get_request)
                                      .service(post_device_cmd);

    conf.service(frontend).service(iclock);
}
