use crate::context::Environment;
use crate::{GrpcServerConfig, CERT_PEM_PATH, KEY_PEM_PATH};
use anyhow::Context;
use bytes::Bytes;
use std::fs;
use std::path::Path;
use std::time::Duration;
use tonic::transport::{Identity, Server, ServerTlsConfig};
use tracing::{debug, info, warn};

const SSL_PEM_SERVE_KEY_PATH: &str = "./local/secret/ssl/server.key";
const SSL_PEM_SERVE_CERT_PATH: &str = "./local/secret/ssl/server.crt";

pub struct GrpcServer {
    timeout: Duration,
    addr: core::net::SocketAddr,
}

impl GrpcServer {
    pub fn new(config: GrpcServerConfig) -> Result<Self, anyhow::Error> {
        let addr = format!("[::]:{}", config.port)
            .parse()
            .context("Failed to parse grpc server address")?;

        let config_timeout = config.timeout;
        Ok(GrpcServer {
            addr,
            timeout: Duration::from_millis(config_timeout as u64),
        })
    }

    pub async fn run_until_stopped(self, app_env: &Environment) -> anyhow::Result<()> {
        info!("starting gRPC server :: port {}", &self.addr.port());
        let key_path = &get_path_from_env_or(KEY_PEM_PATH, SSL_PEM_SERVE_KEY_PATH, &app_env)?;
        let cert_path = &get_path_from_env_or(CERT_PEM_PATH, SSL_PEM_SERVE_CERT_PATH, &app_env)?;

        //// Load the PEM-encoded data directly. Pem (Privacy-Enhanced Mail)
        let cert_pem = load_pem_data(Path::new(cert_path))?;
        ///// This extension is conventionally used for files that contain only
        //         // private keys. Again, it's still PEM-encoded data
        let key_pem = load_pem_data(Path::new(key_path))?;

        info!("starting... gRPC server");
        // Server::builder()
        //     .tls_config(ServerTlsConfig::new().identity(Identity::from_pem(cert_pem, key_pem)))
        //     .context("Failed to create TLS config")?
        //     .max_connection_age(self.timeout)
        //     .serve(self.addr)
        //     .await
        //     .context("gRPC server failed")
        unimplemented!()
    }
}

fn load_pem_data(path: &Path) -> anyhow::Result<Bytes> {
    debug!("loading pem file from :: path={:?}", path);
    fs::read(path)
        .map(Bytes::from)
        .with_context(|| format!("Failed to read PEM data from {}", path.display()))
}

fn get_path_from_env_or(
    env_key: &str,
    default: &str,
    app_env: &Environment,
) -> anyhow::Result<String> {
    let path_from_env = std::env::var(env_key);
    if path_from_env.is_err() {
        if app_env.is_not_local() {
            return Err(anyhow::anyhow!("Invalid/missing XRF Environment variables"));
        }
        warn!(
            "Environment variable {} is missing, will use default :: env={}",
            env_key, app_env
        );
    }
    let path = path_from_env.expect(default);
    Ok(path)
}
