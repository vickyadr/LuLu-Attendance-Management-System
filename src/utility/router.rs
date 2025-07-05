use crate::handler::adms::{h_cdata::*, h_getrequest::*};
use crate::handler::h_list::*;
use actix_web::web;

pub fn config(conf: &mut web::ServiceConfig) {
    // Frontend API
    let scope = web::scope("/api").service(get_list)
                                                .service(post_list);

    // ADMS API
    let iclock = web::scope("/iclock").service(get_cdata)
                                                    .service(post_cdata)
                                                    .service(get_request);

    conf.service(scope).service(iclock);
}
