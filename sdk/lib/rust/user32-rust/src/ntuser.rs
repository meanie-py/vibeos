//! NtUser* syscall FFI surface — the user32→win32k boundary. These resolve at link via
//! the `win32ksys` importlib (win32k.sys exports) and at runtime to win32k's syscall
//! dispatch. NTAPI ABI: i386 = stdcall (`_Name@N`), amd64/arm64 = cdecl — `extern "system"`
//! selects the right one per arch.
//!
//! Routine selector constants for the NtUserCall* multiplexers come from the
//! `SimpleCallRoutines` C enum (win32ss/include/ntuser.h, gated by WIN32K_VERSION =
//! NTDDI_WINXPSP2). Values are the bindgen-authoritative ordinals from
//! sdk/lib/rust/win32k-sys (identical across all 3 arches) — the callee (win32k-rust
//! simplecall.rs) decodes the same integer, so caller/callee agree by construction.
//! If you add a routine, copy its value from win32k-sys/src/bindgen_<arch>.rs (verify
//! the 3 arches match) rather than counting enum positions by hand.

use crate::types::*;

/// SimpleCallRoutines::HWNDPARAM_ROUTINE_KILLSYSTEMTIMER (= 78 on i386/amd64/arm64).
pub const HWNDPARAM_ROUTINE_KILLSYSTEMTIMER: DWORD = 78;

extern "system" {
    /// win32k: dispatch a (HWND, param) call by `routine` selector.
    pub fn NtUserCallHwndParam(hWnd: HWND, Param: DWORD_PTR, Routine: DWORD) -> DWORD_PTR;
}
