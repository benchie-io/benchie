use anyhow::{Context, Result};
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;
use std::path::Path;
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug)]
struct Data {
    benchmarks: Vec<Benchmark>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Benchmark {
    runtime: Duration,
    timestamp: DateTime<Utc>,
    command: Vec<String>,
}

impl Benchmark {
    pub fn new(runtime: Duration, command: Vec<String>) -> Self {
        Self {
            runtime,
            timestamp: Utc::now(),
            command,
        }
    }
}

const PATH: &str = ".benchie";

fn read_from_storage() -> Result<Data> {
    let default = serde_json::to_string(&json! {
       {
           "benchmarks": []
       }
    })?;

    let raw = fs::read_to_string(format!("{}/data.json", PATH)).unwrap_or(default);

    serde_json::from_str(&raw).context("failed to parse benchie data file")
}

fn write_to_storage(data: &Data) -> Result<()> {
    // serialize benchmark to a JSON string
    let json = serde_json::to_string(data)?;

    if !Path::new(PATH).exists() {
        fs::create_dir(PATH)?;
    }
    fs::write(format!("{}/data.json", PATH), json)?;

    Ok(())
}

pub fn load_all_benchmarks() -> Result<Vec<Benchmark>> {
    read_from_storage().map(|d| d.benchmarks)
}

pub fn append_benchmark(command: &[String], runtime: &Duration) -> Result<()> {
    let mut data = read_from_storage()?;

    data.benchmarks.push(Benchmark {
        runtime: *runtime,
        timestamp: Utc::now(),
        command: command.to_vec(),
    });

    write_to_storage(&data)?;

    Ok(())
}
