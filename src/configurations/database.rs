use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use serde_aux::field_attributes::deserialize_number_from_string;
use sqlx::postgres::{PgConnectOptions, PgSslMode};

#[derive(Clone, Debug, Deserialize)]
pub struct DatabaseConfig {
    pub redis: RedisConfig,
    pub postgres: PostgresConfig,
    pub timescale: TimescaleConfig,
    pub cassandra: CassandraConfig,
}

#[derive(Clone, Debug, Deserialize)]
pub struct RedisConfig {
    pub port: String,
    pub database: String,
    pub password: String,
    pub username: String,
    pub hostname: String,
    pub require_tls: bool,
}

impl RedisConfig {
    pub fn test_config(port: String, hostname: String) -> Self {
        RedisConfig {
            port,
            hostname,
            require_tls: false,
            username: "".to_string(),
            password: "".to_string(),
            database: "0".to_string(),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct TimescaleConfig {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub name: String,
    pub username: String,
    // determines of db connection needs to be secure or not
    pub require_ssl: bool,
    pub password: SecretString,

    pub max_conn: u16,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CassandraConfig {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub user: String,
    pub host: String,
    pub keyspace: String,
    pub password: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub connect_timeout: u16,
    pub replication_factor: u8,
}

#[derive(Deserialize, Debug, Clone)]
pub struct PostgresConfig {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub name: String,
    pub username: String,
    // determines of db connection needs to be secure or not
    pub require_ssl: bool,
    pub password: SecretString,
}

impl PostgresConfig {
    pub fn new(
        port: u16,
        host: String,
        name: String,
        username: String,
        require_ssl: bool,
        password: SecretString,
    ) -> Self {
        Self {
            port,
            host,
            name,
            username,
            password,
            require_ssl,
        }
    }

    pub fn connect_to_instance(&self) -> PgConnectOptions {
        Self::connect_to_pg_instance(self)
    }

    fn connect_to_pg_instance(pg_config: &PostgresConfig) -> PgConnectOptions {
        let ssl_mode = if pg_config.require_ssl {
            PgSslMode::Require
        } else {
            PgSslMode::Prefer
        };

        PgConnectOptions::new()
            .port(pg_config.port)
            .host(&pg_config.host)
            .ssl_mode(ssl_mode)
            .username(&pg_config.username)
            .password(&pg_config.password.expose_secret())
    }

    pub fn connect_to_database(&self, database_name: &str) -> PgConnectOptions {
        self.connect_to_instance().database(database_name)
    }
}
