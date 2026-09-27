use actix_cors::Cors;
use actix_web::http::header;
use actix_web::{middleware, web, App, HttpServer};
use actix_web_httpauth::extractors::bearer;

use lulu_attendance_server::utility::router;
use lulu_attendance_server::utility::{db, stor::AppState};


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    //let addr = std::env::var("ALLOW_REMOTE_ADDR").expect("ALLOW_REMOTE_ADDR should be set").to_owned();
    let pool = match db::initialize_db_pool().await {
        Ok(pool) => {
            println!("✅ Connection to database success!");
            pool
        }
        Err(err) => {
            println!("🔥 Failed to connect to database: {:?}", err);
            std::process::exit(1);
        }
    };

    println!("🚀 Server started successfully");

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin_fn(|origin, _| {
                // allow any origin (localhost, LAN IP, 0.0.0.0) — echo back request origin
                // permissive but still supports credentials (Bearer token)
                origin.as_bytes().starts_with(b"http://")
                    || origin.as_bytes().starts_with(b"https://")
            })
            .allowed_methods(vec!["GET", "POST", "PATCH", "DELETE", "OPTIONS"])
            .allowed_headers(vec![
                header::CONTENT_TYPE,
                header::AUTHORIZATION,
                header::ACCEPT,
                header::ACCEPT_ENCODING,
                header::ACCEPT_LANGUAGE,
                header::HOST,
                header::CONNECTION,
                header::ORIGIN,
                header::REFERER,
                header::USER_AGENT,
            ])
            .expose_headers(vec![header::CONTENT_TYPE, header::AUTHORIZATION])
            .supports_credentials()
            .max_age(3600);

        App::new()
            // add DB pool handle to app data; enables use of `web::Data<DbPool>` extractor
            .app_data(web::Data::new(AppState { db: pool.clone() }))
            .app_data(
                bearer::Config::default()
                .realm("restricted area")
                .scope("login user")
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
    .bind(("0.0.0.0", 4343))?
    .run()
    .await
}
