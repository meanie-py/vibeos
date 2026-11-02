//! JSON dispatcher for cell/media manifest functions.
use crate::{cell_manifest as c, media_manifest as m, Error, Result};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};
fn val<T: Serialize>(v: T) -> Result<Value> {
    Ok(serde_json::to_value(v)?)
}
struct A<'a> {
    op: &'a str,
    args: &'a [Value],
    kw: &'a serde_json::Map<String, Value>,
}
impl<'a> A<'a> {
    fn new(v: &'a Value) -> Result<Self> {
        let o = v
            .as_object()
            .ok_or_else(|| Error::new("request must be an object"))?;
        let op = o
            .get("op")
            .and_then(Value::as_str)
            .ok_or_else(|| Error::new("missing op"))?;
        let args = match o.get("args") {
            None => &[][..],
            Some(v) => v
                .as_array()
                .ok_or_else(|| Error::new("args must be an array"))?,
        };
        static EMPTY: std::sync::OnceLock<serde_json::Map<String, Value>> =
            std::sync::OnceLock::new();
        let kw = match o.get("kwargs") {
            None => EMPTY.get_or_init(Default::default),
            Some(v) => v
                .as_object()
                .ok_or_else(|| Error::new("kwargs must be an object"))?,
        };
        Ok(Self { op, args, kw })
    }
    fn bind(&self, n: &[&str]) -> Result<()> {
        if self.args.len() > n.len() {
            return Err(Error::new(format!("{}: too many arguments", self.op)));
        }
        for key in self.kw.keys() {
            let i = n
                .iter()
                .position(|x| x == key)
                .ok_or_else(|| Error::new(format!("{}: unexpected argument {key}", self.op)))?;
            if i < self.args.len() {
                return Err(Error::new(format!("{}: duplicate argument {key}", self.op)));
            }
        }
        Ok(())
    }
    fn get(&self, n: &[&str], k: &str) -> Option<&Value> {
        let i = n.iter().position(|x| *x == k)?;
        self.args.get(i).or_else(|| self.kw.get(k))
    }
    fn req<T: DeserializeOwned>(&self, n: &[&str], k: &str) -> Result<T> {
        let v = self
            .get(n, k)
            .ok_or_else(|| Error::new(format!("{}: missing {k}", self.op)))?;
        serde_json::from_value(v.clone())
            .map_err(|e| Error::new(format!("{}: invalid {k}: {e}", self.op)))
    }
    fn opt<T: DeserializeOwned>(&self, n: &[&str], k: &str, d: T) -> Result<T> {
        match self.get(n, k) {
            None => Ok(d),
            Some(v) => serde_json::from_value(v.clone())
                .map_err(|e| Error::new(format!("{}: invalid {k}: {e}", self.op))),
        }
    }
    fn repo(&self, n: &[&str], root: &Path) -> Result<PathBuf> {
        if let Some(p) = self.opt::<Option<PathBuf>>(n, "repo", None)? {
            if p != root && p.canonicalize()? != root.canonicalize()? {
                return Err(Error::new("repo differs from selected root"));
            }
        }
        Ok(root.to_owned())
    }
}
macro_rules! bind{($a:expr,$($n:literal),* $(,)?)=>{{let n=&[$($n),*];$a.bind(n)?;n}}}
pub fn dispatch(root: &Path, request: &Value) -> Result<Value> {
    let a = A::new(request)?;
    match a.op {
        "cell_manifest_path" | "media_manifest_path" => {
            let n = bind!(a, "arch", "repo");
            let arch = a.req::<String>(n, "arch")?;
            let r = a.repo(n, root)?;
            val(if a.op == "cell_manifest_path" {
                c::manifest_path(&arch, &r)
            } else {
                m::manifest_path(&arch, &r)
            })
        }
        "cell__build_rel" => {
            let n = bind!(a, "path", "arch");
            val(c::build_rel(
                &a.req::<String>(n, "path")?,
                &a.req::<String>(n, "arch")?,
            )?)
        }
        "cell__post" => {
            let n = bind!(a, "cmd", "out", "arch");
            val(c::post(
                &a.req::<Vec<String>>(n, "cmd")?,
                &a.req::<String>(n, "out")?,
                &a.req::<String>(n, "arch")?,
            )?)
        }
        "cell_parse_cell" => {
            let n = bind!(a, "outs", "command", "arch");
            c::parse_cell(
                &a.req::<String>(n, "outs")?,
                &a.req::<String>(n, "command")?,
                &a.req::<String>(n, "arch")?,
            )
        }
        "cell_derive" | "cell_load" | "cell_current" => {
            let n = bind!(a, "arch", "repo");
            let arch = a.req::<String>(n, "arch")?;
            let r = a.repo(n, root)?;
            val(match a.op {
                "cell_derive" => c::derive(&arch, &r)?,
                "cell_load" => c::load(&arch, &r)?,
                _ => c::current(&arch, &r)?,
            })
        }
        "cell_render" => {
            let n = bind!(a, "cells");
            val(c::render(&a.req::<Vec<Value>>(n, "cells")?))
        }
        "cell_drift" => {
            let n = bind!(a, "committed", "derived");
            val(c::drift(
                &a.req::<Vec<Value>>(n, "committed")?,
                &a.req::<Vec<Value>>(n, "derived")?,
            )?)
        }
        "cell_write_plan" | "media_write_plan" => {
            let n = bind!(a, "arches", "repo");
            let r = a.repo(n, root)?;
            let arches = a.opt::<Vec<String>>(
                n,
                "arches",
                crate::ARCHES.iter().map(|s| s.to_string()).collect(),
            )?;
            val(if a.op == "cell_write_plan" {
                c::write_plan(&r, &arches)?
            } else {
                m::write_plan(&r, &arches)?
            })
        }
        "media__enc" => {
            let n = bind!(a, "path", "arch", "cells");
            val(m::enc(
                &a.req::<String>(n, "path")?,
                &a.req::<String>(n, "arch")?,
                &a.req::<BTreeSet<String>>(n, "cells")?,
            )?)
        }
        "media__enc_cmd" => {
            let n = bind!(a, "cmd", "arch");
            val(m::enc_cmd(
                &a.req::<String>(n, "cmd")?,
                &a.req::<String>(n, "arch")?,
            ))
        }
        "media__command" => {
            let n = bind!(a, "ninja", "cd");
            val(m::command(
                &a.req::<String>(n, "ninja")?,
                &a.req::<String>(n, "cd")?,
            ))
        }
        "media__output_command" => {
            let n = bind!(a, "ninja", "out");
            val(m::output_command(
                &a.req::<String>(n, "ninja")?,
                &a.req::<String>(n, "out")?,
            )?)
        }
        "media_derive_cab" => {
            let n = bind!(a, "arch", "build", "ninja", "cells");
            m::derive_cab(
                &a.req::<String>(n, "arch")?,
                &a.req::<PathBuf>(n, "build")?,
                &a.req::<String>(n, "ninja")?,
                &a.req::<BTreeSet<String>>(n, "cells")?,
                root,
            )
        }
        "media_derive" | "media_load" | "media_current" => {
            let n = bind!(a, "arch", "repo");
            let arch = a.req::<String>(n, "arch")?;
            let r = a.repo(n, root)?;
            match a.op {
                "media_derive" => m::derive(&arch, &r),
                "media_load" => m::load(&arch, &r),
                _ => m::current(&arch, &r),
            }
        }
        "media_committed" => {
            let n = bind!(a, "m", "arch");
            m::committed(
                &a.req::<Value>(n, "m")?,
                &a.opt::<String>(n, "arch", String::new())?,
            )
        }
        "media_render" => {
            let n = bind!(a, "m");
            val(m::render(&a.req::<Value>(n, "m")?)?)
        }
        "media_resolve" => {
            let n = bind!(a, "src", "build", "repo", "cells");
            let cells = a.opt::<Option<PathBuf>>(n, "cells", None)?;
            val(m::resolve(
                &a.req::<String>(n, "src")?,
                &a.req::<PathBuf>(n, "build")?,
                &a.repo(n, root)?,
                cells.as_deref(),
            )?)
        }
        "media_write_list" => {
            let n = bind!(a, "entries", "where");
            val(m::write_list(
                &a.req::<Vec<Value>>(n, "entries")?,
                &a.req::<BTreeMap<String, PathBuf>>(n, "where")?,
            )?)
        }
        "media_source_hash" => {
            let n = bind!(a, "repo");
            val(m::source_hash(&a.repo(n, root)?)?)
        }
        "media_cells_source" => {
            let n = bind!(a, "cells", "arch");
            val(m::cells_source(
                &a.req::<PathBuf>(n, "cells")?,
                &a.req::<String>(n, "arch")?,
            )?)
        }
        "media__run" => {
            let n = bind!(a, "cmd");
            m::run(&a.req::<String>(n, "cmd")?)?;
            Ok(Value::Null)
        }
        "media__check_tools" => {
            let n = bind!(a, "cmd");
            m::check_tools(&a.req::<String>(n, "cmd")?)?;
            Ok(Value::Null)
        }
        "media_recab" => {
            let n = bind!(a, "m", "out", "build", "repo", "cells");
            val(m::recab(
                &a.req::<Value>(n, "m")?,
                &a.req::<PathBuf>(n, "out")?,
                &a.req::<PathBuf>(n, "build")?,
                &a.repo(n, root)?,
                &a.req::<PathBuf>(n, "cells")?,
            )?)
        }
        "media_assemble" => {
            let n = bind!(a, "arch", "cd", "out", "cells", "repo");
            let cells = a.opt::<Option<PathBuf>>(n, "cells", None)?;
            val(m::assemble(
                &a.req::<String>(n, "arch")?,
                &a.req::<String>(n, "cd")?,
                &a.req::<PathBuf>(n, "out")?,
                cells.as_deref(),
                &a.repo(n, root)?,
            )?)
        }
        "cell_constants" => {
            a.bind(&[])?;
            Ok(
                json!({"ARCHES":crate::ARCHES,"CANON":"/tmp/vibeos","CELL_ENV":["RUSTFLAGS","RUSTUP_TOOLCHAIN","VIBEOS_PIC_HAL"],"REPO":root,"STMT":r"\nbuild ([^\n]*?): CUSTOM_COMMAND[^\n]*\n  COMMAND = ([^\n]*)\n  DESC = Cargo-linking ([^\n]*)"}),
            )
        }
        "media_constants" => {
            a.bind(&[])?;
            Ok(
                json!({"ARCHES":crate::ARCHES,"CDS":m::CDS,"CANON":"/tmp/vibeos","CONFIG":"Debug","CAB":m::CAB,"CAB_INF":m::CAB_INF,"PACKAGES":m::PACKAGES,"CELLS_STAMP":".cells.json","COMMITTED_ROSTESTS":["kmtests","testdata"],"_ROSTEST_SRC":r"(?:^|/)modules/rostests/([^/]+)/","REPO":root}),
            )
        }
        _ => Err(Error::new(format!("unknown manifest operation {}", a.op))),
    }
}
