//! Native Raspberry Pi payload staging. The caller supplies the authoritative
//! arm64 build config; freshness and the bench lease are enforced here.
use crate::{Error, Result};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

fn captured(c: &mut Command) -> Result<vibeos_build::process::Output> {
    vibeos_build::artifact_executor::supervised_output(c, Some(std::time::Duration::from_secs(180)))
}
fn run(c: &mut Command) -> Result<()> {
    let o = captured(c)?;
    if o.code != 0 {
        return Err(Error::new(format!(
            "command {c:?} failed ({}): {}",
            o.code, o.stderr
        )));
    }
    if !o.stdout.is_empty() {
        eprint!("{}", o.stdout);
    }
    if !o.stderr.is_empty() {
        eprint!("{}", o.stderr);
    }
    Ok(())
}
fn output(c: &mut Command) -> Result<String> {
    let o = captured(c)?;
    if o.code != 0 {
        return Err(Error::new(format!(
            "command {c:?} failed ({}): {}",
            o.code, o.stderr
        )));
    }
    Ok(o.stdout.trim().to_owned())
}
fn env(a: &Value, key: &str, default: &str) -> String {
    a.get("env")
        .and_then(|v| v.get(key))
        .and_then(Value::as_str)
        .map(str::to_owned)
        .or_else(|| std::env::var(key).ok())
        .unwrap_or_else(|| default.to_owned())
}
fn check(root: &Path, a: &Value) -> Result<()> {
    let cfg = a
        .get("cfg")
        .filter(|v| v.is_object())
        .ok_or("missing authoritative arm64 provenance config")?;
    let environment = a
        .get("env")
        .cloned()
        .unwrap_or_else(|| json!(std::env::vars().collect::<std::collections::BTreeMap<_, _>>()));
    let verdict = vibeos_build::provenance::dispatch(root, &json!({"op":"provenance_check_staleness_inner","kwargs":{"repo":root,"arch":"arm64","cfg":cfg,"env":environment}})).map_err(|e| Error::new(e.to_string()))?;
    let stale = !verdict["fresh"].as_bool().unwrap_or(false)
        || verdict["media_missing"].as_bool().unwrap_or(true);
    if stale {
        if a["allow_stale"] != true {
            return Err(Error::new(format!(
                "stale arm64 build: {}",
                verdict["reasons"]
            )));
        }
        eprintln!(
            "WARNING: explicit --allow-stale staging override: {}",
            verdict["reasons"]
        );
    }
    Ok(())
}
fn lease(root: &Path, a: &Value, board: &str) -> Result<()> {
    let mut envs: std::collections::BTreeMap<String, String> = match a.get("env") {
        Some(v) => serde_json::from_value(v.clone())?,
        None => std::env::vars().collect(),
    };
    envs.insert("VIBEOS_BENCH_BOARD".into(), board.into());
    let owner =
        vibeos_bench::lease::owner_of(root, &envs).map_err(|e| Error::new(e.to_string()))?;
    let minutes = envs
        .get("VIBEOS_PI4_LEASE_MINUTES")
        .map(|v| {
            v.parse::<f64>()
                .map_err(|_| Error::new("invalid lease minutes"))
        })
        .transpose()?
        .unwrap_or(vibeos_bench::lease::DEFAULT_MINUTES);
    let (ok, held) = vibeos_bench::lease::take(&owner, minutes, "", &envs, std::process::id())
        .map_err(|e| Error::new(e.to_string()))?;
    if !ok {
        return Err(Error::classified(
            "LEASE_REFUSED",
            format!("{board} bench lease held: {held}"),
        ));
    }
    Ok(())
}
fn required(path: &Path, prefix: &str) -> Result<()> {
    if !path.is_file() {
        return Err(Error::new(format!(
            "{prefix}: missing {} - run ./vibeos-cli build --arch arm64",
            path.display()
        )));
    }
    Ok(())
}
fn copy(src: &Path, dst: &Path) -> Result<()> {
    if let Some(p) = dst.parent() {
        fs::create_dir_all(p)?;
    }
    fs::copy(src, dst)?;
    Ok(())
}
fn digest(path: &Path) -> Result<String> {
    let mut f = fs::File::open(path)?;
    let mut h = Sha256::new();
    let mut buf = [0; 1024 * 1024];
    loop {
        let n = f.read(&mut buf)?;
        if n == 0 {
            break;
        }
        h.update(&buf[..n]);
    }
    Ok(format!("{:x}", h.finalize()))
}
fn git(root: &Path, args: &[&str]) -> Result<String> {
    output(Command::new("git").arg("-C").arg(root).args(args))
}
fn timestamp() -> Result<String> {
    output(Command::new("date").arg("+%Y-%m-%dT%H:%M:%S%z"))
}
fn identity(root: &Path) -> Result<(String, bool, String)> {
    let dirty = !git(root, &["status", "--porcelain", "--untracked-files=no"])?.is_empty();
    let diff = output(
        Command::new("git")
            .arg("-C")
            .arg(root)
            .args(["diff", "--binary", "HEAD", "--"]),
    )?;
    Ok((
        git(root, &["rev-parse", "HEAD"])?,
        dirty,
        format!("{:x}", Sha256::digest(diff.as_bytes())),
    ))
}
fn source_unchanged(root: &Path, before: &(String, bool, String)) -> Result<()> {
    if &identity(root)? != before {
        return Err(Error::new("source identity changed while staging"));
    }
    Ok(())
}
fn pi4_ini(data: &str, kernel: &str) -> Result<String> {
    match kernel {
        "mp" => Ok(data.to_owned()),
        "up" => {
            let mut changed = false;
            let mut result = String::new();
            for line in data.split_inclusive('\n') {
                let (body, ending) = if let Some(x) = line.strip_suffix("\r\n") {
                    (x, "\r\n")
                } else if let Some(x) = line.strip_suffix('\n') {
                    (x, "\n")
                } else {
                    (line, "")
                };
                if body == "DefaultOS=ReactOS_Arm64_MP" {
                    result.push_str("DefaultOS=ReactOS_Arm64_UP");
                    changed = true;
                } else {
                    result.push_str(body);
                }
                result.push_str(ending);
            }
            if !changed {
                return Err(Error::new(
                    "pi4-bench: could not flip DefaultOS to the UP entry",
                ));
            }
            Ok(result)
        }
        _ => Err(Error::new("pi4-bench: PI4_KERNEL must be up or mp")),
    }
}
pub fn pi5_ini(data: &str, extra: &str) -> Result<String> {
    if extra.contains(['\\', '&', '|']) {
        return Err(Error::new(
            "pi5-stage: VIBEOS_LOADER_OPTIONS may not contain \\ & or |",
        ));
    }
    let mut result = String::new();
    for line in data.split_inclusive('\n') {
        let (body, ending) = if let Some(x) = line.strip_suffix("\r\n") {
            (x, "\r\n")
        } else if let Some(x) = line.strip_suffix('\n') {
            (x, "\n")
        } else {
            (line, "")
        };
        if body == "TimeOut=-1" {
            result.push_str("TimeOut=5");
        } else {
            result.push_str(body);
            if body.starts_with("Options=") && !extra.is_empty() {
                result.push(' ');
                result.push_str(extra);
            }
        }
        result.push_str(ending);
    }
    Ok(result)
}
fn pi4(root: &Path, a: &Value) -> Result<Value> {
    check(root, a)?;
    let efi = root.join("build-arm64/boot/freeldr/freeldr/uefildr.efi");
    let img = root.join("qemu/ramdisk.img");
    let boot = root.join("qemu/boot.img");
    for p in [&efi, &img, &boot] {
        required(p, "pi4-bench")?;
    }
    let kernel = a["kernel"]
        .as_str()
        .map(str::to_owned)
        .unwrap_or_else(|| env(a, "PI4_KERNEL", "mp"));
    let ini = pi4_ini(
        &fs::read_to_string(root.join("tools/pi4/freeldr.ini"))?,
        &kernel,
    )?;
    let id = identity(root)?;
    lease(root, a, "pi4")?;
    let bench = PathBuf::from(
        a["bench"]
            .as_str()
            .map(str::to_owned)
            .unwrap_or_else(|| env(a, "PI4_BENCH", "/mnt/d/home/vibeos-rpi4-setup")),
    );
    let payload = bench.join("payload");
    fs::create_dir_all(&payload)?;
    copy(&efi, &payload.join("EFI/BOOT/BOOTAA64.EFI"))?;
    fs::write(payload.join("freeldr.ini"), ini)?;
    copy(&img, &payload.join("ramdisk.img"))?;
    let temp = payload.join("boot.img.tmp");
    copy(&boot, &temp)?;
    run(Command::new("mcopy")
        .args(["-o", "-i"])
        .arg(&temp)
        .arg(payload.join("freeldr.ini"))
        .arg("::/freeldr.ini"))?;
    fs::rename(&temp, payload.join("boot.img"))?;
    let ip = env(a, "PI4_HOST_IP", "192.168.1.183");
    let port = env(a, "PI4_HTTP_PORT", "8069");
    fs::write(payload.join("autoexec.ipxe"),format!("#!ipxe\necho [ipxe] sanboot http://{ip}:{port}/boot.img\ndhcp\nsanboot http://{ip}:{port}/boot.img\n"))?;
    let ipxe = bench.join("ipxe/ipxe.efi");
    if ipxe.is_file() {
        copy(&ipxe, &payload.join("ipxe.efi"))?;
    } else {
        eprintln!(
            "pi4-bench: no {} - --sanboot needs https://boot.ipxe.org/arm64-efi/ipxe.efi there",
            ipxe.display()
        );
    }
    let mut files = Vec::new();
    for path in [
        "EFI/BOOT/BOOTAA64.EFI",
        "freeldr.ini",
        "ramdisk.img",
        "boot.img",
        "autoexec.ipxe",
    ] {
        let p = payload.join(path);
        files.push(json!({"path":path,"bytes":p.metadata()?.len(),"sha256":digest(&p)?}));
    }
    source_unchanged(root, &id)?;
    let manifest = json!({"status":"STAGED","kernel":kernel,"staged":timestamp()?,"source_head":id.0,"source_dirty":id.1,"allow_stale":a["allow_stale"] == true,"files":files});
    fs::write(
        payload.join("manifest.json"),
        serde_json::to_vec_pretty(&manifest)?,
    )?;
    Ok(manifest)
}
const INSTALL: &str = r#"set -e
cd /tmp/vibeos-stage && sha256sum -c --quiet SHA256SUMS
cd /boot/firmware
# Copy the old root stager to linux/ without deleting its only working copy.
if [ -f kernel_2712.img ]; then
    sudo mkdir -p linux
    for f in kernel_2712.img initramfs_2712 bcm2712-rpi-5-b.dtb cmdline.txt; do
        [ -s "$f" ] || { echo "$f missing: not installing config.txt" >&2; exit 1; }
        sudo cp -p "$f" "linux/$f"
    done
fi
for f in kernel_2712.img initramfs_2712 bcm2712-rpi-5-b.dtb cmdline.txt; do
    [ -s "linux/$f" ] || { echo "linux/$f missing: not installing config.txt" >&2; exit 1; }
done
cd /tmp/vibeos-stage
for f in EFI/BOOT/BOOTAA64.EFI freeldr.ini ramdisk.img tryboot.txt bcm2712-rpi-5-b.dtb config.txt; do
    sudo install -D -m 0755 "$f" "/boot/firmware/$f"
done
sync
cd /boot/firmware && sha256sum -c --quiet /tmp/vibeos-stage/SHA256SUMS"#;
fn ssh(bench: &Path, target: &str) -> Command {
    let mut c = Command::new("ssh");
    c.args([
        "-o",
        "BatchMode=yes",
        "-o",
        "ConnectTimeout=5",
        "-o",
        "StrictHostKeyChecking=accept-new",
        "-o",
    ]);
    c.arg(format!(
        "UserKnownHostsFile={}",
        bench.join("known_hosts").display()
    ))
    .arg(target);
    c
}
fn pi5(root: &Path, a: &Value) -> Result<Value> {
    check(root, a)?;
    let efi = root.join("build-arm64/boot/freeldr/freeldr/uefildr.efi");
    let img = root.join("qemu/ramdisk.img");
    for p in [&efi, &img] {
        required(p, "pi5-stage")?;
    }
    let extra = a["options"]
        .as_str()
        .map(str::to_owned)
        .unwrap_or_else(|| env(a, "VIBEOS_LOADER_OPTIONS", ""));
    let ini = pi5_ini(
        &fs::read_to_string(root.join("tools/pi4/freeldr.ini"))?,
        &extra,
    )?;
    if extra
        .split_whitespace()
        .any(|x| x.eq_ignore_ascii_case("/STORTEST"))
        && !ini.lines().any(|line| {
            line.starts_with("Options=")
                && line
                    .split_whitespace()
                    .any(|x| x.eq_ignore_ascii_case("/STORTEST"))
        })
    {
        return Err(Error::new(
            "pi5-stage: /STORTEST was not added to a loader entry",
        ));
    }
    let bench = PathBuf::from(
        a["bench"]
            .as_str()
            .map(str::to_owned)
            .unwrap_or_else(|| env(a, "PI5_BENCH", "/mnt/d/home/vibeos-rpi5-setup")),
    );
    let zip = bench.join("cache/rpi5-uefi-v0.3.zip");
    if !zip.is_file() {
        return Err(Error::new(format!(
            "pi5-stage: missing {} - run tools/pi5/make-card.sh once",
            zip.display()
        )));
    }
    let id = identity(root)?;
    lease(root, a, "pi5")?;
    // The remote identity is checked before creating its staging directory or installing files.
    fs::create_dir_all(&bench)?;
    let user = env(a, "PI5_SSH_USER", "bench");
    let host = env(a, "PI5_HOST", "vibeos-pi5");
    let target = format!("{user}@{host}");
    let model=output(ssh(&bench,&target).arg("tr -d '\\000' < /proc/device-tree/model")).map_err(|e| Error::new(format!("pi5-stage: no SSH to {target} - is the board up in its Linux stager? (vibeos-cli pi5 doctor): {e}")))?;
    if !model.starts_with("Raspberry Pi 5") {
        return Err(Error::new(format!(
            "pi5-stage: refusing unexpected device model {model:?}"
        )));
    }
    let work = tempfile::tempdir()?;
    let dir = work.path();
    copy(&efi, &dir.join("EFI/BOOT/BOOTAA64.EFI"))?;
    copy(&img, &dir.join("ramdisk.img"))?;
    fs::write(dir.join("freeldr.ini"), ini)?;
    for f in ["config.txt", "tryboot.txt"] {
        copy(&root.join("tools/pi5/card").join(f), &dir.join(f))?;
    }
    let dtb = Command::new("unzip")
        .arg("-p")
        .arg(zip)
        .arg("bcm2712-rpi-5-b.dtb")
        .output()?;
    if !dtb.status.success() || dtb.stdout.is_empty() {
        return Err(Error::new("pi5-stage: could not extract UEFI device tree"));
    }
    fs::write(dir.join("bcm2712-rpi-5-b.dtb"), dtb.stdout)?;
    let names = [
        "EFI/BOOT/BOOTAA64.EFI",
        "freeldr.ini",
        "ramdisk.img",
        "config.txt",
        "tryboot.txt",
        "bcm2712-rpi-5-b.dtb",
    ];
    let mut sums = String::new();
    let mut files = Vec::new();
    for p in names {
        let hash = digest(&dir.join(p))?;
        sums.push_str(&format!("{hash}  {p}\n"));
        files.push(json!({"sha256":hash,"path":p}));
    }
    fs::write(dir.join("SHA256SUMS"), sums)?;
    source_unchanged(root, &id)?;
    eprintln!("pi5-stage: copying {} files to {target}", names.len());
    run(
        ssh(&bench, &target).arg("rm -rf /tmp/vibeos-stage && mkdir -p /tmp/vibeos-stage/EFI/BOOT")
    )?;
    let archive = dir.join("payload.tar");
    run(Command::new("tar")
        .current_dir(dir)
        .arg("cf")
        .arg(&archive)
        .args(names)
        .arg("SHA256SUMS"))?;
    let archive_file = fs::File::open(&archive)?;
    run(ssh(&bench, &target)
        .arg("tar xf - -C /tmp/vibeos-stage")
        .stdin(Stdio::from(archive_file)))?;
    run(ssh(&bench, &target).arg(INSTALL))?;
    source_unchanged(root, &id)?;
    let short = git(root, &["rev-parse", "--short=12", "HEAD"])?;
    let manifest = json!({"commit":short,"dirty":id.1,"allow_stale":a["allow_stale"] == true,"staged":timestamp()?,"files":files});
    fs::write(
        bench.join("payload.json"),
        serde_json::to_vec_pretty(&manifest)?,
    )?;
    eprintln!(
        "pi5-stage: staged {}{}; card verified",
        short,
        if id.1 { " (dirty tree)" } else { "" }
    );
    if !a["no_reboot"].as_bool().unwrap_or(false) {
        eprintln!("pi5-stage: rebooting into UEFI + VibeOS (tryboot)");
        run(
            ssh(&bench, &target).arg("sudo systemd-run --on-active=1 /usr/sbin/reboot '0 tryboot'")
        )?;
    }
    Ok(manifest)
}
pub fn dispatch(root: &Path, request: &Value) -> Result<Value> {
    let a = request.get("kwargs").unwrap_or(request);
    match request["op"].as_str().unwrap_or("") {
        "bench_stage_pi4" => pi4(root, a),
        "bench_stage_pi5" => pi5(root, a),
        "bench_stage_pi5_ini" => Ok(json!(pi5_ini(
            &fs::read_to_string(root.join("tools/pi4/freeldr.ini"))?,
            &a["options"]
                .as_str()
                .map(str::to_owned)
                .unwrap_or_else(|| env(a, "VIBEOS_LOADER_OPTIONS", ""))
        )?)),
        _ => Err(Error::new("unknown bench stage operation")),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn options_and_crlf() {
        let ini = "TimeOut=-1\r\nOptions=/BAUDRATE=115200\r\n";
        assert_eq!(
            pi5_ini(ini, "/STORTEST").unwrap(),
            "TimeOut=5\r\nOptions=/BAUDRATE=115200 /STORTEST\r\n"
        );
        assert!(pi5_ini(ini, "bad&option").is_err());
    }
    #[test]
    fn pi4_kernel_and_crlf() {
        assert_eq!(
            pi4_ini("DefaultOS=ReactOS_Arm64_MP\r\n", "up").unwrap(),
            "DefaultOS=ReactOS_Arm64_UP\r\n"
        );
        assert!(pi4_ini("bad\r\n", "up").is_err());
        assert!(pi4_ini("x", "other").is_err());
    }
    #[test]
    fn no_cfg_fails_before_mutation() {
        let dir = tempfile::tempdir().unwrap();
        assert!(dispatch(dir.path(), &json!({"op":"bench_stage_pi4","kwargs":{}})).is_err());
        assert!(fs::read_dir(dir.path()).unwrap().next().is_none());
    }
}
