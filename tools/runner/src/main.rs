mod services;

use services::zomboid;
use std::time::Duration;

// Globally expose the logging macros
#[macro_use]
extern crate log;
use env_logger;

#[tokio::main()]
async fn main() {
    env_logger::init();
    let config = services::config::parse();

    info!("Patching start-server.sh so it propagates signals correctly...");
    if let Err(error) = zomboid::patch_start_script(&config.startup_sh_path).await {
        error!("Unable to patch script: {}", error);
        return;
    }
    info!("Starting wrapper");
    if let Err(error) = zomboid::run(
        &config.startup_sh_path,
        config.custom_server_parameters,
        Duration::from_secs(config.exit_timeout),
    )
    .await
    {
        error!("Failed to run game: {}", error);
    }
}
