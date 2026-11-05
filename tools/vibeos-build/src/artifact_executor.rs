//! Isolated pinned body execution. Caller owns compile and external disk admission.
//! Trusted toolchain/runtime hosts must not mutate their read-only worker mounts.
use crate::{
    artifact_fs as af, artifact_inputs as inputs, artifact_worker as worker, Error, Result,
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::CString,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    os::unix::{
        ffi::OsStrExt,
        fs::{MetadataExt, PermissionsExt},
        process::{CommandExt, ExitStatusExt},
    },
    path::{Component, Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{
        atomic::{AtomicI32, Ordering},
        Mutex,
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
pub const PIN: &str = "1.92.0-x86_64-unknown-linux-gnu";
pub const CANON: &str = "/tmp/vibeos";
pub const WORKSPACE: &str = "sdk/lib/rust";
pub const PREPARED_SCHEMA: &str = "vibeos-prepared-body-inputs-v1";
pub const AMD64: &str = "x86_64-pc-windows-gnu";
pub const TARGETS: [(&str, &str); 3] = [
    ("amd64", AMD64),
    ("i386", "i686-pc-windows-gnu"),
    ("arm64", "aarch64-pc-windows-gnullvm"),
];
fn fail(message: impl Into<String>) -> Error {
    Error::new(message).with_kind("RuntimeError")
}
fn obj<'a>(v: &'a Value, name: &str) -> Result<&'a Map<String, Value>> {
    v.as_object()
        .ok_or_else(|| fail(format!("{name} must be an object")))
}
fn arr<'a>(v: &'a Value, name: &str) -> Result<&'a Vec<Value>> {
    v.as_array()
        .ok_or_else(|| fail(format!("{name} must be an array")))
}
fn text<'a>(v: &'a Value, name: &str) -> Result<&'a str> {
    v.as_str()
        .ok_or_else(|| fail(format!("{name} must be a string")))
}
fn string(v: &Value, name: &str) -> Result<String> {
    Ok(text(v, name)?.to_owned())
}
fn path(v: &Value, name: &str) -> Result<PathBuf> {
    Ok(PathBuf::from(text(v, name)?))
}
fn bool_opt(v: &Value, key: &str, default: bool) -> Result<bool> {
    match v.get(key) {
        None | Some(Value::Null) => Ok(default),
        Some(v) => v
            .as_bool()
            .ok_or_else(|| fail(format!("{key} must be boolean"))),
    }
}
fn strings(v: &Value, name: &str) -> Result<Vec<String>> {
    arr(v, name)?.iter().map(|v| string(v, name)).collect()
}
fn digest_bytes(data: &[u8]) -> String {
    format!("{:x}", Sha256::digest(data))
}
fn read_json(path: &Path) -> Result<Value> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}
fn write_json(path: &Path, value: &Value) -> Result<()> {
    fs::write(path, inputs::canonical(value)?)?;
    Ok(())
}
fn stamp(m: &fs::Metadata) -> (u64, u64, i64, i64, i64, i64) {
    (
        m.ino(),
        m.size(),
        m.mtime(),
        m.mtime_nsec(),
        m.ctime(),
        m.ctime_nsec(),
    )
}
fn name(path: &Path) -> Result<String> {
    path.to_str()
        .map(str::to_owned)
        .ok_or_else(|| fail(format!("non-UTF8 executor path: {}", path.display())))
}
pub fn implementation_repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}
pub fn flags(architecture: &str, kernel: bool) -> Result<&'static str> {
    match (architecture, kernel) {
        ("amd64" | "arm64", false) => Ok(""),
        ("amd64" | "arm64", true) => Ok("-C force-frame-pointers=yes"),
        ("i386", false) => Ok("-C target-feature=-sse,-sse2"),
        ("i386", true) => Ok("-C target-feature=-sse,-sse2 -C force-frame-pointers=yes"),
        _ => Err(fail("unknown executor architecture")),
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ExecutorOptions {
    pub features: Vec<String>,
    pub default_features: bool,
    pub kernel: bool,
    pub provisioned: Option<Value>,
    pub target: String,
    pub flags: Option<String>,
}
impl Default for ExecutorOptions {
    fn default() -> Self {
        Self {
            features: vec![],
            default_features: true,
            kernel: true,
            provisioned: None,
            target: AMD64.into(),
            flags: None,
        }
    }
}
pub fn file_digest(path: &Path) -> Result<Value> {
    let before = fs::metadata(path)?;
    let mut stream = File::open(path)?;
    let mut hash = Sha256::new();
    let mut buf = vec![0u8; 1024 * 1024];
    loop {
        let n = stream.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hash.update(&buf[..n]);
    }
    let after = fs::metadata(path)?;
    if stamp(&before) != stamp(&after) {
        return Err(fail(format!(
            "input changed while hashing: {}",
            path.display()
        )));
    }
    Ok(json!({"sha256":format!("{:x}",hash.finalize()),"bytes":after.size()}))
}
fn entries(root: &Path) -> Result<Vec<fs::DirEntry>> {
    let mut out = fs::read_dir(root)?.collect::<std::io::Result<Vec<_>>>()?;
    out.sort_by_key(|e| e.file_name());
    Ok(out)
}
pub fn allocated(root: &Path) -> Result<Value> {
    let mut seen = BTreeSet::new();
    let (mut logical, mut physical, mut files) = (0u64, 0u64, 0u64);
    fn walk(
        root: &Path,
        seen: &mut BTreeSet<(u64, u64)>,
        logical: &mut u64,
        physical: &mut u64,
        files: &mut u64,
    ) -> Result<()> {
        for entry in match entries(root) {
            Ok(v) => v,
            Err(_e) if !root.exists() => return Ok(()),
            Err(e) => return Err(e),
        } {
            let path = entry.path();
            let stat = match fs::symlink_metadata(&path) {
                Ok(m) => m,
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
                Err(e) => return Err(e.into()),
            };
            if stat.is_dir() {
                walk(&path, seen, logical, physical, files)?;
            } else if !(stat.file_type().is_symlink() && path.is_dir()) {
                *logical = logical
                    .checked_add(stat.size())
                    .ok_or("allocation size overflow")?;
                *files += 1;
                if seen.insert((stat.dev(), stat.ino())) {
                    *physical = physical
                        .checked_add(stat.blocks() * 512)
                        .ok_or("allocation size overflow")?;
                }
            }
        }
        Ok(())
    }
    walk(root, &mut seen, &mut logical, &mut physical, &mut files)?;
    Ok(
        json!({"logical_bytes":logical,"allocated_bytes":physical,"unique_inodes":seen.len(),"files":files}),
    )
}
pub fn value_digest(value: &Value) -> Result<String> {
    inputs::digest(value)
}
pub fn implementation_identity() -> Result<Value> {
    let policy = crate::artifact_api::implementation_identity()?;
    Ok(
        json!({"executor_source":{"sha256":digest_bytes(include_bytes!("artifact_executor.rs")),"bytes":include_bytes!("artifact_executor.rs").len()},"native_build_policy":{"schema":policy["schema"],"sources":policy["sources"]},"support_sources":{}}),
    )
}
pub fn tree_identity(root: &Path) -> Result<Value> {
    if root.is_symlink() || !root.is_dir() {
        return Err(fail(format!(
            "prepared input root is not a real directory: {}",
            root.display()
        )));
    }
    let mut files = Map::new();
    let mut dirs = Vec::new();
    fn walk(
        root: &Path,
        current: &Path,
        files: &mut Map<String, Value>,
        dirs: &mut Vec<String>,
    ) -> Result<()> {
        let relative = current
            .strip_prefix(root)
            .map_err(|e| fail(e.to_string()))?;
        dirs.push(if relative.as_os_str().is_empty() {
            ".".into()
        } else {
            name(relative)?
        });
        for entry in entries(current)? {
            let path = entry.path();
            let meta = fs::symlink_metadata(&path)?;
            if meta.is_symlink() || (!meta.is_file() && !meta.is_dir()) {
                return Err(fail(format!(
                    "prepared input is a symlink or non-regular entry: {}",
                    path.display()
                )));
            }
            if meta.is_dir() {
                walk(root, &path, files, dirs)?;
            } else {
                files.insert(name(path.strip_prefix(root).unwrap())?, file_digest(&path)?);
            }
        }
        Ok(())
    }
    walk(root, root, &mut files, &mut dirs)?;
    dirs.sort();
    let inventory = json!({"directories":dirs,"files":files});
    Ok(
        json!({"sha256":value_digest(&inventory)?,"files":inventory["files"].as_object().unwrap().len()}),
    )
}
pub fn provisioned_copy(tool: &Value) -> Result<Value> {
    let result: Value = serde_json::from_slice(&inputs::canonical(tool)?)?;
    if !result["rustc"]
        .as_str()
        .is_some_and(|v| v.starts_with("rustc 1.92.0 "))
    {
        return Err(fail("provisioned recipe requires pinned rustc 1.92.0"));
    }
    let inputs = obj(
        &result["mounted_inputs"],
        "provisioned mounted input identities",
    )?;
    let native = obj(&result["native_tools"], "provisioned native tools")?;
    let expected: BTreeSet<_> = [
        "/native-tools/bin/llvm-dlltool",
        "/native-tools/lib/libLLVM.so.19.1",
    ]
    .into_iter()
    .collect();
    if native.keys().map(String::as_str).collect::<BTreeSet<_>>() != expected {
        return Err(fail(
            "provisioned native-tool mapping differs from the audited recipe",
        ));
    }
    let mut required: BTreeSet<String> = expected
        .into_iter()
        .chain(["/toolchain/bin/cargo", "/toolchain/bin/rustc"])
        .map(str::to_owned)
        .collect();
    for v in arr(
        &result["runtime_libraries"],
        "provisioned runtime library mapping",
    )? {
        let p = Path::new(text(v, "runtime library path")?);
        if !p.is_absolute() || p.components().any(|c| c == Component::ParentDir) {
            return Err(fail("provisioned runtime library path is not canonical"));
        }
        required.insert(name(p)?);
    }
    for v in native.values() {
        if !Path::new(text(v, "native executable path")?).is_absolute() {
            return Err(fail("provisioned native executable path is not absolute"));
        }
    }
    let hex = Regex::new(r"^[0-9a-f]{64}$")?;
    for p in required {
        let entry = inputs.get(&p).unwrap_or(&Value::Null);
        if !entry["sha256"].as_str().is_some_and(|v| hex.is_match(v)) {
            return Err(fail(format!(
                "provisioned recipe is missing a byte digest: {p}"
            )));
        }
        if entry["bytes"].as_u64().is_none() {
            return Err(fail(format!(
                "provisioned recipe is missing a byte size: {p}"
            )));
        }
    }
    Ok(result)
}
fn command_output(command: &mut Command, timeout: u64) -> Result<crate::process::Output> {
    crate::process::capture(command, timeout)
}
fn checked_output(command: &mut Command, timeout: u64) -> Result<crate::process::Output> {
    let out = command_output(command, timeout)?;
    if out.code != 0 {
        return Err(
            Error::new(format!("subprocess failed ({}): {}", out.code, out.stderr))
                .with_kind("CalledProcessError"),
        );
    }
    Ok(out)
}
pub fn provision(toolchain: &Path) -> Result<Value> {
    let toolchain = toolchain.canonicalize()?;
    let llvm = implementation_repo()
        .join(".build-tools/llvm-mingw")
        .canonicalize()?;
    let native: BTreeMap<_, _> = [
        (
            "/native-tools/bin/llvm-dlltool",
            llvm.join("bin/llvm-dlltool").canonicalize()?,
        ),
        (
            "/native-tools/lib/libLLVM.so.19.1",
            llvm.join("lib/libLLVM.so.19.1").canonicalize()?,
        ),
    ]
    .into_iter()
    .collect();
    let native_sources: BTreeSet<_> = native.values().cloned().collect();
    let mut inputs = Map::new();
    let mut libraries = BTreeSet::new();
    for (target, source) in &native {
        if !source.is_file() {
            return Err(fail(format!(
                "required deterministic import-library input missing: {}",
                source.display()
            )));
        }
        inputs.insert((*target).into(), file_digest(source)?);
    }
    let executables: Vec<_> = [toolchain.join("bin/cargo"), toolchain.join("bin/rustc")]
        .into_iter()
        .chain(native.values().cloned())
        .collect();
    let lib_pattern = Regex::new(r"(?:=>\s*)?(/\S+)")?;
    for executable in executables {
        let out = checked_output(Command::new("ldd").arg(executable), 120)?;
        for line in out.stdout.lines() {
            if let Some(m) = lib_pattern.captures(line) {
                let path = std::path::absolute(&m[1])?;
                let real = path.canonicalize()?;
                if !real.starts_with(&toolchain) && !native_sources.contains(&real) {
                    libraries.insert(path);
                }
            }
        }
    }
    fn walk(root: &Path, current: &Path, inputs: &mut Map<String, Value>) -> Result<()> {
        for entry in entries(current)? {
            let path = entry.path();
            let meta = fs::symlink_metadata(&path)?;
            let key = format!("/toolchain/{}", name(path.strip_prefix(root).unwrap())?);
            if meta.is_symlink() {
                if !path.canonicalize()?.starts_with(root) {
                    return Err(fail(format!(
                        "toolchain symlink escapes mounted tree: {}",
                        path.display()
                    )));
                }
                inputs.insert(key, json!({"symlink":name(&fs::read_link(&path)?)?}));
            } else if meta.is_dir() {
                walk(root, &path, inputs)?;
            } else if meta.is_file() {
                inputs.insert(key, file_digest(&path)?);
            } else {
                return Err(fail(format!(
                    "non-regular toolchain input: {}",
                    path.display()
                )));
            }
        }
        Ok(())
    }
    walk(&toolchain, &toolchain, &mut inputs)?;
    for p in &libraries {
        inputs.insert(name(p)?, file_digest(p)?);
    }
    let version = checked_output(Command::new(toolchain.join("bin/rustc")).arg("-Vv"), 120)?.stdout;
    if !version.starts_with("rustc 1.92.0 ") {
        return Err(fail("executor requires pinned rustc 1.92.0"));
    }
    Ok(
        json!({"rustc":version,"native_tools":native,"mounted_inputs":inputs,"runtime_libraries":libraries}),
    )
}
fn copy_tree(source: &Path, dest: &Path) -> Result<()> {
    fs::create_dir_all(dest)?;
    fs::set_permissions(dest, fs::Permissions::from_mode(0o755))?;
    for entry in entries(source)? {
        let path = entry.path();
        let target = dest.join(entry.file_name());
        let meta = fs::symlink_metadata(&path)?;
        if meta.is_symlink() {
            return Err(fail("snapshot projection contains symlink"));
        } else if meta.is_dir() {
            copy_tree(&path, &target)?;
        } else if meta.is_file() {
            fs::copy(&path, &target)?;
            fs::set_permissions(target, fs::Permissions::from_mode(0o644))?;
        } else {
            return Err(fail("snapshot projection contains non-regular input"));
        }
    }
    Ok(())
}
pub fn normalize(snapshot: &Path, projection: &Value, destination: &Path) -> Result<Value> {
    let descriptor = &projection["descriptor"];
    let active = obj(&descriptor["packages"], "descriptor packages")?;
    let mut paths = Vec::new();
    fn manifests(root: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
        for entry in entries(root)? {
            let p = entry.path();
            let meta = fs::symlink_metadata(&p)?;
            if meta.is_symlink() {
                return Err(fail("snapshot manifest candidate contains symlink"));
            }
            if meta.is_dir() {
                manifests(&p, files)?;
            } else if entry.file_name() == "Cargo.toml" {
                files.push(p);
            }
        }
        Ok(())
    }
    manifests(snapshot, &mut paths)?;
    paths.sort();
    let mut candidates = BTreeMap::new();
    for p in paths {
        if p == snapshot.join(WORKSPACE).join("Cargo.toml") {
            continue;
        }
        let raw: toml::Value =
            toml::from_str(&fs::read_to_string(&p)?).map_err(|e| fail(e.to_string()))?;
        let key = name(p.parent().unwrap().strip_prefix(snapshot).unwrap())?;
        candidates.insert(key, serde_json::to_value(raw)?);
    }
    fs::create_dir(destination)?;
    for (key, raw) in &candidates {
        if !af::safe_relative(Path::new(key)) {
            return Err(fail("unsafe projected package root"));
        }
        let output = destination.join(key);
        if active.contains_key(key) {
            copy_tree(&snapshot.join(key), &output)?;
        } else {
            fs::create_dir_all(&output)?;
            fs::write(
                output.join("__artifact_inactive.rs"),
                "// inactive Cargo resolution candidate\n",
            )?;
        }
        let mut manifest = obj(raw, "package manifest")?.clone();
        manifest.remove("dev-dependencies");
        if !active.contains_key(key) {
            manifest.insert("lib".into(), json!({"path":"__artifact_inactive.rs"}));
        }
        let mut lines = Vec::new();
        for (table, value) in manifest {
            lines.push(format!("[{table}]"));
            for (k, v) in obj(&value, "manifest table")? {
                lines.push(format!(
                    "{} = {}",
                    inputs::toml_value(&json!(k))?,
                    inputs::toml_value(v)?
                ));
            }
        }
        fs::write(output.join("Cargo.toml"), format!("{}\n", lines.join("\n")))?;
    }
    if let Some(data) = descriptor.get("data_inputs") {
        for relative in obj(data, "data inputs")?.keys() {
            let p = Path::new(relative);
            if !af::safe_relative(p) {
                return Err(fail("unsafe projected data input"));
            }
            let dest = destination.join(p);
            fs::create_dir_all(dest.parent().unwrap())?;
            fs::copy(snapshot.join(p), dest)?;
        }
    }
    let selected = text(&descriptor["request"]["package"], "selected package")?;
    let relative = |key: &str| -> Result<String> {
        name(
            Path::new(key)
                .strip_prefix(WORKSPACE)
                .map_err(|_| fail("normalized package outside workspace"))?,
        )
    };
    let excluded: Vec<_> = candidates
        .keys()
        .filter(|k| k.as_str() != selected)
        .map(|k| relative(k))
        .collect::<Result<_>>()?;
    let mut root_lines = vec![
        "[workspace]".to_owned(),
        "resolver = \"2\"".into(),
        format!(
            "members = {}",
            inputs::toml_value(&json!([relative(selected)?]))?
        ),
        format!("exclude = {}", inputs::toml_value(&json!(excluded))?),
        format!(
            "[profile.{}]",
            text(&descriptor["request"]["profile"], "request profile")?
        ),
    ];
    for (k, v) in obj(&descriptor["profiles"], "profiles")? {
        if v != &json!({}) {
            root_lines.push(format!(
                "{} = {}",
                inputs::toml_value(&json!(k))?,
                inputs::toml_value(v)?
            ));
        }
    }
    fs::create_dir_all(destination.join(WORKSPACE))?;
    fs::write(
        destination.join(WORKSPACE).join("Cargo.toml"),
        format!("{}\n", root_lines.join("\n")),
    )?;
    let mut lock_lines = vec!["version = 4".to_owned()];
    let mut packages: Vec<_> = candidates.values().collect();
    packages.sort_by_key(|v| v["package"]["name"].as_str().unwrap_or(""));
    for raw in packages {
        lock_lines.extend([
            "[[package]]".into(),
            format!("name = {}", inputs::toml_value(&raw["package"]["name"])?),
            format!(
                "version = {}",
                inputs::toml_value(&raw["package"]["version"])?
            ),
        ]);
        let mut deps = Vec::new();
        if let Some(dependencies) = raw.get("dependencies") {
            for (k, v) in obj(dependencies, "dependencies")? {
                deps.push(v.get("package").unwrap_or(&json!(k)).clone());
            }
        }
        deps.sort_by(|a, b| a.as_str().cmp(&b.as_str()));
        if !deps.is_empty() {
            lock_lines.push(format!(
                "dependencies = {}",
                inputs::toml_value(&json!(deps))?
            ));
        }
    }
    fs::write(
        destination.join(WORKSPACE).join("Cargo.lock"),
        format!("{}\n", lock_lines.join("\n")),
    )?;
    Ok(
        json!({"active_packages":active.keys().collect::<Vec<_>>(),"manifest_candidates":candidates.keys().collect::<Vec<_>>(),"files":allocated(destination)?,"workspace_policy":"selected member only; no dev edges; projected release profile; original relative package roots"}),
    )
}
fn mount(
    source: &str,
    target: &Path,
    flags: libc::c_ulong,
    filesystem: Option<&str>,
) -> Result<()> {
    let source = CString::new(source).map_err(|_| fail("NUL mount source"))?;
    let target =
        CString::new(target.as_os_str().as_bytes()).map_err(|_| fail("NUL mount target"))?;
    let kind = filesystem
        .map(CString::new)
        .transpose()
        .map_err(|_| fail("NUL filesystem kind"))?;
    let rc = unsafe {
        libc::mount(
            source.as_ptr(),
            target.as_ptr(),
            kind.as_ref().map_or(std::ptr::null(), |s| s.as_ptr()),
            flags,
            std::ptr::null(),
        )
    };
    if rc != 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    Ok(())
}
fn bind(source: &Path, target: &Path, readonly: bool) -> Result<()> {
    if source.is_dir() {
        fs::create_dir_all(target)?;
    } else {
        fs::create_dir_all(target.parent().ok_or("mount target has no parent")?)?;
        OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(false)
            .open(target)?;
    }
    mount(&name(source)?, target, libc::MS_BIND, None)?;
    if readonly {
        mount(
            &name(source)?,
            target,
            libc::MS_BIND | libc::MS_REMOUNT | libc::MS_RDONLY | libc::MS_NOSUID | libc::MS_NODEV,
            None,
        )?;
    }
    Ok(())
}
pub fn inside(config: &Value, probe: bool) -> Result<()> {
    let root = path(&config["root"], "worker root")?;
    mount(
        "none",
        Path::new("/"),
        libc::MS_REC | libc::MS_PRIVATE,
        None,
    )?;
    bind(
        &path(&config["projection"], "projection")?,
        &root.join("tmp/vibeos"),
        true,
    )?;
    bind(
        &path(&config["toolchain"], "toolchain")?,
        &root.join("toolchain"),
        true,
    )?;
    if let Some(native) = config.get("native_tools") {
        for (target, source) in obj(native, "native tools")? {
            bind(
                &path(source, "native source")?,
                &root.join(target.trim_start_matches('/')),
                true,
            )?;
        }
    }
    for library in arr(&config["runtime_libraries"], "runtime libraries")? {
        let p = path(library, "runtime library")?;
        bind(&p, &root.join(name(&p)?.trim_start_matches('/')), true)?;
    }
    for key in ["target", "cargo-home", "home", "scratch"] {
        bind(&path(&config[key], key)?, &root.join(key), false)?;
    }
    fs::create_dir_all(root.join("dev"))?;
    bind(Path::new("/dev/null"), &root.join("dev/null"), false)?;
    fs::create_dir_all(root.join("proc"))?;
    mount(
        "proc",
        &root.join("proc"),
        libc::MS_NOSUID | libc::MS_NODEV | libc::MS_NOEXEC,
        Some("proc"),
    )?;
    std::env::set_current_dir(&root)?;
    let root = CString::new(root.as_os_str().as_bytes()).map_err(|_| fail("NUL root"))?;
    if unsafe { libc::chroot(root.as_ptr()) } != 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    std::env::set_current_dir(Path::new(CANON).join(WORKSPACE))?;
    if probe {
        let mut evidence = json!({"host_checkout_hidden":!Path::new(text(&config["repo"],"repo")?).exists(),"host_home_hidden":!Path::new("/home/curi/.cargo").exists(),"environment_keys":obj(&config["env"],"environment")?.keys().collect::<Vec<_>>()});
        if let Err(e) = fs::write(
            Path::new(CANON).join(WORKSPACE).join("Cargo.toml"),
            "must fail",
        ) {
            evidence["readonly_source_errno"] = json!(e.raw_os_error());
        }
        if let Err(e) = std::net::TcpStream::connect_timeout(
            &"1.1.1.1:443".parse().unwrap(),
            Duration::from_millis(200),
        ) {
            evidence["network_connect_errno"] = json!(e.raw_os_error());
        }
        evidence["network_devices"] = json!(fs::read_to_string("/proc/net/dev")?);
        if evidence["host_checkout_hidden"] != true
            || evidence["readonly_source_errno"] != 30
            || evidence.get("network_connect_errno").is_none()
        {
            return Err(fail("worker isolation negative control failed"));
        }
        println!("{}", crate::json::canonical(&evidence)?);
        std::io::stdout().flush()?;
        return Ok(());
    }
    let argv = strings(&config["argv"], "compiler argv")?;
    if argv.is_empty() {
        return Err(fail("compiler argv is empty"));
    }
    let mut command = Command::new("/toolchain/bin/cargo");
    command.arg0(&argv[0]).args(&argv[1..]).env_clear();
    for (k, v) in obj(&config["env"], "compiler environment")? {
        command.env(k, text(v, "environment value")?);
    }
    Err(command.exec().into())
}
fn unshare_launch(config_path: &Path, probe: bool) -> Result<Vec<String>> {
    Ok(vec![
        "unshare".into(),
        "--user".into(),
        "--map-root-user".into(),
        "--mount".into(),
        "--net".into(),
        "--pid".into(),
        "--fork".into(),
        "--kill-child".into(),
        name(&std::env::current_exe()?)?,
        "--root".into(),
        "/".into(),
        "artifact-executor".into(),
        if probe { "inside-probe" } else { "inside" }.into(),
        name(config_path)?,
    ])
}
/// Keep one native supervisor outside the user namespace. Credential transitions
/// in unshare must not disconnect a live namespace from its executor's lifetime.
pub fn launch(config_path: &Path, probe: bool) -> Result<Vec<String>> {
    Ok(vec![
        name(&std::env::current_exe()?)?,
        "--root".into(),
        "/".into(),
        "artifact-executor".into(),
        if probe {
            "namespace-probe"
        } else {
            "namespace"
        }
        .into(),
        name(config_path)?,
    ])
}
fn command(args: &[String]) -> Result<Command> {
    let (first, rest) = args.split_first().ok_or("empty executor command")?;
    let mut command = Command::new(first);
    command.args(rest);
    Ok(command)
}
pub fn validate_graph(output: &Path, descriptor: &Value, config: &Value) -> Result<Value> {
    let mut metadata = config.clone();
    let mut argv = vec![
        "/toolchain/bin/cargo".to_owned(),
        "metadata".into(),
        "--format-version".into(),
        "1".into(),
        "--locked".into(),
        "--offline".into(),
        "--manifest-path".into(),
        "/tmp/vibeos/sdk/lib/rust/Cargo.toml".into(),
    ];
    let request = &descriptor["request"];
    if let Some(features) = request.get("features") {
        let features = strings(features, "request features")?;
        if !features.is_empty() {
            argv.extend(["--features".into(), features.join(",")]);
        }
    }
    if !bool_opt(request, "default_features", true)? {
        argv.push("--no-default-features".into());
    }
    metadata["argv"] = json!(argv);
    let metadata_path = output.join("metadata-worker.json");
    write_json(&metadata_path, &metadata)?;
    let result = command_output(&mut command(&launch(&metadata_path, false)?)?, 120)?;
    if result.code != 0 {
        return Err(fail(format!(
            "isolated graph resolution failed: {}",
            result.stderr
        )));
    }
    let metadata: Value = serde_json::from_str(&result.stdout)?;
    let mut identities = BTreeMap::new();
    for package in arr(&metadata["packages"], "metadata packages")? {
        let p = path(&package["manifest_path"], "manifest path")?;
        let relative = p
            .parent()
            .ok_or("manifest has no parent")?
            .strip_prefix(CANON)
            .map_err(|_| fail("isolated manifest outside canonical source"))?;
        identities.insert(string(&package["id"], "package id")?, name(relative)?);
    }
    let mut observed = Map::new();
    for node in arr(&metadata["resolve"]["nodes"], "metadata nodes")? {
        let id = text(&node["id"], "node id")?;
        let key = identities.get(id).ok_or("metadata node identity missing")?;
        let mut features = strings(&node["features"], "node features")?;
        features.sort();
        let mut edges = Vec::new();
        for e in arr(&node["deps"], "node deps")? {
            edges.push(json!({"name":e["name"],"package":identities.get(text(&e["pkg"],"edge package")?).ok_or("edge package missing")?}));
        }
        edges.sort_by(|a, b| a["name"].as_str().cmp(&b["name"].as_str()));
        observed.insert(key.clone(), json!({"features":features,"edges":edges}));
    }
    let mut expected = Map::new();
    for (key, package) in obj(&descriptor["packages"], "descriptor packages")? {
        expected.insert(
            key.clone(),
            json!({"features":package["features"],"edges":package["edges"]}),
        );
    }
    if observed != expected {
        return Err(fail(format!(
            "normalized execution graph differs from discovered graph: {}",
            crate::json::dumps(&json!({"expected":expected,"observed":observed}), true)
        )));
    }
    Ok(
        json!({"status":"identical","package_count":observed.len(),"graph":observed,"stderr":result.stderr}),
    )
}
pub fn prepared_identity(config: &Value, output: &Path, context: &Value) -> Result<Value> {
    Ok(
        json!({"schema":PREPARED_SCHEMA,"worker_config_sha256":value_digest(config)?,"context_sha256":value_digest(context)?,"tool_recipe_sha256":value_digest(&context["toolchain"])?,"snapshot":tree_identity(&output.join("snapshot"))?,"projection":tree_identity(&output.join("projection"))?,"root_scaffold":tree_identity(&output.join("root"))?}),
    )
}
fn verify_mount_leaves(config: &Value, tool: &Value) -> Result<()> {
    let mut paths: BTreeMap<String, PathBuf> = [
        (
            "/toolchain/bin/cargo".into(),
            path(&config["toolchain"], "toolchain")?.join("bin/cargo"),
        ),
        (
            "/toolchain/bin/rustc".into(),
            path(&config["toolchain"], "toolchain")?.join("bin/rustc"),
        ),
    ]
    .into_iter()
    .collect();
    for (target, source) in obj(&tool["native_tools"], "native tools")? {
        paths.insert(target.clone(), path(source, "native source")?);
    }
    for library in arr(&tool["runtime_libraries"], "runtime libraries")? {
        paths.insert(
            string(library, "runtime library")?,
            path(library, "runtime library")?,
        );
    }
    for (target, host) in paths {
        if file_digest(&host)? != tool["mounted_inputs"][&target] {
            return Err(fail(format!(
                "provisioned executable/runtime changed: {target}"
            )));
        }
    }
    Ok(())
}
fn request_options(config: &Value) -> Result<ExecutorOptions> {
    let options = config
        .get("request_options")
        .cloned()
        .unwrap_or_else(|| json!({}));
    let recipe = config.get("recipe").cloned().unwrap_or_else(|| json!({}));
    Ok(ExecutorOptions {
        features: options
            .get("features")
            .map(|v| strings(v, "features"))
            .transpose()?
            .unwrap_or_default(),
        default_features: bool_opt(&options, "default_features", true)?,
        kernel: bool_opt(config, "kernel", true)?,
        target: recipe
            .get("target")
            .map(|v| string(v, "recipe target"))
            .transpose()?
            .unwrap_or_else(|| AMD64.into()),
        flags: recipe
            .get("flags")
            .filter(|v| !v.is_null())
            .map(|v| string(v, "recipe flags"))
            .transpose()?,
        ..Default::default()
    })
}
fn discover_options(
    toolchain: &Path,
    options: &ExecutorOptions,
) -> Result<inputs::DiscoverOptions> {
    Ok(inputs::DiscoverOptions {
        features: options.features.clone(),
        default_features: options.default_features,
        cargo: name(&toolchain.join("bin/cargo"))?,
        ..Default::default()
    })
}
pub fn validate_prepared(output: &Path, report: &Value) -> Result<Value> {
    let context = &report["context"];
    for (key, value) in obj(&implementation_identity()?, "implementation identity")? {
        if context["execution_policy"].get(key) != Some(value) {
            return Err(fail(
                "executor changed since preparation; prepare a fresh worker",
            ));
        }
    }
    let config = read_json(&output.join("worker.json"))?;
    for key in [
        "root",
        "projection",
        "target",
        "cargo-home",
        "home",
        "scratch",
    ] {
        let p = path(&config[key], key)?;
        let expected = output.join(key);
        if p != expected || p.is_symlink() || af::resolve(&p)? != std::path::absolute(&expected)? {
            return Err(fail(format!("prepared worker path was redirected: {key}")));
        }
    }
    let tool = provisioned_copy(&context["toolchain"])?;
    if config["native_tools"] != tool["native_tools"]
        || config["runtime_libraries"] != tool["runtime_libraries"]
    {
        return Err(fail(
            "prepared mounted tool recipe differs from action context",
        ));
    }
    let toolchain = path(&config["toolchain"], "toolchain")?;
    let options = request_options(&config)?;
    let expected = request_context(&toolchain, &tool, text(&report["body"], "body")?, &options)?;
    if &expected != context {
        return Err(fail(
            "prepared action context differs from the current exact recipe",
        ));
    }
    if report["prepared_inputs"]["schema"] != PREPARED_SCHEMA {
        return Err(fail(
            "prepared worker lacks an audited integrity record; prepare again",
        ));
    }
    if prepared_identity(&config, output, context)? != report["prepared_inputs"] {
        return Err(fail(
            "prepared config, snapshot or normalized projection changed",
        ));
    }
    let mut private = Map::new();
    for key in ["cargo-home", "home", "scratch"] {
        private.insert(key.into(), tree_identity(&output.join(key))?);
    }
    if json!(private) != report["initial_private_inputs"] {
        return Err(fail(
            "prepared private Cargo/home/scratch inputs changed before execution",
        ));
    }
    if config["argv"] != context["effective_config"]["argv"]
        || config["env"] != context["environment"]
    {
        return Err(fail(
            "prepared compiler command/environment differs from action context",
        ));
    }
    verify_mount_leaves(&config, &tool)?;
    let projection = inputs::discover(
        &output.join("snapshot"),
        text(&report["body"], "body")?,
        context.clone(),
        &discover_options(&toolchain, &options)?,
    );
    if projection["eligible"] != true
        || projection["source_projection_key"] != report["source_projection_key"]
        || projection["action_key"] != report["action_key"]
    {
        return Err(fail(
            "prepared source/action identity does not match captured bytes",
        ));
    }
    Ok(config)
}
pub fn request_context(
    toolchain: &Path,
    provisioned: &Value,
    body: &str,
    options: &ExecutorOptions,
) -> Result<Value> {
    if !toolchain.is_absolute() {
        return Err(fail("toolchain root must be absolute"));
    }
    if !Regex::new(r"^[A-Za-z0-9][A-Za-z0-9_-]*$")?.is_match(body) {
        return Err(fail("body must name one Cargo package"));
    }
    let features: Vec<_> = options
        .features
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let target = &options.target;
    let arch = TARGETS
        .iter()
        .find(|(_, t)| t == target)
        .map(|(a, _)| *a)
        .ok_or_else(|| fail(format!("target is not a builtin recipe triple: {target}")))?;
    let flags = options.flags.as_deref().unwrap_or(if options.kernel {
        "-C force-frame-pointers=yes"
    } else {
        ""
    });
    if ![self::flags(arch, false)?, self::flags(arch, true)?].contains(&flags) {
        return Err(fail(format!("unaudited RUSTFLAGS for {target}: {flags:?}")));
    }
    let tool = provisioned_copy(provisioned)?;
    let env = json!({"PATH":"/native-tools/bin:/toolchain/bin","HOME":"/home","CARGO_HOME":"/cargo-home","CARGO_TARGET_DIR":"/target","TMPDIR":"/scratch","RUSTC":"/toolchain/bin/rustc","RUSTFLAGS":format!("{}-C dlltool=/native-tools/bin/llvm-dlltool",if flags.is_empty(){String::new()}else{format!("{flags} ")}),"CARGO_INCREMENTAL":"0","CARGO_NET_OFFLINE":"true","LC_ALL":"C","TZ":"UTC"});
    let mut argv = vec![
        "/toolchain/bin/cargo".to_owned(),
        "build".into(),
        "--release".into(),
        "--target".into(),
        target.clone(),
        "-p".into(),
        body.into(),
        "--manifest-path".into(),
        "/tmp/vibeos/sdk/lib/rust/Cargo.toml".into(),
        "--lib".into(),
        "--locked".into(),
        "--offline".into(),
        "--jobs".into(),
        "1".into(),
        "-vv".into(),
    ];
    if !features.is_empty() {
        argv.extend(["--features".into(), features.join(",")]);
    }
    if !options.default_features {
        argv.push("--no-default-features".into());
    }
    let mut policy = obj(&implementation_identity()?, "implementation identity")?.clone();
    for (k, v) in obj(
        &json!({"canonical_source":"/tmp/vibeos","worker":"user/mount/net/pid namespace + minimal chroot","compiler_cache":"disabled","broker":"outer single compile permit; Cargo jobs=1","provisioning":"trusted read-only toolchain/runtime bind mounts; no host writes permitted during run"}),
        "policy",
    )? {
        policy.insert(k.clone(), v.clone());
    }
    Ok(
        json!({"target":{"builtin":target},"effective_flags":env["RUSTFLAGS"],"effective_config":{"argv":argv,"config_files":[],"crate_type":"manifest staticlib"},"toolchain":tool,"environment":env,"execution_policy":policy}),
    )
}
pub fn prepare(
    repo: &Path,
    body: &str,
    toolchain: &Path,
    output: &Path,
    options: &ExecutorOptions,
) -> Result<Value> {
    let started = Instant::now();
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::create_dir(output)?;
    let tool_started = Instant::now();
    let tool = match &options.provisioned {
        Some(v) => provisioned_copy(v)?,
        None => provisioned_copy(&provision(toolchain)?)?,
    };
    let tool_seconds = tool_started.elapsed().as_secs_f64();
    let mut options = options.clone();
    options.features = options
        .features
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let context = request_context(toolchain, &tool, body, &options)?;
    let discover_options = discover_options(toolchain, &options)?;
    let discovery = inputs::discover(repo, body, context.clone(), &discover_options);
    if discovery["eligible"] != true {
        return Err(fail(format!(
            "ineligible: {}",
            arr(&discovery["reasons"], "ineligibility reasons")?
                .iter()
                .map(|r| r.as_str().unwrap_or("invalid reason"))
                .collect::<Vec<_>>()
                .join("; ")
        )));
    }
    let capture_options = worker::CaptureOptions {
        expected_projection_key: Some(string(
            &discovery["source_projection_key"],
            "source projection key",
        )?),
        discovery: discover_options,
        ..Default::default()
    };
    let captured = worker::capture(
        repo,
        body,
        context.clone(),
        &output.join("snapshot"),
        &capture_options,
    )?;
    let normalization = normalize(
        &output.join("snapshot"),
        &discovery,
        &output.join("projection"),
    )?;
    for key in ["root", "target", "cargo-home", "home", "scratch"] {
        fs::create_dir(output.join(key))?;
    }
    let request_options =
        json!({"features":options.features,"default_features":options.default_features});
    let mut config = json!({"root":output.join("root"),"projection":output.join("projection"),"toolchain":toolchain,"runtime_libraries":tool["runtime_libraries"],"native_tools":tool["native_tools"],"repo":repo,"kernel":options.kernel,"argv":context["effective_config"]["argv"],"env":context["environment"],"request_options":request_options,"recipe":{"target":options.target,"flags":options.flags}});
    for key in ["target", "cargo-home", "home", "scratch"] {
        config[key] = json!(output.join(key));
    }
    write_json(&output.join("worker.json"), &config)?;
    let graph = validate_graph(output, &discovery["descriptor"], &config)?;
    for entry in entries(&output.join("target"))? {
        if entry.file_name() != ".rustc_info.json" || !entry.path().is_file() {
            return Err(fail(format!(
                "unexpected output from metadata discovery: {}",
                entry.path().display()
            )));
        }
        fs::remove_file(entry.path())?;
    }
    let mut private = Map::new();
    for key in ["cargo-home", "home", "scratch"] {
        private.insert(key.into(), tree_identity(&output.join(key))?);
    }
    let report = json!({"schema":"vibeos-isolated-body-measurement-v1","body":body,"source_projection_key":discovery["source_projection_key"],"action_key":discovery["action_key"],"context":context,"source_capture":captured,"normalization":normalization,"graph_validation":graph,"toolchain_hash_seconds":tool_seconds,"toolchain_hash_mode":if options.provisioned.is_some(){"supervisor-provided"}else{"local full hash"},"prepare_seconds":started.elapsed().as_secs_f64(),"output":output,"compile_status":"not run","prepared_inputs":prepared_identity(&config,output,&context)?,"initial_private_inputs":private});
    write_json(&output.join("report.json"), &report)?;
    Ok(report)
}

static INTERRUPTED: AtomicI32 = AtomicI32::new(0);
static SIGNAL_OWNER: Mutex<()> = Mutex::new(());
extern "C" fn interrupted(signal: libc::c_int) {
    INTERRUPTED.store(signal, Ordering::SeqCst);
}
struct SignalGuard {
    _owner: std::sync::MutexGuard<'static, ()>,
    old_int: libc::sigaction,
    old_term: libc::sigaction,
}
impl SignalGuard {
    fn install() -> Result<Self> {
        let owner = SIGNAL_OWNER
            .try_lock()
            .map_err(|_| fail("another executor run owns signal supervision in this process"))?;
        INTERRUPTED.store(0, Ordering::SeqCst);
        let mut action: libc::sigaction = unsafe { std::mem::zeroed() };
        action.sa_sigaction = interrupted as usize;
        unsafe { libc::sigemptyset(&mut action.sa_mask) };
        action.sa_flags = 0;
        let mut old_int = unsafe { std::mem::zeroed() };
        let mut old_term = unsafe { std::mem::zeroed() };
        if unsafe { libc::sigaction(libc::SIGINT, &action, &mut old_int) } != 0 {
            return Err(std::io::Error::last_os_error().into());
        }
        if unsafe { libc::sigaction(libc::SIGTERM, &action, &mut old_term) } != 0 {
            unsafe { libc::sigaction(libc::SIGINT, &old_int, std::ptr::null_mut()) };
            return Err(std::io::Error::last_os_error().into());
        }
        Ok(Self {
            _owner: owner,
            old_int,
            old_term,
        })
    }
}
impl Drop for SignalGuard {
    fn drop(&mut self) {
        unsafe {
            libc::sigaction(libc::SIGINT, &self.old_int, std::ptr::null_mut());
            libc::sigaction(libc::SIGTERM, &self.old_term, std::ptr::null_mut());
        }
        INTERRUPTED.store(0, Ordering::SeqCst);
    }
}
fn check_interrupted() -> Result<()> {
    match INTERRUPTED.load(Ordering::SeqCst) {
        0 => Ok(()),
        libc::SIGINT => {
            Err(Error::new("executor interrupted by SIGINT").with_kind("KeyboardInterrupt"))
        }
        _ => Err(Error::new("executor interrupted by SIGTERM").with_kind("InterruptedError")),
    }
}
/// Only children created in their own new session are ever selected for termination.
struct OwnedWorker {
    child: Child,
    group: libc::pid_t,
}
impl OwnedWorker {
    fn spawn(args: &[String], stdout: Stdio, stderr: Stdio) -> Result<Self> {
        Self::spawn_command(&mut command(args)?, stdout, stderr)
    }
    fn spawn_command(command: &mut Command, stdout: Stdio, stderr: Stdio) -> Result<Self> {
        Self::spawn_scope(command, stdout, stderr, true)
    }
    fn spawn_scope(
        command: &mut Command,
        stdout: Stdio,
        stderr: Stdio,
        session: bool,
    ) -> Result<Self> {
        command.stdout(stdout).stderr(stderr);
        let current_group = unsafe { libc::getpgrp() };
        let expected_parent = std::process::id() as libc::pid_t;
        unsafe {
            command.pre_exec(move || {
                // Child-side only: do not retain the supervisor's caught-signal handlers
                // in the fork/exec gap. An abrupt parent death must stop this child too.
                let mut action: libc::sigaction = std::mem::zeroed();
                action.sa_sigaction = libc::SIG_DFL;
                libc::sigemptyset(&mut action.sa_mask);
                if libc::sigaction(libc::SIGTERM, &action, std::ptr::null_mut()) != 0 {
                    return Err(std::io::Error::last_os_error());
                }
                if libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGTERM) != 0 {
                    return Err(std::io::Error::last_os_error());
                }
                if libc::getppid() != expected_parent {
                    libc::_exit(125);
                }
                if session && libc::setsid() < 0 {
                    Err(std::io::Error::last_os_error())
                } else {
                    Ok(())
                }
            });
        }
        let child = command.spawn().map_err(|error| {
            Error::classified(
                "LaunchError",
                format!(
                    "cannot launch {}: {error}",
                    command.get_program().to_string_lossy()
                ),
            )
        })?;
        let group = if session {
            child.id() as libc::pid_t
        } else {
            current_group
        };
        Ok(Self { child, group })
    }
    fn signal_group(&self, signal: libc::c_int) {
        unsafe {
            libc::kill(-self.group, signal);
        }
    }
    fn stop(&mut self) -> Result<()> {
        if self.child.try_wait()?.is_some() {
            return Ok(());
        }
        let pid = self.group;
        unsafe { libc::kill(-pid, libc::SIGTERM) };
        let started = Instant::now();
        loop {
            if self.child.try_wait()?.is_some() {
                return Ok(());
            }
            if started.elapsed() >= Duration::from_secs(5) {
                unsafe { libc::kill(-pid, libc::SIGKILL) };
                self.child.wait()?;
                return Ok(());
            }
            std::thread::sleep(Duration::from_millis(20));
        }
    }
    fn code(&mut self) -> Result<Option<i32>> {
        Ok(self
            .child
            .try_wait()?
            .map(|s| s.code().unwrap_or_else(|| -s.signal().unwrap_or(1))))
    }
}
impl Drop for OwnedWorker {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}
/// Supervise an admitted command chain in its own session. The existing broker remains
/// the admission owner; cancellation forwards to its executor and waits for pipe EOF.
pub fn supervised_output(
    command: &mut Command,
    timeout: Option<Duration>,
) -> Result<crate::process::Output> {
    let _signals = SignalGuard::install()?;
    let owner = std::process::id();
    let start = crate::artifact_lifetime::start_identity(owner)
        .ok_or("native supervisor start identity unavailable")?;
    command
        .env("VIBEOS_ARTIFACT_OWNER_PID", owner.to_string())
        .env(
            "VIBEOS_ARTIFACT_OWNER_START",
            crate::json::canonical(&start)?,
        );
    capture_command(command, timeout)
}
fn capture_isolated(args: &[String], timeout: Duration) -> Result<crate::process::Output> {
    capture_command(&mut command(args)?, Some(timeout))
}
fn capture_command(
    command: &mut Command,
    timeout: Option<Duration>,
) -> Result<crate::process::Output> {
    let mut worker = OwnedWorker::spawn_command(command, Stdio::piped(), Stdio::piped())?;
    let mut stdout = worker.child.stdout.take().ok_or("missing worker stdout")?;
    let mut stderr = worker.child.stderr.take().ok_or("missing worker stderr")?;
    let out = std::thread::spawn(move || {
        let mut b = Vec::new();
        stdout.read_to_end(&mut b).map(|_| b)
    });
    let err = std::thread::spawn(move || {
        let mut b = Vec::new();
        stderr.read_to_end(&mut b).map(|_| b)
    });
    let started = Instant::now();
    let result = (|| loop {
        check_interrupted()?;
        if let Some(code) = worker.code()? {
            return Ok(code);
        }
        if timeout.is_some_and(|timeout| started.elapsed() >= timeout) {
            return Err(fail("isolated worker timed out"));
        }
        std::thread::sleep(Duration::from_millis(20));
    })();
    // A broker may fail while an executor still owns its output pipes.
    if result.as_ref().map_or(true, |code| *code != 0) {
        worker.signal_group(libc::SIGTERM);
    }
    let stop = worker.stop();
    let stdout = out
        .join()
        .map_err(|_| fail("worker stdout reader failed"))??;
    let stderr = err
        .join()
        .map_err(|_| fail("worker stderr reader failed"))??;
    stop?;
    Ok(crate::process::Output {
        code: result?,
        stdout: String::from_utf8(stdout).map_err(|e| fail(e.to_string()))?,
        stderr: String::from_utf8_lossy(&stderr).into_owned(),
    })
}
/// Supervise one producer with inherited stdout/stderr, preserving live logs without buffering.
pub fn supervised_code(command: &mut Command) -> Result<i32> {
    supervised_code_scope(command, true)
}
/// Runner-only variant: the controller already created our private session.
pub fn supervised_group_code(command: &mut Command) -> Result<i32> {
    if unsafe { libc::getpgrp() } != std::process::id() as i32 {
        return Err(fail("runner does not own its group"));
    }
    supervised_code_scope(command, false)
}
fn supervised_code_scope(command: &mut Command, session: bool) -> Result<i32> {
    let _signals = SignalGuard::install()?;
    crate::artifact_lifetime::watch_owner(unsafe { libc::getppid() } as u32)?;
    let mut child = OwnedWorker::spawn_scope(command, Stdio::inherit(), Stdio::inherit(), session)?;
    let operation = (|| -> Result<i32> {
        loop {
            check_interrupted()?;
            if let Some(code) = child.code()? {
                return Ok(code);
            }
            std::thread::sleep(Duration::from_millis(20));
        }
    })();
    if !session {
        // Group termination includes the runner. Its result is already captured;
        // ignore self-directed cleanup signals until the guard restores handlers.
        let mut ignored: libc::sigaction = unsafe { std::mem::zeroed() };
        ignored.sa_sigaction = libc::SIG_IGN;
        unsafe { libc::sigemptyset(&mut ignored.sa_mask) };
        if unsafe { libc::sigaction(libc::SIGTERM, &ignored, std::ptr::null_mut()) } != 0
            || unsafe { libc::sigaction(libc::SIGINT, &ignored, std::ptr::null_mut()) } != 0
        {
            return Err(std::io::Error::last_os_error().into());
        }
    }
    if operation.as_ref().map_or(true, |code| *code != 0) {
        child.signal_group(libc::SIGTERM);
    }
    child.stop()?;
    operation
}
/// CLI exit code for the same streaming producer boundary.
pub fn supervised_status(command: &mut Command) -> Result<u8> {
    let code = supervised_code(command)?;
    Ok(if code < 0 {
        (128 - code).min(255) as u8
    } else {
        code.min(255) as u8
    })
}
pub fn namespace_worker(config_path: &Path, probe: bool) -> Result<u8> {
    supervised_status(&mut command(&unshare_launch(config_path, probe)?)?)
}
/// Host effects are injectable only through the Rust library, never worker configuration/RPC.
/// Shipping callers use RealHost; controls can exercise every supervision failure stage.
pub trait RunHost {
    fn validate(&mut self, output: &Path, report: &Value) -> Result<Value> {
        validate_prepared(output, report)
    }
    fn launch(&mut self, config: &Path, probe: bool) -> Result<Vec<String>> {
        launch(config, probe)
    }
    fn allocated(&mut self, root: &Path) -> Result<Value> {
        allocated(root)
    }
    fn provision(&mut self, toolchain: &Path) -> Result<Value> {
        provision(toolchain)
    }
    fn lane_projection(
        &mut self,
        repo: &Path,
        body: &str,
        context: Value,
        toolchain: &Path,
        options: &ExecutorOptions,
    ) -> Result<Value> {
        Ok(inputs::discover(
            repo,
            body,
            context,
            &discover_options(toolchain, options)?,
        ))
    }
    fn prepared_identity(
        &mut self,
        config: &Value,
        output: &Path,
        context: &Value,
    ) -> Result<Value> {
        prepared_identity(config, output, context)
    }
    fn implementation_identity(&mut self) -> Result<Value> {
        implementation_identity()
    }
}
pub struct RealHost;
impl RunHost for RealHost {}
pub fn run(output: &Path) -> Result<Value> {
    run_with_host(output, &mut RealHost)
}
pub fn run_with_host<H: RunHost>(output: &Path, host: &mut H) -> Result<Value> {
    if std::env::var("VIBEOS_HOST_JOB_HELD").as_deref() != Ok("1") {
        return Err(fail(
            "run must be admitted through host-jobs --kind compile",
        ));
    }
    let _signals = SignalGuard::install()?;
    match std::env::var("VIBEOS_ARTIFACT_OWNER_PID") {
        Ok(raw) => {
            let pid = raw
                .parse::<u32>()
                .map_err(|_| fail("invalid artifact supervisor PID"))?;
            let expected: Value = serde_json::from_str(
                &std::env::var("VIBEOS_ARTIFACT_OWNER_START")
                    .map_err(|_| fail("artifact supervisor start identity missing"))?,
            )?;
            crate::artifact_lifetime::watch_owner_checked(pid, Some(&expected))?;
        }
        Err(std::env::VarError::NotPresent) => {
            let parent = unsafe { libc::getppid() } as u32;
            if parent < 2 {
                return Err(fail("executor has no live admitting parent"));
            }
            crate::artifact_lifetime::watch_owner(parent)?
        }
        Err(_) => return Err(fail("artifact supervisor PID is not Unicode")),
    }
    run_engine(output, host)
}
fn run_engine<H: RunHost>(output: &Path, host: &mut H) -> Result<Value> {
    let output = std::path::absolute(output)?;
    let config_path = output.join("worker.json");
    let report_path = output.join("report.json");
    let mut report = read_json(&report_path)?;
    if report["compile_status"] != "not run"
        || fs::read_dir(output.join("target"))?
            .next()
            .transpose()?
            .is_some()
    {
        return Err(fail("cold experiment requires a fresh prepared worker and empty target; refusing to overwrite evidence"));
    }
    let mut marker = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output.join("run.started"))?;
    write!(
        marker,
        "{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| fail(e.to_string()))?
            .as_secs_f64()
    )?;
    drop(marker);
    report["compile_status"] = json!("running");
    write_json(&report_path, &report)?;
    let mut stage = "prepared-input-validation";
    let mut failure_details = None;
    let operation = (|| -> Result<()> {
        let started = Instant::now();
        let config = host.validate(&output, &report)?;
        check_interrupted()?;
        report["prepared_input_validation_seconds"] = json!(started.elapsed().as_secs_f64());
        report["prepared_input_validation"] = json!("identical");
        stage = "isolation-probe";
        let started = Instant::now();
        let probe = capture_isolated(&host.launch(&config_path, true)?, Duration::from_secs(120))?;
        if probe.code != 0 {
            failure_details =
                Some(json!({"returncode":probe.code,"stdout":probe.stdout,"stderr":probe.stderr}));
            return Err(
                Error::new(format!("isolation probe failed with exit {}", probe.code))
                    .with_kind("CalledProcessError"),
            );
        }
        report["isolation_probe"] = serde_json::from_str(&probe.stdout)?;
        report["isolation_probe_seconds"] = json!(started.elapsed().as_secs_f64());
        check_interrupted()?;
        stage = "compile-launch";
        let started = Instant::now();
        let log = File::create(output.join("cargo.log"))?;
        let mut worker = OwnedWorker::spawn(
            &host.launch(&config_path, false)?,
            Stdio::from(log.try_clone()?),
            Stdio::from(log),
        )?;
        let mut peak = 0u64;
        stage = "compile";
        let compilation = (|| -> Result<i32> {
            loop {
                check_interrupted()?;
                if let Some(code) = worker.code()? {
                    return Ok(code);
                }
                let sample = host.allocated(&output.join("target"))?;
                peak = peak.max(
                    sample["allocated_bytes"]
                        .as_u64()
                        .ok_or("sample allocated_bytes is not an unsigned integer")?,
                );
                std::thread::sleep(Duration::from_millis(200));
            }
        })();
        let stopped = worker.stop();
        report["compile_seconds"] = json!(started.elapsed().as_secs_f64());
        report["exit_code"] = json!(worker.code()?);
        stopped?;
        let code = compilation?;
        let archive = output
            .join("target")
            .join(text(
                &report["context"]["target"]["builtin"],
                "builtin target",
            )?)
            .join("release")
            .join(format!(
                "lib{}.a",
                text(&report["body"], "body")?.replace('-', "_")
            ));
        if code != 0 || !archive.is_file() {
            return Err(fail(format!(
                "isolated compilation failed; evidence {}",
                report_path.display()
            )));
        }
        stage = "archive-accounting";
        let mut header = [0u8; 8];
        File::open(&archive)?.read_exact(&mut header)?;
        if &header != b"!<arch>\n" {
            return Err(fail("compiler output is not a static archive"));
        }
        let mut record = file_digest(&archive)?;
        record["path"] = json!(archive);
        report["archive"] = record;
        let mut retained = Map::new();
        for key in ["snapshot", "projection", "target", "scratch"] {
            retained.insert(key.into(), host.allocated(&output.join(key))?);
        }
        peak = peak.max(
            retained["target"]["allocated_bytes"]
                .as_u64()
                .ok_or("retained allocated bytes invalid")?,
        );
        report["retained"] = json!(retained);
        report["sampled_target_peak_allocated_bytes"] = json!(peak);
        report["peak_method"] = json!(
            "target recursive allocated blocks sampled every 0.2s; not exact transient high-water"
        );
        report["rustc_invocations"] = json!(fs::read_to_string(output.join("cargo.log"))?
            .lines()
            .filter(|s| s.contains("Running `") && s.contains("/toolchain/bin/rustc"))
            .count());
        stage = "lane-input-revalidation";
        check_interrupted()?;
        let toolchain = path(&config["toolchain"], "toolchain")?;
        let current = host.lane_projection(
            &path(&config["repo"], "repo")?,
            text(&report["body"], "body")?,
            report["context"].clone(),
            &toolchain,
            &request_options(&config)?,
        )?;
        let identical = current["eligible"] == true
            && current["source_projection_key"] == report["source_projection_key"];
        report["lane_input_revalidation"] = json!(if identical {
            "identical"
        } else {
            "changed; result rejected for this lane"
        });
        if !identical {
            return Err(fail("lane inputs changed; result rejected"));
        }
        stage = "prepared-input-postcheck";
        let started = Instant::now();
        if host.prepared_identity(&config, &output, &report["context"])?
            != report["prepared_inputs"]
        {
            return Err(fail(
                "prepared sources changed during execution; result rejected",
            ));
        }
        if read_json(&config_path)? != config {
            return Err(fail(
                "prepared worker config changed during execution; result rejected",
            ));
        }
        report["prepared_input_postcheck_seconds"] = json!(started.elapsed().as_secs_f64());
        check_interrupted()?;
        stage = "toolchain-postcheck";
        let started = Instant::now();
        let tool_now = host.provision(&toolchain)?;
        report["toolchain_postcheck_seconds"] = json!(started.elapsed().as_secs_f64());
        let identical = tool_now == report["context"]["toolchain"];
        report["toolchain_postcheck"] = json!(if identical {
            "identical"
        } else {
            "changed; result rejected"
        });
        if !identical {
            return Err(fail("toolchain inputs changed; result rejected"));
        }
        stage = "implementation-postcheck";
        for (key, value) in obj(&host.implementation_identity()?, "implementation identity")? {
            if report["context"]["execution_policy"].get(key) != Some(value) {
                return Err(fail(
                    "executor implementation changed during execution; result rejected",
                ));
            }
        }
        check_interrupted()?;
        Ok(())
    })();
    match operation {
        Ok(()) => {
            report["compile_status"] = json!("success");
            write_json(&report_path, &report)?;
            Ok(report)
        }
        Err(error) => {
            let interrupted = matches!(
                error.kind.as_deref(),
                Some("KeyboardInterrupt" | "InterruptedError" | "SystemExit")
            );
            report["compile_status"] = json!(if interrupted { "interrupted" } else { "failed" });
            report["failure_stage"] = json!(stage);
            let mut failure = failure_details.unwrap_or_else(|| json!({}));
            failure["type"] = json!(error.kind.as_deref().unwrap_or("RuntimeError"));
            failure["message"] = json!(error.message);
            report["failure"] = failure;
            write_json(&report_path, &report)?;
            Err(error)
        }
    }
}
pub fn cli(args: &[String]) -> Result<u8> {
    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!("artifact-executor prepare|run|inside|inside-probe OUTPUT [--repo ROOT] [--body PACKAGE] [--toolchain ROOT]");
        return Ok(0);
    }
    let mut positions = Vec::new();
    let mut repo = PathBuf::from("/home/curi/Projects/os-playground/vibeos");
    let mut body = "beep-rust".to_owned();
    let mut toolchain =
        PathBuf::from(std::env::var_os("HOME").ok_or("HOME missing for default toolchain")?)
            .join(".rustup/toolchains")
            .join(PIN);
    let mut args = args.iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--repo" => repo = PathBuf::from(args.next().ok_or("--repo requires ROOT")?),
            "--body" => body = args.next().ok_or("--body requires PACKAGE")?.clone(),
            "--toolchain" => {
                toolchain = PathBuf::from(args.next().ok_or("--toolchain requires ROOT")?)
            }
            s if s.starts_with('-') => return Err(fail(format!("unknown executor option {s}"))),
            _ => positions.push(arg.clone()),
        }
    }
    if positions.len() != 2 {
        return Err(fail("executor requires COMMAND OUTPUT"));
    }
    let command = &positions[0];
    if command == "inside-fd" || command == "inside-fd-probe" {
        use std::os::fd::FromRawFd;
        let fd: i32 = positions[1]
            .parse()
            .map_err(|_| fail("invalid worker configuration fd"))?;
        if fd < 3 {
            return Err(fail("worker configuration fd must be >=3"));
        }
        let config = {
            let mut file = unsafe { std::fs::File::from_raw_fd(fd) };
            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes)?;
            serde_json::from_slice(&bytes)?
        };
        inside(&config, command == "inside-fd-probe")?;
        return Ok(0);
    }
    let output = std::path::absolute(&positions[1])?;
    match command.as_str() {
        "namespace" | "namespace-probe" => namespace_worker(&output, command == "namespace-probe"),
        "inside" | "inside-probe" => {
            inside(&read_json(&output)?, command == "inside-probe")?;
            Ok(0)
        }
        "prepare" => {
            if body != "beep-rust" && body != "kernel-rust" {
                return Err(fail("CLI --body must be beep-rust or kernel-rust; library supports any eligible package"));
            }
            let report = prepare(
                &repo.canonicalize()?,
                &body,
                &toolchain.canonicalize()?,
                &output,
                &Default::default(),
            )?;
            println!(
                "{}",
                crate::json::canonical(
                    &json!({"body":report["body"],"action_key":report["action_key"],"prepare_seconds":report["prepare_seconds"],"output":report["output"]})
                )?
            );
            Ok(0)
        }
        "run" => {
            let report = run(&output)?;
            println!(
                "{}",
                crate::json::canonical(
                    &json!({"body":report["body"],"action_key":report["action_key"],"compile_seconds":report["compile_seconds"],"archive":report["archive"]})
                )?
            );
            Ok(0)
        }
        _ => Err(fail("unknown executor command")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn write(root: &Path, relative: &str, content: &str) {
        let p = root.join(relative);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, content).unwrap();
    }
    fn fake_tool(root: &Path) -> Value {
        let targets = [
            "/toolchain/bin/cargo",
            "/toolchain/bin/rustc",
            "/native-tools/bin/llvm-dlltool",
            "/native-tools/lib/libLLVM.so.19.1",
        ];
        let mut mounted = Map::new();
        for target in targets {
            mounted.insert(target.into(), json!({"sha256":"a".repeat(64),"bytes":1}));
        }
        json!({"rustc":"rustc 1.92.0 fixture\n","mounted_inputs":mounted,"native_tools":{"/native-tools/bin/llvm-dlltool":root.join("llvm-dlltool"),"/native-tools/lib/libLLVM.so.19.1":root.join("libLLVM.so.19.1")},"runtime_libraries":[]})
    }
    fn toolchain() -> PathBuf {
        PathBuf::from(std::env::var_os("HOME").unwrap())
            .join(".rustup/toolchains")
            .join(PIN)
    }
    #[test]
    fn exact_request_context_keeps_arch_flags_features_and_explicit_environment() {
        let temp = tempfile::tempdir().unwrap();
        let tool = fake_tool(temp.path());
        let chain = toolchain();
        let mut options = ExecutorOptions {
            features: vec!["z".into(), "a".into(), "a".into()],
            default_features: false,
            ..Default::default()
        };
        let amd64 = request_context(&chain, &tool, "body-rust", &options).unwrap();
        let argv = strings(&amd64["effective_config"]["argv"], "argv").unwrap();
        assert!(argv.windows(2).any(|w| w == ["--features", "a,z"]));
        assert_eq!(argv.last().unwrap(), "--no-default-features");
        assert_eq!(amd64["environment"].as_object().unwrap().len(), 11);
        assert!(amd64["environment"].get("VIBEOS_HOST_JOB_HELD").is_none());
        for (arch, target) in TARGETS {
            for kernel in [false, true] {
                options.target = target.into();
                options.kernel = kernel;
                options.flags = Some(flags(arch, kernel).unwrap().into());
                let context = request_context(&chain, &tool, "body-rust", &options).unwrap();
                assert_eq!(context["target"]["builtin"], target);
                assert_eq!(
                    context["effective_flags"],
                    format!(
                        "{}-C dlltool=/native-tools/bin/llvm-dlltool",
                        if options.flags.as_deref() == Some("") {
                            String::new()
                        } else {
                            format!("{} ", options.flags.as_ref().unwrap())
                        }
                    )
                );
            }
        }
        options.target = "i686-kernel".into();
        assert!(request_context(&chain, &tool, "body", &options)
            .unwrap_err()
            .message
            .contains("builtin recipe triple"));
        options.target = "i686-pc-windows-gnu".into();
        options.flags = Some("".into());
        assert!(request_context(&chain, &tool, "body", &options)
            .unwrap_err()
            .message
            .contains("unaudited RUSTFLAGS"));
        assert!(request_context(&chain, &tool, "../body", &Default::default()).is_err());
        assert!(
            request_context(Path::new("relative"), &tool, "body", &Default::default()).is_err()
        );
        let mut changed = tool.clone();
        changed["mounted_inputs"]["/toolchain/bin/cargo"]["sha256"] = json!("bad");
        assert!(provisioned_copy(&changed).is_err());
        let detached = provisioned_copy(&tool).unwrap();
        assert_eq!(detached, tool);
    }
    #[test]
    fn normalized_graph_preserves_features_dev_exclusion_and_nested_packages() {
        let temp = tempfile::tempdir().unwrap();
        let snap = temp.path().join("snapshot");
        for key in ["body", "dep", "inactive"] {
            let mut manifest =
                format!("[package]\nname=\"{key}\"\nversion=\"0.1.0\"\nedition=\"2021\"\n");
            if key == "body" {
                manifest.push_str("[dependencies]\ndep={path=\"../dep\",default-features=false,features=[\"base\"]}\ninactive={path=\"../inactive\",optional=true}\n[dev-dependencies]\nmissing-dev={path=\"../missing-dev\"}\n[features]\ndefault=[\"enabled\"]\nenabled=[\"dep/extra\"]\n");
            } else {
                manifest.push_str("[features]\nbase=[]\nextra=[]\ndefault=[\"extra\"]\n");
            }
            write(&snap, &format!("{WORKSPACE}/{key}/Cargo.toml"), &manifest);
            if key != "inactive" {
                write(
                    &snap,
                    &format!("{WORKSPACE}/{key}/src/lib.rs"),
                    "pub fn value(){}\n",
                );
            }
        }
        write(&snap, &format!("{WORKSPACE}/Cargo.toml"), "[workspace]\n");
        let body = format!("{WORKSPACE}/body");
        let dep = format!("{WORKSPACE}/dep");
        let projection = json!({"descriptor":{"request":{"package":body,"profile":"release"},"profiles":{"opt-level":1,"panic":"abort","package":{"dep":{"codegen-units":17}}},"packages":{body.clone():{},dep.clone():{}}}});
        let out = temp.path().join("projection");
        let result = normalize(&snap, &projection, &out).unwrap();
        assert_eq!(result["active_packages"], json!([body, dep]));
        assert!(
            !fs::read_to_string(out.join(WORKSPACE).join("body/Cargo.toml"))
                .unwrap()
                .contains("dev-dependencies")
        );
        assert!(out
            .join(WORKSPACE)
            .join("inactive/__artifact_inactive.rs")
            .is_file());
        let metadata = Command::new(toolchain().join("bin/cargo"))
            .args([
                "metadata",
                "--format-version",
                "1",
                "--offline",
                "--locked",
                "--manifest-path",
            ])
            .arg(out.join(WORKSPACE).join("Cargo.toml"))
            .current_dir(&out)
            .env("RUSTC", toolchain().join("bin/rustc"))
            .env("CARGO_HOME", temp.path().join("cargo-home"))
            .output()
            .unwrap();
        assert!(
            metadata.status.success(),
            "{}",
            String::from_utf8_lossy(&metadata.stderr)
        );
        let metadata: Value = serde_json::from_slice(&metadata.stdout).unwrap();
        let names: BTreeMap<_, _> = metadata["packages"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| (p["id"].as_str().unwrap(), p["name"].as_str().unwrap()))
            .collect();
        let features: BTreeMap<_, _> = metadata["resolve"]["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|n| (names[n["id"].as_str().unwrap()], n["features"].clone()))
            .collect();
        assert_eq!(features["body"], json!(["default", "enabled"]));
        assert_eq!(features["dep"], json!(["base", "extra"]));
        assert_eq!(metadata["workspace_members"].as_array().unwrap().len(), 1);
        let nested = temp.path().join("nested");
        write(&nested,&format!("{WORKSPACE}/outer/Cargo.toml"),"[package]\nname=\"outer\"\nversion=\"0.1.0\"\n[dependencies]\ninner={path=\"inner\"}\n");
        write(
            &nested,
            &format!("{WORKSPACE}/outer/inner/Cargo.toml"),
            "[package]\nname=\"inner\"\nversion=\"0.1.0\"\n",
        );
        for key in ["outer", "outer/inner"] {
            write(
                &nested,
                &format!("{WORKSPACE}/{key}/src/lib.rs"),
                "pub fn f(){}\n",
            );
        }
        let projection = json!({"descriptor":{"request":{"package":format!("{WORKSPACE}/outer"),"profile":"release"},"profiles":{},"packages":{format!("{WORKSPACE}/outer"):{},format!("{WORKSPACE}/outer/inner"):{}}}});
        let out = temp.path().join("nested-out");
        normalize(&nested, &projection, &out).unwrap();
        assert!(out.join(WORKSPACE).join("outer/inner/src/lib.rs").is_file());
    }
    fn prepared_fixture() -> (tempfile::TempDir, PathBuf, Value) {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let workspace = repo.join(WORKSPACE);
        write(
            &workspace,
            "Cargo.toml",
            "[workspace]\nresolver=\"2\"\nmembers=[\"body\"]\n[profile.release]\npanic=\"abort\"\n",
        );
        write(
            &workspace,
            "Cargo.lock",
            "version=4\n[[package]]\nname=\"body\"\nversion=\"0.1.0\"\n",
        );
        write(&workspace,"body/Cargo.toml","[package]\nname=\"body\"\nversion=\"0.1.0\"\nedition=\"2021\"\n[lib]\ncrate-type=[\"staticlib\"]\n");
        write(&workspace, "body/src/lib.rs", "pub fn value()->u32{1}\n");
        let chain = toolchain();
        let mut tool = fake_tool(temp.path());
        for source in ["llvm-dlltool", "libLLVM.so.19.1"] {
            fs::write(temp.path().join(source), b"trusted fixture").unwrap();
        }
        for (key, source) in [
            ("/toolchain/bin/cargo", chain.join("bin/cargo")),
            ("/toolchain/bin/rustc", chain.join("bin/rustc")),
            (
                "/native-tools/bin/llvm-dlltool",
                temp.path().join("llvm-dlltool"),
            ),
            (
                "/native-tools/lib/libLLVM.so.19.1",
                temp.path().join("libLLVM.so.19.1"),
            ),
        ] {
            tool["mounted_inputs"][key] = file_digest(&source).unwrap();
        }
        let options = ExecutorOptions::default();
        let context = request_context(&chain, &tool, "body", &options).unwrap();
        let disc_opts = discover_options(&chain, &options).unwrap();
        let discovery = inputs::discover(&repo, "body", context.clone(), &disc_opts);
        assert_eq!(discovery["eligible"], true, "{}", discovery["reasons"]);
        let output = temp.path().join("worker");
        fs::create_dir(&output).unwrap();
        worker::capture(
            &repo,
            "body",
            context.clone(),
            &output.join("snapshot"),
            &worker::CaptureOptions {
                expected_projection_key: Some(
                    discovery["source_projection_key"].as_str().unwrap().into(),
                ),
                discovery: disc_opts,
                ..Default::default()
            },
        )
        .unwrap();
        normalize(
            &output.join("snapshot"),
            &discovery,
            &output.join("projection"),
        )
        .unwrap();
        for key in ["root", "target", "cargo-home", "home", "scratch"] {
            fs::create_dir(output.join(key)).unwrap();
        }
        let mut config = json!({"root":output.join("root"),"projection":output.join("projection"),"toolchain":chain,"runtime_libraries":tool["runtime_libraries"],"native_tools":tool["native_tools"],"repo":repo,"kernel":true,"argv":context["effective_config"]["argv"],"env":context["environment"],"request_options":{"features":[],"default_features":true},"recipe":{"target":AMD64,"flags":null}});
        for key in ["target", "cargo-home", "home", "scratch"] {
            config[key] = json!(output.join(key));
        }
        write_json(&output.join("worker.json"), &config).unwrap();
        let mut private = Map::new();
        for key in ["cargo-home", "home", "scratch"] {
            private.insert(key.into(), tree_identity(&output.join(key)).unwrap());
        }
        let report = json!({"body":"body","context":context,"source_projection_key":discovery["source_projection_key"],"action_key":discovery["action_key"],"prepared_inputs":prepared_identity(&config,&output,&context).unwrap(),"initial_private_inputs":private});
        (temp, output, report)
    }
    #[test]
    fn prepared_integrity_rejects_all_prior_mutation_classes_before_launch() {
        for mutation in [
            "config",
            "source",
            "same-mtime",
            "new-file",
            "new-dir",
            "manifest",
            "snapshot",
            "symlink",
            "action-key",
            "context",
            "cargo-config",
            "root-scaffold",
            "tool-leaf",
            "executor",
        ] {
            let (_temp, output, mut report) = prepared_fixture();
            assert!(validate_prepared(&output, &report).is_ok());
            let source = output
                .join("projection")
                .join(WORKSPACE)
                .join("body/src/lib.rs");
            match mutation {
                "config" => {
                    let mut config = read_json(&output.join("worker.json")).unwrap();
                    config["env"]["UNDECLARED"] = json!("changed");
                    write_json(&output.join("worker.json"), &config).unwrap();
                }
                "source" | "same-mtime" => {
                    let old = source.metadata().unwrap();
                    fs::write(&source, "pub fn changed(){}\n").unwrap();
                    if mutation == "same-mtime" {
                        let times = [
                            libc::timespec {
                                tv_sec: old.atime(),
                                tv_nsec: old.atime_nsec(),
                            },
                            libc::timespec {
                                tv_sec: old.mtime(),
                                tv_nsec: old.mtime_nsec(),
                            },
                        ];
                        let p = CString::new(source.as_os_str().as_bytes()).unwrap();
                        assert_eq!(
                            unsafe {
                                libc::utimensat(libc::AT_FDCWD, p.as_ptr(), times.as_ptr(), 0)
                            },
                            0
                        );
                    }
                }
                "new-file" => fs::write(source.parent().unwrap().join("added"), b"new").unwrap(),
                "new-dir" => fs::create_dir(source.parent().unwrap().join("new-empty")).unwrap(),
                "manifest" => fs::write(
                    output.join("projection").join(WORKSPACE).join("Cargo.toml"),
                    "changed",
                )
                .unwrap(),
                "snapshot" => {
                    let p = output
                        .join("snapshot")
                        .join(WORKSPACE)
                        .join("body/src/lib.rs");
                    fs::set_permissions(&p, fs::Permissions::from_mode(0o644)).unwrap();
                    fs::write(p, "changed").unwrap();
                }
                "symlink" => {
                    std::os::unix::fs::symlink("/etc/passwd", source.parent().unwrap().join("link"))
                        .unwrap()
                }
                "action-key" => report["action_key"] = json!("wrong"),
                "context" => report["context"]["effective_config"]["argv"]
                    .as_array_mut()
                    .unwrap()
                    .push(json!("--features=changed")),
                "cargo-config" => fs::write(
                    output.join("cargo-home/config.toml"),
                    "[build]\nrustflags=[\"--cfg=unexpected\"]\n",
                )
                .unwrap(),
                "root-scaffold" => {
                    fs::create_dir(output.join("root/etc")).unwrap();
                    fs::write(output.join("root/etc/injected"), b"no").unwrap();
                }
                "tool-leaf" => {
                    let config = read_json(&output.join("worker.json")).unwrap();
                    fs::write(
                        path(
                            &config["native_tools"]["/native-tools/bin/llvm-dlltool"],
                            "tool",
                        )
                        .unwrap(),
                        b"altered tool",
                    )
                    .unwrap();
                }
                "executor" => {
                    report["context"]["execution_policy"]["executor_source"] =
                        json!({"sha256":"wrong"})
                }
                _ => unreachable!(),
            }
            assert!(
                validate_prepared(&output, &report).is_err(),
                "{mutation} escaped prepared integrity checks"
            );
        }
    }
    struct TestHost {
        config: Value,
        prepared: Value,
        tool: Value,
        identity: Value,
        failure: String,
        post_provision: usize,
    }
    impl RunHost for TestHost {
        fn validate(&mut self, _: &Path, _: &Value) -> Result<Value> {
            if self.failure == "validation" {
                Err(fail("changed inputs"))
            } else {
                Ok(self.config.clone())
            }
        }
        fn launch(&mut self, config: &Path, probe: bool) -> Result<Vec<String>> {
            if probe {
                return Ok(vec![
                    "/bin/sh".into(),
                    "-c".into(),
                    if self.failure == "probe" {
                        "printf 'probe failed' >&2; exit 17"
                    } else {
                        "printf '{}'"
                    }
                    .into(),
                ]);
            }
            if self.failure == "compile-launch" {
                return Ok(vec!["/executor-control-no-such-file".into()]);
            }
            if self.failure == "namespace-control" {
                return Ok(vec![
                    name(&std::env::current_exe()?)?,
                    "--ignored".into(),
                    "--exact".into(),
                    "artifact_executor::tests::namespace_supervisor_harness".into(),
                    "--nocapture".into(),
                ]);
            }
            if self.failure == "signal-control" {
                let root = name(config.parent().unwrap())?;
                let leader = shlex::try_quote(&format!("{root}/leader.pid"))
                    .map_err(|e| fail(e.to_string()))?
                    .into_owned();
                let child = shlex::try_quote(&format!("{root}/descendant.pid"))
                    .map_err(|e| fail(e.to_string()))?
                    .into_owned();
                return Ok(vec![
                    "/bin/sh".into(),
                    "-c".into(),
                    format!("echo $$ > {leader}; sleep 30 & echo $! > {child}; wait"),
                ]);
            }
            let archive = config
                .parent()
                .unwrap()
                .join(format!("target/{AMD64}/release/libbody.a"));
            fs::create_dir_all(archive.parent().unwrap())?;
            let delay = if self.failure == "sampling" || self.failure == "interrupt" {
                "sleep 30; "
            } else {
                ""
            };
            let exit = if self.failure == "compile" { 13 } else { 0 };
            let mut script = format!(
                "printf '!<arch>\\nfixture' > {}; {delay}exit {exit}",
                shlex::try_quote(&name(&archive)?).map_err(|e| fail(e.to_string()))?
            );
            if self.failure == "config-post" {
                script = format!(
                    "printf '{{}}' > {}; {script}",
                    shlex::try_quote(&name(config)?).map_err(|e| fail(e.to_string()))?
                );
            }
            Ok(vec!["/bin/sh".into(), "-c".into(), script])
        }
        fn allocated(&mut self, root: &Path) -> Result<Value> {
            if self.failure == "sampling" {
                return Err(fail("sampling failed"));
            }
            if self.failure == "interrupt" {
                return Err(Error::new("interrupted").with_kind("KeyboardInterrupt"));
            }
            allocated(root)
        }
        fn provision(&mut self, _: &Path) -> Result<Value> {
            self.post_provision += 1;
            if self.failure == "tool-postcheck" {
                Err(fail("provision postcheck failed"))
            } else if self.failure == "tool-changed" {
                Ok(json!({"changed":true}))
            } else {
                Ok(self.tool.clone())
            }
        }
        fn lane_projection(
            &mut self,
            _: &Path,
            _: &str,
            _: Value,
            _: &Path,
            _: &ExecutorOptions,
        ) -> Result<Value> {
            Ok(
                json!({"eligible":true,"source_projection_key":if self.failure=="lane-changed"{"wrong"}else{"source"}}),
            )
        }
        fn prepared_identity(&mut self, _: &Value, _: &Path, _: &Value) -> Result<Value> {
            if self.failure == "projection-post" {
                Ok(json!({"changed":true}))
            } else {
                Ok(self.prepared.clone())
            }
        }
        fn implementation_identity(&mut self) -> Result<Value> {
            if self.failure == "implementation-post" {
                Ok(json!({"changed":true}))
            } else {
                Ok(self.identity.clone())
            }
        }
    }
    fn run_fixture(failure: &str) -> (tempfile::TempDir, TestHost) {
        let temp = tempfile::tempdir().unwrap();
        let output = temp.path();
        for key in ["target", "snapshot", "projection", "scratch"] {
            fs::create_dir(output.join(key)).unwrap();
        }
        let tool = json!({"tool":true});
        let identity = json!({"executor":true});
        let prepared = json!({"prepared":true});
        let config = json!({"repo":output.join("repo"),"toolchain":output.join("toolchain"),"request_options":{},"recipe":{"target":AMD64,"flags":null}});
        write_json(&output.join("worker.json"), &config).unwrap();
        let report = json!({"body":"body","compile_status":"not run","context":{"target":{"builtin":AMD64},"toolchain":tool,"execution_policy":identity},"source_projection_key":"source","action_key":"action","prepared_inputs":prepared});
        write_json(&output.join("report.json"), &report).unwrap();
        (
            temp,
            TestHost {
                config,
                prepared,
                tool,
                identity,
                failure: failure.into(),
                post_provision: 0,
            },
        )
    }
    #[test]
    fn supervision_success_failure_and_postcheck_controls_preserve_final_evidence() {
        let cases = [
            ("validation", "prepared-input-validation"),
            ("probe", "isolation-probe"),
            ("compile-launch", "compile-launch"),
            ("compile", "compile"),
            ("sampling", "compile"),
            ("interrupt", "compile"),
            ("tool-postcheck", "toolchain-postcheck"),
            ("tool-changed", "toolchain-postcheck"),
            ("projection-post", "prepared-input-postcheck"),
            ("config-post", "prepared-input-postcheck"),
            ("lane-changed", "lane-input-revalidation"),
            ("implementation-post", "implementation-postcheck"),
        ];
        for (failure, stage) in cases {
            let (temp, mut host) = run_fixture(failure);
            let error = run_engine(temp.path(), &mut host).unwrap_err();
            let report = read_json(&temp.path().join("report.json")).unwrap();
            assert_eq!(report["failure_stage"], stage, "{failure}: {error}");
            assert_eq!(
                report["compile_status"],
                if failure == "interrupt" {
                    "interrupted"
                } else {
                    "failed"
                }
            );
            if failure == "probe" {
                assert_eq!(report["failure"]["stderr"], "probe failed");
                assert_eq!(report["failure"]["returncode"], 17);
            }
            if failure == "sampling" || failure == "interrupt" {
                assert!(report["exit_code"].is_i64());
            }
            assert!(run_engine(temp.path(), &mut host)
                .unwrap_err()
                .message
                .contains("refusing to overwrite evidence"));
        }
        let (temp, mut host) = run_fixture("");
        let report = run_engine(temp.path(), &mut host).unwrap();
        assert_eq!(report["compile_status"], "success");
        assert_eq!(host.post_provision, 1);
        assert_eq!(report["prepared_input_validation"], "identical");
        assert_eq!(report["toolchain_postcheck"], "identical");
        assert!(report["archive"]["sha256"].as_str().is_some());
    }
    #[test]
    #[ignore = "child process fixture invoked by real_signals_stop_only_the_owned_session"]
    fn signal_worker_harness() {
        let output = PathBuf::from(
            std::env::var_os("VIBEOS_EXECUTOR_SIGNAL_FIXTURE").expect("signal fixture path"),
        );
        let mut config = read_json(&output.join("worker.json")).unwrap();
        let namespace = std::env::var_os("VIBEOS_EXECUTOR_NAMESPACE_FIXTURE").is_some();
        let _lease = if namespace {
            Some(fixture_lease(&output))
        } else {
            None
        };
        if namespace {
            for key in ["root", "cargo-home", "home"] {
                fs::create_dir(output.join(key)).unwrap();
            }
            let tools = output.join("toolchain/bin");
            fs::create_dir_all(&tools).unwrap();
            fs::copy("/usr/bin/dash", tools.join("cargo")).unwrap();
            fs::copy("/usr/bin/sleep", tools.join("sleep")).unwrap();
            let mut libraries = BTreeSet::new();
            for executable in ["/usr/bin/dash", "/usr/bin/sleep"] {
                let out = Command::new("ldd").arg(executable).output().unwrap();
                assert!(out.status.success());
                for cap in Regex::new(r"(?:=>\s*)?(/\S+)")
                    .unwrap()
                    .captures_iter(&String::from_utf8(out.stdout).unwrap())
                {
                    libraries.insert(cap[1].to_owned());
                }
            }
            for key in [
                "root",
                "projection",
                "toolchain",
                "target",
                "cargo-home",
                "home",
                "scratch",
            ] {
                config[key] = json!(output.join(key));
            }
            config["runtime_libraries"] = json!(libraries);
            config["env"] = json!({"PATH":"/toolchain/bin"});
            config["argv"] = json!([
                "/toolchain/bin/cargo",
                "-c",
                "/toolchain/bin/sleep 30 & : > /scratch/namespace-ready; wait"
            ]);
            write(
                &output.join("projection"),
                &format!("{WORKSPACE}/Cargo.toml"),
                "namespace fixture\n",
            );
            write_json(&output.join("worker.json"), &config).unwrap();
        }
        fs::write(output.join("executor.pid"), std::process::id().to_string()).unwrap();
        let report = read_json(&output.join("report.json")).unwrap();
        let mut host = TestHost {
            config,
            prepared: report["prepared_inputs"].clone(),
            tool: report["context"]["toolchain"].clone(),
            identity: report["context"]["execution_policy"].clone(),
            failure: if namespace {
                "namespace-control"
            } else {
                "signal-control"
            }
            .into(),
            post_provision: 0,
        };
        match run_with_host(&output, &mut host) {
            Ok(_) => std::process::exit(0),
            Err(error) => {
                eprintln!("{:?}: {}", error.kind, error.message);
                std::process::exit(2)
            }
        }
    }
    struct TestChild(Child);
    impl Drop for TestChild {
        fn drop(&mut self) {
            if self.0.try_wait().ok().flatten().is_none() {
                let _ = self.0.kill();
                let _ = self.0.wait();
            }
        }
    }
    #[test]
    fn real_signals_stop_only_the_owned_session() {
        for (signal, kind) in [
            (libc::SIGINT, "KeyboardInterrupt"),
            (libc::SIGTERM, "InterruptedError"),
        ] {
            let (temp, _) = run_fixture("");
            let mut unrelated = TestChild(Command::new("sleep").arg("30").spawn().unwrap());
            let mut runner = TestChild(
                Command::new(std::env::current_exe().unwrap())
                    .args([
                        "--ignored",
                        "--exact",
                        "artifact_executor::tests::signal_worker_harness",
                        "--nocapture",
                    ])
                    .env("VIBEOS_EXECUTOR_SIGNAL_FIXTURE", temp.path())
                    .env("VIBEOS_HOST_JOB_HELD", "1")
                    .stdout(Stdio::null())
                    .stderr(Stdio::piped())
                    .spawn()
                    .unwrap(),
            );
            let started = Instant::now();
            let descendant = loop {
                if let Ok(bytes) = fs::read_to_string(temp.path().join("descendant.pid")) {
                    if let Ok(pid) = bytes.trim().parse::<i32>() {
                        break pid;
                    }
                }
                assert!(
                    started.elapsed() < Duration::from_secs(10),
                    "signal control worker never started"
                );
                assert!(
                    runner.0.try_wait().unwrap().is_none(),
                    "signal control runner exited before worker startup"
                );
                std::thread::sleep(Duration::from_millis(20));
            };
            let leader: i32 = fs::read_to_string(temp.path().join("leader.pid"))
                .unwrap()
                .trim()
                .parse()
                .unwrap();
            assert_eq!(unsafe { libc::kill(runner.0.id() as i32, signal) }, 0);
            let started = Instant::now();
            let status = loop {
                if let Some(status) = runner.0.try_wait().unwrap() {
                    break status;
                }
                assert!(
                    started.elapsed() < Duration::from_secs(10),
                    "signal control runner failed to stop"
                );
                std::thread::sleep(Duration::from_millis(20));
            };
            assert_eq!(status.code(), Some(2));
            let report = read_json(&temp.path().join("report.json")).unwrap();
            assert_eq!(report["compile_status"], "interrupted");
            assert_eq!(report["failure_stage"], "compile");
            assert_eq!(report["failure"]["type"], kind);
            assert!(report["exit_code"].is_i64());
            for pid in [leader, descendant] {
                if let Ok(stat) = fs::read_to_string(format!("/proc/{pid}/stat")) {
                    assert!(
                        stat.split_once(") ").unwrap().1.starts_with("Z "),
                        "owned child {pid} survived cancellation: {stat}"
                    );
                }
            }
            assert!(
                unrelated.0.try_wait().unwrap().is_none(),
                "unrelated sibling was killed"
            );
        }
    }
    fn fixture_lease(output: &Path) -> File {
        use std::os::fd::AsRawFd;
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(output.join("lease.lock"))
            .unwrap();
        assert_eq!(unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX) }, 0);
        let flags = unsafe { libc::fcntl(file.as_raw_fd(), libc::F_GETFD) };
        assert!(flags >= 0);
        assert_eq!(
            unsafe { libc::fcntl(file.as_raw_fd(), libc::F_SETFD, flags & !libc::FD_CLOEXEC) },
            0
        );
        file
    }
    #[test]
    #[ignore = "namespace parent fixture invoked by abrupt_owner_death_controls"]
    fn namespace_supervisor_harness() {
        let mut command = Command::new("unshare");
        command
            .args([
                "--user",
                "--map-root-user",
                "--mount",
                "--net",
                "--pid",
                "--fork",
                "--kill-child",
            ])
            .arg(std::env::current_exe().unwrap())
            .args([
                "--ignored",
                "--exact",
                "artifact_executor::tests::inside_worker_harness",
                "--nocapture",
            ]);
        match supervised_status(&mut command) {
            Ok(code) => std::process::exit(code as i32),
            Err(error) => {
                eprintln!("{:?}: {}", error.kind, error.message);
                std::process::exit(2)
            }
        }
    }
    #[test]
    #[ignore = "real namespace child fixture invoked by abrupt_owner_death_controls"]
    fn inside_worker_harness() {
        let output = PathBuf::from(
            std::env::var_os("VIBEOS_EXECUTOR_SIGNAL_FIXTURE").expect("namespace fixture path"),
        );
        inside(&read_json(&output.join("worker.json")).unwrap(), false).unwrap();
    }
    #[test]
    #[ignore = "broker supervision child fixture invoked by abrupt_owner_death_controls"]
    fn broker_supervisor_harness() {
        let output = PathBuf::from(
            std::env::var_os("VIBEOS_EXECUTOR_SIGNAL_FIXTURE").expect("supervisor fixture path"),
        );
        let _lease = fixture_lease(&output);
        let mut command = Command::new("/bin/sh");
        command
            .args(["-c", "\"$@\" & wait", "broker-fixture"])
            .arg(std::env::current_exe().unwrap())
            .args([
                "--ignored",
                "--exact",
                "artifact_executor::tests::signal_worker_harness",
                "--nocapture",
            ])
            .env("VIBEOS_EXECUTOR_SIGNAL_FIXTURE", &output)
            .env("VIBEOS_HOST_JOB_HELD", "1");
        match supervised_output(&mut command, None) {
            Ok(_) => std::process::exit(0),
            Err(error) => {
                eprintln!("{:?}: {}", error.kind, error.message);
                std::process::exit(2)
            }
        }
    }
    fn process_descendants(pid: u32) -> Vec<u32> {
        let mut found = BTreeSet::new();
        let mut todo = vec![pid];
        while let Some(pid) = todo.pop() {
            if let Ok(tasks) = fs::read_dir(format!("/proc/{pid}/task")) {
                for task in tasks.flatten() {
                    if let Ok(children) = fs::read_to_string(task.path().join("children")) {
                        for child in children
                            .split_whitespace()
                            .filter_map(|s| s.parse::<u32>().ok())
                        {
                            if found.insert(child) {
                                todo.push(child);
                            }
                        }
                    }
                }
            }
        }
        found.into_iter().collect()
    }
    fn terminated(pid: u32) -> bool {
        match fs::read_to_string(format!("/proc/{pid}/stat")) {
            Ok(stat) => stat
                .rsplit_once(") ")
                .is_some_and(|(_, tail)| tail.starts_with("Z ")),
            Err(_) => true,
        }
    }
    #[test]
    fn abrupt_owner_death_controls_stop_workers_and_release_inherited_lease() {
        use std::os::fd::AsRawFd;
        for supervisor in [false, true] {
            let (temp, _) = run_fixture("");
            let mut unrelated = TestChild(Command::new("sleep").arg("30").spawn().unwrap());
            let harness = if supervisor {
                "artifact_executor::tests::broker_supervisor_harness"
            } else {
                "artifact_executor::tests::signal_worker_harness"
            };
            let mut command = Command::new(std::env::current_exe().unwrap());
            command
                .args(["--ignored", "--exact", harness, "--nocapture"])
                .env("VIBEOS_EXECUTOR_SIGNAL_FIXTURE", temp.path())
                .env("VIBEOS_HOST_JOB_HELD", "1")
                .stdout(Stdio::null())
                .stderr(Stdio::piped());
            if !supervisor {
                command.env("VIBEOS_EXECUTOR_NAMESPACE_FIXTURE", "1");
            }
            let mut runner = TestChild(command.spawn().unwrap());
            let started = Instant::now();
            loop {
                let ready = if supervisor {
                    temp.path().join("descendant.pid").exists()
                } else {
                    temp.path().join("scratch/namespace-ready").exists()
                };
                if ready {
                    break;
                }
                assert!(
                    started.elapsed() < Duration::from_secs(15),
                    "owner-death worker failed to start (supervisor={supervisor})"
                );
                if runner.0.try_wait().unwrap().is_some() {
                    let mut message = String::new();
                    runner
                        .0
                        .stderr
                        .take()
                        .unwrap()
                        .read_to_string(&mut message)
                        .unwrap();
                    panic!("owner-death runner exited before worker start: {message}");
                }
                std::thread::sleep(Duration::from_millis(20));
            }
            let descendants = process_descendants(runner.0.id());
            assert!(
                descendants.len() >= 3,
                "expected a real live child subtree: {descendants:?}"
            );
            let lease = OpenOptions::new()
                .read(true)
                .write(true)
                .open(temp.path().join("lease.lock"))
                .unwrap();
            assert_ne!(
                unsafe { libc::flock(lease.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) },
                0,
                "fixture lease was not inherited/held"
            );
            assert_eq!(
                unsafe { libc::kill(runner.0.id() as i32, libc::SIGKILL) },
                0
            );
            let status = runner.0.wait().unwrap();
            assert_eq!(status.signal(), Some(libc::SIGKILL));
            let started = Instant::now();
            loop {
                if descendants.iter().all(|pid| terminated(*pid)) {
                    break;
                }
                assert!(
                    started.elapsed() < Duration::from_secs(10),
                    "owner death left live descendants: {:?}",
                    descendants
                        .iter()
                        .filter(|pid| !terminated(**pid))
                        .collect::<Vec<_>>()
                );
                std::thread::sleep(Duration::from_millis(20));
            }
            assert_eq!(
                unsafe { libc::flock(lease.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) },
                0,
                "inherited lease survived subtree termination"
            );
            let report = read_json(&temp.path().join("report.json")).unwrap();
            if supervisor {
                assert_eq!(report["compile_status"], "interrupted");
                assert_eq!(report["failure_stage"], "compile");
            } else {
                assert_eq!(
                    report["compile_status"], "running",
                    "SIGKILL cannot invent a final receipt"
                );
                assert!(temp.path().join("run.started").exists());
            }
            assert!(
                unrelated.0.try_wait().unwrap().is_none(),
                "unrelated sibling died"
            );
        }
    }
}
