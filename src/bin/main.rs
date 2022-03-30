use anyhow::Result;
use clap::Parser;
use jet_cli::Opts;

fn main() -> Result<()> {
    jet_cli::run(Opts::parse())
}
