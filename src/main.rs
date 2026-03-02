use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args = rtree::cli::Args::parse();
    rtree::app::run(args)
}
