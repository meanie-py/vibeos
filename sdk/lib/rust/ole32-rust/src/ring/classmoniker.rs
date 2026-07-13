use crate::prelude::*;

// These two are DEFINED by sibling ring modules. They are ALSO (wrongly) declared as
// externs in ole32_exports.rs, so the prelude glob makes the bare name ambiguous —
// name them explicitly so we bind to the Rust definition.
use crate::ring::antimoniker::CreateAntiMoniker;
use crate::ring::compositemoniker::{CreateGenericComposite, MonikerCommonPrefixWith};

extern "system" {
    // compobj.c (Rust, #[no_mangle]) — an ole32 export, so WINAPI/stdcall.
    // Belongs in crate::ole32_exports; declared here until it is added there.
    fn CoGetClassObject(
        rclsid: REFCLSID,
        dwClsContext: DWORD,
        pvReserved: LPVOID,
        riid: REFIID,
        ppv: *mut LPVOID,
    ) -> HRESULT;
}

extern "C" {
    // uuid.lib data symbol. Belongs in crate::iids; declared here until added there.
    static IID_IClassActivator: GUID;
}

// ---- file-scope preamble (translated) ----
pub const CHARS_IN_GUID: i32 = 39;

#[repr(C)]
pub struct ClassMoniker {
    pub IMoniker_iface: IMoniker,
    pub IROTData_iface: IROTData,
    pub ref_: LONG,
    pub clsid: CLSID,
    pub pMarshal: *mut IUnknown,
}

pub type PClassMoniker = *mut ClassMoniker;

static ClassMonikerVtbl: IMonikerVtbl = IMonikerVtbl {
    QueryInterface: Some(ClassMoniker_QueryInterface),
    AddRef: Some(ClassMoniker_AddRef),
    Release: Some(ClassMoniker_Release),
    GetClassID: Some(ClassMoniker_GetClassID),
    IsDirty: Some(ClassMoniker_IsDirty),
    Load: Some(ClassMoniker_Load),
    Save: Some(ClassMoniker_Save),
    GetSizeMax: Some(ClassMoniker_GetSizeMax),
    BindToObject: Some(ClassMoniker_BindToObject),
    BindToStorage: Some(ClassMoniker_BindToStorage),
    Reduce: Some(ClassMoniker_Reduce),
    ComposeWith: Some(ClassMoniker_ComposeWith),
    Enum: Some(ClassMoniker_Enum),
    IsEqual: Some(ClassMoniker_IsEqual),
    Hash: Some(ClassMoniker_Hash),
    IsRunning: Some(ClassMoniker_IsRunning),
    GetTimeOfLastChange: Some(ClassMoniker_GetTimeOfLastChange),
    Inverse: Some(ClassMoniker_Inverse),
    CommonPrefixWith: Some(ClassMoniker_CommonPrefixWith),
    RelativePathTo: Some(ClassMoniker_RelativePathTo),
    GetDisplayName: Some(ClassMoniker_GetDisplayName),
    ParseDisplayName: Some(ClassMoniker_ParseDisplayName),
    IsSystemMoniker: Some(ClassMoniker_IsSystemMoniker),
};

static ROTDataVtbl: IROTDataVtbl = IROTDataVtbl {
    QueryInterface: Some(ClassMonikerROTData_QueryInterface),
    AddRef: Some(ClassMonikerROTData_AddRef),
    Release: Some(ClassMonikerROTData_Release),
    GetComparisonData: Some(ClassMonikerROTData_GetComparisonData),
};

unsafe fn impl_from_IMoniker(iface: *mut IMoniker) -> *mut ClassMoniker {
    CONTAINING_RECORD!(iface, ClassMoniker, IMoniker_iface)
}

unsafe fn impl_from_IROTData(iface: *mut IROTData) -> *mut ClassMoniker {
    CONTAINING_RECORD!(iface, ClassMoniker, IROTData_iface)
}

stdmethod! {
fn ClassMoniker_QueryInterface(
    iface: *mut IMoniker,
    riid: *const IID,
    ppvObject: *mut LPVOID,
) -> HRESULT {
    let This = impl_from_IMoniker(iface);

    TRACE!(b"(%p,%s,%p)\n\0".as_ptr(), This, debugstr_guid(riid), ppvObject);

    /* Perform a sanity check on the parameters. */
    if ppvObject.is_null() {
        return E_POINTER;
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
        let mut hr = S_OK;
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

    /* QueryInterface always increases the reference count by one when successful */
    IMoniker_AddRef(iface);

    S_OK
}
}

stdmethod! {
fn ClassMoniker_AddRef(iface: *mut IMoniker) -> ULONG {
    let This = impl_from_IMoniker(iface);

    TRACE!(b"(%p)\n\0".as_ptr(), This);

    InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_)) as ULONG
}
}

stdmethod! {
fn ClassMoniker_Release(iface: *mut IMoniker) -> ULONG {
    let This = impl_from_IMoniker(iface);

    TRACE!(b"(%p)\n\0".as_ptr(), This);

    let ref_ = InterlockedDecrement(core::ptr::addr_of_mut!((*This).ref_)) as ULONG;

    /* destroy the object if there are no more references to it */
    if ref_ == 0 {
        if !(*This).pMarshal.is_null() {
            IUnknown_Release((*This).pMarshal);
        }
        HeapFree(GetProcessHeap(), 0, This as LPVOID);
    }

    ref_
}
}

stdmethod! {
fn ClassMoniker_GetClassID(iface: *mut IMoniker, pClassID: *mut CLSID) -> HRESULT {
    TRACE!(b"(%p,%p),stub!\n\0".as_ptr(), iface, pClassID);

    if pClassID.is_null() {
        return E_POINTER;
    }

    *pClassID = CLSID_ClassMoniker;

    S_OK
}
}

stdmethod! {
fn ClassMoniker_IsDirty(iface: *mut IMoniker) -> HRESULT {
    /* Note that the OLE-provided implementations of the IPersistStream::IsDirty
       method in the OLE-provided moniker interfaces always return S_FALSE because
       their internal state never changes. */

    TRACE!(b"(%p)\n\0".as_ptr(), iface);

    S_FALSE
}
}

stdmethod! {
fn ClassMoniker_Load(iface: *mut IMoniker, pStm: *mut IStream) -> HRESULT {
    let This = impl_from_IMoniker(iface);
    let mut zero: DWORD = 0;

    TRACE!(b"(%p)\n\0".as_ptr(), pStm);

    let mut hr = IStream_Read(
        pStm,
        core::ptr::addr_of_mut!((*This).clsid) as LPVOID,
        core::mem::size_of::<CLSID>() as ULONG,
        core::ptr::null_mut(),
    );
    if hr != S_OK {
        return STG_E_READFAULT;
    }

    hr = IStream_Read(
        pStm,
        core::ptr::addr_of_mut!(zero) as LPVOID,
        core::mem::size_of::<DWORD>() as ULONG,
        core::ptr::null_mut(),
    );
    if hr != S_OK || zero != 0 {
        return STG_E_READFAULT;
    }

    S_OK
}
}

stdmethod! {
fn ClassMoniker_Save(iface: *mut IMoniker, pStm: *mut IStream, fClearDirty: BOOL) -> HRESULT {
    let This = impl_from_IMoniker(iface);
    let zero: DWORD = 0;

    TRACE!(b"(%p, %d)\n\0".as_ptr(), pStm, fClearDirty);

    let hr = IStream_Write(
        pStm,
        core::ptr::addr_of!((*This).clsid) as *const core::ffi::c_void,
        core::mem::size_of::<CLSID>() as ULONG,
        core::ptr::null_mut(),
    );
    if FAILED(hr) {
        return hr;
    }

    IStream_Write(
        pStm,
        core::ptr::addr_of!(zero) as *const core::ffi::c_void,
        core::mem::size_of::<DWORD>() as ULONG,
        core::ptr::null_mut(),
    )
}
}

stdmethod! {
fn ClassMoniker_GetSizeMax(iface: *mut IMoniker, pcbSize: *mut ULARGE_INTEGER) -> HRESULT {
    TRACE!(b"(%p)\n\0".as_ptr(), pcbSize);

    (*pcbSize).QuadPart =
        (core::mem::size_of::<CLSID>() + core::mem::size_of::<DWORD>()) as ULONGLONG;

    S_OK
}
}

stdmethod! {
fn ClassMoniker_BindToObject(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    riid: *const IID,
    ppvResult: *mut LPVOID,
) -> HRESULT {
    let This = impl_from_IMoniker(iface);
    let mut bindopts: BIND_OPTS2 = core::mem::zeroed();
    let mut pActivator: *mut IClassActivator = core::ptr::null_mut();

    TRACE!(
        b"(%p, %p, %s, %p)\n\0".as_ptr(),
        pbc,
        pmkToLeft,
        debugstr_guid(riid),
        ppvResult,
    );

    bindopts.cbStruct = core::mem::size_of::<BIND_OPTS2>() as DWORD;
    IBindCtx_GetBindOptions(pbc, core::ptr::addr_of_mut!(bindopts) as *mut BIND_OPTS);

    if pmkToLeft.is_null() {
        CoGetClassObject(
            core::ptr::addr_of!((*This).clsid),
            bindopts.dwClassContext,
            core::ptr::null_mut(),
            riid,
            ppvResult,
        )
    } else {
        let mut hr = IMoniker_BindToObject(
            pmkToLeft,
            pbc,
            core::ptr::null_mut(),
            core::ptr::addr_of!(IID_IClassActivator),
            core::ptr::addr_of_mut!(pActivator) as *mut LPVOID,
        );
        if FAILED(hr) {
            return hr;
        }

        hr = IClassActivator_GetClassObject(
            pActivator,
            core::ptr::addr_of!((*This).clsid),
            bindopts.dwClassContext,
            bindopts.locale,
            riid,
            ppvResult,
        );

        IClassActivator_Release(pActivator);

        hr
    }
}
}

stdmethod! {
fn ClassMoniker_BindToStorage(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    riid: *const IID,
    ppvResult: *mut LPVOID,
) -> HRESULT {
    TRACE!(
        b"(%p, %p, %s, %p)\n\0".as_ptr(),
        pbc,
        pmkToLeft,
        debugstr_guid(riid),
        ppvResult,
    );

    IMoniker_BindToObject(iface, pbc, pmkToLeft, riid, ppvResult)
}
}

stdmethod! {
fn ClassMoniker_Reduce(
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

    IMoniker_AddRef(iface);

    *ppmkReduced = iface;

    MK_S_REDUCED_TO_SELF
}
}

stdmethod! {
fn ClassMoniker_ComposeWith(
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
fn ClassMoniker_Enum(
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
fn ClassMoniker_IsEqual(iface: *mut IMoniker, pmkOtherMoniker: *mut IMoniker) -> HRESULT {
    let mut clsid: CLSID = core::mem::zeroed();
    let mut dispName1: LPOLESTR = core::ptr::null_mut();
    let mut dispName2: LPOLESTR = core::ptr::null_mut();
    let mut bind: *mut IBindCtx = core::ptr::null_mut();
    let mut res: HRESULT = S_FALSE;

    TRACE!(b"(%p,%p)\n\0".as_ptr(), iface, pmkOtherMoniker);

    if pmkOtherMoniker.is_null() {
        return S_FALSE;
    }

    /* check if both are ClassMoniker */
    if FAILED(IMoniker_GetClassID(pmkOtherMoniker, &mut clsid)) {
        return S_FALSE;
    }
    if !IsEqualCLSID(&clsid, core::ptr::addr_of!(CLSID_ClassMoniker)) {
        return S_FALSE;
    }

    /* check if both displaynames are the same */
    res = CreateBindCtx(0, &mut bind);
    if SUCCEEDED(res) {
        if SUCCEEDED(IMoniker_GetDisplayName(iface, bind, core::ptr::null_mut(), &mut dispName1)) {
            if SUCCEEDED(IMoniker_GetDisplayName(
                pmkOtherMoniker,
                bind,
                core::ptr::null_mut(),
                &mut dispName2,
            )) {
                if wcscmp(dispName1, dispName2) == 0 {
                    res = S_OK;
                }
                CoTaskMemFree(dispName2 as LPVOID);
            }
            CoTaskMemFree(dispName1 as LPVOID);
        }
    }

    res
}
}

stdmethod! {
fn ClassMoniker_Hash(iface: *mut IMoniker, pdwHash: *mut DWORD) -> HRESULT {
    let This = impl_from_IMoniker(iface);

    TRACE!(b"(%p)\n\0".as_ptr(), pdwHash);

    *pdwHash = (*This).clsid.Data1;

    S_OK
}
}

stdmethod! {
fn ClassMoniker_IsRunning(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    pmkNewlyRunning: *mut IMoniker,
) -> HRESULT {
    TRACE!(b"(%p, %p, %p)\n\0".as_ptr(), pbc, pmkToLeft, pmkNewlyRunning);

    /* as in native */
    E_NOTIMPL
}
}

stdmethod! {
fn ClassMoniker_GetTimeOfLastChange(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    pItemTime: *mut FILETIME,
) -> HRESULT {
    TRACE!(b"(%p, %p, %p)\n\0".as_ptr(), pbc, pmkToLeft, pItemTime);

    MK_E_UNAVAILABLE
}
}

stdmethod! {
fn ClassMoniker_Inverse(iface: *mut IMoniker, ppmk: *mut *mut IMoniker) -> HRESULT {
    TRACE!(b"(%p)\n\0".as_ptr(), ppmk);

    if ppmk.is_null() {
        return E_POINTER;
    }

    CreateAntiMoniker(ppmk)
}
}

stdmethod! {
fn ClassMoniker_CommonPrefixWith(
    iface: *mut IMoniker,
    pmkOther: *mut IMoniker,
    ppmkPrefix: *mut *mut IMoniker,
) -> HRESULT {
    let mut mkSys: DWORD = 0;

    TRACE!(b"(%p, %p)\n\0".as_ptr(), pmkOther, ppmkPrefix);

    *ppmkPrefix = core::ptr::null_mut();

    IMoniker_IsSystemMoniker(pmkOther, &mut mkSys);

    /* If the other moniker is a class moniker that is equal to this moniker, this method sets
       *ppmkPrefix to this moniker and returns MK_S_US */
    if mkSys == MKSYS_CLASSMONIKER {
        if IMoniker_IsEqual(iface, pmkOther) == S_OK {
            *ppmkPrefix = iface;

            IMoniker_AddRef(iface);

            MK_S_US
        } else {
            MK_E_NOPREFIX
        }
    } else {
        /* otherwise, the method calls the MonikerCommonPrefixWith function. This function
           correctly handles the case where the other moniker is a generic composite. */
        MonikerCommonPrefixWith(iface, pmkOther, ppmkPrefix)
    }
}
}

stdmethod! {
fn ClassMoniker_RelativePathTo(
    iface: *mut IMoniker,
    pmOther: *mut IMoniker,
    ppmkRelPath: *mut *mut IMoniker,
) -> HRESULT {
    TRACE!(b"(%p, %p)\n\0".as_ptr(), pmOther, ppmkRelPath);

    if ppmkRelPath.is_null() {
        return E_POINTER;
    }

    *ppmkRelPath = core::ptr::null_mut();

    MK_E_NOTBINDABLE
}
}

stdmethod! {
fn ClassMoniker_GetDisplayName(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    ppszDisplayName: *mut LPOLESTR,
) -> HRESULT {
    let This = impl_from_IMoniker(iface);
    static wszClsidPrefix: [WCHAR; 7] = [
        b'c' as WCHAR,
        b'l' as WCHAR,
        b's' as WCHAR,
        b'i' as WCHAR,
        b'd' as WCHAR,
        b':' as WCHAR,
        0,
    ];

    TRACE!(b"(%p, %p, %p)\n\0".as_ptr(), pbc, pmkToLeft, ppszDisplayName);

    if ppszDisplayName.is_null() {
        return E_POINTER;
    }

    if !pmkToLeft.is_null() {
        return E_INVALIDARG;
    }

    /* ARRAY_SIZE(wszClsidPrefix) — element count, NUL included */
    let array_size_prefix: usize = wszClsidPrefix.len();
    let chars_in_guid: usize = CHARS_IN_GUID as usize;

    let total_size = core::mem::size_of_val(&wszClsidPrefix)
        + (chars_in_guid - 2) * core::mem::size_of::<WCHAR>();
    *ppszDisplayName = CoTaskMemAlloc(total_size as SIZE_T) as LPOLESTR;

    StringFromGUID2(
        core::ptr::addr_of!((*This).clsid),
        (*ppszDisplayName).add(array_size_prefix - 2),
        CHARS_IN_GUID as c_int,
    );

    /* note: this overwrites the opening curly bracket of the CLSID string generated above */
    memcpy(
        *ppszDisplayName as *mut core::ffi::c_void,
        wszClsidPrefix.as_ptr() as *const core::ffi::c_void,
        core::mem::size_of_val(&wszClsidPrefix) - core::mem::size_of::<WCHAR>(),
    );

    /* note: this overwrites the closing curly bracket of the CLSID string generated above */
    *(*ppszDisplayName).add(array_size_prefix - 2 + chars_in_guid - 2) = b':' as WCHAR;
    *(*ppszDisplayName).add(array_size_prefix - 2 + chars_in_guid - 1) = 0;

    TRACE!(b"string is %s\n\0".as_ptr(), debugstr_w(*ppszDisplayName));

    S_OK
}
}

stdmethod! {
fn ClassMoniker_ParseDisplayName(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    pszDisplayName: LPOLESTR,
    pchEaten: *mut ULONG,
    ppmkOut: *mut *mut IMoniker,
) -> HRESULT {
    FIXME!(
        b"(%p, %p, %s, %p, %p)\n\0".as_ptr(),
        pbc,
        pmkToLeft,
        debugstr_w(pszDisplayName),
        pchEaten,
        ppmkOut,
    );

    E_NOTIMPL
}
}

stdmethod! {
fn ClassMoniker_IsSystemMoniker(iface: *mut IMoniker, pwdMksys: *mut DWORD) -> HRESULT {
    TRACE!(b"(%p,%p)\n\0".as_ptr(), iface, pwdMksys);

    if pwdMksys.is_null() {
        return E_POINTER;
    }

    *pwdMksys = MKSYS_CLASSMONIKER;

    S_OK
}
}

stdmethod! {
fn ClassMonikerROTData_QueryInterface(
    iface: *mut IROTData,
    riid: *const IID,
    ppvObject: *mut LPVOID,
) -> HRESULT {
    let This = impl_from_IROTData(iface);

    TRACE!(b"(%p, %s, %p)\n\0".as_ptr(), iface, debugstr_guid(riid), ppvObject);

    IMoniker_QueryInterface(
        core::ptr::addr_of_mut!((*This).IMoniker_iface),
        riid,
        ppvObject,
    )
}
}

stdmethod! {
fn ClassMonikerROTData_AddRef(iface: *mut IROTData) -> ULONG {
    let This = impl_from_IROTData(iface);

    TRACE!(b"(%p)\n\0".as_ptr(), iface);

    IMoniker_AddRef(core::ptr::addr_of_mut!((*This).IMoniker_iface))
}
}

stdmethod! {
fn ClassMonikerROTData_Release(iface: *mut IROTData) -> ULONG {
    let This = impl_from_IROTData(iface);

    TRACE!(b"(%p)\n\0".as_ptr(), iface);

    IMoniker_Release(core::ptr::addr_of_mut!((*This).IMoniker_iface))
}
}

stdmethod! {
fn ClassMonikerROTData_GetComparisonData(
    iface: *mut IROTData,
    pbData: *mut byte,
    cbMax: ULONG,
    pcbData: *mut ULONG,
) -> HRESULT {
    let This = impl_from_IROTData(iface);

    TRACE!(b"(%p, %u, %p)\n\0".as_ptr(), pbData, cbMax, pcbData);

    *pcbData = (2 * core::mem::size_of::<CLSID>()) as ULONG;
    if cbMax < *pcbData {
        return E_OUTOFMEMORY;
    }

    /* write CLSID of the moniker */
    memcpy(
        pbData as *mut core::ffi::c_void,
        core::ptr::addr_of!(CLSID_ClassMoniker) as *const core::ffi::c_void,
        core::mem::size_of::<CLSID>(),
    );
    /* write CLSID the moniker represents */
    memcpy(
        pbData.add(core::mem::size_of::<CLSID>()) as *mut core::ffi::c_void,
        core::ptr::addr_of!((*This).clsid) as *const core::ffi::c_void,
        core::mem::size_of::<CLSID>(),
    );

    S_OK
}
}

unsafe fn ClassMoniker_Construct(This: *mut ClassMoniker, rclsid: REFCLSID) -> HRESULT {
    TRACE!(b"(%p,%s)\n\0".as_ptr(), This, debugstr_guid(rclsid));

    /* Initialize the virtual function table. */
    (*This).IMoniker_iface.lpVtbl =
        core::ptr::addr_of!(ClassMonikerVtbl) as *mut IMonikerVtbl;
    (*This).IROTData_iface.lpVtbl =
        core::ptr::addr_of!(ROTDataVtbl) as *mut IROTDataVtbl;
    (*This).ref_ = 0;
    (*This).clsid = *rclsid;
    (*This).pMarshal = core::ptr::null_mut();

    S_OK
}

/******************************************************************************
 *        CreateClassMoniker	[OLE32.@]
 ******************************************************************************/
#[no_mangle]
pub unsafe extern "system" fn CreateClassMoniker(
    rclsid: REFCLSID,
    ppmk: *mut *mut IMoniker,
) -> HRESULT {
    TRACE!(b"(%s,%p)\n\0".as_ptr(), debugstr_guid(rclsid), ppmk);

    let newClassMoniker = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<ClassMoniker>() as SIZE_T,
    ) as *mut ClassMoniker;

    if newClassMoniker.is_null() {
        return STG_E_INSUFFICIENTMEMORY;
    }

    let hr = ClassMoniker_Construct(newClassMoniker, rclsid);

    if FAILED(hr) {
        HeapFree(GetProcessHeap(), 0, newClassMoniker as LPVOID);
        return hr;
    }

    ClassMoniker_QueryInterface(
        core::ptr::addr_of_mut!((*newClassMoniker).IMoniker_iface),
        core::ptr::addr_of!(IID_IMoniker),
        ppmk as *mut LPVOID,
    )
}

#[no_mangle]
pub unsafe extern "C" fn ClassMoniker_CreateFromDisplayName(
    pbc: *mut IBindCtx,
    szDisplayName: LPCOLESTR,
    pchEaten: LPDWORD,
    ppmk: *mut *mut IMoniker,
) -> HRESULT {
    let mut s: LPCWSTR = wcschr(szDisplayName, b':' as WCHAR);
    let mut end: LPCWSTR;
    let mut clsid: CLSID = core::mem::zeroed();
    let mut table: [u8; 256] = [0; 256];
    let mut i: isize;

    if s.is_null() {
        return MK_E_SYNTAX;
    }

    s = s.add(1);

    end = s;
    while *end != 0 && *end != b':' as WCHAR {
        end = end.add(1);
    }

    /* NOTE: C pointer arithmetic on WCHAR* — the difference is in CHARACTERS, not bytes. */
    let span: isize = end.offset_from(s);

    TRACE!(b"parsing %s\n\0".as_ptr(), debugstr_wn(s, span));

    /* validate the CLSID string */
    if *s == b'{' as WCHAR {
        if span != 38 || *s.add(37) != b'}' as WCHAR {
            return MK_E_SYNTAX;
        }
        s = s.add(1);
    } else if span != 36 {
        return MK_E_SYNTAX;
    }

    i = 0;
    while i < 36 {
        if i == 8 || i == 13 || i == 18 || i == 23 {
            if *s.offset(i) != b'-' as WCHAR {
                return MK_E_SYNTAX;
            }
            i += 1;
            continue;
        }
        let c = *s.offset(i);
        if !((c >= b'0' as WCHAR && c <= b'9' as WCHAR)
            || (c >= b'a' as WCHAR && c <= b'f' as WCHAR)
            || (c >= b'A' as WCHAR && c <= b'F' as WCHAR))
        {
            return MK_E_SYNTAX;
        }
        i += 1;
    }

    /* quick lookup table */
    memset(table.as_mut_ptr() as *mut core::ffi::c_void, 0, 256);

    i = 0;
    while i < 10 {
        table[(b'0' as isize + i) as usize] = i as u8;
        i += 1;
    }
    i = 0;
    while i < 6 {
        table[(b'A' as isize + i) as usize] = (i + 10) as u8;
        table[(b'a' as isize + i) as usize] = (i + 10) as u8;
        i += 1;
    }

    /* in form XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX */
    let nib = |n: isize| -> u32 { table[*s.offset(n) as usize] as u32 };

    clsid.Data1 = (nib(0) << 28)
        | (nib(1) << 24)
        | (nib(2) << 20)
        | (nib(3) << 16)
        | (nib(4) << 12)
        | (nib(5) << 8)
        | (nib(6) << 4)
        | nib(7);
    clsid.Data2 = ((nib(9) << 12) | (nib(10) << 8) | (nib(11) << 4) | nib(12)) as u16;
    clsid.Data3 = ((nib(14) << 12) | (nib(15) << 8) | (nib(16) << 4) | nib(17)) as u16;

    /* these are just sequential bytes */
    clsid.Data4[0] = ((nib(19) << 4) | nib(20)) as u8;
    clsid.Data4[1] = ((nib(21) << 4) | nib(22)) as u8;
    clsid.Data4[2] = ((nib(24) << 4) | nib(25)) as u8;
    clsid.Data4[3] = ((nib(26) << 4) | nib(27)) as u8;
    clsid.Data4[4] = ((nib(28) << 4) | nib(29)) as u8;
    clsid.Data4[5] = ((nib(30) << 4) | nib(31)) as u8;
    clsid.Data4[6] = ((nib(32) << 4) | nib(33)) as u8;
    clsid.Data4[7] = ((nib(34) << 4) | nib(35)) as u8;

    let hr = CreateClassMoniker(&clsid, ppmk);
    if SUCCEEDED(hr) {
        /* character count, NOT byte count — C did WCHAR* - WCHAR* */
        let stop: LPCWSTR = if *end == b':' as WCHAR { end.add(1) } else { end };
        *pchEaten = stop.offset_from(szDisplayName) as DWORD;
    }
    hr
}

#[no_mangle]
pub unsafe extern "system" fn ClassMoniker_CreateInstance(
    iface: *mut IClassFactory,
    pUnk: *mut IUnknown,
    riid: REFIID,
    ppv: *mut LPVOID,
) -> HRESULT {
    let mut pmk: *mut IMoniker = core::ptr::null_mut();

    TRACE!(b"(%p, %s, %p)\n\0".as_ptr(), pUnk, debugstr_guid(riid), ppv);

    *ppv = core::ptr::null_mut();

    if !pUnk.is_null() {
        return CLASS_E_NOAGGREGATION;
    }

    let hr = CreateClassMoniker(core::ptr::addr_of!(CLSID_NULL), &mut pmk);
    if FAILED(hr) {
        return hr;
    }

    let hr = IMoniker_QueryInterface(pmk, riid, ppv);
    IMoniker_Release(pmk);

    hr
}
