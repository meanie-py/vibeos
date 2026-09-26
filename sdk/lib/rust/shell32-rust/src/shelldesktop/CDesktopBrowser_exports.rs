//! `CDesktopBrowser.cpp`'s two EXPORTED desktop entry points, hand-written.
//!
//! A separate file from `CDesktopBrowser_free.rs` on purpose: that one is rustconv output
//! and is rewritten in full on every reassembly, so hand edits there vanish silently — the
//! same trap that once carried CDefView's `m_atl` inside a generated file.
// ---------------------------------------------------------------------------------------
// HAND-WRITTEN. Both functions below were refused by the deterministic audit, correctly:
// each drives a `CComPtr` LOCAL through `Attach`/`Detach`, and rustconv states outright that
// it will not rewrite those — "the guard owns the release, and a `Detach` needs a temporary
// that this cannot express". The audit wants exactly one null-pointer declaration per CComPtr
// local so it can insert a release guard; `Detach` transfers ownership OUT, so inserting one
// would double-release. Guessing here is the difference between a working desktop and a
// use-after-free, so the refusal is the tool working, not a gap in it.
//
// Written from the C++ (CDesktopBrowser.cpp), not from the rejected bodies.
// ---------------------------------------------------------------------------------------

use crate::prelude::*;
use crate::user_notification::MSG;
// A #[macro_export] macro lives at the CRATE ROOT whatever module defines it, so this is
// `crate::FAILED_UNEXPECTEDLY` and not a prelude path.
use crate::FAILED_UNEXPECTEDLY;
use crate::shelldesktop::CDesktopBrowser::{IShellBrowser, IShellView};

extern "system" {
    fn SetLastError(code: DWORD);
    fn GetMessageW(message: *mut MSG, window: HWND, first: UINT, last: UINT) -> BOOL;
    fn TranslateMessage(message: *const MSG) -> BOOL;
    fn DispatchMessageW(message: *const MSG) -> LRESULT;
}

/// `SHCreateDesktop` [SHELL32.200] — create the desktop browser and hand its window back.
///
/// C++: `CComPtr<IShellBrowser> Browser; ... return Browser.Detach();`
/// `Detach` returns the raw pointer and clears the smart pointer WITHOUT releasing, so the
/// reference `CDesktopBrowser_CreateInstance` produced is transferred to the caller. That
/// caller is `SHDesktopMessageLoop` below, which `Attach`es it back and releases it at the end
/// — the two functions are one ownership handoff and must be read together.
#[no_mangle]
pub unsafe extern "system" fn SHCreateDesktop(Tray: *mut IShellDesktopTray) -> HANDLE {
    if Tray.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return core::ptr::null_mut();
    }

    let mut browser: *mut IShellBrowser = core::ptr::null_mut();
    let hr = crate::shelldesktop::CDesktopBrowser::CDesktopBrowser_CreateInstance(
        // One interface, two nominal spellings: the export's signature names the
        // prelude's `IShellDesktopTray`, the generated factory the class module's.
        Tray.cast(),
        // The prelude's `IID_*` come from ntdll_sys; the generated factory's `riid`
        // names com_sys's `_GUID`. One record, two islands.
        core::ptr::addr_of!(IID_IShellBrowser).cast(),
        core::ptr::addr_of_mut!(browser).cast::<*mut core::ffi::c_void>(),
    );
    if FAILED_UNEXPECTEDLY!(hr) {
        return core::ptr::null_mut();
    }

    // `Detach()`: hand the reference over without releasing it.
    browser as HANDLE
}

/// `SHDesktopMessageLoop` [SHELL32.201] — run the desktop's message pump until WM_QUIT.
#[no_mangle]
pub unsafe extern "system" fn SHDesktopMessageLoop(hDesktop: HANDLE) -> BOOL {
    if hDesktop.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }

    // `browser.Attach(static_cast<IShellBrowser*>(hDesktop))` — adopt WITHOUT AddRef. This
    // consumes the reference SHCreateDesktop detached to its caller.
    let browser = hDesktop as *mut IShellBrowser;
    let mut shellView: *mut IShellView = core::ptr::null_mut();

    let hr = ((*(*browser).lpVtbl).QueryActiveShellView)(
        browser,
        core::ptr::addr_of_mut!(shellView),
    );
    if FAILED_UNEXPECTEDLY!(hr) {
        // ~CComPtr runs on EVERY exit path, including this early one. Only `browser` is
        // live here — `shellView` is still null, since QueryActiveShellView failed.
        ((*(*browser).lpVtbl).Release)(browser);
        return FALSE;
    }

    let mut Msg: MSG = core::mem::zeroed();
    loop {
        let bRet = GetMessageW(core::ptr::addr_of_mut!(Msg), core::ptr::null_mut(), 0, 0);
        if bRet == 0 {
            break;
        }
        // GetMessageW returns -1 on error; the C++ skips dispatch but keeps looping.
        if bRet != -1 && ((*(*shellView).lpVtbl).TranslateAccelerator)(
            shellView,
            core::ptr::addr_of_mut!(Msg),
        ) != S_OK
        {
            TranslateMessage(core::ptr::addr_of!(Msg));
            DispatchMessageW(core::ptr::addr_of!(Msg));
        }
    }

    // Both `~CComPtr`s, in reverse declaration order as C++ destroys them.
    if !shellView.is_null() {
        ((*(*shellView).lpVtbl).Release)(shellView);
    }
    ((*(*browser).lpVtbl).Release)(browser);
    TRUE
}
