//! Compatibility transport for native artifact projections; no fallback implementation.
use crate::{
    artifact_config as config, artifact_fs as af, artifact_inputs as inputs,
    artifact_worker as worker, Error, Result,
};
use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};
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
    fn env(&self) -> Result<Option<BTreeMap<String, String>>> {
        self.opt("environment", None)
    }
    fn discovery(&self) -> Result<inputs::DiscoverOptions> {
        Ok(inputs::DiscoverOptions {
            features: self.opt("features", vec![])?,
            default_features: self.opt("default_features", true)?,
            profile: self.opt("profile", "release".into())?,
            force_staticlib: self.opt("force_staticlib", false)?,
            cargo: self.opt("cargo", "cargo".into())?,
            workspace: self.opt("workspace", None)?,
            scratch: self.opt("scratch", None)?,
        })
    }
}
pub fn dispatch(_root: &Path, request: &Value) -> Result<Value> {
    let op = request["op"]
        .as_str()
        .ok_or("artifact operation must be a string")?;
    let kw = &request["kwargs"];
    if !kw.is_object() {
        return Err("artifact kwargs must be an object".into());
    }
    let a = Args(kw);
    match op {
        "artifact_inputs_canonical" => {
            Ok(json!(crate::json::canonical(&a.req::<Value>("value")?)?))
        }
        "artifact_inputs_digest" => Ok(json!(inputs::digest(&a.req::<Value>("value")?)?)),
        "artifact_inputs__toml" => Ok(json!(inputs::toml_value(&a.req::<Value>("value")?)?)),
        "artifact_inputs__without_comments" => {
            Ok(json!(inputs::without_comments(&a.req::<String>("text")?)?))
        }
        "artifact_inputs_explain" => Ok(inputs::explain(
            &a.req::<Value>("before")?,
            &a.req::<Value>("after")?,
        )),
        "artifact_inputs_unchanged" => Ok(json!(inputs::unchanged(&a.req::<Value>("discovery")?))),
        "artifact_inputs_discover" => Ok(inputs::discover(
            &a.path("repo_root")?,
            &a.req::<String>("body")?,
            a.opt("context", json!({}))?,
            &a.discovery()?,
        )),
        "artifact_config_cargo_scope" => Ok(json!(config::cargo_scope(&a.path("repo")?)?)),
        "artifact_config_mapped_cwd" => Ok(json!(config::mapped_cwd(
            &a.path("repo")?,
            a.opt::<Option<PathBuf>>("cargo_cwd", None)?.as_deref(),
            a.opt("kernel", false)?,
            &a.opt("canonical", PathBuf::from(config::CANONICAL_ROOT))?
        )?)),
        "artifact_config__environment" => {
            config::environment(
                &a.req("environment")?,
                a.opt::<Option<PathBuf>>("broker", None)?.as_deref(),
            )?;
            Ok(Value::Null)
        }
        "artifact_config__config_bytes" => Ok(json!(af::verified_read(
            &a.path("path")?,
            "Cargo configuration"
        )?)),
        "artifact_config__allowed" => {
            config::allowed(
                &a.req("config")?,
                &a.path("path")?,
                a.opt::<Option<String>>("requested", None)?.as_deref(),
                a.opt("rustflags_set", false)?,
            )?;
            Ok(Value::Null)
        }
        "artifact_config_audit_scope"
        | "artifact_config_audit_request"
        | "artifact_config__audit" => {
            let repo = a.path("repo")?;
            let canonical = a.opt("canonical", PathBuf::from(config::CANONICAL_ROOT))?;
            let broker = a.opt::<Option<PathBuf>>("broker", None)?;
            let env = a.env()?;
            let scope = op.ends_with("audit_scope");
            let request = op.ends_with("audit_request");
            let cwd = if request {
                Some(config::mapped_cwd(
                    &repo,
                    a.opt::<Option<PathBuf>>("cargo_cwd", None)?.as_deref(),
                    a.opt("kernel", false)?,
                    &canonical,
                )?)
            } else {
                None
            };
            let cwds = if let Some(cwd) = &cwd {
                vec![cwd.clone()]
            } else if scope {
                config::cargo_scope(&repo)?
            } else {
                a.req("cwds")?
            };
            let target = if scope {
                Some(config::TARGET.to_owned())
            } else {
                a.opt(
                    "target",
                    a.opt(
                        "requested",
                        if request {
                            Some(config::TARGET.to_owned())
                        } else {
                            None
                        },
                    )?,
                )?
            };
            let mut report = config::audit(
                &repo,
                &cwds,
                env.as_ref(),
                broker.as_deref(),
                target.as_deref(),
                a.opt("rustflags_set", false)?,
                &canonical,
            )?;
            if let Some(cwd) = cwd {
                report["mapped_cwd"] = json!(cwd);
            }
            Ok(report)
        }
        "artifact_worker__verified_copy" => Ok(json!(worker::verified_copy(
            &a.path("source")?,
            &a.path("output")?,
            &a.req("expected")?
        )?)),
        "artifact_worker_capture" => {
            let options = worker::CaptureOptions {
                expected_projection_key: a.opt("expected_projection_key", None)?,
                discovery: a.discovery()?,
                additional_inputs: a.opt("additional_inputs", Default::default())?,
            };
            worker::capture(
                &a.path("repo_root")?,
                &a.req::<String>("body")?,
                a.req("context")?,
                &a.path("destination")?,
                &options,
            )
        }
        "artifact_worker_executor_requirements" => Ok(worker::executor_requirements()),
        "artifact_history_capture" => Ok(json!(crate::artifact_history::capture(
            &a.path("repo")?,
            &a.req::<String>("revision")?,
            &a.req::<String>("body")?,
            &a.path("destination")?
        )?)),
        "artifact_economics_stable_read" => Ok(json!(crate::artifact_economics::stable_read(
            &a.path("path")?,
            a.0.get("expected")
        )?)),
        "artifact_economics_marker_fields" => Ok(json!(crate::artifact_economics::marker_fields(
            &a.req::<Vec<u8>>("payload")?
        )?)),
        "artifact_economics_map_bodies" => {
            let (mapping, body_order) =
                crate::artifact_economics::map_bodies_ordered(&a.req("report")?, &a.path("repo")?)?;
            Ok(json!({"mapping":mapping,"body_order":body_order}))
        }
        "artifact_economics_size_model" => {
            crate::artifact_economics::size_model(&a.req::<Vec<Value>>("rows")?)
        }
        "artifact_economics_size_by_architecture" => {
            crate::artifact_economics::size_by_architecture(&a.req::<Vec<Value>>("rows")?)
        }
        "artifact_economics_summarize" | "artifact_economics_analyze" => {
            let options = crate::artifact_economics::EconomicsOptions {
                body_order: a.opt("body_order", None)?,
                eligibility_top: a.opt("eligibility_top", 0)?,
                cargo: a.opt("cargo", "cargo".into())?,
                scratch: a.opt("scratch", None)?,
            };
            if op.ends_with("summarize") {
                crate::artifact_economics::summarize(&a.req("mapping")?, &a.path("repo")?, &options)
            } else {
                crate::artifact_economics::analyze(&a.req("inventory")?, &a.path("repo")?, &options)
            }
        }
        "artifact_implementation_identity" => implementation_identity(),
        _ => Err(Error::new(format!("unknown artifact operation {op}"))),
    }
}
/// Identity changes deliberately invalidate old actions; old stored objects remain intact.
pub fn implementation_identity() -> Result<Value> {
    use sha2::{Digest, Sha256};
    let sources = [
        (
            "tools/vibeos-build/src/lib.rs",
            include_bytes!("lib.rs").as_slice(),
        ),
        (
            "tools/vibeos-build/src/main.rs",
            include_bytes!("main.rs").as_slice(),
        ),
        (
            "tools/vibeos-build/src/artifact_inputs.rs",
            include_bytes!("artifact_inputs.rs").as_slice(),
        ),
        (
            "tools/vibeos-build/src/artifact_worker.rs",
            include_bytes!("artifact_worker.rs").as_slice(),
        ),
        (
            "tools/vibeos-build/src/artifact_config.rs",
            include_bytes!("artifact_config.rs").as_slice(),
        ),
        (
            "tools/vibeos-build/src/artifact_fs.rs",
            include_bytes!("artifact_fs.rs").as_slice(),
        ),
        (
            "tools/vibeos-build/src/artifact_api.rs",
            include_bytes!("artifact_api.rs").as_slice(),
        ),
        (
            "tools/vibeos-build/src/json.rs",
            include_bytes!("json.rs").as_slice(),
        ),
        (
            "tools/vibeos-build/Cargo.toml",
            include_bytes!("../Cargo.toml").as_slice(),
        ),
        (
            "tools/vibeos-build/Cargo.lock",
            include_bytes!("../Cargo.lock").as_slice(),
        ),
    ];
    let hashes: BTreeMap<_, _> = sources
        .into_iter()
        .map(|(name, bytes)| (name, format!("{:x}", Sha256::digest(bytes))))
        .collect();
    Ok(
        json!({"schema":"vibeos-artifact-native-policy-v1","sources":hashes,"executable":std::env::current_exe()?}),
    )
}
