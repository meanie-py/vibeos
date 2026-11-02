//! Dedicated local sessions hold OS leases/action locks while compatibility callers run callbacks.
use crate::{artifact_store as s, Error, Result};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    io::{BufRead, Write},
    path::PathBuf,
};
fn cv(e: s::StoreError) -> Error {
    Error::classified(e.kind, e.message)
}
fn val<T: Serialize>(v: T) -> Result<Value> {
    Ok(serde_json::to_value(v)?)
}
struct Args<'a>(&'a Value);
impl Args<'_> {
    fn req<T: DeserializeOwned>(&self, key: &str) -> Result<T> {
        Ok(serde_json::from_value(
            self.0
                .get(key)
                .cloned()
                .ok_or_else(|| Error::new(format!("missing argument {key}")))?,
        )?)
    }
    fn opt<T: DeserializeOwned>(&self, key: &str, default: T) -> Result<T> {
        match self.0.get(key) {
            Some(v) if !v.is_null() => Ok(serde_json::from_value(v.clone())?),
            _ => Ok(default),
        }
    }
    fn store(&self) -> Result<s::Store> {
        s::Store::new(&self.req::<PathBuf>("root")?).map_err(cv)
    }
    fn session(&self, store: &s::Store) -> Result<s::Session> {
        store
            .attach_session(&self.req::<String>("token")?)
            .map_err(cv)
    }
}
pub fn stateless(request: &Value) -> Result<Value> {
    let op = request["op"].as_str().ok_or("missing store operation")?;
    let a = Args(&request["kwargs"]);
    match op {
        "artifact_store_canonical" => Ok(json!(String::from_utf8(
            s::canonical(&a.req::<Value>("value")?).map_err(cv)?
        )
        .map_err(|e| Error::new(e.to_string()))?)),
        "artifact_store_action_key" => {
            val(s::action_key(&a.req::<Value>("descriptor")?).map_err(cv)?)
        }
        "artifact_store_key" => val(s::key(&a.req::<String>("value")?).map_err(cv)?),
        "artifact_store_name" => val(s::name(&a.req::<String>("value")?).map_err(cv)?),
        "artifact_store_start_identity" => val(s::start_identity(a.req("pid")?)),
        "artifact_store_owner" => {
            let pid: u32 = a.req("pid")?;
            let identity =
                s::start_identity(pid).ok_or("Linux process start identity unavailable")?;
            Ok(json!({"pid":pid,"start_identity":identity}))
        }
        "artifact_store_owner_alive" => val(s::owner_alive(&a.req::<Value>("owner")?).map_err(cv)?),
        "artifact_store_handle_blob_path" => {
            val(a.req::<s::Handle>("handle")?.blob_path().map_err(cv)?)
        }
        "artifact_store_read" => val(s::read(&a.req::<PathBuf>("path")?).map_err(cv)?),
        "artifact_store_read_json" => s::read_json(&a.req::<PathBuf>("path")?).map_err(cv),
        "artifact_store_digest_file" => {
            val(s::digest_file(&a.req::<PathBuf>("path")?).map_err(cv)?)
        }
        "artifact_store_replace_json" => {
            s::replace_json(&a.req::<PathBuf>("path")?, &a.req::<Value>("value")?).map_err(cv)?;
            Ok(Value::Null)
        }
        "artifact_store_write_new" => {
            s::write_new(&a.req::<PathBuf>("path")?, &a.req::<Vec<u8>>("payload")?).map_err(cv)?;
            Ok(Value::Null)
        }
        "artifact_store_rename_noreplace" => val(s::rename_noreplace(
            &a.req::<PathBuf>("source")?,
            &a.req::<PathBuf>("destination")?,
        )
        .map_err(cv)?),
        "artifact_store_remove_private_tree" => {
            s::remove_private_tree(&a.req::<PathBuf>("path")?).map_err(cv)?;
            Ok(Value::Null)
        }
        "artifact_store_path" => {
            let store = a.store()?;
            let key: String = a.req("key")?;
            let kind: String = a.req("kind")?;
            val(match kind.as_str() {
                "action" => store.action_path(&key),
                "action_lock" => store.action_lock(&key),
                "blob" => store.blob(&key),
                "lease" => store.lease_path(&key),
                "lease_lock" => store.lease_lock(&key),
                "conflict" => store.conflict_path(&key),
                _ => return Err(Error::new("unknown store path kind")),
            }
            .map_err(cv)?)
        }
        "artifact_store_ref_path" => {
            let store = a.store()?;
            val(store
                .ref_path(&a.req::<String>("lane")?, &a.req::<String>("consumer")?)
                .map_err(cv)?)
        }
        "artifact_store_ref_id" => {
            let store = a.store()?;
            val(store
                .ref_id(&a.req::<String>("lane")?, &a.req::<String>("consumer")?)
                .map_err(cv)?)
        }
        _ => Err(Error::new(format!(
            "unknown stateless store operation {op}"
        ))),
    }
}
struct Wire<R: BufRead, W: Write> {
    input: R,
    output: W,
}
impl<R: BufRead, W: Write> Wire<R, W> {
    fn read(&mut self) -> Result<Option<Value>> {
        let mut line = String::new();
        if self.input.read_line(&mut line)? == 0 {
            return Ok(None);
        }
        Ok(Some(serde_json::from_str(&line)?))
    }
    fn send(&mut self, value: &Value) -> Result<()> {
        writeln!(self.output, "{value}")?;
        self.output.flush()?;
        Ok(())
    }
    fn callback(&mut self, kind: &str, args: Value) -> s::StoreResult<Value> {
        self.send(&json!({"callback":{"kind":kind,"arguments":args}}))
            .map_err(|e| s::StoreError::new("StoreError", e.message))?;
        let response = self
            .read()
            .map_err(|e| s::StoreError::new("StoreError", e.message))?
            .ok_or_else(|| s::StoreError::new("StoreError", "callback client disconnected"))?;
        if let Some(error) = response.get("callback_error") {
            return Err(s::StoreError::new(
                error["kind"].as_str().unwrap_or("StoreError"),
                error["message"].as_str().unwrap_or("callback failed"),
            ));
        }
        response
            .get("callback_result")
            .cloned()
            .ok_or_else(|| s::StoreError::new("StoreError", "invalid callback result"))
    }
}
#[derive(Default)]
struct State {
    sessions: BTreeMap<String, s::Session>,
    guards: BTreeMap<String, s::ConsumerGuard>,
    transactions: BTreeMap<String, s::BuildTransaction>,
    next: u64,
}
impl State {
    fn id(&mut self) -> String {
        self.next += 1;
        self.next.to_string()
    }
    fn abandon(&mut self) {
        for session in self.sessions.values_mut() {
            session.abandon();
        }
        self.sessions.clear();
    }
    fn dispatch<R: BufRead, W: Write>(
        &mut self,
        request: &Value,
        wire: &mut Wire<R, W>,
    ) -> Result<Value> {
        let op = request["op"]
            .as_str()
            .ok_or("missing store service operation")?;
        let a = Args(&request["kwargs"]);
        if op.starts_with("artifact_store_") {
            return stateless(request);
        }
        let store = a.store()?;
        match op {
            "init" => Ok(json!({"root":store.root,"base":store.base})),
            "session_open" => {
                let owner = a.opt::<Option<Value>>("owner", None)?;
                let session = match owner {
                    Some(owner) => store.session_with_owner(owner),
                    None => store.session(),
                }
                .map_err(cv)?;
                let response = json!({"identifier":session.identifier,"token":session.token,"owner":session.owner});
                self.sessions.insert(session.token.clone(), session);
                Ok(response)
            }
            "session_close" => {
                let token: String = a.req("token")?;
                let mut session = self.sessions.remove(&token).ok_or_else(|| {
                    Error::classified("SessionError", "owned session not on this connection")
                })?;
                session.close().map_err(cv)?;
                Ok(Value::Null)
            }
            "attach" => {
                let session = a.session(&store)?;
                Ok(
                    json!({"identifier":session.identifier,"token":session.token,"owner":session.owner}),
                )
            }
            "lookup" => {
                let session = a.session(&store)?;
                val(store
                    .lookup(&a.req::<String>("key")?, &session)
                    .map_err(cv)?)
            }
            "inspect" => {
                let session = a
                    .opt::<Option<String>>("token", None)?
                    .map(|token| store.attach_session(&token).map_err(cv))
                    .transpose()?;
                store
                    .inspect_manifest(&a.req::<String>("key")?, session.as_ref())
                    .map_err(cv)
            }
            "begin" | "begin_repair" => {
                let session = a.session(&store)?;
                let begin = if op == "begin_repair" {
                    s::BeginBuild::Build(
                        store
                            .begin_repair(&a.req::<String>("key")?, &session)
                            .map_err(cv)?,
                    )
                } else {
                    let options = s::BuildOptions {
                        verify: a.opt("verify", false)?,
                        verify_once_per_session: a.opt("verify_once_per_session", false)?,
                        require_verified: a.opt("require_verified", false)?,
                    };
                    store
                        .begin_build(&a.req::<Value>("descriptor")?, &session, &options)
                        .map_err(cv)?
                };
                match begin {
                    s::BeginBuild::Hit(handle) => Ok(json!({"handle":handle})),
                    s::BeginBuild::Build(tx) => {
                        let id = self.id();
                        let response = json!({"transaction":id,"scratch":tx.scratch});
                        self.transactions.insert(id, tx);
                        Ok(response)
                    }
                }
            }
            "finish" => {
                let session = a.session(&store)?;
                let tx = self
                    .transactions
                    .remove(&a.req::<String>("transaction")?)
                    .ok_or("build transaction no longer exists")?;
                let built: s::BuildResult = a.req("built")?;
                val(store.finish_build(tx, built, &session).map_err(cv)?)
            }
            "abort" => {
                let tx = self
                    .transactions
                    .remove(&a.req::<String>("transaction")?)
                    .ok_or("build transaction no longer exists")?;
                store
                    .abort_build(
                        tx,
                        &a.req::<String>("error_kind")?,
                        &a.req::<String>("message")?,
                    )
                    .map_err(cv)?;
                Ok(Value::Null)
            }
            "bind" => {
                let session = a.session(&store)?;
                val(store
                    .bind(
                        &a.req::<String>("lane")?,
                        &a.req::<String>("consumer")?,
                        &a.req::<String>("key")?,
                        &session,
                    )
                    .map_err(cv)?)
            }
            "guard_open" => {
                let guard = store
                    .consumer_guard(&a.req::<String>("lane")?, &a.req::<String>("consumer")?)
                    .map_err(cv)?;
                let id = self.id();
                self.guards.insert(id.clone(), guard);
                val(id)
            }
            "guard_close" => {
                let mut guard = self
                    .guards
                    .remove(&a.req::<String>("guard")?)
                    .ok_or("consumer guard no longer exists")?;
                guard.close();
                Ok(Value::Null)
            }
            "release_ref" => {
                let guard = self
                    .guards
                    .get(&a.req::<String>("guard")?)
                    .ok_or("consumer guard not on this connection")?;
                store
                    .release_ref(
                        &a.req::<String>("lane")?,
                        &a.req::<String>("consumer")?,
                        guard,
                    )
                    .map_err(cv)?;
                Ok(Value::Null)
            }
            "list_refs" => val(store.list_refs(&a.req::<String>("lane")?).map_err(cv)?),
            "restore_refs" => {
                let session = a.session(&store)?;
                val(store
                    .restore_refs(&a.req::<String>("lane")?, &session)
                    .map_err(cv)?)
            }
            "status" => store.status().map_err(cv),
            "gc" => {
                let apply = a.opt("apply", false)?;
                let grace = a.opt("grace_seconds", 3600.0)?;
                if a.opt("ref_alive", false)? {
                    let mut callback = |reference: &Value| {
                        wire.callback("ref_alive", json!({"reference":reference}))
                            .and_then(|v| {
                                v.as_bool().ok_or_else(|| {
                                    s::StoreError::new(
                                        "StoreError",
                                        "ref_alive callback must return bool",
                                    )
                                })
                            })
                    };
                    store.gc(apply, grace, Some(&mut callback)).map_err(cv)
                } else {
                    store.gc(apply, grace, None).map_err(cv)
                }
            }
            _ => Err(Error::new(format!("unknown store service operation {op}"))),
        }
    }
}
pub fn serve() -> Result<u8> {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut wire = Wire {
        input: stdin.lock(),
        output: stdout.lock(),
    };
    let mut state = State::default();
    let result = (|| -> Result<()> {
        while let Some(request) = wire.read()? {
            let response = match state.dispatch(&request, &mut wire) {
                Ok(value) => json!({"ok":value}),
                Err(error) => json!({"error":error}),
            };
            wire.send(&response)?;
        }
        Ok(())
    })();
    state.abandon();
    result?;
    Ok(0)
}
pub fn copy_fds(input: i32, output: i32) -> Result<u8> {
    use std::os::fd::FromRawFd;
    if input < 3 || output < 3 || input == output {
        return Err(Error::new(
            "copy requires two distinct inherited file descriptors >=3",
        ));
    }
    let mut input = unsafe { std::fs::File::from_raw_fd(input) };
    let mut output = unsafe { std::fs::File::from_raw_fd(output) };
    if !input.metadata()?.is_file() || !output.metadata()?.is_file() {
        return Err(Error::new("copy inputs must be regular files"));
    }
    s::clone_or_copy(&mut input, &mut output).map_err(cv)?;
    Ok(0)
}
