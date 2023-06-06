/// The library where the magic happens.Responsible for all logic related to running the server
pub mod zomboid;

pub mod config {
    use clap::Parser;

    /// Ensures importer won't depend on whatever library we use for argument parsing
    pub fn parse() -> Config {
        Config::parse()
    }

    #[derive(Parser, Debug)]
    pub struct Config {
        /// Project zomboid start-server.sh location
        pub zomboid: String,
        pub admin_password: String,
    }
}
