//! `misc/crtmbox.cpp`: `__acrt_show_narrow_message_box` /
//! `__acrt_show_wide_message_box`, the CRT's MessageBox wrapper.
//!
//! With a debugger attached the text goes to `OutputDebugString` and, unless
//! this is a desktop (HWND) app, the answer is `IDRETRY` ("break into the
//! debugger") with no box at all. With developer UI disallowed, or no user32
//! (`__acrt_can_show_message_box`), it is `IDRETRY` under a debugger and
//! `IDABORT` otherwise. A non-interactive window station adds
//! `MB_SERVICE_NOTIFICATION` and passes no owner window. user32 is reached only
//! through the `__acrt_MessageBox*` thunks, which load it on demand.

use super::win_policies::{
    __acrt_get_developer_information_policy, __acrt_get_windowing_model_policy, DEVELOPER_INFORMATION_POLICY_UI,
    WINDOWING_MODEL_POLICY_HWND,
};
use core::ffi::{c_char, c_int, c_void};
use core::ptr::null_mut;

const IDABORT: c_int = 3;
const IDRETRY: c_int = 4;
const MB_SERVICE_NOTIFICATION: u32 = 0x0020_0000;

extern "system" {
    fn IsDebuggerPresent() -> i32;
    fn __acrt_OutputDebugStringA(text: *const c_char);
    fn OutputDebugStringW(text: *const u16);
    fn __acrt_MessageBoxA(hwnd: *mut c_void, text: *const c_char, caption: *const c_char, kind: u32) -> c_int;
    fn __acrt_MessageBoxW(hwnd: *mut c_void, text: *const u16, caption: *const u16, kind: u32) -> c_int;
}

extern "C" {
    fn __acrt_can_show_message_box() -> bool;
    fn __acrt_get_parent_window() -> *mut c_void;
    fn __acrt_is_interactive() -> bool;
}

trait MessageBoxChar {
    /// `traits::output_debug_string`
    unsafe fn output_debug_string(text: *const Self);
    /// `traits::message_box`
    unsafe fn message_box(hwnd: *mut c_void, text: *const Self, caption: *const Self, kind: u32) -> c_int;
}

impl MessageBoxChar for c_char {
    unsafe fn output_debug_string(text: *const c_char) {
        __acrt_OutputDebugStringA(text)
    }
    unsafe fn message_box(hwnd: *mut c_void, text: *const c_char, caption: *const c_char, kind: u32) -> c_int {
        __acrt_MessageBoxA(hwnd, text, caption, kind)
    }
}

impl MessageBoxChar for u16 {
    unsafe fn output_debug_string(text: *const u16) {
        OutputDebugStringW(text)
    }
    unsafe fn message_box(hwnd: *mut c_void, text: *const u16, caption: *const u16, kind: u32) -> c_int {
        __acrt_MessageBoxW(hwnd, text, caption, kind)
    }
}

/// `common_show_message_box<Character>`
unsafe fn common_show_message_box<C: MessageBoxChar>(text: *const C, caption: *const C, kind: u32) -> c_int {
    let debugger_attached = IsDebuggerPresent() != 0;
    if debugger_attached {
        // Output the message to the attached debugger (useful when remotely
        // debugging).
        if !text.is_null() {
            C::output_debug_string(text);
        }

        // No message box under a debugger unless it's a desktop app; let the
        // caller break into the debugger instead.
        let is_desktop_app = __acrt_get_windowing_model_policy() == WINDOWING_MODEL_POLICY_HWND;
        if !is_desktop_app {
            return IDRETRY; // Retry = Break into the debugger
        }
    }

    let show_ui = __acrt_get_developer_information_policy() == DEVELOPER_INFORMATION_POLICY_UI;
    if !show_ui || !__acrt_can_show_message_box() {
        // No message box pointers (e.g. CoreSys): abort, unless a debugger is
        // attached -- then break into it. The text already went to it above.
        return if debugger_attached { IDRETRY } else { IDABORT };
    }

    // Not attached to a visible window station (e.g. a non-interactive
    // service): the box would be invisible and stall the program.
    if !__acrt_is_interactive() {
        return C::message_box(null_mut(), text, caption, kind | MB_SERVICE_NOTIFICATION);
    }

    C::message_box(__acrt_get_parent_window(), text, caption, kind)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn __acrt_show_narrow_message_box(
    text: *const c_char,
    caption: *const c_char,
    kind: u32,
) -> c_int {
    common_show_message_box(text, caption, kind)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn __acrt_show_wide_message_box(text: *const u16, caption: *const u16, kind: u32) -> c_int {
    common_show_message_box(text, caption, kind)
}
