//! Soft memory reservations alongside the CPU/link broker. Reservations are
//! estimates, not kernel limits: running commands are never killed or suspended.
use super::{memory, non_negative, number, positive, profiles};
use std::{ffi::OsString, fs, io, path::Path};

pub const MIB: usize = 1024;
pub const GIB: usize = 1024 * MIB;
#[derive(Clone, Copy)]
pub struct Policy {
    pub budget: usize,
    pub headroom: usize,
}
impl Policy {
    pub fn read() -> io::Result<Self> {
        let snapshot = memory::snapshot()?;
        let headroom = non_negative(
            "VIBEOS_HOST_MEMORY_HEADROOM_MIB",
            (snapshot.total_kib / 8).min(8 * GIB) / MIB,
        )?
        .checked_mul(MIB)
        .ok_or_else(|| io::Error::other("memory headroom overflow"))?;
        let default = snapshot.total_kib.saturating_sub(headroom) / MIB;
        let budget = positive("VIBEOS_HOST_MEMORY_MIB", default)?
            .checked_mul(MIB)
            .ok_or_else(|| io::Error::other("memory budget overflow"))?;
        if budget == 0 || headroom >= snapshot.total_kib {
            return Err(io::Error::other(
                "memory headroom leaves no usable capacity",
            ));
        }
        Ok(Self {
            budget: budget.min(snapshot.total_kib - headroom),
            headroom,
        })
    }
}
pub fn estimate(kind: &str, args: &[OsString], root: &Path, key: &str) -> io::Result<usize> {
    let rust = args
        .iter()
        .any(|a| a == "--crate-name" || a == "tools.artifact_executor")
        || args
            .iter()
            .any(|a| Path::new(a).file_stem().is_some_and(|s| s == "rustc"));
    let floor = match kind {
        "lint" => 6 * GIB,
        _ if rust => 4 * GIB,
        "link" => 2 * GIB,
        _ => 512 * MIB,
    };
    let explicit = std::env::var_os("VIBEOS_HOST_JOB_MEMORY_MIB").is_some();
    let floor = positive("VIBEOS_HOST_JOB_MEMORY_MIB", floor / MIB)?
        .checked_mul(MIB)
        .ok_or_else(|| io::Error::other("memory estimate overflow"))?;
    // An explicit estimate is useful for known workloads and process fixtures.
    // Automatically learned peaks can only raise the conservative floor.
    if explicit {
        return Ok(floor);
    }
    Ok(
        profiles::load_peak(&root.join("memory-profiles"), key)?.map_or(floor, |peak| {
            floor.max(peak.saturating_add(peak / 4).saturating_add(64 * MIB))
        }),
    )
}
pub fn requested(body: &str) -> usize {
    number(body, "memory_kib").unwrap_or(4 * GIB) // conservative old-broker ticket
}
pub fn live(root: &Path, path: &Path) -> (usize, usize) {
    let Ok(text) = fs::read_to_string(root.join("memory-live").join(path.file_name().unwrap()))
    else {
        return (0, 0);
    };
    let peak = number(&text, "peak_kib").unwrap_or(0);
    if super::now().saturating_sub(number(&text, "sample_ms").unwrap_or(0) as u64) > 2000 {
        return (0, peak);
    }
    (number(&text, "rss_kib").unwrap_or(0), peak)
}
pub fn fits(
    request: usize,
    reserved: usize,
    unconsumed: usize,
    policy: Policy,
    available: usize,
) -> bool {
    request <= policy.budget.saturating_sub(reserved)
        && request
            <= available
                .saturating_sub(policy.headroom)
                .saturating_sub(unconsumed)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn charges_unconsumed_reservations_and_preserves_headroom() {
        let p = Policy {
            budget: 100,
            headroom: 10,
        };
        assert!(fits(30, 70, 20, p, 60));
        assert!(!fits(31, 70, 20, p, 100));
        assert!(!fits(30, 70, 21, p, 60));
        assert!(!fits(1, 0, 0, p, 9));
    }
}
