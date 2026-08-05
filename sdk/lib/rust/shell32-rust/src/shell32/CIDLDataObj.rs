// rustconv-assembly: class:CIDLDataObj
use crate::prelude::*;
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

// Deterministic from shldisp.h
#[repr(C)]
pub struct IAsyncOperation {
    pub lpVtbl: *const IAsyncOperationVtbl,
}
#[repr(C)]
pub struct IAsyncOperationVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IAsyncOperation, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IAsyncOperation) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IAsyncOperation) -> ULONG,
    pub SetAsyncMode: unsafe extern "system" fn(This: *mut IAsyncOperation, fDoOpAsync: BOOL) -> HRESULT,
    pub GetAsyncMode: unsafe extern "system" fn(This: *mut IAsyncOperation, pfIsOpAsync: *mut BOOL) -> HRESULT,
    pub StartOperation: unsafe extern "system" fn(This: *mut IAsyncOperation, pbcReserved: *mut IBindCtx) -> HRESULT,
    pub InOperation: unsafe extern "system" fn(This: *mut IAsyncOperation, pfInAsyncOp: *mut BOOL) -> HRESULT,
    pub EndOperation: unsafe extern "system" fn(This: *mut IAsyncOperation, hResult: HRESULT, pbcReserved: *mut IBindCtx, dwEffects: DWORD) -> HRESULT,
}
extern "C" {
    pub static IID_IAsyncOperation: IID;
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

// Deterministic COM scaffold for CIDLDataObj; method bodies are generated separately.
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
pub struct CIDLDataObj {
    pub IDataObject_iface: IDataObject,
    pub IAsyncOperation_iface: IAsyncOperation,
    pub ref_count: core::sync::atomic::AtomicI32,
    pub m_Formats: CSimpleArray<FORMATETC>,
    pub m_Storage: CSimpleArray<STGMEDIUM>,
    pub m_cfShellIDList: UINT,
    pub m_doasync: BOOL,
    pub m_FailGetHDrop: bool,
}

unsafe extern "system" fn CIDLDataObj_IDataObject_QueryInterface(This: *mut IDataObject, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CIDLDataObj, IDataObject_iface)) as *mut CIDLDataObj;
    CIDLDataObj_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CIDLDataObj_IDataObject_AddRef(This: *mut IDataObject) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CIDLDataObj, IDataObject_iface)) as *mut CIDLDataObj;
    CIDLDataObj_AddRef(this)
}
unsafe extern "system" fn CIDLDataObj_IDataObject_Release(This: *mut IDataObject) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CIDLDataObj, IDataObject_iface)) as *mut CIDLDataObj;
    CIDLDataObj_Release(this)
}
unsafe extern "system" fn CIDLDataObj_IDataObject_GetData(This: *mut IDataObject, pformatetcIn: *mut FORMATETC, pmedium: *mut STGMEDIUM) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CIDLDataObj, IDataObject_iface)) as *mut CIDLDataObj;
    CIDLDataObj_GetData_body(this, pformatetcIn, pmedium)
}
unsafe extern "system" fn CIDLDataObj_IDataObject_GetDataHere(This: *mut IDataObject, pformatetc: *mut FORMATETC, pmedium: *mut STGMEDIUM) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CIDLDataObj, IDataObject_iface)) as *mut CIDLDataObj;
    CIDLDataObj_GetDataHere_body(this, pformatetc, pmedium)
}
unsafe extern "system" fn CIDLDataObj_IDataObject_QueryGetData(This: *mut IDataObject, pformatetc: *mut FORMATETC) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CIDLDataObj, IDataObject_iface)) as *mut CIDLDataObj;
    CIDLDataObj_QueryGetData_body(this, pformatetc)
}
unsafe extern "system" fn CIDLDataObj_IDataObject_GetCanonicalFormatEtc(This: *mut IDataObject, pformatectIn: *mut FORMATETC, pformatetcOut: *mut FORMATETC) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CIDLDataObj, IDataObject_iface)) as *mut CIDLDataObj;
    CIDLDataObj_GetCanonicalFormatEtc_body(this, pformatectIn, pformatetcOut)
}
unsafe extern "system" fn CIDLDataObj_IDataObject_SetData(This: *mut IDataObject, pformatetc: *mut FORMATETC, pmedium: *mut STGMEDIUM, fRelease: BOOL) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CIDLDataObj, IDataObject_iface)) as *mut CIDLDataObj;
    CIDLDataObj_SetData_body(this, pformatetc, pmedium, fRelease)
}
unsafe extern "system" fn CIDLDataObj_IDataObject_EnumFormatEtc(This: *mut IDataObject, dwDirection: DWORD, ppenumFormatEtc: *mut *mut IEnumFORMATETC) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CIDLDataObj, IDataObject_iface)) as *mut CIDLDataObj;
    CIDLDataObj_EnumFormatEtc_body(this, dwDirection, ppenumFormatEtc)
}
unsafe extern "system" fn CIDLDataObj_IDataObject_DAdvise(This: *mut IDataObject, pformatetc: *mut FORMATETC, advf: DWORD, pAdvSink: *mut IAdviseSink, pdwConnection: *mut DWORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CIDLDataObj, IDataObject_iface)) as *mut CIDLDataObj;
    CIDLDataObj_DAdvise_body(this, pformatetc, advf, pAdvSink, pdwConnection)
}
unsafe extern "system" fn CIDLDataObj_IDataObject_DUnadvise(This: *mut IDataObject, dwConnection: DWORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CIDLDataObj, IDataObject_iface)) as *mut CIDLDataObj;
    CIDLDataObj_DUnadvise_body(this, dwConnection)
}
unsafe extern "system" fn CIDLDataObj_IDataObject_EnumDAdvise(This: *mut IDataObject, ppenumAdvise: *mut *mut IEnumSTATDATA) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CIDLDataObj, IDataObject_iface)) as *mut CIDLDataObj;
    CIDLDataObj_EnumDAdvise_body(this, ppenumAdvise)
}

pub static CIDLDATAOBJ_IDATAOBJECT_VTBL: IDataObjectVtbl = IDataObjectVtbl {
    QueryInterface: CIDLDataObj_IDataObject_QueryInterface,
    AddRef: CIDLDataObj_IDataObject_AddRef,
    Release: CIDLDataObj_IDataObject_Release,
    GetData: CIDLDataObj_IDataObject_GetData,
    GetDataHere: CIDLDataObj_IDataObject_GetDataHere,
    QueryGetData: CIDLDataObj_IDataObject_QueryGetData,
    GetCanonicalFormatEtc: CIDLDataObj_IDataObject_GetCanonicalFormatEtc,
    SetData: CIDLDataObj_IDataObject_SetData,
    EnumFormatEtc: CIDLDataObj_IDataObject_EnumFormatEtc,
    DAdvise: CIDLDataObj_IDataObject_DAdvise,
    DUnadvise: CIDLDataObj_IDataObject_DUnadvise,
    EnumDAdvise: CIDLDataObj_IDataObject_EnumDAdvise,
};

unsafe extern "system" fn CIDLDataObj_IAsyncOperation_QueryInterface(This: *mut IAsyncOperation, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CIDLDataObj, IAsyncOperation_iface)) as *mut CIDLDataObj;
    CIDLDataObj_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CIDLDataObj_IAsyncOperation_AddRef(This: *mut IAsyncOperation) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CIDLDataObj, IAsyncOperation_iface)) as *mut CIDLDataObj;
    CIDLDataObj_AddRef(this)
}
unsafe extern "system" fn CIDLDataObj_IAsyncOperation_Release(This: *mut IAsyncOperation) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CIDLDataObj, IAsyncOperation_iface)) as *mut CIDLDataObj;
    CIDLDataObj_Release(this)
}
unsafe extern "system" fn CIDLDataObj_IAsyncOperation_SetAsyncMode(This: *mut IAsyncOperation, fDoOpAsync: BOOL) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CIDLDataObj, IAsyncOperation_iface)) as *mut CIDLDataObj;
    CIDLDataObj_SetAsyncMode_body(this, fDoOpAsync)
}
unsafe extern "system" fn CIDLDataObj_IAsyncOperation_GetAsyncMode(This: *mut IAsyncOperation, pfIsOpAsync: *mut BOOL) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CIDLDataObj, IAsyncOperation_iface)) as *mut CIDLDataObj;
    CIDLDataObj_GetAsyncMode_body(this, pfIsOpAsync)
}
unsafe extern "system" fn CIDLDataObj_IAsyncOperation_StartOperation(This: *mut IAsyncOperation, pbcReserved: *mut IBindCtx) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CIDLDataObj, IAsyncOperation_iface)) as *mut CIDLDataObj;
    CIDLDataObj_StartOperation_body(this, pbcReserved)
}
unsafe extern "system" fn CIDLDataObj_IAsyncOperation_InOperation(This: *mut IAsyncOperation, pfInAsyncOp: *mut BOOL) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CIDLDataObj, IAsyncOperation_iface)) as *mut CIDLDataObj;
    CIDLDataObj_InOperation_body(this, pfInAsyncOp)
}
unsafe extern "system" fn CIDLDataObj_IAsyncOperation_EndOperation(This: *mut IAsyncOperation, hResult: HRESULT, pbcReserved: *mut IBindCtx, dwEffects: DWORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CIDLDataObj, IAsyncOperation_iface)) as *mut CIDLDataObj;
    CIDLDataObj_EndOperation_body(this, hResult, pbcReserved, dwEffects)
}

pub static CIDLDATAOBJ_IASYNCOPERATION_VTBL: IAsyncOperationVtbl = IAsyncOperationVtbl {
    QueryInterface: CIDLDataObj_IAsyncOperation_QueryInterface,
    AddRef: CIDLDataObj_IAsyncOperation_AddRef,
    Release: CIDLDataObj_IAsyncOperation_Release,
    SetAsyncMode: CIDLDataObj_IAsyncOperation_SetAsyncMode,
    GetAsyncMode: CIDLDataObj_IAsyncOperation_GetAsyncMode,
    StartOperation: CIDLDataObj_IAsyncOperation_StartOperation,
    InOperation: CIDLDataObj_IAsyncOperation_InOperation,
    EndOperation: CIDLDataObj_IAsyncOperation_EndOperation,
};

unsafe fn CIDLDataObj_QueryInterface(
    this: *mut CIDLDataObj, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IUnknown)) || cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IDataObject)) {
        *ppv = core::ptr::addr_of_mut!((*this).IDataObject_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IAsyncOperation)) {
        *ppv = core::ptr::addr_of_mut!((*this).IAsyncOperation_iface) as *mut core::ffi::c_void;
    }
    else { return E_NOINTERFACE; }
    CIDLDataObj_AddRef(this);
    S_OK
}

unsafe fn CIDLDataObj_AddRef(this: *mut CIDLDataObj) -> ULONG {
    (*this).ref_count.fetch_add(1, core::sync::atomic::Ordering::SeqCst) as ULONG + 1
}

unsafe fn CIDLDataObj_Release(this: *mut CIDLDataObj) -> ULONG {
    let refs = (*this).ref_count.fetch_sub(1, core::sync::atomic::Ordering::SeqCst) - 1;
    if refs == 0 {
        CIDLDataObj_dtor_body(this);
        core::ptr::drop_in_place(&mut (*this).m_Formats);
        core::ptr::drop_in_place(&mut (*this).m_Storage);
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
pub unsafe fn CIDLDataObj_CreateInstance(
    hwndOwner: HWND,
    pMyPidl: PCIDLIST_ABSOLUTE,
    apidlx: PCUIDLIST_RELATIVE_ARRAY,
    cidlx: UINT,
    bAddAdditionalFormats: BOOL,
    riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    let this = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<CIDLDataObj>()) as *mut CIDLDataObj;
    if this.is_null() { return E_OUTOFMEMORY; }
    core::ptr::write(this, CIDLDataObj {
        IDataObject_iface: IDataObject { lpVtbl: core::ptr::addr_of!(CIDLDATAOBJ_IDATAOBJECT_VTBL) },
        IAsyncOperation_iface: IAsyncOperation { lpVtbl: core::ptr::addr_of!(CIDLDATAOBJ_IASYNCOPERATION_VTBL) },
        ref_count: core::sync::atomic::AtomicI32::new(1),
        m_Formats: CSimpleArray::new(),
        m_Storage: CSimpleArray::new(),
        m_cfShellIDList: core::mem::zeroed(),
        m_doasync: core::mem::zeroed(),
        m_FailGetHDrop: core::mem::zeroed(),
    });
    Shell32RustObjectLock();
    CIDLDataObj_ctor_body(this);
    let mut hr = CIDLDataObj_Initialize_body(this, hwndOwner, pMyPidl, apidlx, cidlx, bAddAdditionalFormats);
    if hr >= 0 {
        hr = CIDLDataObj_QueryInterface(this, riid, ppv);
    }
    CIDLDataObj_Release(this);
    hr
}

// Deterministic C++ support closure selected from source identifiers.

pub const CF_HDROP: DWORD = 15;

pub const DATADIR_GET: DWORD = 1;

pub const DVASPECT_CONTENT: DWORD = 1;


pub const TYMED_HGLOBAL: DWORD = 1;

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

extern "C" {
    #[link_name = "memcpy"] fn cpp_com_memcpy(dst: *mut core::ffi::c_void, src: *const core::ffi::c_void, bytes: usize) -> *mut core::ffi::c_void;
}

#[inline]
unsafe fn memcpy<D, S, N: CppComSize>(dst: *mut D, src: *const S, bytes: N) -> *mut core::ffi::c_void {
    cpp_com_memcpy(dst.cast(), src.cast(), bytes.cpp_com_size())
}

use core::ffi::c_void;

// ---------------------------------------------------------------------------------------
// Substrate this TU takes from elsewhere in the crate. Checked for an existing Rust owner
// FIRST: all four clipboard renderers are already translated in `wine/clipboard.rs`, so a
// fresh `extern` block for them would have bound a C symbol that no longer exists.
// ---------------------------------------------------------------------------------------
use crate::wine::clipboard::{RenderFILENAMEA, RenderFILENAMEW, RenderHDROP, RenderSHELLIDLIST};

/// shlobj.h:2525 — `static inline`, so there is no symbol to link against.
#[inline]
unsafe fn ILIsEmpty(pidl: PCUIDLIST_RELATIVE) -> bool {
    !(!pidl.is_null() && (*pidl).mkid.cb != 0)
}

/// shlobj.h — the TCHAR-generic name. ReactOS builds shell32 UNICODE, so `CFSTR_SHELLIDLIST`
/// is the W form, and `shellutils.rs` already owns that literal. `CFSTR_FILENAMEA` and
/// `CFSTR_FILENAMEW` are DIFFERENT clipboard formats, both registered below, so neither is a
/// synonym for the other; the island only carries the A one as a byte string.
use crate::shellutils::CFSTR_SHELLIDLISTW as CFSTR_SHELLIDLIST_CHARS;
/// `CFSTR_FILENAMEW` = "FileNameW" (shlobj.h CF_FILENAMEW).
const CFSTR_FILENAMEW_CHARS: [u16; 10] = [
    b'F' as u16, b'i' as u16, b'l' as u16, b'e' as u16, b'N' as u16, b'a' as u16, b'm' as u16,
    b'e' as u16, b'W' as u16, 0,
];

// OLE HRESULTs (winerror.h). Not in the island: they arrive via ole2.h macros bindgen
// cannot emit.
const DV_E_CLIPFORMAT: HRESULT = 0x8004_006Au32 as HRESULT;
const DATA_S_SAMEFORMATETC: HRESULT = 0x0004_0130u32 as HRESULT;
const OLE_E_ADVISENOTSUPPORTED: HRESULT = 0x8004_0003u32 as HRESULT;

type LPFORMATETC = *mut FORMATETC;

pub unsafe fn CIDLDataObj_ctor_body(this: *mut CIDLDataObj) -> () {
    (*this).m_cfShellIDList = 0;
    (*this).m_doasync = FALSE;
    (*this).m_FailGetHDrop = false;
}

pub unsafe fn CIDLDataObj_dtor_body(this: *mut CIDLDataObj) -> () {
    let mut n = 0;
    while n < (*this).m_Storage.GetSize() {
        ReleaseStgMedium((*this).m_Storage.GetAt(n));
        n += 1;
    }
    (*this).m_Formats.RemoveAll();
    (*this).m_Storage.RemoveAll();
}

pub unsafe fn CIDLDataObj_Initialize_body(this: *mut CIDLDataObj, hwndOwner: HWND, pMyPidl: PCIDLIST_ABSOLUTE, apidlx: PCUIDLIST_RELATIVE_ARRAY, cidlx: UINT, bAddAdditionalFormats: BOOL) -> HRESULT {
    let hida = RenderSHELLIDLIST(pMyPidl as LPITEMIDLIST as *mut _, apidlx as *mut LPITEMIDLIST as *mut _, cidlx);
    if hida.is_null() {
        return E_OUTOFMEMORY;
    }

    (*this).m_cfShellIDList = RegisterClipboardFormatW(CFSTR_SHELLIDLIST_CHARS.as_ptr());

    let mut Format = FORMATETC {
        cfFormat: (*this).m_cfShellIDList as CLIPFORMAT,
        ptd: core::ptr::null_mut(),
        dwAspect: DVASPECT_CONTENT,
        lindex: -1,
        tymed: TYMED_HGLOBAL,
    };
    let mut medium = core::mem::zeroed::<STGMEDIUM>();
    medium.tymed = TYMED_HGLOBAL;
    medium.__bindgen_anon_1.hGlobal = hida;
    let mut hr = CIDLDataObj_SetData_body(this, &mut Format, &mut medium, TRUE);
    if !((hr) < 0) && bAddAdditionalFormats != 0 {
        let mut i = 0;
        while i < cidlx {
            if ILIsEmpty(*apidlx.offset(i as isize)) && ILIsEmpty(pMyPidl) {
                (*this).m_FailGetHDrop = true;
            }
            i += 1;
        }

        Format.cfFormat = CF_HDROP as CLIPFORMAT;
        medium.__bindgen_anon_1.hGlobal = RenderHDROP(pMyPidl as LPITEMIDLIST as *mut _, apidlx as *mut LPITEMIDLIST as *mut _, cidlx);
        hr = CIDLDataObj_SetData_body(this, &mut Format, &mut medium, TRUE);
        if ((hr) < 0) {
            return hr;
        }

        Format.cfFormat = RegisterClipboardFormatA(CFSTR_FILENAMEA.as_ptr() as LPCSTR) as CLIPFORMAT;
        medium.__bindgen_anon_1.hGlobal = RenderFILENAMEA(pMyPidl as LPITEMIDLIST as *mut _, apidlx as *mut LPITEMIDLIST as *mut _, cidlx);
        hr = CIDLDataObj_SetData_body(this, &mut Format, &mut medium, TRUE);
        if ((hr) < 0) {
            return hr;
        }

        Format.cfFormat = RegisterClipboardFormatW(CFSTR_FILENAMEW_CHARS.as_ptr()) as CLIPFORMAT;
        medium.__bindgen_anon_1.hGlobal = RenderFILENAMEW(pMyPidl as LPITEMIDLIST as *mut _, apidlx as *mut LPITEMIDLIST as *mut _, cidlx);
        hr = CIDLDataObj_SetData_body(this, &mut Format, &mut medium, TRUE);
        if ((hr) < 0) {
            return hr;
        }
    }

    hr
}

pub unsafe fn CIDLDataObj_GetData_body(this: *mut CIDLDataObj, pformatetcIn: *mut FORMATETC, pmedium: *mut STGMEDIUM) -> HRESULT {
    if TRACE_ON!(shell) {
        let mut szTemp: [i8; 256] = [0; 256];
        GetClipboardFormatNameA(
            (*pformatetcIn).cfFormat as UINT,
            szTemp.as_mut_ptr(),
            256,
        );
    }
    (*pmedium).__bindgen_anon_1.hGlobal = core::ptr::null_mut();
    (*pmedium).pUnkForRelease = core::ptr::null_mut();
    let mut n: i32 = 0;
    while n < (*this).m_Formats.GetSize() {
        let fmt: *const FORMATETC = (*this).m_Formats.GetAt(n);
        if (*fmt).cfFormat == (*pformatetcIn).cfFormat
            && (*fmt).dwAspect == (*pformatetcIn).dwAspect
            && (*fmt).tymed == (*pformatetcIn).tymed
        {
            if (*this).m_FailGetHDrop && (*fmt).cfFormat == CF_HDROP as CLIPFORMAT {
                return DV_E_CLIPFORMAT;
            }

            if (*pformatetcIn).tymed != TYMED_HGLOBAL {
                // UNIMPLEMENTED (ReactOS debug macro; no-op in a release build)
                return E_INVALIDARG;
            } else {
                *pmedium = *(*this).m_Storage.GetAt(n);
                return CIDLDataObj_QueryInterface(this, core::ptr::addr_of!(IID_IUnknown), core::ptr::addr_of_mut!((*pmedium).pUnkForRelease).cast());
            }
        }
        n += 1;
    }

    E_INVALIDARG
}

pub unsafe fn CIDLDataObj_GetDataHere_body(this: *mut CIDLDataObj, pformatetc: *mut FORMATETC, pmedium: *mut STGMEDIUM) -> HRESULT { E_NOTIMPL }

pub unsafe fn CIDLDataObj_QueryGetData_body(this: *mut CIDLDataObj, pformatetc: *mut FORMATETC) -> HRESULT {
    let mut n = 0;
    while n < (*this).m_Formats.GetSize() {
        let fmt: *mut FORMATETC = (*this).m_Formats.GetAt(n);
        if (*fmt).cfFormat == (*pformatetc).cfFormat &&
            (*fmt).dwAspect == (*pformatetc).dwAspect &&
            (*fmt).tymed == (*pformatetc).tymed
        {
            return S_OK;
        }
        n += 1;
    }

    S_FALSE
}

pub unsafe fn CIDLDataObj_GetCanonicalFormatEtc_body(this: *mut CIDLDataObj, pformatectIn: *mut FORMATETC, pformatetcOut: *mut FORMATETC) -> HRESULT { DATA_S_SAMEFORMATETC }

pub unsafe fn CIDLDataObj_SetData_body(this: *mut CIDLDataObj, pformatetc: *mut FORMATETC, pmedium: *mut STGMEDIUM, fRelease: BOOL) -> HRESULT {
    if fRelease == 0 {
        return E_INVALIDARG;
    }

    let mut n = 0;
    while n < (*this).m_Formats.GetSize() {
        let fmt: *const FORMATETC = (*this).m_Formats.GetAt(n);
        if (*fmt).cfFormat == (*pformatetc).cfFormat
            && (*fmt).dwAspect == (*pformatetc).dwAspect
            && (*fmt).tymed == (*pformatetc).tymed
        {
            ReleaseStgMedium((*this).m_Storage.GetAt(n));
            *(*this).m_Storage.GetAt(n) = *pmedium;
            return S_OK;
        }
        n += 1;
    }

    (*this).m_Formats.Add(*pformatetc);
    (*this).m_Storage.Add(*pmedium);

    S_OK
}

pub unsafe fn CIDLDataObj_EnumFormatEtc_body(this: *mut CIDLDataObj, dwDirection: DWORD, ppenumFormatEtc: *mut *mut IEnumFORMATETC) -> HRESULT {
    *ppenumFormatEtc = core::ptr::null_mut();

    if DATADIR_GET == dwDirection {
        return IEnumFORMATETC_Constructor(
            (*this).m_Formats.GetSize() as UINT,
            (*this).m_Formats.GetData(),
            ppenumFormatEtc.cast(),
        );
    }

    return E_NOTIMPL;
}

pub unsafe fn CIDLDataObj_DAdvise_body(this: *mut CIDLDataObj, pformatetc: *mut FORMATETC, advf: DWORD, pAdvSink: *mut IAdviseSink, pdwConnection: *mut DWORD) -> HRESULT { OLE_E_ADVISENOTSUPPORTED }

pub unsafe fn CIDLDataObj_DUnadvise_body(this: *mut CIDLDataObj, dwConnection: DWORD) -> HRESULT { OLE_E_ADVISENOTSUPPORTED }

pub unsafe fn CIDLDataObj_EnumDAdvise_body(this: *mut CIDLDataObj, ppenumAdvise: *mut *mut IEnumSTATDATA) -> HRESULT { OLE_E_ADVISENOTSUPPORTED }

pub unsafe fn CIDLDataObj_GetAsyncMode_body(this: *mut CIDLDataObj, pfIsOpAsync: *mut BOOL) -> HRESULT {
    *pfIsOpAsync = (*this).m_doasync;
    return S_OK;
}

pub unsafe fn CIDLDataObj_InOperation_body(this: *mut CIDLDataObj, pfInAsyncOp: *mut BOOL) -> HRESULT { E_NOTIMPL }

pub unsafe fn CIDLDataObj_SetAsyncMode_body(this: *mut CIDLDataObj, fDoOpAsync: BOOL) -> HRESULT {
    (*this).m_doasync = fDoOpAsync;
    S_OK
}

pub unsafe fn CIDLDataObj_StartOperation_body(this: *mut CIDLDataObj, pbcReserved: *mut IBindCtx) -> HRESULT { E_NOTIMPL }

pub unsafe fn CIDLDataObj_EndOperation_body(this: *mut CIDLDataObj, hResult: HRESULT, pbcReserved: *mut IBindCtx, dwEffects: DWORD) -> HRESULT { E_NOTIMPL }


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     shell32
//  * LICENSE:     LGPL-2.1-or-later (https://spdx.org/licenses/LGPL-2.1-or-later)
//  * PURPOSE:     IEnumFORMATETC, IDataObject implementation
//  * COPYRIGHT:   Copyright 1998, 1999 <juergen.schmied@metronet.de>
//  *              Copyright 2019 Mark Jansen (mark.jansen@reactos.org)
//  */
//
// #include "precomp.h"
//
// WINE_DEFAULT_DEBUG_CHANNEL(shell);
//
// /***********************************************************************
// *   IEnumFORMATETC implementation
// */
//
// class IEnumFORMATETCImpl :
//     public CComObjectRootEx<CComMultiThreadModelNoCS>,
//     public IEnumFORMATETC
// {
// private:
//     UINT                        posFmt;
//     UINT                        countFmt;
//     LPFORMATETC                    pFmt;
// public:
//     IEnumFORMATETCImpl();
//     ~IEnumFORMATETCImpl();
//     HRESULT WINAPI Initialize(UINT cfmt, const FORMATETC afmt[]);
//
//     // *****************
//     STDMETHOD(Next)(ULONG celt, FORMATETC *rgelt, ULONG *pceltFethed) override;
//     STDMETHOD(Skip)(ULONG celt) override;
//     STDMETHOD(Reset)() override;
//     STDMETHOD(Clone)(LPENUMFORMATETC* ppenum) override;
//
// BEGIN_COM_MAP(IEnumFORMATETCImpl)
//     COM_INTERFACE_ENTRY_IID(IID_IEnumFORMATETC, IEnumFORMATETC)
// END_COM_MAP()
// };
//
// HRESULT WINAPI CIDLDataObj::Initialize(HWND hwndOwner, PCIDLIST_ABSOLUTE pMyPidl, PCUIDLIST_RELATIVE_ARRAY apidlx, UINT cidlx, BOOL bAddAdditionalFormats)
// {
//     HGLOBAL hida = RenderSHELLIDLIST((LPITEMIDLIST)pMyPidl, (LPITEMIDLIST*)apidlx, cidlx);
//     if (!hida)
//     {
//         ERR("Failed to render " CFSTR_SHELLIDLISTA "\n");
//         return E_OUTOFMEMORY;
//     }
//
//     m_cfShellIDList = RegisterClipboardFormatW(CFSTR_SHELLIDLIST_CHARS.as_ptr());
//
//     FORMATETC Format = { (CLIPFORMAT)m_cfShellIDList, NULL, DVASPECT_CONTENT, -1, TYMED_HGLOBAL };
//     STGMEDIUM medium = {0};
//     medium.tymed = TYMED_HGLOBAL;
//     medium.__bindgen_anon_1.hGlobal = hida;
//     HRESULT hr = SetData(&Format, &medium, TRUE);
//     if (!FAILED_UNEXPECTEDLY(hr) && bAddAdditionalFormats)
//     {
//         /* The Windows default shell IDataObject::GetData fails with DV_E_CLIPFORMAT if the desktop is present.
//          * Windows does return HDROP in EnumFormatEtc and does not fail until GetData is called.
//          * Failing GetData causes 7-Zip 23.01 to not add its menu to the desktop folder. */
//         for (UINT i = 0; i < cidlx; ++i)
//         {
//             if (ILIsEmpty(apidlx[i]) && ILIsEmpty(pMyPidl))
//                 m_FailGetHDrop = true;
//         }
//
//         Format.cfFormat = CF_HDROP as CLIPFORMAT;
//         medium.__bindgen_anon_1.hGlobal = RenderHDROP((LPITEMIDLIST)pMyPidl, (LPITEMIDLIST*)apidlx, cidlx);
//         hr = SetData(&Format, &medium, TRUE);
//         if (FAILED_UNEXPECTEDLY(hr))
//             return hr;
//
//         Format.cfFormat = RegisterClipboardFormatA(CFSTR_FILENAMEA.as_ptr() as LPCSTR) as CLIPFORMAT;
//         medium.__bindgen_anon_1.hGlobal = RenderFILENAMEA((LPITEMIDLIST)pMyPidl, (LPITEMIDLIST*)apidlx, cidlx);
//         hr = SetData(&Format, &medium, TRUE);
//         if (FAILED_UNEXPECTEDLY(hr))
//             return hr;
//
//         Format.cfFormat = RegisterClipboardFormatW(CFSTR_FILENAMEW_CHARS.as_ptr()) as CLIPFORMAT;
//         medium.__bindgen_anon_1.hGlobal = RenderFILENAMEW((LPITEMIDLIST)pMyPidl, (LPITEMIDLIST*)apidlx, cidlx);
//         hr = SetData(&Format, &medium, TRUE);
//         if (FAILED_UNEXPECTEDLY(hr))
//             return hr;
//     }
//
//     return hr;
// }
// HRESULT WINAPI CIDLDataObj::GetData(LPFORMATETC pformatetcIn, STGMEDIUM *pmedium)
// {
//     if (TRACE_ON(shell))
//     {
//         char szTemp[256] = {0};
//         GetClipboardFormatNameA (pformatetcIn->cfFormat, szTemp, 256);
//         TRACE("(%p)->(%p %p format=%s)\n", this, pformatetcIn, pmedium, szTemp);
//     }
//     pmedium->hGlobal = NULL;
//     pmedium->pUnkForRelease = NULL;
//     for (int n = 0; n < m_Formats.GetSize(); ++n)
//     {
//         const FORMATETC& fmt = m_Formats[n];
//         if (fmt.cfFormat == pformatetcIn->cfFormat &&
//             fmt.dwAspect == pformatetcIn->dwAspect &&
//             fmt.tymed == pformatetcIn->tymed)
//         {
//             if (m_FailGetHDrop && fmt.cfFormat == CF_HDROP)
//                 return DV_E_CLIPFORMAT;
//
//             if (pformatetcIn->tymed != TYMED_HGLOBAL)
//             {
//                 UNIMPLEMENTED;
//                 return E_INVALIDARG;
//             }
//             else
//             {
//                 *pmedium = m_Storage[n];
//                 return QueryInterface(IID_PPV_ARG(IUnknown, &pmedium->pUnkForRelease));
//             }
//         }
//     }
//
//     return E_INVALIDARG;
// }
// HRESULT WINAPI CIDLDataObj::GetDataHere(LPFORMATETC pformatetc, STGMEDIUM *pmedium)
// {
//     FIXME("(%p)->()\n", this);
//     return E_NOTIMPL;
// }
// HRESULT WINAPI CIDLDataObj::QueryGetData(LPFORMATETC pformatetc)
// {
//     TRACE("(%p)->(fmt=0x%08x tym=0x%08x)\n", this, pformatetc->cfFormat, pformatetc->tymed);
//
//     for (int n = 0; n < m_Formats.GetSize(); ++n)
//     {
//         const FORMATETC& fmt = m_Formats[n];
//         if (fmt.cfFormat == pformatetc->cfFormat &&
//             fmt.dwAspect == pformatetc->dwAspect &&
//             fmt.tymed == pformatetc->tymed)
//         {
//             return S_OK;
//         }
//     }
//
//     return S_FALSE;
// }
// HRESULT WINAPI CIDLDataObj::GetCanonicalFormatEtc(LPFORMATETC pformatectIn, LPFORMATETC pformatetcOut)
// {
//     //FIXME("(%p)->()\n", this);
//     return DATA_S_SAMEFORMATETC;
// }
// HRESULT WINAPI CIDLDataObj::SetData(LPFORMATETC pformatetc, STGMEDIUM *pmedium, BOOL fRelease)
// {
//     if (!fRelease)
//         return E_INVALIDARG;
//
//     for (int n = 0; n < m_Formats.GetSize(); ++n)
//     {
//         const FORMATETC& fmt = m_Formats[n];
//         if (fmt.cfFormat == pformatetc->cfFormat &&
//             fmt.dwAspect == pformatetc->dwAspect &&
//             fmt.tymed == pformatetc->tymed)
//         {
//             ReleaseStgMedium(&m_Storage[n]);
//             m_Storage[n] = *pmedium;
//             return S_OK;
//         }
//     }
//
//     m_Formats.Add(*pformatetc);
//     m_Storage.Add(*pmedium);
//
//     return S_OK;
// }
// HRESULT WINAPI CIDLDataObj::EnumFormatEtc(DWORD dwDirection, IEnumFORMATETC **ppenumFormatEtc)
// {
//     TRACE("(%p)->()\n", this);
//     *ppenumFormatEtc = NULL;
//
//     /* only get data */
//     if (DATADIR_GET == dwDirection)
//     {
//         return IEnumFORMATETC_Constructor(m_Formats.GetSize(), m_Formats.GetData(), ppenumFormatEtc);
//     }
//
//     return E_NOTIMPL;
// }
// HRESULT WINAPI CIDLDataObj::DAdvise(FORMATETC *pformatetc, DWORD advf, IAdviseSink *pAdvSink, DWORD *pdwConnection)
// {
//     return OLE_E_ADVISENOTSUPPORTED;
// }
// HRESULT WINAPI CIDLDataObj::DUnadvise(DWORD dwConnection)
// {
//     return OLE_E_ADVISENOTSUPPORTED;
// }
// HRESULT WINAPI CIDLDataObj::EnumDAdvise(IEnumSTATDATA **ppenumAdvise)
// {
//     return OLE_E_ADVISENOTSUPPORTED;
// }
// HRESULT WINAPI CIDLDataObj::GetAsyncMode(BOOL *pfIsOpAsync)
// {
//     TRACE("(%p)->()\n", this);
//     *pfIsOpAsync = m_doasync;
//     return S_OK;
// }
// HRESULT WINAPI CIDLDataObj::InOperation(BOOL *pfInAsyncOp)
// {
//     FIXME("(%p)->()\n", this);
//     return E_NOTIMPL;
// }
// HRESULT WINAPI CIDLDataObj::SetAsyncMode(BOOL fDoOpAsync)
// {
//     TRACE("(%p)->()\n", this);
//     m_doasync = fDoOpAsync;
//     return S_OK;
// }
// HRESULT WINAPI CIDLDataObj::StartOperation(IBindCtx *pbcReserved)
// {
//     TRACE("(%p)->()\n", this);
//     return E_NOTIMPL;
// }
// HRESULT WINAPI CIDLDataObj::EndOperation(HRESULT hResult, IBindCtx *pbcReserved, DWORD dwEffects)
// {
//     TRACE("(%p)->()\n", this);
//     return E_NOTIMPL;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
