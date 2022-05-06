use crate::cli::CliCommand;
use anyhow::Result;
use benchie::{benchmark, initialize_crash_reporter};
use benchie::{show, show_1d_table, show_2d_table};
use std::env;

mod cli;

fn main() -> Result<()> {
    initialize_crash_reporter();

    let raw_args: Vec<_> = env::args_os().collect();

    match cli::parse_arguments(&raw_args)? {
        CliCommand::Benchmark { command } => benchmark(&command),
        CliCommand::Show { row, col, metric } => match (row, col, metric) {
            (Some(row), Some(col), Some(metric)) => show_2d_table(row, col, metric),
            (Some(row), _, Some(metric)) => show_1d_table(row, metric),
            _ => show(),
        },
    }
}
