use crate::prelude::*;
use crate::menu_band::LPSMDATA;
// Deterministic from shobjidl.h
#[repr(C)]
pub struct IShellMenuCallback {
    pub lpVtbl: *const IShellMenuCallbackVtbl,
}
#[repr(C)]
pub struct IShellMenuCallbackVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IShellMenuCallback, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IShellMenuCallback) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IShellMenuCallback) -> ULONG,
    pub CallbackSM: unsafe extern "system" fn(This: *mut IShellMenuCallback, psmd: LPSMDATA, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> HRESULT,
}
extern "C" {
    pub static IID_IShellMenuCallback: IID;
}

// Deterministic COM scaffold for CShellMenuCallback; method bodies are generated separately.
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
pub struct CShellMenuCallback {
    pub IShellMenuCallback_iface: IShellMenuCallback,
    pub ref_count: core::sync::atomic::AtomicI32,
    pub m_hwndTray: HWND,
    pub m_pShellMenu: *mut IShellMenu,
    pub m_pBandSite: *mut IBandSite,
    pub m_pDeskBar: *mut IDeskBar,
    pub m_pTrayPriv: *mut ITrayPriv,
    pub m_psfPrograms: *mut IShellFolder,
    pub m_pidlPrograms: LPITEMIDLIST,
}

unsafe fn CShellMenuCallback_cpp_com_assign_m_pShellMenu(this: *mut CShellMenuCallback, value: *mut IShellMenu) {
    let old = (*this).m_pShellMenu;
    if old == value { return; }
    if !value.is_null() {
        let unknown = value as *mut IUnknown;
        let add_ref: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).AddRef);
        add_ref(unknown);
    }
    (*this).m_pShellMenu = value;
    if !old.is_null() {
        let unknown = old as *mut IUnknown;
        let release: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).Release);
        release(unknown);
    }
}

unsafe fn CShellMenuCallback_cpp_com_assign_m_pBandSite(this: *mut CShellMenuCallback, value: *mut IBandSite) {
    let old = (*this).m_pBandSite;
    if old == value { return; }
    if !value.is_null() {
        let unknown = value as *mut IUnknown;
        let add_ref: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).AddRef);
        add_ref(unknown);
    }
    (*this).m_pBandSite = value;
    if !old.is_null() {
        let unknown = old as *mut IUnknown;
        let release: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).Release);
        release(unknown);
    }
}

unsafe fn CShellMenuCallback_cpp_com_assign_m_pDeskBar(this: *mut CShellMenuCallback, value: *mut IDeskBar) {
    let old = (*this).m_pDeskBar;
    if old == value { return; }
    if !value.is_null() {
        let unknown = value as *mut IUnknown;
        let add_ref: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).AddRef);
        add_ref(unknown);
    }
    (*this).m_pDeskBar = value;
    if !old.is_null() {
        let unknown = old as *mut IUnknown;
        let release: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).Release);
        release(unknown);
    }
}

unsafe fn CShellMenuCallback_cpp_com_assign_m_pTrayPriv(this: *mut CShellMenuCallback, value: *mut ITrayPriv) {
    let old = (*this).m_pTrayPriv;
    if old == value { return; }
    if !value.is_null() {
        let unknown = value as *mut IUnknown;
        let add_ref: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).AddRef);
        add_ref(unknown);
    }
    (*this).m_pTrayPriv = value;
    if !old.is_null() {
        let unknown = old as *mut IUnknown;
        let release: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).Release);
        release(unknown);
    }
}

unsafe fn CShellMenuCallback_cpp_com_assign_m_psfPrograms(this: *mut CShellMenuCallback, value: *mut IShellFolder) {
    let old = (*this).m_psfPrograms;
    if old == value { return; }
    if !value.is_null() {
        let unknown = value as *mut IUnknown;
        let add_ref: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).AddRef);
        add_ref(unknown);
    }
    (*this).m_psfPrograms = value;
    if !old.is_null() {
        let unknown = old as *mut IUnknown;
        let release: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).Release);
        release(unknown);
    }
}

unsafe extern "system" fn CShellMenuCallback_IShellMenuCallback_QueryInterface(This: *mut IShellMenuCallback, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellMenuCallback, IShellMenuCallback_iface)) as *mut CShellMenuCallback;
    CShellMenuCallback_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CShellMenuCallback_IShellMenuCallback_AddRef(This: *mut IShellMenuCallback) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellMenuCallback, IShellMenuCallback_iface)) as *mut CShellMenuCallback;
    CShellMenuCallback_AddRef(this)
}
unsafe extern "system" fn CShellMenuCallback_IShellMenuCallback_Release(This: *mut IShellMenuCallback) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellMenuCallback, IShellMenuCallback_iface)) as *mut CShellMenuCallback;
    CShellMenuCallback_Release(this)
}
unsafe extern "system" fn CShellMenuCallback_IShellMenuCallback_CallbackSM(This: *mut IShellMenuCallback, psmd: LPSMDATA, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellMenuCallback, IShellMenuCallback_iface)) as *mut CShellMenuCallback;
    CShellMenuCallback_CallbackSM_body(this, psmd, uMsg, wParam, lParam)
}

pub static CSHELLMENUCALLBACK_ISHELLMENUCALLBACK_VTBL: IShellMenuCallbackVtbl = IShellMenuCallbackVtbl {
    QueryInterface: CShellMenuCallback_IShellMenuCallback_QueryInterface,
    AddRef: CShellMenuCallback_IShellMenuCallback_AddRef,
    Release: CShellMenuCallback_IShellMenuCallback_Release,
    CallbackSM: CShellMenuCallback_IShellMenuCallback_CallbackSM,
};

unsafe fn CShellMenuCallback_QueryInterface(
    this: *mut CShellMenuCallback, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IUnknown)) || cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IShellMenuCallback)) {
        *ppv = core::ptr::addr_of_mut!((*this).IShellMenuCallback_iface) as *mut core::ffi::c_void;
    }
    else { return E_NOINTERFACE; }
    CShellMenuCallback_AddRef(this);
    S_OK
}

unsafe fn CShellMenuCallback_AddRef(this: *mut CShellMenuCallback) -> ULONG {
    (*this).ref_count.fetch_add(1, core::sync::atomic::Ordering::SeqCst) as ULONG + 1
}

unsafe fn CShellMenuCallback_Release(this: *mut CShellMenuCallback) -> ULONG {
    let refs = (*this).ref_count.fetch_sub(1, core::sync::atomic::Ordering::SeqCst) - 1;
    if refs == 0 {
        CShellMenuCallback_dtor_body(this);
        CShellMenuCallback_cpp_com_assign_m_pShellMenu(this, core::ptr::null_mut());
        CShellMenuCallback_cpp_com_assign_m_pBandSite(this, core::ptr::null_mut());
        CShellMenuCallback_cpp_com_assign_m_pDeskBar(this, core::ptr::null_mut());
        CShellMenuCallback_cpp_com_assign_m_pTrayPriv(this, core::ptr::null_mut());
        CShellMenuCallback_cpp_com_assign_m_psfPrograms(this, core::ptr::null_mut());
        MenuBandRustModuleUnlock();
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

// Internal activation: this class has no CLSID; the DLL constructs it directly through
// shellutils.h ShellObjectCreator<T> or ATL's own _CreatorClass::CreateInstance, both of
// which take (outer, riid, ppv) — the signature of this factory.
pub unsafe fn CShellMenuCallback_CreateInstance(
    outer: *mut IUnknown,
    riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if !outer.is_null() { return CLASS_E_NOAGGREGATION; }
    let this = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<CShellMenuCallback>()) as *mut CShellMenuCallback;
    if this.is_null() { return E_OUTOFMEMORY; }
    core::ptr::write(this, CShellMenuCallback {
        IShellMenuCallback_iface: IShellMenuCallback { lpVtbl: core::ptr::addr_of!(CSHELLMENUCALLBACK_ISHELLMENUCALLBACK_VTBL) },
        ref_count: core::sync::atomic::AtomicI32::new(1),
        m_hwndTray: core::mem::zeroed(),
        m_pShellMenu: core::mem::zeroed(),
        m_pBandSite: core::mem::zeroed(),
        m_pDeskBar: core::mem::zeroed(),
        m_pTrayPriv: core::mem::zeroed(),
        m_psfPrograms: core::mem::zeroed(),
        m_pidlPrograms: core::mem::zeroed(),
    });
    MenuBandRustModuleLock();
    CShellMenuCallback_ctor_body(this);
    let hr = CShellMenuCallback_QueryInterface(this, riid, ppv);
    CShellMenuCallback_Release(this);
    hr
}

// Deterministic C++ support closure selected from source identifiers.

pub const IDI_SHELL_CONTROL_PANEL: UINT = 22;

pub const IDI_SHELL_CONTROL_PANEL1: UINT = 330;

pub const IDI_SHELL_FAVOTITES: UINT = 322;

pub const IDI_SHELL_HELP2: UINT = 324;

pub const IDI_SHELL_LOGOFF1: UINT = 325;

pub const IDI_SHELL_MY_DOCUMENTS: UINT = 235;

pub const IDI_SHELL_MY_PICTURES: UINT = 236;

pub const IDI_SHELL_NETWORK_CONNECTIONS2: UINT = 257;

pub const IDI_SHELL_PRINTER2: UINT = 138;

pub const IDI_SHELL_PROGRAMS_FOLDER1: UINT = 326;

pub const IDI_SHELL_RECENT_DOCUMENTS1: UINT = 327;

pub const IDI_SHELL_RUN1: UINT = 328;

pub const IDI_SHELL_SEARCH1: UINT = 323;

pub const IDI_SHELL_SHUTDOWN1: UINT = 329;

pub const IDI_SHELL_TSKBAR_STARTMENU: UINT = 40;

pub const IDM_CONTROLPANEL: UINT = 505;

pub const IDM_DISCONNECT: UINT = 5000;

pub const IDM_DOCUMENTS: UINT = 501;

pub const IDM_FAVORITES: UINT = 507;

pub const IDM_HELPANDSUPPORT: UINT = 503;

pub const IDM_LASTSTARTMENU_SEPARATOR: UINT = 450;

pub const IDM_LOGOFF: UINT = 402;

pub const IDM_MYDOCUMENTS: UINT = 516;

pub const IDM_MYPICTURES: UINT = 518;

pub const IDM_NETWORKCONNECTIONS: UINT = 557;

pub const IDM_PRINTERSANDFAXES: UINT = 510;

pub const IDM_PROGRAMS: UINT = 504;

pub const IDM_RUN: UINT = 401;

pub const IDM_SEARCH: UINT = 520;

pub const IDM_SECURITY: UINT = 5001;

pub const IDM_SETTINGS: UINT = 508;

pub const IDM_SHUTDOWN: UINT = 506;

pub const IDM_SYNCHRONIZE: UINT = 553;

pub const IDM_TASKBARANDSTARTMENU: UINT = 413;

pub const IDM_UNDOCKCOMPUTER: UINT = 410;

pub const CSIDL_PERSONAL: INT = 0x0005;

pub const SW_SHOWNORMAL: INT = 1;

pub const CSIDL_COMMON_PROGRAMS: DWORD = 0x0017;

pub const CSIDL_COMMON_STARTMENU: DWORD = 0x0016;

pub const CSIDL_CONNECTIONS: DWORD = 0x0031;

pub const CSIDL_CONTROLS: DWORD = 0x0003;

pub const CSIDL_FAVORITES: DWORD = 0x0006;

pub const CSIDL_MYPICTURES: DWORD = 0x0027;

pub const CSIDL_NETWORK: DWORD = 0x0012;

pub const CSIDL_PRINTERS: DWORD = 0x0004;

pub const CSIDL_PROGRAMS: DWORD = 0x0002;

pub const CSIDL_RECENT: DWORD = 0x0008;

pub const CSIDL_STARTMENU: DWORD = 0x000b;

pub const MF_SEPARATOR: DWORD = 0x800;

pub const MIIM_SUBMENU: DWORD = 4;

pub const REST_NOSMMYDOCS: DWORD = 0x4000004F;

pub const REST_NOSMMYPICS: DWORD = 0x40000050;

pub const SHGDN_INFOLDER: DWORD = 1;

pub const SHGFI_DISPLAYNAME: DWORD = 512;

pub const SHGFI_PIDL: DWORD = 8;

pub const SMC_EXEC: DWORD = 4;

pub const SMC_GETINFO: DWORD = 0x00000005;

pub const SMC_GETOBJECT: DWORD = 0x00000007;

pub const SMC_INITMENU: DWORD = 0x00000001;

pub const SMC_SFEXEC: DWORD = 0x00000009;

pub const SMIF_ICON: DWORD = 0x1;

pub const SMIF_TRACKPOPUP: DWORD = 0x200;

pub const SMIM_FLAGS: DWORD = 0x2;

pub const SMIM_TYPE: DWORD = 0x1;

pub const SMINIT_TOPLEVEL: DWORD = 0x00000004;

pub const SMINIT_VERTICAL: DWORD = 0x10000000;

pub const SMIT_SEPARATOR: DWORD = 0x1;

pub const SMSET_BOTTOM: DWORD = 0x20000000;

pub const SMSET_TOP: DWORD = 0x10000000;

pub const WM_COMMAND: DWORD = 273;

#[repr(C, packed)]
pub struct ITEMIDLIST {
    pub mkid: SHITEMID,
}

#[repr(C, packed)]
pub struct SHITEMID {
    pub cb: WORD,
    pub abID: [BYTE; 1],
}

#[repr(C)]
pub struct SMINFO {
    pub dwMask: DWORD,
    pub dwType: DWORD,
    pub dwFlags: DWORD,
    pub iIcon: i32,
}

// Deterministic from shobjidl.h
#[repr(C)]
pub struct IBandSite {
    pub lpVtbl: *const IBandSiteVtbl,
}
#[repr(C)]
pub struct IBandSiteVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IBandSite, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IBandSite) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IBandSite) -> ULONG,
    pub AddBand: unsafe extern "system" fn(This: *mut IBandSite, punk: *mut IUnknown) -> HRESULT,
    pub EnumBands: unsafe extern "system" fn(This: *mut IBandSite, uBand: UINT, pdwBandID: *mut DWORD) -> HRESULT,
    pub QueryBand: unsafe extern "system" fn(This: *mut IBandSite, dwBandID: DWORD, ppstb: *mut *mut IDeskBand, pdwState: *mut DWORD, pszName: LPWSTR, cchName: i32) -> HRESULT,
    pub SetBandState: unsafe extern "system" fn(This: *mut IBandSite, dwBandID: DWORD, dwMask: DWORD, dwState: DWORD) -> HRESULT,
    pub RemoveBand: unsafe extern "system" fn(This: *mut IBandSite, dwBandID: DWORD) -> HRESULT,
    pub GetBandObject: unsafe extern "system" fn(This: *mut IBandSite, dwBandID: DWORD, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT,
    pub SetBandSiteInfo: unsafe extern "system" fn(This: *mut IBandSite, pbsinfo: *const BANDSITEINFO) -> HRESULT,
    pub GetBandSiteInfo: unsafe extern "system" fn(This: *mut IBandSite, pbsinfo: *mut BANDSITEINFO) -> HRESULT,
}
extern "C" {
    pub static IID_IBandSite: IID;
}

// Deterministic from shobjidl.h
#[repr(C)]
pub struct IDeskBand {
    pub lpVtbl: *const IDeskBandVtbl,
}
#[repr(C)]
pub struct IDeskBandVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IDeskBand, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IDeskBand) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IDeskBand) -> ULONG,
    pub GetWindow: unsafe extern "system" fn(This: *mut IDeskBand, phwnd: *mut HWND) -> HRESULT,
    pub ContextSensitiveHelp: unsafe extern "system" fn(This: *mut IDeskBand, fEnterMode: BOOL) -> HRESULT,
    pub ShowDW: unsafe extern "system" fn(This: *mut IDeskBand, fShow: BOOL) -> HRESULT,
    pub CloseDW: unsafe extern "system" fn(This: *mut IDeskBand, dwReserved: DWORD) -> HRESULT,
    pub ResizeBorderDW: unsafe extern "system" fn(This: *mut IDeskBand, prcBorder: LPCRECT, punkToolbarSite: *mut IUnknown, fReserved: BOOL) -> HRESULT,
    pub GetBandInfo: unsafe extern "system" fn(This: *mut IDeskBand, dwBandID: DWORD, dwViewMode: DWORD, pdbi: *mut DESKBANDINFO) -> HRESULT,
}
extern "C" {
    pub static IID_IDeskBand: IID;
}

// Deterministic from shobjidl.h
#[repr(C)]
pub struct IDeskBar {
    pub lpVtbl: *const IDeskBarVtbl,
}
#[repr(C)]
pub struct IDeskBarVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IDeskBar, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IDeskBar) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IDeskBar) -> ULONG,
    pub GetWindow: unsafe extern "system" fn(This: *mut IDeskBar, phwnd: *mut HWND) -> HRESULT,
    pub ContextSensitiveHelp: unsafe extern "system" fn(This: *mut IDeskBar, fEnterMode: BOOL) -> HRESULT,
    pub SetClient: unsafe extern "system" fn(This: *mut IDeskBar, punkClient: *mut IUnknown) -> HRESULT,
    pub GetClient: unsafe extern "system" fn(This: *mut IDeskBar, ppunkClient: *mut *mut IUnknown) -> HRESULT,
    pub OnPosRectChangeDB: unsafe extern "system" fn(This: *mut IDeskBar, prc: *mut RECT) -> HRESULT,
}
extern "C" {
    pub static IID_IDeskBar: IID;
}

// Deterministic from shobjidl.h
#[repr(C)]
pub struct IShellMenu {
    pub lpVtbl: *const IShellMenuVtbl,
}
#[repr(C)]
pub struct IShellMenuVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IShellMenu, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IShellMenu) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IShellMenu) -> ULONG,
    pub Initialize: unsafe extern "system" fn(This: *mut IShellMenu, psmc: *mut IShellMenuCallback, uId: UINT, uIdAncestor: UINT, dwFlags: DWORD) -> HRESULT,
    pub GetMenuInfo: unsafe extern "system" fn(This: *mut IShellMenu, ppsmc: *mut *mut IShellMenuCallback, puId: *mut UINT, puIdAncestor: *mut UINT, pdwFlags: *mut DWORD) -> HRESULT,
    pub SetShellFolder: unsafe extern "system" fn(This: *mut IShellMenu, psf: *mut IShellFolder, pidlFolder: PCIDLIST_ABSOLUTE, hKey: HKEY, dwFlags: DWORD) -> HRESULT,
    pub GetShellFolder: unsafe extern "system" fn(This: *mut IShellMenu, pdwFlags: *mut DWORD, ppidl: *mut PIDLIST_ABSOLUTE, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT,
    pub SetMenu: unsafe extern "system" fn(This: *mut IShellMenu, hmenu: HMENU, hwnd: HWND, dwFlags: DWORD) -> HRESULT,
    pub GetMenu: unsafe extern "system" fn(This: *mut IShellMenu, phmenu: *mut HMENU, phwnd: *mut HWND, pdwFlags: *mut DWORD) -> HRESULT,
    pub InvalidateItem: unsafe extern "system" fn(This: *mut IShellMenu, psmd: LPSMDATA, dwFlags: DWORD) -> HRESULT,
    pub GetState: unsafe extern "system" fn(This: *mut IShellMenu, psmd: LPSMDATA) -> HRESULT,
    pub SetMenuToolbar: unsafe extern "system" fn(This: *mut IShellMenu, punk: *mut IUnknown, dwFlags: DWORD) -> HRESULT,
}
extern "C" {
    pub static IID_IShellMenu: IID;
}

// Deterministic from shlobj_undoc.h
#[repr(C)]
pub struct ITrayPriv {
    pub lpVtbl: *const ITrayPrivVtbl,
}
#[repr(C)]
pub struct ITrayPrivVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut ITrayPriv, arg1: REFIID, arg2: *mut PVOID) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut ITrayPriv) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut ITrayPriv) -> ULONG,
    pub GetWindow: unsafe extern "system" fn(This: *mut ITrayPriv, arg1: *mut HWND) -> HRESULT,
    pub ContextSensitiveHelp: unsafe extern "system" fn(This: *mut ITrayPriv, arg1: BOOL) -> HRESULT,
    pub Execute: unsafe extern "system" fn(This: *mut ITrayPriv, arg1: *mut IShellFolder, arg2: LPCITEMIDLIST) -> HRESULT,
    pub Unknown: unsafe extern "system" fn(This: *mut ITrayPriv, arg1: PVOID, arg2: PVOID, arg3: PVOID, arg4: PVOID) -> HRESULT,
    pub AppendMenu: unsafe extern "system" fn(This: *mut ITrayPriv, arg1: *mut HMENU) -> HRESULT,
}
extern "C" {
    pub static IID_ITrayPriv: IID;
}

#[repr(C)]
pub struct BANDSITEINFO {
    pub dwMask: DWORD,
    pub dwState: DWORD,
    pub dwStyle: DWORD,
}

extern "system" {
    fn CreateMenu() -> HMENU;
    fn DestroyMenu(menu: HMENU) -> BOOL;
    fn AppendMenuW(menu: HMENU, flags: UINT, id: UINT_PTR, text: LPCWSTR) -> BOOL;
    fn SetMenuItemInfoW(menu: HMENU, item: UINT, by_position: BOOL, info: *const MENUITEMINFOW) -> BOOL;
    fn GetMenuItemInfoW(menu: HMENU, item: UINT, by_position: BOOL, info: *mut MENUITEMINFOW) -> BOOL;
    fn SHGetFileInfoW(path: LPCWSTR, attributes: DWORD, info: *mut SHFILEINFOW, bytes: UINT, flags: UINT) -> DWORD_PTR;
    fn CoTaskMemFree(memory: LPVOID);
    fn ILFree(pidl: LPITEMIDLIST);
    fn SHGetDesktopFolder(folder: *mut *mut IShellFolder) -> HRESULT;
    fn SHGetSpecialFolderLocation(owner: HWND, csidl: INT, pidl: *mut LPITEMIDLIST) -> HRESULT;
    fn InsertMenuItemW(menu: HMENU, item: UINT, by_position: BOOL, info: *const MENUITEMINFOW) -> BOOL;
}

#[repr(transparent)]
pub struct CComHeapPtr<T> { ptr: *mut T }
impl<T> CComHeapPtr<T> {
    pub const fn empty() -> Self { Self { ptr: core::ptr::null_mut() } }
    pub const fn new(ptr: *mut T) -> Self { Self { ptr } }
    pub const fn as_raw(&self) -> *mut T { self.ptr }
    pub fn as_mut_slot(&mut self) -> *mut *mut T { core::ptr::addr_of_mut!(self.ptr) }
    pub unsafe fn Attach(&mut self, ptr: *mut T) {
        if !self.ptr.is_null() { CoTaskMemFree(self.ptr.cast()); }
        self.ptr = ptr;
    }
    pub fn Detach(&mut self) -> *mut T {
        let ptr = self.ptr; self.ptr = core::ptr::null_mut(); ptr
    }
}
impl<T> Drop for CComHeapPtr<T> {
    fn drop(&mut self) {
        unsafe { if !self.ptr.is_null() { CoTaskMemFree(self.ptr.cast()); } }
    }
}

extern "C" {
    static IID_IAugmentedShellFolder: GUID;
    static IID_IContextMenu: GUID;
    static IID_IShellFolder: GUID;
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

// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_1AA6B1B3DCF4: [WCHAR; 26] = [0x0043, 0x0061, 0x0073, 0x0063, 0x0061, 0x0064, 0x0065, 0x004E, 0x0065, 0x0074, 0x0077, 0x006F, 0x0072, 0x006B, 0x0043, 0x006F, 0x006E, 0x006E, 0x0065, 0x0063, 0x0074, 0x0069, 0x006F, 0x006E, 0x0073, 0x0000];
pub static CPP_COM_WSTR_382D520E580E: [WCHAR; 16] = [0x0043, 0x0061, 0x0073, 0x0063, 0x0061, 0x0064, 0x0065, 0x0050, 0x0072, 0x0069, 0x006E, 0x0074, 0x0065, 0x0072, 0x0073, 0x0000];
pub static CPP_COM_WSTR_404F53961224: [WCHAR; 18] = [0x0043, 0x0061, 0x0073, 0x0063, 0x0061, 0x0064, 0x0065, 0x004D, 0x0079, 0x0050, 0x0069, 0x0063, 0x0074, 0x0075, 0x0072, 0x0065, 0x0073, 0x0000];
pub static CPP_COM_WSTR_436ECA13AB23: [WCHAR; 20] = [0x0043, 0x0061, 0x0073, 0x0063, 0x0061, 0x0064, 0x0065, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F, 0x006C, 0x0050, 0x0061, 0x006E, 0x0065, 0x006C, 0x0000];
pub static CPP_COM_WSTR_8888EF3C3D8B: [WCHAR; 60] = [0x0053, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x004D, 0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057, 0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x005C, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069, 0x006F, 0x006E, 0x005C, 0x0045, 0x0078, 0x0070, 0x006C, 0x006F, 0x0072, 0x0065, 0x0072, 0x005C, 0x0041, 0x0064, 0x0076, 0x0061, 0x006E, 0x0063, 0x0065, 0x0064, 0x0000];
pub static CPP_COM_WSTR_AB68B18B313D: [WCHAR; 13] = [0x0065, 0x0078, 0x0070, 0x006C, 0x006F, 0x0072, 0x0065, 0x0072, 0x002E, 0x0065, 0x0078, 0x0065, 0x0000];
pub static CPP_COM_WSTR_C07A84B63277: [WCHAR; 82] = [0x003A, 0x003A, 0x007B, 0x0032, 0x0030, 0x0044, 0x0030, 0x0034, 0x0046, 0x0045, 0x0030, 0x002D, 0x0033, 0x0041, 0x0045, 0x0041, 0x002D, 0x0031, 0x0030, 0x0036, 0x0039, 0x002D, 0x0041, 0x0032, 0x0044, 0x0038, 0x002D, 0x0030, 0x0038, 0x0030, 0x0030, 0x0032, 0x0042, 0x0033, 0x0030, 0x0033, 0x0030, 0x0039, 0x0044, 0x007D, 0x005C, 0x003A, 0x003A, 0x007B, 0x0032, 0x0031, 0x0045, 0x0043, 0x0032, 0x0030, 0x0032, 0x0030, 0x002D, 0x0033, 0x0041, 0x0045, 0x0041, 0x002D, 0x0031, 0x0030, 0x0036, 0x0039, 0x002D, 0x0041, 0x0032, 0x0044, 0x0044, 0x002D, 0x0030, 0x0038, 0x0030, 0x0030, 0x0032, 0x0042, 0x0033, 0x0030, 0x0033, 0x0030, 0x0039, 0x0044, 0x007D, 0x0000];
pub static CPP_COM_WSTR_CC2776C3CD78: [WCHAR; 17] = [0x0053, 0x0074, 0x0061, 0x0072, 0x0074, 0x005F, 0x0053, 0x0068, 0x006F, 0x0077, 0x004D, 0x0079, 0x0050, 0x0069, 0x0063, 0x0073, 0x0000];
pub static CPP_COM_WSTR_D30E8899E227: [WCHAR; 12] = [0x0073, 0x0068, 0x0065, 0x006C, 0x006C, 0x0033, 0x0032, 0x002E, 0x0064, 0x006C, 0x006C, 0x0000];
pub static CPP_COM_WSTR_D44AF584A75E: [WCHAR; 123] = [0x003A, 0x003A, 0x007B, 0x0032, 0x0030, 0x0044, 0x0030, 0x0034, 0x0046, 0x0045, 0x0030, 0x002D, 0x0033, 0x0041, 0x0045, 0x0041, 0x002D, 0x0031, 0x0030, 0x0036, 0x0039, 0x002D, 0x0041, 0x0032, 0x0044, 0x0038, 0x002D, 0x0030, 0x0038, 0x0030, 0x0030, 0x0032, 0x0042, 0x0033, 0x0030, 0x0033, 0x0030, 0x0039, 0x0044, 0x007D, 0x005C, 0x003A, 0x003A, 0x007B, 0x0032, 0x0031, 0x0045, 0x0043, 0x0032, 0x0030, 0x0032, 0x0030, 0x002D, 0x0033, 0x0041, 0x0045, 0x0041, 0x002D, 0x0031, 0x0030, 0x0036, 0x0039, 0x002D, 0x0041, 0x0032, 0x0044, 0x0044, 0x002D, 0x0030, 0x0038, 0x0030, 0x0030, 0x0032, 0x0042, 0x0033, 0x0030, 0x0033, 0x0030, 0x0039, 0x0044, 0x007D, 0x005C, 0x003A, 0x003A, 0x007B, 0x0037, 0x0030, 0x0030, 0x0037, 0x0041, 0x0043, 0x0043, 0x0037, 0x002D, 0x0033, 0x0032, 0x0030, 0x0032, 0x002D, 0x0031, 0x0031, 0x0044, 0x0031, 0x002D, 0x0041, 0x0041, 0x0044, 0x0032, 0x002D, 0x0030, 0x0030, 0x0038, 0x0030, 0x0035, 0x0046, 0x0043, 0x0031, 0x0032, 0x0037, 0x0030, 0x0045, 0x007D, 0x0000];
pub static CPP_COM_WSTR_DF562C7825A2: [WCHAR; 19] = [0x0043, 0x0061, 0x0073, 0x0063, 0x0061, 0x0064, 0x0065, 0x004D, 0x0079, 0x0044, 0x006F, 0x0063, 0x0075, 0x006D, 0x0065, 0x006E, 0x0074, 0x0073, 0x0000];
pub static CPP_COM_WSTR_E7B500FE6D96: [WCHAR; 17] = [0x0053, 0x0074, 0x0061, 0x0072, 0x0074, 0x005F, 0x0053, 0x0068, 0x006F, 0x0077, 0x004D, 0x0079, 0x0044, 0x006F, 0x0063, 0x0073, 0x0000];
pub static CPP_COM_WSTR_FB50816BF2F8: [WCHAR; 123] = [0x003A, 0x003A, 0x007B, 0x0032, 0x0030, 0x0044, 0x0030, 0x0034, 0x0046, 0x0045, 0x0030, 0x002D, 0x0033, 0x0041, 0x0045, 0x0041, 0x002D, 0x0031, 0x0030, 0x0036, 0x0039, 0x002D, 0x0041, 0x0032, 0x0044, 0x0038, 0x002D, 0x0030, 0x0038, 0x0030, 0x0030, 0x0032, 0x0042, 0x0033, 0x0030, 0x0033, 0x0030, 0x0039, 0x0044, 0x007D, 0x005C, 0x003A, 0x003A, 0x007B, 0x0032, 0x0031, 0x0045, 0x0043, 0x0032, 0x0030, 0x0032, 0x0030, 0x002D, 0x0033, 0x0041, 0x0045, 0x0041, 0x002D, 0x0031, 0x0030, 0x0036, 0x0039, 0x002D, 0x0041, 0x0032, 0x0044, 0x0044, 0x002D, 0x0030, 0x0038, 0x0030, 0x0030, 0x0032, 0x0042, 0x0033, 0x0030, 0x0033, 0x0030, 0x0039, 0x0044, 0x007D, 0x005C, 0x003A, 0x003A, 0x007B, 0x0032, 0x0032, 0x0032, 0x0037, 0x0041, 0x0032, 0x0038, 0x0030, 0x002D, 0x0033, 0x0041, 0x0045, 0x0041, 0x002D, 0x0031, 0x0030, 0x0036, 0x0039, 0x002D, 0x0041, 0x0032, 0x0044, 0x0045, 0x002D, 0x0030, 0x0038, 0x0030, 0x0030, 0x0032, 0x0042, 0x0033, 0x0030, 0x0033, 0x0030, 0x0039, 0x0044, 0x007D, 0x0000];

use core::ffi::c_void;

pub unsafe fn CShellMenuCallback_ctor_body(this: *mut CShellMenuCallback) -> () {}

pub unsafe fn CShellMenuCallback_OnInitMenu_body(this: *mut CShellMenuCallback) -> HRESULT {
    let mut hmenu: HMENU = core::ptr::null_mut();
    let mut hr: HRESULT;

    if !((*this).m_pTrayPriv).is_null() {
        return S_OK;
    }

    // IUnknown_GetSite/IUnknown_GetWindow take `*mut IUnknown` (com_sys's canonical type,
    // not a nominal split); IDeskBar/ITrayPriv upcast to it the same way the C++ implicitly
    // upcasts CComPtr<IDeskBar>/CComPtr<ITrayPriv> -- AGENTS.md rule 1.
    hr = IUnknown_GetSite((*this).m_pDeskBar.cast(), core::ptr::addr_of!(IID_ITrayPriv), core::ptr::addr_of_mut!((*this).m_pTrayPriv).cast());
    if ((hr) < 0) {
        return hr;
    }

    hr = IUnknown_GetWindow((*this).m_pTrayPriv.cast(), &mut (*this).m_hwndTray);
    if ((hr) < 0) {
        return hr;
    }

    hr = ((*(*(*this).m_pTrayPriv).lpVtbl).AppendMenu)((*this).m_pTrayPriv, &mut hmenu);
    if ((hr) < 0) {
        return hr;
    }

    hr = ((*(*(*this).m_pShellMenu).lpVtbl).SetMenu)((*this).m_pShellMenu, hmenu, core::ptr::null_mut(), SMSET_BOTTOM);
    if ((hr) < 0) {
        DestroyMenu(hmenu);
        return hr;
    }

    return hr;
}

pub unsafe fn CShellMenuCallback_OnGetInfo_body(this: *mut CShellMenuCallback, psmd: LPSMDATA, psminfo: *mut SMINFO) -> HRESULT {
    let mut iconIndex: i32 = 0;

    match (*psmd).uId {
        IDM_FAVORITES => iconIndex = -(IDI_SHELL_FAVOTITES as i32),
        IDM_SEARCH => iconIndex = -(IDI_SHELL_SEARCH1 as i32),
        IDM_HELPANDSUPPORT => iconIndex = -(IDI_SHELL_HELP2 as i32),
        IDM_LOGOFF => iconIndex = -(IDI_SHELL_LOGOFF1 as i32),
        IDM_PROGRAMS => iconIndex = -(IDI_SHELL_PROGRAMS_FOLDER1 as i32),
        IDM_DOCUMENTS => iconIndex = -(IDI_SHELL_RECENT_DOCUMENTS1 as i32),
        IDM_RUN => iconIndex = -(IDI_SHELL_RUN1 as i32),
        IDM_SHUTDOWN => iconIndex = -(IDI_SHELL_SHUTDOWN1 as i32),
        IDM_SETTINGS => iconIndex = -(IDI_SHELL_CONTROL_PANEL1 as i32),
        IDM_MYDOCUMENTS => iconIndex = -(IDI_SHELL_MY_DOCUMENTS as i32),
        IDM_MYPICTURES => iconIndex = -(IDI_SHELL_MY_PICTURES as i32),
        IDM_CONTROLPANEL => iconIndex = -(IDI_SHELL_CONTROL_PANEL as i32),
        IDM_NETWORKCONNECTIONS => iconIndex = -(IDI_SHELL_NETWORK_CONNECTIONS2 as i32),
        IDM_PRINTERSANDFAXES => iconIndex = -(IDI_SHELL_PRINTER2 as i32),
        IDM_TASKBARANDSTARTMENU => iconIndex = -(IDI_SHELL_TSKBAR_STARTMENU as i32),
        _ => return S_FALSE,
    }

    if iconIndex != 0 {
        if ((*psminfo).dwMask & SMIM_TYPE) != 0 {
            (*psminfo).dwType = SMIT_STRING;
        }
        if ((*psminfo).dwMask & SMIM_ICON) != 0 {
            (*psminfo).iIcon = Shell_GetCachedImageIndex(CPP_COM_WSTR_D30E8899E227.as_ptr(), iconIndex, FALSE as BOOL);
        }
        if ((*psminfo).dwMask & SMIM_FLAGS) != 0 {
            (*psminfo).dwFlags |= SMIF_ICON;
        }
    } else {
        if ((*psminfo).dwMask & SMIM_TYPE) != 0 {
            (*psminfo).dwType = SMIT_SEPARATOR;
        }
    }

    return S_OK;
}

pub unsafe fn CShellMenuCallback_AddOrSetMenuItem_body(this: *mut CShellMenuCallback, hMenu: HMENU, nID: UINT, csidl: INT, bExpand: BOOL, bAdd: BOOL, bSetText: BOOL) -> () {
    let _ = this;
    let mut mii: MENUITEMINFOW = core::mem::zeroed();
    mii.cbSize = core::mem::size_of::<MENUITEMINFOW>() as u32;
    mii.fMask = MIIM_ID | MIIM_SUBMENU;
    mii.wID = nID;

    let mut fileInfo: SHFILEINFOW = core::mem::zeroed();
    if bAdd != 0 || bSetText != 0 {
        let mut pidl: LPITEMIDLIST = core::ptr::null_mut();
        if SHGetSpecialFolderLocation(
            core::ptr::null_mut(),
            csidl,
            &mut pidl as *mut LPITEMIDLIST
        ) != S_OK {
            return;
        }

        SHGetFileInfoW(
            pidl as *const u16,
            0,
            &mut fileInfo as *mut SHFILEINFOW,
            core::mem::size_of::<SHFILEINFOW>() as u32,
            SHGFI_PIDL | SHGFI_DISPLAYNAME
        );
        CoTaskMemFree(pidl as *mut core::ffi::c_void);

        mii.fMask |= MIIM_TYPE;
        mii.fType = MFT_STRING;
        // `addr_of_mut!`, not `.as_mut_ptr()`: SHFILEINFOW is `#[repr(packed)]` in the i386
        // island (and only there), so `.as_mut_ptr()` -- which goes through a `&mut` to the
        // field -- is E0793 on that arch alone. The C++ `mii.dwTypeData = fileInfo.szDisplayName`
        // is an array-to-pointer decay that never forms a reference, and this is that.
        mii.dwTypeData = core::ptr::addr_of_mut!(fileInfo.szDisplayName) as *mut WCHAR;
    }

    if bExpand != 0 {
        mii.hSubMenu = CreatePopupMenu();
    }

    if bAdd != 0 {
        InsertMenuItemW(
            hMenu,
            GetMenuItemCount(hMenu) as UINT,
            TRUE as BOOL,
            &mii as *const MENUITEMINFOW
        );
    } else {
        SetMenuItemInfoW(
            hMenu,
            nID,
            FALSE as BOOL,
            &mii as *const MENUITEMINFOW
        );
    }
}

pub unsafe fn CShellMenuCallback_GetAdvancedValue_body(this: *mut CShellMenuCallback, pszName: LPCWSTR, bDefault: BOOL) -> BOOL {
    return SHRegGetBoolUSValueW(
        CPP_COM_WSTR_8888EF3C3D8B.as_ptr(),
        pszName,
        FALSE as BOOL,
        bDefault,
    );
}

pub unsafe fn CShellMenuCallback_CreateRecentMenu_body(this: *mut CShellMenuCallback) -> HMENU {
    let hMenu: HMENU = CreateMenu();
    let mut bAdded: BOOL = FALSE as BOOL;

    if SHRestricted(REST_NOSMMYDOCS) == 0 &&
       CShellMenuCallback_GetAdvancedValue_body(this, CPP_COM_WSTR_E7B500FE6D96.as_ptr(), TRUE as BOOL) != 0 {
        let bExpand: BOOL = CShellMenuCallback_GetAdvancedValue_body(this, CPP_COM_WSTR_DF562C7825A2.as_ptr(), FALSE as BOOL);
        CShellMenuCallback_AddOrSetMenuItem_body(this, hMenu, IDM_MYDOCUMENTS, CSIDL_MYDOCUMENTS as INT, bExpand, TRUE as BOOL, TRUE as BOOL);
        bAdded = TRUE as BOOL;
    }

    if SHRestricted(REST_NOSMMYPICS) == 0 &&
       CShellMenuCallback_GetAdvancedValue_body(this, CPP_COM_WSTR_CC2776C3CD78.as_ptr(), TRUE as BOOL) != 0 {
        let bExpand: BOOL = CShellMenuCallback_GetAdvancedValue_body(this, CPP_COM_WSTR_404F53961224.as_ptr(), FALSE as BOOL);
        CShellMenuCallback_AddOrSetMenuItem_body(this, hMenu, IDM_MYPICTURES, CSIDL_MYPICTURES as INT, bExpand, TRUE as BOOL, TRUE as BOOL);
        bAdded = TRUE as BOOL;
    }

    if bAdded != 0 {
        AppendMenuW(hMenu, MF_SEPARATOR, 0, core::ptr::null());
    }

    return hMenu;
}

pub unsafe fn CShellMenuCallback_UpdateSettingsMenu_body(this: *mut CShellMenuCallback, hMenu: HMENU) -> () {
    let mut bExpand: BOOL;

    bExpand = CShellMenuCallback_GetAdvancedValue_body(this, CPP_COM_WSTR_436ECA13AB23.as_ptr(), FALSE as BOOL);
    CShellMenuCallback_AddOrSetMenuItem_body(this, hMenu, IDM_CONTROLPANEL, CSIDL_CONTROLS as INT, bExpand, FALSE as BOOL, FALSE as BOOL);

    bExpand = CShellMenuCallback_GetAdvancedValue_body(this, CPP_COM_WSTR_1AA6B1B3DCF4.as_ptr(), FALSE as BOOL);
    CShellMenuCallback_AddOrSetMenuItem_body(this, hMenu, IDM_NETWORKCONNECTIONS, CSIDL_NETWORK as INT, bExpand, FALSE as BOOL, FALSE as BOOL);

    bExpand = CShellMenuCallback_GetAdvancedValue_body(this, CPP_COM_WSTR_382D520E580E.as_ptr(), FALSE as BOOL);
    CShellMenuCallback_AddOrSetMenuItem_body(this, hMenu, IDM_PRINTERSANDFAXES, CSIDL_PRINTERS as INT, bExpand, FALSE as BOOL, FALSE as BOOL);
}

pub unsafe fn CShellMenuCallback_AddStartMenuItems_body(this: *mut CShellMenuCallback, pShellMenu: *mut IShellMenu, csidl: INT, dwFlags: DWORD, psf: *mut IShellFolder) -> HRESULT {
    let mut pidlFolder: *mut ITEMIDLIST = core::ptr::null_mut();
    let mut psfDesktop: *mut IShellFolder = core::ptr::null_mut();
    let _psfDesktop_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(psfDesktop));
    let mut pShellFolder: *mut IShellFolder = core::ptr::null_mut();
    let _pShellFolder_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pShellFolder));
    let mut hr: HRESULT;

    hr = SHGetFolderLocation(core::ptr::null_mut(), csidl, 0 as HANDLE, 0, core::ptr::addr_of_mut!(pidlFolder).cast());
    if ((hr) < 0) {
        return hr;
    }

    if !psf.is_null() {
        pShellFolder = psf;
    } else {
        hr = SHGetDesktopFolder(&mut psfDesktop);
        if ((hr) < 0) {
            return hr;
        }

        hr = ((*(*psfDesktop).lpVtbl).BindToObject)(psfDesktop, pidlFolder.cast(), core::ptr::null_mut(), core::ptr::addr_of!(IID_IShellFolder), core::ptr::addr_of_mut!(pShellFolder).cast());
        if ((hr) < 0) {
            return hr;
        }
    }

    hr = ((*(*pShellMenu).lpVtbl).SetShellFolder)(pShellMenu, pShellFolder, pidlFolder.cast(), core::ptr::null_mut(), dwFlags);
    if ((hr) < 0) {
        return hr;
    }

    return hr;
}

pub unsafe fn CShellMenuCallback_OnGetSubMenu_body(this: *mut CShellMenuCallback, psmd: LPSMDATA, iid: REFIID, pv: *mut *mut core::ffi::c_void) -> HRESULT {
    // ANCESTORDEFAULT (shobjidl.idl: `#define ANCESTORDEFAULT (UINT)-1`) is bare from the
    // prelude now -- used directly below, no local declaration.
    let mut hr: HRESULT = 0;
    let mut pShellMenu: *mut IShellMenu = core::ptr::null_mut();
    let _pShellMenu_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pShellMenu));

    // `core::ptr::addr_of!(IID_IShellMenu), core::ptr::addr_of!(pShellMenu).cast()` expands (in this codebase's Rust convention,
    // see explorer-sys::IID_IShellMenu) to the IID and the void** slot -- not a single-arg call.
    hr = RSHELL_CMenuBand_CreateInstance(
        core::ptr::addr_of!(IID_IShellMenu),
        core::ptr::addr_of_mut!(pShellMenu).cast(),
    );
    if ((hr) < 0) {
        return hr;
    }

    hr = ((*(*pShellMenu).lpVtbl).Initialize)(
        pShellMenu,
        this as *mut IShellMenuCallback,
        0,
        ANCESTORDEFAULT,
        SMINIT_VERTICAL,
    );
    if ((hr) < 0) {
        return hr;
    }

    hr = E_FAIL;
    match (*psmd).uId {
        IDM_PROGRAMS => {
            hr = CShellMenuCallback_AddStartMenuItems_body(
                this,
                pShellMenu,
                CSIDL_PROGRAMS as INT,
                SMSET_TOP,
                (*this).m_psfPrograms,
            );
        }
        IDM_FAVORITES
        | IDM_MYDOCUMENTS
        | IDM_MYPICTURES
        | IDM_CONTROLPANEL
        | IDM_NETWORKCONNECTIONS
        | IDM_PRINTERSANDFAXES => {
            hr = CShellMenuCallback_AddStartMenuItems_body(
                this,
                pShellMenu,
                CShellMenuCallback_CSIDLFromID_body(this, (*psmd).uId),
                SMSET_TOP,
                core::ptr::null_mut(),
            );
        }
        IDM_DOCUMENTS => {
            let hMenu: HMENU = CShellMenuCallback_CreateRecentMenu_body(this);
            if hMenu == core::ptr::null_mut() {
            }

            hr = ((*(*pShellMenu).lpVtbl).SetMenu)(
                pShellMenu,
                hMenu,
                core::ptr::null_mut(),
                SMSET_BOTTOM,
            );
            if ((hr) < 0) {
                return hr;
            }

            hr = CShellMenuCallback_AddStartMenuItems_body(
                this,
                pShellMenu,
                CSIDL_RECENT as INT,
                SMSET_BOTTOM,
                core::ptr::null_mut(),
            );
        }
        IDM_SETTINGS => {
            let mut mii: MENUITEMINFOW = core::mem::zeroed();
            mii.cbSize = core::mem::size_of_val(&mii) as u32;
            mii.fMask = MIIM_SUBMENU;

            if GetMenuItemInfoW((*psmd).hmenu.cast(), (*psmd).uId, FALSE as BOOL, &mut mii) != 0 {
                CShellMenuCallback_UpdateSettingsMenu_body(this, mii.hSubMenu);

                hr = ((*(*pShellMenu).lpVtbl).SetMenu)(
                    pShellMenu,
                    mii.hSubMenu,
                    core::ptr::null_mut(),
                    SMSET_BOTTOM,
                );
                if ((hr) < 0) {
                    return hr;
                }
            }
        }
        _ => {
            let mut mii: MENUITEMINFOW = core::mem::zeroed();
            mii.cbSize = core::mem::size_of_val(&mii) as u32;
            mii.fMask = MIIM_SUBMENU;

            if GetMenuItemInfoW((*psmd).hmenu.cast(), (*psmd).uId, FALSE as BOOL, &mut mii) != 0 {
                hr = ((*(*pShellMenu).lpVtbl).SetMenu)(
                    pShellMenu,
                    mii.hSubMenu,
                    core::ptr::null_mut(),
                    SMSET_BOTTOM,
                );
                if ((hr) < 0) {
                    return hr;
                }
            }
        }
    }

    if hr < 0 {
        return hr;
    }

    hr = ((*(*pShellMenu).lpVtbl).QueryInterface)(pShellMenu, iid, pv);
    pShellMenu = core::ptr::null_mut();
    hr
}

pub unsafe fn CShellMenuCallback_CSIDLFromID_body(this: *mut CShellMenuCallback, uId: UINT) -> INT {
    match uId {
        IDM_PROGRAMS => CSIDL_PROGRAMS as INT,
        IDM_FAVORITES => CSIDL_FAVORITES as INT,
        IDM_DOCUMENTS => CSIDL_RECENT as INT,
        IDM_MYDOCUMENTS => CSIDL_MYDOCUMENTS as INT,
        IDM_MYPICTURES => CSIDL_MYPICTURES as INT,
        IDM_CONTROLPANEL => CSIDL_CONTROLS as INT,
        IDM_NETWORKCONNECTIONS => CSIDL_CONNECTIONS as INT,
        IDM_PRINTERSANDFAXES => CSIDL_PRINTERS as INT,
        _ => 0,
    }
}

pub unsafe fn CShellMenuCallback_OnGetContextMenu_body(this: *mut CShellMenuCallback, psmd: LPSMDATA, iid: REFIID, pv: *mut *mut core::ffi::c_void) -> HRESULT {
    let csidl: INT = CShellMenuCallback_CSIDLFromID_body(this, (*psmd).uId);
    if csidl == 0 {
        return S_FALSE;
    }

    let mut pidl: *mut ITEMIDLIST = core::ptr::null_mut();
    SHGetSpecialFolderLocation(core::ptr::null_mut(), csidl, core::ptr::addr_of_mut!(pidl).cast());

    let mut pSF: *mut IShellFolder = core::ptr::null_mut();
    let _pSF_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pSF));
    let mut pidlChild: *const ITEMIDLIST = core::ptr::null();
    let hr: HRESULT = SHBindToParent(
        pidl.cast(),
        &IID_IShellFolder as *const _,
        core::ptr::addr_of_mut!(pSF) as *mut *mut core::ffi::c_void,
        core::ptr::addr_of_mut!(pidlChild).cast()
    );

    if hr < 0 {
        return hr;
    }

    return ((*(*pSF).lpVtbl).GetUIObjectOf)(
        pSF,
        core::ptr::null_mut(),
        1,
        core::ptr::addr_of!(pidlChild).cast(),
        &IID_IContextMenu as *const _,
        core::ptr::null_mut(),
        pv
    );
}

pub unsafe fn CShellMenuCallback_OnGetObject_body(this: *mut CShellMenuCallback, psmd: LPSMDATA, iid: REFIID, pv: *mut *mut core::ffi::c_void) -> HRESULT {
    if cpp_com_iid_equal(iid, core::ptr::addr_of!(IID_IShellMenu)) {
        return CShellMenuCallback_OnGetSubMenu_body(this, psmd, iid, pv);
    } else if cpp_com_iid_equal(iid, core::ptr::addr_of!(IID_IContextMenu)) {
        return CShellMenuCallback_OnGetContextMenu_body(this, psmd, iid, pv);
    }
    return S_FALSE;
}

pub unsafe fn CShellMenuCallback_OnExec_body(this: *mut CShellMenuCallback, psmd: LPSMDATA) -> HRESULT {
    let mut szPath: [u16; MAX_PATH as usize] = [0u16; MAX_PATH as usize];

    if (*psmd).uId == IDM_CONTROLPANEL {
        ShellExecuteW(
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            CPP_COM_WSTR_AB68B18B313D.as_ptr(),
            CPP_COM_WSTR_C07A84B63277.as_ptr(),
            core::ptr::null_mut(),
            SW_SHOWNORMAL,
        );
    } else if (*psmd).uId == IDM_NETWORKCONNECTIONS {
        ShellExecuteW(
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            CPP_COM_WSTR_AB68B18B313D.as_ptr(),
            CPP_COM_WSTR_D44AF584A75E.as_ptr(),
            core::ptr::null_mut(),
            SW_SHOWNORMAL,
        );
    } else if (*psmd).uId == IDM_PRINTERSANDFAXES {
        ShellExecuteW(
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            CPP_COM_WSTR_AB68B18B313D.as_ptr(),
            CPP_COM_WSTR_FB50816BF2F8.as_ptr(),
            core::ptr::null_mut(),
            SW_SHOWNORMAL,
        );
    } else if (*psmd).uId == IDM_MYDOCUMENTS {
        if SHGetSpecialFolderPathW(
            core::ptr::null_mut(),
            szPath.as_mut_ptr(),
            CSIDL_PERSONAL as INT,
            FALSE as BOOL,
        ) != 0
        {
            ShellExecuteW(
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                szPath.as_ptr(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                SW_SHOWNORMAL,
            );
        } else {
            ERR!("SHGetSpecialFolderPathW failed\n");
        }
    } else if (*psmd).uId == IDM_MYPICTURES {
        if SHGetSpecialFolderPathW(
            core::ptr::null_mut(),
            szPath.as_mut_ptr(),
            CSIDL_MYPICTURES as INT,
            FALSE as BOOL,
        ) != 0
        {
            ShellExecuteW(
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                szPath.as_ptr(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                SW_SHOWNORMAL,
            );
        } else {
            ERR!("SHGetSpecialFolderPathW failed\n");
        }
    } else {
        PostMessageW(
            (*this).m_hwndTray,
            WM_COMMAND,
            (*psmd).uId as WPARAM,
            0,
        );
    }

    S_OK
}

pub unsafe fn CShellMenuCallback_Initialize_body(this: *mut CShellMenuCallback, pShellMenu: *mut IShellMenu, pBandSite: *mut IBandSite, pDeskBar: *mut IDeskBar) -> () {
    CShellMenuCallback_cpp_com_assign_m_pShellMenu(this, pShellMenu);
    CShellMenuCallback_cpp_com_assign_m_pBandSite(this, pBandSite);
    CShellMenuCallback_cpp_com_assign_m_pDeskBar(this, pDeskBar);
}

pub unsafe fn CShellMenuCallback_dtor_body(this: *mut CShellMenuCallback) -> () {}

pub unsafe fn CShellMenuCallback__SetProgramsFolder_body(this: *mut CShellMenuCallback, psf: *mut IShellFolder, pidl: LPITEMIDLIST) -> HRESULT {
    CShellMenuCallback_cpp_com_assign_m_psfPrograms(this, psf);
    (*this).m_pidlPrograms = pidl;
    return S_OK;
}

pub unsafe fn CShellMenuCallback_CallbackSM_body(this: *mut CShellMenuCallback, psmd: LPSMDATA, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> HRESULT {
    match uMsg {
        SMC_INITMENU => return CShellMenuCallback_OnInitMenu_body(this),
        SMC_GETINFO => return CShellMenuCallback_OnGetInfo_body(this, psmd, lParam as *mut SMINFO),
        SMC_GETOBJECT => return CShellMenuCallback_OnGetObject_body(this, psmd, wParam as REFIID, lParam as *mut *mut core::ffi::c_void),
        SMC_EXEC => return CShellMenuCallback_OnExec_body(this, psmd),
        SMC_SFEXEC => {
            ((*(*(*this).m_pTrayPriv).lpVtbl).Execute)((*this).m_pTrayPriv, (*psmd).psf.cast(), (*psmd).pidlItem);
        },
        0x10000000 => {
            if ((*(*(*psmd).psf).lpVtbl).CompareIDs)((*psmd).psf, 0, (*psmd).pidlItem, (*this).m_pidlPrograms) == 0 {
                return S_OK;
            }
            return S_FALSE;
        },
        _ => {},
    }
    return S_FALSE;
}
