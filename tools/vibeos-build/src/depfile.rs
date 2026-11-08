//! Retarget rules without dropping prerequisites; explicit snapshots require equal bytes.
use crate::{Error, Result};
use std::{fs, path::Path};
pub fn escaped_output(path: &str) -> Result<String> {
    if path.chars().any(|c| "\r\n\t\\:#$".contains(c)) {
        return Err(Error::new(format!(
            "unsupported depfile output spelling: {path:?}"
        )));
    }
    Ok(path.replace(' ', "\\ "))
}
pub fn retarget(text: &str, cargo_output: &str, copied_output: &str) -> Result<String> {
    let prefix = format!("{}:", escaped_output(cargo_output)?);
    let Some(body) = text
        .strip_prefix(&prefix)
        .filter(|s| s.starts_with([' ', '\n', '\r']))
    else {
        return Err("Cargo depfile does not start with the expected output rule".into());
    };
    let target = escaped_output(copied_output)?;
    Ok(format!("{target}:{}", drop_self(body, &target)))
}

/// The first rule's prerequisites without the rule's own target: the one prerequisite
/// retargeting drops. A cell whose output sits where its build script watches generated
/// files (mshtml.tlb: reactos-link's rc step watches every `.tlb` in the module's build
/// dir, its own output among them) gets itself as a prerequisite, and every ninja run
/// after the first stops on `dependency cycle: x -> x`.
fn drop_self(body: &str, target: &str) -> String {
    let bytes = body.as_bytes();
    let mut out = String::with_capacity(body.len());
    let mut at = 0;
    while at < bytes.len() {
        if bytes[at] == b'\n' {
            out.push_str(&body[at..]);
            return out;
        }
        if bytes[at] == b'\\' && bytes.get(at + 1) == Some(&b'\n') {
            out.push_str("\\\n");
            at += 2;
            continue;
        }
        if bytes[at..].starts_with(b"\\\r\n") {
            out.push_str("\\\r\n");
            at += 3;
            continue;
        }
        if bytes[at].is_ascii_whitespace() {
            out.push(bytes[at] as char);
            at += 1;
            continue;
        }
        let start = at;
        while at < bytes.len() && !bytes[at].is_ascii_whitespace() {
            at += if bytes[at] == b'\\' && at + 1 < bytes.len() { 2 } else { 1 };
        }
        let token = &body[start..at];
        if token != target {
            out.push_str(token);
        }
    }
    out
}

/// Replace an explicitly declared prerequisite with a byte-identical snapshot.
/// Cargo retains its original depfile; only the copied PE's first rule changes.
pub fn snapshot_dependency(text: &str, source: &Path, snapshot: &Path) -> Result<String> {
    if source == snapshot {
        return Err("dependency snapshot must be distinct from its source".into());
    }
    let from = escaped_output(source.to_str().ok_or("non-UTF8 dependency source")?)?;
    let to = escaped_output(snapshot.to_str().ok_or("non-UTF8 dependency snapshot")?)?;
    verify_snapshot(source, snapshot)?;
    let colon = text.find(':').ok_or("missing dependency rule")?;
    let mut out = text[..=colon].to_string();
    let bytes = text.as_bytes();
    let mut at = colon + 1;
    while at < bytes.len() {
        if bytes[at] == b'\n' {
            out.push_str(&text[at..]);
            break;
        }
        if bytes[at].is_ascii_whitespace() {
            out.push(bytes[at] as char);
            at += 1;
            continue;
        }
        if bytes[at] == b'\\' && bytes.get(at + 1) == Some(&b'\n') {
            out.push_str("\\\n");
            at += 2;
            continue;
        }
        if bytes[at..].starts_with(b"\\\r\n") {
            out.push_str("\\\r\n");
            at += 3;
            continue;
        }
        let start = at;
        while at < bytes.len() && !bytes[at].is_ascii_whitespace() {
            if bytes[at] == b'\\' && at + 1 < bytes.len() {
                at += 2;
            } else {
                at += 1;
            }
        }
        let token = &text[start..at];
        out.push_str(if token == from { &to } else { token });
    }
    Ok(out)
}

fn verify_snapshot(source: &Path, snapshot: &Path) -> Result<()> {
    if fs::read(source)? != fs::read(snapshot)? {
        return Err(Error::new(format!(
            "dependency snapshot {} differs from {}; refresh it through Ninja before linking",
            snapshot.display(),
            source.display()
        )));
    }
    Ok(())
}

/// Refuse to stamp or boot a tree reconfigured since its snapshot was refreshed.
/// Trees predating the snapshot keep their original raw-database dependency.
pub fn check_compile_snapshot(build: &Path) -> Result<()> {
    let snapshot = build.join("reactos-link-compile-commands.json");
    if snapshot.try_exists()? {
        verify_snapshot(&build.join("compile_commands.json"), &snapshot)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn retarget_drops_only_the_rules_own_target() {
        // mshtml.tlb's cargo depfile lists the cell's own output: ninja's second run
        // stopped on `dependency cycle: dll/win32/mshtml.tlb/mshtml.tlb -> itself`.
        let out = "/b/dll/x.tlb/x.tlb";
        let input = format!("/t/x_tlb.exe: \\\n  {out} \\\n  /b/dll/x.tlb/x_tlb.tlb {out}.spec\n{out}:\n");
        let got = retarget(&input, "/t/x_tlb.exe", out).unwrap();
        assert_eq!(
            got,
            format!("{out}: \\\n   \\\n  /b/dll/x.tlb/x_tlb.tlb {out}.spec\n{out}:\n")
        );
    }
    #[test]
    fn snapshot_is_checked_and_only_exact_prerequisites_change() {
        let root = std::env::temp_dir().join(format!("dep-snapshot-é-{}", std::process::id()));
        fs::create_dir(&root).unwrap();
        let source = root.join("raw commands.json");
        let snapshot = root.join("stable commands.json");
        fs::write(&source, b"exact bytes\n").unwrap();
        fs::write(&snapshot, b"exact bytes\n").unwrap();
        let raw = escaped_output(source.to_str().unwrap()).unwrap();
        let stable = escaped_output(snapshot.to_str().unwrap()).unwrap();
        let input = format!("app: a.rs {raw}.other \\\n {raw} tail.rs\n{raw}:\n# {raw}\n");
        let expected = format!("app: a.rs {raw}.other \\\n {stable} tail.rs\n{raw}:\n# {raw}\n");
        assert_eq!(
            snapshot_dependency(&input, &source, &snapshot).unwrap(),
            expected
        );
        assert_eq!(
            snapshot_dependency(&input.replace("\n", "\r\n"), &source, &snapshot).unwrap(),
            expected.replace("\n", "\r\n")
        );
        fs::write(&snapshot, b"different bytes\n").unwrap();
        assert!(snapshot_dependency(&input, &source, &snapshot)
            .unwrap_err()
            .message
            .contains("differs"));
        fs::remove_file(&snapshot).unwrap();
        assert!(snapshot_dependency(&input, &source, &snapshot).is_err());
        assert!(snapshot_dependency(&input, &source, &source).is_err());
        fs::remove_dir_all(root).unwrap();
    }
}
