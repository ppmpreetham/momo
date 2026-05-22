mod cli;
mod macros;
mod manager;
mod test;

use clap::Parser;
use cli::Cli;

fn main() {
    Cli::parse().run();
}
