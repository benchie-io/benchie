use crate::benchmark::{Benchmark, BenchmarkRaw};
use crate::read_git_info;
use anyhow::{Context, Result};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Data<T> {
    schema: u64,
    benchmarks: Vec<T>,
}

/// loads benchmarks from the file in order from oldest to newest.
pub fn load_all_benchmarks() -> Result<Vec<BenchmarkRaw>> {
    read_from_storage::<Data<BenchmarkRaw>>().map(|d| d.benchmarks)
}

pub fn append_benchmark(benchmark: &Benchmark) -> Result<()> {
    let mut data = read_from_storage::<Data<Benchmark>>()?;

    data.benchmarks.push(benchmark.clone());

    write_to_storage(&data)?;

    Ok(())
}

fn data_dir_path() -> PathBuf {
    const PATH: &str = ".benchie";

    if let Ok(git) = read_git_info() {
        git.path.join(PATH)
    } else {
        Path::new(PATH).into()
    }
}

fn data_file_path() -> PathBuf {
    data_dir_path().join("data.json")
}

fn read_from_storage<T>() -> Result<T>
where
    T: DeserializeOwned,
{
    let default = serde_json::to_string(&json! {
       {
           "schema": 1,
           "benchmarks": []
       }
    })?;

    let raw = fs::read_to_string(data_file_path()).unwrap_or(default);

    serde_json::from_str::<T>(&raw).context("failed to parse benchie data file")
}

fn write_to_storage<T: Serialize>(data: &Data<T>) -> Result<()> {
    // serialize benchmark to a JSON string
    let json = serde_json::to_string(data)?;

    let dir_path = data_dir_path();
    if !dir_path.exists() {
        fs::create_dir(&dir_path)?;
    }
    fs::write(data_file_path(), json)?;

    Ok(())
}
