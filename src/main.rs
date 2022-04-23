use anyhow::Result;
use benchie::benchmark;

mod cli;

fn main() -> Result<()> {
    let args = cli::parse_arguments();

    benchmark(&args.command)
}
