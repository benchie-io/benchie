use crate::benchmark::parse_tags_from_stdout;
use crate::ExecutionResult;
use anyhow::{ensure, Context, Result};
use libc::{
    c_char, c_int, close, pid_t, pipe, posix_spawn_file_actions_addclose,
    posix_spawn_file_actions_adddup2, posix_spawn_file_actions_init, posix_spawn_file_actions_t,
    posix_spawnattr_init, posix_spawnattr_t, posix_spawnp, rusage, timeval, wait4,
};
use std::collections::HashMap;
use std::ffi::CString;
use std::mem::MaybeUninit;
use std::time::{Duration, Instant};
use std::{env, fs::File, io::Read, os::unix::io::FromRawFd};

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
