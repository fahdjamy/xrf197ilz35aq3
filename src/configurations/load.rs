use crate::configurations::DatabaseConfig;
use crate::{Environment, XRF_Q3_ENV};
use config::{self, ConfigError};
use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct ApplicationConfig {
    pub name: String,
}

#[derive(Deserialize, Clone)]
pub struct GrpcServerConfig {
    pub port: String,
    pub timeout: u16,
}

#[derive(Deserialize, Clone)]
pub struct ServerConfig {
    pub grpc: GrpcServerConfig,
}

#[derive(Deserialize, Clone)]
pub struct LogConfig {
    pub level: String,
    pub output: String,
    pub suffix: String,
    pub prefix: String,
}

#[derive(serde::Deserialize, Clone)]
pub struct Configurations {
    pub log: LogConfig,
    pub server: ServerConfig,
    pub app: ApplicationConfig,
    pub database: DatabaseConfig,
}

pub fn load_config() -> Result<Configurations, ConfigError> {
    let base_path = std::env::current_dir().expect("Could not determine current directory");
    let config_path = base_path.join("config");

    // load app environment. default to dev (local/dev) if no env is specified
    let env: Environment = std::env::var(XRF_Q3_ENV)
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("XRF_ENV env variable is not accepted environment");

    // load configurations filename for set XRF_ENV environment
    let env_config_file = format!("{}.yaml", env.as_str());

    // Initialise the configurations
    let config = config::Config::builder()
        // Add base configuration values from a file named `app.yaml`.
        .add_source(config::File::from(config_path.join("application.yaml")))
        // Add configuration values from the environment specific file
        .add_source(config::File::from(config_path.join(env_config_file)))
        // Add configurations set from the exported environment
        .add_source(
            config::Environment::with_prefix("XRF_Q3")
                .prefix_separator("_")
                .separator("-"),
        )
        .build()?;

    // Try converting the configuration values into our Config type
    let configurations = config.try_deserialize::<Configurations>()?;

    // make sure timescale and postgres Databases are different
    if configurations.database.postgres.name == configurations.database.timescale.name {
        return Err(ConfigError::Message(
            "invalid databases. postgres and timescale have connect to the same db".to_string(),
        ));
    }
    if configurations.database.postgres.name.is_empty()
        || configurations.database.timescale.name.is_empty()
    {
        return Err(ConfigError::Message(
            "invalid databases. postgres or timescale db name is empty".to_string(),
        ));
    }
    Ok(configurations)
}
