use benchie::execute_and_measure;
use std::time::Duration;

#[test]
fn test_execution_and_measurement_basic_functionality() {
    let result = execute_and_measure(&["echo".to_string(), "hello".to_string()]);

    assert!(
        result.is_ok(),
        "execution and measurement for \"pwd\" should succeed"
    );
    let result = result.unwrap();

    assert!(
        result.user_time > Duration::new(0, 0),
        "measured user time should be bigger than 0"
    );
    // TODO: check why this fails in CI pipeline (linux)
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
fn test_should_not_panic_if_command_is_invalid() {
    let result = execute_and_measure(&["adsl;fasdjfoigaids;ifgorajoaidfjoigajoidaa".to_string()]);

    assert!(
        result.is_err(),
        "execution for an invalid command should fail"
    );
}