use crate::append_benchmark;
use crate::git::{read_git_info, GitError};
use crate::os::execute_and_measure;
use crate::system::System;
use crate::utils::{is_key_value_pair, parse_key_value_pair};
use crate::Value;
use crate::{value, GitInfo};
use anyhow::{bail, Context, Result};
use chrono::prelude::*;
use colored::*;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

#[derive(Serialize, Default, Deserialize, Debug, Clone)]
pub struct BenchmarkRaw {
    #[serde(flatten)]
    pub data: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Benchmark {
    #[serde(with = "value")]
    command: String,

    #[serde(with = "value")]
    created_at: DateTime<Utc>,

    #[serde(flatten)]
    git: Option<GitInfo>,

    #[serde(flatten)]
    system: System,

    #[serde(flatten)]
    result: ExecutionResult,

    #[serde(flatten)]
    tags: HashMap<String, Value>,
}

impl Benchmark {
    pub fn new(
        command: &[String],
        result: &ExecutionResult,
        git: &Option<GitInfo>,
        tags: &HashMap<String, String>,
    ) -> Self {
        Self {
            command: command.join(" "),
            created_at: Utc::now(),
            git: git.clone(),
            system: System::default(),
            result: result.clone(),
            tags: tags
                .iter()
                .map(|(key, value)| (key.clone(), value.into()))
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExecutionResult {
    #[serde(with = "value")]
    pub user_time: Duration,

    #[serde(with = "value")]
    pub system_time: Duration,

    #[serde(with = "value")]
    pub real_time: Duration,

    #[serde(with = "value")]
    pub status_code: i64,
}

#[allow(dead_code)]
pub fn parse_tags_from_stdout(output: &str) -> Result<HashMap<String, String>> {
    let mut pairs = vec![];

    for line in output.split('\n') {
        if line.starts_with("@benchie") {
            if let Some((_, kv)) = line.split_once(' ') {
                if is_key_value_pair(kv).is_err() {
                    println!(
                        "{}",
                        format!("warning: invalid key-value pair format: \"{kv}\"").yellow()
                    );
                } else {
                    let (k, v) = parse_key_value_pair(kv);
                    pairs.push((k, v));
                }
            }
        }
    }

    let unique_keys = pairs.iter().unique_by(|(key, _)| key).count();

    if pairs.len() != unique_keys {
        let duplicates: Vec<_> = pairs.iter().duplicates_by(|(key, _)| key).collect();
        // TODO: should this be a warning?
        bail!(
            "found multiple duplicates when checking key value pairs: {:?}",
            duplicates
        )
    } else {
        Ok(pairs.into_iter().collect())
    }
}

pub fn benchmark(command_and_flags: &[String], tags: &HashMap<String, String>) -> Result<()> {
    let git_info = match read_git_info() {
        Ok(info) => {
            if info.is_dirty {
                println!(
                    "{}",
                    "warning: you have uncommitted changes in your repository".yellow()
                )
            }
            Some(info)
        }
        Err(GitError::Unknown(error)) => {
            return Err(error)
                .context("unknown error during reading of Git repository information");
        }
        Err(known_error) => {
            println!(
                "{}",
                format!(
                    "warning: {} => no Git information will be saved for your benchmark",
                    known_error
                )
                .yellow()
            );
            None
        }
    };

    let (result, cmd_tags) =
        execute_and_measure(command_and_flags).context("failed to execute command")?;

    tags.iter().for_each(|(key, _)| {
        if cmd_tags.contains_key(key.as_str()) {
            println!(
                "{}",
                format!("warning: you are overwriting the tag with key \"{key}\"").yellow()
            )
        }
    });

    let mut merged_tags = tags.clone();
    merged_tags.extend(cmd_tags);

    println!("Running \"{}\" took:", command_and_flags.join(" "));
    println!(
        "{:?} user {:?} system {:?} real",
        result.user_time, result.system_time, result.real_time
    );

    if result.status_code != 0 {
        println!(
            "{}",
            format!(
                "warning: benchmarked program exited with status code {}",
                result.status_code
            )
            .yellow()
        );
    }

    let benchmark = Benchmark::new(command_and_flags, &result, &git_info, &merged_tags);

    append_benchmark(&benchmark).context("unable to save new benchmark")
}
