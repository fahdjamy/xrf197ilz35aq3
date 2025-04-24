use crate::{DatabaseConfig, PostgresConfig};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::time::Duration;
use tokio::sync::OnceCell;

static INIT_TIMESCALE_DB_POOL: OnceCell<PgPool> = OnceCell::const_new();

pub async fn setup_timescale_db(
    database_config: DatabaseConfig,
) -> Result<&'static PgPool, anyhow::Error> {
    INIT_TIMESCALE_DB_POOL
        .get_or_try_init(async || Ok(get_timescale_db_pool(database_config).await))
        .await
}

async fn get_timescale_db_pool(db_config: DatabaseConfig) -> PgPool {
    let timescale_config = db_config.timescale;
    let pg_config = PostgresConfig::new(
        timescale_config.port,
        timescale_config.host.clone(),
        timescale_config.name.clone(),
        timescale_config.username.clone(),
        timescale_config.require_ssl,
        timescale_config.password.clone(),
    );

    let pg_pool = PgPoolOptions::new()
        .max_connections(timescale_config.max_conn as u32)
        .acquire_timeout(Duration::from_secs(5))
        .connect_lazy_with(pg_config.connect_to_database(&pg_config.name));

    // --- TimescaleDB Specific Setup ---
    // 1. Ensure TimescaleDB extension is enabled (often automatic in Docker image)
    sqlx::query("CREATE EXTENSION IF NOT EXISTS timescaledb;")
        .execute(&pg_pool)
        .await
        .expect("Failed to create TimescaleDB extension");

    pg_pool
}
