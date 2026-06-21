//! windows/caret.c → Rust. user32 caret client (forwards to win32k).

use crate::ntwrapper::*;
use win32k_sys::win32k::*;

#[no_mangle]
pub unsafe extern "system" fn DestroyCaret() -> BOOL {
    NtUserxDestroyCaret()
}

#[no_mangle]
pub unsafe extern "system" fn SetCaretBlinkTime(uMSeconds: UINT) -> BOOL {
    NtUserxSetCaretBlinkTime(uMSeconds)
}

#[no_mangle]
pub unsafe extern "system" fn SetCaretPos(X: INT, Y: INT) -> BOOL {
    NtUserxSetCaretPos(X, Y)
}
