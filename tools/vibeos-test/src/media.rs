//! Private boot media transforms and per-run writable disk images.
use crate::backend;
use crate::{Error, Result};
use std::{
    fs::{self, File},
    io::{Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};
fn run(program: &str, args: &[String]) -> Result<()> {
    let (code, out, err) = backend::run_capture(program, args, Duration::from_secs(120))?;
    if code != 0 {
        return Err(Error::new(format!("{program} exited {code}: {out}\n{err}")));
    }
    Ok(())
}
fn run_input(program: &str, args: &[String], input: &[u8]) -> Result<()> {
    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let mut stdout = child.stdout.take().unwrap();
    let mut stderr = child.stderr.take().unwrap();
    let out = thread::spawn(move || {
        let mut b = Vec::new();
        stdout.read_to_end(&mut b).map(|_| b)
    });
    let err = thread::spawn(move || {
        let mut b = Vec::new();
        stderr.read_to_end(&mut b).map(|_| b)
    });
    child.stdin.take().unwrap().write_all(input)?;
    let end = Instant::now() + Duration::from_secs(120);
    let status = loop {
        if let Some(s) = child.try_wait()? {
            break s;
        }
        if Instant::now() >= end {
            child.kill()?;
            child.wait()?;
            return Err(Error::new(format!("{program} timed out")));
        }
        thread::sleep(Duration::from_millis(20));
    };
    let out = out
        .join()
        .map_err(|_| Error::new("stdout read thread failed"))??;
    let err = err
        .join()
        .map_err(|_| Error::new("stderr read thread failed"))??;
    if !status.success() {
        return Err(Error::new(format!(
            "{program} failed: {}\n{}",
            String::from_utf8_lossy(&out),
            String::from_utf8_lossy(&err)
        )));
    }
    Ok(())
}
fn s(p: &Path) -> String {
    p.to_string_lossy().into_owned()
}
pub fn sparse_image(path: &Path, data: &[u8]) -> Result<()> {
    if let Some(p) = path.parent() {
        fs::create_dir_all(p)?;
    }
    let mut out = File::create(path)?;
    for chunk in data.chunks(64 * 1024) {
        if chunk.iter().all(|x| *x == 0) {
            out.seek(SeekFrom::Current(chunk.len() as i64))?;
        } else {
            out.write_all(chunk)?;
        }
    }
    out.set_len(data.len() as u64)?;
    Ok(())
}
fn find_all(data: &[u8], needle: &[u8]) -> Vec<usize> {
    if needle.is_empty() {
        return Vec::new();
    }
    data.windows(needle.len())
        .enumerate()
        .filter_map(|(i, w)| (w == needle).then_some(i))
        .collect()
}
fn line_end(data: &[u8], mut at: usize) -> usize {
    while at < data.len() && !matches!(data[at], b'\r' | b'\n') {
        at += 1;
    }
    at
}
fn section<'a>(data: &'a [u8], at: usize) -> &'a [u8] {
    let Some(start) = data[..at].windows(12).rposition(|w| w == b"[FREELOADER]") else {
        return &[];
    };
    &data[start..line_end(data, at)]
}
fn fat_ini_hits(image: &Path, data: &[u8], hits: &[usize]) -> Vec<usize> {
    let Ok(temp) = tempfile::tempdir() else {
        return hits.into();
    };
    let ini = temp.path().join("freeldr.ini");
    if run(
        "mcopy",
        &[
            "-n".into(),
            "-i".into(),
            s(image),
            "::/freeldr.ini".into(),
            s(&ini),
        ],
    )
    .is_err()
    {
        return hits.into();
    }
    let Ok(bytes) = fs::read(ini) else {
        return hits.into();
    };
    let positions = find_all(data, &bytes);
    if positions.len() != 1 {
        return hits.into();
    }
    let at = positions[0];
    let inside = hits
        .iter()
        .copied()
        .filter(|h| *h >= at && *h < at + bytes.len())
        .collect::<Vec<_>>();
    if inside.is_empty() {
        hits.into()
    } else {
        inside
    }
}
pub fn smp_selected_iso(iso: &Path, work: &Path) -> Result<PathBuf> {
    let mut data = fs::read(iso)?;
    let mut hits = find_all(&data, b"DefaultOS=")
        .into_iter()
        .filter(|i| *i == 0 || data[*i - 1] == b'\n')
        .collect::<Vec<_>>();
    if hits.len() > 1 {
        hits = fat_ini_hits(iso, &data, &hits);
    }
    let copies = hits.len() > 1
        && !section(&data, hits[0]).is_empty()
        && hits
            .iter()
            .all(|h| section(&data, *h) == section(&data, hits[0]));
    if hits.len() != 1 && !copies {
        return Err(Error::new(format!("{}: expected exactly one line-initial DefaultOS= in the image, found {} — rebuild with build --smp-kernel",iso.display(),hits.len())));
    }
    let start = hits[0];
    let end = line_end(&data, start);
    let old = &data[start..end];
    let entry = &old[10..];
    if matches!(entry, b"LiveCD_SMP" | b"ReactOS_Arm64_MP") {
        return Ok(iso.into());
    }
    let mp: &[u8] = if entry == b"ReactOS_Arm64_UP" {
        b"ReactOS_Arm64_MP"
    } else {
        b"LiveCD_SMP"
    };
    let mut section_name = vec![b'['];
    section_name.extend_from_slice(mp);
    section_name.push(b']');
    if find_all(&data, &section_name).is_empty() {
        return Err(Error::new(format!(
            "{}: no [{}] section — media cannot boot the MP pair; rebuild it",
            iso.display(),
            String::from_utf8_lossy(mp)
        )));
    }
    let mut new = b"DefaultOS=".to_vec();
    new.extend_from_slice(mp);
    if old.len() < new.len() {
        return Err(
            "default boot entry is shorter than the MP replacement; cannot patch in place".into(),
        );
    }
    let pad = old.len() - new.len();
    if pad == 1 {
        return Err("1-byte remainder cannot terminate the MP default boot line".into());
    }
    if pad > 0 {
        new.extend_from_slice(b"\r\n");
        new.extend(std::iter::repeat_n(b';', pad - 2));
    }
    for at in hits {
        data[at..at + new.len()].copy_from_slice(&new);
    }
    let out = work.join(iso.file_name().ok_or("boot media lacks basename")?);
    sparse_image(&out, &data)?;
    Ok(out)
}
pub fn stortest_selected_iso(iso: &Path, work: &Path) -> Result<PathBuf> {
    let mut data = fs::read(iso)?;
    let mut patched = 0;
    for at in find_all(&data, b"Options=") {
        if at > 0 && data[at - 1] != b'\n' {
            continue;
        }
        let end = line_end(&data, at);
        let line = &data[at..end];
        if find_all(line, b"/STORTEST").len() > 0 {
            patched += 1;
        } else if let Some(offset) = find_all(line, b"/BAUDRATE=115200").first() {
            let mut replacement = b"/STORTEST".to_vec();
            replacement.resize(16, b' ');
            let pos = at + offset;
            data[pos..pos + 16].copy_from_slice(&replacement);
            patched += 1;
        }
    }
    if patched == 0 {
        return Err(Error::new(format!("{}: no line-initial Options= line holds /BAUDRATE=115200, so STORTEST cannot be added in place",iso.display())));
    }
    let out = work.join(format!(
        "stortest-{}",
        iso.file_name()
            .ok_or("boot media has no basename")?
            .to_string_lossy()
    ));
    sparse_image(&out, &data)?;
    Ok(out)
}
pub fn testkernel_boot_img(src: &Path, work: &Path) -> Result<PathBuf> {
    use std::os::unix::fs::MetadataExt;
    let st = fs::metadata(src)?;
    let ns = (st.mtime() as i128) * 1_000_000_000 + i128::from(st.mtime_nsec());
    let cached = src
        .parent()
        .ok_or("boot image has no parent")?
        .join(format!("boot-test.{ns:x}-{:x}.img", st.len()));
    if !cached.exists() {
        fs::create_dir_all(work)?;
        let staging = work.join("boot-test.staging.img");
        fs::copy(src, &staging)?;
        let ini = work.join("freeldr.ini");
        run(
            "mcopy",
            &[
                "-o".into(),
                "-i".into(),
                s(&staging),
                "::/freeldr.ini".into(),
                s(&ini),
            ],
        )?;
        let text = fs::read_to_string(&ini)?;
        if !text.contains("/TESTKERNEL") {
            fs::write(
                &ini,
                text.replace("/SOS /RDPATH=", "/SOS /TESTKERNEL /RDPATH="),
            )?;
        }
        run(
            "mcopy",
            &[
                "-o".into(),
                "-i".into(),
                s(&staging),
                s(&ini),
                "::/freeldr.ini".into(),
            ],
        )?;
        fs::rename(staging, &cached)?;
    }
    for e in fs::read_dir(src.parent().unwrap())? {
        let p = e?.path();
        let n = p.file_name().unwrap_or_default().to_string_lossy();
        if n.starts_with("boot-test.") && n.ends_with(".img") && p != cached {
            fs::remove_file(p)?;
        }
    }
    Ok(cached)
}
pub fn qcow2_from_raw(src: &Path, dst: &Path) -> Result<()> {
    run(
        "qemu-img",
        &[
            "convert".into(),
            "-f".into(),
            "raw".into(),
            "-O".into(),
            "qcow2".into(),
            s(src),
            s(dst),
        ],
    )
}
pub fn empty_qcow2(dst: &Path) -> Result<()> {
    run(
        "qemu-img",
        &[
            "create".into(),
            "-f".into(),
            "qcow2".into(),
            s(dst),
            "64M".into(),
        ],
    )
}
pub fn make_temp_disk(out: &Path) -> Result<()> {
    let raw = out.with_extension("img");
    File::create(&raw)?.set_len(64 * 1024 * 1024)?;
    run_input(
        "sfdisk",
        &["--quiet".into(), s(&raw)],
        b"start=2048, type=6\n",
    )?;
    run(
        "mkfs.fat",
        &[
            "-F".into(),
            "16".into(),
            "-n".into(),
            "VIBEOSTEMP".into(),
            "--offset".into(),
            "2048".into(),
            s(&raw),
            ((64 * 2048 - 2048) / 2).to_string(),
        ],
    )?;
    run(
        "mmd",
        &[
            "-i".into(),
            format!("{}@@1048576", raw.display()),
            "::/TEMP".into(),
        ],
    )?;
    qcow2_from_raw(&raw, out)?;
    fs::remove_file(raw)?;
    Ok(())
}
pub fn make_usb_stick_image(
    out: &Path,
    fat: u32,
    mib: u64,
    label: &str,
    files: &[(String, Vec<u8>)],
) -> Result<()> {
    if !matches!(fat, 16 | 32) || mib < 2 {
        return Err("USB disk requires FAT16/FAT32 and at least 2 MiB".into());
    }
    let temporary = tempfile::tempdir()?;
    let raw = out.with_extension("tmp");
    File::create(&raw)?.set_len(
        mib.checked_mul(1024 * 1024)
            .ok_or("USB disk size overflow")?,
    )?;
    run_input(
        "sfdisk",
        &["--no-reread".into(), "--no-tell-kernel".into(), s(&raw)],
        format!(
            "label: dos\nstart=2048, type={}\n",
            if fat == 32 { "0c" } else { "06" }
        )
        .as_bytes(),
    )?;
    let mut args = vec!["-F".into(), fat.to_string()];
    if fat == 32 && mib < 512 {
        args.extend(["-s".into(), "1".into()]);
    }
    args.extend([
        "-n".into(),
        label.into(),
        "--offset".into(),
        "2048".into(),
        s(&raw),
        ((mib * 1024 * 1024 - 1024 * 1024) / 1024).to_string(),
    ]);
    run("mkfs.fat", &args)?;
    for (name, bytes) in files {
        if Path::new(name).file_name().and_then(|s| s.to_str()) != Some(name) {
            return Err("USB payload names must be basenames".into());
        }
        let path = temporary.path().join(name);
        fs::write(&path, bytes)?;
        run(
            "mcopy",
            &[
                "-i".into(),
                format!("{}@@1048576", raw.display()),
                s(&path),
                format!("::/{name}"),
            ],
        )?;
    }
    qcow2_from_raw(&raw, out)?;
    fs::remove_file(raw)?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mp_patch_keeps_length_and_original() {
        let t = tempfile::tempdir().unwrap();
        let image = t.path().join("input.iso");
        let data = b"[FREELOADER]\r\nDefaultOS=LiveCD_Debug\r\n[LiveCD_SMP]\r\n";
        fs::write(&image, data).unwrap();
        let out = smp_selected_iso(&image, &t.path().join("out")).unwrap();
        let new = fs::read(out).unwrap();
        assert_eq!(data.len(), new.len());
        assert!(new.windows(20).any(|s| s == b"DefaultOS=LiveCD_SMP"));
        assert_eq!(fs::read(image).unwrap(), data);
    }
    #[test]
    fn ambiguous_media_is_refused() {
        let t = tempfile::tempdir().unwrap();
        let p = t.path().join("boot.img");
        fs::write(&p, b"DefaultOS=LiveCD_Debug\nDefaultOS=Other\n[LiveCD_SMP]").unwrap();
        assert!(smp_selected_iso(&p, t.path()).is_err());
    }
}
