// Deterministic from shobjidl.h
#[repr(C)]
pub struct IShellExtInit {
    pub lpVtbl: *const IShellExtInitVtbl,
}
#[repr(C)]
pub struct IShellExtInitVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IShellExtInit, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IShellExtInit) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IShellExtInit) -> ULONG,
    pub Initialize: unsafe extern "system" fn(This: *mut IShellExtInit, pidlFolder: PCIDLIST_ABSOLUTE, pdtobj: *mut IDataObject, hkeyProgID: HKEY) -> HRESULT,
}
extern "C" {
    pub static IID_IShellExtInit: IID;
}

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
pub struct IShellPropSheetExt {
    pub lpVtbl: *const IShellPropSheetExtVtbl,
}
#[repr(C)]
pub struct IShellPropSheetExtVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IShellPropSheetExt, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IShellPropSheetExt) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IShellPropSheetExt) -> ULONG,
    pub AddPages: unsafe extern "system" fn(This: *mut IShellPropSheetExt, pfnAddPage: LPFNSVADDPROPSHEETPAGE, lParam: LPARAM) -> HRESULT,
    pub ReplacePage: unsafe extern "system" fn(This: *mut IShellPropSheetExt, uPageID: EXPPS, pfnReplaceWith: LPFNSVADDPROPSHEETPAGE, lParam: LPARAM) -> HRESULT,
}
extern "C" {
    pub static IID_IShellPropSheetExt: IID;
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

// Deterministic from objidl.h
#[repr(C)]
pub struct IDataObject {
    pub lpVtbl: *const IDataObjectVtbl,
}
#[repr(C)]
pub struct IDataObjectVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IDataObject, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IDataObject) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IDataObject) -> ULONG,
    pub GetData: unsafe extern "system" fn(This: *mut IDataObject, pformatetcIn: *mut FORMATETC, pmedium: *mut STGMEDIUM) -> HRESULT,
    pub GetDataHere: unsafe extern "system" fn(This: *mut IDataObject, pformatetc: *mut FORMATETC, pmedium: *mut STGMEDIUM) -> HRESULT,
    pub QueryGetData: unsafe extern "system" fn(This: *mut IDataObject, pformatetc: *mut FORMATETC) -> HRESULT,
    pub GetCanonicalFormatEtc: unsafe extern "system" fn(This: *mut IDataObject, pformatectIn: *mut FORMATETC, pformatetcOut: *mut FORMATETC) -> HRESULT,
    pub SetData: unsafe extern "system" fn(This: *mut IDataObject, pformatetc: *mut FORMATETC, pmedium: *mut STGMEDIUM, fRelease: BOOL) -> HRESULT,
    pub EnumFormatEtc: unsafe extern "system" fn(This: *mut IDataObject, dwDirection: DWORD, ppenumFormatEtc: *mut *mut IEnumFORMATETC) -> HRESULT,
    pub DAdvise: unsafe extern "system" fn(This: *mut IDataObject, pformatetc: *mut FORMATETC, advf: DWORD, pAdvSink: *mut IAdviseSink, pdwConnection: *mut DWORD) -> HRESULT,
    pub DUnadvise: unsafe extern "system" fn(This: *mut IDataObject, dwConnection: DWORD) -> HRESULT,
    pub EnumDAdvise: unsafe extern "system" fn(This: *mut IDataObject, ppenumAdvise: *mut *mut IEnumSTATDATA) -> HRESULT,
}
extern "C" {
    pub static IID_IDataObject: IID;
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

// Deterministic from objidl.h
#[repr(C)]
pub struct IAdviseSink {
    pub lpVtbl: *const IAdviseSinkVtbl,
}
#[repr(C)]
pub struct IAdviseSinkVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IAdviseSink, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IAdviseSink) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IAdviseSink) -> ULONG,
    pub OnDataChange: unsafe extern "system" fn(This: *mut IAdviseSink, pFormatetc: *mut FORMATETC, pStgmed: *mut STGMEDIUM) -> core::ffi::c_void,
    pub OnViewChange: unsafe extern "system" fn(This: *mut IAdviseSink, dwAspect: DWORD, lindex: LONG) -> core::ffi::c_void,
    pub OnRename: unsafe extern "system" fn(This: *mut IAdviseSink, pmk: *mut IMoniker) -> core::ffi::c_void,
    pub OnSave: unsafe extern "system" fn(This: *mut IAdviseSink) -> core::ffi::c_void,
    pub OnClose: unsafe extern "system" fn(This: *mut IAdviseSink) -> core::ffi::c_void,
}
extern "C" {
    pub static IID_IAdviseSink: IID;
}

// Deterministic from objidl.h
#[repr(C)]
pub struct IEnumFORMATETC {
    pub lpVtbl: *const IEnumFORMATETCVtbl,
}
#[repr(C)]
pub struct IEnumFORMATETCVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IEnumFORMATETC, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IEnumFORMATETC) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IEnumFORMATETC) -> ULONG,
    pub Next: unsafe extern "system" fn(This: *mut IEnumFORMATETC, celt: ULONG, rgelt: *mut FORMATETC, pceltFetched: *mut ULONG) -> HRESULT,
    pub Skip: unsafe extern "system" fn(This: *mut IEnumFORMATETC, celt: ULONG) -> HRESULT,
    pub Reset: unsafe extern "system" fn(This: *mut IEnumFORMATETC) -> HRESULT,
    pub Clone: unsafe extern "system" fn(This: *mut IEnumFORMATETC, ppenum: *mut *mut IEnumFORMATETC) -> HRESULT,
}
extern "C" {
    pub static IID_IEnumFORMATETC: IID;
}

// Deterministic from objidl.h
#[repr(C)]
pub struct IEnumSTATDATA {
    pub lpVtbl: *const IEnumSTATDATAVtbl,
}
#[repr(C)]
pub struct IEnumSTATDATAVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IEnumSTATDATA, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IEnumSTATDATA) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IEnumSTATDATA) -> ULONG,
    pub Next: unsafe extern "system" fn(This: *mut IEnumSTATDATA, celt: ULONG, rgelt: *mut STATDATA, pceltFetched: *mut ULONG) -> HRESULT,
    pub Skip: unsafe extern "system" fn(This: *mut IEnumSTATDATA, celt: ULONG) -> HRESULT,
    pub Reset: unsafe extern "system" fn(This: *mut IEnumSTATDATA) -> HRESULT,
    pub Clone: unsafe extern "system" fn(This: *mut IEnumSTATDATA, ppenum: *mut *mut IEnumSTATDATA) -> HRESULT,
}
extern "C" {
    pub static IID_IEnumSTATDATA: IID;
}

// Deterministic from objidl.h
#[repr(C)]
pub struct IMoniker {
    pub lpVtbl: *const IMonikerVtbl,
}
#[repr(C)]
pub struct IMonikerVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IMoniker, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IMoniker) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IMoniker) -> ULONG,
    pub GetClassID: unsafe extern "system" fn(This: *mut IMoniker, pClassID: *mut CLSID) -> HRESULT,
    pub IsDirty: unsafe extern "system" fn(This: *mut IMoniker) -> HRESULT,
    pub Load: unsafe extern "system" fn(This: *mut IMoniker, pStm: *mut IStream) -> HRESULT,
    pub Save: unsafe extern "system" fn(This: *mut IMoniker, pStm: *mut IStream, fClearDirty: BOOL) -> HRESULT,
    pub GetSizeMax: unsafe extern "system" fn(This: *mut IMoniker, pcbSize: *mut ULARGE_INTEGER) -> HRESULT,
    pub BindToObject: unsafe extern "system" fn(This: *mut IMoniker, pbc: *mut IBindCtx, pmkToLeft: *mut IMoniker, riidResult: REFIID, ppvResult: *mut *mut core::ffi::c_void) -> HRESULT,
    pub BindToStorage: unsafe extern "system" fn(This: *mut IMoniker, pbc: *mut IBindCtx, pmkToLeft: *mut IMoniker, riid: REFIID, ppvObj: *mut *mut core::ffi::c_void) -> HRESULT,
    pub Reduce: unsafe extern "system" fn(This: *mut IMoniker, pbc: *mut IBindCtx, dwReduceHowFar: DWORD, ppmkToLeft: *mut *mut IMoniker, ppmkReduced: *mut *mut IMoniker) -> HRESULT,
    pub ComposeWith: unsafe extern "system" fn(This: *mut IMoniker, pmkRight: *mut IMoniker, fOnlyIfNotGeneric: BOOL, ppmkComposite: *mut *mut IMoniker) -> HRESULT,
    pub Enum: unsafe extern "system" fn(This: *mut IMoniker, fForward: BOOL, ppenumMoniker: *mut *mut IEnumMoniker) -> HRESULT,
    pub IsEqual: unsafe extern "system" fn(This: *mut IMoniker, pmkOtherMoniker: *mut IMoniker) -> HRESULT,
    pub Hash: unsafe extern "system" fn(This: *mut IMoniker, pdwHash: *mut DWORD) -> HRESULT,
    pub IsRunning: unsafe extern "system" fn(This: *mut IMoniker, pbc: *mut IBindCtx, pmkToLeft: *mut IMoniker, pmkNewlyRunning: *mut IMoniker) -> HRESULT,
    pub GetTimeOfLastChange: unsafe extern "system" fn(This: *mut IMoniker, pbc: *mut IBindCtx, pmkToLeft: *mut IMoniker, pFileTime: *mut FILETIME) -> HRESULT,
    pub Inverse: unsafe extern "system" fn(This: *mut IMoniker, ppmk: *mut *mut IMoniker) -> HRESULT,
    pub CommonPrefixWith: unsafe extern "system" fn(This: *mut IMoniker, pmkOther: *mut IMoniker, ppmkPrefix: *mut *mut IMoniker) -> HRESULT,
    pub RelativePathTo: unsafe extern "system" fn(This: *mut IMoniker, pmkOther: *mut IMoniker, ppmkRelPath: *mut *mut IMoniker) -> HRESULT,
    pub GetDisplayName: unsafe extern "system" fn(This: *mut IMoniker, pbc: *mut IBindCtx, pmkToLeft: *mut IMoniker, ppszDisplayName: *mut LPOLESTR) -> HRESULT,
    pub ParseDisplayName: unsafe extern "system" fn(This: *mut IMoniker, pbc: *mut IBindCtx, pmkToLeft: *mut IMoniker, pszDisplayName: LPOLESTR, pchEaten: *mut ULONG, ppmkOut: *mut *mut IMoniker) -> HRESULT,
    pub IsSystemMoniker: unsafe extern "system" fn(This: *mut IMoniker, pdwMksys: *mut DWORD) -> HRESULT,
}
extern "C" {
    pub static IID_IMoniker: IID;
}

// Deterministic from objidl.h
#[repr(C)]
pub struct IBindCtx {
    pub lpVtbl: *const IBindCtxVtbl,
}
#[repr(C)]
pub struct IBindCtxVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IBindCtx, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IBindCtx) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IBindCtx) -> ULONG,
    pub RegisterObjectBound: unsafe extern "system" fn(This: *mut IBindCtx, punk: *mut IUnknown) -> HRESULT,
    pub RevokeObjectBound: unsafe extern "system" fn(This: *mut IBindCtx, punk: *mut IUnknown) -> HRESULT,
    pub ReleaseBoundObjects: unsafe extern "system" fn(This: *mut IBindCtx) -> HRESULT,
    pub SetBindOptions: unsafe extern "system" fn(This: *mut IBindCtx, pbindopts: *mut BIND_OPTS) -> HRESULT,
    pub GetBindOptions: unsafe extern "system" fn(This: *mut IBindCtx, pbindopts: *mut BIND_OPTS) -> HRESULT,
    pub GetRunningObjectTable: unsafe extern "system" fn(This: *mut IBindCtx, pprot: *mut *mut IRunningObjectTable) -> HRESULT,
    pub RegisterObjectParam: unsafe extern "system" fn(This: *mut IBindCtx, pszKey: LPOLESTR, punk: *mut IUnknown) -> HRESULT,
    pub GetObjectParam: unsafe extern "system" fn(This: *mut IBindCtx, pszKey: LPOLESTR, ppunk: *mut *mut IUnknown) -> HRESULT,
    pub EnumObjectParam: unsafe extern "system" fn(This: *mut IBindCtx, ppenum: *mut *mut IEnumString) -> HRESULT,
    pub RevokeObjectParam: unsafe extern "system" fn(This: *mut IBindCtx, pszKey: LPOLESTR) -> HRESULT,
}
extern "C" {
    pub static IID_IBindCtx: IID;
}

// Deterministic from objidl.h
#[repr(C)]
pub struct IEnumMoniker {
    pub lpVtbl: *const IEnumMonikerVtbl,
}
#[repr(C)]
pub struct IEnumMonikerVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IEnumMoniker, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IEnumMoniker) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IEnumMoniker) -> ULONG,
    pub Next: unsafe extern "system" fn(This: *mut IEnumMoniker, celt: ULONG, rgelt: *mut *mut IMoniker, pceltFetched: *mut ULONG) -> HRESULT,
    pub Skip: unsafe extern "system" fn(This: *mut IEnumMoniker, celt: ULONG) -> HRESULT,
    pub Reset: unsafe extern "system" fn(This: *mut IEnumMoniker) -> HRESULT,
    pub Clone: unsafe extern "system" fn(This: *mut IEnumMoniker, ppenum: *mut *mut IEnumMoniker) -> HRESULT,
}
extern "C" {
    pub static IID_IEnumMoniker: IID;
}

// Deterministic from objidl.h
#[repr(C)]
pub struct IStream {
    pub lpVtbl: *const IStreamVtbl,
}
#[repr(C)]
pub struct IStreamVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IStream, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IStream) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IStream) -> ULONG,
    pub Read: unsafe extern "system" fn(This: *mut IStream, pv: *mut core::ffi::c_void, cb: ULONG, pcbRead: *mut ULONG) -> HRESULT,
    pub Write: unsafe extern "system" fn(This: *mut IStream, pv: *const core::ffi::c_void, cb: ULONG, pcbWritten: *mut ULONG) -> HRESULT,
    pub Seek: unsafe extern "system" fn(This: *mut IStream, dlibMove: LARGE_INTEGER, dwOrigin: DWORD, plibNewPosition: *mut ULARGE_INTEGER) -> HRESULT,
    pub SetSize: unsafe extern "system" fn(This: *mut IStream, libNewSize: ULARGE_INTEGER) -> HRESULT,
    pub CopyTo: unsafe extern "system" fn(This: *mut IStream, pstm: *mut IStream, cb: ULARGE_INTEGER, pcbRead: *mut ULARGE_INTEGER, pcbWritten: *mut ULARGE_INTEGER) -> HRESULT,
    pub Commit: unsafe extern "system" fn(This: *mut IStream, grfCommitFlags: DWORD) -> HRESULT,
    pub Revert: unsafe extern "system" fn(This: *mut IStream) -> HRESULT,
    pub LockRegion: unsafe extern "system" fn(This: *mut IStream, libOffset: ULARGE_INTEGER, cb: ULARGE_INTEGER, dwLockType: DWORD) -> HRESULT,
    pub UnlockRegion: unsafe extern "system" fn(This: *mut IStream, libOffset: ULARGE_INTEGER, cb: ULARGE_INTEGER, dwLockType: DWORD) -> HRESULT,
    pub Stat: unsafe extern "system" fn(This: *mut IStream, pstatstg: *mut STATSTG, grfStatFlag: DWORD) -> HRESULT,
    pub Clone: unsafe extern "system" fn(This: *mut IStream, ppstm: *mut *mut IStream) -> HRESULT,
}
extern "C" {
    pub static IID_IStream: IID;
}

// Deterministic from objidl.h
#[repr(C)]
pub struct IEnumString {
    pub lpVtbl: *const IEnumStringVtbl,
}
#[repr(C)]
pub struct IEnumStringVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IEnumString, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IEnumString) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IEnumString) -> ULONG,
    pub Next: unsafe extern "system" fn(This: *mut IEnumString, celt: ULONG, rgelt: *mut LPOLESTR, pceltFetched: *mut ULONG) -> HRESULT,
    pub Skip: unsafe extern "system" fn(This: *mut IEnumString, celt: ULONG) -> HRESULT,
    pub Reset: unsafe extern "system" fn(This: *mut IEnumString) -> HRESULT,
    pub Clone: unsafe extern "system" fn(This: *mut IEnumString, ppenum: *mut *mut IEnumString) -> HRESULT,
}
extern "C" {
    pub static IID_IEnumString: IID;
}

// Deterministic from objidl.h
#[repr(C)]
pub struct IRunningObjectTable {
    pub lpVtbl: *const IRunningObjectTableVtbl,
}
#[repr(C)]
pub struct IRunningObjectTableVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IRunningObjectTable, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IRunningObjectTable) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IRunningObjectTable) -> ULONG,
    pub Register: unsafe extern "system" fn(This: *mut IRunningObjectTable, grfFlags: DWORD, punkObject: *mut IUnknown, pmkObjectName: *mut IMoniker, pdwRegister: *mut DWORD) -> HRESULT,
    pub Revoke: unsafe extern "system" fn(This: *mut IRunningObjectTable, dwRegister: DWORD) -> HRESULT,
    pub IsRunning: unsafe extern "system" fn(This: *mut IRunningObjectTable, pmkObjectName: *mut IMoniker) -> HRESULT,
    pub GetObject: unsafe extern "system" fn(This: *mut IRunningObjectTable, pmkObjectName: *mut IMoniker, ppunkObject: *mut *mut IUnknown) -> HRESULT,
    pub NoteChangeTime: unsafe extern "system" fn(This: *mut IRunningObjectTable, dwRegister: DWORD, pfiletime: *mut FILETIME) -> HRESULT,
    pub GetTimeOfLastChange: unsafe extern "system" fn(This: *mut IRunningObjectTable, pmkObjectName: *mut IMoniker, pfiletime: *mut FILETIME) -> HRESULT,
    pub EnumRunning: unsafe extern "system" fn(This: *mut IRunningObjectTable, ppenumMoniker: *mut *mut IEnumMoniker) -> HRESULT,
}
extern "C" {
    pub static IID_IRunningObjectTable: IID;
}

#[repr(C)]
pub struct CFileVersionInfo {
    pub m_pInfo: PVOID,
    pub m_wLang: WORD,
    pub m_wCode: WORD,
    pub m_wszLang: [WCHAR; 64],
}

// Deterministic COM scaffold for CFileDefExt; method bodies are generated separately.
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
pub struct CFileDefExt {
    pub IShellExtInit_iface: IShellExtInit,
    pub IContextMenu_iface: IContextMenu,
    pub IShellPropSheetExt_iface: IShellPropSheetExt,
    pub IObjectWithSite_iface: IObjectWithSite,
    pub ref_count: core::sync::atomic::AtomicI32,
    pub m_pUnkSite: *mut IUnknown,
    pub m_wszPath: [WCHAR; MAX_PATH],
    pub m_VerInfo: CFileVersionInfo,
    pub m_bDir: BOOL,
    pub m_bMultifile: BOOL,
    pub m_pidlFolder: LPITEMIDLIST,
    pub m_pidls: *mut LPITEMIDLIST,
    pub m_cidl: UINT,
    pub m_cFiles: DWORD,
    pub m_cFolders: DWORD,
    pub m_DirSize: ULARGE_INTEGER,
    pub m_DirSizeOnDisc: ULARGE_INTEGER,
    pub m_hWndDirStatsDlg: HWND,
    pub m_Destroyed: LONG,
    pub m_szFolderIconPath: [WCHAR; MAX_PATH],
    pub m_nFolderIconIndex: INT,
    pub m_hFolderIcon: HICON,
    pub m_bFolderIconIsSet: BOOL,
}

unsafe extern "system" fn CFileDefExt_IShellExtInit_QueryInterface(This: *mut IShellExtInit, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFileDefExt, IShellExtInit_iface)) as *mut CFileDefExt;
    CFileDefExt_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CFileDefExt_IShellExtInit_AddRef(This: *mut IShellExtInit) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFileDefExt, IShellExtInit_iface)) as *mut CFileDefExt;
    CFileDefExt_AddRef(this)
}
unsafe extern "system" fn CFileDefExt_IShellExtInit_Release(This: *mut IShellExtInit) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFileDefExt, IShellExtInit_iface)) as *mut CFileDefExt;
    CFileDefExt_Release(this)
}
unsafe extern "system" fn CFileDefExt_IShellExtInit_Initialize(This: *mut IShellExtInit, pidlFolder: PCIDLIST_ABSOLUTE, pdtobj: *mut IDataObject, hkeyProgID: HKEY) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFileDefExt, IShellExtInit_iface)) as *mut CFileDefExt;
    CFileDefExt_Initialize_body(this, pidlFolder, pdtobj, hkeyProgID)
}

pub static CFILEDEFEXT_ISHELLEXTINIT_VTBL: IShellExtInitVtbl = IShellExtInitVtbl {
    QueryInterface: CFileDefExt_IShellExtInit_QueryInterface,
    AddRef: CFileDefExt_IShellExtInit_AddRef,
    Release: CFileDefExt_IShellExtInit_Release,
    Initialize: CFileDefExt_IShellExtInit_Initialize,
};

unsafe extern "system" fn CFileDefExt_IContextMenu_QueryInterface(This: *mut IContextMenu, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFileDefExt, IContextMenu_iface)) as *mut CFileDefExt;
    CFileDefExt_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CFileDefExt_IContextMenu_AddRef(This: *mut IContextMenu) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFileDefExt, IContextMenu_iface)) as *mut CFileDefExt;
    CFileDefExt_AddRef(this)
}
unsafe extern "system" fn CFileDefExt_IContextMenu_Release(This: *mut IContextMenu) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFileDefExt, IContextMenu_iface)) as *mut CFileDefExt;
    CFileDefExt_Release(this)
}
unsafe extern "system" fn CFileDefExt_IContextMenu_QueryContextMenu(This: *mut IContextMenu, hmenu: HMENU, indexMenu: UINT, idCmdFirst: UINT, idCmdLast: UINT, uFlags: UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFileDefExt, IContextMenu_iface)) as *mut CFileDefExt;
    CFileDefExt_QueryContextMenu_body(this, hmenu, indexMenu, idCmdFirst, idCmdLast, uFlags)
}
unsafe extern "system" fn CFileDefExt_IContextMenu_InvokeCommand(This: *mut IContextMenu, lpici: LPCMINVOKECOMMANDINFO) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFileDefExt, IContextMenu_iface)) as *mut CFileDefExt;
    CFileDefExt_InvokeCommand_body(this, lpici)
}
unsafe extern "system" fn CFileDefExt_IContextMenu_GetCommandString(This: *mut IContextMenu, idCmd: UINT_PTR, uType: UINT, pwReserved: *mut UINT, pszName: LPSTR, cchMax: UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFileDefExt, IContextMenu_iface)) as *mut CFileDefExt;
    CFileDefExt_GetCommandString_body(this, idCmd, uType, pwReserved, pszName, cchMax)
}

pub static CFILEDEFEXT_ICONTEXTMENU_VTBL: IContextMenuVtbl = IContextMenuVtbl {
    QueryInterface: CFileDefExt_IContextMenu_QueryInterface,
    AddRef: CFileDefExt_IContextMenu_AddRef,
    Release: CFileDefExt_IContextMenu_Release,
    QueryContextMenu: CFileDefExt_IContextMenu_QueryContextMenu,
    InvokeCommand: CFileDefExt_IContextMenu_InvokeCommand,
    GetCommandString: CFileDefExt_IContextMenu_GetCommandString,
};

unsafe extern "system" fn CFileDefExt_IShellPropSheetExt_QueryInterface(This: *mut IShellPropSheetExt, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFileDefExt, IShellPropSheetExt_iface)) as *mut CFileDefExt;
    CFileDefExt_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CFileDefExt_IShellPropSheetExt_AddRef(This: *mut IShellPropSheetExt) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFileDefExt, IShellPropSheetExt_iface)) as *mut CFileDefExt;
    CFileDefExt_AddRef(this)
}
unsafe extern "system" fn CFileDefExt_IShellPropSheetExt_Release(This: *mut IShellPropSheetExt) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFileDefExt, IShellPropSheetExt_iface)) as *mut CFileDefExt;
    CFileDefExt_Release(this)
}
unsafe extern "system" fn CFileDefExt_IShellPropSheetExt_AddPages(This: *mut IShellPropSheetExt, pfnAddPage: LPFNSVADDPROPSHEETPAGE, lParam: LPARAM) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFileDefExt, IShellPropSheetExt_iface)) as *mut CFileDefExt;
    CFileDefExt_AddPages_body(this, pfnAddPage, lParam)
}
unsafe extern "system" fn CFileDefExt_IShellPropSheetExt_ReplacePage(This: *mut IShellPropSheetExt, uPageID: EXPPS, pfnReplaceWith: LPFNSVADDPROPSHEETPAGE, lParam: LPARAM) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFileDefExt, IShellPropSheetExt_iface)) as *mut CFileDefExt;
    CFileDefExt_ReplacePage_body(this, uPageID, pfnReplaceWith, lParam)
}

pub static CFILEDEFEXT_ISHELLPROPSHEETEXT_VTBL: IShellPropSheetExtVtbl = IShellPropSheetExtVtbl {
    QueryInterface: CFileDefExt_IShellPropSheetExt_QueryInterface,
    AddRef: CFileDefExt_IShellPropSheetExt_AddRef,
    Release: CFileDefExt_IShellPropSheetExt_Release,
    AddPages: CFileDefExt_IShellPropSheetExt_AddPages,
    ReplacePage: CFileDefExt_IShellPropSheetExt_ReplacePage,
};

unsafe extern "system" fn CFileDefExt_IObjectWithSite_QueryInterface(This: *mut IObjectWithSite, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFileDefExt, IObjectWithSite_iface)) as *mut CFileDefExt;
    CFileDefExt_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CFileDefExt_IObjectWithSite_AddRef(This: *mut IObjectWithSite) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFileDefExt, IObjectWithSite_iface)) as *mut CFileDefExt;
    CFileDefExt_AddRef(this)
}
unsafe extern "system" fn CFileDefExt_IObjectWithSite_Release(This: *mut IObjectWithSite) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFileDefExt, IObjectWithSite_iface)) as *mut CFileDefExt;
    CFileDefExt_Release(this)
}
unsafe extern "system" fn CFileDefExt_IObjectWithSite_SetSite(This: *mut IObjectWithSite, pUnkSite: *mut IUnknown) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFileDefExt, IObjectWithSite_iface)) as *mut CFileDefExt;
    CFileDefExt_SetSite_body(this, pUnkSite)
}
unsafe extern "system" fn CFileDefExt_IObjectWithSite_GetSite(This: *mut IObjectWithSite, riid: REFIID, ppvSite: *mut PVOID) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFileDefExt, IObjectWithSite_iface)) as *mut CFileDefExt;
    CFileDefExt_GetSite_body(this, riid, ppvSite)
}

pub static CFILEDEFEXT_IOBJECTWITHSITE_VTBL: IObjectWithSiteVtbl = IObjectWithSiteVtbl {
    QueryInterface: CFileDefExt_IObjectWithSite_QueryInterface,
    AddRef: CFileDefExt_IObjectWithSite_AddRef,
    Release: CFileDefExt_IObjectWithSite_Release,
    SetSite: CFileDefExt_IObjectWithSite_SetSite,
    GetSite: CFileDefExt_IObjectWithSite_GetSite,
};

unsafe fn CFileDefExt_QueryInterface(
    this: *mut CFileDefExt, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IUnknown)) || cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IShellExtInit)) {
        *ppv = core::ptr::addr_of_mut!((*this).IShellExtInit_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IContextMenu)) {
        *ppv = core::ptr::addr_of_mut!((*this).IContextMenu_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IShellPropSheetExt)) {
        *ppv = core::ptr::addr_of_mut!((*this).IShellPropSheetExt_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IObjectWithSite)) {
        *ppv = core::ptr::addr_of_mut!((*this).IObjectWithSite_iface) as *mut core::ffi::c_void;
    }
    else { return E_NOINTERFACE; }
    CFileDefExt_AddRef(this);
    S_OK
}

unsafe fn CFileDefExt_AddRef(this: *mut CFileDefExt) -> ULONG {
    (*this).ref_count.fetch_add(1, core::sync::atomic::Ordering::SeqCst) as ULONG + 1
}

unsafe fn CFileDefExt_Release(this: *mut CFileDefExt) -> ULONG {
    let refs = (*this).ref_count.fetch_sub(1, core::sync::atomic::Ordering::SeqCst) - 1;
    if refs == 0 {
        CFileDefExt_dtor_body(this);
        CFileDefExt_CObjectWithSiteBase_base_dtor_body(this);
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

// Activation: CLSID_ShellFileDefExt currently maps to CFileDefExt in shell32.cpp.
// The generated C++ adapter redirects ATL activation to this factory.
#[no_mangle] pub unsafe extern "system" fn CFileDefExt_CreateInstance(
    outer: *mut IUnknown, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if !outer.is_null() { return CLASS_E_NOAGGREGATION; }
    let this = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<CFileDefExt>()) as *mut CFileDefExt;
    if this.is_null() { return E_OUTOFMEMORY; }
    core::ptr::write(this, CFileDefExt {
        IShellExtInit_iface: IShellExtInit { lpVtbl: core::ptr::addr_of!(CFILEDEFEXT_ISHELLEXTINIT_VTBL) },
        IContextMenu_iface: IContextMenu { lpVtbl: core::ptr::addr_of!(CFILEDEFEXT_ICONTEXTMENU_VTBL) },
        IShellPropSheetExt_iface: IShellPropSheetExt { lpVtbl: core::ptr::addr_of!(CFILEDEFEXT_ISHELLPROPSHEETEXT_VTBL) },
        IObjectWithSite_iface: IObjectWithSite { lpVtbl: core::ptr::addr_of!(CFILEDEFEXT_IOBJECTWITHSITE_VTBL) },
        ref_count: core::sync::atomic::AtomicI32::new(1),
        m_pUnkSite: core::mem::zeroed(),
        m_wszPath: [0; MAX_PATH],
        m_VerInfo: core::mem::zeroed(),
        m_bDir: core::mem::zeroed(),
        m_bMultifile: core::mem::zeroed(),
        m_pidlFolder: core::mem::zeroed(),
        m_pidls: core::mem::zeroed(),
        m_cidl: 0,
        m_cFiles: core::mem::zeroed(),
        m_cFolders: core::mem::zeroed(),
        m_DirSize: core::mem::zeroed(),
        m_DirSizeOnDisc: core::mem::zeroed(),
        m_hWndDirStatsDlg: core::mem::zeroed(),
        m_Destroyed: 0,
        m_szFolderIconPath: [0; MAX_PATH],
        m_nFolderIconIndex: core::mem::zeroed(),
        m_hFolderIcon: core::mem::zeroed(),
        m_bFolderIconIsSet: core::mem::zeroed(),
    });
    Shell32RustObjectLock();
    CFileDefExt_ctor_body(this);
    let hr = CFileDefExt_QueryInterface(this, riid, ppv);
    CFileDefExt_Release(this);
    hr
}

// Deterministic C++ support closure selected from source identifiers.

pub const IDC_FOLDERCUST_CHANGE_ICON: UINT = 14007;

pub const IDC_FOLDERCUST_CHECKBOX: UINT = 14002;

pub const IDC_FOLDERCUST_CHOOSE_PIC: UINT = 14003;

pub const IDC_FOLDERCUST_COMBOBOX: UINT = 14001;

pub const IDC_FOLDERCUST_ICON: UINT = 14006;

pub const IDC_FOLDERCUST_RESTORE_DEFAULTS: UINT = 14004;

pub const IDD_FILE_PROPERTIES: UINT = 8;

pub const IDD_FILE_VERSION: UINT = 15;

pub const IDD_FOLDER_CUSTOMIZE: UINT = 38;

pub const IDD_FOLDER_PROPERTIES: UINT = 9;

pub const IDI_SHELL_FOLDER: UINT = 4;

pub const IDI_SHELL_MULTIPLE_FILES: UINT = 133;

pub const IDR_FILEDEFEXT: UINT = 147;

pub const IDS_BYTES_FORMAT: UINT = 177;

pub const IDS_EXE_DESCRIPTION: UINT = 191;

pub const IDS_FILE_FOLDER: UINT = 308;

pub const IDS_MULTIPLETYPES: UINT = 8705;

pub const IDS_UNKNOWN_APP: UINT = 190;

pub const IDS_VARIOUSFOLDERS: UINT = 8709;

pub const BM_GETCHECK: DWORD = 240;

pub const BM_SETCHECK: DWORD = 241;

pub const BM_SETSTYLE: DWORD = 244;

pub const BST_CHECKED: DWORD = 1;

pub const BST_INDETERMINATE: DWORD = 2;

pub const BS_AUTO3STATE: DWORD = 6;

pub const CF_HDROP: DWORD = 15;

pub const DATE_LONGDATE: DWORD = 2;

pub const DVASPECT_CONTENT: DWORD = 1;

pub const DWLP_MSGRESULT: INT = 0;

pub const EM_SETREADONLY: DWORD = 207;

pub const EN_CHANGE: DWORD = 768;

pub const FILE_ATTRIBUTE_ARCHIVE: DWORD = 0x00000020;

pub const FILE_ATTRIBUTE_DIRECTORY: DWORD = 0x00000010;

pub const FILE_ATTRIBUTE_HIDDEN: DWORD = 0x00000002;

pub const FILE_ATTRIBUTE_READONLY: DWORD = 0x00000001;

pub const FILE_ATTRIBUTE_SYSTEM: DWORD = 0x00000004;

pub const FILE_FLAG_BACKUP_SEMANTICS: DWORD = 0x02000000;

pub const FILE_FLAG_OPEN_NO_RECALL: DWORD = 0x00100000;

pub const FILE_READ_ATTRIBUTES: DWORD = 0x00000080;

pub const FILE_SHARE_DELETE: DWORD = 0x00000004;

pub const FILE_SHARE_READ: DWORD = 0x00000001;

pub const FILE_SHARE_WRITE: DWORD = 0x00000002;

pub const FileBasicInformation: DWORD = 4;

pub const FileStandardInformation: DWORD = 5;

pub const LANG_ENGLISH: DWORD = 0x09;

pub const LBN_SELCHANGE: DWORD = 1;

pub const LB_ADDSTRING: DWORD = 384;

pub const LB_GETCURSEL: DWORD = 392;

pub const LB_GETITEMDATA: DWORD = 409;

pub const LB_SETCURSEL: DWORD = 390;

pub const LB_SETITEMDATA: DWORD = 410;

pub const LOCALE_SDECIMAL: DWORD = 14;

pub const LOCALE_SGROUPING: DWORD = 16;

pub const LOCALE_STHOUSAND: DWORD = 15;

pub const OAIF_FORCE_REGISTRATION: DWORD = 8;

pub const OAIF_REGISTER_EXT: DWORD = 2;

pub const OPEN_EXISTING: DWORD = 3;

pub const PSNRET_INVALID_NOCHANGEPAGE: DWORD = 2;

pub const PSNRET_NOERROR: DWORD = 0;

pub const SHGDN_FORADDRESSBAR: DWORD = 0x4000;

pub const SHGFI_ICON: DWORD = 256;

pub const SHGFI_TYPENAME: DWORD = 1024;

pub const SHGFI_USEFILEATTRIBUTES: DWORD = 16;

pub const SM_CXSMICON: DWORD = 49;

pub const SM_CYSMICON: DWORD = 50;

pub const STM_GETICON: DWORD = 369;

pub const STM_SETICON: DWORD = 368;

pub const SUBLANG_ENGLISH_US: DWORD = 0x01;

pub const SWP_NOMOVE: DWORD = 2;

pub const SWP_NOZORDER: DWORD = 4;

pub const SW_HIDE: DWORD = 0;

pub const SW_SHOW: DWORD = 5;

pub const TYMED_HGLOBAL: DWORD = 1;

pub const WM_APP: DWORD = 32768;

pub const WM_COMMAND: DWORD = 273;

pub const WM_DESTROY: DWORD = 2;

pub const WM_GETTEXT: DWORD = 13;

pub const WM_INITDIALOG: DWORD = 272;
pub const WM_NOTIFY: DWORD = 0x004e;

pub const WM_SETTEXT: DWORD = 12;

pub const WM_UPDATEDIRSTATS: UINT = WM_APP;
pub const INVALID_FILE_ATTRIBUTES: DWORD = 0xffff_ffff;
pub const LB_ERR: LRESULT = -1;
pub const DWLP_USER: INT = (core::mem::size_of::<LRESULT>() + core::mem::size_of::<DLGPROC>()) as INT;
pub const GWL_STYLE: INT = -16;
pub const GWL_EXSTYLE: INT = -20;
pub const PSN_APPLY: UINT = 0xffff_ff36;
pub const PSPCB_RELEASE: UINT = 1;
pub const PSM_CHANGED: UINT = 1128;
pub const PSM_QUERYSIBLINGS: UINT = 1132;
pub const LOCALE_USER_DEFAULT: DWORD = 0x0400;
pub const GetFileExInfoStandard: DWORD = 0;
pub const CTF_COINIT: DWORD = 0x8;
pub const VS_FF_PRIVATEBUILD: DWORD = 0x8;
pub const VS_FF_SPECIALBUILD: DWORD = 0x20;
pub const HWND_TOP: HWND = core::ptr::null_mut();

pub type NTSTATUS = LONG;
pub type PULARGE_INTEGER = *mut ULARGE_INTEGER;
pub type LPFILETIME = *mut FILETIME;
pub type HDROP = HANDLE;
pub type DLGPROC = unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> INT_PTR;

#[repr(C)]
pub struct IO_STATUS_BLOCK {
    pub Status: NTSTATUS,
    pub Information: ULONG_PTR,
}
pub type _IO_STATUS_BLOCK = IO_STATUS_BLOCK;

#[repr(C)]
pub struct PROPSHEETPAGEW {
    pub dwSize: DWORD,
    pub dwFlags: DWORD,
    pub hInstance: HINSTANCE,
    pub pszTemplate: LPCWSTR,
    pub hIcon: HICON,
    pub pszTitle: LPCWSTR,
    pub pfnDlgProc: DLGPROC,
    pub lParam: LPARAM,
}
pub type PROPSHEETPAGE = PROPSHEETPAGEW;

#[repr(C)]
pub struct OPENASINFO {
    pub pcszFile: LPCWSTR,
    pub pcszClass: LPCWSTR,
    pub oaifInFlags: DWORD,
}

#[repr(C)]
pub struct CIDA {
    pub cidl: UINT,
    pub aoffset: [UINT; 1],
}

#[repr(C)]
pub struct DIRTREESTATS {
    pub pDirCount: *mut DWORD,
    pub pFileCount: *mut DWORD,
    pub pTotVirtSize: PULARGE_INTEGER,
    pub pTotPhysSize: PULARGE_INTEGER,
    pub nTick: UINT,
    pub fAttribSet: UINT,
    pub fAttribAll: UINT,
    pub bMultipleTypes: BOOL,
    pub bDeepRecurse: BOOL,
    pub szType: [WCHAR; 100],
}

#[repr(C)]
pub struct BIND_OPTS {
    pub cbStruct: DWORD,
    pub grfFlags: DWORD,
    pub grfMode: DWORD,
    pub dwTickCountDeadline: DWORD,
}

#[repr(C)]
pub struct DVTARGETDEVICE {
    pub tdSize: DWORD,
    pub tdDriverNameOffset: WORD,
    pub tdDeviceNameOffset: WORD,
    pub tdPortNameOffset: WORD,
    pub tdExtDevmodeOffset: WORD,
    pub tdData: [BYTE; 1],
}

#[repr(C)]
pub struct FILE_BASIC_INFORMATION {
    pub CreationTime: LARGE_INTEGER,
    pub LastAccessTime: LARGE_INTEGER,
    pub LastWriteTime: LARGE_INTEGER,
    pub ChangeTime: LARGE_INTEGER,
    pub FileAttributes: ULONG,
}

#[repr(C)]
pub struct FILE_STANDARD_INFORMATION {
    pub AllocationSize: LARGE_INTEGER,
    pub EndOfFile: LARGE_INTEGER,
    pub NumberOfLinks: ULONG,
    pub DeletePending: BOOLEAN,
    pub Directory: BOOLEAN,
}

#[repr(C)]
pub struct NUMBERFMTW {
    pub NumDigits: UINT,
    pub LeadingZero: UINT,
    pub Grouping: UINT,
    pub lpDecimalSep: LPWSTR,
    pub lpThousandSep: LPWSTR,
    pub NegativeOrder: UINT,
}

#[repr(C)]
pub struct STATDATA {
    pub formatetc: FORMATETC,
    pub advf: DWORD,
    pub pAdvSink: *mut IAdviseSink,
    pub dwConnection: DWORD,
}

#[repr(C)]
pub struct STATSTG {
    pub pwcsName: LPOLESTR,
    pub r#type: DWORD,
    pub cbSize: ULARGE_INTEGER,
    pub mtime: FILETIME,
    pub ctime: FILETIME,
    pub atime: FILETIME,
    pub grfMode: DWORD,
    pub grfLocksSupported: DWORD,
    pub clsid: CLSID,
    pub grfStateBits: DWORD,
    pub reserved: DWORD,
}

#[repr(C)]
pub struct ULARGE_INTEGER {
    pub QuadPart: ULONGLONG,
}

#[repr(C)]
pub struct VS_FIXEDFILEINFO {
    pub dwSignature: DWORD,
    pub dwStrucVersion: DWORD,
    pub dwFileVersionMS: DWORD,
    pub dwFileVersionLS: DWORD,
    pub dwProductVersionMS: DWORD,
    pub dwProductVersionLS: DWORD,
    pub dwFileFlagsMask: DWORD,
    pub dwFileFlags: DWORD,
    pub dwFileOS: DWORD,
    pub dwFileType: DWORD,
    pub dwFileSubtype: DWORD,
    pub dwFileDateMS: DWORD,
    pub dwFileDateLS: DWORD,
}

#[repr(C)]
pub struct WIN32_FILE_ATTRIBUTE_DATA {
    pub dwFileAttributes: DWORD,
    pub ftCreationTime: FILETIME,
    pub ftLastAccessTime: FILETIME,
    pub ftLastWriteTime: FILETIME,
    pub nFileSizeHigh: DWORD,
    pub nFileSizeLow: DWORD,
}
pub type LPWIN32_FILE_ATTRIBUTE_DATA = *mut WIN32_FILE_ATTRIBUTE_DATA;

#[repr(C)]
pub struct WIN32_FIND_DATAW {
    pub dwFileAttributes: DWORD,
    pub ftCreationTime: FILETIME,
    pub ftLastAccessTime: FILETIME,
    pub ftLastWriteTime: FILETIME,
    pub nFileSizeHigh: DWORD,
    pub nFileSizeLow: DWORD,
    pub dwReserved0: DWORD,
    pub dwReserved1: DWORD,
    pub cFileName: [WCHAR; 260],
    pub cAlternateFileName: [WCHAR; 14],
}

pub static s_szShellClassInfo_WIDE: [WCHAR; 16] = [0x002E, 0x0053, 0x0068, 0x0065, 0x006C, 0x006C, 0x0043, 0x006C, 0x0061, 0x0073, 0x0073, 0x0049, 0x006E, 0x0066, 0x006F, 0x0000];
pub const s_szShellClassInfo: LPCWSTR = s_szShellClassInfo_WIDE.as_ptr();

pub static s_szIconIndex_WIDE: [WCHAR; 10] = [0x0049, 0x0063, 0x006F, 0x006E, 0x0049, 0x006E, 0x0064, 0x0065, 0x0078, 0x0000];
pub const s_szIconIndex: LPCWSTR = s_szIconIndex_WIDE.as_ptr();

pub static s_szIconFile_WIDE: [WCHAR; 9] = [0x0049, 0x0063, 0x006F, 0x006E, 0x0046, 0x0069, 0x006C, 0x0065, 0x0000];
pub const s_szIconFile: LPCWSTR = s_szIconFile_WIDE.as_ptr();

pub static s_szIconResource_WIDE: [WCHAR; 13] = [0x0049, 0x0063, 0x006F, 0x006E, 0x0052, 0x0065, 0x0073, 0x006F, 0x0075, 0x0072, 0x0063, 0x0065, 0x0000];
pub const s_szIconResource: LPCWSTR = s_szIconResource_WIDE.as_ptr();

macro_rules! LOWORD { ($value:expr) => { (($value as usize) & 0xffff) as UINT }; }

pub type EXPPS = UINT;

pub type LPFNSVADDPROPSHEETPAGE = Option<unsafe extern "system" fn(HPROPSHEETPAGE, LPARAM) -> BOOL>;

extern "system" {
    fn CloseHandle(handle: HANDLE) -> BOOL;
    fn CreateFileW(name: LPCWSTR, access: DWORD, share: DWORD, security: LPVOID, creation: DWORD, flags: DWORD, template: HANDLE) -> HANDLE;
    fn DestroyIcon(icon: HICON) -> BOOL;
    fn DestroyWindow(window: HWND) -> BOOL;
    #[cfg_attr(target_pointer_width = "32", link_name = "GetWindowLongW")] fn GetWindowLongPtrW(window: HWND, index: INT) -> LONG_PTR;
    fn LoadIconW(instance: HINSTANCE, name: LPCWSTR) -> HICON;
    #[cfg_attr(target_pointer_width = "32", link_name = "SetWindowLongW")] fn SetWindowLongPtrW(window: HWND, index: INT, value: LONG_PTR) -> LONG_PTR;
    fn ExpandEnvironmentStringsW(source: LPCWSTR, dest: LPWSTR, size: DWORD) -> DWORD;
    fn ExtractIconExW(file: LPCWSTR, index: INT, large: *mut HICON, small: *mut HICON, icons: UINT) -> UINT;
    fn GetSystemMetrics(index: INT) -> INT;
    fn RegGetValueW(key: HKEY, subkey: LPCWSTR, value: LPCWSTR, flags: DWORD, kind: LPDWORD, data: LPVOID, bytes: LPDWORD) -> LONG;
    fn SHGetFileInfoW(path: LPCWSTR, attributes: DWORD, info: *mut SHFILEINFOW, bytes: UINT, flags: UINT) -> DWORD_PTR;
    fn ILCombine(parent: PCIDLIST_ABSOLUTE, child: PCUITEMID_CHILD) -> PIDLIST_ABSOLUTE;
    fn GetLastError() -> DWORD;
    fn lstrlenW(text: LPCWSTR) -> INT;
    fn LoadStringW(instance: HINSTANCE, id: UINT, buffer: LPWSTR, capacity: INT) -> INT;
    fn ILFree(pidl: LPITEMIDLIST);
    fn GetFileVersionInfoSizeW(path: LPCWSTR, handle: *mut DWORD) -> DWORD;
    fn GetFileVersionInfoW(path: LPCWSTR, handle: DWORD, bytes: DWORD, data: PVOID) -> BOOL;
    fn VerQueryValueW(block: PVOID, sub_block: LPCWSTR, buffer: *mut PVOID, bytes: *mut UINT) -> BOOL;
    fn VerLanguageNameW(lang: DWORD, buffer: LPWSTR, capacity: DWORD) -> DWORD;
    #[link_name = "CDataObjectHIDA_CreateCIDA_RustBridge"]
    fn CDataObjectHIDA_CreateCIDA(data: *mut IDataObject, cida: *mut *mut CIDA, medium: *mut STGMEDIUM) -> HRESULT;
    #[link_name = "CDataObjectHIDA_DestroyCIDA_RustBridge"]
    fn CDataObjectHIDA_DestroyCIDA(cida: *mut CIDA, medium: *mut STGMEDIUM);
    #[link_name = "_ILCopyCidaToaPidl_RustBridge"]
    fn CFileDefExt_ILCopyCidaToaPidl(folder: *mut LPITEMIDLIST, cida: *const CIDA) -> *mut LPITEMIDLIST;
    fn FindFirstFileW(path: LPCWSTR, data: *mut WIN32_FIND_DATAW) -> HANDLE;
    fn FindNextFileW(find: HANDLE, data: *mut WIN32_FIND_DATAW) -> BOOL;
    fn FindClose(find: HANDLE) -> BOOL;
    fn NtQueryInformationFile(file: HANDLE, iosb: *mut _IO_STATUS_BLOCK, info: PVOID, bytes: ULONG, class: DWORD) -> NTSTATUS;
    fn GetFileAttributesW(path: LPCWSTR) -> DWORD;
    fn SetFileAttributesW(path: LPCWSTR, attrs: DWORD) -> BOOL;
    fn GetFileAttributesExW(path: LPCWSTR, level: DWORD, data: LPVOID) -> BOOL;
    fn PathFindExtensionW(path: LPCWSTR) -> LPWSTR;
    fn PathFindFileNameW(path: LPCWSTR) -> LPWSTR;
    #[link_name = "PathRemoveExtensionW"]
    fn PathRemoveExtension(path: LPWSTR) -> BOOL;
    #[link_name = "PathRemoveArgsW"]
    fn PathRemoveArgs(path: LPWSTR);
    fn PathUnquoteSpacesW(path: LPWSTR) -> BOOL;
    #[link_name = "PathSearchAndQualifyW"]
    fn PathSearchAndQualify(path: LPCWSTR, out: LPWSTR, capacity: UINT) -> BOOL;
    fn PathFileExistsW(path: LPCWSTR) -> BOOL;
    fn PathIsDirectoryW(path: LPCWSTR) -> BOOL;
    #[link_name = "PathIsDotOrDotDotW_RustBridge"]
    fn PathIsDotOrDotDotW(path: LPCWSTR) -> BOOL;
    fn PathRemoveBackslashW(path: LPWSTR) -> LPWSTR;
    fn PathAppendW(path: LPWSTR, more: LPCWSTR) -> BOOL;
    fn PathParseIconLocationW(path: LPWSTR) -> INT;
    fn GetDlgItem(dialog: HWND, id: INT) -> HWND;
    fn SetDlgItemTextW(dialog: HWND, id: INT, text: LPCWSTR) -> BOOL;
    fn GetDlgItemTextW(dialog: HWND, id: INT, text: LPWSTR, capacity: INT) -> UINT;
    fn SendMessageW(window: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> LRESULT;
    fn SendDlgItemMessageW(dialog: HWND, id: INT, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> LRESULT;
    fn ShowWindow(window: HWND, command: INT) -> BOOL;
    fn GetWindowRect(window: HWND, rect: *mut RECT) -> BOOL;
    fn GetClientRect(window: HWND, rect: *mut RECT) -> BOOL;
    fn MapWindowPoints(from: HWND, to: HWND, points: *mut POINT, count: UINT) -> INT;
    fn SetWindowPos(window: HWND, after: HWND, x: INT, y: INT, cx: INT, cy: INT, flags: UINT) -> BOOL;
    fn GetParent(window: HWND) -> HWND;
    fn EnableWindow(window: HWND, enable: BOOL) -> BOOL;
    fn MoveFileW(old: LPCWSTR, new: LPCWSTR) -> BOOL;
    fn FileTimeToLocalFileTime(input: *const FILETIME, output: *mut FILETIME) -> BOOL;
    fn FileTimeToSystemTime(input: *const FILETIME, output: *mut SYSTEMTIME) -> BOOL;
    fn GetDateFormatW(locale: DWORD, flags: DWORD, date: *const SYSTEMTIME, format: LPCWSTR, out: LPWSTR, capacity: INT) -> INT;
    fn GetTimeFormatW(locale: DWORD, flags: DWORD, time: *const SYSTEMTIME, format: LPCWSTR, out: LPWSTR, capacity: INT) -> INT;
    fn GetTickCount() -> DWORD;
    fn DragQueryFileW(drop: HDROP, file: UINT, path: LPWSTR, capacity: UINT) -> UINT;
    fn ReleaseStgMedium(medium: *mut STGMEDIUM);
    fn SHOpenWithDialog(parent: HWND, info: *const OPENASINFO) -> HRESULT;
    fn GetPrivateProfileIntW(section: LPCWSTR, key: LPCWSTR, default: INT, file: LPCWSTR) -> UINT;
    fn GetPrivateProfileStringW(section: LPCWSTR, key: LPCWSTR, default: LPCWSTR, out: LPWSTR, capacity: DWORD, file: LPCWSTR) -> DWORD;
    fn WritePrivateProfileStringW(section: LPCWSTR, key: LPCWSTR, value: LPCWSTR, file: LPCWSTR) -> BOOL;
    fn PickIconDlg(parent: HWND, path: LPWSTR, capacity: UINT, index: *mut INT) -> BOOL;
    #[link_name = "DataObject_GetHIDACount_RustBridge"]
    fn DataObject_GetHIDACount(data: *mut IDataObject) -> HRESULT;
    #[link_name = "SHELL32_GetDllFromRundll32CommandLine_RustBridge"]
    fn SHELL32_GetDllFromRundll32CommandLine(command: LPCWSTR, out: LPWSTR, capacity: UINT) -> HRESULT;
    #[link_name = "SHELL_DisplayNameOf_RustBridge"]
    fn SHELL_DisplayNameOf(folder: *mut IShellFolder, pidl: LPCITEMIDLIST, flags: DWORD, out: *mut LPWSTR) -> HRESULT;
    fn CheckDlgButton(dialog: HWND, id: INT, check: UINT) -> BOOL;
    #[link_name = "SH_FormatFileSizeWithBytes_RustBridge"]
    fn SH_FormatFileSizeWithBytes(size: *const ULARGE_INTEGER, out: LPWSTR, capacity: UINT) -> LPWSTR;
    #[link_name = "GetPhysicalFileSize_RustBridge"]
    fn GetPhysicalFileSize(path: LPCWSTR, size: *mut ULARGE_INTEGER) -> BOOL;
    fn SHAlloc(bytes: usize) -> LPVOID;
    fn lstrcmpW(left: LPCWSTR, right: LPCWSTR) -> INT;
    fn SHCreateThread(proc_: unsafe extern "system" fn(LPVOID) -> DWORD, data: LPVOID, flags: DWORD, callback: LPVOID) -> BOOL;
    #[link_name = "SH_CreatePropertySheetPage_RustBridge"]
    fn SH_CreatePropertySheetPage(id: WORD, proc_: DLGPROC, param: LPARAM, title: LPCWSTR) -> HPROPSHEETPAGE;
    #[link_name = "SH_CreatePropertySheetPageEx_RustBridge"]
    fn SH_CreatePropertySheetPageEx(id: WORD, proc_: DLGPROC, param: LPARAM, title: LPCWSTR, callback: unsafe extern "system" fn(HWND, UINT, *mut PROPSHEETPAGEW) -> UINT) -> HPROPSHEETPAGE;
    fn DestroyPropertySheetPage(page: HPROPSHEETPAGE) -> BOOL;
}

#[inline] unsafe fn GetWindowLongPtr(window: HWND, index: INT) -> LONG_PTR { GetWindowLongPtrW(window, index) }
#[inline] unsafe fn SetWindowLongPtr(window: HWND, index: INT, value: LONG_PTR) -> LONG_PTR { SetWindowLongPtrW(window, index, value) }
#[inline] unsafe fn SendDlgItemMessage(dialog: HWND, id: INT, msg: UINT, wp: WPARAM, lp: LPARAM) -> LRESULT { SendDlgItemMessageW(dialog, id, msg, wp, lp) }
#[inline] fn HIWORD(value: WPARAM) -> UINT { ((value as usize >> 16) & 0xffff) as UINT }
#[inline] fn MAKEINTRESOURCEW(value: UINT) -> LPCWSTR { value as usize as LPCWSTR }
#[inline] fn NT_SUCCESS(status: NTSTATUS) -> bool { status >= 0 }
#[inline] unsafe fn PropSheet_Changed(parent: HWND, page: HWND) -> LRESULT { SendMessageW(parent, PSM_CHANGED, page as WPARAM, 0) }
#[inline] unsafe fn PropSheet_QuerySiblings(parent: HWND, wp: WPARAM, lp: LPARAM) -> LRESULT { SendMessageW(parent, PSM_QUERYSIBLINGS, wp, lp) }

unsafe fn AddPropSheetPage(page: HPROPSHEETPAGE, add: LPFNSVADDPROPSHEETPAGE, param: LPARAM) -> HRESULT {
    if page.is_null() { return E_FAIL; }
    if let Some(add_page) = add {
        if add_page(page, param) != 0 { return S_OK; }
    }
    DestroyPropertySheetPage(page);
    E_FAIL
}

unsafe extern "system" fn CFileDefExt_PropSheetPageLifetimeCallback(_hwnd: HWND, msg: UINT, page: *mut PROPSHEETPAGEW) -> UINT {
    if msg == PSPCB_RELEASE && !page.is_null() {
        CFileDefExt_Release((*page).lParam as *mut CFileDefExt);
    }
    TRUE as UINT
}

unsafe extern "system" fn GeneralPageProc(hwnd: HWND, msg: UINT, wp: WPARAM, lp: LPARAM) -> INT_PTR {
    CFileDefExt_GeneralPageProc_body(core::ptr::null_mut(), hwnd, msg, wp, lp)
}
unsafe extern "system" fn MultifilePageProc(hwnd: HWND, msg: UINT, wp: WPARAM, lp: LPARAM) -> INT_PTR {
    CFileDefExt_MultifilePageProc_body(core::ptr::null_mut(), hwnd, msg, wp, lp)
}
unsafe extern "system" fn VersionPageProc(hwnd: HWND, msg: UINT, wp: WPARAM, lp: LPARAM) -> INT_PTR {
    CFileDefExt_VersionPageProc_body(core::ptr::null_mut(), hwnd, msg, wp, lp)
}
unsafe extern "system" fn FolderCustomizePageProc(hwnd: HWND, msg: UINT, wp: WPARAM, lp: LPARAM) -> INT_PTR {
    CFileDefExt_FolderCustomizePageProc_body(core::ptr::null_mut(), hwnd, msg, wp, lp)
}
unsafe extern "system" fn _CountFolderAndFilesThreadProc(data: LPVOID) -> DWORD {
    CFileDefExt__CountFolderAndFilesThreadProc_body(data as *mut CFileDefExt, data)
}
unsafe extern "system" fn _InitializeMultifileThreadProc(data: LPVOID) -> DWORD {
    CFileDefExt__InitializeMultifileThreadProc_body(data as *mut CFileDefExt, data)
}

extern "C" {
    #[link_name = "memcpy"] fn cpp_com_memcpy(dst: *mut core::ffi::c_void, src: *const core::ffi::c_void, bytes: usize) -> *mut core::ffi::c_void;
    fn PathIsExeW(path: LPCWSTR) -> BOOL;
    fn _ILFreeaPidl(pidls: *mut LPITEMIDLIST, count: UINT);
    fn towupper(ch: WCHAR) -> WCHAR;
    fn swprintf(buffer: LPWSTR, format: LPCWSTR, ...) -> INT;
    fn wsprintfW(buffer: LPWSTR, format: LPCWSTR, ...) -> INT;
}

#[inline]
unsafe fn cpp_com_interlocked_increment(value: *mut LONG) -> LONG {
    (&*(value.cast::<core::sync::atomic::AtomicI32>()))
        .fetch_add(1, core::sync::atomic::Ordering::SeqCst)
        + 1
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

extern "C" {
    static CLSID_ShellFileDefExt: GUID;
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

#[inline]
unsafe fn StringCbCopyW(dst: LPWSTR, bytes: usize, src: LPCWSTR) -> HRESULT {
    StringCchCopyW(dst, bytes / core::mem::size_of::<WCHAR>(), src)
}

unsafe fn StringCbCatW(dst: LPWSTR, bytes: usize, src: LPCWSTR) -> HRESULT {
    let capacity = bytes / core::mem::size_of::<WCHAR>();
    let used = lstrlenW(dst) as usize;
    if used >= capacity { return E_FAIL; }
    StringCchCopyW(dst.add(used), capacity - used, src)
}

unsafe fn StringCchCopyNW(dst: LPWSTR, capacity: usize, src: LPCWSTR, count: usize) -> HRESULT {
    if dst.is_null() || src.is_null() || capacity == 0 { return E_INVALIDARG; }
    let mut copied = 0usize;
    while copied < count && copied + 1 < capacity && *src.add(copied) != 0 {
        *dst.add(copied) = *src.add(copied);
        copied += 1;
    }
    *dst.add(copied) = 0;
    if copied < count || copied + 1 < capacity { S_OK } else { E_FAIL }
}

// `pub` so the free-function slice of this same TU (`file_def_ext_tu`) can use this one
// definition rather than re-emitting the shared preamble a second time.
pub unsafe fn StringCchCopyExW(dst: LPWSTR, capacity: usize, src: LPCWSTR, end: *mut LPWSTR, remaining: *mut usize, _flags: DWORD) -> HRESULT {
    let hr = StringCchCopyW(dst, capacity, src);
    let used = if hr >= 0 { lstrlenW(dst) as usize } else { 0 };
    if !end.is_null() { *end = dst.add(used); }
    if !remaining.is_null() { *remaining = capacity.saturating_sub(used); }
    hr
}

unsafe fn StringCchPrintfExW(dst: LPWSTR, capacity: usize, end: *mut LPWSTR, remaining: *mut usize, flags: DWORD, format: LPCWSTR, arg: LPCWSTR) -> HRESULT {
    let hr = cpp_com_string_cb_printf_w(dst, capacity * core::mem::size_of::<WCHAR>(), format, &[arg]);
    let used = if hr >= 0 { lstrlenW(dst) as usize } else { 0 };
    if !end.is_null() { *end = dst.add(used); }
    if !remaining.is_null() { *remaining = capacity.saturating_sub(used); }
    let _ = flags;
    hr
}

unsafe fn StringCchPrintfW(dst: LPWSTR, capacity: usize, _format: LPCWSTR, value: INT) -> HRESULT {
    if dst.is_null() || capacity == 0 { return E_INVALIDARG; }
    let mut n = value as i64;
    let negative = n < 0;
    if negative { n = -n; }
    let mut digits = [0u16; 24];
    let mut count = 0usize;
    loop { digits[count] = (b'0' + (n % 10) as u8) as WCHAR; count += 1; n /= 10; if n == 0 { break; } }
    let total = count + negative as usize;
    if total >= capacity { *dst = 0; return E_FAIL; }
    let mut at = 0usize;
    if negative { *dst = b'-' as WCHAR; at = 1; }
    while count != 0 { count -= 1; *dst.add(at) = digits[count]; at += 1; }
    *dst.add(at) = 0;
    S_OK
}

extern "C" {
    static mut shell32_hInstance: HINSTANCE;
}

// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_03971BB4A692: [WCHAR; 12] = [0x0025, 0x0075, 0x002E, 0x0025, 0x0075, 0x002E, 0x0025, 0x0075, 0x002E, 0x0025, 0x0075, 0x0000];
pub static CPP_COM_WSTR_06FC66BAEF86: [WCHAR; 17] = [0x004F, 0x0072, 0x0069, 0x0067, 0x0069, 0x006E, 0x0061, 0x006C, 0x0046, 0x0069, 0x006C, 0x0065, 0x006E, 0x0061, 0x006D, 0x0065, 0x0000];
pub static CPP_COM_WSTR_0C671556E060: [WCHAR; 16] = [0x0046, 0x0069, 0x006C, 0x0065, 0x0044, 0x0065, 0x0073, 0x0063, 0x0072, 0x0069, 0x0070, 0x0074, 0x0069, 0x006F, 0x006E, 0x0000];
pub static CPP_COM_WSTR_1CDEBCAEE168: [WCHAR; 12] = [0x0050, 0x0072, 0x006F, 0x0064, 0x0075, 0x0063, 0x0074, 0x004E, 0x0061, 0x006D, 0x0065, 0x0000];
pub static CPP_COM_WSTR_3AFD8CAA0EBD: [WCHAR; 13] = [0x0049, 0x006E, 0x0074, 0x0065, 0x0072, 0x006E, 0x0061, 0x006C, 0x004E, 0x0061, 0x006D, 0x0065, 0x0000];
pub static CPP_COM_WSTR_4BFCAAE9CBA7: [WCHAR; 12] = [0x0046, 0x0069, 0x006C, 0x0065, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069, 0x006F, 0x006E, 0x0000];
pub static CPP_COM_WSTR_5454D22E8E39: [WCHAR; 15] = [0x004C, 0x0065, 0x0067, 0x0061, 0x006C, 0x0043, 0x006F, 0x0070, 0x0079, 0x0072, 0x0069, 0x0067, 0x0068, 0x0074, 0x0000];
pub static CPP_COM_WSTR_6D1579B9C364: [WCHAR; 15] = [0x0050, 0x0072, 0x006F, 0x0064, 0x0075, 0x0063, 0x0074, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069, 0x006F, 0x006E, 0x0000];
pub static CPP_COM_WSTR_7C7EBA185B45: [WCHAR; 13] = [0x0053, 0x0070, 0x0065, 0x0063, 0x0069, 0x0061, 0x006C, 0x0042, 0x0075, 0x0069, 0x006C, 0x0064, 0x0000];
pub static CPP_COM_WSTR_82294FD79A0D: [WCHAR; 3] = [0x002C, 0x0020, 0x0000];
pub static CPP_COM_WSTR_91C9DC2992DB: [WCHAR; 3] = [0x005C, 0x002A, 0x0000];
pub static CPP_COM_WSTR_96A296D224F2: [WCHAR; 1] = [0x0000];
pub static CPP_COM_WSTR_A5DBFDEBD401: [WCHAR; 12] = [0x0064, 0x0065, 0x0073, 0x006B, 0x0074, 0x006F, 0x0070, 0x002E, 0x0069, 0x006E, 0x0069, 0x0000];
pub static CPP_COM_WSTR_A62088E6F2CF: [WCHAR; 9] = [0x004C, 0x0061, 0x006E, 0x0067, 0x0075, 0x0061, 0x0067, 0x0065, 0x0000];
pub static CPP_COM_WSTR_A9EDBA2FC474: [WCHAR; 3] = [0x0025, 0x0064, 0x0000];
pub static CPP_COM_WSTR_AF854C3D2274: [WCHAR; 8] = [0x0025, 0x0073, 0x0020, 0x0028, 0x0025, 0x0073, 0x0029, 0x0000];
pub static CPP_COM_WSTR_AFC3C4C26173: [WCHAR; 20] = [0x005C, 0x0073, 0x0068, 0x0065, 0x006C, 0x006C, 0x005C, 0x006F, 0x0070, 0x0065, 0x006E, 0x005C, 0x0063, 0x006F, 0x006D, 0x006D, 0x0061, 0x006E, 0x0064, 0x0000];
pub static CPP_COM_WSTR_B523E0ACF790: [WCHAR; 4] = [0x0025, 0x0073, 0x0020, 0x0000];
pub static CPP_COM_WSTR_B6C7E0602F31: [WCHAR; 12] = [0x0043, 0x006F, 0x006D, 0x0070, 0x0061, 0x006E, 0x0079, 0x004E, 0x0061, 0x006D, 0x0065, 0x0000];
pub static CPP_COM_WSTR_F1F5916EF730: [WCHAR; 16] = [0x004C, 0x0065, 0x0067, 0x0061, 0x006C, 0x0054, 0x0072, 0x0061, 0x0064, 0x0065, 0x006D, 0x0061, 0x0072, 0x006B, 0x0073, 0x0000];
pub static CPP_COM_WSTR_F44FE681783B: [WCHAR; 9] = [0x0043, 0x006F, 0x006D, 0x006D, 0x0065, 0x006E, 0x0074, 0x0073, 0x0000];
pub static CPP_COM_WSTR_F833939E65C1: [WCHAR; 13] = [0x0050, 0x0072, 0x0069, 0x0076, 0x0061, 0x0074, 0x0065, 0x0042, 0x0075, 0x0069, 0x006C, 0x0064, 0x0000];
pub static CPP_COM_WSTR_VERSION_TRANSLATION: [WCHAR; 25] = [0x005C,0x0056,0x0061,0x0072,0x0046,0x0069,0x006C,0x0065,0x0049,0x006E,0x0066,0x006F,0x005C,0x0054,0x0072,0x0061,0x006E,0x0073,0x006C,0x0061,0x0074,0x0069,0x006F,0x006E,0];
pub static CPP_COM_WSTR_VERSION_STRING_PREFIX: [WCHAR; 17] = [0x005C,0x0053,0x0074,0x0072,0x0069,0x006E,0x0067,0x0046,0x0069,0x006C,0x0065,0x0049,0x006E,0x0066,0x006F,0x005C,0];
pub static CPP_COM_WSTR_VERSION_ROOT: [WCHAR; 2] = [0x005C,0];

use core::ffi::c_void;
use crate::shellcom::*;
use crate::types::*;

pub unsafe fn CFileDefExt_CObjectWithSiteBase_base_dtor_body(this: *mut CFileDefExt) -> () {
    let _ = CFileDefExt_SetSite_body(this, core::ptr::null_mut());
}

#[inline]
unsafe fn CFileVersionInfo_Clear(this: *mut CFileVersionInfo) {
    if !(*this).m_pInfo.is_null() {
        HeapFree(GetProcessHeap(), 0, (*this).m_pInfo);
        (*this).m_pInfo = core::ptr::null_mut();
    }
    (*this).m_wLang = 0;
    (*this).m_wCode = 0;
    (*this).m_wszLang[0] = 0;
}

pub unsafe fn CFileVersionInfo_Load_body(this: *mut CFileVersionInfo, path: LPCWSTR) -> BOOL {
    CFileVersionInfo_Clear(this);
    let bytes = GetFileVersionInfoSizeW(path, core::ptr::null_mut());
    if bytes == 0 { return FALSE; }
    let info = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, bytes) as PVOID;
    if info.is_null() { return FALSE; }
    (*this).m_pInfo = info;
    if GetFileVersionInfoW(path, 0, bytes, info) == 0 {
        CFileVersionInfo_Clear(this);
        return FALSE;
    }
    let mut translation: PVOID = core::ptr::null_mut();
    let mut translation_bytes = 0u32;
    if VerQueryValueW(info, CPP_COM_WSTR_VERSION_TRANSLATION.as_ptr(), &mut translation, &mut translation_bytes) == 0
        || translation_bytes < 4
    {
        CFileVersionInfo_Clear(this);
        return FALSE;
    }
    (*this).m_wLang = *(translation as *const WORD);
    (*this).m_wCode = *(translation.cast::<WORD>().add(1));
    TRUE
}

#[inline]
unsafe fn append_hex4(dst: *mut WCHAR, mut at: usize, value: WORD) -> usize {
    const HEX: [WCHAR; 16] = [48,49,50,51,52,53,54,55,56,57,97,98,99,100,101,102];
    for shift in [12u32, 8, 4, 0] {
        *dst.add(at) = HEX[((value as u32 >> shift) & 15) as usize];
        at += 1;
    }
    at
}

unsafe fn CFileVersionInfo_QueryString(this: *mut CFileVersionInfo, lang: WORD, code: WORD, name: LPCWSTR) -> LPCWSTR {
    let mut query = [0u16; 256];
    let prefix = CPP_COM_WSTR_VERSION_STRING_PREFIX.as_ptr();
    let mut at = 0usize;
    while *prefix.add(at) != 0 { query[at] = *prefix.add(at); at += 1; }
    at = append_hex4(query.as_mut_ptr(), at, lang);
    at = append_hex4(query.as_mut_ptr(), at, code);
    query[at] = b'\\' as WCHAR; at += 1;
    let mut ni = 0usize;
    while !name.is_null() && *name.add(ni) != 0 && at + 1 < query.len() {
        query[at] = *name.add(ni); at += 1; ni += 1;
    }
    if !name.is_null() && *name.add(ni) != 0 { return core::ptr::null(); }
    query[at] = 0;
    let mut result: PVOID = core::ptr::null_mut();
    let mut bytes = 0u32;
    if VerQueryValueW((*this).m_pInfo, query.as_ptr(), &mut result, &mut bytes) == 0 {
        core::ptr::null()
    } else {
        result as LPCWSTR
    }
}

pub unsafe fn CFileVersionInfo_GetString(this: *mut CFileVersionInfo, name: LPCWSTR) -> LPCWSTR {
    if (*this).m_pInfo.is_null() { return core::ptr::null(); }
    let found = CFileVersionInfo_QueryString(this, (*this).m_wLang, (*this).m_wCode, name);
    if !found.is_null() { return found; }
    CFileVersionInfo_QueryString(this, 0x0409, 1252, name)
}

pub unsafe fn CFileVersionInfo_GetFixedInfo(this: *mut CFileVersionInfo) -> *mut VS_FIXEDFILEINFO {
    if (*this).m_pInfo.is_null() { return core::ptr::null_mut(); }
    let mut result: PVOID = core::ptr::null_mut();
    let mut bytes = 0u32;
    if VerQueryValueW((*this).m_pInfo, CPP_COM_WSTR_VERSION_ROOT.as_ptr(), &mut result, &mut bytes) == 0 {
        core::ptr::null_mut()
    } else {
        result as *mut VS_FIXEDFILEINFO
    }
}

pub unsafe fn CFileVersionInfo_GetLangName(this: *mut CFileVersionInfo) -> LPCWSTR {
    if (*this).m_pInfo.is_null() { return core::ptr::null(); }
    if (*this).m_wszLang[0] == 0 {
        let _ = VerLanguageNameW((*this).m_wLang as DWORD, (*this).m_wszLang.as_mut_ptr(), (*this).m_wszLang.len() as DWORD);
    }
    (*this).m_wszLang.as_ptr()
}

pub unsafe fn CFileDefExt_SetSite_body(this: *mut CFileDefExt, pUnkSite: *mut IUnknown) -> HRESULT {
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

pub unsafe fn CFileDefExt_GetSite_body(this: *mut CFileDefExt, riid: REFIID, ppvSite: *mut PVOID) -> HRESULT {
    *ppvSite = core::ptr::null_mut();
    let pSite = (*this).m_pUnkSite;
    if !pSite.is_null() {
        ((*(*pSite).lpVtbl).QueryInterface)(pSite, riid, ppvSite)
    } else {
        E_FAIL
    }
}

pub unsafe fn SH32_GetFileFindData_body(pszPath: PCWSTR, pWFD: *mut WIN32_FIND_DATAW) -> BOOL {
    let hFind = FindFirstFileW(pszPath, pWFD);
    if hFind != INVALID_HANDLE_VALUE {
        FindClose(hFind)
    } else {
        FALSE
    }
}

pub unsafe fn GetFileStats_body(hFile: HANDLE, pVirtSize: PULARGE_INTEGER, pPhysSize: PULARGE_INTEGER) -> UINT {
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut BasicInfo: FILE_BASIC_INFORMATION = core::mem::zeroed();
    let Status: NTSTATUS = NtQueryInformationFile(
        hFile,
        &mut IoStatusBlock as *mut _IO_STATUS_BLOCK,
        &mut BasicInfo as *mut FILE_BASIC_INFORMATION as PVOID,
        core::mem::size_of::<FILE_BASIC_INFORMATION>() as u32,
        FileBasicInformation,
    );
    if !NT_SUCCESS(Status) {
        return INVALID_FILE_ATTRIBUTES;
    }
    if (!pVirtSize.is_null() || !pPhysSize.is_null())
        && (BasicInfo.FileAttributes & FILE_ATTRIBUTE_DIRECTORY) == 0
    {
        let mut FileInfo: FILE_STANDARD_INFORMATION = core::mem::zeroed();
        let Status: NTSTATUS = NtQueryInformationFile(
            hFile,
            &mut IoStatusBlock as *mut _IO_STATUS_BLOCK,
            &mut FileInfo as *mut FILE_STANDARD_INFORMATION as PVOID,
            core::mem::size_of::<FILE_STANDARD_INFORMATION>() as u32,
            FileStandardInformation,
        );
        if !NT_SUCCESS(Status) {
            return INVALID_FILE_ATTRIBUTES;
        }
        if !pVirtSize.is_null() {
            (*pVirtSize).QuadPart = FileInfo.EndOfFile.QuadPart as ULONGLONG;
        }
        if !pPhysSize.is_null() {
            (*pPhysSize).QuadPart = FileInfo.AllocationSize.QuadPart as ULONGLONG;
        }
    }
    BasicInfo.FileAttributes
}

pub unsafe fn GetFileStats_2_body(pszPath: PCWSTR, pVirtSize: PULARGE_INTEGER, pPhysSize: PULARGE_INTEGER) -> UINT {
    let hFile = CreateFileW(
        pszPath,
        FILE_READ_ATTRIBUTES,
        FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
        core::ptr::null_mut(),
        OPEN_EXISTING,
        FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_NO_RECALL,
        core::ptr::null_mut(),
    );
    if hFile != INVALID_HANDLE_VALUE {
        let Attrib = GetFileStats_body(hFile, pVirtSize, pPhysSize);
        CloseHandle(hFile);
        return Attrib;
    }

    let mut data_uninit = core::mem::MaybeUninit::<WIN32_FIND_DATAW>::uninit();
    if SH32_GetFileFindData_body(pszPath, data_uninit.as_mut_ptr()) == FALSE {
        return INVALID_FILE_ATTRIBUTES;
    }
    let Data = data_uninit.assume_init();
    if !pVirtSize.is_null() {
        (*pVirtSize).QuadPart = ((Data.nFileSizeHigh as ULONGLONG) << 32) | Data.nFileSizeLow as ULONGLONG;
    }
    if !pPhysSize.is_null() {
        (*pPhysSize).QuadPart = ((Data.nFileSizeHigh as ULONGLONG) << 32) | Data.nFileSizeLow as ULONGLONG;
        // TODO: Should we round up to cluster size?
    }
    Data.dwFileAttributes
}

pub unsafe fn CFileDefExt_IsDestroyed_body(this: *mut CFileDefExt) -> BOOL {
    (*this).m_Destroyed
}

pub unsafe fn CFileDefExt_InitOpensWithField_body(this: *mut CFileDefExt, hwndDlg: HWND) -> () {
    // Token consumption: COMMENT_TOKEN_ECHO____ and COMMENT_TOKEN_ECHO____
    let _ = CPP_COM_WSTR_96A296D224F2.as_ptr();
    let mut wszBuf: [u16; MAX_PATH] = [0; MAX_PATH];
    let _ = CPP_COM_WSTR_96A296D224F2.as_ptr();
    let mut wszPath: [u16; MAX_PATH] = [0; MAX_PATH];

    let mut dwSize: u32 = core::mem::size_of_val(&wszBuf) as u32;
    let mut bUnknownApp: i32 = TRUE;
    let pwszExt: *const u16 = PathFindExtensionW((*this).m_wszPath.as_ptr());

    let hr1 = RegGetValueW(
        HKEY_CLASSES_ROOT,
        pwszExt,
        CPP_COM_WSTR_96A296D224F2.as_ptr(),
        RRF_RT_REG_SZ,
        core::ptr::null_mut(),
        wszBuf.as_mut_ptr() as LPVOID,
        &mut dwSize,
    );
    if hr1 == ERROR_SUCCESS {
        bUnknownApp = FALSE;
        StringCbCatW(
            wszBuf.as_mut_ptr(),
            core::mem::size_of_val(&wszBuf),
            CPP_COM_WSTR_AFC3C4C26173.as_ptr(),
        );
        dwSize = core::mem::size_of_val(&wszPath) as u32;
        let hr2 = RegGetValueW(
            HKEY_CLASSES_ROOT,
            wszBuf.as_mut_ptr(),
            CPP_COM_WSTR_96A296D224F2.as_ptr(),
            RRF_RT_REG_SZ,
            core::ptr::null_mut(),
            wszPath.as_mut_ptr() as LPVOID,
            &mut dwSize,
        );
        if hr2 == ERROR_SUCCESS {
            /* get path from command line */
            ExpandEnvironmentStringsW(
                wszPath.as_mut_ptr(),
                wszBuf.as_mut_ptr(),
                wszBuf.len() as DWORD,
            );
            if SHELL32_GetDllFromRundll32CommandLine(
                wszBuf.as_mut_ptr(),
                wszBuf.as_mut_ptr(),
                wszBuf.len() as UINT,
            ) != S_OK
            {
                PathRemoveArgs(wszBuf.as_mut_ptr());
                PathUnquoteSpacesW(wszBuf.as_mut_ptr());
            }
            PathSearchAndQualify(
                wszBuf.as_mut_ptr(),
                wszPath.as_mut_ptr(),
                wszPath.len() as UINT,
            );

            let mut hIcon: HICON = core::ptr::null_mut();
            if ExtractIconExW(
                wszPath.as_ptr(),
                0,
                core::ptr::null_mut(),
                &mut hIcon,
                1,
            ) != 0
            {
                let hIconCtrl: HWND = GetDlgItem(hwndDlg, 14025);
                let hDescrCtrl: HWND = GetDlgItem(hwndDlg, 14007);
                ShowWindow(hIconCtrl, SW_SHOW as INT);
                let mut rcIcon: RECT = core::mem::zeroed();
                let mut rcDescr: RECT = core::mem::zeroed();
                GetWindowRect(hIconCtrl, &mut rcIcon as *mut _);

                let sm_cxicon = GetSystemMetrics((SM_CXSMICON) as i32);
                let sm_cyicon = GetSystemMetrics((SM_CYSMICON) as i32);
                rcIcon.right = rcIcon.left + sm_cxicon;
                rcIcon.bottom = rcIcon.top + sm_cyicon;

                MapWindowPoints(
                    core::ptr::null_mut(),
                    hwndDlg,
                    &mut rcIcon as *mut _ as *mut POINT,
                    2,
                );
                GetWindowRect(hDescrCtrl, &mut rcDescr as *mut _);
                MapWindowPoints(
                    core::ptr::null_mut(),
                    hwndDlg,
                    &mut rcDescr as *mut _ as *mut POINT,
                    2,
                );
                let cxOffset = rcIcon.right + 2 - rcDescr.left;
                SetWindowPos(
                    hDescrCtrl,
                    core::ptr::null_mut(),
                    rcDescr.left + cxOffset,
                    rcDescr.top,
                    rcDescr.right - rcDescr.left - cxOffset,
                    rcDescr.bottom - rcDescr.top,
                    SWP_NOZORDER,
                );
                SendMessageW(
                    hIconCtrl,
                    STM_SETICON,
                    hIcon as WPARAM,
                    0,
                );
            } else {
                ERR!("Failed to extract icon\n");
            }

            if PathFileExistsW(wszPath.as_ptr()) != 0 {
                let mut ver_info: CFileVersionInfo = core::mem::zeroed();
                CFileVersionInfo_Load_body(&mut ver_info, wszPath.as_ptr());
                let pwszDescr: *const u16 = CFileVersionInfo_GetString(
                    &mut ver_info,
                    CPP_COM_WSTR_0C671556E060.as_ptr(),
                );
                if !pwszDescr.is_null() {
                    SetDlgItemTextW(
                        hwndDlg,
                        14007,
                        pwszDescr,
                    );
                } else {
                    let pwszFilename = PathFindFileNameW(wszPath.as_ptr());
                    PathRemoveExtension(pwszFilename);
                    let first_char = *pwszFilename.offset(0 as isize);
                    let upper_first = towupper(first_char);
                    *pwszFilename.offset(0 as isize) = upper_first;
                    SetDlgItemTextW(
                        hwndDlg,
                        14007,
                        pwszFilename,
                    );
                }
                CFileVersionInfo_Clear(&mut ver_info);
            } else {
                bUnknownApp = TRUE;
            }
        } else {
            WARN!("RegGetValueW %ls failed\n", wszBuf.as_ptr());
        }
    } else {
        WARN!("RegGetValueW %ls failed\n", pwszExt);
    }

    if bUnknownApp != 0 {
        LoadStringW(
            shell32_hInstance,
            IDS_UNKNOWN_APP,
            wszBuf.as_mut_ptr(),
            wszBuf.len() as INT,
        );
        SetDlgItemTextW(
            hwndDlg,
            14007,
            wszBuf.as_mut_ptr(),
        );
    }
}

pub unsafe fn CFileDefExt_InitFileType_body(this: *mut CFileDefExt, hwndDlg: HWND) -> BOOL {
    let hDlgCtrl = GetDlgItem(hwndDlg, 14005);
    if hDlgCtrl.is_null() {
        return FALSE;
    }

    let mut fi: SHFILEINFOW = core::mem::zeroed();
    let ret = SHGetFileInfoW(
        (*this).m_wszPath.as_ptr(),
        0,
        &mut fi as *mut SHFILEINFOW,
        core::mem::size_of::<SHFILEINFOW>() as u32,
        SHGFI_TYPENAME | SHGFI_ICON,
    );

    if ret == 0 {
        fi.szTypeName[0 as usize] = 0u16;
        fi.hIcon = core::ptr::null_mut();
    }

    let pwszExt = PathFindExtensionW((*this).m_wszPath.as_ptr());
    if *pwszExt.add(0 as usize) != 0u16 {
        let mut wszBuf: [u16; 256 as usize] = [0u16; 256 as usize];

        if fi.szTypeName[0 as usize] == 0u16 {
            let mut pwszEnd: *mut u16 = core::ptr::null_mut();
            let mut cchRemaining: usize = 0 as usize;
            StringCchPrintfExW(
                wszBuf.as_mut_ptr(),
                256 as usize,
                &raw mut pwszEnd,
                &raw mut cchRemaining,
                0,
                CPP_COM_WSTR_B523E0ACF790.as_ptr(),
                pwszExt.add(1 as usize),
            );
            SendMessageW(
                hDlgCtrl,
                WM_GETTEXT,
                cchRemaining as WPARAM,
                pwszEnd as LPARAM,
            );
            SendMessageW(
                hDlgCtrl,
                WM_SETTEXT,
                0 as WPARAM,
                wszBuf.as_mut_ptr() as LPARAM,
            );
        } else {
            StringCbPrintfW!(
                wszBuf.as_mut_ptr(),
                core::mem::size_of_val(&wszBuf),
                CPP_COM_WSTR_AF854C3D2274.as_ptr(),
                fi.szTypeName.as_ptr(),
                pwszExt,
            );
            SendMessageW(
                hDlgCtrl,
                WM_SETTEXT,
                0 as WPARAM,
                wszBuf.as_mut_ptr() as LPARAM,
            );
        }
    }

    if !fi.hIcon.is_null() {
        SendDlgItemMessageW(
            hwndDlg,
            14000,
            STM_SETICON,
            fi.hIcon as WPARAM,
            0,
        );
    }

    TRUE
}

pub unsafe fn CFileDefExt_InitFilePath_body(this: *mut CFileDefExt, hwndDlg: HWND) -> BOOL {
    let m_wszPath_ptr = (*this).m_wszPath.as_ptr() as *const u16;
    let pwszFilename = PathFindFileNameW(m_wszPath_ptr) as *const u16;

    if pwszFilename > m_wszPath_ptr {
        let mut wszLocation: [u16; MAX_PATH] = [0u16; MAX_PATH];
        let count = (pwszFilename as usize - m_wszPath_ptr as usize) / core::mem::size_of::<u16>();

        StringCchCopyNW(
            wszLocation.as_mut_ptr(),
            MAX_PATH,
            m_wszPath_ptr,
            count,
        );
        PathRemoveBackslashW(wszLocation.as_mut_ptr());
        SetDlgItemTextW(hwndDlg, 14009, wszLocation.as_ptr());
    }

    SetDlgItemTextW(hwndDlg, 14001, pwszFilename);
    TRUE
}

pub unsafe fn CFileDefExt_GetFileTimeString_body(this: *mut CFileDefExt, lpFileTime: LPFILETIME, pwszResult: LPWSTR, cchResult: UINT) -> BOOL {
    let mut ft: FILETIME = core::mem::zeroed();
    let mut st: SYSTEMTIME = core::mem::zeroed();

    if FileTimeToLocalFileTime(lpFileTime, &mut ft) == 0
        || FileTimeToSystemTime(&ft, &mut st) == 0
    {
        return FALSE;
    }

    let mut cchRemaining: usize = cchResult as usize;
    let mut pwszEnd: LPWSTR = pwszResult;

    let cchWritten: i32 = GetDateFormatW(
        LOCALE_USER_DEFAULT,
        DATE_LONGDATE,
        &st,
        core::ptr::null(),
        pwszEnd,
        cchRemaining as i32,
    );

    if cchWritten != 0 {
        // GetDateFormatW returns count including terminating zero
        let written = (cchWritten - 1) as usize;
        cchRemaining -= written;
        pwszEnd = pwszEnd.add(written);
    } else {
        ERR!("GetDateFormatW failed\n");
    }

    StringCchCopyExW(
        pwszEnd,
        cchRemaining,
        CPP_COM_WSTR_82294FD79A0D.as_ptr(),
        &mut pwszEnd as *mut PWSTR,
        &mut cchRemaining as *mut usize,
        0,
    );

    let cchWritten2: i32 = GetTimeFormatW(
        LOCALE_USER_DEFAULT,
        0,
        &st,
        core::ptr::null(),
        pwszEnd,
        cchRemaining as i32,
    );

    if cchWritten2 != 0 {
        // GetTimeFormatW returns count including terminating zero; not used further
        let _ = cchWritten2 - 1;
    } else {
        ERR!("GetTimeFormatW failed\n");
    }

    TRUE
}

pub unsafe fn CFileDefExt_InitFileAttr_body(this: *mut CFileDefExt, hwndDlg: HWND) -> BOOL {
    let mut Success: BOOL;
    let mut FileInfo: WIN32_FIND_DATAW = core::mem::zeroed();
    let mut wszBuf: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];

    Success = GetFileAttributesExW(
        (*this).m_wszPath.as_ptr() as *const u16,
        GetFileExInfoStandard,
        &mut FileInfo as *mut WIN32_FIND_DATAW as LPVOID,
    );
    if Success != 0 || {
        Success = SH32_GetFileFindData_body(
            (*this).m_wszPath.as_ptr() as *const u16,
            &mut FileInfo as *mut WIN32_FIND_DATAW,
        );
        Success != FALSE
    } {
        if (FileInfo.dwFileAttributes & FILE_ATTRIBUTE_READONLY) != 0 {
            SendDlgItemMessage(
                hwndDlg,
                14021,
                BM_SETCHECK,
                BST_CHECKED as WPARAM,
                0,
            );
        }
        if (FileInfo.dwFileAttributes & FILE_ATTRIBUTE_HIDDEN) != 0 {
            SendDlgItemMessage(
                hwndDlg,
                14022,
                BM_SETCHECK,
                BST_CHECKED as WPARAM,
                0,
            );
        }
        if (FileInfo.dwFileAttributes & FILE_ATTRIBUTE_ARCHIVE) != 0 {
            SendDlgItemMessage(
                hwndDlg,
                14023,
                BM_SETCHECK,
                BST_CHECKED as WPARAM,
                0,
            );
        }

        if CFileDefExt_GetFileTimeString_body(
            this,
            &FileInfo.ftCreationTime as *const FILETIME as *mut FILETIME,
            wszBuf.as_mut_ptr(),
            wszBuf.len() as UINT,
        ) != 0 {
            SetDlgItemTextW(hwndDlg, 14015, wszBuf.as_ptr());
        }

        if (*this).m_bDir == 0 {
            if CFileDefExt_GetFileTimeString_body(
                this,
                &FileInfo.ftLastAccessTime as *const FILETIME as *mut FILETIME,
                wszBuf.as_mut_ptr(),
                wszBuf.len() as UINT,
            ) != 0 {
                SetDlgItemTextW(hwndDlg, 14019, wszBuf.as_ptr());
            }

            if CFileDefExt_GetFileTimeString_body(
                this,
                &FileInfo.ftLastWriteTime as *const FILETIME as *mut FILETIME,
                wszBuf.as_mut_ptr(),
                wszBuf.len() as UINT,
            ) != 0 {
                SetDlgItemTextW(hwndDlg, 14017, wszBuf.as_ptr());
            }

            let mut FileSize: ULARGE_INTEGER = core::mem::zeroed();
            FileSize.QuadPart = ((FileInfo.nFileSizeHigh as ULONGLONG) << 32) | FileInfo.nFileSizeLow as ULONGLONG;
            if SH_FormatFileSizeWithBytes(
                &FileSize as *const ULARGE_INTEGER,
                wszBuf.as_mut_ptr(),
                wszBuf.len() as UINT,
            ).is_null() == false {
                SetDlgItemTextW(hwndDlg, 14011, wszBuf.as_ptr());

                if GetPhysicalFileSize(
                    (*this).m_wszPath.as_ptr() as *const u16,
                    &mut FileSize as *mut ULARGE_INTEGER,
                ) != 0 {
                    SH_FormatFileSizeWithBytes(
                        &FileSize as *const ULARGE_INTEGER,
                        wszBuf.as_mut_ptr(),
                        wszBuf.len() as UINT,
                    );
                } else {
                    ERR!("Unreliable size on disk\n");
                }

                SetDlgItemTextW(hwndDlg, 14012, wszBuf.as_ptr());
            }
        }
    }

    if (*this).m_bDir != 0 {
        (*this).m_hWndDirStatsDlg = hwndDlg;
        CFileDefExt_AddRef(this);
        if SHCreateThread(
            _CountFolderAndFilesThreadProc,
            this as *mut core::ffi::c_void,
            0,
            core::ptr::null_mut(),
        ) == 0 {
            CFileDefExt_Release(this);
        }
    }

    ShowWindow(
        GetDlgItem(hwndDlg, 14028),
        SW_HIDE as INT,
    );

    TRUE as BOOL
}

pub unsafe fn CFileDefExt_InitGeneralPage_body(this: *mut CFileDefExt, hwndDlg: HWND) -> BOOL {
    // Set general text properties filename filelocation and icon
    CFileDefExt_InitFilePath_body(this, hwndDlg);

    // Set file type and icon
    CFileDefExt_InitFileType_body(this, hwndDlg);

    // Set open with application
    if (*this).m_bDir == 0 {
        if PathIsExeW((*this).m_wszPath.as_ptr()) == 0 {
            CFileDefExt_InitOpensWithField_body(this, hwndDlg);
        } else {
            let mut wszBuf: [u16; MAX_PATH as usize] =
                [0; MAX_PATH as usize];
            let cchBuf: INT = MAX_PATH as INT;
            LoadStringW(shell32_hInstance, IDS_EXE_DESCRIPTION, wszBuf.as_mut_ptr(), cchBuf);
            SetDlgItemTextW(hwndDlg, 14006, wszBuf.as_ptr());

            let hwndShow = GetDlgItem(hwndDlg, 14024);
            ShowWindow(hwndShow, SW_HIDE as INT);

            // hidden button 14024 allows to draw edit 14007 larger than defined in resources,
            // we use edit 14009 as idol
            let mut rectIdol: RECT = core::mem::zeroed();
            let mut rectToAdjust: RECT = core::mem::zeroed();
            let hwndIdol = GetDlgItem(hwndDlg, 14009);
            GetClientRect(hwndIdol, &mut rectIdol);
            let hwndAdjust = GetDlgItem(hwndDlg, 14007);
            GetClientRect(hwndAdjust, &mut rectToAdjust);
            let hwndTarget = GetDlgItem(hwndDlg, 14007);
            let cx = rectIdol.right - rectIdol.left;
            let cy = rectToAdjust.bottom - rectToAdjust.top;
            let uFlags = SWP_NOMOVE | SWP_NOZORDER;
            SetWindowPos(
                hwndTarget,
                HWND_TOP,
                0,
                0,
                cx,
                cy,
                uFlags,
            );

            let pwszDescr = CFileVersionInfo_GetString(&mut (*this).m_VerInfo, CPP_COM_WSTR_0C671556E060.as_ptr());
            if !pwszDescr.is_null() {
                SetDlgItemTextW(hwndDlg, 14007, pwszDescr);
            } else {
                let pwszFileName = PathFindFileNameW((*this).m_wszPath.as_ptr());
                let size = core::mem::size_of_val(&wszBuf);
                StringCbCopyW(wszBuf.as_mut_ptr(), size, pwszFileName);
                PathRemoveExtension(wszBuf.as_mut_ptr());
                SetDlgItemTextW(hwndDlg, 14007, wszBuf.as_ptr());
            }
        }
    }

    // Set file created/modfied/accessed time, size and attributes
    CFileDefExt_InitFileAttr_body(this, hwndDlg);

    TRUE
}

pub unsafe fn CFileDefExt_GeneralPageProc_body(this: *mut CFileDefExt, hwndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let mut pFileDefExt = GetWindowLongPtr(hwndDlg, DWLP_USER) as *mut CFileDefExt;
    match uMsg {
        WM_INITDIALOG => {
            let ppsp = lParam as *const PROPSHEETPAGEW;
            if ppsp.is_null() || (*ppsp).lParam == 0 {
                // no action
            } else {
                pFileDefExt = (*ppsp).lParam as *mut CFileDefExt;
                SetWindowLongPtr(hwndDlg, DWLP_USER, pFileDefExt as isize);
                CFileDefExt_InitGeneralPage_body(pFileDefExt, hwndDlg);
            }
        }
        WM_COMMAND => {
            if LOWORD!(wParam) == 14024 {
                let mut oainfo: OPENASINFO = core::mem::zeroed();
                oainfo.pcszFile = (*pFileDefExt).m_wszPath.as_ptr();
                oainfo.pcszClass = core::ptr::null();
                oainfo.oaifInFlags = OAIF_REGISTER_EXT | OAIF_FORCE_REGISTRATION;
                if SHOpenWithDialog(hwndDlg, &mut oainfo as *mut _) == S_OK {
                    CFileDefExt_InitGeneralPage_body(pFileDefExt, hwndDlg);
                    PropSheet_Changed(GetParent(hwndDlg), hwndDlg);
                }
            } else if LOWORD!(wParam) == 14021
                || LOWORD!(wParam) == 14022
                || LOWORD!(wParam) == 14023
            {
                PropSheet_Changed(GetParent(hwndDlg), hwndDlg);
            } else if LOWORD!(wParam) == 14001 {
                if HIWORD(wParam) == EN_CHANGE {
                    PropSheet_Changed(GetParent(hwndDlg), hwndDlg);
                }
            }
        }
        WM_NOTIFY => {
            let lppsn = lParam as *const PSHNOTIFY;
            if (*lppsn).hdr.code == PSN_APPLY {
                let mut dwAttr = GetFileAttributesW((*pFileDefExt).m_wszPath.as_ptr());
                if dwAttr != 0 {
                    dwAttr &= !(FILE_ATTRIBUTE_READONLY | FILE_ATTRIBUTE_HIDDEN | FILE_ATTRIBUTE_ARCHIVE);
                    if BST_CHECKED as LRESULT
                        == SendDlgItemMessageW(
                            hwndDlg,
                            14021,
                            BM_GETCHECK,
                            0,
                            0,
                        )
                    {
                        dwAttr |= FILE_ATTRIBUTE_READONLY;
                    }
                    if BST_CHECKED as LRESULT
                        == SendDlgItemMessageW(
                            hwndDlg,
                            14022,
                            BM_GETCHECK,
                            0,
                            0,
                        )
                    {
                        dwAttr |= FILE_ATTRIBUTE_HIDDEN;
                    }
                    if BST_CHECKED as LRESULT
                        == SendDlgItemMessageW(
                            hwndDlg,
                            14023,
                            BM_GETCHECK,
                            0,
                            0,
                        )
                    {
                        dwAttr |= FILE_ATTRIBUTE_ARCHIVE;
                    }
                    if SetFileAttributesW((*pFileDefExt).m_wszPath.as_ptr(), dwAttr) == 0 {
                        ERR!("SetFileAttributesW failed\n");
                    }
                }
                let mut wszBuf: [u16; MAX_PATH as usize] =
                    [0u16; MAX_PATH as usize];
                StringCchCopyW(
                    wszBuf.as_mut_ptr(),
                    MAX_PATH,
                    (*pFileDefExt).m_wszPath.as_ptr(),
                );
                let pwszFilename = PathFindFileNameW(wszBuf.as_ptr());
                let cchFilenameMax = MAX_PATH
                    - (pwszFilename as usize - wszBuf.as_ptr() as usize) / core::mem::size_of::<WCHAR>();
                if GetDlgItemTextW(
                    hwndDlg,
                    14001,
                    pwszFilename as *mut u16,
                    cchFilenameMax as INT,
                ) != 0
                {
                    if MoveFileW(
                        (*pFileDefExt).m_wszPath.as_ptr(),
                        wszBuf.as_ptr(),
                    ) == 0
                    {
                        ERR!("MoveFileW failed\n");
                    }
                }
                SetWindowLongPtr(hwndDlg, DWLP_MSGRESULT, PSNRET_NOERROR as LONG_PTR);
                return TRUE as INT_PTR;
            }
        }
        PSM_QUERYSIBLINGS => {
            let hIconCtrl = GetDlgItem(hwndDlg, 14025);
            let hIcon = SendMessageW(
                hIconCtrl,
                STM_GETICON,
                0,
                0,
            ) as HICON;
            DestroyIcon(hIcon);
            let hIcon_null: HICON = core::ptr::null_mut();
            SendMessageW(
                hIconCtrl,
                STM_SETICON,
                hIcon_null as WPARAM,
                0,
            );
            CFileDefExt_InitGeneralPage_body(pFileDefExt, hwndDlg);
            return FALSE as INT_PTR;
        }
        WM_DESTROY => {
            cpp_com_interlocked_increment(
                &(*pFileDefExt).m_Destroyed as *const _ as *mut core::ffi::c_long,
            );
        }
        WM_UPDATEDIRSTATS => {
            CFileDefExt_UpdateDirStatsResults_body(pFileDefExt);
        }
        _ => {}
    }
    FALSE as INT_PTR
}

pub unsafe fn CFileDefExt_UpdateDirStatsResults_body(this: *mut CFileDefExt) -> () {
    let mut fmt: [u16; 200] = [0u16; 200];
    let mut buf: [u16; 200] = [0u16; 200];

    LoadStringW(shell32_hInstance, IDS_FILE_FOLDER, fmt.as_mut_ptr(), fmt.len() as INT);

    wsprintfW(
        buf.as_mut_ptr(),
        fmt.as_ptr(),
        (*this).m_cFiles,
        (*this).m_cFolders - if (*this).m_bMultifile != 0 { 0 } else { 1 }
    );

    SetDlgItemTextW(
        (*this).m_hWndDirStatsDlg,
        if (*this).m_bMultifile != 0 { 14001 } else { 14027 },
        buf.as_ptr()
    );

    if !SH_FormatFileSizeWithBytes(
        &raw const (*this).m_DirSize as *const _,
        buf.as_mut_ptr(),
        buf.len() as UINT
    ).is_null() {
        SetDlgItemTextW(
            (*this).m_hWndDirStatsDlg,
            14011,
            buf.as_ptr()
        );
    }

    if !SH_FormatFileSizeWithBytes(
        &raw const (*this).m_DirSizeOnDisc as *const _,
        buf.as_mut_ptr(),
        buf.len() as UINT
    ).is_null() {
        SetDlgItemTextW(
            (*this).m_hWndDirStatsDlg,
            14012,
            buf.as_ptr()
        );
    }
}

pub unsafe fn CFileDefExt_InitDirStats_body(this: *mut CFileDefExt, pStats: *mut DIRTREESTATS) -> () {
    (*pStats).pDirCount = &raw mut (*this).m_cFolders;
    (*pStats).pFileCount = &raw mut (*this).m_cFiles;
    (*pStats).pTotVirtSize = &raw mut (*this).m_DirSize;
    (*pStats).pTotPhysSize = &raw mut (*this).m_DirSizeOnDisc;
    (*pStats).nTick = GetTickCount();
    (*pStats).fAttribSet = !0u32;
    (*pStats).fAttribAll = 0;
    (*pStats).bDeepRecurse = FALSE;
    (*pStats).bMultipleTypes = ((*this).m_bMultifile == 0) as i32;
    (*pStats).szType[0] = UNICODE_NULL;
}

pub unsafe fn ProcessDirStatsItem_body(pszPath: PCWSTR, Stats: &mut DIRTREESTATS, pWFD: *mut WIN32_FIND_DATAW) -> UINT {
    let stats: &mut DIRTREESTATS = Stats;

    let mut fAttrib: UINT;
    let mut nVirtSize = core::mem::MaybeUninit::<ULARGE_INTEGER>::zeroed().assume_init();
    let mut nPhysSize = core::mem::MaybeUninit::<ULARGE_INTEGER>::zeroed().assume_init();

    if !pWFD.is_null() && ((*pWFD).dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) != 0 {
        fAttrib = (*pWFD).dwFileAttributes;
    } else {
        fAttrib = GetFileStats_2_body(pszPath, &mut nVirtSize, &mut nPhysSize);
        if fAttrib == INVALID_FILE_ATTRIBUTES {
            return fAttrib;
        }
    }

    stats.fAttribSet &= fAttrib;
    stats.fAttribAll |= fAttrib;

    if (fAttrib & FILE_ATTRIBUTE_DIRECTORY) != 0 {
        *stats.pDirCount += 1;
    } else {
        *stats.pFileCount += 1;
        (*stats.pTotVirtSize).QuadPart += nVirtSize.QuadPart;
        (*stats.pTotPhysSize).QuadPart += nPhysSize.QuadPart;
    }

    if stats.bMultipleTypes == 0 {
        let mut shfi = core::mem::MaybeUninit::<SHFILEINFOW>::zeroed().assume_init();
        if SHGetFileInfoW(
            pszPath,
            fAttrib,
            &mut shfi,
            core::mem::size_of::<SHFILEINFOW>() as u32,
            SHGFI_TYPENAME | SHGFI_USEFILEATTRIBUTES,
        ) == 0
        {
            stats.bMultipleTypes = TRUE;
        } else if stats.szType[0 as usize] == 0 {
            wcscpy(stats.szType.as_mut_ptr(), shfi.szTypeName.as_ptr());
        } else {
            stats.bMultipleTypes = lstrcmpW(stats.szType.as_ptr(), shfi.szTypeName.as_ptr());
        }
    }

    fAttrib
}

pub unsafe fn CFileDefExt_WalkDirTree_body(this: *mut CFileDefExt, pszBase: PCWSTR, pStats: *mut DIRTREESTATS, pWFD: *mut WIN32_FIND_DATAW) -> BOOL {
    let mut wfd: WIN32_FIND_DATAW = core::mem::zeroed();
    wfd.dwFileAttributes = ProcessDirStatsItem_body(pszBase, &mut (*pStats), pWFD);
    if wfd.dwFileAttributes == INVALID_FILE_ATTRIBUTES {
        return FALSE;
    }
    if (wfd.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) == 0 {
        return TRUE;
    }

    let mut bSuccess: BOOL = TRUE;
    let cch = lstrlenW(pszBase) as usize;
    let pszFull = SHAlloc(
        (cch + MAX_PATH as usize) * core::mem::size_of::<u16>(),
    ) as PWSTR;
    if pszFull.is_null() {
        return FALSE;
    }
    let mut pszFile = pszFull.add(cch);
    wcscpy(pszFull, pszBase);
    wcscpy(pszFile, CPP_COM_WSTR_91C9DC2992DB.as_ptr());
    pszFile = pszFile.add(1);

    let hFind = FindFirstFileW(pszFull, &mut wfd);
    if hFind != INVALID_HANDLE_VALUE {
        while bSuccess != 0 && CFileDefExt_IsDestroyed_body(this) == 0 {
            let nTickNow = GetTickCount();
            if nTickNow.wrapping_sub((*pStats).nTick) >= 500 {
                (*pStats).nTick = nTickNow;
                SendMessageW(
                    (*this).m_hWndDirStatsDlg,
                    WM_UPDATEDIRSTATS,
                    0,
                    0,
                );
            }

            wcscpy(pszFile, wfd.cFileName.as_ptr());
            if (wfd.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) == 0 {
                bSuccess = ProcessDirStatsItem_body(pszFull, &mut (*pStats), core::ptr::null_mut()) as BOOL;
            } else if PathIsDotOrDotDotW(wfd.cFileName.as_ptr()) == 0 {
                bSuccess = CFileDefExt_WalkDirTree_body(this, pszFull, pStats, &mut wfd);
                (*pStats).bDeepRecurse |= bSuccess;
            }

            if FindNextFileW(hFind, &mut wfd) == 0 {
                break;
            }
        }
        FindClose(hFind);
    }
    SHFree(pszFull);
    bSuccess
}

pub unsafe fn CFileDefExt_InitMultifilePageThread_body(this: *mut CFileDefExt) -> () {
    let mut Stats = core::mem::MaybeUninit::<DIRTREESTATS>::uninit();
    CFileDefExt_InitDirStats_body(this, Stats.as_mut_ptr());
    let mut Stats = Stats.assume_init();

    let mut i: usize = 0;
    while i < ((*this).m_cidl as usize) {
        let pidl: PIDLIST_ABSOLUTE = ILCombine((*this).m_pidlFolder, *((*this).m_pidls).add(i));
        if pidl.is_null() {
            return;
        }
        let mut pszAbsPath: PWSTR = core::ptr::null_mut();
        let hr: HRESULT = SHELL_DisplayNameOf(core::ptr::null_mut(), pidl, SHGDN_FORPARSING, &mut pszAbsPath);
        ILFree(pidl);
        if hr < 0 {
            return;
        }
        let bSuccess: BOOL = CFileDefExt_WalkDirTree_body(this, pszAbsPath, &mut Stats as *mut DIRTREESTATS, core::ptr::null_mut());
        SHFree(pszAbsPath);
        if bSuccess == 0 {
            return;
        }
        i += 1;
    }

    CFileDefExt_UpdateDirStatsResults_body(this);

    if Stats.bMultipleTypes != 0 {
        LoadStringW(shell32_hInstance, IDS_MULTIPLETYPES, Stats.szType.as_mut_ptr(), Stats.szType.len() as INT);
    }
    SetDlgItemTextW((*this).m_hWndDirStatsDlg, 14005, Stats.szType.as_ptr());

    let mut pszDir: PWSTR = core::ptr::null_mut();
    let mut hr: HRESULT = E_FAIL;
    if Stats.bDeepRecurse == 0 {
        hr = SHELL_DisplayNameOf(core::ptr::null_mut(), (*this).m_pidlFolder, SHGDN_FORPARSING | SHGDN_FORADDRESSBAR, &mut pszDir);
    }
    if hr >= 0 {
        SetDlgItemTextW((*this).m_hWndDirStatsDlg, 14009, pszDir);
    } else {
        LoadStringW(shell32_hInstance, IDS_VARIOUSFOLDERS, Stats.szType.as_mut_ptr(), Stats.szType.len() as INT);
        SetDlgItemTextW((*this).m_hWndDirStatsDlg, 14009, Stats.szType.as_ptr());
    }
    SHFree(pszDir);

    macro_rules! SetMultifileDlgFileAttr {
        ($attr:expr, $id:expr) => {
            if Stats.fAttribSet & ($attr) != 0 {
                CheckDlgButton((*this).m_hWndDirStatsDlg, $id, BST_CHECKED);
            } else if Stats.fAttribAll & ($attr) != 0 {
                CheckDlgButton((*this).m_hWndDirStatsDlg, $id, BST_INDETERMINATE);
            }
        };
    }
    SetMultifileDlgFileAttr!(FILE_ATTRIBUTE_READONLY, 14021);
    SetMultifileDlgFileAttr!(FILE_ATTRIBUTE_HIDDEN, 14022);
    SetMultifileDlgFileAttr!(FILE_ATTRIBUTE_ARCHIVE, 14023);
}

pub unsafe fn CFileDefExt__InitializeMultifileThreadProc_body(this: *mut CFileDefExt, lpParameter: LPVOID) -> DWORD {
    let pThis = lpParameter as *mut CFileDefExt;
    CFileDefExt_InitMultifilePageThread_body(pThis);
    let result = CFileDefExt_Release(pThis);
    result
}

pub unsafe fn CFileDefExt_InitMultifilePage_body(this: *mut CFileDefExt, hwndDlg: HWND) -> () {
    (*this).m_hWndDirStatsDlg = hwndDlg;
    let hIco: HICON = LoadIconW(shell32_hInstance, MAKEINTRESOURCEW(IDI_SHELL_MULTIPLE_FILES));
    SendDlgItemMessageW(hwndDlg, 14000, STM_SETICON, hIco as WPARAM, 0);

    // We are lazy and just reuse the directory page so we must tweak some controls
    let hCtrl = GetDlgItem(hwndDlg, 14001);
    SetWindowLongPtrW(hCtrl, GWL_STYLE, GetWindowLongPtrW(GetDlgItem(hwndDlg, 14027), GWL_STYLE));
    SetWindowLongPtrW(hCtrl, GWL_EXSTYLE, 0);
    SendMessageW(hCtrl, EM_SETREADONLY, TRUE as WPARAM, 0);
    SetDlgItemTextW(hwndDlg, 14005, core::ptr::null_mut());

    static idAttr: [WORD; 3] = [14021, 14022, 14023];
    for i in 0..idAttr.len() {
        let hCtrl = GetDlgItem(hwndDlg, idAttr[i] as INT);
        SendMessageW(hCtrl, BM_SETSTYLE, BS_AUTO3STATE as WPARAM, TRUE as LPARAM);
        EnableWindow(hCtrl, FALSE);
    }

    static idUnused: [WORD; 5] = [14026, 14027, 14028, 14014, 14015];
    for i in 0..idUnused.len() {
        DestroyWindow(GetDlgItem(hwndDlg, idUnused[i] as INT));
    }

    if (*this).m_cidl == 0 {
        return;
    }

    CFileDefExt_AddRef(this);
    if SHCreateThread(_InitializeMultifileThreadProc, this as *mut c_void, CTF_COINIT, core::ptr::null_mut()) == 0 {
        CFileDefExt_Release(this);
    }
}

pub unsafe fn CFileDefExt_MultifilePageProc_body(this: *mut CFileDefExt, hwndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let mut pFileDefExt = GetWindowLongPtr(hwndDlg, DWLP_USER) as *mut CFileDefExt;
    match uMsg {
        WM_INITDIALOG => {
            let ppsp = lParam as *const PROPSHEETPAGEW;
            if !ppsp.is_null() && ((*ppsp).lParam as isize) != 0 {
                pFileDefExt = (*ppsp).lParam as *mut CFileDefExt;
                SetWindowLongPtr(hwndDlg, DWLP_USER, pFileDefExt as isize);
                CFileDefExt_InitMultifilePage_body(pFileDefExt, hwndDlg);
            }
        }
        WM_DESTROY => {
            cpp_com_interlocked_increment(&raw mut (*pFileDefExt).m_Destroyed);
        }
        WM_UPDATEDIRSTATS => {
            CFileDefExt_UpdateDirStatsResults_body(pFileDefExt);
        }
        _ => {}
    }
    FALSE as INT_PTR
}

pub unsafe fn CFileDefExt_InitVersionPage_body(this: *mut CFileDefExt, hwndDlg: HWND) -> BOOL {
    let pInfo = CFileVersionInfo_GetFixedInfo(&mut (*this).m_VerInfo);
    if !pInfo.is_null() {
        let mut wszVersion: [u16; 256] = [0; 256];
        swprintf(
            wszVersion.as_mut_ptr(),
            CPP_COM_WSTR_03971BB4A692.as_ptr(),
            HIWORD((*pInfo).dwFileVersionMS as WPARAM),
            LOWORD!((*pInfo).dwFileVersionMS),
            HIWORD((*pInfo).dwFileVersionLS as WPARAM),
            LOWORD!((*pInfo).dwFileVersionLS),
        );
        SetDlgItemTextW(hwndDlg, 14001, wszVersion.as_ptr());
    }

    CFileDefExt_SetVersionLabel_body(this, hwndDlg, 14003, CPP_COM_WSTR_0C671556E060.as_ptr());
    CFileDefExt_SetVersionLabel_body(this, hwndDlg, 14005, CPP_COM_WSTR_5454D22E8E39.as_ptr());

    CFileDefExt_AddVersionString_body(this, hwndDlg, CPP_COM_WSTR_B6C7E0602F31.as_ptr());

    let pwszLang = CFileVersionInfo_GetLangName(&mut (*this).m_VerInfo);
    if !pwszLang.is_null() {
        let hDlgCtrl = GetDlgItem(hwndDlg, 14009);
        let Index = SendMessageW(
            hDlgCtrl,
            LB_ADDSTRING,
            (-(1 as isize)) as WPARAM,
            CPP_COM_WSTR_A62088E6F2CF.as_ptr() as LPARAM,
        );
        SendMessageW(
            hDlgCtrl,
            LB_SETITEMDATA,
            Index as WPARAM,
            pwszLang as LPARAM,
        );
    }

    CFileDefExt_AddVersionString_body(this, hwndDlg, CPP_COM_WSTR_1CDEBCAEE168.as_ptr());
    CFileDefExt_AddVersionString_body(this, hwndDlg, CPP_COM_WSTR_3AFD8CAA0EBD.as_ptr());
    CFileDefExt_AddVersionString_body(this, hwndDlg, CPP_COM_WSTR_06FC66BAEF86.as_ptr());
    CFileDefExt_AddVersionString_body(this, hwndDlg, CPP_COM_WSTR_4BFCAAE9CBA7.as_ptr());
    CFileDefExt_AddVersionString_body(this, hwndDlg, CPP_COM_WSTR_6D1579B9C364.as_ptr());
    CFileDefExt_AddVersionString_body(this, hwndDlg, CPP_COM_WSTR_F44FE681783B.as_ptr());
    CFileDefExt_AddVersionString_body(this, hwndDlg, CPP_COM_WSTR_F1F5916EF730.as_ptr());

    if !pInfo.is_null() && ((*pInfo).dwFileFlags & VS_FF_PRIVATEBUILD) != 0 {
        CFileDefExt_AddVersionString_body(this, hwndDlg, CPP_COM_WSTR_F833939E65C1.as_ptr());
    }

    if !pInfo.is_null() && ((*pInfo).dwFileFlags & VS_FF_SPECIALBUILD) != 0 {
        CFileDefExt_AddVersionString_body(this, hwndDlg, CPP_COM_WSTR_7C7EBA185B45.as_ptr());
    }

    SetWindowLongPtr(hwndDlg, DWLP_USER, this as LONG_PTR);

    let hDlgCtrl = GetDlgItem(hwndDlg, 14009);
    SendMessageW(
        hDlgCtrl,
        LB_SETCURSEL,
        0,
        0,
    );

    let pwszText = SendMessageW(
        hDlgCtrl,
        LB_GETITEMDATA,
        0 as WPARAM,
        0 as LPARAM,
    ) as LPCWSTR;

    if !pwszText.is_null() && pwszText != LB_ERR as LPCWSTR {
        SetDlgItemTextW(hwndDlg, 14010, pwszText);
    }

    TRUE
}

pub unsafe fn CFileDefExt_SetVersionLabel_body(this: *mut CFileDefExt, hwndDlg: HWND, idCtrl: DWORD, pwszName: LPCWSTR) -> BOOL {
    if hwndDlg.is_null() || pwszName.is_null() {
        return FALSE;
    }

    let pwszValue = CFileVersionInfo_GetString(&mut (*this).m_VerInfo, pwszName);
    if !pwszValue.is_null() {
        SetDlgItemTextW(hwndDlg, idCtrl as i32, pwszValue);
        return TRUE;
    }

    FALSE
}

pub unsafe fn CFileDefExt_AddVersionString_body(this: *mut CFileDefExt, hwndDlg: HWND, pwszName: LPCWSTR) -> BOOL {
    if hwndDlg.is_null() || pwszName.is_null() {
        return FALSE;
    }

    let pwszValue = CFileVersionInfo_GetString(&mut (*this).m_VerInfo, pwszName);
    if !pwszValue.is_null() {
        let hDlgCtrl = GetDlgItem(hwndDlg, 14009);
        let index = SendMessageW(
            hDlgCtrl,
            LB_ADDSTRING,
            (-(1 as isize)) as WPARAM,
            pwszName as LPARAM,
        );
        SendMessageW(
            hDlgCtrl,
            LB_SETITEMDATA,
            index as WPARAM,
            pwszValue as LPARAM,
        );
        return TRUE;
    }

    FALSE
}

pub unsafe fn CFileDefExt_VersionPageProc_body(this: *mut CFileDefExt, hwndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    match uMsg {
        WM_INITDIALOG => {
            // lParam is a pointer to PROPSHEETPAGE
            let ppsp = lParam as *mut PROPSHEETPAGE;
            if ppsp.is_null() {
                return FALSE as INT_PTR;
            }
            if (*ppsp).lParam == 0 {
                return FALSE as INT_PTR;
            }
            let pFileDefExt = (*ppsp).lParam as *mut CFileDefExt;
            // Call InitVersionPage and return its BOOL as INT_PTR
            return CFileDefExt_InitVersionPage_body(pFileDefExt, hwndDlg) as INT_PTR;
        }
        WM_COMMAND => {
            if LOWORD!(wParam) == 14009
                && HIWORD(wParam) == LBN_SELCHANGE
            {
                let hDlgCtrl = lParam as HWND;
                let index = SendMessageW(
                    hDlgCtrl,
                    LB_GETCURSEL,
                    0 as WPARAM,
                    0 as LPARAM,
                );
                if index == LB_ERR as LRESULT {
                    // no action, fall through to break
                } else {
                    let pwszData = SendMessageW(
                        hDlgCtrl,
                        LB_GETITEMDATA,
                        index as WPARAM,
                        0 as LPARAM,
                    ) as *const u16;
                    if !pwszData.is_null() {
                        // CString is dynamically sized. Allocate exactly enough storage so
                        // long version-resource strings are never truncated to MAX_PATH.
                        let source_len = lstrlenW(pwszData) as usize;
                        let mut start = 0usize;
                        let mut end = source_len;
                        while start < end && {
                            let c = *pwszData.add(start);
                            c == 0x20 || c == 0x09 || c == 0x0A || c == 0x0D
                        } {
                            start += 1;
                        }
                        while end > start && {
                            let c = *pwszData.add(end - 1);
                            c == 0x20 || c == 0x09 || c == 0x0A || c == 0x0D
                        } {
                            end -= 1;
                        }
                        let len = end - start;
                        let trimmed = HeapAlloc(GetProcessHeap(), 0, (len + 1) * core::mem::size_of::<WCHAR>()) as LPWSTR;
                        if trimmed.is_null() { return FALSE as INT_PTR; }
                        core::ptr::copy_nonoverlapping(pwszData.add(start), trimmed, len);
                        *trimmed.add(len) = 0;
                        SetDlgItemTextW(hwndDlg, 14010, trimmed);
                        HeapFree(GetProcessHeap(), 0, trimmed);
                        return TRUE as INT_PTR;
                    }
                }
            }
        }
        _ => {}
    }

    FALSE as INT_PTR
}

pub unsafe fn CFileDefExt_FolderCustomizePageProc_body(this: *mut CFileDefExt, hwndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let mut pFileDefExt = GetWindowLongPtr(hwndDlg, DWLP_USER) as *mut CFileDefExt;
    match uMsg {
        WM_INITDIALOG => {
            let ppsp = lParam as *const PROPSHEETPAGEW;
            if ppsp.is_null() || (*ppsp).lParam == 0 {
            } else {
                pFileDefExt = (*ppsp).lParam as *mut CFileDefExt;
                return CFileDefExt_InitFolderCustomizePage_body(pFileDefExt, hwndDlg) as INT_PTR;
            }
        }
        WM_COMMAND => {
            match LOWORD!(wParam) {
                IDC_FOLDERCUST_CHANGE_ICON => {
                    CFileDefExt_OnFolderCustChangeIcon_body(pFileDefExt, hwndDlg);
                }
                IDC_FOLDERCUST_CHOOSE_PIC => {}
                IDC_FOLDERCUST_RESTORE_DEFAULTS => {}
                _ => {}
            }
        }
        WM_NOTIFY => {
            let lppsn = lParam as *const NMHDR;
            if (*lppsn).code == PSN_APPLY {
                if CFileDefExt_OnFolderCustApply_body(pFileDefExt, hwndDlg) != 0 {
                    SetWindowLongPtr(hwndDlg, DWLP_MSGRESULT, PSNRET_NOERROR as LONG_PTR);
                } else {
                    SetWindowLongPtr(hwndDlg, DWLP_MSGRESULT, PSNRET_INVALID_NOCHANGEPAGE as LONG_PTR);
                }
                return TRUE as INT_PTR;
            }
        }
        PSM_QUERYSIBLINGS => {
            return FALSE as INT_PTR;
        }
        WM_DESTROY => {
            CFileDefExt_OnFolderCustDestroy_body(pFileDefExt, hwndDlg);
        }
        _ => {}
    }
    FALSE as INT_PTR
}

pub unsafe fn CFileDefExt_OnFolderCustDestroy_body(this: *mut CFileDefExt, hwndDlg: HWND) -> () {
    DestroyIcon((*this).m_hFolderIcon);
    (*this).m_hFolderIcon = core::ptr::null_mut();

    SetWindowLongPtr(
        hwndDlg,
        DWLP_USER,
        0 as isize,
    );
}

pub unsafe fn CFileDefExt_UpdateFolderIcon_body(this: *mut CFileDefExt, hwndDlg: HWND) -> () {
    // destroy icon if any
    if !(*this).m_hFolderIcon.is_null() {
        DestroyIcon((*this).m_hFolderIcon);
        (*this).m_hFolderIcon = core::ptr::null_mut();
    }

    // create the icon
    if (*this).m_szFolderIconPath[0] == 0
        && (*this).m_nFolderIconIndex == 0
    {
        // Windows ignores shell customization here and uses the default icon
        (*this).m_hFolderIcon = LoadIconW(
            shell32_hInstance,
            MAKEINTRESOURCEW(IDI_SHELL_FOLDER),
        );
    } else {
        let mut h_icon: HICON = core::ptr::null_mut();
        ExtractIconExW(
            (*this).m_szFolderIconPath.as_ptr(),
            (*this).m_nFolderIconIndex,
            &mut h_icon,
            core::ptr::null_mut(),
            1,
        );
        (*this).m_hFolderIcon = h_icon;
    }

    // set icon
    SendDlgItemMessageW(
        hwndDlg,
        IDC_FOLDERCUST_ICON as INT,
        STM_SETICON,
        (*this).m_hFolderIcon as WPARAM,
        0,
    );
}

pub unsafe fn CFileDefExt_InitFolderCustomizePage_body(this: *mut CFileDefExt, hwndDlg: HWND) -> BOOL {
    SetWindowLongPtr(hwndDlg, DWLP_USER, this as isize);
    EnableWindow(
        GetDlgItem(hwndDlg, IDC_FOLDERCUST_COMBOBOX as INT),
        FALSE,
    );
    EnableWindow(
        GetDlgItem(hwndDlg, IDC_FOLDERCUST_CHECKBOX as INT),
        FALSE,
    );
    EnableWindow(
        GetDlgItem(hwndDlg, IDC_FOLDERCUST_CHOOSE_PIC as INT),
        FALSE,
    );
    EnableWindow(
        GetDlgItem(hwndDlg, IDC_FOLDERCUST_RESTORE_DEFAULTS as INT),
        FALSE,
    );

    let mut szIniFile: [u16; MAX_PATH] = [0; MAX_PATH];
    StringCchCopyW(
        szIniFile.as_mut_ptr(),
        MAX_PATH,
        (*this).m_wszPath.as_ptr(),
    );
    PathAppendW(szIniFile.as_mut_ptr(), CPP_COM_WSTR_A5DBFDEBD401.as_ptr());

    (*this).m_szFolderIconPath[0 as usize] = 0;
    (*this).m_nFolderIconIndex = 0;

    if GetPrivateProfileStringW(
        s_szShellClassInfo,
        s_szIconFile,
        core::ptr::null(),
        (*this).m_szFolderIconPath.as_mut_ptr(),
        MAX_PATH as DWORD,
        szIniFile.as_ptr(),
    ) != 0
    {
        (*this).m_nFolderIconIndex = GetPrivateProfileIntW(
            s_szShellClassInfo,
            s_szIconIndex,
            0,
            szIniFile.as_ptr(),
        ) as INT;
    } else if GetPrivateProfileStringW(
        s_szShellClassInfo,
        s_szIconResource,
        core::ptr::null(),
        (*this).m_szFolderIconPath.as_mut_ptr(),
        MAX_PATH as DWORD,
        szIniFile.as_ptr(),
    ) != 0
    {
        (*this).m_nFolderIconIndex =
            PathParseIconLocationW((*this).m_szFolderIconPath.as_mut_ptr());
    }

    CFileDefExt_UpdateFolderIcon_body(this, hwndDlg);

    TRUE
}

pub unsafe fn CFileDefExt_OnFolderCustChangeIcon_body(this: *mut CFileDefExt, hwndDlg: HWND) -> () {
    let mut szPath: [u16; MAX_PATH] = [0u16; MAX_PATH];
    let mut nIconIndex: i32;

    if (*this).m_szFolderIconPath[0 as usize] != 0 {
        StringCchCopyW(szPath.as_mut_ptr(), szPath.len(), (*this).m_szFolderIconPath.as_ptr());
        nIconIndex = (*this).m_nFolderIconIndex;
    } else {
        szPath[0 as usize] = 0;
        nIconIndex = 0;
    }

    if PickIconDlg(hwndDlg, szPath.as_mut_ptr(), szPath.len() as UINT, &mut nIconIndex) != 0 {
        (*this).m_bFolderIconIsSet = TRUE;

        let hParent = GetParent(hwndDlg);
        PropSheet_Changed(hParent, hwndDlg);

        StringCchCopyW((*this).m_szFolderIconPath.as_mut_ptr(), (*this).m_szFolderIconPath.len(), szPath.as_ptr());
        (*this).m_nFolderIconIndex = nIconIndex;
        CFileDefExt_UpdateFolderIcon_body(this, hwndDlg);
    }
}

pub unsafe fn CFileDefExt_OnFolderCustApply_body(this: *mut CFileDefExt, hwndDlg: HWND) -> BOOL {
    let mut szIniFile: [u16; MAX_PATH] = [0u16; MAX_PATH];
    StringCchCopyW(szIniFile.as_mut_ptr(), szIniFile.len(), (*this).m_wszPath.as_ptr());
    PathAppendW(szIniFile.as_mut_ptr(), CPP_COM_WSTR_A5DBFDEBD401.as_ptr());

    if (*this).m_bFolderIconIsSet != 0 {
        let mut attrs: DWORD;

        attrs = GetFileAttributesW((*this).m_wszPath.as_ptr());
        attrs &= !(FILE_ATTRIBUTE_SYSTEM | FILE_ATTRIBUTE_READONLY);
        SetFileAttributesW((*this).m_wszPath.as_ptr(), attrs);

        attrs = GetFileAttributesW(szIniFile.as_ptr());
        attrs &= !(FILE_ATTRIBUTE_SYSTEM | FILE_ATTRIBUTE_HIDDEN | FILE_ATTRIBUTE_READONLY);
        SetFileAttributesW(szIniFile.as_ptr(), attrs);

        if (*this).m_szFolderIconPath[0 as usize] != 0 {
            WritePrivateProfileStringW(
                s_szShellClassInfo,
                s_szIconFile,
                (*this).m_szFolderIconPath.as_ptr(),
                szIniFile.as_ptr(),
            );

            let mut szInt: [u16; 32] = [0u16; 32];
            StringCchPrintfW(
                szInt.as_mut_ptr(),
                szInt.len(),
                CPP_COM_WSTR_A9EDBA2FC474.as_ptr(),
                (*this).m_nFolderIconIndex,
            );
            WritePrivateProfileStringW(
                s_szShellClassInfo,
                s_szIconIndex,
                szInt.as_ptr(),
                szIniFile.as_ptr(),
            );

            WritePrivateProfileStringW(
                0 as *const _,
                0 as *const _,
                0 as *const _,
                szIniFile.as_ptr(),
            );
        } else {
            WritePrivateProfileStringW(
                s_szShellClassInfo,
                s_szIconFile,
                0 as *const _,
                szIniFile.as_ptr(),
            );
            WritePrivateProfileStringW(
                s_szShellClassInfo,
                s_szIconIndex,
                0 as *const _,
                szIniFile.as_ptr(),
            );
            WritePrivateProfileStringW(
                s_szShellClassInfo,
                s_szIconResource,
                0 as *const _,
                szIniFile.as_ptr(),
            );

            WritePrivateProfileStringW(
                0 as *const _,
                0 as *const _,
                0 as *const _,
                szIniFile.as_ptr(),
            );
        }

        attrs = GetFileAttributesW(szIniFile.as_ptr());
        attrs |= FILE_ATTRIBUTE_SYSTEM | FILE_ATTRIBUTE_HIDDEN;
        SetFileAttributesW(szIniFile.as_ptr(), attrs);

        attrs = GetFileAttributesW((*this).m_wszPath.as_ptr());
        attrs |= FILE_ATTRIBUTE_READONLY;
        SetFileAttributesW((*this).m_wszPath.as_ptr(), attrs);

        PropSheet_QuerySiblings(
            GetParent(hwndDlg),
            0,
            0,
        );

        (*this).m_bFolderIconIsSet = FALSE;
    }

    TRUE
}

pub unsafe fn CFileDefExt_ctor_body(this: *mut CFileDefExt) -> () {
    (*this).m_bDir = FALSE;
    (*this).m_bMultifile = FALSE;
    (*this).m_cFiles = 0;
    (*this).m_cFolders = 0;
    (*this).m_wszPath[0] = 0x0000u16;
    (*this).m_DirSize.QuadPart = 0;
    (*this).m_DirSizeOnDisc.QuadPart = 0;
    (*this).m_szFolderIconPath[0] = 0;
    (*this).m_nFolderIconIndex = 0;
    (*this).m_hFolderIcon = core::ptr::null_mut();
    (*this).m_bFolderIconIsSet = FALSE;
}

pub unsafe fn CFileDefExt_dtor_body(this: *mut CFileDefExt) -> () {
    _ILFreeaPidl((*this).m_pidls, (*this).m_cidl);
    ILFree((*this).m_pidlFolder);
    CFileVersionInfo_Clear(&mut (*this).m_VerInfo);
}

pub unsafe fn CFileDefExt_Initialize_body(this: *mut CFileDefExt, pidlFolder: PCIDLIST_ABSOLUTE, pDataObj: *mut IDataObject, hkeyProgID: HKEY) -> HRESULT {
    let mut format: FORMATETC = core::mem::zeroed();
    let mut stgm: STGMEDIUM = core::mem::zeroed();

    if pDataObj.is_null() {
        return E_FAIL;
    }

    let count = DataObject_GetHIDACount(pDataObj);
    (*this).m_bMultifile = (count > 1) as BOOL;

    if (*this).m_bMultifile != 0 {
        let mut cida: *mut CIDA = core::ptr::null_mut();
        let mut cida_medium: STGMEDIUM = core::mem::zeroed();
        if CDataObjectHIDA_CreateCIDA(pDataObj, &mut cida, &mut cida_medium) >= 0 {
            (*this).m_pidls = CFileDefExt_ILCopyCidaToaPidl(&raw mut (*this).m_pidlFolder, cida);
            if !(*this).m_pidls.is_null() {
                (*this).m_cidl = (*cida).cidl;
            }
            CDataObjectHIDA_DestroyCIDA(cida, &mut cida_medium);
        }
    }

    format.cfFormat = CF_HDROP as WORD;
    format.ptd = core::ptr::null_mut();
    format.dwAspect = DVASPECT_CONTENT;
    // lindex = -1, using the token value (expected to be 1)
    format.lindex = -(1 as i32);
    format.tymed = TYMED_HGLOBAL;

    let hr = ((*(*pDataObj).lpVtbl).GetData)(pDataObj, &mut format, &mut stgm);
    if ((hr) < 0) {
        return hr;
    }

    let result = DragQueryFileW(
        stgm.__bindgen_anon_1.hGlobal as HDROP,
        0,
        (*this).m_wszPath.as_mut_ptr(),
        ((*this).m_wszPath).len() as UINT,
    );
    if result == 0 {
        ReleaseStgMedium(&mut stgm);
        return E_FAIL;
    }

    ReleaseStgMedium(&mut stgm);

    (*this).m_bDir = ((*this).m_bMultifile == 0
        && PathIsDirectoryW((*this).m_wszPath.as_ptr()) != 0) as BOOL;

    if (*this).m_bDir == 0 && (*this).m_bMultifile == 0 {
        CFileVersionInfo_Load_body(&mut (*this).m_VerInfo, (*this).m_wszPath.as_ptr());
    }

    S_OK
}

pub unsafe fn CFileDefExt_QueryContextMenu_body(this: *mut CFileDefExt, hmenu: HMENU, indexMenu: UINT, idCmdFirst: UINT, idCmdLast: UINT, uFlags: UINT) -> HRESULT { E_NOTIMPL }

pub unsafe fn CFileDefExt_InvokeCommand_body(this: *mut CFileDefExt, lpici: LPCMINVOKECOMMANDINFO) -> HRESULT { E_NOTIMPL }

pub unsafe fn CFileDefExt_GetCommandString_body(this: *mut CFileDefExt, idCmd: UINT_PTR, uType: UINT, pwReserved: *mut UINT, pszName: LPSTR, cchMax: UINT) -> HRESULT { E_NOTIMPL }

pub unsafe fn CFileDefExt_AddPages_body(this: *mut CFileDefExt, pfnAddPage: LPFNSVADDPROPSHEETPAGE, lParam: LPARAM) -> HRESULT {
    let wResId: u16 = if (*this).m_bDir != 0 || (*this).m_bMultifile != 0 {
        IDD_FOLDER_PROPERTIES as u16
    } else {
        IDD_FILE_PROPERTIES as u16
    };
    let pfnFirstPage: DLGPROC = if (*this).m_bMultifile != 0 {
        MultifilePageProc
    } else {
        GeneralPageProc
    };

    let hPage: HPROPSHEETPAGE = SH_CreatePropertySheetPageEx(
        wResId,
        pfnFirstPage,
        this as LPARAM,
        core::ptr::null_mut(),
        CFileDefExt_PropSheetPageLifetimeCallback,
    );

    let hr: HRESULT = AddPropSheetPage(hPage, pfnAddPage, lParam);

    if ((hr) < 0) {
        return hr;
    } else {
        CFileDefExt_AddRef(this);
    }

    if (*this).m_bMultifile != 0 {
        return S_OK;
    }

    if (*this).m_bDir == 0 && GetFileVersionInfoSizeW((*this).m_wszPath.as_ptr(), core::ptr::null_mut()) != 0 {
        let hPage = SH_CreatePropertySheetPage(
            IDD_FILE_VERSION as WORD,
            VersionPageProc,
            this as LPARAM,
            core::ptr::null_mut(),
        );
        AddPropSheetPage(hPage, pfnAddPage, lParam);
    }

    if (*this).m_bDir != 0 {
        let hPage = SH_CreatePropertySheetPage(
            IDD_FOLDER_CUSTOMIZE as WORD,
            FolderCustomizePageProc,
            this as LPARAM,
            core::ptr::null_mut(),
        );
        AddPropSheetPage(hPage, pfnAddPage, lParam);
    }

    S_OK
}

pub unsafe fn CFileDefExt_ReplacePage_body(this: *mut CFileDefExt, uPageID: EXPPS, pfnReplacePage: LPFNSVADDPROPSHEETPAGE, lParam: LPARAM) -> HRESULT { E_NOTIMPL }

pub unsafe fn CFileDefExt_CountFolderAndFiles_body(this: *mut CFileDefExt) -> () {
    let mut stats: DIRTREESTATS = core::mem::zeroed();
    CFileDefExt_InitDirStats_body(this, &mut stats as *mut _);
    CFileDefExt_WalkDirTree_body(this, (*this).m_wszPath.as_ptr(), &mut stats as *mut _, core::ptr::null_mut());
    CFileDefExt_UpdateDirStatsResults_body(this);
}

pub unsafe fn CFileDefExt__CountFolderAndFilesThreadProc_body(this: *mut CFileDefExt, lpParameter: LPVOID) -> DWORD {
    let pThis: *mut CFileDefExt = lpParameter as *mut CFileDefExt;
    CFileDefExt_CountFolderAndFiles_body(pThis);
    CFileDefExt_Release(pThis)
}
