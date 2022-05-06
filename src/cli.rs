use anyhow::Result;
use clap::{arg, crate_name, crate_version, Arg, Command};
use std::ffi::OsString;

pub mod sub_commands {
    pub const SHOW: &str = "show";
}

pub enum CliCommand {
    Benchmark {
        command: Vec<String>,
    },
    Show {
        row: Option<String>,
        col: Option<String>,
        metric: Option<String>,
    },
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
                .arg(
                    arg!(--row <ROW> "The row to display")
                        .short('r')
                        .required(false)
                        .requires("metric"),
                )
                .arg(
                    arg!(--col <COLUMN> "The column to display")
                        .short('c')
                        .required(false)
                        .requires("row")
                        .requires("metric"),
                )
                .arg(
                    arg!(<METRIC> "The metric to display")
                        .required(false)
                        .id("metric")
                        .requires("row"),
                ),
        )
        .try_get_matches_from(args)?;

    Ok(if let Some(command) = matches.values_of("command") {
        let command: Vec<String> = command.into_iter().map(|s| s.to_owned()).collect();

        CliCommand::Benchmark { command }
    } else {
        match matches.subcommand() {
            Some(("show", sub_commands)) => CliCommand::Show {
                row: sub_commands.value_of("row").map(str::to_string),
                col: sub_commands.value_of("col").map(str::to_string),
                metric: sub_commands.value_of("metric").map(str::to_string),
            },
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
        let cmd_default_show = parse_arguments(&[os("benchie"), os("show")]);

        assert!(
            matches!(
                cmd_default_show,
                Ok(CliCommand::Show {
                    row: None,
                    col: None,
                    metric: None
                })
            ),
            "should succeed to parse show subcommand"
        );
    }

    #[test]
    fn show_1d_table_should_have_row_and_metric() {
        match parse_arguments(&[
            os("benchie"),
            os("show"),
            os("--row"),
            os("test_row"),
            os("test_metric"),
        ]) {
            Ok(CliCommand::Show {
                row,
                col: _,
                metric,
            }) => {
                assert_eq!(row.unwrap(), "test_row");
                assert_eq!(metric.unwrap(), "test_metric");
            }
            _ => panic!("show argument with given row and metric should work"),
        }
    }

    #[test]
    fn show_2d_table_should_have_row_col_and_metric() {
        match parse_arguments(&[
            os("benchie"),
            os("show"),
            os("--row"),
            os("test_row"),
            os("--col"),
            os("test_column"),
            os("test_metric"),
        ]) {
            Ok(CliCommand::Show { row, col, metric }) => {
                assert_eq!(row.unwrap(), "test_row");
                assert_eq!(col.unwrap(), "test_column");
                assert_eq!(metric.unwrap(), "test_metric");
            }
            _ => panic!("show argument with given row, column, and metric should work"),
        }
    }
}
