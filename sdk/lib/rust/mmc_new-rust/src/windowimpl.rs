//! ATL's `CWindowImpl` machinery as CMainWnd / CConsoleWnd use it, without the machine-code
//! thunk (see `atlclasses-rust/src/win.rs` for why: the tree has no per-arch thunk generator).
//!
//! mmc_new cannot use `atl_rust::win::create_window` as the cppapp renderer wired it up, for
//! three reasons the C++ spells out:
//!   * its window classes are hand-built (`CMainWnd::GetWndClassInfo()` with its own WNDCLASSEX,
//!     icon, brush and class name) and `CConsoleWnd` is an MDI CHILD: `CMainWnd::OnFileNew` creates
//!     it with WM_MDICREATE using `CConsoleWnd::GetWndClassName()`, so the class has to be
//!     registered by `CWndClassInfo::Register` with `StartWindowProc` as its procedure and the
//!     object has to be bound by `_AtlWinModule.AddCreateWndData` -- `lpCreateParams` of an MDI
//!     child is an MDICREATESTRUCT, not `this`;
//!   * `CMainWnd` hangs a *frame* superclass procedure on a `CWndProcThunk`
//!     (`m_FrameThunk.Init(XDefFrameProc, this)`), where the thunk calls `XDefFrameProc` with
//!     `this` in the HWND slot;
//!   * `CConsoleWnd`'s superclass procedure is `DefMDIChildProc`.
//!
//! The mechanism, step for step as in `sdk/lib/atl/atlwin.h`:
//!   `AddCreateWndData(&cd, this)` queues `this` for the calling thread; the class's
//!   `StartWindowProc` -- the first procedure any window of the class sees -- pops it, binds
//!   `m_hWnd`, installs the real `WindowProc` with `SetWindowLongPtr(GWLP_WNDPROC)`; `WindowProc`
//!   dispatches `ProcessWindowMessage` and falls back to `DefWindowProc`
//!   (`CallWindowProc(m_pfnSuperWindowProc, ...)`). The ONE substitution is that ATL's thunk
//!   swaps HWND for `this` on every call; here `this` is found from a window property that
//!   `StartWindowProc` sets, and a `CWndProcThunk` is a `{ proc, this }` pair that
//!   `call_super` honours (it passes `this` where the thunk would).
use crate::prelude::*;
use atl_rust::win::MessageMap;
use core::ffi::c_void;

extern "system" {
    pub fn CreateWindowExW(
        dwExStyle: DWORD,
        lpClassName: LPCWSTR,
        lpWindowName: LPCWSTR,
        dwStyle: DWORD,
        X: c_int,
        Y: c_int,
        nWidth: c_int,
        nHeight: c_int,
        hWndParent: HWND,
        hMenu: HMENU,
        hInstance: HINSTANCE,
        lpParam: LPVOID,
    ) -> HWND;
    fn RegisterClassExW(arg1: *const WNDCLASSEXW) -> ATOM;
    fn LoadCursorW(hInstance: HINSTANCE, lpCursorName: LPCWSTR) -> HCURSOR;
    fn CallWindowProcW(lpPrevWndFunc: WNDPROC, hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn DefWindowProcW(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn SetPropW(hWnd: HWND, lpString: LPCWSTR, hData: HANDLE) -> BOOL;
    fn GetPropW(hWnd: HWND, lpString: LPCWSTR) -> HANDLE;
    fn RemovePropW(hWnd: HWND, lpString: LPCWSTR) -> HANDLE;
    fn GetCurrentThreadId() -> DWORD;
}

/// `typedef _ATL_WNDCLASSINFOW CWndClassInfo;`
pub type CWndClassInfo = ATL_CWndClassInfo;

/// `typedef ATL::CAtlStringW CAtlString` -- the ATL string, ported once in atlclasses-rust.
pub type CAtlString = CStringW;

impl CWndClassInfo {
    /// `ATOM Register(WNDPROC *p)` (atlwin.h): the cursor is resolved on first use; ReactOS'
    /// ATL ignores `p` (it only matters for superclassing), so it is ignored here too.
    pub unsafe fn Register(&mut self, _p: *mut WNDPROC) -> ATOM {
        if self.m_wc.hInstance.is_null() {
            self.m_wc.hInstance = _AtlBaseModule.GetModuleInstance();
        }
        if self.m_atom == 0 {
            if self.m_bSystemCursor != 0 {
                self.m_wc.hCursor = LoadCursorW(core::ptr::null_mut(), self.m_lpszCursorID);
            } else {
                self.m_wc.hCursor = LoadCursorW(_AtlBaseModule.GetResourceInstance(), self.m_lpszCursorID);
            }
            self.m_atom = RegisterClassExW(core::ptr::addr_of!(self.m_wc) as *const WNDCLASSEXW);
        }
        self.m_atom
    }
}

/// `CWndProcThunk`: the instruction-bytes thunk, as `{ proc, this }`. `GetWNDPROC()` returns a
/// stable address that only `call_super` interprets.
#[repr(C)]
pub struct CWndProcThunk {
    pub m_proc: WNDPROC,
    pub m_pThis: *mut c_void,
    pub cd: ATL__AtlCreateWndData,
}

/// What a thunk's WNDPROC address is. Called directly (it should never be -- every path that
/// holds it goes through `call_super`), it is the default procedure.
unsafe extern "system" fn thunk_wndproc(hWnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    DefWindowProcW(hWnd, uMsg, wParam, lParam)
}

impl CWndProcThunk {
    /// `BOOL Init(WNDPROC proc, void *pThis)`; the real one fails only when VirtualAlloc does.
    pub unsafe fn Init(&mut self, proc_: WNDPROC, pThis: *mut c_void) -> BOOL {
        self.m_proc = proc_;
        self.m_pThis = pThis;
        1
    }
    pub unsafe fn GetWNDPROC(&mut self) -> WNDPROC {
        Some(thunk_wndproc)
    }
}

/// `CallWindowProc(super, hWnd, ...)`, where `super` may be a thunk's WNDPROC: then the thunk's
/// `{ proc, this }` is called with `this` in the HWND slot, as the machine-code thunk would.
unsafe fn call_super(super_: WNDPROC, thunk: *const CWndProcThunk, hWnd: HWND, msg: UINT, w: WPARAM, l: LPARAM) -> LRESULT {
    let is_thunk = match (super_, thunk.is_null()) {
        (Some(f), false) => f as usize == thunk_wndproc as usize,
        _ => false,
    };
    if is_thunk {
        if let Some(p) = (*thunk).m_proc {
            return p((*thunk).m_pThis as HWND, msg, w, l);
        }
        return DefWindowProcW(hWnd, msg, w, l);
    }
    // m_pfnSuperWindowProc starts as ::DefWindowProc.
    CallWindowProcW(super_.or(Some(DefWindowProcW)), hWnd, msg, w, l)
}

/// Queue of `AddCreateWndData` entries (ATL's `m_pCreateWndList`) and, beside each, the
/// monomorphised binder that knows the object's type.
type Binder = unsafe fn(*mut c_void, HWND, UINT, WPARAM, LPARAM) -> LRESULT;
static mut CREATE_WND_LIST: *mut ATL__AtlCreateWndData = core::ptr::null_mut();
static mut CREATE_WND_BINDERS: [(usize, usize); 16] = [(0, 0); 16];

/// `_AtlWinModule`, which the C++ spells `_AtlWinModule.AddCreateWndData(&m_thunk.cd, this)`.
pub struct CAtlWinModule;
#[allow(non_upper_case_globals)]
pub static _AtlWinModule: CAtlWinModule = CAtlWinModule;

impl CAtlWinModule {
    /// `AtlWinModuleAddCreateWndData`: push `pData` for this thread. (ATL takes a critical
    /// section here; window creation in this app is on the UI thread only.)
    pub unsafe fn AddCreateWndData<T: AtlWindowImpl>(&self, pData: *mut ATL__AtlCreateWndData, pObject: *mut T) {
        (*pData).m_pThis = pObject as *mut c_void;
        (*pData).m_dwThreadID = GetCurrentThreadId();
        (*pData).m_pNext = CREATE_WND_LIST;
        CREATE_WND_LIST = pData;
        let binders = &mut *core::ptr::addr_of_mut!(CREATE_WND_BINDERS);
        let slot = binders.iter().position(|b| b.0 == 0).unwrap_or(0);
        binders[slot] = (pData as usize, bind_and_dispatch::<T> as Binder as usize);
    }

    /// `AtlWinModuleExtractCreateWndData`: pop this thread's most recent entry; the binder goes
    /// with it.
    unsafe fn ExtractCreateWndData(&self) -> Option<(*mut c_void, Binder)> {
        let threadID = GetCurrentThreadId();
        let mut previous: *mut *mut ATL__AtlCreateWndData = core::ptr::addr_of_mut!(CREATE_WND_LIST);
        let mut current = CREATE_WND_LIST;
        while !current.is_null() {
            if (*current).m_dwThreadID == threadID {
                *previous = (*current).m_pNext;
                let binders = &mut *core::ptr::addr_of_mut!(CREATE_WND_BINDERS);
                let binder = binders.iter_mut().find(|b| b.0 == current as usize)?;
                let f = binder.1;
                *binder = (0, 0);
                return Some(((*current).m_pThis, core::mem::transmute::<usize, Binder>(f)));
            }
            previous = core::ptr::addr_of_mut!((*current).m_pNext);
            current = (*current).m_pNext;
        }
        None
    }
}

/// The window property that carries `this` once `StartWindowProc` has bound it (ATL gets it
/// from the thunk).
static PROP_THIS: [u16; 12] = prop_name("AtlRustThis\0");
const fn prop_name<const N: usize>(s: &str) -> [u16; N] {
    let b = s.as_bytes();
    let mut out = [0u16; N];
    let mut i = 0;
    while i < N {
        out[i] = b[i] as u16;
        i += 1;
    }
    out
}

/// The per-class half of `CWindowImpl<T>`: state that lives in the object, and the two operations
/// ATL's `WindowProc` needs from it.
pub trait AtlWindowImpl: MessageMap {
    /// `m_dwState` (WINSTATE_DESTROYED is bit 0).
    unsafe fn atl_state(&mut self) -> *mut DWORD;
    /// `m_pCurrentMsg`.
    unsafe fn atl_current_msg(&mut self) -> *mut *const c_void;
    /// `m_pfnSuperWindowProc`.
    unsafe fn atl_super(&mut self) -> *mut WNDPROC;
    /// The thunk `m_pfnSuperWindowProc` may point into (CMainWnd's `m_FrameThunk`), else null.
    unsafe fn atl_super_thunk(&mut self) -> *const CWndProcThunk;
    /// `CWindowImplBaseT::DefWindowProc(uMsg, wParam, lParam)`.
    unsafe fn atl_def_window_proc(&mut self, msg: UINT, w: WPARAM, l: LPARAM) -> LRESULT {
        let hwnd = *self.hwnd_slot() as HWND;
        let sup = *self.atl_super();
        let thunk = self.atl_super_thunk();
        call_super(sup, thunk, hwnd, msg, w, l)
    }
}

const WINSTATE_DESTROYED: DWORD = 0x00000001;

/// `StartWindowProc`: the first message of a window of a `DECLARE`d class. Pops the queued object,
/// binds it, and from then on `WindowProc<T>` is the window's procedure.
pub unsafe extern "system" fn StartWindowProc(hWnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    match _AtlWinModule.ExtractCreateWndData() {
        Some((pThis, binder)) => binder(pThis, hWnd, uMsg, wParam, lParam),
        // ATL: ATLASSERT(pThis != NULL); return 0;
        None => 0,
    }
}

unsafe fn bind_and_dispatch<T: AtlWindowImpl>(pThis: *mut c_void, hWnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    let this = pThis as *mut T;
    SetPropW(hWnd, PROP_THIS.as_ptr(), pThis);
    SetWindowLongPtr(hWnd, GWLP_WNDPROC, window_proc::<T> as unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT as usize as isize);
    *(*this).hwnd_slot() = hWnd as _;
    window_proc::<T>(hWnd, uMsg, wParam, lParam)
}

/// `CWindowImplBaseT::WindowProc`.
unsafe extern "system" fn window_proc<T: AtlWindowImpl>(hWnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    let this = GetPropW(hWnd, PROP_THIS.as_ptr()) as *mut T;
    if this.is_null() {
        return 0;
    }
    if (*(*this).atl_state() & WINSTATE_DESTROYED) != 0 || (*(*this).hwnd_slot()).is_null() {
        return 0;
    }
    let hWnd: HWND = *(*this).hwnd_slot() as HWND;
    let marker: u8 = 0;
    let previousMessage = *(*this).atl_current_msg();
    *(*this).atl_current_msg() = core::ptr::addr_of!(marker) as *const c_void;

    let mut handled_result: isize = 0;
    let handled = (*this).process_window_message(hWnd as _, uMsg, wParam as _, lParam as _, &mut handled_result, 0);
    let mut lResult: LRESULT = handled_result as LRESULT;
    if !handled {
        if uMsg == WM_NCDESTROY {
            let saveWindowProc = GetWindowLongPtr(hWnd, GWLP_WNDPROC);
            lResult = (*this).atl_def_window_proc(uMsg, wParam, lParam);
            let sup = *(*this).atl_super();
            if sup.map(|f| f as usize) != Some(DefWindowProcW as usize) && sup.is_some() && saveWindowProc == GetWindowLongPtr(hWnd, GWLP_WNDPROC) {
                SetWindowLongPtr(hWnd, GWLP_WNDPROC, sup.map_or(0, |f| f as usize) as isize);
            }
            *(*this).atl_state() |= WINSTATE_DESTROYED;
        } else {
            lResult = (*this).atl_def_window_proc(uMsg, wParam, lParam);
        }
    }
    *(*this).atl_current_msg() = previousMessage;
    if previousMessage.is_null() && (*(*this).atl_state() & WINSTATE_DESTROYED) != 0 {
        *(*this).atl_state() &= !WINSTATE_DESTROYED;
        *(*this).hwnd_slot() = core::ptr::null_mut();
        RemovePropW(hWnd, PROP_THIS.as_ptr());
        (*this).on_final_message(hWnd as _);
    }
    lResult
}

/// `CWindowImpl<T>::Create` + `CWindowImplBaseT::Create` for a class whose window class info is
/// `info` and whose thunk is `thunk` (the control-window traits: CControlWinTraits).
#[allow(clippy::too_many_arguments)]
pub unsafe fn window_impl_create<T: AtlWindowImpl>(
    this: *mut T,
    info: &mut CWndClassInfo,
    thunk: *mut CWndProcThunk,
    hWndParent: HWND,
    rect: *mut RECT,
    szWindowName: LPCWSTR,
    mut dwStyle: DWORD,
    mut dwExStyle: DWORD,
    MenuOrID: UINT,
    lpCreateParam: LPVOID,
) -> HWND {
    // atom = T::GetWndClassInfo().Register(&pThis->m_pfnSuperWindowProc);
    let atom = info.Register((*this).atl_super());
    // T::GetWndStyle / GetWndExStyle: CWinTraits<WS_CHILD | WS_VISIBLE | WS_CLIPCHILDREN |
    // WS_CLIPSIBLINGS, 0>::GetWndStyle -- the default only when the caller passed 0.
    if dwStyle == 0 {
        dwStyle = WS_CHILD | WS_VISIBLE | WS_CLIPCHILDREN | WS_CLIPSIBLINGS;
    }
    if dwExStyle == 0 {
        dwExStyle = 0;
    }
    if atom == 0 {
        return core::ptr::null_mut();
    }
    if (*thunk).Init(None, core::ptr::null_mut()) == 0 {
        return core::ptr::null_mut();
    }
    _AtlWinModule.AddCreateWndData(core::ptr::addr_of_mut!((*thunk).cd), this);
    let mut hMenu: HMENU = MenuOrID as usize as HMENU;
    if hMenu.is_null() && (dwStyle & WS_CHILD) != 0 {
        hMenu = this as HMENU;
    }
    // CWindow::rcDefault = { CW_USEDEFAULT, CW_USEDEFAULT, 0, 0 }
    let rc: RECT = if rect.is_null() {
        RECT { left: CW_USEDEFAULT, top: CW_USEDEFAULT, right: 0, bottom: 0 }
    } else {
        *rect
    };
    CreateWindowExW(
        dwExStyle,
        atom as usize as LPCWSTR,
        szWindowName,
        dwStyle,
        rc.left,
        rc.top,
        rc.right.wrapping_sub(rc.left),
        rc.bottom.wrapping_sub(rc.top),
        hWndParent,
        hMenu,
        _AtlBaseModule.GetModuleInstance(),
        lpCreateParam,
    )
}

impl CWindow {
    /// `HWND Create(LPCTSTR lpstrWndClass, HWND hWndParent, _U_RECT rect = NULL, LPCTSTR
    /// szWindowName = NULL, DWORD dwStyle = 0, DWORD dwExStyle = 0, _U_MENUorID MenuOrID = 0U,
    /// LPVOID lpCreateParam = NULL)` -- the header dereferences `rect.m_lpRect` unconditionally.
    #[allow(clippy::too_many_arguments)]
    pub unsafe fn Create(
        &mut self,
        lpstrWndClass: LPCWSTR,
        hWndParent: HWND,
        rect: *mut RECT,
        szWindowName: LPCWSTR,
        dwStyle: DWORD,
        dwExStyle: DWORD,
        MenuOrID: UINT,
        lpCreateParam: LPVOID,
    ) -> HWND {
        let hWnd = CreateWindowExW(
            dwExStyle,
            lpstrWndClass,
            szWindowName,
            dwStyle,
            (*rect).left,
            (*rect).top,
            (*rect).right - (*rect).left,
            (*rect).bottom - (*rect).top,
            hWndParent,
            MenuOrID as usize as HMENU,
            _AtlBaseModule.GetModuleInstance(),
            lpCreateParam,
        );
        if !hWnd.is_null() {
            self.m_hWnd = hWnd;
        }
        hWnd
    }
}

macro_rules! atl_window_impl {
    ($cls:ident, $thunk:expr) => {
        impl AtlWindowImpl for $cls {
            unsafe fn atl_state(&mut self) -> *mut DWORD {
                core::ptr::addr_of_mut!(self.m_dwState)
            }
            unsafe fn atl_current_msg(&mut self) -> *mut *const c_void {
                core::ptr::addr_of_mut!(self.m_pCurrentMsg)
            }
            unsafe fn atl_super(&mut self) -> *mut WNDPROC {
                core::ptr::addr_of_mut!(self.m_pfnSuperWindowProc)
            }
            unsafe fn atl_super_thunk(&mut self) -> *const CWndProcThunk {
                let t: fn(&mut $cls) -> *const CWndProcThunk = $thunk;
                t(self)
            }
        }
        impl $cls {
            /// ATL `DefWindowProc(uMsg, wParam, lParam)`: the superclass procedure.
            pub unsafe fn DefWindowProc(&mut self, msg: UINT, w: WPARAM, l: LPARAM) -> LRESULT {
                self.atl_def_window_proc(msg, w, l)
            }
        }
    };
}
pub(crate) use atl_window_impl;

atl_window_impl!(CMainWnd, |s| core::ptr::addr_of!(s.m_FrameThunk));
atl_window_impl!(CConsoleWnd, |_s| core::ptr::null());

impl CMainWnd {
    /// `CWindowImpl<CMainWnd>::Create(hWndParent, rect, szWindowName, dwStyle, dwExStyle,
    /// MenuOrID, lpCreateParam)`.
    #[allow(clippy::too_many_arguments)]
    pub unsafe fn Create(
        &mut self,
        hWndParent: HWND,
        rect: *mut RECT,
        szWindowName: LPCWSTR,
        dwStyle: DWORD,
        dwExStyle: DWORD,
        MenuOrID: UINT,
        lpCreateParam: LPVOID,
    ) -> HWND {
        let this: *mut CMainWnd = self;
        window_impl_create(
            this,
            CMainWnd::GetWndClassInfo(),
            core::ptr::addr_of_mut!((*this).m_thunk),
            hWndParent,
            rect,
            szWindowName,
            dwStyle,
            dwExStyle,
            MenuOrID,
            lpCreateParam,
        )
    }
}
