use crate::prelude::*;
use core::ffi::c_void;

extern "C" {
    fn MonikerMarshal_Create(iface: *mut IMoniker, ppMarshal: *mut *mut IUnknown) -> HRESULT;
    static IID_IUnknown: IID;
    static IID_IPersist: IID;
    static IID_IPersistStream: IID;
    static IID_IMoniker: IID;
    static IID_IROTData: IID;
    static IID_IMarshal: IID;
    static IID_IOleItemContainer: IID;
    fn towupper(c: u16) -> u16;
}
extern "C" {
    fn CreateGenericComposite(pmkFirst: *mut IMoniker, pmkRest: *mut IMoniker, ppmkComposite: *mut *mut IMoniker) -> HRESULT;
    fn CoTaskMemFree(pv: *mut core::ffi::c_void);
    fn CreateBindCtx(reserved: u32, ppbc: *mut *mut IBindCtx) -> HRESULT;
    fn wcscmp(string1: *const u16, string2: *const u16) -> i32;
    fn CreateAntiMoniker(ppmk: *mut *mut IMoniker) -> HRESULT;
    fn MultiByteToWideChar(CodePage: u32, dwFlags: u32, lpMultiByteStr: *const i8, cbMultiByte: i32, lpWideCharStr: *mut u16, cchWideChar: i32) -> i32;
    fn WideCharToMultiByte( CodePage: u32, dwFlags: u32, lpWideCharStr: *const u16, cchWideChar: i32, lpMultiByteStr: *mut i8, cbMultiByte: i32, lpDefaultChar: *const i8, lpUsedDefaultChar: *mut i32, ) -> i32;
    fn CoTaskMemAlloc(cb: usize) -> *mut core::ffi::c_void;
    fn lstrcatW(lpString1: LPWSTR, lpString2: LPWSTR) -> LPWSTR;
}

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct ItemMonikerImpl {
    pub IMoniker_iface: IMoniker,
    pub IROTData_iface: IROTData,
    pub ref_: LONG,
    pub itemName: LPOLESTR,
    pub itemDelimiter: LPOLESTR,
    pub pMarshal: *mut IUnknown,
}

pub type PItemMonikerImpl = *mut ItemMonikerImpl;

pub static VT_ItemMonikerImpl: IMonikerVtbl = IMonikerVtbl {
    QueryInterface: ItemMonikerImpl_QueryInterface,
    AddRef: ItemMonikerImpl_AddRef,
    Release: ItemMonikerImpl_Release,
    GetClassID: ItemMonikerImpl_GetClassID,
    IsDirty: ItemMonikerImpl_IsDirty,
    Load: ItemMonikerImpl_Load,
    Save: ItemMonikerImpl_Save,
    GetSizeMax: ItemMonikerImpl_GetSizeMax,
    BindToObject: ItemMonikerImpl_BindToObject,
    BindToStorage: ItemMonikerImpl_BindToStorage,
    Reduce: ItemMonikerImpl_Reduce,
    ComposeWith: ItemMonikerImpl_ComposeWith,
    Enum: ItemMonikerImpl_Enum,
    IsEqual: ItemMonikerImpl_IsEqual,
    Hash: ItemMonikerImpl_Hash,
    IsRunning: ItemMonikerImpl_IsRunning,
    GetTimeOfLastChange: ItemMonikerImpl_GetTimeOfLastChange,
    Inverse: ItemMonikerImpl_Inverse,
    CommonPrefixWith: ItemMonikerImpl_CommonPrefixWith,
    RelativePathTo: ItemMonikerImpl_RelativePathTo,
    GetDisplayName: ItemMonikerImpl_GetDisplayName,
    ParseDisplayName: ItemMonikerImpl_ParseDisplayName,
    IsSystemMoniker: ItemMonikerImpl_IsSystemMoniker,
};

pub static VT_ROTDataImpl: IROTDataVtbl = IROTDataVtbl {
    QueryInterface: ItemMonikerROTDataImpl_QueryInterface,
    AddRef: ItemMonikerROTDataImpl_AddRef,
    Release: ItemMonikerROTDataImpl_Release,
    GetComparisonData: ItemMonikerROTDataImpl_GetComparisonData,
};

unsafe fn impl_from_IMoniker(iface: *mut IMoniker) -> *mut ItemMonikerImpl {
    (iface as *mut u8).offset(-(core::mem::offset_of!(ItemMonikerImpl, IMoniker_iface) as isize)) as *mut ItemMonikerImpl
}

unsafe fn impl_from_IROTData(iface: *mut IROTData) -> *mut ItemMonikerImpl {
    (iface as *mut u8).offset(-(core::mem::offset_of!(ItemMonikerImpl, IROTData_iface) as isize)) as *mut ItemMonikerImpl
}

unsafe fn ItemMonikerImpl_QueryInterface(iface: *mut IMoniker, riid: *const IID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {

    let This = impl_from_IMoniker(iface);

    TRACE!(b"(%p,%s,%p)\n\0".as_ptr() as *const i8, This, debugstr_guid(riid), ppvObject);

    if ppvObject.is_null() {
        return 0x80070057u32 as HRESULT;
    }

    if IsEqualIID(core::ptr::addr_of!(IID_IUnknown), riid) != 0 ||
       IsEqualIID(core::ptr::addr_of!(IID_IPersist), riid) != 0 ||
       IsEqualIID(core::ptr::addr_of!(IID_IPersistStream), riid) != 0 ||
       IsEqualIID(core::ptr::addr_of!(IID_IMoniker), riid) != 0 {
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
    } else {
        *ppvObject = core::ptr::null_mut();
        return 0x80004002u32 as HRESULT;
    }

    IMoniker_AddRef(iface);
    0
}

unsafe fn ItemMonikerImpl_AddRef(iface: *mut IMoniker) -> u32 {
    let This = impl_from_IMoniker(iface);

    InterlockedIncrement(&mut (*This).ref_) as u32
}

unsafe fn ItemMonikerImpl_Release(iface: *mut IMoniker) -> u32 {
    let This = impl_from_IMoniker(iface);
    let ref_: u32;

    ref_ = InterlockedDecrement(&mut (*This).ref_) as u32;

    if ref_ == 0 {
        ItemMonikerImpl_Destroy(This);
    }

    ref_
}
unsafe fn ItemMonikerImpl_GetClassID(iface: *mut IMoniker, pClassID: *mut CLSID) -> HRESULT {
    TRACE!(b"(%p,%p)\n\0".as_ptr() as *const i8, iface, pClassID);

    if pClassID.is_null() {
        return E_POINTER;
    }

    *pClassID = CLSID_ItemMoniker;

    S_OK
}
unsafe fn ItemMonikerImpl_IsDirty(iface: *mut IMoniker) -> HRESULT {
    TRACE!(b"(%p)\n\0".as_ptr() as *const i8, iface);

    S_FALSE
}

unsafe fn ItemMonikerImpl_Load(iface: *mut IMoniker, pStm: *mut IStream) -> HRESULT {

    const CP_ACP: u32 = 0;
    const E_FAIL: HRESULT = 0x80004005u32 as i32;
    const E_OUTOFMEMORY: HRESULT = 0x8007000Eu32 as i32;

    let This = impl_from_IMoniker(iface);
    let mut res: HRESULT;
    let mut delimiterLength: u32 = 0;
    let mut nameLength: u32 = 0;
    let mut lenW: i32;
    let mut itemNameA: *mut i8;
    let mut itemDelimiterA: *mut i8;
    let mut bread: u32 = 0;

    TRACE!(b"\n\0".as_ptr() as *const i8);

    res = IStream_Read(pStm, core::ptr::addr_of_mut!(delimiterLength) as *mut core::ffi::c_void, core::mem::size_of::<u32>() as u32, core::ptr::addr_of_mut!(bread));
    if bread != core::mem::size_of::<u32>() as u32 {
        return E_FAIL;
    }

    itemDelimiterA = HeapAlloc(GetProcessHeap(), 0, delimiterLength as usize) as *mut i8;
    if itemDelimiterA.is_null() {
        return E_OUTOFMEMORY;
    }
    res = IStream_Read(pStm, itemDelimiterA as *mut core::ffi::c_void, delimiterLength, core::ptr::addr_of_mut!(bread));
    if bread != delimiterLength {
        HeapFree(GetProcessHeap(), 0, itemDelimiterA as *mut core::ffi::c_void);
        return E_FAIL;
    }

    lenW = MultiByteToWideChar(CP_ACP, 0, itemDelimiterA, -1, core::ptr::null_mut(), 0);
    (*This).itemDelimiter = HeapReAlloc(GetProcessHeap(), 0, (*This).itemDelimiter as *mut core::ffi::c_void, (lenW as usize) * core::mem::size_of::<u16>()) as LPWSTR;
    if (*This).itemDelimiter.is_null() {
        HeapFree(GetProcessHeap(), 0, itemDelimiterA as *mut core::ffi::c_void);
        return E_OUTOFMEMORY;
    }
    MultiByteToWideChar(CP_ACP, 0, itemDelimiterA, -1, (*This).itemDelimiter, lenW);
    HeapFree(GetProcessHeap(), 0, itemDelimiterA as *mut core::ffi::c_void);

    res = IStream_Read(pStm, core::ptr::addr_of_mut!(nameLength) as *mut core::ffi::c_void, core::mem::size_of::<u32>() as u32, core::ptr::addr_of_mut!(bread));
    if bread != core::mem::size_of::<u32>() as u32 {
        return E_FAIL;
    }

    itemNameA = HeapAlloc(GetProcessHeap(), 0, nameLength as usize) as *mut i8;
    if itemNameA.is_null() {
        return E_OUTOFMEMORY;
    }
    res = IStream_Read(pStm, itemNameA as *mut core::ffi::c_void, nameLength, core::ptr::addr_of_mut!(bread));
    if bread != nameLength {
        HeapFree(GetProcessHeap(), 0, itemNameA as *mut core::ffi::c_void);
        return E_FAIL;
    }

    lenW = MultiByteToWideChar(CP_ACP, 0, itemNameA, -1, core::ptr::null_mut(), 0);
    (*This).itemName = HeapReAlloc(GetProcessHeap(), 0, (*This).itemName as *mut core::ffi::c_void, (lenW as usize) * core::mem::size_of::<u16>()) as LPWSTR;
    if (*This).itemName.is_null() {
        HeapFree(GetProcessHeap(), 0, itemNameA as *mut core::ffi::c_void);
        return E_OUTOFMEMORY;
    }
    MultiByteToWideChar(CP_ACP, 0, itemNameA, -1, (*This).itemName, lenW);
    HeapFree(GetProcessHeap(), 0, itemNameA as *mut core::ffi::c_void);

    res
}

unsafe fn ItemMonikerImpl_Save(iface: *mut IMoniker, pStm: *mut IStream, fClearDirty: i32) -> HRESULT {

    let This = impl_from_IMoniker(iface);
    let mut res: HRESULT;

    let nameLength = WideCharToMultiByte(65001, 0, (*This).itemName, -1, core::ptr::null_mut(), 0, core::ptr::null(), core::ptr::null_mut()) as u32;
    let delimiterLength = WideCharToMultiByte(65001, 0, (*This).itemDelimiter, -1, core::ptr::null_mut(), 0, core::ptr::null(), core::ptr::null_mut()) as u32;
    let itemNameA = HeapAlloc(GetProcessHeap(), 0, nameLength as usize) as *mut i8;
    let itemDelimiterA = HeapAlloc(GetProcessHeap(), 0, delimiterLength as usize) as *mut i8;
    WideCharToMultiByte(65001, 0, (*This).itemName, -1, itemNameA, nameLength as i32, core::ptr::null(), core::ptr::null_mut());
    WideCharToMultiByte(65001, 0, (*This).itemDelimiter, -1, itemDelimiterA, delimiterLength as i32, core::ptr::null(), core::ptr::null_mut());

    TRACE!(b"%p, %s\n\0".as_ptr() as *const i8, pStm, if fClearDirty != 0 { b"TRUE\0".as_ptr() } else { b"FALSE\0".as_ptr() });

    res = IStream_Write(pStm, &delimiterLength as *const u32 as *const core::ffi::c_void, core::mem::size_of::<u32>() as u32, core::ptr::null_mut());
    res = IStream_Write(pStm, itemDelimiterA as *const core::ffi::c_void, delimiterLength * core::mem::size_of::<i8>() as u32, core::ptr::null_mut());
    res = IStream_Write(pStm, &nameLength as *const u32 as *const core::ffi::c_void, core::mem::size_of::<u32>() as u32, core::ptr::null_mut());
    res = IStream_Write(pStm, itemNameA as *const core::ffi::c_void, nameLength * core::mem::size_of::<i8>() as u32, core::ptr::null_mut());

    HeapFree(GetProcessHeap(), 0, itemNameA as *mut core::ffi::c_void);
    HeapFree(GetProcessHeap(), 0, itemDelimiterA as *mut core::ffi::c_void);

    res
}

unsafe fn ItemMonikerImpl_GetSizeMax(iface: *mut IMoniker, pcbSize: *mut ULARGE_INTEGER) -> HRESULT {

    let This = impl_from_IMoniker(iface);
    let delimiterLength = lstrlenW((*This).itemDelimiter) + 1;
    let nameLength = lstrlenW((*This).itemName) + 1;

    if pcbSize.is_null() {
        return 0x80004003u32 as HRESULT;
    }

    (*pcbSize).u.__bindgen_anon_1.LowPart = core::mem::size_of::<u32>() as u32 +
                           (delimiterLength as u32) * 4 +
                           core::mem::size_of::<u32>() as u32 +
                           (nameLength as u32) * 4 +
                           18;
    (*pcbSize).u.__bindgen_anon_1.HighPart = 0;

    0
}
unsafe fn ItemMonikerImpl_BindToObject(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    riid: REFIID,
    ppvResult: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let This = impl_from_IMoniker(iface);
    let mut res: HRESULT;
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

    res = IMoniker_BindToObject(
        pmkToLeft,
        pbc,
        core::ptr::null_mut(),
        &refid,
        &mut poic as *mut _ as *mut *mut core::ffi::c_void,
    );

    if SUCCEEDED(res) != 0 {
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

unsafe fn ItemMonikerImpl_BindToStorage(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    riid: *const IID,
    ppvResult: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let This = impl_from_IMoniker(iface);
    let mut res: HRESULT;
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
        return 0x80070057u32 as HRESULT;
    }

    res = IMoniker_BindToObject(
        pmkToLeft,
        pbc,
        core::ptr::null_mut(),
        &IID_IOleItemContainer,
        &mut poic as *mut *mut IOleItemContainer as *mut *mut core::ffi::c_void,
    );

    if SUCCEEDED(res) != 0 {
        res = IOleItemContainer_GetObjectStorage(poic, (*This).itemName, pbc, riid, ppvResult);

        IOleItemContainer_Release(poic);
    }

    res
}

unsafe fn ItemMonikerImpl_Reduce(
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

    ItemMonikerImpl_AddRef(iface);

    *ppmkReduced = iface;

    return MK_S_REDUCED_TO_SELF;
}

unsafe fn ItemMonikerImpl_ComposeWith(
    iface: *mut IMoniker,
    pmkRight: *mut IMoniker,
    fOnlyIfNotGeneric: u8,
    ppmkComposite: *mut *mut IMoniker,
) -> HRESULT {

    let mut res: HRESULT = 0;
    let mut mkSys: u32 = 0;
    let mut mkSys2: u32 = 0;
    let mut penumMk: *mut IEnumMoniker = core::ptr::null_mut();
    let mut pmostLeftMk: *mut IMoniker = core::ptr::null_mut();
    let mut tempMkComposite: *mut IMoniker = core::ptr::null_mut();

    TRACE!(b"(%p,%p,%d,%p)\n\0".as_ptr(), iface, pmkRight, fOnlyIfNotGeneric, ppmkComposite);

    if ppmkComposite.is_null() || pmkRight.is_null() {
        return 0x80004003u32 as HRESULT;
    }

    *ppmkComposite = core::ptr::null_mut();

    IMoniker_IsSystemMoniker(pmkRight, &mut mkSys);

    if mkSys == 3 {
        return res;
    } else if mkSys == 1 {
        res = IMoniker_Enum(pmkRight, 1, &mut penumMk);

        if FAILED(res) != 0 {
            return res;
        }

        res = IEnumMoniker_Next(penumMk, 1, &mut pmostLeftMk, core::ptr::null_mut());

        IMoniker_IsSystemMoniker(pmostLeftMk, &mut mkSys2);

        if mkSys2 == 3 {
            IMoniker_Release(pmostLeftMk);

            tempMkComposite = iface;
            IMoniker_AddRef(iface);

            while IEnumMoniker_Next(penumMk, 1, &mut pmostLeftMk, core::ptr::null_mut()) == 0 {
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
            return 0x80040013u32 as HRESULT;
        }
    }
}

unsafe fn ItemMonikerImpl_Enum(
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

unsafe fn ItemMonikerImpl_IsEqual(iface: *mut IMoniker, pmkOtherMoniker: *mut IMoniker) -> HRESULT {

    let mut clsid: CLSID = core::mem::zeroed();
    let mut dispName1: LPWSTR = core::ptr::null_mut();
    let mut dispName2: LPWSTR = core::ptr::null_mut();
    let mut bind: *mut IBindCtx = core::ptr::null_mut();
    let mut res: HRESULT = 0x00000001; // S_FALSE

    TRACE!(b"(%p,%p)\n\0".as_ptr(), iface, pmkOtherMoniker);

    if pmkOtherMoniker.is_null() {
        return 0x00000001; // S_FALSE
    }

    if FAILED(IMoniker_GetClassID(pmkOtherMoniker, &mut clsid)) != 0 {
        return 0x00000001; // S_FALSE
    }
    if IsEqualCLSID(&clsid, &CLSID_ItemMoniker) == 0 {
        return 0x00000001; // S_FALSE
    }

    res = CreateBindCtx(0, &mut bind);
    if SUCCEEDED(res) != 0 {
        if SUCCEEDED(IMoniker_GetDisplayName(iface, bind, core::ptr::null_mut(), &mut dispName1)) != 0 {
            if SUCCEEDED(IMoniker_GetDisplayName(pmkOtherMoniker, bind, core::ptr::null_mut(), &mut dispName2)) != 0 {
                if wcscmp(dispName1, dispName2) == 0 {
                    res = 0x00000000; // S_OK
                }
                CoTaskMemFree(dispName2 as *mut core::ffi::c_void);
            }
            CoTaskMemFree(dispName1 as *mut core::ffi::c_void);
        }
    }
    res
}

unsafe fn ItemMonikerImpl_Hash(iface: *mut IMoniker, pdwHash: *mut u32) -> HRESULT {

    let This = impl_from_IMoniker(iface);
    let mut h: u32 = 0;
    let mut off: i32 = 0;

    if pdwHash.is_null() {
        return 0x80004003u32 as HRESULT;
    }

    let val = (*This).itemName;
    let len = lstrlenW(val);

    let mut i = len;
    while i > 0 {
        h = h.wrapping_mul(3) ^ (towupper(*val.offset(off as isize)) as u32);
        off += 1;
        i -= 1;
    }

    *pdwHash = h;

    0x00000000u32 as HRESULT
}

unsafe fn ItemMonikerImpl_IsRunning(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    pmkNewlyRunning: *mut IMoniker,
) -> HRESULT {
    let this = impl_from_IMoniker(iface);
    let mut rot: *mut IRunningObjectTable = core::ptr::null_mut();
    let mut res: HRESULT;
    let mut poic: *mut IOleItemContainer = core::ptr::null_mut();

    TRACE!(
        b"(%p,%p,%p,%p)\n\0".as_ptr() as *const i8,
        iface,
        pbc,
        pmkToLeft,
        pmkNewlyRunning,
    );

    if pmkToLeft.is_null() {
        if !pmkNewlyRunning.is_null()
            && IMoniker_IsEqual(pmkNewlyRunning, iface) == S_OK
        {
            return S_OK;
        } else {
            if pbc.is_null() {
                return E_INVALIDARG;
            }

            res = IBindCtx_GetRunningObjectTable(pbc, &mut rot);

            if FAILED(res) {
                return res;
            }

            res = IRunningObjectTable_IsRunning(rot, iface);

            IRunningObjectTable_Release(rot);
        }
    } else {
        res = IMoniker_BindToObject(
            pmkToLeft,
            pbc,
            core::ptr::null_mut(),
            &IID_IOleItemContainer,
            &mut poic as *mut *mut IOleItemContainer as *mut *mut core::ffi::c_void,
        );

        if SUCCEEDED(res) {
            res = IOleItemContainer_IsRunning(poic, (*this).itemName);

            IOleItemContainer_Release(poic);
        }
    }

    res
}

unsafe fn ItemMonikerImpl_GetTimeOfLastChange(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    pItemTime: *mut FILETIME,
) -> HRESULT {

    TRACE!(
        b"(%p,%p,%p,%p)\n\0".as_ptr(),
        iface,
        pbc,
        pmkToLeft,
        pItemTime,
    );

    if pItemTime.is_null() {
        return E_INVALIDARG;
    }

    if pmkToLeft.is_null() {
        return MK_E_NOTBINDABLE;
    } else {
        let mut compositeMk: *mut IMoniker = core::ptr::null_mut();
        let mut res = CreateGenericComposite(pmkToLeft, iface, &mut compositeMk);
        if FAILED(res) != 0 {
            return res;
        }

        let mut rot: *mut IRunningObjectTable = core::ptr::null_mut();
        res = IBindCtx_GetRunningObjectTable(pbc, &mut rot);
        if FAILED(res) != 0 {
            IMoniker_Release(compositeMk);
            return res;
        }

        if IRunningObjectTable_GetTimeOfLastChange(rot, compositeMk, pItemTime) != S_OK {
            res = IMoniker_GetTimeOfLastChange(pmkToLeft, pbc, core::ptr::null_mut(), pItemTime);
        }

        IMoniker_Release(compositeMk);

        return res;
    }
}

unsafe fn ItemMonikerImpl_Inverse(iface: *mut IMoniker, ppmk: *mut *mut IMoniker) -> HRESULT {

    TRACE!(b"(%p,%p)\n\0".as_ptr(), iface, ppmk);

    if ppmk.is_null() {
        return E_POINTER;
    }

    CreateAntiMoniker(ppmk)
}

unsafe fn ItemMonikerImpl_CommonPrefixWith(
    iface: *mut IMoniker,
    pmkOther: *mut IMoniker,
    ppmkPrefix: *mut *mut IMoniker,
) -> HRESULT {
    let mut mkSys: u32 = 0;

    TRACE!(b"(%p,%p)\n\0".as_ptr() as *const i8, pmkOther, ppmkPrefix);

    IMoniker_IsSystemMoniker(pmkOther, &mut mkSys);

    if mkSys == 1 && IMoniker_IsEqual(iface, pmkOther) == 0 {
        *ppmkPrefix = iface;
        IMoniker_AddRef(iface);
        return 0x000401E4;
    } else {
        return MonikerCommonPrefixWith(iface, pmkOther, ppmkPrefix);
    }
}

unsafe fn ItemMonikerImpl_RelativePathTo(
    iface: *mut IMoniker,
    pmOther: *mut IMoniker,
    ppmkRelPath: *mut *mut IMoniker
) -> HRESULT {
    TRACE!(b"(%p,%p,%p)\n\0".as_ptr() as _, iface, pmOther, ppmkRelPath);

    if ppmkRelPath.is_null() {
        return E_POINTER;
    }

    *ppmkRelPath = core::ptr::null_mut();

    return MK_E_NOTBINDABLE;
}

unsafe fn ItemMonikerImpl_GetDisplayName(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    ppszDisplayName: *mut LPWSTR,
) -> HRESULT {

    let This = impl_from_IMoniker(iface);

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
        return 0x80070057u32 as HRESULT;
    }

    *ppszDisplayName = CoTaskMemAlloc(
        (core::mem::size_of::<u16>()
            * (lstrlenW((*This).itemDelimiter) + lstrlenW((*This).itemName) + 1) as usize),
    ) as LPWSTR;

    if (*ppszDisplayName).is_null() {
        return 0x8007000Eu32 as HRESULT;
    }

    lstrcpyW(*ppszDisplayName, (*This).itemDelimiter);
    lstrcatW(*ppszDisplayName, (*This).itemName);

    TRACE!(b"-- %s\n\0".as_ptr(), debugstr_w(*ppszDisplayName));

    0
}

unsafe fn ItemMonikerImpl_ParseDisplayName(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    pszDisplayName: LPWSTR,
    pchEaten: *mut ULONG,
    ppmkOut: *mut *mut IMoniker,
) -> HRESULT {
    let This = impl_from_IMoniker(iface);
    let mut poic: *mut IOleItemContainer = core::ptr::null_mut();
    let mut ppdn: *mut IParseDisplayName = core::ptr::null_mut();
    let mut displayName: LPWSTR = core::ptr::null_mut();
    let mut res: HRESULT;

    TRACE!(b"%s\n\0".as_ptr() as *const i8, debugstr_w(pszDisplayName));

    if pmkToLeft.is_null() {
        return MK_E_SYNTAX;
    } else {
        res = IMoniker_BindToObject(
            pmkToLeft,
            pbc,
            core::ptr::null_mut(),
            &IID_IOleItemContainer,
            &mut poic as *mut *mut IOleItemContainer as *mut *mut core::ffi::c_void,
        );

        if SUCCEEDED(res) != 0 {
            res = IOleItemContainer_GetObject(
                poic,
                (*This).itemName,
                BINDSPEED_MODERATE,
                pbc,
                &IID_IParseDisplayName,
                &mut ppdn as *mut *mut IParseDisplayName as *mut *mut core::ffi::c_void,
            );

            res = IMoniker_GetDisplayName(iface, pbc, core::ptr::null_mut(), &mut displayName);

            res = IParseDisplayName_ParseDisplayName(ppdn, pbc, displayName, pchEaten, ppmkOut);

            IOleItemContainer_Release(poic);
            IParseDisplayName_Release(ppdn);
        }
    }
    res
}

unsafe fn ItemMonikerImpl_IsSystemMoniker(iface: *mut IMoniker, pwdMksys: *mut DWORD) -> HRESULT {
    TRACE!(b"(%p,%p)\n\0".as_ptr() as PCHAR, iface, pwdMksys);

    if pwdMksys.is_null() {
        return E_POINTER;
    }

    *pwdMksys = MKSYS_ITEMMONIKER;

    S_OK
}
unsafe fn ItemMonikerROTDataImpl_QueryInterface(
    iface: *mut IROTData,
    riid: *const IID,
    ppvObject: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let This = impl_from_IROTData(iface);

    TRACE!(
        b"(%p,%s,%p)\n\0".as_ptr(),
        iface,
        debugstr_guid(riid),
        ppvObject,
    );

    ItemMonikerImpl_QueryInterface(
        core::ptr::addr_of_mut!((*This).IMoniker_iface),
        riid,
        ppvObject,
    )
}

unsafe fn ItemMonikerROTDataImpl_AddRef(iface: *mut IROTData) -> u32 {
    let This = impl_from_IROTData(iface);
    ItemMonikerImpl_AddRef(core::ptr::addr_of_mut!((*This).IMoniker_iface))
}

unsafe fn ItemMonikerROTDataImpl_Release(iface: *mut IROTData) -> u32 {
    let This = impl_from_IROTData(iface);

    ItemMonikerImpl_Release(core::ptr::addr_of_mut!((*This).IMoniker_iface))
}

unsafe fn ItemMonikerROTDataImpl_GetComparisonData(
    iface: *mut IROTData,
    pbData: *mut u8,
    cbMax: u32,
    pcbData: *mut u32,
) -> HRESULT {

    let This = impl_from_IROTData(iface);
    let len = (lstrlenW((*This).itemName) + 1) as usize;
    let mut i: i32;
    let pszItemName: LPWSTR;
    let pszItemDelimiter: LPWSTR;

    *pcbData = (core::mem::size_of::<CLSID>() + core::mem::size_of::<u16>() + len * core::mem::size_of::<u16>()) as u32;
    if cbMax < *pcbData {
        return 0x8007000E;
    }

    core::ptr::copy_nonoverlapping(
        &CLSID_ItemMoniker as *const CLSID as *const u8,
        pbData,
        core::mem::size_of::<CLSID>(),
    );

    pszItemDelimiter = pbData.add(core::mem::size_of::<CLSID>()) as LPWSTR;
    *pszItemDelimiter = *(*This).itemDelimiter;

    pszItemName = pszItemDelimiter.offset(1);
    i = 0;
    while i < len as i32 {
        *pszItemName.offset(i as isize) = towupper(*(*This).itemName.offset(i as isize));
        i += 1;
    }

    0
}

unsafe fn ItemMonikerImpl_Construct(
    This: *mut ItemMonikerImpl,
    lpszDelim: LPCOLESTR,
    lpszItem: LPCOLESTR,
) -> HRESULT {

    let sizeStr1 = lstrlenW(lpszItem as LPCWSTR);
    let sizeStr2: i32;
    static emptystr: [WCHAR; 1] = [0];
    let delim: LPCOLESTR;

    TRACE!(
        b"(%p,%s,%s)\n\0".as_ptr() as LPCSTR,
        This,
        debugstr_w(lpszDelim as LPCWSTR),
        debugstr_w(lpszItem as LPCWSTR),
    );

    (*This).IMoniker_iface.lpVtbl = &VT_ItemMonikerImpl;
    (*This).IROTData_iface.lpVtbl = &VT_ROTDataImpl;
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
    lstrcpyW((*This).itemName as LPWSTR, lpszItem as LPCWSTR);

    if lpszDelim.is_null() {
        FIXME!(b"lpszDelim is NULL. Using empty string which is possibly wrong.\n\0".as_ptr() as LPCSTR);
    }

    delim = if !lpszDelim.is_null() {
        lpszDelim
    } else {
        emptystr.as_ptr()
    };

    sizeStr2 = lstrlenW(delim as LPCWSTR);
    (*This).itemDelimiter = HeapAlloc(
        GetProcessHeap(),
        0,
        (core::mem::size_of::<WCHAR>() * (sizeStr2 as usize + 1)) as SIZE_T,
    ) as LPOLESTR;
    if (*This).itemDelimiter.is_null() {
        HeapFree(GetProcessHeap(), 0, (*This).itemName as LPVOID);
        return E_OUTOFMEMORY;
    }
    lstrcpyW((*This).itemDelimiter as LPWSTR, delim as LPCWSTR);
    S_OK
}

unsafe fn ItemMonikerImpl_Destroy(This: *mut ItemMonikerImpl) -> HRESULT {
    TRACE!(b"(%p)\n\0".as_ptr() as *const i8, This);

    if !(*This).pMarshal.is_null() {
        IUnknown_Release((*This).pMarshal);
    }
    HeapFree(GetProcessHeap(), 0, (*This).itemName as *mut core::ffi::c_void);
    HeapFree(GetProcessHeap(), 0, (*This).itemDelimiter as *mut core::ffi::c_void);
    HeapFree(GetProcessHeap(), 0, This as *mut core::ffi::c_void);

    0
}

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
        core::mem::size_of::<ItemMonikerImpl>(),
    ) as *mut ItemMonikerImpl;

    if newItemMoniker.is_null() {
        return STG_E_INSUFFICIENTMEMORY;
    }

    let hr = ItemMonikerImpl_Construct(newItemMoniker, lpszDelim, lpszItem);

    if FAILED(hr) != 0 {
        HeapFree(GetProcessHeap(), 0, newItemMoniker as *mut core::ffi::c_void);
        return hr;
    }

    return ItemMonikerImpl_QueryInterface(
        core::ptr::addr_of_mut!((*newItemMoniker).IMoniker_iface),
        &IID_IMoniker,
        ppmk as *mut *mut core::ffi::c_void,
    );
}

#[no_mangle]
pub unsafe extern "system" fn ItemMoniker_CreateInstance(
    iface: *mut IClassFactory,
    pUnk: *mut IUnknown,
    riid: *const IID,
    ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {

    static wszEmpty: [WCHAR; 1] = [0];

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

    let newItemMoniker = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<ItemMonikerImpl>(),
    ) as *mut ItemMonikerImpl;
    if newItemMoniker.is_null() {
        return E_OUTOFMEMORY;
    }

    let mut hr = ItemMonikerImpl_Construct(
        newItemMoniker,
        wszEmpty.as_ptr(),
        wszEmpty.as_ptr(),
    );

    if SUCCEEDED(hr) != 0 {
        hr = ItemMonikerImpl_QueryInterface(
            core::ptr::addr_of_mut!((*newItemMoniker).IMoniker_iface),
            riid,
            ppv,
        );
    }
    if FAILED(hr) != 0 {
        HeapFree(GetProcessHeap(), 0, newItemMoniker as LPVOID);
    }

    hr
}
