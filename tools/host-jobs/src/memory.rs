//! Linux host and cgroup memory telemetry for host-jobs.
//!
//! `Snapshot::available_kib` is a point-in-time estimate. It is not a promise
//! that a later allocation will succeed, and process RSS is likewise a sample,
//! not a peak measurement.

use std::{
    fs, io,
    path::{Component, Path, PathBuf},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Snapshot {
    pub total_kib: usize,
    pub available_kib: usize,
}

/// Read host memory and cap available memory by finite cgroup v2 headroom.
pub fn snapshot() -> io::Result<Snapshot> {
    snapshot_from(Path::new("/proc"))
}

fn snapshot_from(proc_root: &Path) -> io::Result<Snapshot> {
    let meminfo = fs::read_to_string(proc_root.join("meminfo"))?;
    let mut result = parse_meminfo(&meminfo)?;
    if let Some((mount_root, mount_point, membership)) = cgroup_v2_paths(proc_root)? {
        if let Some(cgroup_dir) = mounted_cgroup_path(&mount_root, &mount_point, &membership) {
            if let Some((total_kib, available_kib)) =
                cgroup_available_kib(&cgroup_dir, &mount_point)?
            {
                result.total_kib = result.total_kib.min(total_kib);
                result.available_kib = result.available_kib.min(available_kib);
            }
        }
    }
    Ok(result)
}

fn parse_meminfo(text: &str) -> io::Result<Snapshot> {
    let mut total = None;
    let mut available = None;
    for line in text.lines() {
        let (key, rest) = match line.split_once(':') {
            Some(parts) => parts,
            None => continue,
        };
        if key != "MemTotal" && key != "MemAvailable" {
            continue;
        }
        let mut fields = rest.split_whitespace();
        let value = fields
            .next()
            .ok_or_else(|| invalid_data(format!("missing value for {key}")))?
            .parse::<usize>()
            .map_err(|_| invalid_data(format!("invalid value for {key}")))?;
        let unit = fields.next().unwrap_or("kB");
        if unit != "kB" || fields.next().is_some() {
            return Err(invalid_data(format!("invalid units for {key}")));
        }
        match key {
            "MemTotal" => total = Some(value),
            "MemAvailable" => available = Some(value),
            _ => unreachable!(),
        }
    }
    let total_kib = total.ok_or_else(|| invalid_data("MemTotal missing"))?;
    let available_kib = available.ok_or_else(|| invalid_data("MemAvailable missing"))?;
    Ok(Snapshot {
        total_kib,
        available_kib: available_kib.min(total_kib),
    })
}

/// Return the visible v2 mount root, mount point, and this process's membership.
fn cgroup_v2_paths(proc_root: &Path) -> io::Result<Option<(PathBuf, PathBuf, PathBuf)>> {
    let cgroup = fs::read_to_string(proc_root.join("self/cgroup"))?;
    let membership = match parse_cgroup_v2_membership(&cgroup) {
        Some(path) => path,
        None => return Ok(None),
    };
    let mountinfo = fs::read_to_string(proc_root.join("self/mountinfo"))?;
    let mount = match parse_cgroup2_mount(&mountinfo) {
        Some(mount) => mount,
        None => return Ok(None),
    };
    Ok(Some((mount.0, mount.1, membership)))
}

fn parse_cgroup_v2_membership(text: &str) -> Option<PathBuf> {
    text.lines().find_map(|line| {
        let mut fields = line.splitn(3, ':');
        let hierarchy = fields.next()?;
        let controllers = fields.next()?;
        let path = fields.next()?;
        if hierarchy == "0" && controllers.is_empty() && path.starts_with('/') {
            Some(PathBuf::from(path))
        } else {
            None
        }
    })
}

/// Parse `(root, mountpoint)` for the first cgroup2 mount in mountinfo.
fn parse_cgroup2_mount(text: &str) -> Option<(PathBuf, PathBuf)> {
    text.lines().find_map(|line| {
        let (before, after) = line.split_once(" - ")?;
        let mut left = before.split_whitespace();
        let _mount_id = left.next()?;
        let _parent_id = left.next()?;
        let _device = left.next()?;
        let root = unescape_mount_field(left.next()?)?;
        let mount_point = unescape_mount_field(left.next()?)?;
        let mut right = after.split_whitespace();
        if right.next()? != "cgroup2" {
            return None;
        }
        Some((PathBuf::from(root), PathBuf::from(mount_point)))
    })
}

fn unescape_mount_field(value: &str) -> Option<String> {
    let bytes = value.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\\' {
            if i + 3 >= bytes.len()
                || bytes[i + 1] > b'3'
                || !bytes[i + 1..i + 4]
                    .iter()
                    .all(|b| (b'0'..=b'7').contains(b))
            {
                return None;
            }
            let code =
                (bytes[i + 1] - b'0') * 64 + (bytes[i + 2] - b'0') * 8 + (bytes[i + 3] - b'0');
            out.push(code);
            i += 4;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(out).ok()
}

/// Map a membership path through the mount's visible hierarchy root.
fn mounted_cgroup_path(root: &Path, mount_point: &Path, membership: &Path) -> Option<PathBuf> {
    let relative = membership.strip_prefix(root).ok()?;
    if relative
        .components()
        .any(|part| !matches!(part, Component::Normal(_)))
    {
        return None;
    }
    Some(mount_point.join(relative))
}

/// Minimum finite ancestor headroom, through and including the mount root.
/// Returns `None` when no readable finite limit exists.
fn cgroup_available_kib(
    cgroup_dir: &Path,
    mount_point: &Path,
) -> io::Result<Option<(usize, usize)>> {
    let mut current_dir = cgroup_dir.to_path_buf();
    let mut minimum = None;
    loop {
        let max_path = current_dir.join("memory.max");
        let current_path = current_dir.join("memory.current");
        let max_text = read_optional(&max_path)?;
        let current_text = read_optional(&current_path)?;
        let max_bytes = max_text
            .as_deref()
            .map(|text| parse_cgroup_limit(text, &max_path))
            .transpose()?;
        let current_bytes = current_text
            .as_deref()
            .map(|text| parse_cgroup_current(text, &current_path))
            .transpose()?;
        if let (Some(Some(max_bytes)), Some(current_bytes)) = (max_bytes, current_bytes) {
            // memory.current includes filesystem cache. Counting all of it as
            // unavailable can stall a warm build forever: no new allocation is
            // admitted to trigger reclaim. Credit only clean inactive file pages,
            // not anonymous, active, dirty or writeback memory. This is an
            // estimate, still capped by host MemAvailable and every ancestor.
            // Fields: kernel.org/doc/html/latest/admin-guide/cgroup-v2.html
            let stat_path = current_dir.join("memory.stat");
            let reclaimable = read_optional(&stat_path)?
                .map(|text| clean_inactive_file(&text))
                .transpose()?
                .unwrap_or(0);
            let working = current_bytes.saturating_sub(reclaimable);
            let headroom_kib = max_bytes.saturating_sub(working) / 1024;
            minimum = Some(
                minimum.map_or((max_bytes / 1024, headroom_kib), |old: (usize, usize)| {
                    (old.0.min(max_bytes / 1024), old.1.min(headroom_kib))
                }),
            );
        }
        if current_dir == mount_point {
            break;
        }
        if !current_dir.pop() || !current_dir.starts_with(mount_point) {
            return Err(invalid_data("cgroup path escaped mount point"));
        }
    }
    Ok(minimum)
}

fn clean_inactive_file(text: &str) -> io::Result<usize> {
    let mut inactive = 0usize;
    let mut dirty = 0usize;
    let mut writeback = 0usize;
    for line in text.lines() {
        let mut parts = line.split_whitespace();
        let dest = match parts.next() {
            Some("inactive_file") => &mut inactive,
            Some("file_dirty") => &mut dirty,
            Some("file_writeback") => &mut writeback,
            _ => continue,
        };
        *dest = parts
            .next()
            .and_then(|v| v.parse().ok())
            .ok_or_else(|| invalid_data("invalid reclaim counter in memory.stat"))?;
        if parts.next().is_some() {
            return Err(invalid_data("invalid memory.stat counter"));
        }
    }
    Ok(inactive.saturating_sub(dirty).saturating_sub(writeback))
}

fn parse_cgroup_limit(text: &str, path: &Path) -> io::Result<Option<usize>> {
    let value = text.trim();
    if value == "max" {
        return Ok(None);
    }
    let bytes = value
        .parse::<usize>()
        .map_err(|_| invalid_data(format!("invalid cgroup limit in {}", path.display())))?;
    Ok(Some(bytes))
}

fn parse_cgroup_current(text: &str, path: &Path) -> io::Result<usize> {
    text.trim()
        .parse::<usize>()
        .map_err(|_| invalid_data(format!("invalid cgroup usage in {}", path.display())))
}

fn read_optional(path: &Path) -> io::Result<Option<String>> {
    match fs::read_to_string(path) {
        Ok(value) => Ok(Some(value)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

fn invalid_data(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}

/// Sample VmRSS for `pid` and all recursively discovered thread children.
/// Processes that disappear during the walk are skipped. This is a current
/// sample and does not claim to capture the tree's peak RSS.
pub fn process_tree_rss_kib(pid: u32) -> usize {
    process_rss_kib(&process_tree_pids(pid))
}

pub fn process_rss_kib(pids: &std::collections::HashSet<u32>) -> usize {
    pids.iter()
        .filter_map(|pid| fs::read_to_string(format!("/proc/{pid}/status")).ok())
        .filter_map(|status| parse_vm_rss_kib(&status))
        .sum()
}

pub fn process_tree_pids(pid: u32) -> std::collections::HashSet<u32> {
    let mut pending = vec![pid];
    let mut seen = std::collections::HashSet::new();
    while let Some(pid) = pending.pop() {
        if !seen.insert(pid) {
            continue;
        }
        let proc_dir = PathBuf::from("/proc").join(pid.to_string());
        let task_dir = proc_dir.join("task");
        let Ok(tasks) = fs::read_dir(task_dir) else {
            continue;
        };
        for task in tasks.flatten() {
            let children = task.path().join("children");
            if let Ok(text) = fs::read_to_string(children) {
                pending.extend(
                    text.split_whitespace()
                        .filter_map(|child| child.parse::<u32>().ok()),
                );
            }
        }
    }
    seen
}

fn parse_vm_rss_kib(text: &str) -> Option<usize> {
    text.lines().find_map(|line| {
        let rest = line.strip_prefix("VmRSS:")?;
        let mut fields = rest.split_whitespace();
        let value = fields.next()?.parse::<usize>().ok()?;
        (fields.next()? == "kB").then_some(value)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        sync::atomic::{AtomicU64, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    static NEXT: AtomicU64 = AtomicU64::new(0);

    struct Fixture(PathBuf);

    impl Fixture {
        fn new() -> Self {
            let id = NEXT.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "host-jobs-memory-{}-{}-{id}",
                std::process::id(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }

        fn proc(&self) -> PathBuf {
            let proc = self.0.join("proc");
            fs::create_dir_all(proc.join("self")).unwrap();
            proc
        }

        fn cgroup_mount(&self) -> PathBuf {
            let path = self.0.join("visible cgroup");
            fs::create_dir_all(&path).unwrap();
            path
        }

        fn write_limit(path: &Path, max: &str, current: &str) {
            fs::create_dir_all(path).unwrap();
            fs::write(path.join("memory.max"), max).unwrap();
            fs::write(path.join("memory.current"), current).unwrap();
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn parses_meminfo_and_rejects_missing_or_malformed_values() {
        assert_eq!(
            parse_meminfo("MemTotal: 1000 kB\nMemAvailable: 700 kB\n").unwrap(),
            Snapshot {
                total_kib: 1000,
                available_kib: 700
            }
        );
        assert_eq!(
            parse_meminfo("MemTotal: 10 kB\nMemAvailable: 20 kB\n")
                .unwrap()
                .available_kib,
            10
        );
        assert!(parse_meminfo("MemTotal: nope kB\nMemAvailable: 1 kB\n").is_err());
        assert!(parse_meminfo("MemTotal: 1 kB\n").is_err());
    }

    #[test]
    fn parses_cgroup_membership_mount_root_and_escaped_mountpoint() {
        let membership = parse_cgroup_v2_membership("5:cpu:/legacy\n0::/slice/job\n").unwrap();
        assert_eq!(membership, PathBuf::from("/slice/job"));
        let (root, mount) = parse_cgroup2_mount(
            "31 22 0:28 /slice /sys/fs/cgroup/my\\040group rw - cgroup2 cgroup rw\n",
        )
        .unwrap();
        assert_eq!(root, PathBuf::from("/slice"));
        assert_eq!(mount, PathBuf::from("/sys/fs/cgroup/my group"));
        assert_eq!(
            mounted_cgroup_path(&root, &mount, &membership).unwrap(),
            mount.join("job")
        );
        assert!(mounted_cgroup_path(&root, &mount, Path::new("/outside")).is_none());
    }

    #[test]
    fn snapshot_uses_minimum_finite_hierarchical_headroom() {
        let f = Fixture::new();
        let proc = f.proc();
        fs::write(
            proc.join("meminfo"),
            "MemTotal: 100000 kB\nMemAvailable: 90000 kB\n",
        )
        .unwrap();
        let mount = f.cgroup_mount();
        fs::write(proc.join("self/cgroup"), "0::/slice/team/job\n").unwrap();
        fs::write(
            proc.join("self/mountinfo"),
            format!(
                "31 22 0:28 /slice {} rw - cgroup2 cgroup rw\n",
                mount.to_string_lossy().replace(' ', "\\040")
            ),
        )
        .unwrap();
        Fixture::write_limit(&mount, "max", "1");
        Fixture::write_limit(&mount.join("team"), "10485760", "4194304");
        Fixture::write_limit(&mount.join("team/job"), "20971520", "19922944");
        assert_eq!(
            snapshot_from(&proc).unwrap(),
            Snapshot {
                total_kib: 10240,
                available_kib: 1024
            }
        );
    }

    #[test]
    fn unlimited_and_unknown_cgroup_limits_leave_host_availability() {
        let f = Fixture::new();
        let proc = f.proc();
        fs::write(
            proc.join("meminfo"),
            "MemTotal: 100000 kB\nMemAvailable: 90000 kB\n",
        )
        .unwrap();
        let mount = f.cgroup_mount();
        fs::write(proc.join("self/cgroup"), "0::/group\n").unwrap();
        fs::write(
            proc.join("self/mountinfo"),
            format!(
                "31 22 0:28 / {} rw - cgroup2 cgroup rw\n",
                mount.to_string_lossy().replace(' ', "\\040")
            ),
        )
        .unwrap();
        Fixture::write_limit(&mount, "max", "0");
        Fixture::write_limit(&mount.join("group"), "max", "100");
        assert_eq!(snapshot_from(&proc).unwrap().available_kib, 90000);
    }

    #[test]
    fn cgroup_cache_can_be_reclaimed_without_crediting_dirty_pages() {
        let f = Fixture::new();
        let mount = f.cgroup_mount();
        Fixture::write_limit(&mount, "10485760", "14680064");
        fs::write(mount.join("memory.stat"),
            "inactive_file 8388608\nfile_dirty 1048576\nfile_writeback 1048576\nactive_file 99999999\n").unwrap();
        assert_eq!(
            cgroup_available_kib(&mount, &mount).unwrap(),
            Some((10240, 2048))
        );
        assert_eq!(
            clean_inactive_file("inactive_file 10\nfile_dirty 20\n").unwrap(),
            0
        );
        assert!(clean_inactive_file("inactive_file broken\n").is_err());
    }

    #[test]
    fn malformed_existing_cgroup_values_fail_loudly() {
        let f = Fixture::new();
        let proc = f.proc();
        fs::write(
            proc.join("meminfo"),
            "MemTotal: 100000 kB\nMemAvailable: 90000 kB\n",
        )
        .unwrap();
        let mount = f.cgroup_mount();
        fs::write(proc.join("self/cgroup"), "0::/group\n").unwrap();
        fs::write(
            proc.join("self/mountinfo"),
            format!(
                "31 22 0:28 / {} rw - cgroup2 cgroup rw\n",
                mount.to_string_lossy().replace(' ', "\\040")
            ),
        )
        .unwrap();
        Fixture::write_limit(&mount, "broken", "0");
        Fixture::write_limit(&mount.join("group"), "max", "0");
        assert_eq!(
            snapshot_from(&proc).unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
    }

    #[test]
    fn rss_parser_reads_only_vm_rss_kib() {
        assert_eq!(
            parse_vm_rss_kib("Name:\ttest\nVmPeak:\t90 kB\nVmRSS:\t42 kB\n"),
            Some(42)
        );
        assert_eq!(parse_vm_rss_kib("Name:\ttest\n"), None);
    }
}
