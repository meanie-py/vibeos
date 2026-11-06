use crate::dsound::mixer::DSOUND_AmpFactorToVolPan;
use crate::dsound::mixer::DSOUND_bufpos_to_mixpos;
use crate::dsound::mixer::DSOUND_callback;
use crate::dsound::dsound_main::ds_hel_buflen;
use crate::dsound::dsound_main::ds_hw_accel;
use crate::dsound::dsound_main::ds_snd_queue_max;
use crate::dsound::dsound_main::ds_snd_queue_min;
use crate::dsound::dsound_convert::mixfunctions;
use crate::dsound::dsound_convert::normfunctions;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// No declarations emitted: the only file-scope item is the `dspbvt` COM vtable (`IDirectSoundBufferVtbl`), which is generated from the bindgen struct by vtbl_gen.py.

#[no_mangle]
pub unsafe extern "C" fn DSOUND_fraglen(
    mut nSamplesPerSec: DWORD,
    mut nBlockAlign: DWORD,
) -> DWORD {
    if nSamplesPerSec <= 12800 as DWORD {
        return (128 as DWORD).wrapping_mul(nBlockAlign);
    }
    if nSamplesPerSec <= 25600 as DWORD {
        return (256 as DWORD).wrapping_mul(nBlockAlign);
    }
    if nSamplesPerSec <= 51200 as DWORD {
        return (512 as DWORD).wrapping_mul(nBlockAlign);
    }
    return (1024 as DWORD).wrapping_mul(nBlockAlign);
}

unsafe extern "C" fn DSOUND_RecalcPrimary(device: *mut DirectSoundDevice) {
    (*device).fraglen = DSOUND_fraglen(
        (*(*device).pwfx).nSamplesPerSec,
        (*(*device).pwfx).nBlockAlign as DWORD,
    );
    (*device).helfrags = (*device).buflen.wrapping_div((*device).fraglen) as UINT;

    if !(*device).hwbuf.is_null() && ((*device).drvdesc.dwFlags & DSDDESC_DONTNEEDWRITELEAD as DWORD) != 0 {
        (*device).writelead = 0 as DWORD;
    } else {
        /* calculate the 10ms write lead */
        (*device).writelead = (*(*device).pwfx)
            .nSamplesPerSec
            .wrapping_div(100 as DWORD)
            .wrapping_mul((*(*device).pwfx).nBlockAlign as DWORD);
    }
}
#[no_mangle]
pub unsafe extern "C" fn DSOUND_ReopenDevice(device: *mut DirectSoundDevice, forcewave: BOOL) -> HRESULT {
    let mut hres: HRESULT = (DS_OK as i32);

    if !(*device).driver.is_null() {
        IDsDriver_Close((*device).driver);
        if (*device).drvdesc.dwFlags & DSDDESC_DOMMSYSTEMOPEN != 0 {
            waveOutClose((*device).hwo);
        }
        IDsDriver_Release((*device).driver);
        (*device).driver = core::ptr::null_mut();
        (*device).buffer = core::ptr::null_mut();
        (*device).hwo = core::ptr::null_mut();
    } else if (*device).drvdesc.dwFlags & DSDDESC_DOMMSYSTEMOPEN != 0 {
        waveOutClose((*device).hwo);
    }

    /* DRV_QUERYDSOUNDIFACE is a "Wine extension" to get the DSound interface */
    if ds_hw_accel != (DS_HW_ACCEL_EMULATION as i32) && forcewave == 0 {
        waveOutMessage(
            ((*device).drvdesc.dnDevNode as DWORD_PTR) as HWAVEOUT,
            DRV_QUERYDSOUNDIFACE,
            core::ptr::addr_of_mut!((*device).driver) as DWORD_PTR,
            0,
        );
    }

    /* Get driver description */
    if !(*device).driver.is_null() {
        let wod: DWORD = (*device).drvdesc.dnDevNode;
        hres = IDsDriver_GetDriverDesc((*device).driver, core::ptr::addr_of_mut!((*device).drvdesc));
        (*device).drvdesc.dnDevNode = wod;
        if hres < 0 {
            WARN!("IDsDriver_GetDriverDesc failed: %08x\n", hres);
            IDsDriver_Release((*device).driver);
            (*device).driver = core::ptr::null_mut();
        }
    }

    /* if no DirectSound interface available, use WINMM API instead */
    if (*device).driver.is_null() {
        (*device).drvdesc.dwFlags = DSDDESC_DOMMSYSTEMOPEN | DSDDESC_DOMMSYSTEMSETFORMAT;
    }

    if (*device).drvdesc.dwFlags & DSDDESC_DOMMSYSTEMOPEN != 0 {
        let mut flags: DWORD = CALLBACK_FUNCTION | WAVE_MAPPED;

        if !(*device).driver.is_null() {
            flags |= WAVE_DIRECTSOUND;
        }

        hres = mmErr(waveOutOpen(
            core::ptr::addr_of_mut!((*device).hwo),
            (*device).drvdesc.dnDevNode,
            (*device).pwfx,
            DSOUND_callback as DWORD_PTR,
            device as DWORD_PTR,
            flags,
        ));
        if hres < 0 {
            WARN!("waveOutOpen failed\n");
            if !(*device).driver.is_null() {
                IDsDriver_Release((*device).driver);
                (*device).driver = core::ptr::null_mut();
            }
            return hres;
        }
    }

    if !(*device).driver.is_null() {
        hres = IDsDriver_Open((*device).driver);
    }

    return hres;
}

unsafe extern "C" fn DSOUND_PrimaryOpen(device: *mut DirectSoundDevice) -> HRESULT {
    let mut buflen: DWORD = 0;
    let mut err: HRESULT = DS_OK as HRESULT;

    /* on original windows, the buffer it set to a fixed size, no matter what the settings are.
       on windows this size is always fixed (tested on win-xp) */
    if (*device).buflen == 0 {
        (*device).buflen = ds_hel_buflen as DWORD;
    }
    buflen = (*device).buflen;
    buflen = buflen.wrapping_sub(buflen.wrapping_rem((*(*device).pwfx).nBlockAlign as DWORD));
    (*device).buflen = buflen;

    if !(*device).driver.is_null() {
        err = IDsDriver_CreateSoundBuffer(
            (*device).driver,
            (*device).pwfx,
            DSBCAPS_PRIMARYBUFFER,
            0,
            core::ptr::addr_of_mut!((*device).buflen),
            core::ptr::addr_of_mut!((*device).buffer),
            core::ptr::addr_of_mut!((*device).hwbuf) as *mut LPVOID,
        );

        if err != DS_OK as HRESULT {
            WARN!("IDsDriver_CreateSoundBuffer failed (%08x), falling back to waveout\n", err);
            err = DSOUND_ReopenDevice(device, (TRUE) as BOOL);
            if err < 0 as HRESULT {
                WARN!("Falling back to waveout failed too! Giving up\n");
                return err;
            }
        }
        if !(*device).hwbuf.is_null() {
            IDsDriverBuffer_SetVolumePan((*device).hwbuf, core::ptr::addr_of_mut!((*device).volpan));
        }

        DSOUND_RecalcPrimary(device);
        (*device).prebuf = ds_snd_queue_max as UINT;
        if (*device).helfrags < ds_snd_queue_min as UINT {
            WARN!(
                "Too little sound buffer to be effective (%d/%d) falling back to waveout\n",
                (*device).buflen,
                (ds_snd_queue_min as DWORD).wrapping_mul((*device).fraglen)
            );
            (*device).buflen = buflen;
            IDsDriverBuffer_Release((*device).hwbuf);
            (*device).hwbuf = core::ptr::null_mut::<IDsDriverBuffer>();
            err = DSOUND_ReopenDevice(device, (TRUE) as BOOL);
            if err < 0 as HRESULT {
                WARN!("Falling back to waveout failed too! Giving up\n");
                return err;
            }
        } else if (*device).helfrags < ds_snd_queue_max as UINT {
            (*device).prebuf = (*device).helfrags;
        }
    }

    (*device).mix_buffer_len = DSOUND_bufpos_to_mixpos(device, (*device).buflen);
    (*device).mix_buffer = HeapAlloc(
        GetProcessHeap(),
        0,
        (*device).mix_buffer_len as SIZE_T,
    ) as LPBYTE;
    if (*device).mix_buffer.is_null() {
        if !(*device).hwbuf.is_null() {
            IDsDriverBuffer_Release((*device).hwbuf);
        }
        (*device).hwbuf = core::ptr::null_mut::<IDsDriverBuffer>();
        return DSERR_OUTOFMEMORY as HRESULT;
    }

    if (*device).state == STATE_PLAYING as DWORD {
        (*device).state = STATE_STARTING as DWORD;
    } else if (*device).state == STATE_STOPPING as DWORD {
        (*device).state = STATE_STOPPED as DWORD;
    }

    /* are we using waveOut stuff? */
    if (*device).driver.is_null() {
        let mut newbuf: LPBYTE = core::ptr::null_mut::<BYTE>();
        let mut headers: LPWAVEHDR = core::ptr::null_mut::<wavehdr_tag>();
        let mut overshot: DWORD = 0;
        let mut c: u32 = 0;

        /* Start in pause mode, to allow buffers to get filled */
        waveOutPause((*device).hwo);

        /* reallocate emulated primary buffer */
        if !(*device).buffer.is_null() {
            newbuf = HeapReAlloc(
                GetProcessHeap(),
                0,
                (*device).buffer as PVOID,
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
            ERR!("failed to allocate primary buffer\n");
            return DSERR_OUTOFMEMORY as HRESULT;
            /* but the old buffer might still exist and must be re-prepared */
        }

        DSOUND_RecalcPrimary(device);
        if !(*device).pwave.is_null() {
            headers = HeapReAlloc(
                GetProcessHeap(),
                0,
                (*device).pwave as PVOID,
                ((*device).helfrags as SIZE_T)
                    .wrapping_mul(core::mem::size_of::<WAVEHDR>() as SIZE_T),
            ) as LPWAVEHDR;
        } else {
            headers = HeapAlloc(
                GetProcessHeap(),
                0,
                ((*device).helfrags as SIZE_T)
                    .wrapping_mul(core::mem::size_of::<WAVEHDR>() as SIZE_T),
            ) as LPWAVEHDR;
        }

        if headers.is_null() {
            ERR!("failed to allocate wave headers\n");
            HeapFree(GetProcessHeap(), 0, newbuf as PVOID);
            DSOUND_RecalcPrimary(device);
            return DSERR_OUTOFMEMORY as HRESULT;
        }

        (*device).buffer = newbuf;
        (*device).pwave = headers;

        /* prepare fragment headers */
        c = 0;
        while c < (*device).helfrags {
            (*(*device).pwave.offset(c as isize)).lpData =
                ((*device).buffer as *mut ::core::ffi::c_char)
                    .offset((c * (*device).fraglen) as isize) as LPSTR;
            (*(*device).pwave.offset(c as isize)).dwBufferLength = (*device).fraglen;
            (*(*device).pwave.offset(c as isize)).dwUser = device as DWORD_PTR;
            (*(*device).pwave.offset(c as isize)).dwFlags = 0 as DWORD;
            (*(*device).pwave.offset(c as isize)).dwLoops = 0 as DWORD;
            err = mmErr(waveOutPrepareHeader(
                (*device).hwo,
                (*device).pwave.offset(c as isize) as LPWAVEHDR,
                core::mem::size_of::<WAVEHDR>() as UINT,
            ) as UINT) as HRESULT;
            if err != DS_OK as HRESULT {
                loop {
                    let fresh = c;
                    c = c.wrapping_sub(1);
                    if fresh == 0 {
                        break;
                    }
                    waveOutUnprepareHeader(
                        (*device).hwo,
                        (*device).pwave.offset(c as isize) as LPWAVEHDR,
                        core::mem::size_of::<WAVEHDR>() as UINT,
                    );
                }
                break;
            } else {
                c = c.wrapping_add(1);
            }
        }

        overshot = (*device).buflen.wrapping_rem((*device).fraglen);
        /* sanity */
        if overshot != 0 {
            overshot = overshot
                .wrapping_sub(overshot.wrapping_rem((*(*device).pwfx).nBlockAlign as DWORD));
            (*(*device)
                .pwave
                .offset(((*device).helfrags - 1) as isize))
            .dwBufferLength += overshot;
        }
    }

    (*device).mixfunction = mixfunctions[((*(*device).pwfx).wBitsPerSample as i32
        / 8
        - 1) as usize];
    (*device).normfunction = normfunctions[((*(*device).pwfx).wBitsPerSample as i32
        / 8
        - 1) as usize];
    FillMemory!(
        (*device).buffer,
        (*device).buflen,
        if (*(*device).pwfx).wBitsPerSample == 8 {
            128
        } else {
            0
        },
    );
    FillMemory!((*device).mix_buffer, (*device).mix_buffer_len, 0);
    (*device).mixpos = 0 as DWORD;
    (*device).playpos = (*device).mixpos;
    (*device).pwqueue = (*device).playpos as UINT;
    (*device).pwplay = (*device).pwqueue;
    return err;
}

unsafe extern "C" fn DSOUND_PrimaryClose(device: *mut DirectSoundDevice) {
    /* are we using waveOut stuff? */
    if (*device).hwbuf.is_null() {
        let mut c: UINT;

        /* get out of CS when calling the wave system */
        LeaveCriticalSection(core::ptr::addr_of_mut!((*device).mixlock));
        /* **** */
        (*device).pwqueue = (-(1) as i32) as DWORD; /* resetting queues */
        waveOutReset((*device).hwo);
        c = 0 as UINT;
        while c < (*device).helfrags {
            waveOutUnprepareHeader(
                (*device).hwo,
                (*device).pwave.add(c as usize) as LPWAVEHDR,
                core::mem::size_of::<WAVEHDR>() as UINT,
            );
            c = c.wrapping_add(1);
        }
        /* **** */
        EnterCriticalSection(core::ptr::addr_of_mut!((*device).mixlock));

        /* clear the queue */
        (*device).pwqueue = 0 as DWORD;
    } else {
        let ref_0: ULONG = IDsDriverBuffer_Release((*device).hwbuf);
        if ref_0 == 0 {
            (*device).hwbuf = core::ptr::null_mut();
        } else {
            ERR!("Still %d references on primary buffer, refcount leak?\n", ref_0);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn DSOUND_PrimaryCreate(device: *mut DirectSoundDevice) -> HRESULT {
    let mut err: HRESULT = (DS_OK as i32);

    (*device).buflen = ds_hel_buflen as DWORD;
    err = DSOUND_PrimaryOpen(device);

    if err != (DS_OK as HRESULT) {
        WARN!("DSOUND_PrimaryOpen failed\n");
        return err;
    }

    (*device).state = STATE_STOPPED as DWORD;
    return (DS_OK) as HRESULT;
}
#[no_mangle]
pub unsafe extern "C" fn DSOUND_PrimaryDestroy(device: *mut DirectSoundDevice) -> HRESULT {
    EnterCriticalSection(&raw mut (*device).mixlock);

    DSOUND_PrimaryClose(device);
    if !(*device).driver.is_null() {
        if !(*device).hwbuf.is_null() {
            if IDsDriverBuffer_Release((*device).hwbuf as *mut IDsDriverBuffer) == 0 {
                (*device).hwbuf = core::ptr::null_mut();
            }
        }
    } else {
        HeapFree(GetProcessHeap(), 0, (*device).pwave as PVOID);
    }
    HeapFree(GetProcessHeap(), 0, (*device).pwfx as PVOID);
    (*device).pwfx = ::core::ptr::null_mut::<_WAVEFORMATEX>();

    LeaveCriticalSection(&raw mut (*device).mixlock);

    return (DS_OK) as HRESULT;
}
#[no_mangle]
pub unsafe extern "C" fn DSOUND_PrimaryPlay(device: *mut DirectSoundDevice) -> HRESULT {
    let mut err: HRESULT = (DS_OK as i32);
    if !(*device).hwbuf.is_null() {
        err = IDsDriverBuffer_Play((*device).hwbuf as *mut IDsDriverBuffer, 0, 0, DSBPLAY_LOOPING);
        if err != (DS_OK as HRESULT) {
            WARN!("IDsDriverBuffer_Play failed\n");
        }
    } else {
        err = mmErr(waveOutRestart((*device).hwo) as UINT);
        if err != (DS_OK as HRESULT) {
            WARN!("waveOutRestart failed\n");
        }
    }
    return err;
}
#[no_mangle]
pub unsafe extern "C" fn DSOUND_PrimaryStop(device: *mut DirectSoundDevice) -> HRESULT {
    let mut err: HRESULT = (DS_OK as i32);

    if !(*device).hwbuf.is_null() {
        err = IDsDriverBuffer_Stop((*device).hwbuf);
        if err == DSERR_BUFFERLOST {
            DSOUND_PrimaryClose(device);
            err = DSOUND_ReopenDevice(device, (FALSE) as BOOL);
            if err < 0 {
                ERR!("DSOUND_ReopenDevice failed\n");
            } else {
                err = DSOUND_PrimaryOpen(device);
                if err < 0 {
                    WARN!("DSOUND_PrimaryOpen failed\n");
                }
            }
        } else if err != (DS_OK as HRESULT) {
            WARN!("IDsDriverBuffer_Stop failed\n");
        }
    } else {
        /* don't call the wave system with the lock set */
        LeaveCriticalSection(core::ptr::addr_of_mut!((*device).mixlock));
        /* **** */

        err = mmErr(waveOutPause((*device).hwo) as UINT);

        /* **** */
        EnterCriticalSection(core::ptr::addr_of_mut!((*device).mixlock));

        if err != (DS_OK as HRESULT) {
            WARN!("waveOutPause failed\n");
        }
    }

    return err;
}
#[no_mangle]
pub unsafe extern "C" fn DSOUND_PrimaryGetPosition(device: *mut DirectSoundDevice, playpos: LPDWORD, writepos: LPDWORD) -> HRESULT {
    if !(*device).hwbuf.is_null() {
        let err: HRESULT = IDsDriverBuffer_GetPosition((*device).hwbuf as *mut IDsDriverBuffer, playpos, writepos);
        if err != S_OK {
            WARN!("IDsDriverBuffer_GetPosition failed\n");
            return err;
        }
    } else {
        /* check if playpos was requested */
        if !playpos.is_null() {
            /* use the cached play position */
            *playpos = ((*device).pwplay as DWORD).wrapping_mul((*device).fraglen);
        }

        /* check if writepos was requested */
        if !writepos.is_null() {
            /* the writepos is the first non-queued position */
            *writepos = ((*device)
                .pwplay
                .wrapping_add((*device).pwqueue)
                .wrapping_rem((*device).helfrags) as DWORD)
                .wrapping_mul((*device).fraglen);
        }
    }
    return (DS_OK) as HRESULT;
}

unsafe extern "C" fn DSOUND_GetFormatSize(wfex: LPCWAVEFORMATEX) -> DWORD {
    if (*wfex).wFormatTag == (WAVE_FORMAT_PCM as u16) {
        return core::mem::size_of::<WAVEFORMATEX>() as DWORD;
    } else {
        return (core::mem::size_of::<WAVEFORMATEX>() as DWORD).wrapping_add((*wfex).cbSize as DWORD);
    }
}
#[no_mangle]
pub unsafe extern "C" fn DSOUND_CopyFormat(wfex: LPCWAVEFORMATEX) -> LPWAVEFORMATEX {
    let size: DWORD = DSOUND_GetFormatSize(wfex);
    let pwfx: LPWAVEFORMATEX = HeapAlloc(GetProcessHeap(), 0, (size) as SIZE_T) as LPWAVEFORMATEX;
    if pwfx.is_null() {
        WARN!("out of memory\n");
    } else if (*wfex).wFormatTag != (WAVE_FORMAT_PCM as u16) {
        CopyMemory!(pwfx as *mut core::ffi::c_void, wfex as *const core::ffi::c_void, size as usize);
    } else {
        CopyMemory!(pwfx as *mut core::ffi::c_void, wfex as *const core::ffi::c_void, core::mem::size_of::<PCMWAVEFORMAT>());
        (*pwfx).cbSize = 0;
        if (*pwfx).nBlockAlign as i32 != (*pwfx).nChannels as i32 * (*pwfx).wBitsPerSample as i32 / 8 {
            WARN!("Fixing bad nBlockAlign (%u)\n", (*pwfx).nBlockAlign as core::ffi::c_uint);
            (*pwfx).nBlockAlign = ((*pwfx).nChannels as i32 * (*pwfx).wBitsPerSample as i32 / 8) as WORD;
        }
        if (*pwfx).nAvgBytesPerSec != (*pwfx).nSamplesPerSec.wrapping_mul((*pwfx).nBlockAlign as DWORD) {
            WARN!("Fixing bad nAvgBytesPerSec (%u)\n", (*pwfx).nAvgBytesPerSec);
            (*pwfx).nAvgBytesPerSec = (*pwfx).nSamplesPerSec.wrapping_mul((*pwfx).nBlockAlign as DWORD);
        }
    }
    return pwfx;
}
#[no_mangle]
pub unsafe extern "C" fn primarybuffer_SetFormat(device: *mut DirectSoundDevice, wfex: LPCWAVEFORMATEX) -> HRESULT {
    let mut err: HRESULT = DSERR_BUFFERLOST;
    let mut i: i32 = 0;
    let mut nSamplesPerSec: DWORD = 0;
    let mut bpp: DWORD = 0;
    let mut chans: DWORD = 0;
    let mut oldpwfx: LPWAVEFORMATEX = core::ptr::null_mut();
    let mut forced: BOOL = (((*device).priolevel == DSSCL_WRITEPRIMARY) as BOOL);

    if (*device).priolevel == DSSCL_NORMAL {
        WARN!("failed priority check!\n");
        return DSERR_PRIOLEVELNEEDED;
    }

    /* Let's be pedantic! */
    if wfex.is_null() {
        WARN!("invalid parameter: wfex==NULL!\n");
        return DSERR_INVALIDPARAM;
    }

    /* **** */
    RtlAcquireResourceExclusive(core::ptr::addr_of_mut!((*device).buffer_list_lock), TRUE as BYTE);
    EnterCriticalSection(core::ptr::addr_of_mut!((*device).mixlock));

    nSamplesPerSec = (*(*device).pwfx).nSamplesPerSec;
    bpp = (*(*device).pwfx).wBitsPerSample as DWORD;
    chans = (*(*device).pwfx).nChannels as DWORD;

    oldpwfx = (*device).pwfx as LPWAVEFORMATEX;
    (*device).pwfx = DSOUND_CopyFormat(wfex) as PWAVEFORMATEX;
    if (*device).pwfx.is_null() {
        (*device).pwfx = oldpwfx as PWAVEFORMATEX;
        oldpwfx = core::ptr::null_mut();
        err = DSERR_OUTOFMEMORY;
        'done: {
            LeaveCriticalSection(core::ptr::addr_of_mut!((*device).mixlock));
            RtlReleaseResource(core::ptr::addr_of_mut!((*device).buffer_list_lock));
            HeapFree(GetProcessHeap(), 0 as DWORD, oldpwfx as PVOID);
            return err;
        }
    }

    'done: {
        if (((*device).drvdesc.dwFlags & DSDDESC_DOMMSYSTEMSETFORMAT) == 0) && !(*device).hwbuf.is_null() {
            err = IDsDriverBuffer_SetFormat((*device).hwbuf, (*device).pwfx);

            /* On bad format, try to re-create, big chance it will work then, only do this if we <HAVE> to */
            if forced != 0
                && ((*(*device).pwfx).nSamplesPerSec / 100 != (*wfex).nSamplesPerSec / 100
                    || err == DSERR_BADFORMAT)
            {
                let cp_size: DWORD = if (*wfex).wFormatTag == (WAVE_FORMAT_PCM as u16) {
                    core::mem::size_of::<PCMWAVEFORMAT>() as DWORD
                } else {
                    (core::mem::size_of::<WAVEFORMATEX>() + (*wfex).cbSize as usize) as DWORD
                };
                err = DSERR_BUFFERLOST;
                CopyMemory!((*device).pwfx as *mut core::ffi::c_void, wfex as *const core::ffi::c_void, cp_size as usize);
            }

            if err != DSERR_BUFFERLOST && (err < 0) {
                let size: DWORD = DSOUND_GetFormatSize(oldpwfx as LPCWAVEFORMATEX);
                WARN!("IDsDriverBuffer_SetFormat failed\n");
                if forced == 0 {
                    CopyMemory!((*device).pwfx as *mut core::ffi::c_void, oldpwfx as *const core::ffi::c_void, size as usize);
                    err = (DS_OK as i32);
                }
                break 'done;
            }

            if err == S_FALSE {
                /* ALSA specific: S_FALSE tells that recreation was successful,
                 * but size and location may be changed, and buffer has to be restarted
                 * I put it here, so if frequency doesn't match the error will be changed to DSERR_BUFFERLOST
                 * and the entire re-initialization will occur anyway
                 */
                IDsDriverBuffer_Lock(
                    (*device).hwbuf,
                    core::ptr::addr_of_mut!((*device).buffer) as *mut LPVOID,
                    core::ptr::addr_of_mut!((*device).buflen),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    0,
                    0,
                    DSBLOCK_ENTIREBUFFER,
                );
                IDsDriverBuffer_Unlock(
                    (*device).hwbuf,
                    ((*device).buffer).cast(),
                    0,
                    core::ptr::null_mut(),
                    0,
                );

                if (*device).state == STATE_PLAYING {
                    (*device).state = STATE_STARTING;
                } else if (*device).state == STATE_STOPPING {
                    (*device).state = STATE_STOPPED;
                }
                (*device).mixpos = 0;
                (*device).playpos = (*device).mixpos;
                (*device).pwqueue = (*device).playpos as UINT;
                (*device).pwplay = (*device).pwqueue;
                err = (DS_OK as i32);
            }
            DSOUND_RecalcPrimary(device);
        }

        if err == DSERR_BUFFERLOST {
            DSOUND_PrimaryClose(device);

            err = DSOUND_ReopenDevice(device, (FALSE) as BOOL);
            if err < 0 {
                WARN!("DSOUND_ReopenDevice failed: %08x\n", err);
                break 'done;
            }
            err = DSOUND_PrimaryOpen(device);
            if err != (DS_OK as HRESULT) {
                WARN!("DSOUND_PrimaryOpen failed\n");
                break 'done;
            }

            if (*wfex).nSamplesPerSec / 100 != (*(*device).pwfx).nSamplesPerSec / 100
                && forced != 0
                && !(*device).buffer.is_null()
            {
                DSOUND_PrimaryClose(device);
                (*(*device).pwfx).nSamplesPerSec = (*wfex).nSamplesPerSec;
                err = DSOUND_ReopenDevice(device, (TRUE) as BOOL);
                if err < 0 {
                    WARN!("DSOUND_ReopenDevice(2) failed: %08x\n", err);
                } else {
                    err = DSOUND_PrimaryOpen(device);
                    if err < 0 {
                        WARN!("DSOUND_PrimaryOpen(2) failed: %08x\n", err);
                    }
                }
            }
        }

        (*device).mix_buffer_len = DSOUND_bufpos_to_mixpos(device, (*device).buflen);
        (*device).mix_buffer = HeapReAlloc(
            GetProcessHeap(),
            0 as DWORD,
            (*device).mix_buffer as PVOID,
            (*device).mix_buffer_len as SIZE_T,
        ) as LPBYTE;
        FillMemory!((*device).mix_buffer as *mut core::ffi::c_void, (*device).mix_buffer_len as usize, 0 as u8);
        (*device).mixfunction = mixfunctions[((*(*device).pwfx).wBitsPerSample as usize / 8 as usize - 1 as usize)];
        (*device).normfunction = normfunctions[((*(*device).pwfx).wBitsPerSample as usize / 8 as usize - 1 as usize)];

        if nSamplesPerSec != (*(*device).pwfx).nSamplesPerSec
            || bpp != (*(*device).pwfx).wBitsPerSample as DWORD
            || chans != (*(*device).pwfx).nChannels as DWORD
        {
            let mut dsb: *mut *mut IDirectSoundBufferImpl = (*device).buffers;
            i = 0;
            while i < (*device).nrofbuffers {
                /* **** */
                RtlAcquireResourceExclusive(core::ptr::addr_of_mut!((**dsb).lock), TRUE as BYTE);

                (**dsb).freqAdjust = ((((**dsb).freq as DWORD64) << DSOUND_FREQSHIFT)
                    / ((*(*device).pwfx).nSamplesPerSec as DWORD64)) as ULONG;
                DSOUND_RecalcFormat(*dsb);
                DSOUND_MixToTemporary(*dsb, 0 as DWORD, (**dsb).buflen, (FALSE) as BOOL);
                (**dsb).primary_mixpos = 0 as DWORD;

                RtlReleaseResource(core::ptr::addr_of_mut!((**dsb).lock));
                /* **** */
                i += 1;
                dsb = dsb.offset(1);
            }
        }
    }

    /* done: */
    LeaveCriticalSection(core::ptr::addr_of_mut!((*device).mixlock));
    RtlReleaseResource(core::ptr::addr_of_mut!((*device).buffer_list_lock));
    /* **** */

    HeapFree(GetProcessHeap(), 0 as DWORD, oldpwfx as PVOID);
    return err;
}

unsafe extern "C" fn impl_from_IDirectSoundBuffer(iface: *mut IDirectSoundBuffer) -> *mut IDirectSoundBufferImpl {
    return CONTAINING_RECORD!(iface, IDirectSoundBufferImpl, IDirectSoundBuffer8_iface);
}
unsafe extern "system" fn PrimaryBufferImpl_SetFormat(
    iface: LPDIRECTSOUNDBUFFER,
    wfex: LPCWAVEFORMATEX,
) -> HRESULT {
    let mut This: *mut IDirectSoundBufferImpl =
        impl_from_IDirectSoundBuffer(iface as *mut IDirectSoundBuffer);
    TRACE!("(%p,%p)\n", iface, wfex);
    return primarybuffer_SetFormat((*This).device, wfex);
}

unsafe extern "system" fn PrimaryBufferImpl_SetVolume(
    mut iface: LPDIRECTSOUNDBUFFER,
    mut vol: LONG,
) -> HRESULT {
    let mut This: *mut IDirectSoundBufferImpl =
        impl_from_IDirectSoundBuffer(iface as *mut IDirectSoundBuffer);
    let mut device: *mut DirectSoundDevice = (*This).device;
    let mut ampfactors: DWORD = 0;
    let mut hres: HRESULT = (DS_OK as i32);

    if (*This).dsbd.dwFlags & DSBCAPS_CTRLVOLUME == 0 {
        WARN!("control unavailable\n");
        return DSERR_CONTROLUNAVAIL;
    }

    if (vol > (DSBVOLUME_MAX as LONG)) || (vol < DSBVOLUME_MIN) {
        WARN!("invalid parameter: vol = %d\n", vol);
        return DSERR_INVALIDPARAM;
    }

    /* **** */
    EnterCriticalSection(core::ptr::addr_of_mut!((*device).mixlock));

    // Divergence (all four volume/pan paths): the C decodes `ampfactors` whether or not
    // waveOutGetVolume succeeded, so on a device without a volume control (this guest's
    // AC97 answers MMSYSERR_INVALPARAM) it turns an uninitialised stack word into the
    // volume -- -1295 on one i386 run, -10000 on amd64. A failed read keeps the volume and
    // pan last set, which is what Windows reports.
    if waveOutGetVolume((*device).hwo, &mut ampfactors) == MMSYSERR_NOERROR {
        (*device).volpan.dwTotalLeftAmpFactor = ampfactors & 0xffff;
        (*device).volpan.dwTotalRightAmpFactor = ampfactors >> 16;
        DSOUND_AmpFactorToVolPan(core::ptr::addr_of_mut!((*device).volpan));
    }
    if vol != (*device).volpan.lVolume {
        (*device).volpan.lVolume = vol;
        DSOUND_RecalcVolPan(core::ptr::addr_of_mut!((*device).volpan));
        if !(*device).hwbuf.is_null() {
            hres = IDsDriverBuffer_SetVolumePan(
                (*device).hwbuf as *mut IDsDriverBuffer,
                core::ptr::addr_of_mut!((*device).volpan),
            );
            if hres != (DS_OK as HRESULT) {
                WARN!("IDsDriverBuffer_SetVolumePan failed\n");
            }
        } else {
            ampfactors = ((*device).volpan.dwTotalLeftAmpFactor & 0xffff)
                | ((*device).volpan.dwTotalRightAmpFactor << 16);
            waveOutSetVolume((*device).hwo, ampfactors);
        }
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*device).mixlock));
    /* **** */

    return hres;
}

unsafe extern "system" fn PrimaryBufferImpl_GetVolume(
    iface: LPDIRECTSOUNDBUFFER,
    vol: LPLONG,
) -> HRESULT {
    let This: *mut IDirectSoundBufferImpl = impl_from_IDirectSoundBuffer(iface as *mut IDirectSoundBuffer);
    let device: *mut DirectSoundDevice = (*This).device;
    let mut ampfactors: DWORD = 0;

    if ((*This).dsbd.dwFlags & DSBCAPS_CTRLVOLUME) == 0 {
        WARN!("control unavailable\n");
        return DSERR_CONTROLUNAVAIL;
    }

    if vol.is_null() {
        WARN!("invalid parameter: vol = NULL\n");
        return DSERR_INVALIDPARAM;
    }

    if (*device).hwbuf.is_null() {
        if waveOutGetVolume((*device).hwo, core::ptr::addr_of_mut!(ampfactors)) == MMSYSERR_NOERROR {
            (*device).volpan.dwTotalLeftAmpFactor = ampfactors & 0xffff;
            (*device).volpan.dwTotalRightAmpFactor = ampfactors >> 16;
            DSOUND_AmpFactorToVolPan(core::ptr::addr_of_mut!((*device).volpan));
        }
    }
    *vol = (*device).volpan.lVolume;
    return (DS_OK) as HRESULT;
}

unsafe extern "system" fn PrimaryBufferImpl_SetFrequency(
    iface: LPDIRECTSOUNDBUFFER,
    freq: DWORD,
) -> HRESULT {
    let This: *mut IDirectSoundBufferImpl = impl_from_IDirectSoundBuffer(iface as *mut IDirectSoundBuffer);

    /* You cannot set the frequency of the primary buffer */
    WARN!("control unavailable\n");
    return DSERR_CONTROLUNAVAIL;
}

unsafe extern "system" fn PrimaryBufferImpl_Play(
    iface: LPDIRECTSOUNDBUFFER,
    reserved1: DWORD,
    reserved2: DWORD,
    flags: DWORD,
) -> HRESULT {
    let mut This: *mut IDirectSoundBufferImpl = impl_from_IDirectSoundBuffer(iface as *mut IDirectSoundBuffer);
    let mut device: *mut DirectSoundDevice = (*This).device;

    if (flags & DSBPLAY_LOOPING) == 0 {
        WARN!("invalid parameter: flags = %08x\n", flags);
        return DSERR_INVALIDPARAM;
    }

    /* **** */
    EnterCriticalSection(core::ptr::addr_of_mut!((*device).mixlock));

    if (*device).state == STATE_STOPPED {
        (*device).state = STATE_STARTING;
    } else if (*device).state == STATE_STOPPING {
        (*device).state = STATE_PLAYING;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*device).mixlock));
    /* **** */

    return (DS_OK) as HRESULT;
}

unsafe extern "system" fn PrimaryBufferImpl_Stop(iface: LPDIRECTSOUNDBUFFER) -> HRESULT {
    let mut This: *mut IDirectSoundBufferImpl =
        impl_from_IDirectSoundBuffer(iface as *mut IDirectSoundBuffer);
    let mut device: *mut DirectSoundDevice = (*This).device;

    /* **** */
    EnterCriticalSection(core::ptr::addr_of_mut!((*device).mixlock));

    if (*device).state == STATE_PLAYING as DWORD {
        (*device).state = STATE_STOPPING as DWORD;
    } else if (*device).state == STATE_STARTING as DWORD {
        (*device).state = STATE_STOPPED as DWORD;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*device).mixlock));
    /* **** */

    return DS_OK as HRESULT;
}

unsafe extern "system" fn PrimaryBufferImpl_AddRef(iface: LPDIRECTSOUNDBUFFER) -> ULONG {
    let This: *mut IDirectSoundBufferImpl = impl_from_IDirectSoundBuffer(iface as *mut IDirectSoundBuffer);
    let mut ref_0: ULONG = InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_) as *mut core::ffi::c_long) as ULONG;
    TRACE!("(%p) ref was %d\n", This, ref_0.wrapping_sub(1));
    if ref_0 == 1 {
        InterlockedIncrement(core::ptr::addr_of_mut!((*This).numIfaces) as *mut core::ffi::c_long);
    }
    return ref_0;
}
#[no_mangle]
pub unsafe extern "C" fn primarybuffer_destroy(This: *mut IDirectSoundBufferImpl) {
    (*(*This).device).primary = core::ptr::null_mut();
    HeapFree(GetProcessHeap(), 0 as DWORD, This as LPVOID);
}

unsafe extern "system" fn PrimaryBufferImpl_Release(iface: LPDIRECTSOUNDBUFFER) -> ULONG {
    let This: *mut IDirectSoundBufferImpl = impl_from_IDirectSoundBuffer(iface as *mut IDirectSoundBuffer);
    let ref_0: DWORD = InterlockedDecrement(core::ptr::addr_of_mut!((*This).ref_) as *mut core::ffi::c_long) as DWORD;

    if ref_0 == 0 && InterlockedDecrement(core::ptr::addr_of_mut!((*This).numIfaces) as *mut core::ffi::c_long) == 0 {
        primarybuffer_destroy(This);
    }
    return ref_0 as ULONG;
}

unsafe extern "system" fn PrimaryBufferImpl_GetCurrentPosition(
    iface: LPDIRECTSOUNDBUFFER,
    playpos: LPDWORD,
    writepos: LPDWORD,
) -> HRESULT {
    let mut hres: HRESULT;
    let mut This: *mut IDirectSoundBufferImpl = impl_from_IDirectSoundBuffer(iface);
    let mut device: *mut DirectSoundDevice = (*This).device;

    /* **** */
    EnterCriticalSection(core::ptr::addr_of_mut!((*device).mixlock));

    hres = DSOUND_PrimaryGetPosition(device, playpos, writepos);
    if hres != (DS_OK as HRESULT) {
        WARN!("DSOUND_PrimaryGetPosition failed\n");
        LeaveCriticalSection(core::ptr::addr_of_mut!((*device).mixlock));
        return hres;
    }
    if !writepos.is_null() {
        if (*device).state != STATE_STOPPED {
            /* apply the documented 10ms lead to writepos */
            *writepos = (*writepos).wrapping_add((*device).writelead);
        }
        while *writepos >= (*device).buflen {
            *writepos = (*writepos).wrapping_sub((*device).buflen);
        }
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*device).mixlock));
    /* **** */

    return (DS_OK) as HRESULT;
}

unsafe extern "system" fn PrimaryBufferImpl_GetStatus(
    iface: LPDIRECTSOUNDBUFFER,
    status: LPDWORD,
) -> HRESULT {
    let This: *mut IDirectSoundBufferImpl = impl_from_IDirectSoundBuffer(iface as *mut IDirectSoundBuffer);
    let device: *mut DirectSoundDevice = (*This).device;

    if status.is_null() {
        WARN!("invalid parameter: status == NULL\n");
        return DSERR_INVALIDPARAM;
    }

    *status = 0;
    if ((*device).state == STATE_STARTING) ||
        ((*device).state == STATE_PLAYING)
    {
        *status |= DSBSTATUS_PLAYING | DSBSTATUS_LOOPING;
    }

    return (DS_OK) as HRESULT;
}

unsafe extern "system" fn PrimaryBufferImpl_GetFormat(
    iface: LPDIRECTSOUNDBUFFER,
    lpwf: LPWAVEFORMATEX,
    wfsize: DWORD,
    wfwritten: LPDWORD,
) -> HRESULT {
    let mut size: DWORD;
    let This: *mut IDirectSoundBufferImpl = impl_from_IDirectSoundBuffer(iface as *mut IDirectSoundBuffer);
    let device: *mut DirectSoundDevice = (*This).device;

    size = (core::mem::size_of::<WAVEFORMATEX>() as DWORD).wrapping_add((*(*device).pwfx).cbSize as DWORD);

    if !lpwf.is_null() {
        if wfsize >= size {
            CopyMemory!(lpwf as *mut core::ffi::c_void, (*device).pwfx as *const core::ffi::c_void, size as usize);
            if !wfwritten.is_null() {
                *wfwritten = size;
            }
        } else {
            WARN!("invalid parameter: wfsize too small\n");
            if !wfwritten.is_null() {
                *wfwritten = 0;
            }
            return DSERR_INVALIDPARAM;
        }
    } else {
        if !wfwritten.is_null() {
            *wfwritten = (core::mem::size_of::<WAVEFORMATEX>() as DWORD).wrapping_add((*(*device).pwfx).cbSize as DWORD);
        } else {
            WARN!("invalid parameter: wfwritten == NULL\n");
            return DSERR_INVALIDPARAM;
        }
    }

    return (DS_OK) as HRESULT;
}

unsafe extern "system" fn PrimaryBufferImpl_Lock(
    iface: LPDIRECTSOUNDBUFFER,
    mut writecursor: DWORD,
    mut writebytes: DWORD,
    lplpaudioptr1: *mut LPVOID,
    audiobytes1: LPDWORD,
    lplpaudioptr2: *mut LPVOID,
    audiobytes2: LPDWORD,
    flags: DWORD,
) -> HRESULT {
    let mut hres: HRESULT;
    let This: *mut IDirectSoundBufferImpl =
        impl_from_IDirectSoundBuffer(iface as *mut IDirectSoundBuffer);
    let device: *mut DirectSoundDevice = (*This).device;

    if audiobytes1.is_null() {
        return DSERR_INVALIDPARAM;
    }

    if (*device).priolevel != DSSCL_WRITEPRIMARY {
        WARN!("failed priority check!\n");
        return DSERR_PRIOLEVELNEEDED;
    }

    /* when this flag is set, writecursor is meaningless and must be calculated */
    if (flags & DSBLOCK_FROMWRITECURSOR) != 0 {
        /* GetCurrentPosition does too much magic to duplicate here */
        hres = IDirectSoundBuffer_GetCurrentPosition(
            iface,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(writecursor),
        );
        if hres != (DS_OK as HRESULT) {
            WARN!("IDirectSoundBuffer_GetCurrentPosition failed\n");
            return hres;
        }
    }

    /* when this flag is set, writebytes is meaningless and must be set */
    if (flags & DSBLOCK_ENTIREBUFFER) != 0 {
        writebytes = (*device).buflen;
    }

    if writecursor >= (*device).buflen {
        WARN!(
            "Invalid parameter, writecursor: %u >= buflen: %u\n",
            writecursor,
            (*device).buflen
        );
        return DSERR_INVALIDPARAM;
    }

    if writebytes > (*device).buflen {
        WARN!(
            "Invalid parameter, writebytes: %u > buflen: %u\n",
            writebytes,
            (*device).buflen
        );
        return DSERR_INVALIDPARAM;
    }

    if ((*device).drvdesc.dwFlags & DSDDESC_DONTNEEDPRIMARYLOCK) == 0 && !(*device).hwbuf.is_null() {
        hres = IDsDriverBuffer_Lock(
            (*device).hwbuf,
            lplpaudioptr1,
            audiobytes1,
            lplpaudioptr2,
            audiobytes2,
            writecursor,
            writebytes,
            0,
        );
        if hres != (DS_OK as HRESULT) {
            WARN!("IDsDriverBuffer_Lock failed\n");
            return hres;
        }
    } else {
        if writecursor.wrapping_add(writebytes) <= (*device).buflen {
            *(lplpaudioptr1 as *mut LPBYTE) = (*device).buffer.offset(writecursor as isize);
            *audiobytes1 = writebytes;
            if !lplpaudioptr2.is_null() {
                *(lplpaudioptr2 as *mut LPBYTE) = core::ptr::null_mut::<BYTE>();
            }
            if !audiobytes2.is_null() {
                *audiobytes2 = 0;
            }
        } else {
            *(lplpaudioptr1 as *mut LPBYTE) = (*device).buffer.offset(writecursor as isize);
            *audiobytes1 = (*device).buflen.wrapping_sub(writecursor);
            if !lplpaudioptr2.is_null() {
                *(lplpaudioptr2 as *mut LPBYTE) = (*device).buffer;
            }
            if !audiobytes2.is_null() {
                *audiobytes2 = writebytes.wrapping_sub((*device).buflen.wrapping_sub(writecursor));
            }
        }
    }
    return (DS_OK) as HRESULT;
}

unsafe extern "system" fn PrimaryBufferImpl_SetCurrentPosition(
    iface: LPDIRECTSOUNDBUFFER,
    newpos: DWORD,
) -> HRESULT {
    let mut This: *mut IDirectSoundBufferImpl = impl_from_IDirectSoundBuffer(iface as *mut IDirectSoundBuffer);

    /* You cannot set the position of the primary buffer */
    WARN!("invalid call\n");
    return DSERR_INVALIDCALL;
}

unsafe extern "system" fn PrimaryBufferImpl_SetPan(
    iface: LPDIRECTSOUNDBUFFER,
    pan: LONG,
) -> HRESULT {
    let This: *mut IDirectSoundBufferImpl = impl_from_IDirectSoundBuffer(iface as *mut IDirectSoundBuffer);
    let device: *mut DirectSoundDevice = (*This).device;
    let mut ampfactors: DWORD = 0;
    let mut hres: HRESULT = (DS_OK as i32);

    if ((*This).dsbd.dwFlags & DSBCAPS_CTRLPAN) == 0 {
        WARN!("control unavailable\n");
        return DSERR_CONTROLUNAVAIL;
    }

    if (pan > (DSBPAN_RIGHT as LONG)) || (pan < DSBPAN_LEFT) {
        WARN!("invalid parameter: pan = %d\n", pan);
        return DSERR_INVALIDPARAM;
    }

    /* **** */
    EnterCriticalSection(core::ptr::addr_of_mut!((*device).mixlock));

    if (*device).hwbuf.is_null() {
        if waveOutGetVolume((*device).hwo, core::ptr::addr_of_mut!(ampfactors)) == MMSYSERR_NOERROR {
            (*device).volpan.dwTotalLeftAmpFactor = ampfactors & 0xffff;
            (*device).volpan.dwTotalRightAmpFactor = ampfactors >> 16;
            DSOUND_AmpFactorToVolPan(core::ptr::addr_of_mut!((*device).volpan));
        }
    }
    if pan != (*device).volpan.lPan {
        (*device).volpan.lPan = pan;
        DSOUND_RecalcVolPan(core::ptr::addr_of_mut!((*device).volpan));
        if !(*device).hwbuf.is_null() {
            hres = IDsDriverBuffer_SetVolumePan(
                (*device).hwbuf as *mut IDsDriverBuffer,
                core::ptr::addr_of_mut!((*device).volpan),
            );
            if hres != (DS_OK as HRESULT) {
                WARN!("IDsDriverBuffer_SetVolumePan failed\n");
            }
        } else {
            ampfactors = ((*device).volpan.dwTotalLeftAmpFactor & 0xffff)
                | ((*device).volpan.dwTotalRightAmpFactor << 16);
            waveOutSetVolume((*device).hwo, ampfactors);
        }
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*device).mixlock));
    /* **** */

    return hres;
}

unsafe extern "system" fn PrimaryBufferImpl_GetPan(
    iface: LPDIRECTSOUNDBUFFER,
    pan: LPLONG,
) -> HRESULT {
    let This: *mut IDirectSoundBufferImpl =
        impl_from_IDirectSoundBuffer(iface as *mut IDirectSoundBuffer);
    let device: *mut DirectSoundDevice = (*This).device;
    let mut ampfactors: DWORD = 0;

    if ((*This).dsbd.dwFlags & DSBCAPS_CTRLPAN) == 0 {
        WARN!("control unavailable\n");
        return DSERR_CONTROLUNAVAIL;
    }

    if pan.is_null() {
        WARN!("invalid parameter: pan == NULL\n");
        return DSERR_INVALIDPARAM;
    }

    if (*device).hwbuf.is_null() {
        if waveOutGetVolume((*device).hwo, core::ptr::addr_of_mut!(ampfactors)) == MMSYSERR_NOERROR {
            (*device).volpan.dwTotalLeftAmpFactor = ampfactors & 0xffff;
            (*device).volpan.dwTotalRightAmpFactor = ampfactors >> 16;
            DSOUND_AmpFactorToVolPan(core::ptr::addr_of_mut!((*device).volpan));
        }
    }
    *pan = (*device).volpan.lPan;
    return (DS_OK) as HRESULT;
}

unsafe extern "system" fn PrimaryBufferImpl_Unlock(
	iface: LPDIRECTSOUNDBUFFER,
	p1: LPVOID,
	x1: DWORD,
	p2: LPVOID,
	x2: DWORD,
) -> HRESULT {
        let mut This: *mut IDirectSoundBufferImpl = impl_from_IDirectSoundBuffer(iface);
        let mut device: *mut DirectSoundDevice = (*This).device;

	if (*device).priolevel != DSSCL_WRITEPRIMARY {
		WARN!("failed priority check!\n");
		return DSERR_PRIOLEVELNEEDED;
	}

	if ((*device).drvdesc.dwFlags & DSDDESC_DONTNEEDPRIMARYLOCK) == 0 && !(*device).hwbuf.is_null() {
		let mut hres: HRESULT;

		if (p1 as *mut u8).offset_from((*device).buffer as *mut u8) as i64 + x1 as i64
			> (*device).buflen as i64
		{
			hres = DSERR_INVALIDPARAM;
		} else {
			hres = IDsDriverBuffer_Unlock((*device).hwbuf, p1, x1, p2, x2);
		}

		if hres != (DS_OK as HRESULT) {
			WARN!("IDsDriverBuffer_Unlock failed\n");
			return hres;
		}
	}

	return (DS_OK) as HRESULT;
}

unsafe extern "system" fn PrimaryBufferImpl_Restore(iface: LPDIRECTSOUNDBUFFER) -> HRESULT {
    let mut This: *mut IDirectSoundBufferImpl = impl_from_IDirectSoundBuffer(iface as *mut IDirectSoundBuffer);
    return (DS_OK) as HRESULT;
}

unsafe extern "system" fn PrimaryBufferImpl_GetFrequency(
    iface: LPDIRECTSOUNDBUFFER,
    freq: LPDWORD,
) -> HRESULT {
    let This: *mut IDirectSoundBufferImpl = impl_from_IDirectSoundBuffer(iface as *mut IDirectSoundBuffer);
    let device: *mut DirectSoundDevice = (*This).device;

    TRACE!("(%p,%p)\n", iface, freq);

    if freq.is_null() {
        WARN!("invalid parameter: freq == NULL\n");
        return DSERR_INVALIDPARAM as HRESULT;
    }

    if ((*This).dsbd.dwFlags & (DSBCAPS_CTRLFREQUENCY as DWORD)) == 0 {
        WARN!("control unavailable\n");
        return DSERR_CONTROLUNAVAIL as HRESULT;
    }

    *freq = (*(*device).pwfx).nSamplesPerSec;

    TRACE!("-> %d\n", *freq);

    return DS_OK as HRESULT;
}
unsafe extern "system" fn PrimaryBufferImpl_Initialize(
    iface: LPDIRECTSOUNDBUFFER,
    dsound: LPDIRECTSOUND,
    dbsd: LPCDSBUFFERDESC,
) -> HRESULT {
    let mut This: *mut IDirectSoundBufferImpl =
        impl_from_IDirectSoundBuffer(iface as *mut IDirectSoundBuffer);
    WARN!("(%p) already initialized\n", This);
    return DSERR_ALREADYINITIALIZED;
}

unsafe extern "system" fn PrimaryBufferImpl_GetCaps(
    iface: LPDIRECTSOUNDBUFFER,
    caps: LPDSBCAPS,
) -> HRESULT {
    let mut This: *mut IDirectSoundBufferImpl = impl_from_IDirectSoundBuffer(iface as *mut IDirectSoundBuffer);
    let mut device: *mut DirectSoundDevice = (*This).device;

    if caps.is_null() {
        WARN!("invalid parameter: caps == NULL\n");
        return DSERR_INVALIDPARAM;
    }

    if ((*caps).dwSize as usize) < core::mem::size_of::<DSBCAPS>() as usize {
        WARN!("invalid parameter: caps->dwSize = %d\n", (*caps).dwSize);
        return DSERR_INVALIDPARAM;
    }

    (*caps).dwFlags = (*This).dsbd.dwFlags;
    (*caps).dwBufferBytes = (*device).buflen;

    /* Windows reports these as zero */
    (*caps).dwUnlockTransferRate = 0;
    (*caps).dwPlayCpuOverhead = 0;

    return (DS_OK) as HRESULT;
}

unsafe extern "system" fn PrimaryBufferImpl_QueryInterface(
	iface: LPDIRECTSOUNDBUFFER,
	riid: REFIID,
	ppobj: *mut LPVOID,
) -> HRESULT {
        let This: *mut IDirectSoundBufferImpl = impl_from_IDirectSoundBuffer(iface as *mut IDirectSoundBuffer);
        let device: *mut DirectSoundDevice = (*This).device;

	if ppobj.is_null() {
		WARN!("invalid parameter\n");
		return E_INVALIDARG;
	}

	*ppobj = core::ptr::null_mut(); /* assume failure */

	if IsEqualGUID!(riid, core::ptr::addr_of!(IID_IUnknown))!= false
		|| IsEqualGUID!(riid, core::ptr::addr_of!(IID_IDirectSoundBuffer)) != false
	{
		IDirectSoundBuffer_AddRef(This as LPDIRECTSOUNDBUFFER);
		*ppobj = This as LPVOID;
		return S_OK;
	}

	/* DirectSoundBuffer and DirectSoundBuffer8 are different and */
	/* a primary buffer can't have a DirectSoundBuffer8 interface */
	if IsEqualGUID!(core::ptr::addr_of!(IID_IDirectSoundBuffer8), riid) != false {
		WARN!("app requested DirectSoundBuffer8 on primary buffer\n");
		return E_NOINTERFACE;
	}

	if IsEqualGUID!(core::ptr::addr_of!(IID_IDirectSoundNotify), riid) != false {
		ERR!("app requested IDirectSoundNotify on primary buffer\n");
		/* FIXME: should we support this? */
		return E_NOINTERFACE;
	}

	if IsEqualGUID!(core::ptr::addr_of!(IID_IDirectSound3DBuffer), riid) != false {
		ERR!("app requested IDirectSound3DBuffer on primary buffer\n");
		return E_NOINTERFACE;
	}

        if IsEqualGUID!(core::ptr::addr_of!(IID_IDirectSound3DListener), riid) != false {
		if (*device).listener.is_null() {
			IDirectSound3DListenerImpl_Create(device, core::ptr::addr_of_mut!((*device).listener));
		}
		if !(*device).listener.is_null() {
			*ppobj = (*device).listener as LPVOID;
			IDirectSound3DListener_AddRef(*ppobj as LPDIRECTSOUND3DLISTENER);
			return S_OK;
		}

		WARN!("IID_IDirectSound3DListener failed\n");
		return E_NOINTERFACE;
	}

	if IsEqualGUID!(core::ptr::addr_of!(IID_IKsPropertySet), riid) != false {
		/* FIXME: IKsPropertySet is not implemented on primary buffers */
		return E_NOINTERFACE;
	}

	/* FIXME: unknown interface */
	return E_NOINTERFACE;
}
#[no_mangle]
pub unsafe extern "C" fn primarybuffer_create(
    device: *mut DirectSoundDevice,
    ppdsb: *mut *mut IDirectSoundBufferImpl,
    dsbd: *const DSBUFFERDESC,
) -> HRESULT {
    let mut dsb: *mut IDirectSoundBufferImpl = core::ptr::null_mut::<IDirectSoundBufferImpl>();

    TRACE!("%p,%p,%p)\n", device, ppdsb, dsbd);

    if !(*dsbd).lpwfxFormat.is_null() {
        WARN!("invalid parameter: dsbd->lpwfxFormat != NULL\n");
        *ppdsb = core::ptr::null_mut::<IDirectSoundBufferImpl>();
        return DSERR_INVALIDPARAM;
    }

    dsb = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY as DWORD,
        core::mem::size_of::<IDirectSoundBufferImpl>() as SIZE_T,
    ) as *mut IDirectSoundBufferImpl;

    if dsb.is_null() {
        WARN!("out of memory\n");
        *ppdsb = core::ptr::null_mut::<IDirectSoundBufferImpl>();
        return DSERR_OUTOFMEMORY;
    }

    (*dsb).ref_ = 1 as LONG;
    (*dsb).numIfaces = 1 as LONG;
    (*dsb).device = device;
    (*dsb).IDirectSoundBuffer8_iface.lpVtbl = &raw const dspbvt as *mut IDirectSoundBuffer8Vtbl;
    (*dsb).dsbd = *dsbd;

    TRACE!("Created primary buffer at %p\n", dsb);

    TRACE!(
        "(formattag=0x%04x,chans=%d,samplerate=%d,bytespersec=%d,blockalign=%d,bitspersamp=%d,cbSize=%d)\n",
        (*(*device).pwfx).wFormatTag as core::ffi::c_int,
        (*(*device).pwfx).nChannels as core::ffi::c_int,
        (*(*device).pwfx).nSamplesPerSec,
        (*(*device).pwfx).nAvgBytesPerSec,
        (*(*device).pwfx).nBlockAlign as core::ffi::c_int,
        (*(*device).pwfx).wBitsPerSample as core::ffi::c_int,
        (*(*device).pwfx).cbSize as core::ffi::c_int,
    );

    *ppdsb = dsb;
    return S_OK;
}

// COM vtable tables — generated from primary.c by vtbl_gen.py
/// primary.c:1198 `static const IDirectSoundBufferVtbl dspbvt`  (slot order: rust:types_amd64.rs)
pub static dspbvt: IDirectSoundBufferVtbl = IDirectSoundBufferVtbl {
    QueryInterface: Some(PrimaryBufferImpl_QueryInterface),
    AddRef: Some(PrimaryBufferImpl_AddRef),
    Release: Some(PrimaryBufferImpl_Release),
    GetCaps: Some(PrimaryBufferImpl_GetCaps),
    GetCurrentPosition: Some(PrimaryBufferImpl_GetCurrentPosition),
    GetFormat: Some(PrimaryBufferImpl_GetFormat),
    GetVolume: Some(PrimaryBufferImpl_GetVolume),
    GetPan: Some(PrimaryBufferImpl_GetPan),
    GetFrequency: Some(PrimaryBufferImpl_GetFrequency),
    GetStatus: Some(PrimaryBufferImpl_GetStatus),
    Initialize: Some(PrimaryBufferImpl_Initialize),
    Lock: Some(PrimaryBufferImpl_Lock),
    Play: Some(PrimaryBufferImpl_Play),
    SetCurrentPosition: Some(PrimaryBufferImpl_SetCurrentPosition),
    SetFormat: Some(PrimaryBufferImpl_SetFormat),
    SetVolume: Some(PrimaryBufferImpl_SetVolume),
    SetPan: Some(PrimaryBufferImpl_SetPan),
    SetFrequency: Some(PrimaryBufferImpl_SetFrequency),
    Stop: Some(PrimaryBufferImpl_Stop),
    Unlock: Some(PrimaryBufferImpl_Unlock),
    Restore: Some(PrimaryBufferImpl_Restore),
};
