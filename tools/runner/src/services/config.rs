use clap::Parser;

/// Ensures importer won't depend on whatever library we use for argument parsing
pub fn parse() -> Config {
    Config::parse()
}

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Config {
    /// Project zomboid start-server.sh location
    #[clap(env)]
    pub startup_sh_path: String,

    #[clap(env)]
    /// A comma (,) separated string of parameters that will be passed directly
    /// to the zomboid server
    pub custom_server_parameters: String,

    /// The maximum amount of seconds we wait for the game to exit before murdering it.
    /// we should only hit this in worst-case when the server refuses to die despite us asking it to.
    /// 0 means disabled
    #[clap(env)]
    #[arg(default_value_t = 900)]
    pub exit_timeout: u64,
}
