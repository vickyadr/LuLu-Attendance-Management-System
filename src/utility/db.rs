use sqlx::{postgres::PgPoolOptions, Error, Pool, Postgres};

pub async fn initialize_db_pool() -> Result<Pool<Postgres>, Error> {
    dotenvy::dotenv().ok();
    let conn_spec = std::env::var("DATABASE_URL").expect("DATABASE_URL should be set");

    PgPoolOptions::new()
        .max_connections(10)
        .connect(&conn_spec)
        .await
}
