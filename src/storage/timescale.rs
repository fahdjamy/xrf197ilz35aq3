use crate::{DatabaseConfig, PostgresConfig};
use anyhow::Context;
use sqlx::postgres::PgPoolOptions;
use std::sync::OnceLock;

static INIT_TIMESCALE_DB: OnceLock<()> = OnceLock::new();

pub async fn setup_timescale_db(database_config: DatabaseConfig) {
    INIT_TIMESCALE_DB.call_once(async || {
        let timescale_config = database_config.timescale;
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
            .connect_lazy_with(pg_config.connect_to_database(&pg_config.name));

        // --- TimescaleDB Specific Setup ---

        // 1. Ensure TimescaleDB extension is enabled (often automatic in Docker image)
        sqlx::query("CREATE EXTENSION IF NOT EXISTS timescaledb;")
            .execute(&pg_pool)
            .await
            .context("Failed to create timescaledb table".to_string())?;
    });
}
