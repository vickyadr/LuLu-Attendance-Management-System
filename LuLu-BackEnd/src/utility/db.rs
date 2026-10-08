//use actix_web::cookie::time::Duration;
use sqlx::{Error, Pool, Postgres, postgres::PgPoolOptions};

/// Build the connection pool and apply every pending migration.
///
/// Migrations run once and are tracked in the `_sqlx_migrations` table, so a
/// second start is a no-op. Every file in ./migrations is idempotent
/// (`CREATE TABLE IF NOT EXISTS`, `CREATE OR REPLACE VIEW`, `ON CONFLICT DO
/// NOTHING`), which keeps a re-run against an existing database safe.
pub async fn initialize_db_pool() -> Result<Pool<Postgres>, Error> {
    dotenvy::dotenv().ok();
    let conn_spec = std::env::var("DATABASE_URL").expect("DATABASE_URL should be set");

    let pool = PgPoolOptions::new()
        .max_connections(20)
        .min_connections(5)
        .acquire_timeout(std::time::Duration::from_secs(10))
        .idle_timeout(std::time::Duration::from_secs(60))
        .max_lifetime(std::time::Duration::from_secs(30 * 60))
        .connect(&conn_spec)
        .await?;

    // run every file in ./migrations (once; tracked in the _sqlx_migrations table)
    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}

pub fn as_epoch(col: &str) -> String {
    let mut col_as = col;

    if col_as.find('.').is_some() {
        let split: Vec<&str> = col_as.split(".").collect();
        col_as = split[split.len() - 1];
    }

    format!("((EXTRACT(EPOCH FROM {0})::INTEGER) AS `{1}`)", col, col_as)
}
