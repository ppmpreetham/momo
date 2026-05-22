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
    #[command(visible_alias = "b")]
    Build {
        #[arg(short, long)]
        watch: bool,

        #[arg(short, long)]
        minify: bool,

        #[arg(value_name = "ENTRY")]
        entry: Vec<String>,
    },

    /// Run benchmarks
    #[command(visible_alias = "be")]
    Bench,

    /// Initialize a new project
    Init,

    /// Install dependencies
    #[command(visible_alias = "i")]
    Install,

    /// Run a JS/TS file
    Run { file: String },

    /// Test JS/TS files
    #[command(visible_alias = "t")]
    Test {
        // #[arg(short, long)]
        filter: Option<String>,
    },

    #[command(visible_alias = "upd")]
    Update,
    #[command(visible_alias = "upg")]
    Upgrade,

    Version {
        #[arg(short, long, value_enum, default_value = "text")]
        format: VersionFormat,
    },
}
