use anyhow::{bail, Result};
use benchie::{is_key_value_pair, parse_key_value_pair};
use clap::{arg, crate_name, crate_version, Arg, Command, Values};
use itertools::Itertools;
use std::collections::HashMap;
use std::ffi::OsString;

pub mod sub_commands {
    pub const SHOW: &str = "show";
}

#[derive(Debug, Clone)]
pub enum CliCommand {
    Benchmark {
        command: Vec<String>,
        tags: HashMap<String, String>,
    },
    Show {
        row: Option<String>,
        col: Option<String>,
        metric: Option<String>,
        filter: HashMap<String, String>,
    },
}

pub fn parse_arguments(args: &[OsString]) -> Result<CliCommand> {
    let matches = Command::new(crate_name!())
        .version(crate_version!())
        .propagate_version(true)
        .arg_required_else_help(true)
        .args_conflicts_with_subcommands(true)
        .arg(
            Arg::new("tag")
                .long("tag")
                .takes_value(true)
                .multiple_occurrences(true)
                .validator(is_key_value_pair),
        )
        .arg(
            Arg::new("command")
                .takes_value(true)
                .multiple_values(true)
                .allow_hyphen_values(true)
                .required(true),
        )
        .subcommand_precedence_over_arg(false)
        .subcommand_negates_reqs(true)
        .subcommand(
            Command::new(sub_commands::SHOW)
                .about("Shows benchmarking results")
                .arg(
                    arg!(--filter <PREDICATE> "The predicate to use to filter benchmarks")
                        .required(false)
                        .multiple_occurrences(true)
                        .validator(is_key_value_pair),
                )
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

    Ok(match matches.subcommand() {
        Some((sub_commands::SHOW, sub_commands)) => CliCommand::Show {
            row: sub_commands.value_of("row").map(str::to_string),
            col: sub_commands.value_of("col").map(str::to_string),
            metric: sub_commands.value_of("metric").map(str::to_string),
            filter: parse_key_value_pairs(sub_commands.values_of("filter"))?,
        },
        m => {
            if let Some(command) = matches.values_of("command") {
                let command: Vec<String> = command.into_iter().map(|s| s.to_owned()).collect();

                let tags = parse_key_value_pairs(matches.values_of("tag"))?;

                CliCommand::Benchmark { command, tags }
            } else {
                panic!(
                    "can not parse input arguments ({:?}) to subcommand {:?}",
                    args, m
                )
            }
        }
    })
}

fn parse_key_value_pairs(it: Option<Values>) -> Result<HashMap<String, String>> {
    let pairs: Vec<_> = it.map_or(vec![], |it| it.map(parse_key_value_pair).collect());

    let unique_keys = pairs.iter().unique_by(|(key, _)| key).count();

    if pairs.len() != unique_keys {
        let duplicates: Vec<_> = pairs.iter().duplicates_by(|(key, _)| key).collect();

        bail!(
            "found multiple duplicates when checking key value pairs: {:?}",
            duplicates
        )
    } else {
        Ok(pairs.into_iter().collect())
    }
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

        if let Ok(CliCommand::Benchmark { command, tags: _ }) = result {
            assert_eq!(command.len(), 1, "command should has length 1");
            assert_eq!(command[0], "time", "first part of command should be time");
        } else {
            panic!("should succeed to parse benchmark command");
        }
    }

    #[test]
    fn test_benchmark_with_hyphen_command_args() {
        let result = parse_arguments(&[os("benchie"), os("time"), os("--SHOW")]);

        if let Ok(CliCommand::Benchmark { command, tags: _ }) = result {
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
                    metric: None,
                    filter: _
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
                filter: _,
            }) => {
                assert_eq!(row.unwrap(), "test_row");
                assert_eq!(metric.unwrap(), "test_metric");
            }
            _ => panic!("show argument with given row and metric should work"),
        }
    }

    #[test]
    fn tag_arg_is_only_acceptable_if_command_is_set() {
        let result = parse_arguments(&[os("benchie"), os("--tag"), os("key=value")]);

        assert!(
            result.is_err(),
            "tag without benchmarking command is not a valid input"
        );
    }

    #[test]
    fn tag_arg_with_command_should_work() {
        match parse_arguments(&[os("benchie"), os("--tag"), os("key=value"), os("program")]) {
            Ok(CliCommand::Benchmark { command, tags }) => {
                assert_eq!(command.len(), 1);
                assert_eq!(command.get(0).unwrap(), "program");
                assert_eq!(tags.len(), 1);
                assert_eq!(tags.get("key").unwrap(), "value");
            }
            _ => panic!("tag argument with command should work"),
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
            Ok(CliCommand::Show {
                row,
                col,
                metric,
                filter: _,
            }) => {
                assert_eq!(row.unwrap(), "test_row");
                assert_eq!(col.unwrap(), "test_column");
                assert_eq!(metric.unwrap(), "test_metric");
            }
            _ => panic!("show argument with given row, column, and metric should work"),
        }
    }

    #[test]
    fn multiple_tags_are_allowed() {
        match parse_arguments(&[
            os("benchie"),
            os("--tag"),
            os("key=value"),
            os("--tag"),
            os("bla=value"),
            os("program"),
        ]) {
            Ok(CliCommand::Benchmark { command: _, tags }) => {
                assert_eq!(tags.len(), 2);
            }
            _ => panic!("multiple tags should be allowed"),
        }
    }

    #[test]
    fn two_filters_with_same_key_fail() {
        let result = parse_arguments(&[
            os("benchie"),
            os("show"),
            os("--filter"),
            os("key=value"),
            os("--filter"),
            os("key=value2"),
        ]);

        assert!(
            result.is_err(),
            "two filters for the same key are not allowed at the moment"
        );
    }

    #[test]
    fn multiple_filter_arguments_are_allowed() {
        match parse_arguments(&[
            os("benchie"),
            os("show"),
            os("--filter"),
            os("key=value"),
            os("--filter"),
            os("key2=value2"),
        ]) {
            Ok(CliCommand::Show { filter, .. }) => {
                assert_eq!(filter.len(), 2);
                assert_eq!(filter.get("key"), Some(&"value".to_string()));
                assert_eq!(filter.get("key2"), Some(&"value2".to_string()));
            }
            _ => panic!("tag argument with command should work"),
        }
    }
}
