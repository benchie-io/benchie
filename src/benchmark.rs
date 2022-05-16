use crate::append_benchmark;
use crate::git::{read_git_info, GitError};
use crate::system::System;
use crate::utils::{is_key_value_pair, parse_key_value_pair};
use crate::Value;
use crate::{value, GitInfo};
use anyhow::{bail, ensure, Context, Result};
use chrono::prelude::*;
use colored::*;
use itertools::Itertools;
use libc::{
    c_char, c_int, close, pid_t, pipe, posix_spawn_file_actions_addclose,
    posix_spawn_file_actions_adddup2, posix_spawn_file_actions_init, posix_spawn_file_actions_t,
    posix_spawnattr_init, posix_spawnattr_t, posix_spawnp, rusage, timeval, wait4,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::ffi::CString;
use std::mem::MaybeUninit;
use std::time::{Duration, Instant};
use std::{fs::File, io::Read, os::unix::io::FromRawFd};

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

fn parse_tags_from_stdout(output: &str) -> Result<HashMap<String, String>> {
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

pub fn execute_and_measure(
    command_and_flags: &[String],
) -> Result<(ExecutionResult, HashMap<String, String>)> {
    ensure!(
        !command_and_flags.is_empty(),
        "command can not be empty for benchmarking"
    );

    let command_and_flags = command_and_flags
        .iter()
        .map(|str| CString::new(str.as_str()))
        .collect::<Result<Vec<_>, _>>()
        .context("invalid string encoding for user command and flags")?;

    let envs: Vec<_> = env::vars()
        .map(|(key, value)| CString::new(format!("{key}={value}")))
        .collect::<Result<Vec<_>, _>>()
        .context("invalid string encoding of environment variables")?;

    let program = &command_and_flags[0];

    let mut pid = MaybeUninit::<pid_t>::uninit();
    let mut file_actions = MaybeUninit::<posix_spawn_file_actions_t>::uninit();
    let mut spawnattr = MaybeUninit::<posix_spawnattr_t>::uninit();

    let mut status = MaybeUninit::<c_int>::uninit();
    let mut rusage = MaybeUninit::<rusage>::uninit();

    let envp = create_null_terminated(&envs);
    let argv = create_null_terminated(&command_and_flags);

    let mut pipe_file_descriptors = [0, 0];

    unsafe {
        ensure!(posix_spawn_file_actions_init(file_actions.as_mut_ptr()) == 0);
        ensure!(posix_spawnattr_init(spawnattr.as_mut_ptr()) == 0);

        ensure!(pipe(pipe_file_descriptors.as_mut_ptr()) != -1);

        // tell spawned process to close unused read end of pipe
        ensure!(
            posix_spawn_file_actions_addclose(file_actions.as_mut_ptr(), pipe_file_descriptors[0])
                == 0
        );

        // tell spawned process to replace file descriptor 1 (stdout) with write end of the pipe
        ensure!(
            posix_spawn_file_actions_adddup2(
                file_actions.as_mut_ptr(),
                pipe_file_descriptors[1],
                1
            ) == 0
        );

        let now = Instant::now();

        let result = posix_spawnp(
            pid.as_mut_ptr(),
            program.as_ptr(),
            file_actions.as_mut_ptr(),
            spawnattr.as_mut_ptr(),
            argv.as_ptr(),
            envp.as_ptr(),
        );

        close(pipe_file_descriptors[1]);

        ensure!(
            result == 0,
            "spawning of process failed for execution of command"
        );

        let result = wait4(
            pid.assume_init(),
            status.as_mut_ptr(),
            0,
            rusage.as_mut_ptr(),
        );

        let real_time = now.elapsed();

        ensure!(
            result != 0,
            "could not await process after spawning a process"
        );

        let user_time = timeval_to_duration(rusage.assume_init().ru_utime)?;
        let system_time = timeval_to_duration(rusage.assume_init().ru_stime)?;

        let mut f = File::from_raw_fd(pipe_file_descriptors[0]);
        let mut cmd_output = String::new();
        f.read_to_string(&mut cmd_output)?;

        let tags_from_stdout = parse_tags_from_stdout(&cmd_output)?;

        print!("{}", cmd_output);

        Ok((
            ExecutionResult {
                user_time,
                system_time,
                real_time,
                status_code: status.assume_init().into(),
            },
            tags_from_stdout,
        ))
    }
}

fn timeval_to_duration(value: timeval) -> Result<Duration> {
    let secs = Duration::from_secs(value.tv_sec.try_into()?);
    let micros = Duration::from_micros(value.tv_usec.try_into()?);

    Ok(secs + micros)
}

fn create_null_terminated(strings: &[CString]) -> Vec<*mut c_char> {
    let mut list = vec![std::ptr::null_mut::<c_char>(); strings.len() + 1];

    strings.iter().enumerate().for_each(|(i, str)| {
        list[i] = str.as_ptr() as *mut c_char;
    });

    list
}
