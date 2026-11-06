use super::*;
use std::{cell::RefCell, rc::Rc};

#[derive(Default)]
struct Control {
    provisions: usize,
    discoveries: usize,
    prepares: usize,
    executes: usize,
    audits: usize,
    request_audits: usize,
    changed_tool: bool,
    mismatch_prepare: bool,
    invalid_report: bool,
    nondeterministic: bool,
    source_edit: bool,
    edit_on_revalidation: bool,
    fail_after_audit: bool,
    ineligible: bool,
    fail_scope: bool,
    fail_post_scope: bool,
    fail_request_audit: bool,
    fail_execute: bool,
    pending: BTreeMap<PathBuf, (Value, Value)>,
}
#[derive(Clone)]
struct FakeHost {
    control: Rc<RefCell<Control>>,
    repo: PathBuf,
}
impl FakeHost {
    fn new(repo: PathBuf) -> Self {
        Self {
            repo,
            control: Rc::default(),
        }
    }
    fn source(&self) -> PathBuf {
        self.repo.join("sdk/lib/rust/beep-rust/src/lib.rs")
    }
}
impl RuntimeHost for FakeHost {
    fn provision(&mut self, _: &Path) -> Result<Value> {
        let mut c = self.control.borrow_mut();
        c.provisions += 1;
        Ok(
            json!({"rustc":"fixture","tree":if c.changed_tool&&c.provisions>1{"changed"}else{"fixed"}}),
        )
    }
    fn context(
        &mut self,
        _: &Path,
        provisioned: &Value,
        body: &str,
        o: &executor::ExecutorOptions,
    ) -> Result<Value> {
        Ok(
            json!({"toolchain":provisioned,"target":{"builtin":o.target},"effective_flags":format!("{}{}",o.flags.clone().filter(|s|!s.is_empty()).map(|s|format!("{s} ")).unwrap_or_default(),DLLTOOL_FLAG),"body":body,"features":o.features,"default_features":o.default_features,"environment":{},"effective_config":{},"execution_policy":"fixture"}),
        )
    }
    fn discover(
        &mut self,
        repo: &Path,
        body: &str,
        context: &Value,
        o: &RequestOptions,
        _: &Path,
    ) -> Result<Value> {
        self.control.borrow_mut().discoveries += 1;
        let source = inputs::digest(&json!({"source":fs::read_to_string(self.source())?}))?;
        let descriptor = json!({"context":context,"request":{"package":format!("sdk/lib/rust/{body}"),"features":o.features,"default_features":o.default_features,"profile":"release","invocation":"cargo build --lib"},"source":source,"packages":{format!("sdk/lib/rust/{body}"):{"inventory":{"directories":["","src"]}}}});
        let mut reads = BTreeMap::new();
        for p in [
            self.source(),
            repo.join(format!("sdk/lib/rust/{body}/Cargo.toml")),
            repo.join("sdk/lib/rust/Cargo.toml"),
            repo.join("sdk/lib/rust/Cargo.lock"),
        ] {
            reads.insert(
                p.strip_prefix(repo).unwrap().to_string_lossy().into_owned(),
                json!({}),
            );
        }
        Ok(
            json!({"eligible":!self.control.borrow().ineligible,"action_key":inputs::digest(&descriptor)?,"source_projection_key":source,"descriptor":descriptor,"provenance":{"reads":reads},"reasons":["unsupported fixture input"]}),
        )
    }
    fn unchanged(&mut self, _: &Value) -> Result<bool> {
        let mut c = self.control.borrow_mut();
        if c.edit_on_revalidation {
            fs::write(self.source(), "changed while waiting")?;
            c.edit_on_revalidation = false;
        }
        Ok(false)
    }
    fn prepare(
        &mut self,
        repo: &Path,
        body: &str,
        toolchain: &Path,
        out: &Path,
        o: &executor::ExecutorOptions,
    ) -> Result<Value> {
        self.control.borrow_mut().prepares += 1;
        fs::create_dir(out)?;
        let context = self.context(toolchain, o.provisioned.as_ref().unwrap(), body, o)?;
        let opts = RequestOptions {
            features: o.features.clone(),
            default_features: o.default_features,
            ..Default::default()
        };
        let discovery = self.discover(repo, body, &context, &opts, toolchain)?;
        let store = storage::Store::new(&repo.parent().unwrap().join("store")).unwrap();
        let lock = store
            .action_lock(discovery["action_key"].as_str().unwrap())
            .unwrap();
        assert!(
            storage::FileLock::acquire(&lock, true).unwrap().is_none(),
            "preparation escaped action lock"
        );
        let result = json!({"action_key":discovery["action_key"],"context":context});
        let mut c = self.control.borrow_mut();
        c.pending.insert(
            out.to_owned(),
            (result.clone(), discovery["source_projection_key"].clone()),
        );
        if c.mismatch_prepare {
            Ok(json!({"action_key":"bad","context":context}))
        } else {
            Ok(result)
        }
    }
    fn execute(&mut self, _: &Path, out: &Path) -> Result<Value> {
        let mut c = self.control.borrow_mut();
        c.executes += 1;
        if c.fail_execute {
            return Err(fail("controlled compiler failure"));
        }
        let (prepared, source) = c.pending[out].clone();
        let archive = out.join("body.a");
        let payload = format!(
            "{}{}",
            source,
            if c.nondeterministic {
                c.executes.to_string()
            } else {
                String::new()
            }
        );
        fs::write(&archive, payload)?;
        if c.source_edit {
            fs::write(self.source(), "changed while compiling")?;
            c.source_edit = false;
        }
        Ok(
            json!({"action_key":prepared["action_key"],"context":prepared["context"],"archive":{"path":archive},"compile_status":"success","lane_input_revalidation":if c.invalid_report{"changed"}else{"identical"},"toolchain_postcheck":"identical","sampled_target_peak_allocated_bytes":1234,"compile_seconds":0.25}),
        )
    }
    fn audit_scope(&mut self, _: &Path, _: &Path) -> Result<Value> {
        let mut c = self.control.borrow_mut();
        c.audits += 1;
        if c.fail_scope || (c.fail_post_scope && c.audits > 1) {
            Err(fail("controlled scope audit failure"))
        } else {
            Ok(json!({"watches":[]}))
        }
    }
    fn audit_request(&mut self, _: &Path, _: &Path, _: &RequestOptions) -> Result<Value> {
        let mut c = self.control.borrow_mut();
        c.request_audits += 1;
        if c.fail_request_audit || (c.fail_after_audit && c.request_audits > 1) {
            Err(fail("controlled request audit failure"))
        } else {
            Ok(json!({"watches":[]}))
        }
    }
    fn watches(&mut self, repo: &Path, d: &Value) -> Result<Vec<String>> {
        let mut result: Vec<_> = d["provenance"]["reads"]
            .as_object()
            .unwrap()
            .keys()
            .map(|p| repo.join(p).to_string_lossy().into_owned())
            .collect();
        result.push(
            repo.join("sdk/lib/rust/beep-rust/src")
                .to_string_lossy()
                .into_owned(),
        );
        Ok(result)
    }
}
struct Harness {
    _temp: tempfile::TempDir,
    repo: PathBuf,
    store: PathBuf,
    tools: PathBuf,
    context: PathBuf,
    host: FakeHost,
}
impl Harness {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let package = repo.join("sdk/lib/rust/beep-rust");
        fs::create_dir_all(package.join("src")).unwrap();
        fs::write(
            package.join("Cargo.toml"),
            "[package]\nname=\"beep-rust\"\nversion=\"0.1.0\"\n[lib]\ncrate-type=[\"staticlib\"]\n",
        )
        .unwrap();
        fs::write(package.join("src/lib.rs"), "pub fn body() {}\n").unwrap();
        fs::write(
            repo.join("sdk/lib/rust/Cargo.toml"),
            "[workspace]\nresolver=\"2\"\n",
        )
        .unwrap();
        fs::write(repo.join("sdk/lib/rust/Cargo.lock"), "version=4\n").unwrap();
        fs::write(
            repo.join("rust-toolchain.toml"),
            "[toolchain]\nchannel=\"stable\"\n",
        )
        .unwrap();
        let broker = repo.join(".cache/host-jobs/host-jobs");
        fs::create_dir_all(broker.parent().unwrap()).unwrap();
        fs::write(&broker, "#!/bin/sh\nexit 99\n").unwrap();
        fs::set_permissions(&broker, fs::Permissions::from_mode(0o755)).unwrap();
        let host = FakeHost::new(repo.clone());
        Self {
            store: temp.path().join("store"),
            tools: temp.path().join("toolchain"),
            context: temp.path().join("context"),
            repo,
            host,
            _temp: temp,
        }
    }
    fn open(&self, o: SupervisorOptions) -> Result<Supervisor<FakeHost>> {
        Supervisor::open_with_host(
            &self.repo,
            &self.store,
            &self.tools,
            &self.context,
            &o,
            self.host.clone(),
        )
    }
    fn request(&mut self, path: &Path, consumer: &str) -> Result<Value> {
        request_with_host(
            path,
            "beep-rust",
            consumer,
            &RequestOptions::default(),
            &mut self.host,
        )
    }
    fn summary(&self) -> Value {
        read_json(&self.context.join("summary.json")).unwrap()
    }
    fn store(&self) -> storage::Store {
        storage::Store::new(&self.store).unwrap()
    }
}
#[test]
fn miss_hit_owned_lease_archive_and_complete_receipts() {
    let mut h = Harness::new();
    let mut s = h.open(Default::default()).unwrap();
    let path = s.path().to_owned();
    let a = h.request(&path, "cargo:first").unwrap();
    let b = h.request(&path, "cargo:second").unwrap();
    assert_eq!(a["builder_calls"], 1);
    assert_eq!(b["builder_calls"], 0);
    assert_eq!(b["hit"], true);
    assert_eq!(a["archive"], b["archive"]);
    assert!(Path::new(a["archive"].as_str().unwrap()).is_file());
    assert!(a["watches"].as_array().unwrap().len() >= 5);
    assert_eq!(
        h.store()
            .list_refs(&h.repo.to_string_lossy())
            .unwrap()
            .len(),
        2
    );
    let token = supervisor_config(&path).unwrap()["session_token"]
        .as_str()
        .unwrap()
        .to_owned();
    let summary = s.close(None).unwrap();
    assert_eq!(summary["status"], "success");
    assert_eq!(summary["requests"]["hits"], 1);
    assert_eq!(summary["requests"]["misses"], 1);
    assert_eq!(summary["requests"]["builder_calls"], 1);
    assert_eq!(summary["toolchain_postcheck"], "identical");
    assert!(!summary.to_string().contains(&token));
    assert!(h.store().attach_session(&token).is_err());
}
#[test]
fn clean_rebuilds_existing_action_once_per_session() {
    let mut h = Harness::new();
    let mut s = h.open(Default::default()).unwrap();
    h.request(s.path(), "body:original").unwrap();
    s.close(None).unwrap();
    let mut s = h
        .open(SupervisorOptions {
            clean: true,
            ..Default::default()
        })
        .unwrap();
    let a = h.request(s.path(), "body:new").unwrap();
    let b = h.request(s.path(), "body:again").unwrap();
    assert_eq!(a["builder_calls"], 1);
    assert_eq!(a["verified_rebuild"], true);
    assert_eq!(b["builder_calls"], 0);
    s.close(None).unwrap();
    assert_eq!(h.host.control.borrow().executes, 2);
}
#[test]
fn verify_cold_builds_twice_and_coalesces() {
    let mut h = Harness::new();
    let mut s = h
        .open(SupervisorOptions {
            verify: true,
            ..Default::default()
        })
        .unwrap();
    let a = h.request(s.path(), "body:a").unwrap();
    let b = h.request(s.path(), "body:b").unwrap();
    assert_eq!(a["builder_calls"], 2);
    assert_eq!(a["verification_compared"], true);
    assert_eq!(b["builder_calls"], 0);
    let r = s.close(None).unwrap();
    assert_eq!(r["verification_success"], true);
    assert_eq!(r["requests"]["builder_calls"], 2);
}
#[test]
fn verify_existing_builds_once_and_empty_verify_refuses() {
    let mut h = Harness::new();
    let mut s = h.open(Default::default()).unwrap();
    h.request(s.path(), "body:a").unwrap();
    s.close(None).unwrap();
    let mut s = h
        .open(SupervisorOptions {
            verify: true,
            ..Default::default()
        })
        .unwrap();
    assert_eq!(h.request(s.path(), "body:b").unwrap()["builder_calls"], 1);
    s.close(None).unwrap();
    let mut s = h
        .open(SupervisorOptions {
            verify: true,
            ..Default::default()
        })
        .unwrap();
    assert!(s
        .close(None)
        .unwrap_err()
        .message
        .contains("no selected action compared"));
    assert_eq!(h.summary()["verification_success"], false);
    assert_eq!(h.summary()["toolchain_postcheck"], "identical");
}
#[test]
fn nondeterminism_fails_without_bind_or_success_receipt() {
    let mut h = Harness::new();
    h.host.control.borrow_mut().nondeterministic = true;
    let mut s = h
        .open(SupervisorOptions {
            verify: true,
            ..Default::default()
        })
        .unwrap();
    assert!(h.request(s.path(), "body:a").is_err());
    assert!(h
        .store()
        .list_refs(&h.repo.to_string_lossy())
        .unwrap()
        .is_empty());
    assert!(s.close(None).is_err());
    let summary = h.summary();
    assert_eq!(summary["status"], "failed");
    assert_eq!(summary["requests"]["failed_requests"], 1);
    assert_eq!(summary["requests"]["successful_requests"], 0);
}
#[test]
fn rejected_worker_context_report_and_compiler_never_publish() {
    for failure in ["prepare", "report", "execute"] {
        let mut h = Harness::new();
        {
            let mut c = h.host.control.borrow_mut();
            c.mismatch_prepare = failure == "prepare";
            c.invalid_report = failure == "report";
            c.fail_execute = failure == "execute";
        }
        let mut s = h.open(Default::default()).unwrap();
        assert!(h.request(s.path(), "body:a").is_err(), "{failure}");
        assert!(h
            .store()
            .list_refs(&h.repo.to_string_lossy())
            .unwrap()
            .is_empty());
        assert_eq!(s.close(None).unwrap()["requests"]["failed_requests"], 1);
    }
}
#[test]
fn source_edit_is_rejected_before_binding() {
    let mut h = Harness::new();
    h.host.control.borrow_mut().source_edit = true;
    let mut s = h.open(Default::default()).unwrap();
    assert!(h
        .request(s.path(), "body:a")
        .unwrap_err()
        .message
        .contains("lane inputs changed"));
    assert!(h
        .store()
        .list_refs(&h.repo.to_string_lossy())
        .unwrap()
        .is_empty());
    s.close(None).unwrap();
}
#[test]
fn expired_supervisor_rejects_before_discovery() {
    let mut h = Harness::new();
    let mut s = h.open(Default::default()).unwrap();
    let path = s.path().to_owned();
    s.close(None).unwrap();
    assert!(h.request(&path, "body:a").is_err());
    assert_eq!(h.host.control.borrow().discoveries, 0);
}
#[test]
fn missing_durable_blob_fails_preflight_and_persists_summary() {
    let mut h = Harness::new();
    let mut s = h.open(Default::default()).unwrap();
    let a = h.request(s.path(), "body:a").unwrap();
    s.close(None).unwrap();
    fs::remove_file(a["archive"].as_str().unwrap()).unwrap();
    assert!(h.open(Default::default()).is_err());
    assert_eq!(h.summary()["status"], "failed");
    assert_eq!(h.summary()["toolchain_postcheck"], "identical");
}
#[test]
fn final_toolchain_validation_runs_without_requests_and_on_outer_failure() {
    for outer in [false, true] {
        let h = Harness::new();
        let mut s = h.open(Default::default()).unwrap();
        h.host.control.borrow_mut().changed_tool = true;
        let failure = outer.then(|| json!({"type":"ControlledOuter","message":"outer failed"}));
        assert!(s
            .close(failure)
            .unwrap_err()
            .message
            .contains("toolchain/runtime changed"));
        let r = h.summary();
        assert_eq!(r["status"], "failed");
        assert_eq!(r["toolchain_postcheck"], "changed");
        assert_eq!(r["requests"]["requested"], 0);
    }
}
#[test]
fn config_audits_gate_requests_and_finalize_zero_request_builds() {
    let mut h = Harness::new();
    let mut s = h.open(Default::default()).unwrap();
    h.host.control.borrow_mut().fail_request_audit = true;
    assert!(h.request(s.path(), "body:a").is_err());
    assert_eq!(h.host.control.borrow().discoveries, 0);
    h.host.control.borrow_mut().fail_post_scope = true;
    assert!(s.close(None).is_err());
    assert_eq!(h.summary()["config_postcheck"], "failed");
    assert_eq!(h.summary()["requests"]["failed_requests"], 1);
    let h = Harness::new();
    h.host.control.borrow_mut().fail_scope = true;
    assert!(h.open(Default::default()).is_err());
    assert_eq!(h.summary()["config_preflight"], "failed");
}
#[test]
fn outer_exit_and_exception_are_recorded_without_false_success() {
    let h = Harness::new();
    let mut s = h.open(Default::default()).unwrap();
    record_outer_result(s.path(), 7).unwrap();
    let r = s.close(None).unwrap();
    assert_eq!(r["status"], "failed");
    assert_eq!(r["outer_exit_code"], 7);
    assert_eq!(r["error"]["type"], "OuterBuildFailure");
    let mut s = h.open(Default::default()).unwrap();
    assert!(s
        .close(Some(
            json!({"type":"ControlledOuter","message":"outer failed"})
        ))
        .is_err());
    assert_eq!(h.summary()["error"]["type"], "ControlledOuter");
    assert_eq!(h.summary()["toolchain_postcheck"], "identical");
}
#[test]
fn unsupported_options_fail_before_lookup_and_leave_failure_receipts() {
    let mut h = Harness::new();
    let mut s = h.open(Default::default()).unwrap();
    for o in [
        RequestOptions {
            build_std: true,
            ..Default::default()
        },
        RequestOptions {
            emit_ir: true,
            ..Default::default()
        },
        RequestOptions {
            force_staticlib: true,
            ..Default::default()
        },
        RequestOptions {
            flags: "-C bad".into(),
            ..Default::default()
        },
        RequestOptions {
            features: vec!["dep/feat".into()],
            ..Default::default()
        },
        RequestOptions {
            target: "i686-kernel".into(),
            ..Default::default()
        },
    ] {
        assert!(request_with_host(s.path(), "beep-rust", "body:a", &o, &mut h.host).is_err());
    }
    assert_eq!(h.host.control.borrow().discoveries, 0);
    assert_eq!(s.close(None).unwrap()["requests"]["failed_requests"], 6);
}
#[test]
fn ineligible_is_an_explicit_dispatch_and_strict_refuses() {
    let mut h = Harness::new();
    let mut s = h.open(Default::default()).unwrap();
    h.host.control.borrow_mut().ineligible = true;
    let a = h.request(s.path(), "body:a").unwrap();
    assert_eq!(a["status"], "ineligible");
    assert_eq!(h.host.control.borrow().executes, 0);
    assert!(protocol_lines(&a).unwrap().starts_with("ineligible\t"));
    let mut state = supervisor_config(s.path()).unwrap();
    state["strict"] = json!(true);
    fs::write(s.path(), serde_json::to_vec(&state).unwrap()).unwrap();
    assert!(h.request(s.path(), "body:b").is_err());
    let r = s.close(None).unwrap();
    assert_eq!(r["requests"]["ineligible_requests"], 1);
    assert_eq!(r["requests"]["failed_requests"], 1);
}
#[test]
fn protocol_refuses_relative_or_control_paths_and_sanitizes_reason() {
    assert_eq!(
        protocol_lines(&json!({"archive":"/a","watches":["/b"]})).unwrap(),
        "archive\t/a\nwatch\t/b\n"
    );
    for p in ["relative", "/a\tb", "/a\nb", "/a\0b"] {
        assert!(protocol_lines(&json!({"archive":p,"watches":[]})).is_err());
    }
    assert_eq!(
        protocol_lines(&json!({"status":"ineligible","reasons":["a\nb\tc"]})).unwrap(),
        "ineligible\ta b c\n"
    );
}
#[test]
fn durable_bind_failure_is_not_success() {
    let mut h = Harness::new();
    let mut s = h.open(Default::default()).unwrap();
    let store = h.store();
    let bad = store
        .ref_path(&h.repo.to_string_lossy(), "body:bad")
        .unwrap();
    fs::create_dir_all(&bad).unwrap();
    assert!(h.request(s.path(), "body:bad").is_err());
    let summary = s.close(None).unwrap();
    assert_eq!(summary["requests"]["successful_requests"], 0);
    assert_eq!(summary["requests"]["failed_requests"], 1);
}
#[test]
fn only_clean_supervisor_can_retire_its_arch_cargo_refs() {
    let mut h = Harness::new();
    let mut s = h.open(Default::default()).unwrap();
    let consumers = [
        "cargo:cell|/tmp/vibeos/sdk/lib/rust/m-amd64-cargo-link/a/out|x",
        "cargo:cell|/tmp/vibeos/sdk/lib/rust/m-i386-cargo-link/a/out|x",
        "cargo:cell|/tmp/vibeos/sdk/lib/rust/cell-link-v1-amd64-0/a/out|x",
        "body:standalone",
    ];
    for c in consumers {
        h.request(s.path(), c).unwrap();
    }
    assert!(retire_build_refs(s.path()).is_err());
    s.close(None).unwrap();
    let mut s = h
        .open(SupervisorOptions {
            clean: true,
            arch: Some("amd64".into()),
            ..Default::default()
        })
        .unwrap();
    let retired = retire_build_refs(s.path()).unwrap();
    assert_eq!(retired.len(), 2);
    let refs = h.store().list_refs(&h.repo.to_string_lossy()).unwrap();
    assert_eq!(refs.len(), 2);
    assert!(refs.iter().any(|r| r["consumer"] == consumers[1]));
    assert!(refs.iter().any(|r| r["consumer"] == "body:standalone"));
    s.close(None).unwrap();
}
#[test]
fn store_checkout_ancestors_are_refused() {
    let h = Harness::new();
    for store in [
        h.repo.clone(),
        h.repo.join("store"),
        h.repo.parent().unwrap().to_owned(),
    ] {
        assert!(Supervisor::open_with_host(
            &h.repo,
            &store,
            &h.tools,
            &h.context,
            &SupervisorOptions::default(),
            h.host.clone()
        )
        .is_err());
        assert_eq!(h.summary()["status"], "failed");
    }
}
#[test]
fn malformed_activity_is_reported_and_cannot_claim_success() {
    let h = Harness::new();
    let mut s = h.open(Default::default()).unwrap();
    let state = supervisor_config(s.path()).unwrap();
    fs::write(
        Path::new(state["activity"].as_str().unwrap()).join("bad.json"),
        "not JSON",
    )
    .unwrap();
    assert!(s.close(None).is_err());
    let r = h.summary();
    assert_eq!(r["status"], "failed");
    assert!(!r["receipt_error"].is_null());
    assert_eq!(r["toolchain_postcheck"], "identical");
}
#[test]
fn request_context_is_validated_before_store_lookup() {
    let mut h = Harness::new();
    let mut s = h.open(Default::default()).unwrap();
    let state = supervisor_config(s.path()).unwrap();
    let mut modified = state.clone();
    modified["provisioned"]["tree"] = json!("tampered");
    fs::write(s.path(), serde_json::to_vec(&modified).unwrap()).unwrap();
    assert!(h
        .request(s.path(), "body:a")
        .unwrap_err()
        .message
        .contains("descriptor changed"));
    assert_eq!(h.host.control.borrow().discoveries, 0);
    fs::write(s.path(), serde_json::to_vec(&state).unwrap()).unwrap();
    s.close(None).unwrap();
}
#[test]
fn native_broker_command_is_compile_only_and_uses_native_executor() {
    let h = Harness::new();
    let output = h._temp.path().join("execution");
    fs::create_dir(&output).unwrap();
    let broker = h.repo.join(".cache/host-jobs/host-jobs");
    let log = h._temp.path().join("broker-args");
    fs::write(&broker,format!("#!/bin/sh\nprintf '%s\\n' \"$@\" > '{}'\nprintf '{{\"done\":true}}' > '{}/report.json'\n",log.display(),output.display())).unwrap();
    let r = execute(&broker, &output).unwrap();
    assert_eq!(r["done"], true);
    let args = fs::read_to_string(&log).unwrap();
    let args: Vec<_> = args.lines().collect();
    assert_eq!(&args[..3], &["--kind", "compile", "--"]);
    // The child serves this repo explicitly, never its compile-time default.
    assert_eq!(args[4], "--root");
    assert_eq!(
        args[5],
        crate::artifact_executor::implementation_repo().to_str().unwrap()
    );
    assert_eq!(&args[6..8], &["artifact-executor", "run"]);
    assert_eq!(args[8], output.to_str().unwrap());
}
#[test]
fn missing_only_repair_restores_exact_bytes_without_creating_consumer() {
    let mut h = Harness::new();
    let mut s = h.open(Default::default()).unwrap();
    let a = h.request(s.path(), "body:owner").unwrap();
    s.close(None).unwrap();
    let action = a["action_key"].as_str().unwrap();
    let blob = Path::new(a["archive"].as_str().unwrap());
    let bytes = fs::read(blob).unwrap();
    fs::remove_file(blob).unwrap();
    let refs = h.store().list_refs(&h.repo.to_string_lossy()).unwrap();
    let mut s = h
        .open(SupervisorOptions {
            repair_action: Some(action.into()),
            ..Default::default()
        })
        .unwrap();
    let result = repair_with_host(s.path(), action, &mut h.host).unwrap();
    assert_eq!(result["repaired"], true);
    assert_eq!(result["reference"], Value::Null);
    assert_eq!(fs::read(blob).unwrap(), bytes);
    assert_eq!(
        h.store().list_refs(&h.repo.to_string_lossy()).unwrap(),
        refs
    );
    let r = s.close(None).unwrap();
    assert_eq!(r["repair_success"], true);
}
#[test]
fn repair_refuses_changed_current_source_present_blob_and_wrong_supervisor() {
    for mode in ["source", "present", "ordinary"] {
        let mut h = Harness::new();
        let mut s = h.open(Default::default()).unwrap();
        let a = h.request(s.path(), "body:owner").unwrap();
        s.close(None).unwrap();
        let action = a["action_key"].as_str().unwrap();
        if mode != "present" {
            fs::remove_file(a["archive"].as_str().unwrap()).unwrap();
        }
        if mode == "source" {
            fs::write(h.host.source(), "changed source").unwrap();
        }
        let mut s = h
            .open(SupervisorOptions {
                repair_action: Some(action.into()),
                ..Default::default()
            })
            .unwrap();
        let calls = h.host.control.borrow().executes;
        let selected = if mode == "ordinary" {
            "0".repeat(64)
        } else {
            action.into()
        };
        assert!(
            repair_with_host(s.path(), &selected, &mut h.host).is_err(),
            "{mode}"
        );
        assert_eq!(h.host.control.borrow().executes, calls);
        assert!(s.close(None).is_err());
        assert_eq!(h.summary()["repair_success"], false);
    }
}
#[test]
fn repair_only_cannot_serve_consumers_or_finish_without_repair() {
    let mut h = Harness::new();
    let mut s = h.open(Default::default()).unwrap();
    let a = h.request(s.path(), "body:owner").unwrap();
    s.close(None).unwrap();
    let mut s = h
        .open(SupervisorOptions {
            repair_action: Some(a["action_key"].as_str().unwrap().into()),
            ..Default::default()
        })
        .unwrap();
    assert!(h.request(s.path(), "body:new").is_err());
    assert!(s
        .close(None)
        .unwrap_err()
        .message
        .contains("no matching successful repair"));
}
#[test]
fn liveness_only_prunes_known_missing_cargo_outputs() {
    let h = Harness::new();
    assert!(!lane_ref_alive(
        &json!({"lane":h.repo.join("absent"),"consumer":"body:x"}),
        None
    )
    .unwrap());
    assert!(lane_ref_alive(&json!({"lane":h.repo,"consumer":"body:x"}), None).unwrap());
    assert!(lane_ref_alive(
        &json!({"lane":h.repo,"consumer":"cargo:unrecognized"}),
        None
    )
    .unwrap());
    let out = h.repo.join("build-amd64/out");
    fs::create_dir_all(&out).unwrap();
    let r = json!({"lane":h.repo,"consumer":"cargo:body|/tmp/vibeos/build-amd64/out|x"});
    assert!(lane_ref_alive(&r, Some("/tmp/vibeos")).unwrap());
    fs::remove_dir(&out).unwrap();
    assert!(!lane_ref_alive(&r, Some("/tmp/vibeos")).unwrap());
}
#[test]
fn scheduled_gc_respects_kill_switch_recent_stamp_and_claim() {
    let h = Harness::new();
    assert!(scheduled_gc(&h.store, None, false).unwrap().is_none());
    h.store();
    fs::write(h.store.join("gc-disabled"), "").unwrap();
    assert!(scheduled_gc(&h.store, None, true).unwrap().is_none());
    fs::remove_file(h.store.join("gc-disabled")).unwrap();
    let result = scheduled_gc(&h.store, None, false).unwrap();
    assert!(result.is_some());
    assert!(scheduled_gc(&h.store, None, false).unwrap().is_none());
    let claim = fs::OpenOptions::new()
        .write(true)
        .open(h.store.join("gc-claim.lock"))
        .unwrap();
    assert_eq!(
        unsafe { libc::flock(claim.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) },
        0
    );
    assert!(scheduled_gc(&h.store, None, true).unwrap().is_none());
}
#[test]
fn held_compile_permit_never_nests_broker() {
    let h = Harness::new();
    let broker = h.repo.join(".cache/host-jobs/host-jobs");
    let out = h._temp.path().join("unused");
    let mut called = false;
    let result = execute_with(&broker, &out, true, |seen| {
        assert_eq!(seen, out);
        called = true;
        Ok(json!({"held":true}))
    })
    .unwrap();
    assert!(called);
    assert_eq!(result["held"], true);
}
#[test]
fn repair_rejects_changed_tool_and_does_not_replace_corruption() {
    for corrupt in [false, true] {
        let mut h = Harness::new();
        let mut s = h.open(Default::default()).unwrap();
        let a = h.request(s.path(), "body:owner").unwrap();
        s.close(None).unwrap();
        let action = a["action_key"].as_str().unwrap();
        let blob = Path::new(a["archive"].as_str().unwrap());
        if corrupt {
            fs::set_permissions(blob, fs::Permissions::from_mode(0o600)).unwrap();
            fs::write(blob, "corrupt").unwrap();
        } else {
            fs::remove_file(blob).unwrap();
            h.host.control.borrow_mut().changed_tool = true;
        }
        let mut s = h
            .open(SupervisorOptions {
                repair_action: Some(action.into()),
                ..Default::default()
            })
            .unwrap();
        let calls = h.host.control.borrow().executes;
        assert!(repair_with_host(s.path(), action, &mut h.host).is_err());
        assert_eq!(h.host.control.borrow().executes, calls);
        if corrupt {
            assert_eq!(fs::read(blob).unwrap(), b"corrupt");
        }
        assert!(s.close(None).is_err());
    }
}
#[test]
fn ordinary_supervisor_cannot_request_repair() {
    let mut h = Harness::new();
    let mut s = h.open(Default::default()).unwrap();
    let calls = h.host.control.borrow().executes;
    assert!(repair_with_host(s.path(), &"a".repeat(64), &mut h.host)
        .unwrap_err()
        .message
        .contains("exact repair-only"));
    assert_eq!(h.host.control.borrow().executes, calls);
    assert_eq!(s.close(None).unwrap()["requests"]["failed_requests"], 1);
}
#[test]
fn missing_stage_broker_failure_keeps_private_and_public_failure_evidence() {
    let h = Harness::new();
    fs::remove_file(h.repo.join(".cache/host-jobs/host-jobs")).unwrap();
    assert!(h.open(Default::default()).is_err());
    let summary = h.summary();
    assert_eq!(summary["status"], "failed");
    assert!(summary["error"]["message"]
        .as_str()
        .unwrap()
        .contains("broker missing"));
    let private: Vec<_> = fs::read_dir(&h.context)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.is_dir())
        .collect();
    assert_eq!(private.len(), 1);
    assert_eq!(
        read_json(&private[0].join("summary.json")).unwrap(),
        summary
    );
}
#[test]
fn abandoned_supervisor_finalizes_and_does_not_silently_release_lease() {
    let h = Harness::new();
    let s = h.open(Default::default()).unwrap();
    let state = supervisor_config(s.path()).unwrap();
    drop(s);
    assert!(h
        .store()
        .attach_session(state["session_token"].as_str().unwrap())
        .is_err());
    assert_eq!(h.summary()["status"], "failed");
    assert_eq!(h.summary()["toolchain_postcheck"], "identical");
}
#[test]
fn source_edit_while_waiting_on_hit_and_config_postaudit_refuse_before_bind() {
    for config_change in [false, true] {
        let mut h = Harness::new();
        let mut s = h.open(Default::default()).unwrap();
        h.request(s.path(), "body:first").unwrap();
        {
            let mut c = h.host.control.borrow_mut();
            c.edit_on_revalidation = !config_change;
            c.fail_after_audit = config_change;
        }
        assert!(h.request(s.path(), "body:second").is_err());
        assert_eq!(h.host.control.borrow().prepares, 1);
        assert_eq!(
            h.store()
                .list_refs(&h.repo.to_string_lossy())
                .unwrap()
                .len(),
            1
        );
        s.close(None).unwrap();
    }
}
#[test]
fn staticlib_with_rlib_is_eligible_and_rlib_only_is_dispatch() {
    let mut h = Harness::new();
    let manifest = h.repo.join("sdk/lib/rust/beep-rust/Cargo.toml");
    let original = fs::read_to_string(&manifest).unwrap();
    fs::write(
        &manifest,
        original.replace("[\"staticlib\"]", "[\"staticlib\",\"rlib\"]"),
    )
    .unwrap();
    let mut s = h.open(Default::default()).unwrap();
    assert_eq!(h.request(s.path(), "body:a").unwrap()["builder_calls"], 1);
    fs::write(&manifest, original.replace("[\"staticlib\"]", "[\"rlib\"]")).unwrap();
    assert_eq!(
        h.request(s.path(), "body:b").unwrap()["status"],
        "ineligible"
    );
    s.close(None).unwrap();
}
#[test]
fn real_watch_inventory_covers_source_directories_and_refuses_escapes_or_links() {
    let mut h = Harness::new();
    let d = h
        .host
        .discover(
            &h.repo.clone(),
            "beep-rust",
            &json!({}),
            &RequestOptions::default(),
            &h.tools.clone(),
        )
        .unwrap();
    let w = watches(&h.repo, &d).unwrap();
    for path in [
        h.host.source(),
        h.host.source().parent().unwrap().to_owned(),
        h.repo.join("sdk/lib/rust/beep-rust"),
        h.repo.join("rust-toolchain.toml"),
    ] {
        assert!(w.contains(&path.to_string_lossy().into_owned()));
    }
    let mut bad = d.clone();
    bad["provenance"]["reads"]["../escape"] = json!({});
    assert!(watches(&h.repo, &bad).is_err());
    let source = h.host.source();
    let real = source.with_extension("saved");
    fs::rename(&source, &real).unwrap();
    std::os::unix::fs::symlink(&real, &source).unwrap();
    assert!(watches(&h.repo, &d).is_err());
}
#[test]
fn repair_acceptance_revalidates_every_lane_reference() {
    let mut h = Harness::new();
    let original = fs::read_to_string(h.host.source()).unwrap();
    let mut s = h.open(Default::default()).unwrap();
    let a = h.request(s.path(), "body:first").unwrap();
    fs::write(h.host.source(), "other body revision").unwrap();
    let b = h.request(s.path(), "body:second").unwrap();
    s.close(None).unwrap();
    fs::write(h.host.source(), original).unwrap();
    fs::remove_file(a["archive"].as_str().unwrap()).unwrap();
    fs::remove_file(b["archive"].as_str().unwrap()).unwrap();
    let action = a["action_key"].as_str().unwrap();
    let mut s = h
        .open(SupervisorOptions {
            repair_action: Some(action.into()),
            ..Default::default()
        })
        .unwrap();
    assert!(repair_with_host(s.path(), action, &mut h.host).is_err());
    assert!(Path::new(a["archive"].as_str().unwrap()).exists());
    assert!(s.close(None).is_err());
    assert_eq!(h.summary()["repair_success"], false);
}
#[test]
fn native_request_parser_preserves_real_build_options_and_repair_route() {
    let args = [
        "request",
        "--supervisor",
        "/ctx",
        "--body",
        "beep-rust",
        "--consumer",
        "cargo:cell|out",
        "--features",
        "b,a",
        "--no-default-features",
        "--kernel",
        "--flags=-C force-frame-pointers=yes",
        "--cargo-cwd",
        "/tmp/vibeos/sdk/lib/rust",
        "--arch",
        "arm64",
        "--target",
        "aarch64-pc-windows-gnullvm",
    ]
    .map(str::to_owned);
    match parse_cli(&args).unwrap() {
        CliCommand::Request {
            supervisor,
            body,
            consumer,
            options,
        } => {
            assert_eq!(supervisor, PathBuf::from("/ctx"));
            assert_eq!(body, "beep-rust");
            assert_eq!(consumer, "cargo:cell|out");
            assert_eq!(options.features, vec!["b", "a"]);
            assert!(!options.default_features);
            assert!(options.kernel);
            assert_eq!(options.flags, KERNEL_FLAGS);
            assert_eq!(options.architecture, "arm64");
            assert_eq!(options.target, "aarch64-pc-windows-gnullvm");
            assert_eq!(
                options.cargo_cwd,
                Some(PathBuf::from("/tmp/vibeos/sdk/lib/rust"))
            );
        }
        _ => panic!("wrong command"),
    };
    let args = ["repair", "--supervisor", "/ctx", "--action", "abc"].map(str::to_owned);
    assert!(matches!(parse_cli(&args).unwrap(),CliCommand::Repair{action,..} if action=="abc"));
}
#[test]
fn native_cli_rejects_missing_or_wrong_options_without_fallback() {
    for args in [
        vec![],
        vec!["request"],
        vec!["private-build"],
        vec!["repair", "--body", "m"],
        vec!["request", "--kernel=true"],
        vec!["request", "--supervisor", "a", "--supervisor", "b"],
    ] {
        assert!(parse_cli(&args.iter().map(|s| s.to_string()).collect::<Vec<_>>()).is_err());
    }
}
// Append within artifact_runtime_tests' module. These use actual configuration audits
// while retaining the native RuntimeHost fake compiler and real Store/Session locks.
struct ConfigHost {
    fake: FakeHost,
    environment: BTreeMap<String, String>,
    write_during_revalidation: Option<PathBuf>,
}
impl ConfigHost {
    fn new(h: &Harness) -> Self {
        Self {
            fake: h.host.clone(),
            environment: BTreeMap::from([(
                "CARGO_HOME".into(),
                h._temp
                    .path()
                    .join("empty-cargo-home")
                    .to_string_lossy()
                    .into_owned(),
            )]),
            write_during_revalidation: None,
        }
    }
}
impl RuntimeHost for ConfigHost {
    fn provision(&mut self, p: &Path) -> Result<Value> {
        self.fake.provision(p)
    }
    fn context(
        &mut self,
        t: &Path,
        p: &Value,
        b: &str,
        o: &executor::ExecutorOptions,
    ) -> Result<Value> {
        self.fake.context(t, p, b, o)
    }
    fn discover(
        &mut self,
        r: &Path,
        b: &str,
        c: &Value,
        o: &RequestOptions,
        t: &Path,
    ) -> Result<Value> {
        self.fake.discover(r, b, c, o, t)
    }
    fn unchanged(&mut self, d: &Value) -> Result<bool> {
        if let Some(path) = self.write_during_revalidation.take() {
            fs::create_dir_all(path.parent().unwrap())?;
            fs::write(path, "[build]\nrustflags=[\"--cfg=changed\"]\n")?;
        }
        self.fake.unchanged(d)
    }
    fn prepare(
        &mut self,
        r: &Path,
        b: &str,
        t: &Path,
        o: &Path,
        opts: &executor::ExecutorOptions,
    ) -> Result<Value> {
        self.fake.prepare(r, b, t, o, opts)
    }
    fn execute(&mut self, b: &Path, o: &Path) -> Result<Value> {
        self.fake.execute(b, o)
    }
    fn watches(&mut self, r: &Path, d: &Value) -> Result<Vec<String>> {
        self.fake.watches(r, d)
    }
    fn audit_scope(&mut self, r: &Path, b: &Path) -> Result<Value> {
        config::audit(
            r,
            &config::cargo_scope(r)?,
            Some(&self.environment),
            Some(b),
            Some(TARGET),
            false,
            Path::new(config::CANONICAL_ROOT),
        )
    }
    fn audit_request(&mut self, r: &Path, b: &Path, o: &RequestOptions) -> Result<Value> {
        let cwd = config::mapped_cwd(
            r,
            o.cargo_cwd.as_deref(),
            o.kernel,
            Path::new(config::CANONICAL_ROOT),
        )?;
        config::audit(
            r,
            &[cwd],
            Some(&self.environment),
            Some(b),
            Some(&o.target),
            !o.flags.trim().is_empty(),
            Path::new(config::CANONICAL_ROOT),
        )
    }
}
fn open_guarded(h: &Harness) -> Result<Supervisor<ConfigHost>> {
    Supervisor::open_with_host(
        &h.repo,
        &h.store,
        &h.tools,
        &h.context,
        &SupervisorOptions::default(),
        ConfigHost::new(h),
    )
}
#[test]
fn actual_forbidden_config_blocks_preflight_before_provision() {
    let h = Harness::new();
    let config = h.repo.join("sdk/lib/rust/beep-rust/.cargo/config.toml");
    fs::create_dir_all(config.parent().unwrap()).unwrap();
    fs::write(config, "[profile.release]\nopt-level=3\n").unwrap();
    assert!(open_guarded(&h).is_err());
    assert_eq!(h.host.control.borrow().provisions, 0);
    let summary = h.summary();
    assert_eq!(summary["config_preflight"], "failed");
    assert_eq!(summary["status"], "failed");
}
#[test]
fn actual_postcheck_detects_new_config_without_any_request() {
    let h = Harness::new();
    let mut supervisor = open_guarded(&h).unwrap();
    let config = h.repo.join("build-amd64/.cargo/config.toml");
    fs::create_dir_all(config.parent().unwrap()).unwrap();
    fs::write(config, "[env]\nOBSERVED=\"changed\"\n").unwrap();
    assert!(supervisor.close(None).is_err());
    assert_eq!(h.summary()["config_postcheck"], "failed");
    assert_eq!(h.summary()["requests"]["requested"], 0);
}
#[test]
fn actual_config_changed_after_hit_is_refused_before_binding() {
    let h = Harness::new();
    let mut supervisor = open_guarded(&h).unwrap();
    let mut host = ConfigHost::new(&h);
    let path = supervisor.path();
    request_with_host(
        path,
        "beep-rust",
        "cargo:first",
        &RequestOptions::default(),
        &mut host,
    )
    .unwrap();
    host.write_during_revalidation = Some(h.repo.join("build-amd64/.cargo/config.toml"));
    assert!(request_with_host(
        path,
        "beep-rust",
        "cargo:must-not-bind",
        &RequestOptions::default(),
        &mut host
    )
    .is_err());
    let refs = h.store().list_refs(&h.repo.to_string_lossy()).unwrap();
    assert_eq!(refs.len(), 1);
    assert_eq!(refs[0]["consumer"], "cargo:first");
    assert_eq!(h.host.control.borrow().executes, 1);
    assert!(supervisor.close(None).is_err());
}
#[test]
fn actual_inert_config_edit_keeps_action_hit_and_file_watch() {
    let h = Harness::new();
    let config = h.repo.join("sdk/lib/rust/.cargo/config.toml");
    fs::create_dir_all(config.parent().unwrap()).unwrap();
    fs::write(
        &config,
        "[target.x86_64-pc-windows-gnu]\nlinker=\"first\"\n",
    )
    .unwrap();
    let mut supervisor = open_guarded(&h).unwrap();
    let mut host = ConfigHost::new(&h);
    let options = RequestOptions {
        cargo_cwd: Some(h.repo.join("sdk/lib/rust")),
        ..Default::default()
    };
    let first = request_with_host(
        supervisor.path(),
        "beep-rust",
        "cargo:first",
        &options,
        &mut host,
    )
    .unwrap();
    fs::write(
        &config,
        "[target.x86_64-pc-windows-gnu]\nlinker=\"second\"\n",
    )
    .unwrap();
    let second = request_with_host(
        supervisor.path(),
        "beep-rust",
        "cargo:second",
        &options,
        &mut host,
    )
    .unwrap();
    assert_eq!(first["action_key"], second["action_key"]);
    assert_eq!(second["hit"], true);
    assert_eq!(second["builder_calls"], 0);
    assert!(second["watches"]
        .as_array()
        .unwrap()
        .contains(&json!(config)));
    assert_eq!(supervisor.close(None).unwrap()["status"], "success");
}
#[test]
fn native_retirement_keeps_foreign_standalone_and_current_lease_pins() {
    let mut h = Harness::new();
    let mut supervisor = h.open(Default::default()).unwrap();
    let original = h.request(supervisor.path(), "cargo:obsolete").unwrap();
    h.request(supervisor.path(), "standalone:body").unwrap();
    supervisor.close(None).unwrap();
    let store = h.store();
    let key = original["action_key"].as_str().unwrap();
    let mut foreign = store.session().unwrap();
    store.lookup(key, &foreign).unwrap();
    store
        .bind("other-lane", "cargo:foreign", key, &foreign)
        .unwrap();
    foreign.close().unwrap();
    let mut supervisor = h
        .open(SupervisorOptions {
            clean: true,
            ..Default::default()
        })
        .unwrap();
    let retired = retire_build_refs(supervisor.path()).unwrap();
    assert_eq!(retired.len(), 1);
    assert_eq!(retired[0]["consumer"], "cargo:obsolete");
    assert_eq!(store.list_refs("other-lane").unwrap().len(), 1);
    let refs = store.list_refs(&h.repo.to_string_lossy()).unwrap();
    assert_eq!(refs.len(), 1);
    assert_eq!(refs[0]["consumer"], "standalone:body");
    assert!(retire_build_refs(supervisor.path()).unwrap().is_empty());
    let owner_lane = h.repo.to_string_lossy().into_owned();
    for (lane, consumer) in [
        (owner_lane.as_str(), "standalone:body"),
        ("other-lane", "cargo:foreign"),
    ] {
        let mut guard = store.consumer_guard(lane, consumer).unwrap();
        store.release_ref(lane, consumer, &guard).unwrap();
        guard.close();
    }
    let report = store.gc(true, 0.0, None).unwrap();
    assert!(report["actions"].as_array().unwrap().is_empty());
    assert!(report["blobs"].as_array().unwrap().is_empty());
    assert!(Path::new(original["archive"].as_str().unwrap()).exists());
    supervisor.close(None).unwrap();
}
#[test]
fn every_manifest_staticlib_is_a_candidate_and_missing_package_is_ineligible() {
    let mut h = Harness::new();
    for (body, kinds) in [
        ("other-rust", "[\"staticlib\"]"),
        ("both-rust", "[\"staticlib\",\"rlib\"]"),
    ] {
        let package = h.repo.join(format!("sdk/lib/rust/{body}"));
        fs::create_dir_all(package.join("src")).unwrap();
        fs::write(
            package.join("Cargo.toml"),
            format!("[package]\nname=\"{body}\"\nversion=\"0.1.0\"\n[lib]\ncrate-type={kinds}\n"),
        )
        .unwrap();
    }
    let mut supervisor = h.open(Default::default()).unwrap();
    for body in ["other-rust", "both-rust"] {
        let result = request_with_host(
            supervisor.path(),
            body,
            body,
            &RequestOptions::default(),
            &mut h.host,
        )
        .unwrap();
        assert_eq!(result["builder_calls"], 1);
    }
    let missing = request_with_host(
        supervisor.path(),
        "no-such-rust",
        "missing",
        &RequestOptions::default(),
        &mut h.host,
    )
    .unwrap();
    assert_eq!(missing["status"], "ineligible");
    supervisor.close(None).unwrap();
}
#[test]
fn failed_request_receipt_survives_final_toolchain_failure() {
    let mut h = Harness::new();
    let mut supervisor = h.open(Default::default()).unwrap();
    let state = supervisor_config(supervisor.path()).unwrap();
    h.host.control.borrow_mut().fail_execute = true;
    assert!(h.request(supervisor.path(), "cargo:failed").is_err());
    h.host.control.borrow_mut().changed_tool = true;
    assert!(supervisor.close(None).is_err());
    let summary = h.summary();
    assert_eq!(summary["toolchain_postcheck"], "changed");
    assert_eq!(summary["requests"]["failed_requests"], 1);
    assert_eq!(summary["requests"]["successful_requests"], 0);
    assert_eq!(
        summary["requests"]["failed_request_builder_calls"],
        "unknown"
    );
    assert!(!summary
        .to_string()
        .contains(state["session_token"].as_str().unwrap()));
}
