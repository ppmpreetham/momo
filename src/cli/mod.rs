mod styles;
mod types;
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
struct Cli {
    #[command(flatten)]
    global: types::GlobalArgs,

    #[command(subcommand)]
    command: types::Commands,
}
