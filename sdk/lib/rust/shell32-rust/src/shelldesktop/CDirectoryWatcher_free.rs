//! `CDirectoryWatcher.cpp`'s file-scope half: the APC thread, its four callbacks, and the
//! `static` state all of them share with the class half.
//!
//! HAND-INTEGRATED from the paid bodies in
//! `.responses/shelldesktop__CDirectoryWatcher__factseal_v1__callseal_v1/`, rather than left as
//! assembler output, for one structural reason: **this TU's file scope spans two Rust modules.**
//! One rustconv run owns one output file, so a C++ TU with both a class and file-scope
//! functions is two runs — and C `static` means "scoped to the translation unit", which the
//! assembler renders as module-private because for a single-module TU the two coincide. Here
//! they do not: the class methods next door drive `s_hThreadAPC`, `s_buffer` and
//! `s_fTerminateAllWatchers`, and hand `_NotificationCompletion` and
//! `DirectoryWatcherThreadFuncAPC` to the Win32 calls that invoke them. Emitted private, a TU
//! was invisible to its own other half.
//!
//! `pub` is not a linkage change and does not export anything — none of these carries
//! `#[no_mangle]`, so the C's internal linkage is preserved exactly. It is module visibility,
//! widened to match the file scope the C actually has.
//!
//! This is the same shape as `menuband-rust`'s `menu_focus_manager_support.rs`, which exists
//! because a class-static TLS slot shared by three functions cannot be owned by any one worker
//! body. Fixing it in the assembler instead would mean untying `pub` from `#[no_mangle]` in
//! `abi_enforce`, which five tests pin deliberately; that is a shared-tooling change worth
//! making on its own, not as a side effect of one TU's integration.

use crate::prelude::*;
use crate::shelldesktop::CDirectoryWatcher::{
    CDirectoryWatcher, CDirectoryWatcher_QuitWatching_body, CDirectoryWatcher_ReadCompletion_body,
    CDirectoryWatcher_RestartWatching_body,
};

extern "system" {
    fn WaitForSingleObjectEx(handle: HANDLE, milliseconds: DWORD, alertable: BOOL) -> DWORD;
    fn GetCurrentThread() -> HANDLE;
}

// ---- file-scope state (translated preamble) ----
pub static mut s_hThreadAPC: HANDLE = core::ptr::null_mut();
pub static mut s_fTerminateAllWatchers: BOOL = 0;
pub const BUFFER_SIZE: usize = 0x1000;
pub static mut s_buffer: [BYTE; BUFFER_SIZE] = [0; BUFFER_SIZE];

/// The APC thread procedure: sleep alertably until every watcher has been asked to stop.
pub unsafe extern "system" fn DirectoryWatcherThreadFuncAPC(_: *mut core::ffi::c_void) -> u32 {
    while s_fTerminateAllWatchers == 0 {
        WaitForSingleObjectEx(GetCurrentThread(), INFINITE, TRUE);
    }
    0
}

/// `QueueUserAPC` callback for `RequestAddWatcher`.
///
/// The `.Method()` spelling the worker returned is C++; the class is a plain Rust struct with
/// free functions, so each call goes to the translated body explicitly.
pub unsafe extern "system" fn _AddDirectoryProcAPC(Parameter: ULONG_PTR) {
    let pDirectoryWatcher = Parameter as *mut CDirectoryWatcher;
    assert!(!pDirectoryWatcher.is_null());
    CDirectoryWatcher_RestartWatching_body(pDirectoryWatcher);
}

/// `QueueUserAPC` callback for `RequestTermination`.
pub unsafe extern "system" fn _RequestTerminationAPC(Parameter: ULONG_PTR) {
    let pDirectoryWatcher = Parameter as *mut CDirectoryWatcher;
    assert!(!pDirectoryWatcher.is_null());
    CDirectoryWatcher_QuitWatching_body(pDirectoryWatcher);
}

/// `QueueUserAPC` callback for `RequestAllWatchersTermination`.
pub unsafe extern "system" fn _RequestAllTerminationAPC(_Parameter: ULONG_PTR) {
    s_fTerminateAllWatchers = TRUE;
    CloseHandle(s_hThreadAPC);
    s_hThreadAPC = core::ptr::null_mut();
}

/// `ReadDirectoryChangesW`'s completion routine.
///
/// The watcher pointer rides in `OVERLAPPED::hEvent`, which is why `m_overlapped` is a
/// by-value member and not a local.
pub unsafe extern "system" fn _NotificationCompletion(
    dwErrorCode: DWORD,
    dwNumberOfBytesTransfered: DWORD,
    lpOverlapped: *mut OVERLAPPED,
) {
    let pDirectoryWatcher = (*lpOverlapped).hEvent as *mut CDirectoryWatcher;
    assert!(!pDirectoryWatcher.is_null());
    CDirectoryWatcher_ReadCompletion_body(
        pDirectoryWatcher,
        dwErrorCode,
        dwNumberOfBytesTransfered,
    );
}
