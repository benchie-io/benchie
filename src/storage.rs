use crate::benchmark::ExecutionResult;
use anyhow::{Context, Result};
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug)]
struct Data {
    benchmarks: Vec<Benchmark>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "value")]
pub enum Value {
    Timestamp(DateTime<Utc>),
    Duration(Duration),
    String(String),
    Float(f64),
    Integer(i64),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Benchmark {
    pub data: HashMap<String, Value>,
}

impl Benchmark {
    pub fn new(command: &[String], result: &ExecutionResult) -> Self {
        let mut data: HashMap<String, Value> = result.clone().into();
        data.insert("command".to_string(), Value::String(command.join(" ")));
        data.insert("created_at".to_string(), Value::Timestamp(Utc::now()));

        Self { data }
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

    let result = serde_json::from_str(&raw).context("failed to parse benchie data file");

    dbg!(&result);

    result
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

pub fn append_benchmark(command: &[String], result: &ExecutionResult) -> Result<()> {
    let mut data = read_from_storage()?;

    data.benchmarks.push(Benchmark::new(command, result));

    write_to_storage(&data)?;

    Ok(())
}
