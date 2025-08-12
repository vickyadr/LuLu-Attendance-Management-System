//use actix_web::cookie::time::Duration;
use sqlx::{postgres::PgPoolOptions, Error, Pool, Postgres};

pub async fn initialize_db_pool() -> Result<Pool<Postgres>, Error> {
    dotenvy::dotenv().ok();
    let conn_spec = std::env::var("DATABASE_URL").expect("DATABASE_URL should be set");

    PgPoolOptions::new()
        .max_connections(20)
        .min_connections(5)
        //.acquire_timeout(Duration::from_secs(3))
        .connect(&conn_spec)
        .await
}

pub fn as_epoch(col: &str)-> String {
    let mut col_as = col;

    match col_as.find('.') {
        Some(_)=>{
            let split : Vec<&str> = col_as.split(".").collect();
            col_as = split[split.len()-1];
        },
        None =>()
    }

    format!("((EXTRACT(EPOCH FROM {0})::INTEGER) AS `{1}`)", col, col_as)
}