//! Bounded capture with both pipes drained while the child runs.
use crate::{Error, Result};
use std::{
    io::Read,
    process::{Command, Stdio},
    time::{Duration, Instant},
};
#[derive(Debug, serde::Serialize)]
pub struct Output {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}
pub fn capture(cmd: &mut Command, timeout_secs: u64) -> Result<Output> {
    let mut child = cmd.stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;
    let mut stdout = child.stdout.take().ok_or("missing subprocess stdout")?;
    let mut stderr = child.stderr.take().ok_or("missing subprocess stderr")?;
    let out = std::thread::spawn(move || {
        let mut b = Vec::new();
        stdout.read_to_end(&mut b).map(|_| b)
    });
    let err = std::thread::spawn(move || {
        let mut b = Vec::new();
        stderr.read_to_end(&mut b).map(|_| b)
    });
    let start = Instant::now();
    let (status, timed_out) = loop {
        if let Some(s) = child.try_wait()? {
            break (s, false);
        }
        if start.elapsed() >= Duration::from_secs(timeout_secs) {
            let _ = child.kill();
            break (child.wait()?, true);
        }
        std::thread::sleep(Duration::from_millis(20));
    };
    let stdout = out
        .join()
        .map_err(|_| Error::new("stdout reader failed"))??;
    let stderr = err
        .join()
        .map_err(|_| Error::new("stderr reader failed"))??;
    if timed_out {
        return Err(Error::new(format!(
            "subprocess timed out after {timeout_secs}s: {cmd:?}"
        )));
    }
    Ok(Output {
        code: status.code().unwrap_or(1),
        stdout: String::from_utf8_lossy(&stdout).into_owned(),
        stderr: String::from_utf8_lossy(&stderr).into_owned(),
    })
}
