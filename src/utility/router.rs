use crate::handler::{
    adms::{h_cdata::*, h_getrequest::*, h_devicecmd::*},
    h_login::*
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
    let scope = web::scope("/api").service(login_act)
                                    .service(login_check).wrap(auth.clone());

    // ADMS API
    let iclock = web::scope("/iclock").service(get_cdata)
                                      .service(post_cdata)
                                      .service(get_request)
                                      .service(post_device_cmd);

    conf.service(scope).service(iclock);
}
