use crate::dsound::dsound_main::DSOUND_capture_guids;
use crate::dsound::dsound_main::ds_hw_accel;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)

static mut captureStateString: [*const ::core::ffi::c_char; 4] = [
    b"STATE_STOPPED\0" as *const u8 as *const ::core::ffi::c_char,
    b"STATE_STARTING\0" as *const u8 as *const ::core::ffi::c_char,
    b"STATE_CAPTURING\0" as *const u8 as *const ::core::ffi::c_char,
    b"STATE_STOPPING\0" as *const u8 as *const ::core::ffi::c_char,
];

#[no_mangle]
pub static mut DSOUND_capture: [*mut DirectSoundCaptureDevice; 10] =
    [::core::ptr::null::<DirectSoundCaptureDevice>() as *mut DirectSoundCaptureDevice; 10];


pub type PIDirectSoundCaptureNotifyImpl = *mut IDirectSoundCaptureNotifyImpl;



pub type PIDirectSoundCaptureImpl = *mut IDirectSoundCaptureImpl;

unsafe extern "system" fn IDirectSoundCaptureNotifyImpl_QueryInterface(
    iface: LPDIRECTSOUNDNOTIFY,
    riid: REFIID,
    ppobj: *mut LPVOID,
) -> HRESULT {
    let This: *mut IDirectSoundCaptureNotifyImpl = iface as *mut IDirectSoundCaptureNotifyImpl;

    if (*This).dscb.is_null() {
        WARN!("invalid parameter\n");
        return E_INVALIDARG;
    }

    return IDirectSoundCaptureBuffer_QueryInterface(
        (*This).dscb as LPDIRECTSOUNDCAPTUREBUFFER,
        riid,
        ppobj,
    );
}

unsafe extern "system" fn IDirectSoundCaptureNotifyImpl_AddRef(iface: LPDIRECTSOUNDNOTIFY) -> ULONG {
    let This: *mut IDirectSoundCaptureNotifyImpl = iface as *mut IDirectSoundCaptureNotifyImpl;
    let ref_0: ULONG = InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_) as *mut i32) as ULONG;
    return ref_0;
}

unsafe extern "system" fn IDirectSoundCaptureNotifyImpl_Release(
    mut iface: LPDIRECTSOUNDNOTIFY,
) -> ULONG {
    let mut This: *mut IDirectSoundCaptureNotifyImpl = iface as *mut IDirectSoundCaptureNotifyImpl;
    let mut ref_0: ULONG =
        InterlockedDecrement(core::ptr::addr_of_mut!((*This).ref_) as *mut LONG) as ULONG;
    WINE_TRACE!("(%p) ref was %d\n", This, ref_0.wrapping_add(1 as ULONG));
    if ref_0 == 0 {
        if !(*(*This).dscb).hwnotify.is_null() {
            IDsDriverNotify_Release((*(*This).dscb).hwnotify);
        }
        (*(*This).dscb).notify = core::ptr::null_mut::<IDirectSoundCaptureNotifyImpl>();
        IDirectSoundCaptureBuffer_Release((*This).dscb as LPDIRECTSOUNDCAPTUREBUFFER);
        HeapFree(
            GetProcessHeap(),
            0,
            This as LPVOID,
        );
        WINE_TRACE!("(%p) released\n", This);
    }
    return ref_0;
}

unsafe extern "system" fn IDirectSoundCaptureNotifyImpl_SetNotificationPositions(
    iface: LPDIRECTSOUNDNOTIFY,
    howmuch: DWORD,
    notify: LPCDSBPOSITIONNOTIFY,
) -> HRESULT {
    let This: *mut IDirectSoundCaptureNotifyImpl = iface as *mut IDirectSoundCaptureNotifyImpl;

    if howmuch > 0 && notify.is_null() {
        WARN!("invalid parameter: notify == NULL\n");
        return DSERR_INVALIDPARAM;
    }

    if ((TRACE_ON!(__wine_dbch_dsound)) != 0) {
        let mut i: DWORD = 0;
        while i < howmuch {
            TRACE!(
                "notify at %d to %p\n",
                (*notify.add(i as usize)).dwOffset,
                (*notify.add(i as usize)).hEventNotify,
            );
            i = i.wrapping_add(1);
        }
    }

    if !(*(*This).dscb).hwnotify.is_null() {
        let mut hres: HRESULT = 0;
        hres = IDsDriverNotify_SetNotificationPositions((*(*This).dscb).hwnotify, howmuch, notify);
        if hres != (DS_OK as HRESULT) {
            WARN!("IDsDriverNotify_SetNotificationPositions failed\n");
        }
        return hres;
    } else if howmuch > 0 {
        /* Make an internal copy of the caller-supplied array.
         * Replace the existing copy if one is already present. */
        if !(*(*This).dscb).notifies.is_null() {
            (*(*This).dscb).notifies = HeapReAlloc(
                GetProcessHeap(),
                HEAP_ZERO_MEMORY,
                (*(*This).dscb).notifies as LPVOID,
                (howmuch as SIZE_T).wrapping_mul(core::mem::size_of::<DSBPOSITIONNOTIFY>() as SIZE_T),
            ) as LPDSBPOSITIONNOTIFY;
        } else {
            (*(*This).dscb).notifies = HeapAlloc(
                GetProcessHeap(),
                HEAP_ZERO_MEMORY,
                (howmuch as SIZE_T).wrapping_mul(core::mem::size_of::<DSBPOSITIONNOTIFY>() as SIZE_T),
            ) as LPDSBPOSITIONNOTIFY;
        }

        if (*(*This).dscb).notifies.is_null() {
            WARN!("out of memory\n");
            return DSERR_OUTOFMEMORY;
        }
        CopyMemory!(
            (*(*This).dscb).notifies as LPVOID,
            notify as LPCVOID,
            (howmuch as SIZE_T).wrapping_mul(core::mem::size_of::<DSBPOSITIONNOTIFY>() as SIZE_T),
        );
        (*(*This).dscb).nrofnotifies = howmuch as i32;
    } else {
        HeapFree(
            GetProcessHeap(),
            0,
            (*(*This).dscb).notifies as LPVOID,
        );
        (*(*This).dscb).notifies = core::ptr::null_mut();
        (*(*This).dscb).nrofnotifies = 0 as i32;
    }

    return S_OK;
}

unsafe extern "C" fn IDirectSoundCaptureNotifyImpl_Create(
    mut dscb: *mut IDirectSoundCaptureBufferImpl,
    mut pdscn: *mut *mut IDirectSoundCaptureNotifyImpl,
) -> HRESULT {
    let mut dscn: *mut IDirectSoundCaptureNotifyImpl = core::ptr::null_mut();

    dscn = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY as DWORD,
        core::mem::size_of::<IDirectSoundCaptureNotifyImpl>() as SIZE_T,
    ) as *mut IDirectSoundCaptureNotifyImpl;

    if dscn.is_null() {
        WARN!("out of memory\n");
        return DSERR_OUTOFMEMORY as HRESULT;
    }

    (*dscn).ref_ = 0 as LONG;
    (*dscn).lpVtbl = core::ptr::addr_of!(dscnvt);
    (*dscn).dscb = dscb;
    (*dscb).notify = dscn;
    IDirectSoundCaptureBuffer_AddRef(dscb as LPDIRECTSOUNDCAPTUREBUFFER);

    *pdscn = dscn;
    return DS_OK as HRESULT;
}
unsafe extern "system" fn IDirectSoundCaptureBufferImpl_QueryInterface(iface: LPDIRECTSOUNDCAPTUREBUFFER8, riid: *const IID, ppobj: *mut LPVOID) -> HRESULT {
    let This: *mut IDirectSoundCaptureBufferImpl = iface as *mut IDirectSoundCaptureBufferImpl;
    let mut hres: HRESULT;

    if ppobj.is_null() {
        WARN!("invalid parameter\n");
        return E_INVALIDARG;
    }

    *ppobj = core::ptr::null_mut();

    if IsEqualGUID!(core::ptr::addr_of!(IID_IDirectSoundNotify), riid) != false {
        if (*This).notify.is_null() {
            hres = IDirectSoundCaptureNotifyImpl_Create(This, core::ptr::addr_of_mut!((*This).notify));
        }
        if !(*This).notify.is_null() {
            IDirectSoundNotify_AddRef((*This).notify as LPDIRECTSOUNDNOTIFY);
            if !(*(*This).device).hwbuf.is_null() && (*This).hwnotify.is_null() {
                hres = IDsCaptureDriverBuffer_QueryInterface(
                    (*(*This).device).hwbuf,
                    core::ptr::addr_of!(IID_IDsDriverNotify),
                    core::ptr::addr_of_mut!((*This).hwnotify) as *mut LPVOID,
                );
                if hres != (DS_OK as HRESULT) {
                    WARN!("IDsCaptureDriverBuffer_QueryInterface failed\n");
                    IDirectSoundNotify_Release((*This).notify as LPDIRECTSOUNDNOTIFY);
                    *ppobj = core::ptr::null_mut();
                    return hres;
                }
            }

            *ppobj = (*This).notify as LPVOID;
            return (DS_OK) as HRESULT;
        }

        WARN!("IID_IDirectSoundNotify\n");
        return E_FAIL;
    }

    if IsEqualGUID!(core::ptr::addr_of!(IID_IDirectSoundCaptureBuffer), riid) != false
        || IsEqualGUID!(core::ptr::addr_of!(IID_IDirectSoundCaptureBuffer8), riid) != false
    {
        IDirectSoundCaptureBuffer8_AddRef(iface);
        *ppobj = This as LPVOID;
        return (NO_ERROR) as HRESULT;
    }

    return E_NOINTERFACE;
}

unsafe extern "system" fn IDirectSoundCaptureBufferImpl_AddRef(iface: LPDIRECTSOUNDCAPTUREBUFFER8) -> ULONG {
    let This: *mut IDirectSoundCaptureBufferImpl = iface as *mut IDirectSoundCaptureBufferImpl;
    let ref_0: ULONG = InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_) as *mut LONG) as ULONG;
    return ref_0;
}

unsafe extern "system" fn IDirectSoundCaptureBufferImpl_Release(iface: LPDIRECTSOUNDCAPTUREBUFFER8) -> ULONG {
    let This: *mut IDirectSoundCaptureBufferImpl = iface as *mut IDirectSoundCaptureBufferImpl;
    let mut ref_0: ULONG = InterlockedDecrement(core::ptr::addr_of_mut!((*This).ref_) as *mut LONG) as ULONG;

    if ref_0 == 0 {
        if (*(*This).device).state == STATE_CAPTURING {
            (*(*This).device).state = STATE_STOPPING;
        }

        HeapFree(GetProcessHeap(), 0, (*This).pdscbd as PVOID);

        if !(*(*This).device).hwi.is_null() {
            waveInReset((*(*This).device).hwi);
            waveInClose((*(*This).device).hwi);
            HeapFree(GetProcessHeap(), 0, (*(*This).device).pwave as PVOID);
            (*(*This).device).pwave = core::ptr::null_mut();
            (*(*This).device).hwi = core::ptr::null_mut();
        }

        if !(*(*This).device).hwbuf.is_null() {
            IDsCaptureDriverBuffer_Release((*(*This).device).hwbuf);
        }

        /* remove from DirectSoundCaptureDevice */
        (*(*This).device).capture_buffer = core::ptr::null_mut::<IDirectSoundCaptureBufferImpl>();

        if !(*This).notify.is_null() {
            IDirectSoundNotify_Release((*This).notify as LPDIRECTSOUNDNOTIFY);
        }

        /* If driver manages its own buffer, IDsCaptureDriverBuffer_Release
           should have freed the buffer. Prevent freeing it again in
           IDirectSoundCaptureBufferImpl_Create */
        if (*(*This).device).drvdesc.dwFlags & DSDDESC_USESYSTEMMEMORY == 0 {
            (*(*This).device).buffer = core::ptr::null_mut::<BYTE>();
        }

        HeapFree(GetProcessHeap(), 0, (*This).notifies as PVOID);
        HeapFree(GetProcessHeap(), 0, This as PVOID);
    }
    return ref_0;
}

unsafe extern "system" fn IDirectSoundCaptureBufferImpl_GetCaps(iface: LPDIRECTSOUNDCAPTUREBUFFER8, lpDSCBCaps: LPDSCBCAPS) -> HRESULT {
    let This: *mut IDirectSoundCaptureBufferImpl = iface as *mut IDirectSoundCaptureBufferImpl;

    TRACE!("(%p,%p)\n", This, lpDSCBCaps);

    if lpDSCBCaps.is_null() {
        WARN!("invalid parameter: lpDSCBCaps == NULL\n");
        return DSERR_INVALIDPARAM;
    }

    if ((*lpDSCBCaps).dwSize as usize) < core::mem::size_of::<DSCBCAPS>() {
        WARN!("invalid parameter: lpDSCBCaps->dwSize = %d\n", (*lpDSCBCaps).dwSize);
        return DSERR_INVALIDPARAM;
    }

    if (*This).device.is_null() {
        WARN!("invalid parameter: This->device == NULL\n");
        return DSERR_INVALIDPARAM;
    }

    (*lpDSCBCaps).dwSize = core::mem::size_of::<DSCBCAPS>() as DWORD;
    (*lpDSCBCaps).dwFlags = (*This).flags;
    (*lpDSCBCaps).dwBufferBytes = (*(*This).pdscbd).dwBufferBytes;
    (*lpDSCBCaps).dwReserved = 0;

    TRACE!("returning DS_OK\n");
    return (DS_OK) as HRESULT;
}

unsafe extern "system" fn IDirectSoundCaptureBufferImpl_GetCurrentPosition(iface: LPDIRECTSOUNDCAPTUREBUFFER8, lpdwCapturePosition: LPDWORD, lpdwReadPosition: LPDWORD) -> HRESULT {
    let This: *mut IDirectSoundCaptureBufferImpl = iface as *mut IDirectSoundCaptureBufferImpl;
    let mut hres: HRESULT = (DS_OK as i32);

    if (*This).device.is_null() {
        WARN!("invalid parameter: This->device == NULL\n");
        return DSERR_INVALIDPARAM;
    }

    if !(*(*This).device).driver.is_null() {
        hres = IDsCaptureDriverBuffer_GetPosition((*(*This).device).hwbuf, lpdwCapturePosition, lpdwReadPosition);
        if hres != (DS_OK as HRESULT) {
            WARN!("IDsCaptureDriverBuffer_GetPosition failed\n");
        }
    } else if !(*(*This).device).hwi.is_null() {
        let mut pos: DWORD = 0;

        EnterCriticalSection(core::ptr::addr_of_mut!((*(*This).device).lock));
        pos = ((*(*(*This).device).pwave.offset((*(*This).device).index as isize)).lpData as DWORD_PTR)
            .wrapping_sub((*(*This).device).buffer as DWORD_PTR) as DWORD;
        if !lpdwCapturePosition.is_null() {
            *lpdwCapturePosition = (*(*(*This).device).pwave.offset((*(*This).device).index as isize))
                .dwBufferLength
                .wrapping_add(pos)
                .wrapping_rem((*(*This).device).buflen);
        }
        if !lpdwReadPosition.is_null() {
            *lpdwReadPosition = pos;
        }
        LeaveCriticalSection(core::ptr::addr_of_mut!((*(*This).device).lock));

    } else {
        WARN!("no driver\n");
        hres = DSERR_NODRIVER;
    }

    return hres;
}

unsafe extern "system" fn IDirectSoundCaptureBufferImpl_GetFormat(iface: LPDIRECTSOUNDCAPTUREBUFFER8, lpwfxFormat: LPWAVEFORMATEX, dwSizeAllocated: DWORD, lpdwSizeWritten: LPDWORD) -> HRESULT {
    let mut dwSizeAllocated = dwSizeAllocated;
    let This: *mut IDirectSoundCaptureBufferImpl = iface as *mut IDirectSoundCaptureBufferImpl;
    let mut hres: HRESULT = (DS_OK as i32);

    if (*This).device.is_null() {
        WARN!("invalid parameter: This->device == NULL\n");
        return DSERR_INVALIDPARAM;
    }

    if (dwSizeAllocated as usize)
        > (core::mem::size_of::<WAVEFORMATEX>()).wrapping_add((*(*(*This).device).pwfx).cbSize as usize)
    {
        dwSizeAllocated = (core::mem::size_of::<WAVEFORMATEX>())
            .wrapping_add((*(*(*This).device).pwfx).cbSize as usize) as DWORD;
    }

    if !lpwfxFormat.is_null() {
        CopyMemory!(
            lpwfxFormat as *mut core::ffi::c_void,
            (*(*This).device).pwfx as *const core::ffi::c_void,
            dwSizeAllocated as usize,
        );
        if !lpdwSizeWritten.is_null() {
            *lpdwSizeWritten = dwSizeAllocated;
        }
    } else {
        if !lpdwSizeWritten.is_null() {
            *lpdwSizeWritten = (core::mem::size_of::<WAVEFORMATEX>())
                .wrapping_add((*(*(*This).device).pwfx).cbSize as usize) as DWORD;
        } else {
            hres = DSERR_INVALIDPARAM;
        }
    }

    return hres;
}

unsafe extern "system" fn IDirectSoundCaptureBufferImpl_GetStatus(iface: LPDIRECTSOUNDCAPTUREBUFFER8, lpdwStatus: LPDWORD) -> HRESULT {
    let This: *mut IDirectSoundCaptureBufferImpl = iface as *mut IDirectSoundCaptureBufferImpl;
    TRACE!("(%p, %p), thread is %04x\n", This, lpdwStatus, GetCurrentThreadId());

    if (*This).device.is_null() {
        WARN!("invalid parameter: This->device == NULL\n");
        return DSERR_INVALIDPARAM;
    }

    if lpdwStatus.is_null() {
        WARN!("invalid parameter: lpdwStatus == NULL\n");
        return DSERR_INVALIDPARAM;
    }

    *lpdwStatus = 0;
    EnterCriticalSection(core::ptr::addr_of_mut!((*(*This).device).lock));

    if ((*(*This).device).state == STATE_STARTING)
        || ((*(*This).device).state == STATE_CAPTURING)
    {
        *lpdwStatus |= DSCBSTATUS_CAPTURING;
        if ((*This).flags & DSCBSTART_LOOPING) != 0 {
            *lpdwStatus |= DSCBSTATUS_LOOPING;
        }
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*(*This).device).lock));

    return (DS_OK) as HRESULT;
}

unsafe extern "system" fn IDirectSoundCaptureBufferImpl_Initialize(mut iface: LPDIRECTSOUNDCAPTUREBUFFER8, mut lpDSC: LPDIRECTSOUNDCAPTURE, mut lpcDSCBDesc: LPCDSCBUFFERDESC) -> HRESULT {
    let mut This: *mut IDirectSoundCaptureBufferImpl = iface as *mut IDirectSoundCaptureBufferImpl;
    WINE_FIXME!("(%p,%p,%p): stub\n", This, lpDSC, lpcDSCBDesc);
    return DS_OK as HRESULT;
}

unsafe extern "system" fn IDirectSoundCaptureBufferImpl_Lock(iface: LPDIRECTSOUNDCAPTUREBUFFER8, dwReadCusor: DWORD, dwReadBytes: DWORD, lplpvAudioPtr1: *mut LPVOID, lpdwAudioBytes1: LPDWORD, lplpvAudioPtr2: *mut LPVOID, lpdwAudioBytes2: LPDWORD, dwFlags: DWORD) -> HRESULT {
    let mut hres: HRESULT = (DS_OK as i32);
    let This: *mut IDirectSoundCaptureBufferImpl = iface as *mut IDirectSoundCaptureBufferImpl;

    if (*This).device.is_null() {
        WARN!("invalid parameter: This->device == NULL\n");
        return DSERR_INVALIDPARAM;
    }

    if lplpvAudioPtr1.is_null() {
        WARN!("invalid parameter: lplpvAudioPtr1 == NULL\n");
        return DSERR_INVALIDPARAM;
    }

    if lpdwAudioBytes1.is_null() {
        WARN!("invalid parameter: lpdwAudioBytes1 == NULL\n");
        return DSERR_INVALIDPARAM;
    }

    EnterCriticalSection(&raw mut (*(*This).device).lock);

    if !(*(*This).device).driver.is_null() {
        hres = IDsCaptureDriverBuffer_Lock(
            (*(*This).device).hwbuf,
            lplpvAudioPtr1,
            lpdwAudioBytes1,
            lplpvAudioPtr2,
            lpdwAudioBytes2,
            dwReadCusor,
            dwReadBytes,
            dwFlags,
        );
        if hres != (DS_OK as HRESULT) {
            WARN!("IDsCaptureDriverBuffer_Lock failed\n");
        }
    } else if !(*(*This).device).hwi.is_null() {
        *lplpvAudioPtr1 = (*(*This).device).buffer.offset(dwReadCusor as isize) as LPVOID;
        if dwReadCusor.wrapping_add(dwReadBytes) > (*(*This).device).buflen {
            *lpdwAudioBytes1 = (*(*This).device).buflen.wrapping_sub(dwReadCusor);
            if !lplpvAudioPtr2.is_null() {
                *lplpvAudioPtr2 = (*(*This).device).buffer as LPVOID;
            }
            if !lpdwAudioBytes2.is_null() {
                *lpdwAudioBytes2 = dwReadBytes.wrapping_sub(*lpdwAudioBytes1);
            }
        } else {
            *lpdwAudioBytes1 = dwReadBytes;
            if !lplpvAudioPtr2.is_null() {
                *lplpvAudioPtr2 = core::ptr::null_mut();
            }
            if !lpdwAudioBytes2.is_null() {
                *lpdwAudioBytes2 = 0;
            }
        }
    } else {
        hres = DSERR_INVALIDCALL;
    }

    LeaveCriticalSection(&raw mut (*(*This).device).lock);

    return hres;
}

unsafe extern "system" fn IDirectSoundCaptureBufferImpl_Start(iface: LPDIRECTSOUNDCAPTUREBUFFER8, dwFlags: DWORD) -> HRESULT {
    let mut hres: HRESULT = (DS_OK as i32);
    let This: *mut IDirectSoundCaptureBufferImpl = iface as *mut IDirectSoundCaptureBufferImpl;

    if (*This).device.is_null() {
        WARN!("invalid parameter: This->device == NULL\n");
        return DSERR_INVALIDPARAM;
    }

    if ((*(*This).device).driver as usize == 0 as usize)
        && ((*(*This).device).hwi as usize == 0 as usize)
    {
        WARN!("no driver\n");
        return DSERR_NODRIVER;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*(*This).device).lock));

    (*This).flags = dwFlags;

    if (*(*This).device).state == STATE_STOPPED {
        (*(*This).device).state = STATE_STARTING;
    } else if (*(*This).device).state == STATE_STOPPING {
        (*(*This).device).state = STATE_CAPTURING;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*(*This).device).lock));

    if !(*(*This).device).driver.is_null() {
        hres = IDsCaptureDriverBuffer_Start((*(*This).device).hwbuf, dwFlags);
        if hres != (DS_OK as HRESULT) {
            WARN!("IDsCaptureDriverBuffer_Start failed\n");
        }
    } else if !(*(*This).device).hwi.is_null() {
        let device: *mut DirectSoundCaptureDevice = (*This).device;

        if !(*device).buffer.is_null() {
            let mut c: i32 = 0;
            let blocksize: DWORD = DSOUND_fraglen(
                (*(*device).pwfx).nSamplesPerSec,
                (*(*device).pwfx).nBlockAlign as DWORD,
            );
            (*device).nrofpwaves = ((*device).buflen / blocksize
                + (((*device).buflen % blocksize) != 0) as DWORD) as i32;

            /* prepare headers */
            if !(*device).pwave.is_null() {
                (*device).pwave = HeapReAlloc(
                    GetProcessHeap(),
                    0 as DWORD,
                    (*device).pwave as LPVOID,
                    ((*device).nrofpwaves as SIZE_T)
                        .wrapping_mul(core::mem::size_of::<WAVEHDR>() as SIZE_T),
                ) as LPWAVEHDR;
            } else {
                (*device).pwave = HeapAlloc(
                    GetProcessHeap(),
                    0 as DWORD,
                    ((*device).nrofpwaves as SIZE_T)
                        .wrapping_mul(core::mem::size_of::<WAVEHDR>() as SIZE_T),
                ) as LPWAVEHDR;
            }

            c = 0;
            while c < (*device).nrofpwaves {
                (*(*device).pwave.offset(c as isize)).lpData = ((*device).buffer as *mut i8)
                    .offset((c as DWORD).wrapping_mul(blocksize) as isize)
                    as LPSTR;
                if c + 1 == (*device).nrofpwaves {
                    (*(*device).pwave.offset(c as isize)).dwBufferLength = (*device)
                        .buflen
                        .wrapping_sub((c as DWORD).wrapping_mul(blocksize));
                } else {
                    (*(*device).pwave.offset(c as isize)).dwBufferLength = blocksize;
                }
                (*(*device).pwave.offset(c as isize)).dwBytesRecorded = 0;
                (*(*device).pwave.offset(c as isize)).dwUser = device as DWORD_PTR;
                (*(*device).pwave.offset(c as isize)).dwFlags = 0;
                (*(*device).pwave.offset(c as isize)).dwLoops = 0;
                hres = mmErr(waveInPrepareHeader(
                    (*device).hwi,
                    (*device).pwave.offset(c as isize) as LPWAVEHDR,
                    core::mem::size_of::<WAVEHDR>() as UINT,
                ) as UINT);
                if hres != (DS_OK as HRESULT) {
                    WARN!("waveInPrepareHeader failed\n");
                    loop {
                        let fresh = c;
                        c = c - 1;
                        if fresh == 0 {
                            break;
                        }
                        waveInUnprepareHeader(
                            (*device).hwi,
                            (*device).pwave.offset(c as isize) as LPWAVEHDR,
                            core::mem::size_of::<WAVEHDR>() as UINT,
                        );
                    }
                    break;
                }

                hres = mmErr(waveInAddBuffer(
                    (*device).hwi,
                    (*device).pwave.offset(c as isize) as LPWAVEHDR,
                    core::mem::size_of::<WAVEHDR>() as UINT,
                ) as UINT);
                if hres != (DS_OK as HRESULT) {
                    WARN!("waveInAddBuffer failed\n");
                    loop {
                        let fresh = c;
                        c = c - 1;
                        if fresh == 0 {
                            break;
                        }
                        waveInUnprepareHeader(
                            (*device).hwi,
                            (*device).pwave.offset(c as isize) as LPWAVEHDR,
                            core::mem::size_of::<WAVEHDR>() as UINT,
                        );
                    }
                    break;
                }
                c += 1;
            }

            FillMemory!(
                (*device).buffer,
                (*device).buflen as SIZE_T,
                (if (*(*device).pwfx).wBitsPerSample as i32 == 8 {
                    128
                } else {
                    0
                }) as u8,
            );
        }

        (*device).index = 0;

        if hres == (DS_OK as HRESULT) {
            /* start filling the first buffer */
            hres = mmErr(waveInStart((*device).hwi) as UINT);
            if hres != (DS_OK as HRESULT) {
                WARN!("waveInStart failed\n");
            }
        }

        if hres != (DS_OK as HRESULT) {
            WARN!("calling waveInClose because of error\n");
            waveInClose((*device).hwi);
            (*device).hwi = 0 as usize as *mut HWAVEIN__;
        }
    } else {
        WARN!("no driver\n");
        hres = DSERR_NODRIVER;
    }

    return hres;
}
unsafe extern "system" fn IDirectSoundCaptureBufferImpl_Stop(iface: LPDIRECTSOUNDCAPTUREBUFFER8) -> HRESULT {
    let mut hres: HRESULT = (DS_OK as i32);
    let This: *mut IDirectSoundCaptureBufferImpl = iface as *mut IDirectSoundCaptureBufferImpl;

    if (*This).device.is_null() {
        WARN!("invalid parameter: This->device == NULL\n");
        return DSERR_INVALIDPARAM;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*(*This).device).lock));

    if (*(*This).device).state == STATE_CAPTURING {
        (*(*This).device).state = STATE_STOPPING;
    } else if (*(*This).device).state == STATE_STARTING {
        (*(*This).device).state = STATE_STOPPED;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*(*This).device).lock));

    if !(*(*This).device).driver.is_null() {
        hres = IDsCaptureDriverBuffer_Stop((*(*This).device).hwbuf);
        if hres != (DS_OK as HRESULT) {
            WARN!("IDsCaptureDriverBuffer_Stop() failed\n");
        }
    } else if !(*(*This).device).hwi.is_null() {
        hres = mmErr(waveInReset((*(*This).device).hwi) as u32);
        if hres != (DS_OK as HRESULT) {
            WARN!("waveInReset() failed\n");
        }
    } else {
        WARN!("no driver\n");
        hres = DSERR_NODRIVER;
    }

    return hres;
}

unsafe extern "system" fn IDirectSoundCaptureBufferImpl_Unlock(iface: LPDIRECTSOUNDCAPTUREBUFFER8, lpvAudioPtr1: LPVOID, dwAudioBytes1: DWORD, lpvAudioPtr2: LPVOID, dwAudioBytes2: DWORD) -> HRESULT {
    let mut hres: HRESULT = (DS_OK as i32);
    let This: *mut IDirectSoundCaptureBufferImpl = iface as *mut IDirectSoundCaptureBufferImpl;

    if lpvAudioPtr1.is_null() {
        WARN!("invalid parameter: lpvAudioPtr1 == NULL\n");
        return DSERR_INVALIDPARAM;
    }

    if !(*(*This).device).driver.is_null() {
        hres = IDsCaptureDriverBuffer_Unlock(
            (*(*This).device).hwbuf as *mut IDsCaptureDriverBuffer,
            lpvAudioPtr1,
            dwAudioBytes1,
            lpvAudioPtr2,
            dwAudioBytes2,
        );
        if hres != (DS_OK as HRESULT) {
            WARN!("IDsCaptureDriverBuffer_Unlock failed\n");
        }
    } else if (*(*This).device).hwi.is_null() {
        WARN!("invalid call\n");
        hres = DSERR_INVALIDCALL;
    }

    return hres;
}
unsafe extern "system" fn IDirectSoundCaptureBufferImpl_GetObjectInPath(iface: LPDIRECTSOUNDCAPTUREBUFFER8, rguidObject: *const GUID, dwIndex: DWORD, rguidInterface: *const GUID, ppObject: *mut LPVOID) -> HRESULT {
    let mut This: *mut IDirectSoundCaptureBufferImpl = iface as *mut IDirectSoundCaptureBufferImpl;
    WINE_FIXME!("(%p,%s,%u,%s,%p): stub\n", This, debugstr_guid(rguidObject as *const _GUID), dwIndex, debugstr_guid(rguidInterface as *const _GUID), ppObject);
    return DS_OK as HRESULT;
}
unsafe extern "system" fn IDirectSoundCaptureBufferImpl_GetFXStatus(mut iface: LPDIRECTSOUNDCAPTUREBUFFER8, mut dwFXCount: DWORD, mut pdwFXStatus: LPDWORD) -> HRESULT {
    let mut This: *mut IDirectSoundCaptureBufferImpl = iface as *mut IDirectSoundCaptureBufferImpl;

    return (DS_OK) as HRESULT;
}

unsafe extern "C" fn capture_CheckNotify(
    mut This: *mut IDirectSoundCaptureBufferImpl,
    mut from: DWORD,
    mut len: DWORD,
) {
    let mut i: ::core::ffi::c_int = 0;
    i = 0 as ::core::ffi::c_int;
    while i < (*This).nrofnotifies {
        let mut event: LPDSBPOSITIONNOTIFY = (*This).notifies.offset(i as isize);
        let mut offset: DWORD = (*event).dwOffset;
        WINE_TRACE!("checking %d, position %d, event = %p\n", i, offset, (*event).hEventNotify);
        if offset == DSBPN_OFFSETSTOP as DWORD {
            if from == 0 && len == 0 {
                SetEvent((*event).hEventNotify);
                WINE_TRACE!("signalled event %p (%d)\n", (*event).hEventNotify, i);
                return;
            } else {
                return;
            }
        }
        if offset >= from && offset < from.wrapping_add(len) {
            WINE_TRACE!("signalled event %p (%d)\n", (*event).hEventNotify, i);
            SetEvent((*event).hEventNotify);
        }
        i += 1;
    }
}

unsafe extern "system" fn DSOUND_capture_callback(hwi: HWAVEIN, msg: UINT, dwUser: DWORD_PTR, dw1: DWORD_PTR, dw2: DWORD_PTR) {
    let This: *mut DirectSoundCaptureDevice = dwUser as *mut DirectSoundCaptureDevice;
    let Moi: *mut IDirectSoundCaptureBufferImpl = (*This).capture_buffer;

    if msg == MM_WIM_DATA as UINT {
        EnterCriticalSection(core::ptr::addr_of_mut!((*This).lock));
        if (*This).state != STATE_STOPPED as DWORD {
            let index = (*This).index;
            if (*This).state == STATE_STARTING as DWORD {
                (*This).state = STATE_CAPTURING as DWORD;
            }
            capture_CheckNotify(
                Moi,
                ((*(*This).pwave.offset(index as isize)).lpData as DWORD_PTR)
                    .wrapping_sub((*This).buffer as DWORD_PTR) as DWORD,
                (*(*This).pwave.offset(index as isize)).dwBufferLength,
            );
            (*This).index = ((*This).index + 1) % (*This).nrofpwaves;
            if ((*This).index == 0)
                && ((*(*This).capture_buffer).flags & DSCBSTART_LOOPING as DWORD) == 0
            {
                (*This).state = STATE_STOPPED as DWORD;
                capture_CheckNotify(
                    Moi,
                    0 as DWORD,
                    0 as DWORD,
                );
            } else {
                if (*This).state == STATE_CAPTURING as DWORD {
                    waveInUnprepareHeader(
                        hwi,
                        (*This).pwave.offset(index as isize) as LPWAVEHDR,
                        core::mem::size_of::<WAVEHDR>() as UINT,
                    );
                    waveInPrepareHeader(
                        hwi,
                        (*This).pwave.offset(index as isize) as LPWAVEHDR,
                        core::mem::size_of::<WAVEHDR>() as UINT,
                    );
                    waveInAddBuffer(
                        hwi,
                        (*This).pwave.offset(index as isize) as LPWAVEHDR,
                        core::mem::size_of::<WAVEHDR>() as UINT,
                    );
                } else if (*This).state == STATE_STOPPING as DWORD {
                    (*This).state = STATE_STOPPED as DWORD;
                }
            }
        }
        LeaveCriticalSection(core::ptr::addr_of_mut!((*This).lock));
    }
}

unsafe extern "C" fn IDirectSoundCaptureBufferImpl_Create(
    device: *mut DirectSoundCaptureDevice,
    ppobj: *mut *mut IDirectSoundCaptureBufferImpl,
    lpcDSCBufferDesc: LPCDSCBUFFERDESC,
) -> HRESULT {
    let mut wfex: LPWAVEFORMATEX = core::ptr::null_mut();

    if ppobj.is_null() {
        WARN!("invalid parameter: ppobj == NULL\n");
        return DSERR_INVALIDPARAM;
    }

    if device.is_null() {
        WARN!("not initialized\n");
        *ppobj = core::ptr::null_mut();
        return DSERR_UNINITIALIZED;
    }

    if lpcDSCBufferDesc.is_null() {
        WARN!("invalid parameter: lpcDSCBufferDesc == NULL\n");
        *ppobj = core::ptr::null_mut();
        return DSERR_INVALIDPARAM;
    }

    if (((*lpcDSCBufferDesc).dwSize != (core::mem::size_of::<DSCBUFFERDESC>()) as _)
        && ((*lpcDSCBufferDesc).dwSize != (core::mem::size_of::<DSCBUFFERDESC1>()) as _))
        || ((*lpcDSCBufferDesc).dwBufferBytes == 0)
        || (*lpcDSCBufferDesc).lpwfxFormat.is_null()
    {
        /* FIXME: DSERR_BADFORMAT ? */
        WARN!("invalid lpcDSCBufferDesc\n");
        *ppobj = core::ptr::null_mut();
        return DSERR_INVALIDPARAM;
    }

    wfex = (*lpcDSCBufferDesc).lpwfxFormat;

    /* Do some sanity checks for 'recording' SamplesPerSec value */
    if (*wfex).nSamplesPerSec > 100000 {
        (*wfex).nSamplesPerSec = 100000;
    }
    if (*wfex).nSamplesPerSec < 5000 {
        (*wfex).nSamplesPerSec = 5000;
    }

    (*device).pwfx = DSOUND_CopyFormat(wfex as LPCWAVEFORMATEX) as PWAVEFORMATEX;
    if (*device).pwfx.is_null() {
        *ppobj = core::ptr::null_mut();
        return DSERR_OUTOFMEMORY;
    }

    *ppobj = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        core::mem::size_of::<IDirectSoundCaptureBufferImpl>() as SIZE_T,
    ) as *mut IDirectSoundCaptureBufferImpl;

    if (*ppobj).is_null() {
        WARN!("out of memory\n");
        *ppobj = core::ptr::null_mut();
        return DSERR_OUTOFMEMORY;
    } else {
        let mut err: HRESULT = (DS_OK as i32);
        let mut newbuf: LPBYTE = core::ptr::null_mut();
        let mut buflen: DWORD;
        let This: *mut IDirectSoundCaptureBufferImpl = *ppobj;

        (*This).ref_ = 1;
        (*This).device = device;
        (*(*This).device).capture_buffer = This;
        (*This).notify = core::ptr::null_mut();
        (*This).nrofnotifies = 0;
        (*This).hwnotify = core::ptr::null_mut();

        (*This).pdscbd = HeapAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY,
            (*lpcDSCBufferDesc).dwSize as SIZE_T,
        ) as LPDSCBUFFERDESC;
        if !(*This).pdscbd.is_null() {
            CopyMemory!(
                (*This).pdscbd as *mut core::ffi::c_void,
                lpcDSCBufferDesc as *const core::ffi::c_void,
                (*lpcDSCBufferDesc).dwSize as usize,
            );
        } else {
            WARN!("no memory\n");
            (*(*This).device).capture_buffer = core::ptr::null_mut();
            HeapFree(
                GetProcessHeap(),
                0,
                This as LPVOID,
            );
            *ppobj = core::ptr::null_mut();
            return DSERR_OUTOFMEMORY;
        }

        (*This).lpVtbl = core::ptr::addr_of!(dscbvt);

        if !(*device).driver.is_null() {
            if ((*(*This).device).drvdesc.dwFlags & DSDDESC_DOMMSYSTEMOPEN) != 0 {
                FIXME!("DSDDESC_DOMMSYSTEMOPEN not supported\n");
            }

            if ((*(*This).device).drvdesc.dwFlags & DSDDESC_USESYSTEMMEMORY) != 0 {
                /* allocate buffer from system memory */
                buflen = (*lpcDSCBufferDesc).dwBufferBytes;

                if !(*device).buffer.is_null() {
                    newbuf = HeapReAlloc(
                        GetProcessHeap(),
                        0,
                        (*device).buffer as LPVOID,
                        buflen as SIZE_T,
                    ) as LPBYTE;
                } else {
                    newbuf = HeapAlloc(
                        GetProcessHeap(),
                        0,
                        buflen as SIZE_T,
                    ) as LPBYTE;
                }

                if newbuf.is_null() {
                    WARN!("failed to allocate capture buffer\n");
                    err = DSERR_OUTOFMEMORY;
                    /* but the old buffer might still exist and must be re-prepared */
                } else {
                    (*device).buffer = newbuf;
                    (*device).buflen = buflen;
                }
            } else {
                /* let driver allocate memory */
                (*device).buflen = (*lpcDSCBufferDesc).dwBufferBytes;
                /* FIXME: */
                HeapFree(
                    GetProcessHeap(),
                    0,
                    (*device).buffer as LPVOID,
                );
                (*device).buffer = core::ptr::null_mut();
            }

            err = IDsCaptureDriver_CreateCaptureBuffer(
                (*device).driver,
                (*device).pwfx,
                0,
                0,
                core::ptr::addr_of_mut!((*device).buflen),
                core::ptr::addr_of_mut!((*device).buffer),
                core::ptr::addr_of_mut!((*device).hwbuf) as *mut LPVOID,
            );
            if err != (DS_OK as HRESULT) {
                WARN!("IDsCaptureDriver_CreateCaptureBuffer failed\n");
                (*(*This).device).capture_buffer = core::ptr::null_mut();
                HeapFree(
                    GetProcessHeap(),
                    0,
                    This as LPVOID,
                );
                *ppobj = core::ptr::null_mut();
                return err;
            }
        } else {
            let mut flags: DWORD = (CALLBACK_FUNCTION | WAVE_MAPPED) as DWORD;
            err = mmErr(waveInOpen(
                core::ptr::addr_of_mut!((*device).hwi),
                (*device).drvdesc.dnDevNode,
                (*device).pwfx as LPCWAVEFORMATEX,
                DSOUND_capture_callback as DWORD_PTR,
                device as DWORD_PTR,
                flags,
            )) as HRESULT;
            if err != (DS_OK as HRESULT) {
                WARN!("waveInOpen failed\n");
                (*(*This).device).capture_buffer = core::ptr::null_mut();
                HeapFree(
                    GetProcessHeap(),
                    0,
                    This as LPVOID,
                );
                *ppobj = core::ptr::null_mut();
                return err;
            }

            buflen = (*lpcDSCBufferDesc).dwBufferBytes;

            if !(*device).buffer.is_null() {
                newbuf = HeapReAlloc(
                    GetProcessHeap(),
                    0,
                    (*device).buffer as LPVOID,
                    buflen as SIZE_T,
                ) as LPBYTE;
            } else {
                newbuf = HeapAlloc(
                    GetProcessHeap(),
                    0,
                    buflen as SIZE_T,
                ) as LPBYTE;
            }
            if newbuf.is_null() {
                WARN!("failed to allocate capture buffer\n");
                err = DSERR_OUTOFMEMORY;
                /* but the old buffer might still exist and must be re-prepared */
            } else {
                (*device).buffer = newbuf;
                (*device).buflen = buflen;
            }
        }
    }

    return (DS_OK) as HRESULT;
}

unsafe extern "C" fn DirectSoundCaptureDevice_Create(
    ppDevice: *mut *mut DirectSoundCaptureDevice,
) -> HRESULT {
    let mut device: *mut DirectSoundCaptureDevice = core::ptr::null_mut();

    /* Allocate memory */
    device = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY as DWORD,
        core::mem::size_of::<DirectSoundCaptureDevice>() as SIZE_T,
    ) as *mut DirectSoundCaptureDevice;

    if device.is_null() {
        WARN!("out of memory\n");
        return DSERR_OUTOFMEMORY as HRESULT;
    }

    (*device).ref_ = 1 as LONG;
    (*device).state = STATE_STOPPED as DWORD;

    InitializeCriticalSection(core::ptr::addr_of_mut!((*device).lock));
    (*(*device).lock.DebugInfo).__bindgen_anon_1.Spare[0 as usize] =
        concat!(file!(), ": DirectSoundCaptureDevice.lock\0")
            .as_ptr() as *const ::core::ffi::c_char as DWORD_PTR;

    *ppDevice = device;

    return DS_OK as HRESULT;
}

unsafe extern "C" fn DirectSoundCaptureDevice_Release(
    device: *mut DirectSoundCaptureDevice,
) -> ULONG {
    let mut ref_0: ULONG = InterlockedDecrement(
        core::ptr::addr_of_mut!((*device).ref_) as *mut LONG,
    ) as ULONG;

    if ref_0 == 0 {
        if !(*device).capture_buffer.is_null() {
            IDirectSoundCaptureBufferImpl_Release(
                (*device).capture_buffer as LPDIRECTSOUNDCAPTUREBUFFER8,
            );
        }

        if !(*device).driver.is_null() {
            IDsCaptureDriver_Close((*device).driver);
            IDsCaptureDriver_Release((*device).driver);
        }

        HeapFree(
            GetProcessHeap(),
            0,
            (*device).pwfx as LPVOID,
        );
        (*(*device).lock.DebugInfo).__bindgen_anon_1.Spare[0 as usize] =
            0 as DWORD_PTR;
        DeleteCriticalSection(core::ptr::addr_of_mut!((*device).lock));
        DSOUND_capture[(*device).drvdesc.dnDevNode as usize] =
            core::ptr::null_mut::<DirectSoundCaptureDevice>();
        HeapFree(
            GetProcessHeap(),
            0,
            device as LPVOID,
        );
    }
    return ref_0;
}

unsafe extern "C" fn DirectSoundCaptureDevice_Initialize(
    ppDevice: *mut *mut DirectSoundCaptureDevice,
    mut lpcGUID: LPCGUID,
) -> HRESULT {
    let mut err: HRESULT = DSERR_INVALIDPARAM;
    let mut wid: ::core::ffi::c_uint;
    let mut widn: ::core::ffi::c_uint;
    let mut found: BOOLEAN = (FALSE as u8);
    let mut devGUID: GUID = core::mem::zeroed();
    let mut device: *mut DirectSoundCaptureDevice = *ppDevice;

    /* Default device? */
    if lpcGUID.is_null() || IsEqualGUID!(lpcGUID, core::ptr::addr_of!(GUID_NULL)) != false {
        lpcGUID = core::ptr::addr_of!(DSDEVID_DefaultCapture) as LPCGUID;
    }

    if GetDeviceID(lpcGUID, core::ptr::addr_of_mut!(devGUID)) != (DS_OK as i32) {
        WARN!("invalid parameter: lpcGUID\n");
        return DSERR_INVALIDPARAM;
    }

    widn = waveInGetNumDevs() as ::core::ffi::c_uint;
    if widn == 0 {
        WARN!("no audio devices found\n");
        return DSERR_NODRIVER;
    }

    /* enumerate WINMM audio devices and find the one we want */
    wid = 0;
    while wid < widn {
        if IsEqualGUID!(
            core::ptr::addr_of_mut!(devGUID),
            core::ptr::addr_of_mut!(DSOUND_capture_guids[wid as usize]),
        ) != false
        {
            found = (TRUE as u8);
            break;
        }
        wid = wid.wrapping_add(1);
    }

    if found == (FALSE as BOOLEAN) {
        WARN!("No device found matching given ID!\n");
        return DSERR_NODRIVER;
    }

    if !DSOUND_capture[wid as usize].is_null() {
        WARN!("already in use\n");
        return DSERR_ALLOCATED;
    }

    err = DirectSoundCaptureDevice_Create(core::ptr::addr_of_mut!(device));
    if err != (DS_OK as HRESULT) {
        WARN!("DirectSoundCaptureDevice_Create failed\n");
        return err;
    }

    *ppDevice = device;
    (*device).guid = devGUID;

    /* Disable the direct sound driver to force emulation if requested. */
    (*device).driver = core::ptr::null_mut();
    if ds_hw_accel != (DS_HW_ACCEL_EMULATION as i32) {
        err = mmErr(waveInMessage(
            (UlongToHandle!(wid as ::core::ffi::c_ulong)).cast(),
            DRV_QUERYDSOUNDIFACE,
            core::ptr::addr_of_mut!((*device).driver) as DWORD_PTR,
            0,
        ) as UINT);
        if err != (DS_OK as HRESULT) && err != DSERR_UNSUPPORTED {
            WARN!("waveInMessage failed; err=%x\n", err);
            return err;
        }
    }
    err = (DS_OK as i32);

    /* Get driver description */
    if !(*device).driver.is_null() {
        err = (*(*(*device).driver).lpVtbl)
            .GetDriverDesc
            .unwrap_unchecked()(
            (*device).driver as *mut IDsCaptureDriver,
            core::ptr::addr_of_mut!((*device).drvdesc),
        );
        if err != (DS_OK as HRESULT) {
            WARN!("IDsCaptureDriver_GetDriverDesc failed\n");
            return err;
        }
    } else {
        /* if no DirectSound interface available, use WINMM API instead */
        (*device).drvdesc.dwFlags = DSDDESC_DOMMSYSTEMOPEN | DSDDESC_DOMMSYSTEMSETFORMAT;
    }

    (*device).drvdesc.dnDevNode = wid as DWORD;

    /* open the DirectSound driver if available */
    if !(*device).driver.is_null() && err == (DS_OK as HRESULT) {
        err = (*(*(*device).driver).lpVtbl)
            .Open
            .unwrap_unchecked()(
            (*device).driver as *mut IDsCaptureDriver
        );
    }

    if err == (DS_OK as HRESULT) {
        *ppDevice = device;

        /* the driver is now open, so it's now allowed to call GetCaps */
        if !(*device).driver.is_null() {
            (*device).drvcaps.dwSize = ::core::mem::size_of::<DSCDRIVERCAPS>() as DWORD;
            err = (*(*(*device).driver).lpVtbl)
                .GetCaps
                .unwrap_unchecked()(
                (*device).driver as *mut IDsCaptureDriver,
                core::ptr::addr_of_mut!((*device).drvcaps),
            );
            if err != (DS_OK as HRESULT) {
                WARN!("IDsCaptureDriver_GetCaps failed\n");
                return err;
            }
        } else {
            /*if (device->hwi)*/
            let mut wic: WAVEINCAPSA = core::mem::zeroed();
            err = mmErr(waveInGetDevCapsA(
                (*device).drvdesc.dnDevNode as UINT as UINT_PTR,
                core::ptr::addr_of_mut!(wic),
                ::core::mem::size_of::<WAVEINCAPSA>() as UINT,
            ) as UINT);

            if err == (DS_OK as HRESULT) {
                (*device).drvcaps.dwFlags = 0;
                lstrcpynA(
                    core::ptr::addr_of_mut!((*device).drvdesc.szDrvname) as LPSTR,
                    core::ptr::addr_of_mut!(wic.szPname) as *mut CHAR as LPCSTR,
                    ::core::mem::size_of::<[CHAR; 256]>() as ::core::ffi::c_int,
                );

                (*device).drvcaps.dwFlags |= DSCCAPS_EMULDRIVER;
                (*device).drvcaps.dwFormats = wic.dwFormats;
                (*device).drvcaps.dwChannels = wic.wChannels as DWORD;
            }
        }
    }

    return err;
}

unsafe extern "system" fn IDirectSoundCaptureImpl_QueryInterface(iface: LPDIRECTSOUNDCAPTURE, riid: *const IID, ppobj: *mut LPVOID) -> HRESULT {
    let This: *mut IDirectSoundCaptureImpl = iface as *mut IDirectSoundCaptureImpl;

    if ppobj.is_null() {
        WARN!("invalid parameter\n");
        return E_INVALIDARG;
    }

    *ppobj = core::ptr::null_mut();

    if IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown)) != 0 {
        IDirectSoundCapture_AddRef(This as LPDIRECTSOUNDCAPTURE);
        *ppobj = This as LPVOID;
        return (DS_OK) as HRESULT;
    } else if IsEqualIID!(riid, core::ptr::addr_of!(IID_IDirectSoundCapture)) != 0 {
        IDirectSoundCapture_AddRef(This as LPDIRECTSOUNDCAPTURE);
        *ppobj = This as LPVOID;
        return (DS_OK) as HRESULT;
    }

    WARN!("unsupported riid: %s\n", debugstr_guid(riid as *const _GUID));
    return E_NOINTERFACE;
}

unsafe extern "system" fn IDirectSoundCaptureImpl_AddRef(iface: LPDIRECTSOUNDCAPTURE) -> ULONG {
    let This: *mut IDirectSoundCaptureImpl = iface as *mut IDirectSoundCaptureImpl;
    let ref_: ULONG = InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_) as *mut LONG) as ULONG;
    return ref_;
}

unsafe extern "system" fn IDirectSoundCaptureImpl_Release(iface: LPDIRECTSOUNDCAPTURE) -> ULONG {
    let mut This: *mut IDirectSoundCaptureImpl = iface as *mut IDirectSoundCaptureImpl;
    let mut ref_0: ULONG =
        InterlockedDecrement(core::ptr::addr_of_mut!((*This).ref_) as *mut LONG) as ULONG;
    WINE_TRACE!("(%p) ref was %d\n", This, ref_0.wrapping_add(1 as ULONG));
    if ref_0 == 0 {
        if !(*This).device.is_null() {
            DirectSoundCaptureDevice_Release((*This).device);
        }
        HeapFree(
            GetProcessHeap(),
            0,
            This as PVOID,
        );
        WINE_TRACE!("(%p) released\n", This);
    }
    return ref_0;
}

unsafe extern "system" fn IDirectSoundCaptureImpl_CreateCaptureBuffer(
    iface: LPDIRECTSOUNDCAPTURE,
    lpcDSCBufferDesc: LPCDSCBUFFERDESC,
    lplpDSCaptureBuffer: *mut LPDIRECTSOUNDCAPTUREBUFFER,
    pUnk: LPUNKNOWN,
) -> HRESULT {
    let mut hr: HRESULT;
    let This: *mut IDirectSoundCaptureImpl = iface as *mut IDirectSoundCaptureImpl;

    if lpcDSCBufferDesc.is_null() {
        WARN!("invalid parameter: lpcDSCBufferDesc == NULL)\n");
        return DSERR_INVALIDPARAM;
    }

    if lplpDSCaptureBuffer.is_null() {
        WARN!("invalid parameter: lplpDSCaptureBuffer == NULL\n");
        return DSERR_INVALIDPARAM;
    }

    if !pUnk.is_null() {
        WARN!("invalid parameter: pUnk != NULL\n");
        return DSERR_INVALIDPARAM;
    }

    /* FIXME: We can only have one buffer so what do we do here? */
    if !(*(*This).device).capture_buffer.is_null() {
        WARN!("invalid parameter: already has buffer\n");
        return DSERR_INVALIDPARAM; /* DSERR_GENERIC ? */
    }

    hr = IDirectSoundCaptureBufferImpl_Create(
        (*This).device,
        lplpDSCaptureBuffer as *mut *mut IDirectSoundCaptureBufferImpl,
        lpcDSCBufferDesc,
    );

    if hr != (DS_OK as HRESULT) {
        WARN!("IDirectSoundCaptureBufferImpl_Create failed\n");
    }

    return hr;
}
unsafe extern "system" fn IDirectSoundCaptureImpl_GetCaps(
    iface: LPDIRECTSOUNDCAPTURE,
    lpDSCCaps: LPDSCCAPS,
) -> HRESULT {
    let This: *mut IDirectSoundCaptureImpl = iface as *mut IDirectSoundCaptureImpl;

    if (*This).device.is_null() {
        WARN!("not initialized\n");
        return DSERR_UNINITIALIZED;
    }

    if lpDSCCaps.is_null() {
        WARN!("invalid parameter: lpDSCCaps== NULL\n");
        return DSERR_INVALIDPARAM;
    }

    if ((*lpDSCCaps).dwSize as usize) < core::mem::size_of::<_DSCCAPS>() {
        WARN!("invalid parameter: lpDSCCaps->dwSize = %d\n", (*lpDSCCaps).dwSize);
        return DSERR_INVALIDPARAM;
    }

    (*lpDSCCaps).dwFlags = (*(*This).device).drvcaps.dwFlags;
    (*lpDSCCaps).dwFormats = (*(*This).device).drvcaps.dwFormats;
    (*lpDSCCaps).dwChannels = (*(*This).device).drvcaps.dwChannels;

    return (DS_OK) as HRESULT;
}

unsafe extern "system" fn IDirectSoundCaptureImpl_Initialize(
    iface: LPDIRECTSOUNDCAPTURE,
    lpcGUID: LPCGUID,
) -> HRESULT {
    let This: *mut IDirectSoundCaptureImpl = iface as *mut IDirectSoundCaptureImpl;

    if !(*This).device.is_null() {
        WARN!("already initialized\n");
        return DSERR_ALREADYINITIALIZED;
    }
    return DirectSoundCaptureDevice_Initialize(core::ptr::addr_of_mut!((*This).device), lpcGUID);
}

unsafe extern "C" fn IDirectSoundCaptureImpl_Create(
    ppDSC: *mut LPDIRECTSOUNDCAPTURE8,
) -> HRESULT {
    let mut pDSC: *mut IDirectSoundCaptureImpl = core::ptr::null_mut();

    /* Allocate memory */
    pDSC = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY as DWORD,
        (core::mem::size_of::<IDirectSoundCaptureImpl>()) as SIZE_T,
    ) as *mut IDirectSoundCaptureImpl;
    if pDSC.is_null() {
        WARN!("out of memory\n");
        *ppDSC = core::ptr::null_mut();
        return DSERR_OUTOFMEMORY;
    }

    (*pDSC).lpVtbl = core::ptr::addr_of!(dscvt);
    (*pDSC).ref_ = 0;
    (*pDSC).device = core::ptr::null_mut();

    *ppDSC = pDSC as LPDIRECTSOUNDCAPTURE8;

    return (DS_OK) as HRESULT;
}
#[no_mangle]
pub unsafe extern "C" fn DSOUND_CaptureCreate(riid: *const IID, ppDSC: *mut LPDIRECTSOUNDCAPTURE) -> HRESULT {
    let mut pDSC: LPDIRECTSOUNDCAPTURE = core::ptr::null_mut();
    let mut hr: HRESULT;

    if ((IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown))) == 0)
        && ((IsEqualIID!(riid, core::ptr::addr_of!(IID_IDirectSoundCapture))) == 0)
    {
        *ppDSC = core::ptr::null_mut();
        return E_NOINTERFACE;
    }

    /* Get dsound configuration */
    setup_dsound_options();

    hr = IDirectSoundCaptureImpl_Create(core::ptr::addr_of_mut!(pDSC));
    if hr == (DS_OK as HRESULT) {
        IDirectSoundCapture_AddRef(pDSC);
        *ppDSC = pDSC;
    } else {
        WARN!("IDirectSoundCaptureImpl_Create failed\n");
        *ppDSC = core::ptr::null_mut();
    }

    return hr;
}
#[no_mangle]
pub unsafe extern "C" fn DSOUND_CaptureCreate8(riid: *const IID, ppDSC8: *mut LPDIRECTSOUNDCAPTURE8) -> HRESULT {
    let mut pDSC8: LPDIRECTSOUNDCAPTURE8 = core::ptr::null_mut();
    let mut hr: HRESULT;

    if ((IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown))) == 0)
        && ((IsEqualIID!(riid, core::ptr::addr_of!(IID_IDirectSoundCapture8))) == 0)
    {
        *ppDSC8 = core::ptr::null_mut();
        return E_NOINTERFACE;
    }

    /* Get dsound configuration */
    setup_dsound_options();

    hr = IDirectSoundCaptureImpl_Create(core::ptr::addr_of_mut!(pDSC8));
    if hr == (DS_OK as HRESULT) {
        IDirectSoundCapture_AddRef(pDSC8);
        *ppDSC8 = pDSC8;
    } else {
        WARN!("IDirectSoundCaptureImpl_Create failed\n");
        *ppDSC8 = core::ptr::null_mut();
    }

    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn DirectSoundCaptureCreate(
    lpcGUID: LPCGUID,
    ppDSC: *mut LPDIRECTSOUNDCAPTURE,
    pUnkOuter: LPUNKNOWN,
) -> HRESULT {
    let mut hr: HRESULT = 0;
    let mut pDSC: LPDIRECTSOUNDCAPTURE = core::ptr::null_mut();

    if ppDSC.is_null() {
        WARN!("invalid parameter: ppDSC == NULL\n");
        return DSERR_INVALIDPARAM;
    }

    if !pUnkOuter.is_null() {
        WARN!("invalid parameter: pUnkOuter != NULL\n");
        *ppDSC = core::ptr::null_mut();
        return DSERR_NOAGGREGATION;
    }

    hr = DSOUND_CaptureCreate(
        core::ptr::addr_of!(IID_IDirectSoundCapture),
        core::ptr::addr_of_mut!(pDSC),
    );
    if hr == (DS_OK as HRESULT) {
        hr = IDirectSoundCapture_Initialize(pDSC, lpcGUID);
        if hr != (DS_OK as HRESULT) {
            IDirectSoundCapture_Release(pDSC);
            pDSC = core::ptr::null_mut();
        }
    }

    *ppDSC = pDSC;

    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn DirectSoundCaptureCreate8(
    lpcGUID: LPCGUID,
    ppDSC8: *mut LPDIRECTSOUNDCAPTURE8,
    pUnkOuter: LPUNKNOWN,
) -> HRESULT {
    let mut hr: HRESULT;
    let mut pDSC8: LPDIRECTSOUNDCAPTURE8 = core::ptr::null_mut();
    TRACE!("(%s,%p,%p)\n", debugstr_guid(lpcGUID), ppDSC8, pUnkOuter);

    if ppDSC8.is_null() {
        WARN!("invalid parameter: ppDSC8 == NULL\n");
        return DSERR_INVALIDPARAM;
    }

    if !pUnkOuter.is_null() {
        WARN!("invalid parameter: pUnkOuter != NULL\n");
        *ppDSC8 = core::ptr::null_mut();
        return DSERR_NOAGGREGATION;
    }

    hr = DSOUND_CaptureCreate8(
        core::ptr::addr_of!(IID_IDirectSoundCapture8),
        core::ptr::addr_of_mut!(pDSC8),
    );
    if hr == (DS_OK as HRESULT) {
        hr = IDirectSoundCapture_Initialize(pDSC8, lpcGUID);
        if hr != (DS_OK as HRESULT) {
            IDirectSoundCapture_Release(pDSC8);
            pDSC8 = core::ptr::null_mut();
        }
    }

    *ppDSC8 = pDSC8;

    return hr;
}

// COM vtable tables — generated from capture.c by vtbl_gen.py
/// capture.c:136 `static const IDirectSoundNotifyVtbl dscnvt`  (slot order: rust:types_amd64.rs)
pub static dscnvt: IDirectSoundNotifyVtbl = IDirectSoundNotifyVtbl {
    QueryInterface: Some(IDirectSoundCaptureNotifyImpl_QueryInterface),
    AddRef: Some(IDirectSoundCaptureNotifyImpl_AddRef),
    Release: Some(IDirectSoundCaptureNotifyImpl_Release),
    SetNotificationPositions: Some(IDirectSoundCaptureNotifyImpl_SetNotificationPositions),
};

/// capture.c:716 `static const IDirectSoundCaptureBuffer8Vtbl dscbvt`  (slot order: rust:types_amd64.rs)
pub static dscbvt: IDirectSoundCaptureBuffer8Vtbl = IDirectSoundCaptureBuffer8Vtbl {
    QueryInterface: Some(IDirectSoundCaptureBufferImpl_QueryInterface),
    AddRef: Some(IDirectSoundCaptureBufferImpl_AddRef),
    Release: Some(IDirectSoundCaptureBufferImpl_Release),
    GetCaps: Some(IDirectSoundCaptureBufferImpl_GetCaps),
    GetCurrentPosition: Some(IDirectSoundCaptureBufferImpl_GetCurrentPosition),
    GetFormat: Some(IDirectSoundCaptureBufferImpl_GetFormat),
    GetStatus: Some(IDirectSoundCaptureBufferImpl_GetStatus),
    Initialize: Some(IDirectSoundCaptureBufferImpl_Initialize),
    Lock: Some(IDirectSoundCaptureBufferImpl_Lock),
    Start: Some(IDirectSoundCaptureBufferImpl_Start),
    Stop: Some(IDirectSoundCaptureBufferImpl_Stop),
    Unlock: Some(IDirectSoundCaptureBufferImpl_Unlock),
    GetObjectInPath: Some(IDirectSoundCaptureBufferImpl_GetObjectInPath),
    GetFXStatus: Some(IDirectSoundCaptureBufferImpl_GetFXStatus),
};

/// capture.c:1300 `static const IDirectSoundCaptureVtbl dscvt`  (slot order: rust:types_amd64.rs)
pub static dscvt: IDirectSoundCaptureVtbl = IDirectSoundCaptureVtbl {
    QueryInterface: Some(IDirectSoundCaptureImpl_QueryInterface),
    AddRef: Some(IDirectSoundCaptureImpl_AddRef),
    Release: Some(IDirectSoundCaptureImpl_Release),
    CreateCaptureBuffer: Some(IDirectSoundCaptureImpl_CreateCaptureBuffer),
    GetCaps: Some(IDirectSoundCaptureImpl_GetCaps),
    Initialize: Some(IDirectSoundCaptureImpl_Initialize),
};
