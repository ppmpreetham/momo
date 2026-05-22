use clap::{Args, Subcommand, ValueEnum};

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum VersionFormat {
    /// For Humans
    Text,
    /// For Machines
    Json,
}

#[derive(Args, Debug, Clone)]
pub struct GlobalArgs {
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

#[derive(Subcommand, Clone, Debug)]
pub enum Commands {
    /// Bundle JS and TS files
    #[command(alias = "b")]
    Build {
        #[arg(short, long)]
        watch: bool,

        #[arg(short, long)]
        minify: bool,

        #[arg(value_name = "ENTRY")]
        entry: Vec<String>,
    },

    /// Run benchmarks
    #[command(alias = "be")]
    Bench,

    /// Initialize a new project
    Init,

    /// Install dependencies
    #[command(alias = "i")]
    Install,

    /// Run a JS/TS file
    Run { file: String },

    /// Test JS/TS files
    #[command(alias = "t")]
    Test { filter: Option<String> },

    /// Update all the globally installed packages
    #[command(alias = "upd")]
    Update,

    /// Upgrade Momo to the latest version    
    #[command(alias = "upg")]
    Upgrade,

    /// Display version information
    Version {
        #[arg(short, long, value_enum, default_value = "text")]
        format: VersionFormat,
    },
}
