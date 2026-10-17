// Deterministic from shobjidl.h
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

// Deterministic COM scaffold for CCPLItemMenu; method bodies are generated separately.
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
pub struct CCPLItemMenu {
    pub IContextMenu2_iface: IContextMenu2,
    pub ref_count: core::sync::atomic::AtomicI32,
    pub m_apidl: *mut PITEMID_CHILD,
    pub m_cidl: UINT,
}

unsafe extern "system" fn CCPLItemMenu_IContextMenu2_QueryInterface(This: *mut IContextMenu2, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CCPLItemMenu, IContextMenu2_iface)) as *mut CCPLItemMenu;
    CCPLItemMenu_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CCPLItemMenu_IContextMenu2_AddRef(This: *mut IContextMenu2) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CCPLItemMenu, IContextMenu2_iface)) as *mut CCPLItemMenu;
    CCPLItemMenu_AddRef(this)
}
unsafe extern "system" fn CCPLItemMenu_IContextMenu2_Release(This: *mut IContextMenu2) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CCPLItemMenu, IContextMenu2_iface)) as *mut CCPLItemMenu;
    CCPLItemMenu_Release(this)
}
unsafe extern "system" fn CCPLItemMenu_IContextMenu2_QueryContextMenu(This: *mut IContextMenu2, hmenu: HMENU, indexMenu: UINT, idCmdFirst: UINT, idCmdLast: UINT, uFlags: UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CCPLItemMenu, IContextMenu2_iface)) as *mut CCPLItemMenu;
    CCPLItemMenu_QueryContextMenu_body(this, hmenu, indexMenu, idCmdFirst, idCmdLast, uFlags)
}
unsafe extern "system" fn CCPLItemMenu_IContextMenu2_InvokeCommand(This: *mut IContextMenu2, lpici: LPCMINVOKECOMMANDINFO) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CCPLItemMenu, IContextMenu2_iface)) as *mut CCPLItemMenu;
    CCPLItemMenu_InvokeCommand_body(this, lpici)
}
unsafe extern "system" fn CCPLItemMenu_IContextMenu2_GetCommandString(This: *mut IContextMenu2, idCmd: UINT_PTR, uType: UINT, pwReserved: *mut UINT, pszName: LPSTR, cchMax: UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CCPLItemMenu, IContextMenu2_iface)) as *mut CCPLItemMenu;
    CCPLItemMenu_GetCommandString_body(this, idCmd, uType, pwReserved, pszName, cchMax)
}
unsafe extern "system" fn CCPLItemMenu_IContextMenu2_HandleMenuMsg(This: *mut IContextMenu2, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CCPLItemMenu, IContextMenu2_iface)) as *mut CCPLItemMenu;
    CCPLItemMenu_HandleMenuMsg_body(this, uMsg, wParam, lParam)
}

pub static CCPLITEMMENU_ICONTEXTMENU2_VTBL: IContextMenu2Vtbl = IContextMenu2Vtbl {
    QueryInterface: CCPLItemMenu_IContextMenu2_QueryInterface,
    AddRef: CCPLItemMenu_IContextMenu2_AddRef,
    Release: CCPLItemMenu_IContextMenu2_Release,
    QueryContextMenu: CCPLItemMenu_IContextMenu2_QueryContextMenu,
    InvokeCommand: CCPLItemMenu_IContextMenu2_InvokeCommand,
    GetCommandString: CCPLItemMenu_IContextMenu2_GetCommandString,
    HandleMenuMsg: CCPLItemMenu_IContextMenu2_HandleMenuMsg,
};

unsafe fn CCPLItemMenu_QueryInterface(
    this: *mut CCPLItemMenu, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
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
    CCPLItemMenu_AddRef(this);
    S_OK
}

unsafe fn CCPLItemMenu_AddRef(this: *mut CCPLItemMenu) -> ULONG {
    (*this).ref_count.fetch_add(1, core::sync::atomic::Ordering::SeqCst) as ULONG + 1
}

unsafe fn CCPLItemMenu_Release(this: *mut CCPLItemMenu) -> ULONG {
    let refs = (*this).ref_count.fetch_sub(1, core::sync::atomic::Ordering::SeqCst) - 1;
    if refs == 0 {
        CCPLItemMenu_dtor_body(this);
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
pub unsafe fn CCPLItemMenu_CreateInstance(
    cidl: UINT,
    apidl: PCUITEMID_CHILD_ARRAY,
    riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    let this = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<CCPLItemMenu>()) as *mut CCPLItemMenu;
    if this.is_null() { return E_OUTOFMEMORY; }
    core::ptr::write(this, CCPLItemMenu {
        IContextMenu2_iface: IContextMenu2 { lpVtbl: core::ptr::addr_of!(CCPLITEMMENU_ICONTEXTMENU2_VTBL) },
        ref_count: core::sync::atomic::AtomicI32::new(1),
        m_apidl: core::mem::zeroed(),
        m_cidl: core::mem::zeroed(),
    });
    Shell32RustObjectLock();
    CCPLItemMenu_ctor_body(this);
    let mut hr = CCPLItemMenu_Initialize_body(this, cidl, apidl);
    if hr >= 0 {
        hr = CCPLItemMenu_QueryInterface(this, riid, ppv);
    }
    CCPLItemMenu_Release(this);
    hr
}

// Deterministic C++ support closure selected from source identifiers.

pub const IDR_CONTROLPANEL: UINT = 130;

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
// (via `ntdll_sys::nt`) with the identical field layout -- see CControlPanelEnum.rs's header
// for why a local duplicate is the nominal-split trap this crate's `prelude` convention exists
// to avoid.

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
pub struct IOpenControlPanel {
    pub lpVtbl: *const IOpenControlPanelVtbl,
}
#[repr(C)]
pub struct IOpenControlPanelVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IOpenControlPanel, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IOpenControlPanel) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IOpenControlPanel) -> ULONG,
    pub Open: unsafe extern "system" fn(This: *mut IOpenControlPanel, pszName: LPCWSTR, pszPage: LPCWSTR, punkSite: *mut IUnknown) -> HRESULT,
    pub GetPath: unsafe extern "system" fn(This: *mut IOpenControlPanel, pszName: LPCWSTR, pszPath: LPWSTR, cchPath: UINT) -> HRESULT,
    pub GetCurrentView: unsafe extern "system" fn(This: *mut IOpenControlPanel, pView: *mut CPVIEW) -> HRESULT,
}
extern "C" {
    pub static IID_IOpenControlPanel: IID;
}

// Deterministic from objidl.h
#[repr(C)]
pub struct IPersist {
    pub lpVtbl: *const IPersistVtbl,
}
#[repr(C)]
pub struct IPersistVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IPersist, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IPersist) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IPersist) -> ULONG,
    pub GetClassID: unsafe extern "system" fn(This: *mut IPersist, pClassID: *mut CLSID) -> HRESULT,
}
extern "C" {
    pub static IID_IPersist: IID;
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
use crate::shell32::shlexec::ShellExecuteW;
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
    static CLSID_OpenControlPanel: GUID;
    static IID_IDataObject: GUID;
    static IID_IDropTarget: GUID;
    static IID_IEnumIDList: GUID;
    static IID_IExtractIconA: GUID;
    static IID_IExtractIconW: GUID;
    static IID_IPersistFolder: GUID;
    static IID_IPersistFolder2: GUID;
    static IID_IShellFolder: GUID;
    static IID_IShellFolder2: GUID;
    static IID_IShellFolderViewCB: GUID;
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
pub static CPP_COM_WSTR_51AA45493C0B: [WCHAR; 13] = [0x0072, 0x0075, 0x006E, 0x0064, 0x006C, 0x006C, 0x0033, 0x0032, 0x002E, 0x0065, 0x0078, 0x0065, 0x0000];
pub static CPP_COM_WSTR_C7B4F4FBD140: [WCHAR; 33] = [0x0073, 0x0068, 0x0065, 0x006C, 0x006C, 0x0033, 0x0032, 0x002E, 0x0064, 0x006C, 0x006C, 0x002C, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F, 0x006C, 0x005F, 0x0052, 0x0075, 0x006E, 0x0044, 0x004C, 0x004C, 0x0020, 0x0025, 0x0073, 0x002C, 0x0025, 0x0073, 0x0000];

use core::ffi::c_void;
// Substrate comes from the bindgen island via `prelude`, and from nowhere else.
// Re-importing the same names through shellcom/types/win32 does not add anything the
// prelude lacks -- it supplies them by a second path, and every one becomes ambiguous
// at the use site. See prelude.rs for why this is "instead of", not "as well as".
use crate::prelude::*;
// `IDropTarget` is COM/menu substrate: island's own (opaque, com_sys-shaped) glob-import
// would otherwise win, and this file calls through its vtbl directly (needs the real,
// raw-fn-pointer `lpVtbl` shape). Same boundary as prelude's own IShellFolder/IDataObject
// re-exports; not in prelude's list, so imported here by name.
use crate::shellcom::IDropTarget;

// `_ILCopyaPidl`/`_ILFreeaPidl`/`_ILCreateControlPanel` are real exported symbols (wine/pidl.rs)
// typed against `crate::types`, a different nominal type than this island-first TU's PIDL
// aliases of the same layout. Re-declared here against the island types instead of casting at
// every call site (`_ILGetDataPointer`, used just below, already gets the same treatment
// further down in this file's own extern block). `ShellExecuteW` is a plain shell32.dll import
// with no island binding at all.
extern "C" {
    fn _ILCopyaPidl(apidlsrc: *const LPCITEMIDLIST, cidl: UINT) -> *mut LPITEMIDLIST;
    fn _ILFreeaPidl(apidl: *mut LPITEMIDLIST, cidl: UINT);
    fn _ILCreateControlPanel() -> LPITEMIDLIST;
}
extern "system" {
}

pub unsafe fn _ILGetCPanelPointer_body(pidl: LPCITEMIDLIST) -> *mut PIDLCPanelStruct {
    let pdata: crate::shellcom::LPPIDLDATA = _ILGetDataPointer(pidl);

    if !pdata.is_null() && (*pdata).type_ == PT_CPLAPPLET as u8 {
        return core::ptr::addr_of_mut!((*pdata).u.cpanel).cast::<PIDLCPanelStruct>();
    }

    core::ptr::null_mut()
}

pub unsafe fn CCPLItemMenu_ctor_body(this: *mut CCPLItemMenu) -> () {
    (*this).m_apidl = core::ptr::null_mut();
    (*this).m_cidl = 0;
}

pub unsafe fn CCPLItemMenu_Initialize_body(this: *mut CCPLItemMenu, cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY) -> HRESULT {
    (*this).m_cidl = cidl;
    (*this).m_apidl = _ILCopyaPidl(apidl.cast(), (*this).m_cidl);
    if (*this).m_cidl != 0 && (*this).m_apidl.is_null() {
        return E_OUTOFMEMORY;
    }

    S_OK
}

pub unsafe fn CCPLItemMenu_dtor_body(this: *mut CCPLItemMenu) -> () {
    _ILFreeaPidl((*this).m_apidl, (*this).m_cidl);
}

pub unsafe fn CCPLItemMenu_QueryContextMenu_body(this: *mut CCPLItemMenu, hMenu: HMENU, mut indexMenu: UINT, idCmdFirst: UINT, idCmdLast: UINT, uFlags: UINT) -> HRESULT {
    _InsertMenuItemW(
        hMenu,
        {
            let tmp = indexMenu;
            indexMenu = indexMenu.wrapping_add(1);
            tmp
        },
        TRUE,
        idCmdFirst,
        MFT_STRING,
        MAKEINTRESOURCEW(IDS_OPEN),
        MFS_DEFAULT,
    );
    _InsertMenuItemW(
        hMenu,
        {
            let tmp = indexMenu;
            indexMenu = indexMenu.wrapping_add(1);
            tmp
        },
        TRUE,
        IDC_STATIC as UINT,
        MFT_SEPARATOR,
        core::ptr::null_mut(),
        MFS_ENABLED,
    );
    _InsertMenuItemW(
        hMenu,
        {
            let tmp = indexMenu;
            indexMenu = indexMenu.wrapping_add(1);
            tmp
        },
        TRUE,
        idCmdFirst.wrapping_add(1),
        MFT_STRING,
        MAKEINTRESOURCEW(IDS_CREATELINK),
        MFS_ENABLED,
    );

    MAKE_HRESULT(SEVERITY_SUCCESS, 0, 2)
}

pub unsafe fn CCPLItemMenu_InvokeCommand_body(this: *mut CCPLItemMenu, lpcmi: LPCMINVOKECOMMANDINFO) -> HRESULT {
    let mut hResult: HRESULT;

    // NOTE: the paid translation doubled every external call in this function
    // (`F(F(args))` instead of `F(args)`) -- none of it compiled. Each site below is
    // collapsed to the single call the C++ makes; nothing else about the body changed.
    let pCPanel: *mut PIDLCPanelStruct = _ILGetCPanelPointer_body(*(*this).m_apidl);
    if pCPanel.is_null() {
        return E_FAIL;
    }

    if (*lpcmi).lpVerb == MAKEINTRESOURCEA(0) as *const i8 {
        let mut wszParams: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
        let wszFile: PCWSTR = CPP_COM_WSTR_51AA45493C0B.as_ptr();
        let wszFormat: PCWSTR = CPP_COM_WSTR_C7B4F4FBD140.as_ptr();

        // PIDLCPanelStruct is `#[repr(C, packed)]`: `.as_ptr()` needs `&` of the field, which
        // is E0793 on a packed struct. `addr_of!` + `.cast()` avoids ever forming that
        // reference (same fix as CControlPanelFolder.rs's `_ILCreateCPanelApplet`).
        wsprintfW(
            wszParams.as_mut_ptr(),
            wszFormat,
            core::ptr::addr_of!((*pCPanel).szName).cast::<WCHAR>(),
            core::ptr::addr_of!((*pCPanel).szName).cast::<u8>().add((*pCPanel).offsDispName as usize) as *const u16,
        );

        ShellExecuteW(core::ptr::null_mut(), core::ptr::null(), wszFile, wszParams.as_ptr(), core::ptr::null(), 0);
    } else if (*lpcmi).lpVerb == MAKEINTRESOURCEA(1) as *const i8 {
        let mut pDataObj: *mut IDataObject = core::ptr::null_mut();
        let _pDataObj_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pDataObj));
        let pidl: LPITEMIDLIST = _ILCreateControlPanel();

        // SHCreateDataObject/SHSimulateDrop are declared (island) against `IDataObject`/
        // `IDropTarget` as resolved INSIDE the island module itself -- the bindgen/com_sys
        // opaque-class rendering, not this file's lpVtbl-shaped `shellcom` versions. Same
        // layout, different Rust type: `.cast()` at each such boundary.
        hResult = SHCreateDataObject(pidl, (*this).m_cidl, (*this).m_apidl.cast(), core::ptr::null_mut(), core::ptr::addr_of!(IID_IDataObject).cast(), core::ptr::addr_of_mut!(pDataObj).cast());
        if hResult < 0 {
            if !pDataObj.is_null() {
                ((*(*pDataObj).lpVtbl).Release)(pDataObj);
                pDataObj = core::ptr::null_mut();
            }
            return hResult;
        }

        SHFree(pidl as *mut core::ffi::c_void);

        let mut psf: *mut IShellFolder = core::ptr::null_mut();
        let _psf_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(psf));
        let mut pDT: *mut IDropTarget = core::ptr::null_mut();
        let _pDT_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pDT));

        hResult = SHGetDesktopFolder(&mut psf);
        if hResult < 0 {
            if !pDataObj.is_null() {
                ((*(*pDataObj).lpVtbl).Release)(pDataObj);
                pDataObj = core::ptr::null_mut();
            }
            if !psf.is_null() {
                ((*(*psf).lpVtbl).Release)(psf);
                psf = core::ptr::null_mut();
            }
            return hResult;
        }

        hResult = ((*(*psf).lpVtbl).CreateViewObject)(psf, core::ptr::null_mut(), core::ptr::addr_of!(IID_IDropTarget).cast(), core::ptr::addr_of_mut!(pDT).cast());
        if hResult < 0 {
            if !pDataObj.is_null() {
                ((*(*pDataObj).lpVtbl).Release)(pDataObj);
                pDataObj = core::ptr::null_mut();
            }
            if !psf.is_null() {
                ((*(*psf).lpVtbl).Release)(psf);
                psf = core::ptr::null_mut();
            }
            if !pDT.is_null() {
                ((*(*pDT).lpVtbl).Release)(pDT);
                pDT = core::ptr::null_mut();
            }
            return hResult;
        }

        SHSimulateDrop(pDT.cast(), pDataObj.cast(), MK_CONTROL | MK_SHIFT, core::ptr::null_mut(), core::ptr::null_mut());

        if !pDT.is_null() {
            ((*(*pDT).lpVtbl).Release)(pDT);
            pDT = core::ptr::null_mut();
        }
        if !psf.is_null() {
            ((*(*psf).lpVtbl).Release)(psf);
            psf = core::ptr::null_mut();
        }
        if !pDataObj.is_null() {
            ((*(*pDataObj).lpVtbl).Release)(pDataObj);
            pDataObj = core::ptr::null_mut();
        }
    }
    S_OK
}

pub unsafe fn CCPLItemMenu_GetCommandString_body(this: *mut CCPLItemMenu, idCommand: UINT_PTR, uFlags: UINT, lpReserved: *mut UINT, lpszName: LPSTR, uMaxNameLen: UINT) -> HRESULT { E_FAIL }

pub unsafe fn CCPLItemMenu_HandleMenuMsg_body(this: *mut CCPLItemMenu, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> HRESULT { E_NOTIMPL }
