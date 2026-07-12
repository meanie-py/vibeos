use crate::prelude::*;
use core::ffi::c_void;

extern "C" {

    fn CreateAntiMoniker(ppmk: *mut *mut IMoniker) -> HRESULT;
}
extern "C" {
    fn CoGetMarshalSizeMax( pSize: *mut u32, riid: *const CLSID, pUnk: *mut core::ffi::c_void, dwDestContext: u32, pvDestContext: LPVOID, mshlflags: u32, ) -> HRESULT;
    fn CoMarshalInterface( pStm: *mut IStream, riid: *const CLSID, pUnk: *mut core::ffi::c_void, dwDestContext: u32, pvDestContext: LPVOID, mshlflags: u32, ) -> HRESULT;
    static IID_IMoniker: CLSID;
}

// ---- file-scope preamble (translated) ----
pub const BLOCK_TAB_SIZE: usize = 5;

#[repr(C)]
pub struct CompositeMonikerImpl {
    pub IMoniker_iface: IMoniker,
    pub IROTData_iface: IROTData,
    pub IMarshal_iface: IMarshal,
    pub ref_: LONG,
    pub tabMoniker: *mut *mut IMoniker,
    pub tabSize: ULONG,
    pub tabLastIndex: ULONG,
}

pub type PCompositeMonikerImpl = *mut CompositeMonikerImpl;

#[repr(C)]
pub struct EnumMonikerImpl {
    pub IEnumMoniker_iface: IEnumMoniker,
    pub ref_: LONG,
    pub tabMoniker: *mut *mut IMoniker,
    pub tabSize: ULONG,
    pub currentPos: ULONG,
}

pub type PEnumMonikerImpl = *mut EnumMonikerImpl;

pub static VT_EnumMonikerImpl: IEnumMonikerVtbl = IEnumMonikerVtbl {
    QueryInterface: EnumMonikerImpl_QueryInterface,
    AddRef: EnumMonikerImpl_AddRef,
    Release: EnumMonikerImpl_Release,
    Next: EnumMonikerImpl_Next,
    Skip: EnumMonikerImpl_Skip,
    Reset: EnumMonikerImpl_Reset,
    Clone: EnumMonikerImpl_Clone,
};

pub static VT_CompositeMonikerImpl: IMonikerVtbl = IMonikerVtbl {
    QueryInterface: CompositeMonikerImpl_QueryInterface,
    AddRef: CompositeMonikerImpl_AddRef,
    Release: CompositeMonikerImpl_Release,
    GetClassID: CompositeMonikerImpl_GetClassID,
    IsDirty: CompositeMonikerImpl_IsDirty,
    Load: CompositeMonikerImpl_Load,
    Save: CompositeMonikerImpl_Save,
    GetSizeMax: CompositeMonikerImpl_GetSizeMax,
    BindToObject: CompositeMonikerImpl_BindToObject,
    BindToStorage: CompositeMonikerImpl_BindToStorage,
    Reduce: CompositeMonikerImpl_Reduce,
    ComposeWith: CompositeMonikerImpl_ComposeWith,
    Enum: CompositeMonikerImpl_Enum,
    IsEqual: CompositeMonikerImpl_IsEqual,
    Hash: CompositeMonikerImpl_Hash,
    IsRunning: CompositeMonikerImpl_IsRunning,
    GetTimeOfLastChange: CompositeMonikerImpl_GetTimeOfLastChange,
    Inverse: CompositeMonikerImpl_Inverse,
    CommonPrefixWith: CompositeMonikerImpl_CommonPrefixWith,
    RelativePathTo: CompositeMonikerImpl_RelativePathTo,
    GetDisplayName: CompositeMonikerImpl_GetDisplayName,
    ParseDisplayName: CompositeMonikerImpl_ParseDisplayName,
    IsSystemMoniker: CompositeMonikerImpl_IsSystemMoniker,
};

pub static VT_ROTDataImpl: IROTDataVtbl = IROTDataVtbl {
    QueryInterface: CompositeMonikerROTDataImpl_QueryInterface,
    AddRef: CompositeMonikerROTDataImpl_AddRef,
    Release: CompositeMonikerROTDataImpl_Release,
    GetComparisonData: CompositeMonikerROTDataImpl_GetComparisonData,
};

pub static VT_MarshalImpl: IMarshalVtbl = IMarshalVtbl {
    QueryInterface: CompositeMonikerMarshalImpl_QueryInterface,
    AddRef: CompositeMonikerMarshalImpl_AddRef,
    Release: CompositeMonikerMarshalImpl_Release,
    GetUnmarshalClass: CompositeMonikerMarshalImpl_GetUnmarshalClass,
    GetMarshalSizeMax: CompositeMonikerMarshalImpl_GetMarshalSizeMax,
    MarshalInterface: CompositeMonikerMarshalImpl_MarshalInterface,
    UnmarshalInterface: CompositeMonikerMarshalImpl_UnmarshalInterface,
    ReleaseMarshalData: CompositeMonikerMarshalImpl_ReleaseMarshalData,
    DisconnectObject: CompositeMonikerMarshalImpl_DisconnectObject,
};
unsafe fn impl_from_IMoniker(iface: *mut IMoniker) -> *mut CompositeMonikerImpl {
    (iface as *mut u8).offset(-(core::mem::offset_of!(CompositeMonikerImpl, IMoniker_iface) as isize)) as *mut CompositeMonikerImpl
}

unsafe fn impl_from_IROTData(iface: *mut IROTData) -> *mut CompositeMonikerImpl {
    (iface as *mut u8).offset(-(core::mem::offset_of!(CompositeMonikerImpl, IROTData_iface) as isize)) as *mut CompositeMonikerImpl
}

unsafe fn impl_from_IMarshal(iface: *mut IMarshal) -> *mut CompositeMonikerImpl {
    (iface as *mut u8).offset(-(core::mem::offset_of!(CompositeMonikerImpl, IMarshal_iface) as isize)) as *mut CompositeMonikerImpl
}

unsafe fn impl_from_IEnumMoniker(iface: *mut IEnumMoniker) -> *mut EnumMonikerImpl {
    (iface as *mut u8).offset(-(core::mem::offset_of!(EnumMonikerImpl, IEnumMoniker_iface) as isize)) as *mut EnumMonikerImpl
}

unsafe fn CompositeMonikerImpl_QueryInterface(
    iface: *mut IMoniker,
    riid: *const CLSID,
    ppvObject: *mut LPVOID
) -> HRESULT {
    let This = impl_from_IMoniker(iface);

    TRACE!(b"(%p,%s,%p)\n\0".as_ptr(), This, debugstr_guid(riid), ppvObject);

    if ppvObject.is_null() {
        return E_INVALIDARG;
    }

    *ppvObject = core::ptr::null_mut();

    if IsEqualIID(core::ptr::addr_of!(IID_IUnknown), riid) != 0 ||
       IsEqualIID(core::ptr::addr_of!(IID_IPersist), riid) != 0 ||
       IsEqualIID(core::ptr::addr_of!(IID_IPersistStream), riid) != 0 ||
       IsEqualIID(core::ptr::addr_of!(IID_IMoniker), riid) != 0
    {
        *ppvObject = iface as LPVOID;
    } else if IsEqualIID(core::ptr::addr_of!(IID_IROTData), riid) != 0 {
        *ppvObject = core::ptr::addr_of_mut!((*This).IROTData_iface) as LPVOID;
    } else if IsEqualIID(core::ptr::addr_of!(IID_IMarshal), riid) != 0 {
        *ppvObject = core::ptr::addr_of_mut!((*This).IMarshal_iface) as LPVOID;
    }

    if (*ppvObject).is_null() {
        return E_NOINTERFACE;
    }

    IMoniker_AddRef(iface);

    S_OK
}

unsafe fn CompositeMonikerImpl_AddRef(iface: *mut IMoniker) -> u32 {
    let This = impl_from_IMoniker(iface);

    TRACE!(b"(%p)\n\0".as_ptr(), This);

    InterlockedIncrement(&mut (*This).ref_) as u32
}

unsafe fn CompositeMonikerImpl_ReleaseMonikersInTable(This: *mut CompositeMonikerImpl) {
    let mut i: ULONG;

    i = 0;
    while i < (*This).tabLastIndex {
        IMoniker_Release((*(*This).tabMoniker.offset(i as isize)));
        i += 1;
    }

    (*This).tabLastIndex = 0;
}

unsafe fn CompositeMonikerImpl_Release(iface: *mut IMoniker) -> ULONG {
    let This = impl_from_IMoniker(iface);
    let ref_: ULONG;

    TRACE!(b"(%p)\n\0".as_ptr() as *const i8, This);

    ref_ = InterlockedDecrement(&mut (*This).ref_) as ULONG;

    if ref_ == 0 {
        CompositeMonikerImpl_ReleaseMonikersInTable(This);

        HeapFree(GetProcessHeap(), 0, (*This).tabMoniker as LPVOID);
        HeapFree(GetProcessHeap(), 0, This as LPVOID);
    }
    ref_
}

unsafe fn CompositeMonikerImpl_GetClassID(
    iface: *mut IMoniker,
    pClassID: *mut CLSID,
) -> HRESULT {
    TRACE!(b"(%p,%p)\n\0".as_ptr() as *const i8, iface, pClassID);

    if pClassID.is_null() {
        return E_POINTER;
    }

    *pClassID = CLSID_CompositeMoniker;

    S_OK
}
unsafe fn CompositeMonikerImpl_IsDirty(iface: *mut IMoniker) -> HRESULT {
    TRACE!(b"(%p)\n\0".as_ptr() as PCHAR, iface);

    S_FALSE
}

unsafe fn CompositeMonikerImpl_Load(iface: *mut IMoniker, pStm: *mut IStream) -> HRESULT {
    let This = impl_from_IMoniker(iface);
    let mut res: HRESULT;
    let mut moniker_count: u32 = 0;
    let mut i: u32;

    TRACE!(b"(%p,%p)\n\0".as_ptr() as *const i8, iface, pStm);

    res = IStream_Read(pStm, &mut moniker_count as *mut u32 as LPVOID, core::mem::size_of::<u32>() as u32, core::ptr::null_mut());
    if res != 0 {
        ERR!(b"couldn't reading moniker count from stream\n\0".as_ptr() as *const i8);
        return 0x80004005u32 as HRESULT;
    }

    CompositeMonikerImpl_ReleaseMonikersInTable(This);

    i = 0;
    while i < moniker_count {
        res = OleLoadFromStream(
            pStm,
            &IID_IMoniker,
            &mut (*(*This).tabMoniker.offset((*This).tabLastIndex as isize)) as *mut *mut IMoniker as *mut LPVOID
        );
        if FAILED(res) != 0 {
            ERR!(b"couldn't load moniker from stream, res = 0x%08x\n\0".as_ptr() as *const i8, res);
            break;
        }

        (*This).tabLastIndex = (*This).tabLastIndex.wrapping_add(1);
        if (*This).tabLastIndex == (*This).tabSize {
            (*This).tabSize = (*This).tabSize.wrapping_add(5);
            (*This).tabMoniker = HeapReAlloc(
                GetProcessHeap(),
                0,
                (*This).tabMoniker as LPVOID,
                ((*This).tabSize as usize * core::mem::size_of::<*mut IMoniker>()) as usize
            ) as *mut *mut IMoniker;

            if (*This).tabMoniker.is_null() {
                return 0x8007000Eu32 as HRESULT;
            }
        }

        i = i.wrapping_add(1);
    }

    res
}

unsafe fn CompositeMonikerImpl_Save(
    iface: *mut IMoniker,
    pStm: *mut IStream,
    fClearDirty: i32,
) -> HRESULT {
    let This = impl_from_IMoniker(iface);
    let mut res: HRESULT;
    let mut enumMk: *mut IEnumMoniker = core::ptr::null_mut();
    let mut pmk: *mut IMoniker = core::ptr::null_mut();
    let moniker_count: u32 = (*This).tabLastIndex;

    TRACE!(
        b"(%p,%p,%d)\n\0".as_ptr() as *const i8,
        iface as LPVOID,
        pStm as LPVOID,
        fClearDirty,
    );

    res = IStream_Write(
        pStm,
        &moniker_count as *const u32 as *const core::ffi::c_void,
        core::mem::size_of::<u32>() as u32,
        core::ptr::null_mut(),
    );
    if FAILED(res) != 0 {
        return res;
    }

    IMoniker_Enum(iface, 1, &mut enumMk);

    loop {
        if IEnumMoniker_Next(enumMk, 1, &mut pmk, core::ptr::null_mut()) != 0 {
            break;
        }

        res = OleSaveToStream(pmk as *mut core::ffi::c_void, pStm);

        IMoniker_Release(pmk);

        if FAILED(res) != 0 {
            IEnumMoniker_Release(enumMk);
            return res;
        }
    }

    IEnumMoniker_Release(enumMk);

    0
}

unsafe fn CompositeMonikerImpl_GetSizeMax(
    iface: *mut IMoniker,
    pcbSize: *mut ULARGE_INTEGER,
) -> HRESULT {
    let mut enumMk: *mut IEnumMoniker = core::ptr::null_mut();
    let mut pmk: *mut IMoniker = core::ptr::null_mut();
    let mut ptmpSize: ULARGE_INTEGER = core::mem::zeroed();

    TRACE!(b"(%p,%p)\n\0".as_ptr() as _, iface, pcbSize);

    if pcbSize.is_null() {
        return E_POINTER;
    }

    (*pcbSize).QuadPart = core::mem::size_of::<DWORD>() as u64;

    IMoniker_Enum(iface, 1u8, &mut enumMk);

    while IEnumMoniker_Next(enumMk, 1, &mut pmk, core::ptr::null_mut()) == S_OK {
        IMoniker_GetSizeMax(pmk, &mut ptmpSize);

        IMoniker_Release(pmk);

        (*pcbSize).QuadPart = ptmpSize.QuadPart + core::mem::size_of::<CLSID>() as u64;
    }

    IEnumMoniker_Release(enumMk);

    S_OK
}

unsafe fn CompositeMonikerImpl_BindToObject(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    riid: *const CLSID,
    ppvResult: *mut LPVOID,
) -> HRESULT {

    TRACE!(
        b"(%p,%p,%p,%s,%p)\n\0".as_ptr() as *const i8,
        iface,
        pbc,
        pmkToLeft,
        debugstr_guid(riid),
        ppvResult,
    );

    if ppvResult.is_null() {
        return 0x80004003u32 as HRESULT;
    }

    *ppvResult = core::ptr::null_mut();

    let mut res: HRESULT;

    if pmkToLeft.is_null() {
        let mut prot: *mut IRunningObjectTable = core::ptr::null_mut();
        res = IBindCtx_GetRunningObjectTable(pbc, &mut prot);

        if SUCCEEDED(res) != 0 {
            res = IRunningObjectTable_GetObject(prot, iface, ppvResult as *mut *mut core::ffi::c_void);

            if res == 0 {
                return res;
            }
        }
    } else {
        let mut enumMoniker: *mut IEnumMoniker = core::ptr::null_mut();
        let mut rightMostMk: *mut IMoniker = core::ptr::null_mut();
        let mut antiMk: *mut IMoniker = core::ptr::null_mut();
        let mut tempMk: *mut IMoniker = core::ptr::null_mut();

        IMoniker_Enum(iface, 0, &mut enumMoniker);
        IEnumMoniker_Next(enumMoniker, 1, &mut rightMostMk, core::ptr::null_mut());
        IEnumMoniker_Release(enumMoniker);

        res = CreateAntiMoniker(&mut antiMk);
        res = IMoniker_ComposeWith(iface, antiMk, 0, &mut tempMk);
        IMoniker_Release(antiMk);

        res = IMoniker_BindToObject(rightMostMk, pbc, tempMk, riid, ppvResult);

        IMoniker_Release(tempMk);
        IMoniker_Release(rightMostMk);
    }

    res
}

unsafe fn CompositeMonikerImpl_BindToStorage(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    riid: *const CLSID,
    ppvResult: *mut LPVOID,
) -> HRESULT {
    let mut res: HRESULT;
    let mut tempMk: *mut IMoniker;
    let mut antiMk: *mut IMoniker;
    let mut rightMostMk: *mut IMoniker;
    let mut leftMk: *mut IMoniker;
    let mut enumMoniker: *mut IEnumMoniker;

    TRACE!(b"(%p,%p,%p,%s,%p)\n\0".as_ptr(), iface, pbc, pmkToLeft, debugstr_guid(riid), ppvResult);

    *ppvResult = core::ptr::null_mut();

    if !pmkToLeft.is_null() {
        res = IMoniker_ComposeWith(pmkToLeft, iface, 0, &mut leftMk);
        if FAILED(res) != 0 {
            return res;
        }
    } else {
        leftMk = iface;
    }

    IMoniker_Enum(iface, 0, &mut enumMoniker);
    IEnumMoniker_Next(enumMoniker, 1, &mut rightMostMk, core::ptr::null_mut());
    IEnumMoniker_Release(enumMoniker);

    res = CreateAntiMoniker(&mut antiMk);
    if FAILED(res) != 0 {
        return res;
    }
    res = IMoniker_ComposeWith(leftMk, antiMk, 0, &mut tempMk);
    if FAILED(res) != 0 {
        return res;
    }
    IMoniker_Release(antiMk);

    res = IMoniker_BindToStorage(rightMostMk, pbc, tempMk, riid, ppvResult);

    IMoniker_Release(tempMk);

    IMoniker_Release(rightMostMk);

    if !pmkToLeft.is_null() {
        IMoniker_Release(leftMk);
    }

    res
}
unsafe fn CompositeMonikerImpl_Reduce(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    dwReduceHowFar: u32,
    ppmkToLeft: *mut *mut IMoniker,
    ppmkReduced: *mut *mut IMoniker,
) -> HRESULT {
    let mut res: HRESULT;
    let mut tempMk: *mut IMoniker;
    let mut antiMk: *mut IMoniker;
    let mut rightMostMk: *mut IMoniker;
    let mut leftReducedComposedMk: *mut IMoniker;
    let mut rightMostReducedMk: *mut IMoniker;
    let mut enumMoniker: *mut IEnumMoniker;

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

    if ppmkToLeft.is_null() {
        IMoniker_Enum(iface, 0, core::ptr::addr_of_mut!(enumMoniker));
        IEnumMoniker_Next(enumMoniker, 1, core::ptr::addr_of_mut!(rightMostMk), core::ptr::null_mut());
        IEnumMoniker_Release(enumMoniker);

        CreateAntiMoniker(core::ptr::addr_of_mut!(antiMk));
        IMoniker_ComposeWith(iface, antiMk, 0, core::ptr::addr_of_mut!(tempMk));
        IMoniker_Release(antiMk);

        res = IMoniker_Reduce(rightMostMk, pbc, dwReduceHowFar, core::ptr::addr_of_mut!(tempMk), ppmkReduced);
        IMoniker_Release(tempMk);
        IMoniker_Release(rightMostMk);

        return res;
    } else if (*ppmkToLeft).is_null() {
        return IMoniker_Reduce(iface, pbc, dwReduceHowFar, core::ptr::null_mut(), ppmkReduced);
    } else {
        IMoniker_Enum(iface, 0, core::ptr::addr_of_mut!(enumMoniker));
        IEnumMoniker_Next(enumMoniker, 1, core::ptr::addr_of_mut!(rightMostMk), core::ptr::null_mut());
        IEnumMoniker_Release(enumMoniker);

        CreateAntiMoniker(core::ptr::addr_of_mut!(antiMk));
        IMoniker_ComposeWith(iface, antiMk, 0, core::ptr::addr_of_mut!(tempMk));
        IMoniker_Release(antiMk);

        if IMoniker_Reduce(rightMostMk, pbc, dwReduceHowFar, core::ptr::null_mut(), core::ptr::addr_of_mut!(rightMostReducedMk)) != 0
            && IMoniker_Reduce(rightMostMk, pbc, dwReduceHowFar, core::ptr::addr_of_mut!(tempMk), core::ptr::addr_of_mut!(leftReducedComposedMk)) != 0
        {
            IMoniker_Release(tempMk);
            IMoniker_Release(rightMostMk);

            return CreateGenericComposite(leftReducedComposedMk, rightMostReducedMk, ppmkReduced);
        } else {
            IMoniker_Release(tempMk);
            IMoniker_Release(rightMostMk);

            IMoniker_AddRef(iface);

            *ppmkReduced = iface;

            return MK_S_REDUCED_TO_SELF;
        }
    }
}

unsafe fn CompositeMonikerImpl_ComposeWith(
    iface: *mut IMoniker,
    pmkRight: *mut IMoniker,
    fOnlyIfNotGeneric: BOOL,
    ppmkComposite: *mut *mut IMoniker,
) -> HRESULT {
    if ppmkComposite.is_null() || pmkRight.is_null() {
        return E_POINTER;
    }

    *ppmkComposite = core::ptr::null_mut();

    if fOnlyIfNotGeneric != 0 {
        return MK_E_NEEDGENERIC;
    }

    CreateGenericComposite(iface, pmkRight, ppmkComposite)
}

unsafe fn CompositeMonikerImpl_Enum(
    iface: *mut IMoniker,
    fForward: BOOL,
    ppenumMoniker: *mut *mut IEnumMoniker,
) -> HRESULT {
    let This = impl_from_IMoniker(iface);

    TRACE!(
        b"(%p,%d,%p)\n\0".as_ptr(),
        iface,
        fForward,
        ppenumMoniker,
    );

    if ppenumMoniker == core::ptr::null_mut() {
        return E_POINTER;
    }

    EnumMonikerImpl_CreateEnumMoniker(
        (*This).tabMoniker,
        (*This).tabLastIndex,
        0,
        fForward,
        ppenumMoniker,
    )
}
unsafe fn CompositeMonikerImpl_IsEqual(
    iface: *mut IMoniker,
    pmkOtherMoniker: *mut IMoniker,
) -> HRESULT {
    let mut enumMoniker1: *mut IEnumMoniker = core::ptr::null_mut();
    let mut enumMoniker2: *mut IEnumMoniker = core::ptr::null_mut();
    let mut tempMk1: *mut IMoniker;
    let mut tempMk2: *mut IMoniker;
    let mut res1: HRESULT;
    let mut res2: HRESULT;
    let mut res: HRESULT;
    let mut done: u8;

    TRACE!(b"(%p,%p)\n\0".as_ptr() as *const i8, iface, pmkOtherMoniker);

    if pmkOtherMoniker.is_null() {
        return 0x00000001; // S_FALSE
    }

    IMoniker_Enum(pmkOtherMoniker, 1, &mut enumMoniker1);

    if enumMoniker1.is_null() {
        return 0x00000001; // S_FALSE
    }

    IMoniker_Enum(iface, 1, &mut enumMoniker2);

    loop {
        res1 = IEnumMoniker_Next(enumMoniker1, 1, &mut tempMk1, core::ptr::null_mut());
        res2 = IEnumMoniker_Next(enumMoniker2, 1, &mut tempMk2, core::ptr::null_mut());

        if (res1 == 0x00000000) && (res2 == 0x00000000) {
            res = IMoniker_IsEqual(tempMk1, tempMk2);
            done = if res == 0x00000001 { 1 } else { 0 };
        } else {
            res = if (res1 == 0x00000001) && (res2 == 0x00000001) { 0x00000000 } else { 0x00000001 };
            done = 1;
        }

        if res1 == 0x00000000 {
            IMoniker_Release(tempMk1);
        }

        if res2 == 0x00000000 {
            IMoniker_Release(tempMk2);
        }

        if done != 0 {
            break;
        }
    }

    IEnumMoniker_Release(enumMoniker1);
    IEnumMoniker_Release(enumMoniker2);

    res
}
unsafe fn CompositeMonikerImpl_Hash(
    iface: *mut IMoniker,
    pdwHash: *mut DWORD,
) -> HRESULT {

    let mut enumMoniker: *mut IEnumMoniker = core::ptr::null_mut();
    let mut tempMk: *mut IMoniker;
    let mut res: HRESULT;
    let mut tempHash: DWORD;

    TRACE!(b"(%p,%p)\n\0".as_ptr(), iface, pdwHash);

    if pdwHash.is_null() {
        return E_POINTER;
    }

    res = IMoniker_Enum(iface, TRUE, &mut enumMoniker);
    if FAILED(res) != 0 {
        return res;
    }

    *pdwHash = 0;

    loop {
        tempMk = core::ptr::null_mut();
        if IEnumMoniker_Next(enumMoniker, 1, &mut tempMk, core::ptr::null_mut()) != S_OK {
            break;
        }
        res = IMoniker_Hash(tempMk, &mut tempHash);
        if FAILED(res) != 0 {
            IMoniker_Release(tempMk);
            break;
        }
        *pdwHash = *pdwHash ^ tempHash;

        IMoniker_Release(tempMk);
    }

    IEnumMoniker_Release(enumMoniker);

    res
}
unsafe fn CompositeMonikerImpl_IsRunning(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    pmkNewlyRunning: *mut IMoniker,
) -> HRESULT {

    let mut rot: *mut IRunningObjectTable = core::ptr::null_mut();
    let mut res: HRESULT;
    let mut tempMk: *mut IMoniker;
    let mut antiMk: *mut IMoniker;
    let mut rightMostMk: *mut IMoniker;
    let mut enumMoniker: *mut IEnumMoniker;

    TRACE!(
        b"(%p,%p,%p,%p)\n\0".as_ptr(),
        iface,
        pbc,
        pmkToLeft,
        pmkNewlyRunning,
    );

    if !pmkToLeft.is_null() {
        tempMk = core::ptr::null_mut();
        CreateGenericComposite(pmkToLeft, iface, &mut tempMk);

        res = IMoniker_IsRunning(tempMk, pbc, core::ptr::null_mut(), pmkNewlyRunning);

        IMoniker_Release(tempMk);

        return res;
    } else {
        if !pmkNewlyRunning.is_null() {
            if IMoniker_IsEqual(iface, pmkNewlyRunning) == 0 {
                return 0;
            } else {
                return 1;
            }
        } else {
            if pbc.is_null() {
                return 0x80070057u32 as HRESULT;
            }

            res = IBindCtx_GetRunningObjectTable(pbc, &mut rot);

            if FAILED(res) != 0 {
                return res;
            }

            res = IRunningObjectTable_IsRunning(rot, iface);
            IRunningObjectTable_Release(rot);

            if res == 0 {
                return 0;
            } else {
                enumMoniker = core::ptr::null_mut();
                IMoniker_Enum(iface, 0, &mut enumMoniker);
                rightMostMk = core::ptr::null_mut();
                IEnumMoniker_Next(enumMoniker, 1, &mut rightMostMk, core::ptr::null_mut());
                IEnumMoniker_Release(enumMoniker);

                antiMk = core::ptr::null_mut();
                res = CreateAntiMoniker(&mut antiMk);
                tempMk = core::ptr::null_mut();
                res = IMoniker_ComposeWith(iface, antiMk, 0, &mut tempMk);
                IMoniker_Release(antiMk);

                res = IMoniker_IsRunning(rightMostMk, pbc, tempMk, pmkNewlyRunning);

                IMoniker_Release(tempMk);
                IMoniker_Release(rightMostMk);

                return res;
            }
        }
    }
}

unsafe fn CompositeMonikerImpl_GetTimeOfLastChange(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    pCompositeTime: *mut FILETIME,
) -> HRESULT {
    let mut res: HRESULT;
    let mut tempMk: *mut IMoniker;
    let mut antiMk: *mut IMoniker;
    let mut rightMostMk: *mut IMoniker;
    let mut leftMk: *mut IMoniker;
    let mut enumMoniker: *mut IEnumMoniker;

    TRACE!(b"(%p,%p,%p,%p)\n\0".as_ptr() as *const i8, iface, pbc, pmkToLeft, pCompositeTime);

    if pCompositeTime.is_null() {
        return E_INVALIDARG;
    }

    if !pmkToLeft.is_null() {
        let mut rot: *mut IRunningObjectTable = core::ptr::null_mut();

        res = IMoniker_ComposeWith(pmkToLeft, iface, 0, &mut leftMk);
        if FAILED(res) != 0 {
            return res;
        }

        res = IBindCtx_GetRunningObjectTable(pbc, &mut rot);
        if FAILED(res) != 0 {
            IMoniker_Release(leftMk);
            return res;
        }

        if IRunningObjectTable_GetTimeOfLastChange(rot, leftMk, pCompositeTime) == S_OK {
            IMoniker_Release(leftMk);
            return res;
        }
    } else {
        leftMk = iface;
    }

    IMoniker_Enum(iface, 0, &mut enumMoniker);
    IEnumMoniker_Next(enumMoniker, 1, &mut rightMostMk, core::ptr::null_mut());
    IEnumMoniker_Release(enumMoniker);

    res = CreateAntiMoniker(&mut antiMk);
    res = IMoniker_ComposeWith(leftMk, antiMk, 0, &mut tempMk);
    IMoniker_Release(antiMk);

    res = IMoniker_GetTimeOfLastChange(rightMostMk, pbc, tempMk, pCompositeTime);

    IMoniker_Release(tempMk);
    IMoniker_Release(rightMostMk);

    if !pmkToLeft.is_null() {
        IMoniker_Release(leftMk);
    }

    res
}

unsafe fn CompositeMonikerImpl_Inverse(iface: *mut IMoniker, ppmk: *mut *mut IMoniker) -> HRESULT {
    let mut res: HRESULT;
    let mut tempMk: *mut IMoniker;
    let mut antiMk: *mut IMoniker;
    let mut rightMostMk: *mut IMoniker;
    let mut tempInvMk: *mut IMoniker;
    let mut rightMostInvMk: *mut IMoniker;
    let mut enumMoniker: *mut IEnumMoniker;

    TRACE!(b"(%p,%p)\n\0".as_ptr(), iface, ppmk);

    if ppmk == core::ptr::null_mut() {
        return 0x80004003u32 as HRESULT;
    }

    *ppmk = core::ptr::null_mut();

    res = CreateAntiMoniker(core::ptr::addr_of_mut!(antiMk));
    if FAILED(res) != 0 {
        return res;
    }

    res = IMoniker_ComposeWith(iface, antiMk, 0, core::ptr::addr_of_mut!(tempMk));
    IMoniker_Release(antiMk);
    if FAILED(res) != 0 {
        return res;
    }

    if tempMk == core::ptr::null_mut() {
        return IMoniker_Inverse(iface, ppmk);
    } else {
        IMoniker_Enum(iface, 0, core::ptr::addr_of_mut!(enumMoniker));
        IEnumMoniker_Next(enumMoniker, 1, core::ptr::addr_of_mut!(rightMostMk), core::ptr::null_mut());
        IEnumMoniker_Release(enumMoniker);

        IMoniker_Inverse(rightMostMk, core::ptr::addr_of_mut!(rightMostInvMk));
        CompositeMonikerImpl_Inverse(tempMk, core::ptr::addr_of_mut!(tempInvMk));

        res = CreateGenericComposite(rightMostInvMk, tempInvMk, ppmk);

        IMoniker_Release(tempMk);
        IMoniker_Release(rightMostMk);
        IMoniker_Release(tempInvMk);
        IMoniker_Release(rightMostInvMk);

        return res;
    }
}
unsafe fn CompositeMonikerImpl_CommonPrefixWith(
    iface: *mut IMoniker,
    pmkOther: *mut IMoniker,
    ppmkPrefix: *mut *mut IMoniker,
) -> HRESULT {
    let mut mkSys: u32 = 0;
    let mut res1: HRESULT;
    let mut res2: HRESULT;
    let mut tempMk1: *mut IMoniker = core::ptr::null_mut();
    let mut tempMk2: *mut IMoniker = core::ptr::null_mut();
    let mut mostLeftMk1: *mut IMoniker = core::ptr::null_mut();
    let mut mostLeftMk2: *mut IMoniker = core::ptr::null_mut();
    let mut enumMoniker1: *mut IEnumMoniker = core::ptr::null_mut();
    let mut enumMoniker2: *mut IEnumMoniker = core::ptr::null_mut();
    let mut i: u32;
    let mut nbCommonMk: u32 = 0;

    if ppmkPrefix.is_null() {
        return 0x80004003u32 as HRESULT;
    }

    *ppmkPrefix = core::ptr::null_mut();

    if pmkOther.is_null() {
        return 0x800401E7u32 as HRESULT;
    }

    IMoniker_IsSystemMoniker(pmkOther, &mut mkSys);

    if mkSys == 2 {
        IMoniker_Enum(iface, 1, &mut enumMoniker1);
        IMoniker_Enum(pmkOther, 1, &mut enumMoniker2);

        loop {
            res1 = IEnumMoniker_Next(enumMoniker1, 1, &mut mostLeftMk1, core::ptr::null_mut());
            res2 = IEnumMoniker_Next(enumMoniker2, 1, &mut mostLeftMk2, core::ptr::null_mut());

            if (res1 == 1) && (res2 == 1) {
                *ppmkPrefix = iface;
                IMoniker_AddRef(iface);
                return 0x000401E2u32 as HRESULT;
            } else if (res1 == 0) && (res2 == 0) {
                if IMoniker_IsEqual(mostLeftMk1, mostLeftMk2) == 0 {
                    nbCommonMk += 1;
                } else {
                    break;
                }
            } else if res1 == 0 {
                *ppmkPrefix = pmkOther;
                return 0x000401E4u32 as HRESULT;
            } else {
                *ppmkPrefix = iface;
                return 0x000401E3u32 as HRESULT;
            }
        }

        IEnumMoniker_Release(enumMoniker1);
        IEnumMoniker_Release(enumMoniker2);

        if nbCommonMk == 0 {
            return 0x800401E7u32 as HRESULT;
        }

        IEnumMoniker_Reset(enumMoniker1);

        IEnumMoniker_Next(enumMoniker1, 1, &mut tempMk1, core::ptr::null_mut());

        if nbCommonMk > 1 {
            IEnumMoniker_Next(enumMoniker1, 1, &mut tempMk2, core::ptr::null_mut());
            CreateGenericComposite(tempMk1, tempMk2, ppmkPrefix);
            IMoniker_Release(tempMk1);
            IMoniker_Release(tempMk2);

            i = 0;
            while i < nbCommonMk {
                IEnumMoniker_Next(enumMoniker1, 1, &mut tempMk1, core::ptr::null_mut());

                CreateGenericComposite(*ppmkPrefix, tempMk1, &mut tempMk2);

                IMoniker_Release(*ppmkPrefix);

                IMoniker_Release(tempMk1);

                *ppmkPrefix = tempMk2;
                i += 1;
            }
            return 0;
        } else {
            *ppmkPrefix = tempMk1;

            return 0;
        }
    } else {
        IMoniker_Enum(iface, 1, &mut enumMoniker1);

        IEnumMoniker_Next(enumMoniker1, 1, &mut mostLeftMk1, core::ptr::null_mut());

        if IMoniker_IsEqual(pmkOther, mostLeftMk1) == 0 {
            *ppmkPrefix = pmkOther;

            return 0x000401E4u32 as HRESULT;
        } else {
            return 0x800401E7u32 as HRESULT;
        }
    }
}

unsafe fn GetAfterCommonPrefix(pGenMk: *mut IMoniker, commonMk: *mut IMoniker, restMk: *mut *mut IMoniker) {
    let mut tempMk: *mut IMoniker;
    let mut tempMk1: *mut IMoniker;
    let mut tempMk2: *mut IMoniker;
    let mut enumMoniker1: *mut IEnumMoniker = core::ptr::null_mut();
    let mut enumMoniker2: *mut IEnumMoniker;
    let mut enumMoniker3: *mut IEnumMoniker;
    let mut nbRestMk: u32 = 0;
    let mut mkSys: u32;
    let mut res1: HRESULT;
    let mut res2: HRESULT;

    *restMk = core::ptr::null_mut();

    IMoniker_Enum(pGenMk, 1u8, &mut enumMoniker1);

    IMoniker_IsSystemMoniker(commonMk, &mut mkSys);

    if mkSys == 2 {
        IMoniker_Enum(commonMk, 1u8, &mut enumMoniker2);
        loop {
            res1 = IEnumMoniker_Next(enumMoniker1, 1, &mut tempMk1, core::ptr::null_mut());
            res2 = IEnumMoniker_Next(enumMoniker2, 1, &mut tempMk2, core::ptr::null_mut());

            if res1 == 1 || res2 == 1 {
                if res1 == 0 {
                    nbRestMk += 1;
                }

                IMoniker_Release(tempMk1);
                IMoniker_Release(tempMk2);

                break;
            }
            IMoniker_Release(tempMk1);
            IMoniker_Release(tempMk2);
        }
    } else {
        IEnumMoniker_Next(enumMoniker1, 1, &mut tempMk1, core::ptr::null_mut());
        IMoniker_Release(tempMk1);
    }

    IEnumMoniker_Clone(enumMoniker1, &mut enumMoniker3);

    loop {
        if IEnumMoniker_Next(enumMoniker3, 1, &mut tempMk, core::ptr::null_mut()) != 0 {
            break;
        }
        IMoniker_Release(tempMk);
        nbRestMk += 1;
    }

    if nbRestMk == 0 {
        return;
    }

    IEnumMoniker_Next(enumMoniker1, 1, &mut tempMk1, core::ptr::null_mut());

    if nbRestMk == 1 {
        *restMk = tempMk1;
        return;
    } else {
        IEnumMoniker_Next(enumMoniker1, 1, &mut tempMk2, core::ptr::null_mut());

        CreateGenericComposite(tempMk1, tempMk2, restMk);

        IMoniker_Release(tempMk1);

        IMoniker_Release(tempMk2);

        while IEnumMoniker_Next(enumMoniker1, 1, &mut tempMk1, core::ptr::null_mut()) == 0 {
            CreateGenericComposite(*restMk, tempMk1, &mut tempMk2);

            IMoniker_Release(tempMk1);

            IMoniker_Release(*restMk);

            *restMk = tempMk2;
        }
    }
}
unsafe fn CompositeMonikerImpl_RelativePathTo(
    iface: *mut IMoniker,
    pmkOther: *mut IMoniker,
    ppmkRelPath: *mut *mut IMoniker,
) -> HRESULT {

    let mut restOtherMk: *mut IMoniker = core::ptr::null_mut();
    let mut restThisMk: *mut IMoniker = core::ptr::null_mut();
    let mut invRestThisMk: *mut IMoniker = core::ptr::null_mut();
    let mut commonMk: *mut IMoniker = core::ptr::null_mut();

    if ppmkRelPath.is_null() {
        return E_POINTER;
    }

    *ppmkRelPath = core::ptr::null_mut();

    let res = IMoniker_CommonPrefixWith(iface, pmkOther, &mut commonMk);

    if res == MK_E_NOPREFIX || res == MK_S_US {
        *ppmkRelPath = pmkOther;
        IMoniker_AddRef(pmkOther);
        return MK_S_HIM;
    }

    GetAfterCommonPrefix(iface, commonMk, &mut restThisMk);
    GetAfterCommonPrefix(pmkOther, commonMk, &mut restOtherMk);

    if res == MK_S_HIM {
        IMoniker_Inverse(restThisMk, ppmkRelPath);
        IMoniker_Release(restThisMk);
    } else if res == MK_S_ME {
        *ppmkRelPath = restOtherMk;
        IMoniker_AddRef(restOtherMk);
    } else if res == S_OK {
        IMoniker_Inverse(restThisMk, &mut invRestThisMk);
        IMoniker_Release(restThisMk);
        CreateGenericComposite(invRestThisMk, restOtherMk, ppmkRelPath);
        IMoniker_Release(invRestThisMk);
        IMoniker_Release(restOtherMk);
    }

    S_OK
}
unsafe fn CompositeMonikerImpl_GetDisplayName(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    ppszDisplayName: *mut LPOLESTR,
) -> HRESULT {
    let mut lengthStr: ULONG = 1;
    let mut enumMoniker: *mut IEnumMoniker = core::ptr::null_mut();
    let mut tempMk: *mut IMoniker = core::ptr::null_mut();
    let mut tempStr: LPOLESTR = core::ptr::null_mut();

    TRACE!(
        b"(%p,%p,%p,%p)\n\0".as_ptr() as *const i8,
        iface,
        pbc,
        pmkToLeft,
        ppszDisplayName,
    );

    if ppszDisplayName.is_null() {
        return E_POINTER;
    }

    *ppszDisplayName = CoTaskMemAlloc(core::mem::size_of::<WCHAR>()) as LPOLESTR;

    if (*ppszDisplayName).is_null() {
        return E_OUTOFMEMORY;
    }

    *(*ppszDisplayName).offset(0) = 0;

    IMoniker_Enum(iface, 1u8, &mut enumMoniker);

    while IEnumMoniker_Next(enumMoniker, 1, &mut tempMk, core::ptr::null_mut()) == S_OK {
        IMoniker_GetDisplayName(tempMk, pbc, core::ptr::null_mut(), &mut tempStr);

        lengthStr += lstrlenW(tempStr) as ULONG;

        *ppszDisplayName = CoTaskMemRealloc(
            *ppszDisplayName as LPVOID,
            (lengthStr as usize) * core::mem::size_of::<WCHAR>(),
        ) as LPOLESTR;

        if (*ppszDisplayName).is_null() {
            return E_OUTOFMEMORY;
        }

        lstrcatW(*ppszDisplayName, tempStr);

        CoTaskMemFree(tempStr as LPVOID);
        IMoniker_Release(tempMk);
    }

    IEnumMoniker_Release(enumMoniker);

    S_OK
}

unsafe fn CompositeMonikerImpl_ParseDisplayName(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    pszDisplayName: LPOLESTR,
    pchEaten: *mut ULONG,
    ppmkOut: *mut *mut IMoniker,
) -> HRESULT {
    let mut enumMoniker: *mut IEnumMoniker = core::ptr::null_mut();
    let mut tempMk: *mut IMoniker = core::ptr::null_mut();
    let mut rightMostMk: *mut IMoniker = core::ptr::null_mut();
    let mut antiMk: *mut IMoniker = core::ptr::null_mut();

    IMoniker_Enum(iface, 0, &mut enumMoniker);
    IEnumMoniker_Next(enumMoniker, 1, &mut rightMostMk, core::ptr::null_mut());
    IEnumMoniker_Release(enumMoniker);

    CreateAntiMoniker(&mut antiMk);
    IMoniker_ComposeWith(iface, antiMk, 0, &mut tempMk);
    IMoniker_Release(antiMk);

    IMoniker_ParseDisplayName(rightMostMk, pbc, tempMk, pszDisplayName, pchEaten, ppmkOut)
}

unsafe fn CompositeMonikerImpl_IsSystemMoniker(
    iface: *mut IMoniker,
    pwdMksys: *mut u32,
) -> HRESULT {
    TRACE!(b"(%p,%p)\n\0".as_ptr() as *const i8, iface, pwdMksys);

    if pwdMksys.is_null() {
        return E_POINTER;
    }

    *pwdMksys = MKSYS_GENERICCOMPOSITE;

    S_OK
}

unsafe fn CompositeMonikerROTDataImpl_QueryInterface(
    iface: *mut IROTData,
    riid: *const CLSID,
    ppvObject: *mut LPVOID
) -> HRESULT {
    let This = impl_from_IROTData(iface);

    TRACE!(
        b"(%p,%s,%p)\n\0".as_ptr(),
        iface,
        debugstr_guid(riid),
        ppvObject
    );

    CompositeMonikerImpl_QueryInterface(
        core::ptr::addr_of_mut!((*This).IMoniker_iface),
        riid,
        ppvObject
    )
}

unsafe fn CompositeMonikerROTDataImpl_AddRef(iface: *mut IROTData) -> ULONG {
    let This = impl_from_IROTData(iface);


    TRACE!(b"(%p)\n\0".as_ptr(), iface);

    IMoniker_AddRef(core::ptr::addr_of_mut!((*This).IMoniker_iface))
}

unsafe fn CompositeMonikerROTDataImpl_Release(iface: *mut IROTData) -> ULONG {

    let This = impl_from_IROTData(iface);

    TRACE!(b"(%p)\n\0".as_ptr(), iface);

    IMoniker_Release(core::ptr::addr_of_mut!((*This).IMoniker_iface))
}
unsafe fn CompositeMonikerROTDataImpl_GetComparisonData(
    iface: *mut IROTData,
    pbData: *mut u8,
    cbMax: u32,
    pcbData: *mut u32,
) -> HRESULT {
    let This = impl_from_IROTData(iface);
    let mut pEnumMk: *mut IEnumMoniker = core::ptr::null_mut();
    let mut pmk: *mut IMoniker;
    let mut hr: HRESULT;

    TRACE!(b"(%p, %u, %p)\n\0".as_ptr() as *const i8, pbData, cbMax, pcbData);

    *pcbData = core::mem::size_of::<CLSID>() as u32;

    hr = IMoniker_Enum(&mut (*This).IMoniker_iface, 1u8, &mut pEnumMk);
    if FAILED(hr) {
        return hr;
    }

    loop {
        pmk = core::ptr::null_mut();
        if IEnumMoniker_Next(pEnumMk, 1, &mut pmk, core::ptr::null_mut()) != 0 {
            break;
        }

        let mut pROTData: *mut IROTData = core::ptr::null_mut();
        hr = IMoniker_QueryInterface(pmk, &IID_IROTData, &mut pROTData as *mut *mut IROTData as *mut LPVOID);
        if FAILED(hr) {
            ERR!(b"moniker doesn't support IROTData interface\n\0".as_ptr() as *const i8);
        }

        if SUCCEEDED(hr) {
            let mut cbData: u32 = 0;
            hr = IROTData_GetComparisonData(pROTData, core::ptr::null_mut(), 0, &mut cbData);
            IROTData_Release(pROTData);
            if SUCCEEDED(hr) || (hr == 0x8007000E) {
                *pcbData += cbData;
                hr = 0;
            } else {
                ERR!(b"IROTData_GetComparisonData failed with error 0x%08x\n\0".as_ptr() as *const i8, hr);
            }
        }

        IMoniker_Release(pmk);

        if FAILED(hr) {
            IEnumMoniker_Release(pEnumMk);
            return hr;
        }
    }

    if cbMax < *pcbData {
        return 0x8007000E;
    }

    IEnumMoniker_Reset(pEnumMk);

    core::ptr::copy_nonoverlapping(&CLSID_CompositeMoniker as *const CLSID as *const u8, pbData, core::mem::size_of::<CLSID>());
    let mut pbData = pbData.add(core::mem::size_of::<CLSID>());
    let mut cbMax = cbMax - core::mem::size_of::<CLSID>() as u32;

    loop {
        pmk = core::ptr::null_mut();
        if IEnumMoniker_Next(pEnumMk, 1, &mut pmk, core::ptr::null_mut()) != 0 {
            break;
        }

        let mut pROTData: *mut IROTData = core::ptr::null_mut();
        hr = IMoniker_QueryInterface(pmk, &IID_IROTData, &mut pROTData as *mut *mut IROTData as *mut LPVOID);
        if FAILED(hr) {
            ERR!(b"moniker doesn't support IROTData interface\n\0".as_ptr() as *const i8);
        }

        if SUCCEEDED(hr) {
            let mut cbData: u32 = 0;
            hr = IROTData_GetComparisonData(pROTData, pbData, cbMax, &mut cbData);
            IROTData_Release(pROTData);
            if SUCCEEDED(hr) {
                pbData = pbData.add(cbData as usize);
                cbMax -= cbData;
            } else {
                ERR!(b"IROTData_GetComparisonData failed with error 0x%08x\n\0".as_ptr() as *const i8, hr);
            }
        }

        IMoniker_Release(pmk);

        if FAILED(hr) {
            IEnumMoniker_Release(pEnumMk);
            return hr;
        }
    }

    IEnumMoniker_Release(pEnumMk);

    0
}

unsafe fn CompositeMonikerMarshalImpl_QueryInterface(
    iface: *mut IMarshal,
    riid: *const CLSID,
    ppv: *mut LPVOID
) -> HRESULT {
    let This = impl_from_IMarshal(iface);


    TRACE!(
        b"(%p,%s,%p)\n\0".as_ptr(),
        iface,
        debugstr_guid(riid),
        ppv
    );

    CompositeMonikerImpl_QueryInterface(
        core::ptr::addr_of_mut!((*This).IMoniker_iface),
        riid,
        ppv
    )
}

unsafe fn CompositeMonikerMarshalImpl_AddRef(iface: *mut IMarshal) -> u32 {
    let This = impl_from_IMarshal(iface);
    CompositeMonikerImpl_AddRef(core::ptr::addr_of_mut!((*This).IMoniker_iface))
}

unsafe fn CompositeMonikerMarshalImpl_Release(iface: *mut IMarshal) -> ULONG {
    let This = impl_from_IMarshal(iface);

    CompositeMonikerImpl_Release(core::ptr::addr_of_mut!((*This).IMoniker_iface))
}
unsafe fn CompositeMonikerMarshalImpl_GetUnmarshalClass(
    iface: *mut IMarshal,
    riid: *const CLSID,
    pv: LPVOID,
    dwDestContext: u32,
    pvDestContext: LPVOID,
    mshlflags: u32,
    pCid: *mut CLSID,
) -> HRESULT {
    let This = impl_from_IMarshal(iface);

    TRACE!(
        b"(%s, %p, %x, %p, %x, %p)\n\0".as_ptr() as *const i8,
        debugstr_guid(riid),
        pv,
        dwDestContext,
        pvDestContext,
        mshlflags,
        pCid,
    );

    IMoniker_GetClassID(core::ptr::addr_of_mut!((*This).IMoniker_iface), pCid)
}

unsafe fn CompositeMonikerMarshalImpl_GetMarshalSizeMax(
    iface: *mut IMarshal,
    riid: *const CLSID,
    pv: LPVOID,
    dwDestContext: u32,
    pvDestContext: LPVOID,
    mshlflags: u32,
    pSize: *mut u32,
) -> HRESULT {

    let This = impl_from_IMarshal(iface);
    let mut pEnumMk: *mut IEnumMoniker = core::ptr::null_mut();
    let mut pmk: *mut IMoniker = core::ptr::null_mut();
    let mut hr: HRESULT;
    let mut size: ULARGE_INTEGER = core::mem::zeroed();

    TRACE!(
        b"(%s, %p, %x, %p, %x, %p)\n\0".as_ptr(),
        debugstr_guid(riid),
        pv,
        dwDestContext,
        pvDestContext,
        mshlflags,
        pSize,
    );

    *pSize = 0x10;

    hr = ((*(*This).IMoniker_iface.lpVtbl).Enum)(
        &mut (*This).IMoniker_iface,
        1u8,
        &mut pEnumMk,
    );
    if FAILED(hr) != 0 {
        return hr;
    }

    hr = ((*(*This).IMoniker_iface.lpVtbl).GetSizeMax)(
        &mut (*This).IMoniker_iface,
        &mut size,
    );

    while ((*(*pEnumMk).lpVtbl).Next)(
        pEnumMk,
        1,
        &mut pmk,
        core::ptr::null_mut(),
    ) == 0
    {
        let mut size: u32 = 0;

        hr = CoGetMarshalSizeMax(
            &mut size,
            &IID_IMoniker,
            pmk as *mut core::ffi::c_void,
            dwDestContext,
            pvDestContext,
            mshlflags,
        );
        if SUCCEEDED(hr) != 0 {
            *pSize = (*pSize).wrapping_add(size);
        }

        ((*(*pmk).lpVtbl).Release)(pmk);

        if FAILED(hr) != 0 {
            ((*(*pEnumMk).lpVtbl).Release)(pEnumMk);
            return hr;
        }
    }

    ((*(*pEnumMk).lpVtbl).Release)(pEnumMk);

    0
}

unsafe fn CompositeMonikerMarshalImpl_MarshalInterface(
    iface: *mut IMarshal,
    pStm: *mut IStream,
    riid: *const CLSID,
    pv: LPVOID,
    dwDestContext: u32,
    pvDestContext: LPVOID,
    mshlflags: u32,
) -> HRESULT {

    let This = impl_from_IMarshal(iface);
    let mut pEnumMk: *mut IEnumMoniker = core::ptr::null_mut();
    let mut pmk: *mut IMoniker;
    let mut hr: HRESULT;
    let mut i: u32 = 0;

    TRACE!(
        b"(%p, %s, %p, %x, %p, %x)\n\0".as_ptr() as *const i8,
        pStm,
        debugstr_guid(riid),
        pv,
        dwDestContext,
        pvDestContext,
        mshlflags,
    );

    hr = IMoniker_Enum(
        core::ptr::addr_of_mut!((*This).IMoniker_iface),
        1,
        core::ptr::addr_of_mut!(pEnumMk),
    );
    if FAILED(hr) != 0 {
        return hr;
    }

    loop {
        pmk = core::ptr::null_mut();
        if IEnumMoniker_Next(pEnumMk, 1, core::ptr::addr_of_mut!(pmk), core::ptr::null_mut()) != 0 {
            break;
        }

        hr = CoMarshalInterface(
            pStm,
            core::ptr::addr_of!(ntdll_sys::nt::IID_IMoniker),
            pmk as *mut core::ffi::c_void,
            dwDestContext,
            pvDestContext,
            mshlflags,
        );

        IMoniker_Release(pmk);

        if FAILED(hr) != 0 {
            IEnumMoniker_Release(pEnumMk);
            return hr;
        }
        i += 1;
    }

    if i != 2 {
        FIXME!(b"moniker count of %d not supported\n\0".as_ptr() as *const i8, i);
    }

    IEnumMoniker_Release(pEnumMk);

    0
}

unsafe fn CompositeMonikerMarshalImpl_UnmarshalInterface(
    iface: *mut IMarshal,
    pStm: *mut IStream,
    riid: *const CLSID,
    ppv: *mut LPVOID,
) -> HRESULT {
    let This = impl_from_IMarshal(iface);
    let mut hr: HRESULT;

    TRACE!(
        b"(%p, %s, %p)\n\0".as_ptr() as *const i8,
        pStm,
        debugstr_guid(riid),
        ppv,
    );

    CompositeMonikerImpl_ReleaseMonikersInTable(This);

    if (*This).tabLastIndex + 2 > (*This).tabSize {
        (*This).tabSize += max(BLOCK_TAB_SIZE, 2);
        (*This).tabMoniker = HeapReAlloc(
            GetProcessHeap(),
            0,
            (*This).tabMoniker as LPVOID,
            ((*This).tabSize as usize) * core::mem::size_of::<*mut IMoniker>(),
        ) as *mut *mut IMoniker;

        if (*This).tabMoniker.is_null() {
            return E_OUTOFMEMORY;
        }
    }

    hr = CoUnmarshalInterface(
        pStm,
        &IID_IMoniker,
        core::ptr::addr_of_mut!((*This).tabMoniker.add((*This).tabLastIndex as usize).read()) as *mut LPVOID,
    );
    if FAILED(hr) {
        ERR!(
            b"couldn't unmarshal moniker, hr = 0x%08x\n\0".as_ptr() as *const i8,
            hr,
        );
        return hr;
    }
    (*This).tabLastIndex += 1;
    hr = CoUnmarshalInterface(
        pStm,
        &IID_IMoniker,
        core::ptr::addr_of_mut!((*This).tabMoniker.add((*This).tabLastIndex as usize).read()) as *mut LPVOID,
    );
    if FAILED(hr) {
        ERR!(
            b"couldn't unmarshal moniker, hr = 0x%08x\n\0".as_ptr() as *const i8,
            hr,
        );
        return hr;
    }
    (*This).tabLastIndex += 1;

    IMoniker_QueryInterface(
        core::ptr::addr_of_mut!((*This).IMoniker_iface),
        riid,
        ppv,
    )
}

unsafe fn CompositeMonikerMarshalImpl_ReleaseMarshalData(
    iface: *mut IMarshal,
    pStm: *mut IStream,
) -> HRESULT {
    TRACE!(b"(%p)\n\0".as_ptr(), pStm);
    0
}

unsafe fn CompositeMonikerMarshalImpl_DisconnectObject(
    iface: *mut IMarshal,
    dwReserved: u32,
) -> HRESULT {
    TRACE!(b"(0x%x)\n\0".as_ptr(), dwReserved);
    0i32
}

unsafe fn EnumMonikerImpl_QueryInterface(
    iface: *mut IEnumMoniker,
    riid: *const CLSID,
    ppvObject: *mut LPVOID
) -> HRESULT {
    let This = impl_from_IEnumMoniker(iface);

    TRACE!(b"(%p,%s,%p)\n\0".as_ptr() as *const i8, This, debugstr_guid(riid), ppvObject);

    if ppvObject.is_null() {
        return E_INVALIDARG;
    }

    *ppvObject = core::ptr::null_mut();

    if IsEqualIID(&IID_IUnknown, riid) != 0 || IsEqualIID(&IID_IEnumMoniker, riid) != 0 {
        *ppvObject = iface as LPVOID;
    }

    if (*ppvObject).is_null() {
        return E_NOINTERFACE;
    }

    IEnumMoniker_AddRef(iface);

    S_OK
}

unsafe fn EnumMonikerImpl_AddRef(iface: *mut IEnumMoniker) -> ULONG {
    let This = impl_from_IEnumMoniker(iface);

    TRACE!(b"(%p)\n\0".as_ptr() as *const i8, This);

    InterlockedIncrement(&mut (*This).ref_) as ULONG
}

unsafe fn EnumMonikerImpl_Release(iface: *mut IEnumMoniker) -> u32 {
    let This = impl_from_IEnumMoniker(iface);
    let mut i: u32;
    let ref_: u32;
    TRACE!(b"(%p)\n\0".as_ptr() as *const i8, This);

    ref_ = InterlockedDecrement(&mut (*This).ref_) as u32;

    if ref_ == 0 {
        i = 0;
        while i < (*This).tabSize {
            IMoniker_Release(*(*This).tabMoniker.offset(i as isize));
            i += 1;
        }

        HeapFree(GetProcessHeap(), 0, (*This).tabMoniker as LPVOID);
        HeapFree(GetProcessHeap(), 0, This as LPVOID);
    }
    ref_
}
unsafe fn EnumMonikerImpl_Next(
    iface: *mut IEnumMoniker,
    celt: ULONG,
    rgelt: *mut *mut IMoniker,
    pceltFethed: *mut ULONG,
) -> HRESULT {
    let This = impl_from_IEnumMoniker(iface);
    let mut i: ULONG = 0;

    while (*This).currentPos < (*This).tabSize && i < celt {
        *rgelt.add(i as usize) = *(*This).tabMoniker.add((*This).currentPos as usize);
        (*This).currentPos += 1;
        IMoniker_AddRef(*rgelt.add(i as usize));
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

unsafe fn EnumMonikerImpl_Skip(iface: *mut IEnumMoniker, celt: ULONG) -> HRESULT {
    let This = impl_from_IEnumMoniker(iface);

    if ((*This).currentPos + celt) >= (*This).tabSize {
        return S_FALSE;
    }

    (*This).currentPos += celt;

    return S_OK;
}

unsafe fn EnumMonikerImpl_Reset(iface: *mut IEnumMoniker) -> HRESULT {
    let This = impl_from_IEnumMoniker(iface);

    (*This).currentPos = 0;

    S_OK
}

unsafe fn EnumMonikerImpl_Clone(
    iface: *mut IEnumMoniker,
    ppenum: *mut *mut IEnumMoniker,
) -> HRESULT {
    let This = impl_from_IEnumMoniker(iface);

    EnumMonikerImpl_CreateEnumMoniker(
        (*This).tabMoniker,
        (*This).tabSize,
        (*This).currentPos,
        1,
        ppenum,
    )
}

unsafe fn EnumMonikerImpl_CreateEnumMoniker(
    tabMoniker: *mut *mut IMoniker,
    tabSize: ULONG,
    currentPos: ULONG,
    leftToRight: BOOL,
    ppmk: *mut *mut IEnumMoniker,
) -> HRESULT {
    let mut newEnumMoniker: *mut EnumMonikerImpl;
    let mut i: ULONG;

    if currentPos > tabSize {
        return E_INVALIDARG;
    }

    newEnumMoniker = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<EnumMonikerImpl>(),
    ) as *mut EnumMonikerImpl;

    if newEnumMoniker == core::ptr::null_mut() {
        return STG_E_INSUFFICIENTMEMORY;
    }

    (*newEnumMoniker).IEnumMoniker_iface.lpVtbl = &VT_EnumMonikerImpl;
    (*newEnumMoniker).ref_ = 1;

    (*newEnumMoniker).tabSize = tabSize;
    (*newEnumMoniker).currentPos = currentPos;

    (*newEnumMoniker).tabMoniker = HeapAlloc(
        GetProcessHeap(),
        0,
        (tabSize as usize) * core::mem::size_of::<*mut IMoniker>(),
    ) as *mut *mut IMoniker;

    if (*newEnumMoniker).tabMoniker == core::ptr::null_mut() {
        HeapFree(GetProcessHeap(), 0, newEnumMoniker as LPVOID);
        return E_OUTOFMEMORY;
    }

    if leftToRight != 0 {
        i = 0;
        while i < tabSize {
            *(*newEnumMoniker).tabMoniker.add(i as usize) = *tabMoniker.add(i as usize);
            IMoniker_AddRef(*tabMoniker.add(i as usize));
            i += 1;
        }
    } else {
        i = tabSize;
        while i > 0 {
            *(*newEnumMoniker).tabMoniker.add((tabSize - i) as usize) = *tabMoniker.add((i - 1) as usize);
            IMoniker_AddRef(*tabMoniker.add((i - 1) as usize));
            i -= 1;
        }
    }

    *ppmk = &mut (*newEnumMoniker).IEnumMoniker_iface;

    S_OK
}

unsafe fn CompositeMonikerImpl_Construct(ppMoniker: *mut *mut IMoniker, pmkFirst: *mut IMoniker, pmkRest: *mut IMoniker) -> HRESULT {
    let mut mkSys: u32 = 0;
    let mut enumMoniker: *mut IEnumMoniker = core::ptr::null_mut();
    let mut tempMk: *mut IMoniker = core::ptr::null_mut();
    let mut res: HRESULT;
    let This: *mut CompositeMonikerImpl;
    let mut i: i32;

    This = HeapAlloc(GetProcessHeap(), 0, core::mem::size_of::<CompositeMonikerImpl>()) as *mut CompositeMonikerImpl;

    if This.is_null() {
        return E_OUTOFMEMORY;
    }

    TRACE!(b"(%p,%p,%p)\n\0".as_ptr() as *const i8, This, pmkFirst, pmkRest);

    (*This).IMoniker_iface.lpVtbl = &VT_CompositeMonikerImpl;
    (*This).IROTData_iface.lpVtbl = &VT_ROTDataImpl;
    (*This).IMarshal_iface.lpVtbl = &VT_MarshalImpl;
    (*This).ref_ = 1;

    (*This).tabSize = BLOCK_TAB_SIZE;
    (*This).tabLastIndex = 0;

    (*This).tabMoniker = HeapAlloc(GetProcessHeap(), 0, (*This).tabSize * core::mem::size_of::<*mut IMoniker>()) as *mut *mut IMoniker;
    if (*This).tabMoniker.is_null() {
        HeapFree(GetProcessHeap(), 0, This as LPVOID);
        return E_OUTOFMEMORY;
    }

    if pmkFirst.is_null() && pmkRest.is_null() {
        *ppMoniker = &mut (*This).IMoniker_iface;
        return S_OK;
    }

    IMoniker_IsSystemMoniker(pmkFirst, &mut mkSys);

    if mkSys != MKSYS_GENERICCOMPOSITE {
        *(*This).tabMoniker.offset((*This).tabLastIndex as isize) = pmkFirst;
        (*This).tabLastIndex += 1;
        IMoniker_AddRef(pmkFirst);
    } else {
        IMoniker_Enum(pmkFirst, 1u8, &mut enumMoniker);

        while IEnumMoniker_Next(enumMoniker, 1, &mut *(*This).tabMoniker.offset((*This).tabLastIndex as isize), core::ptr::null_mut()) == S_OK {
            (*This).tabLastIndex += 1;
            if (*This).tabLastIndex == (*This).tabSize {
                let tab_moniker = (*This).tabMoniker;

                (*This).tabSize += BLOCK_TAB_SIZE;
                (*This).tabMoniker = HeapReAlloc(GetProcessHeap(), 0, (*This).tabMoniker as LPVOID, (*This).tabSize * core::mem::size_of::<*mut IMoniker>()) as *mut *mut IMoniker;

                if (*This).tabMoniker.is_null() {
                    i = 0;
                    while i < (*This).tabLastIndex as i32 {
                        IMoniker_Release(*tab_moniker.offset(i as isize));
                        i += 1;
                    }
                    HeapFree(GetProcessHeap(), 0, tab_moniker as LPVOID);
                    HeapFree(GetProcessHeap(), 0, This as LPVOID);
                    return E_OUTOFMEMORY;
                }
            }
        }

        IEnumMoniker_Release(enumMoniker);
    }

    IMoniker_IsSystemMoniker(pmkRest, &mut mkSys);

    if mkSys != MKSYS_GENERICCOMPOSITE {
        res = IMoniker_ComposeWith(*(*This).tabMoniker.offset(((*This).tabLastIndex - 1) as isize), pmkRest, 1u8, &mut tempMk);

        if res == MK_E_NEEDGENERIC {
            *(*This).tabMoniker.offset((*This).tabLastIndex as isize) = pmkRest;
            (*This).tabLastIndex += 1;
            IMoniker_AddRef(pmkRest);
        } else if tempMk.is_null() {
            IMoniker_Release(*(*This).tabMoniker.offset(((*This).tabLastIndex - 1) as isize));
            (*This).tabLastIndex -= 1;
        } else if SUCCEEDED(res) {
            IMoniker_Release(*(*This).tabMoniker.offset(((*This).tabLastIndex - 1) as isize));
            *(*This).tabMoniker.offset(((*This).tabLastIndex - 1) as isize) = tempMk;
        } else {
            i = 0;
            while i < (*This).tabLastIndex as i32 {
                IMoniker_Release(*(*This).tabMoniker.offset(i as isize));
                i += 1;
            }
            HeapFree(GetProcessHeap(), 0, (*This).tabMoniker as LPVOID);
            HeapFree(GetProcessHeap(), 0, This as LPVOID);
            return res;
        }

        if (*This).tabLastIndex == (*This).tabSize {
            let tab_moniker = (*This).tabMoniker;

            (*This).tabSize += BLOCK_TAB_SIZE;
            (*This).tabMoniker = HeapReAlloc(GetProcessHeap(), 0, (*This).tabMoniker as LPVOID, (*This).tabSize * core::mem::size_of::<IMoniker>()) as *mut *mut IMoniker;

            if (*This).tabMoniker.is_null() {
                i = 0;
                while i < (*This).tabLastIndex as i32 {
                    IMoniker_Release(*tab_moniker.offset(i as isize));
                    i += 1;
                }
                HeapFree(GetProcessHeap(), 0, tab_moniker as LPVOID);
                HeapFree(GetProcessHeap(), 0, This as LPVOID);
                return E_OUTOFMEMORY;
            }
        }
    } else {
        IMoniker_Enum(pmkRest, 1u8, &mut enumMoniker);

        while IEnumMoniker_Next(enumMoniker, 1, &mut *(*This).tabMoniker.offset((*This).tabLastIndex as isize), core::ptr::null_mut()) == S_OK {
            res = IMoniker_ComposeWith(*(*This).tabMoniker.offset(((*This).tabLastIndex - 1) as isize), *(*This).tabMoniker.offset((*This).tabLastIndex as isize), 1u8, &mut tempMk);

            if res == MK_E_NEEDGENERIC {
                (*This).tabLastIndex += 1;
            } else if tempMk.is_null() {
                IMoniker_Release(*(*This).tabMoniker.offset(((*This).tabLastIndex - 1) as isize));
                IMoniker_Release(*(*This).tabMoniker.offset((*This).tabLastIndex as isize));
                (*This).tabLastIndex -= 1;
            } else {
                IMoniker_Release(*(*This).tabMoniker.offset(((*This).tabLastIndex - 1) as isize));
                *(*This).tabMoniker.offset(((*This).tabLastIndex - 1) as isize) = tempMk;
            }

            if (*This).tabLastIndex == (*This).tabSize {
                let tab_moniker = (*This).tabMoniker;

                (*This).tabSize += BLOCK_TAB_SIZE;
                (*This).tabMoniker = HeapReAlloc(GetProcessHeap(), 0, (*This).tabMoniker as LPVOID, (*This).tabSize * core::mem::size_of::<*mut IMoniker>()) as *mut *mut IMoniker;

                if (*This).tabMoniker.is_null() {
                    i = 0;
                    while i < (*This).tabLastIndex as i32 {
                        IMoniker_Release(*tab_moniker.offset(i as isize));
                        i += 1;
                    }
                    HeapFree(GetProcessHeap(), 0, tab_moniker as LPVOID);
                    HeapFree(GetProcessHeap(), 0, This as LPVOID);
                    return E_OUTOFMEMORY;
                }
            }
        }

        IEnumMoniker_Release(enumMoniker);
    }

    if (*This).tabLastIndex == 1 {
        *ppMoniker = *(*This).tabMoniker.offset(0);
        IMoniker_AddRef(*ppMoniker);
        IMoniker_Release(&mut (*This).IMoniker_iface);
    } else {
        *ppMoniker = &mut (*This).IMoniker_iface;
    }

    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn CreateGenericComposite(
    pmkFirst: *mut IMoniker,
    pmkRest: *mut IMoniker,
    ppmkComposite: *mut *mut IMoniker,
) -> HRESULT {

    const S_OK: HRESULT = 0;
    const E_POINTER: HRESULT = 0x80004003u32 as HRESULT;

    let mut moniker: *mut IMoniker = core::ptr::null_mut();
    let mut hr: HRESULT = S_OK;

    TRACE!(
        b"(%p,%p,%p)\n\0".as_ptr(),
        pmkFirst,
        pmkRest,
        ppmkComposite,
    );

    if ppmkComposite.is_null() {
        return E_POINTER;
    }

    *ppmkComposite = core::ptr::null_mut();

    if pmkFirst.is_null() && !pmkRest.is_null() {
        *ppmkComposite = pmkRest;
        IMoniker_AddRef(pmkRest);
        return S_OK;
    } else if !pmkFirst.is_null() && pmkRest.is_null() {
        *ppmkComposite = pmkFirst;
        IMoniker_AddRef(pmkFirst);
        return S_OK;
    } else if pmkFirst.is_null() && pmkRest.is_null() {
        return S_OK;
    }

    hr = CompositeMonikerImpl_Construct(&mut moniker, pmkFirst, pmkRest);

    if FAILED(hr) {
        return hr;
    }

    hr = IMoniker_QueryInterface(moniker, &IID_IMoniker, ppmkComposite as *mut LPVOID);
    IMoniker_Release(moniker);

    hr
}

#[no_mangle]
pub unsafe extern "system" fn MonikerCommonPrefixWith(
    pmkThis: *mut IMoniker,
    pmkOther: *mut IMoniker,
    ppmkCommon: *mut *mut IMoniker,
) -> HRESULT {
    FIXME!(b"(),stub!\n\0".as_ptr());
    0x80004001u32 as HRESULT
}

#[no_mangle]
pub unsafe extern "system" fn CompositeMoniker_CreateInstance(
    iface: *mut IClassFactory,
    pUnk: *mut IUnknown,
    riid: *const CLSID,
    ppv: *mut LPVOID,
) -> HRESULT {

    let mut pMoniker: *mut IMoniker = core::ptr::null_mut();
    let mut hr: HRESULT;

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

    hr = CompositeMonikerImpl_Construct(
        &mut pMoniker,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    if SUCCEEDED(hr) != 0 {
        hr = IMoniker_QueryInterface(pMoniker, riid, ppv);
        IMoniker_Release(pMoniker);
    }

    hr
}
