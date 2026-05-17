pub mod styles;
pub mod types;

use clap::Parser;

#[derive(Parser)]
#[command(
    name = "momo",
    version,
    about = "Momo is a fast JS/TS runtime, package manager, bundler, and test runner",
    long_about = None,
    propagate_version = true,
    styles = styles::STYLES,
    color = clap::ColorChoice::Always,
)]
pub struct Cli {
    #[command(flatten)]
    pub global: types::GlobalArgs,

    #[command(subcommand)]
    pub command: types::Commands,
}

impl Cli {
    pub fn run(self) {
        match self.command {
            types::Commands::Build {
                watch,
                minify,
                entry,
            } => {
                println!("build");
            }

            types::Commands::Bench => {
                println!("bench");
            }

            types::Commands::Install => {
                println!("install");
            }

            types::Commands::Run { file } => {
                println!("run {file}");
            }

            types::Commands::Test { watch } => {
                println!("test {watch}");
            }

            types::Commands::Version { format } => {
                println!("{format:?}");
            }

            _ => {}
        }
    }
}
