//! Read-only snapshot attribution and conservative archive-only storage models.
use crate::{Error, Result};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::CString,
    fs::{self, File, Metadata, OpenOptions},
    io::{Read, Write},
    os::{
        fd::{AsRawFd, FromRawFd},
        unix::{
            ffi::OsStrExt,
            fs::{MetadataExt, OpenOptionsExt},
        },
    },
    path::{Component, Path, PathBuf},
};
fn hash(data: &[u8]) -> String {
    format!("{:x}", Sha256::digest(data))
}
fn array<'a>(value: &'a Value, field: &str) -> Result<&'a Vec<Value>> {
    value
        .as_array()
        .ok_or_else(|| Error::new(format!("{field}: expected array")))
}
fn object<'a>(value: &'a Value, field: &str) -> Result<&'a serde_json::Map<String, Value>> {
    value
        .as_object()
        .ok_or_else(|| Error::new(format!("{field}: expected object")))
}
fn string<'a>(value: &'a Value, field: &str) -> Result<&'a str> {
    value
        .as_str()
        .ok_or_else(|| Error::new(format!("{field}: expected string")))
}
fn num(value: &Value) -> i64 {
    value.as_i64().unwrap_or(0)
}
fn ns(sec: i64, nano: i64) -> i128 {
    sec as i128 * 1_000_000_000 + nano as i128
}
fn identity(m: &Metadata) -> Vec<i128> {
    vec![
        m.dev() as i128,
        m.ino() as i128,
        m.mode() as i128,
        m.size() as i128,
        ns(m.mtime(), m.mtime_nsec()),
        ns(m.ctime(), m.ctime_nsec()),
        m.blocks() as i128,
        m.nlink() as i128,
    ]
}
fn openat(directory: &File, name: &std::ffi::OsStr, flags: i32) -> Result<File> {
    let name =
        CString::new(name.as_bytes()).map_err(|_| Error::new("NUL in attribution input path"))?;
    let fd = unsafe { libc::openat(directory.as_raw_fd(), name.as_ptr(), flags) };
    if fd < 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    Ok(unsafe { File::from_raw_fd(fd) })
}
/// No ancestor symlink may substitute a different inventory root during attribution.
fn open_root(path: &Path) -> Result<File> {
    let path = if path.is_absolute() {
        path.to_owned()
    } else {
        std::env::current_dir()?.join(path)
    };
    let flags = libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC;
    let mut file = OpenOptions::new()
        .read(true)
        .custom_flags(flags)
        .open("/")?;
    for part in path.components() {
        match part {
            Component::RootDir | Component::CurDir => {}
            Component::Normal(name) => file = openat(&file, name, flags)?,
            _ => return Err("unsafe attribution input parent".into()),
        }
    }
    Ok(file)
}
pub fn stable_read(path: &Path, expected: Option<&Value>) -> Result<Vec<u8>> {
    let directory = open_root(path.parent().ok_or("attribution path has no parent")?)?;
    let name = path.file_name().ok_or("attribution path has no filename")?;
    let mut file = openat(
        &directory,
        name,
        libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_CLOEXEC | libc::O_NONBLOCK,
    )?;
    let before = file.metadata()?;
    if !before.is_file() {
        return Err("not a regular file".into());
    }
    if let Some(expected) = expected.filter(|v| !v.is_null()) {
        let observed = [
            before.dev() as i128,
            before.ino() as i128,
            before.size() as i128,
            ns(before.mtime(), before.mtime_nsec()),
            ns(before.ctime(), before.ctime_nsec()),
            before.blocks() as i128 * 512,
            before.nlink() as i128,
        ];
        let keys = [
            "device",
            "inode",
            "logical_bytes",
            "mtime_ns",
            "ctime_ns",
            "allocated_bytes",
            "link_count",
        ];
        for (key, n) in keys.iter().zip(observed) {
            if expected[*key].to_string().parse::<i128>().ok() != Some(n) {
                return Err("changed since inventory".into());
            }
        }
    }
    if before.size() > 1024 * 1024 {
        return Err("unexpectedly large attribution input".into());
    }
    let mut payload = Vec::new();
    (&mut file)
        .take(1024 * 1024 + 1)
        .read_to_end(&mut payload)?;
    let after = file.metadata()?;
    // O_PATH observes the named entry without opening a possibly substituted FIFO.
    let at_path = openat(
        &directory,
        name,
        libc::O_PATH | libc::O_NOFOLLOW | libc::O_CLOEXEC,
    )?
    .metadata()?;
    if identity(&before) != identity(&after)
        || identity(&before) != identity(&at_path)
        || payload.len() as u64 != before.size()
    {
        return Err("changed while reading".into());
    }
    Ok(payload)
}
pub fn marker_fields(payload: &[u8]) -> Result<Vec<Vec<u8>>> {
    let mut fields = Vec::new();
    let mut offset = 0;
    while offset < payload.len() {
        if payload.len() - offset < 8 {
            return Err("truncated request field length".into());
        }
        let length = u64::from_le_bytes(payload[offset..offset + 8].try_into().unwrap());
        offset += 8;
        if length > (payload.len() - offset) as u64 {
            return Err("truncated request field".into());
        }
        let length = length as usize;
        fields.push(payload[offset..offset + length].to_vec());
        offset += length;
    }
    if fields.len() != 5 {
        return Err(Error::new(format!(
            "expected 5 request marker fields, found {}",
            fields.len()
        )));
    }
    Ok(fields)
}
fn attribution(
    alias: &Value,
    by_path: &BTreeMap<String, &Value>,
    manifests: &mut BTreeMap<String, Value>,
    repo: &Path,
) -> Result<Value> {
    let archive = PathBuf::from(string(&alias["path"], "snapshot path")?);
    let marker = archive
        .parent()
        .ok_or("snapshot has no parent")?
        .join(".body-storage.request-v1");
    let expected = by_path
        .get(&marker.to_string_lossy().into_owned())
        .ok_or("request marker not inventoried")?;
    let payload = stable_read(&marker, Some(expected))?;
    let fields = marker_fields(&payload)?;
    let utf =
        |bytes: &[u8]| String::from_utf8(bytes.to_vec()).map_err(|e| Error::new(e.to_string()));
    let arch = utf(&fields[0])?;
    let body = utf(&fields[1])?;
    let config = &fields[2];
    let request = utf(&fields[3])?;
    let filename = utf(&fields[4])?;
    if body.is_empty()
        || !body
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'-')
    {
        return Err("invalid body package component".into());
    }
    if !crate::ARCHES.contains(&arch.as_str()) {
        return Err("unknown marker architecture".into());
    }
    if archive.file_name().and_then(|s| s.to_str()) != Some(&filename) {
        return Err("marker archive name mismatch".into());
    }
    if !manifests.contains_key(&body) {
        let path = repo.join("sdk/lib/rust").join(&body).join("Cargo.toml");
        let bytes = stable_read(&path, None)?;
        let text = std::str::from_utf8(&bytes).map_err(|e| Error::new(e.to_string()))?;
        let raw: toml::Value = toml::from_str(text).map_err(|e| Error::new(e.to_string()))?;
        let package = raw
            .get("package")
            .and_then(|v| v.get("name"))
            .and_then(toml::Value::as_str)
            .ok_or("manifest package name missing")?;
        if package != body {
            return Err("marker package does not match manifest package".into());
        }
        let library = raw
            .get("lib")
            .and_then(|v| v.get("name"))
            .and_then(toml::Value::as_str)
            .map(str::to_owned)
            .unwrap_or_else(|| package.replace('-', "_"));
        let types = raw
            .get("lib")
            .and_then(|v| v.get("crate-type"))
            .map(|v| serde_json::to_value(v))
            .transpose()?
            .unwrap_or_else(|| json!(["rlib"]));
        manifests.insert(body.clone(),json!({"manifest_path":path,"manifest_sha256":hash(&bytes),"package":package,"library_name":library,"manifest_crate_types":types}));
    }
    let parts: Vec<_> = request.splitn(6, '|').collect();
    if parts.len() != 6 || parts[0] != body {
        return Err("request invocation does not identify marker package".into());
    }
    if parts[5].as_bytes() != config {
        return Err("request configuration does not match namespace configuration".into());
    }
    if parts[2..5].iter().any(|s| *s != "true" && *s != "false") {
        return Err("request boolean not understood".into());
    }
    let features: Value = serde_json::from_str(parts[1])?;
    if !features
        .as_array()
        .is_some_and(|a| a.iter().all(Value::is_string))
    {
        return Err("request features not string list".into());
    }
    let staticlib = parts[4] == "true"
        || array(&manifests[&body]["manifest_crate_types"], "crate types")?
            .iter()
            .any(|v| v == "staticlib");
    let library = string(&manifests[&body]["library_name"], "library name")?;
    if filename != format!("lib{library}.{}", if staticlib { "a" } else { "rlib" }) {
        return Err("archive name does not match Cargo manifest lib name".into());
    }
    let consumer = Path::new(string(&alias["relative"], "relative snapshot path")?)
        .components()
        .next()
        .ok_or("empty relative snapshot path")?
        .as_os_str()
        .to_string_lossy()
        .into_owned();
    Ok(
        json!({"path":archive,"body":body,"architecture":arch,"consumer_directory":consumer,"features":features,"default_features":parts[2]=="true","kernel":parts[3]=="true","force_staticlib":parts[4]=="true","staticlib":staticlib,"archive_name":filename,"configuration_sha256":hash(config),"request_sha256":hash(&fields[3]),"request_marker_sha256":hash(&payload)}),
    )
}
fn grouped<'a>(rows: &'a [Value], keys: &[&str]) -> Vec<(Vec<Value>, Vec<&'a Value>)> {
    let mut groups: Vec<(Vec<Value>, Vec<&Value>)> = Vec::new();
    for row in rows {
        let alias = &row["attributions"][0];
        let key: Vec<_> = keys.iter().map(|k| alias[*k].clone()).collect();
        if let Some((_, members)) = groups.iter_mut().find(|(k, _)| *k == key) {
            members.push(row)
        } else {
            groups.push((key, vec![row]))
        }
    }
    groups
}
fn sum(rows: &[&Value], key: &str) -> i64 {
    rows.iter().map(|v| num(&v[key])).sum()
}
fn consumers(rows: &[&Value]) -> Vec<String> {
    rows.iter()
        .flat_map(|v| v["attributions"].as_array().into_iter().flatten())
        .filter_map(|a| a["consumer_directory"].as_str().map(str::to_owned))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}
pub fn map_bodies_ordered(report: &Value, repo: &Path) -> Result<(Value, Vec<String>)> {
    let inventory = array(&report["inventory"], "inventory")?;
    let mut total = 0i64;
    let mut total_logical = 0i64;
    for record in inventory {
        for key in ["allocated_bytes", "logical_bytes"] {
            let value = record[key].as_i64().filter(|v| *v >= 0).ok_or_else(|| {
                Error::new(format!("{key} must be a nonnegative signed 64-bit integer"))
            })?;
            if key == "allocated_bytes" {
                total = total
                    .checked_add(value)
                    .ok_or("inventory allocation total exceeds signed 64-bit range")?;
            } else {
                total_logical = total_logical
                    .checked_add(value)
                    .ok_or("inventory logical total exceeds signed 64-bit range")?;
            }
        }
    }
    let mut by_path = BTreeMap::new();
    for record in inventory {
        for alias in array(&record["paths"], "paths")? {
            by_path.insert(string(&alias["path"], "inventory path")?.to_owned(), record);
        }
    }
    let mut manifests = BTreeMap::new();
    let mut bodies: BTreeMap<String, Vec<Value>> = BTreeMap::new();
    let mut unknown = Vec::new();
    let mut snapshot_inodes = 0;
    let mut body_order = Vec::new();
    for record in inventory {
        if !array(&record["roles"], "roles")?
            .iter()
            .any(|v| v == "consumer_snapshot")
            || record["kind"] != "regular"
        {
            continue;
        }
        snapshot_inodes += 1;
        let mut attributions = Vec::new();
        for alias in array(&record["paths"], "paths")? {
            if alias["role"] != "consumer_snapshot" {
                continue;
            }
            match attribution(alias,&by_path,&mut manifests,repo) {
                Ok(value)=>attributions.push(value),
                Err(error)=>unknown.push(json!({"device":record["device"],"inode":record["inode"],"allocated_bytes":record["allocated_bytes"],"path":alias["path"],"reason":error.message})),
            }
        }
        let identities: BTreeSet<_> = attributions
            .iter()
            .map(|a| {
                (
                    a["body"].to_string(),
                    a["architecture"].to_string(),
                    a["request_sha256"].to_string(),
                )
            })
            .collect();
        if identities.len() != 1 {
            if !identities.is_empty() {
                unknown.push(json!({"device":record["device"],"inode":record["inode"],"allocated_bytes":record["allocated_bytes"],"reason":"same inode has conflicting snapshot request attributions"}));
            }
            continue;
        }
        let body = string(&attributions[0]["body"], "body")?.to_owned();
        if !bodies.contains_key(&body) {
            body_order.push(body.clone());
        }
        bodies.entry(body).or_default().push(json!({"device":record["device"],"inode":record["inode"],"allocated_bytes":record["allocated_bytes"],"logical_bytes":record["logical_bytes"],"sha256":record["sha256"],"hash_status":record["hash_status"],"roles":record["roles"],"attributions":attributions}));
    }
    let mut output = serde_json::Map::new();
    for (body, rows) in bodies {
        let mut configs = Vec::new();
        for (key, members) in grouped(&rows, &["architecture", "configuration_sha256"]) {
            let mut hashes: BTreeMap<String, Vec<&Value>> = BTreeMap::new();
            let mut requests: Vec<(Value, Value)> = Vec::new();
            for row in &members {
                if row["hash_status"] == "verified" {
                    hashes
                        .entry(row["sha256"].to_string())
                        .or_default()
                        .push(row);
                }
                for a in array(&row["attributions"], "attributions")? {
                    let request = json!({"request_sha256":a["request_sha256"],"features":a["features"],"default_features":a["default_features"],"kernel":a["kernel"],"staticlib":a["staticlib"],"force_staticlib":a["force_staticlib"]});
                    if let Some((_, v)) =
                        requests.iter_mut().find(|(k, _)| *k == a["request_sha256"])
                    {
                        *v = request;
                    } else {
                        requests.push((a["request_sha256"].clone(), request));
                    }
                }
            }
            configs.push(json!({"architecture":key[0],"configuration_sha256":key[1],"snapshot_inodes":members.len(),"snapshot_allocated_bytes":sum(&members,"allocated_bytes"),"snapshot_logical_bytes":sum(&members,"logical_bytes"),"verified_content_groups":hashes.len(),"content_unique_allocated_bytes":hashes.values().map(|rs|rs.iter().map(|r|num(&r["allocated_bytes"])).max().unwrap_or(0)).sum::<i64>(),"consumers":consumers(&members),"requests":requests.into_iter().map(|(_,v)|v).collect::<Vec<_>>() }));
        }
        let mut value = manifests[&body].clone();
        let object = value.as_object_mut().unwrap();
        object.insert("snapshot_inodes".into(), json!(rows.len()));
        object.insert(
            "snapshot_allocated_bytes".into(),
            json!(rows.iter().map(|r| num(&r["allocated_bytes"])).sum::<i64>()),
        );
        object.insert("configurations".into(), json!(configs));
        object.insert("snapshots".into(), json!(rows));
        output.insert(body, value);
    }
    let attributed_inodes: i64 = output.values().map(|v| num(&v["snapshot_inodes"])).sum();
    let allocated: i64 = output
        .values()
        .map(|v| num(&v["snapshot_allocated_bytes"]))
        .sum();
    body_order.sort_by(|a, b| {
        num(&output[b]["snapshot_allocated_bytes"])
            .cmp(&num(&output[a]["snapshot_allocated_bytes"]))
    });
    Ok((
        json!({"schema_version":1,"inventory_started_at":report["started_at"],"inventory_finished_at":report["finished_at"],"snapshot_inodes":snapshot_inodes,"attributed_bodies":output.len(),"attributed_snapshot_inodes":attributed_inodes,"attributed_snapshot_allocated_bytes":allocated,"unknown":unknown,"bodies":output,"method":["Read .body-storage.request-v1 length-delimited fields; verify unchanged against saved inventory and before/after no-symlink reads.","Resolve marker package to current Cargo.toml package name and lib name; verify archive filename; keep manifest SHA256.","Count each observed snapshot inode once; hashes/configuration/request fields preserved separately.","Configuration/request SHA256 identifies stored descriptor bytes, not semantic action keys or eligibility.","Historical/stale configurations are retained; no currentness, cross-lane reuse, or reclaimability is inferred."]}),
        body_order,
    ))
}
pub fn map_bodies(report: &Value, repo: &Path) -> Result<Value> {
    map_bodies_ordered(report, repo).map(|(value, _)| value)
}

pub fn size_model(rows: &[Value]) -> Result<Value> {
    for row in rows {
        if row["allocated_bytes"].as_i64().is_none_or(|v| v < 0) {
            return Err("allocated_bytes must be a nonnegative signed 64-bit integer".into());
        }
        string(&row["hash_status"], "hash_status")?;
        array(&row["roles"], "roles")?;
        if row["hash_status"] == "verified" && row.get("sha256").is_none() {
            return Err("verified row has no sha256".into());
        }
    }
    let verified: Vec<_> = rows
        .iter()
        .filter(|r| r["hash_status"] == "verified" && r["roles"] == json!(["consumer_snapshot"]))
        .collect();
    let mut contents: BTreeMap<String, i64> = BTreeMap::new();
    for row in &verified {
        let bytes = num(&row["allocated_bytes"]);
        contents
            .entry(row["sha256"].to_string())
            .and_modify(|v| *v = (*v).max(bytes))
            .or_insert(bytes);
    }
    let total = verified.iter().try_fold(0i64, |total, r| {
        total
            .checked_add(num(&r["allocated_bytes"]))
            .ok_or("snapshot allocation total exceeds signed 64-bit range")
    })?;
    let store = contents.values().try_fold(0i64, |total, n| {
        total
            .checked_add(*n)
            .ok_or("store allocation total exceeds signed 64-bit range")
    })?;
    Ok(
        json!({"snapshot_inodes":rows.len(),"verified_exclusive_snapshot_inodes":verified.len(),"unknown_or_ambiguous_snapshot_inodes":rows.len()-verified.len(),"snapshot_allocated_bytes":total,"unique_content_groups":contents.len(),"new_store_blobs_allocated_bytes":store,"direct_archive_only_net_before_overheads_bytes":total-store,"copying_archive_only_net_before_overheads_bytes":-store,"additional_publication_peak_before_worker_source_overheads_bytes":store,"shared_intermediate_reclaim_credit_bytes":0,"primary_archive_reclaim_credit_bytes":0}),
    )
}
pub fn size_by_architecture(rows: &[Value]) -> Result<Value> {
    let mut groups: BTreeMap<String, Vec<Value>> = BTreeMap::new();
    for row in rows {
        let arch = string(
            &row["attributions"][0]["architecture"],
            "attribution architecture",
        )?
        .to_owned();
        groups.entry(arch).or_default().push(row.clone());
    }
    Ok(Value::Object(
        groups
            .into_iter()
            .map(|(k, v)| Ok((k, size_model(&v)?)))
            .collect::<Result<_>>()?,
    ))
}
#[derive(Clone, Debug)]
pub struct EconomicsOptions {
    pub body_order: Option<Vec<String>>,
    pub eligibility_top: i64,
    pub cargo: String,
    pub scratch: Option<PathBuf>,
}
impl Default for EconomicsOptions {
    fn default() -> Self {
        Self {
            body_order: None,
            eligibility_top: 0,
            cargo: "cargo".into(),
            scratch: None,
        }
    }
}
pub fn summarize(mapping: &Value, repo: &Path, options: &EconomicsOptions) -> Result<Value> {
    if options.eligibility_top < 0 {
        return Err("eligibility_top must be nonnegative".into());
    }
    if options.eligibility_top > 0 && options.scratch.is_none() {
        return Err("explicit scratch directory required for source discovery".into());
    }
    let scratch = if options.eligibility_top > 0 {
        options
            .scratch
            .as_ref()
            .map(|p| {
                fs::create_dir_all(p)?;
                p.canonicalize()
            })
            .transpose()?
    } else {
        None
    };
    let body_map = object(&mapping["bodies"], "bodies")?;
    let names = options.body_order.clone().unwrap_or_else(|| {
        let mut names: Vec<_> = body_map.keys().cloned().collect();
        names.sort_by(|a, b| {
            num(&body_map[b]["snapshot_allocated_bytes"])
                .cmp(&num(&body_map[a]["snapshot_allocated_bytes"]))
        });
        names
    });
    if names.len() != body_map.len()
        || names.iter().collect::<BTreeSet<_>>().len() != names.len()
        || names.iter().any(|n| !body_map.contains_key(n))
    {
        return Err("body order does not match mapping".into());
    }
    let sorted: Vec<_> = names.iter().map(|name| (name, &body_map[name])).collect();
    let top: Vec<_> = sorted
        .iter()
        .take(options.eligibility_top as usize)
        .map(|(name, _)| (*name).clone())
        .collect();
    let mut discovery = serde_json::Map::new();
    let mut bodies = serde_json::Map::new();
    let mut all_rows = Vec::new();
    let mut eligible_rows = Vec::new();
    for (name, body) in sorted {
        let rows = array(&body["snapshots"], "snapshots")?;
        all_rows.extend_from_slice(rows);
        let mut variants = Vec::new();
        for (key, members) in grouped(
            rows,
            &["architecture", "configuration_sha256", "request_sha256"],
        ) {
            let alias = &members[0]["attributions"][0];
            let source_value = json!({"body":name,"features":alias["features"],"default_features":alias["default_features"],"force_staticlib":alias["force_staticlib"]});
            let source_id = crate::json::dumps(&source_value, true);
            if top.contains(name) && !discovery.contains_key(&source_id) {
                let options = crate::artifact_inputs::DiscoverOptions {
                    features: serde_json::from_value(alias["features"].clone())?,
                    default_features: alias["default_features"]
                        .as_bool()
                        .ok_or("default_features must be bool")?,
                    force_staticlib: alias["force_staticlib"]
                        .as_bool()
                        .ok_or("force_staticlib must be bool")?,
                    cargo: options.cargo.clone(),
                    scratch: scratch.clone(),
                    ..Default::default()
                };
                let result = crate::artifact_inputs::discover(repo, name, json!({}), &options);
                let value: serde_json::Map<_, _> = [
                    "eligible",
                    "reasons",
                    "source_projection_key",
                    "action_key",
                    "missing_context",
                    "files",
                    "bytes",
                    "package_count",
                    "provenance",
                ]
                .into_iter()
                .map(|k| (k.to_owned(), result[k].clone()))
                .collect();
                discovery.insert(source_id.clone(), Value::Object(value));
            }
            let projection = discovery.get(&source_id);
            if projection.is_some_and(|p| p["eligible"] == true) {
                eligible_rows.extend(members.iter().map(|v| (*v).clone()));
            }
            let member_values: Vec<_> = members.iter().map(|v| (*v).clone()).collect();
            let mut variant = size_model(&member_values)?;
            let out = variant.as_object_mut().unwrap();
            for (k, v) in [
                ("architecture", key[0].clone()),
                ("configuration_sha256", key[1].clone()),
                ("request_sha256", key[2].clone()),
                ("features", alias["features"].clone()),
                ("default_features", alias["default_features"].clone()),
                ("kernel", alias["kernel"].clone()),
                ("force_staticlib", alias["force_staticlib"].clone()),
                ("staticlib", alias["staticlib"].clone()),
                ("consumers", json!(consumers(&members))),
                (
                    "source_projection_id",
                    if projection.is_some() {
                        json!(source_id)
                    } else {
                        Value::Null
                    },
                ),
                (
                    "source_projection_eligible",
                    projection
                        .map(|p| p["eligible"].clone())
                        .unwrap_or(Value::Null),
                ),
            ] {
                out.insert(k.into(), v);
            }
            variants.push(variant);
        }
        let mut value = serde_json::Map::new();
        for k in [
            "manifest_path",
            "manifest_sha256",
            "package",
            "library_name",
            "manifest_crate_types",
        ] {
            value.insert(k.into(), body[k].clone());
        }
        value.insert("archive_only_model".into(), size_model(rows)?);
        value.insert("requests".into(), json!(variants));
        bodies.insert(name.clone(), Value::Object(value));
    }
    // Native descriptor identity replaces the retired Python implementation identity.
    let discovery_sha = if options.eligibility_top > 0 {
        json!(hash(
            crate::json::canonical(
                &crate::artifact_api::implementation_identity()?
                    .get("sources")
                    .cloned()
                    .unwrap_or(Value::Null)
            )?
            .as_bytes()
        ))
    } else {
        Value::Null
    };
    Ok(
        json!({"schema_version":1,"inventory_started_at":mapping["inventory_started_at"],"inventory_finished_at":mapping["inventory_finished_at"],"attributed_bodies":mapping["attributed_bodies"],"unknown":mapping["unknown"],"all_bodies_archive_only_model":size_model(&all_rows)?,"all_bodies_by_architecture":size_by_architecture(&all_rows)?,"source_eligible_archive_only_model":size_model(&eligible_rows)?,"source_eligible_by_architecture":size_by_architecture(&eligible_rows)?,"eligibility_bodies":top,"source_discovery":discovery,"discovery_tool_sha256":discovery_sha,"discovery_cargo":options.cargo,"bodies":bodies,"method":["Body/feature/default/force-staticlib attribution from verified markers/manifests.","SHA256 actual inventory bytes, unique inodes; retain largest allocation per content.","Direct removes verified consumer snapshots and adds blobs; all other outputs stay.","Copying keeps snapshots and adds blobs; still owes source/scratch/ref/GC overhead.","Optional discover uses current source and actual recorded request features; missing context means action_key is null.","Source support does not prove execution eligibility, historical provenance, cross-lane reuse, or reproducibility.","Keep historical/configuration variants; no currentness or pruning assumptions."]}),
    )
}
pub fn analyze(inventory: &Value, repo: &Path, options: &EconomicsOptions) -> Result<Value> {
    let (mapping, body_order) = map_bodies_ordered(inventory, repo)?;
    let mut options = options.clone();
    options.body_order = Some(body_order);
    let mut report = summarize(&mapping, repo, &options)?;
    let coverage: serde_json::Map<_, _> = [
        "snapshot_inodes",
        "attributed_bodies",
        "attributed_snapshot_inodes",
        "attributed_snapshot_allocated_bytes",
        "unknown",
    ]
    .into_iter()
    .map(|k| (k.to_owned(), mapping[k].clone()))
    .collect();
    let complete = inventory["coverage"]["complete"] == true
        && inventory["coverage"]["hash_complete"] == true
        && array(&mapping["unknown"], "unknown")?.is_empty()
        && mapping["snapshot_inodes"] == mapping["attributed_snapshot_inodes"];
    let out = report.as_object_mut().unwrap();
    out.insert("inventory_coverage".into(), inventory["coverage"].clone());
    out.insert("attribution_coverage".into(), Value::Object(coverage));
    out.insert("complete".into(), json!(complete));
    out.insert("attribution_method".into(), mapping["method"].clone());
    Ok(report)
}
fn absolute(path: &Path) -> Result<PathBuf> {
    if path.is_absolute() {
        Ok(path.to_owned())
    } else {
        Ok(std::env::current_dir()?.join(path))
    }
}
fn resolved_output(path: &Path) -> Result<PathBuf> {
    let mut out = PathBuf::from("/");
    for component in absolute(path)?.components() {
        match component {
            Component::RootDir | Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            Component::Normal(part) => {
                out.push(part);
                if out.exists() {
                    out = out.canonicalize()?;
                } else if out.is_symlink() {
                    return Err("dangling output/scratch ancestor symlink".into());
                }
            }
            Component::Prefix(_) => return Err("unsupported output path prefix".into()),
        }
    }
    Ok(out)
}
/// Native CLI retains the read-only roots guard and exclusive report publication.
pub fn cli(args: &[String]) -> Result<u8> {
    if args.iter().any(|s| s == "--help" || s == "-h") {
        println!("artifact-economics INVENTORY REPO OUTPUT [--eligibility-top N --scratch DIR] [--cargo CARGO]");
        return Ok(0);
    }
    let mut positions = Vec::new();
    let mut options = EconomicsOptions::default();
    let expanded: Vec<String> = args
        .iter()
        .flat_map(|arg| {
            if arg.starts_with("--") {
                if let Some((key, value)) = arg.split_once('=') {
                    return vec![key.to_owned(), value.to_owned()];
                }
            }
            vec![arg.clone()]
        })
        .collect();
    let mut args = expanded.iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--eligibility-top" => {
                options.eligibility_top = args
                    .next()
                    .ok_or("--eligibility-top requires N")?
                    .parse()
                    .map_err(|_| Error::new("invalid eligibility count"))?
            }
            "--cargo" => options.cargo = args.next().ok_or("--cargo requires executable")?.clone(),
            "--scratch" => {
                options.scratch = Some(PathBuf::from(args.next().ok_or("--scratch requires DIR")?))
            }
            s if s.starts_with('-') => {
                return Err(Error::new(format!("unknown economics option {s}")))
            }
            _ => positions.push(arg.clone()),
        }
    }
    if positions.len() != 3 {
        return Err("artifact-economics requires INVENTORY REPO OUTPUT".into());
    }
    if options.eligibility_top > 0 && options.scratch.is_none() {
        return Err("--eligibility-top requires --scratch".into());
    }
    let inventory_path = PathBuf::from(&positions[0]);
    let repo = PathBuf::from(&positions[1]);
    let output = PathBuf::from(&positions[2]);
    let payload = fs::read(&inventory_path)?;
    let inventory: Value = serde_json::from_slice(&payload)?;
    let destination = resolved_output(&output)?;
    for source in array(&inventory["roots"], "roots")? {
        let root = Path::new(string(&source["path"], "inventory root")?).canonicalize()?;
        if destination.starts_with(&root) {
            return Err("output must be outside every surveyed root".into());
        }
        if let Some(scratch) = &options.scratch {
            if resolved_output(scratch)?.starts_with(&root) {
                return Err("scratch must be outside every surveyed root".into());
            }
        }
    }
    let mut report = analyze(&inventory, &repo, &options)?;
    report["inventory_file"] = json!(absolute(&inventory_path)?);
    report["inventory_sha256"] = json!(hash(&payload));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&output)?;
    file.write_all(serde_json::to_string_pretty(&report)?.as_bytes())?;
    file.write_all(b"\n")?;
    let variants: serde_json::Map<_, _> = object(&report["source_discovery"], "source discovery")?
        .iter()
        .map(|(k, v)| {
            (
                k.clone(),
                json!({"eligible":v["eligible"],"reasons":v["reasons"]}),
            )
        })
        .collect();
    println!(
        "{}",
        serde_json::to_string_pretty(
            &json!({"complete":report["complete"],"attributed_bodies":report["attributed_bodies"],"unknown_count":array(&report["unknown"],"unknown")?.len(),"all_bodies":report["all_bodies_archive_only_model"],"source_eligible":report["source_eligible_archive_only_model"],"source_variants":variants})
        )?
    );
    Ok(if report["complete"] == true { 0 } else { 2 })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fields(values: &[&[u8]]) -> Vec<u8> {
        let mut out = Vec::new();
        for v in values {
            out.extend_from_slice(&(v.len() as u64).to_le_bytes());
            out.extend_from_slice(v);
        }
        out
    }
    #[test]
    fn marker_lengths_are_bounded_and_count_is_exact() {
        for bad in [vec![0], fields(&[b"one"]), {
            let mut b = 999u64.to_le_bytes().to_vec();
            b.extend_from_slice(b"short");
            b
        }] {
            assert!(marker_fields(&bad).is_err());
        }
        let good = fields(&[b"amd64", b"body", b"config", b"request", b"archive"]);
        assert_eq!(marker_fields(&good).unwrap()[1], b"body");
    }
    #[test]
    fn changed_or_symlinked_attribution_input_is_refused() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("marker");
        fs::write(&path, b"payload").unwrap();
        let m = path.metadata().unwrap();
        let expected = json!({"device":m.dev(),"inode":m.ino(),"logical_bytes":m.size(),"mtime_ns":ns(m.mtime(),m.mtime_nsec()),"ctime_ns":ns(m.ctime(),m.ctime_nsec()),"allocated_bytes":m.blocks()*512,"link_count":m.nlink()});
        assert_eq!(stable_read(&path, Some(&expected)).unwrap(), b"payload");
        fs::write(&path, b"mutated").unwrap();
        assert!(stable_read(&path, Some(&expected))
            .unwrap_err()
            .message
            .contains("changed since inventory"));
        let alias = temp.path().join("alias");
        std::os::unix::fs::symlink(&path, &alias).unwrap();
        assert!(stable_read(&alias, None).is_err());
        let real = temp.path().join("real");
        fs::create_dir(&real).unwrap();
        fs::write(real.join("a"), b"a").unwrap();
        let link = temp.path().join("link");
        std::os::unix::fs::symlink(real, &link).unwrap();
        assert!(stable_read(&link.join("a"), None).is_err());
    }
    #[test]
    fn malformed_allocation_is_an_error_and_missing_parents_do_not_hide_root_membership() {
        assert!(size_model(&[json!({"allocated_bytes":"4096","hash_status":"verified","roles":["consumer_snapshot"],"sha256":"x"})]).is_err());
        let temp = tempfile::tempdir().unwrap();
        let surveyed = temp.path().join("survey");
        fs::create_dir(&surveyed).unwrap();
        assert_eq!(
            resolved_output(&temp.path().join("absent/../survey/out.json")).unwrap(),
            surveyed.join("out.json")
        );
    }
    #[test]
    fn archive_model_retains_largest_allocation_and_excludes_ambiguous_inodes() {
        let rows = vec![
            json!({"hash_status":"verified","roles":["consumer_snapshot"],"sha256":"same","allocated_bytes":8192}),
            json!({"hash_status":"verified","roles":["consumer_snapshot"],"sha256":"same","allocated_bytes":4096}),
            json!({"hash_status":"verified","roles":["consumer_snapshot","body_primary_archive"],"sha256":"same","allocated_bytes":65536}),
        ];
        let result = size_model(&rows).unwrap();
        assert_eq!(result["new_store_blobs_allocated_bytes"], 8192);
        assert_eq!(
            result["direct_archive_only_net_before_overheads_bytes"],
            4096
        );
        assert_eq!(result["unknown_or_ambiguous_snapshot_inodes"], 1);
        assert_eq!(result["shared_intermediate_reclaim_credit_bytes"], 0);
    }
}
