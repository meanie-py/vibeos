//! Shared plumbing for the whole `filesystem` batch: the errno/doserrno
//! validation helpers every export in this directory uses, the Win32 externs
//! shared by more than one sibling file, and a thin `wide_ptr` accessor over
//! `crate::findfile::Buf`.
//!
//! The narrow<->wide path conversion itself (`file_codepage`, `mbs_to_wcs`,
//! `wcs_to_mbs`, `os_errno`, `Buf<C>`) is `crate::findfile`'s -- `filesystem`
//! now depends on the `findfile` Cargo feature (see `Cargo.toml`), and those
//! five items are `pub(crate)` there, so every sibling file in this module
//! imports them directly instead of this file re-implementing them.
//! `fullpath.rs` is the one exception: it needs its own alloc/detach and its
//! own no-resize conversion, neither of which `findfile.rs` offers (see that
//! file's module doc for exactly how and why).

use core::ffi::{c_char, c_int, c_void};

pub(super) const EINVAL: c_int = 22;
pub(super) const EACCES: c_int = 13; // nolint: const-contradiction (matches misc/getcwd.rs's own EACCES; the repo-wide contradiction is freeldr-rust's unrelated bootloader EACCES=0x2)
pub(super) const ERANGE: c_int = 34;

pub(super) const ERROR_ACCESS_DENIED: u32 = 5;

pub(super) const FILE_ATTRIBUTE_READONLY: u32 = 0x01;
pub(super) const FILE_ATTRIBUTE_DIRECTORY: u32 = 0x10;

/// `GetFileExInfoStandard`.
pub(super) const GET_FILE_EX_INFO_STANDARD: c_int = 0;

/// `WIN32_FILE_ATTRIBUTE_DATA`. Only `file_attributes` is ever read; the rest
/// exists so the struct is the right size for `GetFileAttributesExW` to write.
#[repr(C)]
pub(super) struct Win32FileAttributeData {
    pub(super) file_attributes: u32,
    creation_time: [u32; 2],
    last_access_time: [u32; 2],
    last_write_time: [u32; 2],
    file_size_high: u32,
    file_size_low: u32,
}
const _: () = assert!(core::mem::size_of::<Win32FileAttributeData>() == 36);

extern "system" {
    pub(super) fn GetFileAttributesExW(
        name: *const u16,
        level: c_int,
        info: *mut Win32FileAttributeData,
    ) -> c_int;
    pub(super) fn SetFileAttributesW(name: *const u16, attributes: u32) -> c_int;
    pub(super) fn CreateDirectoryW(name: *const u16, security: *mut c_void) -> c_int;
    pub(super) fn RemoveDirectoryW(name: *const u16) -> c_int;
    pub(super) fn DeleteFileW(name: *const u16) -> c_int;
    pub(super) fn MoveFileExW(old_name: *const u16, new_name: *const u16, flags: u32) -> c_int;
    pub(super) fn GetFullPathNameW(
        file_name: *const u16,
        buffer_length: u32,
        buffer: *mut u16,
        file_part: *mut *mut u16,
    ) -> u32;
}

extern "C" {
    pub(super) fn _errno() -> *mut c_int;
    pub(super) fn __doserrno() -> *mut u32;
    pub(super) fn _invalid_parameter_noinfo();
    pub(super) fn __acrt_WideCharToMultiByte(
        cp: u32,
        flags: u32,
        src: *const u16,
        src_len: c_int,
        dst: *mut c_char,
        dst_len: c_int,
        default_char: *const c_char,
        used_default: *mut c_int,
    ) -> c_int;
    pub(super) fn _getcwd(buf: *mut u8, size: c_int) -> *mut u8;
    pub(super) fn _wgetcwd(buf: *mut u16, size: c_int) -> *mut u16;
}

/// `_VALIDATE_RETURN(expr, errorcode, retexpr)`'s failure arm in a release
/// build: `_INVALID_PARAMETER` is `_invalid_parameter_noinfo`, no extra info.
#[cold]
pub(super) unsafe fn invalid_parameter(err: c_int) {
    *_errno() = err;
    _invalid_parameter_noinfo();
}

/// `_VALIDATE_CLEAR_OSSERR_RETURN(expr, errorcode, retexpr)`'s failure arm.
#[cold]
pub(super) unsafe fn invalid_parameter_clear_oss(err: c_int) {
    *__doserrno() = 0;
    *_errno() = err;
    _invalid_parameter_noinfo();
}

/// A converted wide buffer's data pointer, or NULL for the `None` (input was
/// NULL) case -- `wide_path.data()` on a buffer that had `set_to_nullptr`
/// called on it (`crate::findfile::mbs_to_wcs`'s `Ok(None)` arm).
pub(super) fn wide_ptr(b: &Option<crate::findfile::Buf<u16>>) -> *const u16 {
    match b {
        Some(buf) => buf.0,
        None => core::ptr::null(),
    }
}
