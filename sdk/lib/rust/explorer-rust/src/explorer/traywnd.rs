//! traywnd.cpp + appbar.cpp — `CTrayWindow` (the taskbar window, class
//! "Shell_TrayWnd") together with `CAppBarManager` (its base class), the
//! embedded-by-value `CStartButton` (a subclassed WC_BUTTON), the
//! `CTrayWindowCtxMenu` helper COM object, and the free functions
//! (`CreateTrayWindow`, `Tray_OnStartMenuDismissed`, `TrayProcessMessages`,
//! `TrayMessageLoop`). The taskbar CORE.
//!
//! All three C++ pieces (traywnd.cpp CTrayWindow+CStartButton, appbar.cpp
//! CAppBarManager) become Rust AT ONCE, so the object LAYOUT IS FREE — nothing
//! external sees the C++ layout (CTrayWindow is reached only via its interface
//! pointers or its own WndProc). One flat `#[repr(C)]` CTrayWindow: the
//! QI/cast-reachable interface vtbl pointers first (ITrayWindow — the primary,
//! reached via `(ITrayWindow*)this` casts; IShellDesktopTray; IOleWindow;
//! IContextMenu), then ref_, m_hWnd, then ALL of CAppBarManager's fields +
//! CTrayWindow's fields + the embedded CStartButton (a nested flat sub-object
//! with its own m_hWnd + subclass proc, like syspager's CNotifyToolbar).
//!
//! Multi-vtable object model mirrors taskband.rs. The embedded subclassed
//! CStartButton mirrors syspager.rs's CNotifyToolbar. no_std.

use core::ffi::c_void;
use core::mem::{size_of, transmute};
use core::ptr::{addr_of_mut, null, null_mut};

use crate::explorer::settings::g_TaskbarSettings;
use crate::explorer::util::LoadPopupMenu;
use explorer_sys::com::*;
use explorer_sys::externs::{
    HDPA, DPA_Create, DPA_Destroy, DPA_GetPtr, DPA_DeletePtr, DPA_InsertPtr, HRESULT_FROM_WIN32,
    IUnknown_Exec, IUnknown_GetWindow, SHRestricted,
};

// DPA_GetPtrCount(hdpa) is a commctrl.h MACRO — `(*(int*)hdpa)`, the item count
// is the first int of the HDPA — NOT a comctl32 export. Inline it.
#[inline]
unsafe fn DPA_GetPtrCount(hdpa: HDPA) -> i32 {
    *(hdpa as *const i32)
}
use explorer_sys::globals::*;
use explorer_sys::server::{
    get_window_this, interlocked_decrement, interlocked_increment, register_class_once,
    set_window_this, wndproc_nccreate_this, CloseThemeData, GET_X_LPARAM, GET_Y_LPARAM, HIWORD,
    LOWORD, OpenThemeData, SetWindowTheme, ShellExecuteW, HTHEME,
};
use explorer_sys::types::{TaskbarSettings, TW_STRUCKRECTS2};
// ClearRecentAndMru is now Rust (startmnucust.rs, #[no_mangle] extern "C").
use crate::explorer::startmnucust::ClearRecentAndMru;
use winsrv_sys::w;
use winsrv_sys::winsrv::*;
// POINTL is in BOTH explorer_sys::com and winsrv bindgen — disambiguate.
use winsrv_sys::winsrv::POINTL;

// ============================================================================
// file-scope constants
// ============================================================================

// precomp.h / undocshell.h tray + appbar window messages (WM_USER == 0x400).
const WM_APP_TRAYDESTROY: UINT = WM_APP + 0x100;
const TWM_GETTASKSWITCH: UINT = WM_USER + 236;
const TWM_OPENSTARTMENU: UINT = WM_USER + 260;
const TWM_SETTINGSCHANGED: UINT = WM_USER + 300;
const TWM_SETZORDER: UINT = WM_USER + 338;
const TWM_DOEXITWINDOWS: UINT = WM_USER + 342;
const TNWM_GETMINIMUMSIZE: UINT = WM_USER + 0x100;
const TNWM_GETSHOWDESKTOPBUTTON: UINT = WM_USER + 0x7601;
const NTNWM_REALIGN: UINT = 0x1;
const TSWM_UPDATETASKBARPOS: UINT = WM_USER + 2;
const WM_PROGMAN_SAVESTATE: UINT = WM_USER + 77;

// traywnd.cpp file-scope #defines
const TIMER_ID_AUTOHIDE: UINT_PTR = 1;
const TIMER_ID_MOUSETRACK: UINT_PTR = 2;
const MOUSETRACK_INTERVAL: UINT = 100;
const AUTOHIDE_DELAY_HIDE: UINT = 2000;
const AUTOHIDE_DELAY_SHOW: UINT = 50;
const AUTOHIDE_INTERVAL_ANIMATING: UINT = 10;
const AUTOHIDE_SPEED_SHOW: LONG = 10;
const AUTOHIDE_SPEED_HIDE: LONG = 1;
const AUTOHIDE_HIDDEN: UINT = 0;
const AUTOHIDE_SHOWING: UINT = 1;
const AUTOHIDE_SHOWN: UINT = 2;
const AUTOHIDE_HIDING: UINT = 3;

const IDHK_RUN: UINT = 0x1f4;
const IDHK_MINIMIZE_ALL: UINT = 0x1f5;
const IDHK_RESTORE_ALL: UINT = 0x1f6;
const IDHK_HELP: UINT = 0x1f7;
const IDHK_EXPLORE: UINT = 0x1f8;
const IDHK_FIND: UINT = 0x1f9;
const IDHK_FIND_COMPUTER: UINT = 0x1fa;
const IDHK_NEXT_TASK: UINT = 0x1fb;
const IDHK_PREV_TASK: UINT = 0x1fc;
const IDHK_SYS_PROPERTIES: UINT = 0x1fd;
const IDHK_DESKTOP: UINT = 0x1fe;
const IDHK_PAGER: UINT = 0x1ff;

// resource.h ids
const IDB_START: u16 = 143;
const IDB_STARTMENU: u16 = 158;
const IDS_START: u16 = 595;
const IDS_HELP_COMMAND: u16 = 732;
const IDS_RESTORE_ALL: u16 = 811;
const IDS_TRAYWND_UNDO_CASCADE: u16 = 812;
const IDS_TRAYWND_UNDO_TILE: u16 = 813;
const IDC_STARTBTN: usize = 1140;
const IDM_TRAYWND: u16 = 205;
const IDM_SEARCH: UINT = 520;

// resource.h shell command ids (context menu)
const ID_SHELL_CMD_OPEN_TASKMGR: UINT = 405;
const ID_SHELL_CMD_UNDO_ACTION: UINT = 406;
const ID_SHELL_CMD_SHOW_DESKTOP: UINT = 407;
const ID_SHELL_CMD_TILE_WND_V: UINT = 408;
const ID_SHELL_CMD_TILE_WND_H: UINT = 409;
const ID_SHELL_CMD_CASCADE_WND: UINT = 410;
const ID_SHELL_CMD_CUST_NOTIF: UINT = 411;
const ID_SHELL_CMD_ADJUST_DAT: UINT = 412;
const ID_SHELL_CMD_RESTORE_ALL: UINT = 413;
const ID_LOCKTASKBAR: UINT = 404;

// undocshell.h TRAYCMD_*
const TRAYCMD_STARTMENU: UINT = 305;
const TRAYCMD_RUN_DIALOG: UINT = 401;
const TRAYCMD_LOGOFF_DIALOG: UINT = 402;
const TRAYCMD_CASCADE: UINT = 403;
const TRAYCMD_TILE_H: UINT = 404;
const TRAYCMD_TILE_V: UINT = 405;
const TRAYCMD_TOGGLE_DESKTOP: UINT = 407;
const TRAYCMD_DATE_AND_TIME: UINT = 408;
const TRAYCMD_TASKBAR_PROPERTIES: UINT = 413;
const TRAYCMD_MINIMIZE_ALL: UINT = 415;
const TRAYCMD_RESTORE_ALL: UINT = 416;
const TRAYCMD_SHOW_DESKTOP: UINT = 419;
const TRAYCMD_SHOW_TASK_MGR: UINT = 420;
const TRAYCMD_CUSTOMIZE_TASKBAR: UINT = 421;
const TRAYCMD_LOCK_TASKBAR: UINT = 424;
const TRAYCMD_HELP_AND_SUPPORT: UINT = 503;
const TRAYCMD_CONTROL_PANEL: UINT = 505;
const TRAYCMD_SHUTDOWN_DIALOG: UINT = 506;
const TRAYCMD_PRINTERS_AND_FAXES: UINT = 510;
const TRAYCMD_LOCK_DESKTOP: UINT = 517;
const TRAYCMD_SWITCH_USER_DIALOG: UINT = 5000;
const TRAYCMD_SEARCH_FILES: UINT = 41093;
const TRAYCMD_SEARCH_COMPUTERS: UINT = 41094;

// shlobj.h RESTRICTIONS enum (implicit-increment values computed from the enum)
const REST_NOSAVESET: u32 = 0x0000_0004;
const REST_NOTRAYCONTEXTMENU: u32 = 0x0200_0000;
const REST_CLEARRECENTDOCSONEXIT: u32 = 0x4000_0007;
const REST_CLASSICSHELL: u32 = 0x4000_0008;
const REST_NOCLOSE: u32 = 0x0000_0002;
const REST_NOWINKEYS: u32 = 0x4000_0044;

// commctrl.h BUTTON / rebar / toolbar / notify
const BCM_FIRST: UINT = 0x1600;
const BCM_GETIDEALSIZE: UINT = BCM_FIRST + 0x1;
const BCM_SETIMAGELIST: UINT = BCM_FIRST + 0x2;
const BUTTON_IMAGELIST_ALIGN_LEFT: UINT = 0;
const RB_SIZETORECT: UINT = WM_USER + 23;
const RBSTR_CHANGERECT: WPARAM = 0x1;
const CCS_VERT: DWORD = 0x80;
const RBN_FIRST: UINT = 0u32.wrapping_sub(831);
const RBN_AUTOSIZE: UINT = RBN_FIRST.wrapping_sub(3);

// shell menu-merge flags (shlobj.h)
const MM_ADDSEPARATOR: UINT = 0x0000_0001;
const MM_SUBMENUSHAVEIDS: UINT = 0x0000_0002;

// shobjidl.h IMenuPopup::Popup flags + CMF_NORMAL
const MPPF_TOP: DWORD = 0x2000_0000;
const MPPF_LEFT: DWORD = 0x4000_0000;
const MPPF_RIGHT: DWORD = 0x6000_0000;
const MPPF_BOTTOM: DWORD = 0x8000_0000;
const CMF_NORMAL: UINT = 0x0000_0000;

// uxtheme THEMESIZE + vssym32.h taskbar theme parts
const TS_TRUE: i32 = 1;
const TBP_BACKGROUNDBOTTOM: i32 = 1;
const TBP_BACKGROUNDRIGHT: i32 = 2;
const TBP_BACKGROUNDTOP: i32 = 3;
const TBP_BACKGROUNDLEFT: i32 = 4;
const TBP_SIZINGBARBOTTOM: i32 = 5;
const TBP_SIZINGBARRIGHT: i32 = 6;
const TBP_SIZINGBARTOP: i32 = 7;
const TBP_SIZINGBARLEFT: i32 = 8;

// undocshell.h RunFileDlg flag
const RFF_CALCDIRECTORY: UINT = 0x04;

// undocshell.h WM_COPYDATA appbar/notify discriminators
const TABDMC_APPBAR: usize = 0;
const TABDMC_NOTIFY: usize = 1;
const TABDMC_LOADINPROC: usize = 2;

// shellapi.h ABS_* (state)
const ABS_AUTOHIDE_C: UINT = 1;
const ABS_ALWAYSONTOP_C: UINT = 2;

// special HWND sentinels (winuser.h)
const HWND_TOPMOST: HWND = (-1isize) as usize as HWND;
const HWND_NOTOPMOST: HWND = (-2isize) as usize as HWND;
// MAKEINTATOM(32770) — the dialog class atom (GetClassLongPtrW(GCW_ATOM)).
const WC_DIALOG_ATOM: usize = 0x8002;

// window class name literals
static WC_BUTTON_W: [u16; 7] = [
    b'B' as u16, b'u' as u16, b't' as u16, b't' as u16, b'o' as u16, b'n' as u16, 0,
];
static WC_STATIC_W: [u16; 7] = [
    b'S' as u16, b't' as u16, b'a' as u16, b't' as u16, b'i' as u16, b'c' as u16, 0,
];
static szTrayWindowClass: [u16; 14] = [
    b'S' as u16, b'h' as u16, b'e' as u16, b'l' as u16, b'l' as u16, b'_' as u16, b'T' as u16,
    b'r' as u16, b'a' as u16, b'y' as u16, b'W' as u16, b'n' as u16, b'd' as u16, 0,
];

// ============================================================================
// extern surface not in winsrv-sys / explorer-sys yet (cited in the MANIFEST)
// ============================================================================

// user32 (stdcall)
extern "system" {
    fn RedrawWindow(hWnd: HWND, lprcUpdate: *const RECT, hrgnUpdate: HRGN, flags: UINT) -> BOOL;
    fn ChildWindowFromPoint(hWndParent: HWND, pt: POINT) -> HWND;
    fn ChildWindowFromPointEx(hWndParent: HWND, pt: POINT, flags: UINT) -> HWND;
    fn TileWindows(
        hwndParent: HWND,
        wHow: UINT,
        lpRect: *const RECT,
        cKids: UINT,
        lpKids: *const HWND,
    ) -> u16;
    fn CascadeWindows(
        hwndParent: HWND,
        wHow: UINT,
        lpRect: *const RECT,
        cKids: UINT,
        lpKids: *const HWND,
    ) -> u16;
}

// uxtheme (stdcall)
extern "system" {
    fn GetThemePartSize(
        hTheme: HTHEME,
        hdc: HDC,
        iPartId: i32,
        iStateId: i32,
        prc: *const RECT,
        eSize: i32,
        psz: *mut SIZE,
    ) -> HRESULT;
    fn DrawThemeBackground(
        hTheme: HTHEME,
        hdc: HDC,
        iPartId: i32,
        iStateId: i32,
        prc: *const RECT,
        pClipRect: *const RECT,
    ) -> HRESULT;
}

// shell32 undoc (stdcall)
extern "system" {
    fn ExitWindowsDialog(hWndOwner: HWND);
    fn LogoffWindowsDialog(hWndOwner: HWND) -> i32;
    fn RunFileDlg(
        hWndOwner: HWND,
        hIcon: HICON,
        lpstrDirectory: LPCWSTR,
        lpstrTitle: LPCWSTR,
        lpstrDescription: LPCWSTR,
        uFlags: UINT,
    );
    fn SHFindFiles(pidlRoot: *const c_void, pidlSaved: *const c_void) -> BOOL;
    fn SHFindComputer(pidlRoot: *const c_void, pidlSaved: *const c_void) -> BOOL;
    fn SHGetSpecialFolderPathW(hwndOwner: HWND, szPath: LPWSTR, csidl: i32, fCreate: BOOL) -> BOOL;
    fn Shell_MergeMenus(
        hmDst: HMENU,
        hmSrc: HMENU,
        uInsert: UINT,
        uIDAdjust: UINT,
        uIDAdjustMax: UINT,
        uFlags: u32,
    ) -> UINT;
}

// notifyiconscust.rs (Rust) — #[no_mangle] extern "C" (cdecl).
extern "C" {
    fn ShowCustomizeNotifyIcons(hInst: HINSTANCE, hwndTaskbar: HWND);
}

// shlwapi (stdcall) — appbar shared-memory + UI activation
extern "system" {
    fn SHLockShared(hData: HANDLE, dwProcessId: DWORD) -> *mut c_void;
    fn SHUnlockShared(pvData: *mut c_void) -> BOOL;
    fn IUnknown_UIActivateIO(punk: *mut IUnknown, fActivate: BOOL, lpMsg: *mut MSG) -> HRESULT;
}

// comctl32 (stdcall)
extern "system" {
    fn ImageList_LoadImageW(
        hi: HINSTANCE,
        lpbmp: LPCWSTR,
        cx: i32,
        cGrow: i32,
        crMask: COLORREF,
        uType: UINT,
        uFlags: UINT,
    ) -> HIMAGELIST;
    fn ImageList_Destroy(himl: HIMAGELIST) -> BOOL;
}

// shlwapi SHSetWindowBits (the SetWindowStyle helper target)
extern "system" {
    fn SHSetWindowBits(hWnd: HWND, iType: i32, dwMask: DWORD, dwFlags: DWORD) -> DWORD;
}

// trayprop.h (still C++)
extern "C" {
    fn DisplayTrayProperties(hwndOwner: HWND, hwndTaskbar: HWND);
}

// still-C++ / already-Rust helpers reached by C linkage
extern "C" {
    // trayntfy.cpp — STILL C++
    fn CTrayNotifyWnd_CreateInstance(hwndParent: HWND, riid: REFIID, ppv: *mut *mut c_void) -> HRESULT;
    // Rust modules
    fn CTaskBand_CreateInstance(
        Tray: *mut ITrayWindow,
        hWndStartButton: HWND,
        riid: REFIID,
        ppv: *mut *mut c_void,
    ) -> HRESULT;
    fn CTrayBandSite_CreateInstance(
        tray: *mut ITrayWindow,
        pTaskBand: *mut IDeskBand,
        pBandSite: *mut *mut ITrayBandSite,
    ) -> HRESULT;
    fn CStartMenuBtnCtxMenu_CreateInstance(
        TrayWnd: *mut ITrayWindow,
        hWndOwner: HWND,
        ppCtxMenu: *mut *mut IContextMenu,
    ) -> HRESULT;
    fn CreateStartMenu(
        Tray: *mut ITrayWindow,
        ppMenuBand: *mut *mut IMenuBand,
        hbmBanner: HBITMAP,
        bSmallIcons: BOOL,
    ) -> *mut IMenuPopup;
    fn InitShellServices(phdpa: *mut HDPA) -> HRESULT;
    fn ShutdownShellServices(hdpa: HDPA) -> HRESULT;
}

pub type HIMAGELIST = *mut c_void;

// ============================================================================
// small helpers
// ============================================================================

#[inline]
unsafe fn guid_eq(a: REFIID, b: *const IID) -> bool {
    let a = &*a;
    let b = &*b;
    a.Data1 == b.Data1 && a.Data2 == b.Data2 && a.Data3 == b.Data3 && a.Data4 == b.Data4
}

/// ATL/ROS free helper `SetWindowStyle(hWnd, dwMask, dwFlags)` == set the bits
/// in `dwMask` of GWL_STYLE to `dwFlags` (SHSetWindowBits over GWL_STYLE).
#[inline]
unsafe fn set_window_style(hwnd: HWND, mask: DWORD, flags: DWORD) {
    SHSetWindowBits(hwnd, GWL_STYLE as i32, mask, flags);
}

// GetWindowLongPtrW / GetClassLongPtrW are C macros on i386 (== the non-Ptr W
// forms) — no real import there; real LONG_PTR imports on LLP64.
#[cfg(target_arch = "x86")]
#[inline]
unsafe fn get_window_long_ptr(hwnd: HWND, idx: i32) -> LONG_PTR {
    GetWindowLongW(hwnd, idx) as LONG_PTR
}
#[cfg(not(target_arch = "x86"))]
#[inline]
unsafe fn get_window_long_ptr(hwnd: HWND, idx: i32) -> LONG_PTR {
    GetWindowLongPtrW(hwnd, idx)
}
#[cfg(target_arch = "x86")]
#[inline]
unsafe fn get_class_long_ptr(hwnd: HWND, idx: i32) -> ULONG_PTR {
    GetClassLongW(hwnd, idx) as ULONG_PTR
}
#[cfg(not(target_arch = "x86"))]
#[inline]
unsafe fn get_class_long_ptr(hwnd: HWND, idx: i32) -> ULONG_PTR {
    GetClassLongPtrW(hwnd, idx)
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

#[inline]
unsafe fn wcscpy_local(dst: *mut u16, src: *const u16) {
    let mut i = 0usize;
    loop {
        let c = *src.add(i);
        *dst.add(i) = c;
        if c == 0 {
            break;
        }
        i += 1;
    }
}

#[inline]
unsafe fn wcschr_local(mut s: *mut u16, c: u16) -> *mut u16 {
    loop {
        if *s == c {
            return s;
        }
        if *s == 0 {
            return null_mut();
        }
        s = s.add(1);
    }
}

#[inline]
unsafe fn wcscmp_local(a: *const u16, b: *const u16) -> i32 {
    let mut i = 0usize;
    loop {
        let ca = *a.add(i);
        let cb = *b.add(i);
        if ca != cb {
            return ca as i32 - cb as i32;
        }
        if ca == 0 {
            return 0;
        }
        i += 1;
    }
}

// WNDENUMPROC / MONITORENUMPROC are stdcall on i386 and C on LLP64, exactly like WNDPROC —
// which is what `extern "system"` MEANS, so one definition covers all three arches. These
// forked the body under complementary cfgs only because the island spelled the slot with the
// arch's literal convention and Rust unifies fn-pointer ABIs nominally; the slots are spelled
// `system` now (normalize_island_abi.py).
macro_rules! wndenum_cb {
    ($name:ident, $inner:path) => {
        unsafe extern "system" fn $name(h: HWND, l: LPARAM) -> BOOL {
            $inner(h, l)
        }
    };
}
macro_rules! monitorenum_cb {
    ($name:ident, $inner:path) => {
        unsafe extern "system" fn $name(h: HMONITOR, dc: HDC, r: *mut RECT, l: LPARAM) -> BOOL {
            $inner(h, dc, r, l)
        }
    };
}

// ============================================================================
// IID_IShellDesktopTray — was a C++ `const GUID` in traywnd.cpp referenced by
// explorer-sys via `extern "C" { pub static IID_IShellDesktopTray: IID; }`.
// Now that traywnd is Rust it becomes the #[no_mangle] DEFINITION that binding
// resolves to. {213e2df9-9a14-4328-99b1-6961f9143ce9}
// ============================================================================
#[no_mangle]
pub static IID_IShellDesktopTray: IID = IID {
    Data1: 0x213e_2df9,
    Data2: 0x9a14,
    Data3: 0x4328,
    Data4: [0x99, 0xb1, 0x69, 0x61, 0xf9, 0x14, 0x3c, 0xe9],
};

// ============================================================================
// appbar interop structs (undocshell.h, pshpack8). Layout MUST match the C
// header byte-for-byte per arch — these arrive via cross-process WM_COPYDATA.
// ============================================================================

#[repr(C)]
#[derive(Clone, Copy)]
struct APPBARDATAINTEROP {
    cbSize: DWORD,
    hWnd32: u32,
    uCallbackMessage: UINT,
    uEdge: UINT,
    rc: RECT,
    lParam64: i64,
}

#[repr(C)]
struct APPBAR_COMMAND {
    abd: APPBARDATAINTEROP,
    dwMessage: DWORD,
    // APPBAR_OUTPUT: UINT64 on LLP64 (win64), HANDLE (4 bytes) on i386.
    #[cfg(target_pointer_width = "64")]
    hOutput: u64,
    #[cfg(target_pointer_width = "32")]
    hOutput: u32,
    dwProcessId: DWORD,
}

/// tagAPPBAR (appbar.h) — the per-registered-appbar record stored in the DPA.
#[repr(C)]
struct APPBAR {
    hWnd: HWND,
    uCallbackMessage: UINT,
    uEdge: UINT,
    rc: RECT,
}

/// RecomputeWorkArea return (appbar.h WORKAREA_TYPE).
const WORKAREA_NO_TRAY_AREA: i32 = 0;
const WORKAREA_IS_NOT_MONITOR: i32 = 1;
const WORKAREA_SAME_AS_MONITOR: i32 = 2;

#[inline]
unsafe fn appbar_lock_output(pdata: *mut APPBAR_COMMAND) -> *mut APPBARDATAINTEROP {
    SHLockShared((*pdata).hOutput as usize as HANDLE, (*pdata).dwProcessId)
        as *mut APPBARDATAINTEROP
}
#[inline]
unsafe fn appbar_unlock_output(pout: *mut APPBARDATAINTEROP) {
    SHUnlockShared(pout as *mut c_void);
}
#[inline]
fn edge_is_vertical(u_edge: UINT) -> bool {
    u_edge == ABE_TOP || u_edge == ABE_BOTTOM
}

// ============================================================================
// file-scope globals (CSimpleArray<T> + g_Arrangement)
// ============================================================================

// enum { NONE, TILED, CASCADED } g_Arrangement
const ARRANGE_NONE: i32 = 0;
const ARRANGE_TILED: i32 = 1;
const ARRANGE_CASCADED: i32 = 2;
static mut g_Arrangement: i32 = ARRANGE_NONE;

#[repr(C)]
#[derive(Clone, Copy)]
struct WINDOWPOSBACKUPDATA {
    hwnd: HWND,
    wplt: WINDOWPLACEMENT,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct MINWNDPOS {
    hwnd: HWND,
    wndpl: WINDOWPLACEMENT,
}

/// Minimal `CSimpleArray<T>` — a grow-by-doubling heap array (Add / GetSize /
/// index / RemoveAll), enough for g_WindowPosBackup + g_MinimizedAll.
struct SimpleArray<T: Copy> {
    data: *mut T,
    size: i32,
    cap: i32,
}
impl<T: Copy> SimpleArray<T> {
    const fn new() -> Self {
        SimpleArray {
            data: null_mut(),
            size: 0,
            cap: 0,
        }
    }
    unsafe fn get_size(&self) -> i32 {
        self.size
    }
    unsafe fn add(&mut self, item: T) {
        if self.size >= self.cap {
            let newcap = if self.cap == 0 { 8 } else { self.cap * 2 };
            let bytes = (newcap as usize * size_of::<T>()) as SIZE_T;
            let np = if self.data.is_null() {
                HeapAlloc(hProcessHeap, 0, bytes) as *mut T
            } else {
                HeapReAlloc(hProcessHeap, 0, self.data as *mut c_void, bytes) as *mut T
            };
            if np.is_null() {
                return;
            }
            self.data = np;
            self.cap = newcap;
        }
        *self.data.add(self.size as usize) = item;
        self.size += 1;
    }
    unsafe fn at(&self, i: i32) -> T {
        *self.data.add(i as usize)
    }
    unsafe fn remove_all(&mut self) {
        if !self.data.is_null() {
            HeapFree(hProcessHeap, 0, self.data as *mut c_void);
        }
        self.data = null_mut();
        self.size = 0;
        self.cap = 0;
    }
}

static mut g_WindowPosBackup: SimpleArray<WINDOWPOSBACKUPDATA> = SimpleArray::new();
static mut g_MinimizedAll: SimpleArray<MINWNDPOS> = SimpleArray::new();

// ============================================================================
// window-arrangement / minimize free functions (traywnd.cpp file scope)
// ============================================================================

unsafe fn backup_windows_pos_proc_impl(hwnd: HWND, _l: LPARAM) -> BOOL {
    let h_desk = GetDesktopWindow();
    if IsWindowVisible(hwnd) != 0 && IsIconic(hwnd) == 0 && hwnd != h_desk {
        let mut wposdata: WINDOWPOSBACKUPDATA = core::mem::zeroed();
        wposdata.hwnd = hwnd;
        wposdata.wplt.length = size_of::<WINDOWPLACEMENT>() as UINT;
        GetWindowPlacement(hwnd, &mut wposdata.wplt);
        (*&raw mut g_WindowPosBackup).add(wposdata);
    }
    TRUE as BOOL
}
wndenum_cb!(BackupWindowsPosProc, backup_windows_pos_proc_impl);

unsafe fn BackupWindowPos() {
    EnumWindows(Some(BackupWindowsPosProc), 0);
}

unsafe fn RestoreWindowPos() {
    g_Arrangement = ARRANGE_NONE;
    let mut i = (*&raw const g_WindowPosBackup).get_size() - 1;
    while i >= 0 {
        let e = (*&raw const g_WindowPosBackup).at(i);
        SetWindowPlacement(e.hwnd, &e.wplt);
        i -= 1;
    }
    (*&raw mut g_WindowPosBackup).remove_all();
}

unsafe fn CanBeMinimized(hwnd: HWND) -> BOOL {
    if IsWindowVisible(hwnd) != 0
        && IsIconic(hwnd) == 0
        && IsWindowEnabled(hwnd) != 0
        && IsHungAppWindow(hwnd) == 0
    {
        if get_class_long_ptr(hwnd, GCW_ATOM) == WC_DIALOG_ATOM as ULONG_PTR {
            return TRUE as BOOL;
        }
        let exstyle = get_window_long_ptr(hwnd, GWL_EXSTYLE) as DWORD;
        if exstyle & WS_EX_TOPMOST == 0 {
            return TRUE as BOOL;
        }
    }
    FALSE as BOOL
}

#[repr(C)]
struct EFFECTIVE_INFO {
    hwndFound: HWND,
    hwndDesktop: HWND,
    hwndProgman: HWND,
    hTrayWnd: HWND,
    bMustBeInMonitor: BOOL,
}

unsafe fn find_effective_proc_impl(hwnd: HWND, l_param: LPARAM) -> BOOL {
    let pei = l_param as *mut EFFECTIVE_INFO;

    if CanBeMinimized(hwnd) == 0 {
        return TRUE as BOOL;
    }
    if (*pei).hTrayWnd == hwnd || (*pei).hwndDesktop == hwnd || (*pei).hwndProgman == hwnd {
        return TRUE as BOOL;
    }
    if (*pei).bMustBeInMonitor != 0 {
        let h_mon = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
        if !h_mon.is_null() {
            let mut info: MONITORINFO = core::mem::zeroed();
            info.cbSize = size_of::<MONITORINFO>() as DWORD;
            if GetMonitorInfoW(h_mon, &mut info) != 0 {
                let rc_monitor = info.rcMonitor;
                let mut rc_window: RECT = core::mem::zeroed();
                let mut rc_intersect: RECT = core::mem::zeroed();
                GetWindowRect(hwnd, &mut rc_window);
                if IntersectRect(&mut rc_intersect, &rc_monitor, &rc_window) == 0 {
                    return TRUE as BOOL;
                }
            }
        }
    }
    (*pei).hwndFound = hwnd;
    FALSE as BOOL
}
wndenum_cb!(FindEffectiveProc, find_effective_proc_impl);

unsafe fn IsThereAnyEffectiveWindow(b_must_be_in_monitor: BOOL) -> BOOL {
    let mut ei: EFFECTIVE_INFO = core::mem::zeroed();
    ei.hwndFound = null_mut();
    ei.hwndDesktop = GetDesktopWindow();
    ei.hTrayWnd = FindWindowW(szTrayWindowClass.as_ptr(), null());
    ei.hwndProgman = FindWindowW(w!("Progman").as_ptr(), null());
    ei.bMustBeInMonitor = b_must_be_in_monitor;
    EnumWindows(Some(FindEffectiveProc), &mut ei as *mut EFFECTIVE_INFO as LPARAM);
    (!ei.hwndFound.is_null()) as BOOL
}

// ============================================================================
// CTrayShowDesktopButton — STILL C++ (traydeskbtn.cpp). Opaque here; its
// methods/fields are reached via extern "C" shims the integrator must add
// (traydeskbtn_shim.cpp — see MANIFEST). The appbar-shim pattern in reverse.
// ============================================================================

#[repr(C)]
struct CTrayShowDesktopButton {
    _opaque: [u8; 0],
}

extern "C" {
    fn TrayShowDesktopButton_PtInButton(p: *mut CTrayShowDesktopButton, ppt: *const POINT) -> BOOL;
    fn TrayShowDesktopButton_OnLButtonDown(
        p: *mut CTrayShowDesktopButton,
        uMsg: UINT,
        wParam: WPARAM,
        lParam: LPARAM,
        bHandled: *mut BOOL,
    ) -> LRESULT;
    fn TrayShowDesktopButton_OnLButtonUp(
        p: *mut CTrayShowDesktopButton,
        uMsg: UINT,
        wParam: WPARAM,
        lParam: LPARAM,
        bHandled: *mut BOOL,
    ) -> LRESULT;
    fn TrayShowDesktopButton_Click(p: *mut CTrayShowDesktopButton);
    fn TrayShowDesktopButton_GetHWND(p: *mut CTrayShowDesktopButton) -> HWND;
    fn TrayShowDesktopButton_IsWindow(p: *mut CTrayShowDesktopButton) -> BOOL;
    fn TrayShowDesktopButton_GetPressed(p: *mut CTrayShowDesktopButton) -> BOOL;
}

// ============================================================================
// CStartButton — a subclassed WC_BUTTON embedded by value in CTrayWindow.
// Same subclass model as syspager's CNotifyToolbar: `this` in GWLP_USERDATA,
// unhandled messages forward to the saved super proc via CallWindowProcW.
// ============================================================================

#[repr(C)]
pub struct CStartButton {
    m_hWnd: HWND,
    m_pfnSuperWindowProc: WNDPROC,
    m_ImageList: HIMAGELIST,
    m_Size: SIZE,
    m_Font: HFONT,
}

unsafe fn sb_ctor(sb: *mut CStartButton) {
    (*sb).m_hWnd = null_mut();
    (*sb).m_pfnSuperWindowProc = None;
    (*sb).m_ImageList = null_mut();
    (*sb).m_Size = SIZE { cx: 0, cy: 0 };
    (*sb).m_Font = null_mut();
}

/// ~CStartButton().
unsafe fn sb_dtor(sb: *mut CStartButton) {
    if !(*sb).m_ImageList.is_null() {
        ImageList_Destroy((*sb).m_ImageList);
    }
    if !(*sb).m_Font.is_null() {
        DeleteObject((*sb).m_Font as HGDIOBJ);
    }
}

unsafe fn sb_get_size(sb: *mut CStartButton) -> SIZE {
    (*sb).m_Size
}

unsafe fn sb_update_size(sb: *mut CStartButton) {
    let mut size = SIZE { cx: 0, cy: 0 };
    if (*sb).m_ImageList.is_null()
        || SendMessageW(
            (*sb).m_hWnd,
            BCM_GETIDEALSIZE,
            0,
            &mut size as *mut SIZE as LPARAM,
        ) == 0
    {
        size.cx = 2 * GetSystemMetrics(SM_CXEDGE as i32) + GetSystemMetrics(SM_CYCAPTION as i32) * 3;
    }
    let cy_caption = GetSystemMetrics(SM_CYCAPTION as i32);
    if size.cy < cy_caption {
        size.cy = cy_caption;
    }
    (*sb).m_Size = size;
}

unsafe fn sb_update_font(sb: *mut CStartButton) {
    let mut ncm: NONCLIENTMETRICSW = core::mem::zeroed();
    ncm.cbSize = size_of::<NONCLIENTMETRICSW>() as UINT;
    if SystemParametersInfoW(
        SPI_GETNONCLIENTMETRICS,
        size_of::<NONCLIENTMETRICSW>() as UINT,
        &mut ncm as *mut NONCLIENTMETRICSW as *mut c_void,
        0,
    ) == 0
    {
        return;
    }
    if !(*sb).m_Font.is_null() {
        DeleteObject((*sb).m_Font as HGDIOBJ);
    }
    ncm.lfCaptionFont.lfWeight = FW_BOLD as LONG;
    (*sb).m_Font = CreateFontIndirectW(&ncm.lfCaptionFont);
    SendMessageW((*sb).m_hWnd, WM_SETFONT, (*sb).m_Font as WPARAM, 0);
}

unsafe fn sb_initialize(sb: *mut CStartButton) {
    // HACK & FIXME: CORE-18016 — m_hWnd=NULL then SubclassWindow.
    let h_wnd = (*sb).m_hWnd;
    (*sb).m_hWnd = null_mut();
    sb_subclass_window(sb, h_wnd);

    SetWindowTheme((*sb).m_hWnd, w!("Start").as_ptr(), null());

    (*sb).m_ImageList = ImageList_LoadImageW(
        hExplorerInstance,
        IDB_START as usize as LPCWSTR,
        0,
        0,
        0,
        IMAGE_BITMAP,
        LR_LOADTRANSPARENT | LR_CREATEDIBSECTION,
    );

    // BUTTON_IMAGELIST bil = { m_ImageList, {1,1,1,1}, BUTTON_IMAGELIST_ALIGN_LEFT }
    #[repr(C)]
    struct BUTTON_IMAGELIST {
        himl: HIMAGELIST,
        margin: RECT,
        uAlign: UINT,
    }
    let bil = BUTTON_IMAGELIST {
        himl: (*sb).m_ImageList,
        margin: RECT {
            left: 1,
            top: 1,
            right: 1,
            bottom: 1,
        },
        uAlign: BUTTON_IMAGELIST_ALIGN_LEFT,
    };
    SendMessageW(
        (*sb).m_hWnd,
        BCM_SETIMAGELIST,
        0,
        &bil as *const BUTTON_IMAGELIST as LPARAM,
    );
    sb_update_size(sb);
}

unsafe fn sb_create(sb: *mut CStartButton, hwnd_parent: HWND) -> HWND {
    let mut sz_start_caption: [u16; 32] = [0; 32];
    if LoadStringW(
        hExplorerInstance,
        IDS_START as UINT,
        sz_start_caption.as_mut_ptr(),
        sz_start_caption.len() as i32,
    ) == 0
    {
        wcscpy_local(sz_start_caption.as_mut_ptr(), w!("Start").as_ptr());
    }

    let dw_style: DWORD =
        WS_CHILD | WS_VISIBLE | WS_CLIPSIBLINGS | BS_PUSHBUTTON | BS_LEFT | BS_VCENTER;

    // HACK & FIXME: CORE-18016
    (*sb).m_hWnd = CreateWindowExW(
        0,
        WC_BUTTON_W.as_ptr(),
        sz_start_caption.as_ptr(),
        dw_style,
        0,
        0,
        0,
        0,
        hwnd_parent,
        IDC_STARTBTN as HMENU,
        hExplorerInstance,
        null_mut(),
    );

    if !(*sb).m_hWnd.is_null() {
        sb_initialize(sb);
    }
    (*sb).m_hWnd
}

/// CStartButton::OnLButtonDown (MESSAGE_HANDLER WM_LBUTTONDOWN).
unsafe fn sb_on_lbutton_down(
    sb: *mut CStartButton,
    u_msg: UINT,
    w_param: WPARAM,
    _l_param: LPARAM,
) -> LRESULT {
    if u_msg == WM_KEYUP && w_param != VK_SPACE as WPARAM {
        return 0;
    }
    PostMessageW(GetParent((*sb).m_hWnd), TWM_OPENSTARTMENU, 0, 0);
    0
}

// ---- CStartButton subclass machinery ----

#[cfg(target_arch = "x86")]
unsafe fn set_wndproc(hwnd: HWND, np: WNDPROC) -> WNDPROC {
    let old = SetWindowLongW(hwnd, GWL_WNDPROC, transmute::<WNDPROC, LONG>(np));
    transmute::<LONG, WNDPROC>(old)
}
#[cfg(not(target_arch = "x86"))]
unsafe fn set_wndproc(hwnd: HWND, np: WNDPROC) -> WNDPROC {
    let old = SetWindowLongPtrW(hwnd, GWLP_WNDPROC, transmute::<WNDPROC, LONG_PTR>(np));
    transmute::<LONG_PTR, WNDPROC>(old)
}
#[cfg(not(target_arch = "x86"))]
extern "system" {
    fn SetWindowLongPtrW(hWnd: HWND, nIndex: i32, dwNewLong: LONG_PTR) -> LONG_PTR;
}

unsafe fn sb_subclass_window(sb: *mut CStartButton, hwnd: HWND) {
    set_window_this(hwnd, sb as *mut c_void);
    let old = set_wndproc(hwnd, Some(CStartButton_WindowProc));
    (*sb).m_pfnSuperWindowProc = old;
    (*sb).m_hWnd = hwnd;
}

unsafe fn sb_process(
    sb: *mut CStartButton,
    hWnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    if uMsg == WM_LBUTTONDOWN {
        return sb_on_lbutton_down(sb, uMsg, wParam, lParam);
    }
    // CWindowImplBaseT::DefWindowProc == CallWindowProc(super).
    CallWindowProcW((*sb).m_pfnSuperWindowProc, hWnd, uMsg, wParam, lParam)
}

unsafe extern "system" fn CStartButton_WindowProc(
    hWnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    CStartButton_window_proc_impl(hWnd, uMsg, wParam, lParam)
}

#[inline]
unsafe fn CStartButton_window_proc_impl(
    hWnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let sb = get_window_this(hWnd) as *mut CStartButton;
    if sb.is_null() {
        return DefWindowProcW(hWnd, uMsg, wParam, lParam);
    }
    sb_process(sb, hWnd, uMsg, wParam, lParam)
}

// ============================================================================
// CTrayWindow — the flat multi-vtable object (CAppBarManager base + CTrayWindow
// members + embedded CStartButton). Layout is free (only reached via interface
// pointers or its own WndProc), so vtbls first, then everything else.
// ============================================================================

// Flags union bit assignment (internal-only).
const F_INSIZEMOVE: DWORD = 1 << 0;
const F_ISDRAGGING: DWORD = 1 << 1;
const F_NEWPOSSIZE: DWORD = 1 << 2;

#[repr(C)]
pub struct CTrayWindow {
    // one vtbl ptr per QI/cast-reachable interface. ITrayWindow is the PRIMARY
    // (reached via `(ITrayWindow*)this` casts) — first, at offset 0.
    lpVtbl_TrayWindow: *const ITrayWindowVtbl,
    lpVtbl_ShellDesktopTray: *const IShellDesktopTrayVtbl,
    lpVtbl_OleWindow: *const IOleWindowVtbl,
    lpVtbl_ContextMenu: *const IContextMenuVtbl,
    ref_: LONG,
    m_hWnd: HWND,

    // --- CAppBarManager base fields ---
    m_hAppBarDPA: HDPA,
    m_ahwndAutoHideBars: [HWND; 4],

    // --- CTrayWindow members ---
    m_pShowDesktopButton: *mut CTrayShowDesktopButton,
    m_StartMenuBand: *mut IMenuBand,   // CComPtr
    m_StartMenuPopup: *mut IMenuPopup, // CComPtr
    m_TaskBand: *mut IDeskBand,        // CComPtr
    m_ContextMenu: *mut IContextMenu,  // CComPtr
    m_Theme: HTHEME,
    m_Font: HFONT,
    m_DesktopWnd: HWND,
    m_Rebar: HWND,
    m_TaskSwitch: HWND,
    m_TrayNotify: HWND,
    m_TrayNotifyInstance: *mut IUnknown, // CComPtr
    m_Position: DWORD,
    m_Monitor: HMONITOR,
    m_PreviousMonitor: HMONITOR,
    m_DraggingPosition: DWORD,
    m_DraggingMonitor: HMONITOR,
    m_TrayRects: [RECT; 4],
    m_TraySize: SIZE,
    m_TrayPropertiesOwner: HWND,
    m_RunFileDlgOwner: HWND,
    m_AutoHideState: UINT,
    m_AutoHideOffset: SIZE,
    m_MouseTrackingInfo: TRACKMOUSEEVENT,
    m_ShellServices: HDPA,
    m_TrayBandSite: *mut ITrayBandSite, // CComPtr (public)
    Flags: DWORD,

    // embedded by value
    m_StartButton: CStartButton,
}

// ---- object base recovery (CONTAINING_RECORD from each interface field) ----

macro_rules! from_iface {
    ($fn:ident, $iface:ty, $field:ident) => {
        #[inline]
        unsafe fn $fn(this: *mut $iface) -> *mut CTrayWindow {
            (this as usize - core::mem::offset_of!(CTrayWindow, $field)) as *mut CTrayWindow
        }
    };
}
from_iface!(from_tw, ITrayWindow, lpVtbl_TrayWindow);
from_iface!(from_sdt, IShellDesktopTray, lpVtbl_ShellDesktopTray);
from_iface!(from_ow, IOleWindow, lpVtbl_OleWindow);
from_iface!(from_cm, IContextMenu, lpVtbl_ContextMenu);

// ---- Flags accessors ----
impl CTrayWindow {
    #[inline]
    fn get_flag(&self, m: DWORD) -> bool {
        self.Flags & m != 0
    }
    #[inline]
    fn set_flag(&mut self, m: DWORD, v: bool) {
        if v {
            self.Flags |= m;
        } else {
            self.Flags &= !m;
        }
    }
}

// ---- shared IUnknown (BEGIN_COM_MAP: IShellDesktopTray, IOleWindow,
//      IContextMenu; IUnknown == first entry == IShellDesktopTray) ----

unsafe fn tw_qi(this: *mut CTrayWindow, riid: REFIID, ppv: *mut *mut c_void) -> HRESULT {
    if ppv.is_null() {
        return E_POINTER;
    }
    let p: *mut c_void = if guid_eq(riid, &IID_IUnknown) || guid_eq(riid, &IID_IShellDesktopTray) {
        addr_of_mut!((*this).lpVtbl_ShellDesktopTray) as *mut c_void
    } else if guid_eq(riid, &IID_IOleWindow) {
        addr_of_mut!((*this).lpVtbl_OleWindow) as *mut c_void
    } else if guid_eq(riid, &IID_IContextMenu) {
        addr_of_mut!((*this).lpVtbl_ContextMenu) as *mut c_void
    } else {
        *ppv = null_mut();
        return E_NOINTERFACE;
    };
    *ppv = p;
    tw_addref(this);
    S_OK
}

unsafe fn tw_addref(this: *mut CTrayWindow) -> ULONG {
    interlocked_increment(&mut (*this).ref_) as ULONG
}

unsafe fn tw_release(this: *mut CTrayWindow) -> ULONG {
    let n = interlocked_decrement(&mut (*this).ref_);
    if n == 0 {
        tw_dtor(this);
        HeapFree(hProcessHeap, 0, this as *mut c_void);
        return 0;
    }
    n as ULONG
}

// per-interface IUnknown trampolines (4 ifaces × QI/AddRef/Release)
macro_rules! iunknown_slots {
    ($qi:ident, $ar:ident, $rel:ident, $iface:ty, $recover:ident) => {
        unsafe extern "system" fn $qi(
            this: *mut $iface,
            riid: REFIID,
            ppv: *mut *mut c_void,
        ) -> HRESULT {
            tw_qi($recover(this), riid, ppv)
        }
        unsafe extern "system" fn $ar(this: *mut $iface) -> ULONG {
            tw_addref($recover(this))
        }
        unsafe extern "system" fn $rel(this: *mut $iface) -> ULONG {
            tw_release($recover(this))
        }
    };
}
iunknown_slots!(tw_QI, tw_AddRef, tw_Release, ITrayWindow, from_tw);
iunknown_slots!(sdt_QI, sdt_AddRef, sdt_Release, IShellDesktopTray, from_sdt);
iunknown_slots!(ow_QI, ow_AddRef, ow_Release, IOleWindow, from_ow);
iunknown_slots!(cm_QI, cm_AddRef, cm_Release, IContextMenu, from_cm);

// ============================================================================
// CAppBarManager — the pure-virtual accessors CTrayWindow overrides (resolved
// directly against the flat object) + the appbar.cpp method bodies. All free
// fns over `this: *mut CTrayWindow`.
// ============================================================================

#[inline]
unsafe fn ab_is_auto_hide_state(_this: *mut CTrayWindow) -> BOOL {
    (*&raw const g_TaskbarSettings).sr.AutoHide() as BOOL
}
#[inline]
unsafe fn ab_is_hiding_state(this: *mut CTrayWindow) -> BOOL {
    ((*this).m_AutoHideState == AUTOHIDE_HIDING) as BOOL
}
#[inline]
unsafe fn ab_is_always_on_top(_this: *mut CTrayWindow) -> BOOL {
    (*&raw const g_TaskbarSettings).sr.AlwaysOnTop() as BOOL
}
#[inline]
unsafe fn ab_get_monitor(this: *mut CTrayWindow) -> HMONITOR {
    (*this).m_Monitor
}
#[inline]
unsafe fn ab_get_previous_monitor(this: *mut CTrayWindow) -> HMONITOR {
    (*this).m_PreviousMonitor
}
#[inline]
unsafe fn ab_get_position(this: *mut CTrayWindow) -> i32 {
    (*this).m_Position as i32
}
#[inline]
unsafe fn ab_get_tray_rect(this: *mut CTrayWindow) -> *const RECT {
    &(*this).m_TrayRects[(*this).m_Position as usize]
}
#[inline]
unsafe fn ab_get_tray_wnd(this: *mut CTrayWindow) -> HWND {
    (*this).m_hWnd
}
#[inline]
unsafe fn ab_get_desktop_wnd(this: *mut CTrayWindow) -> HWND {
    (*this).m_DesktopWnd
}

unsafe fn ab_find_app_bar(this: *mut CTrayWindow, hwnd_app_bar: HWND) -> *mut APPBAR {
    if (*this).m_hAppBarDPA.is_null() {
        return null_mut();
    }
    let mut n_items = DPA_GetPtrCount((*this).m_hAppBarDPA);
    loop {
        n_items -= 1;
        if n_items < 0 {
            break;
        }
        let p_app_bar = DPA_GetPtr((*this).m_hAppBarDPA, n_items) as *mut APPBAR;
        if !p_app_bar.is_null() && hwnd_app_bar == (*p_app_bar).hWnd {
            return p_app_bar;
        }
    }
    null_mut()
}

unsafe fn ab_eliminate_app_bar(this: *mut CTrayWindow, i_item: i32) {
    LocalFree(DPA_GetPtr((*this).m_hAppBarDPA, i_item) as HLOCAL);
    DPA_DeletePtr((*this).m_hAppBarDPA, i_item);
}

unsafe fn ab_destroy_app_bar_dpa(this: *mut CTrayWindow) {
    if (*this).m_hAppBarDPA.is_null() {
        return;
    }
    let mut n_items = DPA_GetPtrCount((*this).m_hAppBarDPA);
    loop {
        n_items -= 1;
        if n_items < 0 {
            break;
        }
        LocalFree(DPA_GetPtr((*this).m_hAppBarDPA, n_items) as HLOCAL);
    }
    DPA_Destroy((*this).m_hAppBarDPA);
    (*this).m_hAppBarDPA = null_mut();
}

// ABM_NEW
unsafe fn ab_on_app_bar_new(this: *mut CTrayWindow, p_data: *const APPBAR_COMMAND) -> BOOL {
    let h_wnd = (*p_data).abd.hWnd32 as usize as HWND;

    if !(*this).m_hAppBarDPA.is_null() {
        if !ab_find_app_bar(this, h_wnd).is_null() {
            return FALSE as BOOL;
        }
    } else {
        (*this).m_hAppBarDPA = DPA_Create(4);
        if (*this).m_hAppBarDPA.is_null() {
            return FALSE as BOOL;
        }
    }

    let p_app_bar = LocalAlloc(LPTR, size_of::<APPBAR>() as SIZE_T) as *mut APPBAR;
    if !p_app_bar.is_null() {
        (*p_app_bar).hWnd = h_wnd;
        (*p_app_bar).uEdge = u32::MAX;
        (*p_app_bar).uCallbackMessage = (*p_data).abd.uCallbackMessage;
        if DPA_InsertPtr((*this).m_hAppBarDPA, i32::MAX, p_app_bar as *mut c_void) >= 0 {
            return TRUE as BOOL;
        }
        LocalFree(p_app_bar as HLOCAL);
    }
    FALSE as BOOL
}

// ABM_REMOVE
unsafe fn ab_on_app_bar_remove(this: *mut CTrayWindow, p_data: *const APPBAR_COMMAND) {
    if (*this).m_hAppBarDPA.is_null() {
        return;
    }
    let h_wnd = (*p_data).abd.hWnd32 as usize as HWND;
    let mut n_items = DPA_GetPtrCount((*this).m_hAppBarDPA);
    loop {
        n_items -= 1;
        if n_items < 0 {
            break;
        }
        let p_app_bar = DPA_GetPtr((*this).m_hAppBarDPA, n_items) as *mut APPBAR;
        if p_app_bar.is_null() {
            continue;
        }
        if (*p_app_bar).hWnd == h_wnd {
            let rc_old = (*p_app_bar).rc;
            ab_eliminate_app_bar(this, n_items);
            ab_stuck_app_change(this, h_wnd, &rc_old, null(), FALSE as BOOL);
        }
    }
}

unsafe fn ab_app_bar_subtract_rect(p_app_bar: *mut APPBAR, prc: *mut RECT) {
    match (*p_app_bar).uEdge {
        ABE_LEFT => (*prc).left = (*prc).left.max((*p_app_bar).rc.right),
        ABE_TOP => (*prc).top = (*prc).top.max((*p_app_bar).rc.bottom),
        ABE_RIGHT => (*prc).right = (*prc).right.min((*p_app_bar).rc.left),
        ABE_BOTTOM => (*prc).bottom = (*prc).bottom.min((*p_app_bar).rc.top),
        _ => {}
    }
}

unsafe fn ab_app_bar_outside_of(p1: *const APPBAR, p2: *const APPBAR) -> BOOL {
    if (*p1).uEdge != (*p2).uEdge {
        return FALSE as BOOL;
    }
    let r = match (*p2).uEdge {
        ABE_LEFT => (*p1).rc.left >= (*p2).rc.left,
        ABE_TOP => (*p1).rc.top >= (*p2).rc.top,
        ABE_RIGHT => (*p1).rc.right <= (*p2).rc.right,
        ABE_BOTTOM => (*p1).rc.bottom <= (*p2).rc.bottom,
        _ => false,
    };
    r as BOOL
}

unsafe fn ab_get_docked_rect(this: *mut CTrayWindow, prc_docked: *mut RECT) {
    *prc_docked = *ab_get_tray_rect(this);
    if ab_is_auto_hide_state(this) != 0 && ab_is_hiding_state(this) != 0 {
        ab_compute_hidden_rect(prc_docked, ab_get_position(this) as UINT);
    }
}

unsafe fn ab_compute_hidden_rect(prc: *mut RECT, u_side: UINT) {
    let mut mi: MONITORINFO = core::mem::zeroed();
    mi.cbSize = size_of::<MONITORINFO>() as DWORD;
    let h_monitor = MonitorFromRect(prc, MONITOR_DEFAULTTONULL);
    if GetMonitorInfoW(h_monitor, &mut mi) == 0 {
        return;
    }
    let rc_mon = mi.rcMonitor;
    let cxy = if edge_is_vertical(u_side) {
        (*prc).bottom - (*prc).top
    } else {
        (*prc).right - (*prc).left
    };
    match u_side {
        ABE_LEFT => {
            (*prc).right = rc_mon.left + GetSystemMetrics(SM_CXFRAME as i32) / 2;
            (*prc).left = (*prc).right - cxy;
        }
        ABE_TOP => {
            (*prc).bottom = rc_mon.top + GetSystemMetrics(SM_CYFRAME as i32) / 2;
            (*prc).top = (*prc).bottom - cxy;
        }
        ABE_RIGHT => {
            (*prc).left = rc_mon.right - GetSystemMetrics(SM_CXFRAME as i32) / 2;
            (*prc).right = (*prc).left + cxy;
        }
        ABE_BOTTOM => {
            (*prc).top = rc_mon.bottom - GetSystemMetrics(SM_CYFRAME as i32) / 2;
            (*prc).bottom = (*prc).top + cxy;
        }
        _ => {}
    }
}

unsafe fn ab_stuck_app_change(
    this: *mut CTrayWindow,
    hwnd_target: HWND,
    prc_old: *const RECT,
    prc_new: *const RECT,
    b_tray: BOOL,
) {
    const SET_WORKAREA_1: UINT = 1;
    const SET_WORKAREA_2: UINT = 2;
    const NEED_SIZING: UINT = 4;

    let mut rc_work_area1: RECT = core::mem::zeroed();
    let mut rc_work_area2: RECT = core::mem::zeroed();
    let mut h_mon1: HMONITOR = null_mut();
    let mut flags: UINT = 0;

    if !prc_old.is_null() {
        h_mon1 = if b_tray != 0 {
            ab_get_previous_monitor(this)
        } else {
            MonitorFromRect(prc_old, MONITOR_DEFAULTTONEAREST)
        };
        if !h_mon1.is_null() {
            let type1 = ab_recompute_work_area(this, ab_get_tray_rect(this), h_mon1, &mut rc_work_area1);
            if type1 == WORKAREA_IS_NOT_MONITOR {
                flags = SET_WORKAREA_1;
            }
            if type1 == WORKAREA_SAME_AS_MONITOR {
                flags = NEED_SIZING;
            }
        }
    }

    if !prc_new.is_null() {
        let h_mon2 = MonitorFromRect(prc_new, MONITOR_DEFAULTTONULL);
        if !h_mon2.is_null() && h_mon2 != h_mon1 {
            let type2 = ab_recompute_work_area(this, ab_get_tray_rect(this), h_mon2, &mut rc_work_area2);
            if type2 == WORKAREA_IS_NOT_MONITOR {
                flags |= SET_WORKAREA_2;
            } else if type2 == WORKAREA_SAME_AS_MONITOR && flags == 0 {
                flags = NEED_SIZING;
            }
        }
    }

    if flags & SET_WORKAREA_1 != 0 {
        let f_win_ini: UINT = if flags == SET_WORKAREA_1 && !ab_get_desktop_wnd(this).is_null() {
            SPIF_SENDCHANGE
        } else {
            0
        };
        SystemParametersInfoW(
            SPI_SETWORKAREA,
            TRUE as UINT,
            &mut rc_work_area1 as *mut RECT as *mut c_void,
            f_win_ini,
        );
        ab_redraw_desktop(ab_get_desktop_wnd(this), &mut rc_work_area1);
    }

    if flags & SET_WORKAREA_2 != 0 {
        let f_win_ini: UINT = if !ab_get_desktop_wnd(this).is_null() {
            SPIF_SENDCHANGE
        } else {
            0
        };
        SystemParametersInfoW(
            SPI_SETWORKAREA,
            TRUE as UINT,
            &mut rc_work_area2 as *mut RECT as *mut c_void,
            f_win_ini,
        );
        ab_redraw_desktop(ab_get_desktop_wnd(this), &mut rc_work_area2);
    }

    if b_tray != 0 || flags == NEED_SIZING {
        SendMessageW(ab_get_desktop_wnd(this), WM_SIZE, 0, 0);
    }

    ab_on_app_bar_notify_all(this, null_mut(), hwnd_target, ABN_POSCHANGED, TRUE as LPARAM);
}

unsafe fn ab_redraw_desktop(hwnd_desktop: HWND, prc: *mut RECT) {
    if hwnd_desktop.is_null() {
        return;
    }
    MapWindowPoints(null_mut(), hwnd_desktop, prc as *mut POINT, 2);
    RedrawWindow(
        hwnd_desktop,
        prc,
        null_mut(),
        RDW_ALLCHILDREN | RDW_ERASE | RDW_INVALIDATE,
    );
}

unsafe fn ab_recompute_work_area(
    this: *mut CTrayWindow,
    prc_tray: *const RECT,
    h_monitor: HMONITOR,
    prc_work_area: *mut RECT,
) -> i32 {
    let mut mi: MONITORINFO = core::mem::zeroed();
    mi.cbSize = size_of::<MONITORINFO>() as DWORD;
    if GetMonitorInfoW(h_monitor, &mut mi) == 0 {
        return WORKAREA_NO_TRAY_AREA;
    }

    if ab_is_auto_hide_state(this) != 0 {
        *prc_work_area = mi.rcMonitor;
    } else {
        SubtractRect(prc_work_area, &mi.rcMonitor, prc_tray);
    }

    if !(*this).m_hAppBarDPA.is_null() {
        let mut n_items = DPA_GetPtrCount((*this).m_hAppBarDPA);
        loop {
            n_items -= 1;
            if n_items < 0 {
                break;
            }
            let p_app_bar = DPA_GetPtr((*this).m_hAppBarDPA, n_items) as *mut APPBAR;
            if !p_app_bar.is_null()
                && h_monitor == MonitorFromRect(&(*p_app_bar).rc, MONITOR_DEFAULTTONULL)
            {
                ab_app_bar_subtract_rect(p_app_bar, prc_work_area);
            }
        }
    }

    if EqualRect(prc_work_area, &mi.rcWork) == 0 {
        return WORKAREA_IS_NOT_MONITOR;
    }
    if ab_is_auto_hide_state(this) != 0 || IsRectEmpty(prc_tray) != 0 {
        return WORKAREA_NO_TRAY_AREA;
    }
    WORKAREA_SAME_AS_MONITOR
}

unsafe fn monitor_enum_proc_impl(
    h_monitor: HMONITOR,
    _h_dc: HDC,
    prc: *mut RECT,
    l_param: LPARAM,
) -> BOOL {
    let this = l_param as *mut CTrayWindow;
    let mut rc_work_area: RECT = core::mem::zeroed();
    if ab_recompute_work_area(this, prc, h_monitor, &mut rc_work_area) != WORKAREA_IS_NOT_MONITOR {
        return TRUE as BOOL;
    }
    let hwnd_desktop = ab_get_desktop_wnd(this);
    SystemParametersInfoW(
        SPI_SETWORKAREA,
        0,
        &mut rc_work_area as *mut RECT as *mut c_void,
        if !hwnd_desktop.is_null() { SPIF_SENDCHANGE } else { 0 },
    );
    ab_redraw_desktop(hwnd_desktop, &mut rc_work_area);
    TRUE as BOOL
}
monitorenum_cb!(MonitorEnumProc, monitor_enum_proc_impl);

unsafe fn ab_recompute_all_workareas(this: *mut CTrayWindow) {
    EnumDisplayMonitors(
        null_mut(),
        null(),
        Some(MonitorEnumProc),
        this as LPARAM,
    );
}

unsafe fn ab_set_auto_hide_bar(
    this: *mut CTrayWindow,
    hwnd_target: HWND,
    b_set_or_reset: BOOL,
    u_side: UINT,
) -> BOOL {
    let phwnd_auto_hide = &mut (*this).m_ahwndAutoHideBars[u_side as usize];
    if IsWindow(*phwnd_auto_hide) == 0 {
        *phwnd_auto_hide = null_mut();
    }
    if b_set_or_reset != 0 {
        if (*phwnd_auto_hide).is_null() {
            *phwnd_auto_hide = hwnd_target;
        }
        (*phwnd_auto_hide == hwnd_target) as BOOL
    } else {
        if *phwnd_auto_hide == hwnd_target {
            *phwnd_auto_hide = null_mut();
        }
        TRUE as BOOL
    }
}

unsafe fn ab_on_app_bar_activation_change2(this: *mut CTrayWindow, hwnd_new_auto_hide: HWND, u_side: UINT) {
    let hwnd_auto_hide_bar = ab_on_app_bar_get_auto_hide_bar(this, u_side);
    if !hwnd_auto_hide_bar.is_null() && hwnd_auto_hide_bar != hwnd_new_auto_hide {
        PostMessageW(
            ab_get_tray_wnd(this),
            TWM_SETZORDER,
            hwnd_auto_hide_bar as WPARAM,
            u_side as LPARAM,
        );
    }
}

unsafe fn ab_get_app_bar_message(p_copy_data: *mut COPYDATASTRUCT) -> *mut APPBAR_COMMAND {
    let p_data = (*p_copy_data).lpData as *mut APPBAR_COMMAND;
    if (*p_copy_data).cbData as usize != size_of::<APPBAR_COMMAND>()
        || (*p_data).abd.cbSize as usize != size_of::<APPBARDATAINTEROP>()
    {
        return null_mut();
    }
    p_data
}

// ABM_GETSTATE
unsafe fn ab_on_app_bar_get_state(this: *mut CTrayWindow) -> UINT {
    (if ab_is_auto_hide_state(this) != 0 { ABS_AUTOHIDE_C } else { 0 })
        | (if ab_is_always_on_top(this) != 0 { ABS_ALWAYSONTOP_C } else { 0 })
}

// ABM_GETTASKBARPOS
unsafe fn ab_on_app_bar_get_taskbar_pos(this: *mut CTrayWindow, p_data: *mut APPBAR_COMMAND) -> BOOL {
    let p_output = appbar_lock_output(p_data);
    if p_output.is_null() {
        return FALSE as BOOL;
    }
    (*p_output).rc = *ab_get_tray_rect(this);
    (*p_output).uEdge = ab_get_position(this) as UINT;
    appbar_unlock_output(p_output);
    TRUE as BOOL
}

// ABM_QUERYPOS
unsafe fn ab_on_app_bar_query_pos(this: *mut CTrayWindow, p_data: *mut APPBAR_COMMAND) {
    let h_wnd = (*p_data).abd.hWnd32 as usize as HWND;
    let p_app_bar1 = ab_find_app_bar(this, h_wnd);
    if p_app_bar1.is_null() {
        return;
    }

    let p_output = appbar_lock_output(p_data);
    if p_output.is_null() {
        return;
    }
    (*p_output).rc = (*p_data).abd.rc;

    let h_mon1 = MonitorFromRect(&(*p_output).rc, MONITOR_DEFAULTTOPRIMARY);

    // Subtract tray rectangle if necessary.
    if h_mon1 == ab_get_monitor(this) && ab_is_auto_hide_state(this) == 0 {
        let mut dummy: APPBAR = core::mem::zeroed();
        dummy.uEdge = ab_get_position(this) as UINT;
        ab_get_docked_rect(this, &mut dummy.rc);
        ab_app_bar_subtract_rect(&mut dummy, &mut (*p_output).rc);
    }

    let u_edge = (*p_data).abd.uEdge;
    let mut n_items = DPA_GetPtrCount((*this).m_hAppBarDPA);
    loop {
        n_items -= 1;
        if n_items < 0 {
            break;
        }
        let p_app_bar2 = DPA_GetPtr((*this).m_hAppBarDPA, n_items) as *mut APPBAR;
        if p_app_bar2.is_null() || (*p_app_bar1).hWnd == (*p_app_bar2).hWnd {
            continue;
        }

        if (edge_is_vertical(u_edge) || !edge_is_vertical((*p_app_bar2).uEdge))
            && ((*p_app_bar1).uEdge != u_edge || ab_app_bar_outside_of(p_app_bar1, p_app_bar2) == 0)
        {
            if (*p_app_bar1).uEdge == u_edge || (*p_app_bar2).uEdge != u_edge {
                continue;
            }
        }

        let h_mon2 = MonitorFromRect(&(*p_app_bar2).rc, MONITOR_DEFAULTTONULL);
        if h_mon1 == h_mon2 {
            ab_app_bar_subtract_rect(p_app_bar2, &mut (*p_output).rc);
        }
    }

    appbar_unlock_output(p_output);
}

// ABM_SETPOS
unsafe fn ab_on_app_bar_set_pos(this: *mut CTrayWindow, p_data: *mut APPBAR_COMMAND) {
    let h_wnd = (*p_data).abd.hWnd32 as usize as HWND;
    let p_app_bar = ab_find_app_bar(this, h_wnd);
    if p_app_bar.is_null() {
        return;
    }

    ab_on_app_bar_query_pos(this, p_data);

    let p_output = appbar_lock_output(p_data);
    if p_output.is_null() {
        return;
    }

    let rc_old = (*p_app_bar).rc;
    let rc_new = (*p_data).abd.rc;
    let b_changed = EqualRect(&rc_old, &rc_new) == 0;

    (*p_app_bar).rc = rc_new;
    (*p_app_bar).uEdge = (*p_data).abd.uEdge;

    appbar_unlock_output(p_output);

    if b_changed {
        ab_stuck_app_change(this, h_wnd, &rc_old, &rc_new, FALSE as BOOL);
    }
}

unsafe fn ab_on_app_bar_notify_all(
    this: *mut CTrayWindow,
    h_mon: HMONITOR,
    hwnd_ignore: HWND,
    dw_notify: DWORD,
    l_param: LPARAM,
) {
    if (*this).m_hAppBarDPA.is_null() {
        return;
    }
    let mut n_items = DPA_GetPtrCount((*this).m_hAppBarDPA);
    loop {
        n_items -= 1;
        if n_items < 0 {
            break;
        }
        let p_app_bar = DPA_GetPtr((*this).m_hAppBarDPA, n_items) as *mut APPBAR;
        if p_app_bar.is_null() || (*p_app_bar).hWnd == hwnd_ignore {
            continue;
        }
        let hwnd_app_bar = (*p_app_bar).hWnd;
        if IsWindow(hwnd_app_bar) == 0 {
            ab_eliminate_app_bar(this, n_items);
            continue;
        }
        if h_mon.is_null() || h_mon == MonitorFromWindow(hwnd_app_bar, MONITOR_DEFAULTTONULL) {
            PostMessageW(hwnd_app_bar, (*p_app_bar).uCallbackMessage, dw_notify as WPARAM, l_param);
        }
    }
}

// ABM_ACTIVATE / ABM_WINDOWPOSCHANGED
unsafe fn ab_on_app_bar_activation_change(this: *mut CTrayWindow, p_data: *const APPBAR_COMMAND) {
    let h_wnd = (*p_data).abd.hWnd32 as usize as HWND;
    let p_app_bar = ab_find_app_bar(this, h_wnd);
    if p_app_bar.is_null() {
        return;
    }
    let hwnd_app_bar = (*p_app_bar).hWnd;
    let mut u_side = ABE_LEFT;
    while u_side <= ABE_BOTTOM {
        if (*this).m_ahwndAutoHideBars[u_side as usize] == hwnd_app_bar
            && u_side != (*p_app_bar).uEdge
        {
            return;
        }
        u_side += 1;
    }
    ab_on_app_bar_activation_change2(this, hwnd_app_bar, (*p_app_bar).uEdge);
}

// ABM_GETAUTOHIDEBAR
unsafe fn ab_on_app_bar_get_auto_hide_bar(this: *mut CTrayWindow, u_side: UINT) -> HWND {
    if u_side as usize >= 4 {
        return null_mut();
    }
    if IsWindow((*this).m_ahwndAutoHideBars[u_side as usize]) == 0 {
        (*this).m_ahwndAutoHideBars[u_side as usize] = null_mut();
    }
    (*this).m_ahwndAutoHideBars[u_side as usize]
}

// ABM_SETAUTOHIDEBAR
unsafe fn ab_on_app_bar_set_auto_hide_bar(this: *mut CTrayWindow, p_data: *const APPBAR_COMMAND) -> BOOL {
    if (*p_data).abd.uEdge as usize >= 4 {
        return FALSE as BOOL;
    }
    let hwnd_target = (*p_data).abd.hWnd32 as usize as HWND;
    ab_set_auto_hide_bar(this, hwnd_target, (*p_data).abd.lParam64 as BOOL, (*p_data).abd.uEdge)
}

// ABM_SETSTATE
unsafe fn ab_on_app_bar_set_state(this: *mut CTrayWindow, u_state: UINT) {
    if u_state & !(ABS_AUTOHIDE_C | ABS_ALWAYSONTOP_C) != 0 {
        return;
    }
    tw_set_auto_hide_state(this, (u_state & ABS_AUTOHIDE_C != 0) as BOOL);
    tw_update_always_on_top(this, (u_state & ABS_ALWAYSONTOP_C != 0) as BOOL);
}

// WM_COPYDATA TABDMC_APPBAR
unsafe fn ab_on_app_bar_message(this: *mut CTrayWindow, p_copy_data: *mut COPYDATASTRUCT) -> LRESULT {
    let p_data = ab_get_app_bar_message(p_copy_data);
    if p_data.is_null() {
        return 0;
    }
    match (*p_data).dwMessage {
        ABM_NEW => return ab_on_app_bar_new(this, p_data) as LRESULT,
        ABM_REMOVE => ab_on_app_bar_remove(this, p_data),
        ABM_QUERYPOS => ab_on_app_bar_query_pos(this, p_data),
        ABM_SETPOS => ab_on_app_bar_set_pos(this, p_data),
        ABM_GETSTATE => return ab_on_app_bar_get_state(this) as LRESULT,
        ABM_GETTASKBARPOS => return ab_on_app_bar_get_taskbar_pos(this, p_data) as LRESULT,
        ABM_ACTIVATE | ABM_WINDOWPOSCHANGED => ab_on_app_bar_activation_change(this, p_data),
        ABM_GETAUTOHIDEBAR => {
            return ab_on_app_bar_get_auto_hide_bar(this, (*p_data).abd.uEdge) as LRESULT
        }
        ABM_SETAUTOHIDEBAR => return ab_on_app_bar_set_auto_hide_bar(this, p_data) as LRESULT,
        ABM_SETSTATE => ab_on_app_bar_set_state(this, (*p_data).abd.lParam64 as UINT),
        _ => return FALSE as LRESULT,
    }
    TRUE as LRESULT
}

// ============================================================================
// CTrayWindow — command handling
// ============================================================================

unsafe fn tw_exec_resource_cmd(this: *mut CTrayWindow, id: i32) -> HRESULT {
    let mut sz_command: [u16; 256] = [0; 256];
    if LoadStringW(
        hExplorerInstance,
        id as UINT,
        sz_command.as_mut_ptr(),
        sz_command.len() as i32,
    ) == 0
    {
        return E_FAIL;
    }
    let psz_parameters = wcschr_local(sz_command.as_mut_ptr(), b'>' as u16);
    let params_ptr = if !psz_parameters.is_null() {
        *psz_parameters = 0;
        psz_parameters.add(1) as LPCWSTR
    } else {
        null()
    };
    ShellExecuteW(
        (*this).m_hWnd,
        null(),
        sz_command.as_ptr(),
        params_ptr,
        null(),
        SW_SHOWNORMAL as INT,
    );
    S_OK
}

unsafe fn tw_save_state(this: *mut CTrayWindow) {
    if SHRestricted(REST_NOSAVESET) != 0 {
        return;
    }
    SendMessageW((*this).m_DesktopWnd, WM_PROGMAN_SAVESTATE, 0, 0);
    if SHRestricted(REST_CLEARRECENTDOCSONEXIT) != 0 {
        ClearRecentAndMru();
    }
}

unsafe fn tw_do_exit_windows(this: *mut CTrayWindow) -> LRESULT {
    tw_save_state(this);
    ExitWindowsDialog((*this).m_hWnd);
    if (GetKeyState(VK_CONTROL as i32) & 0x8000u16 as i16) != 0
        && (GetKeyState(VK_SHIFT as i32) & 0x8000u16 as i16) != 0
        && (GetKeyState(VK_MENU as i32) & 0x8000u16 as i16) != 0
    {
        PostMessageW((*this).m_hWnd, WM_QUIT, 0, 0);
    }
    0
}

// --- RunFileDlg / TrayProperties worker threads ---

unsafe fn tw_run_file_dlg_thread(this: *mut CTrayWindow) -> u32 {
    let mut pos_rect: RECT = core::mem::zeroed();
    GetWindowRect((*this).m_StartButton.m_hWnd, &mut pos_rect);

    let hwnd = CreateWindowExW(
        0,
        WC_STATIC_W.as_ptr(),
        null(),
        WS_OVERLAPPED | WS_DISABLED | WS_CLIPSIBLINGS | WS_BORDER | SS_LEFT,
        pos_rect.left,
        pos_rect.top,
        pos_rect.right - pos_rect.left,
        pos_rect.bottom - pos_rect.top,
        null_mut(),
        null_mut(),
        null_mut(),
        null_mut(),
    );

    (*this).m_RunFileDlgOwner = hwnd;

    // default dir = %HOMEDRIVE%%HOMEPATH%
    let mut sz_default_dir: [u16; 520] = [0; 520];
    let n = GetEnvironmentVariableW(
        w!("HOMEDRIVE").as_ptr(),
        sz_default_dir.as_mut_ptr(),
        sz_default_dir.len() as DWORD,
    ) as usize;
    let n = if n < sz_default_dir.len() { n } else { 0 };
    GetEnvironmentVariableW(
        w!("HOMEPATH").as_ptr(),
        sz_default_dir.as_mut_ptr().add(n),
        (sz_default_dir.len() - n) as DWORD,
    );

    RunFileDlg(hwnd, null_mut(), sz_default_dir.as_ptr(), null(), null(), RFF_CALCDIRECTORY);

    (*this).m_RunFileDlgOwner = null_mut();
    DestroyWindow(hwnd);
    0
}

unsafe extern "system" fn s_RunFileDlgThread(p: LPVOID) -> u32 {
    tw_run_file_dlg_thread(p as *mut CTrayWindow)
}

unsafe fn tw_display_run_file_dlg(this: *mut CTrayWindow) {
    if !(*this).m_RunFileDlgOwner.is_null() {
        let h_run_dlg = GetLastActivePopup((*this).m_RunFileDlgOwner);
        if !h_run_dlg.is_null() && h_run_dlg != (*this).m_RunFileDlgOwner {
            SetForegroundWindow(h_run_dlg);
            return;
        }
    }
    let h_thread = CreateThread(null_mut(), 0, Some(s_RunFileDlgThread), this as LPVOID, 0, null_mut());
    if !h_thread.is_null() {
        CloseHandle(h_thread);
    }
}

unsafe fn tw_tray_properties_thread(this: *mut CTrayWindow) -> u32 {
    let mut pos_rect: RECT = core::mem::zeroed();
    GetWindowRect((*this).m_StartButton.m_hWnd, &mut pos_rect);

    let hwnd = CreateWindowExW(
        0,
        WC_STATIC_W.as_ptr(),
        null(),
        WS_OVERLAPPED | WS_DISABLED | WS_CLIPSIBLINGS | WS_BORDER | SS_LEFT,
        pos_rect.left,
        pos_rect.top,
        pos_rect.right - pos_rect.left,
        pos_rect.bottom - pos_rect.top,
        null_mut(),
        null_mut(),
        null_mut(),
        null_mut(),
    );

    (*this).m_TrayPropertiesOwner = hwnd;
    DisplayTrayProperties(hwnd, (*this).m_hWnd);
    (*this).m_TrayPropertiesOwner = null_mut();
    DestroyWindow(hwnd);
    0
}

unsafe extern "system" fn s_TrayPropertiesThread(p: LPVOID) -> u32 {
    tw_tray_properties_thread(p as *mut CTrayWindow)
}

unsafe fn tw_display_properties(this: *mut CTrayWindow) -> HWND {
    if !(*this).m_TrayPropertiesOwner.is_null() {
        let h_tray_prop = GetLastActivePopup((*this).m_TrayPropertiesOwner);
        if !h_tray_prop.is_null() && h_tray_prop != (*this).m_TrayPropertiesOwner {
            SetForegroundWindow(h_tray_prop);
            return null_mut();
        }
    }
    let h_thread = CreateThread(null_mut(), 0, Some(s_TrayPropertiesThread), this as LPVOID, 0, null_mut());
    if !h_thread.is_null() {
        CloseHandle(h_thread);
    }
    null_mut()
}

unsafe fn tw_open_common_start_menu_directory(hwnd_owner: HWND, lp_operation: LPCWSTR) {
    let mut sz_dir: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    // CSIDL_COMMON_STARTMENU == 0x16
    if SHGetSpecialFolderPathW(hwnd_owner, sz_dir.as_mut_ptr(), 0x16, FALSE as BOOL) != 0 {
        ShellExecuteW(hwnd_owner, lp_operation, sz_dir.as_ptr(), null(), null(), SW_SHOWNORMAL as INT);
    }
}

unsafe fn tw_open_task_manager(hwnd_owner: HWND) {
    ShellExecuteW(
        hwnd_owner,
        w!("open").as_ptr(),
        w!("taskmgr.exe").as_ptr(),
        null(),
        null(),
        SW_SHOWNORMAL as INT,
    );
}

unsafe fn tw_toggle_desktop(this: *mut CTrayWindow) {
    if IsThereAnyEffectiveWindow(TRUE as BOOL) != 0 {
        tw_show_desktop(this);
    } else {
        tw_restore_all(this);
    }
}

unsafe fn tw_exec_context_menu_cmd(this: *mut CTrayWindow, ui_cmd: UINT) -> BOOL {
    match ui_cmd {
        ID_SHELL_CMD_PROPERTIES => {
            tw_display_properties(this);
        }
        ID_SHELL_CMD_OPEN_ALL_USERS => {
            tw_open_common_start_menu_directory((*this).m_hWnd, w!("open").as_ptr());
        }
        ID_SHELL_CMD_EXPLORE_ALL_USERS => {
            tw_open_common_start_menu_directory((*this).m_hWnd, w!("explore").as_ptr());
        }
        ID_LOCKTASKBAR => {
            tw_handle_command(this, TRAYCMD_LOCK_TASKBAR);
        }
        ID_SHELL_CMD_OPEN_TASKMGR => {
            tw_open_task_manager((*this).m_hWnd);
        }
        ID_SHELL_CMD_UNDO_ACTION => {
            RestoreWindowPos();
        }
        ID_SHELL_CMD_SHOW_DESKTOP => {
            tw_show_desktop(this);
        }
        ID_SHELL_CMD_TILE_WND_H => {
            ab_on_app_bar_notify_all(this, null_mut(), null_mut(), ABN_WINDOWARRANGE, TRUE as LPARAM);
            if g_Arrangement == ARRANGE_NONE {
                BackupWindowPos();
            }
            TileWindows(null_mut(), MDITILE_HORIZONTAL, null(), 0, null());
            ab_on_app_bar_notify_all(this, null_mut(), null_mut(), ABN_WINDOWARRANGE, FALSE as LPARAM);
            g_Arrangement = ARRANGE_TILED;
        }
        ID_SHELL_CMD_TILE_WND_V => {
            ab_on_app_bar_notify_all(this, null_mut(), null_mut(), ABN_WINDOWARRANGE, TRUE as LPARAM);
            if g_Arrangement == ARRANGE_NONE {
                BackupWindowPos();
            }
            TileWindows(null_mut(), MDITILE_VERTICAL, null(), 0, null());
            ab_on_app_bar_notify_all(this, null_mut(), null_mut(), ABN_WINDOWARRANGE, FALSE as LPARAM);
            g_Arrangement = ARRANGE_TILED;
        }
        ID_SHELL_CMD_CASCADE_WND => {
            ab_on_app_bar_notify_all(this, null_mut(), null_mut(), ABN_WINDOWARRANGE, TRUE as LPARAM);
            if g_Arrangement == ARRANGE_NONE {
                BackupWindowPos();
            }
            CascadeWindows(null_mut(), MDITILE_SKIPDISABLED, null(), 0, null());
            ab_on_app_bar_notify_all(this, null_mut(), null_mut(), ABN_WINDOWARRANGE, FALSE as LPARAM);
            g_Arrangement = ARRANGE_CASCADED;
        }
        ID_SHELL_CMD_CUST_NOTIF => {
            ShowCustomizeNotifyIcons(hExplorerInstance, (*this).m_hWnd);
        }
        ID_SHELL_CMD_ADJUST_DAT => {
            ShellExecuteW((*this).m_hWnd, null(), w!("timedate.cpl").as_ptr(), null(), null(), SW_NORMAL as INT);
        }
        ID_SHELL_CMD_RESTORE_ALL => {
            tw_restore_all(this);
        }
        _ => return FALSE as BOOL,
    }
    TRUE as BOOL
}

unsafe fn tw_hide_start_menu(this: *mut CTrayWindow) {
    (*(*this).m_StartMenuPopup).OnSelect(MPOS_CANCELLEVEL);
}

unsafe fn tw_handle_hot_key(this: *mut CTrayWindow, id: DWORD) -> LRESULT {
    match id {
        IDHK_RUN => {
            tw_hide_start_menu(this);
            tw_display_run_file_dlg(this);
        }
        IDHK_HELP => {
            tw_exec_resource_cmd(this, IDS_HELP_COMMAND as i32);
        }
        IDHK_EXPLORE => {
            ShellExecuteW(null_mut(), null(), w!("explorer.exe").as_ptr(), w!("/e ,").as_ptr(), null(), 1);
        }
        IDHK_FIND => {
            SHFindFiles(null(), null());
        }
        IDHK_FIND_COMPUTER => {
            SHFindComputer(null(), null());
        }
        IDHK_SYS_PROPERTIES => {
            ShellExecuteW((*this).m_hWnd, null(), w!("sysdm.cpl").as_ptr(), null(), null(), SW_NORMAL as INT);
        }
        IDHK_NEXT_TASK => {}
        IDHK_PREV_TASK => {}
        IDHK_MINIMIZE_ALL => {
            tw_minimize_all(this, FALSE as BOOL);
        }
        IDHK_RESTORE_ALL => {
            tw_restore_all(this);
        }
        IDHK_DESKTOP => {
            tw_toggle_desktop(this);
        }
        IDHK_PAGER => {}
        _ => {}
    }
    0
}

unsafe fn tw_handle_command(this: *mut CTrayWindow, u_command: UINT) -> LRESULT {
    match u_command {
        TRAYCMD_STARTMENU => {}
        TRAYCMD_RUN_DIALOG => {
            tw_hide_start_menu(this);
            tw_display_run_file_dlg(this);
        }
        TRAYCMD_LOGOFF_DIALOG => {
            tw_save_state(this);
            LogoffWindowsDialog((*this).m_hWnd);
        }
        TRAYCMD_CASCADE => {
            CascadeWindows(null_mut(), MDITILE_SKIPDISABLED, null(), 0, null());
        }
        TRAYCMD_TILE_H => {
            TileWindows(null_mut(), MDITILE_HORIZONTAL, null(), 0, null());
        }
        TRAYCMD_TILE_V => {
            TileWindows(null_mut(), MDITILE_VERTICAL, null(), 0, null());
        }
        TRAYCMD_TOGGLE_DESKTOP => {
            tw_toggle_desktop(this);
        }
        TRAYCMD_DATE_AND_TIME => {
            ShellExecuteW((*this).m_hWnd, null(), w!("timedate.cpl").as_ptr(), null(), null(), SW_NORMAL as INT);
        }
        TRAYCMD_TASKBAR_PROPERTIES => {
            tw_display_properties(this);
        }
        TRAYCMD_MINIMIZE_ALL => {
            tw_minimize_all(this, FALSE as BOOL);
        }
        TRAYCMD_RESTORE_ALL => {
            tw_restore_all(this);
        }
        TRAYCMD_SHOW_DESKTOP => {
            tw_show_desktop(this);
        }
        TRAYCMD_SHOW_TASK_MGR => {
            tw_open_task_manager((*this).m_hWnd);
        }
        TRAYCMD_CUSTOMIZE_TASKBAR => {}
        TRAYCMD_LOCK_TASKBAR => {
            if SHRestricted(REST_CLASSICSHELL) == 0 {
                tw_lock(this, (g_TaskbarSettings.bLock == 0) as BOOL);
                TaskbarSettings_save_global();
            }
        }
        TRAYCMD_HELP_AND_SUPPORT => {
            tw_exec_resource_cmd(this, IDS_HELP_COMMAND as i32);
        }
        TRAYCMD_CONTROL_PANEL => {}
        TRAYCMD_SHUTDOWN_DIALOG => {
            tw_do_exit_windows(this);
        }
        TRAYCMD_PRINTERS_AND_FAXES => {}
        TRAYCMD_LOCK_DESKTOP => {}
        TRAYCMD_SWITCH_USER_DIALOG => {}
        x if x == IDM_SEARCH || x == TRAYCMD_SEARCH_FILES => {
            SHFindFiles(null(), null());
        }
        TRAYCMD_SEARCH_COMPUTERS => {
            SHFindComputer(null(), null());
        }
        _ => {}
    }
    FALSE as LRESULT
}

/// `g_TaskbarSettings.Save()` — the Save() method is the Rust #[no_mangle]
/// TaskbarSettings_Save(&g_TaskbarSettings) in settings.rs.
unsafe fn TaskbarSettings_save_global() {
    extern "C" {
        fn TaskbarSettings_Save(this: *mut TaskbarSettings) -> i32;
    }
    TaskbarSettings_Save(addr_of_mut!(g_TaskbarSettings));
}

unsafe fn tw_track_menu(
    this: *mut CTrayWindow,
    h_menu: HMENU,
    ppt: *const POINT,
    hwnd_exclude: HWND,
    track_up: BOOL,
    is_context_menu: BOOL,
) -> UINT {
    let mut tmp: TPMPARAMS = core::mem::zeroed();
    let mut ptmp: *mut TPMPARAMS = null_mut();
    let mut pt: POINT = POINT { x: 0, y: 0 };

    if !hwnd_exclude.is_null() {
        if GetClientRect(hwnd_exclude, &mut tmp.rcExclude) != 0
            && MapWindowPoints(hwnd_exclude, null_mut(), &mut tmp.rcExclude as *mut RECT as *mut POINT, 2) != 0
        {
            ptmp = &mut tmp;
        }
    }

    if ppt.is_null() {
        if ptmp.is_null()
            && GetClientRect((*this).m_hWnd, &mut tmp.rcExclude) != 0
            && MapWindowPoints((*this).m_hWnd, null_mut(), &mut tmp.rcExclude as *mut RECT as *mut POINT, 2) != 0
        {
            ptmp = &mut tmp;
        }
        if !ptmp.is_null() {
            pt.x = (*ptmp).rcExclude.left;
            pt.y = (*ptmp).rcExclude.top;
        } else {
            pt.x = 0;
            pt.y = 0;
        }
    } else {
        pt = *ppt;
    }

    tmp.cbSize = size_of::<TPMPARAMS>() as UINT;

    let mut fu_flags: UINT = TPM_RETURNCMD | TPM_VERTICAL;
    fu_flags |= if track_up != 0 { TPM_BOTTOMALIGN } else { TPM_TOPALIGN };
    if is_context_menu != 0 {
        fu_flags |= TPM_RIGHTBUTTON;
    } else {
        fu_flags |= if track_up != 0 { TPM_VERNEGANIMATION } else { TPM_VERPOSANIMATION };
    }

    TrackPopupMenuEx(h_menu, fu_flags, pt.x, pt.y, (*this).m_hWnd, ptmp) as UINT
}

unsafe fn tw_track_ctx_menu(
    this: *mut CTrayWindow,
    context_menu: *mut IContextMenu,
    ppt: *const POINT,
    hwnd_exclude: HWND,
    _track_up: BOOL,
    _context: *mut c_void,
) -> HRESULT {
    let pt: POINT;
    let mut params: TPMPARAMS = core::mem::zeroed();
    let mut rc: RECT = core::mem::zeroed();
    let mut hr: HRESULT;
    let popup = CreatePopupMenu();

    if popup.is_null() {
        return E_FAIL;
    }

    if !ppt.is_null() {
        pt = *ppt;
    } else {
        GetWindowRect((*this).m_hWnd, &mut rc);
        pt = POINT { x: rc.left, y: rc.top };
    }

    hr = ((*(*context_menu).lpVtbl).QueryContextMenu)(context_menu, popup, 0, 0, u32::MAX, CMF_NORMAL);
    if FAILED_UNEXPECTEDLY(hr) != 0 {
        DestroyMenu(popup);
        return hr;
    }

    SetForegroundWindow((*this).m_hWnd);
    let u_command: UINT;
    if !hwnd_exclude.is_null() {
        GetWindowRect(hwnd_exclude, &mut rc);
        params = core::mem::zeroed();
        params.cbSize = size_of::<TPMPARAMS>() as UINT;
        params.rcExclude = rc;
        u_command = TrackPopupMenuEx(popup, TPM_RETURNCMD, pt.x, pt.y, (*this).m_hWnd, &mut params) as UINT;
    } else {
        u_command = TrackPopupMenuEx(popup, TPM_RETURNCMD, pt.x, pt.y, (*this).m_hWnd, null_mut()) as UINT;
    }
    PostMessageW((*this).m_hWnd, WM_NULL, 0, 0);

    if u_command != 0 {
        let mut cmi: CMINVOKECOMMANDINFO = core::mem::zeroed();
        cmi.cbSize = size_of::<CMINVOKECOMMANDINFO>() as DWORD;
        cmi.lpVerb = u_command as usize as LPCSTR; // MAKEINTRESOURCEA(uCommand)
        cmi.hwnd = (*this).m_hWnd;
        hr = ((*(*context_menu).lpVtbl).InvokeCommand)(context_menu, &cmi);
    } else {
        hr = S_FALSE;
    }

    DestroyMenu(popup);
    hr
}

// ============================================================================
// CTrayWindow — moving & sizing
// ============================================================================

unsafe fn tw_update_fonts(this: *mut CTrayWindow) {
    if !(*this).m_Theme.is_null() {
        return;
    }
    sb_update_font(addr_of_mut!((*this).m_StartButton));

    let mut ncm: NONCLIENTMETRICSW = core::mem::zeroed();
    ncm.cbSize = size_of::<NONCLIENTMETRICSW>() as UINT;
    if SystemParametersInfoW(
        SPI_GETNONCLIENTMETRICS,
        size_of::<NONCLIENTMETRICSW>() as UINT,
        &mut ncm as *mut NONCLIENTMETRICSW as *mut c_void,
        0,
    ) == 0
    {
        return;
    }

    if !(*this).m_Font.is_null() {
        DeleteObject((*this).m_Font as HGDIOBJ);
    }
    ncm.lfCaptionFont.lfWeight = FW_NORMAL as LONG;
    (*this).m_Font = CreateFontIndirectW(&ncm.lfCaptionFont);
    if (*this).m_Font.is_null() {
        return;
    }
    SendMessageW((*this).m_Rebar, WM_SETFONT, (*this).m_Font as WPARAM, TRUE as LPARAM);
    SendMessageW((*this).m_TaskSwitch, WM_SETFONT, (*this).m_Font as WPARAM, TRUE as LPARAM);
    SendMessageW((*this).m_TrayNotify, WM_SETFONT, (*this).m_Font as WPARAM, TRUE as LPARAM);
}

unsafe fn tw_get_screen_rect_from_rect(p_rect: *mut RECT, dw_flags: DWORD) -> HMONITOR {
    let mut mi: MONITORINFO = core::mem::zeroed();
    mi.cbSize = size_of::<MONITORINFO>() as DWORD;
    let mut h_mon = MonitorFromRect(p_rect, dw_flags);
    if !h_mon.is_null() && GetMonitorInfoW(h_mon, &mut mi) != 0 {
        *p_rect = mi.rcMonitor;
    } else {
        (*p_rect).left = 0;
        (*p_rect).top = 0;
        (*p_rect).right = GetSystemMetrics(SM_CXSCREEN as i32);
        (*p_rect).bottom = GetSystemMetrics(SM_CYSCREEN as i32);
        h_mon = null_mut();
    }
    h_mon
}

unsafe fn tw_get_monitor_from_rect(p_rect: *const RECT) -> HMONITOR {
    let mut h_mon = MonitorFromRect(p_rect, MONITOR_DEFAULTTONULL);
    if h_mon.is_null() {
        let pt = POINT {
            x: (*p_rect).left + ((*p_rect).right - (*p_rect).left) / 2,
            y: (*p_rect).top + ((*p_rect).bottom - (*p_rect).top) / 2,
        };
        h_mon = MonitorFromPoint(pt, MONITOR_DEFAULTTONEAREST);
    }
    h_mon
}

unsafe fn tw_get_screen_rect(h_monitor: HMONITOR, p_rect: *mut RECT) -> HMONITOR {
    let mut h_mon: HMONITOR = null_mut();

    if !h_monitor.is_null() {
        let mut mi: MONITORINFO = core::mem::zeroed();
        mi.cbSize = size_of::<MONITORINFO>() as DWORD;
        if GetMonitorInfoW(h_monitor, &mut mi) == 0 {
            h_mon = tw_get_monitor_from_rect(p_rect);
            if h_mon.is_null() || GetMonitorInfoW(h_mon, &mut mi) == 0 {
                h_mon = null_mut();
                (*p_rect).left = 0;
                (*p_rect).top = 0;
                (*p_rect).right = GetSystemMetrics(SM_CXSCREEN as i32);
                (*p_rect).bottom = GetSystemMetrics(SM_CYSCREEN as i32);
                return h_mon;
            }
        }
        *p_rect = mi.rcMonitor;
    } else {
        (*p_rect).left = 0;
        (*p_rect).top = 0;
        (*p_rect).right = GetSystemMetrics(SM_CXSCREEN as i32);
        (*p_rect).bottom = GetSystemMetrics(SM_CYSCREEN as i32);
    }
    h_mon
}

unsafe fn tw_adjust_sizer_rect(this: *mut CTrayWindow, rc: *mut RECT, pos: DWORD) {
    let i_sizer_part = [
        TBP_SIZINGBARLEFT,
        TBP_SIZINGBARTOP,
        TBP_SIZINGBARRIGHT,
        TBP_SIZINGBARBOTTOM,
    ];
    let mut size: SIZE = core::mem::zeroed();
    let pos = if pos > ABE_BOTTOM { ABE_BOTTOM } else { pos };

    let hr = GetThemePartSize(
        (*this).m_Theme,
        null_mut(),
        i_sizer_part[pos as usize],
        0,
        null(),
        TS_TRUE,
        &mut size,
    );
    if FAILED_UNEXPECTEDLY(hr) != 0 {
        return;
    }
    match pos {
        ABE_TOP => (*rc).bottom -= size.cy,
        ABE_BOTTOM => (*rc).top += size.cy,
        ABE_LEFT => (*rc).right -= size.cx,
        ABE_RIGHT => (*rc).left += size.cx,
        _ => {}
    }
}

unsafe fn tw_make_tray_rect_with_size(position: DWORD, p_tray_size: *const SIZE, p_rect: *mut RECT) {
    match position {
        ABE_LEFT => (*p_rect).right = (*p_rect).left + (*p_tray_size).cx,
        ABE_TOP => (*p_rect).bottom = (*p_rect).top + (*p_tray_size).cy,
        ABE_RIGHT => (*p_rect).left = (*p_rect).right - (*p_tray_size).cx,
        _ => (*p_rect).top = (*p_rect).bottom - (*p_tray_size).cy,
    }
}

unsafe fn tw_get_tray_rect_from_screen_rect(
    this: *mut CTrayWindow,
    position: DWORD,
    p_screen: *const RECT,
    p_tray_size: *const SIZE,
    p_rect: *mut RECT,
) {
    let p_tray_size = if p_tray_size.is_null() {
        &(*this).m_TraySize as *const SIZE
    } else {
        p_tray_size
    };
    *p_rect = *p_screen;
    if (*this).m_Theme.is_null() {
        InflateRect(p_rect, GetSystemMetrics(SM_CXEDGE as i32), GetSystemMetrics(SM_CYEDGE as i32));
    }
    tw_make_tray_rect_with_size(position, p_tray_size, p_rect);
}

unsafe fn tw_is_pos_horizontal(this: *mut CTrayWindow) -> BOOL {
    ((*this).m_Position == ABE_TOP || (*this).m_Position == ABE_BOTTOM) as BOOL
}

unsafe fn tw_calculate_valid_size(this: *mut CTrayWindow, position: DWORD, p_rect: *mut RECT) -> HMONITOR {
    let mut sz_wnd = SIZE {
        cx: (*p_rect).right - (*p_rect).left,
        cy: (*p_rect).bottom - (*p_rect).top,
    };
    let mut rc_screen = *p_rect;
    let h_mon = tw_get_screen_rect_from_rect(&mut rc_screen, MONITOR_DEFAULTTONEAREST);

    let sz_max = SIZE {
        cx: (rc_screen.right - rc_screen.left) / 2,
        cy: (rc_screen.bottom - rc_screen.top) / 2,
    };
    if sz_wnd.cx > sz_max.cx {
        sz_wnd.cx = sz_max.cx;
    }
    if sz_wnd.cy > sz_max.cy {
        sz_wnd.cy = sz_max.cy;
    }

    tw_get_tray_rect_from_screen_rect(this, position, &rc_screen, &sz_wnd, p_rect);
    h_mon
}

unsafe fn tw_get_dragging_rect_from_pt(
    this: *mut CTrayWindow,
    mut pt: POINT,
    p_rect: *mut RECT,
    ph_monitor: *mut HMONITOR,
) -> DWORD {
    let mut rc_screen: RECT = core::mem::zeroed();
    let mut screen_offset: SIZE = core::mem::zeroed();
    rc_screen.left = 0;
    rc_screen.top = 0;

    let mut h_mon = MonitorFromPoint(pt, MONITOR_DEFAULTTONULL);
    let mut got_primary = h_mon.is_null();
    if !h_mon.is_null() {
        let mut mi: MONITORINFO = core::mem::zeroed();
        mi.cbSize = size_of::<MONITORINFO>() as DWORD;
        if GetMonitorInfoW(h_mon, &mut mi) == 0 {
            h_mon = null_mut();
            got_primary = true;
        } else {
            pt.x -= mi.rcMonitor.left;
            pt.y -= mi.rcMonitor.top;
            screen_offset.cx = mi.rcMonitor.left;
            screen_offset.cy = mi.rcMonitor.top;
            rc_screen.right = mi.rcMonitor.right - mi.rcMonitor.left;
            rc_screen.bottom = mi.rcMonitor.bottom - mi.rcMonitor.top;
        }
    }
    if got_primary {
        screen_offset.cx = 0;
        screen_offset.cy = 0;
        rc_screen.right = GetSystemMetrics(SM_CXSCREEN as i32);
        rc_screen.bottom = GetSystemMetrics(SM_CYSCREEN as i32);
    }

    let mut delta_pt: SIZE = core::mem::zeroed();
    let pos_h: DWORD;
    let pos_v: DWORD;
    if pt.x < rc_screen.right / 2 {
        delta_pt.cx = pt.x;
        pos_h = ABE_LEFT;
    } else {
        delta_pt.cx = rc_screen.right - pt.x;
        pos_h = ABE_RIGHT;
    }
    if pt.y < rc_screen.bottom / 2 {
        delta_pt.cy = pt.y;
        pos_v = ABE_TOP;
    } else {
        delta_pt.cy = rc_screen.bottom - pt.y;
        pos_v = ABE_BOTTOM;
    }

    let pos = if delta_pt.cx * rc_screen.bottom < delta_pt.cy * rc_screen.right {
        pos_h
    } else {
        pos_v
    };

    OffsetRect(&mut rc_screen, screen_offset.cx, screen_offset.cy);

    let rc_pos = (*this).m_TrayRects[pos as usize];
    let h_mon_new = tw_get_monitor_from_rect(&rc_pos);
    if h_mon != h_mon_new {
        let sz_tray = SIZE {
            cx: rc_pos.right - rc_pos.left,
            cy: rc_pos.bottom - rc_pos.top,
        };
        tw_get_tray_rect_from_screen_rect(this, pos, &rc_screen, &sz_tray, p_rect);
        h_mon = h_mon_new;
    } else {
        *p_rect = rc_pos;
    }

    *ph_monitor = h_mon;
    pos
}

unsafe fn tw_get_dragging_rect_from_rect(
    this: *mut CTrayWindow,
    p_rect: *mut RECT,
    ph_monitor: *mut HMONITOR,
) -> DWORD {
    let pt = POINT {
        x: (*p_rect).left + ((*p_rect).right - (*p_rect).left) / 2,
        y: (*p_rect).top + ((*p_rect).bottom - (*p_rect).top) / 2,
    };
    tw_get_dragging_rect_from_pt(this, pt, p_rect, ph_monitor)
}

unsafe fn tw_changing_win_pos(this: *mut CTrayWindow, pwp: *mut WINDOWPOS) {
    let mut rc_tray: RECT = core::mem::zeroed();

    if (*this).get_flag(F_ISDRAGGING) {
        rc_tray.left = (*pwp).x;
        rc_tray.top = (*pwp).y;
        rc_tray.right = rc_tray.left + (*pwp).cx;
        rc_tray.bottom = rc_tray.top + (*pwp).cy;

        if EqualRect(&rc_tray, &(*this).m_TrayRects[(*this).m_DraggingPosition as usize]) == 0 {
            (*this).m_DraggingPosition =
                tw_get_dragging_rect_from_rect(this, &mut rc_tray, &mut (*this).m_DraggingMonitor);
        }

        (*this).m_Monitor = (*this).m_DraggingMonitor;
        (*this).m_Position = (*this).m_DraggingPosition;
        g_TaskbarSettings.sr.Position = (*this).m_Position;
        TaskbarSettings_save_global();
        (*this).set_flag(F_ISDRAGGING, false);

        (*this).m_TrayRects[(*this).m_Position as usize] = rc_tray;
        tw_changing_win_pos_apply(this, pwp, &mut rc_tray);
    } else if GetWindowRect((*this).m_hWnd, &mut rc_tray) != 0 {
        if (*this).get_flag(F_INSIZEMOVE) {
            if (*pwp).flags & SWP_NOMOVE == 0 {
                rc_tray.left = (*pwp).x;
                rc_tray.top = (*pwp).y;
            }
            if (*pwp).flags & SWP_NOSIZE == 0 {
                rc_tray.right = rc_tray.left + (*pwp).cx;
                rc_tray.bottom = rc_tray.top + (*pwp).cy;
            }

            (*this).m_Position = tw_get_dragging_rect_from_rect(this, &mut rc_tray, &mut (*this).m_Monitor);

            if (*pwp).flags & (SWP_NOMOVE | SWP_NOSIZE) == 0 {
                let sz_wnd = SIZE {
                    cx: (*pwp).cx,
                    cy: (*pwp).cy,
                };
                tw_make_tray_rect_with_size((*this).m_Position, &sz_wnd, &mut rc_tray);
            }

            (*this).m_TrayRects[(*this).m_Position as usize] = rc_tray;
        } else if (*this).m_Position != DWORD::MAX {
            rc_tray = (*this).m_TrayRects[(*this).m_Position as usize];
            if ab_is_auto_hide_state(this) != 0 {
                rc_tray.left += (*this).m_AutoHideOffset.cx;
                rc_tray.right += (*this).m_AutoHideOffset.cx;
                rc_tray.top += (*this).m_AutoHideOffset.cy;
                rc_tray.bottom += (*this).m_AutoHideOffset.cy;
            }
        }

        tw_changing_win_pos_apply(this, pwp, &mut rc_tray);
    }
}

/// The `ChangePos:` tail of ChangingWinPos (shared by both the dragging and the
/// GetWindowRect branches).
unsafe fn tw_changing_win_pos_apply(this: *mut CTrayWindow, pwp: *mut WINDOWPOS, rc_tray: *mut RECT) {
    (*this).m_TraySize.cx = (*rc_tray).right - (*rc_tray).left;
    (*this).m_TraySize.cy = (*rc_tray).bottom - (*rc_tray).top;

    (*pwp).flags &= !(SWP_NOMOVE | SWP_NOSIZE);
    (*pwp).x = (*rc_tray).left;
    (*pwp).y = (*rc_tray).top;
    (*pwp).cx = (*this).m_TraySize.cx;
    (*pwp).cy = (*this).m_TraySize.cy;
}

unsafe fn tw_apply_clipping(this: *mut CTrayWindow, clip: BOOL) {
    let mut rc_window: RECT = core::mem::zeroed();
    if GetWindowRect((*this).m_hWnd, &mut rc_window) == 0 {
        return;
    }

    let mut clip = clip;
    if GetSystemMetrics(SM_CMONITORS as i32) <= 1 {
        clip = FALSE as BOOL;
    }

    let h_clip_rgn: HRGN = if clip != 0 {
        let mut rc_clip = rc_window;
        tw_get_screen_rect((*this).m_Monitor, &mut rc_clip);
        let mut tmp: RECT = core::mem::zeroed();
        if IntersectRect(&mut tmp, &rc_clip, &rc_window) == 0 {
            rc_clip = rc_window;
        } else {
            rc_clip = tmp;
        }
        OffsetRect(&mut rc_clip, -rc_window.left, -rc_window.top);
        CreateRectRgnIndirect(&rc_clip)
    } else {
        null_mut()
    };

    SetWindowRgn((*this).m_hWnd, h_clip_rgn, TRUE as BOOL);
}

unsafe fn tw_resize_work_area(this: *mut CTrayWindow) {
    let mut rc_work_area: RECT = core::mem::zeroed();

    if (*this).m_PreviousMonitor != (*this).m_Monitor {
        tw_get_screen_rect((*this).m_PreviousMonitor, &mut rc_work_area);
        SystemParametersInfoW(
            SPI_SETWORKAREA,
            1,
            &mut rc_work_area as *mut RECT as *mut c_void,
            SPIF_SENDCHANGE,
        );
    }

    let rc_tray = (*this).m_TrayRects[(*this).m_Position as usize];
    tw_get_screen_rect((*this).m_Monitor, &mut rc_work_area);
    (*this).m_PreviousMonitor = (*this).m_Monitor;

    if ab_is_auto_hide_state(this) == 0 {
        match (*this).m_Position {
            ABE_TOP => rc_work_area.top = rc_tray.bottom,
            ABE_LEFT => rc_work_area.left = rc_tray.right,
            ABE_RIGHT => rc_work_area.right = rc_tray.left,
            ABE_BOTTOM => rc_work_area.bottom = rc_tray.top,
            _ => {}
        }
    }

    SystemParametersInfoW(
        SPI_SETWORKAREA,
        1,
        &mut rc_work_area as *mut RECT as *mut c_void,
        SPIF_SENDCHANGE,
    );
}

unsafe fn tw_check_tray_wnd_position(this: *mut CTrayWindow) {
    IUnknown_Exec(
        (*this).m_TrayBandSite as *mut IUnknown,
        &IID_IDeskBand,
        DBID_BANDINFOCHANGED,
        0,
        null_mut(),
        null_mut(),
    );
    tw_fit_to_rebar(this, &mut (*this).m_TrayRects[(*this).m_Position as usize]);
    SetWindowPos((*this).m_hWnd, null_mut(), 0, 0, 0, 0, SWP_NOZORDER | SWP_NOACTIVATE);
    tw_resize_work_area(this);
    tw_apply_clipping(this, TRUE as BOOL);
    RedrawWindow(
        (*this).m_hWnd,
        null(),
        null_mut(),
        RDW_ERASE | RDW_FRAME | RDW_INTERNALPAINT | RDW_INVALIDATE | RDW_ALLCHILDREN,
    );
}

unsafe fn tw_reg_load_settings(this: *mut CTrayWindow) {
    let start_btn_size = sb_get_size(addr_of_mut!((*this).m_StartButton));

    let edge_size = SIZE {
        cx: GetSystemMetrics(SM_CXEDGE as i32),
        cy: GetSystemMetrics(SM_CYEDGE as i32),
    };
    let dlg_frame_size = SIZE {
        cx: GetSystemMetrics(SM_CXDLGFRAME as i32),
        cy: GetSystemMetrics(SM_CYDLGFRAME as i32),
    };

    (*this).m_Position = g_TaskbarSettings.sr.Position;
    let mut rc_screen = g_TaskbarSettings.sr.Rect;
    tw_get_screen_rect_from_rect(&mut rc_screen, MONITOR_DEFAULTTONEAREST);

    if g_TaskbarSettings.sr.Size.cx == 0 || g_TaskbarSettings.sr.Size.cy == 0 {
        if (*this).m_Theme.is_null() {
            g_TaskbarSettings.sr.Size.cx = start_btn_size.cx + 2 * (edge_size.cx + dlg_frame_size.cx);
            g_TaskbarSettings.sr.Size.cy = start_btn_size.cy + 2 * (edge_size.cy + dlg_frame_size.cy);
        } else {
            g_TaskbarSettings.sr.Size.cx = start_btn_size.cx - edge_size.cx;
            g_TaskbarSettings.sr.Size.cy = start_btn_size.cy - edge_size.cy;
            if g_TaskbarSettings.bLock == 0 {
                g_TaskbarSettings.sr.Size.cy += GetSystemMetrics(SM_CYSIZEFRAME as i32);
            }
        }
    }

    let mut wnd_size: SIZE;
    if (*this).m_Theme.is_null() {
        wnd_size = SIZE {
            cx: 2 * (edge_size.cx + dlg_frame_size.cx),
            cy: start_btn_size.cy + 2 * (edge_size.cy + dlg_frame_size.cy),
        };
    } else {
        wnd_size = SIZE {
            cx: start_btn_size.cx,
            cy: start_btn_size.cy - edge_size.cy,
        };
    }

    if wnd_size.cx < g_TaskbarSettings.sr.Size.cx {
        wnd_size.cx = g_TaskbarSettings.sr.Size.cx;
    }
    if wnd_size.cy < g_TaskbarSettings.sr.Size.cy {
        wnd_size.cy = g_TaskbarSettings.sr.Size.cy;
    }

    (*this).m_TraySize = wnd_size;

    let mut pos = ABE_LEFT;
    while pos <= ABE_BOTTOM {
        tw_get_tray_rect_from_screen_rect(
            this,
            pos,
            &rc_screen,
            &(*this).m_TraySize,
            &mut (*this).m_TrayRects[pos as usize],
        );
        pos += 1;
    }

    (*this).m_Monitor = tw_get_monitor_from_rect(&(*this).m_TrayRects[ABE_LEFT as usize]);
}

unsafe fn tw_align_controls(this: *mut CTrayWindow, prc_client: *const RECT) {
    let mut rc_client: RECT = core::mem::zeroed();
    let mut pt_tray_notify = POINT { x: 0, y: 0 };

    sb_update_size(addr_of_mut!((*this).m_StartButton));
    if !prc_client.is_null() {
        rc_client = *prc_client;
    } else if GetClientRect((*this).m_hWnd, &mut rc_client) == 0 {
        return;
    }

    let horizontal = tw_is_pos_horizontal(this);

    let mut dwp = BeginDeferWindowPos(4);
    if dwp.is_null() {
        return;
    }

    let mut start_size = sb_get_size(addr_of_mut!((*this).m_StartButton));
    if start_size.cx > rc_client.right {
        start_size.cx = rc_client.right;
    }

    let hwnd_task_toolbar = GetWindow((*this).m_TaskSwitch, GW_CHILD);
    if !hwnd_task_toolbar.is_null() {
        let size = SendMessageW(hwnd_task_toolbar, TB_GETBUTTONSIZE, 0, 0);
        start_size.cy = HIWORD(size as usize) as i32
            + if !(*this).m_Theme.is_null() {
                GetSystemMetrics(SM_CYEDGE as i32)
            } else {
                0
            };
    }

    if !(*this).m_StartButton.m_hWnd.is_null() {
        dwp = DeferWindowPos(
            dwp,
            (*this).m_StartButton.m_hWnd,
            null_mut(),
            0,
            0,
            start_size.cx,
            start_size.cy,
            SWP_NOZORDER | SWP_NOACTIVATE,
        );
        if dwp.is_null() {
            return;
        }
    }

    let mut tray_size: SIZE = if horizontal != 0 {
        SIZE { cx: 0, cy: rc_client.bottom }
    } else {
        SIZE { cx: rc_client.right, cy: 0 }
    };

    if !(*this).m_TrayNotify.is_null()
        && SendMessageW(
            (*this).m_TrayNotify,
            TNWM_GETMINIMUMSIZE,
            horizontal as WPARAM,
            &mut tray_size as *mut SIZE as LPARAM,
        ) != 0
    {
        if horizontal != 0 {
            pt_tray_notify.x = rc_client.right - tray_size.cx;
        } else {
            pt_tray_notify.y = rc_client.bottom - tray_size.cy;
        }

        dwp = DeferWindowPos(
            dwp,
            (*this).m_TrayNotify,
            null_mut(),
            pt_tray_notify.x,
            pt_tray_notify.y,
            tray_size.cx,
            tray_size.cy,
            SWP_NOZORDER | SWP_NOACTIVATE,
        );
        if dwp.is_null() {
            return;
        }
    }

    if !(*this).m_Rebar.is_null() {
        let mut pt_rebar = POINT { x: 0, y: 0 };
        let sz_rebar: SIZE;

        set_window_style((*this).m_Rebar, CCS_VERT, if horizontal != 0 { 0 } else { CCS_VERT });

        if horizontal != 0 {
            pt_rebar.x = start_size.cx + GetSystemMetrics(SM_CXSIZEFRAME as i32);
            sz_rebar = SIZE {
                cx: pt_tray_notify.x - pt_rebar.x,
                cy: rc_client.bottom,
            };
        } else {
            pt_rebar.y = start_size.cy + GetSystemMetrics(SM_CYSIZEFRAME as i32);
            sz_rebar = SIZE {
                cx: rc_client.right,
                cy: pt_tray_notify.y - pt_rebar.y,
            };
        }

        dwp = DeferWindowPos(
            dwp,
            (*this).m_Rebar,
            null_mut(),
            pt_rebar.x,
            pt_rebar.y,
            sz_rebar.cx,
            sz_rebar.cy,
            SWP_NOZORDER | SWP_NOACTIVATE,
        );
    }

    if !dwp.is_null() {
        EndDeferWindowPos(dwp);
    }

    if !(*this).m_TaskSwitch.is_null() {
        SendMessageW((*this).m_TaskSwitch, TSWM_UPDATETASKBARPOS, 0, 0);
    }
}

unsafe fn tw_fit_to_rebar(this: *mut CTrayWindow, p_rect: *mut RECT) {
    let mut rebar_rect: RECT = core::mem::zeroed();
    let mut taskbar_rect: RECT = core::mem::zeroed();
    let mut client_rect: RECT = core::mem::zeroed();
    GetWindowRect((*this).m_Rebar, &mut rebar_rect);
    GetWindowRect((*this).m_hWnd, &mut taskbar_rect);
    GetClientRect((*this).m_hWnd, &mut client_rect);
    OffsetRect(&mut rebar_rect, -taskbar_rect.left, -taskbar_rect.top);

    let margins = SIZE {
        cx: taskbar_rect.right - taskbar_rect.left - client_rect.right + client_rect.left,
        cy: taskbar_rect.bottom - taskbar_rect.top - client_rect.bottom + client_rect.top,
    };

    match (*this).m_Position {
        ABE_TOP => {
            rebar_rect.bottom = rebar_rect.top + (*p_rect).bottom - (*p_rect).top - margins.cy;
            SendMessageW((*this).m_Rebar, RB_SIZETORECT, RBSTR_CHANGERECT, &mut rebar_rect as *mut RECT as LPARAM);
            (*p_rect).bottom = (*p_rect).top + rebar_rect.bottom - rebar_rect.top + margins.cy;
        }
        ABE_BOTTOM => {
            rebar_rect.top = rebar_rect.bottom - ((*p_rect).bottom - (*p_rect).top - margins.cy);
            SendMessageW((*this).m_Rebar, RB_SIZETORECT, RBSTR_CHANGERECT, &mut rebar_rect as *mut RECT as LPARAM);
            (*p_rect).top = (*p_rect).bottom - (rebar_rect.bottom - rebar_rect.top + margins.cy);
        }
        ABE_LEFT => {
            rebar_rect.right = rebar_rect.left + ((*p_rect).right - (*p_rect).left - margins.cx);
            SendMessageW((*this).m_Rebar, RB_SIZETORECT, RBSTR_CHANGERECT, &mut rebar_rect as *mut RECT as LPARAM);
            (*p_rect).right = (*p_rect).left + (rebar_rect.right - rebar_rect.left + margins.cx);
        }
        ABE_RIGHT => {
            rebar_rect.left = rebar_rect.right - ((*p_rect).right - (*p_rect).left - margins.cx);
            SendMessageW((*this).m_Rebar, RB_SIZETORECT, RBSTR_CHANGERECT, &mut rebar_rect as *mut RECT as LPARAM);
            (*p_rect).left = (*p_rect).right - (rebar_rect.right - rebar_rect.left + margins.cx);
        }
        _ => {}
    }

    tw_calculate_valid_size(this, (*this).m_Position, p_rect);
}

unsafe fn tw_popup_start_menu(this: *mut CTrayWindow) {
    if (*this).m_StartMenuPopup.is_null() {
        return;
    }
    let mut rc_exclude: RECT = core::mem::zeroed();
    let mut dw_flags: DWORD = 0;

    if GetWindowRect((*this).m_StartButton.m_hWnd, &mut rc_exclude) != 0 {
        // POINTL pt reuses rc_exclude corners.
        let mut pt = POINTL { x: 0, y: 0 };
        match (*this).m_Position {
            ABE_BOTTOM => {
                pt.x = rc_exclude.left;
                pt.y = rc_exclude.top;
                dw_flags |= MPPF_TOP;
            }
            ABE_TOP => {
                pt.x = rc_exclude.left;
                pt.y = rc_exclude.bottom;
                dw_flags |= MPPF_BOTTOM;
            }
            ABE_LEFT => {
                pt.x = rc_exclude.right;
                pt.y = rc_exclude.top;
                dw_flags |= MPPF_RIGHT;
            }
            ABE_RIGHT => {
                pt.x = rc_exclude.left;
                pt.y = rc_exclude.top;
                dw_flags |= MPPF_LEFT;
            }
            _ => {}
        }

        (*(*this).m_StartMenuPopup).Popup(
            &mut pt as *mut POINTL as *mut c_void,
            &mut rc_exclude as *mut RECT as *mut c_void,
            dw_flags,
        );

        SendMessageW((*this).m_StartButton.m_hWnd, BM_SETSTATE, TRUE as WPARAM, 0);
    }
}

unsafe fn tw_process_mouse_tracking(this: *mut CTrayWindow) {
    let mut pt: POINT = core::mem::zeroed();
    GetCursorPos(&mut pt);

    let mut rc_current: RECT = core::mem::zeroed();
    GetWindowRect((*this).m_hWnd, &mut rc_current);

    let mut over = PtInRect(&rc_current, pt) != 0;
    if SendMessageW((*this).m_StartButton.m_hWnd, BM_GETSTATE, 0, 0) != BST_UNCHECKED as LRESULT {
        over = true;
    }

    let state = (*this).m_AutoHideState;
    if over {
        if state == AUTOHIDE_HIDING {
            (*this).m_AutoHideState = AUTOHIDE_SHOWING;
            SetTimer((*this).m_hWnd, TIMER_ID_AUTOHIDE, AUTOHIDE_INTERVAL_ANIMATING, None);
        } else if state == AUTOHIDE_HIDDEN {
            (*this).m_AutoHideState = AUTOHIDE_SHOWING;
            SetTimer((*this).m_hWnd, TIMER_ID_AUTOHIDE, AUTOHIDE_DELAY_SHOW, None);
        }
    } else {
        if state == AUTOHIDE_SHOWING {
            (*this).m_AutoHideState = AUTOHIDE_HIDING;
            SetTimer((*this).m_hWnd, TIMER_ID_AUTOHIDE, AUTOHIDE_INTERVAL_ANIMATING, None);
        } else if state == AUTOHIDE_SHOWN {
            (*this).m_AutoHideState = AUTOHIDE_HIDING;
            SetTimer((*this).m_hWnd, TIMER_ID_AUTOHIDE, AUTOHIDE_DELAY_HIDE, None);
        }
        KillTimer((*this).m_hWnd, TIMER_ID_MOUSETRACK);
    }
}

unsafe fn tw_process_auto_hide(this: *mut CTrayWindow) {
    let w = (*this).m_TraySize.cx - GetSystemMetrics(SM_CXSIZEFRAME as i32);
    let h = (*this).m_TraySize.cy - GetSystemMetrics(SM_CYSIZEFRAME as i32);

    // Emulate the C++ switch with the two `fallthrough` cases.
    let mut state = (*this).m_AutoHideState;

    if state == AUTOHIDE_HIDING {
        match (*this).m_Position {
            ABE_LEFT => {
                (*this).m_AutoHideOffset.cy = 0;
                (*this).m_AutoHideOffset.cx -= AUTOHIDE_SPEED_HIDE;
                if (*this).m_AutoHideOffset.cx < -w {
                    (*this).m_AutoHideOffset.cx = w;
                }
            }
            ABE_TOP => {
                (*this).m_AutoHideOffset.cx = 0;
                (*this).m_AutoHideOffset.cy -= AUTOHIDE_SPEED_HIDE;
                if (*this).m_AutoHideOffset.cy < -h {
                    (*this).m_AutoHideOffset.cy = h;
                }
            }
            ABE_RIGHT => {
                (*this).m_AutoHideOffset.cy = 0;
                (*this).m_AutoHideOffset.cx += AUTOHIDE_SPEED_HIDE;
                if (*this).m_AutoHideOffset.cx > w {
                    (*this).m_AutoHideOffset.cx = w;
                }
            }
            ABE_BOTTOM => {
                (*this).m_AutoHideOffset.cx = 0;
                (*this).m_AutoHideOffset.cy += AUTOHIDE_SPEED_HIDE;
                if (*this).m_AutoHideOffset.cy > h {
                    (*this).m_AutoHideOffset.cy = h;
                }
            }
            _ => {}
        }

        if (*this).m_AutoHideOffset.cx != w && (*this).m_AutoHideOffset.cy != h {
            SetTimer((*this).m_hWnd, TIMER_ID_AUTOHIDE, AUTOHIDE_INTERVAL_ANIMATING, None);
            SetWindowPos((*this).m_hWnd, null_mut(), 0, 0, 0, 0, SWP_NOACTIVATE | SWP_NOZORDER);
            return;
        }
        state = AUTOHIDE_HIDDEN; // fallthrough
    }

    if state == AUTOHIDE_HIDDEN {
        match (*this).m_Position {
            ABE_LEFT => {
                (*this).m_AutoHideOffset.cx = -w;
                (*this).m_AutoHideOffset.cy = 0;
            }
            ABE_TOP => {
                (*this).m_AutoHideOffset.cx = 0;
                (*this).m_AutoHideOffset.cy = -h;
            }
            ABE_RIGHT => {
                (*this).m_AutoHideOffset.cx = w;
                (*this).m_AutoHideOffset.cy = 0;
            }
            ABE_BOTTOM => {
                (*this).m_AutoHideOffset.cx = 0;
                (*this).m_AutoHideOffset.cy = h;
            }
            _ => {}
        }
        KillTimer((*this).m_hWnd, TIMER_ID_AUTOHIDE);
        (*this).m_AutoHideState = AUTOHIDE_HIDDEN;
        SetWindowPos((*this).m_hWnd, null_mut(), 0, 0, 0, 0, SWP_NOACTIVATE | SWP_NOZORDER);
        return;
    }

    if state == AUTOHIDE_SHOWING {
        if (*this).m_AutoHideOffset.cx >= AUTOHIDE_SPEED_SHOW {
            (*this).m_AutoHideOffset.cx -= AUTOHIDE_SPEED_SHOW;
        } else if (*this).m_AutoHideOffset.cx <= -AUTOHIDE_SPEED_SHOW {
            (*this).m_AutoHideOffset.cx += AUTOHIDE_SPEED_SHOW;
        } else {
            (*this).m_AutoHideOffset.cx = 0;
        }

        if (*this).m_AutoHideOffset.cy >= AUTOHIDE_SPEED_SHOW {
            (*this).m_AutoHideOffset.cy -= AUTOHIDE_SPEED_SHOW;
        } else if (*this).m_AutoHideOffset.cy <= -AUTOHIDE_SPEED_SHOW {
            (*this).m_AutoHideOffset.cy += AUTOHIDE_SPEED_SHOW;
        } else {
            (*this).m_AutoHideOffset.cy = 0;
        }

        if (*this).m_AutoHideOffset.cx != 0 || (*this).m_AutoHideOffset.cy != 0 {
            SetTimer((*this).m_hWnd, TIMER_ID_AUTOHIDE, AUTOHIDE_INTERVAL_ANIMATING, None);
            SetWindowPos((*this).m_hWnd, null_mut(), 0, 0, 0, 0, SWP_NOACTIVATE | SWP_NOZORDER);
            return;
        }
        // fallthrough to AUTOHIDE_SHOWN
    }

    // AUTOHIDE_SHOWN (reached directly or via SHOWING fallthrough)
    KillTimer((*this).m_hWnd, TIMER_ID_AUTOHIDE);
    (*this).m_AutoHideState = AUTOHIDE_SHOWN;
    SetWindowPos((*this).m_hWnd, null_mut(), 0, 0, 0, 0, SWP_NOACTIVATE | SWP_NOZORDER);
}

// ============================================================================
// CTrayWindow — drawing
// ============================================================================

unsafe fn tw_erase_background_with_theme(this: *mut CTrayWindow, hdc: HDC) -> LRESULT {
    let i_sbkgnd_part = [
        TBP_BACKGROUNDLEFT,
        TBP_BACKGROUNDTOP,
        TBP_BACKGROUNDRIGHT,
        TBP_BACKGROUNDBOTTOM,
    ];
    if !(*this).m_Theme.is_null() {
        let mut rect: RECT = core::mem::zeroed();
        GetClientRect((*this).m_hWnd, &mut rect);
        DrawThemeBackground(
            (*this).m_Theme,
            hdc,
            i_sbkgnd_part[(*this).m_Position as usize],
            0,
            &rect,
            null(),
        );
    }
    0
}

unsafe fn tw_draw_sizer_with_theme(this: *mut CTrayWindow, _h_rgn: HRGN) -> i32 {
    let i_sizer_part = [
        TBP_SIZINGBARLEFT,
        TBP_SIZINGBARTOP,
        TBP_SIZINGBARRIGHT,
        TBP_SIZINGBARBOTTOM,
    ];
    let mut size: SIZE = core::mem::zeroed();

    let hr = GetThemePartSize(
        (*this).m_Theme,
        null_mut(),
        i_sizer_part[(*this).m_Position as usize],
        0,
        null(),
        TS_TRUE,
        &mut size,
    );
    if FAILED_UNEXPECTEDLY(hr) != 0 {
        return 0;
    }

    let mut rect: RECT = core::mem::zeroed();
    GetWindowRect((*this).m_hWnd, &mut rect);
    OffsetRect(&mut rect, -rect.left, -rect.top);

    let hdc = GetWindowDC((*this).m_hWnd);

    match (*this).m_Position {
        ABE_LEFT => rect.left = rect.right - size.cx,
        ABE_TOP => rect.top = rect.bottom - size.cy,
        ABE_RIGHT => rect.right = rect.left + size.cx,
        _ => rect.bottom = rect.top + size.cy,
    }

    DrawThemeBackground(
        (*this).m_Theme,
        hdc,
        i_sizer_part[(*this).m_Position as usize],
        0,
        &rect,
        null(),
    );

    ReleaseDC((*this).m_hWnd, hdc);
    0
}

unsafe fn tw_draw_show_desktop_button(this: *mut CTrayWindow) {
    if (*this).m_pShowDesktopButton.is_null()
        || TrayShowDesktopButton_IsWindow((*this).m_pShowDesktopButton) == 0
    {
        return;
    }
    RedrawWindow(
        (*this).m_TrayNotify,
        null(),
        null_mut(),
        RDW_INVALIDATE | RDW_ERASENOW | RDW_UPDATENOW,
    );
}

// ============================================================================
// CTrayWindow — ITrayWindow methods + window creation
// ============================================================================

/// CWindowImpl::Create for the tray window itself (DECLARE_WND_CLASS_EX
/// Shell_TrayWnd / CS_DBLCLKS / COLOR_3DFACE). Registers the class + creates
/// the top-level popup, stashing `this` for the WndProc.
unsafe fn tw_create_window(
    this: *mut CTrayWindow,
    rc: RECT,
    dw_style: DWORD,
    dw_ex_style: DWORD,
) -> HWND {
    register_class_once(
        hExplorerInstance,
        szTrayWindowClass.as_ptr(),
        Some(CTrayWindow_WindowProc),
        CS_DBLCLKS,
        COLOR_3DFACE as i32,
    );

    (*this).m_hWnd = CreateWindowExW(
        dw_ex_style,
        szTrayWindowClass.as_ptr(),
        null(),
        dw_style,
        rc.left,
        rc.top,
        rc.right - rc.left,
        rc.bottom - rc.top,
        null_mut(),
        null_mut(),
        hExplorerInstance,
        this as LPVOID,
    );
    (*this).m_hWnd
}

unsafe fn tw_open(this: *mut CTrayWindow) -> HRESULT {
    if !(*this).m_hWnd.is_null() && IsWindow((*this).m_hWnd) != 0 {
        return S_OK;
    }

    let mut dw_ex_style: DWORD = WS_EX_TOOLWINDOW | WS_EX_WINDOWEDGE;
    if (*&raw const g_TaskbarSettings).sr.AlwaysOnTop() != 0 {
        dw_ex_style |= WS_EX_TOPMOST;
    }

    let mut dw_style: DWORD = WS_POPUP | WS_VISIBLE | WS_CLIPSIBLINGS | WS_CLIPCHILDREN;
    if (*this).m_Theme.is_null() {
        dw_style |= WS_THICKFRAME | WS_BORDER;
    }

    let mut rc_wnd: RECT = core::mem::zeroed();
    if (*this).m_Position != DWORD::MAX {
        rc_wnd = (*this).m_TrayRects[(*this).m_Position as usize];
    }

    if tw_create_window(this, rc_wnd, dw_style, dw_ex_style).is_null() {
        return E_FAIL;
    }

    tw_align_controls(this, null());
    tw_check_tray_wnd_position(this);
    S_OK
}

unsafe fn tw_close(this: *mut CTrayWindow) -> HRESULT {
    if !(*this).m_hWnd.is_null() {
        SendMessageW((*this).m_hWnd, WM_APP_TRAYDESTROY, 0, 0);
    }
    S_OK
}

unsafe fn tw_is_special_hwnd(this: *mut CTrayWindow, h_wnd: HWND) -> BOOL {
    ((*this).m_hWnd == h_wnd
        || (!(*this).m_DesktopWnd.is_null() && (*this).m_hWnd == (*this).m_DesktopWnd)) as BOOL
}

unsafe fn tw_lock(this: *mut CTrayWindow, b_lock: BOOL) -> BOOL {
    let b_prev_lock = g_TaskbarSettings.bLock;

    if g_TaskbarSettings.bLock != b_lock {
        g_TaskbarSettings.bLock = b_lock;

        if !(*this).m_TrayBandSite.is_null() {
            let hr = ((*(*(*this).m_TrayBandSite).lpVtbl).Lock)((*this).m_TrayBandSite, b_lock);
            if hr < 0 {
                g_TaskbarSettings.bLock = b_prev_lock;
                return b_prev_lock;
            }
        }

        if !(*this).m_Theme.is_null() {
            let mut pos = ABE_LEFT;
            while pos <= ABE_BOTTOM {
                let mut rc_gripper: RECT = core::mem::zeroed();
                tw_adjust_sizer_rect(this, &mut rc_gripper, pos);
                if g_TaskbarSettings.bLock != 0 {
                    (*this).m_TrayRects[pos as usize].top += rc_gripper.top;
                    (*this).m_TrayRects[pos as usize].left += rc_gripper.left;
                    (*this).m_TrayRects[pos as usize].bottom += rc_gripper.bottom;
                    (*this).m_TrayRects[pos as usize].right += rc_gripper.right;
                } else {
                    (*this).m_TrayRects[pos as usize].top -= rc_gripper.top;
                    (*this).m_TrayRects[pos as usize].left -= rc_gripper.left;
                    (*this).m_TrayRects[pos as usize].bottom -= rc_gripper.bottom;
                    (*this).m_TrayRects[pos as usize].right -= rc_gripper.right;
                }
                pos += 1;
            }
        }
        SetWindowPos(
            (*this).m_hWnd,
            null_mut(),
            0,
            0,
            0,
            0,
            SWP_FRAMECHANGED | SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOZORDER | SWP_NOOWNERZORDER,
        );
        tw_resize_work_area(this);
        tw_apply_clipping(this, TRUE as BOOL);
    }

    b_prev_lock
}

unsafe fn tw_is_task_wnd(this: *mut CTrayWindow, h_wnd: HWND) -> BOOL {
    if IsWindow(h_wnd) != 0 && IsWindowVisible(h_wnd) != 0 && tw_is_special_hwnd(this, h_wnd) == 0 {
        let ex_style = get_window_long_ptr(h_wnd, GWL_EXSTYLE) as DWORD;
        if (ex_style & WS_EX_APPWINDOW != 0 || GetWindow(h_wnd, GW_OWNER).is_null())
            && ex_style & WS_EX_TOOLWINDOW == 0
        {
            return TRUE as BOOL;
        }
    }
    FALSE as BOOL
}

unsafe fn tw_raise_start_button(this: *mut CTrayWindow) -> HRESULT {
    SendMessageW((*this).m_StartButton.m_hWnd, BM_SETSTATE, FALSE as WPARAM, 0);
    S_OK
}

// --- minimize / restore / show-desktop (file-scope + methods) ---

#[repr(C)]
struct MINIMIZE_INFO {
    hwndDesktop: HWND,
    hTrayWnd: HWND,
    hwndProgman: HWND,
    bShowDesktop: BOOL,
}

unsafe fn is_dialog(hwnd: HWND) -> BOOL {
    let mut sz_class: [u16; 32] = [0; 32];
    GetClassNameW(hwnd, sz_class.as_mut_ptr(), sz_class.len() as i32);
    (wcscmp_local(sz_class.as_ptr(), w!("#32770").as_ptr()) == 0) as BOOL
}

unsafe fn minimize_windows_proc_impl(hwnd: HWND, l_param: LPARAM) -> BOOL {
    let info = l_param as *mut MINIMIZE_INFO;
    if hwnd == (*info).hwndDesktop || hwnd == (*info).hTrayWnd || hwnd == (*info).hwndProgman {
        return TRUE as BOOL;
    }

    if (*info).bShowDesktop == 0 {
        if IsWindowEnabled(hwnd) == 0 || is_dialog(hwnd) != 0 {
            return TRUE as BOOL;
        }
        let hwnd_owner = GetWindow(hwnd, GW_OWNER);
        if !hwnd_owner.is_null() && IsWindowEnabled(hwnd_owner) == 0 {
            return TRUE as BOOL;
        }
    }

    if CanBeMinimized(hwnd) != 0 {
        let mut mwp: MINWNDPOS = core::mem::zeroed();
        mwp.hwnd = hwnd;
        mwp.wndpl.length = size_of::<WINDOWPLACEMENT>() as UINT;
        if GetWindowPlacement(hwnd, &mut mwp.wndpl) != 0 && ShowWindowAsync(hwnd, SW_SHOWMINNOACTIVE as i32) != 0 {
            (*&raw mut g_MinimizedAll).add(mwp);
        }
    }
    TRUE as BOOL
}
wndenum_cb!(MinimizeWindowsProc, minimize_windows_proc_impl);

unsafe fn tw_minimize_all(this: *mut CTrayWindow, b_show_desktop: BOOL) {
    let mut info: MINIMIZE_INFO = core::mem::zeroed();
    info.hwndDesktop = GetDesktopWindow();
    info.hTrayWnd = FindWindowW(szTrayWindowClass.as_ptr(), null());
    info.hwndProgman = FindWindowW(w!("Progman").as_ptr(), null());
    info.bShowDesktop = b_show_desktop;
    EnumWindows(Some(MinimizeWindowsProc), &mut info as *mut MINIMIZE_INFO as LPARAM);

    SetForegroundWindow((*this).m_DesktopWnd);
    SetFocus((*this).m_DesktopWnd);
}

unsafe fn tw_show_desktop(this: *mut CTrayWindow) {
    tw_minimize_all(this, TRUE as BOOL);
}

unsafe fn tw_restore_all(_this: *mut CTrayWindow) {
    let mut i = (*&raw const g_MinimizedAll).get_size() - 1;
    while i >= 0 {
        let e = (*&raw const g_MinimizedAll).at(i);
        if IsWindowVisible(e.hwnd) != 0 && IsIconic(e.hwnd) != 0 {
            SetWindowPlacement(e.hwnd, &e.wndpl);
        }
        i -= 1;
    }
    (*&raw mut g_MinimizedAll).remove_all();
}

// --- CAppBarManager overrides: SetAutoHideState / UpdateAlwaysOnTop ---

#[inline]
unsafe fn sr_set_bit(mask: DWORD, v: BOOL) {
    if v != 0 {
        g_TaskbarSettings.sr.dwFlags |= mask;
    } else {
        g_TaskbarSettings.sr.dwFlags &= !mask;
    }
}

unsafe fn tw_set_auto_hide_state(this: *mut CTrayWindow, b_auto_hide: BOOL) {
    sr_set_bit(TW_STRUCKRECTS2::F_AUTOHIDE, b_auto_hide);
    (*this).m_AutoHideOffset = SIZE { cx: 0, cy: 0 };
    (*this).m_AutoHideState = AUTOHIDE_SHOWN;
    if b_auto_hide != 0 {
        SetTimer((*this).m_hWnd, TIMER_ID_MOUSETRACK, MOUSETRACK_INTERVAL, None);
    } else {
        SetWindowPos((*this).m_hWnd, null_mut(), 0, 0, 0, 0, SWP_NOACTIVATE | SWP_NOZORDER);
    }
}

unsafe fn tw_update_always_on_top(this: *mut CTrayWindow, b_always_on_top: BOOL) {
    sr_set_bit(TW_STRUCKRECTS2::F_ALWAYSONTOP, b_always_on_top);
    let hwnd_insert_after = if b_always_on_top != 0 { HWND_TOPMOST } else { HWND_NOTOPMOST };
    SetWindowPos(
        (*this).m_hWnd,
        hwnd_insert_after,
        0,
        0,
        0,
        0,
        SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE,
    );
}

// ============================================================================
// CTrayWindow — message handlers (MESSAGE_HANDLER bodies)
// ============================================================================

unsafe fn on_create(this: *mut CTrayWindow, b_handled: *mut BOOL) -> LRESULT {
    let _ = b_handled;
    OutputDebugStringA(b"[EXPLORER] taskbar: OnCreate begin\n\0".as_ptr() as *const i8);
    // ((ITrayWindow*)this)->AddRef();
    tw_addref(this);

    SetWindowTheme((*this).m_hWnd, w!("TaskBar").as_ptr(), null());

    sb_create(addr_of_mut!((*this).m_StartButton), (*this).m_hWnd);

    tw_reg_load_settings(this);

    let hbm_banner = LoadBitmapW(hExplorerInstance, IDB_STARTMENU as usize as LPCWSTR);
    (*this).m_StartMenuPopup = CreateStartMenu(
        addr_of_mut!((*this).lpVtbl_TrayWindow) as *mut ITrayWindow,
        &mut (*this).m_StartMenuBand,
        hbm_banner,
        (*&raw const g_TaskbarSettings).sr.SmSmallIcons() as BOOL,
    );

    // CTaskBand_CreateInstance(this, m_StartButton.m_hWnd, IID_PPV_ARG(IDeskBand, &m_TaskBand))
    let mut h_ret = CTaskBand_CreateInstance(
        addr_of_mut!((*this).lpVtbl_TrayWindow) as *mut ITrayWindow,
        (*this).m_StartButton.m_hWnd,
        &IID_IDeskBand,
        &mut (*this).m_TaskBand as *mut *mut IDeskBand as *mut *mut c_void,
    );
    if FAILED_UNEXPECTEDLY(h_ret) != 0 {
        return FALSE as LRESULT;
    }

    h_ret = CTrayBandSite_CreateInstance(
        addr_of_mut!((*this).lpVtbl_TrayWindow) as *mut ITrayWindow,
        (*this).m_TaskBand,
        &mut (*this).m_TrayBandSite,
    );
    if FAILED_UNEXPECTEDLY(h_ret) != 0 {
        return FALSE as LRESULT;
    }

    h_ret = CTrayNotifyWnd_CreateInstance(
        (*this).m_hWnd,
        &IID_IUnknown,
        &mut (*this).m_TrayNotifyInstance as *mut *mut IUnknown as *mut *mut c_void,
    );
    if FAILED_UNEXPECTEDLY(h_ret) != 0 {
        return FALSE as LRESULT;
    }

    OutputDebugStringA(b"[EXPLORER] taskbar: child controls created\n\0".as_ptr() as *const i8);

    h_ret = IUnknown_GetWindow((*this).m_TrayBandSite as *mut IUnknown, &mut (*this).m_Rebar);
    if FAILED_UNEXPECTEDLY(h_ret) != 0 {
        return FALSE as LRESULT;
    }
    h_ret = IUnknown_GetWindow((*this).m_TaskBand as *mut IUnknown, &mut (*this).m_TaskSwitch);
    if FAILED_UNEXPECTEDLY(h_ret) != 0 {
        return FALSE as LRESULT;
    }
    h_ret = IUnknown_GetWindow((*this).m_TrayNotifyInstance, &mut (*this).m_TrayNotify);
    if FAILED_UNEXPECTEDLY(h_ret) != 0 {
        return FALSE as LRESULT;
    }

    SendMessageW(
        (*this).m_TrayNotify,
        TNWM_GETSHOWDESKTOPBUTTON,
        &mut (*this).m_pShowDesktopButton as *mut *mut CTrayShowDesktopButton as WPARAM,
        0,
    );
    if (*this).m_pShowDesktopButton.is_null() {
        return FALSE as LRESULT;
    }

    SetWindowTheme((*this).m_Rebar, w!("TaskBar").as_ptr(), null());

    tw_update_fonts(this);

    InitShellServices(&mut (*this).m_ShellServices);

    if ab_is_auto_hide_state(this) != 0 {
        (*this).m_AutoHideState = AUTOHIDE_HIDING;
        SetTimer((*this).m_hWnd, TIMER_ID_AUTOHIDE, AUTOHIDE_DELAY_HIDE, None);
    }

    ((*(*(*this).m_TrayBandSite).lpVtbl).Lock)((*this).m_TrayBandSite, g_TaskbarSettings.bLock);

    // winkey hotkeys: MAKELONG(id, MAKEWORD(vk, mod))
    let winkeys: [u32; 12] = [
        make_hotkey(IDHK_RUN, b'R' as u32, MOD_WIN),
        make_hotkey(IDHK_MINIMIZE_ALL, b'M' as u32, MOD_WIN),
        make_hotkey(IDHK_RESTORE_ALL, b'M' as u32, MOD_WIN | MOD_SHIFT),
        make_hotkey(IDHK_HELP, VK_F1 as u32, MOD_WIN),
        make_hotkey(IDHK_EXPLORE, b'E' as u32, MOD_WIN),
        make_hotkey(IDHK_FIND, b'F' as u32, MOD_WIN),
        make_hotkey(IDHK_FIND_COMPUTER, b'F' as u32, MOD_WIN | MOD_CONTROL),
        make_hotkey(IDHK_NEXT_TASK, VK_TAB as u32, MOD_WIN),
        make_hotkey(IDHK_PREV_TASK, VK_TAB as u32, MOD_WIN | MOD_SHIFT),
        make_hotkey(IDHK_SYS_PROPERTIES, VK_PAUSE as u32, MOD_WIN),
        make_hotkey(IDHK_DESKTOP, b'D' as u32, MOD_WIN),
        make_hotkey(IDHK_PAGER, b'B' as u32, MOD_WIN),
    ];
    if SHRestricted(REST_NOWINKEYS) == 0 {
        for wk in winkeys.iter() {
            let modk = ((*wk >> 24) & 0xFF) as UINT; // HIBYTE(HIWORD)
            let key = ((*wk >> 16) & 0xFF) as UINT; // LOBYTE(HIWORD)
            RegisterHotKey((*this).m_hWnd, (*wk & 0xFFFF) as i32, modk, key);
        }
    }

    TRUE as LRESULT
}

#[inline]
fn make_hotkey(id: UINT, vk: u32, modk: u32) -> u32 {
    // MAKELONG(id, MAKEWORD(vk, mod))
    (id & 0xFFFF) | (((vk & 0xFF) | ((modk & 0xFF) << 8)) << 16)
}

unsafe fn on_theme_changed(this: *mut CTrayWindow) -> LRESULT {
    if !(*this).m_Theme.is_null() {
        CloseThemeData((*this).m_Theme);
    }
    (*this).m_Theme = OpenThemeData((*this).m_hWnd, w!("TaskBar").as_ptr());

    if !(*this).m_Theme.is_null() {
        set_window_style((*this).m_hWnd, WS_THICKFRAME | WS_BORDER, 0);
    } else {
        set_window_style((*this).m_hWnd, WS_THICKFRAME | WS_BORDER, WS_THICKFRAME | WS_BORDER);
    }
    SetWindowPos(
        (*this).m_hWnd,
        null_mut(),
        0,
        0,
        0,
        0,
        SWP_FRAMECHANGED | SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOOWNERZORDER,
    );
    TRUE as LRESULT
}

unsafe fn on_setting_changed(this: *mut CTrayWindow, u_msg: UINT, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    if w_param == SPI_SETNONCLIENTMETRICS as WPARAM {
        SendMessageW((*this).m_TrayNotify, u_msg, w_param, l_param);
        SendMessageW((*this).m_TaskSwitch, u_msg, w_param, l_param);
        tw_update_fonts(this);
        tw_align_controls(this, null());
        tw_check_tray_wnd_position(this);
    }

    if !(*this).m_DesktopWnd.is_null() {
        SendMessageW((*this).m_DesktopWnd, u_msg, w_param, l_param);
    }

    if !(*this).m_StartMenuPopup.is_null()
        && lstrcmpi_local(l_param as LPCWSTR, w!("TraySettings").as_ptr()) == 0
    {
        tw_hide_start_menu(this);
        let hbm_banner = LoadBitmapW(hExplorerInstance, IDB_STARTMENU as usize as LPCWSTR);
        // FIXME: Please re-use the start menu (re-create it).
        if !(*this).m_StartMenuBand.is_null() {
            (*(*this).m_StartMenuBand).Release();
            (*this).m_StartMenuBand = null_mut();
        }
        (*this).m_StartMenuPopup = CreateStartMenu(
            addr_of_mut!((*this).lpVtbl_TrayWindow) as *mut ITrayWindow,
            &mut (*this).m_StartMenuBand,
            hbm_banner,
            (*&raw const g_TaskbarSettings).sr.SmSmallIcons() as BOOL,
        );
    }

    0
}

#[inline]
unsafe fn lstrcmpi_local(a: LPCWSTR, b: LPCWSTR) -> i32 {
    // Case-insensitive ASCII compare is enough for the L"TraySettings" literal.
    if a.is_null() {
        return -1;
    }
    let mut i = 0isize;
    loop {
        let mut ca = *a.offset(i);
        let mut cb = *b.offset(i);
        if (b'A' as u16..=b'Z' as u16).contains(&ca) {
            ca += 32;
        }
        if (b'A' as u16..=b'Z' as u16).contains(&cb) {
            cb += 32;
        }
        if ca != cb {
            return ca as i32 - cb as i32;
        }
        if ca == 0 {
            return 0;
        }
        i += 1;
    }
}

unsafe fn on_erase_background(this: *mut CTrayWindow, w_param: WPARAM, b_handled: *mut BOOL) -> LRESULT {
    let hdc = w_param as HDC;
    if (*this).m_Theme.is_null() {
        *b_handled = FALSE as BOOL;
        return 0;
    }
    tw_erase_background_with_theme(this, hdc)
}

unsafe fn on_display_change(this: *mut CTrayWindow) -> LRESULT {
    ab_recompute_all_workareas(this);
    tw_reg_load_settings(this);
    tw_check_tray_wnd_position(this);
    TRUE as LRESULT
}

unsafe fn on_copy_data(this: *mut CTrayWindow, u_msg: UINT, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    let p_copy_data = l_param as PCOPYDATASTRUCT;
    if p_copy_data.is_null() {
        return FALSE as LRESULT;
    }
    match (*p_copy_data).dwData as usize {
        TABDMC_APPBAR => ab_on_app_bar_message(this, p_copy_data),
        TABDMC_NOTIFY | TABDMC_LOADINPROC => {
            SendMessageW((*this).m_TrayNotify, u_msg, w_param, l_param)
        }
        _ => FALSE as LRESULT,
    }
}

unsafe fn on_nc_paint(this: *mut CTrayWindow, u_msg: UINT, w_param: WPARAM, l_param: LPARAM, b_handled: *mut BOOL) -> LRESULT {
    DefWindowProcW((*this).m_hWnd, u_msg, w_param, l_param);
    *b_handled = TRUE as BOOL;

    if (*this).m_Theme.is_null() || g_TaskbarSettings.bLock != 0 {
        tw_draw_show_desktop_button(this);
        return 0;
    }
    tw_draw_sizer_with_theme(this, w_param as HRGN);
    tw_draw_show_desktop_button(this);
    0
}

unsafe fn on_ctl_color_btn(w_param: WPARAM) -> LRESULT {
    SetBkMode(w_param as HDC, TRANSPARENT as i32);
    GetStockObject(HOLLOW_BRUSH as i32) as LRESULT
}

unsafe fn on_sys_color_change(this: *mut CTrayWindow, u_msg: UINT, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    SendMessageW((*this).m_Rebar, u_msg, w_param, l_param)
}

unsafe fn on_nc_hit_test(this: *mut CTrayWindow, l_param: LPARAM) -> LRESULT {
    let mut rc_client: RECT = core::mem::zeroed();

    if g_TaskbarSettings.bLock != 0 {
        return HTBORDER as LRESULT;
    }

    SetLastError(ERROR_SUCCESS);
    if GetClientRect((*this).m_hWnd, &mut rc_client) != 0
        && (MapWindowPoints((*this).m_hWnd, null_mut(), &mut rc_client as *mut RECT as *mut POINT, 2) != 0
            || GetLastError() == ERROR_SUCCESS)
    {
        let pt = POINT {
            x: GET_X_LPARAM(l_param),
            y: GET_Y_LPARAM(l_param),
        };

        if !(*this).m_pShowDesktopButton.is_null()
            && IsWindow(TrayShowDesktopButton_GetHWND((*this).m_pShowDesktopButton)) != 0
            && TrayShowDesktopButton_PtInButton((*this).m_pShowDesktopButton, &pt) != 0
        {
            return HTBORDER as LRESULT;
        }

        if PtInRect(&rc_client, pt) != 0 {
            return HTCAPTION as LRESULT;
        }

        match (*this).m_Position {
            ABE_TOP => {
                if pt.y > rc_client.bottom {
                    return HTBOTTOM as LRESULT;
                }
            }
            ABE_LEFT => {
                if pt.x > rc_client.right {
                    return HTRIGHT as LRESULT;
                }
            }
            ABE_RIGHT => {
                if pt.x < rc_client.left {
                    return HTLEFT as LRESULT;
                }
            }
            _ => {
                if pt.y < rc_client.top {
                    return HTTOP as LRESULT;
                }
            }
        }
    }
    HTBORDER as LRESULT
}

unsafe fn on_moving(this: *mut CTrayWindow, l_param: LPARAM) -> LRESULT {
    let mut pt_cursor: POINT = core::mem::zeroed();
    let p_rect = l_param as *mut RECT;

    if g_TaskbarSettings.bLock == 0 && GetCursorPos(&mut pt_cursor) != 0 {
        (*this).set_flag(F_ISDRAGGING, true);
        (*this).m_DraggingPosition =
            tw_get_dragging_rect_from_pt(this, pt_cursor, p_rect, &mut (*this).m_DraggingMonitor);
    } else {
        *p_rect = (*this).m_TrayRects[(*this).m_Position as usize];
    }
    TRUE as LRESULT
}

unsafe fn on_sizing(this: *mut CTrayWindow, l_param: LPARAM) -> LRESULT {
    let p_rect = l_param as *mut RECT;
    if g_TaskbarSettings.bLock == 0 {
        tw_fit_to_rebar(this, p_rect);
    } else {
        *p_rect = (*this).m_TrayRects[(*this).m_Position as usize];
    }
    TRUE as LRESULT
}

unsafe fn on_window_pos_changing(this: *mut CTrayWindow, l_param: LPARAM) -> LRESULT {
    tw_changing_win_pos(this, l_param as *mut WINDOWPOS);
    TRUE as LRESULT
}

unsafe fn on_size(this: *mut CTrayWindow, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    let mut rc_client: RECT = core::mem::zeroed();
    if w_param == SIZE_RESTORED as WPARAM && l_param == 0 {
        tw_resize_work_area(this);
        tw_apply_clipping(this, TRUE as BOOL);
        if GetClientRect((*this).m_hWnd, &mut rc_client) == 0 {
            return FALSE as LRESULT;
        }
    } else {
        rc_client.left = 0;
        rc_client.top = 0;
        rc_client.right = LOWORD(l_param as usize) as i32;
        rc_client.bottom = HIWORD(l_param as usize) as i32;
    }
    tw_align_controls(this, &rc_client);
    TRUE as LRESULT
}

unsafe fn on_enter_size_move(this: *mut CTrayWindow) -> LRESULT {
    (*this).set_flag(F_INSIZEMOVE, true);
    (*this).set_flag(F_ISDRAGGING, false);
    if g_TaskbarSettings.bLock == 0 {
        tw_apply_clipping(this, FALSE as BOOL);
    }
    (*this).m_PreviousMonitor = (*this).m_Monitor;
    TRUE as LRESULT
}

unsafe fn on_exit_size_move(this: *mut CTrayWindow) -> LRESULT {
    (*this).set_flag(F_INSIZEMOVE, false);
    if g_TaskbarSettings.bLock == 0 {
        tw_fit_to_rebar(this, &mut (*this).m_TrayRects[(*this).m_Position as usize]);
        PostMessageW((*this).m_hWnd, WM_SIZE, SIZE_RESTORED as WPARAM, 0);
    }
    TRUE as LRESULT
}

unsafe fn on_sys_char(this: *mut CTrayWindow, w_param: WPARAM, b_handled: *mut BOOL) -> LRESULT {
    if w_param == b' ' as WPARAM {
        // Alt+Space — the tray window has no system menu; synthesise one.
        let uid_disable_item = [SC_RESTORE, SC_MOVE, SC_SIZE, SC_MAXIMIZE, SC_MINIMIZE];

        set_window_style((*this).m_hWnd, WS_SYSMENU, WS_SYSMENU);

        let h_sys_menu = GetSystemMenu((*this).m_hWnd, FALSE as BOOL);
        if !h_sys_menu.is_null() {
            for item in uid_disable_item.iter() {
                EnableMenuItem(h_sys_menu, *item, MF_BYCOMMAND | MF_GRAYED);
            }
            EnableMenuItem(
                h_sys_menu,
                SC_CLOSE,
                MF_BYCOMMAND | if SHRestricted(REST_NOCLOSE) != 0 { MF_GRAYED } else { MF_ENABLED },
            );

            let uid = tw_track_menu(
                this,
                h_sys_menu,
                null(),
                (*this).m_StartButton.m_hWnd,
                ((*this).m_Position != ABE_TOP) as BOOL,
                FALSE as BOOL,
            );
            if uid != 0 {
                SendMessageW((*this).m_hWnd, WM_SYSCOMMAND, uid as WPARAM, 0);
            }
        }
        set_window_style((*this).m_hWnd, WS_SYSMENU, 0);
    } else {
        *b_handled = FALSE as BOOL;
    }
    TRUE as LRESULT
}

unsafe fn tw_is_point_within_start_button(
    this: *mut CTrayWindow,
    ppt: *const POINT,
    prc_start_btn: *const RECT,
    pwi: *const WINDOWINFO,
) -> BOOL {
    if ppt.is_null() || prc_start_btn.is_null() || pwi.is_null() {
        return FALSE as BOOL;
    }
    match (*this).m_Position {
        ABE_TOP | ABE_LEFT => {
            if (*ppt).x > (*prc_start_btn).right || (*ppt).y > (*prc_start_btn).bottom {
                return FALSE as BOOL;
            }
        }
        ABE_RIGHT => {
            if (*ppt).x < (*prc_start_btn).left || (*ppt).y > (*prc_start_btn).bottom {
                return FALSE as BOOL;
            }
            if (*prc_start_btn).right + (*pwi).cxWindowBorders as i32 * 2 + 1 < (*pwi).rcWindow.right
                && (*ppt).x > (*prc_start_btn).right
            {
                return FALSE as BOOL;
            }
        }
        ABE_BOTTOM => {
            if (*ppt).x > (*prc_start_btn).right || (*ppt).y < (*prc_start_btn).top {
                return FALSE as BOOL;
            }
            if (*prc_start_btn).bottom + (*pwi).cyWindowBorders as i32 * 2 + 1 < (*pwi).rcWindow.bottom
                && (*ppt).y > (*prc_start_btn).bottom
            {
                return FALSE as BOOL;
            }
        }
        _ => {}
    }
    TRUE as BOOL
}

unsafe fn on_nc_lbutton_down(this: *mut CTrayWindow, l_param: LPARAM, b_handled: *mut BOOL) -> LRESULT {
    let pt = POINT {
        x: GET_X_LPARAM(l_param),
        y: GET_Y_LPARAM(l_param),
    };
    let mut wi: WINDOWINFO = core::mem::zeroed();
    wi.cbSize = size_of::<WINDOWINFO>() as DWORD;

    *b_handled = FALSE as BOOL;

    let mut rc_start_btn: RECT = core::mem::zeroed();
    GetWindowRect((*this).m_StartButton.m_hWnd, &mut rc_start_btn);
    GetWindowInfo((*this).m_hWnd, &mut wi);

    if tw_is_point_within_start_button(this, &pt, &rc_start_btn, &wi) != 0 {
        *b_handled = TRUE as BOOL;
        tw_popup_start_menu(this);
        return 0;
    }

    if !(*this).m_pShowDesktopButton.is_null()
        && TrayShowDesktopButton_PtInButton((*this).m_pShowDesktopButton, &pt) != 0
    {
        TrayShowDesktopButton_OnLButtonDown((*this).m_pShowDesktopButton, WM_LBUTTONDOWN, 0, 0, b_handled);
    }
    0
}

unsafe fn on_nc_rbutton_up(this: *mut CTrayWindow, l_param: LPARAM, b_handled: *mut BOOL) -> LRESULT {
    on_context_menu(this, WM_CONTEXTMENU, (*this).m_hWnd as WPARAM, l_param, b_handled)
}

unsafe fn on_context_menu(this: *mut CTrayWindow, u_msg: UINT, w_param: WPARAM, l_param: LPARAM, _b_handled: *mut BOOL) -> LRESULT {
    let mut ret: LRESULT = FALSE as LRESULT;
    let mut pt: POINT = POINT { x: 0, y: 0 };
    let mut ppt: *const POINT = null();
    let mut hwnd_exclude: HWND = null_mut();

    if SHRestricted(REST_NOTRAYCONTEXTMENU) != 0 {
        return FALSE as LRESULT;
    }

    pt.x = (LOWORD(l_param as usize) as i16) as i32;
    pt.y = (HIWORD(l_param as usize) as i16) as i32;

    if pt.x != -1 || pt.y != -1 {
        ppt = &pt;
    } else {
        hwnd_exclude = (*this).m_StartButton.m_hWnd;
    }

    if w_param as HWND == (*this).m_StartButton.m_hWnd {
        if SendMessageW((*this).m_StartButton.m_hWnd, BM_GETSTATE, 0, 0) & BST_PUSHED as LRESULT == 0 {
            let mut ctx_menu: *mut IContextMenu = null_mut();
            CStartMenuBtnCtxMenu_CreateInstance(
                addr_of_mut!((*this).lpVtbl_TrayWindow) as *mut ITrayWindow,
                (*this).m_hWnd,
                &mut ctx_menu,
            );
            tw_track_ctx_menu(
                this,
                ctx_menu,
                ppt,
                hwnd_exclude,
                ((*this).m_Position == ABE_BOTTOM) as BOOL,
                this as *mut c_void,
            );
            if !ctx_menu.is_null() {
                (*(ctx_menu as *mut IUnknown)).Release();
            }
        }
        return ret;
    }

    // Non-start-button path.
    let mut handle_tray_ctx = false;
    if !ppt.is_null() && !(*this).m_TrayBandSite.is_null() {
        let mut pt_client = *ppt;
        MapWindowPoints(null_mut(), (*this).m_hWnd, &mut pt_client, 1);

        let mut hwnd_at_pt = ChildWindowFromPoint((*this).m_hWnd, pt_client);
        if !hwnd_at_pt.is_null() && (hwnd_at_pt == (*this).m_Rebar || IsChild((*this).m_Rebar, hwnd_at_pt) != 0)
        {
            pt_client = *ppt;
            MapWindowPoints(null_mut(), (*this).m_Rebar, &mut pt_client, 1);
            hwnd_at_pt = ChildWindowFromPointEx((*this).m_Rebar, pt_client, CWP_SKIPINVISIBLE | CWP_SKIPDISABLED);
            if hwnd_at_pt == (*this).m_TaskSwitch {
                handle_tray_ctx = true;
            } else {
                ((*(*(*this).m_TrayBandSite).lpVtbl).ProcessMessage)(
                    (*this).m_TrayBandSite,
                    (*this).m_hWnd,
                    u_msg,
                    w_param,
                    l_param,
                    &mut ret,
                );
            }
        } else {
            handle_tray_ctx = true;
        }
    } else {
        handle_tray_ctx = true;
    }

    if handle_tray_ctx {
        tw_track_ctx_menu(
            this,
            addr_of_mut!((*this).lpVtbl_ContextMenu) as *mut IContextMenu,
            ppt,
            null_mut(),
            FALSE as BOOL,
            this as *mut c_void,
        );
    }
    ret
}

unsafe fn on_notify(this: *mut CTrayWindow, u_msg: UINT, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    let mut ret: LRESULT = FALSE as LRESULT;
    let mut hr: HRESULT = E_FAIL;

    if !(*this).m_TrayBandSite.is_null() {
        hr = ((*(*(*this).m_TrayBandSite).lpVtbl).ProcessMessage)(
            (*this).m_TrayBandSite,
            (*this).m_hWnd,
            u_msg,
            w_param,
            l_param,
            &mut ret,
        );
        if hr >= 0 {
            return ret;
        }
    }

    if (*this).m_TrayBandSite.is_null() || hr < 0 {
        let nmh = l_param as *const NMHDR;
        if (*nmh).hwndFrom == (*this).m_TrayNotify && (*nmh).code == NTNWM_REALIGN {
            PostMessageW((*this).m_hWnd, WM_SIZE, SIZE_RESTORED as WPARAM, 0);
        }
    }
    ret
}

unsafe fn on_nc_lbutton_dbl_click(this: *mut CTrayWindow, u_msg: UINT, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    SendMessageW((*this).m_TrayNotify, u_msg, w_param, l_param);
    TRUE as LRESULT
}

unsafe fn on_nc_lbutton_up(this: *mut CTrayWindow, b_handled: *mut BOOL) -> LRESULT {
    if !(*this).m_pShowDesktopButton.is_null()
        && TrayShowDesktopButton_GetPressed((*this).m_pShowDesktopButton) != 0
    {
        TrayShowDesktopButton_Click((*this).m_pShowDesktopButton);
        TrayShowDesktopButton_OnLButtonUp((*this).m_pShowDesktopButton, WM_LBUTTONUP, 0, 0, b_handled);
        *b_handled = TRUE as BOOL;
    }
    FALSE as LRESULT
}

unsafe fn on_lbutton_up(this: *mut CTrayWindow, u_msg: UINT, w_param: WPARAM, l_param: LPARAM, b_handled: *mut BOOL) -> LRESULT {
    if !(*this).m_pShowDesktopButton.is_null() {
        TrayShowDesktopButton_OnLButtonUp((*this).m_pShowDesktopButton, u_msg, w_param, l_param, b_handled);
    }
    FALSE as LRESULT
}

unsafe fn on_app_tray_destroy(this: *mut CTrayWindow) -> LRESULT {
    DestroyWindow((*this).m_hWnd);
    TRUE as LRESULT
}

unsafe fn on_open_start_menu(this: *mut CTrayWindow) -> LRESULT {
    let mut hwnd_start_menu: HWND = null_mut();
    let hr = IUnknown_GetWindow((*this).m_StartMenuPopup as *mut IUnknown, &mut hwnd_start_menu);
    if FAILED_UNEXPECTEDLY(hr) != 0 {
        return FALSE as LRESULT;
    }
    if IsWindowVisible(hwnd_start_menu) != 0 {
        tw_hide_start_menu(this);
    } else {
        tw_popup_start_menu(this);
    }
    TRUE as LRESULT
}

unsafe fn on_do_exit_windows(this: *mut CTrayWindow) -> LRESULT {
    tw_do_exit_windows(this)
}

unsafe fn on_sys_command(this: *mut CTrayWindow, w_param: WPARAM, b_handled: *mut BOOL) -> LRESULT {
    if w_param == SC_CLOSE as WPARAM {
        return tw_do_exit_windows(this);
    }
    *b_handled = FALSE as BOOL;
    TRUE as LRESULT
}

unsafe fn on_get_task_switch(this: *mut CTrayWindow, b_handled: *mut BOOL) -> LRESULT {
    *b_handled = TRUE as BOOL;
    (*this).m_TaskSwitch as LRESULT
}

unsafe fn on_set_zorder(this: *mut CTrayWindow, w_param: WPARAM) -> LRESULT {
    SetWindowPos(
        (*this).m_hWnd,
        w_param as HWND,
        0,
        0,
        0,
        0,
        SWP_NOOWNERZORDER | SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE,
    ) as LRESULT
}

unsafe fn on_hotkey(this: *mut CTrayWindow, w_param: WPARAM) -> LRESULT {
    tw_handle_hot_key(this, w_param as DWORD)
}

unsafe fn on_command(this: *mut CTrayWindow, u_msg: UINT, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    let mut ret: LRESULT = FALSE as LRESULT;

    if l_param as HWND == (*this).m_StartButton.m_hWnd {
        return FALSE as LRESULT;
    }

    if (*this).m_TrayBandSite.is_null()
        || FAILED_UNEXPECTEDLY(((*(*(*this).m_TrayBandSite).lpVtbl).ProcessMessage)(
            (*this).m_TrayBandSite,
            (*this).m_hWnd,
            u_msg,
            w_param,
            l_param,
            &mut ret,
        )) != 0
    {
        return tw_handle_command(this, LOWORD(w_param as usize) as UINT);
    }
    ret
}

unsafe fn on_mouse_move(this: *mut CTrayWindow, u_msg: UINT, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    SendMessageW((*this).m_TrayNotify, u_msg, w_param, l_param);
    if ab_is_auto_hide_state(this) != 0 {
        SetTimer((*this).m_hWnd, TIMER_ID_MOUSETRACK, MOUSETRACK_INTERVAL, None);
    }
    TRUE as LRESULT
}

unsafe fn on_timer(this: *mut CTrayWindow, w_param: WPARAM, b_handled: *mut BOOL) -> LRESULT {
    match w_param as UINT_PTR {
        TIMER_ID_MOUSETRACK => tw_process_mouse_tracking(this),
        TIMER_ID_AUTOHIDE => tw_process_auto_hide(this),
        _ => *b_handled = FALSE as BOOL,
    }
    0
}

unsafe fn on_nc_activate(this: *mut CTrayWindow, u_msg: UINT, w_param: WPARAM, l_param: LPARAM, b_handled: *mut BOOL) -> LRESULT {
    let ret = DefWindowProcW((*this).m_hWnd, u_msg, w_param, l_param);
    tw_draw_show_desktop_button(this);
    *b_handled = TRUE as BOOL;
    ret
}

unsafe fn on_nc_calc_size(this: *mut CTrayWindow, w_param: WPARAM, l_param: LPARAM, b_handled: *mut BOOL) -> LRESULT {
    let rc: *mut RECT;
    if (*this).m_Theme.is_null() || g_TaskbarSettings.bLock != 0 {
        *b_handled = FALSE as BOOL;
        return 0;
    }
    if w_param == 0 {
        rc = w_param as *mut RECT; // == null, matches C++ (RECT*)wParam
    } else {
        let prms = l_param as *mut NCCALCSIZE_PARAMS;
        if (*(*prms).lppos).flags & SWP_NOSENDCHANGING != 0 {
            *b_handled = FALSE as BOOL;
            return 0;
        }
        rc = &mut (*prms).rgrc[0];
    }
    tw_adjust_sizer_rect(this, rc, (*this).m_Position);
    0
}

unsafe fn on_init_menu_popup(_this: *mut CTrayWindow, w_param: WPARAM) -> LRESULT {
    let h_menu = w_param as HMENU;
    if IsThereAnyEffectiveWindow(FALSE as BOOL) != 0 {
        EnableMenuItem(h_menu, ID_SHELL_CMD_CASCADE_WND, MF_BYCOMMAND | MF_ENABLED);
        EnableMenuItem(h_menu, ID_SHELL_CMD_TILE_WND_H, MF_BYCOMMAND | MF_ENABLED);
        EnableMenuItem(h_menu, ID_SHELL_CMD_TILE_WND_V, MF_BYCOMMAND | MF_ENABLED);
        if g_Arrangement != ARRANGE_NONE {
            let res_id = if g_Arrangement == ARRANGE_TILED {
                IDS_TRAYWND_UNDO_TILE
            } else {
                IDS_TRAYWND_UNDO_CASCADE
            };
            let mut sz_caption: [u16; 128] = [0; 128];
            LoadStringW(hExplorerInstance, res_id as UINT, sz_caption.as_mut_ptr(), sz_caption.len() as i32);
            let mut mii: MENUITEMINFOW = core::mem::zeroed();
            mii.cbSize = size_of::<MENUITEMINFOW>() as UINT;
            GetMenuItemInfoW(h_menu, ID_SHELL_CMD_UNDO_ACTION, FALSE as BOOL, &mut mii);
            mii.fMask = MIIM_TYPE;
            mii.fType = MFT_STRING;
            mii.dwTypeData = sz_caption.as_mut_ptr();
            SetMenuItemInfoW(h_menu, ID_SHELL_CMD_UNDO_ACTION, FALSE as BOOL, &mii);
        } else {
            DeleteMenu(h_menu, ID_SHELL_CMD_UNDO_ACTION, MF_BYCOMMAND);
        }
    } else {
        EnableMenuItem(h_menu, ID_SHELL_CMD_CASCADE_WND, MF_BYCOMMAND | MF_GRAYED);
        EnableMenuItem(h_menu, ID_SHELL_CMD_TILE_WND_H, MF_BYCOMMAND | MF_GRAYED);
        EnableMenuItem(h_menu, ID_SHELL_CMD_TILE_WND_V, MF_BYCOMMAND | MF_GRAYED);
        DeleteMenu(h_menu, ID_SHELL_CMD_UNDO_ACTION, MF_BYCOMMAND);
        g_Arrangement = ARRANGE_NONE;
        (*&raw mut g_WindowPosBackup).remove_all();
    }
    0
}

unsafe fn on_activate(this: *mut CTrayWindow, w_param: WPARAM) -> LRESULT {
    ab_on_app_bar_activation_change2(this, (*this).m_hWnd, (*this).m_Position);
    if w_param == 0 {
        SendMessageW(
            (*this).m_hWnd,
            WM_CHANGEUISTATE,
            make_hotkey_word(UIS_SET, UISF_HIDEACCEL | UISF_HIDEFOCUS) as WPARAM,
            0,
        );
        IUnknown_UIActivateIO((*this).m_TrayBandSite as *mut IUnknown, FALSE as BOOL, null_mut());
    }
    0
}

#[inline]
fn make_hotkey_word(lo: UINT, hi: UINT) -> u32 {
    // MAKELONG(lo, hi)
    (lo & 0xFFFF) | ((hi & 0xFFFF) << 16)
}

unsafe fn on_set_focus(this: *mut CTrayWindow) -> LRESULT {
    IUnknown_UIActivateIO((*this).m_TrayBandSite as *mut IUnknown, TRUE as BOOL, null_mut());
    0
}

unsafe fn on_get_min_max_info(this: *mut CTrayWindow, l_param: LPARAM) -> LRESULT {
    let p_info = l_param as *mut MINMAXINFO;
    let start_size = sb_get_size(addr_of_mut!((*this).m_StartButton));
    (*p_info).ptMinTrackSize.x = start_size.cx + 2 * GetSystemMetrics(SM_CXFRAME as i32);
    (*p_info).ptMinTrackSize.y = start_size.cy + 2 * GetSystemMetrics(SM_CYFRAME as i32);
    0
}

unsafe fn on_taskbar_settings_changed(this: *mut CTrayWindow, u_msg: UINT, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    let new_settings = l_param as *const TaskbarSettings;

    SendMessageW((*this).m_TaskSwitch, u_msg, w_param, l_param);
    SendMessageW((*this).m_TrayNotify, u_msg, w_param, l_param);

    tw_set_auto_hide_state(this, (*new_settings).sr.AutoHide() as BOOL);
    tw_lock(this, (*new_settings).bLock);
    tw_update_always_on_top(this, (*new_settings).sr.AlwaysOnTop() as BOOL);
    tw_check_tray_wnd_position(this);

    TaskbarSettings_save_global();
    0
}

// ============================================================================
// ITrayWindow vtable methods
// ============================================================================

unsafe extern "system" fn itw_Open(this: *mut ITrayWindow) -> HRESULT {
    tw_open(from_tw(this))
}
unsafe extern "system" fn itw_Close(this: *mut ITrayWindow) -> HRESULT {
    tw_close(from_tw(this))
}
unsafe extern "system" fn itw_GetHWND(this: *mut ITrayWindow) -> HWND {
    (*from_tw(this)).m_hWnd
}
unsafe extern "system" fn itw_IsSpecialHWND(this: *mut ITrayWindow, h_wnd: HWND) -> BOOL {
    tw_is_special_hwnd(from_tw(this), h_wnd)
}
unsafe extern "system" fn itw_IsHorizontal(this: *mut ITrayWindow) -> BOOL {
    tw_is_pos_horizontal(from_tw(this))
}
unsafe extern "system" fn itw_DisplayProperties(this: *mut ITrayWindow) -> HWND {
    tw_display_properties(from_tw(this))
}
unsafe extern "system" fn itw_ExecContextMenuCmd(this: *mut ITrayWindow, ui_cmd: UINT) -> BOOL {
    tw_exec_context_menu_cmd(from_tw(this), ui_cmd)
}
unsafe extern "system" fn itw_Lock(this: *mut ITrayWindow, b_lock: BOOL) -> BOOL {
    tw_lock(from_tw(this), b_lock)
}
unsafe extern "system" fn itw_IsTaskWnd(this: *mut ITrayWindow, h_wnd: HWND) -> BOOL {
    tw_is_task_wnd(from_tw(this), h_wnd)
}
unsafe extern "system" fn itw_NotifyFullScreenToAppBars(
    this: *mut ITrayWindow,
    h_monitor: HMONITOR,
    b_full_opening: BOOL,
) -> HRESULT {
    ab_on_app_bar_notify_all(
        from_tw(this),
        h_monitor,
        null_mut(),
        ABN_FULLSCREENAPP,
        b_full_opening as LPARAM,
    );
    S_OK
}

// ============================================================================
// IShellDesktopTray vtable methods
// ============================================================================

unsafe extern "system" fn sdt_GetState(_this: *mut IShellDesktopTray) -> ULONG {
    0
}
unsafe extern "system" fn sdt_GetTrayWindow(this: *mut IShellDesktopTray, phwnd_tray: *mut HWND) -> HRESULT {
    *phwnd_tray = (*from_sdt(this)).m_hWnd;
    S_OK
}
unsafe extern "system" fn sdt_RegisterDesktopWindow(this: *mut IShellDesktopTray, hwnd_desktop: HWND) -> HRESULT {
    (*from_sdt(this)).m_DesktopWnd = hwnd_desktop;
    S_OK
}
unsafe extern "system" fn sdt_Unknown(_this: *mut IShellDesktopTray, _d1: DWORD, _d2: DWORD) -> HRESULT {
    S_OK
}

// ============================================================================
// IOleWindow vtable methods
// ============================================================================

unsafe extern "system" fn ow_GetWindow(this: *mut IOleWindow, phwnd: *mut HWND) -> HRESULT {
    if phwnd.is_null() {
        return E_INVALIDARG;
    }
    *phwnd = (*from_ow(this)).m_hWnd;
    S_OK
}
unsafe extern "system" fn ow_ContextSensitiveHelp(_this: *mut IOleWindow, _f: BOOL) -> HRESULT {
    E_NOTIMPL
}

// ============================================================================
// IContextMenu vtable methods (forward to m_ContextMenu, created lazily)
// ============================================================================

unsafe extern "system" fn cm_QueryContextMenu(
    this: *mut IContextMenu,
    h_popup: HMENU,
    index_menu: UINT,
    id_cmd_first: UINT,
    id_cmd_last: UINT,
    u_flags: UINT,
) -> HRESULT {
    let obj = from_cm(this);
    if (*obj).m_ContextMenu.is_null() {
        let hr = TrayWindowCtxMenuCreator(
            addr_of_mut!((*obj).lpVtbl_TrayWindow) as *mut ITrayWindow,
            (*obj).m_hWnd,
            &mut (*obj).m_ContextMenu,
        );
        if FAILED_UNEXPECTEDLY(hr) != 0 {
            return hr;
        }
    }
    ((*(*(*obj).m_ContextMenu).lpVtbl).QueryContextMenu)(
        (*obj).m_ContextMenu,
        h_popup,
        index_menu,
        id_cmd_first,
        id_cmd_last,
        u_flags,
    )
}
unsafe extern "system" fn cm_InvokeCommand(this: *mut IContextMenu, lpici: *const CMINVOKECOMMANDINFO) -> HRESULT {
    let obj = from_cm(this);
    if (*obj).m_ContextMenu.is_null() {
        return E_INVALIDARG;
    }
    ((*(*(*obj).m_ContextMenu).lpVtbl).InvokeCommand)((*obj).m_ContextMenu, lpici)
}
unsafe extern "system" fn cm_GetCommandString(
    this: *mut IContextMenu,
    id_cmd: UINT_PTR,
    u_type: UINT,
    pw_reserved: *mut UINT,
    psz_name: LPSTR,
    cch_max: UINT,
) -> HRESULT {
    let obj = from_cm(this);
    if (*obj).m_ContextMenu.is_null() {
        return E_INVALIDARG;
    }
    ((*(*(*obj).m_ContextMenu).lpVtbl).GetCommandString)(
        (*obj).m_ContextMenu,
        id_cmd,
        u_type,
        pw_reserved,
        psz_name,
        cch_max,
    )
}

// ============================================================================
// static vtables
// ============================================================================

static vt_TrayWindow: ITrayWindowVtbl = ITrayWindowVtbl {
    QueryInterface: tw_QI,
    AddRef: tw_AddRef,
    Release: tw_Release,
    Open: itw_Open,
    Close: itw_Close,
    GetHWND: itw_GetHWND,
    IsSpecialHWND: itw_IsSpecialHWND,
    IsHorizontal: itw_IsHorizontal,
    DisplayProperties: itw_DisplayProperties,
    ExecContextMenuCmd: itw_ExecContextMenuCmd,
    Lock: itw_Lock,
    IsTaskWnd: itw_IsTaskWnd,
    NotifyFullScreenToAppBars: itw_NotifyFullScreenToAppBars,
};
static vt_ShellDesktopTray: IShellDesktopTrayVtbl = IShellDesktopTrayVtbl {
    QueryInterface: sdt_QI,
    AddRef: sdt_AddRef,
    Release: sdt_Release,
    GetState: sdt_GetState,
    GetTrayWindow: sdt_GetTrayWindow,
    RegisterDesktopWindow: sdt_RegisterDesktopWindow,
    Unknown: sdt_Unknown,
};
static vt_OleWindow: IOleWindowVtbl = IOleWindowVtbl {
    QueryInterface: ow_QI,
    AddRef: ow_AddRef,
    Release: ow_Release,
    GetWindow: ow_GetWindow,
    ContextSensitiveHelp: ow_ContextSensitiveHelp,
};
static vt_ContextMenu: IContextMenuVtbl = IContextMenuVtbl {
    QueryInterface: cm_QI,
    AddRef: cm_AddRef,
    Release: cm_Release,
    QueryContextMenu: cm_QueryContextMenu,
    InvokeCommand: cm_InvokeCommand,
    GetCommandString: cm_GetCommandString,
};

// ============================================================================
// construction / destruction
// ============================================================================

unsafe fn tw_ctor(this: *mut CTrayWindow) {
    (*this).lpVtbl_TrayWindow = &vt_TrayWindow;
    (*this).lpVtbl_ShellDesktopTray = &vt_ShellDesktopTray;
    (*this).lpVtbl_OleWindow = &vt_OleWindow;
    (*this).lpVtbl_ContextMenu = &vt_ContextMenu;
    (*this).ref_ = 0;
    (*this).m_hWnd = null_mut();

    // CAppBarManager()
    (*this).m_hAppBarDPA = null_mut();
    (*this).m_ahwndAutoHideBars = [null_mut(); 4];

    (*this).m_pShowDesktopButton = null_mut();
    (*this).m_StartMenuBand = null_mut();
    (*this).m_StartMenuPopup = null_mut();
    (*this).m_TaskBand = null_mut();
    (*this).m_ContextMenu = null_mut();
    (*this).m_Theme = null_mut();
    (*this).m_Font = null_mut();
    (*this).m_DesktopWnd = null_mut();
    (*this).m_Rebar = null_mut();
    (*this).m_TaskSwitch = null_mut();
    (*this).m_TrayNotify = null_mut();
    (*this).m_TrayNotifyInstance = null_mut();
    (*this).m_Position = 0;
    (*this).m_Monitor = null_mut();
    (*this).m_PreviousMonitor = null_mut();
    (*this).m_DraggingPosition = 0;
    (*this).m_DraggingMonitor = null_mut();
    (*this).m_TrayRects = core::mem::zeroed();
    (*this).m_TraySize = SIZE { cx: 0, cy: 0 };
    (*this).m_TrayPropertiesOwner = null_mut();
    (*this).m_RunFileDlgOwner = null_mut();
    (*this).m_AutoHideState = AUTOHIDE_HIDDEN;
    (*this).m_AutoHideOffset = SIZE { cx: 0, cy: 0 };
    (*this).m_MouseTrackingInfo = core::mem::zeroed();
    (*this).m_ShellServices = null_mut();
    (*this).m_TrayBandSite = null_mut();
    (*this).Flags = 0;

    sb_ctor(addr_of_mut!((*this).m_StartButton));
}

/// ~CTrayWindow().
unsafe fn tw_dtor(this: *mut CTrayWindow) {
    if !(*this).m_ShellServices.is_null() {
        ShutdownShellServices((*this).m_ShellServices);
        (*this).m_ShellServices = null_mut();
    }
    if !(*this).m_Font.is_null() {
        DeleteObject((*this).m_Font as HGDIOBJ);
        (*this).m_Font = null_mut();
    }
    if !(*this).m_Theme.is_null() {
        CloseThemeData((*this).m_Theme);
        (*this).m_Theme = null_mut();
    }

    // CComPtr members release.
    if !(*this).m_StartMenuBand.is_null() {
        (*(*this).m_StartMenuBand).Release();
    }
    if !(*this).m_StartMenuPopup.is_null() {
        (*((*this).m_StartMenuPopup as *mut IUnknown)).Release();
    }
    if !(*this).m_TaskBand.is_null() {
        (*((*this).m_TaskBand as *mut IUnknown)).Release();
    }
    if !(*this).m_ContextMenu.is_null() {
        (*((*this).m_ContextMenu as *mut IUnknown)).Release();
    }
    if !(*this).m_TrayNotifyInstance.is_null() {
        (*(*this).m_TrayNotifyInstance).Release();
    }
    if !(*this).m_TrayBandSite.is_null() {
        (*((*this).m_TrayBandSite as *mut IUnknown)).Release();
    }

    sb_dtor(addr_of_mut!((*this).m_StartButton));

    ab_destroy_app_bar_dpa(this);

    PostQuitMessage(0);
}

/// `_Init()`.
unsafe fn tw_init(this: *mut CTrayWindow) {
    (*this).m_Position = DWORD::MAX;
}

// ============================================================================
// WndProc (BEGIN_MSG_MAP) — arch-split callback ABI
// ============================================================================

unsafe fn tw_process(
    this: *mut CTrayWindow,
    hWnd: HWND,
    uMsg: UINT,
    mut wParam: WPARAM,
    mut lParam: LPARAM,
) -> LRESULT {
    // BEGIN_MSG_MAP preamble: give the start-menu band first crack.
    if !(*this).m_StartMenuBand.is_null() {
        let mut msg: MSG = core::mem::zeroed();
        let mut l_ret: LRESULT = 0;
        msg.hwnd = (*this).m_hWnd;
        msg.message = uMsg;
        msg.wParam = wParam;
        msg.lParam = lParam;
        if (*(*this).m_StartMenuBand).TranslateMenuMessage(&mut msg, &mut l_ret) == S_OK {
            return l_ret;
        }
        wParam = msg.wParam;
        lParam = msg.lParam;
    }

    let mut b_handled: BOOL = TRUE as BOOL;
    let mut l_result: LRESULT = 0;

    match uMsg {
        WM_THEMECHANGED => l_result = on_theme_changed(this),
        WM_SETTINGCHANGE => l_result = on_setting_changed(this, uMsg, wParam, lParam),
        WM_ERASEBKGND => l_result = on_erase_background(this, wParam, &mut b_handled),
        WM_SIZE => l_result = on_size(this, wParam, lParam),
        WM_CREATE => l_result = on_create(this, &mut b_handled),
        WM_DESTROY => l_result = 0, // OnDestroy is empty
        WM_ENDSESSION => {
            if wParam != 0 {
                tw_save_state(this);
            }
            l_result = 0;
        }
        WM_NCHITTEST => l_result = on_nc_hit_test(this, lParam),
        WM_COMMAND => l_result = on_command(this, uMsg, wParam, lParam),
        WM_SYSCOMMAND => l_result = on_sys_command(this, wParam, &mut b_handled),
        WM_NOTIFY => {
            // NOTIFY_CODE_HANDLER(RBN_AUTOSIZE) is a documented no-op (#if 0) —
            // it sets bHandled=FALSE and falls through to OnNotify, so OnNotify
            // handles every WM_NOTIFY.
            let _ = RBN_AUTOSIZE;
            l_result = on_notify(this, uMsg, wParam, lParam);
        }
        WM_CONTEXTMENU => l_result = on_context_menu(this, uMsg, wParam, lParam, &mut b_handled),
        WM_TIMER => l_result = on_timer(this, wParam, &mut b_handled),
        WM_DISPLAYCHANGE => l_result = on_display_change(this),
        WM_COPYDATA => l_result = on_copy_data(this, uMsg, wParam, lParam),
        WM_NCPAINT => l_result = on_nc_paint(this, uMsg, wParam, lParam, &mut b_handled),
        WM_NCACTIVATE => l_result = on_nc_activate(this, uMsg, wParam, lParam, &mut b_handled),
        WM_CTLCOLORBTN => l_result = on_ctl_color_btn(wParam),
        WM_SYSCOLORCHANGE => l_result = on_sys_color_change(this, uMsg, wParam, lParam),
        WM_MOVING => l_result = on_moving(this, lParam),
        WM_SIZING => l_result = on_sizing(this, lParam),
        WM_WINDOWPOSCHANGING => l_result = on_window_pos_changing(this, lParam),
        WM_ENTERSIZEMOVE => l_result = on_enter_size_move(this),
        WM_EXITSIZEMOVE => l_result = on_exit_size_move(this),
        WM_NCLBUTTONDOWN => l_result = on_nc_lbutton_down(this, lParam, &mut b_handled),
        WM_SYSCHAR => l_result = on_sys_char(this, wParam, &mut b_handled),
        WM_NCRBUTTONUP => l_result = on_nc_rbutton_up(this, lParam, &mut b_handled),
        WM_NCLBUTTONDBLCLK => l_result = on_nc_lbutton_dbl_click(this, uMsg, wParam, lParam),
        WM_LBUTTONUP => l_result = on_lbutton_up(this, uMsg, wParam, lParam, &mut b_handled),
        WM_NCLBUTTONUP => l_result = on_nc_lbutton_up(this, &mut b_handled),
        WM_MOUSEMOVE | WM_NCMOUSEMOVE => l_result = on_mouse_move(this, uMsg, wParam, lParam),
        x if x == WM_APP_TRAYDESTROY => l_result = on_app_tray_destroy(this),
        WM_CLOSE => l_result = on_do_exit_windows(this),
        WM_HOTKEY => l_result = on_hotkey(this, wParam),
        WM_NCCALCSIZE => l_result = on_nc_calc_size(this, wParam, lParam, &mut b_handled),
        WM_INITMENUPOPUP => l_result = on_init_menu_popup(this, wParam),
        WM_ACTIVATE => l_result = on_activate(this, wParam),
        WM_SETFOCUS => l_result = on_set_focus(this),
        WM_GETMINMAXINFO => l_result = on_get_min_max_info(this, lParam),
        x if x == TWM_SETTINGSCHANGED => l_result = on_taskbar_settings_changed(this, uMsg, wParam, lParam),
        x if x == TWM_OPENSTARTMENU => l_result = on_open_start_menu(this),
        x if x == TWM_DOEXITWINDOWS => l_result = on_do_exit_windows(this),
        x if x == TWM_GETTASKSWITCH => l_result = on_get_task_switch(this, &mut b_handled),
        x if x == TWM_SETZORDER => l_result = on_set_zorder(this, wParam),
        _ => b_handled = FALSE as BOOL,
    }

    if b_handled != 0 {
        l_result
    } else {
        DefWindowProcW(hWnd, uMsg, wParam, lParam)
    }
}

#[no_mangle]
pub unsafe extern "system" fn CTrayWindow_WindowProc(
    hWnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    CTrayWindow_window_proc_impl(hWnd, uMsg, wParam, lParam)
}

#[inline]
unsafe fn CTrayWindow_window_proc_impl(
    hWnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let this = if uMsg == WM_NCCREATE {
        let p = wndproc_nccreate_this(hWnd, lParam) as *mut CTrayWindow;
        if !p.is_null() {
            (*p).m_hWnd = hWnd;
        }
        p
    } else {
        get_window_this(hWnd) as *mut CTrayWindow
    };

    if this.is_null() {
        return DefWindowProcW(hWnd, uMsg, wParam, lParam);
    }

    let r = tw_process(this, hWnd, uMsg, wParam, lParam);

    if uMsg == WM_NCDESTROY {
        (*this).m_hWnd = null_mut();
    }
    r
}

// ============================================================================
// CTrayWindowCtxMenu — the tray's IContextMenu helper (single interface).
// ============================================================================

#[repr(C)]
struct CTrayWindowCtxMenu {
    lpVtbl: *const IContextMenuVtbl,
    ref_: LONG,
    hWndOwner: HWND,
    TrayWnd: *mut CTrayWindow, // CComPtr<CTrayWindow>
    pcm: *mut IContextMenu,    // CComPtr<IContextMenu>
    m_idCmdCmFirst: UINT,
}

unsafe extern "system" fn cwcm_QueryInterface(
    iface: *mut IContextMenu,
    riid: REFIID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    let this = iface as *mut CTrayWindowCtxMenu;
    if ppv.is_null() {
        return E_POINTER;
    }
    if guid_eq(riid, &IID_IUnknown) || guid_eq(riid, &IID_IContextMenu) {
        *ppv = this as *mut c_void;
        cwcm_AddRef(iface);
        return S_OK;
    }
    *ppv = null_mut();
    E_NOINTERFACE
}
unsafe extern "system" fn cwcm_AddRef(iface: *mut IContextMenu) -> ULONG {
    let this = iface as *mut CTrayWindowCtxMenu;
    interlocked_increment(&mut (*this).ref_) as ULONG
}
unsafe extern "system" fn cwcm_Release(iface: *mut IContextMenu) -> ULONG {
    let this = iface as *mut CTrayWindowCtxMenu;
    let n = interlocked_decrement(&mut (*this).ref_);
    if n == 0 {
        // ~CTrayWindowCtxMenu + CComPtr member release.
        if !(*this).TrayWnd.is_null() {
            tw_release((*this).TrayWnd);
        }
        if !(*this).pcm.is_null() {
            (*((*this).pcm as *mut IUnknown)).Release();
        }
        HeapFree(hProcessHeap, 0, this as *mut c_void);
        return 0;
    }
    n as ULONG
}

unsafe extern "system" fn cwcm_QueryContextMenu(
    iface: *mut IContextMenu,
    h_popup: HMENU,
    index_menu: UINT,
    id_cmd_first: UINT,
    id_cmd_last: UINT,
    _u_flags: UINT,
) -> HRESULT {
    let this = iface as *mut CTrayWindowCtxMenu;

    let h_menu_base = LoadPopupMenu(hExplorerInstance, IDM_TRAYWND as usize as LPCWSTR);
    if h_menu_base.is_null() {
        return HRESULT_FROM_WIN32(GetLastError());
    }

    if (*&raw const g_MinimizedAll).get_size() != 0 && IsThereAnyEffectiveWindow(TRUE as BOOL) == 0 {
        let mut sz_restore: [u16; 128] = [0; 128];
        LoadStringW(hExplorerInstance, IDS_RESTORE_ALL as UINT, sz_restore.as_mut_ptr(), sz_restore.len() as i32);
        let mut mii: MENUITEMINFOW = core::mem::zeroed();
        mii.cbSize = size_of::<MENUITEMINFOW>() as UINT;
        mii.fMask = MIIM_ID | MIIM_TYPE;
        mii.wID = ID_SHELL_CMD_RESTORE_ALL;
        mii.fType = MFT_STRING;
        mii.dwTypeData = sz_restore.as_mut_ptr();
        SetMenuItemInfoW(h_menu_base, ID_SHELL_CMD_SHOW_DESKTOP, FALSE as BOOL, &mii);
    }

    if SHRestricted(REST_CLASSICSHELL) != 0 {
        DeleteMenu(h_popup, ID_LOCKTASKBAR, MF_BYCOMMAND);
    }

    CheckMenuItem(
        h_menu_base,
        ID_LOCKTASKBAR,
        MF_BYCOMMAND | if g_TaskbarSettings.bLock != 0 { MF_CHECKED } else { MF_UNCHECKED },
    );

    let id_cmd_next = Shell_MergeMenus(
        h_popup,
        h_menu_base,
        index_menu,
        id_cmd_first,
        id_cmd_last,
        MM_SUBMENUSHAVEIDS | MM_ADDSEPARATOR,
    );
    (*this).m_idCmdCmFirst = id_cmd_next - id_cmd_first;

    DestroyMenu(h_menu_base);

    if !(*(*this).TrayWnd).m_TrayBandSite.is_null() {
        if !(*this).pcm.is_null() {
            (*((*this).pcm as *mut IUnknown)).Release();
            (*this).pcm = null_mut();
        }
        let band_site = (*(*this).TrayWnd).m_TrayBandSite;
        if ((*(*band_site).lpVtbl).AddContextMenus)(
            band_site,
            h_popup,
            index_menu,
            id_cmd_next,
            id_cmd_last,
            CMF_NORMAL,
            &mut (*this).pcm,
        ) < 0
        {
            if !(*this).pcm.is_null() {
                (*((*this).pcm as *mut IUnknown)).Release();
                (*this).pcm = null_mut();
            }
        }
    }

    S_OK
}

unsafe extern "system" fn cwcm_InvokeCommand(iface: *mut IContextMenu, lpici: *const CMINVOKECOMMANDINFO) -> HRESULT {
    let this = iface as *mut CTrayWindowCtxMenu;
    let ui_cmd_id = (*lpici).lpVerb as usize as UINT; // PtrToUlong
    if ui_cmd_id != 0 {
        if ui_cmd_id >= (*this).m_idCmdCmFirst {
            if !(*this).pcm.is_null() {
                let mut cmici: CMINVOKECOMMANDINFO = core::mem::zeroed();
                cmici.cbSize = size_of::<CMINVOKECOMMANDINFO>() as DWORD;
                cmici.hwnd = (*this).hWndOwner;
                cmici.lpVerb = (ui_cmd_id - (*this).m_idCmdCmFirst) as usize as LPCSTR;
                cmici.nShow = SW_NORMAL as INT;
                ((*(*(*this).pcm).lpVtbl).InvokeCommand)((*this).pcm, &cmici);
            }
        } else {
            tw_exec_context_menu_cmd((*this).TrayWnd, ui_cmd_id);
        }
    }
    S_OK
}

unsafe extern "system" fn cwcm_GetCommandString(
    _iface: *mut IContextMenu,
    _id_cmd: UINT_PTR,
    _u_type: UINT,
    _pw_reserved: *mut UINT,
    _psz_name: LPSTR,
    _cch_max: UINT,
) -> HRESULT {
    E_NOTIMPL
}

static vt_CtxMenu: IContextMenuVtbl = IContextMenuVtbl {
    QueryInterface: cwcm_QueryInterface,
    AddRef: cwcm_AddRef,
    Release: cwcm_Release,
    QueryContextMenu: cwcm_QueryContextMenu,
    InvokeCommand: cwcm_InvokeCommand,
    GetCommandString: cwcm_GetCommandString,
};

unsafe fn TrayWindowCtxMenuCreator(
    tray_wnd: *mut ITrayWindow,
    hwnd_owner: HWND,
    pp_ctx_menu: *mut *mut IContextMenu,
) -> HRESULT {
    let mnu = HeapAlloc(hProcessHeap, 0, size_of::<CTrayWindowCtxMenu>() as SIZE_T)
        as *mut CTrayWindowCtxMenu;
    if mnu.is_null() {
        return E_OUTOFMEMORY;
    }
    (*mnu).lpVtbl = &vt_CtxMenu;
    (*mnu).ref_ = 0; // new CComObject<> starts at 0 (faithful)
    // Initialize(pTrayWnd, hWndOwner): TrayWnd = (CTrayWindow*)pTrayWnd.
    (*mnu).hWndOwner = hwnd_owner;
    (*mnu).TrayWnd = from_tw(tray_wnd);
    (*mnu).pcm = null_mut();
    (*mnu).m_idCmdCmFirst = 0;
    // The CComPtr<CTrayWindow> member AddRefs on assignment.
    tw_addref((*mnu).TrayWnd);

    *pp_ctx_menu = mnu as *mut IContextMenu;
    S_OK
}

// ============================================================================
// message pumps + free functions (extern "C" — replace the C++ ones)
// ============================================================================

unsafe fn tw_tray_process_messages(this: *mut CTrayWindow) {
    let mut msg: MSG = core::mem::zeroed();
    while PeekMessageW(&mut msg, null_mut(), 0, 0, PM_REMOVE) != 0 {
        if msg.message == WM_QUIT {
            break;
        }
        if (*this).m_StartMenuBand.is_null()
            || (*(*this).m_StartMenuBand).IsMenuMessage(&mut msg) != S_OK
        {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

unsafe fn tw_tray_message_loop(this: *mut CTrayWindow) {
    let mut msg: MSG = core::mem::zeroed();
    loop {
        let ret = GetMessageW(&mut msg, null_mut(), 0, 0);
        if ret == 0 || ret == -1 {
            break;
        }
        if (*this).m_StartMenuBand.is_null()
            || (*(*this).m_StartMenuBand).IsMenuMessage(&mut msg) != S_OK
        {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

/// `HRESULT CreateTrayWindow(ITrayWindow **ppTray)` — the factory. Was a plain
/// C++ function; now #[no_mangle] extern "C" (precomp.h must gain `extern "C"`
/// — see MANIFEST). Returns the ITrayWindow* (== &lpVtbl_TrayWindow, offset 0).
#[no_mangle]
pub unsafe extern "C" fn CreateTrayWindow(pp_tray: *mut *mut ITrayWindow) -> HRESULT {
    let tray = HeapAlloc(hProcessHeap, 0, size_of::<CTrayWindow>() as SIZE_T) as *mut CTrayWindow;
    if tray.is_null() {
        return E_OUTOFMEMORY;
    }
    tw_ctor(tray);

    // CComPtr<CTrayWindow> Tray = new CComObject<CTrayWindow>() → the local ptr
    // holds a ref for the duration; OnCreate AddRefs the ITrayWindow the caller
    // keeps. Take a construction ref, mirror _Init()/Open(), then hand out.
    tw_addref(tray);

    tw_init(tray);
    tw_open(tray);

    // *ppTray = (ITrayWindow *) Tray;  — the caller's out-ref. The CComPtr local
    // releases its construction ref at end of C++ scope; here we drop ours after
    // publishing so the net public ref is the one OnCreate took.
    *pp_tray = addr_of_mut!((*tray).lpVtbl_TrayWindow) as *mut ITrayWindow;

    tw_release(tray);

    S_OK
}

/// `Tray_OnStartMenuDismissed(ITrayWindow*)` — extern "C" (already declared so
/// in precomp.h). RaiseStartButton.
#[no_mangle]
pub unsafe extern "C" fn Tray_OnStartMenuDismissed(tray: *mut ITrayWindow) -> HRESULT {
    tw_raise_start_button(from_tw(tray))
}

/// `VOID TrayProcessMessages(ITrayWindow*)` — extern "C".
#[no_mangle]
pub unsafe extern "C" fn TrayProcessMessages(tray: *mut ITrayWindow) {
    tw_tray_process_messages(from_tw(tray));
}

/// `VOID TrayMessageLoop(ITrayWindow*)` — extern "C".
#[no_mangle]
pub unsafe extern "C" fn TrayMessageLoop(tray: *mut ITrayWindow) {
    tw_tray_message_loop(from_tw(tray));
}
