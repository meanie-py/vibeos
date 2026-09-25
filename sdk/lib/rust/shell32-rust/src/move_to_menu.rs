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

// Deterministic COM scaffold for CMoveToMenu; method bodies are generated separately.
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
pub struct CMoveToMenu {
    pub IContextMenu2_iface: IContextMenu2,
    pub IShellExtInit_iface: IShellExtInit,
    pub IObjectWithSite_iface: IObjectWithSite,
    pub ref_count: core::sync::atomic::AtomicI32,
    pub m_pDataObject: *mut IDataObject,
    pub m_pSite: *mut IUnknown,
    pub m_pidlFolder: *mut ITEMIDLIST,
    pub m_fnOldWndProc: WNDPROC,
    pub m_bIgnoreTextBoxChange: BOOL,
}

unsafe fn CMoveToMenu_cpp_com_assign_m_pDataObject(this: *mut CMoveToMenu, value: *mut IDataObject) {
    let old = (*this).m_pDataObject;
    if old == value { return; }
    if !value.is_null() {
        let unknown = value as *mut IUnknown;
        let add_ref: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).AddRef);
        add_ref(unknown);
    }
    (*this).m_pDataObject = value;
    if !old.is_null() {
        let unknown = old as *mut IUnknown;
        let release: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).Release);
        release(unknown);
    }
}

unsafe fn CMoveToMenu_cpp_com_assign_m_pSite(this: *mut CMoveToMenu, value: *mut IUnknown) {
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

unsafe extern "system" fn CMoveToMenu_IContextMenu2_QueryInterface(This: *mut IContextMenu2, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMoveToMenu, IContextMenu2_iface)) as *mut CMoveToMenu;
    CMoveToMenu_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CMoveToMenu_IContextMenu2_AddRef(This: *mut IContextMenu2) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMoveToMenu, IContextMenu2_iface)) as *mut CMoveToMenu;
    CMoveToMenu_AddRef(this)
}
unsafe extern "system" fn CMoveToMenu_IContextMenu2_Release(This: *mut IContextMenu2) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMoveToMenu, IContextMenu2_iface)) as *mut CMoveToMenu;
    CMoveToMenu_Release(this)
}
unsafe extern "system" fn CMoveToMenu_IContextMenu2_QueryContextMenu(This: *mut IContextMenu2, hmenu: HMENU, indexMenu: UINT, idCmdFirst: UINT, idCmdLast: UINT, uFlags: UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMoveToMenu, IContextMenu2_iface)) as *mut CMoveToMenu;
    CMoveToMenu_QueryContextMenu_body(this, hmenu, indexMenu, idCmdFirst, idCmdLast, uFlags)
}
unsafe extern "system" fn CMoveToMenu_IContextMenu2_InvokeCommand(This: *mut IContextMenu2, lpici: LPCMINVOKECOMMANDINFO) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMoveToMenu, IContextMenu2_iface)) as *mut CMoveToMenu;
    CMoveToMenu_InvokeCommand_body(this, lpici)
}
unsafe extern "system" fn CMoveToMenu_IContextMenu2_GetCommandString(This: *mut IContextMenu2, idCmd: UINT_PTR, uType: UINT, pwReserved: *mut UINT, pszName: LPSTR, cchMax: UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMoveToMenu, IContextMenu2_iface)) as *mut CMoveToMenu;
    CMoveToMenu_GetCommandString_body(this, idCmd, uType, pwReserved, pszName, cchMax)
}
unsafe extern "system" fn CMoveToMenu_IContextMenu2_HandleMenuMsg(This: *mut IContextMenu2, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMoveToMenu, IContextMenu2_iface)) as *mut CMoveToMenu;
    CMoveToMenu_HandleMenuMsg_body(this, uMsg, wParam, lParam)
}

pub static CMOVETOMENU_ICONTEXTMENU2_VTBL: IContextMenu2Vtbl = IContextMenu2Vtbl {
    QueryInterface: CMoveToMenu_IContextMenu2_QueryInterface,
    AddRef: CMoveToMenu_IContextMenu2_AddRef,
    Release: CMoveToMenu_IContextMenu2_Release,
    QueryContextMenu: CMoveToMenu_IContextMenu2_QueryContextMenu,
    InvokeCommand: CMoveToMenu_IContextMenu2_InvokeCommand,
    GetCommandString: CMoveToMenu_IContextMenu2_GetCommandString,
    HandleMenuMsg: CMoveToMenu_IContextMenu2_HandleMenuMsg,
};

unsafe extern "system" fn CMoveToMenu_IShellExtInit_QueryInterface(This: *mut IShellExtInit, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMoveToMenu, IShellExtInit_iface)) as *mut CMoveToMenu;
    CMoveToMenu_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CMoveToMenu_IShellExtInit_AddRef(This: *mut IShellExtInit) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMoveToMenu, IShellExtInit_iface)) as *mut CMoveToMenu;
    CMoveToMenu_AddRef(this)
}
unsafe extern "system" fn CMoveToMenu_IShellExtInit_Release(This: *mut IShellExtInit) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMoveToMenu, IShellExtInit_iface)) as *mut CMoveToMenu;
    CMoveToMenu_Release(this)
}
unsafe extern "system" fn CMoveToMenu_IShellExtInit_Initialize(This: *mut IShellExtInit, pidlFolder: PCIDLIST_ABSOLUTE, pdtobj: *mut IDataObject, hkeyProgID: HKEY) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMoveToMenu, IShellExtInit_iface)) as *mut CMoveToMenu;
    CMoveToMenu_Initialize_body(this, pidlFolder, pdtobj, hkeyProgID)
}

pub static CMOVETOMENU_ISHELLEXTINIT_VTBL: IShellExtInitVtbl = IShellExtInitVtbl {
    QueryInterface: CMoveToMenu_IShellExtInit_QueryInterface,
    AddRef: CMoveToMenu_IShellExtInit_AddRef,
    Release: CMoveToMenu_IShellExtInit_Release,
    Initialize: CMoveToMenu_IShellExtInit_Initialize,
};

unsafe extern "system" fn CMoveToMenu_IObjectWithSite_QueryInterface(This: *mut IObjectWithSite, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMoveToMenu, IObjectWithSite_iface)) as *mut CMoveToMenu;
    CMoveToMenu_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CMoveToMenu_IObjectWithSite_AddRef(This: *mut IObjectWithSite) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMoveToMenu, IObjectWithSite_iface)) as *mut CMoveToMenu;
    CMoveToMenu_AddRef(this)
}
unsafe extern "system" fn CMoveToMenu_IObjectWithSite_Release(This: *mut IObjectWithSite) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMoveToMenu, IObjectWithSite_iface)) as *mut CMoveToMenu;
    CMoveToMenu_Release(this)
}
unsafe extern "system" fn CMoveToMenu_IObjectWithSite_SetSite(This: *mut IObjectWithSite, pUnkSite: *mut IUnknown) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMoveToMenu, IObjectWithSite_iface)) as *mut CMoveToMenu;
    CMoveToMenu_SetSite_body(this, pUnkSite)
}
unsafe extern "system" fn CMoveToMenu_IObjectWithSite_GetSite(This: *mut IObjectWithSite, riid: REFIID, ppvSite: *mut PVOID) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMoveToMenu, IObjectWithSite_iface)) as *mut CMoveToMenu;
    CMoveToMenu_GetSite_body(this, riid, ppvSite)
}

pub static CMOVETOMENU_IOBJECTWITHSITE_VTBL: IObjectWithSiteVtbl = IObjectWithSiteVtbl {
    QueryInterface: CMoveToMenu_IObjectWithSite_QueryInterface,
    AddRef: CMoveToMenu_IObjectWithSite_AddRef,
    Release: CMoveToMenu_IObjectWithSite_Release,
    SetSite: CMoveToMenu_IObjectWithSite_SetSite,
    GetSite: CMoveToMenu_IObjectWithSite_GetSite,
};

unsafe fn CMoveToMenu_QueryInterface(
    this: *mut CMoveToMenu, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
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
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IObjectWithSite)) {
        *ppv = core::ptr::addr_of_mut!((*this).IObjectWithSite_iface) as *mut core::ffi::c_void;
    }
    else { return E_NOINTERFACE; }
    CMoveToMenu_AddRef(this);
    S_OK
}

unsafe fn CMoveToMenu_AddRef(this: *mut CMoveToMenu) -> ULONG {
    (*this).ref_count.fetch_add(1, core::sync::atomic::Ordering::SeqCst) as ULONG + 1
}

unsafe fn CMoveToMenu_Release(this: *mut CMoveToMenu) -> ULONG {
    let refs = (*this).ref_count.fetch_sub(1, core::sync::atomic::Ordering::SeqCst) - 1;
    if refs == 0 {
        CMoveToMenu_dtor_body(this);
        CMoveToMenu_cpp_com_assign_m_pDataObject(this, core::ptr::null_mut());
        CMoveToMenu_cpp_com_assign_m_pSite(this, core::ptr::null_mut());
        if !(*this).m_pidlFolder.is_null() { CoTaskMemFree((*this).m_pidlFolder.cast()); }
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

// Activation: CLSID_MoveToMenu currently maps to CMoveToMenu in shell32.cpp.
// The generated C++ adapter redirects ATL activation to this factory.
#[no_mangle] pub unsafe extern "system" fn CMoveToMenu_CreateInstance(
    outer: *mut IUnknown, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if !outer.is_null() { return CLASS_E_NOAGGREGATION; }
    let this = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<CMoveToMenu>()) as *mut CMoveToMenu;
    if this.is_null() { return E_OUTOFMEMORY; }
    core::ptr::write(this, CMoveToMenu {
        IContextMenu2_iface: IContextMenu2 { lpVtbl: core::ptr::addr_of!(CMOVETOMENU_ICONTEXTMENU2_VTBL) },
        IShellExtInit_iface: IShellExtInit { lpVtbl: core::ptr::addr_of!(CMOVETOMENU_ISHELLEXTINIT_VTBL) },
        IObjectWithSite_iface: IObjectWithSite { lpVtbl: core::ptr::addr_of!(CMOVETOMENU_IOBJECTWITHSITE_VTBL) },
        ref_count: core::sync::atomic::AtomicI32::new(1),
        m_pDataObject: core::mem::zeroed(),
        m_pSite: core::mem::zeroed(),
        m_pidlFolder: core::mem::zeroed(),
        m_fnOldWndProc: core::mem::zeroed(),
        m_bIgnoreTextBoxChange: core::mem::zeroed(),
    });
    Shell32RustObjectLock();
    CMoveToMenu_ctor_body(this);
    let hr = CMoveToMenu_QueryInterface(this, riid, ppv);
    CMoveToMenu_Release(this);
    hr
}

// Deterministic C++ support closure selected from source identifiers.

pub const IDC_BROWSE_FOR_FOLDER_FOLDER_TEXT: UINT = 0x3745;

pub const IDR_COPYTOMENU: UINT = 159;

pub const IDR_MOVETOMENU: UINT = 160;

pub const IDS_COPYBUTTON: UINT = 60007;

pub const IDS_COPYITEMS: UINT = 30534;

pub const IDS_COPYTOMENU: UINT = 30532;

pub const IDS_MOVEITEMS: UINT = 30538;

pub const IDS_MOVETOMENU: UINT = 30536;

pub const IDS_MOVETOTITLE: UINT = 30537;

pub const BFFM_INITIALIZED: DWORD = 1;

pub const BFFM_SELCHANGED: DWORD = 2;

pub const BIF_RETURNONLYFSDIRS: DWORD = 0x0001;

pub const CMF_VERBSONLY: DWORD = 0x00000002;

pub const CSIDL_DESKTOPDIRECTORY: core::ffi::c_int = 0x0010; // SHGetSpecialFolderPathW csidl is int

pub const EN_CHANGE: DWORD = 768;

pub const ERROR_MENU_ITEM_NOT_FOUND: DWORD = 1456;

pub const FOF_ALLOWUNDO: WORD = 0x0040; // FILEOP_FLAGS (shellapi.h) is WORD-typed

pub const FO_COPY: DWORD = 2;

pub const FO_MOVE: DWORD = 1;

pub const GCS_UNICODE: DWORD = 0x00000004;

pub const GCS_VALIDATEW: DWORD = 0x00000006;

pub const GCS_VERBW: DWORD = 0x00000004;

pub const IDOK: DWORD = 1;

pub const MFT_SEPARATOR: DWORD = 0x800;

pub const SHGDN_INFOLDER: DWORD = 1;

pub const WM_COMMAND: DWORD = 273;

#[repr(C)]
pub struct BIND_OPTS {
    pub cbStruct: DWORD,
    pub grfFlags: DWORD,
    pub grfMode: DWORD,
    pub dwTickCountDeadline: DWORD,
}

#[repr(C)]
pub struct CIDA {
    pub cidl: UINT,
    pub aoffset: [UINT; 1],
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

extern "system" {
    fn GetMenuItemInfoW(menu: HMENU, item: UINT, by_position: BOOL, info: *mut MENUITEMINFOW) -> BOOL;
    fn SHGetPathFromIDListW(pidl: LPCITEMIDLIST, path: LPWSTR) -> BOOL;
    fn GetLastError() -> DWORD;
    fn lstrlenW(text: LPCWSTR) -> INT;
    fn CoTaskMemFree(memory: LPVOID);
    fn ILRemoveLastID(pidl: LPITEMIDLIST) -> BOOL;
    fn InsertMenuItemW(menu: HMENU, item: UINT, by_position: BOOL, info: *const MENUITEMINFOW) -> BOOL;
}

extern "C" {
    fn _ILIsDesktop(pidl: LPCITEMIDLIST) -> BOOL;
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
    // Spellings used by the translated bodies (ATL operator!/operator T*/ctor forms).
    pub fn is_null(&self) -> bool { self.ptr.is_null() }
    pub const fn get(&self) -> *mut T { self.ptr }
    pub const fn as_ptr(&self) -> *mut T { self.ptr }
    pub const fn from_raw(ptr: *mut T) -> Self { Self { ptr } }
}
impl<T> Drop for CComHeapPtr<T> {
    fn drop(&mut self) {
        unsafe { if !self.ptr.is_null() { CoTaskMemFree(self.ptr.cast()); } }
    }
}

extern "C" {
    static CLSID_CopyToMenu: GUID;
    static CLSID_MoveToMenu: GUID;
}

extern "system" {
    fn HeapReAlloc(heap: HANDLE, flags: DWORD, memory: LPVOID, bytes: usize) -> LPVOID;
    fn GetModuleHandleW(name: LPCWSTR) -> HINSTANCE;
    fn LoadStringW(instance: HINSTANCE, id: UINT, buffer: LPWSTR, capacity: INT) -> INT;
}

pub struct CStringW { buffer: LPWSTR, len: usize, capacity: usize }
impl CStringW {
    pub const fn new() -> Self { Self { buffer: core::ptr::null_mut(), len: 0, capacity: 0 } }
    pub unsafe fn from_ptr(text: LPCWSTR) -> Self {
        let mut value = Self::new(); value.append_ptr(text); value
    }
    pub fn IsEmpty(&self) -> bool { self.len == 0 }
    pub unsafe fn push(&mut self, ch: WCHAR) { self.append_char(ch); }
    pub unsafe fn append_from_ptr(&mut self, text: LPCWSTR) { self.append_ptr(text); }
    // CStringW::Replace(WCHAR, WCHAR) — in-place, returns replacement count.
    pub unsafe fn replace(&mut self, from: WCHAR, to: WCHAR) -> INT {
        let mut count: INT = 0;
        let mut index = 0usize;
        while index < self.len {
            if *self.buffer.add(index) == from { *self.buffer.add(index) = to; count += 1; }
            index += 1;
        }
        count
    }
    // CStringW::Format(UINT nFormatID, ...) — resource-ID flavor over `format`.
    pub unsafe fn Format(&mut self, format_id: UINT, arg: LPCWSTR) {
        self.format(format_id as usize as LPCWSTR, arg);
    }
    pub unsafe fn from_resource(id: UINT) -> Self {
        let mut value = Self::new();
        let mut stack = [0u16; 256];
        let count = LoadStringW(
            shell32_hInstance, id, stack.as_mut_ptr(), stack.len() as INT,
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
    pub unsafe fn assign(&mut self, text: LPCWSTR) {
        self.len = 0;
        if !self.buffer.is_null() { *self.buffer = 0; }
        self.append_ptr(text);
    }
    pub unsafe fn format(&mut self, format: LPCWSTR, arg: LPCWSTR) {
        let mut resource = [0u16; 256];
        let format = if (format as usize) <= 0xffff {
            LoadStringW(shell32_hInstance, format as usize as UINT,
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

// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_75DC0A9E9650: [WCHAR; 5] = [0x0020, 0x002E, 0x002E, 0x002E, 0x0000];
pub static CPP_COM_WSTR_96A296D224F2: [WCHAR; 1] = [0x0000];

use core::ffi::c_void;
use crate::shellcom::*;
use crate::types::*;

pub unsafe fn CMoveToMenu_dtor_body(this: *mut CMoveToMenu) -> () {}

pub unsafe fn CMoveToMenu_ctor_body(this: *mut CMoveToMenu) -> () {}

pub unsafe fn CMoveToMenu_GetCaptionStringID_body(this: *mut CMoveToMenu) -> UINT { IDS_MOVEITEMS }

pub unsafe fn CMoveToMenu_GetButtonStringID_body(this: *mut CMoveToMenu) -> UINT { IDS_MOVEBUTTON }

pub unsafe fn CMoveToMenu_GetActionTitleStringID_body(this: *mut CMoveToMenu) -> UINT { IDS_MOVETOTITLE }

pub unsafe fn CMoveToMenu_GetFileOp_body(this: *mut CMoveToMenu) -> UINT { FO_MOVE }

pub unsafe fn CMoveToMenu_GetVerb_body(this: *mut CMoveToMenu) -> LPCSTR {
    c"moveto".as_ptr()
}

pub unsafe fn CMoveToMenu_DoRealFileOp_body(this: *mut CMoveToMenu, pCIDA: *const CIDA, lpici: LPCMINVOKECOMMANDINFO, pidlDestination: PCUIDLIST_ABSOLUTE) -> HRESULT {
    let mut strFiles = CStringW::new();
    let mut szPath: [WCHAR; MAX_PATH] = core::mem::zeroed();
    
    let mut n = 0;
    while n < (*pCIDA).cidl {
        let pidlCombine = CComHeapPtr::<ITEMIDLIST>::new(
            SHELL_CIDA_ILCloneFull(pCIDA, n)
        );
        if pidlCombine.is_null() {
            return E_FAIL;
        }

        if SHGetPathFromIDListW(pidlCombine.get(), szPath.as_mut_ptr()) == 0 {
            return E_FAIL;
        }

        if n > 0 {
            strFiles.push(0x007Cu16);
        }
        strFiles.append_from_ptr(szPath.as_ptr());
        
        n += 1;
    }

    strFiles.push(0x007Cu16);
    strFiles.replace(0x007Cu16, 0x0000u16);

    if _ILIsDesktop(pidlDestination) != 0 {
        SHGetSpecialFolderPathW(
            (*lpici).hwnd,
            szPath.as_mut_ptr(),
            CSIDL_DESKTOPDIRECTORY,
            TRUE
        );
    } else {
        SHGetPathFromIDListW(pidlDestination, szPath.as_mut_ptr());
    }
    
    let cchPath = lstrlenW(szPath.as_ptr());
    if ((cchPath + 1) as usize) < MAX_PATH {
        szPath[(cchPath + 1) as usize] = 0;
    } else {
        return E_FAIL;
    }

    let mut op = SHFILEOPSTRUCTW {
        hwnd: (*lpici).hwnd,
        wFunc: CMoveToMenu_GetFileOp_body(this),
        pFrom: strFiles.as_ptr(),
        pTo: szPath.as_ptr(),
        fFlags: FOF_ALLOWUNDO,
        fAnyOperationsAborted: 0,
        hNameMappings: core::ptr::null_mut(),
        lpszProgressTitle: core::ptr::null(),
    };
    
    let res = SHFileOperationW(&mut op);
    if res != 0 {
        return E_FAIL;
    }
    S_OK
}

pub unsafe fn DoGetFileTitle_body(pCIDA: *const CIDA, strTitle: &mut CStringW) -> HRESULT {
    let pidlCombine: *mut ITEMIDLIST = SHELL_CIDA_ILCloneFull(pCIDA, 0);
    if pidlCombine.is_null() {
        return E_OUTOFMEMORY;
    }
    let mut szPath: [u16; MAX_PATH] = [0u16; MAX_PATH];
    let hr: HRESULT = SHGetNameAndFlagsW(
        pidlCombine,
        SHGDN_INFOLDER,
        szPath.as_mut_ptr(),
        szPath.len() as UINT,
        core::ptr::null_mut(),
    );
    *strTitle = if hr >= 0 {
        CStringW::from_ptr(szPath.as_ptr())
    } else {
        CStringW::from_ptr(CPP_COM_WSTR_96A296D224F2.as_ptr())
    };
    if strTitle.IsEmpty() {
        CoTaskMemFree(pidlCombine as *mut core::ffi::c_void);
        return E_FAIL;
    }
    if (*pCIDA).cidl > 1 {
        *strTitle += CPP_COM_WSTR_75DC0A9E9650.as_ptr();
    }
    CoTaskMemFree(pidlCombine as *mut core::ffi::c_void);
    S_OK
}

pub unsafe fn CMoveToMenu_DoAction_body(this: *mut CMoveToMenu, lpici: LPCMINVOKECOMMANDINFO) -> HRESULT {
    let mut wszPath: [u16; MAX_PATH] = [0; MAX_PATH];
    let pCIDA = CDataObjectHIDA::new((*this).m_pDataObject);
    let mut hr = pCIDA.hr();
    if ((hr) < 0) {
        return hr;
    }
    if SHGetPathFromIDListW((*this).m_pidlFolder, wszPath.as_mut_ptr()) == 0 {
        return E_FAIL;
    }
    let mut strFileTitle = CStringW::new();
    let cida_ptr: *const CIDA = pCIDA.as_cida_ptr();
    hr = DoGetFileTitle_body(cida_ptr, &mut strFileTitle);
    if hr < 0 {
        return hr;
    }
    let mut strTitle = CStringW::new();
    let fmt_id = CMoveToMenu_GetActionTitleStringID_body(this);
    strTitle.Format(fmt_id, strFileTitle.as_ptr() as LPCWSTR);
    let mut info = BROWSEINFOW {
        hwndOwner: (*lpici).hwnd,
        pidlRoot: core::ptr::null_mut(),
        pszDisplayName: core::ptr::null_mut(),
        lpszTitle: strTitle.as_ptr(),
        ulFlags: BIF_RETURNONLYFSDIRS | BIF_USENEWUI,
        lpfn: Some(BrowseCallbackProc),
        lParam: this as LPARAM,
        iImage: 0,
    };
    let pidl: CComHeapPtr<ITEMIDLIST> = CComHeapPtr::from_raw(SHBrowseForFolder(&mut info));
    if !pidl.is_null() {
        hr = CMoveToMenu_DoRealFileOp_body(this, cida_ptr, lpici, pidl.as_ptr());
    }
    hr
}

pub unsafe fn GetPreviousMenuItemInfo_body(hMenu: HMENU, mut iItem: UINT, lpmii: LPMENUITEMINFOW) -> BOOL {
    let mut bSuccess: BOOL = FALSE;
    while iItem > 0 {
        iItem -= 1;
        bSuccess = GetMenuItemInfoW(hMenu, iItem, TRUE, lpmii);
        if bSuccess != 0 || (bSuccess == 0 && GetLastError() != ERROR_MENU_ITEM_NOT_FOUND) {
            break;
        }
    }
    bSuccess
}

pub unsafe fn CMoveToMenu_QueryContextMenu_body(this: *mut CMoveToMenu, hMenu: HMENU, indexMenu: UINT, idCmdFirst: UINT, idCmdLast: UINT, uFlags: UINT) -> HRESULT {
    CMoveToMenu_QueryContextMenuImpl_body(this, FALSE, hMenu, indexMenu, idCmdFirst, idCmdLast, uFlags)
}

pub unsafe fn CMoveToMenu_QueryContextMenuImpl_body(this: *mut CMoveToMenu, IsCopyOp: BOOL, hMenu: HMENU, mut indexMenu: UINT, idCmdFirst: UINT, idCmdLast: UINT, uFlags: UINT) -> HRESULT {
    if (uFlags & (CMF_NOVERBS | CMF_VERBSONLY)) != 0 {
        return MAKE_HRESULT(SEVERITY_SUCCESS, 0, 0);
    }

    let mut idHighest: UINT = 0;

    // Resource string buffers (mimicking CStringW lifetimes with a stack buffer)
    let mut str_copy_buf: [u16; 128 as usize] =
        [0u16; 128 as usize];
    let mut str_move_buf: [u16; 128 as usize] =
        [0u16; 128 as usize];

    // Get module instance for resource loading (shell32.dll)
    let shell32_name: [u16; 13] = [
        0x0073, 0x0068, 0x0065, 0x006C, 0x006C, // "shell"
        0x0033, 0x0032, 0x002E, 0x0064, 0x006C, 0x006C, // "32.dll"
        0x0000, // terminator, padding for alignment
        0x0000,
    ];
    let h_module = GetModuleHandleW(shell32_name.as_ptr());

    // Load the copy‑to string (CStringW(strCopyTo) equivalent)
    let copy_len = LoadStringW(
        h_module,
        IDS_COPYTOMENU, // MAKEINTRESOURCEW
        str_copy_buf.as_mut_ptr(),
        str_copy_buf.len() as i32,
    );
    if copy_len == 0 {
        str_copy_buf[0] = 0;
    }

    let mut itemText: *mut u16 = str_copy_buf.as_mut_ptr();

    // Load the move‑to string if needed
    if IsCopyOp == 0 {
        let move_len = LoadStringW(
            h_module,
            IDS_MOVETOMENU, // MAKEINTRESOURCEW
            str_move_buf.as_mut_ptr(),
            str_move_buf.len() as i32,
        );
        if move_len == 0 {
            str_move_buf[0] = 0;
        }
        itemText = str_move_buf.as_mut_ptr();
    }

    // Insert separator if necessary
    let mut szBuff: [u16; 128 as usize] =
        [0u16; 128 as usize];
    let mut mii: MENUITEMINFOW = core::mem::zeroed();
    mii.cbSize = core::mem::size_of::<MENUITEMINFOW>() as UINT;
    mii.fMask = MIIM_TYPE;
    mii.dwTypeData = szBuff.as_mut_ptr();
    mii.cch = 128; // szBuff.len()

    let bPrev = GetPreviousMenuItemInfo_body(hMenu, indexMenu, &mut mii as *mut _);

    // Condition for inserting a separator:
    // mii.fType != COMMENT_TOKEN_ECHO____
    //   && (IsCopyOp || !(mii.fType == COMMENT_TOKEN_ECHO____
    //                     && !COMMENT_TOKEN_ECHO____(szBuff, strCopyTo.GetBuffer())))
    if bPrev != 0
        && mii.fType != MFT_SEPARATOR
        && (IsCopyOp != 0
            || {
                if mii.fType == MFT_STRING {
                    let cmp = wcscmp(
                        szBuff.as_ptr(),
                        str_copy_buf.as_ptr(),
                    );
                    cmp != 0
                } else {
                    true
                }
            })
    {
        mii.fMask = MIIM_TYPE;
        mii.fType = MFT_SEPARATOR;
        mii.dwTypeData = core::ptr::null_mut();
        mii.cch = 0;
        if InsertMenuItemW(hMenu, indexMenu, TRUE, &mut mii as *mut _) != 0
        {
            indexMenu += 1;
        }
    }

    // Insert the main menu item (copy‑to / move‑to)
    mii.fMask = MIIM_ID | MIIM_TYPE;
    mii.fType = MFT_STRING;
    mii.dwTypeData = itemText;
    mii.wID = idCmdFirst + IDC_ACTION;
    if InsertMenuItemW(hMenu, indexMenu, TRUE, &mut mii as *mut _) != 0 {
        idHighest = core::cmp::max(idHighest, mii.wID);
        indexMenu += 1;
    }

    if idHighest >= idCmdFirst {
        return MAKE_HRESULT(
            SEVERITY_SUCCESS,
            0,
            idHighest - idCmdFirst + 1,
        );
    } else {
        return E_FAIL;
    }
}

pub unsafe fn CMoveToMenu_InvokeCommand_body(this: *mut CMoveToMenu, lpici: LPCMINVOKECOMMANDINFO) -> HRESULT {
    let mut hr: HRESULT = E_FAIL;
    if IS_INTRESOURCE((*lpici).lpVerb) {
        if LOWORD!((*lpici).lpVerb) == IDC_ACTION {
            hr = CMoveToMenu_DoAction_body(this, lpici);
        }
    } else {
        if lstrcmpiA((*lpici).lpVerb, CMoveToMenu_GetVerb_body(this)) == 0 {
            hr = CMoveToMenu_DoAction_body(this, lpici);
        }
    }
    hr
}

pub unsafe fn CMoveToMenu_GetCommandString_body(this: *mut CMoveToMenu, idCmd: UINT_PTR, uType: UINT, pwReserved: *mut UINT, pszName: LPSTR, cchMax: UINT) -> HRESULT {
    if (uType | GCS_UNICODE) == GCS_VALIDATEW {
        return if idCmd == IDC_ACTION as UINT_PTR { S_OK } else { S_FALSE };
    }
    if uType == GCS_VERBW && idCmd == IDC_ACTION as UINT_PTR {
        return SHAnsiToUnicode(CMoveToMenu_GetVerb_body(this), pszName as *mut u16, cchMax as INT) as HRESULT;
    }
    return E_NOTIMPL;
}

pub unsafe fn CMoveToMenu_HandleMenuMsg_body(this: *mut CMoveToMenu, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> HRESULT { E_NOTIMPL }

pub unsafe fn CMoveToMenu_Initialize_body(this: *mut CMoveToMenu, pidlFolder: PCIDLIST_ABSOLUTE, pdtobj: *mut IDataObject, hkeyProgID: HKEY) -> HRESULT {
    CMoveToMenu_cpp_com_assign_m_pDataObject(this, pdtobj);
    let mut hr: HRESULT = E_FAIL;
    // NOTE: the machine translation had SHILClone/ILRemoveLastID swapped between
    // the two branches; restored to match CCopyMoveToMenu.cpp:362-384.
    let mut pidlFolder: PIDLIST_ABSOLUTE = pidlFolder as PIDLIST_ABSOLUTE;

    if !pidlFolder.is_null() {
        // C++: hr = SHILClone(pidlFolder, &pidlFolder);
        let mut pidlClone: LPITEMIDLIST = core::ptr::null_mut();
        hr = SHILClone(pidlFolder, &mut pidlClone);
        pidlFolder = pidlClone;
    } else {
        pidlFolder = SHELL_DataObject_ILCloneFullItem(pdtobj, 0);
        if !pidlFolder.is_null() {
            ILRemoveLastID(pidlFolder);
            hr = S_OK;
        }
    }
    if hr >= 0 {
        // C++: m_pidlFolder.Attach(...) — the CComHeapPtr member frees any prior pidl.
        if !(*this).m_pidlFolder.is_null() { CoTaskMemFree((*this).m_pidlFolder.cast()); }
        (*this).m_pidlFolder = pidlFolder;
    }
    hr
}

pub unsafe fn CMoveToMenu_SetSite_body(this: *mut CMoveToMenu, pUnkSite: *mut IUnknown) -> HRESULT {
    CMoveToMenu_cpp_com_assign_m_pSite(this, pUnkSite);
    return S_OK;
}

pub unsafe fn CMoveToMenu_GetSite_body(this: *mut CMoveToMenu, riid: REFIID, ppvSite: *mut PVOID) -> HRESULT {
    let pSite = (*this).m_pSite;
    if pSite.is_null() {
        return E_FAIL;
    }
    ((*(*pSite).lpVtbl).QueryInterface)(pSite, riid, ppvSite)
}

// ============================================================================
// Substrate closure — deterministic support for the translated bodies above.
// Ground truth: dll/win32/shell32/CCopyMoveToMenu.cpp/.h,
// sdk/include/reactos/shellutils.h, dll/win32/shell32/shresdef.h.
// ============================================================================

pub type PCUIDLIST_ABSOLUTE = LPCITEMIDLIST; // shtypes.h (shellcom.rs has only the non-U flavor)
pub type WNDPROC = Option<unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT>; // winuser.h
pub type LPMENUITEMINFOW = *mut MENUITEMINFOW; // winuser.h
pub type BFFCALLBACK = Option<unsafe extern "system" fn(HWND, UINT, LPARAM, LPARAM) -> INT>; // shlobj.h

// CCopyMoveToMenu.cpp:12 `enum { IDC_ACTION = 0 };`
pub const IDC_ACTION: UINT = 0;
// shresdef.h:382
pub const IDS_MOVEBUTTON: UINT = 30539;
// shlobj.h
pub const BIF_USENEWUI: DWORD = 0x0040 | 0x0010; // BIF_NEWDIALOGSTYLE | BIF_EDITBOX
pub const BFFM_ENABLEOK: UINT = 0x0400 + 101; // WM_USER + 101
pub const BFFM_SETSELECTIONW: UINT = 0x0400 + 103; // WM_USER + 103 (BFFM_SETSELECTION, UNICODE)
// winuser.h
pub const GWLP_WNDPROC: INT = -4;
pub const GWLP_USERDATA: INT = -21;

macro_rules! HIWORD { ($value:expr) => { ((($value as usize) >> 16) & 0xffff) as UINT }; }

// winuser.h IS_INTRESOURCE — spelling copied from open_with_menu.rs:834.
#[inline]
fn IS_INTRESOURCE(value: LPCSTR) -> bool {
    (value as usize) >> 16 == 0
}

// shlobj.h BROWSEINFOW
#[repr(C)]
pub struct BROWSEINFOW {
    pub hwndOwner: HWND,
    pub pidlRoot: PCIDLIST_ABSOLUTE,
    pub pszDisplayName: LPWSTR,
    pub lpszTitle: LPCWSTR,
    pub ulFlags: UINT,
    pub lpfn: BFFCALLBACK,
    pub lParam: LPARAM,
    pub iImage: INT,
}

// shellapi.h SHFILEOPSTRUCTW (fFlags is FILEOP_FLAGS = WORD)
#[repr(C)]
pub struct SHFILEOPSTRUCTW {
    pub hwnd: HWND,
    pub wFunc: UINT,
    pub pFrom: LPCWSTR,
    pub pTo: LPCWSTR,
    pub fFlags: WORD,
    pub fAnyOperationsAborted: BOOL,
    pub hNameMappings: LPVOID,
    pub lpszProgressTitle: LPCWSTR,
}

// -- real DLL exports (WINAPI/stdcall -> extern "system"; i386 decoration) --
extern "system" {
    fn SHGetSpecialFolderPathW(hwnd: HWND, pszPath: LPWSTR, csidl: core::ffi::c_int, fCreate: BOOL) -> BOOL; // shell32.spec:320
    fn SHFileOperationW(lpFileOp: *mut SHFILEOPSTRUCTW) -> core::ffi::c_int; // shell32.spec:291
    #[link_name = "SHBrowseForFolderW"] // UNICODE TCHAR macro resolves to the W export
    fn SHBrowseForFolder(lpbi: *mut BROWSEINFOW) -> PIDLIST_ABSOLUTE; // shell32.spec:274
    fn ILClone(pidl: LPCITEMIDLIST) -> LPITEMIDLIST; // shell32.spec:17
    fn ILIsEqual(pidl1: LPCITEMIDLIST, pidl2: LPCITEMIDLIST) -> BOOL; // shell32.spec:20
    fn SHAnsiToUnicode(src: LPCSTR, dst: LPWSTR, cch: INT) -> DWORD; // shlwapi (shell_link.rs:5228 spelling)
    fn lstrcmpiA(lpString1: LPCSTR, lpString2: LPCSTR) -> INT; // kernel32
    fn SendMessageW(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT; // user32
    fn PostMessageW(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> BOOL; // user32
    fn SetWindowTextW(hWnd: HWND, lpString: LPCWSTR) -> BOOL; // user32
    fn SetDlgItemTextW(hDlg: HWND, nIDDlgItem: INT, lpString: LPCWSTR) -> BOOL; // user32
    fn GetDlgItemTextW(hDlg: HWND, nIDDlgItem: INT, lpString: LPWSTR, cchMax: INT) -> UINT; // user32
    fn CallWindowProcW(lpPrevWndFunc: WNDPROC, hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT; // user32
    fn StrTrimW(psz: LPWSTR, pszTrimChars: LPCWSTR) -> BOOL; // shlwapi (cfs_folder.rs:3112 spelling)
    fn PathIsRelativeW(pszPath: LPCWSTR) -> BOOL; // shlwapi
    fn PathIsDirectoryW(pszPath: LPCWSTR) -> BOOL; // shlwapi
    fn PathFileExistsW(pszPath: LPCWSTR) -> BOOL; // shlwapi
}

// ARCH-SPLIT (explorer-sys/src/server.rs precedent): on i386 Get/SetWindowLongPtrW
// are C macros over Get/SetWindowLongW — the *Ptr exports exist only on 64-bit
// (user32.spec:400-401,704-705).
#[cfg(target_arch = "x86")]
extern "system" {
    fn GetWindowLongW(hWnd: HWND, nIndex: INT) -> LONG;
    fn SetWindowLongW(hWnd: HWND, nIndex: INT, dwNewLong: LONG) -> LONG;
}
#[cfg(not(target_arch = "x86"))]
extern "system" {
    fn GetWindowLongPtrW(hWnd: HWND, nIndex: INT) -> LONG_PTR;
    fn SetWindowLongPtrW(hWnd: HWND, nIndex: INT, dwNewLong: LONG_PTR) -> LONG_PTR;
}
#[inline]
unsafe fn GetWindowLongPtr(hwnd: HWND, index: INT) -> LONG_PTR {
    #[cfg(target_arch = "x86")]
    { GetWindowLongW(hwnd, index) as LONG_PTR }
    #[cfg(not(target_arch = "x86"))]
    { GetWindowLongPtrW(hwnd, index) }
}
#[inline]
unsafe fn SetWindowLongPtr(hwnd: HWND, index: INT, value: LONG_PTR) -> LONG_PTR {
    #[cfg(target_arch = "x86")]
    { SetWindowLongW(hwnd, index, value as LONG) as LONG_PTR }
    #[cfg(not(target_arch = "x86"))]
    { SetWindowLongPtrW(hwnd, index, value) }
}

// wine/shell32_main.h:867 precedent (module instance for resource loads) — the C++
// CStringW resource constructors load through _AtlBaseModule (= shell32's own
// instance), NOT the process module; see shell_link.rs / open_with_menu.rs siblings.
extern "C" {
    static shell32_hInstance: HINSTANCE;
}

// -- Rust bridges to retained C++ (EXTERN_C ... WINAPI shims C++-side) --
extern "system" {
    // EXISTING bridges (created by the CFSFolder/CDesktopFolder landings —
    // spellings copied from desktop_folder.rs:2334-2338):
    #[link_name = "SHGetNameAndFlagsW_RustBridge"]
    fn SHGetNameAndFlagsW(pidl: LPCITEMIDLIST, flags: DWORD, text: LPWSTR, cch: UINT, attributes: *mut DWORD) -> HRESULT;
    #[link_name = "SHELL_DataObject_ILCloneFullItem_RustBridge"]
    fn SHELL_DataObject_ILCloneFullItem(pDO: *mut IDataObject, Index: UINT) -> PIDLIST_ABSOLUTE;
    // NEW — must be created C++-side (integration TODO). Expected shims, next to
    // SHELL_DataObject_ILCloneFullItem_RustBridge in shldataobject.cpp:
    //   extern "C" PIDLIST_ABSOLUTE WINAPI
    //   SHELL_CIDA_ILCloneFull_RustBridge(const CIDA *pCIDA, UINT Index)
    //       { return SHELL_CIDA_ILCloneFull(pCIDA, Index); }
    #[link_name = "SHELL_CIDA_ILCloneFull_RustBridge"]
    fn SHELL_CIDA_ILCloneFull(pCIDA: *const CIDA, Index: UINT) -> PIDLIST_ABSOLUTE;
    //   extern "C" HRESULT WINAPI
    //   CDataObjectHIDA_CreateCIDA_RustBridge(IDataObject *pDataObject, CIDA **ppcida, STGMEDIUM *pmedium)
    //       { return CDataObjectHIDA::CreateCIDA(pDataObject, ppcida, *pmedium); }
    #[link_name = "CDataObjectHIDA_CreateCIDA_RustBridge"]
    fn CDataObjectHIDA_CreateCIDA(pDataObject: *mut IDataObject, ppcida: *mut *mut CIDA, pmedium: *mut STGMEDIUM) -> HRESULT;
    //   extern "C" void WINAPI
    //   CDataObjectHIDA_DestroyCIDA_RustBridge(CIDA *pcida, STGMEDIUM *pmedium)
    //       { CDataObjectHIDA::DestroyCIDA(pcida, *pmedium); }
    #[link_name = "CDataObjectHIDA_DestroyCIDA_RustBridge"]
    fn CDataObjectHIDA_DestroyCIDA(pcida: *mut CIDA, pmedium: *mut STGMEDIUM);
}

// shellutils.h:462-466 template SHILClone — header-inline, no export; local impl
// over the real ILClone export (cfs_folder.rs/recycle_bin.rs sibling idiom).
#[inline]
unsafe fn SHILClone(pidl: LPCITEMIDLIST, ppOut: *mut LPITEMIDLIST) -> HRESULT {
    *ppOut = ILClone(pidl);
    if (*ppOut).is_null() { E_OUTOFMEMORY } else { S_OK }
}

// shellutils.h:738 CDataObjectHIDA — RAII view of the HIDA in an IDataObject,
// over the two bridged statics above (ctor = CreateCIDA, Drop = DestroyCIDA).
pub struct CDataObjectHIDA {
    m_medium: STGMEDIUM,
    m_cida: *mut CIDA,
    m_hr: HRESULT,
}
impl CDataObjectHIDA {
    pub unsafe fn new(pDataObject: *mut IDataObject) -> Self {
        let mut medium: STGMEDIUM = core::mem::zeroed();
        let mut cida: *mut CIDA = core::ptr::null_mut();
        let hr = CDataObjectHIDA_CreateCIDA(pDataObject, &mut cida, &mut medium);
        Self { m_medium: medium, m_cida: cida, m_hr: hr }
    }
    pub fn hr(&self) -> HRESULT { self.m_hr }
    pub fn as_cida_ptr(&self) -> *const CIDA { self.m_cida }
}
impl Drop for CDataObjectHIDA {
    fn drop(&mut self) {
        unsafe { CDataObjectHIDA_DestroyCIDA(self.m_cida, core::ptr::addr_of_mut!(self.m_medium)); }
    }
}

// L" \t" (StrTrimW trim set in WindowProc)
static CPP_COM_WSTR_SPACE_TAB: [WCHAR; 3] = [0x0020, 0x0009, 0x0000];

// CCopyMoveToMenu.cpp:20-60 WindowProc — file-static subclass proc the scanner
// missed; hand-translated faithfully (installed over the SHBrowseForFolder dialog
// by BrowseCallbackProc below).
unsafe extern "system" fn WindowProc(hwnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    let mut szPath: [WCHAR; MAX_PATH] = [0; MAX_PATH];
    let this_ = GetWindowLongPtr(hwnd, GWLP_USERDATA) as *mut CMoveToMenu;

    if uMsg == WM_COMMAND && LOWORD!(wParam) == IDC_BROWSE_FOR_FOLDER_FOLDER_TEXT {
        if HIWORD!(wParam) == EN_CHANGE {
            if (*this_).m_bIgnoreTextBoxChange == 0 {
                // get the text
                GetDlgItemTextW(hwnd, IDC_BROWSE_FOR_FOLDER_FOLDER_TEXT as INT,
                                szPath.as_mut_ptr(), MAX_PATH as INT);
                StrTrimW(szPath.as_mut_ptr(), CPP_COM_WSTR_SPACE_TAB.as_ptr());

                // update OK button
                let bValid: BOOL =
                    (PathIsRelativeW(szPath.as_ptr()) == 0 && PathIsDirectoryW(szPath.as_ptr()) != 0) as BOOL;
                SendMessageW(hwnd, BFFM_ENABLEOK, 0, bValid as LPARAM);

                return 0;
            }

            // reset flag
            (*this_).m_bIgnoreTextBoxChange = FALSE;
        }
    }
    CallWindowProcW((*this_).m_fnOldWndProc, hwnd, uMsg, wParam, lParam)
}

// CCopyMoveToMenu.cpp:62-121 BrowseCallbackProc — file-static SHBrowseForFolder
// callback the scanner missed; hand-translated faithfully.
unsafe extern "system" fn BrowseCallbackProc(hwnd: HWND, uMsg: UINT, lParam: LPARAM, lpData: LPARAM) -> INT {
    let mut this_ = GetWindowLongPtr(hwnd, GWLP_USERDATA) as *mut CMoveToMenu;

    match uMsg {
        BFFM_INITIALIZED => {
            SetWindowLongPtr(hwnd, GWLP_USERDATA, lpData as LONG_PTR);
            this_ = lpData as *mut CMoveToMenu;

            // Select initial directory
            SendMessageW(hwnd, BFFM_SETSELECTIONW, FALSE as WPARAM,
                         (*this_).m_pidlFolder as LPARAM);

            // Set caption
            let strCaption = CStringW::new_from_lpcwstr(
                CMoveToMenu_GetCaptionStringID_body(this_) as usize as LPCWSTR);
            SetWindowTextW(hwnd, strCaption.as_ptr());

            // Set OK button text
            let strCopyOrMove = CStringW::new_from_lpcwstr(
                CMoveToMenu_GetButtonStringID_body(this_) as usize as LPCWSTR);
            SetDlgItemTextW(hwnd, IDOK as INT, strCopyOrMove.as_ptr());

            // Subclassing
            (*this_).m_fnOldWndProc = core::mem::transmute::<LONG_PTR, WNDPROC>(
                SetWindowLongPtr(hwnd, GWLP_WNDPROC,
                    WindowProc as unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT
                        as usize as LONG_PTR));

            // Disable OK
            PostMessageW(hwnd, BFFM_ENABLEOK, 0, FALSE as LPARAM);
        }
        BFFM_SELCHANGED => {
            if !this_.is_null() {
                let mut szPath: [WCHAR; MAX_PATH] = [0; MAX_PATH];
                let pidl = lParam as LPCITEMIDLIST;

                szPath[0] = 0;
                SHGetPathFromIDListW(pidl, szPath.as_mut_ptr());

                if ILIsEqual(pidl, (*this_).m_pidlFolder) != 0 {
                    PostMessageW(hwnd, BFFM_ENABLEOK, 0,
                                 (CMoveToMenu_GetFileOp_body(this_) == FO_COPY) as LPARAM);
                } else if PathFileExistsW(szPath.as_ptr()) != 0 || _ILIsDesktop(pidl) != 0 {
                    PostMessageW(hwnd, BFFM_ENABLEOK, 0, TRUE as LPARAM);
                } else {
                    PostMessageW(hwnd, BFFM_ENABLEOK, 0, FALSE as LPARAM);
                }

                // the text box will be updated later soon, ignore it
                (*this_).m_bIgnoreTextBoxChange = TRUE;
            }
        }
        _ => {}
    }

    FALSE
}
