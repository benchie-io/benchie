use anyhow::Result;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::time::Duration;

#[derive(Serialize, Deserialize)]
struct Benchmark {
    runtime: Duration,
    timestamp: DateTime<Utc>,
    command: Vec<String>,
}

pub fn write_to_storage(command: &[String], runtime: &Duration) -> Result<()> {
    let path = ".benchie";

    // serialize benchmark to a JSON string
    let j = serde_json::to_string(&Benchmark {
        runtime: *runtime,
        timestamp: Utc::now(),
        command: command.to_vec(),
    })?;

    if !Path::new(path).exists() {
        fs::create_dir(path)?;
    }
    fs::write(format!("{}/data.json", path), j)?;

    Ok(())
}
