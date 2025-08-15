use crate::{ApplicationContext, Configurations, DatabaseConfig, GrpcServer};
use cassandra_cpp::Session;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

pub struct Server {
    pub grpc_server: GrpcServer,
}

impl Server {
    pub async fn build_and_load(
        config: Configurations,
        cassandra_session: Session,
        app_ctx: ApplicationContext,
    ) -> Result<Self, anyhow::Error> {
        let pool = get_connection_pool(&config.database);

        let grpc_server = GrpcServer::new(pool, config.server.grpc, cassandra_session, app_ctx)
            .map_err(|err| anyhow::anyhow!("{}", err))?;

        Ok(Server { grpc_server })
    }
}

fn get_connection_pool(configuration: &DatabaseConfig) -> PgPool {
    PgPoolOptions::new().connect_lazy_with(
        configuration
            .postgres
            .connect_to_database(&configuration.postgres.name),
    )
}
