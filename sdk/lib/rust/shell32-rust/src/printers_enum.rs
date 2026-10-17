// Deterministic from shobjidl.h
#[repr(C)]
pub struct IEnumIDList {
    pub lpVtbl: *const IEnumIDListVtbl,
}
#[repr(C)]
pub struct IEnumIDListVtbl {
    pub QueryInterface: unsafe extern "system" fn(
        This: *mut IEnumIDList,
        riid: REFIID,
        ppvObject: *mut *mut core::ffi::c_void,
    ) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IEnumIDList) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IEnumIDList) -> ULONG,
    pub Next: unsafe extern "system" fn(
        This: *mut IEnumIDList,
        celt: ULONG,
        rgelt: *mut PITEMID_CHILD,
        pceltFetched: *mut ULONG,
    ) -> HRESULT,
    pub Skip: unsafe extern "system" fn(This: *mut IEnumIDList, celt: ULONG) -> HRESULT,
    pub Reset: unsafe extern "system" fn(This: *mut IEnumIDList) -> HRESULT,
    pub Clone:
        unsafe extern "system" fn(This: *mut IEnumIDList, ppenum: *mut *mut IEnumIDList) -> HRESULT,
}
extern "C" {
    pub static IID_IEnumIDList: IID;
}

// Deterministic COM scaffold for CPrintersEnum; method bodies are generated separately.
extern "system" {
    fn GetProcessHeap() -> HANDLE;
    #[link_name = "HeapAlloc"]
    fn cpp_com_HeapAlloc(heap: HANDLE, flags: DWORD, bytes: usize) -> *mut core::ffi::c_void;
    #[link_name = "HeapFree"]
    fn cpp_com_HeapFree(heap: HANDLE, flags: DWORD, memory: *mut core::ffi::c_void) -> BOOL;
    fn Shell32RustObjectLock();
    fn Shell32RustObjectUnlock();
}
trait CppComSize {
    fn cpp_com_size(self) -> usize;
}
impl CppComSize for usize {
    fn cpp_com_size(self) -> usize {
        self
    }
}
impl CppComSize for u32 {
    fn cpp_com_size(self) -> usize {
        self as usize
    }
}
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

pub use crate::island::ENUMLIST;

#[repr(C)]
pub struct CPrintersEnum {
    pub IEnumIDList_iface: IEnumIDList,
    pub ref_count: core::sync::atomic::AtomicI32,
    pub mpFirst: *mut ENUMLIST,
    pub mpLast: *mut ENUMLIST,
    pub mpCurrent: *mut ENUMLIST,
}

unsafe extern "system" fn CPrintersEnum_IEnumIDList_QueryInterface(
    This: *mut IEnumIDList,
    riid: REFIID,
    ppvObject: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CPrintersEnum, IEnumIDList_iface))
        as *mut CPrintersEnum;
    CPrintersEnum_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CPrintersEnum_IEnumIDList_AddRef(This: *mut IEnumIDList) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CPrintersEnum, IEnumIDList_iface))
        as *mut CPrintersEnum;
    CPrintersEnum_AddRef(this)
}
unsafe extern "system" fn CPrintersEnum_IEnumIDList_Release(This: *mut IEnumIDList) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CPrintersEnum, IEnumIDList_iface))
        as *mut CPrintersEnum;
    CPrintersEnum_Release(this)
}
unsafe extern "system" fn CPrintersEnum_IEnumIDList_Next(
    This: *mut IEnumIDList,
    celt: ULONG,
    rgelt: *mut PITEMID_CHILD,
    pceltFetched: *mut ULONG,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CPrintersEnum, IEnumIDList_iface))
        as *mut CPrintersEnum;
    CPrintersEnum_Next_body(this, celt, rgelt, pceltFetched)
}
unsafe extern "system" fn CPrintersEnum_IEnumIDList_Skip(
    This: *mut IEnumIDList,
    celt: ULONG,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CPrintersEnum, IEnumIDList_iface))
        as *mut CPrintersEnum;
    CPrintersEnum_Skip_body(this, celt)
}
unsafe extern "system" fn CPrintersEnum_IEnumIDList_Reset(This: *mut IEnumIDList) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CPrintersEnum, IEnumIDList_iface))
        as *mut CPrintersEnum;
    CPrintersEnum_Reset_body(this)
}
unsafe extern "system" fn CPrintersEnum_IEnumIDList_Clone(
    This: *mut IEnumIDList,
    ppenum: *mut *mut IEnumIDList,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CPrintersEnum, IEnumIDList_iface))
        as *mut CPrintersEnum;
    CPrintersEnum_Clone_body(this, ppenum)
}

pub static CPRINTERSENUM_IENUMIDLIST_VTBL: IEnumIDListVtbl = IEnumIDListVtbl {
    QueryInterface: CPrintersEnum_IEnumIDList_QueryInterface,
    AddRef: CPrintersEnum_IEnumIDList_AddRef,
    Release: CPrintersEnum_IEnumIDList_Release,
    Next: CPrintersEnum_IEnumIDList_Next,
    Skip: CPrintersEnum_IEnumIDList_Skip,
    Reset: CPrintersEnum_IEnumIDList_Reset,
    Clone: CPrintersEnum_IEnumIDList_Clone,
};

unsafe fn CPrintersEnum_QueryInterface(
    this: *mut CPrintersEnum,
    riid: REFIID,
    ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() {
        return E_POINTER;
    }
    *ppv = core::ptr::null_mut();
    if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IUnknown))
        || cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IEnumIDList))
    {
        *ppv = core::ptr::addr_of_mut!((*this).IEnumIDList_iface) as *mut core::ffi::c_void;
    } else {
        return E_NOINTERFACE;
    }
    CPrintersEnum_AddRef(this);
    S_OK
}

unsafe fn CPrintersEnum_AddRef(this: *mut CPrintersEnum) -> ULONG {
    (*this)
        .ref_count
        .fetch_add(1, core::sync::atomic::Ordering::SeqCst) as ULONG
        + 1
}

unsafe fn CPrintersEnum_Release(this: *mut CPrintersEnum) -> ULONG {
    let refs = (*this)
        .ref_count
        .fetch_sub(1, core::sync::atomic::Ordering::SeqCst)
        - 1;
    if refs == 0 {
        CPrintersEnum_dtor_body(this);
        CPrintersEnum_CEnumIDListBase_base_dtor_body(this);
        Shell32RustObjectUnlock();
        HeapFree(GetProcessHeap(), 0, this as *mut core::ffi::c_void);
    }
    refs as ULONG
}

#[inline]
unsafe fn cpp_com_iid_equal(left: REFIID, right: REFIID) -> bool {
    !left.is_null()
        && !right.is_null()
        && (*left).Data1 == (*right).Data1
        && (*left).Data2 == (*right).Data2
        && (*left).Data3 == (*right).Data3
        && (*left).Data4 == (*right).Data4
}

// Internal activation: shellutils.h ShellObjectCreator<T> uses this non-aggregating factory.
unsafe fn CPrintersEnum_CreateInstanceWithIid(
    hwndOwner: HWND,
    dwFlags: DWORD,
    riid: REFIID,
    ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() {
        return E_POINTER;
    }
    *ppv = core::ptr::null_mut();
    let this = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        core::mem::size_of::<CPrintersEnum>(),
    ) as *mut CPrintersEnum;
    if this.is_null() {
        return E_OUTOFMEMORY;
    }
    core::ptr::write(
        this,
        CPrintersEnum {
            IEnumIDList_iface: IEnumIDList {
                lpVtbl: core::ptr::addr_of!(CPRINTERSENUM_IENUMIDLIST_VTBL),
            },
            ref_count: core::sync::atomic::AtomicI32::new(1),
            mpFirst: core::mem::zeroed(),
            mpLast: core::mem::zeroed(),
            mpCurrent: core::mem::zeroed(),
        },
    );
    Shell32RustObjectLock();
    CPrintersEnum_ctor_body(this);
    let mut hr = CPrintersEnum_Initialize_body(this, hwndOwner, dwFlags);
    if hr >= 0 {
        hr = CPrintersEnum_QueryInterface(this, riid, ppv);
    }
    CPrintersEnum_Release(this);
    hr
}

// Deterministic C++ support closure selected from source identifiers.

pub const COLUMN_COMMENTS: UINT = 3;

pub const COLUMN_DOCUMENTS: UINT = 1;

pub const COLUMN_LOCATION: UINT = 4;

pub const COLUMN_MODEL: UINT = 5;

pub const COLUMN_NAME: UINT = 0;

pub const COLUMN_STATUS: UINT = 2;

pub const IDI_SHELL_PRINTER: UINT = 17;

pub const IDS_SHV_COLUMN_COMMENTS: UINT = 321;

pub const IDS_SHV_COLUMN_DOCUMENTS: UINT = 319;

pub const IDS_SHV_COLUMN_LOCATION: UINT = 322;

pub const IDS_SHV_COLUMN_MODEL: UINT = 323;

pub const IDS_SHV_COLUMN_NAME: UINT = 7;

pub const IDS_SHV_COLUMN_STATUS: UINT = 320;

pub const PrinterSHELLVIEWCOLUMNS: UINT = 6;

pub const SFGAO_CANDELETE: SFGAOF = 0x0000_0020;

pub const SFGAO_CANRENAME: SFGAOF = 0x0000_0010;

pub const SFGAO_HASPROPSHEET: SFGAOF = 0x0000_0040;

pub const LVCFMT_LEFT: DWORD = 0x0;

pub const MAXUSHORT: DWORD = 0xffff;

pub const PRINTER_ENUM_LOCAL: DWORD = 0x00000002;

pub const SHCOLSTATE_ONBYDEFAULT: DWORD = 0x10;

pub const SHCOLSTATE_TYPE_STR: DWORD = 1;
pub const SHCOLSTATE_TYPE_INT: DWORD = 2;

pub const SHCONTF_NONFOLDERS: DWORD = 64;

#[repr(C)]
pub struct PRINTER_INFO_4W {
    pub pPrinterName: LPWSTR,
    pub pServerName: LPWSTR,
    pub Attributes: DWORD,
}

// Deterministic from shobjidl.h
#[repr(C)]
pub struct IDefaultExtractIconInit {
    pub lpVtbl: *const IDefaultExtractIconInitVtbl,
}
#[repr(C)]
pub struct IDefaultExtractIconInitVtbl {
    pub QueryInterface: unsafe extern "system" fn(
        This: *mut IDefaultExtractIconInit,
        riid: REFIID,
        ppvObject: *mut *mut core::ffi::c_void,
    ) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IDefaultExtractIconInit) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IDefaultExtractIconInit) -> ULONG,
    pub SetDefaultIcon: unsafe extern "system" fn(
        This: *mut IDefaultExtractIconInit,
        pszFile: LPCWSTR,
        iIcon: i32,
    ) -> HRESULT,
    pub SetFlags:
        unsafe extern "system" fn(This: *mut IDefaultExtractIconInit, uFlags: UINT) -> HRESULT,
    pub SetKey:
        unsafe extern "system" fn(This: *mut IDefaultExtractIconInit, hkey: HKEY) -> HRESULT,
    pub SetNormalIcon: unsafe extern "system" fn(
        This: *mut IDefaultExtractIconInit,
        pszFile: LPCWSTR,
        iIcon: i32,
    ) -> HRESULT,
    pub SetOpenIcon: unsafe extern "system" fn(
        This: *mut IDefaultExtractIconInit,
        pszFile: LPCWSTR,
        iIcon: i32,
    ) -> HRESULT,
    pub SetShortcutIcon: unsafe extern "system" fn(
        This: *mut IDefaultExtractIconInit,
        pszFile: LPCWSTR,
        iIcon: i32,
    ) -> HRESULT,
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
    pub QueryInterface: unsafe extern "system" fn(
        This: *mut IShellBrowser,
        riid: REFIID,
        ppvObject: *mut *mut core::ffi::c_void,
    ) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IShellBrowser) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IShellBrowser) -> ULONG,
    pub GetWindow: unsafe extern "system" fn(This: *mut IShellBrowser, phwnd: *mut HWND) -> HRESULT,
    pub ContextSensitiveHelp:
        unsafe extern "system" fn(This: *mut IShellBrowser, fEnterMode: BOOL) -> HRESULT,
    pub InsertMenusSB: unsafe extern "system" fn(
        This: *mut IShellBrowser,
        hmenuShared: HMENU,
        lpMenuWidths: LPOLEMENUGROUPWIDTHS,
    ) -> HRESULT,
    pub SetMenuSB: unsafe extern "system" fn(
        This: *mut IShellBrowser,
        hmenuShared: HMENU,
        holemenuReserved: HOLEMENU,
        hwndActiveObject: HWND,
    ) -> HRESULT,
    pub RemoveMenusSB:
        unsafe extern "system" fn(This: *mut IShellBrowser, hmenuShared: HMENU) -> HRESULT,
    pub SetStatusTextSB:
        unsafe extern "system" fn(This: *mut IShellBrowser, pszStatusText: LPCOLESTR) -> HRESULT,
    pub EnableModelessSB:
        unsafe extern "system" fn(This: *mut IShellBrowser, fEnable: BOOL) -> HRESULT,
    pub TranslateAcceleratorSB:
        unsafe extern "system" fn(This: *mut IShellBrowser, pmsg: *mut MSG, wID: WORD) -> HRESULT,
    pub BrowseObject: unsafe extern "system" fn(
        This: *mut IShellBrowser,
        pidl: PCUIDLIST_RELATIVE,
        wFlags: UINT,
    ) -> HRESULT,
    pub GetViewStateStream: unsafe extern "system" fn(
        This: *mut IShellBrowser,
        grfMode: DWORD,
        ppStrm: *mut *mut IStream,
    ) -> HRESULT,
    pub GetControlWindow:
        unsafe extern "system" fn(This: *mut IShellBrowser, id: UINT, phwnd: *mut HWND) -> HRESULT,
    pub SendControlMsg: unsafe extern "system" fn(
        This: *mut IShellBrowser,
        id: UINT,
        uMsg: UINT,
        wParam: WPARAM,
        lParam: LPARAM,
        pret: *mut LRESULT,
    ) -> HRESULT,
    pub QueryActiveShellView:
        unsafe extern "system" fn(This: *mut IShellBrowser, ppshv: *mut *mut IShellView) -> HRESULT,
    pub OnViewWindowActive:
        unsafe extern "system" fn(This: *mut IShellBrowser, pshv: *mut IShellView) -> HRESULT,
    pub SetToolbarItems: unsafe extern "system" fn(
        This: *mut IShellBrowser,
        lpButtons: LPTBBUTTONSB,
        nButtons: UINT,
        uFlags: UINT,
    ) -> HRESULT,
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
    pub QueryInterface: unsafe extern "system" fn(
        This: *mut IShellView,
        riid: REFIID,
        ppvObject: *mut *mut core::ffi::c_void,
    ) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IShellView) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IShellView) -> ULONG,
    pub GetWindow: unsafe extern "system" fn(This: *mut IShellView, phwnd: *mut HWND) -> HRESULT,
    pub ContextSensitiveHelp:
        unsafe extern "system" fn(This: *mut IShellView, fEnterMode: BOOL) -> HRESULT,
    pub TranslateAccelerator:
        unsafe extern "system" fn(This: *mut IShellView, pmsg: *mut MSG) -> HRESULT,
    pub EnableModeless: unsafe extern "system" fn(This: *mut IShellView, fEnable: BOOL) -> HRESULT,
    pub UIActivate: unsafe extern "system" fn(This: *mut IShellView, uState: UINT) -> HRESULT,
    pub Refresh: unsafe extern "system" fn(This: *mut IShellView) -> HRESULT,
    pub CreateViewWindow: unsafe extern "system" fn(
        This: *mut IShellView,
        psvPrevious: *mut IShellView,
        lpfs: LPCFOLDERSETTINGS,
        psb: *mut IShellBrowser,
        prcView: *mut RECT,
        phWnd: *mut HWND,
    ) -> HRESULT,
    pub DestroyViewWindow: unsafe extern "system" fn(This: *mut IShellView) -> HRESULT,
    pub GetCurrentInfo:
        unsafe extern "system" fn(This: *mut IShellView, lpfs: LPFOLDERSETTINGS) -> HRESULT,
    pub AddPropertySheetPages: unsafe extern "system" fn(
        This: *mut IShellView,
        dwReserved: DWORD,
        pfn: LPFNSVADDPROPSHEETPAGE,
        lparam: LPARAM,
    ) -> HRESULT,
    pub SaveViewState: unsafe extern "system" fn(This: *mut IShellView) -> HRESULT,
    pub SelectItem: unsafe extern "system" fn(
        This: *mut IShellView,
        pidlItem: PCUITEMID_CHILD,
        uFlags: SVSIF,
    ) -> HRESULT,
    pub GetItemObject: unsafe extern "system" fn(
        This: *mut IShellView,
        uItem: UINT,
        riid: REFIID,
        ppv: *mut *mut core::ffi::c_void,
    ) -> HRESULT,
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
pub static PrinterSFHeader: [shvheader; 6] = [
    shvheader {
        colnameid: (IDS_SHV_COLUMN_NAME) as WORD,
        colstate: (SHCOLSTATE_TYPE_STR | SHCOLSTATE_ONBYDEFAULT) as WORD,
        fmt: (LVCFMT_LEFT) as WORD,
        cxChar: (15) as WORD,
    },
    shvheader {
        colnameid: (IDS_SHV_COLUMN_DOCUMENTS) as WORD,
        colstate: (SHCOLSTATE_TYPE_INT | SHCOLSTATE_ONBYDEFAULT) as WORD,
        fmt: (LVCFMT_LEFT) as WORD,
        cxChar: (15) as WORD,
    },
    shvheader {
        colnameid: (IDS_SHV_COLUMN_STATUS) as WORD,
        colstate: (SHCOLSTATE_TYPE_STR | SHCOLSTATE_ONBYDEFAULT) as WORD,
        fmt: (LVCFMT_LEFT) as WORD,
        cxChar: (15) as WORD,
    },
    shvheader {
        colnameid: (IDS_SHV_COLUMN_COMMENTS) as WORD,
        colstate: (SHCOLSTATE_TYPE_STR | SHCOLSTATE_ONBYDEFAULT) as WORD,
        fmt: (LVCFMT_LEFT) as WORD,
        cxChar: (15) as WORD,
    },
    shvheader {
        colnameid: (IDS_SHV_COLUMN_LOCATION) as WORD,
        colstate: (SHCOLSTATE_TYPE_STR | SHCOLSTATE_ONBYDEFAULT) as WORD,
        fmt: (LVCFMT_LEFT) as WORD,
        cxChar: (15) as WORD,
    },
    shvheader {
        colnameid: (IDS_SHV_COLUMN_MODEL) as WORD,
        colstate: (SHCOLSTATE_TYPE_STR | SHCOLSTATE_ONBYDEFAULT) as WORD,
        fmt: (LVCFMT_LEFT) as WORD,
        cxChar: (15) as WORD,
    },
];

extern "system" {
    fn lstrlenW(text: LPCWSTR) -> INT;
    #[link_name = "CPrintersExtractIconW_RustBridge"]
    fn CPrintersExtractIconW_CreateInstane(
        pidl: LPCITEMIDLIST,
        riid: REFIID,
        out: *mut LPVOID,
    ) -> HRESULT;
    #[link_name = "SHAlloc"]
    fn cpp_com_SHAlloc(bytes: SIZE_T) -> LPVOID;
    fn EnumPrintersW(
        flags: DWORD,
        name: LPWSTR,
        level: DWORD,
        buffer: LPBYTE,
        buffer_size: DWORD,
        needed: *mut DWORD,
        returned: *mut DWORD,
    ) -> BOOL;
}

extern "C" {
    #[link_name = "memcpy"]
    fn cpp_com_memcpy(
        dst: *mut core::ffi::c_void,
        src: *const core::ffi::c_void,
        bytes: usize,
    ) -> *mut core::ffi::c_void;
}

#[inline]
unsafe fn memcpy<D, S, N: CppComSize>(
    dst: *mut D,
    src: *const S,
    bytes: N,
) -> *mut core::ffi::c_void {
    cpp_com_memcpy(dst.cast(), src.cast(), bytes.cpp_com_size())
}

extern "system" {
    #[link_name = "SHFree"]
    fn cpp_com_SHFree(memory: LPVOID);
}
#[inline]
unsafe fn SHFree<T>(memory: *mut T) {
    cpp_com_SHFree(memory.cast());
}

#[inline]
unsafe fn SHAlloc(bytes: SIZE_T) -> LPVOID {
    cpp_com_SHAlloc(bytes)
}

extern "C" {
    static CLSID_Printers: GUID;
    static IID_IContextMenu: GUID;
    static IID_IDropTarget: GUID;
    static IID_IExtractIconA: GUID;
    static IID_IExtractIconW: GUID;
}

unsafe fn cpp_com_SHSetStrRetW(out: *mut STRRET, value: LPCWSTR) -> HRESULT {
    let len = lstrlenW(value) as usize;
    let bytes = (len + 1) * core::mem::size_of::<WCHAR>();
    let buffer = CoTaskMemAlloc((bytes) as SIZE_T) as LPWSTR;
    if buffer.is_null() {
        return E_OUTOFMEMORY;
    }
    let _ = memcpy(buffer, value, bytes);
    (*out).uType = STRRET_WSTR;
    (*out).u.pOleStr = buffer;
    S_OK
}
unsafe fn cpp_com_SHSetStrRetResource(out: *mut STRRET, id: DWORD) -> HRESULT {
    let mut buffer = [0u16; MAX_PATH];
    if LoadStringW(shell32_hInstance, id, buffer.as_mut_ptr(), MAX_PATH as INT) == 0 {
        return E_FAIL;
    }
    cpp_com_SHSetStrRetW(out, buffer.as_ptr())
}

pub struct CppComPtrGuard<T> {
    slot: *mut *mut T,
}
impl<T> CppComPtrGuard<T> {
    pub const fn new(slot: *mut *mut T) -> Self {
        Self { slot }
    }
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

use crate::shellcom::*;
// The crate's own functions: the definition is the declaration.
use crate::wine::pidl::ILClone;
use crate::def_view::SHCreateShellFolderView;
use crate::wine::pidl::_ILGetDataPointer;
// One declaration per external symbol: island.rs.
use crate::island::CoTaskMemAlloc;
use crate::island::LoadStringW;
use crate::types::*;
use core::ffi::c_void;
use core::mem::size_of;

pub unsafe fn CPrintersEnum_dtor_body(this: *mut CPrintersEnum) -> () {}

pub unsafe fn CPrintersEnum_CEnumIDListBase_base_dtor_body(this: *mut CPrintersEnum) -> () {
    CPrintersEnum_DeleteList_body(this);
}

pub unsafe fn CPrintersEnum_AddToEnumList_body(
    this: *mut CPrintersEnum,
    pidl: LPITEMIDLIST,
) -> BOOL {
    if pidl.is_null() {
        return FALSE;
    }
    let pNew: *mut ENUMLIST = SHAlloc((core::mem::size_of::<ENUMLIST>()) as SIZE_T) as *mut ENUMLIST;
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

pub unsafe fn CPrintersEnum_DeleteList_body(this: *mut CPrintersEnum) -> BOOL {
    while !(*this).mpFirst.is_null() {
        let p_delete = (*this).mpFirst;
        (*this).mpFirst = (*p_delete).pNext;
        SHFree((*p_delete).pidl);
        SHFree(p_delete);
    }
    (*this).mpFirst = core::ptr::null_mut();
    (*this).mpLast = core::ptr::null_mut();
    (*this).mpCurrent = core::ptr::null_mut();
    TRUE
}

pub unsafe fn CPrintersEnum_ctor_body(this: *mut CPrintersEnum) -> () {}

pub unsafe fn CPrintersEnum_Initialize_body(
    this: *mut CPrintersEnum,
    hwndOwner: HWND,
    dwFlags: DWORD,
) -> HRESULT {
    if CPrintersEnum_CreatePrintersEnumList_body(this, dwFlags) == FALSE {
        return E_FAIL;
    }
    return S_OK;
}

pub unsafe fn CPrintersEnum_AppendItemsFromEnumerator_body(
    this: *mut CPrintersEnum,
    pEnum: *mut IEnumIDList,
) -> HRESULT {
    if pEnum.is_null() {
        return E_INVALIDARG;
    }
    ((*(*pEnum).lpVtbl).Reset)(pEnum);
    let mut pidl: LPITEMIDLIST = core::ptr::null_mut();
    let mut dwFetched: u32 = 0;
    while S_OK == ((*(*pEnum).lpVtbl).Next)(pEnum, 1, &mut pidl, &mut dwFetched) && dwFetched != 0 {
        CPrintersEnum_AddToEnumList_body(this, pidl);
    }
    S_OK
}

pub unsafe fn _ILCreatePrinterItem_body(pi: *mut PRINTER_INFO_4W) -> LPITEMIDLIST {
    let mut tmp: PIDLDATA = core::mem::zeroed();
    let size0 = (core::ptr::addr_of!(tmp.u.cprinter.szName) as *const u8)
        .offset_from(core::ptr::addr_of!(tmp.u.cprinter) as *const u8) as i32;
    let mut size = size0;

    let mut cchPrinterName: usize = 0 as usize;
    let mut cchServerName: usize = 0 as usize;

    if !(*pi).pPrinterName.is_null() {
        cchPrinterName = wcslen((*pi).pPrinterName);
    }
    if !(*pi).pServerName.is_null() {
        cchServerName = wcslen((*pi).pServerName);
    }
    if (cchPrinterName + cchServerName) > ((MAXUSHORT as usize) - (2 as usize)) {
        return core::ptr::null_mut();
    }

    tmp.type_ = 0x00;
    tmp.u.cprinter.dummy = 0xFF;
    if !(*pi).pPrinterName.is_null() {
        tmp.u.cprinter.offsServer = (cchPrinterName + 1 as usize) as u16;
    } else {
        tmp.u.cprinter.offsServer = 1;
    }

    size += tmp.u.cprinter.offsServer as i32 * size_of::<u16>() as i32;
    if !(*pi).pServerName.is_null() {
        size += ((cchServerName + 1 as usize) as i32) * size_of::<u16>() as i32;
    } else {
        size += size_of::<u16>() as i32;
    }

    let pidl = SHAlloc((size + 4) as SIZE_T) as LPITEMIDLIST;
    if pidl.is_null() {
        return pidl;
    }

    (*pidl).mkid.cb = (size + 2) as u16;
    memcpy(
        &raw mut (*pidl).mkid.abID as *mut _,
        &raw const tmp as *const _,
        (2 + size0) as usize,
    );

    let p: *mut PIDLPrinterStruct =
        core::ptr::addr_of_mut!((*(&raw mut (*pidl).mkid.abID as *mut PIDLDATA)).u.cprinter);

    (*p).Attributes = (*pi).Attributes;
    if !(*pi).pPrinterName.is_null() {
        wcscpy(
            core::ptr::addr_of_mut!((*p).szName).cast(),
            (*pi).pPrinterName,
        );
    } else {
        (*p).szName[0 as usize] = 0x0000u16;
    }

    if !(*pi).pServerName.is_null() {
        let name = core::ptr::addr_of_mut!((*p).szName).cast::<WCHAR>();
        let server_offset = core::ptr::read_unaligned(core::ptr::addr_of!((*p).offsServer));
        wcscpy(name.add(server_offset as usize), (*pi).pServerName);
    } else {
        // szName is flexible (`WCHAR szName[1]`, "followed by display name and
        // comment string"), so bindgen's `[T; 1]` panics for any non-zero offset.
        // The correct walk is already four lines above this one.
        let name = core::ptr::addr_of_mut!((*p).szName).cast::<WCHAR>();
        let server_offset = core::ptr::read_unaligned(core::ptr::addr_of!((*p).offsServer));
        name.add(server_offset as usize).write(0x0000u16);
    }

    *((pidl as *mut u8).add((size + 2) as usize) as *mut u16) = 0 as u16;

    pidl
}

pub unsafe fn CPrintersEnum_Next_body(
    this: *mut CPrintersEnum,
    celt: ULONG,
    rgelt: *mut PITEMID_CHILD,
    pceltFetched: *mut ULONG,
) -> HRESULT {
    let mut i: ULONG;
    let mut hr: HRESULT = S_OK;
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

        *rgelt.offset(i as isize) = temp;
        (*this).mpCurrent = (*(*this).mpCurrent).pNext;
        i += 1;
    }

    if !pceltFetched.is_null() {
        *pceltFetched = i;
    }

    hr
}

pub unsafe fn CPrintersEnum_CreatePrintersEnumList_body(
    this: *mut CPrintersEnum,
    dwFlags: DWORD,
) -> BOOL {
    let mut ret = TRUE;

    if (dwFlags & SHCONTF_NONFOLDERS) != 0 {
        let mut needed: DWORD = 0;
        let mut num: DWORD = 0;

        EnumPrintersW(
            PRINTER_ENUM_LOCAL,
            core::ptr::null_mut(),
            4,
            core::ptr::null_mut(),
            0,
            &mut needed,
            &mut num,
        );
        if needed == 0 {
            return ret;
        }

        let pi: *mut PRINTER_INFO_4W =
            HeapAlloc(GetProcessHeap(), 0, needed) as *mut PRINTER_INFO_4W;

        if EnumPrintersW(
            PRINTER_ENUM_LOCAL,
            core::ptr::null_mut(),
            4,
            pi as *mut u8,
            needed,
            &mut needed,
            &mut num,
        ) == 0
        {
            HeapFree(GetProcessHeap(), 0, pi as _);
            return FALSE;
        }

        let mut i: DWORD = 0;
        while i < num {
            let pidl: LPITEMIDLIST = _ILCreatePrinterItem_body(pi.add(i as usize));
            if !pidl.is_null() {
                if CPrintersEnum_AddToEnumList_body(this, pidl) == 0 {
                    SHFree(pidl);
                }
            }
            i += 1;
        }

        HeapFree(GetProcessHeap(), 0, pi as _);
    }

    ret
}

pub unsafe fn CPrintersEnum_Skip_body(this: *mut CPrintersEnum, celt: ULONG) -> HRESULT {
    let mut dwIndex: u32 = 0;
    let mut hr: i32 = S_OK;

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

pub unsafe fn CPrintersEnum_Reset_body(this: *mut CPrintersEnum) -> HRESULT {
    (*this).mpCurrent = (*this).mpFirst;
    S_OK
}

pub unsafe fn CPrintersEnum_Clone_body(
    this: *mut CPrintersEnum,
    ppenum: *mut *mut IEnumIDList,
) -> HRESULT {
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn CPrintersEnum_CreateInstance(
    hwndOwner: HWND,
    dwFlags: DWORD,
    ppEnumIDList: *mut *mut IEnumIDList,
) -> HRESULT {
    CPrintersEnum_CreateInstanceWithIid(
        hwndOwner,
        dwFlags,
        core::ptr::addr_of!(IID_IEnumIDList),
        ppEnumIDList.cast(),
    )
}
