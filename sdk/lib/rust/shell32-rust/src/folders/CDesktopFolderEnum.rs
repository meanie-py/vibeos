// Deterministic from shobjidl.h
#[repr(C)]
pub struct IEnumIDList {
    pub lpVtbl: *const IEnumIDListVtbl,
}
#[repr(C)]
pub struct IEnumIDListVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IEnumIDList, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IEnumIDList) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IEnumIDList) -> ULONG,
    pub Next: unsafe extern "system" fn(This: *mut IEnumIDList, celt: ULONG, rgelt: *mut PITEMID_CHILD, pceltFetched: *mut ULONG) -> HRESULT,
    pub Skip: unsafe extern "system" fn(This: *mut IEnumIDList, celt: ULONG) -> HRESULT,
    pub Reset: unsafe extern "system" fn(This: *mut IEnumIDList) -> HRESULT,
    pub Clone: unsafe extern "system" fn(This: *mut IEnumIDList, ppenum: *mut *mut IEnumIDList) -> HRESULT,
}
extern "C" {
    pub static IID_IEnumIDList: IID;
}

// Deterministic COM scaffold for CDesktopFolderEnum; method bodies are generated separately.
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
extern "C" {
    pub static IID_IUnknown: IID;
}

#[repr(C)]
pub struct CDesktopFolderEnum {
    pub IEnumIDList_iface: IEnumIDList,
    pub ref_count: core::sync::atomic::AtomicI32,
    pub mpFirst: *mut ENUMLIST,
    pub mpLast: *mut ENUMLIST,
    pub mpCurrent: *mut ENUMLIST,
}

unsafe extern "system" fn CDesktopFolderEnum_IEnumIDList_QueryInterface(This: *mut IEnumIDList, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDesktopFolderEnum, IEnumIDList_iface)) as *mut CDesktopFolderEnum;
    CDesktopFolderEnum_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CDesktopFolderEnum_IEnumIDList_AddRef(This: *mut IEnumIDList) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDesktopFolderEnum, IEnumIDList_iface)) as *mut CDesktopFolderEnum;
    CDesktopFolderEnum_AddRef(this)
}
unsafe extern "system" fn CDesktopFolderEnum_IEnumIDList_Release(This: *mut IEnumIDList) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDesktopFolderEnum, IEnumIDList_iface)) as *mut CDesktopFolderEnum;
    CDesktopFolderEnum_Release(this)
}
unsafe extern "system" fn CDesktopFolderEnum_IEnumIDList_Next(This: *mut IEnumIDList, celt: ULONG, rgelt: *mut PITEMID_CHILD, pceltFetched: *mut ULONG) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDesktopFolderEnum, IEnumIDList_iface)) as *mut CDesktopFolderEnum;
    CDesktopFolderEnum_Next_body(this, celt, rgelt, pceltFetched)
}
unsafe extern "system" fn CDesktopFolderEnum_IEnumIDList_Skip(This: *mut IEnumIDList, celt: ULONG) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDesktopFolderEnum, IEnumIDList_iface)) as *mut CDesktopFolderEnum;
    CDesktopFolderEnum_Skip_body(this, celt)
}
unsafe extern "system" fn CDesktopFolderEnum_IEnumIDList_Reset(This: *mut IEnumIDList) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDesktopFolderEnum, IEnumIDList_iface)) as *mut CDesktopFolderEnum;
    CDesktopFolderEnum_Reset_body(this)
}
unsafe extern "system" fn CDesktopFolderEnum_IEnumIDList_Clone(This: *mut IEnumIDList, ppenum: *mut *mut IEnumIDList) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDesktopFolderEnum, IEnumIDList_iface)) as *mut CDesktopFolderEnum;
    CDesktopFolderEnum_Clone_body(this, ppenum)
}

pub static CDESKTOPFOLDERENUM_IENUMIDLIST_VTBL: IEnumIDListVtbl = IEnumIDListVtbl {
    QueryInterface: CDesktopFolderEnum_IEnumIDList_QueryInterface,
    AddRef: CDesktopFolderEnum_IEnumIDList_AddRef,
    Release: CDesktopFolderEnum_IEnumIDList_Release,
    Next: CDesktopFolderEnum_IEnumIDList_Next,
    Skip: CDesktopFolderEnum_IEnumIDList_Skip,
    Reset: CDesktopFolderEnum_IEnumIDList_Reset,
    Clone: CDesktopFolderEnum_IEnumIDList_Clone,
};

unsafe fn CDesktopFolderEnum_QueryInterface(
    this: *mut CDesktopFolderEnum, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IUnknown)) || cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IEnumIDList)) {
        *ppv = core::ptr::addr_of_mut!((*this).IEnumIDList_iface) as *mut core::ffi::c_void;
    }
    else { return E_NOINTERFACE; }
    CDesktopFolderEnum_AddRef(this);
    S_OK
}

unsafe fn CDesktopFolderEnum_AddRef(this: *mut CDesktopFolderEnum) -> ULONG {
    (*this).ref_count.fetch_add(1, core::sync::atomic::Ordering::SeqCst) as ULONG + 1
}

unsafe fn CDesktopFolderEnum_Release(this: *mut CDesktopFolderEnum) -> ULONG {
    let refs = (*this).ref_count.fetch_sub(1, core::sync::atomic::Ordering::SeqCst) - 1;
    if refs == 0 {
        CDesktopFolderEnum_dtor_body(this);
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
pub unsafe fn CDesktopFolderEnum_CreateInstance(
    pRegFolder: *mut IShellFolder,
    dwFlags: SHCONTF,
    pRegEnumerator: *mut IEnumIDList,
    pDesktopEnumerator: *mut IEnumIDList,
    pCommonDesktopEnumerator: *mut IEnumIDList,
    riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    let this = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<CDesktopFolderEnum>()) as *mut CDesktopFolderEnum;
    if this.is_null() { return E_OUTOFMEMORY; }
    core::ptr::write(this, CDesktopFolderEnum {
        IEnumIDList_iface: IEnumIDList { lpVtbl: core::ptr::addr_of!(CDESKTOPFOLDERENUM_IENUMIDLIST_VTBL) },
        ref_count: core::sync::atomic::AtomicI32::new(1),
        mpFirst: core::mem::zeroed(),
        mpLast: core::mem::zeroed(),
        mpCurrent: core::mem::zeroed(),
    });
    Shell32RustObjectLock();
    CDesktopFolderEnum_ctor_body(this);
    let mut hr = CDesktopFolderEnum_Initialize_body(this, pRegFolder, dwFlags, pRegEnumerator, pDesktopEnumerator, pCommonDesktopEnumerator);
    if hr >= 0 {
        hr = CDesktopFolderEnum_QueryInterface(this, riid, ppv);
    }
    CDesktopFolderEnum_Release(this);
    hr
}

// Deterministic C++ support closure selected from source identifiers.


pub const E_UNEXPECTED: HRESULT = 0x8000_FFFFu32 as HRESULT;

pub const SFGAO_CANDELETE: SFGAOF = 0x0000_0020;

pub const SFGAO_CANLINK: SFGAOF = 0x0000_0004;

pub const SFGAO_CANRENAME: SFGAOF = 0x0000_0010;

pub const SFGAO_HASPROPSHEET: SFGAOF = 0x0000_0040;

pub const CSIDL_COMMON_DESKTOPDIRECTORY: DWORD = 0x0019;

pub const CSIDL_DESKTOPDIRECTORY: DWORD = 0x0010;

pub const CSIDL_FLAG_CREATE: DWORD = 0x8000;

pub const DFM_INVOKECOMMAND: DWORD = 2;

pub const DFM_MERGECONTEXTMENU: DWORD = 1;

pub const ERROR_FILE_NOT_FOUND: DWORD = 2;

pub const ERROR_PATH_NOT_FOUND: DWORD = 3;

pub const FVM_ICON: DWORD = 1;

pub const FWF_DESKTOP: DWORD = 0x20;

pub const MFS_ENABLED: DWORD = 0;

pub const MM_ADDSEPARATOR: DWORD = 0x00000001;

pub const ROSPOLICY_DESKTOPFOLDER_DEFLARGEICONS: DWORD = 1;

pub const SFGAO_DROPTARGET: DWORD = 0x00000100;

pub const SFGAO_STREAM: DWORD = 0x00400000;

pub const SFVM_DEFVIEWMODE: DWORD = 27;

pub const SFVM_GETCOMMANDDIR: DWORD = 33;

pub const SFVM_VIEWRELEASE: DWORD = 12;

pub const SHCIDS_ALLFIELDS: DWORD = 0x80000000;

pub const SHCIDS_CANONICALONLY: DWORD = 0x10000000;

pub const SHCIDS_COLUMNMASK: DWORD = 0x0000FFFF;

pub const SSF_STARTPANELON: DWORD = 0x00200000;

pub const STGM_CREATE: DWORD = 0x00001000;

#[repr(C)]
pub struct BIND_OPTS {
    pub cbStruct: DWORD,
    pub grfFlags: DWORD,
    pub grfMode: DWORD,
    pub dwTickCountDeadline: DWORD,
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

pub use crate::island::PARSEDURLW;

pub use crate::island::QCMINFO;

pub use crate::island::QCMINFO_IDMAP;

pub use crate::island::QCMINFO_IDMAP_PLACEMENT;

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

#[repr(C)]
pub struct SFV_CREATE {
    pub cbSize: UINT,
    pub pshf: *mut IShellFolder,
    pub psvOuter: *mut core::ffi::c_void,
    pub psfvcb: *mut core::ffi::c_void,
}

macro_rules! ASSERT { ($($arg:tt)*) => {{}}; }

pub type SVSIF = UINT;

pub type LPTBBUTTONSB = *mut core::ffi::c_void;

pub type LPFNSVADDPROPSHEETPAGE = Option<unsafe extern "system" fn(HPROPSHEETPAGE, LPARAM) -> BOOL>;

extern "system" {
    fn lstrlenW(text: LPCWSTR) -> INT;
    fn CoTaskMemFree(memory: LPVOID);
    #[link_name = "Shell32RustCoCreateInitSF"] fn SHELL32_CoCreateInitSFWithCsidl(root: LPCITEMIDLIST, clsid: *const GUID, csidl: INT, riid: REFIID, out: *mut *mut core::ffi::c_void) -> HRESULT;
}

extern "C" {
    fn _ILIsSpecialFolder(pidl: LPCITEMIDLIST) -> BOOL;
    #[link_name = "memcpy"] fn cpp_com_memcpy(dst: *mut core::ffi::c_void, src: *const core::ffi::c_void, bytes: usize) -> *mut core::ffi::c_void;
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
    static CLSID_Internet: GUID;
    static CLSID_MyComputer: GUID;
    static CLSID_NetworkPlaces: GUID;
    static CLSID_ShellDesktop: GUID;
    static CLSID_ShellFSFolder: GUID;
    static IID_IContextMenu: GUID;
    static IID_IDataObject: GUID;
    static IID_IDropTarget: GUID;
    static IID_IExtractIconA: GUID;
    static IID_IExtractIconW: GUID;
    static IID_IShellFolder: GUID;
    static IID_IShellFolder2: GUID;
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

unsafe fn cpp_com_SHSetStrRetW(out: *mut STRRET, value: LPCWSTR) -> HRESULT {
    let len = lstrlenW(value) as usize;
    let bytes = (len + 1) * core::mem::size_of::<WCHAR>();
    let buffer = CoTaskMemAlloc(bytes as SIZE_T) as LPWSTR;
    if buffer.is_null() { return E_OUTOFMEMORY; }
    let _ = memcpy(buffer, value, bytes);
    (*out).uType = STRRET_WSTR; (*out).u.pOleStr = buffer; S_OK
}
unsafe fn cpp_com_SHSetStrRetResource(out: *mut STRRET, id: DWORD) -> HRESULT {
    let mut buffer = [0u16; MAX_PATH as usize];
    if LoadStringW(shell32_hInstance, id, buffer.as_mut_ptr(), MAX_PATH as INT) == 0 { return E_FAIL; }
    cpp_com_SHSetStrRetW(out, buffer.as_ptr())
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

use core::ffi::c_void;
// The crate's own functions: the definition is the declaration.
use crate::wine::pidl::ILClone;
use crate::wine::pidl::ILCombine;
use crate::wine::pidl::ILFree;
use crate::def_view::SHCreateShellFolderView;
use crate::desktop_folder_tu::SHGetDesktopFolder;
use crate::wine::pidl::_ILIsDesktop;
// One declaration per external symbol: island.rs.
use crate::island::CoTaskMemAlloc;
use crate::island::DestroyMenu;
use crate::island::LoadStringW;
// Substrate comes from the bindgen island via `prelude`, and from nowhere else.
// Re-importing the same names through shellcom/types/win32 does not add anything the
// prelude lacks -- it supplies them by a second path, and every one becomes ambiguous
// at the use site. See prelude.rs for why this is "instead of", not "as well as".
use crate::prelude::*;
// `STRRET` is a COM boundary type the island renders as a bindgen union workaround
// (`__bindgen_anon_1` + `__BindgenUnionField<T>`, no direct field access) rather than a real
// Rust `union`; `shellcom::STRRET` is the plain, directly-writable union every other flipped
// module in this crate already uses for the exact same `uType`/`pOleStr` pattern (matches
// CNetFolderEnum.rs's identical import for the identical reason). `STRRET_WSTR` only exists in
// `shellcom` (an anonymous C enumerator, so bindgen never named it).
use crate::shellcom::{STRRET, STRRET_WSTR};

pub unsafe fn CDesktopFolderEnum_ctor_body(this: *mut CDesktopFolderEnum) -> () {}

pub unsafe fn CDesktopFolderEnum_dtor_body(this: *mut CDesktopFolderEnum) -> () {
    CDesktopFolderEnum_DeleteList_body(this);
}

pub unsafe fn CDesktopFolderEnum_AddToEnumList_body(this: *mut CDesktopFolderEnum, pidl: LPITEMIDLIST) -> BOOL {
    let pNew: *mut ENUMLIST;

    if pidl.is_null() {
        return FALSE;
    }

    pNew = SHAlloc(core::mem::size_of::<ENUMLIST>() as SIZE_T) as *mut ENUMLIST;
    if !pNew.is_null() {
        (*pNew).pNext = core::ptr::null_mut();
        (*pNew).pidl = pidl;

        if (*this).mpFirst.is_null() {
            (*this).mpFirst = pNew;
            (*this).mpCurrent = pNew;
        }

        if !(*this).mpLast.is_null() {
            (*(*this).mpLast).pNext = pNew;
        }

        (*this).mpLast = pNew;
        return TRUE;
    }
    return FALSE;
}

pub unsafe fn MarkAsCommonItem_body(pidl: LPITEMIDLIST) -> () {
    (*((*pidl).mkid.abID.as_mut_ptr() as *mut PIDLDATA)).type_ |= PT_FS_COMMON_FLAG as BYTE;
}

pub unsafe fn CDesktopFolderEnum_DeleteList_body(this: *mut CDesktopFolderEnum) -> BOOL {
    let mut pDelete: *mut ENUMLIST;

    while !(*this).mpFirst.is_null() {
        pDelete = (*this).mpFirst;
        (*this).mpFirst = (*pDelete).pNext;
        SHFree((*pDelete).pidl);
        SHFree(pDelete);
    }
    (*this).mpFirst = core::ptr::null_mut();
    (*this).mpLast = core::ptr::null_mut();
    (*this).mpCurrent = core::ptr::null_mut();
    TRUE
}

pub unsafe fn CDesktopFolderEnum_AppendItemsFromEnumerator_body(this: *mut CDesktopFolderEnum, pEnum: *mut IEnumIDList) -> HRESULT {
    let mut pidl: LPITEMIDLIST = core::ptr::null_mut();
    let mut dwFetched: u32 = 0;

    if pEnum.is_null() {
        return E_INVALIDARG;
    }

    ((*(*pEnum).lpVtbl).Reset)(pEnum);

    while S_OK == ((*(*pEnum).lpVtbl).Next)(pEnum, 1, &mut pidl, &mut dwFetched) && dwFetched != 0 {
        CDesktopFolderEnum_AddToEnumList_body(this, pidl);
    }

    S_OK
}

pub unsafe fn CDesktopFolderEnum_Next_body(this: *mut CDesktopFolderEnum, celt: ULONG, rgelt: *mut PITEMID_CHILD, pceltFetched: *mut ULONG) -> HRESULT {
    let mut i: ULONG;
    let mut hr: HRESULT = S_OK;
    let mut temp: LPITEMIDLIST;

    if !pceltFetched.is_null() {
        *pceltFetched = 0;
    }

    *rgelt = core::ptr::null_mut();

    if celt > 1 && pceltFetched.is_null() {
        return E_INVALIDARG;
    }

    if celt > 0 && (*this).mpCurrent.is_null() {
        return S_FALSE;
    }

    i = 0;
    while i < celt {
        if (*this).mpCurrent.is_null() {
            hr = S_FALSE;
            break;
        }

        temp = ILClone((*(*this).mpCurrent).pidl);
        if temp.is_null() {
            hr = if i != 0 { S_FALSE } else { E_OUTOFMEMORY };
            break;
        }
        *rgelt.add(i as usize) = temp;
        (*this).mpCurrent = (*(*this).mpCurrent).pNext;
        i += 1;
    }

    if !pceltFetched.is_null() {
        *pceltFetched = i;
    }

    hr
}

pub unsafe fn CDesktopFolderEnum_Skip_body(this: *mut CDesktopFolderEnum, celt: ULONG) -> HRESULT {
    let mut dwIndex: u32;
    let mut hr: i32 = S_OK;

    dwIndex = 0;
    while dwIndex < celt {
        if (*this).mpCurrent.is_null() {
            hr = S_FALSE;
            break;
        }
        (*this).mpCurrent = (*(*this).mpCurrent).pNext;
        dwIndex = dwIndex.wrapping_add(1);
    }
    hr
}

pub unsafe fn CDesktopFolderEnum_Reset_body(this: *mut CDesktopFolderEnum) -> HRESULT {
    (*this).mpCurrent = (*this).mpFirst;
    return S_OK;
}

pub unsafe fn CDesktopFolderEnum_Clone_body(this: *mut CDesktopFolderEnum, ppenum: *mut *mut IEnumIDList) -> HRESULT { E_NOTIMPL }

pub unsafe fn CDesktopFolderEnum_Initialize_body(this: *mut CDesktopFolderEnum, pRegFolder: *mut IShellFolder, dwFlags: SHCONTF, pRegEnumerator: *mut IEnumIDList, pDesktopEnumerator: *mut IEnumIDList, pCommonDesktopEnumerator: *mut IEnumIDList) -> HRESULT {
    CDesktopFolderEnum_AppendItemsFromEnumerator_body(this, pRegEnumerator);

    CDesktopFolderEnum_AppendItemsFromEnumerator_body(this, pDesktopEnumerator);
    let pCommon = (*this).mpLast;
    CDesktopFolderEnum_AppendItemsFromEnumerator_body(this, pCommonDesktopEnumerator);
    if pCommon != (*this).mpLast {
        let mut fake: ENUMLIST = core::mem::zeroed();
        let mut pCommon = if pCommon.is_null() {
            fake.pNext = (*this).mpFirst;
            &mut fake as *mut ENUMLIST
        } else {
            pCommon
        };
        loop {
            pCommon = (*pCommon).pNext;
            if pCommon.is_null() {
                break;
            }
            MarkAsCommonItem_body((*pCommon).pidl);
        }
    }
    S_OK
}
