// The island/bindgen substrate this whole TU takes its scalars and Win32 surface from; see
// prelude.rs for why this replaces (not joins) the hand-written `types`/`win32`/`shellcom`
// globs this file used to carry — those three together made every shared name ambiguous
// (E0659) at 100+ sites, since the island already provides all of them.
use crate::prelude::*;
// The crate's own functions: the definition is the declaration.
use crate::wine::pidl::ILClone;
use crate::def_view::SHCreateShellFolderView;
// One declaration per external symbol: island.rs.
use crate::island::CoTaskMemAlloc;
use crate::island::DestroyMenu;
use crate::island::LoadStringW;
// `STRRET` is a COM boundary type the island renders as a bindgen union workaround
// (`__bindgen_anon_1` + `__BindgenUnionField<T>`, no direct field access) rather than a real
// Rust `union`; `shellcom::STRRET` is the plain, directly-writable union every other flipped
// module in this crate already uses for the exact same `uType`/`pOleStr` pattern (cfs_folder.rs,
// drives_folder.rs, desktop_folder.rs, file_sys_enum.rs, recycle_bin.rs, printer_folder.rs,
// printers_enum.rs, control_panel_folder.rs). `STRRET_WSTR` only exists in `shellcom` (an
// anonymous C enumerator, so bindgen never named it) — imported alongside for the same reason.
use crate::shellcom::{STRRET, STRRET_WSTR};

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

// Deterministic COM scaffold for CNetFolderEnum; method bodies are generated separately.
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
pub struct CNetFolderEnum {
    pub IEnumIDList_iface: IEnumIDList,
    pub ref_count: core::sync::atomic::AtomicI32,
    pub mpFirst: *mut ENUMLIST,
    pub mpLast: *mut ENUMLIST,
    pub mpCurrent: *mut ENUMLIST,
}

unsafe extern "system" fn CNetFolderEnum_IEnumIDList_QueryInterface(This: *mut IEnumIDList, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CNetFolderEnum, IEnumIDList_iface)) as *mut CNetFolderEnum;
    CNetFolderEnum_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CNetFolderEnum_IEnumIDList_AddRef(This: *mut IEnumIDList) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CNetFolderEnum, IEnumIDList_iface)) as *mut CNetFolderEnum;
    CNetFolderEnum_AddRef(this)
}
unsafe extern "system" fn CNetFolderEnum_IEnumIDList_Release(This: *mut IEnumIDList) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CNetFolderEnum, IEnumIDList_iface)) as *mut CNetFolderEnum;
    CNetFolderEnum_Release(this)
}
unsafe extern "system" fn CNetFolderEnum_IEnumIDList_Next(This: *mut IEnumIDList, celt: ULONG, rgelt: *mut PITEMID_CHILD, pceltFetched: *mut ULONG) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CNetFolderEnum, IEnumIDList_iface)) as *mut CNetFolderEnum;
    CNetFolderEnum_Next_body(this, celt, rgelt, pceltFetched)
}
unsafe extern "system" fn CNetFolderEnum_IEnumIDList_Skip(This: *mut IEnumIDList, celt: ULONG) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CNetFolderEnum, IEnumIDList_iface)) as *mut CNetFolderEnum;
    CNetFolderEnum_Skip_body(this, celt)
}
unsafe extern "system" fn CNetFolderEnum_IEnumIDList_Reset(This: *mut IEnumIDList) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CNetFolderEnum, IEnumIDList_iface)) as *mut CNetFolderEnum;
    CNetFolderEnum_Reset_body(this)
}
unsafe extern "system" fn CNetFolderEnum_IEnumIDList_Clone(This: *mut IEnumIDList, ppenum: *mut *mut IEnumIDList) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CNetFolderEnum, IEnumIDList_iface)) as *mut CNetFolderEnum;
    CNetFolderEnum_Clone_body(this, ppenum)
}

pub static CNETFOLDERENUM_IENUMIDLIST_VTBL: IEnumIDListVtbl = IEnumIDListVtbl {
    QueryInterface: CNetFolderEnum_IEnumIDList_QueryInterface,
    AddRef: CNetFolderEnum_IEnumIDList_AddRef,
    Release: CNetFolderEnum_IEnumIDList_Release,
    Next: CNetFolderEnum_IEnumIDList_Next,
    Skip: CNetFolderEnum_IEnumIDList_Skip,
    Reset: CNetFolderEnum_IEnumIDList_Reset,
    Clone: CNetFolderEnum_IEnumIDList_Clone,
};

unsafe fn CNetFolderEnum_QueryInterface(
    this: *mut CNetFolderEnum, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IUnknown)) || cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IEnumIDList)) {
        *ppv = core::ptr::addr_of_mut!((*this).IEnumIDList_iface) as *mut core::ffi::c_void;
    }
    else { return E_NOINTERFACE; }
    CNetFolderEnum_AddRef(this);
    S_OK
}

unsafe fn CNetFolderEnum_AddRef(this: *mut CNetFolderEnum) -> ULONG {
    (*this).ref_count.fetch_add(1, core::sync::atomic::Ordering::SeqCst) as ULONG + 1
}

unsafe fn CNetFolderEnum_Release(this: *mut CNetFolderEnum) -> ULONG {
    let refs = (*this).ref_count.fetch_sub(1, core::sync::atomic::Ordering::SeqCst) - 1;
    if refs == 0 {
        CNetFolderEnum_dtor_body(this);
        CNetFolderEnum_CEnumIDListBase_base_dtor_body(this);
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
pub unsafe fn CNetFolderEnum_CreateInstance(
    hwndOwner: HWND,
    dwFlags: DWORD,
    riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    let this = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<CNetFolderEnum>()) as *mut CNetFolderEnum;
    if this.is_null() { return E_OUTOFMEMORY; }
    core::ptr::write(this, CNetFolderEnum {
        IEnumIDList_iface: IEnumIDList { lpVtbl: core::ptr::addr_of!(CNETFOLDERENUM_IENUMIDLIST_VTBL) },
        ref_count: core::sync::atomic::AtomicI32::new(1),
        mpFirst: core::mem::zeroed(),
        mpLast: core::mem::zeroed(),
        mpCurrent: core::mem::zeroed(),
    });
    Shell32RustObjectLock();
    CNetFolderEnum_ctor_body(this);
    let mut hr = CNetFolderEnum_Initialize_body(this, hwndOwner, dwFlags);
    if hr >= 0 {
        hr = CNetFolderEnum_QueryInterface(this, riid, ppv);
    }
    CNetFolderEnum_Release(this);
    hr
}

// Deterministic C++ support closure selected from source identifiers.

pub const COLUMN_CATEGORY: UINT = 1;

pub const COLUMN_NAME: UINT = 0;

pub const COLUMN_NETLOCATION: UINT = 3;

pub const COLUMN_WORKGROUP: UINT = 2;

pub const IDI_SHELL_NETWORK_FOLDER: UINT = 172;


pub const IDS_SHV_COLUMN_CATEGORY: UINT = 313;

pub const IDS_SHV_COLUMN_NAME: UINT = 7;

pub const IDS_SHV_COLUMN_NETLOCATION: UINT = 315;

pub const IDS_SHV_COLUMN_WORKGROUP: UINT = 314;

pub const NETWORKPLACESSHELLVIEWCOLUMNS: UINT = 4;

pub const E_UNEXPECTED: HRESULT = 0x8000_FFFFu32 as HRESULT;

pub const SFGAO_CANDELETE: SFGAOF = 0x0000_0020;

pub const SFGAO_CANLINK: SFGAOF = 0x0000_0004;

pub const SFGAO_CANRENAME: SFGAOF = 0x0000_0010;

pub const SFGAO_HASPROPSHEET: SFGAOF = 0x0000_0040;

pub const DFM_INVOKECOMMAND: DWORD = 2;

pub const DFM_MERGECONTEXTMENU: DWORD = 1;

pub const FILE_ATTRIBUTE_DIRECTORY: DWORD = 0x00000010;

pub const LVCFMT_RIGHT: DWORD = 0x1;

pub const MFS_ENABLED: DWORD = 0;

pub const MM_ADDSEPARATOR: DWORD = 0x00000001;

pub const RESOURCETYPE_DISK: DWORD = 1;

pub const RESOURCEUSAGE_CONTAINER: DWORD = 0x00000002;

pub const RESOURCE_GLOBALNET: DWORD = 2;

pub const SHCOLSTATE_ONBYDEFAULT: DWORD = 0x10;

pub const SHCOLSTATE_TYPE_STR: DWORD = 1;

pub const SHCONTF_FOLDERS: DWORD = 32;

pub use crate::island::QCMINFO;

pub use crate::island::QCMINFO_IDMAP;

pub use crate::island::QCMINFO_IDMAP_PLACEMENT;

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

pub type LPFNSVADDPROPSHEETPAGE = Option<unsafe extern "system" fn(HPROPSHEETPAGE, LPARAM) -> BOOL>;

pub use crate::island::shvheader;
pub static NetworkPlacesSFHeader: [shvheader; 4] = [
    shvheader { colnameid: (IDS_SHV_COLUMN_NAME) as WORD, colstate: (SHCOLSTATE_TYPE_STR | SHCOLSTATE_ONBYDEFAULT) as WORD, fmt: (LVCFMT_RIGHT) as WORD, cxChar: (15) as WORD },
    shvheader { colnameid: (IDS_SHV_COLUMN_CATEGORY) as WORD, colstate: (SHCOLSTATE_TYPE_STR | SHCOLSTATE_ONBYDEFAULT) as WORD, fmt: (LVCFMT_RIGHT) as WORD, cxChar: (10) as WORD },
    shvheader { colnameid: (IDS_SHV_COLUMN_WORKGROUP) as WORD, colstate: (SHCOLSTATE_TYPE_STR | SHCOLSTATE_ONBYDEFAULT) as WORD, fmt: (LVCFMT_RIGHT) as WORD, cxChar: (15) as WORD },
    shvheader { colnameid: (IDS_SHV_COLUMN_NETLOCATION) as WORD, colstate: (SHCOLSTATE_TYPE_STR | SHCOLSTATE_ONBYDEFAULT) as WORD, fmt: (LVCFMT_RIGHT) as WORD, cxChar: (15) as WORD },
];

extern "system" {
    fn lstrlenW(text: LPCWSTR) -> INT;
    fn CoTaskMemFree(memory: LPVOID);
}

// `ILCreateFromNetworkPlaceW` is a real `#[no_mangle]` export of the sibling `CNetFolder.rs`
// (`crate::net_folder_tu`) — declared here rather than imported across the module boundary,
// matching how every other flipped folder module reaches a sibling's `#[no_mangle]` fn.
extern "C" {
    fn ILCreateFromNetworkPlaceW(lpNetworkPlace: LPCWSTR) -> LPITEMIDLIST;
}

// `WNetOpenEnum`/`WNetEnumResource` are `#define`d to their `...W` forms under UNICODE
// (winnetwk.h) — the island only carries the real A/W-suffixed exports (see `WNetOpenEnumW`/
// `WNetEnumResourceW`), so this TU needs the same A/W mapping prelude.rs hand-writes for
// SendMessage and friends. `WNetCloseEnum` has no A/W split and resolves straight from the
// island.
#[inline]
unsafe fn WNetOpenEnum(
    dwScope: DWORD,
    dwType: DWORD,
    dwUsage: DWORD,
    lpNet: LPNETRESOURCE,
    lphEnum: *mut HANDLE,
) -> DWORD {
    WNetOpenEnumW(dwScope, dwType, dwUsage, lpNet, lphEnum)
}
#[inline]
unsafe fn WNetEnumResource(
    hEnum: HANDLE,
    lpcCount: *mut DWORD,
    lpBuffer: LPNETRESOURCE,
    lpBufferSize: *mut DWORD,
) -> DWORD {
    WNetEnumResourceW(hEnum, lpcCount, lpBuffer.cast(), lpBufferSize)
}

extern "C" {
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
    static CLSID_NetworkPlaces: GUID;
    static CLSID_ShellFSFolder: GUID;
    static IID_IContextMenu: GUID;
    static IID_IDataObject: GUID;
    static IID_IDropTarget: GUID;
    static IID_IExtractIconA: GUID;
    static IID_IExtractIconW: GUID;
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

pub unsafe fn CNetFolderEnum_dtor_body(this: *mut CNetFolderEnum) -> () {}

pub unsafe fn CNetFolderEnum_CEnumIDListBase_base_dtor_body(this: *mut CNetFolderEnum) -> () {
    CNetFolderEnum_DeleteList_body(this);
}

pub unsafe fn CNetFolderEnum_AddToEnumList_body(this: *mut CNetFolderEnum, pidl: LPITEMIDLIST) -> BOOL {
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

pub unsafe fn CNetFolderEnum_DeleteList_body(this: *mut CNetFolderEnum) -> BOOL {
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

pub unsafe fn CNetFolderEnum_AppendItemsFromEnumerator_body(this: *mut CNetFolderEnum, pEnum: *mut IEnumIDList) -> HRESULT {
    let mut pidl: LPITEMIDLIST;
    let mut dwFetched: u32;

    if pEnum.is_null() {
        return E_INVALIDARG;
    }

    ((*(*pEnum).lpVtbl).Reset)(pEnum);

    loop {
        pidl = core::ptr::null_mut();
        dwFetched = 0;
        if S_OK != ((*(*pEnum).lpVtbl).Next)(pEnum, 1, &mut pidl, &mut dwFetched) {
            break;
        }
        if dwFetched == 0 {
            break;
        }
        CNetFolderEnum_AddToEnumList_body(this, pidl);
    }

    S_OK
}

pub unsafe fn CNetFolderEnum_ctor_body(this: *mut CNetFolderEnum) -> () {}

pub unsafe fn CNetFolderEnum_Initialize_body(this: *mut CNetFolderEnum, hwndOwner: HWND, dwFlags: DWORD) -> HRESULT {
    if CNetFolderEnum_CreateMyCompEnumList_body(this, dwFlags) == FALSE {
        return E_FAIL;
    }

    return S_OK;
}

pub unsafe fn CNetFolderEnum_Next_body(this: *mut CNetFolderEnum, celt: ULONG, rgelt: *mut PITEMID_CHILD, pceltFetched: *mut ULONG) -> HRESULT {
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
            hr = if i != 0 {
                S_FALSE
            } else {
                E_OUTOFMEMORY
            };
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

pub unsafe fn CNetFolderEnum_EnumerateRec_body(this: *mut CNetFolderEnum, lpNet: LPNETRESOURCE) -> BOOL {
    let mut bRet: BOOL = TRUE;
    let mut dRet: DWORD;
    // C: `HANDLE hEnum;` (uninitialized local, filled by the `&hEnum` out-parameter below) —
    // Rust's borrow checker rejects `&mut` of a definitely-uninitialized binding even for an
    // FFI out-param, so seed it first; `WNetOpenEnum` always writes it before any successful
    // path reads it.
    let mut hEnum: HANDLE = core::ptr::null_mut();
    let mut lpRes: LPNETRESOURCE;
    let mut dSize: DWORD = 0x1000;
    let mut dCount: DWORD = (!1u32).wrapping_add(1);
    let mut lpCur: LPNETRESOURCE;

    dRet = WNetOpenEnum(RESOURCE_GLOBALNET, RESOURCETYPE_DISK, 0, lpNet, &mut hEnum);
    if dRet != WN_SUCCESS {
        return FALSE;
    }

    lpRes = CoTaskMemAlloc(dSize as SIZE_T) as LPNETRESOURCE;
    if lpRes.is_null() {
        WNetCloseEnum(hEnum);
        return FALSE;
    }

    loop {
        dSize = 0x1000;
        dCount = (!1u32).wrapping_add(1);

        memset(lpRes as *mut core::ffi::c_void, 0, dSize as usize);
        dRet = WNetEnumResource(hEnum, &mut dCount, lpRes, &mut dSize);
        if dRet == WN_SUCCESS || dRet == WN_MORE_DATA {
            lpCur = lpRes;
            while dCount > 0 {
                if ((*lpCur).dwUsage & RESOURCEUSAGE_CONTAINER) == RESOURCEUSAGE_CONTAINER {
                    CNetFolderEnum_EnumerateRec_body(this, lpCur);
                } else {
                    let pidl: LPITEMIDLIST = ILCreateFromNetworkPlaceW((*lpCur).lpRemoteName);
                    if !pidl.is_null() {
                        bRet = CNetFolderEnum_AddToEnumList_body(this, pidl);
                    } else {
                        bRet = FALSE;
                        break;
                    }
                }

                lpCur = lpCur.offset(1);
                dCount -= 1;
            }
        }

        if dRet == WN_NO_MORE_ENTRIES {
            break;
        }
    }

    CoTaskMemFree(lpRes as *mut core::ffi::c_void);
    WNetCloseEnum(hEnum);

    bRet
}

pub unsafe fn CNetFolderEnum_Skip_body(this: *mut CNetFolderEnum, celt: ULONG) -> HRESULT {
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
    hr
}

pub unsafe fn CNetFolderEnum_Reset_body(this: *mut CNetFolderEnum) -> HRESULT {
    (*this).mpCurrent = (*this).mpFirst;
    S_OK
}

pub unsafe fn CNetFolderEnum_Clone_body(this: *mut CNetFolderEnum, ppenum: *mut *mut IEnumIDList) -> HRESULT { E_NOTIMPL }

pub unsafe fn CNetFolderEnum_CreateMyCompEnumList_body(this: *mut CNetFolderEnum, dwFlags: DWORD) -> BOOL {
    let mut bRet: BOOL = TRUE;

    if (dwFlags & SHCONTF_FOLDERS) != 0 {
        bRet = CNetFolderEnum_EnumerateRec_body(this, core::ptr::null_mut());
    }

    return bRet;
}
