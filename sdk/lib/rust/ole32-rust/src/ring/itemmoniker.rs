use crate::prelude::*;

// Defined by sibling ring modules, but ALSO (wrongly) declared as externs in
// ole32_exports.rs — the prelude glob makes the bare names ambiguous, so bind explicitly.
use crate::ring::antimoniker::CreateAntiMoniker;
use crate::ring::compositemoniker::{CreateGenericComposite, MonikerCommonPrefixWith};

extern "C" {
    // uuid.lib data symbol. Belongs in crate::iids; declared here until added there.
    static IID_IOleItemContainer: IID;
}

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct ItemMonikerImpl {
    /// VTable relative to the IMoniker interface.
    pub IMoniker_iface: IMoniker,
    /// VTable relative to the IROTData interface.
    pub IROTData_iface: IROTData,
    pub ref_: LONG,
    /// item name identified by this ItemMoniker
    pub itemName: LPOLESTR,
    /// Delimiter string
    pub itemDelimiter: LPOLESTR,
    /// custom marshaler
    pub pMarshal: *mut IUnknown,
}

pub type PItemMonikerImpl = *mut ItemMonikerImpl;

static VT_ItemMonikerImpl: IMonikerVtbl = IMonikerVtbl {
    QueryInterface: Some(ItemMonikerImpl_QueryInterface),
    AddRef: Some(ItemMonikerImpl_AddRef),
    Release: Some(ItemMonikerImpl_Release),
    GetClassID: Some(ItemMonikerImpl_GetClassID),
    IsDirty: Some(ItemMonikerImpl_IsDirty),
    Load: Some(ItemMonikerImpl_Load),
    Save: Some(ItemMonikerImpl_Save),
    GetSizeMax: Some(ItemMonikerImpl_GetSizeMax),
    BindToObject: Some(ItemMonikerImpl_BindToObject),
    BindToStorage: Some(ItemMonikerImpl_BindToStorage),
    Reduce: Some(ItemMonikerImpl_Reduce),
    ComposeWith: Some(ItemMonikerImpl_ComposeWith),
    Enum: Some(ItemMonikerImpl_Enum),
    IsEqual: Some(ItemMonikerImpl_IsEqual),
    Hash: Some(ItemMonikerImpl_Hash),
    IsRunning: Some(ItemMonikerImpl_IsRunning),
    GetTimeOfLastChange: Some(ItemMonikerImpl_GetTimeOfLastChange),
    Inverse: Some(ItemMonikerImpl_Inverse),
    CommonPrefixWith: Some(ItemMonikerImpl_CommonPrefixWith),
    RelativePathTo: Some(ItemMonikerImpl_RelativePathTo),
    GetDisplayName: Some(ItemMonikerImpl_GetDisplayName),
    ParseDisplayName: Some(ItemMonikerImpl_ParseDisplayName),
    IsSystemMoniker: Some(ItemMonikerImpl_IsSystemMoniker),
};

static VT_ROTDataImpl: IROTDataVtbl = IROTDataVtbl {
    QueryInterface: Some(ItemMonikerROTDataImpl_QueryInterface),
    AddRef: Some(ItemMonikerROTDataImpl_AddRef),
    Release: Some(ItemMonikerROTDataImpl_Release),
    GetComparisonData: Some(ItemMonikerROTDataImpl_GetComparisonData),
};

unsafe fn impl_from_IMoniker(iface: *mut IMoniker) -> *mut ItemMonikerImpl {
    CONTAINING_RECORD!(iface, ItemMonikerImpl, IMoniker_iface)
}

unsafe fn impl_from_IROTData(iface: *mut IROTData) -> *mut ItemMonikerImpl {
    CONTAINING_RECORD!(iface, ItemMonikerImpl, IROTData_iface)
}

stdmethod! {
fn ItemMonikerImpl_QueryInterface(
    iface: *mut IMoniker,
    riid: *const IID,
    ppvObject: *mut LPVOID,
) -> HRESULT {
    let This = impl_from_IMoniker(iface);

    TRACE!(b"(%p,%s,%p)\n\0".as_ptr(), This, debugstr_guid(riid), ppvObject);

    if ppvObject.is_null() {
        return E_INVALIDARG;
    }

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
    } else {
        *ppvObject = core::ptr::null_mut();
        return E_NOINTERFACE;
    }

    IMoniker_AddRef(iface);

    S_OK
}
}

stdmethod! {
fn ItemMonikerImpl_AddRef(iface: *mut IMoniker) -> ULONG {
    let This = impl_from_IMoniker(iface);

    TRACE!(b"(%p)\n\0".as_ptr(), This);

    InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_)) as ULONG
}
}

stdmethod! {
fn ItemMonikerImpl_Release(iface: *mut IMoniker) -> ULONG {
    let This = impl_from_IMoniker(iface);

    TRACE!(b"(%p)\n\0".as_ptr(), This);

    let ref_ = InterlockedDecrement(core::ptr::addr_of_mut!((*This).ref_)) as ULONG;

    /* destroy the object if there are no more references to it */
    if ref_ == 0 {
        ItemMonikerImpl_Destroy(This);
    }

    ref_
}
}

stdmethod! {
fn ItemMonikerImpl_GetClassID(iface: *mut IMoniker, pClassID: *mut CLSID) -> HRESULT {
    TRACE!(b"(%p,%p)\n\0".as_ptr(), iface, pClassID);

    if pClassID.is_null() {
        return E_POINTER;
    }

    *pClassID = CLSID_ItemMoniker;

    S_OK
}
}

stdmethod! {
fn ItemMonikerImpl_IsDirty(iface: *mut IMoniker) -> HRESULT {
    /* Note that the OLE-provided implementations of the IPersistStream::IsDirty
       method in the OLE-provided moniker interfaces always return S_FALSE because
       their internal state never changes. */

    TRACE!(b"(%p)\n\0".as_ptr(), iface);

    S_FALSE
}
}

stdmethod! {
fn ItemMonikerImpl_Load(iface: *mut IMoniker, pStm: *mut IStream) -> HRESULT {
    let This = impl_from_IMoniker(iface);
    let mut delimiterLength: DWORD = 0;
    let mut nameLength: DWORD = 0;
    let mut bread: ULONG = 0;

    TRACE!(b"\n\0".as_ptr());

    /* for more details about data read by this function see comments of
       ItemMonikerImpl_Save function */

    /* read item delimiter string length + 1 */
    let mut res = IStream_Read(
        pStm,
        core::ptr::addr_of_mut!(delimiterLength) as LPVOID,
        core::mem::size_of::<DWORD>() as ULONG,
        core::ptr::addr_of_mut!(bread),
    );
    if bread != core::mem::size_of::<DWORD>() as ULONG {
        return E_FAIL;
    }

    /* read item delimiter string */
    let itemDelimiterA =
        HeapAlloc(GetProcessHeap(), 0, delimiterLength as SIZE_T) as *mut c_char;
    if itemDelimiterA.is_null() {
        return E_OUTOFMEMORY;
    }
    res = IStream_Read(
        pStm,
        itemDelimiterA as LPVOID,
        delimiterLength,
        core::ptr::addr_of_mut!(bread),
    );
    if bread != delimiterLength {
        HeapFree(GetProcessHeap(), 0, itemDelimiterA as LPVOID);
        return E_FAIL;
    }

    let mut lenW = MultiByteToWideChar(CP_ACP, 0, itemDelimiterA, -1, core::ptr::null_mut(), 0);
    (*This).itemDelimiter = HeapReAlloc(
        GetProcessHeap(),
        0,
        (*This).itemDelimiter as LPVOID,
        (lenW as usize * core::mem::size_of::<WCHAR>()) as SIZE_T,
    ) as LPOLESTR;
    if (*This).itemDelimiter.is_null() {
        HeapFree(GetProcessHeap(), 0, itemDelimiterA as LPVOID);
        return E_OUTOFMEMORY;
    }
    MultiByteToWideChar(CP_ACP, 0, itemDelimiterA, -1, (*This).itemDelimiter, lenW);
    HeapFree(GetProcessHeap(), 0, itemDelimiterA as LPVOID);

    /* read item name string length + 1 */
    res = IStream_Read(
        pStm,
        core::ptr::addr_of_mut!(nameLength) as LPVOID,
        core::mem::size_of::<DWORD>() as ULONG,
        core::ptr::addr_of_mut!(bread),
    );
    if bread != core::mem::size_of::<DWORD>() as ULONG {
        return E_FAIL;
    }

    /* read item name string */
    let itemNameA = HeapAlloc(GetProcessHeap(), 0, nameLength as SIZE_T) as *mut c_char;
    if itemNameA.is_null() {
        return E_OUTOFMEMORY;
    }
    res = IStream_Read(
        pStm,
        itemNameA as LPVOID,
        nameLength,
        core::ptr::addr_of_mut!(bread),
    );
    if bread != nameLength {
        HeapFree(GetProcessHeap(), 0, itemNameA as LPVOID);
        return E_FAIL;
    }

    lenW = MultiByteToWideChar(CP_ACP, 0, itemNameA, -1, core::ptr::null_mut(), 0);
    (*This).itemName = HeapReAlloc(
        GetProcessHeap(),
        0,
        (*This).itemName as LPVOID,
        (lenW as usize * core::mem::size_of::<WCHAR>()) as SIZE_T,
    ) as LPOLESTR;
    if (*This).itemName.is_null() {
        HeapFree(GetProcessHeap(), 0, itemNameA as LPVOID);
        return E_OUTOFMEMORY;
    }
    MultiByteToWideChar(CP_ACP, 0, itemNameA, -1, (*This).itemName, lenW);
    HeapFree(GetProcessHeap(), 0, itemNameA as LPVOID);

    res
}
}

stdmethod! {
fn ItemMonikerImpl_Save(iface: *mut IMoniker, pStm: *mut IStream, fClearDirty: BOOL) -> HRESULT {
    let This = impl_from_IMoniker(iface);

    /* data written by this function are :
       1) DWORD : size of item delimiter string ('\0' included)
       2) String (type A): item delimiter string ('\0' included)
       3) DWORD : size of item name string ('\0' included)
       4) String (type A): item name string ('\0' included) */

    let nameLength = WideCharToMultiByte(
        CP_ACP,
        0,
        (*This).itemName,
        -1,
        core::ptr::null_mut(),
        0,
        core::ptr::null(),
        core::ptr::null_mut(),
    ) as DWORD;
    let delimiterLength = WideCharToMultiByte(
        CP_ACP,
        0,
        (*This).itemDelimiter,
        -1,
        core::ptr::null_mut(),
        0,
        core::ptr::null(),
        core::ptr::null_mut(),
    ) as DWORD;
    let itemNameA = HeapAlloc(GetProcessHeap(), 0, nameLength as SIZE_T) as *mut c_char;
    let itemDelimiterA =
        HeapAlloc(GetProcessHeap(), 0, delimiterLength as SIZE_T) as *mut c_char;
    WideCharToMultiByte(
        CP_ACP,
        0,
        (*This).itemName,
        -1,
        itemNameA,
        nameLength as c_int,
        core::ptr::null(),
        core::ptr::null_mut(),
    );
    WideCharToMultiByte(
        CP_ACP,
        0,
        (*This).itemDelimiter,
        -1,
        itemDelimiterA,
        delimiterLength as c_int,
        core::ptr::null(),
        core::ptr::null_mut(),
    );

    TRACE!(b"%p, %d\n\0".as_ptr(), pStm, fClearDirty);

    let mut res = IStream_Write(
        pStm,
        core::ptr::addr_of!(delimiterLength) as *const core::ffi::c_void,
        core::mem::size_of::<DWORD>() as ULONG,
        core::ptr::null_mut(),
    );
    res = IStream_Write(
        pStm,
        itemDelimiterA as *const core::ffi::c_void,
        delimiterLength * core::mem::size_of::<c_char>() as ULONG,
        core::ptr::null_mut(),
    );
    res = IStream_Write(
        pStm,
        core::ptr::addr_of!(nameLength) as *const core::ffi::c_void,
        core::mem::size_of::<DWORD>() as ULONG,
        core::ptr::null_mut(),
    );
    res = IStream_Write(
        pStm,
        itemNameA as *const core::ffi::c_void,
        nameLength * core::mem::size_of::<c_char>() as ULONG,
        core::ptr::null_mut(),
    );

    HeapFree(GetProcessHeap(), 0, itemNameA as LPVOID);
    HeapFree(GetProcessHeap(), 0, itemDelimiterA as LPVOID);

    res
}
}

stdmethod! {
fn ItemMonikerImpl_GetSizeMax(iface: *mut IMoniker, pcbSize: *mut ULARGE_INTEGER) -> HRESULT {
    let This = impl_from_IMoniker(iface);
    let delimiterLength = (lstrlenW((*This).itemDelimiter) + 1) as DWORD;
    let nameLength = (lstrlenW((*This).itemName) + 1) as DWORD;

    TRACE!(b"(%p,%p)\n\0".as_ptr(), iface, pcbSize);

    if pcbSize.is_null() {
        return E_POINTER;
    }

    /* for more details see ItemMonikerImpl_Save comments */
    (*pcbSize).u.LowPart = core::mem::size_of::<DWORD>() as DWORD /* DWORD which contains delimiter length */
        + delimiterLength * 4                                     /* item delimiter string */
        + core::mem::size_of::<DWORD>() as DWORD                  /* DWORD which contains item name length */
        + nameLength * 4                                          /* item name string */
        + 18;                                                     /* strange, but true */
    (*pcbSize).u.HighPart = 0;

    S_OK
}
}

stdmethod! {
fn ItemMonikerImpl_BindToObject(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    riid: *const IID,
    ppvResult: *mut LPVOID,
) -> HRESULT {
    let This = impl_from_IMoniker(iface);
    let refid: IID = IID_IOleItemContainer;
    let mut poic: *mut IOleItemContainer = core::ptr::null_mut();

    TRACE!(
        b"(%p,%p,%p,%s,%p)\n\0".as_ptr(),
        iface,
        pbc,
        pmkToLeft,
        debugstr_guid(riid),
        ppvResult,
    );

    if ppvResult.is_null() {
        return E_POINTER;
    }

    if pmkToLeft.is_null() {
        return E_INVALIDARG;
    }

    *ppvResult = core::ptr::null_mut();

    let mut res = IMoniker_BindToObject(
        pmkToLeft,
        pbc,
        core::ptr::null_mut(),
        core::ptr::addr_of!(refid),
        core::ptr::addr_of_mut!(poic) as *mut LPVOID,
    );

    if SUCCEEDED(res) {
        res = IOleItemContainer_GetObject(
            poic,
            (*This).itemName,
            BINDSPEED_MODERATE,
            pbc,
            riid,
            ppvResult,
        );

        IOleItemContainer_Release(poic);
    }

    res
}
}

stdmethod! {
fn ItemMonikerImpl_BindToStorage(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    riid: *const IID,
    ppvResult: *mut LPVOID,
) -> HRESULT {
    let This = impl_from_IMoniker(iface);
    let mut poic: *mut IOleItemContainer = core::ptr::null_mut();

    TRACE!(
        b"(%p,%p,%p,%s,%p)\n\0".as_ptr(),
        iface,
        pbc,
        pmkToLeft,
        debugstr_guid(riid),
        ppvResult,
    );

    *ppvResult = core::ptr::null_mut();

    if pmkToLeft.is_null() {
        return E_INVALIDARG;
    }

    let mut res = IMoniker_BindToObject(
        pmkToLeft,
        pbc,
        core::ptr::null_mut(),
        core::ptr::addr_of!(IID_IOleItemContainer),
        core::ptr::addr_of_mut!(poic) as *mut LPVOID,
    );

    if SUCCEEDED(res) {
        res = IOleItemContainer_GetObjectStorage(poic, (*This).itemName, pbc, riid, ppvResult);

        IOleItemContainer_Release(poic);
    }

    res
}
}

stdmethod! {
fn ItemMonikerImpl_Reduce(
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

    ItemMonikerImpl_AddRef(iface);

    *ppmkReduced = iface;

    MK_S_REDUCED_TO_SELF
}
}

stdmethod! {
fn ItemMonikerImpl_ComposeWith(
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
fn ItemMonikerImpl_Enum(
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
fn ItemMonikerImpl_IsEqual(iface: *mut IMoniker, pmkOtherMoniker: *mut IMoniker) -> HRESULT {
    let mut clsid: CLSID = core::mem::zeroed();
    let mut dispName1: LPOLESTR = core::ptr::null_mut();
    let mut dispName2: LPOLESTR = core::ptr::null_mut();
    let mut bind: *mut IBindCtx = core::ptr::null_mut();
    let mut res: HRESULT = S_FALSE;

    TRACE!(b"(%p,%p)\n\0".as_ptr(), iface, pmkOtherMoniker);

    if pmkOtherMoniker.is_null() {
        return S_FALSE;
    }

    /* check if both are ItemMoniker */
    if FAILED(IMoniker_GetClassID(pmkOtherMoniker, &mut clsid)) {
        return S_FALSE;
    }
    if !IsEqualCLSID(&clsid, core::ptr::addr_of!(CLSID_ItemMoniker)) {
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
fn ItemMonikerImpl_Hash(iface: *mut IMoniker, pdwHash: *mut DWORD) -> HRESULT {
    let This = impl_from_IMoniker(iface);
    let mut h: DWORD = 0;
    let mut off: isize = 0;

    if pdwHash.is_null() {
        return E_POINTER;
    }

    let val = (*This).itemName;
    let len = lstrlenW(val);

    let mut i = len;
    while i > 0 {
        h = h.wrapping_mul(3) ^ (towupper(*val.offset(off) as c_int) as DWORD);
        off += 1;
        i -= 1;
    }

    *pdwHash = h;

    S_OK
}
}

stdmethod! {
fn ItemMonikerImpl_IsRunning(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    pmkNewlyRunning: *mut IMoniker,
) -> HRESULT {
    let This = impl_from_IMoniker(iface);
    let mut rot: *mut IRunningObjectTable = core::ptr::null_mut();
    let res: HRESULT;
    let mut poic: *mut IOleItemContainer = core::ptr::null_mut();

    TRACE!(
        b"(%p,%p,%p,%p)\n\0".as_ptr(),
        iface,
        pbc,
        pmkToLeft,
        pmkNewlyRunning,
    );

    /* If pmkToLeft is NULL, this method returns TRUE if pmkNewlyRunning is non-NULL and is
       equal to this moniker. Otherwise, the method checks the ROT to see whether this
       moniker is running. */
    if pmkToLeft.is_null() {
        if !pmkNewlyRunning.is_null() && IMoniker_IsEqual(pmkNewlyRunning, iface) == S_OK {
            return S_OK;
        } else {
            if pbc.is_null() {
                return E_INVALIDARG;
            }

            let hr = IBindCtx_GetRunningObjectTable(pbc, &mut rot);

            if FAILED(hr) {
                return hr;
            }

            res = IRunningObjectTable_IsRunning(rot, iface);

            IRunningObjectTable_Release(rot);
        }
    } else {
        /* If pmkToLeft is non-NULL, the method calls IMoniker::BindToObject on the pmkToLeft
           parameter, requesting an IOleItemContainer interface pointer. The method then calls
           IOleItemContainer::IsRunning, passing the string contained within this moniker. */
        let mut hr = IMoniker_BindToObject(
            pmkToLeft,
            pbc,
            core::ptr::null_mut(),
            core::ptr::addr_of!(IID_IOleItemContainer),
            core::ptr::addr_of_mut!(poic) as *mut LPVOID,
        );

        if SUCCEEDED(hr) {
            hr = IOleItemContainer_IsRunning(poic, (*This).itemName);

            IOleItemContainer_Release(poic);
        }
        res = hr;
    }

    res
}
}

stdmethod! {
fn ItemMonikerImpl_GetTimeOfLastChange(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    pItemTime: *mut FILETIME,
) -> HRESULT {
    let mut rot: *mut IRunningObjectTable = core::ptr::null_mut();
    let mut compositeMk: *mut IMoniker = core::ptr::null_mut();

    TRACE!(b"(%p,%p,%p,%p)\n\0".as_ptr(), iface, pbc, pmkToLeft, pItemTime);

    if pItemTime.is_null() {
        return E_INVALIDARG;
    }

    /* If pmkToLeft is NULL, this method returns MK_E_NOTBINDABLE */
    if pmkToLeft.is_null() {
        MK_E_NOTBINDABLE
    } else {
        /* Otherwise, the method creates a composite of pmkToLeft and this moniker and uses the
           ROT to access the time of last change. If the object is not in the ROT, the method
           calls IMoniker::GetTimeOfLastChange on the pmkToLeft parameter. */
        let mut res = CreateGenericComposite(pmkToLeft, iface, &mut compositeMk);
        if FAILED(res) {
            return res;
        }

        res = IBindCtx_GetRunningObjectTable(pbc, &mut rot);
        if FAILED(res) {
            IMoniker_Release(compositeMk);
            return res;
        }

        if IRunningObjectTable_GetTimeOfLastChange(rot, compositeMk, pItemTime) != S_OK {
            res = IMoniker_GetTimeOfLastChange(pmkToLeft, pbc, core::ptr::null_mut(), pItemTime);
        }

        IMoniker_Release(compositeMk);

        res
    }
}
}

stdmethod! {
fn ItemMonikerImpl_Inverse(iface: *mut IMoniker, ppmk: *mut *mut IMoniker) -> HRESULT {
    TRACE!(b"(%p,%p)\n\0".as_ptr(), iface, ppmk);

    if ppmk.is_null() {
        return E_POINTER;
    }

    CreateAntiMoniker(ppmk)
}
}

stdmethod! {
fn ItemMonikerImpl_CommonPrefixWith(
    iface: *mut IMoniker,
    pmkOther: *mut IMoniker,
    ppmkPrefix: *mut *mut IMoniker,
) -> HRESULT {
    let mut mkSys: DWORD = 0;

    TRACE!(b"(%p,%p)\n\0".as_ptr(), pmkOther, ppmkPrefix);

    IMoniker_IsSystemMoniker(pmkOther, &mut mkSys);

    /* If the other moniker is an item moniker that is equal to this moniker, this method sets
       *ppmkPrefix to this moniker and returns MK_S_US */
    if mkSys == MKSYS_ITEMMONIKER && IMoniker_IsEqual(iface, pmkOther) == S_OK {
        *ppmkPrefix = iface;

        IMoniker_AddRef(iface);

        MK_S_US
    } else {
        /* otherwise, the method calls the MonikerCommonPrefixWith function. This function
           correctly handles the case where the other moniker is a generic composite. */
        MonikerCommonPrefixWith(iface, pmkOther, ppmkPrefix)
    }
}
}

stdmethod! {
fn ItemMonikerImpl_RelativePathTo(
    iface: *mut IMoniker,
    pmOther: *mut IMoniker,
    ppmkRelPath: *mut *mut IMoniker,
) -> HRESULT {
    TRACE!(b"(%p,%p,%p)\n\0".as_ptr(), iface, pmOther, ppmkRelPath);

    if ppmkRelPath.is_null() {
        return E_POINTER;
    }

    *ppmkRelPath = core::ptr::null_mut();

    MK_E_NOTBINDABLE
}
}

stdmethod! {
fn ItemMonikerImpl_GetDisplayName(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    ppszDisplayName: *mut LPOLESTR,
) -> HRESULT {
    let This = impl_from_IMoniker(iface);

    TRACE!(b"(%p,%p,%p,%p)\n\0".as_ptr(), iface, pbc, pmkToLeft, ppszDisplayName);

    if ppszDisplayName.is_null() {
        return E_POINTER;
    }

    if !pmkToLeft.is_null() {
        return E_INVALIDARG;
    }

    let chars =
        (lstrlenW((*This).itemDelimiter) + lstrlenW((*This).itemName) + 1) as usize;
    *ppszDisplayName =
        CoTaskMemAlloc((core::mem::size_of::<WCHAR>() * chars) as SIZE_T) as LPOLESTR;

    if (*ppszDisplayName).is_null() {
        return E_OUTOFMEMORY;
    }

    lstrcpyW(*ppszDisplayName, (*This).itemDelimiter);
    lstrcatW(*ppszDisplayName, (*This).itemName);

    TRACE!(b"-- %s\n\0".as_ptr(), debugstr_w(*ppszDisplayName));

    S_OK
}
}

stdmethod! {
fn ItemMonikerImpl_ParseDisplayName(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    pszDisplayName: LPOLESTR,
    pchEaten: *mut ULONG,
    ppmkOut: *mut *mut IMoniker,
) -> HRESULT {
    let This = impl_from_IMoniker(iface);
    let mut poic: *mut IOleItemContainer = core::ptr::null_mut();
    let mut ppdn: *mut IParseDisplayName = core::ptr::null_mut();
    let mut displayName: LPOLESTR = core::ptr::null_mut();

    TRACE!(b"%s\n\0".as_ptr(), debugstr_w(pszDisplayName));

    /* If pmkToLeft is NULL, this method returns MK_E_SYNTAX */
    if pmkToLeft.is_null() {
        MK_E_SYNTAX
    } else {
        /* Otherwise, the method calls IMoniker::BindToObject on the pmkToLeft parameter,
           requesting an IParseDisplayName interface pointer to the object identified by the
           moniker, and passes the display name to IParseDisplayName::ParseDisplayName */
        let mut res = IMoniker_BindToObject(
            pmkToLeft,
            pbc,
            core::ptr::null_mut(),
            core::ptr::addr_of!(IID_IOleItemContainer),
            core::ptr::addr_of_mut!(poic) as *mut LPVOID,
        );

        if SUCCEEDED(res) {
            res = IOleItemContainer_GetObject(
                poic,
                (*This).itemName,
                BINDSPEED_MODERATE,
                pbc,
                core::ptr::addr_of!(IID_IParseDisplayName),
                core::ptr::addr_of_mut!(ppdn) as *mut LPVOID,
            );

            res = IMoniker_GetDisplayName(iface, pbc, core::ptr::null_mut(), &mut displayName);

            res = IParseDisplayName_ParseDisplayName(ppdn, pbc, displayName, pchEaten, ppmkOut);

            IOleItemContainer_Release(poic);
            IParseDisplayName_Release(ppdn);
        }

        res
    }
}
}

stdmethod! {
fn ItemMonikerImpl_IsSystemMoniker(iface: *mut IMoniker, pwdMksys: *mut DWORD) -> HRESULT {
    TRACE!(b"(%p,%p)\n\0".as_ptr(), iface, pwdMksys);

    if pwdMksys.is_null() {
        return E_POINTER;
    }

    *pwdMksys = MKSYS_ITEMMONIKER;

    S_OK
}
}

stdmethod! {
fn ItemMonikerROTDataImpl_QueryInterface(
    iface: *mut IROTData,
    riid: *const IID,
    ppvObject: *mut LPVOID,
) -> HRESULT {
    let This = impl_from_IROTData(iface);

    TRACE!(b"(%p,%s,%p)\n\0".as_ptr(), iface, debugstr_guid(riid), ppvObject);

    ItemMonikerImpl_QueryInterface(
        core::ptr::addr_of_mut!((*This).IMoniker_iface),
        riid,
        ppvObject,
    )
}
}

stdmethod! {
fn ItemMonikerROTDataImpl_AddRef(iface: *mut IROTData) -> ULONG {
    let This = impl_from_IROTData(iface);

    TRACE!(b"(%p)\n\0".as_ptr(), iface);

    ItemMonikerImpl_AddRef(core::ptr::addr_of_mut!((*This).IMoniker_iface))
}
}

stdmethod! {
fn ItemMonikerROTDataImpl_Release(iface: *mut IROTData) -> ULONG {
    let This = impl_from_IROTData(iface);

    TRACE!(b"(%p)\n\0".as_ptr(), iface);

    ItemMonikerImpl_Release(core::ptr::addr_of_mut!((*This).IMoniker_iface))
}
}

stdmethod! {
fn ItemMonikerROTDataImpl_GetComparisonData(
    iface: *mut IROTData,
    pbData: *mut byte,
    cbMax: ULONG,
    pcbData: *mut ULONG,
) -> HRESULT {
    let This = impl_from_IROTData(iface);
    let len = (lstrlenW((*This).itemName) + 1) as isize;

    TRACE!(b"(%p, %u, %p)\n\0".as_ptr(), pbData, cbMax, pcbData);

    *pcbData = (core::mem::size_of::<CLSID>()
        + core::mem::size_of::<WCHAR>()
        + len as usize * core::mem::size_of::<WCHAR>()) as ULONG;
    if cbMax < *pcbData {
        return E_OUTOFMEMORY;
    }

    /* write CLSID */
    memcpy(
        pbData as *mut core::ffi::c_void,
        core::ptr::addr_of!(CLSID_ItemMoniker) as *const core::ffi::c_void,
        core::mem::size_of::<CLSID>(),
    );
    /* write delimiter */
    let pszItemDelimiter = pbData.add(core::mem::size_of::<CLSID>()) as LPWSTR;
    *pszItemDelimiter = *(*This).itemDelimiter;
    /* write name */
    let pszItemName = pszItemDelimiter.offset(1);
    let mut i: isize = 0;
    while i < len {
        *pszItemName.offset(i) = towupper(*(*This).itemName.offset(i) as c_int) as WCHAR;
        i += 1;
    }

    S_OK
}
}

unsafe fn ItemMonikerImpl_Construct(
    This: *mut ItemMonikerImpl,
    lpszDelim: LPCOLESTR,
    lpszItem: LPCOLESTR,
) -> HRESULT {
    let sizeStr1 = lstrlenW(lpszItem);
    static emptystr: [OLECHAR; 1] = [0];

    TRACE!(
        b"(%p,%s,%s)\n\0".as_ptr(),
        This,
        debugstr_w(lpszDelim),
        debugstr_w(lpszItem),
    );

    /* Initialize the virtual function table. */
    (*This).IMoniker_iface.lpVtbl =
        core::ptr::addr_of!(VT_ItemMonikerImpl) as *mut IMonikerVtbl;
    (*This).IROTData_iface.lpVtbl =
        core::ptr::addr_of!(VT_ROTDataImpl) as *mut IROTDataVtbl;
    (*This).ref_ = 0;
    (*This).pMarshal = core::ptr::null_mut();

    (*This).itemName = HeapAlloc(
        GetProcessHeap(),
        0,
        (core::mem::size_of::<WCHAR>() * (sizeStr1 as usize + 1)) as SIZE_T,
    ) as LPOLESTR;
    if (*This).itemName.is_null() {
        return E_OUTOFMEMORY;
    }
    lstrcpyW((*This).itemName, lpszItem);

    if lpszDelim.is_null() {
        FIXME!(b"lpszDelim is NULL. Using empty string which is possibly wrong.\n\0".as_ptr());
    }

    let delim: LPCOLESTR = if !lpszDelim.is_null() {
        lpszDelim
    } else {
        emptystr.as_ptr()
    };

    let sizeStr2 = lstrlenW(delim);
    (*This).itemDelimiter = HeapAlloc(
        GetProcessHeap(),
        0,
        (core::mem::size_of::<WCHAR>() * (sizeStr2 as usize + 1)) as SIZE_T,
    ) as LPOLESTR;
    if (*This).itemDelimiter.is_null() {
        HeapFree(GetProcessHeap(), 0, (*This).itemName as LPVOID);
        return E_OUTOFMEMORY;
    }
    lstrcpyW((*This).itemDelimiter, delim);

    S_OK
}

unsafe fn ItemMonikerImpl_Destroy(This: *mut ItemMonikerImpl) -> HRESULT {
    TRACE!(b"(%p)\n\0".as_ptr(), This);

    if !(*This).pMarshal.is_null() {
        IUnknown_Release((*This).pMarshal);
    }
    HeapFree(GetProcessHeap(), 0, (*This).itemName as LPVOID);
    HeapFree(GetProcessHeap(), 0, (*This).itemDelimiter as LPVOID);
    HeapFree(GetProcessHeap(), 0, This as LPVOID);

    S_OK
}

/******************************************************************************
 *        CreateItemMoniker	[OLE32.@]
 ******************************************************************************/
#[no_mangle]
pub unsafe extern "system" fn CreateItemMoniker(
    lpszDelim: LPCOLESTR,
    lpszItem: LPCOLESTR,
    ppmk: *mut *mut IMoniker,
) -> HRESULT {
    TRACE!(
        b"(%s,%s,%p)\n\0".as_ptr(),
        debugstr_w(lpszDelim),
        debugstr_w(lpszItem),
        ppmk,
    );

    let newItemMoniker = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<ItemMonikerImpl>() as SIZE_T,
    ) as *mut ItemMonikerImpl;

    if newItemMoniker.is_null() {
        return STG_E_INSUFFICIENTMEMORY;
    }

    let hr = ItemMonikerImpl_Construct(newItemMoniker, lpszDelim, lpszItem);

    if FAILED(hr) {
        HeapFree(GetProcessHeap(), 0, newItemMoniker as LPVOID);
        return hr;
    }

    ItemMonikerImpl_QueryInterface(
        core::ptr::addr_of_mut!((*newItemMoniker).IMoniker_iface),
        core::ptr::addr_of!(IID_IMoniker),
        ppmk as *mut LPVOID,
    )
}

#[no_mangle]
pub unsafe extern "system" fn ItemMoniker_CreateInstance(
    _iface: *mut IClassFactory,
    pUnk: *mut IUnknown,
    riid: REFIID,
    ppv: *mut LPVOID,
) -> HRESULT {
    static wszEmpty: [WCHAR; 1] = [0];

    TRACE!(b"(%p, %s, %p)\n\0".as_ptr(), pUnk, debugstr_guid(riid), ppv);

    *ppv = core::ptr::null_mut();

    if !pUnk.is_null() {
        return CLASS_E_NOAGGREGATION;
    }

    let newItemMoniker = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<ItemMonikerImpl>() as SIZE_T,
    ) as *mut ItemMonikerImpl;
    if newItemMoniker.is_null() {
        return E_OUTOFMEMORY;
    }

    let mut hr =
        ItemMonikerImpl_Construct(newItemMoniker, wszEmpty.as_ptr(), wszEmpty.as_ptr());

    if SUCCEEDED(hr) {
        hr = ItemMonikerImpl_QueryInterface(
            core::ptr::addr_of_mut!((*newItemMoniker).IMoniker_iface),
            riid,
            ppv,
        );
    }
    if FAILED(hr) {
        HeapFree(GetProcessHeap(), 0, newItemMoniker as LPVOID);
    }

    hr
}
