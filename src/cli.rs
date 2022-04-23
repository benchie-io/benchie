use crate::storage::append_benchmark;
use clap::Parser;
use std::process::Command;
use std::time::Instant;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(long, short)]
    flag: bool,

    #[clap(required = true, multiple_values = true, allow_hyphen_values = true)]
    command: Vec<String>,
}

pub fn cli() {
    let cli = Cli::parse();

    let command_and_flags: Vec<String> = cli.command;

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

    let result = append_benchmark(&command_and_flags, &elapsed_time);
    match result {
        Ok(_) => {}
        Err(e) => panic!("{}", e),
    }
}
