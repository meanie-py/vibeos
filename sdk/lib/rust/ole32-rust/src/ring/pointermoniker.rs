use crate::prelude::*;
use core::ffi::c_void;

extern "C" {
    fn CreateAntiMoniker(ppmk: *mut *mut IMoniker) -> HRESULT;
}
extern "C" {
    fn CreateGenericComposite(pmkFirst: *mut IMoniker, pmkRest: *mut IMoniker, ppmkComposite: *mut *mut IMoniker) -> HRESULT;
    fn PtrToUlong(p: *const core::ffi::c_void) -> u32;
}

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct PointerMonikerImpl {
    pub IMoniker_iface: IMoniker,
    pub ref_: LONG,
    pub pObject: *mut IUnknown,
}

pub type PPointerMonikerImpl = *mut PointerMonikerImpl;

pub static VT_PointerMonikerImpl: IMonikerVtbl = IMonikerVtbl {
    QueryInterface: PointerMonikerImpl_QueryInterface,
    AddRef: PointerMonikerImpl_AddRef,
    Release: PointerMonikerImpl_Release,
    GetClassID: PointerMonikerImpl_GetClassID,
    IsDirty: PointerMonikerImpl_IsDirty,
    Load: PointerMonikerImpl_Load,
    Save: PointerMonikerImpl_Save,
    GetSizeMax: PointerMonikerImpl_GetSizeMax,
    BindToObject: PointerMonikerImpl_BindToObject,
    BindToStorage: PointerMonikerImpl_BindToStorage,
    Reduce: PointerMonikerImpl_Reduce,
    ComposeWith: PointerMonikerImpl_ComposeWith,
    Enum: PointerMonikerImpl_Enum,
    IsEqual: PointerMonikerImpl_IsEqual,
    Hash: PointerMonikerImpl_Hash,
    IsRunning: PointerMonikerImpl_IsRunning,
    GetTimeOfLastChange: PointerMonikerImpl_GetTimeOfLastChange,
    Inverse: PointerMonikerImpl_Inverse,
    CommonPrefixWith: PointerMonikerImpl_CommonPrefixWith,
    RelativePathTo: PointerMonikerImpl_RelativePathTo,
    GetDisplayName: PointerMonikerImpl_GetDisplayName,
    ParseDisplayName: PointerMonikerImpl_ParseDisplayName,
    IsSystemMoniker: PointerMonikerImpl_IsSystemMoniker,
};

unsafe fn impl_from_IMoniker(iface: *mut IMoniker) -> *mut PointerMonikerImpl {
    (iface as *mut u8).offset(-(core::mem::offset_of!(PointerMonikerImpl, IMoniker_iface) as isize)) as *mut PointerMonikerImpl
}
unsafe fn PointerMonikerImpl_QueryInterface(
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

    if This.is_null() || ppvObject.is_null() {
        return E_INVALIDARG;
    }

    *ppvObject = core::ptr::null_mut();

    if IsEqualIID(core::ptr::addr_of!(IID_IUnknown), riid) != 0
        || IsEqualIID(core::ptr::addr_of!(IID_IPersist), riid) != 0
        || IsEqualIID(core::ptr::addr_of!(IID_IPersistStream), riid) != 0
        || IsEqualIID(core::ptr::addr_of!(IID_IMoniker), riid) != 0
    {
        *ppvObject = iface as *mut core::ffi::c_void;
    }

    if (*ppvObject).is_null() {
        return E_NOINTERFACE;
    }

    IMoniker_AddRef(iface);

    S_OK
}

unsafe fn PointerMonikerImpl_AddRef(iface: *mut IMoniker) -> ULONG {
    let This = impl_from_IMoniker(iface);

    InterlockedIncrement(&mut (*This).ref_) as ULONG
}

unsafe fn PointerMonikerImpl_Release(iface: *mut IMoniker) -> u32 {
    let This = impl_from_IMoniker(iface);
    let ref_: u32;

    TRACE!(b"(%p)\n\0".as_ptr() as *const i8, This);

    ref_ = InterlockedDecrement(&mut (*This).ref_) as u32;

    if ref_ == 0 {
        if !(*This).pObject.is_null() {
            IUnknown_Release((*This).pObject);
        }
        HeapFree(GetProcessHeap(), 0, This as *mut _);
    }

    ref_
}

unsafe fn PointerMonikerImpl_GetClassID(
    iface: *mut IMoniker,
    pClassID: *mut CLSID,
) -> HRESULT {
    TRACE!(b"(%p,%p)\n\0".as_ptr() as *const i8, iface, pClassID);

    if pClassID.is_null() {
        return E_POINTER;
    }

    *pClassID = CLSID_PointerMoniker;

    S_OK
}

unsafe fn PointerMonikerImpl_IsDirty(iface: *mut IMoniker) -> HRESULT {
    TRACE!(b"(%p)\n\0".as_ptr() as *const i8, iface);
    S_FALSE
}
unsafe fn PointerMonikerImpl_Load(
    iface: *mut IMoniker,
    pStm: *mut IStream,
) -> HRESULT {

    TRACE!(b"(%p)\n\0".as_ptr(), pStm);

    E_NOTIMPL
}

unsafe fn PointerMonikerImpl_Save(
    iface: *mut IMoniker,
    pStm: *mut IStream,
    fClearDirty: i32,
) -> HRESULT {

    TRACE!(b"(%p, %d)\n\0".as_ptr(), pStm, fClearDirty);

    0x80004001u32 as HRESULT
}

unsafe fn PointerMonikerImpl_GetSizeMax(
    iface: *mut IMoniker,
    pcbSize: *mut ULARGE_INTEGER,
) -> HRESULT {
    TRACE!(b"(%p,%p)\n\0".as_ptr() as *const i8, iface, pcbSize);

    if pcbSize.is_null() {
        return E_POINTER;
    }

    (*pcbSize).u.__bindgen_anon_1.LowPart = 0;
    (*pcbSize).u.__bindgen_anon_1.HighPart = 0;

    E_NOTIMPL
}

unsafe fn PointerMonikerImpl_BindToObject(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    riid: REFIID,
    ppvResult: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let This = impl_from_IMoniker(iface);

    TRACE!(
        b"(%p,%p,%p,%s,%p)\n\0".as_ptr() as *const i8,
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
unsafe fn PointerMonikerImpl_BindToStorage(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    riid: REFIID,
    ppvResult: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let This = impl_from_IMoniker(iface);

    TRACE!(
        b"(%p,%p,%p,%s,%p)\n\0".as_ptr() as *const i8,
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

unsafe fn PointerMonikerImpl_Reduce(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    dwReduceHowFar: DWORD,
    ppmkToLeft: *mut *mut IMoniker,
    ppmkReduced: *mut *mut IMoniker,
) -> HRESULT {
    TRACE!(
        b"(%p,%p,%d,%p,%p)\n\0".as_ptr() as *const i8,
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
unsafe fn PointerMonikerImpl_ComposeWith(
    iface: *mut IMoniker,
    pmkRight: *mut IMoniker,
    fOnlyIfNotGeneric: u8,
    ppmkComposite: *mut *mut IMoniker,
) -> HRESULT {

    const S_OK: HRESULT = 0;
    const E_POINTER: HRESULT = 0x80004003u32 as HRESULT;
    const MK_E_NEEDGENERIC: HRESULT = 0x800401E2u32 as HRESULT;
    const MKSYS_ANTIMONIKER: u32 = 3;
    const MKSYS_GENERICCOMPOSITE: u32 = 1;

    let mut res: HRESULT = S_OK;
    let mut mkSys: u32 = 0;
    let mut mkSys2: u32 = 0;
    let mut penumMk: *mut IEnumMoniker = core::ptr::null_mut();
    let mut pmostLeftMk: *mut IMoniker = core::ptr::null_mut();
    let mut tempMkComposite: *mut IMoniker = core::ptr::null_mut();

    TRACE!(b"(%p,%d,%p)\n\0".as_ptr(), pmkRight, fOnlyIfNotGeneric, ppmkComposite);

    if ppmkComposite.is_null() || pmkRight.is_null() {
        return E_POINTER;
    }

    *ppmkComposite = core::ptr::null_mut();

    IMoniker_IsSystemMoniker(pmkRight, &mut mkSys);

    if mkSys == MKSYS_ANTIMONIKER {
        return res;
    } else if mkSys == MKSYS_GENERICCOMPOSITE {
        res = IMoniker_Enum(pmkRight, 1u8, &mut penumMk);

        if FAILED(res) != 0 {
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
            return res;
        } else {
            return CreateGenericComposite(iface, pmkRight, ppmkComposite);
        }
    } else {
        if fOnlyIfNotGeneric == 0 {
            return CreateGenericComposite(iface, pmkRight, ppmkComposite);
        } else {
            return MK_E_NEEDGENERIC;
        }
    }
}

unsafe fn PointerMonikerImpl_Enum(
    iface: *mut IMoniker,
    fForward: i32,
    ppenumMoniker: *mut *mut IEnumMoniker,
) -> HRESULT {
    TRACE!(
        b"(%p,%d,%p)\n\0".as_ptr() as *const i8,
        iface,
        fForward,
        ppenumMoniker,
    );

    if ppenumMoniker.is_null() {
        return E_POINTER;
    }

    *ppenumMoniker = core::ptr::null_mut();

    return S_OK;
}
unsafe fn PointerMonikerImpl_IsEqual(
    iface: *mut IMoniker,
    pmkOtherMoniker: *mut IMoniker,
) -> HRESULT {
    let This = impl_from_IMoniker(iface);
    let mut mkSys: u32 = 0;

    TRACE!(b"(%p,%p)\n\0".as_ptr() as *const i8, iface, pmkOtherMoniker);

    if pmkOtherMoniker.is_null() {
        return 1;
    }

    IMoniker_IsSystemMoniker(pmkOtherMoniker, &mut mkSys);

    if mkSys == 3 {
        let pOtherMoniker = impl_from_IMoniker(pmkOtherMoniker);
        if (*This).pObject == (*pOtherMoniker).pObject {
            0
        } else {
            1
        }
    } else {
        1
    }
}
unsafe fn PointerMonikerImpl_Hash(
    iface: *mut IMoniker,
    pdwHash: *mut u32,
) -> HRESULT {
    let This = impl_from_IMoniker(iface);

    if pdwHash.is_null() {
        return 0x80004003u32 as HRESULT;
    }

    *pdwHash = PtrToUlong((*This).pObject as *const core::ffi::c_void);

    return 0x00000000u32 as HRESULT;
}

unsafe fn PointerMonikerImpl_IsRunning(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    pmkNewlyRunning: *mut IMoniker,
) -> HRESULT {
    TRACE!(
        b"(%p,%p,%p,%p)\n\0".as_ptr() as *const i8,
        iface,
        pbc,
        pmkToLeft,
        pmkNewlyRunning,
    );

    S_OK
}

unsafe fn PointerMonikerImpl_GetTimeOfLastChange(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    pAntiTime: *mut FILETIME,
) -> HRESULT {
    TRACE!(b"(%p,%p,%p,%p)\n\0".as_ptr(), iface, pbc, pmkToLeft, pAntiTime);
    E_NOTIMPL
}

unsafe fn PointerMonikerImpl_Inverse(
    iface: *mut IMoniker,
    ppmk: *mut *mut IMoniker,
) -> HRESULT {

    TRACE!(b"(%p,%p)\n\0".as_ptr(), iface, ppmk);

    CreateAntiMoniker(ppmk)
}
unsafe fn PointerMonikerImpl_CommonPrefixWith(
    iface: *mut IMoniker,
    pmkOther: *mut IMoniker,
    ppmkPrefix: *mut *mut IMoniker,
) -> HRESULT {
    TRACE!(b"(%p, %p)\n\0".as_ptr() as _, pmkOther, ppmkPrefix);

    *ppmkPrefix = core::ptr::null_mut();

    if PointerMonikerImpl_IsEqual(iface, pmkOther) == 0 {
        IMoniker_AddRef(iface);
        *ppmkPrefix = iface;
        return MK_S_US;
    } else {
        return MK_E_NOPREFIX;
    }
}

unsafe fn PointerMonikerImpl_RelativePathTo(
    iface: *mut IMoniker,
    pmOther: *mut IMoniker,
    ppmkRelPath: *mut *mut IMoniker,
) -> HRESULT {
    TRACE!(b"(%p,%p,%p)\n\0".as_ptr() as *const i8, iface, pmOther, ppmkRelPath);

    if ppmkRelPath.is_null() {
        return E_POINTER;
    }

    *ppmkRelPath = core::ptr::null_mut();

    return E_NOTIMPL;
}

unsafe fn PointerMonikerImpl_GetDisplayName(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    ppszDisplayName: *mut LPOLESTR
) -> HRESULT {
    if ppszDisplayName.is_null() {
        return E_POINTER;
    }

    *ppszDisplayName = core::ptr::null_mut();
    E_NOTIMPL
}

unsafe fn PointerMonikerImpl_ParseDisplayName(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    pszDisplayName: LPOLESTR,
    pchEaten: *mut ULONG,
    ppmkOut: *mut *mut IMoniker,
) -> HRESULT {
    let This = impl_from_IMoniker(iface);
    let mut hr: HRESULT;
    let mut pPDN: *mut IParseDisplayName = core::ptr::null_mut();

    TRACE!(
        b"(%p,%p,%p,%p,%p,%p)\n\0".as_ptr() as *const i8,
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

    hr = IUnknown_QueryInterface(
        (*This).pObject,
        &IID_IParseDisplayName,
        &mut pPDN as *mut *mut IParseDisplayName as *mut *mut core::ffi::c_void,
    );
    if FAILED(hr) {
        return hr;
    }

    hr = IParseDisplayName_ParseDisplayName(pPDN, pbc, pszDisplayName, pchEaten, ppmkOut);
    IParseDisplayName_Release(pPDN);

    hr
}

unsafe fn PointerMonikerImpl_IsSystemMoniker(
    iface: *mut IMoniker,
    pwdMksys: *mut DWORD,
) -> HRESULT {
    TRACE!(b"(%p,%p)\n\0".as_ptr() as *const i8, iface, pwdMksys);

    if pwdMksys.is_null() {
        return E_POINTER;
    }

    *pwdMksys = MKSYS_POINTERMONIKER;

    S_OK
}

unsafe fn PointerMonikerImpl_Construct(This: *mut PointerMonikerImpl, punk: *mut IUnknown) {

    TRACE!(b"(%p)\n\0".as_ptr(), This);

    (*This).IMoniker_iface.lpVtbl = &VT_PointerMonikerImpl as *const IMonikerVtbl;
    (*This).ref_ = 1;
    if !punk.is_null() {
        IUnknown_AddRef(punk);
    }
    (*This).pObject = punk;
}

#[no_mangle]
pub unsafe extern "system" fn CreatePointerMoniker(punk: *mut IUnknown, ppmk: *mut *mut IMoniker) -> HRESULT {

    TRACE!(b"(%p, %p)\n\0".as_ptr(), punk, ppmk);

    if ppmk.is_null() {
        return E_INVALIDARG;
    }

    let This = HeapAlloc(GetProcessHeap(), 0, core::mem::size_of::<PointerMonikerImpl>()) as *mut PointerMonikerImpl;
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
    iface: *mut IClassFactory,
    pUnk: *mut IUnknown,
    riid: *const CLSID,
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
        return 0x80040110u32 as HRESULT;
    }

    let mut pMoniker: *mut IMoniker = core::ptr::null_mut();
    let mut hr = CreatePointerMoniker(core::ptr::null_mut(), &mut pMoniker);
    if FAILED(hr) != 0 {
        return hr;
    }

    hr = IMoniker_QueryInterface(pMoniker, riid, ppv);

    if FAILED(hr) != 0 {
        IMoniker_Release(pMoniker);
    }

    hr
}
