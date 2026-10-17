// Deterministic from shlobj.h
#[repr(C)]
pub struct IShellFolderViewCB {
    pub lpVtbl: *const IShellFolderViewCBVtbl,
}
#[repr(C)]
pub struct IShellFolderViewCBVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IShellFolderViewCB, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IShellFolderViewCB) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IShellFolderViewCB) -> ULONG,
    pub MessageSFVCB: unsafe extern "system" fn(This: *mut IShellFolderViewCB, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> HRESULT,
}
extern "C" {
    pub static IID_IShellFolderViewCB: IID;
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

// Deterministic COM scaffold for CShellFolderViewCBWrapper; method bodies are generated separately.
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
pub struct CShellFolderViewCBWrapper {
    pub IShellFolderViewCB_iface: IShellFolderViewCB,
    pub IObjectWithSite_iface: IObjectWithSite,
    pub ref_count: core::sync::atomic::AtomicI32,
    pub m_pUnkSite: *mut IUnknown,
    pub m_hWndMain: HWND,
    pub m_Pidl: PIDLIST_ABSOLUTE,
    pub m_psf: *mut IShellFolder,
    pub m_psvOuter: *mut IShellView,
    pub m_Callback: LPFNVIEWCALLBACK,
    pub m_FVM: FOLDERVIEWMODE,
    pub m_Events: LONG,
}

unsafe fn CShellFolderViewCBWrapper_cpp_com_assign_m_psf(this: *mut CShellFolderViewCBWrapper, value: *mut IShellFolder) {
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

unsafe fn CShellFolderViewCBWrapper_cpp_com_assign_m_psvOuter(this: *mut CShellFolderViewCBWrapper, value: *mut IShellView) {
    let old = (*this).m_psvOuter;
    if old == value { return; }
    if !value.is_null() {
        let unknown = value as *mut IUnknown;
        let add_ref: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).AddRef);
        add_ref(unknown);
    }
    (*this).m_psvOuter = value;
    if !old.is_null() {
        let unknown = old as *mut IUnknown;
        let release: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).Release);
        release(unknown);
    }
}

unsafe extern "system" fn CShellFolderViewCBWrapper_IShellFolderViewCB_QueryInterface(This: *mut IShellFolderViewCB, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellFolderViewCBWrapper, IShellFolderViewCB_iface)) as *mut CShellFolderViewCBWrapper;
    CShellFolderViewCBWrapper_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CShellFolderViewCBWrapper_IShellFolderViewCB_AddRef(This: *mut IShellFolderViewCB) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellFolderViewCBWrapper, IShellFolderViewCB_iface)) as *mut CShellFolderViewCBWrapper;
    CShellFolderViewCBWrapper_AddRef(this)
}
unsafe extern "system" fn CShellFolderViewCBWrapper_IShellFolderViewCB_Release(This: *mut IShellFolderViewCB) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellFolderViewCBWrapper, IShellFolderViewCB_iface)) as *mut CShellFolderViewCBWrapper;
    CShellFolderViewCBWrapper_Release(this)
}
unsafe extern "system" fn CShellFolderViewCBWrapper_IShellFolderViewCB_MessageSFVCB(This: *mut IShellFolderViewCB, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellFolderViewCBWrapper, IShellFolderViewCB_iface)) as *mut CShellFolderViewCBWrapper;
    CShellFolderViewCBWrapper_MessageSFVCB_body(this, uMsg, wParam, lParam)
}

pub static CSHELLFOLDERVIEWCBWRAPPER_ISHELLFOLDERVIEWCB_VTBL: IShellFolderViewCBVtbl = IShellFolderViewCBVtbl {
    QueryInterface: CShellFolderViewCBWrapper_IShellFolderViewCB_QueryInterface,
    AddRef: CShellFolderViewCBWrapper_IShellFolderViewCB_AddRef,
    Release: CShellFolderViewCBWrapper_IShellFolderViewCB_Release,
    MessageSFVCB: CShellFolderViewCBWrapper_IShellFolderViewCB_MessageSFVCB,
};

unsafe extern "system" fn CShellFolderViewCBWrapper_IObjectWithSite_QueryInterface(This: *mut IObjectWithSite, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellFolderViewCBWrapper, IObjectWithSite_iface)) as *mut CShellFolderViewCBWrapper;
    CShellFolderViewCBWrapper_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CShellFolderViewCBWrapper_IObjectWithSite_AddRef(This: *mut IObjectWithSite) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellFolderViewCBWrapper, IObjectWithSite_iface)) as *mut CShellFolderViewCBWrapper;
    CShellFolderViewCBWrapper_AddRef(this)
}
unsafe extern "system" fn CShellFolderViewCBWrapper_IObjectWithSite_Release(This: *mut IObjectWithSite) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellFolderViewCBWrapper, IObjectWithSite_iface)) as *mut CShellFolderViewCBWrapper;
    CShellFolderViewCBWrapper_Release(this)
}
unsafe extern "system" fn CShellFolderViewCBWrapper_IObjectWithSite_SetSite(This: *mut IObjectWithSite, pUnkSite: *mut IUnknown) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellFolderViewCBWrapper, IObjectWithSite_iface)) as *mut CShellFolderViewCBWrapper;
    CShellFolderViewCBWrapper_SetSite_body(this, pUnkSite)
}
unsafe extern "system" fn CShellFolderViewCBWrapper_IObjectWithSite_GetSite(This: *mut IObjectWithSite, riid: REFIID, ppvSite: *mut PVOID) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellFolderViewCBWrapper, IObjectWithSite_iface)) as *mut CShellFolderViewCBWrapper;
    CShellFolderViewCBWrapper_GetSite_body(this, riid, ppvSite)
}

pub static CSHELLFOLDERVIEWCBWRAPPER_IOBJECTWITHSITE_VTBL: IObjectWithSiteVtbl = IObjectWithSiteVtbl {
    QueryInterface: CShellFolderViewCBWrapper_IObjectWithSite_QueryInterface,
    AddRef: CShellFolderViewCBWrapper_IObjectWithSite_AddRef,
    Release: CShellFolderViewCBWrapper_IObjectWithSite_Release,
    SetSite: CShellFolderViewCBWrapper_IObjectWithSite_SetSite,
    GetSite: CShellFolderViewCBWrapper_IObjectWithSite_GetSite,
};

unsafe fn CShellFolderViewCBWrapper_QueryInterface(
    this: *mut CShellFolderViewCBWrapper, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IUnknown)) || cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IShellFolderViewCB)) {
        *ppv = core::ptr::addr_of_mut!((*this).IShellFolderViewCB_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IObjectWithSite)) {
        *ppv = core::ptr::addr_of_mut!((*this).IObjectWithSite_iface) as *mut core::ffi::c_void;
    }
    else { return E_NOINTERFACE; }
    CShellFolderViewCBWrapper_AddRef(this);
    S_OK
}

unsafe fn CShellFolderViewCBWrapper_AddRef(this: *mut CShellFolderViewCBWrapper) -> ULONG {
    (*this).ref_count.fetch_add(1, core::sync::atomic::Ordering::SeqCst) as ULONG + 1
}

unsafe fn CShellFolderViewCBWrapper_Release(this: *mut CShellFolderViewCBWrapper) -> ULONG {
    let refs = (*this).ref_count.fetch_sub(1, core::sync::atomic::Ordering::SeqCst) - 1;
    if refs == 0 {
        CShellFolderViewCBWrapper_dtor_body(this);
        CShellFolderViewCBWrapper_CObjectWithSiteBase_base_dtor_body(this);
        CShellFolderViewCBWrapper_cpp_com_assign_m_psf(this, core::ptr::null_mut());
        CShellFolderViewCBWrapper_cpp_com_assign_m_psvOuter(this, core::ptr::null_mut());
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
pub unsafe fn CShellFolderViewCBWrapper_CreateInstance(
    psvcbi: LPCSFV,
    riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    let this = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<CShellFolderViewCBWrapper>()) as *mut CShellFolderViewCBWrapper;
    if this.is_null() { return E_OUTOFMEMORY; }
    core::ptr::write(this, CShellFolderViewCBWrapper {
        IShellFolderViewCB_iface: IShellFolderViewCB { lpVtbl: core::ptr::addr_of!(CSHELLFOLDERVIEWCBWRAPPER_ISHELLFOLDERVIEWCB_VTBL) },
        IObjectWithSite_iface: IObjectWithSite { lpVtbl: core::ptr::addr_of!(CSHELLFOLDERVIEWCBWRAPPER_IOBJECTWITHSITE_VTBL) },
        ref_count: core::sync::atomic::AtomicI32::new(1),
        m_pUnkSite: core::mem::zeroed(),
        m_hWndMain: core::mem::zeroed(),
        m_Pidl: core::mem::zeroed(),
        m_psf: core::mem::zeroed(),
        m_psvOuter: core::mem::zeroed(),
        m_Callback: core::mem::zeroed(),
        m_FVM: core::mem::zeroed(),
        m_Events: core::mem::zeroed(),
    });
    Shell32RustObjectLock();
    CShellFolderViewCBWrapper_ctor_body(this);
    let mut hr = CShellFolderViewCBWrapper_Initialize_body(this, psvcbi);
    if hr >= 0 {
        hr = CShellFolderViewCBWrapper_QueryInterface(this, riid, ppv);
    }
    CShellFolderViewCBWrapper_Release(this);
    hr
}

// Deterministic C++ support closure selected from source identifiers.

pub const E_UNEXPECTED: HRESULT = 0x8000_FFFFu32 as HRESULT;

pub const FVM_DETAILS: DWORD = 4;

pub const FVM_ICON: DWORD = 1;

pub const FVM_FIRST: DWORD = FVM_ICON;

pub const FVM_LIST: DWORD = 3;

pub const FVM_SMALLICON: DWORD = 2;

pub const FVM_THUMBNAIL: DWORD = 5;

pub const FVM_THUMBSTRIP: DWORD = 7;

pub const FVM_TILE: DWORD = 6;

pub const SFVM_DEFVIEWMODE: DWORD = 27;

pub const SFVM_GETNOTIFY: DWORD = 49;

pub const SFVM_HWNDMAIN: DWORD = 46;

pub const SFVM_SETISFV: DWORD = 39;

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

pub type LPFNSVADDPROPSHEETPAGE = Option<unsafe extern "system" fn(HPROPSHEETPAGE, LPARAM) -> BOOL>;

extern "system" {
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
// The crate's own functions: the definition is the declaration.
use crate::wine::pidl::ILClone;
use crate::wine::pidl::ILFree;
use crate::prelude::*;

pub unsafe fn CShellFolderViewCBWrapper_CObjectWithSiteBase_base_dtor_body(this: *mut CShellFolderViewCBWrapper) -> () {
    CShellFolderViewCBWrapper_SetSite_body(this, core::ptr::null_mut());
}

pub unsafe fn CShellFolderViewCBWrapper_CObjectWithSiteBase_base_SetSite_body(this: *mut CShellFolderViewCBWrapper, pUnkSite: *mut IUnknown) -> HRESULT {
    let punkOrg = (*this).m_pUnkSite;
    if !punkOrg.is_null() {
        ((*(*punkOrg).lpVtbl).Release)(punkOrg);
    }
    (*this).m_pUnkSite = pUnkSite;
    if !pUnkSite.is_null() {
        ((*(*pUnkSite).lpVtbl).AddRef)(pUnkSite);
    }
    S_OK
}

pub unsafe fn CShellFolderViewCBWrapper_GetSite_body(this: *mut CShellFolderViewCBWrapper, riid: REFIID, ppvSite: *mut PVOID) -> HRESULT {
    *ppvSite = core::ptr::null_mut();
    if !(*this).m_pUnkSite.is_null() {
        ((*(*(*this).m_pUnkSite).lpVtbl).QueryInterface)((*this).m_pUnkSite, riid, ppvSite)
    } else {
        E_FAIL
    }
}

pub unsafe fn CShellFolderViewCBWrapper_ctor_body(this: *mut CShellFolderViewCBWrapper) -> () {}

pub unsafe fn CShellFolderViewCBWrapper_dtor_body(this: *mut CShellFolderViewCBWrapper) -> () {
    ILFree((*this).m_Pidl);
}

pub unsafe fn CShellFolderViewCBWrapper_Initialize_body(this: *mut CShellFolderViewCBWrapper, psvcbi: LPCSFV) -> HRESULT {
    CShellFolderViewCBWrapper_cpp_com_assign_m_psf(this, (*psvcbi).pshf);
    CShellFolderViewCBWrapper_cpp_com_assign_m_psvOuter(this, (*psvcbi).psvOuter.cast());
    (*this).m_Pidl = if !(*psvcbi).pidl.is_null() { ILClone((*psvcbi).pidl) } else { core::ptr::null_mut() };
    (*this).m_Callback = (*psvcbi).pfnCallback;
    (*this).m_FVM = (*psvcbi).fvm;
    (*this).m_Events = (*psvcbi).lEvents;
    S_OK
}

pub unsafe fn CShellFolderViewCBWrapper_MessageSFVCB_body(this: *mut CShellFolderViewCBWrapper, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> HRESULT {
    match uMsg {
        SFVM_HWNDMAIN => {
            (*this).m_hWndMain = lParam as HWND;
        }
        SFVM_DEFVIEWMODE => {
            if (*this).m_FVM != 0 {
                *(lParam as *mut FOLDERVIEWMODE) = (*this).m_FVM;
            }
        }
        _ => {}
    }

    let Some(callback) = (*this).m_Callback else { return E_FAIL };
    let hr = callback(
        (*this).m_psvOuter.cast(),
        (*this).m_psf.cast(),
        (*this).m_hWndMain,
        uMsg,
        wParam,
        lParam,
    );
    if hr >= 0 {
        return hr;
    }

    match uMsg {
        SFVM_GETNOTIFY => {
            *(wParam as *mut LPITEMIDLIST) = (*this).m_Pidl;
            *(lParam as *mut LONG) = (*this).m_Events;
            return S_OK;
        }
        _ => {}
    }
    hr
}

pub unsafe fn CShellFolderViewCBWrapper_SetSite_body(this: *mut CShellFolderViewCBWrapper, pUnkSite: *mut IUnknown) -> HRESULT {
    let hr = CShellFolderViewCBWrapper_CObjectWithSiteBase_base_SetSite_body(this, pUnkSite);
    CShellFolderViewCBWrapper_MessageSFVCB_body(this, SFVM_SETISFV, 0, pUnkSite as LPARAM);
    hr
}

// ---- CDefViewUtil.cpp free functions ----

#[no_mangle]
pub unsafe extern "C" fn ShellViewIdToFolderViewMode(pVid: *const SHELLVIEWID) -> HRESULT {
    if IsEqualGUID(pVid, core::ptr::addr_of!(VID_LargeIcons)) {
        return FVM_ICON as HRESULT;
    } else if IsEqualGUID(pVid, core::ptr::addr_of!(VID_SmallIcons)) {
        return FVM_SMALLICON as HRESULT;
    } else if IsEqualGUID(pVid, core::ptr::addr_of!(VID_List)) {
        return FVM_LIST as HRESULT;
    } else if IsEqualGUID(pVid, core::ptr::addr_of!(VID_Details)) {
        return FVM_DETAILS as HRESULT;
    } else if IsEqualGUID(pVid, core::ptr::addr_of!(VID_Thumbnails)) {
        return FVM_THUMBNAIL as HRESULT;
    } else if IsEqualGUID(pVid, core::ptr::addr_of!(VID_Tile)) {
        return FVM_TILE as HRESULT;
    } else if IsEqualGUID(pVid, core::ptr::addr_of!(VID_ThumbStrip)) {
        return FVM_THUMBSTRIP as HRESULT;
    }
    return E_UNEXPECTED;
}
#[no_mangle]
pub unsafe extern "C" fn FolderViewModeToShellViewId(mut FVM: UINT) -> *const SHELLVIEWID {
    let vids: [*const SHELLVIEWID; 6] = [
        core::ptr::addr_of!(VID_LargeIcons),
        core::ptr::addr_of!(VID_SmallIcons),
        core::ptr::addr_of!(VID_Details),
        core::ptr::addr_of!(VID_Thumbnails),
        core::ptr::addr_of!(VID_Tile),
        core::ptr::addr_of!(VID_ThumbStrip),
    ];
    FVM -= FVM_FIRST;
    if (FVM as usize) < vids.len() {
        vids[FVM as usize]
    } else {
        core::ptr::null()
    }
}

#[no_mangle]
pub unsafe extern "system" fn SHCreateShellFolderViewEx(
    pcsfv: LPCSFV,
    ppsv: *mut *mut IShellView,
) -> HRESULT {
    if ppsv.is_null() {
        return E_INVALIDARG;
    }
    *ppsv = core::ptr::null_mut();

    let mut psfvcb: *mut IShellFolderViewCB = core::ptr::null_mut();
    let mut create = SFV_CREATE {
        cbSize: core::mem::size_of::<SFV_CREATE>() as UINT,
        pshf: (*pcsfv).pshf,
        psvOuter: (*pcsfv).psvOuter,
        psfvcb: core::ptr::null_mut(),
    };

    if (*pcsfv).pfnCallback.is_some() {
        let hr = CShellFolderViewCBWrapper_CreateInstance(
            pcsfv,
            core::ptr::addr_of!(IID_IShellFolderViewCB),
            core::ptr::addr_of_mut!(psfvcb).cast(),
        );
        if hr < 0 {
            return hr;
        }
        create.psfvcb = psfvcb.cast();
    }
    SHCreateShellFolderView(&create, ppsv.cast())
}
