use crate::prelude::*;
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

// Deterministic COM scaffold for IEnumFORMATETCImpl; method bodies are generated separately.
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
pub struct IEnumFORMATETCImpl {
    pub IEnumFORMATETC_iface: IEnumFORMATETC,
    pub ref_count: core::sync::atomic::AtomicI32,
    pub posFmt: UINT,
    pub countFmt: UINT,
    pub pFmt: LPFORMATETC,
}

unsafe extern "system" fn IEnumFORMATETCImpl_IEnumFORMATETC_QueryInterface(This: *mut IEnumFORMATETC, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(IEnumFORMATETCImpl, IEnumFORMATETC_iface)) as *mut IEnumFORMATETCImpl;
    IEnumFORMATETCImpl_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn IEnumFORMATETCImpl_IEnumFORMATETC_AddRef(This: *mut IEnumFORMATETC) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(IEnumFORMATETCImpl, IEnumFORMATETC_iface)) as *mut IEnumFORMATETCImpl;
    IEnumFORMATETCImpl_AddRef(this)
}
unsafe extern "system" fn IEnumFORMATETCImpl_IEnumFORMATETC_Release(This: *mut IEnumFORMATETC) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(IEnumFORMATETCImpl, IEnumFORMATETC_iface)) as *mut IEnumFORMATETCImpl;
    IEnumFORMATETCImpl_Release(this)
}
unsafe extern "system" fn IEnumFORMATETCImpl_IEnumFORMATETC_Next(This: *mut IEnumFORMATETC, celt: ULONG, rgelt: *mut FORMATETC, pceltFetched: *mut ULONG) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(IEnumFORMATETCImpl, IEnumFORMATETC_iface)) as *mut IEnumFORMATETCImpl;
    IEnumFORMATETCImpl_Next_body(this, celt, rgelt, pceltFetched)
}
unsafe extern "system" fn IEnumFORMATETCImpl_IEnumFORMATETC_Skip(This: *mut IEnumFORMATETC, celt: ULONG) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(IEnumFORMATETCImpl, IEnumFORMATETC_iface)) as *mut IEnumFORMATETCImpl;
    IEnumFORMATETCImpl_Skip_body(this, celt)
}
unsafe extern "system" fn IEnumFORMATETCImpl_IEnumFORMATETC_Reset(This: *mut IEnumFORMATETC) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(IEnumFORMATETCImpl, IEnumFORMATETC_iface)) as *mut IEnumFORMATETCImpl;
    IEnumFORMATETCImpl_Reset_body(this)
}
unsafe extern "system" fn IEnumFORMATETCImpl_IEnumFORMATETC_Clone(This: *mut IEnumFORMATETC, ppenum: *mut *mut IEnumFORMATETC) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(IEnumFORMATETCImpl, IEnumFORMATETC_iface)) as *mut IEnumFORMATETCImpl;
    IEnumFORMATETCImpl_Clone_body(this, ppenum)
}

pub static IENUMFORMATETCIMPL_IENUMFORMATETC_VTBL: IEnumFORMATETCVtbl = IEnumFORMATETCVtbl {
    QueryInterface: IEnumFORMATETCImpl_IEnumFORMATETC_QueryInterface,
    AddRef: IEnumFORMATETCImpl_IEnumFORMATETC_AddRef,
    Release: IEnumFORMATETCImpl_IEnumFORMATETC_Release,
    Next: IEnumFORMATETCImpl_IEnumFORMATETC_Next,
    Skip: IEnumFORMATETCImpl_IEnumFORMATETC_Skip,
    Reset: IEnumFORMATETCImpl_IEnumFORMATETC_Reset,
    Clone: IEnumFORMATETCImpl_IEnumFORMATETC_Clone,
};

unsafe fn IEnumFORMATETCImpl_QueryInterface(
    this: *mut IEnumFORMATETCImpl, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IUnknown)) || cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IEnumFORMATETC)) {
        *ppv = core::ptr::addr_of_mut!((*this).IEnumFORMATETC_iface) as *mut core::ffi::c_void;
    }
    else { return E_NOINTERFACE; }
    IEnumFORMATETCImpl_AddRef(this);
    S_OK
}

unsafe fn IEnumFORMATETCImpl_AddRef(this: *mut IEnumFORMATETCImpl) -> ULONG {
    (*this).ref_count.fetch_add(1, core::sync::atomic::Ordering::SeqCst) as ULONG + 1
}

unsafe fn IEnumFORMATETCImpl_Release(this: *mut IEnumFORMATETCImpl) -> ULONG {
    let refs = (*this).ref_count.fetch_sub(1, core::sync::atomic::Ordering::SeqCst) - 1;
    if refs == 0 {
        IEnumFORMATETCImpl_dtor_body(this);
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
pub unsafe fn IEnumFORMATETCImpl_CreateInstance(
    cfmt: UINT,
    afmt: *const FORMATETC,
    riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    let this = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<IEnumFORMATETCImpl>()) as *mut IEnumFORMATETCImpl;
    if this.is_null() { return E_OUTOFMEMORY; }
    core::ptr::write(this, IEnumFORMATETCImpl {
        IEnumFORMATETC_iface: IEnumFORMATETC { lpVtbl: core::ptr::addr_of!(IENUMFORMATETCIMPL_IENUMFORMATETC_VTBL) },
        ref_count: core::sync::atomic::AtomicI32::new(1),
        posFmt: core::mem::zeroed(),
        countFmt: core::mem::zeroed(),
        pFmt: core::mem::zeroed(),
    });
    Shell32RustObjectLock();
    IEnumFORMATETCImpl_ctor_body(this);
    let mut hr = IEnumFORMATETCImpl_Initialize_body(this, cfmt, afmt);
    if hr >= 0 {
        hr = IEnumFORMATETCImpl_QueryInterface(this, riid, ppv);
    }
    IEnumFORMATETCImpl_Release(this);
    hr
}

// Deterministic C++ support closure selected from source identifiers.

pub const CF_HDROP: DWORD = 15;

pub const DATADIR_GET: DWORD = 1;

pub const DVASPECT_CONTENT: DWORD = 1;

pub const TRACE_ON: DWORD = 1;

pub const TYMED_HGLOBAL: DWORD = 1;


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


#[repr(C)]
pub struct STATDATA {
    pub formatetc: FORMATETC,
    pub advf: DWORD,
    pub pAdvSink: *mut IAdviseSink,
    pub dwConnection: DWORD,
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

extern "C" {
    static IID_IDataObject: GUID;
}

use core::ffi::c_void;

type LPFORMATETC = *mut FORMATETC;

pub unsafe fn IEnumFORMATETCImpl_ctor_body(this: *mut IEnumFORMATETCImpl) -> () {
    (*this).posFmt = 0;
    (*this).countFmt = 0;
    (*this).pFmt = core::ptr::null_mut();
}

pub unsafe fn IEnumFORMATETCImpl_dtor_body(this: *mut IEnumFORMATETCImpl) -> () {}

pub unsafe fn IEnumFORMATETCImpl_Initialize_body(this: *mut IEnumFORMATETCImpl, cfmt: UINT, afmt: *const FORMATETC) -> HRESULT {
    let size: DWORD;

    size = cfmt * core::mem::size_of::<FORMATETC>() as DWORD;
    (*this).countFmt = cfmt;
    (*this).pFmt = SHAlloc(size as SIZE_T) as LPFORMATETC;
    if (*this).pFmt.is_null() {
        return E_OUTOFMEMORY;
    }

    memcpy((*this).pFmt as *mut core::ffi::c_void, afmt as *const core::ffi::c_void, size as usize);
    return S_OK;
}

pub unsafe fn IEnumFORMATETCImpl_Next_body(this: *mut IEnumFORMATETCImpl, celt: ULONG, rgelt: *mut FORMATETC, pceltFethed: *mut ULONG) -> HRESULT {
    let mut i: UINT;

    if (*this).pFmt.is_null() {
        return S_FALSE;
    }
    if rgelt.is_null() {
        return E_INVALIDARG;
    }
    if !pceltFethed.is_null() {
        *pceltFethed = 0;
    }

    i = 0;
    let mut rgelt_ptr = rgelt;
    while (*this).posFmt < (*this).countFmt && celt > i {
        *rgelt_ptr = *(*this).pFmt.add((*this).posFmt as usize);
        rgelt_ptr = rgelt_ptr.add(1);
        (*this).posFmt += 1;
        i += 1;
    }

    if !pceltFethed.is_null() {
        *pceltFethed = i;
    }

    if i == celt {
        S_OK
    } else {
        S_FALSE
    }
}

pub unsafe fn IEnumFORMATETCImpl_Skip_body(this: *mut IEnumFORMATETCImpl, celt: ULONG) -> HRESULT {
    if (*this).posFmt + celt >= (*this).countFmt {
        return S_FALSE;
    }
    (*this).posFmt += celt;
    S_OK
}

pub unsafe fn IEnumFORMATETCImpl_Reset_body(this: *mut IEnumFORMATETCImpl) -> HRESULT {
    (*this).posFmt = 0;
    return S_OK;
}

pub unsafe fn IEnumFORMATETCImpl_Clone_body(this: *mut IEnumFORMATETCImpl, ppenum: *mut *mut IEnumFORMATETC) -> HRESULT {
    let hResult: HRESULT;

    if ppenum.is_null() {
        return E_INVALIDARG;
    }
    hResult = IEnumFORMATETC_Constructor((*this).countFmt, (*this).pFmt, ppenum.cast());
    if ((hResult) < 0) {
        return hResult;
    }
    return ((*(**ppenum).lpVtbl).Skip)(*ppenum, (*this).posFmt);
}
