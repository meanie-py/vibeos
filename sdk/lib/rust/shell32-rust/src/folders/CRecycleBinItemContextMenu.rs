// rustconv-assembly: class:CRecycleBinItemContextMenu
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
#[inline]
unsafe fn IRecycleBinFileFromHDELFILE(hDF: HDELFILE) -> *mut core::ffi::c_void { hDF as *mut core::ffi::c_void }
use crate::wine::pidl::{_ILCopyaPidl, _ILFreeaPidl};
use crate::shell32::shlfileop::SHELL32_FileOperation;
use crate::shell32::sdk_consts::{FOCE_POSTDELETEITEM, FOCE_POSTMOVEITEM, FOCE_FINISHOPERATIONS};
use crate::recycle_bin_tu::{FILEOPDATA, IsRecycleBinEmpty, CRecycleBin_NotifyRemovedFromRecycleBin};
use crate::win32::{PathFindFileNameW, SHELL_ErrorBox};
use crate::shell32::shlfileop::SHELL_ConfirmYesNoW;
use crate::shell32::sdk_consts::{CMIC_MASK_FLAG_NO_UI};
use crate::recycle_bin_tu::{IDC_BB_RESTORE, IDC_BB_CUT, IDC_BB_DELETE, IDC_BB_PROPERTIES, BBITEMDATA, BBITEMFOOTER, BBITEMFILETYPE, BBITEMFOLDERTYPE, PIDLDATA_FILE_SZNAMES_OFFSET};
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

// Deterministic COM scaffold for CRecycleBinItemContextMenu; method bodies are generated separately.
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
pub struct CRecycleBinItemContextMenu {
    pub IContextMenu2_iface: IContextMenu2,
    pub ref_count: core::sync::atomic::AtomicI32,
    pub m_apidl: *mut PITEMID_CHILD,
    pub m_cidl: UINT,
}

unsafe extern "system" fn CRecycleBinItemContextMenu_IContextMenu2_QueryInterface(This: *mut IContextMenu2, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBinItemContextMenu, IContextMenu2_iface)) as *mut CRecycleBinItemContextMenu;
    CRecycleBinItemContextMenu_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CRecycleBinItemContextMenu_IContextMenu2_AddRef(This: *mut IContextMenu2) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBinItemContextMenu, IContextMenu2_iface)) as *mut CRecycleBinItemContextMenu;
    CRecycleBinItemContextMenu_AddRef(this)
}
unsafe extern "system" fn CRecycleBinItemContextMenu_IContextMenu2_Release(This: *mut IContextMenu2) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBinItemContextMenu, IContextMenu2_iface)) as *mut CRecycleBinItemContextMenu;
    CRecycleBinItemContextMenu_Release(this)
}
unsafe extern "system" fn CRecycleBinItemContextMenu_IContextMenu2_QueryContextMenu(This: *mut IContextMenu2, hmenu: HMENU, indexMenu: UINT, idCmdFirst: UINT, idCmdLast: UINT, uFlags: UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBinItemContextMenu, IContextMenu2_iface)) as *mut CRecycleBinItemContextMenu;
    CRecycleBinItemContextMenu_QueryContextMenu_body(this, hmenu, indexMenu, idCmdFirst, idCmdLast, uFlags)
}
unsafe extern "system" fn CRecycleBinItemContextMenu_IContextMenu2_InvokeCommand(This: *mut IContextMenu2, lpici: LPCMINVOKECOMMANDINFO) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBinItemContextMenu, IContextMenu2_iface)) as *mut CRecycleBinItemContextMenu;
    CRecycleBinItemContextMenu_InvokeCommand_body(this, lpici)
}
unsafe extern "system" fn CRecycleBinItemContextMenu_IContextMenu2_GetCommandString(This: *mut IContextMenu2, idCmd: UINT_PTR, uType: UINT, pwReserved: *mut UINT, pszName: LPSTR, cchMax: UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBinItemContextMenu, IContextMenu2_iface)) as *mut CRecycleBinItemContextMenu;
    CRecycleBinItemContextMenu_GetCommandString_body(this, idCmd, uType, pwReserved, pszName, cchMax)
}
unsafe extern "system" fn CRecycleBinItemContextMenu_IContextMenu2_HandleMenuMsg(This: *mut IContextMenu2, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBinItemContextMenu, IContextMenu2_iface)) as *mut CRecycleBinItemContextMenu;
    CRecycleBinItemContextMenu_HandleMenuMsg_body(this, uMsg, wParam, lParam)
}

pub static CRECYCLEBINITEMCONTEXTMENU_ICONTEXTMENU2_VTBL: IContextMenu2Vtbl = IContextMenu2Vtbl {
    QueryInterface: CRecycleBinItemContextMenu_IContextMenu2_QueryInterface,
    AddRef: CRecycleBinItemContextMenu_IContextMenu2_AddRef,
    Release: CRecycleBinItemContextMenu_IContextMenu2_Release,
    QueryContextMenu: CRecycleBinItemContextMenu_IContextMenu2_QueryContextMenu,
    InvokeCommand: CRecycleBinItemContextMenu_IContextMenu2_InvokeCommand,
    GetCommandString: CRecycleBinItemContextMenu_IContextMenu2_GetCommandString,
    HandleMenuMsg: CRecycleBinItemContextMenu_IContextMenu2_HandleMenuMsg,
};

unsafe fn CRecycleBinItemContextMenu_QueryInterface(
    this: *mut CRecycleBinItemContextMenu, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IUnknown)) || cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IContextMenu)) {
        *ppv = core::ptr::addr_of_mut!((*this).IContextMenu2_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IContextMenu2)) {
        *ppv = core::ptr::addr_of_mut!((*this).IContextMenu2_iface) as *mut core::ffi::c_void;
    }
    else { return E_NOINTERFACE; }
    CRecycleBinItemContextMenu_AddRef(this);
    S_OK
}

unsafe fn CRecycleBinItemContextMenu_AddRef(this: *mut CRecycleBinItemContextMenu) -> ULONG {
    (*this).ref_count.fetch_add(1, core::sync::atomic::Ordering::SeqCst) as ULONG + 1
}

unsafe fn CRecycleBinItemContextMenu_Release(this: *mut CRecycleBinItemContextMenu) -> ULONG {
    let refs = (*this).ref_count.fetch_sub(1, core::sync::atomic::Ordering::SeqCst) - 1;
    if refs == 0 {
        CRecycleBinItemContextMenu_dtor_body(this);
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
pub unsafe fn CRecycleBinItemContextMenu_CreateInstance(
    cidl: UINT,
    apidl: PCUITEMID_CHILD_ARRAY,
    riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    let this = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<CRecycleBinItemContextMenu>()) as *mut CRecycleBinItemContextMenu;
    if this.is_null() { return E_OUTOFMEMORY; }
    core::ptr::write(this, CRecycleBinItemContextMenu {
        IContextMenu2_iface: IContextMenu2 { lpVtbl: core::ptr::addr_of!(CRECYCLEBINITEMCONTEXTMENU_ICONTEXTMENU2_VTBL) },
        ref_count: core::sync::atomic::AtomicI32::new(1),
        m_apidl: core::mem::zeroed(),
        m_cidl: core::mem::zeroed(),
    });
    Shell32RustObjectLock();
    CRecycleBinItemContextMenu_ctor_body(this);
    let mut hr = CRecycleBinItemContextMenu_Initialize_body(this, cidl, apidl);
    if hr >= 0 {
        hr = CRecycleBinItemContextMenu_QueryInterface(this, riid, ppv);
    }
    CRecycleBinItemContextMenu_Release(this);
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
    static IID_IDropTarget: GUID;
    static IID_IEnumIDList: GUID;
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
pub static CPP_COM_WSTR_A9EDBA2FC474: [WCHAR; 3] = [0x0025, 0x0064, 0x0000];
pub static CPP_COM_WSTR_F51AA22D711D: [WCHAR; 5] = [0x0043, 0x003A, 0x005C, 0x0058, 0x0000];

use core::ffi::c_void;

pub unsafe fn IsFolder_body(pidl: LPCITEMIDLIST) -> BOOL {
    (_ILGetFSType(pidl) & PT_FS_FOLDER_FLAG as u8) as BOOL
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

pub unsafe fn GetItemOriginalFullPath_body(Data: &BBITEMDATA) -> LPCWSTR {
    (*Data).OriginalLocation.as_ptr()
}

pub unsafe fn GetItemOriginalFileName_body(Data: &BBITEMDATA) -> LPCWSTR {
    return PathFindFileNameW(GetItemOriginalFullPath_body(Data));
}

pub unsafe fn GetItemRecycledFullPath_body(Data: &BBITEMDATA) -> LPCWSTR {
    ((Data as *const BBITEMDATA as *const u8).add((*Data).RecycledPathOffset as usize)) as LPCWSTR
}

pub unsafe fn CRecycleBinItemContextMenu_ctor_body(this: *mut CRecycleBinItemContextMenu) -> () {
    (*this).m_apidl = core::ptr::null_mut();
    (*this).m_cidl = 0;
}

pub unsafe fn CRecycleBinItemContextMenu_dtor_body(this: *mut CRecycleBinItemContextMenu) -> () {
    _ILFreeaPidl(((*this).m_apidl).cast(), (*this).m_cidl);
}

pub unsafe fn CRecycleBinItemContextMenu_Initialize_body(this: *mut CRecycleBinItemContextMenu, cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY) -> HRESULT {
    (*this).m_apidl = _ILCopyaPidl(apidl.cast(), cidl).cast();
    if (*this).m_apidl.is_null() {
        return E_OUTOFMEMORY;
    }
    (*this).m_cidl = cidl;
    S_OK
}

pub unsafe fn CRecycleBinItemContextMenu_QueryContextMenu_body(this: *mut CRecycleBinItemContextMenu, hMenu: HMENU, indexMenu: UINT, idCmdFirst: UINT, idCmdLast: UINT, uFlags: UINT) -> HRESULT {
    let mut idHigh: UINT = 0;
    let mut id: UINT;
    let mut indexMenu = indexMenu;

    id = idCmdFirst + IDC_BB_RESTORE;
    if _InsertMenuItemW(
        hMenu,
        indexMenu,
        TRUE,
        id,
        MFT_STRING,
        MAKEINTRESOURCEW(IDS_RESTORE),
        0,
    ) != 0 {
        idHigh = core::cmp::max(idHigh, id);
        indexMenu += 1;
    }

    id = idCmdFirst + IDC_BB_CUT;
    if _InsertMenuItemW(
        hMenu,
        indexMenu,
        TRUE,
        id,
        MFT_STRING,
        MAKEINTRESOURCEW(IDS_CUT),
        MFS_DISABLED,
    ) != 0 {
        idHigh = core::cmp::max(idHigh, id);
        if _InsertMenuItemW(
            hMenu,
            {
                let tmp = indexMenu;
                indexMenu += 1;
                tmp
            },
            TRUE,
            -1i32 as UINT,
            MFT_SEPARATOR,
            core::ptr::null_mut(),
            0,
        ) != 0 {
            indexMenu += 1;
        }
    }

    id = idCmdFirst + IDC_BB_DELETE;
    if _InsertMenuItemW(
        hMenu,
        indexMenu,
        TRUE,
        id,
        MFT_STRING,
        MAKEINTRESOURCEW(IDS_DELETE),
        0,
    ) != 0 {
        idHigh = core::cmp::max(idHigh, id);
        if _InsertMenuItemW(
            hMenu,
            {
                let tmp = indexMenu;
                indexMenu += 1;
                tmp
            },
            TRUE,
            -1i32 as UINT,
            MFT_SEPARATOR,
            core::ptr::null_mut(),
            0,
        ) != 0 {
            indexMenu += 1;
        }
    }

    id = idCmdFirst + IDC_BB_PROPERTIES;
    if _InsertMenuItemW(
        hMenu,
        indexMenu,
        TRUE,
        id,
        MFT_STRING,
        MAKEINTRESOURCEW(IDS_PROPERTIES),
        0,
    ) != 0 {
        idHigh = core::cmp::max(idHigh, id);
        if _InsertMenuItemW(
            hMenu,
            {
                let tmp = indexMenu;
                indexMenu += 1;
                tmp
            },
            TRUE,
            -1i32 as UINT,
            MFT_SEPARATOR,
            core::ptr::null_mut(),
            0,
        ) != 0 {
            indexMenu += 1;
        }
    }

    if idHigh != 0 {
        MAKE_HRESULT(
            SEVERITY_SUCCESS,
            0,
            idHigh - idCmdFirst + 1,
        )
    } else {
        S_OK
    }
}

pub unsafe fn ConfirmDelete_body(lpcmi: LPCMINVOKECOMMANDINFO, cidl: UINT, pidl: LPCITEMIDLIST) -> BOOL {
    let mut pData: *mut BBITEMDATA;
    if (*lpcmi).fMask & CMIC_MASK_FLAG_NO_UI != 0 {
        return TRUE;
    } else {
        pData = ValidateItem_body(pidl);
        if cidl == 1 && pData != core::ptr::null_mut() {
            let ask = if IsFolder_body(pidl) != 0 {
                ASK_DELETE_FOLDER
            } else {
                ASK_DELETE_FILE
            };
            return SHELL_ConfirmYesNoW((*lpcmi).hwnd, ask as i32, GetItemOriginalFileName_body(&*pData));
        }
    }
    let mut buf: [WCHAR; 42] = [0; 42];
    wsprintfW(buf.as_mut_ptr(), CPP_COM_WSTR_A9EDBA2FC474.as_ptr(), cidl);
    return SHELL_ConfirmYesNoW((*lpcmi).hwnd, ASK_DELETE_MULTIPLE_ITEM as i32, buf.as_ptr());
}

pub unsafe fn CreateFileOpStrings_body(cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY, RecycledPath: BOOL) -> LPWSTR {
    let mut mem: PWSTR = core::ptr::null_mut();
    let mut cb: SIZE_T = 0;
    let mut i: SIZE_T = 0;

    while i < cidl as SIZE_T {
        let pData = ValidateItem_body(*apidl.offset(i as isize));
        if pData.is_null() {
            LocalFree(mem as HLOCAL);
            return core::ptr::null_mut();
        }

        let path = if RecycledPath != 0 {
            GetItemRecycledFullPath_body(&*pData)
        } else {
            GetItemOriginalFullPath_body(&*pData)
        };

        let cbPath = (lstrlenW(path) + 1) * core::mem::size_of::<WCHAR>() as i32;
        let cb2 = cb + cbPath as SIZE_T;
        let cbTot = cb2 + core::mem::size_of::<WCHAR>() as SIZE_T;

        let newmem = if i != 0 {
            LocalReAlloc(mem as HLOCAL, cbTot, LMEM_MOVEABLE)
        } else {
            LocalAlloc(LPTR, cbTot)
        } as PWSTR;

        if newmem.is_null() {
            LocalFree(mem as HLOCAL);
            return core::ptr::null_mut();
        }

        mem = newmem;
        CopyMemory((mem as *mut u8).add(cb as usize) as *mut core::ffi::c_void, path as *const core::ffi::c_void, cbPath as usize);
        *((mem as *mut u8).add(cb as usize + cbPath as usize) as *mut WCHAR) = UNICODE_NULL;

        cb = cb2;
        i += 1;
    }

    mem
}

/// `CRecycleBin.cpp:516` `static HRESULT CALLBACK FileOpCallback(...)`. HAND-WRITTEN: rustconv
/// left it out of the worklist (recorded as an audit finding by `--skip-fn`) because its
/// `CComHeapPtr`/`CComPtr` locals and the `IID_PPV_ARG` macro have no lowering. The RAII here
/// is straightforward — one heap PIDL and one interface — so both are released explicitly on
/// every path below.
unsafe extern "system" fn FileOpCallback(
    Event: FILEOPCALLBACKEVENT,
    Src: LPCWSTR,
    _Dst: LPCWSTR,
    _Attrib: UINT,
    hrOp: HRESULT,
    CallerData: *mut core::ffi::c_void,
) -> HRESULT {
    let data = CallerData as *const FILEOPDATA;
    if (Event == FOCE_POSTDELETEITEM || Event == FOCE_POSTMOVEITEM) && hrOp >= 0 {
        let mut i: UINT = 0;
        while i < (*data).cidl {
            let pItem = ValidateItem_body((*(*data).apidl.offset(i as isize)).cast());
            if !pItem.is_null() && _wcsicmp(Src, GetItemRecycledFullPath_body(&*pItem)) == 0 {
                let identity = RECYCLEBINFILEIDENTITY {
                    DeletionTime: (*pItem).DeletionTime,
                    RecycledFullPath: GetItemRecycledFullPath_body(&*pItem),
                };
                RemoveFromRecycleBinDatabase(&identity);
                CRecycleBin_NotifyRemovedFromRecycleBin(*(*data).apidl.offset(i as isize));
                break;
            }
            i += 1;
        }
    } else if Event == FOCE_FINISHOPERATIONS {
        let pidlBB = SHCloneSpecialIDList(core::ptr::null_mut(), CSIDL_BITBUCKET, FALSE);
        if !pidlBB.is_null() {
            let mut pSF: *mut IShellFolder = core::ptr::null_mut();
            let hr = SHBindToObject(
                core::ptr::null_mut(),
                pidlBB,
                core::ptr::addr_of!(IID_IShellFolder).cast(),
                core::ptr::addr_of_mut!(pSF) as *mut *mut core::ffi::c_void,
            );
            if hr >= 0 && !pSF.is_null() {
                if IsRecycleBinEmpty(pSF) != 0 {
                    SHUpdateRecycleBinIcon();
                }
                ((*(*pSF).lpVtbl).Release)(pSF);
            }
            CoTaskMemFree(pidlBB as *mut core::ffi::c_void);
        }
    }
    S_OK
}

pub unsafe fn CRecycleBinItemContextMenu_InvokeCommand_body(this: *mut CRecycleBinItemContextMenu, lpcmi: LPCMINVOKECOMMANDINFO) -> HRESULT {
    let mut CmdId: i32 = SHELL_MapContextMenuVerbToCmdId(lpcmi, g_BBItemVerbMap.as_ptr());

    if (*lpcmi).lpVerb as usize == FCIDM_SHVIEW_CUT as usize {
        CmdId = IDC_BB_CUT as i32;
    }
    if (*lpcmi).lpVerb as usize == FCIDM_SHVIEW_DELETE as usize {
        CmdId = IDC_BB_DELETE as i32;
    }
    if (*lpcmi).lpVerb as usize == FCIDM_SHVIEW_PROPERTIES as usize {
        CmdId = IDC_BB_PROPERTIES as i32;
    }

    if CmdId == IDC_BB_RESTORE as i32 || CmdId == IDC_BB_DELETE as i32 {
        let mut hr: HRESULT = S_OK;
        if CmdId == IDC_BB_DELETE as i32 && ConfirmDelete_body(lpcmi, (*this).m_cidl, *(*this).m_apidl.offset(0)) == 0 {
            return S_OK;
        }

        let mut pszzDst: LPWSTR = core::ptr::null_mut();
        let pszzSrc: LPWSTR = CreateFileOpStrings_body((*this).m_cidl, ((*this).m_apidl).cast(), TRUE);
        if pszzSrc.is_null() {
            return E_OUTOFMEMORY;
        }
        let mut shfos = SHFILEOPSTRUCTW {
            hwnd: (*lpcmi).hwnd,
            wFunc: FO_DELETE,
            pFrom: pszzSrc,
            pTo: core::ptr::null(),
            fFlags: FOF_NOCONFIRMMKDIR as u16,
            fAnyOperationsAborted: 0,
            hNameMappings: core::ptr::null_mut(),
            lpszProgressTitle: core::ptr::null(),
        };
        if CmdId == IDC_BB_RESTORE as i32 {
            pszzDst = CreateFileOpStrings_body((*this).m_cidl, ((*this).m_apidl).cast(), FALSE);
            if pszzDst.is_null() {
                hr = E_OUTOFMEMORY;
            }
            shfos.wFunc = FO_MOVE;
            shfos.pTo = pszzDst;
            shfos.fFlags |= FOF_MULTIDESTFILES as u16;
        } else {
            shfos.fFlags |= FOF_NOCONFIRMATION as u16;
        }
        if hr >= 0 {
            if (*lpcmi).fMask & CMIC_MASK_FLAG_NO_UI != 0 {
                shfos.fFlags |= (FOF_SILENT | FOF_NOERRORUI | FOF_NOCONFIRMATION) as u16;
            }
            let mut data = FILEOPDATA {
                apidl: ((*this).m_apidl).cast(),
                cidl: (*this).m_cidl,
            };
            let res: i32 = SHELL32_FileOperation(&mut shfos, Some(FileOpCallback), core::ptr::addr_of_mut!(data) as *mut core::ffi::c_void);
            if res != 0 && res != DE_OPCANCELLED as i32 && res != ERROR_CANCELLED as i32 {
                hr = SHELL_ErrorBox((*lpcmi).hwnd.cast(), E_FAIL as UINT) as i32;
            }
        }
        LocalFree(pszzDst as *mut core::ffi::c_void);
        LocalFree(pszzSrc as *mut core::ffi::c_void);
        return hr;
    } else if CmdId == IDC_BB_CUT as i32 {
        SHELL_ErrorBox((*lpcmi).hwnd.cast(), ERROR_NOT_SUPPORTED as UINT);
        return E_NOTIMPL;
    } else if CmdId == IDC_BB_PROPERTIES as i32 {
        SHELL_ErrorBox((*lpcmi).hwnd.cast(), ERROR_NOT_SUPPORTED as UINT);
        return E_NOTIMPL;
    }
    return E_UNEXPECTED;
}

pub unsafe fn CRecycleBinItemContextMenu_GetCommandString_body(this: *mut CRecycleBinItemContextMenu, idCommand: UINT_PTR, uFlags: UINT, lpReserved: *mut UINT, lpszName: LPSTR, uMaxNameLen: UINT) -> HRESULT {
    SHELL_GetCommandStringImpl(idCommand, uFlags, lpszName, uMaxNameLen, g_BBItemVerbMap.as_ptr())
}

pub unsafe fn CRecycleBinItemContextMenu_HandleMenuMsg_body(this: *mut CRecycleBinItemContextMenu, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> HRESULT { E_NOTIMPL }


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
//     int fmt;
//     int cxChars;
// } columninfo;
//
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
//     WORD DataSize;
//     WORD EndSignature;
// };
// #include "poppack.h"
//
// static inline BOOL IsFolder(LPCITEMIDLIST pidl)
// {
//     return _ILGetFSType(pidl) & PT_FS_FOLDER_FLAG;
// }
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
// static inline LPCWSTR GetItemOriginalFullPath(const BBITEMDATA &Data)
// {
//     return Data.OriginalLocation;
// }
// static LPCWSTR GetItemOriginalFileName(const BBITEMDATA &Data)
// {
//     return PathFindFileNameW(GetItemOriginalFullPath(Data));
// }
// static inline LPCWSTR GetItemRecycledFullPath(const BBITEMDATA &Data)
// {
//     return (LPCWSTR)((BYTE*)&Data + Data.RecycledPathOffset);
// }
// HRESULT WINAPI CRecycleBinItemContextMenu::Initialize(UINT cidl, PCUITEMID_CHILD_ARRAY apidl)
// {
//     m_apidl = _ILCopyaPidl(apidl, cidl);
//     if (m_apidl == NULL)
//         return E_OUTOFMEMORY;
//     m_cidl = cidl;
//     return S_OK;
// }
// HRESULT WINAPI CRecycleBinItemContextMenu::QueryContextMenu(HMENU hMenu, UINT indexMenu, UINT idCmdFirst, UINT idCmdLast, UINT uFlags)
// {
//     UINT idHigh = 0, id;
//
//     TRACE("(%p)->(hmenu=%p indexmenu=%x cmdfirst=%x cmdlast=%x flags=%x )\n", this, hMenu, indexMenu, idCmdFirst, idCmdLast, uFlags);
//
//     id = idCmdFirst + IDC_BB_RESTORE;
//     if (_InsertMenuItemW(hMenu, indexMenu, TRUE, id, MFT_STRING, MAKEINTRESOURCEW(IDS_RESTORE), 0))
//     {
//         idHigh = core::cmp::max(idHigh, id);
//         indexMenu++;
//     }
//     id = idCmdFirst + IDC_BB_CUT;
//     if (_InsertMenuItemW(hMenu, indexMenu, TRUE, id, MFT_STRING, MAKEINTRESOURCEW(IDS_CUT), MFS_DISABLED))
//     {
//         idHigh = core::cmp::max(idHigh, id);
//         if (_InsertMenuItemW(hMenu, indexMenu++, TRUE, -1, MFT_SEPARATOR, NULL, 0))
//             indexMenu++;
//     }
//     id = idCmdFirst + IDC_BB_DELETE;
//     if (_InsertMenuItemW(hMenu, indexMenu, TRUE, id, MFT_STRING, MAKEINTRESOURCEW(IDS_DELETE), 0))
//     {
//         idHigh = core::cmp::max(idHigh, id);
//         if (_InsertMenuItemW(hMenu, indexMenu++, TRUE, -1, MFT_SEPARATOR, NULL, 0))
//             indexMenu++;
//     }
//     id = idCmdFirst + IDC_BB_PROPERTIES;
//     if (_InsertMenuItemW(hMenu, indexMenu, TRUE, id, MFT_STRING, MAKEINTRESOURCEW(IDS_PROPERTIES), 0))
//     {
//         idHigh = core::cmp::max(idHigh, id);
//         if (_InsertMenuItemW(hMenu, indexMenu++, TRUE, -1, MFT_SEPARATOR, NULL, 0))
//             indexMenu++;
//     }
//     return idHigh ? MAKE_HRESULT(SEVERITY_SUCCESS, 0, idHigh - idCmdFirst + 1) : S_OK;
// }
// static BOOL ConfirmDelete(LPCMINVOKECOMMANDINFO lpcmi, UINT cidl, LPCITEMIDLIST pidl)
// {
//     BBITEMDATA *pData;
//     if (lpcmi->fMask & CMIC_MASK_FLAG_NO_UI)
//     {
//         return TRUE;
//     }
//     else if (cidl == 1 && (pData = ValidateItem(pidl)) != NULL)
//     {
//         const UINT ask = IsFolder(pidl) ? ASK_DELETE_FOLDER : ASK_DELETE_FILE;
//         return SHELL_ConfirmYesNoW(lpcmi->hwnd, ask, GetItemOriginalFileName(*pData));
//     }
//     WCHAR buf[42];
//     wsprintfW(buf, L"%d", cidl);
//     return SHELL_ConfirmYesNoW(lpcmi->hwnd, ASK_DELETE_MULTIPLE_ITEM, buf);
// }
// static LPWSTR CreateFileOpStrings(UINT cidl, PCUITEMID_CHILD_ARRAY apidl, BOOL RecycledPath)
// {
//     PWSTR mem = NULL, newmem;
//     for (SIZE_T i = 0, cb = 0, cb2, cbPath; i < cidl; ++i, cb = cb2)
//     {
//         BBITEMDATA *pData = ValidateItem(apidl[i]);
//         if (!pData)
//         {
// fail:
//             LocalFree(mem);
//             return NULL;
//         }
//         LPCWSTR path = RecycledPath ? GetItemRecycledFullPath(*pData) : GetItemOriginalFullPath(*pData);
//         cbPath = (lstrlenW(path) + 1) * sizeof(WCHAR);
//         cb2 = cb + cbPath;
//         SIZE_T cbTot = cb2 + sizeof(WCHAR); // \0\0 termination
//         newmem = (PWSTR)(i ? LocalReAlloc(mem, cbTot, LMEM_MOVEABLE) : LocalAlloc(LPTR, cbTot));
//         if (!newmem)
//             goto fail;
//         mem = newmem;
//         CopyMemory((char*)mem + cb, path, cbPath);
//         *(PWSTR)((char*)mem + cb + cbPath) = UNICODE_NULL;
//     }
//     return mem;
// }
// HRESULT WINAPI CRecycleBinItemContextMenu::InvokeCommand(LPCMINVOKECOMMANDINFO lpcmi)
// {
//     TRACE("(%p)->(invcom=%p verb=%p wnd=%p)\n", this, lpcmi, lpcmi->lpVerb, lpcmi->hwnd);
//
//     int CmdId = SHELL_MapContextMenuVerbToCmdId(lpcmi, g_BBItemVerbMap.as_ptr());
//
//     // Handle DefView accelerators
//     if ((SIZE_T)lpcmi->lpVerb == FCIDM_SHVIEW_CUT)
//         CmdId = IDC_BB_CUT;
//     if ((SIZE_T)lpcmi->lpVerb == FCIDM_SHVIEW_DELETE)
//         CmdId = IDC_BB_DELETE;
//     if ((SIZE_T)lpcmi->lpVerb == FCIDM_SHVIEW_PROPERTIES)
//         CmdId = IDC_BB_PROPERTIES;
//
//     if (CmdId == IDC_BB_RESTORE || CmdId == IDC_BB_DELETE)
//     {
//         HRESULT hr = S_OK;
//         if (CmdId == IDC_BB_DELETE && !ConfirmDelete(lpcmi, m_cidl, m_apidl[0]))
//             return S_OK;
//
//         LPWSTR pszzDst = NULL;
//         LPWSTR pszzSrc = CreateFileOpStrings(m_cidl, m_apidl, TRUE);
//         if (!pszzSrc)
//             return E_OUTOFMEMORY;
//         SHFILEOPSTRUCTW shfos = { lpcmi->hwnd, FO_DELETE, pszzSrc, NULL, FOF_NOCONFIRMMKDIR };
//         if (CmdId == IDC_BB_RESTORE)
//         {
//             pszzDst = CreateFileOpStrings(m_cidl, m_apidl, FALSE);
//             if (!pszzDst)
//                 hr = E_OUTOFMEMORY;
//             shfos.wFunc = FO_MOVE;
//             shfos.pTo = pszzDst;
//             shfos.fFlags |= FOF_MULTIDESTFILES;
//         }
//         else // IDC_BB_DELETE
//         {
//             shfos.fFlags |= FOF_NOCONFIRMATION;
//         }
//         if (SUCCEEDED(hr))
//         {
//             if (lpcmi->fMask & CMIC_MASK_FLAG_NO_UI)
//                 shfos.fFlags |= FOF_SILENT | FOF_NOERRORUI | FOF_NOCONFIRMATION;
//             FILEOPDATA data = { m_apidl, m_cidl };
//             int res = SHELL32_FileOperation(&shfos, FileOpCallback, &data);
//             if (res && res != DE_OPCANCELLED && res != ERROR_CANCELLED)
//                 hr = SHELL_ErrorBox((*lpcmi).hwnd.cast(), E_FAIL as UINT);
//         }
//         LocalFree(pszzDst);
//         LocalFree(pszzSrc);
//         return hr;
//     }
//     else if (CmdId == IDC_BB_CUT)
//     {
//         FIXME("implement cut\n");
//         SHELL_ErrorBox((*lpcmi).hwnd.cast(), ERROR_NOT_SUPPORTED as UINT);
//         return E_NOTIMPL;
//     }
//     else if (CmdId == IDC_BB_PROPERTIES)
//     {
//         FIXME("implement properties\n");
//         SHELL_ErrorBox((*lpcmi).hwnd.cast(), ERROR_NOT_SUPPORTED as UINT);
//         return E_NOTIMPL;
//     }
//     return E_UNEXPECTED;
// }
// HRESULT WINAPI CRecycleBinItemContextMenu::GetCommandString(UINT_PTR idCommand, UINT uFlags, UINT *lpReserved, LPSTR lpszName, UINT uMaxNameLen)
// {
//     TRACE("(%p)->(idcom=%lx flags=%x %p name=%p len=%x)\n", this, idCommand, uFlags, lpReserved, lpszName, uMaxNameLen);
//     return SHELL_GetCommandStringImpl(idCommand, uFlags, lpszName, uMaxNameLen, g_BBItemVerbMap.as_ptr());
// }
// HRESULT WINAPI CRecycleBinItemContextMenu::HandleMenuMsg(UINT uMsg, WPARAM wParam, LPARAM lParam)
// {
//     TRACE("CRecycleBin_IContextMenu2Item_HandleMenuMsg (%p)->(msg=%x wp=%lx lp=%lx)\n", this, uMsg, wParam, lParam);
//
//     return E_NOTIMPL;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
