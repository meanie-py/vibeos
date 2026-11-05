//! Existing Windows launch-bind retries, separate from guest boot retry decisions.
use crate::{Error, Result};
use std::collections::BTreeMap;
pub fn retryable_stderr(stderr: &str) -> bool {
    [
        "Could not set up host forwarding rule",
        "Could not open gdbserver",
        "Failed to connect to '",
    ]
    .iter()
    .any(|pattern| stderr.contains(pattern))
}
pub fn attempts(backend: &str, env: &BTreeMap<String, String>) -> Result<usize> {
    let count = if backend == "windows-host" {
        env.get("VIBEOS_HOSTFWD_LAUNCH_RETRIES")
            .map(|value| {
                value
                    .parse::<usize>()
                    .map_err(|_| Error::new("invalid launch retry count"))
            })
            .transpose()?
            .unwrap_or(4)
    } else {
        1
    };
    if count == 0 {
        return Err("launch retry count must be positive".into());
    }
    Ok(count)
}
pub fn should_retry(backend: &str, attempt: usize, attempts: usize, stderr: &str) -> bool {
    backend == "windows-host" && attempt.saturating_add(1) < attempts && retryable_stderr(stderr)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn retries_each_known_bind_race_but_never_bad_guest_configuration() {
        for reason in [
            "Could not set up host forwarding rule 'tcp:0.0.0.0:42000-:80'",
            "Could not open gdbserver on tcp:0.0.0.0:42000",
            "Failed to connect to '127.0.0.1:42000': connection refused",
        ] {
            assert!(should_retry("windows-host", 0, 4, reason));
            assert!(should_retry("windows-host", 2, 4, reason));
            assert!(
                !should_retry("windows-host", 3, 4, reason),
                "last failure must be surfaced"
            );
            assert!(
                !should_retry("native", 0, 4, reason),
                "native launch retry belongs to outer controller"
            );
        }
        for reason in [
            "Could not open '/missing/disk.qcow2': No such file or directory",
            "invalid accelerator whpx",
            "fatal: bad machine property",
            "",
        ] {
            assert!(!should_retry("windows-host", 0, 4, reason));
        }
    }
    #[test]
    fn default_and_configured_attempt_limits_are_exact_and_fail_loud() {
        assert_eq!(attempts("windows-host", &BTreeMap::new()).unwrap(), 4);
        assert_eq!(attempts("native", &BTreeMap::new()).unwrap(), 1);
        let env = BTreeMap::from([("VIBEOS_HOSTFWD_LAUNCH_RETRIES".into(), "2".into())]);
        assert_eq!(attempts("windows-host", &env).unwrap(), 2);
        for raw in ["0", "-1", "invalid"] {
            assert!(attempts(
                "windows-host",
                &BTreeMap::from([("VIBEOS_HOSTFWD_LAUNCH_RETRIES".into(), raw.into())])
            )
            .is_err());
        }
    }
    #[test]
    fn exhaustion_returns_control_after_exactly_the_permitted_attempts() {
        let count = attempts("windows-host", &BTreeMap::new()).unwrap();
        let stderr = "Could not open gdbserver";
        assert_eq!(
            (0..count)
                .filter(|&attempt| should_retry("windows-host", attempt, count, stderr))
                .count(),
            count - 1
        );
        assert!(!should_retry("windows-host", usize::MAX, count, stderr));
    }
}
