//! Pinned checkpoint bundles. Restoration never consults current boot media.
use crate::{backend, vm::Vm, Error, Result};
use regex::Regex;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
const TAG: &str = "checkpoint";
fn s<'a>(v: &'a Value, k: &str) -> &'a str {
    v.get(k).and_then(Value::as_str).unwrap_or("")
}
fn strings(v: &Value, k: &str) -> Result<Vec<String>> {
    Ok(serde_json::from_value(
        v.get(k)
            .cloned()
            .ok_or_else(|| Error::new(format!("missing {k}")))?,
    )?)
}
pub fn digest(path: &Path) -> Result<String> {
    let mut f = File::open(path)?;
    let mut h = Sha256::new();
    let mut buf = vec![0; 1024 * 1024];
    loop {
        let n = f.read(&mut buf)?;
        if n == 0 {
            break;
        }
        h.update(&buf[..n]);
    }
    Ok(format!("{:x}", h.finalize()))
}
pub fn host_path(raw: &str) -> Result<PathBuf> {
    let re = Regex::new(r"^([A-Za-z]):[/\\](.*)$")?;
    if let Some(c) = re.captures(raw) {
        return Ok(Path::new("/mnt")
            .join(c[1].to_ascii_lowercase())
            .join(c[2].replace('\\', "/")));
    }
    let p = PathBuf::from(raw);
    if !p.is_absolute() {
        return Err(Error::new(format!(
            "Expected an absolute media path: {raw}"
        )));
    }
    Ok(p)
}
fn drive(raw: &str) -> Result<Vec<(String, String)>> {
    if raw.contains(",,") {
        return Err(Error::new(format!("Unsupported drive syntax: {raw}")));
    }
    let mut out: Vec<(String, String)> = Vec::new();
    for token in raw.split(',').filter(|s| !s.is_empty()) {
        let (key, value) = token.split_once('=').unwrap_or(("file", token));
        if let Some(old) = out.iter_mut().find(|(k, _)| k == key) {
            old.1 = value.into();
        } else {
            out.push((key.into(), value.into()));
        }
    }
    Ok(out)
}
fn dget<'a>(d: &'a [(String, String)], k: &str) -> Option<&'a str> {
    d.iter().find(|(key, _)| key == k).map(|(_, v)| v.as_str())
}
fn format_drive(d: &[(String, String)]) -> String {
    d.iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join(",")
}
pub fn validate_template(argv: &[String]) -> Result<(Vec<String>, Vec<Value>)> {
    if argv.is_empty() {
        return Err("Missing checkpoint QEMU executable".into());
    }
    let pairs = [
        "-M", "-machine", "-cpu", "-accel", "-m", "-smp", "-device", "-global", "-fw_cfg",
        "-display", "-vga", "-rtc", "-nic", "-net",
    ];
    let controls = [
        "-name",
        "-qmp",
        "-mon",
        "-serial",
        "-monitor",
        "-debugcon",
        "-gdb",
    ];
    let flags = ["-no-reboot", "-no-shutdown", "-nodefaults"];
    let mut out = Vec::new();
    let mut media = Vec::new();
    let mut i = 1;
    let external = Regex::new(r"(^|,)(file|mem-path|romfile|chardev|netdev|rng|fsdev)=")?;
    let charid = Regex::new(r",id=(ser0|qmp0)(,|$)")?;
    while i < argv.len() {
        let option = &argv[i];
        i += 1;
        if option == "-S" {
            continue;
        }
        if flags.contains(&option.as_str()) {
            out.push(option.clone());
            continue;
        }
        let value = argv
            .get(i)
            .ok_or_else(|| Error::new(format!("Missing value for {option}")))?;
        i += 1;
        if controls.contains(&option.as_str()) {
            continue;
        }
        if option == "-chardev" {
            if !charid.is_match(value) {
                return Err(Error::new(format!(
                    "External chardev cannot be checkpointed: {value}"
                )));
            }
            continue;
        }
        if option == "-drive" {
            let mut d = drive(value)?;
            let file = dget(&d, "file")
                .ok_or_else(|| Error::new(format!("Unsupported drive syntax: {value}")))?
                .to_owned();
            let readonly = dget(&d, "readonly") == Some("on");
            let format = dget(&d, "format").unwrap_or("raw").to_owned();
            if !readonly && format != "qcow2" {
                return Err("Writable checkpoint media must be qcow2".into());
            }
            let name = format!(
                "media-{:02}.{}",
                media.len(),
                if readonly { "raw" } else { "qcow2" }
            );
            media.push(
                json!({"name":name,"source":host_path(&file)?,"readonly":readonly,"format":format}),
            );
            d.iter_mut().find(|(k, _)| k == "file").unwrap().1 = format!("@{name}");
            out.extend([option.clone(), format_drive(&d)]);
        } else if pairs.contains(&option.as_str()) {
            if option == "-nic" && !matches!(value.as_str(), "none" | "user,restrict=on") {
                return Err("Live networking is not checkpointable".into());
            }
            if option == "-net" && value != "nic" {
                return Err("Only a disconnected default NIC is supported".into());
            }
            if option == "-accel" && value.split(',').next() != Some("tcg") {
                return Err("Checkpoint capture currently requires TCG".into());
            }
            if external.is_match(value) {
                return Err(Error::new(format!(
                    "Unpinned external device input: {option} {value}"
                )));
            }
            if option == "-device"
                && ["usb", "vfio", "virtio-net"]
                    .iter()
                    .any(|x| value.contains(x))
            {
                return Err(Error::new(format!(
                    "Unsupported checkpoint device: {value}"
                )));
            }
            out.extend([option.clone(), value.clone()]);
        } else {
            return Err(Error::new(format!(
                "Unsupported checkpoint option: {option}"
            )));
        }
    }
    if media.is_empty() || !media.iter().any(|v| v["readonly"] == false) {
        return Err("No writable snapshot store".into());
    }
    Ok((out, media))
}
fn binary(raw: &str) -> Result<PathBuf> {
    let p = PathBuf::from(raw);
    if p.components().count() > 1 {
        return Ok(fs::canonicalize(p)?);
    }
    let paths = std::env::var_os("PATH").unwrap_or_default();
    for dir in std::env::split_paths(&paths) {
        let p = dir.join(raw);
        if p.is_file() {
            return Ok(fs::canonicalize(p)?);
        }
    }
    Err(Error::new(format!("QEMU executable not found: {raw}")))
}
pub fn emulator_identity(raw: &str) -> Result<Value> {
    let p = binary(raw)?;
    let (rc, stdout, stderr) = backend::run_capture(
        p.to_str().ok_or("QEMU path is not Unicode")?,
        &["--version".into()],
        Duration::from_secs(30),
    )?;
    if rc != 0 {
        return Err(Error::new(format!("QEMU --version exited {rc}: {stderr}")));
    }
    let mut deps = BTreeMap::new();
    if p.extension().is_some_and(|s| s == "exe") {
        fn walk(base: &Path, dir: &Path, out: &mut BTreeMap<String, String>) -> Result<()> {
            for entry in fs::read_dir(dir)? {
                let p = entry?.path();
                if p.is_dir() {
                    walk(base, &p, out)?
                } else if p.is_file()
                    && p.extension().is_some_and(|s| {
                        matches!(
                            s.to_string_lossy().to_ascii_lowercase().as_str(),
                            "dll" | "bin" | "rom" | "fd"
                        )
                    })
                {
                    out.insert(
                        p.strip_prefix(base)
                            .map_err(|e| Error::new(e.to_string()))?
                            .to_string_lossy()
                            .into_owned(),
                        digest(&p)?,
                    );
                }
            }
            Ok(())
        }
        walk(p.parent().unwrap(), p.parent().unwrap(), &mut deps)?;
    }
    Ok(json!({"path":p,"sha256":digest(&p)?,"version":stdout,"dependencies":deps}))
}
fn media_entries<'a>(m: &'a Value) -> Result<&'a Vec<Value>> {
    m["media"]
        .as_array()
        .ok_or_else(|| Error::new("Missing checkpoint media inventory"))
}
fn safe_extra(name: &str) -> bool {
    !name.is_empty()
        && name != "."
        && name != ".."
        && Path::new(name).file_name().is_some_and(|n| n == name)
        && !name.contains('\\')
}
/// Validate media/extras before invoking any executable or accessing today's build.
pub fn validate_bundle(bundle: &Path, m: &Value) -> Result<()> {
    if m["schema"] != 1 || m["accel"] != "tcg" {
        return Err("Unsupported checkpoint schema/accelerator".into());
    }
    if !matches!(s(m, "backend"), "windows-host" | "native") {
        return Err("Unsupported checkpoint backend".into());
    }
    let names = Regex::new(r"^media-\d+\.(raw|qcow2)$")?;
    let mut seen = BTreeSet::new();
    for item in media_entries(m)? {
        let name = s(item, "name");
        if !names.is_match(name) || !seen.insert(name) {
            return Err("Invalid bundle media name".into());
        }
        let p = bundle.join(name);
        if p.is_symlink() || digest(&p).ok().as_deref() != Some(s(item, "sha256")) {
            return Err(Error::new(format!(
                "Checkpoint media missing or damaged: {}",
                p.display()
            )));
        }
    }
    for item in m
        .get("extras")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let name = s(item, "name");
        if !safe_extra(name) || !seen.insert(name) {
            return Err("Invalid checkpoint extra name".into());
        }
        let p = bundle.join(name);
        if p.is_symlink() || digest(&p).ok().as_deref() != Some(s(item, "sha256")) {
            return Err(Error::new(format!(
                "Checkpoint metadata/symbols damaged: {name}"
            )));
        }
    }
    Ok(())
}
pub fn load(bundle: &Path) -> Result<Value> {
    let manifest: Value = serde_json::from_slice(&fs::read(bundle.join("manifest.json"))?)?;
    validate_bundle(bundle, &manifest)?;
    if emulator_identity(s(&manifest["emulator"], "path"))? != manifest["emulator"] {
        return Err("QEMU build changed; use the checkpoint's original QEMU installation".into());
    }
    Ok(manifest)
}
pub fn prepare_guest(
    _root: &Path,
    bundle: &Path,
    m: &Value,
    staging: Option<&Path>,
    windows: bool,
) -> Result<Vec<String>> {
    // Repeat admission checks after the VM has acquired resources; no stale load DTO
    // can bypass a changed bundle or emulator between inspect and launch.
    validate_bundle(bundle, m)?;
    if emulator_identity(s(&m["emulator"], "path"))? != m["emulator"] {
        return Err("QEMU build changed; use the checkpoint's original QEMU installation".into());
    }
    let staging = staging.ok_or("Missing private checkpoint staging directory")?;
    fs::create_dir_all(staging)?;
    let mut paths = BTreeMap::new();
    for item in media_entries(m)? {
        let src = bundle.join(s(item, "name"));
        let target = staging.join(s(item, "name"));
        if fs::canonicalize(&src).ok() == fs::canonicalize(&target).ok() && target.exists() {
            return Err("Checkpoint attempts must not modify their seed media".into());
        }
        let path = if windows {
            if item["readonly"] == true {
                backend::stage_readonly_file(&src, staging)?
            } else {
                backend::stage_file(&src, staging)?
            }
        } else {
            fs::copy(&src, &target)?;
            target.to_string_lossy().into_owned()
        };
        if digest(&target)? != s(item, "sha256") {
            return Err(Error::new(format!(
                "Checkpoint media changed during staging: {}",
                src.display()
            )));
        }
        paths.insert(format!("@{}", s(item, "name")), path);
    }
    let mut args = strings(m, "argv")?;
    let mut consumed = BTreeSet::new();
    let mut i = 0;
    while i < args.len() {
        if args[i] == "-drive" {
            let raw = args.get(i + 1).ok_or("Missing pinned drive value")?;
            let mut d = drive(raw)?;
            let file = dget(&d, "file")
                .ok_or("Missing pinned drive file")?
                .to_owned();
            let mapped = paths
                .get(&file)
                .ok_or("Checkpoint drive references media outside its bundle")?;
            consumed.insert(file);
            d.iter_mut().find(|(k, _)| k == "file").unwrap().1 = mapped.clone();
            args[i + 1] = format_drive(&d);
            i += 2;
        } else {
            i += 1;
        }
    }
    if consumed.len() != paths.len() {
        return Err("Checkpoint media inventory and guest argv differ".into());
    }
    // Re-run the closed topology/device guard after resolving all placeholders.
    let mut full = vec![s(&m["emulator"], "path").to_owned()];
    full.extend(args.clone());
    validate_template(&full)?;
    Ok(args)
}
pub fn pause(vm: &mut Vm) -> Result<()> {
    vm.qmp.execute("stop", json!({}))?;
    let v = vm.qmp.execute("query-status", json!({}))?;
    let state = &v["return"];
    if state["running"] != false || !matches!(s(state, "status"), "paused" | "prelaunch" | "debug")
    {
        return Err(Error::new(format!("QEMU did not stop cleanly: {state}")));
    }
    Ok(())
}
pub fn snapshot(vm: &mut Vm, operation: &str) -> Result<()> {
    if !matches!(operation, "savevm" | "loadvm") {
        return Err("Unsupported checkpoint snapshot operation".into());
    }
    pause(vm)?;
    let reply = vm.qmp.hmp(&format!("{operation} {TAG}"))?;
    if !reply.trim().is_empty() {
        return Err(Error::new(format!("{operation} failed: {}", reply.trim())));
    }
    pause(vm)
}
pub fn registers(vm: &mut Vm) -> Result<String> {
    let v = vm.qmp.hmp("info registers -a")?.trim().to_owned();
    if v.is_empty() || !Regex::new(r"\b(PC|RIP|EIP)=")?.is_match(&v) {
        return Err("Cannot obtain all-vCPU register witness".into());
    }
    Ok(v)
}
pub fn restore(vm: &mut Vm, m: &Value) -> Result<()> {
    snapshot(vm, "loadvm")?;
    if registers(vm)? != s(m, "registers") {
        return Err("Restored vCPU registers differ from saved checkpoint".into());
    }
    let cpus = vm.qmp.execute("query-cpus-fast", json!({}))?;
    if cpus["return"].as_array().map(Vec::len) != m["vcpus"].as_u64().map(|x| x as usize) {
        return Err("Restored CPU topology differs".into());
    }
    let deadline = Instant::now() + Duration::from_secs(5);
    while !vm
        .serial
        .read_available(Duration::from_millis(200))?
        .is_empty()
    {
        if Instant::now() >= deadline {
            return Err("Serial transport did not quiesce while stopped".into());
        }
    }
    Ok(())
}
pub fn watch(vm: &mut Vm, until: &str, fail: &str, timeout: f64, log: &Path) -> Result<String> {
    if !timeout.is_finite() || timeout <= 0. || until.is_empty() {
        return Err("Checkpoint watch needs a positive timeout and nonempty marker".into());
    }
    let failure = fancy_regex::Regex::new(fail).map_err(|e| Error::new(e.to_string()))?;
    let end = Instant::now()
        .checked_add(Duration::from_secs_f64(timeout))
        .ok_or("Checkpoint timeout overflow")?;
    let mut text = String::new();
    let mut file = File::create(log)?;
    while Instant::now() < end {
        let d = end
            .saturating_duration_since(Instant::now())
            .as_secs_f64()
            .clamp(0.001, 0.1);
        let chunk = vm.poll_serial(d)?;
        file.write_all(chunk.as_bytes())?;
        file.flush()?;
        text.push_str(&chunk);
        if failure
            .is_match(&text)
            .map_err(|e| Error::new(e.to_string()))?
        {
            return Ok("failure-pattern".into());
        }
        if text.contains(until) {
            return Ok("reached-marker".into());
        }
    }
    Ok("timeout".into())
}
fn qemu_img(path: &Path, format: &str) -> Result<Value> {
    let (rc, out, err) = backend::run_capture(
        "qemu-img",
        &[
            "info".into(),
            "--output=json".into(),
            "-f".into(),
            format.into(),
            path.to_string_lossy().into_owned(),
        ],
        Duration::from_secs(120),
    )?;
    if rc != 0 {
        return Err(Error::new(format!("qemu-img info exited {rc}: {err}")));
    }
    Ok(serde_json::from_str(&out)?)
}
pub fn export(
    vm: &mut Vm,
    dest: &Path,
    parent: Option<&str>,
    extras: &BTreeMap<String, PathBuf>,
) -> Result<Value> {
    let launch_path = vm
        .config
        .live_log
        .with_file_name(format!("{}_launch.json", vm.config.arch));
    let launch: Value = serde_json::from_slice(&fs::read(&launch_path)?)?;
    let argv = strings(&launch, "argv")?;
    let (template, mut media) = validate_template(&argv)?;
    if launch["accel"] != "tcg" {
        return Err("Checkpoint export requires TCG".into());
    }
    snapshot(vm, "savevm")?;
    let witness = registers(vm)?;
    let cpus = vm.qmp.execute("query-cpus-fast", json!({}))?;
    let cpus = cpus["return"]
        .as_array()
        .ok_or("Invalid CPU topology response")?
        .len();
    vm.qmp.execute("quit", json!({}))?;
    let rc = vm.wait_exit(30.)?;
    if rc != 0 {
        return Err(Error::new(format!(
            "QEMU exited abnormally before checkpoint export: {rc}"
        )));
    }
    if dest.exists() {
        return Err(Error::new(format!(
            "Checkpoint already exists: {}",
            dest.display()
        )));
    }
    let base = dest
        .parent()
        .ok_or("Checkpoint destination has no parent")?;
    fs::create_dir_all(base)?;
    let temp = tempfile::Builder::new()
        .prefix(".partial-")
        .tempdir_in(base)?;
    let mut has_state = false;
    for item in &mut media {
        let src = PathBuf::from(s(item, "source"));
        item.as_object_mut().unwrap().remove("source");
        let target = temp.path().join(s(item, "name"));
        fs::copy(src, &target)?;
        item["sha256"] = json!(digest(&target)?);
        let info = qemu_img(&target, s(item, "format"))?;
        if info["dirty-flag"] == true {
            return Err(Error::new(format!(
                "Image was not cleanly closed: {}",
                s(item, "name")
            )));
        }
        if info
            .get("backing-filename")
            .is_some_and(|x| x.as_str().is_some_and(|s| !s.is_empty()))
        {
            return Err("Backing chains must be flattened before capture".into());
        }
        let snaps = info.get("snapshots").and_then(Value::as_array);
        let named = snaps.is_some_and(|a| a.iter().any(|s| s["name"] == TAG));
        if item["readonly"] != true && !named {
            return Err(Error::new(format!(
                "Missing snapshot in {}",
                s(item, "name")
            )));
        }
        has_state |= snaps.is_some_and(|a| {
            a.iter()
                .any(|s| s["name"] == TAG && s["vm-state-size"].as_u64().unwrap_or(0) > 0)
        });
    }
    if !has_state {
        return Err("Snapshot contains no saved VM state".into());
    }
    let mut config = serde_json::to_value(&vm.config)?;
    if let Some(map) = config.as_object_mut() {
        for key in ["env", "windows"] {
            map.insert(key.into(), json!({}));
        }
        map.insert("owner_pid".into(), json!(0));
    }
    let mut manifest = json!({"schema":1,"config":config,"backend":vm.config.backend,"accel":"tcg","argv":template,"emulator":emulator_identity(&argv[0])?,"media":media,"registers":witness,"vcpus":cpus,"parent":parent,"created":SystemTime::now().duration_since(UNIX_EPOCH).map_err(|e|Error::new(e.to_string()))?.as_secs_f64(),"semantics":"checkpoint-retry, not deterministic replay","debugcon":argv.iter().any(|v|v=="-debugcon"),"extras":[]});
    let mut extra_entries = Vec::new();
    for (name, src) in extras {
        if !safe_extra(name)
            || name == "manifest.json"
            || media_entries(&manifest)?
                .iter()
                .any(|m| s(m, "name") == name)
        {
            return Err("Invalid checkpoint extra name".into());
        }
        let dst = temp.path().join(name);
        fs::copy(src, &dst)?;
        extra_entries.push(json!({"name":name,"sha256":digest(&dst)?}));
    }
    manifest["extras"] = json!(extra_entries);
    fs::write(
        temp.path().join("manifest.json"),
        format!("{}\n", serde_json::to_string_pretty(&manifest)?),
    )?;
    // Never overwrite a bundle another capture published after the initial check.
    use std::os::unix::ffi::OsStrExt;
    let from = std::ffi::CString::new(temp.path().as_os_str().as_bytes())
        .map_err(|e| Error::new(e.to_string()))?;
    let to = std::ffi::CString::new(dest.as_os_str().as_bytes())
        .map_err(|e| Error::new(e.to_string()))?;
    if unsafe {
        libc::renameat2(
            libc::AT_FDCWD,
            from.as_ptr(),
            libc::AT_FDCWD,
            to.as_ptr(),
            libc::RENAME_NOREPLACE,
        )
    } != 0
    {
        return Err(std::io::Error::last_os_error().into());
    }
    Ok(manifest)
}
pub fn dispatch_vm(vm: &mut Vm, op: &str, a: &Value) -> Result<Value> {
    Ok(match op {
        "vm_checkpoint_pause" => {
            pause(vm)?;
            Value::Null
        }
        "vm_checkpoint_snapshot" => {
            snapshot(vm, s(a, "operation"))?;
            Value::Null
        }
        "vm_checkpoint_registers" => json!(registers(vm)?),
        "vm_checkpoint_restore" => {
            restore(vm, a.get("manifest").ok_or("Missing restore manifest")?)?;
            Value::Null
        }
        "vm_checkpoint_watch" => json!(watch(
            vm,
            s(a, "until"),
            s(a, "fail"),
            a["timeout"].as_f64().ok_or("Missing checkpoint timeout")?,
            Path::new(s(a, "log"))
        )?),
        "vm_checkpoint_export" => {
            let extras: BTreeMap<String, PathBuf> =
                serde_json::from_value(a.get("extras").cloned().unwrap_or(json!({})))?;
            export(
                vm,
                Path::new(s(a, "destination")),
                a.get("parent").and_then(Value::as_str),
                &extras,
            )?
        }
        _ => return Err(Error::new(format!("Unknown checkpoint VM operation {op}"))),
    })
}
pub fn dispatch(_root: &Path, request: &Value) -> Result<Value> {
    let a = request.get("kwargs").unwrap_or(request);
    Ok(match s(request, "op") {
        "checkpoint_digest" => json!(digest(Path::new(s(a, "path")))?),
        "checkpoint_host_path" => json!(host_path(s(a, "raw"))?),
        "checkpoint_guest_template" => json!(validate_template(&strings(a, "argv")?)?),
        "checkpoint_emulator_identity" => emulator_identity(s(a, "executable"))?,
        "checkpoint_load" => load(Path::new(s(a, "bundle")))?,
        "checkpoint_validate_bundle" => {
            validate_bundle(
                Path::new(s(a, "bundle")),
                a.get("manifest").ok_or("Missing manifest")?,
            )?;
            Value::Null
        }
        "checkpoint_prepare_guest" => json!(prepare_guest(
            _root,
            Path::new(s(a, "bundle")),
            a.get("manifest").ok_or("Missing manifest")?,
            a.get("staging").and_then(Value::as_str).map(Path::new),
            a["windows"] == true
        )?),
        _ => return Err(Error::new("Unknown checkpoint operation")),
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn closed_template_preserves_topology_and_refuses_external_inputs() {
        let args = vec![
            "qemu",
            "-S",
            "-name",
            "old",
            "-qmp",
            "old.sock",
            "-smp",
            "4",
            "-rtc",
            "base=utc,clock=vm",
            "-accel",
            "tcg,thread=multi",
            "-device",
            "virtio-blk-device,drive=scratch",
            "-drive",
            "file=C:/temp/scratch.qcow2,format=qcow2,if=none,id=scratch",
        ]
        .into_iter()
        .map(String::from)
        .collect::<Vec<_>>();
        let (t, m) = validate_template(&args).unwrap();
        assert!(!t.contains(&"old.sock".into()));
        assert_eq!(m[0]["source"], "/mnt/c/temp/scratch.qcow2");
        assert!(t.last().unwrap().contains("@media-00.qcow2"));
        for pair in [
            ["-netdev", "user,id=n"],
            ["-accel", "whpx"],
            ["-fw_cfg", "name=test,file=/hidden"],
            ["-device", "usb-storage,drive=x"],
        ] {
            let mut a = args.clone();
            a.extend(pair.map(String::from));
            assert!(validate_template(&a).is_err());
        }
    }
    #[test]
    fn corruption_refused_before_any_emulator_invocation() {
        let d = tempfile::tempdir().unwrap();
        fs::write(d.path().join("media-00.qcow2"), b"damaged").unwrap();
        let m = json!({"schema":1,"accel":"tcg","backend":"windows-host","media":[{"name":"media-00.qcow2","sha256":"wrong"}],"emulator":{"path":"/never-run"}});
        fs::write(
            d.path().join("manifest.json"),
            serde_json::to_vec(&m).unwrap(),
        )
        .unwrap();
        assert!(load(d.path()).unwrap_err().to_string().contains("damaged"));
    }
    #[test]
    fn duplicate_and_escaping_extra_names_refused() {
        let d = tempfile::tempdir().unwrap();
        let mut m = json!({"schema":1,"accel":"tcg","backend":"native","media":[],"extras":[{"name":"../escape","sha256":"x"}]});
        assert!(validate_bundle(d.path(), &m).is_err());
        m["extras"] = json!([{"name":"C:\\external","sha256":"x"}]);
        assert!(validate_bundle(d.path(), &m).is_err());
    }
}
