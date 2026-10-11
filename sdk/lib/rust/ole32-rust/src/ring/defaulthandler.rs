use crate::prelude::*;

/// com-sys spells the COM vtbl slots `unsafe extern "stdcall" fn` on i386 and
/// `unsafe extern "C" fn` on amd64/arm64 — exactly what STDMETHODCALLTYPE compiles to
/// per arch. `extern "system"` *lowers* to the same ABI but rustc keeps it a DISTINCT
/// fn type that will not unify with the bindgen slot type, so the vtbl impls must spell
/// the ABI the way com-sys does. Same macro as ring/hglobalstream.rs, extended to also
/// take the void-returning form (IAdviseSink::OnDataChange & friends).
macro_rules! stdmethod {
    ($(
        $(#[$attr:meta])*
        unsafe fn $name:ident ( $($arg:ident : $ty:ty),* $(,)? ) $(-> $ret:ty)? $body:block
    )*) => {$(
        $(#[$attr])*
        unsafe extern "system" fn $name($($arg: $ty),*) $(-> $ret)? $body
    )*};
}

// ole2.h:47-50. win_consts.rs only carries EMBDHLP_INPROC_SERVER (0x1) and
// EMBDHLP_DELAYCREATE (0x10000); these two belong there as well (reported), but they
// are spelled here from the header rather than left as the raw `0x1 | 0x4` the
// translation invented for OleCreateDefaultHandler (see the call site).

// ---- file-scope preamble (translated) ----
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct storage_state(pub i32);
pub const storage_state_uninitialised: storage_state = storage_state(0);
pub const storage_state_initialised: storage_state = storage_state(1);
pub const storage_state_loaded: storage_state = storage_state(2);

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct object_state(pub i32);
pub const object_state_not_running: object_state = object_state(0);
pub const object_state_running: object_state = object_state(1);
pub const object_state_deferred_close: object_state = object_state(2);

#[repr(C)]
pub struct DefaultHandler {
    pub IOleObject_iface: IOleObject,
    pub IUnknown_iface: IUnknown,
    pub IDataObject_iface: IDataObject,
    pub IRunnableObject_iface: IRunnableObject,
    pub IAdviseSink_iface: IAdviseSink,
    pub IPersistStorage_iface: IPersistStorage,
    pub ref_: LONG,
    pub outerUnknown: *mut IUnknown,
    pub clsid: CLSID,
    pub dataCache: *mut IUnknown,
    pub dataCache_PersistStg: *mut IPersistStorage,
    pub clientSite: *mut IOleClientSite,
    pub oleAdviseHolder: *mut IOleAdviseHolder,
    pub dataAdviseHolder: *mut IDataAdviseHolder,
    pub containerApp: LPWSTR,
    pub containerObj: LPWSTR,
    pub pOleDelegate: *mut IOleObject,
    pub pPSDelegate: *mut IPersistStorage,
    pub pDataDelegate: *mut IDataObject,
    pub object_state: object_state,
    pub in_call: ULONG,
    pub dwAdvConn: DWORD,
    pub storage: *mut IStorage,
    pub storage_state: storage_state,
    pub pCFObject: *mut IClassFactory,
    pub inproc_server: BOOL,
}
pub type PDefaultHandler = *mut DefaultHandler;

pub static OleStream: [WCHAR; 5] = [1, 'O' as u16, 'l' as u16, 'e' as u16, 0];

#[repr(C)]
pub struct ole_stream_header_t {
    pub version: DWORD,
    pub flags: DWORD,
    pub link_update_opt: DWORD,
    pub res: DWORD,
    pub moniker_size: DWORD,
}

pub const ole_stream_version: DWORD = 0x02000001;

pub static DefaultHandler_IOleObject_VTable: IOleObjectVtbl = IOleObjectVtbl {
    QueryInterface: Some(DefaultHandler_QueryInterface),
    AddRef: Some(DefaultHandler_AddRef),
    Release: Some(DefaultHandler_Release),
    SetClientSite: Some(DefaultHandler_SetClientSite),
    GetClientSite: Some(DefaultHandler_GetClientSite),
    SetHostNames: Some(DefaultHandler_SetHostNames),
    Close: Some(DefaultHandler_Close),
    SetMoniker: Some(DefaultHandler_SetMoniker),
    GetMoniker: Some(DefaultHandler_GetMoniker),
    InitFromData: Some(DefaultHandler_InitFromData),
    GetClipboardData: Some(DefaultHandler_GetClipboardData),
    DoVerb: Some(DefaultHandler_DoVerb),
    EnumVerbs: Some(DefaultHandler_EnumVerbs),
    Update: Some(DefaultHandler_Update),
    IsUpToDate: Some(DefaultHandler_IsUpToDate),
    GetUserClassID: Some(DefaultHandler_GetUserClassID),
    GetUserType: Some(DefaultHandler_GetUserType),
    SetExtent: Some(DefaultHandler_SetExtent),
    GetExtent: Some(DefaultHandler_GetExtent),
    Advise: Some(DefaultHandler_Advise),
    Unadvise: Some(DefaultHandler_Unadvise),
    EnumAdvise: Some(DefaultHandler_EnumAdvise),
    GetMiscStatus: Some(DefaultHandler_GetMiscStatus),
    SetColorScheme: Some(DefaultHandler_SetColorScheme),
};

pub static DefaultHandler_NDIUnknown_VTable: IUnknownVtbl = IUnknownVtbl {
    QueryInterface: Some(DefaultHandler_NDIUnknown_QueryInterface),
    AddRef: Some(DefaultHandler_NDIUnknown_AddRef),
    Release: Some(DefaultHandler_NDIUnknown_Release),
};

pub static DefaultHandler_IDataObject_VTable: IDataObjectVtbl = IDataObjectVtbl {
    QueryInterface: Some(DefaultHandler_IDataObject_QueryInterface),
    AddRef: Some(DefaultHandler_IDataObject_AddRef),
    Release: Some(DefaultHandler_IDataObject_Release),
    GetData: Some(DefaultHandler_GetData),
    GetDataHere: Some(DefaultHandler_GetDataHere),
    QueryGetData: Some(DefaultHandler_QueryGetData),
    GetCanonicalFormatEtc: Some(DefaultHandler_GetCanonicalFormatEtc),
    SetData: Some(DefaultHandler_SetData),
    EnumFormatEtc: Some(DefaultHandler_EnumFormatEtc),
    DAdvise: Some(DefaultHandler_DAdvise),
    DUnadvise: Some(DefaultHandler_DUnadvise),
    EnumDAdvise: Some(DefaultHandler_EnumDAdvise),
};

pub static DefaultHandler_IRunnableObject_VTable: IRunnableObjectVtbl = IRunnableObjectVtbl {
    QueryInterface: Some(DefaultHandler_IRunnableObject_QueryInterface),
    AddRef: Some(DefaultHandler_IRunnableObject_AddRef),
    Release: Some(DefaultHandler_IRunnableObject_Release),
    GetRunningClass: Some(DefaultHandler_GetRunningClass),
    Run: Some(DefaultHandler_Run),
    IsRunning: Some(DefaultHandler_IsRunning),
    LockRunning: Some(DefaultHandler_LockRunning),
    SetContainedObject: Some(DefaultHandler_SetContainedObject),
};

pub static DefaultHandler_IAdviseSink_VTable: IAdviseSinkVtbl = IAdviseSinkVtbl {
    QueryInterface: Some(DefaultHandler_IAdviseSink_QueryInterface),
    AddRef: Some(DefaultHandler_IAdviseSink_AddRef),
    Release: Some(DefaultHandler_IAdviseSink_Release),
    OnDataChange: Some(DefaultHandler_IAdviseSink_OnDataChange),
    OnViewChange: Some(DefaultHandler_IAdviseSink_OnViewChange),
    OnRename: Some(DefaultHandler_IAdviseSink_OnRename),
    OnSave: Some(DefaultHandler_IAdviseSink_OnSave),
    OnClose: Some(DefaultHandler_IAdviseSink_OnClose),
};

pub static DefaultHandler_IPersistStorage_VTable: IPersistStorageVtbl = IPersistStorageVtbl {
    QueryInterface: Some(DefaultHandler_IPersistStorage_QueryInterface),
    AddRef: Some(DefaultHandler_IPersistStorage_AddRef),
    Release: Some(DefaultHandler_IPersistStorage_Release),
    GetClassID: Some(DefaultHandler_IPersistStorage_GetClassID),
    IsDirty: Some(DefaultHandler_IPersistStorage_IsDirty),
    InitNew: Some(DefaultHandler_IPersistStorage_InitNew),
    Load: Some(DefaultHandler_IPersistStorage_Load),
    Save: Some(DefaultHandler_IPersistStorage_Save),
    SaveCompleted: Some(DefaultHandler_IPersistStorage_SaveCompleted),
    HandsOffStorage: Some(DefaultHandler_IPersistStorage_HandsOffStorage),
};

#[repr(C)]
pub struct HandlerCF {
    pub IClassFactory_iface: IClassFactory,
    pub refs: LONG,
    pub clsid: CLSID,
}
pub type PHandlerCF = *mut HandlerCF;

pub static HandlerClassFactoryVtbl: IClassFactoryVtbl = IClassFactoryVtbl {
    QueryInterface: Some(HandlerCF_QueryInterface),
    AddRef: Some(HandlerCF_AddRef),
    Release: Some(HandlerCF_Release),
    CreateInstance: Some(HandlerCF_CreateInstance),
    LockServer: Some(HandlerCF_LockServer),
};

unsafe fn impl_from_IOleObject(iface: *mut IOleObject) -> *mut DefaultHandler {
    (iface as *mut u8).offset(-(core::mem::offset_of!(DefaultHandler, IOleObject_iface) as isize)) as *mut DefaultHandler
}

unsafe fn impl_from_IUnknown(iface: *mut IUnknown) -> *mut DefaultHandler {
    (iface as *mut u8).offset(-(core::mem::offset_of!(DefaultHandler, IUnknown_iface) as isize)) as *mut DefaultHandler
}

unsafe fn impl_from_IDataObject(iface: *mut IDataObject) -> *mut DefaultHandler {
    (iface as *mut u8).offset(-(core::mem::offset_of!(DefaultHandler, IDataObject_iface) as isize)) as *mut DefaultHandler
}

unsafe fn impl_from_IRunnableObject(iface: *mut IRunnableObject) -> *mut DefaultHandler {
    (iface as *mut u8).offset(-(core::mem::offset_of!(DefaultHandler, IRunnableObject_iface) as isize)) as *mut DefaultHandler
}

unsafe fn impl_from_IAdviseSink(iface: *mut IAdviseSink) -> *mut DefaultHandler {
    (iface as *mut u8).offset(-(core::mem::offset_of!(DefaultHandler, IAdviseSink_iface) as isize)) as *mut DefaultHandler
}

unsafe fn impl_from_IPersistStorage(iface: *mut IPersistStorage) -> *mut DefaultHandler {
    (iface as *mut u8).offset(-(core::mem::offset_of!(DefaultHandler, IPersistStorage_iface) as isize)) as *mut DefaultHandler
}

unsafe fn object_is_running(This: *mut DefaultHandler) -> BOOL {
    IRunnableObject_IsRunning(core::ptr::addr_of_mut!((*This).IRunnableObject_iface))
}

unsafe fn start_object_call(This: *mut DefaultHandler) {
    (*This).in_call = (*This).in_call.wrapping_add(1);
}

unsafe fn end_object_call(This: *mut DefaultHandler) {
    (*This).in_call = (*This).in_call.wrapping_sub(1);
    if (*This).in_call == 0 && (*This).object_state == object_state_deferred_close {
        DefaultHandler_Stop(This);
    }
}

stdmethod! {

unsafe fn DefaultHandler_NDIUnknown_QueryInterface(
    iface: *mut IUnknown,
    riid: *const GUID,
    ppvObject: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let This = impl_from_IUnknown(iface);

    if ppvObject.is_null() {
        return E_INVALIDARG;
    }

    *ppvObject = core::ptr::null_mut();

    if IsEqualIID(riid, &IID_IUnknown) {
        *ppvObject = iface as *mut core::ffi::c_void;
    } else if IsEqualIID(riid, &IID_IOleObject) {
        *ppvObject = core::ptr::addr_of_mut!((*This).IOleObject_iface) as *mut core::ffi::c_void;
    } else if IsEqualIID(riid, &IID_IDataObject) {
        *ppvObject = core::ptr::addr_of_mut!((*This).IDataObject_iface) as *mut core::ffi::c_void;
    } else if IsEqualIID(riid, &IID_IRunnableObject) {
        *ppvObject = core::ptr::addr_of_mut!((*This).IRunnableObject_iface) as *mut core::ffi::c_void;
    } else if IsEqualIID(riid, &IID_IPersist) || IsEqualIID(riid, &IID_IPersistStorage) {
        *ppvObject = core::ptr::addr_of_mut!((*This).IPersistStorage_iface) as *mut core::ffi::c_void;
    } else if IsEqualIID(riid, &IID_IViewObject)
        || IsEqualIID(riid, &IID_IViewObject2)
        || IsEqualIID(riid, &IID_IOleCache)
        || IsEqualIID(riid, &IID_IOleCache2)
    {
        let hr = IUnknown_QueryInterface((*This).dataCache, riid, ppvObject);
        if FAILED(hr) {
            FIXME!(
                b"interface %s not implemented by data cache\n\0".as_ptr() as PCHAR,
                debugstr_guid((riid).cast()),
            );
        }
        return hr;
    } else if (*This).inproc_server != 0 && !(*This).pOleDelegate.is_null() {
        return IOleObject_QueryInterface((*This).pOleDelegate, riid, ppvObject);
    }

    if (*ppvObject).is_null() {
        WARN!(
            b"() : asking for unsupported interface %s\n\0".as_ptr() as PCHAR,
            debugstr_guid(riid),
        );
        return E_NOINTERFACE;
    }

    IUnknown_AddRef(*ppvObject as *mut IUnknown);

    S_OK
}

unsafe fn DefaultHandler_NDIUnknown_AddRef(
    iface: *mut IUnknown
) -> ULONG {
    let This = impl_from_IUnknown(iface);
    InterlockedIncrement(&mut (*This).ref_) as ULONG
}

unsafe fn DefaultHandler_NDIUnknown_Release(
    iface: *mut IUnknown
) -> ULONG {
    let This = impl_from_IUnknown(iface);
    let ref_count: ULONG;

    ref_count = InterlockedDecrement(&mut (*This).ref_) as ULONG;

    if ref_count == 0 {
        DefaultHandler_Destroy(This);
    }

    ref_count
}

unsafe fn DefaultHandler_QueryInterface(
    iface: *mut IOleObject,
    riid: *const IID,
    ppvObject: *mut LPVOID,
) -> HRESULT {
    let This = impl_from_IOleObject(iface);
    IUnknown_QueryInterface((*This).outerUnknown, riid, ppvObject)
}
unsafe fn DefaultHandler_AddRef(
    iface: *mut IOleObject
) -> ULONG {
    let This = impl_from_IOleObject(iface);
    IUnknown_AddRef((*This).outerUnknown)
}

unsafe fn DefaultHandler_Release(iface: *mut IOleObject) -> ULONG {
    let This = impl_from_IOleObject(iface);
    IUnknown_Release((*This).outerUnknown)
}
unsafe fn DefaultHandler_SetClientSite(
    iface: *mut IOleObject,
    pClientSite: *mut IOleClientSite,
) -> HRESULT {
    let This = impl_from_IOleObject(iface);
    let mut hr = S_OK;

    TRACE!(b"(%p, %p)\n\0".as_ptr() as *const i8, iface, pClientSite);

    if object_is_running(This) != 0 {
        start_object_call(This);
        hr = IOleObject_SetClientSite((*This).pOleDelegate, pClientSite);
        end_object_call(This);
    }

    if !(*This).clientSite.is_null() {
        IOleClientSite_Release((*This).clientSite);
    }

    (*This).clientSite = pClientSite;

    if !(*This).clientSite.is_null() {
        IOleClientSite_AddRef((*This).clientSite);
    }

    hr
}

unsafe fn DefaultHandler_GetClientSite(
    iface: *mut IOleObject,
    ppClientSite: *mut *mut IOleClientSite
) -> HRESULT {
    let This = impl_from_IOleObject(iface);

    if ppClientSite.is_null() {
        return E_POINTER;
    }

    *ppClientSite = (*This).clientSite;

    if !(*This).clientSite.is_null() {
        IOleClientSite_AddRef((*This).clientSite);
    }

    S_OK
}
unsafe fn DefaultHandler_SetHostNames(
    iface: *mut IOleObject,
    szContainerApp: *const u16,
    szContainerObj: *const u16,
) -> HRESULT {
    let This = impl_from_IOleObject(iface);

    TRACE!(
        b"(%p, %s, %s)\n\0".as_ptr() as *const i8,
        iface,
        debugstr_w(szContainerApp),
        debugstr_w(szContainerObj),
    );

    if object_is_running(This) != 0 {
        start_object_call(This);
        IOleObject_SetHostNames((*This).pOleDelegate, szContainerApp, szContainerObj);
        end_object_call(This);
    }

    HeapFree(GetProcessHeap(), 0, (*This).containerApp as LPVOID);
    (*This).containerApp = core::ptr::null_mut();
    HeapFree(GetProcessHeap(), 0, (*This).containerObj as LPVOID);
    (*This).containerObj = core::ptr::null_mut();

    if !szContainerApp.is_null() {
        let len = lstrlenW(szContainerApp) as usize;
        let ptr = HeapAlloc(
            GetProcessHeap(),
            0,
            ((len + 1) * core::mem::size_of::<u16>()) as SIZE_T,
        );
        if !ptr.is_null() {
            (*This).containerApp = ptr as *mut u16;
            lstrcpyW((*This).containerApp, szContainerApp);
        }
    }

    if !szContainerObj.is_null() {
        let len = lstrlenW(szContainerObj) as usize;
        let ptr = HeapAlloc(
            GetProcessHeap(),
            0,
            ((len + 1) * core::mem::size_of::<u16>()) as SIZE_T,
        );
        if !ptr.is_null() {
            (*This).containerObj = ptr as *mut u16;
            lstrcpyW((*This).containerObj, szContainerObj);
        }
    }

    S_OK
}

} // stdmethod!

unsafe fn release_delegates(This: *mut DefaultHandler) {
    if !(*This).pDataDelegate.is_null() {
        IDataObject_Release((*This).pDataDelegate);
        (*This).pDataDelegate = core::ptr::null_mut();
    }
    if !(*This).pPSDelegate.is_null() {
        IPersistStorage_Release((*This).pPSDelegate);
        (*This).pPSDelegate = core::ptr::null_mut();
    }
    if !(*This).pOleDelegate.is_null() {
        IOleObject_Release((*This).pOleDelegate);
        (*This).pOleDelegate = core::ptr::null_mut();
    }
}

unsafe fn DefaultHandler_Stop(This: *mut DefaultHandler) {
    let mut cache_ctrl: *mut IOleCacheControl = core::ptr::null_mut();
    let hr: HRESULT;

    if (*This).object_state == object_state_not_running {
        return;
    }

    hr = IUnknown_QueryInterface(
        (*This).dataCache,
        &IID_IOleCacheControl,
        &mut cache_ctrl as *mut *mut IOleCacheControl as *mut LPVOID,
    );
    if SUCCEEDED(hr) {
        IOleCacheControl_OnStop(cache_ctrl);
        IOleCacheControl_Release(cache_ctrl);
    }

    IOleObject_Unadvise((*This).pOleDelegate, (*This).dwAdvConn);

    if !(*This).dataAdviseHolder.is_null() {
        DataAdviseHolder_OnDisconnect((*This).dataAdviseHolder);
    }

    (*This).object_state = object_state_not_running;
    release_delegates(This);
}

stdmethod! {

unsafe fn DefaultHandler_Close(
    iface: *mut IOleObject,
    dwSaveOption: DWORD,
) -> HRESULT {
    let This = impl_from_IOleObject(iface);
    let hr: HRESULT;

    TRACE!(b"(%d)\n\0".as_ptr(), dwSaveOption);

    if object_is_running(This) == 0 {
        return S_OK;
    }

    start_object_call(This);
    hr = IOleObject_Close((*This).pOleDelegate, dwSaveOption);
    end_object_call(This);

    DefaultHandler_Stop(This);

    hr
}
unsafe fn DefaultHandler_SetMoniker(
    iface: *mut IOleObject,
    dwWhichMoniker: DWORD,
    pmk: *mut IMoniker,
) -> HRESULT {
    let This = impl_from_IOleObject(iface);
    let mut hr = S_OK;

    TRACE!(b"(%p, %d, %p)\n\0".as_ptr() as PCHAR, iface, dwWhichMoniker, pmk);

    if object_is_running(This) != 0 {
        start_object_call(This);
        hr = IOleObject_SetMoniker((*This).pOleDelegate, dwWhichMoniker, pmk);
        end_object_call(This);
    }

    hr
}
unsafe fn DefaultHandler_GetMoniker(
    iface: *mut IOleObject,
    dwAssign: DWORD,
    dwWhichMoniker: DWORD,
    ppmk: *mut *mut IMoniker,
) -> HRESULT {
    let This = impl_from_IOleObject(iface);
    let hr: HRESULT;

    TRACE!(
        b"(%p, %d, %d, %p)\n\0".as_ptr() as PCHAR,
        iface,
        dwAssign,
        dwWhichMoniker,
        ppmk,
    );

    if object_is_running(This) != 0 {
        start_object_call(This);
        hr = IOleObject_GetMoniker(
            (*This).pOleDelegate,
            dwAssign,
            dwWhichMoniker,
            ppmk,
        );
        end_object_call(This);
        return hr;
    }

    if !(*This).clientSite.is_null() {
        return IOleClientSite_GetMoniker(
            (*This).clientSite,
            dwAssign,
            dwWhichMoniker,
            ppmk,
        );
    }

    return E_FAIL;
}

unsafe fn DefaultHandler_InitFromData(
    iface: *mut IOleObject,
    pDataObject: *mut IDataObject,
    fCreation: BOOL,
    dwReserved: DWORD,
) -> HRESULT {
    let This = impl_from_IOleObject(iface);
    let mut hr = OLE_E_NOTRUNNING;

    TRACE!(
        b"(%p, %p, %d, %d)\n\0".as_ptr(),
        iface,
        pDataObject,
        fCreation,
        dwReserved,
    );

    if object_is_running(This) != 0 {
        start_object_call(This);
        hr = IOleObject_InitFromData((*This).pOleDelegate, pDataObject, fCreation, dwReserved);
        end_object_call(This);
    }

    hr
}

unsafe fn DefaultHandler_GetClipboardData(
    iface: *mut IOleObject,
    dwReserved: DWORD,
    ppDataObject: *mut *mut IDataObject,
) -> HRESULT {
    let This = impl_from_IOleObject(iface);
    let mut hr: HRESULT = OLE_E_NOTRUNNING;

    TRACE!(
        b"(%p, %d, %p)\n\0".as_ptr(),
        iface,
        dwReserved,
        ppDataObject,
    );

    if object_is_running(This) != 0 {
        start_object_call(This);
        hr = IOleObject_GetClipboardData(
            (*This).pOleDelegate,
            dwReserved,
            ppDataObject,
        );
        end_object_call(This);
    }

    hr
}

unsafe fn DefaultHandler_DoVerb(
    iface: *mut IOleObject,
    iVerb: LONG,
    lpmsg: *mut tagMSG,
    pActiveSite: *mut IOleClientSite,
    lindex: LONG,
    hwndParent: HWND,
    lprcPosRect: LPCRECT,
) -> HRESULT {

    let This = impl_from_IOleObject(iface);
    let pRunnableObj = core::ptr::addr_of_mut!((*This).IRunnableObject_iface);
    let mut hr: HRESULT;

    TRACE!(
        b"(%d, %p, %p, %d, %p, %s)\n\0".as_ptr(),
        iVerb,
        lpmsg,
        pActiveSite,
        lindex,
        hwndParent,
        wine_dbgstr_rect(lprcPosRect),
    );

    hr = IRunnableObject_Run(pRunnableObj, core::ptr::null_mut());
    if FAILED(hr) {
        return hr;
    }

    start_object_call(This);
    hr = IOleObject_DoVerb(
        (*This).pOleDelegate,
        iVerb,
        lpmsg,
        pActiveSite,
        lindex,
        hwndParent,
        lprcPosRect,
    );
    end_object_call(This);

    hr
}

unsafe fn DefaultHandler_EnumVerbs(
    iface: *mut IOleObject,
    ppEnumOleVerb: *mut *mut IEnumOLEVERB,
) -> HRESULT {
    let This = impl_from_IOleObject(iface);
    let mut hr = OLE_S_USEREG;

    TRACE!(b"(%p, %p)\n\0".as_ptr() as PCHAR, iface, ppEnumOleVerb);

    if object_is_running(This) != 0 {
        start_object_call(This);
        hr = IOleObject_EnumVerbs((*This).pOleDelegate, ppEnumOleVerb);
        end_object_call(This);
    }

    if hr == OLE_S_USEREG {
        return OleRegEnumVerbs(core::ptr::addr_of_mut!((*This).clsid), ppEnumOleVerb);
    } else {
        return hr;
    }
}

unsafe fn DefaultHandler_Update(
    iface: *mut IOleObject
) -> HRESULT {
    let This: *mut DefaultHandler = impl_from_IOleObject(iface);
    let hr: HRESULT;

    TRACE!(b"(%p)\n\0".as_ptr() as *const i8, iface);

    if object_is_running(This) == 0 {
        FIXME!(b"Should run object\n\0".as_ptr() as *const i8);
        return E_NOTIMPL;
    }

    start_object_call(This);
    hr = IOleObject_Update((*This).pOleDelegate);
    end_object_call(This);

    hr
}

unsafe fn DefaultHandler_IsUpToDate(
    iface: *mut IOleObject
) -> HRESULT {
    let This = impl_from_IOleObject(iface);
    let mut hr: HRESULT = OLE_E_NOTRUNNING;

    TRACE!(b"(%p)\n\0".as_ptr(), iface);

    if object_is_running(This) != 0 {
        start_object_call(This);
        hr = IOleObject_IsUpToDate((*This).pOleDelegate);
        end_object_call(This);
    }
    
    hr
}
unsafe fn DefaultHandler_GetUserClassID(
    iface: *mut IOleObject,
    pClsid: *mut CLSID,
) -> HRESULT {
    let This = impl_from_IOleObject(iface);
    let hr: HRESULT;

    TRACE!(b"(%p, %p)\n\0".as_ptr(), iface, pClsid);

    if object_is_running(This) != 0 {
        start_object_call(This);
        hr = IOleObject_GetUserClassID((*This).pOleDelegate, pClsid);
        end_object_call(This);
        return hr;
    }

    if pClsid.is_null() {
        return E_POINTER;
    }

    *pClsid = (*This).clsid;

    return S_OK;
}

unsafe fn DefaultHandler_GetUserType(
    iface: *mut IOleObject,
    dwFormOfType: u32,
    pszUserType: *mut *mut u16,
) -> HRESULT {

    let This = impl_from_IOleObject(iface);
    let hr: HRESULT;

    TRACE!(
        b"(%p, %d, %p)\n\0".as_ptr(),
        iface,
        dwFormOfType,
        pszUserType,
    );

    if object_is_running(This) != 0 {
        start_object_call(This);
        hr = IOleObject_GetUserType((*This).pOleDelegate, dwFormOfType, pszUserType);
        end_object_call(This);
        return hr;
    }

    OleRegGetUserType(
        core::ptr::addr_of!((*This).clsid),
        dwFormOfType,
        pszUserType,
    )
}

unsafe fn DefaultHandler_SetExtent(
    iface: *mut IOleObject,
    dwDrawAspect: DWORD,
    psizel: *mut SIZEL,
) -> HRESULT {
    let This = impl_from_IOleObject(iface);
    let mut hr = OLE_E_NOTRUNNING;

    TRACE!(
        b"(%p, %x, (%d x %d))\n\0".as_ptr() as PCHAR,
        iface,
        dwDrawAspect,
        (*psizel).cx,
        (*psizel).cy,
    );

    if object_is_running(This) != 0 {
        start_object_call(This);
        hr = IOleObject_SetExtent((*This).pOleDelegate, dwDrawAspect, psizel);
        end_object_call(This);
    }

    hr
}
unsafe fn DefaultHandler_GetExtent(
    iface: *mut IOleObject,
    dwDrawAspect: DWORD,
    psizel: *mut SIZEL
) -> HRESULT {
    let targetDevice: *mut DVTARGETDEVICE;
    let mut cacheView: *mut IViewObject2 = core::ptr::null_mut();

    let This = impl_from_IOleObject(iface);

    TRACE!(b"(%p, %x, %p)\n\0".as_ptr() as PCHAR, iface, dwDrawAspect, psizel);

    if object_is_running(This) != 0 {
        start_object_call(This);
        let hres = IOleObject_GetExtent((*This).pOleDelegate, dwDrawAspect, psizel);
        end_object_call(This);
        return hres;
    }

    let hres = IUnknown_QueryInterface(
        (*This).dataCache,
        &IID_IViewObject2,
        &mut cacheView as *mut *mut IViewObject2 as *mut LPVOID
    );
    if FAILED(hres) {
        return E_UNEXPECTED;
    }

    targetDevice = core::ptr::null_mut();

    let hres = IViewObject2_GetExtent(
        cacheView,
        dwDrawAspect,
        -1i32,
        targetDevice,
        psizel
    );

    IViewObject2_Release(cacheView);

    hres
}

unsafe fn DefaultHandler_Advise(
    iface: *mut IOleObject,
    pAdvSink: *mut IAdviseSink,
    pdwConnection: *mut DWORD,
) -> HRESULT {
    let mut hres: HRESULT = S_OK;
    let This = impl_from_IOleObject(iface);

    TRACE!(b"(%p, %p, %p)\n\0".as_ptr() as *const i8, iface, pAdvSink, pdwConnection);

    if (*This).oleAdviseHolder.is_null() {
        hres = CreateOleAdviseHolder(core::ptr::addr_of_mut!((*This).oleAdviseHolder));
    }

    if SUCCEEDED(hres) {
        hres = IOleAdviseHolder_Advise((*This).oleAdviseHolder, pAdvSink, pdwConnection);
    }

    hres
}

unsafe fn DefaultHandler_Unadvise(
    iface: *mut IOleObject,
    dwConnection: DWORD,
) -> HRESULT {
    let This = impl_from_IOleObject(iface);

    TRACE!(b"(%p, %d)\n\0".as_ptr() as *const i8, iface, dwConnection);

    if (*This).oleAdviseHolder.is_null() {
        return OLE_E_NOCONNECTION;
    }

    IOleAdviseHolder_Unadvise((*This).oleAdviseHolder, dwConnection)
}
unsafe fn DefaultHandler_EnumAdvise(
    iface: *mut IOleObject,
    ppenumAdvise: *mut *mut IEnumSTATDATA,
) -> HRESULT {
    let This = impl_from_IOleObject(iface);

    TRACE!(b"(%p, %p)\n\0".as_ptr() as *const i8, iface, ppenumAdvise);

    if ppenumAdvise.is_null() {
        return E_POINTER;
    }

    *ppenumAdvise = core::ptr::null_mut();

    if (*This).oleAdviseHolder.is_null() {
        return S_OK;
    }

    IOleAdviseHolder_EnumAdvise((*This).oleAdviseHolder, ppenumAdvise)
}
unsafe fn DefaultHandler_GetMiscStatus(
    iface: *mut IOleObject,
    dwAspect: u32,
    pdwStatus: *mut u32,
) -> HRESULT {

    let This: *mut DefaultHandler = impl_from_IOleObject(iface);

    TRACE!(
        b"(%p, %x, %p)\n\0".as_ptr(),
        iface,
        dwAspect,
        pdwStatus,
    );

    if object_is_running(This) != 0 {
        start_object_call(This);
        let hres: HRESULT = IOleObject_GetMiscStatus((*This).pOleDelegate, dwAspect, pdwStatus);
        end_object_call(This);
        return hres;
    }

    let hres: HRESULT = OleRegGetMiscStatus(
        core::ptr::addr_of_mut!((*This).clsid),
        dwAspect,
        pdwStatus,
    );

    if FAILED(hres) {
        *pdwStatus = 0;
    }

    hres
}
unsafe fn DefaultHandler_SetColorScheme(
    iface: *mut IOleObject,
    pLogpal: *mut tagLOGPALETTE,
) -> HRESULT {
    let This = impl_from_IOleObject(iface);
    let mut hr = OLE_E_NOTRUNNING;

    TRACE!(b"(%p, %p))\n\0".as_ptr() as *const i8, iface, pLogpal);

    if object_is_running(This) != 0 {
        start_object_call(This);
        hr = IOleObject_SetColorScheme((*This).pOleDelegate, pLogpal);
        end_object_call(This);
    }

    hr
}
unsafe fn DefaultHandler_IDataObject_QueryInterface(
    iface: *mut IDataObject,
    riid: *const CLSID,
    ppvObject: *mut LPVOID,
) -> HRESULT {
    let This = impl_from_IDataObject(iface);
    
    IUnknown_QueryInterface((*This).outerUnknown, riid, ppvObject)
}
unsafe fn DefaultHandler_IDataObject_AddRef(
    iface: *mut IDataObject
) -> ULONG {
    let This = impl_from_IDataObject(iface);
    IUnknown_AddRef((*This).outerUnknown)
}

unsafe fn DefaultHandler_IDataObject_Release(
    iface: *mut IDataObject
) -> ULONG {
    let This = impl_from_IDataObject(iface);
    
    IUnknown_Release((*This).outerUnknown)
}
unsafe fn DefaultHandler_GetData(
    iface: *mut IDataObject,
    pformatetcIn: LPFORMATETC,
    pmedium: *mut STGMEDIUM,
) -> HRESULT {
    let mut cacheDataObject: *mut IDataObject = core::ptr::null_mut();
    let mut hres: HRESULT;

    let This = impl_from_IDataObject(iface);

    TRACE!(
        b"(%p, %p, %p)\n\0".as_ptr() as PCHAR,
        iface,
        pformatetcIn,
        pmedium,
    );

    hres = IUnknown_QueryInterface(
        (*This).dataCache,
        &IID_IDataObject,
        &mut cacheDataObject as *mut *mut IDataObject as *mut LPVOID,
    );

    if FAILED(hres) {
        return E_UNEXPECTED;
    }

    hres = IDataObject_GetData(cacheDataObject, pformatetcIn, pmedium);

    IDataObject_Release(cacheDataObject);

    if hres == S_OK {
        return hres;
    }

    if object_is_running(This) != 0 {
        start_object_call(This);
        hres = IDataObject_GetData((*This).pDataDelegate, pformatetcIn, pmedium);
        end_object_call(This);
        if hres == S_OK {
            return hres;
        }
    }

    if object_is_running(This) == 0 {
        hres = OLE_E_NOTRUNNING;
    }

    hres
}

unsafe fn DefaultHandler_GetDataHere(
    iface: *mut IDataObject,
    pformatetc: LPFORMATETC,
    pmedium: *mut STGMEDIUM,
) -> HRESULT {
    FIXME!(b": Stub\n\0".as_ptr() as *const i8);
    E_NOTIMPL
}
unsafe fn DefaultHandler_QueryGetData(
    iface: *mut IDataObject,
    pformatetc: LPFORMATETC,
) -> HRESULT {
    let mut cacheDataObject: *mut IDataObject = core::ptr::null_mut();
    let mut hres: HRESULT;

    let This = impl_from_IDataObject(iface);

    TRACE!(b"(%p, %p)\n\0".as_ptr() as *const i8, iface, pformatetc);

    hres = IUnknown_QueryInterface(
        (*This).dataCache,
        &IID_IDataObject,
        &mut cacheDataObject as *mut *mut IDataObject as *mut LPVOID,
    );

    if FAILED(hres) {
        return E_UNEXPECTED;
    }

    hres = IDataObject_QueryGetData(cacheDataObject, pformatetc);

    IDataObject_Release(cacheDataObject);

    if hres == S_OK {
        return hres;
    }

    if object_is_running(This) != 0 {
        start_object_call(This);
        hres = IDataObject_QueryGetData((*This).pDataDelegate, pformatetc);
        end_object_call(This);
        if hres == S_OK {
            return hres;
        }
    }

    if object_is_running(This) == 0 {
        hres = OLE_E_NOTRUNNING;
    }

    hres
}
unsafe fn DefaultHandler_GetCanonicalFormatEtc(
    iface: *mut IDataObject,
    pformatetcIn: LPFORMATETC,
    pformatetcOut: LPFORMATETC,
) -> HRESULT {
    let This = impl_from_IDataObject(iface);
    let hr: HRESULT;

    TRACE!(b"(%p, %p, %p)\n\0".as_ptr() as PCHAR, iface, pformatetcIn, pformatetcOut);

    if object_is_running(This) == 0 {
        return OLE_E_NOTRUNNING;
    }

    start_object_call(This);
    hr = IDataObject_GetCanonicalFormatEtc((*This).pDataDelegate, pformatetcIn, pformatetcOut);
    end_object_call(This);

    hr
}
unsafe fn DefaultHandler_SetData(
    iface: *mut IDataObject,
    pformatetc: LPFORMATETC,
    pmedium: *mut STGMEDIUM,
    fRelease: BOOL,
) -> HRESULT {
    let This = impl_from_IDataObject(iface);
    let mut cacheDataObject: *mut IDataObject = core::ptr::null_mut();
    let mut hres: HRESULT;

    TRACE!(
        b"(%p, %p, %p, %d)\n\0".as_ptr() as *const i8,
        iface,
        pformatetc,
        pmedium,
        fRelease,
    );

    hres = IUnknown_QueryInterface(
        (*This).dataCache,
        &IID_IDataObject,
        &mut cacheDataObject as *mut *mut IDataObject as *mut LPVOID,
    );

    if FAILED(hres) {
        return E_UNEXPECTED;
    }

    hres = IDataObject_SetData(cacheDataObject, pformatetc, pmedium, fRelease);

    IDataObject_Release(cacheDataObject);

    return hres;
}

unsafe fn DefaultHandler_EnumFormatEtc(
    iface: *mut IDataObject,
    dwDirection: DWORD,
    ppenumFormatEtc: *mut *mut IEnumFORMATETC,
) -> HRESULT {
    let This = impl_from_IDataObject(iface);

    TRACE!(
        b"(%p, %x, %p)\n\0".as_ptr() as PCHAR,
        iface,
        dwDirection,
        ppenumFormatEtc,
    );

    OleRegEnumFormatEtc(
        core::ptr::addr_of_mut!((*This).clsid),
        dwDirection,
        ppenumFormatEtc,
    )
}

unsafe fn DefaultHandler_DAdvise(
    iface: *mut IDataObject,
    pformatetc: *mut FORMATETC,
    advf: DWORD,
    pAdvSink: *mut IAdviseSink,
    pdwConnection: *mut DWORD,
) -> HRESULT {
    let mut hres: HRESULT = S_OK;
    let This: *mut DefaultHandler = impl_from_IDataObject(iface);

    TRACE!(
        b"(%p, %p, %d, %p, %p)\n\0".as_ptr() as PCSTR,
        iface,
        pformatetc,
        advf,
        pAdvSink,
        pdwConnection,
    );

    if (*This).dataAdviseHolder.is_null() {
        hres = CreateDataAdviseHolder(core::ptr::addr_of_mut!((*This).dataAdviseHolder));
        if SUCCEEDED(hres) && object_is_running(This) != 0 {
            start_object_call(This);
            DataAdviseHolder_OnConnect((*This).dataAdviseHolder, (*This).pDataDelegate);
            end_object_call(This);
        }
    }

    if SUCCEEDED(hres) {
        hres = IDataAdviseHolder_Advise(
            (*This).dataAdviseHolder,
            iface,
            pformatetc,
            advf,
            pAdvSink,
            pdwConnection,
        );
    }

    hres
}

unsafe fn DefaultHandler_DUnadvise(
    iface: *mut IDataObject,
    dwConnection: DWORD,
) -> HRESULT {
    let This = impl_from_IDataObject(iface);

    TRACE!(b"(%p, %d)\n\0".as_ptr(), iface, dwConnection);

    if (*This).dataAdviseHolder.is_null() {
        return OLE_E_NOCONNECTION;
    }

    IDataAdviseHolder_Unadvise((*This).dataAdviseHolder, dwConnection)
}
unsafe fn DefaultHandler_EnumDAdvise(
    iface: *mut IDataObject,
    ppenumAdvise: *mut *mut IEnumSTATDATA,
) -> HRESULT {

    let This = impl_from_IDataObject(iface);

    TRACE!(b"(%p, %p)\n\0".as_ptr() as PCHAR, iface, ppenumAdvise);

    if ppenumAdvise.is_null() {
        return E_POINTER;
    }

    *ppenumAdvise = core::ptr::null_mut();

    if !(*This).dataAdviseHolder.is_null() {
        return IDataAdviseHolder_EnumAdvise((*This).dataAdviseHolder, ppenumAdvise);
    }

    S_OK
}
unsafe fn DefaultHandler_IRunnableObject_QueryInterface(
    iface: *mut IRunnableObject,
    riid: *const GUID,
    ppvObject: *mut LPVOID,
) -> HRESULT {
    let This = impl_from_IRunnableObject(iface);
    IUnknown_QueryInterface((*This).outerUnknown, riid, ppvObject)
}

unsafe fn DefaultHandler_IRunnableObject_AddRef(
    iface: *mut IRunnableObject
) -> ULONG {
    let This = impl_from_IRunnableObject(iface);
    IUnknown_AddRef((*This).outerUnknown)
}

unsafe fn DefaultHandler_IRunnableObject_Release(
    iface: *mut IRunnableObject
) -> ULONG {
    let This = impl_from_IRunnableObject(iface);
    IUnknown_Release((*This).outerUnknown)
}

unsafe fn DefaultHandler_GetRunningClass(
    iface: *mut IRunnableObject,
    lpClsid: LPCLSID,
) -> HRESULT {
    FIXME!(b"()\n\0".as_ptr());
    S_OK
}

unsafe fn DefaultHandler_Run(
    iface: *mut IRunnableObject,
    pbc: *mut IBindCtx,
) -> HRESULT {
    let This = impl_from_IRunnableObject(iface);
    let mut hr: HRESULT;
    let mut cache_ctrl: *mut IOleCacheControl = core::ptr::null_mut();

    FIXME!(b"(%p): semi-stub\n\0".as_ptr() as *const i8, pbc);

    if object_is_running(This) != 0 {
        return S_OK;
    }

    release_delegates(This);

    hr = CoCreateInstance(
        core::ptr::addr_of!((*This).clsid),
        core::ptr::null_mut(),
        CLSCTX_LOCAL_SERVER | CLSCTX_REMOTE_SERVER,
        &IID_IOleObject,
        core::ptr::addr_of_mut!((*This).pOleDelegate) as *mut LPVOID,
    );
    if FAILED(hr) {
        return hr;
    }

    hr = IOleObject_Advise(
        (*This).pOleDelegate,
        core::ptr::addr_of_mut!((*This).IAdviseSink_iface),
        core::ptr::addr_of_mut!((*This).dwAdvConn),
    );
    if FAILED(hr) {
        DefaultHandler_Stop(This);
        return hr;
    }

    if !(*This).clientSite.is_null() {
        hr = IOleObject_SetClientSite((*This).pOleDelegate, (*This).clientSite);
        if FAILED(hr) {
            DefaultHandler_Stop(This);
            return hr;
        }
    }

    hr = IOleObject_QueryInterface(
        (*This).pOleDelegate,
        &IID_IPersistStorage,
        core::ptr::addr_of_mut!((*This).pPSDelegate) as *mut LPVOID,
    );
    if FAILED(hr) {
        DefaultHandler_Stop(This);
        return hr;
    }

    if (*This).storage_state == storage_state_initialised {
        hr = IPersistStorage_InitNew((*This).pPSDelegate, (*This).storage);
    } else if (*This).storage_state == storage_state_loaded {
        hr = IPersistStorage_Load((*This).pPSDelegate, (*This).storage);
    }
    if FAILED(hr) {
        DefaultHandler_Stop(This);
        return hr;
    }

    if !(*This).containerApp.is_null() {
        hr = IOleObject_SetHostNames(
            (*This).pOleDelegate,
            (*This).containerApp,
            (*This).containerObj,
        );
        if FAILED(hr) {
            DefaultHandler_Stop(This);
            return hr;
        }
    }

    hr = IOleObject_QueryInterface(
        (*This).pOleDelegate,
        &IID_IDataObject,
        core::ptr::addr_of_mut!((*This).pDataDelegate) as *mut LPVOID,
    );
    if FAILED(hr) {
        DefaultHandler_Stop(This);
        return hr;
    }

    (*This).object_state = object_state_running;

    if !(*This).dataAdviseHolder.is_null() {
        hr = DataAdviseHolder_OnConnect((*This).dataAdviseHolder, (*This).pDataDelegate);
        if FAILED(hr) {
            DefaultHandler_Stop(This);
            return hr;
        }
    }

    hr = IUnknown_QueryInterface(
        (*This).dataCache,
        &IID_IOleCacheControl,
        core::ptr::addr_of_mut!(cache_ctrl) as *mut LPVOID,
    );
    if FAILED(hr) {
        DefaultHandler_Stop(This);
        return hr;
    }
    hr = IOleCacheControl_OnRun(cache_ctrl, (*This).pDataDelegate);
    IOleCacheControl_Release(cache_ctrl);
    if FAILED(hr) {
        DefaultHandler_Stop(This);
        return hr;
    }

    hr
}

unsafe fn DefaultHandler_IsRunning(
    iface: *mut IRunnableObject
) -> BOOL {
    let This = impl_from_IRunnableObject(iface);

    TRACE!(b"()\n\0".as_ptr());

    if (*This).object_state == object_state_running {
        1 // TRUE
    } else {
        0 // FALSE
    }
}

unsafe fn DefaultHandler_LockRunning(
    iface: *mut IRunnableObject,
    fLock: BOOL,
    fLastUnlockCloses: BOOL,
) -> HRESULT {
    FIXME!(b"()\n\0".as_ptr());
    S_OK
}

unsafe fn DefaultHandler_SetContainedObject(
    iface: *mut IRunnableObject,
    fContained: BOOL,
) -> HRESULT {
    FIXME!(b"()\n\0".as_ptr());
    S_OK
}

unsafe fn DefaultHandler_IAdviseSink_QueryInterface(
    iface: *mut IAdviseSink,
    riid: *const CLSID,
    ppvObject: *mut LPVOID,
) -> HRESULT {
    
    
    if IsEqualIID(riid, &IID_IUnknown) ||
       IsEqualIID(riid, &IID_IAdviseSink)
    {
        *ppvObject = iface as LPVOID;
        IAdviseSink_AddRef(iface);
        return S_OK;
    }

    E_NOINTERFACE
}
unsafe fn DefaultHandler_IAdviseSink_AddRef(
    iface: *mut IAdviseSink
) -> ULONG {
    let This = impl_from_IAdviseSink(iface);

    IUnknown_AddRef(&mut (*This).IUnknown_iface as *mut IUnknown)
}
unsafe fn DefaultHandler_IAdviseSink_Release(
    iface: *mut IAdviseSink
) -> ULONG {
    let This = impl_from_IAdviseSink(iface);
    
    IUnknown_Release(&mut (*This).IUnknown_iface as *mut _ as *mut IUnknown)
}
unsafe fn DefaultHandler_IAdviseSink_OnDataChange(
    iface: *mut IAdviseSink,
    pFormatetc: *mut FORMATETC,
    pStgmed: *mut STGMEDIUM,
) {
    FIXME!(b": stub\n\0".as_ptr() as *const i8);
}
unsafe fn DefaultHandler_IAdviseSink_OnViewChange(
    iface: *mut IAdviseSink,
    dwAspect: DWORD,
    lindex: LONG,
) {
    FIXME!(b": stub\n\0".as_ptr() as *const i8);
}
unsafe fn DefaultHandler_IAdviseSink_OnRename(
    iface: *mut IAdviseSink,
    pmk: *mut IMoniker,
) {
    let This = impl_from_IAdviseSink(iface);

    TRACE!(b"(%p)\n\0".as_ptr() as PCHAR, pmk);

    if !(*This).oleAdviseHolder.is_null() {
        IOleAdviseHolder_SendOnRename((*This).oleAdviseHolder, pmk);
    }
}
unsafe fn DefaultHandler_IAdviseSink_OnSave(
    iface: *mut IAdviseSink) {
    let This = impl_from_IAdviseSink(iface);

    TRACE!(b"()\n\0".as_ptr() as *const i8);

    if !(*This).oleAdviseHolder.is_null() {
        IOleAdviseHolder_SendOnSave((*This).oleAdviseHolder);
    }
}
unsafe fn DefaultHandler_IAdviseSink_OnClose(
    iface: *mut IAdviseSink) {
    let This = impl_from_IAdviseSink(iface);
    
    TRACE!(b"()\n\0".as_ptr() as PCHAR);

    if !(*This).oleAdviseHolder.is_null() {
        IOleAdviseHolder_SendOnClose((*This).oleAdviseHolder);
    }

    if (*This).in_call == 0 {
        DefaultHandler_Stop(This);
    }
    else
    {
        TRACE!(b"OnClose during call.  Deferring shutdown\n\0".as_ptr() as PCHAR);
        (*This).object_state = object_state_deferred_close;
    }
}
unsafe fn DefaultHandler_IPersistStorage_QueryInterface(
    iface: *mut IPersistStorage,
    riid: *const CLSID,
    ppvObject: *mut LPVOID,
) -> HRESULT {
    let This = impl_from_IPersistStorage(iface);

    IUnknown_QueryInterface((*This).outerUnknown, riid, ppvObject)
}
unsafe fn DefaultHandler_IPersistStorage_AddRef(
    iface: *mut IPersistStorage
) -> ULONG {
    let This = impl_from_IPersistStorage(iface);
    
    IUnknown_AddRef((*This).outerUnknown)
}
unsafe fn DefaultHandler_IPersistStorage_Release(
    iface: *mut IPersistStorage
) -> ULONG {
    let This = impl_from_IPersistStorage(iface);
    IUnknown_Release((*This).outerUnknown)
}

unsafe fn DefaultHandler_IPersistStorage_GetClassID(
    iface: *mut IPersistStorage,
    clsid: *mut CLSID,
) -> HRESULT {
    let This = impl_from_IPersistStorage(iface);
    let hr: HRESULT;

    TRACE!(b"(%p)->(%p)\n\0".as_ptr(), iface, clsid);

    if object_is_running(This) != 0 {
        start_object_call(This);
        hr = IPersistStorage_GetClassID((*This).pPSDelegate, clsid);
        end_object_call(This);
    } else {
        hr = IPersistStorage_GetClassID((*This).dataCache_PersistStg, clsid);
    }

    hr
}

unsafe fn DefaultHandler_IPersistStorage_IsDirty(
    iface: *mut IPersistStorage
) -> HRESULT {
    let This = impl_from_IPersistStorage(iface);
    let mut hr: HRESULT;

    TRACE!(b"(%p)\n\0".as_ptr(), iface);

    hr = IPersistStorage_IsDirty((*This).dataCache_PersistStg);
    if hr != S_FALSE { return hr; }

    if object_is_running(This) != 0 {
        start_object_call(This);
        hr = IPersistStorage_IsDirty((*This).pPSDelegate);
        end_object_call(This);
    }

    hr
}

} // stdmethod!

unsafe fn load_ole_stream(_This: *mut DefaultHandler, storage: *mut IStorage) -> HRESULT {
    let mut stream: *mut IStream = core::ptr::null_mut();
    let mut hr: HRESULT;

    hr = IStorage_OpenStream(
        storage,
        OleStream.as_ptr(),
        core::ptr::null_mut(),
        STGM_READ | STGM_SHARE_EXCLUSIVE,
        0,
        &mut stream
    );

    if SUCCEEDED(hr) {
        let mut read: u32 = 0;
        let mut header: ole_stream_header_t = core::mem::zeroed();

        hr = IStream_Read(
            stream,
            &mut header as *mut _ as *mut core::ffi::c_void,
            core::mem::size_of::<ole_stream_header_t>() as u32,
            &mut read
        );
        
        if hr == S_OK && read == core::mem::size_of::<ole_stream_header_t>() as u32 && header.version == ole_stream_version {
            if (header.flags & 1) != 0 {
                FIXME!(b"Linked objects are not supported yet\n\0".as_ptr() as *const i8);
            }
        } else {
            WARN!(b"Incorrect OleStream header\n\0".as_ptr() as *const i8);
            hr = DV_E_CLIPFORMAT;
        }
        IStream_Release(stream);
    } else {
        hr = STORAGE_CreateOleStream(storage, 0);
    }

    hr
}

stdmethod! {

unsafe fn DefaultHandler_IPersistStorage_InitNew(
    iface: *mut IPersistStorage,
    pStg: *mut IStorage,
) -> HRESULT {
    let This = impl_from_IPersistStorage(iface);
    let mut hr: HRESULT;

    TRACE!(b"(%p)->(%p)\n\0".as_ptr() as *const i8, iface, pStg);
    hr = STORAGE_CreateOleStream(pStg, 0);
    if hr != S_OK {
        return hr;
    }

    hr = IPersistStorage_InitNew((*This).dataCache_PersistStg, pStg);

    if SUCCEEDED(hr) && object_is_running(This) != 0 {
        start_object_call(This);
        hr = IPersistStorage_InitNew((*This).pPSDelegate, pStg);
        end_object_call(This);
    }

    if SUCCEEDED(hr) {
        IStorage_AddRef(pStg);
        (*This).storage = pStg;
        (*This).storage_state = storage_state_initialised;
    }

    hr
}

unsafe fn DefaultHandler_IPersistStorage_Load(
    iface: *mut IPersistStorage,
    pStg: *mut IStorage,
) -> HRESULT {
    let This = impl_from_IPersistStorage(iface);
    let mut hr: HRESULT;

    TRACE!(b"(%p)->(%p)\n\0".as_ptr() as PCHAR, iface, pStg);

    hr = load_ole_stream(This, pStg);

    if SUCCEEDED(hr) {
        hr = IPersistStorage_Load((*This).dataCache_PersistStg, pStg);
    }

    if SUCCEEDED(hr) && object_is_running(This) != 0 {
        start_object_call(This);
        hr = IPersistStorage_Load((*This).pPSDelegate, pStg);
        end_object_call(This);
    }

    if SUCCEEDED(hr) {
        IStorage_AddRef(pStg);
        (*This).storage = pStg;
        (*This).storage_state = storage_state_loaded;
    }
    hr
}

unsafe fn DefaultHandler_IPersistStorage_Save(
    iface: *mut IPersistStorage,
    pStgSave: *mut IStorage,
    fSameAsLoad: BOOL,
) -> HRESULT {
    let This = impl_from_IPersistStorage(iface);
    let mut hr: HRESULT;

    TRACE!(b"(%p)->(%p, %d)\n\0".as_ptr() as PCHAR, iface, pStgSave, fSameAsLoad);

    hr = IPersistStorage_Save((*This).dataCache_PersistStg, pStgSave, fSameAsLoad);
    if SUCCEEDED(hr) && object_is_running(This) != 0 {
        start_object_call(This);
        hr = IPersistStorage_Save((*This).pPSDelegate, pStgSave, fSameAsLoad);
        end_object_call(This);
    }

    return hr;
}

unsafe fn DefaultHandler_IPersistStorage_SaveCompleted(
    iface: *mut IPersistStorage,
    pStgNew: *mut IStorage,
) -> HRESULT {
    let This = impl_from_IPersistStorage(iface);
    let mut hr: HRESULT;

    TRACE!(b"(%p)->(%p)\n\0".as_ptr() as *const i8, iface, pStgNew);

    hr = IPersistStorage_SaveCompleted((*This).dataCache_PersistStg, pStgNew);

    if SUCCEEDED(hr) && object_is_running(This) != 0 {
        start_object_call(This);
        hr = IPersistStorage_SaveCompleted((*This).pPSDelegate, pStgNew);
        end_object_call(This);
    }

    if !pStgNew.is_null() {
        IStorage_AddRef(pStgNew);
        if !(*This).storage.is_null() {
            IStorage_Release((*This).storage);
        }
        (*This).storage = pStgNew;
        (*This).storage_state = storage_state_loaded;
    }

    hr
}
unsafe fn DefaultHandler_IPersistStorage_HandsOffStorage(
    iface: *mut IPersistStorage
) -> HRESULT {
    let This = impl_from_IPersistStorage(iface);
    let mut hr: HRESULT;

    TRACE!(b"(%p)\n\0".as_ptr() as *const i8, iface);

    hr = IPersistStorage_HandsOffStorage((*This).dataCache_PersistStg);

    if SUCCEEDED(hr) && object_is_running(This) != 0 {
        start_object_call(This);
        hr = IPersistStorage_HandsOffStorage((*This).pPSDelegate);
        end_object_call(This);
    }

    if !(*This).storage.is_null() {
        IStorage_Release((*This).storage);
    }
    (*This).storage = core::ptr::null_mut();
    (*This).storage_state = storage_state_uninitialised;

    hr
}

} // stdmethod!

unsafe fn DefaultHandler_Construct(
    clsid: REFCLSID,
    pUnkOuter: *mut IUnknown,
    flags: DWORD,
    pCF: *mut IClassFactory,
) -> *mut DefaultHandler {
    let This: *mut DefaultHandler;
    let mut hr: HRESULT;

    This = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<DefaultHandler>() as SIZE_T,
    ) as *mut DefaultHandler;

    if This.is_null() {
        return This;
    }

    (*This).IOleObject_iface.lpVtbl =
        &DefaultHandler_IOleObject_VTable as *const IOleObjectVtbl as *mut IOleObjectVtbl;
    (*This).IUnknown_iface.lpVtbl =
        &DefaultHandler_NDIUnknown_VTable as *const IUnknownVtbl as *mut IUnknownVtbl;
    (*This).IDataObject_iface.lpVtbl =
        &DefaultHandler_IDataObject_VTable as *const IDataObjectVtbl as *mut IDataObjectVtbl;
    (*This).IRunnableObject_iface.lpVtbl = &DefaultHandler_IRunnableObject_VTable
        as *const IRunnableObjectVtbl
        as *mut IRunnableObjectVtbl;
    (*This).IAdviseSink_iface.lpVtbl =
        &DefaultHandler_IAdviseSink_VTable as *const IAdviseSinkVtbl as *mut IAdviseSinkVtbl;
    (*This).IPersistStorage_iface.lpVtbl = &DefaultHandler_IPersistStorage_VTable
        as *const IPersistStorageVtbl
        as *mut IPersistStorageVtbl;

    (*This).inproc_server = ((flags & EMBDHLP_INPROC_SERVER) != 0) as BOOL;

    (*This).ref_ = 1;

    let mut pUnkOuter = pUnkOuter;
    if pUnkOuter.is_null() {
        pUnkOuter = &mut (*This).IUnknown_iface as *mut IUnknown;
    }

    (*This).outerUnknown = pUnkOuter;

    hr = CreateDataCache(
        (*This).outerUnknown,
        clsid,
        &IID_IUnknown,
        &mut (*This).dataCache as *mut *mut IUnknown as *mut LPVOID,
    );
    if SUCCEEDED(hr) {
        hr = IUnknown_QueryInterface(
            (*This).dataCache,
            &IID_IPersistStorage,
            &mut (*This).dataCache_PersistStg as *mut *mut IPersistStorage as *mut LPVOID,
        );
        if SUCCEEDED(hr) {
            IUnknown_Release((*This).outerUnknown);
        } else {
            IUnknown_Release((*This).dataCache);
        }
    }
    if FAILED(hr) {
        ERR!(b"Unexpected error creating data cache\n\0".as_ptr() as PCHAR);
        HeapFree(GetProcessHeap(), 0, This as LPVOID);
        return core::ptr::null_mut();
    }

    (*This).clsid = *clsid;
    (*This).clientSite = core::ptr::null_mut();
    (*This).oleAdviseHolder = core::ptr::null_mut();
    (*This).dataAdviseHolder = core::ptr::null_mut();
    (*This).containerApp = core::ptr::null_mut();
    (*This).containerObj = core::ptr::null_mut();
    (*This).pOleDelegate = core::ptr::null_mut();
    (*This).pPSDelegate = core::ptr::null_mut();
    (*This).pDataDelegate = core::ptr::null_mut();
    (*This).object_state = object_state_not_running;
    (*This).in_call = 0;

    (*This).dwAdvConn = 0;
    (*This).storage = core::ptr::null_mut();
    (*This).storage_state = storage_state_uninitialised;

    if (*This).inproc_server != 0 && (flags & EMBDHLP_DELAYCREATE) == 0 {
        let mut hr: HRESULT;
        (*This).pCFObject = core::ptr::null_mut();
        if !pCF.is_null() {
            hr = IClassFactory_CreateInstance(
                pCF,
                core::ptr::null_mut(),
                &IID_IOleObject,
                &mut (*This).pOleDelegate as *mut *mut IOleObject as *mut LPVOID,
            );
        } else {
            hr = CoCreateInstance(
                &(*This).clsid,
                core::ptr::null_mut(),
                CLSCTX_INPROC_SERVER,
                &IID_IOleObject,
                &mut (*This).pOleDelegate as *mut *mut IOleObject as *mut LPVOID,
            );
        }
        if SUCCEEDED(hr) {
            hr = IOleObject_QueryInterface(
                (*This).pOleDelegate,
                &IID_IPersistStorage,
                &mut (*This).pPSDelegate as *mut *mut IPersistStorage as *mut LPVOID,
            );
        }
        if SUCCEEDED(hr) {
            hr = IOleObject_QueryInterface(
                (*This).pOleDelegate,
                &IID_IDataObject,
                &mut (*This).pDataDelegate as *mut *mut IDataObject as *mut LPVOID,
            );
        }
        if SUCCEEDED(hr) {
            (*This).object_state = object_state_running;
        }
        if FAILED(hr) {
            WARN!(b"object creation failed with error %08x\n\0".as_ptr() as PCHAR, hr);
        }
    } else {
        (*This).pCFObject = pCF;
        if !pCF.is_null() {
            IClassFactory_AddRef(pCF);
        }
    }

    This
}

unsafe fn DefaultHandler_Destroy(This: *mut DefaultHandler) {

    TRACE!(b"(%p)\n\0".as_ptr(), This);

    (*This).ref_ = 10000;

    DefaultHandler_Stop(This);

    HeapFree(GetProcessHeap(), 0, (*This).containerApp as LPVOID);
    (*This).containerApp = core::ptr::null_mut();
    HeapFree(GetProcessHeap(), 0, (*This).containerObj as LPVOID);
    (*This).containerObj = core::ptr::null_mut();

    if !(*This).dataCache.is_null() {
        IUnknown_AddRef((*This).outerUnknown);
        IPersistStorage_Release((*This).dataCache_PersistStg);
        IUnknown_Release((*This).dataCache);
        (*This).dataCache_PersistStg = core::ptr::null_mut();
        (*This).dataCache = core::ptr::null_mut();
    }

    if !(*This).clientSite.is_null() {
        IOleClientSite_Release((*This).clientSite);
        (*This).clientSite = core::ptr::null_mut();
    }

    if !(*This).oleAdviseHolder.is_null() {
        IOleAdviseHolder_Release((*This).oleAdviseHolder);
        (*This).oleAdviseHolder = core::ptr::null_mut();
    }

    if !(*This).dataAdviseHolder.is_null() {
        IDataAdviseHolder_Release((*This).dataAdviseHolder);
        (*This).dataAdviseHolder = core::ptr::null_mut();
    }

    if !(*This).storage.is_null() {
        IStorage_Release((*This).storage);
        (*This).storage = core::ptr::null_mut();
    }

    if !(*This).pCFObject.is_null() {
        IClassFactory_Release((*This).pCFObject);
        (*This).pCFObject = core::ptr::null_mut();
    }

    HeapFree(GetProcessHeap(), 0, This as LPVOID);
}

#[no_mangle]
pub unsafe extern "system" fn OleCreateEmbeddingHelper(
    clsid: *const CLSID,
    pUnkOuter: *mut IUnknown,
    flags: DWORD,
    pCF: *mut IClassFactory,
    riid: *const IID,
    ppvObj: *mut LPVOID,
) -> HRESULT {

    let newHandler: *mut DefaultHandler;
    let hr: HRESULT;

    TRACE!(
        b"(%s, %p, %08x, %p, %s, %p)\n\0".as_ptr(),
        debugstr_guid(clsid),
        pUnkOuter,
        flags,
        pCF,
        debugstr_guid(riid),
        ppvObj,
    );

    if ppvObj.is_null() {
        return E_POINTER;
    }

    *ppvObj = core::ptr::null_mut();

    if !pUnkOuter.is_null() && !IsEqualIID(&IID_IUnknown, riid) {
        return CLASS_E_NOAGGREGATION;
    }

    newHandler = DefaultHandler_Construct(clsid, pUnkOuter, flags, pCF);

    if newHandler.is_null() {
        return E_OUTOFMEMORY;
    }

    hr = IUnknown_QueryInterface(
        core::ptr::addr_of_mut!((*newHandler).IUnknown_iface),
        riid,
        ppvObj,
    );

    IUnknown_Release(core::ptr::addr_of_mut!((*newHandler).IUnknown_iface));

    hr
}

#[no_mangle]
pub unsafe extern "system" fn OleCreateDefaultHandler(
    clsid: *const CLSID,
    pUnkOuter: *mut IUnknown,
    riid: *const IID,
    ppvObj: *mut LPVOID,
) -> HRESULT {
    TRACE!(
        b"(%s, %p, %s, %p)\n\0".as_ptr(),
        debugstr_guid(clsid),
        pUnkOuter,
        debugstr_guid(riid),
        ppvObj,
    );
    OleCreateEmbeddingHelper(
        clsid,
        pUnkOuter,
        EMBDHLP_INPROC_HANDLER | EMBDHLP_CREATENOW,
        core::ptr::null_mut(),
        riid,
        ppvObj,
    )
}

unsafe fn impl_from_IClassFactory(iface: *mut IClassFactory) -> *mut HandlerCF {
    CONTAINING_RECORD!(iface, HandlerCF, IClassFactory_iface)
}

stdmethod! {

unsafe fn HandlerCF_QueryInterface(
    iface: LPCLASSFACTORY,
    riid: *const GUID,
    ppv: *mut LPVOID,
) -> HRESULT {

    *ppv = core::ptr::null_mut();
    if IsEqualIID(riid, &IID_IUnknown) ||
       IsEqualIID(riid, &IID_IClassFactory)
    {
        *ppv = iface as LPVOID;
        IClassFactory_AddRef(iface);
        return S_OK;
    }
    E_NOINTERFACE
}

unsafe fn HandlerCF_AddRef(iface: LPCLASSFACTORY) -> ULONG {
    let This = impl_from_IClassFactory(iface);
    InterlockedIncrement(core::ptr::addr_of_mut!((*This).refs)) as ULONG
}

unsafe fn HandlerCF_Release(iface: LPCLASSFACTORY) -> ULONG {
    let This = impl_from_IClassFactory(iface);
    let refs = InterlockedDecrement(core::ptr::addr_of_mut!((*This).refs)) as ULONG;
    if refs == 0 {
        HeapFree(GetProcessHeap(), 0, This as LPVOID);
    }
    refs
}

unsafe fn HandlerCF_CreateInstance(
    iface: *mut IClassFactory,
    pUnk: *mut IUnknown,
    riid: *const CLSID,
    ppv: *mut LPVOID,
) -> HRESULT {
    let This = impl_from_IClassFactory(iface);
    OleCreateDefaultHandler(
        core::ptr::addr_of!((*This).clsid),
        pUnk,
        riid,
        ppv,
    )
}

unsafe fn HandlerCF_LockServer(iface: LPCLASSFACTORY, fLock: BOOL) -> HRESULT {
    FIXME!(b"(%d), stub!\n\0".as_ptr(), fLock);
    S_OK
}

} // stdmethod!

#[no_mangle]
pub unsafe extern "C" fn HandlerCF_Create(rclsid: *const CLSID, riid: *const CLSID, ppv: *mut LPVOID) -> HRESULT {

    let This = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<HandlerCF>() as SIZE_T,
    ) as *mut HandlerCF;
    if This.is_null() {
        return E_OUTOFMEMORY;
    }
    (*This).IClassFactory_iface.lpVtbl =
        &HandlerClassFactoryVtbl as *const IClassFactoryVtbl as *mut IClassFactoryVtbl;
    (*This).refs = 0;
    (*This).clsid = *rclsid;

    let hr = IClassFactory_QueryInterface(
        core::ptr::addr_of_mut!((*This).IClassFactory_iface),
        riid,
        ppv,
    );
    if FAILED(hr) {
        HeapFree(GetProcessHeap(), 0, This as LPVOID);
    }

    hr
}
