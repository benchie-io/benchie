use anyhow::Result;
use benchie::show;
use benchie::{benchmark, initialize_crash_reporter};
use std::env;

mod cli;

fn main() -> Result<()> {
    initialize_crash_reporter();

    let raw_args: Vec<_> = env::args_os().collect();
    let args = cli::parse_arguments(&raw_args)?;

    // TODO: should be a subcommand?
    if args.show {
        show()
    } else {
        benchmark(&args.command)
    }
}
