use actix_web::body::MessageBody;
use actix_web::dev::ServiceResponse;
use actix_web::http::{StatusCode, header};
use actix_web::middleware::{ErrorHandlerResponse, ErrorHandlers};
use actix_web::{App, HttpResponse, HttpServer, middleware, web};
use actix_web_httpauth::extractors::bearer;

use lulu_attendance_server::utility::router;
use lulu_attendance_server::utility::{
    cors, db, device_sweeper, hash, shutdown, stor::AppState, stor::NoDataResponse, wshub,
};
use lulu_attendance_server::service::pull_scheduler::{self, PullRegistry};
use lulu_attendance_server::service::live_scheduler::{self, LiveRegistry};

/// Message shown for a status raised outside any handler.
fn message_for(status: StatusCode) -> &'static str {
    match status {
        StatusCode::UNAUTHORIZED => "Session token invalid",
        StatusCode::FORBIDDEN => "Sorry, doesnt have permission",
        StatusCode::NOT_FOUND => "Route or resource not found",
        StatusCode::PAYLOAD_TOO_LARGE => "Payload too large",
        _ => "Internal error, please try again!",
    }
}

fn json_error<B: MessageBody + 'static>(
    status: StatusCode,
) -> impl Fn(ServiceResponse<B>) -> Result<ErrorHandlerResponse<B>, actix_web::Error> {
    move |res: ServiceResponse<B>| {

        let is_json = res
            .response()
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .is_some_and(|v| v.starts_with("application/json"));
        if is_json {
            let (req, response) = res.into_parts();
            return Ok(ErrorHandlerResponse::Response(ServiceResponse::new(
                req,
                response.map_into_left_body(),
            )));
        }

        let (req, response) = res.into_parts();
        let response = response
            .set_body(
                HttpResponse::build(status)
                    .json(NoDataResponse::new(
                        message_for(status).to_string(),
                        status.as_u16() as i32,
                    ))
                    .into_body(),
            )
            .map_into_boxed_body()
            .map_into_right_body();
        Ok(ErrorHandlerResponse::Response(ServiceResponse::new(
            req, response,
        )))
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    if let Err(err) = hash::ensure_secret_configured() {
        log::error!("{err}");
        std::process::exit(1);
    }

    cors::log_scope();

    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(4343);

    let pool = match db::initialize_db_pool().await {
        Ok(pool) => {
            log::info!("database connected, migrations applied");
            pool
        }
        Err(err) => {
            log::error!("failed to connect to database: {err:?}");
            std::process::exit(1);
        }
    };

    log::info!("server starting on http://{host}:{port}");

    shutdown::install();

    let ws_hub = wshub::WsHub::new();

    // Pull task registry: track active ZKNet device pull loops
    let pull_registry: PullRegistry = std::sync::Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new()));

    let live_registry: LiveRegistry = std::sync::Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new()));

    actix_web::rt::spawn(device_sweeper::run_sweeper(pool.clone(), ws_hub.clone()));

    let app_state = web::Data::new(AppState {
        db: pool.clone(),
        ws: ws_hub.clone(),
    });

    // Initialize ZKNet pull scheduler: spawn background tasks for all ZKNET devices
    pull_scheduler::init_pull_scheduler(app_state.clone(), web::Data::new(pull_registry.clone())).await;

    live_scheduler::init_live_scheduler(app_state.clone(), web::Data::new(live_registry.clone())).await;

    let server = HttpServer::new(move || {
        let cors = cors::policy();

        App::new()
            // add DB pool handle to app data; enables use of `web::Data<DbPool>` extractor
            .app_data(web::Data::new(AppState {
                db: pool.clone(),
                ws: ws_hub.clone(),
            }))
            .app_data(web::Data::new(pull_registry.clone()))
            // The pull endpoint has to hand the device socket over: a manual pull
            // stops the live session first and restarts it afterwards, so it needs
            // the same registry the scheduler owns.
            .app_data(web::Data::new(live_registry.clone()))
            .app_data(web::JsonConfig::default().limit(32 * 1024))
            .app_data(
                bearer::Config::default()
                    .realm("restricted area")
                    .scope("login user"),
            )

            .wrap(
                ErrorHandlers::new()
                    .handler(
                        StatusCode::UNAUTHORIZED,
                        json_error(StatusCode::UNAUTHORIZED),
                    )
                    .handler(StatusCode::FORBIDDEN, json_error(StatusCode::FORBIDDEN))
                    .handler(StatusCode::NOT_FOUND, json_error(StatusCode::NOT_FOUND))
                    .handler(
                        StatusCode::PAYLOAD_TOO_LARGE,
                        json_error(StatusCode::PAYLOAD_TOO_LARGE),
                    )
                    .handler(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        json_error(StatusCode::INTERNAL_SERVER_ERROR),
                    ),
            )
            // add request logger middleware
            .wrap(middleware::Logger::default())
            // add authentication middleware
            //.wrap(auth)
            // add route handlers
            .wrap(cors)
            .configure(router::config)
    })
    //.bind(("10.55.54.145", 8080))?
    .bind((host.as_str(), port))?
    .disable_signals()
    .run();

    let server_handle = server.handle();

    actix_web::rt::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
            if shutdown::triggered() {
                break;
            }
        }

        log::info!("shutdown signal received, closing server");
        server_handle.stop(false).await;

        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        log::info!("shutdown complete");
        log::logger().flush();
        std::process::exit(0);
    });

    server.await
}
