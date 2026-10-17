// rustconv-assembly: class:CDefaultContextMenu
use crate::atl;
// The crate's own functions: the definition is the declaration.
use crate::shell32::shlfileop::SHCreateDirectory;
use crate::wine::pidl::SHILCreateFromPathW;
// One declaration per external symbol: island.rs.
use crate::island::DestroyMenu;
use crate::island::GetModuleHandleW;
use crate::island::GlobalAlloc;
use crate::island::HeapReAlloc;
use crate::island::IUnknown_QueryService;
use crate::island::InsertMenuItemW;
use crate::island::LoadStringW;
use crate::island::RegCloseKey;
use crate::island::RegGetValueW;
use crate::island::RegOpenKeyExW;
use crate::shellcom::REFGUID;
use crate::prelude::*;
use crate::default_context_menu_support::*;
// The COM interfaces this half calls through a vtable. An explicit `use` beats the prelude's
// island glob, which renders these as bindgen C++ classes with an opaque `vtable_` (no
// `lpVtbl` field) or, via com-sys, with `Option<fn>` slots that no call site can invoke
// without putting a panic on every COM call. Same boundary the other 40 landed COM modules
// sit on; prelude.rs's header states the rule and why it is not a blanket re-export.
// `IContextMenu` is NOT imported: this file declares its own just below, with every slot
// typed, and it is the definition the TU half and the support module both use.
use crate::shellcom::{IDropTarget, IDropTargetVtbl};
use crate::default_context_menu_support::{IShellFolderView, IShellFolderViewVtbl};
// `StrIsNullOrEmpty` is overloaded in C++; Rust needs the A and W names spelled out.
use crate::shell_link::{StrIsNullOrEmptyA, StrIsNullOrEmptyW};
// The TU's file-scope verb table and free helpers live in the --free-functions half.
// One definition, imported, rather than a second copy that could drift from it.
use crate::default_context_menu_tu::{
    g_StaticInvokeCmdMap, MapFcidmCmdToVerb, MapVerbToDfmCmd, SHGetMenuIdFromMenuMsg,
    SHELL32_EnumDefaultVerbList, _StaticInvokeCommandMap_,
};
// The local ATL fallbacks the cpp-com support block emits were REMOVED: this crate has
// the real ports in `crate::atl`, and a local `pub struct` beats the prelude glob, so
// keeping them silently shadowed the genuine types. That mattered beyond missing methods
// (`GetString`, `CompareNoCase`): the fallback CStringW is a 3-word
// {buffer,len,capacity} struct where ATL's is ONE pointer, and `StaticShellEntry` is
// #[repr(C)] with a `CStringW Verb` field — so the shadow silently changed that struct's
// layout away from the C++ it has to match.
// Deterministic from shobjidl.h
#[repr(C)]
pub struct IContextMenu {
    pub lpVtbl: *const IContextMenuVtbl,
}
#[repr(C)]
pub struct IContextMenuVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IContextMenu, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IContextMenu) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IContextMenu) -> ULONG,
    pub QueryContextMenu: unsafe extern "system" fn(This: *mut IContextMenu, hmenu: HMENU, indexMenu: UINT, idCmdFirst: UINT, idCmdLast: UINT, uFlags: UINT) -> HRESULT,
    pub InvokeCommand: unsafe extern "system" fn(This: *mut IContextMenu, lpici: LPCMINVOKECOMMANDINFO) -> HRESULT,
    pub GetCommandString: unsafe extern "system" fn(This: *mut IContextMenu, idCmd: UINT_PTR, uType: UINT, pwReserved: *mut UINT, pszName: LPSTR, cchMax: UINT) -> HRESULT,
}
extern "C" {
    pub static IID_IContextMenu: IID;
}

// Deterministic from shobjidl.h
#[repr(C)]
pub struct IContextMenu2 {
    pub lpVtbl: *const IContextMenu2Vtbl,
}
#[repr(C)]
pub struct IContextMenu2Vtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IContextMenu2, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IContextMenu2) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IContextMenu2) -> ULONG,
    pub QueryContextMenu: unsafe extern "system" fn(This: *mut IContextMenu2, hmenu: HMENU, indexMenu: UINT, idCmdFirst: UINT, idCmdLast: UINT, uFlags: UINT) -> HRESULT,
    pub InvokeCommand: unsafe extern "system" fn(This: *mut IContextMenu2, lpici: LPCMINVOKECOMMANDINFO) -> HRESULT,
    pub GetCommandString: unsafe extern "system" fn(This: *mut IContextMenu2, idCmd: UINT_PTR, uType: UINT, pwReserved: *mut UINT, pszName: LPSTR, cchMax: UINT) -> HRESULT,
    pub HandleMenuMsg: unsafe extern "system" fn(This: *mut IContextMenu2, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> HRESULT,
}
extern "C" {
    pub static IID_IContextMenu2: IID;
}

// Deterministic from shobjidl.h
#[repr(C)]
pub struct IContextMenu3 {
    pub lpVtbl: *const IContextMenu3Vtbl,
}
#[repr(C)]
pub struct IContextMenu3Vtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IContextMenu3, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IContextMenu3) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IContextMenu3) -> ULONG,
    pub QueryContextMenu: unsafe extern "system" fn(This: *mut IContextMenu3, hmenu: HMENU, indexMenu: UINT, idCmdFirst: UINT, idCmdLast: UINT, uFlags: UINT) -> HRESULT,
    pub InvokeCommand: unsafe extern "system" fn(This: *mut IContextMenu3, lpici: LPCMINVOKECOMMANDINFO) -> HRESULT,
    pub GetCommandString: unsafe extern "system" fn(This: *mut IContextMenu3, idCmd: UINT_PTR, uType: UINT, pwReserved: *mut UINT, pszName: LPSTR, cchMax: UINT) -> HRESULT,
    pub HandleMenuMsg: unsafe extern "system" fn(This: *mut IContextMenu3, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> HRESULT,
    pub HandleMenuMsg2: unsafe extern "system" fn(This: *mut IContextMenu3, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, plResult: *mut LRESULT) -> HRESULT,
}
extern "C" {
    pub static IID_IContextMenu3: IID;
}

// Deterministic from ocidl.h
#[repr(C)]
pub struct IObjectWithSite {
    pub lpVtbl: *const IObjectWithSiteVtbl,
}
#[repr(C)]
pub struct IObjectWithSiteVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IObjectWithSite, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IObjectWithSite) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IObjectWithSite) -> ULONG,
    pub SetSite: unsafe extern "system" fn(This: *mut IObjectWithSite, pUnkSite: *mut IUnknown) -> HRESULT,
    pub GetSite: unsafe extern "system" fn(This: *mut IObjectWithSite, riid: REFIID, ppvSite: *mut PVOID) -> HRESULT,
}
extern "C" {
    pub static IID_IObjectWithSite: IID;
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
    pub QueryService: unsafe extern "system" fn(This: *mut IServiceProvider, guidService: REFIID, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
}
extern "C" {
    pub static IID_IServiceProvider: IID;
}

// Deterministic from unknwn.h
#[repr(C)]
pub struct IUnknown {
    pub lpVtbl: *const IUnknownVtbl,
}
#[repr(C)]
pub struct IUnknownVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IUnknown, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IUnknown) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IUnknown) -> ULONG,
}
extern "C" {
    pub static IID_IUnknown: IID;
}

#[repr(C)]
pub struct DynamicShellEntry {
    pub iIdCmdFirst: UINT,
    pub NumIds: UINT,
    pub ClassID: CLSID,
    pub pCM: CComPtr<IContextMenu>,
}

/// C++: `typedef struct _StaticShellEntry_ { ... } StaticShellEntry, *PStaticShellEntry;`
pub type PStaticShellEntry = *mut StaticShellEntry;
pub type PDynamicShellEntry = *mut DynamicShellEntry;

#[repr(C)]
pub struct StaticShellEntry {
    pub Verb: CStringW,
    pub hkClass: HKEY,
}

// Deterministic COM scaffold for CDefaultContextMenu; method bodies are generated separately.
extern "system" {
    fn GetProcessHeap() -> HANDLE;
    #[link_name = "HeapAlloc"] fn cpp_com_HeapAlloc(heap: HANDLE, flags: DWORD, bytes: usize) -> *mut core::ffi::c_void;
    #[link_name = "HeapFree"] fn cpp_com_HeapFree(heap: HANDLE, flags: DWORD, memory: *mut core::ffi::c_void) -> BOOL;
    fn Shell32RustObjectLock();
    fn Shell32RustObjectUnlock();
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

#[repr(C)]
pub struct CDefaultContextMenu {
    pub IContextMenu3_iface: IContextMenu3,
    pub IObjectWithSite_iface: IObjectWithSite,
    pub IServiceProvider_iface: IServiceProvider,
    pub ref_count: core::sync::atomic::AtomicI32,
    pub m_site: *mut IUnknown,
    pub m_psf: *mut IShellFolder,
    pub m_pmcb: *mut IContextMenuCB,
    pub m_pfnmcb: LPFNDFMCALLBACK,
    pub m_cidl: UINT,
    pub m_apidl: PCUITEMID_CHILD_ARRAY,
    pub m_pDataObj: *mut IDataObject,
    pub m_aKeys: [HKEY; 16],
    pub m_cKeys: UINT,
    pub m_pidlFolder: PIDLIST_ABSOLUTE,
    pub m_bGroupPolicyActive: DWORD,
    pub m_iIdQCMFirst: UINT,
    pub m_DynamicEntries: CAtlList<DynamicShellEntry>,
    pub m_iIdSHEFirst: UINT,
    pub m_iIdSHELast: UINT,
    pub m_StaticEntries: CAtlList<StaticShellEntry>,
    pub m_iIdSCMFirst: UINT,
    pub m_iIdSCMLast: UINT,
    pub m_iIdCBFirst: UINT,
    pub m_iIdCBLast: UINT,
    pub m_iIdDfltFirst: UINT,
    pub m_iIdDfltLast: UINT,
    pub m_hwnd: HWND,
    pub m_DefVerbs: [WCHAR; 260],
}

unsafe fn CDefaultContextMenu_cpp_com_assign_m_site(this: *mut CDefaultContextMenu, value: *mut IUnknown) {
    let old = (*this).m_site;
    if old == value { return; }
    if !value.is_null() {
        let unknown = value as *mut IUnknown;
        let add_ref: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).AddRef);
        add_ref(unknown);
    }
    (*this).m_site = value;
    if !old.is_null() {
        let unknown = old as *mut IUnknown;
        let release: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).Release);
        release(unknown);
    }
}

unsafe fn CDefaultContextMenu_cpp_com_assign_m_psf(this: *mut CDefaultContextMenu, value: *mut IShellFolder) {
    let old = (*this).m_psf;
    if old == value { return; }
    if !value.is_null() {
        let unknown = value as *mut IUnknown;
        let add_ref: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).AddRef);
        add_ref(unknown);
    }
    (*this).m_psf = value;
    if !old.is_null() {
        let unknown = old as *mut IUnknown;
        let release: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).Release);
        release(unknown);
    }
}

unsafe fn CDefaultContextMenu_cpp_com_assign_m_pmcb(this: *mut CDefaultContextMenu, value: *mut IContextMenuCB) {
    let old = (*this).m_pmcb;
    if old == value { return; }
    if !value.is_null() {
        let unknown = value as *mut IUnknown;
        let add_ref: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).AddRef);
        add_ref(unknown);
    }
    (*this).m_pmcb = value;
    if !old.is_null() {
        let unknown = old as *mut IUnknown;
        let release: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).Release);
        release(unknown);
    }
}

unsafe fn CDefaultContextMenu_cpp_com_assign_m_pDataObj(this: *mut CDefaultContextMenu, value: *mut IDataObject) {
    let old = (*this).m_pDataObj;
    if old == value { return; }
    if !value.is_null() {
        let unknown = value as *mut IUnknown;
        let add_ref: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).AddRef);
        add_ref(unknown);
    }
    (*this).m_pDataObj = value;
    if !old.is_null() {
        let unknown = old as *mut IUnknown;
        let release: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).Release);
        release(unknown);
    }
}

unsafe extern "system" fn CDefaultContextMenu_IContextMenu3_QueryInterface(This: *mut IContextMenu3, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefaultContextMenu, IContextMenu3_iface)) as *mut CDefaultContextMenu;
    CDefaultContextMenu_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CDefaultContextMenu_IContextMenu3_AddRef(This: *mut IContextMenu3) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefaultContextMenu, IContextMenu3_iface)) as *mut CDefaultContextMenu;
    CDefaultContextMenu_AddRef(this)
}
unsafe extern "system" fn CDefaultContextMenu_IContextMenu3_Release(This: *mut IContextMenu3) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefaultContextMenu, IContextMenu3_iface)) as *mut CDefaultContextMenu;
    CDefaultContextMenu_Release(this)
}
unsafe extern "system" fn CDefaultContextMenu_IContextMenu3_QueryContextMenu(This: *mut IContextMenu3, hmenu: HMENU, indexMenu: UINT, idCmdFirst: UINT, idCmdLast: UINT, uFlags: UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefaultContextMenu, IContextMenu3_iface)) as *mut CDefaultContextMenu;
    CDefaultContextMenu_QueryContextMenu_body(this, hmenu, indexMenu, idCmdFirst, idCmdLast, uFlags)
}
unsafe extern "system" fn CDefaultContextMenu_IContextMenu3_InvokeCommand(This: *mut IContextMenu3, lpici: LPCMINVOKECOMMANDINFO) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefaultContextMenu, IContextMenu3_iface)) as *mut CDefaultContextMenu;
    CDefaultContextMenu_InvokeCommand_body(this, lpici)
}
unsafe extern "system" fn CDefaultContextMenu_IContextMenu3_GetCommandString(This: *mut IContextMenu3, idCmd: UINT_PTR, uType: UINT, pwReserved: *mut UINT, pszName: LPSTR, cchMax: UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefaultContextMenu, IContextMenu3_iface)) as *mut CDefaultContextMenu;
    CDefaultContextMenu_GetCommandString_body(this, idCmd, uType, pwReserved, pszName, cchMax)
}
unsafe extern "system" fn CDefaultContextMenu_IContextMenu3_HandleMenuMsg(This: *mut IContextMenu3, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefaultContextMenu, IContextMenu3_iface)) as *mut CDefaultContextMenu;
    CDefaultContextMenu_HandleMenuMsg_body(this, uMsg, wParam, lParam)
}
unsafe extern "system" fn CDefaultContextMenu_IContextMenu3_HandleMenuMsg2(This: *mut IContextMenu3, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, plResult: *mut LRESULT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefaultContextMenu, IContextMenu3_iface)) as *mut CDefaultContextMenu;
    CDefaultContextMenu_HandleMenuMsg2_body(this, uMsg, wParam, lParam, plResult)
}

pub static CDEFAULTCONTEXTMENU_ICONTEXTMENU3_VTBL: IContextMenu3Vtbl = IContextMenu3Vtbl {
    QueryInterface: CDefaultContextMenu_IContextMenu3_QueryInterface,
    AddRef: CDefaultContextMenu_IContextMenu3_AddRef,
    Release: CDefaultContextMenu_IContextMenu3_Release,
    QueryContextMenu: CDefaultContextMenu_IContextMenu3_QueryContextMenu,
    InvokeCommand: CDefaultContextMenu_IContextMenu3_InvokeCommand,
    GetCommandString: CDefaultContextMenu_IContextMenu3_GetCommandString,
    HandleMenuMsg: CDefaultContextMenu_IContextMenu3_HandleMenuMsg,
    HandleMenuMsg2: CDefaultContextMenu_IContextMenu3_HandleMenuMsg2,
};

unsafe extern "system" fn CDefaultContextMenu_IObjectWithSite_QueryInterface(This: *mut IObjectWithSite, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefaultContextMenu, IObjectWithSite_iface)) as *mut CDefaultContextMenu;
    CDefaultContextMenu_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CDefaultContextMenu_IObjectWithSite_AddRef(This: *mut IObjectWithSite) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefaultContextMenu, IObjectWithSite_iface)) as *mut CDefaultContextMenu;
    CDefaultContextMenu_AddRef(this)
}
unsafe extern "system" fn CDefaultContextMenu_IObjectWithSite_Release(This: *mut IObjectWithSite) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefaultContextMenu, IObjectWithSite_iface)) as *mut CDefaultContextMenu;
    CDefaultContextMenu_Release(this)
}
unsafe extern "system" fn CDefaultContextMenu_IObjectWithSite_SetSite(This: *mut IObjectWithSite, pUnkSite: *mut IUnknown) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefaultContextMenu, IObjectWithSite_iface)) as *mut CDefaultContextMenu;
    CDefaultContextMenu_SetSite_body(this, pUnkSite)
}
unsafe extern "system" fn CDefaultContextMenu_IObjectWithSite_GetSite(This: *mut IObjectWithSite, riid: REFIID, ppvSite: *mut PVOID) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefaultContextMenu, IObjectWithSite_iface)) as *mut CDefaultContextMenu;
    CDefaultContextMenu_GetSite_body(this, riid, ppvSite)
}

pub static CDEFAULTCONTEXTMENU_IOBJECTWITHSITE_VTBL: IObjectWithSiteVtbl = IObjectWithSiteVtbl {
    QueryInterface: CDefaultContextMenu_IObjectWithSite_QueryInterface,
    AddRef: CDefaultContextMenu_IObjectWithSite_AddRef,
    Release: CDefaultContextMenu_IObjectWithSite_Release,
    SetSite: CDefaultContextMenu_IObjectWithSite_SetSite,
    GetSite: CDefaultContextMenu_IObjectWithSite_GetSite,
};

unsafe extern "system" fn CDefaultContextMenu_IServiceProvider_QueryInterface(This: *mut IServiceProvider, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefaultContextMenu, IServiceProvider_iface)) as *mut CDefaultContextMenu;
    CDefaultContextMenu_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CDefaultContextMenu_IServiceProvider_AddRef(This: *mut IServiceProvider) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefaultContextMenu, IServiceProvider_iface)) as *mut CDefaultContextMenu;
    CDefaultContextMenu_AddRef(this)
}
unsafe extern "system" fn CDefaultContextMenu_IServiceProvider_Release(This: *mut IServiceProvider) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefaultContextMenu, IServiceProvider_iface)) as *mut CDefaultContextMenu;
    CDefaultContextMenu_Release(this)
}
unsafe extern "system" fn CDefaultContextMenu_IServiceProvider_QueryService(This: *mut IServiceProvider, guidService: REFIID, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefaultContextMenu, IServiceProvider_iface)) as *mut CDefaultContextMenu;
    CDefaultContextMenu_QueryService_body(this, guidService, riid, ppvObject)
}

pub static CDEFAULTCONTEXTMENU_ISERVICEPROVIDER_VTBL: IServiceProviderVtbl = IServiceProviderVtbl {
    QueryInterface: CDefaultContextMenu_IServiceProvider_QueryInterface,
    AddRef: CDefaultContextMenu_IServiceProvider_AddRef,
    Release: CDefaultContextMenu_IServiceProvider_Release,
    QueryService: CDefaultContextMenu_IServiceProvider_QueryService,
};

unsafe fn CDefaultContextMenu_QueryInterface(
    this: *mut CDefaultContextMenu, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IUnknown)) || cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IContextMenu)) {
        *ppv = core::ptr::addr_of_mut!((*this).IContextMenu3_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IContextMenu2)) {
        *ppv = core::ptr::addr_of_mut!((*this).IContextMenu3_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IContextMenu3)) {
        *ppv = core::ptr::addr_of_mut!((*this).IContextMenu3_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IObjectWithSite)) {
        *ppv = core::ptr::addr_of_mut!((*this).IObjectWithSite_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IServiceProvider)) {
        *ppv = core::ptr::addr_of_mut!((*this).IServiceProvider_iface) as *mut core::ffi::c_void;
    }
    else { return E_NOINTERFACE; }
    CDefaultContextMenu_AddRef(this);
    S_OK
}

unsafe fn CDefaultContextMenu_AddRef(this: *mut CDefaultContextMenu) -> ULONG {
    (*this).ref_count.fetch_add(1, core::sync::atomic::Ordering::SeqCst) as ULONG + 1
}

unsafe fn CDefaultContextMenu_Release(this: *mut CDefaultContextMenu) -> ULONG {
    let refs = (*this).ref_count.fetch_sub(1, core::sync::atomic::Ordering::SeqCst) - 1;
    if refs == 0 {
        CDefaultContextMenu_dtor_body(this);
        CDefaultContextMenu_cpp_com_assign_m_site(this, core::ptr::null_mut());
        CDefaultContextMenu_cpp_com_assign_m_psf(this, core::ptr::null_mut());
        CDefaultContextMenu_cpp_com_assign_m_pmcb(this, core::ptr::null_mut());
        CDefaultContextMenu_cpp_com_assign_m_pDataObj(this, core::ptr::null_mut());
        core::ptr::drop_in_place(&mut (*this).m_DynamicEntries);
        core::ptr::drop_in_place(&mut (*this).m_StaticEntries);
        Shell32RustObjectUnlock();
        HeapFree(GetProcessHeap(), 0, this as *mut core::ffi::c_void);
    }
    refs as ULONG
}

#[inline]
unsafe fn cpp_com_iid_equal(left: REFIID, right: REFIID) -> bool {
    !left.is_null() && !right.is_null()
        && (*left).Data1 == (*right).Data1
        && (*left).Data2 == (*right).Data2
        && (*left).Data3 == (*right).Data3
        && (*left).Data4 == (*right).Data4
}

// Internal activation: shellutils.h ShellObjectCreator<T> uses this non-aggregating factory.
pub(crate) unsafe extern "C" fn CDefaultContextMenu_CreateInstance(
    pdcm: *const DEFCONTEXTMENU,
    lpfn: LPFNDFMCALLBACK,
    riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    let this = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<CDefaultContextMenu>()) as *mut CDefaultContextMenu;
    if this.is_null() { return E_OUTOFMEMORY; }
    core::ptr::write(this, CDefaultContextMenu {
        IContextMenu3_iface: IContextMenu3 { lpVtbl: core::ptr::addr_of!(CDEFAULTCONTEXTMENU_ICONTEXTMENU3_VTBL) },
        IObjectWithSite_iface: IObjectWithSite { lpVtbl: core::ptr::addr_of!(CDEFAULTCONTEXTMENU_IOBJECTWITHSITE_VTBL) },
        IServiceProvider_iface: IServiceProvider { lpVtbl: core::ptr::addr_of!(CDEFAULTCONTEXTMENU_ISERVICEPROVIDER_VTBL) },
        ref_count: core::sync::atomic::AtomicI32::new(1),
        m_site: core::mem::zeroed(),
        m_psf: core::mem::zeroed(),
        m_pmcb: core::mem::zeroed(),
        m_pfnmcb: core::mem::zeroed(),
        m_cidl: core::mem::zeroed(),
        m_apidl: core::mem::zeroed(),
        m_pDataObj: core::mem::zeroed(),
        m_aKeys: core::mem::zeroed(),
        m_cKeys: core::mem::zeroed(),
        m_pidlFolder: core::mem::zeroed(),
        m_bGroupPolicyActive: core::mem::zeroed(),
        m_iIdQCMFirst: core::mem::zeroed(),
        m_DynamicEntries: CAtlList::new(),
        m_iIdSHEFirst: core::mem::zeroed(),
        m_iIdSHELast: core::mem::zeroed(),
        m_StaticEntries: CAtlList::new(),
        m_iIdSCMFirst: core::mem::zeroed(),
        m_iIdSCMLast: core::mem::zeroed(),
        m_iIdCBFirst: core::mem::zeroed(),
        m_iIdCBLast: core::mem::zeroed(),
        m_iIdDfltFirst: core::mem::zeroed(),
        m_iIdDfltLast: core::mem::zeroed(),
        m_hwnd: core::mem::zeroed(),
        m_DefVerbs: core::mem::zeroed(),
    });
    Shell32RustObjectLock();
    CDefaultContextMenu_ctor_body(this);
    let mut hr = CDefaultContextMenu_Initialize_body(this, pdcm, lpfn);
    if hr >= 0 {
        hr = CDefaultContextMenu_QueryInterface(this, riid, ppv);
    }
    CDefaultContextMenu_Release(this);
    hr
}

// Deterministic C++ support closure selected from source identifiers.

pub const DCM_FCIDM_SHVIEW_OFFSET: UINT = 0x7000;

pub const FCIDM_SHVIEW_COPY: UINT = 0x7019;

pub const FCIDM_SHVIEW_COPYTO: UINT = 0x701E;

pub const FCIDM_SHVIEW_CREATELINK: UINT = 0x7010;

pub const FCIDM_SHVIEW_CUT: UINT = 0x7018;

pub const FCIDM_SHVIEW_DELETE: UINT = 0x7011;

pub const FCIDM_SHVIEW_INSERT: UINT = 0x701A;

pub const FCIDM_SHVIEW_INSERTLINK: UINT = 0x701C;

pub const FCIDM_SHVIEW_MOVETO: UINT = 0x701F;

pub const FCIDM_SHVIEW_NEWFOLDER: UINT = 0x7053;

pub const FCIDM_SHVIEW_PROPERTIES: UINT = 0x7013;

pub const FCIDM_SHVIEW_RENAME: UINT = 0x7012;

pub const FCIDM_SHVIEW_UNDO: UINT = 0x701B;

pub const IDS_EDIT_VERB: UINT = 303;

pub const IDS_EXPLORE_VERB: UINT = 301;

pub const IDS_FIND_VERB: UINT = 304;

pub const IDS_NEWFOLDER: UINT = 149;

pub const IDS_OPEN_VERB: UINT = 300;

pub const IDS_PRINT_VERB: UINT = 305;

pub const IDS_RUNAS_VERB: UINT = 302;

pub const MAX_VERB: usize = 260;

pub const MENU_SHV_FILE: UINT = 210;

pub const SEE_MASK_IDLIST: ULONG = 0x0000_0004;

pub const SFGAO_CANCOPY: SFGAOF = 0x0000_0001;

pub const SFGAO_CANDELETE: SFGAOF = 0x0000_0020;

pub const SFGAO_CANLINK: SFGAOF = 0x0000_0004;

pub const SFGAO_CANRENAME: SFGAOF = 0x0000_0010;

pub const SFGAO_HASPROPSHEET: SFGAOF = 0x0000_0040;

pub const CHARS_IN_GUID: DWORD = 39;

pub const CMF_CANRENAME: DWORD = 0x00000010;

pub const CMF_DEFAULTONLY: DWORD = 0x00000001;

pub const CMF_DISABLEDVERBS: DWORD = 0x00000200;

pub const CMF_DONOTPICKDEFAULT: DWORD = 0x00002000;

pub const CMF_EXPLORE: DWORD = 0x00000004;

pub const CMF_NODEFAULT: DWORD = 0x00000020;

pub const CMF_OPTIMIZEFORINVOKE: DWORD = 0x00000800;

pub const CMF_VERBSONLY: DWORD = 0x00000002;

pub const CMIC_MASK_CONTROL_DOWN: DWORD = 0x40000000;

pub const CMIC_MASK_PTINVOKE: DWORD = 0x20000000;

pub const CMIC_MASK_SHIFT_DOWN: DWORD = 0x10000000;

pub const DFM_GETDEFSTATICID: DWORD = 14;

pub const DFM_INVOKECOMMAND: DWORD = 2;

pub const DFM_INVOKECOMMANDEX: DWORD = 12;

pub const DFM_MERGECONTEXTMENU: DWORD = 1;

pub const ECF_SEPARATORAFTER: DWORD = 0x40;

pub const ECF_SEPARATORBEFORE: DWORD = 0x20;

pub const ERROR_NOT_FOUND: DWORD = 1168;

pub const ERROR_NO_MORE_ITEMS: DWORD = 259;

pub const FCIDM_SHVIEW_EXPLORE: DWORD = 0x7101;

pub const FCIDM_SHVIEW_OPEN: DWORD = 0x7102;

pub const FCW_TREE: DWORD = 0x0003;

pub const GCS_HELPTEXTA: DWORD = 0x00000001;

pub const GCS_HELPTEXTW: DWORD = 0x00000005;

pub const GCS_VALIDATEA: DWORD = 0x00000002;

pub const GCS_VALIDATEW: DWORD = 0x00000006;

pub const GCS_VERBA: DWORD = 0x00000000;

pub const GCS_VERBW: DWORD = 0x00000004;

pub const GHND: DWORD = 66;

// IDM_CUT/COPY/CREATELINK/DELETE/RENAME/PROPERTIES come from the island: shresdef.h:905-914
// (FCIDM_SHVIEW_* - 0x7000), the ids the MENU_SHV_FILE resource uses -- not mshtmcid.h's
// unrelated MSHTML command ids, which these hand constants had copied.

pub const KEY_READ: DWORD = 0x20019;

pub const MFS_DEFAULT: DWORD = 4096;

pub const MFS_ENABLED: DWORD = 0;

pub const MFT_SEPARATOR: DWORD = 0x800;

pub const MF_BYCOMMAND: DWORD = 0;

pub const MF_BYPOSITION: DWORD = 1024;

pub const MF_SEPARATOR: DWORD = 0x800;

pub const MIIM_DATA: DWORD = 32;

pub const MIIM_STATE: DWORD = 1;

pub const MK_CONTROL: DWORD = 8;

pub const MK_SHIFT: DWORD = 4;

pub const SBSP_RELATIVE: DWORD = 0x00001000;

pub const SEE_MASK_CLASSKEY: DWORD = 0x00000003;

pub const SEE_MASK_INVOKEIDLIST: DWORD = 0x0000000C;

pub const SFGAO_BROWSABLE: DWORD = 0x08000000;

pub const SHCNE_MKDIR: DWORD = 0x00000008;

pub const SHCNF_FLUSH: DWORD = 0x1000;

pub const SHCNF_PATHW: DWORD = 0x0005;

pub const STATUS_SUCCESS: DWORD = 0;

pub const STGM_READ: DWORD = 0x00000000;

pub const SVSI_DESELECTOTHERS: DWORD = 0x00000004;

pub const SVSI_EDIT: DWORD = 0x00000003;

pub const SVSI_ENSUREVISIBLE: DWORD = 0x00000008;

pub const SVSI_FOCUSED: DWORD = 0x00000010;

pub const SVSI_SELECT: DWORD = 0x00000001;

pub const TYMED_HGLOBAL: DWORD = 1;

pub const WM_DRAWITEM: DWORD = 43;

pub const WM_INITMENUPOPUP: DWORD = 279;

pub const WM_MEASUREITEM: DWORD = 44;

pub const _WIN32_WINNT_VISTA: DWORD = 0x0600;

pub const _WIN32_WINNT_WIN7: DWORD = 0x0601;

#[repr(C)]
pub struct CMINVOKECOMMANDINFOEX {
    pub cbSize: DWORD,
    pub fMask: DWORD,
    pub hwnd: HWND,
    pub lpVerb: LPCSTR,
    pub lpParameters: LPCSTR,
    pub lpDirectory: LPCSTR,
    pub nShow: INT,
    pub dwHotKey: DWORD,
    pub hIcon: HANDLE,
    pub lpTitle: LPCSTR,
    pub lpVerbW: LPCWSTR,
    pub lpParametersW: LPCWSTR,
    pub lpDirectoryW: LPCWSTR,
    pub lpTitleW: LPCWSTR,
    pub ptInvoke: POINT,
}

#[repr(C)]
pub struct DEFCONTEXTMENU {
    pub hwnd: HWND,
    pub pcmcb: *mut IContextMenuCB,
    pub pidlFolder: PCIDLIST_ABSOLUTE,
    pub psf: *mut IShellFolder,
    pub cidl: UINT,
    pub apidl: PCUITEMID_CHILD_ARRAY,
    pub punkAssociationInfo: *mut IUnknown,
    pub cKeys: UINT,
    pub aKeys: *const HKEY,
}

#[repr(C)]
pub struct DVTARGETDEVICE {
    pub tdSize: DWORD,
    pub tdDriverNameOffset: WORD,
    pub tdDeviceNameOffset: WORD,
    pub tdPortNameOffset: WORD,
    pub tdExtDevmodeOffset: WORD,
    pub tdData: [BYTE; 1],
}

pub use crate::island::MEASUREITEMSTRUCT;

pub use crate::island::QCMINFO;

pub use crate::island::QCMINFO_IDMAP;

pub use crate::island::QCMINFO_IDMAP_PLACEMENT;

// Deterministic from shobjidl.h
#[repr(C)]
pub struct ICommDlgBrowser {
    pub lpVtbl: *const ICommDlgBrowserVtbl,
}
#[repr(C)]
pub struct ICommDlgBrowserVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut ICommDlgBrowser, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut ICommDlgBrowser) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut ICommDlgBrowser) -> ULONG,
    pub OnDefaultCommand: unsafe extern "system" fn(This: *mut ICommDlgBrowser, shv: *mut IShellView) -> HRESULT,
    pub OnStateChange: unsafe extern "system" fn(This: *mut ICommDlgBrowser, shv: *mut IShellView, uChange: ULONG) -> HRESULT,
    pub IncludeObject: unsafe extern "system" fn(This: *mut ICommDlgBrowser, shv: *mut IShellView, pidl: PCUITEMID_CHILD) -> HRESULT,
}
extern "C" {
    pub static IID_ICommDlgBrowser: IID;
}

// Deterministic from shobjidl.h
#[repr(C)]
pub struct IContextMenuCB {
    pub lpVtbl: *const IContextMenuCBVtbl,
}
#[repr(C)]
pub struct IContextMenuCBVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IContextMenuCB, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IContextMenuCB) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IContextMenuCB) -> ULONG,
    pub CallBack: unsafe extern "system" fn(This: *mut IContextMenuCB, psf: *mut IShellFolder, hwndOwner: HWND, pdtobj: *mut IDataObject, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> HRESULT,
}
extern "C" {
    pub static IID_IContextMenuCB: IID;
}

// Deterministic from shobjidl.h
#[repr(C)]
pub struct IExecuteCommand {
    pub lpVtbl: *const IExecuteCommandVtbl,
}
#[repr(C)]
pub struct IExecuteCommandVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IExecuteCommand, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IExecuteCommand) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IExecuteCommand) -> ULONG,
    pub SetKeyState: unsafe extern "system" fn(This: *mut IExecuteCommand, grfKeyState: DWORD) -> HRESULT,
    pub SetParameters: unsafe extern "system" fn(This: *mut IExecuteCommand, pszParameters: LPCWSTR) -> HRESULT,
    pub SetPosition: unsafe extern "system" fn(This: *mut IExecuteCommand, pt: POINT) -> HRESULT,
    pub SetShowWindow: unsafe extern "system" fn(This: *mut IExecuteCommand, nShow: i32) -> HRESULT,
    pub SetNoShowUI: unsafe extern "system" fn(This: *mut IExecuteCommand, fNoShowUI: BOOL) -> HRESULT,
    pub SetDirectory: unsafe extern "system" fn(This: *mut IExecuteCommand, pszDirectory: LPCWSTR) -> HRESULT,
    pub Execute: unsafe extern "system" fn(This: *mut IExecuteCommand) -> HRESULT,
}
extern "C" {
    pub static IID_IExecuteCommand: IID;
}

// Deterministic from oaidl.h
#[repr(C)]
pub struct IPropertyBag {
    pub lpVtbl: *const IPropertyBagVtbl,
}
#[repr(C)]
pub struct IPropertyBagVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IPropertyBag, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IPropertyBag) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IPropertyBag) -> ULONG,
    pub Read: unsafe extern "system" fn(This: *mut IPropertyBag, pszPropName: LPCOLESTR, pVar: *mut VARIANT, pErrorLog: *mut IErrorLog) -> HRESULT,
    pub Write: unsafe extern "system" fn(This: *mut IPropertyBag, pszPropName: LPCOLESTR, pVar: *mut VARIANT) -> HRESULT,
}
extern "C" {
    pub static IID_IPropertyBag: IID;
}

// Deterministic from shobjidl.h
#[repr(C)]
pub struct IShellBrowser {
    pub lpVtbl: *const IShellBrowserVtbl,
}
#[repr(C)]
pub struct IShellBrowserVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IShellBrowser, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IShellBrowser) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IShellBrowser) -> ULONG,
    pub GetWindow: unsafe extern "system" fn(This: *mut IShellBrowser, phwnd: *mut HWND) -> HRESULT,
    pub ContextSensitiveHelp: unsafe extern "system" fn(This: *mut IShellBrowser, fEnterMode: BOOL) -> HRESULT,
    pub InsertMenusSB: unsafe extern "system" fn(This: *mut IShellBrowser, hmenuShared: HMENU, lpMenuWidths: LPOLEMENUGROUPWIDTHS) -> HRESULT,
    pub SetMenuSB: unsafe extern "system" fn(This: *mut IShellBrowser, hmenuShared: HMENU, holemenuReserved: HOLEMENU, hwndActiveObject: HWND) -> HRESULT,
    pub RemoveMenusSB: unsafe extern "system" fn(This: *mut IShellBrowser, hmenuShared: HMENU) -> HRESULT,
    pub SetStatusTextSB: unsafe extern "system" fn(This: *mut IShellBrowser, pszStatusText: LPCOLESTR) -> HRESULT,
    pub EnableModelessSB: unsafe extern "system" fn(This: *mut IShellBrowser, fEnable: BOOL) -> HRESULT,
    pub TranslateAcceleratorSB: unsafe extern "system" fn(This: *mut IShellBrowser, pmsg: *mut MSG, wID: WORD) -> HRESULT,
    pub BrowseObject: unsafe extern "system" fn(This: *mut IShellBrowser, pidl: PCUIDLIST_RELATIVE, wFlags: UINT) -> HRESULT,
    pub GetViewStateStream: unsafe extern "system" fn(This: *mut IShellBrowser, grfMode: DWORD, ppStrm: *mut *mut IStream) -> HRESULT,
    pub GetControlWindow: unsafe extern "system" fn(This: *mut IShellBrowser, id: UINT, phwnd: *mut HWND) -> HRESULT,
    pub SendControlMsg: unsafe extern "system" fn(This: *mut IShellBrowser, id: UINT, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, pret: *mut LRESULT) -> HRESULT,
    pub QueryActiveShellView: unsafe extern "system" fn(This: *mut IShellBrowser, ppshv: *mut *mut IShellView) -> HRESULT,
    pub OnViewWindowActive: unsafe extern "system" fn(This: *mut IShellBrowser, pshv: *mut IShellView) -> HRESULT,
    pub SetToolbarItems: unsafe extern "system" fn(This: *mut IShellBrowser, lpButtons: LPTBBUTTONSB, nButtons: UINT, uFlags: UINT) -> HRESULT,
}
extern "C" {
    pub static IID_IShellBrowser: IID;
}

// Deterministic from shobjidl.h
#[repr(C)]
pub struct IShellExtInit {
    pub lpVtbl: *const IShellExtInitVtbl,
}
#[repr(C)]
pub struct IShellExtInitVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IShellExtInit, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IShellExtInit) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IShellExtInit) -> ULONG,
    pub Initialize: unsafe extern "system" fn(This: *mut IShellExtInit, pidlFolder: PCIDLIST_ABSOLUTE, pdtobj: *mut IDataObject, hkeyProgID: HKEY) -> HRESULT,
}
extern "C" {
    pub static IID_IShellExtInit: IID;
}

// Deterministic from shobjidl.h
#[repr(C)]
pub struct IShellView {
    pub lpVtbl: *const IShellViewVtbl,
}
#[repr(C)]
pub struct IShellViewVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IShellView, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IShellView) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IShellView) -> ULONG,
    pub GetWindow: unsafe extern "system" fn(This: *mut IShellView, phwnd: *mut HWND) -> HRESULT,
    pub ContextSensitiveHelp: unsafe extern "system" fn(This: *mut IShellView, fEnterMode: BOOL) -> HRESULT,
    pub TranslateAccelerator: unsafe extern "system" fn(This: *mut IShellView, pmsg: *mut MSG) -> HRESULT,
    pub EnableModeless: unsafe extern "system" fn(This: *mut IShellView, fEnable: BOOL) -> HRESULT,
    pub UIActivate: unsafe extern "system" fn(This: *mut IShellView, uState: UINT) -> HRESULT,
    pub Refresh: unsafe extern "system" fn(This: *mut IShellView) -> HRESULT,
    pub CreateViewWindow: unsafe extern "system" fn(This: *mut IShellView, psvPrevious: *mut IShellView, lpfs: LPCFOLDERSETTINGS, psb: *mut IShellBrowser, prcView: *mut RECT, phWnd: *mut HWND) -> HRESULT,
    pub DestroyViewWindow: unsafe extern "system" fn(This: *mut IShellView) -> HRESULT,
    pub GetCurrentInfo: unsafe extern "system" fn(This: *mut IShellView, lpfs: LPFOLDERSETTINGS) -> HRESULT,
    pub AddPropertySheetPages: unsafe extern "system" fn(This: *mut IShellView, dwReserved: DWORD, pfn: LPFNSVADDPROPSHEETPAGE, lparam: LPARAM) -> HRESULT,
    pub SaveViewState: unsafe extern "system" fn(This: *mut IShellView) -> HRESULT,
    pub SelectItem: unsafe extern "system" fn(This: *mut IShellView, pidlItem: PCUITEMID_CHILD, uFlags: SVSIF) -> HRESULT,
    pub GetItemObject: unsafe extern "system" fn(This: *mut IShellView, uItem: UINT, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT,
}
extern "C" {
    pub static IID_IShellView: IID;
}

pub type SVSIF = UINT;

pub type LPTBBUTTONSB = *mut core::ffi::c_void;

macro_rules! LOWORD { ($value:expr) => { (($value as usize) & 0xffff) as UINT }; }

pub type LPFNSVADDPROPSHEETPAGE = Option<unsafe extern "system" fn(HPROPSHEETPAGE, LPARAM) -> BOOL>;

#[repr(C)]
pub struct SHELLEXECUTEINFOW {
    pub cbSize: DWORD, pub fMask: ULONG, pub hwnd: HWND,
    pub lpVerb: LPCWSTR, pub lpFile: LPCWSTR, pub lpParameters: LPCWSTR,
    pub lpDirectory: LPCWSTR, pub nShow: INT, pub hInstApp: HINSTANCE,
    pub lpIDList: LPVOID, pub lpClass: LPCWSTR, pub hkeyClass: HKEY,
    pub dwHotKey: DWORD, pub icon_or_monitor: HANDLE, pub hProcess: HANDLE,
}

pub const DROPEFFECT_COPY: DWORD = 1;
pub const DROPEFFECT_MOVE: DWORD = 2;

extern "system" {
    fn SHChangeNotify(event: LONG, flags: UINT, item1: LPCVOID, item2: LPCVOID);
    fn GlobalLock(memory: HGLOBAL) -> LPVOID;
    fn GlobalUnlock(memory: HGLOBAL) -> BOOL;
    fn GetLastError() -> DWORD;
    fn lstrlenW(text: LPCWSTR) -> INT;
    fn StrChrW(text: LPCWSTR, ch: WCHAR) -> LPWSTR;
    fn CoTaskMemFree(memory: LPVOID);
}

extern "C" {
    #[link_name = "memcpy"] fn cpp_com_memcpy(dst: *mut core::ffi::c_void, src: *const core::ffi::c_void, bytes: usize) -> *mut core::ffi::c_void;
}

unsafe fn cpp_com_string_cb_printf_w(
    dest: LPWSTR, bytes: usize, format: LPCWSTR, args: &[LPCWSTR],
) -> HRESULT {
    // strsafe.h StringCbPrintfW, for the conversions translated call sites use: `%s` and `%%`.
    // Anything else is REFUSED (STRSAFE_E_INVALID_PARAMETER, empty result). This used to copy
    // an unknown conversion through literally and leave its argument unconsumed — and it did
    // not know `%%` either, so COpenWithList's `"%s" "%%1"` registered `"app.exe" "%%1"` as an
    // Open-With command line instead of `"app.exe" "%1"`, with nothing red anywhere.
    let capacity = bytes / core::mem::size_of::<WCHAR>();
    if dest.is_null() || format.is_null() || capacity == 0 { return E_FAIL; }
    let (mut out, mut input, mut arg_index) = (0usize, 0usize, 0usize);
    while *format.add(input) != 0 {
        let unit = *format.add(input);
        if unit == b'%' as WCHAR {
            let spec = *format.add(input + 1);
            if spec == b'%' as WCHAR {
                if out + 1 >= capacity { *dest.add(capacity - 1) = 0; return 0x8007007Au32 as HRESULT; }
                *dest.add(out) = b'%' as WCHAR; out += 1; input += 2;
                continue;
            }
            if spec != b's' as WCHAR { *dest = 0; return 0x80070057u32 as HRESULT; }
            if arg_index >= args.len() { return E_FAIL; }
            let arg = args[arg_index]; arg_index += 1; input += 2;
            if arg.is_null() { continue; }
            let mut index = 0usize;
            while *arg.add(index) != 0 {
                if out + 1 >= capacity { *dest.add(capacity - 1) = 0; return 0x8007007Au32 as HRESULT; }
                *dest.add(out) = *arg.add(index); out += 1; index += 1;
            }
            continue;
        }
        if out + 1 >= capacity { *dest.add(capacity - 1) = 0; return 0x8007007Au32 as HRESULT; }
        *dest.add(out) = unit; out += 1; input += 1;
    }
    *dest.add(out) = 0; S_OK
}
macro_rules! StringCbPrintfW {
    ($dest:expr, $bytes:expr, $format:expr, $arg1:expr $(,)?) => {
        cpp_com_string_cb_printf_w($dest, $bytes, $format, &[$arg1])
    };
    ($dest:expr, $bytes:expr, $format:expr, $arg1:expr, $arg2:expr $(,)?) => {
        cpp_com_string_cb_printf_w($dest, $bytes, $format, &[$arg1, $arg2])
    };
}

#[inline]
unsafe fn memcpy<D, S, N: CppComSize>(dst: *mut D, src: *const S, bytes: N) -> *mut core::ffi::c_void {
    cpp_com_memcpy(dst.cast(), src.cast(), bytes.cpp_com_size())
}

extern "system" {
    #[link_name = "SHFree"] fn cpp_com_SHFree(memory: LPVOID);
}
#[inline]
unsafe fn SHFree<T>(memory: *mut T) { cpp_com_SHFree(memory.cast()); }


extern "C" {
    static CLSID_CopyToMenu: GUID;
    static CLSID_MoveToMenu: GUID;
    static IID_IDataObject: GUID;
    static IID_IDropTarget: GUID;
    static IID_IFolderView: GUID;
    static IID_IPersistFolder2: GUID;
    static IID_IShellFolderView: GUID;
}

#[inline]
unsafe fn CopyMemory<D, S, N: CppComSize>(
    dst: *mut D, src: *const S, bytes: N) {
    let _ = memcpy(dst, src, bytes);
}
#[inline]
unsafe fn ZeroMemory<T, N: CppComSize>(dst: *mut T, bytes: N) {
    let _ = memset(dst.cast(), 0, bytes.cpp_com_size());
}

#[inline]
unsafe fn StringCchCopyW(dst: LPWSTR, capacity: usize, src: LPCWSTR) -> HRESULT {
    if dst.is_null() || src.is_null() || capacity == 0 { return E_INVALIDARG; }
    let len = lstrlenW(src) as usize;
    if len >= capacity { *dst = 0; return E_FAIL; }
    let bytes = (len + 1) * core::mem::size_of::<WCHAR>();
    let _ = memcpy(dst, src, bytes);
    S_OK
}

extern "system" {
}

static CPP_COM_EMPTY_WSTR: [WCHAR; 1] = [0];

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

extern "C" {
    static mut shell32_hInstance: HINSTANCE;
}

// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_0C71B3CB5580: [WCHAR; 8] = [0x0063, 0x006F, 0x006D, 0x006D, 0x0061, 0x006E, 0x0064, 0x0000];
pub static CPP_COM_WSTR_0EB6EACBADB7: [WCHAR; 16] = [0x0044, 0x0065, 0x006C, 0x0065, 0x0067, 0x0061, 0x0074, 0x0065, 0x0045, 0x0078, 0x0065, 0x0063, 0x0075, 0x0074, 0x0065, 0x0000];
pub static CPP_COM_WSTR_1338A3B61B21: [WCHAR; 5] = [0x0066, 0x0069, 0x006E, 0x0064, 0x0000];
pub static CPP_COM_WSTR_31AF3113D116: [WCHAR; 8] = [0x0065, 0x0078, 0x0070, 0x006C, 0x006F, 0x0072, 0x0065, 0x0000];
pub static CPP_COM_WSTR_325A56B0AAB7: [WCHAR; 13] = [0x0042, 0x0072, 0x006F, 0x0077, 0x0073, 0x0065, 0x0072, 0x0046, 0x006C, 0x0061, 0x0067, 0x0073, 0x0000];
pub static CPP_COM_WSTR_353157B4DE84: [WCHAR; 9] = [0x0045, 0x0078, 0x0074, 0x0065, 0x006E, 0x0064, 0x0065, 0x0064, 0x0000];
pub static CPP_COM_WSTR_6A302070A1F6: [WCHAR; 8] = [0x0070, 0x0072, 0x0069, 0x006E, 0x0074, 0x0074, 0x006F, 0x0000];
pub static CPP_COM_WSTR_736C68B3EC8E: [WCHAR; 15] = [0x0053, 0x0065, 0x0070, 0x0061, 0x0072, 0x0061, 0x0074, 0x006F, 0x0072, 0x0041, 0x0066, 0x0074, 0x0065, 0x0072, 0x0000];
pub static CPP_COM_WSTR_82DB442E7A60: [WCHAR; 5] = [0x006F, 0x0070, 0x0065, 0x006E, 0x0000];
pub static CPP_COM_WSTR_87660BB31B23: [WCHAR; 28] = [0x0073, 0x0068, 0x0065, 0x006C, 0x006C, 0x0065, 0x0078, 0x005C, 0x0043, 0x006F, 0x006E, 0x0074, 0x0065, 0x0078, 0x0074, 0x004D, 0x0065, 0x006E, 0x0075, 0x0048, 0x0061, 0x006E, 0x0064, 0x006C, 0x0065, 0x0072, 0x0073, 0x0000];
pub static CPP_COM_WSTR_878FA9FF9662: [WCHAR; 6] = [0x0072, 0x0075, 0x006E, 0x0061, 0x0073, 0x0000];
pub static CPP_COM_WSTR_9A7909160783: [WCHAR; 14] = [0x0045, 0x0078, 0x0070, 0x006C, 0x006F, 0x0072, 0x0065, 0x0072, 0x0046, 0x006C, 0x0061, 0x0067, 0x0073, 0x0000];
pub static CPP_COM_WSTR_A08BD2ED73B2: [WCHAR; 14] = [0x004C, 0x0065, 0x0067, 0x0061, 0x0063, 0x0079, 0x0044, 0x0069, 0x0073, 0x0061, 0x0062, 0x006C, 0x0065, 0x0000];
pub static CPP_COM_WSTR_A131D6DE6E8D: [WCHAR; 9] = [0x0073, 0x0068, 0x0065, 0x006C, 0x006C, 0x005C, 0x0025, 0x0073, 0x0000];
pub static CPP_COM_WSTR_A66B4CE0DBF3: [WCHAR; 7] = [0x006F, 0x0070, 0x0065, 0x006E, 0x0061, 0x0073, 0x0000];
pub static CPP_COM_WSTR_AFF3436D7481: [WCHAR; 6] = [0x0043, 0x004C, 0x0053, 0x0049, 0x0044, 0x0000];
pub static CPP_COM_WSTR_B044B339A54F: [WCHAR; 11] = [0x0044, 0x0072, 0x006F, 0x0070, 0x0054, 0x0061, 0x0072, 0x0067, 0x0065, 0x0074, 0x0000];
pub static CPP_COM_WSTR_D1B3A29D6E8A: [WCHAR; 13] = [0x004E, 0x0065, 0x0076, 0x0065, 0x0072, 0x0044, 0x0065, 0x0066, 0x0061, 0x0075, 0x006C, 0x0074, 0x0000];
pub static CPP_COM_WSTR_D928DD0458CA: [WCHAR; 68] = [0x0053, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x004D, 0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057, 0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x005C, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069, 0x006F, 0x006E, 0x005C, 0x0053, 0x0068, 0x0065, 0x006C, 0x006C, 0x0020, 0x0045, 0x0078, 0x0074, 0x0065, 0x006E, 0x0073, 0x0069, 0x006F, 0x006E, 0x0073, 0x005C, 0x0041, 0x0070, 0x0070, 0x0072, 0x006F, 0x0076, 0x0065, 0x0064, 0x0000];
pub static CPP_COM_WSTR_DA67076D6619: [WCHAR; 8] = [0x004D, 0x0055, 0x0049, 0x0056, 0x0065, 0x0072, 0x0062, 0x0000];
pub static CPP_COM_WSTR_DACCBBBCFACF: [WCHAR; 6] = [0x0070, 0x0072, 0x0069, 0x006E, 0x0074, 0x0000];
pub static CPP_COM_WSTR_DCA3A3F4A49E: [WCHAR; 23] = [0x0050, 0x0072, 0x006F, 0x0067, 0x0072, 0x0061, 0x006D, 0x006D, 0x0061, 0x0074, 0x0069, 0x0063, 0x0041, 0x0063, 0x0063, 0x0065, 0x0073, 0x0073, 0x004F, 0x006E, 0x006C, 0x0079, 0x0000];
pub static CPP_COM_WSTR_E58D4A4F3B68: [WCHAR; 18] = [0x0053, 0x0075, 0x0070, 0x0070, 0x0072, 0x0065, 0x0073, 0x0073, 0x0069, 0x006F, 0x006E, 0x0050, 0x006F, 0x006C, 0x0069, 0x0063, 0x0079, 0x0000];
pub static CPP_COM_WSTR_EE2AF488DA28: [WCHAR; 16] = [0x0053, 0x0065, 0x0070, 0x0061, 0x0072, 0x0061, 0x0074, 0x006F, 0x0072, 0x0042, 0x0065, 0x0066, 0x006F, 0x0072, 0x0065, 0x0000];
pub static CPP_COM_WSTR_EE89605B31B0: [WCHAR; 6] = [0x0073, 0x0068, 0x0065, 0x006C, 0x006C, 0x0000];
pub static CPP_COM_WSTR_FDF5BD4663B6: [WCHAR; 5] = [0x0065, 0x0064, 0x0069, 0x0074, 0x0000];

use core::ffi::c_void;

pub unsafe fn SHELL_GetRegCLSID_body(hKey: HKEY, SubKey: LPCWSTR, Value: LPCWSTR, clsid: &mut CLSID) -> HRESULT {
    let mut buf: [u16; 42] = [0; 42];
    let mut cb: u32 = core::mem::size_of_val(&buf) as u32;
    let err: u32 = RegGetValueW(hKey, SubKey, Value, RRF_RT_REG_SZ, core::ptr::null_mut(), buf.as_mut_ptr().cast(), &mut cb) as u32;
    if err == 0 {
        CLSIDFromString(buf.as_ptr(), clsid)
    } else {
        HRESULT_FROM_WIN32(err as LONG)
    }
}

// NB the parameter is `Pos_`, not the C's `Pos`: `ntdll-sys` exports
// `pub const Pos: _BUS_DATA_TYPE` (a bindgen enum alias) and the prelude glob brings it
// into scope here, so an identifier `Pos` in a parameter list resolves as a CONSTANT
// PATTERN rather than a binding. rustc then reports the mismatch against the const's
// i32 type, on the parameter list itself, with no mention of the collision.
pub unsafe fn InsertMenuItemAt_body(hMenu: HMENU, Pos_: UINT, Flags: UINT) -> BOOL {
    let mut mii: MENUITEMINFOW = core::mem::zeroed();
    mii.cbSize = FIELD_OFFSET!(MENUITEMINFOW, hbmpItem) as UINT;
    mii.fMask = MIIM_TYPE;
    mii.fType = Flags;
    InsertMenuItemW(hMenu, Pos_, TRUE, &mut mii) as BOOL
}

pub unsafe fn MapVerbToCmdId_body(Verb: PVOID, IsUnicode: BOOL, pCM: *mut IContextMenu, idFirst: UINT, idLast: UINT) -> HRESULT {
    let gcs = if IsUnicode != 0 {
        GCS_VERBW
    } else {
        GCS_VERBA
    };
    let mut id = idFirst;
    while id <= idLast {
        let mut buf: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
        buf[0] = UNICODE_NULL;
        let hr = ((*(*pCM).lpVtbl).GetCommandString)(
            pCM,
            id as UINT_PTR,
            gcs,
            core::ptr::null_mut(),
            buf.as_mut_ptr() as LPSTR,
            buf.len() as u32,
        );
        if hr < 0 || buf[0] == 0 {
            id = id.wrapping_add(1);
            continue;
        } else if IsUnicode != 0 && _wcsicmp(Verb as LPWSTR, buf.as_mut_ptr()) == 0 {
            return id as HRESULT;
        } else if IsUnicode == 0 && lstrcmpiA(Verb as LPSTR, buf.as_mut_ptr() as LPSTR) == 0 {
            return id as HRESULT;
        }
        id = id.wrapping_add(1);
    }
    HRESULT_FROM_WIN32(ERROR_NOT_FOUND as LONG)
}

pub unsafe fn IsVerbListSeparator_body(Ch: WCHAR) -> bool {
    Ch == 0x0020u16 || Ch == 0x002Cu16
}

pub unsafe fn FindVerbInDefaultVerbList_body(List: LPCWSTR, Verb: LPCWSTR) -> i32 {
    let mut List = List;
    let mut index: u32 = 0;
    while *List != 0 {
        while IsVerbListSeparator_body(*List) {
            List = List.offset(1);
        }
        let Start = List;
        while *List != 0 && !IsVerbListSeparator_body(*List) {
            List = List.offset(1);
        }
        if _wcsnicmp(Verb, Start, (List.offset_from(Start)) as usize) == 0 && List > Start {
            return index as i32;
        }
        index += 1;
    }
    -1
}

pub unsafe fn GetFriendlyVerb_body(pszVerb: PCWSTR, pszBuf: PWSTR, cchMax: SIZE_T) -> HRESULT {
    #[repr(C)]
    struct VerbMap {
        pszVerb: PCWSTR,
        iResId: u16,
    }
    // A `static` must be Sync, and the raw `PCWSTR` is not. These point at this file's own
    // `static [WCHAR; N]` literals, which are immutable for the life of the process — the
    // same reason the landed tables in debughlp/CDesktopFolder/CRecycleBin carry this impl.
    unsafe impl Sync for VerbMap {}

    static MAP: [VerbMap; 5] = [
        VerbMap { pszVerb: CPP_COM_WSTR_FDF5BD4663B6.as_ptr(), iResId: IDS_EDIT_VERB as u16 },
        VerbMap { pszVerb: CPP_COM_WSTR_DACCBBBCFACF.as_ptr(), iResId: IDS_PRINT_VERB as u16 },
        VerbMap { pszVerb: CPP_COM_WSTR_878FA9FF9662.as_ptr(), iResId: IDS_RUNAS_VERB as u16 },
        VerbMap { pszVerb: CPP_COM_WSTR_A66B4CE0DBF3.as_ptr(), iResId: IDS_OPEN_VERB as u16 },
        VerbMap { pszVerb: CPP_COM_WSTR_1338A3B61B21.as_ptr(), iResId: IDS_FIND_VERB as u16 },
    ];

    let mut i: usize = 0;
    while i < MAP.len() {
        if _wcsicmp(pszVerb, MAP[i].pszVerb) == 0 &&
            LoadStringW(shell32_hInstance, MAP[i].iResId.into(), pszBuf, cchMax as i32) != 0
        {
            return S_OK;
        }
        i += 1;
    }

    if *pszVerb.offset(0) < 127 &&
        StrChrW(pszVerb, '&' as u16).is_null() &&
        {
            let mut temp_cchMax = cchMax - 1;
            StringCchCopyW(pszBuf.offset(1), temp_cchMax as usize, pszVerb) >= 0
        }
    {
        *pszBuf = 0x0026u16;
        return S_OK;
    }
    E_FAIL
}

pub unsafe fn CDefaultContextMenu_QueryService_body(this: *mut CDefaultContextMenu, svc: REFIID, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT {
    return IUnknown_QueryService(((*this).m_site).cast(), (svc).cast(), (riid).cast(), ppv);
}

pub unsafe fn CDefaultContextMenu_ctor_body(this: *mut CDefaultContextMenu) -> () {
    (*this).m_psf = core::ptr::null_mut();
    (*this).m_pmcb = core::ptr::null_mut();
    (*this).m_pfnmcb = None;
    (*this).m_cidl = 0;
    (*this).m_apidl = core::ptr::null_mut();
    (*this).m_pDataObj = core::ptr::null_mut();
    (*this).m_cKeys = 0;
    (*this).m_pidlFolder = core::ptr::null_mut();
    (*this).m_bGroupPolicyActive = 0;
    (*this).m_iIdQCMFirst = 0;
    (*this).m_iIdSHEFirst = 0;
    (*this).m_iIdSHELast = 0;
    (*this).m_iIdSCMFirst = 0;
    (*this).m_iIdSCMLast = 0;
    (*this).m_iIdCBFirst = 0;
    (*this).m_iIdCBLast = 0;
    (*this).m_iIdDfltFirst = 0;
    (*this).m_iIdDfltLast = 0;
    (*this).m_hwnd = core::ptr::null_mut();
    (*this).m_DefVerbs[0] = UNICODE_NULL;  // C: `*m_DefVerbs = UNICODE_NULL;`
}

pub unsafe fn CDefaultContextMenu_dtor_body(this: *mut CDefaultContextMenu) -> () {
    let mut it = (*this).m_DynamicEntries.GetHeadPosition();
    while it != core::ptr::null_mut() {
        let info = (*this).m_DynamicEntries.GetNext(&mut it);
        IUnknown_SetSite((*info).pCM.p.cast(), core::ptr::null_mut());
    }
    (*this).m_DynamicEntries.RemoveAll();
    (*this).m_StaticEntries.RemoveAll();

    let mut i = 0;
    while i < (*this).m_cKeys {
        RegCloseKey((*this).m_aKeys[i as usize]);
        i += 1;
    }

    if (*this).m_pidlFolder != core::ptr::null_mut() {
        CoTaskMemFree((*this).m_pidlFolder.cast());
    }
    _ILFreeaPidl((*this).m_apidl as *mut PITEMID_CHILD, (*this).m_cidl);
}

pub unsafe fn CDefaultContextMenu_Initialize_body(this: *mut CDefaultContextMenu, pdcm: *const DEFCONTEXTMENU, lpfn: LPFNDFMCALLBACK) -> HRESULT {
    let mut hr: HRESULT = S_OK;
    if (*pdcm).pcmcb.is_null() && lpfn.is_none() {
        return E_INVALIDARG;
    }

    (*this).m_cidl = (*pdcm).cidl;
    (*this).m_apidl = _ILCopyaPidl((*pdcm).apidl, (*this).m_cidl) as *mut _;
    if (*this).m_cidl != 0 && (*this).m_apidl.is_null() {
        return E_OUTOFMEMORY;
    }
    CDefaultContextMenu_cpp_com_assign_m_psf(this, (*pdcm).psf);
    CDefaultContextMenu_cpp_com_assign_m_pmcb(this, (*pdcm).pcmcb);
    (*this).m_pfnmcb = lpfn;
    (*this).m_hwnd = (*pdcm).hwnd;

    let mut i: u32 = 0;
    while i < (*pdcm).cKeys {
        if i >= (*this).m_aKeys.len() as u32 {
            hr = E_INVALIDARG;
        } else {
            (*this).m_aKeys[i as usize] = SHRegDuplicateHKey(*(*pdcm).aKeys.wrapping_add(i as usize));
            if !(*this).m_aKeys[i as usize].is_null() {
                (*this).m_cKeys += 1;
            } else {
                hr = E_OUTOFMEMORY;
            }
        }
        i += 1;
    }

    ((*(*(*this).m_psf).lpVtbl).GetUIObjectOf)(
        (*this).m_psf,
        (*pdcm).hwnd.cast(),
        (*this).m_cidl,
        (*this).m_apidl.cast(),
        core::ptr::addr_of!(IID_IDataObject).cast(), core::ptr::null_mut(), (core::ptr::addr_of_mut!((*this).m_pDataObj)).cast(),
    );

    if !(*pdcm).pidlFolder.is_null() {
        (*this).m_pidlFolder = ILClone((*pdcm).pidlFolder);
    } else {
        // C: `CComPtr<IPersistFolder2> pf = NULL;`. The copy-assign-from-NULL lowering
        // emitted an AddRef guard on the pointer the line above just nulled — dead, and it
        // would have needed a vtable shape for an interface nothing here calls through.
        let mut pf: *mut IPersistFolder2 = core::ptr::null_mut();
        let _pf_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pf));
        if ((*(*(*this).m_psf).lpVtbl).QueryInterface)(
            (*this).m_psf,
            core::ptr::addr_of!(IID_IPersistFolder2).cast(), (core::ptr::addr_of_mut!(pf)).cast(),
        ) >= 0
        {
            if ((*(*pf).lpVtbl).GetCurFolder)(pf, core::ptr::addr_of_mut!((*this).m_pidlFolder).cast())
                < 0
            {
                ERR!("GetCurFolder failed\n");
            }
        }
        if !pf.is_null() {
            ((*(*pf).lpVtbl).Release)(pf);
            pf = core::ptr::null_mut();
        }
    }

    hr
}

pub unsafe fn CDefaultContextMenu__DoCallback_body(this: *mut CDefaultContextMenu, uMsg: UINT, wParam: WPARAM, lParam: LPVOID) -> HRESULT {
    if !(*this).m_pmcb.is_null() {
        return ((*(*(*this).m_pmcb).lpVtbl).CallBack)((*this).m_pmcb, (*this).m_psf, (*this).m_hwnd, (*this).m_pDataObj.cast(), uMsg, wParam, lParam as LPARAM);
    } else if let Some(pfnmcb) = (*this).m_pfnmcb {
        // `if let` rather than `is_some()` + `unwrap()`: same control flow, but the C's
        // `else if (m_pfnmcb)` has no panic in it and neither should this.
        return pfnmcb((*this).m_psf, (*this).m_hwnd, (*this).m_pDataObj.cast(), uMsg, wParam, lParam as LPARAM);
    }

    E_FAIL
}

pub unsafe fn CDefaultContextMenu_AddStaticEntry_body(this: *mut CDefaultContextMenu, hkeyClass: HKEY, szVerb: *const WCHAR, uFlags: UINT) -> () {
    let mut it = (*this).m_StaticEntries.GetHeadPosition();
    while it != core::ptr::null_mut() {
        let info = (*this).m_StaticEntries.GetNext(&mut it);
        if (*info).Verb.CompareNoCase(szVerb) == 0 {
            return;
        }
    }

    // C: `if (!_wcsicmp(szVerb, L"open") && !(uFlags & CMF_NODEFAULT))`. Both `!` are C
    // truthiness on an int, so they are `== 0` here — Rust's `!` on an integer is a bitwise
    // complement and would have inverted the "open goes first" rule.
    if _wcsicmp(szVerb, CPP_COM_WSTR_82DB442E7A60.as_ptr()) == 0 && (uFlags & CMF_NODEFAULT) == 0 {
        /* open verb is always inserted in front */
        (*this).m_StaticEntries.AddHead(StaticShellEntry {
            Verb: CStringW::from_ptr(szVerb),
            hkClass: hkeyClass,
        });
    } else {
        (*this).m_StaticEntries.AddTail(StaticShellEntry {
            Verb: CStringW::from_ptr(szVerb),
            hkClass: hkeyClass,
        });
    }
}

pub unsafe fn CDefaultContextMenu_AddStaticEntriesForKey_body(this: *mut CDefaultContextMenu, hKey: HKEY, uFlags: UINT) -> () {
    let mut wszName: [WCHAR; VERBKEY_CCHMAX as usize] = [0; VERBKEY_CCHMAX as usize];
    let mut cchName: DWORD;
    let mut dwIndex: DWORD = 0;
    let mut hShellKey: HKEY = core::ptr::null_mut();

    let lres = RegOpenKeyExW(hKey, CPP_COM_WSTR_EE89605B31B0.as_ptr(), 0, KEY_READ, &mut hShellKey);
    if lres != STATUS_SUCCESS as i32 {
        return;
    }

    if (*this).m_DefVerbs[0] == 0 {
        let mut cb: DWORD = core::mem::size_of_val(&(*this).m_DefVerbs) as DWORD;
        RegGetValueW(hShellKey, core::ptr::null(), core::ptr::null(), RRF_RT_REG_SZ, core::ptr::null_mut(), (*this).m_DefVerbs.as_mut_ptr() as *mut _, &mut cb);
    }

    loop {
        cchName = _countof!(wszName);
        if RegEnumKeyExW(hShellKey, dwIndex, wszName.as_mut_ptr(), &mut cchName, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut()) != ERROR_SUCCESS as i32 {
            break;
        }
        dwIndex += 1;

        CDefaultContextMenu_AddStaticEntry_body(this, hKey, wszName.as_ptr(), uFlags);
    }

    RegCloseKey(hShellKey);
}

pub unsafe fn HasClipboardData_body() -> BOOL {
    let mut bRet: BOOL = FALSE;
    let mut pDataObj: *mut IDataObject = core::ptr::null_mut();
    let _pDataObj_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pDataObj));

    if OleGetClipboard(&mut pDataObj) >= 0 {
        let mut formatetc: FORMATETC = core::mem::zeroed();

        InitFormatEtc(&mut formatetc, RegisterClipboardFormatW(CFSTR_SHELLIDLIST.as_ptr()), TYMED_HGLOBAL);
        bRet = (((*(*pDataObj).lpVtbl).QueryGetData)(pDataObj, core::ptr::addr_of_mut!(formatetc).cast()) >= 0) as BOOL;

        if !pDataObj.is_null() {
            ((*(*pDataObj).lpVtbl).Release)(pDataObj);
            pDataObj = core::ptr::null_mut();
        }
    }

    bRet
}

pub unsafe fn CDefaultContextMenu_IsShellExtensionAlreadyLoaded_body(this: *mut CDefaultContextMenu, clsid: REFCLSID) -> BOOL {
    let mut it = (*this).m_DynamicEntries.GetHeadPosition();
    while it != core::ptr::null_mut() {
        let info = (*this).m_DynamicEntries.GetNext(&mut it);
        if IsEqualGUID(core::ptr::addr_of!((*info).ClassID), clsid) {
            return TRUE;
        }
    }

    return FALSE;
}

pub unsafe fn CDefaultContextMenu_LoadDynamicContextMenuHandler_body(this: *mut CDefaultContextMenu, hKey: HKEY, clsid: REFCLSID) -> HRESULT {
    let mut hr: HRESULT;

    if CDefaultContextMenu_IsShellExtensionAlreadyLoaded_body(this, clsid) != 0 {
        return S_OK;
    }

    let mut pcm: *mut IContextMenu = core::ptr::null_mut();
    let _pcm_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pcm));
    let mut pcm_refcount = 0u32;
    hr = SHCoCreateInstance(
        core::ptr::null_mut(),
        (clsid).cast(),
        core::ptr::null_mut(),
        (core::ptr::addr_of!(IID_IContextMenu)).cast(), core::ptr::addr_of_mut!(pcm).cast(),
    );
    if hr < 0 {
        return hr;
    }
    pcm_refcount += 1;

    let mut pExtInit: *mut IShellExtInit = core::ptr::null_mut();
    let _pExtInit_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pExtInit));
    let mut pExtInit_refcount = 0u32;
    hr = ((*(*pcm).lpVtbl).QueryInterface)(
        pcm,
        core::ptr::addr_of!(IID_IShellExtInit), core::ptr::addr_of_mut!(pExtInit).cast(),
    );
    if hr < 0 {
        if pcm_refcount > 0 {
            ((*(*pcm).lpVtbl).Release)(pcm);
            pcm = core::ptr::null_mut();
        }
        return hr;
    }
    pExtInit_refcount += 1;

    hr = ((*(*pExtInit).lpVtbl).Initialize)(
        pExtInit,
        if !(*this).m_pDataObj.is_null() {
            core::ptr::null_mut()
        } else {
            (*this).m_pidlFolder
        },
        (*this).m_pDataObj,
        hKey,
    );
    if hr < 0 {
        if pExtInit_refcount > 0 {
            ((*(*pExtInit).lpVtbl).Release)(pExtInit);
            pExtInit = core::ptr::null_mut();
        }
        if pcm_refcount > 0 {
            ((*(*pcm).lpVtbl).Release)(pcm);
            pcm = core::ptr::null_mut();
        }
        return hr;
    }

    if !(*this).m_site.is_null() {
        IUnknown_SetSite(pcm.cast(), (*this).m_site.cast());
    }

    (*this).m_DynamicEntries.AddTail(DynamicShellEntry {
        iIdCmdFirst: 0,
        NumIds: 0,
        ClassID: *clsid.cast(),
        pCM: atl::CComPtr { p: pcm },
    });
    pcm_refcount += 1;

    if pExtInit_refcount > 0 {
        ((*(*pExtInit).lpVtbl).Release)(pExtInit);
        pExtInit = core::ptr::null_mut();
    }
    if pcm_refcount > 0 {
        ((*(*pcm).lpVtbl).Release)(pcm);
        pcm = core::ptr::null_mut();
    }

    S_OK
}

pub unsafe fn CDefaultContextMenu_EnumerateDynamicContextHandlerForKey_body(this: *mut CDefaultContextMenu, hRootKey: HKEY) -> BOOL {
    let mut wszName: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut wszBuf: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut pwszClsid: PWSTR;
    let mut cchName: u32;
    let mut hr: HRESULT;
    let mut hKey: HKEY = core::ptr::null_mut();

    if RegOpenKeyExW(hRootKey, CPP_COM_WSTR_87660BB31B23.as_ptr(), 0, KEY_READ, &mut hKey) != ERROR_SUCCESS as i32 {
        return FALSE;
    }

    let mut dwIndex: u32 = 0;
    loop {
        cchName = _countof!(wszName);
        if RegEnumKeyExW(hKey, dwIndex, wszName.as_mut_ptr(), &mut cchName, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut()) != ERROR_SUCCESS as i32 {
            break;
        }
        dwIndex = dwIndex.wrapping_add(1);

        let mut clsid: CLSID = core::mem::zeroed();
        hr = CLSIDFromString(wszName.as_ptr(), &mut clsid);
        if hr == S_OK {
            pwszClsid = wszName.as_mut_ptr();
        } else {
            let mut cchBuf: u32 = wszBuf.len() as u32;
            if RegGetValueW(hKey, wszName.as_ptr(), core::ptr::null(), RRF_RT_REG_SZ, core::ptr::null_mut(), wszBuf.as_mut_ptr() as *mut _, &mut cchBuf) == ERROR_SUCCESS as i32 {
                hr = CLSIDFromString(wszBuf.as_ptr(), &mut clsid);
            }
            pwszClsid = wszBuf.as_mut_ptr();
        }

        if hr < 0 {
            continue;
        }

        if (*this).m_bGroupPolicyActive != 0 {
            if RegGetValueW(HKEY_LOCAL_MACHINE,
                                     CPP_COM_WSTR_D928DD0458CA.as_ptr(),
                                     pwszClsid,
                                     RRF_RT_REG_SZ,
                                     core::ptr::null_mut(),
                                     core::ptr::null_mut(),
                                     core::ptr::null_mut()) != ERROR_SUCCESS as i32 {
                continue;
            }
        }

        hr = CDefaultContextMenu_LoadDynamicContextMenuHandler_body(this, hRootKey, core::ptr::addr_of!(clsid).cast());
        if hr < 0 {
            WARN!(c"Failed to get context menu entires from shell extension! clsid: %S\n", pwszClsid);
        }
    }

    RegCloseKey(hKey);
    TRUE
}

pub unsafe fn CDefaultContextMenu_AddShellExtensionsToMenu_body(this: *mut CDefaultContextMenu, hMenu: HMENU, pIndexMenu: *mut UINT, idCmdFirst: UINT, idCmdLast: UINT, uFlags: UINT) -> UINT {
    let mut cIds: UINT = 0;

    if (*this).m_DynamicEntries.IsEmpty() {
        return cIds;
    }

    let mut it = (*this).m_DynamicEntries.GetHeadPosition();
    while it != core::ptr::null_mut() {
        let info: *mut DynamicShellEntry = (*this).m_DynamicEntries.GetNext(&mut it);

        let hr: HRESULT = ((*(*(*(*info).pCM)).lpVtbl).QueryContextMenu)(
            *(*info).pCM,
            hMenu,
            *pIndexMenu,
            idCmdFirst + cIds,
            idCmdLast,
            uFlags,
        );
        if hr >= 0 {
            (*info).iIdCmdFirst = cIds;
            (*info).NumIds = HRESULT_CODE(hr) as UINT;
            *pIndexMenu += (*info).NumIds;

            cIds += (*info).NumIds;
            if idCmdFirst + cIds >= idCmdLast {
                break;
            }
        }
    }
    return cIds;
}

pub unsafe fn CDefaultContextMenu_AddStaticContextMenusToMenu_body(this: *mut CDefaultContextMenu, hMenu: HMENU, pIndexMenu: *mut UINT, iIdCmdFirst: UINT, iIdCmdLast: UINT, uFlags: UINT) -> UINT {
    let ntver = RosGetProcessEffectiveVersion();
    let mut mii = MENUITEMINFOW {
        cbSize: core::mem::size_of::<MENUITEMINFOW>() as UINT,
        ..core::mem::zeroed()
    };
    let mut wszDispVerb: [WCHAR; 80] = core::mem::zeroed();
    let mut fState: UINT;
    let mut idVerbRes: UINT;
    let mut cIds = 0;
    let indexFirst = *pIndexMenu;
    let mut indexDefault: UINT = core::mem::zeroed();
    let mut iDefVerbIndex: i32 = -1;

    mii.fMask = MIIM_ID | MIIM_TYPE | MIIM_STATE | MIIM_DATA;
    mii.fType = MFT_STRING;

    let mut it = (*this).m_StaticEntries.GetHeadPosition();
    let mut first = true;
    while it != core::ptr::null_mut() {
        let info = (*this).m_StaticEntries.GetNext(&mut it);
        let mut forceFirstPos = FALSE;
        let mut hide = false;

        fState = MFS_ENABLED;

        if first {
            fState |= MFS_DEFAULT;
            first = false;
        }

        if (*info).Verb.CompareNoCase(CPP_COM_WSTR_82DB442E7A60.as_ptr()) == 0 {
            idVerbRes = IDS_OPEN_VERB;
            fState |= MFS_DEFAULT;
            forceFirstPos += 1;
        } else if (*info).Verb.CompareNoCase(CPP_COM_WSTR_31AF3113D116.as_ptr()) == 0 {
            idVerbRes = IDS_EXPLORE_VERB;
            if (uFlags & CMF_EXPLORE) != 0 {
                fState |= MFS_DEFAULT;
                forceFirstPos += 1;
            }
        } else if (*info).Verb.CompareNoCase(CPP_COM_WSTR_6A302070A1F6.as_ptr()) == 0 {
            hide = true;
            idVerbRes = 0;
        } else {
            idVerbRes = 0;
        }

        mii.dwTypeData = (*info).Verb.GetString() as LPWSTR;

        let mut wszKey: [WCHAR; (core::mem::size_of::<[u8; 6]>() + MAX_VERB) / core::mem::size_of::<WCHAR>()] = core::mem::zeroed();
        let hr = StringCbPrintfW!(
            wszKey.as_mut_ptr(),
            core::mem::size_of_val(&wszKey),
            CPP_COM_WSTR_A131D6DE6E8D.as_ptr(),
            (*info).Verb.GetString(),
        );
        if ((hr) < 0) {
            hide = true;
        }

        let mut cmdFlags = 0;
        let mut hkVerb: HKEY = core::mem::zeroed();
        if RegOpenKeyExW(
            (*info).hkClass,
            wszKey.as_ptr(),
            0,
            KEY_READ,
            &mut hkVerb,
        ) == ERROR_SUCCESS as i32
        {
            if (uFlags & CMF_OPTIMIZEFORINVOKE) == 0 {
                let cbVerb = core::mem::size_of_val(&wszDispVerb) as DWORD;
                let mut res = RegLoadMUIStringW(
                    hkVerb,
                    CPP_COM_WSTR_DA67076D6619.as_ptr(),
                    wszDispVerb.as_mut_ptr(),
                    cbVerb,
                    core::ptr::null_mut(),
                    0,
                    core::ptr::null_mut(),
                );
                if res != 0 || *wszDispVerb.as_ptr() == 0 {
                    res = RegLoadMUIStringW(
                        hkVerb,
                        core::ptr::null(),
                        wszDispVerb.as_mut_ptr(),
                        cbVerb,
                        core::ptr::null_mut(),
                        0,
                        core::ptr::null_mut(),
                    );
                }

                if (res == ERROR_SUCCESS as i32 && *wszDispVerb.as_ptr() != 0)
                    || (idVerbRes != 0
                        && LoadStringW(
                            shell32_hInstance,
                            idVerbRes,
                            wszDispVerb.as_mut_ptr(),
                            _countof!(wszDispVerb) as i32,
                        ) != 0)
                    || (GetFriendlyVerb_body(
                        (*info).Verb.GetString(),
                        wszDispVerb.as_mut_ptr(),
                        _countof!(wszDispVerb) as SIZE_T,
                    ) >= 0)
                {
                    mii.dwTypeData = wszDispVerb.as_mut_ptr();
                }
            }
        } else {
            hkVerb = core::ptr::null_mut();
        }

        if !hkVerb.is_null() {
            if !hide && (uFlags & CMF_EXTENDEDVERBS) == 0 {
                hide = RegValueExists(hkVerb, CPP_COM_WSTR_353157B4DE84.as_ptr());
            }

            if !hide {
                hide = RegValueExists(hkVerb, CPP_COM_WSTR_DCA3A3F4A49E.as_ptr());
            }

            if !hide && (uFlags & CMF_DISABLEDVERBS) == 0 {
                hide = RegValueExists(hkVerb, CPP_COM_WSTR_A08BD2ED73B2.as_ptr());
            }

            let dwRest: DWORD = if hide {
                0
            } else {
                RegGetDword(
                    hkVerb,
                    core::ptr::null(),
                    CPP_COM_WSTR_E58D4A4F3B68.as_ptr(),
                    0,
                )
            };
            if dwRest != 0 {
                hide = SHRestricted(dwRest as RESTRICTIONS) != 0;
            }

            if RegValueExists(hkVerb, CPP_COM_WSTR_D1B3A29D6E8A.as_ptr()) {
                fState &= !MFS_DEFAULT;
            }

            if RegValueExists(hkVerb, CPP_COM_WSTR_EE2AF488DA28.as_ptr()) {
                cmdFlags |= ECF_SEPARATORBEFORE;
            }
            if RegValueExists(hkVerb, CPP_COM_WSTR_736C68B3EC8E.as_ptr()) {
                cmdFlags |= ECF_SEPARATORAFTER;
            }

            RegCloseKey(hkVerb);
        }

        if ((uFlags & CMF_NODEFAULT) != 0 && ntver >= _WIN32_WINNT_VISTA)
            || ((uFlags & CMF_DONOTPICKDEFAULT) != 0 && ntver >= _WIN32_WINNT_WIN7)
        {
            fState &= !MFS_DEFAULT;
        }

        if !hide {
            if (cmdFlags & ECF_SEPARATORBEFORE) != 0 {
                if InsertMenuItemAt_body(hMenu, *pIndexMenu, MF_SEPARATOR) != 0 {
                    *pIndexMenu += 1;
                }
            }

            let mut pos = *pIndexMenu;
            let verbIndex: i32 = if !hkVerb.is_null() {
                FindVerbInDefaultVerbList_body((*this).m_DefVerbs.as_ptr(), (*info).Verb.GetString())
            } else {
                -1
            };
            if verbIndex >= 0 {
                if verbIndex < iDefVerbIndex || iDefVerbIndex < 0 {
                    iDefVerbIndex = verbIndex;
                    fState |= MFS_DEFAULT;
                    forceFirstPos = TRUE;
                } else {
                    fState &= !MFS_DEFAULT;
                    pos = indexDefault;
                }
            } else if iDefVerbIndex >= 0 {
                fState &= !MFS_DEFAULT;
                if forceFirstPos != 0 {
                    pos = indexDefault;
                }
                forceFirstPos = FALSE;
            }

            mii.fState = fState;
            mii.wID = iIdCmdFirst + cIds;
            if InsertMenuItemW(
                hMenu,
                if forceFirstPos != 0 { indexFirst } else { pos },
                TRUE,
                &mii,
            ) != 0
            {
                *pIndexMenu += 1;
            }

            if (cmdFlags & ECF_SEPARATORAFTER) != 0 {
                if InsertMenuItemAt_body(hMenu, *pIndexMenu, MF_SEPARATOR) != 0 {
                    *pIndexMenu += 1;
                }
            }

            if (fState & MFS_DEFAULT) != 0 {
                indexDefault = *pIndexMenu;
            }
        }
        cIds += 1;

        if mii.wID >= iIdCmdLast {
            break;
        }
    }

    cIds
}

pub unsafe fn CDefaultContextMenu_TryPickDefault_body(this: *mut CDefaultContextMenu, hMenu: HMENU, idCmdFirst: UINT, DfltOffset: UINT, uFlags: UINT) -> () {
    if (uFlags & CMF_NODEFAULT) != 0 ||
        ((uFlags & CMF_DONOTPICKDEFAULT) != 0 && RosGetProcessEffectiveVersion() >= _WIN32_WINNT_WIN7)
    {
        return;
    }

    if GetMenuDefaultItem(hMenu, MF_BYPOSITION, 0) as i32 != -1
    {
        return;
    }

    let mut forceDfm: INT_PTR = 0;
    if CDefaultContextMenu__DoCallback_body(this, DFM_GETDEFSTATICID, 0, &mut forceDfm as *mut INT_PTR as LPVOID) >= 0 && forceDfm != 0
    {
        let mut i: UINT = 0;
        while i < g_StaticInvokeCmdMap.len() as u32 {
            let menuItemId: UINT = ((*g_StaticInvokeCmdMap.as_ptr().add(i as usize)).IntVerb as UINT + DfltOffset - DCM_FCIDM_SHVIEW_OFFSET).into();
            if INT_PTR::from((*g_StaticInvokeCmdMap.as_ptr().add(i as usize)).DfmCmd) == forceDfm &&
                SetMenuDefaultItem(hMenu, menuItemId, MF_BYCOMMAND) != 0
            {
                return;
            }
            i += 1;
        }
    }

    if (*this).m_iIdSCMLast > (*this).m_iIdSCMFirst || (*this).m_iIdSHELast > (*this).m_iIdSHEFirst
    {
        SetMenuDefaultItem(hMenu, idCmdFirst, MF_BYCOMMAND);
    }
}

pub unsafe fn CDefaultContextMenu_QueryContextMenu_body(this: *mut CDefaultContextMenu, hMenu: HMENU, IndexMenu: UINT, idCmdFirst: UINT, idCmdLast: UINT, uFlags: UINT) -> HRESULT {
    let hr: HRESULT;
    let mut idCmdNext: UINT = idCmdFirst;
    (*this).m_iIdQCMFirst = idCmdFirst;
    let mut cIds: UINT = 0;

    let mut i: UINT = 0;
    while i < (*this).m_cKeys && (uFlags & CMF_NOVERBS) == 0 {
        CDefaultContextMenu_AddStaticEntriesForKey_body(this, (*this).m_aKeys[i as usize], uFlags);
        CDefaultContextMenu_EnumerateDynamicContextHandlerForKey_body(this, (*this).m_aKeys[i as usize]);
        i += 1;
    }

    let mut IndexMenu_mut = IndexMenu;
    cIds = CDefaultContextMenu_AddStaticContextMenusToMenu_body(this, hMenu, &mut IndexMenu_mut, idCmdFirst, idCmdLast, uFlags);
    (*this).m_iIdSCMFirst = 0;
    (*this).m_iIdSCMLast = cIds;
    idCmdNext = idCmdFirst + cIds;

    cIds += CDefaultContextMenu_AddShellExtensionsToMenu_body(this, hMenu, &mut IndexMenu_mut, idCmdNext, idCmdLast, uFlags);
    (*this).m_iIdSHEFirst = (*this).m_iIdSCMLast;
    (*this).m_iIdSHELast = cIds;
    idCmdNext = idCmdFirst + cIds;

    let mut qcminfo = QCMINFO {
        hmenu: hMenu,
        indexMenu: IndexMenu_mut,
        idCmdFirst: idCmdNext,
        idCmdLast: idCmdLast,
        // C's positional aggregate `{hMenu, IndexMenu, idCmdNext, idCmdLast, NULL}` — the
        // generator took the 5th initializer's spelling for the field name.
        pIdMap: core::ptr::null(),
    };
    if CDefaultContextMenu__DoCallback_body(this, DFM_MERGECONTEXTMENU, uFlags as WPARAM, core::ptr::addr_of_mut!(qcminfo).cast()) >= 0 {
        let added = qcminfo.idCmdFirst - idCmdNext;
        cIds += added;
        IndexMenu_mut += added;
        (*this).m_iIdCBFirst = (*this).m_iIdSHELast;
        (*this).m_iIdCBLast = cIds;
        idCmdNext = idCmdFirst + cIds;
    }

    let mut idDefaultOffset: UINT = 0;
    let isBackgroundMenu: BOOL = ((*this).m_cidl == 0 ) as BOOL;
    if (uFlags & CMF_VERBSONLY) == 0 && isBackgroundMenu == 0 {
        let mut rfg: SFGAOF = SFGAO_BROWSABLE | SFGAO_CANCOPY | SFGAO_CANLINK | SFGAO_CANMOVE | SFGAO_CANDELETE | SFGAO_CANRENAME | SFGAO_HASPROPSHEET | SFGAO_FILESYSTEM | SFGAO_FOLDER;
        hr = ((*(*(*this).m_psf).lpVtbl).GetAttributesOf)((*this).m_psf, (*this).m_cidl, (*this).m_apidl.cast(), &mut rfg);
        if ((hr) < 0) {
            return MAKE_HRESULT(SEVERITY_SUCCESS, 0, cIds);
        }

        let hmenuDefault: HMENU = LoadMenuW(shell32_hInstance, MAKEINTRESOURCEW(MENU_SHV_FILE));

        if (rfg & SFGAO_CANMOVE) == 0 {
            DeleteMenu(hmenuDefault, IDM_CUT, MF_BYCOMMAND);
        }
        if (rfg & SFGAO_CANCOPY) == 0 {
            DeleteMenu(hmenuDefault, IDM_COPY, MF_BYCOMMAND);
        }
        if !((rfg & SFGAO_FILESYSTEM) != 0 && HasClipboardData_body() != 0) {
            DeleteMenu(hmenuDefault, IDM_INSERT, MF_BYCOMMAND);
        }
        if (rfg & SFGAO_CANLINK) == 0 {
            DeleteMenu(hmenuDefault, IDM_CREATELINK, MF_BYCOMMAND);
        }
        if (rfg & SFGAO_CANDELETE) == 0 {
            DeleteMenu(hmenuDefault, IDM_DELETE, MF_BYCOMMAND);
        }
        if (rfg & SFGAO_CANRENAME) == 0 || (uFlags & CMF_CANRENAME) == 0 {
            DeleteMenu(hmenuDefault, IDM_RENAME, MF_BYCOMMAND);
        }
        if (rfg & SFGAO_HASPROPSHEET) == 0 {
            DeleteMenu(hmenuDefault, IDM_PROPERTIES, MF_BYCOMMAND);
        }

        idDefaultOffset = idCmdNext;
        let idMax: UINT = Shell_MergeMenus(hMenu, GetSubMenu(hmenuDefault, 0), IndexMenu_mut, idCmdNext, idCmdLast, 0);
        (*this).m_iIdDfltFirst = cIds;
        cIds += idMax - idCmdNext;
        (*this).m_iIdDfltLast = cIds;

        DestroyMenu(hmenuDefault);
    }

    CDefaultContextMenu_TryPickDefault_body(this, hMenu, idCmdFirst, idDefaultOffset, uFlags);

    return MAKE_HRESULT(SEVERITY_SUCCESS, 0, cIds);
}

pub unsafe fn CDefaultContextMenu_DoPaste_body(this: *mut CDefaultContextMenu, lpcmi: LPCMINVOKECOMMANDINFOEX, bLink: BOOL) -> HRESULT {
    let mut hr = CDefaultContextMenu__DoInvokeCommandCallback_body(this, lpcmi, DFM_CMD_PASTE as WPARAM);
    if hr == S_OK {
        return hr;
    }

    let mut pda: *mut IDataObject = core::ptr::null_mut();
    let _pda_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pda));
    hr = OleGetClipboard(&mut pda);
    if ((hr) < 0) {
        return hr;
    }

    let mut formatetc2: FORMATETC = core::mem::zeroed();
    let mut medium2: STGMEDIUM = core::mem::zeroed();
    InitFormatEtc(&mut formatetc2, RegisterClipboardFormatW(CFSTR_PREFERREDDROPEFFECT.as_ptr()), TYMED_HGLOBAL);

    let mut dwKey = 0;

    if ((*(*pda).lpVtbl).GetData)(pda, core::ptr::addr_of_mut!(formatetc2).cast(), core::ptr::addr_of_mut!(medium2).cast()) >= 0 {
        let pdwFlag = GlobalLock(medium2.__bindgen_anon_1.hGlobal) as *mut DWORD;
        if !pdwFlag.is_null() {
            if *pdwFlag == DROPEFFECT_COPY {
                dwKey = MK_CONTROL;
            } else {
                dwKey = MK_SHIFT;
            }
        } else {
        }
        GlobalUnlock(medium2.__bindgen_anon_1.hGlobal);
    }

    if bLink != 0 {
        dwKey = MK_CONTROL | MK_SHIFT;
    }

    let mut pdrop: *mut IDropTarget = core::ptr::null_mut();
    let _pdrop_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pdrop));
    if (*this).m_cidl != 0 {
        hr = ((*(*(*this).m_psf).lpVtbl).GetUIObjectOf)(
            (*this).m_psf,
            core::ptr::null_mut(),
            1,
            (*this).m_apidl.wrapping_add(0).cast(),
            core::ptr::addr_of!(IID_IDropTarget).cast(), core::ptr::null_mut(), core::ptr::addr_of_mut!(pdrop).cast()
        );
    } else {
        hr = ((*(*(*this).m_psf).lpVtbl).CreateViewObject)(
            (*this).m_psf,
            core::ptr::null_mut(),
            core::ptr::addr_of!(IID_IDropTarget).cast(), core::ptr::addr_of_mut!(pdrop).cast()
        );
    }

    if ((hr) < 0) {
        if !pda.is_null() {
            ((*(*pda).lpVtbl).Release)(pda);
            pda = core::ptr::null_mut();
        }
        return hr;
    }

    SHSimulateDrop(pdrop.cast(), pda.cast(), dwKey, core::ptr::null_mut(), core::ptr::null_mut());

    if !pdrop.is_null() {
        ((*(*pdrop).lpVtbl).Release)(pdrop);
        pdrop = core::ptr::null_mut();
    }
    if !pda.is_null() {
        ((*(*pda).lpVtbl).Release)(pda);
        pda = core::ptr::null_mut();
    }

    S_OK
}


pub unsafe fn CDefaultContextMenu_DoCreateLink_body(this: *mut CDefaultContextMenu, lpcmi: LPCMINVOKECOMMANDINFOEX) -> HRESULT {
    let hr = CDefaultContextMenu__DoInvokeCommandCallback_body(this, lpcmi, DFM_CMD_LINK as WPARAM);
    if hr == S_OK {
        return hr;
    }

    if (*this).m_cidl == 0 || (*this).m_pDataObj.is_null() {
        return E_FAIL;
    }

    let mut pDT: *mut IDropTarget = core::ptr::null_mut();
    let _pDT_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pDT));
    let hr = ((*(*(*this).m_psf).lpVtbl).CreateViewObject)(
        (*this).m_psf,
        core::ptr::null_mut(),
        core::ptr::addr_of!(IID_IDropTarget).cast(),
        &mut pDT as *mut *mut IDropTarget as *mut *mut core::ffi::c_void
    );
    if !pDT.is_null() {
        ((*(*pDT).lpVtbl).AddRef)(pDT);
    }
    if ((hr) < 0) {
        if !pDT.is_null() {
            ((*(*pDT).lpVtbl).Release)(pDT);
            pDT = core::ptr::null_mut();
        }
        return hr;
    }

    SHSimulateDrop(pDT.cast(), (*this).m_pDataObj.cast(), MK_CONTROL | MK_SHIFT, core::ptr::null_mut(), core::ptr::null_mut());

    if !pDT.is_null() {
        ((*(*pDT).lpVtbl).Release)(pDT);
        pDT = core::ptr::null_mut();
    }
    S_OK
}

pub unsafe fn CDefaultContextMenu_DoDelete_body(this: *mut CDefaultContextMenu, lpcmi: LPCMINVOKECOMMANDINFOEX) -> HRESULT {
    let mut hr = CDefaultContextMenu__DoInvokeCommandCallback_body(this, lpcmi, DFM_CMD_DELETE as WPARAM);
    if hr == S_OK {
        return hr;
    }

    if (*this).m_cidl == 0 || (*this).m_pDataObj.is_null() {
        return E_FAIL;
    }

    let mut pDT: *mut IDropTarget = core::ptr::null_mut();
    let _pDT_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pDT));
    hr = CRecyclerDropTarget_CreateInstance_RustBridge(core::ptr::addr_of!(IID_IDropTarget).cast(), core::ptr::addr_of_mut!(pDT).cast());
    if ((hr) < 0) {
        return hr;
    }

    let grfKeyState: u32 = if (*lpcmi).fMask & CMIC_MASK_SHIFT_DOWN != 0 {
        MK_SHIFT
    } else {
        0
    };
    SHSimulateDrop(pDT.cast(), (*this).m_pDataObj.cast(), grfKeyState, core::ptr::null_mut(), core::ptr::null_mut());

    if !pDT.is_null() {
        ((*(*pDT).lpVtbl).Release)(pDT);
        pDT = core::ptr::null_mut();
    }

    S_OK
}

pub unsafe fn CDefaultContextMenu_DoCopyOrCut_body(this: *mut CDefaultContextMenu, lpcmi: LPCMINVOKECOMMANDINFOEX, bCopy: BOOL) -> HRESULT {
    if (*this).m_cidl == 0 || (*this).m_pDataObj.is_null() {
        return E_FAIL;
    }

    let mut formatetc: FORMATETC = core::mem::zeroed();
    InitFormatEtc(
        &mut formatetc,
        RegisterClipboardFormatW(CFSTR_PREFERREDDROPEFFECT.as_ptr()),
        TYMED_HGLOBAL,
    );
    let mut medium: STGMEDIUM = core::mem::zeroed();
    medium.tymed = TYMED_HGLOBAL;
    medium.__bindgen_anon_1.hGlobal = GlobalAlloc(GHND, core::mem::size_of::<DWORD>() as SIZE_T);
    let pdwFlag = GlobalLock(medium.__bindgen_anon_1.hGlobal) as *mut DWORD;
    if !pdwFlag.is_null() {
        *pdwFlag = if bCopy != 0 {
            DROPEFFECT_COPY
        } else {
            DROPEFFECT_MOVE
        };
    }
    GlobalUnlock(medium.__bindgen_anon_1.hGlobal);
    ((*(*(*this).m_pDataObj).lpVtbl).SetData)(
        (*this).m_pDataObj,
        core::ptr::addr_of_mut!(formatetc).cast(),
        core::ptr::addr_of_mut!(medium).cast(),
        TRUE,
    );

    let mut psfv: *mut IShellFolderView = core::ptr::null_mut();
    let _psfv_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(psfv));
    if IUnknown_QueryService(
        ((*this).m_site).cast(),
        core::ptr::addr_of!(IID_IFolderView).cast(),
        (core::ptr::addr_of!(IID_IShellFolderView)).cast(), core::ptr::addr_of_mut!(psfv).cast(),
    ) >= 0
    {
        ((*(*psfv).lpVtbl).SetPoints)(psfv, (*this).m_pDataObj);
    }

    let hr = OleSetClipboard((*this).m_pDataObj);
    if ((hr) < 0) {
        return hr;
    }

    if !psfv.is_null() {
        ((*(*psfv).lpVtbl).SetClipboard)(psfv, if bCopy != 0 { 0 } else { 1 });
    }
    S_OK
}

pub unsafe fn CDefaultContextMenu_DoRename_body(this: *mut CDefaultContextMenu, lpcmi: LPCMINVOKECOMMANDINFOEX) -> HRESULT {
    let mut psb: *mut IShellBrowser = core::ptr::null_mut();
    let _psb_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(psb));
    let mut hr: HRESULT;

    hr = CDefaultContextMenu__DoInvokeCommandCallback_body(this, lpcmi, DFM_CMD_RENAME as WPARAM);
    if hr == S_OK {
        return hr;
    }

    if (*this).m_site.is_null() || (*this).m_cidl == 0 {
        return E_FAIL;
    }

    // shlguid_undoc.h:215: `SID_IShellBrowser` IS `IID_IShellBrowser` (a #define, not
    // its own symbol) — the generator's `extern static SID_IShellBrowser` was a phantom.
    hr = IUnknown_QueryService(((*this).m_site).cast(), core::ptr::from_ref(&IID_IShellBrowser).cast(), core::ptr::from_ref(&IID_IShellBrowser).cast(), &mut psb as *mut *mut IShellBrowser as *mut *mut core::ffi::c_void);
    if ((hr) < 0) {
        if !psb.is_null() {
            ((*(*psb).lpVtbl).Release)(psb);
            psb = core::ptr::null_mut();
        }
        return hr;
    }

    let mut lpSV: *mut IShellView = core::ptr::null_mut();
    let _lpSV_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(lpSV));
    hr = ((*(*psb).lpVtbl).QueryActiveShellView)(psb, &mut lpSV);
    if ((hr) < 0) {
        if !lpSV.is_null() {
            ((*(*lpSV).lpVtbl).Release)(lpSV);
            lpSV = core::ptr::null_mut();
        }
        if !psb.is_null() {
            ((*(*psb).lpVtbl).Release)(psb);
            psb = core::ptr::null_mut();
        }
        return hr;
    }

    let selFlags: SVSIF = SVSI_DESELECTOTHERS | SVSI_EDIT | SVSI_ENSUREVISIBLE | SVSI_FOCUSED | SVSI_SELECT;
    hr = ((*(*lpSV).lpVtbl).SelectItem)(lpSV, *(*this).m_apidl.offset(0 as isize), selFlags);
    if ((hr) < 0) {
        if !lpSV.is_null() {
            ((*(*lpSV).lpVtbl).Release)(lpSV);
            lpSV = core::ptr::null_mut();
        }
        if !psb.is_null() {
            ((*(*psb).lpVtbl).Release)(psb);
            psb = core::ptr::null_mut();
        }
        return hr;
    }

    if !lpSV.is_null() {
        ((*(*lpSV).lpVtbl).Release)(lpSV);
        lpSV = core::ptr::null_mut();
    }
    if !psb.is_null() {
        ((*(*psb).lpVtbl).Release)(psb);
        psb = core::ptr::null_mut();
    }

    S_OK
}

pub unsafe fn CDefaultContextMenu_DoProperties_body(this: *mut CDefaultContextMenu, lpcmi: LPCMINVOKECOMMANDINFOEX) -> HRESULT {
    let hr = CDefaultContextMenu__DoInvokeCommandCallback_body(this, lpcmi, DFM_CMD_PROPERTIES as WPARAM);

    if hr == S_FALSE {
        return SHELL32_ShowPropertiesDialog((*this).m_pDataObj);
    }

    hr
}

pub unsafe fn CDefaultContextMenu_DoUndo_body(this: *mut CDefaultContextMenu, lpcmi: LPCMINVOKECOMMANDINFOEX) -> HRESULT { E_NOTIMPL }

pub unsafe fn CDefaultContextMenu_DoCopyToMoveToFolder_body(this: *mut CDefaultContextMenu, lpici: LPCMINVOKECOMMANDINFOEX, bCopy: BOOL) -> HRESULT {
    let mut hr: HRESULT = E_FAIL;
    if (*this).m_pDataObj.is_null() {
        return hr;
    }

    let mut pContextMenu: *mut IContextMenu = core::ptr::null_mut();
    let _pContextMenu_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pContextMenu));
    if bCopy != 0 {
        hr = SHCoCreateInstance(
            core::ptr::null_mut(),
            core::ptr::from_ref(&CLSID_CopyToMenu).cast(),
            core::ptr::null_mut(),
            (core::ptr::addr_of!(IID_IContextMenu)).cast(), core::ptr::addr_of_mut!(pContextMenu).cast(),
        );
    } else {
        hr = SHCoCreateInstance(
            core::ptr::null_mut(),
            core::ptr::from_ref(&CLSID_MoveToMenu).cast(),
            core::ptr::null_mut(),
            (core::ptr::addr_of!(IID_IContextMenu)).cast(), core::ptr::addr_of_mut!(pContextMenu).cast(),
        );
    }
    if ((hr) < 0) {
        return hr;
    }
    let pContextMenu_refcount = core::cell::Cell::new(1u32);

    let mut pInit: *mut IShellExtInit = core::ptr::null_mut();
    let _pInit_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pInit));
    hr = ((*(*pContextMenu).lpVtbl).QueryInterface)(
        pContextMenu,
        core::ptr::addr_of!(IID_IShellExtInit), core::ptr::addr_of_mut!(pInit).cast(),
    );
    if ((hr) < 0) {
        ((*(*pContextMenu).lpVtbl).Release)(pContextMenu);
        pContextMenu = core::ptr::null_mut();
        return hr;
    }
    let pInit_refcount = core::cell::Cell::new(1u32);

    hr = ((*(*pInit).lpVtbl).Initialize)(pInit, (*this).m_pidlFolder, (*this).m_pDataObj, core::ptr::null_mut());
    if ((hr) < 0) {
        ((*(*pInit).lpVtbl).Release)(pInit);
        pInit = core::ptr::null_mut();
        ((*(*pContextMenu).lpVtbl).Release)(pContextMenu);
        pContextMenu = core::ptr::null_mut();
        return hr;
    }

    if bCopy != 0 {
        (*lpici).lpVerb = b"copyto\0".as_ptr() as *const i8;
    } else {
        (*lpici).lpVerb = b"moveto\0".as_ptr() as *const i8;
    }

    let result = ((*(*pContextMenu).lpVtbl).InvokeCommand)(
        pContextMenu,
        lpici as LPCMINVOKECOMMANDINFO,
    );

    ((*(*pInit).lpVtbl).Release)(pInit);
    pInit = core::ptr::null_mut();
    ((*(*pContextMenu).lpVtbl).Release)(pContextMenu);
    pContextMenu = core::ptr::null_mut();

    result
}

pub unsafe fn CDefaultContextMenu_GetDynamicEntry_body(this: *mut CDefaultContextMenu, idCmd: UINT) -> PDynamicShellEntry {
    let mut it = (*this).m_DynamicEntries.GetHeadPosition();
    while it != core::ptr::null_mut() {
        let info: *mut DynamicShellEntry = (*this).m_DynamicEntries.GetNext(&mut it);

        if idCmd >= (*info).iIdCmdFirst + (*info).NumIds {
            continue;
        }

        if idCmd < (*info).iIdCmdFirst || idCmd > (*info).iIdCmdFirst + (*info).NumIds {
            return core::ptr::null_mut();
        }

        return info;
    }

    core::ptr::null_mut()
}

// Hand-written: the model would not reproduce this one faithfully. The C frees `pidl`
// exactly ONCE, on the success path, and leaks it when SelectItem fails; every sample
// "helpfully" added a second SHFree on the failure path, which the call envelope rejected
// as a duplicated token. The leak is ReactOS's, and a translation is not the place to fix
// it — see the C at CDefaultContextMenu.cpp:1465.
pub unsafe fn CDefaultContextMenu_DoCreateNewFolder_body(this: *mut CDefaultContextMenu, lpici: LPCMINVOKECOMMANDINFOEX) -> HRESULT {
    let mut wszPath: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut wszName: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut wszNewFolder: [u16; 25] = [0; 25];
    let mut hr: HRESULT;

    /* Get folder path */
    hr = SHGetPathFromIDListW((*this).m_pidlFolder, wszPath.as_mut_ptr());
    if ((hr) < 0) {
        return hr;
    }

    if LoadStringW(
        shell32_hInstance,
        IDS_NEWFOLDER,
        wszNewFolder.as_mut_ptr(),
        wszNewFolder.len() as INT,
    ) == 0
    {
        return E_FAIL;
    }

    /* Create the name of the new directory */
    if PathYetAnotherMakeUniqueName(
        wszName.as_mut_ptr(),
        wszPath.as_ptr(),
        core::ptr::null(),
        wszNewFolder.as_ptr(),
    ) == 0
    {
        return E_FAIL;
    }

    /* Create the new directory and show the appropriate dialog in case of error */
    if SHCreateDirectory((*lpici).hwnd, wszName.as_ptr()) != ERROR_SUCCESS as INT {
        return E_FAIL;
    }

    /* Show and select the new item in the def view */
    let mut pidl: LPITEMIDLIST = core::ptr::null_mut();
    let pidlNewItem: PITEMID_CHILD;
    let mut psv: *mut IShellView = core::ptr::null_mut();
    let _psv_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(psv));

    /* Notify the view object about the new item */
    SHChangeNotify(
        SHCNE_MKDIR as LONG,
        SHCNF_PATHW | SHCNF_FLUSH,
        wszName.as_ptr() as LPCVOID,
        core::ptr::null(),
    );

    if (*this).m_site.is_null() {
        return S_OK;
    }

    /* Get a pointer to the shell view */
    hr = IUnknown_QueryService(
        ((*this).m_site).cast(),
        // shobjidl.idl:904: `SID_SFolderView` IS `IID_IFolderView` (a #define).
        (core::ptr::addr_of!(IID_IFolderView)).cast(),
        (core::ptr::addr_of!(IID_IShellView)).cast(),
        core::ptr::addr_of_mut!(psv).cast(),
    );
    if ((hr) < 0) {
        return S_OK;
    }

    /* Attempt to get the pidl of the new item */
    hr = SHILCreateFromPathW(wszName.as_ptr(), &mut pidl, core::ptr::null_mut());
    if ((hr) < 0) {
        return hr;
    }

    pidlNewItem = ILFindLastID(pidl);

    hr = ((*(*psv).lpVtbl).SelectItem)(
        psv,
        pidlNewItem,
        SVSI_DESELECTOTHERS | SVSI_EDIT | SVSI_ENSUREVISIBLE | SVSI_FOCUSED | SVSI_SELECT,
    );
    if ((hr) < 0) {
        // The C returns here WITHOUT freeing pidl. Faithful, leak included.
        return hr;
    }

    SHFree(pidl);

    S_OK
}

pub unsafe fn CDefaultContextMenu_MapVerbToCmdId_body(this: *mut CDefaultContextMenu, Verb: PVOID, idCmd: PUINT, IsUnicode: BOOL) -> BOOL {
    let mut UnicodeStr: [WCHAR; MAX_VERB] = [0; MAX_VERB];

    let mut i: UINT = 0;
    while i < g_StaticInvokeCmdMap.len() as u32 {
        if IsUnicode != 0 {
            SHAnsiToUnicode(
                (*g_StaticInvokeCmdMap.as_ptr().add(i as usize)).szStringVerb,
                UnicodeStr.as_mut_ptr(),
                MAX_VERB as i32,
            );
            if _wcsicmp(UnicodeStr.as_ptr(), Verb as LPWSTR) == 0 {
                *idCmd = (*g_StaticInvokeCmdMap.as_ptr().add(i as usize)).IntVerb as UINT;
                return TRUE;
            }
        } else {
            if _stricmp(
                (*g_StaticInvokeCmdMap.as_ptr().add(i as usize)).szStringVerb,
                Verb as LPSTR,
            ) == 0
            {
                *idCmd = (*g_StaticInvokeCmdMap.as_ptr().add(i as usize)).IntVerb as UINT;
                return TRUE;
            }
        }
        i += 1;
    }

    // Check the registry verbs (upstream 16bb138e325). Upstream compares against the
    // scratch buffer in both cases, which for a Unicode verb still holds the last static
    // verb converted above; compare the caller's Unicode verb itself instead.
    let mut verb_w: *const WCHAR = Verb as *const WCHAR;
    if IsUnicode == 0 {
        // StringCchPrintfW(L"%hs") fails rather than truncate; so does this.
        verb_w = if lstrlenA(Verb as LPCSTR) < MAX_VERB as i32
            && SHAnsiToUnicode(Verb as LPCSTR, UnicodeStr.as_mut_ptr(), MAX_VERB as i32) != 0
        {
            UnicodeStr.as_ptr()
        } else {
            core::ptr::null()
        };
    }
    if !verb_w.is_null() {
        let mut i: UINT = 0;
        let mut it: POSITION = (*this).m_StaticEntries.GetHeadPosition();
        while it != core::ptr::null_mut() {
            let entry = (*this).m_StaticEntries.GetNext(&mut it);
            if (*entry).Verb.CompareNoCase(verb_w) == 0 {
                *idCmd = (*this).m_iIdSCMFirst + i;
                return TRUE;
            }
            i += 1;
        }
    }

    let mut it: POSITION = (*this).m_DynamicEntries.GetHeadPosition();
    while it != core::ptr::null_mut() {
        let entry: *mut DynamicShellEntry = (*this).m_DynamicEntries.GetNext(&mut it);
        if (*entry).NumIds == 0 {
            continue;
        }
        let hr: HRESULT = MapVerbToCmdId_body(
            Verb,
            IsUnicode,
            *(*entry).pCM,
            0,
            (*entry).NumIds - 1,
        );
        if hr >= 0 {
            *idCmd = (*this).m_iIdSHEFirst + (*entry).iIdCmdFirst + hr as UINT;
            return TRUE;
        }
    }

    // Last, let the folder callback name its own verbs (upstream 62519e131a7):
    // learn.microsoft.com/en-us/windows/win32/shell/dfm-mapcommandname
    let mut pszVerbW: PWSTR = Verb as PWSTR;
    if IsUnicode == 0 {
        SHAnsiToUnicode(Verb as LPCSTR, UnicodeStr.as_mut_ptr(), MAX_VERB as i32);
        pszVerbW = UnicodeStr.as_mut_ptr();
    }
    let mut i: UINT = UINT::MAX; // i = -1
    if *pszVerbW != 0
        && CDefaultContextMenu__DoCallback_body(this, DFM_MAPCOMMANDNAME, &mut i as *mut UINT as WPARAM, pszVerbW as LPVOID) >= 0
        && HIWORD(i as DWORD) == 0
    {
        *idCmd = (*this).m_iIdCBFirst + i;
        return TRUE;
    }

    return FALSE; // even for "open": no handler means no command
}

// Pre-Vista callers get E_INVALIDARG for an unknown verb, Vista+ ERROR_NO_ASSOCIATION.
unsafe fn CDefaultContextMenu_GetNoAssocError() -> HRESULT {
    if LOBYTE(GetVersion()) < 6 { E_INVALIDARG } else { HRESULT_FROM_WIN32(ERROR_NO_ASSOCIATION as LONG) }
}

pub unsafe fn CDefaultContextMenu_InvokeShellExt_body(this: *mut CDefaultContextMenu, lpcmi: LPCMINVOKECOMMANDINFOEX) -> HRESULT {
    let idCmd: UINT = LOWORD!((*lpcmi).lpVerb);
    let pEntry: PDynamicShellEntry = CDefaultContextMenu_GetDynamicEntry_body(this, idCmd);
    if pEntry.is_null() {
        return E_FAIL;
    }

    (*lpcmi).lpVerb = MAKEINTRESOURCEA(idCmd.wrapping_sub((*pEntry).iIdCmdFirst));
    ((*(*(*(*pEntry).pCM)).lpVtbl).InvokeCommand)(
        *(*pEntry).pCM,
        lpcmi as LPCMINVOKECOMMANDINFO,
    )
}

pub unsafe fn CDefaultContextMenu_BrowserFlagsFromVerb_body(this: *mut CDefaultContextMenu, lpcmi: LPCMINVOKECOMMANDINFOEX, pEntry: PStaticShellEntry) -> DWORD {
    let mut psb: *mut IShellBrowser = core::ptr::null_mut();
    let _psb_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(psb));
    let mut hwndTree: HWND = core::ptr::null_mut();
    let FlagsName: PCWSTR;
    let mut wszKey: [u16; (core::mem::size_of::<[u8; 6]>() + MAX_VERB)] = [0; (core::mem::size_of::<[u8; 6]>() + MAX_VERB)];
    let hr: HRESULT;

    if (*this).m_site.is_null() {
        return 0;
    }

    hr = IUnknown_QueryService(((*this).m_site).cast(), core::ptr::addr_of!(IID_IShellBrowser).cast(), (core::ptr::addr_of!(IID_IShellBrowser)).cast(), core::ptr::addr_of_mut!(psb).cast());
    if ((hr) < 0) {
        return 0;
    }

    if ((((*(*psb).lpVtbl).GetControlWindow)(psb, FCW_TREE, &mut hwndTree)) >= 0) && !hwndTree.is_null() {
        FlagsName = CPP_COM_WSTR_9A7909160783.as_ptr();
    } else {
        FlagsName = CPP_COM_WSTR_325A56B0AAB7.as_ptr();
    }

    let mut pcdb: *mut ICommDlgBrowser = core::ptr::null_mut();
    let _pcdb_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pcdb));
    if ((((*(*psb).lpVtbl).QueryInterface)(psb, core::ptr::addr_of!(IID_ICommDlgBrowser), core::ptr::addr_of_mut!(pcdb).cast())) >= 0) {
        if LOBYTE(GetVersion()) < 6 || *FlagsName.offset(0) == 'E' as u16 {
            return 0;
        }
    }

    let hr = StringCbPrintfW!(
        wszKey.as_mut_ptr(),
        core::mem::size_of_val(&wszKey),
        CPP_COM_WSTR_A131D6DE6E8D.as_ptr(),
        (*pEntry).Verb.GetString(),
    );
    if ((hr) < 0) {
        return 0;
    }
    RegGetDword((*pEntry).hkClass, wszKey.as_ptr(), FlagsName, 0)
}

pub unsafe fn CDefaultContextMenu_TryToBrowse_body(this: *mut CDefaultContextMenu, lpcmi: LPCMINVOKECOMMANDINFOEX, pidlChild: LPCITEMIDLIST, wFlags: DWORD) -> HRESULT {
    let mut psb: *mut IShellBrowser = core::ptr::null_mut();
    let _psb_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(psb));
    let hr: HRESULT;

    if (*this).m_site.is_null() {
        return E_FAIL;
    }

    hr = IUnknown_QueryService(
        ((*this).m_site).cast(),
        core::ptr::addr_of!(IID_IShellBrowser).cast(),
        (core::ptr::addr_of!(IID_IShellBrowser)).cast(), core::ptr::addr_of_mut!(psb).cast(),
    );
    if hr < 0 {
        if !psb.is_null() {
            ((*(*psb).lpVtbl).Release)(psb);
            psb = core::ptr::null_mut();
        }
        return hr;
    }

    let mut pidl: PIDLIST_ABSOLUTE = core::ptr::null_mut();
    let hr = SHILCombine((*this).m_pidlFolder, pidlChild, &mut pidl);
    if ((hr) < 0) {
        if !psb.is_null() {
            ((*(*psb).lpVtbl).Release)(psb);
            psb = core::ptr::null_mut();
        }
        return hr;
    }

    let hr = ((*(*psb).lpVtbl).BrowseObject)(psb, pidl, wFlags & !SBSP_RELATIVE);
    ILFree(pidl);

    if !psb.is_null() {
        ((*(*psb).lpVtbl).Release)(psb);
        psb = core::ptr::null_mut();
    }

    hr
}

pub unsafe fn CDefaultContextMenu_InvokePidl_body(this: *mut CDefaultContextMenu, lpcmi: LPCMINVOKECOMMANDINFOEX, pidl: LPCITEMIDLIST, pEntry: PStaticShellEntry) -> HRESULT {
    let unicode = IsUnicode(lpcmi);

    let pidlFull = ILCombine((*this).m_pidlFolder, pidl);
    if pidlFull.is_null() {
        return E_FAIL;
    }

    let mut wszPath: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let bHasPath = SHGetPathFromIDListW(pidlFull, wszPath.as_mut_ptr());

    let mut wszDir: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];

    let mut sei: SHELLEXECUTEINFOW = core::mem::zeroed();
    sei.cbSize = core::mem::size_of::<SHELLEXECUTEINFOW>() as u32;
    sei.fMask = SEE_MASK_CLASSKEY | SEE_MASK_IDLIST | (CmicFlagsToSeeFlags((*lpcmi).fMask) & !SEE_MASK_INVOKEIDLIST);
    sei.hwnd = (*lpcmi).hwnd;
    sei.nShow = (*lpcmi).nShow;
    sei.lpVerb = (*pEntry).Verb.GetString();
    sei.lpIDList = pidlFull as *mut core::ffi::c_void;
    sei.hkeyClass = (*pEntry).hkClass;
    sei.dwHotKey = (*lpcmi).dwHotKey;
    sei.icon_or_monitor = (*lpcmi).hIcon as HANDLE;  // C's DUMMYUNIONNAME { hIcon; hMonitor; }
    sei.lpDirectory = wszDir.as_ptr();

    if unicode && !StrIsNullOrEmptyW((*lpcmi).lpDirectoryW) {
        sei.lpDirectory = (*lpcmi).lpDirectoryW;
    } else if bHasPath != 0 {
        wcscpy(wszDir.as_mut_ptr(), wszPath.as_ptr());
        PathRemoveFileSpec(wszDir.as_mut_ptr());
    } else {
        if SHGetPathFromIDListW((*this).m_pidlFolder, wszDir.as_mut_ptr()) == 0 {
            wszDir[0] = UNICODE_NULL;
        }
    }

    if bHasPath != 0 {
        sei.lpFile = wszPath.as_ptr();
    }

    let mut pszParamsW: CComHeapPtr<u16> = CComHeapPtr::new();
    if unicode && !StrIsNullOrEmptyW((*lpcmi).lpParametersW) {
        sei.lpParameters = (*lpcmi).lpParametersW;
    } else if !StrIsNullOrEmptyA((*lpcmi).lpParameters)
        && !__SHCloneStrAtoW(&mut pszParamsW.m_pData, (*lpcmi).lpParameters).is_null()
    {
        sei.lpParameters = pszParamsW.m_pData;
    }

    if sei.lpClass.is_null() && ((*lpcmi).fMask & (CMIC_MASK_HASLINKNAME | CMIC_MASK_HASTITLE)) != 0 && unicode {
        sei.lpClass = (*lpcmi).lpTitleW;
    }

    let hr = if ShellExecuteExW(core::ptr::addr_of_mut!(sei).cast()) != 0 {
        S_OK
    } else {
        HResultFromWin32(GetLastError())
    };
    ILFree(pidlFull);
    hr
}

// Hand-written: --skip-fn'd out of the paid worklist. The C++ builds on three constructs
// the generator has no lowering for — CRegKey, CScopedSetObjectWithSite, and
// `static_cast<IContextMenu*>(this)` — so the model had to invent all three at once and
// could not. The first two now exist as real RAII types (atl.rs / shellutils.rs).
//
// The cast is the file's own QueryInterface answer for IID_IContextMenu: CDefaultContextMenu
// declares IContextMenu3_iface first, and IContextMenu3 : IContextMenu2 : IContextMenu is a
// single-inheritance chain, so the IContextMenu subobject IS that field's address. Spelling
// it with addr_of_mut! rather than `this as *mut IContextMenu` keeps it correct if a field
// is ever added ahead of the interface.
pub unsafe fn CDefaultContextMenu_InvokeRegVerb_body(this: *mut CDefaultContextMenu, lpcmi: LPCMINVOKECOMMANDINFOEX) -> HRESULT {
    let iCmd = LOWORD!((*lpcmi).lpVerb) as INT;
    let mut hr: HRESULT;
    let mut i: UINT;

    let it = (*this).m_StaticEntries.FindIndex(iCmd);

    if it.is_null() {
        return HRESULT_FROM_WIN32(ERROR_NO_ASSOCIATION as LONG);
    }

    let pEntry: PStaticShellEntry = (*this).m_StaticEntries.GetAt(it);

    let mut VerbKey = CRegKey::new();
    // C: WCHAR VerbKeyPath[sizeof("shell\\") + MAX_VERB] — sizeof of the NARROW literal is 7
    // BYTES (6 characters + NUL), and it is added to a WCHAR count. Reproduce the C's element
    // count exactly; "fixing" it to 6 or to a byte length changes the buffer.
    let mut VerbKeyPath: [WCHAR; 7 + MAX_VERB as usize] = [0; 7 + MAX_VERB as usize];
    hr = StringCbPrintfW!(
        VerbKeyPath.as_mut_ptr(),
        core::mem::size_of_val(&VerbKeyPath),
        CPP_COM_WSTR_A131D6DE6E8D.as_ptr(),
        (*pEntry).Verb.GetString()
    );
    if hr >= 0
        && !(*this).m_pDataObj.is_null()
        && VerbKey.Open((*pEntry).hkClass, VerbKeyPath.as_ptr(), KEY_READ) == ERROR_SUCCESS as i32
    {
        let mut clsid: CLSID = core::mem::zeroed();

        let mut KeyState: DWORD = 0;
        if (*lpcmi).fMask & CMIC_MASK_SHIFT_DOWN != 0 {
            KeyState |= MK_SHIFT;
        }
        if (*lpcmi).fMask & CMIC_MASK_CONTROL_DOWN != 0 {
            KeyState |= MK_CONTROL;
        }

        let mut pPtl: *mut POINTL = core::ptr::null_mut();
        // C_ASSERT(sizeof(POINT) == sizeof(POINTL)) — the cast below reinterprets one as the
        // other, so keep the check that licenses it.
        const _: () = assert!(core::mem::size_of::<POINT>() == core::mem::size_of::<POINTL>());
        if (*lpcmi).fMask & CMIC_MASK_PTINVOKE != 0 {
            pPtl = core::ptr::addr_of!((*lpcmi).ptInvoke) as *mut POINTL;
        }

        let mut pEC: *mut IExecuteCommand = core::ptr::null_mut();
        let _pEC_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pEC));
        hr = SHELL_GetRegCLSID_body(
            VerbKey.m_hKey,
            CPP_COM_WSTR_0C71B3CB5580.as_ptr(),
            CPP_COM_WSTR_0EB6EACBADB7.as_ptr(),
            &mut clsid,
        );
        if hr >= 0 {
            hr = CoCreateInstance(
                &clsid,
                core::ptr::null_mut(),
                CLSCTX_ALL,
                core::ptr::addr_of!(IID_IExecuteCommand).cast(),
                core::ptr::addr_of_mut!(pEC).cast(),
            );
        }
        if hr >= 0 {
            let mut pPB: *mut IPropertyBag = core::ptr::null_mut();
            let _pPB_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pPB));
            SHCreatePropertyBagOnRegKey(
                VerbKey.m_hKey,
                core::ptr::null(),
                STGM_READ,
                core::ptr::addr_of!(IID_IPropertyBag).cast(),
                core::ptr::addr_of_mut!(pPB).cast(),
            );
            return InvokeIExecuteCommandWithDataObject(
                pEC.cast(),
                (*pEntry).Verb.GetString(),
                pPB.cast(),
                (*this).m_pDataObj.cast(),
                lpcmi,
                core::ptr::addr_of_mut!((*this).IContextMenu3_iface).cast::<IUnknown>().cast(),
            );
        }

        let mut pDT: *mut IDropTarget = core::ptr::null_mut();
        let _pDT_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pDT));
        hr = SHELL_GetRegCLSID_body(
            VerbKey.m_hKey,
            CPP_COM_WSTR_B044B339A54F.as_ptr(),
            CPP_COM_WSTR_AFF3436D7481.as_ptr(),
            &mut clsid,
        );
        if hr >= 0 {
            hr = CoCreateInstance(
                &clsid,
                core::ptr::null_mut(),
                CLSCTX_ALL,
                core::ptr::addr_of!(IID_IDropTarget).cast(),
                core::ptr::addr_of_mut!(pDT).cast(),
            );
        }
        if hr >= 0 {
            // Declared before pPB so it drops AFTER it, matching the C++ destruction order.
            let _site = CScopedSetObjectWithSite::new(
                pDT.cast::<IUnknown>().cast(),
                core::ptr::addr_of_mut!((*this).IContextMenu3_iface).cast::<IUnknown>().cast(),
            );
            let mut pPB: *mut IPropertyBag = core::ptr::null_mut();
            let _pPB_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pPB));
            SHCreatePropertyBagOnRegKey(
                VerbKey.m_hKey,
                core::ptr::null(),
                STGM_READ,
                core::ptr::addr_of!(IID_IPropertyBag).cast(),
                core::ptr::addr_of_mut!(pPB).cast(),
            );
            IUnknown_InitializeCommand(pDT.cast::<IUnknown>().cast(), (*pEntry).Verb.GetString(), pPB.cast());
            hr = SHSimulateDrop(pDT.cast(), (*this).m_pDataObj.cast(), KeyState, pPtl, core::ptr::null_mut());
            return hr;
        }
    }

    /* Get the browse flags to see if we need to browse */
    let mut wFlags: DWORD = CDefaultContextMenu_BrowserFlagsFromVerb_body(this, lpcmi, pEntry);

    // A `for` in the C, but the body assigns the induction variable (`i = m_cidl`) to end the
    // loop early, which Rust's `for` cannot express. `continue` in C runs the increment, so the
    // early-exit assignment is followed by `i += 1` here too.
    i = 0;
    while i < (*this).m_cidl {
        /* Check if we need to browse */
        if wFlags != 0 {
            hr = CDefaultContextMenu_TryToBrowse_body(
                this,
                lpcmi,
                *(*this).m_apidl.add(i as usize),
                wFlags,
            );
            if hr >= 0 {
                /* In WinXP if we have browsed, we don't open any more folders.
                 * In Win7 we browse to the first folder we find and
                 * open new windows for each of the rest of the folders */
                let ntver: UINT = RosGetProcessEffectiveVersion();
                if ntver >= _WIN32_WINNT_VISTA {
                    wFlags = 0; // FIXME: = SBSP_NEWBROWSER | (wFlags & ~SBSP_SAMEBROWSER);
                } else {
                    i = (*this).m_cidl;
                }

                i += 1;
                continue;
            }
        }

        CDefaultContextMenu_InvokePidl_body(this, lpcmi, *(*this).m_apidl.add(i as usize), pEntry);
        i += 1;
    }

    S_OK
}

pub unsafe fn CDefaultContextMenu__DoInvokeCommandCallback_body(this: *mut CDefaultContextMenu, lpcmi: LPCMINVOKECOMMANDINFOEX, CmdId: WPARAM) -> HRESULT {
    let Unicode: BOOL = IsUnicode(lpcmi) as BOOL;
    let mut lParamBuf: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut lParam: LPARAM = 0;

    if Unicode != 0 && !(*lpcmi).lpParametersW.is_null() {
        lParam = (*lpcmi).lpParametersW as LPARAM;
    } else if !(*lpcmi).lpParameters.is_null() {
        lParam = if SHAnsiToUnicode((*lpcmi).lpParameters, lParamBuf.as_mut_ptr(), lParamBuf.len() as i32) != 0 {
            lParamBuf.as_ptr() as LPARAM
        } else {
            0
        };
    }

    let hr: HRESULT = CDefaultContextMenu__DoCallback_body(this, DFM_INVOKECOMMAND, CmdId, lParam as *mut core::ffi::c_void);
    hr
}

pub unsafe fn CDefaultContextMenu_InvokeCommand_body(this: *mut CDefaultContextMenu, lpcmi: LPCMINVOKECOMMANDINFO) -> HRESULT {
    let mut LocalInvokeInfo: CMINVOKECOMMANDINFOEX = core::mem::zeroed();
    let Result: HRESULT;
    let mut CmdId: UINT = 0;

    memcpy(
        &mut LocalInvokeInfo as *mut _ as *mut core::ffi::c_void,
        lpcmi as *const _ as *const core::ffi::c_void,
        min(
            core::mem::size_of::<CMINVOKECOMMANDINFOEX>(),
            (*lpcmi).cbSize as usize,
        ),
    );

    if !IS_INTRESOURCE(LocalInvokeInfo.lpVerb) {
        if CDefaultContextMenu_MapVerbToCmdId_body(this, LocalInvokeInfo.lpVerb as LPVOID,
            &mut CmdId as *mut UINT,
            FALSE,
        ) != 0
        {
            LocalInvokeInfo.lpVerb = MAKEINTRESOURCEA(CmdId) as *const i8;
        } else {
            return CDefaultContextMenu_GetNoAssocError();
        }
    }
    CmdId = LOWORD!(LocalInvokeInfo.lpVerb as usize);

    if !(*this).m_DynamicEntries.IsEmpty()
        && CmdId >= (*this).m_iIdSHEFirst
        && CmdId < (*this).m_iIdSHELast
    {
        LocalInvokeInfo.lpVerb = (LocalInvokeInfo.lpVerb as usize - (*this).m_iIdSHEFirst as usize)
            as *const i8;
        Result = CDefaultContextMenu_InvokeShellExt_body(
            this,
            &LocalInvokeInfo as *const CMINVOKECOMMANDINFOEX as LPCMINVOKECOMMANDINFOEX,
        );
        return Result;
    }

    if !(*this).m_StaticEntries.IsEmpty()
        && CmdId >= (*this).m_iIdSCMFirst
        && CmdId < (*this).m_iIdSCMLast
    {
        LocalInvokeInfo.lpVerb = (LocalInvokeInfo.lpVerb as usize - (*this).m_iIdSCMFirst as usize)
            as *const i8;
        Result = CDefaultContextMenu_InvokeRegVerb_body(
            this,
            &LocalInvokeInfo as *const CMINVOKECOMMANDINFOEX as LPCMINVOKECOMMANDINFOEX,
        );
        return Result;
    }

    if (*this).m_iIdCBFirst != (*this).m_iIdCBLast
        && CmdId >= (*this).m_iIdCBFirst
        && CmdId < (*this).m_iIdCBLast
    {
        Result = CDefaultContextMenu__DoInvokeCommandCallback_body(
            this,
            &LocalInvokeInfo as *const CMINVOKECOMMANDINFOEX as LPCMINVOKECOMMANDINFOEX,
            (CmdId - (*this).m_iIdCBFirst) as WPARAM,
        );
        return Result;
    }

    if (*this).m_iIdDfltFirst != (*this).m_iIdDfltLast
        && CmdId >= (*this).m_iIdDfltFirst
        && CmdId < (*this).m_iIdDfltLast
    {
        CmdId -= (*this).m_iIdDfltFirst;
        CmdId += DCM_FCIDM_SHVIEW_OFFSET;
    }

    if LocalInvokeInfo.cbSize >= core::mem::size_of::<CMINVOKECOMMANDINFOEX>() as u32
        && (LocalInvokeInfo.fMask & CMIC_MASK_PTINVOKE) != 0
    {
        if !(*this).m_pDataObj.is_null()
            && FAILED_UNEXPECTEDLY!(DataObject_SetOffset(
                (*this).m_pDataObj,
                &mut LocalInvokeInfo.ptInvoke as *mut POINT,
            ))
        {}
    }

    Result = match CmdId {
        FCIDM_SHVIEW_INSERT => CDefaultContextMenu_DoPaste_body(
            this,
            &LocalInvokeInfo as *const CMINVOKECOMMANDINFOEX as LPCMINVOKECOMMANDINFOEX,
            FALSE,
        ),
        FCIDM_SHVIEW_INSERTLINK => CDefaultContextMenu_DoPaste_body(
            this,
            &LocalInvokeInfo as *const CMINVOKECOMMANDINFOEX as LPCMINVOKECOMMANDINFOEX,
            TRUE,
        ),
        FCIDM_SHVIEW_COPY | FCIDM_SHVIEW_CUT => CDefaultContextMenu_DoCopyOrCut_body(
            this,
            &LocalInvokeInfo as *const CMINVOKECOMMANDINFOEX as LPCMINVOKECOMMANDINFOEX,
            if CmdId == FCIDM_SHVIEW_COPY {
                1
            } else {
                0
            },
        ),
        FCIDM_SHVIEW_CREATELINK => CDefaultContextMenu_DoCreateLink_body(
            this,
            &LocalInvokeInfo as *const CMINVOKECOMMANDINFOEX as LPCMINVOKECOMMANDINFOEX,
        ),
        FCIDM_SHVIEW_DELETE => CDefaultContextMenu_DoDelete_body(
            this,
            &LocalInvokeInfo as *const CMINVOKECOMMANDINFOEX as LPCMINVOKECOMMANDINFOEX,
        ),
        FCIDM_SHVIEW_RENAME => CDefaultContextMenu_DoRename_body(
            this,
            &LocalInvokeInfo as *const CMINVOKECOMMANDINFOEX as LPCMINVOKECOMMANDINFOEX,
        ),
        FCIDM_SHVIEW_PROPERTIES => CDefaultContextMenu_DoProperties_body(
            this,
            &LocalInvokeInfo as *const CMINVOKECOMMANDINFOEX as LPCMINVOKECOMMANDINFOEX,
        ),
        FCIDM_SHVIEW_NEWFOLDER => CDefaultContextMenu_DoCreateNewFolder_body(
            this,
            &LocalInvokeInfo as *const CMINVOKECOMMANDINFOEX as LPCMINVOKECOMMANDINFOEX,
        ),
        FCIDM_SHVIEW_COPYTO => CDefaultContextMenu_DoCopyToMoveToFolder_body(
            this,
            &LocalInvokeInfo as *const CMINVOKECOMMANDINFOEX as LPCMINVOKECOMMANDINFOEX,
            TRUE,
        ),
        FCIDM_SHVIEW_MOVETO => CDefaultContextMenu_DoCopyToMoveToFolder_body(
            this,
            &LocalInvokeInfo as *const CMINVOKECOMMANDINFOEX as LPCMINVOKECOMMANDINFOEX,
            FALSE,
        ),
        FCIDM_SHVIEW_UNDO => CDefaultContextMenu_DoUndo_body(
            this,
            &LocalInvokeInfo as *const CMINVOKECOMMANDINFOEX as LPCMINVOKECOMMANDINFOEX,
        ),
        _ => E_INVALIDARG,
    };

    Result
}

pub unsafe fn CDefaultContextMenu_GetCommandString_body(this: *mut CDefaultContextMenu, idCommand: UINT_PTR, uFlags: UINT, lpReserved: *mut UINT, lpszName: LPSTR, uMaxNameLen: UINT) -> HRESULT {
    if uFlags == GCS_HELPTEXTA
        || uFlags == GCS_HELPTEXTW
        || HIWORD(idCommand as DWORD) != 0
    {
        return E_NOTIMPL;
    }

    let mut CmdId = LOWORD!(idCommand);

    if !(*this).m_DynamicEntries.IsEmpty()
        && CmdId >= (*this).m_iIdSHEFirst
        && CmdId < (*this).m_iIdSHELast
    {
        let mut idCommand = idCommand;
        idCommand -= (*this).m_iIdSHEFirst as UINT_PTR;
        let pEntry = CDefaultContextMenu_GetDynamicEntry_body(this, idCommand as UINT);
        if pEntry.is_null() {
            return E_FAIL;
        }

        idCommand -= (*pEntry).iIdCmdFirst as UINT_PTR;
        return ((*(*(*(*pEntry).pCM)).lpVtbl).GetCommandString)(
            *(*pEntry).pCM,
            idCommand,
            uFlags,
            lpReserved,
            lpszName,
            uMaxNameLen,
        );
    }

    if !(*this).m_StaticEntries.IsEmpty()
        && CmdId >= (*this).m_iIdSCMFirst
        && CmdId < (*this).m_iIdSCMLast
    {
        if uFlags == GCS_VALIDATEA || uFlags == GCS_VALIDATEW {
            return S_OK;
        }

        CmdId -= (*this).m_iIdSCMFirst;

        let it = (*this).m_StaticEntries.FindIndex(CmdId as i32);

        if it.is_null() {
            return E_INVALIDARG;
        }

        let pEntry = (*this).m_StaticEntries.GetAt(it);

        if uFlags == GCS_VERBW {
            return StringCchCopyW(
                lpszName as LPWSTR,
                uMaxNameLen as usize,
                (*pEntry).Verb.GetString(),
            );
        }

        if uFlags == GCS_VERBA {
            if SHUnicodeToAnsi((*pEntry).Verb.GetString(), lpszName, uMaxNameLen as i32) != 0 {
                return S_OK;
            }
        }

        return E_INVALIDARG;
    }

    if (*this).m_iIdDfltFirst != (*this).m_iIdDfltLast
        && CmdId >= (*this).m_iIdDfltFirst
        && CmdId < (*this).m_iIdDfltLast
    {
        CmdId -= (*this).m_iIdDfltFirst;
        CmdId += DCM_FCIDM_SHVIEW_OFFSET;
    }

    let mut i = 0;
    while i < g_StaticInvokeCmdMap.len() {
        if u32::from(g_StaticInvokeCmdMap[i].IntVerb) == CmdId {
            if uFlags == GCS_VALIDATEA || uFlags == GCS_VALIDATEW {
                return S_OK;
            }

            if uFlags == GCS_VERBA {
                return StringCchCopyA(
                    lpszName,
                    uMaxNameLen as usize,
                    g_StaticInvokeCmdMap[i].szStringVerb,
                );
            }

            if uFlags == GCS_VERBW {
                if 0 != SHAnsiToUnicode(
                    g_StaticInvokeCmdMap[i].szStringVerb,
                    lpszName as LPWSTR,
                    uMaxNameLen as i32,
                ) {
                    return S_OK;
                }
            }
        }
        i += 1;
    }

    E_INVALIDARG
}

pub unsafe fn CDefaultContextMenu_HandleMenuMsg_body(this: *mut CDefaultContextMenu, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> HRESULT {
    return CDefaultContextMenu_HandleMenuMsg2_body(this, uMsg, wParam, lParam, core::ptr::null_mut());
}

pub unsafe fn CDefaultContextMenu_HandleMenuMsg2_body(this: *mut CDefaultContextMenu, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, plResult: *mut LRESULT) -> HRESULT {
    if SHELL_IsContextMenuMsg(uMsg) == 0 {
        return E_FAIL;
    }

    let CmdId: UINT;
    if uMsg == WM_INITMENUPOPUP {
        CmdId = GetMenuItemID(wParam as HMENU, 0);
        if CmdId == !0u32 {
            return E_FAIL;
        }
    } else {
        let mut temp_CmdId: UINT = 0;
        let hr = SHGetMenuIdFromMenuMsg(uMsg, lParam, &mut temp_CmdId);
        if hr < 0 {
            return S_FALSE;
        }
        CmdId = temp_CmdId;
    }

    let CmdId = CmdId.wrapping_sub((*this).m_iIdQCMFirst);

    if CmdId >= (*this).m_iIdSHEFirst && CmdId < (*this).m_iIdSHELast {
        let pEntry = CDefaultContextMenu_GetDynamicEntry_body(this, CmdId - (*this).m_iIdSHEFirst);
        if !pEntry.is_null() {
            return SHForwardContextMenuMsg((*pEntry).pCM.p.cast(), uMsg, wParam, lParam, plResult, TRUE);
        }
    }

    E_FAIL
}

pub unsafe fn CDefaultContextMenu_SetSite_body(this: *mut CDefaultContextMenu, pUnkSite: *mut IUnknown) -> HRESULT {
    CDefaultContextMenu_cpp_com_assign_m_site(this, pUnkSite);
    return S_OK;
}

pub unsafe fn CDefaultContextMenu_GetSite_body(this: *mut CDefaultContextMenu, riid: REFIID, ppvSite: *mut PVOID) -> HRESULT {
    if (*this).m_site.is_null() {
        return E_FAIL;
    }

    ((*(*(*this).m_site).lpVtbl).QueryInterface)((*this).m_site, riid, ppvSite)
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     shell32
//  * LICENSE:     GPL - See COPYING in the top level directory
//  * FILE:        dll/win32/shell32/shv_item_new.c
//  * PURPOSE:     provides default context menu implementation
//  * PROGRAMMERS: Johannes Anderwald (johannes.anderwald@reactos.org)
//  */
//
// #include "precomp.h"
// #include <compat_undoc.h>
//
// WINE_DEFAULT_DEBUG_CHANNEL(dmenu);
//
// // FIXME: 260 is correct, but should this be part of the SDK or just MAX_PATH?
// #define MAX_VERB 260
//
// static HRESULT
// SHELL_GetRegCLSID(HKEY hKey, LPCWSTR SubKey, LPCWSTR Value, CLSID &clsid)
// {
//     WCHAR buf[42];
//     DWORD cb = sizeof(buf);
//     DWORD err = RegGetValueW(hKey, SubKey, Value, RRF_RT_REG_SZ, NULL, buf, &cb);
//     return !err ? CLSIDFromString(buf, &clsid) : HRESULT_FROM_WIN32(err);
// }
// static BOOL InsertMenuItemAt(HMENU hMenu, UINT Pos, UINT Flags)
// {
//     MENUITEMINFOW mii;
//     mii.cbSize = FIELD_OFFSET!(MENUITEMINFOW, hbmpItem) as UINT; // USER32 version agnostic
//     mii.fMask = MIIM_TYPE;
//     mii.fType = Flags;
//     return InsertMenuItemW(hMenu, Pos, TRUE, &mii);
// }
// static HRESULT
// MapVerbToCmdId(PVOID Verb, BOOL IsUnicode, IContextMenu *pCM, UINT idFirst, UINT idLast)
// {
//     const UINT gcs = IsUnicode ? GCS_VERBW : GCS_VERBA;
//     for (UINT id = idFirst; id <= idLast; ++id)
//     {
//         WCHAR buf[MAX_PATH];
//         *buf = UNICODE_NULL;
//         HRESULT hr = pCM->GetCommandString(id, gcs, NULL, (LPSTR)buf, _countof(buf));
//         if (FAILED(hr) || !*buf)
//             continue;
//         else if (IsUnicode && !_wcsicmp((LPWSTR)Verb, buf))
//             return id;
//         else if (!IsUnicode && !lstrcmpiA((LPSTR)Verb, (LPSTR)buf))
//             return id;
//     }
//     return HRESULT_FROM_WIN32(ERROR_NOT_FOUND);
// }
// static inline bool IsVerbListSeparator(WCHAR Ch)
// {
//     return Ch == L' ' || Ch == L','; // learn.microsoft.com/en-us/windows/win32/shell/context-menu-handlers
// }
// static int FindVerbInDefaultVerbList(LPCWSTR List, LPCWSTR Verb)
// {
//     for (UINT index = 0; *List; ++index)
//     {
//         while (IsVerbListSeparator(*List))
//             List++;
//         LPCWSTR Start = List;
//         while (*List && !IsVerbListSeparator(*List))
//             List++;
//         // "List > Start" to verify that the list item is non-empty to avoid the edge case where Verb is "" and the list contains ",,"
//         if (!_wcsnicmp(Verb, Start, List - Start) && List > Start)
//             return index;
//     }
//     return -1;
// }
// static HRESULT GetFriendlyVerb( PCWSTR pszVerb,  PWSTR pszBuf,  SIZE_T cchMax)
// {
//     static const struct { PCWSTR pszVerb; WORD iResId; } map[] =
//     {
//         // { L"open", IDS_OPEN_VERB }, These two have already been handled
//         // { L"explore", IDS_EXPLORE_VERB },
//         { L"edit", IDS_EDIT_VERB },
//         { L"print", IDS_PRINT_VERB },
//         { L"runas", IDS_RUNAS_VERB },
//         { L"openas", IDS_OPEN_VERB },
//         { L"find", IDS_FIND_VERB },
//     };
//     for (SIZE_T i = 0; i < _countof(map); ++i)
//     {
//         if (!_wcsicmp(pszVerb, map[i].pszVerb) &&
//             LoadStringW(shell32_hInstance, map[i].iResId, pszBuf, cchMax))
//         {
//             return S_OK;
//         }
//     }
//
//     // Try to make a friendly verb based on the verb subkey
//     if (pszVerb[0] < 127 && !StrChrW(pszVerb, '&') && SUCCEEDED(StringCchCopyW(pszBuf + 1, --cchMax, pszVerb)))
//     {
//         *pszBuf = L'&';
//         return S_OK; // This can be changed to S_FALSE if the caller needs to know we faked it
//     }
//     return E_FAIL;
// }
//         HRESULT QueryService(REFGUID svc, REFIID riid, void**ppv) {
//             return IUnknown_QueryService(m_site, svc, riid, ppv);
//         }
// HRESULT WINAPI CDefaultContextMenu::Initialize(const DEFCONTEXTMENU *pdcm, LPFNDFMCALLBACK lpfn)
// {
//     TRACE("cidl %u\n", pdcm->cidl);
//
//     HRESULT hr = S_OK;
//     if (!pdcm->pcmcb && !lpfn)
//     {
//         ERR("CDefaultContextMenu needs a callback!\n");
//         return E_INVALIDARG;
//     }
//
//     m_cidl = pdcm->cidl;
//     m_apidl = const_cast<PCUITEMID_CHILD_ARRAY>(_ILCopyaPidl(pdcm->apidl, m_cidl));
//     if (m_cidl && !m_apidl)
//         return E_OUTOFMEMORY;
//     m_psf = pdcm->psf;
//     m_pmcb = pdcm->pcmcb;
//     m_pfnmcb = lpfn;
//     m_hwnd = pdcm->hwnd;
//
//     for (UINT i = 0; i < pdcm->cKeys; ++i)
//     {
//         if (i >= _countof(m_aKeys))
//             hr = E_INVALIDARG;
//         else if ((m_aKeys[i] = SHRegDuplicateHKey(pdcm->aKeys[i])) != NULL)
//             m_cKeys++;
//         else
//             hr = E_OUTOFMEMORY;
//     }
//
//     m_psf->GetUIObjectOf(pdcm->hwnd, m_cidl, m_apidl, IID_NULL_PPV_ARG(IDataObject, &m_pDataObj));
//
//     if (pdcm->pidlFolder)
//     {
//         m_pidlFolder = ILClone(pdcm->pidlFolder);
//     }
//     else
//     {
//         CComPtr<IPersistFolder2> pf = NULL;
//         if (SUCCEEDED(m_psf->QueryInterface(IID_PPV_ARG(IPersistFolder2, &pf))))
//         {
//             if (FAILED(pf->GetCurFolder(&m_pidlFolder)))
//                 ERR("GetCurFolder failed\n");
//         }
//         TRACE("pidlFolder %p\n", m_pidlFolder);
//     }
//
//     return hr;
// }
// HRESULT CDefaultContextMenu::_DoCallback(UINT uMsg, WPARAM wParam, LPVOID lParam)
// {
//     if (m_pmcb)
//     {
//         return m_pmcb->CallBack(m_psf, m_hwnd, m_pDataObj, uMsg, wParam, (LPARAM)lParam);
//     }
//     else if(m_pfnmcb)
//     {
//         return ((*this).m_pfnmcb.unwrap())(m_psf, m_hwnd, m_pDataObj, uMsg, wParam, (LPARAM)lParam);
//     }
//
//     return E_FAIL;
// }
// void CDefaultContextMenu::AddStaticEntry(const HKEY hkClass, const WCHAR *szVerb, UINT uFlags)
// {
//     POSITION it = m_StaticEntries.GetHeadPosition();
//     while (it != NULL)
//     {
//         const StaticShellEntry& info = m_StaticEntries.GetNext(it);
//         if (info.Verb.CompareNoCase(szVerb) == 0)
//         {
//             /* entry already exists */
//             return;
//         }
//     }
//
//     TRACE("adding verb %s\n", debugstr_w(szVerb));
//
//     if (!_wcsicmp(szVerb, L"open") && !(uFlags & CMF_NODEFAULT))
//     {
//         /* open verb is always inserted in front */
//         m_StaticEntries.AddHead({ szVerb, hkeyClass });
//     }
//     else
//     {
//         m_StaticEntries.AddTail({ szVerb, hkeyClass });
//     }
// }
// void CDefaultContextMenu::AddStaticEntriesForKey(HKEY hKey, UINT uFlags)
// {
//     WCHAR wszName[VERBKEY_CCHMAX];
//     DWORD cchName, dwIndex = 0;
//     HKEY hShellKey;
//
//     LRESULT lres = RegOpenKeyExW(hKey, L"shell", 0, KEY_READ, &hShellKey);
//     if (lres != STATUS_SUCCESS)
//         return;
//
//     if (!*m_DefVerbs)
//     {
//         DWORD cb = sizeof(m_DefVerbs);
//         RegGetValueW(hShellKey, NULL, NULL, RRF_RT_REG_SZ, NULL, m_DefVerbs, &cb);
//     }
//
//     while(TRUE)
//     {
//         cchName = _countof(wszName);
//         if (RegEnumKeyExW(hShellKey, dwIndex++, wszName, &cchName, NULL, NULL, NULL, NULL) != ERROR_SUCCESS)
//             break;
//
//         AddStaticEntry(hKey, wszName, uFlags);
//     }
//
//     RegCloseKey(hShellKey);
// }
// static
// BOOL
// HasClipboardData()
// {
//     BOOL bRet = FALSE;
//     CComPtr<IDataObject> pDataObj;
//
//     if (SUCCEEDED(OleGetClipboard(&pDataObj)))
//     {
//         FORMATETC formatetc;
//
//         TRACE("pDataObj=%p\n", pDataObj.p);
//
//         /* Set the FORMATETC structure*/
//         InitFormatEtc(&mut formatetc, RegisterClipboardFormatW(CFSTR_SHELLIDLIST), TYMED_HGLOBAL);
//         bRet = SUCCEEDED(pDataObj->QueryGetData(&formatetc));
//     }
//
//     return bRet;
// }
// BOOL
// CDefaultContextMenu::IsShellExtensionAlreadyLoaded(REFCLSID clsid)
// {
//     POSITION it = m_DynamicEntries.GetHeadPosition();
//     while (it != NULL)
//     {
//         const DynamicShellEntry& info = m_DynamicEntries.GetNext(it);
//         if (info.ClassID == clsid)
//             return TRUE;
//     }
//
//     return FALSE;
// }
// HRESULT
// CDefaultContextMenu::LoadDynamicContextMenuHandler(HKEY hKey, REFCLSID clsid)
// {
//     HRESULT hr;
//     TRACE("LoadDynamicContextMenuHandler entered with This %p hKey %p pclsid %s\n", this, hKey, wine_dbgstr_guid(&clsid));
//
//     if (IsShellExtensionAlreadyLoaded(clsid))
//         return S_OK;
//
//     CComPtr<IContextMenu> pcm;
//     hr = SHCoCreateInstance(NULL, &clsid, NULL, IID_PPV_ARG(IContextMenu, &pcm));
//     if (FAILED(hr))
//     {
//         ERR("SHCoCreateInstance(IContextMenu) failed.clsid %s hr 0x%x\n", wine_dbgstr_guid(&clsid), hr);
//         return hr;
//     }
//
//     CComPtr<IShellExtInit> pExtInit;
//     hr = pcm->QueryInterface(IID_PPV_ARG(IShellExtInit, &pExtInit));
//     if (FAILED(hr))
//     {
//         ERR("IContextMenu->QueryInterface(IShellExtInit) failed.clsid %s hr 0x%x\n", wine_dbgstr_guid(&clsid), hr);
//         return hr;
//     }
//
//     hr = pExtInit->Initialize(m_pDataObj ? NULL : m_pidlFolder, m_pDataObj, hKey);
//     if (FAILED(hr))
//     {
//         WARN("IShellExtInit::Initialize failed.clsid %s hr 0x%x\n", wine_dbgstr_guid(&clsid), hr);
//         return hr;
//     }
//
//     if (m_site)
//         IUnknown_SetSite(pcm, m_site);
//
//     m_DynamicEntries.AddTail({ 0, 0, clsid, pcm });
//
//     return S_OK;
// }
// BOOL
// CDefaultContextMenu::EnumerateDynamicContextHandlerForKey(HKEY hRootKey)
// {
//     WCHAR wszName[MAX_PATH], wszBuf[MAX_PATH], *pwszClsid;
//     DWORD cchName;
//     HRESULT hr;
//     HKEY hKey;
//
//     if (RegOpenKeyExW(hRootKey, L"shellex\\ContextMenuHandlers", 0, KEY_READ, &hKey) != ERROR_SUCCESS)
//     {
//         TRACE("RegOpenKeyExW failed\n");
//         return FALSE;
//     }
//
//     DWORD dwIndex = 0;
//     while (TRUE)
//     {
//         cchName = _countof(wszName);
//         if (RegEnumKeyExW(hKey, dwIndex++, wszName, &cchName, NULL, NULL, NULL, NULL) != ERROR_SUCCESS)
//             break;
//
//         /* Key name or key value is CLSID */
//         CLSID clsid;
//         hr = CLSIDFromString(wszName, &clsid);
//         if (hr == S_OK)
//             pwszClsid = wszName;
//         else
//         {
//             DWORD cchBuf = _countof(wszBuf);
//             if (RegGetValueW(hKey, wszName, NULL, RRF_RT_REG_SZ, NULL, wszBuf, &cchBuf) == ERROR_SUCCESS)
//                 hr = CLSIDFromString(wszBuf, &clsid);
//             pwszClsid = wszBuf;
//         }
//
//         if (FAILED(hr))
//         {
//             ERR("CLSIDFromString failed for clsid %S hr 0x%x\n", pwszClsid, hr);
//             continue;
//         }
//
//         if (m_bGroupPolicyActive)
//         {
//             if (RegGetValueW(HKEY_LOCAL_MACHINE,
//                              L"Software\\Microsoft\\Windows\\CurrentVersion\\Shell Extensions\\Approved",
//                              pwszClsid,
//                              RRF_RT_REG_SZ,
//                              NULL,
//                              NULL,
//                              NULL) != ERROR_SUCCESS)
//             {
//                 ERR("Shell extension %s not approved!\n", pwszClsid);
//                 continue;
//             }
//         }
//
//         hr = LoadDynamicContextMenuHandler(hRootKey, clsid);
//         if (FAILED(hr))
//             WARN("Failed to get context menu entires from shell extension! clsid: %S\n", pwszClsid);
//     }
//
//     RegCloseKey(hKey);
//     return TRUE;
// }
// UINT
// CDefaultContextMenu::AddShellExtensionsToMenu(HMENU hMenu, UINT* pIndexMenu, UINT idCmdFirst, UINT idCmdLast, UINT uFlags)
// {
//     UINT cIds = 0;
//
//     if (m_DynamicEntries.IsEmpty())
//         return cIds;
//
//     POSITION it = m_DynamicEntries.GetHeadPosition();
//     while (it != NULL)
//     {
//         DynamicShellEntry& info = m_DynamicEntries.GetNext(it);
//
//         HRESULT hr = info.pCM->QueryContextMenu(hMenu, *pIndexMenu, idCmdFirst + cIds, idCmdLast, uFlags);
//         if (SUCCEEDED(hr))
//         {
//             info.iIdCmdFirst = cIds;
//             info.NumIds = HRESULT_CODE(hr);
//             (*pIndexMenu) += info.NumIds;
//
//             cIds += info.NumIds;
//             if (idCmdFirst + cIds >= idCmdLast)
//                 break;
//         }
//         TRACE("pEntry hr %x contextmenu %p cmdfirst %x num ids %x\n", hr, info.pCM.p, info.iIdCmdFirst, info.NumIds);
//     }
//     return cIds;
// }
// UINT
// CDefaultContextMenu::AddStaticContextMenusToMenu(
//     HMENU hMenu,
//     UINT* pIndexMenu,
//     UINT iIdCmdFirst,
//     UINT iIdCmdLast,
//     UINT uFlags)
// {
//     UINT ntver = RosGetProcessEffectiveVersion();
//     MENUITEMINFOW mii = { sizeof(mii) };
//     WCHAR wszDispVerb[80]; // The limit on XP. If the friendly string is longer, it falls back to the verb key.
//     UINT fState, idVerbRes;
//     UINT cIds = 0, indexFirst = *pIndexMenu, indexDefault;
//     int iDefVerbIndex = -1;
//
//     mii.fMask = MIIM_ID | MIIM_TYPE | MIIM_STATE | MIIM_DATA;
//     mii.fType = MFT_STRING;
//
//     POSITION it = m_StaticEntries.GetHeadPosition();
//     bool first = true;
//     while (it != NULL)
//     {
//         StaticShellEntry& info = m_StaticEntries.GetNext(it);
//         BOOL forceFirstPos = FALSE;
//         bool hide = false;
//
//         fState = MFS_ENABLED;
//
//         /* set first entry as default */
//         if (first)
//         {
//             fState |= MFS_DEFAULT;
//             first = false;
//         }
//
//         if (info.Verb.CompareNoCase(L"open") == 0)
//         {
//             idVerbRes = IDS_OPEN_VERB;
//             fState |= MFS_DEFAULT; /* override default when open verb is found */
//             forceFirstPos++;
//         }
//         else if (info.Verb.CompareNoCase(L"explore") == 0)
//         {
//             idVerbRes = IDS_EXPLORE_VERB;
//             if (uFlags & CMF_EXPLORE)
//             {
//                 fState |= MFS_DEFAULT;
//                 forceFirstPos++;
//             }
//         }
//         else if (info.Verb.CompareNoCase(L"printto") == 0)
//             hide = true;
//         else
//             idVerbRes = 0;
//
//         /* By default use verb for menu item name */
//         mii.dwTypeData = (LPWSTR)info.Verb.GetString();
//
//         WCHAR wszKey[sizeof("shell\\") + MAX_VERB];
//         HRESULT hr = StringCbPrintfW(wszKey, sizeof(wszKey), L"shell\\%s", info.Verb.GetString());
//         if (FAILED_UNEXPECTEDLY(hr))
//             hide = true;
//
//         UINT cmdFlags = 0;
//         HKEY hkVerb;
//         if (RegOpenKeyExW(info.hkClass, wszKey, 0, KEY_READ, &hkVerb) == ERROR_SUCCESS)
//         {
//             if (!(uFlags & CMF_OPTIMIZEFORINVOKE))
//             {
//                 DWORD cbVerb = sizeof(wszDispVerb);
//                 LONG res = RegLoadMUIStringW(hkVerb, L"MUIVerb", wszDispVerb, cbVerb, NULL, 0, NULL);
//                 if (res || !*wszDispVerb)
//                     res = RegLoadMUIStringW(hkVerb, NULL, wszDispVerb, cbVerb, NULL, 0, NULL);
//
//                 if ((res == ERROR_SUCCESS && *wszDispVerb) ||
//                     (idVerbRes && LoadStringW(shell32_hInstance, idVerbRes, wszDispVerb, _countof(wszDispVerb))) ||
//                     SUCCEEDED(GetFriendlyVerb(info.Verb, wszDispVerb, _countof(wszDispVerb))))
//                 {
//                     mii.dwTypeData = wszDispVerb;
//                 }
//             }
//         }
//         else
//         {
//             hkVerb = NULL;
//         }
//
//         if (hkVerb)
//         {
//             if (!hide && !(uFlags & CMF_EXTENDEDVERBS))
//                 hide = RegValueExists(hkVerb, L"Extended");
//
//             if (!hide)
//                 hide = RegValueExists(hkVerb, L"ProgrammaticAccessOnly");
//
//             if (!hide && !(uFlags & CMF_DISABLEDVERBS))
//                 hide = RegValueExists(hkVerb, L"LegacyDisable");
//
//             if (DWORD dwRest = (hide ? 0 : RegGetDword(hkVerb, NULL, L"SuppressionPolicy", 0)))
//                 hide = SHRestricted((RESTRICTIONS)dwRest);
//
//             if (RegValueExists(hkVerb, L"NeverDefault"))
//                 fState &= ~MFS_DEFAULT;
//
//             if (RegValueExists(hkVerb, L"SeparatorBefore"))
//                 cmdFlags |= ECF_SEPARATORBEFORE;
//             if (RegValueExists(hkVerb, L"SeparatorAfter"))
//                 cmdFlags |= ECF_SEPARATORAFTER;
//
//             RegCloseKey(hkVerb);
//         }
//
//         if (((uFlags & CMF_NODEFAULT) && ntver >= _WIN32_WINNT_VISTA) ||
//             ((uFlags & CMF_DONOTPICKDEFAULT) && ntver >= _WIN32_WINNT_WIN7))
//         {
//             fState &= ~MFS_DEFAULT;
//         }
//
//         if (!hide)
//         {
//             if (cmdFlags & ECF_SEPARATORBEFORE)
//             {
//                 if (InsertMenuItemAt(hMenu, *pIndexMenu, MF_SEPARATOR))
//                     (*pIndexMenu)++;
//             }
//
//             UINT pos = *pIndexMenu;
//             int verbIndex = hkVerb ? FindVerbInDefaultVerbList(m_DefVerbs, info.Verb) : -1;
//             if (verbIndex >= 0)
//             {
//                 if (verbIndex < iDefVerbIndex || iDefVerbIndex < 0)
//                 {
//                     iDefVerbIndex = verbIndex;
//                     fState |= MFS_DEFAULT;
//                     forceFirstPos = TRUE;
//                 }
//                 else
//                 {
//                     fState &= ~MFS_DEFAULT; // We have already set a better default
//                     pos = indexDefault;
//                 }
//             }
//             else if (iDefVerbIndex >= 0)
//             {
//                 fState &= ~MFS_DEFAULT; // We have already set the default
//                 if (forceFirstPos)
//                     pos = indexDefault;
//                 forceFirstPos = FALSE;
//             }
//
//             mii.fState = fState;
//             mii.wID = iIdCmdFirst + cIds;
//             if (InsertMenuItemW(hMenu, forceFirstPos ? indexFirst : pos, TRUE, &mii))
//                 (*pIndexMenu)++;
//
//             if (cmdFlags & ECF_SEPARATORAFTER)
//             {
//                 if (InsertMenuItemAt(hMenu, *pIndexMenu, MF_SEPARATOR))
//                     (*pIndexMenu)++;
//             }
//
//             if (fState & MFS_DEFAULT)
//                 indexDefault = *pIndexMenu; // This is where we want to insert "high priority" verbs
//         }
//         cIds++; // Always increment the id because it acts as the index into m_StaticEntries
//
//         if (mii.wID >= iIdCmdLast)
//             break;
//     }
//
//     return cIds;
// }
// void
// CDefaultContextMenu::TryPickDefault(HMENU hMenu, UINT idCmdFirst, UINT DfltOffset, UINT uFlags)
// {
//     // Are we allowed to pick a default?
//     if ((uFlags & CMF_NODEFAULT) ||
//         ((uFlags & CMF_DONOTPICKDEFAULT) && RosGetProcessEffectiveVersion() >= _WIN32_WINNT_WIN7))
//     {
//         return;
//     }
//
//     // Do we already have a default?
//     if ((int)GetMenuDefaultItem(hMenu, MF_BYPOSITION, 0) != -1)
//         return;
//
//     // Does the view want to pick one?
//     INT_PTR forceDfm = 0;
//     if (SUCCEEDED(_DoCallback(DFM_GETDEFSTATICID, 0, &forceDfm)) && forceDfm)
//     {
//         for (UINT i = 0; i < _countof(g_StaticInvokeCmdMap); ++i)
//         {
//             UINT menuItemId = g_StaticInvokeCmdMap[i].IntVerb + DfltOffset - DCM_FCIDM_SHVIEW_OFFSET;
//             if (g_StaticInvokeCmdMap[i].DfmCmd == forceDfm &&
//                 SetMenuDefaultItem(hMenu, menuItemId, MF_BYCOMMAND))
//             {
//                 return;
//             }
//         }
//     }
//
//     // Don't want to pick something like cut or delete as the default but
//     // a static or dynamic verb is a good default.
//     if (m_iIdSCMLast > m_iIdSCMFirst || m_iIdSHELast > m_iIdSHEFirst)
//         SetMenuDefaultItem(hMenu, idCmdFirst, MF_BYCOMMAND);
// }
// CDefaultContextMenu::QueryContextMenu(
//     HMENU hMenu,
//     UINT IndexMenu,
//     UINT idCmdFirst,
//     UINT idCmdLast,
//     UINT uFlags)
// {
//     HRESULT hr;
//     UINT idCmdNext = m_iIdQCMFirst = idCmdFirst;
//     UINT cIds = 0;
//
//     TRACE("BuildShellItemContextMenu entered\n");
//
//     /* Load static verbs and shell extensions from registry */
//     for (UINT i = 0; i < m_cKeys && !(uFlags & CMF_NOVERBS); i++)
//     {
//         AddStaticEntriesForKey(m_aKeys[i], uFlags);
//         EnumerateDynamicContextHandlerForKey(m_aKeys[i]);
//     }
//
//     /* Add static context menu handlers */
//     cIds = AddStaticContextMenusToMenu(hMenu, &IndexMenu, idCmdFirst, idCmdLast, uFlags);
//     m_iIdSCMFirst = 0; // FIXME: This should be = idCmdFirst?
//     m_iIdSCMLast = cIds;
//     idCmdNext = idCmdFirst + cIds;
//
//     /* Add dynamic context menu handlers */
//     cIds += AddShellExtensionsToMenu(hMenu, &IndexMenu, idCmdNext, idCmdLast, uFlags);
//     m_iIdSHEFirst = m_iIdSCMLast;
//     m_iIdSHELast = cIds;
//     idCmdNext = idCmdFirst + cIds;
//     TRACE("SH_LoadContextMenuHandlers first %x last %x\n", m_iIdSHEFirst, m_iIdSHELast);
//
//     /* Now let the callback add its own items */
//     QCMINFO qcminfo = {hMenu, IndexMenu, idCmdNext, idCmdLast, NULL};
//     if (SUCCEEDED(_DoCallback(DFM_MERGECONTEXTMENU, uFlags, &qcminfo)))
//     {
//         UINT added = qcminfo.idCmdFirst - idCmdNext;
//         cIds += added;
//         IndexMenu += added;
//         m_iIdCBFirst = m_iIdSHELast;
//         m_iIdCBLast = cIds;
//         idCmdNext = idCmdFirst + cIds;
//     }
//
//     //TODO: DFM_MERGECONTEXTMENU_BOTTOM
//
//     UINT idDefaultOffset = 0;
//     BOOL isBackgroundMenu = !m_cidl;
//     if (!(uFlags & CMF_VERBSONLY) && !isBackgroundMenu)
//     {
//         /* Get the attributes of the items */
//         SFGAOF rfg = SFGAO_BROWSABLE | SFGAO_CANCOPY | SFGAO_CANLINK | SFGAO_CANMOVE | SFGAO_CANDELETE | SFGAO_CANRENAME | SFGAO_HASPROPSHEET | SFGAO_FILESYSTEM | SFGAO_FOLDER;
//         hr = m_psf->GetAttributesOf(m_cidl, m_apidl, &rfg);
//         if (FAILED_UNEXPECTEDLY(hr))
//             return MAKE_HRESULT(SEVERITY_SUCCESS, 0, cIds);
//
//         /* Add the default part of the menu */
//         HMENU hmenuDefault = LoadMenuW(shell32_hInstance, MAKEINTRESOURCEW(MENU_SHV_FILE));
//
//         /* Remove uneeded entries */
//         if (!(rfg & SFGAO_CANMOVE))
//             DeleteMenu(hmenuDefault, IDM_CUT, MF_BYCOMMAND);
//         if (!(rfg & SFGAO_CANCOPY))
//             DeleteMenu(hmenuDefault, IDM_COPY, MF_BYCOMMAND);
//         if (!((rfg & SFGAO_FILESYSTEM) && HasClipboardData()))
//             DeleteMenu(hmenuDefault, IDM_INSERT, MF_BYCOMMAND);
//         if (!(rfg & SFGAO_CANLINK))
//             DeleteMenu(hmenuDefault, IDM_CREATELINK, MF_BYCOMMAND);
//         if (!(rfg & SFGAO_CANDELETE))
//             DeleteMenu(hmenuDefault, IDM_DELETE, MF_BYCOMMAND);
//         if (!(rfg & SFGAO_CANRENAME) || !(uFlags & CMF_CANRENAME))
//             DeleteMenu(hmenuDefault, IDM_RENAME, MF_BYCOMMAND);
//         if (!(rfg & SFGAO_HASPROPSHEET))
//             DeleteMenu(hmenuDefault, IDM_PROPERTIES, MF_BYCOMMAND);
//
//         idDefaultOffset = idCmdNext;
//         UINT idMax = Shell_MergeMenus(hMenu, GetSubMenu(hmenuDefault, 0), IndexMenu, idCmdNext, idCmdLast, 0);
//         m_iIdDfltFirst = cIds;
//         cIds += idMax - idCmdNext;
//         m_iIdDfltLast = cIds;
//
//         DestroyMenu(hmenuDefault);
//     }
//
//     TryPickDefault(hMenu, idCmdFirst, idDefaultOffset, uFlags);
//
//     // TODO: DFM_MERGECONTEXTMENU_TOP
//
//     // TODO: Remove duplicate verbs. This will be easier when the static items handling
//     // has been moved to CLSID_ShellFileDefExt so we only have to deal with ShellEx.
//     // This is a Windows XP+ feature. On an unknown file type, Windows 2000 will
//     // display both "Open" (openas from Unknown) and "Open with..." (openas from *).
//
//     return MAKE_HRESULT(SEVERITY_SUCCESS, 0, cIds);
// }
// HRESULT CDefaultContextMenu::DoPaste(LPCMINVOKECOMMANDINFOEX lpcmi, BOOL bLink)
// {
//     HRESULT hr = _DoInvokeCommandCallback(lpcmi, DFM_CMD_PASTE);
//     if (hr == S_OK)
//         return hr;
//
//     CComPtr<IDataObject> pda;
//     hr = OleGetClipboard(&pda);
//     if (FAILED_UNEXPECTEDLY(hr))
//         return hr;
//
//     FORMATETC formatetc2;
//     STGMEDIUM medium2;
//     InitFormatEtc(formatetc2, RegisterClipboardFormatW(CFSTR_PREFERREDDROPEFFECT), TYMED_HGLOBAL);
//
//     DWORD dwKey= 0;
//
//     if (SUCCEEDED(pda->GetData(&formatetc2, &medium2)))
//     {
//         DWORD * pdwFlag = (DWORD*)GlobalLock(medium2.hGlobal);
//         if (pdwFlag)
//         {
//             if (*pdwFlag == DROPEFFECT_COPY)
//                 dwKey = MK_CONTROL;
//             else
//                 dwKey = MK_SHIFT;
//         }
//         else
//         {
//             ERR("No drop effect obtained\n");
//         }
//         GlobalUnlock(medium2.hGlobal);
//     }
//
//     if (bLink)
//     {
//         dwKey = MK_CONTROL|MK_SHIFT;
//     }
//
//     CComPtr<IDropTarget> pdrop;
//     if (m_cidl)
//         hr = m_psf->GetUIObjectOf(NULL, 1, &m_apidl[0], IID_NULL_PPV_ARG(IDropTarget, &pdrop));
//     else
//         hr = m_psf->CreateViewObject(NULL, IID_PPV_ARG(IDropTarget, &pdrop));
//
//     if (FAILED_UNEXPECTEDLY(hr))
//         return hr;
//
//     SHSimulateDrop(pdrop, pda, dwKey, NULL, NULL);
//
//     TRACE("CP result %x\n", hr);
//     return S_OK;
// }
// HRESULT
// CDefaultContextMenu::DoOpenOrExplore(LPCMINVOKECOMMANDINFOEX lpcmi)
// {
//     UNIMPLEMENTED;
//     return E_FAIL;
// }
// HRESULT CDefaultContextMenu::DoCreateLink(LPCMINVOKECOMMANDINFOEX lpcmi)
// {
//     HRESULT hr = _DoInvokeCommandCallback(lpcmi, DFM_CMD_LINK);
//     if (hr == S_OK)
//         return hr;
//
//     if (!m_cidl || !m_pDataObj)
//         return E_FAIL;
//
//     CComPtr<IDropTarget> pDT;
//     hr = m_psf->CreateViewObject(NULL, IID_PPV_ARG(IDropTarget, &pDT));
//     if (FAILED_UNEXPECTEDLY(hr))
//         return hr;
//
//     SHSimulateDrop(pDT, m_pDataObj, MK_CONTROL|MK_SHIFT, NULL, NULL);
//
//     return S_OK;
// }
// HRESULT CDefaultContextMenu::DoDelete(LPCMINVOKECOMMANDINFOEX lpcmi)
// {
//     HRESULT hr = _DoInvokeCommandCallback(lpcmi, DFM_CMD_DELETE);
//     if (hr == S_OK)
//         return hr;
//
//     if (!m_cidl || !m_pDataObj)
//         return E_FAIL;
//
//     CComPtr<IDropTarget> pDT;
//     hr = CRecyclerDropTarget_CreateInstance_RustBridge(IID_PPV_ARG(IDropTarget, &pDT));
//     if (FAILED_UNEXPECTEDLY(hr))
//         return hr;
//
//     DWORD grfKeyState = (lpcmi->fMask & CMIC_MASK_SHIFT_DOWN) ? MK_SHIFT : 0;
//     SHSimulateDrop(pDT, m_pDataObj, grfKeyState, NULL, NULL);
//
//     return S_OK;
// }
// HRESULT CDefaultContextMenu::DoCopyOrCut(LPCMINVOKECOMMANDINFOEX lpcmi, BOOL bCopy)
// {
//     if (!m_cidl || !m_pDataObj)
//         return E_FAIL;
//
//     FORMATETC formatetc;
//     InitFormatEtc(formatetc, RegisterClipboardFormatW(CFSTR_PREFERREDDROPEFFECT), TYMED_HGLOBAL);
//     STGMEDIUM medium = {0};
//     medium.tymed = TYMED_HGLOBAL;
//     medium.hGlobal = GlobalAlloc(GHND, sizeof(DWORD));
//     DWORD* pdwFlag = (DWORD*)GlobalLock(medium.hGlobal);
//     if (pdwFlag)
//         *pdwFlag = bCopy ? DROPEFFECT_COPY : DROPEFFECT_MOVE;
//     GlobalUnlock(medium.hGlobal);
//     m_pDataObj->SetData(&formatetc, &medium, TRUE);
//
//     CComPtr<IShellFolderView> psfv;
//     if (SUCCEEDED(IUnknown_QueryService(m_site, SID_SFolderView, IID_PPV_ARG(IShellFolderView, &psfv))))
//         psfv->SetPoints(m_pDataObj);
//
//     HRESULT hr = OleSetClipboard(m_pDataObj);
//     if (FAILED_UNEXPECTEDLY(hr))
//         return hr;
//
//     if (psfv)
//         psfv->SetClipboard(!bCopy);
//     return S_OK;
// }
// HRESULT CDefaultContextMenu::DoRename(LPCMINVOKECOMMANDINFOEX lpcmi)
// {
//     CComPtr<IShellBrowser> psb;
//     HRESULT hr;
//
//     hr = _DoInvokeCommandCallback(lpcmi, DFM_CMD_RENAME);
//     if (hr == S_OK)
//         return hr;
//
//     if (!m_site || !m_cidl)
//         return E_FAIL;
//
//     /* Get a pointer to the shell browser */
//     hr = IUnknown_QueryService(m_site, SID_IShellBrowser, IID_PPV_ARG(IShellBrowser, &psb));
//     if (FAILED_UNEXPECTEDLY(hr))
//         return hr;
//
//     CComPtr<IShellView> lpSV;
//     hr = psb->QueryActiveShellView(&lpSV);
//     if (FAILED_UNEXPECTEDLY(hr))
//         return hr;
//
//     SVSIF selFlags = SVSI_DESELECTOTHERS | SVSI_EDIT | SVSI_ENSUREVISIBLE | SVSI_FOCUSED | SVSI_SELECT;
//     hr = lpSV->SelectItem(m_apidl[0], selFlags);
//     if (FAILED_UNEXPECTEDLY(hr))
//         return hr;
//
//     return S_OK;
// }
// HRESULT
// CDefaultContextMenu::DoProperties(
//     LPCMINVOKECOMMANDINFOEX lpcmi)
// {
//     HRESULT hr = _DoInvokeCommandCallback(lpcmi, DFM_CMD_PROPERTIES);
//
//     // We are asked to run the default property sheet
//     if (hr == S_FALSE)
//     {
//         return SHELL32_ShowPropertiesDialog(m_pDataObj);
//     }
//
//     return hr;
// }
// HRESULT
// CDefaultContextMenu::DoUndo(LPCMINVOKECOMMANDINFOEX lpcmi)
// {
//     ERR("TODO: Undo\n");
//     return E_NOTIMPL;
// }
// HRESULT
// CDefaultContextMenu::DoCopyToMoveToFolder(LPCMINVOKECOMMANDINFOEX lpici, BOOL bCopy)
// {
//     HRESULT hr = E_FAIL;
//     if (!m_pDataObj)
//     {
//         ERR("m_pDataObj is NULL\n");
//         return hr;
//     }
//
//     CComPtr<IContextMenu> pContextMenu;
//     if (bCopy)
//         hr = SHCoCreateInstance(NULL, &CLSID_CopyToMenu, NULL,
//                                 IID_PPV_ARG(IContextMenu, &pContextMenu));
//     else
//         hr = SHCoCreateInstance(NULL, &CLSID_MoveToMenu, NULL,
//                                 IID_PPV_ARG(IContextMenu, &pContextMenu));
//     if (FAILED_UNEXPECTEDLY(hr))
//         return hr;
//
//     CComPtr<IShellExtInit> pInit;
//     hr = pContextMenu->QueryInterface(IID_PPV_ARG(IShellExtInit, &pInit));
//     if (FAILED_UNEXPECTEDLY(hr))
//         return hr;
//
//     hr = pInit->Initialize(m_pidlFolder, m_pDataObj, NULL);
//     if (FAILED_UNEXPECTEDLY(hr))
//         return hr;
//
//     if (bCopy)
//         lpici->lpVerb = "copyto";
//     else
//         lpici->lpVerb = "moveto";
//
//     return pContextMenu->InvokeCommand((LPCMINVOKECOMMANDINFO)lpici);
// }
// PDynamicShellEntry CDefaultContextMenu::GetDynamicEntry(UINT idCmd)
// {
//     POSITION it = m_DynamicEntries.GetHeadPosition();
//     while (it != NULL)
//     {
//         DynamicShellEntry& info = m_DynamicEntries.GetNext(it);
//
//         if (idCmd >= info.iIdCmdFirst + info.NumIds)
//             continue;
//
//         if (idCmd < info.iIdCmdFirst || idCmd > info.iIdCmdFirst + info.NumIds)
//             return NULL;
//
//         return &info;
//     }
//
//     return NULL;
// }
// BOOL
// CDefaultContextMenu::MapVerbToCmdId(PVOID Verb, PUINT idCmd, BOOL IsUnicode)
// {
//     WCHAR UnicodeStr[MAX_VERB];
//
//     /* Loop through all the static verbs looking for a match */
//     for (UINT i = 0; i < _countof(g_StaticInvokeCmdMap); i++)
//     {
//         /* We can match both ANSI and unicode strings */
//         if (IsUnicode)
//         {
//             /* The static verbs are ANSI, get a unicode version before doing the compare */
//             SHAnsiToUnicode(g_StaticInvokeCmdMap[i].szStringVerb, UnicodeStr, MAX_VERB);
//             if (!_wcsicmp(UnicodeStr, (LPWSTR)Verb))
//             {
//                 /* Return the Corresponding Id */
//                 *idCmd = g_StaticInvokeCmdMap[i].IntVerb;
//                 return TRUE;
//             }
//         }
//         else
//         {
//             if (!_stricmp(g_StaticInvokeCmdMap[i].szStringVerb, (LPSTR)Verb))
//             {
//                 *idCmd = g_StaticInvokeCmdMap[i].IntVerb;
//                 return TRUE;
//             }
//         }
//     }
//
//     for (POSITION it = m_DynamicEntries.GetHeadPosition(); it != NULL;)
//     {
//         DynamicShellEntry& entry = m_DynamicEntries.GetNext(it);
//         if (!entry.NumIds)
//             continue;
//         HRESULT hr = ::MapVerbToCmdId(Verb, IsUnicode, entry.pCM, 0, entry.NumIds - 1);
//         if (SUCCEEDED(hr))
//         {
//             *idCmd = m_iIdSHEFirst + entry.iIdCmdFirst + hr;
//             return TRUE;
//         }
//     }
//     return FALSE;
// }
// HRESULT
// CDefaultContextMenu::InvokeShellExt(
//     LPCMINVOKECOMMANDINFOEX lpcmi)
// {
//     TRACE("verb %p first %x last %x\n", lpcmi->lpVerb, m_iIdSHEFirst, m_iIdSHELast);
//
//     UINT idCmd = LOWORD(lpcmi->lpVerb);
//     PDynamicShellEntry pEntry = GetDynamicEntry(idCmd);
//     if (!pEntry)
//         return E_FAIL;
//
//     /* invoke the dynamic context menu */
//     lpcmi->lpVerb = MAKEINTRESOURCEA(idCmd - pEntry->iIdCmdFirst);
//     return pEntry->pCM->InvokeCommand((LPCMINVOKECOMMANDINFO)lpcmi);
// }
// DWORD
// CDefaultContextMenu::BrowserFlagsFromVerb(LPCMINVOKECOMMANDINFOEX lpcmi, PStaticShellEntry pEntry)
// {
//     CComPtr<IShellBrowser> psb;
//     HWND hwndTree;
//     LPCWSTR FlagsName;
//     WCHAR wszKey[sizeof("shell\\") + MAX_VERB];
//     HRESULT hr;
//
//     if (!m_site)
//         return 0;
//
//     /* Get a pointer to the shell browser */
//     hr = IUnknown_QueryService(m_site, SID_IShellBrowser, IID_PPV_ARG(IShellBrowser, &psb));
//     if (FAILED(hr))
//         return 0;
//
//     /* See if we are in Explore or Browse mode. If the browser's tree is present, we are in Explore mode.*/
//     if (SUCCEEDED(psb->GetControlWindow(FCW_TREE, &hwndTree)) && hwndTree)
//         FlagsName = L"ExplorerFlags";
//     else
//         FlagsName = L"BrowserFlags";
//
//     CComPtr<ICommDlgBrowser> pcdb;
//     if (SUCCEEDED(psb->QueryInterface(IID_PPV_ARG(ICommDlgBrowser, &pcdb))))
//     {
//         if (LOBYTE(GetVersion()) < 6 || FlagsName[0] == 'E')
//             return 0; // Don't browse in-place
//     }
//
//     /* Try to get the flag from the verb */
//     hr = StringCbPrintfW(wszKey, sizeof(wszKey), L"shell\\%s", pEntry->Verb.GetString());
//     if (FAILED_UNEXPECTEDLY(hr))
//         return 0;
//     return RegGetDword(pEntry->hkClass, wszKey, FlagsName, 0);
// }
// HRESULT
// CDefaultContextMenu::TryToBrowse(
//     LPCMINVOKECOMMANDINFOEX lpcmi, LPCITEMIDLIST pidlChild, DWORD wFlags)
// {
//     CComPtr<IShellBrowser> psb;
//     HRESULT hr;
//
//     if (!m_site)
//         return E_FAIL;
//
//     /* Get a pointer to the shell browser */
//     hr = IUnknown_QueryService(m_site, SID_IShellBrowser, IID_PPV_ARG(IShellBrowser, &psb));
//     if (FAILED(hr))
//         return hr;
//
//     PIDLIST_ABSOLUTE pidl;
//     hr = SHILCombine(m_pidlFolder, pidlChild, &pidl);
//     if (FAILED_UNEXPECTEDLY(hr))
//         return hr;
//
//     hr = psb->BrowseObject(pidl, wFlags & ~SBSP_RELATIVE);
//     ILFree(pidl);
//     return hr;
// }
// HRESULT
// CDefaultContextMenu::InvokePidl(LPCMINVOKECOMMANDINFOEX lpcmi, LPCITEMIDLIST pidl, PStaticShellEntry pEntry)
// {
//     const BOOL unicode = IsUnicode(lpcmi);
//
//     LPITEMIDLIST pidlFull = ILCombine(m_pidlFolder, pidl);
//     if (pidlFull == NULL)
//     {
//         return E_FAIL;
//     }
//
//     WCHAR wszPath[MAX_PATH];
//     BOOL bHasPath = SHGetPathFromIDListW(pidlFull, wszPath);
//
//     WCHAR wszDir[MAX_PATH];
//
//     SHELLEXECUTEINFOW sei = { sizeof(sei) };
//     sei.fMask = SEE_MASK_CLASSKEY | SEE_MASK_IDLIST | (CmicFlagsToSeeFlags(lpcmi->fMask) & ~SEE_MASK_INVOKEIDLIST);
//     sei.hwnd = lpcmi->hwnd;
//     sei.nShow = lpcmi->nShow;
//     sei.lpVerb = pEntry->Verb;
//     sei.lpIDList = pidlFull;
//     sei.hkeyClass = pEntry->hkClass;
//     sei.dwHotKey = lpcmi->dwHotKey;
//     sei.hIcon = lpcmi->hIcon;
//     sei.lpDirectory = wszDir;
//
//     if (unicode && !StrIsNullOrEmpty(lpcmi->lpDirectoryW))
//     {
//         sei.lpDirectory = lpcmi->lpDirectoryW;
//     }
//     else if (bHasPath)
//     {
//         wcscpy(wszDir, wszPath);
//         PathRemoveFileSpec(wszDir);
//     }
//     else
//     {
//         if (!SHGetPathFromIDListW(m_pidlFolder, wszDir))
//             *wszDir = UNICODE_NULL;
//     }
//
//     if (bHasPath)
//         sei.lpFile = wszPath;
//
//     CComHeapPtr<WCHAR> pszParamsW;
//     if (unicode && !StrIsNullOrEmpty(lpcmi->lpParametersW))
//         sei.lpParameters = lpcmi->lpParametersW;
//     else if (!StrIsNullOrEmpty(lpcmi->lpParameters) && __SHCloneStrAtoW(&pszParamsW, lpcmi->lpParameters))
//         sei.lpParameters = pszParamsW;
//
//     if (!sei.lpClass && (lpcmi->fMask & (CMIC_MASK_HASLINKNAME | CMIC_MASK_HASTITLE)) && unicode)
//         sei.lpClass = lpcmi->lpTitleW; // Forward .lnk path from CShellLink::DoOpen (for consrv STARTF_TITLEISLINKNAME)
//
//     HRESULT hr = ShellExecuteExW(&sei) ? S_OK : HResultFromWin32(GetLastError());
//     ILFree(pidlFull);
//     return hr;
// }
// HRESULT
// CDefaultContextMenu::_DoInvokeCommandCallback(
//     LPCMINVOKECOMMANDINFOEX lpcmi, WPARAM CmdId)
// {
//     BOOL Unicode = IsUnicode(lpcmi);
//     WCHAR lParamBuf[MAX_PATH];
//     LPARAM lParam = 0;
//
//     if (Unicode && lpcmi->lpParametersW)
//         lParam = (LPARAM)lpcmi->lpParametersW;
//     else if (lpcmi->lpParameters)
//         lParam = SHAnsiToUnicode(lpcmi->lpParameters, lParamBuf, _countof(lParamBuf)) ? (LPARAM)lParamBuf : 0;
//
//     HRESULT hr;
//         hr = _DoCallback(DFM_INVOKECOMMAND, CmdId, (void*)lParam);
//     return hr;
// }
// CDefaultContextMenu::InvokeCommand(
//     LPCMINVOKECOMMANDINFO lpcmi)
// {
//     CMINVOKECOMMANDINFOEX LocalInvokeInfo = {};
//     HRESULT Result;
//     UINT CmdId;
//
//     /* Take a local copy of the fixed members of the
//        struct as we might need to modify the verb */
//     memcpy(&LocalInvokeInfo, lpcmi, min(sizeof(LocalInvokeInfo), lpcmi->cbSize));
//
//     /* Check if this is a string verb */
//     if (!IS_INTRESOURCE(LocalInvokeInfo.lpVerb))
//     {
//         /* Get the ID which corresponds to this verb, and update our local copy */
//         if (MapVerbToCmdId((LPVOID)LocalInvokeInfo.lpVerb, &CmdId, FALSE))
//             LocalInvokeInfo.lpVerb = MAKEINTRESOURCEA(CmdId);
//         else
//             return E_INVALIDARG;
//     }
//     CmdId = LOWORD(LocalInvokeInfo.lpVerb);
//
//     if (!m_DynamicEntries.IsEmpty() && CmdId >= m_iIdSHEFirst && CmdId < m_iIdSHELast)
//     {
//         LocalInvokeInfo.lpVerb -= m_iIdSHEFirst;
//         Result = InvokeShellExt(&LocalInvokeInfo);
//         return Result;
//     }
//
//     if (!m_StaticEntries.IsEmpty() && CmdId >= m_iIdSCMFirst && CmdId < m_iIdSCMLast)
//     {
//         LocalInvokeInfo.lpVerb -= m_iIdSCMFirst;
//         Result = InvokeRegVerb(&LocalInvokeInfo);
//         // TODO: if (FAILED(Result) && !(lpcmi->fMask & CMIC_MASK_FLAG_NO_UI)) SHELL_ErrorBox(m_pSite, Result);
//         return Result;
//     }
//
//     if (m_iIdCBFirst != m_iIdCBLast && CmdId >= m_iIdCBFirst && CmdId < m_iIdCBLast)
//     {
//         Result = _DoInvokeCommandCallback(&LocalInvokeInfo, CmdId - m_iIdCBFirst);
//         return Result;
//     }
//
//     if (m_iIdDfltFirst != m_iIdDfltLast && CmdId >= m_iIdDfltFirst && CmdId < m_iIdDfltLast)
//     {
//         CmdId -= m_iIdDfltFirst;
//         /* See the definitions of IDM_CUT and co to see how this works */
//         CmdId += DCM_FCIDM_SHVIEW_OFFSET;
//     }
//
//     if (LocalInvokeInfo.cbSize >= sizeof(CMINVOKECOMMANDINFOEX) && (LocalInvokeInfo.fMask & CMIC_MASK_PTINVOKE))
//     {
//         if (m_pDataObj && FAILED_UNEXPECTEDLY(DataObject_SetOffset(m_pDataObj, &LocalInvokeInfo.ptInvoke)))
//         {
//             ERR("Unable to add OFFSET to DataObject!\n");
//         }
//     }
//
//     /* Check if this is a Id */
//     switch (CmdId)
//     {
//     case FCIDM_SHVIEW_INSERT:
//         Result = DoPaste(&LocalInvokeInfo, FALSE);
//         break;
//     case FCIDM_SHVIEW_INSERTLINK:
//         Result = DoPaste(&LocalInvokeInfo, TRUE);
//         break;
//     case FCIDM_SHVIEW_OPEN:
//     case FCIDM_SHVIEW_EXPLORE:
//         Result = DoOpenOrExplore(&LocalInvokeInfo);
//         break;
//     case FCIDM_SHVIEW_COPY:
//     case FCIDM_SHVIEW_CUT:
//         Result = DoCopyOrCut(&LocalInvokeInfo, CmdId == FCIDM_SHVIEW_COPY);
//         break;
//     case FCIDM_SHVIEW_CREATELINK:
//         Result = DoCreateLink(&LocalInvokeInfo);
//         break;
//     case FCIDM_SHVIEW_DELETE:
//         Result = DoDelete(&LocalInvokeInfo);
//         break;
//     case FCIDM_SHVIEW_RENAME:
//         Result = DoRename(&LocalInvokeInfo);
//         break;
//     case FCIDM_SHVIEW_PROPERTIES:
//         Result = DoProperties(&LocalInvokeInfo);
//         break;
//     case FCIDM_SHVIEW_NEWFOLDER:
//         Result = DoCreateNewFolder(&LocalInvokeInfo);
//         break;
//     case FCIDM_SHVIEW_COPYTO:
//         Result = DoCopyToMoveToFolder(&LocalInvokeInfo, TRUE);
//         break;
//     case FCIDM_SHVIEW_MOVETO:
//         Result = DoCopyToMoveToFolder(&LocalInvokeInfo, FALSE);
//         break;
//     case FCIDM_SHVIEW_UNDO:
//         Result = DoUndo(&LocalInvokeInfo);
//         break;
//     default:
//         Result = E_INVALIDARG;
//         ERR("Unhandled Verb %xl\n", LOWORD(LocalInvokeInfo.lpVerb));
//         break;
//     }
//
//     return Result;
// }
// CDefaultContextMenu::GetCommandString(
//     UINT_PTR idCommand,
//     UINT uFlags,
//     UINT* lpReserved,
//     LPSTR lpszName,
//     UINT uMaxNameLen)
// {
//     /* We don't handle the help text yet */
//     if (uFlags == GCS_HELPTEXTA ||
//         uFlags == GCS_HELPTEXTW ||
//         HIWORD(idCommand) != 0)
//     {
//         return E_NOTIMPL;
//     }
//
//     UINT CmdId = LOWORD(idCommand);
//
//     if (!m_DynamicEntries.IsEmpty() && CmdId >= m_iIdSHEFirst && CmdId < m_iIdSHELast)
//     {
//         idCommand -= m_iIdSHEFirst;
//         PDynamicShellEntry pEntry = GetDynamicEntry(idCommand);
//         if (!pEntry)
//             return E_FAIL;
//
//         idCommand -= pEntry->iIdCmdFirst;
//         return pEntry->pCM->GetCommandString(idCommand,
//                                              uFlags,
//                                              lpReserved,
//                                              lpszName,
//                                              uMaxNameLen);
//     }
//
//     if (!m_StaticEntries.IsEmpty() && CmdId >= m_iIdSCMFirst && CmdId < m_iIdSCMLast)
//     {
//         /* Validation just returns S_OK on a match. The id exists. */
//         if (uFlags == GCS_VALIDATEA || uFlags == GCS_VALIDATEW)
//             return S_OK;
//
//         CmdId -= m_iIdSCMFirst;
//
//         POSITION it = m_StaticEntries.FindIndex(CmdId);
//
//         if (it == NULL)
//             return E_INVALIDARG;
//
//         PStaticShellEntry pEntry = &m_StaticEntries.GetAt(it);
//
//         if (uFlags == GCS_VERBW)
//             return StringCchCopyW((LPWSTR)lpszName, uMaxNameLen, pEntry->Verb);
//
//         if (uFlags == GCS_VERBA)
//         {
//             if (SHUnicodeToAnsi(pEntry->Verb, lpszName, uMaxNameLen))
//                 return S_OK;
//         }
//
//         return E_INVALIDARG;
//     }
//
//     //FIXME: Should we handle callbacks here?
//     if (m_iIdDfltFirst != m_iIdDfltLast && CmdId >= m_iIdDfltFirst && CmdId < m_iIdDfltLast)
//     {
//         CmdId -= m_iIdDfltFirst;
//         /* See the definitions of IDM_CUT and co to see how this works */
//         CmdId += DCM_FCIDM_SHVIEW_OFFSET;
//     }
//
//     /* Loop looking for a matching Id */
//     for (UINT i = 0; i < _countof(g_StaticInvokeCmdMap); i++)
//     {
//         if (g_StaticInvokeCmdMap[i].IntVerb == CmdId)
//         {
//             /* Validation just returns S_OK on a match */
//             if (uFlags == GCS_VALIDATEA || uFlags == GCS_VALIDATEW)
//                 return S_OK;
//
//             /* Return a copy of the ANSI verb */
//             if (uFlags == GCS_VERBA)
//                 return StringCchCopyA(lpszName, uMaxNameLen, g_StaticInvokeCmdMap[i].szStringVerb);
//
//             /* Convert the ANSI verb to unicode and return that */
//             if (uFlags == GCS_VERBW)
//             {
//                 if (SHAnsiToUnicode(g_StaticInvokeCmdMap[i].szStringVerb, (LPWSTR)lpszName, uMaxNameLen))
//                     return S_OK;
//             }
//         }
//     }
//
//     return E_INVALIDARG;
// }
// CDefaultContextMenu::HandleMenuMsg(
//     UINT uMsg,
//     WPARAM wParam,
//     LPARAM lParam)
// {
//     return HandleMenuMsg2(uMsg, wParam, lParam, NULL);
// }
// CDefaultContextMenu::HandleMenuMsg2(
//     UINT uMsg,
//     WPARAM wParam,
//     LPARAM lParam,
//     LRESULT *plResult)
// {
//     if (!SHELL_IsContextMenuMsg(uMsg))
//         return E_FAIL;
//
//     UINT CmdId;
//     if (uMsg == WM_INITMENUPOPUP)
//     {
//         CmdId = GetMenuItemID((HMENU)wParam, 0);
//         if (CmdId == ~0ul)
//             return E_FAIL;
//     }
//     else
//     {
//         HRESULT hr = SHGetMenuIdFromMenuMsg(uMsg, lParam, &CmdId);
//         if (FAILED(hr))
//             return S_FALSE;
//     }
//     CmdId -= m_iIdQCMFirst; // Convert from Win32 id to our base
//
//     if (CmdId >= m_iIdSHEFirst && CmdId < m_iIdSHELast)
//     {
//         if (PDynamicShellEntry pEntry = GetDynamicEntry(CmdId - m_iIdSHEFirst))
//             return SHForwardContextMenuMsg(pEntry->pCM, uMsg, wParam, lParam, plResult, TRUE);
//     }
//     // TODO: _DoCallback(DFM_WM_*, ...)
//     return E_FAIL;
// }
// CDefaultContextMenu::SetSite(IUnknown *pUnkSite)
// {
//     m_site = pUnkSite;
//     return S_OK;
// }
// CDefaultContextMenu::GetSite(REFIID riid, void **ppvSite)
// {
//     if (!m_site)
//         return E_FAIL;
//
//     return m_site->QueryInterface(riid, ppvSite);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
