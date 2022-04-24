use anyhow::Result;
use benchie::benchmark;
use benchie::show;
use std::env;

mod cli;

fn main() -> Result<()> {
    let raw_args: Vec<_> = env::args_os().collect();
    let args = cli::parse_arguments(&raw_args)?;

    // TODO: should be a subcommand?
    if args.show {
        show()
    } else {
        benchmark(&args.command)
    }
}
