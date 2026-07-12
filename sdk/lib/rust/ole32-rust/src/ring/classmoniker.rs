use crate::prelude::*;
use core::ffi::c_void;

extern "C" {
    fn CreateAntiMoniker(ppmk: *mut *mut IMoniker) -> HRESULT;
    fn memcpy(dest: *mut core::ffi::c_void, src: *const core::ffi::c_void, n: usize) -> *mut core::ffi::c_void;
    fn wcschr(s: LPCWSTR, c: u16) -> LPCWSTR;
    fn memset(s: *mut core::ffi::c_void, c: i32, n: usize) -> *mut core::ffi::c_void;
}
extern "C" {
    fn CreateGenericComposite(pmkFirst: *mut IMoniker, pmkRest: *mut IMoniker, ppmkComposite: *mut *mut IMoniker) -> HRESULT;
    fn CoTaskMemFree(pv: *mut core::ffi::c_void);
    fn CreateBindCtx(reserved: DWORD, ppbc: *mut *mut IBindCtx) -> HRESULT;
    fn wcscmp(string1: *const WCHAR, string2: *const WCHAR) -> c_int;
    fn CoTaskMemAlloc(cb: usize) -> *mut core::ffi::c_void;
    fn StringFromGUID2(rguid: *const CLSID, lpsz: LPWSTR, cchMax: i32) -> i32;
}

// ---- file-scope preamble (translated) ----
pub const CHARS_IN_GUID: i32 = 39;

#[repr(C)]
pub struct ClassMoniker {
    pub IMoniker_iface: IMoniker,
    pub IROTData_iface: IROTData,
    pub ref_: i32,
    pub clsid: CLSID,
    pub pMarshal: *mut IUnknown,
}

pub type PClassMoniker = *mut ClassMoniker;

pub static ClassMonikerVtbl: IMonikerVtbl = IMonikerVtbl {
    QueryInterface: ClassMoniker_QueryInterface,
    AddRef: ClassMoniker_AddRef,
    Release: ClassMoniker_Release,
    GetClassID: ClassMoniker_GetClassID,
    IsDirty: ClassMoniker_IsDirty,
    Load: ClassMoniker_Load,
    Save: ClassMoniker_Save,
    GetSizeMax: ClassMoniker_GetSizeMax,
    BindToObject: ClassMoniker_BindToObject,
    BindToStorage: ClassMoniker_BindToStorage,
    Reduce: ClassMoniker_Reduce,
    ComposeWith: ClassMoniker_ComposeWith,
    Enum: ClassMoniker_Enum,
    IsEqual: ClassMoniker_IsEqual,
    Hash: ClassMoniker_Hash,
    IsRunning: ClassMoniker_IsRunning,
    GetTimeOfLastChange: ClassMoniker_GetTimeOfLastChange,
    Inverse: ClassMoniker_Inverse,
    CommonPrefixWith: ClassMoniker_CommonPrefixWith,
    RelativePathTo: ClassMoniker_RelativePathTo,
    GetDisplayName: ClassMoniker_GetDisplayName,
    ParseDisplayName: ClassMoniker_ParseDisplayName,
    IsSystemMoniker: ClassMoniker_IsSystemMoniker,
};

pub static ROTDataVtbl: IROTDataVtbl = IROTDataVtbl {
    QueryInterface: ClassMonikerROTData_QueryInterface,
    AddRef: ClassMonikerROTData_AddRef,
    Release: ClassMonikerROTData_Release,
    GetComparisonData: ClassMonikerROTData_GetComparisonData,
};

unsafe fn impl_from_IMoniker(iface: *mut IMoniker) -> *mut ClassMoniker {
    (iface as *mut u8).offset(-core::mem::offset_of!(ClassMoniker, IMoniker_iface) as isize) as *mut ClassMoniker
}

unsafe fn impl_from_IROTData(iface: *mut IROTData) -> *mut ClassMoniker {
    (iface as *mut u8).offset(-((core::mem::offset_of!(ClassMoniker, IROTData_iface)) as isize)) as *mut ClassMoniker
}

unsafe fn ClassMoniker_QueryInterface(iface: *mut IMoniker, riid: *const CLSID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let This = impl_from_IMoniker(iface);

    TRACE!(b"(%p,%s,%p)\n\0".as_ptr() as *const i8, This, debugstr_guid(riid), ppvObject);

    if ppvObject.is_null() {
        return E_POINTER;
    }

    *ppvObject = core::ptr::null_mut();

    if IsEqualIID(&IID_IUnknown, riid) != 0 ||
       IsEqualIID(&IID_IPersist, riid) != 0 ||
       IsEqualIID(&IID_IPersistStream, riid) != 0 ||
       IsEqualIID(&IID_IMoniker, riid) != 0
    {
        *ppvObject = iface as *mut core::ffi::c_void;
    }
    else if IsEqualIID(&IID_IROTData, riid) != 0 {
        *ppvObject = core::ptr::addr_of_mut!((*This).IROTData_iface) as *mut core::ffi::c_void;
    }
    else if IsEqualIID(&IID_IMarshal, riid) != 0 {
        let mut hr = S_OK;
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

    return S_OK;
}

unsafe fn ClassMoniker_AddRef(iface: *mut IMoniker) -> ULONG {
    let This = impl_from_IMoniker(iface);

    InterlockedIncrement(&mut (*This).ref_) as ULONG
}

unsafe fn ClassMoniker_Release(iface: *mut IMoniker) -> ULONG {
    let This = impl_from_IMoniker(iface);
    let ref_: ULONG;

    TRACE!(b"(%p)\n\0".as_ptr() as *const i8, This);

    ref_ = InterlockedDecrement(&mut (*This).ref_) as ULONG;

    if ref_ == 0 {
        if !(*This).pMarshal.is_null() {
            IUnknown_Release((*This).pMarshal);
        }
        HeapFree(GetProcessHeap(), 0, This as *mut _);
    }

    ref_
}

unsafe fn ClassMoniker_GetClassID(iface: *mut IMoniker, pClassID: *mut CLSID) -> HRESULT {
    TRACE!(b"(%p,%p),stub!\n\0".as_ptr() as *const i8, iface, pClassID);

    if pClassID.is_null() {
        return E_POINTER;
    }

    *pClassID = CLSID_ClassMoniker;

    S_OK
}

unsafe fn ClassMoniker_IsDirty(iface: *mut IMoniker) -> HRESULT {
    TRACE!(b"(%p)\n\0".as_ptr() as *const i8, iface);

    S_FALSE
}

unsafe fn ClassMoniker_Load(iface: *mut IMoniker, pStm: *mut IStream) -> HRESULT {
    let This = impl_from_IMoniker(iface);
    let mut hr: HRESULT;
    let mut zero: DWORD = 0;

    TRACE!(b"(%p)\n\0".as_ptr() as *const i8, pStm);

    hr = IStream_Read(
        pStm,
        core::ptr::addr_of_mut!((*This).clsid) as *mut core::ffi::c_void,
        core::mem::size_of_val(&(*This).clsid) as u32,
        core::ptr::null_mut(),
    );
    if hr != S_OK {
        return STG_E_READFAULT;
    }

    hr = IStream_Read(
        pStm,
        core::ptr::addr_of_mut!(zero) as *mut core::ffi::c_void,
        core::mem::size_of::<DWORD>() as u32,
        core::ptr::null_mut(),
    );
    if hr != S_OK || zero != 0 {
        return STG_E_READFAULT;
    }

    S_OK
}

unsafe fn ClassMoniker_Save(iface: *mut IMoniker, pStm: *mut IStream, fClearDirty: i32) -> HRESULT {
    let This = impl_from_IMoniker(iface);
    let mut hr: HRESULT;
    let zero: DWORD = 0;

    TRACE!(b"(%p, %s)\n\0".as_ptr() as *const i8, pStm, if fClearDirty != 0 { b"TRUE\0".as_ptr() } else { b"FALSE\0".as_ptr() });

    hr = IStream_Write(pStm, core::ptr::addr_of!((*This).clsid) as *const core::ffi::c_void, core::mem::size_of_val(&(*This).clsid) as ULONG, core::ptr::null_mut());
    if FAILED(hr) != 0 { return hr; }

    return IStream_Write(pStm, core::ptr::addr_of!(zero) as *const core::ffi::c_void, core::mem::size_of_val(&zero) as ULONG, core::ptr::null_mut());
}

unsafe fn ClassMoniker_GetSizeMax(
    iface: *mut IMoniker,
    pcbSize: *mut ULARGE_INTEGER,
) -> HRESULT {
    TRACE!(b"(%p)\n\0".as_ptr() as *const i8, pcbSize);

    (*pcbSize).QuadPart = (core::mem::size_of::<CLSID>() + core::mem::size_of::<DWORD>()) as u64;

    S_OK
}
unsafe fn ClassMoniker_BindToObject(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    riid: REFIID,
    ppvResult: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let This = impl_from_IMoniker(iface);
    let mut bindopts: BIND_OPTS2 = core::mem::zeroed();
    let mut pActivator: *mut IClassActivator = core::ptr::null_mut();
    let hr: HRESULT;

    TRACE!(
        b"(%p, %p, %s, %p)\n\0".as_ptr() as *const i8,
        pbc,
        pmkToLeft,
        debugstr_guid(riid),
        ppvResult,
    );

    bindopts.cbStruct = core::mem::size_of::<BIND_OPTS2>() as u32;
    IBindCtx_GetBindOptions(pbc, &mut bindopts as *mut BIND_OPTS2 as *mut BIND_OPTS);

    if pmkToLeft.is_null() {
        return CoGetClassObject(
            &(*This).clsid,
            bindopts.dwClassContext,
            core::ptr::null_mut(),
            riid,
            ppvResult,
        );
    } else {
        hr = IMoniker_BindToObject(
            pmkToLeft,
            pbc,
            core::ptr::null_mut(),
            &IID_IClassActivator,
            &mut pActivator as *mut *mut IClassActivator as *mut *mut core::ffi::c_void,
        );
        if FAILED(hr) {
            return hr;
        }

        hr = IClassActivator_GetClassObject(
            pActivator,
            &(*This).clsid,
            bindopts.dwClassContext,
            bindopts.locale,
            riid,
            ppvResult,
        );

        IClassActivator_Release(pActivator);

        return hr;
    }
}
unsafe fn ClassMoniker_BindToStorage(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    riid: *const IID,
    ppvResult: *mut *mut core::ffi::c_void,
) -> HRESULT {

    TRACE!(
        b"(%p, %p, %s, %p)\n\0".as_ptr() as *const i8,
        pbc,
        pmkToLeft,
        debugstr_guid(riid),
        ppvResult,
    );
    IMoniker_BindToObject(iface, pbc, pmkToLeft, riid, ppvResult)
}

unsafe fn ClassMoniker_Reduce(
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

    IMoniker_AddRef(iface);

    *ppmkReduced = iface;

    MK_S_REDUCED_TO_SELF
}

unsafe fn ClassMoniker_ComposeWith(
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
        res = IMoniker_Enum(pmkRight, 1, &mut penumMk);

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
    } else if fOnlyIfNotGeneric == 0 {
        return CreateGenericComposite(iface, pmkRight, ppmkComposite);
    } else {
        return MK_E_NEEDGENERIC;
    }
}

unsafe fn ClassMoniker_Enum(
    iface: *mut IMoniker,
    fForward: BOOL,
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

unsafe fn ClassMoniker_IsEqual(iface: *mut IMoniker, pmkOtherMoniker: *mut IMoniker) -> HRESULT {
    let mut clsid: CLSID = core::mem::zeroed();
    let mut dispName1: LPOLESTR = core::ptr::null_mut();
    let mut dispName2: LPOLESTR = core::ptr::null_mut();
    let mut bind: *mut IBindCtx = core::ptr::null_mut();
    let mut res: HRESULT = S_FALSE;

    if pmkOtherMoniker.is_null() {
        return S_FALSE;
    }

    if FAILED(IMoniker_GetClassID(pmkOtherMoniker, &mut clsid)) {
        return S_FALSE;
    }
    if IsEqualCLSID(&clsid, &CLSID_ClassMoniker) == 0 {
        return S_FALSE;
    }

    res = CreateBindCtx(0, &mut bind);
    if SUCCEEDED(res) {
        if SUCCEEDED(IMoniker_GetDisplayName(iface, bind, core::ptr::null_mut(), &mut dispName1)) {
            if SUCCEEDED(IMoniker_GetDisplayName(pmkOtherMoniker, bind, core::ptr::null_mut(), &mut dispName2)) {
                if wcscmp(dispName1, dispName2) == 0 {
                    res = S_OK;
                }
                CoTaskMemFree(dispName2 as *mut core::ffi::c_void);
            }
            CoTaskMemFree(dispName1 as *mut core::ffi::c_void);
        }
    }
    res
}

unsafe fn ClassMoniker_Hash(iface: *mut IMoniker, pdwHash: LPDWORD) -> HRESULT {
    let This = impl_from_IMoniker(iface);

    *pdwHash = (*This).clsid.Data1;

    S_OK
}

unsafe fn ClassMoniker_IsRunning(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    pmkNewlyRunning: *mut IMoniker,
) -> HRESULT {
    E_NOTIMPL
}

unsafe fn ClassMoniker_GetTimeOfLastChange(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    pItemTime: *mut FILETIME,
) -> HRESULT {
    TRACE!(b"(%p, %p, %p)\n\0".as_ptr() as *const i8, pbc, pmkToLeft, pItemTime);

    MK_E_UNAVAILABLE
}

unsafe fn ClassMoniker_Inverse(iface: *mut IMoniker, ppmk: *mut *mut IMoniker) -> HRESULT {

    TRACE!(b"(%p)\n\0".as_ptr(), ppmk);

    if ppmk.is_null() {
        return 0x80004003u32 as HRESULT;
    }

    CreateAntiMoniker(ppmk)
}

unsafe fn ClassMoniker_CommonPrefixWith(
    iface: *mut IMoniker,
    pmkOther: *mut IMoniker,
    ppmkPrefix: *mut *mut IMoniker,
) -> HRESULT {
    let mut mkSys: DWORD = 0;

    TRACE!(b"(%p, %p)\n\0".as_ptr() as *const i8, pmkOther, ppmkPrefix);

    *ppmkPrefix = core::ptr::null_mut();

    IMoniker_IsSystemMoniker(pmkOther, &mut mkSys);

    if mkSys == MKSYS_CLASSMONIKER {
        if IMoniker_IsEqual(iface, pmkOther) == S_OK {
            *ppmkPrefix = iface;

            IMoniker_AddRef(iface);

            return MK_S_US;
        } else {
            return MK_E_NOPREFIX;
        }
    } else {
        return MonikerCommonPrefixWith(iface, pmkOther, ppmkPrefix);
    }
}

unsafe fn ClassMoniker_RelativePathTo(
    iface: *mut IMoniker,
    pmOther: *mut IMoniker,
    ppmkRelPath: *mut *mut IMoniker,
) -> HRESULT {
    TRACE!(b"(%p, %p)\n\0".as_ptr() as *const i8, pmOther, ppmkRelPath);

    if ppmkRelPath.is_null() {
        return E_POINTER;
    }

    *ppmkRelPath = core::ptr::null_mut();

    return MK_E_NOTBINDABLE;
}

unsafe fn ClassMoniker_GetDisplayName(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    ppszDisplayName: *mut LPWSTR,
) -> HRESULT {

    let This = impl_from_IMoniker(iface);
    static wszClsidPrefix: [u16; 7] = [b'c' as u16, b'l' as u16, b's' as u16, b'i' as u16, b'd' as u16, b':' as u16, 0];

    if ppszDisplayName.is_null() {
        return E_POINTER;
    }

    if !pmkToLeft.is_null() {
        return E_INVALIDARG;
    }

    let array_size_prefix = wszClsidPrefix.len();
    let total_size = core::mem::size_of_val(&wszClsidPrefix) + (CHARS_IN_GUID - 2) * core::mem::size_of::<u16>();
    *ppszDisplayName = CoTaskMemAlloc(total_size) as LPWSTR;

    StringFromGUID2(
        core::ptr::addr_of!((*This).clsid),
        (*ppszDisplayName).offset((array_size_prefix - 2) as isize),
        CHARS_IN_GUID,
    );

    memcpy(
        *ppszDisplayName as *mut core::ffi::c_void,
        wszClsidPrefix.as_ptr() as *const core::ffi::c_void,
        core::mem::size_of_val(&wszClsidPrefix) - core::mem::size_of::<u16>(),
    );

    *(*ppszDisplayName).offset((array_size_prefix - 2 + CHARS_IN_GUID - 2) as isize) = b':' as u16;
    *(*ppszDisplayName).offset((array_size_prefix - 2 + CHARS_IN_GUID - 1) as isize) = 0;

    S_OK
}
unsafe fn ClassMoniker_ParseDisplayName(
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
        debugstr_w(pszDisplayName as LPCWSTR),
        pchEaten,
        ppmkOut,
    );
    E_NOTIMPL
}

unsafe fn ClassMoniker_IsSystemMoniker(iface: *mut IMoniker, pwdMksys: LPDWORD) -> HRESULT {
    TRACE!(b"(%p,%p)\n\0".as_ptr() as *const i8, iface, pwdMksys);

    if pwdMksys.is_null() {
        return E_POINTER;
    }

    *pwdMksys = MKSYS_CLASSMONIKER;

    S_OK
}

unsafe fn ClassMonikerROTData_QueryInterface(
    iface: *mut IROTData,
    riid: *const CLSID,
    ppvObject: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let This = impl_from_IROTData(iface);

    IMoniker_QueryInterface(
        core::ptr::addr_of_mut!((*This).IMoniker_iface),
        riid,
        ppvObject,
    )
}

unsafe fn ClassMonikerROTData_AddRef(iface: *mut IROTData) -> ULONG {
    let this = impl_from_IROTData(iface);

    TRACE!(b"(%p)\n\0".as_ptr() as *const i8, iface);

    IMoniker_AddRef(&mut (*this).IMoniker_iface)
}

unsafe fn ClassMonikerROTData_Release(iface: *mut IROTData) -> ULONG {
    let This = impl_from_IROTData(iface);

    TRACE!(b"(%p)\n\0".as_ptr() as PCHAR, iface);

    IMoniker_Release(core::ptr::addr_of_mut!((*This).IMoniker_iface))
}

unsafe fn ClassMonikerROTData_GetComparisonData(
    iface: *mut IROTData,
    pbData: *mut u8,
    cbMax: u32,
    pcbData: *mut u32,
) -> HRESULT {
    let This = impl_from_IROTData(iface);

    *pcbData = (2 * core::mem::size_of::<CLSID>()) as u32;
    if cbMax < *pcbData {
        return 0x8007000E; // E_OUTOFMEMORY
    }

    core::ptr::copy_nonoverlapping(
        &CLSID_ClassMoniker as *const CLSID as *const u8,
        pbData,
        core::mem::size_of::<CLSID>(),
    );
    core::ptr::copy_nonoverlapping(
        &(*This).clsid as *const CLSID as *const u8,
        pbData.add(core::mem::size_of::<CLSID>()),
        core::mem::size_of::<CLSID>(),
    );

    0 // S_OK
}

unsafe fn ClassMoniker_Construct(This: *mut ClassMoniker, rclsid: *const CLSID) -> HRESULT {

    TRACE!(b"(%p,%s)\n\0".as_ptr(), This, debugstr_guid(rclsid));

    (*This).IMoniker_iface.lpVtbl = &ClassMonikerVtbl;
    (*This).IROTData_iface.lpVtbl = &ROTDataVtbl;
    (*This).ref_ = 0;
    (*This).clsid = *rclsid;
    (*This).pMarshal = core::ptr::null_mut();

    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn CreateClassMoniker(
    rclsid: *const CLSID,
    ppmk: *mut *mut IMoniker,
) -> HRESULT {

    TRACE!(b"(%s,%p)\n\0".as_ptr(), debugstr_guid(rclsid), ppmk);

    let newClassMoniker = HeapAlloc(GetProcessHeap(), 0, core::mem::size_of::<ClassMoniker>()) as *mut ClassMoniker;

    if newClassMoniker.is_null() {
        return 0x80030008u32 as HRESULT;
    }

    let hr = ClassMoniker_Construct(newClassMoniker, rclsid);

    if FAILED(hr) != 0 {
        HeapFree(GetProcessHeap(), 0, newClassMoniker as *mut core::ffi::c_void);
        return hr;
    }

    return ClassMoniker_QueryInterface(
        core::ptr::addr_of_mut!((*newClassMoniker).IMoniker_iface),
        &IID_IMoniker,
        ppmk as *mut *mut core::ffi::c_void,
    );
}
#[no_mangle]
pub unsafe extern "C" fn ClassMoniker_CreateFromDisplayName(
    pbc: *mut core::ffi::c_void,
    szDisplayName: LPCWSTR,
    pchEaten: LPDWORD,
    ppmk: *mut *mut IMoniker,
) -> HRESULT {

    let mut s: LPCWSTR = wcschr(szDisplayName, ':' as u16);
    let mut end: LPCWSTR;
    let mut clsid: CLSID = core::mem::zeroed();
    let mut table: [u8; 256] = [0; 256];
    let mut i: i32;

    if s.is_null() {
        return 0x800401E4u32 as HRESULT; // MK_E_SYNTAX
    }

    s = s.offset(1);

    end = s;
    while *end != 0 && *end != ':' as u16 {
        end = end.offset(1);
    }

    TRACE!(b"parsing %s\n\0".as_ptr() as *const i8, debugstr_wn(s, (end as isize - s as isize) / 2));

    if *s == '{' as u16 {
        if (end as isize - s as isize) / 2 != 38 || *s.offset(37) != '}' as u16 {
            return 0x800401E4u32 as HRESULT; // MK_E_SYNTAX
        }
        s = s.offset(1);
    } else {
        if (end as isize - s as isize) / 2 != 36 {
            return 0x800401E4u32 as HRESULT; // MK_E_SYNTAX
        }
    }

    i = 0;
    while i < 36 {
        if i == 8 || i == 13 || i == 18 || i == 23 {
            if *s.offset(i as isize) != '-' as u16 {
                return 0x800401E4u32 as HRESULT; // MK_E_SYNTAX
            }
            i += 1;
            continue;
        }
        if !(((*s.offset(i as isize) >= '0' as u16) && (*s.offset(i as isize) <= '9' as u16))
            || ((*s.offset(i as isize) >= 'a' as u16) && (*s.offset(i as isize) <= 'f' as u16))
            || ((*s.offset(i as isize) >= 'A' as u16) && (*s.offset(i as isize) <= 'F' as u16)))
        {
            return 0x800401E4u32 as HRESULT; // MK_E_SYNTAX
        }
        i += 1;
    }

    memset(table.as_mut_ptr() as *mut core::ffi::c_void, 0, 256);

    i = 0;
    while i < 10 {
        table[('0' as u8 + i as u8) as usize] = i as u8;
        i += 1;
    }
    i = 0;
    while i < 6 {
        table[('A' as u8 + i as u8) as usize] = (i + 10) as u8;
        table[('a' as u8 + i as u8) as usize] = (i + 10) as u8;
        i += 1;
    }

    clsid.Data1 = (table[*s.offset(0) as usize] as u32) << 28
        | (table[*s.offset(1) as usize] as u32) << 24
        | (table[*s.offset(2) as usize] as u32) << 20
        | (table[*s.offset(3) as usize] as u32) << 16
        | (table[*s.offset(4) as usize] as u32) << 12
        | (table[*s.offset(5) as usize] as u32) << 8
        | (table[*s.offset(6) as usize] as u32) << 4
        | (table[*s.offset(7) as usize] as u32);
    clsid.Data2 = ((table[*s.offset(9) as usize] as u16) << 12
        | (table[*s.offset(10) as usize] as u16) << 8
        | (table[*s.offset(11) as usize] as u16) << 4
        | (table[*s.offset(12) as usize] as u16)) as u16;
    clsid.Data3 = ((table[*s.offset(14) as usize] as u16) << 12
        | (table[*s.offset(15) as usize] as u16) << 8
        | (table[*s.offset(16) as usize] as u16) << 4
        | (table[*s.offset(17) as usize] as u16)) as u16;

    clsid.Data4[0] = (table[*s.offset(19) as usize] << 4 | table[*s.offset(20) as usize]) as u8;
    clsid.Data4[1] = (table[*s.offset(21) as usize] << 4 | table[*s.offset(22) as usize]) as u8;
    clsid.Data4[2] = (table[*s.offset(24) as usize] << 4 | table[*s.offset(25) as usize]) as u8;
    clsid.Data4[3] = (table[*s.offset(26) as usize] << 4 | table[*s.offset(27) as usize]) as u8;
    clsid.Data4[4] = (table[*s.offset(28) as usize] << 4 | table[*s.offset(29) as usize]) as u8;
    clsid.Data4[5] = (table[*s.offset(30) as usize] << 4 | table[*s.offset(31) as usize]) as u8;
    clsid.Data4[6] = (table[*s.offset(32) as usize] << 4 | table[*s.offset(33) as usize]) as u8;
    clsid.Data4[7] = (table[*s.offset(34) as usize] << 4 | table[*s.offset(35) as usize]) as u8;

    let hr = CreateClassMoniker(&clsid, ppmk);
    if SUCCEEDED(hr) != 0 {
        *pchEaten = ((if *end == ':' as u16 { end.offset(1) } else { end }) as usize
            - szDisplayName as usize) as u32;
    }
    hr
}

#[no_mangle]
pub unsafe extern "system" fn ClassMoniker_CreateInstance(
    iface: *mut IClassFactory,
    pUnk: *mut IUnknown,
    riid: *const IID,
    ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {

    let mut hr: HRESULT;
    let mut pmk: *mut IMoniker = core::ptr::null_mut();

    TRACE!(b"(%p, %s, %p)\n\0".as_ptr(), pUnk, debugstr_guid(riid), ppv);

    *ppv = core::ptr::null_mut();

    if !pUnk.is_null() {
        return CLASS_E_NOAGGREGATION;
    }

    hr = CreateClassMoniker(&CLSID_NULL, &mut pmk);
    if FAILED(hr) != 0 {
        return hr;
    }

    hr = IMoniker_QueryInterface(pmk, riid, ppv);
    IMoniker_Release(pmk);

    return hr;
}
