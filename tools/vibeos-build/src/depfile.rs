//! Retarget only the emitted rule prefix; preserve its complete prerequisite body.
use crate::{Error, Result};
pub fn escaped_output(path: &str) -> Result<String> {
    if path.chars().any(|c| "\r\n\t\\:#$".contains(c)) {
        return Err(Error::new(format!(
            "unsupported depfile output spelling: {path:?}"
        )));
    }
    Ok(path.replace(' ', "\\ "))
}
pub fn retarget(text: &str, cargo_output: &str, copied_output: &str) -> Result<String> {
    let prefix = format!("{}:", escaped_output(cargo_output)?);
    let Some(body) = text
        .strip_prefix(&prefix)
        .filter(|s| s.starts_with([' ', '\n', '\r']))
    else {
        return Err("Cargo depfile does not start with the expected output rule".into());
    };
    Ok(format!("{}:{body}", escaped_output(copied_output)?))
}
