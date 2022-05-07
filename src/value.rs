use crate::Benchmark;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;
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

pub struct Values(pub Vec<Value>);

impl Values {
    pub fn push(&mut self, v: Value) {
        self.0.push(v);
    }
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

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Timestamp(v) => write!(f, "{}", v),
            Value::Duration(v) => write!(f, "{}", format_args!("{:?}", v)),
            Value::String(v) => write!(f, "{}", v),
            Value::Float(v) => write!(f, "{}", v),
            Value::Integer(v) => write!(f, "{}", v),
            Value::Bool(v) => write!(f, "{}", v),
        }
    }
}

impl fmt::Display for Values {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut out = String::new();
        if self.0.len() > 1 {
            out.push('{');

            for val in &self.0[0..self.0.len() - 1] {
                out.push_str(&format!("{}", val));
                out.push_str(", ");
            }

            out.push_str(&self.0[self.0.len() - 1].to_string());
            out.push('}');
        } else {
            out.push_str(&self.0[self.0.len() - 1].to_string());
        }

        write!(f, "{}", out)
    }
}
