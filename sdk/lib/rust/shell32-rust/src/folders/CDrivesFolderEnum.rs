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

// Deterministic COM scaffold for CDrivesFolderEnum; method bodies are generated separately.
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
pub struct CDrivesFolderEnum {
    pub IEnumIDList_iface: IEnumIDList,
    pub ref_count: core::sync::atomic::AtomicI32,
    pub mpFirst: *mut ENUMLIST,
    pub mpLast: *mut ENUMLIST,
    pub mpCurrent: *mut ENUMLIST,
}

unsafe extern "system" fn CDrivesFolderEnum_IEnumIDList_QueryInterface(This: *mut IEnumIDList, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDrivesFolderEnum, IEnumIDList_iface)) as *mut CDrivesFolderEnum;
    CDrivesFolderEnum_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CDrivesFolderEnum_IEnumIDList_AddRef(This: *mut IEnumIDList) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDrivesFolderEnum, IEnumIDList_iface)) as *mut CDrivesFolderEnum;
    CDrivesFolderEnum_AddRef(this)
}
unsafe extern "system" fn CDrivesFolderEnum_IEnumIDList_Release(This: *mut IEnumIDList) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDrivesFolderEnum, IEnumIDList_iface)) as *mut CDrivesFolderEnum;
    CDrivesFolderEnum_Release(this)
}
unsafe extern "system" fn CDrivesFolderEnum_IEnumIDList_Next(This: *mut IEnumIDList, celt: ULONG, rgelt: *mut PITEMID_CHILD, pceltFetched: *mut ULONG) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDrivesFolderEnum, IEnumIDList_iface)) as *mut CDrivesFolderEnum;
    CDrivesFolderEnum_Next_body(this, celt, rgelt, pceltFetched)
}
unsafe extern "system" fn CDrivesFolderEnum_IEnumIDList_Skip(This: *mut IEnumIDList, celt: ULONG) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDrivesFolderEnum, IEnumIDList_iface)) as *mut CDrivesFolderEnum;
    CDrivesFolderEnum_Skip_body(this, celt)
}
unsafe extern "system" fn CDrivesFolderEnum_IEnumIDList_Reset(This: *mut IEnumIDList) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDrivesFolderEnum, IEnumIDList_iface)) as *mut CDrivesFolderEnum;
    CDrivesFolderEnum_Reset_body(this)
}
unsafe extern "system" fn CDrivesFolderEnum_IEnumIDList_Clone(This: *mut IEnumIDList, ppenum: *mut *mut IEnumIDList) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDrivesFolderEnum, IEnumIDList_iface)) as *mut CDrivesFolderEnum;
    CDrivesFolderEnum_Clone_body(this, ppenum)
}

pub static CDRIVESFOLDERENUM_IENUMIDLIST_VTBL: IEnumIDListVtbl = IEnumIDListVtbl {
    QueryInterface: CDrivesFolderEnum_IEnumIDList_QueryInterface,
    AddRef: CDrivesFolderEnum_IEnumIDList_AddRef,
    Release: CDrivesFolderEnum_IEnumIDList_Release,
    Next: CDrivesFolderEnum_IEnumIDList_Next,
    Skip: CDrivesFolderEnum_IEnumIDList_Skip,
    Reset: CDrivesFolderEnum_IEnumIDList_Reset,
    Clone: CDrivesFolderEnum_IEnumIDList_Clone,
};

unsafe fn CDrivesFolderEnum_QueryInterface(
    this: *mut CDrivesFolderEnum, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IUnknown)) || cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IEnumIDList)) {
        *ppv = core::ptr::addr_of_mut!((*this).IEnumIDList_iface) as *mut core::ffi::c_void;
    }
    else { return E_NOINTERFACE; }
    CDrivesFolderEnum_AddRef(this);
    S_OK
}

unsafe fn CDrivesFolderEnum_AddRef(this: *mut CDrivesFolderEnum) -> ULONG {
    (*this).ref_count.fetch_add(1, core::sync::atomic::Ordering::SeqCst) as ULONG + 1
}

unsafe fn CDrivesFolderEnum_Release(this: *mut CDrivesFolderEnum) -> ULONG {
    let refs = (*this).ref_count.fetch_sub(1, core::sync::atomic::Ordering::SeqCst) - 1;
    if refs == 0 {
        CDrivesFolderEnum_dtor_body(this);
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
pub unsafe fn CDrivesFolderEnum_CreateInstance(
    hwndOwner: HWND,
    dwFlags: DWORD,
    pRegEnumerator: *mut IEnumIDList,
    riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    let this = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<CDrivesFolderEnum>()) as *mut CDrivesFolderEnum;
    if this.is_null() { return E_OUTOFMEMORY; }
    core::ptr::write(this, CDrivesFolderEnum {
        IEnumIDList_iface: IEnumIDList { lpVtbl: core::ptr::addr_of!(CDRIVESFOLDERENUM_IENUMIDLIST_VTBL) },
        ref_count: core::sync::atomic::AtomicI32::new(1),
        mpFirst: core::mem::zeroed(),
        mpLast: core::mem::zeroed(),
        mpCurrent: core::mem::zeroed(),
    });
    Shell32RustObjectLock();
    CDrivesFolderEnum_ctor_body(this);
    let mut hr = CDrivesFolderEnum_Initialize_body(this, hwndOwner, dwFlags, pRegEnumerator);
    if hr >= 0 {
        hr = CDrivesFolderEnum_QueryInterface(this, riid, ppv);
    }
    CDrivesFolderEnum_Release(this);
    hr
}

// Deterministic C++ support closure selected from source identifiers.

pub const CMDID_DISCONNECT: UINT = 2;

pub const CMDID_EJECT: UINT = 1;

pub const CMDID_FORMAT: UINT = 0;

pub const IDI_SHELL_3_14_FLOPPY: UINT = 7;

pub const IDI_SHELL_CDROM: UINT = 12;

pub const IDI_SHELL_DRIVE: UINT = 9;

pub const IDI_SHELL_NETDRIVE: UINT = 10;

pub const IDI_SHELL_NOT_CONNECTED_HDD: UINT = 54;

pub const IDI_SHELL_RAMDISK: UINT = 13;

pub const IDI_SHELL_REMOVEABLE: UINT = 8;

pub const IDS_CANTDISCONNECT: UINT = 160;

pub const IDS_CANTDISMOUNTVOLUME: UINT = 157;

pub const IDS_CANTEJECTMEDIA: UINT = 158;

pub const IDS_CANTLOCKVOLUME: UINT = 156;

pub const IDS_DISCONNECT: UINT = 340;

pub const IDS_DRIVE_CDROM: UINT = 73;

pub const IDS_DRIVE_FIXED: UINT = 72;

pub const IDS_DRIVE_FLOPPY: UINT = 75;

pub const IDS_DRIVE_NETWORK: UINT = 74;

pub const IDS_DRIVE_REMOVABLE: UINT = 76;

pub const IDS_EJECT: UINT = 339;

pub const IDS_FORMATDRIVE: UINT = 328;

// IDS_PROPERTIES comes from the island (shresdef.h:225); a local copy is a second source
// of truth for the same number and the const-contradiction scan flags it as one.

pub const IDS_SHV_COLUMN_COMMENTS: UINT = 321;

pub const IDS_SHV_COLUMN_DISK_AVAILABLE: UINT = 13;

pub const IDS_SHV_COLUMN_DISK_CAPACITY: UINT = 12;

pub const IDS_SHV_COLUMN_NAME: UINT = 7;

pub const IDS_SHV_COLUMN_TYPE: UINT = 9;

pub const RETRY_COUNT: UINT = 3;

pub const RETRY_SLEEP: UINT = 250;

pub const SFGAO_CANCOPY: SFGAOF = 0x0000_0001;

pub const SFGAO_CANDELETE: SFGAOF = 0x0000_0020;

pub const SFGAO_CANLINK: SFGAOF = 0x0000_0004;

pub const SFGAO_CANRENAME: SFGAOF = 0x0000_0010;

pub const SFGAO_HASPROPSHEET: SFGAOF = 0x0000_0040;

pub const CHARS_IN_GUID: DWORD = 39;

pub const DFM_INVOKECOMMAND: DWORD = 2;

pub const DFM_MERGECONTEXTMENU: DWORD = 1;

pub const DRIVE_CDROM: DWORD = 5;

pub const DRIVE_FIXED: DWORD = 3;

pub const DRIVE_NO_ROOT_DIR: DWORD = 1;

pub const DRIVE_RAMDISK: DWORD = 6;

pub const DRIVE_REMOTE: DWORD = 4;

pub const DRIVE_REMOVABLE: DWORD = 2;

pub const DRIVE_UNKNOWN: DWORD = 0;

pub const ERROR_INVALID_FUNCTION: DWORD = 1;

pub const ERROR_PATH_NOT_FOUND: DWORD = 3;

pub const FILE_READ_ONLY_VOLUME: DWORD = 0x00080000;

pub const FILE_SHARE_READ: DWORD = 0x00000001;

pub const FILE_SHARE_WRITE: DWORD = 0x00000002;

pub const FVM_ICON: DWORD = 1;

pub const GENERIC_READ: DWORD = 0x80000000;

pub const LVCFMT_LEFT: DWORD = 0x0;

pub const LVCFMT_RIGHT: DWORD = 0x1;

pub const MB_ICONERROR: DWORD = 16;

pub const MFS_ENABLED: DWORD = 0;

pub const MFT_SEPARATOR: DWORD = 0x800;

pub const MM_ADDSEPARATOR: DWORD = 0x00000001;

pub const NO_ERROR: DWORD = 0;

pub const OPEN_EXISTING: DWORD = 3;

pub const PIDSI_COMMENTS: DWORD = 0x00000006;

pub const PID_DESCRIPTIONID: DWORD = 2;

pub const REG_DWORD: DWORD = 4;

pub const REST_NODRIVEAUTORUN: DWORD = 0x00000200;

pub const REST_NODRIVES: DWORD = 0x00000100;

pub const REST_NODRIVETYPEAUTORUN: DWORD = 0x00000400;

pub const ROSPOLICY_DRIVESFOLDER_DEFLARGEICONS: DWORD = 1;

pub const SFGAO_DROPTARGET: DWORD = 0x00000100;

pub const SFVM_DEFVIEWMODE: DWORD = 27;

pub const SFVM_FSNOTIFY: DWORD = 14;

pub const SHCNE_DRIVEADD: DWORD = 0x00000100;

pub const SHCNE_DRIVEREMOVED: DWORD = 0x00000080;

pub const SHCNE_MEDIAREMOVED: DWORD = 0x00000040;

pub const SHCNE_RENAMEFOLDER: DWORD = 0x00020000;

pub const SHCNF_FLUSHNOWAIT: DWORD = 0x3000;

pub const SHCNF_PATHW: DWORD = 0x0005;

pub const SHCOLSTATE_ONBYDEFAULT: DWORD = 0x10;

pub const SHCOLSTATE_TYPE_STR: DWORD = 1;

pub const SHCONTF_FOLDERS: DWORD = 32;

pub const SHDID_COMPUTER_CDROM: DWORD = 10;

pub const SHDID_COMPUTER_DRIVE35: DWORD = 5;

pub const SHDID_COMPUTER_FIXED: DWORD = 8;

pub const SHDID_COMPUTER_NETDRIVE: DWORD = 9;

pub const SHDID_COMPUTER_OTHER: DWORD = 12;

pub const SHDID_COMPUTER_RAMDISK: DWORD = 11;

pub const SHDID_COMPUTER_REMOVABLE: DWORD = 7;

pub const SHDID_ROOT_REGITEM: DWORD = 1;

pub const SHFMT_ID_DEFAULT: DWORD = 0xFFFF;

pub const SHGDN_FOREDITING: DWORD = 0x1000;

pub const SHGDN_INFOLDER: DWORD = 1;

pub const STGM_CREATE: DWORD = 0x00001000;

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
pub struct PREVENT_MEDIA_REMOVAL {
    pub PreventMediaRemoval: BOOLEAN,
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
pub struct ULARGE_INTEGER {
    pub QuadPart: ULONGLONG,
}

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
pub static MyComputerSFHeader: [shvheader; 5] = [
    shvheader { colnameid: (IDS_SHV_COLUMN_NAME) as WORD, colstate: (SHCOLSTATE_TYPE_STR | SHCOLSTATE_ONBYDEFAULT) as WORD, fmt: (LVCFMT_LEFT) as WORD, cxChar: (15) as WORD },
    shvheader { colnameid: (IDS_SHV_COLUMN_TYPE) as WORD, colstate: (SHCOLSTATE_TYPE_STR | SHCOLSTATE_ONBYDEFAULT) as WORD, fmt: (LVCFMT_LEFT) as WORD, cxChar: (10) as WORD },
    shvheader { colnameid: (IDS_SHV_COLUMN_DISK_CAPACITY) as WORD, colstate: (SHCOLSTATE_TYPE_STR | SHCOLSTATE_ONBYDEFAULT) as WORD, fmt: (LVCFMT_RIGHT) as WORD, cxChar: (10) as WORD },
    shvheader { colnameid: (IDS_SHV_COLUMN_DISK_AVAILABLE) as WORD, colstate: (SHCOLSTATE_TYPE_STR | SHCOLSTATE_ONBYDEFAULT) as WORD, fmt: (LVCFMT_RIGHT) as WORD, cxChar: (10) as WORD },
    shvheader { colnameid: (IDS_SHV_COLUMN_COMMENTS) as WORD, colstate: (SHCOLSTATE_TYPE_STR) as WORD, fmt: (LVCFMT_LEFT) as WORD, cxChar: (10) as WORD },
];

extern "system" {
    fn CloseHandle(handle: HANDLE) -> BOOL;
    fn DestroyMenu(menu: HMENU) -> BOOL;
    fn DestroyWindow(window: HWND) -> BOOL;
    fn ExpandEnvironmentStringsW(source: LPCWSTR, dest: LPWSTR, size: DWORD) -> DWORD;
    fn MessageBoxW(owner: HWND, text: LPCWSTR, caption: LPCWSTR, kind: UINT) -> INT;
    fn RegGetValueW(key: HKEY, subkey: LPCWSTR, value: LPCWSTR, flags: DWORD, kind: LPDWORD, data: LPVOID, bytes: LPDWORD) -> LONG;
    fn SHChangeNotify(event: LONG, flags: UINT, item1: LPCVOID, item2: LPCVOID);
    fn GetLastError() -> DWORD;
    fn lstrlenW(text: LPCWSTR) -> INT;
    fn CoTaskMemAlloc(bytes: SIZE_T) -> LPVOID;
    fn LoadStringW(instance: HINSTANCE, id: UINT, buffer: LPWSTR, capacity: INT) -> INT;
    fn CoTaskMemFree(memory: LPVOID);
    fn ILClone(pidl: LPCITEMIDLIST) -> LPITEMIDLIST;
    fn SHCreateShellFolderView(create: *const SFV_CREATE, view: *mut *mut IShellView) -> HRESULT;
}

extern "C" {
    fn HLM_GetIconW(index: UINT, destination: LPWSTR, length: DWORD, icon_index: *mut INT) -> BOOL;
    fn _ILIsSpecialFolder(pidl: LPCITEMIDLIST) -> BOOL;
    fn _ILGetDataPointer(pidl: LPCITEMIDLIST) -> *mut PIDLDATA;
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
    static CLSID_ControlPanel: GUID;
    static CLSID_MyComputer: GUID;
    static CLSID_NULL: GUID;
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

// The island/bindgen substrate (via `prelude`) is the ONLY substrate source for this slice;
// the `types`/`win32`/`shellcom` globs the assembler also emits are dropped rather than
// joined, because carrying both makes every shared name ambiguous (E0659). See prelude.rs.
use core::ffi::c_void;
use crate::prelude::*;
// Owned by the free-function slice of this same TU.
use crate::drives_folder_tu::STRRET_WSTR;
unsafe extern "C" { fn _ILCreateDrive(lpszNew: LPCWSTR) -> LPITEMIDLIST; }

pub unsafe fn CDrivesFolderEnum_ctor_body(this: *mut CDrivesFolderEnum) -> () {}

pub unsafe fn CDrivesFolderEnum_dtor_body(this: *mut CDrivesFolderEnum) -> () {
    CDrivesFolderEnum_DeleteList_body(this);
}

pub unsafe fn CDrivesFolderEnum_AddToEnumList_body(this: *mut CDrivesFolderEnum, pidl: LPITEMIDLIST) -> BOOL {
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

pub unsafe fn CDrivesFolderEnum_DeleteList_body(this: *mut CDrivesFolderEnum) -> BOOL {
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

pub unsafe fn CDrivesFolderEnum_AppendItemsFromEnumerator_body(this: *mut CDrivesFolderEnum, pEnum: *mut IEnumIDList) -> HRESULT {
    let mut pidl: LPITEMIDLIST = core::ptr::null_mut();
    let mut dwFetched: u32 = 0;

    if pEnum.is_null() {
        return E_INVALIDARG;
    }

    ((*(*pEnum).lpVtbl).Reset)(pEnum);

    while S_OK == ((*(*pEnum).lpVtbl).Next)(pEnum, 1, &mut pidl, &mut dwFetched) && dwFetched != 0 {
        CDrivesFolderEnum_AddToEnumList_body(this, pidl);
    }

    S_OK
}

pub unsafe fn CDrivesFolderEnum_Next_body(this: *mut CDrivesFolderEnum, celt: ULONG, rgelt: *mut PITEMID_CHILD, pceltFetched: *mut ULONG) -> HRESULT {
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

pub unsafe fn CDrivesFolderEnum_Skip_body(this: *mut CDrivesFolderEnum, celt: ULONG) -> HRESULT {
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

pub unsafe fn CDrivesFolderEnum_Reset_body(this: *mut CDrivesFolderEnum) -> HRESULT {
    (*this).mpCurrent = (*this).mpFirst;
    return S_OK;
}

pub unsafe fn CDrivesFolderEnum_Clone_body(this: *mut CDrivesFolderEnum, ppenum: *mut *mut IEnumIDList) -> HRESULT { E_NOTIMPL }

pub unsafe fn CDrivesFolderEnum_Initialize_body(this: *mut CDrivesFolderEnum, hwndOwner: HWND, dwFlags: DWORD, pRegEnumerator: *mut IEnumIDList) -> HRESULT {
    if dwFlags & SHCONTF_FOLDERS != 0 {
        let mut wszDriveName: [u16; 4] = [b'A' as u16, b':' as u16, b'\\' as u16, 0];
        let mut dwDrivemap = GetLogicalDrives() & !SHRestricted(REST_NODRIVES);
        while wszDriveName[0] <= b'Z' as u16 {
            if dwDrivemap & 1 != 0 {
                CDrivesFolderEnum_AddToEnumList_body(this, _ILCreateDrive(wszDriveName.as_ptr()));
            }
            dwDrivemap >>= 1;
            wszDriveName[0] += 1;
        }
    }

    CDrivesFolderEnum_AppendItemsFromEnumerator_body(this, pRegEnumerator);

    S_OK
}
