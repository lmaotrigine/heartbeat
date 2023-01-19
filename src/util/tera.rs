use std::collections::HashMap;

use super::hf_time::{Accuracy, HumanTime, Tense};
use chrono::Duration;
use rocket_dyn_templates::tera::{Error, Result, Value};

pub fn format_relative(value: &Value, _: &HashMap<String, Value>) -> Result<Value> {
    let input = match value {
        Value::Number(n) => Ok(n.as_i64().unwrap()),
        _ => Err(Error::msg("Expected a number")),
    }?;
    let dur = Duration::seconds(input);
    let res = HumanTime::from(dur).to_text(Accuracy::Precise, Tense::Present);
    Ok(Value::from(res))
}

pub fn format_num(value: &Value, _: &HashMap<String, Value>) -> Result<Value> {
    let input = match value {
        Value::Number(n) => Ok(n.as_i64().unwrap()),
        _ => Err(Error::msg("Expected a number")),
    }?;
    let res = input
        .to_string()
        .as_bytes()
        .rchunks(3)
        .rev()
        .map(std::str::from_utf8)
        .collect::<std::result::Result<Vec<&str>, _>>()
        .unwrap()
        .join(",");
    Ok(Value::from(res))
}
