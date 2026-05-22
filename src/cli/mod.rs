pub mod styles;
pub mod types;

use clap::Parser;
use std::time::Instant;

#[derive(Parser)]
#[command(
    name = "momo",
    version,
    about = "Momo is a fast JS/TS runtime, package manager, bundler, and test runner",
    long_about = None,
    propagate_version = true,
    styles = styles::STYLES,
    color = clap::ColorChoice::Always,
    arg_required_else_help = true
)]
pub struct Cli {
    #[command(flatten)]
    pub global: types::GlobalArgs,

    #[command(subcommand)]
    pub command: types::Commands,
}

impl Cli {
    pub fn run(self) {
        let start_time = Instant::now();

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

            types::Commands::Test { filter } => {
                use crate::test::test;
                if let Some(ref filter) = filter {
                    println!("Running tests matching: {filter}");
                }
                test(filter.as_deref());
            }

            types::Commands::Version { format } => {
                println!("{format:?}");
            }

            _ => {}
        }

        Self::print_execution_time(start_time);
    }

    fn print_execution_time(start: Instant) {
        let elapsed = start.elapsed();

        if elapsed.as_secs() >= 1 {
            println!("\n Command Ran in {:.2}s.", elapsed.as_secs_f32());
        } else {
            println!("\n Command Ran in {}ms.", elapsed.as_millis());
        }
    }
}
