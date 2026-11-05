//! Local verified byte publication and immutable cache pinning, independent of host path syntax.
use crate::{Error, Result};
use regex::Regex;
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    os::unix::{fs::MetadataExt, io::AsRawFd},
    path::{Path, PathBuf},
    time::SystemTime,
};
fn identity(m: &fs::Metadata) -> (u64, u64, u64, i64, i64, i64, i64) {
    (
        m.dev(),
        m.ino(),
        m.len(),
        m.mtime(),
        m.mtime_nsec(),
        m.ctime(),
        m.ctime_nsec(),
    )
}
fn same_file(a: &Path, b: &Path) -> bool {
    match (fs::metadata(a), fs::metadata(b)) {
        (Ok(a), Ok(b)) => a.dev() == b.dev() && a.ino() == b.ino(),
        _ => false,
    }
}

pub fn stage_file(src: &Path, staging: &Path) -> Result<PathBuf> {
    stage_file_with(src, staging, |input, output| {
        let mut buf = [0u8; 16 * 1024];
        loop {
            let n = input.read(&mut buf)?;
            if n == 0 {
                break;
            }
            output.write_all(&buf[..n])?;
        }
        Ok(())
    })
}
fn stage_file_with(
    src: &Path,
    staging: &Path,
    mut copier: impl FnMut(&mut File, &mut File) -> Result<()>,
) -> Result<PathBuf> {
    fs::create_dir_all(staging)?;
    let name = src.file_name().ok_or("staging source has no basename")?;
    let dst = staging.join(name);
    if same_file(src, &dst) {
        return Ok(dst);
    }
    let before = fs::metadata(src)?;
    let mut temporary = tempfile::NamedTempFile::new_in(staging)?;
    let mut input = File::open(src)?;
    copier(&mut input, temporary.as_file_mut())?;
    temporary.flush()?;
    if identity(&before) != identity(&fs::metadata(src)?)
        || temporary.as_file().metadata()?.len() != before.len()
    {
        return Err(Error::new(format!(
            "QEMU staging input changed while copying: {}",
            src.display()
        )));
    }
    fs::set_permissions(temporary.path(), before.permissions())?;
    let times = std::fs::FileTimes::new()
        .set_accessed(before.accessed()?)
        .set_modified(before.modified()?);
    temporary.as_file().set_times(times)?;
    temporary
        .persist(&dst)
        .map_err(|e| Error::new(format!("publish staging {}: {}", dst.display(), e.error)))?;
    Ok(dst)
}

pub struct MediaLock(File);
impl MediaLock {
    pub fn acquire(name: &str, root: Option<&Path>) -> Result<Self> {
        if name.contains('/') || name.contains('\\') {
            return Err("invalid media lock name".into());
        }
        let root = root
            .map(Path::to_owned)
            .unwrap_or_else(|| {
                PathBuf::from(
                    std::env::var("VIBEOS_HOSTLOCK_DIR")
                        .unwrap_or_else(|_| "/tmp/vibeos-qemu-locks/host".into()),
                )
            })
            .join("media");
        fs::create_dir_all(&root)?;
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .open(root.join(name))?;
        if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX) } != 0 {
            return Err(std::io::Error::last_os_error().into());
        }
        Ok(Self(file))
    }
}
impl Drop for MediaLock {
    fn drop(&mut self) {
        unsafe { libc::flock(self.0.as_raw_fd(), libc::LOCK_UN) };
    }
}

pub fn prune_media_cache(cache: &Path, keep: &Path) -> Result<()> {
    prune_with_limit(cache, keep, 2 * 1024u64.pow(3))
}
fn prune_with_limit(cache: &Path, keep: &Path, limit: u64) -> Result<()> {
    let re = Regex::new(r"^[0-9a-f]{64}-.+$")?;
    let mut entries = Vec::new();
    let mut total = 0u64;
    for e in fs::read_dir(cache)? {
        let e = e?;
        if !re.is_match(&e.file_name().to_string_lossy()) {
            continue;
        }
        let m = match e.metadata() {
            Ok(m) => m,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => return Err(e.into()),
        };
        total = total.saturating_add(m.len());
        entries.push((m.mtime(), m.mtime_nsec(), m.len(), m.nlink(), e.path()));
    }
    entries.sort();
    for (_, _, size, links, path) in entries {
        if total <= limit {
            break;
        }
        if path == keep || links > 1 {
            continue;
        }
        match fs::remove_file(&path) {
            Ok(()) => total = total.saturating_sub(size),
            Err(e)
                if matches!(
                    e.kind(),
                    std::io::ErrorKind::NotFound | std::io::ErrorKind::PermissionDenied
                ) => {}
            Err(e) => return Err(e.into()),
        }
    }
    Ok(())
}
fn pin(
    src: &Path,
    before: &fs::Metadata,
    entry: &Path,
    staging: &Path,
    limit: u64,
) -> Result<bool> {
    if !entry.try_exists()? {
        return Ok(false);
    }
    if fs::metadata(entry)?.len() != before.len() {
        return Err(Error::new(format!(
            "QEMU read-only media cache is damaged: {}",
            entry.display()
        )));
    }
    if identity(before) != identity(&fs::metadata(src)?) {
        return Err(Error::new(format!(
            "QEMU staging input changed before pinning: {}",
            src.display()
        )));
    }
    fs::create_dir_all(staging)?;
    let pinned = staging.join(src.file_name().ok_or("media source has no basename")?);
    if pinned.try_exists()? {
        if !same_file(&pinned, entry) {
            return Err(Error::new(format!(
                "Conflicting read-only staging basename: {}",
                pinned.display()
            )));
        }
    } else {
        fs::hard_link(entry, &pinned)?;
    }
    let file = File::open(entry)?;
    file.set_times(fs::FileTimes::new().set_modified(SystemTime::now()))?;
    prune_with_limit(
        entry.parent().ok_or("cache entry lacks parent")?,
        entry,
        limit,
    )?;
    Ok(true)
}

pub fn stage_readonly_file(src: &Path, staging: &Path) -> Result<PathBuf> {
    readonly_with(src, staging, None, 2 * 1024u64.pow(3), stage_file)
}
fn readonly_with(
    src: &Path,
    staging: &Path,
    lock_root: Option<&Path>,
    limit: u64,
    mut copier: impl FnMut(&Path, &Path) -> Result<PathBuf>,
) -> Result<PathBuf> {
    let before = fs::metadata(src)?;
    let mut input = File::open(src)?;
    let mut digest = Sha256::new();
    let mut buf = [0u8; 1024 * 1024];
    loop {
        let n = input.read(&mut buf)?;
        if n == 0 {
            break;
        }
        digest.update(&buf[..n]);
    }
    if identity(&before) != identity(&fs::metadata(src)?) {
        return Err(Error::new(format!(
            "QEMU staging input changed while hashing: {}",
            src.display()
        )));
    }
    let cache = staging
        .parent()
        .ok_or("staging root has no parent")?
        .join("media-cache-v1");
    let basename = src
        .file_name()
        .ok_or("media source has no basename")?
        .to_string_lossy();
    let entry = cache.join(format!("{:x}-{basename}", digest.finalize()));
    {
        let _lock = MediaLock::acquire("cache.lock", lock_root)?;
        fs::create_dir_all(&cache)?;
        if pin(src, &before, &entry, staging, limit)? {
            return Ok(staging.join(src.file_name().unwrap()));
        }
    }
    let key = format!("{:x}", Sha256::digest(entry.to_string_lossy().as_bytes()));
    let _fill = MediaLock::acquire(&format!("fill-{key}.lock"), lock_root)?;
    {
        let _lock = MediaLock::acquire("cache.lock", lock_root)?;
        if pin(src, &before, &entry, staging, limit)? {
            return Ok(staging.join(src.file_name().unwrap()));
        }
    }
    let private = tempfile::Builder::new()
        .prefix(&format!(".populate-v2-{}-", std::process::id()))
        .tempdir_in(&cache)?;
    copier(src, private.path())?;
    {
        let _lock = MediaLock::acquire("cache.lock", lock_root)?;
        if identity(&before) != identity(&fs::metadata(src)?) {
            return Err(Error::new(format!(
                "QEMU staging input changed before publication: {}",
                src.display()
            )));
        }
        if !entry.try_exists()? {
            fs::rename(private.path().join(src.file_name().unwrap()), &entry)?;
        }
        if !pin(src, &before, &entry, staging, limit)? {
            return Err("QEMU cache entry disappeared under lock".into());
        }
    }
    Ok(staging.join(src.file_name().unwrap()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        sync::{
            atomic::{AtomicUsize, Ordering},
            mpsc, Arc, Barrier, Condvar, Mutex,
        },
        thread,
        time::Duration,
    };
    const LIMIT: u64 = 2 * 1024 * 1024 * 1024;
    fn cached(src: &Path, staging: &Path, locks: &Path) -> Result<PathBuf> {
        readonly_with(src, staging, Some(locks), LIMIT, stage_file)
    }
    fn entry(root: &Path, name: &str, bytes: &[u8]) -> PathBuf {
        root.join("media-cache-v1")
            .join(format!("{:x}-{name}", Sha256::digest(bytes)))
    }
    struct Rendezvous {
        count: Mutex<usize>,
        changed: Condvar,
    }
    impl Rendezvous {
        fn new() -> Self {
            Self {
                count: Mutex::new(0),
                changed: Condvar::new(),
            }
        }
        fn wait(&self) -> Result<()> {
            let mut count = self.count.lock().unwrap();
            *count += 1;
            self.changed.notify_all();
            let (count, _) = self
                .changed
                .wait_timeout_while(count, Duration::from_secs(3), |n| *n < 2)
                .unwrap();
            if *count < 2 {
                return Err("independent media copies did not overlap".into());
            }
            Ok(())
        }
    }
    #[test]
    fn readonly_pins_bytes_and_detects_same_size_mtime_rebuild() {
        let temp = tempfile::tempdir().unwrap();
        let src = temp.path().join("disk.iso");
        let root = temp.path().join("windows");
        let locks = temp.path().join("locks");
        fs::write(&src, b"first image").unwrap();
        let times = fs::metadata(&src).unwrap();
        let first = cached(&src, &root.join("one"), &locks).unwrap();
        let second = cached(&src, &root.join("two"), &locks).unwrap();
        assert!(same_file(&first, &second));
        fs::write(&src, b"other image").unwrap();
        File::open(&src)
            .unwrap()
            .set_times(fs::FileTimes::new().set_modified(times.modified().unwrap()))
            .unwrap();
        let third = cached(&src, &root.join("three"), &locks).unwrap();
        assert_eq!(fs::read(&first).unwrap(), b"first image");
        assert_eq!(fs::read(&third).unwrap(), b"other image");
        assert!(!same_file(&first, &third));
    }
    #[test]
    fn immutable_sources_do_not_need_write_permissions_to_be_pinned() {
        use std::os::unix::fs::PermissionsExt;
        let temp = tempfile::tempdir().unwrap();
        let src = temp.path().join("code.fd");
        fs::write(&src, b"firmware").unwrap();
        fs::set_permissions(&src, fs::Permissions::from_mode(0o444)).unwrap();
        let output = cached(
            &src,
            &temp.path().join("windows/run"),
            &temp.path().join("locks"),
        )
        .unwrap();
        assert_eq!(fs::read(output).unwrap(), b"firmware");
    }
    #[test]
    fn writable_publications_and_open_old_inode_are_private() {
        let temp = tempfile::tempdir().unwrap();
        let src = temp.path().join("vars.qcow2");
        fs::write(&src, b"original bytes").unwrap();
        let first = stage_file(&src, &temp.path().join("one")).unwrap();
        let second = stage_file(&src, &temp.path().join("two")).unwrap();
        assert!(!same_file(&first, &second));
        let mut old = File::open(&first).unwrap();
        fs::write(&src, b"newer bytes").unwrap();
        stage_file(&src, &temp.path().join("one")).unwrap();
        let mut bytes = Vec::new();
        old.read_to_end(&mut bytes).unwrap();
        assert_eq!(bytes, b"original bytes");
        assert_eq!(fs::read(second).unwrap(), b"original bytes");
        assert_eq!(fs::read(first).unwrap(), b"newer bytes");
    }
    #[test]
    fn warm_cache_hit_does_not_invoke_the_copy_path() {
        let temp = tempfile::tempdir().unwrap();
        let src = temp.path().join("code.fd");
        let root = temp.path().join("windows");
        let locks = temp.path().join("locks");
        fs::write(&src, b"firmware").unwrap();
        cached(&src, &root.join("one"), &locks).unwrap();
        let output = readonly_with(&src, &root.join("two"), Some(&locks), LIMIT, |_, _| {
            panic!("warm cache copied media")
        })
        .unwrap();
        assert_eq!(fs::read(output).unwrap(), b"firmware");
    }
    #[test]
    fn prune_removes_only_unpinned_entries() {
        let temp = tempfile::tempdir().unwrap();
        let src = temp.path().join("boot.img");
        let root = temp.path().join("windows");
        let locks = temp.path().join("locks");
        fs::write(&src, b"first").unwrap();
        let first = readonly_with(&src, &root.join("one"), Some(&locks), 8, stage_file).unwrap();
        fs::write(&src, b"later").unwrap();
        let second = readonly_with(&src, &root.join("two"), Some(&locks), 8, stage_file).unwrap();
        let old_entry = entry(&root, "boot.img", b"first");
        assert!(old_entry.exists());
        fs::remove_file(first).unwrap();
        prune_with_limit(&root.join("media-cache-v1"), Path::new("not-an-entry"), 8).unwrap();
        assert!(!old_entry.exists());
        assert_eq!(fs::read(second).unwrap(), b"later");
    }
    #[test]
    fn different_cold_fills_copy_concurrently() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("windows");
        let locks = temp.path().join("locks");
        let rendezvous = Arc::new(Rendezvous::new());
        let mut tasks = Vec::new();
        for index in 0..2 {
            let source = temp.path().join(format!("{index}.iso"));
            fs::write(&source, vec![index as u8; 4096]).unwrap();
            let staging = root.join(index.to_string());
            let locks = locks.clone();
            let rendezvous = rendezvous.clone();
            tasks.push(thread::spawn(move || {
                readonly_with(&source, &staging, Some(&locks), LIMIT, |src, work| {
                    rendezvous.wait()?;
                    stage_file(src, work)
                })
            }));
        }
        for task in tasks {
            assert!(task.join().unwrap().is_ok());
        }
    }
    #[test]
    fn warm_hit_does_not_wait_for_a_cold_copy() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("windows");
        let locks = temp.path().join("locks");
        let warm = temp.path().join("warm.iso");
        let cold = temp.path().join("cold.iso");
        fs::write(&warm, b"warm").unwrap();
        fs::write(&cold, b"cold").unwrap();
        cached(&warm, &root.join("first"), &locks).unwrap();
        let (entered_tx, entered_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let cold_root = root.clone();
        let cold_locks = locks.clone();
        let cold_task = thread::spawn(move || {
            readonly_with(
                &cold,
                &cold_root.join("cold-run"),
                Some(&cold_locks),
                LIMIT,
                |src, work| {
                    entered_tx.send(()).unwrap();
                    release_rx
                        .recv_timeout(Duration::from_secs(5))
                        .map_err(|_| Error::new("cold copy fixture release timed out"))?;
                    stage_file(src, work)
                },
            )
        });
        entered_rx.recv_timeout(Duration::from_secs(3)).unwrap();
        let (done_tx, done_rx) = mpsc::channel();
        let warm_task = thread::spawn(move || {
            done_tx
                .send(cached(&warm, &root.join("warm-run"), &locks))
                .unwrap();
        });
        let hit = done_rx.recv_timeout(Duration::from_secs(2));
        let _ = release_tx.send(());
        assert!(cold_task.join().unwrap().is_ok());
        warm_task.join().unwrap();
        assert_eq!(
            fs::read(
                hit.expect("warm hit blocked behind unrelated copy")
                    .unwrap()
            )
            .unwrap(),
            b"warm"
        );
    }
    #[test]
    fn simultaneous_same_entry_is_copied_once() {
        let temp = tempfile::tempdir().unwrap();
        let src = temp.path().join("shared.iso");
        let root = temp.path().join("windows");
        let locks = temp.path().join("locks");
        fs::write(&src, b"shared").unwrap();
        let start = Arc::new(Barrier::new(4));
        let copies = Arc::new(AtomicUsize::new(0));
        let mut tasks = Vec::new();
        for index in 0..4 {
            let src = src.clone();
            let staging = root.join(index.to_string());
            let locks = locks.clone();
            let start = start.clone();
            let copies = copies.clone();
            tasks.push(thread::spawn(move || {
                start.wait();
                readonly_with(&src, &staging, Some(&locks), LIMIT, |src, work| {
                    copies.fetch_add(1, Ordering::SeqCst);
                    stage_file(src, work)
                })
            }));
        }
        let outputs: Vec<_> = tasks
            .into_iter()
            .map(|task| task.join().unwrap().unwrap())
            .collect();
        assert_eq!(copies.load(Ordering::SeqCst), 1);
        assert!(outputs.iter().all(|path| same_file(path, &outputs[0])));
    }
    #[test]
    fn concurrently_published_legacy_entry_is_never_overwritten() {
        let temp = tempfile::tempdir().unwrap();
        let src = temp.path().join("disk.iso");
        let root = temp.path().join("windows");
        let locks = temp.path().join("locks");
        fs::write(&src, b"immutable").unwrap();
        let existing = entry(&root, "disk.iso", b"immutable");
        let pinned = root.join("legacy-pin");
        let output = readonly_with(&src, &root.join("new"), Some(&locks), LIMIT, |src, work| {
            let output = stage_file(src, work)?;
            let _guard = MediaLock::acquire("cache.lock", Some(&locks))?;
            fs::write(&existing, b"immutable")?;
            fs::hard_link(&existing, &pinned)?;
            Ok(output)
        })
        .unwrap();
        assert!(same_file(&output, &pinned));
    }
    #[test]
    fn failed_write_preserves_old_publication_and_removes_temporary_bytes() {
        let temp = tempfile::tempdir().unwrap();
        let src = temp.path().join("disk.iso");
        let staging = temp.path().join("work");
        fs::write(&src, b"new complete media").unwrap();
        fs::create_dir_all(&staging).unwrap();
        fs::write(staging.join("disk.iso"), b"previous complete media").unwrap();
        let result = stage_file_with(&src, &staging, |_, output| {
            output.write_all(b"partial")?;
            Err("fixture write failed".into())
        });
        assert!(result.is_err());
        assert_eq!(
            fs::read(staging.join("disk.iso")).unwrap(),
            b"previous complete media"
        );
        assert_eq!(fs::read_dir(staging).unwrap().count(), 1);
    }
    #[test]
    fn failed_fill_never_publishes_partial_cached_media() {
        let temp = tempfile::tempdir().unwrap();
        let src = temp.path().join("disk.iso");
        let root = temp.path().join("windows");
        let locks = temp.path().join("locks");
        fs::write(&src, b"complete").unwrap();
        let output = root.join("run/disk.iso");
        let result = readonly_with(&src, &root.join("run"), Some(&locks), LIMIT, |_, work| {
            fs::write(work.join("disk.iso"), b"partial")?;
            Err("fixture copy failed".into())
        });
        assert!(result.is_err());
        assert!(!output.exists());
        assert_eq!(
            fs::read_dir(root.join("media-cache-v1")).unwrap().count(),
            0
        );
    }
    #[test]
    fn changed_source_during_copy_is_not_published() {
        let temp = tempfile::tempdir().unwrap();
        let src = temp.path().join("disk.iso");
        let work = temp.path().join("work");
        fs::write(&src, b"original").unwrap();
        let modified = fs::metadata(&src).unwrap().modified().unwrap();
        let result = stage_file_with(&src, &work, |input, output| {
            std::io::copy(input, output)?;
            fs::write(&src, b"mutation")?;
            File::open(&src)?.set_times(fs::FileTimes::new().set_modified(modified))?;
            Ok(())
        });
        assert!(result
            .unwrap_err()
            .message
            .contains("changed while copying"));
        assert!(!work.join("disk.iso").exists());
    }
    #[test]
    fn changed_source_after_copy_is_not_published() {
        let temp = tempfile::tempdir().unwrap();
        let src = temp.path().join("disk.iso");
        let root = temp.path().join("windows");
        let locks = temp.path().join("locks");
        fs::write(&src, b"original").unwrap();
        let modified = fs::metadata(&src).unwrap().modified().unwrap();
        let result = readonly_with(&src, &root.join("run"), Some(&locks), LIMIT, |src, work| {
            let copied = stage_file(src, work)?;
            fs::write(src, b"mutation")?;
            File::open(src)?.set_times(fs::FileTimes::new().set_modified(modified))?;
            Ok(copied)
        });
        assert!(result
            .unwrap_err()
            .message
            .contains("changed before publication"));
        assert!(!root.join("run/disk.iso").exists());
        assert_eq!(
            fs::read_dir(root.join("media-cache-v1")).unwrap().count(),
            0
        );
    }
    #[test]
    #[ignore = "subprocess entry point exercised by crashed_fill_releases_lease_and_is_reclaimable"]
    fn crashed_fill_helper() {
        let root = PathBuf::from(
            std::env::var_os("VIBEOS_PUBLICATION_CRASH_FIXTURE").expect("fixture root required"),
        );
        let src = root.join("disk.iso");
        let staging = root.join("windows/dead");
        let locks = root.join("locks");
        let _ = readonly_with(&src, &staging, Some(&locks), LIMIT, |_, work| {
            fs::write(work.join("disk.iso"), b"partial")?;
            std::process::exit(9)
        });
        panic!("crash fixture did not enter copy");
    }
    #[test]
    fn crashed_fill_releases_lease_and_is_reclaimable() {
        let temp = tempfile::tempdir().unwrap();
        let src = temp.path().join("disk.iso");
        let root = temp.path().join("windows");
        let locks = temp.path().join("locks");
        fs::write(&src, b"complete").unwrap();
        let status = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "publication::tests::crashed_fill_helper",
                "--ignored",
                "--nocapture",
            ])
            .env("VIBEOS_PUBLICATION_CRASH_FIXTURE", temp.path())
            .status()
            .unwrap();
        assert_eq!(status.code(), Some(9));
        let fills = fs::read_dir(root.join("media-cache-v1"))
            .unwrap()
            .map(|e| e.unwrap().path())
            .collect::<Vec<_>>();
        assert_eq!(fills.len(), 1);
        assert!(fills[0]
            .file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with(".populate-v2-"));
        let result = crate::backend::cleanup_staging(&root, false, 0.0).unwrap();
        assert!(result["removed"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v.as_str() == fills[0].to_str()));
        assert_eq!(
            fs::read(cached(&src, &root.join("retry"), &locks).unwrap()).unwrap(),
            b"complete"
        );
    }
}
