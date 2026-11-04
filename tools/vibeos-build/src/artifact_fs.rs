//! Filesystem identity shared by artifact audits and verified captures.
use crate::{Error, Result};
use std::{
    collections::VecDeque,
    fs,
    io::Read,
    os::unix::{
        ffi::OsStrExt,
        fs::{MetadataExt, OpenOptionsExt},
    },
    path::{Component, Path, PathBuf},
};

/// Resolve existing symlink components even when the last component does not exist.
pub fn resolve(path: &Path) -> Result<PathBuf> {
    let path = if path.is_absolute() {
        path.to_owned()
    } else {
        std::env::current_dir()?.join(path)
    };
    let mut queue: VecDeque<_> = path
        .components()
        .map(|c| c.as_os_str().to_owned())
        .collect();
    let mut out = PathBuf::new();
    let mut links = 0;
    while let Some(part) = queue.pop_front() {
        if part == "/" {
            out = PathBuf::from("/");
        } else if part == "." {
        } else if part == ".." {
            out.pop();
        } else {
            out.push(&part);
            match fs::symlink_metadata(&out) {
                Ok(m) if m.file_type().is_symlink() => {
                    links += 1;
                    if links > 40 {
                        return Err(Error::new(format!("symlink loop: {}", path.display())));
                    }
                    let target = fs::read_link(&out)?;
                    out.pop();
                    for c in target.components().rev() {
                        queue.push_front(c.as_os_str().to_owned());
                    }
                }
                Ok(_) => {}
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => return Err(e.into()),
            }
        }
    }
    Ok(out)
}
pub fn same_file(a: &Path, b: &Path) -> bool {
    match (fs::metadata(a), fs::metadata(b)) {
        (Ok(a), Ok(b)) => a.dev() == b.dev() && a.ino() == b.ino(),
        _ => false,
    }
}
pub fn stamp(m: &fs::Metadata) -> [i128; 5] {
    [
        m.dev() as i128,
        m.ino() as i128,
        m.size() as i128,
        m.mtime() as i128 * 1_000_000_000 + m.mtime_nsec() as i128,
        m.ctime() as i128 * 1_000_000_000 + m.ctime_nsec() as i128,
    ]
}
pub fn verified_read(path: &Path, what: &str) -> Result<Vec<u8>> {
    let mut file = fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC | libc::O_NONBLOCK)
        .open(path)?;
    let before = file.metadata()?;
    if !before.is_file() {
        return Err(Error::new(format!(
            "{what} is not a regular file: {}",
            path.display()
        )));
    }
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    let after = file.metadata()?;
    let at_path = fs::symlink_metadata(path)?;
    if stamp(&before) != stamp(&after)
        || stamp(&after) != stamp(&at_path)
        || at_path.file_type().is_symlink()
    {
        return Err(Error::new(format!(
            "{what} changed during audit: {}",
            path.display()
        )));
    }
    Ok(bytes)
}
pub fn safe_relative(path: &Path) -> bool {
    !path.is_absolute()
        && !path.as_os_str().as_bytes().contains(&0)
        && !path.components().any(|c| matches!(c, Component::ParentDir))
}
