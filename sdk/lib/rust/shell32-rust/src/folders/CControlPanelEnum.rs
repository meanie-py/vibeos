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

// Deterministic COM scaffold for CControlPanelEnum; method bodies are generated separately.
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
pub struct CControlPanelEnum {
    pub IEnumIDList_iface: IEnumIDList,
    pub ref_count: core::sync::atomic::AtomicI32,
    pub mpFirst: *mut ENUMLIST,
    pub mpLast: *mut ENUMLIST,
    pub mpCurrent: *mut ENUMLIST,
}

unsafe extern "system" fn CControlPanelEnum_IEnumIDList_QueryInterface(This: *mut IEnumIDList, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CControlPanelEnum, IEnumIDList_iface)) as *mut CControlPanelEnum;
    CControlPanelEnum_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CControlPanelEnum_IEnumIDList_AddRef(This: *mut IEnumIDList) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CControlPanelEnum, IEnumIDList_iface)) as *mut CControlPanelEnum;
    CControlPanelEnum_AddRef(this)
}
unsafe extern "system" fn CControlPanelEnum_IEnumIDList_Release(This: *mut IEnumIDList) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CControlPanelEnum, IEnumIDList_iface)) as *mut CControlPanelEnum;
    CControlPanelEnum_Release(this)
}
unsafe extern "system" fn CControlPanelEnum_IEnumIDList_Next(This: *mut IEnumIDList, celt: ULONG, rgelt: *mut PITEMID_CHILD, pceltFetched: *mut ULONG) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CControlPanelEnum, IEnumIDList_iface)) as *mut CControlPanelEnum;
    CControlPanelEnum_Next_body(this, celt, rgelt, pceltFetched)
}
unsafe extern "system" fn CControlPanelEnum_IEnumIDList_Skip(This: *mut IEnumIDList, celt: ULONG) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CControlPanelEnum, IEnumIDList_iface)) as *mut CControlPanelEnum;
    CControlPanelEnum_Skip_body(this, celt)
}
unsafe extern "system" fn CControlPanelEnum_IEnumIDList_Reset(This: *mut IEnumIDList) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CControlPanelEnum, IEnumIDList_iface)) as *mut CControlPanelEnum;
    CControlPanelEnum_Reset_body(this)
}
unsafe extern "system" fn CControlPanelEnum_IEnumIDList_Clone(This: *mut IEnumIDList, ppenum: *mut *mut IEnumIDList) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CControlPanelEnum, IEnumIDList_iface)) as *mut CControlPanelEnum;
    CControlPanelEnum_Clone_body(this, ppenum)
}

pub static CCONTROLPANELENUM_IENUMIDLIST_VTBL: IEnumIDListVtbl = IEnumIDListVtbl {
    QueryInterface: CControlPanelEnum_IEnumIDList_QueryInterface,
    AddRef: CControlPanelEnum_IEnumIDList_AddRef,
    Release: CControlPanelEnum_IEnumIDList_Release,
    Next: CControlPanelEnum_IEnumIDList_Next,
    Skip: CControlPanelEnum_IEnumIDList_Skip,
    Reset: CControlPanelEnum_IEnumIDList_Reset,
    Clone: CControlPanelEnum_IEnumIDList_Clone,
};

unsafe fn CControlPanelEnum_QueryInterface(
    this: *mut CControlPanelEnum, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IUnknown)) || cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IEnumIDList)) {
        *ppv = core::ptr::addr_of_mut!((*this).IEnumIDList_iface) as *mut core::ffi::c_void;
    }
    else { return E_NOINTERFACE; }
    CControlPanelEnum_AddRef(this);
    S_OK
}

unsafe fn CControlPanelEnum_AddRef(this: *mut CControlPanelEnum) -> ULONG {
    (*this).ref_count.fetch_add(1, core::sync::atomic::Ordering::SeqCst) as ULONG + 1
}

unsafe fn CControlPanelEnum_Release(this: *mut CControlPanelEnum) -> ULONG {
    let refs = (*this).ref_count.fetch_sub(1, core::sync::atomic::Ordering::SeqCst) - 1;
    if refs == 0 {
        CControlPanelEnum_dtor_body(this);
        CControlPanelEnum_CEnumIDListBase_base_dtor_body(this);
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
pub unsafe fn CControlPanelEnum_CreateInstance(
    dwFlags: DWORD,
    pRegEnumerator: *mut IEnumIDList,
    riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    let this = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<CControlPanelEnum>()) as *mut CControlPanelEnum;
    if this.is_null() { return E_OUTOFMEMORY; }
    core::ptr::write(this, CControlPanelEnum {
        IEnumIDList_iface: IEnumIDList { lpVtbl: core::ptr::addr_of!(CCONTROLPANELENUM_IENUMIDLIST_VTBL) },
        ref_count: core::sync::atomic::AtomicI32::new(1),
        mpFirst: core::mem::zeroed(),
        mpLast: core::mem::zeroed(),
        mpCurrent: core::mem::zeroed(),
    });
    Shell32RustObjectLock();
    CControlPanelEnum_ctor_body(this);
    let mut hr = CControlPanelEnum_Initialize_body(this, dwFlags, pRegEnumerator);
    if hr >= 0 {
        hr = CControlPanelEnum_QueryInterface(this, riid, ppv);
    }
    CControlPanelEnum_Release(this);
    hr
}

// Deterministic C++ support closure selected from source identifiers.

pub const IDS_CREATELINK: UINT = 309;

pub const IDS_OPEN: UINT = 23;

pub const IDS_SHV_COLUMN_COMMENTS: UINT = 321;

pub const IDS_SHV_COLUMN_NAME: UINT = 7;

pub const SFGAO_CANLINK: SFGAOF = 0x0000_0004;

pub const CPVIEW_CLASSIC: DWORD = 0;

pub const CSIDL_CONTROLS: DWORD = 0x0003;

pub const ERROR_NOT_FOUND: DWORD = 1168;

pub const FILE_ATTRIBUTE_DIRECTORY: DWORD = 0x00000010;

pub const FILE_ATTRIBUTE_HIDDEN: DWORD = 0x00000002;

pub const FVM_ICON: DWORD = 1;

pub const FVM_LIST: DWORD = 3;

pub const KEY_READ: DWORD = 0x20019;

pub const LVCFMT_LEFT: DWORD = 0x0;

pub const MFS_DEFAULT: DWORD = 4096;

pub const MFS_ENABLED: DWORD = 0;

pub const MFT_SEPARATOR: DWORD = 0x800;

pub const MK_CONTROL: DWORD = 8;

pub const MK_SHIFT: DWORD = 4;

pub const PIDSI_COMMENTS: DWORD = 0x00000006;

pub const ROSPOLICY_CONTROLSFOLDER_DEFLARGEICONS: DWORD = 1;

pub const SEE_MASK_CLASSNAME: DWORD = 0x00000001;

pub const SFVM_DEFVIEWMODE: DWORD = 27;

pub const SHCOLSTATE_ONBYDEFAULT: DWORD = 0x10;

pub const SHCOLSTATE_TYPE_STR: DWORD = 1;

pub const SHCONTF_INCLUDEHIDDEN: DWORD = 128;

pub const SHCONTF_NONFOLDERS: DWORD = 64;

pub const SW_SHOW: DWORD = 5;

// WIN32_FIND_DATAW is NOT redeclared here: the island already carries `_WIN32_FIND_DATAW`
// (via `ntdll_sys::nt`) with the identical field layout, and FindFirstFileW/FindNextFileW
// (also island functions) are typed against THAT one -- a local duplicate here is the exact
// nominal-split trap the crate-wide `use crate::prelude::*` convention exists to avoid.

// Deterministic from shobjidl.h
#[repr(C)]
pub struct IDefaultExtractIconInit {
    pub lpVtbl: *const IDefaultExtractIconInitVtbl,
}
#[repr(C)]
pub struct IDefaultExtractIconInitVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IDefaultExtractIconInit, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IDefaultExtractIconInit) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IDefaultExtractIconInit) -> ULONG,
    pub SetDefaultIcon: unsafe extern "system" fn(This: *mut IDefaultExtractIconInit, pszFile: LPCWSTR, iIcon: i32) -> HRESULT,
    pub SetFlags: unsafe extern "system" fn(This: *mut IDefaultExtractIconInit, uFlags: UINT) -> HRESULT,
    pub SetKey: unsafe extern "system" fn(This: *mut IDefaultExtractIconInit, hkey: HKEY) -> HRESULT,
    pub SetNormalIcon: unsafe extern "system" fn(This: *mut IDefaultExtractIconInit, pszFile: LPCWSTR, iIcon: i32) -> HRESULT,
    pub SetOpenIcon: unsafe extern "system" fn(This: *mut IDefaultExtractIconInit, pszFile: LPCWSTR, iIcon: i32) -> HRESULT,
    pub SetShortcutIcon: unsafe extern "system" fn(This: *mut IDefaultExtractIconInit, pszFile: LPCWSTR, iIcon: i32) -> HRESULT,
}
extern "C" {
    pub static IID_IDefaultExtractIconInit: IID;
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

// `shvheader`/`ControlPanelSFHeader` are this TU's own file-scope preamble, cut three ways by
// the assembler; CControlPanelFolder.rs (the free-functions slice) is the designated owner --
// see its header comment. Import by name instead of carrying a second copy.
use crate::control_panel_folder_tu::{shvheader, ControlPanelSFHeader};
// The crate's own functions: the definition is the declaration.
use crate::wine::pidl::ILClone;
use crate::wine::pidl::ILFree;
use crate::def_view::SHCreateShellFolderView;
use crate::desktop_folder_tu::SHGetDesktopFolder;
use crate::wine::shellpath::SHGetSpecialFolderLocation;
use crate::shell32::shlexec::ShellExecuteExW;
use crate::wine::pidl::_ILGetDataPointer;
// One declaration per external symbol: island.rs.
use crate::island::CoTaskMemAlloc;
use crate::island::LoadStringW;
use crate::island::RegCloseKey;
use crate::island::RegEnumKeyW;
use crate::island::RegGetValueW;
use crate::island::RegOpenKeyExW;

extern "system" {
    fn ExpandEnvironmentStringsW(source: LPCWSTR, dest: LPWSTR, size: DWORD) -> DWORD;
    fn GetLastError() -> DWORD;
    fn lstrlenW(text: LPCWSTR) -> INT;
    fn SHGetNameFromIDList(pidl: PCIDLIST_ABSOLUTE, sigdn: SIGDN, name: *mut PWSTR) -> HRESULT;
}

extern "C" {
    fn _ILIsSpecialFolder(pidl: LPCITEMIDLIST) -> BOOL;
    #[link_name = "CPrinterFolder_CompareDetails_RustBridge"] fn SHELL32_CompareDetails(folder: *mut IShellFolder2, lparam: LPARAM, left: PCUIDLIST_RELATIVE, right: PCUIDLIST_RELATIVE) -> HRESULT;
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

extern "C" {
    static CLSID_ControlPanel: GUID;
    static IID_IContextMenu: GUID;
    static IID_IDataObject: GUID;
    static IID_IDropTarget: GUID;
    static IID_IExtractIconA: GUID;
    static IID_IExtractIconW: GUID;
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

// shobjidl.h STRRET_WSTR -- not in the island under this clean name (its STRRET is bindgen's
// own, with a `__bindgen_anon_1` union rather than a `u` field); every TU that needs it
// declares its own copy (matches shellcom.rs's `STRRET_WSTR`).
const STRRET_WSTR: UINT = 0;

unsafe fn cpp_com_SHSetStrRetW(out: *mut STRRET, value: LPCWSTR) -> HRESULT {
    let len = lstrlenW(value) as usize;
    let bytes = (len + 1) * core::mem::size_of::<WCHAR>();
    let buffer = CoTaskMemAlloc(bytes as SIZE_T) as LPWSTR;
    if buffer.is_null() { return E_OUTOFMEMORY; }
    let _ = memcpy(buffer, value, bytes);
    (*out).uType = STRRET_WSTR;
    // STRRET.__bindgen_anon_1 is a bindgen union: write through a raw pointer to the wrapped
    // field instead of the ZST `__BindgenUnionField` marker (same pattern as PIDLCPanelStruct's
    // `.u.cpanel` in CControlPanelFolder.rs).
    *core::ptr::addr_of_mut!((*out).__bindgen_anon_1).cast::<LPWSTR>() = buffer;
    S_OK
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

// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_DCC3EC907EAC: [WCHAR; 6] = [0x002A, 0x002E, 0x0063, 0x0070, 0x006C, 0x0000];
pub static CPP_COM_WSTR_EB4DDBFDBA58: [WCHAR; 61] = [0x0053, 0x004F, 0x0046, 0x0054, 0x0057, 0x0041, 0x0052, 0x0045, 0x005C, 0x004D, 0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057, 0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x005C, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069, 0x006F, 0x006E, 0x005C, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F, 0x006C, 0x0020, 0x0050, 0x0061, 0x006E, 0x0065, 0x006C, 0x005C, 0x0043, 0x0070, 0x006C, 0x0073, 0x0000];
pub static CPP_COM_WSTR_FEA2588FF02D: [WCHAR; 9] = [0x006E, 0x0063, 0x0070, 0x0061, 0x002E, 0x0063, 0x0070, 0x006C, 0x0000];

use core::ffi::c_void;
// Substrate comes from the bindgen island via `prelude`, and from nowhere else.
// Re-importing the same names through shellcom/types/win32 does not add anything the
// prelude lacks -- it supplies them by a second path, and every one becomes ambiguous
// at the use site. See prelude.rs for why this is "instead of", not "as well as".
use crate::prelude::*;

pub unsafe fn CControlPanelEnum_dtor_body(this: *mut CControlPanelEnum) -> () {}

pub unsafe fn CControlPanelEnum_CEnumIDListBase_base_dtor_body(this: *mut CControlPanelEnum) -> () {
    CControlPanelEnum_DeleteList_body(this);
}

pub unsafe fn CControlPanelEnum_AddToEnumList_body(this: *mut CControlPanelEnum, pidl: LPITEMIDLIST) -> BOOL {
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

pub unsafe fn CControlPanelEnum_ctor_body(this: *mut CControlPanelEnum) -> () {}

pub unsafe fn CControlPanelEnum_Initialize_body(this: *mut CControlPanelEnum, dwFlags: DWORD, pRegEnumerator: *mut IEnumIDList) -> HRESULT {
    if CControlPanelEnum_CreateCPanelEnumList_body(this, dwFlags) == FALSE {
        return E_FAIL;
    }
    CControlPanelEnum_AppendItemsFromEnumerator_body(this, pRegEnumerator);
    return S_OK;
}

pub unsafe fn CControlPanelEnum_DeleteList_body(this: *mut CControlPanelEnum) -> BOOL {
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

pub unsafe fn CControlPanelEnum_AppendItemsFromEnumerator_body(this: *mut CControlPanelEnum, pEnum: *mut IEnumIDList) -> HRESULT {
    let mut pidl: LPITEMIDLIST;
    let mut dwFetched: u32;

    if pEnum.is_null() {
        return E_INVALIDARG;
    }

    ((*(*pEnum).lpVtbl).Reset)(pEnum);

    loop {
        pidl = core::ptr::null_mut();
        dwFetched = 0;
        let hr = ((*(*pEnum).lpVtbl).Next)(pEnum, 1, &mut pidl, &mut dwFetched);
        if hr != S_OK || dwFetched == 0 {
            break;
        }
        CControlPanelEnum_AddToEnumList_body(this, pidl);
    }

    S_OK
}

pub unsafe fn CControlPanelEnum_Next_body(this: *mut CControlPanelEnum, celt: ULONG, rgelt: *mut PITEMID_CHILD, pceltFetched: *mut ULONG) -> HRESULT {
    let mut i: u32;
    let mut hr: i32 = S_OK;
    let mut temp: PITEMID_CHILD;

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

pub unsafe fn CControlPanelEnum_RegisterCPanelApp_body(this: *mut CControlPanelEnum, wpath: LPCWSTR) -> BOOL {
    // Control_LoadApplet/Control_UnloadApplet (wine/control.rs) are typed against
    // `crate::types`, not the island -- but HWND/LPCWSTR/WCHAR are all plain aliases down to
    // the same primitives (`*mut c_void`, `*const u16`) in both, so no cast is needed at this
    // call boundary, only at the CPanel out-param (which this TU never touches: NULL in, NULL
    // out).
    let applet = crate::wine::control::Control_LoadApplet(core::ptr::null_mut(), wpath, core::ptr::null_mut());
    let mut iconIdx: i32;

    if !applet.is_null() {
        let mut i: u32 = 0;
        while i < (*applet).count {
            // cpanel.h: `struct applet_info info[1];  /* array of count information */`
            // -- a flexible array whose real length is `count`, which is this loop's
            // bound. Any .cpl exporting more than one applet (main.cpl: mouse +
            // keyboard) panicked on the second.
            let info = (&raw const (*applet).info).cast::<applet_info>().add(i as usize);
            if (*info).idIcon > 0 {
                iconIdx = -((*info).idIcon as i32);
            } else {
                iconIdx = 0;
            }

            // `.name`/`.info` are fixed `[WCHAR; N]` arrays (wine/control.rs's AppletInfo), not
            // pointers -- C's array-to-pointer decay, made explicit.
            let pidl = crate::control_panel_folder_tu::_ILCreateCPanelApplet(
                wpath,
                (*info).name.as_ptr(),
                (*info).info.as_ptr(),
                iconIdx,
            );

            if !pidl.is_null() {
                CControlPanelEnum_AddToEnumList_body(this, pidl);
            }
            i += 1;
        }
        crate::wine::control::Control_UnloadApplet(applet);
    }
    TRUE
}

pub unsafe fn CControlPanelEnum_Skip_body(this: *mut CControlPanelEnum, celt: ULONG) -> HRESULT {
    let mut dwIndex: u32;
    let mut hr: i32 = S_OK;

    dwIndex = 0;
    while dwIndex < celt {
        if (*this).mpCurrent.is_null() {
            hr = S_FALSE;
            break;
        }
        (*this).mpCurrent = (*(*this).mpCurrent).pNext;
        dwIndex += 1;
    }
    return hr;
}

pub unsafe fn CControlPanelEnum_Reset_body(this: *mut CControlPanelEnum) -> HRESULT {
    (*this).mpCurrent = (*this).mpFirst;
    return S_OK;
}

pub unsafe fn CControlPanelEnum_RegisterRegistryCPanelApps_body(this: *mut CControlPanelEnum, hkey_root: HKEY, szRepPath: LPCWSTR) -> i32 {
    let mut name: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut value: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut hkey: HKEY = core::ptr::null_mut();

    let mut cnt: i32 = 0;

    if RegOpenKeyW(hkey_root, szRepPath, &mut hkey) == ERROR_SUCCESS as LSTATUS {
        let mut idx: i32 = 0;

        loop {
            let mut nameLen: u32 = MAX_PATH;
            let mut valueLen: u32 = MAX_PATH;
            let mut buffer: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];

            if RegEnumValueW(
                hkey,
                idx as u32,
                name.as_mut_ptr(),
                &mut nameLen,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                value.as_mut_ptr() as *mut u8,
                &mut valueLen,
            ) != ERROR_SUCCESS as LSTATUS
            {
                break;
            }

            if ExpandEnvironmentStringsW(value.as_ptr(), buffer.as_mut_ptr(), MAX_PATH as DWORD)
                != 0
            {
                wcscpy(value.as_mut_ptr(), buffer.as_ptr());
            }

            if CControlPanelEnum_RegisterCPanelApp_body(this, value.as_ptr()) != 0 {
                cnt += 1;
            }

            idx += 1;
        }
        RegCloseKey(hkey);
    }

    cnt
}

pub unsafe fn CControlPanelEnum_Clone_body(this: *mut CControlPanelEnum, ppenum: *mut *mut IEnumIDList) -> HRESULT { E_NOTIMPL }

// CControlPanelFolder.cpp:231. Hand-written: the call-seal audit refused the paid worker for
// emitting FindNextFileW twice ("not exactly once"), which is the documented
// restructuring-breaks-1:1 class rather than a defect — the C is a `do { ... } while
// (FindNextFileW(...))` whose `continue` jumps to the CONDITION, so a faithful lowering has to
// reach that call from two places. Written here as a single `loop` with one tail call, which is
// the same control flow with the duplication removed: `continue` in the C falls through to the
// condition, and here it falls through to the same tail test.
pub unsafe fn CControlPanelEnum_CreateCPanelEnumList_body(
    this: *mut CControlPanelEnum,
    dwFlags: DWORD,
) -> BOOL {
    let mut szPath: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut wfd: WIN32_FIND_DATAW = core::mem::zeroed();

    // enumerate the control panel applets
    if (dwFlags & SHCONTF_NONFOLDERS) != 0 {
        GetSystemDirectoryW(szPath.as_mut_ptr(), MAX_PATH);
        let p: LPWSTR = PathAddBackslashW(szPath.as_mut_ptr());
        wcscpy(p, CPP_COM_WSTR_DCC3EC907EAC.as_ptr());

        let hFile: HANDLE = FindFirstFileW(szPath.as_ptr(), &mut wfd);

        if hFile != INVALID_HANDLE_VALUE {
            loop {
                let hidden_and_excluded = (dwFlags & SHCONTF_INCLUDEHIDDEN) == 0
                    && (wfd.dwFileAttributes & FILE_ATTRIBUTE_HIDDEN) != 0;
                if !hidden_and_excluded
                    && (wfd.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) == 0
                {
                    wcscpy(p, wfd.cFileName.as_ptr());
                    // ncpa.cpl is the Network applet, which the shell surfaces through the
                    // Network Connections folder instead — skipping it here is deliberate.
                    if wcscmp(wfd.cFileName.as_ptr(), CPP_COM_WSTR_FEA2588FF02D.as_ptr()) != 0 {
                        CControlPanelEnum_RegisterCPanelApp_body(this, szPath.as_ptr());
                    }
                }
                if FindNextFileW(hFile, &mut wfd) == 0 {
                    break;
                }
            }
            FindClose(hFile);
        }

        CControlPanelEnum_RegisterRegistryCPanelApps_body(
            this,
            HKEY_LOCAL_MACHINE,
            CPP_COM_WSTR_EB4DDBFDBA58.as_ptr(),
        );
        CControlPanelEnum_RegisterRegistryCPanelApps_body(
            this,
            HKEY_CURRENT_USER,
            CPP_COM_WSTR_EB4DDBFDBA58.as_ptr(),
        );
    }
    TRUE
}
