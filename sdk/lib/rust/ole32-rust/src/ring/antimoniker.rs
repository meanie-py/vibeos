use crate::prelude::*;
use core::ffi::c_void;

extern "C" {
    fn CoTaskMemAlloc(cb: usize) -> *mut core::ffi::c_void;
}
extern "C" {

    fn CreateGenericComposite( pmkFirst: *mut IMoniker, pmkRest: *mut IMoniker, ppmkComposite: *mut *mut IMoniker, ) -> HRESULT;
    fn MonikerCommonPrefixWith( pmkThis: *mut IMoniker, pmkOther: *mut IMoniker, ppmkCommon: *mut *mut IMoniker ) -> HRESULT;
}

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct AntiMonikerImpl {
    pub IMoniker_iface: IMoniker,
    pub IROTData_iface: IROTData,
    pub ref_: LONG,
    pub pMarshal: *mut IUnknown,
}

pub type PAntiMonikerImpl = *mut AntiMonikerImpl;

pub static VT_AntiMonikerImpl: IMonikerVtbl = IMonikerVtbl {
    QueryInterface: AntiMonikerImpl_QueryInterface,
    AddRef: AntiMonikerImpl_AddRef,
    Release: AntiMonikerImpl_Release,
    GetClassID: AntiMonikerImpl_GetClassID,
    IsDirty: AntiMonikerImpl_IsDirty,
    Load: AntiMonikerImpl_Load,
    Save: AntiMonikerImpl_Save,
    GetSizeMax: AntiMonikerImpl_GetSizeMax,
    BindToObject: AntiMonikerImpl_BindToObject,
    BindToStorage: AntiMonikerImpl_BindToStorage,
    Reduce: AntiMonikerImpl_Reduce,
    ComposeWith: AntiMonikerImpl_ComposeWith,
    Enum: AntiMonikerImpl_Enum,
    IsEqual: AntiMonikerImpl_IsEqual,
    Hash: AntiMonikerImpl_Hash,
    IsRunning: AntiMonikerImpl_IsRunning,
    GetTimeOfLastChange: AntiMonikerImpl_GetTimeOfLastChange,
    Inverse: AntiMonikerImpl_Inverse,
    CommonPrefixWith: AntiMonikerImpl_CommonPrefixWith,
    RelativePathTo: AntiMonikerImpl_RelativePathTo,
    GetDisplayName: AntiMonikerImpl_GetDisplayName,
    ParseDisplayName: AntiMonikerImpl_ParseDisplayName,
    IsSystemMoniker: AntiMonikerImpl_IsSystemMoniker,
};

pub static VT_ROTDataImpl: IROTDataVtbl = IROTDataVtbl {
    QueryInterface: AntiMonikerROTDataImpl_QueryInterface,
    AddRef: AntiMonikerROTDataImpl_AddRef,
    Release: AntiMonikerROTDataImpl_Release,
    GetComparisonData: AntiMonikerROTDataImpl_GetComparisonData,
};

unsafe fn impl_from_IMoniker(iface: *mut IMoniker) -> *mut AntiMonikerImpl {
    (iface as *mut u8).offset(-(core::mem::offset_of!(AntiMonikerImpl, IMoniker_iface) as isize)) as *mut AntiMonikerImpl
}

unsafe fn impl_from_IROTData(iface: *mut IROTData) -> *mut AntiMonikerImpl {
    (iface as *mut u8).offset(-(core::mem::offset_of!(AntiMonikerImpl, IROTData_iface) as isize)) as *mut AntiMonikerImpl
}

unsafe fn AntiMonikerImpl_QueryInterface(
    iface: *mut IMoniker,
    riid: *const CLSID,
    ppvObject: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let This = impl_from_IMoniker(iface);

    TRACE!(
        b"(%p,%s,%p)\n\0".as_ptr() as *const i8,
        This,
        debugstr_guid(riid),
        ppvObject,
    );

    if ppvObject.is_null() {
        return 0x80070057u32 as HRESULT;
    }

    *ppvObject = core::ptr::null_mut();

    if IsEqualIID(core::ptr::addr_of!(IID_IUnknown), riid) != 0
        || IsEqualIID(core::ptr::addr_of!(IID_IPersist), riid) != 0
        || IsEqualIID(core::ptr::addr_of!(IID_IPersistStream), riid) != 0
        || IsEqualIID(core::ptr::addr_of!(IID_IMoniker), riid) != 0
    {
        *ppvObject = iface as *mut core::ffi::c_void;
    } else if IsEqualIID(core::ptr::addr_of!(IID_IROTData), riid) != 0 {
        *ppvObject = core::ptr::addr_of_mut!((*This).IROTData_iface) as *mut core::ffi::c_void;
    } else if IsEqualIID(core::ptr::addr_of!(IID_IMarshal), riid) != 0 {
        let mut hr: HRESULT = 0;
        if (*This).pMarshal.is_null() {
            hr = MonikerMarshal_Create(iface, core::ptr::addr_of_mut!((*This).pMarshal));
        }
        if hr != 0 {
            return hr;
        }
        return IUnknown_QueryInterface((*This).pMarshal, riid, ppvObject);
    }

    if (*ppvObject).is_null() {
        return 0x80004002u32 as HRESULT;
    }

    IMoniker_AddRef(iface);

    return 0;
}

unsafe fn AntiMonikerImpl_AddRef(iface: *mut IMoniker) -> ULONG {
    let This = impl_from_IMoniker(iface);

    InterlockedIncrement(&mut (*This).ref_) as ULONG
}

unsafe fn AntiMonikerImpl_Release(iface: *mut IMoniker) -> u32 {
    let This = impl_from_IMoniker(iface);
    let ref_count: u32;

    TRACE!(b"(%p)\n\0".as_ptr() as *const i8, This);

    ref_count = InterlockedDecrement(&mut (*This).ref_) as u32;

    if ref_count == 0 {
        if !(*This).pMarshal.is_null() {
            IUnknown_Release((*This).pMarshal);
        }
        HeapFree(GetProcessHeap(), 0, This as *mut _);
    }

    ref_count
}

unsafe fn AntiMonikerImpl_GetClassID(
    iface: *mut IMoniker,
    pClassID: *mut CLSID,
) -> HRESULT {
    if pClassID.is_null() {
        return E_POINTER;
    }

    *pClassID = CLSID_AntiMoniker;

    S_OK
}

unsafe fn AntiMonikerImpl_IsDirty(iface: *mut IMoniker) -> HRESULT {
    TRACE!(b"(%p)\n\0".as_ptr() as *const i8, iface);
    S_FALSE
}

unsafe fn AntiMonikerImpl_Load(iface: *mut IMoniker, pStm: *mut IStream) -> HRESULT {
    let constant: u32 = 1;
    let mut dwbuffer: u32 = 0;

    let res = IStream_Read(
        pStm,
        &mut dwbuffer as *mut u32 as *mut core::ffi::c_void,
        core::mem::size_of::<u32>() as u32,
        core::ptr::null_mut()
    );

    if SUCCEEDED(res) != 0 && dwbuffer != constant {
        return E_FAIL;
    }

    res
}

unsafe fn AntiMonikerImpl_Save(
    iface: *mut IMoniker,
    pStm: *mut IStream,
    fClearDirty: i32,
) -> HRESULT {
    static CONSTANT: u32 = 1;
    IStream_Write(
        pStm,
        &CONSTANT as *const u32 as *const core::ffi::c_void,
        core::mem::size_of::<u32>() as u32,
        core::ptr::null_mut(),
    )
}
unsafe fn AntiMonikerImpl_GetSizeMax(
    iface: *mut IMoniker,
    pcbSize: *mut ULARGE_INTEGER,
) -> HRESULT {
    TRACE!(b"(%p,%p)\n\0".as_ptr() as *const i8, iface, pcbSize);

    if pcbSize.is_null() {
        return E_POINTER;
    }

    (*pcbSize).u.__bindgen_anon_1.LowPart = core::mem::size_of::<u32>() as u32 + 16;
    (*pcbSize).u.__bindgen_anon_1.HighPart = 0;

    S_OK
}
unsafe fn AntiMonikerImpl_BindToObject(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    riid: *const IID,
    ppvResult: *mut *mut core::ffi::c_void,
) -> HRESULT {
    TRACE!(
        b"(%p,%p,%p,%s,%p)\n\0".as_ptr(),
        iface,
        pbc,
        pmkToLeft,
        debugstr_guid(riid as *const GUID),
        ppvResult,
    );
    E_NOTIMPL
}
unsafe fn AntiMonikerImpl_BindToStorage(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    riid: *const IID,
    ppvResult: *mut *mut core::ffi::c_void,
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

unsafe fn AntiMonikerImpl_Reduce(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    dwReduceHowFar: DWORD,
    ppmkToLeft: *mut *mut IMoniker,
    ppmkReduced: *mut *mut IMoniker,
) -> HRESULT {
    if ppmkReduced == core::ptr::null_mut() {
        return E_POINTER;
    }

    AntiMonikerImpl_AddRef(iface);

    *ppmkReduced = iface;

    return MK_S_REDUCED_TO_SELF;
}

unsafe fn AntiMonikerImpl_ComposeWith(
    iface: *mut IMoniker,
    pmkRight: *mut IMoniker,
    fOnlyIfNotGeneric: i32,
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
        return 0x80004003u32 as HRESULT;
    }

    *ppmkComposite = core::ptr::null_mut();

    if fOnlyIfNotGeneric != 0 {
        return 0x800401E1u32 as HRESULT;
    } else {
        return CreateGenericComposite(iface, pmkRight, ppmkComposite);
    }
}

unsafe fn AntiMonikerImpl_Enum(
    iface: *mut IMoniker,
    fForward: i32,
    ppenumMoniker: *mut *mut IEnumMoniker
) -> HRESULT {
    TRACE!(b"(%p,%d,%p)\n\0".as_ptr() as *const i8, iface, fForward, ppenumMoniker);

    if ppenumMoniker.is_null() {
        return E_POINTER;
    }

    *ppenumMoniker = core::ptr::null_mut();

    S_OK
}

unsafe fn AntiMonikerImpl_IsEqual(iface: *mut IMoniker, pmkOtherMoniker: *mut IMoniker) -> HRESULT {
    let mut mkSys: u32 = 0;

    TRACE!(b"(%p,%p)\n\0".as_ptr() as *const i8, iface, pmkOtherMoniker);

    if pmkOtherMoniker.is_null() {
        return 1;
    }

    IMoniker_IsSystemMoniker(pmkOtherMoniker, &mut mkSys);

    if mkSys == 6 {
        return 0;
    } else {
        return 1;
    }
}

unsafe fn AntiMonikerImpl_Hash(iface: *mut IMoniker, pdwHash: *mut DWORD) -> HRESULT {
    if pdwHash.is_null() {
        return E_POINTER;
    }

    *pdwHash = 0x80000001;

    S_OK
}

unsafe fn AntiMonikerImpl_IsRunning(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    pmkNewlyRunning: *mut IMoniker,
) -> HRESULT {
    let mut rot: *mut IRunningObjectTable = core::ptr::null_mut();
    let mut res: HRESULT;

    TRACE!(
        b"(%p,%p,%p,%p)\n\0".as_ptr() as *const i8,
        iface,
        pbc,
        pmkToLeft,
        pmkNewlyRunning,
    );

    if pbc.is_null() {
        return E_INVALIDARG;
    }

    res = IBindCtx_GetRunningObjectTable(pbc, &mut rot);

    if FAILED(res) != 0 {
        return res;
    }

    res = IRunningObjectTable_IsRunning(rot, iface);

    IRunningObjectTable_Release(rot);

    res
}

unsafe fn AntiMonikerImpl_GetTimeOfLastChange(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    pAntiTime: *mut FILETIME,
) -> HRESULT {
    TRACE!(b"(%p,%p,%p,%p)\n\0".as_ptr() as *const i8, iface, pbc, pmkToLeft, pAntiTime);
    E_NOTIMPL
}

unsafe fn AntiMonikerImpl_Inverse(iface: *mut IMoniker, ppmk: *mut *mut IMoniker) -> HRESULT {
    TRACE!(b"(%p,%p)\n\0".as_ptr() as *const i8, iface, ppmk);

    if ppmk.is_null() {
        return E_POINTER;
    }

    *ppmk = core::ptr::null_mut();

    return MK_E_NOINVERSE;
}

unsafe fn AntiMonikerImpl_CommonPrefixWith(
    iface: *mut IMoniker,
    pmkOther: *mut IMoniker,
    ppmkPrefix: *mut *mut IMoniker
) -> HRESULT {
    let mut mkSys: u32 = 0;

    IMoniker_IsSystemMoniker(pmkOther, &mut mkSys);

    if mkSys == 4 {
        IMoniker_AddRef(iface);

        *ppmkPrefix = iface;

        IMoniker_AddRef(iface);

        return 0x00040002u32 as HRESULT;
    } else {
        return MonikerCommonPrefixWith(iface, pmkOther, ppmkPrefix);
    }
}

unsafe fn AntiMonikerImpl_RelativePathTo(
    iface: *mut IMoniker,
    pmOther: *mut IMoniker,
    ppmkRelPath: *mut *mut IMoniker,
) -> HRESULT {

    TRACE!(
        b"(%p,%p,%p)\n\0".as_ptr(),
        iface,
        pmOther,
        ppmkRelPath,
    );

    if ppmkRelPath.is_null() {
        return 0x80004003u32 as HRESULT;
    }

    IMoniker_AddRef(pmOther);

    *ppmkRelPath = pmOther;

    return 0x00000001u32 as HRESULT;
}

unsafe fn AntiMonikerImpl_GetDisplayName(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    ppszDisplayName: *mut LPOLESTR,
) -> HRESULT {
    static BACK: [u16; 4] = ['\\' as u16, '.' as u16, '.' as u16, 0];

    TRACE!(
        b"(%p,%p,%p,%p)\n\0".as_ptr(),
        iface,
        pbc,
        pmkToLeft,
        ppszDisplayName,
    );

    if ppszDisplayName.is_null() {
        return 0x80004003u32 as HRESULT;
    }

    if !pmkToLeft.is_null() {
        FIXME!(b"() pmkToLeft!=NULL not implemented\n\0".as_ptr());
        return 0x80004001u32 as HRESULT;
    }

    *ppszDisplayName = CoTaskMemAlloc(core::mem::size_of_val(&BACK)) as LPOLESTR;

    if (*ppszDisplayName).is_null() {
        return 0x8007000Eu32 as HRESULT;
    }

    lstrcpyW(*ppszDisplayName, BACK.as_ptr());

    0u32 as HRESULT
}

unsafe fn AntiMonikerImpl_ParseDisplayName(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    pszDisplayName: LPOLESTR,
    pchEaten: *mut ULONG,
    ppmkOut: *mut *mut IMoniker,
) -> HRESULT {
    TRACE!(
        b"(%p,%p,%p,%p,%p,%p)\n\0".as_ptr() as PCHAR,
        iface,
        pbc,
        pmkToLeft,
        pszDisplayName,
        pchEaten,
        ppmkOut,
    );
    E_NOTIMPL
}

unsafe fn AntiMonikerImpl_IsSystemMoniker(
    iface: *mut IMoniker,
    pwdMksys: *mut DWORD,
) -> HRESULT {
    TRACE!(b"(%p,%p)\n\0".as_ptr() as PCHAR, iface, pwdMksys);

    if pwdMksys.is_null() {
        return E_POINTER;
    }

    *pwdMksys = MKSYS_ANTIMONIKER;

    S_OK
}

unsafe fn AntiMonikerROTDataImpl_QueryInterface(
    iface: *mut IROTData,
    riid: *const CLSID,
    ppvObject: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let This = impl_from_IROTData(iface);

    AntiMonikerImpl_QueryInterface(
        core::ptr::addr_of_mut!((*This).IMoniker_iface),
        riid,
        ppvObject,
    )
}

unsafe fn AntiMonikerROTDataImpl_AddRef(iface: *mut IROTData) -> ULONG {
    let This = impl_from_IROTData(iface);

    AntiMonikerImpl_AddRef(core::ptr::addr_of_mut!((*This).IMoniker_iface))
}

unsafe fn AntiMonikerROTDataImpl_Release(iface: *mut IROTData) -> u32 {
    let This = impl_from_IROTData(iface);
    AntiMonikerImpl_Release(core::ptr::addr_of_mut!((*This).IMoniker_iface))
}
unsafe fn AntiMonikerROTDataImpl_GetComparisonData(
    iface: *mut IROTData,
    pbData: *mut u8,
    cbMax: u32,
    pcbData: *mut u32,
) -> HRESULT {
    let constant: u32 = 1;

    *pcbData = (core::mem::size_of::<CLSID>() + core::mem::size_of::<u32>()) as u32;
    if cbMax < *pcbData {
        return 0x8007000E; // E_OUTOFMEMORY
    }

    core::ptr::copy_nonoverlapping(
        &CLSID_AntiMoniker as *const CLSID as *const u8,
        pbData,
        core::mem::size_of::<CLSID>(),
    );
    core::ptr::copy_nonoverlapping(
        &constant as *const u32 as *const u8,
        pbData.add(core::mem::size_of::<CLSID>()),
        core::mem::size_of::<u32>(),
    );

    0 // S_OK
}

unsafe fn AntiMonikerImpl_Construct(This: *mut AntiMonikerImpl) -> HRESULT {
    (*This).IMoniker_iface.lpVtbl = &VT_AntiMonikerImpl;
    (*This).IROTData_iface.lpVtbl = &VT_ROTDataImpl;
    (*This).ref_ = 0;
    (*This).pMarshal = core::ptr::null_mut();

    0
}

#[no_mangle]
pub unsafe extern "system" fn CreateAntiMoniker(ppmk: *mut *mut IMoniker) -> HRESULT {

    TRACE!(b"(%p)\n\0".as_ptr(), ppmk);

    let newAntiMoniker = HeapAlloc(GetProcessHeap(), 0, core::mem::size_of::<AntiMonikerImpl>()) as *mut AntiMonikerImpl;

    if newAntiMoniker.is_null() {
        return 0x80030008u32 as HRESULT;
    }

    let hr = AntiMonikerImpl_Construct(newAntiMoniker);
    if FAILED(hr) != 0 {
        HeapFree(GetProcessHeap(), 0, newAntiMoniker as *mut core::ffi::c_void);
        return hr;
    }

    return AntiMonikerImpl_QueryInterface(
        core::ptr::addr_of_mut!((*newAntiMoniker).IMoniker_iface),
        &IID_IMoniker,
        ppmk as *mut *mut core::ffi::c_void,
    );
}

#[no_mangle]
pub unsafe extern "system" fn AntiMoniker_CreateInstance(
    iface: *mut IClassFactory,
    pUnk: *mut IUnknown,
    riid: REFIID,
    ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {

    TRACE!(
        b"(%p, %s, %p)\n\0".as_ptr(),
        pUnk,
        debugstr_guid(riid),
        ppv,
    );

    *ppv = core::ptr::null_mut();

    if !pUnk.is_null() {
        return CLASS_E_NOAGGREGATION;
    }

    let mut pMoniker: *mut IMoniker = core::ptr::null_mut();
    let mut hr = CreateAntiMoniker(&mut pMoniker);
    if FAILED(hr) {
        return hr;
    }

    hr = IMoniker_QueryInterface(pMoniker, riid, ppv);

    if FAILED(hr) {
        IMoniker_Release(pMoniker);
    }

    hr
}
