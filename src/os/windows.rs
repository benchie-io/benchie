use crate::ExecutionResult;
use anyhow::{ensure, Result};
use libc::c_void;
use std::collections::HashMap;
use std::mem::MaybeUninit;
use std::ptr::null;
use std::time::Duration;
use windows::core::{PCWSTR, PWSTR};
use windows::Win32::Foundation::{GetLastError, BOOL, FILETIME, HANDLE};
use windows::Win32::Security::SECURITY_ATTRIBUTES;
use windows::Win32::System::Threading::{
    CreateProcessW, GetExitCodeProcess, GetProcessTimes, WaitForSingleObject,
    PROCESS_CREATION_FLAGS, PROCESS_INFORMATION, STARTUPINFOW,
};

pub fn execute_and_measure(
    command_and_flags: &[String],
) -> Result<(ExecutionResult, HashMap<String, String>)> {
    ensure!(
        !command_and_flags.is_empty(),
        "command can not be empty for benchmarking"
    );

    let mut program_and_flags: Vec<u16> = dbg!(command_and_flags)
        .join(" ")
        .encode_utf16()
        .chain(Some(0)) // null-terminate
        .collect();
    let mut process_info = PROCESS_INFORMATION::default();
    let startup_info = STARTUPINFOW {
        cb: std::mem::size_of::<STARTUPINFOW>() as u32,
        ..STARTUPINFOW::default()
    };
    let process_creation_flags = PROCESS_CREATION_FLAGS(0);
    let command_line = PWSTR(program_and_flags.as_mut_ptr());
    let mut status_code = 0_u32;

    unsafe {
        // Start the child process.
        let result = CreateProcessW(
            PCWSTR::default(), // No module name (use command line)
            command_line,
            null() as *const SECURITY_ATTRIBUTES, // Process handle not inheritable
            null() as *const SECURITY_ATTRIBUTES, // Thread handle not inheritable
            BOOL(0),                              // Set handle inheritance to FALSE
            process_creation_flags,               // No creation flags
            null() as *const c_void,              // Use parent's environment block
            PCWSTR::default(),                    // Use parent's starting directory
            &startup_info,                        // Pointer to STARTUPINFO structure
            &mut process_info,                    // Pointer to PROCESS_INFORMATION structure
        );
        ensure!(
            result.as_bool(),
            "spawning of process failed for execution of command with: {:?}",
            GetLastError().ok()
        );

        // Wait until child process exits.
        const INFINITE: u32 = 0xFFFFFFFF;
        let _time = WaitForSingleObject(process_info.hProcess, INFINITE);

        ensure!(
            GetExitCodeProcess(process_info.hProcess, &mut status_code).as_bool(),
            "waiting for process execution end failed with: {:?}",
            GetLastError().ok()
        );
    }

    let (user_time, system_time, real_time) = get_execution_times(process_info.hProcess);

    Ok((
        ExecutionResult {
            user_time,
            system_time,
            real_time,
            status_code: status_code.into(),
        },
        HashMap::new(),
    ))
}

fn get_execution_times(process_handle: HANDLE) -> (Duration, Duration, Duration) {
    let mut creation_time = MaybeUninit::<FILETIME>::uninit();
    let mut exit_time = MaybeUninit::<FILETIME>::uninit();
    let mut kernel_time = MaybeUninit::<FILETIME>::uninit();
    let mut user_time = MaybeUninit::<FILETIME>::uninit();

    unsafe {
        GetProcessTimes(
            process_handle,
            creation_time.as_mut_ptr(),
            exit_time.as_mut_ptr(),
            kernel_time.as_mut_ptr(),
            user_time.as_mut_ptr(),
        );

        (
            filetime_to_duration(user_time.assume_init()),
            filetime_to_duration(kernel_time.assume_init()),
            filetime_to_duration(exit_time.assume_init())
                - filetime_to_duration(creation_time.assume_init()),
        )
    }
}

fn filetime_to_duration(value: FILETIME) -> Duration {
    let nanoseconds =
        (((value.dwHighDateTime as u128) << 32) | (value.dwLowDateTime as u128)) * 100;

    let seconds = nanoseconds / 1_000_000_000;
    let nanoseconds = nanoseconds - seconds * 1_000_000_000;

    Duration::new(
        seconds
            .try_into()
            .expect("has to fit into u128 because of previous arithmetics"),
        nanoseconds
            .try_into()
            .expect("has to fit into u32 because of previous arithmetics"),
    )
}
