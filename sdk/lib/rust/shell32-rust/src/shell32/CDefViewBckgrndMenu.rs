// rustconv-assembly: class:CDefViewBckgrndMenu
// Deterministic from shobjidl.h
use crate::prelude::*;
// The crate's own functions: the definition is the declaration.
use crate::wine::shlmenu::Shell_MergeMenus;
// One declaration per external symbol: island.rs.
use crate::island::DestroyMenu;
use crate::island::IUnknown_QueryService;
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

// Deterministic COM scaffold for CDefViewBckgrndMenu; method bodies are generated separately.
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
pub struct CDefViewBckgrndMenu {
    pub IContextMenu3_iface: IContextMenu3,
    pub IObjectWithSite_iface: IObjectWithSite,
    pub ref_count: core::sync::atomic::AtomicI32,
    pub m_site: *mut IUnknown,
    pub m_psf: *mut IShellFolder,
    pub m_folderCM: *mut IContextMenu,
    pub m_idCmdFirst: UINT,
    pub m_LastFolderCMId: UINT,
}

unsafe fn CDefViewBckgrndMenu_cpp_com_assign_m_site(this: *mut CDefViewBckgrndMenu, value: *mut IUnknown) {
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

unsafe fn CDefViewBckgrndMenu_cpp_com_assign_m_psf(this: *mut CDefViewBckgrndMenu, value: *mut IShellFolder) {
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

unsafe fn CDefViewBckgrndMenu_cpp_com_assign_m_folderCM(this: *mut CDefViewBckgrndMenu, value: *mut IContextMenu) {
    let old = (*this).m_folderCM;
    if old == value { return; }
    if !value.is_null() {
        let unknown = value as *mut IUnknown;
        let add_ref: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).AddRef);
        add_ref(unknown);
    }
    (*this).m_folderCM = value;
    if !old.is_null() {
        let unknown = old as *mut IUnknown;
        let release: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).Release);
        release(unknown);
    }
}

unsafe extern "system" fn CDefViewBckgrndMenu_IContextMenu3_QueryInterface(This: *mut IContextMenu3, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefViewBckgrndMenu, IContextMenu3_iface)) as *mut CDefViewBckgrndMenu;
    CDefViewBckgrndMenu_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CDefViewBckgrndMenu_IContextMenu3_AddRef(This: *mut IContextMenu3) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefViewBckgrndMenu, IContextMenu3_iface)) as *mut CDefViewBckgrndMenu;
    CDefViewBckgrndMenu_AddRef(this)
}
unsafe extern "system" fn CDefViewBckgrndMenu_IContextMenu3_Release(This: *mut IContextMenu3) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefViewBckgrndMenu, IContextMenu3_iface)) as *mut CDefViewBckgrndMenu;
    CDefViewBckgrndMenu_Release(this)
}
unsafe extern "system" fn CDefViewBckgrndMenu_IContextMenu3_QueryContextMenu(This: *mut IContextMenu3, hmenu: HMENU, indexMenu: UINT, idCmdFirst: UINT, idCmdLast: UINT, uFlags: UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefViewBckgrndMenu, IContextMenu3_iface)) as *mut CDefViewBckgrndMenu;
    CDefViewBckgrndMenu_QueryContextMenu_body(this, hmenu, indexMenu, idCmdFirst, idCmdLast, uFlags)
}
unsafe extern "system" fn CDefViewBckgrndMenu_IContextMenu3_InvokeCommand(This: *mut IContextMenu3, lpici: LPCMINVOKECOMMANDINFO) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefViewBckgrndMenu, IContextMenu3_iface)) as *mut CDefViewBckgrndMenu;
    CDefViewBckgrndMenu_InvokeCommand_body(this, lpici)
}
unsafe extern "system" fn CDefViewBckgrndMenu_IContextMenu3_GetCommandString(This: *mut IContextMenu3, idCmd: UINT_PTR, uType: UINT, pwReserved: *mut UINT, pszName: LPSTR, cchMax: UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefViewBckgrndMenu, IContextMenu3_iface)) as *mut CDefViewBckgrndMenu;
    CDefViewBckgrndMenu_GetCommandString_body(this, idCmd, uType, pwReserved, pszName, cchMax)
}
unsafe extern "system" fn CDefViewBckgrndMenu_IContextMenu3_HandleMenuMsg(This: *mut IContextMenu3, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefViewBckgrndMenu, IContextMenu3_iface)) as *mut CDefViewBckgrndMenu;
    CDefViewBckgrndMenu_HandleMenuMsg_body(this, uMsg, wParam, lParam)
}
unsafe extern "system" fn CDefViewBckgrndMenu_IContextMenu3_HandleMenuMsg2(This: *mut IContextMenu3, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, plResult: *mut LRESULT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefViewBckgrndMenu, IContextMenu3_iface)) as *mut CDefViewBckgrndMenu;
    CDefViewBckgrndMenu_HandleMenuMsg2_body(this, uMsg, wParam, lParam, plResult)
}

pub static CDEFVIEWBCKGRNDMENU_ICONTEXTMENU3_VTBL: IContextMenu3Vtbl = IContextMenu3Vtbl {
    QueryInterface: CDefViewBckgrndMenu_IContextMenu3_QueryInterface,
    AddRef: CDefViewBckgrndMenu_IContextMenu3_AddRef,
    Release: CDefViewBckgrndMenu_IContextMenu3_Release,
    QueryContextMenu: CDefViewBckgrndMenu_IContextMenu3_QueryContextMenu,
    InvokeCommand: CDefViewBckgrndMenu_IContextMenu3_InvokeCommand,
    GetCommandString: CDefViewBckgrndMenu_IContextMenu3_GetCommandString,
    HandleMenuMsg: CDefViewBckgrndMenu_IContextMenu3_HandleMenuMsg,
    HandleMenuMsg2: CDefViewBckgrndMenu_IContextMenu3_HandleMenuMsg2,
};

unsafe extern "system" fn CDefViewBckgrndMenu_IObjectWithSite_QueryInterface(This: *mut IObjectWithSite, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefViewBckgrndMenu, IObjectWithSite_iface)) as *mut CDefViewBckgrndMenu;
    CDefViewBckgrndMenu_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CDefViewBckgrndMenu_IObjectWithSite_AddRef(This: *mut IObjectWithSite) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefViewBckgrndMenu, IObjectWithSite_iface)) as *mut CDefViewBckgrndMenu;
    CDefViewBckgrndMenu_AddRef(this)
}
unsafe extern "system" fn CDefViewBckgrndMenu_IObjectWithSite_Release(This: *mut IObjectWithSite) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefViewBckgrndMenu, IObjectWithSite_iface)) as *mut CDefViewBckgrndMenu;
    CDefViewBckgrndMenu_Release(this)
}
unsafe extern "system" fn CDefViewBckgrndMenu_IObjectWithSite_SetSite(This: *mut IObjectWithSite, pUnkSite: *mut IUnknown) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefViewBckgrndMenu, IObjectWithSite_iface)) as *mut CDefViewBckgrndMenu;
    CDefViewBckgrndMenu_SetSite_body(this, pUnkSite)
}
unsafe extern "system" fn CDefViewBckgrndMenu_IObjectWithSite_GetSite(This: *mut IObjectWithSite, riid: REFIID, ppvSite: *mut PVOID) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefViewBckgrndMenu, IObjectWithSite_iface)) as *mut CDefViewBckgrndMenu;
    CDefViewBckgrndMenu_GetSite_body(this, riid, ppvSite)
}

pub static CDEFVIEWBCKGRNDMENU_IOBJECTWITHSITE_VTBL: IObjectWithSiteVtbl = IObjectWithSiteVtbl {
    QueryInterface: CDefViewBckgrndMenu_IObjectWithSite_QueryInterface,
    AddRef: CDefViewBckgrndMenu_IObjectWithSite_AddRef,
    Release: CDefViewBckgrndMenu_IObjectWithSite_Release,
    SetSite: CDefViewBckgrndMenu_IObjectWithSite_SetSite,
    GetSite: CDefViewBckgrndMenu_IObjectWithSite_GetSite,
};

unsafe fn CDefViewBckgrndMenu_QueryInterface(
    this: *mut CDefViewBckgrndMenu, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
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
    else { return E_NOINTERFACE; }
    CDefViewBckgrndMenu_AddRef(this);
    S_OK
}

unsafe fn CDefViewBckgrndMenu_AddRef(this: *mut CDefViewBckgrndMenu) -> ULONG {
    (*this).ref_count.fetch_add(1, core::sync::atomic::Ordering::SeqCst) as ULONG + 1
}

unsafe fn CDefViewBckgrndMenu_Release(this: *mut CDefViewBckgrndMenu) -> ULONG {
    let refs = (*this).ref_count.fetch_sub(1, core::sync::atomic::Ordering::SeqCst) - 1;
    if refs == 0 {
        CDefViewBckgrndMenu_dtor_body(this);
        CDefViewBckgrndMenu_cpp_com_assign_m_site(this, core::ptr::null_mut());
        CDefViewBckgrndMenu_cpp_com_assign_m_psf(this, core::ptr::null_mut());
        CDefViewBckgrndMenu_cpp_com_assign_m_folderCM(this, core::ptr::null_mut());
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
#[no_mangle]
pub unsafe extern "C" fn CDefViewBckgrndMenu_CreateInstance(
    psf: *mut IShellFolder,
    riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    let this = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<CDefViewBckgrndMenu>()) as *mut CDefViewBckgrndMenu;
    if this.is_null() { return E_OUTOFMEMORY; }
    core::ptr::write(this, CDefViewBckgrndMenu {
        IContextMenu3_iface: IContextMenu3 { lpVtbl: core::ptr::addr_of!(CDEFVIEWBCKGRNDMENU_ICONTEXTMENU3_VTBL) },
        IObjectWithSite_iface: IObjectWithSite { lpVtbl: core::ptr::addr_of!(CDEFVIEWBCKGRNDMENU_IOBJECTWITHSITE_VTBL) },
        ref_count: core::sync::atomic::AtomicI32::new(1),
        m_site: core::mem::zeroed(),
        m_psf: core::mem::zeroed(),
        m_folderCM: core::mem::zeroed(),
        m_idCmdFirst: core::mem::zeroed(),
        m_LastFolderCMId: core::mem::zeroed(),
    });
    Shell32RustObjectLock();
    CDefViewBckgrndMenu_ctor_body(this);
    let mut hr = CDefViewBckgrndMenu_Initialize_body(this, psf);
    if hr >= 0 {
        hr = CDefViewBckgrndMenu_QueryInterface(this, riid, ppv);
    }
    CDefViewBckgrndMenu_Release(this);
    hr
}

// Deterministic C++ support closure selected from source identifiers.

pub const FCIDM_SHVIEW_ALIGNTOGRID: UINT = 0x7033;

pub const FCIDM_SHVIEW_AUTOARRANGE: UINT = 0x7031;

pub const FCIDM_SHVIEW_BIGICON: UINT = 0x7029;

pub const FCIDM_SHVIEW_INSERT: UINT = 0x701A;

pub const FCIDM_SHVIEW_INSERTLINK: UINT = 0x701C;

pub const FCIDM_SHVIEW_LISTVIEW: UINT = 0x702B;

pub const FCIDM_SHVIEW_REFRESH: UINT = 0x7103;

pub const FCIDM_SHVIEW_REPORTVIEW: UINT = 0x702C;

pub const FCIDM_SHVIEW_SMALLICON: UINT = 0x702A;

pub const FCIDM_SHVIEW_SNAPTOGRID: UINT = 0x7032;

pub const FCIDM_SHVIEW_VIEW: UINT = 0x7002;

pub const FWF_DESKTOP: DWORD = 0x20;

pub const MF_BYCOMMAND: DWORD = 0;

pub const MF_GRAYED: DWORD = 1;

pub const MM_ADDSEPARATOR: DWORD = 0x00000001;

pub const MM_DONTREMOVESEPS: DWORD = 0x00000004;

pub const MM_SUBMENUSHAVEIDS: DWORD = 0x00000002;

pub const TYMED_HGLOBAL: DWORD = 1;

pub const WM_COMMAND: DWORD = 273;

#[repr(C)]
pub struct DVTARGETDEVICE {
    pub tdSize: DWORD,
    pub tdDriverNameOffset: WORD,
    pub tdDeviceNameOffset: WORD,
    pub tdPortNameOffset: WORD,
    pub tdExtDevmodeOffset: WORD,
    pub tdData: [BYTE; 1],
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

extern "system" {
}

extern "C" {
    static IID_IDropTarget: GUID;
    static IID_IFolderView: GUID;
}

// ---- integration substrate (hand-added; the assembly references these but declares none) ----
// shellcom owns the COM vtable ABI, and taking the GUID family from it is ALL-OR-NOTHING: the
// vtable slots here are spelled with shellcom's GUID, the IID_* statics above with IID, and
// the signatures with REFIID/REFGUID, so importing some and not the rest only moves the
// mismatch. IDropTarget comes from shellcom for a second reason — the island renders its
// vtable slots as `Option<fn>` where shellcom uses raw fn pointers, so the island's cannot be
// called at all (prelude.rs says exactly this).
use crate::shellcom::{GUID, IID, IDropTarget, IDropTargetVtbl, REFGUID, REFIID};
use crate::shellutils::CFSTR_SHELLIDLISTW;

// shell32_main.h:144 — a #define, so it is in no header the island reads.
#[inline]
unsafe fn InitFormatEtc(fe: *mut FORMATETC, cf: UINT, med: DWORD) {
    (*fe).cfFormat = cf as CLIPFORMAT;
    (*fe).dwAspect = DVASPECT_CONTENT;
    (*fe).ptd = core::ptr::null_mut();
    (*fe).tymed = med;
    (*fe).lindex = -1;
}

// winuser.h — `MAKEWPARAM(l, h)` is `(WPARAM)(DWORD)MAKELONG(l, h)`.
#[inline]
fn MAKEWPARAM(lo: UINT, hi: UINT) -> WPARAM {
    (((lo & 0xffff) | ((hi & 0xffff) << 16)) as DWORD) as WPARAM
}

extern "system" {
    fn OleGetClipboard(ppDataObj: *mut *mut IDataObject) -> HRESULT;
}

extern "C" {
    fn strcmp(left: LPCSTR, right: LPCSTR) -> INT;
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

extern "C" {
    static mut shell32_hInstance: HINSTANCE;
}

// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_2A313251BCDB: [WCHAR; 9] = [0x004D, 0x0045, 0x004E, 0x0055, 0x005F, 0x0030, 0x0030, 0x0032, 0x0000];

use core::ffi::c_void;
// ONE glob only. The scaffold header already emits `use crate::prelude::*` at the top of
// this file; adding the shellcom/types/win32 set beside it makes every shared name
// E0659-ambiguous, which is exactly what prelude.rs's own header warns about. Same
// resolution as the two droptargets/ modules.

pub unsafe fn CDefViewBckgrndMenu_ctor_body(this: *mut CDefViewBckgrndMenu) -> () {
    (*this).m_idCmdFirst = 0;
    (*this).m_LastFolderCMId = 0;
}

pub unsafe fn CDefViewBckgrndMenu_dtor_body(this: *mut CDefViewBckgrndMenu) -> () {}

pub unsafe fn CDefViewBckgrndMenu__bIsDesktopBrowserMenu_body(this: *mut CDefViewBckgrndMenu) -> BOOL {
    if (*this).m_site.is_null() {
        return FALSE;
    }

    let mut psv: *mut IShellView = core::ptr::null_mut();
    let _psv_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(psv));
    let hr = IUnknown_QueryService(((*this).m_site).cast(), core::ptr::addr_of!(IID_IFolderView), core::ptr::addr_of!(IID_IShellView), core::ptr::addr_of_mut!(psv).cast());
    if ((hr) < 0) {
        return FALSE;
    }

    let mut FolderSettings: FOLDERSETTINGS = core::mem::zeroed();
    let hr = ((*(*psv).lpVtbl).GetCurrentInfo)(psv, &mut FolderSettings);
    if ((hr) < 0) {
        if !psv.is_null() {
            ((*(*psv).lpVtbl).Release)(psv);
            psv = core::ptr::null_mut();
        }
        return FALSE;
    }

    if !psv.is_null() {
        ((*(*psv).lpVtbl).Release)(psv);
        psv = core::ptr::null_mut();
    }

    ((FolderSettings.fFlags & FWF_DESKTOP) == FWF_DESKTOP) as BOOL
}

pub unsafe fn CDefViewBckgrndMenu__bCanPaste_body(this: *mut CDefViewBckgrndMenu) -> BOOL {
    let mut pdt: *mut IDropTarget = core::ptr::null_mut();
    let _pdt_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pdt));
    let hr = ((*(*(*this).m_psf).lpVtbl).CreateViewObject)(
        (*this).m_psf,
        core::ptr::null_mut(),
        core::ptr::addr_of!(IID_IDropTarget), core::ptr::addr_of_mut!(pdt).cast(),
    );
    if hr < 0 {
        if !pdt.is_null() {
            ((*(*pdt).lpVtbl).Release)(pdt);
            pdt = core::ptr::null_mut();
        }
        return FALSE;
    }

    let mut pDataObj: *mut IDataObject = core::ptr::null_mut();
    let _pDataObj_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pDataObj));
    let hr = OleGetClipboard(&mut pDataObj);
    if hr < 0 {
        if !pdt.is_null() {
            ((*(*pdt).lpVtbl).Release)(pdt);
            pdt = core::ptr::null_mut();
        }
        return FALSE;
    }

    let mut medium: STGMEDIUM = core::mem::zeroed();
    let mut formatetc: FORMATETC = core::mem::zeroed();

    InitFormatEtc(&mut formatetc, RegisterClipboardFormatW(CFSTR_SHELLIDLISTW.as_ptr()), TYMED_HGLOBAL);
    let hr = ((*(*pDataObj).lpVtbl).GetData)(pDataObj, core::ptr::addr_of_mut!(formatetc).cast(), core::ptr::addr_of_mut!(medium).cast());
    if hr < 0 {
        if !pDataObj.is_null() {
            ((*(*pDataObj).lpVtbl).Release)(pDataObj);
            pDataObj = core::ptr::null_mut();
        }
        if !pdt.is_null() {
            ((*(*pdt).lpVtbl).Release)(pdt);
            pdt = core::ptr::null_mut();
        }
        return FALSE;
    }

    ReleaseStgMedium(&mut medium);

    if !pDataObj.is_null() {
        ((*(*pDataObj).lpVtbl).Release)(pDataObj);
        pDataObj = core::ptr::null_mut();
    }
    if !pdt.is_null() {
        ((*(*pdt).lpVtbl).Release)(pdt);
        pdt = core::ptr::null_mut();
    }

    TRUE
}

pub unsafe fn CDefViewBckgrndMenu_Initialize_body(this: *mut CDefViewBckgrndMenu, psf: *mut IShellFolder) -> HRESULT {
    CDefViewBckgrndMenu_cpp_com_assign_m_psf(this, psf);

    ((*(*(*this).m_psf).lpVtbl).CreateViewObject)(
        (*this).m_psf,
        core::ptr::null_mut(),
        core::ptr::addr_of!(IID_IContextMenu), core::ptr::addr_of_mut!((*this).m_folderCM).cast()
    );

    S_OK
}

pub unsafe fn CDefViewBckgrndMenu_SetSite_body(this: *mut CDefViewBckgrndMenu, pUnkSite: *mut IUnknown) -> HRESULT {
    CDefViewBckgrndMenu_cpp_com_assign_m_site(this, pUnkSite);

    if !(*this).m_folderCM.is_null() {
        IUnknown_SetSite((*this).m_folderCM.cast(), pUnkSite.cast());
    }

    S_OK
}

pub unsafe fn CDefViewBckgrndMenu_GetSite_body(this: *mut CDefViewBckgrndMenu, riid: REFIID, ppvSite: *mut PVOID) -> HRESULT {
    if (*this).m_site.is_null() {
        return E_FAIL;
    }

    ((*(*(*this).m_site).lpVtbl).QueryInterface)((*this).m_site, riid, ppvSite)
}

pub unsafe fn CDefViewBckgrndMenu_QueryContextMenu_body(this: *mut CDefViewBckgrndMenu, hMenu: HMENU, indexMenu: UINT, idCmdFirst: UINT, idCmdLast: UINT, uFlags: UINT) -> HRESULT {
    let mut hr: HRESULT;
    let hMenuPart: HMENU;
    let mut cIds: UINT = 0;

    (*this).m_idCmdFirst = idCmdFirst;

    if !(*this).m_folderCM.is_null() {
        hr = ((*(*(*this).m_folderCM).lpVtbl).QueryContextMenu)(
            (*this).m_folderCM,
            hMenu,
            indexMenu,
            idCmdFirst,
            idCmdLast,
            uFlags,
        );
        if hr >= 0 {
            (*this).m_LastFolderCMId = LOWORD!(hr);
            cIds = (*this).m_LastFolderCMId;
        } else {
        }
    } else {
    }

    hMenuPart = LoadMenuW(shell32_hInstance, CPP_COM_WSTR_2A313251BCDB.as_ptr());
    if !hMenuPart.is_null() {
        if CDefViewBckgrndMenu__bIsDesktopBrowserMenu_body(this) != 0 {
            DeleteMenu(hMenuPart, FCIDM_SHVIEW_VIEW, MF_BYCOMMAND);
        }

        if CDefViewBckgrndMenu__bCanPaste_body(this) == 0 {
            EnableMenuItem(
                hMenuPart,
                FCIDM_SHVIEW_INSERT,
                MF_BYCOMMAND | MF_GRAYED,
            );
            EnableMenuItem(
                hMenuPart,
                FCIDM_SHVIEW_INSERTLINK,
                MF_BYCOMMAND | MF_GRAYED,
            );
        }

        Shell_MergeMenus(
            hMenu,
            GetSubMenu(hMenuPart, 0),
            indexMenu,
            0,
            idCmdLast,
            MM_DONTREMOVESEPS | MM_SUBMENUSHAVEIDS | MM_ADDSEPARATOR,
        );
        DestroyMenu(hMenuPart);
    } else {
    }

    MAKE_HRESULT(SEVERITY_SUCCESS, 0, cIds)
}

pub unsafe fn CDefViewBckgrndMenu_InvokeCommand_body(this: *mut CDefViewBckgrndMenu, lpcmi: LPCMINVOKECOMMANDINFO) -> HRESULT {
    let mut idCmd = LOWORD!((*lpcmi).lpVerb);

    if HIWORD((*lpcmi).lpVerb as usize as DWORD) != 0 && strcmp((*lpcmi).lpVerb, CMDSTR_VIEWLISTA.as_ptr() as LPCSTR) == 0 {
        idCmd = FCIDM_SHVIEW_LISTVIEW;
    } else if HIWORD((*lpcmi).lpVerb as usize as DWORD) != 0 && strcmp((*lpcmi).lpVerb, CMDSTR_VIEWDETAILSA.as_ptr() as LPCSTR) == 0 {
        idCmd = FCIDM_SHVIEW_REPORTVIEW;
    } else if HIWORD((*lpcmi).lpVerb as usize as DWORD) != 0 || idCmd < (*this).m_LastFolderCMId {
        if !(*this).m_folderCM.is_null() {
            return ((*(*(*this).m_folderCM).lpVtbl).InvokeCommand)((*this).m_folderCM, lpcmi);
        }

        return E_NOTIMPL;
    } else {
        idCmd += (*this).m_idCmdFirst;
    }

    match idCmd {
        FCIDM_SHVIEW_INSERT | FCIDM_SHVIEW_INSERTLINK => {
            if !(*this).m_folderCM.is_null() {
                (*lpcmi).lpVerb = MAKEINTRESOURCEA(idCmd);
                return ((*(*(*this).m_folderCM).lpVtbl).InvokeCommand)((*this).m_folderCM, lpcmi);
            }

            return E_NOTIMPL;
        }
        FCIDM_SHVIEW_BIGICON
        | FCIDM_SHVIEW_SMALLICON
        | FCIDM_SHVIEW_LISTVIEW
        | FCIDM_SHVIEW_REPORTVIEW
        | FCIDM_SHVIEW_AUTOARRANGE
        | FCIDM_SHVIEW_SNAPTOGRID
        | FCIDM_SHVIEW_REFRESH
        | FCIDM_SHVIEW_ALIGNTOGRID => {
            if (*this).m_site.is_null() {
                return E_FAIL;
            }

            let mut psv: *mut IShellView = core::ptr::null_mut();
            let _psv_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(psv));
            let hr = IUnknown_QueryService(((*this).m_site).cast(), core::ptr::addr_of!(IID_IFolderView), core::ptr::addr_of!(IID_IShellView), core::ptr::addr_of_mut!(psv).cast());
            if ((hr) < 0) {
                return hr;
            }

            let mut hwndSV: HWND = core::ptr::null_mut();
            if ((*(*psv).lpVtbl).GetWindow)(psv, &mut hwndSV) >= 0 {
                SendMessageW(hwndSV, WM_COMMAND, MAKEWPARAM(idCmd, 0), 0);
            }
            if !psv.is_null() {
                ((*(*psv).lpVtbl).Release)(psv);
                psv = core::ptr::null_mut();
            }
            return S_OK;
        }
        _ => {}
    }

    E_FAIL
}

pub unsafe fn CDefViewBckgrndMenu_GetCommandString_body(this: *mut CDefViewBckgrndMenu, idCommand: UINT_PTR, uFlags: UINT, lpReserved: *mut UINT, lpszName: LPSTR, uMaxNameLen: UINT) -> HRESULT {
    if !(*this).m_folderCM.is_null() {
        return ((*(*(*this).m_folderCM).lpVtbl).GetCommandString)(
            (*this).m_folderCM,
            idCommand,
            uFlags,
            lpReserved,
            lpszName,
            uMaxNameLen,
        );
    }

    E_NOTIMPL
}

pub unsafe fn CDefViewBckgrndMenu_HandleMenuMsg_body(this: *mut CDefViewBckgrndMenu, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> HRESULT {
    if !(*this).m_folderCM.is_null() {
        let mut pfolderCM2: *mut IContextMenu2 = core::ptr::null_mut();
        let _pfolderCM2_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pfolderCM2));
        let hr = ((*(*(*this).m_folderCM).lpVtbl).QueryInterface)(
            (*this).m_folderCM,
            core::ptr::addr_of!(IID_IContextMenu2), core::ptr::addr_of_mut!(pfolderCM2).cast(),
        );
        if ((hr) < 0) {
            if !pfolderCM2.is_null() {
                ((*(*pfolderCM2).lpVtbl).Release)(pfolderCM2);
                pfolderCM2 = core::ptr::null_mut();
            }
            return hr;
        }

        let result = ((*(*pfolderCM2).lpVtbl).HandleMenuMsg)(pfolderCM2, uMsg, wParam, lParam);
        ((*(*pfolderCM2).lpVtbl).Release)(pfolderCM2);
        pfolderCM2 = core::ptr::null_mut();
        return result;
    }

    E_NOTIMPL
}

pub unsafe fn CDefViewBckgrndMenu_HandleMenuMsg2_body(this: *mut CDefViewBckgrndMenu, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, plResult: *mut LRESULT) -> HRESULT {
    if !(*this).m_folderCM.is_null() {
        let mut pfolderCM3: *mut IContextMenu3 = core::ptr::null_mut();
        let _pfolderCM3_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pfolderCM3));
        let hr = ((*(*(*this).m_folderCM).lpVtbl).QueryInterface)(
            (*this).m_folderCM,
            core::ptr::addr_of!(IID_IContextMenu3), core::ptr::addr_of_mut!(pfolderCM3).cast(),
        );
        if ((hr) < 0) {
            if !pfolderCM3.is_null() {
                ((*(*pfolderCM3).lpVtbl).Release)(pfolderCM3);
                pfolderCM3 = core::ptr::null_mut();
            }
            return hr;
        }

        let result = ((*(*pfolderCM3).lpVtbl).HandleMenuMsg2)(
            pfolderCM3,
            uMsg,
            wParam,
            lParam,
            plResult,
        );
        ((*(*pfolderCM3).lpVtbl).Release)(pfolderCM3);
        pfolderCM3 = core::ptr::null_mut();
        return result;
    }

    E_NOTIMPL
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     shell32
//  * LICENSE:     GPL - See COPYING in the top level directory
//  * FILE:        dll/win32/shell32/CDefViewBckgrndMenu.cpp
//  * PURPOSE:     background context menu of the CDefView
//  * PROGRAMMERS: Giannis Adamopoulos
//  */
//
// #include <precomp.h>
//
// WINE_DEFAULT_DEBUG_CHANNEL(shell);
//
// class CDefViewBckgrndMenu :
//     public CComObjectRootEx<CComMultiThreadModelNoCS>,
//     public IContextMenu3,
//     public IObjectWithSite
// {
//     private:
//         CComPtr<IUnknown> m_site;
//         CComPtr<IShellFolder> m_psf;
//         CComPtr<IContextMenu> m_folderCM;
//
//         UINT m_idCmdFirst;
//         UINT m_LastFolderCMId;
//
//         BOOL _bIsDesktopBrowserMenu();
//         BOOL _bCanPaste();
//     public:
//         CDefViewBckgrndMenu();
//         ~CDefViewBckgrndMenu();
//         HRESULT Initialize(IShellFolder* psf);
//
//         // IContextMenu
//         STDMETHOD(QueryContextMenu)(HMENU hMenu, UINT indexMenu, UINT idCmdFirst, UINT idCmdLast, UINT uFlags) override;
//         STDMETHOD(InvokeCommand)(LPCMINVOKECOMMANDINFO lpcmi) override;
//         STDMETHOD(GetCommandString)(UINT_PTR idCommand, UINT uFlags, UINT *lpReserved, LPSTR lpszName, UINT uMaxNameLen) override;
//
//         // IContextMenu2
//         STDMETHOD(HandleMenuMsg)(UINT uMsg, WPARAM wParam, LPARAM lParam) override;
//
//         // IContextMenu3
//         STDMETHOD(HandleMenuMsg2)(UINT uMsg, WPARAM wParam, LPARAM lParam, LRESULT *plResult) override;
//
//         // IObjectWithSite
//         STDMETHOD(SetSite)(IUnknown *pUnkSite) override;
//         STDMETHOD(GetSite)(REFIID riid, void **ppvSite) override;
//
//         BEGIN_COM_MAP(CDefViewBckgrndMenu)
//         COM_INTERFACE_ENTRY_IID(IID_IContextMenu, IContextMenu)
//         COM_INTERFACE_ENTRY_IID(IID_IContextMenu2, IContextMenu2)
//         COM_INTERFACE_ENTRY_IID(IID_IContextMenu3, IContextMenu3)
//         COM_INTERFACE_ENTRY_IID(IID_IObjectWithSite, IObjectWithSite)
//         END_COM_MAP()
// };
//
// BOOL CDefViewBckgrndMenu::_bIsDesktopBrowserMenu()
// {
//     if (!m_site)
//         return FALSE;
//
//     /* Get a pointer to the shell browser */
//     CComPtr<IShellView> psv;
//     HRESULT hr = IUnknown_QueryService(m_site, SID_SFolderView, IID_PPV_ARG(IShellView, &psv));
//     if (FAILED_UNEXPECTEDLY(hr))
//         return FALSE;
//
//     FOLDERSETTINGS FolderSettings;
//     hr = psv->GetCurrentInfo(&FolderSettings);
//     if (FAILED_UNEXPECTEDLY(hr))
//         return FALSE;
//
//     return ((FolderSettings.fFlags & FWF_DESKTOP) == FWF_DESKTOP);
// }
// BOOL CDefViewBckgrndMenu::_bCanPaste()
// {
//     /* If the folder doesn't have a drop target we can't paste */
//     CComPtr<IDropTarget> pdt;
//     HRESULT hr = m_psf->CreateViewObject(NULL, IID_PPV_ARG(IDropTarget, &pdt));
//     if (FAILED(hr))
//         return FALSE;
//
//     /* We can only paste if CFSTR_SHELLIDLIST is present in the clipboard */
//     CComPtr<IDataObject> pDataObj;
//     hr = OleGetClipboard(&pDataObj);
//     if (FAILED(hr))
//         return FALSE;
//
//     STGMEDIUM medium;
//     FORMATETC formatetc;
//
//     /* Set the FORMATETC structure*/
//     InitFormatEtc(&mut formatetc, RegisterClipboardFormatW(CFSTR_SHELLIDLISTW.as_ptr()), TYMED_HGLOBAL);
//     hr = pDataObj->GetData(&formatetc, &medium);
//     if (FAILED(hr))
//         return FALSE;
//
//     ReleaseStgMedium(&medium);
//     return TRUE;
// }
// HRESULT
// CDefViewBckgrndMenu::Initialize(IShellFolder* psf)
// {
//     m_psf = psf;
//
//     /* Get the context menu of the folder. Do it here because someone may call
//        InvokeCommand without calling QueryContextMenu. It is fine if this fails */
//     m_psf->CreateViewObject(NULL, IID_PPV_ARG(IContextMenu, &m_folderCM));
//
//     return S_OK;
// }
// HRESULT
// WINAPI
// CDefViewBckgrndMenu::SetSite(IUnknown *pUnkSite)
// {
//     m_site = pUnkSite;
//
//     if(m_folderCM)
//         IUnknown_SetSite(m_folderCM, pUnkSite);
//
//     return S_OK;
// }
// HRESULT
// WINAPI
// CDefViewBckgrndMenu::GetSite(REFIID riid, void **ppvSite)
// {
//     if (!m_site)
//         return E_FAIL;
//
//     return m_site->QueryInterface(riid, ppvSite);
// }
// HRESULT
// WINAPI
// CDefViewBckgrndMenu::QueryContextMenu(HMENU hMenu, UINT indexMenu, UINT idCmdFirst, UINT idCmdLast, UINT uFlags)
// {
//     HRESULT hr;
//     HMENU hMenuPart;
//     UINT cIds = 0;
//
//     /* This is something the implementations of IContextMenu should never really do.
//        However CDefViewBckgrndMenu is more or less an overengineering result, its code could really be part of the
//        CDefView. Given this, I think that abusing the interface here is not that bad since only CDefView is the ony
//        user of this class. Here we need to do two things to keep things as simple as possible.
//        First we want the menu part added by the shell folder to be the first to add so as to make as few id translations
//        as possible. Second, we want to add the default part of the background menu without shifted ids, so as
//        to let the CDefView fill some parts like filling the arrange modes or checking the view mode. In order
//        for that to work we need to save idCmdFirst because our caller will pass id offsets to InvokeCommand.
//        This makes it impossible to concatenate the CDefViewBckgrndMenu with other menus since it abuses IContextMenu
//        but as stated above, its sole user is CDefView and should really be that way. */
//     m_idCmdFirst = idCmdFirst;
//
//     /* Let the shell folder add any items it wants to add in the background context menu */
//     if (m_folderCM)
//     {
//         hr = m_folderCM->QueryContextMenu(hMenu, indexMenu, idCmdFirst, idCmdLast, uFlags);
//         if (SUCCEEDED(hr))
//         {
//             m_LastFolderCMId = LOWORD(hr);
//             cIds = m_LastFolderCMId;
//         }
//         else
//         {
//             WARN("QueryContextMenu failed!\n");
//         }
//     }
//     else
//     {
//         WARN("GetUIObjectOf didn't give any context menu!\n");
//     }
//
//     /* Load the default part of the background context menu */
//     hMenuPart = LoadMenuW(shell32_hInstance, L"MENU_002");
//     if (hMenuPart)
//     {
//         /* Don't show the view submenu for the desktop */
//         if (_bIsDesktopBrowserMenu())
//         {
//             DeleteMenu(hMenuPart, FCIDM_SHVIEW_VIEW, MF_BYCOMMAND);
//         }
//
//         /* Disable the paste options if it is not possible */
//         if (!_bCanPaste())
//         {
//             EnableMenuItem(hMenuPart, FCIDM_SHVIEW_INSERT, MF_BYCOMMAND | MF_GRAYED);
//             EnableMenuItem(hMenuPart, FCIDM_SHVIEW_INSERTLINK, MF_BYCOMMAND | MF_GRAYED);
//         }
//
//         /* merge general background context menu in */
//         Shell_MergeMenus(hMenu, GetSubMenu(hMenuPart, 0), indexMenu, 0, idCmdLast, MM_DONTREMOVESEPS | MM_SUBMENUSHAVEIDS | MM_ADDSEPARATOR);
//         DestroyMenu(hMenuPart);
//     }
//     else
//     {
//         ERR("Failed to load menu from resource!\n");
//     }
//
//     return MAKE_HRESULT(SEVERITY_SUCCESS, 0, cIds);
// }
// HRESULT
// WINAPI
// CDefViewBckgrndMenu::InvokeCommand(LPCMINVOKECOMMANDINFO lpcmi)
// {
//     UINT idCmd = LOWORD(lpcmi->lpVerb);
//
//     if (HIWORD(lpcmi->lpVerb) && !strcmp(lpcmi->lpVerb, CMDSTR_VIEWLISTA))
//     {
//         idCmd = FCIDM_SHVIEW_LISTVIEW;
//     }
//     else if (HIWORD(lpcmi->lpVerb) && !strcmp(lpcmi->lpVerb, CMDSTR_VIEWDETAILSA))
//     {
//         idCmd = FCIDM_SHVIEW_REPORTVIEW;
//     }
//     else if(HIWORD(lpcmi->lpVerb) != 0 || idCmd < m_LastFolderCMId)
//     {
//         if (m_folderCM)
//         {
//             return m_folderCM->InvokeCommand(lpcmi);
//         }
//         WARN("m_folderCM is NULL!\n");
//         return E_NOTIMPL;
//     }
//     else
//     {
//         /* The default part of the background menu doesn't have shifted ids so we need to convert the id offset to the real id */
//         idCmd += m_idCmdFirst;
//     }
//
//     /* The commands that are handled by the def view are forwarded to it */
//     switch (idCmd)
//     {
//     case FCIDM_SHVIEW_INSERT:
//     case FCIDM_SHVIEW_INSERTLINK:
//         if (m_folderCM)
//         {
//             lpcmi->lpVerb = MAKEINTRESOURCEA(idCmd);
//             return m_folderCM->InvokeCommand(lpcmi);
//         }
//         WARN("m_folderCM is NULL!\n");
//         return E_NOTIMPL;
//     case FCIDM_SHVIEW_BIGICON:
//     case FCIDM_SHVIEW_SMALLICON:
//     case FCIDM_SHVIEW_LISTVIEW:
//     case FCIDM_SHVIEW_REPORTVIEW:
//     case FCIDM_SHVIEW_AUTOARRANGE:
//     case FCIDM_SHVIEW_SNAPTOGRID:
//     case FCIDM_SHVIEW_REFRESH:
//     case FCIDM_SHVIEW_ALIGNTOGRID:
//         if (!m_site)
//             return E_FAIL;
//
//         /* Get a pointer to the shell browser */
//         CComPtr<IShellView> psv;
//         HRESULT hr = IUnknown_QueryService(m_site, SID_SFolderView, IID_PPV_ARG(IShellView, &psv));
//         if (FAILED_UNEXPECTEDLY(hr))
//             return hr;
//
//         HWND hwndSV = NULL;
//         if (SUCCEEDED(psv->GetWindow(&hwndSV)))
//             SendMessageW(hwndSV, WM_COMMAND, MAKEWPARAM(idCmd, 0), 0);
//         return S_OK;
//     }
//
//     ERR("Got unknown command id %ul\n", LOWORD(lpcmi->lpVerb));
//     return E_FAIL;
// }
// HRESULT
// WINAPI
// CDefViewBckgrndMenu::GetCommandString(UINT_PTR idCommand, UINT uFlags, UINT *lpReserved, LPSTR lpszName, UINT uMaxNameLen)
// {
//     if (m_folderCM)
//     {
//         return m_folderCM->GetCommandString(idCommand, uFlags, lpReserved, lpszName, uMaxNameLen);
//     }
//
//     return E_NOTIMPL;
// }
// HRESULT
// WINAPI
// CDefViewBckgrndMenu::HandleMenuMsg(UINT uMsg, WPARAM wParam, LPARAM lParam)
// {
//     if(m_folderCM)
//     {
//         CComPtr<IContextMenu2> pfolderCM2;
//         HRESULT hr = m_folderCM->QueryInterface(IID_PPV_ARG(IContextMenu2, &pfolderCM2));
//         if (FAILED_UNEXPECTEDLY(hr))
//             return hr;
//
//         return pfolderCM2->HandleMenuMsg(uMsg, wParam, lParam);
//     }
//
//     return E_NOTIMPL;
// }
// HRESULT
// WINAPI
// CDefViewBckgrndMenu::HandleMenuMsg2(UINT uMsg, WPARAM wParam, LPARAM lParam, LRESULT *plResult)
// {
//     if(m_folderCM)
//     {
//         CComPtr<IContextMenu3> pfolderCM3;
//         HRESULT hr = m_folderCM->QueryInterface(IID_PPV_ARG(IContextMenu3, &pfolderCM3));
//         if (FAILED_UNEXPECTEDLY(hr))
//             return hr;
//
//         return pfolderCM3->HandleMenuMsg2(uMsg, wParam, lParam, plResult);
//     }
//
//     return E_NOTIMPL;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
