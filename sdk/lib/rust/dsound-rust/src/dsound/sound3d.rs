use crate::dsound::buffer::secondarybuffer_destroy;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const DEFAULT_VELOCITY: i32 = 340;


unsafe extern "C" fn ScalarProduct(mut a: *const D3DVECTOR, mut b: *const D3DVECTOR) -> D3DVALUE {
    let mut c: D3DVALUE = 0.;
    c = ((*a).x * (*b).x + (*a).y * (*b).y + (*a).z * (*b).z) as D3DVALUE;
    WINE_TRACE!("(%f,%f,%f) * (%f,%f,%f) = %f)\n", (*a).x as core::ffi::c_double, (*a).y as core::ffi::c_double, (*a).z as core::ffi::c_double, (*b).x as core::ffi::c_double, (*b).y as core::ffi::c_double, (*b).z as core::ffi::c_double, c as core::ffi::c_double);
    return c;
}

unsafe extern "C" fn VectorProduct(mut a: *const D3DVECTOR, mut b: *const D3DVECTOR) -> D3DVECTOR {
    let mut c: D3DVECTOR = D3DVECTOR {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    c.x = (*a).y * (*b).z - (*a).z * (*b).y;
    c.y = (*a).z * (*b).x - (*a).x * (*b).z;
    c.z = (*a).x * (*b).y - (*a).y * (*b).x;
    WINE_TRACE!("(%f,%f,%f) x (%f,%f,%f) = (%f,%f,%f)\n", (*a).x as core::ffi::c_double, (*a).y as core::ffi::c_double, (*a).z as core::ffi::c_double, (*b).x as core::ffi::c_double, (*b).y as core::ffi::c_double, (*b).z as core::ffi::c_double, c.x as core::ffi::c_double, c.y as core::ffi::c_double, c.z as core::ffi::c_double);
    return c;
}

unsafe extern "C" fn VectorMagnitude(a: *const D3DVECTOR) -> D3DVALUE {
    let mut l: D3DVALUE = 0.;
    l = sqrt(ScalarProduct(a, a) as f64) as D3DVALUE;
    return l;
}

unsafe extern "C" fn RadToDeg(mut angle: D3DVALUE) -> D3DVALUE {
    let mut newangle: D3DVALUE = 0.;
    newangle = (angle as f64 * (360.0 / (2.0 * M_PI))) as D3DVALUE;
    WINE_TRACE!("%f rad = %f deg\n", angle as ::core::ffi::c_double, newangle as ::core::ffi::c_double);
    return newangle;
}

unsafe extern "C" fn AngleBetweenVectorsRad(mut a: *const D3DVECTOR, mut b: *const D3DVECTOR) -> D3DVALUE {
    let mut la: D3DVALUE = 0.0;
    let mut lb: D3DVALUE = 0.0;
    let mut product: D3DVALUE = 0.0;
    let mut angle: D3DVALUE = 0.0;
    let mut cos: D3DVALUE = 0.0;
    /* definition of scalar product: a*b = |a|*|b|*cos... therefore: */
    product = ScalarProduct(a, b);
    la = VectorMagnitude(a);
    lb = VectorMagnitude(b);
    if la == 0.0 || lb == 0.0 {
        return 0.0;
    }

    cos = product / (la * lb);
    angle = acos(cos as f64) as D3DVALUE;

    return angle;
}

unsafe extern "C" fn AngleBetweenVectorsDeg(a: *const D3DVECTOR, b: *const D3DVECTOR) -> D3DVALUE {
	return RadToDeg(AngleBetweenVectorsRad(a, b));
}

unsafe extern "C" fn VectorBetweenTwoPoints(
    mut a: *const D3DVECTOR,
    mut b: *const D3DVECTOR,
) -> D3DVECTOR {
    let mut c: D3DVECTOR = D3DVECTOR {
        x: 0.,
        y: 0.,
        z: 0.,
    };
    c.x = (*b).x - (*a).x;
    c.y = (*b).y - (*a).y;
    c.z = (*b).z - (*a).z;
    WINE_TRACE!("A (%f,%f,%f), B (%f,%f,%f), AB = (%f,%f,%f)\n", (*a).x as f64, (*a).y as f64, (*a).z as f64, (*b).x as f64, (*b).y as f64, (*b).z as f64, c.x as f64, c.y as f64, c.z as f64);
    return c;
}
#[no_mangle]
pub unsafe extern "C" fn DSOUND_Calc3DBuffer(dsb: *mut IDirectSoundBufferImpl) {
    /* volume, at which the sound will be played after all calcs. */
    let mut lVolume: D3DVALUE = 0 as D3DVALUE;
    /* stuff for distance related stuff calc. */
    let mut vDistance: D3DVECTOR = _D3DVECTOR {
        x: 0.,
        y: 0.,
        z: 0.,
    };
    let mut flDistance: D3DVALUE = 0 as D3DVALUE;
    /* panning related stuff */
    let mut flAngle: D3DVALUE = 0.;
    let mut vLeft: D3DVECTOR = _D3DVECTOR {
        x: 0.,
        y: 0.,
        z: 0.,
    };

    /* initial buffer volume */
    lVolume = (*dsb).ds3db_lVolume as D3DVALUE;

    match (*dsb).ds3db_ds3db.dwMode {
        DS3DMODE_DISABLE => {
            /* TRACE("3D processing disabled\n"); */
            /* this one is here only to eliminate annoying warning message */
            DSOUND_RecalcVolPan(core::ptr::addr_of_mut!((*dsb).volpan));
        }
        DS3DMODE_NORMAL => {
            /* TRACE("Normal 3D processing mode\n"); */
            /* we need to calculate distance between buffer and listener*/
            vDistance = VectorBetweenTwoPoints(
                core::ptr::addr_of_mut!((*dsb).ds3db_ds3db.vPosition),
                core::ptr::addr_of_mut!((*(*dsb).device).ds3dl.vPosition),
            );
            flDistance = VectorMagnitude(core::ptr::addr_of_mut!(vDistance));
        }
        DS3DMODE_HEADRELATIVE => {
            /* TRACE("Head-relative 3D processing mode\n"); */
            /* distance between buffer and listener is same as buffer's position */
            flDistance = VectorMagnitude(core::ptr::addr_of_mut!((*dsb).ds3db_ds3db.vPosition));
        }
        _ => {}
    }

    if flDistance > (*dsb).ds3db_ds3db.flMaxDistance {
        /* some apps don't want you to hear too distant sounds... */
        if ((*dsb).dsbd.dwFlags & (DSBCAPS_MUTE3DATMAXDISTANCE as DWORD)) != 0 {
            (*dsb).volpan.lVolume = DSBVOLUME_MIN as LONG;
            DSOUND_RecalcVolPan(core::ptr::addr_of_mut!((*dsb).volpan));
            /* i guess mixing here would be a waste of power */
            return;
        } else {
            flDistance = (*dsb).ds3db_ds3db.flMaxDistance;
        }
    }

    if flDistance < (*dsb).ds3db_ds3db.flMinDistance {
        flDistance = (*dsb).ds3db_ds3db.flMinDistance;
    }

    /* attenuation proportional to the distance squared, converted to millibels as in lVolume*/
    lVolume = (lVolume as f64
        - log10(
            (flDistance / (*dsb).ds3db_ds3db.flMinDistance * flDistance
                / (*dsb).ds3db_ds3db.flMinDistance) as f64,
        ) * 1000 as f64) as D3DVALUE;

    /* conning */
    /* sometimes it happens that vConeOrientation vector = (0,0,0); in this case angle is "nan" and it's useless*/
    if (*dsb).ds3db_ds3db.vConeOrientation.x == 0 as f32
        && (*dsb).ds3db_ds3db.vConeOrientation.y == 0 as f32
        && (*dsb).ds3db_ds3db.vConeOrientation.z == 0 as f32
    {
        /* TRACE("conning: cones not set\n"); */
    } else {
        /* calculate angle */
        flAngle = AngleBetweenVectorsDeg(
            core::ptr::addr_of_mut!((*dsb).ds3db_ds3db.vConeOrientation),
            core::ptr::addr_of_mut!(vDistance),
        );
        /* if by any chance it happens that OutsideConeAngle = InsideConeAngle (that means that conning has no effect) */
        if (*dsb).ds3db_ds3db.dwInsideConeAngle != (*dsb).ds3db_ds3db.dwOutsideConeAngle {
            /* my test show that for my way of calc., we need only half of angles */
            let mut dwInsideConeAngle: DWORD =
                (*dsb).ds3db_ds3db.dwInsideConeAngle.wrapping_div(2 as DWORD);
            let mut dwOutsideConeAngle: DWORD =
                (*dsb).ds3db_ds3db.dwOutsideConeAngle.wrapping_div(2 as DWORD);
            if dwOutsideConeAngle == dwInsideConeAngle {
                dwOutsideConeAngle = dwOutsideConeAngle.wrapping_add(1);
            }

            /* full volume */
            if flAngle < dwInsideConeAngle as D3DVALUE {
                flAngle = dwInsideConeAngle as D3DVALUE;
            }
            /* min (app defined) volume */
            if flAngle > dwOutsideConeAngle as D3DVALUE {
                flAngle = dwOutsideConeAngle as D3DVALUE;
            }
            /* this probably isn't the right thing, but it's ok for the time being */
            lVolume += ((*dsb).ds3db_ds3db.lConeOutsideVolume as DWORD)
                .wrapping_div(dwOutsideConeAngle.wrapping_sub(dwInsideConeAngle))
                as D3DVALUE
                * flAngle;
        }
        /* TRACE("conning: Angle = %f deg; ...\n"); */
    }
    (*dsb).volpan.lVolume = lVolume as LONG;

    /* panning */
    if (*(*dsb).device).ds3dl.vPosition.x == (*dsb).ds3db_ds3db.vPosition.x
        && (*(*dsb).device).ds3dl.vPosition.y == (*dsb).ds3db_ds3db.vPosition.y
        && (*(*dsb).device).ds3dl.vPosition.z == (*dsb).ds3db_ds3db.vPosition.z
    {
        (*dsb).volpan.lPan = 0 as LONG;
        flAngle = 0.0;
    } else {
        vDistance = VectorBetweenTwoPoints(
            core::ptr::addr_of_mut!((*(*dsb).device).ds3dl.vPosition),
            core::ptr::addr_of_mut!((*dsb).ds3db_ds3db.vPosition),
        );
        vLeft = VectorProduct(
            core::ptr::addr_of_mut!((*(*dsb).device).ds3dl.vOrientFront),
            core::ptr::addr_of_mut!((*(*dsb).device).ds3dl.vOrientTop),
        );
        flAngle = AngleBetweenVectorsRad(
            core::ptr::addr_of_mut!(vLeft),
            core::ptr::addr_of_mut!(vDistance),
        );
        /* for now, we'll use "linear formula" (which is probably incorrect); if someone has it in book, correct it */
        (*dsb).volpan.lPan = ((((10000 * 2) as D3DVALUE
            * flAngle) as f64
            / M_PI)
            - 10000 as f64) as LONG;
    }

    /* FIXME: Doppler Effect disabled since i have no idea which frequency to change and how to do it */

    /* time for remix */
    DSOUND_RecalcVolPan(core::ptr::addr_of_mut!((*dsb).volpan));
}

unsafe extern "C" fn DSOUND_Mix3DBuffer(dsb: *mut IDirectSoundBufferImpl) {
    // TRACE("(%p)\n",dsb);
    DSOUND_Calc3DBuffer(dsb);
}

unsafe extern "C" fn DSOUND_ChangeListener(ds3dl: *mut IDirectSound3DListenerImpl) {
    let mut i: i32;
    i = 0;
    while i < (*(*ds3dl).device).nrofbuffers {
        if (**(*(*ds3dl).device).buffers.offset(i as isize)).ds3db_need_recalc != 0 {
            DSOUND_Mix3DBuffer(*(*(*ds3dl).device).buffers.offset(i as isize));
        }
        i += 1;
    }
}

unsafe extern "system" fn IDirectSound3DBufferImpl_QueryInterface(
    iface: LPDIRECTSOUND3DBUFFER,
    riid: REFIID,
    ppobj: *mut LPVOID,
) -> HRESULT {
    let This: *mut IDirectSound3DBufferImpl = iface as *mut IDirectSound3DBufferImpl;

    return IDirectSoundBuffer_QueryInterface(((*This).dsb as LPDIRECTSOUNDBUFFER8).cast(), riid, ppobj);
}
unsafe extern "system" fn IDirectSound3DBufferImpl_AddRef(iface: LPDIRECTSOUND3DBUFFER) -> ULONG {
    let This: *mut IDirectSound3DBufferImpl = iface as *mut IDirectSound3DBufferImpl;
    let mut ref_0: ULONG = InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_) as *mut core::ffi::c_long) as ULONG;

    TRACE!("(%p) ref was %d\n", This, ref_0.wrapping_sub(1 as ULONG));

    if ref_0 == 1 {
        InterlockedIncrement(core::ptr::addr_of_mut!((*(*This).dsb).numIfaces) as *mut core::ffi::c_long);
    }

    return ref_0;
}

unsafe extern "system" fn IDirectSound3DBufferImpl_Release(iface: LPDIRECTSOUND3DBUFFER) -> ULONG {
    let This: *mut IDirectSound3DBufferImpl = iface as *mut IDirectSound3DBufferImpl;
    let ref_0: ULONG = InterlockedDecrement(core::ptr::addr_of_mut!((*This).ref_) as *mut ::core::ffi::c_long) as ULONG;

    WINE_TRACE!("(%p) ref was %d\n", This, ref_0.wrapping_add(1 as ULONG));
    if ref_0 == 0 {
        (*(*This).dsb).ds3db = core::ptr::null_mut::<IDirectSound3DBufferImpl>();
        if InterlockedDecrement(core::ptr::addr_of_mut!((*(*This).dsb).numIfaces) as *mut ::core::ffi::c_long) == 0
        {
            secondarybuffer_destroy((*This).dsb);
        }
        HeapFree(GetProcessHeap(), 0, This as PVOID);
        WINE_TRACE!("(%p) released\n", This);
    }
    return ref_0;
}
unsafe extern "system" fn IDirectSound3DBufferImpl_GetAllParameters(
    mut iface: LPDIRECTSOUND3DBUFFER,
    mut lpDs3dBuffer: LPDS3DBUFFER,
) -> HRESULT {
    let mut This: *mut IDirectSound3DBufferImpl = iface as *mut IDirectSound3DBufferImpl;
    TRACE!("(%p,%p)\n", This, lpDs3dBuffer);

    if lpDs3dBuffer.is_null() {
        WARN!("invalid parameter: lpDs3dBuffer == NULL\n");
        return DSERR_INVALIDPARAM;
    }

    if ((*lpDs3dBuffer).dwSize as usize) < core::mem::size_of::<DS3DBUFFER>() {
        WARN!("invalid parameter: lpDs3dBuffer->dwSize = %d\n", (*lpDs3dBuffer).dwSize);
        return DSERR_INVALIDPARAM;
    }

    TRACE!("returning: all parameters\n");
    *lpDs3dBuffer = (*(*This).dsb).ds3db_ds3db;
    return (DS_OK) as HRESULT;
}

unsafe extern "system" fn IDirectSound3DBufferImpl_GetConeAngles(
    iface: LPDIRECTSOUND3DBUFFER,
    lpdwInsideConeAngle: LPDWORD,
    lpdwOutsideConeAngle: LPDWORD,
) -> HRESULT {
    let This: *mut IDirectSound3DBufferImpl = iface as *mut IDirectSound3DBufferImpl;
    WINE_TRACE!("returning: Inside Cone Angle = %d degrees; Outside Cone Angle = %d degrees\n", (*(*This).dsb).ds3db_ds3db.dwInsideConeAngle, (*(*This).dsb).ds3db_ds3db.dwOutsideConeAngle);
    *lpdwInsideConeAngle = (*(*This).dsb).ds3db_ds3db.dwInsideConeAngle;
    *lpdwOutsideConeAngle = (*(*This).dsb).ds3db_ds3db.dwOutsideConeAngle;
    return (DS_OK) as HRESULT;
}
unsafe extern "system" fn IDirectSound3DBufferImpl_GetConeOrientation(
    mut iface: LPDIRECTSOUND3DBUFFER,
    mut lpvConeOrientation: LPD3DVECTOR,
) -> HRESULT {
    let mut This: *mut IDirectSound3DBufferImpl = iface as *mut IDirectSound3DBufferImpl;
    WINE_TRACE!("returning: Cone Orientation vector = (%f,%f,%f)\n", (*(*This).dsb).ds3db_ds3db.vConeOrientation.x as ::core::ffi::c_double, (*(*This).dsb).ds3db_ds3db.vConeOrientation.y as ::core::ffi::c_double, (*(*This).dsb).ds3db_ds3db.vConeOrientation.z as ::core::ffi::c_double);
    *lpvConeOrientation = (*(*This).dsb).ds3db_ds3db.vConeOrientation;
    return (DS_OK) as HRESULT;
}
unsafe extern "system" fn IDirectSound3DBufferImpl_GetConeOutsideVolume(
    mut iface: LPDIRECTSOUND3DBUFFER,
    mut lplConeOutsideVolume: LPLONG,
) -> HRESULT {
    let mut This: *mut IDirectSound3DBufferImpl = iface as *mut IDirectSound3DBufferImpl;
    WINE_TRACE!("returning: Cone Outside Volume = %d\n", (*(*This).dsb).ds3db_ds3db.lConeOutsideVolume);
    *lplConeOutsideVolume = (*(*This).dsb).ds3db_ds3db.lConeOutsideVolume as ::core::ffi::c_long;
    return (DS_OK) as HRESULT;
}

unsafe extern "system" fn IDirectSound3DBufferImpl_GetMaxDistance(
    iface: LPDIRECTSOUND3DBUFFER,
    lpfMaxDistance: LPD3DVALUE,
) -> HRESULT {
    let mut This: *mut IDirectSound3DBufferImpl = iface as *mut IDirectSound3DBufferImpl;
    *lpfMaxDistance = (*(*This).dsb).ds3db_ds3db.flMaxDistance;
    return (DS_OK) as HRESULT;
}
unsafe extern "system" fn IDirectSound3DBufferImpl_GetMinDistance(
    iface: LPDIRECTSOUND3DBUFFER,
    lpfMinDistance: LPD3DVALUE,
) -> HRESULT {
    let This: *mut IDirectSound3DBufferImpl = iface as *mut IDirectSound3DBufferImpl;
    *lpfMinDistance = (*(*This).dsb).ds3db_ds3db.flMinDistance;
    return (DS_OK) as HRESULT;
}
unsafe extern "system" fn IDirectSound3DBufferImpl_GetMode(
    mut iface: LPDIRECTSOUND3DBUFFER,
    mut lpdwMode: LPDWORD,
) -> HRESULT {
    let mut This: *mut IDirectSound3DBufferImpl = iface as *mut IDirectSound3DBufferImpl;
    WINE_TRACE!("returning: Mode = %d\n", (*(*This).dsb).ds3db_ds3db.dwMode);
    *lpdwMode = (*(*This).dsb).ds3db_ds3db.dwMode;
    return (DS_OK) as HRESULT;
}
unsafe extern "system" fn IDirectSound3DBufferImpl_GetPosition(
    mut iface: LPDIRECTSOUND3DBUFFER,
    mut lpvPosition: LPD3DVECTOR,
) -> HRESULT {
    let mut This: *mut IDirectSound3DBufferImpl = iface as *mut IDirectSound3DBufferImpl;
    WINE_TRACE!("returning: Position vector = (%f,%f,%f)\n", (*(*This).dsb).ds3db_ds3db.vPosition.x as ::core::ffi::c_double, (*(*This).dsb).ds3db_ds3db.vPosition.y as ::core::ffi::c_double, (*(*This).dsb).ds3db_ds3db.vPosition.z as ::core::ffi::c_double);
    *lpvPosition = (*(*This).dsb).ds3db_ds3db.vPosition;
    return (DS_OK) as HRESULT;
}
unsafe extern "system" fn IDirectSound3DBufferImpl_GetVelocity(
    mut iface: LPDIRECTSOUND3DBUFFER,
    mut lpvVelocity: LPD3DVECTOR,
) -> HRESULT {
    let mut This: *mut IDirectSound3DBufferImpl = iface as *mut IDirectSound3DBufferImpl;
    WINE_TRACE!("returning: Velocity vector = (%f,%f,%f)\n", (*(*This).dsb).ds3db_ds3db.vVelocity.x as ::core::ffi::c_double, (*(*This).dsb).ds3db_ds3db.vVelocity.y as ::core::ffi::c_double, (*(*This).dsb).ds3db_ds3db.vVelocity.z as ::core::ffi::c_double);
    *lpvVelocity = (*(*This).dsb).ds3db_ds3db.vVelocity;
    return DS_OK as HRESULT;
}
unsafe extern "system" fn IDirectSound3DBufferImpl_SetAllParameters(
    iface: LPDIRECTSOUND3DBUFFER,
    lpcDs3dBuffer: LPCDS3DBUFFER,
    dwApply: DWORD,
) -> HRESULT {
    let This: *mut IDirectSound3DBufferImpl = iface as *mut IDirectSound3DBufferImpl;
    let mut status: DWORD = (DSERR_INVALIDPARAM) as u32;

    if lpcDs3dBuffer.is_null() {
        WARN!("invalid parameter: lpcDs3dBuffer == NULL\n");
        return status as HRESULT;
    }

    if (*lpcDs3dBuffer).dwSize as usize != core::mem::size_of::<DS3DBUFFER>() {
        WARN!("invalid parameter: lpcDs3dBuffer->dwSize = %d\n", (*lpcDs3dBuffer).dwSize);
        return status as HRESULT;
    }

    (*(*This).dsb).ds3db_ds3db = *lpcDs3dBuffer;

    if dwApply == DS3D_IMMEDIATE as DWORD {
        DSOUND_Mix3DBuffer((*This).dsb);
    }
    (*(*This).dsb).ds3db_need_recalc = TRUE as BOOL;
    status = DS_OK as DWORD;

    return status as HRESULT;
}

unsafe extern "system" fn IDirectSound3DBufferImpl_SetConeAngles(
    iface: LPDIRECTSOUND3DBUFFER,
    dwInsideConeAngle: DWORD,
    dwOutsideConeAngle: DWORD,
    dwApply: DWORD,
) -> HRESULT {
    let mut This: *mut IDirectSound3DBufferImpl = iface as *mut IDirectSound3DBufferImpl;
    (*(*This).dsb).ds3db_ds3db.dwInsideConeAngle = dwInsideConeAngle;
    (*(*This).dsb).ds3db_ds3db.dwOutsideConeAngle = dwOutsideConeAngle;
    if dwApply == DS3D_IMMEDIATE {
        DSOUND_Mix3DBuffer((*This).dsb);
    }
    (*(*This).dsb).ds3db_need_recalc = (TRUE as i32);
    return (DS_OK) as HRESULT;
}

unsafe extern "system" fn IDirectSound3DBufferImpl_SetConeOrientation(
    iface: LPDIRECTSOUND3DBUFFER,
    x: D3DVALUE,
    y: D3DVALUE,
    z: D3DVALUE,
    dwApply: DWORD,
) -> HRESULT {
    let mut This: *mut IDirectSound3DBufferImpl = iface as *mut IDirectSound3DBufferImpl;
    (*(*This).dsb).ds3db_ds3db.vConeOrientation.x = x;
    (*(*This).dsb).ds3db_ds3db.vConeOrientation.y = y;
    (*(*This).dsb).ds3db_ds3db.vConeOrientation.z = z;
    if dwApply == DS3D_IMMEDIATE {
        (*(*This).dsb).ds3db_need_recalc = (FALSE as i32);
        DSOUND_Mix3DBuffer((*This).dsb);
    }
    (*(*This).dsb).ds3db_need_recalc = (TRUE as i32);
    return (DS_OK) as HRESULT;
}
unsafe extern "system" fn IDirectSound3DBufferImpl_SetConeOutsideVolume(
    iface: LPDIRECTSOUND3DBUFFER,
    lConeOutsideVolume: LONG,
    dwApply: DWORD,
) -> HRESULT {
    let This: *mut IDirectSound3DBufferImpl = iface as *mut IDirectSound3DBufferImpl;
    (*(*This).dsb).ds3db_ds3db.lConeOutsideVolume = lConeOutsideVolume;
    if dwApply == DS3D_IMMEDIATE as DWORD {
        (*(*This).dsb).ds3db_need_recalc = FALSE as BOOL;
        DSOUND_Mix3DBuffer((*This).dsb);
    }
    (*(*This).dsb).ds3db_need_recalc = TRUE as BOOL;
    return DS_OK as HRESULT;
}

unsafe extern "system" fn IDirectSound3DBufferImpl_SetMaxDistance(
    iface: LPDIRECTSOUND3DBUFFER,
    fMaxDistance: D3DVALUE,
    dwApply: DWORD,
) -> HRESULT {
    let mut This: *mut IDirectSound3DBufferImpl = iface as *mut IDirectSound3DBufferImpl;
    (*(*This).dsb).ds3db_ds3db.flMaxDistance = fMaxDistance;
    if dwApply == DS3D_IMMEDIATE {
        (*(*This).dsb).ds3db_need_recalc = (FALSE as i32);
        DSOUND_Mix3DBuffer((*This).dsb);
    }
    (*(*This).dsb).ds3db_need_recalc = (TRUE as i32);
    return (DS_OK) as HRESULT;
}

unsafe extern "system" fn IDirectSound3DBufferImpl_SetMinDistance(
    iface: LPDIRECTSOUND3DBUFFER,
    fMinDistance: D3DVALUE,
    dwApply: DWORD,
) -> HRESULT {
    let This: *mut IDirectSound3DBufferImpl = iface as *mut IDirectSound3DBufferImpl;
    (*(*This).dsb).ds3db_ds3db.flMinDistance = fMinDistance;
    if dwApply == DS3D_IMMEDIATE {
        (*(*This).dsb).ds3db_need_recalc = (FALSE as i32);
        DSOUND_Mix3DBuffer((*This).dsb);
    }
    (*(*This).dsb).ds3db_need_recalc = (TRUE as i32);
    return (DS_OK) as HRESULT;
}
unsafe extern "system" fn IDirectSound3DBufferImpl_SetMode(
    iface: LPDIRECTSOUND3DBUFFER,
    dwMode: DWORD,
    dwApply: DWORD,
) -> HRESULT {
    let mut This: *mut IDirectSound3DBufferImpl = iface as *mut IDirectSound3DBufferImpl;
    (*(*This).dsb).ds3db_ds3db.dwMode = dwMode;
    if dwApply == DS3D_IMMEDIATE {
        (*(*This).dsb).ds3db_need_recalc = (FALSE as i32);
        DSOUND_Mix3DBuffer((*This).dsb);
    }
    (*(*This).dsb).ds3db_need_recalc = (TRUE as i32);
    return (DS_OK) as HRESULT;
}

unsafe extern "system" fn IDirectSound3DBufferImpl_SetPosition(
    iface: LPDIRECTSOUND3DBUFFER,
    x: D3DVALUE,
    y: D3DVALUE,
    z: D3DVALUE,
    dwApply: DWORD,
) -> HRESULT {
    let This: *mut IDirectSound3DBufferImpl = iface as *mut IDirectSound3DBufferImpl;
    (*(*This).dsb).ds3db_ds3db.vPosition.x = x;
    (*(*This).dsb).ds3db_ds3db.vPosition.y = y;
    (*(*This).dsb).ds3db_ds3db.vPosition.z = z;
    if dwApply == DS3D_IMMEDIATE {
        (*(*This).dsb).ds3db_need_recalc = (FALSE as i32);
        DSOUND_Mix3DBuffer((*This).dsb);
    }
    (*(*This).dsb).ds3db_need_recalc = (TRUE as i32);
    return (DS_OK) as HRESULT;
}

unsafe extern "system" fn IDirectSound3DBufferImpl_SetVelocity(
    iface: LPDIRECTSOUND3DBUFFER,
    x: D3DVALUE,
    y: D3DVALUE,
    z: D3DVALUE,
    dwApply: DWORD,
) -> HRESULT {
    let mut This: *mut IDirectSound3DBufferImpl = iface as *mut IDirectSound3DBufferImpl;
    WINE_TRACE!("setting: Velocity vector = (%f,%f,%f); dwApply = %d\n", x as f64, y as f64, z as f64, dwApply);
    (*(*This).dsb).ds3db_ds3db.vVelocity.x = x as f32;
    (*(*This).dsb).ds3db_ds3db.vVelocity.y = y as f32;
    (*(*This).dsb).ds3db_ds3db.vVelocity.z = z as f32;
    if dwApply == DS3D_IMMEDIATE {
        (*(*This).dsb).ds3db_need_recalc = (FALSE as i32);
        DSOUND_Mix3DBuffer((*This).dsb);
    }
    (*(*This).dsb).ds3db_need_recalc = (TRUE as i32);
    return (DS_OK) as HRESULT;
}
#[no_mangle]
pub unsafe extern "C" fn IDirectSound3DBufferImpl_Create(
    mut dsb: *mut IDirectSoundBufferImpl,
    mut pds3db: *mut *mut IDirectSound3DBufferImpl,
) -> HRESULT {
    let mut ds3db: *mut IDirectSound3DBufferImpl = core::ptr::null_mut::<IDirectSound3DBufferImpl>();

    ds3db = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY as DWORD,
        core::mem::size_of::<IDirectSound3DBufferImpl>() as SIZE_T,
    ) as *mut IDirectSound3DBufferImpl;

    if ds3db.is_null() {
        WARN!("out of memory\n");
        *pds3db = core::ptr::null_mut();
        return DSERR_OUTOFMEMORY;
    }

    (*ds3db).ref_ = 0 as LONG;
    (*ds3db).dsb = dsb;
    (*ds3db).lpVtbl = core::ptr::addr_of!(ds3dbvt);

    (*(*ds3db).dsb).ds3db_ds3db.dwSize = core::mem::size_of::<DS3DBUFFER>() as DWORD;
    (*(*ds3db).dsb).ds3db_ds3db.vPosition.x = 0.0f32;
    (*(*ds3db).dsb).ds3db_ds3db.vPosition.y = 0.0f32;
    (*(*ds3db).dsb).ds3db_ds3db.vPosition.z = 0.0f32;
    (*(*ds3db).dsb).ds3db_ds3db.vVelocity.x = 0.0f32;
    (*(*ds3db).dsb).ds3db_ds3db.vVelocity.y = 0.0f32;
    (*(*ds3db).dsb).ds3db_ds3db.vVelocity.z = 0.0f32;
    (*(*ds3db).dsb).ds3db_ds3db.dwInsideConeAngle = DS3D_DEFAULTCONEANGLE as DWORD;
    (*(*ds3db).dsb).ds3db_ds3db.dwOutsideConeAngle = DS3D_DEFAULTCONEANGLE as DWORD;
    (*(*ds3db).dsb).ds3db_ds3db.vConeOrientation.x = 0.0f32;
    (*(*ds3db).dsb).ds3db_ds3db.vConeOrientation.y = 0.0f32;
    (*(*ds3db).dsb).ds3db_ds3db.vConeOrientation.z = 0.0f32;
    (*(*ds3db).dsb).ds3db_ds3db.lConeOutsideVolume = DS3D_DEFAULTCONEOUTSIDEVOLUME as LONG;
    (*(*ds3db).dsb).ds3db_ds3db.flMinDistance = DS3D_DEFAULTMINDISTANCE as D3DVALUE;
    (*(*ds3db).dsb).ds3db_ds3db.flMaxDistance = DS3D_DEFAULTMAXDISTANCE as D3DVALUE;
    (*(*ds3db).dsb).ds3db_ds3db.dwMode = DS3DMODE_NORMAL as DWORD;

    (*(*ds3db).dsb).ds3db_need_recalc = TRUE as BOOL;

    *pds3db = ds3db;
    return S_OK;
}
#[no_mangle]
pub unsafe extern "C" fn IDirectSound3DBufferImpl_Destroy(
    pds3db: *mut IDirectSound3DBufferImpl,
) -> HRESULT {
    while IDirectSound3DBufferImpl_Release(pds3db as LPDIRECTSOUND3DBUFFER) > 0 {}

    return S_OK;
}

unsafe extern "system" fn IDirectSound3DListenerImpl_QueryInterface(
    iface: LPDIRECTSOUND3DLISTENER,
    riid: REFIID,
    ppobj: *mut LPVOID,
) -> HRESULT {
    let This: *mut IDirectSound3DListenerImpl = iface as *mut IDirectSound3DListenerImpl;

    DPRINT1!("(%p,%s,%p)\n", This, debugstr_guid(riid as *const _), ppobj);

    if ppobj.is_null() {
        WARN!("invalid parameter\n");
        return E_INVALIDARG;
    }

    *ppobj = core::ptr::null_mut(); /* assume failure */

    if IsEqualGUID!(riid, core::ptr::addr_of!(IID_IUnknown))
        || IsEqualGUID!(riid, core::ptr::addr_of!(IID_IDirectSound3DListener))
    {
        IDirectSound3DListener_AddRef(This as LPDIRECTSOUND3DLISTENER);
        *ppobj = This as LPVOID;
        return S_OK;
    }

    if IsEqualGUID!(riid, core::ptr::addr_of!(IID_IDirectSoundBuffer)) {
        *ppobj = core::ptr::addr_of_mut!((*(*(*This).device).primary).IDirectSoundBuffer8_iface) as LPVOID;
        IDirectSoundBuffer8_AddRef(core::ptr::addr_of_mut!(
            (*(*(*This).device).primary).IDirectSoundBuffer8_iface
        ));
        return S_OK;
    }

    DPRINT1!("Unknown IID %s\n", debugstr_guid(riid as *const _));
    return E_NOINTERFACE;
}

unsafe extern "system" fn IDirectSound3DListenerImpl_AddRef(
    mut iface: LPDIRECTSOUND3DLISTENER,
) -> ULONG {
    let mut This: *mut IDirectSound3DListenerImpl = iface as *mut IDirectSound3DListenerImpl;
    let mut ref_0: ULONG =
        InterlockedIncrement(&raw mut (*This).ref_ as *mut ::core::ffi::c_long) as ULONG;
    WINE_TRACE!("(%p) ref was %d\n", This, ref_0.wrapping_sub(1 as ULONG));
    if ref_0 == 1 {
        InterlockedIncrement(
            &raw mut (*(*(*This).device).primary).numIfaces as *mut ::core::ffi::c_long,
        );
    }
    return ref_0;
}

unsafe extern "system" fn IDirectSound3DListenerImpl_Release(iface: LPDIRECTSOUND3DLISTENER) -> ULONG {
    let mut This: *mut IDirectSound3DListenerImpl = iface as *mut IDirectSound3DListenerImpl;
    let mut ref_0: ULONG = InterlockedDecrement(core::ptr::addr_of_mut!((*This).ref_) as LPLONG) as ULONG;

    if ref_0 == 0 {
        (*(*This).device).listener = core::ptr::null_mut();
        if InterlockedDecrement(core::ptr::addr_of_mut!((*(*(*This).device).primary).numIfaces) as LPLONG) == 0 {
            primarybuffer_destroy((*(*This).device).primary);
        }
        HeapFree(GetProcessHeap(), 0, This as PVOID);
    }
    return ref_0;
}

unsafe extern "system" fn IDirectSound3DListenerImpl_GetAllParameter(
    iface: LPDIRECTSOUND3DLISTENER,
    lpDS3DL: LPDS3DLISTENER,
) -> HRESULT {
    let This: *mut IDirectSound3DListenerImpl = iface as *mut IDirectSound3DListenerImpl;

    TRACE!("(%p,%p)\n", This, lpDS3DL);

    if lpDS3DL.is_null() {
        WARN!("invalid parameter: lpDS3DL == NULL\n");
        return DSERR_INVALIDPARAM;
    }

    if ((*lpDS3DL).dwSize as usize) < core::mem::size_of::<DS3DLISTENER>() {
        WARN!("invalid parameter: lpDS3DL->dwSize = %d\n", (*lpDS3DL).dwSize);
        return DSERR_INVALIDPARAM;
    }

    TRACE!("returning: all parameters\n");

    *lpDS3DL = (*(*This).device).ds3dl;
    return (DS_OK) as HRESULT;
}
unsafe extern "system" fn IDirectSound3DListenerImpl_GetDistanceFactor(
    mut iface: LPDIRECTSOUND3DLISTENER,
    mut lpfDistanceFactor: LPD3DVALUE,
) -> HRESULT {
    let mut This: *mut IDirectSound3DListenerImpl = iface as *mut IDirectSound3DListenerImpl;
    WINE_TRACE!("returning: Distance Factor = %f\n", (*(*This).device).ds3dl.flDistanceFactor as ::core::ffi::c_double);
    *lpfDistanceFactor = (*(*This).device).ds3dl.flDistanceFactor as ::core::ffi::c_float;
    return (DS_OK) as HRESULT;
}

unsafe extern "system" fn IDirectSound3DListenerImpl_GetDopplerFactor(
    iface: LPDIRECTSOUND3DLISTENER,
    lpfDopplerFactor: LPD3DVALUE,
) -> HRESULT {
    let mut This: *mut IDirectSound3DListenerImpl = iface as *mut IDirectSound3DListenerImpl;
    WINE_TRACE!("returning: Doppler Factor = %f\n", (*(*This).device).ds3dl.flDopplerFactor as f64);
    *lpfDopplerFactor = (*(*This).device).ds3dl.flDopplerFactor as f32;
    return (DS_OK) as HRESULT;
}
unsafe extern "system" fn IDirectSound3DListenerImpl_GetOrientation(
    mut iface: LPDIRECTSOUND3DLISTENER,
    mut lpvOrientFront: LPD3DVECTOR,
    mut lpvOrientTop: LPD3DVECTOR,
) -> HRESULT {
    let mut This: *mut IDirectSound3DListenerImpl = iface as *mut IDirectSound3DListenerImpl;
    WINE_TRACE!("returning: OrientFront vector = (%f,%f,%f); OrientTop vector = (%f,%f,%f)\n", (*(*This).device).ds3dl.vOrientFront.x as f64, (*(*This).device).ds3dl.vOrientFront.y as f64, (*(*This).device).ds3dl.vOrientFront.z as f64, (*(*This).device).ds3dl.vOrientTop.x as f64, (*(*This).device).ds3dl.vOrientTop.y as f64, (*(*This).device).ds3dl.vOrientTop.z as f64);
    *lpvOrientFront = (*(*This).device).ds3dl.vOrientFront;
    *lpvOrientTop = (*(*This).device).ds3dl.vOrientTop;
    return (DS_OK) as HRESULT;
}

unsafe extern "system" fn IDirectSound3DListenerImpl_GetPosition(
    mut iface: LPDIRECTSOUND3DLISTENER,
    mut lpvPosition: LPD3DVECTOR,
) -> HRESULT {
    let mut This: *mut IDirectSound3DListenerImpl = iface as *mut IDirectSound3DListenerImpl;
    WINE_TRACE!("returning: Position vector = (%f,%f,%f)\n", (*(*This).device).ds3dl.vPosition.x as ::core::ffi::c_double, (*(*This).device).ds3dl.vPosition.y as ::core::ffi::c_double, (*(*This).device).ds3dl.vPosition.z as ::core::ffi::c_double);
    *lpvPosition = (*(*This).device).ds3dl.vPosition;
    return (DS_OK) as HRESULT;
}

unsafe extern "system" fn IDirectSound3DListenerImpl_GetRolloffFactor(
    iface: LPDIRECTSOUND3DLISTENER,
    lpfRolloffFactor: LPD3DVALUE,
) -> HRESULT {
    let This: *mut IDirectSound3DListenerImpl = iface as *mut IDirectSound3DListenerImpl;
    *lpfRolloffFactor = (*(*This).device).ds3dl.flRolloffFactor;
    return (DS_OK) as HRESULT;
}

unsafe extern "system" fn IDirectSound3DListenerImpl_GetVelocity(
    mut iface: LPDIRECTSOUND3DLISTENER,
    mut lpvVelocity: LPD3DVECTOR,
) -> HRESULT {
    let mut This: *mut IDirectSound3DListenerImpl = iface as *mut IDirectSound3DListenerImpl;
    WINE_TRACE!("returning: Velocity vector = (%f,%f,%f)\n", (*(*This).device).ds3dl.vVelocity.x as f64, (*(*This).device).ds3dl.vVelocity.y as f64, (*(*This).device).ds3dl.vVelocity.z as f64);
    *lpvVelocity = (*(*This).device).ds3dl.vVelocity;
    return DS_OK as HRESULT;
}

unsafe extern "system" fn IDirectSound3DListenerImpl_SetAllParameters(
    mut iface: LPDIRECTSOUND3DLISTENER,
    mut lpcDS3DL: LPCDS3DLISTENER,
    mut dwApply: DWORD,
) -> HRESULT {
    let mut This: *mut IDirectSound3DListenerImpl = iface as *mut IDirectSound3DListenerImpl;
    WINE_TRACE!("setting: all parameters; dwApply = %d\n", dwApply);
    (*(*This).device).ds3dl = *lpcDS3DL;
    if dwApply == DS3D_IMMEDIATE {
        (*(*This).device).ds3dl_need_recalc = (FALSE as i32);
        DSOUND_ChangeListener(This);
    }
    (*(*This).device).ds3dl_need_recalc = (TRUE as i32);
    return (DS_OK) as HRESULT;
}
unsafe extern "system" fn IDirectSound3DListenerImpl_SetDistanceFactor(
    mut iface: LPDIRECTSOUND3DLISTENER,
    mut fDistanceFactor: D3DVALUE,
    mut dwApply: DWORD,
) -> HRESULT {
    let mut This: *mut IDirectSound3DListenerImpl = iface as *mut IDirectSound3DListenerImpl;
    (*(*This).device).ds3dl.flDistanceFactor = fDistanceFactor;
    if dwApply == DS3D_IMMEDIATE as DWORD {
        (*(*This).device).ds3dl_need_recalc = FALSE as BOOL;
        DSOUND_ChangeListener(This);
    }
    (*(*This).device).ds3dl_need_recalc = TRUE as BOOL;
    return DS_OK as HRESULT;
}

unsafe extern "system" fn IDirectSound3DListenerImpl_SetDopplerFactor(
    iface: LPDIRECTSOUND3DLISTENER,
    fDopplerFactor: D3DVALUE,
    dwApply: DWORD,
) -> HRESULT {
    let mut This: *mut IDirectSound3DListenerImpl = iface as *mut IDirectSound3DListenerImpl;
    (*(*This).device).ds3dl.flDopplerFactor = fDopplerFactor;
    if dwApply == DS3D_IMMEDIATE {
        (*(*This).device).ds3dl_need_recalc = (FALSE as i32);
        DSOUND_ChangeListener(This);
    }
    (*(*This).device).ds3dl_need_recalc = (TRUE as i32);
    return (DS_OK) as HRESULT;
}

unsafe extern "system" fn IDirectSound3DListenerImpl_SetOrientation(
    iface: LPDIRECTSOUND3DLISTENER,
    xFront: D3DVALUE,
    yFront: D3DVALUE,
    zFront: D3DVALUE,
    xTop: D3DVALUE,
    yTop: D3DVALUE,
    zTop: D3DVALUE,
    dwApply: DWORD,
) -> HRESULT {
    let This: *mut IDirectSound3DListenerImpl = iface as *mut IDirectSound3DListenerImpl;
    (*(*This).device).ds3dl.vOrientFront.x = xFront;
    (*(*This).device).ds3dl.vOrientFront.y = yFront;
    (*(*This).device).ds3dl.vOrientFront.z = zFront;
    (*(*This).device).ds3dl.vOrientTop.x = xTop;
    (*(*This).device).ds3dl.vOrientTop.y = yTop;
    (*(*This).device).ds3dl.vOrientTop.z = zTop;
    if dwApply == DS3D_IMMEDIATE {
        (*(*This).device).ds3dl_need_recalc = (FALSE as i32);
        DSOUND_ChangeListener(This);
    }
    (*(*This).device).ds3dl_need_recalc = (TRUE as i32);
    return (DS_OK) as HRESULT;
}

unsafe extern "system" fn IDirectSound3DListenerImpl_SetPosition(
    iface: LPDIRECTSOUND3DLISTENER,
    x: D3DVALUE,
    y: D3DVALUE,
    z: D3DVALUE,
    dwApply: DWORD,
) -> HRESULT {
    let This: *mut IDirectSound3DListenerImpl = iface as *mut IDirectSound3DListenerImpl;
    (*(*This).device).ds3dl.vPosition.x = x;
    (*(*This).device).ds3dl.vPosition.y = y;
    (*(*This).device).ds3dl.vPosition.z = z;
    if dwApply == DS3D_IMMEDIATE {
        (*(*This).device).ds3dl_need_recalc = (FALSE as i32);
        DSOUND_ChangeListener(This);
    }
    (*(*This).device).ds3dl_need_recalc = (TRUE as i32);
    return (DS_OK) as HRESULT;
}

unsafe extern "system" fn IDirectSound3DListenerImpl_SetRolloffFactor(
    iface: LPDIRECTSOUND3DLISTENER,
    fRolloffFactor: D3DVALUE,
    dwApply: DWORD,
) -> HRESULT {
    let This: *mut IDirectSound3DListenerImpl = iface as *mut IDirectSound3DListenerImpl;
    (*(*This).device).ds3dl.flRolloffFactor = fRolloffFactor;
    if dwApply == DS3D_IMMEDIATE {
        (*(*This).device).ds3dl_need_recalc = (FALSE as i32);
        DSOUND_ChangeListener(This);
    }
    (*(*This).device).ds3dl_need_recalc = (TRUE as i32);
    return (DS_OK) as HRESULT;
}

unsafe extern "system" fn IDirectSound3DListenerImpl_SetVelocity(
	iface: LPDIRECTSOUND3DLISTENER,
	x: D3DVALUE,
	y: D3DVALUE,
	z: D3DVALUE,
	dwApply: DWORD,
) -> HRESULT {
	let mut This: *mut IDirectSound3DListenerImpl = iface as *mut IDirectSound3DListenerImpl;
	(*(*This).device).ds3dl.vVelocity.x = x as f32;
	(*(*This).device).ds3dl.vVelocity.y = y as f32;
	(*(*This).device).ds3dl.vVelocity.z = z as f32;
	if dwApply == DS3D_IMMEDIATE {
		(*(*This).device).ds3dl_need_recalc = (FALSE as i32);
		DSOUND_ChangeListener(This);
	}
	(*(*This).device).ds3dl_need_recalc = (TRUE as i32);
	return (DS_OK) as HRESULT;
}

unsafe extern "system" fn IDirectSound3DListenerImpl_CommitDeferredSettings(
    iface: LPDIRECTSOUND3DLISTENER,
) -> HRESULT {
    let This: *mut IDirectSound3DListenerImpl = iface as *mut IDirectSound3DListenerImpl;
    DSOUND_ChangeListener(This);
    return (DS_OK) as HRESULT;
}
#[no_mangle]
pub unsafe extern "C" fn IDirectSound3DListenerImpl_Create(device: *mut DirectSoundDevice, ppdsl: *mut *mut IDirectSound3DListenerImpl) -> HRESULT {
    let mut pdsl: *mut IDirectSound3DListenerImpl;
    pdsl = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY as DWORD, core::mem::size_of::<IDirectSound3DListenerImpl>() as SIZE_T) as *mut IDirectSound3DListenerImpl;

    if pdsl.is_null() {
        WARN!("out of memory\n");
        *ppdsl = core::ptr::null_mut();
        return DSERR_OUTOFMEMORY;
    }

    (*pdsl).ref_ = 0 as LONG;
    (*pdsl).lpVtbl = core::ptr::addr_of!(ds3dlvt);

    (*pdsl).device = device;

    (*(*pdsl).device).ds3dl.dwSize = core::mem::size_of::<DS3DLISTENER>() as DWORD;
    (*(*pdsl).device).ds3dl.vPosition.x = 0.0f32;
    (*(*pdsl).device).ds3dl.vPosition.y = 0.0f32;
    (*(*pdsl).device).ds3dl.vPosition.z = 0.0f32;
    (*(*pdsl).device).ds3dl.vVelocity.x = 0.0f32;
    (*(*pdsl).device).ds3dl.vVelocity.y = 0.0f32;
    (*(*pdsl).device).ds3dl.vVelocity.z = 0.0f32;
    (*(*pdsl).device).ds3dl.vOrientFront.x = 0.0f32;
    (*(*pdsl).device).ds3dl.vOrientFront.y = 0.0f32;
    (*(*pdsl).device).ds3dl.vOrientFront.z = 1.0f32;
    (*(*pdsl).device).ds3dl.vOrientTop.x = 0.0f32;
    (*(*pdsl).device).ds3dl.vOrientTop.y = 1.0f32;
    (*(*pdsl).device).ds3dl.vOrientTop.z = 0.0f32;
    (*(*pdsl).device).ds3dl.flDistanceFactor = DS3D_DEFAULTDISTANCEFACTOR as D3DVALUE;
    (*(*pdsl).device).ds3dl.flRolloffFactor = DS3D_DEFAULTROLLOFFFACTOR as D3DVALUE;
    (*(*pdsl).device).ds3dl.flDopplerFactor = DS3D_DEFAULTDOPPLERFACTOR as D3DVALUE;

    (*(*pdsl).device).ds3dl_need_recalc = TRUE as BOOL;

    *ppdsl = pdsl;
    return S_OK;
}

// COM vtable tables — generated from sound3d.c by vtbl_gen.py
/// sound3d.c:640 `static const IDirectSound3DBufferVtbl ds3dbvt`  (slot order: rust:types_amd64.rs)
pub static ds3dbvt: IDirectSound3DBufferVtbl = IDirectSound3DBufferVtbl {
    QueryInterface: Some(IDirectSound3DBufferImpl_QueryInterface),
    AddRef: Some(IDirectSound3DBufferImpl_AddRef),
    Release: Some(IDirectSound3DBufferImpl_Release),
    GetAllParameters: Some(IDirectSound3DBufferImpl_GetAllParameters),
    GetConeAngles: Some(IDirectSound3DBufferImpl_GetConeAngles),
    GetConeOrientation: Some(IDirectSound3DBufferImpl_GetConeOrientation),
    GetConeOutsideVolume: Some(IDirectSound3DBufferImpl_GetConeOutsideVolume),
    GetMaxDistance: Some(IDirectSound3DBufferImpl_GetMaxDistance),
    GetMinDistance: Some(IDirectSound3DBufferImpl_GetMinDistance),
    GetMode: Some(IDirectSound3DBufferImpl_GetMode),
    GetPosition: Some(IDirectSound3DBufferImpl_GetPosition),
    GetVelocity: Some(IDirectSound3DBufferImpl_GetVelocity),
    SetAllParameters: Some(IDirectSound3DBufferImpl_SetAllParameters),
    SetConeAngles: Some(IDirectSound3DBufferImpl_SetConeAngles),
    SetConeOrientation: Some(IDirectSound3DBufferImpl_SetConeOrientation),
    SetConeOutsideVolume: Some(IDirectSound3DBufferImpl_SetConeOutsideVolume),
    SetMaxDistance: Some(IDirectSound3DBufferImpl_SetMaxDistance),
    SetMinDistance: Some(IDirectSound3DBufferImpl_SetMinDistance),
    SetMode: Some(IDirectSound3DBufferImpl_SetMode),
    SetPosition: Some(IDirectSound3DBufferImpl_SetPosition),
    SetVelocity: Some(IDirectSound3DBufferImpl_SetVelocity),
};

/// sound3d.c:1010 `static const IDirectSound3DListenerVtbl ds3dlvt`  (slot order: rust:types_amd64.rs)
pub static ds3dlvt: IDirectSound3DListenerVtbl = IDirectSound3DListenerVtbl {
    QueryInterface: Some(IDirectSound3DListenerImpl_QueryInterface),
    AddRef: Some(IDirectSound3DListenerImpl_AddRef),
    Release: Some(IDirectSound3DListenerImpl_Release),
    GetAllParameters: Some(IDirectSound3DListenerImpl_GetAllParameter),
    GetDistanceFactor: Some(IDirectSound3DListenerImpl_GetDistanceFactor),
    GetDopplerFactor: Some(IDirectSound3DListenerImpl_GetDopplerFactor),
    GetOrientation: Some(IDirectSound3DListenerImpl_GetOrientation),
    GetPosition: Some(IDirectSound3DListenerImpl_GetPosition),
    GetRolloffFactor: Some(IDirectSound3DListenerImpl_GetRolloffFactor),
    GetVelocity: Some(IDirectSound3DListenerImpl_GetVelocity),
    SetAllParameters: Some(IDirectSound3DListenerImpl_SetAllParameters),
    SetDistanceFactor: Some(IDirectSound3DListenerImpl_SetDistanceFactor),
    SetDopplerFactor: Some(IDirectSound3DListenerImpl_SetDopplerFactor),
    SetOrientation: Some(IDirectSound3DListenerImpl_SetOrientation),
    SetPosition: Some(IDirectSound3DListenerImpl_SetPosition),
    SetRolloffFactor: Some(IDirectSound3DListenerImpl_SetRolloffFactor),
    SetVelocity: Some(IDirectSound3DListenerImpl_SetVelocity),
    CommitDeferredSettings: Some(IDirectSound3DListenerImpl_CommitDeferredSettings),
};
