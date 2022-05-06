mod common;

use crate::common::with_temp_dir;
use benchie::{append_benchmark, load_all_benchmarks, Benchmark, ExecutionResult, GitInfo, Value};
use serial_test::serial;
use std::collections::HashMap;
use std::fs;
use std::fs::create_dir;
use std::time::Duration;

#[test]
#[serial]
fn test_with_missing_dir() {
    with_temp_dir(|temp_dir| {
        let data_file_path = temp_dir.path().join(".benchie/data.json");
        assert!(!data_file_path.exists());

        let result = load_all_benchmarks();

        assert!(
            result.is_ok(),
            "should not fail to read data with missing benchie directory"
        );
        assert!(
            !data_file_path.exists(),
            "should not have created a file, while reading"
        );

        let result = append_benchmark(&create_benchmark());

        assert!(result.is_ok(), "should not fail with empty dir");
        assert!(
            data_file_path.exists(),
            "should have created a file as a result"
        );
    })
}

#[test]
#[serial]
fn test_with_existing_dir_but_missing_data() {
    with_temp_dir(|temp_dir| {
        let benchie_dir = temp_dir.path().join(".benchie");
        let data_file_path = benchie_dir.join("data.json");

        let _ = create_dir(&benchie_dir);

        assert!(benchie_dir.exists());
        assert!(!data_file_path.exists());

        let result = load_all_benchmarks();

        assert!(
            result.is_ok(),
            "should not fail to read data with empty dir"
        );
        assert!(
            !data_file_path.exists(),
            "should not have created a file as a result"
        );

        let result = append_benchmark(&create_benchmark());

        assert!(result.is_ok(), "should not fail with empty dir");
        assert!(
            data_file_path.exists(),
            "should have created a file as a result"
        );
    })
}

#[test]
#[serial]
fn test_with_existing_dir_and_data() {
    with_temp_dir(|temp_dir| {
        let benchie_dir = temp_dir.path().join(".benchie");
        let data_file_path = benchie_dir.join("data.json");

        let _ = create_dir(&benchie_dir);

        let benchmark = create_benchmark();
        let data = format!(
            "{{ \"benchmarks\": [{}]}}",
            serde_json::to_string(&benchmark).unwrap()
        );

        let _ = fs::write(&data_file_path, data);

        let result = load_all_benchmarks();

        assert!(
            result.is_ok(),
            "should not fail to read data with missing data file"
        );
        assert_eq!(
            result.unwrap().len(),
            1,
            "should be able to load pre saved benchmark"
        );

        let result = append_benchmark(&create_benchmark());
        assert!(result.is_ok(), "should succeed to append a benchmark");

        let result = load_all_benchmarks();

        assert!(
            result.is_ok(),
            "should successfully load multiple benchmarks"
        );
        assert_eq!(result.unwrap().len(), 2, "should have added a benchmark");
    })
}

#[test]
#[serial]
fn should_save_tags_in_benchmark() {
    with_temp_dir(|_| {
        append_benchmark(&create_benchmark()).expect("should succeed to append a benchmark");

        let benchmarks = load_all_benchmarks().expect("should successfully load benchmarks");

        let benchmark = benchmarks.get(0).expect("should have loaded one benchmark");

        assert_eq!(
            benchmark.data.get("key"),
            Some(&Value::String(String::from("value"))),
            "should have added a key=value pair"
        );
    })
}

fn create_execution_result() -> ExecutionResult {
    ExecutionResult {
        real_time: Duration::from_secs(1),
        ..Default::default()
    }
}

fn create_benchmark() -> Benchmark {
    let result = create_execution_result();

    let info = GitInfo {
        commit_id: "adfadsfasd".to_string(),
        commit_message: "hello commit".to_string(),
        branch: "master".to_string(),
        is_dirty: false,
    };

    let mut tags = HashMap::new();
    tags.insert(String::from("key"), String::from("value"));

    Benchmark::new(
        &["ls".to_string(), "-la".to_string()],
        &result,
        &Some(info),
        &tags,
    )
}
