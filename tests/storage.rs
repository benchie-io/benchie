mod common;

use crate::common::with_temp_dir;
use benchie::{append_benchmark, load_all_benchmarks, Benchmark};
use serial_test::serial;
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

        let result = append_benchmark(
            &["sleep".to_string(), "1".to_string()],
            &Duration::from_secs(1),
        );

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
        let data_file_path = benchie_dir.clone().join("data.json");

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

        let result = append_benchmark(
            &["sleep".to_string(), "1".to_string()],
            &Duration::from_secs(1),
        );

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
        let data_file_path = benchie_dir.clone().join("data.json");

        let _ = create_dir(&benchie_dir);

        let benchmark = Benchmark::new(Duration::from_secs(42), vec![String::from("pwd")]);
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

        let result = append_benchmark(
            &["sleep".to_string(), "1".to_string()],
            &Duration::from_secs(1),
        );
        assert!(result.is_ok(), "should succeed to append a benchmark");

        let result = load_all_benchmarks();

        assert!(
            result.is_ok(),
            "should successfully load multiple benchmarks"
        );
        assert_eq!(result.unwrap().len(), 2, "should have added a benchmark");
    })
}
