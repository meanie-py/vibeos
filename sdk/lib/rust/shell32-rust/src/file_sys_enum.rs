//! `CFileSysEnum` — the file-local filesystem enumerator from
//! `dll/win32/shell32/folders/CFSFolder.cpp`, translated with `--cpp-com --internal`.
//!
//! It is not an activatable object: `CFSFolder::EnumObjects` builds one through
//! `ShellObjectCreator<T>`, so there is no `OBJECT_ENTRY` and no class factory. Its
//! `CEnumIDListBase` list methods are FLATTENED into the class here — the same shape
//! `printers_enum.rs` already uses — because the base is a concrete C++ class whose `.cpp`
//! bodies belong to whichever derived enumerator instantiates them.
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

// Deterministic COM scaffold for CFileSysEnum; method bodies are generated separately.
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

// The `CEnumIDListBase` list node (wine/enumidlist.h). Flattened in here for the same reason
// the base's methods are: it is a concrete C++ class, so each derived enumerator owns a copy.
#[repr(C)]
pub struct ENUMLIST {
    pub pNext: *mut ENUMLIST,
    pub pidl: LPITEMIDLIST,
}

#[repr(C)]
pub struct CFileSysEnum {
    pub IEnumIDList_iface: IEnumIDList,
    pub ref_count: core::sync::atomic::AtomicI32,
    pub mpFirst: *mut ENUMLIST,
    pub mpLast: *mut ENUMLIST,
    pub mpCurrent: *mut ENUMLIST,
}

unsafe extern "system" fn CFileSysEnum_IEnumIDList_QueryInterface(This: *mut IEnumIDList, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFileSysEnum, IEnumIDList_iface)) as *mut CFileSysEnum;
    CFileSysEnum_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CFileSysEnum_IEnumIDList_AddRef(This: *mut IEnumIDList) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFileSysEnum, IEnumIDList_iface)) as *mut CFileSysEnum;
    CFileSysEnum_AddRef(this)
}
unsafe extern "system" fn CFileSysEnum_IEnumIDList_Release(This: *mut IEnumIDList) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFileSysEnum, IEnumIDList_iface)) as *mut CFileSysEnum;
    CFileSysEnum_Release(this)
}
unsafe extern "system" fn CFileSysEnum_IEnumIDList_Next(This: *mut IEnumIDList, celt: ULONG, rgelt: *mut PITEMID_CHILD, pceltFetched: *mut ULONG) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFileSysEnum, IEnumIDList_iface)) as *mut CFileSysEnum;
    CFileSysEnum_Next_body(this, celt, rgelt, pceltFetched)
}
unsafe extern "system" fn CFileSysEnum_IEnumIDList_Skip(This: *mut IEnumIDList, celt: ULONG) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFileSysEnum, IEnumIDList_iface)) as *mut CFileSysEnum;
    CFileSysEnum_Skip_body(this, celt)
}
unsafe extern "system" fn CFileSysEnum_IEnumIDList_Reset(This: *mut IEnumIDList) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFileSysEnum, IEnumIDList_iface)) as *mut CFileSysEnum;
    CFileSysEnum_Reset_body(this)
}
unsafe extern "system" fn CFileSysEnum_IEnumIDList_Clone(This: *mut IEnumIDList, ppenum: *mut *mut IEnumIDList) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFileSysEnum, IEnumIDList_iface)) as *mut CFileSysEnum;
    CFileSysEnum_Clone_body(this, ppenum)
}

pub static CFILESYSENUM_IENUMIDLIST_VTBL: IEnumIDListVtbl = IEnumIDListVtbl {
    QueryInterface: CFileSysEnum_IEnumIDList_QueryInterface,
    AddRef: CFileSysEnum_IEnumIDList_AddRef,
    Release: CFileSysEnum_IEnumIDList_Release,
    Next: CFileSysEnum_IEnumIDList_Next,
    Skip: CFileSysEnum_IEnumIDList_Skip,
    Reset: CFileSysEnum_IEnumIDList_Reset,
    Clone: CFileSysEnum_IEnumIDList_Clone,
};

unsafe fn CFileSysEnum_QueryInterface(
    this: *mut CFileSysEnum, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IUnknown)) || cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IEnumIDList)) {
        *ppv = core::ptr::addr_of_mut!((*this).IEnumIDList_iface) as *mut core::ffi::c_void;
    }
    else { return E_NOINTERFACE; }
    CFileSysEnum_AddRef(this);
    S_OK
}

unsafe fn CFileSysEnum_AddRef(this: *mut CFileSysEnum) -> ULONG {
    (*this).ref_count.fetch_add(1, core::sync::atomic::Ordering::SeqCst) as ULONG + 1
}

unsafe fn CFileSysEnum_Release(this: *mut CFileSysEnum) -> ULONG {
    let refs = (*this).ref_count.fetch_sub(1, core::sync::atomic::Ordering::SeqCst) - 1;
    if refs == 0 {
        CFileSysEnum_dtor_body(this);
        CFileSysEnum_CEnumIDListBase_base_dtor_body(this);
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
pub unsafe fn CFileSysEnum_CreateInstance(
    sPathTarget: LPWSTR,
    dwFlags: DWORD,
    riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    let this = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<CFileSysEnum>()) as *mut CFileSysEnum;
    if this.is_null() { return E_OUTOFMEMORY; }
    core::ptr::write(this, CFileSysEnum {
        IEnumIDList_iface: IEnumIDList { lpVtbl: core::ptr::addr_of!(CFILESYSENUM_IENUMIDLIST_VTBL) },
        ref_count: core::sync::atomic::AtomicI32::new(1),
        mpFirst: core::mem::zeroed(),
        mpLast: core::mem::zeroed(),
        mpCurrent: core::mem::zeroed(),
    });
    Shell32RustObjectLock();
    CFileSysEnum_ctor_body(this);
    let mut hr = CFileSysEnum_Initialize_body(this, sPathTarget, dwFlags);
    if hr >= 0 {
        hr = CFileSysEnum_QueryInterface(this, riid, ppv);
    }
    CFileSysEnum_Release(this);
    hr
}

// Deterministic C++ support closure selected from source identifiers.

pub const IDI_SHELL_DOCUMENT: UINT = 1;

pub const IDI_SHELL_EXE: UINT = 3;

pub const IDI_SHELL_FOLDER: UINT = 4;

pub const IDS_SHV_COLUMN_ATTRIBUTES: UINT = 11;

pub const IDS_SHV_COLUMN_COMMENTS: UINT = 321;

pub const IDS_SHV_COLUMN_MODIFIED: UINT = 10;

pub const IDS_SHV_COLUMN_NAME: UINT = 7;

pub const IDS_SHV_COLUMN_SIZE: UINT = 8;

pub const IDS_SHV_COLUMN_TYPE: UINT = 9;

pub const SFGAO_CANCOPY: SFGAOF = 0x0000_0001;

pub const SFGAO_CANDELETE: SFGAOF = 0x0000_0020;

pub const SFGAO_CANLINK: SFGAOF = 0x0000_0004;

pub const SFGAO_CANRENAME: SFGAOF = 0x0000_0010;

pub const SFGAO_HASPROPSHEET: SFGAOF = 0x0000_0040;

pub const CHARS_IN_GUID: DWORD = 39;

pub const CLR_INVALID: DWORD = 0xFFFFFFFF;

pub const CSIDL_FLAG_CREATE: DWORD = 0x8000;

pub const DATE_SHORTDATE: DWORD = 1;

pub const DFM_INVOKECOMMAND: DWORD = 2;

pub const DFM_MERGECONTEXTMENU: DWORD = 1;

pub const ERROR_BAD_DEVICE: DWORD = 1200;

pub const ERROR_FILE_NOT_FOUND: DWORD = 2;

pub const ERROR_NO_MORE_FILES: DWORD = 18;

pub const ERROR_PATH_NOT_FOUND: DWORD = 3;

pub const FACILITY_SHELL: DWORD = 39;

pub const FILE_ATTRIBUTE_COMPRESSED: DWORD = 0x00000800;

pub const FILE_ATTRIBUTE_DIRECTORY: DWORD = 0x00000010;

pub const FILE_ATTRIBUTE_ENCRYPTED: DWORD = 0x00004000;

pub const FILE_ATTRIBUTE_HIDDEN: DWORD = 0x00000002;

pub const FILE_ATTRIBUTE_NORMAL: DWORD = 0x00000080;

pub const FILE_ATTRIBUTE_READONLY: DWORD = 0x00000001;

pub const FILE_ATTRIBUTE_SYSTEM: DWORD = 0x00000004;

pub const FOF_ALLOWUNDO: DWORD = 0x0040;

pub const FOF_SILENT: DWORD = 0x0004;

pub const FO_RENAME: DWORD = 4;

pub const GIL_DEFAULTICON: DWORD = 0x0040;

pub const GIL_OPENICON: DWORD = 0x0001;

pub const ILGDN_FORPARSING: DWORD = 0;

pub const IMAGE_BITMAP: DWORD = 0;

pub const KEY_READ: DWORD = 0x20019;

pub const LR_CREATEDIBSECTION: DWORD = 8192;

pub const LVCFMT_LEFT: DWORD = 0x0;

pub const LVCFMT_RIGHT: DWORD = 0x1;

pub const MFS_ENABLED: DWORD = 0;

pub const MM_ADDSEPARATOR: DWORD = 0x00000001;

pub const PIDSI_COMMENTS: DWORD = 0x00000006;

pub const PID_DESCRIPTIONID: DWORD = 2;

pub const SEVERITY_ERROR: DWORD = 1;

pub const SFGAO_COMPRESSED: DWORD = 0x04000000;

pub const SFGAO_DROPTARGET: DWORD = 0x00000100;

pub const SFGAO_ENCRYPTED: DWORD = 0x00002000;

pub const SFGAO_GHOSTED: DWORD = 0x00008000;

pub const SFGAO_HIDDEN: DWORD = 0x00080000;

pub const SFGAO_LINK: DWORD = 0x00010000;

pub const SFGAO_NONENUMERATED: DWORD = 0x00100000;

pub const SFGAO_READONLY: DWORD = 0x00040000;

pub const SFGAO_STREAM: DWORD = 0x00400000;

pub const SFGAO_SYSTEM: DWORD = 0x00001000;

pub const SFVM_GETCOMMANDDIR: DWORD = 33;

pub const SFVM_GET_CUSTOMVIEWINFO: DWORD = 77;

pub const SHCOLSTATE_ONBYDEFAULT: DWORD = 0x10;

pub const SHCOLSTATE_SLOW: DWORD = 0x20;

pub const SHCOLSTATE_TYPE_DATE: DWORD = 3;

pub const SHCOLSTATE_TYPE_INT: DWORD = 2;

pub const SHCOLSTATE_TYPE_STR: DWORD = 1;

pub const SHCONTF_FOLDERS: DWORD = 32;

pub const SHCONTF_INCLUDEHIDDEN: DWORD = 128;

pub const SHCONTF_NONFOLDERS: DWORD = 64;

pub const SHDID_FS_DIRECTORY: DWORD = 3;

pub const SHDID_FS_FILE: DWORD = 2;

pub const SHDID_FS_OTHER: DWORD = 4;

pub const SHGDN_INFOLDER: DWORD = 1;

pub const SHGFI_PIDL: DWORD = 8;

pub const SHGFI_TYPENAME: DWORD = 1024;

pub const SSF_SHOWEXTENSIONS: DWORD = 0x00000002;

pub const STATUS_SUCCESS: DWORD = 0;

pub const STGM_CREATE: DWORD = 0x00001000;

pub const STGM_READ: DWORD = 0x00000000;

pub const TIME_NOSECONDS: DWORD = 2;

pub const ULONG_MAX: DWORD = 0xffffffff;

pub const VT_DATE: DWORD = 7;

pub const VT_UI4: DWORD = 19;

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
pub struct QCMINFO {
    pub hmenu: HMENU,
    pub indexMenu: UINT,
    pub idCmdFirst: UINT,
    pub idCmdLast: UINT,
    pub pIdMap: *const QCMINFO_IDMAP,
}

#[repr(C)]
pub struct QCMINFO_IDMAP {
    pub nMaxIds: UINT,
    pub pIdList: [QCMINFO_IDMAP_PLACEMENT; 1],
}

#[repr(C)]
pub struct QCMINFO_IDMAP_PLACEMENT {
    pub id: UINT,
    pub fFlags: UINT,
}

#[repr(C)]
pub struct SHDESCRIPTIONID {
    pub dwDescriptionId: DWORD,
    pub clsid: CLSID,
}

// WIN32_FIND_DATAW is NOT redefined here: this module is the same translation unit as
// `cfs_folder.rs` (see the `use crate::cfs_folder::*;` glob below), which already defines it
// and every FindFirstFileW/FindNextFileW/_ILCreateFromFindDataW extern is typed against that
// one. A second identically-shaped struct here would be a DISTINCT nominal type and break
// every call across the TU boundary.

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

// Deterministic from objidl.h
#[repr(C)]
pub struct IPersistFile {
    pub lpVtbl: *const IPersistFileVtbl,
}
#[repr(C)]
pub struct IPersistFileVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IPersistFile, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IPersistFile) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IPersistFile) -> ULONG,
    pub GetClassID: unsafe extern "system" fn(This: *mut IPersistFile, pClassID: *mut CLSID) -> HRESULT,
    pub IsDirty: unsafe extern "system" fn(This: *mut IPersistFile) -> HRESULT,
    pub Load: unsafe extern "system" fn(This: *mut IPersistFile, pszFileName: LPCOLESTR, dwMode: DWORD) -> HRESULT,
    pub Save: unsafe extern "system" fn(This: *mut IPersistFile, pszFileName: LPCOLESTR, fRemember: BOOL) -> HRESULT,
    pub SaveCompleted: unsafe extern "system" fn(This: *mut IPersistFile, pszFileName: LPCOLESTR) -> HRESULT,
    pub GetCurFile: unsafe extern "system" fn(This: *mut IPersistFile, ppszFileName: *mut LPOLESTR) -> HRESULT,
}
extern "C" {
    pub static IID_IPersistFile: IID;
}

// Deterministic from ocidl.h
#[repr(C)]
pub struct IPicture {
    pub lpVtbl: *const IPictureVtbl,
}
#[repr(C)]
pub struct IPictureVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IPicture, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IPicture) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IPicture) -> ULONG,
    pub get_Handle: unsafe extern "system" fn(This: *mut IPicture, pHandle: *mut OLE_HANDLE) -> HRESULT,
    pub get_hPal: unsafe extern "system" fn(This: *mut IPicture, phPal: *mut OLE_HANDLE) -> HRESULT,
    pub get_Type: unsafe extern "system" fn(This: *mut IPicture, pType: *mut SHORT) -> HRESULT,
    pub get_Width: unsafe extern "system" fn(This: *mut IPicture, pWidth: *mut OLE_XSIZE_HIMETRIC) -> HRESULT,
    pub get_Height: unsafe extern "system" fn(This: *mut IPicture, pHeight: *mut OLE_YSIZE_HIMETRIC) -> HRESULT,
    pub Render: unsafe extern "system" fn(This: *mut IPicture, hdc: HDC, x: LONG, y: LONG, cx: LONG, cy: LONG, xSrc: OLE_XPOS_HIMETRIC, ySrc: OLE_YPOS_HIMETRIC, cxSrc: OLE_XSIZE_HIMETRIC, cySrc: OLE_YSIZE_HIMETRIC, pRcWBounds: LPCRECT) -> HRESULT,
    pub set_hPal: unsafe extern "system" fn(This: *mut IPicture, hPal: OLE_HANDLE) -> HRESULT,
    pub get_CurDC: unsafe extern "system" fn(This: *mut IPicture, phDC: *mut HDC) -> HRESULT,
    pub SelectPicture: unsafe extern "system" fn(This: *mut IPicture, hDCIn: HDC, phDCOut: *mut HDC, phBmpOut: *mut OLE_HANDLE) -> HRESULT,
    pub get_KeepOriginalFormat: unsafe extern "system" fn(This: *mut IPicture, pKeep: *mut BOOL) -> HRESULT,
    pub put_KeepOriginalFormat: unsafe extern "system" fn(This: *mut IPicture, keep: BOOL) -> HRESULT,
    pub PictureChanged: unsafe extern "system" fn(This: *mut IPicture) -> HRESULT,
    pub SaveAsFile: unsafe extern "system" fn(This: *mut IPicture, pStream: LPSTREAM, fSaveMemCopy: BOOL, pCbSize: *mut LONG) -> HRESULT,
    pub get_Attributes: unsafe extern "system" fn(This: *mut IPicture, pDwAttr: *mut DWORD) -> HRESULT,
}
extern "C" {
    pub static IID_IPicture: IID;
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
pub struct shvheader {
    pub colnameid: WORD, pub colstate: WORD,
    pub fmt: WORD, pub cxChar: WORD,
}
pub static GenericSFHeader: [shvheader; 6] = [
    shvheader { colnameid: (IDS_SHV_COLUMN_NAME) as WORD, colstate: (SHCOLSTATE_TYPE_STR | SHCOLSTATE_ONBYDEFAULT) as WORD, fmt: (LVCFMT_LEFT) as WORD, cxChar: (18) as WORD },
    shvheader { colnameid: (IDS_SHV_COLUMN_SIZE) as WORD, colstate: (SHCOLSTATE_TYPE_INT | SHCOLSTATE_ONBYDEFAULT) as WORD, fmt: (LVCFMT_RIGHT) as WORD, cxChar: (10) as WORD },
    shvheader { colnameid: (IDS_SHV_COLUMN_TYPE) as WORD, colstate: (SHCOLSTATE_TYPE_STR | SHCOLSTATE_ONBYDEFAULT) as WORD, fmt: (LVCFMT_LEFT) as WORD, cxChar: (10) as WORD },
    shvheader { colnameid: (IDS_SHV_COLUMN_MODIFIED) as WORD, colstate: (SHCOLSTATE_TYPE_DATE | SHCOLSTATE_ONBYDEFAULT) as WORD, fmt: (LVCFMT_LEFT) as WORD, cxChar: (15) as WORD },
    shvheader { colnameid: (IDS_SHV_COLUMN_ATTRIBUTES) as WORD, colstate: (SHCOLSTATE_TYPE_STR | SHCOLSTATE_ONBYDEFAULT) as WORD, fmt: (LVCFMT_LEFT) as WORD, cxChar: (8) as WORD },
    shvheader { colnameid: (IDS_SHV_COLUMN_COMMENTS) as WORD, colstate: (SHCOLSTATE_TYPE_STR | SHCOLSTATE_SLOW) as WORD, fmt: (LVCFMT_LEFT) as WORD, cxChar: (10) as WORD },
];

extern "system" {
    fn DestroyMenu(menu: HMENU) -> BOOL;
    fn ExpandEnvironmentStringsW(source: LPCWSTR, dest: LPWSTR, size: DWORD) -> DWORD;
    fn RegCloseKey(key: HKEY) -> LONG;
    fn RegGetValueW(key: HKEY, subkey: LPCWSTR, value: LPCWSTR, flags: DWORD, kind: LPDWORD, data: LPVOID, bytes: LPDWORD) -> LONG;
    fn RegOpenKeyExW(key: HKEY, subkey: LPCWSTR, options: DWORD, access: REGSAM, result: PHKEY) -> LONG;
    fn RegQueryValueExW(key: HKEY, value: LPCWSTR, reserved: LPDWORD, kind: LPDWORD, data: PBYTE, bytes: LPDWORD) -> LONG;
    fn SHGetFileInfoW(path: LPCWSTR, attributes: DWORD, info: *mut SHFILEINFOW, bytes: UINT, flags: UINT) -> DWORD_PTR;
    fn SHGetPathFromIDListW(pidl: LPCITEMIDLIST, path: LPWSTR) -> BOOL;
    fn StrRetToStrW(value: *mut STRRET, pidl: PCUITEMID_CHILD, out: *mut LPWSTR) -> HRESULT;
    // shlobj.h: REFGUID guid, LPCWSTR pszValueName, BOOL bUseHKCU, BOOL bCreate, PHKEY phkey.
    // `GUID` here is `crate::shellcom::GUID` (glob-imported below), not `ntdll_sys::nt::GUID`
    // as the bindgen island's own copy of this symbol uses — both are repr(C) with identical
    // field layout, so the FFI call is sound either way; only the Rust nominal type differs,
    // and this file's other CLSID plumbing (GetCLSIDForFileTypeFromExtension_body etc.) is
    // already typed against the shellcom one.
    fn SHRegGetCLSIDKeyW(guid: *const GUID, pszValueName: LPCWSTR, bUseHKCU: BOOL, bCreate: BOOL, phkey: PHKEY) -> HRESULT;
    fn FindNextFileW(find: HANDLE, data: *mut WIN32_FIND_DATAW) -> BOOL;
    fn GetLastError() -> DWORD;
    fn lstrlenW(text: LPCWSTR) -> INT;
    fn CoTaskMemAlloc(bytes: SIZE_T) -> LPVOID;
    fn LoadStringW(instance: HINSTANCE, id: UINT, buffer: LPWSTR, capacity: INT) -> INT;
    fn CoTaskMemFree(memory: LPVOID);
    fn SHStrDupW(source: LPCWSTR, out: *mut LPWSTR) -> HRESULT;
    fn ILClone(pidl: LPCITEMIDLIST) -> LPITEMIDLIST;
    fn ILFindLastID(pidl: LPCITEMIDLIST) -> LPITEMIDLIST;
    fn ILFree(pidl: LPITEMIDLIST);
    fn SHCreateShellFolderView(create: *const SFV_CREATE, view: *mut *mut IShellView) -> HRESULT;
    fn ILGetNext(pidl: PCUIDLIST_RELATIVE) -> LPITEMIDLIST;
}

extern "C" {
    fn HLM_GetIconW(index: UINT, destination: LPWSTR, length: DWORD, icon_index: *mut INT) -> BOOL;
    fn _ILGetDataPointer(pidl: LPCITEMIDLIST) -> *mut PIDLDATA;
    #[link_name = "CPrinterFolder_CompareDetails_RustBridge"] fn SHELL32_CompareDetails(folder: *mut IShellFolder2, lparam: LPARAM, left: PCUIDLIST_RELATIVE, right: PCUIDLIST_RELATIVE) -> HRESULT;
    #[link_name = "memcpy"] fn cpp_com_memcpy(dst: *mut core::ffi::c_void, src: *const core::ffi::c_void, bytes: usize) -> *mut core::ffi::c_void;
    // wsprintfW is variadic, so it must be declared `extern "C"` (Rust variadics only support
    // the C calling convention) even though the real symbol is WINAPI/stdcall.
    fn wsprintfW(buffer: LPWSTR, format: LPCWSTR, ...) -> INT;
}

#[inline]
unsafe fn memcpy<D, S, N: CppComSize>(dst: *mut D, src: *const S, bytes: N) -> *mut core::ffi::c_void {
    cpp_com_memcpy(dst.cast(), src.cast(), bytes.cpp_com_size())
}

extern "system" {
    #[link_name = "SHFree"] fn cpp_com_SHFree(memory: LPVOID);
    #[link_name = "SHAlloc"] fn cpp_com_SHAlloc(bytes: SIZE_T) -> LPVOID;
}
#[inline]
unsafe fn SHFree<T>(memory: *mut T) { cpp_com_SHFree(memory.cast()); }
#[inline]
unsafe fn SHAlloc(bytes: SIZE_T) -> LPVOID { cpp_com_SHAlloc(bytes) }

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
    static CLSID_NULL: GUID;
    static CLSID_ShellFSFolder: GUID;
    static IID_IContextMenu: GUID;
    static IID_IDataObject: GUID;
    static IID_IDropTarget: GUID;
    static IID_IExtractIconA: GUID;
    static IID_IExtractIconW: GUID;
    static IID_IPersistFolder: GUID;
    static IID_IShellFolder: GUID;
}

extern "C" {
    fn memset(dst: *mut core::ffi::c_void, value: INT, bytes: usize) -> *mut core::ffi::c_void;
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

unsafe fn cpp_com_SHSetStrRetW(out: *mut STRRET, value: LPCWSTR) -> HRESULT {
    let len = lstrlenW(value) as usize;
    let bytes = (len + 1) * core::mem::size_of::<WCHAR>();
    let buffer = CoTaskMemAlloc(bytes) as LPWSTR;
    if buffer.is_null() { return E_OUTOFMEMORY; }
    let _ = memcpy(buffer, value, bytes);
    (*out).uType = STRRET_WSTR; (*out).u.pOleStr = buffer; S_OK
}
unsafe fn cpp_com_SHSetStrRetResource(out: *mut STRRET, id: DWORD) -> HRESULT {
    let mut buffer = [0u16; MAX_PATH];
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
pub static CPP_COM_WSTR_0EEFFF37A29C: [WCHAR; 13] = [0x005C, 0x0064, 0x0065, 0x0073, 0x006B, 0x0074, 0x006F, 0x0070, 0x002E, 0x0069, 0x006E, 0x0069, 0x0000];
pub static CPP_COM_WSTR_31CCAA880800: [WCHAR; 12] = [0x0053, 0x0068, 0x0065, 0x006C, 0x006C, 0x0046, 0x006F, 0x006C, 0x0064, 0x0065, 0x0072, 0x0000];
pub static CPP_COM_WSTR_77ECA80E282A: [WCHAR; 3] = [0x002E, 0x002E, 0x0000];
pub static CPP_COM_WSTR_932FF3CCC020: [WCHAR; 16] = [0x002E, 0x0053, 0x0068, 0x0065, 0x006C, 0x006C, 0x0043, 0x006C, 0x0061, 0x0073, 0x0073, 0x0049, 0x006E, 0x0066, 0x006F, 0x0000];
pub static CPP_COM_WSTR_955300DD21E1: [WCHAR; 4] = [0x002A, 0x002E, 0x002A, 0x0000];
pub static CPP_COM_WSTR_96A296D224F2: [WCHAR; 1] = [0x0000];
pub static CPP_COM_WSTR_AFF3436D7481: [WCHAR; 6] = [0x0043, 0x004C, 0x0053, 0x0049, 0x0044, 0x0000];
pub static CPP_COM_WSTR_B7FC02907B1F: [WCHAR; 2] = [0x005C, 0x0000];
pub static CPP_COM_WSTR_BA2342D760A2: [WCHAR; 6] = [0x0025, 0x0073, 0x005C, 0x0025, 0x0073, 0x0000];
pub static CPP_COM_WSTR_C14D9C3E5FFD: [WCHAR; 67] = [0x0053, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x004D, 0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057, 0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x005C, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069, 0x006F, 0x006E, 0x005C, 0x0053, 0x0068, 0x0065, 0x006C, 0x006C, 0x0020, 0x0045, 0x0078, 0x0074, 0x0065, 0x006E, 0x0073, 0x0069, 0x006F, 0x006E, 0x0073, 0x005C, 0x0042, 0x006C, 0x006F, 0x0063, 0x006B, 0x0065, 0x0064, 0x0000];
pub static CPP_COM_WSTR_C245AFF91259: [WCHAR; 2] = [0x002E, 0x0000];

use core::ffi::c_void;
use crate::shellcom::*;
use crate::types::*;
// This module is the same translation unit as `cfs_folder.rs`, so it takes its substrate from
// there rather than re-deriving one: the SFGAO_*/SHCOLSTATE_* constants, the `_IL*` PIDL
// accessors and the folder-column tables are that TU's closure, already proven by the class half.
use crate::cfs_folder::*;
// A COM interface the island renders in more than one place: name the owner so the two
// globs above cannot disagree. `cfs_folder`'s definitions are the ones this TU's own
// functions are typed against, so they are the ones to pick.
use crate::cfs_folder::{IDataObject, IShellFolder, IShellFolder2, IStream, IUnknown};
// wine/unicode.h `strcmpW` is `wcscmp`; the crate's public re-export is the established idiom
// for this (see wine/shellpath.rs's own private copy, and prelude.rs's doc comment on it).
use crate::prelude::strcmpW;

pub unsafe fn CFileSysEnum_dtor_body(this: *mut CFileSysEnum) -> () {}

pub unsafe fn CFileSysEnum_CEnumIDListBase_base_dtor_body(this: *mut CFileSysEnum) -> () {
    CFileSysEnum_DeleteList_body(this);
}

pub unsafe fn CFileSysEnum_AddToEnumList_body(this: *mut CFileSysEnum, pidl: LPITEMIDLIST) -> BOOL {
    let pNew: *mut ENUMLIST;

    if pidl.is_null() {
        return FALSE;
    }

    pNew = SHAlloc(core::mem::size_of::<ENUMLIST>()) as *mut ENUMLIST;
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

pub unsafe fn OpenKeyFromFileType_body(pExtension: LPCWSTR, KeyName: LPCWSTR) -> HKEY {
    let mut hkey: HKEY = core::ptr::null_mut();

    let mut FullName: [u16; MAX_PATH] = [0; MAX_PATH];
    let mut dwSize: u32 = core::mem::size_of_val(&FullName) as u32;
    wsprintfW(FullName.as_mut_ptr(), CPP_COM_WSTR_BA2342D760A2.as_ptr(), pExtension, KeyName);

    let mut res: i32 = RegOpenKeyExW(HKEY_CLASSES_ROOT, FullName.as_ptr(), 0, KEY_READ, &mut hkey);
    if res == 0 {
        return hkey;
    }

    res = RegGetValueW(HKEY_CLASSES_ROOT, pExtension, core::ptr::null_mut(), RRF_RT_REG_SZ, core::ptr::null_mut(), FullName.as_mut_ptr().cast(), &mut dwSize);
    if res != 0 {
        return core::ptr::null_mut();
    }

    wcscat(FullName.as_mut_ptr(), CPP_COM_WSTR_B7FC02907B1F.as_ptr());
    wcscat(FullName.as_mut_ptr(), KeyName);

    hkey = core::ptr::null_mut();
    res = RegOpenKeyExW(HKEY_CLASSES_ROOT, FullName.as_ptr(), 0, KEY_READ, &mut hkey);
    if res != 0 {
        WARN!("Could not open key %S for extension %S\n", KeyName, pExtension);
    }

    hkey
}

pub unsafe fn CFileSysEnum_DeleteList_body(this: *mut CFileSysEnum) -> BOOL {
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
    return TRUE;
}

pub unsafe fn CFileSysEnum_AppendItemsFromEnumerator_body(this: *mut CFileSysEnum, pEnum: *mut IEnumIDList) -> HRESULT {
    let mut pidl: LPITEMIDLIST = core::ptr::null_mut();
    let mut dwFetched: u32 = 0;

    if pEnum.is_null() {
        return E_INVALIDARG;
    }

    ((*(*pEnum).lpVtbl).Reset)(pEnum);

    while (S_OK == ((*(*pEnum).lpVtbl).Next)(pEnum, 1, &mut pidl, &mut dwFetched)) && dwFetched != 0 {
        CFileSysEnum_AddToEnumList_body(this, pidl);
    }

    S_OK
}

pub unsafe fn GetCLSIDForFileTypeFromExtension_body(pExtension: LPCWSTR, KeyName: LPCWSTR, pclsid: *mut CLSID) -> HRESULT {
    let hkeyProgId = OpenKeyFromFileType_body(pExtension, KeyName);
    if hkeyProgId.is_null()
    {
        WARN!("OpenKeyFromFileType failed for key %S\n", KeyName);
        return S_FALSE;
    }

    let mut wszCLSIDValue: [u16; CHARS_IN_GUID as usize] = [0; CHARS_IN_GUID as usize];
    let mut dwSize: u32 = core::mem::size_of_val(&wszCLSIDValue) as u32;
    let res = RegGetValueW(hkeyProgId, core::ptr::null(), core::ptr::null(), RRF_RT_REG_SZ, core::ptr::null_mut(), wszCLSIDValue.as_mut_ptr().cast(), &mut dwSize);
    RegCloseKey(hkeyProgId);
    if res != 0
    {
        ERR!("OpenKeyFromFileType succeeded but RegGetValueW failed\n");
        return S_FALSE;
    }

    if RegGetValueW(HKEY_LOCAL_MACHINE,
                     CPP_COM_WSTR_C14D9C3E5FFD.as_ptr(),
                     wszCLSIDValue.as_ptr(),
                     RRF_RT_REG_SZ,
                     core::ptr::null_mut(),
                     core::ptr::null_mut(),
                     core::ptr::null_mut()) == ERROR_SUCCESS
    {
        ERR!("Extension %S  not approved\n", wszCLSIDValue.as_ptr());
        return E_ACCESSDENIED;
    }

    let hres = CLSIDFromString(wszCLSIDValue.as_ptr(), pclsid);
    if ((hres) < 0) {
        return hres;
    }

    return S_OK;
}

pub unsafe fn CFileSysEnum_Next_body(this: *mut CFileSysEnum, celt: ULONG, rgelt: *mut PITEMID_CHILD, pceltFetched: *mut ULONG) -> HRESULT {
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

pub unsafe fn CFileSysEnum_Skip_body(this: *mut CFileSysEnum, celt: ULONG) -> HRESULT {
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

pub unsafe fn CFileSysEnum_Reset_body(this: *mut CFileSysEnum) -> HRESULT {
    (*this).mpCurrent = (*this).mpFirst;
    return S_OK;
}

pub unsafe fn CFileSysEnum_Clone_body(this: *mut CFileSysEnum, ppenum: *mut *mut IEnumIDList) -> HRESULT { E_NOTIMPL }

pub unsafe fn CFileSysEnum__AddFindResult_body(this: *mut CFileSysEnum, sParentDir: LPWSTR, FindData: &WIN32_FIND_DATAW, dwFlags: DWORD) -> HRESULT {
    const SUPER_HIDDEN: DWORD = FILE_ATTRIBUTE_HIDDEN | FILE_ATTRIBUTE_SYSTEM;

    if FindData.dwFileAttributes & FILE_ATTRIBUTE_HIDDEN != 0 {
        let dwHidden = FindData.dwFileAttributes & SUPER_HIDDEN;

        if dwHidden == FILE_ATTRIBUTE_HIDDEN && (dwFlags & SHCONTF_INCLUDEHIDDEN) == 0 {
            return S_OK;
        }

        if dwHidden == SUPER_HIDDEN && (dwFlags & SHCONTF_INCLUDESUPERHIDDEN) == 0 {
            return S_OK;
        }
    }

    let mut bDirectory = (FindData.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) != 0;

    let mut hr: HRESULT;
    if bDirectory {
        if strcmpW(FindData.cFileName.as_ptr(), CPP_COM_WSTR_C245AFF91259.as_ptr()) == 0
            || strcmpW(FindData.cFileName.as_ptr(), CPP_COM_WSTR_77ECA80E282A.as_ptr()) == 0
        {
            return S_OK;
        }

        if (FindData.dwFileAttributes & (FILE_ATTRIBUTE_SYSTEM | FILE_ATTRIBUTE_READONLY))
            != 0
        {
            let mut Tmp: [u16; MAX_PATH] = [0; MAX_PATH];
            let mut clsidFolder: CLSID = core::mem::zeroed();

            PathCombineW(Tmp.as_mut_ptr(), sParentDir, FindData.cFileName.as_ptr());

            hr = SHELL32_GetCLSIDForDirectory_body(
                Tmp.as_ptr(),
                CPP_COM_WSTR_AFF3436D7481.as_ptr(),
                &mut clsidFolder,
            );
            if hr >= 0 {
                // The C++ body of this branch is exactly this diagnostic (CFSFolder.cpp:452):
                // dropping it left an empty `if`, which is what a lost always-on print looks
                // like from the outside — a branch that tests a condition and does nothing.
                ERR!("Got CLSID override for '%S'\n", Tmp.as_ptr());
            }
        }
    } else {
        let mut clsidFile: CLSID = core::mem::zeroed();
        let pExtension = PathFindExtensionW(FindData.cFileName.as_ptr());
        if !pExtension.is_null() {
            hr = GetCLSIDForFileTypeFromExtension_body(
                pExtension,
                CPP_COM_WSTR_AFF3436D7481.as_ptr(),
                &mut clsidFile,
            );
            if hr == S_OK {
                let mut hkey: HKEY = core::ptr::null_mut();
                hr = SHRegGetCLSIDKeyW(
                    &clsidFile,
                    CPP_COM_WSTR_31CCAA880800.as_ptr(),
                    FALSE,
                    FALSE,
                    &mut hkey,
                );
                if hr >= 0 {
                    RegCloseKey(hkey);

                    bDirectory = true;
                }
            }
        }
    }

    let mut pidl: LPITEMIDLIST = core::ptr::null_mut();
    if bDirectory {
        if dwFlags & SHCONTF_FOLDERS != 0 {
            pidl = _ILCreateFromFindDataW(FindData);
        }
    } else {
        if dwFlags & SHCONTF_NONFOLDERS != 0 {
            pidl = _ILCreateFromFindDataW(FindData);
        }
    }

    if !pidl.is_null() && CFileSysEnum_AddToEnumList_body(this, pidl) == 0 {
        return E_FAIL;
    }

    return S_OK;
}

pub unsafe fn CFileSysEnum_ctor_body(this: *mut CFileSysEnum) -> () {}

pub unsafe fn CFileSysEnum_Initialize_body(this: *mut CFileSysEnum, sPathTarget: LPWSTR, dwFlags: DWORD) -> HRESULT {
    if sPathTarget.is_null() || *sPathTarget.offset(0) == 0 {
        return S_FALSE;
    }

    let mut szFindPattern: [u16; MAX_PATH] = core::mem::zeroed();
    let mut hr = StringCchCopyW(szFindPattern.as_mut_ptr(), szFindPattern.len(), sPathTarget);
    if ((hr) < 0) {
        return hr;
    }

    PathAddBackslashW(szFindPattern.as_mut_ptr());

    hr = StringCchCatW(szFindPattern.as_mut_ptr(), szFindPattern.len(), CPP_COM_WSTR_955300DD21E1.as_ptr());
    if ((hr) < 0) {
        return hr;
    }

    let mut FindData: WIN32_FIND_DATAW = core::mem::zeroed();
    let hFind = FindFirstFileW(szFindPattern.as_ptr(), &mut FindData);
    if hFind == INVALID_HANDLE_VALUE {
        return HRESULT_FROM_WIN32(GetLastError() as LONG);
    }

    loop {
        hr = CFileSysEnum__AddFindResult_body(this, sPathTarget, &FindData, dwFlags);

        if ((hr) < 0) {
            break;
        }

        if FindNextFileW(hFind, &mut FindData) == 0 {
            break;
        }
    }

    if hr >= 0 {
        let dwError = GetLastError();
        if dwError != ERROR_NO_MORE_FILES {
            hr = HRESULT_FROM_WIN32(dwError as LONG);
        }
    }

    FindClose(hFind);
    hr
}

pub unsafe fn SHELL32_GetCLSIDForDirectory_body(pwszDir: LPCWSTR, KeyName: LPCWSTR, pclsidFolder: *mut CLSID) -> HRESULT {
    let mut wszCLSIDValue: [u16; CHARS_IN_GUID as usize] = [0; CHARS_IN_GUID as usize];
    let mut wszDesktopIni: [u16; MAX_PATH] = [0; MAX_PATH];

    StringCchCopyW(wszDesktopIni.as_mut_ptr(), MAX_PATH, pwszDir);
    StringCchCatW(wszDesktopIni.as_mut_ptr(), MAX_PATH, CPP_COM_WSTR_0EEFFF37A29C.as_ptr());

    if GetPrivateProfileStringW(CPP_COM_WSTR_932FF3CCC020.as_ptr(),
                               KeyName,
                               CPP_COM_WSTR_96A296D224F2.as_ptr(),
                               wszCLSIDValue.as_mut_ptr(),
                               CHARS_IN_GUID,
                               wszDesktopIni.as_ptr()) != 0
    {
        return CLSIDFromString(wszCLSIDValue.as_ptr(), pclsidFolder);
    }
    return E_FAIL;
}
