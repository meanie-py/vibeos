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

// Deterministic from shobjidl.h
#[repr(C)]
pub struct IContextMenu3 {
    pub lpVtbl: *const IContextMenu3Vtbl,
}
#[repr(C)]
pub struct IContextMenu3Vtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IContextMenu3, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IContextMenu3) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IContextMenu3) -> ULONG,
    pub QueryContextMenu: unsafe extern "system" fn(This: *mut IContextMenu3, hmenu: HMENU, indexMenu: UINT, idCmdFirst: UINT, idCmdLast: UINT, uFlags: UINT) -> HRESULT,
    pub InvokeCommand: unsafe extern "system" fn(This: *mut IContextMenu3, lpici: LPCMINVOKECOMMANDINFO) -> HRESULT,
    pub GetCommandString: unsafe extern "system" fn(This: *mut IContextMenu3, idCmd: UINT_PTR, uType: UINT, pwReserved: *mut UINT, pszName: LPSTR, cchMax: UINT) -> HRESULT,
    pub HandleMenuMsg: unsafe extern "system" fn(This: *mut IContextMenu3, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> HRESULT,
    pub HandleMenuMsg2: unsafe extern "system" fn(This: *mut IContextMenu3, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, plResult: *mut LRESULT) -> HRESULT,
}
extern "C" {
    pub static IID_IContextMenu3: IID;
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

pub type SHELLNEW_TYPE = INT;
pub const SHELLNEW_TYPE_INVALID: SHELLNEW_TYPE = -1;
pub const SHELLNEW_TYPE_COMMAND: SHELLNEW_TYPE = 1;
pub const SHELLNEW_TYPE_DATA: SHELLNEW_TYPE = 2;
pub const SHELLNEW_TYPE_FILENAME: SHELLNEW_TYPE = 4;
pub const SHELLNEW_TYPE_NULLFILE: SHELLNEW_TYPE = 8;

#[repr(C)]
pub struct SHELLNEW_ITEM {
    pub Type: SHELLNEW_TYPE,
    pub pwszExt: LPWSTR,
    pub pData: PBYTE,
    pub cbData: ULONG,
    pub pwszDesc: LPWSTR,
    pub hIcon: HICON,
    pub pNext: *mut SHELLNEW_ITEM,
}

// Deterministic COM scaffold for CNewMenu; method bodies are generated separately.
extern "system" {
    fn GetProcessHeap() -> HANDLE;
    #[link_name = "HeapAlloc"] fn cpp_com_HeapAlloc(heap: HANDLE, flags: DWORD, bytes: usize) -> *mut core::ffi::c_void;
    #[link_name = "HeapFree"] fn cpp_com_HeapFree(heap: HANDLE, flags: DWORD, memory: *mut core::ffi::c_void) -> BOOL;
    fn Shell32RustObjectLock();
    fn Shell32RustObjectUnlock();
}
#[inline]
unsafe fn HeapAlloc<N: CppComSize>(heap: HANDLE, flags: DWORD, bytes: N) -> *mut core::ffi::c_void {
    cpp_com_HeapAlloc(heap, flags, bytes.cpp_com_size())
}
#[inline]
unsafe fn HeapFree<T>(heap: HANDLE, flags: DWORD, memory: *mut T) -> BOOL {
    cpp_com_HeapFree(heap, flags, memory.cast())
}

#[repr(C)]
pub struct CNewMenu {
    pub IObjectWithSite_iface: IObjectWithSite,
    pub IContextMenu3_iface: IContextMenu3,
    pub IContextMenu2_iface: IContextMenu2,
    pub IContextMenu_iface: IContextMenu,
    pub IShellExtInit_iface: IShellExtInit,
    pub ref_count: core::sync::atomic::AtomicI32,
    pub m_pidlFolder: LPITEMIDLIST,
    pub m_pItems: *mut SHELLNEW_ITEM,
    pub m_pLinkItem: *mut SHELLNEW_ITEM,
    pub m_pSite: *mut IUnknown,
    pub m_hSubMenu: HMENU,
    pub m_idCmdFirst: UINT,
    pub m_idCmdFolder: UINT,
    pub m_idCmdLink: UINT,
    pub m_bCustomIconFolder: BOOL,
    pub m_bCustomIconLink: BOOL,
    pub m_hIconFolder: HICON,
    pub m_hIconLink: HICON,
}

unsafe fn CNewMenu_cpp_com_assign_m_pSite(this: *mut CNewMenu, value: *mut IUnknown) {
    let old = (*this).m_pSite;
    if old == value { return; }
    if !value.is_null() {
        let unknown = value as *mut IUnknown;
        let add_ref: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).AddRef);
        add_ref(unknown);
    }
    (*this).m_pSite = value;
    if !old.is_null() {
        let unknown = old as *mut IUnknown;
        let release: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).Release);
        release(unknown);
    }
}

unsafe extern "system" fn CNewMenu_IObjectWithSite_QueryInterface(This: *mut IObjectWithSite, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CNewMenu, IObjectWithSite_iface)) as *mut CNewMenu;
    CNewMenu_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CNewMenu_IObjectWithSite_AddRef(This: *mut IObjectWithSite) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CNewMenu, IObjectWithSite_iface)) as *mut CNewMenu;
    CNewMenu_AddRef(this)
}
unsafe extern "system" fn CNewMenu_IObjectWithSite_Release(This: *mut IObjectWithSite) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CNewMenu, IObjectWithSite_iface)) as *mut CNewMenu;
    CNewMenu_Release(this)
}
unsafe extern "system" fn CNewMenu_IObjectWithSite_SetSite(This: *mut IObjectWithSite, pUnkSite: *mut IUnknown) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CNewMenu, IObjectWithSite_iface)) as *mut CNewMenu;
    CNewMenu_SetSite_body(this, pUnkSite)
}
unsafe extern "system" fn CNewMenu_IObjectWithSite_GetSite(This: *mut IObjectWithSite, riid: REFIID, ppvSite: *mut PVOID) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CNewMenu, IObjectWithSite_iface)) as *mut CNewMenu;
    CNewMenu_GetSite_body(this, riid, ppvSite)
}

pub static CNEWMENU_IOBJECTWITHSITE_VTBL: IObjectWithSiteVtbl = IObjectWithSiteVtbl {
    QueryInterface: CNewMenu_IObjectWithSite_QueryInterface,
    AddRef: CNewMenu_IObjectWithSite_AddRef,
    Release: CNewMenu_IObjectWithSite_Release,
    SetSite: CNewMenu_IObjectWithSite_SetSite,
    GetSite: CNewMenu_IObjectWithSite_GetSite,
};

unsafe extern "system" fn CNewMenu_IContextMenu3_QueryInterface(This: *mut IContextMenu3, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CNewMenu, IContextMenu3_iface)) as *mut CNewMenu;
    CNewMenu_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CNewMenu_IContextMenu3_AddRef(This: *mut IContextMenu3) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CNewMenu, IContextMenu3_iface)) as *mut CNewMenu;
    CNewMenu_AddRef(this)
}
unsafe extern "system" fn CNewMenu_IContextMenu3_Release(This: *mut IContextMenu3) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CNewMenu, IContextMenu3_iface)) as *mut CNewMenu;
    CNewMenu_Release(this)
}
unsafe extern "system" fn CNewMenu_IContextMenu3_QueryContextMenu(This: *mut IContextMenu3, hmenu: HMENU, indexMenu: UINT, idCmdFirst: UINT, idCmdLast: UINT, uFlags: UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CNewMenu, IContextMenu3_iface)) as *mut CNewMenu;
    CNewMenu_QueryContextMenu_body(this, hmenu, indexMenu, idCmdFirst, idCmdLast, uFlags)
}
unsafe extern "system" fn CNewMenu_IContextMenu3_InvokeCommand(This: *mut IContextMenu3, lpici: LPCMINVOKECOMMANDINFO) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CNewMenu, IContextMenu3_iface)) as *mut CNewMenu;
    CNewMenu_InvokeCommand_body(this, lpici)
}
unsafe extern "system" fn CNewMenu_IContextMenu3_GetCommandString(This: *mut IContextMenu3, idCmd: UINT_PTR, uType: UINT, pwReserved: *mut UINT, pszName: LPSTR, cchMax: UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CNewMenu, IContextMenu3_iface)) as *mut CNewMenu;
    CNewMenu_GetCommandString_body(this, idCmd, uType, pwReserved, pszName, cchMax)
}
unsafe extern "system" fn CNewMenu_IContextMenu3_HandleMenuMsg(This: *mut IContextMenu3, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CNewMenu, IContextMenu3_iface)) as *mut CNewMenu;
    CNewMenu_HandleMenuMsg_body(this, uMsg, wParam, lParam)
}
unsafe extern "system" fn CNewMenu_IContextMenu3_HandleMenuMsg2(This: *mut IContextMenu3, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, plResult: *mut LRESULT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CNewMenu, IContextMenu3_iface)) as *mut CNewMenu;
    CNewMenu_HandleMenuMsg2_body(this, uMsg, wParam, lParam, plResult)
}

pub static CNEWMENU_ICONTEXTMENU3_VTBL: IContextMenu3Vtbl = IContextMenu3Vtbl {
    QueryInterface: CNewMenu_IContextMenu3_QueryInterface,
    AddRef: CNewMenu_IContextMenu3_AddRef,
    Release: CNewMenu_IContextMenu3_Release,
    QueryContextMenu: CNewMenu_IContextMenu3_QueryContextMenu,
    InvokeCommand: CNewMenu_IContextMenu3_InvokeCommand,
    GetCommandString: CNewMenu_IContextMenu3_GetCommandString,
    HandleMenuMsg: CNewMenu_IContextMenu3_HandleMenuMsg,
    HandleMenuMsg2: CNewMenu_IContextMenu3_HandleMenuMsg2,
};

unsafe extern "system" fn CNewMenu_IContextMenu2_QueryInterface(This: *mut IContextMenu2, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CNewMenu, IContextMenu2_iface)) as *mut CNewMenu;
    CNewMenu_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CNewMenu_IContextMenu2_AddRef(This: *mut IContextMenu2) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CNewMenu, IContextMenu2_iface)) as *mut CNewMenu;
    CNewMenu_AddRef(this)
}
unsafe extern "system" fn CNewMenu_IContextMenu2_Release(This: *mut IContextMenu2) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CNewMenu, IContextMenu2_iface)) as *mut CNewMenu;
    CNewMenu_Release(this)
}
unsafe extern "system" fn CNewMenu_IContextMenu2_QueryContextMenu(This: *mut IContextMenu2, hmenu: HMENU, indexMenu: UINT, idCmdFirst: UINT, idCmdLast: UINT, uFlags: UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CNewMenu, IContextMenu2_iface)) as *mut CNewMenu;
    CNewMenu_QueryContextMenu_body(this, hmenu, indexMenu, idCmdFirst, idCmdLast, uFlags)
}
unsafe extern "system" fn CNewMenu_IContextMenu2_InvokeCommand(This: *mut IContextMenu2, lpici: LPCMINVOKECOMMANDINFO) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CNewMenu, IContextMenu2_iface)) as *mut CNewMenu;
    CNewMenu_InvokeCommand_body(this, lpici)
}
unsafe extern "system" fn CNewMenu_IContextMenu2_GetCommandString(This: *mut IContextMenu2, idCmd: UINT_PTR, uType: UINT, pwReserved: *mut UINT, pszName: LPSTR, cchMax: UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CNewMenu, IContextMenu2_iface)) as *mut CNewMenu;
    CNewMenu_GetCommandString_body(this, idCmd, uType, pwReserved, pszName, cchMax)
}
unsafe extern "system" fn CNewMenu_IContextMenu2_HandleMenuMsg(This: *mut IContextMenu2, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CNewMenu, IContextMenu2_iface)) as *mut CNewMenu;
    CNewMenu_HandleMenuMsg_body(this, uMsg, wParam, lParam)
}

pub static CNEWMENU_ICONTEXTMENU2_VTBL: IContextMenu2Vtbl = IContextMenu2Vtbl {
    QueryInterface: CNewMenu_IContextMenu2_QueryInterface,
    AddRef: CNewMenu_IContextMenu2_AddRef,
    Release: CNewMenu_IContextMenu2_Release,
    QueryContextMenu: CNewMenu_IContextMenu2_QueryContextMenu,
    InvokeCommand: CNewMenu_IContextMenu2_InvokeCommand,
    GetCommandString: CNewMenu_IContextMenu2_GetCommandString,
    HandleMenuMsg: CNewMenu_IContextMenu2_HandleMenuMsg,
};

unsafe extern "system" fn CNewMenu_IContextMenu_QueryInterface(This: *mut IContextMenu, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CNewMenu, IContextMenu_iface)) as *mut CNewMenu;
    CNewMenu_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CNewMenu_IContextMenu_AddRef(This: *mut IContextMenu) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CNewMenu, IContextMenu_iface)) as *mut CNewMenu;
    CNewMenu_AddRef(this)
}
unsafe extern "system" fn CNewMenu_IContextMenu_Release(This: *mut IContextMenu) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CNewMenu, IContextMenu_iface)) as *mut CNewMenu;
    CNewMenu_Release(this)
}
unsafe extern "system" fn CNewMenu_IContextMenu_QueryContextMenu(This: *mut IContextMenu, hmenu: HMENU, indexMenu: UINT, idCmdFirst: UINT, idCmdLast: UINT, uFlags: UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CNewMenu, IContextMenu_iface)) as *mut CNewMenu;
    CNewMenu_QueryContextMenu_body(this, hmenu, indexMenu, idCmdFirst, idCmdLast, uFlags)
}
unsafe extern "system" fn CNewMenu_IContextMenu_InvokeCommand(This: *mut IContextMenu, lpici: LPCMINVOKECOMMANDINFO) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CNewMenu, IContextMenu_iface)) as *mut CNewMenu;
    CNewMenu_InvokeCommand_body(this, lpici)
}
unsafe extern "system" fn CNewMenu_IContextMenu_GetCommandString(This: *mut IContextMenu, idCmd: UINT_PTR, uType: UINT, pwReserved: *mut UINT, pszName: LPSTR, cchMax: UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CNewMenu, IContextMenu_iface)) as *mut CNewMenu;
    CNewMenu_GetCommandString_body(this, idCmd, uType, pwReserved, pszName, cchMax)
}

pub static CNEWMENU_ICONTEXTMENU_VTBL: IContextMenuVtbl = IContextMenuVtbl {
    QueryInterface: CNewMenu_IContextMenu_QueryInterface,
    AddRef: CNewMenu_IContextMenu_AddRef,
    Release: CNewMenu_IContextMenu_Release,
    QueryContextMenu: CNewMenu_IContextMenu_QueryContextMenu,
    InvokeCommand: CNewMenu_IContextMenu_InvokeCommand,
    GetCommandString: CNewMenu_IContextMenu_GetCommandString,
};

unsafe extern "system" fn CNewMenu_IShellExtInit_QueryInterface(This: *mut IShellExtInit, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CNewMenu, IShellExtInit_iface)) as *mut CNewMenu;
    CNewMenu_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CNewMenu_IShellExtInit_AddRef(This: *mut IShellExtInit) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CNewMenu, IShellExtInit_iface)) as *mut CNewMenu;
    CNewMenu_AddRef(this)
}
unsafe extern "system" fn CNewMenu_IShellExtInit_Release(This: *mut IShellExtInit) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CNewMenu, IShellExtInit_iface)) as *mut CNewMenu;
    CNewMenu_Release(this)
}
unsafe extern "system" fn CNewMenu_IShellExtInit_Initialize(This: *mut IShellExtInit, pidlFolder: PCIDLIST_ABSOLUTE, pdtobj: *mut IDataObject, hkeyProgID: HKEY) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CNewMenu, IShellExtInit_iface)) as *mut CNewMenu;
    CNewMenu_Initialize_body(this, pidlFolder, pdtobj, hkeyProgID)
}

pub static CNEWMENU_ISHELLEXTINIT_VTBL: IShellExtInitVtbl = IShellExtInitVtbl {
    QueryInterface: CNewMenu_IShellExtInit_QueryInterface,
    AddRef: CNewMenu_IShellExtInit_AddRef,
    Release: CNewMenu_IShellExtInit_Release,
    Initialize: CNewMenu_IShellExtInit_Initialize,
};

unsafe fn CNewMenu_QueryInterface(
    this: *mut CNewMenu, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IUnknown)) || cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IObjectWithSite)) {
        *ppv = core::ptr::addr_of_mut!((*this).IObjectWithSite_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IContextMenu3)) {
        *ppv = core::ptr::addr_of_mut!((*this).IContextMenu3_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IContextMenu2)) {
        *ppv = core::ptr::addr_of_mut!((*this).IContextMenu2_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IContextMenu)) {
        *ppv = core::ptr::addr_of_mut!((*this).IContextMenu_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IShellExtInit)) {
        *ppv = core::ptr::addr_of_mut!((*this).IShellExtInit_iface) as *mut core::ffi::c_void;
    }
    else { return E_NOINTERFACE; }
    CNewMenu_AddRef(this);
    S_OK
}

unsafe fn CNewMenu_AddRef(this: *mut CNewMenu) -> ULONG {
    (*this).ref_count.fetch_add(1, core::sync::atomic::Ordering::SeqCst) as ULONG + 1
}

unsafe fn CNewMenu_Release(this: *mut CNewMenu) -> ULONG {
    let refs = (*this).ref_count.fetch_sub(1, core::sync::atomic::Ordering::SeqCst) - 1;
    if refs == 0 {
        CNewMenu_dtor_body(this);
        CNewMenu_cpp_com_assign_m_pSite(this, core::ptr::null_mut());
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

// Activation: CLSID_NewMenu currently maps to CNewMenu in shell32.cpp.
// The generated C++ adapter redirects ATL activation to this factory.
#[no_mangle] pub unsafe extern "system" fn CNewMenu_CreateInstance(
    outer: *mut IUnknown, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if !outer.is_null() { return CLASS_E_NOAGGREGATION; }
    let this = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<CNewMenu>()) as *mut CNewMenu;
    if this.is_null() { return E_OUTOFMEMORY; }
    core::ptr::write(this, CNewMenu {
        IObjectWithSite_iface: IObjectWithSite { lpVtbl: core::ptr::addr_of!(CNEWMENU_IOBJECTWITHSITE_VTBL) },
        IContextMenu3_iface: IContextMenu3 { lpVtbl: core::ptr::addr_of!(CNEWMENU_ICONTEXTMENU3_VTBL) },
        IContextMenu2_iface: IContextMenu2 { lpVtbl: core::ptr::addr_of!(CNEWMENU_ICONTEXTMENU2_VTBL) },
        IContextMenu_iface: IContextMenu { lpVtbl: core::ptr::addr_of!(CNEWMENU_ICONTEXTMENU_VTBL) },
        IShellExtInit_iface: IShellExtInit { lpVtbl: core::ptr::addr_of!(CNEWMENU_ISHELLEXTINIT_VTBL) },
        ref_count: core::sync::atomic::AtomicI32::new(1),
        m_pidlFolder: core::mem::zeroed(),
        m_pItems: core::mem::zeroed(),
        m_pLinkItem: core::mem::zeroed(),
        m_pSite: core::mem::zeroed(),
        m_hSubMenu: core::mem::zeroed(),
        m_idCmdFirst: core::mem::zeroed(),
        m_idCmdFolder: core::mem::zeroed(),
        m_idCmdLink: core::mem::zeroed(),
        m_bCustomIconFolder: core::mem::zeroed(),
        m_bCustomIconLink: core::mem::zeroed(),
        m_hIconFolder: core::mem::zeroed(),
        m_hIconLink: core::mem::zeroed(),
    });
    Shell32RustObjectLock();
    CNewMenu_ctor_body(this);
    let hr = CNewMenu_QueryInterface(this, riid, ppv);
    CNewMenu_Release(this);
    hr
}

// Deterministic C++ support closure selected from source identifiers.

pub const FCIDM_SHVIEW_NEW: UINT = 146;

pub const FCIDM_SHVIEW_NEWFOLDER: UINT = 0x7053;

pub const FCIDM_SHVIEW_NEWLINK: UINT = 0x7052;

pub const IDI_SHELL_FOLDER: UINT = 4;

pub const IDI_SHELL_SHORTCUT: UINT = 30;

pub const IDR_NEWMENU: UINT = 139;

pub const IDS_CREATEFILE_CAPTION: UINT = 126;

pub const IDS_CREATEFILE_DENIED: UINT = 127;

pub const IDS_NEWFOLDER: UINT = 149;

pub const IDS_NEWITEMFORMAT: UINT = 150;

pub const CREATE_NEW: DWORD = 1;

pub const DI_NORMAL: DWORD = 3;

pub const FILE_ATTRIBUTE_NORMAL: DWORD = 0x00000080;

pub const GENERIC_WRITE: DWORD = 0x40000000;

pub const IMAGE_ICON: DWORD = 1;

pub const KEY_READ: DWORD = 0x20019;

pub const KEY_WRITE: DWORD = 0x20006;

pub const LR_SHARED: DWORD = 32768;

pub const MB_ICONEXCLAMATION: DWORD = 0x30;

pub const MB_OK: DWORD = 0;

pub const MFS_ENABLED: DWORD = 0;

pub const MFT_SEPARATOR: DWORD = 0x800;

pub const MIIM_BITMAP: DWORD = 128;

pub const MIIM_DATA: DWORD = 32;

pub const MIIM_STATE: DWORD = 1;

pub const MIIM_STRING: DWORD = 64;

pub const MIIM_SUBMENU: DWORD = 4;

pub const ODT_MENU: DWORD = 1;

pub const REG_EXPAND_SZ: DWORD = 2;

pub const REG_MULTI_SZ: DWORD = 7;

pub const REG_OPTION_NON_VOLATILE: DWORD = 0;

pub const REG_SZ: DWORD = 1;

pub const SHCNE_CREATE: DWORD = 0x00000002;

pub const SHCNE_MKDIR: DWORD = 0x00000008;

pub const SHCNF_FLUSH: DWORD = 0x1000;

pub const SHCNF_PATHW: DWORD = 0x0005;

pub const SHGFI_ICON: DWORD = 256;

pub const SHGFI_SMALLICON: DWORD = 1;

pub const SHGFI_TYPENAME: DWORD = 1024;

pub const SHGFI_USEFILEATTRIBUTES: DWORD = 16;

pub const SM_CXMENUCHECK: DWORD = 71;

pub const SM_CXSMICON: DWORD = 49;

pub const SM_CYSMICON: DWORD = 50;

pub const SVSI_DESELECTOTHERS: DWORD = 0x00000004;

pub const SVSI_EDIT: DWORD = 0x00000003;

pub const SVSI_ENSUREVISIBLE: DWORD = 0x00000008;

pub const SVSI_FOCUSED: DWORD = 0x00000010;

pub const SVSI_SELECT: DWORD = 0x00000001;

pub const WM_DRAWITEM: DWORD = 43;

pub const WM_MEASUREITEM: DWORD = 44;

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
pub struct MEASUREITEMSTRUCT {
    pub CtlType: UINT,
    pub CtlID: UINT,
    pub itemID: UINT,
    pub itemWidth: UINT,
    pub itemHeight: UINT,
    pub itemData: ULONG_PTR,
}

#[repr(C)]
pub struct PROCESS_INFORMATION {
    pub hProcess: HANDLE,
    pub hThread: HANDLE,
    pub dwProcessId: DWORD,
    pub dwThreadId: DWORD,
}

#[repr(C)]
pub struct STARTUPINFOW {
    pub cb: DWORD,
    pub lpReserved: LPWSTR,
    pub lpDesktop: LPWSTR,
    pub lpTitle: LPWSTR,
    pub dwX: DWORD,
    pub dwY: DWORD,
    pub dwXSize: DWORD,
    pub dwYSize: DWORD,
    pub dwXCountChars: DWORD,
    pub dwYCountChars: DWORD,
    pub dwFillAttribute: DWORD,
    pub dwFlags: DWORD,
    pub wShowWindow: WORD,
    pub cbReserved2: WORD,
    pub lpReserved2: PBYTE,
    pub hStdInput: HANDLE,
    pub hStdOutput: HANDLE,
    pub hStdError: HANDLE,
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

pub static ShellNewKey_WIDE: [WCHAR; 82] = [0x0053, 0x004F, 0x0046, 0x0054, 0x0057, 0x0041, 0x0052, 0x0045, 0x005C, 0x004D, 0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057, 0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x005C, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069, 0x006F, 0x006E, 0x005C, 0x0045, 0x0078, 0x0070, 0x006C, 0x006F, 0x0072, 0x0065, 0x0072, 0x005C, 0x0044, 0x0069, 0x0073, 0x0063, 0x0061, 0x0072, 0x0064, 0x0061, 0x0062, 0x006C, 0x0065, 0x005C, 0x0050, 0x006F, 0x0073, 0x0074, 0x0053, 0x0065, 0x0074, 0x0075, 0x0070, 0x005C, 0x0053, 0x0068, 0x0065, 0x006C, 0x006C, 0x004E, 0x0065, 0x0077, 0x0000];
pub const ShellNewKey: LPCWSTR = ShellNewKey_WIDE.as_ptr();

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

pub type SVSIF = UINT;

pub type LPTBBUTTONSB = *mut core::ffi::c_void;

macro_rules! LOWORD { ($value:expr) => { (($value as usize) & 0xffff) as UINT }; }

#[inline]
const fn MAKEINTRESOURCE(value: UINT) -> LPCWSTR {
    value as usize as LPCWSTR
}

pub type LPFNSVADDPROPSHEETPAGE = Option<unsafe extern "system" fn(HPROPSHEETPAGE, LPARAM) -> BOOL>;

extern "system" {
    fn CloseHandle(handle: HANDLE) -> BOOL;
    fn CopyFileW(existing: LPCWSTR, new: LPCWSTR, fail_if_exists: BOOL) -> BOOL;
    fn CreateFileW(name: LPCWSTR, access: DWORD, share: DWORD, security: LPVOID, creation: DWORD, flags: DWORD, template: HANDLE) -> HANDLE;
    fn CreateMenu() -> HMENU;
    fn CreateProcessW(app: LPCWSTR, command: LPWSTR, process_attrs: LPCVOID, thread_attrs: LPCVOID, inherit: BOOL, flags: DWORD, environment: LPCVOID, directory: LPCWSTR, startup: *const STARTUPINFOW, info: *mut PROCESS_INFORMATION) -> BOOL;
    fn DestroyIcon(icon: HICON) -> BOOL;
    fn DrawIconEx(dc: HDC, x: INT, y: INT, icon: HICON, width: INT, height: INT, step: UINT, brush: HANDLE, flags: UINT) -> BOOL;
    fn ExpandEnvironmentStringsW(source: LPCWSTR, dest: LPWSTR, size: DWORD) -> DWORD;
    fn ExtractIconExW(file: LPCWSTR, index: INT, large: *mut HICON, small: *mut HICON, icons: UINT) -> UINT;
    fn GetMenuItemInfoW(menu: HMENU, item: UINT, by_position: BOOL, info: *mut MENUITEMINFOW) -> BOOL;
    fn GetSystemMetrics(index: INT) -> INT;
    fn IUnknown_QueryService(object: *mut IUnknown, service: REFGUID, riid: REFIID, out: *mut *mut core::ffi::c_void) -> HRESULT;
    #[link_name = "LoadImageW"] fn LoadImage(instance: HINSTANCE, name: LPCWSTR, kind: UINT, width: INT, height: INT, flags: UINT) -> HANDLE;
    fn MessageBoxW(owner: HWND, text: LPCWSTR, caption: LPCWSTR, kind: UINT) -> INT;
    fn PathYetAnotherMakeUniqueName(buffer: LPWSTR, path: LPCWSTR, short: LPCWSTR, long: LPCWSTR) -> BOOL;
    fn RegCloseKey(key: HKEY) -> LONG;
    #[link_name = "RegCreateKeyExW"] fn RegCreateKeyEx(key: HKEY, subkey: LPCWSTR, reserved: DWORD, class: LPWSTR, options: DWORD, access: REGSAM, security: LPVOID, result: PHKEY, disposition: LPDWORD) -> LONG;
    fn RegEnumKeyW(key: HKEY, index: DWORD, name: LPWSTR, capacity: DWORD) -> LONG;
    fn RegGetValueW(key: HKEY, subkey: LPCWSTR, value: LPCWSTR, flags: DWORD, kind: LPDWORD, data: LPVOID, bytes: LPDWORD) -> LONG;
    fn RegOpenKeyExW(key: HKEY, subkey: LPCWSTR, options: DWORD, access: REGSAM, result: PHKEY) -> LONG;
    fn RegQueryValueExW(key: HKEY, value: LPCWSTR, reserved: LPDWORD, kind: LPDWORD, data: PBYTE, bytes: LPDWORD) -> LONG;
    fn RegSetValueExW(key: HKEY, value: LPCWSTR, reserved: DWORD, kind: DWORD, data: *const BYTE, bytes: DWORD) -> LONG;
    fn SHChangeNotify(event: LONG, flags: UINT, item1: LPCVOID, item2: LPCVOID);
    fn SHCreateDirectory(owner: HWND, path: LPCWSTR) -> INT;
    fn SHGetFileInfoW(path: LPCWSTR, attributes: DWORD, info: *mut SHFILEINFOW, bytes: UINT, flags: UINT) -> DWORD_PTR;
    fn SHGetPathFromIDListW(pidl: LPCITEMIDLIST, path: LPWSTR) -> BOOL;
    fn SHILCreateFromPathW(path: LPCWSTR, pidl: *mut LPITEMIDLIST, attributes: LPDWORD) -> HRESULT;
    fn WideCharToMultiByte(page: UINT, flags: DWORD, wide: LPCWSTR, wide_len: INT, multi: LPSTR, multi_len: INT, default_char: LPCSTR, used_default: *mut BOOL) -> INT;
    fn WriteFile(file: HANDLE, buffer: LPCVOID, bytes: DWORD, written: LPDWORD, overlapped: LPVOID) -> BOOL;
    fn lstrlenW(text: LPCWSTR) -> INT;
    fn ILClone(pidl: LPCITEMIDLIST) -> LPITEMIDLIST;
    fn ILFindLastID(pidl: LPCITEMIDLIST) -> LPITEMIDLIST;
    fn ILFree(pidl: LPITEMIDLIST);
    fn InsertMenuItemW(menu: HMENU, item: UINT, by_position: BOOL, info: *const MENUITEMINFOW) -> BOOL;
}

extern "C" {
    fn HLM_GetIconW(index: INT, destination: LPWSTR, length: DWORD, icon_index: *mut INT) -> BOOL;
    #[link_name = "memcpy"] fn cpp_com_memcpy(dst: *mut core::ffi::c_void, src: *const core::ffi::c_void, bytes: usize) -> *mut core::ffi::c_void;
    fn malloc(bytes: usize) -> *mut ();
    fn wcsstr(haystack: LPCWSTR, needle: LPCWSTR) -> LPWSTR;
    fn _wcsdup(text: LPCWSTR) -> LPWSTR;
    fn _wcsicmp(left: LPCWSTR, right: LPCWSTR) -> INT;
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

trait CppComSize { fn cpp_com_size(self) -> usize; }
impl CppComSize for usize { fn cpp_com_size(self) -> usize { self } }
impl CppComSize for u32 { fn cpp_com_size(self) -> usize { self as usize } }
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
    #[link_name = "free"] fn cpp_com_free(memory: *mut core::ffi::c_void);
}
#[inline]
unsafe fn free<T>(memory: *mut T) { cpp_com_free(memory.cast()); }

extern "C" {
    static CLSID_NewMenu: GUID;
    static IID_IFolderView: GUID;
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

extern "system" {
    fn HeapReAlloc(heap: HANDLE, flags: DWORD, memory: LPVOID, bytes: usize) -> LPVOID;
    fn GetModuleHandleW(name: LPCWSTR) -> HINSTANCE;
    fn LoadStringW(instance: HINSTANCE, id: UINT, buffer: LPWSTR, capacity: INT) -> INT;
}

pub struct CStringW { buffer: LPWSTR, len: usize, capacity: usize }
impl CStringW {
    pub const fn new() -> Self { Self { buffer: core::ptr::null_mut(), len: 0, capacity: 0 } }
    pub unsafe fn from_resource(id: UINT) -> Self {
        let mut value = Self::new();
        let mut stack = [0u16; 256];
        let count = LoadStringW(
            GetModuleHandleW(core::ptr::null()), id, stack.as_mut_ptr(), stack.len() as INT,
        );
        if count > 0 { value.append_ptr_len(stack.as_ptr(), count as usize); }
        value
    }
    pub unsafe fn new_from_lpcwstr(text: LPCWSTR) -> Self {
        if (text as usize) <= 0xffff { return Self::from_resource(text as usize as UINT); }
        let mut value = Self::new(); value.append_ptr(text); value
    }
    pub fn as_ptr(&self) -> LPCWSTR {
        if self.buffer.is_null() { CPP_COM_EMPTY_WSTR.as_ptr() } else { self.buffer }
    }
    pub fn as_mut_ptr(&mut self) -> LPWSTR { self.as_ptr() as LPWSTR }
    unsafe fn reserve(&mut self, needed: usize) -> bool {
        if needed <= self.capacity { return true; }
        let capacity = needed.next_power_of_two().max(16);
        let bytes = capacity * core::mem::size_of::<WCHAR>();
        let next = if self.buffer.is_null() {
            HeapAlloc(GetProcessHeap(), 0, bytes)
        } else {
            HeapReAlloc(GetProcessHeap(), 0, self.buffer.cast(), bytes)
        } as LPWSTR;
        if next.is_null() { return false; }
        self.buffer = next; self.capacity = capacity; true
    }
    unsafe fn append_ptr_len(&mut self, text: LPCWSTR, count: usize) {
        if text.is_null() || !self.reserve(self.len + count + 1) { return; }
        let _ = memcpy(
            self.buffer.add(self.len), text,
            count * core::mem::size_of::<WCHAR>(),
        );
        self.len += count; *self.buffer.add(self.len) = 0;
    }
    unsafe fn append_ptr(&mut self, text: LPCWSTR) {
        if !text.is_null() { self.append_ptr_len(text, lstrlenW(text) as usize); }
    }
    unsafe fn append_char(&mut self, ch: WCHAR) { self.append_ptr_len(core::ptr::addr_of!(ch), 1); }
    pub unsafe fn concat(&mut self, text: LPCWSTR) { self.append_ptr(text); }
    pub unsafe fn format(&mut self, format: LPCWSTR, arg: LPCWSTR) {
        let mut resource = [0u16; 256];
        let format = if (format as usize) <= 0xffff {
            LoadStringW(GetModuleHandleW(core::ptr::null()), format as usize as UINT,
                resource.as_mut_ptr(), resource.len() as INT);
            resource.as_ptr()
        } else { format };
        self.len = 0;
        if self.reserve(512) {
            let _ = StringCbPrintfW!(self.buffer, self.capacity * 2, format, arg);
            self.len = lstrlenW(self.buffer) as usize;
        }
    }
    pub unsafe fn format_message(&mut self, arg: LPCWSTR) {
        let mut format = [0u16; 512];
        let count = lstrlenW(self.as_ptr()).max(0) as usize;
        let count = count.min(format.len() - 1);
        let _ = memcpy(format.as_mut_ptr(), self.as_ptr(), count * 2);
        self.format(format.as_ptr(), arg);
    }
}
impl core::ops::AddAssign<LPCWSTR> for CStringW {
    fn add_assign(&mut self, rhs: LPCWSTR) { unsafe { self.append_ptr(rhs) } }
}
impl core::ops::AddAssign<WCHAR> for CStringW {
    fn add_assign(&mut self, rhs: WCHAR) { unsafe { self.append_char(rhs) } }
}
impl Drop for CStringW {
    fn drop(&mut self) {
        unsafe {
            if !self.buffer.is_null() { HeapFree(GetProcessHeap(), 0, self.buffer); }
        }
    }
}
static CPP_COM_EMPTY_WSTR: [WCHAR; 1] = [0];

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
pub static CPP_COM_WSTR_05F3C233D307: [WCHAR; 5] = [0x0044, 0x0061, 0x0074, 0x0061, 0x0000];
pub static CPP_COM_WSTR_1FD85BA5E382: [WCHAR; 8] = [0x0043, 0x006C, 0x0061, 0x0073, 0x0073, 0x0065, 0x0073, 0x0000];
pub static CPP_COM_WSTR_318FCA4E3B28: [WCHAR; 9] = [0x0046, 0x0069, 0x006C, 0x0065, 0x004E, 0x0061, 0x006D, 0x0065, 0x0000];
pub static CPP_COM_WSTR_425FC060B288: [WCHAR; 8] = [0x0043, 0x006F, 0x006D, 0x006D, 0x0061, 0x006E, 0x0064, 0x0000];
pub static CPP_COM_WSTR_74002357F85B: [WCHAR; 12] = [0x0025, 0x0073, 0x005C, 0x0053, 0x0068, 0x0065, 0x006C, 0x006C, 0x004E, 0x0065, 0x0077, 0x0000];
pub static CPP_COM_WSTR_7C8F589C7B27: [WCHAR; 3] = [0x0025, 0x0031, 0x0000];
pub static CPP_COM_WSTR_9FEA0545564B: [WCHAR; 15] = [0x0025, 0x0073, 0x005C, 0x0025, 0x0073, 0x005C, 0x0053, 0x0068, 0x0065, 0x006C, 0x006C, 0x004E, 0x0065, 0x0077, 0x0000];
pub static CPP_COM_WSTR_F4D7F37FC85E: [WCHAR; 9] = [0x004E, 0x0075, 0x006C, 0x006C, 0x0046, 0x0069, 0x006C, 0x0065, 0x0000];
pub static CPP_COM_WSTR_FE706C396A0C: [WCHAR; 5] = [0x002E, 0x006C, 0x006E, 0x006B, 0x0000];

use core::ffi::c_void;
use crate::shellcom::*;
use crate::types::*;

pub unsafe fn CNewMenu_ctor_body(this: *mut CNewMenu) -> () {
    (*this).m_pidlFolder = core::ptr::null_mut();
    (*this).m_pItems = core::ptr::null_mut();
    (*this).m_pLinkItem = core::ptr::null_mut();
    (*this).m_pSite = core::ptr::null_mut();
    (*this).m_idCmdFirst = 0;
    (*this).m_idCmdFolder = UINT::MAX;
    (*this).m_idCmdLink = UINT::MAX;
    (*this).m_bCustomIconFolder = FALSE;
    (*this).m_bCustomIconLink = FALSE;
    (*this).m_hIconFolder = core::ptr::null_mut();
    (*this).m_hIconLink = core::ptr::null_mut();
}

pub unsafe fn CNewMenu_dtor_body(this: *mut CNewMenu) -> () {
    CNewMenu_UnloadAllItems_body(this);
    if (*this).m_bCustomIconFolder != 0 && !(*this).m_hIconFolder.is_null() {
        DestroyIcon((*this).m_hIconFolder);
    }
    if (*this).m_bCustomIconLink != 0 && !(*this).m_hIconLink.is_null() {
        DestroyIcon((*this).m_hIconLink);
    }
    if !(*this).m_pidlFolder.is_null() {
        ILFree((*this).m_pidlFolder);
    }
}

pub unsafe fn CNewMenu_UnloadItem_body(this: *mut CNewMenu, pItem: *mut SHELLNEW_ITEM) -> () {
    free((*pItem).pData);
    free((*pItem).pwszDesc);
    free((*pItem).pwszExt);

    if !(*pItem).hIcon.is_null() {
        DestroyIcon((*pItem).hIcon);
    }

    HeapFree(GetProcessHeap(), 0, pItem);
}

pub unsafe fn CNewMenu_UnloadAllItems_body(this: *mut CNewMenu) -> () {
    while !(*this).m_pItems.is_null() {
        let pCurItem = (*this).m_pItems;
        (*this).m_pItems = (*pCurItem).pNext;
        CNewMenu_UnloadItem_body(this, pCurItem);
    }
    (*this).m_pItems = core::ptr::null_mut();
    (*this).m_pLinkItem = core::ptr::null_mut();
}

pub unsafe fn CNewMenu_LoadItem_body(this: *mut CNewMenu, pwszExt: LPCWSTR) -> *mut SHELLNEW_ITEM {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut wszBuf: [u16; MAX_PATH] = [0u16; MAX_PATH];
    let mut pData: *mut u8 = core::ptr::null_mut();
    let mut cbData: u32 = 0;

    StringCbPrintfW!(
        wszBuf.as_mut_ptr().cast(),
        core::mem::size_of_val(&wszBuf) as usize,
        CPP_COM_WSTR_74002357F85B.as_ptr(),
        pwszExt,
    );

    if RegOpenKeyExW(
        HKEY_CLASSES_ROOT,
        wszBuf.as_ptr(),
        0,
        KEY_READ,
        &mut hKey as *mut HKEY,
    ) != ERROR_SUCCESS as i32
    {
        let mut wszValue: [u16; MAX_PATH] = [0u16; MAX_PATH];
        cbData = core::mem::size_of_val(&wszValue) as u32;
        if RegGetValueW(
            HKEY_CLASSES_ROOT,
            pwszExt,
            core::ptr::null_mut(),
            RRF_RT_REG_SZ,
            core::ptr::null_mut(),
            wszValue.as_mut_ptr().cast(),
            &mut cbData,
        ) != ERROR_SUCCESS as i32
            || wszValue[0 as usize] == 0
        {
            return core::ptr::null_mut();
        }
        wszValue[wszValue.len() - 1] = UNICODE_NULL as u16;
        StringCbPrintfW!(
            wszBuf.as_mut_ptr(),
            core::mem::size_of_val(&wszBuf) as usize,
            CPP_COM_WSTR_9FEA0545564B.as_ptr(),
            pwszExt,
            wszValue.as_ptr(),
        );
        if RegOpenKeyExW(
            HKEY_CLASSES_ROOT,
            wszBuf.as_ptr(),
            0,
            KEY_READ,
            &mut hKey as *mut HKEY,
        ) != ERROR_SUCCESS as i32
        {
            return core::ptr::null_mut();
        }
    }

    struct TypeInfo {
        pszName: LPCWSTR,
        Type: SHELLNEW_TYPE,
        bNeedData: BOOL,
        bStr: BOOL,
    }
    let Types: [TypeInfo; 5] = [
        TypeInfo {
            pszName: CPP_COM_WSTR_318FCA4E3B28.as_ptr(),
            Type: SHELLNEW_TYPE_FILENAME,
            bNeedData: TRUE,
            bStr: TRUE,
        },
        TypeInfo {
            pszName: CPP_COM_WSTR_425FC060B288.as_ptr(),
            Type: SHELLNEW_TYPE_COMMAND,
            bNeedData: TRUE,
            bStr: TRUE,
        },
        TypeInfo {
            pszName: CPP_COM_WSTR_05F3C233D307.as_ptr(),
            Type: SHELLNEW_TYPE_DATA,
            bNeedData: TRUE,
            bStr: FALSE,
        },
        TypeInfo {
            pszName: CPP_COM_WSTR_F4D7F37FC85E.as_ptr(),
            Type: SHELLNEW_TYPE_NULLFILE,
            bNeedData: FALSE,
            bStr: FALSE,
        },
        TypeInfo {
            pszName: core::ptr::null_mut(),
            Type: 0 as SHELLNEW_TYPE,
            bNeedData: FALSE,
            bStr: FALSE,
        },
    ];

    let mut i: usize = 0 as usize;
    while i < Types.len() && !Types[i].pszName.is_null() {
        cbData = 0;
        let dwFlags: u32 = if Types[i].bStr != 0 {
            RRF_RT_REG_SZ
        } else {
            RRF_RT_ANY
        };
        let mut dwType: u32 = 0;
        if RegGetValueW(
            hKey,
            core::ptr::null_mut(),
            Types[i].pszName,
            dwFlags,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &mut cbData,
        ) == ERROR_SUCCESS as i32
        {
            if Types[i].bNeedData != 0 && cbData > 0 {
                pData = malloc(cbData as usize) as *mut u8;
                RegGetValueW(
                    hKey,
                    core::ptr::null_mut(),
                    Types[i].pszName,
                    dwFlags,
                    &mut dwType,
                    pData.cast(),
                    &mut cbData,
                );
                if Types[i].bStr == 0
                    && (dwType == REG_SZ || dwType == REG_EXPAND_SZ)
                {
                    let pData2: *mut u8 =
                        malloc(cbData as usize) as *mut u8;
                    cbData = WideCharToMultiByte(
                        CP_ACP,
                        0,
                        pData as *const u16,
                        -(1 as i32),
                        pData2 as *mut i8,
                        cbData as i32,
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                    ) as u32;
                    free(pData as *mut core::ffi::c_void);
                    pData = pData2;
                }
            }
            break;
        }
        i += 1;
    }

    RegCloseKey(hKey);

    if i >= Types.len() || Types[i].pszName.is_null() {
        if !pData.is_null() {
            free(pData as *mut core::ffi::c_void);
        }
        return core::ptr::null_mut();
    }

    let mut fi: SHFILEINFOW = core::mem::zeroed();
    if SHGetFileInfoW(
        pwszExt,
        FILE_ATTRIBUTE_NORMAL,
        &mut fi,
        core::mem::size_of::<SHFILEINFOW>() as u32,
        SHGFI_USEFILEATTRIBUTES
            | SHGFI_TYPENAME
            | SHGFI_ICON
            | SHGFI_SMALLICON,
    ) == 0
    {
        if !pData.is_null() {
            free(pData as *mut core::ffi::c_void);
        }
        return core::ptr::null_mut();
    }

    let alloc_size = core::mem::size_of::<SHELLNEW_ITEM>();
    let pNewItem: *mut SHELLNEW_ITEM = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        alloc_size,
    ) as *mut SHELLNEW_ITEM;
    if pNewItem.is_null() {
        if !pData.is_null() {
            free(pData as *mut core::ffi::c_void);
        }
        return core::ptr::null_mut();
    }

    let pNewItem_shell = pNewItem as *mut SHELLNEW_ITEM;
    (*pNewItem_shell).Type = Types[i].Type;
    (*pNewItem_shell).pData = pData;
    (*pNewItem_shell).cbData = if pData.is_null() {
        0
    } else {
        cbData
    };
    (*pNewItem_shell).pwszExt = _wcsdup(pwszExt);
    (*pNewItem_shell).pwszDesc = _wcsdup(fi.szTypeName.as_ptr());
    if !fi.hIcon.is_null() {
        (*pNewItem_shell).hIcon = fi.hIcon;
    }

    pNewItem_shell
}

pub unsafe fn CNewMenu_CacheItems_body(this: *mut CNewMenu) -> BOOL {
    let mut dwSize: u32 = 0;
    let mut dwIndex: u32 = 0;
    let mut wszName: [u16; MAX_PATH as usize] = [0u16; MAX_PATH as usize];
    let mut pCurItem: *mut SHELLNEW_ITEM = core::ptr::null_mut();

    // Enumerate all extensions
    loop {
        let old_index = dwIndex;
        dwIndex = dwIndex.wrapping_add(1);
        let enum_result = RegEnumKeyW(
            HKEY_CLASSES_ROOT,
            old_index,
            wszName.as_mut_ptr(), MAX_PATH as DWORD,
        );
        if enum_result != ERROR_SUCCESS {
            break;
        }
        // Check if first character is '.'
        if wszName[0 as usize] != 0x002Eu16 {
            continue;
        }

        let pNewItem: *mut SHELLNEW_ITEM = CNewMenu_LoadItem_body(this, wszName.as_ptr());
        if !pNewItem.is_null() {
            dwSize = dwSize.wrapping_add(wcslen(wszName.as_ptr()) as u32).wrapping_add(1);
            if (*this).m_pLinkItem.is_null()
                && _wcsicmp((*pNewItem).pwszExt, CPP_COM_WSTR_FE706C396A0C.as_ptr()) == 0
            {
                // The unique link handler
                (*this).m_pLinkItem = pNewItem;
            }

            // Add at the end of the list
            if !pCurItem.is_null() {
                (*pCurItem).pNext = pNewItem;
                pCurItem = pNewItem;
            } else {
                pCurItem = pNewItem;
                (*this).m_pItems = pNewItem;
            }
        }
    }

    dwSize = dwSize.wrapping_add(1);

    let heap = GetProcessHeap();
    let lpValues: *mut u16 = HeapAlloc(
        heap,
        HEAP_ZERO_MEMORY,
        dwSize.wrapping_mul(core::mem::size_of::<u16>() as u32),
    ) as *mut u16;
    if lpValues.is_null() {
        return FALSE;
    }

    // Build multi-string from items
    let mut pCurItem2: *mut SHELLNEW_ITEM = (*this).m_pItems;
    let mut lpValue: *mut u16 = lpValues;
    while !pCurItem2.is_null() {
        let ext = (*pCurItem2).pwszExt;
        let len = wcslen(ext) as u32;
        let byte_count = (len.wrapping_add(1)).wrapping_mul(core::mem::size_of::<u16>() as u32);
        memcpy(lpValue as *mut _, ext as *const _, byte_count);
        lpValue = lpValue.add((wcslen(ext) + 1) as usize);
        pCurItem2 = (*pCurItem2).pNext;
    }

    // Create or open registry key
    let mut hKey: HKEY = core::ptr::null_mut();
    if RegCreateKeyEx(
        HKEY_CURRENT_USER,
        ShellNewKey,
        0,
        core::ptr::null_mut(),
        REG_OPTION_NON_VOLATILE,
        KEY_WRITE,
        core::ptr::null_mut(),
        &mut hKey,
        core::ptr::null_mut(),
    ) != ERROR_SUCCESS
    {
        HeapFree(GetProcessHeap(), 0, lpValues as *mut _);
        return FALSE;
    }

    // Set the cached multi-string value
    if RegSetValueExW(
        hKey,
        CPP_COM_WSTR_1FD85BA5E382.as_ptr(),
        0,
        REG_MULTI_SZ,
        lpValues as *const u8,
        dwSize.wrapping_mul(core::mem::size_of::<u16>() as u32),
    ) != ERROR_SUCCESS
    {
        HeapFree(GetProcessHeap(), 0, lpValues as *mut _);
        RegCloseKey(hKey);
        return FALSE;
    }

    // Success: free buffer and close key
    HeapFree(GetProcessHeap(), 0, lpValues as *mut _);
    RegCloseKey(hKey);

    TRUE
}

pub unsafe fn CNewMenu_LoadCachedItems_body(this: *mut CNewMenu) -> BOOL {
    let mut wszName: *mut u16;
    let lpValues: *mut u16;
    let mut dwSize: u32 = 0;
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut pNewItem: *mut SHELLNEW_ITEM;
    let mut pCurItem: *mut SHELLNEW_ITEM = core::ptr::null_mut();

    if RegOpenKeyExW(
        HKEY_CURRENT_USER,
        ShellNewKey,
        0,
        KEY_READ,
        &mut hKey,
    ) != ERROR_SUCCESS
    {
        return FALSE;
    }

    if RegQueryValueExW(
        hKey,
        CPP_COM_WSTR_1FD85BA5E382.as_ptr(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &mut dwSize,
    ) != ERROR_SUCCESS
    {
        return FALSE;
    }

    lpValues = HeapAlloc(
        GetProcessHeap(),
        0,
        dwSize,
    ) as *mut u16;
    if lpValues.is_null() {
        return FALSE;
    }

    if RegQueryValueExW(
        hKey,
        CPP_COM_WSTR_1FD85BA5E382.as_ptr(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        lpValues as *mut u8,
        &mut dwSize,
    ) != ERROR_SUCCESS
    {
        HeapFree(
            GetProcessHeap(),
            0,
            lpValues,
        );
        return FALSE;
    }

    wszName = lpValues;
    loop {
        if *wszName == 0 {
            break;
        }
        pNewItem = CNewMenu_LoadItem_body(this, wszName);
        if !pNewItem.is_null() {
            if (*this).m_pLinkItem.is_null()
                && _wcsicmp(
                    (*pNewItem).pwszExt,
                    CPP_COM_WSTR_FE706C396A0C.as_ptr(),
                ) == 0
            {
                (*this).m_pLinkItem = pNewItem;
            }

            if !pCurItem.is_null() {
                (*pCurItem).pNext = pNewItem;
                pCurItem = pNewItem;
            } else {
                pCurItem = pNewItem;
                (*this).m_pItems = pNewItem;
            }
        }

        let len = wcslen(wszName);
        wszName = wszName.offset(len as isize + 1 as isize);
    }

    HeapFree(
        GetProcessHeap(),
        0,
        lpValues,
    );
    RegCloseKey(hKey);

    TRUE
}

pub unsafe fn CNewMenu_LoadAllItems_body(this: *mut CNewMenu) -> BOOL {
    // TODO: We need to find a way to refresh the cache from time to time, when
    // e.g. new extensions with ShellNew handlers have been added or removed.

    /* If there are any unload them */
    CNewMenu_UnloadAllItems_body(this);

    if CNewMenu_CacheItems_body(this) == 0 {
        CNewMenu_LoadCachedItems_body(this);
    }

    ((*this).m_pItems != core::ptr::null_mut()) as BOOL
}

pub unsafe fn CNewMenu_InsertShellNewItems_body(this: *mut CNewMenu, hMenu: HMENU, idCmdFirst: UINT, mut Pos: UINT) -> UINT {
    let mut mii = core::mem::MaybeUninit::<MENUITEMINFOW>::uninit();
    let mut idCmd = idCmdFirst;
    let mut wszBuf: [u16; 256] = [0; 256];

    if (*this).m_pItems.is_null() {
        if CNewMenu_LoadAllItems_body(this) == FALSE {
            return 0;
        }
    }

    ZeroMemory(
        mii.as_mut_ptr() as *mut core::ffi::c_void,
        ::core::mem::size_of::<MENUITEMINFOW>(),
    );
    let mut mii = mii.assume_init();
    mii.cbSize = ::core::mem::size_of::<MENUITEMINFOW>() as UINT;

    (*this).m_idCmdFirst = idCmd;

    /* Insert the new folder action */
    if LoadStringW(
        shell32_hInstance,
        FCIDM_SHVIEW_NEWFOLDER,
        wszBuf.as_mut_ptr(),
        wszBuf.len() as INT,
    ) == 0
    {
        wszBuf[0 as usize] = 0 as u16;
    }
    mii.fMask = MIIM_ID | MIIM_BITMAP | MIIM_STRING;
    mii.dwTypeData = wszBuf.as_mut_ptr();
    mii.cch = wcslen(mii.dwTypeData) as UINT;
    mii.wID = idCmd;
    mii.hbmpItem = HBMMENU_CALLBACK as _;
    let old_pos = Pos;
    Pos += 1;
    if InsertMenuItemW(hMenu, old_pos, TRUE, &mut mii) != 0 {
        (*this).m_idCmdFolder = idCmd;
        idCmd += 1;
    }

    /* Insert the new shortcut action */
    if !(*this).m_pLinkItem.is_null() {
        if LoadStringW(
            shell32_hInstance,
            FCIDM_SHVIEW_NEWLINK,
            wszBuf.as_mut_ptr(),
            wszBuf.len() as INT,
        ) == 0
        {
            wszBuf[0 as usize] = 0 as u16;
        }
        mii.dwTypeData = wszBuf.as_mut_ptr();
        mii.cch = wcslen(mii.dwTypeData) as UINT;
        mii.wID = idCmd;
        let old_pos = Pos;
        Pos += 1;
        if InsertMenuItemW(hMenu, old_pos, TRUE, &mut mii) != 0 {
            (*this).m_idCmdLink = idCmd;
            idCmd += 1;
        }
    }

    /* Insert a separator for the custom new action */
    mii.fMask = MIIM_TYPE | MIIM_ID;
    mii.fType = MFT_SEPARATOR;
    mii.wID = (- (1 as i32)) as UINT;
    let old_pos = Pos;
    Pos += 1;
    InsertMenuItemW(hMenu, old_pos, TRUE, &mut mii);

    /* Insert the rest of the items */
    mii.fMask = MIIM_ID
        | MIIM_BITMAP
        | MIIM_STRING
        | MIIM_DATA;
    mii.fType = 0;

    let mut pCurItem = (*this).m_pItems;
    while !pCurItem.is_null() {
        /* Skip shortcut item */
        if pCurItem == (*this).m_pLinkItem {
            pCurItem = (*pCurItem).pNext;
            continue;
        }

        mii.dwItemData = pCurItem as ULONG_PTR;
        mii.dwTypeData = (*pCurItem).pwszDesc;
        mii.cch = wcslen(mii.dwTypeData) as UINT;
        mii.wID = idCmd;
        let old_pos = Pos;
        Pos += 1;
        if InsertMenuItemW(hMenu, old_pos, TRUE, &mut mii) != 0 {
            idCmd += 1;
        }

        pCurItem = (*pCurItem).pNext;
    }

    idCmd - idCmdFirst
}

pub unsafe fn CNewMenu_FindItemFromIdOffset_body(this: *mut CNewMenu, IdOffset: UINT) -> *mut SHELLNEW_ITEM {
    // Folder
    if (*this).m_idCmdFirst + IdOffset == (*this).m_idCmdFolder {
        return core::ptr::null_mut();
    }

    // Shortcut
    if (*this).m_idCmdFirst + IdOffset == (*this).m_idCmdLink {
        return (*this).m_pLinkItem;
    }

    // Find shell new item - Retrieve menu item info
    let mut mii: MENUITEMINFOW = core::mem::zeroed();
    ZeroMemory(
        &mut mii as *mut _ as *mut core::ffi::c_void,
        core::mem::size_of::<MENUITEMINFOW>(),
    );
    mii.cbSize = core::mem::size_of::<MENUITEMINFOW>() as u32;
    mii.fMask = MIIM_DATA;

    if GetMenuItemInfoW(
        (*this).m_hSubMenu,
        (*this).m_idCmdFirst + IdOffset,
        FALSE,
        &mut mii,
    ) != 0 && mii.dwItemData != 0
    {
        mii.dwItemData as *mut SHELLNEW_ITEM
    } else {
        core::ptr::null_mut()
    }
}

pub unsafe fn CNewMenu_SelectNewItem_body(this: *mut CNewMenu, wEventId: LONG, uFlags: UINT, pszName: LPWSTR, bRename: BOOL) -> HRESULT {
    // CComPtr<IShellBrowser> lpSB is declared but never used in the original body.
    let mut lpSB: *mut IShellBrowser = core::ptr::null_mut();
    let _lpSB_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(lpSB));
    let _lpSB_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(lpSB));
    let mut lpSV: *mut IShellView = core::ptr::null_mut();
    let _lpSV_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(lpSV));
    let _lpSV_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(lpSV));
    let mut hr: HRESULT = E_FAIL;
    let mut dwSelectFlags: u32 = SVSI_DESELECTOTHERS
        | SVSI_ENSUREVISIBLE
        | SVSI_FOCUSED
        | SVSI_SELECT;
    if bRename != 0 {
        dwSelectFlags |= SVSI_EDIT;
    }

    // SHChangeNotify(wEventId, uFlags | ..., pszName, NULL)
    SHChangeNotify(
        wEventId,
        uFlags | SHCNF_FLUSH,
        pszName as *const ::core::ffi::c_void,
        ::core::ptr::null_mut(),
    );

    // If there is no site object, return failure.
    if (*this).m_pSite.is_null() {
        return S_OK;
    }

    // IUnknown_QueryService(m_pSite, IID_IFolderView, IID_PPV_ARG(IShellView, &lpSV))

    hr = IUnknown_QueryService((*this).m_pSite, &IID_IFolderView, core::ptr::addr_of!(IID_IShellView), core::ptr::addr_of_mut!(lpSV).cast());
    if hr < 0 {
        return S_OK;
    }

    // SHILCreateFromPathW(pszName, &pidl, NULL)
    let mut pidl: *mut ITEMIDLIST = ::core::ptr::null_mut();
    hr = SHILCreateFromPathW(pszName, &mut pidl as *mut _, ::core::ptr::null_mut());
    if hr < 0 {
        return hr;
    }

    let pidlNewItem = ILFindLastID(pidl) as PITEMID_CHILD;

    // lpSV->SelectItem(pidlNewItem, dwSelectFlags)
    let psv: *mut IShellView = lpSV;
    hr = ((*(*psv).lpVtbl).SelectItem)(psv, pidlNewItem, dwSelectFlags);

    // SHFree(pidl)
    SHFree(pidl);

    hr
}

pub unsafe fn CNewMenu_CreateNewFolder_body(this: *mut CNewMenu, lpici: LPCMINVOKECOMMANDINFO) -> HRESULT {
    let mut wszPath: [u16; MAX_PATH] = [0u16; MAX_PATH];
    let mut wszName: [u16; MAX_PATH] = [0u16; MAX_PATH];
    let mut wszNewFolder: [u16; 25] = [0u16; 25];
    let hr: HRESULT = SHGetPathFromIDListW((*this).m_pidlFolder, wszPath.as_mut_ptr());
    if ((hr) < 0) {
        return hr;
    }
    if LoadStringW(shell32_hInstance, IDS_NEWFOLDER, wszNewFolder.as_mut_ptr(), 25) == 0 {
        return E_FAIL;
    }
    if PathYetAnotherMakeUniqueName(wszName.as_mut_ptr(), wszPath.as_ptr(), ::core::ptr::null(), wszNewFolder.as_ptr()) == 0 {
        return E_FAIL;
    }
    if SHCreateDirectory((*lpici).hwnd, wszName.as_mut_ptr()) != ERROR_SUCCESS {
        return E_FAIL;
    }
    CNewMenu_SelectNewItem_body(this, SHCNE_MKDIR as LONG, SHCNF_PATHW, wszName.as_mut_ptr(), TRUE);
    S_OK
}

pub unsafe fn CNewMenu_NewItemByCommand_body(this: *mut CNewMenu, pItem: *mut SHELLNEW_ITEM, wszPath: LPCWSTR) -> HRESULT {
    // WCHAR wszBuf[MAX_PATH];
    let mut wszBuf: [u16; MAX_PATH] = [0u16; MAX_PATH];
    // WCHAR wszTemp[MAX_PATH];
    let mut wszTemp: [u16; MAX_PATH] = [0u16; MAX_PATH];
    // STARTUPINFOW si; PROCESS_INFORMATION pi;
    let mut si: STARTUPINFOW = core::mem::zeroed();
    let mut pi: PROCESS_INFORMATION = core::mem::zeroed();

    let pData_ptr: *mut u16 = (*pItem).pData as *mut u16;

    // ExpandEnvironmentStringsW((LPWSTR)pItem->pData, wszBuf, wszBuf.len())
    if ExpandEnvironmentStringsW(pData_ptr, wszBuf.as_mut_ptr(), MAX_PATH as DWORD) == 0 {
        return E_FAIL;
    }

    // Ptr = wcsstr(wszBuf, CPP_COM_WSTR_7C8F589C7B27);
    let ptr: *mut u16 = wcsstr(wszBuf.as_ptr(), CPP_COM_WSTR_7C8F589C7B27.as_ptr()) as *mut u16;
    let pwszCmd: *const u16;
    if !ptr.is_null() {
        // Ptr[1] = 0x0073u16;
        *ptr.add(1) = 0x0073u16;
        // StringCbPrintfW!(wszTemp, sizeof(wszTemp), wszBuf, wszPath);
        StringCbPrintfW!(
            wszTemp.as_mut_ptr(),
            core::mem::size_of_val(&wszTemp),
            wszBuf.as_ptr(),
            wszPath,
        );
        pwszCmd = wszTemp.as_ptr();
    } else {
        pwszCmd = wszBuf.as_ptr();
    }

    // ZeroMemory(&si, sizeof(si));
    ZeroMemory(core::ptr::addr_of_mut!(si), core::mem::size_of_val(&si));
    si.cb = core::mem::size_of_val(&si) as u32;

    // CreateProcessW(NULL, pwszCmd, NULL, NULL, FALSE, 0, NULL, NULL, &si, &pi)
    if CreateProcessW(
        core::ptr::null(),
        pwszCmd as *mut u16,
        core::ptr::null(),
        core::ptr::null(),
        FALSE,
        0,
        core::ptr::null(),
        core::ptr::null(),
        core::ptr::addr_of!(si),
        core::ptr::addr_of_mut!(pi),
    ) != 0
    {
        // CloseHandle(pi.hProcess)
        CloseHandle(pi.hProcess);
        // CloseHandle(pi.hThread)
        CloseHandle(pi.hThread);
        S_OK
    } else {
        E_FAIL
    }
}

pub unsafe fn CNewMenu_NewItemByNonCommand_body(this: *mut CNewMenu, pItem: *mut SHELLNEW_ITEM, wszName: LPWSTR, cchNameMax: DWORD, wszPath: LPCWSTR) -> HRESULT {
    let mut bSuccess: BOOL = TRUE;

    let mut str_new_item = CStringW::new();
    str_new_item.format(
        IDS_NEWITEMFORMAT as *const u16,
        (*pItem).pwszDesc,
    );
    str_new_item.concat((*pItem).pwszExt);

    if PathYetAnotherMakeUniqueName(wszName, wszPath, core::ptr::null_mut(), str_new_item.as_ptr()) == 0 {
        return E_FAIL;
    }

    let hFile = CreateFileW(
        wszName,
        GENERIC_WRITE,
        0,
        core::ptr::null_mut(),
        CREATE_NEW,
        FILE_ATTRIBUTE_NORMAL,
        core::ptr::null_mut(),
    );
    if hFile != INVALID_HANDLE_VALUE {
        if (*pItem).Type == SHELLNEW_TYPE_DATA {
            let mut cbWritten: DWORD = 0;
            WriteFile(
                hFile,
                (*pItem).pData as *const core::ffi::c_void,
                (*pItem).cbData,
                &mut cbWritten,
                core::ptr::null_mut(),
            );
        }
        CloseHandle(hFile);
    } else {
        bSuccess = FALSE;
    }

    if (*pItem).Type == SHELLNEW_TYPE_FILENAME {
        if CopyFileW(
            (*pItem).pData as LPWSTR,
            wszName,
            FALSE,
        ) == 0
        {
            ERR!("Copy file failed: %ls\n", ((*pItem).pData as *mut u16));
        }
    }

    if bSuccess != 0 {
        let rename = (pItem != (*this).m_pLinkItem) as BOOL;
        CNewMenu_SelectNewItem_body(
            this,
            SHCNE_CREATE as LONG,
            SHCNF_PATHW,
            wszName,
            rename,
        );
    } else {
        let caption_ptr = ((IDS_CREATEFILE_CAPTION as u16) as usize) as LPWSTR;
        let message_ptr = ((IDS_CREATEFILE_DENIED as u16) as usize) as LPWSTR;
        let mut caption = CStringW::new_from_lpcwstr(caption_ptr);
        let mut message = CStringW::new_from_lpcwstr(message_ptr);
        message.format_message(wszName);
        MessageBoxW(core::ptr::null_mut(),
            message.as_ptr(),
            caption.as_ptr(),
            MB_ICONEXCLAMATION | MB_OK,
        );
    }

    S_OK
}

pub unsafe fn CNewMenu_CreateNewItem_body(this: *mut CNewMenu, pItem: *mut SHELLNEW_ITEM, lpcmi: LPCMINVOKECOMMANDINFO) -> HRESULT {
    let mut hr: HRESULT;
    let mut wszPath: [u16; MAX_PATH] = [0u16; MAX_PATH];
    let mut wszName: [u16; MAX_PATH] = [0u16; MAX_PATH];

    hr = SHGetPathFromIDListW((*this).m_pidlFolder, wszPath.as_mut_ptr());
    if ((hr) < 0) {
        return hr;
    }

    if pItem == (*this).m_pLinkItem {
        CNewMenu_NewItemByNonCommand_body(this, pItem, wszName.as_mut_ptr(), wszName.len() as DWORD, wszPath.as_ptr());
        CNewMenu_NewItemByCommand_body(this, pItem, wszName.as_mut_ptr());
        return S_OK;
    }

    match (*pItem).Type {
        SHELLNEW_TYPE_COMMAND => {
            CNewMenu_NewItemByCommand_body(this, pItem, wszPath.as_ptr());
        }
        SHELLNEW_TYPE_DATA | SHELLNEW_TYPE_FILENAME | SHELLNEW_TYPE_NULLFILE => {
            CNewMenu_NewItemByNonCommand_body(this, pItem, wszName.as_mut_ptr(), wszName.len() as DWORD, wszPath.as_ptr());
        }
        SHELLNEW_TYPE_INVALID => {
            ERR!("Invalid type\n");
        }
        _ => {}
}

    S_OK
}

pub unsafe fn CNewMenu_SetSite_body(this: *mut CNewMenu, pUnkSite: *mut IUnknown) -> HRESULT {
    CNewMenu_cpp_com_assign_m_pSite(this, pUnkSite);
    S_OK
}

pub unsafe fn CNewMenu_GetSite_body(this: *mut CNewMenu, riid: REFIID, ppvSite: *mut PVOID) -> HRESULT {
    let pSite = (*this).m_pSite;
    ((*(*pSite).lpVtbl).QueryInterface)(pSite, riid, ppvSite)
}

pub unsafe fn CNewMenu_QueryContextMenu_body(this: *mut CNewMenu, hMenu: HMENU, indexMenu: UINT, idCmdFirst: UINT, idCmdLast: UINT, uFlags: UINT) -> HRESULT {
    let mut wszNew: [u16; 200] = [0u16; 200];

    // Load the "New" label string
    let load_result = LoadStringW(
        shell32_hInstance,
        FCIDM_SHVIEW_NEW,
        wszNew.as_mut_ptr(),
        wszNew.len() as INT,
    );
    if load_result == 0 {
        return E_FAIL;
    }

    // Create the sub-menu
    let sub_menu = CreateMenu();
    if sub_menu.is_null() {
        return E_FAIL;
    }
    (*this).m_hSubMenu = sub_menu;

    // Populate the sub-menu with shell new items
    let cItems = CNewMenu_InsertShellNewItems_body(
        this,
        sub_menu,
        idCmdFirst,
        0,
    );

    // Build the menu item info for the "New" entry
    let mut mii: MENUITEMINFOW = core::mem::zeroed();
    ZeroMemory(
        &mut mii as *mut _ as *mut core::ffi::c_void,
        core::mem::size_of::<MENUITEMINFOW>() as u32,
    );

    mii.cbSize = core::mem::size_of::<MENUITEMINFOW>() as u32;
    mii.fMask = MIIM_TYPE | MIIM_ID | MIIM_STATE | MIIM_SUBMENU;
    mii.fType = MFT_STRING;
    mii.wID = (-(1 as i32)) as UINT;
    mii.dwTypeData = wszNew.as_mut_ptr();
    mii.cch = wcslen(mii.dwTypeData as *const u16) as UINT;
    mii.fState = MFS_ENABLED;
    mii.hSubMenu = sub_menu;

    // Insert the parent menu item
    let insert_ok = InsertMenuItemW(
        hMenu,
        indexMenu,
        TRUE,
        &mii as *const MENUITEMINFOW,
    );
    if insert_ok == 0 {
        return E_FAIL;
    }

    // Return HRESULT with count of inserted items
    MAKE_HRESULT(SEVERITY_SUCCESS, 0, cItems)
}

pub unsafe fn CNewMenu_InvokeCommand_body(this: *mut CNewMenu, lpici: LPCMINVOKECOMMANDINFO) -> HRESULT {
    let mut hr: HRESULT = E_FAIL;
    if (*this).m_idCmdFirst + LOWORD!((*lpici).lpVerb) == (*this).m_idCmdFolder {
        hr = CNewMenu_CreateNewFolder_body(this, lpici);
    } else {
        let pItem: *mut SHELLNEW_ITEM = CNewMenu_FindItemFromIdOffset_body(this, LOWORD!((*lpici).lpVerb));
        if !pItem.is_null() {
            hr = CNewMenu_CreateNewItem_body(this, pItem, lpici);
        }
    }
    hr
}

pub unsafe fn CNewMenu_GetCommandString_body(this: *mut CNewMenu, idCmd: UINT_PTR, uType: UINT, pwReserved: *mut UINT, pszName: LPSTR, cchMax: UINT) -> HRESULT { E_NOTIMPL }

pub unsafe fn CNewMenu_HandleMenuMsg_body(this: *mut CNewMenu, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> HRESULT {
    CNewMenu_HandleMenuMsg2_body(this, uMsg, wParam, lParam, core::ptr::null_mut())
}

pub unsafe fn CNewMenu_HandleMenuMsg2_body(this: *mut CNewMenu, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, plResult: *mut LRESULT) -> HRESULT {
    match uMsg {
        WM_MEASUREITEM => {
            let lpmis: *mut MEASUREITEMSTRUCT = lParam as *mut MEASUREITEMSTRUCT;
            if lpmis.is_null() || (*lpmis).CtlType != ODT_MENU {
                // break out of switch
            } else {
                if ((*lpmis).itemWidth as UINT)
                    < (GetSystemMetrics((SM_CXMENUCHECK) as i32) as UINT)
                {
                    (*lpmis).itemWidth = GetSystemMetrics((SM_CXMENUCHECK) as i32) as UINT;
                }
                if (*lpmis).itemHeight < 16 {
                    (*lpmis).itemHeight = 16;
                }
                if !plResult.is_null() {
                    *plResult = TRUE as LRESULT;
                }
            }
        }
        WM_DRAWITEM => {
            let lpdis: *mut DRAWITEMSTRUCT = lParam as *mut DRAWITEMSTRUCT;
            if lpdis.is_null() || (*lpdis).CtlType != ODT_MENU {
            } else {
                let id: DWORD = (*lpdis).itemID;
                let mut hIcon: HICON = core::ptr::null_mut();
                if id == (*this).m_idCmdFolder {
                    hIcon = (*this).m_hIconFolder;
                } else if id == (*this).m_idCmdLink {
                    hIcon = (*this).m_hIconLink;
                } else {
                    let pItem = CNewMenu_FindItemFromIdOffset_body(
                        this,
                        id.wrapping_sub((*this).m_idCmdFirst),
                    );
                    if !pItem.is_null() {
                        hIcon = (*pItem).hIcon;
                    }
                }
                if !hIcon.is_null() {
                    DrawIconEx(
                        (*lpdis).hDC,
                        2,
                        (*lpdis).rcItem.top
                            + ((*lpdis).rcItem.bottom
                                - (*lpdis).rcItem.top
                                - 16)
                                / 2,
                        hIcon,
                        16,
                        16,
                        0,
                        core::ptr::null_mut(),
                        DI_NORMAL,
                    );
                }
                if !plResult.is_null() {
                    *plResult = TRUE as LRESULT;
                }
            }
        }
        _ => {}
    }
    S_OK
}

pub unsafe fn CNewMenu_Initialize_body(this: *mut CNewMenu, pidlFolder: PCIDLIST_ABSOLUTE, pdtobj: *mut IDataObject, hkeyProgID: HKEY) -> HRESULT {
    let cx: i32 = GetSystemMetrics((SM_CXSMICON) as i32);
    let cy: i32 = GetSystemMetrics((SM_CYSMICON) as i32);
    let mut wszIconPath: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut icon_idx: i32 = 0;

    (*this).m_pidlFolder = ILClone(pidlFolder);

    if HLM_GetIconW(
        (IDI_SHELL_FOLDER - 1) as i32,
        wszIconPath.as_mut_ptr(),
        wszIconPath.len() as u32,
        &mut icon_idx,
    ) != 0
    {
        ExtractIconExW(
            wszIconPath.as_mut_ptr(),
            icon_idx,
            &mut (*this).m_hIconFolder,
            core::ptr::null_mut(),
            1,
        );
        (*this).m_bCustomIconFolder = TRUE;
    } else {
        (*this).m_hIconFolder = LoadImage(
            shell32_hInstance,
            MAKEINTRESOURCE(IDI_SHELL_FOLDER),
            IMAGE_ICON,
            cx,
            cy,
            LR_SHARED,
        ) as HICON;
    }

    if HLM_GetIconW(
        (IDI_SHELL_SHORTCUT - 1) as i32,
        wszIconPath.as_mut_ptr(),
        wszIconPath.len() as u32,
        &mut icon_idx,
    ) != 0
    {
        ExtractIconExW(
            wszIconPath.as_mut_ptr(),
            icon_idx,
            &mut (*this).m_hIconLink,
            core::ptr::null_mut(),
            1,
        );
        (*this).m_bCustomIconLink = TRUE;
    } else {
        (*this).m_hIconLink = LoadImage(
            shell32_hInstance,
            MAKEINTRESOURCE(IDI_SHELL_SHORTCUT),
            IMAGE_ICON,
            cx,
            cy,
            LR_SHARED,
        ) as HICON;
    }

    S_OK
}
