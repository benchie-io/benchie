use crate::cli::CliCommand;
use anyhow::Result;
use benchie::show;
use benchie::{benchmark, initialize_crash_reporter};
use std::env;

mod cli;

fn main() -> Result<()> {
    initialize_crash_reporter();

    let raw_args: Vec<_> = env::args_os().collect();

    match cli::parse_arguments(&raw_args)? {
        CliCommand::Benchmark { command } => benchmark(&command),
        CliCommand::Show => show(),
    }
}
