//! Leased artifact supervision and exact, isolated workspace-body requests.
//! Disk admission belongs to the outer build; compile admission occurs only in a store builder.
use crate::{
    artifact_config as config, artifact_executor as executor, artifact_fs as af,
    artifact_inputs as inputs, artifact_store as storage, Error, Result,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Write,
    os::unix::{
        fs::{OpenOptionsExt, PermissionsExt},
        io::AsRawFd,
    },
    path::{Path, PathBuf},
    process::Command,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

pub const SCHEMA: &str = "vibeos-artifact-runtime-v1";
pub const TARGET: &str = "x86_64-pc-windows-gnu";
pub const KERNEL_FLAGS: &str = "-C force-frame-pointers=yes";
pub const DLLTOOL_FLAG: &str = "-C dlltool=/native-tools/bin/llvm-dlltool";
pub const GC_INTERVAL_SECONDS: f64 = 86400.0;
fn fail(message: impl Into<String>) -> Error {
    Error::classified("RuntimeError_", message)
}
fn se(e: storage::StoreError) -> Error {
    Error::classified(e.kind, e.message)
}
fn to_store(e: Error) -> storage::StoreError {
    storage::StoreError {
        kind: e.kind.unwrap_or_else(|| "RuntimeError_".into()),
        message: e.message,
    }
}
fn error_value(e: &Error) -> Value {
    json!({"type": e.kind.as_deref().unwrap_or("RuntimeError_"), "message": e.message})
}
fn s<'a>(v: &'a Value, k: &str) -> Result<&'a str> {
    v[k].as_str()
        .ok_or_else(|| fail(format!("missing or invalid {k}")))
}
fn arr<'a>(v: &'a Value, k: &str) -> Result<&'a Vec<Value>> {
    v[k].as_array()
        .ok_or_else(|| fail(format!("missing or invalid {k}")))
}
fn strings(v: &Value) -> Result<Vec<String>> {
    serde_json::from_value(v.clone()).map_err(|e| fail(e.to_string()))
}
fn ns() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64
}
fn unique() -> String {
    format!(
        "{}-{}-{}",
        std::process::id(),
        ns(),
        NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    )
}
static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
fn regular(p: &Path) -> bool {
    fs::symlink_metadata(p).is_ok_and(|m| m.is_file())
}
fn read_json(p: &Path) -> Result<Value> {
    let v: Value = serde_json::from_slice(&af::verified_read(p, "JSON")?)?;
    if !v.is_object() {
        return Err(fail(format!("JSON must be an object: {}", p.display())));
    }
    Ok(v)
}
fn write_json(p: &Path, v: &Value) -> Result<()> {
    let mut f = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(p)?;
    f.write_all(&inputs::canonical(v)?)?;
    f.sync_all()?;
    Ok(())
}
fn paths(repo: &Path, store: &Path) -> Result<()> {
    if repo.starts_with(store) || store.starts_with(repo) {
        Err(fail(
            "artifact store and source checkout must be disjoint directories",
        ))
    } else {
        Ok(())
    }
}
fn action_key(v: &str) -> bool {
    v.len() == 64
        && v.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
pub fn supervisor_config(path: &Path) -> Result<Value> {
    let v = read_json(path)?;
    if v["schema"] != SCHEMA {
        return Err(fail("unsupported artifact supervisor schema"));
    }
    for name in ["repo", "store_root", "toolchain", "activity", "broker"] {
        if !Path::new(s(&v, name)?).is_absolute() {
            return Err(fail(format!("supervisor {name} must be an absolute path")));
        }
    }
    paths(Path::new(s(&v, "repo")?), Path::new(s(&v, "store_root")?))?;
    if inputs::digest(&v["provisioned"])? != s(&v, "toolchain_key")? {
        return Err(fail("supervisor toolchain descriptor changed"));
    }
    if !v["clean"].is_boolean() || !v["verify"].is_boolean() {
        return Err(fail("supervisor clean/verify must be booleans"));
    }
    if !v["repair_action"].is_null() && !v["repair_action"].as_str().is_some_and(action_key) {
        return Err(fail("repair action must be a SHA-256 action identity"));
    }
    Ok(v)
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct SupervisorOptions {
    pub clean: bool,
    pub verify: bool,
    pub repair_action: Option<String>,
    pub arch: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RequestOptions {
    pub features: Vec<String>,
    pub default_features: bool,
    pub kernel: bool,
    pub flags: String,
    pub architecture: String,
    pub target: String,
    pub build_std: bool,
    pub emit_ir: bool,
    pub force_staticlib: bool,
    pub cargo_cwd: Option<PathBuf>,
}
impl Default for RequestOptions {
    fn default() -> Self {
        Self {
            features: vec![],
            default_features: true,
            kernel: false,
            flags: String::new(),
            architecture: "amd64".into(),
            target: TARGET.into(),
            build_std: false,
            emit_ir: false,
            force_staticlib: false,
            cargo_cwd: None,
        }
    }
}
impl RequestOptions {
    fn validate(&mut self, body: &str) -> Result<()> {
        if !regex::Regex::new(r"^[A-Za-z0-9][A-Za-z0-9_-]*$")
            .unwrap()
            .is_match(body)
        {
            return Err(fail(format!(
                "body must name one workspace package: {body:?}"
            )));
        }
        let expected = match self.architecture.as_str() {
            "amd64" => TARGET,
            "i386" => "i686-pc-windows-gnu",
            "arm64" => "aarch64-pc-windows-gnullvm",
            _ => "",
        };
        if expected.is_empty()
            || expected != self.target
            || self.build_std
            || self.emit_ir
            || self.force_staticlib
        {
            return Err(fail(
                "store requires builtin-target manifest staticlibs without build-std/IR",
            ));
        }
        if self.flags.trim() != executor::flags(&self.architecture, self.kernel)? {
            return Err(fail(format!(
                "unsupported RUSTFLAGS for {}: {:?}",
                self.architecture, self.flags
            )));
        }
        if self.features.iter().any(|s| {
            s.is_empty()
                || !s
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"_-".contains(&b))
        }) {
            return Err(fail(
                "features must be explicit nonempty package feature names",
            ));
        }
        self.features.sort();
        self.features.dedup();
        Ok(())
    }
    fn executor(&self, provisioned: Option<Value>) -> executor::ExecutorOptions {
        executor::ExecutorOptions {
            features: self.features.clone(),
            default_features: self.default_features,
            kernel: self.kernel,
            provisioned,
            target: self.target.clone(),
            flags: Some(self.flags.trim().into()),
        }
    }
}

/// Injectable compiler/discovery boundary for controlled native tests. Store leases and locks stay real.
pub trait RuntimeHost {
    fn provision(&mut self, toolchain: &Path) -> Result<Value> {
        executor::provision(toolchain)
    }
    fn context(
        &mut self,
        toolchain: &Path,
        provisioned: &Value,
        body: &str,
        o: &executor::ExecutorOptions,
    ) -> Result<Value> {
        executor::request_context(toolchain, provisioned, body, o)
    }
    fn discover(
        &mut self,
        repo: &Path,
        body: &str,
        context: &Value,
        o: &RequestOptions,
        toolchain: &Path,
    ) -> Result<Value> {
        Ok(inputs::discover(
            repo,
            body,
            context.clone(),
            &inputs::DiscoverOptions {
                features: o.features.clone(),
                default_features: o.default_features,
                cargo: toolchain.join("bin/cargo").to_string_lossy().into_owned(),
                ..Default::default()
            },
        ))
    }
    fn unchanged(&mut self, discovery: &Value) -> Result<bool> {
        Ok(inputs::unchanged(discovery))
    }
    fn prepare(
        &mut self,
        repo: &Path,
        body: &str,
        toolchain: &Path,
        out: &Path,
        o: &executor::ExecutorOptions,
    ) -> Result<Value> {
        executor::prepare(repo, body, toolchain, out, o)
    }
    fn execute(&mut self, broker: &Path, out: &Path) -> Result<Value> {
        execute(broker, out)
    }
    fn audit_scope(&mut self, repo: &Path, broker: &Path) -> Result<Value> {
        config::audit(
            repo,
            &config::cargo_scope(repo)?,
            None,
            Some(broker),
            Some(TARGET),
            false,
            Path::new(config::CANONICAL_ROOT),
        )
    }
    fn audit_request(&mut self, repo: &Path, broker: &Path, o: &RequestOptions) -> Result<Value> {
        let cwd = config::mapped_cwd(
            repo,
            o.cargo_cwd.as_deref(),
            o.kernel,
            Path::new(config::CANONICAL_ROOT),
        )?;
        let mut r = config::audit(
            repo,
            &[cwd.clone()],
            None,
            Some(broker),
            Some(&o.target),
            !o.flags.trim().is_empty(),
            Path::new(config::CANONICAL_ROOT),
        )?;
        r["mapped_cwd"] = json!(cwd);
        Ok(r)
    }
    fn watches(&mut self, repo: &Path, discovery: &Value) -> Result<Vec<String>> {
        watches(repo, discovery)
    }
}
#[derive(Default)]
pub struct NativeHost;
impl RuntimeHost for NativeHost {}

pub struct Supervisor<H: RuntimeHost = NativeHost> {
    pub host: H,
    store: Option<storage::Store>,
    session: Option<storage::Session>,
    state: Option<Value>,
    leased_context: Option<(PathBuf, PathBuf, PathBuf, Value)>,
    summary: Value,
    context_dir: PathBuf,
    private: PathBuf,
    path: PathBuf,
    activity: PathBuf,
    started: Instant,
    outer_started: Option<Instant>,
    closed: bool,
}
impl Supervisor<NativeHost> {
    pub fn open(
        repo: &Path,
        store_root: &Path,
        toolchain: &Path,
        context_dir: &Path,
        o: &SupervisorOptions,
    ) -> Result<Self> {
        Self::open_with_host(repo, store_root, toolchain, context_dir, o, NativeHost)
    }
}
impl<H: RuntimeHost> Supervisor<H> {
    pub fn open_with_host(
        repo: &Path,
        store_root: &Path,
        toolchain: &Path,
        context_dir: &Path,
        o: &SupervisorOptions,
        host: H,
    ) -> Result<Self> {
        let started = Instant::now();
        let context_dir = af::resolve(context_dir)?;
        fs::create_dir_all(&context_dir)?;
        let private = tempfile::Builder::new()
            .prefix("artifact-supervisor-")
            .tempdir_in(&context_dir)?
            .keep();
        let activity = private.join("activity");
        fs::create_dir(&activity)?;
        fs::set_permissions(&activity, fs::Permissions::from_mode(0o700))?;
        let summary = json!({"schema":"vibeos-artifact-supervisor-summary-v1","started_ns":ns(),"finished_ns":null,"status":"failed","error":null,"clean":o.clean,"verify":o.verify,"repair_action":o.repair_action,"repair_success":false,"toolchain_key":null,"toolchain_postcheck":"not_run","config_preflight":"not_run","config_postcheck":"not_run","verification_success":false,"requests":null,"timings":{}});
        let mut this = Self {
            host,
            store: None,
            session: None,
            state: None,
            leased_context: None,
            summary,
            context_dir,
            path: private.join("supervisor.json"),
            private,
            activity,
            started,
            outer_started: None,
            closed: false,
        };
        let init = this.initialize(repo, store_root, toolchain, o);
        if let Err(mut e) = init {
            if let Some((repo, toolchain, broker, provision)) = this.leased_context.clone() {
                if let Err(post) = this.postcheck(&repo, &toolchain, &broker, &provision) {
                    e = post;
                }
            }
            if let Some(mut session) = this.session.take() {
                if let Err(close) = session.close() {
                    e = se(close);
                }
            }
            this.summary["error"] = error_value(&e);
            this.closed = true;
            this.publish()?;
            return Err(e);
        }
        Ok(this)
    }
    fn initialize(
        &mut self,
        repo: &Path,
        store_root: &Path,
        toolchain: &Path,
        o: &SupervisorOptions,
    ) -> Result<()> {
        let repo = af::resolve(repo)?;
        let store_root = af::resolve(store_root)?;
        let toolchain = af::resolve(toolchain)?;
        self.summary["repo"] = json!(repo);
        self.summary["store_root"] = json!(store_root);
        self.summary["toolchain"] = json!(toolchain);
        paths(&repo, &store_root)?;
        let broker = repo.join(".cache/host-jobs/host-jobs");
        if !regular(&broker) || fs::metadata(&broker)?.permissions().mode() & 0o111 == 0 {
            return Err(fail(format!(
                "compiler admission broker missing or not executable: {}",
                broker.display()
            )));
        }
        if o.repair_action
            .as_deref()
            .is_some_and(|a| !action_key(a) || o.clean || o.verify)
        {
            return Err(fail(
                "repair-only supervision requires one action key without clean/verify",
            ));
        }
        let start = Instant::now();
        let result = self.host.audit_scope(&repo, &broker);
        self.summary["timings"]["config_preflight_seconds"] = json!(start.elapsed().as_secs_f64());
        match result {
            Ok(r) => {
                self.summary["cargo_config_preflight"] = r;
                self.summary["config_preflight"] = json!("valid")
            }
            Err(e) => {
                self.summary["config_preflight"] = json!("failed");
                return Err(e);
            }
        }
        let start = Instant::now();
        let provision = self.host.provision(&toolchain);
        self.summary["timings"]["provision_seconds"] = json!(start.elapsed().as_secs_f64());
        let provision = provision?;
        self.summary["toolchain_key"] = json!(inputs::digest(&provision)?);
        let store = storage::Store::new(&store_root).map_err(se)?;
        let session = store.session().map_err(se)?;
        self.store = Some(store);
        self.session = Some(session);
        self.leased_context = Some((
            repo.clone(),
            toolchain.clone(),
            broker.clone(),
            provision.clone(),
        ));
        let start = Instant::now();
        let restored = (|| -> Result<Vec<String>> {
            let store = self.store.as_ref().unwrap();
            let session = self.session.as_ref().unwrap();
            if let Some(action) = &o.repair_action {
                store.inspect_manifest(action, Some(session)).map_err(se)?;
                Ok(vec![])
            } else {
                Ok(store
                    .restore_refs(&repo.to_string_lossy(), session)
                    .map_err(se)?
                    .into_iter()
                    .map(|h| h.action_key)
                    .collect())
            }
        })();
        self.summary["timings"]["preflight_seconds"] = json!(start.elapsed().as_secs_f64());
        // Once leased, every exit must perform postchecks, including a failed restore.
        let restored = match restored {
            Ok(mut r) => {
                r.sort();
                r.dedup();
                r
            }
            Err(e) => return Err(e),
        };
        let state = json!({"schema":SCHEMA,"repo":repo,"store_root":store_root,"toolchain":toolchain,"provisioned":provision,"toolchain_key":self.summary["toolchain_key"],"session_token":self.session.as_ref().unwrap().token,"activity":self.activity,"broker":broker,"summary":self.context_dir.join("summary.json"),"outer_result":self.private.join("outer-result.json"),"clean":o.clean,"verify":o.verify,"restored_actions":restored,"repair_action":o.repair_action,"arch":o.arch,"strict":std::env::var("VIBEOS_ARTIFACT_STRICT").as_deref()==Ok("1")});
        self.summary["restored_actions"] = state["restored_actions"].clone();
        self.summary["preflight_mode"] = json!(if o.repair_action.is_some() {
            "pin repair metadata only"
        } else {
            "restore lane refs"
        });
        write_json(&self.path, &state)?;
        self.state = Some(state);
        self.outer_started = Some(Instant::now());
        Ok(())
    }
    pub fn path(&self) -> &Path {
        &self.path
    }
    fn postcheck(
        &mut self,
        repo: &Path,
        toolchain: &Path,
        broker: &Path,
        provision: &Value,
    ) -> Result<()> {
        let start = Instant::now();
        let check = self.host.audit_scope(repo, broker);
        self.summary["timings"]["config_postcheck_seconds"] = json!(start.elapsed().as_secs_f64());
        match check {
            Ok(r) => {
                self.summary["cargo_config_postcheck"] = r;
                self.summary["config_postcheck"] = json!("valid")
            }
            Err(e) => {
                self.summary["config_postcheck"] = json!("failed");
                return Err(e);
            }
        }
        let start = Instant::now();
        let check = self.host.provision(toolchain);
        self.summary["timings"]["postcheck_seconds"] = json!(start.elapsed().as_secs_f64());
        match check {
            Ok(r) if &r == provision => {
                self.summary["toolchain_postcheck"] = json!("identical");
                Ok(())
            }
            Ok(_) => {
                self.summary["toolchain_postcheck"] = json!("changed");
                Err(fail(
                    "supervisor toolchain/runtime changed during outer build",
                ))
            }
            Err(e) => {
                self.summary["toolchain_postcheck"] = json!("failed");
                Err(e)
            }
        }
    }
    pub fn close(&mut self, outer_failure: Option<Value>) -> Result<Value> {
        if self.closed {
            return Ok(self.summary.clone());
        }
        if let Some(start) = self.outer_started {
            self.summary["timings"]["outer_work_seconds"] = json!(start.elapsed().as_secs_f64())
        }
        let mut failure = outer_failure.map(|v| {
            Error::classified(
                v["type"].as_str().unwrap_or("OuterBuildFailure"),
                v["message"].as_str().unwrap_or("outer build failed"),
            )
        });
        if let Some(state) = self.state.clone() {
            let result = (|| -> Result<()> {
                if failure.is_some() {
                    return Ok(());
                }
                self.summary["requests"] = receipt_stats(&self.activity)?;
                if let Some(action) = state["repair_action"].as_str() {
                    if self.summary["requests"]["repaired_actions"] != json!([action]) {
                        return Err(fail(
                            "repair requested but no matching successful repair recorded",
                        ));
                    }
                    let mut keys: Vec<_> = self
                        .store
                        .as_ref()
                        .unwrap()
                        .restore_refs(s(&state, "repo")?, self.session.as_ref().unwrap())
                        .map_err(se)?
                        .into_iter()
                        .map(|h| h.action_key)
                        .collect();
                    keys.sort();
                    keys.dedup();
                    self.summary["restored_actions"] = json!(keys);
                }
                let outer = Path::new(s(&state, "outer_result")?);
                self.summary["outer_exit_code"] = if outer.exists() {
                    read_json(outer)?["exit_code"].clone()
                } else {
                    Value::Null
                };
                if state["verify"] == true
                    && self.summary["requests"]["verification_compared_requests"] == 0
                    && (self.summary["outer_exit_code"].is_null()
                        || self.summary["outer_exit_code"] == 0)
                {
                    return Err(fail(
                        "artifact verification requested but no selected action compared",
                    ));
                }
                Ok(())
            })();
            if let Err(e) = result {
                failure = Some(e)
            }
            if let Err(e) = self.postcheck(
                Path::new(s(&state, "repo")?),
                Path::new(s(&state, "toolchain")?),
                Path::new(s(&state, "broker")?),
                &state["provisioned"],
            ) {
                failure = Some(e)
            }
        }
        if let Some(mut session) = self.session.take() {
            if let Err(e) = session.close() {
                failure = Some(se(e))
            }
        }
        if let Some(e) = &failure {
            self.summary["error"] = error_value(e)
        } else if self.state.is_some() {
            let failed = self.summary["requests"]["failed_requests"]
                .as_u64()
                .unwrap_or(0);
            let exit = self.summary["outer_exit_code"].clone();
            if (!exit.is_null() && exit != 0) || failed > 0 {
                self.summary["error"] = json!({"type":"OuterBuildFailure","message":format!("outer exit code {exit}; {failed} failed selected requests")})
            } else {
                self.summary["status"] = json!("success");
                self.summary["verification_success"] = self.summary["verify"].clone();
                self.summary["repair_success"] = json!(!self.summary["repair_action"].is_null());
            }
        }
        let publish = self.publish();
        self.closed = true;
        publish?;
        if let Some(e) = failure {
            return Err(e);
        }
        Ok(self.summary.clone())
    }
    pub fn finalize(&mut self, outer_failure: Option<Value>) -> Result<Value> {
        self.close(outer_failure)
    }
    fn publish(&mut self) -> Result<()> {
        let mut receipt_failure = None;
        if self.summary["requests"].is_null() {
            match receipt_stats(&self.activity) {
                Ok(r) => self.summary["requests"] = r,
                Err(e) => {
                    let v = error_value(&e);
                    self.summary["receipt_error"] = v.clone();
                    if self.summary["error"].is_null() {
                        self.summary["error"] = v;
                        self.summary["status"] = json!("failed");
                        self.summary["verification_success"] = json!(false);
                        receipt_failure = Some(e);
                    }
                }
            }
        }
        self.summary["finished_ns"] = json!(ns());
        self.summary["timings"]["total_seconds"] = json!(self.started.elapsed().as_secs_f64());
        write_json(&self.private.join("summary.json"), &self.summary)?;
        let temporary = self.context_dir.join(format!(".summary-{}.json", unique()));
        write_json(&temporary, &self.summary)?;
        fs::rename(&temporary, self.context_dir.join("summary.json"))?;
        fs::File::open(&self.context_dir)?.sync_all()?;
        if let Some(e) = receipt_failure {
            return Err(e);
        }
        Ok(())
    }
}
impl<H: RuntimeHost> Drop for Supervisor<H> {
    fn drop(&mut self) {
        if !self.closed {
            let _=self.close(Some(json!({"type":"OuterBuildFailure","message":"supervisor dropped without explicit finalization"})));
        }
    }
}

pub fn receipt_stats(activity: &Path) -> Result<Value> {
    let mut paths: Vec<_> = fs::read_dir(activity)?
        .map(|e| e.map(|e| e.path()))
        .collect::<std::io::Result<_>>()?;
    paths.sort();
    let attempts: Vec<_> = paths
        .iter()
        .filter(|p| p.extension().is_some_and(|s| s == "json"))
        .map(|p| read_json(p))
        .collect::<Result<_>>()?;
    let receipts: Vec<_> = attempts
        .iter()
        .filter(|r| r["status"].is_null() || r["status"] == "success")
        .collect();
    let failures: Vec<_> = attempts
        .iter()
        .filter(|r| r["status"] == "failed")
        .collect();
    let ineligible: Vec<_> = attempts
        .iter()
        .filter(|r| r["status"] == "ineligible")
        .collect();
    for receipt in &receipts {
        s(receipt, "body")?;
        for field in ["hit", "verification_compared", "verified_rebuild"] {
            if !receipt[field].is_boolean() {
                return Err(fail(format!("receipt missing {field}")));
            }
        }
    }
    let count = |name: &str| receipts.iter().filter(|r| r[name] == true).count();
    let mut timings = BTreeMap::new();
    for name in [
        "discovery_seconds",
        "lookup_wait_build_seconds",
        "revalidation_seconds",
        "total_seconds",
        "compile_seconds",
    ] {
        timings.insert(
            name,
            receipts
                .iter()
                .map(|r| r["timings"][name].as_f64().unwrap_or(0.0))
                .sum::<f64>(),
        );
    }
    let mut bodies = BTreeMap::new();
    for r in &ineligible {
        bodies.insert(
            s(r, "body")?.to_owned(),
            arr(r, "reasons")?
                .first()
                .cloned()
                .ok_or_else(|| fail("ineligible request has no reason"))?,
        );
    }
    let actions: BTreeSet<_> = receipts
        .iter()
        .map(|r| s(r, "action_key").map(str::to_owned))
        .collect::<Result<_>>()?;
    let repaired: BTreeSet<_> = receipts
        .iter()
        .filter(|r| r["repaired"] == true)
        .map(|r| s(r, "action_key").map(str::to_owned))
        .collect::<Result<_>>()?;
    let builder_calls = receipts
        .iter()
        .map(|r| {
            r["builder_calls"]
                .as_u64()
                .ok_or_else(|| fail("receipt missing builder_calls"))
        })
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .sum::<u64>();
    let peak = receipts
        .iter()
        .filter_map(|r| r["sampled_worker_target_peak_allocated_bytes"].as_u64())
        .max();
    Ok(
        json!({"requested":attempts.len(),"successful_requests":receipts.len(),"failed_requests":failures.len(),"failure_examples":failures.iter().take(10).map(|r|json!({"body":r["body"],"error":r["error"]})).collect::<Vec<_>>(),"ineligible_requests":ineligible.len(),"ineligible_bodies":bodies,"hits":count("hit"),"misses":receipts.len()-count("hit"),"builder_calls":builder_calls,"failed_request_builder_calls":if failures.is_empty(){json!(0)}else{json!("unknown")},"verification_compared_requests":count("verification_compared"),"actual_verified_rebuild_requests":count("verified_rebuild"),"unique_actions":actions,"repaired_actions":repaired,"sampled_worker_target_peak_allocated_bytes":peak,"worker_requests_without_peak_measurement":receipts.iter().filter(|r|r["builder_calls"].as_u64().unwrap_or(0)>0&&r["sampled_worker_target_peak_allocated_bytes"].is_null()).count(),"request_timings":timings}),
    )
}
pub fn record_outer_result(supervisor: &Path, exit_code: i64) -> Result<()> {
    let state = supervisor_config(supervisor)?;
    let store = storage::Store::new(Path::new(s(&state, "store_root")?)).map_err(se)?;
    let mut session = store
        .attach_session(s(&state, "session_token")?)
        .map_err(se)?;
    let result = write_json(
        Path::new(s(&state, "outer_result")?),
        &json!({"exit_code":exit_code}),
    );
    session.close().map_err(se)?;
    result
}

fn execute(broker: &Path, out: &Path) -> Result<Value> {
    execute_with(
        broker,
        out,
        std::env::var("VIBEOS_HOST_JOB_HELD").as_deref() == Ok("1"),
        executor::run,
    )
}
fn execute_with(
    broker: &Path,
    out: &Path,
    held: bool,
    run: impl FnOnce(&Path) -> Result<Value>,
) -> Result<Value> {
    if held {
        return run(out);
    }

    let mut command = Command::new(broker);
    command
        .args(["--kind", "compile", "--"])
        .arg(std::env::current_exe()?)
        // The child must serve THIS repo. Without --root it falls back to the compile-time
        // CARGO_MANIFEST_DIR, which is /tmp/vibeos/tools/vibeos-build/../.. when build-infra
        // was provisioned inside a canon-run mount: every fresh tree then failed "isolated
        // compiler failed (2) ... No such file or directory" while the integration tree,
        // provisioned outside a mount, passed (2026-11-06).
        .arg("--root")
        .arg(executor::implementation_repo())
        .args(["artifact-executor", "run"])
        .arg(out);
    let result = executor::supervised_output(&mut command, None)?;
    if result.code != 0 {
        let mut diagnostic = result.stderr.clone();
        diagnostic.push_str(&result.stdout.as_str());
        let tail = diagnostic
            .chars()
            .rev()
            .take(2000)
            .collect::<String>()
            .chars()
            .rev()
            .collect::<String>();
        return Err(fail(format!(
            "isolated compiler failed ({}); evidence {}: {tail}",
            result.code,
            out.join("report.json").display()
        )));
    }
    read_json(&out.join("report.json"))
}
fn manifest_staticlib(repo: &Path, body: &str) -> Result<bool> {
    let p = repo.join(format!("sdk/lib/rust/{body}/Cargo.toml"));
    if !p.is_file() {
        return Ok(false);
    }
    let text = fs::read_to_string(p)?;
    let parsed: toml::Value = toml::from_str(&text).map_err(|e| fail(e.to_string()))?;
    let value = serde_json::to_value(parsed)?;
    let kinds = value["lib"]["crate-type"].as_array();
    Ok(value["package"]["name"] == body
        && kinds.is_some_and(|k| {
            k.iter().any(|v| v == "staticlib") && k.iter().all(|v| v == "staticlib" || v == "rlib")
        }))
}
pub fn watches(repo: &Path, discovery: &Value) -> Result<Vec<String>> {
    let mut paths = BTreeSet::new();
    for relative in discovery["provenance"]["reads"]
        .as_object()
        .ok_or_else(|| fail("discovery missing reads inventory"))?
        .keys()
    {
        let p = Path::new(relative);
        if !af::safe_relative(p) {
            return Err(fail("discovery watch path is not repository relative"));
        }
        paths.insert(repo.join(p));
    }
    for (relative, package) in discovery["descriptor"]["packages"]
        .as_object()
        .ok_or_else(|| fail("discovery missing packages"))?
    {
        let p = Path::new(relative);
        if !af::safe_relative(p) {
            return Err(fail("discovery package root is not repository relative"));
        }
        paths.insert(repo.join(p));
        for d in arr(&package["inventory"], "directories")? {
            let d = Path::new(
                d.as_str()
                    .ok_or_else(|| fail("invalid package membership directory"))?,
            );
            if !af::safe_relative(d) {
                return Err(fail("discovery membership directory escapes package"));
            }
            paths.insert(repo.join(p).join(d));
        }
    }
    paths.extend(crate::artifact_api::implementation_watches(repo)?);
    paths.insert(repo.join("rust-toolchain.toml"));
    for path in &paths {
        if !path.exists() || path.is_symlink() {
            return Err(fail(format!(
                "required watch is absent or symlinked: {}",
                path.display()
            )));
        }
    }
    Ok(paths
        .into_iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect())
}
fn ineligible(state: &Value, body: &str, reasons: Vec<String>, started: Instant) -> Result<Value> {
    if state["strict"] == true {
        return Err(fail(format!(
            "strict store: {body} is ineligible: {}",
            reasons.join("; ")
        )));
    }
    let r = json!({"schema":SCHEMA,"status":"ineligible","body":body,"reasons":reasons,"timings":{"total_seconds":started.elapsed().as_secs_f64()}});
    write_json(
        &Path::new(s(state, "activity")?).join(format!("{}.json", unique())),
        &r,
    )?;
    Ok(r)
}
fn body_build<H: RuntimeHost>(
    host: &mut H,
    state: &Value,
    body: &str,
    o: &RequestOptions,
    context: &Value,
    discovery: &Value,
    builds: &mut Vec<Value>,
    scratch: &Path,
) -> Result<storage::BuildResult> {
    let expected = s(discovery, "action_key")?;
    let output = scratch.join("execution");
    let repo = Path::new(s(state, "repo")?);
    let toolchain = Path::new(s(state, "toolchain")?);
    let full_flags = s(context, "effective_flags")?;
    let flags = full_flags
        .strip_suffix(DLLTOOL_FLAG)
        .ok_or_else(|| fail("action context lacks the audited dlltool flag"))?
        .trim();
    let mut opts = o.executor(Some(state["provisioned"].clone()));
    opts.target = s(&context["target"], "builtin")?.into();
    opts.flags = Some(flags.into());
    let prepared = host.prepare(repo, body, toolchain, &output, &opts)?;
    if prepared["action_key"] != expected || prepared["context"] != *context {
        return Err(fail(
            "worker preparation differs from discovered action/context",
        ));
    }
    let report = host.execute(Path::new(s(state, "broker")?), &output)?;
    let archive = PathBuf::from(report["archive"]["path"].as_str().unwrap_or(""));
    if report["action_key"] != expected
        || report["context"] != *context
        || report["compile_status"] != "success"
        || report["lane_input_revalidation"] != "identical"
        || report["toolchain_postcheck"] != "identical"
        || !archive.is_absolute()
        || !regular(&archive)
        || !af::resolve(&archive)?.starts_with(af::resolve(scratch)?)
        || af::resolve(&archive)? == af::resolve(scratch)?
    {
        return Err(fail(format!(
            "isolated worker did not prove its result; evidence {}",
            output.display()
        )));
    }
    let mut worker = serde_json::Map::new();
    for k in [
        "compile_status",
        "compile_seconds",
        "rustc_invocations",
        "lane_input_revalidation",
        "toolchain_postcheck",
        "retained",
        "sampled_target_peak_allocated_bytes",
    ] {
        worker.insert(k.into(), report[k].clone());
    }
    let provenance = json!({"runtime_schema":SCHEMA,"source_projection_key":discovery["source_projection_key"],"worker":worker});
    builds.push(report);
    Ok(storage::BuildResult {
        outputs: BTreeMap::from([("body.a".into(), storage::Output::archive(archive))]),
        input_inventory: discovery["provenance"]["reads"].clone(),
        dependency_actions: vec![],
        provenance,
    })
}
fn record_failure(
    supervisor: &Path,
    body: &str,
    action: Option<&str>,
    started: Instant,
    original: &Error,
) -> Result<()> {
    let state = supervisor_config(supervisor)?;
    let store = storage::Store::new(Path::new(s(&state, "store_root")?)).map_err(se)?;
    let mut session = store
        .attach_session(s(&state, "session_token")?)
        .map_err(se)?;
    let mut receipt = json!({"schema":SCHEMA,"status":"failed","body":body,"error":error_value(original),"timings":{"total_seconds":started.elapsed().as_secs_f64()}});
    if let Some(action) = action {
        receipt["repair_action"] = json!(action)
    }
    let result = write_json(
        &Path::new(s(&state, "activity")?).join(format!("{}.json", unique())),
        &receipt,
    );
    session.close().map_err(se)?;
    result
}
pub fn request(supervisor: &Path, body: &str, consumer: &str, o: &RequestOptions) -> Result<Value> {
    request_with_host(supervisor, body, consumer, o, &mut NativeHost)
}
pub fn request_with_host<H: RuntimeHost>(
    supervisor: &Path,
    body: &str,
    consumer: &str,
    o: &RequestOptions,
    host: &mut H,
) -> Result<Value> {
    let started = Instant::now();
    let result = request_inner(supervisor, body, consumer, o.clone(), host, started);
    if let Err(original) = &result {
        if let Err(e) = record_failure(supervisor, body, None, started, original) {
            return Err(fail(format!(
                "request failed ({original}); activity recording failed: {e}"
            )));
        }
    }
    result
}
fn request_inner<H: RuntimeHost>(
    supervisor: &Path,
    body: &str,
    consumer: &str,
    mut o: RequestOptions,
    host: &mut H,
    started: Instant,
) -> Result<Value> {
    o.validate(body)?;
    if consumer.is_empty() || consumer.chars().any(|c| "\0\r\n\t".contains(c)) {
        return Err(fail(
            "consumer must be a nonempty stable identity without control characters",
        ));
    }
    let state = supervisor_config(supervisor)?;
    if !state["repair_action"].is_null() {
        return Err(fail(
            "repair-only supervisor cannot serve consumer requests",
        ));
    }
    let repo = Path::new(s(&state, "repo")?);
    let toolchain = Path::new(s(&state, "toolchain")?);
    let broker = Path::new(s(&state, "broker")?);
    let store = storage::Store::new(Path::new(s(&state, "store_root")?)).map_err(se)?;
    let mut session = store
        .attach_session(s(&state, "session_token")?)
        .map_err(se)?;
    let result = (|| -> Result<Value> {
        let config_before = host.audit_request(repo, broker, &o)?;
        if !manifest_staticlib(repo, body)? {
            return ineligible(
                &state,
                body,
                vec!["not a manifest staticlib at sdk/lib/rust/<body>".into()],
                started,
            );
        }
        let context = host.context(toolchain, &state["provisioned"], body, &o.executor(None))?;
        let start = Instant::now();
        let discovery = host.discover(repo, body, &context, &o, toolchain)?;
        if discovery["eligible"] != true
            || discovery["action_key"].as_str().is_none_or(str::is_empty)
        {
            let reasons = if discovery["reasons"]
                .as_array()
                .is_some_and(|a| !a.is_empty())
            {
                strings(&discovery["reasons"])?
            } else {
                vec!["no action key".into()]
            };
            return ineligible(&state, body, reasons, started);
        }
        let discovery_seconds = start.elapsed().as_secs_f64();
        let descriptor = &discovery["descriptor"];
        let expected = s(&discovery, "action_key")?;
        if inputs::digest(descriptor)? != expected || descriptor["context"] != context {
            return Err(fail("discovery action key/context mismatch"));
        }
        let mut watches = host.watches(repo, &discovery)?;
        let mut builds = vec![];
        let force = state["clean"] == true || state["verify"] == true;
        let start = Instant::now();
        let opts = storage::BuildOptions {
            verify: force,
            verify_once_per_session: force,
            require_verified: state["verify"] == true,
        };
        let mut handle = store
            .lookup_or_build(
                descriptor,
                |scratch| {
                    body_build(
                        host,
                        &state,
                        body,
                        &o,
                        &context,
                        &discovery,
                        &mut builds,
                        scratch,
                    )
                    .map_err(to_store)
                },
                &session,
                &opts,
            )
            .map_err(se)?;
        if state["verify"] == true && !handle.session_verified {
            handle = store
                .lookup_or_build(
                    descriptor,
                    |scratch| {
                        body_build(
                            host,
                            &state,
                            body,
                            &o,
                            &context,
                            &discovery,
                            &mut builds,
                            scratch,
                        )
                        .map_err(to_store)
                    },
                    &session,
                    &storage::BuildOptions {
                        verify: true,
                        verify_once_per_session: true,
                        require_verified: true,
                    },
                )
                .map_err(se)?;
        }
        if handle.action_key != expected || (state["verify"] == true && !handle.session_verified) {
            return Err(fail("store result/action verification mismatch"));
        }
        let lookup = start.elapsed().as_secs_f64();
        let start = Instant::now();
        let current = if host.unchanged(&discovery)? {
            discovery.clone()
        } else {
            host.discover(repo, body, &context, &o, toolchain)?
        };
        if current["eligible"] != true || current["action_key"] != expected {
            return Err(fail(
                "lane inputs changed while waiting/building the artifact",
            ));
        }
        let revalidation = start.elapsed().as_secs_f64();
        let config_after = host.audit_request(repo, broker, &o)?;
        watches.extend(strings(&config_after["watches"])?);
        watches.sort();
        watches.dedup();
        let reference = store
            .bind(&repo.to_string_lossy(), consumer, expected, &session)
            .map_err(se)?;
        let peak = builds
            .iter()
            .filter_map(|r| r["sampled_target_peak_allocated_bytes"].as_u64())
            .max();
        let compile: f64 = builds
            .iter()
            .map(|r| r["compile_seconds"].as_f64().unwrap_or(0.0))
            .sum();
        let result = json!({"schema":SCHEMA,"cargo_config":{"before":config_before,"after":config_after},"body":body,"action_key":expected,"archive":handle.blob_path().map_err(se)?,"watches":watches,"hit":handle.hit&&builds.is_empty(),"builder_calls":builds.len(),"sampled_worker_target_peak_allocated_bytes":peak,"verified_rebuild":handle.verified_rebuild,"verification_compared":handle.session_verified,"manifest":store.manifest_path(expected).map_err(se)?,"reference":reference,"timings":{"discovery_seconds":discovery_seconds,"lookup_wait_build_seconds":lookup,"revalidation_seconds":revalidation,"total_seconds":started.elapsed().as_secs_f64(),"compile_seconds":compile}});
        protocol_lines(&result)?;
        write_json(
            &Path::new(s(&state, "activity")?).join(format!("{}.json", unique())),
            &result,
        )?;
        Ok(result)
    })();
    session.close().map_err(se)?;
    result
}

fn repair_request<H: RuntimeHost>(
    host: &mut H,
    state: &Value,
    manifest: &Value,
) -> Result<(String, RequestOptions, Value, Value)> {
    let descriptor = &manifest["descriptor"];
    let req = &descriptor["request"];
    let package = s(req, "package")?;
    let body = Path::new(package)
        .file_name()
        .and_then(|p| p.to_str())
        .ok_or_else(|| fail("repair manifest has no selected package"))?
        .to_owned();
    if package != format!("sdk/lib/rust/{body}") {
        return Err(fail("repair package is not a workspace body"));
    }
    if req["profile"] != "release" || req["invocation"] != "cargo build --lib" {
        return Err(fail("repair requires a manifest staticlib release recipe"));
    }
    if !arr(manifest, "dependency_actions")?.is_empty()
        || arr(manifest, "outputs")?.len() != 1
        || manifest["outputs"][0]["name"] != "body.a"
        || manifest["outputs"][0]["role"] != "archive"
    {
        return Err(fail("repair requires one standalone body archive"));
    }
    let options = RequestOptions {
        features: strings(&req["features"])?,
        default_features: req["default_features"]
            .as_bool()
            .ok_or_else(|| fail("kernel and default_features must be booleans"))?,
        ..Default::default()
    };
    let toolchain = Path::new(s(state, "toolchain")?);
    let mut matches = vec![];
    for arch in crate::ARCHES {
        for kernel in [false, true] {
            let mut o = options.clone();
            o.architecture = arch.into();
            o.target = match arch {
                "amd64" => TARGET,
                "i386" => "i686-pc-windows-gnu",
                _ => "aarch64-pc-windows-gnullvm",
            }
            .into();
            o.kernel = kernel;
            o.flags = executor::flags(arch, kernel)?.into();
            o.validate(&body)?;
            let context =
                host.context(toolchain, &state["provisioned"], &body, &o.executor(None))?;
            if context == descriptor["context"] {
                matches.push((o, context));
            }
        }
    }
    if matches.len() != 1 {
        return Err(fail(
            "current toolchain/implementation cannot reproduce indexed repair recipe",
        ));
    }
    let (options, context) = matches.pop().unwrap();
    let repo = Path::new(s(state, "repo")?);
    if !manifest_staticlib(repo, &body)? {
        return Err(fail(
            "repair requires the current package's manifest staticlib",
        ));
    }
    let discovery = host.discover(repo, &body, &context, &options, toolchain)?;
    if discovery["eligible"] != true
        || discovery["action_key"] != manifest["action_key"]
        || discovery["descriptor"] != *descriptor
        || inputs::digest(descriptor)? != s(manifest, "action_key")?
    {
        return Err(fail(
            "current source/toolchain key does not exactly match indexed repair action",
        ));
    }
    Ok((body, options, context, discovery))
}
pub fn repair(supervisor: &Path, action: &str) -> Result<Value> {
    repair_with_host(supervisor, action, &mut NativeHost)
}
pub fn repair_with_host<H: RuntimeHost>(
    supervisor: &Path,
    action: &str,
    host: &mut H,
) -> Result<Value> {
    let started = Instant::now();
    let result = repair_inner(supervisor, action, host, started);
    if let Err(original) = &result {
        if let Err(e) = record_failure(supervisor, "repair", Some(action), started, original) {
            return Err(fail(format!(
                "repair failed ({original}); activity recording failed: {e}"
            )));
        }
    }
    result
}
fn repair_inner<H: RuntimeHost>(
    supervisor: &Path,
    action: &str,
    host: &mut H,
    started: Instant,
) -> Result<Value> {
    let state = supervisor_config(supervisor)?;
    if state["repair_action"] != action {
        return Err(fail("repair requires its exact repair-only supervisor"));
    }
    let repo = Path::new(s(&state, "repo")?);
    let toolchain = Path::new(s(&state, "toolchain")?);
    let store = storage::Store::new(Path::new(s(&state, "store_root")?)).map_err(se)?;
    let mut session = store
        .attach_session(s(&state, "session_token")?)
        .map_err(se)?;
    let result = (|| -> Result<Value> {
        let manifest = store.inspect_manifest(action, Some(&session)).map_err(se)?;
        let start = Instant::now();
        let (body, o, context, discovery) = repair_request(host, &state, &manifest)?;
        let discovery_seconds = start.elapsed().as_secs_f64();
        let mut builds = vec![];
        let start = Instant::now();
        let handle = store
            .repair_missing(
                action,
                |scratch| {
                    body_build(
                        host,
                        &state,
                        &body,
                        &o,
                        &context,
                        &discovery,
                        &mut builds,
                        scratch,
                    )
                    .map_err(to_store)
                },
                &session,
            )
            .map_err(se)?;
        let repair_seconds = start.elapsed().as_secs_f64();
        let start = Instant::now();
        let current = host.discover(repo, &body, &context, &o, toolchain)?;
        if current["eligible"] != true || current["action_key"] != action {
            return Err(fail("lane inputs changed during explicit repair"));
        }
        let mut restored: Vec<_> = store
            .restore_refs(&repo.to_string_lossy(), &session)
            .map_err(se)?
            .into_iter()
            .map(|h| h.action_key)
            .collect();
        restored.sort();
        restored.dedup();
        let result = json!({"schema":SCHEMA,"body":body,"action_key":action,"archive":handle.blob_path().map_err(se)?,"watches":[],"reference":null,"repaired":true,"hit":false,"builder_calls":builds.len(),"verified_rebuild":true,"verification_compared":true,"manifest":store.manifest_path(action).map_err(se)?,"restored_actions":restored,"sampled_worker_target_peak_allocated_bytes":builds.iter().filter_map(|r|r["sampled_target_peak_allocated_bytes"].as_u64()).max(),"timings":{"discovery_seconds":discovery_seconds,"lookup_wait_build_seconds":repair_seconds,"revalidation_seconds":start.elapsed().as_secs_f64(),"compile_seconds":builds.iter().map(|r|r["compile_seconds"].as_f64().unwrap_or(0.0)).sum::<f64>(),"total_seconds":started.elapsed().as_secs_f64()}});
        write_json(
            &Path::new(s(&state, "activity")?).join(format!("{}.json", unique())),
            &result,
        )?;
        Ok(result)
    })();
    session.close().map_err(se)?;
    result
}
pub fn retire_build_refs(supervisor: &Path) -> Result<Vec<Value>> {
    let state = supervisor_config(supervisor)?;
    if state["clean"] != true {
        return Err(fail(
            "Cargo ref retirement requires a clean supervisor after output removal",
        ));
    }
    let store = storage::Store::new(Path::new(s(&state, "store_root")?)).map_err(se)?;
    let mut session = store
        .attach_session(s(&state, "session_token")?)
        .map_err(se)?;
    let result = (|| -> Result<Vec<Value>> {
        let lane = s(&state, "repo")?;
        let markers = match state["arch"].as_str() {
            Some(arch) if !arch.is_empty() => vec![
                format!("-{arch}-cargo-link/"),
                format!("/cell-link-v1-{arch}-"),
            ],
            _ => vec![String::new()],
        };
        let mut retired = vec![];
        for reference in store.list_refs(lane).map_err(se)? {
            let consumer = s(&reference, "consumer")?;
            if consumer.starts_with("cargo:") && markers.iter().any(|m| consumer.contains(m)) {
                let mut guard = store.consumer_guard(lane, consumer).map_err(se)?;
                store.release_ref(lane, consumer, &guard).map_err(se)?;
                guard.close();
                retired.push(reference)
            }
        }
        Ok(retired)
    })();
    session.close().map_err(se)?;
    result
}
pub fn lane_ref_alive(reference: &Value, canonical_root: Option<&str>) -> Result<bool> {
    let lane = Path::new(s(reference, "lane")?);
    if !lane.is_dir() {
        return Ok(false);
    }
    let consumer = s(reference, "consumer")?;
    let Some(rest) = consumer.strip_prefix("cargo:") else {
        return Ok(true);
    };
    let fields: Vec<_> = rest.split('|').collect();
    let root = canonical_root
        .map(str::to_owned)
        .or_else(|| std::env::var("VIBEOS_CANON").ok())
        .unwrap_or_else(|| "/tmp/vibeos".into());
    let root = root.trim_end_matches('/');
    if fields.len() < 2 {
        return Ok(true);
    }
    let Some(relative) = fields[1].strip_prefix(&format!("{root}/")) else {
        return Ok(true);
    };
    Ok(lane.join(relative).is_dir())
}
pub fn scheduled_gc(store_root: &Path, now: Option<f64>, force: bool) -> Result<Option<Value>> {
    if !store_root.join("v1").is_dir() || store_root.join("gc-disabled").exists() {
        return Ok(None);
    }
    let stamp = store_root.join("last-gc");
    let now = now.unwrap_or_else(|| {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64()
    });
    if !now.is_finite() {
        return Err(fail("GC time must be finite"));
    }
    match fs::metadata(&stamp) {
        Ok(m) => {
            let modified = m
                .modified()?
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs_f64();
            if !force && now - modified < GC_INTERVAL_SECONDS {
                return Ok(None);
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(e.into()),
    }
    let claim = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(store_root.join("gc-claim.lock"))?;
    let rc = unsafe { libc::flock(claim.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
    if rc != 0 {
        let e = std::io::Error::last_os_error();
        if e.kind() == std::io::ErrorKind::WouldBlock {
            return Ok(None);
        }
        return Err(e.into());
    }
    let store = storage::Store::new(store_root).map_err(se)?;
    let mut alive = |reference: &Value| lane_ref_alive(reference, None).map_err(to_store);
    let report = store.gc(true, 3600.0, Some(&mut alive)).map_err(se)?;
    let mut summary = json!({"at":now});
    if let Some(o) = report.as_object() {
        for (key, value) in o {
            if let Some(a) = value.as_array() {
                summary[key] = json!(a.len());
            }
        }
    }
    fs::write(stamp, serde_json::to_vec(&summary)?)?;
    Ok(Some(report))
}
pub fn protocol_lines(result: &Value) -> Result<String> {
    if result["status"] == "ineligible" {
        let reasons = strings(&result["reasons"])?;
        let reason = reasons
            .join("; ")
            .chars()
            .map(|c| if "\0\r\n\t".contains(c) { ' ' } else { c })
            .collect::<String>();
        return Ok(format!("ineligible\t{reason}\n"));
    }
    let archive = s(result, "archive")?;
    let watches = strings(&result["watches"])?;
    let mut out = String::new();
    for (kind, path) in
        std::iter::once(("archive", archive)).chain(watches.iter().map(|p| ("watch", p.as_str())))
    {
        if !Path::new(path).is_absolute() || path.chars().any(|c| "\0\r\n\t".contains(c)) {
            return Err(fail(
                "protocol paths must be absolute and contain no tabs/newlines",
            ));
        }
        out.push_str(kind);
        out.push('\t');
        out.push_str(path);
        out.push('\n');
    }
    Ok(out)
}

#[cfg(test)]
#[path = "artifact_runtime_tests.rs"]
mod tests;

enum CliCommand {
    Request {
        supervisor: PathBuf,
        body: String,
        consumer: String,
        options: RequestOptions,
    },
    Repair {
        supervisor: PathBuf,
        action: String,
    },
}
fn parse_cli(args: &[String]) -> Result<CliCommand> {
    let (command, args) = args
        .split_first()
        .ok_or_else(|| fail("artifact-runtime requires request or repair"))?;
    if command != "request" && command != "repair" {
        return Err(fail(format!("unknown artifact-runtime command {command}")));
    }
    let mut values = BTreeMap::new();
    let mut o = RequestOptions::default();
    let mut i = 0;
    while i < args.len() {
        let raw = &args[i];
        let (name, inline) = raw
            .split_once('=')
            .map(|(a, b)| (a, Some(b.to_owned())))
            .unwrap_or((raw.as_str(), None));
        if command == "request" && ["--kernel", "--no-default-features"].contains(&name) {
            if inline.is_some() {
                return Err(fail(format!("{name} takes no value")));
            }
            if name == "--kernel" {
                o.kernel = true
            } else {
                o.default_features = false
            }
            i += 1;
            continue;
        }
        let allowed = if command == "request" {
            &[
                "--supervisor",
                "--body",
                "--consumer",
                "--features",
                "--flags",
                "--cargo-cwd",
                "--arch",
                "--target",
            ][..]
        } else {
            &["--supervisor", "--action"][..]
        };
        if !allowed.contains(&name) {
            return Err(fail(format!("unknown {command} option {name}")));
        }
        let value = if let Some(v) = inline {
            v
        } else {
            i += 1;
            args.get(i)
                .cloned()
                .ok_or_else(|| fail(format!("{name} requires a value")))?
        };
        if values.insert(name.to_owned(), value).is_some() {
            return Err(fail(format!("duplicate {name}")));
        }
        i += 1;
    }
    let required = |key: &str| {
        values
            .get(key)
            .map(String::as_str)
            .ok_or_else(|| fail(format!("{key} is required")))
    };
    let supervisor = PathBuf::from(required("--supervisor")?);
    if command == "repair" {
        return Ok(CliCommand::Repair {
            supervisor,
            action: required("--action")?.into(),
        });
    }
    if let Some(v) = values.get("--features") {
        o.features = if v.is_empty() {
            vec![]
        } else {
            v.split(',').map(str::to_owned).collect()
        };
    }
    if let Some(v) = values.get("--flags") {
        o.flags = v.clone();
    }
    if let Some(v) = values.get("--cargo-cwd") {
        o.cargo_cwd = Some(v.into());
    }
    if let Some(v) = values.get("--arch") {
        o.architecture = v.clone();
    }
    if let Some(v) = values.get("--target") {
        o.target = v.clone();
    }
    Ok(CliCommand::Request {
        supervisor,
        body: required("--body")?.into(),
        consumer: required("--consumer")?.into(),
        options: o,
    })
}
/// Native build.rs-facing command; selected failures always remain fatal.
pub fn cli(args: &[String]) -> Result<u8> {
    match parse_cli(args)? {
        CliCommand::Request {
            supervisor,
            body,
            consumer,
            options,
        } => {
            let result = request(&supervisor, &body, &consumer, &options)?;
            print!("{}", protocol_lines(&result)?);
        }
        CliCommand::Repair { supervisor, action } => {
            println!("{}", serde_json::to_string(&repair(&supervisor, &action)?)?)
        }
    }
    Ok(0)
}
