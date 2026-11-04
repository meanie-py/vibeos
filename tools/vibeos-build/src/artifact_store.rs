//! Trusted local immutable body store. Lock order: consumer -> action -> GC.
//! GC never waits for an action lock. Stable lock files are never removed.
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::CString,
    fs::{self, File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    os::{
        fd::AsRawFd,
        unix::{
            ffi::OsStrExt,
            fs::{MetadataExt, OpenOptionsExt, PermissionsExt},
        },
    },
    path::{Component, Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
pub const SCHEMA: &str = "vibeos-artifact-store-v1";
#[derive(Debug, Clone, Serialize)]
pub struct StoreError {
    pub kind: String,
    pub message: String,
}
impl StoreError {
    pub fn new(kind: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            kind: kind.into(),
            message: message.into(),
        }
    }
}
impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}
impl std::error::Error for StoreError {}
impl From<std::io::Error> for StoreError {
    fn from(e: std::io::Error) -> Self {
        Self::new("StoreError", e.to_string())
    }
}
impl From<serde_json::Error> for StoreError {
    fn from(e: serde_json::Error) -> Self {
        Self::new("StoreError", e.to_string())
    }
}
pub type StoreResult<T> = std::result::Result<T, StoreError>;
fn error(message: impl Into<String>) -> StoreError {
    StoreError::new("StoreError", message)
}
fn corrupt(message: impl Into<String>) -> StoreError {
    StoreError::new("CorruptStore", message)
}
fn session_error(message: impl Into<String>) -> StoreError {
    StoreError::new("SessionError", message)
}
fn conflict(message: impl Into<String>) -> StoreError {
    StoreError::new("ActionConflict", message)
}
pub fn canonical(v: &Value) -> StoreResult<Vec<u8>> {
    crate::json::canonical(v)
        .map(|s| s.into_bytes())
        .map_err(|e| error(e.message))
}
pub fn action_key(v: &Value) -> StoreResult<String> {
    if !v.is_object() {
        return Err(error("action descriptor must be a JSON object"));
    }
    Ok(hash(&canonical(v)?))
}
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
pub fn key(value: &str) -> StoreResult<&str> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(error(format!("invalid SHA-256 identity: {value:?}")));
    }
    Ok(value)
}
pub fn name(value: &str) -> StoreResult<&str> {
    let p = Path::new(value);
    let normalized = p
        .components()
        .filter(|c| !matches!(c, Component::CurDir))
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/");
    if value.is_empty()
        || p.is_absolute()
        || p.components().any(|c| matches!(c, Component::ParentDir))
        || normalized != value
        || value == "."
    {
        return Err(error(format!("invalid output name: {value:?}")));
    }
    Ok(value)
}
fn now_ns() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
}
fn now() -> f64 {
    now_ns() as f64 / 1_000_000_000.0
}
fn random_hex(bytes: usize) -> StoreResult<String> {
    let mut b = vec![0; bytes];
    File::open("/dev/urandom")?.read_exact(&mut b)?;
    Ok(b.iter().map(|b| format!("{b:02x}")).collect())
}
pub fn start_identity(pid: u32) -> Option<Value> {
    crate::artifact_lifetime::start_identity(pid)
}
pub fn owner() -> StoreResult<Value> {
    let pid = std::process::id();
    Ok(
        json!({"pid":pid,"start_identity":start_identity(pid).ok_or_else(||error("Linux process start identity unavailable"))?}),
    )
}
fn fields(v: &Value, names: &[&str]) -> bool {
    v.as_object()
        .is_some_and(|m| m.len() == names.len() && names.iter().all(|n| m.contains_key(*n)))
}
pub fn owner_alive(v: &Value) -> StoreResult<bool> {
    if !fields(v, &["pid", "start_identity"]) {
        return Err(corrupt("invalid process owner identity"));
    }
    let identity = &v["start_identity"];
    let pid = v["pid"]
        .as_u64()
        .filter(|p| *p > 0 && *p <= u32::MAX as u64);
    if pid.is_none()
        || !fields(identity, &["boot_id", "start_ticks"])
        || identity
            .as_object()
            .is_none_or(|m| m.values().any(|v| v.as_str().is_none_or(|s| s.is_empty())))
    {
        return Err(corrupt("invalid process start identity"));
    }
    Ok(start_identity(pid.unwrap() as u32).as_ref() == Some(identity))
}
pub fn exists(p: &Path) -> bool {
    fs::symlink_metadata(p).is_ok()
}
pub fn fsync_dir(p: &Path) -> StoreResult<()> {
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW)
        .open(p)?;
    file.sync_all()?;
    Ok(())
}
pub fn mkdir(p: &Path) -> StoreResult<()> {
    let mut missing = vec![];
    let mut current = p;
    while !exists(current) {
        missing.push(current.to_owned());
        current = current
            .parent()
            .ok_or_else(|| corrupt("directory has no existing ancestor"))?;
    }
    if !fs::symlink_metadata(current)?.is_dir() {
        return Err(corrupt(format!(
            "store directory ancestor is not a real directory: {}",
            current.display()
        )));
    }
    for directory in missing.iter().rev() {
        match fs::create_dir(directory) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(e) => return Err(e.into()),
        }
        if !fs::symlink_metadata(directory)?.is_dir() {
            return Err(corrupt(format!(
                "store directory is not a real directory: {}",
                directory.display()
            )));
        }
        fsync_dir(directory)?;
        fsync_dir(directory.parent().unwrap())?;
    }
    if !fs::symlink_metadata(p)?.is_dir() {
        return Err(corrupt(format!(
            "store directory is not a real directory: {}",
            p.display()
        )));
    }
    Ok(())
}
pub fn open_regular(p: &Path) -> StoreResult<File> {
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(p)
        .map_err(|e| {
            corrupt(format!(
                "cannot open regular store file {}: {e}",
                p.display()
            ))
        })?;
    if !file.metadata()?.is_file() {
        return Err(corrupt(format!("not a regular file: {}", p.display())));
    }
    Ok(file)
}
fn stamp(m: &fs::Metadata) -> [i128; 5] {
    crate::artifact_fs::stamp(m)
}
pub fn read(p: &Path) -> StoreResult<Vec<u8>> {
    let mut f = open_regular(p)?;
    let before = stamp(&f.metadata()?);
    let mut b = vec![];
    f.read_to_end(&mut b)?;
    if before != stamp(&f.metadata()?) {
        return Err(corrupt(format!(
            "file changed during read: {}",
            p.display()
        )));
    }
    Ok(b)
}
pub fn read_json(p: &Path) -> StoreResult<Value> {
    let raw = read(p)?;
    let v: Value = serde_json::from_slice(&raw)
        .map_err(|e| corrupt(format!("invalid JSON at {}: {e}", p.display())))?;
    if !v.is_object() || canonical(&v)? != raw {
        return Err(corrupt(format!(
            "invalid JSON at {}: noncanonical JSON object",
            p.display()
        )));
    }
    Ok(v)
}
#[cfg(test)]
thread_local! {static DIGEST_OVERRIDE:std::cell::RefCell<Option<(String,u64)>>=const{std::cell::RefCell::new(None)};static DIGEST_READS:std::cell::Cell<usize>=const{std::cell::Cell::new(0)};static FAIL_ACTION_WRITE:std::cell::Cell<bool>=const{std::cell::Cell::new(false)};static CRASH_AFTER_BLOB:std::cell::Cell<bool>=const{std::cell::Cell::new(false)};}
pub fn digest_file(p: &Path) -> StoreResult<(String, u64)> {
    #[cfg(test)]
    {
        DIGEST_READS.with(|n| n.set(n.get() + 1));
        if let Some(v) = DIGEST_OVERRIDE.with(|v| v.borrow().clone()) {
            return Ok(v);
        }
    }
    let mut f = open_regular(p)?;
    let before = stamp(&f.metadata()?);
    let mut digest = Sha256::new();
    let mut size = 0;
    let mut b = vec![0; 1024 * 1024];
    loop {
        let n = f.read(&mut b)?;
        if n == 0 {
            break;
        }
        digest.update(&b[..n]);
        size += n as u64;
    }
    if before != stamp(&f.metadata()?) {
        return Err(corrupt(format!(
            "file changed during hashing: {}",
            p.display()
        )));
    }
    Ok((format!("{:x}", digest.finalize()), size))
}
pub fn same_bytes(a: &Path, b: &Path) -> StoreResult<bool> {
    let mut a = open_regular(a)?;
    let mut b = open_regular(b)?;
    let mut x = vec![0; 1024 * 1024];
    let mut y = vec![0; 1024 * 1024];
    loop {
        let an = a.read(&mut x)?;
        let bn = b.read(&mut y)?;
        if x[..an] != y[..bn] {
            return Ok(false);
        }
        if an == 0 {
            return Ok(true);
        }
    }
}
pub fn rename_noreplace(src: &Path, dst: &Path) -> StoreResult<bool> {
    let a = CString::new(src.as_os_str().as_bytes()).map_err(|_| error("NUL in store path"))?;
    let b = CString::new(dst.as_os_str().as_bytes()).map_err(|_| error("NUL in store path"))?;
    let rc = unsafe {
        libc::renameat2(
            libc::AT_FDCWD,
            a.as_ptr(),
            libc::AT_FDCWD,
            b.as_ptr(),
            libc::RENAME_NOREPLACE,
        )
    };
    if rc == 0 {
        fsync_dir(dst.parent().unwrap())?;
        if src.parent() != dst.parent() {
            fsync_dir(src.parent().unwrap())?;
        }
        return Ok(true);
    }
    let e = std::io::Error::last_os_error();
    if e.raw_os_error() == Some(libc::EEXIST) {
        return Ok(false);
    }
    Err(error(format!("atomic no-replace publication failed: {e}")))
}
pub fn write_new(p: &Path, payload: &[u8]) -> StoreResult<()> {
    #[cfg(test)]
    if FAIL_ACTION_WRITE.with(|v| v.get()) && p.file_name().is_some_and(|n| n == "action.json") {
        return Err(std::io::Error::from_raw_os_error(libc::ENOSPC).into());
    }
    let mut f = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW)
        .open(p)?;
    f.write_all(payload)?;
    f.sync_all()?;
    fs::set_permissions(p, fs::Permissions::from_mode(0o444))?;
    Ok(())
}
pub fn remove_private_tree(p: &Path) -> StoreResult<()> {
    let m = fs::symlink_metadata(p)?;
    if !m.is_dir() {
        return Err(corrupt(format!(
            "scratch root is not a real directory: {}",
            p.display()
        )));
    }
    fn chmod_dirs(p: &Path) -> StoreResult<()> {
        let m = fs::symlink_metadata(p)?;
        if !m.is_dir() {
            return Ok(());
        }
        fs::set_permissions(p, fs::Permissions::from_mode(m.mode() | 0o700))?;
        for e in fs::read_dir(p)? {
            let e = e?;
            if e.file_type()?.is_dir() {
                chmod_dirs(&e.path())?;
            }
        }
        Ok(())
    }
    chmod_dirs(p)?;
    fs::remove_dir_all(p)?;
    Ok(())
}
pub fn replace_json(p: &Path, v: &Value) -> StoreResult<()> {
    let temporary = p.with_file_name(format!(
        ".{}.{}",
        p.file_name().unwrap().to_string_lossy(),
        random_hex(16)?
    ));
    write_new(&temporary, &canonical(v)?)?;
    fs::rename(&temporary, p)?;
    fsync_dir(p.parent().unwrap())
}
#[derive(Debug)]
pub struct FileLock {
    file: File,
}
impl FileLock {
    pub fn acquire(p: &Path, nonblocking: bool) -> StoreResult<Option<Self>> {
        let f = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .mode(0o600)
            .custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW)
            .open(p)?;
        if !f.metadata()?.is_file() {
            return Err(corrupt(format!("lock is not regular: {}", p.display())));
        }
        let rc = unsafe {
            libc::flock(
                f.as_raw_fd(),
                libc::LOCK_EX | if nonblocking { libc::LOCK_NB } else { 0 },
            )
        };
        if rc < 0 {
            let e = std::io::Error::last_os_error();
            if nonblocking && e.kind() == std::io::ErrorKind::WouldBlock {
                return Ok(None);
            }
            return Err(e.into());
        }
        Ok(Some(Self { file: f }))
    }
}
fn held(p: &Path) -> StoreResult<bool> {
    if !exists(p) {
        return Ok(false);
    }
    Ok(FileLock::acquire(p, true)?.is_none())
}
fn clone_or_copy_with<F>(incoming: &mut File, outgoing: &mut File, mut copy: F) -> StoreResult<()>
where
    F: FnMut(i32, i32) -> std::io::Result<usize>,
{
    loop {
        match copy(incoming.as_raw_fd(), outgoing.as_raw_fd()) {
            Ok(0) => return Ok(()),
            Ok(_) => {}
            Err(e) if e.raw_os_error() == Some(libc::EINTR) => continue,
            Err(e)
                if [libc::EXDEV, libc::EINVAL, libc::ENOSYS, libc::EOPNOTSUPP]
                    .contains(&e.raw_os_error().unwrap_or(0)) =>
            {
                break
            }
            Err(e) => return Err(e.into()),
        }
    }
    incoming.seek(SeekFrom::Start(0))?;
    outgoing.seek(SeekFrom::Start(0))?;
    outgoing.set_len(0)?;
    std::io::copy(incoming, outgoing)?;
    Ok(())
}
pub fn clone_or_copy(incoming: &mut File, outgoing: &mut File) -> StoreResult<()> {
    clone_or_copy_with(incoming, outgoing, |src, dst| {
        let rc = unsafe {
            libc::copy_file_range(
                src,
                std::ptr::null_mut(),
                dst,
                std::ptr::null_mut(),
                1 << 30,
                0,
            )
        };
        if rc < 0 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(rc as usize)
        }
    })
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Output {
    pub path: PathBuf,
    #[serde(default = "archive_role")]
    pub role: String,
}
fn archive_role() -> String {
    "archive".into()
}
impl Output {
    pub fn archive(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            role: archive_role(),
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BuildResult {
    pub outputs: BTreeMap<String, Output>,
    #[serde(default = "empty_object")]
    pub provenance: Value,
    #[serde(default)]
    pub dependency_actions: Vec<String>,
    #[serde(default)]
    pub input_inventory: Value,
}
fn empty_object() -> Value {
    json!({})
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Handle {
    pub action_key: String,
    pub manifest: Value,
    pub outputs: BTreeMap<String, PathBuf>,
    pub hit: bool,
    pub verified_rebuild: bool,
    pub session_verified: bool,
}
impl Handle {
    pub fn blob_path(&self) -> StoreResult<PathBuf> {
        let archives: Vec<_> = array(&self.manifest["outputs"], "outputs")?
            .iter()
            .filter(|v| v["role"] == "archive")
            .collect();
        if archives.len() != 1 {
            return Err(error("handle does not have exactly one archive output"));
        }
        self.outputs
            .get(text(&archives[0]["name"], "output name")?)
            .cloned()
            .ok_or_else(|| corrupt("archive path missing"))
    }
}
#[derive(Clone, Debug)]
pub struct Store {
    pub root: PathBuf,
    pub base: PathBuf,
}
#[derive(Debug)]
pub struct Session {
    pub store: Store,
    pub owner: Value,
    pub owned: bool,
    pub identifier: String,
    pub token: String,
    pub closed: bool,
    lock: Option<FileLock>,
    creator_pid: u32,
}
impl Session {
    /// Release a disconnected supervisor's OS lease but retain its durable evidence.
    pub fn abandon(&mut self) {
        self.lock.take();
        self.closed = true;
        self.owned = false;
    }
    pub fn close(&mut self) -> StoreResult<()> {
        if self.owned && self.lock.is_some() {
            if self.creator_pid != std::process::id() {
                return Err(session_error("fork child cannot close supervisor session"));
            }
            let _gc = self.store.gc_lock()?;
            unlink_missing(&self.store.lease_path(&self.identifier)?)?;
            fsync_dir(&self.store.base.join("leases"))?;
            self.lock.take();
        }
        self.closed = true;
        Ok(())
    }
    pub fn pin(&self, key: &str) -> StoreResult<Handle> {
        self.store.lookup(key, self)
    }
    pub fn bind(&self, lane: &str, consumer: &str, key: &str) -> StoreResult<PathBuf> {
        self.store.bind(lane, consumer, key, self)
    }
}
impl Drop for Session {
    fn drop(&mut self) {
        if self.creator_pid == std::process::id() {
            let _ = self.close();
        }
    }
}
#[derive(Debug)]
pub struct ConsumerGuard {
    pub store: Store,
    pub lane: String,
    pub consumer: String,
    owner: u32,
    lock: Option<FileLock>,
}
impl ConsumerGuard {
    pub fn close(&mut self) {
        self.lock.take();
    }
}
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct BuildOptions {
    pub verify: bool,
    pub verify_once_per_session: bool,
    pub require_verified: bool,
}
#[derive(Debug)]
pub struct BuildTransaction {
    pub key: String,
    pub stage: PathBuf,
    pub scratch: PathBuf,
    descriptor: Value,
    original: Option<Value>,
    verify: bool,
    repair: bool,
    _action: FileLock,
}
#[derive(Debug)]
pub enum BeginBuild {
    Hit(Handle),
    Build(BuildTransaction),
}
fn text<'a>(v: &'a Value, label: &str) -> StoreResult<&'a str> {
    v.as_str()
        .ok_or_else(|| corrupt(format!("invalid {label}")))
}
fn array<'a>(v: &'a Value, label: &str) -> StoreResult<&'a Vec<Value>> {
    v.as_array()
        .ok_or_else(|| corrupt(format!("invalid {label}")))
}
fn sorted_json(directory: &Path) -> StoreResult<Vec<PathBuf>> {
    let mut v = fs::read_dir(directory)?
        .map(|e| e.map(|e| e.path()))
        .collect::<std::io::Result<Vec<_>>>()?;
    v.retain(|p| p.extension().is_some_and(|e| e == "json"));
    v.sort();
    Ok(v)
}
fn children(directory: &Path) -> StoreResult<Vec<PathBuf>> {
    let mut v = fs::read_dir(directory)?
        .map(|e| e.map(|e| e.path()))
        .collect::<std::io::Result<Vec<_>>>()?;
    v.sort();
    Ok(v)
}
fn unlink_missing(p: &Path) -> StoreResult<()> {
    match fs::remove_file(p) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e.into()),
    }
}
impl Store {
    pub fn new(root: &Path) -> StoreResult<Self> {
        if !root.is_absolute() {
            return Err(error("store root must be explicitly absolute"));
        }
        let store = Self {
            root: root.to_owned(),
            base: root.join("v1"),
        };
        for p in [root.to_owned(), store.base.clone()] {
            mkdir(&p)?;
        }
        for n in [
            "blobs",
            "blobs/sha256",
            "actions",
            "locks",
            "staging",
            "leases",
            "refs",
            "quarantine",
        ] {
            mkdir(&store.base.join(n))?;
        }
        Ok(store)
    }
    pub fn manifest_path(&self, k: &str) -> StoreResult<PathBuf> {
        self.action_path(k)
    }
    pub fn action_path(&self, k: &str) -> StoreResult<PathBuf> {
        Ok(self.base.join("actions").join(format!("{}.json", key(k)?)))
    }
    pub fn action_lock(&self, k: &str) -> StoreResult<PathBuf> {
        Ok(self.base.join("locks").join(format!("{}.lock", key(k)?)))
    }
    pub fn blob(&self, k: &str) -> StoreResult<PathBuf> {
        key(k)?;
        Ok(self.base.join("blobs/sha256").join(&k[..2]).join(k))
    }
    pub fn lease_path(&self, id: &str) -> StoreResult<PathBuf> {
        if id.len() != 32
            || !id
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return Err(session_error("invalid supervisor session id"));
        }
        Ok(self.base.join("leases").join(format!("{id}.json")))
    }
    pub fn lease_lock(&self, id: &str) -> StoreResult<PathBuf> {
        self.lease_path(id)?;
        Ok(self.base.join("locks").join(format!("lease-{id}.lock")))
    }
    pub fn conflict_path(&self, k: &str) -> StoreResult<PathBuf> {
        Ok(self
            .base
            .join("quarantine")
            .join(key(k)?)
            .join("conflict.json"))
    }
    pub fn gc_lock(&self) -> StoreResult<FileLock> {
        Ok(FileLock::acquire(&self.base.join("locks/gc.lock"), false)?.unwrap())
    }
    pub fn session(&self) -> StoreResult<Session> {
        self.session_with_owner(owner()?)
    }
    pub fn session_with_owner(&self, identity: Value) -> StoreResult<Session> {
        if !owner_alive(&identity)? {
            return Err(session_error("supervisor owner is dead/reused"));
        }
        let identifier = random_hex(16)?;
        let token = format!("{identifier}:{}", random_hex(32)?);
        let lock = FileLock::acquire(&self.lease_lock(&identifier)?, false)?.unwrap();
        let session = Session {
            store: self.clone(),
            owner: identity,
            owned: true,
            identifier,
            token,
            closed: false,
            lock: Some(lock),
            creator_pid: std::process::id(),
        };
        {
            let _gc = self.gc_lock()?;
            replace_json(
                &self.lease_path(&session.identifier)?,
                &json!({"schema":SCHEMA,"id":session.identifier,"owner":session.owner,"capability_sha256":hash(session.token.as_bytes()),"actions":[],"forced_actions":{}}),
            )?;
        }
        Ok(session)
    }
    pub fn attach_session(&self, token: &str) -> StoreResult<Session> {
        let Some((id, _)) = token.split_once(':') else {
            return Err(session_error("invalid session capability"));
        };
        let s = Session {
            store: self.clone(),
            owner: owner()?,
            owned: false,
            identifier: id.to_owned(),
            token: token.to_owned(),
            closed: false,
            lock: None,
            creator_pid: std::process::id(),
        };
        {
            let _gc = self.gc_lock()?;
            self.session_record(&s)?;
        }
        Ok(s)
    }
    pub fn lock_held(&self, path: &Path) -> StoreResult<bool> {
        held(path)
    }
    fn validate_lease(&self, record: &Value) -> StoreResult<()> {
        if !fields(
            record,
            &[
                "schema",
                "id",
                "owner",
                "capability_sha256",
                "actions",
                "forced_actions",
            ],
        ) || record["schema"] != SCHEMA
        {
            return Err(corrupt("invalid supervisor lease fields/schema"));
        }
        self.lease_path(text(&record["id"], "supervisor id")?)?;
        owner_alive(&record["owner"])?;
        key(text(&record["capability_sha256"], "capability hash")?)?;
        let actions = array(&record["actions"], "supervisor pin inventory")?;
        let names = actions
            .iter()
            .map(|a| text(a, "supervisor pin inventory"))
            .collect::<StoreResult<Vec<_>>>()?;
        let sorted: BTreeSet<_> = names.iter().copied().collect();
        if names != sorted.into_iter().collect::<Vec<_>>() {
            return Err(corrupt("noncanonical supervisor pin inventory"));
        }
        for a in names {
            key(a)?;
        }
        let forced = record["forced_actions"]
            .as_object()
            .ok_or_else(|| corrupt("invalid supervisor forced-build inventory"))?;
        if forced.iter().any(|(k, v)| {
            !actions.contains(&json!(k)) || !matches!(v.as_str(), Some("built" | "verified"))
        }) {
            return Err(corrupt("invalid supervisor forced-build inventory"));
        }
        Ok(())
    }
    pub fn session_record(&self, s: &Session) -> StoreResult<Value> {
        if s.store.root != self.root || s.closed {
            return Err(session_error("closed session or wrong store"));
        }
        let record = read_json(&self.lease_path(&s.identifier)?).map_err(|e| {
            if e.kind == "CorruptStore" {
                session_error("supervisor session no longer exists")
            } else {
                e
            }
        })?;
        self.validate_lease(&record)?;
        if record["id"] != s.identifier {
            return Err(session_error("invalid supervisor lease identity"));
        }
        if record["capability_sha256"] != hash(s.token.as_bytes()) {
            return Err(session_error("supervisor capability mismatch"));
        }
        if !owner_alive(&record["owner"])? || !held(&self.lease_lock(&s.identifier)?)? {
            return Err(session_error(
                "supervisor owner is dead/reused or its OS lease lock is not held",
            ));
        }
        if s.owned && s.creator_pid != std::process::id() {
            return Err(session_error(
                "fork child must attach to supervisor capability",
            ));
        }
        Ok(record)
    }
    fn pin_locked(&self, s: &Session, k: &str) -> StoreResult<()> {
        key(k)?;
        let mut record = self.session_record(s)?;
        let mut actions: BTreeSet<String> = array(&record["actions"], "pins")?
            .iter()
            .map(|v| text(v, "pin").map(str::to_owned))
            .collect::<StoreResult<_>>()?;
        actions.insert(k.to_owned());
        record["actions"] = json!(actions);
        replace_json(&self.lease_path(&s.identifier)?, &record)
    }
    fn mark_forced_locked(&self, s: &Session, k: &str, status: &str) -> StoreResult<()> {
        let mut record = self.session_record(s)?;
        record["forced_actions"][k] = json!(status);
        replace_json(&self.lease_path(&s.identifier)?, &record)
    }
    fn quarantine_corruption_locked(&self, k: &str, err: &StoreError) -> StoreResult<()> {
        let folder = self
            .base
            .join("quarantine")
            .join(k)
            .join(format!("corruption-{}", random_hex(16)?));
        mkdir(&folder)?;
        let mut evidence = json!({"schema":SCHEMA,"action_key":k,"error":err.message,"observer":owner()?,"created_ns":now_ns(),"outputs":[]});
        let parsed = match read(&self.action_path(k)?) {
            Ok(raw) => {
                write_new(&folder.join("manifest.raw"), &raw)?;
                match serde_json::from_slice::<Value>(&raw) {
                    Ok(v) => v,
                    Err(e) => {
                        evidence["manifest_read_error"] = json!(e.to_string());
                        json!({})
                    }
                }
            }
            Err(e) => {
                evidence["manifest_read_error"] = json!(e.message);
                json!({})
            }
        };
        for output in parsed["outputs"].as_array().into_iter().flatten() {
            let Some(identity) = output["sha256"].as_str() else {
                continue;
            };
            if key(identity).is_err() {
                continue;
            }
            let blob = self.blob(identity)?;
            let mut observed =
                json!({"expected_sha256":identity,"path":blob,"exists":exists(&blob)});
            if exists(&blob) {
                match digest_file(&blob) {
                    Ok((digest, size)) => {
                        observed["observed_sha256"] = json!(digest);
                        observed["observed_bytes"] = json!(size);
                        if digest != identity || output["bytes"] != size {
                            let retained = folder.join(format!("{identity}.corrupt"));
                            if !exists(&retained) {
                                write_new(&retained, &read(&blob)?)?;
                            }
                            observed["retained_file"] =
                                json!(retained.file_name().unwrap().to_string_lossy());
                        }
                    }
                    Err(e) => observed["read_error"] = json!(e.message),
                }
            }
            evidence["outputs"].as_array_mut().unwrap().push(observed);
        }
        write_new(&folder.join("diagnostic.json"), &canonical(&evidence)?)?;
        fsync_dir(&folder)
    }
    fn manifest_locked(
        &self,
        k: &str,
        missing_ok: bool,
        allow_missing: bool,
        verify: bool,
    ) -> StoreResult<Option<Value>> {
        key(k)?;
        match self.read_manifest_locked(k, missing_ok, allow_missing, verify) {
            Err(e) if e.kind == "CorruptStore" => {
                if let Err(q) = self.quarantine_corruption_locked(k, &e) {
                    return Err(corrupt(format!(
                        "{}; corruption evidence failed: {}",
                        e.message, q.message
                    )));
                }
                Err(e)
            }
            v => v,
        }
    }
    fn read_manifest_locked(
        &self,
        k: &str,
        missing_ok: bool,
        allow_missing: bool,
        verify: bool,
    ) -> StoreResult<Option<Value>> {
        key(k)?;
        let cp = self.conflict_path(k)?;
        if exists(&cp) {
            return Err(conflict(format!(
                "action {k} is marked conflicting: {}",
                cp.display()
            )));
        }
        let path = self.action_path(k)?;
        if !exists(&path) {
            return if missing_ok {
                Ok(None)
            } else {
                Err(corrupt(format!(
                    "indexed/referenced action is missing: {k}"
                )))
            };
        }
        let manifest = read_json(&path)?;
        if !fields(
            &manifest,
            &[
                "schema",
                "action_key",
                "descriptor",
                "input_inventory",
                "configuration",
                "outputs",
                "dependency_actions",
                "publication",
            ],
        ) || !manifest["descriptor"].is_object()
        {
            return Err(corrupt(format!(
                "invalid manifest fields: {}",
                path.display()
            )));
        }
        if manifest["schema"] != SCHEMA
            || manifest["action_key"] != k
            || action_key(&manifest["descriptor"])? != k
        {
            return Err(corrupt(format!(
                "action identity/schema mismatch: {}",
                path.display()
            )));
        }
        let outputs = array(&manifest["outputs"], "output inventory")?;
        if outputs.is_empty() {
            return Err(corrupt(format!(
                "missing output inventory: {}",
                path.display()
            )));
        }
        let mut names = vec![];
        for output in outputs {
            let validate = || -> StoreResult<(String, String, u64)> {
                let n = name(text(&output["name"], "output name")?)?.to_owned();
                let d = key(text(&output["sha256"], "output digest")?)?.to_owned();
                if output["role"].as_str().is_none_or(|s| s.is_empty()) {
                    return Err(error("invalid role"));
                }
                let bytes = output["bytes"]
                    .as_u64()
                    .ok_or_else(|| error("invalid size"))?;
                Ok((n, d, bytes))
            };
            let (n, d, bytes) = validate()
                .map_err(|_| corrupt(format!("invalid output inventory: {}", path.display())))?;
            names.push(n);
            let blob = self.blob(&d)?;
            if !verify || (allow_missing && !exists(&blob)) {
                continue;
            }
            if digest_file(&blob)? != (d, bytes) {
                return Err(corrupt(format!(
                    "blob digest/size mismatch: {}",
                    blob.display()
                )));
            }
        }
        if names
            != names
                .iter()
                .cloned()
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>()
        {
            return Err(corrupt(format!(
                "duplicate/unsorted output names: {}",
                path.display()
            )));
        }
        let deps = array(
            &manifest["dependency_actions"],
            "dependency action inventory",
        )?;
        let ds = deps
            .iter()
            .map(|v| text(v, "dependency action inventory"))
            .collect::<StoreResult<Vec<_>>>()?;
        if ds
            != ds
                .iter()
                .copied()
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>()
        {
            return Err(corrupt(format!(
                "invalid dependency action inventory: {}",
                path.display()
            )));
        }
        for d in ds {
            key(d)?;
        }
        Ok(Some(manifest))
    }
    pub fn inspect_manifest(&self, k: &str, s: Option<&Session>) -> StoreResult<Value> {
        let _gc = self.gc_lock()?;
        let m = self.manifest_locked(k, false, false, false)?.unwrap();
        if let Some(s) = s {
            self.pin_locked(s, k)?;
        }
        Ok(m)
    }
    fn verify_tree_locked(
        &self,
        k: &str,
        cache: &mut BTreeMap<String, Value>,
        visiting: &mut BTreeSet<String>,
    ) -> StoreResult<Value> {
        if visiting.contains(k) {
            return Err(corrupt(format!("cyclic dependency action references: {k}")));
        }
        if let Some(v) = cache.get(k) {
            return Ok(v.clone());
        }
        visiting.insert(k.to_owned());
        let m = self.manifest_locked(k, false, false, true)?.unwrap();
        for d in array(&m["dependency_actions"], "dependencies")? {
            self.verify_tree_locked(text(d, "dependency")?, cache, visiting)?;
        }
        visiting.remove(k);
        cache.insert(k.to_owned(), m.clone());
        Ok(m)
    }
    fn verify_tree(&self, k: &str) -> StoreResult<Value> {
        self.verify_tree_locked(k, &mut BTreeMap::new(), &mut BTreeSet::new())
    }
    fn handle(
        &self,
        m: Value,
        hit: bool,
        verified: bool,
        session_verified: bool,
    ) -> StoreResult<Handle> {
        let mut outputs = BTreeMap::new();
        for o in array(&m["outputs"], "outputs")? {
            outputs.insert(
                text(&o["name"], "output name")?.to_owned(),
                self.blob(text(&o["sha256"], "output digest")?)?,
            );
        }
        Ok(Handle {
            action_key: text(&m["action_key"], "action key")?.to_owned(),
            manifest: m,
            outputs,
            hit,
            verified_rebuild: verified,
            session_verified,
        })
    }
    pub fn lookup(&self, k: &str, s: &Session) -> StoreResult<Handle> {
        let _gc = self.gc_lock()?;
        let m = self.verify_tree(k)?;
        self.pin_locked(s, k)?;
        self.handle(m, true, false, false)
    }
    fn stage(
        &self,
        k: &str,
        descriptor: Value,
        original: Option<Value>,
        verify: bool,
        repair: bool,
        action: FileLock,
    ) -> StoreResult<BuildTransaction> {
        let stage = self
            .base
            .join("staging")
            .join(format!("{k}-{}", random_hex(16)?));
        fs::create_dir(&stage)?;
        write_new(
            &stage.join("producer.json"),
            &canonical(&json!({"schema":SCHEMA,"action_key":k,"owner":owner()?}))?,
        )?;
        let scratch = stage.join("worker");
        fs::create_dir(&scratch)?;
        Ok(BuildTransaction {
            key: k.to_owned(),
            stage,
            scratch,
            descriptor,
            original,
            verify,
            repair,
            _action: action,
        })
    }
    pub fn begin_build(
        &self,
        descriptor: &Value,
        s: &Session,
        options: &BuildOptions,
    ) -> StoreResult<BeginBuild> {
        if options.verify_once_per_session && !options.verify {
            return Err(error("verify_once_per_session requires verify=True"));
        }
        if options.require_verified && !options.verify {
            return Err(error("require_verified requires verify=True"));
        }
        let descriptor = descriptor.clone();
        canonical(&descriptor)?;
        let k = action_key(&descriptor)?;
        let action = FileLock::acquire(&self.action_lock(&k)?, false)?.unwrap();
        {
            let _gc = self.gc_lock()?;
            self.pin_locked(s, &k)?;
            let previous = self.manifest_locked(&k, true, false, true)?;
            if previous.is_some() {
                let previous = self.verify_tree(&k)?;
                let record = self.session_record(s)?;
                let forced = record["forced_actions"][&k].as_str();
                let satisfied = forced == Some("verified")
                    || (forced == Some("built") && !options.require_verified);
                if !options.verify || (options.verify_once_per_session && satisfied) {
                    return Ok(BeginBuild::Hit(self.handle(
                        previous,
                        true,
                        false,
                        forced == Some("verified"),
                    )?));
                }
            }
        }
        Ok(BeginBuild::Build(self.stage(
            &k,
            descriptor,
            None,
            options.verify,
            false,
            action,
        )?))
    }
    pub fn begin_repair(&self, k: &str, s: &Session) -> StoreResult<BuildTransaction> {
        key(k)?;
        let action = FileLock::acquire(&self.action_lock(k)?, false)?.unwrap();
        let original;
        {
            let _gc = self.gc_lock()?;
            original = self.manifest_locked(k, false, true, true)?.unwrap();
            if array(&original["outputs"], "outputs")?.iter().all(|o| {
                self.blob(o["sha256"].as_str().unwrap())
                    .is_ok_and(|p| exists(&p))
            }) {
                return Err(error(
                    "repair_missing requires at least one absent indexed blob",
                ));
            }
            self.pin_locked(s, k)?;
        }
        self.stage(
            k,
            original["descriptor"].clone(),
            Some(original),
            true,
            true,
            action,
        )
    }
    fn prepare(&self, tx: &BuildTransaction, built: &BuildResult) -> StoreResult<Value> {
        if built.outputs.is_empty() {
            return Err(error(if tx.repair {
                "repair builder must return nonempty BuildResult"
            } else {
                "builder must return nonempty BuildResult"
            }));
        }
        let mut outputs = vec![];
        for (n, output) in &built.outputs {
            name(n)?;
            let source = if output.path.is_absolute() {
                output.path.clone()
            } else {
                tx.scratch.join(&output.path)
            };
            let resolved = crate::artifact_fs::resolve(&source).map_err(|e| error(e.message))?;
            if !resolved.starts_with(&tx.scratch) {
                return Err(error(format!(
                    "builder output escapes private scratch: {}",
                    source.display()
                )));
            }
            if output.role.is_empty() {
                return Err(error("output role must be nonempty"));
            }
            let target = tx.stage.join(format!("output-{}", outputs.len()));
            let mut incoming = open_regular(&source)?;
            let mut outgoing = OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&target)?;
            let before = stamp(&incoming.metadata()?);
            clone_or_copy(&mut incoming, &mut outgoing)?;
            if before != stamp(&incoming.metadata()?) {
                return Err(error(format!(
                    "builder output changed during copy: {}",
                    source.display()
                )));
            }
            outgoing.flush()?;
            outgoing.sync_all()?;
            fs::set_permissions(&target, fs::Permissions::from_mode(0o444))?;
            let (digest, size) = digest_file(&target)?;
            outputs.push(json!({"name":n,"role":output.role,"sha256":digest,"bytes":size}));
        }
        let mut dependencies = BTreeSet::new();
        for value in &built.dependency_actions {
            dependencies.insert(key(value)?.to_owned());
        }
        if dependencies.contains(&tx.key) {
            return Err(error("action cannot depend on itself"));
        }
        let m = json!({"schema":SCHEMA,"action_key":tx.key,"descriptor":tx.descriptor,"input_inventory":if built.input_inventory.is_null(){tx.descriptor.get("packages").cloned().unwrap_or_else(empty_object)}else{built.input_inventory.clone()},"configuration":tx.descriptor.get("context").cloned().unwrap_or_else(empty_object),"outputs":outputs,"dependency_actions":dependencies,"publication":{"producer":owner()?,"created_ns":now_ns(),"provenance":built.provenance}});
        canonical(&m)?;
        fsync_dir(&tx.stage)?;
        Ok(m)
    }
    fn publish_outputs(&self, stage: &Path, m: &Value) -> StoreResult<()> {
        for (i, output) in array(&m["outputs"], "outputs")?.iter().enumerate() {
            let candidate = stage.join(format!("output-{i}"));
            let digest = text(&output["sha256"], "digest")?;
            let destination = self.blob(digest)?;
            mkdir(destination.parent().unwrap())?;
            let published = rename_noreplace(&candidate, &destination)?;
            #[cfg(test)]
            if published && CRASH_AFTER_BLOB.with(|v| v.get()) {
                std::process::exit(73);
            }
            if !published {
                if digest_file(&destination)?
                    != (
                        digest.to_owned(),
                        output["bytes"]
                            .as_u64()
                            .ok_or_else(|| corrupt("invalid output size"))?,
                    )
                {
                    return Err(corrupt(format!(
                        "existing blob has wrong identity: {}",
                        destination.display()
                    )));
                }
                if !same_bytes(&candidate, &destination)? {
                    return Err(StoreError::new(
                        "BlobCollision",
                        format!("different bytes claim digest {digest}"),
                    ));
                }
                fs::remove_file(&candidate)?;
                fsync_dir(stage)?;
            }
        }
        Ok(())
    }
    fn quarantine_conflict(
        &self,
        stage: &Path,
        k: &str,
        previous: &Value,
        candidate: &Value,
    ) -> StoreResult<()> {
        let folder = self.base.join("quarantine").join(k);
        mkdir(&folder)?;
        let retained = folder.join(stage.file_name().unwrap());
        if !rename_noreplace(stage, &retained)? {
            return Err(corrupt("quarantine destination already exists"));
        }
        let temporary = folder.join(format!(".conflict-{}", random_hex(16)?));
        write_new(
            &temporary,
            &canonical(
                &json!({"schema":SCHEMA,"action_key":k,"previous":previous,"candidate":candidate,"candidate_directory":retained.file_name().unwrap().to_string_lossy()}),
            )?,
        )?;
        if !rename_noreplace(&temporary, &folder.join("conflict.json"))? {
            fs::remove_file(temporary)?;
        }
        Ok(())
    }
    fn finish_inner(
        &self,
        tx: &BuildTransaction,
        built: &BuildResult,
        s: &Session,
    ) -> StoreResult<Handle> {
        let candidate = self.prepare(tx, built)?;
        let handle;
        {
            let _gc = self.gc_lock()?;
            self.session_record(s)?;
            if tx.repair {
                let original = tx
                    .original
                    .as_ref()
                    .ok_or_else(|| error("repair transaction lacks original manifest"))?;
                let current = self.manifest_locked(&tx.key, false, true, true)?.unwrap();
                if &current != original {
                    return Err(corrupt("indexed manifest changed during explicit repair"));
                }
                let mut identical = original["outputs"] == candidate["outputs"]
                    && original["dependency_actions"] == candidate["dependency_actions"];
                if identical {
                    for (i, o) in array(&candidate["outputs"], "outputs")?.iter().enumerate() {
                        let blob = self.blob(text(&o["sha256"], "digest")?)?;
                        if exists(&blob)
                            && !same_bytes(&tx.stage.join(format!("output-{i}")), &blob)?
                        {
                            identical = false;
                            break;
                        }
                    }
                }
                if !identical {
                    self.quarantine_conflict(&tx.stage, &tx.key, original, &candidate)?;
                    return Err(conflict(format!(
                        "repair outputs disagree with indexed identity for {}",
                        tx.key
                    )));
                }
                for d in array(&original["dependency_actions"], "dependencies")? {
                    self.verify_tree(text(d, "dependency")?)?;
                }
                self.publish_outputs(&tx.stage, &candidate)?;
                self.pin_locked(s, &tx.key)?;
                self.mark_forced_locked(s, &tx.key, "verified")?;
                handle = self.handle(original.clone(), false, true, true)?;
            } else {
                let previous = self.manifest_locked(&tx.key, true, false, true)?;
                if let Some(previous) = previous {
                    let mut same = previous["outputs"] == candidate["outputs"]
                        && previous["dependency_actions"] == candidate["dependency_actions"];
                    if same {
                        for (i, o) in array(&candidate["outputs"], "outputs")?.iter().enumerate() {
                            if !same_bytes(
                                &tx.stage.join(format!("output-{i}")),
                                &self.blob(text(&o["sha256"], "digest")?)?,
                            )? {
                                same = false;
                                break;
                            }
                        }
                    }
                    if !same {
                        self.quarantine_conflict(&tx.stage, &tx.key, &previous, &candidate)?;
                        return Err(conflict(format!(
                            "independent output conflict for {}; evidence retained",
                            tx.key
                        )));
                    }
                    self.pin_locked(s, &tx.key)?;
                    if tx.verify {
                        self.mark_forced_locked(s, &tx.key, "verified")?;
                    }
                    handle = self.handle(previous, false, true, true)?;
                } else {
                    for d in array(&candidate["dependency_actions"], "dependencies")? {
                        self.verify_tree(text(d, "dependency")?)?;
                    }
                    self.publish_outputs(&tx.stage, &candidate)?;
                    let temporary = tx.stage.join("action.json");
                    write_new(&temporary, &canonical(&candidate)?)?;
                    if !rename_noreplace(&temporary, &self.action_path(&tx.key)?)? {
                        return Err(corrupt(
                            "action index appeared despite exclusive producer lock",
                        ));
                    }
                    self.pin_locked(s, &tx.key)?;
                    if tx.verify {
                        self.mark_forced_locked(s, &tx.key, "built")?;
                    }
                    handle = self.handle(candidate, false, false, false)?;
                }
            }
        }
        remove_private_tree(&tx.stage)?;
        fsync_dir(&self.base.join("staging"))?;
        Ok(handle)
    }
    fn record_failure(&self, tx: &BuildTransaction, kind: &str, message: &str) -> StoreResult<()> {
        if exists(&tx.stage) {
            replace_json(
                &tx.stage.join("failure.json"),
                &json!({"schema":SCHEMA,"error":kind,"message":message}),
            )?;
        }
        Ok(())
    }
    pub fn finish_build(
        &self,
        tx: BuildTransaction,
        built: BuildResult,
        s: &Session,
    ) -> StoreResult<Handle> {
        match self.finish_inner(&tx, &built, s) {
            Ok(h) => Ok(h),
            Err(e) => {
                self.record_failure(&tx, &e.kind, &e.message)?;
                Err(e)
            }
        }
    }
    pub fn abort_build(&self, tx: BuildTransaction, kind: &str, message: &str) -> StoreResult<()> {
        self.record_failure(&tx, kind, message)
    }
    pub fn lookup_or_build<F>(
        &self,
        descriptor: &Value,
        builder: F,
        s: &Session,
        options: &BuildOptions,
    ) -> StoreResult<Handle>
    where
        F: FnOnce(&Path) -> StoreResult<BuildResult>,
    {
        match self.begin_build(descriptor, s, options)? {
            BeginBuild::Hit(h) => Ok(h),
            BeginBuild::Build(tx) => match builder(&tx.scratch) {
                Ok(b) => self.finish_build(tx, b, s),
                Err(e) => {
                    self.abort_build(tx, &e.kind, &e.message)?;
                    Err(e)
                }
            },
        }
    }
    pub fn repair_missing<F>(&self, k: &str, builder: F, s: &Session) -> StoreResult<Handle>
    where
        F: FnOnce(&Path) -> StoreResult<BuildResult>,
    {
        let tx = self.begin_repair(k, s)?;
        match builder(&tx.scratch) {
            Ok(b) => self.finish_build(tx, b, s),
            Err(e) => {
                self.abort_build(tx, &e.kind, &e.message)?;
                Err(e)
            }
        }
    }
    pub fn ref_id(&self, lane: &str, consumer: &str) -> StoreResult<String> {
        if lane.is_empty() || consumer.is_empty() {
            return Err(error(
                "lane and consumer must be nonempty logical identities",
            ));
        }
        Ok(hash(&canonical(&json!([lane, consumer]))?))
    }
    pub fn ref_path(&self, lane: &str, consumer: &str) -> StoreResult<PathBuf> {
        Ok(self
            .base
            .join("refs")
            .join(format!("{}.json", self.ref_id(lane, consumer)?)))
    }
    pub fn consumer_guard(&self, lane: &str, consumer: &str) -> StoreResult<ConsumerGuard> {
        let id = self.ref_id(lane, consumer)?;
        let lock = FileLock::acquire(
            &self.base.join("locks").join(format!("consumer-{id}.lock")),
            false,
        )?
        .unwrap();
        Ok(ConsumerGuard {
            store: self.clone(),
            lane: lane.to_owned(),
            consumer: consumer.to_owned(),
            owner: std::process::id(),
            lock: Some(lock),
        })
    }
    pub fn bind(&self, lane: &str, consumer: &str, k: &str, s: &Session) -> StoreResult<PathBuf> {
        let _consumer = self.consumer_guard(lane, consumer)?;
        let _gc = self.gc_lock()?;
        self.verify_tree(k)?;
        self.pin_locked(s, k)?;
        let path = self.ref_path(lane, consumer)?;
        replace_json(
            &path,
            &json!({"schema":SCHEMA,"lane":lane,"consumer":consumer,"action_key":key(k)?}),
        )?;
        Ok(path)
    }
    pub fn release_ref(
        &self,
        lane: &str,
        consumer: &str,
        guard: &ConsumerGuard,
    ) -> StoreResult<()> {
        if guard.store.root != self.root
            || guard.lane != lane
            || guard.consumer != consumer
            || guard.lock.is_none()
            || guard.owner != std::process::id()
        {
            return Err(error(
                "release_ref requires the matching held consumer guard",
            ));
        }
        guard.lock.as_ref().unwrap().file.metadata()?;
        let _gc = self.gc_lock()?;
        unlink_missing(&self.ref_path(lane, consumer)?)?;
        fsync_dir(&self.base.join("refs"))
    }
    fn validate_ref(&self, p: &Path, v: &Value) -> StoreResult<()> {
        if !fields(v, &["schema", "lane", "consumer", "action_key"])
            || v["schema"] != SCHEMA
            || v["lane"].as_str().is_none_or(|s| s.is_empty())
            || v["consumer"].as_str().is_none_or(|s| s.is_empty())
        {
            return Err(corrupt(format!(
                "invalid durable reference: {}",
                p.display()
            )));
        }
        if self.ref_path(v["lane"].as_str().unwrap(), v["consumer"].as_str().unwrap())? != p {
            return Err(corrupt(format!(
                "durable reference identity/path mismatch: {}",
                p.display()
            )));
        }
        key(text(&v["action_key"], "reference key")?)?;
        Ok(())
    }
    pub fn list_refs(&self, lane: &str) -> StoreResult<Vec<Value>> {
        if lane.is_empty() {
            return Err(error("lane must be a nonempty logical identity"));
        }
        let _gc = self.gc_lock()?;
        let mut refs = vec![];
        for p in sorted_json(&self.base.join("refs"))? {
            let r = read_json(&p)?;
            self.validate_ref(&p, &r)?;
            if r["lane"] == lane {
                refs.push(r);
            }
        }
        refs.sort_by(|a, b| a["consumer"].as_str().cmp(&b["consumer"].as_str()));
        Ok(refs)
    }
    pub fn restore_refs(&self, lane: &str, s: &Session) -> StoreResult<Vec<Handle>> {
        let _gc = self.gc_lock()?;
        let mut keys = BTreeSet::new();
        for p in sorted_json(&self.base.join("refs"))? {
            let r = read_json(&p)?;
            self.validate_ref(&p, &r)?;
            if r["lane"] == lane {
                keys.insert(text(&r["action_key"], "reference key")?.to_owned());
            }
        }
        let mut cache = BTreeMap::new();
        let mut handles = vec![];
        for k in keys {
            let manifest = self.verify_tree_locked(&k, &mut cache, &mut BTreeSet::new())?;
            self.pin_locked(s, &k)?;
            handles.push(self.handle(manifest, true, false, false)?);
        }
        Ok(handles)
    }
    pub fn status(&self) -> StoreResult<Value> {
        let _gc = self.gc_lock()?;
        let mut report = json!({"schema":SCHEMA,"root":self.root,"verified":false,"roles":{},"logical_bytes":0,"allocated_bytes":0,"unique_inodes":0});
        let mut seen = BTreeSet::new();
        let mut infrastructure = 0;
        for p in [&self.root, &self.base] {
            let m = fs::symlink_metadata(p)?;
            infrastructure += m.blocks() * 512;
            seen.insert((m.dev(), m.ino()));
        }
        report["roles"]["infrastructure"] =
            json!({"files":0,"logical_bytes":0,"allocated_bytes":infrastructure});
        let mut logical = 0;
        let mut allocated = infrastructure;
        for role in [
            "blobs",
            "actions",
            "staging",
            "leases",
            "refs",
            "quarantine",
            "locks",
        ] {
            let root = self.base.join(role);
            let m = fs::symlink_metadata(&root)?;
            seen.insert((m.dev(), m.ino()));
            let mut count = 0;
            let mut bytes = 0;
            let mut blocks = m.blocks() * 512;
            let mut pending = vec![root];
            while let Some(dir) = pending.pop() {
                for path in children(&dir)? {
                    let info = fs::symlink_metadata(&path)?;
                    let scratch_link =
                        ["staging", "quarantine"].contains(&role) && info.file_type().is_symlink();
                    if !(info.is_file() || info.is_dir() || scratch_link) {
                        return Err(corrupt(format!(
                            "non-regular accounting object: {}",
                            path.display()
                        )));
                    }
                    if info.is_dir() {
                        pending.push(path);
                    }
                    if !seen.insert((info.dev(), info.ino())) {
                        continue;
                    }
                    blocks += info.blocks() * 512;
                    if info.is_file() || scratch_link {
                        count += 1;
                        bytes += info.size();
                    }
                }
            }
            report["roles"][role] =
                json!({"files":count,"logical_bytes":bytes,"allocated_bytes":blocks});
            logical += bytes;
            allocated += blocks;
        }
        report["logical_bytes"] = json!(logical);
        report["allocated_bytes"] = json!(allocated);
        report["unique_inodes"] = json!(seen.len());
        Ok(report)
    }
    pub fn gc(
        &self,
        apply: bool,
        grace_seconds: f64,
        mut ref_alive: Option<&mut dyn FnMut(&Value) -> StoreResult<bool>>,
    ) -> StoreResult<Value> {
        if !grace_seconds.is_finite() || grace_seconds < 0.0 {
            return Err(error("GC grace must be finite and nonnegative"));
        }
        let now = now();
        let mut report = json!({"apply":apply,"actions":[],"blobs":[],"staging":[],"stale_leases":[],"dead_refs":[],"protected_actions":[],"lease_anomalies":[]});
        let _gc = self.gc_lock()?;
        let mut roots = BTreeSet::new();
        for p in sorted_json(&self.base.join("leases"))? {
            let lease = read_json(&p)?;
            self.validate_lease(&lease)?;
            let id = text(&lease["id"], "lease id")?;
            if lease["schema"] != SCHEMA || self.lease_path(id)? != p {
                return Err(corrupt(format!("invalid lease: {}", p.display())));
            }
            let lock = held(&self.lease_lock(id)?)?;
            let alive = owner_alive(&lease["owner"])?;
            if lock || alive {
                for k in array(&lease["actions"], "lease actions")? {
                    roots.insert(key(text(k, "lease key")?)?.to_owned());
                }
                if lock != alive {
                    report["lease_anomalies"]
                        .as_array_mut()
                        .unwrap()
                        .push(json!({"id":id,"held_lock":lock,"same_owner_alive":alive}));
                }
            } else {
                push_path(&mut report, "stale_leases", &p);
            }
        }
        let mut manifests = BTreeMap::new();
        for p in sorted_json(&self.base.join("actions"))? {
            let k = key(p
                .file_stem()
                .and_then(|s| s.to_str())
                .ok_or_else(|| corrupt("invalid action filename"))?)?;
            manifests.insert(
                k.to_owned(),
                self.manifest_locked(k, false, false, false)?.unwrap(),
            );
            if now - mtime(&fs::metadata(&p)?) < grace_seconds {
                roots.insert(k.to_owned());
            }
        }
        for p in sorted_json(&self.base.join("refs"))? {
            let reference = read_json(&p)?;
            self.validate_ref(&p, &reference)?;
            if now - mtime(&fs::metadata(&p)?) >= grace_seconds {
                if let Some(callback) = ref_alive.as_mut() {
                    if !callback(&reference)? {
                        push_path(&mut report, "dead_refs", &p);
                        continue;
                    }
                }
            }
            let k = text(&reference["action_key"], "reference key")?;
            if !manifests.contains_key(k) {
                self.manifest_locked(k, false, false, false)?;
            }
            roots.insert(k.to_owned());
        }
        for m in manifests.values() {
            if array(&m["dependency_actions"], "dependencies")?
                .iter()
                .any(|d| d.as_str().is_none_or(|s| !manifests.contains_key(s)))
            {
                return Err(corrupt(
                    "indexed manifest references a missing dependency action",
                ));
            }
        }
        let mut protected = BTreeSet::new();
        let mut pending: Vec<_> = roots.into_iter().collect();
        while let Some(k) = pending.pop() {
            if !protected.insert(k.clone()) {
                continue;
            }
            if let Some(m) = manifests.get(&k) {
                for d in array(&m["dependency_actions"], "dependencies")? {
                    let d = text(d, "dependency")?;
                    if !manifests.contains_key(d) {
                        return Err(corrupt(format!("missing dependency action {d}")));
                    }
                    pending.push(d.to_owned());
                }
            }
        }
        report["protected_actions"] = json!(protected);
        let mut keep_blobs = BTreeSet::new();
        for k in &protected {
            if let Some(m) = manifests.get(k) {
                for o in array(&m["outputs"], "outputs")? {
                    let digest = text(&o["sha256"], "digest")?;
                    if !exists(&self.blob(digest)?) {
                        return Err(corrupt(format!("protected action {k} is missing a blob")));
                    }
                    keep_blobs.insert(digest.to_owned());
                }
            }
        }
        for k in manifests.keys().filter(|k| !protected.contains(*k)) {
            push_path(&mut report, "actions", &self.action_path(k)?);
        }
        for prefix in children(&self.base.join("blobs/sha256"))? {
            if !fs::symlink_metadata(&prefix)?.is_dir() {
                return Err(corrupt(format!(
                    "invalid blob prefix: {}",
                    prefix.display()
                )));
            }
            for p in children(&prefix)? {
                let digest = p
                    .file_name()
                    .and_then(|n| n.to_str())
                    .ok_or_else(|| corrupt("invalid blob filename"))?;
                if p != self.blob(digest)? {
                    return Err(corrupt(format!("invalid blob path: {}", p.display())));
                }
                if keep_blobs.contains(digest) || now - mtime(&fs::metadata(&p)?) < grace_seconds {
                    continue;
                }
                if digest_file(&p)?.0 != digest {
                    return Err(corrupt(format!(
                        "orphan/indexed blob digest mismatch: {}",
                        p.display()
                    )));
                }
                push_path(&mut report, "blobs", &p);
            }
        }
        for stage in children(&self.base.join("staging"))? {
            if !fs::symlink_metadata(&stage)?.is_dir() {
                return Err(corrupt(format!(
                    "non-directory staging object: {}",
                    stage.display()
                )));
            }
            let producer = stage.join("producer.json");
            if !exists(&producer) {
                continue;
            }
            let producer = read_json(&producer)?;
            if producer["schema"] != SCHEMA {
                return Err(corrupt(format!(
                    "invalid staging producer: {}",
                    stage.display()
                )));
            }
            if held(&self.action_lock(text(&producer["action_key"], "producer action")?)?)? {
                continue;
            }
            if owner_alive(&producer["owner"])? && !exists(&stage.join("failure.json")) {
                continue;
            }
            if now - mtime(&fs::metadata(&stage)?) >= grace_seconds {
                push_path(&mut report, "staging", &stage);
            }
        }
        if apply {
            for p in array(&report["dead_refs"], "dead refs")? {
                unlink_missing(Path::new(text(p, "dead ref")?))?;
            }
            if !array(&report["dead_refs"], "dead refs")?.is_empty() {
                fsync_dir(&self.base.join("refs"))?;
            }
            for p in array(&report["actions"], "actions")? {
                fs::remove_file(text(p, "action")?)?;
            }
            fsync_dir(&self.base.join("actions"))?;
            for role in ["blobs", "stale_leases"] {
                for p in array(&report[role], role)? {
                    fs::remove_file(text(p, role)?)?;
                }
            }
            for p in array(&report["staging"], "staging")? {
                remove_private_tree(Path::new(text(p, "stage")?))?;
            }
            for p in [
                self.base.join("actions"),
                self.base.join("leases"),
                self.base.join("staging"),
            ]
            .into_iter()
            .chain(children(&self.base.join("blobs/sha256"))?)
            {
                fsync_dir(&p)?;
            }
        }
        Ok(report)
    }
}
fn mtime(m: &fs::Metadata) -> f64 {
    m.mtime() as f64 + m.mtime_nsec() as f64 / 1_000_000_000.0
}
fn push_path(v: &mut Value, k: &str, p: &Path) {
    v[k].as_array_mut().unwrap().push(json!(p));
}

#[cfg(test)]
mod tests {
    use super::*;
    fn build(scratch: &Path, bytes: &[u8]) -> StoreResult<BuildResult> {
        let p = scratch.join("body.a");
        fs::write(&p, bytes)?;
        Ok(BuildResult {
            outputs: BTreeMap::from([("body.a".into(), Output::archive(p))]),
            provenance: json!({"fixture":true}),
            dependency_actions: vec![],
            input_inventory: Value::Null,
        })
    }
    fn fixture() -> (tempfile::TempDir, Store) {
        let root = tempfile::tempdir().unwrap();
        let store = Store::new(&root.path().join("store")).unwrap();
        (root, store)
    }
    #[test]
    fn publication_hit_verification_and_gc_keep_bytes_immutable() {
        let (_root, store) = fixture();
        let descriptor = json!({"fixture":1});
        let mut session = store.session().unwrap();
        let first = store
            .lookup_or_build(
                &descriptor,
                |p| build(p, b"archive"),
                &session,
                &BuildOptions::default(),
            )
            .unwrap();
        assert!(!first.hit);
        assert_eq!(fs::read(first.blob_path().unwrap()).unwrap(), b"archive");
        let metadata = fs::metadata(first.blob_path().unwrap()).unwrap();
        assert_eq!(metadata.nlink(), 1);
        assert_eq!(metadata.mode() & 0o222, 0);
        let hit = store
            .lookup_or_build(
                &descriptor,
                |_| panic!("hit compiled"),
                &session,
                &BuildOptions::default(),
            )
            .unwrap();
        assert!(hit.hit);
        assert_eq!(first.manifest, hit.manifest);
        let verified = store
            .lookup_or_build(
                &descriptor,
                |p| build(p, b"archive"),
                &session,
                &BuildOptions {
                    verify: true,
                    ..Default::default()
                },
            )
            .unwrap();
        assert!(verified.verified_rebuild && verified.session_verified && !verified.hit);
        let again = store
            .lookup_or_build(
                &descriptor,
                |_| panic!("session verification repeated"),
                &session,
                &BuildOptions {
                    verify: true,
                    verify_once_per_session: true,
                    ..Default::default()
                },
            )
            .unwrap();
        assert!(again.hit && again.session_verified);
        DIGEST_READS.with(|n| n.set(0));
        assert!(store.gc(false, 0.0, None).unwrap()["blobs"]
            .as_array()
            .unwrap()
            .is_empty());
        assert_eq!(DIGEST_READS.with(|n| n.get()), 0);
        session.close().unwrap();
        let report = store.gc(true, 0.0, None).unwrap();
        assert_eq!(report["actions"].as_array().unwrap().len(), 1);
        assert_eq!(report["blobs"].as_array().unwrap().len(), 1);
        assert!(!first.blob_path().unwrap().exists());
        assert!(store.action_lock(&first.action_key).unwrap().exists());
    }
    #[test]
    fn digest_collisions_compare_bytes_and_cannot_replace_the_first_blob() {
        let (_root, store) = fixture();
        let session = store.session().unwrap();
        struct Reset;
        impl Drop for Reset {
            fn drop(&mut self) {
                DIGEST_OVERRIDE.with(|v| *v.borrow_mut() = None);
            }
        }
        let _reset = Reset;
        DIGEST_OVERRIDE.with(|v| *v.borrow_mut() = Some(("a".repeat(64), 4)));
        let first = store
            .lookup_or_build(
                &json!({"variant":1}),
                |p| build(p, b"aaaa"),
                &session,
                &BuildOptions::default(),
            )
            .unwrap();
        let second = json!({"variant":2});
        let err = store
            .lookup_or_build(
                &second,
                |p| build(p, b"bbbb"),
                &session,
                &BuildOptions::default(),
            )
            .unwrap_err();
        assert_eq!(err.kind, "BlobCollision");
        assert_eq!(fs::read(first.blob_path().unwrap()).unwrap(), b"aaaa");
        assert!(!store
            .action_path(&action_key(&second).unwrap())
            .unwrap()
            .exists());
    }
    #[test]
    fn disk_full_during_manifest_write_exposes_no_action() {
        let (_root, store) = fixture();
        let session = store.session().unwrap();
        struct Reset;
        impl Drop for Reset {
            fn drop(&mut self) {
                FAIL_ACTION_WRITE.with(|v| v.set(false));
            }
        }
        let _reset = Reset;
        FAIL_ACTION_WRITE.with(|v| v.set(true));
        let d = json!({"disk-full":true});
        let err = store
            .lookup_or_build(
                &d,
                |p| build(p, b"orphan output"),
                &session,
                &BuildOptions::default(),
            )
            .unwrap_err();
        assert!(err.message.contains("No space"));
        assert!(!store
            .action_path(&action_key(&d).unwrap())
            .unwrap()
            .exists());
        let report = store.gc(true, 0.0, None).unwrap();
        assert_eq!(report["staging"].as_array().unwrap().len(), 1);
        assert_eq!(report["blobs"].as_array().unwrap().len(), 1);
    }
    #[test]
    fn copy_range_fallback_restarts_both_files_only_for_allowed_errors() {
        let root = tempfile::tempdir().unwrap();
        let source = root.path().join("in");
        let dest = root.path().join("out");
        fs::write(&source, b"whole source").unwrap();
        for errno in [libc::EXDEV, libc::EINVAL, libc::ENOSYS, libc::EOPNOTSUPP] {
            let mut incoming = File::open(&source).unwrap();
            incoming.seek(SeekFrom::Start(3)).unwrap();
            let mut outgoing = OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open(&dest)
                .unwrap();
            outgoing.write_all(b"partial trash").unwrap();
            clone_or_copy_with(&mut incoming, &mut outgoing, |_, _| {
                Err(std::io::Error::from_raw_os_error(errno))
            })
            .unwrap();
            assert_eq!(fs::read(&dest).unwrap(), b"whole source");
        }
        for errno in [libc::ENOSPC, libc::EIO] {
            let mut incoming = File::open(&source).unwrap();
            let mut outgoing = OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open(&dest)
                .unwrap();
            outgoing.write_all(b"partial").unwrap();
            assert!(clone_or_copy_with(&mut incoming, &mut outgoing, |_, _| Err(
                std::io::Error::from_raw_os_error(errno)
            ))
            .is_err());
            assert_eq!(fs::read(&dest).unwrap(), b"partial");
        }
    }
    #[test]
    fn restore_refs_hashes_each_distinct_action_once_and_gc_can_drop_dead_refs() {
        let (_root, store) = fixture();
        let mut session = store.session().unwrap();
        let h = store
            .lookup_or_build(
                &json!({"shared":true}),
                |p| build(p, b"shared"),
                &session,
                &BuildOptions::default(),
            )
            .unwrap();
        session.bind("lane", "one", &h.action_key).unwrap();
        session.bind("lane", "two", &h.action_key).unwrap();
        DIGEST_READS.with(|n| n.set(0));
        let handles = store.restore_refs("lane", &session).unwrap();
        assert_eq!(handles.len(), 1);
        assert_eq!(DIGEST_READS.with(|n| n.get()), 1);
        assert_eq!(store.list_refs("lane").unwrap().len(), 2);
        session.close().unwrap();
        let mut dead = |_: &Value| Ok(false);
        let report = store.gc(true, 0.0, Some(&mut dead)).unwrap();
        assert_eq!(report["dead_refs"].as_array().unwrap().len(), 2);
        assert_eq!(report["actions"].as_array().unwrap().len(), 1);
    }
    #[test]
    fn repair_preserves_the_exact_index_and_conflict_is_persistent() {
        let (_root, store) = fixture();
        let session = store.session().unwrap();
        let d = json!({"repair":true});
        let h = store
            .lookup_or_build(
                &d,
                |p| build(p, b"same"),
                &session,
                &BuildOptions::default(),
            )
            .unwrap();
        let bytes = fs::read(store.action_path(&h.action_key).unwrap()).unwrap();
        fs::remove_file(h.blob_path().unwrap()).unwrap();
        let repair = store
            .repair_missing(&h.action_key, |p| build(p, b"same"), &session)
            .unwrap();
        assert!(repair.verified_rebuild && repair.session_verified);
        assert_eq!(
            fs::read(store.action_path(&h.action_key).unwrap()).unwrap(),
            bytes
        );
        let err = store
            .lookup_or_build(
                &d,
                |p| build(p, b"different"),
                &session,
                &BuildOptions {
                    verify: true,
                    ..Default::default()
                },
            )
            .unwrap_err();
        assert_eq!(err.kind, "ActionConflict");
        assert_eq!(fs::read(h.blob_path().unwrap()).unwrap(), b"same");
        assert_eq!(
            store
                .lookup_or_build(
                    &d,
                    |_| panic!("conflicting action rebuilt"),
                    &session,
                    &BuildOptions::default()
                )
                .unwrap_err()
                .kind,
            "ActionConflict"
        );
        assert!(store.conflict_path(&h.action_key).unwrap().exists());
    }
    #[test]
    fn bad_capabilities_and_missing_os_lease_do_not_authorize_reads() {
        let (_root, store) = fixture();
        let session = store.session().unwrap();
        assert_eq!(
            store
                .attach_session(&format!("{}:wrong", session.identifier))
                .unwrap_err()
                .kind,
            "SessionError"
        );
        let p = store.lease_path(&session.identifier).unwrap();
        let mut record = read_json(&p).unwrap();
        record["owner"]["start_identity"]["start_ticks"] = json!("not-current");
        replace_json(&p, &record).unwrap();
        assert_eq!(
            store.attach_session(&session.token).unwrap_err().kind,
            "SessionError"
        );
        let report = store.gc(true, 0.0, None).unwrap();
        assert!(report["stale_leases"].as_array().unwrap().is_empty());
        assert_eq!(report["lease_anomalies"].as_array().unwrap().len(), 1);
    }
    #[test]
    fn gc_does_not_wait_for_a_live_producer_and_abort_bounds_scratch_retention() {
        let (_root, store) = fixture();
        let session = store.session().unwrap();
        let BeginBuild::Build(tx) = store
            .begin_build(&json!({"producer":1}), &session, &BuildOptions::default())
            .unwrap()
        else {
            panic!("unexpected hit")
        };
        let path = tx.stage.clone();
        let copy = store.clone();
        let started = std::time::Instant::now();
        let report = std::thread::spawn(move || copy.gc(true, 0.0, None).unwrap())
            .join()
            .unwrap();
        assert!(started.elapsed().as_secs() < 5);
        assert!(report["staging"].as_array().unwrap().is_empty());
        store
            .abort_build(tx, "FixtureError", "producer failed")
            .unwrap();
        let report = store.gc(true, 0.0, None).unwrap();
        assert_eq!(report["staging"].as_array().unwrap().len(), 1);
        assert!(!path.exists());
    }

    #[test]
    fn abandoned_lease_retains_evidence_until_both_lock_and_owner_are_gone() {
        let (_root, store) = fixture();
        let mut session = store.session().unwrap();
        let handle = store
            .lookup_or_build(
                &json!({"abandon":true}),
                |p| build(p, b"retained"),
                &session,
                &BuildOptions::default(),
            )
            .unwrap();
        let path = store.lease_path(&session.identifier).unwrap();
        session.abandon();
        assert!(path.exists());
        let report = store.gc(true, 0.0, None).unwrap();
        assert!(report["stale_leases"].as_array().unwrap().is_empty());
        assert_eq!(report["lease_anomalies"].as_array().unwrap().len(), 1);
        assert!(handle.blob_path().unwrap().exists());
        let mut record = read_json(&path).unwrap();
        record["owner"]["start_identity"]["start_ticks"] = json!("reused-pid");
        replace_json(&path, &record).unwrap();
        let report = store.gc(true, 0.0, None).unwrap();
        assert_eq!(report["stale_leases"].as_array().unwrap().len(), 1);
        assert!(!handle.blob_path().unwrap().exists());
    }

    #[test]
    #[ignore = "subprocess crash worker; invoked by publication_crash_exposes_no_index"]
    fn crash_worker() {
        let root = std::env::var_os("VIBEOS_STORE_TEST_CRASH_ROOT").expect("private test root");
        let store = Store::new(Path::new(&root)).unwrap();
        let session = store.session().unwrap();
        CRASH_AFTER_BLOB.with(|v| v.set(true));
        store
            .lookup_or_build(
                &json!({"crash":true}),
                |p| build(p, b"orphan from crashed publisher"),
                &session,
                &BuildOptions::default(),
            )
            .unwrap();
        panic!("crash hook did not exit");
    }

    #[test]
    fn publication_crash_exposes_no_index_and_gc_recovers_without_waiting() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("store");
        let status = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "artifact_store::tests::crash_worker",
                "--ignored",
            ])
            .env("VIBEOS_STORE_TEST_CRASH_ROOT", &path)
            .status()
            .unwrap();
        assert_eq!(status.code(), Some(73));
        let store = Store::new(&path).unwrap();
        assert!(!store
            .action_path(&action_key(&json!({"crash":true})).unwrap())
            .unwrap()
            .exists());
        let report = store.gc(true, 0.0, None).unwrap();
        assert_eq!(report["staging"].as_array().unwrap().len(), 1);
        assert_eq!(report["stale_leases"].as_array().unwrap().len(), 1);
        assert_eq!(report["blobs"].as_array().unwrap().len(), 1);
    }
}
