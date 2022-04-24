use anyhow::{Context, Result};
use clap::Parser;
use std::ffi::OsString;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    #[clap(long, short)]
    pub show: bool,

    #[clap(required = true, multiple_values = true, allow_hyphen_values = true)]
    pub command: Vec<String>,
}

pub fn parse_arguments(args: &[OsString]) -> Result<Args> {
    Args::try_parse_from(args).context("failed to parse arguments")
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
        let result = parse_arguments(&[os("benchie"), os("time"), os("--show")]);

        assert!(result.is_ok(), "should succeed to parse benchmark command");

        let args = result.unwrap();

        assert_eq!(args.command.len(), 2, "command should has length 2");
        assert_eq!(
            args.command[0], "time",
            "first part of command should be time"
        );
        assert_eq!(
            args.command[1], "--show",
            "second part of command should be --show"
        );
    }
}
