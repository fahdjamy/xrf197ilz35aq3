use crate::storage::apply_cql_file;
use crate::CassandraConfig;
use anyhow::anyhow;
use cassandra_cpp::Cluster;
use std::fs::read_dir;
use std::time::Duration;
use tracing::{debug, error, info};

pub async fn connect_session(
    config: &CassandraConfig,
) -> Result<cassandra_cpp::Session, anyhow::Error> {
    // !!!!!! Configure cassandra cluster !!!!!!!
    let mut cluster = Cluster::default();
    cluster.set_connect_timeout(Duration::from_micros(2));

    // Specify nodes to contact:
    debug!(
        "****** connecting to cluster, port={}, password={}, user={}, host={} ******",
        &config.port, &config.password, &config.user, &config.host
    );
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

    // (Optional) How long to wait for connecting before bailing out:
    cluster.set_connect_timeout(Duration::from_secs(config.connect_timeout as u64));

    // Optional: Set other cluster configurations (load balancing, timeouts, etc.)
    // cluster.set_load_balance_round_robin();

    // !!!!!! Connect to cassandra cluster !!!!!!!
    let session = cluster.connect().await.map_err(|e| {
        error!("Failed to connect to cassandra: {:?}", e);
        anyhow::anyhow!("Failed to connect to cassandra :: err={}", e)
    })?;

    Ok(session)
}

pub async fn create_keyspace(
    keyspace: &str,
    replication_factor: u8,
    session: &cassandra_cpp::Session,
) -> Result<(), anyhow::Error> {
    let query = format!("CREATE KEYSPACE IF NOT EXISTS {} WITH REPLICATION = {{ 'class' : 'SimpleStrategy', 'replication_factor' : {} }}",
                        keyspace,
                        replication_factor);

    session.execute(query.as_str()).await.map_err(|err| {
        error!("Failed to create keyspace: {:?}", err);
        anyhow::anyhow!("Failed to create keyspace :: err={}", err)
    })?;

    Ok(())
}

pub async fn apply_cql_migrations(
    cql_dir: &str,
    session: &cassandra_cpp::Session,
) -> Result<(), anyhow::Error> {
    info!("applying cql migrations...");

    let mut cql_files = Vec::new();

    // 1. Read the directory entries.
    let entries =
        read_dir(cql_dir).map_err(|err| anyhow!("Failed to read cql directory: {:?}", err))?;

    // 2. Collect all valid .cql file paths.
    for entry in entries {
        let entry = entry.map_err(|err| anyhow!("Failed to read cql entry: {:?}", err))?;
        let path = entry.path();

        // Check if it's a file and has the .cql extension.
        if path.is_file() {
            if let Some(extension) = path.extension() {
                if extension == "cql" {
                    cql_files.push(path);
                }
            }
        }
    }

    if cql_files.is_empty() {
        info!("No cql files found, skipping CQL migration run...");
        return Ok(());
    }

    // 3. Sort the files alphabetically to ensure a consistent execution order.
    //    This is crucial for migrations (e.g., 001_..., 002_...).
    cql_files.sort();

    // 4. Execute each file in order.
    for path in &cql_files {
        apply_cql_file(path, session).await?;
    }

    info!("CQL migrations successfully run.");
    Ok(())
}
