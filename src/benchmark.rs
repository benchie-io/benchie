use crate::append_benchmark;
use crate::storage::Value;
use anyhow::{ensure, Context, Result};
use libc::{
    c_char, c_int, pid_t, posix_spawn_file_actions_init, posix_spawn_file_actions_t,
    posix_spawnattr_init, posix_spawnattr_t, posix_spawnp, rusage, timeval, wait4,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::ffi::CString;
use std::mem::MaybeUninit;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub user_time: Duration,
    pub system_time: Duration,
    pub real_time: Duration,
    pub status_code: i64,
}

impl From<ExecutionResult> for HashMap<String, Value> {
    fn from(result: ExecutionResult) -> Self {
        let mut map = Self::new();
        map.insert("user_time".to_string(), Value::Duration(result.user_time));
        map.insert(
            "system_time".to_string(),
            Value::Duration(result.system_time),
        );
        map.insert("real_time".to_string(), Value::Duration(result.real_time));
        map.insert(
            "status_code".to_string(),
            Value::Integer(result.status_code),
        );

        map
    }
}

impl TryFrom<HashMap<String, Value>> for ExecutionResult {
    type Error = ();

    fn try_from(map: HashMap<String, Value>) -> std::result::Result<Self, Self::Error> {
        let v = map.get("user_time").ok_or(())?;

        let user_time = match v {
            Value::Duration(dur) => dur,
            _ => return Err(()),
        };

        let v = map.get("system_time").ok_or(())?;
        let system_time = match v {
            Value::Duration(dur) => dur,
            _ => return Err(()),
        };

        let v = map.get("real_time").ok_or(())?;
        let real_time = match v {
            Value::Duration(dur) => dur,
            _ => return Err(()),
        };

        let v = map.get("status").ok_or(())?;
        let status_code = match v {
            Value::Integer(int) => int,
            _ => return Err(()),
        };

        Ok(Self {
            user_time: *user_time,
            system_time: *system_time,
            real_time: *real_time,
            status_code: *status_code,
        })
    }
}

pub fn benchmark(command_and_flags: &[String]) -> Result<()> {
    let result = execute_and_measure(command_and_flags).context("failed to execute command")?;

    println!("Running \"{}\" took:", command_and_flags.join(" "),);
    println!(
        "{:?} user {:?} system {:?} real",
        result.user_time, result.system_time, result.real_time
    );

    // TODO: save result if execution was not successfull? (status != 0)
    append_benchmark(command_and_flags, &result).context("unable to save new benchmark")
}

pub fn execute_and_measure(command_and_flags: &[String]) -> Result<ExecutionResult> {
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

    unsafe {
        ensure!(posix_spawn_file_actions_init(file_actions.as_mut_ptr()) == 0);
        ensure!(posix_spawnattr_init(spawnattr.as_mut_ptr()) == 0);

        let now = Instant::now();

        let result = posix_spawnp(
            pid.as_mut_ptr(),
            program.as_ptr(),
            file_actions.as_mut_ptr(),
            spawnattr.as_mut_ptr(),
            argv.as_ptr(),
            envp.as_ptr(),
        );

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

        Ok(ExecutionResult {
            user_time,
            system_time,
            real_time,
            status_code: status.assume_init().into(),
        })
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
