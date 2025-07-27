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
            .allowed_origin("http://127.0.0.1:8080")
            .allowed_origin("http://127.0.0.1:3000")
            .allowed_origin("http://localhost:3000")
            //.allow_any_origin()
            //.allow_any_header()
            //.allow_any_method()
            //.send_wildcard();
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
            .supports_credentials();

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
    .bind(("10.55.54.145", 8080))?
    //.bind(("0.0.0.0", 8080))?
    .run()
    .await
}
