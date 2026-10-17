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

// Deterministic COM scaffold for CDrvDefExt; method bodies are generated separately.
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
pub struct CDrvDefExt {
    pub IShellExtInit_iface: IShellExtInit,
    pub IContextMenu_iface: IContextMenu,
    pub IShellPropSheetExt_iface: IShellPropSheetExt,
    pub IObjectWithSite_iface: IObjectWithSite,
    pub ref_count: core::sync::atomic::AtomicI32,
    pub m_wszDrive: [WCHAR; MAX_PATH],
    pub m_FreeSpacePerc: UINT,
    pub m_Multiple: *mut IDataObject,
}

unsafe fn CDrvDefExt_cpp_com_assign_m_Multiple(this: *mut CDrvDefExt, value: *mut IDataObject) {
    let old = (*this).m_Multiple;
    if old == value { return; }
    if !value.is_null() {
        let unknown = value as *mut IUnknown;
        let add_ref: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).AddRef);
        add_ref(unknown);
    }
    (*this).m_Multiple = value;
    if !old.is_null() {
        let unknown = old as *mut IUnknown;
        let release: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).Release);
        release(unknown);
    }
}

unsafe extern "system" fn CDrvDefExt_IShellExtInit_QueryInterface(This: *mut IShellExtInit, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDrvDefExt, IShellExtInit_iface)) as *mut CDrvDefExt;
    CDrvDefExt_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CDrvDefExt_IShellExtInit_AddRef(This: *mut IShellExtInit) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDrvDefExt, IShellExtInit_iface)) as *mut CDrvDefExt;
    CDrvDefExt_AddRef(this)
}
unsafe extern "system" fn CDrvDefExt_IShellExtInit_Release(This: *mut IShellExtInit) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDrvDefExt, IShellExtInit_iface)) as *mut CDrvDefExt;
    CDrvDefExt_Release(this)
}
unsafe extern "system" fn CDrvDefExt_IShellExtInit_Initialize(This: *mut IShellExtInit, pidlFolder: PCIDLIST_ABSOLUTE, pdtobj: *mut IDataObject, hkeyProgID: HKEY) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDrvDefExt, IShellExtInit_iface)) as *mut CDrvDefExt;
    CDrvDefExt_Initialize_body(this, pidlFolder, pdtobj, hkeyProgID)
}

pub static CDRVDEFEXT_ISHELLEXTINIT_VTBL: IShellExtInitVtbl = IShellExtInitVtbl {
    QueryInterface: CDrvDefExt_IShellExtInit_QueryInterface,
    AddRef: CDrvDefExt_IShellExtInit_AddRef,
    Release: CDrvDefExt_IShellExtInit_Release,
    Initialize: CDrvDefExt_IShellExtInit_Initialize,
};

unsafe extern "system" fn CDrvDefExt_IContextMenu_QueryInterface(This: *mut IContextMenu, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDrvDefExt, IContextMenu_iface)) as *mut CDrvDefExt;
    CDrvDefExt_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CDrvDefExt_IContextMenu_AddRef(This: *mut IContextMenu) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDrvDefExt, IContextMenu_iface)) as *mut CDrvDefExt;
    CDrvDefExt_AddRef(this)
}
unsafe extern "system" fn CDrvDefExt_IContextMenu_Release(This: *mut IContextMenu) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDrvDefExt, IContextMenu_iface)) as *mut CDrvDefExt;
    CDrvDefExt_Release(this)
}
unsafe extern "system" fn CDrvDefExt_IContextMenu_QueryContextMenu(This: *mut IContextMenu, hmenu: HMENU, indexMenu: UINT, idCmdFirst: UINT, idCmdLast: UINT, uFlags: UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDrvDefExt, IContextMenu_iface)) as *mut CDrvDefExt;
    CDrvDefExt_QueryContextMenu_body(this, hmenu, indexMenu, idCmdFirst, idCmdLast, uFlags)
}
unsafe extern "system" fn CDrvDefExt_IContextMenu_InvokeCommand(This: *mut IContextMenu, lpici: LPCMINVOKECOMMANDINFO) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDrvDefExt, IContextMenu_iface)) as *mut CDrvDefExt;
    CDrvDefExt_InvokeCommand_body(this, lpici)
}
unsafe extern "system" fn CDrvDefExt_IContextMenu_GetCommandString(This: *mut IContextMenu, idCmd: UINT_PTR, uType: UINT, pwReserved: *mut UINT, pszName: LPSTR, cchMax: UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDrvDefExt, IContextMenu_iface)) as *mut CDrvDefExt;
    CDrvDefExt_GetCommandString_body(this, idCmd, uType, pwReserved, pszName, cchMax)
}

pub static CDRVDEFEXT_ICONTEXTMENU_VTBL: IContextMenuVtbl = IContextMenuVtbl {
    QueryInterface: CDrvDefExt_IContextMenu_QueryInterface,
    AddRef: CDrvDefExt_IContextMenu_AddRef,
    Release: CDrvDefExt_IContextMenu_Release,
    QueryContextMenu: CDrvDefExt_IContextMenu_QueryContextMenu,
    InvokeCommand: CDrvDefExt_IContextMenu_InvokeCommand,
    GetCommandString: CDrvDefExt_IContextMenu_GetCommandString,
};

unsafe extern "system" fn CDrvDefExt_IShellPropSheetExt_QueryInterface(This: *mut IShellPropSheetExt, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDrvDefExt, IShellPropSheetExt_iface)) as *mut CDrvDefExt;
    CDrvDefExt_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CDrvDefExt_IShellPropSheetExt_AddRef(This: *mut IShellPropSheetExt) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDrvDefExt, IShellPropSheetExt_iface)) as *mut CDrvDefExt;
    CDrvDefExt_AddRef(this)
}
unsafe extern "system" fn CDrvDefExt_IShellPropSheetExt_Release(This: *mut IShellPropSheetExt) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDrvDefExt, IShellPropSheetExt_iface)) as *mut CDrvDefExt;
    CDrvDefExt_Release(this)
}
unsafe extern "system" fn CDrvDefExt_IShellPropSheetExt_AddPages(This: *mut IShellPropSheetExt, pfnAddPage: LPFNSVADDPROPSHEETPAGE, lParam: LPARAM) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDrvDefExt, IShellPropSheetExt_iface)) as *mut CDrvDefExt;
    CDrvDefExt_AddPages_body(this, pfnAddPage, lParam)
}
unsafe extern "system" fn CDrvDefExt_IShellPropSheetExt_ReplacePage(This: *mut IShellPropSheetExt, uPageID: EXPPS, pfnReplaceWith: LPFNSVADDPROPSHEETPAGE, lParam: LPARAM) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDrvDefExt, IShellPropSheetExt_iface)) as *mut CDrvDefExt;
    CDrvDefExt_ReplacePage_body(this, uPageID, pfnReplaceWith, lParam)
}

pub static CDRVDEFEXT_ISHELLPROPSHEETEXT_VTBL: IShellPropSheetExtVtbl = IShellPropSheetExtVtbl {
    QueryInterface: CDrvDefExt_IShellPropSheetExt_QueryInterface,
    AddRef: CDrvDefExt_IShellPropSheetExt_AddRef,
    Release: CDrvDefExt_IShellPropSheetExt_Release,
    AddPages: CDrvDefExt_IShellPropSheetExt_AddPages,
    ReplacePage: CDrvDefExt_IShellPropSheetExt_ReplacePage,
};

unsafe extern "system" fn CDrvDefExt_IObjectWithSite_QueryInterface(This: *mut IObjectWithSite, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDrvDefExt, IObjectWithSite_iface)) as *mut CDrvDefExt;
    CDrvDefExt_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CDrvDefExt_IObjectWithSite_AddRef(This: *mut IObjectWithSite) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDrvDefExt, IObjectWithSite_iface)) as *mut CDrvDefExt;
    CDrvDefExt_AddRef(this)
}
unsafe extern "system" fn CDrvDefExt_IObjectWithSite_Release(This: *mut IObjectWithSite) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDrvDefExt, IObjectWithSite_iface)) as *mut CDrvDefExt;
    CDrvDefExt_Release(this)
}
unsafe extern "system" fn CDrvDefExt_IObjectWithSite_SetSite(This: *mut IObjectWithSite, pUnkSite: *mut IUnknown) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDrvDefExt, IObjectWithSite_iface)) as *mut CDrvDefExt;
    CDrvDefExt_SetSite_body(this, pUnkSite)
}
unsafe extern "system" fn CDrvDefExt_IObjectWithSite_GetSite(This: *mut IObjectWithSite, riid: REFIID, ppvSite: *mut PVOID) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDrvDefExt, IObjectWithSite_iface)) as *mut CDrvDefExt;
    CDrvDefExt_GetSite_body(this, riid, ppvSite)
}

pub static CDRVDEFEXT_IOBJECTWITHSITE_VTBL: IObjectWithSiteVtbl = IObjectWithSiteVtbl {
    QueryInterface: CDrvDefExt_IObjectWithSite_QueryInterface,
    AddRef: CDrvDefExt_IObjectWithSite_AddRef,
    Release: CDrvDefExt_IObjectWithSite_Release,
    SetSite: CDrvDefExt_IObjectWithSite_SetSite,
    GetSite: CDrvDefExt_IObjectWithSite_GetSite,
};

unsafe fn CDrvDefExt_QueryInterface(
    this: *mut CDrvDefExt, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
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
    CDrvDefExt_AddRef(this);
    S_OK
}

unsafe fn CDrvDefExt_AddRef(this: *mut CDrvDefExt) -> ULONG {
    (*this).ref_count.fetch_add(1, core::sync::atomic::Ordering::SeqCst) as ULONG + 1
}

unsafe fn CDrvDefExt_Release(this: *mut CDrvDefExt) -> ULONG {
    let refs = (*this).ref_count.fetch_sub(1, core::sync::atomic::Ordering::SeqCst) - 1;
    if refs == 0 {
        CDrvDefExt_dtor_body(this);
        CDrvDefExt_cpp_com_assign_m_Multiple(this, core::ptr::null_mut());
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

// Activation: CLSID_ShellDrvDefExt currently maps to CDrvDefExt in shell32.cpp.
// The generated C++ adapter redirects ATL activation to this factory.
#[no_mangle] pub unsafe extern "system" fn CDrvDefExt_CreateInstance(
    outer: *mut IUnknown, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if !outer.is_null() { return CLASS_E_NOAGGREGATION; }
    let this = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<CDrvDefExt>()) as *mut CDrvDefExt;
    if this.is_null() { return E_OUTOFMEMORY; }
    core::ptr::write(this, CDrvDefExt {
        IShellExtInit_iface: IShellExtInit { lpVtbl: core::ptr::addr_of!(CDRVDEFEXT_ISHELLEXTINIT_VTBL) },
        IContextMenu_iface: IContextMenu { lpVtbl: core::ptr::addr_of!(CDRVDEFEXT_ICONTEXTMENU_VTBL) },
        IShellPropSheetExt_iface: IShellPropSheetExt { lpVtbl: core::ptr::addr_of!(CDRVDEFEXT_ISHELLPROPSHEETEXT_VTBL) },
        IObjectWithSite_iface: IObjectWithSite { lpVtbl: core::ptr::addr_of!(CDRVDEFEXT_IOBJECTWITHSITE_VTBL) },
        ref_count: core::sync::atomic::AtomicI32::new(1),
        m_wszDrive: [0; MAX_PATH],
        m_FreeSpacePerc: core::mem::zeroed(),
        m_Multiple: core::mem::zeroed(),
    });
    Shell32RustObjectLock();
    CDrvDefExt_ctor_body(this);
    let hr = CDrvDefExt_QueryInterface(this, riid, ppv);
    CDrvDefExt_Release(this);
    hr
}

// Deterministic C++ support closure selected from source identifiers.

pub const IDD_CHECK_DISK: UINT = 26;

pub const IDD_DRIVE_HARDWARE: UINT = 12;

pub const IDD_DRIVE_PROPERTIES: UINT = 10;

pub const IDD_DRIVE_TOOLS: UINT = 11;

pub const IDI_SHELL_3_14_FLOPPY: UINT = 7;

pub const IDI_SHELL_CDROM: UINT = 12;

pub const IDI_SHELL_DRIVE: UINT = 9;

pub const IDI_SHELL_NETDRIVE: UINT = 10;

pub const IDI_SHELL_RAMDISK: UINT = 13;

pub const IDI_SHELL_REMOVEABLE: UINT = 8;

pub const IDR_DRVDEFEXT: UINT = 148;

pub const IDS_DRIVE_CDROM: UINT = 73;

pub const IDS_DRIVE_FIXED: UINT = 72;

pub const IDS_DRIVE_FLOPPY: UINT = 75;

pub const IDS_DRIVE_NETWORK: UINT = 74;

pub const IDS_DRIVE_REMOVABLE: UINT = 76;

pub const IDS_FS_UNKNOWN: UINT = 77;

pub const BLACK_PEN: DWORD = 7;

pub const BM_GETCHECK: DWORD = 240;

pub const BST_CHECKED: DWORD = 1;

pub const CF_HDROP: DWORD = 15;

pub const DONE: DWORD = 0x01;

pub const DRIVE_CDROM: DWORD = 5;

pub const DRIVE_FIXED: DWORD = 3;

pub const DRIVE_NO_ROOT_DIR: DWORD = 1;

pub const DRIVE_RAMDISK: DWORD = 6;

pub const DRIVE_REMOTE: DWORD = 4;

pub const DRIVE_REMOVABLE: DWORD = 2;

pub const DVASPECT_CONTENT: DWORD = 1;

pub const DWLP_MSGRESULT: DWORD = 0;

pub const EM_SETREADONLY: DWORD = 207;

pub const EN_CHANGE: DWORD = 768;

pub const FILE_FLOPPY_DISKETTE: DWORD = 0x00000004;

pub const FILE_READ_ATTRIBUTES: DWORD = 0x00000080;

pub const FILE_SHARE_READ: DWORD = 0x00000001;

pub const FILE_SHARE_WRITE: DWORD = 0x00000002;

pub const HWPD_STANDARDLIST: DWORD = 0;

pub const IDCANCEL: DWORD = 2;

pub const IDOK: DWORD = 1;

pub const IMAGE_ICON: DWORD = 1;

pub const LR_SHARED: DWORD = 32768;

// C++: _USE_MATH_DEFINES math.h M_PI. (The generated closure emitted `DWORD = 3`,
// a truncation that would flatten the pie-chart trigonometry — pinned to the real value.)
pub const M_PI: f64 = 3.14159265358979323846;

pub const NO_ERROR: DWORD = 0;

pub const OPEN_EXISTING: DWORD = 3;

pub const PSNRET_INVALID: DWORD = 1;

pub const PSNRET_NOERROR: DWORD = 0;

pub const PS_SOLID: DWORD = 0;

pub const SHGDN_INFOLDER: DWORD = 1;

pub const STM_SETICON: DWORD = 368;

pub const SW_SHOW: DWORD = 5;

pub const TYMED_HGLOBAL: DWORD = 1;

pub const WM_COMMAND: DWORD = 273;

pub const WM_DRAWITEM: DWORD = 43;

pub const WM_INITDIALOG: DWORD = 272;

pub const WM_PAINT: DWORD = 15;

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

pub type EXPPS = UINT;

pub type LPFNSVADDPROPSHEETPAGE = Option<unsafe extern "system" fn(HPROPSHEETPAGE, LPARAM) -> BOOL>;

pub type ITEMIDLIST_ABSOLUTE = ITEMIDLIST;

extern "system" {
    #[link_name = "LoadImageW"] fn LoadImage(instance: HINSTANCE, name: LPCWSTR, kind: UINT, width: INT, height: INT, flags: UINT) -> HANDLE;
    fn CoTaskMemFree(memory: LPVOID);
}

// ── Deterministic substrate additions (compile closure; donor: shell_link.rs) ──────────

// Scalar/type aliases not provided by types.rs/shellcom.rs.
pub type CLIPFORMAT = WORD; // wtypes.h
pub type LONGLONG = i64; // winnt.h
pub type size_t = usize;
pub type HDROP = HANDLE;
pub use crate::island::HBRUSH;
pub use crate::island::HPEN;
pub use crate::island::HGDIOBJ;
pub type LPDRAWITEMSTRUCT = *mut DRAWITEMSTRUCT;
pub type LPFNADDPROPSHEETPAGE = LPFNSVADDPROPSHEETPAGE; // prsht.h (same shape)
// winuser.h DLGPROC; prsht.h LPFNPSPCALLBACKW (donor shell_link.rs:4899-4900)
pub type DLGPROC = unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> INT_PTR;
pub type LPFNPSPCALLBACKW = unsafe extern "system" fn(HWND, UINT, *mut PROPSHEETPAGEW) -> UINT;

// prsht.h _PROPSHEETPAGEW (donor shell_link.rs:4906 — faithful prefix layout).
pub use crate::island::PROPSHEETPAGEW;

// winuser.h / prsht.h / winreg.h constants missing from the generated closure.
pub const WM_NOTIFY: UINT = 0x004E;
pub const PSN_APPLY: UINT = 0u32.wrapping_sub(202); // PSN_FIRST (0U-200U) - 2
pub const PSPCB_RELEASE: UINT = 1;
pub const PSM_CHANGED: UINT = 0x0468; // WM_USER+104
// DWLP_MSGRESULT=0; DWLP_DLGPROC=+sizeof(LRESULT); DWLP_USER=+sizeof(DLGPROC) => 2 pointers
pub const DWLP_USER: INT = (2 * core::mem::size_of::<LONG_PTR>()) as INT;
pub const HKEY_LOCAL_MACHINE: HKEY = 0x8000_0002u32 as i32 as isize as HKEY;

// devguid.h (sdk/include/psdk/devguid.h:23,26) — pinned SDK GUIDs.
pub const GUID_DEVCLASS_DISKDRIVE: GUID = GUID {
    Data1: 0x4D36E967, Data2: 0xE325, Data3: 0x11CE,
    Data4: [0xBF, 0xC1, 0x08, 0x00, 0x2B, 0xE1, 0x03, 0x18],
};
pub const GUID_DEVCLASS_CDROM: GUID = GUID {
    Data1: 0x4D36E965, Data2: 0xE325, Data3: 0x11CE,
    Data4: [0xBF, 0xC1, 0x08, 0x00, 0x2B, 0xE1, 0x03, 0x18],
};

// windef.h numeric macros.
#[inline]
pub const fn RGB(r: DWORD, g: DWORD, b: DWORD) -> COLORREF {
    (r & 0xff) | ((g & 0xff) << 8) | ((b & 0xff) << 16)
}
#[inline]
pub const fn MAKEINTRESOURCEW(id: UINT) -> LPCWSTR { (id & 0xffff) as usize as LPCWSTR }
#[inline]
pub const fn HIWORD(v: WPARAM) -> WORD { ((v >> 16) & 0xffff) as WORD }

// prsht.h: PropSheet_Changed(hDlg, hwnd) macro (donor shell_link.rs:5030).
#[inline]
pub unsafe fn PropSheet_Changed(hPropSheetDlg: HWND, hwndPage: HWND) -> LRESULT {
    SendMessage(hPropSheetDlg, PSM_CHANGED, hwndPage as WPARAM, 0)
}

// precomp.h:154-162 AddPropSheetPage (static inline; donor shell_link.rs:5004).
pub unsafe fn AddPropSheetPage(
    hPage: HPROPSHEETPAGE, pfnAddPage: LPFNSVADDPROPSHEETPAGE, lParam: LPARAM,
) -> HRESULT {
    if hPage.is_null() { return E_FAIL; }
    if let Some(add) = pfnAddPage {
        if add(hPage, lParam) != 0 { return S_OK; }
    }
    DestroyPropertySheetPage((hPage).cast());
    E_FAIL
}

// precomp.h:165-171 template<class T> PropSheetPageLifetimeCallback, instantiated
// for T = CDrvDefExt (donor pattern: shell_link.rs PropSheetPageLifetimeCallback_CShellLink).
pub unsafe extern "system" fn PropSheetPageLifetimeCallback_CDrvDefExt(
    _hWnd: HWND, uMsg: UINT, pPSP: *mut PROPSHEETPAGEW,
) -> UINT {
    if uMsg == PSPCB_RELEASE {
        CDrvDefExt_Release((*pPSP).lParam as *mut CDrvDefExt);
    }
    TRUE as UINT
}

// reactos/shellutils.h ShellObjectCreator<CDrvDefExt> — the class is Rust-owned here,
// so the template instantiation is local: create the object with one reference
// (the reference the C++ CComPtr would hold).
pub unsafe fn ShellObjectCreator(ppv: *mut *mut CDrvDefExt) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    let this = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<CDrvDefExt>()) as *mut CDrvDefExt;
    if this.is_null() { return E_OUTOFMEMORY; }
    core::ptr::write(this, CDrvDefExt {
        IShellExtInit_iface: IShellExtInit { lpVtbl: core::ptr::addr_of!(CDRVDEFEXT_ISHELLEXTINIT_VTBL) },
        IContextMenu_iface: IContextMenu { lpVtbl: core::ptr::addr_of!(CDRVDEFEXT_ICONTEXTMENU_VTBL) },
        IShellPropSheetExt_iface: IShellPropSheetExt { lpVtbl: core::ptr::addr_of!(CDRVDEFEXT_ISHELLPROPSHEETEXT_VTBL) },
        IObjectWithSite_iface: IObjectWithSite { lpVtbl: core::ptr::addr_of!(CDRVDEFEXT_IOBJECTWITHSITE_VTBL) },
        ref_count: core::sync::atomic::AtomicI32::new(1),
        m_wszDrive: [0; MAX_PATH],
        m_FreeSpacePerc: core::mem::zeroed(),
        m_Multiple: core::mem::zeroed(),
    });
    Shell32RustObjectLock();
    CDrvDefExt_ctor_body(this);
    *ppv = this;
    S_OK
}

// CDrvDefExt static member dialog procs (INT_PTR CALLBACK). The C++ statics recover
// the object from ppsp->lParam on WM_INITDIALOG and from DWLP_USER afterwards; the
// generated *_body workers take `this` as a parameter, so the recovery lives here.
pub unsafe extern "system" fn GeneralPageProc(
    hwndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM,
) -> INT_PTR {
    let this = if uMsg == WM_INITDIALOG {
        let ppsp = lParam as *const PROPSHEETPAGEW;
        if ppsp.is_null() { core::ptr::null_mut() } else { (*ppsp).lParam as *mut CDrvDefExt }
    } else {
        GetWindowLongPtr(hwndDlg, DWLP_USER) as *mut CDrvDefExt
    };
    CDrvDefExt_GeneralPageProc_body(this, hwndDlg, uMsg, wParam, lParam)
}
pub unsafe extern "system" fn ExtraPageProc(
    hwndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM,
) -> INT_PTR {
    // The worker recovers the object itself (ppsp->lParam / DWLP_USER per message).
    CDrvDefExt_ExtraPageProc_body(core::ptr::null_mut(), hwndDlg, uMsg, wParam, lParam)
}
pub unsafe extern "system" fn HardwarePageProc(
    hwndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM,
) -> INT_PTR {
    // The worker does not use `this` (C++ static body never touches the object).
    CDrvDefExt_HardwarePageProc_body(core::ptr::null_mut(), hwndDlg, uMsg, wParam, lParam)
}

// ── FFI imports (WINAPI => extern "system"; cdecl => extern "C") ──
extern "system" {
    // kernel32
    fn GetDriveTypeW(root: LPCWSTR) -> UINT;
    fn GetVolumeInformationW(root: LPCWSTR, name: LPWSTR, name_cch: DWORD, serial: LPDWORD,
                             max_len: LPDWORD, flags: LPDWORD, fs_name: LPWSTR, fs_cch: DWORD) -> BOOL;
    // user32
    #[link_name = "SendMessageW"] // UNICODE build: SendMessage => SendMessageW
    fn SendMessage(hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> LRESULT;
    // gdi32
    fn GetStockObject(object: INT) -> HGDIOBJ;
    fn DeleteObject(object: HGDIOBJ) -> BOOL;
    fn MoveToEx(dc: HDC, x: INT, y: INT, point: *mut POINT) -> BOOL;
    fn LineTo(dc: HDC, x: INT, y: INT) -> BOOL;
    fn SetPixel(dc: HDC, x: INT, y: INT, color: COLORREF) -> COLORREF;
    fn Ellipse(dc: HDC, left: INT, top: INT, right: INT, bottom: INT) -> BOOL;
    fn Pie(dc: HDC, left: INT, top: INT, right: INT, bottom: INT,
           xr1: INT, yr1: INT, xr2: INT, yr2: INT) -> BOOL;
    // comctl32
    // ole32
    fn ReleaseStgMedium(medium: *mut STGMEDIUM);
    // shell32 public exports
    fn DragQueryFileW(drop: HDROP, index: UINT, path: LPWSTR, cch: UINT) -> UINT;
    fn SHSimpleIDListFromPathW(path: LPCWSTR) -> LPITEMIDLIST;
    // shlwapi
    fn PathGetArgsW(path: LPCWSTR) -> LPWSTR;
    fn StrFormatByteSizeW(qdw: LONGLONG, buffer: LPWSTR, cch: UINT) -> LPWSTR;
    // devmgr.dll (EXTERN_C HWND WINAPI, drvdefext.cpp:42-46)
    fn DeviceCreateHardwarePageEx(parent: HWND, guids: *mut GUID, num_guids: UINT,
                                  display_mode: DWORD) -> HWND;
}

// winuser.h: on win32 the *Ptr forms are macro aliases of the LONG forms —
// bind to the real 32-bit exports there (shell_link.rs precedent).
#[cfg(target_pointer_width = "64")]
extern "system" {
    #[link_name = "GetWindowLongPtrW"]
    fn GetWindowLongPtr(hwnd: HWND, index: INT) -> LONG_PTR;
    #[link_name = "SetWindowLongPtrW"]
    fn SetWindowLongPtr(hwnd: HWND, index: INT, value: LONG_PTR) -> LONG_PTR;
}
#[cfg(target_pointer_width = "32")]
extern "system" {
    #[link_name = "GetWindowLongW"]
    fn GetWindowLongPtr(hwnd: HWND, index: INT) -> LONG_PTR;
    #[link_name = "SetWindowLongW"]
    fn SetWindowLongPtr(hwnd: HWND, index: INT, value: LONG_PTR) -> LONG_PTR;
}

// -- Rust bridges to retained C++ companions (EXTERN_C ... WINAPI shims C++-side) --
extern "system" {
    // EXISTING bridge (utils.cpp:2267, created by the CFSFolder landing):
    #[link_name = "SHGetNameAndFlagsW_RustBridge"]
    fn SHGetNameAndFlagsW(pidl: LPCITEMIDLIST, flags: DWORD, text: LPWSTR, cch: UINT,
                          attributes: *mut DWORD) -> HRESULT;
    // NEW — must be created C++-side (see integration TODO):
    // C++: UINT SH_FormatByteSize(LONGLONG cbSize, LPWSTR pwszResult, UINT cchResultMax)
    //      (dialogs/filedefext.cpp:274, C++ linkage)
    #[link_name = "SH_FormatByteSize_RustBridge"]
    fn SH_FormatByteSize(cbSize: LONGLONG, pwszResult: LPWSTR, cchResultMax: UINT) -> UINT;
    // C++: BOOL IsDriveFloppyW(LPCWSTR pszDriveRoot) (dialogs/drvdefext.cpp, C++ linkage)
    #[link_name = "IsDriveFloppyW_RustBridge"]
    fn IsDriveFloppyW(pszDriveRoot: LPCWSTR) -> BOOL;
    // C++: static HRESULT CDrivesFolder::SetDriveLabel(HWND, PCWSTR, PCWSTR)
    //      (folders/CDrivesFolder.cpp:1193; also Rust-owned as
    //      drives_folder.rs::CDrivesFolder_SetDriveLabel_body)
    #[link_name = "CDrivesFolder_SetDriveLabel_RustBridge"]
    fn SetDriveLabel(hwndOwner: HWND, DrivePath: PCWSTR, Label: PCWSTR) -> HRESULT;
    // C++: static INT_PTR CALLBACK ChkDskDlg(HWND, UINT, WPARAM, LPARAM)
    //      (dialogs/drvdefext.cpp:704 region, file-static — shim must forward)
    #[link_name = "ChkDskDlg_RustBridge"]
    fn ChkDskDlg(hwndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR;
}

extern "C" {
    // msvcrt (cdecl). Declaring/calling a C-variadic is stable; only defining needs nightly.
    fn cos(x: f64) -> f64;
    fn sin(x: f64) -> f64;
    fn acos(x: f64) -> f64;
    fn swprintf(buffer: *mut WCHAR, format: *const WCHAR, ...) -> INT;
    // wine/shell32_main.h:250-251 (extern "C" block; defined in propsheet.cpp —
    // donor shell_link.rs:5288)
}

// Both StringCbPrintfW call sites in this file format a registry-sourced command
// template with the drive LETTER (`%c`, WCHAR argument) — the original generated
// helper only handled `%s`/LPCWSTR and would have copied "%c" literally.
unsafe fn cpp_com_string_cb_printf_w(
    dest: LPWSTR, bytes: usize, format: LPCWSTR, args: &[WCHAR],
) -> HRESULT {
    let capacity = bytes / core::mem::size_of::<WCHAR>();
    if dest.is_null() || format.is_null() || capacity == 0 { return E_FAIL; }
    let (mut out, mut input, mut arg_index) = (0usize, 0usize, 0usize);
    while *format.add(input) != 0 {
        let unit = *format.add(input);
        if unit == b'%' as WCHAR && *format.add(input + 1) == b'c' as WCHAR {
            if arg_index >= args.len() { return E_FAIL; }
            let arg = args[arg_index]; arg_index += 1; input += 2;
            if out + 1 >= capacity { *dest.add(capacity - 1) = 0; return 0x8007007Au32 as HRESULT; }
            *dest.add(out) = arg; out += 1;
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
    static CLSID_ShellDrvDefExt: GUID;
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
pub static CPP_COM_WSTR_8339E3D14E51: [WCHAR; 73] = [0x0053, 0x004F, 0x0046, 0x0054, 0x0057, 0x0041, 0x0052, 0x0045, 0x005C, 0x004D, 0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057, 0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x005C, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069, 0x006F, 0x006E, 0x005C, 0x0045, 0x0078, 0x0070, 0x006C, 0x006F, 0x0072, 0x0065, 0x0072, 0x005C, 0x004D, 0x0079, 0x0043, 0x006F, 0x006D, 0x0070, 0x0075, 0x0074, 0x0065, 0x0072, 0x005C, 0x0042, 0x0061, 0x0063, 0x006B, 0x0075, 0x0070, 0x0050, 0x0061, 0x0074, 0x0068, 0x0000];
pub static CPP_COM_WSTR_A7185C6D7A2F: [WCHAR; 73] = [0x0053, 0x004F, 0x0046, 0x0054, 0x0057, 0x0041, 0x0052, 0x0045, 0x005C, 0x004D, 0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057, 0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x005C, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069, 0x006F, 0x006E, 0x005C, 0x0045, 0x0078, 0x0070, 0x006C, 0x006F, 0x0072, 0x0065, 0x0072, 0x005C, 0x004D, 0x0079, 0x0043, 0x006F, 0x006D, 0x0070, 0x0075, 0x0074, 0x0065, 0x0072, 0x005C, 0x0044, 0x0065, 0x0066, 0x0072, 0x0061, 0x0067, 0x0050, 0x0061, 0x0074, 0x0068, 0x0000];
pub static CPP_COM_WSTR_FCB7C6756111: [WCHAR; 74] = [0x0053, 0x004F, 0x0046, 0x0054, 0x0057, 0x0041, 0x0052, 0x0045, 0x005C, 0x004D, 0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057, 0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x005C, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069, 0x006F, 0x006E, 0x005C, 0x0045, 0x0078, 0x0070, 0x006C, 0x006F, 0x0072, 0x0065, 0x0072, 0x005C, 0x004D, 0x0079, 0x0043, 0x006F, 0x006D, 0x0070, 0x0075, 0x0074, 0x0065, 0x0072, 0x005C, 0x0043, 0x006C, 0x0065, 0x0061, 0x006E, 0x0075, 0x0070, 0x0050, 0x0061, 0x0074, 0x0068, 0x0000];

// --local-raii CDrop: hand-verified lowering (spliced verbatim; workers received this API as a sealed contract)
// Hand-verified Rust lowering of the file-local C++ RAII struct `CDrop`
// (dll/win32/shell32/dialogs/drvdefext.cpp:704). Consumed by
// `rustconv translate --cpp-com --local-raii CDrop=<this file>`: spliced verbatim
// into the assembled module, and its API sealed into each affected worker prompt
// so the model maps C++ usage onto this fixed surface instead of inventing the
// drop semantics.
//
// C++ semantics preserved exactly:
//   ctor      — IDataObject::GetData(CF_HDROP, TYMED_HGLOBAL); hr = result;
//               hDrop = stgm.hGlobal on success, else NULL. stgm is only
//               meaningful when hDrop != NULL.
//   dtor      — if (hDrop) ReleaseStgMedium(&stgm)         → impl Drop
//   GetCount  — DragQueryFileW(hDrop, 0xFFFFFFFF, NULL, 0)  (count query)
struct CDrop {
    hr: HRESULT,
    stgm: STGMEDIUM,
    hDrop: HDROP,
}

impl CDrop {
    unsafe fn new(pDO: *mut IDataObject) -> CDrop {
        let mut this = CDrop {
            hr: 0,
            stgm: core::mem::zeroed(),
            hDrop: core::ptr::null_mut(),
        };
        let mut format = FORMATETC {
            cfFormat: CF_HDROP as CLIPFORMAT,
            ptd: core::ptr::null_mut(),
            dwAspect: DVASPECT_CONTENT,
            lindex: -1,
            tymed: TYMED_HGLOBAL,
        };
        this.hr = ((*(*pDO).lpVtbl).GetData)(pDO, &mut format, &mut this.stgm);
        if this.hr >= 0 {
            this.hDrop = this.stgm.__bindgen_anon_1.hGlobal as HDROP;
        }
        this
    }
    pub unsafe fn GetCount(&mut self) -> UINT {
        DragQueryFileW(self.hDrop, 0xFFFFFFFF, core::ptr::null_mut(), 0)
    }
}

impl Drop for CDrop {
    fn drop(&mut self) {
        unsafe {
            if !self.hDrop.is_null() {
                ReleaseStgMedium(&mut self.stgm);
            }
        }
    }
}

use core::ffi::c_void;
// The crate's own functions: the definition is the declaration.
use crate::propsheet::SH_CreatePropertySheetPage;
use crate::propsheet::SH_CreatePropertySheetPageEx;
use crate::shell32::shlexec::ShellExecuteW;
// One declaration per external symbol: island.rs.
use crate::island::CloseHandle;
use crate::island::CreateFileW;
use crate::island::CreatePen;
use crate::island::CreateSolidBrush;
use crate::island::DestroyPropertySheetPage;
use crate::island::DialogBoxParamW;
use crate::island::FillRect;
use crate::island::GetDiskFreeSpaceExW;
use crate::island::GetDlgItem;
use crate::island::GetDlgItemTextW;
use crate::island::GetParent;
use crate::island::GetWindowTextLengthW;
use crate::island::GetWindowTextW;
use crate::island::LoadStringW;
use crate::island::RegGetValueW;
use crate::island::SelectObject;
use crate::island::SendDlgItemMessageW;
use crate::island::SetDlgItemTextW;
use crate::island::ShowWindow;
use crate::shellcom::*;
use crate::types::*;
// A no-op debug macro must stay a macro: an out-of-line no-op `fn` is an optimization barrier
// that can uncloak a latent ABI mismatch. `#[macro_export]` puts this at the crate root, so an
// explicit import reaches it from anywhere — `macro_rules!` TEXTUAL scoping does not, because
// this module is declared before `win32` in lib.rs, which is why a `fn` shadow grew here.
use crate::UNREFERENCED_PARAMETER;

pub unsafe fn CDrvDefExt_PaintStaticControls_body(this: *mut CDrvDefExt, hwndDlg: HWND, pDrawItem: LPDRAWITEMSTRUCT) -> () {
    if (*pDrawItem).CtlID == 14013 {
        let hBrush = CreateSolidBrush(RGB(0, 0, 255));
        if !hBrush.is_null() {
            FillRect((*pDrawItem).hDC, &(*pDrawItem).rcItem, hBrush);
            DeleteObject(hBrush as HGDIOBJ);
        }
    } else if (*pDrawItem).CtlID == 14014 {
        let hBrush = CreateSolidBrush(RGB(255, 0, 255));
        if !hBrush.is_null() {
            FillRect((*pDrawItem).hDC, &(*pDrawItem).rcItem, hBrush);
            DeleteObject(hBrush as HGDIOBJ);
        }
    } else if (*pDrawItem).CtlID == 14015 {
        let hBlueBrush = CreateSolidBrush(RGB(0, 0, 255));
        let hMagBrush = CreateSolidBrush(RGB(255, 0, 255));
        let hBlackPen = GetStockObject(BLACK_PEN as INT) as HPEN;
        let hDarkBluePen = CreatePen(PS_SOLID as INT, 1, RGB(0, 0, 128));
        let hDarkMagPen = CreatePen(PS_SOLID as INT, 1, RGB(128, 0, 128));
        let hOldPen = SelectObject((*pDrawItem).hDC, (hDarkMagPen).cast()) as HPEN;

        let xCenter = ((*pDrawItem).rcItem.left + (*pDrawItem).rcItem.right) / 2;
        let yCenter = ((*pDrawItem).rcItem.top + (*pDrawItem).rcItem.bottom - 10) / 2;
        let cx = (*pDrawItem).rcItem.right - (*pDrawItem).rcItem.left;
        let cy = (*pDrawItem).rcItem.bottom - (*pDrawItem).rcItem.top - 10;
        
        let xRadial = xCenter + (cos((M_PI as f64 + (*this).m_FreeSpacePerc as f64 / 100 as f64 * M_PI as f64 * 2 as f64) as f64) * cx as f64 / 2 as f64) as INT;
        let yRadial = yCenter - (sin((M_PI as f64 + (*this).m_FreeSpacePerc as f64 / 100 as f64 * M_PI as f64 * 2 as f64) as f64) * cy as f64 / 2 as f64) as INT;

        for x in (*pDrawItem).rcItem.left..(*pDrawItem).rcItem.right {
            let cos_val = (x - xCenter) as f64 * 2 as f64 / cx as f64;
            let y = yCenter + (sin(acos(cos_val)) * cy as f64 / 2 as f64) as INT - 1;
            let mut hCenterPen: HPEN = 0 as HPEN;

            if (*this).m_FreeSpacePerc < 50 && x == xRadial {
                SelectObject((*pDrawItem).hDC, (hDarkBluePen).cast());
            }

            if x == (*pDrawItem).rcItem.left {
                hCenterPen = SelectObject((*pDrawItem).hDC, (hBlackPen).cast()) as HPEN;
            } else if x == (*pDrawItem).rcItem.right - 1 {
                SelectObject((*pDrawItem).hDC, (hBlackPen).cast());
            }

            MoveToEx(((*pDrawItem).hDC).cast(), x, y, core::ptr::null_mut());
            LineTo(((*pDrawItem).hDC).cast(), x, y + 10);
            SetPixel(((*pDrawItem).hDC).cast(), x, y + 10, RGB(0, 0, 0));

            if x == (*pDrawItem).rcItem.left {
                SelectObject((*pDrawItem).hDC, (hCenterPen).cast());
            }
        }

        SelectObject((*pDrawItem).hDC, (hBlackPen).cast());

        let hbrOld: HBRUSH;
        if (*this).m_FreeSpacePerc > 50 {
            hbrOld = SelectObject((*pDrawItem).hDC, (hMagBrush).cast()) as HBRUSH;
            Ellipse(((*pDrawItem).hDC).cast(), (*pDrawItem).rcItem.left, (*pDrawItem).rcItem.top, (*pDrawItem).rcItem.right, (*pDrawItem).rcItem.bottom - 10);
            SelectObject((*pDrawItem).hDC, (hBlueBrush).cast());

            if (*this).m_FreeSpacePerc < 100 {
                Pie(((*pDrawItem).hDC).cast(), (*pDrawItem).rcItem.left, (*pDrawItem).rcItem.top, (*pDrawItem).rcItem.right,
                    (*pDrawItem).rcItem.bottom - 10, xRadial, yRadial, (*pDrawItem).rcItem.left, yCenter);
            }
        } else {
            hbrOld = SelectObject((*pDrawItem).hDC, (hBlueBrush).cast()) as HBRUSH;
            Ellipse(((*pDrawItem).hDC).cast(), (*pDrawItem).rcItem.left, (*pDrawItem).rcItem.top, (*pDrawItem).rcItem.right, (*pDrawItem).rcItem.bottom - 10);
            SelectObject((*pDrawItem).hDC, (hMagBrush).cast());

            if (*this).m_FreeSpacePerc > 0 {
                Pie(((*pDrawItem).hDC).cast(), (*pDrawItem).rcItem.left, (*pDrawItem).rcItem.top, (*pDrawItem).rcItem.right,
                    (*pDrawItem).rcItem.bottom - 10, (*pDrawItem).rcItem.left, yCenter, xRadial, yRadial);
            }
        }

        SelectObject((*pDrawItem).hDC, (hbrOld).cast());
        SelectObject((*pDrawItem).hDC, (hOldPen).cast());

        DeleteObject((hBlueBrush).cast());
        DeleteObject((hMagBrush).cast());
        DeleteObject((hDarkBluePen).cast());
        DeleteObject((hDarkMagPen).cast());
    }
}

pub unsafe fn CDrvDefExt_InitGeneralPage_body(this: *mut CDrvDefExt, hwndDlg: HWND) -> () {
    let mut wszVolumeName: [u16; (MAX_PATH + 1) as usize] =
        [0 as u16; (MAX_PATH + 1) as usize];
    let mut wszFileSystem: [u16; (MAX_PATH + 1) as usize] =
        [0 as u16; (MAX_PATH + 1) as usize];
    let mut wszBuf: [u16; 128 as usize] = [0u16; 128 as usize];
    let mut bFloppy: i32 = FALSE;
    let mut bHasFS: i32 = FALSE;

    let ret = GetVolumeInformationW(
        (*this).m_wszDrive.as_ptr(),
        wszVolumeName.as_mut_ptr(),
        wszVolumeName.len() as u32,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        wszFileSystem.as_mut_ptr(),
        wszFileSystem.len() as u32,
    );

    if ret != 0 {
        SetDlgItemTextW(hwndDlg, 14000, wszVolumeName.as_ptr());
        SetDlgItemTextW(hwndDlg, 14002, wszFileSystem.as_ptr());
        bHasFS = (wszFileSystem[0] != UNICODE_NULL) as i32;
    } else {
        LoadStringW(
            shell32_hInstance,
            IDS_FS_UNKNOWN,
            wszFileSystem.as_mut_ptr(),
            wszFileSystem.len() as INT,
        );
        SetDlgItemTextW(hwndDlg, 14002, wszFileSystem.as_ptr());
    }

    let DriveType = GetDriveTypeW((*this).m_wszDrive.as_ptr());
    let mut TypeStrId: u32 = 0;
    let IconId: u32;

    match DriveType {
        DRIVE_REMOVABLE => {
            bFloppy = IsDriveFloppyW((*this).m_wszDrive.as_ptr());
            IconId = if bFloppy != 0 {
                IDI_SHELL_3_14_FLOPPY
            } else {
                IDI_SHELL_REMOVEABLE
            };
            TypeStrId = if bFloppy != 0 {
                IDS_DRIVE_FLOPPY
            } else {
                IDS_DRIVE_REMOVABLE
            };
        }
        DRIVE_CDROM => {
            IconId = IDI_SHELL_CDROM;
            TypeStrId = IDS_DRIVE_CDROM;
        }
        DRIVE_REMOTE => {
            IconId = IDI_SHELL_NETDRIVE;
            TypeStrId = IDS_DRIVE_NETWORK;
        }
        DRIVE_RAMDISK => {
            IconId = IDI_SHELL_RAMDISK;
        }
        _ => {
            IconId = IDI_SHELL_DRIVE;
            TypeStrId = IDS_DRIVE_FIXED;
        }
    }

    let mut bCanSetLabel = bHasFS;
    if DriveType == DRIVE_CDROM || DriveType == DRIVE_REMOTE || bFloppy != 0 {
        bCanSetLabel = ((bFloppy != 0) && (bHasFS != 0)) as BOOL;
        ShowWindow(
            GetDlgItem(hwndDlg, 14011),
            FALSE,
        );
        ShowWindow(
            GetDlgItem(hwndDlg, 14012),
            FALSE,
        );
    }

    // C++: `!bCanSetLabel` (logical not) — a Rust `!` on i32 is bitwise and inverts the flag.
    SendMessage(
        GetDlgItem(hwndDlg, 14000),
        EM_SETREADONLY,
        (bCanSetLabel == FALSE) as WPARAM,
        0,
    );

    let hIcon = LoadImage(
        shell32_hInstance,
        MAKEINTRESOURCE(IconId),
        IMAGE_ICON,
        32,
        32,
        LR_SHARED,
    );

    if !hIcon.is_null() {
        SendDlgItemMessageW(
            hwndDlg,
            14016,
            STM_SETICON,
            hIcon as WPARAM,
            0,
        );
    }

    if TypeStrId != 0
        && LoadStringW(
            shell32_hInstance,
            TypeStrId,
            wszBuf.as_mut_ptr(),
            wszBuf.len() as INT,
        ) != 0
    {
        SetDlgItemTextW(hwndDlg, 14001, wszBuf.as_ptr());
    }

    let mut FreeBytesAvailable: ULARGE_INTEGER = core::mem::zeroed();
    let mut TotalNumberOfBytes: ULARGE_INTEGER = core::mem::zeroed();

    if GetDiskFreeSpaceExW(
        (*this).m_wszDrive.as_ptr(),
        (&raw mut FreeBytesAvailable).cast(),
        (&raw mut TotalNumberOfBytes).cast(),
        core::ptr::null_mut(),
    ) != 0
    {
        (*(&raw mut (*this).m_FreeSpacePerc)) =
            ((FreeBytesAvailable.QuadPart * 100u64)
                / TotalNumberOfBytes.QuadPart) as u32;

        if SH_FormatByteSize(
            (TotalNumberOfBytes.QuadPart - FreeBytesAvailable.QuadPart) as LONGLONG,
            wszBuf.as_mut_ptr(),
            wszBuf.len() as u32,
        ) != 0
        {
            SetDlgItemTextW(hwndDlg, 14003, wszBuf.as_ptr());
        }

        if !StrFormatByteSizeW(
            (TotalNumberOfBytes.QuadPart - FreeBytesAvailable.QuadPart) as LONGLONG,
            wszBuf.as_mut_ptr(),
            wszBuf.len() as u32,
        ).is_null()
        {
            SetDlgItemTextW(hwndDlg, 14004, wszBuf.as_ptr());
        }

        if SH_FormatByteSize(
            FreeBytesAvailable.QuadPart as LONGLONG,
            wszBuf.as_mut_ptr(),
            wszBuf.len() as u32,
        ) != 0
        {
            SetDlgItemTextW(hwndDlg, 14005, wszBuf.as_ptr());
        }

        if !StrFormatByteSizeW(
            FreeBytesAvailable.QuadPart as LONGLONG,
            wszBuf.as_mut_ptr(),
            wszBuf.len() as u32,
        ).is_null()
        {
            SetDlgItemTextW(hwndDlg, 14006, wszBuf.as_ptr());
        }

        if SH_FormatByteSize(
            TotalNumberOfBytes.QuadPart as LONGLONG,
            wszBuf.as_mut_ptr(),
            wszBuf.len() as u32,
        ) != 0
        {
            SetDlgItemTextW(hwndDlg, 14007, wszBuf.as_ptr());
        }

        if !StrFormatByteSizeW(
            TotalNumberOfBytes.QuadPart as LONGLONG,
            wszBuf.as_mut_ptr(),
            wszBuf.len() as u32,
        ).is_null()
        {
            SetDlgItemTextW(hwndDlg, 14008, wszBuf.as_ptr());
        }
    } else {
        (*(&raw mut (*this).m_FreeSpacePerc)) = 0;

        if SH_FormatByteSize(
            0,
            wszBuf.as_mut_ptr(),
            wszBuf.len() as u32,
        ) != 0
        {
            SetDlgItemTextW(hwndDlg, 14003, wszBuf.as_ptr());
            SetDlgItemTextW(hwndDlg, 14005, wszBuf.as_ptr());
            SetDlgItemTextW(hwndDlg, 14007, wszBuf.as_ptr());
        }

        if !StrFormatByteSizeW(
            0,
            wszBuf.as_mut_ptr(),
            wszBuf.len() as u32,
        ).is_null()
        {
            SetDlgItemTextW(hwndDlg, 14004, wszBuf.as_ptr());
            SetDlgItemTextW(hwndDlg, 14006, wszBuf.as_ptr());
            SetDlgItemTextW(hwndDlg, 14008, wszBuf.as_ptr());
        }
    }

    let mut wszFormat: [u16; 50 as usize] = [0u16; 50 as usize];
    GetDlgItemTextW(
        hwndDlg,
        14009,
        wszFormat.as_mut_ptr(),
        wszFormat.len() as INT,
    );

    swprintf(
        wszBuf.as_mut_ptr(),
        wszFormat.as_ptr(),
        (*this).m_wszDrive[0 as usize] as INT, // wchar_t promotes to int through `...`
    );

    SetDlgItemTextW(hwndDlg, 14009, wszBuf.as_ptr());

    ShowWindow(
        GetDlgItem(hwndDlg, 14010),
        (DriveType == DRIVE_FIXED) as i32,
    );
}

pub unsafe fn CDrvDefExt_GeneralPageProc_body(this: *mut CDrvDefExt, hwndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    match uMsg {
        WM_INITDIALOG => {
            let ppsp = lParam as *mut PROPSHEETPAGEW;
            if !ppsp.is_null() {
                SetWindowLongPtr(hwndDlg, DWLP_USER, this as LONG_PTR);
                CDrvDefExt_InitGeneralPage_body(this, hwndDlg);
                return TRUE as INT_PTR;
            }
        }
        WM_DRAWITEM => {
            let pDrawItem = lParam as *mut DRAWITEMSTRUCT;
            if (*pDrawItem).CtlID >= 14013 && (*pDrawItem).CtlID <= 14015 {
                GetWindowLongPtr(hwndDlg, DWLP_USER);
                CDrvDefExt_PaintStaticControls_body(this, hwndDlg, pDrawItem);
                return TRUE as INT_PTR;
            }
        }
        WM_PAINT => {}
        WM_COMMAND => {
            if LOWORD!(wParam) == 14010 {
                GetWindowLongPtr(hwndDlg, DWLP_USER);
                let mut wszBuf: [u16; 256] = [0u16; 256];
                let mut cbBuf: DWORD = size_of_val(&wszBuf) as DWORD;
                let __rustconv_tmp = CPP_COM_WSTR_FCB7C6756111.as_ptr();
                if RegGetValueW(
                    HKEY_LOCAL_MACHINE,
                    __rustconv_tmp,
                    core::ptr::null::<u16>(),
                    RRF_RT_REG_SZ,
                    core::ptr::null_mut::<u32>(),
                    wszBuf.as_mut_ptr() as *mut core::ffi::c_void,
                    &mut cbBuf as *mut DWORD,
                ) == ERROR_SUCCESS
                {
                    let mut wszCmd: [u16; MAX_PATH] = [0u16; MAX_PATH];
                    StringCbPrintfW!(
                        wszCmd.as_mut_ptr(),
                        size_of_val(&wszCmd) as size_t,
                        wszBuf.as_ptr(),
                        (*this).m_wszDrive[0],
                    );
                    let wszArgs_raw = PathGetArgsW(wszCmd.as_mut_ptr() as *mut u16);
                    let wszArgs = if !wszArgs_raw.is_null()
                        && *wszArgs_raw != 0
                        && wszArgs_raw != wszCmd.as_mut_ptr()
                    {
                        *wszArgs_raw.wrapping_offset(-1) = UNICODE_NULL as u16;
                        wszArgs_raw
                    } else {
                        core::ptr::null_mut()
                    };
                    if ShellExecuteW(
                        hwndDlg,
                        core::ptr::null::<u16>(),
                        wszCmd.as_ptr(),
                        wszArgs,
                        core::ptr::null::<u16>(),
                        SW_SHOW as INT,
                    ) as usize <= 32
                    {
                        // ERR!("Failed to create cleanup process %ls\n", wszCmd);
                    }
                }
            } else if LOWORD!(wParam) == 14000 {
                if HIWORD(wParam) as DWORD == EN_CHANGE {
                    PropSheet_Changed(GetParent(hwndDlg), hwndDlg);
                }
            }
        }
        WM_NOTIFY => {
            let pnmh = lParam as *mut NMHDR;
            if (*pnmh).hwndFrom == GetParent(hwndDlg) {
                let lppsn = lParam as *mut PSHNOTIFY;
                if (*lppsn).hdr.code == PSN_APPLY {
                    GetWindowLongPtr(hwndDlg, DWLP_USER);
                    let mut hr: HRESULT = E_FAIL;
                    let hLabel = GetDlgItem(hwndDlg, 14000);
                    let mut wszBuf: [u16; 256] = [0u16; 256];
                    wszBuf[0] = UNICODE_NULL;
                    if GetWindowTextW(hLabel, wszBuf.as_mut_ptr(), 256) != 0
                        || GetWindowTextLengthW(hLabel) == 0
                    {
                        hr = SetDriveLabel(
                            hwndDlg,
                            (*this).m_wszDrive.as_ptr(),
                            wszBuf.as_ptr(),
                        );
                    }
                    let result = if hr < 0 {
                        PSNRET_INVALID
                    } else {
                        PSNRET_NOERROR
                    };
                    SetWindowLongPtr(hwndDlg, DWLP_MSGRESULT as INT, result as LONG_PTR);
                    return TRUE as INT_PTR;
                }
            }
        }
        _ => {}
    }
    FALSE as INT_PTR
}

pub unsafe fn CDrvDefExt_ExtraPageProc_body(this: *mut CDrvDefExt, hwndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    match uMsg {
        WM_INITDIALOG => {
            let ppsp = lParam as *const PROPSHEETPAGEW;
            SetWindowLongPtr(hwndDlg, DWLP_USER, ((*ppsp).lParam as isize) as _);
            return TRUE as INT_PTR;
        }
        WM_COMMAND => {
            let mut wszBuf: [u16; MAX_PATH] = [0; MAX_PATH];
            let mut cbBuf: u32 = (wszBuf.len() * core::mem::size_of::<u16>()) as u32;
            let pDrvDefExt =
                GetWindowLongPtr(hwndDlg, DWLP_USER) as *mut CDrvDefExt;

            match LOWORD!(wParam) {
                14000 => {
                    let lpDrive =
                        &((*pDrvDefExt).m_wszDrive) as *const _ as LPARAM;
                    DialogBoxParamW(
                        shell32_hInstance,
                        MAKEINTRESOURCEW(IDD_CHECK_DISK),
                        hwndDlg,
                        Some(ChkDskDlg),
                        lpDrive,
                    );
                }
                14001 => {
                    if RegGetValueW(
                        HKEY_LOCAL_MACHINE,
                        CPP_COM_WSTR_A7185C6D7A2F.as_ptr(),
                        core::ptr::null(),
                        RRF_RT_REG_SZ,
                        core::ptr::null_mut(),
                        wszBuf.as_mut_ptr() as *mut core::ffi::c_void,
                        &mut cbBuf,
                    ) == ERROR_SUCCESS
                    {
                        let mut wszCmd: [u16; MAX_PATH] =
                            [0; MAX_PATH];
                        StringCbPrintfW!(
                            wszCmd.as_mut_ptr(),
                            core::mem::size_of::<[u16; MAX_PATH]>(),
                            wszBuf.as_ptr(),
                            (*pDrvDefExt).m_wszDrive[0 as usize],
                        );
                        if ShellExecuteW(
                            hwndDlg,
                            core::ptr::null(),
                            wszCmd.as_ptr(),
                            core::ptr::null(),
                            core::ptr::null(),
                            SW_SHOW as INT,
                        ) as usize <= 32
                        {
                            unsafe {
                                ERR!(
                                    "Failed to create defrag process {}\n",
                                    wszCmd.as_ptr()
                                )
                            };
                        }
                    }
                }
                14002 => {
                    if RegGetValueW(
                        HKEY_LOCAL_MACHINE,
                        CPP_COM_WSTR_8339E3D14E51.as_ptr(),
                        core::ptr::null(),
                        RRF_RT_REG_SZ,
                        core::ptr::null_mut(),
                        wszBuf.as_mut_ptr() as *mut core::ffi::c_void,
                        &mut cbBuf,
                    ) == ERROR_SUCCESS
                    {
                        if ShellExecuteW(
                            hwndDlg,
                            core::ptr::null(),
                            wszBuf.as_ptr(),
                            core::ptr::null(),
                            core::ptr::null(),
                            SW_SHOW as INT,
                        ) as usize <= 32
                        {
                            unsafe {
                                ERR!(
                                    "Failed to create backup process {}\n",
                                    wszBuf.as_ptr()
                                )
                            };
                        }
                    }
                }
                _ => {}
            }
        }
        _ => {}
    }
    FALSE as INT_PTR
}

pub unsafe fn CDrvDefExt_HardwarePageProc_body(this: *mut CDrvDefExt, hwndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    UNREFERENCED_PARAMETER!(lParam);
    UNREFERENCED_PARAMETER!(wParam);

    match uMsg {
        WM_INITDIALOG => {
            let mut guids: [GUID; 2] = unsafe { core::mem::zeroed() };
            guids[0] = GUID_DEVCLASS_DISKDRIVE;
            guids[1] = GUID_DEVCLASS_CDROM;
            DeviceCreateHardwarePageEx(hwndDlg, guids.as_mut_ptr(), 2, HWPD_STANDARDLIST);
        }
        _ => {}
    }

    FALSE as INT_PTR
}

pub unsafe fn CDrvDefExt_ctor_body(this: *mut CDrvDefExt) -> () {
    (*this).m_wszDrive[0] = 0x0000u16;
}

pub unsafe fn CDrvDefExt_dtor_body(this: *mut CDrvDefExt) -> () {}

pub unsafe fn IsValidDrivePath_body(Path: PCWSTR) -> bool {
    GetDriveTypeW(Path) > DRIVE_NO_ROOT_DIR
}

pub unsafe fn CDrvDefExt_Initialize_body(this: *mut CDrvDefExt, pidlFolder: PCIDLIST_ABSOLUTE, pDataObj: *mut IDataObject, hkeyProgID: HKEY) -> HRESULT {
    if pDataObj.is_null() {
        return E_FAIL;
    }

    let mut drop = CDrop::new(pDataObj);
    let hr = drop.hr;
    if ((hr) < 0) {
        return hr;
    }

    if DragQueryFileW(
        drop.hDrop,
        0,
        (*this).m_wszDrive.as_mut_ptr(),
        (*this).m_wszDrive.len() as u32,
    ) == 0
    {
        return E_FAIL;
    }

    if drop.GetCount() > 1 {
        CDrvDefExt_cpp_com_assign_m_Multiple(this, pDataObj);
    }

    S_OK
}

pub unsafe fn CDrvDefExt_QueryContextMenu_body(this: *mut CDrvDefExt, hmenu: HMENU, indexMenu: UINT, idCmdFirst: UINT, idCmdLast: UINT, uFlags: UINT) -> HRESULT { E_NOTIMPL }

pub unsafe fn CDrvDefExt_InvokeCommand_body(this: *mut CDrvDefExt, lpici: LPCMINVOKECOMMANDINFO) -> HRESULT { E_NOTIMPL }

pub unsafe fn CDrvDefExt_GetCommandString_body(this: *mut CDrvDefExt, idCmd: UINT_PTR, uType: UINT, pwReserved: *mut UINT, pszName: LPSTR, cchMax: UINT) -> HRESULT { E_NOTIMPL }

pub unsafe fn CDrvDefExt_AddMainPage_body(this: *mut CDrvDefExt, pfnAddPage: LPFNADDPROPSHEETPAGE, lParam: LPARAM) -> HRESULT {
    let mut szTitle: [u16; MAX_PATH] = [0; MAX_PATH];
    let mut pszTitle: *const u16 = core::ptr::null();
    if !(*this).m_Multiple.is_null() {
        let pidl = CComHeapPtr::<ITEMIDLIST_ABSOLUTE>::new(
            SHSimpleIDListFromPathW(core::ptr::addr_of!((*this).m_wszDrive) as *const u16),
        );
        if SHGetNameAndFlagsW(
            pidl.as_raw(),
            SHGDN_INFOLDER,
            szTitle.as_mut_ptr(),
            szTitle.len() as UINT,
            core::ptr::null_mut(),
        ) >= 0
        {
            pszTitle = szTitle.as_ptr();
        }
    }
    let hPage = SH_CreatePropertySheetPageEx(
        IDD_DRIVE_PROPERTIES as WORD,
        Some(GeneralPageProc),
        this as LPARAM,
        pszTitle,
        Some(PropSheetPageLifetimeCallback_CDrvDefExt),
    );
    let hr: HRESULT = AddPropSheetPage(hPage, pfnAddPage, lParam);
    if ((hr) < 0) {
        return hr;
    }
    CDrvDefExt_AddRef(this); // For PropSheetPageLifetimeCallback
    hr
}

pub unsafe fn CDrvDefExt_AddPages_body(this: *mut CDrvDefExt, pfnAddPage: LPFNSVADDPROPSHEETPAGE, lParam: LPARAM) -> HRESULT {
    let hr = CDrvDefExt_AddMainPage_body(this, pfnAddPage, lParam);
    if ((hr) < 0) {
        return hr;
    }

    if !(*this).m_Multiple.is_null() {
        let mut drop = CDrop::new((*this).m_Multiple);
        let count = if drop.hr >= 0 {
            drop.GetCount()
        } else {
            0
        };
        let mut i = 0;
        while {
            i += 1;
            i < count
        } {
            let mut SheetExt: *mut CDrvDefExt = core::ptr::null_mut();
            let _SheetExt_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(SheetExt));
            let hr = ShellObjectCreator(&mut SheetExt);
            if ((hr) < 0) {
                continue;
            }
            if DragQueryFileW(
                drop.hDrop,
                i,
                (*SheetExt).m_wszDrive.as_mut_ptr(),
                (*SheetExt).m_wszDrive.len() as UINT,
            ) == 0
            {
                CDrvDefExt_Release(SheetExt);
                SheetExt = core::ptr::null_mut();
                continue;
            }
            if !IsValidDrivePath_body((*SheetExt).m_wszDrive.as_ptr()) {
                CDrvDefExt_Release(SheetExt);
                SheetExt = core::ptr::null_mut();
                continue;
            }

            (*SheetExt).m_Multiple = (*this).m_Multiple;
            ((*(*(*this).m_Multiple).lpVtbl).AddRef)((*this).m_Multiple);
            CDrvDefExt_AddMainPage_body(SheetExt, pfnAddPage, lParam);
            CDrvDefExt_Release(SheetExt);
            SheetExt = core::ptr::null_mut();
        }
    } else {
        let mut hPage;
        if GetDriveTypeW((*this).m_wszDrive.as_ptr()) == DRIVE_FIXED {
            hPage = SH_CreatePropertySheetPage(
                IDD_DRIVE_TOOLS as WORD,
                Some(ExtraPageProc),
                this as LPARAM,
                core::ptr::null(),
            );
            if hPage != core::ptr::null_mut() {
                if let Some(add) = pfnAddPage {
                    add(hPage, lParam);
                }
            }
        }

        if GetDriveTypeW((*this).m_wszDrive.as_ptr()) != DRIVE_REMOTE {
            hPage = SH_CreatePropertySheetPage(
                IDD_DRIVE_HARDWARE as WORD,
                Some(HardwarePageProc),
                this as LPARAM,
                core::ptr::null(),
            );
            if hPage != core::ptr::null_mut() {
                if let Some(add) = pfnAddPage {
                    add(hPage, lParam);
                }
            }
        }
    }

    S_OK
}

pub unsafe fn CDrvDefExt_ReplacePage_body(this: *mut CDrvDefExt, uPageID: EXPPS, pfnReplacePage: LPFNSVADDPROPSHEETPAGE, lParam: LPARAM) -> HRESULT { E_NOTIMPL }

pub unsafe fn CDrvDefExt_SetSite_body(this: *mut CDrvDefExt, punk: *mut IUnknown) -> HRESULT { E_NOTIMPL }

pub unsafe fn CDrvDefExt_GetSite_body(this: *mut CDrvDefExt, iid: REFIID, ppvSite: *mut PVOID) -> HRESULT { E_NOTIMPL }


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * Provides default drive shell extension
//  *
//  * Copyright 2005 Johannes Anderwald
//  * Copyright 2012 Rafal Harabien
//  * Copyright 2020 Katayama Hirofumi MZ
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
// #include "precomp.h"
//
// #define _USE_MATH_DEFINES
// #include <math.h>
// #include <devguid.h>
//
// #define NTOS_MODE_USER
// #include <ndk/iofuncs.h>
// #include <ndk/obfuncs.h>
//
// WINE_DEFAULT_DEBUG_CHANNEL(shell);
//
// typedef enum
// {
//     HWPD_STANDARDLIST = 0,
//     HWPD_LARGELIST,
//     HWPD_MAX = HWPD_LARGELIST
// } HWPAGE_DISPLAYMODE, *PHWPAGE_DISPLAYMODE;
//
// EXTERN_C HWND WINAPI
// DeviceCreateHardwarePageEx(HWND hWndParent,
//                            LPGUID lpGuids,
//                            UINT uNumberOfGuids,
//                            HWPAGE_DISPLAYMODE DisplayMode);
// UINT SH_FormatByteSize(LONGLONG cbSize, LPWSTR pwszResult, UINT cchResultMax);
//
// static VOID
// GetDriveNameWithLetter(LPWSTR pwszText, UINT cchTextMax, LPCWSTR pwszDrive)
// {
//     DWORD dwMaxComp, dwFileSys;
//     SIZE_T cchText = 0;
//
//     if (GetVolumeInformationW(pwszDrive, pwszText, cchTextMax, NULL, &dwMaxComp, &dwFileSys, NULL, 0))
//     {
//         cchText = wcslen(pwszText);
//         if (cchText == 0)
//         {
//             /* load default volume label */
//             cchText = LoadStringW(shell32_hInstance, IDS_DRIVE_FIXED, pwszText, cchTextMax);
//         }
//     }
//
//     StringCchPrintfW(pwszText + cchText, cchTextMax - cchText, L" (%c:)", pwszDrive[0]);
// }
//
// static VOID
// InitializeChkDskDialog(HWND hwndDlg, LPCWSTR pwszDrive)
// {
//     WCHAR wszText[100];
//     UINT Length;
//     SetWindowLongPtr(hwndDlg, DWLP_USER, (INT_PTR)pwszDrive);
//
//     Length = GetWindowTextW(hwndDlg, wszText, sizeof(wszText) / sizeof(WCHAR));
//     wszText[Length] = L' ';
//     GetDriveNameWithLetter(&wszText[Length + 1], (sizeof(wszText) / sizeof(WCHAR)) - Length - 1, pwszDrive);
//     SetWindowText(hwndDlg, wszText);
// }
//
// static HWND hChkdskDrvDialog = NULL;
// static BOOLEAN bChkdskSuccess = FALSE;
//
// static BOOLEAN NTAPI
// ChkdskCallback(
//      CALLBACKCOMMAND Command,
//      ULONG SubAction,
//      PVOID ActionInfo)
// {
//     PDWORD Progress;
//     PBOOLEAN pSuccess;
//     switch(Command)
//     {
//         case PROGRESS:
//             Progress = (PDWORD)ActionInfo;
//             SendDlgItemMessageW(hChkdskDrvDialog, 14002, PBM_SETPOS, (WPARAM)*Progress, 0);
//             break;
//         case DONE:
//             pSuccess = (PBOOLEAN)ActionInfo;
//             bChkdskSuccess = (*pSuccess);
//             break;
//
//         case VOLUMEINUSE:
//         case INSUFFICIENTRIGHTS:
//         case FSNOTSUPPORTED:
//         case CLUSTERSIZETOOSMALL:
//             bChkdskSuccess = FALSE;
//             FIXME("\n");
//             break;
//
//         default:
//             break;
//     }
//
//     return TRUE;
// }
//
// static VOID
// ChkDskNow(HWND hwndDlg, LPCWSTR pwszDrive)
// {
//     //DWORD ClusterSize = 0;
//     WCHAR wszFs[30];
//     ULARGE_INTEGER TotalNumberOfFreeBytes, FreeBytesAvailableUser;
//     BOOLEAN bCorrectErrors = FALSE, bScanDrive = FALSE;
//
//     if(!GetVolumeInformationW(pwszDrive, NULL, 0, NULL, NULL, NULL, wszFs, _countof(wszFs)))
//     {
//         FIXME("failed to get drive fs type\n");
//         return;
//     }
//
//     if (!GetDiskFreeSpaceExW(pwszDrive, &FreeBytesAvailableUser, &TotalNumberOfFreeBytes, NULL))
//     {
//         FIXME("failed to get drive space type\n");
//         return;
//     }
//
//     /*if (!GetDefaultClusterSize(wszFs, &ClusterSize, &TotalNumberOfFreeBytes))
//     {
//         FIXME("invalid cluster size\n");
//         return;
//     }*/
//
//     if (SendDlgItemMessageW(hwndDlg, 14000, BM_GETCHECK, 0, 0) == BST_CHECKED)
//         bCorrectErrors = TRUE;
//
//     if (SendDlgItemMessageW(hwndDlg, 14001, BM_GETCHECK, 0, 0) == BST_CHECKED)
//         bScanDrive = TRUE;
//
//     hChkdskDrvDialog = hwndDlg;
//     bChkdskSuccess = FALSE;
//     SendDlgItemMessageW(hwndDlg, 14002, PBM_SETRANGE, 0, MAKELPARAM(0, 100));
//     Chkdsk((LPWSTR)pwszDrive, (LPWSTR)wszFs, bCorrectErrors, TRUE, FALSE, bScanDrive, NULL, NULL, ChkdskCallback); // FIXME: casts
//
//     hChkdskDrvDialog = NULL;
//     bChkdskSuccess = FALSE;
// }
//
// static INT_PTR __stdcall
// ChkDskDlg(
//     HWND hwndDlg,
//     UINT uMsg,
//     WPARAM wParam,
//     LPARAM lParam)
// {
//     switch(uMsg)
//     {
//         case WM_INITDIALOG:
//             SetWindowLongPtr(hwndDlg, DWLP_USER, (LONG_PTR)lParam);
//             InitializeChkDskDialog(hwndDlg, (LPCWSTR)lParam);
//             return TRUE;
//         case WM_COMMAND:
//             switch(LOWORD(wParam))
//             {
//                 case IDCANCEL:
//                     EndDialog(hwndDlg, 0);
//                     break;
//                 case IDOK:
//                 {
//                     LPCWSTR pwszDrive = (LPCWSTR)GetWindowLongPtr(hwndDlg, DWLP_USER);
//                     ChkDskNow(hwndDlg, pwszDrive);
//                     break;
//                 }
//             }
//             break;
//     }
//
//     return FALSE;
// }
//
// VOID
// CDrvDefExt::PaintStaticControls(HWND hwndDlg, LPDRAWITEMSTRUCT pDrawItem)
// {
//     HBRUSH hBrush;
//
//     if (pDrawItem->CtlID == 14013)
//     {
//         hBrush = CreateSolidBrush(RGB(0, 0, 255));
//         if (hBrush)
//         {
//             FillRect(pDrawItem->hDC, &pDrawItem->rcItem, hBrush);
//             DeleteObject((HGDIOBJ)hBrush);
//         }
//     }
//     else if (pDrawItem->CtlID == 14014)
//     {
//         hBrush = CreateSolidBrush(RGB(255, 0, 255));
//         if (hBrush)
//         {
//             FillRect(pDrawItem->hDC, &pDrawItem->rcItem, hBrush);
//             DeleteObject((HGDIOBJ)hBrush);
//         }
//     }
//     else if (pDrawItem->CtlID == 14015)
//     {
//         HBRUSH hBlueBrush = CreateSolidBrush(RGB(0, 0, 255));
//         HBRUSH hMagBrush = CreateSolidBrush(RGB(255, 0, 255));
//         HBRUSH hbrOld;
//         HPEN hBlackPen = (HPEN)GetStockObject(BLACK_PEN);
//         HPEN hDarkBluePen = CreatePen(PS_SOLID, 1, RGB(0, 0, 128));
//         HPEN hDarkMagPen = CreatePen(PS_SOLID, 1, RGB(128, 0, 128));
//         HPEN hOldPen = (HPEN)SelectObject(pDrawItem->hDC, hDarkMagPen);
//         INT xCenter = (pDrawItem->rcItem.left + pDrawItem->rcItem.right) / 2;
//         INT yCenter = (pDrawItem->rcItem.top + pDrawItem->rcItem.bottom - 10) / 2;
//         INT cx = pDrawItem->rcItem.right - pDrawItem->rcItem.left;
//         INT cy = pDrawItem->rcItem.bottom - pDrawItem->rcItem.top - 10;
//         INT xRadial = xCenter + (INT)(cos(M_PI + m_FreeSpacePerc / 100.0f * M_PI * 2.0f) * cx / 2);
//         INT yRadial = yCenter - (INT)(sin(M_PI + m_FreeSpacePerc / 100.0f * M_PI * 2.0f) * cy / 2);
//
//         TRACE("FreeSpace %u a %f cx %d\n", m_FreeSpacePerc, M_PI+m_FreeSpacePerc / 100.0f * M_PI * 2.0f, cx);
//
//         for (INT x = pDrawItem->rcItem.left; x < pDrawItem->rcItem.right; ++x)
//         {
//             double cos_val = (x - xCenter) * 2.0f / cx;
//             INT y = yCenter + (INT)(sin(acos(cos_val)) * cy / 2) - 1;
//             HPEN hCenterPen;
//
//             if (m_FreeSpacePerc < 50 && x == xRadial)
//                 SelectObject(pDrawItem->hDC, hDarkBluePen);
//             
//             /* Temporarily change pens to draw edges */
//             if (x == pDrawItem->rcItem.left)
//                 hCenterPen = (HPEN)SelectObject(pDrawItem->hDC, hBlackPen);
//             else if (x == pDrawItem->rcItem.right - 1)
//                 SelectObject(pDrawItem->hDC, hBlackPen);
//
//             MoveToEx(pDrawItem->hDC, x, y, NULL);
//             LineTo(pDrawItem->hDC, x, y + 10);
//             SetPixel(pDrawItem->hDC, x, y + 10, RGB(0, 0, 0));
//             
//             /* Restore fill section pens */
//             if (x == pDrawItem->rcItem.left)
//                 SelectObject(pDrawItem->hDC, hCenterPen);
//         }
//
//         SelectObject(pDrawItem->hDC, hBlackPen);
//
//         if (m_FreeSpacePerc > 50)
//         {
//             hbrOld = (HBRUSH)SelectObject(pDrawItem->hDC, hMagBrush);
//
//             Ellipse(pDrawItem->hDC, pDrawItem->rcItem.left, pDrawItem->rcItem.top,
//                     pDrawItem->rcItem.right, pDrawItem->rcItem.bottom - 10);
//
//             SelectObject(pDrawItem->hDC, hBlueBrush);
//
//             if (m_FreeSpacePerc < 100)
//             {
//                 Pie(pDrawItem->hDC, pDrawItem->rcItem.left, pDrawItem->rcItem.top, pDrawItem->rcItem.right,
//                     pDrawItem->rcItem.bottom - 10, xRadial, yRadial, pDrawItem->rcItem.left, yCenter);
//             }
//         }
//         else
//         {
//             hbrOld = (HBRUSH)SelectObject(pDrawItem->hDC, hBlueBrush);
//
//             Ellipse(pDrawItem->hDC, pDrawItem->rcItem.left, pDrawItem->rcItem.top,
//                     pDrawItem->rcItem.right, pDrawItem->rcItem.bottom - 10);
//
//             SelectObject(pDrawItem->hDC, hMagBrush);
//
//             if (m_FreeSpacePerc > 0)
//             {
//                 Pie(pDrawItem->hDC, pDrawItem->rcItem.left, pDrawItem->rcItem.top, pDrawItem->rcItem.right,
//                     pDrawItem->rcItem.bottom - 10, pDrawItem->rcItem.left, yCenter, xRadial, yRadial);
//             }
//         }
//
//         SelectObject(pDrawItem->hDC, hbrOld);
//         SelectObject(pDrawItem->hDC, hOldPen);
//
//         DeleteObject(hBlueBrush);
//         DeleteObject(hMagBrush);
//         DeleteObject(hDarkBluePen);
//         DeleteObject(hDarkMagPen);
//     }
// }
//
// // https://stackoverflow.com/questions/3098696/get-information-about-disk-drives-result-on-windows7-32-bit-system/3100268#3100268
// static BOOL
// GetDriveTypeAndCharacteristics(HANDLE hDevice, DEVICE_TYPE *pDeviceType, ULONG *pCharacteristics)
// {
//     NTSTATUS Status;
//     IO_STATUS_BLOCK IoStatusBlock;
//     FILE_FS_DEVICE_INFORMATION DeviceInfo;
//
//     Status = NtQueryVolumeInformationFile(hDevice, &IoStatusBlock,
//                                           &DeviceInfo, sizeof(DeviceInfo),
//                                           FileFsDeviceInformation);
//     if (Status == NO_ERROR)
//     {
//         *pDeviceType = DeviceInfo.DeviceType;
//         *pCharacteristics = DeviceInfo.Characteristics;
//         return TRUE;
//     }
//
//     return FALSE;
// }
//
// BOOL IsDriveFloppyW(LPCWSTR pszDriveRoot)
// {
//     LPCWSTR RootPath = pszDriveRoot;
//     WCHAR szRoot[16], szDeviceName[16];
//     UINT uType;
//     HANDLE hDevice;
//     DEVICE_TYPE DeviceType;
//     ULONG ulCharacteristics;
//     BOOL ret;
//
//     lstrcpynW(szRoot, RootPath, _countof(szRoot));
//
//     if (L'a' <= szRoot[0] && szRoot[0] <= 'z')
//     {
//         szRoot[0] += ('A' - 'a');
//     }
//
//     if ('A' <= szRoot[0] && szRoot[0] <= L'Z' &&
//         szRoot[1] == L':' && szRoot[2] == 0)
//     {
//         // 'C:' --> 'C:\'
//         szRoot[2] = L'\\';
//         szRoot[3] = 0;
//     }
//
//     if (!PathIsRootW(szRoot))
//     {
//         return FALSE;
//     }
//
//     uType = GetDriveTypeW(szRoot);
//     if (uType == DRIVE_REMOVABLE)
//     {
//         if (szRoot[0] == L'A' || szRoot[0] == L'B')
//             return TRUE;
//     }
//     else
//     {
//         return FALSE;
//     }
//
//     lstrcpynW(szDeviceName, L"\\\\.\\", _countof(szDeviceName));
//     szDeviceName[4] = szRoot[0];
//     szDeviceName[5] = L':';
//     szDeviceName[6] = UNICODE_NULL;
//
//     hDevice = CreateFileW(szDeviceName, FILE_READ_ATTRIBUTES,
//                           FILE_SHARE_READ | FILE_SHARE_WRITE,
//                           NULL, OPEN_EXISTING, 0, NULL);
//     if (hDevice == INVALID_HANDLE_VALUE)
//     {
//         return FALSE;
//     }
//
//     ret = FALSE;
//     if (GetDriveTypeAndCharacteristics(hDevice, &DeviceType, &ulCharacteristics))
//     {
//         if ((ulCharacteristics & FILE_FLOPPY_DISKETTE) == FILE_FLOPPY_DISKETTE)
//             ret = TRUE;
//     }
//
//     CloseHandle(hDevice);
//
//     return ret;
// }
//
// BOOL IsDriveFloppyA(LPCSTR pszDriveRoot)
// {
//     WCHAR szRoot[8];
//     MultiByteToWideChar(CP_ACP, 0, pszDriveRoot, -1, szRoot, _countof(szRoot));
//     return IsDriveFloppyW(szRoot);
// }
//
// VOID
// CDrvDefExt::InitGeneralPage(HWND hwndDlg)
// {
//     WCHAR wszVolumeName[MAX_PATH+1] = {0};
//     WCHAR wszFileSystem[MAX_PATH+1] = {0};
//     WCHAR wszBuf[128];
//     BOOL bRet, bFloppy = FALSE, bHasFS = FALSE;
//
//     bRet = GetVolumeInformationW(m_wszDrive, wszVolumeName, _countof(wszVolumeName), NULL, NULL, NULL, wszFileSystem, _countof(wszFileSystem));
//     if (bRet)
//     {
//         /* Set volume label and filesystem */
//         SetDlgItemTextW(hwndDlg, 14000, wszVolumeName);
//         SetDlgItemTextW(hwndDlg, 14002, wszFileSystem);
//         bHasFS = *wszFileSystem != UNICODE_NULL;
//     }
//     else
//     {
//         LoadStringW(shell32_hInstance, IDS_FS_UNKNOWN, wszFileSystem, _countof(wszFileSystem));
//         SetDlgItemTextW(hwndDlg, 14002, wszFileSystem);
//     }
//
//     /* Set drive type and icon */
//     // TODO: Call SHGetFileInfo to get this info
//     UINT DriveType = GetDriveTypeW(m_wszDrive);
//     UINT IconId, TypeStrId;
//     switch (DriveType)
//     {
//         case DRIVE_REMOVABLE:
//             bFloppy = IsDriveFloppyW(m_wszDrive);
//             IconId = bFloppy ? IDI_SHELL_3_14_FLOPPY : IDI_SHELL_REMOVEABLE;
//             TypeStrId = bFloppy ? IDS_DRIVE_FLOPPY : IDS_DRIVE_REMOVABLE;
//             break;
//         case DRIVE_CDROM: IconId = IDI_SHELL_CDROM; TypeStrId = IDS_DRIVE_CDROM; break;
//         case DRIVE_REMOTE: IconId = IDI_SHELL_NETDRIVE; TypeStrId = IDS_DRIVE_NETWORK; break;
//         case DRIVE_RAMDISK: IconId = IDI_SHELL_RAMDISK; break;
//         default: IconId = IDI_SHELL_DRIVE; TypeStrId = IDS_DRIVE_FIXED;
//     }
//
//     BOOL bCanSetLabel = bHasFS;
//     if (DriveType == DRIVE_CDROM || DriveType == DRIVE_REMOTE || bFloppy)
//     {
//         bCanSetLabel = bFloppy && bHasFS;
//
//         /* disk compression */
//         ShowWindow(GetDlgItem(hwndDlg, 14011), FALSE);
//
//         /* index */
//         ShowWindow(GetDlgItem(hwndDlg, 14012), FALSE);
//     }
//     /* volume label textbox */
//     SendMessage(GetDlgItem(hwndDlg, 14000), EM_SETREADONLY, !bCanSetLabel, 0);
//
//     HICON hIcon = (HICON)LoadImage(shell32_hInstance, MAKEINTRESOURCE(IconId), IMAGE_ICON, 32, 32, LR_SHARED);
//     if (hIcon)
//         SendDlgItemMessageW(hwndDlg, 14016, STM_SETICON, (WPARAM)hIcon, 0);
//     if (TypeStrId && LoadStringW(shell32_hInstance, TypeStrId, wszBuf, _countof(wszBuf)))
//         SetDlgItemTextW(hwndDlg, 14001, wszBuf);
//
//     ULARGE_INTEGER FreeBytesAvailable, TotalNumberOfBytes;
//     if(GetDiskFreeSpaceExW(m_wszDrive, &FreeBytesAvailable, &TotalNumberOfBytes, NULL))
//     {
//         /* Init free space percentage used for drawing piechart */
//         m_FreeSpacePerc = (UINT)(FreeBytesAvailable.QuadPart * 100ull / TotalNumberOfBytes.QuadPart);
//
//         /* Used space */
//         if (SH_FormatByteSize(TotalNumberOfBytes.QuadPart - FreeBytesAvailable.QuadPart, wszBuf, _countof(wszBuf)))
//             SetDlgItemTextW(hwndDlg, 14003, wszBuf);
//
//         if (StrFormatByteSizeW(TotalNumberOfBytes.QuadPart - FreeBytesAvailable.QuadPart, wszBuf, _countof(wszBuf)))
//             SetDlgItemTextW(hwndDlg, 14004, wszBuf);
//
//         /* Free space */
//         if (SH_FormatByteSize(FreeBytesAvailable.QuadPart, wszBuf, _countof(wszBuf)))
//             SetDlgItemTextW(hwndDlg, 14005, wszBuf);
//
//         if (StrFormatByteSizeW(FreeBytesAvailable.QuadPart, wszBuf, _countof(wszBuf)))
//             SetDlgItemTextW(hwndDlg, 14006, wszBuf);
//
//         /* Total space */
//         if (SH_FormatByteSize(TotalNumberOfBytes.QuadPart, wszBuf, _countof(wszBuf)))
//             SetDlgItemTextW(hwndDlg, 14007, wszBuf);
//
//         if (StrFormatByteSizeW(TotalNumberOfBytes.QuadPart, wszBuf, _countof(wszBuf)))
//             SetDlgItemTextW(hwndDlg, 14008, wszBuf);
//     }
//     else
//     {
//         m_FreeSpacePerc = 0;
//
//         if (SH_FormatByteSize(0, wszBuf, _countof(wszBuf)))
//         {
//             SetDlgItemTextW(hwndDlg, 14003, wszBuf);
//             SetDlgItemTextW(hwndDlg, 14005, wszBuf);
//             SetDlgItemTextW(hwndDlg, 14007, wszBuf);
//         }
//         if (StrFormatByteSizeW(0, wszBuf, _countof(wszBuf)))
//         {
//             SetDlgItemTextW(hwndDlg, 14004, wszBuf);
//             SetDlgItemTextW(hwndDlg, 14006, wszBuf);
//             SetDlgItemTextW(hwndDlg, 14008, wszBuf);
//         }
//     }
//
//     /* Set drive description */
//     WCHAR wszFormat[50];
//     GetDlgItemTextW(hwndDlg, 14009, wszFormat, _countof(wszFormat));
//     swprintf(wszBuf, wszFormat, m_wszDrive[0]);
//     SetDlgItemTextW(hwndDlg, 14009, wszBuf);
//
//     /* show disk cleanup button only for fixed drives */
//     ShowWindow(GetDlgItem(hwndDlg, 14010), DriveType == DRIVE_FIXED);
// }
//
// INT_PTR __stdcall
// CDrvDefExt::GeneralPageProc(
//     HWND hwndDlg,
//     UINT uMsg,
//     WPARAM wParam,
//     LPARAM lParam)
// {
//     switch(uMsg)
//     {
//         case WM_INITDIALOG:
//         {
//             LPPROPSHEETPAGEW ppsp = (LPPROPSHEETPAGEW)lParam;
//             if (ppsp == NULL)
//                 break;
//
//             CDrvDefExt *pDrvDefExt = reinterpret_cast<CDrvDefExt *>(ppsp->lParam);
//             SetWindowLongPtr(hwndDlg, DWLP_USER, (LONG_PTR)pDrvDefExt);
//             pDrvDefExt->InitGeneralPage(hwndDlg);
//             return TRUE;
//         }
//         case WM_DRAWITEM:
//         {
//             LPDRAWITEMSTRUCT pDrawItem = (LPDRAWITEMSTRUCT)lParam;
//
//             if (pDrawItem->CtlID >= 14013 && pDrawItem->CtlID <= 14015)
//             {
//                 CDrvDefExt *pDrvDefExt = reinterpret_cast<CDrvDefExt *>(GetWindowLongPtr(hwndDlg, DWLP_USER));
//                 pDrvDefExt->PaintStaticControls(hwndDlg, pDrawItem);
//                 return TRUE;
//             }
//             break;
//         }
//         case WM_PAINT:
//             break;
//         case WM_COMMAND:
//             if (LOWORD(wParam) == 14010) /* Disk Cleanup */
//             {
//                 CDrvDefExt *pDrvDefExt = reinterpret_cast<CDrvDefExt *>(GetWindowLongPtr(hwndDlg, DWLP_USER));
//                 WCHAR wszBuf[256];
//                 DWORD cbBuf = sizeof(wszBuf);
//
//                 if (RegGetValueW(HKEY_LOCAL_MACHINE,
//                                  L"SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Explorer\\MyComputer\\CleanupPath",
//                                  NULL,
//                                  RRF_RT_REG_SZ,
//                                  NULL,
//                                  (PVOID)wszBuf,
//                                  &cbBuf) == ERROR_SUCCESS)
//                 {
//                     WCHAR wszCmd[MAX_PATH];
//
//                     StringCbPrintfW(wszCmd, sizeof(wszCmd), wszBuf, pDrvDefExt->m_wszDrive[0]);
//                     WCHAR* wszArgs = PathGetArgsW(wszCmd);
//                     if (wszArgs && *wszArgs && wszArgs != wszCmd)
//                         wszArgs[-1] = UNICODE_NULL;
//                     else
//                         wszArgs = NULL;
//
//                     if (ShellExecuteW(hwndDlg, NULL, wszCmd, wszArgs, NULL, SW_SHOW) <= (HINSTANCE)32)
//                         ERR("Failed to create cleanup process %ls\n", wszCmd);
//                 }
//             }
//             else if (LOWORD(wParam) == 14000) /* Label */
//             {
//                 if (HIWORD(wParam) == EN_CHANGE)
//                     PropSheet_Changed(GetParent(hwndDlg), hwndDlg);
//             }
//             break;
//         case WM_NOTIFY:
//             if (((LPNMHDR)lParam)->hwndFrom == GetParent(hwndDlg))
//             {
//                 /* Property Sheet */
//                 LPPSHNOTIFY lppsn = (LPPSHNOTIFY)lParam;
//
//                 if (lppsn->hdr.code == PSN_APPLY)
//                 {
//                     CDrvDefExt *pDrvDefExt = reinterpret_cast<CDrvDefExt *>(GetWindowLongPtr(hwndDlg, DWLP_USER));
//
//                     HRESULT hr = E_FAIL;
//                     HWND hLabel = GetDlgItem(hwndDlg, 14000);
//                     WCHAR wszBuf[256];
//                     *wszBuf = UNICODE_NULL;
//                     if (GetWindowTextW(hLabel, wszBuf, _countof(wszBuf)) || GetWindowTextLengthW(hLabel) == 0)
//                         hr = CDrivesFolder::SetDriveLabel(hwndDlg, pDrvDefExt->m_wszDrive, wszBuf);
//
//                     SetWindowLongPtr(hwndDlg, DWLP_MSGRESULT, FAILED(hr) ? PSNRET_INVALID : PSNRET_NOERROR);
//                     return TRUE;
//                 }
//             }
//             break;
//
//         default:
//             break;
//     }
//
//     return FALSE;
// }
//
// INT_PTR __stdcall
// CDrvDefExt::ExtraPageProc(
//     HWND hwndDlg,
//     UINT uMsg,
//     WPARAM wParam,
//     LPARAM lParam)
// {
//     switch (uMsg)
//     {
//         case WM_INITDIALOG:
//         {
//             LPPROPSHEETPAGEW ppsp = (LPPROPSHEETPAGEW)lParam;
//             SetWindowLongPtr(hwndDlg, DWLP_USER, (LONG_PTR)ppsp->lParam);
//             return TRUE;
//         }
//         case WM_COMMAND:
//         {
//             WCHAR wszBuf[MAX_PATH];
//             DWORD cbBuf = sizeof(wszBuf);
//             CDrvDefExt *pDrvDefExt = reinterpret_cast<CDrvDefExt *>(GetWindowLongPtr(hwndDlg, DWLP_USER));
//
//             switch(LOWORD(wParam))
//             {
//                 case 14000:
//                     DialogBoxParamW(shell32_hInstance, MAKEINTRESOURCEW(IDD_CHECK_DISK), hwndDlg, ChkDskDlg, (LPARAM)pDrvDefExt->m_wszDrive);
//                     break;
//                 case 14001:
//                     if (RegGetValueW(HKEY_LOCAL_MACHINE,
//                                      L"SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Explorer\\MyComputer\\DefragPath",
//                                      NULL,
//                                      RRF_RT_REG_SZ,
//                                      NULL,
//                                      (PVOID)wszBuf,
//                                      &cbBuf) == ERROR_SUCCESS)
//                     {
//                         WCHAR wszCmd[MAX_PATH];
//
//                         StringCbPrintfW(wszCmd, sizeof(wszCmd), wszBuf, pDrvDefExt->m_wszDrive[0]);
//
//                         if (ShellExecuteW(hwndDlg, NULL, wszCmd, NULL, NULL, SW_SHOW) <= (HINSTANCE)32)
//                             ERR("Failed to create defrag process %ls\n", wszCmd);
//                     }
//                     break;
//                 case 14002:
//                     if (RegGetValueW(HKEY_LOCAL_MACHINE,
//                                      L"SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Explorer\\MyComputer\\BackupPath",
//                                      NULL,
//                                      RRF_RT_REG_SZ,
//                                      NULL,
//                                      (PVOID)wszBuf,
//                                      &cbBuf) == ERROR_SUCCESS)
//                     {
//                         if (ShellExecuteW(hwndDlg, NULL, wszBuf, NULL, NULL, SW_SHOW) <= (HINSTANCE)32)
//                             ERR("Failed to create backup process %ls\n", wszBuf);
//                     }
//             }
//             break;
//         }
//     }
//     return FALSE;
// }
//
// INT_PTR __stdcall
// CDrvDefExt::HardwarePageProc(
//     HWND hwndDlg,
//     UINT uMsg,
//     WPARAM wParam,
//     LPARAM lParam)
// {
//     UNREFERENCED_PARAMETER(lParam);
//     UNREFERENCED_PARAMETER(wParam);
//
//     switch(uMsg)
//     {
//         case WM_INITDIALOG:
//         {
//             GUID Guids[2];
//             Guids[0] = GUID_DEVCLASS_DISKDRIVE;
//             Guids[1] = GUID_DEVCLASS_CDROM;
//
//             /* create the hardware page */
//             DeviceCreateHardwarePageEx(hwndDlg, Guids, _countof(Guids), HWPD_STANDARDLIST);
//             break;
//         }
//     }
//
//     return FALSE;
// }
//
// CDrvDefExt::CDrvDefExt()
// {
//     m_wszDrive[0] = L'\0';
// }
//
// CDrvDefExt::~CDrvDefExt()
// {
//
// }
//
// struct CDrop
// {
//     HRESULT hr;
//     STGMEDIUM stgm;
//     HDROP hDrop;
//
//     explicit CDrop(IDataObject *pDO)
//     {
//         FORMATETC format = { CF_HDROP, NULL, DVASPECT_CONTENT, -1, TYMED_HGLOBAL };
//         hDrop = SUCCEEDED(hr = pDO->GetData(&format, &stgm)) ? (HDROP)stgm.hGlobal : NULL;
//     }
//
//     ~CDrop()
//     {
//         if (hDrop)
//             ReleaseStgMedium(&stgm);
//     }
//
//     UINT GetCount()
//     {
//         return DragQueryFileW(hDrop, -1, NULL, 0);
//     }
// };
//
// static inline bool
// IsValidDrivePath(PCWSTR Path)
// {
//     return GetDriveTypeW(Path) > DRIVE_NO_ROOT_DIR;
// }
//
// HRESULT WINAPI
// CDrvDefExt::Initialize(PCIDLIST_ABSOLUTE pidlFolder, IDataObject *pDataObj, HKEY hkeyProgID)
// {
//     HRESULT hr;
//
//     TRACE("%p %p %p %p\n", this, pidlFolder, pDataObj, hkeyProgID);
//
//     if (!pDataObj)
//         return E_FAIL;
//
//
//     CDrop drop(pDataObj);
//     if (FAILED_UNEXPECTEDLY(hr = drop.hr))
//         return hr;
//
//     if (!DragQueryFileW(drop.hDrop, 0, m_wszDrive, _countof(m_wszDrive)))
//     {
//         ERR("DragQueryFileW failed\n");
//         return E_FAIL;
//     }
//
//     if (drop.GetCount() > 1)
//         m_Multiple = pDataObj;
//
//     TRACE("Drive properties %ls\n", m_wszDrive);
//
//     return S_OK;
// }
//
// HRESULT WINAPI
// CDrvDefExt::QueryContextMenu(HMENU hmenu, UINT indexMenu, UINT idCmdFirst, UINT idCmdLast, UINT uFlags)
// {
//     UNIMPLEMENTED;
//     return E_NOTIMPL;
// }
//
// HRESULT WINAPI
// CDrvDefExt::InvokeCommand(LPCMINVOKECOMMANDINFO lpici)
// {
//     UNIMPLEMENTED;
//     return E_NOTIMPL;
// }
//
// HRESULT WINAPI
// CDrvDefExt::GetCommandString(UINT_PTR idCmd, UINT uType, UINT *pwReserved, LPSTR pszName, UINT cchMax)
// {
//     UNIMPLEMENTED;
//     return E_NOTIMPL;
// }
//
// HRESULT
// CDrvDefExt::AddMainPage(LPFNADDPROPSHEETPAGE pfnAddPage, LPARAM lParam)
// {
//     WCHAR szTitle[MAX_PATH], *pszTitle = NULL;
//     if (m_Multiple)
//     {
//         CComHeapPtr<ITEMIDLIST_ABSOLUTE> pidl(SHSimpleIDListFromPathW(m_wszDrive));
//         if (SUCCEEDED(SHGetNameAndFlagsW(pidl, SHGDN_INFOLDER, szTitle, _countof(szTitle), NULL)))
//             pszTitle = szTitle;
//     }
//
//     HPROPSHEETPAGE hPage;
//     hPage = SH_CreatePropertySheetPageEx(IDD_DRIVE_PROPERTIES, GeneralPageProc, (LPARAM)this,
//                                          pszTitle, &PropSheetPageLifetimeCallback<CDrvDefExt>);
//     HRESULT hr = AddPropSheetPage(hPage, pfnAddPage, lParam);
//     if (FAILED_UNEXPECTEDLY(hr))
//         return hr;
//     else
//         AddRef(); // For PropSheetPageLifetimeCallback
//     return hr;
// }
//
// HRESULT WINAPI
// CDrvDefExt::AddPages(LPFNADDPROPSHEETPAGE pfnAddPage, LPARAM lParam)
// {
//     HRESULT hr = AddMainPage(pfnAddPage, lParam);
//     if (FAILED_UNEXPECTEDLY(hr))
//         return hr;
//
//     if (m_Multiple)
//     {
//         CDrop drop(m_Multiple);
//         UINT count = SUCCEEDED(drop.hr) ? drop.GetCount() : 0;
//         for (UINT i = 0; ++i < count;) // Skipping the first drive since it already has a page
//         {
//             CComPtr<CDrvDefExt> SheetExt;
//             if (FAILED_UNEXPECTEDLY(hr = ShellObjectCreator(SheetExt)))
//                 continue;
//             if (!DragQueryFileW(drop.hDrop, i, SheetExt->m_wszDrive, _countof(SheetExt->m_wszDrive)))
//                 continue;
//             if (!IsValidDrivePath(SheetExt->m_wszDrive))
//                 continue;
//
//             SheetExt->m_Multiple = m_Multiple;
//             SheetExt->AddMainPage(pfnAddPage, lParam);
//         }
//     }
//     else
//     {
//         HPROPSHEETPAGE hPage;
//         if (GetDriveTypeW(m_wszDrive) == DRIVE_FIXED)
//         {
//             hPage = SH_CreatePropertySheetPage(IDD_DRIVE_TOOLS,
//                                                ExtraPageProc,
//                                                (LPARAM)this,
//                                                NULL);
//             if (hPage)
//                 pfnAddPage(hPage, lParam);
//         }
//
//         if (GetDriveTypeW(m_wszDrive) != DRIVE_REMOTE)
//         {
//             hPage = SH_CreatePropertySheetPage(IDD_DRIVE_HARDWARE,
//                                                HardwarePageProc,
//                                                (LPARAM)this,
//                                                NULL);
//             if (hPage)
//                 pfnAddPage(hPage, lParam);
//         }
//     }
//
//     return S_OK;
// }
//
// HRESULT WINAPI
// CDrvDefExt::ReplacePage(UINT uPageID, LPFNADDPROPSHEETPAGE pfnReplacePage, LPARAM lParam)
// {
//     UNIMPLEMENTED;
//     return E_NOTIMPL;
// }
//
// HRESULT WINAPI
// CDrvDefExt::SetSite(IUnknown *punk)
// {
//     UNIMPLEMENTED;
//     return E_NOTIMPL;
// }
//
// HRESULT WINAPI
// CDrvDefExt::GetSite(REFIID iid, void **ppvSite)
// {
//     UNIMPLEMENTED;
//     return E_NOTIMPL;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
