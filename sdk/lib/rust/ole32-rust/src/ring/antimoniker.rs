use crate::prelude::*;

// Defined by a sibling ring module, but ALSO (wrongly) declared as an extern in
// ole32_exports.rs — the prelude glob makes the bare name ambiguous, so bind explicitly.
use crate::ring::compositemoniker::{CreateGenericComposite, MonikerCommonPrefixWith};

// Semantically ported to wine-10.0 (upstream 5f6b295cb90): an anti-moniker now
// carries an ORDER (`count`), i.e. how many components it cancels. Load/Save/Hash/
// IsEqual/CommonPrefixWith/GetDisplayName/GetComparisonData all speak in that count,
// and `is_anti_moniker`/`create_anti_moniker` are the helpers every other moniker's
// ComposeWith uses to cancel against it.

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct AntiMonikerImpl {
    pub IMoniker_iface: IMoniker,
    pub IROTData_iface: IROTData,
    pub ref_: LONG,
    pub pMarshal: *mut IUnknown,
    pub count: DWORD,
}

pub type PAntiMonikerImpl = *mut AntiMonikerImpl;

static VT_AntiMonikerImpl: IMonikerVtbl = IMonikerVtbl {
    QueryInterface: Some(AntiMonikerImpl_QueryInterface),
    AddRef: Some(AntiMonikerImpl_AddRef),
    Release: Some(AntiMonikerImpl_Release),
    GetClassID: Some(AntiMonikerImpl_GetClassID),
    IsDirty: Some(AntiMonikerImpl_IsDirty),
    Load: Some(AntiMonikerImpl_Load),
    Save: Some(AntiMonikerImpl_Save),
    GetSizeMax: Some(AntiMonikerImpl_GetSizeMax),
    BindToObject: Some(AntiMonikerImpl_BindToObject),
    BindToStorage: Some(AntiMonikerImpl_BindToStorage),
    Reduce: Some(AntiMonikerImpl_Reduce),
    ComposeWith: Some(AntiMonikerImpl_ComposeWith),
    Enum: Some(AntiMonikerImpl_Enum),
    IsEqual: Some(AntiMonikerImpl_IsEqual),
    Hash: Some(AntiMonikerImpl_Hash),
    IsRunning: Some(AntiMonikerImpl_IsRunning),
    GetTimeOfLastChange: Some(AntiMonikerImpl_GetTimeOfLastChange),
    Inverse: Some(AntiMonikerImpl_Inverse),
    CommonPrefixWith: Some(AntiMonikerImpl_CommonPrefixWith),
    RelativePathTo: Some(AntiMonikerImpl_RelativePathTo),
    GetDisplayName: Some(AntiMonikerImpl_GetDisplayName),
    ParseDisplayName: Some(AntiMonikerImpl_ParseDisplayName),
    IsSystemMoniker: Some(AntiMonikerImpl_IsSystemMoniker),
};

static VT_ROTDataImpl: IROTDataVtbl = IROTDataVtbl {
    QueryInterface: Some(AntiMonikerROTDataImpl_QueryInterface),
    AddRef: Some(AntiMonikerROTDataImpl_AddRef),
    Release: Some(AntiMonikerROTDataImpl_Release),
    GetComparisonData: Some(AntiMonikerROTDataImpl_GetComparisonData),
};

unsafe fn impl_from_IMoniker(iface: *mut IMoniker) -> *mut AntiMonikerImpl {
    CONTAINING_RECORD!(iface, AntiMonikerImpl, IMoniker_iface)
}

unsafe fn impl_from_IROTData(iface: *mut IROTData) -> *mut AntiMonikerImpl {
    CONTAINING_RECORD!(iface, AntiMonikerImpl, IROTData_iface)
}

/// wine-10 `unsafe_impl_from_IMoniker`: identify OUR anti-moniker by its vtable,
/// never by `IsSystemMoniker` (a foreign moniker can claim MKSYS_ANTIMONIKER).
unsafe fn unsafe_impl_from_IMoniker(iface: *mut IMoniker) -> *mut AntiMonikerImpl {
    if iface.is_null()
        || (*iface).lpVtbl as *const IMonikerVtbl != core::ptr::addr_of!(VT_AntiMonikerImpl)
    {
        return core::ptr::null_mut();
    }
    impl_from_IMoniker(iface)
}

/// wine-10 `is_anti_moniker`: true with the anti-moniker's order in `*order`, or
/// false with `*order = 0`. (C returns BOOL; crate-internal, so a Rust bool.)
pub unsafe fn is_anti_moniker(iface: *mut IMoniker, order: *mut DWORD) -> bool {
    let moniker = unsafe_impl_from_IMoniker(iface);

    if moniker.is_null() {
        *order = 0;
        return false;
    }

    *order = (*moniker).count;

    true
}

stdmethod! {
fn AntiMonikerImpl_QueryInterface(
    iface: *mut IMoniker,
    riid: *const IID,
    ppvObject: *mut LPVOID,
) -> HRESULT {
    let This = impl_from_IMoniker(iface);

    TRACE!(b"(%p,%s,%p)\n\0".as_ptr(), This, debugstr_guid(riid), ppvObject);

    /* NOTE: unlike the other monikers, the C returns E_INVALIDARG here, not E_POINTER. */
    if ppvObject.is_null() {
        return E_INVALIDARG;
    }

    *ppvObject = core::ptr::null_mut();

    if IsEqualIID(core::ptr::addr_of!(IID_IUnknown), riid)
        || IsEqualIID(core::ptr::addr_of!(IID_IPersist), riid)
        || IsEqualIID(core::ptr::addr_of!(IID_IPersistStream), riid)
        || IsEqualIID(core::ptr::addr_of!(IID_IMoniker), riid)
        || IsEqualGUID(core::ptr::addr_of!(CLSID_AntiMoniker), riid)
    {
        *ppvObject = iface as LPVOID;
    } else if IsEqualIID(core::ptr::addr_of!(IID_IROTData), riid) {
        *ppvObject = core::ptr::addr_of_mut!((*This).IROTData_iface) as LPVOID;
    } else if IsEqualIID(core::ptr::addr_of!(IID_IMarshal), riid) {
        let mut hr: HRESULT = S_OK;
        if (*This).pMarshal.is_null() {
            hr = MonikerMarshal_Create(iface, core::ptr::addr_of_mut!((*This).pMarshal));
        }
        if hr != S_OK {
            return hr;
        }
        return IUnknown_QueryInterface((*This).pMarshal, riid, ppvObject);
    }

    if (*ppvObject).is_null() {
        return E_NOINTERFACE;
    }

    IMoniker_AddRef(iface);

    S_OK
}
}

stdmethod! {
fn AntiMonikerImpl_AddRef(iface: *mut IMoniker) -> ULONG {
    let moniker = impl_from_IMoniker(iface);
    let refcount = InterlockedIncrement(core::ptr::addr_of_mut!((*moniker).ref_)) as ULONG;

    TRACE!(b"%p, refcount %u.\n\0".as_ptr(), iface, refcount);

    refcount
}
}

stdmethod! {
fn AntiMonikerImpl_Release(iface: *mut IMoniker) -> ULONG {
    let moniker = impl_from_IMoniker(iface);
    let refcount = InterlockedDecrement(core::ptr::addr_of_mut!((*moniker).ref_)) as ULONG;

    TRACE!(b"%p, refcount %u.\n\0".as_ptr(), iface, refcount);

    if refcount == 0 {
        if !(*moniker).pMarshal.is_null() {
            IUnknown_Release((*moniker).pMarshal);
        }
        HeapFree(GetProcessHeap(), 0, moniker as LPVOID);
    }

    refcount
}
}

stdmethod! {
fn AntiMonikerImpl_GetClassID(iface: *mut IMoniker, pClassID: *mut CLSID) -> HRESULT {
    TRACE!(b"(%p,%p)\n\0".as_ptr(), iface, pClassID);

    if pClassID.is_null() {
        return E_POINTER;
    }

    *pClassID = CLSID_AntiMoniker;

    S_OK
}
}

stdmethod! {
fn AntiMonikerImpl_IsDirty(iface: *mut IMoniker) -> HRESULT {
    /* Note that the OLE-provided implementations of the IPersistStream::IsDirty
       method in the OLE-provided moniker interfaces always return S_FALSE because
       their internal state never changes. */

    TRACE!(b"(%p)\n\0".as_ptr(), iface);

    S_FALSE
}
}

stdmethod! {
fn AntiMonikerImpl_Load(iface: *mut IMoniker, stream: *mut IStream) -> HRESULT {
    let moniker = impl_from_IMoniker(iface);
    let mut count: DWORD = 0;

    TRACE!(b"%p, %p.\n\0".as_ptr(), iface, stream);

    let hr = IStream_Read(
        stream,
        core::ptr::addr_of_mut!(count) as LPVOID,
        core::mem::size_of::<DWORD>() as ULONG,
        core::ptr::null_mut(),
    );
    if FAILED(hr) {
        return hr;
    }

    if count > 0xfffff {
        return E_INVALIDARG;
    }

    (*moniker).count = count;

    S_OK
}
}

stdmethod! {
fn AntiMonikerImpl_Save(iface: *mut IMoniker, stream: *mut IStream, clear_dirty: BOOL) -> HRESULT {
    let moniker = impl_from_IMoniker(iface);

    TRACE!(b"%p, %p, %d.\n\0".as_ptr(), iface, stream, clear_dirty);

    IStream_Write(
        stream,
        core::ptr::addr_of!((*moniker).count) as *const core::ffi::c_void,
        core::mem::size_of::<DWORD>() as ULONG,
        core::ptr::null_mut(),
    )
}
}

stdmethod! {
fn AntiMonikerImpl_GetSizeMax(iface: *mut IMoniker, pcbSize: *mut ULARGE_INTEGER) -> HRESULT {
    TRACE!(b"(%p,%p)\n\0".as_ptr(), iface, pcbSize);

    if pcbSize.is_null() {
        return E_POINTER;
    }

    /* Normally the sizemax must be sizeof DWORD, but native returns 16 bytes more
       than the number of bytes AntiMoniker::Save writes. */
    (*pcbSize).QuadPart = (core::mem::size_of::<DWORD>() + 16) as ULONGLONG;

    S_OK
}
}

stdmethod! {
fn AntiMonikerImpl_BindToObject(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    riid: *const IID,
    ppvResult: *mut LPVOID,
) -> HRESULT {
    TRACE!(
        b"(%p,%p,%p,%s,%p)\n\0".as_ptr(),
        iface,
        pbc,
        pmkToLeft,
        debugstr_guid(riid),
        ppvResult,
    );

    E_NOTIMPL
}
}

stdmethod! {
fn AntiMonikerImpl_BindToStorage(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    riid: *const IID,
    ppvResult: *mut LPVOID,
) -> HRESULT {
    TRACE!(
        b"(%p,%p,%p,%s,%p)\n\0".as_ptr(),
        iface,
        pbc,
        pmkToLeft,
        debugstr_guid(riid),
        ppvResult,
    );

    E_NOTIMPL
}
}

stdmethod! {
fn AntiMonikerImpl_Reduce(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    dwReduceHowFar: DWORD,
    ppmkToLeft: *mut *mut IMoniker,
    ppmkReduced: *mut *mut IMoniker,
) -> HRESULT {
    TRACE!(
        b"%p, %p, %d, %p, %p.\n\0".as_ptr(),
        iface,
        pbc,
        dwReduceHowFar,
        ppmkToLeft,
        ppmkReduced,
    );

    if ppmkReduced.is_null() {
        return E_POINTER;
    }

    AntiMonikerImpl_AddRef(iface);

    *ppmkReduced = iface;

    MK_S_REDUCED_TO_SELF
}
}

stdmethod! {
fn AntiMonikerImpl_ComposeWith(
    iface: *mut IMoniker,
    pmkRight: *mut IMoniker,
    fOnlyIfNotGeneric: BOOL,
    ppmkComposite: *mut *mut IMoniker,
) -> HRESULT {
    TRACE!(b"(%p,%p,%d,%p)\n\0".as_ptr(), iface, pmkRight, fOnlyIfNotGeneric, ppmkComposite);

    if ppmkComposite.is_null() || pmkRight.is_null() {
        return E_POINTER;
    }

    *ppmkComposite = core::ptr::null_mut();

    if fOnlyIfNotGeneric != 0 {
        MK_E_NEEDGENERIC
    } else {
        CreateGenericComposite(iface, pmkRight, ppmkComposite)
    }
}
}

stdmethod! {
fn AntiMonikerImpl_Enum(
    iface: *mut IMoniker,
    forward: BOOL,
    ppenumMoniker: *mut *mut IEnumMoniker,
) -> HRESULT {
    TRACE!(b"%p, %d, %p.\n\0".as_ptr(), iface, forward, ppenumMoniker);

    if ppenumMoniker.is_null() {
        return E_INVALIDARG;
    }

    *ppenumMoniker = core::ptr::null_mut();

    S_OK
}
}

stdmethod! {
fn AntiMonikerImpl_IsEqual(iface: *mut IMoniker, other: *mut IMoniker) -> HRESULT {
    let moniker = impl_from_IMoniker(iface);

    TRACE!(b"%p, %p.\n\0".as_ptr(), iface, other);

    if other.is_null() {
        return E_INVALIDARG;
    }

    let other_moniker = unsafe_impl_from_IMoniker(other);
    if other_moniker.is_null() {
        return S_FALSE;
    }

    if (*moniker).count == (*other_moniker).count {
        S_OK
    } else {
        S_FALSE
    }
}
}

stdmethod! {
fn AntiMonikerImpl_Hash(iface: *mut IMoniker, hash: *mut DWORD) -> HRESULT {
    let moniker = impl_from_IMoniker(iface);

    TRACE!(b"%p, %p.\n\0".as_ptr(), iface, hash);

    if hash.is_null() {
        return E_POINTER;
    }

    *hash = 0x8000_0000 | (*moniker).count;

    S_OK
}
}

stdmethod! {
fn AntiMonikerImpl_IsRunning(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    pmkNewlyRunning: *mut IMoniker,
) -> HRESULT {
    let mut rot: *mut IRunningObjectTable = core::ptr::null_mut();

    TRACE!(b"(%p,%p,%p,%p)\n\0".as_ptr(), iface, pbc, pmkToLeft, pmkNewlyRunning);

    if pbc.is_null() {
        return E_INVALIDARG;
    }

    let mut res = IBindCtx_GetRunningObjectTable(pbc, &mut rot);

    if FAILED(res) {
        return res;
    }

    res = IRunningObjectTable_IsRunning(rot, iface);

    IRunningObjectTable_Release(rot);

    res
}
}

stdmethod! {
fn AntiMonikerImpl_GetTimeOfLastChange(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    pAntiTime: *mut FILETIME,
) -> HRESULT {
    TRACE!(b"(%p,%p,%p,%p)\n\0".as_ptr(), iface, pbc, pmkToLeft, pAntiTime);

    E_NOTIMPL
}
}

stdmethod! {
fn AntiMonikerImpl_Inverse(iface: *mut IMoniker, ppmk: *mut *mut IMoniker) -> HRESULT {
    TRACE!(b"(%p,%p)\n\0".as_ptr(), iface, ppmk);

    if ppmk.is_null() {
        return E_POINTER;
    }

    *ppmk = core::ptr::null_mut();

    MK_E_NOINVERSE
}
}

stdmethod! {
fn AntiMonikerImpl_CommonPrefixWith(
    iface: *mut IMoniker,
    other: *mut IMoniker,
    prefix: *mut *mut IMoniker,
) -> HRESULT {
    let moniker = impl_from_IMoniker(iface);

    TRACE!(b"%p, %p, %p.\n\0".as_ptr(), iface, other, prefix);

    let other_moniker = unsafe_impl_from_IMoniker(other);
    if !other_moniker.is_null() {
        let hr;

        if (*moniker).count <= (*other_moniker).count {
            *prefix = iface;
            hr = if (*moniker).count == (*other_moniker).count {
                MK_S_US
            } else {
                MK_S_ME
            };
        } else {
            *prefix = other;
            hr = MK_S_HIM;
        }

        IMoniker_AddRef(*prefix);
        return hr;
    }

    MonikerCommonPrefixWith(iface, other, prefix)
}
}

stdmethod! {
fn AntiMonikerImpl_RelativePathTo(
    iface: *mut IMoniker,
    other: *mut IMoniker,
    result: *mut *mut IMoniker,
) -> HRESULT {
    TRACE!(b"%p, %p, %p.\n\0".as_ptr(), iface, other, result);

    if other.is_null() || result.is_null() {
        return E_INVALIDARG;
    }

    IMoniker_AddRef(other);
    *result = other;

    MK_S_HIM
}
}

stdmethod! {
fn AntiMonikerImpl_GetDisplayName(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    displayname: *mut LPOLESTR,
) -> HRESULT {
    let moniker = impl_from_IMoniker(iface);
    const nameW: [WCHAR; 3] = [b'\\' as WCHAR, b'.' as WCHAR, b'.' as WCHAR];

    TRACE!(b"%p, %p, %p, %p.\n\0".as_ptr(), iface, pbc, pmkToLeft, displayname);

    if displayname.is_null() {
        return E_POINTER;
    }

    if !pmkToLeft.is_null() {
        FIXME!(b"() pmkToLeft!=NULL not implemented\n\0".as_ptr());
        return E_NOTIMPL;
    }

    let count = (*moniker).count as usize;
    let ptrW = CoTaskMemAlloc(
        ((count * nameW.len() + 1) * core::mem::size_of::<WCHAR>()) as SIZE_T,
    ) as *mut WCHAR;
    *displayname = ptrW;
    if ptrW.is_null() {
        return E_OUTOFMEMORY;
    }

    /* One "\.." per cancelled component. */
    for i in 0..count {
        core::ptr::copy_nonoverlapping(nameW.as_ptr(), ptrW.add(i * nameW.len()), nameW.len());
    }
    *ptrW.add(count * nameW.len()) = 0;

    S_OK
}
}

stdmethod! {
fn AntiMonikerImpl_ParseDisplayName(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    pszDisplayName: LPOLESTR,
    pchEaten: *mut ULONG,
    ppmkOut: *mut *mut IMoniker,
) -> HRESULT {
    TRACE!(
        b"(%p,%p,%p,%p,%p,%p)\n\0".as_ptr(),
        iface,
        pbc,
        pmkToLeft,
        pszDisplayName,
        pchEaten,
        ppmkOut,
    );

    E_NOTIMPL
}
}

stdmethod! {
fn AntiMonikerImpl_IsSystemMoniker(iface: *mut IMoniker, pwdMksys: *mut DWORD) -> HRESULT {
    TRACE!(b"(%p,%p)\n\0".as_ptr(), iface, pwdMksys);

    if pwdMksys.is_null() {
        return E_POINTER;
    }

    *pwdMksys = MKSYS_ANTIMONIKER;

    S_OK
}
}

stdmethod! {
fn AntiMonikerROTDataImpl_QueryInterface(
    iface: *mut IROTData,
    riid: *const IID,
    ppvObject: *mut LPVOID,
) -> HRESULT {
    let This = impl_from_IROTData(iface);

    TRACE!(b"(%p,%s,%p)\n\0".as_ptr(), iface, debugstr_guid(riid), ppvObject);

    AntiMonikerImpl_QueryInterface(
        core::ptr::addr_of_mut!((*This).IMoniker_iface),
        riid,
        ppvObject,
    )
}
}

stdmethod! {
fn AntiMonikerROTDataImpl_AddRef(iface: *mut IROTData) -> ULONG {
    let This = impl_from_IROTData(iface);

    TRACE!(b"(%p)\n\0".as_ptr(), iface);

    AntiMonikerImpl_AddRef(core::ptr::addr_of_mut!((*This).IMoniker_iface))
}
}

stdmethod! {
fn AntiMonikerROTDataImpl_Release(iface: *mut IROTData) -> ULONG {
    let This = impl_from_IROTData(iface);

    TRACE!(b"(%p)\n\0".as_ptr(), iface);

    AntiMonikerImpl_Release(core::ptr::addr_of_mut!((*This).IMoniker_iface))
}
}

stdmethod! {
fn AntiMonikerROTDataImpl_GetComparisonData(
    iface: *mut IROTData,
    data: *mut byte,
    data_len: ULONG,
    data_req: *mut ULONG,
) -> HRESULT {
    let moniker = impl_from_IROTData(iface);

    TRACE!(b"%p, %p, %u, %p.\n\0".as_ptr(), iface, data, data_len, data_req);

    *data_req = (core::mem::size_of::<CLSID>() + core::mem::size_of::<DWORD>()) as ULONG;
    if data_len < *data_req {
        return E_OUTOFMEMORY;
    }

    memcpy(
        data as *mut core::ffi::c_void,
        core::ptr::addr_of!(CLSID_AntiMoniker) as *const core::ffi::c_void,
        core::mem::size_of::<CLSID>(),
    );
    memcpy(
        data.add(core::mem::size_of::<CLSID>()) as *mut core::ffi::c_void,
        core::ptr::addr_of!((*moniker).count) as *const core::ffi::c_void,
        core::mem::size_of::<DWORD>(),
    );

    S_OK
}
}

/// wine-10 `create_anti_moniker`: a new anti-moniker of the given order, refcount 1.
pub unsafe fn create_anti_moniker(order: DWORD, ret: *mut *mut IMoniker) -> HRESULT {
    let moniker = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        core::mem::size_of::<AntiMonikerImpl>() as usize,
    ) as *mut AntiMonikerImpl;

    if moniker.is_null() {
        return E_OUTOFMEMORY;
    }

    (*moniker).IMoniker_iface.lpVtbl =
        core::ptr::addr_of!(VT_AntiMonikerImpl) as *mut IMonikerVtbl;
    (*moniker).IROTData_iface.lpVtbl =
        core::ptr::addr_of!(VT_ROTDataImpl) as *mut IROTDataVtbl;
    (*moniker).ref_ = 1;
    (*moniker).count = order;

    *ret = core::ptr::addr_of_mut!((*moniker).IMoniker_iface);

    S_OK
}

/******************************************************************************
 *        CreateAntiMoniker	[OLE32.@]
 ******************************************************************************/
#[no_mangle]
pub unsafe extern "system" fn CreateAntiMoniker(moniker: *mut *mut IMoniker) -> HRESULT {
    TRACE!(b"%p.\n\0".as_ptr(), moniker);

    create_anti_moniker(1, moniker)
}

#[no_mangle]
pub unsafe extern "system" fn AntiMoniker_CreateInstance(
    _iface: *mut IClassFactory,
    pUnk: *mut IUnknown,
    riid: REFIID,
    ppv: *mut LPVOID,
) -> HRESULT {
    let mut pMoniker: *mut IMoniker = core::ptr::null_mut();

    TRACE!(b"(%p, %s, %p)\n\0".as_ptr(), pUnk, debugstr_guid(riid), ppv);

    *ppv = core::ptr::null_mut();

    if !pUnk.is_null() {
        return CLASS_E_NOAGGREGATION;
    }

    let hr = CreateAntiMoniker(&mut pMoniker);
    if FAILED(hr) {
        return hr;
    }

    let hr = IMoniker_QueryInterface(pMoniker, riid, ppv);
    /* wine-10: always drop the creation reference (the old code leaked it on success). */
    IMoniker_Release(pMoniker);

    hr
}
