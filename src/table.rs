use chrono::{DateTime, Utc};
use cli_table::Table;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::time::Duration;

#[derive(Table, Serialize, Deserialize, Debug)]
pub struct TableView {
    #[table(title = "Runtime in ms", display_fn = "display_runtime", order = 1)]
    runtime: Duration,
    #[table(title = "Timestamp", order = 2)]
    timestamp: DateTime<Utc>,
    #[table(title = "Command", display_fn = "display_command", order = 0)]
    command: Vec<String>,
}

fn display_runtime(value: &Duration) -> impl Display {
    value.as_secs_f32() * 1000.
}

fn display_command(value: &[String]) -> impl Display {
    value.join(" ")
}
