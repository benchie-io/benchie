use crate::append_benchmark;
use anyhow::{Context, Result};
use std::process::Command;
use std::time::Instant;

pub fn benchmark(command_and_flags: &[String]) -> Result<()> {
    let (command, flags) = command_and_flags.split_at(1);
    let mut command = Command::new(&command[0]);

    for flag in flags {
        command.arg(flag);
    }

    let now = Instant::now();
    command.status().expect("running command failed");

    let elapsed_time = now.elapsed();
    println!(
        "Running \"{}\" took {:?}",
        command_and_flags.join(" "),
        elapsed_time
    );

    append_benchmark(command_and_flags, &elapsed_time).context("unable to save new benchmark")
}
