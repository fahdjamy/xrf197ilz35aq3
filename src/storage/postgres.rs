use crate::PostgresConfig;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::sync::OnceLock;

static INIT_PG_DB: OnceLock<PgPool> = OnceLock::new();

pub fn setup_postgres(postgres_config: PostgresConfig) -> &'static PgPool {
    INIT_PG_DB.get_or_init(|| {
        PgPoolOptions::new()
            .connect_lazy_with(postgres_config.connect_to_database(&postgres_config.name))
    })
}
