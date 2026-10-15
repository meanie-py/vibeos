//! wer-rust: `wer.dll` — the Windows Error Reporting API, for an OS with no WER service.
//! A port of Wine's dlls/wer/main.c (the 9 implemented exports), same HRESULTs, same report
//! table semantics. Every `Werp*` / `Wer*Sysprep*` entry stays `@ stub`, exactly as in
//! Wine's wer.spec.
//!
//! Semantics (all copied from Wine unless marked DIVERGES):
//!  * WerReportCreate: `*phandle` is nulled first (when non-NULL); NULL/empty event type, NULL
//!    phandle or `reporttype >= WerReportInvalid` (5) = E_INVALIDARG (the type check matches
//!    recent Windows 10, per Wine's test). The report is a heap block {link, WER_REPORT_INFORMATION
//!    copy or zeros, type, event type string} chained into a process-wide list; the HREPORT is
//!    that block's address. Allocation failure = HRESULT_FROM_WIN32(ERROR_OUTOFMEMORY).
//!  * WerReportCloseHandle: E_INVALIDARG for NULL, an unknown handle, or a double close (the
//!    handle is looked up in the list; nothing is dereferenced before it is found).
//!  * WerReportSubmit: this OS has no WER service, so the machine behaves as if error reporting
//!    were DISABLED by policy: `*presult = WerDisabled (5)` and S_OK, nothing is queued, no UI,
//!    no restart/terminate. NULL `presult` = E_INVALIDARG. As in Wine the handle, consent and
//!    flags are NOT validated (unmeasured on Windows, so no guess is made).
//!  * WerAddExcludedApplication / WerRemoveExcludedApplication: the base name (after the last
//!    `\`) is written/deleted as a REG_DWORD 1 under
//!    `Software\Microsoft\Windows Error Reporting\ExcludedApplications` in HKLM (allUsers) or
//!    HKCU. NULL/empty name or a trailing `\` = E_INVALIDARG; unwritable key = E_ACCESSDENIED;
//!    Remove of an absent value = HRESULT_FROM_WIN32(ERROR_ENVVAR_NOT_FOUND).
//!  * WerReportAddFile / WerReportSetParameter: accepted and ignored, S_OK (Wine stubs).
//!    WerReportAddDump / WerReportSetUIOption: E_NOTIMPL (Wine stubs).
//!
//! DIVERGES from Wine: allocation uses the process heap (no msvcrt calloc/free), the report-table
//! lock is a spin lock (no kernel32 critical section / DllMain). Like Wine, a caller-supplied
//! WER_REPORT_INFORMATION is copied whole without consulting its dwSize.

#![cfg_attr(not(test), no_std)]
#![allow(non_camel_case_types, non_snake_case, clippy::missing_safety_doc)]

use core::ffi::c_void;
use core::mem::{offset_of, size_of};
use core::ptr::{copy_nonoverlapping, null_mut};

mod plat;
#[cfg(test)]
mod tests;

pub type HRESULT = i32;
pub const S_OK: HRESULT = 0;
pub const E_NOTIMPL: HRESULT = 0x8000_4001u32 as i32;
pub const E_INVALIDARG: HRESULT = 0x8007_0057u32 as i32;
pub const E_ACCESSDENIED: HRESULT = 0x8007_0005u32 as i32;
/// HRESULT_FROM_WIN32(ERROR_OUTOFMEMORY = 14)
pub const HR_OUTOFMEMORY: HRESULT = 0x8007_000Eu32 as i32;
/// HRESULT_FROM_WIN32(ERROR_ENVVAR_NOT_FOUND = 203)
pub const HR_ENVVAR_NOT_FOUND: HRESULT = 0x8007_00CBu32 as i32;

pub const WER_REPORT_INVALID: i32 = 5; // WerReportInvalid
pub const WER_DISABLED: i32 = 5; // WER_SUBMIT_RESULT::WerDisabled

/// WER_REPORT_INFORMATION (werapi.h).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct WerReportInformation {
    pub dw_size: u32,
    pub h_process: *mut c_void,
    pub consent_key: [u16; 64],
    pub friendly_event_name: [u16; 128],
    pub application_name: [u16; 128],
    pub application_path: [u16; 260],
    pub description: [u16; 512],
    pub hwnd_parent: *mut c_void,
}

#[repr(C)]
struct Report {
    next: *mut Report,
    info: WerReportInformation,
    report_type: i32,
    event_type: [u16; 1], // NUL-terminated copy continues past the struct
}

static TABLE_LOCK: plat::Lock = plat::Lock::new();
static mut TABLE: *mut Report = null_mut(); // guarded by TABLE_LOCK

const EXCLUDE_KEY: [u16; 64] = utf16z::<63, 64>(b"Software\\Microsoft\\Windows Error Reporting\\ExcludedApplications");

/// Widen an ASCII literal at compile time into a NUL-terminated UTF-16 array.
const fn utf16z<const N: usize, const M: usize>(s: &[u8; N]) -> [u16; M] {
    let mut o = [0u16; M];
    let mut i = 0;
    while i < N {
        o[i] = s[i] as u16;
        i += 1;
    }
    o
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

// staticlib needs an eh personality (no_std cohab).
#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}

unsafe fn wcslen(p: *const u16) -> usize {
    let mut n = 0;
    while *p.add(n) != 0 {
        n += 1;
    }
    n
}

/// The base name after the last backslash, or None for NULL / empty / trailing-backslash.
unsafe fn exe_base(exe: *const u16) -> Option<*const u16> {
    if exe.is_null() || *exe == 0 {
        return None;
    }
    let len = wcslen(exe);
    let mut i = len;
    while i > 0 {
        if *exe.add(i - 1) == b'\\' as u16 {
            return if i == len { None } else { Some(exe.add(i)) };
        }
        i -= 1;
    }
    Some(exe)
}

#[no_mangle]
pub unsafe extern "system" fn WerAddExcludedApplication(exe_name: *const u16, all_users: i32) -> HRESULT {
    let Some(base) = exe_base(exe_name) else { return E_INVALIDARG };
    match plat::exclude_set(all_users != 0, EXCLUDE_KEY.as_ptr(), base) {
        Some(()) => S_OK,
        None => E_ACCESSDENIED,
    }
}

#[no_mangle]
pub unsafe extern "system" fn WerRemoveExcludedApplication(exe_name: *const u16, all_users: i32) -> HRESULT {
    let Some(base) = exe_base(exe_name) else { return E_INVALIDARG };
    match plat::exclude_del(all_users != 0, EXCLUDE_KEY.as_ptr(), base) {
        Some(true) => S_OK,
        Some(false) => HR_ENVVAR_NOT_FOUND,
        None => E_ACCESSDENIED,
    }
}

#[no_mangle]
pub unsafe extern "system" fn WerReportAddDump(
    _report: *mut c_void, _process: *mut c_void, _thread: *mut c_void, _dump_type: i32,
    _exception: *mut c_void, _custom: *mut c_void, _flags: u32,
) -> HRESULT {
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn WerReportAddFile(_report: *mut c_void, _path: *const u16, _file_type: i32, _flags: u32) -> HRESULT {
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn WerReportCloseHandle(report: *mut c_void) -> HRESULT {
    let target = report as *mut Report;
    let mut found = false;
    let mut size = 0usize;
    {
        let _g = TABLE_LOCK.lock();
        if !target.is_null() {
            let mut link: *mut *mut Report = &raw mut TABLE;
            while !(*link).is_null() {
                if *link == target {
                    *link = (*target).next;
                    size = report_size(wcslen((*target).event_type.as_ptr()));
                    found = true;
                    break;
                }
                link = &raw mut (**link).next;
            }
        }
    }
    if !found {
        return E_INVALIDARG;
    }
    plat::free(target as *mut u8, size);
    S_OK
}

fn report_size(event_len: usize) -> usize {
    offset_of!(Report, event_type) + (event_len + 1) * size_of::<u16>()
}

#[no_mangle]
pub unsafe extern "system" fn WerReportCreate(
    event_type: *const u16, report_type: i32, info: *const WerReportInformation, handle: *mut *mut c_void,
) -> HRESULT {
    if !handle.is_null() {
        *handle = null_mut();
    }
    if event_type.is_null() || *event_type == 0 || handle.is_null() || report_type as u32 >= WER_REPORT_INVALID as u32 {
        return E_INVALIDARG;
    }
    let len = wcslen(event_type);
    let size = report_size(len);
    let r = plat::alloc_zeroed(size) as *mut Report;
    if r.is_null() {
        return HR_OUTOFMEMORY;
    }
    let dst = (r as *mut u8).add(offset_of!(Report, event_type)) as *mut u16;
    copy_nonoverlapping(event_type, dst, len); // terminating NUL is already there (zeroed block)
    (*r).report_type = report_type;
    if !info.is_null() {
        copy_nonoverlapping(info, &raw mut (*r).info, 1);
    }
    {
        let _g = TABLE_LOCK.lock();
        (*r).next = TABLE;
        TABLE = r;
    }
    *handle = r as *mut c_void;
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn WerReportSetParameter(_report: *mut c_void, _id: u32, _name: *const u16, _value: *const u16) -> HRESULT {
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn WerReportSetUIOption(_report: *mut c_void, _ui_type: i32, _value: *const u16) -> HRESULT {
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn WerReportSubmit(_report: *mut c_void, _consent: i32, _flags: u32, result: *mut i32) -> HRESULT {
    if result.is_null() {
        return E_INVALIDARG;
    }
    *result = WER_DISABLED;
    S_OK
}
