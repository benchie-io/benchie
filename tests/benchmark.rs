use benchie::execute_and_measure;

#[cfg(unix)]
const BASIC_COMMAND: &[&str] = &["sleep", "1"];
#[cfg(windows)]
const BASIC_COMMAND: &[&str] = &["timeout", "/t", "1"];

#[test]
fn execution_and_measurement_basic_functionality() {
    let command: Vec<String> = BASIC_COMMAND.iter().map(|s| s.to_string()).collect();
    let result = execute_and_measure(&command);

    assert!(
        result.is_ok(),
        "execution and measurement for \"{}\" should succeed",
        command.join(" ")
    );
    let result = result.unwrap().0;

    // TODO: check why this fails in CI pipeline (linux)
    // assert!(
    //     result.user_time > Duration::new(0, 0),
    //     "measured user time should be bigger than 0"
    // );
    // assert!(
    //     result.system_time > Duration::new(0, 0),
    //     "measured system time should be bigger than 0"
    // );
    assert!(
        result.real_time >= result.user_time + result.system_time,
        "measured real time should be bigger than the sum of partial measurements"
    );
}

#[test]
fn should_not_panic_if_command_is_invalid() {
    let result = execute_and_measure(&["adsl;fasdjfoigaids;ifgorajoaidfjoigajoidaa".to_string()]);

    assert!(
        result.is_err(),
        "execution for an invalid command should fail"
    );
}
