mod services;
use services::zomboid;
/// Cool
#[tokio::main()]
async fn main() {
    let config = services::config::parse();
    zomboid::run(
        &config.steamcmd,
        &config.zomboid,
        "/install_dir",
        config.workshop_ids,
        &config.admin_name,
        &config.admin_password,
    )
    .unwrap();
}
