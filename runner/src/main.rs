mod services;
use services::zomboid;

// Globally expose the logging macros
#[macro_use]
extern crate log;
use env_logger;

#[tokio::main()]
async fn main() {
    env_logger::init();
    info!("This is a test");
    let config = services::config::parse();
    zomboid::run(&config.zomboid, &config.admin_password)
        .await
        .unwrap();
}
