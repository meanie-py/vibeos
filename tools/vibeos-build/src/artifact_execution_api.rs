//! Executor RPC and dedicated outer-build supervision, retaining real native resources.
use crate::{
    artifact_config as config, artifact_executor as ex, artifact_runtime as rt, Error, Result,
};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{json, Value};
use std::{
    cell::RefCell,
    collections::BTreeMap,
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    rc::Rc,
};
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
    fn path(&self, key: &str) -> Result<PathBuf> {
        self.req(key)
    }
    fn executor(&self) -> Result<ex::ExecutorOptions> {
        Ok(ex::ExecutorOptions {
            features: self.opt("features", vec![])?,
            default_features: self.opt("default_features", true)?,
            kernel: self.opt("kernel", true)?,
            provisioned: self.opt("provisioned", None)?,
            target: self.opt("target", ex::AMD64.to_owned())?,
            flags: self.opt("flags", None)?,
        })
    }
}
pub fn dispatch(request: &Value) -> Result<Value> {
    let op = request["op"]
        .as_str()
        .ok_or("missing execution operation")?;
    let a = Args(&request["kwargs"]);
    match op {
        "artifact_executor_constants" => Ok(
            json!({"PIN":ex::PIN,"CANON":ex::CANON,"WORKSPACE":ex::WORKSPACE,"PREPARED_SCHEMA":ex::PREPARED_SCHEMA,"AMD64":ex::AMD64,"TARGETS":ex::TARGETS,"IMPLEMENTATION_REPO":ex::implementation_repo(),"FLAGS":[["amd64",false,ex::flags("amd64",false)?],["amd64",true,ex::flags("amd64",true)?],["arm64",false,ex::flags("arm64",false)?],["arm64",true,ex::flags("arm64",true)?],["i386",false,ex::flags("i386",false)?],["i386",true,ex::flags("i386",true)?]]}),
        ),
        "artifact_executor_file_digest" => ex::file_digest(&a.path("path")?),
        "artifact_executor_allocated" => ex::allocated(&a.path("root")?),
        "artifact_executor_value_digest" => val(ex::value_digest(&a.req::<Value>("value")?)?),
        "artifact_executor_implementation_identity" => ex::implementation_identity(),
        "artifact_executor_tree_identity" => ex::tree_identity(&a.path("root")?),
        "artifact_executor__provisioned_copy" => ex::provisioned_copy(&a.req::<Value>("tool")?),
        "artifact_executor_provision" => ex::provision(&a.path("toolchain")?),
        "artifact_executor_normalize" => ex::normalize(
            &a.path("snapshot")?,
            &a.req::<Value>("projection")?,
            &a.path("destination")?,
        ),
        "artifact_executor_launch" => {
            val(ex::launch(&a.path("config_path")?, a.opt("probe", false)?)?)
        }
        "artifact_executor_validate_graph" => ex::validate_graph(
            &a.path("output")?,
            &a.req::<Value>("descriptor")?,
            &a.req::<Value>("config")?,
        ),
        "artifact_executor_prepared_identity" => ex::prepared_identity(
            &a.req::<Value>("config")?,
            &a.path("output")?,
            &a.req::<Value>("context")?,
        ),
        "artifact_executor_validate_prepared" => {
            ex::validate_prepared(&a.path("output")?, &a.req::<Value>("report")?)
        }
        "artifact_executor_request_context" => ex::request_context(
            &a.path("toolchain")?,
            &a.req::<Value>("provisioned")?,
            &a.req::<String>("body")?,
            &a.executor()?,
        ),
        "artifact_executor_prepare" => ex::prepare(
            &a.path("repo")?,
            &a.req::<String>("body")?,
            &a.path("toolchain")?,
            &a.path("output")?,
            &a.executor()?,
        ),
        "artifact_executor_run" => ex::run(&a.path("output")?),
        "artifact_runtime_supervisor_config" => rt::supervisor_config(&a.path("path")?),
        "artifact_runtime_record_outer_result" => {
            rt::record_outer_result(&a.path("supervisor")?, a.req("exit_code")?)?;
            Ok(Value::Null)
        }
        "artifact_runtime_retire_build_refs" => val(rt::retire_build_refs(&a.path("supervisor")?)?),
        "artifact_runtime_lane_ref_alive" => val(rt::lane_ref_alive(
            &a.req::<Value>("reference")?,
            a.opt::<Option<String>>("canonical_root", None)?.as_deref(),
        )?),
        "artifact_runtime_scheduled_gc" => val(rt::scheduled_gc(
            &a.path("store_root")?,
            a.opt("now", None)?,
            a.opt("force", false)?,
        )?),
        "artifact_runtime_protocol_lines" => val(rt::protocol_lines(&a.req::<Value>("result")?)?),
        "artifact_runtime__watches" => val(rt::watches(
            &a.path("repo")?,
            &a.req::<Value>("discovery")?,
        )?),
        "artifact_runtime__receipt_stats" => rt::receipt_stats(&a.path("activity")?),
        _ => Err(Error::new(format!("unknown execution operation {op}"))),
    }
}
struct Wire {
    input: BufReader<std::io::Stdin>,
    output: std::io::Stdout,
}
impl Wire {
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
    fn environment(&mut self) -> Result<BTreeMap<String, String>> {
        self.send(&json!({"callback":{"kind":"environment","arguments":{}}}))?;
        let response = self
            .read()?
            .ok_or("supervisor environment client disconnected")?;
        if let Some(error) = response.get("callback_error") {
            return Err(Error::classified(
                error["kind"].as_str().unwrap_or("RuntimeError_"),
                error["message"]
                    .as_str()
                    .unwrap_or("environment callback failed"),
            ));
        }
        Ok(serde_json::from_value(
            response
                .get("callback_result")
                .cloned()
                .ok_or("missing supervisor environment callback")?,
        )?)
    }
}
struct EnvironmentHost {
    environment: BTreeMap<String, String>,
    callback: Option<Rc<RefCell<Wire>>>,
}
impl EnvironmentHost {
    fn refresh(&mut self) -> Result<()> {
        if let Some(wire) = &self.callback {
            self.environment = wire.borrow_mut().environment()?;
        }
        Ok(())
    }
}
impl rt::RuntimeHost for EnvironmentHost {
    fn audit_scope(&mut self, repo: &Path, broker: &Path) -> Result<Value> {
        self.refresh()?;
        config::audit(
            repo,
            &config::cargo_scope(repo)?,
            Some(&self.environment),
            Some(broker),
            Some(rt::TARGET),
            false,
            Path::new(config::CANONICAL_ROOT),
        )
        .map_err(|e| e.with_kind("ConfigError"))
    }
    fn audit_request(
        &mut self,
        repo: &Path,
        broker: &Path,
        o: &rt::RequestOptions,
    ) -> Result<Value> {
        self.refresh()?;
        let cwd = config::mapped_cwd(
            repo,
            o.cargo_cwd.as_deref(),
            o.kernel,
            Path::new(config::CANONICAL_ROOT),
        )
        .map_err(|e| e.with_kind("ConfigError"))?;
        let mut report = config::audit(
            repo,
            &[cwd.clone()],
            Some(&self.environment),
            Some(broker),
            Some(&o.target),
            !o.flags.trim().is_empty(),
            Path::new(config::CANONICAL_ROOT),
        )
        .map_err(|e| e.with_kind("ConfigError"))?;
        report["mapped_cwd"] = json!(cwd);
        Ok(report)
    }
}
fn host(a: &Args, wire: &Rc<RefCell<Wire>>) -> Result<EnvironmentHost> {
    Ok(EnvironmentHost {
        environment: a.opt("environment", std::env::vars().collect())?,
        callback: if a.opt("environment_callback", false)? {
            Some(wire.clone())
        } else {
            None
        },
    })
}
pub fn serve() -> Result<u8> {
    let wire = Rc::new(RefCell::new(Wire {
        input: BufReader::new(std::io::stdin()),
        output: std::io::stdout(),
    }));
    let mut supervisor: Option<rt::Supervisor<EnvironmentHost>> = None;
    let outcome = (|| -> Result<()> {
        loop {
            let request = match wire.borrow_mut().read()? {
                Some(request) => request,
                None => break,
            };
            let operation = (|| -> Result<Value> {
                let op = request["op"]
                    .as_str()
                    .ok_or("missing supervisor operation")?;
                let a = Args(&request["kwargs"]);
                match op {
                    "open" => {
                        if supervisor.is_some() {
                            return Err("supervisor already open on this connection".into());
                        }
                        let options: rt::SupervisorOptions =
                            a.opt("options", Default::default())?;
                        let owned = rt::Supervisor::open_with_host(
                            &a.path("repo")?,
                            &a.path("store_root")?,
                            &a.path("toolchain")?,
                            &a.path("context_dir")?,
                            &options,
                            host(&a, &wire)?,
                        )?;
                        let path = owned.path().to_owned();
                        supervisor = Some(owned);
                        val(path)
                    }
                    "close" => {
                        let mut owned = supervisor
                            .take()
                            .ok_or("supervisor is not open on this connection")?;
                        owned.host = host(&a, &wire)?;
                        owned.close(a.opt("outer_failure", None)?)
                    }
                    "request" => {
                        let options: rt::RequestOptions = a.opt("options", Default::default())?;
                        rt::request_with_host(
                            &a.path("supervisor")?,
                            &a.req::<String>("body")?,
                            &a.req::<String>("consumer")?,
                            &options,
                            &mut host(&a, &wire)?,
                        )
                    }
                    "repair" => rt::repair_with_host(
                        &a.path("supervisor")?,
                        &a.req::<String>("action")?,
                        &mut host(&a, &wire)?,
                    ),
                    _ => dispatch(&request),
                }
            })();
            let response = match operation {
                Ok(value) => json!({"ok":value}),
                Err(error) => json!({"error":error}),
            };
            wire.borrow_mut().send(&response)?;
        }
        Ok(())
    })();
    if let Some(mut supervisor) = supervisor {
        let _=supervisor.close(Some(json!({"type":"ClientDisconnected","message":"outer build client disconnected before supervision completed"})));
    }
    outcome?;
    Ok(0)
}
