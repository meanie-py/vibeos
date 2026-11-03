//! A controller retains the runner's unreaped PID while stopping its private group.
//! Detached services (for example sccache servers) keep their separate lifetimes.
use crate::{Error, Result};
use serde_json::{json, Value};
use std::{
    fs::{self, File},
    io::{Read, Seek, SeekFrom, Write},
    os::{
        fd::{AsRawFd, FromRawFd},
        unix::process::{CommandExt, ExitStatusExt},
    },
    process::Command,
    time::{Duration, Instant},
};
fn mapped(code: i32) -> u8 {
    if code < 0 {
        (128_i64 - i64::from(code)).min(255) as u8
    } else {
        code.min(255) as u8
    }
}
fn descriptor_flag(fd: i32, inherited: bool) -> Result<()> {
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFD) };
    if flags < 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    let flags = if inherited {
        flags & !libc::FD_CLOEXEC
    } else {
        flags | libc::FD_CLOEXEC
    };
    if unsafe { libc::fcntl(fd, libc::F_SETFD, flags) } < 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    Ok(())
}
fn write_receipt(file: &mut File, value: &Value) -> Result<()> {
    file.set_len(0)?;
    file.seek(SeekFrom::Start(0))?;
    serde_json::to_writer(&mut *file, value)?;
    file.write_all(b"\n")?;
    file.sync_all()?;
    Ok(())
}
fn active_group(group: u32) -> Result<bool> {
    for e in fs::read_dir("/proc")? {
        let e = e?;
        let Some(pid) = e.file_name().to_str().and_then(|s| s.parse::<u32>().ok()) else {
            continue;
        };
        if pid == group {
            continue;
        }
        let stat = match fs::read_to_string(e.path().join("stat")) {
            Ok(s) => s,
            Err(e)
                if e.kind() == std::io::ErrorKind::NotFound
                    || e.raw_os_error() == Some(libc::ESRCH) =>
            {
                continue
            }
            Err(e) => return Err(e.into()),
        };
        let fields: Vec<_> = stat
            .rsplit_once(')')
            .ok_or("invalid process stat while cleaning owned group")?
            .1
            .split_whitespace()
            .collect();
        if fields.get(2).and_then(|s| s.parse::<u32>().ok()) == Some(group)
            && !matches!(fields.first(), Some(&"Z" | &"X"))
        {
            return Ok(true);
        }
    }
    Ok(false)
}
fn stop_group(group: u32) -> Result<()> {
    // The runner is still an unreaped child: its numeric PID cannot be reused.
    // Never enumerate-and-kill arbitrary PIDs, and never select an outer session.
    let start = Instant::now();
    while active_group(group)? {
        let signal = if start.elapsed() < Duration::from_secs(5) {
            libc::SIGTERM
        } else {
            libc::SIGKILL
        };
        if unsafe { libc::kill(-(group as i32), signal) } != 0 {
            let e = std::io::Error::last_os_error();
            if e.raw_os_error() != Some(libc::ESRCH) {
                return Err(e.into());
            }
        }
        if start.elapsed() > Duration::from_secs(10) {
            return Err(Error::new(format!(
                "owned process group {group} did not stop after SIGKILL"
            )));
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    Ok(())
}
/// Controller used by dedicated command workers; stdout/stderr stay inherited.
pub fn controller(argv: &[String], receipt: &mut File) -> Result<u8> {
    if argv.is_empty() {
        return Err("owned command requires argv".into());
    }
    crate::artifact_lifetime::watch_owner(unsafe { libc::getppid() } as u32)?;
    let mode = unsafe { libc::fcntl(receipt.as_raw_fd(), libc::F_GETFL) };
    if mode < 0 || mode & libc::O_ACCMODE != libc::O_RDWR {
        return Err(Error::new(
            "owned command receipt must be open for read/write",
        ));
    }
    descriptor_flag(receipt.as_raw_fd(), true)?;
    let mut cmd = Command::new(std::env::current_exe()?);
    let owner = std::process::id();
    cmd.args([
        "--root",
        std::env::current_dir()?
            .to_str()
            .ok_or("non-UTF8 command directory")?,
        "--client-pid",
        &owner.to_string(),
        "owned-command-fd",
        &receipt.as_raw_fd().to_string(),
    ])
    .args(argv);
    unsafe {
        cmd.pre_exec(move || {
            if libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGTERM) != 0 {
                return Err(std::io::Error::last_os_error());
            }
            if libc::getppid() != owner as i32 {
                libc::_exit(125);
            }
            if libc::setsid() < 0 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
    let mut child = cmd.spawn()?;
    let group = child.id();
    // WNOWAIT retains the zombie runner until group cleanup is complete, closing
    // the PID/group reuse race that a normal Child::wait would introduce here.
    let observed = loop {
        let mut info: libc::siginfo_t = unsafe { std::mem::zeroed() };
        if unsafe { libc::waitid(libc::P_PID, group, &mut info, libc::WEXITED | libc::WNOWAIT) }
            == 0
        {
            break Ok(());
        }
        let e = std::io::Error::last_os_error();
        if e.kind() != std::io::ErrorKind::Interrupted {
            break Err(e);
        }
    };
    let cleanup = stop_group(group);
    if observed.is_err() || cleanup.is_err() {
        unsafe { libc::kill(-(group as i32), libc::SIGKILL) };
    }
    let status = child.wait()?;
    observed?;
    cleanup?;
    descriptor_flag(receipt.as_raw_fd(), false)?;
    receipt.seek(SeekFrom::Start(0))?;
    let mut bytes = Vec::new();
    receipt.read_to_end(&mut bytes)?;
    let raw = serde_json::from_slice::<Value>(&bytes)
        .ok()
        .and_then(|v| v["exit_code"].as_i64())
        .and_then(|n| i32::try_from(n).ok());
    let worker = status
        .code()
        .unwrap_or_else(|| -status.signal().unwrap_or(1));
    if status.code().is_some() && raw.is_some_and(|raw| mapped(raw) as i32 == worker) {
        return Ok(mapped(worker));
    }
    write_receipt(
        receipt,
        &json!({"worker_exit_code":worker,"child_status_unknown":true}),
    )?;
    Ok(if worker == 0 { 1 } else { mapped(worker) })
}
/// Internal runner: a private session leader, with its receipt excluded from exec.
pub fn runner(argv: &[String]) -> Result<u8> {
    let (fd, argv) = argv
        .split_first()
        .ok_or("owned runner requires receipt fd and argv")?;
    let fd: i32 = fd
        .parse()
        .map_err(|_| Error::new("invalid owned receipt fd"))?;
    if fd < 3 || argv.is_empty() {
        return Err("owned runner requires fd >=3 and argv".into());
    }
    if unsafe { libc::getpgrp() } != std::process::id() as i32
        || unsafe { libc::getsid(0) } != std::process::id() as i32
    {
        return Err("owned runner must lead its private session".into());
    }
    let mut file = unsafe { File::from_raw_fd(fd) };
    if !file.metadata()?.is_file() {
        return Err("owned receipt fd must be regular".into());
    }
    descriptor_flag(fd, false)?;
    let result =
        crate::artifact_executor::supervised_group_code(Command::new(&argv[0]).args(&argv[1..]));
    // This runner is short-lived. Keep cleanup signals from killing the group
    // leader before it can stop descendants whose immediate parent already died.
    let mut ignore: libc::sigaction = unsafe { std::mem::zeroed() };
    ignore.sa_sigaction = libc::SIG_IGN;
    unsafe { libc::sigemptyset(&mut ignore.sa_mask) };
    if unsafe { libc::sigaction(libc::SIGTERM, &ignore, std::ptr::null_mut()) } != 0
        || unsafe { libc::sigaction(libc::SIGINT, &ignore, std::ptr::null_mut()) } != 0
    {
        return Err(std::io::Error::last_os_error().into());
    }
    stop_group(std::process::id())?;
    let (code, launch) = match result {
        Ok(code) => (code, None),
        Err(e) if e.kind.as_deref() == Some("LaunchError") => (127, Some(e.message)),
        Err(e) => return Err(e),
    };
    let mut value = json!({"exit_code":code});
    if let Some(error) = launch {
        eprintln!("{error}");
        value["launch_error"] = json!(error);
    }
    write_receipt(&mut file, &value)?;
    Ok(mapped(code))
}
