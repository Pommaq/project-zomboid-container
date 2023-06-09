mod services;

use ctrlc::set_handler;
use services::zomboid;
use std::{process::exit, time::Duration};
use tokio::sync::mpsc;

#[macro_use]
extern crate log;

#[tokio::main()]
async fn main() {
    env_logger::init();
    let config = services::config::parse();

    info!("Patching start-server.sh so it propagates signals correctly...");
    if let Err(error) = zomboid::patch_start_script(&config.startup_sh_path).await {
        error!("Unable to patch script: {}", error);
        return;
    }

    let (tx, rx) = mpsc::channel(32);
    set_handler(move || {
        tx.blocking_send(()).expect("Unable to kill zomboid server");
    })
    .expect("Unable to install handler");
    info!("Installed signal handler for stopping server");

    info!("Starting wrapper");
    let game = match zomboid::run(&config.startup_sh_path, config.custom_server_parameters).await {
        Ok(g) => g,
        Err(error) => {
            error!("Failed to run game: {}", error);
            return;
        }
    };

    match zomboid::wait_for(game, rx, Duration::from_secs(config.exit_timeout)).await {
        Ok(code) => {
            info!("Exit status: {}", code);
            // We do this to propagate errors to caller, and to ensure our
            // other routines die.
            exit(code)
        }
        Err(error) => error!("Failed: {}", error),
    }
}
