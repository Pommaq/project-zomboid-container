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
        pub steamcmd: String,
        pub zomboid: String,
        pub admin_name: String,
        pub admin_password: String,
        /// List of steam workshop IDs to be installed on the server
        pub workshop_ids: Option<Vec<usize>>,
    }
}
