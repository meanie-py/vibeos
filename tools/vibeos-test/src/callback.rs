//! Callback transport for CLI presentation and explicitly owned external tools.
use crate::{Error, Result};
use serde_json::{json, Value};
use std::io::{BufRead, Write};

pub trait Host {
    fn call(&mut self, operation: &str, arguments: Value) -> Result<Value>;
}

pub struct Callback<'a> {
    pub input: &'a mut dyn BufRead,
    pub output: &'a mut dyn Write,
}

impl Host for Callback<'_> {
    fn call(&mut self, operation: &str, arguments: Value) -> Result<Value> {
        serde_json::to_writer(
            &mut *self.output,
            &json!({"callback": {
                "kind": operation, "arguments": arguments
            }}),
        )?;
        self.output.write_all(b"\n")?;
        self.output.flush()?;
        let mut line = String::new();
        if self.input.read_line(&mut line)? == 0 {
            return Err("test callback owner closed its input".into());
        }
        let response: Value = serde_json::from_str(&line)?;
        let map = response
            .as_object()
            .ok_or("invalid test callback response")?;
        if map.len() != 1 {
            return Err("test callback requires exactly one result or error".into());
        }
        if let Some(value) = map.get("callback_result") {
            return Ok(value.clone());
        }
        if let Some(error) = map.get("callback_error") {
            return Err(Error::new(format!(
                "test callback {operation} failed: {}",
                error["message"].as_str().ok_or("invalid callback error")?
            )));
        }
        Err("unknown test callback response".into())
    }
}
