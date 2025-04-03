use anyhow::anyhow;
use xrfq3::{load_config, setup_tracing_logger};

fn main() -> anyhow::Result<()> {
    let config = load_config().map_err(|err| {
        anyhow!("Failed to load configuration: {}", err)
    })?;

    let _guard = setup_tracing_logger(&config.app.name, &config.log);

    println!("!!!xrf197ilz35aq3!!!");

    Ok(())
}
