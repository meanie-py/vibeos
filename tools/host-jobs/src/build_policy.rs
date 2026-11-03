//! Shared capacity contract. Admission remains owned by the existing broker.
pub const VERSION: usize = 1;
pub const FIELDS: [&str; 5] = [
    "version",
    "leases",
    "max_builds",
    "ninja_jobs",
    "cargo_jobs",
];
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Capacity {
    pub version: usize,
    pub leases: usize,
    pub max_builds: usize,
    pub ninja_jobs: usize,
    pub cargo_jobs: usize,
}
pub fn positive(key: &str, value: &str) -> Result<usize, String> {
    value
        .parse::<usize>()
        .ok()
        .filter(|n| *n > 0)
        .ok_or_else(|| format!("{key} must be positive"))
}
impl Capacity {
    pub fn new(values: [usize; 5]) -> Result<Self, String> {
        let [version, leases, max_builds, ninja_jobs, cargo_jobs] = values;
        if values.contains(&0) || version != VERSION || leases % max_builds != 0 || cargo_jobs != 1
        {
            return Err("invalid host build policy".into());
        }
        max_builds
            .checked_mul(ninja_jobs)
            .ok_or("host build job budget overflows")?;
        Ok(Self {
            version,
            leases,
            max_builds,
            ninja_jobs,
            cargo_jobs,
        })
    }
    pub fn trial(rollback: bool) -> Self {
        Self::new(if rollback {
            [VERSION, 4, 2, 6, 1]
        } else {
            [VERSION, 6, 3, 6, 1]
        })
        .unwrap()
    }
    pub fn slots(self) -> usize {
        self.max_builds * self.ninja_jobs
    }
    pub fn build_leases(self) -> usize {
        self.leases / self.max_builds
    }
    pub fn values(self) -> [usize; 5] {
        [
            self.version,
            self.leases,
            self.max_builds,
            self.ninja_jobs,
            self.cargo_jobs,
        ]
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn capacity_and_numeric_refusals() {
        assert_eq!(Capacity::trial(false).slots(), 18);
        assert_eq!(Capacity::trial(false).build_leases(), 2);
        assert_eq!(Capacity::trial(true).slots(), 12);
        for value in [
            [2, 6, 3, 6, 1],
            [1, 5, 3, 6, 1],
            [1, 6, 0, 6, 1],
            [1, 6, 3, 6, 2],
            [1, usize::MAX, usize::MAX, 2, 1],
        ] {
            assert!(Capacity::new(value).is_err());
        }
        for text in ["", "0", "-1", "true", "1.0"] {
            assert!(positive("slots", text).is_err());
        }
    }
}
