//! Per-command observations: sampled simultaneous tree RSS plus kernel-reported
//! largest-process high-water RSS. Local cache workers are request-tagged.
use super::{code, lock, memory, now, open, profiles, quote, Ticket};
use std::{
    fs,
    io::{self, Write},
    path::Path,
    process::Command,
    time::{Duration, Instant},
};
#[repr(C)]
#[derive(Default)]
struct Timeval {
    seconds: std::ffi::c_long,
    micros: std::ffi::c_long,
}
#[repr(C)]
#[derive(Default)]
struct Usage {
    user: Timeval,
    system: Timeval,
    maxrss: std::ffi::c_long,
    other: [std::ffi::c_long; 13],
}
unsafe extern "C" {
    fn wait4(pid: i32, status: *mut i32, options: i32, usage: *mut Usage) -> i32;
}

pub fn execute(
    cmd: &mut Command,
    root: &Path,
    ticket: &Ticket,
    key: &str,
    cached: bool,
) -> io::Result<i32> {
    use std::os::unix::process::ExitStatusExt;
    let before = memory::snapshot()?;
    let pressure_before = pressure()?;
    fs::create_dir_all(root.join("memory-live"))?;
    let live_path = root
        .join("memory-live")
        .join(ticket.path.file_name().unwrap());
    let started = Instant::now();
    let start_ms = now();
    let token = format!(
        "{}-{}-{}",
        std::process::id(),
        start_ms,
        ticket.path.file_name().unwrap().to_string_lossy()
    );
    if cached {
        cmd.env(crate::cache_memory::MARKER, &token);
    }
    let child = cmd.spawn()?;
    let mut usage = Usage::default();
    let mut status = 0;
    // Wait synchronously for exit; a sampler sleeps on a condition variable
    // so cache hits do not pay a polling interval on every invocation.
    let stop = std::sync::Arc::new((std::sync::Mutex::new(false), std::sync::Condvar::new()));
    let signal = stop.clone();
    let sample_path = live_path.clone();
    let pid = child.id();
    let sampler = std::thread::Builder::new()
        .name("memory-sampler".into())
        .spawn(move || {
            let mut peak = 0;
            let mut workers = crate::cache_memory::Workers::new(&token);
            let mut samples = 0;
            let mut error = None;
            loop {
                let rss = if cached && samples > 0 {
                    let mut pids = memory::process_tree_pids(pid);
                    workers.extend(&mut pids, pid);
                    memory::process_rss_kib(&pids)
                } else if cached {
                    // Delay daemon discovery until 100ms: short cache hits do
                    // not need a process-table scan. Avoid charging a server
                    // which has not yet detached from its startup client.
                    memory::process_rss_kib(&std::collections::HashSet::from([pid]))
                } else {
                    memory::process_tree_rss_kib(pid)
                };
                samples += 1;
                peak = peak.max(rss);
                if error.is_none() {
                    let contents = format!(
                        "{{\"rss_kib\":{rss},\"peak_kib\":{peak},\"sample_ms\":{}}}",
                        now()
                    );
                    if let Err(e) = atomic_live(&sample_path, &contents) {
                        error = Some(e);
                    }
                }
                let (mutex, wake) = &*signal;
                let done = mutex.lock().unwrap();
                let (done, _) = wake
                    .wait_timeout_while(done, Duration::from_millis(100), |done| !*done)
                    .unwrap();
                if *done {
                    break;
                }
            }
            (peak, error, workers.matched.len(), workers.unreadable)
        });
    let waited = loop {
        let result = unsafe { wait4(child.id() as i32, &mut status, 0, &mut usage) };
        if result > 0 {
            break Ok(());
        }
        let e = io::Error::last_os_error();
        if e.kind() != io::ErrorKind::Interrupted {
            break Err(e);
        }
    };
    *stop.0.lock().unwrap() = true;
    stop.1.notify_one();
    // Even if the OS refused the sampler thread, reap the command above
    // before releasing its leases or returning the telemetry error.
    let (peak, error, cache_worker_pids, cache_scan_denied) = sampler?
        .join()
        .map_err(|_| io::Error::other("memory sampler panicked"))?;
    waited?;
    let exit = code(std::process::ExitStatus::from_raw(status));
    let child_peak = usize::try_from(usage.maxrss).unwrap_or(0);
    let observed = peak.max(child_peak);
    let _ = fs::remove_file(live_path);
    if let Some(e) = error {
        return Err(e);
    } // never abandon a live child on telemetry I/O failure
    if observed > 0 {
        profiles::record_peak(&root.join("memory-profiles"), key, observed)?;
    }
    let after = memory::snapshot()?;
    let pressure_after = pressure()?;
    let attribution = if cached {
        "request-tagged-local-cache-workers-and-client"
    } else {
        "process-tree-samples-and-wait4"
    };
    let elapsed = started.elapsed().as_millis();
    let queued = start_ms.saturating_sub(
        super::number(&ticket.meta, "created_ms").unwrap_or(start_ms as usize) as u64,
    );
    let cpu_user_us = usage.user.seconds as i128 * 1_000_000 + usage.user.micros as i128;
    let cpu_system_us = usage.system.seconds as i128 * 1_000_000 + usage.system.micros as i128;
    let record = format!("{{{},\"profile\":{},\"attribution\":{},\"exit\":{exit},\"queue_ms\":{queued},\"elapsed_ms\":{elapsed},\"sampled_tree_peak_kib\":{peak},\"wait4_maxrss_kib\":{child_peak},\"cache_worker_pids\":{cache_worker_pids},\"cache_scan_denied\":{cache_scan_denied},\"cpu_user_us\":{cpu_user_us},\"cpu_system_us\":{cpu_system_us},\"memory_pressure_before\":{},\"memory_pressure_after\":{},\"available_before_kib\":{},\"available_after_kib\":{}}}\n", ticket.meta, quote(key), quote(attribution), quote(&pressure_before), quote(&pressure_after), before.available_kib, after.available_kib);
    append(root, &record)?;
    Ok(exit)
}
fn atomic_live(path: &Path, contents: &str) -> io::Result<()> {
    let temp = path.with_extension("tmp");
    fs::write(&temp, contents)?;
    fs::rename(temp, path)
}
fn append(root: &Path, record: &str) -> io::Result<()> {
    let guard = open(&root.join("job-metrics.lock"))?;
    lock(&guard, false)?;
    let path = root.join("job-metrics.jsonl");
    if fs::metadata(&path).is_ok_and(|m| m.len() >= 64 * 1024 * 1024) {
        fs::rename(&path, root.join("job-metrics.previous.jsonl"))?;
    }
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    file.write_all(record.as_bytes())
}

// PSI reports host contention independently of the broker's reservations.
// Absence means the kernel does not expose PSI; other read errors are fatal.
fn pressure() -> io::Result<String> {
    match fs::read_to_string("/proc/pressure/memory") {
        Ok(text) => Ok(text),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok("unavailable".into()),
        Err(e) => Err(e),
    }
}
