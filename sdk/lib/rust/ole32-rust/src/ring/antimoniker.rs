use crate::prelude::*;

// Defined by a sibling ring module, but ALSO (wrongly) declared as an extern in
// ole32_exports.rs — the prelude glob makes the bare name ambiguous, so bind explicitly.
use crate::ring::compositemoniker::{CreateGenericComposite, MonikerCommonPrefixWith};

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct AntiMonikerImpl {
    pub IMoniker_iface: IMoniker,
    pub IROTData_iface: IROTData,
    pub ref_: LONG,
    pub pMarshal: *mut IUnknown,
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

stdmethod! {
fn AntiMonikerImpl_QueryInterface(
    iface: *mut IMoniker,
    riid: *const IID,
    ppvObject: *mut LPVOID,
) -> HRESULT {
    let This = impl_from_IMoniker(iface);

    TRACE!(b"(%p,%s,%p)\n\0".as_ptr(), This, debugstr_guid(riid), ppvObject);

    /* Perform a sanity check on the parameters. */
    /* NOTE: unlike the other monikers, the C returns E_INVALIDARG here, not E_POINTER. */
    if ppvObject.is_null() {
        return E_INVALIDARG;
    }

    /* Initialize the return parameter */
    *ppvObject = core::ptr::null_mut();

    /* Compare the riid with the interface IDs implemented by this object. */
    if IsEqualIID(core::ptr::addr_of!(IID_IUnknown), riid)
        || IsEqualIID(core::ptr::addr_of!(IID_IPersist), riid)
        || IsEqualIID(core::ptr::addr_of!(IID_IPersistStream), riid)
        || IsEqualIID(core::ptr::addr_of!(IID_IMoniker), riid)
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

    /* Check that we obtained an interface. */
    if (*ppvObject).is_null() {
        return E_NOINTERFACE;
    }

    /* always increase the reference count by one when it is successful */
    IMoniker_AddRef(iface);

    S_OK
}
}

stdmethod! {
fn AntiMonikerImpl_AddRef(iface: *mut IMoniker) -> ULONG {
    let This = impl_from_IMoniker(iface);

    TRACE!(b"(%p)\n\0".as_ptr(), This);

    InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_)) as ULONG
}
}

stdmethod! {
fn AntiMonikerImpl_Release(iface: *mut IMoniker) -> ULONG {
    let This = impl_from_IMoniker(iface);

    TRACE!(b"(%p)\n\0".as_ptr(), This);

    let ref_count = InterlockedDecrement(core::ptr::addr_of_mut!((*This).ref_)) as ULONG;

    /* destroy the object if there are no more references to it */
    if ref_count == 0 {
        if !(*This).pMarshal.is_null() {
            IUnknown_Release((*This).pMarshal);
        }
        HeapFree(GetProcessHeap(), 0, This as LPVOID);
    }

    ref_count
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
fn AntiMonikerImpl_Load(iface: *mut IMoniker, pStm: *mut IStream) -> HRESULT {
    let constant: DWORD = 1;
    let mut dwbuffer: DWORD = 0;

    /* data read by this function is only a DWORD constant (must be 1) ! */
    let res = IStream_Read(
        pStm,
        core::ptr::addr_of_mut!(dwbuffer) as LPVOID,
        core::mem::size_of::<DWORD>() as ULONG,
        core::ptr::null_mut(),
    );

    if SUCCEEDED(res) && dwbuffer != constant {
        return E_FAIL;
    }

    res
}
}

stdmethod! {
fn AntiMonikerImpl_Save(iface: *mut IMoniker, pStm: *mut IStream, fClearDirty: BOOL) -> HRESULT {
    static CONSTANT: DWORD = 1;

    /* data written by this function is only a DWORD constant set to 1 ! */
    IStream_Write(
        pStm,
        core::ptr::addr_of!(CONSTANT) as *const core::ffi::c_void,
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

    /* for more details see AntiMonikerImpl_Save comments */

    /*
     * Normally the sizemax must be sizeof DWORD, but
     * I tested this function it usually return 16 bytes
     * more than the number of bytes used by AntiMoniker::Save function
     */
    (*pcbSize).u.LowPart = core::mem::size_of::<DWORD>() as DWORD + 16;
    (*pcbSize).u.HighPart = 0;

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
        b"(%p,%p,%d,%p,%p)\n\0".as_ptr(),
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
    TRACE!(
        b"(%p,%p,%d,%p)\n\0".as_ptr(),
        iface,
        pmkRight,
        fOnlyIfNotGeneric,
        ppmkComposite,
    );

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
    fForward: BOOL,
    ppenumMoniker: *mut *mut IEnumMoniker,
) -> HRESULT {
    TRACE!(b"(%p,%d,%p)\n\0".as_ptr(), iface, fForward, ppenumMoniker);

    if ppenumMoniker.is_null() {
        return E_POINTER;
    }

    *ppenumMoniker = core::ptr::null_mut();

    S_OK
}
}

stdmethod! {
fn AntiMonikerImpl_IsEqual(iface: *mut IMoniker, pmkOtherMoniker: *mut IMoniker) -> HRESULT {
    let mut mkSys: DWORD = 0;

    TRACE!(b"(%p,%p)\n\0".as_ptr(), iface, pmkOtherMoniker);

    if pmkOtherMoniker.is_null() {
        return S_FALSE;
    }

    IMoniker_IsSystemMoniker(pmkOtherMoniker, &mut mkSys);

    if mkSys == MKSYS_ANTIMONIKER {
        S_OK
    } else {
        S_FALSE
    }
}
}

stdmethod! {
fn AntiMonikerImpl_Hash(iface: *mut IMoniker, pdwHash: *mut DWORD) -> HRESULT {
    if pdwHash.is_null() {
        return E_POINTER;
    }

    *pdwHash = 0x80000001;

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

    TRACE!(
        b"(%p,%p,%p,%p)\n\0".as_ptr(),
        iface,
        pbc,
        pmkToLeft,
        pmkNewlyRunning,
    );

    if pbc.is_null() {
        return E_INVALIDARG;
    }

    let res = IBindCtx_GetRunningObjectTable(pbc, &mut rot);

    if FAILED(res) {
        return res;
    }

    let res = IRunningObjectTable_IsRunning(rot, iface);

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
    pmkOther: *mut IMoniker,
    ppmkPrefix: *mut *mut IMoniker,
) -> HRESULT {
    let mut mkSys: DWORD = 0;

    IMoniker_IsSystemMoniker(pmkOther, &mut mkSys);

    if mkSys == MKSYS_ANTIMONIKER {
        /* NOTE: the C AddRefs twice here — preserved verbatim. */
        IMoniker_AddRef(iface);

        *ppmkPrefix = iface;

        IMoniker_AddRef(iface);

        MK_S_US
    } else {
        MonikerCommonPrefixWith(iface, pmkOther, ppmkPrefix)
    }
}
}

stdmethod! {
fn AntiMonikerImpl_RelativePathTo(
    iface: *mut IMoniker,
    pmOther: *mut IMoniker,
    ppmkRelPath: *mut *mut IMoniker,
) -> HRESULT {
    TRACE!(b"(%p,%p,%p)\n\0".as_ptr(), iface, pmOther, ppmkRelPath);

    if ppmkRelPath.is_null() {
        return E_POINTER;
    }

    IMoniker_AddRef(pmOther);

    *ppmkRelPath = pmOther;

    MK_S_HIM
}
}

stdmethod! {
fn AntiMonikerImpl_GetDisplayName(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    ppszDisplayName: *mut LPOLESTR,
) -> HRESULT {
    static BACK: [WCHAR; 4] = [b'\\' as WCHAR, b'.' as WCHAR, b'.' as WCHAR, 0];

    TRACE!(
        b"(%p,%p,%p,%p)\n\0".as_ptr(),
        iface,
        pbc,
        pmkToLeft,
        ppszDisplayName,
    );

    if ppszDisplayName.is_null() {
        return E_POINTER;
    }

    if !pmkToLeft.is_null() {
        FIXME!(b"() pmkToLeft!=NULL not implemented\n\0".as_ptr());
        return E_NOTIMPL;
    }

    *ppszDisplayName = CoTaskMemAlloc(core::mem::size_of_val(&BACK) as SIZE_T) as LPOLESTR;

    if (*ppszDisplayName).is_null() {
        return E_OUTOFMEMORY;
    }

    lstrcpyW(*ppszDisplayName, BACK.as_ptr());

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
    pbData: *mut byte,
    cbMax: ULONG,
    pcbData: *mut ULONG,
) -> HRESULT {
    let constant: DWORD = 1;

    TRACE!(b"(%p, %u, %p)\n\0".as_ptr(), pbData, cbMax, pcbData);

    *pcbData = (core::mem::size_of::<CLSID>() + core::mem::size_of::<DWORD>()) as ULONG;
    if cbMax < *pcbData {
        return E_OUTOFMEMORY;
    }

    memcpy(
        pbData as *mut core::ffi::c_void,
        core::ptr::addr_of!(CLSID_AntiMoniker) as *const core::ffi::c_void,
        core::mem::size_of::<CLSID>(),
    );
    memcpy(
        pbData.add(core::mem::size_of::<CLSID>()) as *mut core::ffi::c_void,
        core::ptr::addr_of!(constant) as *const core::ffi::c_void,
        core::mem::size_of::<DWORD>(),
    );

    S_OK
}
}

unsafe fn AntiMonikerImpl_Construct(This: *mut AntiMonikerImpl) -> HRESULT {
    TRACE!(b"(%p)\n\0".as_ptr(), This);

    /* Initialize the virtual function table. */
    (*This).IMoniker_iface.lpVtbl =
        core::ptr::addr_of!(VT_AntiMonikerImpl) as *mut IMonikerVtbl;
    (*This).IROTData_iface.lpVtbl =
        core::ptr::addr_of!(VT_ROTDataImpl) as *mut IROTDataVtbl;
    (*This).ref_ = 0;
    (*This).pMarshal = core::ptr::null_mut();

    S_OK
}

/******************************************************************************
 *        CreateAntiMoniker	[OLE32.@]
 ******************************************************************************/
#[no_mangle]
pub unsafe extern "system" fn CreateAntiMoniker(ppmk: *mut *mut IMoniker) -> HRESULT {
    TRACE!(b"(%p)\n\0".as_ptr(), ppmk);

    let newAntiMoniker = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<AntiMonikerImpl>() as SIZE_T,
    ) as *mut AntiMonikerImpl;

    if newAntiMoniker.is_null() {
        return STG_E_INSUFFICIENTMEMORY;
    }

    let hr = AntiMonikerImpl_Construct(newAntiMoniker);
    if FAILED(hr) {
        HeapFree(GetProcessHeap(), 0, newAntiMoniker as LPVOID);
        return hr;
    }

    AntiMonikerImpl_QueryInterface(
        core::ptr::addr_of_mut!((*newAntiMoniker).IMoniker_iface),
        core::ptr::addr_of!(IID_IMoniker),
        ppmk as *mut LPVOID,
    )
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

    if FAILED(hr) {
        IMoniker_Release(pMoniker);
    }

    hr
}
