mod services;
use services::zomboid;

// Globally expose the logging macros
#[macro_use]
extern crate log;
use env_logger;

#[tokio::main()]
async fn main() {
    env_logger::init();
    info!("Starting wrapper");
    let config = services::config::parse();
    let res = zomboid::run(&config.startup_sh_path, config.custom_server_parameters).await;
    if let Err(error) = res {
        error!("{}", error);
    }
}
