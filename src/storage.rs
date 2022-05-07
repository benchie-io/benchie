use crate::benchmark::ExecutionResult;
use crate::git::GitInfo;
use crate::Value;
use anyhow::{Context, Result};
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Data {
    benchmarks: Vec<Benchmark>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Benchmark {
    pub data: HashMap<String, Value>,
}

impl Benchmark {
    pub fn new(
        command: &[String],
        result: &ExecutionResult,
        git: &Option<GitInfo>,
        tags: &HashMap<String, String>,
    ) -> Self {
        let mut data: HashMap<String, Value> = result.clone().into();
        if let Some(git) = git {
            data.extend(git.into_iter());
        }
        data.insert("command".to_string(), Value::String(command.join(" ")));
        data.insert("created_at".to_string(), Value::Timestamp(Utc::now()));

        tags.iter().for_each(|(key, value)| {
            if data.get(key.as_str()).is_some() {
                println!("warning: overwriting key \"{key}\" with user provided tag");
                data.remove(key);
            }
            data.insert(key.clone(), Value::String(value.clone()));
        });

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
