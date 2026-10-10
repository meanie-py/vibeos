use win32k_sys::win32k::*;
use core::ffi::c_void;
// Per-arch FLOATOBJ shim (engine fns on i386 / f32 ops on amd64+arm64).
use crate::floatobj::*;

// Pure macros on every arch.
#[inline(always)]
fn Int32x32To64(a: i32, b: i32) -> i64 { (a as i64) * (b as i64) }
#[inline(always)]
fn FIX2LONG(x: i32) -> i32 { (x + 8) >> 4 }
#[inline(always)]
fn LONG2FIX(x: i32) -> i32 { x << 4 }
#[inline(always)]
unsafe fn RtlCopyMemory(dst: *mut c_void, src: *const c_void, len: usize) {
    core::ptr::copy_nonoverlapping(src as *const u8, dst as *mut u8, len);
}

// XFORMOBJ_pmx / XFORMOBJ_vInit are FORCEINLINE (xformobj.h) — no symbol. The
// win32k-internal XFORMOBJ is EXFORMOBJ {MATRIX *pmx} (xformobj.h #defines
// XFORMOBJ→EXFORMOBJ); bindgen's public `XFORMOBJ` is the opaque {ulReserved}
// DDI handle, so reinterpret the pointer as _EXFORMOBJ to reach `pmx`.
#[inline(always)]
unsafe fn XFORMOBJ_pmx(pxo: *mut XFORMOBJ) -> PMATRIX { (*(pxo as *mut _EXFORMOBJ)).pmx }
#[inline(always)]
unsafe fn XFORMOBJ_vInit(pxo: *mut XFORMOBJ, pmx: *mut MATRIX) { (*(pxo as *mut _EXFORMOBJ)).pmx = pmx; }

// ---- file-scope preamble (translated) ----
pub const NDEBUG: () = ();

#[inline(always)]

unsafe fn MulAdd(
    pfoDest: PFLOATOBJ,
    pfo1: PFLOATOBJ,
    pfo2: PFLOATOBJ,
    pfo3: PFLOATOBJ,
    pfo4: PFLOATOBJ,
) {

    let mut foTmp: FLOATOBJ;

    *pfoDest = *pfo1;
    FLOATOBJ_Mul(pfoDest, pfo2);
    foTmp = *pfo3;
    FLOATOBJ_Mul(&mut foTmp, pfo4);
    FLOATOBJ_Add(pfoDest, &mut foTmp);
}

unsafe fn MulAddLong(
    pfoDest: PFLOATOBJ,
    pfo1: PFLOATOBJ,
    l2: i32,
    pfo3: PFLOATOBJ,
    l4: i32,
) {

    let mut foTmp: FLOATOBJ = core::mem::zeroed();

    *pfoDest = *pfo1;
    FLOATOBJ_MulLong(pfoDest, l2);
    foTmp = *pfo3;
    FLOATOBJ_MulLong(&mut foTmp, l4);
    FLOATOBJ_Add(pfoDest, &mut foTmp);
}

unsafe fn MulSub(
    pfoDest: PFLOATOBJ,
    pfo1: PFLOATOBJ,
    pfo2: PFLOATOBJ,
    pfo3: PFLOATOBJ,
    pfo4: PFLOATOBJ,
) {
    let mut foTmp: FLOATOBJ = core::mem::zeroed();

    *pfoDest = *pfo1;
    FLOATOBJ_Mul(pfoDest, pfo2);
    foTmp = *pfo3;
    FLOATOBJ_Mul(&mut foTmp, pfo4);
    FLOATOBJ_Sub(pfoDest, &mut foTmp);
}

unsafe fn HintFromAccel(flAccel: u32) -> u32 {
    match flAccel & (XFORM_SCALE | XFORM_UNITY | XFORM_NO_TRANSLATION) {
        x if x == (XFORM_SCALE | XFORM_UNITY | XFORM_NO_TRANSLATION) => GX_IDENTITY,
        x if x == (XFORM_SCALE | XFORM_UNITY) => GX_OFFSET,
        XFORM_SCALE => GX_SCALE,
        _ => GX_GENERAL,
    }
}

unsafe fn MX_UpdateAccel(pmx: PMATRIX) -> u32 {

    (*pmx).fxDx = FLOATOBJ_GetFix(core::ptr::addr_of!((*pmx).efDx));
    (*pmx).fxDy = FLOATOBJ_GetFix(core::ptr::addr_of!((*pmx).efDy));

    (*pmx).flAccel = 0;

    if FLOATOBJ_Equal0(core::ptr::addr_of!((*pmx).efDx)) != 0 &&
       FLOATOBJ_Equal0(core::ptr::addr_of!((*pmx).efDy)) != 0
    {
        (*pmx).flAccel |= XFORM_NO_TRANSLATION;
    }

    if FLOATOBJ_Equal0(core::ptr::addr_of!((*pmx).efM12)) != 0 &&
       FLOATOBJ_Equal0(core::ptr::addr_of!((*pmx).efM21)) != 0
    {
        (*pmx).flAccel |= XFORM_SCALE;
    }

    if FLOATOBJ_Equal1(core::ptr::addr_of!((*pmx).efM11)) != 0 &&
       FLOATOBJ_Equal1(core::ptr::addr_of!((*pmx).efM22)) != 0
    {
        (*pmx).flAccel |= XFORM_UNITY;
    }

    if FLOATOBJ_IsLong(core::ptr::addr_of!((*pmx).efM11)) != 0 &&
       FLOATOBJ_IsLong(core::ptr::addr_of!((*pmx).efM12)) != 0 &&
       FLOATOBJ_IsLong(core::ptr::addr_of!((*pmx).efM21)) != 0 &&
       FLOATOBJ_IsLong(core::ptr::addr_of!((*pmx).efM22)) != 0
    {
        (*pmx).flAccel |= XFORM_INTEGER;
    }

    HintFromAccel((*pmx).flAccel)
}

#[no_mangle]
pub unsafe extern "system" fn XFORMOBJ_UpdateAccel(
    pxo: *mut XFORMOBJ
) -> u32 {
    
    let pmx = XFORMOBJ_pmx(pxo);
    
    MX_UpdateAccel(pmx)
}

#[no_mangle]
pub unsafe extern "system" fn EXFORMOBJ_iSetXform(
    pxo: *mut XFORMOBJ,
    pxform: *const XFORML,
) -> u32 {

    let pmx: PMATRIX;
    let mut mxTemp: MATRIX = core::mem::zeroed();
    let Hint: u32;

    if pxo.is_null() || pxform.is_null() {
        return DDI_ERROR;
    }

    FLOATOBJ_SetFloat(core::ptr::addr_of_mut!(mxTemp.efM11), (*pxform).eM11);
    FLOATOBJ_SetFloat(core::ptr::addr_of_mut!(mxTemp.efM12), (*pxform).eM12);
    FLOATOBJ_SetFloat(core::ptr::addr_of_mut!(mxTemp.efM21), (*pxform).eM21);
    FLOATOBJ_SetFloat(core::ptr::addr_of_mut!(mxTemp.efM22), (*pxform).eM22);
    FLOATOBJ_SetFloat(core::ptr::addr_of_mut!(mxTemp.efDx), (*pxform).eDx);
    FLOATOBJ_SetFloat(core::ptr::addr_of_mut!(mxTemp.efDy), (*pxform).eDy);

    Hint = MX_UpdateAccel(core::ptr::addr_of_mut!(mxTemp));

    if Hint == GX_SCALE {
        if FLOATOBJ_Equal0(core::ptr::addr_of!(mxTemp.efM11)) != 0 ||
           FLOATOBJ_Equal0(core::ptr::addr_of!(mxTemp.efM22)) != 0
        {
            return DDI_ERROR;
        }
    } else if Hint == GX_GENERAL {
        if MX_IsInvertible(core::ptr::addr_of_mut!(mxTemp)) == 0 {
            return DDI_ERROR;
        }
    }

    pmx = XFORMOBJ_pmx(pxo);
    *pmx = mxTemp;

    Hint
}

#[no_mangle]
pub unsafe extern "system" fn EXFORMOBJ_iCombine(
    pxo: *mut XFORMOBJ,
    pxo1: *mut XFORMOBJ,
    pxo2: *mut XFORMOBJ,
) -> u32 {

    let mut mx: MATRIX = core::mem::zeroed();
    let pmx: PMATRIX;
    let pmx1: PMATRIX;
    let pmx2: PMATRIX;

    pmx1 = XFORMOBJ_pmx(pxo1);
    pmx2 = XFORMOBJ_pmx(pxo2);

    MulAdd(
        core::ptr::addr_of_mut!(mx.efM11),
        core::ptr::addr_of_mut!((*pmx1).efM11),
        core::ptr::addr_of_mut!((*pmx2).efM11),
        core::ptr::addr_of_mut!((*pmx1).efM12),
        core::ptr::addr_of_mut!((*pmx2).efM21),
    );
    MulAdd(
        core::ptr::addr_of_mut!(mx.efM12),
        core::ptr::addr_of_mut!((*pmx1).efM11),
        core::ptr::addr_of_mut!((*pmx2).efM12),
        core::ptr::addr_of_mut!((*pmx1).efM12),
        core::ptr::addr_of_mut!((*pmx2).efM22),
    );
    MulAdd(
        core::ptr::addr_of_mut!(mx.efM21),
        core::ptr::addr_of_mut!((*pmx1).efM21),
        core::ptr::addr_of_mut!((*pmx2).efM11),
        core::ptr::addr_of_mut!((*pmx1).efM22),
        core::ptr::addr_of_mut!((*pmx2).efM21),
    );
    MulAdd(
        core::ptr::addr_of_mut!(mx.efM22),
        core::ptr::addr_of_mut!((*pmx1).efM21),
        core::ptr::addr_of_mut!((*pmx2).efM12),
        core::ptr::addr_of_mut!((*pmx1).efM22),
        core::ptr::addr_of_mut!((*pmx2).efM22),
    );
    MulAdd(
        core::ptr::addr_of_mut!(mx.efDx),
        core::ptr::addr_of_mut!((*pmx1).efDx),
        core::ptr::addr_of_mut!((*pmx2).efM11),
        core::ptr::addr_of_mut!((*pmx1).efDy),
        core::ptr::addr_of_mut!((*pmx2).efM21),
    );
    FLOATOBJ_Add(
        core::ptr::addr_of_mut!(mx.efDx),
        core::ptr::addr_of_mut!((*pmx2).efDx),
    );
    MulAdd(
        core::ptr::addr_of_mut!(mx.efDy),
        core::ptr::addr_of_mut!((*pmx1).efDx),
        core::ptr::addr_of_mut!((*pmx2).efM12),
        core::ptr::addr_of_mut!((*pmx1).efDy),
        core::ptr::addr_of_mut!((*pmx2).efM22),
    );
    FLOATOBJ_Add(
        core::ptr::addr_of_mut!(mx.efDy),
        core::ptr::addr_of_mut!((*pmx2).efDy),
    );

    pmx = XFORMOBJ_pmx(pxo);
    *pmx = mx;

    XFORMOBJ_UpdateAccel(pxo)
}

#[no_mangle]
pub unsafe extern "system" fn EXFORMOBJ_iCombineXform(
    pxo: *mut XFORMOBJ,
    pxo1: *mut XFORMOBJ,
    pxform: *const XFORML,
    bLeftMultiply: i32,
) -> u32 {

    let mut mx: MATRIX = core::mem::zeroed();
    let mut xo2: EXFORMOBJ = core::mem::zeroed(); // 8B on 64-bit: holds pmx (XFORMOBJ is 4B → vInit OOB)

    XFORMOBJ_vInit(core::ptr::addr_of_mut!(xo2) as *mut XFORMOBJ , core::ptr::addr_of_mut!(mx));
    EXFORMOBJ_iSetXform(core::ptr::addr_of_mut!(xo2) as *mut XFORMOBJ , pxform);

    if bLeftMultiply != 0 {
        EXFORMOBJ_iCombine(pxo, core::ptr::addr_of_mut!(xo2) as *mut XFORMOBJ , pxo1)
    } else {
        EXFORMOBJ_iCombine(pxo, pxo1, core::ptr::addr_of_mut!(xo2) as *mut XFORMOBJ )
    }
}

#[cfg(target_pointer_width = "32")]
#[inline(always)]
#[no_mangle]
pub unsafe extern "fastcall" fn MX_IsInvertible(pmx: PMATRIX) -> BOOL {
    let mut foDet: FLOATOBJ = core::mem::zeroed();
    MulSub(
        &mut foDet,
        &mut (*pmx).efM11,
        &mut (*pmx).efM22,
        &mut (*pmx).efM12,
        &mut (*pmx).efM21,
    );
    if FLOATOBJ_Equal0(&mut foDet) != 0 {
        0
    } else {
        1
    }
}

#[cfg(target_pointer_width = "64")]
#[inline(always)]
#[no_mangle]
pub unsafe extern "C" fn MX_IsInvertible(pmx: PMATRIX) -> BOOL {
    let mut foDet: FLOATOBJ = core::mem::zeroed();
    MulSub(
        &mut foDet,
        &mut (*pmx).efM11,
        &mut (*pmx).efM22,
        &mut (*pmx).efM12,
        &mut (*pmx).efM21,
    );
    if FLOATOBJ_Equal0(&mut foDet) != 0 {
        0
    } else {
        1
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn MX_Set0(pmx: PMATRIX) {
    
    FLOATOBJ_Set0(core::ptr::addr_of_mut!((*pmx).efM11) as *mut core::ffi::c_void);
    FLOATOBJ_Set0(core::ptr::addr_of_mut!((*pmx).efM12) as *mut core::ffi::c_void);
    FLOATOBJ_Set0(core::ptr::addr_of_mut!((*pmx).efM21) as *mut core::ffi::c_void);
    FLOATOBJ_Set0(core::ptr::addr_of_mut!((*pmx).efM22) as *mut core::ffi::c_void);
    FLOATOBJ_Set0(core::ptr::addr_of_mut!((*pmx).efDx) as *mut core::ffi::c_void);
    FLOATOBJ_Set0(core::ptr::addr_of_mut!((*pmx).efDy) as *mut core::ffi::c_void);
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn MX_Set0(pmx: PMATRIX) {
    
    FLOATOBJ_Set0(core::ptr::addr_of_mut!((*pmx).efM11) as *mut core::ffi::c_void);
    FLOATOBJ_Set0(core::ptr::addr_of_mut!((*pmx).efM12) as *mut core::ffi::c_void);
    FLOATOBJ_Set0(core::ptr::addr_of_mut!((*pmx).efM21) as *mut core::ffi::c_void);
    FLOATOBJ_Set0(core::ptr::addr_of_mut!((*pmx).efM22) as *mut core::ffi::c_void);
    FLOATOBJ_Set0(core::ptr::addr_of_mut!((*pmx).efDx) as *mut core::ffi::c_void);
    FLOATOBJ_Set0(core::ptr::addr_of_mut!((*pmx).efDy) as *mut core::ffi::c_void);
}

#[no_mangle]
pub unsafe extern "system" fn EXFORMOBJ_iInverse(
    pxoDst: *mut XFORMOBJ,
    pxoSrc: *mut XFORMOBJ,
) -> u32 {

    let pmxDst: PMATRIX;
    let pmxSrc: PMATRIX;
    let mut foDet: FLOATOBJ = core::mem::zeroed();
    let mut xformSrc: XFORM = core::mem::zeroed();

    pmxDst = XFORMOBJ_pmx(pxoDst);
    pmxSrc = XFORMOBJ_pmx(pxoSrc);

    EXFORMOBJ_iGetXform(pxoSrc, &mut xformSrc as *mut XFORM as *mut XFORML);

    MulSub(&mut foDet, core::ptr::addr_of_mut!((*pmxSrc).efM11), core::ptr::addr_of_mut!((*pmxSrc).efM22), core::ptr::addr_of_mut!((*pmxSrc).efM12), core::ptr::addr_of_mut!((*pmxSrc).efM21));

    if FLOATOBJ_Equal0(&foDet) != 0 {
        return DDI_ERROR;
    }

    (*pmxDst).efM11 = (*pmxSrc).efM22;
    FLOATOBJ_Div(&mut (*pmxDst).efM11, &mut foDet);
    (*pmxDst).efM22 = (*pmxSrc).efM11;
    FLOATOBJ_Div(&mut (*pmxDst).efM22, &mut foDet);

    FLOATOBJ_Neg(&mut foDet);
    (*pmxDst).efM12 = (*pmxSrc).efM12;
    FLOATOBJ_Div(&mut (*pmxDst).efM12, &mut foDet);
    (*pmxDst).efM21 = (*pmxSrc).efM21;
    FLOATOBJ_Div(&mut (*pmxDst).efM21, &mut foDet);

    (*pmxDst).efDx = (*pmxSrc).efDx;
    FLOATOBJ_Neg(&mut (*pmxDst).efDx);
    MulSub(&mut (*pmxDst).efDx, core::ptr::addr_of_mut!((*pmxDst).efDx), core::ptr::addr_of_mut!((*pmxDst).efM11), core::ptr::addr_of_mut!((*pmxSrc).efDy), core::ptr::addr_of_mut!((*pmxDst).efM21));

    (*pmxDst).efDy = (*pmxSrc).efDy;
    FLOATOBJ_Neg(&mut (*pmxDst).efDy);
    MulSub(&mut (*pmxDst).efDy, core::ptr::addr_of_mut!((*pmxDst).efDy), core::ptr::addr_of_mut!((*pmxDst).efM22), core::ptr::addr_of_mut!((*pmxSrc).efDx), core::ptr::addr_of_mut!((*pmxDst).efM12));

    XFORMOBJ_UpdateAccel(pxoDst)
}

unsafe fn XFORMOBJ_bXformFixPoints(
    pxo: *mut XFORMOBJ,
    cPoints: u32,
    pptIn: PPOINTL,
    pptOut: PPOINTL,
) -> i32 {

    let pmx: PMATRIX;
    let mut i: i32;
    let mut fo1: FLOATOBJ = core::mem::zeroed();
    let mut fo2: FLOATOBJ = core::mem::zeroed();
    let flAccel: FLONG;
    let mut lM11: i32 = 0;
    let mut lM12: i32 = 0;
    let mut lM21: i32 = 0;
    let mut lM22: i32 = 0;
    let mut lTemp: i32 = 0;
    let mut llx: i64;
    let mut lly: i64;

    pmx = XFORMOBJ_pmx(pxo);
    flAccel = (*pmx).flAccel;

    if (flAccel & (XFORM_SCALE | XFORM_UNITY)) == (XFORM_SCALE | XFORM_UNITY) {
        RtlCopyMemory(
            pptOut as *mut core::ffi::c_void,
            pptIn as *const core::ffi::c_void,
            (cPoints as usize) * core::mem::size_of::<POINTL>(),
        );
    } else if (flAccel & XFORM_INTEGER) != 0 {
        if (flAccel & XFORM_UNITY) != 0 {
            if FLOATOBJ_bConvertToLong(core::ptr::addr_of!((*pmx).efM12), &mut lM12) == 0
                || FLOATOBJ_bConvertToLong(core::ptr::addr_of!((*pmx).efM21), &mut lM21) == 0
            {
                NT_ASSERT!(0);
                return 0;
            }

            i = cPoints as i32 - 1;
            loop {
                llx = Int32x32To64((*pptIn.offset(i as isize)).y, lM21) + (*pptIn.offset(i as isize)).x as i64;
                if (llx as i32 as i64) != llx {
                    return 0;
                }

                lly = Int32x32To64((*pptIn.offset(i as isize)).x, lM12) + (*pptIn.offset(i as isize)).y as i64;
                if (lly as i32 as i64) != lly {
                    return 0;
                }

                (*pptOut.offset(i as isize)).x = llx as i32;
                (*pptOut.offset(i as isize)).y = lly as i32;

                i -= 1;
                if i < 0 {
                    break;
                }
            }
        } else if (flAccel & XFORM_SCALE) != 0 {
            if FLOATOBJ_bConvertToLong(core::ptr::addr_of!((*pmx).efM11), &mut lM11) == 0
                || FLOATOBJ_bConvertToLong(core::ptr::addr_of!((*pmx).efM22), &mut lM22) == 0
            {
                NT_ASSERT!(0);
                return 0;
            }

            i = cPoints as i32 - 1;
            loop {
                llx = Int32x32To64((*pptIn.offset(i as isize)).x, lM11);
                if (llx as i32 as i64) != llx {
                    return 0;
                }

                lly = Int32x32To64((*pptIn.offset(i as isize)).y, lM22);
                if (lly as i32 as i64) != lly {
                    return 0;
                }

                (*pptOut.offset(i as isize)).x = llx as i32;
                (*pptOut.offset(i as isize)).y = lly as i32;

                i -= 1;
                if i < 0 {
                    break;
                }
            }
        } else {
            if FLOATOBJ_bConvertToLong(core::ptr::addr_of!((*pmx).efM11), &mut lM11) == 0
                || FLOATOBJ_bConvertToLong(core::ptr::addr_of!((*pmx).efM12), &mut lM12) == 0
                || FLOATOBJ_bConvertToLong(core::ptr::addr_of!((*pmx).efM21), &mut lM21) == 0
                || FLOATOBJ_bConvertToLong(core::ptr::addr_of!((*pmx).efM22), &mut lM22) == 0
            {
                NT_ASSERT!(0);
                return 0;
            }

            i = cPoints as i32 - 1;
            loop {
                llx = Int32x32To64((*pptIn.offset(i as isize)).x, lM11);
                llx += Int32x32To64((*pptIn.offset(i as isize)).y, lM21);
                if (llx as i32 as i64) != llx {
                    return 0;
                }

                lly = Int32x32To64((*pptIn.offset(i as isize)).y, lM22);
                lly += Int32x32To64((*pptIn.offset(i as isize)).x, lM12);
                if (lly as i32 as i64) != lly {
                    return 0;
                }

                (*pptOut.offset(i as isize)).x = llx as i32;
                (*pptOut.offset(i as isize)).y = lly as i32;

                i -= 1;
                if i < 0 {
                    break;
                }
            }
        }
    } else if (flAccel & XFORM_UNITY) != 0 {
        i = cPoints as i32 - 1;
        loop {
            fo1 = (*pmx).efM21;
            FLOATOBJ_MulLong(&mut fo1, (*pptIn.offset(i as isize)).y);
            if FLOATOBJ_bConvertToLong(&fo1, &mut lTemp) == 0 {
                return 0;
            }
            llx = (*pptIn.offset(i as isize)).x as i64 + lTemp as i64;
            if (llx as i32 as i64) != llx {
                return 0;
            }

            fo2 = (*pmx).efM12;
            FLOATOBJ_MulLong(&mut fo2, (*pptIn.offset(i as isize)).x);
            if FLOATOBJ_bConvertToLong(&fo2, &mut lTemp) == 0 {
                return 0;
            }
            lly = (*pptIn.offset(i as isize)).y as i64 + lTemp as i64;
            if (lly as i32 as i64) != lly {
                return 0;
            }

            (*pptOut.offset(i as isize)).x = llx as i32;
            (*pptOut.offset(i as isize)).y = lly as i32;

            i -= 1;
            if i < 0 {
                break;
            }
        }
    } else if (flAccel & XFORM_SCALE) != 0 {
        i = cPoints as i32 - 1;
        loop {
            fo1 = (*pmx).efM11;
            FLOATOBJ_MulLong(&mut fo1, (*pptIn.offset(i as isize)).x);
            if FLOATOBJ_bConvertToLong(&fo1, &mut (*pptOut.offset(i as isize)).x) == 0 {
                return 0;
            }

            fo2 = (*pmx).efM22;
            FLOATOBJ_MulLong(&mut fo2, (*pptIn.offset(i as isize)).y);
            if FLOATOBJ_bConvertToLong(&fo2, &mut (*pptOut.offset(i as isize)).y) == 0 {
                return 0;
            }

            i -= 1;
            if i < 0 {
                break;
            }
        }
    } else {
        i = cPoints as i32 - 1;
        loop {
            MulAddLong(
                &mut fo1,
                core::ptr::addr_of_mut!((*pmx).efM11),
                (*pptIn.offset(i as isize)).x,
                core::ptr::addr_of_mut!((*pmx).efM21),
                (*pptIn.offset(i as isize)).y,
            );

            MulAddLong(
                &mut fo2,
                core::ptr::addr_of_mut!((*pmx).efM12),
                (*pptIn.offset(i as isize)).x,
                core::ptr::addr_of_mut!((*pmx).efM22),
                (*pptIn.offset(i as isize)).y,
            );

            if FLOATOBJ_bConvertToLong(&fo1, &mut (*pptOut.offset(i as isize)).x) == 0 {
                return 0;
            }

            if FLOATOBJ_bConvertToLong(&fo2, &mut (*pptOut.offset(i as isize)).y) == 0 {
                return 0;
            }

            i -= 1;
            if i < 0 {
                break;
            }
        }
    }

    if ((*pmx).flAccel & XFORM_NO_TRANSLATION) == 0 {
        i = cPoints as i32 - 1;
        loop {
            llx = (*pptOut.offset(i as isize)).x as i64 + (*pmx).fxDx as i64;
            if (llx as i32 as i64) != llx {
                return 0;
            }
            (*pptOut.offset(i as isize)).x = llx as i32;

            lly = (*pptOut.offset(i as isize)).y as i64 + (*pmx).fxDy as i64;
            if (lly as i32 as i64) != lly {
                return 0;
            }
            (*pptOut.offset(i as isize)).y = lly as i32;

            i -= 1;
            if i < 0 {
                break;
            }
        }
    }

    1
}
#[no_mangle]
pub unsafe extern "system" fn EXFORMOBJ_iGetXform(
    pxo: *mut XFORMOBJ,
    pxform: *mut XFORML,
) -> u32 {

    let pmx = XFORMOBJ_pmx(pxo);

    if pxo.is_null() || pxform.is_null() {
        return DDI_ERROR;
    }

    (*pxform).eM11 = FLOATOBJ_GetFloat(core::ptr::addr_of!((*pmx).efM11) as *const _);
    (*pxform).eM12 = FLOATOBJ_GetFloat(core::ptr::addr_of!((*pmx).efM12) as *const _);
    (*pxform).eM21 = FLOATOBJ_GetFloat(core::ptr::addr_of!((*pmx).efM21) as *const _);
    (*pxform).eM22 = FLOATOBJ_GetFloat(core::ptr::addr_of!((*pmx).efM22) as *const _);
    (*pxform).eDx = FLOATOBJ_GetFloat(core::ptr::addr_of!((*pmx).efDx) as *const _);
    (*pxform).eDy = FLOATOBJ_GetFloat(core::ptr::addr_of!((*pmx).efDy) as *const _);

    HintFromAccel((*pmx).flAccel)
}
#[no_mangle]
pub unsafe extern "system" fn EXFORMOBJ_iGetFloatObjXform(
    pxo: *mut XFORMOBJ,
    pxfo: *mut FLOATOBJ_XFORM,
) -> u32 {
    let pmx: PMATRIX = XFORMOBJ_pmx(pxo);

    if pxo.is_null() || pxfo.is_null() {
        return DDI_ERROR;
    }

    (*pxfo).eM11 = (*pmx).efM11;
    (*pxfo).eM12 = (*pmx).efM12;
    (*pxfo).eM21 = (*pmx).efM21;
    (*pxfo).eM22 = (*pmx).efM22;
    (*pxfo).eDx = (*pmx).efDx;
    (*pxfo).eDy = (*pmx).efDy;

    HintFromAccel((*pmx).flAccel)
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn EXFORMOBJ_bApplyXform(
    pxo: *mut XFORMOBJ,
    iMode: u32,
    cPoints: u32,
    pvIn: *mut core::ffi::c_void,
    pvOut: *mut core::ffi::c_void,
) -> i32 {

    let mut mx: MATRIX = core::mem::zeroed();
    let mut xoInv: EXFORMOBJ = core::mem::zeroed(); // 8B on 64-bit: holds pmx (XFORMOBJ is 4B → vInit OOB)
    let mut pptlIn: PPOINTL;
    let mut pptlOut: PPOINTL;
    let mut i: i32;
    let mut pxo_local = pxo;
    let mut pvIn_local = pvIn;

    if pxo.is_null() || pvIn.is_null() || pvOut.is_null() || cPoints < 1 {
        return 0;
    }

    if iMode == XF_INV_FXTOL || iMode == XF_INV_LTOL {
        XFORMOBJ_vInit((core::ptr::addr_of_mut!(xoInv) as *mut XFORMOBJ), core::ptr::addr_of_mut!(mx));
        if EXFORMOBJ_iInverse((core::ptr::addr_of_mut!(xoInv) as *mut XFORMOBJ), pxo) == DDI_ERROR {
            return 0;
        }
        pxo_local = (core::ptr::addr_of_mut!(xoInv) as *mut XFORMOBJ);
    }

    if iMode == XF_LTOFX || iMode == XF_LTOL || iMode == XF_INV_LTOL {
        pptlIn = pvIn as PPOINTL;
        pptlOut = pvOut as PPOINTL;
        i = cPoints as i32 - 1;
        while i >= 0 {
            (*pptlOut.offset(i as isize)).x = LONG2FIX((*pptlIn.offset(i as isize)).x);
            (*pptlOut.offset(i as isize)).y = LONG2FIX((*pptlIn.offset(i as isize)).y);
            i -= 1;
        }

        pvIn_local = pvOut;
    }

    if XFORMOBJ_bXformFixPoints(pxo_local, cPoints, pvIn_local as PPOINTL, pvOut as PPOINTL) == 0 {
        return 0;
    }

    if iMode == XF_INV_FXTOL || iMode == XF_INV_LTOL || iMode == XF_LTOL {
        pptlOut = pvOut as PPOINTL;
        i = cPoints as i32 - 1;
        while i >= 0 {
            (*pptlOut.offset(i as isize)).x = FIX2LONG((*pptlOut.offset(i as isize)).x);
            (*pptlOut.offset(i as isize)).y = FIX2LONG((*pptlOut.offset(i as isize)).y);
            i -= 1;
        }
    }

    return 1;
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn EXFORMOBJ_bApplyXform(
    pxo: *mut XFORMOBJ,
    iMode: u32,
    cPoints: u32,
    pvIn: *mut core::ffi::c_void,
    pvOut: *mut core::ffi::c_void,
) -> i32 {

    let mut mx: MATRIX = core::mem::zeroed();
    let mut xoInv: EXFORMOBJ = core::mem::zeroed(); // 8B on 64-bit: holds pmx (XFORMOBJ is 4B → vInit OOB)
    let mut pptlIn: PPOINTL;
    let mut pptlOut: PPOINTL;
    let mut i: i32;
    let mut pxo_local = pxo;
    let mut pvIn_local = pvIn;

    if pxo.is_null() || pvIn.is_null() || pvOut.is_null() || cPoints < 1 {
        return 0;
    }

    if iMode == XF_INV_FXTOL || iMode == XF_INV_LTOL {
        XFORMOBJ_vInit(core::ptr::addr_of_mut!(xoInv) as *mut XFORMOBJ , core::ptr::addr_of_mut!(mx));
        if EXFORMOBJ_iInverse(core::ptr::addr_of_mut!(xoInv) as *mut XFORMOBJ , pxo) == DDI_ERROR {
            return 0;
        }
        pxo_local = core::ptr::addr_of_mut!(xoInv) as *mut XFORMOBJ ;
    }

    if iMode == XF_LTOFX || iMode == XF_LTOL || iMode == XF_INV_LTOL {
        pptlIn = pvIn as PPOINTL;
        pptlOut = pvOut as PPOINTL;
        i = cPoints as i32 - 1;
        while i >= 0 {
            (*pptlOut.offset(i as isize)).x = LONG2FIX((*pptlIn.offset(i as isize)).x);
            (*pptlOut.offset(i as isize)).y = LONG2FIX((*pptlIn.offset(i as isize)).y);
            i -= 1;
        }

        pvIn_local = pvOut;
    }

    if XFORMOBJ_bXformFixPoints(pxo_local, cPoints, pvIn_local as PPOINTL, pvOut as PPOINTL) == 0 {
        return 0;
    }

    if iMode == XF_INV_FXTOL || iMode == XF_INV_LTOL || iMode == XF_LTOL {
        pptlOut = pvOut as PPOINTL;
        i = cPoints as i32 - 1;
        while i >= 0 {
            (*pptlOut.offset(i as isize)).x = FIX2LONG((*pptlOut.offset(i as isize)).x);
            (*pptlOut.offset(i as isize)).y = FIX2LONG((*pptlOut.offset(i as isize)).y);
            i -= 1;
        }
    }

    return 1;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS win32 kernel mode subsystem
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            win32ss/gdi/ntgdi/xformobj.c
//  * PURPOSE:         XFORMOBJ API
//  * PROGRAMMERS:     Timo Kreuzer
//  *                  Katayama Hirofumi MZ
//  */
//
// /** Includes ******************************************************************/
//
// #include <win32k.h>
// #define NDEBUG
// #include <debug.h>
//
// #define DOES_VALUE_OVERFLOW_LONG(x) \
//     (((__int64)((long)(x))) != (x))
//
// /** Inline helper functions ***************************************************/
//
// /*
//  * Inline helper to calculate pfo1 * pfo2 + pfo3 * pfo4
//  */
// FORCEINLINE
// VOID
// MulAdd(
//     PFLOATOBJ pfoDest,
//     PFLOATOBJ pfo1,
//     PFLOATOBJ pfo2,
//     PFLOATOBJ pfo3,
//     PFLOATOBJ pfo4)
// {
//     FLOATOBJ foTmp;
//
//     *pfoDest = *pfo1;
//     FLOATOBJ_Mul(pfoDest, pfo2);
//     foTmp = *pfo3;
//     FLOATOBJ_Mul(&foTmp, pfo4);
//     FLOATOBJ_Add(pfoDest, &foTmp);
// }
//
// /*
//  * Inline helper to calculate pfo1 * l2 + pfo3 * l4
//  */
// FORCEINLINE
// VOID
// MulAddLong(
//     PFLOATOBJ pfoDest,
//     PFLOATOBJ pfo1,
//     LONG l2,
//     PFLOATOBJ pfo3,
//     LONG l4)
// {
//     FLOATOBJ foTmp;
//
//     *pfoDest = *pfo1;
//     FLOATOBJ_MulLong(pfoDest, l2);
//     foTmp = *pfo3;
//     FLOATOBJ_MulLong(&foTmp, l4);
//     FLOATOBJ_Add(pfoDest, &foTmp);
// }
//
// /*
//  * Inline helper to calculate pfo1 * pfo2 - pfo3 * pfo4
//  */
// FORCEINLINE
// VOID
// MulSub(
//     PFLOATOBJ pfoDest,
//     PFLOATOBJ pfo1,
//     PFLOATOBJ pfo2,
//     PFLOATOBJ pfo3,
//     PFLOATOBJ pfo4)
// {
//     FLOATOBJ foTmp;
//
//     *pfoDest = *pfo1;
//     FLOATOBJ_Mul(pfoDest, pfo2);
//     foTmp = *pfo3;
//     FLOATOBJ_Mul(&foTmp, pfo4);
//     FLOATOBJ_Sub(pfoDest, &foTmp);
// }
//
// /*
//  * Inline helper to get the complexity hint from flAccel
//  */
// FORCEINLINE
// ULONG
// HintFromAccel(ULONG flAccel)
// {
//     switch (flAccel & (XFORM_SCALE|XFORM_UNITY|XFORM_NO_TRANSLATION))
//     {
//         case (XFORM_SCALE|XFORM_UNITY|XFORM_NO_TRANSLATION):
//             return GX_IDENTITY;
//         case (XFORM_SCALE|XFORM_UNITY):
//             return GX_OFFSET;
//         case XFORM_SCALE:
//             return GX_SCALE;
//         default:
//             return GX_GENERAL;
//     }
// }
//
// /** Internal functions ********************************************************/
//
// ULONG
// FASTCALL
// MX_UpdateAccel(IN OUT PMATRIX pmx)
// {
//     /* Copy Dx and Dy to FIX format */
//     pmx->fxDx = FLOATOBJ_GetFix(&pmx->efDx);
//     pmx->fxDy = FLOATOBJ_GetFix(&pmx->efDy);
//
//     pmx->flAccel = 0;
//
//     if (FLOATOBJ_Equal0(&pmx->efDx) &&
//         FLOATOBJ_Equal0(&pmx->efDy))
//     {
//         pmx->flAccel |= XFORM_NO_TRANSLATION;
//     }
//
//     if (FLOATOBJ_Equal0(&pmx->efM12) &&
//         FLOATOBJ_Equal0(&pmx->efM21))
//     {
//         pmx->flAccel |= XFORM_SCALE;
//     }
//
//     if (FLOATOBJ_Equal1(&pmx->efM11) &&
//         FLOATOBJ_Equal1(&pmx->efM22))
//     {
//         pmx->flAccel |= XFORM_UNITY;
//     }
//
//     if (FLOATOBJ_IsLong(&pmx->efM11) && FLOATOBJ_IsLong(&pmx->efM12) &&
//         FLOATOBJ_IsLong(&pmx->efM21) && FLOATOBJ_IsLong(&pmx->efM22))
//     {
//         pmx->flAccel |= XFORM_INTEGER;
//     }
//
//     return HintFromAccel(pmx->flAccel);
// }
//
// ULONG
// NTAPI
// XFORMOBJ_UpdateAccel(
//     IN OUT XFORMOBJ *pxo)
// {
//     PMATRIX pmx = XFORMOBJ_pmx(pxo);
//
//     return MX_UpdateAccel(pmx);
// }
//
//
// ULONG
// NTAPI
// XFORMOBJ_iSetXform(
//     IN OUT XFORMOBJ *pxo,
//     IN const XFORML *pxform)
// {
//     PMATRIX pmx;
//     MATRIX mxTemp;
//     ULONG Hint;
//
//     /* Check parameters */
//     if (!pxo || !pxform) return DDI_ERROR;
//
//     /* Copy members */
//     FLOATOBJ_SetFloat(&mxTemp.efM11, pxform->eM11);
//     FLOATOBJ_SetFloat(&mxTemp.efM12, pxform->eM12);
//     FLOATOBJ_SetFloat(&mxTemp.efM21, pxform->eM21);
//     FLOATOBJ_SetFloat(&mxTemp.efM22, pxform->eM22);
//     FLOATOBJ_SetFloat(&mxTemp.efDx, pxform->eDx);
//     FLOATOBJ_SetFloat(&mxTemp.efDy, pxform->eDy);
//
//     /* Update accelerators and return complexity */
//     Hint = MX_UpdateAccel(&mxTemp);
//
//     /* Check whether det = (M11 * M22 - M12 * M21) is non-zero */
//     if (Hint == GX_SCALE)
//     {
//         if (FLOATOBJ_Equal0(&mxTemp.efM11) || FLOATOBJ_Equal0(&mxTemp.efM22))
//         {
//             return DDI_ERROR;
//         }
//     }
//     else if (Hint == GX_GENERAL)
//     {
//         if (!MX_IsInvertible(&mxTemp))
//         {
//             return DDI_ERROR;
//         }
//     }
//
//     /* Store */
//     pmx = XFORMOBJ_pmx(pxo);
//     *pmx = mxTemp;
//
//     return Hint;
// }
//
//
// /*
//  * Multiplies pxo1 with pxo2 and stores the result in pxo.
//  * returns complexity hint
//  * | efM11 efM12 0 |
//  * | efM21 efM22 0 |
//  * | efDx  efDy  1 |
//  */
// ULONG
// NTAPI
// XFORMOBJ_iCombine(
//     IN OUT XFORMOBJ *pxo,
//     IN XFORMOBJ *pxo1,
//     IN XFORMOBJ *pxo2)
// {
//     MATRIX mx;
//     PMATRIX pmx, pmx1, pmx2;
//
//     /* Get the source matrices */
//     pmx1 = XFORMOBJ_pmx(pxo1);
//     pmx2 = XFORMOBJ_pmx(pxo2);
//
//     /* Do a 3 x 3 matrix multiplication with mx as destinantion */
//     MulAdd(&mx.efM11, &pmx1->efM11, &pmx2->efM11, &pmx1->efM12, &pmx2->efM21);
//     MulAdd(&mx.efM12, &pmx1->efM11, &pmx2->efM12, &pmx1->efM12, &pmx2->efM22);
//     MulAdd(&mx.efM21, &pmx1->efM21, &pmx2->efM11, &pmx1->efM22, &pmx2->efM21);
//     MulAdd(&mx.efM22, &pmx1->efM21, &pmx2->efM12, &pmx1->efM22, &pmx2->efM22);
//     MulAdd(&mx.efDx, &pmx1->efDx, &pmx2->efM11, &pmx1->efDy, &pmx2->efM21);
//     FLOATOBJ_Add(&mx.efDx, &pmx2->efDx);
//     MulAdd(&mx.efDy, &pmx1->efDx, &pmx2->efM12, &pmx1->efDy, &pmx2->efM22);
//     FLOATOBJ_Add(&mx.efDy, &pmx2->efDy);
//
//     /* Copy back */
//     pmx = XFORMOBJ_pmx(pxo);
//     *pmx = mx;
//
//     /* Update accelerators and return complexity */
//     return XFORMOBJ_UpdateAccel(pxo);
// }
//
//
// ULONG
// NTAPI
// XFORMOBJ_iCombineXform(
//     IN OUT XFORMOBJ *pxo,
//     IN XFORMOBJ *pxo1,
//     IN XFORML *pxform,
//     IN BOOL bLeftMultiply)
// {
//     MATRIX mx;
//     XFORMOBJ xo2;
//
//     XFORMOBJ_vInit(&xo2, &mx);
//     XFORMOBJ_iSetXform(&xo2, pxform);
//
//     if (bLeftMultiply)
//     {
//         return XFORMOBJ_iCombine(pxo, &xo2, pxo1);
//     }
//     else
//     {
//         return XFORMOBJ_iCombine(pxo, pxo1, &xo2);
//     }
// }
//
// BOOL FASTCALL
// MX_IsInvertible(IN PMATRIX pmx)
// {
//     FLOATOBJ foDet;
//     MulSub(&foDet, &pmx->efM11, &pmx->efM22, &pmx->efM12, &pmx->efM21);
//     return !FLOATOBJ_Equal0(&foDet);
// }
//
// VOID FASTCALL
// MX_Set0(OUT PMATRIX pmx)
// {
//     FLOATOBJ_Set0(&pmx->efM11);
//     FLOATOBJ_Set0(&pmx->efM12);
//     FLOATOBJ_Set0(&pmx->efM21);
//     FLOATOBJ_Set0(&pmx->efM22);
//     FLOATOBJ_Set0(&pmx->efDx);
//     FLOATOBJ_Set0(&pmx->efDy);
// }
//
// /*
//  * A^-1 = adj(A) / det(AT)
//  * A^-1 = 1/(a*d - b*c) * (a22,-a12,a21,-a11)
//  */
// ULONG
// NTAPI
// XFORMOBJ_iInverse(
//     OUT XFORMOBJ *pxoDst,
//     IN XFORMOBJ *pxoSrc)
// {
//     PMATRIX pmxDst, pmxSrc;
//     FLOATOBJ foDet;
//     XFORM xformSrc;
//
//     pmxDst = XFORMOBJ_pmx(pxoDst);
//     pmxSrc = XFORMOBJ_pmx(pxoSrc);
//
//     XFORMOBJ_iGetXform(pxoSrc, (XFORML*)&xformSrc);
//
//     /* det = M11 * M22 - M12 * M21 */
//     MulSub(&foDet, &pmxSrc->efM11, &pmxSrc->efM22, &pmxSrc->efM12, &pmxSrc->efM21);
//
//     if (FLOATOBJ_Equal0(&foDet))
//     {
//         /* Determinant is 0! */
//         return DDI_ERROR;
//     }
//
//     /* Calculate adj(A) / det(A) */
//     pmxDst->efM11 = pmxSrc->efM22;
//     FLOATOBJ_Div(&pmxDst->efM11, &foDet);
//     pmxDst->efM22 = pmxSrc->efM11;
//     FLOATOBJ_Div(&pmxDst->efM22, &foDet);
//
//     /* The other 2 are negative, negate foDet for that */
//     FLOATOBJ_Neg(&foDet);
//     pmxDst->efM12 = pmxSrc->efM12;
//     FLOATOBJ_Div(&pmxDst->efM12, &foDet);
//     pmxDst->efM21 = pmxSrc->efM21;
//     FLOATOBJ_Div(&pmxDst->efM21, &foDet);
//
//     /* Calculate the inverted x shift: Dx' = -Dx * M11' - Dy * M21' */
//     pmxDst->efDx = pmxSrc->efDx;
//     FLOATOBJ_Neg(&pmxDst->efDx);
//     MulSub(&pmxDst->efDx, &pmxDst->efDx, &pmxDst->efM11, &pmxSrc->efDy, &pmxDst->efM21);
//
//     /* Calculate the inverted y shift: Dy' = -Dy * M22' - Dx * M12' */
//     pmxDst->efDy = pmxSrc->efDy;
//     FLOATOBJ_Neg(&pmxDst->efDy);
//     MulSub(&pmxDst->efDy, &pmxDst->efDy, &pmxDst->efM22, &pmxSrc->efDx, &pmxDst->efM12);
//
//     /* Update accelerators and return complexity */
//     return XFORMOBJ_UpdateAccel(pxoDst);
// }
//
//
// /*!
//  * \brief Transforms fix-point coordinates in an array of POINTL structures using
//  *        the transformation matrix from the XFORMOBJ.
//  *
//  * \param pxo - Pointer to the XFORMOBJ
//  *
//  * \param cPoints - Number of coordinates to transform
//  *
//  * \param pptIn - Pointer to an array of POINTL structures containing the
//  *        source coordinates.
//  *
//  * \param pptOut - Pointer to an array of POINTL structures, receiving the
//  *        transformed coordinates. Can be the same as pptIn.
//  *
//  * \return TRUE if the operation was successful, FALSE if any of the calculations
//  *         caused an integer overflow.
//  *
//  * \note If the function returns FALSE, it might still have written to the
//  *       output buffer. If pptIn and pptOut are equal, the source coordinates
//  *       might have been partly overwritten!
//  */
// static
// BOOL
// NTAPI
// XFORMOBJ_bXformFixPoints(
//     _In_ XFORMOBJ *pxo,
//     _In_ ULONG cPoints,
//     _In_reads_(cPoints) PPOINTL pptIn,
//     _Out_writes_(cPoints) PPOINTL pptOut)
// {
//     PMATRIX pmx;
//     INT i;
//     FLOATOBJ fo1, fo2;
//     FLONG flAccel;
//     LONG lM11, lM12, lM21, lM22, lTemp;
//     register LONGLONG llx, lly;
//
//     pmx = XFORMOBJ_pmx(pxo);
//     flAccel = pmx->flAccel;
//
//     if ((flAccel & (XFORM_SCALE|XFORM_UNITY)) == (XFORM_SCALE|XFORM_UNITY))
//     {
//         /* Identity transformation */
//         RtlCopyMemory(pptOut, pptIn, cPoints * sizeof(POINTL));
//     }
//     else if (flAccel & XFORM_INTEGER)
//     {
//         if (flAccel & XFORM_UNITY)
//         {
//             /* 1-scale integer transform, get the off-diagonal elements */
//             if (!FLOATOBJ_bConvertToLong(&pmx->efM12, &lM12) ||
//                 !FLOATOBJ_bConvertToLong(&pmx->efM21, &lM21))
//             {
//                 NT_ASSERT(FALSE);
//                 return FALSE;
//             }
//
//             i = cPoints - 1;
//             do
//             {
//                 /* Calculate x in 64 bit and check for overflow */
//                 llx = Int32x32To64(pptIn[i].y, lM21) + pptIn[i].x;
//                 if (DOES_VALUE_OVERFLOW_LONG(llx))
//                 {
//                     return FALSE;
//                 }
//
//                 /* Calculate y in 64 bit and check for overflow */
//                 lly = Int32x32To64(pptIn[i].x, lM12) + pptIn[i].y;
//                 if (DOES_VALUE_OVERFLOW_LONG(lly))
//                 {
//                     return FALSE;
//                 }
//
//                 /* Write back the results */
//                 pptOut[i].x = (LONG)llx;
//                 pptOut[i].y = (LONG)lly;
//             }
//             while (--i >= 0);
//         }
//         else if (flAccel & XFORM_SCALE)
//         {
//             /* Diagonal integer transform, get the diagonal elements */
//             if (!FLOATOBJ_bConvertToLong(&pmx->efM11, &lM11) ||
//                 !FLOATOBJ_bConvertToLong(&pmx->efM22, &lM22))
//             {
//                 NT_ASSERT(FALSE);
//                 return FALSE;
//             }
//
//             i = cPoints - 1;
//             do
//             {
//                 /* Calculate x in 64 bit and check for overflow */
//                 llx = Int32x32To64(pptIn[i].x, lM11);
//                 if (DOES_VALUE_OVERFLOW_LONG(llx))
//                 {
//                     return FALSE;
//                 }
//
//                 /* Calculate y in 64 bit and check for overflow */
//                 lly = Int32x32To64(pptIn[i].y, lM22);
//                 if (DOES_VALUE_OVERFLOW_LONG(lly))
//                 {
//                     return FALSE;
//                 }
//
//                 /* Write back the results */
//                 pptOut[i].x = (LONG)llx;
//                 pptOut[i].y = (LONG)lly;
//             }
//             while (--i >= 0);
//         }
//         else
//         {
//             /* Full integer transform */
//             if (!FLOATOBJ_bConvertToLong(&pmx->efM11, &lM11) ||
//                 !FLOATOBJ_bConvertToLong(&pmx->efM12, &lM12) ||
//                 !FLOATOBJ_bConvertToLong(&pmx->efM21, &lM21) ||
//                 !FLOATOBJ_bConvertToLong(&pmx->efM22, &lM22))
//             {
//                 NT_ASSERT(FALSE);
//                 return FALSE;
//             }
//
//             i = cPoints - 1;
//             do
//             {
//                 /* Calculate x in 64 bit and check for overflow */
//                 llx  = Int32x32To64(pptIn[i].x, lM11);
//                 llx += Int32x32To64(pptIn[i].y, lM21);
//                 if (DOES_VALUE_OVERFLOW_LONG(llx))
//                 {
//                     return FALSE;
//                 }
//
//                 /* Calculate y in 64 bit and check for overflow */
//                 lly  = Int32x32To64(pptIn[i].y, lM22);
//                 lly += Int32x32To64(pptIn[i].x, lM12);
//                 if (DOES_VALUE_OVERFLOW_LONG(lly))
//                 {
//                     return FALSE;
//                 }
//
//                 /* Write back the results */
//                 pptOut[i].x = (LONG)llx;
//                 pptOut[i].y = (LONG)lly;
//             }
//             while (--i >= 0);
//         }
//     }
//     else if (flAccel & XFORM_UNITY)
//     {
//         /* 1-scale transform */
//         i = cPoints - 1;
//         do
//         {
//             /* Calculate x in 64 bit and check for overflow */
//             fo1 = pmx->efM21;
//             FLOATOBJ_MulLong(&fo1, pptIn[i].y);
//             if (!FLOATOBJ_bConvertToLong(&fo1, &lTemp))
//             {
//                 return FALSE;
//             }
//             llx = (LONGLONG)pptIn[i].x + lTemp;
//             if (DOES_VALUE_OVERFLOW_LONG(llx))
//             {
//                 return FALSE;
//             }
//
//             /* Calculate y in 64 bit and check for overflow */
//             fo2 = pmx->efM12;
//             FLOATOBJ_MulLong(&fo2, pptIn[i].x);
//             if (!FLOATOBJ_bConvertToLong(&fo2, &lTemp))
//             {
//                 return FALSE;
//             }
//             lly = (LONGLONG)pptIn[i].y + lTemp;
//             if (DOES_VALUE_OVERFLOW_LONG(lly))
//             {
//                 return FALSE;
//             }
//
//             /* Write back the results */
//             pptOut[i].x = (LONG)llx;
//             pptOut[i].y = (LONG)lly;
//         }
//         while (--i >= 0);
//     }
//     else if (flAccel & XFORM_SCALE)
//     {
//         /* Diagonal float transform */
//         i = cPoints - 1;
//         do
//         {
//             fo1 = pmx->efM11;
//             FLOATOBJ_MulLong(&fo1, pptIn[i].x);
//             if (!FLOATOBJ_bConvertToLong(&fo1, &pptOut[i].x))
//             {
//                 return FALSE;
//             }
//
//             fo2 = pmx->efM22;
//             FLOATOBJ_MulLong(&fo2, pptIn[i].y);
//             if (!FLOATOBJ_bConvertToLong(&fo2, &pptOut[i].y))
//             {
//                 return FALSE;
//             }
//         }
//         while (--i >= 0);
//     }
//     else
//     {
//         /* Full float transform */
//         i = cPoints - 1;
//         do
//         {
//             /* Calculate x as FLOATOBJ */
//             MulAddLong(&fo1, &pmx->efM11, pptIn[i].x, &pmx->efM21, pptIn[i].y);
//
//             /* Calculate y as FLOATOBJ */
//             MulAddLong(&fo2, &pmx->efM12, pptIn[i].x, &pmx->efM22, pptIn[i].y);
//
//             if (!FLOATOBJ_bConvertToLong(&fo1, &pptOut[i].x))
//             {
//                 return FALSE;
//             }
//
//             if (!FLOATOBJ_bConvertToLong(&fo2, &pptOut[i].y))
//             {
//                 return FALSE;
//             }
//         }
//         while (--i >= 0);
//     }
//
//     if (!(pmx->flAccel & XFORM_NO_TRANSLATION))
//     {
//         /* Translate points */
//         i = cPoints - 1;
//         do
//         {
//             llx = (LONGLONG)pptOut[i].x + pmx->fxDx;
//             if (DOES_VALUE_OVERFLOW_LONG(llx))
//             {
//                 return FALSE;
//             }
//             pptOut[i].x = (LONG)llx;
//
//             lly = (LONGLONG)pptOut[i].y + pmx->fxDy;
//             if (DOES_VALUE_OVERFLOW_LONG(lly))
//             {
//                 return FALSE;
//             }
//             pptOut[i].y = (LONG)lly;
//         }
//         while (--i >= 0);
//     }
//
//     return TRUE;
// }
//
// /** Public functions **********************************************************/
//
// // www.osr.com/ddk/graphics/gdifncs_0s2v.htm
// ULONG
// APIENTRY
// XFORMOBJ_iGetXform(
//     IN XFORMOBJ *pxo,
//     OUT XFORML *pxform)
// {
//     PMATRIX pmx = XFORMOBJ_pmx(pxo);
//
//     /* Check parameters */
//     if (!pxo || !pxform)
//     {
//         return DDI_ERROR;
//     }
//
//     /* Copy members */
//     pxform->eM11 = FLOATOBJ_GetFloat(&pmx->efM11);
//     pxform->eM12 = FLOATOBJ_GetFloat(&pmx->efM12);
//     pxform->eM21 = FLOATOBJ_GetFloat(&pmx->efM21);
//     pxform->eM22 = FLOATOBJ_GetFloat(&pmx->efM22);
//     pxform->eDx = FLOATOBJ_GetFloat(&pmx->efDx);
//     pxform->eDy = FLOATOBJ_GetFloat(&pmx->efDy);
//
//     /* Return complexity hint */
//     return HintFromAccel(pmx->flAccel);
// }
//
//
// // www.osr.com/ddk/graphics/gdifncs_5ig7.htm
// ULONG
// APIENTRY
// XFORMOBJ_iGetFloatObjXform(
//     IN XFORMOBJ *pxo,
//     OUT FLOATOBJ_XFORM *pxfo)
// {
//     PMATRIX pmx = XFORMOBJ_pmx(pxo);
//
//     /* Check parameters */
//     if (!pxo || !pxfo)
//     {
//         return DDI_ERROR;
//     }
//
//     /* Copy members */
//     pxfo->eM11 = pmx->efM11;
//     pxfo->eM12 = pmx->efM12;
//     pxfo->eM21 = pmx->efM21;
//     pxfo->eM22 = pmx->efM22;
//     pxfo->eDx = pmx->efDx;
//     pxfo->eDy = pmx->efDy;
//
//     /* Return complexity hint */
//     return HintFromAccel(pmx->flAccel);
// }
//
//
// // www.osr.com/ddk/graphics/gdifncs_027b.htm
// BOOL
// APIENTRY
// XFORMOBJ_bApplyXform(
//     IN XFORMOBJ *pxo,
//     IN ULONG iMode,
//     IN ULONG cPoints,
//     IN PVOID pvIn,
//     OUT PVOID pvOut)
// {
//     MATRIX mx;
//     XFORMOBJ xoInv;
//     PPOINTL pptlIn, pptlOut;
//     INT i;
//
//     /* Check parameters */
//     if (!pxo || !pvIn || !pvOut || cPoints < 1)
//     {
//         return FALSE;
//     }
//
//     /* Use inverse xform? */
//     if (iMode == XF_INV_FXTOL || iMode == XF_INV_LTOL)
//     {
//         XFORMOBJ_vInit(&xoInv, &mx);
//         if (XFORMOBJ_iInverse(&xoInv, pxo) == DDI_ERROR)
//         {
//             return FALSE;
//         }
//         pxo = &xoInv;
//     }
//
//     /* Convert POINTL to POINTFIX? */
//     if (iMode == XF_LTOFX || iMode == XF_LTOL || iMode == XF_INV_LTOL)
//     {
//         pptlIn = pvIn;
//         pptlOut = pvOut;
//         for (i = cPoints - 1; i >= 0; i--)
//         {
//             pptlOut[i].x = LONG2FIX(pptlIn[i].x);
//             pptlOut[i].y = LONG2FIX(pptlIn[i].y);
//         }
//
//         /* The input is in the out buffer now! */
//         pvIn = pvOut;
//     }
//
//     /* Do the actual fixpoint transformation */
//     if (!XFORMOBJ_bXformFixPoints(pxo, cPoints, pvIn, pvOut))
//     {
//         return FALSE;
//     }
//
//     /* Convert POINTFIX to POINTL? */
//     if (iMode == XF_INV_FXTOL || iMode == XF_INV_LTOL || iMode == XF_LTOL)
//     {
//         pptlOut = pvOut;
//         for (i = cPoints - 1; i >= 0; i--)
//         {
//             pptlOut[i].x = FIX2LONG(pptlOut[i].x);
//             pptlOut[i].y = FIX2LONG(pptlOut[i].y);
//         }
//     }
//
//     return TRUE;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
