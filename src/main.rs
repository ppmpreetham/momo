mod cli;
mod macros;
mod manager;
mod test;

use clap::Parser;
use cli::Cli;

fn main() {
    let cli = Cli::parse();

    cli.run();
}
