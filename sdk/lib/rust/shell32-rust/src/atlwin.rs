//! atlwin — the ATL `CWindowImpl` substrate, without the machine-code thunk.
//!
//! WHAT ATL DOES. `CWindowImplBaseT` builds a per-instance THUNK: a few bytes of generated
//! machine code that rewrite the first argument of the window procedure from the `HWND` to
//! `this`, then jump to a static `WindowProc`. That is why ATL's `WindowProc` opens with
//! `pThis = reinterpret_cast<...>(hWnd)` — reading the first parameter as the object pointer
//! is correct there, not a bug, and a faithful translation of the body says the same thing.
//!
//! WHAT WE DO INSTEAD, and why. Reproducing the thunk means emitting instruction bytes into
//! executable memory for i386, amd64 AND arm64 — three hand-written code generators, on a
//! tree whose whole point is that arm64 is a first-class target, and arm64 is precisely where
//! we can least afford another per-arch path. `GWLP_USERDATA` gets `this` back from the
//! `HWND` with no codegen at all and is the same one indirection ATL's thunk saves.
//!
//! The recorded constraint this has to respect is NOT the mechanism. utils.cpp, beside
//! `CStubWindow32_CreateStub_RustBridge`, records it: an ATL `CWindowImpl`'s per-instance
//! thunk and its REGISTERED WINDOW CLASS must have exactly ONE owner, which is why
//! `CStubWindow32` was bridged to C++ rather than reimplemented while two C++ TUs still
//! constructed it: a second implementation would have registered the same class name and
//! split `FindStubWindow`. Once both were Rust, it moved onto this substrate as the ONE owner
//! (crate::atl_stub_windows, 2026-09-24, with CMSGlobalFolderOptionsStub and CChangeNotifyBroker). CDefView is not in that position: `SHELLDLL_DefView` is registered in
//! exactly one place (CDefView.cpp:528) and no other TU constructs the class, so a Rust
//! implementation splits nothing. `CMenuFocusManager`, `CMenuToolbars`, `CChangeNotifyServer`
//! and `CDirectoryWatcher` inherit this MECHANISM when they are ported, but each still needs
//! its OWN ownership check — the mechanism is shared, the ownership question is per-class.
//!
//! THE ONE BEHAVIOURAL DIFFERENCE, stated because it is not free. ATL parks `this` in a
//! per-thread list inside `_AtlWinModule` and picks it up on the FIRST message the window
//! receives, whatever that is. We pass it as `CreateWindowExW`'s `lpCreateParam` and pick it
//! up from `CREATESTRUCTW::lpCreateParams` at `WM_NCCREATE`, which needs no global state and
//! is reentrant and thread-safe by construction. The cost: a message delivered BEFORE
//! `WM_NCCREATE` (`WM_GETMINMAXINFO`, which an OVERLAPPED window gets and a `WS_CHILD` window
//! does not) has no `this` yet and goes to `DefWindowProcW`. CDefView is `WS_CHILD`
//! (CDefView.cpp:3912), so it never sees one. A future overlapped ATL window that needs a
//! handler for such a message must not use this path silently — check it when you port one.
//! The second cost is that `lpCreateParam` is now spoken for; ATL let a caller pass its own,
//! and no ATL window in this tree does.
//!
//! ONE CAST HERE IS NOT COSMETIC, so it is named rather than hidden. The island renders
//! `ATL_CWndClassInfo` (= `_ATL_WNDCLASSINFOW`) with ANSI-typed members, because `ntdll-sys`
//! is generated without `-DUNICODE` and the struct's `WNDCLASSEX`/`LPCTSTR`/`TCHAR` resolve
//! through that glob — the field-manual entry "A UNICODE module inherits the ANSI TCHAR family
//! from the `-sys` glob". shell32 is a Unicode build, so `AtlRegisterWndClass` casts to the W
//! forms at the two API boundaries; every field it touches is a pointer, so the cast is a
//! spelling fix and not a width change. The one member that IS the wrong width is
//! `m_szAutoName: [TCHAR; 21]`, half the `[WCHAR; 21]` ATL declares. That is harmless only
//! because every ATL window in this tree supplies its own `lpszClassName`, so ATL's
//! auto-generated "ATL:%p" name is never written. A class that relies on the auto name must
//! not use this struct as the island renders it.

use crate::prelude::*;
use crate::win32::{GetWindowLongPtr as W32GetWindowLongPtr, SetWindowLongPtr as W32SetWindowLongPtr};

/// `winuser.h` `#define CW_USEDEFAULT ((int)0x80000000)` — a CAST macro, so no island carries it.
pub const CW_USEDEFAULT: INT = 0x8000_0000u32 as INT;

// `crate::win32`'s accessors are declared against the hand-written `HWND` (a bare `HANDLE`),
// while this module is island-first and `HWND` is `*mut HWND__`. Same pointer, different
// nominal type; wrap once here rather than casting at eight call sites.
#[inline]
unsafe fn GetWindowLongPtr(hWnd: HWND, index: INT) -> LONG_PTR {
    W32GetWindowLongPtr(hWnd.cast(), index) as LONG_PTR
}
#[inline]
unsafe fn SetWindowLongPtr(hWnd: HWND, index: INT, value: LONG_PTR) -> LONG_PTR {
    // Both are the machine's pointer width; the split is nominal (island `LONG_PTR` vs
    // `types::LONG_PTR`), which is the trap prelude.rs's header describes.
    W32SetWindowLongPtr(hWnd.cast(), index, value as _) as LONG_PTR
}

/// `CWindowImplRoot::WINSTATE_DESTROYED` (atlwin.h).
pub const WINSTATE_DESTROYED: DWORD = 0x0000_0001;

/// The base-class state ATL keeps in `CWindowImplRoot` / `CWindowImplBaseT`. A translated
/// class embeds this and hands it out through [`AtlWindow::atl_state`].
///
/// `m_pCurrentMsg` is deliberately absent: ATL keeps it only to assert that a handler did not
/// re-enter the map, and to decide at the end of the OUTERMOST dispatch whether the window
/// has been destroyed. A `None`/`Some` depth counter expresses the second use exactly, and
/// the assertions compile out in a release build anyway.
#[repr(C)]
pub struct AtlWindowState {
    pub m_dwState: DWORD,
    /// `::DefWindowProcW` unless the class SUBCLASSED an existing window.
    pub m_pfnSuperWindowProc: WNDPROC,
    /// Dispatch depth. ATL reads `m_pCurrentMsg == NULL` for "this is the outermost message".
    pub m_cDispatchDepth: UINT,
}

impl AtlWindowState {
    #[inline]
    pub const fn new() -> Self {
        Self { m_dwState: 0, m_pfnSuperWindowProc: None, m_cDispatchDepth: 0 }
    }
}

impl Default for AtlWindowState {
    fn default() -> Self {
        Self::new()
    }
}

/// What a translated ATL window class has to provide. Every method takes a raw `*mut Self`
/// rather than `&mut self`: the window procedure is reentrant (a handler can dispatch another
/// message to the same window), so a `&mut` would be two live mutable borrows of one object.
///
/// # Safety
/// `atl_hwnd` and `atl_state` must return pointers to fields of the SAME object they are given,
/// stable for the window's lifetime.
pub unsafe trait AtlWindow: Sized {
    unsafe fn atl_hwnd(this: *mut Self) -> *mut HWND;
    unsafe fn atl_state(this: *mut Self) -> *mut AtlWindowState;

    /// The class's `BEGIN_MSG_MAP` table. Returns `TRUE` when a handler claimed the message.
    unsafe fn atl_process_window_message(
        this: *mut Self,
        hWnd: HWND,
        uMsg: UINT,
        wParam: WPARAM,
        lParam: LPARAM,
        lResult: *mut LRESULT,
        dwMsgMapID: DWORD,
    ) -> BOOL;

    /// `CWindowImplBaseT::OnFinalMessage` — runs after the last message, with `m_hWnd` already
    /// cleared. ATL's default is empty, but this has NO default on purpose: an empty default
    /// let CDefView's port omit its override (`this->Release()`, balancing the AddRef in
    /// `OnNCCreate`), so every folder view window leaked its view and browseui warned
    /// `Unexpected RefCount > 0 (1)` on each close. Every implementor states what it does,
    /// even when the answer is nothing.
    unsafe fn atl_on_final_message(this: *mut Self, hWnd: HWND);

    /// `CWindowImplBaseT::GetWindowProc` — the procedure [`AtlStartWindowProc`] installs on the
    /// window once `this` is known. **ATL calls the VIRTUAL**, so a derived class that overrides
    /// `GetWindowProc` gets ITS procedure installed and the base one is never reached directly.
    ///
    /// This hook exists because hardcoding `AtlWindowProc::<T>` here drops that override
    /// silently, and the override is not decoration: `CDefView::GetWindowProc` returns a
    /// procedure whose only job is to `AddRef` around the dispatch, because a handler can
    /// release the view's last reference and ATL's base would then keep touching freed memory.
    /// Skipping it shipped a use-after-free — explorer's desktop died with
    /// STATUS_ILLEGAL_INSTRUCTION at a stable heap address (the freed block's free-list link,
    /// reached through a dangling vtable), taking the kmtest sweep from 91 launched to 0. It
    /// reproduced byte-identically across boots and VANISHED under any added tracing, which is
    /// what a use-after-free looks like from the outside.
    ///
    /// The default is the base procedure, which is right for a class that does not override.
    unsafe fn atl_window_proc() -> WNDPROC {
        Some(AtlWindowProc::<Self>)
    }
}

/// `CWindowImplBaseT::DefWindowProc` — forward to the superclass procedure.
#[inline]
pub unsafe fn AtlDefWindowProc<T: AtlWindow>(
    this: *mut T,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let st = T::atl_state(this);
    let hWnd = *T::atl_hwnd(this);
    match (*st).m_pfnSuperWindowProc {
        Some(p) => CallWindowProcW(Some(p), hWnd, uMsg, wParam, lParam),
        None => DefWindowProcW(hWnd, uMsg, wParam, lParam),
    }
}

/// `CWindowImplBaseT::StartWindowProc` — the procedure the WNDCLASS is registered with. It
/// runs exactly once per window: it installs `this`, swaps itself out for [`AtlWindowProc`],
/// and forwards the message that brought it here.
pub unsafe extern "system" fn AtlStartWindowProc<T: AtlWindow>(
    hWnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    if uMsg != WM_NCCREATE {
        // See the module header: no `this` exists yet. ATL cannot reach this case because it
        // parks `this` before CreateWindowEx; we can, for an overlapped window only.
        return DefWindowProcW(hWnd, uMsg, wParam, lParam);
    }

    let cs = lParam as *mut CREATESTRUCTW;
    if cs.is_null() {
        return DefWindowProcW(hWnd, uMsg, wParam, lParam);
    }
    let this = (*cs).lpCreateParams as *mut T;
    if this.is_null() {
        return DefWindowProcW(hWnd, uMsg, wParam, lParam);
    }

    SetWindowLongPtr(hWnd, GWLP_USERDATA as INT, this as LONG_PTR);
    // ATL installs `pThis->GetWindowProc()` (the virtual), not its own base procedure, and
    // then FORWARDS this message through the same pointer it installed. Both halves matter:
    // installing the base proc skips a derived override for every later message, and calling
    // the base proc here skips it for WM_NCCREATE specifically.
    let wndproc = T::atl_window_proc().unwrap_or(AtlWindowProc::<T>);
    SetWindowLongPtr(hWnd, GWLP_WNDPROC as INT, wndproc as LONG_PTR);
    *T::atl_hwnd(this) = hWnd;

    wndproc(hWnd, uMsg, wParam, lParam)
}

/// `CWindowImplBaseT::WindowProc`, line for line, with `this` read from `GWLP_USERDATA`
/// instead of from the first parameter.
pub unsafe extern "system" fn AtlWindowProc<T: AtlWindow>(
    hWnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let this = GetWindowLongPtr(hWnd, GWLP_USERDATA as INT) as *mut T;
    if this.is_null() {
        return DefWindowProcW(hWnd, uMsg, wParam, lParam);
    }

    let st = T::atl_state(this);
    // ATL asserts both of these and then returns 0 in a release build.
    if ((*st).m_dwState & WINSTATE_DESTROYED) != 0 || (*T::atl_hwnd(this)).is_null() {
        return 0;
    }

    // ATL re-reads hWnd from the object; the thunk's argument and m_hWnd can differ while a
    // window is being subclassed.
    let hWnd = *T::atl_hwnd(this);
    let outermost = (*st).m_cDispatchDepth == 0;
    (*st).m_cDispatchDepth += 1;

    let mut lResult: LRESULT = 0;
    let handled = T::atl_process_window_message(this, hWnd, uMsg, wParam, lParam, &mut lResult, 0);

    if handled == FALSE {
        if uMsg == WM_NCDESTROY {
            // Restore the superclass procedure, but ONLY if nobody re-subclassed us in the
            // meantime -- ATL compares the value it is about to overwrite.
            let saveWindowProc = GetWindowLongPtr(hWnd, GWLP_WNDPROC as INT);
            lResult = AtlDefWindowProc(this, uMsg, wParam, lParam);
            if let Some(sup) = (*st).m_pfnSuperWindowProc {
                if saveWindowProc == GetWindowLongPtr(hWnd, GWLP_WNDPROC as INT) {
                    SetWindowLongPtr(hWnd, GWLP_WNDPROC as INT, sup as LONG_PTR);
                }
            }
            (*st).m_dwState |= WINSTATE_DESTROYED;
        } else {
            lResult = AtlDefWindowProc(this, uMsg, wParam, lParam);
        }
    }

    (*st).m_cDispatchDepth -= 1;
    if outermost && ((*st).m_dwState & WINSTATE_DESTROYED) != 0 {
        (*st).m_dwState &= !WINSTATE_DESTROYED;
        *T::atl_hwnd(this) = core::ptr::null_mut();
        // The window is gone, so USERDATA must not outlive it either: the HWND can be
        // recycled, and a stale `this` behind a recycled handle is a use-after-free that
        // only shows up under window-handle pressure.
        SetWindowLongPtr(hWnd, GWLP_USERDATA as INT, 0);
        T::atl_on_final_message(this, hWnd);
    }
    lResult
}

/// `CWindowImpl::Create` -> `CWindowImplBaseT::Create`, with `this` delivered through
/// `lpCreateParam` (see the module header). `atom` is the value `CWndClassInfo::Register`
/// returned; the caller owns registration, exactly as in C++.
#[allow(clippy::too_many_arguments)]
pub unsafe fn AtlCreateWindow<T: AtlWindow>(
    this: *mut T,
    hWndParent: HWND,
    rect: *const RECT,
    szWindowName: LPCWSTR,
    dwStyle: DWORD,
    dwExStyle: DWORD,
    MenuOrID: HMENU,
    atom: ATOM,
    hInstance: HINSTANCE,
) -> HWND {
    if atom == 0 {
        return core::ptr::null_mut();
    }
    // ATL: a child window with no menu gets `this` as its control id.
    let mut MenuOrID = MenuOrID;
    if MenuOrID.is_null() && (dwStyle & WS_CHILD) != 0 {
        MenuOrID = this as UINT_PTR as HMENU;
    }
    let (x, y, cx, cy) = if rect.is_null() {
        (CW_USEDEFAULT, CW_USEDEFAULT, CW_USEDEFAULT, CW_USEDEFAULT)
    } else {
        (
            (*rect).left,
            (*rect).top,
            (*rect).right - (*rect).left,
            (*rect).bottom - (*rect).top,
        )
    };
    CreateWindowExW(
        dwExStyle,
        atom as usize as LPCWSTR, // MAKEINTATOM
        szWindowName,
        dwStyle,
        x,
        y,
        cx,
        cy,
        hWndParent,
        MenuOrID,
        hInstance,
        this as *mut core::ffi::c_void,
    )
}

/// `_ATL_WNDCLASSINFOW::Register` (atlwin.h:2080). ReactOS's implementation takes a
/// `WNDPROC*` and never writes it — the superclass procedure is only ever set by
/// `SubclassWindow` — so this takes no out-parameter rather than carry a lie in the signature.
pub unsafe fn AtlRegisterWndClass(wc: &mut ATL_CWndClassInfo, hInstance: HINSTANCE) -> ATOM {
    if wc.m_wc.hInstance.is_null() {
        wc.m_wc.hInstance = hInstance;
    }
    if wc.m_atom == 0 {
        // `m_bSystemCursor` picks which module the cursor id resolves against; shell32's ATL
        // windows all use a system cursor, i.e. NULL.
        wc.m_wc.hCursor = LoadCursorW(
            if wc.m_bSystemCursor != FALSE { core::ptr::null_mut() } else { hInstance },
            wc.m_lpszCursorID.cast(),
        );
        wc.m_atom = RegisterClassExW(core::ptr::addr_of!(wc.m_wc).cast());
    }
    wc.m_atom
}

/// `CWindowImplBaseT::SubclassWindow` (atlwin.h:1585) — adopt an EXISTING window.
///
/// The sibling of [`AtlStartWindowProc`]: that one takes `this` out of the `CREATESTRUCT` of a
/// window we are creating, this one attaches to a window somebody else already made. Both end
/// in the same three writes — park `this` on the window, install the procedure, record the
/// `HWND` — and the only addition here is saving the procedure being displaced, because a
/// subclassed window's `DefWindowProc` must chain to it rather than to `DefWindowProcW`.
///
/// ATL does this through `m_thunk.Init(GetWindowProc(), this)`; we use `GWLP_USERDATA`, for the
/// reason in the module header (no machine-code emission, one indirection either way).
///
/// `CChangeNotifyServer::Initialize` is the first caller: it creates a worker window with
/// `SHCreateDefaultWorkerWindow()` and then subclasses it.
pub unsafe fn AtlSubclassWindow<T: AtlWindow>(this: *mut T, hWnd: HWND) -> BOOL {
    // ATL asserts both of these. An assert is a no-op in a shipping build, so check: a failed
    // subclass has to be reported, not walked past — the caller's next act is to use m_hWnd.
    if this.is_null() || hWnd.is_null() || IsWindow(hWnd) == 0 {
        return FALSE;
    }
    if !(*T::atl_hwnd(this)).is_null() {
        return FALSE; // ATLASSERT(m_hWnd == NULL) — already attached to a window.
    }

    SetWindowLongPtr(hWnd, GWLP_USERDATA as INT, this as LONG_PTR);
    // The VIRTUAL `GetWindowProc`, exactly as in the create path — see `atl_window_proc`.
    let wndproc = T::atl_window_proc().unwrap_or(AtlWindowProc::<T>);
    let previous = SetWindowLongPtr(hWnd, GWLP_WNDPROC as INT, wndproc as LONG_PTR);
    if previous == 0 {
        // ATL returns FALSE here. Undo the USERDATA write so a failed attach leaves no
        // half-attached window pointing at an object that does not own it.
        SetWindowLongPtr(hWnd, GWLP_USERDATA as INT, 0);
        return FALSE;
    }

    (*T::atl_state(this)).m_pfnSuperWindowProc =
        core::mem::transmute::<LONG_PTR, WNDPROC>(previous);
    *T::atl_hwnd(this) = hWnd;
    TRUE
}
