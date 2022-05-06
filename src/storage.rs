use crate::benchmark::ExecutionResult;
use crate::git::GitInfo;
use anyhow::{Context, Result};
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Data {
    benchmarks: Vec<Benchmark>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "value")]
pub enum Value {
    Timestamp(DateTime<Utc>),
    Duration(Duration),
    String(String),
    Float(f64),
    Integer(i64),
    Bool(bool),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Timestamp(lhs), Value::Timestamp(rhs)) => lhs == rhs,
            (Value::Duration(lhs), Value::Duration(rhs)) => lhs == rhs,
            (Value::String(lhs), Value::String(rhs)) => lhs == rhs,
            (Value::Float(lhs), Value::Float(rhs)) => lhs == rhs,
            (Value::Integer(lhs), Value::Integer(rhs)) => lhs == rhs,
            (Value::Bool(lhs), Value::Bool(rhs)) => lhs == rhs,
            _ => false,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Benchmark {
    pub data: HashMap<String, Value>,
}

impl Benchmark {
    pub fn new(command: &[String], result: &ExecutionResult, git: &Option<GitInfo>) -> Self {
        let mut data: HashMap<String, Value> = result.clone().into();
        if let Some(git) = git {
            data.extend(git.into_iter());
        }
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

pub fn append_benchmark(benchmark: &Benchmark) -> Result<()> {
    let mut data = read_from_storage()?;

    data.benchmarks.push(benchmark.clone());

    write_to_storage(&data)?;

    Ok(())
}
