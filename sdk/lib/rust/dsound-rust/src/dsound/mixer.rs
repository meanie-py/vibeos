use crate::dsound::dsound_main::DSOUND_renderer;
use crate::dsound::dsound_convert::convertbpp;
use crate::dsound::dsound_main::ds_snd_shadow_maxsize;
use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "C" fn DSOUND_RecalcVolPan(volpan: PDSVOLUMEPAN) {
    let mut temp: f64 = 0.0;
    /* the AmpFactors are expressed in 16.16 fixed point */
    (*volpan).dwVolAmpFactor = (pow(2.0f64, (*volpan).lVolume as f64 / 600.0f64) * (0xffff as f64)) as ULONG;
    /* FIXME: dwPan{Left|Right}AmpFactor */

    /* FIXME: use calculated vol and pan ampfactors */
    temp = ((*volpan).lVolume - ((if (*volpan).lPan > (0 as LONG) { (*volpan).lPan } else { (0 as LONG) }))) as f64;
    (*volpan).dwTotalLeftAmpFactor = (pow(2.0f64, temp / 600.0f64) * (0xffff as f64)) as ULONG;
    temp = ((*volpan).lVolume + ((if (*volpan).lPan < (0 as LONG) { (*volpan).lPan } else { (0 as LONG) }))) as f64;
    (*volpan).dwTotalRightAmpFactor = (pow(2.0f64, temp / 600.0f64) * (0xffff as f64)) as ULONG;
}
#[no_mangle]
pub unsafe extern "C" fn DSOUND_AmpFactorToVolPan(mut volpan: PDSVOLUMEPAN) {
    let mut left: ::core::ffi::c_double = 0.;
    let mut right: ::core::ffi::c_double = 0.;
    WINE_TRACE!("(%p)\n", volpan);
    WINE_TRACE!("left=%x, right=%x\n", (*volpan).dwTotalLeftAmpFactor, (*volpan).dwTotalRightAmpFactor);
    if (*volpan).dwTotalLeftAmpFactor == 0 {
        left = -(10000) as ::core::ffi::c_double;
    } else {
        left = 600 as ::core::ffi::c_double
            * log((*volpan).dwTotalLeftAmpFactor as ::core::ffi::c_double
                / 0xffff as ::core::ffi::c_double)
            / log(2 as ::core::ffi::c_double);
    }
    if (*volpan).dwTotalRightAmpFactor == 0 {
        right = -(10000) as ::core::ffi::c_double;
    } else {
        right = 600 as ::core::ffi::c_double
            * log((*volpan).dwTotalRightAmpFactor as ::core::ffi::c_double
                / 0xffff as ::core::ffi::c_double)
            / log(2 as ::core::ffi::c_double);
    }
    if left < right {
        (*volpan).lVolume = right as LONG;
        (*volpan).dwVolAmpFactor = (*volpan).dwTotalRightAmpFactor;
    } else {
        (*volpan).lVolume = left as LONG;
        (*volpan).dwVolAmpFactor = (*volpan).dwTotalLeftAmpFactor;
    }
    if (*volpan).lVolume < -(10000) as LONG {
        (*volpan).lVolume = -(10000) as LONG;
    }
    (*volpan).lPan = (right - left) as LONG;
    if (*volpan).lPan < -(10000) as LONG {
        (*volpan).lPan = -(10000) as LONG;
    }
    WINE_TRACE!("Vol=%d Pan=%d\n", (*volpan).lVolume, (*volpan).lPan);
}
#[no_mangle]
pub unsafe extern "C" fn DSOUND_bufpos_to_mixpos(device: *const DirectSoundDevice, pos: DWORD) -> DWORD {
    let mut ret: DWORD = pos
        .wrapping_mul(32 as DWORD)
        .wrapping_div((*(*device).pwfx).wBitsPerSample as DWORD);
    if (*(*device).pwfx).wBitsPerSample as core::ffi::c_int == 32 {
        ret = ret.wrapping_mul(2 as DWORD);
    }
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn DSOUND_secpos_to_bufpos(
    dsb: *const IDirectSoundBufferImpl,
    secpos: DWORD,
    secmixpos: DWORD,
    overshot: *mut DWORD,
) -> DWORD {
    let mut framelen: DWORD64 = (secpos / ((*(*dsb).pwfx).nBlockAlign as DWORD)) as DWORD64;
    let mut freqAdjust: DWORD64 = (*dsb).freqAdjust as DWORD64;
    let mut acc: DWORD64;
    let mut freqAcc: DWORD64;

    if secpos < secmixpos {
        freqAcc = (*dsb).freqAccNext as DWORD64;
    } else {
        freqAcc = (*dsb).freqAcc as DWORD64;
    }
    acc = (framelen << DSOUND_FREQSHIFT).wrapping_add(
        freqAdjust
            .wrapping_sub(1 as DWORD64)
            .wrapping_sub(freqAcc),
    );
    acc = acc / freqAdjust;
    if !overshot.is_null() {
        let mut oshot: DWORD64 = acc.wrapping_mul(freqAdjust).wrapping_add(freqAcc);
        assert!(
            (oshot >= (framelen << DSOUND_FREQSHIFT))
        );
        oshot = oshot.wrapping_sub(framelen << DSOUND_FREQSHIFT);
        *overshot = oshot as DWORD;
        assert!(
            (*overshot < (*dsb).freqAdjust)
        );
    }
    return (acc as DWORD).wrapping_mul((*(*(*dsb).device).pwfx).nBlockAlign as DWORD);
}

unsafe extern "C" fn DSOUND_bufpos_to_secpos(dsb: *const IDirectSoundBufferImpl, bufpos: DWORD) -> DWORD {
    let mut oAdv: DWORD = (*(*(*dsb).device).pwfx).nBlockAlign as DWORD;
    let mut iAdv: DWORD = (*(*dsb).pwfx).nBlockAlign as DWORD;
    let mut pos: DWORD = 0;
    let mut framelen: DWORD64 = 0;
    let mut acc: DWORD64 = 0;

    framelen = (bufpos / oAdv) as DWORD64;
    acc = framelen
        .wrapping_mul((*dsb).freqAdjust as DWORD64)
        .wrapping_add((*dsb).freqAccNext as DWORD64);
    acc = acc >> DSOUND_FREQSHIFT;
    pos = (acc as DWORD).wrapping_mul(iAdv);
    if pos >= (*dsb).buflen {
        /* Because of differences between freqAcc and freqAccNext, this might happen */
        pos = (*dsb).buflen.wrapping_sub(iAdv);
    }

    return pos;
}

unsafe extern "C" fn DSOUND_RecalcFreqAcc(dsb: *mut IDirectSoundBufferImpl) {
    if (*dsb).freqneeded == 0 {
        return;
    }
    (*dsb).freqAcc = (*dsb).freqAccNext;
    (*dsb).tmp_buffer_len = DSOUND_secpos_to_bufpos(
        dsb,
        (*dsb).buflen,
        0 as DWORD,
        core::ptr::addr_of_mut!((*dsb).freqAccNext),
    );
    WINE_TRACE!("New freqadjust: %04x, new buflen: %d\n", (*dsb).freqAccNext, (*dsb).tmp_buffer_len);
}
#[no_mangle]
pub unsafe extern "C" fn DSOUND_RecalcFormat(dsb: *mut IDirectSoundBufferImpl) {
    let mut needremix: BOOL = (TRUE as i32);
    let mut needresample: BOOL = ((*dsb).freq != (*(*(*dsb).device).pwfx).nSamplesPerSec) as BOOL;
    let mut bAlign: DWORD = (*(*dsb).pwfx).nBlockAlign as DWORD;
    let mut pAlign: DWORD = (*(*(*dsb).device).pwfx).nBlockAlign as DWORD;
    let mut pwfxe: *mut WAVEFORMATEXTENSIBLE;
    let mut ieee: BOOL = (FALSE as i32);

    pwfxe = (*dsb).pwfx as *mut WAVEFORMATEXTENSIBLE;

    if ((*pwfxe).Format.wFormatTag as DWORD == WAVE_FORMAT_IEEE_FLOAT)
        || (((*pwfxe).Format.wFormatTag as DWORD == WAVE_FORMAT_EXTENSIBLE)
            && (IsEqualGUID!(core::ptr::addr_of!((*pwfxe).SubFormat), &KSDATAFORMAT_SUBTYPE_IEEE_FLOAT) != false))
    {
        ieee = (TRUE as i32);
    }

    /* calculate the 10ms write lead */
    (*dsb).writelead = (*dsb)
        .freq
        .wrapping_div(100)
        .wrapping_mul((*(*dsb).pwfx).nBlockAlign as DWORD);

    if ((*(*dsb).pwfx).wBitsPerSample == (*(*(*dsb).device).pwfx).wBitsPerSample)
        && ((*(*dsb).pwfx).nChannels == (*(*(*dsb).device).pwfx).nChannels)
        && needresample == 0
        && ieee == 0
    {
        needremix = (FALSE as i32);
    }
    HeapFree(GetProcessHeap(), 0, (*dsb).tmp_buffer as PVOID);
    (*dsb).tmp_buffer = core::ptr::null_mut();
    (*dsb).freqAccNext = 0;
    (*dsb).freqAcc = (*dsb).freqAccNext;
    (*dsb).max_buffer_len = (*dsb).freqAcc as DWORD;
    (*dsb).freqneeded = needresample as ULONG;

    if ieee != 0 {
        (*dsb).convert = convertbpp[4 as usize]
            [(((*(*(*dsb).device).pwfx).wBitsPerSample as i32) / 8 - 1) as usize];
    } else {
        (*dsb).convert = convertbpp
            [(((*(*dsb).pwfx).wBitsPerSample as i32) / 8 - 1) as usize]
            [(((*(*(*dsb).device).pwfx).wBitsPerSample as i32) / 8 - 1) as usize];
    }

    (*dsb).resampleinmixer = FALSE;

    if needremix != 0 {
        if needresample != 0 {
            DSOUND_RecalcFreqAcc(dsb);
        } else {
            (*dsb).tmp_buffer_len = (*dsb).buflen.wrapping_div(bAlign).wrapping_mul(pAlign);
        }
        (*dsb).max_buffer_len = (*dsb).tmp_buffer_len;
        if ((*dsb).max_buffer_len <= (*(*dsb).device).buflen
            || (*dsb).max_buffer_len
                < (ds_snd_shadow_maxsize * 1024 * 1024) as DWORD)
            && ds_snd_shadow_maxsize >= 0
        {
            (*dsb).tmp_buffer = HeapAlloc(
                GetProcessHeap(),
                0,
                (*dsb).max_buffer_len as SIZE_T,
            ) as LPBYTE;
        }
        if !(*dsb).tmp_buffer.is_null() {
            FillMemory!(
                (*dsb).tmp_buffer as LPVOID,
                (*dsb).tmp_buffer_len,
                if (*(*(*dsb).device).pwfx).wBitsPerSample as i32 == 8 {
                    128
                } else {
                    0
                },
            );
        } else {
            (*dsb).resampleinmixer = TRUE;
        }
    } else {
        (*dsb).tmp_buffer_len = (*dsb).buflen;
        (*dsb).max_buffer_len = (*dsb).tmp_buffer_len;
    }
    (*dsb).buf_mixpos = DSOUND_secpos_to_bufpos(
        dsb,
        (*dsb).sec_mixpos,
        0,
        core::ptr::null_mut(),
    );
}
#[no_mangle]
pub unsafe extern "C" fn DSOUND_CheckEvent(dsb: *const IDirectSoundBufferImpl, playpos: DWORD, len: i32) {
    let mut i: i32;
    let mut offset: DWORD;
    let mut event: LPDSBPOSITIONNOTIFY;

    if (*dsb).nrofnotifies == 0 {
        return;
    }

    i = 0;
    while i < (*dsb).nrofnotifies {
        'body: {
            event = (*dsb).notifies.offset(i as isize);
            offset = (*event).dwOffset;

            if offset == (DSBPN_OFFSETSTOP) as u32 {
                if (*dsb).state == STATE_STOPPED {
                    SetEvent((*event).hEventNotify);
                }
                break 'body;
            }
            if playpos.wrapping_add(len as DWORD) >= (*dsb).buflen {
                if offset < playpos.wrapping_add(len as DWORD).wrapping_rem((*dsb).buflen)
                    || offset >= playpos
                {
                    SetEvent((*event).hEventNotify);
                }
            } else {
                if offset >= playpos && offset < playpos.wrapping_add(len as DWORD) {
                    SetEvent((*event).hEventNotify);
                }
            }
        }
        i += 1;
    }
}

unsafe extern "C" fn cp_fields(
    dsb: *const IDirectSoundBufferImpl,
    ibuf: *const BYTE,
    obuf: *mut BYTE,
    istride: UINT,
    ostride: UINT,
    count: UINT,
    freqAcc: UINT,
    adj: UINT,
) {
    let mut device: *mut DirectSoundDevice = (*dsb).device;
    let mut istep: INT = (*(*dsb).pwfx).wBitsPerSample as INT / 8;
    let mut ostep: INT = (*(*device).pwfx).wBitsPerSample as INT / 8;

    if (*(*device).pwfx).nChannels as ::core::ffi::c_int
        == (*(*dsb).pwfx).nChannels as ::core::ffi::c_int
        || ((*(*device).pwfx).nChannels as ::core::ffi::c_int == 2 as ::core::ffi::c_int
            && (*(*dsb).pwfx).nChannels as ::core::ffi::c_int == 6 as ::core::ffi::c_int)
        || ((*(*device).pwfx).nChannels as ::core::ffi::c_int == 8 as ::core::ffi::c_int
            && (*(*dsb).pwfx).nChannels as ::core::ffi::c_int == 2 as ::core::ffi::c_int)
        || ((*(*device).pwfx).nChannels as ::core::ffi::c_int == 6 as ::core::ffi::c_int
            && (*(*dsb).pwfx).nChannels as ::core::ffi::c_int == 2 as ::core::ffi::c_int)
    {
        (*dsb).convert.unwrap_unchecked()(
            ibuf as *const ::core::ffi::c_void,
            obuf as *mut ::core::ffi::c_void,
            istride,
            ostride,
            count as INT,
            freqAcc,
            adj,
        );
        if (*(*device).pwfx).nChannels as ::core::ffi::c_int == 2 as ::core::ffi::c_int
            || (*(*dsb).pwfx).nChannels as ::core::ffi::c_int == 2 as ::core::ffi::c_int
        {
            (*dsb).convert.unwrap_unchecked()(
                ibuf.offset(istep as isize) as *const ::core::ffi::c_void,
                obuf.offset(ostep as isize) as *mut ::core::ffi::c_void,
                istride,
                ostride,
                count as INT,
                freqAcc,
                adj,
            );
        }
        return;
    }

    if (*(*device).pwfx).nChannels as ::core::ffi::c_int == 1 as ::core::ffi::c_int
        && (*(*dsb).pwfx).nChannels as ::core::ffi::c_int == 2 as ::core::ffi::c_int
    {
        (*dsb).convert.unwrap_unchecked()(
            ibuf as *const ::core::ffi::c_void,
            obuf as *mut ::core::ffi::c_void,
            istride,
            ostride,
            count as INT,
            freqAcc,
            adj,
        );
        return;
    }

    if (*(*device).pwfx).nChannels as ::core::ffi::c_int == 2 as ::core::ffi::c_int
        && (*(*dsb).pwfx).nChannels as ::core::ffi::c_int == 1 as ::core::ffi::c_int
    {
        (*dsb).convert.unwrap_unchecked()(
            ibuf as *const ::core::ffi::c_void,
            obuf as *mut ::core::ffi::c_void,
            istride,
            ostride,
            count as INT,
            freqAcc,
            adj,
        );
        (*dsb).convert.unwrap_unchecked()(
            ibuf as *const ::core::ffi::c_void,
            obuf.offset(ostep as isize) as *mut ::core::ffi::c_void,
            istride,
            ostride,
            count as INT,
            freqAcc,
            adj,
        );
        return;
    }

    WARN!(
        "Unable to remap channels: device=%u, buffer=%u\n",
        (*(*device).pwfx).nChannels as ::core::ffi::c_int,
        (*(*dsb).pwfx).nChannels as ::core::ffi::c_int
    );
}

unsafe extern "C" fn DSOUND_BufPtrDiff(buflen: DWORD, ptr1: DWORD, ptr2: DWORD) -> DWORD {
    /* If these asserts fail, the problem is not here, but in the underlying code */
    assert!(ptr1 < buflen);
    assert!(ptr2 < buflen);
    if ptr1 >= ptr2 {
        return ptr1.wrapping_sub(ptr2);
    } else {
        return buflen.wrapping_add(ptr1).wrapping_sub(ptr2);
    }
}
#[no_mangle]
pub unsafe extern "C" fn DSOUND_MixToTemporary(dsb: *const IDirectSoundBufferImpl, mut writepos: DWORD, mut len: DWORD, mut inmixer: BOOL) {
    let mut size: INT = 0;
    let mut ibp: *mut BYTE = core::ptr::null_mut();
    let mut obp: *mut BYTE = core::ptr::null_mut();
    let mut obp_begin: *mut BYTE = core::ptr::null_mut();
    let mut iAdvance: INT = (*(*dsb).pwfx).nBlockAlign as INT;
    let mut oAdvance: INT = (*(*(*dsb).device).pwfx).nBlockAlign as INT;
    let mut freqAcc: DWORD = 0;
    let mut target_writepos: DWORD = 0;
    let mut overshot: DWORD = 0;
    let mut maxlen: DWORD = 0;

    /* We resample only when needed */
    if (!(*dsb).tmp_buffer.is_null() && inmixer != 0)
        || ((*dsb).tmp_buffer.is_null() && inmixer == 0)
        || ((*dsb).resampleinmixer != inmixer as ULONG)
    {
        return;
    }

    if !(writepos.wrapping_add(len) <= (*dsb).buflen) {
        _assert(
            b"writepos + len <= dsb->buflen\0" as *const u8 as *const core::ffi::c_char,
            b"dll/directx/wine/dsound/mixer.c\0"
                as *const u8 as *const core::ffi::c_char,
            344 as core::ffi::c_uint,
        );
    }
    if inmixer != 0 && writepos.wrapping_add(len) < (*dsb).buflen {
        len = len.wrapping_add((*(*dsb).pwfx).nBlockAlign as DWORD);
    }

    maxlen = DSOUND_secpos_to_bufpos(dsb, len, 0, core::ptr::null_mut::<DWORD>());

    ibp = (*(*dsb).buffer).memory.offset(writepos as isize) as *mut BYTE;
    if inmixer == 0 {
        obp_begin = (*dsb).tmp_buffer as *mut BYTE;
    } else if (*(*dsb).device).tmp_buffer_len < maxlen || (*(*dsb).device).tmp_buffer.is_null() {
        (*(*dsb).device).tmp_buffer_len = maxlen;
        if !(*(*dsb).device).tmp_buffer.is_null() {
            (*(*dsb).device).tmp_buffer = HeapReAlloc(
                GetProcessHeap(),
                0,
                (*(*dsb).device).tmp_buffer as PVOID,
                maxlen as SIZE_T,
            ) as LPBYTE;
        } else {
            (*(*dsb).device).tmp_buffer = HeapAlloc(
                GetProcessHeap(),
                0,
                maxlen as SIZE_T,
            ) as LPBYTE;
        }
        obp_begin = (*(*dsb).device).tmp_buffer as *mut BYTE;
    } else {
        obp_begin = (*(*dsb).device).tmp_buffer as *mut BYTE;
    }

    size = len.wrapping_div(iAdvance as DWORD) as INT;

    /* Check for same sample rate */
    if (*dsb).freq == (*(*(*dsb).device).pwfx).nSamplesPerSec {
        WINE_TRACE!("(%p) Same sample rate %d = primary %d\n", dsb, (*dsb).freq, (*(*(*dsb).device).pwfx).nSamplesPerSec);

        obp = obp_begin;
        if inmixer == 0 {
            obp = obp.offset(
                writepos
                    .wrapping_div(iAdvance as DWORD)
                    .wrapping_mul(oAdvance as DWORD) as isize,
            );
        }

        cp_fields(
            dsb,
            ibp,
            obp,
            iAdvance as UINT,
            oAdvance as UINT,
            size as UINT,
            0,
            (1 << DSOUND_FREQSHIFT) as UINT,
        );
        return;
    }

    /* Mix in different sample rates */
    WINE_TRACE!("(%p) Adjusting frequency: %d -> %d\n", dsb, (*dsb).freq, (*(*(*dsb).device).pwfx).nSamplesPerSec);

    target_writepos = DSOUND_secpos_to_bufpos(
        dsb,
        writepos,
        (*dsb).sec_mixpos,
        core::ptr::addr_of_mut!(freqAcc),
    );
    overshot = freqAcc >> DSOUND_FREQSHIFT;
    if overshot != 0 {
        if overshot >= size as DWORD {
            return;
        }
        size = (size as DWORD).wrapping_sub(overshot) as INT;
        writepos = writepos.wrapping_add(overshot.wrapping_mul(iAdvance as DWORD));
        if writepos >= (*dsb).buflen {
            return;
        }
        ibp = (*(*dsb).buffer).memory.offset(writepos as isize) as *mut BYTE;
        freqAcc &= ((1 << DSOUND_FREQSHIFT) - 1) as DWORD;
        WINE_TRACE!("Overshot: %d, freqAcc: %04x\n", overshot, freqAcc);
    }

    if inmixer == 0 {
        obp = obp_begin.offset(target_writepos as isize);
    } else {
        obp = obp_begin;
    }

    /* FIXME: Small problem here when we're overwriting buf_mixpos, it then STILL uses old freqAcc, not sure if it matters or not */
    cp_fields(
        dsb,
        ibp,
        obp,
        iAdvance as UINT,
        oAdvance as UINT,
        size as UINT,
        freqAcc as UINT,
        (*dsb).freqAdjust as UINT,
    );
}

unsafe extern "C" fn DSOUND_MixerVol(dsb: *const IDirectSoundBufferImpl, len: INT) -> LPBYTE {
    let mut i: INT = 0;
    let mut bpc: *mut BYTE = core::ptr::null_mut();
    let mut bps: *mut INT16 = core::ptr::null_mut();
    let mut mems: *mut INT16 = core::ptr::null_mut();
    let mut vLeft: DWORD = 0;
    let mut vRight: DWORD = 0;
    let mut nChannels: INT = (*(*(*dsb).device).pwfx).nChannels as INT;
    let mut mem: LPBYTE = (if !(*dsb).tmp_buffer.is_null() {
        (*dsb).tmp_buffer
    } else {
        (*(*dsb).buffer).memory
    })
    .offset((*dsb).buf_mixpos as isize);

    if (*dsb).resampleinmixer != 0 {
        mem = (*(*dsb).device).tmp_buffer;
    }

    if (((*dsb).dsbd.dwFlags & DSBCAPS_CTRLPAN) == 0 || (*dsb).volpan.lPan == 0)
        && (((*dsb).dsbd.dwFlags & DSBCAPS_CTRLVOLUME) == 0 || (*dsb).volpan.lVolume == 0)
        && ((*dsb).dsbd.dwFlags & DSBCAPS_CTRL3D) == 0
    {
        return core::ptr::null_mut::<BYTE>();
    }

    if nChannels != 1 && nChannels != 2 {
        return core::ptr::null_mut::<BYTE>();
    }

    if (*(*(*dsb).device).pwfx).wBitsPerSample as INT != 8
        && (*(*(*dsb).device).pwfx).wBitsPerSample as INT != 16
    {
        return core::ptr::null_mut::<BYTE>();
    }

    if (*(*dsb).device).tmp_buffer_len < len as DWORD || (*(*dsb).device).tmp_buffer.is_null() {
        assert!((*dsb).resampleinmixer == 0);
        (*(*dsb).device).tmp_buffer_len = len as DWORD;
        if !(*(*dsb).device).tmp_buffer.is_null() {
            (*(*dsb).device).tmp_buffer = HeapReAlloc(
                GetProcessHeap(),
                0,
                (*(*dsb).device).tmp_buffer as PVOID,
                len as SIZE_T,
            ) as LPBYTE;
        } else {
            (*(*dsb).device).tmp_buffer = HeapAlloc(
                GetProcessHeap(),
                0,
                len as SIZE_T,
            ) as LPBYTE;
        }
    }

    bpc = (*(*dsb).device).tmp_buffer as *mut BYTE;
    bps = bpc as *mut INT16;
    mems = mem as *mut INT16;
    vLeft = (*dsb).volpan.dwTotalLeftAmpFactor;
    if nChannels > 1 {
        vRight = (*dsb).volpan.dwTotalRightAmpFactor;
    } else {
        vRight = vLeft;
    }

    match (*(*(*dsb).device).pwfx).wBitsPerSample as INT {
        8 => {
            i = 0 as INT;
            while i < len - 1 {
                let fresh15 = mem;
                mem = mem.offset(1);
                let fresh16 = bpc;
                bpc = bpc.offset(1);
                *fresh16 = ((((*fresh15 as INT - 128) as DWORD)
                    .wrapping_mul(vLeft)
                    >> 16)
                    .wrapping_add(128)) as BYTE;
                let fresh17 = mem;
                mem = mem.offset(1);
                let fresh18 = bpc;
                bpc = bpc.offset(1);
                *fresh18 = ((((*fresh17 as INT - 128) as DWORD)
                    .wrapping_mul(vRight)
                    >> 16)
                    .wrapping_add(128)) as BYTE;
                i += 2;
            }
            if len % 2 == 1 && nChannels == 1 {
                let fresh19 = mem;
                mem = mem.offset(1);
                let fresh20 = bpc;
                bpc = bpc.offset(1);
                *fresh20 = ((((*fresh19 as INT - 128) as DWORD)
                    .wrapping_mul(vLeft)
                    >> 16)
                    .wrapping_add(128)) as BYTE;
            }
        }
        16 => {
            i = 0 as INT;
            while i < len - 3 {
                let fresh21 = mems;
                mems = mems.offset(1);
                let fresh22 = bps;
                bps = bps.offset(1);
                *fresh22 = ((*fresh21 as DWORD).wrapping_mul(vLeft) >> 16) as INT16;
                let fresh23 = mems;
                mems = mems.offset(1);
                let fresh24 = bps;
                bps = bps.offset(1);
                *fresh24 = ((*fresh23 as DWORD).wrapping_mul(vRight) >> 16) as INT16;
                i += 4;
            }
            if len % 4 == 2 && nChannels == 1 {
                let fresh25 = mems;
                mems = mems.offset(1);
                let fresh26 = bps;
                bps = bps.offset(1);
                *fresh26 = ((*fresh25 as INT as DWORD).wrapping_mul(vLeft) >> 16) as INT16;
            }
        }
        _ => {}
    }
    return (*(*dsb).device).tmp_buffer;
}

unsafe extern "C" fn DSOUND_MixInBuffer(
    dsb: *mut IDirectSoundBufferImpl,
    writepos: DWORD,
    fraglen: DWORD,
) -> DWORD {
    let mut len: INT = fraglen as INT;
    let mut ilen: INT = 0;
    let mut ibuf: *mut BYTE = (if !(*dsb).tmp_buffer.is_null() {
        (*dsb).tmp_buffer
    } else {
        (*(*dsb).buffer).memory
    })
    .offset((*dsb).buf_mixpos as isize);
    let mut volbuf: *mut BYTE = core::ptr::null_mut::<BYTE>();
    let mut oldpos: DWORD = 0;
    let mut mixbufpos: DWORD = 0;

    assert!((*dsb).buf_mixpos.wrapping_add(len as DWORD) <= (*dsb).tmp_buffer_len);

    if (len as INT) % ((*(*(*dsb).device).pwfx).nBlockAlign as INT) != 0 {
        let mut nBlockAlign: INT = (*(*(*dsb).device).pwfx).nBlockAlign as INT;
        ERR!(
            "length not a multiple of block size, len = %d, block size = %d\n",
            len,
            nBlockAlign
        );
        len -= len % nBlockAlign;
    }

    DSOUND_MixToTemporary(
        dsb,
        (*dsb).sec_mixpos,
        DSOUND_bufpos_to_secpos(dsb, (*dsb).buf_mixpos.wrapping_add(len as DWORD))
            .wrapping_sub((*dsb).sec_mixpos),
        (TRUE) as BOOL,
    );
    if (*dsb).resampleinmixer != 0 {
        ibuf = (*(*dsb).device).tmp_buffer as *mut BYTE;
    }

    volbuf = DSOUND_MixerVol(dsb, len) as *mut BYTE;
    if !volbuf.is_null() {
        ibuf = volbuf;
    }

    mixbufpos = DSOUND_bufpos_to_mixpos((*dsb).device, writepos);
    if writepos.wrapping_add(len as DWORD) <= (*(*dsb).device).buflen {
        (*(*dsb).device)
            .mixfunction
            .unwrap_unchecked()(
            ibuf as *const core::ffi::c_void,
            (*(*dsb).device).mix_buffer.offset(mixbufpos as isize) as *mut core::ffi::c_void,
            len as core::ffi::c_uint,
        );
    } else {
        let mut todo: DWORD = (*(*dsb).device).buflen.wrapping_sub(writepos);
        (*(*dsb).device)
            .mixfunction
            .unwrap_unchecked()(
            ibuf as *const core::ffi::c_void,
            (*(*dsb).device).mix_buffer.offset(mixbufpos as isize) as *mut core::ffi::c_void,
            todo as core::ffi::c_uint,
        );
        (*(*dsb).device)
            .mixfunction
            .unwrap_unchecked()(
            ibuf.offset(todo as isize) as *const core::ffi::c_void,
            (*(*dsb).device).mix_buffer as *mut core::ffi::c_void,
            (len as DWORD).wrapping_sub(todo) as core::ffi::c_uint,
        );
    }

    oldpos = (*dsb).sec_mixpos;
    (*dsb).buf_mixpos = (*dsb).buf_mixpos.wrapping_add(len as DWORD);

    if (*dsb).buf_mixpos >= (*dsb).tmp_buffer_len {
        if (*dsb).buf_mixpos > (*dsb).tmp_buffer_len {
            ERR!(
                "Mixpos (%u) past buflen (%u), capping...\n",
                (*dsb).buf_mixpos,
                (*dsb).tmp_buffer_len
            );
        }
        if (*dsb).playflags & DSBPLAY_LOOPING as DWORD != 0 {
            (*dsb).buf_mixpos = (*dsb).buf_mixpos.wrapping_sub((*dsb).tmp_buffer_len);
        } else if (*dsb).buf_mixpos >= (*dsb).tmp_buffer_len {
            (*dsb).sec_mixpos = 0 as DWORD;
            (*dsb).buf_mixpos = (*dsb).sec_mixpos;
            (*dsb).state = STATE_STOPPED as DWORD;
        }
        DSOUND_RecalcFreqAcc(dsb);
    }

    (*dsb).sec_mixpos = DSOUND_bufpos_to_secpos(dsb, (*dsb).buf_mixpos);
    ilen = DSOUND_BufPtrDiff((*dsb).buflen, (*dsb).sec_mixpos, oldpos) as INT;

    if (*dsb).dsbd.dwFlags & DSBCAPS_CTRLPOSITIONNOTIFY as DWORD != 0
        && (*dsb).state != STATE_STARTING as DWORD
    {
        DSOUND_CheckEvent(dsb, oldpos, ilen);
    }

    (*dsb).primary_mixpos = (*dsb).primary_mixpos.wrapping_add(len as DWORD);
    if (*dsb).primary_mixpos >= (*(*dsb).device).buflen {
        (*dsb).primary_mixpos = (*dsb).primary_mixpos.wrapping_sub((*(*dsb).device).buflen);
    }
    return len as DWORD;
}

unsafe extern "C" fn DSOUND_MixOne(
    dsb: *mut IDirectSoundBufferImpl,
    mut writepos: DWORD,
    mut mixlen: DWORD,
) -> DWORD {
    /* The buffer's primary_mixpos may be before or after the device
     * buffer's mixpos, but both must be ahead of writepos. */
    let mut primary_done: DWORD;

    /* If leading in, only mix about 20 ms, and 'skip' mixing the rest, for more fluid pointer advancement */
    if (*dsb).leadin != 0 && (*dsb).state == (STATE_STARTING as DWORD) {
        if mixlen > (2 as DWORD).wrapping_mul((*(*dsb).device).fraglen) {
            (*dsb).primary_mixpos = (*dsb).primary_mixpos.wrapping_add(
                mixlen.wrapping_sub(
                    (2 as DWORD).wrapping_mul((*(*dsb).device).fraglen),
                ),
            );
            (*dsb).primary_mixpos = (*dsb).primary_mixpos.wrapping_rem((*(*dsb).device).buflen);
        }
    }
    (*dsb).leadin = FALSE as DWORD;

    /* calculate how much pre-buffering has already been done for this buffer */
    primary_done = DSOUND_BufPtrDiff(
        (*(*dsb).device).buflen,
        (*dsb).primary_mixpos,
        writepos,
    );

    /* sanity */
    if mixlen < primary_done {
        /* Should *NEVER* happen */
        ERR!(
            "Fatal error. Under/Overflow? primary_done=%d, mixpos=%d/%d (%d/%d), primary_mixpos=%d, writepos=%d, mixlen=%d\n",
            primary_done,
            (*dsb).buf_mixpos,
            (*dsb).tmp_buffer_len,
            (*dsb).sec_mixpos,
            (*dsb).buflen,
            (*dsb).primary_mixpos,
            writepos,
            mixlen
        );
        (*dsb).primary_mixpos = writepos.wrapping_add(mixlen);
        (*dsb).primary_mixpos = (*dsb).primary_mixpos.wrapping_rem((*(*dsb).device).buflen);
        return mixlen;
    }

    /* take into account already mixed data */
    mixlen = mixlen.wrapping_sub(primary_done);

    if mixlen == 0 {
        return primary_done;
    }

    /* First try to mix to the end of the buffer if possible
     * Theoretically it would allow for better optimization
     */
    if mixlen.wrapping_add((*dsb).buf_mixpos) >= (*dsb).tmp_buffer_len {
        let mut newmixed: DWORD;
        let mut mixfirst: DWORD = (*dsb).tmp_buffer_len.wrapping_sub((*dsb).buf_mixpos);
        newmixed = DSOUND_MixInBuffer(dsb, (*dsb).primary_mixpos, mixfirst);
        mixlen = mixlen.wrapping_sub(newmixed);

        if ((*dsb).playflags & (DSBPLAY_LOOPING as DWORD)) != 0 {
            while newmixed != 0 && mixlen != 0 {
                mixfirst = if (*dsb).tmp_buffer_len < mixlen {
                    (*dsb).tmp_buffer_len
                } else {
                    mixlen
                };
                newmixed = DSOUND_MixInBuffer(dsb, (*dsb).primary_mixpos, mixfirst);
                mixlen = mixlen.wrapping_sub(newmixed);
            }
        }
    } else {
        DSOUND_MixInBuffer(dsb, (*dsb).primary_mixpos, mixlen);
    }

    /* re-calculate the primary done */
    primary_done = DSOUND_BufPtrDiff(
        (*(*dsb).device).buflen,
        (*dsb).primary_mixpos,
        writepos,
    );

    /* Report back the total prebuffered amount for this buffer */
    return primary_done;
}

unsafe extern "C" fn DSOUND_MixToPrimary(device: *const DirectSoundDevice, writepos: DWORD, mixlen: DWORD, recover: BOOL, all_stopped: *mut BOOL) -> DWORD {
    let mut i: INT = 0;
    let mut len: INT = 0;
    let mut minlen: DWORD = 0 as DWORD;
    let mut dsb: *mut IDirectSoundBufferImpl = ::core::ptr::null_mut::<IDirectSoundBufferImpl>();

    /* unless we find a running buffer, all have stopped */
    *all_stopped = TRUE as BOOL;

    i = 0 as INT;
    while i < (*device).nrofbuffers {
        dsb = *(*device).buffers.offset(i as isize);

        if (*dsb).buflen != 0 && (*dsb).state != 0 && (*dsb).hwbuf.is_null() {
            RtlAcquireResourceShared(core::ptr::addr_of_mut!((*dsb).lock), TRUE as BYTE);
            /* if buffer is stopping it is stopped now */
            if (*dsb).state == STATE_STOPPING as DWORD {
                (*dsb).state = STATE_STOPPED as DWORD;
                DSOUND_CheckEvent(dsb, 0 as DWORD, 0 as INT);
            } else if (*dsb).state != STATE_STOPPED as DWORD {

                /* if recovering, reset the mix position */
                if (*dsb).state == STATE_STARTING as DWORD || recover != 0 {
                    (*dsb).primary_mixpos = writepos;
                }

                /* if the buffer was starting, it must be playing now */
                if (*dsb).state == STATE_STARTING as DWORD {
                    (*dsb).state = STATE_PLAYING as DWORD;
                }

                /* mix next buffer into the main buffer */
                len = DSOUND_MixOne(dsb, writepos, mixlen) as INT;

                if minlen == 0 {
                    minlen = len as DWORD;
                }
                /* record the minimum length mixed from all buffers */
                /* we only want to return the length which *all* buffers have mixed */
                else if len != 0 {
                    minlen = if (len as DWORD) < minlen {
                        len as DWORD
                    } else {
                        minlen
                    };
                }

                *all_stopped = FALSE as BOOL;
            }
            RtlReleaseResource(core::ptr::addr_of_mut!((*dsb).lock));
        }
        i += 1;
    }

    return minlen;
}

unsafe extern "C" fn DSOUND_WaveQueue(device: *mut DirectSoundDevice, force: BOOL) {
    let mut prebuf_frags: DWORD = 0;
    let mut wave_writepos: DWORD = 0;
    let mut wave_fragpos: DWORD = 0;
    let mut i: DWORD = 0;

    /* calculate the current wave frag position */
    wave_fragpos = ((*device).pwplay as DWORD)
        .wrapping_add((*device).pwqueue as DWORD)
        .wrapping_rem((*device).helfrags as DWORD);

    /* calculate the current wave write position */
    wave_writepos = wave_fragpos.wrapping_mul((*device).fraglen as DWORD);

    if force == 0 {
        /* check remaining prebuffered frags */
        prebuf_frags = ((*device).mixpos as DWORD).wrapping_div((*device).fraglen as DWORD);
        if prebuf_frags == (*device).helfrags as DWORD {
            prebuf_frags = prebuf_frags.wrapping_sub(1);
        }
        if prebuf_frags < wave_fragpos {
            prebuf_frags = prebuf_frags.wrapping_add((*device).helfrags as DWORD);
        }
        prebuf_frags = prebuf_frags.wrapping_sub(wave_fragpos);
    } else {
        /* buffer the maximum amount of frags */
        prebuf_frags = (*device).prebuf as DWORD;
    }

    /* limit to the queue we have left */
    if prebuf_frags.wrapping_add((*device).pwqueue as DWORD) > (*device).prebuf as DWORD {
        prebuf_frags = ((*device).prebuf as DWORD).wrapping_sub((*device).pwqueue as DWORD);
    }

    /* adjust queue */
    (*device).pwqueue = ((*device).pwqueue as DWORD).wrapping_add(prebuf_frags) as UINT as UINT;

    /* get out of CS when calling the wave system */
    LeaveCriticalSection(core::ptr::addr_of_mut!((*device).mixlock));
    /* **** */

    /* queue up the new buffers */
    i = 0;
    while i < prebuf_frags {
        waveOutWrite(
            (*device).hwo,
            (*device).pwave.offset(wave_fragpos as isize) as LPWAVEHDR,
            core::mem::size_of::<WAVEHDR>() as UINT,
        );
        wave_fragpos = wave_fragpos.wrapping_add(1);
        wave_fragpos = wave_fragpos.wrapping_rem((*device).helfrags as DWORD);
        i = i.wrapping_add(1);
    }

    /* **** */
    EnterCriticalSection(core::ptr::addr_of_mut!((*device).mixlock));
}

unsafe extern "C" fn DSOUND_PerformMix(device: *mut DirectSoundDevice) {
    TRACE!("(%p)\n", device);
    EnterCriticalSection(core::ptr::addr_of_mut!((*device).mixlock));

    if (*device).priolevel != DSSCL_WRITEPRIMARY {
        let mut recover: BOOL = (FALSE as i32);
        let mut all_stopped: BOOL = (FALSE as i32);
        let mut playpos: DWORD = 0;
        let mut writepos: DWORD = 0;
        let mut writelead: DWORD = 0;
        let mut maxq: DWORD = 0;
        let mut frag: DWORD = 0;
        let mut prebuff_max: DWORD = 0;
        let mut prebuff_left: DWORD = 0;
        let mut size1: DWORD = 0;
        let mut size2: DWORD = 0;
        let mut mixplaypos: DWORD = 0;
        let mut mixplaypos2: DWORD = 0;
        let mut buf1: LPVOID = core::ptr::null_mut();
        let mut buf2: LPVOID = core::ptr::null_mut();
        let mut lock: BOOL = (!(*device).hwbuf.is_null()
            && (((*device).drvdesc.dwFlags & DSDDESC_DONTNEEDPRIMARYLOCK) == 0)) as BOOL;
        let mut nfiller: i32 = 0;

        /* the sound of silence */
        nfiller = if (*(*device).pwfx).wBitsPerSample as i32 == 8 {
            128
        } else {
            0
        };

        /* get the position in the primary buffer */
        if DSOUND_PrimaryGetPosition(
            device,
            core::ptr::addr_of_mut!(playpos),
            core::ptr::addr_of_mut!(writepos),
        ) != 0
        {
            LeaveCriticalSection(core::ptr::addr_of_mut!((*device).mixlock));
            return;
        }

        TRACE!(
            "primary playpos=%d, writepos=%d, clrpos=%d, mixpos=%d, buflen=%d\n",
            playpos,
            writepos,
            (*device).playpos,
            (*device).mixpos,
            (*device).buflen
        );

        assert!((*device).playpos < (*device).buflen);

        mixplaypos = DSOUND_bufpos_to_mixpos(device, (*device).playpos);
        mixplaypos2 = DSOUND_bufpos_to_mixpos(device, playpos);

        /* calc maximum prebuff */
        prebuff_max = ((*device).prebuf as DWORD).wrapping_mul((*device).fraglen);
        if (*device).hwbuf.is_null()
            && playpos.wrapping_add(prebuff_max)
                >= ((*device).helfrags as DWORD).wrapping_mul((*device).fraglen)
        {
            prebuff_max = prebuff_max.wrapping_add(
                (*device)
                    .buflen
                    .wrapping_sub(((*device).helfrags as DWORD).wrapping_mul((*device).fraglen)),
            );
        }

        /* check how close we are to an underrun. It occurs when the writepos overtakes the mixpos */
        prebuff_left = DSOUND_BufPtrDiff((*device).buflen, (*device).mixpos, playpos);
        writelead = DSOUND_BufPtrDiff((*device).buflen, writepos, playpos);

        /* check for underrun. underrun occurs when the write position passes the mix position
         * also wipe out just-played sound data */
        if (prebuff_left > prebuff_max)
            || ((*device).state == STATE_STOPPED)
            || ((*device).state == STATE_STARTING)
        {
            if (*device).state == STATE_STOPPING
                || (*device).state == STATE_PLAYING
            {
                WARN!("Probable buffer underrun\n");
            } else {
                TRACE!("Buffer starting or buffer underrun\n");
            }

            /* recover mixing for all buffers */
            recover = (TRUE as i32);

            /* reset mix position to write position */
            (*device).mixpos = writepos;

            ZeroMemory!(
                (*device).mix_buffer as LPVOID,
                (*device).mix_buffer_len as usize,
            );
            ZeroMemory!((*device).buffer as LPVOID, (*device).buflen as usize);
        } else if playpos < (*device).playpos {
            buf1 = (*device).buffer.add((*device).playpos as usize) as LPVOID;
            buf2 = (*device).buffer as LPVOID;
            size1 = (*device).buflen.wrapping_sub((*device).playpos);
            size2 = playpos;
            FillMemory!(
                (*device).mix_buffer.add(mixplaypos as usize) as LPVOID,
                (*device).mix_buffer_len.wrapping_sub(mixplaypos) as usize,
                0 as u8,
            );
            FillMemory!(
                (*device).mix_buffer as LPVOID,
                mixplaypos2 as usize,
                0 as u8,
            );
            if lock != 0 {
                IDsDriverBuffer_Lock(
                    (*device).hwbuf,
                    core::ptr::addr_of_mut!(buf1),
                    core::ptr::addr_of_mut!(size1),
                    core::ptr::addr_of_mut!(buf2),
                    core::ptr::addr_of_mut!(size2),
                    (*device).playpos,
                    size1.wrapping_add(size2),
                    0,
                );
            }
            FillMemory!(buf1, size1 as usize, nfiller as u8);
            if playpos != 0 && (buf2.is_null() || size2 == 0) {
                FIXME!(
                    "%d: (%d, %d)=>(%d, %d) There should be an additional buffer here!!\n",
                    (line!() as ::core::ffi::c_int),
                    (*device).playpos,
                    (*device).mixpos,
                    playpos,
                    writepos
                );
            }
            FillMemory!(buf2, size2 as usize, nfiller as u8);
            if lock != 0 {
                IDsDriverBuffer_Unlock((*device).hwbuf, buf1, size1, buf2, size2);
            }
        } else {
            buf1 = (*device).buffer.add((*device).playpos as usize) as LPVOID;
            buf2 = core::ptr::null_mut();
            size1 = playpos.wrapping_sub((*device).playpos);
            size2 = 0;
            FillMemory!(
                (*device).mix_buffer.add(mixplaypos as usize) as LPVOID,
                mixplaypos2.wrapping_sub(mixplaypos) as usize,
                0 as u8,
            );
            if lock != 0 {
                IDsDriverBuffer_Lock(
                    (*device).hwbuf,
                    core::ptr::addr_of_mut!(buf1),
                    core::ptr::addr_of_mut!(size1),
                    core::ptr::addr_of_mut!(buf2),
                    core::ptr::addr_of_mut!(size2),
                    (*device).playpos,
                    size1.wrapping_add(size2),
                    0,
                );
            }
            FillMemory!(buf1, size1 as usize, nfiller as u8);
            if !buf2.is_null() && size2 != 0 {
                FIXME!(
                    "%d: There should be no additional buffer here!!\n",
                    867
                );
                FillMemory!(buf2, size2 as usize, nfiller as u8);
            }
            if lock != 0 {
                IDsDriverBuffer_Unlock((*device).hwbuf, buf1, size1, buf2, size2);
            }
        }
        (*device).playpos = playpos;

        /* find the maximum we can prebuffer from current write position */
        maxq = if writelead < prebuff_max {
            prebuff_max.wrapping_sub(writelead)
        } else {
            0
        };

        TRACE!(
            "prebuff_left = %d, prebuff_max = %dx%d=%d, writelead=%d\n",
            prebuff_left,
            (*device).prebuf,
            (*device).fraglen,
            prebuff_max,
            writelead
        );

        if lock != 0 {
            IDsDriverBuffer_Lock(
                (*device).hwbuf,
                core::ptr::addr_of_mut!(buf1),
                core::ptr::addr_of_mut!(size1),
                core::ptr::addr_of_mut!(buf2),
                core::ptr::addr_of_mut!(size2),
                writepos,
                maxq,
                0,
            );
        }

        /* do the mixing */
        frag = DSOUND_MixToPrimary(
            device,
            writepos,
            maxq,
            recover,
            core::ptr::addr_of_mut!(all_stopped),
        );

        if frag.wrapping_add(writepos) > (*device).buflen {
            let mut todo: DWORD = (*device).buflen.wrapping_sub(writepos);
            (*device).normfunction.unwrap_unchecked()(
                (*device)
                    .mix_buffer
                    .add(DSOUND_bufpos_to_mixpos(device, writepos) as usize)
                    as *const core::ffi::c_void,
                (*device).buffer.add(writepos as usize) as *mut core::ffi::c_void,
                todo as u32,
            );
            (*device).normfunction.unwrap_unchecked()(
                (*device).mix_buffer as *const core::ffi::c_void,
                (*device).buffer as *mut core::ffi::c_void,
                frag.wrapping_sub(todo) as u32,
            );
        } else {
            (*device).normfunction.unwrap_unchecked()(
                (*device)
                    .mix_buffer
                    .add(DSOUND_bufpos_to_mixpos(device, writepos) as usize)
                    as *const core::ffi::c_void,
                (*device).buffer.add(writepos as usize) as *mut core::ffi::c_void,
                frag as u32,
            );
        }

        /* update the mix position, taking wrap-around into account */
        (*device).mixpos = writepos.wrapping_add(frag);
        (*device).mixpos = (*device).mixpos.wrapping_rem((*device).buflen);

        if lock != 0 {
            let mut frag2: DWORD = if frag > size1 {
                frag.wrapping_sub(size1)
            } else {
                0
            };
            frag = frag.wrapping_sub(frag2);
            if frag2 > size2 {
                FIXME!(
                    "Buffering too much! (%d, %d, %d, %d)\n",
                    maxq,
                    frag,
                    size2,
                    frag2.wrapping_sub(size2)
                );
                frag2 = size2;
            }
            IDsDriverBuffer_Unlock((*device).hwbuf, buf1, frag, buf2, frag2);
        }

        /* update prebuff left */
        prebuff_left = DSOUND_BufPtrDiff((*device).buflen, (*device).mixpos, playpos);

        /* check if have a whole fragment */
        if prebuff_left >= (*device).fraglen {
            /* update the wave queue if using wave system */
            if (*device).hwbuf.is_null() {
                DSOUND_WaveQueue(device, (FALSE) as BOOL);
            }

            /* buffers are full. start playing if applicable */
            if (*device).state == STATE_STARTING {
                TRACE!("started primary buffer\n");
                if DSOUND_PrimaryPlay(device) != (DS_OK as i32) {
                    WARN!("DSOUND_PrimaryPlay failed\n");
                } else {
                    /* we are playing now */
                    (*device).state = STATE_PLAYING;
                }
            }

            /* buffers are full. start stopping if applicable */
            if (*device).state == STATE_STOPPED {
                TRACE!("restarting primary buffer\n");
                if DSOUND_PrimaryPlay(device) != (DS_OK as i32) {
                    WARN!("DSOUND_PrimaryPlay failed\n");
                } else {
                    /* start stopping again. as soon as there is no more data, it will stop */
                    (*device).state = STATE_STOPPING;
                }
            }
        }
        /* if device was stopping, its for sure stopped when all buffers have stopped */
        else if (all_stopped != (FALSE as BOOL))
            && ((*device).state == STATE_STOPPING)
        {
            TRACE!("All buffers have stopped. Stopping primary buffer\n");
            (*device).state = STATE_STOPPED;

            /* stop the primary buffer now */
            DSOUND_PrimaryStop(device);
        }
    } else {
        /* update the wave queue if using wave system */
        if (*device).hwbuf.is_null() {
            DSOUND_WaveQueue(device, (TRUE) as BOOL);
        } else {
            /* Keep alsa happy, which needs GetPosition called once every 10 ms */
            IDsDriverBuffer_GetPosition(
                (*device).hwbuf,
                core::ptr::null_mut::<DWORD>(),
                core::ptr::null_mut::<DWORD>(),
            );
        }

        /* in the DSSCL_WRITEPRIMARY mode, the app is totally in charge... */
        if (*device).state == STATE_STARTING {
            if DSOUND_PrimaryPlay(device) != (DS_OK as i32) {
                WARN!("DSOUND_PrimaryPlay failed\n");
            } else {
                (*device).state = STATE_PLAYING;
            }
        } else if (*device).state == STATE_STOPPING {
            if DSOUND_PrimaryStop(device) != (DS_OK as i32) {
                WARN!("DSOUND_PrimaryStop failed\n");
            } else {
                (*device).state = STATE_STOPPED;
            }
        }
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*device).mixlock));
}
#[no_mangle]
pub unsafe extern "system" fn DSOUND_timer(timerID: UINT, msg: UINT, dwUser: DWORD_PTR, dw1: DWORD_PTR, dw2: DWORD_PTR) {
    let device: *mut DirectSoundDevice = dwUser as *mut DirectSoundDevice;
    let start_time: DWORD = GetTickCount();
    let end_time: DWORD;

    if DSOUND_renderer[(*device).drvdesc.dnDevNode as usize] != device {
        ERR!("dsound died without killing us?\n");
        timeKillEvent(timerID);
        timeEndPeriod(DS_TIME_RES as UINT);
        return;
    }

    RtlAcquireResourceShared(&raw mut (*device).buffer_list_lock, TRUE as BYTE);

    if (*device).ref_ != 0 {
        DSOUND_PerformMix(device);
    }

    RtlReleaseResource(&raw mut (*device).buffer_list_lock);

    end_time = GetTickCount();
}
#[no_mangle]
pub unsafe extern "system" fn DSOUND_callback(hwo: HWAVEOUT, msg: UINT, dwUser: DWORD_PTR, dw1: DWORD_PTR, dw2: DWORD_PTR) {
    let device: *mut DirectSoundDevice = dwUser as *mut DirectSoundDevice;

    /* check if packet completed from wave driver */
    if msg == MM_WOM_DONE as UINT {

        /* **** */
        EnterCriticalSection(core::ptr::addr_of_mut!((*device).mixlock));

        /* update playpos */
        (*device).pwplay = (*device).pwplay.wrapping_add(1);
        (*device).pwplay = (*device).pwplay.wrapping_rem((*device).helfrags);

        /* sanity */
        if (*device).pwqueue == 0 as UINT {
            ERR!("Wave queue corrupted!\n");
        }

        /* update queue */
        (*device).pwqueue = (*device).pwqueue.wrapping_sub(1);

        LeaveCriticalSection(core::ptr::addr_of_mut!((*device).mixlock));
        /* **** */
    }
}
