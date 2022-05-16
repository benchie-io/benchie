use anyhow::anyhow;
use bytesize::ByteSize;
use chrono::prelude::*;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::fmt::Debug;
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "type", content = "value")]
pub enum Value {
    Bool(bool),
    Integer(i64),
    Float(f64),
    String(String),
    ByteSize(ByteSize),
    Duration(Duration),
    Timestamp(DateTime<Utc>),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Bool(v) => write!(f, "{}", v),
            Value::Integer(v) => write!(f, "{}", v),
            Value::Float(v) => write!(f, "{}", v),
            Value::String(v) => write!(f, "{}", v),
            Value::ByteSize(v) => write!(f, "{}", v),
            Value::Duration(v) => write!(f, "{}", format_args!("{:?}", v)),
            Value::Timestamp(v) => write!(f, "{}", v),
        }
    }
}

pub struct Values(pub Vec<Value>);

impl Values {
    pub fn push(&mut self, v: Value) {
        self.0.push(v);
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

pub fn serialize<'a, T, S>(v: &'a T, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    &'a T: Into<Value>,
{
    let value: Value = v.into();

    value.serialize(s)
}

pub fn deserialize<'de, D, T>(d: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    Value: TryInto<T>,
    <Value as TryInto<T>>::Error: Debug,
{
    let s: Value = Deserialize::deserialize(d)?;

    Ok(s.try_into().unwrap())
}

impl From<&Duration> for Value {
    fn from(v: &Duration) -> Self {
        Value::Duration(*v)
    }
}

impl From<&String> for Value {
    fn from(v: &String) -> Self {
        Value::String(v.clone())
    }
}

impl From<&f64> for Value {
    fn from(v: &f64) -> Self {
        Value::Float(*v)
    }
}

impl From<&i64> for Value {
    fn from(v: &i64) -> Self {
        Value::Integer(*v)
    }
}

impl From<&bool> for Value {
    fn from(v: &bool) -> Self {
        Value::Bool(*v)
    }
}

impl From<&ByteSize> for Value {
    fn from(v: &ByteSize) -> Self {
        Value::ByteSize(*v)
    }
}

impl From<&DateTime<Utc>> for Value {
    fn from(v: &DateTime<Utc>) -> Self {
        Value::Timestamp(*v)
    }
}

const OPTION_SERIALIZATION_ERROR: &str = "trying to serialize an optional value with none is not allowed => try adding \"skip_serializing_if\"";

impl From<&Option<bool>> for Value {
    fn from(v: &Option<bool>) -> Self {
        v.as_ref().expect(OPTION_SERIALIZATION_ERROR).into()
    }
}

impl From<&Option<i64>> for Value {
    fn from(v: &Option<i64>) -> Self {
        v.as_ref().expect(OPTION_SERIALIZATION_ERROR).into()
    }
}

impl From<&Option<f64>> for Value {
    fn from(v: &Option<f64>) -> Self {
        v.as_ref().expect(OPTION_SERIALIZATION_ERROR).into()
    }
}

impl From<&Option<String>> for Value {
    fn from(v: &Option<String>) -> Self {
        v.as_ref().expect(OPTION_SERIALIZATION_ERROR).into()
    }
}

impl From<&Option<ByteSize>> for Value {
    fn from(v: &Option<ByteSize>) -> Self {
        v.as_ref().expect(OPTION_SERIALIZATION_ERROR).into()
    }
}

impl From<&Option<Duration>> for Value {
    fn from(v: &Option<Duration>) -> Self {
        v.as_ref().expect(OPTION_SERIALIZATION_ERROR).into()
    }
}

impl From<&Option<DateTime<Utc>>> for Value {
    fn from(v: &Option<DateTime<Utc>>) -> Self {
        v.as_ref().expect(OPTION_SERIALIZATION_ERROR).into()
    }
}

impl TryInto<DateTime<Utc>> for Value {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<DateTime<Utc>, Self::Error> {
        match self {
            Value::Timestamp(v) => Ok(v),
            _ => Err(anyhow!("failed to parse {:?} into a DateTime<Utc>", self)),
        }
    }
}

impl TryInto<String> for Value {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<String, Self::Error> {
        match self {
            Value::String(v) => Ok(v),
            _ => Err(anyhow!("failed to parse {:?} into a String", self)),
        }
    }
}

impl TryInto<ByteSize> for Value {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<ByteSize, Self::Error> {
        match self {
            Value::ByteSize(v) => Ok(v),
            _ => Err(anyhow!("failed to parse {:?} into a ByteSize", self)),
        }
    }
}

impl TryInto<i64> for Value {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<i64, Self::Error> {
        match self {
            Value::Integer(v) => Ok(v),
            _ => Err(anyhow!("failed to parse {:?} into a i64", self)),
        }
    }
}

impl TryInto<bool> for Value {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<bool, Self::Error> {
        match self {
            Value::Bool(v) => Ok(v),
            _ => Err(anyhow!("failed to parse {:?} into a bool", self)),
        }
    }
}

impl TryInto<Duration> for Value {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Duration, Self::Error> {
        match self {
            Value::Duration(v) => Ok(v),
            _ => Err(anyhow!("failed to parse {:?} into a Duration", self)),
        }
    }
}

impl TryInto<Option<String>> for Value {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Option<String>, Self::Error> {
        match self {
            Value::String(v) => Ok(Some(v)),
            _ => Err(anyhow!("failed to parse {:?} into a Option<String>", self)),
        }
    }
}
