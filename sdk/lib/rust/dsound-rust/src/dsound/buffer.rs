use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----

unsafe extern "system" fn IDirectSoundNotifyImpl_QueryInterface(
    iface: LPDIRECTSOUNDNOTIFY,
    riid: REFIID,
    ppobj: *mut LPVOID,
) -> HRESULT {
    let This: *mut IDirectSoundNotifyImpl = iface as *mut IDirectSoundNotifyImpl;

    if (*This).dsb.is_null() {
        WARN!("invalid parameter\n");
        return E_INVALIDARG;
    }

    return IDirectSoundBuffer_QueryInterface((*This).dsb as LPDIRECTSOUNDBUFFER, riid, ppobj);
}
unsafe extern "system" fn IDirectSoundNotifyImpl_AddRef(iface: LPDIRECTSOUNDNOTIFY) -> ULONG {
    let mut This: *mut IDirectSoundNotifyImpl = iface as *mut IDirectSoundNotifyImpl;
    let mut ref_0: ULONG = InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_) as *mut i32) as ULONG;
    WINE_TRACE!("(%p) ref was %d\n", This, ref_0.wrapping_sub(1 as ULONG));
    return ref_0;
}
unsafe extern "system" fn IDirectSoundNotifyImpl_Release(iface: LPDIRECTSOUNDNOTIFY) -> ULONG {
    let This: *mut IDirectSoundNotifyImpl = iface as *mut IDirectSoundNotifyImpl;
    let ref_0: ULONG = InterlockedDecrement(core::ptr::addr_of_mut!((*This).ref_) as *mut ::core::ffi::c_long) as ULONG;

    if ref_0 == 0 {
        (*(*This).dsb).notify = core::ptr::null_mut::<IDirectSoundNotifyImpl>();
        IDirectSoundBuffer_Release((*This).dsb as LPDIRECTSOUNDBUFFER);
        HeapFree(GetProcessHeap(), 0, This as LPVOID);
    }
    return ref_0;
}

unsafe extern "system" fn IDirectSoundNotifyImpl_SetNotificationPositions(
	iface: LPDIRECTSOUNDNOTIFY,
	howmuch: DWORD,
	notify: LPCDSBPOSITIONNOTIFY,
) -> HRESULT {
	let mut This: *mut IDirectSoundNotifyImpl = iface as *mut IDirectSoundNotifyImpl;

	if howmuch > 0 && notify.is_null() {
		WARN!("invalid parameter: notify == NULL\n");
		return DSERR_INVALIDPARAM;
	}

	if ((TRACE_ON!(__wine_dbch_dsound)) != 0) {
		let mut i: u32 = 0;
		i = 0;
		while i < howmuch {
			TRACE!("notify at %d to %p\n",
				(*notify.offset(i as isize)).dwOffset,
				(*notify.offset(i as isize)).hEventNotify);
			i = i.wrapping_add(1);
		}
	}

	if !(*(*This).dsb).hwnotify.is_null() {
		let mut hres: HRESULT = 0;
		hres = IDsDriverNotify_SetNotificationPositions((*(*This).dsb).hwnotify, howmuch, notify);
		if hres != (DS_OK as HRESULT) {
			WARN!("IDsDriverNotify_SetNotificationPositions failed\n");
		}
		return hres;
	} else if howmuch > 0 {
		/* Make an internal copy of the caller-supplied array.
		 * Replace the existing copy if one is already present. */
		HeapFree(GetProcessHeap(), 0, (*(*This).dsb).notifies as LPVOID);
		(*(*This).dsb).notifies = HeapAlloc(GetProcessHeap(), 0,
			(howmuch as SIZE_T).wrapping_mul(core::mem::size_of::<DSBPOSITIONNOTIFY>() as SIZE_T)) as LPDSBPOSITIONNOTIFY;

		if (*(*This).dsb).notifies.is_null() {
			WARN!("out of memory\n");
			return DSERR_OUTOFMEMORY;
		}
		CopyMemory!((*(*This).dsb).notifies as LPVOID, notify as LPVOID,
			(howmuch as SIZE_T).wrapping_mul(core::mem::size_of::<DSBPOSITIONNOTIFY>() as SIZE_T));
		(*(*This).dsb).nrofnotifies = (howmuch) as i32;
	} else {
		HeapFree(GetProcessHeap(), 0, (*(*This).dsb).notifies as LPVOID);
		(*(*This).dsb).notifies = core::ptr::null_mut();
		(*(*This).dsb).nrofnotifies = 0;
	}

	return S_OK;
}

unsafe extern "C" fn IDirectSoundNotifyImpl_Create(
    mut dsb: *mut IDirectSoundBufferImpl,
    mut pdsn: *mut *mut IDirectSoundNotifyImpl,
) -> HRESULT {
    let mut dsn: *mut IDirectSoundNotifyImpl = core::ptr::null_mut();

    dsn = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY as DWORD,
        (core::mem::size_of::<IDirectSoundNotifyImpl>()) as SIZE_T,
    ) as *mut IDirectSoundNotifyImpl;

    if dsn.is_null() {
        WARN!("out of memory\n");
        return DSERR_OUTOFMEMORY;
    }

    (*dsn).ref_ = 0 as LONG;
    (*dsn).lpVtbl = core::ptr::addr_of!(dsnvt);
    (*dsn).dsb = dsb;
    (*dsb).notify = dsn;
    IDirectSoundBuffer_AddRef(dsb as LPDIRECTSOUNDBUFFER);

    *pdsn = dsn;
    return DS_OK as HRESULT;
}

unsafe extern "C" fn IDirectSoundNotifyImpl_Destroy(
    pdsn: *mut IDirectSoundNotifyImpl,
) -> HRESULT {
    while IDirectSoundNotifyImpl_Release(pdsn as LPDIRECTSOUNDNOTIFY) > 0 {}

    return (DS_OK) as HRESULT;
}

unsafe extern "C" fn impl_from_IDirectSoundBuffer8(iface: *mut IDirectSoundBuffer8) -> *mut IDirectSoundBufferImpl {
    return (iface as usize).wrapping_sub(core::mem::offset_of!(IDirectSoundBufferImpl, IDirectSoundBuffer8_iface)) as *mut IDirectSoundBufferImpl;
}

unsafe extern "C" fn is_primary_buffer(This: *mut IDirectSoundBufferImpl) -> BOOL {
    return if (*This).dsbd.dwFlags & DSBCAPS_PRIMARYBUFFER as DWORD != 0 {
        (TRUE as i32)
    } else {
        (FALSE as i32)
    };
}

unsafe extern "system" fn IDirectSoundBufferImpl_SetFormat(iface: *mut IDirectSoundBuffer8, wfex: LPCWAVEFORMATEX) -> HRESULT {
    let mut This: *mut IDirectSoundBufferImpl = impl_from_IDirectSoundBuffer8(iface);
    TRACE!("(%p,%p)\n", iface, wfex);
    if is_primary_buffer(This) != 0 {
        return primarybuffer_SetFormat((*This).device, wfex);
    } else {
        WARN!("not available for secondary buffers.\n");
        return DSERR_INVALIDCALL;
    }
}

unsafe extern "system" fn IDirectSoundBufferImpl_SetVolume(iface: *mut IDirectSoundBuffer8, vol: LONG) -> HRESULT {
    let This: *mut IDirectSoundBufferImpl = impl_from_IDirectSoundBuffer8(iface);
    let mut oldVol: LONG = 0;

    let mut hres: HRESULT = (DS_OK as i32);

    if ((*This).dsbd.dwFlags & DSBCAPS_CTRLVOLUME) == 0 {
        WARN!("control unavailable: This->dsbd.dwFlags = 0x%08x\n", (*This).dsbd.dwFlags);
        return DSERR_CONTROLUNAVAIL;
    }

    if (vol > (DSBVOLUME_MAX as LONG)) || (vol < DSBVOLUME_MIN) {
        WARN!("invalid parameter: vol = %d\n", vol);
        return DSERR_INVALIDPARAM;
    }

    /* **** */
    RtlAcquireResourceExclusive(core::ptr::addr_of_mut!((*This).lock), TRUE as BYTE);

    if ((*This).dsbd.dwFlags & DSBCAPS_CTRL3D) != 0 {
        oldVol = (*This).ds3db_lVolume;
        (*This).ds3db_lVolume = vol;
        if vol != oldVol {
            /* recalc 3d volume, which in turn recalcs the pans */
            DSOUND_Calc3DBuffer(This);
        }
    } else {
        oldVol = (*This).volpan.lVolume;
        (*This).volpan.lVolume = vol;
        if vol != oldVol {
            DSOUND_RecalcVolPan(core::ptr::addr_of_mut!((*This).volpan));
        }
    }

    if vol != oldVol {
        if !(*This).hwbuf.is_null() {
            hres = IDsDriverBuffer_SetVolumePan((*This).hwbuf, core::ptr::addr_of_mut!((*This).volpan));
            if hres != (DS_OK as HRESULT) {
                WARN!("IDsDriverBuffer_SetVolumePan failed\n");
            }
        }
    }

    RtlReleaseResource(core::ptr::addr_of_mut!((*This).lock));
    /* **** */

    return hres;
}

unsafe extern "system" fn IDirectSoundBufferImpl_GetVolume(iface: *mut IDirectSoundBuffer8, vol: *mut LONG) -> HRESULT {
        let This: *mut IDirectSoundBufferImpl = impl_from_IDirectSoundBuffer8(iface);

	if ((*This).dsbd.dwFlags & DSBCAPS_CTRLVOLUME) == 0 {
		WARN!("control unavailable\n");
		return DSERR_CONTROLUNAVAIL;
	}

	if vol.is_null() {
		WARN!("invalid parameter: vol == NULL\n");
		return DSERR_INVALIDPARAM;
	}

	*vol = (*This).volpan.lVolume;

	return (DS_OK) as HRESULT;
}
unsafe extern "system" fn IDirectSoundBufferImpl_SetFrequency(iface: *mut IDirectSoundBuffer8, mut freq: DWORD) -> HRESULT {
    let mut This: *mut IDirectSoundBufferImpl = impl_from_IDirectSoundBuffer8(iface);
    let mut oldFreq: DWORD;

    if is_primary_buffer(This) != 0 {
        WARN!("not available for primary buffers.\n");
        return DSERR_CONTROLUNAVAIL as HRESULT;
    }

    if !((*This).dsbd.dwFlags & (DSBCAPS_CTRLFREQUENCY as DWORD) != 0) {
        WARN!("control unavailable\n");
        return DSERR_CONTROLUNAVAIL as HRESULT;
    }

    if freq == (DSBFREQUENCY_ORIGINAL as DWORD) {
        freq = (*(*This).pwfx).nSamplesPerSec;
    }

    if (freq < (DSBFREQUENCY_MIN as DWORD)) || (freq > (DSBFREQUENCY_MAX as DWORD)) {
        WARN!("invalid parameter: freq = %d\n", freq);
        return DSERR_INVALIDPARAM as HRESULT;
    }

    /* **** */
    RtlAcquireResourceExclusive(core::ptr::addr_of_mut!((*This).lock), TRUE as BYTE);

    oldFreq = (*This).freq;
    (*This).freq = freq;
    if freq != oldFreq {
        (*This).freqAdjust = ((((*This).freq as DWORD64) << DSOUND_FREQSHIFT)
            .wrapping_div((*(*(*This).device).pwfx).nSamplesPerSec as DWORD64)) as ULONG;
        (*This).nAvgBytesPerSec = freq.wrapping_mul((*(*This).pwfx).nBlockAlign as DWORD);
        DSOUND_RecalcFormat(This);
        DSOUND_MixToTemporary(This, 0 as DWORD, (*This).buflen, (FALSE) as BOOL);
    }

    RtlReleaseResource(core::ptr::addr_of_mut!((*This).lock));
    /* **** */

    return DS_OK as HRESULT;
}

unsafe extern "system" fn IDirectSoundBufferImpl_Play(
    iface: *mut IDirectSoundBuffer8,
    reserved1: DWORD,
    reserved2: DWORD,
    flags: DWORD,
) -> HRESULT {
    let mut This: *mut IDirectSoundBufferImpl = impl_from_IDirectSoundBuffer8(iface);
    let mut hres: HRESULT = (DS_OK as i32);

    RtlAcquireResourceExclusive(core::ptr::addr_of_mut!((*This).lock), (TRUE) as BYTE);

    (*This).playflags = flags;
    if (*This).state == STATE_STOPPED && (*This).hwbuf.is_null() {
        (*This).leadin = TRUE;
        (*This).state = STATE_STARTING;
    } else if (*This).state == STATE_STOPPING {
        (*This).state = STATE_PLAYING;
    }
    if !(*This).hwbuf.is_null() {
        hres = IDsDriverBuffer_Play(
            (*This).hwbuf,
            0,
            0,
            (*This).playflags,
        );
        if hres != (DS_OK as HRESULT) {
            WARN!("IDsDriverBuffer_Play failed\n");
        } else {
            (*This).state = STATE_PLAYING;
        }
    }

    RtlReleaseResource(core::ptr::addr_of_mut!((*This).lock));

    return hres;
}

unsafe extern "system" fn IDirectSoundBufferImpl_Stop(iface: *mut IDirectSoundBuffer8) -> HRESULT {
    let This: *mut IDirectSoundBufferImpl = impl_from_IDirectSoundBuffer8(iface);
    let mut hres: HRESULT = (DS_OK as i32);

    /* **** */
    RtlAcquireResourceExclusive(core::ptr::addr_of_mut!((*This).lock), TRUE as u8);

    if (*This).state == STATE_PLAYING {
        (*This).state = STATE_STOPPING;
    } else if (*This).state == STATE_STARTING {
        (*This).state = STATE_STOPPED;
        DSOUND_CheckEvent(This, 0, 0);
    }
    if !(*This).hwbuf.is_null() {
        hres = IDsDriverBuffer_Stop((*This).hwbuf);
        if hres != (DS_OK as HRESULT) {
            WARN!("IDsDriverBuffer_Stop failed\n");
        } else {
            (*This).state = STATE_STOPPED;
        }
    }

    RtlReleaseResource(core::ptr::addr_of_mut!((*This).lock));
    /* **** */

    return hres;
}

unsafe extern "system" fn IDirectSoundBufferImpl_AddRef(iface: *mut IDirectSoundBuffer8) -> ULONG {
    let mut This: *mut IDirectSoundBufferImpl = impl_from_IDirectSoundBuffer8(iface);
    let mut ref_0: ULONG =
        InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_) as *mut ::core::ffi::c_long)
            as ULONG;

    TRACE!("(%p) ref was %d\n", This, ref_0.wrapping_sub(1));

    if ref_0 == 1 {
        InterlockedIncrement(core::ptr::addr_of_mut!((*This).numIfaces) as *mut ::core::ffi::c_long);
    }

    return ref_0;
}

unsafe extern "system" fn IDirectSoundBufferImpl_Release(iface: *mut IDirectSoundBuffer8) -> ULONG {
    let mut This: *mut IDirectSoundBufferImpl = impl_from_IDirectSoundBuffer8(iface);
    let mut ref_0: ULONG = InterlockedDecrement(core::ptr::addr_of_mut!((*This).ref_)) as ULONG;

    if ref_0 == 0 && InterlockedDecrement(core::ptr::addr_of_mut!((*This).numIfaces)) == 0 {
        if is_primary_buffer(This) != 0 {
            primarybuffer_destroy(This);
        } else {
            secondarybuffer_destroy(This);
        }
    }
    return ref_0;
}

unsafe extern "system" fn IDirectSoundBufferImpl_GetCurrentPosition(
    iface: *mut IDirectSoundBuffer8,
    playpos: *mut DWORD,
    writepos: *mut DWORD,
) -> HRESULT {
    let This: *mut IDirectSoundBufferImpl = impl_from_IDirectSoundBuffer8(iface);
    let mut hres: HRESULT = 0;

    RtlAcquireResourceShared(core::ptr::addr_of_mut!((*This).lock), TRUE as BYTE);
    if !(*This).hwbuf.is_null() {
        hres = IDsDriverBuffer_GetPosition((*This).hwbuf as *mut IDsDriverBuffer, playpos, writepos);
        if hres != DS_OK as HRESULT {
            WARN!("IDsDriverBuffer_GetPosition failed\n");
            return hres;
        }
    } else {
        let mut pos: DWORD = (*This).sec_mixpos;

        /* sanity */
        if pos >= (*This).buflen {
            pos %= (*This).buflen;
        }

        if !playpos.is_null() {
            *playpos = pos;
        }
        if !writepos.is_null() {
            *writepos = pos;
        }
    }
    if !writepos.is_null()
        && (*This).state != STATE_STOPPED as DWORD
        && ((*This).hwbuf.is_null()
            || ((*(*This).device).drvdesc.dwFlags & DSDDESC_DONTNEEDWRITELEAD as DWORD) == 0)
    {
        /* apply the documented 10ms lead to writepos */
        *writepos = (*writepos).wrapping_add((*This).writelead);
        *writepos %= (*This).buflen;
    }
    RtlReleaseResource(core::ptr::addr_of_mut!((*This).lock));

    return DS_OK as HRESULT;
}

unsafe extern "system" fn IDirectSoundBufferImpl_GetStatus(iface: *mut IDirectSoundBuffer8, status: *mut DWORD) -> HRESULT {
    let This: *mut IDirectSoundBufferImpl = impl_from_IDirectSoundBuffer8(iface);

    if status.is_null() {
        WARN!("invalid parameter: status = NULL\n");
        return DSERR_INVALIDPARAM;
    }

    *status = 0;
    RtlAcquireResourceShared(core::ptr::addr_of_mut!((*This).lock), (TRUE) as BYTE);
    if ((*This).state == STATE_STARTING) || ((*This).state == STATE_PLAYING) {
        *status |= DSBSTATUS_PLAYING;
        if (*This).playflags & DSBPLAY_LOOPING != 0 {
            *status |= DSBSTATUS_LOOPING;
        }
    }
    RtlReleaseResource(core::ptr::addr_of_mut!((*This).lock));

    return (DS_OK) as HRESULT;
}

unsafe extern "system" fn IDirectSoundBufferImpl_GetFormat(iface: *mut IDirectSoundBuffer8, lpwf: LPWAVEFORMATEX, wfsize: DWORD, wfwritten: *mut DWORD) -> HRESULT {
    let This: *mut IDirectSoundBufferImpl = impl_from_IDirectSoundBuffer8(iface);
    let mut size: DWORD;

    size = (core::mem::size_of::<WAVEFORMATEX>() as usize).wrapping_add((*(*This).pwfx).cbSize as usize) as DWORD;

    if !lpwf.is_null() { /* NULL is valid */
        if wfsize >= size {
            CopyMemory!(lpwf as *mut core::ffi::c_void, (*This).pwfx as *const core::ffi::c_void, size as usize);
            if !wfwritten.is_null() {
                *wfwritten = size;
            }
        } else {
            WARN!("invalid parameter: wfsize too small\n");
            CopyMemory!(lpwf as *mut core::ffi::c_void, (*This).pwfx as *const core::ffi::c_void, wfsize as usize);
            if !wfwritten.is_null() {
                *wfwritten = wfsize;
            }
            return DSERR_INVALIDPARAM;
        }
    } else {
        if !wfwritten.is_null() {
            *wfwritten = (core::mem::size_of::<WAVEFORMATEX>() as usize).wrapping_add((*(*This).pwfx).cbSize as usize) as DWORD;
        } else {
            WARN!("invalid parameter: wfwritten == NULL\n");
            return DSERR_INVALIDPARAM;
        }
    }

    return (DS_OK) as HRESULT;
}

unsafe extern "system" fn IDirectSoundBufferImpl_Lock(iface: *mut IDirectSoundBuffer8, mut writecursor: DWORD, mut writebytes: DWORD, lplpaudioptr1: *mut *mut core::ffi::c_void, audiobytes1: *mut DWORD, lplpaudioptr2: *mut *mut core::ffi::c_void, audiobytes2: *mut DWORD, flags: DWORD) -> HRESULT {
    let mut This: *mut IDirectSoundBufferImpl = impl_from_IDirectSoundBuffer8(iface);
    let mut hres: HRESULT = (DS_OK as i32);

    TRACE!("(%p,%d,%d,%p,%p,%p,%p,0x%08x) at %d\n", This, writecursor, writebytes, lplpaudioptr1, audiobytes1, lplpaudioptr2, audiobytes2, flags, GetTickCount());

    if audiobytes1.is_null() {
        return DSERR_INVALIDPARAM;
    }

    /* when this flag is set, writecursor is meaningless and must be calculated */
    if flags & DSBLOCK_FROMWRITECURSOR != 0 {
        /* GetCurrentPosition does too much magic to duplicate here */
        hres = IDirectSoundBufferImpl_GetCurrentPosition(iface, core::ptr::null_mut::<DWORD>(), core::ptr::addr_of_mut!(writecursor));
        if hres != (DS_OK as HRESULT) {
            WARN!("IDirectSoundBufferImpl_GetCurrentPosition failed\n");
            return hres;
        }
    }

    /* when this flag is set, writebytes is meaningless and must be set */
    if flags & DSBLOCK_ENTIREBUFFER != 0 {
        writebytes = (*This).buflen;
    }

    if writecursor >= (*This).buflen {
        WARN!("Invalid parameter, writecursor: %u >= buflen: %u\n", writecursor, (*This).buflen);
        return DSERR_INVALIDPARAM;
    }

    if writebytes > (*This).buflen {
        WARN!("Invalid parameter, writebytes: %u > buflen: %u\n", writebytes, (*This).buflen);
        return DSERR_INVALIDPARAM;
    }

    /* **** */
    RtlAcquireResourceShared(core::ptr::addr_of_mut!((*This).lock), (TRUE) as BYTE);

    if ((*(*This).device).drvdesc.dwFlags & DSDDESC_DONTNEEDSECONDARYLOCK) == 0 && !(*This).hwbuf.is_null() {
        hres = IDsDriverBuffer_Lock(
            (*This).hwbuf,
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
            RtlReleaseResource(core::ptr::addr_of_mut!((*This).lock));
            return hres;
        }
    } else {
        if writecursor.wrapping_add(writebytes) <= (*This).buflen {
            *(lplpaudioptr1 as *mut LPBYTE) = (*(*This).buffer).memory.offset(writecursor as isize);
            if (*This).sec_mixpos >= writecursor
                && (*This).sec_mixpos < writecursor.wrapping_add(writebytes)
                && (*This).state == STATE_PLAYING
            {
                WARN!("Overwriting mixing position, case 1\n");
            }
            *audiobytes1 = writebytes;
            if !lplpaudioptr2.is_null() {
                *(lplpaudioptr2 as *mut LPBYTE) = core::ptr::null_mut::<BYTE>();
            }
            if !audiobytes2.is_null() {
                *audiobytes2 = 0;
            }
            TRACE!("Locked %p(%i bytes) and %p(%i bytes) writecursor=%d\n",
                *(lplpaudioptr1 as *mut LPBYTE),
                *audiobytes1,
                if !lplpaudioptr2.is_null() { *(lplpaudioptr2 as *mut LPBYTE) } else { core::ptr::null_mut::<BYTE>() },
                if !audiobytes2.is_null() { *audiobytes2 } else { 0 as DWORD },
                writecursor);
            TRACE!("->%d.0\n", writebytes);
        } else {
            let mut remainder: DWORD = writebytes.wrapping_add(writecursor).wrapping_sub((*This).buflen);
            *(lplpaudioptr1 as *mut LPBYTE) = (*(*This).buffer).memory.offset(writecursor as isize);
            *audiobytes1 = (*This).buflen.wrapping_sub(writecursor);
            if (*This).sec_mixpos >= writecursor
                && (*This).sec_mixpos < writecursor.wrapping_add(writebytes)
                && (*This).state == STATE_PLAYING
            {
                WARN!("Overwriting mixing position, case 2\n");
            }
            if !lplpaudioptr2.is_null() {
                *(lplpaudioptr2 as *mut LPBYTE) = (*(*This).buffer).memory;
            }
            if !audiobytes2.is_null() {
                *audiobytes2 = writebytes.wrapping_sub((*This).buflen.wrapping_sub(writecursor));
            }
            if !audiobytes2.is_null() && (*This).sec_mixpos < remainder && (*This).state == STATE_PLAYING {
                WARN!("Overwriting mixing position, case 3\n");
            }
            TRACE!("Locked %p(%i bytes) and %p(%i bytes) writecursor=%d\n",
                *(lplpaudioptr1 as *mut LPBYTE),
                *audiobytes1,
                if !lplpaudioptr2.is_null() { *(lplpaudioptr2 as *mut LPBYTE) } else { core::ptr::null_mut::<BYTE>() },
                if !audiobytes2.is_null() { *audiobytes2 } else { 0 as DWORD },
                writecursor);
        }
    }

    RtlReleaseResource(core::ptr::addr_of_mut!((*This).lock));
    /* **** */

    return (DS_OK) as HRESULT;
}

unsafe extern "system" fn IDirectSoundBufferImpl_SetCurrentPosition(iface: *mut IDirectSoundBuffer8, newpos: DWORD) -> HRESULT {
    let mut newpos = newpos;
    let This: *mut IDirectSoundBufferImpl = impl_from_IDirectSoundBuffer8(iface);
    let mut hres: HRESULT = (DS_OK as i32);
    let mut oldpos: DWORD;

    RtlAcquireResourceExclusive(core::ptr::addr_of_mut!((*This).lock), TRUE as BYTE);

    oldpos = (*This).sec_mixpos;

    newpos = newpos.wrapping_rem((*This).buflen);
    newpos = newpos.wrapping_sub(newpos.wrapping_rem((*(*This).pwfx).nBlockAlign as DWORD));
    (*This).sec_mixpos = newpos;

    if !(*This).hwbuf.is_null() {
        hres = IDsDriverBuffer_SetPosition((*This).hwbuf, (*This).buf_mixpos);
        if hres != (DS_OK as HRESULT) {
            WARN!("IDsDriverBuffer_SetPosition failed\n");
        }
    } else if oldpos != newpos {
        (*This).buf_mixpos = DSOUND_secpos_to_bufpos(This, newpos, 0 as DWORD, core::ptr::null_mut());
    }

    RtlReleaseResource(core::ptr::addr_of_mut!((*This).lock));

    return hres;
}

unsafe extern "system" fn IDirectSoundBufferImpl_SetPan(iface: *mut IDirectSoundBuffer8, pan: LONG) -> HRESULT {
    let mut This: *mut IDirectSoundBufferImpl = impl_from_IDirectSoundBuffer8(iface);
    let mut hres: HRESULT = (DS_OK as i32);

    if (pan > DSBPAN_RIGHT as LONG) || (pan < DSBPAN_LEFT as LONG) {
        WARN!("invalid parameter: pan = %d\n", pan);
        return DSERR_INVALIDPARAM;
    }

    /* You cannot use both pan and 3D controls */
    if !(((*This).dsbd.dwFlags & (DSBCAPS_CTRLPAN as DWORD)) != 0)
        || (((*This).dsbd.dwFlags & (DSBCAPS_CTRL3D as DWORD)) != 0)
    {
        WARN!("control unavailable\n");
        return DSERR_CONTROLUNAVAIL;
    }

    /* **** */
    RtlAcquireResourceExclusive(core::ptr::addr_of_mut!((*This).lock), TRUE as BYTE);

    if (*This).volpan.lPan != pan {
        (*This).volpan.lPan = pan;
        DSOUND_RecalcVolPan(core::ptr::addr_of_mut!((*This).volpan));

        if !(*This).hwbuf.is_null() {
            hres = IDsDriverBuffer_SetVolumePan((*This).hwbuf, core::ptr::addr_of_mut!((*This).volpan));
            if hres != (DS_OK as HRESULT) {
                WARN!("IDsDriverBuffer_SetVolumePan failed\n");
            }
        }
    }

    RtlReleaseResource(core::ptr::addr_of_mut!((*This).lock));
    /* **** */

    return hres;
}

unsafe extern "system" fn IDirectSoundBufferImpl_GetPan(iface: *mut IDirectSoundBuffer8, pan: *mut LONG) -> HRESULT {
    let mut This: *mut IDirectSoundBufferImpl = impl_from_IDirectSoundBuffer8(iface);

    if !((*This).dsbd.dwFlags & DSBCAPS_CTRLPAN as DWORD != 0) {
        WARN!("control unavailable\n");
        return DSERR_CONTROLUNAVAIL as HRESULT;
    }

    if pan.is_null() {
        WARN!("invalid parameter: pan = NULL\n");
        return DSERR_INVALIDPARAM as HRESULT;
    }

    *pan = (*This).volpan.lPan;

    return DS_OK as HRESULT;
}

unsafe extern "system" fn IDirectSoundBufferImpl_Unlock(
    mut iface: *mut IDirectSoundBuffer8,
    mut p1: *mut ::core::ffi::c_void,
    mut x1: DWORD,
    mut p2: *mut ::core::ffi::c_void,
    mut x2: DWORD,
) -> HRESULT {
    let mut This: *mut IDirectSoundBufferImpl = impl_from_IDirectSoundBuffer8(iface);
    let mut iter: *mut IDirectSoundBufferImpl = ::core::ptr::null_mut::<IDirectSoundBufferImpl>();
    let mut hres: HRESULT = DS_OK as HRESULT;

    RtlAcquireResourceShared(&raw mut (*This).lock, TRUE as BYTE);

    if ((*(*This).device).drvdesc.dwFlags & DSDDESC_DONTNEEDSECONDARYLOCK as DWORD) == 0
        && !(*This).hwbuf.is_null()
    {
        hres = IDsDriverBuffer_Unlock(
            (*This).hwbuf as *mut IDsDriverBuffer,
            p1 as LPVOID,
            x1,
            p2 as LPVOID,
            x2,
        );
        if hres != DS_OK as HRESULT {
            WARN!("IDsDriverBuffer_Unlock failed\n");
        }
    }

    RtlReleaseResource(&raw mut (*This).lock);

    if p2.is_null() {
        x2 = 0 as DWORD;
    }

    if (*This).hwbuf.is_null() && (x1 != 0 || x2 != 0) {
        RtlAcquireResourceShared(
            &raw mut (*(*This).device).buffer_list_lock,
            TRUE as BYTE,
        );

        iter = ((*(*This).buffer).buffers.next as *mut ::core::ffi::c_char).offset(
            -(core::mem::offset_of!(IDirectSoundBufferImpl, entry) as isize),
        ) as *mut IDirectSoundBufferImpl;
        while &raw mut (*iter).entry != &raw mut (*(*This).buffer).buffers {
            RtlAcquireResourceShared(&raw mut (*iter).lock, TRUE as BYTE);

            if x1 != 0 {
                if (x1 as DWORD_PTR)
                    .wrapping_add(p1 as DWORD_PTR)
                    .wrapping_sub((*(*iter).buffer).memory as DWORD_PTR)
                    > (*iter).buflen as DWORD_PTR
                {
                    hres = DSERR_INVALIDPARAM;
                } else {
                    DSOUND_MixToTemporary(
                        iter,
                        (p1 as DWORD_PTR).wrapping_sub((*(*iter).buffer).memory as DWORD_PTR)
                            as DWORD,
                        x1,
                        (FALSE) as BOOL,
                    );
                }
            }

            if x2 != 0 {
                DSOUND_MixToTemporary(iter, 0 as DWORD, x2, (FALSE) as BOOL);
            }

            RtlReleaseResource(&raw mut (*iter).lock);

            iter = ((*iter).entry.next as *mut ::core::ffi::c_char).offset(
                -(core::mem::offset_of!(IDirectSoundBufferImpl, entry) as isize),
            ) as *mut IDirectSoundBufferImpl;
        }

        RtlReleaseResource(&raw mut (*(*This).device).buffer_list_lock);
    }

    return hres;
}

unsafe extern "system" fn IDirectSoundBufferImpl_Restore(iface: *mut IDirectSoundBuffer8) -> HRESULT {
    let mut This: *mut IDirectSoundBufferImpl = impl_from_IDirectSoundBuffer8(iface);
    FIXME!("(%p):stub\n", This);
    return (DS_OK) as HRESULT;
}

unsafe extern "system" fn IDirectSoundBufferImpl_GetFrequency(iface: *mut IDirectSoundBuffer8, freq: *mut DWORD) -> HRESULT {
    let This: *mut IDirectSoundBufferImpl = impl_from_IDirectSoundBuffer8(iface);

    if freq.is_null() {
        WARN!("invalid parameter: freq = NULL\n");
        return DSERR_INVALIDPARAM;
    }

    *freq = (*This).freq;

    return (DS_OK) as HRESULT;
}

unsafe extern "system" fn IDirectSoundBufferImpl_SetFX(iface: *mut IDirectSoundBuffer8, dwEffectsCount: DWORD, pDSFXDesc: LPDSEFFECTDESC, pdwResultCodes: *mut DWORD) -> HRESULT {
    let mut This: *mut IDirectSoundBufferImpl = impl_from_IDirectSoundBuffer8(iface);
    let mut u: DWORD = 0;

    if !pdwResultCodes.is_null() {
        u = 0 as DWORD;
        while u < dwEffectsCount {
            *pdwResultCodes.offset(u as isize) = DSFXR_UNKNOWN as DWORD;
            u = u.wrapping_add(1);
        }
    }

    WARN!("control unavailable\n");
    return DSERR_CONTROLUNAVAIL;
}

unsafe extern "system" fn IDirectSoundBufferImpl_AcquireResources(
    iface: *mut IDirectSoundBuffer8,
    dwFlags: DWORD,
    dwEffectsCount: DWORD,
    pdwResultCodes: *mut DWORD,
) -> HRESULT {
    let mut This: *mut IDirectSoundBufferImpl = impl_from_IDirectSoundBuffer8(iface);
    let mut u: DWORD;

    if !pdwResultCodes.is_null() {
        u = 0 as DWORD;
        while u < dwEffectsCount {
            *pdwResultCodes.offset(u as isize) = DSFXR_UNKNOWN as DWORD;
            u = u.wrapping_add(1);
        }
    }

    WARN!("control unavailable\n");
    return DS_OK as HRESULT;
}

unsafe extern "system" fn IDirectSoundBufferImpl_GetObjectInPath(iface: *mut IDirectSoundBuffer8, rguidObject: *const GUID, dwIndex: DWORD, rguidInterface: *const GUID, ppObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let mut This: *mut IDirectSoundBufferImpl = impl_from_IDirectSoundBuffer8(iface);

    WARN!("control unavailable\n");
    return DSERR_CONTROLUNAVAIL;
}

unsafe extern "system" fn IDirectSoundBufferImpl_Initialize(iface: *mut IDirectSoundBuffer8, dsound: *mut IDirectSound, dbsd: LPCDSBUFFERDESC) -> HRESULT {
        let This: *mut IDirectSoundBufferImpl = impl_from_IDirectSoundBuffer8(iface);

	WARN!("(%p) already initialized\n", This);
	return DSERR_ALREADYINITIALIZED;
}

unsafe extern "system" fn IDirectSoundBufferImpl_GetCaps(iface: *mut IDirectSoundBuffer8, caps: LPDSBCAPS) -> HRESULT {
    let mut This: *mut IDirectSoundBufferImpl = impl_from_IDirectSoundBuffer8(iface);

    if caps.is_null() {
        WARN!("invalid parameter: caps == NULL\n");
        return DSERR_INVALIDPARAM;
    }

    if ((*caps).dwSize as usize) < core::mem::size_of::<_DSBCAPS>() {
        WARN!("invalid parameter: caps->dwSize = %d\n", (*caps).dwSize);
        return DSERR_INVALIDPARAM;
    }

    (*caps).dwFlags = (*This).dsbd.dwFlags;
    if !(*This).hwbuf.is_null() {
        (*caps).dwFlags |= DSBCAPS_LOCHARDWARE;
    } else {
        (*caps).dwFlags |= DSBCAPS_LOCSOFTWARE;
    }

    (*caps).dwBufferBytes = (*This).buflen;

    /* According to windows, this is zero*/
    (*caps).dwUnlockTransferRate = 0;
    (*caps).dwPlayCpuOverhead = 0;

    return (DS_OK) as HRESULT;
}

unsafe extern "system" fn IDirectSoundBufferImpl_QueryInterface(iface: *mut IDirectSoundBuffer8, riid: REFIID, ppobj: *mut *mut core::ffi::c_void) -> HRESULT {
    let mut This: *mut IDirectSoundBufferImpl = impl_from_IDirectSoundBuffer8(iface);

    if ppobj.is_null() {
        WARN!("invalid parameter\n");
        return E_INVALIDARG;
    }

    *ppobj = core::ptr::null_mut();

    if IsEqualGUID!(riid, core::ptr::addr_of!(IID_IUnknown)) != false
        || IsEqualGUID!(riid, core::ptr::addr_of!(IID_IDirectSoundBuffer)) != false
        || IsEqualGUID!(riid, core::ptr::addr_of!(IID_IDirectSoundBuffer8)) != false
    {
        IDirectSoundBuffer8_AddRef(iface);
        *ppobj = iface as *mut core::ffi::c_void;
        return S_OK;
    }

    if IsEqualGUID!(core::ptr::addr_of!(IID_IDirectSoundNotify), riid) != false {
        if (*This).notify.is_null() {
            IDirectSoundNotifyImpl_Create(This, core::ptr::addr_of_mut!((*This).notify));
        }
        if !(*This).notify.is_null() {
            IDirectSoundNotify_AddRef((*This).notify as LPDIRECTSOUNDNOTIFY);
            *ppobj = (*This).notify as *mut core::ffi::c_void;
            return S_OK;
        }
        WARN!("IID_IDirectSoundNotify\n");
        return E_NOINTERFACE;
    }

    if IsEqualGUID!(core::ptr::addr_of!(IID_IDirectSound3DBuffer), riid) != false {
        if (*This).ds3db.is_null() {
            IDirectSound3DBufferImpl_Create(This, core::ptr::addr_of_mut!((*This).ds3db));
        }
        if !(*This).ds3db.is_null() {
            IDirectSound3DBuffer_AddRef((*This).ds3db as LPDIRECTSOUND3DBUFFER);
            *ppobj = (*This).ds3db as *mut core::ffi::c_void;
            return S_OK;
        }
        WARN!("IID_IDirectSound3DBuffer\n");
        return E_NOINTERFACE;
    }

    if IsEqualGUID!(core::ptr::addr_of!(IID_IDirectSound3DListener), riid) != false {
        ERR!("app requested IDirectSound3DListener on secondary buffer\n");
        return E_NOINTERFACE;
    }

    if IsEqualGUID!(core::ptr::addr_of!(IID_IKsPropertySet), riid) != false {
        if (*This).iks.is_null() {
            IKsBufferPropertySetImpl_Create(This, core::ptr::addr_of_mut!((*This).iks));
        }
        if !(*This).iks.is_null() {
            IKsPropertySet_AddRef((*This).iks as LPKSPROPERTYSET);
            *ppobj = (*This).iks as *mut core::ffi::c_void;
            return S_OK;
        }
        WARN!("IID_IKsPropertySet\n");
        return E_NOINTERFACE;
    }

    return E_NOINTERFACE;
}
#[no_mangle]
pub unsafe extern "C" fn IDirectSoundBufferImpl_Create(mut device: *mut DirectSoundDevice, mut pdsb: *mut *mut IDirectSoundBufferImpl, mut dsbd: LPCDSBUFFERDESC) -> HRESULT {
    let mut dsb: *mut IDirectSoundBufferImpl = core::ptr::null_mut();
    let mut wfex: LPWAVEFORMATEX = (*dsbd).lpwfxFormat;
    let mut err: HRESULT = (DS_OK as i32);
    let mut capf: DWORD = 0;
    let mut use_hw: i32 = 0;

    if (*dsbd).dwBufferBytes < DSBSIZE_MIN || (*dsbd).dwBufferBytes > DSBSIZE_MAX {
        WARN!("invalid parameter: dsbd->dwBufferBytes = %d\n", (*dsbd).dwBufferBytes);
        *pdsb = core::ptr::null_mut();
        return DSERR_INVALIDPARAM;
    }

    dsb = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        (core::mem::size_of::<IDirectSoundBufferImpl>()) as SIZE_T,
    ) as *mut IDirectSoundBufferImpl;

    if dsb == core::ptr::null_mut() {
        WARN!("out of memory\n");
        *pdsb = core::ptr::null_mut();
        return DSERR_OUTOFMEMORY;
    }

    (*dsb).ref_ = 1;
    (*dsb).numIfaces = 1;
    (*dsb).device = device;
    (*dsb).IDirectSoundBuffer8_iface.lpVtbl = core::ptr::addr_of!(dsbvt) as *const IDirectSoundBuffer8Vtbl;
    (*dsb).iks = core::ptr::null_mut();

    /* size depends on version */
    CopyMemory!(
        core::ptr::addr_of_mut!((*dsb).dsbd) as *mut core::ffi::c_void,
        dsbd as *const core::ffi::c_void,
        (*dsbd).dwSize as usize,
    );

    (*dsb).pwfx = DSOUND_CopyFormat(wfex) as PWAVEFORMATEX;
    if (*dsb).pwfx.is_null() {
        HeapFree(GetProcessHeap(), 0, dsb as LPVOID);
        *pdsb = core::ptr::null_mut();
        return DSERR_OUTOFMEMORY;
    }

    if (*dsbd).dwBufferBytes % ((*(*dsbd).lpwfxFormat).nBlockAlign as DWORD) != 0 {
        (*dsb).buflen = (*dsbd).dwBufferBytes
            + (((*(*dsbd).lpwfxFormat).nBlockAlign as DWORD)
                - ((*dsbd).dwBufferBytes % ((*(*dsbd).lpwfxFormat).nBlockAlign as DWORD)));
    } else {
        (*dsb).buflen = (*dsbd).dwBufferBytes;
    }

    (*dsb).freq = (*(*dsbd).lpwfxFormat).nSamplesPerSec;
    (*dsb).notify = core::ptr::null_mut();
    (*dsb).notifies = core::ptr::null_mut();
    (*dsb).nrofnotifies = 0;
    (*dsb).hwnotify = core::ptr::null_mut();

    /* Check necessary hardware mixing capabilities */
    if (*wfex).nChannels == 2 {
        capf |= DSCAPS_SECONDARYSTEREO;
    } else {
        capf |= DSCAPS_SECONDARYMONO;
    }
    if (*wfex).wBitsPerSample == 16 {
        capf |= DSCAPS_SECONDARY16BIT;
    } else {
        capf |= DSCAPS_SECONDARY8BIT;
    }

    use_hw = (((*dsbd).dwFlags & DSBCAPS_LOCHARDWARE) != 0) as i32;

    if use_hw != 0 && (((*device).drvcaps.dwFlags & capf) != capf || (*device).driver.is_null()) {
        if !(*device).driver.is_null() {
            WARN!("Format not supported for hardware buffer\n");
        }
        HeapFree(GetProcessHeap(), 0, (*dsb).pwfx as LPVOID);
        HeapFree(GetProcessHeap(), 0, dsb as LPVOID);
        *pdsb = core::ptr::null_mut();
        if ((*device).drvcaps.dwFlags & capf) != capf {
            return DSERR_BADFORMAT;
        }
        return DSERR_GENERIC;
    }

    /* FIXME: check hardware sample rate mixing capabilities */
    /* FIXME: check app hints for software/hardware buffer (STATIC, LOCHARDWARE, etc) */
    /* FIXME: check whether any hardware buffers are left */
    /* FIXME: handle DSDHEAP_CREATEHEAP for hardware buffers */

    /* Allocate an empty buffer */
    (*dsb).buffer = HeapAlloc(
        GetProcessHeap(),
        0,
        (core::mem::size_of::<BufferMemory>()) as SIZE_T,
    ) as *mut BufferMemory;
    if (*dsb).buffer.is_null() {
        WARN!("out of memory\n");
        HeapFree(GetProcessHeap(), 0, (*dsb).pwfx as LPVOID);
        HeapFree(GetProcessHeap(), 0, dsb as LPVOID);
        *pdsb = core::ptr::null_mut();
        return DSERR_OUTOFMEMORY;
    }

    /* Allocate system memory for buffer if applicable */
    if ((*device).drvdesc.dwFlags & DSDDESC_USESYSTEMMEMORY) != 0 || use_hw == 0 {
        (*(*dsb).buffer).memory = HeapAlloc(
            GetProcessHeap(),
            0,
            (*dsb).buflen as SIZE_T,
        ) as LPBYTE;
        if (*(*dsb).buffer).memory.is_null() {
            WARN!("out of memory\n");
            HeapFree(GetProcessHeap(), 0, (*dsb).pwfx as LPVOID);
            HeapFree(GetProcessHeap(), 0, (*dsb).buffer as LPVOID);
            HeapFree(GetProcessHeap(), 0, dsb as LPVOID);
            *pdsb = core::ptr::null_mut();
            return DSERR_OUTOFMEMORY;
        }
    }

    /* Allocate the hardware buffer */
    if use_hw != 0 {
        err = IDsDriver_CreateSoundBuffer(
            (*device).driver as *mut IDsDriver,
            wfex,
            (*dsbd).dwFlags,
            0,
            core::ptr::addr_of_mut!((*dsb).buflen),
            core::ptr::addr_of_mut!((*(*dsb).buffer).memory),
            core::ptr::addr_of_mut!((*dsb).hwbuf) as *mut LPVOID,
        );
        if err < 0 {
            WARN!("Failed to create hardware secondary buffer: %08x\n", err);
            if ((*device).drvdesc.dwFlags & DSDDESC_USESYSTEMMEMORY) != 0 {
                HeapFree(
                    GetProcessHeap(),
                    0,
                    (*(*dsb).buffer).memory as LPVOID,
                );
            }
            HeapFree(GetProcessHeap(), 0, (*dsb).buffer as LPVOID);
            HeapFree(GetProcessHeap(), 0, (*dsb).pwfx as LPVOID);
            HeapFree(GetProcessHeap(), 0, dsb as LPVOID);
            *pdsb = core::ptr::null_mut();
            return DSERR_GENERIC;
        }
    }

    (*(*dsb).buffer).ref_ = 1;
    list_init(core::ptr::addr_of_mut!((*(*dsb).buffer).buffers));
    list_add_head(
        core::ptr::addr_of_mut!((*(*dsb).buffer).buffers),
        core::ptr::addr_of_mut!((*dsb).entry),
    );
    FillMemory!(
        (*(*dsb).buffer).memory,
        (*dsb).buflen,
        if (*(*dsbd).lpwfxFormat).wBitsPerSample == 8 {
            128
        } else {
            0
        },
    );

    /* It's not necessary to initialize values to zero since */
    /* we allocated this structure with HEAP_ZERO_MEMORY... */
    (*dsb).sec_mixpos = 0;
    (*dsb).buf_mixpos = (*dsb).sec_mixpos;
    (*dsb).state = STATE_STOPPED;

    (*dsb).freqAdjust = ((((*dsb).freq as DWORD64) << DSOUND_FREQSHIFT)
        / ((*(*device).pwfx).nSamplesPerSec as DWORD64)) as ULONG;
    (*dsb).nAvgBytesPerSec = (*dsb).freq * ((*(*dsbd).lpwfxFormat).nBlockAlign as DWORD);

    /* calculate fragment size and write lead */
    DSOUND_RecalcFormat(dsb);

    if ((*dsb).dsbd.dwFlags & DSBCAPS_CTRL3D) != 0 {
        (*dsb).ds3db_ds3db.dwSize = (core::mem::size_of::<DS3DBUFFER>()) as _;
        (*dsb).ds3db_ds3db.vPosition.x = 0.0f32;
        (*dsb).ds3db_ds3db.vPosition.y = 0.0f32;
        (*dsb).ds3db_ds3db.vPosition.z = 0.0f32;
        (*dsb).ds3db_ds3db.vVelocity.x = 0.0f32;
        (*dsb).ds3db_ds3db.vVelocity.y = 0.0f32;
        (*dsb).ds3db_ds3db.vVelocity.z = 0.0f32;
        (*dsb).ds3db_ds3db.dwInsideConeAngle = DS3D_DEFAULTCONEANGLE;
        (*dsb).ds3db_ds3db.dwOutsideConeAngle = DS3D_DEFAULTCONEANGLE;
        (*dsb).ds3db_ds3db.vConeOrientation.x = 0.0f32;
        (*dsb).ds3db_ds3db.vConeOrientation.y = 0.0f32;
        (*dsb).ds3db_ds3db.vConeOrientation.z = 0.0f32;
        (*dsb).ds3db_ds3db.lConeOutsideVolume = (DS3D_DEFAULTCONEOUTSIDEVOLUME as i32);
        (*dsb).ds3db_ds3db.flMinDistance = DS3D_DEFAULTMINDISTANCE as D3DVALUE;
        (*dsb).ds3db_ds3db.flMaxDistance = DS3D_DEFAULTMAXDISTANCE as D3DVALUE;
        (*dsb).ds3db_ds3db.dwMode = DS3DMODE_NORMAL;

        (*dsb).ds3db_need_recalc = (FALSE as i32);
        DSOUND_Calc3DBuffer(dsb);
    } else {
        DSOUND_RecalcVolPan(core::ptr::addr_of_mut!((*dsb).volpan));
    }

    RtlInitializeResource(core::ptr::addr_of_mut!((*dsb).lock));

    /* register buffer if not primary */
    if ((*dsbd).dwFlags & DSBCAPS_PRIMARYBUFFER) == 0 {
        err = DirectSoundDevice_AddBuffer(device, dsb);
        if err != (DS_OK as HRESULT) {
            HeapFree(
                GetProcessHeap(),
                0,
                (*(*dsb).buffer).memory as LPVOID,
            );
            HeapFree(GetProcessHeap(), 0, (*dsb).buffer as LPVOID);
            RtlDeleteResource(core::ptr::addr_of_mut!((*dsb).lock));
            HeapFree(GetProcessHeap(), 0, (*dsb).pwfx as LPVOID);
            HeapFree(GetProcessHeap(), 0, dsb as LPVOID);
            dsb = core::ptr::null_mut();
        }
    }

    *pdsb = dsb;
    return err;
}
#[no_mangle]
pub unsafe extern "C" fn secondarybuffer_destroy(This: *mut IDirectSoundBufferImpl) {
    DirectSoundDevice_RemoveBuffer((*This).device, This);
    RtlDeleteResource(core::ptr::addr_of_mut!((*This).lock));

    if !(*This).hwbuf.is_null() {
        IDsDriverBuffer_Release((*This).hwbuf as *mut IDsDriverBuffer);
    }
    if (*This).hwbuf.is_null() || ((*(*This).device).drvdesc.dwFlags & DSDDESC_USESYSTEMMEMORY) != 0 {
        (*(*This).buffer).ref_ -= 1;
        list_remove(core::ptr::addr_of_mut!((*This).entry));
        if (*(*This).buffer).ref_ == 0 {
            HeapFree(
                GetProcessHeap(),
                0,
                (*(*This).buffer).memory as PVOID,
            );
            HeapFree(
                GetProcessHeap(),
                0,
                (*This).buffer as PVOID,
            );
        }
    }

    HeapFree(
        GetProcessHeap(),
        0,
        (*This).tmp_buffer as PVOID,
    );
    HeapFree(
        GetProcessHeap(),
        0,
        (*This).notifies as PVOID,
    );
    HeapFree(
        GetProcessHeap(),
        0,
        (*This).pwfx as PVOID,
    );
    HeapFree(
        GetProcessHeap(),
        0,
        This as PVOID,
    );

    WINE_TRACE!("(%p) released\n", This);
}
#[no_mangle]
pub unsafe extern "C" fn IDirectSoundBufferImpl_Destroy(pdsb: *mut IDirectSoundBufferImpl) -> HRESULT {
    /* This keeps the *_Destroy functions from possibly deleting
     * this object until it is ready to be deleted */
    InterlockedIncrement(core::ptr::addr_of_mut!((*pdsb).numIfaces));

    if !(*pdsb).iks.is_null() {
        WARN!("iks not NULL\n");
        IKsBufferPropertySetImpl_Destroy((*pdsb).iks);
        (*pdsb).iks = core::ptr::null_mut();
    }

    if !(*pdsb).ds3db.is_null() {
        WARN!("ds3db not NULL\n");
        IDirectSound3DBufferImpl_Destroy((*pdsb).ds3db);
        (*pdsb).ds3db = core::ptr::null_mut();
    }

    if !(*pdsb).notify.is_null() {
        WARN!("notify not NULL\n");
        IDirectSoundNotifyImpl_Destroy((*pdsb).notify);
        (*pdsb).notify = core::ptr::null_mut();
    }

    secondarybuffer_destroy(pdsb);

    return S_OK;
}
#[no_mangle]
pub unsafe extern "C" fn IDirectSoundBufferImpl_Duplicate(device: *mut DirectSoundDevice, ppdsb: *mut *mut IDirectSoundBufferImpl, pdsb: *mut IDirectSoundBufferImpl) -> HRESULT {
    let mut dsb: *mut IDirectSoundBufferImpl = core::ptr::null_mut();
    let mut hres: HRESULT = (DS_OK as i32);

    dsb = HeapAlloc(GetProcessHeap(), 0, core::mem::size_of::<IDirectSoundBufferImpl>() as SIZE_T) as *mut IDirectSoundBufferImpl;
    if dsb.is_null() {
        WARN!("out of memory\n");
        *ppdsb = core::ptr::null_mut();
        return DSERR_OUTOFMEMORY;
    }
    CopyMemory!(dsb as LPVOID, pdsb as LPVOID, core::mem::size_of::<IDirectSoundBufferImpl>() as SIZE_T);

    (*dsb).pwfx = DSOUND_CopyFormat((*pdsb).pwfx) as PWAVEFORMATEX;
    if (*dsb).pwfx.is_null() {
        HeapFree(GetProcessHeap(), 0, dsb as PVOID);
        *ppdsb = core::ptr::null_mut();
        return DSERR_OUTOFMEMORY;
    }

    if !(*pdsb).hwbuf.is_null() {
        hres = IDsDriver_DuplicateSoundBuffer((*device).driver as *mut IDsDriver, (*pdsb).hwbuf, &raw mut (*dsb).hwbuf as *mut LPVOID);
        if hres < 0 {
            WARN!("IDsDriver_DuplicateSoundBuffer failed (%08x)\n", hres);
            HeapFree(GetProcessHeap(), 0, (*dsb).pwfx as PVOID);
            HeapFree(GetProcessHeap(), 0, dsb as PVOID);
            *ppdsb = core::ptr::null_mut();
            return hres;
        }
    }

    (*(*dsb).buffer).ref_ += 1;
    list_add_head(&raw mut (*(*dsb).buffer).buffers, &raw mut (*dsb).entry);
    (*dsb).ref_ = 1;
    (*dsb).numIfaces = 1;
    (*dsb).state = STATE_STOPPED;
    (*dsb).sec_mixpos = 0;
    (*dsb).buf_mixpos = (*dsb).sec_mixpos;
    (*dsb).notify = core::ptr::null_mut();
    (*dsb).notifies = core::ptr::null_mut();
    (*dsb).nrofnotifies = 0;
    (*dsb).device = device;
    (*dsb).ds3db = core::ptr::null_mut();
    (*dsb).iks = core::ptr::null_mut(); /* FIXME? */
    (*dsb).tmp_buffer = core::ptr::null_mut();
    DSOUND_RecalcFormat(dsb);
    DSOUND_MixToTemporary(dsb, 0, (*dsb).buflen, (FALSE) as BOOL);

    RtlInitializeResource(&raw mut (*dsb).lock);

    /* register buffer */
    hres = DirectSoundDevice_AddBuffer(device, dsb);
    if hres != (DS_OK as HRESULT) {
        RtlDeleteResource(&raw mut (*dsb).lock);
        HeapFree(GetProcessHeap(), 0, (*dsb).tmp_buffer as PVOID);
        list_remove(&raw mut (*dsb).entry);
        (*(*dsb).buffer).ref_ -= 1;
        HeapFree(GetProcessHeap(), 0, (*dsb).pwfx as PVOID);
        HeapFree(GetProcessHeap(), 0, dsb as PVOID);
        dsb = core::ptr::null_mut();
    }

    *ppdsb = dsb;
    return hres;
}

unsafe extern "system" fn IKsBufferPropertySetImpl_QueryInterface(
    iface: LPKSPROPERTYSET,
    riid: REFIID,
    ppobj: *mut LPVOID,
) -> HRESULT {
    let This: *mut IKsBufferPropertySetImpl = iface as *mut IKsBufferPropertySetImpl;
    WINE_TRACE!("(%p,%s,%p)\n", This, debugstr_guid(riid as *const _GUID), ppobj);
    return IDirectSoundBuffer_QueryInterface(((*This).dsb as LPDIRECTSOUNDBUFFER8).cast(), riid, ppobj);
}
unsafe extern "system" fn IKsBufferPropertySetImpl_AddRef(mut iface: LPKSPROPERTYSET) -> ULONG {
    let mut This: *mut IKsBufferPropertySetImpl = iface as *mut IKsBufferPropertySetImpl;
    let mut ref_: ULONG =
        InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_) as *mut ::core::ffi::c_long)
            as ULONG;
    WINE_TRACE!("(%p) ref was %d\n", This, ref_.wrapping_sub(1 as ULONG));
    return ref_;
}

unsafe extern "system" fn IKsBufferPropertySetImpl_Release(iface: LPKSPROPERTYSET) -> ULONG {
    let This: *mut IKsBufferPropertySetImpl = iface as *mut IKsBufferPropertySetImpl;
    let mut ref_0: ULONG = InterlockedDecrement(&raw mut (*This).ref_ as *mut LONG) as ULONG;

    if ref_0 == 0 {
        (*(*This).dsb).iks = core::ptr::null_mut();
        IDirectSoundBuffer_Release(((*This).dsb as LPDIRECTSOUND3DBUFFER).cast());
        HeapFree(GetProcessHeap(), 0, This as PVOID);
    }
    return ref_0;
}

unsafe extern "system" fn IKsBufferPropertySetImpl_Get(
    iface: LPKSPROPERTYSET,
    guidPropSet: REFGUID,
    dwPropID: ULONG,
    pInstanceData: LPVOID,
    cbInstanceData: ULONG,
    pPropData: LPVOID,
    cbPropData: ULONG,
    pcbReturned: PULONG,
) -> HRESULT {
    let This: *mut IKsBufferPropertySetImpl = iface as *mut IKsBufferPropertySetImpl;
    let mut ps: PIDSDRIVERPROPERTYSET = core::ptr::null_mut();

    if !(*(*This).dsb).hwbuf.is_null() {
        IDsDriver_QueryInterface(
            ((*(*This).dsb).hwbuf).cast(),
            &raw const IID_IDsDriverPropertySet,
            &raw mut ps as *mut *mut core::ffi::c_void,
        );

        if !ps.is_null() {
            let mut prop: DSPROPERTY = core::mem::zeroed();
            let mut hres: HRESULT;

            prop.s.Set = *guidPropSet;
            prop.s.Id = dwPropID;
            prop.s.Flags = 0; /* unused */
            prop.s.InstanceId = (*(*This).dsb).device as ULONG_PTR as ULONG;

            hres = IDsDriverPropertySet_Get(
                ps,
                &raw mut prop,
                pInstanceData,
                cbInstanceData,
                pPropData,
                cbPropData,
                pcbReturned,
            );

            IDsDriverPropertySet_Release(ps);

            return hres;
        }
    }

    return E_PROP_ID_UNSUPPORTED;
}

unsafe extern "system" fn IKsBufferPropertySetImpl_Set(
    iface: LPKSPROPERTYSET,
    guidPropSet: *const GUID,
    dwPropID: ULONG,
    pInstanceData: LPVOID,
    cbInstanceData: ULONG,
    pPropData: LPVOID,
    cbPropData: ULONG,
) -> HRESULT {
    let This: *mut IKsBufferPropertySetImpl = iface as *mut IKsBufferPropertySetImpl;
    let mut ps: PIDSDRIVERPROPERTYSET = core::ptr::null_mut();

    if !(*(*This).dsb).hwbuf.is_null() {
        IDsDriver_QueryInterface(
            ((*(*This).dsb).hwbuf as *mut IDsDriverBuffer).cast(),
            core::ptr::addr_of!(IID_IDsDriverPropertySet),
            core::ptr::addr_of_mut!(ps) as *mut *mut core::ffi::c_void,
        );

        if !ps.is_null() {
            let mut prop: DSPROPERTY = core::mem::zeroed();
            let mut hres: HRESULT = 0;

            prop.s.Set = *guidPropSet;
            prop.s.Id = dwPropID;
            prop.s.Flags = 0; /* unused */
            prop.s.InstanceId = (*(*This).dsb).device as ULONG_PTR as ULONG;
            hres = IDsDriverPropertySet_Set(
                ps,
                core::ptr::addr_of_mut!(prop),
                pInstanceData,
                cbInstanceData,
                pPropData,
                cbPropData,
            );

            IDsDriverPropertySet_Release(ps);

            return hres;
        }
    }

    return E_PROP_ID_UNSUPPORTED;
}

unsafe extern "system" fn IKsBufferPropertySetImpl_QuerySupport(
    iface: LPKSPROPERTYSET,
    guidPropSet: REFGUID,
    dwPropID: ULONG,
    pTypeSupport: PULONG,
) -> HRESULT {
    let This: *mut IKsBufferPropertySetImpl = iface as *mut IKsBufferPropertySetImpl;
    let mut ps: PIDSDRIVERPROPERTYSET = core::ptr::null_mut();

    if !(*(*This).dsb).hwbuf.is_null() {
        IDsDriver_QueryInterface(
            ((*(*This).dsb).hwbuf).cast(),
            core::ptr::addr_of!(IID_IDsDriverPropertySet),
            core::ptr::addr_of_mut!(ps) as *mut *mut core::ffi::c_void,
        );

        if !ps.is_null() {
            let mut hres: HRESULT;

            hres = IDsDriverPropertySet_QuerySupport(ps, guidPropSet, dwPropID, pTypeSupport);

            IDsDriverPropertySet_Release(ps);

            return hres;
        }
    }

    return E_PROP_ID_UNSUPPORTED;
}
#[no_mangle]
pub unsafe extern "C" fn IKsBufferPropertySetImpl_Create(
    dsb: *mut IDirectSoundBufferImpl,
    piks: *mut *mut IKsBufferPropertySetImpl,
) -> HRESULT {
    let mut iks: *mut IKsBufferPropertySetImpl;

    *piks = core::ptr::null_mut();

    iks = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        core::mem::size_of::<IKsBufferPropertySetImpl>() as SIZE_T,
    ) as *mut IKsBufferPropertySetImpl;
    if iks == core::ptr::null_mut() {
        WARN!("out of memory\n");
        *piks = core::ptr::null_mut();
        return DSERR_OUTOFMEMORY;
    }

    (*iks).ref_ = 0;
    (*iks).dsb = dsb;
    (*dsb).iks = iks;
    (*iks).lpVtbl = core::ptr::addr_of!(iksbvt);

    IDirectSoundBuffer_AddRef(dsb as LPDIRECTSOUNDBUFFER);

    *piks = iks;
    return S_OK;
}
#[no_mangle]
pub unsafe extern "C" fn IKsBufferPropertySetImpl_Destroy(
    piks: *mut IKsBufferPropertySetImpl,
) -> HRESULT {
    while IKsBufferPropertySetImpl_Release(piks as LPKSPROPERTYSET) > 0 {}

    return S_OK;
}

// COM vtable tables — generated from buffer.c by vtbl_gen.py
/// buffer.c:123 `static const IDirectSoundNotifyVtbl dsnvt`  (slot order: rust:types_amd64.rs)
pub static dsnvt: IDirectSoundNotifyVtbl = IDirectSoundNotifyVtbl {
    QueryInterface: Some(IDirectSoundNotifyImpl_QueryInterface),
    AddRef: Some(IDirectSoundNotifyImpl_AddRef),
    Release: Some(IDirectSoundNotifyImpl_Release),
    SetNotificationPositions: Some(IDirectSoundNotifyImpl_SetNotificationPositions),
};

/// buffer.c:903 `static const IDirectSoundBuffer8Vtbl dsbvt`  (slot order: rust:types_amd64.rs)
pub static dsbvt: IDirectSoundBuffer8Vtbl = IDirectSoundBuffer8Vtbl {
    QueryInterface: Some(IDirectSoundBufferImpl_QueryInterface),
    AddRef: Some(IDirectSoundBufferImpl_AddRef),
    Release: Some(IDirectSoundBufferImpl_Release),
    GetCaps: Some(IDirectSoundBufferImpl_GetCaps),
    GetCurrentPosition: Some(IDirectSoundBufferImpl_GetCurrentPosition),
    GetFormat: Some(IDirectSoundBufferImpl_GetFormat),
    GetVolume: Some(IDirectSoundBufferImpl_GetVolume),
    GetPan: Some(IDirectSoundBufferImpl_GetPan),
    GetFrequency: Some(IDirectSoundBufferImpl_GetFrequency),
    GetStatus: Some(IDirectSoundBufferImpl_GetStatus),
    Initialize: Some(IDirectSoundBufferImpl_Initialize),
    Lock: Some(IDirectSoundBufferImpl_Lock),
    Play: Some(IDirectSoundBufferImpl_Play),
    SetCurrentPosition: Some(IDirectSoundBufferImpl_SetCurrentPosition),
    SetFormat: Some(IDirectSoundBufferImpl_SetFormat),
    SetVolume: Some(IDirectSoundBufferImpl_SetVolume),
    SetPan: Some(IDirectSoundBufferImpl_SetPan),
    SetFrequency: Some(IDirectSoundBufferImpl_SetFrequency),
    Stop: Some(IDirectSoundBufferImpl_Stop),
    Unlock: Some(IDirectSoundBufferImpl_Unlock),
    Restore: Some(IDirectSoundBufferImpl_Restore),
    SetFX: Some(IDirectSoundBufferImpl_SetFX),
    AcquireResources: Some(IDirectSoundBufferImpl_AcquireResources),
    GetObjectInPath: Some(IDirectSoundBufferImpl_GetObjectInPath),
};

/// buffer.c:1381 `static const IKsPropertySetVtbl iksbvt`  (slot order: rust:types_amd64.rs)
pub static iksbvt: IKsPropertySetVtbl = IKsPropertySetVtbl {
    QueryInterface: Some(IKsBufferPropertySetImpl_QueryInterface),
    AddRef: Some(IKsBufferPropertySetImpl_AddRef),
    Release: Some(IKsBufferPropertySetImpl_Release),
    Get: Some(IKsBufferPropertySetImpl_Get),
    Set: Some(IKsBufferPropertySetImpl_Set),
    QuerySupport: Some(IKsBufferPropertySetImpl_QuerySupport),
};
