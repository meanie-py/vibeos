use crate::prelude::*;
use crate::menu_band::CMenuBand;
use crate::menu_band::CMenuToolbarBase;
use crate::menu_focus_manager_support::s_GetMsgHook;
use crate::menu_focus_manager_support::s_MsgFilterHook;
pub type StackEntryType = INT;
pub const NoEntry: StackEntryType = 0;
pub const MenuBarEntry: StackEntryType = 1;
pub const MenuPopupEntry: StackEntryType = 2;
pub const TrackedMenuEntry: StackEntryType = 3;

#[repr(C)]
pub struct StackEntry {
    pub type_: StackEntryType,
    pub mb: *mut CMenuBand,
    pub hmenu: HMENU,
    pub hwnd: HWND,
}

// Deterministic COM scaffold for CMenuFocusManager; method bodies are generated separately.
extern "system" {
    fn GetProcessHeap() -> HANDLE;
    #[link_name = "HeapAlloc"] fn cpp_com_HeapAlloc(heap: HANDLE, flags: DWORD, bytes: usize) -> *mut core::ffi::c_void;
    #[link_name = "HeapFree"] fn cpp_com_HeapFree(heap: HANDLE, flags: DWORD, memory: *mut core::ffi::c_void) -> BOOL;
    fn MenuBandRustModuleLock();
    fn MenuBandRustModuleUnlock();
}
trait CppComSize { fn cpp_com_size(self) -> usize; }
impl CppComSize for usize { fn cpp_com_size(self) -> usize { self } }
impl CppComSize for u32 { fn cpp_com_size(self) -> usize { self as usize } }
#[inline]
unsafe fn HeapAlloc<N: CppComSize>(heap: HANDLE, flags: DWORD, bytes: N) -> *mut core::ffi::c_void {
    cpp_com_HeapAlloc(heap, flags, bytes.cpp_com_size())
}
#[inline]
unsafe fn HeapFree<T>(heap: HANDLE, flags: DWORD, memory: *mut T) -> BOOL {
    cpp_com_HeapFree(heap, flags, memory.cast())
}
extern "C" {
    pub static IID_IUnknown: IID;
}

#[repr(C)]
pub struct CMenuFocusManager {
    pub ref_count: core::sync::atomic::AtomicI32,
    pub m_current: *mut StackEntry,
    pub m_parent: *mut StackEntry,
    pub m_menuBar: *mut StackEntry,
    pub m_hMsgFilterHook: HHOOK,
    pub m_hGetMsgHook: HHOOK,
    pub m_threadId: DWORD,
    pub m_mouseTrackDisabled: BOOL,
    pub m_ptPrev: POINT,
    pub m_captureHwnd: HWND,
    pub m_hwndUnderMouse: HWND,
    pub m_entryUnderMouse: *mut StackEntry,
    pub m_selectedMenu: HMENU,
    pub m_selectedItem: INT,
    pub m_selectedItemFlags: DWORD,
    pub m_movedSinceDown: BOOL,
    pub m_windowAtDown: HWND,
    pub m_PreviousForeground: HWND,
    pub m_bandStack: [StackEntry; 20],
    pub m_bandCount: i32,
    pub m_menuDepth: i32,
}

/// `new CMenuFocusManager` — operator new, the vptr store, then the constructor body.
pub unsafe fn CMenuFocusManager_new() -> *mut CMenuFocusManager {
    let this = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        core::mem::size_of::<CMenuFocusManager>(),
    ) as *mut CMenuFocusManager;
    if this.is_null() { return core::ptr::null_mut(); }
    CMenuFocusManager_ctor_body(this);
    this
}

/// `delete` — the destructor body, then the matching free. Pairs with
/// `CMenuFocusManager_new`; freeing through any other allocator is a heap bug.
pub unsafe fn CMenuFocusManager_delete(this: *mut CMenuFocusManager) {
    if this.is_null() { return; }
    CMenuFocusManager_dtor_body(this);
    HeapFree(GetProcessHeap(), 0, this);
}

// Deterministic C++ support closure selected from source identifiers.

pub const GA_ROOT: DWORD = 2;

pub const HC_ACTION: DWORD = 0;

pub const HICF_MOUSE: DWORD = 0x1;

pub const MF_POPUP: DWORD = 16;

pub const MPOS_EXECUTE: DWORD = 0;

pub const MPOS_FULLCANCEL: DWORD = 1;

pub const MSGF_MENU: DWORD = 2;

pub const VK_DOWN: DWORD = 40;

pub const VK_ESCAPE: DWORD = 0x1B;

pub const VK_LEFT: DWORD = 37;

pub const VK_LMENU: DWORD = 0xA4;

pub const VK_MENU: DWORD = 18;

pub const VK_RETURN: DWORD = 13;

pub const VK_RIGHT: DWORD = 39;

pub const VK_RMENU: DWORD = 0xA5;

pub const VK_UP: DWORD = 38;

pub const WH_GETMESSAGE: DWORD = 3;

pub const WM_CANCELMODE: DWORD = 31;

pub const WM_CAPTURECHANGED: DWORD = 533;

pub const WM_INITMENUPOPUP: DWORD = 279;

pub const WM_KEYDOWN: DWORD = 256;

pub const WM_LBUTTONDBLCLK: DWORD = 515;

pub const WM_LBUTTONDOWN: DWORD = 513;

pub const WM_LBUTTONUP: DWORD = 514;

pub const WM_MENUSELECT: DWORD = 287;

pub const WM_MOUSEMOVE: DWORD = 512;

pub const WM_NCLBUTTONDOWN: DWORD = 161;

pub const WM_NCLBUTTONUP: DWORD = 162;

pub const WM_NCRBUTTONUP: DWORD = 165;

pub const WM_NULL: DWORD = 0;

pub const WM_RBUTTONDOWN: DWORD = 516;

pub const WM_RBUTTONUP: DWORD = 517;

pub const WM_SYSKEYDOWN: DWORD = 260;

// Deterministic from oleidl.h
#[repr(C)]
pub struct IOleWindow {
    pub lpVtbl: *const IOleWindowVtbl,
}
#[repr(C)]
pub struct IOleWindowVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IOleWindow, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IOleWindow) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IOleWindow) -> ULONG,
    pub GetWindow: unsafe extern "system" fn(This: *mut IOleWindow, phwnd: *mut HWND) -> HRESULT,
    pub ContextSensitiveHelp: unsafe extern "system" fn(This: *mut IOleWindow, fEnterMode: BOOL) -> HRESULT,
}
extern "C" {
    pub static IID_IOleWindow: IID;
}

// Deterministic from servprov.h
#[repr(C)]
pub struct IServiceProvider {
    pub lpVtbl: *const IServiceProviderVtbl,
}
#[repr(C)]
pub struct IServiceProviderVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IServiceProvider, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IServiceProvider) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IServiceProvider) -> ULONG,
    pub QueryService: unsafe extern "system" fn(This: *mut IServiceProvider, guidService: REFGUID, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
}
extern "C" {
    pub static IID_IServiceProvider: IID;
}

extern "system" {
    fn IUnknown_QueryService(object: *mut IUnknown, service: REFGUID, riid: REFIID, out: *mut *mut core::ffi::c_void) -> HRESULT;
}

extern "C" {
    static SID_SMenuBandParent: GUID;
    static SID_SMenuPopup: GUID;
}

pub struct CppComPtrGuard<T> { slot: *mut *mut T }
impl<T> CppComPtrGuard<T> {
    pub const fn new(slot: *mut *mut T) -> Self { Self { slot } }
}
impl<T> Drop for CppComPtrGuard<T> {
    fn drop(&mut self) {
        unsafe {
            let value = *self.slot;
            if !value.is_null() {
                let unknown = value as *mut IUnknown;
                ((*(*unknown).lpVtbl).Release)(unknown);
                *self.slot = core::ptr::null_mut();
            }
        }
    }
}

use core::ffi::c_void;
use crate::menu_band::CMenuBand_AddRef_RustExport;
use crate::menu_band::CMenuBand_IsWindowOwner_RustExport;
use crate::menu_band::CMenuBand_MenuBarMouseDown_RustExport;
use crate::menu_band::CMenuBand_MenuBarMouseUp_RustExport;
use crate::menu_band::{CMenuBand_SetChildBand_RustExport, CMenuBand_SetParentBand_RustExport};
use crate::menu_band::{CMenuBand_BeforeCancelPopup_RustExport, CMenuBand_MenuItemSelect_RustExport};
use crate::menu_band::{CMenuBand_ChangeHotItem_RustExport, CMenuBand_IsTracking_RustExport};
use crate::menu_band::{CMenuBand_DisableMouseTrack_RustExport, CMenuBand_GetTopLevelWindow_RustExport};
use crate::menu_band::CMenuBand_GetSite_RustExport;
use crate::menu_band::CMenuBand_IsPopup_RustExport;
use crate::menu_band::CMenuBand_Release_RustExport;

pub unsafe fn CMenuFocusManager_GetManager_body(this: *mut CMenuFocusManager) -> *mut CMenuFocusManager {
    TlsGetValue(TlsIndex) as *mut CMenuFocusManager
}

pub unsafe fn CMenuFocusManager_AcquireManager_body(this: *mut CMenuFocusManager) -> *mut CMenuFocusManager {
    unsafe {
        // winbase.h `#define TLS_OUT_OF_INDEXES ((DWORD)0xFFFFFFFF)` — only used in this
        // fn, so safe as a local const (unlike TlsIndex, above).

        let mut obj: *mut CMenuFocusManager = core::ptr::null_mut();

        if TlsIndex == 0 {
            TlsIndex = TlsAlloc();
            if TlsIndex == TLS_OUT_OF_INDEXES {
                return core::ptr::null_mut();
            }
        }

        obj = CMenuFocusManager_GetManager_body(this);

        if obj.is_null() {
            // `CMenuFocusManager::new()` doesn't exist — this is a plain struct with no
            // COM map here; the scaffold's actual ctor entry point is the free function
            // `CMenuFocusManager_new()` (menu_focus_manager.rs:65), which allocates +
            // runs CMenuFocusManager_ctor_body, matching C++'s `new CComObject<...>()`.
            obj = CMenuFocusManager_new();
            // TlsSetValue's `PVOID` param needs the raw pointer cast — Rust won't
            // implicitly coerce `*mut CMenuFocusManager` to `*mut c_void`.
            TlsSetValue(TlsIndex, obj.cast());
        }

        // C++ `obj->AddRef()`. CMenuFocusManager exposes NO COM interface — it derives
        // CComObjectRootEx only — so this is ATL's `InternalAddRef`, a plain increment of
        // the inherited `m_dwRef`, NOT a vtable dispatch. The scaffold emits that counter as
        // `ref_count` for an ATL-refcounted class.
        (*obj).ref_count.fetch_add(1, core::sync::atomic::Ordering::SeqCst);

        obj
    }
}

pub unsafe fn CMenuFocusManager_ReleaseManager_body(this: *mut CMenuFocusManager, obj: *mut CMenuFocusManager) -> () {
    // C++ `if (!obj->Release())`. ATL's `InternalRelease` returns the DECREMENTED count,
    // so the C's "falsy" test is "reached zero". `fetch_sub` returns the value BEFORE
    // the subtraction, hence the `- 1`.
    if (*obj).ref_count.fetch_sub(1, core::sync::atomic::Ordering::SeqCst) - 1 == 0 {
        TlsSetValue(TlsIndex, core::ptr::null_mut());
    }
}

pub unsafe fn CMenuFocusManager_s_MsgFilterHook_body(this: *mut CMenuFocusManager, nCode: INT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    return CMenuFocusManager_MsgFilterHook_body(this, nCode, wParam, lParam);
}

pub unsafe fn CMenuFocusManager_s_GetMsgHook_body(this: *mut CMenuFocusManager, nCode: INT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    unsafe {
        CMenuFocusManager_GetMsgHook_body(this, nCode, wParam, lParam)
    }
}

pub unsafe fn CMenuFocusManager_PushToArray_body(this: *mut CMenuFocusManager, r#type: StackEntryType, mb: *mut CMenuBand, hmenu: HMENU) -> HRESULT {
    // CMenuFocusManager.h `#define MAX_RECURSE 20` — sizes m_bandStack and bounds
    // m_bandCount. Only used in this fn, so a local const (module-scope items declared
    // before this fn's signature don't survive materialization — see GetManager.rs).

    if (*this).m_bandCount >= MAX_RECURSE {
        return E_OUTOFMEMORY;
    }

    // FLAG: `m_bandStack` is not a field on the `CMenuFocusManager` struct in the current
    // assembled scaffold (it declares m_current/m_parent/m_menuBar as single StackEntry
    // pointers, no array) — this needs `pub m_bandStack: [StackEntry; MAX_RECURSE]` (or
    // similar) added there; not something this bank file can add. Left as written rather
    // than invented. Also note: whatever the materializer does with `.r#type` here gets
    // normalized to `.type_` in the assembled output (confirmed by reading it back), so
    // the struct's `StackEntry::r#type` field will need the same rename to agree —
    // flagging rather than guessing which name is the intended final one.
    (*this).m_bandStack[(*this).m_bandCount as usize].type_ = r#type;
    (*this).m_bandStack[(*this).m_bandCount as usize].mb = mb;
    (*this).m_bandStack[(*this).m_bandCount as usize].hmenu = hmenu;
    (*this).m_bandCount += 1;

    return S_OK;
}

pub unsafe fn CMenuFocusManager_PopFromArray_body(this: *mut CMenuFocusManager, pType: *mut StackEntryType, pMb: *mut *mut CMenuBand, pHmenu: *mut HMENU) -> HRESULT {
    if !pType.is_null() {
        *pType = NoEntry;
    }
    if !pMb.is_null() {
        *pMb = core::ptr::null_mut();
    }
    if !pHmenu.is_null() {
        *pHmenu = core::ptr::null_mut();
    }

    if (*this).m_bandCount <= 0 {
        return S_FALSE;
    }

    (*this).m_bandCount -= 1;
    let index = (*this).m_bandCount as usize;

    if !pType.is_null() {
        *pType = (*this).m_bandStack[index].type_;
    }
    if *pType == TrackedMenuEntry {
        if !pHmenu.is_null() {
            *pHmenu = (*this).m_bandStack[index].hmenu;
        }
    } else {
        if !pMb.is_null() {
            *pMb = (*this).m_bandStack[index].mb;
        }
    }

    return S_OK;
}

pub unsafe fn CMenuFocusManager_ctor_body(this: *mut CMenuFocusManager) -> () {
    (*this).m_current = 0 as _;
    (*this).m_parent = 0 as _;
    (*this).m_hMsgFilterHook = 0 as _;
    (*this).m_hGetMsgHook = 0 as _;
    (*this).m_mouseTrackDisabled = FALSE as BOOL;
    (*this).m_captureHwnd = core::ptr::null_mut();
    (*this).m_hwndUnderMouse = 0 as _;
    (*this).m_entryUnderMouse = 0 as _;
    (*this).m_selectedMenu = 0 as _;
    (*this).m_selectedItem = 0;
    (*this).m_selectedItemFlags = 0;
    (*this).m_movedSinceDown = FALSE as BOOL;
    (*this).m_windowAtDown = 0 as _;
    (*this).m_PreviousForeground = 0 as _;
    (*this).m_bandCount = 0;
    (*this).m_menuDepth = 0;
    (*this).m_ptPrev.x = 0;
    (*this).m_ptPrev.y = 0;
    (*this).m_threadId = GetCurrentThreadId();
}

pub unsafe fn CMenuFocusManager_dtor_body(this: *mut CMenuFocusManager) -> () {}

pub unsafe fn CMenuFocusManager_DisableMouseTrack_body(this: *mut CMenuFocusManager, parent: HWND, disableThis: BOOL) -> () {
    // A `use` before this fn's signature is dropped by materialization (only the text
    // inside the braces survives) — function-scoped instead.

    let mut bDisable: BOOL = FALSE as BOOL;
    let mut lastDisable: BOOL = FALSE as BOOL;

    let mut i: i32 = (*this).m_bandCount;
    while {
        i -= 1;
        i >= 0
    } {
        let entry_ptr: *mut StackEntry = (*this).m_bandStack.as_mut_ptr().add(i as usize);
        let entry_mb = (*entry_ptr).mb;
        let entry_type = (*entry_ptr).type_;

        if entry_type != TrackedMenuEntry {
            let mut hwnd: HWND = core::mem::zeroed();
            let hr: HRESULT = CMenuBand_GetTopLevelWindow_RustExport(entry_mb, core::ptr::addr_of_mut!(hwnd).cast());
            if ((hr) < 0) {
                break;
            }

            if hwnd == parent {
                lastDisable = disableThis;
                CMenuBand_DisableMouseTrack_RustExport(entry_mb, disableThis);
                bDisable = TRUE as BOOL;
            } else {
                lastDisable = bDisable;
                CMenuBand_DisableMouseTrack_RustExport(entry_mb, bDisable);
            }
        }
    }
    (*this).m_mouseTrackDisabled = lastDisable;
}

pub unsafe fn CMenuFocusManager_SetMenuCapture_body(this: *mut CMenuFocusManager, child: HWND) -> () {
    if (*this).m_captureHwnd != child {
        if child != core::ptr::null_mut() {
            SetCapture(child);
            (*this).m_captureHwnd = child;
        } else {
            ReleaseCapture();
            (*this).m_captureHwnd = core::ptr::null_mut();
        }
    }
}

pub unsafe fn CMenuFocusManager_IsTrackedWindow_body(this: *mut CMenuFocusManager, hWnd: HWND, pentry: *mut *mut StackEntry) -> HRESULT {

    if !pentry.is_null() {
        *pentry = core::ptr::null_mut();
    }

    let mut i: i32 = (*this).m_bandCount;
    loop {
        i -= 1;
        if !(i >= 0) {
            break;
        }

        let entry: *mut StackEntry = &mut (*this).m_bandStack[i as usize];

        if (*entry).type_ != TrackedMenuEntry {
            let hr: HRESULT = CMenuBand_IsWindowOwner_RustExport((*entry).mb, hWnd.cast());
            if ((hr) < 0) {
                return hr;
            }
            if hr == S_OK {
                if !pentry.is_null() {
                    *pentry = entry;
                }
                return S_OK;
            }
        }
    }

    S_FALSE
}

pub unsafe fn CMenuFocusManager_IsTrackedWindowOrParent_body(this: *mut CMenuFocusManager, hWnd: HWND) -> HRESULT {

    let mut i: i32 = (*this).m_bandCount;
    while {
        i -= 1;
        i >= 0
    } {
        let entry = &(*this).m_bandStack[i as usize];
        if (*entry).type_ != TrackedMenuEntry {
            let mut hr: HRESULT = CMenuBand_IsWindowOwner_RustExport(entry.mb, hWnd.cast());
            if (((hr) < 0)) {
                return hr;
            }
            if (hr == S_OK) {
                return S_OK;
            }
            if (CMenuBand_IsPopup_RustExport(entry.mb) == S_OK) {
                let mut site: *mut IUnknown = core::ptr::null_mut();
                let _site_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(site));
                let mut pw: *mut IOleWindow = core::ptr::null_mut();
                let _pw_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pw));
                hr = CMenuBand_GetSite_RustExport(entry.mb, core::ptr::addr_of!(IID_IUnknown), core::ptr::addr_of_mut!(site).cast());
                if (((hr) < 0)) {
                    continue;
                }
                hr = IUnknown_QueryService(site, core::ptr::addr_of!(SID_SMenuBandParent), core::ptr::addr_of!(IID_IOleWindow), core::ptr::addr_of_mut!(pw).cast());
                if (((hr) < 0)) {
                    continue;
                }

                let mut hParent: HWND = core::ptr::null_mut();
                if ((*(*pw).lpVtbl).GetWindow)(pw, &mut hParent as *mut HWND) == S_OK && hParent == hWnd {
                    return S_OK;
                }
            }
        }
    }
    return S_FALSE;
}

pub unsafe fn CMenuFocusManager_ProcessMouseMove_body(this: *mut CMenuFocusManager, msg: *mut MSG) -> LRESULT {

    // windowsx.h `GET_X_LPARAM`/`GET_Y_LPARAM` and minwindef.h `HIWORD` — bindgen doesn't
    // emit macros, so these are hand-declared, nested in this fn's body (an item declared
    // before this fn's signature is dropped by materialization). Made generic because this
    // helper is ALSO needed, and duplicated, inside MsgFilterHook.rs — applied there to
    // both an LPARAM (mouse position) and a WPARAM (WM_MENUSELECT payload); same
    // low/high-word extraction either way, only the source integer's width/signedness
    // differs.
    trait CppComWordSource { fn cpp_com_word_source(self) -> u32; }
    impl CppComWordSource for LPARAM { fn cpp_com_word_source(self) -> u32 { self as u32 } }
    impl CppComWordSource for WPARAM { fn cpp_com_word_source(self) -> u32 { self as u32 } }

    #[inline]
    unsafe fn GET_X_LPARAM<T: CppComWordSource>(lp: T) -> INT {
        (lp.cpp_com_word_source() as u16) as i16 as INT
    }
    #[inline]
    unsafe fn GET_Y_LPARAM<T: CppComWordSource>(lp: T) -> INT {
        ((lp.cpp_com_word_source() >> 16) as u16) as i16 as INT
    }

    let child: HWND;
    let mut iHitTestResult: i32 = -1;

    let mut pt2 = POINT { x: GET_X_LPARAM((*msg).lParam), y: GET_Y_LPARAM((*msg).lParam) };
    ClientToScreen((*msg).hwnd, &mut pt2);

    let mut pt = (*msg).pt;

    let cCapture: HWND = GetCapture();
    if (cCapture as usize) != 0 && cCapture != (*this).m_captureHwnd && (*(*this).m_current).type_ != TrackedMenuEntry {
        return TRUE as LRESULT;
    }

    (*this).m_movedSinceDown = TRUE as BOOL;

    (*this).m_ptPrev = pt;

    child = WindowFromPoint(pt);

    let mut entry: *mut StackEntry = core::ptr::null_mut();
    if CMenuFocusManager_IsTrackedWindow_body(this, child, &mut entry) == S_OK {
    }

    let mut isTracking: BOOL = FALSE as BOOL;
    if (entry as usize) != 0 && ((*entry).type_ == MenuBarEntry || (*(*this).m_current).type_ != TrackedMenuEntry) {
        ScreenToClient(child, &mut pt);
        iHitTestResult = SendMessageW(child, TB_HITTEST, 0, &mut pt as *mut POINT as LPARAM) as i32;
        isTracking = CMenuBand_IsTracking_RustExport((*entry).mb);

        if iHitTestResult < -1 {
            iHitTestResult = -iHitTestResult;
        } else if iHitTestResult == -1 {
            let mut rc = RECT { left: 0, top: 0, right: 0, bottom: 0 };
            SendMessageW(child, TB_GETITEMRECT, 1, &mut rc as *mut RECT as LPARAM);
            if PtInRect(&mut rc, pt) != 0 {
                iHitTestResult = 1;
            }
        }

        if SendMessageW(child, WM_USER_ISTRACKEDITEM, iHitTestResult as WPARAM, 0) == (S_FALSE as LRESULT) {
            CMenuFocusManager_DisableMouseTrack_body(this, 0 as HWND, FALSE as BOOL);
            if isTracking != 0 && iHitTestResult >= 0 && (*(*this).m_current).type_ == TrackedMenuEntry {
                SendMessageW((*entry).hwnd, WM_CANCELMODE, 0, 0);
            }
            PostMessageW(child, WM_USER_CHANGETRACKEDITEM, iHitTestResult as WPARAM, MAKELPARAM!(isTracking, TRUE));
            if (*(*this).m_current).type_ == TrackedMenuEntry {
                return FALSE as LRESULT;
            }
        }
    }

    if (*this).m_entryUnderMouse != entry {
        if ((*this).m_entryUnderMouse as usize) != 0 {
            CMenuBand_ChangeHotItem_RustExport((*(*this).m_entryUnderMouse).mb, 0 as *mut crate::menu_band::CMenuToolbarBase, -1, HICF_MOUSE);
        }
    }

    if (*this).m_hwndUnderMouse != child {
        if (entry as usize) != 0 {
            if (*(*this).m_current).type_ == MenuPopupEntry {
                ScreenToClient(child, &mut pt2);
                SendMessageW(child, WM_MOUSEMOVE, (*msg).wParam, MAKELPARAM!(pt2.x, pt2.y));
            }
        }

        (*this).m_hwndUnderMouse = child;
        (*this).m_entryUnderMouse = entry;
    }

    if (*(*this).m_current).type_ == MenuPopupEntry {
        let parent: HWND = GetAncestor(child, GA_ROOT);
        CMenuFocusManager_DisableMouseTrack_body(this, parent, FALSE as BOOL);
    }

    return TRUE as LRESULT;
}

pub unsafe fn CMenuFocusManager_ProcessMouseDown_body(this: *mut CMenuFocusManager, msg: *mut MSG, isLButton: BOOL) -> LRESULT {

    let mut iHitTestResult: i32 = -1;

    let cCapture = GetCapture();
    if !cCapture.is_null() && cCapture != (*this).m_captureHwnd && (*(*this).m_current).type_ != TrackedMenuEntry {
        return TRUE as LRESULT;
    }

    let mut pt = (*msg).pt;

    let child = WindowFromPoint(pt);

    let mut entry: *mut StackEntry = core::ptr::null_mut();
    if CMenuFocusManager_IsTrackedWindow_body(this, child, &mut entry) != S_OK {
        return TRUE as LRESULT;
    }

    if (*entry).type_ == MenuBarEntry {
        if entry != (*this).m_current {
            return TRUE as LRESULT;
        }
    }

    if !entry.is_null() {
        ScreenToClient(child, &mut pt);
        iHitTestResult = SendMessageW(
            child,
            TB_HITTEST,
            0,
            core::ptr::addr_of_mut!(pt) as LPARAM,
        ) as i32;

        if iHitTestResult >= 0 {
            CMenuBand_MenuBarMouseDown_RustExport((*entry).mb, child.cast(), iHitTestResult, isLButton);
        }
    }

    (*msg).message = WM_NULL;

    (*this).m_movedSinceDown = FALSE as BOOL;
    (*this).m_windowAtDown = child;

    return TRUE as LRESULT;
}

pub unsafe fn CMenuFocusManager_ProcessMouseUp_body(this: *mut CMenuFocusManager, msg: *mut MSG, isLButton: BOOL) -> LRESULT {

    let child: HWND;
    let mut iHitTestResult: i32 = -1;

    let cCapture: HWND = GetCapture();
    if !cCapture.is_null()
        && cCapture != (*this).m_captureHwnd
        && (*(*this).m_current).type_ != TrackedMenuEntry
    {
        return TRUE as LRESULT;
    }

    let mut pt: POINT = (*msg).pt;

    child = WindowFromPoint(pt);

    let mut entry: *mut StackEntry = core::ptr::null_mut();
    if CMenuFocusManager_IsTrackedWindow_body(this, child, &mut entry as *mut *mut StackEntry) != S_OK {
        return TRUE as LRESULT;
    }

    if !entry.is_null() {
        ScreenToClient(child, &mut pt as *mut POINT);
        iHitTestResult = SendMessageW(
            child,
            TB_HITTEST,
            0,
            &mut pt as *mut POINT as LPARAM,
        ) as i32;

        if iHitTestResult >= 0 {
            CMenuBand_MenuBarMouseUp_RustExport((*entry).mb, child.cast(), iHitTestResult, isLButton);
        }
    }

    return TRUE as LRESULT;
}

pub unsafe fn CMenuFocusManager_MsgFilterHook_body(this: *mut CMenuFocusManager, nCode: INT, hookWParam: WPARAM, hookLParam: LPARAM) -> LRESULT {

    // Same GET_X_LPARAM/HIWORD helper as ProcessMouseMove.rs, duplicated locally — see the
    // comment there for why (item declarations outside a fn's own braces are dropped by
    // materialization, so this can't be shared module-scope).
    trait CppComWordSource { fn cpp_com_word_source(self) -> u32; }
    impl CppComWordSource for LPARAM { fn cpp_com_word_source(self) -> u32 { self as u32 } }
    impl CppComWordSource for WPARAM { fn cpp_com_word_source(self) -> u32 { self as u32 } }

    #[inline]
    unsafe fn GET_X_LPARAM<T: CppComWordSource>(lp: T) -> INT {
        (lp.cpp_com_word_source() as u16) as i16 as INT
    }
    #[inline]
    unsafe fn HIWORD<T: CppComWordSource>(l: T) -> WORD {
        (l.cpp_com_word_source() >> 16) as WORD
    }

    if nCode < 0
    {
        return CallNextHookEx((*this).m_hMsgFilterHook, nCode, hookWParam, hookLParam);
    }

    if nCode == MSGF_MENU as INT
    {
        let mut callNext: BOOL = TRUE as BOOL;
        let msg: *mut MSG = hookLParam as *mut MSG;

        match (*msg).message
        {
            WM_LBUTTONDOWN | WM_RBUTTONDOWN =>
            {
                if !(*this).m_menuBar.is_null() && (*(*this).m_current).type_ == TrackedMenuEntry
                {
                    let pt = (*msg).pt;
                    let child = WindowFromPoint(pt);
                    let hoveringMenuBar: BOOL = (CMenuBand_IsWindowOwner_RustExport((*(*this).m_menuBar).mb, child.cast()) == S_OK) as BOOL;
                    if hoveringMenuBar != 0
                    {
                        CMenuBand_BeforeCancelPopup_RustExport((*(*this).m_menuBar).mb);
                    }
                }
            }
            WM_MOUSEMOVE =>
            {
                callNext = CMenuFocusManager_ProcessMouseMove_body(this, msg) as BOOL;
            }
            WM_INITMENUPOPUP =>
            {
                TRACE!("WM_INITMENUPOPUP %p %p\n", (*msg).wParam, (*msg).lParam);
                (*this).m_selectedMenu = (*msg).lParam as HMENU;
                (*this).m_selectedItem = (-(1 as INT)) as _;
                (*this).m_selectedItemFlags = 0 as _;
            }
            WM_MENUSELECT =>
            {
                TRACE!("WM_MENUSELECT %p %p\n", (*msg).wParam, (*msg).lParam);
                (*this).m_selectedMenu = (*msg).lParam as HMENU;
                (*this).m_selectedItem = GET_X_LPARAM((*msg).wParam) as _;
                (*this).m_selectedItemFlags = HIWORD((*msg).wParam) as _;
            }
            WM_KEYDOWN =>
            {
                match (*msg).wParam as DWORD
                {
                    VK_LEFT =>
                    {
                        if (*(*this).m_current).hmenu == (*this).m_selectedMenu
                        {
                            CMenuBand_MenuItemSelect_RustExport((*(*this).m_parent).mb, VK_LEFT);
                        }
                    }
                    VK_RIGHT =>
                    {
                        if (*this).m_selectedItem < (0 as INT) || ((*this).m_selectedItemFlags & (MF_POPUP as DWORD)) == 0
                        {
                            CMenuBand_MenuItemSelect_RustExport((*(*this).m_parent).mb, VK_RIGHT);
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }

        if callNext == 0
        {
            return 1;
        }
    }

    return CallNextHookEx((*this).m_hMsgFilterHook, nCode, hookWParam, hookLParam);
}

pub unsafe fn CMenuFocusManager_GetMsgHook_body(this: *mut CMenuFocusManager, nCode: INT, hookWParam: WPARAM, hookLParam: LPARAM) -> LRESULT {

    let mut isLButton: BOOL = FALSE as BOOL;
    if nCode < 0 {
        return CallNextHookEx((*this).m_hGetMsgHook, nCode, hookWParam, hookLParam);
    }

    if nCode == HC_ACTION as INT {
        let mut callNext: BOOL = TRUE as BOOL;
        let msg = hookLParam as *mut MSG;
        let pt = (*msg).pt;

        'msg_switch: {
            match (*msg).message {
                WM_CAPTURECHANGED => {
                    if !(*this).m_captureHwnd.is_null() {
                        (*this).m_captureHwnd = core::ptr::null_mut();
                    }
                }
                WM_NCLBUTTONDOWN | WM_LBUTTONDBLCLK | WM_LBUTTONDOWN => {
                    isLButton = TRUE as BOOL;

                    if !(*this).m_menuBar.is_null() && (*(*this).m_current).type_ == MenuPopupEntry {
                        let pt = (*msg).pt;
                        let child = WindowFromPoint(pt);
                        let hoveringMenuBar = CMenuBand_IsWindowOwner_RustExport((*(*this).m_menuBar).mb, child.cast()) == S_OK;
                        if hoveringMenuBar {
                            CMenuBand_MenuItemSelect_RustExport((*(*this).m_current).mb, MPOS_FULLCANCEL);
                            break 'msg_switch;
                        }
                    }

                    if (*(*this).m_current).type_ == MenuPopupEntry {
                        let child = WindowFromPoint(pt);
                        if CMenuFocusManager_IsTrackedWindowOrParent_body(this, child) != S_OK {
                            CMenuBand_MenuItemSelect_RustExport((*(*this).m_current).mb, MPOS_FULLCANCEL);
                            break 'msg_switch;
                        }
                    }

                    CMenuFocusManager_ProcessMouseDown_body(this, msg, isLButton);
                }
                WM_NCRBUTTONUP | WM_RBUTTONUP => {
                    CMenuFocusManager_ProcessMouseUp_body(this, msg, isLButton);
                }
                WM_NCLBUTTONUP | WM_LBUTTONUP => {
                    isLButton = TRUE as BOOL;
                    CMenuFocusManager_ProcessMouseUp_body(this, msg, isLButton);
                }
                WM_MOUSEMOVE => {
                    callNext = CMenuFocusManager_ProcessMouseMove_body(this, msg) as BOOL;
                }
                WM_MOUSELEAVE => {
                    callNext = CMenuFocusManager_ProcessMouseMove_body(this, msg) as BOOL;
                }
                WM_SYSKEYDOWN | WM_KEYDOWN => {
                    if (*(*this).m_current).type_ == MenuPopupEntry {
                        CMenuFocusManager_DisableMouseTrack_body(this, (*(*this).m_current).hwnd, TRUE as BOOL);

                        match (*msg).wParam as DWORD {
                            VK_ESCAPE | VK_MENU | VK_LMENU | VK_RMENU => {
                                CMenuBand_MenuItemSelect_RustExport((*(*this).m_current).mb, MPOS_FULLCANCEL);
                            }
                            VK_RETURN => {
                                CMenuBand_MenuItemSelect_RustExport((*(*this).m_current).mb, MPOS_EXECUTE);
                            }
                            VK_LEFT => {
                                CMenuBand_MenuItemSelect_RustExport((*(*this).m_current).mb, VK_LEFT);
                            }
                            VK_RIGHT => {
                                CMenuBand_MenuItemSelect_RustExport((*(*this).m_current).mb, VK_RIGHT);
                            }
                            VK_UP => {
                                CMenuBand_MenuItemSelect_RustExport((*(*this).m_current).mb, VK_UP);
                            }
                            VK_DOWN => {
                                CMenuBand_MenuItemSelect_RustExport((*(*this).m_current).mb, VK_DOWN);
                            }
                            _ => {}
                        }

                        (*msg).message = WM_NULL;
                        (*msg).lParam = 0;
                        (*msg).wParam = 0;
                    }
                }
                _ => {}
            }
        }

        if callNext == 0 {
            return 1;
        }
    }

    return CallNextHookEx((*this).m_hGetMsgHook, nCode, hookWParam, hookLParam);
}

pub unsafe fn CMenuFocusManager_PlaceHooks_body(this: *mut CMenuFocusManager) -> HRESULT {
    // The two HOOKPROCs are ABI declarations, which the worker audit refuses in a body
    // (the scaffold owns ABI artifacts). They live in menu_focus_manager_support.rs.
    let s_MsgFilterHook: HOOKPROC = Some(crate::menu_focus_manager_support::s_MsgFilterHook);
    let s_GetMsgHook: HOOKPROC = Some(crate::menu_focus_manager_support::s_GetMsgHook);

    if (*this).m_hGetMsgHook as usize != 0 {
        return S_OK;
    }
    if (*this).m_hMsgFilterHook as usize != 0 {
        return S_OK;
    }
    if (*(*this).m_current).type_ == TrackedMenuEntry {
        (*this).m_hMsgFilterHook = SetWindowsHookExW(WH_MSGFILTER as i32, s_MsgFilterHook, core::ptr::null_mut(), (*this).m_threadId);
    } else {
        (*this).m_hGetMsgHook = SetWindowsHookExW(WH_GETMESSAGE as i32, s_GetMsgHook, core::ptr::null_mut(), (*this).m_threadId);
    }
    return S_OK;
}

pub unsafe fn CMenuFocusManager_RemoveHooks_body(this: *mut CMenuFocusManager) -> HRESULT {
    if (*this).m_hMsgFilterHook != core::ptr::null_mut() {
        UnhookWindowsHookEx((*this).m_hMsgFilterHook);
        (*this).m_hMsgFilterHook = core::ptr::null_mut();
    }
    if (*this).m_hGetMsgHook != core::ptr::null_mut() {
        UnhookWindowsHookEx((*this).m_hGetMsgHook);
        (*this).m_hGetMsgHook = core::ptr::null_mut();
    }
    return S_OK;
}

pub unsafe fn CMenuFocusManager_UpdateFocus_body(this: *mut CMenuFocusManager) -> HRESULT {

    let mut hr: HRESULT;
    let mut old: *mut StackEntry = (*this).m_current;

    if (*this).m_bandCount > 0 {
        (*this).m_current = core::ptr::addr_of_mut!(
            (*this).m_bandStack[((*this).m_bandCount as usize) - (1 as usize)]
        );
    } else {
        (*this).m_current = core::ptr::null_mut();
    }

    if (*this).m_current.is_null() || (*(*this).m_current).type_ != MenuPopupEntry {
        CMenuFocusManager_SetMenuCapture_body(this, core::ptr::null_mut());
        if !old.is_null()
            && (*old).type_ == MenuPopupEntry
            && !(*this).m_PreviousForeground.is_null()
        {
            SetForegroundWindow((*this).m_PreviousForeground);
            (*this).m_PreviousForeground = core::ptr::null_mut();
        }
    }

    if !(*this).m_current.is_null() && (*(*this).m_current).type_ != TrackedMenuEntry {
        hr = CMenuBand_GetTopLevelWindow_RustExport(
            (*(*this).m_current).mb,
            core::ptr::addr_of_mut!((*(*this).m_current).hwnd).cast(),
        );
        if FAILED_UNEXPECTEDLY!(hr) {
            return hr;
        }
    }

    if (*this).m_bandCount >= 2 {
        (*this).m_parent = core::ptr::addr_of_mut!(
            (*this).m_bandStack[((*this).m_bandCount as usize) - (2 as usize)]
        );
        _ASSERT!((*(*this).m_parent).type_ != TrackedMenuEntry);
    } else {
        (*this).m_parent = core::ptr::null_mut();
    }

    if (*this).m_bandCount >= 1
        && (*this).m_bandStack[0 as usize].type_ == MenuBarEntry
    {
        (*this).m_menuBar =
            core::ptr::addr_of_mut!((*this).m_bandStack[0 as usize]);
    } else {
        (*this).m_menuBar = core::ptr::null_mut();
    }

    if !old.is_null()
        && ((*this).m_current.is_null() || (*old).type_ != (*(*this).m_current).type_)
    {
        if !(*this).m_current.is_null() && (*(*this).m_current).type_ != TrackedMenuEntry {
            CMenuFocusManager_DisableMouseTrack_body(
                this,
                (*(*this).m_current).hwnd,
                FALSE as BOOL,
            );
        }

        hr = CMenuFocusManager_RemoveHooks_body(this);
        if FAILED_UNEXPECTEDLY!(hr) {
            return hr;
        }
    }

    if !(*this).m_current.is_null()
        && (old.is_null() || (*old).type_ != (*(*this).m_current).type_)
    {
        hr = CMenuFocusManager_PlaceHooks_body(this);
        if FAILED_UNEXPECTEDLY!(hr) {
            return hr;
        }
    }

    if !(*this).m_parent.is_null() {
        CMenuFocusManager_DisableMouseTrack_body(this, (*(*this).m_parent).hwnd, TRUE as BOOL);
    }

    if !(*this).m_current.is_null() && (*(*this).m_current).type_ == MenuPopupEntry {
        if (*this).m_captureHwnd.is_null() {
            'NoCapture: {
                let mut topMenu: *mut StackEntry =
                    core::ptr::addr_of_mut!((*this).m_bandStack[0]);
                if (*topMenu).type_ == MenuBarEntry {
                    topMenu = topMenu.add(1);
                }

                let mut bandSite: *mut IServiceProvider = core::ptr::null_mut();
                let _bandSite_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(bandSite));
                let mut deskBar: *mut IOleWindow = core::ptr::null_mut();
                let _deskBar_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(deskBar));

                hr = CMenuBand_GetSite_RustExport(
                    (*topMenu).mb,
                    core::ptr::addr_of!(IID_IServiceProvider),
                    core::ptr::addr_of_mut!(bandSite).cast(),
                );
                if hr < 0 {
                    break 'NoCapture;
                }

                hr = ((*(*bandSite).lpVtbl).QueryService)(
                    bandSite,
                    core::ptr::addr_of!(SID_SMenuPopup),
                    core::ptr::addr_of!(IID_IOleWindow),
                    core::ptr::addr_of_mut!(deskBar).cast(),
                );
                if hr < 0 {
                    break 'NoCapture;
                }

                let mut deskBarSite: *mut IOleWindow = core::ptr::null_mut();
                let _deskBarSite_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(deskBarSite));
                hr = IUnknown_GetSite(
                    deskBar.cast(),
                    core::ptr::addr_of!(IID_IOleWindow),
                    core::ptr::addr_of_mut!(deskBarSite).cast(),
                );
                if hr < 0 {
                    break 'NoCapture;
                }

                let mut hWndOwner: HWND = core::ptr::null_mut();
                hr = ((*(*deskBarSite).lpVtbl).GetWindow)(deskBarSite, &mut hWndOwner);
                if hr < 0 {
                    break 'NoCapture;
                }

                (*this).m_PreviousForeground = GetForegroundWindow();
                if (*this).m_PreviousForeground != hWndOwner {
                    SetForegroundWindow(hWndOwner);
                } else {
                    (*this).m_PreviousForeground = core::ptr::null_mut();
                }

                let mut hWndSite: HWND = core::ptr::null_mut();
                hr = ((*(*deskBar).lpVtbl).GetWindow)(deskBar, &mut hWndSite);
                if hr < 0 {
                    break 'NoCapture;
                }

                CMenuFocusManager_SetMenuCapture_body(this, hWndSite);
            }
        }

        if (*this).m_parent.is_null() || (*(*this).m_parent).type_ == MenuBarEntry {
            if !old.is_null() && (*old).type_ == TrackedMenuEntry {
                let mut pt2: POINT = core::mem::zeroed();
                let mut rc2: RECT = core::mem::zeroed();

                GetCursorPos(&mut pt2);
                ScreenToClient((*(*this).m_current).hwnd, &mut pt2);
                GetClientRect((*(*this).m_current).hwnd, &mut rc2);

                if PtInRect(core::ptr::addr_of!(rc2), pt2) != 0 {
                    SendMessageW(
                        (*(*this).m_current).hwnd,
                        WM_MOUSEMOVE,
                        0,
                        MAKELPARAM!(pt2.x, pt2.y),
                    );
                } else {
                    SendMessageW(
                        (*(*this).m_current).hwnd,
                        WM_MOUSELEAVE,
                        0,
                        0,
                    );
                }
            }
        }
    }

    _ASSERT!((*this).m_parent.is_null() || (*(*this).m_parent).type_ != TrackedMenuEntry);

    return S_OK;
}

pub unsafe fn CMenuFocusManager_PushMenuBar_body(this: *mut CMenuFocusManager, mb: *mut CMenuBand) -> HRESULT {

    CMenuBand_AddRef_RustExport(mb);

    _ASSERT!((*this).m_bandCount == 0);

    let hr: HRESULT = CMenuFocusManager_PushToArray_body(this, MenuBarEntry, mb, core::ptr::null_mut());
    if (((hr) < 0)) {
        return hr;
    }

    return CMenuFocusManager_UpdateFocus_body(this);
}

pub unsafe fn CMenuFocusManager_PushMenuPopup_body(this: *mut CMenuFocusManager, mb: *mut CMenuBand) -> HRESULT {

    CMenuBand_AddRef_RustExport(mb);

    _ASSERT!((*this).m_current.is_null() || (*(*this).m_current).type_ != StackEntryType::TrackedMenuEntry);

    let mut hr: HRESULT = CMenuFocusManager_PushToArray_body(this, MenuPopupEntry, mb, core::ptr::null_mut());
    if ((hr) < 0) {
        return hr;
    }

    hr = CMenuFocusManager_UpdateFocus_body(this);

    (*this).m_menuDepth += 1;

    if !(*this).m_parent.is_null() && (*(*this).m_parent).type_ != TrackedMenuEntry {
        CMenuBand_SetChildBand_RustExport((*(*this).m_parent).mb, mb);
        CMenuBand_SetParentBand_RustExport(mb, (*(*this).m_parent).mb);
    }

    return hr;
}

pub unsafe fn CMenuFocusManager_PushTrackedPopup_body(this: *mut CMenuFocusManager, popup: HMENU) -> HRESULT {
    _ASSERT!((*this).m_bandCount > 0);
    _ASSERT!((*this).m_current.is_null() || (*(*this).m_current).type_ != TrackedMenuEntry);

    let hr: HRESULT = CMenuFocusManager_PushToArray_body(this, TrackedMenuEntry, core::ptr::null_mut(), popup);
    if ((hr) < 0) {
        return hr;
    }

    (*this).m_selectedMenu = popup;
    (*this).m_selectedItem = -1;
    (*this).m_selectedItemFlags = 0;

    return CMenuFocusManager_UpdateFocus_body(this);
}

pub unsafe fn CMenuFocusManager_PopMenuBar_body(this: *mut CMenuFocusManager, mb: *mut CMenuBand) -> HRESULT {

    let mut r#type: StackEntryType = MenuBarEntry;
    let mut mbc: *mut CMenuBand = core::ptr::null_mut();
    let mut hr: HRESULT;

    if (*this).m_current == (*this).m_entryUnderMouse {
        (*this).m_entryUnderMouse = core::ptr::null_mut();
    }

    hr = CMenuFocusManager_PopFromArray_body(this, &mut r#type, &mut mbc, core::ptr::null_mut());
    if ((hr) < 0) {
        CMenuFocusManager_UpdateFocus_body(this);
        return hr;
    }

    _ASSERT!(r#type == MenuBarEntry);
    if r#type != MenuBarEntry {
        return E_FAIL;
    }

    if mbc.is_null() {
        return E_FAIL;
    }

    CMenuBand_SetParentBand_RustExport(mbc, core::ptr::null_mut());

    /* Balances the AddRef in PushMenuBar */
    CMenuBand_Release_RustExport(mbc);

    hr = CMenuFocusManager_UpdateFocus_body(this);
    if ((hr) < 0) {
        return hr;
    }

    if !(*this).m_current.is_null() {
        _ASSERT!((*(*this).m_current).type_ != TrackedMenuEntry);
        CMenuBand_SetChildBand_RustExport((*(*this).m_current).mb, core::ptr::null_mut());
    }

    return S_OK;
}

pub unsafe fn CMenuFocusManager_PopMenuPopup_body(this: *mut CMenuFocusManager, mb: *mut CMenuBand) -> HRESULT {

    if (*this).m_current == (*this).m_entryUnderMouse {
        (*this).m_entryUnderMouse = core::ptr::null_mut();
    }

    (*this).m_menuDepth = (*this).m_menuDepth.wrapping_sub(1);

    let mut r#type = core::mem::MaybeUninit::<StackEntryType>::uninit();
    let mut mbc: *mut CMenuBand = core::ptr::null_mut();
    let mut hr: HRESULT;

    hr = CMenuFocusManager_PopFromArray_body(this, r#type.as_mut_ptr(), &mut mbc, core::ptr::null_mut());
    if ((hr) < 0) {
        CMenuFocusManager_UpdateFocus_body(this);
        return hr;
    }

    let r#type: StackEntryType = r#type.assume_init();

    _ASSERT!(r#type == MenuPopupEntry);
    if r#type != MenuPopupEntry {
        return E_FAIL;
    }

    if mbc.is_null() {
        return E_FAIL;
    }

    CMenuBand_SetParentBand_RustExport(mbc, core::ptr::null_mut());

    CMenuBand_Release_RustExport(mbc);

    hr = CMenuFocusManager_UpdateFocus_body(this);
    if ((hr) < 0) {
        return hr;
    }

    if !(*this).m_current.is_null() {
        _ASSERT!((*(*this).m_current).type_ != TrackedMenuEntry);
        CMenuBand_SetChildBand_RustExport((*(*this).m_current).mb, core::ptr::null_mut());
    }

    return S_OK;
}

pub unsafe fn CMenuFocusManager_PopTrackedPopup_body(this: *mut CMenuFocusManager, popup: HMENU) -> HRESULT {
    let mut type_ = core::mem::MaybeUninit::<StackEntryType>::uninit();
    let mut hmenu = core::mem::MaybeUninit::<HMENU>::uninit();

    let hr = CMenuFocusManager_PopFromArray_body(this, type_.as_mut_ptr(), core::ptr::null_mut(), hmenu.as_mut_ptr());
    if ((hr) < 0) {
        CMenuFocusManager_UpdateFocus_body(this);
        return hr;
    }

    let type_ = unsafe { type_.assume_init() };
    let hmenu = unsafe { hmenu.assume_init() };

    _ASSERT!(type_ == TrackedMenuEntry);
    if type_ != TrackedMenuEntry {
        return E_FAIL;
    }

    if hmenu != popup {
        return E_FAIL;
    }

    let hr = CMenuFocusManager_UpdateFocus_body(this);
    if ((hr) < 0) {
        return hr;
    }

    return S_OK;
}
