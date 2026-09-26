//! syspager.cpp — `CSysPagerWnd`, the systray icon pager, plus its three
//! file-local helper classes: `CIconWatcher` (a worker thread that watches the
//! processes owning tray icons and synthesises a NIM_DELETE when one dies),
//! `CBalloonQueue` (serialises balloon-tip display through the shared tooltip),
//! and `CNotifyToolbar` (a `CToolbar<InternalIconData>` SUBCLASSED to host the
//! icon buttons). THIRD tier-C window/COM class; single QI-reachable interface
//! (IOleWindow) like trayclock, but far more entangled — see the SUBSTRATE
//! MANIFEST accompanying this file for the comctl32/notifyicon/thread surface it
//! needs and the subclass-thunk mechanism it introduces.
//!
//! Object model mirrors trayclock.rs: a flat `#[repr(C)]` lpVtbl-first object
//! reached only through its IOleWindow* (from the factory) or its own WndProc
//! (GWLP_USERDATA), so the layout is free. The embedded `CNotifyToolbar` is a
//! second window whose WndProc SUBCLASSES a real comctl32 toolbar: its `this`
//! rides in the toolbar's GWLP_USERDATA and unhandled messages forward to the
//! saved original toolbar proc via CallWindowProcW (the CWindowImplBaseT
//! ::DefWindowProc == CallWindowProc(super) contract).

use core::ffi::c_void;
use core::mem::{size_of, transmute};
use core::ptr::{null_mut, write_bytes};

use crate::explorer::settings::g_TaskbarSettings;
use explorer_sys::com::*; // REFIID, S_OK, E_*, IID_IUnknown/IID_IOleWindow, IOleWindow(+Vtbl)
use explorer_sys::globals::*;
use explorer_sys::server::{
    get_window_this, interlocked_decrement, interlocked_increment, register_class_once,
    set_window_this, wndproc_nccreate_this, DrawThemeParentBackground, IsAppThemed, SetWindowTheme,
    CW_USEDEFAULT, GET_X_LPARAM, TOOLINFOW, TTF_IDISHWND, TTS_NOPREFIX, TTTOOLINFOW_V1_SIZE,
};
use explorer_sys::types::TIM_Default;
use winsrv_sys::w;
use winsrv_sys::winsrv::*;

// ===========================================================================
// SUBSTRATE that is NOT in winsrv-sys/explorer-sys yet (comctl32 toolbar +
// imagelist, notifyicon structs/consts, critical-section enter/init, MulDiv,
// _beginthreadex). Declared locally so this staging file is self-contained;
// the MANIFEST recommends promoting the reusable parts into explorer_sys.
// explorer.exe already links comctl32/gdi32/kernel32/msvcrt, so these resolve.
// ===========================================================================

pub type HIMAGELIST = *mut c_void;

// ---- constants (cited in the manifest) ----
const NIM_ADD: DWORD = 0;
const NIM_MODIFY: DWORD = 1;
const NIM_DELETE: DWORD = 2;
const NIM_SETFOCUS: DWORD = 3;
const NIM_SETVERSION: DWORD = 4;

const NIF_MESSAGE: UINT = 1;
const NIF_ICON: UINT = 2;
const NIF_TIP: UINT = 4;
const NIF_STATE: UINT = 8;
const NIF_INFO: UINT = 16;

const NIS_HIDDEN: DWORD = 1;
const NIS_SHAREDICON: DWORD = 2;

const NIIF_USER: DWORD = 4;
const NIIF_ICON_MASK: DWORD = 0xf;

const NOTIFYICON_VERSION: UINT = 3;

const NIN_SELECT: UINT = WM_USER; // WM_USER + 0
const NIN_BALLOONSHOW: UINT = WM_USER + 2;
const NIN_BALLOONHIDE: UINT = WM_USER + 3;
const NIN_BALLOONTIMEOUT: UINT = WM_USER + 4;

// undocshell.h taskbar WM_COPYDATA
const NI_NOTIFY_SIG: DWORD = 0x3475_3423;
const TABDMC_NOTIFY: usize = 1;
const TABDMC_LOADINPROC: usize = 2;

// precomp.h tray window messages
const TNWM_GETMINIMUMSIZE: UINT = WM_USER + 0x100;
const NTNWM_REALIGN: UINT = 0x1;

// comctl32 toolbar messages (commctrl.h; W-variants for the UNICODE build)
const TB_ADDBUTTONSW: UINT = WM_USER + 68;
const TB_GETBUTTON: UINT = WM_USER + 23;
const TB_BUTTONCOUNT: UINT = WM_USER + 24;
const TB_DELETEBUTTON: UINT = WM_USER + 22;
const TB_GETITEMRECT: UINT = WM_USER + 29;
const TB_BUTTONSTRUCTSIZE: UINT = WM_USER + 30;
const TB_SETBUTTONSIZE: UINT = WM_USER + 31;
const TB_AUTOSIZE: UINT = WM_USER + 33;
const TB_GETTOOLTIPS: UINT = WM_USER + 35;
const TB_SETIMAGELIST: UINT = WM_USER + 48;
const TB_SETBUTTONINFOW: UINT = WM_USER + 64;
const TB_HITTEST: UINT = WM_USER + 69;
const TB_GETHOTITEM: UINT = WM_USER + 71;
const TB_SETMETRICS: UINT = WM_USER + 102;

const TBSTATE_ENABLED: u8 = 0x4;
const TBSTATE_HIDDEN: u8 = 0x8;
const TBSTYLE_TOOLTIPS: DWORD = 0x100;
const TBSTYLE_WRAPABLE: DWORD = 0x200;
const TBSTYLE_FLAT: DWORD = 0x800;
const TBSTYLE_TRANSPARENT: DWORD = 0x8000;
const TBSTYLE_NOPREFIX: DWORD = 0x20;
const BTNS_NOPREFIX: u8 = TBSTYLE_NOPREFIX as u8;

const TBIF_IMAGE: DWORD = 0x1;
const TBIF_STATE: DWORD = 0x4;
const TBIF_COMMAND: DWORD = 0x20;
const TBIF_BYINDEX: DWORD = 0x8000_0000;

const TBMF_PAD: DWORD = 0x1;
const TBMF_BARPAD: DWORD = 0x2;
const TBMF_BUTTONSPACING: DWORD = 0x4;

const CCS_TOP: DWORD = 0x1;
const CCS_NORESIZE: DWORD = 0x4;
const CCS_NOPARENTALIGN: DWORD = 0x8;
const CCS_NODIVIDER: DWORD = 0x40;

const ILC_MASK: UINT = 0x1;
const ILC_COLOR32: UINT = 0x20;

// custom-draw
const CDRF_NOTIFYITEMDRAW: LRESULT = 0x20;
const CDDS_PREPAINT: DWORD = 0x1;
const CDDS_ITEM: DWORD = 0x1_0000;
const CDDS_ITEMPREPAINT: DWORD = CDDS_ITEM | CDDS_PREPAINT;
const TBCDRF_NOEDGES: LRESULT = 0x1_0000;
const TBCDRF_NOOFFSET: LRESULT = 0x4_0000;
const TBCDRF_NOMARK: LRESULT = 0x8_0000;
const TBCDRF_NOETCHEDEFFECT: LRESULT = 0x10_0000;
const TBCDRF_NOBACKGROUND: LRESULT = 0x40_0000;

// notify codes
const NM_FIRST: UINT = 0u32.wrapping_sub(0);
const NM_CUSTOMDRAW: UINT = NM_FIRST.wrapping_sub(12);
const TTN_FIRST: UINT = 0u32.wrapping_sub(520);
const TTN_SHOW: UINT = TTN_FIRST.wrapping_sub(1);
const TTN_POP: UINT = TTN_FIRST.wrapping_sub(2);
const TBN_FIRST: UINT = 0u32.wrapping_sub(700);
const TBN_GETINFOTIPW: UINT = TBN_FIRST.wrapping_sub(19);

// tooltip styles / flags / messages (commctrl.h)
const TTS_BALLOON: DWORD = 0x40;
const TTS_CLOSE: DWORD = 0x80;
const TTF_TRACK: UINT = 0x20;
const TTS_ALWAYSTIP: DWORD = 0x1;
const TTM_ADDTOOLW: UINT = WM_USER + 50;
const TTM_TRACKACTIVATE: UINT = WM_USER + 17;
const TTM_TRACKPOSITION: UINT = WM_USER + 18;
const TTM_SETMAXTIPWIDTH: UINT = WM_USER + 24;
const TTM_SETTITLEW: UINT = WM_USER + 33;
const TTM_UPDATETIPTEXTW: UINT = WM_USER + 57;

const BALLOON_MAXWIDTH: i32 = 340;

// missing-from-bindgen misc
const HWND_BROADCAST: HWND = 0xffff as usize as HWND;
const WAIT_FAILED: DWORD = 0xffff_ffff;

static TOOLBARCLASSNAMEW: [u16; 16] = [
    b'T' as u16,
    b'o' as u16,
    b'o' as u16,
    b'l' as u16,
    b'b' as u16,
    b'a' as u16,
    b'r' as u16,
    b'W' as u16,
    b'i' as u16,
    b'n' as u16,
    b'd' as u16,
    b'o' as u16,
    b'w' as u16,
    b'3' as u16,
    b'2' as u16,
    0,
];
// L"tooltips_class32" (16 wide chars + NUL). The trailing 0 is load-bearing: commctrl.h defines
// this as a C string literal, which carries its terminator for free, and the hand-encoding does
// not. It is passed as CreateWindowExW's lpClassName, which reads until a NUL.
static TOOLTIPS_CLASSW: [u16; 17] = [
    b't' as u16,
    b'o' as u16,
    b'o' as u16,
    b'l' as u16,
    b't' as u16,
    b'i' as u16,
    b'p' as u16,
    b's' as u16,
    b'_' as u16,
    b'c' as u16,
    b'l' as u16,
    b'a' as u16,
    b's' as u16,
    b's' as u16,
    b'3' as u16,
    b'2' as u16,
    0,
];

// ---- comctl32 toolbar structs (commctrl.h) ----

#[repr(C)]
#[derive(Clone, Copy)]
struct TBBUTTON {
    iBitmap: i32,
    idCommand: i32,
    fsState: u8,
    fsStyle: u8,
    #[cfg(target_pointer_width = "64")]
    bReserved: [u8; 6],
    #[cfg(target_pointer_width = "32")]
    bReserved: [u8; 2],
    dwData: usize,  // DWORD_PTR
    iString: isize, // INT_PTR
}

#[repr(C)]
#[derive(Clone, Copy)]
struct TBBUTTONINFOW {
    cbSize: UINT,
    dwMask: DWORD,
    idCommand: i32,
    iImage: i32,
    fsState: u8,
    fsStyle: u8,
    cx: u16,
    lParam: usize, // DWORD_PTR
    pszText: LPWSTR,
    cchText: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct TBMETRICS {
    cbSize: UINT,
    dwMask: DWORD,
    cxPad: i32,
    cyPad: i32,
    cxBarPad: i32,
    cyBarPad: i32,
    cxButtonSpacing: i32,
    cyButtonSpacing: i32,
}

#[repr(C)]
struct NMTBGETINFOTIPW {
    hdr: NMHDR,
    pszText: LPWSTR,
    cchTextMax: i32,
    iItem: i32,
    lParam: LPARAM,
}

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

/// `NOTIFYICONDATAW` (shellapi.h; WIN2K szTip[128] .. VISTA hBalloonIcon). The
/// incoming icon payload (WM_COPYDATA from other processes) — layout must match
/// the PSDK byte-for-byte on every arch.
#[repr(C)]
#[derive(Clone, Copy)]
struct NOTIFYICONDATAW {
    cbSize: DWORD,
    hWnd: HWND,
    uID: UINT,
    uFlags: UINT,
    uCallbackMessage: UINT,
    hIcon: HICON,
    szTip: [u16; 128],
    dwState: DWORD,
    dwStateMask: DWORD,
    szInfo: [u16; 256],
    uTimeout: UINT, // union { uTimeout; uVersion }
    szInfoTitle: [u16; 64],
    dwInfoFlags: DWORD,
    guidItem: GUID,
    hBalloonIcon: HICON,
}

/// `struct InternalIconData : NOTIFYICONDATA { UINT uVersionCopy; }` — flattened
/// (the base fields inline first, then the extra member) since it is never
/// passed to a Win32 API as a NOTIFYICONDATA*; it is internal button storage.
#[repr(C)]
struct InternalIconData {
    cbSize: DWORD,
    hWnd: HWND,
    uID: UINT,
    uFlags: UINT,
    uCallbackMessage: UINT,
    hIcon: HICON,
    szTip: [u16; 128],
    dwState: DWORD,
    dwStateMask: DWORD,
    szInfo: [u16; 256],
    uTimeout: UINT,
    szInfoTitle: [u16; 64],
    dwInfoFlags: DWORD,
    guidItem: GUID,
    hBalloonIcon: HICON,
    uVersionCopy: UINT,
    /// Set once RemoveButton starts tearing the item down. Closing its balloon sends
    /// NIN_BALLOONHIDE to the owner, which may answer with NIM_DELETE for the same
    /// icon; without this the re-entered RemoveButton freed the item twice
    /// (upstream 8d44ccdd5f7, CORE-20685, which packs this into cbSize's high word).
    bRemoved: BOOL,
}

/// `TRAYNOTIFYDATAW` (undocshell.h) — the WM_COPYDATA payload.
#[repr(C)]
struct TRAYNOTIFYDATAW {
    dwSignature: DWORD,
    dwMessage: DWORD,
    nid: NOTIFYICONDATAW,
}

// ---- extern surface (comctl32 / kernel32 / msvcrt) ----

extern "system" {
    fn ImageList_Create(cx: i32, cy: i32, flags: UINT, cInitial: i32, cGrow: i32) -> HIMAGELIST;
    fn ImageList_AddIcon(himl: HIMAGELIST, hicon: HICON) -> i32;
    fn ImageList_ReplaceIcon(himl: HIMAGELIST, i: i32, hicon: HICON) -> i32;
    fn ImageList_Remove(himl: HIMAGELIST, i: i32) -> BOOL;
    fn ImageList_Destroy(himl: HIMAGELIST) -> BOOL;
    fn ImageList_GetIconSize(himl: HIMAGELIST, cx: *mut i32, cy: *mut i32) -> BOOL;

    fn InitializeCriticalSection(lpCriticalSection: *mut CRITICAL_SECTION);
    fn EnterCriticalSection(lpCriticalSection: *mut CRITICAL_SECTION);
    fn LeaveCriticalSection(lpCriticalSection: *mut CRITICAL_SECTION);

    fn MulDiv(nNumber: i32, nNumerator: i32, nDenominator: i32) -> i32;
}

extern "C" {
    // msvcrt: uintptr_t _beginthreadex(void*, unsigned, unsigned(__stdcall*)(void*),
    //                                  void*, unsigned, unsigned*);
    fn _beginthreadex(
        security: *mut c_void,
        stack_size: u32,
        start_address: Option<unsafe extern "system" fn(*mut c_void) -> u32>,
        arglist: *mut c_void,
        initflag: u32,
        thrdaddr: *mut u32,
    ) -> usize;
}

// SetWindowLongPtrW is a C macro on i386 (== SetWindowLongW); a real import on
// LLP64. Declare it only for the non-x86 subclass path.
#[cfg(not(target_arch = "x86"))]
extern "system" {
    fn SetWindowLongPtrW(hWnd: HWND, nIndex: i32, dwNewLong: LONG_PTR) -> LONG_PTR;
}

// ---- small helpers ----

#[inline]
unsafe fn guid_eq(a: *const IID, b: *const IID) -> bool {
    let a = &*a;
    let b = &*b;
    a.Data1 == b.Data1 && a.Data2 == b.Data2 && a.Data3 == b.Data3 && a.Data4 == b.Data4
}

#[inline]
fn make_lparam(lo: u16, hi: u16) -> LPARAM {
    ((lo as u32) | ((hi as u32) << 16)) as i32 as LPARAM
}

#[inline]
fn loword(v: usize) -> u16 {
    (v & 0xFFFF) as u16
}
#[inline]
fn hiword(v: usize) -> u16 {
    ((v >> 16) & 0xFFFF) as u16
}

#[inline]
unsafe fn heap_zeroed<T>() -> *mut T {
    let p = HeapAlloc(hProcessHeap, 0, size_of::<T>() as SIZE_T) as *mut T;
    if !p.is_null() {
        write_bytes(p as *mut u8, 0, size_of::<T>());
    }
    p
}

#[inline]
unsafe fn heap_free<T>(p: *mut T) {
    if !p.is_null() {
        HeapFree(hProcessHeap, 0, p as *mut c_void);
    }
}

/// strsafe `StringCchCopyW` — bounded copy incl. NUL (cch = element capacity).
unsafe fn string_cch_copy(dst: *mut u16, cch: usize, src: *const u16) {
    if cch == 0 {
        return;
    }
    let mut i = 0usize;
    while i < cch - 1 && *src.add(i) != 0 {
        *dst.add(i) = *src.add(i);
        i += 1;
    }
    *dst.add(i) = 0;
}

/// strsafe `StringCchLength` — returns element length (bounded).
unsafe fn string_cch_length(s: *const u16, cch_max: usize) -> usize {
    let mut n = 0usize;
    while n < cch_max && *s.add(n) != 0 {
        n += 1;
    }
    n
}

/// CWindow::ClientToScreen(LPRECT) — maps both corners.
unsafe fn client_to_screen_rect(hwnd: HWND, rc: *mut RECT) {
    let mut tl = POINT {
        x: (*rc).left,
        y: (*rc).top,
    };
    let mut br = POINT {
        x: (*rc).right,
        y: (*rc).bottom,
    };
    ClientToScreen(hwnd, &mut tl);
    ClientToScreen(hwnd, &mut br);
    (*rc).left = tl.x;
    (*rc).top = tl.y;
    (*rc).right = br.x;
    (*rc).bottom = br.y;
}

// ===========================================================================
// CIconWatcher list — CAtlList<IconWatcherData*> (doubly-linked, POSITION=node)
// ===========================================================================

struct IconWatcherData {
    hProcess: HANDLE,
    ProcessId: DWORD,
    IconData: NOTIFYICONDATAW,
}

#[repr(C)]
struct PtrNode {
    next: *mut PtrNode,
    prev: *mut PtrNode,
    data: *mut IconWatcherData,
}

#[repr(C)]
struct PtrList {
    head: *mut PtrNode,
    tail: *mut PtrNode,
    count: usize,
}

impl PtrList {
    fn zeroed() -> PtrList {
        PtrList {
            head: null_mut(),
            tail: null_mut(),
            count: 0,
        }
    }
    unsafe fn get_count(&self) -> usize {
        self.count
    }
    unsafe fn add_tail(&mut self, data: *mut IconWatcherData) {
        let node = heap_zeroed::<PtrNode>();
        if node.is_null() {
            return;
        }
        (*node).data = data;
        (*node).prev = self.tail;
        (*node).next = null_mut();
        if self.tail.is_null() {
            self.head = node;
        } else {
            (*self.tail).next = node;
        }
        self.tail = node;
        self.count += 1;
    }
    unsafe fn get_head_position(&self) -> *mut PtrNode {
        self.head
    }
    /// CAtlList::GetNext — returns the element at `*pos` and advances `*pos`.
    unsafe fn get_next(&self, pos: *mut *mut PtrNode) -> *mut IconWatcherData {
        let node = *pos;
        if node.is_null() {
            return null_mut();
        }
        *pos = (*node).next;
        (*node).data
    }
    /// CAtlList::FindIndex(i) — POSITION of the i-th element.
    unsafe fn find_index(&self, i: usize) -> *mut PtrNode {
        let mut node = self.head;
        let mut k = 0usize;
        while !node.is_null() && k < i {
            node = (*node).next;
            k += 1;
        }
        node
    }
    unsafe fn get_at(&self, node: *mut PtrNode) -> *mut IconWatcherData {
        if node.is_null() {
            null_mut()
        } else {
            (*node).data
        }
    }
    /// CAtlList::RemoveAt(POSITION) — unlink+free the node (NOT the data).
    unsafe fn remove_at(&mut self, node: *mut PtrNode) {
        if node.is_null() {
            return;
        }
        if !(*node).prev.is_null() {
            (*(*node).prev).next = (*node).next;
        } else {
            self.head = (*node).next;
        }
        if !(*node).next.is_null() {
            (*(*node).next).prev = (*node).prev;
        } else {
            self.tail = (*node).prev;
        }
        self.count -= 1;
        heap_free(node);
    }
    /// CAtlList::RemoveAll — free all nodes (not the data).
    unsafe fn remove_all(&mut self) {
        let mut node = self.head;
        while !node.is_null() {
            let next = (*node).next;
            heap_free(node);
            node = next;
        }
        self.head = null_mut();
        self.tail = null_mut();
        self.count = 0;
    }
}

// ===========================================================================
// CBalloonQueue list — CAtlList<Info> (nodes embed Info by value)
// ===========================================================================

#[repr(C)]
#[derive(Clone, Copy)]
struct Info {
    pSource: *mut InternalIconData,
    szInfo: [u16; 256],
    szInfoTitle: [u16; 64],
    uIcon: WPARAM,
    uTimeout: UINT,
}

impl Info {
    /// Info(InternalIconData* source).
    unsafe fn from_source(source: *mut InternalIconData) -> Info {
        let mut info: Info = core::mem::zeroed();
        info.pSource = source;
        string_cch_copy(info.szInfo.as_mut_ptr(), 256, (*source).szInfo.as_ptr());
        string_cch_copy(
            info.szInfoTitle.as_mut_ptr(),
            64,
            (*source).szInfoTitle.as_ptr(),
        );
        info.uIcon = ((*source).dwInfoFlags & NIIF_ICON_MASK) as WPARAM;
        if (*source).dwInfoFlags == NIIF_USER {
            info.uIcon = (*source).hIcon as WPARAM;
        }
        info.uTimeout = (*source).uTimeout;
        info
    }
}

#[repr(C)]
struct InfoNode {
    next: *mut InfoNode,
    prev: *mut InfoNode,
    data: Info,
}

#[repr(C)]
struct InfoList {
    head: *mut InfoNode,
    tail: *mut InfoNode,
    count: usize,
}

impl InfoList {
    fn zeroed() -> InfoList {
        InfoList {
            head: null_mut(),
            tail: null_mut(),
            count: 0,
        }
    }
    unsafe fn is_empty(&self) -> bool {
        self.count == 0
    }
    unsafe fn add_tail(&mut self, info: Info) {
        let node = heap_zeroed::<InfoNode>();
        if node.is_null() {
            return;
        }
        (*node).data = info;
        (*node).prev = self.tail;
        (*node).next = null_mut();
        if self.tail.is_null() {
            self.head = node;
        } else {
            (*self.tail).next = node;
        }
        self.tail = node;
        self.count += 1;
    }
    unsafe fn remove_head(&mut self) -> Info {
        let node = self.head;
        let info = (*node).data;
        self.head = (*node).next;
        if self.head.is_null() {
            self.tail = null_mut();
        } else {
            (*self.head).prev = null_mut();
        }
        self.count -= 1;
        heap_free(node);
        info
    }
    unsafe fn get_head_position(&self) -> *mut InfoNode {
        self.head
    }
    /// CAtlList::GetNext — returns &element at `*pos`, advances `*pos`.
    unsafe fn get_next(&self, pos: *mut *mut InfoNode) -> *mut Info {
        let node = *pos;
        if node.is_null() {
            return null_mut();
        }
        *pos = (*node).next;
        &mut (*node).data
    }
    unsafe fn remove_at(&mut self, node: *mut InfoNode) {
        // Faithful to CAtlList::RemoveAt, but guarded against NULL (the pre-fix
        // C++ RemoveInfo loop passed an advanced-past-end POSITION; upstream
        // 8d44ccdd5f7 fixed the loop, bq_remove_info follows it).
        if node.is_null() {
            return;
        }
        if !(*node).prev.is_null() {
            (*(*node).prev).next = (*node).next;
        } else {
            self.head = (*node).next;
        }
        if !(*node).next.is_null() {
            (*(*node).next).prev = (*node).prev;
        } else {
            self.tail = (*node).prev;
        }
        self.count -= 1;
        heap_free(node);
    }
}

// ===========================================================================
// The object (ATL bases flattened). CIconWatcher base + CSysPagerWnd members.
// ===========================================================================

#[repr(C)]
pub struct CNotifyToolbar {
    m_hWnd: HWND,
    m_pfnSuperWindowProc: WNDPROC,
    m_ImageList: HIMAGELIST,
    m_VisibleButtonCount: i32,
    m_BalloonQueue: *mut CBalloonQueue,
}

#[repr(C)]
pub struct CBalloonQueue {
    m_hwndParent: HWND,
    m_tooltips: *mut HWND, // CTooltips* → &m_Balloons (its HWND)
    m_toolbar: *mut CNotifyToolbar,
    m_queue: InfoList,
    m_current: *mut InternalIconData,
    m_timer: i32,
}

#[repr(C)]
pub struct CSysPagerWnd {
    // COM identity — IOleWindow vtbl first (IUnknown shares it).
    lpVtbl: *const IOleWindowVtbl,
    ref_: LONG,
    m_hWnd: HWND,
    // CIconWatcher base members
    iw_WatcherList: PtrList,
    iw_ListLock: CRITICAL_SECTION,
    iw_hWatcherThread: HANDLE,
    iw_WakeUpEvent: HANDLE,
    iw_hwndSysTray: HWND,
    iw_Loop: bool,
    // CSysPagerWnd members
    Toolbar: CNotifyToolbar,
    m_Balloons: HWND, // CTooltips → its HWND
    m_BalloonQueue: CBalloonQueue,
}

// CBalloonQueue constants
const BQ_TIMER_INTERVAL: i32 = 2000;
const BQ_BALLOONS_TIMER_ID: UINT_PTR = 1;
const BQ_MIN_TIMEOUT: i32 = 10000;
const BQ_MAX_TIMEOUT: i32 = 30000;
const BQ_COOLDOWN: i32 = 2000;

// ===========================================================================
// COM: IUnknown / IOleWindow (BEGIN_COM_MAP + CComObject) — single interface.
// ===========================================================================

#[no_mangle]
pub unsafe extern "system" fn CSysPagerWnd_QueryInterface(
    iface: *mut IOleWindow,
    riid: REFIID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    let this = iface as *mut CSysPagerWnd;
    if ppv.is_null() {
        return E_POINTER;
    }
    if guid_eq(riid, &IID_IUnknown) || guid_eq(riid, &IID_IOleWindow) {
        *ppv = this as *mut c_void;
        CSysPagerWnd_AddRef(iface);
        return S_OK;
    }
    *ppv = null_mut();
    E_NOINTERFACE
}

#[no_mangle]
pub unsafe extern "system" fn CSysPagerWnd_AddRef(iface: *mut IOleWindow) -> u32 {
    let this = iface as *mut CSysPagerWnd;
    interlocked_increment(&mut (*this).ref_) as u32
}

#[no_mangle]
pub unsafe extern "system" fn CSysPagerWnd_Release(iface: *mut IOleWindow) -> u32 {
    let this = iface as *mut CSysPagerWnd;
    let n = interlocked_decrement(&mut (*this).ref_);
    if n == 0 {
        CSysPagerWnd_delete(this);
        return 0;
    }
    n as u32
}

/// IOleWindow::GetWindow (inline).
#[no_mangle]
pub unsafe extern "system" fn CSysPagerWnd_GetWindow(
    iface: *mut IOleWindow,
    phwnd: *mut HWND,
) -> HRESULT {
    let this = iface as *mut CSysPagerWnd;
    if phwnd.is_null() {
        return E_INVALIDARG;
    }
    *phwnd = (*this).m_hWnd;
    S_OK
}

/// IOleWindow::ContextSensitiveHelp (inline).
#[no_mangle]
pub unsafe extern "system" fn CSysPagerWnd_ContextSensitiveHelp(
    _iface: *mut IOleWindow,
    _f_enter_mode: BOOL,
) -> HRESULT {
    E_NOTIMPL
}

static vt_CSysPagerWnd: IOleWindowVtbl = IOleWindowVtbl {
    QueryInterface: CSysPagerWnd_QueryInterface,
    AddRef: CSysPagerWnd_AddRef,
    Release: CSysPagerWnd_Release,
    GetWindow: CSysPagerWnd_GetWindow,
    ContextSensitiveHelp: CSysPagerWnd_ContextSensitiveHelp,
};

// ===========================================================================
// construction / destruction
// ===========================================================================

unsafe fn CSysPagerWnd_ctor(this: *mut CSysPagerWnd) {
    (*this).lpVtbl = &vt_CSysPagerWnd;
    (*this).ref_ = 0;
    (*this).m_hWnd = null_mut();
    // CIconWatcher()
    (*this).iw_WatcherList = PtrList::zeroed();
    (*this).iw_ListLock = core::mem::zeroed();
    (*this).iw_hWatcherThread = null_mut();
    (*this).iw_WakeUpEvent = null_mut();
    (*this).iw_hwndSysTray = null_mut();
    (*this).iw_Loop = false;
    // CNotifyToolbar()
    (*this).Toolbar = CNotifyToolbar {
        m_hWnd: null_mut(),
        // ATL sets ::DefWindowProc; only ever read after SubclassWindow
        // overwrites it with the real toolbar proc, so None is safe here.
        m_pfnSuperWindowProc: None,
        m_ImageList: null_mut(),
        m_VisibleButtonCount: 0,
        m_BalloonQueue: null_mut(),
    };
    // CTooltips m_Balloons
    (*this).m_Balloons = null_mut();
    // CBalloonQueue()
    (*this).m_BalloonQueue = CBalloonQueue {
        m_hwndParent: null_mut(),
        m_tooltips: null_mut(),
        m_toolbar: null_mut(),
        m_queue: InfoList::zeroed(),
        m_current: null_mut(),
        m_timer: -1,
    };
}

unsafe fn CSysPagerWnd_new() -> *mut CSysPagerWnd {
    let p = HeapAlloc(hProcessHeap, 0, size_of::<CSysPagerWnd>() as SIZE_T) as *mut CSysPagerWnd;
    if !p.is_null() {
        CSysPagerWnd_ctor(p);
    }
    p
}

unsafe fn CSysPagerWnd_delete(this: *mut CSysPagerWnd) {
    if this.is_null() {
        return;
    }
    // ~CSysPagerWnd() is empty; ~CIconWatcher() runs Uninitialize + tears down
    // the critical section and handles.
    iw_uninitialize(this);
    DeleteCriticalSection(&mut (*this).iw_ListLock);
    if !(*this).iw_WakeUpEvent.is_null() {
        CloseHandle((*this).iw_WakeUpEvent);
    }
    if !(*this).iw_hWatcherThread.is_null() {
        CloseHandle((*this).iw_hWatcherThread);
    }
    HeapFree(hProcessHeap, 0, this as *mut c_void);
}

// ===========================================================================
// CIconWatcher (free fns over the flattened base; `this` = *mut CSysPagerWnd)
// ===========================================================================

unsafe fn iw_initialize(this: *mut CSysPagerWnd, hWndParent: HWND) -> bool {
    (*this).iw_hwndSysTray = hWndParent;

    InitializeCriticalSection(&mut (*this).iw_ListLock);
    (*this).iw_WakeUpEvent =
        CreateEventW(null_mut(), FALSE as BOOL, FALSE as BOOL, core::ptr::null());
    if (*this).iw_WakeUpEvent.is_null() {
        return false;
    }

    (*this).iw_hWatcherThread = _beginthreadex(
        null_mut(),
        0,
        Some(iw_watcher_thread),
        this as *mut c_void,
        0,
        null_mut(),
    ) as HANDLE;
    if (*this).iw_hWatcherThread.is_null() {
        return false;
    }

    true
}

unsafe fn iw_uninitialize(this: *mut CSysPagerWnd) {
    (*this).iw_Loop = false;
    if !(*this).iw_WakeUpEvent.is_null() {
        SetEvent((*this).iw_WakeUpEvent);
    }

    EnterCriticalSection(&mut (*this).iw_ListLock);

    let count = (*this).iw_WatcherList.get_count();
    for i in 0..count {
        let pos = (*this).iw_WatcherList.find_index(i);
        if !pos.is_null() {
            let icon = (*this).iw_WatcherList.get_at(pos);
            iwd_delete(icon);
        }
    }
    (*this).iw_WatcherList.remove_all();

    LeaveCriticalSection(&mut (*this).iw_ListLock);
}

/// IconWatcherData(const NOTIFYICONDATA*) + `new`.
unsafe fn iwd_new(icon_data: *const NOTIFYICONDATAW) -> *mut IconWatcherData {
    let p = heap_zeroed::<IconWatcherData>();
    if p.is_null() {
        return p;
    }
    (*p).hProcess = null_mut();
    (*p).ProcessId = 0;
    (*p).IconData.cbSize = size_of::<NOTIFYICONDATAW>() as DWORD;
    (*p).IconData.hWnd = (*icon_data).hWnd;
    (*p).IconData.uID = (*icon_data).uID;
    (*p).IconData.guidItem = (*icon_data).guidItem;
    p
}

unsafe fn iwd_delete(icon: *mut IconWatcherData) {
    if icon.is_null() {
        return;
    }
    if !(*icon).hProcess.is_null() {
        CloseHandle((*icon).hProcess);
    }
    heap_free(icon);
}

unsafe fn iw_add_icon_to_watcher(
    this: *mut CSysPagerWnd,
    icon_data: *const NOTIFYICONDATAW,
) -> bool {
    let mut process_id: DWORD = 0;
    GetWindowThreadProcessId((*icon_data).hWnd, &mut process_id);

    let h_process = OpenProcess(SYNCHRONIZE, FALSE as BOOL, process_id);
    if h_process.is_null() {
        return false;
    }

    let icon = iwd_new(icon_data);
    if icon.is_null() {
        CloseHandle(h_process);
        return false;
    }
    (*icon).hProcess = h_process;
    (*icon).ProcessId = process_id;

    let mut added = false;
    EnterCriticalSection(&mut (*this).iw_ListLock);

    if (*this).iw_WatcherList.get_count() < MAXIMUM_WAIT_OBJECTS as usize {
        (*this).iw_WatcherList.add_tail(icon);
        SetEvent((*this).iw_WakeUpEvent);
        added = true;
    }

    LeaveCriticalSection(&mut (*this).iw_ListLock);

    if !added {
        iwd_delete(icon);
    }

    added
}

unsafe fn iw_remove_icon_from_watcher(
    this: *mut CSysPagerWnd,
    icon_data: *const NOTIFYICONDATAW,
) -> bool {
    EnterCriticalSection(&mut (*this).iw_ListLock);

    let icon = iw_get_list_entry(this, icon_data, null_mut(), true);

    SetEvent((*this).iw_WakeUpEvent);
    LeaveCriticalSection(&mut (*this).iw_ListLock);

    iwd_delete(icon);
    true
}

unsafe fn iw_get_list_entry(
    this: *mut CSysPagerWnd,
    icon_data: *const NOTIFYICONDATAW,
    h_process: HANDLE,
    remove: bool,
) -> *mut IconWatcherData {
    let mut entry: *mut IconWatcherData;
    let mut next_position = (*this).iw_WatcherList.get_head_position();
    let mut position: *mut PtrNode;
    loop {
        position = next_position;

        entry = (*this).iw_WatcherList.get_next(&mut next_position);
        if !entry.is_null() {
            let by_icon = !icon_data.is_null()
                && (*entry).IconData.hWnd == (*icon_data).hWnd
                && (*entry).IconData.uID == (*icon_data).uID;
            let by_proc = !h_process.is_null() && (*entry).hProcess == h_process;
            if by_icon || by_proc {
                if remove {
                    (*this).iw_WatcherList.remove_at(position);
                }
                break;
            }
        }
        entry = null_mut();

        if next_position.is_null() {
            break;
        }
    }

    entry
}

/// UINT WINAPI CIconWatcher::WatcherThread(LPVOID).
unsafe extern "system" fn iw_watcher_thread(lp_param: *mut c_void) -> u32 {
    let this = lp_param as *mut CSysPagerWnd;
    let mut watch_list: *mut HANDLE = null_mut();

    (*this).iw_Loop = true;
    while (*this).iw_Loop {
        EnterCriticalSection(&mut (*this).iw_ListLock);

        let size = (*this).iw_WatcherList.get_count() + 1;

        if !watch_list.is_null() {
            heap_free(watch_list);
        }
        watch_list =
            HeapAlloc(hProcessHeap, 0, (size * size_of::<HANDLE>()) as SIZE_T) as *mut HANDLE;
        *watch_list.add(0) = (*this).iw_WakeUpEvent;

        let count = (*this).iw_WatcherList.get_count();
        for i in 0..count {
            let pos = (*this).iw_WatcherList.find_index(i);
            if !pos.is_null() {
                let icon = (*this).iw_WatcherList.get_at(pos);
                *watch_list.add(i + 1) = (*icon).hProcess;
            }
        }

        LeaveCriticalSection(&mut (*this).iw_ListLock);

        let status = WaitForMultipleObjects(size as DWORD, watch_list, FALSE as BOOL, INFINITE);
        if status == WAIT_OBJECT_0 {
            // Kicked: list updated (or exiting). Nothing to do here.
        } else if status >= WAIT_OBJECT_0 + 1 && status < size as DWORD {
            let icon = iw_get_list_entry(
                this,
                core::ptr::null(),
                *watch_list.add(status as usize),
                false,
            );
            if icon.is_null() {
                continue;
            }

            let mut tnid: TRAYNOTIFYDATAW = core::mem::zeroed();
            tnid.dwSignature = NI_NOTIFY_SIG;
            tnid.dwMessage = NIM_DELETE;
            core::ptr::copy_nonoverlapping(
                &(*icon).IconData as *const NOTIFYICONDATAW as *const u8,
                &mut tnid.nid as *mut NOTIFYICONDATAW as *mut u8,
                (*icon).IconData.cbSize as usize,
            );

            let mut data: COPYDATASTRUCT = core::mem::zeroed();
            data.dwData = 1;
            data.cbData = size_of::<TRAYNOTIFYDATAW>() as DWORD;
            data.lpData = &mut tnid as *mut TRAYNOTIFYDATAW as *mut c_void;

            let success = SendMessageW(
                (*this).iw_hwndSysTray,
                WM_COPYDATA,
                &(*icon).IconData as *const NOTIFYICONDATAW as WPARAM,
                &data as *const COPYDATASTRUCT as LPARAM,
            );
            if success == 0 {
                iw_remove_icon_from_watcher(this, &(*icon).IconData);
            }
        } else {
            // WAIT_FAILED or other error → stop watching.
            let _ = if status == WAIT_FAILED {
                GetLastError()
            } else {
                status
            };
            iw_uninitialize(this);
        }
    }

    if !watch_list.is_null() {
        heap_free(watch_list);
    }

    0
}

// ===========================================================================
// CBalloonQueue
// ===========================================================================

unsafe fn bq_init(
    bq: *mut CBalloonQueue,
    hwnd_parent: HWND,
    toolbar: *mut CNotifyToolbar,
    balloons: *mut HWND,
) {
    (*bq).m_hwndParent = hwnd_parent;
    (*bq).m_toolbar = toolbar;
    (*bq).m_tooltips = balloons;
}

unsafe fn bq_deinit(bq: *mut CBalloonQueue) {
    if (*bq).m_timer >= 0 {
        KillTimer((*bq).m_hwndParent, (*bq).m_timer as UINT_PTR);
    }
}

unsafe fn bq_on_timer(bq: *mut CBalloonQueue, timer_id: i32) -> bool {
    if timer_id != (*bq).m_timer {
        return false;
    }

    KillTimer((*bq).m_hwndParent, (*bq).m_timer as UINT_PTR);
    (*bq).m_timer = -1;

    if !(*bq).m_current.is_null() {
        bq_close(bq, (*bq).m_current, NIN_BALLOONTIMEOUT);
    } else {
        if !(*bq).m_queue.is_empty() {
            let mut info = (*bq).m_queue.remove_head();
            bq_show(bq, &mut info);
        }
    }

    true
}

unsafe fn bq_update_info(bq: *mut CBalloonQueue, notify_item: *mut InternalIconData) {
    let len = string_cch_length((*notify_item).szInfo.as_ptr(), 256);
    if len > 0 {
        let mut info = Info::from_source(notify_item);

        // If m_current == notifyItem, replace the previous balloon even if
        // there is a queue.
        if (*bq).m_current != notify_item
            && (!(*bq).m_current.is_null() || !(*bq).m_queue.is_empty())
        {
            (*bq).m_queue.add_tail(info);
        } else {
            bq_show(bq, &mut info);
        }
    } else {
        bq_close(bq, notify_item, NIN_BALLOONHIDE);
    }
}

unsafe fn bq_remove_info(bq: *mut CBalloonQueue, notify_item: *mut InternalIconData) {
    bq_close(bq, notify_item, NIN_BALLOONHIDE);

    // get_next advances the cursor, so the node to drop is the one read, not the
    // cursor: the pre-fix loop unlinked the NEXT entry (or none, at the tail) and left
    // this item's pending balloon queued against memory about to be freed.
    let mut next = (*bq).m_queue.get_head_position();
    while !next.is_null() {
        let current = next;
        let info = (*bq).m_queue.get_next(&mut next);
        if !info.is_null() && (*info).pSource == notify_item {
            (*bq).m_queue.remove_at(current);
            break;
        }
    }
}

unsafe fn bq_close_current(bq: *mut CBalloonQueue) {
    if !(*bq).m_current.is_null() {
        bq_close(bq, (*bq).m_current, NIN_BALLOONTIMEOUT);
    }
}

unsafe fn bq_index_of(bq: *mut CBalloonQueue, pdata: *mut InternalIconData) -> i32 {
    let tb = (*bq).m_toolbar;
    let count = tb_get_button_count(tb);
    for i in 0..count {
        if tb_get_item_data(tb, i) == pdata {
            return i;
        }
    }
    -1
}

unsafe fn bq_set_timer(bq: *mut CBalloonQueue, length: i32) {
    (*bq).m_timer = SetTimer(
        (*bq).m_hwndParent,
        BQ_BALLOONS_TIMER_ID,
        length as UINT,
        None,
    ) as i32;
}

unsafe fn bq_show(bq: *mut CBalloonQueue, info: *mut Info) {
    // TODO: NIF_REALTIME, NIIF_NOSOUND, other Vista+ flags.
    (*bq).m_current = (*info).pSource;

    let tb = (*bq).m_toolbar;
    let mut rc: RECT = core::mem::zeroed();
    tb_get_item_rect(tb, bq_index_of(bq, (*bq).m_current), &mut rc);
    client_to_screen_rect((*tb).m_hWnd, &mut rc);
    let x = ((rc.left + rc.right) / 2) as u16;
    let y = ((rc.top + rc.bottom) / 2) as u16;

    let tooltip = *(*bq).m_tooltips;
    tt_set_title(tooltip, (*info).szInfoTitle.as_ptr(), (*info).uIcon);
    tt_track_position(tooltip, x, y);
    tt_set_max_tip_width(tooltip, BALLOON_MAXWIDTH);
    tt_update_tip_text(
        tooltip,
        (*bq).m_hwndParent,
        (*tb).m_hWnd as UINT_PTR,
        (*info).szInfo.as_ptr(),
    );
    tt_track_activate(tooltip, (*bq).m_hwndParent, (*tb).m_hWnd as UINT_PTR);

    let mut timeout = (*info).uTimeout as i32;
    if timeout < BQ_MIN_TIMEOUT {
        timeout = BQ_MIN_TIMEOUT;
    }
    if timeout > BQ_MAX_TIMEOUT {
        timeout = BQ_MAX_TIMEOUT;
    }

    bq_set_timer(bq, timeout);

    tb_send_notify_callback(tb, (*bq).m_current, NIN_BALLOONSHOW);
}

unsafe fn bq_close(bq: *mut CBalloonQueue, notify_item: *mut InternalIconData, u_reason: UINT) {
    if (*bq).m_current == notify_item {
        tb_send_notify_callback((*bq).m_toolbar, (*bq).m_current, u_reason);

        // Prevent re-entry (TrackDeactivate generates a TTN_POP). Clearing m_current,
        // rather than flagging it closed, also stops a later timer or queue step from
        // touching an item RemoveButton is about to free (upstream 8d44ccdd5f7).
        (*bq).m_current = null_mut();
        tt_track_deactivate(*(*bq).m_tooltips);
        bq_set_timer(bq, BQ_COOLDOWN);
    }
}

// ===========================================================================
// CTooltips wrappers (rosctrls.h) — thin TTM_* SendMessage forwards.
// ===========================================================================

unsafe fn tt_set_title(tooltip: HWND, title: LPCWSTR, icon: WPARAM) {
    SendMessageW(tooltip, TTM_SETTITLEW, icon, title as LPARAM);
}
unsafe fn tt_track_position(tooltip: HWND, x: u16, y: u16) {
    SendMessageW(tooltip, TTM_TRACKPOSITION, 0, make_lparam(x, y));
}
unsafe fn tt_set_max_tip_width(tooltip: HWND, width: i32) {
    SendMessageW(tooltip, TTM_SETMAXTIPWIDTH, 0, width as LPARAM);
}
unsafe fn tt_update_tip_text(tooltip: HWND, hwnd_owner: HWND, u_id: UINT_PTR, text: LPCWSTR) {
    let mut info: TOOLINFOW = core::mem::zeroed();
    info.cbSize = size_of::<TOOLINFOW>() as UINT;
    info.hwnd = hwnd_owner;
    info.uId = u_id;
    info.lpszText = text as LPWSTR;
    info.hinst = null_mut();
    SendMessageW(
        tooltip,
        TTM_UPDATETIPTEXTW,
        0,
        &info as *const TOOLINFOW as LPARAM,
    );
}
unsafe fn tt_track_activate(tooltip: HWND, hwnd_owner: HWND, u_id: UINT_PTR) {
    let mut info: TOOLINFOW = core::mem::zeroed();
    info.cbSize = size_of::<TOOLINFOW>() as UINT;
    info.hwnd = hwnd_owner;
    info.uId = u_id;
    SendMessageW(
        tooltip,
        TTM_TRACKACTIVATE,
        TRUE as WPARAM,
        &info as *const TOOLINFOW as LPARAM,
    );
}
unsafe fn tt_track_deactivate(tooltip: HWND) {
    SendMessageW(tooltip, TTM_TRACKACTIVATE, FALSE as WPARAM, 0);
}

// ===========================================================================
// CNotifyToolbar — CToolbar<InternalIconData> template helpers (SendMessage
// forwards) + the subclassed message map.
// ===========================================================================

unsafe fn tb_get_button_count(tb: *mut CNotifyToolbar) -> i32 {
    SendMessageW((*tb).m_hWnd, TB_BUTTONCOUNT, 0, 0) as i32
}
unsafe fn tb_get_button(tb: *mut CNotifyToolbar, index: i32, btn: *mut TBBUTTON) -> DWORD {
    SendMessageW((*tb).m_hWnd, TB_GETBUTTON, index as WPARAM, btn as LPARAM) as DWORD
}
unsafe fn tb_add_button(tb: *mut CNotifyToolbar, btn: *mut TBBUTTON) -> DWORD {
    SendMessageW((*tb).m_hWnd, TB_ADDBUTTONSW, 1, btn as LPARAM) as DWORD
}
unsafe fn tb_delete_button(tb: *mut CNotifyToolbar, index: i32) -> DWORD {
    SendMessageW((*tb).m_hWnd, TB_DELETEBUTTON, index as WPARAM, 0) as DWORD
}
unsafe fn tb_set_button_info(
    tb: *mut CNotifyToolbar,
    cmd_id: i32,
    info: *mut TBBUTTONINFOW,
) -> DWORD {
    SendMessageW(
        (*tb).m_hWnd,
        TB_SETBUTTONINFOW,
        cmd_id as WPARAM,
        info as LPARAM,
    ) as DWORD
}
unsafe fn tb_set_button_size(tb: *mut CNotifyToolbar, w: i32, h: i32) -> DWORD {
    SendMessageW(
        (*tb).m_hWnd,
        TB_SETBUTTONSIZE,
        0,
        make_lparam(w as u16, h as u16),
    ) as DWORD
}
unsafe fn tb_auto_size(tb: *mut CNotifyToolbar) -> DWORD {
    SendMessageW((*tb).m_hWnd, TB_AUTOSIZE, 0, 0) as DWORD
}
unsafe fn tb_set_metrics(tb: *mut CNotifyToolbar, tbm: *mut TBMETRICS) -> DWORD {
    SendMessageW((*tb).m_hWnd, TB_SETMETRICS, 0, tbm as LPARAM) as DWORD
}
unsafe fn tb_get_item_rect(tb: *mut CNotifyToolbar, index: i32, rc: *mut RECT) -> DWORD {
    SendMessageW((*tb).m_hWnd, TB_GETITEMRECT, index as WPARAM, rc as LPARAM) as DWORD
}
unsafe fn tb_hit_test(tb: *mut CNotifyToolbar, ppt: *mut POINT) -> i32 {
    SendMessageW((*tb).m_hWnd, TB_HITTEST, 0, ppt as LPARAM) as i32
}
unsafe fn tb_get_hot_item(tb: *mut CNotifyToolbar) -> i32 {
    SendMessageW((*tb).m_hWnd, TB_GETHOTITEM, 0, 0) as i32
}
unsafe fn tb_set_image_list(tb: *mut CNotifyToolbar, himl: HIMAGELIST) -> HIMAGELIST {
    SendMessageW((*tb).m_hWnd, TB_SETIMAGELIST, 0, himl as LPARAM) as HIMAGELIST
}
unsafe fn tb_get_item_data(tb: *mut CNotifyToolbar, index: i32) -> *mut InternalIconData {
    let mut btn: TBBUTTON = core::mem::zeroed();
    tb_get_button(tb, index, &mut btn);
    btn.dwData as *mut InternalIconData
}
unsafe fn tb_set_focus(tb: *mut CNotifyToolbar) {
    SetFocus((*tb).m_hWnd);
}

unsafe fn tb_get_visible_button_count(tb: *mut CNotifyToolbar) -> i32 {
    (*tb).m_VisibleButtonCount
}

unsafe fn tb_find_item(
    tb: *mut CNotifyToolbar,
    h_wnd: HWND,
    u_id: UINT,
    pdata: *mut *mut InternalIconData,
) -> i32 {
    let count = tb_get_button_count(tb);
    for i in 0..count {
        let data = tb_get_item_data(tb, i);
        if (*data).hWnd == h_wnd && (*data).uID == u_id {
            if !pdata.is_null() {
                *pdata = data;
            }
            return i;
        }
    }
    -1
}

unsafe fn tb_find_existing_shared_icon(tb: *mut CNotifyToolbar, handle: HICON) -> i32 {
    let count = tb_get_button_count(tb);
    for i in 0..count {
        let data = tb_get_item_data(tb, i);
        if (*data).hIcon == handle {
            let mut btn: TBBUTTON = core::mem::zeroed();
            tb_get_button(tb, i, &mut btn);
            return btn.iBitmap;
        }
    }
    -1
}

unsafe fn tb_add_button_notify(tb: *mut CNotifyToolbar, icon_data: *const NOTIFYICONDATAW) -> BOOL {
    let mut tb_btn: TBBUTTON = core::mem::zeroed();
    let text: [u16; 1] = [0];

    let mut notify_item: *mut InternalIconData = null_mut();
    let index = tb_find_item(tb, (*icon_data).hWnd, (*icon_data).uID, &mut notify_item);
    if index >= 0 {
        return FALSE as BOOL;
    }

    notify_item = heap_zeroed::<InternalIconData>();

    (*notify_item).hWnd = (*icon_data).hWnd;
    (*notify_item).uID = (*icon_data).uID;

    tb_btn.fsState = TBSTATE_ENABLED;
    tb_btn.fsStyle = BTNS_NOPREFIX;
    tb_btn.dwData = notify_item as usize;
    tb_btn.iString = text.as_ptr() as isize;
    tb_btn.idCommand = tb_get_button_count(tb);
    tb_btn.iBitmap = -1;

    if (*icon_data).uFlags & NIF_STATE != 0 {
        (*notify_item).dwState = (*icon_data).dwState & (*icon_data).dwStateMask;
    }

    if (*icon_data).uFlags & NIF_MESSAGE != 0 {
        (*notify_item).uCallbackMessage = (*icon_data).uCallbackMessage;
    }

    if (*icon_data).uFlags & NIF_ICON != 0 {
        (*notify_item).hIcon = (*icon_data).hIcon;
        let has_shared_icon = (*notify_item).dwState & NIS_SHAREDICON != 0;
        if has_shared_icon {
            let i_icon = tb_find_existing_shared_icon(tb, (*notify_item).hIcon);
            if i_icon < 0 {
                (*notify_item).hIcon = null_mut();
            }
            tb_btn.iBitmap = i_icon;
        } else {
            tb_btn.iBitmap = ImageList_AddIcon((*tb).m_ImageList, (*notify_item).hIcon);
        }
    }

    if (*icon_data).uFlags & NIF_TIP != 0 {
        string_cch_copy(
            (*notify_item).szTip.as_mut_ptr(),
            128,
            (*icon_data).szTip.as_ptr(),
        );
    }

    if (*icon_data).uFlags & NIF_INFO != 0 {
        string_cch_copy(
            (*notify_item).szInfo.as_mut_ptr(),
            256,
            (*icon_data).szInfo.as_ptr(),
        );
        string_cch_copy(
            (*notify_item).szInfoTitle.as_mut_ptr(),
            64,
            (*icon_data).szInfoTitle.as_ptr(),
        );
        (*notify_item).dwInfoFlags = (*icon_data).dwInfoFlags;
        (*notify_item).uTimeout = (*icon_data).uTimeout;
    }

    if (*notify_item).dwState & NIS_HIDDEN != 0 {
        tb_btn.fsState |= TBSTATE_HIDDEN;
    } else {
        (*tb).m_VisibleButtonCount += 1;
    }

    /* TODO: support VERSION_4 (NIF_GUID, NIF_REALTIME, NIF_SHOWTIP) */

    tb_add_button(tb, &mut tb_btn);
    tb_set_button_size(
        tb,
        GetSystemMetrics(SM_CXSMICON as i32),
        GetSystemMetrics(SM_CYSMICON as i32),
    );

    if (*icon_data).uFlags & NIF_INFO != 0 {
        bq_update_info((*tb).m_BalloonQueue, notify_item);
    }

    TRUE as BOOL
}

unsafe fn tb_switch_version(tb: *mut CNotifyToolbar, icon_data: *const NOTIFYICONDATAW) -> BOOL {
    let mut notify_item: *mut InternalIconData = null_mut();
    let index = tb_find_item(tb, (*icon_data).hWnd, (*icon_data).uID, &mut notify_item);
    if index < 0 {
        return FALSE as BOOL;
    }

    // uVersion is union'd with uTimeout in the incoming data.
    if (*icon_data).uTimeout != 0 && (*icon_data).uTimeout != NOTIFYICON_VERSION {
        return FALSE as BOOL;
    }

    (*notify_item).uVersionCopy = (*icon_data).uTimeout;

    TRUE as BOOL
}

unsafe fn tb_update_button(tb: *mut CNotifyToolbar, icon_data: *const NOTIFYICONDATAW) -> BOOL {
    let mut notify_item: *mut InternalIconData = null_mut();
    let mut tbbi: TBBUTTONINFOW = core::mem::zeroed();

    let index = tb_find_item(tb, (*icon_data).hWnd, (*icon_data).uID, &mut notify_item);
    if index < 0 {
        // NIM_MODIFY on an unknown icon fails; it must not resurrect one (upstream 8d44ccdd5f7).
        return FALSE as BOOL;
    }

    let mut btn: TBBUTTON = core::mem::zeroed();
    tb_get_button(tb, index, &mut btn);
    let old_icon_index = btn.iBitmap;

    tbbi.cbSize = size_of::<TBBUTTONINFOW>() as UINT;
    tbbi.dwMask = TBIF_BYINDEX | TBIF_COMMAND;
    tbbi.idCommand = index;

    if (*icon_data).uFlags & NIF_STATE != 0 {
        if (*icon_data).dwStateMask & NIS_HIDDEN != 0
            && ((*notify_item).dwState & NIS_HIDDEN) != ((*icon_data).dwState & NIS_HIDDEN)
        {
            tbbi.dwMask |= TBIF_STATE;
            if (*icon_data).dwState & NIS_HIDDEN != 0 {
                tbbi.fsState |= TBSTATE_HIDDEN;
                (*tb).m_VisibleButtonCount -= 1;
            } else {
                tbbi.fsState &= !TBSTATE_HIDDEN;
                (*tb).m_VisibleButtonCount += 1;
            }
        }

        (*notify_item).dwState &= !(*icon_data).dwStateMask;
        (*notify_item).dwState |= (*icon_data).dwState & (*icon_data).dwStateMask;
    }

    if (*icon_data).uFlags & NIF_MESSAGE != 0 {
        (*notify_item).uCallbackMessage = (*icon_data).uCallbackMessage;
    }

    if (*icon_data).uFlags & NIF_ICON != 0 {
        let has_shared_icon = (*notify_item).dwState & NIS_SHAREDICON != 0;
        if has_shared_icon {
            let i_icon = tb_find_existing_shared_icon(tb, (*icon_data).hIcon);
            if i_icon >= 0 {
                (*notify_item).hIcon = (*icon_data).hIcon;
                tbbi.dwMask |= TBIF_IMAGE;
                tbbi.iImage = i_icon;
            }
        } else {
            (*notify_item).hIcon = (*icon_data).hIcon;
            tbbi.dwMask |= TBIF_IMAGE;
            tbbi.iImage =
                ImageList_ReplaceIcon((*tb).m_ImageList, old_icon_index, (*notify_item).hIcon);
        }
    }

    if (*icon_data).uFlags & NIF_TIP != 0 {
        string_cch_copy(
            (*notify_item).szTip.as_mut_ptr(),
            128,
            (*icon_data).szTip.as_ptr(),
        );
    }

    if (*icon_data).uFlags & NIF_INFO != 0 {
        string_cch_copy(
            (*notify_item).szInfo.as_mut_ptr(),
            256,
            (*icon_data).szInfo.as_ptr(),
        );
        string_cch_copy(
            (*notify_item).szInfoTitle.as_mut_ptr(),
            64,
            (*icon_data).szInfoTitle.as_ptr(),
        );
        (*notify_item).dwInfoFlags = (*icon_data).dwInfoFlags;
        (*notify_item).uTimeout = (*icon_data).uTimeout;
    }

    /* TODO: support VERSION_4 (NIF_GUID, NIF_REALTIME, NIF_SHOWTIP) */

    tb_set_button_info(tb, index, &mut tbbi);

    if (*icon_data).uFlags & NIF_INFO != 0 {
        bq_update_info((*tb).m_BalloonQueue, notify_item);
    }

    TRUE as BOOL
}

unsafe fn tb_remove_button(tb: *mut CNotifyToolbar, icon_data: *const NOTIFYICONDATAW) -> BOOL {
    let mut notify_item: *mut InternalIconData = null_mut();

    let index = tb_find_item(tb, (*icon_data).hWnd, (*icon_data).uID, &mut notify_item);
    if index < 0 {
        return FALSE as BOOL;
    }
    if (*notify_item).bRemoved != 0 {
        return FALSE as BOOL; // re-entered from the balloon close below
    }
    (*notify_item).bRemoved = TRUE as BOOL;

    if (*notify_item).dwState & NIS_HIDDEN == 0 {
        (*tb).m_VisibleButtonCount -= 1;
    }

    if (*notify_item).dwState & NIS_SHAREDICON == 0 {
        let mut btn: TBBUTTON = core::mem::zeroed();
        tb_get_button(tb, index, &mut btn);
        let old_icon_index = btn.iBitmap;
        ImageList_Remove((*tb).m_ImageList, old_icon_index);

        // Update other icons!
        let count = tb_get_button_count(tb);
        for i in 0..count {
            let mut btn2: TBBUTTON = core::mem::zeroed();
            tb_get_button(tb, i, &mut btn2);

            if btn2.iBitmap > old_icon_index {
                let mut tbbi2: TBBUTTONINFOW = core::mem::zeroed();
                tbbi2.cbSize = size_of::<TBBUTTONINFOW>() as UINT;
                tbbi2.dwMask = TBIF_BYINDEX | TBIF_IMAGE;
                tbbi2.iImage = btn2.iBitmap - 1;
                tb_set_button_info(tb, i, &mut tbbi2);
            }
        }
    }

    bq_remove_info((*tb).m_BalloonQueue, notify_item);

    tb_delete_button(tb, index);

    heap_free(notify_item);

    TRUE as BOOL
}

unsafe fn tb_resize_imagelist(tb: *mut CNotifyToolbar) {
    let mut cx = 0i32;
    let mut cy = 0i32;

    if ImageList_GetIconSize((*tb).m_ImageList, &mut cx, &mut cy) == 0 {
        return;
    }

    if cx == GetSystemMetrics(SM_CXSMICON as i32) && cy == GetSystemMetrics(SM_CYSMICON as i32) {
        return;
    }

    let iml = ImageList_Create(
        GetSystemMetrics(SM_CXSMICON as i32),
        GetSystemMetrics(SM_CYSMICON as i32),
        ILC_COLOR32 | ILC_MASK,
        0,
        1000,
    );
    if iml.is_null() {
        return;
    }

    ImageList_Destroy((*tb).m_ImageList);
    (*tb).m_ImageList = iml;
    tb_set_image_list(tb, (*tb).m_ImageList);

    let count = tb_get_button_count(tb);
    for i in 0..count {
        let data = tb_get_item_data(tb, i);
        let has_shared_icon = (*data).dwState & NIS_SHAREDICON != 0;
        let mut i_icon = if has_shared_icon {
            tb_find_existing_shared_icon(tb, (*data).hIcon)
        } else {
            -1
        };
        if i_icon < 0 {
            i_icon = ImageList_AddIcon(iml, (*data).hIcon);
        }
        let mut tbbi: TBBUTTONINFOW = core::mem::zeroed();
        tbbi.cbSize = size_of::<TBBUTTONINFOW>() as UINT;
        tbbi.dwMask = TBIF_BYINDEX | TBIF_IMAGE;
        tbbi.iImage = i_icon;
        tb_set_button_info(tb, i, &mut tbbi);
    }

    tb_set_button_size(
        tb,
        GetSystemMetrics(SM_CXSMICON as i32),
        GetSystemMetrics(SM_CYSMICON as i32),
    );
}

unsafe fn tb_send_notify_callback(
    tb: *mut CNotifyToolbar,
    notify_item: *mut InternalIconData,
    u_msg: UINT,
) -> bool {
    if IsWindow((*notify_item).hWnd) == 0 {
        // Destroy icons with invalid handles (same as MS, only on mouse-over).
        // InternalIconData's leading layout IS NOTIFYICONDATAW (flattened base).
        tb_remove_button(tb, notify_item as *const NOTIFYICONDATAW);

        /* Ask the parent to resize */
        let h_parent = GetParent((*tb).m_hWnd);
        let mut nmh: NMHDR = core::mem::zeroed();
        nmh.hwndFrom = h_parent;
        nmh.idFrom = 0;
        nmh.code = NTNWM_REALIGN;
        SendMessageW(h_parent, WM_NOTIFY, 0, &nmh as *const NMHDR as LPARAM);

        return true;
    }

    let mut pid: DWORD = 0;
    GetWindowThreadProcessId((*notify_item).hWnd, &mut pid);

    if pid == GetCurrentProcessId() || (u_msg >= WM_MOUSEFIRST && u_msg <= WM_MOUSELAST) {
        PostMessageW(
            (*notify_item).hWnd,
            (*notify_item).uCallbackMessage,
            (*notify_item).uID as WPARAM,
            u_msg as LPARAM,
        );
    } else {
        SendMessageW(
            (*notify_item).hWnd,
            (*notify_item).uCallbackMessage,
            (*notify_item).uID as WPARAM,
            u_msg as LPARAM,
        );
    }
    false
}

unsafe fn tb_send_mouse_event(
    tb: *mut CNotifyToolbar,
    w_index: u16,
    u_msg: UINT,
    _w_param: WPARAM,
) {
    let notify_item = tb_get_item_data(tb, w_index as i32);
    tb_send_notify_callback(tb, notify_item, u_msg);
}

// ---- CNotifyToolbar message handlers ----

unsafe fn tb_on_destroy(tb: *mut CNotifyToolbar) -> LRESULT {
    if !(*tb).m_ImageList.is_null() {
        ImageList_Destroy((*tb).m_ImageList);
        (*tb).m_ImageList = null_mut();
    }
    0
}

unsafe fn tb_on_ctx_menu(
    tb: *mut CNotifyToolbar,
    _u_msg: UINT,
    _w_param: WPARAM,
    l_param: LPARAM,
    b_handled: *mut BOOL,
) -> LRESULT {
    *b_handled = FALSE as BOOL;

    let i_btn = tb_get_hot_item(tb);
    if i_btn < 0 {
        return 0;
    }

    let notify_item = tb_get_item_data(tb, i_btn);

    if IsWindow((*notify_item).hWnd) == 0 {
        return 0;
    }

    if (*notify_item).uVersionCopy >= NOTIFYICON_VERSION {
        SendNotifyMessageW(
            (*notify_item).hWnd,
            (*notify_item).uCallbackMessage,
            (*notify_item).uID as WPARAM,
            WM_CONTEXTMENU as LPARAM,
        );
    } else if l_param == -1 {
        SendNotifyMessageW(
            (*notify_item).hWnd,
            (*notify_item).uCallbackMessage,
            (*notify_item).uID as WPARAM,
            WM_RBUTTONDOWN as LPARAM,
        );
        SendNotifyMessageW(
            (*notify_item).hWnd,
            (*notify_item).uCallbackMessage,
            (*notify_item).uID as WPARAM,
            WM_RBUTTONUP as LPARAM,
        );
    }

    0
}

unsafe fn tb_on_mouse_event(
    tb: *mut CNotifyToolbar,
    u_msg: UINT,
    w_param: WPARAM,
    l_param: LPARAM,
    b_handled: *mut BOOL,
) -> LRESULT {
    let mut pt = POINT {
        x: GET_X_LPARAM(l_param),
        y: get_y_lparam(l_param),
    };
    let i_btn = tb_hit_test(tb, &mut pt);

    if i_btn >= 0 {
        tb_send_mouse_event(tb, i_btn as u16, u_msg, w_param);
    }

    *b_handled = FALSE as BOOL;
    FALSE as LRESULT
}

unsafe fn get_y_lparam(lp: LPARAM) -> i32 {
    ((lp >> 16) & 0xFFFF) as i16 as i32
}

unsafe fn tb_on_tooltip_show(
    tb: *mut CNotifyToolbar,
    hdr: *mut NMHDR,
    b_handled: *mut BOOL,
) -> LRESULT {
    let mut rc_tip: RECT = core::mem::zeroed();
    let mut rc_item: RECT = core::mem::zeroed();
    GetWindowRect((*hdr).hwndFrom, &mut rc_tip);

    let sz_tip = SIZE {
        cx: rc_tip.right - rc_tip.left,
        cy: rc_tip.bottom - rc_tip.top,
    };

    let i_btn = tb_get_hot_item(tb);

    if i_btn >= 0 {
        let mut mon_info: MONITORINFO = core::mem::zeroed();
        let h_mon = MonitorFromWindow((*tb).m_hWnd, MONITOR_DEFAULTTONEAREST);

        mon_info.cbSize = size_of::<MONITORINFO>() as DWORD;

        if !h_mon.is_null() {
            GetMonitorInfoW(h_mon, &mut mon_info);
        } else {
            GetWindowRect(GetDesktopWindow(), &mut mon_info.rcMonitor);
        }

        tb_get_item_rect(tb, i_btn, &mut rc_item);

        let mut pt_item = POINT {
            x: rc_item.left,
            y: rc_item.top,
        };
        let sz_item = SIZE {
            cx: rc_item.right - rc_item.left,
            cy: rc_item.bottom - rc_item.top,
        };
        ClientToScreen((*tb).m_hWnd, &mut pt_item);

        pt_item.x += sz_item.cx / 2;
        pt_item.y -= sz_tip.cy;

        if pt_item.x + sz_tip.cx > mon_info.rcMonitor.right {
            pt_item.x = mon_info.rcMonitor.right - sz_tip.cx;
        }
        if pt_item.y + sz_tip.cy > mon_info.rcMonitor.bottom {
            pt_item.y = mon_info.rcMonitor.bottom - sz_tip.cy;
        }
        if pt_item.x < mon_info.rcMonitor.left {
            pt_item.x = mon_info.rcMonitor.left;
        }
        if pt_item.y < mon_info.rcMonitor.top {
            pt_item.y = mon_info.rcMonitor.top;
        }

        SetWindowPos(
            (*hdr).hwndFrom,
            null_mut(),
            pt_item.x,
            pt_item.y,
            0,
            0,
            SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
        );

        return TRUE as LRESULT;
    }

    *b_handled = FALSE as BOOL;
    0
}

unsafe fn tb_refresh_toolbar_metrics(tb: *mut CNotifyToolbar, b_force_refresh: BOOL) {
    if g_TaskbarSettings.eCompactTrayIcons == TIM_Default || b_force_refresh != 0 {
        let mut tbm: TBMETRICS = core::mem::zeroed();
        tbm.cbSize = size_of::<TBMETRICS>() as UINT;
        tbm.dwMask = TBMF_BARPAD | TBMF_BUTTONSPACING | TBMF_PAD;
        tbm.cxPad = 1;
        tbm.cyPad = 1;
        if g_TaskbarSettings.UseCompactTrayIcons() == 0 {
            tbm.cxPad = GetSystemMetrics(SM_CXSMICON as i32) / 2;
            tbm.cyPad = GetSystemMetrics(SM_CYSMICON as i32) / 2;
        }
        tbm.cxBarPad = 1;
        tbm.cyBarPad = 1;
        tbm.cxButtonSpacing = 1;
        tbm.cyButtonSpacing = 1;
        tb_set_metrics(tb, &mut tbm);
    }
}

unsafe fn tb_initialize(tb: *mut CNotifyToolbar, hwnd_parent: HWND, queue: *mut CBalloonQueue) {
    (*tb).m_BalloonQueue = queue;

    let styles: DWORD = WS_CHILD
        | WS_VISIBLE
        | WS_CLIPCHILDREN
        | TBSTYLE_FLAT
        | TBSTYLE_TOOLTIPS
        | TBSTYLE_WRAPABLE
        | TBSTYLE_TRANSPARENT
        | CCS_TOP
        | CCS_NORESIZE
        | CCS_NOPARENTALIGN
        | CCS_NODIVIDER;

    // CToolbar::Create — real comctl32 toolbar; then SubclassWindow.
    // HACK & FIXME: CORE-18016 (m_hWnd = NULL between Create and Subclass).
    let hwnd = CreateWindowExW(
        WS_EX_TOOLWINDOW,
        TOOLBARCLASSNAMEW.as_ptr(),
        core::ptr::null(),
        styles,
        0,
        0,
        0,
        0,
        hwnd_parent,
        null_mut(),
        hExplorerInstance,
        null_mut(),
    );
    // CToolbar::Create: SetButtonStructSize.
    SendMessageW(
        hwnd,
        TB_BUTTONSTRUCTSIZE,
        size_of::<TBBUTTON>() as WPARAM,
        0,
    );

    (*tb).m_hWnd = null_mut();
    tb_subclass_window(tb, hwnd);

    // Force toolbar tooltips to always show even when not foreground.
    let tooltips_wnd = SendMessageW((*tb).m_hWnd, TB_GETTOOLTIPS, 0, 0) as HWND;
    if !tooltips_wnd.is_null() {
        SetWindowLongW(
            tooltips_wnd,
            GWL_STYLE,
            GetWindowLongW(tooltips_wnd, GWL_STYLE) | TTS_ALWAYSTIP as LONG,
        );
    }

    SetWindowTheme((*tb).m_hWnd, w!("TrayNotify").as_ptr(), core::ptr::null());

    (*tb).m_ImageList = ImageList_Create(
        GetSystemMetrics(SM_CXSMICON as i32),
        GetSystemMetrics(SM_CYSMICON as i32),
        ILC_COLOR32 | ILC_MASK,
        0,
        1000,
    );
    tb_set_image_list(tb, (*tb).m_ImageList);

    tb_refresh_toolbar_metrics(tb, TRUE as BOOL);

    tb_set_button_size(
        tb,
        GetSystemMetrics(SM_CXSMICON as i32),
        GetSystemMetrics(SM_CYSMICON as i32),
    );
}

// ---- CNotifyToolbar subclass machinery (CWindowImplBaseT::SubclassWindow) ----

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

unsafe fn tb_subclass_window(tb: *mut CNotifyToolbar, hwnd: HWND) {
    // `this` rides in the toolbar's GWLP_USERDATA (comctl32 toolbar leaves it
    // free — ATL uses a code thunk instead; USERDATA is the faithful-behaviour
    // stand-in here). Unhandled messages forward to the saved original proc.
    set_window_this(hwnd, tb as *mut c_void);
    let old = set_wndproc(hwnd, Some(CNotifyToolbar_WindowProc));
    (*tb).m_pfnSuperWindowProc = old;
    (*tb).m_hWnd = hwnd;
}

unsafe fn CNotifyToolbar_process(
    tb: *mut CNotifyToolbar,
    hWnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let mut b_handled: BOOL = TRUE as BOOL;
    let mut l_result: LRESULT = 0;

    if uMsg == WM_DESTROY {
        l_result = tb_on_destroy(tb);
    } else if uMsg == WM_CONTEXTMENU {
        l_result = tb_on_ctx_menu(tb, uMsg, wParam, lParam, &mut b_handled);
    } else if uMsg >= WM_MOUSEFIRST && uMsg <= WM_MOUSELAST {
        l_result = tb_on_mouse_event(tb, uMsg, wParam, lParam, &mut b_handled);
    } else if uMsg == WM_NOTIFY {
        let hdr = lParam as *mut NMHDR;
        if !hdr.is_null() && (*hdr).code == TTN_SHOW {
            l_result = tb_on_tooltip_show(tb, hdr, &mut b_handled);
        } else {
            b_handled = FALSE as BOOL;
        }
    } else {
        b_handled = FALSE as BOOL;
    }

    if b_handled != 0 {
        l_result
    } else {
        // CWindowImplBaseT::DefWindowProc == CallWindowProc(super).
        CallWindowProcW((*tb).m_pfnSuperWindowProc, hWnd, uMsg, wParam, lParam)
    }
}

// Internal (not #[no_mangle]) — only referenced via the WNDPROC fn pointer, and
// taskswnd.cpp has an unrelated C++ class also named CNotifyToolbar, so keep
// this symbol module-private to avoid any collision.
unsafe extern "system" fn CNotifyToolbar_WindowProc(
    hWnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    CNotifyToolbar_window_proc_impl(hWnd, uMsg, wParam, lParam)
}

#[inline]
unsafe fn CNotifyToolbar_window_proc_impl(
    hWnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let tb = get_window_this(hWnd) as *mut CNotifyToolbar;
    if tb.is_null() {
        return DefWindowProcW(hWnd, uMsg, wParam, lParam);
    }
    CNotifyToolbar_process(tb, hWnd, uMsg, wParam, lParam)
}

// ===========================================================================
// CSysPagerWnd message handlers
// ===========================================================================

unsafe fn sp_on_erase_background(
    this: *mut CSysPagerWnd,
    _u_msg: UINT,
    w_param: WPARAM,
    _l_param: LPARAM,
    b_handled: *mut BOOL,
) -> LRESULT {
    let hdc = w_param as HDC;

    if IsAppThemed() == 0 {
        *b_handled = FALSE as BOOL;
        return 0;
    }

    let mut rect: RECT = core::mem::zeroed();
    GetClientRect((*this).m_hWnd, &mut rect);
    DrawThemeParentBackground((*this).m_hWnd, hdc, &rect);

    TRUE as LRESULT
}

unsafe fn sp_on_create(
    this: *mut CSysPagerWnd,
    _u_msg: UINT,
    _w_param: WPARAM,
    _l_param: LPARAM,
    _b_handled: *mut BOOL,
) -> LRESULT {
    let tb = core::ptr::addr_of_mut!((*this).Toolbar);
    let bq = core::ptr::addr_of_mut!((*this).m_BalloonQueue);
    tb_initialize(tb, (*this).m_hWnd, bq);
    iw_initialize(this, (*this).m_hWnd);

    let hwnd_top = GetAncestor((*this).m_hWnd, GA_ROOT);

    // m_Balloons.Create(hWndTop, TTS_NOPREFIX | TTS_BALLOON | TTS_CLOSE)
    (*this).m_Balloons = CreateWindowExW(
        WS_EX_TOPMOST,
        TOOLTIPS_CLASSW.as_ptr(),
        w!("").as_ptr(),
        TTS_NOPREFIX | TTS_BALLOON | TTS_CLOSE,
        0,
        0,
        0,
        0,
        hwnd_top,
        null_mut(),
        hExplorerInstance,
        null_mut(),
    );

    let mut ti: TOOLINFOW = core::mem::zeroed();
    ti.cbSize = TTTOOLINFOW_V1_SIZE();
    ti.uFlags = TTF_TRACK | TTF_IDISHWND;
    ti.uId = (*this).Toolbar.m_hWnd as UINT_PTR;
    ti.hwnd = (*this).m_hWnd;
    ti.lpszText = null_mut();
    ti.lParam = 0;

    let _ret = SendMessageW(
        (*this).m_Balloons,
        TTM_ADDTOOLW,
        0,
        &ti as *const TOOLINFOW as LPARAM,
    );

    bq_init(
        bq,
        (*this).m_hWnd,
        tb,
        core::ptr::addr_of_mut!((*this).m_Balloons),
    );

    // Ask running applications to re-register their systray icons.
    SendNotifyMessageW(
        HWND_BROADCAST,
        RegisterWindowMessageW(w!("TaskbarCreated").as_ptr()),
        0,
        0,
    );

    TRUE as LRESULT
}

unsafe fn sp_on_destroy(
    this: *mut CSysPagerWnd,
    _u_msg: UINT,
    _w_param: WPARAM,
    _l_param: LPARAM,
    _b_handled: *mut BOOL,
) -> LRESULT {
    bq_deinit(&mut (*this).m_BalloonQueue);
    iw_uninitialize(this);
    TRUE as LRESULT
}

unsafe fn sp_notify_icon(
    this: *mut CSysPagerWnd,
    dw_message: DWORD,
    icon_data: *const NOTIFYICONDATAW,
) -> BOOL {
    let mut ret: BOOL = FALSE as BOOL;
    let tb = core::ptr::addr_of_mut!((*this).Toolbar);

    let visible_button_count = tb_get_visible_button_count(tb);

    match dw_message {
        NIM_ADD => {
            ret = tb_add_button_notify(tb, icon_data);
            if ret == TRUE as BOOL {
                iw_add_icon_to_watcher(this, icon_data);
            }
        }
        NIM_MODIFY => {
            ret = tb_update_button(tb, icon_data);
        }
        NIM_DELETE => {
            ret = tb_remove_button(tb, icon_data);
            if ret == TRUE as BOOL {
                iw_remove_icon_from_watcher(this, icon_data);
            }
        }
        NIM_SETFOCUS => {
            tb_set_focus(tb);
            ret = TRUE as BOOL;
        }
        NIM_SETVERSION => {
            ret = tb_switch_version(tb, icon_data);
        }
        _ => {
            return FALSE as BOOL;
        }
    }

    if visible_button_count != tb_get_visible_button_count(tb) {
        /* Ask the parent to resize */
        let h_parent = GetParent((*this).m_hWnd);
        let mut nmh: NMHDR = core::mem::zeroed();
        nmh.hwndFrom = h_parent;
        nmh.idFrom = 0;
        nmh.code = NTNWM_REALIGN;
        SendMessageW(h_parent, WM_NOTIFY, 0, &nmh as *const NMHDR as LPARAM);
    }

    ret
}

unsafe fn sp_get_size(this: *mut CSysPagerWnd, is_horizontal: BOOL, size: *mut SIZE) {
    let rows: i32;
    let columns: i32;
    let mut cy_button = GetSystemMetrics(SM_CYSMICON as i32) + 2;
    let mut cx_button = GetSystemMetrics(SM_CXSMICON as i32) + 2;
    if g_TaskbarSettings.UseCompactTrayIcons() == 0 {
        cy_button = MulDiv(GetSystemMetrics(SM_CYSMICON as i32), 3, 2);
        cx_button = MulDiv(GetSystemMetrics(SM_CXSMICON as i32), 3, 2);
    }
    let visible_button_count =
        tb_get_visible_button_count(core::ptr::addr_of_mut!((*this).Toolbar));

    if is_horizontal != 0 {
        if g_TaskbarSettings.UseCompactTrayIcons() == 0 {
            rows = ((*size).cy / MulDiv(cy_button, 3, 2)).max(1);
        } else {
            rows = ((*size).cy / cy_button).max(1);
        }
        columns = (visible_button_count + rows - 1) / rows;
    } else {
        columns = ((*size).cx / cx_button).max(1);
        rows = (visible_button_count + columns - 1) / columns;
    }
    (*size).cx = columns * cx_button;
    (*size).cy = rows * cy_button;
}

unsafe fn sp_on_get_info_tip(_this: *mut CSysPagerWnd, hdr: *mut NMHDR) -> LRESULT {
    let nmtip = hdr as *mut NMTBGETINFOTIPW;
    let notify_item = (*nmtip).lParam as *mut InternalIconData;
    if !notify_item.is_null() {
        string_cch_copy(
            (*nmtip).pszText,
            (*nmtip).cchTextMax as usize,
            (*notify_item).szTip.as_ptr(),
        );
    } else {
        string_cch_copy(
            (*nmtip).pszText,
            (*nmtip).cchTextMax as usize,
            w!("").as_ptr(),
        );
    }
    TRUE as LRESULT
}

unsafe fn sp_on_custom_draw(_this: *mut CSysPagerWnd, hdr: *mut NMHDR) -> LRESULT {
    let cdraw = hdr as *mut NMCUSTOMDRAW;
    match (*cdraw).dwDrawStage {
        x if x == CDDS_PREPAINT => CDRF_NOTIFYITEMDRAW,
        x if x == CDDS_ITEMPREPAINT => {
            TBCDRF_NOBACKGROUND
                | TBCDRF_NOEDGES
                | TBCDRF_NOOFFSET
                | TBCDRF_NOMARK
                | TBCDRF_NOETCHEDEFFECT
        }
        _ => TRUE as LRESULT,
    }
}

unsafe fn sp_on_command(
    this: *mut CSysPagerWnd,
    _u_msg: UINT,
    w_param: WPARAM,
    l_param: LPARAM,
    b_handled: *mut BOOL,
) -> LRESULT {
    *b_handled = FALSE as BOOL;

    /* BN_CLICKED from the CNotifyToolbar member */
    if hiword(w_param as usize) != BN_CLICKED as u16 {
        return 0;
    }

    // LOWORD → WORD (unsigned) assigned to INT: zero-extended, so the `< 0`
    // guard is effectively dead — preserved faithfully.
    let i_btn = loword(w_param as usize) as i32;
    if i_btn < 0 {
        return 0;
    }

    let notify_item = tb_get_item_data(core::ptr::addr_of_mut!((*this).Toolbar), i_btn);

    if IsWindow((*notify_item).hWnd) == 0 {
        return 0;
    }

    if (*notify_item).uVersionCopy >= NOTIFYICON_VERSION {
        SendNotifyMessageW(
            (*notify_item).hWnd,
            (*notify_item).uCallbackMessage,
            (*notify_item).uID as WPARAM,
            NIN_SELECT as LPARAM,
        );
    } else if l_param == -1 {
        SendNotifyMessageW(
            (*notify_item).hWnd,
            (*notify_item).uCallbackMessage,
            (*notify_item).uID as WPARAM,
            WM_LBUTTONDOWN as LPARAM,
        );
        SendNotifyMessageW(
            (*notify_item).hWnd,
            (*notify_item).uCallbackMessage,
            (*notify_item).uID as WPARAM,
            WM_LBUTTONUP as LPARAM,
        );
    }

    0
}

unsafe fn sp_on_size(
    this: *mut CSysPagerWnd,
    u_msg: UINT,
    w_param: WPARAM,
    l_param: LPARAM,
    _b_handled: *mut BOOL,
) -> LRESULT {
    let sz_client = SIZE {
        cx: loword(l_param as usize) as i32,
        cy: hiword(l_param as usize) as i32,
    };

    let ret = DefWindowProcW((*this).m_hWnd, u_msg, w_param, l_param);

    let tb = core::ptr::addr_of_mut!((*this).Toolbar);
    if !(*tb).m_hWnd.is_null() {
        SetWindowPos(
            (*tb).m_hWnd,
            null_mut(),
            0,
            0,
            sz_client.cx,
            sz_client.cy,
            SWP_NOZORDER,
        );
        tb_auto_size(tb);

        let mut rc: RECT = core::mem::zeroed();
        GetClientRect((*tb).m_hWnd, &mut rc);

        let sz_bar = SIZE {
            cx: rc.right - rc.left,
            cy: rc.bottom - rc.top,
        };

        let x_off = (sz_client.cx - sz_bar.cx) / 2;
        let y_off = (sz_client.cy - sz_bar.cy) / 2;

        SetWindowPos(
            (*tb).m_hWnd,
            null_mut(),
            x_off,
            y_off,
            sz_bar.cx,
            sz_bar.cy,
            SWP_NOZORDER,
        );
        tb_refresh_toolbar_metrics(tb, FALSE as BOOL);
    }
    ret
}

unsafe fn sp_on_ctx_menu(
    _this: *mut CSysPagerWnd,
    _u_msg: UINT,
    _w_param: WPARAM,
    _l_param: LPARAM,
    b_handled: *mut BOOL,
) -> LRESULT {
    *b_handled = TRUE as BOOL;
    0
}

unsafe fn sp_on_balloon_pop(this: *mut CSysPagerWnd, b_handled: *mut BOOL) -> LRESULT {
    bq_close_current(&mut (*this).m_BalloonQueue);
    *b_handled = TRUE as BOOL;
    0
}

unsafe fn sp_on_timer(
    this: *mut CSysPagerWnd,
    _u_msg: UINT,
    w_param: WPARAM,
    _l_param: LPARAM,
    b_handled: *mut BOOL,
) -> LRESULT {
    if bq_on_timer(&mut (*this).m_BalloonQueue, w_param as i32) {
        *b_handled = TRUE as BOOL;
    }
    0
}

unsafe fn sp_on_copy_data(
    this: *mut CSysPagerWnd,
    _u_msg: UINT,
    _w_param: WPARAM,
    l_param: LPARAM,
    _b_handled: *mut BOOL,
) -> LRESULT {
    let cp_data = l_param as PCOPYDATASTRUCT;
    if (*cp_data).dwData as usize == TABDMC_NOTIFY {
        /* A taskbar NotifyIcon notification */
        let p_data = (*cp_data).lpData as *mut TRAYNOTIFYDATAW;
        if (*p_data).dwSignature == NI_NOTIFY_SIG {
            return sp_notify_icon(this, (*p_data).dwMessage, &(*p_data).nid) as LRESULT;
        }
    } else if (*cp_data).dwData as usize == TABDMC_LOADINPROC {
        // FIXME: Taskbar Load In Proc
    }

    FALSE as LRESULT
}

unsafe fn sp_on_setting_changed(
    this: *mut CSysPagerWnd,
    _u_msg: UINT,
    w_param: WPARAM,
    _l_param: LPARAM,
    _b_handled: *mut BOOL,
) -> LRESULT {
    if w_param == SPI_SETNONCLIENTMETRICS as WPARAM {
        tb_resize_imagelist(core::ptr::addr_of_mut!((*this).Toolbar));
    }
    0
}

unsafe fn sp_on_get_minimum_size(
    this: *mut CSysPagerWnd,
    _u_msg: UINT,
    w_param: WPARAM,
    l_param: LPARAM,
    _b_handled: *mut BOOL,
) -> LRESULT {
    sp_get_size(this, w_param as BOOL, l_param as *mut SIZE);
    0
}

// ===========================================================================
// CSysPagerWnd WndProc (BEGIN_MSG_MAP) — arch-split callback ABI.
// ===========================================================================

unsafe fn CSysPagerWnd_process(
    this: *mut CSysPagerWnd,
    hWnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let mut b_handled: BOOL = TRUE as BOOL;
    let mut l_result: LRESULT = 0;

    match uMsg {
        WM_CREATE => l_result = sp_on_create(this, uMsg, wParam, lParam, &mut b_handled),
        WM_DESTROY => l_result = sp_on_destroy(this, uMsg, wParam, lParam, &mut b_handled),
        WM_ERASEBKGND => {
            l_result = sp_on_erase_background(this, uMsg, wParam, lParam, &mut b_handled)
        }
        WM_COMMAND => l_result = sp_on_command(this, uMsg, wParam, lParam, &mut b_handled),
        WM_SIZE => l_result = sp_on_size(this, uMsg, wParam, lParam, &mut b_handled),
        WM_CONTEXTMENU => l_result = sp_on_ctx_menu(this, uMsg, wParam, lParam, &mut b_handled),
        WM_TIMER => l_result = sp_on_timer(this, uMsg, wParam, lParam, &mut b_handled),
        WM_COPYDATA => l_result = sp_on_copy_data(this, uMsg, wParam, lParam, &mut b_handled),
        WM_SETTINGCHANGE => {
            l_result = sp_on_setting_changed(this, uMsg, wParam, lParam, &mut b_handled)
        }
        x if x == TNWM_GETMINIMUMSIZE => {
            l_result = sp_on_get_minimum_size(this, uMsg, wParam, lParam, &mut b_handled)
        }
        WM_NOTIFY => {
            let hdr = lParam as *mut NMHDR;
            if !hdr.is_null() {
                match (*hdr).code {
                    x if x == TTN_POP => l_result = sp_on_balloon_pop(this, &mut b_handled),
                    x if x == TBN_GETINFOTIPW => l_result = sp_on_get_info_tip(this, hdr),
                    x if x == NM_CUSTOMDRAW => l_result = sp_on_custom_draw(this, hdr),
                    _ => b_handled = FALSE as BOOL,
                }
            } else {
                b_handled = FALSE as BOOL;
            }
        }
        _ => b_handled = FALSE as BOOL,
    }

    if b_handled != 0 {
        l_result
    } else {
        DefWindowProcW(hWnd, uMsg, wParam, lParam)
    }
}

#[no_mangle]
pub unsafe extern "system" fn CSysPagerWnd_WindowProc(
    hWnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    CSysPagerWnd_window_proc_impl(hWnd, uMsg, wParam, lParam)
}

#[inline]
unsafe fn CSysPagerWnd_window_proc_impl(
    hWnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let this = if uMsg == WM_NCCREATE {
        let p = wndproc_nccreate_this(hWnd, lParam) as *mut CSysPagerWnd;
        if !p.is_null() {
            (*p).m_hWnd = hWnd;
        }
        p
    } else {
        get_window_this(hWnd) as *mut CSysPagerWnd
    };

    if this.is_null() {
        return DefWindowProcW(hWnd, uMsg, wParam, lParam);
    }

    let r = CSysPagerWnd_process(this, hWnd, uMsg, wParam, lParam);

    if uMsg == WM_NCDESTROY {
        (*this).m_hWnd = null_mut();
    }
    r
}

// ===========================================================================
// CWindowImpl::Create + Initialize + factory.
// ===========================================================================

static szSysPagerWndClass: [u16; 9] = [
    b'S' as u16,
    b'y' as u16,
    b's' as u16,
    b'P' as u16,
    b'a' as u16,
    b'g' as u16,
    b'e' as u16,
    b'r' as u16,
    0,
];

unsafe fn CSysPagerWnd_Create(this: *mut CSysPagerWnd, hWndParent: HWND, dwStyle: DWORD) -> HWND {
    // DECLARE_WND_CLASS_EX(szSysPagerWndClass, CS_DBLCLKS, COLOR_3DFACE)
    register_class_once(
        hExplorerInstance,
        szSysPagerWndClass.as_ptr(),
        Some(CSysPagerWnd_WindowProc),
        CS_DBLCLKS,
        COLOR_3DFACE as i32,
    );

    (*this).m_hWnd = CreateWindowExW(
        0,
        szSysPagerWndClass.as_ptr(),
        core::ptr::null(),
        dwStyle,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        hWndParent,
        null_mut(),
        hExplorerInstance,
        this as LPVOID,
    );
    (*this).m_hWnd
}

unsafe fn CSysPagerWnd_Initialize(this: *mut CSysPagerWnd, hWndParent: HWND) -> HRESULT {
    let dw_style: DWORD = WS_CHILD | WS_CLIPSIBLINGS | WS_VISIBLE;
    CSysPagerWnd_Create(this, hWndParent, dw_style);
    if (*this).m_hWnd.is_null() {
        return E_FAIL;
    }

    SetWindowTheme((*this).m_hWnd, w!("TrayNotify").as_ptr(), core::ptr::null());

    S_OK
}

#[no_mangle]
pub unsafe extern "C" fn CSysPagerWnd_CreateInstance(
    hwndParent: HWND,
    riid: REFIID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    // ShellObjectCreatorInit<CSysPagerWnd>(hwndParent, riid, ppv).
    let pobj = CSysPagerWnd_new();
    if pobj.is_null() {
        return E_OUTOFMEMORY;
    }
    let iface = pobj as *mut IOleWindow;

    CSysPagerWnd_AddRef(iface);

    let mut hr = CSysPagerWnd_Initialize(pobj, hwndParent);
    if hr >= 0 {
        hr = CSysPagerWnd_QueryInterface(iface, riid, ppv);
    }

    CSysPagerWnd_Release(iface);
    hr
}
