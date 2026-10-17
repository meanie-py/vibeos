// rustconv-assembly: class:CRecyclerDropTarget
// Deterministic from oleidl.h
use crate::prelude::*;
// One declaration per external symbol: island.rs.
use crate::island::SHCreateThread;
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

// Deterministic COM scaffold for CRecyclerDropTarget; method bodies are generated separately.
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
pub struct CRecyclerDropTarget {
    pub IDropTarget_iface: IDropTarget,
    pub ref_count: core::sync::atomic::AtomicI32,
    pub fAcceptFmt: BOOL,
    pub cfShellIDList: UINT,
}

unsafe extern "system" fn CRecyclerDropTarget_IDropTarget_QueryInterface(This: *mut IDropTarget, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecyclerDropTarget, IDropTarget_iface)) as *mut CRecyclerDropTarget;
    CRecyclerDropTarget_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CRecyclerDropTarget_IDropTarget_AddRef(This: *mut IDropTarget) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecyclerDropTarget, IDropTarget_iface)) as *mut CRecyclerDropTarget;
    CRecyclerDropTarget_AddRef(this)
}
unsafe extern "system" fn CRecyclerDropTarget_IDropTarget_Release(This: *mut IDropTarget) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecyclerDropTarget, IDropTarget_iface)) as *mut CRecyclerDropTarget;
    CRecyclerDropTarget_Release(this)
}
unsafe extern "system" fn CRecyclerDropTarget_IDropTarget_DragEnter(This: *mut IDropTarget, pDataObj: *mut IDataObject, grfKeyState: DWORD, pt: POINTL, pdwEffect: *mut DWORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecyclerDropTarget, IDropTarget_iface)) as *mut CRecyclerDropTarget;
    CRecyclerDropTarget_DragEnter_body(this, pDataObj, grfKeyState, pt, pdwEffect)
}
unsafe extern "system" fn CRecyclerDropTarget_IDropTarget_DragOver(This: *mut IDropTarget, grfKeyState: DWORD, pt: POINTL, pdwEffect: *mut DWORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecyclerDropTarget, IDropTarget_iface)) as *mut CRecyclerDropTarget;
    CRecyclerDropTarget_DragOver_body(this, grfKeyState, pt, pdwEffect)
}
unsafe extern "system" fn CRecyclerDropTarget_IDropTarget_DragLeave(This: *mut IDropTarget) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecyclerDropTarget, IDropTarget_iface)) as *mut CRecyclerDropTarget;
    CRecyclerDropTarget_DragLeave_body(this)
}
unsafe extern "system" fn CRecyclerDropTarget_IDropTarget_Drop(This: *mut IDropTarget, pDataObj: *mut IDataObject, grfKeyState: DWORD, pt: POINTL, pdwEffect: *mut DWORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecyclerDropTarget, IDropTarget_iface)) as *mut CRecyclerDropTarget;
    CRecyclerDropTarget_Drop_body(this, pDataObj, grfKeyState, pt, pdwEffect)
}

pub static CRECYCLERDROPTARGET_IDROPTARGET_VTBL: IDropTargetVtbl = IDropTargetVtbl {
    QueryInterface: CRecyclerDropTarget_IDropTarget_QueryInterface,
    AddRef: CRecyclerDropTarget_IDropTarget_AddRef,
    Release: CRecyclerDropTarget_IDropTarget_Release,
    DragEnter: CRecyclerDropTarget_IDropTarget_DragEnter,
    DragOver: CRecyclerDropTarget_IDropTarget_DragOver,
    DragLeave: CRecyclerDropTarget_IDropTarget_DragLeave,
    Drop: CRecyclerDropTarget_IDropTarget_Drop,
};

unsafe fn CRecyclerDropTarget_QueryInterface(
    this: *mut CRecyclerDropTarget, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IUnknown)) || cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IDropTarget)) {
        *ppv = core::ptr::addr_of_mut!((*this).IDropTarget_iface) as *mut core::ffi::c_void;
    }
    else { return E_NOINTERFACE; }
    CRecyclerDropTarget_AddRef(this);
    S_OK
}

unsafe fn CRecyclerDropTarget_AddRef(this: *mut CRecyclerDropTarget) -> ULONG {
    (*this).ref_count.fetch_add(1, core::sync::atomic::Ordering::SeqCst) as ULONG + 1
}

unsafe fn CRecyclerDropTarget_Release(this: *mut CRecyclerDropTarget) -> ULONG {
    let refs = (*this).ref_count.fetch_sub(1, core::sync::atomic::Ordering::SeqCst) - 1;
    if refs == 0 {
        CRecyclerDropTarget_dtor_body(this);
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
#[no_mangle]
pub unsafe extern "C" fn CRecyclerDropTarget_CreateInstance(
    outer: *mut IUnknown,
    riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if !outer.is_null() { return CLASS_E_NOAGGREGATION; }
    let this = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<CRecyclerDropTarget>()) as *mut CRecyclerDropTarget;
    if this.is_null() { return E_OUTOFMEMORY; }
    core::ptr::write(this, CRecyclerDropTarget {
        IDropTarget_iface: IDropTarget { lpVtbl: core::ptr::addr_of!(CRECYCLERDROPTARGET_IDROPTARGET_VTBL) },
        ref_count: core::sync::atomic::AtomicI32::new(1),
        fAcceptFmt: core::mem::zeroed(),
        cfShellIDList: core::mem::zeroed(),
    });
    Shell32RustObjectLock();
    CRecyclerDropTarget_ctor_body(this);
    let hr = CRecyclerDropTarget_QueryInterface(this, riid, ppv);
    CRecyclerDropTarget_Release(this);
    hr
}

// Deterministic C++ support closure selected from source identifiers.

#[repr(C)]
pub struct DeleteThreadData {
    pub s: *mut IStream,
    pub fMask: DWORD,
}

pub const CF_HDROP: DWORD = 15;

pub const CMIC_MASK_SHIFT_DOWN: DWORD = 0x10000000;

pub const FOF_ALLOWUNDO: DWORD = 0x0040;

pub const FO_DELETE: DWORD = 3;

pub const MK_SHIFT: DWORD = 4;

pub const TYMED_HGLOBAL: DWORD = 1;

#[repr(C)]
pub struct BIND_OPTS {
    pub cbStruct: DWORD,
    pub grfFlags: DWORD,
    pub grfMode: DWORD,
    pub dwTickCountDeadline: DWORD,
}

pub use crate::island::DROPFILES;
pub type LPDROPFILES = *mut DROPFILES;

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

pub const DROPEFFECT_MOVE: DWORD = 2;

extern "system" {
    fn GlobalLock(memory: HGLOBAL) -> LPVOID;
    fn GlobalUnlock(memory: HGLOBAL) -> BOOL;
}

extern "C" {
    #[link_name = "memcpy"] fn cpp_com_memcpy(dst: *mut core::ffi::c_void, src: *const core::ffi::c_void, bytes: usize) -> *mut core::ffi::c_void;
}

#[inline]
unsafe fn memcpy<D, S, N: CppComSize>(dst: *mut D, src: *const S, bytes: N) -> *mut core::ffi::c_void {
    cpp_com_memcpy(dst.cast(), src.cast(), bytes.cpp_com_size())
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

use core::ffi::c_void;

// ---- integration substrate (hand-added; the assembly references these but declares none) ----
use crate::shell32::shlfileop::SHFileOperationW;
use crate::shellutils::CFSTR_SHELLIDLISTW;

// shell32_main.h:144 — a #define, so it is in no header the island reads.
#[inline]
unsafe fn InitFormatEtc(fe: *mut FORMATETC, cf: UINT, med: DWORD) {
    (*fe).cfFormat = cf as CLIPFORMAT;
    (*fe).dwAspect = DVASPECT_CONTENT;
    (*fe).ptd = core::ptr::null_mut();
    (*fe).tymed = med;
    (*fe).lindex = -1;
}

// Redeclared rather than taken from the island, which spells the thread proc as bindgen's
// per-target concrete ABI (`extern "stdcall"` on i386). `extern "system"` IS that same calling
// convention everywhere we build, but rustc keeps the two SPELLINGS as distinct fn types, so an
// `extern "system"` fn item will not coerce into the island's slot. Same shape as
// droptargets/CFSDropTarget.rs and dialogs/filedefext.rs.
extern "system" {
}

// ONE glob only. The scaffold header already emits `use crate::prelude::*` at the top of this
// file; adding the shellcom/types/win32 set beside it makes every shared name E0659-ambiguous
// (339 errors here), which is exactly what prelude.rs's own header warns about. The prelude
// resolves strictly more names, so it is the one kept and per-type ownership is stated
// explicitly instead — same resolution as droptargets/CFSDropTarget.rs.

pub unsafe fn CRecyclerDropTarget_dtor_body(this: *mut CRecyclerDropTarget) -> () {}

pub unsafe fn CRecyclerDropTarget__DoDeleteDataObject_body(this: *mut CRecyclerDropTarget, pda: *mut IDataObject, fMask: DWORD) -> HRESULT {
    let mut hr: HRESULT;
    let mut medium: STGMEDIUM = core::mem::zeroed();
    // The C leaves this uninitialized because InitFormatEtc writes every field; Rust will not
    // let a `&mut` be taken of an uninitialized binding, so zero it first. Same bytes either way.
    let mut formatetc: FORMATETC = core::mem::zeroed();
    InitFormatEtc(&mut formatetc, CF_HDROP, TYMED_HGLOBAL);
    hr = ((*(*pda).lpVtbl).GetData)(pda, &mut formatetc, &mut medium);
    if ((hr) < 0) {
        return hr;
    }

    // bindgen renders STGMEDIUM's anonymous union as __bindgen_anon_1.
    let lpdf = GlobalLock(medium.__bindgen_anon_1.hGlobal) as *mut DROPFILES;
    if lpdf.is_null() {
        ReleaseStgMedium(&mut medium);
        return E_FAIL;
    }

    let mut FileOp: SHFILEOPSTRUCTW = core::mem::zeroed();
    ZeroMemory(&mut FileOp as *mut _ as *mut core::ffi::c_void, core::mem::size_of::<SHFILEOPSTRUCTW>());
    FileOp.wFunc = FO_DELETE;
    FileOp.pFrom = ((lpdf as *mut u8).add((*lpdf).pFiles as usize)) as LPWSTR;
    if (fMask & CMIC_MASK_SHIFT_DOWN) == 0 {
        FileOp.fFlags = FOF_ALLOWUNDO as FILEOP_FLAGS;
    }

    let res = SHFileOperationW(&mut FileOp);
    if res != 0 {
        hr = E_FAIL;
    }

    GlobalUnlock(medium.__bindgen_anon_1.hGlobal);
    ReleaseStgMedium(&mut medium);

    return hr;
}

// `_DoDeleteThreadProc` and `_DoDeleteDataObject` are both STATIC members: the C has no `this`
// on either. The scaffold gives every method body a `this` parameter anyway, which is why the
// call envelope demanded three arguments at the `_DoDeleteDataObject_body` call below and
// rejected the model's (correct) two on the first attempt. Neither body reads `this`, and the
// thread entry has no receiver to supply — DeleteThreadData carries only { s, fMask }, unlike
// CFSDropTarget's _DoDropData which at least carries a `This`. So the trampoline passes null
// and that is the honest value, not a placeholder to be filled in later.
unsafe extern "system" fn CRecyclerDropTarget_DoDeleteThreadProc_entry(lpParameter: LPVOID) -> DWORD {
    CRecyclerDropTarget__DoDeleteThreadProc_body(core::ptr::null_mut(), lpParameter)
}

pub unsafe fn CRecyclerDropTarget__DoDeleteThreadProc_body(this: *mut CRecyclerDropTarget, lpParameter: LPVOID) -> DWORD {
    let data = lpParameter as *mut DeleteThreadData;
    CoInitialize(core::ptr::null_mut());
    let mut pDataObject: *mut IDataObject = core::ptr::null_mut();
    // The island's CoGetInterfaceAndReleaseStream is typed against com-sys' IStream/GUID; this
    // file declares its own layout-identical IStream, so the pointers cross with a `.cast()`.
    let hr = CoGetInterfaceAndReleaseStream((*data).s.cast(), core::ptr::addr_of!(IID_IDataObject).cast(), core::ptr::addr_of_mut!(pDataObject).cast());
    if hr >= 0 {
        CRecyclerDropTarget__DoDeleteDataObject_body(this, pDataObject, (*data).fMask);
    }
    ((*(*pDataObject).lpVtbl).Release)(pDataObject);
    CoUninitialize();
    HeapFree(GetProcessHeap(), 0, data as LPVOID);
    return 0;
}

pub unsafe fn CRecyclerDropTarget__DoDeleteAsync_body(this: *mut CRecyclerDropTarget, pda: *mut IDataObject, fMask: DWORD) -> () {
    let data = HeapAlloc(GetProcessHeap(), 0, core::mem::size_of::<DeleteThreadData>()) as *mut DeleteThreadData;
    (*data).fMask = fMask;
    CoMarshalInterThreadInterfaceInStream(core::ptr::addr_of!(IID_IDataObject).cast(), pda.cast(), core::ptr::addr_of_mut!((*data).s).cast());
    // The C passes NULL for both the flags and the callback; flags is a DWORD, not a pointer.
    SHCreateThread(Some(CRecyclerDropTarget_DoDeleteThreadProc_entry), data as LPVOID, 0, None);
}

pub unsafe fn CRecyclerDropTarget_ctor_body(this: *mut CRecyclerDropTarget) -> () {
    (*this).fAcceptFmt = FALSE;
    // CFSTR_SHELLIDLIST is the TCHAR macro and this TU is compiled UNICODE, so it is the W
    // form. shellutils.rs owns the one definition of that clipboard-format name.
    (*this).cfShellIDList = RegisterClipboardFormatW(CFSTR_SHELLIDLISTW.as_ptr());
}

pub unsafe fn CRecyclerDropTarget_DragEnter_body(this: *mut CRecyclerDropTarget, pDataObject: *mut IDataObject, dwKeyState: DWORD, pt: POINTL, pdwEffect: *mut DWORD) -> HRESULT {
    (*this).fAcceptFmt = TRUE;
    *pdwEffect = DROPEFFECT_MOVE;
    S_OK
}

pub unsafe fn CRecyclerDropTarget_DragOver_body(this: *mut CRecyclerDropTarget, dwKeyState: DWORD, pt: POINTL, pdwEffect: *mut DWORD) -> HRESULT {
    if pdwEffect.is_null() {
        return E_INVALIDARG;
    }

    *pdwEffect = DROPEFFECT_MOVE;

    return S_OK;
}

pub unsafe fn CRecyclerDropTarget_DragLeave_body(this: *mut CRecyclerDropTarget) -> HRESULT {
    (*this).fAcceptFmt = FALSE;

    return S_OK;
}

pub unsafe fn CRecyclerDropTarget_Drop_body(this: *mut CRecyclerDropTarget, pDataObject: *mut IDataObject, grfKeyState: DWORD, pt: POINTL, pdwEffect: *mut DWORD) -> HRESULT {
    let mut fmt: FORMATETC = core::mem::zeroed();

    InitFormatEtc(&mut fmt, (*this).cfShellIDList, TYMED_HGLOBAL);

    if ((*(*pDataObject).lpVtbl).QueryGetData)(pDataObject, &mut fmt) >= 0 {
        let mut fMask: DWORD = 0;

        if (grfKeyState & MK_SHIFT) == MK_SHIFT {
            fMask |= CMIC_MASK_SHIFT_DOWN;
        }

        CRecyclerDropTarget__DoDeleteAsync_body(this, pDataObject, fMask);
    } else {
    }

    S_OK
}

// ============================================================================
// HAND-WRITTEN: the C-ABI entry point the rest of the tree calls.
//
// `CRecyclerDropTarget_CreateInstance` above is the `--internal` non-aggregating factory, so
// it takes the aggregation outer as its FIRST argument. Every caller — the Rust CRecycleBin
// (recycle_bin.rs:1908) and the still-C++ CDefaultContextMenu.cpp — uses the 2-argument
// spelling the C header had, so the adapter lives here rather than at each call site. Sharing
// the plain name between a 2-arg C++ declaration and a 3-arg Rust definition would have
// linked without complaint and then read garbage for ppvOut.
// ============================================================================
#[no_mangle]
pub unsafe extern "system" fn CRecyclerDropTarget_CreateInstance_RustBridge(
    riid: REFIID,
    ppvOut: *mut LPVOID,
) -> HRESULT {
    CRecyclerDropTarget_CreateInstance(core::ptr::null_mut(), riid, ppvOut.cast())
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * Trash virtual folder support. The trashing engine is implemented in trash.c
//  *
//  * Copyright (C) 2006 Mikolaj Zalewski
//  * Copyright (C) 2009 Andrew Hill
//  * Copyright (C) 2017 Giannis Adamopoulos
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
// WINE_DEFAULT_DEBUG_CHANNEL (shell);
//
// class CRecyclerDropTarget :
//     public CComObjectRootEx<CComMultiThreadModelNoCS>,
//     public IDropTarget
// {
//     private:
//         BOOL fAcceptFmt;       /* flag for pending Drop */
//         UINT cfShellIDList;
//
//         static HRESULT _DoDeleteDataObject(IDataObject *pda, DWORD fMask)
//         {
//             HRESULT hr;
//             STGMEDIUM medium;
//             FORMATETC formatetc;
//             InitFormatEtc (formatetc, CF_HDROP, TYMED_HGLOBAL);
//             hr = pda->GetData(&formatetc, &medium);
//             if (FAILED_UNEXPECTEDLY(hr))
//                 return hr;
//
//             LPDROPFILES lpdf = (LPDROPFILES) GlobalLock(medium.hGlobal);
//             if (!lpdf)
//             {
//                 ERR("Error locking global\n");
//                 ReleaseStgMedium(&medium);
//                 return E_FAIL;
//             }
//
//             /* Delete them */
//             SHFILEOPSTRUCTW FileOp;
//             ZeroMemory(&FileOp, sizeof(FileOp));
//             FileOp.wFunc = FO_DELETE;
//             FileOp.pFrom = (LPWSTR) (((byte*) lpdf) + lpdf->pFiles);;
//             if ((fMask & CMIC_MASK_SHIFT_DOWN) == 0)
//                 FileOp.fFlags = FOF_ALLOWUNDO;
//             TRACE("Deleting file (just the first) = %s, allowundo: %d\n", debugstr_w(FileOp.pFrom), (FileOp.fFlags == FOF_ALLOWUNDO));
//
//             int res = SHFileOperationW(&FileOp);
//             if (res)
//             {
//                 ERR("SHFileOperation failed with 0x%x\n", res);
//                 hr = E_FAIL;
//             }
//
//             GlobalUnlock(medium.hGlobal);
//             ReleaseStgMedium(&medium);
//
//             return hr;
//         }
//         static DWORD WINAPI _DoDeleteThreadProc(LPVOID lpParameter)
//         {
//             DeleteThreadData *data = static_cast<DeleteThreadData*>(lpParameter);
//             CoInitialize(NULL);
//             IDataObject *pDataObject;
//             HRESULT hr = CoGetInterfaceAndReleaseStream (data->s, IID_PPV_ARG(IDataObject, &pDataObject));
//             if (SUCCEEDED(hr))
//             {
//                 _DoDeleteDataObject(pDataObject, data->fMask);
//             }
//             pDataObject->Release();
//             CoUninitialize();
//             HeapFree(GetProcessHeap(), 0, data);
//             return 0;
//         }
//         static void _DoDeleteAsync(IDataObject *pda, DWORD fMask)
//         {
//             DeleteThreadData *data = static_cast<DeleteThreadData*>(HeapAlloc(GetProcessHeap(), 0, sizeof(DeleteThreadData)));
//             data->fMask = fMask;
//             CoMarshalInterThreadInterfaceInStream(IID_IDataObject, pda, &data->s);
//             SHCreateThread(_DoDeleteThreadProc, data, NULL, NULL);
//         }
//         DragEnter(IDataObject *pDataObject, DWORD dwKeyState, POINTL pt, DWORD *pdwEffect) override
//         {
//             TRACE("Recycle bin drag over (%p)\n", this);
//             /* The recycle bin accepts pretty much everything, and sets a CSIDL flag. */
//             fAcceptFmt = TRUE;
//
//             *pdwEffect = DROPEFFECT_MOVE;
//             return S_OK;
//         }
//         DragOver(DWORD dwKeyState, POINTL pt, DWORD *pdwEffect) override
//         {
//             TRACE("(%p)\n", this);
//
//             if (!pdwEffect)
//                 return E_INVALIDARG;
//
//             *pdwEffect = DROPEFFECT_MOVE;
//
//             return S_OK;
//         }
//         DragLeave() override
//         {
//             TRACE("(%p)\n", this);
//
//             fAcceptFmt = FALSE;
//
//             return S_OK;
//         }
//         Drop(IDataObject *pDataObject, DWORD grfKeyState, POINTL pt, DWORD *pdwEffect) override
//         {
//             TRACE("(%p) object dropped on recycle bin, effect %u\n", this, *pdwEffect);
//
//             FORMATETC fmt;
//             TRACE("(%p)->(DataObject=%p)\n", this, pDataObject);
//             InitFormatEtc (fmt, cfShellIDList, TYMED_HGLOBAL);
//
//             /* Handle cfShellIDList Drop objects here, otherwise send the appropriate message to other software */
//             if (SUCCEEDED(pDataObject->QueryGetData(&fmt)))
//             {
//                 DWORD fMask = 0;
//
//                 if ((grfKeyState & MK_SHIFT) == MK_SHIFT)
//                     fMask |= CMIC_MASK_SHIFT_DOWN;
//
//                 _DoDeleteAsync(pDataObject, fMask);
//             }
//             else
//             {
//                 /*
//                  * TODO call SetData on the data object with format CFSTR_TARGETCLSID
//                  * set to the Recycle Bin's class identifier CLSID_RecycleBin.
//                  */
//             }
//             return S_OK;
//         }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
