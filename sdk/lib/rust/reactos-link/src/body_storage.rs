//! Shared Cargo storage for nested body crates.
//!
//! Callers must include the actual target specification bytes (or builtin target
//! triple), toolchain identity, and every setting that changes Cargo's compatible
//! storage group in `configuration`. Keep `Shared` alive through Cargo, depfile
//! reads, and `snapshot`: its exclusive lock protects all three operations.
//!
//! `request` identifies the complete body build invocation, including settings
//! that can change its output within a storage group. Snapshots additionally
//! include the storage configuration, body name, architecture, and archive name
//! in their identity. They are independent copies suitable for linker inputs.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

const DEFAULT_SHARDS: usize = 4;
const MAX_SHARDS: usize = 16;
const CONFIG_MARKER: &str = ".body-storage.config-v1";
const REQUEST_MARKER: &str = ".body-storage.request-v1";
const LOCK_NAME: &str = ".body-storage.lock";
static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// Parse the optional shard setting. Zero disables shared storage at the caller.
/// Invalid values panic so a typo cannot silently choose a different build mode.
pub fn parse_shards(setting: Option<&str>) -> usize {
    match setting {
        None => DEFAULT_SHARDS,
        Some(value) => value
            .parse::<usize>()
            .ok()
            .filter(|count| *count <= MAX_SHARDS)
            .unwrap_or_else(|| {
                panic!("invalid body storage shard setting {value:?}: expected an integer from 0 through {MAX_SHARDS}")
            }),
    }
}

/// IR audits consume per-module directories (`tools/audit-captures.sh`). Keep
/// their existing layout; diagnostic builds are not the normal retained set.
pub fn enabled(shards: usize, emit_ir: bool) -> bool {
    shards != 0 && !emit_ir
}

/// The storage lock is held until this value is dropped, including on unwinding.
#[derive(Debug)]
pub struct Shared {
    target_dir: PathBuf,
    snapshot_namespace: Vec<u8>,
    _lock: File,
}

impl Shared {
    /// Acquire a configuration's shard and check its exact configuration bytes.
    /// `shards == 0` must be handled by the caller's nonshared build path.
    pub fn acquire(
        workspace: &Path,
        arch: &str,
        body: &str,
        configuration: &[u8],
        shards: usize,
    ) -> io::Result<Self> {
        if !(1..=MAX_SHARDS).contains(&shards) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("shared body storage requires 1 through {MAX_SHARDS} shards; got {shards}"),
            ));
        }
        validate_component("architecture", arch)?;
        validate_component("body crate", body)?;
        let shard = digest(body.as_bytes()) % shards as u64;
        let target_dir = workspace.join("target").join(format!(
            "body-storage-v1-{arch}-{:016x}-{shard}-rustlib",
            digest(configuration)
        ));
        fs::create_dir_all(&target_dir)?;
        let lock = acquire_lock(&target_dir)?;
        check_marker(&target_dir.join(CONFIG_MARKER), configuration)?;

        let mut snapshot_namespace = Vec::new();
        append_field(&mut snapshot_namespace, arch.as_bytes());
        append_field(&mut snapshot_namespace, body.as_bytes());
        append_field(&mut snapshot_namespace, configuration);
        Ok(Self {
            target_dir,
            snapshot_namespace,
            _lock: lock,
        })
    }

    pub fn target_dir(&self) -> &Path {
        &self.target_dir
    }

    /// Copy an archive atomically into the consumer's OUT_DIR while still locked.
    /// This deliberately uses a copy, never a hardlink: Cargo may replace or
    /// overwrite the source archive on the next request using the same shard.
    pub fn snapshot(&self, archive: &Path, out_dir: &Path, request: &[u8]) -> io::Result<PathBuf> {
        let name = archive.file_name().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "archive path has no filename")
        })?;
        let mut identity = self.snapshot_namespace.clone();
        append_field(&mut identity, request);
        append_field(&mut identity, name.as_encoded_bytes());
        let directory = out_dir
            .join("body-storage")
            .join(format!("{:016x}", digest(&identity)));
        fs::create_dir_all(&directory)?;
        // OUT_DIR can be shared by independent callers, so the request marker
        // and destination rename need their own lock as well as the shard lock.
        let _snapshot_lock = acquire_lock(&directory)?;
        check_marker(&directory.join(REQUEST_MARKER), &identity)?;
        let destination = directory.join(name);
        let (temporary, temporary_file) = create_temporary(&directory)?;
        drop(temporary_file);
        let result = (|| {
            fs::copy(archive, &temporary)?;
            fs::rename(&temporary, &destination)?;
            Ok(destination)
        })();
        if result.is_err() {
            let _ = fs::remove_file(&temporary);
        }
        result
    }
}

fn validate_component(label: &str, value: &str) -> io::Result<()> {
    if value.is_empty()
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("invalid {label} for body storage: {value:?}"),
        ));
    }
    Ok(())
}

fn acquire_lock(directory: &Path) -> io::Result<File> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(directory.join(LOCK_NAME))?;
    file.lock()?;
    Ok(file)
}

fn check_marker(path: &Path, expected: &[u8]) -> io::Result<()> {
    match fs::read(path) {
        Ok(actual) if actual == expected => Ok(()),
        Ok(_) => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "body storage identity mismatch at {}: digest collision or incompatible storage; refusing reuse",
                path.display()
            ),
        )),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            let directory = path.parent().expect("marker has a parent");
            let (temporary, mut file) = create_temporary(directory)?;
            let result = (|| {
                file.write_all(expected)?;
                file.sync_all()?;
                drop(file);
                fs::rename(&temporary, path)
            })();
            if result.is_err() {
                let _ = fs::remove_file(&temporary);
            }
            result
        }
        Err(error) => Err(error),
    }
}

fn create_temporary(directory: &Path) -> io::Result<(PathBuf, File)> {
    loop {
        let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let path = directory.join(format!(
            ".body-storage-{}-{sequence}.tmp",
            std::process::id()
        ));
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(file) => return Ok((path, file)),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }
}

fn append_field(destination: &mut Vec<u8>, field: &[u8]) {
    destination.extend_from_slice(&(field.len() as u64).to_le_bytes());
    destination.extend_from_slice(field);
}

// Stable FNV-1a, not DefaultHasher (whose algorithm is unspecified). Exact-byte
// markers under exclusive locks make hash collisions an error, not unsafe reuse.
fn digest(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf29ce484222325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestDirectory(PathBuf);

    impl TestDirectory {
        fn new() -> Self {
            let directory = std::env::temp_dir().join(format!(
                "body-storage-test-{}-{}",
                std::process::id(),
                TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&directory).unwrap();
            Self(directory)
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }

    #[test]
    fn ir_audits_keep_the_private_module_layout() {
        assert!(enabled(4, false));
        assert!(!enabled(0, false));
        assert!(!enabled(4, true));
    }

    #[test]
    fn settings_are_explicit_and_bounded() {
        assert_eq!(parse_shards(None), 4);
        assert_eq!(parse_shards(Some("0")), 0);
        assert_eq!(parse_shards(Some("16")), 16);
        for invalid in [
            "",
            "-1",
            "17",
            "4.0",
            "four",
            " 4",
            "999999999999999999999999",
        ] {
            assert!(std::panic::catch_unwind(|| parse_shards(Some(invalid))).is_err());
        }
    }

    #[test]
    fn separately_opened_lock_is_exclusive_and_drop_releases_it() {
        let directory = TestDirectory::new();
        let shared = Shared::acquire(&directory.0, "arm64", "foo", b"stable;target-a", 4).unwrap();
        let contender = OpenOptions::new()
            .read(true)
            .write(true)
            .open(shared.target_dir().join(LOCK_NAME))
            .unwrap();
        assert!(matches!(
            contender.try_lock(),
            Err(std::fs::TryLockError::WouldBlock)
        ));
        drop(shared);
        contender.try_lock().unwrap();
    }

    #[test]
    fn configuration_architecture_and_toolchain_get_separate_storage() {
        let directory = TestDirectory::new();
        let stable = Shared::acquire(&directory.0, "arm64", "foo", b"stable;target-a", 4).unwrap();
        let target = Shared::acquire(&directory.0, "arm64", "foo", b"stable;target-b", 4).unwrap();
        let nightly =
            Shared::acquire(&directory.0, "arm64", "foo", b"nightly;target-a", 4).unwrap();
        let arch = Shared::acquire(&directory.0, "i386", "foo", b"stable;target-a", 4).unwrap();
        assert_ne!(stable.target_dir(), target.target_dir());
        assert_ne!(stable.target_dir(), nightly.target_dir());
        assert_ne!(stable.target_dir(), arch.target_dir());
        let expected = stable.target_dir().to_path_buf();
        drop(stable);
        let reopened =
            Shared::acquire(&directory.0, "arm64", "foo", b"stable;target-a", 4).unwrap();
        assert_eq!(reopened.target_dir(), expected);
        assert!(expected
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .ends_with("-rustlib"));
    }

    #[test]
    fn snapshots_remain_independent_and_request_variants_are_separate() {
        let directory = TestDirectory::new();
        let shared = Shared::acquire(&directory.0, "amd64", "foo", b"stable;target-a", 4).unwrap();
        let archive = shared.target_dir().join("libfoo.a");
        fs::write(&archive, b"first archive").unwrap();
        let first = shared
            .snapshot(&archive, &directory.0.join("out"), b"feature-a")
            .unwrap();
        fs::write(&archive, b"second archive").unwrap();
        let second = shared
            .snapshot(&archive, &directory.0.join("out"), b"feature-b")
            .unwrap();
        assert_ne!(first, second);
        assert_eq!(fs::read(&first).unwrap(), b"first archive");
        assert_eq!(fs::read(&second).unwrap(), b"second archive");
        let replaced = shared
            .snapshot(&archive, &directory.0.join("out"), b"feature-a")
            .unwrap();
        assert_eq!(replaced, first);
        assert_eq!(fs::read(&first).unwrap(), b"second archive");
    }

    #[test]
    fn altered_markers_refuse_reuse() {
        let directory = TestDirectory::new();
        let shared = Shared::acquire(&directory.0, "amd64", "foo", b"stable;target-a", 4).unwrap();
        let root = shared.target_dir().to_path_buf();
        let archive = root.join("libfoo.a");
        fs::write(&archive, b"archive").unwrap();
        let snapshot = shared
            .snapshot(&archive, &directory.0.join("out"), b"feature-a")
            .unwrap();
        fs::write(
            snapshot.parent().unwrap().join(REQUEST_MARKER),
            b"collision",
        )
        .unwrap();
        assert_eq!(
            shared
                .snapshot(&archive, &directory.0.join("out"), b"feature-a")
                .unwrap_err()
                .kind(),
            io::ErrorKind::InvalidData
        );
        drop(shared);
        fs::write(root.join(CONFIG_MARKER), b"collision").unwrap();
        assert_eq!(
            Shared::acquire(&directory.0, "amd64", "foo", b"stable;target-a", 4)
                .unwrap_err()
                .kind(),
            io::ErrorKind::InvalidData
        );
    }

    #[test]
    fn invalid_paths_and_disabled_storage_fail_loudly() {
        let directory = TestDirectory::new();
        for (arch, body, shards) in [
            ("../arm64", "foo", 4),
            ("arm64", "../foo", 4),
            ("arm64", "foo", 0),
            ("arm64", "foo", 17),
        ] {
            assert_eq!(
                Shared::acquire(&directory.0, arch, body, b"config", shards)
                    .unwrap_err()
                    .kind(),
                io::ErrorKind::InvalidInput
            );
        }
    }
}
