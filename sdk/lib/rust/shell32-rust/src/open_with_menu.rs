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

// Deterministic COM scaffold for COpenWithMenu; method bodies are generated separately.
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
pub struct COpenWithMenu {
    pub IContextMenu2_iface: IContextMenu2,
    pub IShellExtInit_iface: IShellExtInit,
    pub ref_count: core::sync::atomic::AtomicI32,
    pub m_idCmdFirst: UINT,
    pub m_idCmdLast: UINT,
    pub m_wszPath: [WCHAR; 260],
    pub m_hSubMenu: HMENU,
    pub m_pAppList: *mut core::ffi::c_void,
}

unsafe extern "system" fn COpenWithMenu_IContextMenu2_QueryInterface(This: *mut IContextMenu2, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(COpenWithMenu, IContextMenu2_iface)) as *mut COpenWithMenu;
    COpenWithMenu_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn COpenWithMenu_IContextMenu2_AddRef(This: *mut IContextMenu2) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(COpenWithMenu, IContextMenu2_iface)) as *mut COpenWithMenu;
    COpenWithMenu_AddRef(this)
}
unsafe extern "system" fn COpenWithMenu_IContextMenu2_Release(This: *mut IContextMenu2) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(COpenWithMenu, IContextMenu2_iface)) as *mut COpenWithMenu;
    COpenWithMenu_Release(this)
}
unsafe extern "system" fn COpenWithMenu_IContextMenu2_QueryContextMenu(This: *mut IContextMenu2, hmenu: HMENU, indexMenu: UINT, idCmdFirst: UINT, idCmdLast: UINT, uFlags: UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(COpenWithMenu, IContextMenu2_iface)) as *mut COpenWithMenu;
    COpenWithMenu_QueryContextMenu_body(this, hmenu, indexMenu, idCmdFirst, idCmdLast, uFlags)
}
unsafe extern "system" fn COpenWithMenu_IContextMenu2_InvokeCommand(This: *mut IContextMenu2, lpici: LPCMINVOKECOMMANDINFO) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(COpenWithMenu, IContextMenu2_iface)) as *mut COpenWithMenu;
    COpenWithMenu_InvokeCommand_body(this, lpici)
}
unsafe extern "system" fn COpenWithMenu_IContextMenu2_GetCommandString(This: *mut IContextMenu2, idCmd: UINT_PTR, uType: UINT, pwReserved: *mut UINT, pszName: LPSTR, cchMax: UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(COpenWithMenu, IContextMenu2_iface)) as *mut COpenWithMenu;
    COpenWithMenu_GetCommandString_body(this, idCmd, uType, pwReserved, pszName, cchMax)
}
unsafe extern "system" fn COpenWithMenu_IContextMenu2_HandleMenuMsg(This: *mut IContextMenu2, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(COpenWithMenu, IContextMenu2_iface)) as *mut COpenWithMenu;
    COpenWithMenu_HandleMenuMsg_body(this, uMsg, wParam, lParam)
}

pub static COPENWITHMENU_ICONTEXTMENU2_VTBL: IContextMenu2Vtbl = IContextMenu2Vtbl {
    QueryInterface: COpenWithMenu_IContextMenu2_QueryInterface,
    AddRef: COpenWithMenu_IContextMenu2_AddRef,
    Release: COpenWithMenu_IContextMenu2_Release,
    QueryContextMenu: COpenWithMenu_IContextMenu2_QueryContextMenu,
    InvokeCommand: COpenWithMenu_IContextMenu2_InvokeCommand,
    GetCommandString: COpenWithMenu_IContextMenu2_GetCommandString,
    HandleMenuMsg: COpenWithMenu_IContextMenu2_HandleMenuMsg,
};

unsafe extern "system" fn COpenWithMenu_IShellExtInit_QueryInterface(This: *mut IShellExtInit, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(COpenWithMenu, IShellExtInit_iface)) as *mut COpenWithMenu;
    COpenWithMenu_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn COpenWithMenu_IShellExtInit_AddRef(This: *mut IShellExtInit) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(COpenWithMenu, IShellExtInit_iface)) as *mut COpenWithMenu;
    COpenWithMenu_AddRef(this)
}
unsafe extern "system" fn COpenWithMenu_IShellExtInit_Release(This: *mut IShellExtInit) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(COpenWithMenu, IShellExtInit_iface)) as *mut COpenWithMenu;
    COpenWithMenu_Release(this)
}
unsafe extern "system" fn COpenWithMenu_IShellExtInit_Initialize(This: *mut IShellExtInit, pidlFolder: PCIDLIST_ABSOLUTE, pdtobj: *mut IDataObject, hkeyProgID: HKEY) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(COpenWithMenu, IShellExtInit_iface)) as *mut COpenWithMenu;
    COpenWithMenu_Initialize_body(this, pidlFolder, pdtobj, hkeyProgID)
}

pub static COPENWITHMENU_ISHELLEXTINIT_VTBL: IShellExtInitVtbl = IShellExtInitVtbl {
    QueryInterface: COpenWithMenu_IShellExtInit_QueryInterface,
    AddRef: COpenWithMenu_IShellExtInit_AddRef,
    Release: COpenWithMenu_IShellExtInit_Release,
    Initialize: COpenWithMenu_IShellExtInit_Initialize,
};

unsafe fn COpenWithMenu_QueryInterface(
    this: *mut COpenWithMenu, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IUnknown)) || cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IContextMenu2)) {
        *ppv = core::ptr::addr_of_mut!((*this).IContextMenu2_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IContextMenu)) {
        *ppv = core::ptr::addr_of_mut!((*this).IContextMenu2_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IShellExtInit)) {
        *ppv = core::ptr::addr_of_mut!((*this).IShellExtInit_iface) as *mut core::ffi::c_void;
    }
    else { return E_NOINTERFACE; }
    COpenWithMenu_AddRef(this);
    S_OK
}

unsafe fn COpenWithMenu_AddRef(this: *mut COpenWithMenu) -> ULONG {
    (*this).ref_count.fetch_add(1, core::sync::atomic::Ordering::SeqCst) as ULONG + 1
}

unsafe fn COpenWithMenu_Release(this: *mut COpenWithMenu) -> ULONG {
    let refs = (*this).ref_count.fetch_sub(1, core::sync::atomic::Ordering::SeqCst) - 1;
    if refs == 0 {
        COpenWithMenu_dtor_body(this);
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

// Activation: CLSID_OpenWithMenu currently maps to COpenWithMenu in shell32.cpp.
// The generated C++ adapter redirects ATL activation to this factory.
#[no_mangle] pub unsafe extern "system" fn COpenWithMenu_CreateInstance(
    outer: *mut IUnknown, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if !outer.is_null() { return CLASS_E_NOAGGREGATION; }
    let this = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<COpenWithMenu>()) as *mut COpenWithMenu;
    if this.is_null() { return E_OUTOFMEMORY; }
    core::ptr::write(this, COpenWithMenu {
        IContextMenu2_iface: IContextMenu2 { lpVtbl: core::ptr::addr_of!(COPENWITHMENU_ICONTEXTMENU2_VTBL) },
        IShellExtInit_iface: IShellExtInit { lpVtbl: core::ptr::addr_of!(COPENWITHMENU_ISHELLEXTINIT_VTBL) },
        ref_count: core::sync::atomic::AtomicI32::new(1),
        m_idCmdFirst: core::mem::zeroed(),
        m_idCmdLast: core::mem::zeroed(),
        m_wszPath: core::mem::zeroed(),
        m_hSubMenu: core::mem::zeroed(),
        m_pAppList: core::mem::zeroed(),
    });
    Shell32RustObjectLock();
    COpenWithMenu_ctor_body(this);
    let hr = COpenWithMenu_QueryInterface(this, riid, ppv);
    COpenWithMenu_Release(this);
    hr
}

// Deterministic C++ support closure selected from source identifiers.

pub const IDD_NOOPEN: UINT = 27;

pub const IDD_OPEN_WITH: UINT = 24;

pub const IDI_SHELL_PROGRAMS_FOLDER: UINT = 20;

pub const IDR_OPENWITHMENU: UINT = 146;

pub const IDS_OPEN: UINT = 23;

pub const IDS_OPEN_WITH: UINT = 142;

pub const IDS_OPEN_WITH_CHOOSE: UINT = 143;

pub const IDS_OPEN_WITH_FILTER: UINT = 155;

pub const IDS_OPEN_WITH_OTHER: UINT = 179;

pub const IDS_OPEN_WITH_RECOMMENDED: UINT = 178;

pub const E_UNEXPECTED: HRESULT = 0x8000_FFFFu32 as HRESULT;

pub const SW_SHOWNORMAL: INT = 1;

pub const ASSOCSTR_COMMAND: DWORD = 1;

pub const BM_SETCHECK: DWORD = 241;

pub const BST_CHECKED: DWORD = 1;

pub const COLOR_MENU: DWORD = 4;

pub const DI_NORMAL: DWORD = 3;

pub const ERROR_FILE_NOT_FOUND: DWORD = 2;

pub const ERROR_INVALID_PARAMETER: DWORD = 87;

pub const ERROR_NO_ASSOCIATION: DWORD = 1155;

pub const IDCANCEL: DWORD = 2;

pub const IDNO: DWORD = 7;

pub const IDOK: DWORD = 1;

pub const IDYES: DWORD = 6;

pub const ILC_COLOR32: DWORD = 0x20;

pub const ILC_MASK: DWORD = 0x1;

pub const IMAGE_ICON: DWORD = 1;

pub const KEY_READ: DWORD = 0x20019;

pub const KEY_WRITE: DWORD = 0x20006;

pub const MFS_DEFAULT: DWORD = 4096;

pub const MFS_ENABLED: DWORD = 0;

pub const MFT_SEPARATOR: DWORD = 0x800;

pub const MIIM_CHECKMARKS: DWORD = 8;

pub const MIIM_DATA: DWORD = 32;

pub const MIIM_FTYPE: DWORD = 256;

pub const MIIM_STATE: DWORD = 1;

pub const MIIM_SUBMENU: DWORD = 4;

pub const MRU_STRING: DWORD = 0x0;

pub const OAIF_ALLOW_REGISTRATION: DWORD = 1;

pub const OAIF_EXEC: DWORD = 4;

pub const OAIF_FORCE_REGISTRATION: DWORD = 8;

pub const OAIF_HIDE_REGISTRATION: DWORD = 32;

pub const OAIF_REGISTER_EXT: DWORD = 2;

pub const OFN_EXPLORER: DWORD = 0x80000;

pub const OFN_FILEMUSTEXIST: DWORD = 0x1000;

pub const OFN_PATHMUSTEXIST: DWORD = 0x800;

pub const PRF_DONTFINDLNK: DWORD = 0x08;

pub const PRF_FIRSTDIRDEF: DWORD = 0x04;

pub const PRF_VERIFYEXISTS: DWORD = 0x01;

pub const REG_EXPAND_SZ: DWORD = 2;

pub const REG_SZ: DWORD = 1;

pub const SEE_MASK_CLASSKEY: DWORD = 0x00000003;

pub const SEE_MASK_NOCLOSEPROCESS: DWORD = 0x00000040;

pub const SEE_MASK_NOZONECHECKS: DWORD = 0x00800000;

pub const SHCNE_ASSOCCHANGED: DWORD = 0x08000000;

pub const SHCNF_FLUSHNOWAIT: DWORD = 0x3000;

pub const SHGFI_ICON: DWORD = 256;

pub const SHGFI_SHELLICONSIZE: DWORD = 4;

pub const SHGFI_SMALLICON: DWORD = 1;

pub const SM_CXMENUCHECK: DWORD = 71;

pub const SM_CYMENUCHECK: DWORD = 72;

pub const SW_HIDE: DWORD = 0;

pub const TVIF_IMAGE: DWORD = 0x2;

pub const TVIF_PARAM: DWORD = 0x4;

pub const TVIF_SELECTEDIMAGE: DWORD = 0x20;

pub const TVIF_STATE: DWORD = 0x8;

pub const TVIF_TEXT: DWORD = 0x1;

pub const TVIS_EXPANDED: DWORD = 0x20;

pub const TVSIL_NORMAL: DWORD = 0;

pub const WM_CLOSE: DWORD = 16;

pub const WM_COMMAND: DWORD = 273;

pub const WM_INITDIALOG: DWORD = 272;

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

macro_rules! LOWORD { ($value:expr) => { (($value as usize) & 0xffff) as UINT }; }

#[inline]
const fn MAKEINTRESOURCE(value: UINT) -> LPCWSTR {
    value as usize as LPCWSTR
}

#[repr(C)]
pub struct SHELLEXECUTEINFOW {
    pub cbSize: DWORD, pub fMask: ULONG, pub hwnd: HWND,
    pub lpVerb: LPCWSTR, pub lpFile: LPCWSTR, pub lpParameters: LPCWSTR,
    pub lpDirectory: LPCWSTR, pub nShow: INT, pub hInstApp: HINSTANCE,
    pub lpIDList: LPVOID, pub lpClass: LPCWSTR, pub hkeyClass: HKEY,
    pub dwHotKey: DWORD, pub icon_or_monitor: HANDLE, pub hProcess: HANDLE,
}

#[repr(C)]
pub struct CMVERBMAP { pub Verb: LPCSTR, pub CmdId: WORD }
pub static mut g_VerbMap: [CMVERBMAP; 2] = [
    CMVERBMAP { Verb: b"openas\0".as_ptr() as LPCSTR, CmdId: 0 as WORD },
    CMVERBMAP { Verb: core::ptr::null(), CmdId: 0 as WORD },
];

extern "system" {
    fn DestroyIcon(icon: HICON) -> BOOL;
    fn DrawIconEx(dc: HDC, x: INT, y: INT, icon: HICON, width: INT, height: INT, step: UINT, brush: HANDLE, flags: UINT) -> BOOL;
    fn ExtractIconExW(file: LPCWSTR, index: INT, large: *mut HICON, small: *mut HICON, icons: UINT) -> UINT;
    fn GetMenuItemInfoW(menu: HMENU, item: UINT, by_position: BOOL, info: *mut MENUITEMINFOW) -> BOOL;
    fn GetMenuItemCount(menu: HMENU) -> INT;
    fn GetMenuDefaultItem(menu: HMENU, by_position: UINT, flags: UINT) -> UINT;
    fn CreatePopupMenu() -> HMENU;
    fn GetDC(window: HWND) -> HDC;
    fn ReleaseDC(window: HWND, dc: HDC) -> INT;
    fn CreateCompatibleDC(dc: HDC) -> HDC;
    fn CreateCompatibleBitmap(dc: HDC, width: INT, height: INT) -> HBITMAP;
    fn SelectObject(dc: HDC, object: HANDLE) -> HANDLE;
    fn DeleteObject(object: HANDLE) -> BOOL;
    fn DeleteDC(dc: HDC) -> BOOL;
    fn FillRect(dc: HDC, rect: *const RECT, brush: HANDLE) -> INT;
    fn SetRect(rect: *mut RECT, left: INT, top: INT, right: INT, bottom: INT) -> BOOL;
    fn GetSystemMetrics(index: INT) -> INT;
    #[link_name = "LoadImageW"] fn LoadImage(instance: HINSTANCE, name: LPCWSTR, kind: UINT, width: INT, height: INT, flags: UINT) -> HANDLE;
    fn RegCloseKey(key: HKEY) -> LONG;
    #[link_name = "RegCreateKeyExW"] fn RegCreateKeyEx(key: HKEY, subkey: LPCWSTR, reserved: DWORD, class: LPWSTR, options: DWORD, access: REGSAM, security: LPVOID, result: PHKEY, disposition: LPDWORD) -> LONG;
    fn RegGetValueW(key: HKEY, subkey: LPCWSTR, value: LPCWSTR, flags: DWORD, kind: LPDWORD, data: LPVOID, bytes: LPDWORD) -> LONG;
    fn RegOpenKeyExW(key: HKEY, subkey: LPCWSTR, options: DWORD, access: REGSAM, result: PHKEY) -> LONG;
    fn RegQueryValueExW(key: HKEY, value: LPCWSTR, reserved: LPDWORD, kind: LPDWORD, data: PBYTE, bytes: LPDWORD) -> LONG;
    fn RegSetValueExW(key: HKEY, value: LPCWSTR, reserved: DWORD, kind: DWORD, data: *const BYTE, bytes: DWORD) -> LONG;
    fn SHChangeNotify(event: LONG, flags: UINT, item1: LPCVOID, item2: LPCVOID);
    fn SHGetFileInfoW(path: LPCWSTR, attributes: DWORD, info: *mut SHFILEINFOW, bytes: UINT, flags: UINT) -> DWORD_PTR;
    fn SHGetPathFromIDListW(pidl: LPCITEMIDLIST, path: LPWSTR) -> BOOL;
    fn PathFindExtensionW(path: LPCWSTR) -> LPWSTR;
    fn ILCombine(parent: PCIDLIST_ABSOLUTE, child: PCUITEMID_CHILD) -> PIDLIST_ABSOLUTE;
    fn GetLastError() -> DWORD;
    fn lstrlenW(text: LPCWSTR) -> INT;
    fn LoadStringW(instance: HINSTANCE, id: UINT, buffer: LPWSTR, capacity: INT) -> INT;
    fn CoTaskMemFree(memory: LPVOID);
    fn ShellExecuteExW(info: *mut SHELLEXECUTEINFOW) -> BOOL;
    fn InsertMenuItemW(menu: HMENU, item: UINT, by_position: BOOL, info: *const MENUITEMINFOW) -> BOOL;
    #[link_name = "Shell32RustMapContextMenuVerbToCmdId"] fn SHELL_MapContextMenuVerbToCmdId(info: LPCMINVOKECOMMANDINFO, map: *const CMVERBMAP) -> INT;
    #[link_name = "Shell32RustGetCommandStringImpl"] fn SHELL_GetCommandStringImpl(command: SIZE_T, flags: UINT, buffer: LPSTR, capacity: UINT, map: *const CMVERBMAP) -> HRESULT;
    fn COpenWithList_Create_RustBridge() -> *mut core::ffi::c_void;
    fn COpenWithList_Destroy_RustBridge(list: *mut core::ffi::c_void);
    fn COpenWithList_Load_RustBridge(list: *mut core::ffi::c_void) -> BOOL;
    fn COpenWithList_LoadRecommended_RustBridge(list: *mut core::ffi::c_void, path: LPCWSTR) -> BOOL;
    fn COpenWithList_GetRecommendedCount_RustBridge(list: *mut core::ffi::c_void) -> UINT;
    fn COpenWithList_GetCount_RustBridge(list: *mut core::ffi::c_void) -> UINT;
    fn COpenWithList_GetAt_RustBridge(list: *mut core::ffi::c_void, index: UINT) -> PVOID;
    fn COpenWithList_IsRecommended_RustBridge(app: PVOID) -> BOOL;
    fn COpenWithList_GetName_RustBridge(app: PVOID) -> LPCWSTR;
    fn COpenWithList_GetIcon_RustBridge(app: PVOID) -> HICON;
    fn COpenWithList_Execute_RustBridge(app: PVOID, path: LPCWSTR) -> BOOL;
    fn COpenWithMenu_ExtractPath_RustBridge(data: *mut IDataObject, path: LPWSTR, capacity: UINT) -> HRESULT;
    #[link_name = "Shell32RustInvokeOpenWith"]
    fn SH32_InvokeOpenWith(path: PCWSTR, info: LPCMINVOKECOMMANDINFO, process: *mut HANDLE) -> HRESULT;
}

extern "C" {
    #[link_name = "memcpy"] fn cpp_com_memcpy(dst: *mut core::ffi::c_void, src: *const core::ffi::c_void, bytes: usize) -> *mut core::ffi::c_void;
    fn _wcsicmp(left: LPCWSTR, right: LPCWSTR) -> INT;
    fn PathIsExeW(path: LPCWSTR) -> BOOL;
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
    static CLSID_OpenWithMenu: GUID;
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

extern "C" {
    static mut shell32_hInstance: HINSTANCE;
}

// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_FE706C396A0C: [WCHAR; 5] = [0x002E, 0x006C, 0x006E, 0x006B, 0x0000];

use core::ffi::c_void;
use crate::shellcom::*;
use crate::types::*;

#[inline]
fn IS_INTRESOURCE(value: LPCSTR) -> bool {
    (value as usize) >> 16 == 0
}

pub unsafe fn COpenWithMenu_ctor_body(this: *mut COpenWithMenu) -> () {
    (*this).m_idCmdFirst = 0;
    (*this).m_idCmdLast = 0;
    (*this).m_pAppList = COpenWithList_Create_RustBridge();
}

pub unsafe fn COpenWithMenu_dtor_body(this: *mut COpenWithMenu) -> () {
    if !(*this).m_hSubMenu.is_null() {
        let count = GetMenuItemCount((*this).m_hSubMenu);
        if count == -1 {
            return;
        }
        let mut mii: MENUITEMINFOW = core::mem::zeroed();
        ZeroMemory((&mut mii) as *mut _ as *mut _, core::mem::size_of::<MENUITEMINFOW>());
        mii.cbSize = core::mem::size_of::<MENUITEMINFOW>() as _;
        mii.fMask = MIIM_DATA | MIIM_FTYPE | MIIM_CHECKMARKS;
        let mut index = 0;
        while index < count {
            if GetMenuItemInfoW((*this).m_hSubMenu, index as UINT, TRUE, &mut mii as *mut _) != 0 {
                if !mii.hbmpChecked.is_null() {
                    DeleteObject(mii.hbmpChecked);
                }
            }
            index += 1;
        }
    }
    if !(*this).m_pAppList.is_null() {
        COpenWithList_Destroy_RustBridge((*this).m_pAppList);
        (*this).m_pAppList = core::ptr::null_mut();
    }
}

pub unsafe fn COpenWithMenu_IconToBitmap_body(this: *mut COpenWithMenu, hIcon: HICON) -> HBITMAP {
    let hdcScr: HDC;
    let hdc: HDC;
    let hbm: HBITMAP;
    let hbmOld: HBITMAP;
    let mut rc: RECT = core::mem::zeroed();

    hdcScr = GetDC(core::ptr::null_mut());
    hdc = CreateCompatibleDC(hdcScr);
    SetRect(
        &raw mut rc,
        0,
        0,
        GetSystemMetrics((SM_CXMENUCHECK) as i32),
        GetSystemMetrics((SM_CYMENUCHECK) as i32),
    );
    hbm = CreateCompatibleBitmap(hdcScr, rc.right, rc.bottom);
    ReleaseDC(core::ptr::null_mut(), hdcScr);

    hbmOld = SelectObject(hdc, hbm) as HBITMAP;
    FillRect(
        hdc,
        &raw mut rc,
        (COLOR_MENU + 1) as usize as HANDLE,
    );
    if DrawIconEx(
        hdc,
        0,
        0,
        hIcon,
        rc.right,
        rc.bottom,
        0,
        core::ptr::null_mut(),
        DI_NORMAL,
    ) == 0
    {
        ERR!("DrawIcon failed: %x\n", GetLastError());
    }
    SelectObject(hdc, hbmOld);
    DeleteDC(hdc);

    hbm
}

pub unsafe fn COpenWithMenu_AddChooseProgramItem_body(this: *mut COpenWithMenu) -> () {
    let mut mii: MENUITEMINFOW = core::mem::zeroed();
    ZeroMemory((&raw mut mii) as *mut _, core::mem::size_of::<MENUITEMINFOW>());
    mii.cbSize = core::mem::size_of::<MENUITEMINFOW>() as u32;
    mii.fMask = MIIM_TYPE | MIIM_ID;
    mii.fType = MFT_SEPARATOR;
    mii.wID = (0u32).wrapping_sub(1);
    InsertMenuItemW(
        (*this).m_hSubMenu,
        u32::MAX,
        TRUE,
        &raw mut mii,
    );

    let mut wszBuf: [u16; 128] = [0u16; 128];
    if LoadStringW(
        shell32_hInstance,
        IDS_OPEN_WITH_CHOOSE,
        wszBuf.as_mut_ptr(),
        wszBuf.len() as i32,
    ) == 0
    {
        return;
    }

    mii.fMask = MIIM_ID | MIIM_TYPE | MIIM_STATE;
    mii.fType = MFT_STRING;
    mii.fState = MFS_ENABLED;
    mii.wID = (*this).m_idCmdLast;
    mii.dwTypeData = wszBuf.as_mut_ptr() as *mut u16;
    mii.cch = wcslen(wszBuf.as_ptr()) as u32;

    InsertMenuItemW(
        (*this).m_hSubMenu,
        u32::MAX,
        TRUE,
        &raw mut mii,
    );
}

pub unsafe fn COpenWithMenu_AddApp_body(this: *mut COpenWithMenu, pApp: PVOID) -> () {
    let pwszName = COpenWithList_GetName_RustBridge(pApp);
    if pwszName.is_null() {
        return;
    }
    let mut mii: MENUITEMINFOW = core::mem::zeroed();
    ZeroMemory(
        &mut mii as *mut MENUITEMINFOW as *mut core::ffi::c_void,
        core::mem::size_of::<MENUITEMINFOW>(),
    );
    mii.cbSize = core::mem::size_of::<MENUITEMINFOW>() as u32;
    mii.fMask = MIIM_ID | MIIM_TYPE | MIIM_STATE | MIIM_DATA;
    mii.fType = MFT_STRING;
    mii.fState = MFS_ENABLED;
    mii.wID = (*this).m_idCmdLast;
    mii.dwTypeData = pwszName as LPWSTR;
    mii.dwItemData = pApp as usize;
    let hIcon = COpenWithList_GetIcon_RustBridge(pApp);
    if !hIcon.is_null() {
        mii.fMask |= MIIM_CHECKMARKS;
        let hbmp = COpenWithMenu_IconToBitmap_body(this, hIcon);
        mii.hbmpChecked = hbmp;
        mii.hbmpUnchecked = hbmp;
    }
    if InsertMenuItemW(
        (*this).m_hSubMenu,
        u32::MAX,
        TRUE,
        &mii,
    ) != 0
    {
        (*this).m_idCmdLast += 1;
    }
}

pub unsafe fn COpenWithMenu_QueryContextMenu_body(this: *mut COpenWithMenu, hMenu: HMENU, indexMenu: UINT, idCmdFirst: UINT, idCmdLast: UINT, uFlags: UINT) -> HRESULT {
    let default_pos = GetMenuDefaultItem(hMenu, TRUE as UINT, 0) as i32;
    let mut wszName: [u16; 100 as usize] = [0u16; 100 as usize];
    let name_id: u32 = if default_pos == -1 { IDS_OPEN } else { IDS_OPEN_WITH };
    if LoadStringW(shell32_hInstance, name_id, wszName.as_mut_ptr(), 100) == 0 {
        return E_FAIL;
    }
    (*this).m_idCmdFirst = idCmdFirst;
    (*this).m_idCmdLast = idCmdFirst;
    (*this).m_hSubMenu = core::ptr::null_mut();
    if default_pos != -1 {
        COpenWithList_Load_RustBridge((*this).m_pAppList);
        COpenWithList_LoadRecommended_RustBridge((*this).m_pAppList, (*this).m_wszPath.as_ptr());
        if COpenWithList_GetRecommendedCount_RustBridge((*this).m_pAppList) > 1 {
            (*this).m_hSubMenu = CreatePopupMenu();
            let mut i: u32 = 0;
            while i < COpenWithList_GetCount_RustBridge((*this).m_pAppList) {
                let pApp = COpenWithList_GetAt_RustBridge((*this).m_pAppList, i);
                if COpenWithList_IsRecommended_RustBridge(pApp) != FALSE {
                    COpenWithMenu_AddApp_body(this, pApp);
                }
                i += 1;
            }
            COpenWithMenu_AddChooseProgramItem_body(this);
        }
    }
    let mut mii: MENUITEMINFOW = core::mem::zeroed();
    ZeroMemory((&mut mii) as *mut _ as *mut core::ffi::c_void, core::mem::size_of::<MENUITEMINFOW>());
    mii.cbSize = core::mem::size_of::<MENUITEMINFOW>() as u32;
    mii.fMask = MIIM_ID | MIIM_TYPE | MIIM_STATE;
    if !(*this).m_hSubMenu.is_null() {
        mii.fMask |= MIIM_SUBMENU;
        mii.hSubMenu = (*this).m_hSubMenu;
        mii.wID = u32::MAX;
    } else {
        mii.wID = (*this).m_idCmdLast;
    }
    mii.fType = MFT_STRING;
    mii.dwTypeData = wszName.as_mut_ptr() as LPWSTR;
    mii.fState = MFS_ENABLED;
    let mut idx_menu = indexMenu;
    if default_pos == -1 {
        mii.fState |= MFS_DEFAULT;
        idx_menu = 0;
    }
    if InsertMenuItemW(hMenu, idx_menu, TRUE, &mii) == 0 {
        return E_FAIL;
    }
    MAKE_HRESULT(
        SEVERITY_SUCCESS,
        0,
        ((*this).m_idCmdLast - (*this).m_idCmdFirst + 1) as u32,
    )
}

pub unsafe fn COpenWithMenu_InvokeCommand_body(this: *mut COpenWithMenu, lpici: LPCMINVOKECOMMANDINFO) -> HRESULT {
    let idChooseApp = (*this).m_idCmdLast;
    let mut hr: HRESULT = E_FAIL;

    let should_invoke_open_with = if !IS_INTRESOURCE((*lpici).lpVerb)
        && SHELL_MapContextMenuVerbToCmdId(lpici, core::ptr::addr_of!(g_VerbMap).cast()) == 0
    {
        true
    } else if IS_INTRESOURCE((*lpici).lpVerb)
        && (*this).m_idCmdFirst + LOWORD!((*lpici).lpVerb) <= (*this).m_idCmdLast
    {
        let idx = (*this).m_idCmdFirst + LOWORD!((*lpici).lpVerb);
        if idx == idChooseApp {
            true
        } else {
            let mut mii: MENUITEMINFOW = core::mem::zeroed();
            ZeroMemory(
                &mut mii as *mut _,
                core::mem::size_of::<MENUITEMINFOW>(),
            );
            mii.cbSize = core::mem::size_of::<MENUITEMINFOW>() as u32;
            mii.fMask = MIIM_DATA | MIIM_FTYPE;

            if GetMenuItemInfoW(
                (*this).m_hSubMenu,
                LOWORD!((*lpici).lpVerb),
                TRUE,
                &mut mii as *mut MENUITEMINFOW,
            ) != 0
                && mii.dwItemData != 0
            {
                let pApp = mii.dwItemData as PVOID;
                let path_ptr = (*this).m_wszPath.as_ptr() as PCWSTR;
                COpenWithList_Execute_RustBridge(pApp, path_ptr);
                hr = S_OK;
            }
            false
        }
    } else {
        false
    };

    if should_invoke_open_with {
        let path_ptr = (*this).m_wszPath.as_ptr() as PCWSTR;
        hr = SH32_InvokeOpenWith(path_ptr, lpici, core::ptr::null_mut());
    }

    hr
}

pub unsafe fn COpenWithMenu_GetCommandString_body(this: *mut COpenWithMenu, idCmd: UINT_PTR, uType: UINT, pwReserved: *mut UINT, pszName: LPSTR, cchMax: UINT) -> HRESULT {
    let idChooseApp = (*this).m_idCmdLast as usize;
    if (*this).m_idCmdFirst as usize + idCmd as usize == idChooseApp {
        return SHELL_GetCommandStringImpl(0, uType, pszName, cchMax, core::ptr::addr_of!(g_VerbMap).cast());
    }
    E_NOTIMPL
}

pub unsafe fn COpenWithMenu_HandleMenuMsg_body(this: *mut COpenWithMenu, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> HRESULT { E_NOTIMPL }

pub unsafe fn COpenWithMenu_Initialize_body(this: *mut COpenWithMenu, pidlFolder: PCIDLIST_ABSOLUTE, pdtobj: *mut IDataObject, hkeyProgID: HKEY) -> HRESULT {
    if pdtobj.is_null() {
        return E_INVALIDARG;
    }

    let hr = COpenWithMenu_ExtractPath_RustBridge(
        pdtobj,
        (*this).m_wszPath.as_mut_ptr(),
        (*this).m_wszPath.len() as UINT,
    );
    if hr < 0 { return hr; }

    let pwszExt = PathFindExtensionW(&(*this).m_wszPath as *const u16);
    if PathIsExeW(pwszExt) != 0
        || _wcsicmp(pwszExt, CPP_COM_WSTR_FE706C396A0C.as_ptr()) == 0
    {
        return E_FAIL;
    }

    S_OK
}
