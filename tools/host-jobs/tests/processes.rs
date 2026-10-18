use std::{
    fs,
    path::PathBuf,
    process::{Child, Command},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "host-jobs-test-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&p).unwrap();
        Self(p)
    }
    fn command(&self) -> Command {
        let mut c = Command::new(env!("CARGO_BIN_EXE_host-jobs"));
        c.env("VIBEOS_HOSTLOCK_DIR", &self.0)
            .env("VIBEOS_HOST_JOB_SLOTS", "2")
            .env("VIBEOS_HOST_LINK_SLOTS", "1")
            .env("VIBEOS_HOST_MEMORY_MIB", "1024")
            .env("VIBEOS_HOST_JOB_MEMORY_MIB", "4")
            .env("VIBEOS_HOST_MEMORY_HEADROOM_MIB", "0")
            .env_remove("VIBEOS_HOST_JOB_HELD")
            .env_remove("VIBEOS_INNER_RUSTC_WRAPPER")
            // Hermetic class: the gate runs this suite with VIBEOS_JOB_CLASS=integration,
            // which silently turned every unlabelled job here into a landing job. A test
            // that wants a class sets it on the command explicitly.
            .env_remove("VIBEOS_JOB_CLASS")
            .env_remove("VIBEOS_HOST_JOB_LAND_RESERVE");
        c
    }
    fn spawn(&self, kind: &str, name: &str) -> Child {
        self.command()
            .args([
                "--kind",
                kind,
                "--",
                "sh",
                "-c",
                &format!(
                    "touch '{}'; while test ! -f '{}'; do sleep .02; done",
                    self.0.join(name).display(),
                    self.0.join(format!("release-{name}")).display()
                ),
            ])
            .spawn()
            .unwrap()
    }
    fn wait(&self, name: &str) {
        let end = Instant::now() + Duration::from_secs(10);
        while !self.0.join(name).exists() {
            assert!(Instant::now() < end, "missing {name}");
            thread::sleep(Duration::from_millis(10));
        }
    }
    fn release(&self, name: &str, c: &mut Child) {
        fs::write(self.0.join(format!("release-{name}")), "").unwrap();
        assert!(c.wait().unwrap().success());
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
#[test]
fn capacity_and_link_wait_does_not_consume_cpu() {
    let f = Fixture::new();
    let mut a = f.spawn("link", "a");
    f.wait("a");
    let mut b = f.spawn("link", "b");
    thread::sleep(Duration::from_millis(80));
    let mut c = f.spawn("compile", "c");
    f.wait("c");
    assert!(!f.0.join("b").exists());
    let mut d = f.spawn("compile", "d");
    thread::sleep(Duration::from_millis(80));
    assert!(!f.0.join("d").exists());
    f.release("c", &mut c);
    f.wait("d");
    f.release("d", &mut d);
    f.release("a", &mut a);
    f.wait("b");
    f.release("b", &mut b);
}
#[test]
fn supervisor_death_does_not_release_live_child() {
    let f = Fixture::new();
    let mut a = f.spawn("compile", "a");
    f.wait("a");
    let mut b = f.spawn("compile", "b");
    f.wait("b");
    a.kill().unwrap();
    a.wait().unwrap();
    let mut c = f.spawn("compile", "c");
    thread::sleep(Duration::from_millis(150));
    assert!(!f.0.join("c").exists());
    fs::write(f.0.join("release-a"), "").unwrap();
    f.wait("c");
    f.release("c", &mut c);
    f.release("b", &mut b);
    let s = f.command().arg("--status").output().unwrap();
    assert!(String::from_utf8(s.stdout).unwrap().contains("\"jobs\":[]"));
}
#[test]
fn propagates_failure_and_signals() {
    let f = Fixture::new();
    for (body, want) in [("exit 42", 42), ("kill -TERM $$", 143)] {
        let s = f
            .command()
            .args(["--kind", "compile", "--", "sh", "-c", body])
            .status()
            .unwrap();
        assert_eq!(s.code(), Some(want));
    }
}
#[test]
fn priority_chooses_interactive_over_older_background() {
    let f = Fixture::new();
    let mut a = f.spawn("compile", "a");
    let mut b = f.spawn("compile", "b");
    f.wait("a");
    f.wait("b");
    let mut bg = f
        .command()
        .env("VIBEOS_JOB_CLASS", "background")
        .args([
            "--kind",
            "compile",
            "--",
            "sh",
            "-c",
            &format!("echo bg >> '{}'; sleep .2", f.0.join("order").display()),
        ])
        .spawn()
        .unwrap();
    thread::sleep(Duration::from_millis(100));
    let mut fg = f
        .command()
        .env("VIBEOS_JOB_CLASS", "interactive")
        .args([
            "--kind",
            "compile",
            "--",
            "sh",
            "-c",
            &format!("echo fg >> '{}'; sleep .2", f.0.join("order").display()),
        ])
        .spawn()
        .unwrap();
    thread::sleep(Duration::from_millis(100));
    f.release("a", &mut a);
    assert!(fg.wait().unwrap().success());
    assert!(bg.wait().unwrap().success());
    assert_eq!(fs::read_to_string(f.0.join("order")).unwrap(), "fg\nbg\n");
    f.release("b", &mut b);
}

#[test]
fn cache_guardian_survives_supervisor_and_does_not_leak_into_daemon() {
    use std::os::unix::fs::PermissionsExt;
    let f = Fixture::new();
    let wrapper = f.0.join("cache-wrapper");
    fs::write(
        &wrapper,
        "#!/bin/sh\nsleep 2 >/dev/null 2>&1 &\nexec \"$@\"\n",
    )
    .unwrap();
    fs::set_permissions(&wrapper, fs::Permissions::from_mode(0o755)).unwrap();
    let body = format!(
        "touch '{}'; while test ! -f '{}'; do sleep .02; done",
        f.0.join("cached").display(),
        f.0.join("release-cached").display()
    );
    let mut cached = f
        .command()
        .env("VIBEOS_INNER_RUSTC_WRAPPER", &wrapper)
        .args(["sh", "-c", &body])
        .spawn()
        .unwrap();
    f.wait("cached");
    let mut other = f.spawn("compile", "other");
    f.wait("other");
    cached.kill().unwrap();
    cached.wait().unwrap();
    let mut waiting = f.spawn("compile", "waiting");
    thread::sleep(Duration::from_millis(100));
    assert!(!f.0.join("waiting").exists());
    fs::write(f.0.join("release-cached"), "").unwrap();
    f.wait("waiting"); // cache's detached sleep is still alive: it owns no lease.
    f.release("waiting", &mut waiting);
    f.release("other", &mut other);
    let result = f.command().arg("--status").output().unwrap();
    assert!(String::from_utf8(result.stdout)
        .unwrap()
        .contains("\"jobs\":[]"));
}

#[test]
fn respects_existing_legacy_lease_without_mutating_it() {
    use std::{fs::OpenOptions, os::fd::AsRawFd};
    unsafe extern "C" {
        fn flock(fd: i32, op: i32) -> i32;
    }
    let f = Fixture::new();
    let path = f.0.join("cpuheavy-00");
    fs::write(&path, "legacy-owner-metadata").unwrap();
    let lease = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&path)
        .unwrap();
    assert_eq!(unsafe { flock(lease.as_raw_fd(), 2) }, 0);
    let mut first = f.spawn("compile", "first");
    f.wait("first");
    let mut second = f.spawn("compile", "second");
    thread::sleep(Duration::from_millis(100));
    assert!(!f.0.join("second").exists());
    assert_eq!(fs::read_to_string(&path).unwrap(), "legacy-owner-metadata");
    drop(lease);
    f.wait("second");
    f.release("first", &mut first);
    f.release("second", &mut second);
}

#[test]
fn smaller_limits_count_active_slots_above_the_new_prefix() {
    for kind in ["compile", "link"] {
        let f = Fixture::new();
        let spawn = |name: &str, smaller: bool| {
            let body = format!(
                "touch '{}'; while test ! -f '{}'; do sleep .02; done",
                f.0.join(name).display(),
                f.0.join(format!("release-{name}")).display()
            );
            let mut command = f.command();
            command.env("VIBEOS_HOST_LINK_SLOTS", if smaller { "1" } else { "2" });
            if smaller && kind == "compile" {
                command.env("VIBEOS_HOST_JOB_SLOTS", "1");
            }
            command
                .args(["--kind", kind, "--", "sh", "-c", &body])
                .spawn()
                .unwrap()
        };
        let mut low = spawn("low", false);
        f.wait("low");
        let mut high = spawn("high", false);
        f.wait("high");
        f.release("low", &mut low);
        let mut waiting = spawn("waiting", true);
        thread::sleep(Duration::from_millis(150));
        assert!(
            !f.0.join("waiting").exists(),
            "{kind} ignored higher active slot"
        );
        f.release("high", &mut high);
        f.wait("waiting");
        f.release("waiting", &mut waiting);
    }
}

#[test]
fn queued_compilers_fill_the_pool_and_report_run_ownership() {
    let f = Fixture::new();
    let mut children = Vec::new();
    for n in 0..24 {
        let body = format!(
            "touch '{}'; while test ! -f '{}'; do sleep .02; done",
            f.0.join(format!("started-{n}")).display(),
            f.0.join("release-all").display()
        );
        children.push(
            f.command()
                .env("VIBEOS_HOST_JOB_SLOTS", "8")
                .env("VIBEOS_JOB_RUN", "/artifacts/exact-run")
                .env("VIBEOS_JOB_LAUNCHER_PID", "123")
                .args(["--kind", "compile", "--", "sh", "-c", &body])
                .spawn()
                .unwrap(),
        );
    }
    let count = || {
        fs::read_dir(&f.0)
            .unwrap()
            .filter(|p| {
                p.as_ref()
                    .unwrap()
                    .file_name()
                    .to_string_lossy()
                    .starts_with("started-")
            })
            .count()
    };
    let end = Instant::now() + Duration::from_secs(10);
    while count() < 8 {
        assert!(Instant::now() < end, "pool did not fill");
        thread::sleep(Duration::from_millis(10));
    }
    thread::sleep(Duration::from_millis(100));
    assert_eq!(
        count(),
        8,
        "pool exceeded capacity while all admitted compilers were blocked at barrier"
    );
    let result = f.command().arg("--status").output().unwrap();
    let status = String::from_utf8(result.stdout).unwrap();
    assert_eq!(status.matches("\"state\":\"active\"").count(), 8);
    assert_eq!(
        status
            .matches("\"run_dir\":\"/artifacts/exact-run\"")
            .count(),
        24
    );
    assert_eq!(status.matches("\"launcher_pid\":\"123\"").count(), 24);
    fs::write(f.0.join("release-all"), "").unwrap();
    for child in &mut children {
        assert!(child.wait().unwrap().success());
    }
}

#[test]
fn more_than_128_waiters_work_with_low_file_descriptor_limit() {
    use std::os::unix::process::CommandExt;
    #[repr(C)]
    struct Limit {
        current: usize,
        maximum: usize,
    }
    unsafe extern "C" {
        fn setrlimit(resource: i32, limit: *const Limit) -> i32;
    }
    let f = Fixture::new();
    let mut blocker = f
        .command()
        .env("VIBEOS_HOST_JOB_SLOTS", "1")
        .args([
            "--kind",
            "compile",
            "--",
            "sh",
            "-c",
            &format!(
                "touch '{}'; while test ! -f '{}'; do sleep .02; done",
                f.0.join("blocker").display(),
                f.0.join("release-blocker").display()
            ),
        ])
        .spawn()
        .unwrap();
    f.wait("blocker");
    struct Waiters(Vec<Child>);
    impl Drop for Waiters {
        fn drop(&mut self) {
            for child in &mut self.0 {
                let _ = child.kill();
                let _ = child.wait();
            }
        }
    }
    let mut waiters = Waiters(Vec::new());
    for _ in 0..144 {
        let mut command = f.command();
        command
            .env("VIBEOS_HOST_JOB_SLOTS", "1")
            .args(["--kind", "compile", "--", "true"]);
        unsafe {
            command.pre_exec(|| {
                if setrlimit(
                    7,
                    &Limit {
                        current: 32,
                        maximum: 32,
                    },
                ) < 0
                {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }
        waiters.0.push(command.spawn().unwrap());
    }
    let end = Instant::now() + Duration::from_secs(20);
    loop {
        let registered = fs::read_dir(f.0.join("jobs")).unwrap().count();
        for child in &mut waiters.0 {
            assert!(
                child.try_wait().unwrap().is_none(),
                "queued runner exited before admission"
            );
        }
        if registered == 145 {
            break;
        }
        assert!(
            Instant::now() < end,
            "only {registered}/145 runners registered"
        );
        thread::sleep(Duration::from_millis(20));
    }
    // The exact resource which broke cold fleet builds is absent entirely.
    for child in &waiters.0 {
        for fd in fs::read_dir(format!("/proc/{}/fd", child.id())).unwrap() {
            if let Ok(target) = fs::read_link(fd.unwrap().path()) {
                assert_ne!(target.to_string_lossy(), "anon_inode:inotify");
            }
        }
    }
    // What 144 runner starts cost on this host RIGHT NOW, uncontended: each one
    // spawns, admits at once and runs `true`, which is exactly the work one
    // contended handoff below does. A fixed 5 s budget measured the host instead
    // of the scheduler: under a merge-queue gate plus four fleet builds it failed
    // at 127/144 and rejected lanes that had not touched host-jobs (2026-10-04).
    let baseline = {
        let solo = Fixture::new();
        let t0 = Instant::now();
        for _ in 0..48 {
            let status = solo
                .command()
                .env("VIBEOS_HOST_JOB_SLOTS", "1")
                .args(["--kind", "compile", "--", "true"])
                .status()
                .unwrap();
            assert!(status.success());
        }
        t0.elapsed() * 3
    };
    f.release("blocker", &mut blocker);
    // A normal completion must wake the next cohort immediately. The former
    // wake-all/full-scan convoy took over 12 seconds here (against a baseline of
    // about 1 s), and one-second timeout driven handoffs would take minutes; both
    // stay far outside three baselines plus slack, while scheduler noise scales
    // with the baseline it inflates.
    let budget = (baseline * 3 + Duration::from_secs(2)).max(Duration::from_secs(5));
    let deadline = Instant::now() + budget;
    loop {
        let mut finished = 0;
        for child in &mut waiters.0 {
            if let Some(status) = child.try_wait().unwrap() {
                assert!(status.success());
                finished += 1;
            }
        }
        if finished == 144 {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "only {finished}/144 normal handoffs completed in {budget:?} (uncontended baseline \
             {baseline:?}); possible admission convoy or missed wake"
        );
        thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn killed_unclaimed_grant_is_reclaimed_by_ticket_lock() {
    unsafe extern "C" {
        fn kill(pid: i32, signal: i32) -> i32;
    }
    let f = Fixture::new();
    let body = format!(
        "touch '{}'; while test ! -f '{}'; do sleep .02; done",
        f.0.join("blocker").display(),
        f.0.join("release-blocker").display()
    );
    let mut blocker = f
        .command()
        .env("VIBEOS_HOST_JOB_SLOTS", "1")
        .args(["--kind", "compile", "--", "sh", "-c", &body])
        .spawn()
        .unwrap();
    f.wait("blocker");
    let mut stopped = f
        .command()
        .env("VIBEOS_HOST_JOB_SLOTS", "1")
        .args(["--kind", "compile", "--", "true"])
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    while fs::read_to_string(format!("/proc/{}/wchan", stopped.id())).unwrap_or_default()
        != "futex_do_wait"
    {
        assert!(Instant::now() < deadline, "waiter never slept");
        thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(unsafe { kill(stopped.id() as i32, 19) }, 0); // stop only this test's sleeping waiter
    let mut next = f
        .command()
        .env("VIBEOS_HOST_JOB_SLOTS", "1")
        .args([
            "--kind",
            "compile",
            "--",
            "sh",
            "-c",
            &format!("touch '{}'", f.0.join("next").display()),
        ])
        .spawn()
        .unwrap();
    f.release("blocker", &mut blocker);
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let granted = fs::read_dir(f.0.join("jobs")).unwrap().any(|p| {
            let p = p.unwrap().path();
            p.file_name()
                .unwrap()
                .to_string_lossy()
                .ends_with(&format!("-{}.json", stopped.id()))
                && fs::read_to_string(p)
                    .unwrap_or_default()
                    .contains("\"state\":\"granted\"")
        });
        if granted {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "stopped waiter was not granted its slot"
        );
        thread::sleep(Duration::from_millis(10));
    }
    assert!(
        !f.0.join("next").exists(),
        "reserved slot was double-assigned"
    );
    stopped.kill().unwrap();
    stopped.wait().unwrap();
    f.wait("next");
    assert!(next.wait().unwrap().success());
}

#[test]
fn memory_waiters_leave_cpu_for_small_jobs_and_survive_owner_death() {
    let f = Fixture::new();
    let spawn = |name: &str, kind: &str, mib: &str| {
        f.command()
            .env("VIBEOS_HOST_MEMORY_MIB", "128")
            .env("VIBEOS_HOST_JOB_MEMORY_MIB", mib)
            .args([
                "--kind",
                kind,
                "--",
                "sh",
                "-c",
                &format!(
                    "touch '{}'; while test ! -f '{}'; do sleep .02; done",
                    f.0.join(name).display(),
                    f.0.join(format!("release-{name}")).display()
                ),
            ])
            .spawn()
            .unwrap()
    };
    let mut large = spawn("large", "compile", "96");
    f.wait("large");
    let mut lint = spawn("lint", "lint", "96");
    thread::sleep(Duration::from_millis(100));
    assert!(!f.0.join("lint").exists());
    let mut small = spawn("small", "compile", "16");
    f.wait("small");
    large.kill().unwrap();
    large.wait().unwrap();
    f.release("small", &mut small);
    thread::sleep(Duration::from_millis(150));
    assert!(
        !f.0.join("lint").exists(),
        "child must retain memory reservation"
    );
    fs::write(f.0.join("release-large"), "").unwrap();
    f.wait("lint");
    f.release("lint", &mut lint);
}

#[test]
fn impossible_reservation_fails_before_command_and_metrics_preserve_exit() {
    let f = Fixture::new();
    let result = f
        .command()
        .env("VIBEOS_HOST_MEMORY_MIB", "16")
        .env("VIBEOS_HOST_JOB_MEMORY_MIB", "32")
        .args([
            "--kind",
            "lint",
            "--",
            "touch",
            f.0.join("unexpected").to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(125));
    assert!(String::from_utf8(result.stderr)
        .unwrap()
        .contains("job requests"));
    assert!(!f.0.join("unexpected").exists());

    let result = f
        .command()
        .args([
            "--kind",
            "lint",
            "--",
            "python3",
            "-c",
            "x=bytearray(32*1024*1024); raise SystemExit(42)",
        ])
        .status()
        .unwrap();
    assert_eq!(result.code(), Some(42));
    let log = fs::read_to_string(f.0.join("job-metrics.jsonl")).unwrap();
    assert_eq!(log.lines().count(), 1);
    assert!(log.contains("\"exit\":42"));
    let peak: usize = log
        .split("\"wait4_maxrss_kib\":")
        .nth(1)
        .unwrap()
        .split(',')
        .next()
        .unwrap()
        .parse()
        .unwrap();
    assert!(
        peak >= 32 * 1024,
        "kernel high-water measurement missed short allocation: {log}"
    );
    assert!(log.contains("\"kind\":\"lint\""));
}

#[test]
fn old_dispatcher_grant_is_rechecked_before_execution() {
    let f = Fixture::new();
    let mut a = f
        .command()
        .env("VIBEOS_HOST_MEMORY_MIB", "128")
        .env("VIBEOS_HOST_JOB_MEMORY_MIB", "96")
        .args([
            "--kind",
            "compile",
            "--",
            "sh",
            "-c",
            &format!(
                "touch '{}'; while test ! -f '{}'; do sleep .02; done",
                f.0.join("a").display(),
                f.0.join("release-a").display()
            ),
        ])
        .spawn()
        .unwrap();
    f.wait("a");
    let mut b = f
        .command()
        .env("VIBEOS_HOST_MEMORY_MIB", "128")
        .env("VIBEOS_HOST_JOB_MEMORY_MIB", "96")
        .args([
            "--kind",
            "lint",
            "--",
            "touch",
            f.0.join("b").to_str().unwrap(),
        ])
        .spawn()
        .unwrap();
    thread::sleep(Duration::from_millis(100));
    // Simulate an older broker granting the spare CPU without memory checks.
    use std::os::fd::AsRawFd;
    unsafe extern "C" {
        fn flock(fd: i32, op: i32) -> i32;
    }
    let mutex = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(f.0.join("jobs.lock"))
        .unwrap();
    assert_eq!(unsafe { flock(mutex.as_raw_fd(), 2) }, 0);
    let path = fs::read_dir(f.0.join("jobs"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .find(|p| {
            p.to_string_lossy()
                .ends_with(&format!("-lint-{}.json", b.id()))
        })
        .unwrap();
    let body = fs::read_to_string(&path).unwrap();
    assert!(body.contains("\"state\":\"waiting\""));
    fs::write(
        path,
        body.replace(
            "\"state\":\"waiting\"}",
            "\"state\":\"granted\",\"cpu_slot\":1}",
        ),
    )
    .unwrap();
    drop(mutex);
    thread::sleep(Duration::from_millis(1200));
    assert!(!f.0.join("b").exists());
    f.release("a", &mut a);
    assert!(b.wait().unwrap().success());
    assert!(f.0.join("b").exists());
}

#[test]
fn excessive_old_grants_do_not_deadlock_new_claimants() {
    let f = Fixture::new();
    // Hold both CPU slots while two memory-heavy jobs enqueue.
    let mut a = f.spawn("compile", "a");
    let mut b = f.spawn("compile", "b");
    f.wait("a");
    f.wait("b");
    let spawn = |name: &str| {
        f.command()
            .env("VIBEOS_HOST_MEMORY_MIB", "128")
            .env("VIBEOS_HOST_JOB_MEMORY_MIB", "96")
            .args([
                "--kind",
                "lint",
                "--",
                "sh",
                "-c",
                &format!(
                    "touch '{}'; while test ! -f '{}'; do sleep .02; done",
                    f.0.join(name).display(),
                    f.0.join(format!("release-{name}")).display()
                ),
            ])
            .spawn()
            .unwrap()
    };
    let mut c = spawn("c");
    let mut d = spawn("d");
    thread::sleep(Duration::from_millis(100));
    // Freeze claimants, release holders, and install two simultaneous old-style
    // grants whose total exceeds memory capacity. Neither is actually running.
    unsafe extern "C" {
        fn kill(pid: i32, signal: i32) -> i32;
        fn waitpid(pid: i32, status: *mut i32, options: i32) -> i32;
        fn flock(fd: i32, op: i32) -> i32;
    }
    use std::os::fd::AsRawFd;
    // Never stop a broker while it owns the queue mutex.
    let freeze_lock = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(f.0.join("jobs.lock"))
        .unwrap();
    assert_eq!(unsafe { flock(freeze_lock.as_raw_fd(), 2) }, 0);
    for child in [&c, &d] {
        assert_eq!(unsafe { kill(child.id() as i32, 19) }, 0);
        let mut status = 0;
        assert_eq!(
            unsafe { waitpid(child.id() as i32, &mut status, 2) },
            child.id() as i32
        );
        assert_eq!(status & 0xff, 0x7f);
    }
    drop(freeze_lock);
    f.release("a", &mut a);
    f.release("b", &mut b);
    let mutex = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(f.0.join("jobs.lock"))
        .unwrap();
    assert_eq!(unsafe { flock(mutex.as_raw_fd(), 2) }, 0);
    for (slot, child) in [&c, &d].iter().enumerate() {
        let path = fs::read_dir(f.0.join("jobs"))
            .unwrap()
            .map(|e| e.unwrap().path())
            .find(|p| {
                p.to_string_lossy()
                    .ends_with(&format!("-lint-{}.json", child.id()))
            })
            .unwrap();
        let mut body = fs::read_to_string(&path).unwrap();
        // A new dispatcher may already have granted one job after a holder exited.
        if body.contains("\"state\":\"waiting\"") {
            body = body.replace(
                "\"state\":\"waiting\"}",
                &format!("\"state\":\"granted\",\"cpu_slot\":{slot}}}"),
            );
        }
        fs::write(path, body).unwrap();
    }
    drop(mutex);
    // Resume C alone: it must start even though D has an excessive promise.
    assert_eq!(unsafe { kill(c.id() as i32, 18) }, 0);
    f.wait("c");
    assert_eq!(unsafe { kill(d.id() as i32, 18) }, 0);
    thread::sleep(Duration::from_millis(100));
    assert!(!f.0.join("d").exists());
    f.release("c", &mut c);
    f.wait("d");
    f.release("d", &mut d);
}
