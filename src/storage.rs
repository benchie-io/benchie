use crate::benchmark::{Benchmark, BenchmarkRaw};
use anyhow::{Context, Result};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Data<T> {
    schema: u64,
    benchmarks: Vec<T>,
}

const PATH: &str = ".benchie";

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

    let raw = fs::read_to_string(format!("{}/data.json", PATH)).unwrap_or(default);

    serde_json::from_str::<T>(&raw).context("failed to parse benchie data file")
}

fn write_to_storage<T: Serialize>(data: &Data<T>) -> Result<()> {
    // serialize benchmark to a JSON string
    let json = serde_json::to_string(data)?;

    if !Path::new(PATH).exists() {
        fs::create_dir(PATH)?;
    }
    fs::write(format!("{}/data.json", PATH), json)?;

    Ok(())
}

pub fn load_all_benchmarks() -> Result<Vec<BenchmarkRaw>> {
    read_from_storage::<Data<BenchmarkRaw>>().map(|d| d.benchmarks)
}

pub fn append_benchmark(benchmark: &Benchmark) -> Result<()> {
    let mut data = read_from_storage::<Data<Benchmark>>()?;

    data.benchmarks.push(benchmark.clone());

    write_to_storage(&data)?;

    Ok(())
}
