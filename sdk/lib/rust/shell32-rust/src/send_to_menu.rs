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

// Deterministic COM scaffold for CSendToMenu; method bodies are generated separately.
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
pub struct CSendToMenu {
    pub IContextMenu3_iface: IContextMenu3,
    pub IContextMenu2_iface: IContextMenu2,
    pub IContextMenu_iface: IContextMenu,
    pub IShellExtInit_iface: IShellExtInit,
    pub ref_count: core::sync::atomic::AtomicI32,
    pub m_hSubMenu: HMENU,
    pub m_pItems: *mut SENDTO_ITEM,
    pub m_idCmdFirst: UINT,
    pub m_pDesktop: *mut IShellFolder,
    pub m_pSendTo: *mut IShellFolder,
    pub m_pDataObject: *mut IDataObject,
}

unsafe fn CSendToMenu_cpp_com_assign_m_pDesktop(this: *mut CSendToMenu, value: *mut IShellFolder) {
    let old = (*this).m_pDesktop;
    if old == value { return; }
    if !value.is_null() {
        let unknown = value as *mut IUnknown;
        let add_ref: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).AddRef);
        add_ref(unknown);
    }
    (*this).m_pDesktop = value;
    if !old.is_null() {
        let unknown = old as *mut IUnknown;
        let release: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).Release);
        release(unknown);
    }
}

unsafe fn CSendToMenu_cpp_com_assign_m_pSendTo(this: *mut CSendToMenu, value: *mut IShellFolder) {
    let old = (*this).m_pSendTo;
    if old == value { return; }
    if !value.is_null() {
        let unknown = value as *mut IUnknown;
        let add_ref: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).AddRef);
        add_ref(unknown);
    }
    (*this).m_pSendTo = value;
    if !old.is_null() {
        let unknown = old as *mut IUnknown;
        let release: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).Release);
        release(unknown);
    }
}

unsafe fn CSendToMenu_cpp_com_assign_m_pDataObject(this: *mut CSendToMenu, value: *mut IDataObject) {
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

unsafe extern "system" fn CSendToMenu_IContextMenu3_QueryInterface(This: *mut IContextMenu3, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CSendToMenu, IContextMenu3_iface)) as *mut CSendToMenu;
    CSendToMenu_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CSendToMenu_IContextMenu3_AddRef(This: *mut IContextMenu3) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CSendToMenu, IContextMenu3_iface)) as *mut CSendToMenu;
    CSendToMenu_AddRef(this)
}
unsafe extern "system" fn CSendToMenu_IContextMenu3_Release(This: *mut IContextMenu3) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CSendToMenu, IContextMenu3_iface)) as *mut CSendToMenu;
    CSendToMenu_Release(this)
}
unsafe extern "system" fn CSendToMenu_IContextMenu3_QueryContextMenu(This: *mut IContextMenu3, hmenu: HMENU, indexMenu: UINT, idCmdFirst: UINT, idCmdLast: UINT, uFlags: UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CSendToMenu, IContextMenu3_iface)) as *mut CSendToMenu;
    CSendToMenu_QueryContextMenu_body(this, hmenu, indexMenu, idCmdFirst, idCmdLast, uFlags)
}
unsafe extern "system" fn CSendToMenu_IContextMenu3_InvokeCommand(This: *mut IContextMenu3, lpici: LPCMINVOKECOMMANDINFO) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CSendToMenu, IContextMenu3_iface)) as *mut CSendToMenu;
    CSendToMenu_InvokeCommand_body(this, lpici)
}
unsafe extern "system" fn CSendToMenu_IContextMenu3_GetCommandString(This: *mut IContextMenu3, idCmd: UINT_PTR, uType: UINT, pwReserved: *mut UINT, pszName: LPSTR, cchMax: UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CSendToMenu, IContextMenu3_iface)) as *mut CSendToMenu;
    CSendToMenu_GetCommandString_body(this, idCmd, uType, pwReserved, pszName, cchMax)
}
unsafe extern "system" fn CSendToMenu_IContextMenu3_HandleMenuMsg(This: *mut IContextMenu3, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CSendToMenu, IContextMenu3_iface)) as *mut CSendToMenu;
    CSendToMenu_HandleMenuMsg_body(this, uMsg, wParam, lParam)
}
unsafe extern "system" fn CSendToMenu_IContextMenu3_HandleMenuMsg2(This: *mut IContextMenu3, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, plResult: *mut LRESULT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CSendToMenu, IContextMenu3_iface)) as *mut CSendToMenu;
    CSendToMenu_HandleMenuMsg2_body(this, uMsg, wParam, lParam, plResult)
}

pub static CSENDTOMENU_ICONTEXTMENU3_VTBL: IContextMenu3Vtbl = IContextMenu3Vtbl {
    QueryInterface: CSendToMenu_IContextMenu3_QueryInterface,
    AddRef: CSendToMenu_IContextMenu3_AddRef,
    Release: CSendToMenu_IContextMenu3_Release,
    QueryContextMenu: CSendToMenu_IContextMenu3_QueryContextMenu,
    InvokeCommand: CSendToMenu_IContextMenu3_InvokeCommand,
    GetCommandString: CSendToMenu_IContextMenu3_GetCommandString,
    HandleMenuMsg: CSendToMenu_IContextMenu3_HandleMenuMsg,
    HandleMenuMsg2: CSendToMenu_IContextMenu3_HandleMenuMsg2,
};

unsafe extern "system" fn CSendToMenu_IContextMenu2_QueryInterface(This: *mut IContextMenu2, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CSendToMenu, IContextMenu2_iface)) as *mut CSendToMenu;
    CSendToMenu_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CSendToMenu_IContextMenu2_AddRef(This: *mut IContextMenu2) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CSendToMenu, IContextMenu2_iface)) as *mut CSendToMenu;
    CSendToMenu_AddRef(this)
}
unsafe extern "system" fn CSendToMenu_IContextMenu2_Release(This: *mut IContextMenu2) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CSendToMenu, IContextMenu2_iface)) as *mut CSendToMenu;
    CSendToMenu_Release(this)
}
unsafe extern "system" fn CSendToMenu_IContextMenu2_QueryContextMenu(This: *mut IContextMenu2, hmenu: HMENU, indexMenu: UINT, idCmdFirst: UINT, idCmdLast: UINT, uFlags: UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CSendToMenu, IContextMenu2_iface)) as *mut CSendToMenu;
    CSendToMenu_QueryContextMenu_body(this, hmenu, indexMenu, idCmdFirst, idCmdLast, uFlags)
}
unsafe extern "system" fn CSendToMenu_IContextMenu2_InvokeCommand(This: *mut IContextMenu2, lpici: LPCMINVOKECOMMANDINFO) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CSendToMenu, IContextMenu2_iface)) as *mut CSendToMenu;
    CSendToMenu_InvokeCommand_body(this, lpici)
}
unsafe extern "system" fn CSendToMenu_IContextMenu2_GetCommandString(This: *mut IContextMenu2, idCmd: UINT_PTR, uType: UINT, pwReserved: *mut UINT, pszName: LPSTR, cchMax: UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CSendToMenu, IContextMenu2_iface)) as *mut CSendToMenu;
    CSendToMenu_GetCommandString_body(this, idCmd, uType, pwReserved, pszName, cchMax)
}
unsafe extern "system" fn CSendToMenu_IContextMenu2_HandleMenuMsg(This: *mut IContextMenu2, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CSendToMenu, IContextMenu2_iface)) as *mut CSendToMenu;
    CSendToMenu_HandleMenuMsg_body(this, uMsg, wParam, lParam)
}

pub static CSENDTOMENU_ICONTEXTMENU2_VTBL: IContextMenu2Vtbl = IContextMenu2Vtbl {
    QueryInterface: CSendToMenu_IContextMenu2_QueryInterface,
    AddRef: CSendToMenu_IContextMenu2_AddRef,
    Release: CSendToMenu_IContextMenu2_Release,
    QueryContextMenu: CSendToMenu_IContextMenu2_QueryContextMenu,
    InvokeCommand: CSendToMenu_IContextMenu2_InvokeCommand,
    GetCommandString: CSendToMenu_IContextMenu2_GetCommandString,
    HandleMenuMsg: CSendToMenu_IContextMenu2_HandleMenuMsg,
};

unsafe extern "system" fn CSendToMenu_IContextMenu_QueryInterface(This: *mut IContextMenu, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CSendToMenu, IContextMenu_iface)) as *mut CSendToMenu;
    CSendToMenu_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CSendToMenu_IContextMenu_AddRef(This: *mut IContextMenu) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CSendToMenu, IContextMenu_iface)) as *mut CSendToMenu;
    CSendToMenu_AddRef(this)
}
unsafe extern "system" fn CSendToMenu_IContextMenu_Release(This: *mut IContextMenu) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CSendToMenu, IContextMenu_iface)) as *mut CSendToMenu;
    CSendToMenu_Release(this)
}
unsafe extern "system" fn CSendToMenu_IContextMenu_QueryContextMenu(This: *mut IContextMenu, hmenu: HMENU, indexMenu: UINT, idCmdFirst: UINT, idCmdLast: UINT, uFlags: UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CSendToMenu, IContextMenu_iface)) as *mut CSendToMenu;
    CSendToMenu_QueryContextMenu_body(this, hmenu, indexMenu, idCmdFirst, idCmdLast, uFlags)
}
unsafe extern "system" fn CSendToMenu_IContextMenu_InvokeCommand(This: *mut IContextMenu, lpici: LPCMINVOKECOMMANDINFO) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CSendToMenu, IContextMenu_iface)) as *mut CSendToMenu;
    CSendToMenu_InvokeCommand_body(this, lpici)
}
unsafe extern "system" fn CSendToMenu_IContextMenu_GetCommandString(This: *mut IContextMenu, idCmd: UINT_PTR, uType: UINT, pwReserved: *mut UINT, pszName: LPSTR, cchMax: UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CSendToMenu, IContextMenu_iface)) as *mut CSendToMenu;
    CSendToMenu_GetCommandString_body(this, idCmd, uType, pwReserved, pszName, cchMax)
}

pub static CSENDTOMENU_ICONTEXTMENU_VTBL: IContextMenuVtbl = IContextMenuVtbl {
    QueryInterface: CSendToMenu_IContextMenu_QueryInterface,
    AddRef: CSendToMenu_IContextMenu_AddRef,
    Release: CSendToMenu_IContextMenu_Release,
    QueryContextMenu: CSendToMenu_IContextMenu_QueryContextMenu,
    InvokeCommand: CSendToMenu_IContextMenu_InvokeCommand,
    GetCommandString: CSendToMenu_IContextMenu_GetCommandString,
};

unsafe extern "system" fn CSendToMenu_IShellExtInit_QueryInterface(This: *mut IShellExtInit, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CSendToMenu, IShellExtInit_iface)) as *mut CSendToMenu;
    CSendToMenu_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CSendToMenu_IShellExtInit_AddRef(This: *mut IShellExtInit) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CSendToMenu, IShellExtInit_iface)) as *mut CSendToMenu;
    CSendToMenu_AddRef(this)
}
unsafe extern "system" fn CSendToMenu_IShellExtInit_Release(This: *mut IShellExtInit) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CSendToMenu, IShellExtInit_iface)) as *mut CSendToMenu;
    CSendToMenu_Release(this)
}
unsafe extern "system" fn CSendToMenu_IShellExtInit_Initialize(This: *mut IShellExtInit, pidlFolder: PCIDLIST_ABSOLUTE, pdtobj: *mut IDataObject, hkeyProgID: HKEY) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CSendToMenu, IShellExtInit_iface)) as *mut CSendToMenu;
    CSendToMenu_Initialize_body(this, pidlFolder, pdtobj, hkeyProgID)
}

pub static CSENDTOMENU_ISHELLEXTINIT_VTBL: IShellExtInitVtbl = IShellExtInitVtbl {
    QueryInterface: CSendToMenu_IShellExtInit_QueryInterface,
    AddRef: CSendToMenu_IShellExtInit_AddRef,
    Release: CSendToMenu_IShellExtInit_Release,
    Initialize: CSendToMenu_IShellExtInit_Initialize,
};

unsafe fn CSendToMenu_QueryInterface(
    this: *mut CSendToMenu, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IUnknown)) || cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IContextMenu3)) {
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
    CSendToMenu_AddRef(this);
    S_OK
}

unsafe fn CSendToMenu_AddRef(this: *mut CSendToMenu) -> ULONG {
    (*this).ref_count.fetch_add(1, core::sync::atomic::Ordering::SeqCst) as ULONG + 1
}

unsafe fn CSendToMenu_Release(this: *mut CSendToMenu) -> ULONG {
    let refs = (*this).ref_count.fetch_sub(1, core::sync::atomic::Ordering::SeqCst) - 1;
    if refs == 0 {
        CSendToMenu_dtor_body(this);
        CSendToMenu_cpp_com_assign_m_pDesktop(this, core::ptr::null_mut());
        CSendToMenu_cpp_com_assign_m_pSendTo(this, core::ptr::null_mut());
        CSendToMenu_cpp_com_assign_m_pDataObject(this, core::ptr::null_mut());
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

// Activation: CLSID_SendToMenu currently maps to CSendToMenu in shell32.cpp.
// The generated C++ adapter redirects ATL activation to this factory.
#[no_mangle] pub unsafe extern "system" fn CSendToMenu_CreateInstance(
    outer: *mut IUnknown, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if !outer.is_null() { return CLASS_E_NOAGGREGATION; }
    let this = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<CSendToMenu>()) as *mut CSendToMenu;
    if this.is_null() { return E_OUTOFMEMORY; }
    core::ptr::write(this, CSendToMenu {
        IContextMenu3_iface: IContextMenu3 { lpVtbl: core::ptr::addr_of!(CSENDTOMENU_ICONTEXTMENU3_VTBL) },
        IContextMenu2_iface: IContextMenu2 { lpVtbl: core::ptr::addr_of!(CSENDTOMENU_ICONTEXTMENU2_VTBL) },
        IContextMenu_iface: IContextMenu { lpVtbl: core::ptr::addr_of!(CSENDTOMENU_ICONTEXTMENU_VTBL) },
        IShellExtInit_iface: IShellExtInit { lpVtbl: core::ptr::addr_of!(CSENDTOMENU_ISHELLEXTINIT_VTBL) },
        ref_count: core::sync::atomic::AtomicI32::new(1),
        m_hSubMenu: core::mem::zeroed(),
        m_pItems: core::mem::zeroed(),
        m_idCmdFirst: core::mem::zeroed(),
        m_pDesktop: core::mem::zeroed(),
        m_pSendTo: core::mem::zeroed(),
        m_pDataObject: core::mem::zeroed(),
    });
    Shell32RustObjectLock();
    CSendToMenu_ctor_body(this);
    let hr = CSendToMenu_QueryInterface(this, riid, ppv);
    CSendToMenu_Release(this);
    hr
}

// Deterministic C++ support closure selected from source identifiers.

pub const IDR_SENDTOMENU: UINT = 158;

pub const IDS_NONE: UINT = 161;

pub const IDS_SENDTO_MENU: UINT = 343;

pub const CMF_VERBSONLY: DWORD = 0x00000002;

pub const CSIDL_SENDTO: DWORD = 0x0009;

pub const DI_NORMAL: DWORD = 3;

pub const MFS_ENABLED: DWORD = 0;

pub const MF_BYPOSITION: DWORD = 1024;

pub const MF_DISABLED: DWORD = 2;

pub const MF_ENABLED: DWORD = 0;

pub const MF_GRAYED: DWORD = 1;

pub const MF_STRING: DWORD = 0;

pub const MIIM_BITMAP: DWORD = 128;

pub const MIIM_DATA: DWORD = 32;

pub const MIIM_STATE: DWORD = 1;

pub const MIIM_SUBMENU: DWORD = 4;

pub const MK_CONTROL: DWORD = 8;

pub const MK_LBUTTON: DWORD = 1;

pub const MK_SHIFT: DWORD = 4;

pub const ODT_MENU: DWORD = 1;

pub const SHCONTF_FOLDERS: DWORD = 32;

pub const SHCONTF_NONFOLDERS: DWORD = 64;

pub const SHGFI_ICON: DWORD = 256;

pub const SHGFI_PIDL: DWORD = 8;

pub const SHGFI_SMALLICON: DWORD = 1;

pub const SHGFI_TYPENAME: DWORD = 1024;

pub const SM_CXMENUCHECK: DWORD = 71;

pub const SM_CXSMICON: DWORD = 49;

pub const SM_CYSMICON: DWORD = 50;

pub const VK_CONTROL: DWORD = 17;

pub const VK_SHIFT: DWORD = 16;

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

pub use crate::island::MEASUREITEMSTRUCT;

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

// Deterministic from oleidl.h
#[repr(C)]
pub struct IDropTarget {
    pub lpVtbl: *const IDropTargetVtbl,
}
#[repr(C)]
pub struct IDropTargetVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IDropTarget, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IDropTarget) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IDropTarget) -> ULONG,
    pub DragEnter: unsafe extern "system" fn(This: *mut IDropTarget, pDataObj: *mut IDataObject, grfKeyState: DWORD, pt: POINTL, pdwEffect: *mut DWORD) -> HRESULT,
    pub DragOver: unsafe extern "system" fn(This: *mut IDropTarget, grfKeyState: DWORD, pt: POINTL, pdwEffect: *mut DWORD) -> HRESULT,
    pub DragLeave: unsafe extern "system" fn(This: *mut IDropTarget) -> HRESULT,
    pub Drop: unsafe extern "system" fn(This: *mut IDropTarget, pDataObj: *mut IDataObject, grfKeyState: DWORD, pt: POINTL, pdwEffect: *mut DWORD) -> HRESULT,
}
extern "C" {
    pub static IID_IDropTarget: IID;
}

macro_rules! LOWORD { ($value:expr) => { (($value as usize) & 0xffff) as UINT }; }

pub type ITEMIDLIST_ABSOLUTE = ITEMIDLIST;

pub type ITEMID_CHILD = ITEMIDLIST;

pub const DROPEFFECT_NONE: DWORD = 0;
pub const DROPEFFECT_COPY: DWORD = 1;
pub const DROPEFFECT_MOVE: DWORD = 2;
pub const DROPEFFECT_LINK: DWORD = 4;

extern "system" {
    fn GetAsyncKeyState(key: INT) -> i16;
    fn GetLastError() -> DWORD;
    fn lstrlenW(text: LPCWSTR) -> INT;
    fn CoTaskMemFree(memory: LPVOID);
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

trait CppComSize { fn cpp_com_size(self) -> usize; }
impl CppComSize for usize { fn cpp_com_size(self) -> usize { self } }
impl CppComSize for u32 { fn cpp_com_size(self) -> usize { self as usize } }
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

#[repr(C)]
pub struct SENDTO_ITEM {
    pub pNext: *mut SENDTO_ITEM,
    pub pidlChild: PITEMID_CHILD,
    pub pszText: LPWSTR,
    pub hIcon: HICON,
}
unsafe fn SENDTO_ITEM_new(
    child: PITEMID_CHILD, text: LPWSTR, icon: HICON,
) -> *mut SENDTO_ITEM {
    let item = HeapAlloc(
        GetProcessHeap(), 0, core::mem::size_of::<SENDTO_ITEM>(),
    ) as *mut SENDTO_ITEM;
    if !item.is_null() {
        item.write(SENDTO_ITEM {
            pNext: core::ptr::null_mut(), pidlChild: child, pszText: text, hIcon: icon,
        });
    }
    item
}
unsafe fn SENDTO_ITEM_delete(item: *mut SENDTO_ITEM) {
    if item.is_null() { return; }
    CoTaskMemFree((*item).pidlChild.cast());
    CoTaskMemFree((*item).pszText.cast());
    if !(*item).hIcon.is_null() { DestroyIcon(((*item).hIcon).cast()); }
    HeapFree(GetProcessHeap(), 0, item.cast::<core::ffi::c_void>());
}

extern "C" {
    static CLSID_SendToMenu: GUID;
    static IID_IShellFolder: GUID;
}

extern "system" {
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
            HeapReAlloc(GetProcessHeap(), 0, self.buffer.cast(), (bytes) as SIZE_T)
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

use core::ffi::c_void;
// The crate's own functions: the definition is the declaration.
use crate::wine::pidl::ILCombine;
use crate::shell32::utils::SHELL_GetUIObjectOfAbsoluteItem;
use crate::desktop_folder_tu::SHGetDesktopFolder;
use crate::wine::shell32_main::SHGetFileInfoW;
use crate::wine::shellpath::SHGetSpecialFolderLocation;
// One declaration per external symbol: island.rs.
use crate::island::AppendMenuW;
use crate::island::CreateMenu;
use crate::island::DestroyIcon;
use crate::island::DestroyMenu;
use crate::island::DrawIconEx;
use crate::island::GetMenuItemInfoW;
use crate::island::GetModuleHandleW;
use crate::island::GetSystemMetrics;
use crate::island::HeapReAlloc;
use crate::island::InsertMenuItemW;
use crate::island::InsertMenuW;
use crate::island::LoadStringW;
use crate::island::SetMenuItemInfoW;
use crate::island::StrRetToStrW;
use crate::shellcom::*;
use crate::types::*;

pub unsafe fn CSendToMenu_ctor_body(this: *mut CSendToMenu) -> () {
    // Initializer list
    (*this).m_hSubMenu = core::ptr::null_mut::<HMENU>() as _;
    (*this).m_pItems = core::ptr::null_mut::<SENDTO_ITEM>();
    (*this).m_idCmdFirst = 0;

    // HRESULT hr = COMMENT_TOKEN_ECHO____(&this->m_pDesktop);
    let hr: HRESULT = SHGetDesktopFolder(&raw mut ((*this).m_pDesktop));
    // if (FAILED(hr)) { }
    if hr < 0 {
    }

    // COMMENT_TOKEN_ECHO____(NULL, &this->m_pSendTo, COMMENT_TOKEN_ECHO____);
    CSendToMenu_GetSpecialFolder_body(this, core::ptr::null_mut(),
        &raw mut ((*this).m_pSendTo),
        CSIDL_SENDTO as INT,
    core::ptr::null_mut());
}

pub unsafe fn CSendToMenu_dtor_body(this: *mut CSendToMenu) -> () {
    CSendToMenu_UnloadAllItems_body(this);
    if !((*this).m_hSubMenu).is_null() {
        DestroyMenu((*this).m_hSubMenu);
        (*this).m_hSubMenu = core::ptr::null_mut();
    }
}

pub unsafe fn CSendToMenu_DoDrop_body(this: *mut CSendToMenu, pDataObject: *mut IDataObject, pDropTarget: *mut IDropTarget) -> HRESULT {
    let mut dwEffect: u32 = DROPEFFECT_MOVE | DROPEFFECT_COPY | DROPEFFECT_LINK;
    let bShift: bool = (GetAsyncKeyState(VK_SHIFT as INT) < 0);
    let bCtrl: bool = (GetAsyncKeyState(VK_CONTROL as INT) < 0);

    // THIS CODE IS NOT HUMAN-FRIENDLY. SORRY.
    // (We have to translate a SendTo action to a Drop action)
    let mut dwKeyState: u32 = MK_LBUTTON;
    if bShift && bCtrl {
        dwKeyState |= MK_SHIFT | MK_CONTROL;
    } else if !bShift {
        dwKeyState |= MK_CONTROL;
    }
    if bCtrl {
        dwKeyState |= MK_SHIFT;
    }

    let mut ptl = POINTL {
        x: 0,
        y: 0,
    };
    let hr: HRESULT = ((*(*pDropTarget).lpVtbl).DragEnter)(pDropTarget, pDataObject, dwKeyState, ptl, &mut dwEffect);
    if ((hr) < 0) {
        ((*(*pDropTarget).lpVtbl).DragLeave)(pDropTarget);
        return hr;
    }

    if dwEffect == DROPEFFECT_NONE {
        // ... possibly a strange comment alignment in C++, we keep the space:
        ((*(*pDropTarget).lpVtbl).DragLeave)(pDropTarget);
        return E_FAIL;
    }

    // THIS CODE IS NOT HUMAN-FRIENDLY. SORRY.
    // (We have to translate a SendTo action to a Drop action)
    if bShift && bCtrl {
        dwEffect = DROPEFFECT_LINK;
    } else if !bShift {
        dwEffect = DROPEFFECT_MOVE;
    } else {
        dwEffect = DROPEFFECT_COPY;
    }

    let hr: HRESULT = ((*(*pDropTarget).lpVtbl).Drop)(pDropTarget, pDataObject, dwKeyState, ptl, &mut dwEffect);
    if ((hr) < 0) {
        return hr;
    }

    hr
}

pub unsafe fn CSendToMenu_GetSpecialFolder_body(this: *mut CSendToMenu, hwnd: HWND, ppFolder: *mut *mut IShellFolder, csidl: i32, ppidl: *mut PIDLIST_ABSOLUTE) -> HRESULT {
    if ppFolder.is_null() {
        return E_POINTER;
    }
    *ppFolder = core::ptr::null_mut();
    if !ppidl.is_null() {
        *ppidl = core::ptr::null_mut();
    }

    let mut pidl: *mut ITEMIDLIST_ABSOLUTE = core::ptr::null_mut();
    let mut hr: HRESULT = SHGetSpecialFolderLocation(hwnd, csidl, &mut pidl as *mut _);

    if ((hr) < 0) {
        if !pidl.is_null() {
            CoTaskMemFree(pidl as *mut core::ffi::c_void);
        }
        return hr;
    }

    let mut pFolder: *mut IShellFolder = core::ptr::null_mut();
    let m_pDesktop = (*this).m_pDesktop;
    let riid = core::ptr::addr_of!(IID_IShellFolder);

    hr = ((*(*m_pDesktop).lpVtbl).BindToObject)(
        m_pDesktop,
        pidl as *const _,
        core::ptr::null_mut(),
        riid,
        &mut pFolder as *mut *mut IShellFolder as *mut *mut core::ffi::c_void,
    );

    if !ppidl.is_null() {
        *ppidl = pidl;
        pidl = core::ptr::null_mut(); // ownership transferred
    }

    if ((hr) < 0) {
        if !pidl.is_null() {
            CoTaskMemFree(pidl as *mut core::ffi::c_void);
        }
        return hr;
    }

    *ppFolder = pFolder;

    if !pidl.is_null() {
        CoTaskMemFree(pidl as *mut core::ffi::c_void);
    }

    hr
}

pub unsafe fn CSendToMenu_GetUIObjectFromPidl_body(this: *mut CSendToMenu, hwnd: HWND, pidl: PIDLIST_ABSOLUTE, riid: REFIID, ppvOut: *mut LPVOID) -> HRESULT {
    *ppvOut = core::ptr::null_mut();
    let hr = SHELL_GetUIObjectOfAbsoluteItem(hwnd, pidl, (riid).cast(), ppvOut);
    if ((hr) < 0) {
        return hr;
    }
    hr
}

pub unsafe fn CSendToMenu_UnloadAllItems_body(this: *mut CSendToMenu) -> () {
    let mut pItems = (*this).m_pItems;
    (*this).m_pItems = core::ptr::null_mut();
    while !pItems.is_null() {
        let pCurItem = pItems;
        pItems = (*pItems).pNext;
        // C++ delete: calls destructor and frees memory.
        // The deterministic layer guarantees that SENDTO_ITEM was allocated
        // via the standard allocator, so Box::from_raw safely reclaims it.
        SENDTO_ITEM_delete(pCurItem);
    }
}

pub unsafe fn CSendToMenu_LoadAllItems_body(this: *mut CSendToMenu, hwnd: HWND) -> HRESULT {
    CSendToMenu_UnloadAllItems_body(this);

    let mut pidlSendTo: CComHeapPtr<ITEMIDLIST_ABSOLUTE> = CComHeapPtr::empty();

    if !(*this).m_pSendTo.is_null() {
        let value = (*this).m_pSendTo as *mut IUnknown;
        ((*(*value).lpVtbl).Release)(value);
        (*this).m_pSendTo = core::ptr::null_mut();
    }
    let mut hr: HRESULT = CSendToMenu_GetSpecialFolder_body(this, hwnd,
        &raw mut (*this).m_pSendTo,
        CSIDL_SENDTO as INT,
        pidlSendTo.as_mut_slot(),
    );
    if ((hr) < 0) {
        return hr;
    }

    let mut pEnumIDList: *mut IEnumIDList = core::ptr::null_mut();
    let _pEnumIDList_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pEnumIDList));
    let _pEnumIDList_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pEnumIDList));
    let pSendTo: *mut IShellFolder = (*this).m_pSendTo;
    hr = ((*(*pSendTo).lpVtbl).EnumObjects)(
        pSendTo,
        hwnd,
        SHCONTF_FOLDERS | SHCONTF_NONFOLDERS,
        &raw mut pEnumIDList as *mut *mut IEnumIDList,
    );
    if ((hr) < 0) {
        return hr;
    }

    hr = S_OK;
    let mut child: PITEMID_CHILD = core::ptr::null_mut();
    while {
        let pEnum = pEnumIDList;
        ((*(*pEnum).lpVtbl).Next)(
            pEnum,
            1,
            &mut child,
            core::ptr::null_mut(),
        )
    } == S_OK
    {
        let mut pidlChild: CComHeapPtr<ITEMID_CHILD> =
            CComHeapPtr::new(child);

        let mut strret: STRRET = core::mem::zeroed();
        hr = ((*(*pSendTo).lpVtbl).GetDisplayNameOf)(
            pSendTo,
            pidlChild.as_raw(),
            SHGDN_NORMAL,
            &mut strret,
        );
        if ((hr) < 0) {
            continue;
        }

        let mut pszText: CComHeapPtr<WCHAR> = CComHeapPtr::empty();
        hr = StrRetToStrW((&raw mut strret).cast(), pidlChild.as_raw(), pszText.as_mut_slot());
        if ((hr) < 0) {
            continue;
        }

        let mut pidlAbsolute: CComHeapPtr<ITEMIDLIST_ABSOLUTE> =
            CComHeapPtr::empty();
        pidlAbsolute.Attach(ILCombine(
            pidlSendTo.as_raw(),
            pidlChild.as_raw(),
        ));

        let mut fi: SHFILEINFOW = core::mem::zeroed();
        let uFlags: u32 = SHGFI_PIDL
            | SHGFI_TYPENAME
            | SHGFI_ICON
            | SHGFI_SMALLICON;
        SHGetFileInfoW(
            (pidlAbsolute.as_raw() as PIDLIST_ABSOLUTE)
                as LPWSTR,
            0,
            &mut fi,
            core::mem::size_of::<SHFILEINFOW>() as u32,
            uFlags,
        );

        let pNewItem: *mut SENDTO_ITEM = SENDTO_ITEM_new(
            pidlChild.Detach(),
            pszText.Detach(),
            (fi.hIcon).cast(),
        );
        if !(*this).m_pItems.is_null() {
            (*pNewItem).pNext = (*this).m_pItems;
        }
        (*this).m_pItems = pNewItem;
    }

    hr
}

pub unsafe fn CSendToMenu_InsertSendToItems_body(this: *mut CSendToMenu, hMenu: HMENU, idCmdFirst: UINT, Pos: UINT) -> UINT {
    if (*this).m_pItems.is_null() {
        let hr = CSendToMenu_LoadAllItems_body(this, core::ptr::null_mut());
        if ((hr) < 0) {
            return 0;
        }
    }
    (*this).m_idCmdFirst = idCmdFirst;
    let mut idCmd = idCmdFirst;
    let mut pCurItem: *mut SENDTO_ITEM = (*this).m_pItems;
    while !pCurItem.is_null() {
        let uFlags: UINT = MF_BYPOSITION | MF_STRING | MF_ENABLED;
        if InsertMenuW(
            hMenu,
            Pos,
            uFlags,
            idCmd as UINT_PTR,
            (*pCurItem).pszText,
        ) != 0
        {
            let mut mii: MENUITEMINFOW = core::mem::zeroed();
            mii.cbSize = core::mem::size_of::<MENUITEMINFOW>() as u32;
            mii.fMask = MIIM_DATA | MIIM_BITMAP;
            mii.dwItemData = pCurItem as ULONG_PTR;
            mii.hbmpItem = HBMMENU_CALLBACK;
            SetMenuItemInfoW(
                hMenu,
                idCmd,
                FALSE,
                &mut mii,
            );
            idCmd = idCmd.wrapping_add(1);
        }
        pCurItem = (*pCurItem).pNext;
    }
    if idCmd == idCmdFirst {
        let mut str_none_buf: [u16; 256] = [0u16; 256];
        let h_instance = GetModuleHandleW(core::ptr::null());
        let len = LoadStringW(
            h_instance,
            IDS_NONE as u32,
            str_none_buf.as_mut_ptr(),
            (str_none_buf.len() - 1) as i32,
        );
        let str_none_ptr = if len > 0 {
            str_none_buf.as_ptr()
        } else {
            core::ptr::null()
        };
        AppendMenuW(
            hMenu,
            MF_GRAYED | MF_DISABLED | MF_STRING,
            idCmd as UINT_PTR,
            str_none_ptr,
        );
        idCmd = idCmd.wrapping_add(1);
    }
    idCmd.wrapping_sub(idCmdFirst)
}

pub unsafe fn CSendToMenu_FindItemFromIdOffset_body(this: *mut CSendToMenu, IdOffset: UINT) -> *mut SENDTO_ITEM {
    let idCmd = (*this).m_idCmdFirst + IdOffset;

    let mut mii: MENUITEMINFOW = core::mem::zeroed();
    mii.cbSize = core::mem::size_of::<MENUITEMINFOW>() as u32;
    mii.fMask = MIIM_DATA;

    if GetMenuItemInfoW((*this).m_hSubMenu, idCmd, FALSE, &mut mii) != 0 {
        return mii.dwItemData as *mut SENDTO_ITEM;
    }

    core::ptr::null_mut()
}

pub unsafe fn CSendToMenu_DoSendToItem_body(this: *mut CSendToMenu, pItem: *mut SENDTO_ITEM, lpici: LPCMINVOKECOMMANDINFO) -> HRESULT {
    if (*this).m_pDataObject.is_null() {
        return E_FAIL;
    }

    let mut hr: HRESULT;
    let mut pDropTarget: *mut IDropTarget = core::ptr::null_mut();
    let _pDropTarget_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pDropTarget));
    let _pDropTarget_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pDropTarget));

    hr = ((*(*(*this).m_pSendTo).lpVtbl).GetUIObjectOf)(
        (*this).m_pSendTo,
        core::ptr::null_mut(),
        1,
        core::ptr::addr_of!((*pItem).pidlChild).cast::<PCUITEMID_CHILD>(),
        &IID_IDropTarget,
        core::ptr::null_mut(),
        &mut pDropTarget as *mut *mut IDropTarget as *mut LPVOID,
    );

    if ((hr) < 0) {
        return hr;
    }

    hr = CSendToMenu_DoDrop_body(this, (*this).m_pDataObject, pDropTarget);

    if ((hr) < 0) {
        if !pDropTarget.is_null() {
            ((*(*pDropTarget).lpVtbl).Release)(pDropTarget);
            pDropTarget = core::ptr::null_mut();
            pDropTarget = core::ptr::null_mut();
        }
        return hr;
    }

    if !pDropTarget.is_null() {
        ((*(*pDropTarget).lpVtbl).Release)(pDropTarget);
        pDropTarget = core::ptr::null_mut();
        pDropTarget = core::ptr::null_mut();
    }

    return hr;
}

pub unsafe fn CSendToMenu_QueryContextMenu_body(this: *mut CSendToMenu, hMenu: HMENU, indexMenu: UINT, idCmdFirst: UINT, idCmdLast: UINT, uFlags: UINT) -> HRESULT {
    if (uFlags & (CMF_NOVERBS | CMF_VERBSONLY)) != 0 {
        return MAKE_HRESULT(SEVERITY_SUCCESS, 0, idCmdFirst);
    }
    let hSubMenu = CreateMenu();
    if hSubMenu.is_null() {
        return E_FAIL;
    }
    let cItems = CSendToMenu_InsertSendToItems_body(this, hSubMenu, idCmdFirst, 0);
    let p_sz = IDS_SENDTO_MENU as usize as *mut u16;
    let mut mii = MENUITEMINFOW {
        cbSize: core::mem::size_of::<MENUITEMINFOW>() as u32,
        fMask: MIIM_TYPE | MIIM_ID | MIIM_STATE | MIIM_SUBMENU,
        fType: MFT_STRING,
        fState: MFS_ENABLED,
        wID: (-(1 as i32)) as u32,
        hSubMenu,
        hbmpChecked: core::ptr::null_mut(),
        hbmpUnchecked: core::ptr::null_mut(),
        dwItemData: 0,
        dwTypeData: p_sz,
        cch: wcslen(p_sz) as u32,
        hbmpItem: core::ptr::null_mut(),
    };
    if InsertMenuItemW(hMenu, indexMenu, TRUE, &mut mii) == 0 {
        return E_FAIL;
    }
    let hOldSubMenu = (*this).m_hSubMenu;
    (*this).m_hSubMenu = hSubMenu;
    DestroyMenu(hOldSubMenu);
    MAKE_HRESULT(SEVERITY_SUCCESS, 0, idCmdFirst + cItems)
}

pub unsafe fn CSendToMenu_InvokeCommand_body(this: *mut CSendToMenu, lpici: LPCMINVOKECOMMANDINFO) -> HRESULT {
    let mut hr: HRESULT = E_FAIL;
    let idCmd: UINT = LOWORD!((*lpici).lpVerb);
    let pItem: *mut SENDTO_ITEM = CSendToMenu_FindItemFromIdOffset_body(this, idCmd);
    if !pItem.is_null() {
        hr = CSendToMenu_DoSendToItem_body(this, pItem, lpici);
    }
    hr
}

pub unsafe fn CSendToMenu_GetCommandString_body(this: *mut CSendToMenu, idCmd: UINT_PTR, uType: UINT, pwReserved: *mut UINT, pszName: LPSTR, cchMax: UINT) -> HRESULT { E_NOTIMPL }

pub unsafe fn CSendToMenu_HandleMenuMsg_body(this: *mut CSendToMenu, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> HRESULT { S_OK }

pub unsafe fn CSendToMenu_HandleMenuMsg2_body(this: *mut CSendToMenu, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, plResult: *mut LRESULT) -> HRESULT {
    let cxSmall = GetSystemMetrics((SM_CXSMICON) as i32);
    let cySmall = GetSystemMetrics((SM_CYSMICON) as i32);

    match uMsg {
        WM_MEASUREITEM => {
            let lpmis: *mut MEASUREITEMSTRUCT = lParam as *mut MEASUREITEMSTRUCT;
            if !lpmis.is_null() && (*lpmis).CtlType == ODT_MENU {
                let cxMenuCheck = GetSystemMetrics((SM_CXMENUCHECK) as i32);
                if (*lpmis).itemWidth < cxMenuCheck as UINT {
                    (*lpmis).itemWidth = cxMenuCheck as UINT as UINT;
                }
                if (*lpmis).itemHeight < cySmall as UINT {
                    (*lpmis).itemHeight = cySmall as UINT as UINT;
                }
                if !plResult.is_null() {
                    *plResult = TRUE as LRESULT as LRESULT;
                }
            }
        }
        WM_DRAWITEM => {
            let lpdis: *mut DRAWITEMSTRUCT = lParam as *mut DRAWITEMSTRUCT;
            if !lpdis.is_null() && (*lpdis).CtlType == ODT_MENU {
                let pItem = (*lpdis).itemData as *mut SENDTO_ITEM;
                let hIcon = if !pItem.is_null() { (*pItem).hIcon } else { core::ptr::null_mut() };
                if !hIcon.is_null() {
                    let rcItem: &RECT = &(*lpdis).rcItem;
                    let x = 4;
                    let y = (*lpdis).rcItem.top;
                    let y = y + (rcItem.bottom - rcItem.top - cySmall) / 2;
                    DrawIconEx(
                        (*lpdis).hDC,
                        x,
                        y,
                        (hIcon).cast(),
                        cxSmall,
                        cySmall,
                        0,
                        core::ptr::null_mut(),
                        DI_NORMAL,
                    );
                    if !plResult.is_null() {
                        *plResult = TRUE as LRESULT as LRESULT;
                    }
                }
            }
        }
        _ => {}
    }

    return S_OK;
}

pub unsafe fn CSendToMenu_Initialize_body(this: *mut CSendToMenu, pidlFolder: PCIDLIST_ABSOLUTE, pdtobj: *mut IDataObject, hkeyProgID: HKEY) -> HRESULT {
    CSendToMenu_cpp_com_assign_m_pDataObject(this, pdtobj);
    S_OK
}
