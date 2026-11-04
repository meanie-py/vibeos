//! JSON operations used by the SDK recipe compatibility frontend.
use crate::sdk_recipe as s;
use crate::{Error, Result};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
fn val<T: Serialize>(v: T) -> Result<Value> {
    Ok(serde_json::to_value(v)?)
}
struct Args<'a> {
    op: &'a str,
    args: &'a [Value],
    kw: &'a serde_json::Map<String, Value>,
}
impl<'a> Args<'a> {
    fn new(v: &'a Value) -> Result<Self> {
        let op = v
            .get("op")
            .and_then(Value::as_str)
            .ok_or_else(|| Error::new("request requires op"))?;
        let args = match v.get("args") {
            Some(x) => x
                .as_array()
                .ok_or_else(|| Error::new("args must be array"))?
                .as_slice(),
            None => &[],
        };
        static EMPTY: std::sync::OnceLock<serde_json::Map<String, Value>> =
            std::sync::OnceLock::new();
        let kw = match v.get("kwargs") {
            Some(x) => x
                .as_object()
                .ok_or_else(|| Error::new("kwargs must be object"))?,
            None => EMPTY.get_or_init(Default::default),
        };
        Ok(Self { op, args, kw })
    }
    fn bind(&self, n: &[&str]) -> Result<()> {
        if self.args.len() > n.len() {
            return Err(Error::new(format!("{}: too many arguments", self.op)));
        }
        for k in self.kw.keys() {
            let i = n
                .iter()
                .position(|x| *x == k)
                .ok_or_else(|| Error::new(format!("{}: unexpected keyword {k}", self.op)))?;
            if i < self.args.len() {
                return Err(Error::new(format!("{}: duplicate argument {k}", self.op)));
            }
        }
        Ok(())
    }
    fn get(&self, n: &[&str], k: &str) -> Option<&Value> {
        let i = n.iter().position(|x| *x == k)?;
        self.args.get(i).or_else(|| self.kw.get(k))
    }
    fn req<T: DeserializeOwned>(&self, n: &[&str], k: &str) -> Result<T> {
        serde_json::from_value(
            self.get(n, k)
                .ok_or_else(|| Error::new(format!("{}: missing {k}", self.op)))?
                .clone(),
        )
        .map_err(|e| Error::new(format!("{}: invalid {k}: {e}", self.op)))
    }
    fn opt<T: DeserializeOwned>(&self, n: &[&str], k: &str, default: T) -> Result<T> {
        match self.get(n, k) {
            None => Ok(default),
            Some(v) => serde_json::from_value(v.clone())
                .map_err(|e| Error::new(format!("{}: invalid {k}: {e}", self.op))),
        }
    }
    fn root(&self, n: &[&str], root: &Path) -> Result<()> {
        let repo: Option<PathBuf> = self.opt(n, "repo", None)?;
        if let Some(r) = repo {
            if r != root && r.canonicalize()? != root.canonicalize()? {
                return Err(Error::new("repo differs from selected root"));
            }
        }
        Ok(())
    }
}
macro_rules! bind{($a:expr,$($n:literal),* $(,)?)=>{{let names=&[$($n),*];$a.bind(names)?;names}}}
/// Dispatch JSON requests. Replay execution is explicit (`sdk_run`) and never a fallback.
pub fn dispatch(root: &Path, request: &Value) -> Result<Value> {
    let a = Args::new(request)?;
    match a.op {
        "sdk_recipe_path" => {
            let n = bind!(a, "arch", "host", "repo");
            a.root(n, root)?;
            let arch = a.req::<String>(n, "arch")?;
            let host = a.opt(n, "host", false)?;
            val(s::recipe_path(root, &arch, host))
        }
        "sdk__ninja_commands" => {
            let n = bind!(a, "build", "targets");
            val(s::ninja_commands(
                root,
                &a.req::<PathBuf>(n, "build")?,
                &a.req::<Vec<String>>(n, "targets")?,
            )?)
        }
        "sdk__tool" | "sdk__keep" | "sdk__unwrap" => {
            let n = bind!(a, "line");
            let line = a.req::<String>(n, "line")?;
            match a.op {
                "sdk__tool" => val(s::tool(&line)?),
                "sdk__keep" => val(s::keep(&line)?),
                _ => val(s::unwrap(&line)),
            }
        }
        "sdk__cell_deps" => {
            let n = bind!(a, "ninja");
            val(s::cell_deps(&a.req::<String>(n, "ninja")?))
        }
        "sdk__encode" => {
            let n = bind!(a, "line", "arch");
            val(s::encode(
                &a.req::<String>(n, "line")?,
                &a.req::<String>(n, "arch")?,
            ))
        }
        "sdk__factor" => {
            let n = bind!(a, "lines");
            val(s::factor(&a.req::<Vec<String>>(n, "lines")?)?)
        }
        "sdk__majority_run" => {
            let n = bind!(a, "lists", "share");
            val(s::majority_run(
                &a.req::<Vec<Vec<String>>>(n, "lists")?,
                a.opt(n, "share", 0.95)?,
            ))
        }
        "sdk_derive" => {
            let n = bind!(a, "arch", "repo");
            a.root(n, root)?;
            val(s::derive(root, &a.req::<String>(n, "arch")?)?)
        }
        "sdk_plan" | "sdk_check" => {
            let n = bind!(a, "arches", "repo");
            a.root(n, root)?;
            let arches = s::selected_arches(&a.opt::<Vec<String>>(n, "arches", vec![])?)?;
            if a.op == "sdk_plan" {
                val(s::write_plan(root, &arches)?)
            } else {
                val(s::check(root, &arches)?)
            }
        }
        "sdk_replay_plan" | "sdk_run" => {
            let n = bind!(a, "arch", "gen", "repo");
            a.root(n, root)?;
            let plan = s::replay_plan(
                root,
                &a.req::<String>(n, "arch")?,
                &a.req::<PathBuf>(n, "gen")?,
            )?;
            if a.op == "sdk_run" {
                s::run(&plan)?;
                Ok(Value::Null)
            } else {
                val(plan)
            }
        }
        "sdk_constants" => {
            a.bind(&[])?;
            Ok(
                json!({"ARCHES":crate::ARCHES,"CANON":s::CANON,"GENERATORS":s::GENERATORS,"HOST_TOOLS":s::HOST_TOOLS,"MEDIA_TOOLS":s::MEDIA_TOOLS}),
            )
        }
        _ => Err(Error::new(format!(
            "unknown SDK recipe operation '{}'",
            a.op
        ))),
    }
}
