// rustconv-assembly: class:CRecycleBinEnum
// Deterministic from shobjidl.h
// The assembler emits the hand-written types/win32/shellcom globs alongside
// `crate::prelude::*`. They are DROPPED, not joined: carrying both makes every shared
// name ambiguous (E0659). The island via `prelude` is the only substrate source.
use crate::prelude::*;
// The crate's own functions: the definition is the declaration.
use crate::def_view::SHCreateShellFolderView;
use crate::recycle_bin_tu::SHEmptyRecycleBinW;
use crate::desktop_folder_tu::SHGetDesktopFolder;
// One declaration per external symbol: island.rs.
use crate::island::CoTaskMemAlloc;
use crate::island::LoadStringW;
use crate::island::RegCloseKey;
use crate::island::RegQueryValueExW;
use crate::island::RegSetValueExW;
use crate::recycle_bin::{IDS_PROPERTIES};

// ── PIDL static-inline helpers (wine/pidl.h) ──
// Defined here rather than imported from `cfs_folder`: that module predates the island and its
// LPCITEMIDLIST is the hand-written `shellcom::ITEMIDLIST`, so importing across the substrate
// boundary makes every call site a nominal mismatch. Two lines each; a local copy is cheaper
// and cannot drift, because the C is a header inline with no symbol either way.
#[inline] unsafe fn _ILGetType(pidl: LPCITEMIDLIST) -> BYTE {
    if !pidl.is_null() && (*pidl).mkid.cb >= 3 { *(pidl as *const u8).add(2) } else { 0 }
}
#[inline] unsafe fn _ILGetFSType(pidl: LPCITEMIDLIST) -> BYTE {
    let t = _ILGetType(pidl);
    if (t as u32 & PT_FOLDERTYPEMASK) == PT_FS { t } else { 0 }
}
use crate::shellcom::STRRET_WSTR;

/// `recyclebin.h:50` `#define IRecycleBinFileFromHDELFILE(hDF) ((IRecycleBinFile*)(hDF))`.
/// Only the two slots this file uses are named; the rest are reserved so the indices stay
/// right. Transcribed from recyclebin.h's DECLARE_INTERFACE_(IRecycleBinFile, IUnknown).
#[repr(C)]
pub struct IRecycleBinFileVtbl {
    pub QueryInterface: unsafe extern "system" fn(*mut IRecycleBinFile, REFIID, *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(*mut IRecycleBinFile) -> ULONG,
    pub Release: unsafe extern "system" fn(*mut IRecycleBinFile) -> ULONG,
    pub IsEqualIdentity: unsafe extern "system" fn(*mut IRecycleBinFile, *const core::ffi::c_void) -> HRESULT,
    pub GetInfo: unsafe extern "system" fn(*mut IRecycleBinFile, *mut DELETED_FILE_INFO) -> HRESULT,
}
#[repr(C)]
pub struct IRecycleBinFile { pub lpVtbl: *const IRecycleBinFileVtbl }

#[inline]
unsafe fn IRecycleBinFileFromHDELFILE(hDF: HDELFILE) -> *mut IRecycleBinFile { hDF as *mut IRecycleBinFile }

/// `recyclebin.h:54` `static inline void FreeRecycleBinString(PRECYCLEBINSTRING)` — a header
/// inline, so no symbol exists to import.
#[inline]
unsafe fn FreeRecycleBinString(pRBS: *mut RECYCLEBINSTRING) {
    SHFree((*pRBS).Alloc as *mut core::ffi::c_void);
    (*pRBS).Alloc = core::ptr::null_mut();
    (*pRBS).String = core::ptr::null();
}
use crate::win32::{PathFindFileNameW, SHELL_ErrorBox};
use crate::shell32::shlfileop::SHELL_ConfirmYesNoW;
use crate::shell32::sdk_consts::{CMIC_MASK_FLAG_NO_UI};
use crate::recycle_bin_tu::{IDC_BB_RESTORE, IDC_BB_CUT, IDC_BB_DELETE, IDC_BB_PROPERTIES, BBITEMDATA, BBITEMFOOTER, BBITEMFILETYPE, BBITEMFOLDERTYPE, PIDLDATA_FILE_SZNAMES_OFFSET};
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

// Deterministic COM scaffold for CRecycleBinEnum; method bodies are generated separately.
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
pub struct CRecycleBinEnum {
    pub IEnumIDList_iface: IEnumIDList,
    pub ref_count: core::sync::atomic::AtomicI32,
    pub mpFirst: *mut ENUMLIST,
    pub mpLast: *mut ENUMLIST,
    pub mpCurrent: *mut ENUMLIST,
}

unsafe extern "system" fn CRecycleBinEnum_IEnumIDList_QueryInterface(This: *mut IEnumIDList, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBinEnum, IEnumIDList_iface)) as *mut CRecycleBinEnum;
    CRecycleBinEnum_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CRecycleBinEnum_IEnumIDList_AddRef(This: *mut IEnumIDList) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBinEnum, IEnumIDList_iface)) as *mut CRecycleBinEnum;
    CRecycleBinEnum_AddRef(this)
}
unsafe extern "system" fn CRecycleBinEnum_IEnumIDList_Release(This: *mut IEnumIDList) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBinEnum, IEnumIDList_iface)) as *mut CRecycleBinEnum;
    CRecycleBinEnum_Release(this)
}
unsafe extern "system" fn CRecycleBinEnum_IEnumIDList_Next(This: *mut IEnumIDList, celt: ULONG, rgelt: *mut PITEMID_CHILD, pceltFetched: *mut ULONG) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBinEnum, IEnumIDList_iface)) as *mut CRecycleBinEnum;
    CRecycleBinEnum_Next_body(this, celt, rgelt, pceltFetched)
}
unsafe extern "system" fn CRecycleBinEnum_IEnumIDList_Skip(This: *mut IEnumIDList, celt: ULONG) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBinEnum, IEnumIDList_iface)) as *mut CRecycleBinEnum;
    CRecycleBinEnum_Skip_body(this, celt)
}
unsafe extern "system" fn CRecycleBinEnum_IEnumIDList_Reset(This: *mut IEnumIDList) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBinEnum, IEnumIDList_iface)) as *mut CRecycleBinEnum;
    CRecycleBinEnum_Reset_body(this)
}
unsafe extern "system" fn CRecycleBinEnum_IEnumIDList_Clone(This: *mut IEnumIDList, ppenum: *mut *mut IEnumIDList) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBinEnum, IEnumIDList_iface)) as *mut CRecycleBinEnum;
    CRecycleBinEnum_Clone_body(this, ppenum)
}

pub static CRECYCLEBINENUM_IENUMIDLIST_VTBL: IEnumIDListVtbl = IEnumIDListVtbl {
    QueryInterface: CRecycleBinEnum_IEnumIDList_QueryInterface,
    AddRef: CRecycleBinEnum_IEnumIDList_AddRef,
    Release: CRecycleBinEnum_IEnumIDList_Release,
    Next: CRecycleBinEnum_IEnumIDList_Next,
    Skip: CRecycleBinEnum_IEnumIDList_Skip,
    Reset: CRecycleBinEnum_IEnumIDList_Reset,
    Clone: CRecycleBinEnum_IEnumIDList_Clone,
};

unsafe fn CRecycleBinEnum_QueryInterface(
    this: *mut CRecycleBinEnum, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IUnknown)) || cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IEnumIDList)) {
        *ppv = core::ptr::addr_of_mut!((*this).IEnumIDList_iface) as *mut core::ffi::c_void;
    }
    else { return E_NOINTERFACE; }
    CRecycleBinEnum_AddRef(this);
    S_OK
}

unsafe fn CRecycleBinEnum_AddRef(this: *mut CRecycleBinEnum) -> ULONG {
    (*this).ref_count.fetch_add(1, core::sync::atomic::Ordering::SeqCst) as ULONG + 1
}

unsafe fn CRecycleBinEnum_Release(this: *mut CRecycleBinEnum) -> ULONG {
    let refs = (*this).ref_count.fetch_sub(1, core::sync::atomic::Ordering::SeqCst) - 1;
    if refs == 0 {
        CRecycleBinEnum_dtor_body(this);
        CRecycleBinEnum_CEnumIDListBase_base_dtor_body(this);
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
pub unsafe fn CRecycleBinEnum_CreateInstance(
    dwFlags: DWORD,
    riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    let this = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<CRecycleBinEnum>()) as *mut CRecycleBinEnum;
    if this.is_null() { return E_OUTOFMEMORY; }
    core::ptr::write(this, CRecycleBinEnum {
        IEnumIDList_iface: IEnumIDList { lpVtbl: core::ptr::addr_of!(CRECYCLEBINENUM_IENUMIDLIST_VTBL) },
        ref_count: core::sync::atomic::AtomicI32::new(1),
        mpFirst: core::mem::zeroed(),
        mpLast: core::mem::zeroed(),
        mpCurrent: core::mem::zeroed(),
    });
    Shell32RustObjectLock();
    CRecycleBinEnum_ctor_body(this);
    let mut hr = CRecycleBinEnum_Initialize_body(this, dwFlags);
    if hr >= 0 {
        hr = CRecycleBinEnum_QueryInterface(this, riid, ppv);
    }
    CRecycleBinEnum_Release(this);
    hr
}

// Deterministic C++ support closure selected from source identifiers.

pub const COLUMNS_COUNT: UINT = 6;

pub const COLUMN_DATEDEL: UINT = 2;

pub const COLUMN_DELFROM: UINT = 1;

pub const COLUMN_MTIME: UINT = 5;

pub const COLUMN_NAME: UINT = 0;

pub const COLUMN_SIZE: UINT = 3;

pub const COLUMN_TYPE: UINT = 4;

pub const FCIDM_SHVIEW_CUT: UINT = 0x7018;

pub const FCIDM_SHVIEW_DELETE: UINT = 0x7011;

pub const FCIDM_SHVIEW_PROPERTIES: UINT = 0x7013;

pub const IDS_CUT: UINT = 324;

pub const IDS_DELETE: UINT = 317;

pub const IDS_DELETEITEM_TEXT: UINT = 132;

pub const IDS_DELETEMULTIPLE_TEXT: UINT = 133;

pub const IDS_EMPTY_BITBUCKET: UINT = 172;


pub const IDS_RESTORE: UINT = 325;

pub const IDS_SHV_COLUMN_DELDATE: UINT = 19;

pub const IDS_SHV_COLUMN_DELFROM: UINT = 18;

pub const IDS_SHV_COLUMN_MODIFIED: UINT = 10;

pub const IDS_SHV_COLUMN_NAME: UINT = 7;

pub const IDS_SHV_COLUMN_SIZE: UINT = 8;

pub const IDS_SHV_COLUMN_TYPE: UINT = 9;

pub const CSIDL_BITBUCKET: INT = 0x000a;

pub const E_UNEXPECTED: HRESULT = 0x8000_FFFFu32 as HRESULT;

pub const SFGAO_CANDELETE: SFGAOF = 0x0000_0020;

pub const SFGAO_CANLINK: SFGAOF = 0x0000_0004;

pub const SFGAO_CANRENAME: SFGAOF = 0x0000_0010;

pub const SFGAO_HASPROPSHEET: SFGAOF = 0x0000_0040;

pub const SHERB_NOCONFIRMATION: DWORD = 0x0000_0001;

pub const SHERB_NOPROGRESSUI: DWORD = 0x0000_0002;

pub const SHERB_NOSOUND: DWORD = 0x0000_0004;

pub const ANYSIZE_ARRAY: DWORD = 1;

pub const DE_OPCANCELLED: DWORD = 0x75;

pub const DRIVE_FIXED: DWORD = 3;

pub const ERROR_CANCELLED: DWORD = 1223;

pub const ERROR_FILE_NOT_FOUND: DWORD = 2;

pub const ERROR_NOT_SUPPORTED: DWORD = 50;

pub const FILE_ATTRIBUTE_DIRECTORY: DWORD = 0x00000010;

pub const FOF_MULTIDESTFILES: DWORD = 0x0001;

pub const FOF_NOCONFIRMATION: DWORD = 0x0010;

pub const FOF_NOCONFIRMMKDIR: DWORD = 0x0200;

pub const FOF_NOERRORUI: DWORD = 0x0400;

pub const FOF_SILENT: DWORD = 0x0004;

pub const FO_DELETE: DWORD = 3;

pub const FO_MOVE: DWORD = 1;

pub const IDNO: DWORD = 7;

pub const KEY_READ: DWORD = 0x20019;

pub const KEY_WRITE: DWORD = 0x20006;

pub const LMEM_MOVEABLE: DWORD = 0x0002;

pub const LVCFMT_LEFT: DWORD = 0x0;

pub const LVCFMT_RIGHT: DWORD = 0x1;

pub const MB_DEFBUTTON2: DWORD = 256;

pub const MB_ICONEXCLAMATION: DWORD = 0x30;

pub const MB_YESNO: DWORD = 4;

pub const MFS_DISABLED: DWORD = 3;

pub const MFS_ENABLED: DWORD = 0;

pub const MFT_SEPARATOR: DWORD = 0x800;

pub const PID_DISPLACED_DATE: DWORD = 3;

pub const PID_DISPLACED_FROM: DWORD = 2;

pub const REG_CREATED_NEW_KEY: DWORD = 1;

pub const REG_DWORD: DWORD = 4;

pub const REG_EXPAND_SZ: DWORD = 2;

pub const REG_SZ: DWORD = 1;

pub const SFGAO_DROPTARGET: DWORD = 0x00000100;

pub const SFGAO_LINK: DWORD = 0x00010000;

pub const SHCIDS_CANONICALONLY: DWORD = 0x10000000;

pub const SHCIDS_COLUMNMASK: DWORD = 0x0000FFFF;

pub const SHCNE_CREATE: DWORD = 0x00000002;

pub const SHCNE_DELETE: DWORD = 0x00000004;

pub const SHCNE_MKDIR: DWORD = 0x00000008;

pub const SHCNE_RMDIR: DWORD = 0x00000010;

pub const SHCNE_UPDATEDIR: DWORD = 0x00001000;

pub const SHCNE_UPDATEIMAGE: DWORD = 0x00008000;

pub const SHCNF_DWORD: DWORD = 0x0003;

pub const SHCNF_IDLIST: DWORD = 0x0000;

pub const SHCOLSTATE_ONBYDEFAULT: DWORD = 0x10;

pub const SHCOLSTATE_TYPE_DATE: DWORD = 3;

pub const SHCOLSTATE_TYPE_INT: DWORD = 2;

pub const SHCOLSTATE_TYPE_STR: DWORD = 1;

pub const SHCONTF_FOLDERS: DWORD = 32;

pub const SHCONTF_INCLUDEHIDDEN: DWORD = 128;

pub const SHCONTF_NONFOLDERS: DWORD = 64;

pub const SHGDN_INFOLDER: DWORD = 1;

pub const SHGFI_TYPENAME: DWORD = 1024;

pub const SHGFI_USEFILEATTRIBUTES: DWORD = 16;

pub const SND_ASYNC: DWORD = 1;

pub const SND_FILENAME: DWORD = 0x20000;

pub const SND_NODEFAULT: DWORD = 2;

#[repr(C)]
pub struct ULARGE_INTEGER {
    pub QuadPart: ULONGLONG,
}

pub use crate::island::WIN32_FIND_DATAW;

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

macro_rules! LOWORD { ($value:expr) => { ((($value as usize) & 0xffff) as UINT) as u16 }; }

pub type EXPPS = UINT;

pub type LPFNSVADDPROPSHEETPAGE = Option<unsafe extern "system" fn(HPROPSHEETPAGE, LPARAM) -> BOOL>;

pub use crate::island::SHQUERYRBINFO;

// CMVERBMAP comes from the island (utils.h:87-90). The assembler re-declared it;
// a second nominal copy of a layout is a bug waiting to diverge.
pub static mut g_BBItemVerbMap: [CMVERBMAP; 5] = [
    CMVERBMAP { Verb: b"undelete\0".as_ptr() as LPCSTR, CmdId: IDC_BB_RESTORE as WORD },
    CMVERBMAP { Verb: b"cut\0".as_ptr() as LPCSTR, CmdId: IDC_BB_CUT as WORD },
    CMVERBMAP { Verb: b"delete\0".as_ptr() as LPCSTR, CmdId: IDC_BB_DELETE as WORD },
    CMVERBMAP { Verb: b"properties\0".as_ptr() as LPCSTR, CmdId: IDC_BB_PROPERTIES as WORD },
    CMVERBMAP { Verb: core::ptr::null(), CmdId: 0 as WORD },
];

extern "system" {
    fn PlaySoundW(sound: LPCWSTR, module: HANDLE, flags: DWORD) -> BOOL;
    fn ExpandEnvironmentStringsW(source: LPCWSTR, dest: LPWSTR, size: DWORD) -> DWORD;
    fn SHChangeNotify(event: LONG, flags: UINT, item1: LPCVOID, item2: LPCVOID);
    fn GetLastError() -> DWORD;
    fn lstrlenW(text: LPCWSTR) -> INT;
    fn CoTaskMemFree(memory: LPVOID);
    fn SHStrDupW(source: LPCWSTR, out: *mut LPWSTR) -> HRESULT;
    fn SHQueryRecycleBinW(root: LPCWSTR, info: *mut SHQUERYRBINFO) -> HRESULT;
    #[link_name = "Shell32RustMapContextMenuVerbToCmdId"] fn SHELL_MapContextMenuVerbToCmdId(info: LPCMINVOKECOMMANDINFO, map: *const CMVERBMAP) -> INT;
    #[link_name = "Shell32RustGetCommandStringImpl"] fn SHELL_GetCommandStringImpl(command: SIZE_T, flags: UINT, buffer: LPSTR, capacity: UINT, map: *const CMVERBMAP) -> HRESULT;
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
    static CLSID_ShellFSFolder: GUID;
    static IID_IContextMenu: GUID;
    static IID_IDropTarget: GUID;
    static IID_IExtractIconA: GUID;
    static IID_IExtractIconW: GUID;
    static IID_IShellFolder: GUID;
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

unsafe fn cpp_com_SHSetStrRetW(out: *mut STRRET, value: LPCWSTR) -> HRESULT {
    let len = lstrlenW(value) as usize;
    let bytes = (len + 1) * core::mem::size_of::<WCHAR>();
    let buffer = CoTaskMemAlloc(bytes as SIZE_T) as LPWSTR;
    if buffer.is_null() { return E_OUTOFMEMORY; }
    let _ = memcpy(buffer, value, bytes);
    (*out).uType = STRRET_WSTR; *(*out).__bindgen_anon_1.pOleStr.as_mut() = buffer; S_OK
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
pub static CPP_COM_WSTR_9855CDA098DC: [WCHAR; 2] = [0x0058, 0x0000];
pub static CPP_COM_WSTR_F51AA22D711D: [WCHAR; 5] = [0x0043, 0x003A, 0x005C, 0x0058, 0x0000];

use core::ffi::c_void;

pub unsafe fn CRecycleBinEnum_dtor_body(this: *mut CRecycleBinEnum) -> () {}

pub unsafe fn CRecycleBinEnum_CEnumIDListBase_base_dtor_body(this: *mut CRecycleBinEnum) -> () {
    CRecycleBinEnum_DeleteList_body(this);
}

pub unsafe fn CRecycleBinEnum_AddToEnumList_body(this: *mut CRecycleBinEnum, pidl: LPITEMIDLIST) -> BOOL {
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

pub unsafe fn CRecycleBinEnum_DeleteList_body(this: *mut CRecycleBinEnum) -> BOOL {
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

pub unsafe fn ValidateItem_body(pidl: LPCITEMIDLIST) -> *mut BBITEMDATA {
    let minstringsize = core::mem::size_of_val(&CPP_COM_WSTR_9855CDA098DC.as_ptr()) + core::mem::size_of_val(&"");
    let minfs = core::mem::size_of::<u16>() + crate::recycle_bin_tu::PIDLDATA_FILE_SZNAMES_OFFSET() + minstringsize;
    let mindatasize = FIELD_OFFSET!(BBITEMDATA, OriginalLocation) + ((core::mem::size_of_val(&CPP_COM_WSTR_F51AA22D711D.as_ptr()) * 2)) as isize;
    let minsize = minfs + mindatasize as usize + core::mem::size_of::<BBITEMFOOTER>();
    let type_ = _ILGetType(pidl);
    if (type_ == BBITEMFILETYPE as u8 || type_ == BBITEMFOLDERTYPE as u8) && (*pidl).mkid.cb >= minsize as u16 {
        let pEnd = (pidl as *const u8).add((*pidl).mkid.cb as usize - core::mem::size_of::<BBITEMFOOTER>()) as *mut BBITEMFOOTER;
        if (*pEnd).EndSignature == BBITEMFOOTER::ENDSIG && (*pEnd).DataSize >= mindatasize as u16 {
            return (pEnd as *mut u8).sub((*pEnd).DataSize as usize) as *mut BBITEMDATA;
        }
    }
    core::ptr::null_mut()
}

pub unsafe fn CRecycleBinEnum_AppendItemsFromEnumerator_body(this: *mut CRecycleBinEnum, pEnum: *mut IEnumIDList) -> HRESULT {
    let mut pidl: LPITEMIDLIST = core::ptr::null_mut();
    let mut dwFetched: u32 = 0;

    if pEnum.is_null() {
        return E_INVALIDARG;
    }

    ((*(*pEnum).lpVtbl).Reset)(pEnum);

    while S_OK == ((*(*pEnum).lpVtbl).Next)(pEnum, 1, &mut pidl, &mut dwFetched) && dwFetched != 0 {
        CRecycleBinEnum_AddToEnumList_body(this, pidl);
    }

    S_OK
}

pub unsafe fn CreateItem_body(pszTrash: LPCWSTR, pszOrig: LPCWSTR, Details: &DELETED_FILE_INFO) -> LPITEMIDLIST {
    let folder = (Details.Attributes & FILE_ATTRIBUTE_DIRECTORY) != 0;
    let pszName = PathFindFileNameW(pszTrash);
    let ofsName = pszName.offset_from(pszTrash) as usize;
    let cchName = wcslen(pszName) as usize + 1;
    let cbName = cchName * core::mem::size_of::<u16>();
    let cbFSNames = cbName + core::mem::size_of::<[u8; 1]>() + 1;
    let cbFS = core::mem::size_of::<u16>() + crate::recycle_bin_tu::PIDLDATA_FILE_SZNAMES_OFFSET() + cbFSNames;
    let cchTrash = ofsName + cchName;
    let cbTrash = cchTrash * core::mem::size_of::<u16>();
    let cchOrig = wcslen(pszOrig) as usize + 1;
    let cbOrig = cchOrig * core::mem::size_of::<u16>();
    let cbData = FIELD_OFFSET!(BBITEMDATA, OriginalLocation) + cbOrig as isize + cbTrash as isize;
    let cb = cbFS + cbData as usize + core::mem::size_of::<BBITEMFOOTER>();
    if cb > 0xffff {
        return core::ptr::null_mut();
    }
    let pidl = SHAlloc((cb + core::mem::size_of::<WORD>()) as SIZE_T) as LPITEMIDLIST;
    if pidl.is_null() {
        return pidl;
    }

    (*pidl).mkid.cb = cb as u16;
    (*pidl).mkid.abID[0] = if folder { BBITEMFOLDERTYPE as u8 } else { BBITEMFILETYPE as u8 };
    (*ILGetNext(pidl)).mkid.cb = 0;
    // bindgen wraps each union arm; select the arm once, here.
    let fsitem = (*((*pidl).mkid.abID.as_mut_ptr() as *mut PIDLDATA)).u.file.as_mut() as *mut FileStruct;
    (*fsitem).dummy = 0;
    const _: () = assert!(core::mem::size_of::<RECYCLEBINFILESIZETYPE>() <= core::mem::size_of::<DWORD>());
    (*fsitem).dwFileSize = Details.FileSize;
    (*fsitem).uFileAttribs = LOWORD!(Details.Attributes);
    FileTimeToDosDateTime(core::ptr::addr_of!(Details.LastModification),
        core::ptr::addr_of_mut!((*fsitem).uFileDate), core::ptr::addr_of_mut!((*fsitem).uFileTime));
    CopyMemory((*fsitem).szNames.as_mut_ptr() as *mut core::ffi::c_void, pszName as *const core::ffi::c_void, cbName);
    let pszShort = (*fsitem).szNames.as_mut_ptr().add(cbName) as LPSTR;
    *pszShort.add(0) = b'\0' as i8;
    *pszShort.add(1) = b'\0' as i8;

    let footer = (pidl as *mut u8).add(cb - core::mem::size_of::<BBITEMFOOTER>()) as *mut BBITEMFOOTER;
    (*footer).DataSize = cbData as u16;
    (*footer).EndSignature = BBITEMFOOTER::ENDSIG;

    let data = (footer as *mut u8).offset(-((*footer).DataSize as isize)) as *mut BBITEMDATA;
    (*data).DeletionTime = Details.DeletionTime;
    (*data).RecycledPathOffset = ((FIELD_OFFSET!(BBITEMDATA, OriginalLocation) + cbOrig as isize) as u16 as isize) as u16;
    CopyMemory((*data).OriginalLocation.as_mut_ptr() as *mut core::ffi::c_void, pszOrig as *const core::ffi::c_void, cbOrig);
    CopyMemory((data as *mut u8).add(((*data).RecycledPathOffset) as usize) as *mut core::ffi::c_void, pszTrash as *const core::ffi::c_void, cbTrash);

    assert!(((&(*fsitem).szNames as *const _ as usize) & 1) == 0);
    const _: () = assert!(core::mem::offset_of!(BBITEMDATA, OriginalLocation) % 2 == 0);
    assert!(((data as usize) & 1) == 0);
    assert!(_ILGetFSType(pidl) != 0);
    assert!(_ILIsPidlSimple(pidl) != 0);
    assert!((*(((pidl as *mut u8).add((*pidl).mkid.cb as usize - core::mem::size_of::<WORD>()) as *const u16)) & 1) != 0);
    assert!(_ILGetFileStructW(pidl).is_null());
    assert!(ValidateItem_body(pidl) == data);
    pidl
}

pub unsafe fn CRecycleBinEnum_Next_body(this: *mut CRecycleBinEnum, celt: ULONG, rgelt: *mut PITEMID_CHILD, pceltFetched: *mut ULONG) -> HRESULT {
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

pub unsafe fn CRecycleBinEnum_Skip_body(this: *mut CRecycleBinEnum, celt: ULONG) -> HRESULT {
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

pub unsafe fn CRecycleBinEnum_Reset_body(this: *mut CRecycleBinEnum) -> HRESULT {
    (*this).mpCurrent = (*this).mpFirst;
    return S_OK;
}

pub unsafe fn CRecycleBinEnum_Clone_body(this: *mut CRecycleBinEnum, ppenum: *mut *mut IEnumIDList) -> HRESULT { E_NOTIMPL }

pub unsafe fn GetGlobalRecycleBinPath_body() -> LPCWSTR {
    core::ptr::null()
}

/// C: `static BOOL __stdcall CBEnumRecycleBin(PVOID Context, HDELFILE hDeletedFile)` — the
/// plain-function trampoline `EnumerateRecycleBinW` is handed, which downcasts `Context` and
/// re-enters the method. A C++ static member function, so it has no `this` of its own.
unsafe extern "system" fn CBEnumRecycleBin(Context: PVOID, hDeletedFile: HDELFILE) -> BOOL {
    CRecycleBinEnum_CBEnumRecycleBin_2_body(Context as *mut CRecycleBinEnum, hDeletedFile)
}

pub unsafe fn CRecycleBinEnum_CBEnumRecycleBin_body(this: *mut CRecycleBinEnum, Context: PVOID, hDeletedFile: HDELFILE) -> BOOL {
    CRecycleBinEnum_CBEnumRecycleBin_2_body(Context as *mut CRecycleBinEnum, hDeletedFile)
}

pub unsafe fn CRecycleBinEnum_ctor_body(this: *mut CRecycleBinEnum) -> () {}

pub unsafe fn CRecycleBinEnum_Initialize_body(this: *mut CRecycleBinEnum, dwFlags: DWORD) -> HRESULT {
    let szDrive: LPCWSTR = GetGlobalRecycleBinPath_body();
    if dwFlags & SHCONTF_NONFOLDERS != 0 {
        if EnumerateRecycleBinW(szDrive, Some(CBEnumRecycleBin), this as LPVOID) == 0 {
            return E_FAIL;
        }
    } else {
    }
    return S_OK;
}

pub unsafe fn CRecycleBinEnum_CBEnumRecycleBin_2_body(this: *mut CRecycleBinEnum, hDeletedFile: HDELFILE) -> BOOL {
    let mut pidl: LPITEMIDLIST = core::ptr::null_mut();
    let mut info: DELETED_FILE_INFO = core::mem::zeroed();
    let pRBF = IRecycleBinFileFromHDELFILE(hDeletedFile);
    let mut ret: BOOL = (((*(*pRBF).lpVtbl).GetInfo)(pRBF, &mut info) >= 0) as BOOL;
    if ret != 0 {
        pidl = CreateItem_body(info.RecycledFullPath.String, info.OriginalFullPath.String, &info);
        ret = (pidl != core::ptr::null_mut()) as BOOL;
        FreeRecycleBinString(&mut info.OriginalFullPath);
        FreeRecycleBinString(&mut info.RecycledFullPath);
    }
    if pidl != core::ptr::null_mut() {
        ret = CRecycleBinEnum_AddToEnumList_body(this, pidl);
        if ret == 0 {
            ILFree(pidl);
        }
    }
    CloseRecycleBinHandle(hDeletedFile);
    ret
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * Trash virtual folder support. The trashing engine is implemented in trash.c
//  *
//  * Copyright (C) 2006 Mikolaj Zalewski
//  * Copyright (C) 2009 Andrew Hill
//  * Copyright (C) 2018 Russell Johnson
//  *
//  * This library is free software; you can redistribute it and/or
//  * modify it under the terms of the GNU Lesser General Public
//  * License as published by the Free Software Foundation; either
//  * version 2.1 of the License, or (at your option) any later version.
//  *
//  * This library is distributed in the hope that it will be useful,
//  * but WITHOUT ANY WARRANTY; without even the implied warranty of
//  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
//  * Lesser General Public License for more details.
//  *
//  * You should have received a copy of the GNU Lesser General Public
//  * License along with this library; if not, write to the Free Software
//  * Foundation, Inc., 51 Franklin St, Fifth Floor, Boston, MA 02110-1301, USA
//  */
//
// #include <precomp.h>
//
// #include <mmsystem.h>
// #include <ntquery.h>
//
// WINE_DEFAULT_DEBUG_CHANNEL(CRecycleBin);
//
// typedef struct
// {
//     int column_name_id;
//     const GUID *fmtId;
//     DWORD pid;
//     int pcsFlags;
// static const columninfo RecycleBinColumns[] =
// {
//     {IDS_SHV_COLUMN_NAME,     &FMTID_Storage,   PID_STG_NAME,        SHCOLSTATE_TYPE_STR | SHCOLSTATE_ONBYDEFAULT,  LVCFMT_LEFT,  25},
//     {IDS_SHV_COLUMN_DELFROM,  &FMTID_Displaced, PID_DISPLACED_FROM,  SHCOLSTATE_TYPE_STR | SHCOLSTATE_ONBYDEFAULT,  LVCFMT_LEFT,  35},
//     {IDS_SHV_COLUMN_DELDATE,  &FMTID_Displaced, PID_DISPLACED_DATE,  SHCOLSTATE_TYPE_DATE | SHCOLSTATE_ONBYDEFAULT, LVCFMT_LEFT,  15},
//     {IDS_SHV_COLUMN_SIZE,     &FMTID_Storage,   PID_STG_SIZE,        SHCOLSTATE_TYPE_INT | SHCOLSTATE_ONBYDEFAULT,  LVCFMT_RIGHT, 10},
//     {IDS_SHV_COLUMN_TYPE,     &FMTID_Storage,   PID_STG_STORAGETYPE, SHCOLSTATE_TYPE_INT | SHCOLSTATE_ONBYDEFAULT,  LVCFMT_LEFT,  15},
//     {IDS_SHV_COLUMN_MODIFIED, &FMTID_Storage,   PID_STG_WRITETIME,   SHCOLSTATE_TYPE_DATE | SHCOLSTATE_ONBYDEFAULT, LVCFMT_LEFT,  15},
//     /* {"creation time",  &FMTID_Storage,   PID_STG_CREATETIME, SHCOLSTATE_TYPE_DATE, LVCFMT_LEFT,  20}, */
//     /* {"attribs",        &FMTID_Storage,   PID_STG_ATTRIBUTES, SHCOLSTATE_TYPE_STR,  LVCFMT_LEFT,  20}, */
// };
//
// #define COLUMN_NAME    0
// #define COLUMN_DELFROM 1
// #define COLUMN_DATEDEL 2
// #define COLUMN_SIZE    3
// #define COLUMN_TYPE    4
// #define COLUMN_MTIME   5
//
// #define COLUMNS_COUNT  6
//
// // The ROS Recycle Bin PIDL format starts with a NT4/2000 Unicode FS PIDL followed by
// // BBITEMDATA and BBITEMFOOTER. This makes it compatible with SHChangeNotify listeners.
// #include "pshpack1.h"
// #define BBITEMFILETYPE (PT_FS | PT_FS_UNICODE_FLAG | PT_FS_FILE_FLAG)
// #define BBITEMFOLDERTYPE (PT_FS | PT_FS_UNICODE_FLAG | PT_FS_FOLDER_FLAG)
// struct BBITEMDATA
// {
//     FILETIME DeletionTime;
//     WORD RecycledPathOffset;
//     WCHAR OriginalLocation[ANYSIZE_ARRAY];
//     // ... @RecycledPathOffset WCHAR RecycledPath[ANYSIZE_ARRAY];
// };
// struct BBITEMFOOTER
// {
//     enum { ENDSIG = MAKEWORD('K', 'I') }; // "Killed item". MUST have the low bit set so _ILGetFileStructW returns NULL.
//
// static inline BOOL IsFolder(LPCITEMIDLIST pidl)
// {
//     return _ILGetFSType(pidl) & PT_FS_FOLDER_FLAG;
// }
//
// static BBITEMDATA* ValidateItem(LPCITEMIDLIST pidl)
// {
//     const UINT minstringsize = sizeof(L"X") + sizeof(""); // PT_FS strings
//     const UINT minfs = sizeof(WORD) + crate::recycle_bin_tu::PIDLDATA_FILE_SZNAMES_OFFSET() + minstringsize;
//     const UINT mindatasize = FIELD_OFFSET!(BBITEMDATA, OriginalLocation) + (sizeof(L"C:\\X") * 2);
//     const UINT minsize = minfs + mindatasize + sizeof(BBITEMFOOTER);
//     const BYTE type = _ILGetType(pidl);
//     if ((type == BBITEMFILETYPE || type == BBITEMFOLDERTYPE) && pidl->mkid.cb >= minsize)
//     {
//         BBITEMFOOTER *pEnd = (BBITEMFOOTER*)((BYTE*)pidl + pidl->mkid.cb - sizeof(BBITEMFOOTER));
//         if (pEnd->EndSignature == BBITEMFOOTER::ENDSIG && pEnd->DataSize >= mindatasize)
//             return (BBITEMDATA*)((BYTE*)pEnd - pEnd->DataSize);
//     }
//     return NULL;
// }
//
// static LPITEMIDLIST CreateItem(LPCWSTR pszTrash, LPCWSTR pszOrig, const DELETED_FILE_INFO &Details)
// {
//     const BOOL folder = Details.Attributes & FILE_ATTRIBUTE_DIRECTORY;
//     LPCWSTR pszName = PathFindFileNameW(pszTrash);
//     SIZE_T ofsName = (SIZE_T)(pszName - pszTrash);
//     SIZE_T cchName = wcslen(pszName) + 1, cbName = cchName * sizeof(WCHAR);
//     SIZE_T cbFSNames = cbName + sizeof("") + 1; // Empty short name + 1 for WORD alignment
//     SIZE_T cbFS = sizeof(WORD) + crate::recycle_bin_tu::PIDLDATA_FILE_SZNAMES_OFFSET() + cbFSNames;
//     SIZE_T cchTrash = ofsName + cchName, cbTrash = cchTrash * sizeof(WCHAR);
//     SIZE_T cchOrig = wcslen(pszOrig) + 1, cbOrig = cchOrig * sizeof(WCHAR);
//     SIZE_T cbData = FIELD_OFFSET!(BBITEMDATA, OriginalLocation) + cbOrig + cbTrash;
//     SIZE_T cb = cbFS + cbData + sizeof(BBITEMFOOTER);
//     if (cb > 0xffff)
//         return NULL;
//     LPITEMIDLIST pidl = (LPITEMIDLIST)SHAlloc(cb + sizeof(WORD));
//     if (!pidl)
//         return pidl;
//
//     pidl->mkid.cb = cb;
//     pidl->mkid.abID[0] = folder ? BBITEMFOLDERTYPE : BBITEMFILETYPE;
//     ILGetNext(pidl)->mkid.cb = 0; // Terminator
//     FileStruct &fsitem = ((PIDLDATA*)pidl->mkid.abID)->u.file;
//     fsitem.dummy = 0;
//     C_ASSERT(sizeof(RECYCLEBINFILESIZETYPE) <= sizeof(fsitem.dwFileSize));
//     fsitem.dwFileSize = Details.FileSize;
//     fsitem.uFileAttribs = LOWORD(Details.Attributes);
//     FileTimeToDosDateTime(&Details.LastModification, &fsitem.uFileDate, &fsitem.uFileTime);
//     CopyMemory(fsitem.szNames, pszName, cbName);
//     LPSTR pszShort = const_cast<LPSTR>(&fsitem.szNames[cbName]);
//     pszShort[0] = '\0';
//     pszShort[1] = '\0'; // Fill alignment padding (for ILIsEqual memcmp)
//
//     BBITEMFOOTER *footer = (BBITEMFOOTER*)((BYTE*)pidl + cb - sizeof(BBITEMFOOTER));
//     footer->DataSize = cbData;
//     footer->EndSignature = BBITEMFOOTER::ENDSIG;
//
//     BBITEMDATA *data = (BBITEMDATA*)((BYTE*)footer - footer->DataSize);
//     data->DeletionTime = Details.DeletionTime;
//     data->RecycledPathOffset = FIELD_OFFSET!(BBITEMDATA, OriginalLocation) + cbOrig;
//     CopyMemory(data->OriginalLocation, pszOrig, cbOrig);
//     CopyMemory((BYTE*)data + data->RecycledPathOffset, pszTrash, cbTrash);
//
//     assert(!(((SIZE_T)&fsitem.szNames) & 1)); // WORD aligned please
//     C_ASSERT(!(FIELD_OFFSET!(BBITEMDATA, OriginalLocation) & 1)); // WORD aligned please
//     assert(!(((SIZE_T)data) & 1)); // WORD aligned please
//     assert(_ILGetFSType(pidl));
//     assert(_ILIsPidlSimple(pidl));
//     assert(*(WORD*)((BYTE*)pidl + pidl->mkid.cb - sizeof(WORD)) & 1); // ENDSIG bit
//     assert(_ILGetFileStructW(pidl) == NULL); // Our custom footer is incompatible with WinXP pidl data
//     assert(ValidateItem(pidl) == data);
//     return pidl;
// }
//
// static inline UINT GetItemFileSize(LPCITEMIDLIST pidl)
// {
//     return _ILGetFSType(pidl) ? ((PIDLDATA*)pidl->mkid.abID)->u.file.dwFileSize : 0;
// }
//
// static inline LPCWSTR GetItemOriginalFullPath(const BBITEMDATA &Data)
// {
//     return Data.OriginalLocation;
// }
//
// static HRESULT GetItemOriginalFolder(const BBITEMDATA &Data, LPWSTR &Out)
//     return hr;
// }
//
// static LPCWSTR GetItemOriginalFileName(const BBITEMDATA &Data)
// {
//     return PathFindFileNameW(GetItemOriginalFullPath(Data));
// }
//
// static inline LPCWSTR GetItemRecycledFullPath(const BBITEMDATA &Data)
// {
//     return (LPCWSTR)((BYTE*)&Data + Data.RecycledPathOffset);
// }
//
//
// static int GetItemDriveNumber(LPCITEMIDLIST pidl)
// {
//     if (BBITEMDATA *pData = ValidateItem(pidl))
//
// static HRESULT GetItemTypeName(PCUITEMID_CHILD pidl, const BBITEMDATA &Data, SHFILEINFOW &shfi)
// {
//     LPCWSTR path = GetItemRecycledFullPath(Data);
//     UINT attribs = ((PIDLDATA*)pidl->mkid.abID)->u.file.uFileAttribs;
//     if (SHGetFileInfoW(path, attribs, &shfi, sizeof(shfi), SHGFI_TYPENAME | SHGFI_USEFILEATTRIBUTES))
//
// /*
//  * Recycle Bin folder
//  */
//
// static inline LPCWSTR GetGlobalRecycleBinPath()
// {
//     return NULL;
// }
//         static BOOL __stdcall CBEnumRecycleBin( PVOID Context,  HDELFILE hDeletedFile)
//         {
//             return static_cast<CRecycleBinEnum*>(Context)->CBEnumRecycleBin(hDeletedFile);
//         }
// HRESULT WINAPI CRecycleBinEnum::Initialize(DWORD dwFlags)
// {
//     LPCWSTR szDrive = GetGlobalRecycleBinPath();
//     if (dwFlags & SHCONTF_NONFOLDERS)
//     {
//         TRACE("Starting Enumeration\n");
//
//         if (!EnumerateRecycleBinW(szDrive, CBEnumRecycleBin, this))
//         {
//             WARN("Error: EnumerateCRecycleBinW failed\n");
//             return E_FAIL;
//         }
//     }
//     else
//     {
//         // do nothing
//     }
//     return S_OK;
// }
// BOOL CRecycleBinEnum::CBEnumRecycleBin( HDELFILE hDeletedFile)
// {
//     LPITEMIDLIST pidl = NULL;
//     DELETED_FILE_INFO info;
//     IRecycleBinFile *pRBF = IRecycleBinFileFromHDELFILE(hDeletedFile);
//     BOOL ret = SUCCEEDED(pRBF->GetInfo(&info));
//     if (ret)
//     {
//         pidl = CreateItem(info.RecycledFullPath.String, info.OriginalFullPath.String, info);
//         ret = pidl != NULL;
//         FreeRecycleBinString(&info.OriginalFullPath);
//         FreeRecycleBinString(&info.RecycledFullPath);
//     }
//     if (pidl)
//     {
//         ret = AddToEnumList(pidl);
//         if (!ret)
//             ILFree(pidl);
//     }
//     CloseRecycleBinHandle(hDeletedFile);
//     return ret;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
