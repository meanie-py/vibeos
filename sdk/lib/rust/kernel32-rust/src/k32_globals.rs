//! k32_globals.rs — the file-scope globals defined in proc.c.
//!
//! Globals cross-referenced by OTHER kernel32 .c objects must keep their C
//! symbol (`#[no_mangle]`) so the still-C files link against the Rust copy:
//!   BasePathVariableName (path.c), BasepExeLdrEntry (path/loader/utils.c),
//!   BasepAppCertDllsList + gcsAppCert (dllmain.c), UserWaitForInputIdleRoutine
//!   (loader.c).
//! The rest are file-local to proc.c → crate-internal `pub static mut` (shared
//! across the translated body modules via the substrate re-export).
//!
//! NOTE: `BasepAppCertTable` (RTL_QUERY_REGISTRY_TABLE[2]) is defined alongside
//! the AppCert code in the helpers module — it embeds a pointer to
//! BasepConfigureAppCertDlls and to BasepAppCertDllsList.

use crate::k32_types::*;
use ntdll_sys::nt::*;

/// Helper: a const all-zero value of any plain C struct (pointers→null, ints→0).
/// Valid for the repr(C) NT structs here (no references/NonNull fields).
const fn zeroed<T>() -> T {
    unsafe { core::mem::MaybeUninit::<T>::zeroed().assume_init() }
}

/* ---- exported (cross-file referenced) -------------------------------- */

#[no_mangle]
pub static mut UserWaitForInputIdleRoutine: WaitForInputIdleType = None;

#[no_mangle]
pub static mut BasepExeLdrEntry: PLDR_DATA_TABLE_ENTRY = core::ptr::null_mut();

#[no_mangle]
pub static mut BasepAppCertDllsList: LIST_ENTRY = zeroed();

#[no_mangle]
pub static mut gcsAppCert: RTL_CRITICAL_SECTION = zeroed();

// BasePathVariableName = RTL_CONSTANT_STRING(L"PATH")
static PATH_BUF: [u16; 5] = [0x0050, 0x0041, 0x0054, 0x0048, 0x0000]; // "PATH\0"
#[no_mangle]
pub static mut BasePathVariableName: UNICODE_STRING = UNICODE_STRING {
    Length: 8,
    MaximumLength: 10,
    Buffer: PATH_BUF.as_ptr() as *mut u16,
};

/* ---- file-local (crate-internal) ------------------------------------ */

pub static mut BaseUnicodeCommandLine: UNICODE_STRING = zeroed();
pub static mut BaseAnsiCommandLine: ANSI_STRING = zeroed();
pub static mut BaseAnsiStartupInfo: LPSTARTUPINFOA = core::ptr::null_mut();
pub static mut g_AppCertInitialized: BOOLEAN = 0; // FALSE
pub static mut g_HaveAppCerts: BOOLEAN = 0;
pub static mut fEmbeddedCertFunc: PBASEP_APPCERT_EMBEDDED_FUNC = None;
pub static mut g_AppCertStatus: NTSTATUS = 0;
pub static mut g_SaferReplaceProcessThreadTokens: PSAFER_REPLACE_PROCESS_THREAD_TOKENS = None;
pub static mut gSaferHandle: HMODULE = -1isize as HMODULE;
