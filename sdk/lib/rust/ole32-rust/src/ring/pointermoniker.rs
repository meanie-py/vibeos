use crate::prelude::*;

// Defined by sibling ring modules, but ALSO (wrongly) declared as externs in
// ole32_exports.rs — the prelude glob makes the bare names ambiguous, so bind explicitly.
use crate::ring::antimoniker::CreateAntiMoniker;
use crate::ring::compositemoniker::CreateGenericComposite;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct PointerMonikerImpl {
    pub IMoniker_iface: IMoniker,
    /// reference counter for this object
    pub ref_: LONG,
    /// the object this moniker points at
    pub pObject: *mut IUnknown,
}

pub type PPointerMonikerImpl = *mut PointerMonikerImpl;

static VT_PointerMonikerImpl: IMonikerVtbl = IMonikerVtbl {
    QueryInterface: Some(PointerMonikerImpl_QueryInterface),
    AddRef: Some(PointerMonikerImpl_AddRef),
    Release: Some(PointerMonikerImpl_Release),
    GetClassID: Some(PointerMonikerImpl_GetClassID),
    IsDirty: Some(PointerMonikerImpl_IsDirty),
    Load: Some(PointerMonikerImpl_Load),
    Save: Some(PointerMonikerImpl_Save),
    GetSizeMax: Some(PointerMonikerImpl_GetSizeMax),
    BindToObject: Some(PointerMonikerImpl_BindToObject),
    BindToStorage: Some(PointerMonikerImpl_BindToStorage),
    Reduce: Some(PointerMonikerImpl_Reduce),
    ComposeWith: Some(PointerMonikerImpl_ComposeWith),
    Enum: Some(PointerMonikerImpl_Enum),
    IsEqual: Some(PointerMonikerImpl_IsEqual),
    Hash: Some(PointerMonikerImpl_Hash),
    IsRunning: Some(PointerMonikerImpl_IsRunning),
    GetTimeOfLastChange: Some(PointerMonikerImpl_GetTimeOfLastChange),
    Inverse: Some(PointerMonikerImpl_Inverse),
    CommonPrefixWith: Some(PointerMonikerImpl_CommonPrefixWith),
    RelativePathTo: Some(PointerMonikerImpl_RelativePathTo),
    GetDisplayName: Some(PointerMonikerImpl_GetDisplayName),
    ParseDisplayName: Some(PointerMonikerImpl_ParseDisplayName),
    IsSystemMoniker: Some(PointerMonikerImpl_IsSystemMoniker),
};

unsafe fn impl_from_IMoniker(iface: *mut IMoniker) -> *mut PointerMonikerImpl {
    CONTAINING_RECORD!(iface, PointerMonikerImpl, IMoniker_iface)
}

stdmethod! {
fn PointerMonikerImpl_QueryInterface(
    iface: *mut IMoniker,
    riid: *const IID,
    ppvObject: *mut LPVOID,
) -> HRESULT {
    let This = impl_from_IMoniker(iface);

    TRACE!(b"(%p,%s,%p)\n\0".as_ptr(), This, debugstr_guid(riid), ppvObject);

    /* Perform a sanity check on the parameters. */
    if This.is_null() || ppvObject.is_null() {
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
fn PointerMonikerImpl_AddRef(iface: *mut IMoniker) -> ULONG {
    let This = impl_from_IMoniker(iface);

    TRACE!(b"(%p)\n\0".as_ptr(), This);

    InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_)) as ULONG
}
}

stdmethod! {
fn PointerMonikerImpl_Release(iface: *mut IMoniker) -> ULONG {
    let This = impl_from_IMoniker(iface);

    TRACE!(b"(%p)\n\0".as_ptr(), This);

    let ref_ = InterlockedDecrement(core::ptr::addr_of_mut!((*This).ref_)) as ULONG;

    /* destroy the object if there are no more references on it */
    if ref_ == 0 {
        if !(*This).pObject.is_null() {
            IUnknown_Release((*This).pObject);
        }
        HeapFree(GetProcessHeap(), 0, This as LPVOID);
    }

    ref_
}
}

stdmethod! {
fn PointerMonikerImpl_GetClassID(iface: *mut IMoniker, pClassID: *mut CLSID) -> HRESULT {
    TRACE!(b"(%p,%p)\n\0".as_ptr(), iface, pClassID);

    if pClassID.is_null() {
        return E_POINTER;
    }

    *pClassID = CLSID_PointerMoniker;

    S_OK
}
}

stdmethod! {
fn PointerMonikerImpl_IsDirty(iface: *mut IMoniker) -> HRESULT {
    /* Note that the OLE-provided implementations of the IPersistStream::IsDirty
       method in the OLE-provided moniker interfaces always return S_FALSE because
       their internal state never changes. */

    TRACE!(b"(%p)\n\0".as_ptr(), iface);

    S_FALSE
}
}

stdmethod! {
fn PointerMonikerImpl_Load(iface: *mut IMoniker, pStm: *mut IStream) -> HRESULT {
    TRACE!(b"(%p)\n\0".as_ptr(), pStm);

    E_NOTIMPL
}
}

stdmethod! {
fn PointerMonikerImpl_Save(
    iface: *mut IMoniker,
    pStm: *mut IStream,
    fClearDirty: BOOL,
) -> HRESULT {
    TRACE!(b"(%p, %d)\n\0".as_ptr(), pStm, fClearDirty);

    E_NOTIMPL
}
}

stdmethod! {
fn PointerMonikerImpl_GetSizeMax(iface: *mut IMoniker, pcbSize: *mut ULARGE_INTEGER) -> HRESULT {
    TRACE!(b"(%p,%p)\n\0".as_ptr(), iface, pcbSize);

    if pcbSize.is_null() {
        return E_POINTER;
    }

    (*pcbSize).u.LowPart = 0;
    (*pcbSize).u.HighPart = 0;

    E_NOTIMPL
}
}

stdmethod! {
fn PointerMonikerImpl_BindToObject(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    riid: *const IID,
    ppvResult: *mut LPVOID,
) -> HRESULT {
    let This = impl_from_IMoniker(iface);

    TRACE!(
        b"(%p,%p,%p,%s,%p)\n\0".as_ptr(),
        iface,
        pbc,
        pmkToLeft,
        debugstr_guid(riid),
        ppvResult,
    );

    if (*This).pObject.is_null() {
        return E_UNEXPECTED;
    }

    IUnknown_QueryInterface((*This).pObject, riid, ppvResult)
}
}

stdmethod! {
fn PointerMonikerImpl_BindToStorage(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    riid: *const IID,
    ppvResult: *mut LPVOID,
) -> HRESULT {
    let This = impl_from_IMoniker(iface);

    TRACE!(
        b"(%p,%p,%p,%s,%p)\n\0".as_ptr(),
        iface,
        pbc,
        pmkToLeft,
        debugstr_guid(riid),
        ppvResult,
    );

    if (*This).pObject.is_null() {
        return E_UNEXPECTED;
    }

    IUnknown_QueryInterface((*This).pObject, riid, ppvResult)
}
}

stdmethod! {
fn PointerMonikerImpl_Reduce(
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

    PointerMonikerImpl_AddRef(iface);

    *ppmkReduced = iface;

    MK_S_REDUCED_TO_SELF
}
}

stdmethod! {
fn PointerMonikerImpl_ComposeWith(
    iface: *mut IMoniker,
    pmkRight: *mut IMoniker,
    fOnlyIfNotGeneric: BOOL,
    ppmkComposite: *mut *mut IMoniker,
) -> HRESULT {
    let mut res: HRESULT = S_OK;
    let mut mkSys: DWORD = 0;
    let mut mkSys2: DWORD = 0;
    let mut penumMk: *mut IEnumMoniker = core::ptr::null_mut();
    let mut pmostLeftMk: *mut IMoniker = core::ptr::null_mut();
    let mut tempMkComposite: *mut IMoniker;

    TRACE!(b"(%p,%d,%p)\n\0".as_ptr(), pmkRight, fOnlyIfNotGeneric, ppmkComposite);

    if ppmkComposite.is_null() || pmkRight.is_null() {
        return E_POINTER;
    }

    *ppmkComposite = core::ptr::null_mut();

    IMoniker_IsSystemMoniker(pmkRight, &mut mkSys);

    /* If pmkRight is an anti-moniker, the returned moniker is NULL */
    if mkSys == MKSYS_ANTIMONIKER {
        res
    }
    /* if pmkRight is a composite whose leftmost component is an anti-moniker,           */
    /* the returned moniker is the composite after the leftmost anti-moniker is removed. */
    else if mkSys == MKSYS_GENERICCOMPOSITE {
        res = IMoniker_Enum(pmkRight, TRUE as BOOL, &mut penumMk);

        if FAILED(res) {
            return res;
        }

        res = IEnumMoniker_Next(penumMk, 1, &mut pmostLeftMk, core::ptr::null_mut());

        IMoniker_IsSystemMoniker(pmostLeftMk, &mut mkSys2);

        if mkSys2 == MKSYS_ANTIMONIKER {
            IMoniker_Release(pmostLeftMk);

            tempMkComposite = iface;
            IMoniker_AddRef(iface);

            while IEnumMoniker_Next(penumMk, 1, &mut pmostLeftMk, core::ptr::null_mut()) == S_OK {
                res = CreateGenericComposite(tempMkComposite, pmostLeftMk, ppmkComposite);

                IMoniker_Release(tempMkComposite);
                IMoniker_Release(pmostLeftMk);

                tempMkComposite = *ppmkComposite;
                IMoniker_AddRef(tempMkComposite);
            }
            res
        } else {
            CreateGenericComposite(iface, pmkRight, ppmkComposite)
        }
    }
    /* If pmkRight is not an anti-moniker, the method combines the two monikers into a generic
       composite if fOnlyIfNotGeneric is FALSE; if fOnlyIfNotGeneric is TRUE, the method returns
       a NULL moniker and a return value of MK_E_NEEDGENERIC */
    else if fOnlyIfNotGeneric == 0 {
        CreateGenericComposite(iface, pmkRight, ppmkComposite)
    } else {
        MK_E_NEEDGENERIC
    }
}
}

stdmethod! {
fn PointerMonikerImpl_Enum(
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
fn PointerMonikerImpl_IsEqual(
    iface: *mut IMoniker,
    pmkOtherMoniker: *mut IMoniker,
) -> HRESULT {
    let This = impl_from_IMoniker(iface);
    let mut mkSys: DWORD = 0;

    TRACE!(b"(%p,%p)\n\0".as_ptr(), iface, pmkOtherMoniker);

    if pmkOtherMoniker.is_null() {
        return S_FALSE;
    }

    IMoniker_IsSystemMoniker(pmkOtherMoniker, &mut mkSys);

    if mkSys == MKSYS_POINTERMONIKER {
        let pOtherMoniker = impl_from_IMoniker(pmkOtherMoniker);
        if (*This).pObject == (*pOtherMoniker).pObject {
            S_OK
        } else {
            S_FALSE
        }
    } else {
        S_FALSE
    }
}
}

stdmethod! {
fn PointerMonikerImpl_Hash(iface: *mut IMoniker, pdwHash: *mut DWORD) -> HRESULT {
    let This = impl_from_IMoniker(iface);

    if pdwHash.is_null() {
        return E_POINTER;
    }

    *pdwHash = PtrToUlong((*This).pObject as *const core::ffi::c_void);

    S_OK
}
}

stdmethod! {
fn PointerMonikerImpl_IsRunning(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    pmkNewlyRunning: *mut IMoniker,
) -> HRESULT {
    TRACE!(
        b"(%p,%p,%p,%p)\n\0".as_ptr(),
        iface,
        pbc,
        pmkToLeft,
        pmkNewlyRunning,
    );

    S_OK
}
}

stdmethod! {
fn PointerMonikerImpl_GetTimeOfLastChange(
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
fn PointerMonikerImpl_Inverse(iface: *mut IMoniker, ppmk: *mut *mut IMoniker) -> HRESULT {
    TRACE!(b"(%p,%p)\n\0".as_ptr(), iface, ppmk);

    CreateAntiMoniker(ppmk)
}
}

stdmethod! {
fn PointerMonikerImpl_CommonPrefixWith(
    iface: *mut IMoniker,
    pmkOther: *mut IMoniker,
    ppmkPrefix: *mut *mut IMoniker,
) -> HRESULT {
    TRACE!(b"(%p, %p)\n\0".as_ptr(), pmkOther, ppmkPrefix);

    // pointermoniker.c (current): `if (!prefix || !other) return E_INVALIDARG;`
    if ppmkPrefix.is_null() || pmkOther.is_null() {
        return E_INVALIDARG;
    }

    *ppmkPrefix = core::ptr::null_mut();

    /* FAITHFUL to the C oracle (pointermoniker.c:422):
     *
     *     if (PointerMonikerImpl_IsEqual(iface, pmkOther))
     *
     * IsEqual returns an HRESULT, and C truthiness is "non-zero", so this branch is taken
     * when IsEqual returns S_FALSE (1) — i.e. when the monikers are NOT equal — and skipped
     * when it returns S_OK (0). That inversion is an UPSTREAM Wine/ReactOS bug, preserved
     * here verbatim so the Rust behaves identically to the C it replaces. Do NOT "fix" it
     * here without also fixing the C oracle; see the burn-down report.
     */
    if PointerMonikerImpl_IsEqual(iface, pmkOther) != 0 {
        IMoniker_AddRef(iface);

        *ppmkPrefix = iface;

        MK_S_US
    } else {
        MK_E_NOPREFIX
    }
}
}

stdmethod! {
fn PointerMonikerImpl_RelativePathTo(
    iface: *mut IMoniker,
    pmOther: *mut IMoniker,
    ppmkRelPath: *mut *mut IMoniker,
) -> HRESULT {
    TRACE!(b"(%p,%p,%p)\n\0".as_ptr(), iface, pmOther, ppmkRelPath);

    if ppmkRelPath.is_null() {
        return E_POINTER;
    }

    *ppmkRelPath = core::ptr::null_mut();

    E_NOTIMPL
}
}

stdmethod! {
fn PointerMonikerImpl_GetDisplayName(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    ppszDisplayName: *mut LPOLESTR,
) -> HRESULT {
    TRACE!(b"(%p,%p,%p,%p)\n\0".as_ptr(), iface, pbc, pmkToLeft, ppszDisplayName);

    if ppszDisplayName.is_null() {
        return E_POINTER;
    }

    *ppszDisplayName = core::ptr::null_mut();

    E_NOTIMPL
}
}

stdmethod! {
fn PointerMonikerImpl_ParseDisplayName(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    pszDisplayName: LPOLESTR,
    pchEaten: *mut ULONG,
    ppmkOut: *mut *mut IMoniker,
) -> HRESULT {
    let This = impl_from_IMoniker(iface);
    let mut pPDN: *mut IParseDisplayName = core::ptr::null_mut();

    TRACE!(
        b"(%p,%p,%p,%p,%p,%p)\n\0".as_ptr(),
        iface,
        pbc,
        pmkToLeft,
        pszDisplayName,
        pchEaten,
        ppmkOut,
    );

    if !pmkToLeft.is_null() {
        return MK_E_SYNTAX;
    }

    if (*This).pObject.is_null() {
        return E_UNEXPECTED;
    }

    let hr = IUnknown_QueryInterface(
        (*This).pObject,
        core::ptr::addr_of!(IID_IParseDisplayName),
        core::ptr::addr_of_mut!(pPDN) as *mut LPVOID,
    );
    if FAILED(hr) {
        return hr;
    }

    let hr = IParseDisplayName_ParseDisplayName(pPDN, pbc, pszDisplayName, pchEaten, ppmkOut);
    IParseDisplayName_Release(pPDN);

    hr
}
}

stdmethod! {
fn PointerMonikerImpl_IsSystemMoniker(iface: *mut IMoniker, pwdMksys: *mut DWORD) -> HRESULT {
    TRACE!(b"(%p,%p)\n\0".as_ptr(), iface, pwdMksys);

    if pwdMksys.is_null() {
        return E_POINTER;
    }

    *pwdMksys = MKSYS_POINTERMONIKER;

    S_OK
}
}

unsafe fn PointerMonikerImpl_Construct(This: *mut PointerMonikerImpl, punk: *mut IUnknown) {
    TRACE!(b"(%p)\n\0".as_ptr(), This);

    /* Initialize the virtual function table. */
    (*This).IMoniker_iface.lpVtbl =
        core::ptr::addr_of!(VT_PointerMonikerImpl) as *mut IMonikerVtbl;
    (*This).ref_ = 1;
    if !punk.is_null() {
        IUnknown_AddRef(punk);
    }
    (*This).pObject = punk;
}

/***********************************************************************
 *           CreatePointerMoniker (OLE32.@)
 *
 * Creates a moniker which represents a pointer.
 */
#[no_mangle]
pub unsafe extern "system" fn CreatePointerMoniker(
    punk: LPUNKNOWN,
    ppmk: *mut LPMONIKER,
) -> HRESULT {
    TRACE!(b"(%p, %p)\n\0".as_ptr(), punk, ppmk);

    if ppmk.is_null() {
        return E_INVALIDARG;
    }

    let This = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<PointerMonikerImpl>() as SIZE_T,
    ) as *mut PointerMonikerImpl;
    if This.is_null() {
        *ppmk = core::ptr::null_mut();
        return E_OUTOFMEMORY;
    }

    PointerMonikerImpl_Construct(This, punk);
    *ppmk = core::ptr::addr_of_mut!((*This).IMoniker_iface);

    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn PointerMoniker_CreateInstance(
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

    let hr = CreatePointerMoniker(core::ptr::null_mut(), &mut pMoniker);
    if FAILED(hr) {
        return hr;
    }

    let hr = IMoniker_QueryInterface(pMoniker, riid, ppv);

    if FAILED(hr) {
        IMoniker_Release(pMoniker);
    }

    hr
}
