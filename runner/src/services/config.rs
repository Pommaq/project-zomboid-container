use clap::Parser;

/// Ensures importer won't depend on whatever library we use for argument parsing
pub fn parse() -> Config {
    Config::parse()
}

#[derive(Parser, Debug)]
pub struct Config {
    /// Project zomboid start-server.sh location
    #[clap(env)]
    pub startup_sh_path: String,

    #[clap(env)]
    pub custom_server_parameters: Vec<String>,
}