use anyhow::anyhow;
use tracing::{error, info};
use xrfq3::{load_config, setup_tracing_logger};

fn main() -> anyhow::Result<()> {
    let config = load_config().map_err(|err| {
        error!("Failed to load configs, err={}", err);
        anyhow!("Failed to load configuration: {}", err)
    })?;

    let _guard = setup_tracing_logger(&config.app.name, &config.log);

    info!("!!!xrf197ilz35aq3!!!");

    Ok(())
}
