//! misc/timer.c → Rust. user32 system-timer client.

use crate::ntuser::*;
use crate::types::*;

/// Kill a system timer. Forwards to win32k via NtUserCallHwndParam
/// (NtUserxKillSystemTimer in the C ntwrapper.h inline). i386 export: `_KillSystemTimer@8`.
///
/// C: `BOOL WINAPI KillSystemTimer(HWND hWnd, UINT_PTR IDEvent)`
#[no_mangle]
pub unsafe extern "system" fn KillSystemTimer(hWnd: HWND, IDEvent: UINT_PTR) -> BOOL {
    NtUserCallHwndParam(hWnd, IDEvent as DWORD_PTR, HWNDPARAM_ROUTINE_KILLSYSTEMTIMER) as BOOL
}
