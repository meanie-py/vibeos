//! Process-scoped compiler admission. Locks, not PID existence, are authoritative.
use std::{
    env,
    fs::{self, File, OpenOptions},
    io::{self, Write},
    os::{
        fd::{AsRawFd, RawFd},
        unix::process::ExitStatusExt,
    },
    path::{Path, PathBuf},
    process::{Command, ExitCode},
    rc::Rc,
    sync::atomic::{AtomicU32, AtomicU64, Ordering},
    thread,
    time::{SystemTime, UNIX_EPOCH},
};
unsafe extern "C" {
    fn flock(fd: i32, op: i32) -> i32;
    fn fcntl(fd: i32, cmd: i32, ...) -> i32;
    fn fork() -> i32;
    fn waitpid(pid: i32, status: *mut i32, options: i32) -> i32;
    fn _exit(code: i32) -> !;
    fn mmap(
        address: *mut std::ffi::c_void,
        length: usize,
        prot: i32,
        flags: i32,
        fd: i32,
        offset: std::ffi::c_long,
    ) -> *mut std::ffi::c_void;
    fn munmap(address: *mut std::ffi::c_void, length: usize) -> i32;
    fn syscall(number: std::ffi::c_long, ...) -> std::ffi::c_long;
}
#[cfg(target_arch = "x86_64")]
const SYS_FUTEX: std::ffi::c_long = 202;
#[cfg(target_arch = "aarch64")]
const SYS_FUTEX: std::ffi::c_long = 98;
#[cfg(target_arch = "x86")]
const SYS_FUTEX: std::ffi::c_long = 240;
#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64", target_arch = "x86")))]
compile_error!("host-jobs needs the Linux futex syscall number for this host architecture");
#[repr(C)]
struct Timespec {
    seconds: std::ffi::c_long,
    nanos: std::ffi::c_long,
}
/// All waiters share one futex word: no per-user inotify-instance consumption.
/// Generation capture precedes admission; FUTEX_WAIT checks it atomically so a
/// transition between the scan and sleeping cannot be lost.
#[repr(C)]
struct EventState {
    generation: AtomicU32,
    dirty: AtomicU32,
    dispatched: AtomicU32,
    reap_after: AtomicU64,
}
struct Wakeup(*mut EventState);
impl Wakeup {
    fn new(root: &Path) -> io::Result<Self> {
        let file = open(&root.join("jobs.wake-v2"))?;
        lock(&file, false)?;
        match file.metadata()?.len() {
            0 => file.set_len(24)?,
            24 => {}
            _ => return Err(io::Error::other("invalid jobs.wake size")),
        }
        let address = unsafe { mmap(std::ptr::null_mut(), 24, 3, 1, file.as_raw_fd(), 0) }; // READ|WRITE, SHARED
        if address as isize == -1 {
            return Err(io::Error::last_os_error());
        }
        // mmap retains the open file description: closing the fd alone does
        // not release this initialization flock while the mapping survives.
        if unsafe { flock(file.as_raw_fd(), 8) } < 0 {
            unsafe {
                munmap(address, 24);
            }
            return Err(io::Error::last_os_error());
        }
        Ok(Self(address.cast()))
    }
    fn state(&self) -> &EventState {
        unsafe { &*self.0 }
    }
    fn needs_dispatch(&self) -> bool {
        self.state().dispatched.load(Ordering::Acquire)
            != self.state().dirty.load(Ordering::Acquire)
            || now() >= self.state().reap_after.load(Ordering::Acquire)
    }
    fn request(&self) {
        self.state().dirty.fetch_add(1, Ordering::Release);
    }
    fn dispatched(&self, observed: u32) {
        self.state().dispatched.store(observed, Ordering::Release);
    }
    fn generation(&self) -> u32 {
        self.state().generation.load(Ordering::Acquire)
    }
    fn notify(&self) -> io::Result<()> {
        self.state().generation.fetch_add(1, Ordering::Release);
        if unsafe { syscall(SYS_FUTEX, self.0, 1i32, i32::MAX) } < 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }
    fn wait(&self, observed: u32) -> io::Result<()> {
        let timeout = Timespec {
            seconds: 1,
            nanos: 0,
        };
        if unsafe {
            syscall(
                SYS_FUTEX,
                self.0,
                0i32,
                observed,
                &timeout as *const Timespec,
            )
        } < 0
        {
            let error = io::Error::last_os_error();
            // EAGAIN means a notification preceded WAIT; EINTR and ETIMEDOUT
            // recheck legacy locks, crashed owners, and starvation aging.
            if !matches!(error.raw_os_error(), Some(11 | 4 | 110)) {
                return Err(error);
            }
        }
        Ok(())
    }
}
impl Drop for Wakeup {
    fn drop(&mut self) {
        unsafe {
            munmap(self.0.cast(), 24);
        }
    }
}
const EX: i32 = 2;
const NB: i32 = 4;
fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}
fn open(p: &Path) -> io::Result<File> {
    OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(p)
}
fn lock(f: &File, nonblock: bool) -> io::Result<bool> {
    if unsafe { flock(f.as_raw_fd(), EX | if nonblock { NB } else { 0 }) } == 0 {
        return Ok(true);
    }
    let e = io::Error::last_os_error();
    if nonblock && e.kind() == io::ErrorKind::WouldBlock {
        Ok(false)
    } else {
        Err(e)
    }
}
fn inherit(fd: RawFd) -> io::Result<()> {
    if unsafe { fcntl(fd, 2, 0) } < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}
fn positive(key: &str, default: usize) -> io::Result<usize> {
    match env::var(key) {
        Ok(s) => s
            .parse::<usize>()
            .ok()
            .filter(|n| *n > 0)
            .ok_or_else(|| io::Error::other(format!("{key} must be positive"))),
        Err(env::VarError::NotPresent) => Ok(default),
        Err(e) => Err(io::Error::other(e)),
    }
}
/// Like `positive`, but 0 is a value: `VIBEOS_HOST_JOB_LAND_RESERVE=0` disables
/// the landing reserve, the same as `VIBEOS_LAND_BOOT_RESERVE=0` for boots.
fn non_negative(key: &str, default: usize) -> io::Result<usize> {
    match env::var(key) {
        Ok(s) => s
            .parse::<usize>()
            .map_err(|_| io::Error::other(format!("{key} must be a non-negative integer"))),
        Err(env::VarError::NotPresent) => Ok(default),
        Err(e) => Err(io::Error::other(e)),
    }
}
fn quote(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c < ' ' => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}
struct Broker {
    root: PathBuf,
    slots: usize,
    links: usize,
    wakeup: Rc<Wakeup>,
}
/// Wake after releasing the mutex. Waking while locked lets a recipient lose
/// try_lock and sleep on the already-updated event until the aging timeout.
struct JobGuard {
    file: File,
    wakeup: Rc<Wakeup>,
}
impl JobGuard {
    fn take(path: &Path, wakeup: &Rc<Wakeup>, nonblock: bool) -> io::Result<Option<Self>> {
        let file = open(path)?;
        if lock(&file, nonblock)? {
            Ok(Some(Self {
                file,
                wakeup: Rc::clone(wakeup),
            }))
        } else {
            Ok(None)
        }
    }
}
impl Drop for JobGuard {
    fn drop(&mut self) {
        unsafe {
            flock(self.file.as_raw_fd(), 8);
        }
        if let Err(e) = self.wakeup.notify() {
            eprintln!("host-jobs: queue wake failed: {e}");
        }
    }
}
struct Ticket {
    path: PathBuf,
    file: File,
    kind: String,
    meta: String,
    wakeup: Rc<Wakeup>,
    mutex_path: PathBuf,
}
impl Drop for Ticket {
    fn drop(&mut self) {
        let result = (|| -> io::Result<()> {
            let _guard = JobGuard::take(&self.mutex_path, &self.wakeup, false)?.unwrap();
            match fs::remove_file(&self.path) {
                Ok(()) => {}
                Err(e) if e.kind() == io::ErrorKind::NotFound => {}
                Err(e) => return Err(e),
            }
            self.wakeup.request();
            Ok(())
        })();
        if let Err(e) = result {
            eprintln!("host-jobs: completion notification failed: {e}");
        }
    }
}
impl Broker {
    fn new() -> io::Result<Self> {
        let root = PathBuf::from(
            env::var_os("VIBEOS_HOSTLOCK_DIR")
                .unwrap_or_else(|| "/tmp/vibeos-qemu-locks/host".into()),
        );
        let slots = positive(
            "VIBEOS_HOST_JOB_SLOTS",
            thread::available_parallelism()?.get(),
        )?;
        let mem = fs::read_to_string("/proc/meminfo")?
            .lines()
            .find_map(|l| {
                l.strip_prefix("MemAvailable:")
                    .and_then(|s| s.split_whitespace().next())
                    .and_then(|s| s.parse::<usize>().ok())
            })
            .ok_or_else(|| io::Error::other("MemAvailable missing"))?;
        let links = positive(
            "VIBEOS_HOST_LINK_SLOTS",
            (slots / 4).max(1).min((mem / (2 * 1024 * 1024)).max(1)),
        )?;
        fs::create_dir_all(root.join("jobs"))?;
        let wakeup = Rc::new(Wakeup::new(&root)?);
        Ok(Self {
            root,
            slots,
            links,
            wakeup,
        })
    }
    fn mutex(&self) -> io::Result<JobGuard> {
        Ok(JobGuard::take(&self.root.join("jobs.lock"), &self.wakeup, false)?.unwrap())
    }
    fn ticket(&self, kind: &str, args: &[std::ffi::OsString]) -> io::Result<Ticket> {
        let _guard = self.mutex()?;
        let class = env::var("VIBEOS_JOB_CLASS").unwrap_or_else(|_| "interactive".into());
        let rank = rank_of(&class).ok_or_else(|| {
            io::Error::other("VIBEOS_JOB_CLASS must be interactive, integration, or background")
        })?;
        let born = now();
        let pid = std::process::id();
        let path = self
            .root
            .join("jobs")
            .join(format!("{born:020}-{rank}-{kind}-{pid}.json"));
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(&path)?;
        lock(&file, false)?;
        let meta=format!("\"pid\":{pid},\"created_ms\":{born},\"job_class\":{},\"kind\":{},\"cwd\":{},\"repo\":{},\"owner_pid\":{},\"launcher_pid\":{},\"run_dir\":{},\"cpu_limit\":{},\"link_limit\":{},\"argv\":[{}]",quote(&class),quote(kind),quote(&env::current_dir()?.to_string_lossy()),quote(&env::var("VIBEOS_JOB_REPO").unwrap_or_default()),quote(&env::var("VIBEOS_JOB_OWNER_PID").unwrap_or_default()),quote(&env::var("VIBEOS_JOB_LAUNCHER_PID").unwrap_or_default()),quote(&env::var("VIBEOS_JOB_RUN").unwrap_or_default()),self.slots,self.links,args.iter().map(|s|quote(&s.to_string_lossy())).collect::<Vec<_>>().join(","));
        write!(file, "{{{meta},\"state\":\"waiting\"}}")?;
        self.wakeup.request();
        Ok(Ticket {
            path,
            file,
            kind: kind.into(),
            meta,
            wakeup: Rc::clone(&self.wakeup),
            mutex_path: self.root.join("jobs.lock"),
        })
    }
    fn rows(&self) -> io::Result<Vec<(PathBuf, u64, u8, String, String)>> {
        let mut rows = Vec::new();
        for entry in fs::read_dir(self.root.join("jobs"))? {
            let p = entry?.path();
            if p.extension().is_none_or(|x| x != "json") {
                continue;
            }
            // Lock identity, not PID liveness, determines stale tickets.
            let f = match File::open(&p) {
                Ok(f) => f,
                Err(e) if e.kind() == io::ErrorKind::NotFound => continue,
                Err(e) => return Err(e),
            };
            if lock(&f, true)? {
                match fs::remove_file(&p) {
                    Ok(()) => self.wakeup.request(),
                    Err(e) if e.kind() == io::ErrorKind::NotFound => {}
                    Err(e) => return Err(e),
                }
                continue;
            }
            let stem = p.file_stem().unwrap().to_string_lossy();
            let parts: Vec<_> = stem.split('-').collect();
            if parts.len() != 4 {
                return Err(io::Error::other(format!("invalid ticket {}", p.display())));
            }
            let born = parts[0].parse().map_err(io::Error::other)?;
            let rank = parts[1].parse().map_err(io::Error::other)?;
            let kind = parts[2].to_string();
            let body = match fs::read_to_string(&p) {
                Ok(body) => body,
                Err(e) if e.kind() == io::ErrorKind::NotFound => continue,
                Err(e) => return Err(e),
            };
            rows.push((p, born, rank, kind, body));
        }
        Ok(rows)
    }
    fn legacy_budget(&self) -> io::Result<usize> {
        let count = positive("VIBEOS_HOST_CPU_LEASES", 4)?;
        let mut held = 0;
        for i in 0..count {
            let p = self.root.join(format!("cpuheavy-{i:02}"));
            if p.exists() && !lock(&open(&p)?, true)? {
                held += 1;
            }
        }
        Ok(self.slots * (count - held) / count)
    }
    fn slot_at(&self, kind: &str, index: usize) -> io::Result<Option<File>> {
        let f = open(&self.root.join(format!("job-{kind}-{index:03}")))?;
        if lock(&f, true)? {
            Ok(Some(f))
        } else {
            Ok(None)
        }
    }
    fn dispatch(&self) -> io::Result<()> {
        let requested = self.wakeup.state().dirty.load(Ordering::Acquire);
        let mut rows = self.rows()?;
        let occupied = |body: &str| {
            body.contains("\"state\":\"active\"") || body.contains("\"state\":\"granted\"")
        };
        let mut active_cpu = rows.iter().filter(|r| occupied(&r.4)).count();
        let mut active_links = rows
            .iter()
            .filter(|r| occupied(&r.4) && r.3 == "link")
            .count();
        let max_cpu = rows
            .iter()
            .map(|r| number(&r.4, "cpu_limit").unwrap_or(self.slots))
            .max()
            .unwrap_or(self.slots);
        let max_link = rows
            .iter()
            .map(|r| number(&r.4, "link_limit").unwrap_or(self.links))
            .max()
            .unwrap_or(self.links);
        let free = |kind: &str, count: usize| -> io::Result<Vec<usize>> {
            let key = format!("{kind}_slot");
            let mut free = Vec::new();
            for index in 0..count {
                if !rows
                    .iter()
                    .any(|r| occupied(&r.4) && number(&r.4, &key) == Some(index))
                    && self.slot_at(kind, index)?.is_some()
                {
                    free.push(index);
                }
            }
            Ok(free)
        };
        let mut cpus = free("cpu", max_cpu)?;
        let mut links = free("link", max_link)?;
        let leases = positive("VIBEOS_HOST_CPU_LEASES", 4)?;
        let mut held = 0;
        for i in 0..leases {
            let p = self.root.join(format!("cpuheavy-{i:02}"));
            if p.exists() && !lock(&open(&p)?, true)? {
                held += 1;
            }
        }
        // A landing gate (job_class=integration) is the fleet's critical path: every
        // queued lane waits on it, while a lane's own build only serves that lane's
        // NEXT submission. While any integration job is present, non-landing jobs may
        // hold at most `cpu_limit - reserve` slots between them, the same way the boot
        // pool keeps half its permits for landing. With no gate running, lanes use every
        // slot. A COUNT, not an index range: the first version fenced lanes into the low
        // slot NUMBERS, so landing jobs sitting in those numbers left lanes no slot while
        // the high ones idled (and deadlocked test_runner_process_contracts in the gate).
        let land_present = rows.iter().any(|r| is_landing(&r.4));
        let mut active_lanes = rows
            .iter()
            .filter(|r| occupied(&r.4) && !is_landing(&r.4))
            .count();
        let reserve = non_negative("VIBEOS_HOST_JOB_LAND_RESERVE", self.slots / 2)?;
        rows.sort_by_key(|r| (effective_rank(r.2, r.1, now()), r.1, r.0.clone()));
        for (path, _, _, kind, body) in rows {
            if !body.contains("\"state\":\"waiting\"") {
                continue;
            }
            let cpu_limit =
                number(&body, "cpu_limit").unwrap_or(self.slots) * (leases - held) / leases;
            let link_limit = number(&body, "link_limit").unwrap_or(self.links);
            if active_cpu >= cpu_limit || (kind == "link" && active_links >= link_limit) {
                continue;
            }
            let landing = is_landing(&body);
            if !may_start(active_lanes, cpu_limit, reserve, landing, land_present) {
                continue;
            }
            let Some(cpu_pos) = cpus.iter().position(|i| *i < cpu_limit) else {
                continue;
            };
            let link_pos = if kind == "link" {
                match links.iter().position(|i| *i < link_limit) {
                    Some(p) => Some(p),
                    None => continue,
                }
            } else {
                None
            };
            let cpu = cpus.remove(cpu_pos);
            let mut grant = body.replace("\"state\":\"waiting\"", "\"state\":\"granted\"");
            if grant.pop() != Some('}') {
                return Err(io::Error::other("invalid ticket JSON"));
            }
            grant.push_str(&format!(",\"cpu_slot\":{cpu}"));
            if let Some(p) = link_pos {
                let link = links.remove(p);
                grant.push_str(&format!(",\"link_slot\":{link}"));
                active_links += 1;
            }
            grant.push('}');
            fs::write(path, grant)?;
            active_cpu += 1;
            if !landing {
                active_lanes += 1;
            }
        }
        self.wakeup
            .state()
            .reap_after
            .store(now() + 1000, Ordering::Release);
        self.wakeup.dispatched(requested);
        Ok(())
    }
    fn admit(&self, t: &mut Ticket) -> io::Result<Vec<File>> {
        loop {
            let observed = self.wakeup.generation();
            // Only a generation change needs a dispatcher. Grantees inspect
            // their own ticket; everyone else sleeps without queue scans/locks.
            let granted = fs::read_to_string(&t.path)?.contains("\"state\":\"granted\"");
            if granted || self.wakeup.needs_dispatch() {
                if let Some(_guard) =
                    JobGuard::take(&self.root.join("jobs.lock"), &self.wakeup, true)?
                {
                    if self.wakeup.needs_dispatch() {
                        self.dispatch()?;
                    }
                    let body = fs::read_to_string(&t.path)?;
                    if body.contains("\"state\":\"granted\"") {
                        let cpu = number(&body, "cpu_slot")
                            .ok_or_else(|| io::Error::other("grant missing CPU slot"))?;
                        if let Some(cpu_lock) = self.slot_at("cpu", cpu)? {
                            let mut locks = vec![cpu_lock];
                            if t.kind == "link" {
                                let link = number(&body, "link_slot")
                                    .ok_or_else(|| io::Error::other("grant missing link slot"))?;
                                locks.push(self.slot_at("link", link)?.ok_or_else(|| {
                                    io::Error::other("reserved link slot occupied")
                                })?);
                            }
                            let active =
                                body.replace("\"state\":\"granted\"", "\"state\":\"active\"");
                            fs::write(&t.path, &active)?;
                            // The cached-command guardian preserves slot metadata.
                            t.meta = active
                                .trim_start_matches('{')
                                .trim_end_matches('}')
                                .replace(",\"state\":\"active\"", "");
                            return Ok(locks);
                        }
                    }
                }
            }
            self.wakeup.wait(observed)?;
        }
    }
    fn status(&self) -> io::Result<()> {
        let _guard = self.mutex()?;
        let rows = self.rows()?;
        println!(
            "{{\"cpu_slots\":{},\"link_slots\":{},\"legacy_adjusted_cpu_slots\":{},\"jobs\":[{}]}}",
            self.slots,
            self.links,
            self.legacy_budget()?,
            rows.iter()
                .map(|r| r.4.clone())
                .collect::<Vec<_>>()
                .join(",")
        );
        Ok(())
    }
}
fn number(body: &str, key: &str) -> Option<usize> {
    let value = body.split_once(&format!("\"{key}\":"))?.1;
    value
        .split(|c: char| !c.is_ascii_digit())
        .next()?
        .parse()
        .ok()
}
/// Dispatch order by job class. A landing gate (integration) ranks FIRST: it is
/// the merge queue's serialized critical path, and every queued lane waits on it,
/// while an interactive build only serves its own lane's next submission. The
/// reverse order (interactive first, 2026-07..10) stretched gates from ~530s to
/// 1500-2100s whenever a few lanes were building, which the queue then paid per
/// landing. Aging (`effective_rank`) still promotes every class after 120s, so
/// a lane is delayed by a gate, never starved by one.
fn rank_of(class: &str) -> Option<u8> {
    match class {
        "integration" => Some(0),
        "interactive" => Some(1),
        "background" => Some(2),
        _ => None,
    }
}
fn is_landing(body: &str) -> bool {
    body.contains("\"job_class\":\"integration\"")
}
/// May a job start now? A landing job always may (the slot count is checked by the
/// caller). While a landing job is present, non-landing jobs share at most
/// `cpu_limit - reserve` slots; lanes already over that simply finish. The reserve
/// never takes a lane's last slot, as the boot pool's land_boot_reserve does, so a
/// gate delays a lane without ever stopping it.
fn may_start(
    active_lanes: usize,
    cpu_limit: usize,
    reserve: usize,
    landing: bool,
    land_present: bool,
) -> bool {
    let reserve = reserve.min(cpu_limit.saturating_sub(1));
    landing || !land_present || active_lanes < cpu_limit - reserve
}
fn effective_rank(rank: u8, born: u64, current: u64) -> u8 {
    if current.saturating_sub(born) >= 120_000 {
        0
    } else {
        rank
    }
}
fn run() -> io::Result<i32> {
    let mut args: Vec<_> = env::args_os().skip(1).collect();
    if args.first().is_some_and(|s| s == "--status") {
        Broker::new()?.status()?;
        return Ok(0);
    }
    let mut cached = false;
    let kind = if args.first().is_some_and(|s| s == "--kind") {
        if args.len() < 4 || args[2] != "--" {
            return Err(io::Error::other(
                "usage: host-jobs --kind compile|link -- command [args]",
            ));
        }
        let k = args[1].to_string_lossy().into_owned();
        args.drain(..3);
        k
    } else {
        if let Some(wrapper) = env::var_os("VIBEOS_INNER_RUSTC_WRAPPER").filter(|s| !s.is_empty()) {
            cached = true;
            args.insert(0, wrapper);
        }
        rust_kind(&args).into()
    };
    if !matches!(kind.as_str(), "compile" | "link") || args.is_empty() {
        return Err(io::Error::other("expected compile/link kind and command"));
    }
    cached |= Path::new(&args[0])
        .file_name()
        .is_some_and(|n| n == "sccache" || n == "ccache");
    let mut cmd = Command::new(&args[0]);
    cmd.args(&args[1..]);
    if env::var_os("VIBEOS_HOST_JOB_HELD").is_some() {
        return Ok(code(cmd.status()?));
    }
    let b = Broker::new()?;
    let mut ticket = b.ticket(&kind, &args)?;
    let locks = b.admit(&mut ticket)?;
    cmd.env("VIBEOS_HOST_JOB_HELD", "1");
    if cached {
        // Cache daemons retain inherited descriptors. A separate guardian keeps
        // CLOEXEC leases alive while the cache client runs, even if our parent
        // is killed. No lease descriptor reaches the persistent cache server.
        let guardian = unsafe { fork() };
        if guardian < 0 {
            return Err(io::Error::last_os_error());
        }
        if guardian == 0 {
            let result = (|| -> io::Result<i32> {
                let _guard = b.mutex()?;
                use std::io::Seek;
                ticket.file.set_len(0)?;
                ticket.file.rewind()?;
                write!(
                    ticket.file,
                    "{{{},\"state\":\"active\",\"guardian_pid\":{}}}",
                    ticket.meta,
                    std::process::id()
                )?;
                drop(_guard);
                Ok(code(cmd.status()?))
            })();
            let exit = match result {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("host-jobs guardian: {e}");
                    125
                }
            };
            // _exit bypasses destructors: release leases before notifying
            // waiters, including when the original supervisor was killed.
            drop(locks);
            drop(ticket);
            unsafe { _exit(exit) }
        }
        let mut status = 0;
        loop {
            if unsafe { waitpid(guardian, &mut status, 0) } >= 0 {
                break;
            }
            let e = io::Error::last_os_error();
            if e.kind() != io::ErrorKind::Interrupted {
                return Err(e);
            }
        }
        Ok(code(std::process::ExitStatus::from_raw(status)))
    } else {
        for f in locks.iter().chain(std::iter::once(&ticket.file)) {
            inherit(f.as_raw_fd())?;
        }
        Ok(code(cmd.status()?))
    }
}
fn rust_kind(args: &[std::ffi::OsString]) -> &'static str {
    let mut crate_type = None;
    let mut emits_link = false;
    for (i, arg) in args.iter().enumerate() {
        let arg = arg.to_string_lossy();
        if arg == "--crate-type" {
            crate_type = args.get(i + 1).map(|s| s.to_string_lossy().into_owned());
        }
        if let Some(v) = arg.strip_prefix("--crate-type=") {
            crate_type = Some(v.into());
        }
        let emit = arg.strip_prefix("--emit=").map(str::to_owned).or_else(|| {
            if arg == "--emit" {
                args.get(i + 1).map(|s| s.to_string_lossy().into_owned())
            } else {
                None
            }
        });
        emits_link |=
            emit.is_some_and(|v| v.split(',').any(|s| s.split('=').next() == Some("link")));
    }
    if emits_link
        && crate_type
            .as_deref()
            .unwrap_or("bin")
            .split(',')
            .any(|s| matches!(s, "bin" | "proc-macro" | "cdylib" | "dylib"))
    {
        "link"
    } else {
        "compile"
    }
}
fn code(s: std::process::ExitStatus) -> i32 {
    s.code().unwrap_or_else(|| 128 + s.signal().unwrap_or(1))
}
fn main() -> ExitCode {
    match run() {
        Ok(c) => ExitCode::from(c.clamp(0, 255) as u8),
        Err(e) => {
            eprintln!("host-jobs: {e}");
            ExitCode::from(125)
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn aging() {
        assert_eq!(effective_rank(2, 0, 119999), 2);
        assert_eq!(effective_rank(2, 0, 120000), 0);
    }
    #[test]
    fn landing_ranks_first_and_unknown_classes_are_refused() {
        assert_eq!(rank_of("integration"), Some(0));
        assert_eq!(rank_of("interactive"), Some(1));
        assert_eq!(rank_of("background"), Some(2));
        assert_eq!(rank_of("urgent"), None);
    }
    #[test]
    fn lanes_share_a_count_of_slots_while_a_gate_is_present() {
        // 18 slots, 9 reserved: lanes may hold 9 between them while a gate is present.
        assert!(may_start(8, 18, 9, false, true));
        assert!(!may_start(9, 18, 9, false, true));
        // A gate job is never held back by the lane count.
        assert!(may_start(17, 18, 9, true, true));
        // No gate present: lanes use every slot.
        assert!(may_start(17, 18, 9, false, false));
        // No reserve takes a lane's LAST slot.
        assert!(may_start(0, 18, 40, false, true));
        assert!(!may_start(1, 18, 40, false, true));
        // Two slots, one reserved, gate jobs holding BOTH and one finishing: the lane
        // gets the freed slot whichever number it is (the index-range version did not).
        assert!(may_start(0, 2, 1, false, true));
        // A one-slot broker and a reserve of 0 both mean "no reserve".
        assert!(may_start(0, 1, 0, false, true));
        assert!(may_start(17, 18, 0, false, true));
    }
    #[test]
    fn json() {
        assert_eq!(quote("a\n\"\\"), "\"a\\n\\\"\\\\\"");
    }
}
