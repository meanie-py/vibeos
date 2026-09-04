//! taskswnd.cpp — `CTaskSwitchWnd`, the task-switch / window-list toolbar (the
//! "MSTaskSwWClass" child window that hosts the per-window task buttons inside
//! the tray's task band). Tier-C COM/window-server class: single QI-reachable
//! interface (IOleWindow), factory `CTaskSwitchWnd_CreateInstance`. Follows the
//! trayclock.rs template (CWindowImpl→arch-split WndProc, CComObjectRootEx→
//! refcount+QI, ShellObjectCreatorInit→factory) but adds two EMBEDDED-BY-VALUE
//! C++ sub-objects that make this file heavier than trayclock/taskband:
//!
//!   * `CTaskToolbar m_TaskBar` — a `CWindowImplBaseT<CToolbar<TASK_ITEM>>`: a
//!     comctl32 "ToolbarWindow32" control that is SUBCLASSED so its WM_NCHITTEST
//!     goes transparent off-button. Modelled here as a flat `#[repr(C)]`
//!     sub-object (`CTaskToolbar`) with its OWN arch-split subclass WndProc that
//!     recovers `this` from the toolbar HWND's GWLP_USERDATA and chains to the
//!     saved super-proc via CallWindowProcW. The CToolbar<T> method surface
//!     (TB_* SendMessage wrappers) is reproduced as free fns over the HWND.
//!   * `CHardErrorThread m_HardErrorThread` — a worker thread that pops a
//!     hard-error balloon via IUserNotification. Modelled as a flat sub-object;
//!     the two `CStringW` members become owned heap wide-string buffers.
//!
//! The comctl32 toolbar/custom-draw surface + IUserNotification are declared
//! FILE-LOCAL (only this file needs them today); see the substrate manifest for
//! what should be hoisted to explorer-sys when trayntfy/syspager reuse it.

use core::ffi::c_void;
use explorer_sys::com::{
    CoCreateInstance, IOleWindow, IOleWindowVtbl, ITrayWindow, IUnknown, CLSCTX_INPROC_SERVER,
    E_FAIL, E_INVALIDARG, E_NOINTERFACE, E_NOTIMPL, E_OUTOFMEMORY, E_POINTER, FAILED_UNEXPECTEDLY,
    IID_IOleWindow, IID_IUnknown, REFIID, S_OK,
};
use explorer_sys::globals::*;
use explorer_sys::server::{
    get_window_this, interlocked_decrement, interlocked_increment, register_class_once,
    wndproc_nccreate_this, CloseThemeData, DrawThemeParentBackground, GET_X_LPARAM, GET_Y_LPARAM,
    IsAppThemed, IsThemeActive, OpenThemeData, SetWindowTheme, CW_USEDEFAULT, HTHEME, LOWORD,
};
use crate::explorer::settings::g_TaskbarSettings;
use explorer_sys::types::TaskbarSettings;
use winsrv_sys::w;
use winsrv_sys::winsrv::*;

// ===========================================================================
// File-local substrate (see manifest — candidates to hoist to explorer-sys)
// ===========================================================================

pub type HIMAGELIST = *mut c_void;

// --- precomp.h tray window messages ---
const TSWM_UPDATETASKBARPOS: UINT = WM_USER + 2;
const TWM_SETTINGSCHANGED: UINT = WM_USER + 300;
const TWM_OPENSTARTMENU: UINT = WM_USER + 260;

// --- taskswnd.cpp file-scope constants ---
const MAX_TASKS_COUNT: WORD = 0x7FFF;
const TASK_ITEM_ARRAY_ALLOC: WORD = 64;
const TIMER_ID_VALIDATE_RUDE_APP: UINT_PTR = 5;
const VALIDATE_RUDE_INTERVAL: UINT = 1000;
const VALIDATE_RUDE_MAX_COUNT: INT = 5;

// --- undocshell.h RegisterShellHook types ---
const RSH_DEREGISTER: DWORD = 0;
const RSH_REGISTER_TASKMAN: DWORD = 3;

// --- window / GDI / shell constants used here (shadow winsrv globs if any) ---
const WM_SETREDRAW: UINT = 11;
const WM_QUIT: UINT = 0x0012;
const WM_GETICON: UINT = 0x007F;
const WM_POPUPSYSTEMMENU: UINT = 0x0313;
const GWLP_WNDPROC: INT = -4;
const GWL_STYLE: INT = -16;
const GWL_EXSTYLE: INT = -20;
const GCLP_HICON: INT = -14;
const GCLP_HICONSM: INT = -34;
const ICON_BIG: WPARAM = 1;
const ICON_SMALL: WPARAM = 0;
const ICON_SMALL2: WPARAM = 2;
const SMTO_NOTIMEOUTIFNOTHUNG: UINT = 8;
const SW_MINIMIZE: INT = 6;
const HTTRANSPARENT: LRESULT = -1;
const HTVSCROLL: LRESULT = 7;
const HTHSCROLL: LRESULT = 6;
const MA_NOACTIVATE: LRESULT = 3;
const GW_CHILD: UINT = 5;
const OIC_SAMPLE: u16 = 32512;
const IMAGE_ICON: UINT = 1;
const LR_SHARED: UINT = 32768;
const LR_DEFAULTSIZE: UINT = 64;
const SPI_SETNONCLIENTMETRICS: UINT = 0x002A;
const NIIF_WARNING: DWORD = 2;
const TDP_FLASHBUTTON: INT = 2;
const WAIT_TIMEOUT: DWORD = 0x00000102;
const FAPPCOMMAND_MASK: u32 = 0xF000;
const APPCOMMAND_BROWSER_SEARCH: UINT = 5;
const APPCOMMAND_VOLUME_MUTE: UINT = 8;
const APPCOMMAND_VOLUME_DOWN: UINT = 9;
const APPCOMMAND_VOLUME_UP: UINT = 10;

// --- shell hook notification codes (HSHELL_*) — winuser.h ---
const HSHELL_WINDOWCREATED: INT = 1;
const HSHELL_WINDOWDESTROYED: INT = 2;
const HSHELL_ACTIVATESHELLWINDOW: INT = 3;
const HSHELL_WINDOWACTIVATED: INT = 4;
const HSHELL_TASKMAN: INT = 7;
const HSHELL_REDRAW: INT = 6;
const HSHELL_RUDEAPPACTIVATED: INT = 0x8004; // HSHELL_HIGHBIT | HSHELL_WINDOWACTIVATED
const HSHELL_FLASH: INT = 0x8006; // HSHELL_HIGHBIT | HSHELL_REDRAW
const HSHELL_APPCOMMAND: INT = 12;

// --- comctl32 toolbar messages (commctrl.h; WM_USER == 0x400) ---
const TB_DELETEBUTTON: UINT = WM_USER + 22;
const TB_BUTTONSTRUCTSIZE: UINT = WM_USER + 30;
const TB_GETITEMRECT: UINT = WM_USER + 29;
const TB_SETIMAGELIST: UINT = WM_USER + 48;
const TB_GETBUTTONSIZE: UINT = WM_USER + 58;
const TB_SETBUTTONINFOW: UINT = WM_USER + 64;
const TB_INSERTBUTTONW: UINT = WM_USER + 67;
const TB_HITTEST: UINT = WM_USER + 69;
const TB_GETMETRICS: UINT = WM_USER + 101;
const TB_SETMETRICS: UINT = WM_USER + 102;

// --- toolbar button state / style / info-mask / metrics-mask ---
const TBSTATE_CHECKED: u8 = 0x01;
const TBSTATE_ENABLED: u8 = 0x04;
const TBSTATE_WRAP: u8 = 0x20;
const TBSTATE_ELLIPSES: u8 = 0x40;
const TBSTATE_MARKED: u8 = 0x80;
const TBSTYLE_TOOLTIPS: DWORD = 0x0100;
const TBSTYLE_WRAPABLE: DWORD = 0x0200;
const TBSTYLE_LIST: DWORD = 0x1000;
const TBSTYLE_TRANSPARENT: DWORD = 0x8000;
const BTNS_CHECK: u8 = 0x02; // TBSTYLE_CHECK
const BTNS_NOPREFIX: u8 = 0x20; // TBSTYLE_NOPREFIX
const BTNS_SHOWTEXT: u8 = 0x40;
const TBIF_IMAGE: DWORD = 0x0001;
const TBIF_TEXT: DWORD = 0x0002;
const TBIF_STATE: DWORD = 0x0004;
const TBIF_COMMAND: DWORD = 0x0020;
const TBIF_SIZE: DWORD = 0x0040;
const TBIF_BYINDEX: DWORD = 0x8000_0000;
const TBMF_BARPAD: DWORD = 0x2;
const TBMF_BUTTONSPACING: DWORD = 0x4;
const CCS_TOP: DWORD = 0x1;
const CCS_NORESIZE: DWORD = 0x4;
const CCS_NODIVIDER: DWORD = 0x40;
const ILC_MASK: UINT = 0x1;
const ILC_COLOR32: UINT = 0x20;

// --- custom-draw (NM_CUSTOMDRAW == NM_FIRST-12 == (UINT)-12) ---
const NM_CUSTOMDRAW: UINT = 0u32.wrapping_sub(12);
const CDRF_DODEFAULT: LRESULT = 0x0;
const CDRF_NOTIFYITEMDRAW: LRESULT = 0x20;
const CDDS_PREPAINT: DWORD = 0x1;
const CDDS_ITEM: DWORD = 0x0001_0000;
const CDDS_ITEMPREPAINT: DWORD = CDDS_ITEM | CDDS_PREPAINT;
const CDIS_MARKED: UINT = 0x80;
const TBCDRF_NOBACKGROUND: LRESULT = 0x0040_0000;

// --- structs whose exact layout matters (from headers) ---

/// commctrl.h `TBBUTTON` — `bReserved` sizing is arch-split (6 bytes on _WIN64,
/// 2 on _WIN32) so `dwData`/`iString` land at the ABI offset comctl32 reads.
#[repr(C)]
#[derive(Clone, Copy)]
struct TBBUTTON {
    iBitmap: INT,
    idCommand: INT,
    fsState: u8,
    fsStyle: u8,
    #[cfg(target_pointer_width = "64")]
    bReserved: [u8; 6],
    #[cfg(target_pointer_width = "32")]
    bReserved: [u8; 2],
    dwData: usize,  // DWORD_PTR
    iString: isize, // INT_PTR
}

/// commctrl.h `TBBUTTONINFOW`.
#[repr(C)]
#[derive(Clone, Copy)]
struct TBBUTTONINFOW {
    cbSize: UINT,
    dwMask: DWORD,
    idCommand: INT,
    iImage: INT,
    fsState: u8,
    fsStyle: u8,
    cx: WORD,
    lParam: usize, // DWORD_PTR
    pszText: LPWSTR,
    cchText: INT,
}

/// commctrl.h `TBMETRICS`.
#[repr(C)]
#[derive(Clone, Copy)]
struct TBMETRICS {
    cbSize: UINT,
    dwMask: DWORD,
    cxPad: INT,
    cyPad: INT,
    cxBarPad: INT,
    cyBarPad: INT,
    cxButtonSpacing: INT,
    cyButtonSpacing: INT,
}

/// commctrl.h `NMCUSTOMDRAW` (embeds winsrv `NMHDR`).
#[repr(C)]
struct NMCUSTOMDRAW {
    hdr: NMHDR,
    dwDrawStage: DWORD,
    hdc: HDC,
    rc: RECT,
    dwItemSpec: usize, // DWORD_PTR
    uItemState: UINT,
    lItemlParam: LPARAM,
}

/// commctrl.h `NMTBCUSTOMDRAW`.
#[repr(C)]
struct NMTBCUSTOMDRAW {
    nmcd: NMCUSTOMDRAW,
    hbrMonoDither: HBRUSH,
    hbrLines: HBRUSH,
    hpenLines: HPEN,
    clrText: COLORREF,
    clrMark: COLORREF,
    clrTextHighlight: COLORREF,
    clrBtnFace: COLORREF,
    clrBtnHighlight: COLORREF,
    clrHighlightHotTrack: COLORREF,
    rcText: RECT,
    nStringBkMode: INT,
    nHLStringBkMode: INT,
    iListGap: INT,
}

/// winuser.h `MONITORINFO`.
#[repr(C)]
#[derive(Clone, Copy)]
struct MONITORINFO {
    cbSize: DWORD,
    rcMonitor: RECT,
    rcWork: RECT,
    dwFlags: DWORD,
}

/// winuser.h `COPYDATASTRUCT`.
#[repr(C)]
struct COPYDATASTRUCT {
    dwData: usize, // ULONG_PTR
    cbData: DWORD,
    lpData: *mut c_void,
}

/// undocuser.h `BALLOON_HARD_ERROR_DATA`.
#[repr(C)]
struct BALLOON_HARD_ERROR_DATA {
    cbHeaderSize: DWORD,
    Status: DWORD,
    dwType: DWORD,
    TitleOffset: usize,   // ULONG_PTR
    MessageOffset: usize, // ULONG_PTR
}

// --- IUserNotification (shobjidl.idl) — CONSUMED COM interface ---

static IID_IUserNotification: IID = IID {
    Data1: 0xBA97_11BA,
    Data2: 0x5893,
    Data3: 0x4787,
    Data4: [0xA7, 0xE1, 0x41, 0x27, 0x71, 0x51, 0x55, 0x0B],
};
/// CLSID_UserNotification {0010890e-8789-413c-adbc-48f5b511b3af}.
static CLSID_UserNotification: GUID = GUID {
    Data1: 0x0010_890E,
    Data2: 0x8789,
    Data3: 0x413C,
    Data4: [0xAD, 0xBC, 0x48, 0xF5, 0xB5, 0x11, 0xB3, 0xAF],
};

#[repr(C)]
struct IUserNotificationVtbl {
    QueryInterface:
        unsafe extern "system" fn(*mut IUserNotification, REFIID, *mut *mut c_void) -> HRESULT,
    AddRef: unsafe extern "system" fn(*mut IUserNotification) -> ULONG,
    Release: unsafe extern "system" fn(*mut IUserNotification) -> ULONG,
    SetBalloonInfo:
        unsafe extern "system" fn(*mut IUserNotification, LPCWSTR, LPCWSTR, DWORD) -> HRESULT,
    SetBalloonRetry:
        unsafe extern "system" fn(*mut IUserNotification, DWORD, DWORD, UINT) -> HRESULT,
    SetIconInfo: unsafe extern "system" fn(*mut IUserNotification, HICON, LPCWSTR) -> HRESULT,
    Show: unsafe extern "system" fn(*mut IUserNotification, *mut c_void, DWORD) -> HRESULT,
    PlaySound: unsafe extern "system" fn(*mut IUserNotification, LPCWSTR) -> HRESULT,
}
#[repr(C)]
struct IUserNotification {
    lpVtbl: *const IUserNotificationVtbl,
}

// --- externs beyond the winsrv/explorer-sys surface ---

// comctl32 image list (stdcall)
extern "system" {
    fn ImageList_Create(cx: INT, cy: INT, flags: UINT, cInitial: INT, cGrow: INT) -> HIMAGELIST;
    fn ImageList_ReplaceIcon(himl: HIMAGELIST, i: INT, hicon: HICON) -> INT;
    fn ImageList_Remove(himl: HIMAGELIST, i: INT) -> BOOL;
    fn ImageList_Destroy(himl: HIMAGELIST) -> BOOL;
    fn ImageList_SetIconSize(himl: HIMAGELIST, cx: INT, cy: INT) -> BOOL;
    fn ImageList_GetIconSize(himl: HIMAGELIST, cx: *mut INT, cy: *mut INT) -> BOOL;
}
// user32 / gdi32 (stdcall)
extern "system" {
    fn InternalGetWindowText(hwnd: HWND, pString: LPWSTR, cchMaxCount: INT) -> INT;
    fn SwitchToThisWindow(hwnd: HWND, fUnknown: BOOL);
    fn Rectangle(hdc: HDC, left: INT, top: INT, right: INT, bottom: INT) -> BOOL;
}
// shell32 / shlwapi undoc (stdcall)
extern "system" {
    fn RegisterShellHook(hwnd: HWND, dwType: DWORD) -> BOOL;
    fn SHFindFiles(pidlFolder: *const c_void, pidlSaveFile: *const c_void) -> BOOL;
    fn ShellExec_RunDLLW(hwnd: HWND, hInstance: HINSTANCE, pszCmd: LPCWSTR, nShow: INT);
    fn SHRunIndirectRegClientCommand(hwnd: HWND, pszCmd: LPCWSTR) -> HRESULT;
    fn AssocQueryStringW(
        flags: DWORD,
        strg: DWORD,
        pszAssoc: LPCWSTR,
        pszExtra: LPCWSTR,
        pszOut: LPWSTR,
        pcchOut: *mut DWORD,
    ) -> HRESULT;
    fn SHRegOpenUSKeyW(
        pszPath: LPCWSTR,
        samDesired: DWORD,
        hRelativeUSKey: *mut c_void,
        phNewUSKey: *mut *mut c_void,
        fIgnoreHKCU: BOOL,
    ) -> DWORD;
    fn SHRegQueryUSValueW(
        hUSKey: *mut c_void,
        pszValue: LPCWSTR,
        pdwType: *mut DWORD,
        pvData: *mut c_void,
        pcbData: *mut DWORD,
        fIgnoreHKCU: BOOL,
        pvDefaultData: *mut c_void,
        dwDefaultDataSize: DWORD,
    ) -> DWORD;
    fn SHRegCloseUSKey(hUSKey: *mut c_void) -> DWORD;
}
// ole32 (stdcall)
extern "system" {
    fn OleInitialize(pvReserved: *mut c_void) -> HRESULT;
    fn OleUninitialize();
}
// uxtheme (stdcall)
extern "system" {
    fn DrawThemeBackground(
        hTheme: HTHEME,
        hdc: HDC,
        iPartId: INT,
        iStateId: INT,
        prc: *const RECT,
        pClipRect: *const RECT,
    ) -> HRESULT;
}
// user32 wsprintfW (cdecl variadic).
extern "C" {
    fn wsprintfW(pszDest: LPWSTR, pszFmt: LPCWSTR, ...) -> INT;
}

// ASSOC flags (shlwapi.h)
const ASSOCF_NOTRUNCATE: DWORD = 0x0000_0020;
const ASSOCSTR_EXECUTABLE: DWORD = 2;
const KEY_READ: DWORD = 0x2_0019;
const SW_SHOW: INT = 5;
const HWND_BOTTOM: HWND = 1isize as HWND;

// ---------------------------------------------------------------------------
// small helpers (guid compare, interlocked exchange, arch-split window longs)
// ---------------------------------------------------------------------------

#[inline]
unsafe fn guid_eq(a: REFIID, b: *const IID) -> bool {
    let a = &*a;
    let b = &*b;
    a.Data1 == b.Data1 && a.Data2 == b.Data2 && a.Data3 == b.Data3 && a.Data4 == b.Data4
}

use core::sync::atomic::{AtomicI32, Ordering};
#[inline]
unsafe fn interlocked_exchange(p: *mut i32, v: i32) -> i32 {
    (*(p as *const AtomicI32)).swap(v, Ordering::SeqCst)
}

/// GetSystemMetrics wrapper: the SM_* consts are `u32` in the bindgen surface
/// but the API takes `c_int` — cast once here so call sites stay clean.
#[inline]
unsafe fn gsm(index: u32) -> INT {
    GetSystemMetrics(index as INT)
}

#[cfg(target_arch = "x86")]
#[inline]
unsafe fn get_window_long(hwnd: HWND, idx: INT) -> isize {
    GetWindowLongW(hwnd, idx) as isize
}
#[cfg(not(target_arch = "x86"))]
#[inline]
unsafe fn get_window_long(hwnd: HWND, idx: INT) -> isize {
    GetWindowLongPtrW(hwnd, idx) as isize
}

#[cfg(target_arch = "x86")]
#[inline]
unsafe fn get_class_long(hwnd: HWND, idx: INT) -> usize {
    GetClassLongW(hwnd, idx) as usize
}
#[cfg(not(target_arch = "x86"))]
#[inline]
unsafe fn get_class_long(hwnd: HWND, idx: INT) -> usize {
    GetClassLongPtrW(hwnd, idx) as usize
}

/// Subclass: install `newproc`, returning the previous window proc (the ATL
/// SubclassWindow super-proc). GWLP_WNDPROC store is arch-split like USERDATA.
#[cfg(target_arch = "x86")]
#[inline]
unsafe fn set_wndproc(hwnd: HWND, newproc: WNDPROC) -> WNDPROC {
    let np = core::mem::transmute::<WNDPROC, isize>(newproc);
    let old = SetWindowLongW(hwnd, GWLP_WNDPROC, np as LONG) as isize;
    core::mem::transmute::<isize, WNDPROC>(old)
}
#[cfg(not(target_arch = "x86"))]
#[inline]
unsafe fn set_wndproc(hwnd: HWND, newproc: WNDPROC) -> WNDPROC {
    let np = core::mem::transmute::<WNDPROC, isize>(newproc);
    let old = SetWindowLongPtrW(hwnd, GWLP_WNDPROC, np as LONG_PTR) as isize;
    core::mem::transmute::<isize, WNDPROC>(old)
}

#[inline]
unsafe fn set_window_this(hwnd: HWND, this: *mut c_void) {
    explorer_sys::server::set_window_this(hwnd, this);
}

#[inline]
unsafe fn wcslen_local(mut s: *const u16) -> usize {
    let mut n = 0usize;
    while *s != 0 {
        n += 1;
        s = s.add(1);
    }
    n
}

/// CStringW-ish: heap-dup a wide string into the process heap.
unsafe fn wcsdup_heap(src: *const u16) -> *mut u16 {
    if src.is_null() {
        return core::ptr::null_mut();
    }
    let n = wcslen_local(src) + 1;
    let p = HeapAlloc(hProcessHeap, 0, (n * 2) as SIZE_T) as *mut u16;
    if !p.is_null() {
        core::ptr::copy_nonoverlapping(src, p, n);
    }
    p
}
unsafe fn wcsfree_heap(p: *mut u16) {
    if !p.is_null() {
        HeapFree(hProcessHeap, 0, p as *mut c_void);
    }
}

/// windef.h MAKELPARAM.
#[inline]
fn MAKELPARAM(lo: INT, hi: INT) -> LPARAM {
    ((lo as u16 as u32) | ((hi as u16 as u32) << 16)) as i32 as isize as LPARAM
}

// ===========================================================================
// TASK_GROUP / TASK_ITEM (internal heap structures; layout is private)
// ===========================================================================

#[repr(C)]
struct TASK_GROUP {
    Next: *mut TASK_GROUP,
    dwTaskCount: DWORD,
    dwProcessId: DWORD,
    Index: INT,
    /// union { DWORD dwFlags; struct { DWORD IsCollapsed:1; } } — bit 0.
    dwFlags: DWORD,
}
impl TASK_GROUP {
    const F_ISCOLLAPSED: DWORD = 1 << 0;
    #[inline]
    fn IsCollapsed(&self) -> bool {
        self.dwFlags & Self::F_ISCOLLAPSED != 0
    }
}

#[repr(C)]
struct TASK_ITEM {
    hWnd: HWND,
    Group: *mut TASK_GROUP,
    Index: INT,
    IconIndex: INT,
    /// union { DWORD dwFlags; struct { IsFlashing:1; RenderFlashed:1; } }.
    dwFlags: DWORD,
}
impl TASK_ITEM {
    const F_ISFLASHING: DWORD = 1 << 0;
    const F_RENDERFLASHED: DWORD = 1 << 1;
    #[inline]
    fn RenderFlashed(&self) -> bool {
        self.dwFlags & Self::F_RENDERFLASHED != 0
    }
    #[inline]
    fn set_RenderFlashed(&mut self, v: bool) {
        if v {
            self.dwFlags |= Self::F_RENDERFLASHED;
        } else {
            self.dwFlags &= !Self::F_RENDERFLASHED;
        }
    }
    #[allow(dead_code)]
    #[inline]
    fn set_IsFlashing(&mut self, v: bool) {
        if v {
            self.dwFlags |= Self::F_ISFLASHING;
        } else {
            self.dwFlags &= !Self::F_ISFLASHING;
        }
    }
}

// ===========================================================================
// CHardErrorThread — the hard-error balloon worker (embedded by value)
// ===========================================================================

#[repr(C)]
struct CHardErrorThread {
    m_ThreadId: DWORD,
    m_hThread: HANDLE,
    m_bThreadRunning: LONG,
    m_Status: DWORD,
    m_dwType: DWORD,
    m_Title: *mut u16, // CStringW → owned heap buffer
    m_Text: *mut u16,
}

unsafe fn CHardErrorThread_ctor(this: *mut CHardErrorThread) {
    (*this).m_ThreadId = 0;
    (*this).m_hThread = core::ptr::null_mut();
    (*this).m_bThreadRunning = 0;
    (*this).m_Status = 0;
    (*this).m_dwType = 0;
    (*this).m_Title = core::ptr::null_mut();
    (*this).m_Text = core::ptr::null_mut();
}

/// ~CHardErrorThread: unstick a running Show() then reap the thread.
unsafe fn CHardErrorThread_dtor(this: *mut CHardErrorThread) {
    if (*this).m_bThreadRunning != 0 {
        PostThreadMessageW((*this).m_ThreadId, WM_QUIT, 0, 0);
        let ret = WaitForSingleObject((*this).m_hThread, 3 * 1000);
        if ret == WAIT_TIMEOUT {
            TerminateThread((*this).m_hThread, 0);
        }
        CloseHandle((*this).m_hThread);
    }
    wcsfree_heap((*this).m_Title);
    wcsfree_heap((*this).m_Text);
    (*this).m_Title = core::ptr::null_mut();
    (*this).m_Text = core::ptr::null_mut();
}

unsafe fn CHardErrorThread_ThreadProc(this: *mut CHardErrorThread) -> HRESULT {
    let mut pnotification: *mut IUserNotification = core::ptr::null_mut();

    let mut hr = OleInitialize(core::ptr::null_mut());
    if FAILED_UNEXPECTEDLY(hr) != 0 {
        return hr;
    }

    hr = CoCreateInstance(
        &CLSID_UserNotification,
        core::ptr::null_mut(),
        CLSCTX_INPROC_SERVER,
        &IID_IUserNotification,
        &mut pnotification as *mut *mut IUserNotification as *mut *mut c_void,
    );
    if FAILED_UNEXPECTEDLY(hr) != 0 {
        return hr;
    }

    hr = ((*(*pnotification).lpVtbl).SetBalloonInfo)(
        pnotification,
        (*this).m_Title,
        (*this).m_Text,
        NIIF_WARNING,
    );
    if FAILED_UNEXPECTEDLY(hr) != 0 {
        ((*(*pnotification).lpVtbl).Release)(pnotification);
        return hr;
    }

    hr = ((*(*pnotification).lpVtbl).SetIconInfo)(
        pnotification,
        core::ptr::null_mut(),
        core::ptr::null(),
    );
    if FAILED_UNEXPECTEDLY(hr) != 0 {
        ((*(*pnotification).lpVtbl).Release)(pnotification);
        return hr;
    }

    /* Show blocks until the balloon closes */
    hr = ((*(*pnotification).lpVtbl).Show)(pnotification, core::ptr::null_mut(), 0);

    // CComPtr<IUserNotification> release on scope exit.
    ((*(*pnotification).lpVtbl).Release)(pnotification);
    hr
}

// LPTHREAD_START_ROUTINE is arch-split (stdcall on i386, C otherwise).
unsafe extern "system" fn s_HardErrorThreadProc(lpParameter: *mut c_void) -> DWORD {
    s_HardErrorThreadProc_impl(lpParameter)
}
#[inline]
unsafe fn s_HardErrorThreadProc_impl(lpParameter: *mut c_void) -> DWORD {
    let pThis = lpParameter as *mut CHardErrorThread;
    CHardErrorThread_ThreadProc(pThis);
    CloseHandle((*pThis).m_hThread);
    OleUninitialize();
    interlocked_exchange(&mut (*pThis).m_bThreadRunning, 0);
    0
}

unsafe fn CHardErrorThread_StartThread(this: *mut CHardErrorThread, pData: *mut BALLOON_HARD_ERROR_DATA) {
    let bIsRunning = interlocked_exchange(&mut (*this).m_bThreadRunning, 1);

    /* Ignore the new message if we are already showing one */
    if bIsRunning != 0 {
        return;
    }

    (*this).m_Status = (*pData).Status;
    (*this).m_dwType = (*pData).dwType;

    let base = pData as usize;
    let title_ptr = (base + (*pData).TitleOffset) as *const u16;
    let text_ptr = (base + (*pData).MessageOffset) as *const u16;

    wcsfree_heap((*this).m_Title);
    wcsfree_heap((*this).m_Text);
    (*this).m_Title = wcsdup_heap(title_ptr);
    (*this).m_Text = wcsdup_heap(text_ptr);

    (*this).m_hThread = CreateThread(
        core::ptr::null_mut(),
        0,
        Some(s_HardErrorThreadProc),
        this as *mut c_void,
        0,
        &mut (*this).m_ThreadId,
    );
    if (*this).m_hThread.is_null() {
        (*this).m_bThreadRunning = 0;
    }
}

// ===========================================================================
// CTaskToolbar — the subclassed comctl32 toolbar (embedded by value)
// ===========================================================================

#[repr(C)]
struct CTaskToolbar {
    m_hWnd: HWND,
    m_pfnSuperWindowProc: WNDPROC,
}

unsafe fn CTaskToolbar_ctor(this: *mut CTaskToolbar) {
    (*this).m_hWnd = core::ptr::null_mut();
    (*this).m_pfnSuperWindowProc = None;
}

// --- CToolbar<T> method surface (TB_* SendMessage wrappers over m_hWnd) ---

#[inline]
unsafe fn tb_set_image_list(hwnd: HWND, himl: HIMAGELIST) -> HIMAGELIST {
    SendMessageW(hwnd, TB_SETIMAGELIST, 0, himl as LPARAM) as HIMAGELIST
}
#[inline]
unsafe fn tb_set_button_info(hwnd: HWND, cmd_id: INT, info: *const TBBUTTONINFOW) -> DWORD {
    SendMessageW(hwnd, TB_SETBUTTONINFOW, cmd_id as WPARAM, info as LPARAM) as DWORD
}
#[inline]
unsafe fn tb_insert_button(hwnd: HWND, insert_at: INT, btn: *const TBBUTTON) -> DWORD {
    SendMessageW(hwnd, TB_INSERTBUTTONW, insert_at as WPARAM, btn as LPARAM) as DWORD
}
#[inline]
unsafe fn tb_delete_button(hwnd: HWND, index: INT) -> DWORD {
    SendMessageW(hwnd, TB_DELETEBUTTON, index as WPARAM, 0) as DWORD
}
#[inline]
unsafe fn tb_get_metrics(hwnd: HWND, tbm: *mut TBMETRICS) -> DWORD {
    SendMessageW(hwnd, TB_GETMETRICS, 0, tbm as LPARAM) as DWORD
}
#[inline]
unsafe fn tb_set_metrics(hwnd: HWND, tbm: *mut TBMETRICS) -> DWORD {
    SendMessageW(hwnd, TB_SETMETRICS, 0, tbm as LPARAM) as DWORD
}
#[inline]
unsafe fn tb_get_item_rect(hwnd: HWND, index: INT, prc: *mut RECT) -> DWORD {
    SendMessageW(hwnd, TB_GETITEMRECT, index as WPARAM, prc as LPARAM) as DWORD
}
#[inline]
unsafe fn tb_hit_test(hwnd: HWND, ppt: *mut POINT) -> INT {
    SendMessageW(hwnd, TB_HITTEST, 0, ppt as LPARAM) as INT
}

/// CTaskToolbar::UpdateTbButtonSpacing.
unsafe fn tt_update_button_spacing(
    this: *mut CTaskToolbar,
    b_horizontal: BOOL,
    b_themed: BOOL,
    ui_rows: UINT,
    ui_btns_per_line: UINT,
) -> INT {
    let mut tbm: TBMETRICS = core::mem::zeroed();
    tbm.cbSize = core::mem::size_of::<TBMETRICS>() as UINT;
    tbm.dwMask = TBMF_BARPAD | TBMF_BUTTONSPACING;
    tbm.cxBarPad = 0;
    tbm.cyBarPad = 0;

    if b_themed != 0 {
        tbm.cxButtonSpacing = 0;
        tbm.cyButtonSpacing = 0;
    } else {
        if b_horizontal != 0 || ui_btns_per_line > 1 {
            tbm.cxButtonSpacing = 3 * gsm(SM_CXEDGE) / 2;
        } else {
            tbm.cxButtonSpacing = 0;
        }

        if b_horizontal == 0 || ui_rows > 1 {
            tbm.cyButtonSpacing = 3 * gsm(SM_CYEDGE) / 2;
        } else {
            tbm.cyButtonSpacing = 0;
        }
    }

    tb_set_metrics((*this).m_hWnd, &mut tbm);

    tbm.cxButtonSpacing
}

/// CTaskToolbar::BeginUpdate.
unsafe fn tt_begin_update(this: *mut CTaskToolbar) {
    SendMessageW((*this).m_hWnd, WM_SETREDRAW, 0, 0);
}
/// CTaskToolbar::EndUpdate.
unsafe fn tt_end_update(this: *mut CTaskToolbar) {
    SendMessageW((*this).m_hWnd, WM_SETREDRAW, 1, 0);
    InvalidateRect((*this).m_hWnd, core::ptr::null(), 1);
}

/// CTaskToolbar::SetButtonCommandId.
unsafe fn tt_set_button_command_id(this: *mut CTaskToolbar, i_button_index: INT, i_command_id: INT) -> BOOL {
    let mut tbbi: TBBUTTONINFOW = core::mem::zeroed();
    tbbi.cbSize = core::mem::size_of::<TBBUTTONINFOW>() as UINT;
    tbbi.dwMask = TBIF_BYINDEX | TBIF_COMMAND;
    tbbi.idCommand = i_command_id;
    (tb_set_button_info((*this).m_hWnd, i_button_index, &tbbi) != 0) as BOOL
}

/// CTaskToolbar::OnNcHitTestToolbar — transparent off any button.
unsafe fn tt_on_nc_hit_test(this: *mut CTaskToolbar, l_param: LPARAM) -> (LRESULT, BOOL) {
    let mut pt = POINT {
        x: GET_X_LPARAM(l_param),
        y: GET_Y_LPARAM(l_param),
    };
    ScreenToClient((*this).m_hWnd, &mut pt);

    let index = tb_hit_test((*this).m_hWnd, &mut pt);
    if index < 0 {
        // Make the control appear transparent outside of any buttons.
        (HTTRANSPARENT, 1)
    } else {
        (0, 0) // bHandled=FALSE → super proc
    }
}

// --- the subclass WndProc (ATL SubclassWindow thunk replacement) ---

unsafe extern "system" fn CTaskToolbar_SubclassProc(
    hWnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    CTaskToolbar_subclass_impl(hWnd, uMsg, wParam, lParam)
}

#[inline]
unsafe fn CTaskToolbar_subclass_impl(
    hWnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let this = get_window_this(hWnd) as *mut CTaskToolbar;
    if this.is_null() {
        return DefWindowProcW(hWnd, uMsg, wParam, lParam);
    }

    let super_proc = (*this).m_pfnSuperWindowProc;

    if uMsg == WM_NCHITTEST {
        let (l_result, b_handled) = tt_on_nc_hit_test(this, lParam);
        if b_handled != 0 {
            return l_result;
        }
    }

    CallWindowProcW(super_proc, hWnd, uMsg, wParam, lParam)
}

/// CTaskToolbar::Initialize — create the toolbar, then subclass it.
unsafe fn CTaskToolbar_Initialize(this: *mut CTaskToolbar, hwnd_parent: HWND) -> BOOL {
    let styles: DWORD = WS_CHILD
        | WS_VISIBLE
        | WS_CLIPCHILDREN
        | TBSTYLE_TOOLTIPS
        | TBSTYLE_WRAPABLE
        | TBSTYLE_LIST
        | TBSTYLE_TRANSPARENT
        | CCS_TOP
        | CCS_NORESIZE
        | CCS_NODIVIDER;

    // CToolbar::Create(hWndParent, styles): CreateWindowExW("ToolbarWindow32")
    // + TB_BUTTONSTRUCTSIZE. (HACK & FIXME: CORE-18016.)
    let toolbar = CreateWindowExW(
        WS_EX_TOOLWINDOW,
        w!("ToolbarWindow32").as_ptr(),
        core::ptr::null(),
        styles,
        0,
        0,
        0,
        0,
        hwnd_parent,
        core::ptr::null_mut(),
        hExplorerInstance,
        core::ptr::null_mut(),
    );
    if toolbar.is_null() {
        return 0;
    }
    SendMessageW(
        toolbar,
        TB_BUTTONSTRUCTSIZE,
        core::mem::size_of::<TBBUTTON>() as WPARAM,
        0,
    );

    // m_hWnd = NULL; return SubclassWindow(toolbar);
    (*this).m_hWnd = core::ptr::null_mut();

    // SubclassWindow: attach this to the existing toolbar HWND — stash `this`
    // in the toolbar's GWLP_USERDATA and swap in our subclass proc, saving the
    // original as the super-proc.
    set_window_this(toolbar, this as *mut c_void);
    let old = set_wndproc(toolbar, Some(CTaskToolbar_SubclassProc));
    (*this).m_pfnSuperWindowProc = old;
    (*this).m_hWnd = toolbar;
    1
}

// ===========================================================================
// SHELL fullscreen (rude-app) helpers — file-scope statics in C++
// ===========================================================================

const CHECK_STYLE: DWORD = WS_THICKFRAME | WS_DLGFRAME | WS_BORDER;

unsafe fn SHELL_GetMonitorRect(h_monitor: HMONITOR, prc_dest: *mut RECT, b_work_area_only: BOOL) -> BOOL {
    let mut mi: MONITORINFO = core::mem::zeroed();
    mi.cbSize = core::mem::size_of::<MONITORINFO>() as DWORD;
    if h_monitor.is_null() || GetMonitorInfoW(h_monitor, &mut mi as *mut MONITORINFO as *mut _) == 0 {
        if prc_dest.is_null() {
            return 0;
        }
        if b_work_area_only != 0 {
            SystemParametersInfoW(SPI_GETWORKAREA, 0, prc_dest as *mut c_void, 0);
        } else {
            SetRect(
                prc_dest,
                0,
                0,
                gsm(SM_CXSCREEN),
                gsm(SM_CYSCREEN),
            );
        }
        return 0;
    }

    if !prc_dest.is_null() {
        *prc_dest = if b_work_area_only != 0 { mi.rcWork } else { mi.rcMonitor };
    }
    1
}

unsafe fn SHELL_IsParentOwnerOrSelf(hwnd_target: HWND, mut hwnd: HWND) -> BOOL {
    while !hwnd.is_null() {
        if hwnd == hwnd_target {
            return 1;
        }
        hwnd = GetParent(hwnd);
    }
    0
}

unsafe fn SHELL_IsRudeWindowActive(hwnd: HWND) -> BOOL {
    let hwnd_fore = GetForegroundWindow();
    let dw_thread_id = GetWindowThreadProcessId(hwnd, core::ptr::null_mut());
    (dw_thread_id == GetWindowThreadProcessId(hwnd_fore, core::ptr::null_mut())
        || SHELL_IsParentOwnerOrSelf(hwnd, hwnd_fore) != 0) as BOOL
}

unsafe fn SHELL_IsRudeWindow(h_monitor: HMONITOR, hwnd: HWND, b_dont_check_active: BOOL) -> BOOL {
    if IsWindowVisible(hwnd) == 0 || hwnd == GetDesktopWindow() {
        return 0;
    }

    let mut rc_monitor: RECT = core::mem::zeroed();
    SHELL_GetMonitorRect(h_monitor, &mut rc_monitor, 0);

    let style = get_window_long(hwnd, GWL_STYLE) as DWORD;

    let mut rc_wnd: RECT = core::mem::zeroed();
    if (style & CHECK_STYLE) == CHECK_STYLE {
        GetClientRect(hwnd, &mut rc_wnd); // Ignore frame
        MapWindowPoints(hwnd, core::ptr::null_mut(), &mut rc_wnd as *mut RECT as *mut POINT, 2);
    } else {
        GetWindowRect(hwnd, &mut rc_wnd);
    }

    let mut rc_union: RECT = core::mem::zeroed();
    UnionRect(&mut rc_union, &rc_wnd, &rc_monitor);

    (EqualRect(&rc_union, &rc_wnd) != 0
        && (b_dont_check_active != 0 || SHELL_IsRudeWindowActive(hwnd) != 0)) as BOOL
}

// ===========================================================================
// The object (ATL bases flattened) — single interface (IOleWindow)
// ===========================================================================

#[repr(C)]
pub struct CTaskSwitchWnd {
    // COM identity — IOleWindow vtbl first (IUnknown shares it).
    pub lpVtbl: *const IOleWindowVtbl,
    pub ref_: LONG,     // CComObjectRootEx m_dwRef
    pub m_hWnd: HWND,   // CWindowImpl m_hWnd
    // --- CTaskSwitchWnd members ---
    m_TaskBar: CTaskToolbar,
    m_Tray: *mut ITrayWindow, // CComPtr
    m_ShellHookMsg: UINT,
    m_TaskItemCount: WORD,
    m_AllocatedTaskItems: WORD,
    m_TaskGroups: *mut TASK_GROUP,
    m_TaskItems: *mut TASK_ITEM,
    m_ActiveTaskItem: *mut TASK_ITEM,
    m_Theme: HTHEME,
    m_ButtonsPerLine: UINT,
    m_ButtonCount: WORD,
    m_ImageList: HIMAGELIST,
    m_IsGroupingEnabled: BOOL,
    m_IsDestroying: BOOL,
    m_nRudeAppValidationCounter: INT,
    m_ButtonSize: SIZE,
    m_uHardErrorMsg: UINT,
    m_HardErrorThread: CHardErrorThread,
}

// --- ITrayWindow vtbl call helpers (iface_consumed! only gives QI/AddRef/Rel) ---

#[inline]
unsafe fn tray_get_hwnd(t: *mut ITrayWindow) -> HWND {
    ((*(*t).lpVtbl).GetHWND)(t)
}
#[inline]
unsafe fn tray_is_special_hwnd(t: *mut ITrayWindow, hwnd: HWND) -> BOOL {
    ((*(*t).lpVtbl).IsSpecialHWND)(t, hwnd)
}
#[inline]
unsafe fn tray_is_horizontal(t: *mut ITrayWindow) -> BOOL {
    ((*(*t).lpVtbl).IsHorizontal)(t)
}
#[inline]
unsafe fn tray_is_task_wnd(t: *mut ITrayWindow, hwnd: HWND) -> BOOL {
    ((*(*t).lpVtbl).IsTaskWnd)(t, hwnd)
}
#[inline]
unsafe fn tray_notify_fullscreen(t: *mut ITrayWindow, hmon: HMONITOR, b: BOOL) -> HRESULT {
    ((*(*t).lpVtbl).NotifyFullScreenToAppBars)(t, hmon, b)
}

impl CTaskSwitchWnd {
    #[inline]
    unsafe fn task_bar(&mut self) -> *mut CTaskToolbar {
        &mut self.m_TaskBar
    }
}

// ===========================================================================
// The class methods (formerly CTaskSwitchWnd::*)
// ===========================================================================

unsafe fn GetWndTextFromTaskItem(TaskItem: *mut TASK_ITEM, szBuf: LPWSTR, cchBuf: DWORD) -> INT {
    // No message sent — don't hang on a non-responding app.
    InternalGetWindowText((*TaskItem).hWnd, szBuf, cchBuf as INT)
}

unsafe fn UpdateIndexesAfter(this: *mut CTaskSwitchWnd, iIndex: INT, bInserted: BOOL) {
    let offset: INT = if bInserted != 0 { 1 } else { -1 };

    if (*this).m_IsGroupingEnabled != 0 {
        // Update all affected groups.
        let mut CurrentGroup = (*this).m_TaskGroups;
        while !CurrentGroup.is_null() {
            if (*CurrentGroup).IsCollapsed() && (*CurrentGroup).Index >= iIndex {
                let NewIndex = (*CurrentGroup).Index + offset;
                if tt_set_button_command_id((*this).task_bar(), (*CurrentGroup).Index + offset, NewIndex) != 0 {
                    (*CurrentGroup).Index = NewIndex;
                } else {
                    (*CurrentGroup).Index = -1;
                }
            }
            CurrentGroup = (*CurrentGroup).Next;
        }
    }

    // Update all affected task items.
    let mut CurrentTaskItem = (*this).m_TaskItems;
    let LastTaskItem = CurrentTaskItem.add((*this).m_TaskItemCount as usize);
    while CurrentTaskItem != LastTaskItem {
        let CurrentGroup = (*CurrentTaskItem).Group;
        let do_update = if !CurrentGroup.is_null() {
            !(*CurrentGroup).IsCollapsed() && (*CurrentTaskItem).Index >= iIndex
        } else {
            (*CurrentTaskItem).Index >= iIndex
        };

        if do_update {
            let NewIndex = (*CurrentTaskItem).Index + offset;
            if tt_set_button_command_id((*this).task_bar(), (*CurrentTaskItem).Index + offset, NewIndex) != 0 {
                (*CurrentTaskItem).Index = NewIndex;
            } else {
                (*CurrentTaskItem).Index = -1;
            }
        }

        CurrentTaskItem = CurrentTaskItem.add(1);
    }
}

unsafe fn UpdateTaskGroupButton(TaskGroup: *mut TASK_GROUP) -> INT {
    // FIXME: Implement
    (*TaskGroup).Index
}

unsafe fn ExpandTaskGroup(_TaskGroup: *mut TASK_GROUP) {
    // FIXME: Implement
}

unsafe fn GetWndIcon(hwnd: HWND) -> HICON {
    let mut hIcon: HICON = core::ptr::null_mut();

    // GET_ICON(type): SendMessageTimeout(WM_GETICON) writing hIcon via PDWORD_PTR.
    unsafe fn get_icon(hwnd: HWND, ty: WPARAM, hicon: *mut HICON) -> LRESULT {
        SendMessageTimeoutW(
            hwnd,
            WM_GETICON,
            ty,
            0,
            SMTO_NOTIMEOUTIFNOTHUNG,
            100,
            hicon as *mut usize as PDWORD_PTR,
        )
    }

    let mut bAlive = get_icon(
        hwnd,
        if g_TaskbarSettings.bSmallIcons != 0 { ICON_SMALL2 } else { ICON_BIG },
        &mut hIcon,
    );
    if !hIcon.is_null() {
        return hIcon;
    }

    if bAlive != 0 {
        bAlive = get_icon(hwnd, ICON_SMALL, &mut hIcon);
        if !hIcon.is_null() {
            return hIcon;
        }
    }

    if bAlive != 0 {
        get_icon(
            hwnd,
            if g_TaskbarSettings.bSmallIcons != 0 { ICON_BIG } else { ICON_SMALL2 },
            &mut hIcon,
        );
        if !hIcon.is_null() {
            return hIcon;
        }
    }

    // Fall back to the window class icon.
    hIcon = get_class_long(hwnd, if g_TaskbarSettings.bSmallIcons != 0 { GCLP_HICONSM } else { GCLP_HICON }) as HICON;
    if !hIcon.is_null() {
        return hIcon;
    }

    get_class_long(hwnd, if g_TaskbarSettings.bSmallIcons != 0 { GCLP_HICON } else { GCLP_HICONSM }) as HICON
}

unsafe fn UpdateTaskItemButton(this: *mut CTaskSwitchWnd, TaskItem: *mut TASK_ITEM) -> INT {
    let mut tbbi: TBBUTTONINFOW = core::mem::zeroed();
    let mut windowText: [u16; 255] = [0; 255];

    tbbi.cbSize = core::mem::size_of::<TBBUTTONINFOW>() as UINT;
    tbbi.dwMask = TBIF_BYINDEX | TBIF_STATE | TBIF_TEXT | TBIF_IMAGE;
    tbbi.fsState = TBSTATE_ENABLED;
    if (*this).m_ActiveTaskItem == TaskItem {
        tbbi.fsState |= TBSTATE_CHECKED;
    }
    if (*TaskItem).RenderFlashed() {
        tbbi.fsState |= TBSTATE_MARKED;
    }

    // Last button in a line needs TBSTATE_WRAP.
    if tray_is_horizontal((*this).m_Tray) == 0
        || ((*this).m_ButtonsPerLine != 0 && ((*TaskItem).Index + 1) as UINT % (*this).m_ButtonsPerLine == 0)
    {
        tbbi.fsState |= TBSTATE_WRAP;
    }

    if GetWndTextFromTaskItem(TaskItem, windowText.as_mut_ptr(), windowText.len() as DWORD) > 0 {
        tbbi.pszText = windowText.as_mut_ptr();
    }

    let mut icon = GetWndIcon((*TaskItem).hWnd);
    if icon.is_null() {
        icon = LoadImageW(
            core::ptr::null_mut(),
            explorer_sys::MAKEINTRESOURCEW(OIC_SAMPLE),
            IMAGE_ICON,
            0,
            0,
            LR_SHARED | LR_DEFAULTSIZE,
        ) as HICON;
    }
    (*TaskItem).IconIndex = ImageList_ReplaceIcon((*this).m_ImageList, (*TaskItem).IconIndex, icon);
    tbbi.iImage = (*TaskItem).IconIndex;

    if tb_set_button_info((*this).m_TaskBar.m_hWnd, (*TaskItem).Index, &tbbi) == 0 {
        (*TaskItem).Index = -1;
        return -1;
    }

    (*TaskItem).Index
}

unsafe fn RemoveIcon(this: *mut CTaskSwitchWnd, TaskItem: *mut TASK_ITEM) {
    if (*TaskItem).IconIndex == -1 {
        return;
    }

    let mut tbbi: TBBUTTONINFOW = core::mem::zeroed();
    tbbi.cbSize = core::mem::size_of::<TBBUTTONINFOW>() as UINT;
    tbbi.dwMask = TBIF_IMAGE;

    let mut currentTaskItem = (*this).m_TaskItems;
    let LastItem = currentTaskItem.add((*this).m_TaskItemCount as usize);
    while currentTaskItem != LastItem {
        if (*currentTaskItem).IconIndex > (*TaskItem).IconIndex {
            (*currentTaskItem).IconIndex -= 1;
            tbbi.iImage = (*currentTaskItem).IconIndex;
            tb_set_button_info((*this).m_TaskBar.m_hWnd, (*currentTaskItem).Index, &tbbi);
        }
        currentTaskItem = currentTaskItem.add(1);
    }

    ImageList_Remove((*this).m_ImageList, (*TaskItem).IconIndex);
}

unsafe fn FindLastTaskItemOfGroup(
    this: *mut CTaskSwitchWnd,
    TaskGroup: *mut TASK_GROUP,
    NewTaskItem: *mut TASK_ITEM,
) -> *mut TASK_ITEM {
    let mut FoundTaskItem: *mut TASK_ITEM = core::ptr::null_mut();

    let mut TaskItem = (*this).m_TaskItems;
    let LastTaskItem = TaskItem.add((*this).m_TaskItemCount as usize);

    let mut dwTaskCount: DWORD = if !TaskGroup.is_null() {
        (*TaskGroup).dwTaskCount
    } else {
        MAX_TASKS_COUNT as DWORD
    };

    while TaskItem != LastTaskItem {
        if (*TaskItem).Group == TaskGroup {
            if (!NewTaskItem.is_null() && TaskItem != NewTaskItem) || NewTaskItem.is_null() {
                FoundTaskItem = TaskItem;
            }

            dwTaskCount -= 1;
            if dwTaskCount == 0 {
                break;
            }
        }
        TaskItem = TaskItem.add(1);
    }

    FoundTaskItem
}

unsafe fn CalculateTaskItemNewButtonIndex(this: *mut CTaskSwitchWnd, TaskItem: *mut TASK_ITEM) -> INT {
    // NOTE: assumes the group is *not* collapsed.
    let TaskGroup = (*TaskItem).Group;
    if (*this).m_IsGroupingEnabled != 0 {
        if !TaskGroup.is_null() {
            if (*TaskGroup).dwTaskCount > 1 {
                let LastTaskItem = FindLastTaskItemOfGroup(this, TaskGroup, TaskItem);
                if !LastTaskItem.is_null() {
                    return (*LastTaskItem).Index + 1;
                }
            }
        } else {
            let LastTaskItem = FindLastTaskItemOfGroup(this, core::ptr::null_mut(), TaskItem);
            if !LastTaskItem.is_null() {
                return (*LastTaskItem).Index + 1;
            }
        }
    }

    (*this).m_ButtonCount as INT
}

unsafe fn AddTaskItemButton(this: *mut CTaskSwitchWnd, TaskItem: *mut TASK_ITEM) -> INT {
    let mut windowText: [u16; 255] = [0; 255];
    let mut tbBtn: TBBUTTON = core::mem::zeroed();

    if (*TaskItem).Index >= 0 {
        return UpdateTaskItemButton(this, TaskItem);
    }

    if !(*TaskItem).Group.is_null() && (*(*TaskItem).Group).IsCollapsed() {
        // Group collapsed — only update the group button.
        return UpdateTaskGroupButton((*TaskItem).Group);
    }

    let mut icon = GetWndIcon((*TaskItem).hWnd);
    if icon.is_null() {
        icon = LoadImageW(
            core::ptr::null_mut(),
            explorer_sys::MAKEINTRESOURCEW(OIC_SAMPLE),
            IMAGE_ICON,
            0,
            0,
            LR_SHARED | LR_DEFAULTSIZE,
        ) as HICON;
    }
    (*TaskItem).IconIndex = ImageList_ReplaceIcon((*this).m_ImageList, -1, icon);

    tbBtn.iBitmap = (*TaskItem).IconIndex;
    tbBtn.fsState = TBSTATE_ENABLED | TBSTATE_ELLIPSES;
    tbBtn.fsStyle = BTNS_CHECK | BTNS_NOPREFIX | BTNS_SHOWTEXT;
    tbBtn.dwData = (*TaskItem).Index as usize;

    if GetWndTextFromTaskItem(TaskItem, windowText.as_mut_ptr(), windowText.len() as DWORD) > 0 {
        tbBtn.iString = windowText.as_ptr() as isize;
    }

    // Find out where to insert the new button.
    let iIndex = CalculateTaskItemNewButtonIndex(this, TaskItem);
    tbBtn.idCommand = iIndex;

    tt_begin_update((*this).task_bar());

    if tb_insert_button((*this).m_TaskBar.m_hWnd, iIndex, &tbBtn) != 0 {
        UpdateIndexesAfter(this, iIndex, 1);

        (*TaskItem).Index = iIndex;
        (*this).m_ButtonCount += 1;

        // Update button sizes and fix the button wrapping.
        UpdateButtonsSize(this, 1);
        return iIndex;
    }

    tt_end_update((*this).task_bar());

    -1
}

unsafe fn DeleteTaskItemButton(this: *mut CTaskSwitchWnd, TaskItem: *mut TASK_ITEM) -> BOOL {
    let TaskGroup = (*TaskItem).Group;

    if (*TaskItem).Index >= 0 {
        if (!TaskGroup.is_null() && !(*TaskGroup).IsCollapsed()) || TaskGroup.is_null() {
            tt_begin_update((*this).task_bar());

            RemoveIcon(this, TaskItem);
            let iIndex = (*TaskItem).Index;
            if tb_delete_button((*this).m_TaskBar.m_hWnd, iIndex) != 0 {
                (*TaskItem).Index = -1;
                (*this).m_ButtonCount -= 1;

                UpdateIndexesAfter(this, iIndex, 0);

                UpdateButtonsSize(this, 1);
                return 1;
            }

            tt_end_update((*this).task_bar());
        }
    }

    0
}

unsafe fn AddToTaskGroup(this: *mut CTaskSwitchWnd, hWnd: HWND) -> *mut TASK_GROUP {
    let mut dwProcessId: DWORD = 0;
    if GetWindowThreadProcessId(hWnd, &mut dwProcessId) == 0 {
        return core::ptr::null_mut();
    }

    // Try to find an existing task group.
    let mut TaskGroup = (*this).m_TaskGroups;
    let mut PrevLink: *mut *mut TASK_GROUP = &mut (*this).m_TaskGroups;
    while !TaskGroup.is_null() {
        if (*TaskGroup).dwProcessId == dwProcessId {
            (*TaskGroup).dwTaskCount += 1;
            return TaskGroup;
        }
        PrevLink = &mut (*TaskGroup).Next;
        TaskGroup = (*TaskGroup).Next;
    }

    // Allocate a new task group.
    TaskGroup = HeapAlloc(
        hProcessHeap,
        HEAP_ZERO_MEMORY,
        core::mem::size_of::<TASK_GROUP>() as SIZE_T,
    ) as *mut TASK_GROUP;
    if !TaskGroup.is_null() {
        (*TaskGroup).dwTaskCount = 1;
        (*TaskGroup).dwProcessId = dwProcessId;
        (*TaskGroup).Index = -1;

        *PrevLink = TaskGroup;
    }

    TaskGroup
}

unsafe fn RemoveTaskFromTaskGroup(this: *mut CTaskSwitchWnd, TaskItem: *mut TASK_ITEM) {
    let TaskGroup = (*TaskItem).Group;
    if TaskGroup.is_null() {
        return;
    }

    (*TaskGroup).dwTaskCount -= 1;
    let dwNewTaskCount = (*TaskGroup).dwTaskCount;
    if dwNewTaskCount == 0 {
        // Find the previous pointer in the chain.
        let mut CurrentGroup = (*this).m_TaskGroups;
        let mut PrevLink: *mut *mut TASK_GROUP = &mut (*this).m_TaskGroups;
        while CurrentGroup != TaskGroup {
            PrevLink = &mut (*CurrentGroup).Next;
            CurrentGroup = (*CurrentGroup).Next;
        }

        *PrevLink = (*TaskGroup).Next;
        HeapFree(hProcessHeap, 0, TaskGroup as *mut c_void);
    } else if (*TaskGroup).IsCollapsed() && (*TaskGroup).Index >= 0 {
        if dwNewTaskCount > 1 {
            UpdateTaskGroupButton(TaskGroup);
        } else {
            ExpandTaskGroup(TaskGroup);
        }
    }
}

unsafe fn FindTaskItem(this: *mut CTaskSwitchWnd, hWnd: HWND) -> *mut TASK_ITEM {
    let mut TaskItem = (*this).m_TaskItems;
    let LastItem = TaskItem.add((*this).m_TaskItemCount as usize);
    while TaskItem != LastItem {
        if (*TaskItem).hWnd == hWnd {
            return TaskItem;
        }
        TaskItem = TaskItem.add(1);
    }
    core::ptr::null_mut()
}

unsafe fn FindOtherTaskItem(this: *mut CTaskSwitchWnd, hWnd: HWND) -> *mut TASK_ITEM {
    let mut dwProcessId: DWORD = 0;
    if GetWindowThreadProcessId(hWnd, &mut dwProcessId) == 0 {
        return core::ptr::null_mut();
    }

    let mut TaskItem = (*this).m_TaskItems;
    let LastItem = TaskItem.add((*this).m_TaskItemCount as usize);
    while TaskItem != LastItem {
        let TaskGroup = (*TaskItem).Group;
        if !TaskGroup.is_null() {
            if (*TaskGroup).dwProcessId == dwProcessId {
                return TaskItem;
            }
        } else {
            let mut dwProcessIdTask: DWORD = 0;
            if GetWindowThreadProcessId((*TaskItem).hWnd, &mut dwProcessIdTask) != 0
                && dwProcessIdTask == dwProcessId
            {
                return TaskItem;
            }
        }
        TaskItem = TaskItem.add(1);
    }

    core::ptr::null_mut()
}

unsafe fn AllocTaskItem(this: *mut CTaskSwitchWnd) -> *mut TASK_ITEM {
    if (*this).m_TaskItemCount >= MAX_TASKS_COUNT {
        // MSB of the 16-bit command ID flags task-group vs task-item.
        return core::ptr::null_mut();
    }

    if (*this).m_TaskItemCount == 0 {
        (*this).m_TaskItems = HeapAlloc(
            hProcessHeap,
            0,
            TASK_ITEM_ARRAY_ALLOC as SIZE_T * core::mem::size_of::<TASK_ITEM>() as SIZE_T,
        ) as *mut TASK_ITEM;
        if !(*this).m_TaskItems.is_null() {
            (*this).m_AllocatedTaskItems = TASK_ITEM_ARRAY_ALLOC;
        } else {
            return core::ptr::null_mut();
        }
    } else if (*this).m_TaskItemCount >= (*this).m_AllocatedTaskItems {
        let NewArrayLength = (*this).m_AllocatedTaskItems as SIZE_T + TASK_ITEM_ARRAY_ALLOC as SIZE_T;

        let NewArray = HeapReAlloc(
            hProcessHeap,
            0,
            (*this).m_TaskItems as *mut c_void,
            NewArrayLength * core::mem::size_of::<TASK_ITEM>() as SIZE_T,
        ) as *mut TASK_ITEM;
        if !NewArray.is_null() {
            if !(*this).m_ActiveTaskItem.is_null() {
                let ActiveTaskItemIndex = (*this).m_ActiveTaskItem.offset_from((*this).m_TaskItems);
                (*this).m_ActiveTaskItem = NewArray.offset(ActiveTaskItemIndex);
            }
            (*this).m_AllocatedTaskItems = NewArrayLength as WORD;
            (*this).m_TaskItems = NewArray;
        } else {
            return core::ptr::null_mut();
        }
    }

    let p = (*this).m_TaskItems.add((*this).m_TaskItemCount as usize);
    (*this).m_TaskItemCount += 1;
    p
}

unsafe fn FreeTaskItem(this: *mut CTaskSwitchWnd, TaskItem: *mut TASK_ITEM) {
    if TaskItem == (*this).m_ActiveTaskItem {
        (*this).m_ActiveTaskItem = core::ptr::null_mut();
    }

    let wIndex = TaskItem.offset_from((*this).m_TaskItems) as WORD;
    if (wIndex + 1) < (*this).m_TaskItemCount {
        core::ptr::copy(
            TaskItem.add(1),
            TaskItem,
            ((*this).m_TaskItemCount - wIndex - 1) as usize,
        );
    }

    (*this).m_TaskItemCount -= 1;
}

unsafe fn DeleteTaskItem(this: *mut CTaskSwitchWnd, TaskItem: *mut TASK_ITEM) {
    if (*this).m_IsDestroying == 0 {
        DeleteTaskItemButton(this, TaskItem);
    }
    RemoveTaskFromTaskGroup(this, TaskItem);
    FreeTaskItem(this, TaskItem);
}

unsafe fn CheckActivateTaskItem(this: *mut CTaskSwitchWnd, TaskItem: *mut TASK_ITEM) {
    let CurrentTaskItem = (*this).m_ActiveTaskItem;

    let mut TaskGroup: *mut TASK_GROUP = core::ptr::null_mut();
    if !TaskItem.is_null() {
        TaskGroup = (*TaskItem).Group;
    }

    if (*this).m_IsGroupingEnabled != 0 && !TaskGroup.is_null() && (*TaskGroup).IsCollapsed() {
        // FIXME
        return;
    }

    if !CurrentTaskItem.is_null() {
        if CurrentTaskItem == TaskItem {
            return;
        }

        let CurrentTaskGroup = (*CurrentTaskItem).Group;

        if (*this).m_IsGroupingEnabled != 0
            && !CurrentTaskGroup.is_null()
            && (*CurrentTaskGroup).IsCollapsed()
        {
            if CurrentTaskGroup == TaskGroup {
                return;
            }
            // FIXME
        } else {
            (*this).m_ActiveTaskItem = core::ptr::null_mut();
            if (*CurrentTaskItem).Index >= 0 {
                UpdateTaskItemButton(this, CurrentTaskItem);
            }
        }
    }

    (*this).m_ActiveTaskItem = TaskItem;

    if !TaskItem.is_null() && (*TaskItem).Index >= 0 {
        UpdateTaskItemButton(this, TaskItem);
    }
}

unsafe fn FindTaskItemByIndex(this: *mut CTaskSwitchWnd, Index: INT) -> *mut TASK_ITEM {
    let mut TaskItem = (*this).m_TaskItems;
    let LastItem = TaskItem.add((*this).m_TaskItemCount as usize);
    while TaskItem != LastItem {
        if (*TaskItem).Index == Index {
            return TaskItem;
        }
        TaskItem = TaskItem.add(1);
    }
    core::ptr::null_mut()
}

unsafe fn FindTaskGroupByIndex(this: *mut CTaskSwitchWnd, Index: INT) -> *mut TASK_GROUP {
    let mut CurrentGroup = (*this).m_TaskGroups;
    while !CurrentGroup.is_null() {
        if (*CurrentGroup).Index == Index {
            break;
        }
        CurrentGroup = (*CurrentGroup).Next;
    }
    CurrentGroup
}

unsafe fn AddTask(this: *mut CTaskSwitchWnd, hWnd: HWND) -> BOOL {
    if IsWindow(hWnd) == 0 || tray_is_special_hwnd((*this).m_Tray, hWnd) != 0 {
        return 0;
    }

    let mut TaskItem = FindTaskItem(this, hWnd);
    if TaskItem.is_null() {
        TaskItem = AllocTaskItem(this);
        if !TaskItem.is_null() {
            core::ptr::write_bytes(TaskItem, 0, 1);
            (*TaskItem).hWnd = hWnd;
            (*TaskItem).Index = -1;
            (*TaskItem).Group = AddToTaskGroup(this, hWnd);

            if (*this).m_IsDestroying == 0 {
                AddTaskItemButton(this, TaskItem);
            }
        }
    }

    (!TaskItem.is_null()) as BOOL
}

unsafe fn ActivateTaskItem(this: *mut CTaskSwitchWnd, TaskItem: *mut TASK_ITEM) -> BOOL {
    CheckActivateTaskItem(this, TaskItem);
    0
}

unsafe fn ActivateTask(this: *mut CTaskSwitchWnd, hWnd: HWND) -> BOOL {
    if hWnd.is_null() {
        return ActivateTaskItem(this, core::ptr::null_mut());
    }

    let mut TaskItem = FindTaskItem(this, hWnd);
    if TaskItem.is_null() {
        TaskItem = FindOtherTaskItem(this, hWnd);
    }

    if TaskItem.is_null() {
        RefreshWindowList(this);
    }

    ActivateTaskItem(this, TaskItem)
}

unsafe fn DeleteTask(this: *mut CTaskSwitchWnd, hWnd: HWND) -> BOOL {
    let TaskItem = FindTaskItem(this, hWnd);
    if !TaskItem.is_null() {
        DeleteTaskItem(this, TaskItem);
        return 1;
    }
    0
}

unsafe fn DeleteAllTasks(this: *mut CTaskSwitchWnd) {
    if (*this).m_TaskItemCount > 0 {
        let mut CurrentTask = (*this).m_TaskItems.add((*this).m_TaskItemCount as usize);
        loop {
            CurrentTask = CurrentTask.sub(1);
            DeleteTaskItem(this, CurrentTask);
            if CurrentTask == (*this).m_TaskItems {
                break;
            }
        }
    }
}

unsafe fn FlashTaskItem(this: *mut CTaskSwitchWnd, TaskItem: *mut TASK_ITEM) {
    (*TaskItem).set_RenderFlashed(true);
    UpdateTaskItemButton(this, TaskItem);
}

unsafe fn FlashTask(this: *mut CTaskSwitchWnd, hWnd: HWND) -> BOOL {
    let TaskItem = FindTaskItem(this, hWnd);
    if !TaskItem.is_null() {
        FlashTaskItem(this, TaskItem);
        return 1;
    }
    0
}

unsafe fn RedrawTaskItem(this: *mut CTaskSwitchWnd, TaskItem: *mut TASK_ITEM) {
    let TaskGroup = (*TaskItem).Group;

    let mut do_update_item = false;
    if (*this).m_IsGroupingEnabled != 0 && !TaskGroup.is_null() {
        if (*TaskGroup).IsCollapsed() && (*TaskGroup).Index >= 0 {
            UpdateTaskGroupButton(TaskGroup);
        } else if (*TaskItem).Index >= 0 {
            do_update_item = true;
        }
    } else if (*TaskItem).Index >= 0 {
        do_update_item = true;
    }

    if do_update_item {
        (*TaskItem).set_RenderFlashed(false);
        UpdateTaskItemButton(this, TaskItem);
    }
}

unsafe fn RedrawTask(this: *mut CTaskSwitchWnd, hWnd: HWND) -> BOOL {
    let TaskItem = FindTaskItem(this, hWnd);
    if !TaskItem.is_null() {
        RedrawTaskItem(this, TaskItem);
        return 1;
    }
    0
}

unsafe fn UpdateButtonsSize(this: *mut CTaskSwitchWnd, bRedrawDisabled: BOOL) {
    let mut rcClient: RECT = core::mem::zeroed();

    // Update the image-list icon size if needed.
    let mut cx: INT = 0;
    let mut cy: INT = 0;
    ImageList_GetIconSize((*this).m_ImageList, &mut cx, &mut cy);
    let want_cx = gsm(if g_TaskbarSettings.bSmallIcons != 0 { SM_CXSMICON } else { SM_CXICON });
    let want_cy = gsm(if g_TaskbarSettings.bSmallIcons != 0 { SM_CYSMICON } else { SM_CYICON });
    if cx != want_cx || cy != want_cy {
        ImageList_SetIconSize((*this).m_ImageList, want_cx, want_cy);

        // SetIconSize removes all icons; reinsert them.
        let mut TaskItem = (*this).m_TaskItems;
        let LastTaskItem = (*this).m_TaskItems.add((*this).m_TaskItemCount as usize);
        while TaskItem != LastTaskItem {
            (*TaskItem).IconIndex = -1;
            UpdateTaskItemButton(this, TaskItem);
            TaskItem = TaskItem.add(1);
        }
        tb_set_image_list((*this).m_TaskBar.m_hWnd, (*this).m_ImageList);
    }

    if GetClientRect((*this).m_hWnd, &mut rcClient) != 0 && IsRectEmpty(&rcClient) == 0 {
        if (*this).m_ButtonCount > 0 {
            let Horizontal = tray_is_horizontal((*this).m_Tray);

            let mut uiRows: UINT;
            let mut uiBtnsPerLine: UINT;
            if Horizontal != 0 {
                let mut tbm: TBMETRICS = core::mem::zeroed();
                tbm.cbSize = core::mem::size_of::<TBMETRICS>() as UINT;
                tbm.dwMask = TBMF_BUTTONSPACING;
                tb_get_metrics((*this).m_TaskBar.m_hWnd, &mut tbm);

                if (*this).m_ButtonSize.cy + tbm.cyButtonSpacing != 0 {
                    uiRows = ((rcClient.bottom + tbm.cyButtonSpacing)
                        / ((*this).m_ButtonSize.cy + tbm.cyButtonSpacing)) as UINT;
                } else {
                    uiRows = 1;
                }

                if uiRows == 0 {
                    uiRows = 1;
                }

                uiBtnsPerLine = ((*this).m_ButtonCount as UINT + uiRows - 1) / uiRows;
            } else {
                uiBtnsPerLine = 1;
                uiRows = (*this).m_ButtonCount as UINT;
            }

            if bRedrawDisabled == 0 {
                tt_begin_update((*this).task_bar());
            }

            let cxButtonSpacing = tt_update_button_spacing(
                (*this).task_bar(),
                Horizontal,
                (!(*this).m_Theme.is_null()) as BOOL,
                uiRows,
                uiBtnsPerLine,
            );

            let uiMin = (gsm(SM_CXSIZE) + 2 * gsm(SM_CXEDGE)) as UINT;
            let mut NewBtnSize: LONG;
            let uiMax: UINT;
            if Horizontal != 0 {
                uiMax = gsm(SM_CXMINIMIZED) as UINT;

                NewBtnSize = (rcClient.right - (uiBtnsPerLine as INT * cxButtonSpacing)) / uiBtnsPerLine as INT;

                if NewBtnSize < uiMin as LONG {
                    NewBtnSize = uiMin as LONG;
                }
                if NewBtnSize > uiMax as LONG {
                    NewBtnSize = uiMax as LONG;
                }

                uiBtnsPerLine = (rcClient.right / (NewBtnSize + cxButtonSpacing)) as UINT;
                if uiBtnsPerLine == 0 {
                    uiBtnsPerLine += 1;
                }
            } else {
                NewBtnSize = rcClient.right;
            }

            (*this).m_ButtonSize.cx = NewBtnSize;
            (*this).m_ButtonsPerLine = uiBtnsPerLine;

            let mut ui: UINT = 0;
            while ui != (*this).m_ButtonCount as UINT {
                let mut tbbi: TBBUTTONINFOW = core::mem::zeroed();
                tbbi.cbSize = core::mem::size_of::<TBBUTTONINFOW>() as UINT;
                tbbi.dwMask = TBIF_BYINDEX | TBIF_SIZE | TBIF_STATE;
                tbbi.cx = NewBtnSize as WORD;
                tbbi.fsState = TBSTATE_ENABLED;

                if Horizontal != 0 {
                    if (ui + 1) % uiBtnsPerLine == 0 {
                        tbbi.fsState |= TBSTATE_WRAP;
                    }
                } else {
                    tbbi.fsState |= TBSTATE_WRAP;
                }

                if !(*this).m_ActiveTaskItem.is_null()
                    && (*(*this).m_ActiveTaskItem).Index == ui as INT
                {
                    tbbi.fsState |= TBSTATE_CHECKED;
                }

                tb_set_button_info((*this).m_TaskBar.m_hWnd, ui as INT, &tbbi);
                ui += 1;
            }
        } else {
            (*this).m_ButtonsPerLine = 0;
            (*this).m_ButtonSize.cx = 0;
        }
    }

    // FIXME: enables redraws prematurely (upstream comment).
    tt_end_update((*this).task_bar());
}

// --- EnumWindows refresh (arch-split WNDENUMPROC) ---

unsafe fn EnumWindowsProc(this: *mut CTaskSwitchWnd, hWnd: HWND) -> BOOL {
    if tray_is_task_wnd((*this).m_Tray, hWnd) != 0 {
        AddTask(this, hWnd);
    }
    1
}

unsafe extern "system" fn s_EnumWindowsProc(hWnd: HWND, lParam: LPARAM) -> BOOL {
    EnumWindowsProc(lParam as *mut CTaskSwitchWnd, hWnd)
}

unsafe fn RefreshWindowList(this: *mut CTaskSwitchWnd) -> BOOL {
    EnumWindows(Some(s_EnumWindowsProc), this as LPARAM)
}

// ===========================================================================
// Message handlers (MESSAGE_HANDLER bodies)
// ===========================================================================

unsafe fn OnThemeChanged(this: *mut CTaskSwitchWnd, _uMsg: UINT, _wParam: WPARAM, _lParam: LPARAM, _bHandled: *mut BOOL) -> LRESULT {
    if !(*this).m_Theme.is_null() {
        CloseThemeData((*this).m_Theme);
    }

    if IsThemeActive() != 0 {
        (*this).m_Theme = OpenThemeData((*this).m_hWnd, w!("TaskBand").as_ptr());
    } else {
        (*this).m_Theme = core::ptr::null_mut();
    }

    1
}

unsafe fn OnCreate(this: *mut CTaskSwitchWnd, _uMsg: UINT, _wParam: WPARAM, _lParam: LPARAM, _bHandled: *mut BOOL) -> LRESULT {
    if CTaskToolbar_Initialize((*this).task_bar(), (*this).m_hWnd) == 0 {
        return 0; // FALSE
    }

    SetWindowTheme((*this).m_TaskBar.m_hWnd, w!("TaskBand").as_ptr(), core::ptr::null());

    (*this).m_ImageList = ImageList_Create(
        gsm(if g_TaskbarSettings.bSmallIcons != 0 { SM_CXSMICON } else { SM_CXICON }),
        gsm(if g_TaskbarSettings.bSmallIcons != 0 { SM_CYSMICON } else { SM_CYICON }),
        ILC_COLOR32 | ILC_MASK,
        0,
        1000,
    );
    tb_set_image_list((*this).m_TaskBar.m_hWnd, (*this).m_ImageList);

    // Set proper spacing between buttons.
    tt_update_button_spacing((*this).task_bar(), tray_is_horizontal((*this).m_Tray), (!(*this).m_Theme.is_null()) as BOOL, 0, 0);

    // Register the shell hook.
    (*this).m_ShellHookMsg = RegisterWindowMessageW(w!("SHELLHOOK").as_ptr());

    RegisterShellHook((*this).m_hWnd, RSH_REGISTER_TASKMAN);

    RefreshWindowList(this);

    UpdateButtonsSize(this, 0);

    1
}

unsafe fn OnDestroy(this: *mut CTaskSwitchWnd, _uMsg: UINT, _wParam: WPARAM, _lParam: LPARAM, _bHandled: *mut BOOL) -> LRESULT {
    (*this).m_IsDestroying = 1;

    KillTimer((*this).m_hWnd, TIMER_ID_VALIDATE_RUDE_APP);

    RegisterShellHook((*this).m_hWnd, RSH_DEREGISTER);

    CloseThemeData((*this).m_Theme);
    DeleteAllTasks(this);

    if !(*this).m_ImageList.is_null() {
        ImageList_Destroy((*this).m_ImageList);
        (*this).m_ImageList = core::ptr::null_mut();
    }

    1
}

unsafe fn InvokeRegistryAppKeyCommand(uAppCmd: UINT) -> BOOL {
    let mut bResult: BOOL = 0;
    let mut szBuf: [u16; MAX_PATH as usize * 2] = [0; MAX_PATH as usize * 2];

    wsprintfW(
        szBuf.as_mut_ptr(),
        w!("%s\\AppKey\\%u").as_ptr(),
        w!("Software\\Microsoft\\Windows\\CurrentVersion\\Explorer").as_ptr(),
        uAppCmd,
    );

    let mut hKey: *mut c_void = core::ptr::null_mut();
    if SHRegOpenUSKeyW(szBuf.as_ptr(), KEY_READ, core::ptr::null_mut(), &mut hKey, 0) != ERROR_SUCCESS {
        return bResult;
    }

    let mut cb: DWORD = core::mem::size_of_val(&szBuf) as DWORD;
    if bResult == 0
        && SHRegQueryUSValueW(
            hKey,
            w!("ShellExecute").as_ptr(),
            core::ptr::null_mut(),
            szBuf.as_mut_ptr() as *mut c_void,
            &mut cb,
            0,
            core::ptr::null_mut(),
            0,
        ) == ERROR_SUCCESS
    {
        bResult = 1;
    }

    cb = core::mem::size_of_val(&szBuf) as DWORD;
    if bResult == 0
        && SHRegQueryUSValueW(
            hKey,
            w!("Association").as_ptr(),
            core::ptr::null_mut(),
            szBuf.as_mut_ptr() as *mut c_void,
            &mut cb,
            0,
            core::ptr::null_mut(),
            0,
        ) == ERROR_SUCCESS
    {
        bResult = 1;
        cb = szBuf.len() as DWORD;
        if AssocQueryStringW(
            ASSOCF_NOTRUNCATE,
            ASSOCSTR_EXECUTABLE,
            szBuf.as_ptr(),
            core::ptr::null(),
            szBuf.as_mut_ptr(),
            &mut cb,
        ) != S_OK
        {
            szBuf[0] = 0;
        }
    }

    cb = core::mem::size_of_val(&szBuf) as DWORD;
    if bResult == 0
        && SHRegQueryUSValueW(
            hKey,
            w!("RegisteredApp").as_ptr(),
            core::ptr::null_mut(),
            szBuf.as_mut_ptr() as *mut c_void,
            &mut cb,
            0,
            core::ptr::null_mut(),
            0,
        ) == ERROR_SUCCESS
    {
        bResult = 1;
        SHRunIndirectRegClientCommand(core::ptr::null_mut(), szBuf.as_ptr());
        szBuf[0] = 0; // Don't execute again
    }
    SHRegCloseUSKey(hKey);

    // Tweak UI uses an empty string for "Do nothing".
    if bResult != 0 && szBuf[0] != 0 {
        ShellExec_RunDLLW(core::ptr::null_mut(), core::ptr::null_mut(), szBuf.as_ptr(), SW_SHOW);
    }
    bResult
}

unsafe fn HandleAppCommand(_this: *mut CTaskSwitchWnd, _wParam: WPARAM, lParam: LPARAM) -> BOOL {
    let uAppCmd = GET_APPCOMMAND_LPARAM(lParam);
    if InvokeRegistryAppKeyCommand(uAppCmd) != 0 {
        return 1;
    }
    match uAppCmd {
        APPCOMMAND_VOLUME_MUTE | APPCOMMAND_VOLUME_DOWN | APPCOMMAND_VOLUME_UP => {
            // TODO: mixer API. FIXME.
            1
        }
        APPCOMMAND_BROWSER_SEARCH => SHFindFiles(core::ptr::null(), core::ptr::null()),
        _ => 0,
    }
}

#[inline]
fn GET_APPCOMMAND_LPARAM(lparam: LPARAM) -> UINT {
    let hi = ((lparam >> 16) & 0xFFFF) as u32;
    ((hi & !FAPPCOMMAND_MASK) as i16 as i32) as UINT
}

unsafe fn OnShellHook(this: *mut CTaskSwitchWnd, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: *mut BOOL) -> LRESULT {
    let mut Ret: BOOL = 0;

    // If the shell hook wasn't registered properly, ignore WM_NULLs.
    if uMsg == 0 {
        *bHandled = 0;
        return 0;
    }

    match wParam as INT {
        HSHELL_APPCOMMAND => {
            Ret = HandleAppCommand(this, 0, lParam);
        }
        HSHELL_WINDOWCREATED => {
            AddTask(this, lParam as HWND);
        }
        HSHELL_WINDOWDESTROYED => {
            // The window still exists — delay-destroy a bit.
            OnWindowDestroyed(this, lParam as HWND);
            DeleteTask(this, lParam as HWND);
        }
        HSHELL_RUDEAPPACTIVATED | HSHELL_WINDOWACTIVATED => {
            OnWindowActivated(this, lParam as HWND);
            ActivateTask(this, lParam as HWND);
        }
        HSHELL_FLASH => {
            FlashTask(this, lParam as HWND);
        }
        HSHELL_REDRAW => {
            RedrawTask(this, lParam as HWND);
        }
        HSHELL_TASKMAN => {
            PostMessageW(tray_get_hwnd((*this).m_Tray), TWM_OPENSTARTMENU, 0, 0);
        }
        HSHELL_ACTIVATESHELLWINDOW => {
            SwitchToThisWindow(tray_get_hwnd((*this).m_Tray), 1);
            SetForegroundWindow(tray_get_hwnd((*this).m_Tray));
        }
        _ => {
            // unhandled
        }
    }

    Ret as LRESULT
}

unsafe fn HandleTaskItemClick(this: *mut CTaskSwitchWnd, TaskItem: *mut TASK_ITEM) {
    if IsWindow((*TaskItem).hWnd) != 0 {
        let bIsMinimized = IsIconic((*TaskItem).hWnd);
        let bIsActive = TaskItem == (*this).m_ActiveTaskItem;

        if bIsMinimized == 0 && bIsActive {
            if IsHungAppWindow((*TaskItem).hWnd) == 0 {
                ShowWindowAsync((*TaskItem).hWnd, SW_MINIMIZE);
            }
        } else {
            SwitchToThisWindow((*TaskItem).hWnd, 1);
        }
    }
}

unsafe fn HandleTaskGroupClick(_TaskGroup: *mut TASK_GROUP) {
    // TODO: Show task group menu
}

unsafe fn HandleButtonClick(this: *mut CTaskSwitchWnd, wIndex: WORD) -> BOOL {
    if (*this).m_IsGroupingEnabled != 0 {
        let TaskGroup = FindTaskGroupByIndex(this, wIndex as INT);
        if !TaskGroup.is_null() && (*TaskGroup).IsCollapsed() {
            HandleTaskGroupClick(TaskGroup);
            return 1;
        }
    }

    let TaskItem = FindTaskItemByIndex(this, wIndex as INT);
    if !TaskItem.is_null() {
        HandleTaskItemClick(this, TaskItem);
        return 1;
    }

    0
}

// SendAsyncProc (arch-split SENDASYNCPROC).
unsafe extern "system" fn SendAsyncProc(hwnd: HWND, _uMsg: UINT, _dwData: ULONG_PTR, _lResult: LRESULT) {
    PostMessageW(hwnd, WM_NULL, 0, 0);
}

unsafe fn HandleTaskItemRightClick(this: *mut CTaskSwitchWnd, TaskItem: *mut TASK_ITEM) {
    let mut pt: POINT = core::mem::zeroed();
    GetCursorPos(&mut pt);

    SetForegroundWindow((*TaskItem).hWnd);

    ActivateTask(this, (*TaskItem).hWnd);

    SendMessageCallbackW(
        (*TaskItem).hWnd,
        WM_POPUPSYSTEMMENU,
        0,
        MAKELPARAM(pt.x, pt.y),
        Some(SendAsyncProc),
        TaskItem as usize as ULONG_PTR,
    );
}

unsafe fn HandleTaskGroupRightClick(_TaskGroup: *mut TASK_GROUP) {
    // TODO
}

unsafe fn HandleButtonRightClick(this: *mut CTaskSwitchWnd, wIndex: WORD) -> BOOL {
    if (*this).m_IsGroupingEnabled != 0 {
        let TaskGroup = FindTaskGroupByIndex(this, wIndex as INT);
        if !TaskGroup.is_null() && (*TaskGroup).IsCollapsed() {
            HandleTaskGroupRightClick(TaskGroup);
            return 1;
        }
    }

    let TaskItem = FindTaskItemByIndex(this, wIndex as INT);
    if !TaskItem.is_null() {
        HandleTaskItemRightClick(this, TaskItem);
        return 1;
    }

    0
}

unsafe fn HandleItemPaint(this: *mut CTaskSwitchWnd, nmtbcd: *mut NMTBCUSTOMDRAW) -> LRESULT {
    let mut Ret: LRESULT = CDRF_DODEFAULT;

    let TaskItem = FindTaskItemByIndex(this, (*nmtbcd).nmcd.dwItemSpec as INT);
    let TaskGroup = FindTaskGroupByIndex(this, (*nmtbcd).nmcd.dwItemSpec as INT);
    if TaskGroup.is_null() && !TaskItem.is_null() {
        if !TaskItem.is_null() && IsWindow((*TaskItem).hWnd) != 0 {
            // Make the entire button flash if necessary.
            if (*nmtbcd).nmcd.uItemState & CDIS_MARKED != 0 {
                Ret = TBCDRF_NOBACKGROUND;
                if (*this).m_Theme.is_null() {
                    SelectObject((*nmtbcd).nmcd.hdc, GetSysColorBrush(COLOR_HIGHLIGHT as INT) as HGDIOBJ);
                    Rectangle(
                        (*nmtbcd).nmcd.hdc,
                        (*nmtbcd).nmcd.rc.left,
                        (*nmtbcd).nmcd.rc.top,
                        (*nmtbcd).nmcd.rc.right,
                        (*nmtbcd).nmcd.rc.bottom,
                    );
                } else {
                    DrawThemeBackground(
                        (*this).m_Theme,
                        (*nmtbcd).nmcd.hdc,
                        TDP_FLASHBUTTON,
                        0,
                        &(*nmtbcd).nmcd.rc,
                        core::ptr::null(),
                    );
                }
                (*nmtbcd).clrText = GetSysColor(COLOR_HIGHLIGHTTEXT as INT);
                return Ret;
            }
        }
    } else if !TaskGroup.is_null() {
        // FIXME: paint task groups
    }
    Ret
}

unsafe fn HandleToolbarNotification(this: *mut CTaskSwitchWnd, nmh: *const NMHDR) -> LRESULT {
    let mut Ret: LRESULT = 0;

    if (*nmh).code == NM_CUSTOMDRAW {
        let nmtbcd = nmh as *mut NMTBCUSTOMDRAW;
        match (*nmtbcd).nmcd.dwDrawStage {
            x if x == CDDS_ITEMPREPAINT => {
                Ret = HandleItemPaint(this, nmtbcd);
            }
            x if x == CDDS_PREPAINT => {
                Ret = CDRF_NOTIFYITEMDRAW;
            }
            _ => {
                Ret = CDRF_DODEFAULT;
            }
        }
    }

    Ret
}

// --- rude-app fullscreen enumeration ---

#[repr(C)]
struct RUDEAPPDATA {
    hTargetMonitor: HMONITOR,
    hwndFound: HWND,
    hwndFirstCheck: HWND,
}

unsafe extern "system" fn IsRudeEnumProc(hwnd: HWND, lParam: LPARAM) -> BOOL {
    IsRudeEnumProc_impl(hwnd, lParam)
}
#[inline]
unsafe fn IsRudeEnumProc_impl(hwnd: HWND, lParam: LPARAM) -> BOOL {
    let pData = lParam as *mut RUDEAPPDATA;

    let hMon = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
    if hMon.is_null()
        || (!(*pData).hTargetMonitor.is_null() && (*pData).hTargetMonitor != hMon)
        || SHELL_IsRudeWindow(hMon, hwnd, (hwnd == (*pData).hwndFirstCheck) as BOOL) == 0
    {
        return 1; // Continue
    }

    (*pData).hwndFound = hwnd;
    0 // Finish
}

#[repr(C)]
struct FULLSCREENDATA {
    pRect: *const RECT,
    hTargetMonitor: HMONITOR,
    pTray: *mut ITrayWindow,
}

unsafe extern "system" fn FullScreenEnumProc(hMonitor: HMONITOR, hDC: HDC, prc: *mut RECT, lParam: LPARAM) -> BOOL {
    FullScreenEnumProc_impl(hMonitor, hDC, prc, lParam)
}
#[inline]
unsafe fn FullScreenEnumProc_impl(hMonitor: HMONITOR, _hDC: HDC, _prc: *mut RECT, lParam: LPARAM) -> BOOL {
    let pData = lParam as *mut FULLSCREENDATA;

    let mut bFullOpening = (*pData).hTargetMonitor == hMonitor;
    if !bFullOpening && !(*pData).pRect.is_null() {
        let mut rc: RECT = core::mem::zeroed();
        let mut rcMon: RECT = core::mem::zeroed();
        SHELL_GetMonitorRect(hMonitor, &mut rcMon, 0);
        IntersectRect(&mut rc, &rcMon, (*pData).pRect);
        bFullOpening = EqualRect(&rc, &rcMon) != 0;
    }

    tray_notify_fullscreen((*pData).pTray, hMonitor, bFullOpening as BOOL);
    1
}

unsafe fn HandleFullScreenApp(this: *mut CTaskSwitchWnd, hwndRude: HWND) {
    let mut rc: RECT = core::mem::zeroed();
    let mut Data = FULLSCREENDATA {
        pRect: core::ptr::null(),
        hTargetMonitor: core::ptr::null_mut(),
        pTray: core::ptr::null_mut(),
    };
    if !hwndRude.is_null() && GetWindowRect(hwndRude, &mut rc) != 0 {
        Data.pRect = &rc;
        Data.hTargetMonitor = MonitorFromWindow(hwndRude, MONITOR_DEFAULTTONULL);
    }
    Data.pTray = (*this).m_Tray;
    EnumDisplayMonitors(
        core::ptr::null_mut(),
        core::ptr::null(),
        Some(FullScreenEnumProc),
        &mut Data as *mut FULLSCREENDATA as LPARAM,
    );

    if !hwndRude.is_null() {
        // Make the taskbar bottom.
        let uFlags = SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_NOOWNERZORDER;
        let hwndTray = tray_get_hwnd((*this).m_Tray);
        SetWindowPos(hwndTray, HWND_BOTTOM, 0, 0, 0, 0, uFlags);

        // Switch to the rude app if necessary.
        let exstyle = get_window_long(hwndRude, GWL_EXSTYLE) as DWORD;
        if exstyle & WS_EX_TOPMOST == 0 && SHELL_IsRudeWindowActive(hwndRude) == 0 {
            SwitchToThisWindow(hwndRude, 1);
        }
    }
}

unsafe fn FindRudeApp(hwndFirstCheck: HWND) -> HWND {
    let hMon = MonitorFromWindow(hwndFirstCheck, MONITOR_DEFAULTTONEAREST);
    let mut data = RUDEAPPDATA {
        hTargetMonitor: hMon,
        hwndFound: core::ptr::null_mut(),
        hwndFirstCheck,
    };
    if IsWindow(hwndFirstCheck) != 0
        && IsRudeEnumProc_impl(hwndFirstCheck, &mut data as *mut RUDEAPPDATA as LPARAM) == 0
    {
        return hwndFirstCheck;
    }

    EnumWindows(Some(IsRudeEnumProc), &mut data as *mut RUDEAPPDATA as LPARAM);

    data.hwndFound
}

unsafe fn OnWindowPosChanged(this: *mut CTaskSwitchWnd, _uMsg: UINT, _wParam: WPARAM, _lParam: LPARAM, bHandled: *mut BOOL) -> LRESULT {
    KillTimer((*this).m_hWnd, TIMER_ID_VALIDATE_RUDE_APP);
    SetTimer((*this).m_hWnd, TIMER_ID_VALIDATE_RUDE_APP, VALIDATE_RUDE_INTERVAL, None);
    (*this).m_nRudeAppValidationCounter = 0;
    *bHandled = 0;
    0
}

unsafe fn OnWindowActivated(this: *mut CTaskSwitchWnd, _hwndTarget: HWND) {
    KillTimer((*this).m_hWnd, TIMER_ID_VALIDATE_RUDE_APP);
    SetTimer((*this).m_hWnd, TIMER_ID_VALIDATE_RUDE_APP, VALIDATE_RUDE_INTERVAL, None);
    (*this).m_nRudeAppValidationCounter = 0;
}

unsafe fn OnWindowDestroyed(this: *mut CTaskSwitchWnd, hwndTarget: HWND) {
    if FindTaskItem(this, hwndTarget).is_null() {
        return;
    }
    let hwndRude = FindRudeApp(hwndTarget);
    HandleFullScreenApp(this, hwndRude);
}

unsafe fn OnEraseBackground(this: *mut CTaskSwitchWnd, _uMsg: UINT, wParam: WPARAM, _lParam: LPARAM, bHandled: *mut BOOL) -> LRESULT {
    let hdc = wParam as HDC;

    if IsAppThemed() == 0 {
        *bHandled = 0;
        return 0;
    }

    let mut rect: RECT = core::mem::zeroed();
    GetClientRect((*this).m_hWnd, &mut rect);
    DrawThemeParentBackground((*this).m_hWnd, hdc, &rect);

    1
}

unsafe fn OnSize(this: *mut CTaskSwitchWnd, _uMsg: UINT, _wParam: WPARAM, lParam: LPARAM, _bHandled: *mut BOOL) -> LRESULT {
    let szClient = SIZE {
        cx: (lParam & 0xFFFF) as u16 as i32,
        cy: ((lParam >> 16) & 0xFFFF) as u16 as i32,
    };
    if !(*this).m_TaskBar.m_hWnd.is_null() {
        SetWindowPos((*this).m_TaskBar.m_hWnd, core::ptr::null_mut(), 0, 0, szClient.cx, szClient.cy, SWP_NOZORDER);
        UpdateButtonsSize(this, 0);
    }
    1
}

unsafe fn OnNcHitTest(this: *mut CTaskSwitchWnd, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, _bHandled: *mut BOOL) -> LRESULT {
    // Make the control appear transparent so the tray window is draggable.
    let mut Ret = DefWindowProcW((*this).m_hWnd, uMsg, wParam, lParam);
    if Ret != HTVSCROLL && Ret != HTHSCROLL {
        Ret = HTTRANSPARENT;
    }
    Ret
}

unsafe fn OnCommand(this: *mut CTaskSwitchWnd, _uMsg: UINT, wParam: WPARAM, lParam: LPARAM, _bHandled: *mut BOOL) -> LRESULT {
    if lParam != 0 && lParam as HWND == (*this).m_TaskBar.m_hWnd {
        HandleButtonClick(this, LOWORD(wParam as usize));
    }
    1
}

unsafe fn OnNotify(this: *mut CTaskSwitchWnd, _uMsg: UINT, _wParam: WPARAM, lParam: LPARAM, _bHandled: *mut BOOL) -> LRESULT {
    let mut Ret: LRESULT = 1;
    let nmh = lParam as *const NMHDR;
    if (*nmh).hwndFrom == (*this).m_TaskBar.m_hWnd {
        Ret = HandleToolbarNotification(this, nmh);
    }
    Ret
}

unsafe fn OnUpdateTaskbarPos(this: *mut CTaskSwitchWnd, _uMsg: UINT, _wParam: WPARAM, _lParam: LPARAM, _bHandled: *mut BOOL) -> LRESULT {
    tt_update_button_spacing((*this).task_bar(), tray_is_horizontal((*this).m_Tray), (!(*this).m_Theme.is_null()) as BOOL, 0, 0);
    1
}

unsafe fn OnTaskbarSettingsChanged(this: *mut CTaskSwitchWnd, _uMsg: UINT, _wParam: WPARAM, lParam: LPARAM, _bHandled: *mut BOOL) -> LRESULT {
    let mut bSettingsChanged: BOOL = 0;
    let newSettings = lParam as *const TaskbarSettings;

    if (*newSettings).bGroupButtons != g_TaskbarSettings.bGroupButtons {
        bSettingsChanged = 1;
        g_TaskbarSettings.bGroupButtons = (*newSettings).bGroupButtons;
        (*this).m_IsGroupingEnabled = g_TaskbarSettings.bGroupButtons;
    }

    if (*newSettings).bSmallIcons != g_TaskbarSettings.bSmallIcons {
        bSettingsChanged = 1;
        g_TaskbarSettings.bSmallIcons = (*newSettings).bSmallIcons;
    }

    if bSettingsChanged != 0 {
        RefreshWindowList(this);
        UpdateButtonsSize(this, 0);
    }

    0
}

unsafe fn OnContextMenu(this: *mut CTaskSwitchWnd, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, _bHandled: *mut BOOL) -> LRESULT {
    let mut Ret: LRESULT = 0;
    let mut iBtn: isize = -1;

    if !(*this).m_TaskBar.m_hWnd.is_null() {
        let mut pt = POINT {
            x: GET_X_LPARAM(lParam),
            y: GET_Y_LPARAM(lParam),
        };
        ScreenToClient((*this).m_TaskBar.m_hWnd, &mut pt);

        iBtn = tb_hit_test((*this).m_TaskBar.m_hWnd, &mut pt) as isize;
        if iBtn >= 0 {
            HandleButtonRightClick(this, iBtn as WORD);
        }
    }
    if iBtn < 0 {
        Ret = SendMessageW(tray_get_hwnd((*this).m_Tray), uMsg, wParam, lParam);
    }
    Ret
}

unsafe fn OnKludgeItemRect(this: *mut CTaskSwitchWnd, _uMsg: UINT, wParam: WPARAM, lParam: LPARAM, _bHandled: *mut BOOL) -> LRESULT {
    let TaskItem = FindTaskItem(this, wParam as HWND);
    if !TaskItem.is_null() {
        let prcMinRect = lParam as *mut RECT;
        let mut rcItem: RECT = core::mem::zeroed();
        let mut rcToolbar: RECT = core::mem::zeroed();
        tb_get_item_rect((*this).m_TaskBar.m_hWnd, (*TaskItem).Index, &mut rcItem);
        GetWindowRect((*this).m_TaskBar.m_hWnd, &mut rcToolbar);

        OffsetRect(&mut rcItem, rcToolbar.left, rcToolbar.top);

        *prcMinRect = rcItem;
        return 1;
    }
    0
}

unsafe fn OnMouseActivate(_this: *mut CTaskSwitchWnd, _uMsg: UINT, _wParam: WPARAM, _lParam: LPARAM, _bHandled: *mut BOOL) -> LRESULT {
    MA_NOACTIVATE
}

unsafe fn OnTimer(this: *mut CTaskSwitchWnd, _uMsg: UINT, wParam: WPARAM, _lParam: LPARAM, _bHandled: *mut BOOL) -> LRESULT {
    if wParam as UINT_PTR == TIMER_ID_VALIDATE_RUDE_APP {
        // Real rude-app activation can lag HSHELL_...ACTIVATED; poll ≤5s.
        let hwndRude = FindRudeApp(core::ptr::null_mut());
        HandleFullScreenApp(this, hwndRude);

        KillTimer((*this).m_hWnd, wParam as UINT_PTR);
        (*this).m_nRudeAppValidationCounter += 1;
        if (*this).m_nRudeAppValidationCounter < VALIDATE_RUDE_MAX_COUNT && hwndRude.is_null() {
            SetTimer((*this).m_hWnd, wParam as UINT_PTR, VALIDATE_RUDE_INTERVAL, None);
        }
    }
    1
}

unsafe fn OnSetFont(this: *mut CTaskSwitchWnd, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, _bHandled: *mut BOOL) -> LRESULT {
    SendMessageW((*this).m_TaskBar.m_hWnd, uMsg, wParam, lParam)
}

unsafe fn OnSettingChanged(this: *mut CTaskSwitchWnd, _uMsg: UINT, wParam: WPARAM, _lParam: LPARAM, _bHandled: *mut BOOL) -> LRESULT {
    if wParam as UINT == SPI_SETNONCLIENTMETRICS {
        // Don't update the font — that arrives via WM_SETFONT from the parent.
        UpdateButtonsSize(this, 0);
    }
    0
}

unsafe fn OnCopyData(this: *mut CTaskSwitchWnd, _uMsg: UINT, _wParam: WPARAM, lParam: LPARAM, _bHandled: *mut BOOL) -> LRESULT {
    let cpData = lParam as *const COPYDATASTRUCT;
    if (*cpData).dwData == (*this).m_uHardErrorMsg as usize {
        let pData = (*cpData).lpData as *mut BALLOON_HARD_ERROR_DATA;
        if (*pData).cbHeaderSize == core::mem::size_of::<BALLOON_HARD_ERROR_DATA>() as DWORD {
            CHardErrorThread_StartThread(&mut (*this).m_HardErrorThread, pData);
        }
        return 1;
    }
    0
}

// ===========================================================================
// COM: IUnknown / IOleWindow (BEGIN_COM_MAP + CComObject)
// ===========================================================================

#[no_mangle]
pub unsafe extern "system" fn CTaskSwitchWnd_QueryInterface(
    iface: *mut IOleWindow,
    riid: REFIID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    let this = iface as *mut CTaskSwitchWnd;
    if ppv.is_null() {
        return E_POINTER;
    }
    if guid_eq(riid, &IID_IUnknown) || guid_eq(riid, &IID_IOleWindow) {
        *ppv = this as *mut c_void;
        CTaskSwitchWnd_AddRef(iface);
        return S_OK;
    }
    *ppv = core::ptr::null_mut();
    E_NOINTERFACE
}

#[no_mangle]
pub unsafe extern "system" fn CTaskSwitchWnd_AddRef(iface: *mut IOleWindow) -> u32 {
    let this = iface as *mut CTaskSwitchWnd;
    interlocked_increment(&mut (*this).ref_) as u32
}

#[no_mangle]
pub unsafe extern "system" fn CTaskSwitchWnd_Release(iface: *mut IOleWindow) -> u32 {
    let this = iface as *mut CTaskSwitchWnd;
    let n = interlocked_decrement(&mut (*this).ref_);
    if n == 0 {
        CTaskSwitchWnd_delete(this);
        return 0;
    }
    n as u32
}

/// IOleWindow::GetWindow (override).
#[no_mangle]
pub unsafe extern "system" fn CTaskSwitchWnd_GetWindow(iface: *mut IOleWindow, phwnd: *mut HWND) -> HRESULT {
    let this = iface as *mut CTaskSwitchWnd;
    if phwnd.is_null() {
        return E_INVALIDARG;
    }
    *phwnd = (*this).m_hWnd;
    S_OK
}

/// IOleWindow::ContextSensitiveHelp (override).
#[no_mangle]
pub unsafe extern "system" fn CTaskSwitchWnd_ContextSensitiveHelp(_iface: *mut IOleWindow, _f: BOOL) -> HRESULT {
    E_NOTIMPL
}

static vt_CTaskSwitchWnd: IOleWindowVtbl = IOleWindowVtbl {
    QueryInterface: CTaskSwitchWnd_QueryInterface,
    AddRef: CTaskSwitchWnd_AddRef,
    Release: CTaskSwitchWnd_Release,
    GetWindow: CTaskSwitchWnd_GetWindow,
    ContextSensitiveHelp: CTaskSwitchWnd_ContextSensitiveHelp,
};

// ===========================================================================
// Construction / destruction (CComObject new/delete of a Rust class)
// ===========================================================================

unsafe fn CTaskSwitchWnd_ctor(this: *mut CTaskSwitchWnd) {
    (*this).lpVtbl = &vt_CTaskSwitchWnd;
    (*this).ref_ = 0;
    (*this).m_hWnd = core::ptr::null_mut();

    CTaskToolbar_ctor(&mut (*this).m_TaskBar);
    (*this).m_Tray = core::ptr::null_mut();
    (*this).m_ShellHookMsg = 0;
    (*this).m_TaskItemCount = 0;
    (*this).m_AllocatedTaskItems = 0;
    (*this).m_TaskGroups = core::ptr::null_mut();
    (*this).m_TaskItems = core::ptr::null_mut();
    (*this).m_ActiveTaskItem = core::ptr::null_mut();
    (*this).m_Theme = core::ptr::null_mut();
    (*this).m_ButtonsPerLine = 0;
    (*this).m_ButtonCount = 0;
    (*this).m_ImageList = core::ptr::null_mut();
    (*this).m_IsGroupingEnabled = 0;
    (*this).m_IsDestroying = 0;
    (*this).m_nRudeAppValidationCounter = 0;
    (*this).m_ButtonSize = SIZE { cx: 0, cy: 0 };
    CHardErrorThread_ctor(&mut (*this).m_HardErrorThread);
    (*this).m_uHardErrorMsg = RegisterWindowMessageW(w!("HardError").as_ptr());
}

unsafe fn CTaskSwitchWnd_new() -> *mut CTaskSwitchWnd {
    let p = HeapAlloc(hProcessHeap, 0, core::mem::size_of::<CTaskSwitchWnd>() as SIZE_T) as *mut CTaskSwitchWnd;
    if !p.is_null() {
        CTaskSwitchWnd_ctor(p);
    }
    p
}

unsafe fn CTaskSwitchWnd_delete(this: *mut CTaskSwitchWnd) {
    if this.is_null() {
        return;
    }
    // ~CTaskSwitchWnd: members with non-trivial dtors run in reverse order.
    CHardErrorThread_dtor(&mut (*this).m_HardErrorThread);
    if !(*this).m_Tray.is_null() {
        (*((*this).m_Tray as *mut IUnknown)).Release();
        (*this).m_Tray = core::ptr::null_mut();
    }
    // NOTE: m_TaskItems/m_TaskGroups arrays are intentionally NOT freed here —
    // faithful to the C++ (empty dtor; FreeTaskItem never frees the buffer).
    HeapFree(hProcessHeap, 0, this as *mut c_void);
}

// ===========================================================================
// The WndProc (BEGIN_MSG_MAP) — arch-split class callback
// ===========================================================================

unsafe fn CTaskSwitchWnd_process(
    this: *mut CTaskSwitchWnd,
    hWnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let mut bHandled: BOOL = 1;
    let mut lResult: LRESULT = 0;

    match uMsg {
        WM_THEMECHANGED => lResult = OnThemeChanged(this, uMsg, wParam, lParam, &mut bHandled),
        WM_ERASEBKGND => lResult = OnEraseBackground(this, uMsg, wParam, lParam, &mut bHandled),
        WM_SIZE => lResult = OnSize(this, uMsg, wParam, lParam, &mut bHandled),
        WM_CREATE => lResult = OnCreate(this, uMsg, wParam, lParam, &mut bHandled),
        WM_DESTROY => lResult = OnDestroy(this, uMsg, wParam, lParam, &mut bHandled),
        WM_NCHITTEST => lResult = OnNcHitTest(this, uMsg, wParam, lParam, &mut bHandled),
        WM_COMMAND => lResult = OnCommand(this, uMsg, wParam, lParam, &mut bHandled),
        WM_NOTIFY => lResult = OnNotify(this, uMsg, wParam, lParam, &mut bHandled),
        x if x == TSWM_UPDATETASKBARPOS => lResult = OnUpdateTaskbarPos(this, uMsg, wParam, lParam, &mut bHandled),
        x if x == TWM_SETTINGSCHANGED => lResult = OnTaskbarSettingsChanged(this, uMsg, wParam, lParam, &mut bHandled),
        WM_CONTEXTMENU => lResult = OnContextMenu(this, uMsg, wParam, lParam, &mut bHandled),
        WM_TIMER => lResult = OnTimer(this, uMsg, wParam, lParam, &mut bHandled),
        WM_SETFONT => lResult = OnSetFont(this, uMsg, wParam, lParam, &mut bHandled),
        WM_SETTINGCHANGE => lResult = OnSettingChanged(this, uMsg, wParam, lParam, &mut bHandled),
        WM_MOUSEACTIVATE => lResult = OnMouseActivate(this, uMsg, wParam, lParam, &mut bHandled),
        WM_KLUDGEMINRECT => lResult = OnKludgeItemRect(this, uMsg, wParam, lParam, &mut bHandled),
        WM_COPYDATA => lResult = OnCopyData(this, uMsg, wParam, lParam, &mut bHandled),
        WM_WINDOWPOSCHANGED => lResult = OnWindowPosChanged(this, uMsg, wParam, lParam, &mut bHandled),
        // MESSAGE_HANDLER(m_ShellHookMsg, OnShellHook) — dynamic id.
        x if (*this).m_ShellHookMsg != 0 && x == (*this).m_ShellHookMsg => {
            lResult = OnShellHook(this, uMsg, wParam, lParam, &mut bHandled)
        }
        _ => bHandled = 0,
    }

    if bHandled != 0 {
        lResult
    } else {
        DefWindowProcW(hWnd, uMsg, wParam, lParam)
    }
}

#[no_mangle]
pub unsafe extern "system" fn CTaskSwitchWnd_WindowProc(hWnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    CTaskSwitchWnd_window_proc_impl(hWnd, uMsg, wParam, lParam)
}

#[inline]
unsafe fn CTaskSwitchWnd_window_proc_impl(hWnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    let this = if uMsg == WM_NCCREATE {
        let p = wndproc_nccreate_this(hWnd, lParam) as *mut CTaskSwitchWnd;
        if !p.is_null() {
            (*p).m_hWnd = hWnd;
        }
        p
    } else {
        get_window_this(hWnd) as *mut CTaskSwitchWnd
    };

    if this.is_null() {
        return DefWindowProcW(hWnd, uMsg, wParam, lParam);
    }

    let r = CTaskSwitchWnd_process(this, hWnd, uMsg, wParam, lParam);

    if uMsg == WM_NCDESTROY {
        (*this).m_hWnd = core::ptr::null_mut();
    }
    r
}

// ===========================================================================
// CWindowImpl::Create + Initialize + factory
// ===========================================================================

unsafe fn CTaskSwitchWnd_Create(this: *mut CTaskSwitchWnd, hWndParent: HWND, szWindowName: LPCWSTR, dwStyle: DWORD) -> HWND {
    // DECLARE_WND_CLASS_EX(szTaskSwitchWndClass, CS_DBLCLKS, COLOR_3DFACE)
    register_class_once(
        hExplorerInstance,
        w!("MSTaskSwWClass").as_ptr(),
        Some(CTaskSwitchWnd_WindowProc),
        CS_DBLCLKS,
        COLOR_3DFACE as i32,
    );

    (*this).m_hWnd = CreateWindowExW(
        0,
        w!("MSTaskSwWClass").as_ptr(),
        szWindowName,
        dwStyle,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        hWndParent,
        core::ptr::null_mut(),
        hExplorerInstance,
        this as LPVOID,
    );
    (*this).m_hWnd
}

unsafe fn CTaskSwitchWnd_Initialize(this: *mut CTaskSwitchWnd, hWndParent: HWND, tray: *mut ITrayWindow) -> HRESULT {
    // m_Tray = tray (CComPtr AddRef).
    if !tray.is_null() {
        (*(tray as *mut IUnknown)).AddRef();
    }
    (*this).m_Tray = tray;

    (*this).m_IsGroupingEnabled = g_TaskbarSettings.bGroupButtons;
    CTaskSwitchWnd_Create(
        this,
        hWndParent,
        w!("Running Applications").as_ptr(),
        WS_CHILD | WS_VISIBLE | WS_CLIPSIBLINGS | WS_CLIPCHILDREN | WS_TABSTOP,
    );
    if (*this).m_hWnd.is_null() {
        return E_FAIL;
    }
    S_OK
}

#[no_mangle]
pub unsafe extern "C" fn CTaskSwitchWnd_CreateInstance(
    hWndParent: HWND,
    Tray: *mut ITrayWindow,
    riid: REFIID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    // ShellObjectCreatorInit<CTaskSwitchWnd>(hWndParent, Tray, riid, ppv):
    // new(ref=0) → AddRef → Initialize(hWndParent, Tray) → QI → Release.
    let pobj = CTaskSwitchWnd_new();
    if pobj.is_null() {
        return E_OUTOFMEMORY;
    }
    let iface = pobj as *mut IOleWindow;

    CTaskSwitchWnd_AddRef(iface);

    let mut hr = CTaskSwitchWnd_Initialize(pobj, hWndParent, Tray);
    if hr >= 0 {
        hr = CTaskSwitchWnd_QueryInterface(iface, riid, ppv);
    }

    CTaskSwitchWnd_Release(iface);
    hr
}
