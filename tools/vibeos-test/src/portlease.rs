//! Explicit native resource lifetime for legacy launch-handshake clients.
//! All callers share the launcher's admission::port_handshake inode/protocol.
use crate::{
    admission::{self, HeldLock},
    Error, Result,
};
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, HashMap},
    path::Path,
    time::{Duration, Instant},
};
#[derive(Default)]
pub struct PortLeases {
    next: u64,
    held: HashMap<u64, HeldLock>,
}
impl PortLeases {
    pub fn handle(&mut self, root: &Path, request: &Value) -> Result<Value> {
        let a = request.get("kwargs").unwrap_or(request);
        match request["op"].as_str().unwrap_or("") {
            "portlease_acquire" => {
                let seconds = match a.get("timeout") {
                    None => 120.,
                    Some(v) => v.as_f64().ok_or_else(|| {
                        Error::new("QEMU port handshake timeout must be finite and nonnegative")
                            .with_kind("ValueError")
                    })?,
                };
                if !seconds.is_finite() || seconds < 0. {
                    return Err(Error::new(
                        "QEMU port handshake timeout must be finite and nonnegative",
                    )
                    .with_kind("ValueError"));
                }
                let timeout = Duration::try_from_secs_f64(seconds).map_err(|_| {
                    Error::new("QEMU port handshake timeout is not representable")
                        .with_kind("ValueError")
                })?;
                Instant::now().checked_add(timeout).ok_or_else(|| {
                    Error::new("QEMU port handshake timeout is not representable")
                        .with_kind("ValueError")
                })?;
                let env: BTreeMap<String, String> = a
                    .get("env")
                    .cloned()
                    .map(serde_json::from_value)
                    .transpose()?
                    .unwrap_or_else(|| std::env::vars().collect());
                let owner = a
                    .get("owner_pid")
                    .and_then(Value::as_u64)
                    .unwrap_or(unsafe { libc::getppid() } as u64);
                let cwd = a
                    .get("cwd")
                    .and_then(Value::as_str)
                    .map(Path::new)
                    .unwrap_or(root);
                let held = admission::port_handshake_owned(&env, cwd, timeout, owner)?;
                self.next = self
                    .next
                    .checked_add(1)
                    .ok_or("Port lease resource id overflow")?;
                let id = self.next;
                let path = held.path.clone();
                self.held.insert(id, held);
                Ok(json!({"id":id,"path":path}))
            }
            "portlease_release" => {
                let id = a["id"]
                    .as_u64()
                    .ok_or("Missing port handshake resource id")?;
                self.held.remove(&id);
                Ok(Value::Null)
            }
            _ => Err(Error::new("Unknown port lease resource operation")),
        }
    }
    pub fn close_all(&mut self) {
        self.held.clear();
    }
}
