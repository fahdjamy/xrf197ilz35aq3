use anyhow::anyhow;
use std::env;
use std::fmt::{Debug, Display};
use tokio::task::JoinError;
use tracing::{error, info};
use uuid::Uuid;
use xrfq3::storage::{connect_session, create_keyspace, PreparedAppStatements};
use xrfq3::{
    load_config, setup_tracing_logger, ApplicationContext, Environment, Server, APP_REGION,
    XRF_Q3_ENV,
};

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    // load app environment. default to dev (local/dev) if no env is specified
    let environment = load_environment().map_err(|err| {
        error!("Failed to load environment, err={}", err);
        anyhow!("Failed to load environment, err={}", err)
    })?;

    // load configurations
    let config = load_config(&environment).map_err(|err| {
        error!("Failed to load configs, err={}", err);
        anyhow!("Failed to load configuration: {}", err)
    })?;

    let _guard = setup_tracing_logger(&config.app.name, &config.log);

    // Connect to cassandra sessions
    let cassandra_session = match connect_session(&config.database.cassandra).await {
        Ok(cassandra_session) => cassandra_session,
        Err(err) => return Err(anyhow!("Failed to connect to Cassandra: {}", err)),
    };

    // create cassandra keyspace
    create_keyspace(
        &config.database.cassandra.keyspace,
        config.database.cassandra.replication_factor,
        &cassandra_session,
    )
    .await
    .map_err(|err| {
        error!("Failed to create keyspace: {}", err);
        anyhow!("Failed to create keyspace: {}", err)
    })?;

    // create Cassandra prepared statements
    let prepared_stmts = match PreparedAppStatements::new(&cassandra_session).await {
        Ok(stmts) => stmts,
        Err(err) => {
            return Err(anyhow!("failed to load prepared-statements, err={}", err));
        }
    };

    let region = match get_region_from_env(&environment) {
        None => {
            return Err(anyhow!("region not found"));
        }
        Some(region) => region,
    };

    let app_ctx = match ApplicationContext::load(
        Uuid::new_v4().to_string(),
        region,
        &config.database.redis,
        prepared_stmts,
    )
    .await
    {
        Ok(app_ctx) => app_ctx,
        Err(err) => {
            return Err(anyhow!("failed to load application context: {}", err));
        }
    };

    let server = Server::build_and_load(config, cassandra_session, app_ctx)
        .await
        .map_err(|err| {
            error!("failed to build server, err={}", err);
            anyhow!("failed to build server, err={}", err)
        })?;

    // start the servers
    // these tasks is spawn in a thread
    // let grpc_server_task = tokio::spawn(server.grpc_server.run_until_stopped(&environment.clone()));
    let grpc_server_task = tokio::spawn(async move {
        server
            .grpc_server
            .run_until_stopped(&environment)
            .await
            .map_err(|err| {
                error!("server stopped due to error={}", err);
                anyhow!("server stopped due to error={}", err)
            })
    });

    tokio::select! {
        outcome = grpc_server_task => report_exit("gRPC-worker", outcome),
    }

    info!("!!! xrf197ilz35aq3 started successfully !!!");
    Ok(())
}

fn report_exit(task_name: &str, outcome: Result<Result<(), impl Debug + Display>, JoinError>) {
    match outcome {
        Ok(Ok(())) => {
            info!("{} has exited", task_name)
        }
        Ok(Err(e)) => {
            error!(
                error.cause_chain = ?e,
                error.message = %e,
                "{} failed",
                task_name
            )
        }
        Err(e) => {
            error!(
                error.cause_chain = ?e,
                error.message = %e,
                "{}' task failed to complete",
                task_name
            )
        }
    }
}

fn get_region_from_env(env: &Environment) -> Option<String> {
    match env::var(APP_REGION) {
        Ok(region) => Some(region),
        Err(_) => {
            if env.is_local() {
                return Some("USEastOhio".to_string());
            };
            None
        }
    }
}

fn load_environment() -> Result<Environment, String> {
    env::var(XRF_Q3_ENV)
        .unwrap_or_else(|_| "local".into())
        .try_into()
}
