use anyhow::Result;
use clap::{arg, crate_name, crate_version, Arg, Command};
use std::ffi::OsString;

pub mod sub_commands {
    pub const SHOW: &str = "show";
}

pub enum CliCommand {
    Benchmark { command: Vec<String> },
    Show,
}

pub fn parse_arguments(args: &[OsString]) -> Result<CliCommand> {
    let matches = Command::new(crate_name!())
        .version(crate_version!())
        .propagate_version(true)
        .arg_required_else_help(true)
        .args_conflicts_with_subcommands(true)
        .arg(
            Arg::new("command")
                .takes_value(true)
                .multiple_values(true)
                .allow_hyphen_values(true),
        )
        .subcommand(
            Command::new(sub_commands::SHOW)
                .about("Shows benchmarking results")
                .arg(arg!([NAME])),
        )
        .try_get_matches_from(args)?;

    Ok(if let Some(command) = matches.values_of("command") {
        let command: Vec<String> = command.into_iter().map(|s| s.to_owned()).collect();

        CliCommand::Benchmark { command }
    } else {
        match matches.subcommand() {
            Some(("show", _)) => CliCommand::Show,
            m => panic!("unknown subcommand {:?}", m),
        }
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;

    fn os(string: &str) -> OsString {
        OsString::from_str(string).unwrap()
    }

    #[test]
    fn test_minimum_required_arguments() {
        let result = parse_arguments(&[os("benchie")]);

        assert!(result.is_err(), "should has at least one argument");
    }

    #[test]
    fn test_benchmark_command() {
        let result = parse_arguments(&[os("benchie"), os("time")]);

        if let Ok(CliCommand::Benchmark { command }) = result {
            assert_eq!(command.len(), 1, "command should has length 1");
            assert_eq!(command[0], "time", "first part of command should be time");
        } else {
            panic!("should succeed to parse benchmark command");
        }
    }

    #[test]
    fn test_benchmark_with_hyphen_command_args() {
        let result = parse_arguments(&[os("benchie"), os("time"), os("--SHOW")]);

        if let Ok(CliCommand::Benchmark { command }) = result {
            assert_eq!(command.len(), 2, "command should has length 2");
            assert_eq!(command[0], "time", "first part of command should be time");
            assert_eq!(
                command[1], "--SHOW",
                "second part of command should be --SHOW"
            );
        } else {
            panic!("should succeed to parse benchmark command");
        }
    }

    #[test]
    fn test_show_subcommand() {
        let command = parse_arguments(&[os("benchie"), os("show")]);

        assert!(
            matches!(command, Ok(CliCommand::Show)),
            "should succeed to parse show subcommand"
        );
    }
}
