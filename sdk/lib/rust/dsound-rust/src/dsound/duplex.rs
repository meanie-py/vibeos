use crate::dsound::capture::DSOUND_CaptureCreate8;
use crate::dsound::dsound::DSOUND_Create8;
use crate::dsound::dsound::DirectSoundDevice_Initialize;
use crate::dsound::buffer::IDirectSoundBufferImpl_Create;
use crate::dsound::dsound::dumpCooperativeLevel;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct IDirectSoundFullDuplexImpl {
    pub lpVtbl: *const IDirectSoundFullDuplexVtbl,
    pub ref_: LONG,
    pub renderer_device: *mut IDirectSound8,
    pub capture_device: *mut IDirectSoundCapture,
    pub pUnknown: LPUNKNOWN,
    pub pDS8: LPDIRECTSOUND8,
    pub pDSC: LPDIRECTSOUNDCAPTURE,
}

#[repr(C)]
pub struct IDirectSoundFullDuplex_IUnknown {
    pub lpVtbl: *const IUnknownVtbl,
    pub ref_: LONG,
    pub pdsfd: *mut IDirectSoundFullDuplexImpl,
}

#[repr(C)]
pub struct IDirectSoundFullDuplex_IDirectSound8 {
    pub lpVtbl: *const IDirectSound8Vtbl,
    pub ref_: LONG,
    pub pdsfd: *mut IDirectSoundFullDuplexImpl,
}

#[repr(C)]
pub struct IDirectSoundFullDuplex_IDirectSoundCapture {
    pub lpVtbl: *const IDirectSoundCaptureVtbl,
    pub ref_: LONG,
    pub pdsfd: *mut IDirectSoundFullDuplexImpl,
}

static IDirectSoundFullDuplex_IUnknown_AddRef_dummy: u8 = 0;
unsafe extern "system" fn IDirectSoundFullDuplex_IUnknown_QueryInterface(
    mut iface: LPUNKNOWN,
    riid: *const IID,
    mut ppobj: *mut LPVOID,
) -> HRESULT {
    let mut This: *mut IDirectSoundFullDuplex_IUnknown =
        iface as *mut IDirectSoundFullDuplex_IUnknown;
    WINE_TRACE!("(%p,%s,%p)\n", This, debugstr_guid(riid as *const _GUID), ppobj);
    return (*(*((*This).pdsfd as LPDIRECTSOUNDFULLDUPLEX)).lpVtbl)
        .QueryInterface
        .unwrap_unchecked()(
        (*This).pdsfd as *mut IDirectSoundFullDuplex,
        riid,
        ppobj as *mut *mut ::core::ffi::c_void,
    );
}

unsafe extern "system" fn IDirectSoundFullDuplex_IUnknown_AddRef(iface: LPUNKNOWN) -> ULONG {
    let This: *mut IDirectSoundFullDuplex_IUnknown = iface as *mut IDirectSoundFullDuplex_IUnknown;
    let ref_0: ULONG = InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_) as *mut LONG) as ULONG;
    return ref_0;
}

unsafe extern "system" fn IDirectSoundFullDuplex_IUnknown_Release(iface: LPUNKNOWN) -> ULONG {
    let This: *mut IDirectSoundFullDuplex_IUnknown = iface as *mut IDirectSoundFullDuplex_IUnknown;
    let mut ref_0: ULONG =
        InterlockedDecrement(core::ptr::addr_of_mut!((*This).ref_) as *mut LONG) as ULONG;
    if ref_0 == 0 {
        (*(*This).pdsfd).pUnknown = core::ptr::null_mut::<IUnknown>();
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            This as PVOID,
        );
    }
    return ref_0;
}

unsafe extern "C" fn IDirectSoundFullDuplex_IUnknown_Create(
    mut pdsfd: LPDIRECTSOUNDFULLDUPLEX,
    mut ppunk: *mut LPUNKNOWN,
) -> HRESULT {
    let mut pdsfdunk: *mut IDirectSoundFullDuplex_IUnknown = core::ptr::null_mut();

    if pdsfd.is_null() {
        ERR!("invalid parameter: pdsfd == NULL\n");
        return DSERR_INVALIDPARAM;
    }

    if ppunk.is_null() {
        ERR!("invalid parameter: ppunk == NULL\n");
        return DSERR_INVALIDPARAM;
    }

    pdsfdunk = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        core::mem::size_of::<IDirectSoundFullDuplex_IUnknown>() as SIZE_T,
    ) as *mut IDirectSoundFullDuplex_IUnknown;
    if pdsfdunk.is_null() {
        WARN!("out of memory\n");
        *ppunk = core::ptr::null_mut();
        return DSERR_OUTOFMEMORY;
    }

    (*pdsfdunk).lpVtbl = &DirectSoundFullDuplex_Unknown_Vtbl as *const IUnknownVtbl;
    (*pdsfdunk).ref_ = 0;
    (*pdsfdunk).pdsfd = pdsfd as *mut IDirectSoundFullDuplexImpl;

    *ppunk = pdsfdunk as LPUNKNOWN;

    return (DS_OK) as HRESULT;
}

unsafe extern "system" fn IDirectSoundFullDuplex_IDirectSound8_QueryInterface(
    iface: LPDIRECTSOUND8,
    riid: REFIID,
    ppobj: *mut LPVOID,
) -> HRESULT {
    let This: *mut IDirectSoundFullDuplex_IDirectSound8 =
        iface as *mut IDirectSoundFullDuplex_IDirectSound8;
    return IDirectSoundFullDuplex_QueryInterface(
        (*This).pdsfd as LPDIRECTSOUNDFULLDUPLEX,
        riid,
        ppobj,
    );
}

unsafe extern "system" fn IDirectSoundFullDuplex_IDirectSound8_AddRef(
    iface: LPDIRECTSOUND8,
) -> ULONG {
    let This: *mut IDirectSoundFullDuplex_IDirectSound8 =
        iface as *mut IDirectSoundFullDuplex_IDirectSound8;
    let ref_0: ULONG =
        InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_) as *mut LONG) as ULONG;
    return ref_0;
}

unsafe extern "system" fn IDirectSoundFullDuplex_IDirectSound8_Release(
    iface: LPDIRECTSOUND8,
) -> ULONG {
    let This: *mut IDirectSoundFullDuplex_IDirectSound8 =
        iface as *mut IDirectSoundFullDuplex_IDirectSound8;
    let ref_0: ULONG = InterlockedDecrement(
        core::ptr::addr_of_mut!((*This).ref_) as *mut ::core::ffi::c_long,
    ) as ULONG;
    if ref_0 == 0 {
        (*(*This).pdsfd).pDS8 = core::ptr::null_mut();
        HeapFree(
            GetProcessHeap(),
            0,
            This as PVOID,
        );
    }
    return ref_0;
}

unsafe extern "system" fn IDirectSoundFullDuplex_IDirectSound8_CreateSoundBuffer(
    iface: LPDIRECTSOUND8,
    dsbd: LPCDSBUFFERDESC,
    ppdsb: LPLPDIRECTSOUNDBUFFER,
    lpunk: LPUNKNOWN,
) -> HRESULT {
    let mut This: *mut IDirectSoundFullDuplex_IDirectSound8 =
        iface as *mut IDirectSoundFullDuplex_IDirectSound8;
    WINE_TRACE!("(%p,%p,%p,%p)\n", This, dsbd, ppdsb, lpunk);
    return (*(*(*(*This).pdsfd).renderer_device).lpVtbl)
        .CreateSoundBuffer
        .unwrap_unchecked()(
        (*(*This).pdsfd).renderer_device,
        dsbd,
        ppdsb,
        lpunk as *mut IUnknown,
    );
}

unsafe extern "system" fn IDirectSoundFullDuplex_IDirectSound8_GetCaps(
    iface: LPDIRECTSOUND8,
    lpDSCaps: LPDSCAPS,
) -> HRESULT {
    let This: *mut IDirectSoundFullDuplex_IDirectSound8 =
        iface as *mut IDirectSoundFullDuplex_IDirectSound8;
    return IDirectSound8_GetCaps((*(*This).pdsfd).renderer_device, lpDSCaps);
}

unsafe extern "system" fn IDirectSoundFullDuplex_IDirectSound8_DuplicateSoundBuffer(
    mut iface: LPDIRECTSOUND8,
    mut psb: LPDIRECTSOUNDBUFFER,
    mut ppdsb: LPLPDIRECTSOUNDBUFFER,
) -> HRESULT {
    let mut This: *mut IDirectSoundFullDuplex_IDirectSound8 =
        iface as *mut IDirectSoundFullDuplex_IDirectSound8;
    WINE_TRACE!("(%p,%p,%p)\n", This, psb, ppdsb);
    return (*(*(*(*This).pdsfd).renderer_device).lpVtbl)
        .DuplicateSoundBuffer
        .unwrap_unchecked()(
        (*(*This).pdsfd).renderer_device,
        psb,
        ppdsb,
    );
}
unsafe extern "system" fn IDirectSoundFullDuplex_IDirectSound8_SetCooperativeLevel(
    iface: LPDIRECTSOUND8,
    hwnd: HWND,
    level: DWORD,
) -> HRESULT {
    let This: *mut IDirectSoundFullDuplex_IDirectSound8 =
        iface as *mut IDirectSoundFullDuplex_IDirectSound8;
    WINE_TRACE!("(%p,%p,%s)\n", This, hwnd, dumpCooperativeLevel(level));
    return IDirectSound8_SetCooperativeLevel((*(*This).pdsfd).renderer_device, hwnd, level);
}
unsafe extern "system" fn IDirectSoundFullDuplex_IDirectSound8_Compact(
    iface: LPDIRECTSOUND8,
) -> HRESULT {
    let This: *mut IDirectSoundFullDuplex_IDirectSound8 =
        iface as *mut IDirectSoundFullDuplex_IDirectSound8;
    return IDirectSound8_Compact((*(*This).pdsfd).renderer_device);
}

unsafe extern "system" fn IDirectSoundFullDuplex_IDirectSound8_GetSpeakerConfig(
    iface: LPDIRECTSOUND8,
    lpdwSpeakerConfig: LPDWORD,
) -> HRESULT {
    let This: *mut IDirectSoundFullDuplex_IDirectSound8 =
        iface as *mut IDirectSoundFullDuplex_IDirectSound8;
    return IDirectSound8_GetSpeakerConfig((*(*This).pdsfd).renderer_device, lpdwSpeakerConfig);
}

unsafe extern "system" fn IDirectSoundFullDuplex_IDirectSound8_SetSpeakerConfig(
    iface: LPDIRECTSOUND8,
    config: DWORD,
) -> HRESULT {
    let This: *mut IDirectSoundFullDuplex_IDirectSound8 =
        iface as *mut IDirectSoundFullDuplex_IDirectSound8;
    return IDirectSound8_SetSpeakerConfig((*(*This).pdsfd).renderer_device, config);
}
unsafe extern "system" fn IDirectSoundFullDuplex_IDirectSound8_Initialize(
    iface: LPDIRECTSOUND8,
    lpcGuid: LPCGUID,
) -> HRESULT {
    let This: *mut IDirectSoundFullDuplex_IDirectSound8 =
        iface as *mut IDirectSoundFullDuplex_IDirectSound8;
    WINE_TRACE!("(%p, %s)\n", This, debugstr_guid(lpcGuid as *const _GUID));
    return (*(*(*(*This).pdsfd).renderer_device).lpVtbl)
        .Initialize
        .unwrap_unchecked()((*(*This).pdsfd).renderer_device, lpcGuid);
}
unsafe extern "system" fn IDirectSoundFullDuplex_IDirectSound8_VerifyCertification(
    iface: LPDIRECTSOUND8,
    cert: *mut DWORD,
) -> HRESULT {
    let This: *mut IDirectSoundFullDuplex_IDirectSound8 =
        iface as *mut IDirectSoundFullDuplex_IDirectSound8;
    WINE_TRACE!("(%p, %p)\n", This, cert);
    return IDirectSound8_VerifyCertification((*(*This).pdsfd).renderer_device, cert);
}

unsafe extern "C" fn IDirectSoundFullDuplex_IDirectSound8_Create(
    pdsfd: LPDIRECTSOUNDFULLDUPLEX,
    ppds8: *mut LPDIRECTSOUND8,
) -> HRESULT {
    let mut pdsfdds8: *mut IDirectSoundFullDuplex_IDirectSound8 =
        core::ptr::null_mut::<IDirectSoundFullDuplex_IDirectSound8>();

    if pdsfd.is_null() {
        ERR!("invalid parameter: pdsfd == NULL\n");
        return DSERR_INVALIDPARAM;
    }

    if ppds8.is_null() {
        ERR!("invalid parameter: ppds8 == NULL\n");
        return DSERR_INVALIDPARAM;
    }

    if (*(pdsfd as *mut IDirectSoundFullDuplexImpl)).renderer_device.is_null() {
        WARN!("not initialized\n");
        *ppds8 = core::ptr::null_mut::<IDirectSound8>();
        return DSERR_UNINITIALIZED;
    }

    pdsfdds8 = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<IDirectSoundFullDuplex_IDirectSound8>() as SIZE_T,
    ) as *mut IDirectSoundFullDuplex_IDirectSound8;
    if pdsfdds8.is_null() {
        WARN!("out of memory\n");
        *ppds8 = core::ptr::null_mut::<IDirectSound8>();
        return DSERR_OUTOFMEMORY;
    }

    (*pdsfdds8).lpVtbl = core::ptr::addr_of!(DirectSoundFullDuplex_DirectSound8_Vtbl);
    (*pdsfdds8).ref_ = 0 as LONG;
    (*pdsfdds8).pdsfd = pdsfd as *mut IDirectSoundFullDuplexImpl;

    *ppds8 = pdsfdds8 as LPDIRECTSOUND8;

    return (DS_OK) as HRESULT;
}

unsafe extern "system" fn IDirectSoundFullDuplex_IDirectSoundCapture_QueryInterface(
    iface: LPDIRECTSOUNDCAPTURE,
    riid: REFIID,
    ppobj: *mut LPVOID,
) -> HRESULT {
    let This: *mut IDirectSoundFullDuplex_IDirectSoundCapture =
        iface as *mut IDirectSoundFullDuplex_IDirectSoundCapture;
    WINE_TRACE!("(%p,%s,%p)\n", This, debugstr_guid(riid as *const _GUID), ppobj);
    return IDirectSoundFullDuplex_QueryInterface(
        (*This).pdsfd as LPDIRECTSOUNDFULLDUPLEX,
        riid,
        ppobj,
    );
}

unsafe extern "system" fn IDirectSoundFullDuplex_IDirectSoundCapture_AddRef(
    iface: LPDIRECTSOUNDCAPTURE,
) -> ULONG {
    let This: *mut IDirectSoundFullDuplex_IDirectSoundCapture =
        iface as *mut IDirectSoundFullDuplex_IDirectSoundCapture;
    let ref_0: ULONG = InterlockedIncrement(
        core::ptr::addr_of_mut!((*This).ref_) as *mut LONG,
    ) as ULONG;
    return ref_0;
}

unsafe extern "system" fn IDirectSoundFullDuplex_IDirectSoundCapture_Release(iface: LPDIRECTSOUNDCAPTURE) -> ULONG {
    let This: *mut IDirectSoundFullDuplex_IDirectSoundCapture = iface as *mut IDirectSoundFullDuplex_IDirectSoundCapture;
    let ref_0: ULONG = InterlockedDecrement(core::ptr::addr_of_mut!((*This).ref_) as *mut ::core::ffi::c_long) as ULONG;
    WINE_TRACE!("(%p) ref was %d\n", This, ref_0.wrapping_add(1 as ULONG));
    if ref_0 == 0 {
        (*(*This).pdsfd).pDSC = core::ptr::null_mut::<IDirectSoundCapture>();
        HeapFree(GetProcessHeap(), 0 as DWORD, This as PVOID);
        WINE_TRACE!("(%p) released\n", This);
    }
    return ref_0;
}
unsafe extern "system" fn IDirectSoundFullDuplex_IDirectSoundCapture_CreateCaptureBuffer(
    mut iface: LPDIRECTSOUNDCAPTURE,
    mut lpcDSCBufferDesc: LPCDSCBUFFERDESC,
    mut lplpDSCaptureBuffer: *mut LPDIRECTSOUNDCAPTUREBUFFER,
    mut pUnk: LPUNKNOWN,
) -> HRESULT {
    let mut This: *mut IDirectSoundFullDuplex_IDirectSoundCapture =
        iface as *mut IDirectSoundFullDuplex_IDirectSoundCapture;
    WINE_TRACE!("(%p,%p,%p,%p)\n", This, lpcDSCBufferDesc, lplpDSCaptureBuffer, pUnk);
    return IDirectSoundCapture_CreateCaptureBuffer(
        (*(*This).pdsfd).capture_device,
        lpcDSCBufferDesc,
        lplpDSCaptureBuffer,
        pUnk,
    );
}

unsafe extern "system" fn IDirectSoundFullDuplex_IDirectSoundCapture_GetCaps(
    iface: LPDIRECTSOUNDCAPTURE,
    lpDSCCaps: LPDSCCAPS,
) -> HRESULT {
    let This: *mut IDirectSoundFullDuplex_IDirectSoundCapture =
        iface as *mut IDirectSoundFullDuplex_IDirectSoundCapture;
    return (*(*(*(*This).pdsfd).capture_device).lpVtbl)
        .GetCaps
        .unwrap_unchecked()((*(*This).pdsfd).capture_device, lpDSCCaps);
}

unsafe extern "system" fn IDirectSoundFullDuplex_IDirectSoundCapture_Initialize(
    mut iface: LPDIRECTSOUNDCAPTURE,
    mut lpcGUID: LPCGUID,
) -> HRESULT {
    let mut This: *mut IDirectSoundFullDuplex_IDirectSoundCapture =
        iface as *mut IDirectSoundFullDuplex_IDirectSoundCapture;
    return ((*(*(*(*This).pdsfd).capture_device).lpVtbl)
        .Initialize
        .unwrap_unchecked())((*(*This).pdsfd).capture_device, lpcGUID);
}

unsafe extern "C" fn IDirectSoundFullDuplex_IDirectSoundCapture_Create(
    pdsfd: LPDIRECTSOUNDFULLDUPLEX,
    ppdsc8: *mut LPDIRECTSOUNDCAPTURE8,
) -> HRESULT {
    let mut pdsfddsc: *mut IDirectSoundFullDuplex_IDirectSoundCapture = core::ptr::null_mut();

    if pdsfd.is_null() {
        ERR!("invalid parameter: pdsfd == NULL\n");
        return DSERR_INVALIDPARAM;
    }

    if ppdsc8.is_null() {
        ERR!("invalid parameter: ppdsc8 == NULL\n");
        return DSERR_INVALIDPARAM;
    }

    if (*(pdsfd as *mut IDirectSoundFullDuplexImpl)).capture_device.is_null() {
        WARN!("not initialized\n");
        *ppdsc8 = core::ptr::null_mut();
        return DSERR_UNINITIALIZED;
    }

    pdsfddsc = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<IDirectSoundFullDuplex_IDirectSoundCapture>() as SIZE_T,
    ) as *mut IDirectSoundFullDuplex_IDirectSoundCapture;
    if pdsfddsc.is_null() {
        WARN!("out of memory\n");
        *ppdsc8 = core::ptr::null_mut();
        return DSERR_OUTOFMEMORY;
    }

    (*pdsfddsc).lpVtbl = core::ptr::addr_of!(DirectSoundFullDuplex_DirectSoundCapture_Vtbl);
    (*pdsfddsc).ref_ = 0;
    (*pdsfddsc).pdsfd = pdsfd as *mut IDirectSoundFullDuplexImpl;

    *ppdsc8 = pdsfddsc as LPDIRECTSOUNDCAPTURE as LPDIRECTSOUNDCAPTURE8;

    return (DS_OK) as HRESULT;
}

unsafe extern "system" fn IDirectSoundFullDuplexImpl_AddRef(iface: LPDIRECTSOUNDFULLDUPLEX) -> ULONG {
    let This: *mut IDirectSoundFullDuplexImpl = iface as *mut IDirectSoundFullDuplexImpl;
    let ref_0: ULONG = InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_) as *mut i32) as ULONG;
    return ref_0;
}
unsafe extern "system" fn IDirectSoundFullDuplexImpl_QueryInterface(iface: LPDIRECTSOUNDFULLDUPLEX, riid: *const IID, ppobj: *mut LPVOID) -> HRESULT {
    let This: *mut IDirectSoundFullDuplexImpl = iface as *mut IDirectSoundFullDuplexImpl;

    if ppobj.is_null() {
        WARN!("invalid parameter\n");
        return E_INVALIDARG;
    }

    *ppobj = core::ptr::null_mut();

    if IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown)) != 0 {
        if (*This).pUnknown.is_null() {
            IDirectSoundFullDuplex_IUnknown_Create(iface, core::ptr::addr_of_mut!((*This).pUnknown));
            if (*This).pUnknown.is_null() {
                WARN!("IDirectSoundFullDuplex_IUnknown_Create() failed\n");
                *ppobj = core::ptr::null_mut();
                return E_NOINTERFACE;
            }
        }
        IDirectSoundFullDuplex_IUnknown_AddRef((*This).pUnknown);
        *ppobj = (*This).pUnknown as LPVOID;
        return S_OK;
    } else if IsEqualIID!(riid, core::ptr::addr_of!(IID_IDirectSoundFullDuplex)) != 0 {
        IDirectSoundFullDuplexImpl_AddRef(iface);
        *ppobj = This as LPVOID;
        return S_OK;
    } else if IsEqualIID!(riid, core::ptr::addr_of!(IID_IDirectSound)) != 0
        || IsEqualIID!(riid, core::ptr::addr_of!(IID_IDirectSound8)) != 0
    {
        if (*This).pDS8.is_null() {
            IDirectSoundFullDuplex_IDirectSound8_Create(iface, core::ptr::addr_of_mut!((*This).pDS8));
            if (*This).pDS8.is_null() {
                WARN!("IDirectSoundFullDuplex_IDirectSound8_Create() failed\n");
                *ppobj = core::ptr::null_mut();
                return E_NOINTERFACE;
            }
        }
        IDirectSoundFullDuplex_IDirectSound8_AddRef((*This).pDS8);
        *ppobj = (*This).pDS8 as LPVOID;
        return S_OK;
    } else if IsEqualIID!(riid, core::ptr::addr_of!(IID_IDirectSoundCapture)) != 0 {
        if (*This).pDSC.is_null() {
            IDirectSoundFullDuplex_IDirectSoundCapture_Create(iface, core::ptr::addr_of_mut!((*This).pDSC));
            if (*This).pDSC.is_null() {
                WARN!("IDirectSoundFullDuplex_IDirectSoundCapture_Create() failed\n");
                *ppobj = core::ptr::null_mut();
                return E_NOINTERFACE;
            }
        }
        IDirectSoundFullDuplex_IDirectSoundCapture_AddRef((*This).pDSC);
        *ppobj = (*This).pDSC as LPVOID;
        return S_OK;
    }

    return E_NOINTERFACE;
}

unsafe extern "system" fn IDirectSoundFullDuplexImpl_Release(mut iface: LPDIRECTSOUNDFULLDUPLEX) -> ULONG {
    let mut This: *mut IDirectSoundFullDuplexImpl = iface as *mut IDirectSoundFullDuplexImpl;
    let mut ref_0: ULONG =
        InterlockedDecrement(&raw mut (*This).ref_ as *mut ::core::ffi::c_long) as ULONG;
    WINE_TRACE!("(%p) ref was %d\n", This, ref_0.wrapping_sub(1 as ULONG));
    if ref_0 == 0 {
        if !(*This).capture_device.is_null() {
            IDirectSoundCapture_Release((*This).capture_device);
        }
        if !(*This).renderer_device.is_null() {
            IDirectSound_Release(((*This).renderer_device).cast());
        }
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            This as PVOID,
        );
        WINE_TRACE!("(%p) released\n", This);
    }
    return ref_0;
}
unsafe extern "system" fn IDirectSoundFullDuplexImpl_Initialize(iface: LPDIRECTSOUNDFULLDUPLEX, pCaptureGuid: LPCGUID, pRendererGuid: LPCGUID, lpDscBufferDesc: LPCDSCBUFFERDESC, lpDsBufferDesc: LPCDSBUFFERDESC, hWnd: HWND, dwLevel: DWORD, lplpDirectSoundCaptureBuffer8: LPLPDIRECTSOUNDCAPTUREBUFFER8, lplpDirectSoundBuffer8: LPLPDIRECTSOUNDBUFFER8) -> HRESULT {
    let mut hr: HRESULT;
    let This: *mut IDirectSoundFullDuplexImpl = iface as *mut IDirectSoundFullDuplexImpl;

    if lplpDirectSoundCaptureBuffer8.is_null() || lplpDirectSoundBuffer8.is_null() {
        return E_INVALIDARG;
    }

    if !(*This).renderer_device.is_null() || !(*This).capture_device.is_null() {
        WARN!("already initialized\n");
        *lplpDirectSoundCaptureBuffer8 = core::ptr::null_mut();
        *lplpDirectSoundBuffer8 = core::ptr::null_mut();
        return DSERR_ALREADYINITIALIZED;
    }

    hr = DSOUND_Create8(
        core::ptr::addr_of!(IID_IDirectSound8),
        core::ptr::addr_of_mut!((*This).renderer_device),
    );
    if hr >= 0 {
        hr = IDirectSound_Initialize(((*This).renderer_device).cast(), pRendererGuid);
    }
    if hr != (DS_OK as HRESULT) {
        WARN!("DirectSoundDevice_Initialize() failed\n");
        *lplpDirectSoundCaptureBuffer8 = core::ptr::null_mut();
        *lplpDirectSoundBuffer8 = core::ptr::null_mut();
        return hr;
    }

    IDirectSound8_SetCooperativeLevel((*This).renderer_device, hWnd, dwLevel);

    hr = IDirectSound8_CreateSoundBuffer(
        (*This).renderer_device,
        lpDsBufferDesc,
        lplpDirectSoundBuffer8 as *mut *mut IDirectSoundBuffer,
        core::ptr::null_mut(),
    );
    if hr != (DS_OK as HRESULT) {
        WARN!("IDirectSoundBufferImpl_Create() failed\n");
        *lplpDirectSoundCaptureBuffer8 = core::ptr::null_mut();
        *lplpDirectSoundBuffer8 = core::ptr::null_mut();
        return hr;
    }

    hr = DSOUND_CaptureCreate8(
        core::ptr::addr_of!(IID_IDirectSoundCapture8),
        core::ptr::addr_of_mut!((*This).capture_device),
    );
    if hr >= 0 {
        hr = IDirectSoundCapture_Initialize((*This).capture_device, pCaptureGuid);
    }
    if hr != (DS_OK as HRESULT) {
        WARN!("DirectSoundCaptureDevice_Initialize() failed\n");
        *lplpDirectSoundCaptureBuffer8 = core::ptr::null_mut();
        *lplpDirectSoundBuffer8 = core::ptr::null_mut();
        return hr;
    }

    hr = IDirectSoundCapture_CreateCaptureBuffer(
        (*This).capture_device,
        lpDscBufferDesc,
        lplpDirectSoundCaptureBuffer8 as *mut *mut IDirectSoundCaptureBuffer,
        core::ptr::null_mut(),
    );
    if hr != (DS_OK as HRESULT) {
        WARN!("IDirectSoundCaptureBufferImpl_Create() failed\n");
        *lplpDirectSoundCaptureBuffer8 = core::ptr::null_mut();
        *lplpDirectSoundBuffer8 = core::ptr::null_mut();
        return hr;
    }

    return hr;
}
#[no_mangle]
pub unsafe extern "C" fn DSOUND_FullDuplexCreate(riid: *const IID, ppDSFD: *mut LPDIRECTSOUNDFULLDUPLEX) -> HRESULT {
    let mut This: *mut IDirectSoundFullDuplexImpl = core::ptr::null_mut();

    if ppDSFD.is_null() {
        WARN!("invalid parameter: ppDSFD == NULL\n");
        return DSERR_INVALIDPARAM;
    }

    if ((IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown))) == 0)
        && ((IsEqualIID!(riid, core::ptr::addr_of!(IID_IDirectSoundFullDuplex))) == 0)
    {
        *ppDSFD = core::ptr::null_mut();
        return E_NOINTERFACE;
    }

    /* Get dsound configuration */
    setup_dsound_options();

    This = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY as DWORD,
        core::mem::size_of::<IDirectSoundFullDuplexImpl>() as SIZE_T,
    ) as *mut IDirectSoundFullDuplexImpl;

    if This.is_null() {
        WARN!("out of memory\n");
        *ppDSFD = core::ptr::null_mut();
        return DSERR_OUTOFMEMORY;
    }

    (*This).lpVtbl = core::ptr::addr_of!(dsfdvt);
    (*This).ref_ = 1 as LONG;
    (*This).capture_device = core::ptr::null_mut();
    (*This).renderer_device = core::ptr::null_mut();

    *ppDSFD = This as LPDIRECTSOUNDFULLDUPLEX;

    return (DS_OK) as HRESULT;
}

#[no_mangle]
pub unsafe extern "system" fn DirectSoundFullDuplexCreate(
    pcGuidCaptureDevice: LPCGUID,
    pcGuidRenderDevice: LPCGUID,
    pcDSCBufferDesc: LPCDSCBUFFERDESC,
    pcDSBufferDesc: LPCDSBUFFERDESC,
    hWnd: HWND,
    dwLevel: DWORD,
    ppDSFD: *mut LPDIRECTSOUNDFULLDUPLEX,
    ppDSCBuffer8: *mut LPDIRECTSOUNDCAPTUREBUFFER8,
    ppDSBuffer8: *mut LPDIRECTSOUNDBUFFER8,
    pUnkOuter: LPUNKNOWN,
) -> HRESULT {
    let mut hres: HRESULT;
    let mut This: *mut IDirectSoundFullDuplexImpl = core::ptr::null_mut();

    if !pUnkOuter.is_null() {
        WARN!("pUnkOuter != 0\n");
        *ppDSFD = core::ptr::null_mut();
        return DSERR_NOAGGREGATION;
    }

    if pcDSCBufferDesc.is_null() {
        WARN!("invalid parameter: pcDSCBufferDesc == NULL\n");
        *ppDSFD = core::ptr::null_mut();
        return DSERR_INVALIDPARAM;
    }

    if pcDSBufferDesc.is_null() {
        WARN!("invalid parameter: pcDSBufferDesc == NULL\n");
        *ppDSFD = core::ptr::null_mut();
        return DSERR_INVALIDPARAM;
    }

    if ppDSFD.is_null() {
        WARN!("invalid parameter: ppDSFD == NULL\n");
        return DSERR_INVALIDPARAM;
    }

    if ppDSCBuffer8.is_null() {
        WARN!("invalid parameter: ppDSCBuffer8 == NULL\n");
        *ppDSFD = core::ptr::null_mut();
        return DSERR_INVALIDPARAM;
    }

    if ppDSBuffer8.is_null() {
        WARN!("invalid parameter: ppDSBuffer8 == NULL\n");
        *ppDSFD = core::ptr::null_mut();
        return DSERR_INVALIDPARAM;
    }

    hres = DSOUND_FullDuplexCreate(
        core::ptr::addr_of!(IID_IDirectSoundFullDuplex),
        core::ptr::addr_of_mut!(This) as *mut LPDIRECTSOUNDFULLDUPLEX,
    );
    if hres < 0 {
        return hres;
    }

    hres = IDirectSoundFullDuplexImpl_Initialize(
        This as LPDIRECTSOUNDFULLDUPLEX,
        pcGuidCaptureDevice,
        pcGuidRenderDevice,
        pcDSCBufferDesc,
        pcDSBufferDesc,
        hWnd,
        dwLevel,
        ppDSCBuffer8 as LPLPDIRECTSOUNDCAPTUREBUFFER8,
        ppDSBuffer8 as LPLPDIRECTSOUNDBUFFER8,
    );
    if hres != (DS_OK as HRESULT) {
        IUnknown_Release((This as LPDIRECTSOUNDFULLDUPLEX).cast());
        WARN!("IDirectSoundFullDuplexImpl_Initialize failed\n");
        *ppDSFD = core::ptr::null_mut();
    } else {
        *ppDSFD = This as LPDIRECTSOUNDFULLDUPLEX;
    }

    return hres;
}

// COM vtable tables — generated from duplex.c by vtbl_gen.py
/// duplex.c:97 `static const IUnknownVtbl DirectSoundFullDuplex_Unknown_Vtbl`  (slot order: com-sys/bindgen_i386.rs)
pub static DirectSoundFullDuplex_Unknown_Vtbl: IUnknownVtbl = IUnknownVtbl {
    QueryInterface: Some(IDirectSoundFullDuplex_IUnknown_QueryInterface),
    AddRef: Some(IDirectSoundFullDuplex_IUnknown_AddRef),
    Release: Some(IDirectSoundFullDuplex_IUnknown_Release),
};

/// duplex.c:257 `static const IDirectSound8Vtbl DirectSoundFullDuplex_DirectSound8_Vtbl`  (slot order: rust:types_amd64.rs)
pub static DirectSoundFullDuplex_DirectSound8_Vtbl: IDirectSound8Vtbl = IDirectSound8Vtbl {
    QueryInterface: Some(IDirectSoundFullDuplex_IDirectSound8_QueryInterface),
    AddRef: Some(IDirectSoundFullDuplex_IDirectSound8_AddRef),
    Release: Some(IDirectSoundFullDuplex_IDirectSound8_Release),
    CreateSoundBuffer: Some(IDirectSoundFullDuplex_IDirectSound8_CreateSoundBuffer),
    GetCaps: Some(IDirectSoundFullDuplex_IDirectSound8_GetCaps),
    DuplicateSoundBuffer: Some(IDirectSoundFullDuplex_IDirectSound8_DuplicateSoundBuffer),
    SetCooperativeLevel: Some(IDirectSoundFullDuplex_IDirectSound8_SetCooperativeLevel),
    Compact: Some(IDirectSoundFullDuplex_IDirectSound8_Compact),
    GetSpeakerConfig: Some(IDirectSoundFullDuplex_IDirectSound8_GetSpeakerConfig),
    SetSpeakerConfig: Some(IDirectSoundFullDuplex_IDirectSound8_SetSpeakerConfig),
    Initialize: Some(IDirectSoundFullDuplex_IDirectSound8_Initialize),
    VerifyCertification: Some(IDirectSoundFullDuplex_IDirectSound8_VerifyCertification),
};

/// duplex.c:377 `static const IDirectSoundCaptureVtbl DirectSoundFullDuplex_DirectSoundCapture_Vtbl`  (slot order: rust:types_amd64.rs)
pub static DirectSoundFullDuplex_DirectSoundCapture_Vtbl: IDirectSoundCaptureVtbl = IDirectSoundCaptureVtbl {
    QueryInterface: Some(IDirectSoundFullDuplex_IDirectSoundCapture_QueryInterface),
    AddRef: Some(IDirectSoundFullDuplex_IDirectSoundCapture_AddRef),
    Release: Some(IDirectSoundFullDuplex_IDirectSoundCapture_Release),
    CreateCaptureBuffer: Some(IDirectSoundFullDuplex_IDirectSoundCapture_CreateCaptureBuffer),
    GetCaps: Some(IDirectSoundFullDuplex_IDirectSoundCapture_GetCaps),
    Initialize: Some(IDirectSoundFullDuplex_IDirectSoundCapture_Initialize),
};

/// duplex.c:593 `static const IDirectSoundFullDuplexVtbl dsfdvt`  (slot order: rust:types_amd64.rs)
pub static dsfdvt: IDirectSoundFullDuplexVtbl = IDirectSoundFullDuplexVtbl {
    QueryInterface: Some(IDirectSoundFullDuplexImpl_QueryInterface),
    AddRef: Some(IDirectSoundFullDuplexImpl_AddRef),
    Release: Some(IDirectSoundFullDuplexImpl_Release),
    Initialize: Some(IDirectSoundFullDuplexImpl_Initialize),
};
