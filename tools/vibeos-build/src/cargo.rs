//! Locked, offline Cargo graph discovery. Compiler depfiles still own actual input closure.
use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
    process::Command,
};
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Package {
    pub name: String,
    pub manifest: String,
}
#[derive(Deserialize)]
struct Metadata {
    packages: Vec<MetaPackage>,
    resolve: Option<Resolve>,
    #[serde(default)]
    workspace_root: String,
}
#[derive(Deserialize)]
struct MetaPackage {
    id: String,
    name: String,
    source: Option<String>,
    manifest_path: String,
}
#[derive(Deserialize)]
struct Resolve {
    nodes: Vec<Node>,
}
#[derive(Deserialize)]
struct Node {
    id: String,
    dependencies: Vec<String>,
}
pub fn local_closure(metadata: &Value, roots: &[String]) -> Result<Vec<Package>> {
    let m: Metadata = serde_json::from_value(metadata.clone())?;
    let resolve = m
        .resolve
        .ok_or("Cargo metadata has no resolved graph; --no-deps is insufficient")?;
    let mut packages = BTreeMap::new();
    for p in m.packages {
        if packages.insert(p.id.clone(), p).is_some() {
            return Err("duplicate Cargo package id".into());
        }
    }
    let mut nodes = BTreeMap::new();
    for n in resolve.nodes {
        if nodes.insert(n.id, n.dependencies).is_some() {
            return Err("duplicate Cargo resolve node".into());
        }
    }
    let mut pending = Vec::new();
    for root in roots {
        let found: Vec<_> = packages
            .values()
            .filter(|p| p.name == *root && p.source.is_none())
            .collect();
        if found.len() != 1 {
            return Err(Error::new(format!(
                "expected one local package named {root:?}, found {}",
                found.len()
            )));
        }
        pending.push(found[0].id.clone());
    }
    if pending.is_empty() {
        return Err("at least one root package is required".into());
    }
    let mut seen = BTreeSet::new();
    while let Some(key) = pending.pop() {
        if seen.contains(&key) {
            continue;
        }
        if !packages.contains_key(&key) || !nodes.contains_key(&key) {
            return Err(Error::new(format!("incomplete Cargo graph: {key}")));
        }
        seen.insert(key.clone());
        pending.extend(nodes[&key].iter().cloned());
    }
    let mut result: Vec<_> = seen
        .into_iter()
        .map(|key| &packages[&key])
        .filter(|p| p.source.is_none())
        .map(|p| Package {
            name: p.name.clone(),
            manifest: p.manifest_path.clone(),
        })
        .collect();
    result.sort_by(|a, b| (&a.name, &a.manifest).cmp(&(&b.name, &b.manifest)));
    Ok(result)
}
pub fn inspect(manifest: &Path, roots: &[String]) -> Result<Value> {
    let manifest = manifest.canonicalize()?;
    let mut cmd = Command::new("cargo");
    cmd.args([
        "metadata",
        "--offline",
        "--locked",
        "--all-features",
        "--format-version",
        "1",
        "--manifest-path",
    ])
    .arg(&manifest)
    .current_dir(manifest.parent().ok_or("manifest has no parent")?);
    let out = crate::process::capture(&mut cmd, 120)?;
    if out.code != 0 {
        return Err(Error::new(format!(
            "locked offline Cargo metadata failed:\n{}",
            out.stderr.trim()
        )));
    }
    let value: Value = serde_json::from_str(&out.stdout)?;
    let m: Metadata = serde_json::from_value(value.clone())?;
    let mut roots = roots.to_vec();
    roots.sort();
    roots.dedup();
    Ok(
        serde_json::json!({"roots":roots,"workspace":m.workspace_root,"scope":"all-features/all-targets; package closure only","local_packages":local_closure(&value,&roots)?}),
    )
}
