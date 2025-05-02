use crate::CassandraConfig;
use cassandra_cpp::Cluster;
use std::time::Duration;
use tracing::error;

pub async fn connect_session(
    config: CassandraConfig,
) -> Result<cassandra_cpp::Session, anyhow::Error> {
    // !!!!!! Configure cassandra cluster !!!!!!!
    let mut cluster = Cluster::default();
    cluster.set_connect_timeout(Duration::from_micros(2));

    cluster.set_contact_points(&config.host).map_err(|e| {
        error!("Failed to set contact points to cassandra: {:?}", e);
        anyhow::anyhow!("Failed to set contact points to cassandra :: err={}", e)
    })?;
    // For multiple nodes: cluster.set_contact_points("10.0.0.1,10.0.0.2,10.0.0.3")?;

    // Optional: Configure authentication if your cluster requires it
    cluster
        .set_credentials(&config.user, &config.password)
        .map_err(|e| {
            error!("Failed to set credentials to cassandra: {:?}", e);
            anyhow::anyhow!("Failed to set credentials to cassandra :: err={}", e)
        })?;

    // Optional: Set other cluster configurations (load balancing, timeouts, etc.)
    // cluster.set_load_balance_round_robin();

    // !!!!!! Connect to cassandra cluster !!!!!!!
    let session = cluster.connect().await.map_err(|e| {
        error!("Failed to connect to cassandra: {:?}", e);
        anyhow::anyhow!("Failed to connect to cassandra :: err={}", e)
    })?;

    Ok(session)
}
