//! Stable command identities and monotonic peak-memory observations.
//!
//! The profile filename is derived only from a validated hexadecimal key. The
//! lock and temporary directories use that same key as their only filename.

use std::{
    env,
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

const FNV_OFFSET_A: u64 = 0xcbf29ce484222325;
const FNV_OFFSET_B: u64 = 0x84222325cbf29ce4;
const FNV_PRIME: u64 = 0x100000001b3;

/// Build a deterministic profile key for a command.
///
/// The complete argument vector is retained, so target triples, flags,
/// compiler inputs and outputs remain part of the identity. Repository roots
/// and the canonical `/tmp/vibeos` root are normalized to a common token so
/// equivalent invocations can share history across worktrees.
pub fn identity(kind: &str, args: &[std::ffi::OsString]) -> String {
    let repo = env::var_os("VIBEOS_JOB_REPO").map(PathBuf::from);
    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let normalized_cwd = normalize_path(&cwd.to_string_lossy(), repo.as_deref());

    let mut a = FNV_OFFSET_A;
    let mut b = FNV_OFFSET_B;
    feed(&mut a, &mut b, kind.as_bytes());
    feed(&mut a, &mut b, normalized_cwd.as_bytes());
    for arg in args {
        let normalized = normalize_path(&arg.to_string_lossy(), repo.as_deref());
        feed(&mut a, &mut b, normalized.as_bytes());
    }
    format!("{a:016x}{b:016x}")
}

fn feed(a: &mut u64, b: &mut u64, bytes: &[u8]) {
    let len = (bytes.len() as u64).to_le_bytes();
    for byte in len.iter().chain(bytes) {
        *a ^= u64::from(*byte);
        *a = a.wrapping_mul(FNV_PRIME);
        *b ^= u64::from(*byte);
        *b = b.wrapping_mul(FNV_PRIME);
    }
}

fn normalize_path(value: &str, repo: Option<&Path>) -> String {
    let mut out = value.to_owned();
    if let Some(repo) = repo {
        let repo = repo.to_string_lossy();
        if !repo.is_empty() {
            out = replace_prefix_occurrences(&out, &repo, "$REPO");
        }
    }
    replace_prefix_occurrences(&out, "/tmp/vibeos", "$REPO")
}

fn replace_prefix_occurrences(value: &str, prefix: &str, replacement: &str) -> String {
    if prefix.is_empty() {
        return value.to_owned();
    }
    let mut result = String::with_capacity(value.len());
    let mut rest = value;
    while let Some(index) = rest.find(prefix) {
        let end = index + prefix.len();
        let boundary = rest[end..]
            .chars()
            .next()
            .is_none_or(|c| c == '/' || c == '\\' || c == ':' || c == '=');
        result.push_str(&rest[..index]);
        if boundary {
            result.push_str(replacement);
        } else {
            result.push_str(prefix);
        }
        rest = &rest[end..];
    }
    result.push_str(rest);
    result
}

fn validate_key(key: &str) -> io::Result<()> {
    if key.len() != 32 || !key.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "profile key must be 32 hexadecimal characters",
        ));
    }
    Ok(())
}

fn lock_for(root: &Path, key: &str) -> io::Result<File> {
    validate_key(key)?;
    fs::create_dir_all(root.join(".locks"))?;
    let file = super::open(&root.join(".locks").join(key))?;
    super::lock(&file, false)?;
    Ok(file)
}

fn parse_peak(text: &str) -> io::Result<usize> {
    let value = text.strip_suffix('\n').unwrap_or(text);
    if value.is_empty() || !value.bytes().all(|b| b.is_ascii_digit()) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "corrupt memory profile value",
        ));
    }
    let peak = value
        .parse::<usize>()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "corrupt memory profile value"))?;
    if peak == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "corrupt zero memory profile value",
        ));
    }
    Ok(peak)
}

fn read_peak(path: &Path) -> io::Result<Option<usize>> {
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    };
    let mut text = String::new();
    file.read_to_string(&mut text)?;
    parse_peak(&text).map(Some)
}

/// Read a recorded peak in KiB. Invalid on-disk values are reported as errors.
pub fn load_peak(root: &Path, key: &str) -> io::Result<Option<usize>> {
    validate_key(key)?;
    if !root.exists() {
        return Ok(None);
    }
    let _lock = lock_for(root, key)?;
    read_peak(&root.join(key))
}

/// Record the maximum observed peak in KiB; observations never lower history.
pub fn record_peak(root: &Path, key: &str, peak_kib: usize) -> io::Result<()> {
    validate_key(key)?;
    if peak_kib == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "peak memory must be positive",
        ));
    }
    fs::create_dir_all(root)?;
    let _lock = lock_for(root, key)?;
    let target = root.join(key);
    if read_peak(&target)?.is_some_and(|old| old >= peak_kib) {
        return Ok(());
    }

    // Keep the temporary file in a same-filesystem directory. The per-key
    // lock makes a fixed hex filename safe and lets a later writer recover
    // from a process that died before the rename.
    let temp_dir = root.join(".tmp");
    fs::create_dir_all(&temp_dir)?;
    let temp = temp_dir.join(key);
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&temp)?;
    writeln!(file, "{peak_kib}")?;
    fs::rename(&temp, &target)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct Fixture(PathBuf);

    impl Fixture {
        fn new() -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path =
                env::temp_dir().join(format!("memory-profile-{}-{unique}", std::process::id()));
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn profile_updates_are_monotonic_and_keyed_by_hex_name() {
        let fixture = Fixture::new();
        let key = "0123456789abcdef0123456789abcdef";
        assert_eq!(load_peak(&fixture.0, key).unwrap(), None);
        record_peak(&fixture.0, key, 1024).unwrap();
        record_peak(&fixture.0, key, 768).unwrap();
        assert_eq!(load_peak(&fixture.0, key).unwrap(), Some(1024));
        record_peak(&fixture.0, key, 2048).unwrap();
        assert_eq!(load_peak(&fixture.0, key).unwrap(), Some(2048));
        assert!(fixture.0.join(key).is_file());
    }

    #[test]
    fn corrupt_profiles_and_keys_are_rejected() {
        let fixture = Fixture::new();
        let key = "abcdef0123456789abcdef0123456789";
        fs::write(fixture.0.join(key), "2 GiB\n").unwrap();
        assert_eq!(
            load_peak(&fixture.0, key).unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
        assert_eq!(
            load_peak(&fixture.0, "../escape").unwrap_err().kind(),
            io::ErrorKind::InvalidInput
        );
        assert_eq!(
            record_peak(&fixture.0, key, 0).unwrap_err().kind(),
            io::ErrorKind::InvalidInput
        );
    }

    #[test]
    fn command_identity_separates_kind_target_and_options() {
        let base = vec![
            "rustc".into(),
            "--crate-name".into(),
            "fixture".into(),
            "--target".into(),
            "aarch64-pc-windows-gnu".into(),
            "src/lib.rs".into(),
            "-o".into(),
            "target/libfixture.rlib".into(),
        ];
        let first = identity("compile", &base);
        assert_eq!(first, identity("compile", &base));
        assert_ne!(first, identity("link", &base));

        let mut other_target = base.clone();
        other_target[4] = "x86_64-pc-windows-gnu".into();
        assert_ne!(first, identity("compile", &other_target));

        let mut other_option = base.clone();
        other_option.push("-Copt-level=2".into());
        assert_ne!(first, identity("compile", &other_option));
    }

    #[test]
    fn repository_and_canonical_prefixes_share_normalized_spelling() {
        let repo = Path::new("/worktree/agent-17");
        assert_eq!(
            normalize_path("/worktree/agent-17/sdk/lib.rs", Some(repo)),
            "$REPO/sdk/lib.rs"
        );
        assert_eq!(
            normalize_path("/tmp/vibeos/sdk/lib.rs", Some(repo)),
            "$REPO/sdk/lib.rs"
        );
        assert_eq!(
            normalize_path("/worktree/agent-170/sdk/lib.rs", Some(repo)),
            "/worktree/agent-170/sdk/lib.rs"
        );
    }
}
