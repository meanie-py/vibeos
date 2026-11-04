//! Process-incarnation lifetime observation shared by artifact ownership and supervision.
use crate::{Error, Result};
use serde_json::{json, Value};
use std::{
    collections::BTreeSet,
    fs,
    os::fd::{AsRawFd, FromRawFd, OwnedFd},
    sync::{Mutex, OnceLock},
};
pub fn start_identity(pid: u32) -> Option<Value> {
    let text = fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    let fields: Vec<_> = text.rsplit_once(')')?.1.split_whitespace().collect();
    Some(
        json!({"boot_id":fs::read_to_string("/proc/sys/kernel/random/boot_id").ok()?.trim(),"start_ticks":*fields.get(19)?}),
    )
}
type Identity = (u32, String, String);
fn watched() -> &'static Mutex<BTreeSet<Identity>> {
    static WATCHED: OnceLock<Mutex<BTreeSet<Identity>>> = OnceLock::new();
    WATCHED.get_or_init(Default::default)
}
pub fn watch_owner(pid: u32) -> Result<()> {
    watch_owner_checked(pid, None)
}
/// A pidfd never follows numeric PID reuse; expected startup identity closes the launch race.
pub fn watch_owner_checked(pid: u32, expected: Option<&Value>) -> Result<()> {
    if pid == 0 || pid == std::process::id() {
        return Err(Error::new(
            "native lifetime owner must be another live process",
        ));
    }
    let identity = start_identity(pid).ok_or("native lifetime owner is absent")?;
    if expected.is_some_and(|value| value != &identity) {
        return Err(Error::new("native lifetime owner PID was reused"));
    }
    let raw = unsafe { libc::syscall(libc::SYS_pidfd_open, pid as libc::pid_t, 0) };
    if raw < 0 {
        return Err(Error::new(format!(
            "cannot observe native owner process: {}",
            std::io::Error::last_os_error()
        )));
    }
    let fd = unsafe { OwnedFd::from_raw_fd(raw as i32) };
    if start_identity(pid).as_ref() != Some(&identity) {
        return Err(Error::new(
            "native lifetime owner changed while observing it",
        ));
    }
    let mut poll = libc::pollfd {
        fd: fd.as_raw_fd(),
        events: libc::POLLIN,
        revents: 0,
    };
    let rc = unsafe { libc::poll(&mut poll, 1, 0) };
    if rc != 0 {
        return Err(Error::new("native lifetime owner has already exited"));
    }
    let key = (
        pid,
        identity["boot_id"]
            .as_str()
            .ok_or("invalid owner boot identity")?
            .to_owned(),
        identity["start_ticks"]
            .as_str()
            .ok_or("invalid owner start identity")?
            .to_owned(),
    );
    let mut guard = watched()
        .lock()
        .map_err(|_| Error::new("native owner registry poisoned"))?;
    if guard.contains(&key) {
        return Ok(());
    }
    guard.insert(key.clone());
    let spawned = std::thread::Builder::new()
        .name("artifact-owner".into())
        .spawn(move || {
            let mut poll = libc::pollfd {
                fd: fd.as_raw_fd(),
                events: libc::POLLIN,
                revents: 0,
            };
            loop {
                let rc = unsafe { libc::poll(&mut poll, 1, -1) };
                if rc > 0
                    || (rc < 0
                        && std::io::Error::last_os_error().kind()
                            != std::io::ErrorKind::Interrupted)
                {
                    if let Ok(mut watched) = watched().lock() {
                        watched.remove(&key);
                    }
                    // SIGTERM permits an active executor to stop its exact owned worker group.
                    unsafe { libc::kill(libc::getpid(), libc::SIGTERM) };
                    break;
                }
            }
        });
    if let Err(error) = spawned {
        guard.remove(&(
            pid,
            identity["boot_id"].as_str().unwrap().to_owned(),
            identity["start_ticks"].as_str().unwrap().to_owned(),
        ));
        return Err(Error::new(format!(
            "cannot supervise native owner lifetime: {error}"
        )));
    }
    Ok(())
}
