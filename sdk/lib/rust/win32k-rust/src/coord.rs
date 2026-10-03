use win32k_sys::win32k::*;
#[allow(unused_imports)]
use core::ffi::c_void;
// FLOATOBJ per-arch shim: real ENGAPI(stdcall) symbols on i386 / inline f32 ops
// on amd64+arm64. coord.c's FLOATOBJ_* (incl. Add/DivLong) come from here — the
// LLM's local `extern "C"` decls were wrong ABI on i386 AND undefined on non-x86.
use crate::floatobj::*;
// EngMulDiv is glob-exported by BOTH win32k_sys and floatobj — explicit import
// disambiguates (same underlying engine symbol).
use win32k_sys::win32k::EngMulDiv;
use ntkrnl_sys::base::STATUS_SUCCESS;
const STATUS_ACCESS_VIOLATION: i32 = 0xC0000005_u32 as i32;
use ntkrnl_sys::ddk::{ExAllocatePoolWithTag, ExFreePoolWithTag};
use ntkrnl_sys::rtl::RtlCopyMemory;

extern "C" {
    fn abs(n: i64) -> i64;
}

// FLOATOBJ constants (identity-matrix cells) — per-arch (struct on i386, f32 else).
#[cfg(target_arch = "x86")] const FLOATOBJ_0: FLOATOBJ = FLOATOBJ { ul1: 0x00000000, ul2: 0x00000000 };
#[cfg(target_arch = "x86")] const FLOATOBJ_1: FLOATOBJ = FLOATOBJ { ul1: 0x40000000, ul2: 0x00000002 };
#[cfg(not(target_arch = "x86"))] const FLOATOBJ_0: FLOATOBJ = 0.;
#[cfg(not(target_arch = "x86"))] const FLOATOBJ_1: FLOATOBJ = 1.;

// GDITAG_TEMP is 'pmtG' (tags.h); pool tags are stored little-endian → b"Gtmp".
const GDITAG_TEMP: u32 = u32::from_le_bytes(*b"Gtmp");

#[inline(always)]
fn NT_SUCCESS(s: NTSTATUS) -> bool { s >= 0 }
// SafeProbeFor{Read,Write} are NTAPI (stdcall on i386); extern "C" mis-decorates.
extern "system" {
    fn SafeProbeForRead(Address: *const core::ffi::c_void, Length: SIZE_T, Alignment: ULONG) -> NTSTATUS;
    fn SafeProbeForWrite(Ptr: *mut core::ffi::c_void, Length: SIZE_T, Alignment: u32) -> i32;
}
// XFORMOBJ_* are FORCEINLINE macros → EXFORMOBJ_* real symbols (xformobj.h /
// xformobj.rs). vInit just sets pmx (no symbol — local inline, region.rs idiom);
// the rest forward to the exported EXFORMOBJ_* (keeps call sites unchanged).
// CRITICAL (amd64): XFORMOBJ is {ulReserved: ULONG} = 4 bytes, but the kernel
// stores an _EXFORMOBJ {pmx: *mut MATRIX} = 8 bytes on 64-bit into it. So the
// stack locals MUST be allocated as EXFORMOBJ (8B) or the pmx write overflows a
// 4B slot → uninitialized high dword read back as a garbage ptr → 0x1E AV.
// (i386: both 4B, no overflow.) Helpers take *EXFORMOBJ; cast to *XFORMOBJ only
// when forwarding to the exported EXFORMOBJ_* symbols.
use crate::xformobj::{EXFORMOBJ_iSetXform, EXFORMOBJ_iCombine, EXFORMOBJ_iInverse, EXFORMOBJ_iGetXform};
#[inline(always)] unsafe fn XFORMOBJ_vInit(pxo: *mut EXFORMOBJ, pmx: *mut MATRIX) { (*pxo).pmx = pmx; }
#[inline(always)] unsafe fn XFORMOBJ_iSetXform(pxo: *mut EXFORMOBJ, pxform: *const XFORML) -> ULONG { EXFORMOBJ_iSetXform(pxo as *mut XFORMOBJ, pxform) }
#[inline(always)] unsafe fn XFORMOBJ_iCombine(pxo: *mut EXFORMOBJ, pxo1: *mut EXFORMOBJ, pxo2: *mut EXFORMOBJ) -> core::ffi::c_ulong { EXFORMOBJ_iCombine(pxo as *mut XFORMOBJ, pxo1 as *mut XFORMOBJ, pxo2 as *mut XFORMOBJ) }
#[inline(always)] unsafe fn XFORMOBJ_iGetXform(pxo: *const EXFORMOBJ, pxform: *mut XFORML) -> u32 { EXFORMOBJ_iGetXform(pxo as *mut XFORMOBJ, pxform) }
#[inline(always)] unsafe fn XFORMOBJ_iInverse(pxo: *mut EXFORMOBJ, pxo1: *mut EXFORMOBJ) -> ULONG { EXFORMOBJ_iInverse(pxo as *mut XFORMOBJ, pxo1 as *mut XFORMOBJ) }
// SetLastNtError + MX_Set0 are FASTCALL (@Name@N on i386), cdecl elsewhere.
#[cfg(target_arch = "x86")] extern "fastcall" {
    fn SetLastNtError(Status: NTSTATUS);
    fn MX_Set0(pmx: PMATRIX);
}
#[cfg(not(target_arch = "x86"))] extern "C" {
    fn SetLastNtError(Status: NTSTATUS);
    fn MX_Set0(pmx: PMATRIX);
}

// ── coord.h FORCEINLINE helpers (no exported symbol → local Rust impls) ──────
// INTERNAL_APPLY_MATRIX mirrors dcutil.rs/line.rs verbatim (validated).
#[inline(always)]
unsafe fn INTERNAL_APPLY_MATRIX(matrix: PMATRIX, points: LPPOINT, count: INT) -> BOOL {
    let mut count = count as u32;
    while count != 0 {
        count -= 1;
        let p = points.offset(count as isize);
        let mut x: FLOATOBJ = core::mem::zeroed();
        let mut y: FLOATOBJ = core::mem::zeroed();
        let mut tmp: FLOATOBJ;
        FLOATOBJ_SetLong(core::ptr::addr_of_mut!(x), (*p).x);
        FLOATOBJ_Mul(core::ptr::addr_of_mut!(x), core::ptr::addr_of_mut!((*matrix).efM11));
        tmp = (*matrix).efM21;
        FLOATOBJ_MulLong(core::ptr::addr_of_mut!(tmp), (*p).y);
        FLOATOBJ_Add(core::ptr::addr_of_mut!(x), core::ptr::addr_of_mut!(tmp));
        FLOATOBJ_Add(core::ptr::addr_of_mut!(x), core::ptr::addr_of_mut!((*matrix).efDx));
        FLOATOBJ_SetLong(core::ptr::addr_of_mut!(y), (*p).y);
        FLOATOBJ_Mul(core::ptr::addr_of_mut!(y), core::ptr::addr_of_mut!((*matrix).efM22));
        tmp = (*matrix).efM12;
        FLOATOBJ_MulLong(core::ptr::addr_of_mut!(tmp), (*p).x);
        FLOATOBJ_Add(core::ptr::addr_of_mut!(y), core::ptr::addr_of_mut!(tmp));
        FLOATOBJ_Add(core::ptr::addr_of_mut!(y), core::ptr::addr_of_mut!((*matrix).efDy));
        if FLOATOBJ_bConvertToLong(core::ptr::addr_of!(x), core::ptr::addr_of_mut!((*p).x)) == 0 { return FALSE as BOOL; }
        if FLOATOBJ_bConvertToLong(core::ptr::addr_of!(y), core::ptr::addr_of_mut!((*p).y)) == 0 { return FALSE as BOOL; }
    }
    TRUE as BOOL
}

#[inline(always)]
unsafe fn DC_pmxWorldToPage(pdc: PDC) -> PMATRIX {
    core::ptr::addr_of_mut!((*(*pdc).pdcattr).mxWorldToPage)
}

#[inline(always)]
unsafe fn DC_pmxWorldToDevice(pdc: PDC) -> PMATRIX {
    let m = (PAGE_XLATE_CHANGED | PAGE_EXTENTS_CHANGED | WORLD_XFORM_CHANGED) as core::ffi::c_ulong;
    if (*(*pdc).pdcattr).flXform & m != 0 {
        DC_vUpdateWorldToDevice(pdc);
    }
    core::ptr::addr_of_mut!((*(*pdc).pdcattr).mxWorldToDevice)
}

#[inline(always)]
unsafe fn DC_pmxDeviceToWorld(pdc: PDC) -> PMATRIX {
    if (*(*pdc).pdcattr).flXform & DEVICE_TO_WORLD_INVALID as core::ffi::c_ulong != 0 {
        DC_vUpdateDeviceToWorld(pdc);
    }
    core::ptr::addr_of_mut!((*(*pdc).pdcattr).mxDeviceToWorld)
}

#[inline(always)]
unsafe fn DC_pszlViewportExt(pdc: PDC) -> PSIZEL {
    let pdcattr = (*pdc).pdcattr;
    if (*pdcattr).flXform & PAGE_EXTENTS_CHANGED as core::ffi::c_ulong != 0
        && (*pdcattr).iMapMode == MM_ISOTROPIC as i32
    {
        DC_vFixIsotropicMapping(pdc);
    }
    core::ptr::addr_of_mut!((*pdcattr).szlViewportExt)
}

// ---- file-scope preamble (translated) ----
pub static gmxIdentity: MATRIX = MATRIX {
    efM11: FLOATOBJ_1,
    efM12: FLOATOBJ_0,
    efM21: FLOATOBJ_0,
    efM22: FLOATOBJ_1,
    efDx: FLOATOBJ_0,
    efDy: FLOATOBJ_0,
    fxDx: 0,
    fxDy: 0,
    flAccel: (XFORM_NO_TRANSLATION | XFORM_FORMAT_LTOL | XFORM_UNITY | XFORM_SCALE) as FLONG,
};

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn DC_vFixIsotropicMapping(pdc: PDC) {
    DC_vFixIsotropicMapping_impl(pdc)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn DC_vFixIsotropicMapping(pdc: PDC) {
    DC_vFixIsotropicMapping_impl(pdc)
}

unsafe fn DC_vFixIsotropicMapping_impl(pdc: PDC) {

    let pdcattr: PDC_ATTR = (*pdc).pdcattr;

    let szlWindowExt: SIZEL = (*pdcattr).szlWindowExt;
    let szlViewportExt: SIZEL = (*pdcattr).szlViewportExt;

    if szlWindowExt.cx == 0 || szlWindowExt.cy == 0
        || szlViewportExt.cx == 0 || szlViewportExt.cy == 0
    {
        return;
    }

    let fx: i64 = abs(szlWindowExt.cx as i64 * szlViewportExt.cy as i64);
    let fy: i64 = abs(szlWindowExt.cy as i64 * szlViewportExt.cx as i64);

    if fx < fy {
        let s: i64 = if (szlWindowExt.cy ^ szlViewportExt.cx) > 0 { 1 } else { -1 };
        (*pdcattr).szlViewportExt.cx = (fx * s / szlWindowExt.cy as i64) as i32;
    } else if fx > fy {
        let s: i64 = if (szlWindowExt.cx ^ szlViewportExt.cy) > 0 { 1 } else { -1 };
        (*pdcattr).szlViewportExt.cy = (fy * s / szlWindowExt.cx as i64) as i32;
    }

    (*(*pdc).pdcattr).flXform &= !PAGE_EXTENTS_CHANGED;
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn DC_vGetPageToDevice(pdc: PDC, pmx: *mut MATRIX) {
    DC_vGetPageToDevice_impl(pdc, pmx)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn DC_vGetPageToDevice(pdc: PDC, pmx: *mut MATRIX) {
    DC_vGetPageToDevice_impl(pdc, pmx)
}

unsafe fn DC_vGetPageToDevice_impl(pdc: PDC, pmx: *mut MATRIX) {

    let pdcattr: PDC_ATTR = (*pdc).pdcattr;
    let pszlViewPortExt: PSIZEL = DC_pszlViewportExt(pdc);
    let szlWindowExt: SIZEL = (*pdcattr).szlWindowExt;

    FLOATOBJ_SetLong(core::ptr::addr_of_mut!((*pmx).efM12), 0);
    FLOATOBJ_SetLong(core::ptr::addr_of_mut!((*pmx).efM21), 0);

    if szlWindowExt.cx != 0 {
        FLOATOBJ_SetLong(core::ptr::addr_of_mut!((*pmx).efM11), (*pszlViewPortExt).cx);
        FLOATOBJ_DivLong(core::ptr::addr_of_mut!((*pmx).efM11), szlWindowExt.cx);
    } else {
        FLOATOBJ_SetLong(core::ptr::addr_of_mut!((*pmx).efM11), 1);
    }

    if szlWindowExt.cy != 0 {
        FLOATOBJ_SetLong(core::ptr::addr_of_mut!((*pmx).efM22), (*pszlViewPortExt).cy);
        FLOATOBJ_DivLong(core::ptr::addr_of_mut!((*pmx).efM22), szlWindowExt.cy);
    } else {
        FLOATOBJ_SetLong(core::ptr::addr_of_mut!((*pmx).efM22), 1);
    }

    FLOATOBJ_SetLong(core::ptr::addr_of_mut!((*pmx).efDx), -(*pdcattr).ptlWindowOrg.x);
    FLOATOBJ_Mul(
        core::ptr::addr_of_mut!((*pmx).efDx),
        core::ptr::addr_of_mut!((*pmx).efM11),
    );
    FLOATOBJ_AddLong(core::ptr::addr_of_mut!((*pmx).efDx), (*pdcattr).ptlViewportOrg.x);

    FLOATOBJ_SetLong(core::ptr::addr_of_mut!((*pmx).efDy), -(*pdcattr).ptlWindowOrg.y);
    FLOATOBJ_Mul(
        core::ptr::addr_of_mut!((*pmx).efDy),
        core::ptr::addr_of_mut!((*pmx).efM22),
    );
    FLOATOBJ_AddLong(core::ptr::addr_of_mut!((*pmx).efDy), (*pdcattr).ptlViewportOrg.y);
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn DC_vUpdateWorldToDevice(pdc: PDC) {
    DC_vUpdateWorldToDevice_impl(pdc)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn DC_vUpdateWorldToDevice(pdc: PDC) {
    DC_vUpdateWorldToDevice_impl(pdc)
}

unsafe fn DC_vUpdateWorldToDevice_impl(pdc: PDC) {

    let mut xoPageToDevice: EXFORMOBJ = core::mem::zeroed();
    let mut xoWorldToPage: EXFORMOBJ = core::mem::zeroed();
    let mut xoWorldToDevice: EXFORMOBJ = core::mem::zeroed();
    let mut mxPageToDevice: MATRIX = core::mem::zeroed();

    DC_vGetPageToDevice(pdc, &mut mxPageToDevice);
    XFORMOBJ_vInit(&mut xoPageToDevice, &mut mxPageToDevice);

    XFORMOBJ_vInit(&mut xoWorldToPage, core::ptr::addr_of_mut!((*(*pdc).pdcattr).mxWorldToPage) as *mut MATRIX);
    XFORMOBJ_vInit(&mut xoWorldToDevice, core::ptr::addr_of_mut!((*(*pdc).pdcattr).mxWorldToDevice) as *mut MATRIX);
    XFORMOBJ_iCombine(&mut xoWorldToDevice, &mut xoWorldToPage, &mut xoPageToDevice);

    (*(*pdc).pdcattr).flXform &= !(WORLD_XFORM_CHANGED as core::ffi::c_ulong);
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn DC_vUpdateDeviceToWorld(pdc: PDC) {
    DC_vUpdateDeviceToWorld_impl(pdc)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn DC_vUpdateDeviceToWorld(pdc: PDC) {
    DC_vUpdateDeviceToWorld_impl(pdc)
}

// Reconstructed: the stitcher dropped this body and fused the signature with
// GreCombineTransform. Mirrors coord.c DC_vUpdateDeviceToWorld (L203-224).
unsafe fn DC_vUpdateDeviceToWorld_impl(pdc: PDC) {
    let mut xoWorldToDevice: EXFORMOBJ = core::mem::zeroed();
    let mut xoDeviceToWorld: EXFORMOBJ = core::mem::zeroed();

    /* Get the world-to-device translation */
    let pmxWorldToDevice: PMATRIX = DC_pmxWorldToDevice(pdc);
    XFORMOBJ_vInit(&mut xoWorldToDevice, pmxWorldToDevice);

    /* Create inverse of world-to-device transformation */
    XFORMOBJ_vInit(
        &mut xoDeviceToWorld,
        core::ptr::addr_of_mut!((*(*pdc).pdcattr).mxDeviceToWorld) as *mut MATRIX,
    );
    if XFORMOBJ_iInverse(&mut xoDeviceToWorld, &mut xoWorldToDevice) == DDI_ERROR {
        MX_Set0(core::ptr::addr_of_mut!((*(*pdc).pdcattr).mxDeviceToWorld));
        return;
    }

    /* Reset the flag */
    (*(*pdc).pdcattr).flXform &= !(DEVICE_TO_WORLD_INVALID as core::ffi::c_ulong);
}

#[no_mangle]
pub unsafe extern "system" fn GreCombineTransform(
    pxformDest: *mut XFORML,
    pxform1: *mut XFORML,
    pxform2: *mut XFORML,
) -> BOOL {

    let mut mxDest: MATRIX = core::mem::zeroed();
    let mut mx1: MATRIX = core::mem::zeroed();
    let mut mx2: MATRIX = core::mem::zeroed();
    let mut xoDest: EXFORMOBJ = core::mem::zeroed();
    let mut xo1: EXFORMOBJ = core::mem::zeroed();
    let mut xo2: EXFORMOBJ = core::mem::zeroed();

    if pxformDest.is_null() || pxform1.is_null() || pxform2.is_null() {
        return 0;
    }

    XFORMOBJ_vInit(&mut xoDest, &mut mxDest);
    XFORMOBJ_vInit(&mut xo1, &mut mx1);
    XFORMOBJ_vInit(&mut xo2, &mut mx2);

    XFORMOBJ_iSetXform(&mut xo1, pxform1);
    XFORMOBJ_iSetXform(&mut xo2, pxform2);

    XFORMOBJ_iCombine(&mut xoDest, &mut xo1, &mut xo2);

    XFORMOBJ_iGetXform(&xoDest, pxformDest);

    1
}

#[no_mangle]
pub unsafe extern "system" fn NtGdiCombineTransform(
    UnsafeXFormResult: LPXFORM,
    Unsafexform1: LPXFORM,
    Unsafexform2: LPXFORM,
) -> BOOL {
    // The C probes all three XFORMs inside one _SEH2_TRY and returns FALSE from the
    // handler. All three probes were dropped in translation, so this called straight
    // through with three unvalidated user pointers — and GreCombineTransform WRITES
    // through UnsafeXFormResult, so a kernel-space pointer passed here is an arbitrary
    // kernel write available to any unprivileged caller.
    //
    // Validate-first per docs/engineering/dummy-pseh-verdict.md: under DUMMY_PSEH a
    // fault has no handler to reach, so the range must be proven before the call, not
    // recovered from after it. SafeProbeFor* is win32k's own non-touching validator.
    if !NT_SUCCESS(SafeProbeForWrite(
        UnsafeXFormResult as *mut core::ffi::c_void,
        core::mem::size_of::<XFORML>() as SIZE_T,
        1,
    )) {
        return FALSE as BOOL;
    }
    if !NT_SUCCESS(SafeProbeForRead(
        Unsafexform1 as *const core::ffi::c_void,
        core::mem::size_of::<XFORML>() as SIZE_T,
        1,
    )) {
        return FALSE as BOOL;
    }
    if !NT_SUCCESS(SafeProbeForRead(
        Unsafexform2 as *const core::ffi::c_void,
        core::mem::size_of::<XFORML>() as SIZE_T,
        1,
    )) {
        return FALSE as BOOL;
    }

    GreCombineTransform(
        UnsafeXFormResult as *mut XFORML,
        Unsafexform1 as *mut XFORML,
        Unsafexform2 as *mut XFORML,
    )
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiGetTransform(
    hdc: HDC,
    iXform: u32,
    pXForm: LPXFORM,
) -> i32 {

    let pdc: PDC;
    let mut ret: i32 = 1;
    let mut mxPageToDevice: MATRIX = core::mem::zeroed();
    let mut xo: EXFORMOBJ = core::mem::zeroed();
    let pmx: PMATRIX;

    if pXForm.is_null() {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    pdc = DC_LockDc(hdc);
    if pdc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }

    'leave: {
        match iXform {
            x if x == GdiWorldSpaceToPageSpace => {
                pmx = DC_pmxWorldToPage(pdc);
            }
            x if x == GdiWorldSpaceToDeviceSpace => {
                pmx = DC_pmxWorldToDevice(pdc);
            }
            x if x == GdiDeviceSpaceToWorldSpace => {
                pmx = DC_pmxDeviceToWorld(pdc);
            }
            x if x == GdiPageSpaceToDeviceSpace => {
                DC_vGetPageToDevice(pdc, core::ptr::addr_of_mut!(mxPageToDevice));
                pmx = core::ptr::addr_of_mut!(mxPageToDevice);
            }
            _ => {
                ret = 0;
                break 'leave;
            }
        }

        XFORMOBJ_vInit(core::ptr::addr_of_mut!(xo), pmx);

        // The C probes pXForm for WRITE here and sets ret = FALSE from the handler;
        // the probe was dropped and XFORMOBJ_iGetXform writes the caller's pointer
        // unchecked. The null test above is not a substitute — it rejects one bad
        // pointer out of the whole kernel range.
        if !NT_SUCCESS(SafeProbeForWrite(
            pXForm as *mut core::ffi::c_void,
            core::mem::size_of::<XFORML>() as SIZE_T,
            1,
        )) {
            ret = 0;
            break 'leave;
        }

        XFORMOBJ_iGetXform(core::ptr::addr_of!(xo), pXForm as *mut XFORML);
    }

    DC_UnlockDc(pdc);
    ret
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn NtGdiTransformPoints(
    hDC: HDC,
    UnsafePtsIn: PPOINT,
    UnsafePtOut: PPOINT,
    Count: INT,
    iMode: INT,
) -> BOOL {
    let pdc: PDC;
    let Points: LPPOINT;
    let Size: ULONG;
    let mut ret: BOOL = TRUE as BOOL;
    let mut Status: NTSTATUS;

    if Count <= 0 {
        return TRUE as BOOL;
    }

    if UnsafePtsIn.is_null() || UnsafePtOut.is_null() {
        return FALSE as BOOL;
    }

    pdc = DC_LockDc(hDC);
    if pdc.is_null() {
        return FALSE as BOOL;
    }

    Size = (Count as ULONG) * (core::mem::size_of::<POINT>() as ULONG);

    Points = ExAllocatePoolWithTag(PagedPool, Size as SIZE_T, GDITAG_TEMP) as LPPOINT;
    if Points.is_null() {
        DC_UnlockDc(pdc);
        EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return FALSE as BOOL;
    }

    'leave: {
        Status = SafeProbeForWrite(UnsafePtOut as *mut _, Size, 1);
        if NT_SUCCESS(Status) {
            Status = SafeProbeForRead(UnsafePtsIn as *mut _, Size, 1);
        }
        // The probes VALIDATE only; this capture is the access, and a fault takes the
        // same exit a failed probe takes.
        if !NT_SUCCESS(Status)
            || !crate::copy::capture_from_user(
                Points as *mut core::ffi::c_void,
                UnsafePtsIn as *const core::ffi::c_void,
                Size as usize,
            )
        {
            break 'leave;
        }

        match iMode as u32 {
            GdiDpToLp => {
                ret = INTERNAL_APPLY_MATRIX(DC_pmxDeviceToWorld(pdc), Points, Count);
            }
            GdiLpToDp => {
                ret = INTERNAL_APPLY_MATRIX(DC_pmxWorldToDevice(pdc), Points, Count);
            }
            _ => {
                EngSetLastError(ERROR_INVALID_PARAMETER);
                ret = FALSE as BOOL;
                break 'leave;
            }
        }

        // Probed above, but the probe only validates -- this publish is the access.
        // A fault here means the caller's output buffer went away, so report the
        // failure rather than claiming success.
        if ret != 0
            && !crate::copy::publish_to_user(
                UnsafePtOut as *mut core::ffi::c_void,
                Points as *const core::ffi::c_void,
                Size as usize,
            )
        {
            EngSetLastError(ERROR_INVALID_PARAMETER);
            ret = FALSE as BOOL;
        }
    }

    DC_UnlockDc(pdc);
    ExFreePoolWithTag(Points as *mut _, GDITAG_TEMP);
    ret
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn NtGdiTransformPoints(
    hDC: HDC,
    UnsafePtsIn: PPOINT,
    UnsafePtOut: PPOINT,
    Count: INT,
    iMode: INT,
) -> BOOL {
    let pdc: PDC;
    let Points: LPPOINT;
    let Size: ULONG;
    let mut ret: BOOL = TRUE as BOOL;
    let mut Status: NTSTATUS;

    if Count <= 0 {
        return TRUE as BOOL;
    }

    if UnsafePtsIn.is_null() || UnsafePtOut.is_null() {
        return FALSE as BOOL;
    }

    pdc = DC_LockDc(hDC);
    if pdc.is_null() {
        return FALSE as BOOL;
    }

    Size = (Count as ULONG) * (core::mem::size_of::<POINT>() as ULONG);

    Points = ExAllocatePoolWithTag(PagedPool, Size as SIZE_T, GDITAG_TEMP) as LPPOINT;
    if Points.is_null() {
        DC_UnlockDc(pdc);
        EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return FALSE as BOOL;
    }

    'leave: {
        Status = SafeProbeForWrite(UnsafePtOut as *mut _, Size as _, 1);
        if NT_SUCCESS(Status) {
            Status = SafeProbeForRead(UnsafePtsIn as *mut _, Size as _, 1);
        }
        // The probes VALIDATE only; this capture is the access, and a fault takes the
        // same exit a failed probe takes.
        if !NT_SUCCESS(Status)
            || !crate::copy::capture_from_user(
                Points as *mut core::ffi::c_void,
                UnsafePtsIn as *const core::ffi::c_void,
                Size as usize,
            )
        {
            break 'leave;
        }

        match iMode as u32 {
            GdiDpToLp => {
                ret = INTERNAL_APPLY_MATRIX(DC_pmxDeviceToWorld(pdc), Points, Count);
            }
            GdiLpToDp => {
                ret = INTERNAL_APPLY_MATRIX(DC_pmxWorldToDevice(pdc), Points, Count);
            }
            _ => {
                EngSetLastError(ERROR_INVALID_PARAMETER);
                ret = FALSE as BOOL;
                break 'leave;
            }
        }

        // Probed above, but the probe only validates -- this publish is the access.
        // A fault here means the caller's output buffer went away, so report the
        // failure rather than claiming success.
        if ret != 0
            && !crate::copy::publish_to_user(
                UnsafePtOut as *mut core::ffi::c_void,
                Points as *const core::ffi::c_void,
                Size as usize,
            )
        {
            EngSetLastError(ERROR_INVALID_PARAMETER);
            ret = FALSE as BOOL;
        }
    }

    DC_UnlockDc(pdc);
    ExFreePoolWithTag(Points as *mut _, GDITAG_TEMP);
    ret
}
#[no_mangle]
pub unsafe extern "system" fn GreModifyWorldTransform(
    pdc: PDC,
    pxform: *const XFORML,
    dwMode: u32,
) -> i32 {
    let mut mxSrc: MATRIX = core::mem::zeroed();
    let mut xoSrc: EXFORMOBJ = core::mem::zeroed();
    let mut xoDC: EXFORMOBJ = core::mem::zeroed();

    match dwMode {
        MWT_IDENTITY => {
            (*(*pdc).pdcattr).mxWorldToPage = gmxIdentity;
        }
        MWT_LEFTMULTIPLY => {
            XFORMOBJ_vInit(&mut xoDC, core::ptr::addr_of_mut!((*(*pdc).pdcattr).mxWorldToPage));
            XFORMOBJ_vInit(&mut xoSrc, &mut mxSrc);
            if XFORMOBJ_iSetXform(&mut xoSrc, pxform) == DDI_ERROR {
                return 0;
            }
            XFORMOBJ_iCombine(&mut xoDC, &mut xoSrc, &mut xoDC);
        }
        MWT_RIGHTMULTIPLY => {
            XFORMOBJ_vInit(&mut xoDC, core::ptr::addr_of_mut!((*(*pdc).pdcattr).mxWorldToPage));
            XFORMOBJ_vInit(&mut xoSrc, &mut mxSrc);
            if XFORMOBJ_iSetXform(&mut xoSrc, pxform) == DDI_ERROR {
                return 0;
            }
            XFORMOBJ_iCombine(&mut xoDC, &mut xoDC, &mut xoSrc);
        }
        MWT_SET => {
            XFORMOBJ_vInit(&mut xoDC, core::ptr::addr_of_mut!((*(*pdc).pdcattr).mxWorldToPage));
            if XFORMOBJ_iSetXform(&mut xoDC, pxform) == DDI_ERROR {
                return 0;
            }
        }
        _ => return 0,
    }

    (*(*pdc).pdcattr).flXform |= (WORLD_XFORM_CHANGED | DEVICE_TO_WORLD_INVALID) as u32;

    1
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn NtGdiModifyWorldTransform(
    hdc: HDC,
    pxformUnsafe: LPXFORM,
    dwMode: u32,
) -> u32 {
    let mut xformSafe: XFORML = core::mem::zeroed();
    let mut Ret: u32 = 1;

    let pdc: PDC = DC_LockDc(hdc);
    if pdc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }

    if dwMode != MWT_IDENTITY && pxformUnsafe.is_null() {
        DC_UnlockDc(pdc);
        return 0;
    }

    if !pxformUnsafe.is_null() {
        let Status = SafeProbeForRead(
            pxformUnsafe as *const core::ffi::c_void,
            core::mem::size_of::<XFORML>() as u32,
            1,
        );
        // Validate-only probe; the capture is the access. Both faults clear Ret, which
        // is what gates the GreModifyWorldTransform call below.
        if !NT_SUCCESS(Status)
            || !crate::copy::capture_from_user(
                core::ptr::addr_of_mut!(xformSafe) as *mut core::ffi::c_void,
                pxformUnsafe as *const core::ffi::c_void,
                core::mem::size_of::<XFORML>(),
            )
        {
            Ret = 0;
        }
    }

    if Ret != 0 {
        Ret = GreModifyWorldTransform(pdc, core::ptr::addr_of!(xformSafe), dwMode) as u32;
    }
    DC_UnlockDc(pdc);
    Ret
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn NtGdiModifyWorldTransform(
    hdc: HDC,
    pxformUnsafe: LPXFORM,
    dwMode: u32,
) -> u32 {
    let mut xformSafe: XFORML = core::mem::zeroed();
    let mut Ret: u32 = 1;

    let pdc: PDC = DC_LockDc(hdc);
    if pdc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }

    if dwMode != MWT_IDENTITY && pxformUnsafe.is_null() {
        DC_UnlockDc(pdc);
        return 0;
    }

    if !pxformUnsafe.is_null() {
        let Status = SafeProbeForRead(
            pxformUnsafe as *const core::ffi::c_void,
            core::mem::size_of::<XFORML>() as _,
            1,
        );
        // Validate-only probe; the capture is the access. Both faults clear Ret, which
        // is what gates the GreModifyWorldTransform call below.
        if !NT_SUCCESS(Status)
            || !crate::copy::capture_from_user(
                core::ptr::addr_of_mut!(xformSafe) as *mut core::ffi::c_void,
                pxformUnsafe as *const core::ffi::c_void,
                core::mem::size_of::<XFORML>(),
            )
        {
            Ret = 0;
        }
    }

    if Ret != 0 {
        Ret = GreModifyWorldTransform(pdc, core::ptr::addr_of!(xformSafe), dwMode) as u32;
    }
    DC_UnlockDc(pdc);
    Ret
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn NtGdiOffsetViewportOrgEx(
    hDC: HDC,
    mut XOffset: i32,
    YOffset: i32,
    UnsafePoint: LPPOINT,
) -> u8 {
    let dc: PDC;
    let pdcattr: PDC_ATTR;
    let mut Status: i32 = STATUS_SUCCESS;

    dc = DC_LockDc(hDC);
    if dc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0u8;
    }
    pdcattr = (*dc).pdcattr;

    if !UnsafePoint.is_null() {
        Status = SafeProbeForWrite(UnsafePoint as *mut _, core::mem::size_of::<POINT>() as u32, 1);
        if NT_SUCCESS(Status) {
            let mut SafePoint: POINT = core::mem::zeroed();
            SafePoint.x = (*pdcattr).ptlViewportOrg.x;
            SafePoint.y = (*pdcattr).ptlViewportOrg.y;
            if (*pdcattr).dwLayout & LAYOUT_RTL != 0 {
                SafePoint.x = -SafePoint.x;
            }
            // The probe only range-checks (validate-only by design under
            // USE_DUMMY_PSEH); the C wrapped probe AND store in one _SEH2_TRY, so
            // the store itself must be the fault-tolerant copy.
            if !crate::copy::publish_to_user(
                UnsafePoint as *mut core::ffi::c_void,
                core::ptr::addr_of!(SafePoint) as *const core::ffi::c_void,
                core::mem::size_of::<POINT>(),
            ) {
                Status = STATUS_ACCESS_VIOLATION;
            }
        }

        if !NT_SUCCESS(Status) {
            SetLastNtError(Status);
            DC_UnlockDc(dc);
            return 0u8;
        }
    }

    if (*pdcattr).dwLayout & LAYOUT_RTL != 0 {
        XOffset = -XOffset;
    }
    (*pdcattr).ptlViewportOrg.x += XOffset;
    (*pdcattr).ptlViewportOrg.y += YOffset;
    (*pdcattr).flXform |= PAGE_XLATE_CHANGED | WORLD_XFORM_CHANGED | DEVICE_TO_WORLD_INVALID;

    DC_UnlockDc(dc);

    1u8
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn NtGdiOffsetViewportOrgEx(
    hDC: HDC,
    mut XOffset: i32,
    YOffset: i32,
    UnsafePoint: LPPOINT,
) -> u8 {
    let dc: PDC;
    let pdcattr: PDC_ATTR;
    let mut Status: i32 = STATUS_SUCCESS;

    dc = DC_LockDc(hDC);
    if dc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0u8;
    }
    pdcattr = (*dc).pdcattr;

    if !UnsafePoint.is_null() {
        Status = SafeProbeForWrite(UnsafePoint as *mut _, core::mem::size_of::<POINT>() as _, 1);
        if NT_SUCCESS(Status) {
            let mut SafePoint: POINT = core::mem::zeroed();
            SafePoint.x = (*pdcattr).ptlViewportOrg.x;
            SafePoint.y = (*pdcattr).ptlViewportOrg.y;
            if (*pdcattr).dwLayout & LAYOUT_RTL != 0 {
                SafePoint.x = -SafePoint.x;
            }
            // The probe only range-checks (validate-only by design under
            // USE_DUMMY_PSEH); the C wrapped probe AND store in one _SEH2_TRY, so
            // the store itself must be the fault-tolerant copy.
            if !crate::copy::publish_to_user(
                UnsafePoint as *mut core::ffi::c_void,
                core::ptr::addr_of!(SafePoint) as *const core::ffi::c_void,
                core::mem::size_of::<POINT>(),
            ) {
                Status = STATUS_ACCESS_VIOLATION;
            }
        }

        if !NT_SUCCESS(Status) {
            SetLastNtError(Status);
            DC_UnlockDc(dc);
            return 0u8;
        }
    }

    if (*pdcattr).dwLayout & LAYOUT_RTL != 0 {
        XOffset = -XOffset;
    }
    (*pdcattr).ptlViewportOrg.x += XOffset;
    (*pdcattr).ptlViewportOrg.y += YOffset;
    (*pdcattr).flXform |= PAGE_XLATE_CHANGED | WORLD_XFORM_CHANGED | DEVICE_TO_WORLD_INVALID;

    DC_UnlockDc(dc);

    1u8
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn NtGdiOffsetWindowOrgEx(
    hDC: HDC,
    XOffset: i32,
    YOffset: i32,
    Point: LPPOINT,
) -> u32 {

    let dc: PDC = DC_LockDc(hDC);
    if dc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }
    let pdcattr: PDC_ATTR = (*dc).pdcattr;

    if !Point.is_null() {
        let mut Status: i32 = SafeProbeForWrite(
            Point as *mut core::ffi::c_void,
            core::mem::size_of::<POINT>() as u32,
            1,
        );
        if NT_SUCCESS(Status) {
            let mut SafePoint: POINT = core::mem::zeroed();
            SafePoint.x = (*pdcattr).ptlWindowOrg.x;
            SafePoint.y = (*pdcattr).ptlWindowOrg.y;
            // The probe only range-checks (validate-only by design under
            // USE_DUMMY_PSEH); the C wrapped probe AND store in one _SEH2_TRY, so
            // the store itself must be the fault-tolerant copy.
            if !crate::copy::publish_to_user(
                Point as *mut core::ffi::c_void,
                core::ptr::addr_of!(SafePoint) as *const core::ffi::c_void,
                core::mem::size_of::<POINT>(),
            ) {
                Status = STATUS_ACCESS_VIOLATION;
            }
        }

        if !NT_SUCCESS(Status) {
            SetLastNtError(Status);
            DC_UnlockDc(dc);
            return 0;
        }
    }

    (*pdcattr).ptlWindowOrg.x += XOffset;
    (*pdcattr).ptlWindowOrg.y += YOffset;
    (*pdcattr).flXform |= PAGE_XLATE_CHANGED | WORLD_XFORM_CHANGED | DEVICE_TO_WORLD_INVALID;

    DC_UnlockDc(dc);

    1
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn NtGdiOffsetWindowOrgEx(
    hDC: HDC,
    XOffset: i32,
    YOffset: i32,
    Point: LPPOINT,
) -> u32 {

    let dc: PDC = DC_LockDc(hDC);
    if dc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }
    let pdcattr: PDC_ATTR = (*dc).pdcattr;

    if !Point.is_null() {
        let mut Status: i32 = SafeProbeForWrite(
            Point as *mut core::ffi::c_void,
            core::mem::size_of::<POINT>() as _,
            1,
        );
        if NT_SUCCESS(Status) {
            let mut SafePoint: POINT = core::mem::zeroed();
            SafePoint.x = (*pdcattr).ptlWindowOrg.x;
            SafePoint.y = (*pdcattr).ptlWindowOrg.y;
            // The probe only range-checks (validate-only by design under
            // USE_DUMMY_PSEH); the C wrapped probe AND store in one _SEH2_TRY, so
            // the store itself must be the fault-tolerant copy.
            if !crate::copy::publish_to_user(
                Point as *mut core::ffi::c_void,
                core::ptr::addr_of!(SafePoint) as *const core::ffi::c_void,
                core::mem::size_of::<POINT>(),
            ) {
                Status = STATUS_ACCESS_VIOLATION;
            }
        }

        if !NT_SUCCESS(Status) {
            SetLastNtError(Status);
            DC_UnlockDc(dc);
            return 0;
        }
    }

    (*pdcattr).ptlWindowOrg.x += XOffset;
    (*pdcattr).ptlWindowOrg.y += YOffset;
    (*pdcattr).flXform |= PAGE_XLATE_CHANGED | WORLD_XFORM_CHANGED | DEVICE_TO_WORLD_INVALID;

    DC_UnlockDc(dc);

    1
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn NtGdiScaleViewportExtEx(
    hDC: HDC,
    Xnum: i32,
    Xdenom: i32,
    Ynum: i32,
    Ydenom: i32,
    pSize: LPSIZE,
) -> i32 {

    let pDC: PDC;
    let pdcattr: PDC_ATTR;
    let mut Ret: i32 = 0;
    let X: i32;
    let Y: i32;
    let Status: i32;

    pDC = DC_LockDc(hDC);
    if pDC.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }
    pdcattr = (*pDC).pdcattr;

    if (*pdcattr).iMapMode > MM_TWIPS as i32 {
        if Xdenom != 0 && Ydenom != 0 {
            DC_pszlViewportExt(pDC);
            X = Xnum * (*pdcattr).szlViewportExt.cx / Xdenom;
            if X != 0 {
                Y = Ynum * (*pdcattr).szlViewportExt.cy / Ydenom;
                if Y != 0 {
                    (*pdcattr).szlViewportExt.cx = X;
                    (*pdcattr).szlViewportExt.cy = Y;
                    (*pdcattr).flXform |= PAGE_XLATE_CHANGED;

                    IntMirrorWindowOrg(pDC);

                    (*pdcattr).flXform |= PAGE_EXTENTS_CHANGED
                        | INVALIDATE_ATTRIBUTES
                        | WORLD_XFORM_CHANGED
                        | DEVICE_TO_WORLD_INVALID;

                    if (*pdcattr).iMapMode == MM_ISOTROPIC as i32 {
                        DC_vFixIsotropicMapping(pDC);
                    }

                    Ret = 1;
                }
            }
        }
    } else {
        Ret = 1;
    }

    if !pSize.is_null() {
        Status = SafeProbeForWrite(
            pSize as *mut core::ffi::c_void,
            core::mem::size_of::<SIZE>() as u32,
            1,
        );
        if !NT_SUCCESS(Status) {
            SetLastNtError(Status);
            Ret = 0;
        } else {
            let mut SafeSize: SIZE = core::mem::zeroed();
            SafeSize.cx = (*pdcattr).szlViewportExt.cx;
            SafeSize.cy = (*pdcattr).szlViewportExt.cy;
            // The probe only range-checks (validate-only by design under
            // USE_DUMMY_PSEH); the C wrapped probe AND store in one _SEH2_TRY, so
            // the store itself must be the fault-tolerant copy.
            if !crate::copy::publish_to_user(
                pSize as *mut core::ffi::c_void,
                core::ptr::addr_of!(SafeSize) as *const core::ffi::c_void,
                core::mem::size_of::<SIZE>(),
            ) {
                SetLastNtError(STATUS_ACCESS_VIOLATION);
                Ret = 0;
            }
        }
    }

    DC_UnlockDc(pDC);
    Ret
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn NtGdiScaleViewportExtEx(
    hDC: HDC,
    Xnum: i32,
    Xdenom: i32,
    Ynum: i32,
    Ydenom: i32,
    pSize: LPSIZE,
) -> i32 {

    let pDC: PDC;
    let pdcattr: PDC_ATTR;
    let mut Ret: i32 = 0;
    let X: i32;
    let Y: i32;
    let Status: i32;

    pDC = DC_LockDc(hDC);
    if pDC.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }
    pdcattr = (*pDC).pdcattr;

    if (*pdcattr).iMapMode > MM_TWIPS as i32 {
        if Xdenom != 0 && Ydenom != 0 {
            DC_pszlViewportExt(pDC);
            X = Xnum * (*pdcattr).szlViewportExt.cx / Xdenom;
            if X != 0 {
                Y = Ynum * (*pdcattr).szlViewportExt.cy / Ydenom;
                if Y != 0 {
                    (*pdcattr).szlViewportExt.cx = X;
                    (*pdcattr).szlViewportExt.cy = Y;
                    (*pdcattr).flXform |= PAGE_XLATE_CHANGED;

                    IntMirrorWindowOrg(pDC);

                    (*pdcattr).flXform |= PAGE_EXTENTS_CHANGED
                        | INVALIDATE_ATTRIBUTES
                        | WORLD_XFORM_CHANGED
                        | DEVICE_TO_WORLD_INVALID;

                    if (*pdcattr).iMapMode == MM_ISOTROPIC as i32 {
                        DC_vFixIsotropicMapping(pDC);
                    }

                    Ret = 1;
                }
            }
        }
    } else {
        Ret = 1;
    }

    if !pSize.is_null() {
        Status = SafeProbeForWrite(
            pSize as *mut core::ffi::c_void,
            core::mem::size_of::<SIZE>() as _,
            1,
        );
        if !NT_SUCCESS(Status) {
            SetLastNtError(Status);
            Ret = 0;
        } else {
            let mut SafeSize: SIZE = core::mem::zeroed();
            SafeSize.cx = (*pdcattr).szlViewportExt.cx;
            SafeSize.cy = (*pdcattr).szlViewportExt.cy;
            // The probe only range-checks (validate-only by design under
            // USE_DUMMY_PSEH); the C wrapped probe AND store in one _SEH2_TRY, so
            // the store itself must be the fault-tolerant copy.
            if !crate::copy::publish_to_user(
                pSize as *mut core::ffi::c_void,
                core::ptr::addr_of!(SafeSize) as *const core::ffi::c_void,
                core::mem::size_of::<SIZE>(),
            ) {
                SetLastNtError(STATUS_ACCESS_VIOLATION);
                Ret = 0;
            }
        }
    }

    DC_UnlockDc(pDC);
    Ret
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn NtGdiScaleWindowExtEx(
    hDC: HDC,
    Xnum: i32,
    Xdenom: i32,
    Ynum: i32,
    Ydenom: i32,
    pSize: LPSIZE,
) -> u8 {
    let pDC: PDC;
    let pdcattr: PDC_ATTR;
    let mut Ret: u8 = 0u8;
    let mut X: i32;
    let Y: i32;

    pDC = DC_LockDc(hDC);
    if pDC.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0u8;
    }
    pdcattr = (*pDC).pdcattr;

    if !pSize.is_null() {
        let mut Status =
            SafeProbeForWrite(pSize as *mut core::ffi::c_void, core::mem::size_of::<SIZE>() as u32, 1);
        if NT_SUCCESS(Status) {
            X = (*pdcattr).szlWindowExt.cx;
            if ((*pdcattr).dwLayout & LAYOUT_RTL) != 0 {
                X = -X;
            }
            let mut SafeSize: SIZE = core::mem::zeroed();
            SafeSize.cx = X;
            SafeSize.cy = (*pdcattr).szlWindowExt.cy;
            // The probe only range-checks (validate-only by design under
            // USE_DUMMY_PSEH); the C wrapped probe AND store in one _SEH2_TRY, so
            // the store itself must be the fault-tolerant copy.
            if !crate::copy::publish_to_user(
                pSize as *mut core::ffi::c_void,
                core::ptr::addr_of!(SafeSize) as *const core::ffi::c_void,
                core::mem::size_of::<SIZE>(),
            ) {
                Status = STATUS_ACCESS_VIOLATION;
            }
        }

        if !NT_SUCCESS(Status) {
            SetLastNtError(Status);
            DC_UnlockDc(pDC);
            return 0u8;
        }
    }

    if (*pdcattr).iMapMode > MM_TWIPS as i32 {
        if Xdenom != 0 && Ydenom != 0 {
            X = Xnum * (*pdcattr).szlWindowExt.cx / Xdenom;
            if X != 0 {
                Y = Ynum * (*pdcattr).szlWindowExt.cy / Ydenom;
                if Y != 0 {
                    (*pdcattr).szlWindowExt.cx = X;
                    (*pdcattr).szlWindowExt.cy = Y;

                    IntMirrorWindowOrg(pDC);

                    (*pdcattr).flXform |= PAGE_EXTENTS_CHANGED
                        | INVALIDATE_ATTRIBUTES
                        | WORLD_XFORM_CHANGED
                        | DEVICE_TO_WORLD_INVALID;

                    Ret = 1u8;
                }
            }
        }
    } else {
        Ret = 1u8;
    }

    DC_UnlockDc(pDC);
    Ret
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn NtGdiScaleWindowExtEx(
    hDC: HDC,
    Xnum: i32,
    Xdenom: i32,
    Ynum: i32,
    Ydenom: i32,
    pSize: LPSIZE,
) -> u8 {
    let pDC: PDC;
    let pdcattr: PDC_ATTR;
    let mut Ret: u8 = 0u8;
    let mut X: i32;
    let Y: i32;

    pDC = DC_LockDc(hDC);
    if pDC.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0u8;
    }
    pdcattr = (*pDC).pdcattr;

    if !pSize.is_null() {
        let mut Status =
            SafeProbeForWrite(pSize as *mut core::ffi::c_void, core::mem::size_of::<SIZE>() as _, 1);
        if NT_SUCCESS(Status) {
            X = (*pdcattr).szlWindowExt.cx;
            if ((*pdcattr).dwLayout & LAYOUT_RTL) != 0 {
                X = -X;
            }
            let mut SafeSize: SIZE = core::mem::zeroed();
            SafeSize.cx = X;
            SafeSize.cy = (*pdcattr).szlWindowExt.cy;
            // The probe only range-checks (validate-only by design under
            // USE_DUMMY_PSEH); the C wrapped probe AND store in one _SEH2_TRY, so
            // the store itself must be the fault-tolerant copy.
            if !crate::copy::publish_to_user(
                pSize as *mut core::ffi::c_void,
                core::ptr::addr_of!(SafeSize) as *const core::ffi::c_void,
                core::mem::size_of::<SIZE>(),
            ) {
                Status = STATUS_ACCESS_VIOLATION;
            }
        }

        if !NT_SUCCESS(Status) {
            SetLastNtError(Status);
            DC_UnlockDc(pDC);
            return 0u8;
        }
    }

    if (*pdcattr).iMapMode > MM_TWIPS as i32 {
        if Xdenom != 0 && Ydenom != 0 {
            X = Xnum * (*pdcattr).szlWindowExt.cx / Xdenom;
            if X != 0 {
                Y = Ynum * (*pdcattr).szlWindowExt.cy / Ydenom;
                if Y != 0 {
                    (*pdcattr).szlWindowExt.cx = X;
                    (*pdcattr).szlWindowExt.cy = Y;

                    IntMirrorWindowOrg(pDC);

                    (*pdcattr).flXform |= PAGE_EXTENTS_CHANGED
                        | INVALIDATE_ATTRIBUTES
                        | WORLD_XFORM_CHANGED
                        | DEVICE_TO_WORLD_INVALID;

                    Ret = 1u8;
                }
            }
        }
    } else {
        Ret = 1u8;
    }

    DC_UnlockDc(pDC);
    Ret
}
#[no_mangle]
pub unsafe extern "system" fn IntGdiSetMapMode(dc: PDC, MapMode: i32) -> i32 {
    let iPrevMapMode: i32;
    let mut flXform: FLONG;
    let pdcattr: PDC_ATTR = (*dc).pdcattr;

    if MapMode == (*pdcattr).iMapMode {
        return MapMode;
    }

    flXform = (*pdcattr).flXform & !(ISO_OR_ANISO_MAP_MODE | PTOD_EFM22_NEGATIVE
        | PTOD_EFM11_NEGATIVE | POSITIVE_Y_IS_UP | PAGE_TO_DEVICE_SCALE_IDENTITY
        | PAGE_TO_DEVICE_IDENTITY);

    match MapMode as u32 {
        MM_TEXT => {
            (*pdcattr).szlWindowExt.cx = 1;
            (*pdcattr).szlWindowExt.cy = 1;
            (*pdcattr).szlViewportExt.cx = 1;
            (*pdcattr).szlViewportExt.cy = 1;
            flXform |= PAGE_TO_DEVICE_SCALE_IDENTITY;
        }
        MM_ISOTROPIC => {
            flXform |= ISO_OR_ANISO_MAP_MODE;
            (*pdcattr).szlWindowExt.cx = (*pdcattr).szlVirtualDeviceMm.cx * 10;
            (*pdcattr).szlWindowExt.cy = (*pdcattr).szlVirtualDeviceMm.cy * 10;
            (*pdcattr).szlViewportExt.cx = (*pdcattr).szlVirtualDevicePixel.cx;
            (*pdcattr).szlViewportExt.cy = -(*pdcattr).szlVirtualDevicePixel.cy;
        }
        MM_LOMETRIC => {
            (*pdcattr).szlWindowExt.cx = (*pdcattr).szlVirtualDeviceMm.cx * 10;
            (*pdcattr).szlWindowExt.cy = (*pdcattr).szlVirtualDeviceMm.cy * 10;
            (*pdcattr).szlViewportExt.cx = (*pdcattr).szlVirtualDevicePixel.cx;
            (*pdcattr).szlViewportExt.cy = -(*pdcattr).szlVirtualDevicePixel.cy;
        }
        MM_HIMETRIC => {
            (*pdcattr).szlWindowExt.cx = (*pdcattr).szlVirtualDeviceMm.cx * 100;
            (*pdcattr).szlWindowExt.cy = (*pdcattr).szlVirtualDeviceMm.cy * 100;
            (*pdcattr).szlViewportExt.cx = (*pdcattr).szlVirtualDevicePixel.cx;
            (*pdcattr).szlViewportExt.cy = -(*pdcattr).szlVirtualDevicePixel.cy;
        }
        MM_LOENGLISH => {
            (*pdcattr).szlWindowExt.cx = EngMulDiv(1000, (*pdcattr).szlVirtualDeviceMm.cx, 254);
            (*pdcattr).szlWindowExt.cy = EngMulDiv(1000, (*pdcattr).szlVirtualDeviceMm.cy, 254);
            (*pdcattr).szlViewportExt.cx = (*pdcattr).szlVirtualDevicePixel.cx;
            (*pdcattr).szlViewportExt.cy = -(*pdcattr).szlVirtualDevicePixel.cy;
        }
        MM_HIENGLISH => {
            (*pdcattr).szlWindowExt.cx = EngMulDiv(10000, (*pdcattr).szlVirtualDeviceMm.cx, 254);
            (*pdcattr).szlWindowExt.cy = EngMulDiv(10000, (*pdcattr).szlVirtualDeviceMm.cy, 254);
            (*pdcattr).szlViewportExt.cx = (*pdcattr).szlVirtualDevicePixel.cx;
            (*pdcattr).szlViewportExt.cy = -(*pdcattr).szlVirtualDevicePixel.cy;
        }
        MM_TWIPS => {
            (*pdcattr).szlWindowExt.cx = EngMulDiv(14400, (*pdcattr).szlVirtualDeviceMm.cx, 254);
            (*pdcattr).szlWindowExt.cy = EngMulDiv(14400, (*pdcattr).szlVirtualDeviceMm.cy, 254);
            (*pdcattr).szlViewportExt.cx = (*pdcattr).szlVirtualDevicePixel.cx;
            (*pdcattr).szlViewportExt.cy = -(*pdcattr).szlVirtualDevicePixel.cy;
        }
        MM_ANISOTROPIC => {
            flXform &= !(PAGE_TO_DEVICE_IDENTITY | POSITIVE_Y_IS_UP);
            flXform |= ISO_OR_ANISO_MAP_MODE;
        }
        _ => return 0,
    }

    iPrevMapMode = (*pdcattr).iMapMode;
    (*pdcattr).iMapMode = MapMode;

    (*pdcattr).flXform = flXform | (PAGE_XLATE_CHANGED | PAGE_EXTENTS_CHANGED
        | INVALIDATE_ATTRIBUTES | DEVICE_TO_PAGE_INVALID
        | WORLD_XFORM_CHANGED | DEVICE_TO_WORLD_INVALID);

    iPrevMapMode
}

#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe extern "fastcall" fn GreSetViewportOrgEx(
    hDC: HDC,
    X: core::ffi::c_int,
    Y: core::ffi::c_int,
    Point: LPPOINT,
) -> u32 {
    GreSetViewportOrgEx_impl(hDC, X, Y, Point)
}

#[no_mangle]
#[cfg(not(target_arch = "x86"))]
pub unsafe extern "C" fn GreSetViewportOrgEx(
    hDC: HDC,
    X: core::ffi::c_int,
    Y: core::ffi::c_int,
    Point: LPPOINT,
) -> u32 {
    GreSetViewportOrgEx_impl(hDC, X, Y, Point)
}

unsafe fn GreSetViewportOrgEx_impl(
    hDC: HDC,
    X: core::ffi::c_int,
    Y: core::ffi::c_int,
    Point: LPPOINT,
) -> u32 {
    let dc: PDC = DC_LockDc(hDC);
    if dc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }
    let pdcattr: PDC_ATTR = (*dc).pdcattr;

    if !Point.is_null() {
        (*Point).x = (*pdcattr).ptlViewportOrg.x;
        (*Point).y = (*pdcattr).ptlViewportOrg.y;
    }

    (*pdcattr).ptlViewportOrg.x = X;
    (*pdcattr).ptlViewportOrg.y = Y;
    (*pdcattr).flXform |= (PAGE_XLATE_CHANGED | WORLD_XFORM_CHANGED | DEVICE_TO_WORLD_INVALID) as core::ffi::c_ulong;

    DC_UnlockDc(dc);
    1
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn NtGdiSetViewportOrgEx(
    hDC: HDC,
    X: core::ffi::c_int,
    Y: core::ffi::c_int,
    Point: LPPOINT,
) -> u32 {

    let dc: PDC = DC_LockDc(hDC);
    if dc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }
    let pdcattr: PDC_ATTR = (*dc).pdcattr;

    if !Point.is_null() {
        let mut Status: i32 = STATUS_SUCCESS;

        Status = SafeProbeForWrite(
            Point as *mut core::ffi::c_void,
            core::mem::size_of::<POINT>() as u32,
            1,
        );
        if NT_SUCCESS(Status) {
            let mut SafePoint: POINT = core::mem::zeroed();
            SafePoint.x = (*pdcattr).ptlViewportOrg.x;
            SafePoint.y = (*pdcattr).ptlViewportOrg.y;
            // The probe only range-checks (validate-only by design under
            // USE_DUMMY_PSEH); the C wrapped probe AND store in one _SEH2_TRY, so
            // the store itself must be the fault-tolerant copy.
            if !crate::copy::publish_to_user(
                Point as *mut core::ffi::c_void,
                core::ptr::addr_of!(SafePoint) as *const core::ffi::c_void,
                core::mem::size_of::<POINT>(),
            ) {
                Status = STATUS_ACCESS_VIOLATION;
            }
        }

        if !NT_SUCCESS(Status) {
            SetLastNtError(Status);
            DC_UnlockDc(dc);
            return 0;
        }
    }

    (*pdcattr).ptlViewportOrg.x = X;
    (*pdcattr).ptlViewportOrg.y = Y;
    (*pdcattr).flXform |= PAGE_XLATE_CHANGED | WORLD_XFORM_CHANGED | DEVICE_TO_WORLD_INVALID;

    DC_UnlockDc(dc);

    1
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn NtGdiSetViewportOrgEx(
    hDC: HDC,
    X: core::ffi::c_int,
    Y: core::ffi::c_int,
    Point: LPPOINT,
) -> u32 {

    let dc: PDC = DC_LockDc(hDC);
    if dc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }
    let pdcattr: PDC_ATTR = (*dc).pdcattr;

    if !Point.is_null() {
        let mut Status: i32 = STATUS_SUCCESS;

        Status = SafeProbeForWrite(
            Point as *mut core::ffi::c_void,
            core::mem::size_of::<POINT>() as _,
            1,
        );
        if NT_SUCCESS(Status) {
            let mut SafePoint: POINT = core::mem::zeroed();
            SafePoint.x = (*pdcattr).ptlViewportOrg.x;
            SafePoint.y = (*pdcattr).ptlViewportOrg.y;
            // The probe only range-checks (validate-only by design under
            // USE_DUMMY_PSEH); the C wrapped probe AND store in one _SEH2_TRY, so
            // the store itself must be the fault-tolerant copy.
            if !crate::copy::publish_to_user(
                Point as *mut core::ffi::c_void,
                core::ptr::addr_of!(SafePoint) as *const core::ffi::c_void,
                core::mem::size_of::<POINT>(),
            ) {
                Status = STATUS_ACCESS_VIOLATION;
            }
        }

        if !NT_SUCCESS(Status) {
            SetLastNtError(Status);
            DC_UnlockDc(dc);
            return 0;
        }
    }

    (*pdcattr).ptlViewportOrg.x = X;
    (*pdcattr).ptlViewportOrg.y = Y;
    (*pdcattr).flXform |= PAGE_XLATE_CHANGED | WORLD_XFORM_CHANGED | DEVICE_TO_WORLD_INVALID;

    DC_UnlockDc(dc);

    1
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn NtGdiSetWindowOrgEx(
    hDC: HDC,
    X: core::ffi::c_int,
    Y: core::ffi::c_int,
    Point: LPPOINT,
) -> u32 {

    let dc: PDC = DC_LockDc(hDC);
    if dc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }
    let pdcattr: PDC_ATTR = (*dc).pdcattr;

    if !Point.is_null() {
        let mut Status: i32 = SafeProbeForWrite(
            Point as *mut core::ffi::c_void,
            core::mem::size_of::<POINT>() as u32,
            1,
        );
        if NT_SUCCESS(Status) {
            let mut SafePoint: POINT = core::mem::zeroed();
            SafePoint.x = (*pdcattr).ptlWindowOrg.x;
            SafePoint.y = (*pdcattr).ptlWindowOrg.y;
            // The probe only range-checks (validate-only by design under
            // USE_DUMMY_PSEH); the C wrapped probe AND store in one _SEH2_TRY, so
            // the store itself must be the fault-tolerant copy.
            if !crate::copy::publish_to_user(
                Point as *mut core::ffi::c_void,
                core::ptr::addr_of!(SafePoint) as *const core::ffi::c_void,
                core::mem::size_of::<POINT>(),
            ) {
                Status = STATUS_ACCESS_VIOLATION;
            }
        }

        if !NT_SUCCESS(Status) {
            SetLastNtError(Status);
            DC_UnlockDc(dc);
            return 0;
        }
    }

    (*pdcattr).ptlWindowOrg.x = X;
    (*pdcattr).ptlWindowOrg.y = Y;
    (*pdcattr).flXform |= PAGE_XLATE_CHANGED | WORLD_XFORM_CHANGED | DEVICE_TO_WORLD_INVALID;

    DC_UnlockDc(dc);

    1
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn NtGdiSetWindowOrgEx(
    hDC: HDC,
    X: core::ffi::c_int,
    Y: core::ffi::c_int,
    Point: LPPOINT,
) -> u32 {

    let dc: PDC = DC_LockDc(hDC);
    if dc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }
    let pdcattr: PDC_ATTR = (*dc).pdcattr;

    if !Point.is_null() {
        let mut Status: i32 = SafeProbeForWrite(
            Point as *mut core::ffi::c_void,
            core::mem::size_of::<POINT>() as _,
            1,
        );
        if NT_SUCCESS(Status) {
            let mut SafePoint: POINT = core::mem::zeroed();
            SafePoint.x = (*pdcattr).ptlWindowOrg.x;
            SafePoint.y = (*pdcattr).ptlWindowOrg.y;
            // The probe only range-checks (validate-only by design under
            // USE_DUMMY_PSEH); the C wrapped probe AND store in one _SEH2_TRY, so
            // the store itself must be the fault-tolerant copy.
            if !crate::copy::publish_to_user(
                Point as *mut core::ffi::c_void,
                core::ptr::addr_of!(SafePoint) as *const core::ffi::c_void,
                core::mem::size_of::<POINT>(),
            ) {
                Status = STATUS_ACCESS_VIOLATION;
            }
        }

        if !NT_SUCCESS(Status) {
            SetLastNtError(Status);
            DC_UnlockDc(dc);
            return 0;
        }
    }

    (*pdcattr).ptlWindowOrg.x = X;
    (*pdcattr).ptlWindowOrg.y = Y;
    (*pdcattr).flXform |= PAGE_XLATE_CHANGED | WORLD_XFORM_CHANGED | DEVICE_TO_WORLD_INVALID;

    DC_UnlockDc(dc);

    1
}

#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe extern "fastcall" fn IntMirrorWindowOrg(dc: PDC) {
    IntMirrorWindowOrgImpl(dc);
}

#[no_mangle]
#[cfg(not(target_arch = "x86"))]
pub unsafe extern "C" fn IntMirrorWindowOrg(dc: PDC) {
    IntMirrorWindowOrgImpl(dc);
}

unsafe fn IntMirrorWindowOrgImpl(dc: PDC) {
    let pdcattr: PDC_ATTR = (*dc).pdcattr;

    if ((*pdcattr).dwLayout & LAYOUT_RTL as u32) == 0 {
        (*pdcattr).ptlWindowOrg.x = (*pdcattr).lWindowOrgx;
        return;
    }

    let cx: LONG = (*pdcattr).szlViewportExt.cx;
    if cx == 0 {
        return;
    }

    let mut X: LONG = ((*dc).erclWindow.right - (*dc).erclWindow.left) - 1;

    X = (X * (*pdcattr).szlWindowExt.cx) / cx;

    (*pdcattr).ptlWindowOrg.x = (*pdcattr).lWindowOrgx - X;
    (*pdcattr).flXform |= (PAGE_XLATE_CHANGED | WORLD_XFORM_CHANGED | DEVICE_TO_WORLD_INVALID) as u32;
}

#[no_mangle]
pub unsafe extern "system" fn DC_vSetLayout(
    pdc: PDC,
    _wox: LONG,
    dwLayout: DWORD,
) {
    let pdcattr: PDC_ATTR = (*pdc).pdcattr;

    (*pdcattr).dwLayout = dwLayout;

    if (dwLayout & LAYOUT_ORIENTATIONMASK) == 0 {
        return;
    }

    if (dwLayout & LAYOUT_RTL) != 0 {
        (*pdcattr).iMapMode = MM_ANISOTROPIC as i32;
    }

    if ((*pdcattr).flTextAlign & TA_CENTER) == 0 {
        (*pdcattr).flTextAlign |= TA_RIGHT;
    }

    if ((*pdc).dclevel.flPath & DCPATH_CLOCKWISE) != 0 {
        (*pdc).dclevel.flPath &= !DCPATH_CLOCKWISE;
    } else {
        (*pdc).dclevel.flPath |= DCPATH_CLOCKWISE;
    }

    (*pdcattr).flXform |= PAGE_EXTENTS_CHANGED
        | INVALIDATE_ATTRIBUTES
        | WORLD_XFORM_CHANGED
        | DEVICE_TO_WORLD_INVALID;
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiSetLayout(
    hdc: HDC,
    wox: LONG,
    dwLayout: DWORD,
) -> DWORD {
    let pdc: PDC = DC_LockDc(hdc);
    if pdc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return GDI_ERROR;
    }

    let dwOldLayout: DWORD = (*(*pdc).pdcattr).dwLayout;
    DC_vSetLayout(pdc, wox, dwLayout);

    DC_UnlockDc(pdc);
    dwOldLayout
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiGetDeviceWidth(hdc: HDC) -> LONG {
    let dc: PDC;
    let Ret: LONG;
    dc = DC_LockDc(hdc);
    if dc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }
    Ret = (*dc).erclWindow.right - (*dc).erclWindow.left;
    DC_UnlockDc(dc);
    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiMirrorWindowOrg(hdc: HDC) -> BOOL {
    let dc: PDC = DC_LockDc(hdc);
    if dc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }
    IntMirrorWindowOrg(dc);
    DC_UnlockDc(dc);
    1
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiSetSizeDevice(
    hdc: HDC,
    cxVirtualDevice: i32,
    cyVirtualDevice: i32,
) -> u32 {
    let dc: PDC;
    let pdcattr: PDC_ATTR;

    if cxVirtualDevice == 0 || cyVirtualDevice == 0 {
        return 0;
    }

    dc = DC_LockDc(hdc);
    if dc.is_null() {
        return 0;
    }

    pdcattr = (*dc).pdcattr;

    (*pdcattr).szlVirtualDeviceSize.cx = cxVirtualDevice;
    (*pdcattr).szlVirtualDeviceSize.cy = cyVirtualDevice;

    DC_UnlockDc(dc);

    1
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiSetVirtualResolution(
    hdc: HDC,
    mut cxVirtualDevicePixel: i32,
    mut cyVirtualDevicePixel: i32,
    mut cxVirtualDeviceMm: i32,
    mut cyVirtualDeviceMm: i32,
) -> u32 {
    let dc: PDC;
    let pdcattr: PDC_ATTR;

    if cxVirtualDevicePixel == 0 && cyVirtualDevicePixel == 0 &&
       cxVirtualDeviceMm == 0 && cyVirtualDeviceMm == 0
    {
        cxVirtualDevicePixel = NtGdiGetDeviceCaps(hdc, HORZRES as i32);
        cyVirtualDevicePixel = NtGdiGetDeviceCaps(hdc, VERTRES as i32);
        cxVirtualDeviceMm = NtGdiGetDeviceCaps(hdc, HORZSIZE as i32);
        cyVirtualDeviceMm = NtGdiGetDeviceCaps(hdc, VERTSIZE as i32);
    }
    else if cxVirtualDevicePixel == 0 || cyVirtualDevicePixel == 0 ||
            cxVirtualDeviceMm == 0 || cyVirtualDeviceMm == 0
    {
        return 0;
    }

    dc = DC_LockDc(hdc);
    if dc.is_null() {
        return 0;
    }

    pdcattr = (*dc).pdcattr;

    (*pdcattr).szlVirtualDevicePixel.cx = cxVirtualDevicePixel;
    (*pdcattr).szlVirtualDevicePixel.cy = cyVirtualDevicePixel;
    (*pdcattr).szlVirtualDeviceMm.cx = cxVirtualDeviceMm;
    (*pdcattr).szlVirtualDeviceMm.cy = cyVirtualDeviceMm;

    DC_UnlockDc(dc);
    1
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn DC_vGetAspectRatioFilter(pDC: PDC, AspectRatio: LPSIZE) {
    DC_vGetAspectRatioFilter_impl(pDC, AspectRatio)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn DC_vGetAspectRatioFilter(pDC: PDC, AspectRatio: LPSIZE) {
    DC_vGetAspectRatioFilter_impl(pDC, AspectRatio)
}

unsafe fn DC_vGetAspectRatioFilter_impl(pDC: PDC, AspectRatio: LPSIZE) {
    if (*(*pDC).pdcattr).flFontMapper & 1 != 0 {
        (*AspectRatio).cx = (*(*pDC).ppdev).gdiinfo.ulLogPixelsX as i32;
        (*AspectRatio).cy = (*(*pDC).ppdev).gdiinfo.ulLogPixelsY as i32;
    } else {
        (*AspectRatio).cx = 0;
        (*AspectRatio).cy = 0;
    }
}
#[no_mangle]
pub unsafe extern "system" fn GreGetDCPoint(
    hDC: HDC,
    iPoint: UINT,
    Point: PPOINTL,
) -> BOOL {
    let mut Ret: BOOL = 1;
    let pdc: *mut DC;
    let mut Size: SIZE = core::mem::zeroed();
    let pszlViewportExt: PSIZEL;

    if Point.is_null() {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    pdc = DC_LockDc(hDC);
    if pdc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }

    match iPoint {
        GdiGetViewPortExt => {
            pszlViewportExt = DC_pszlViewportExt(pdc);
            (*Point).x = (*pszlViewportExt).cx;
            (*Point).y = (*pszlViewportExt).cy;
        }
        GdiGetWindowExt => {
            (*Point).x = (*(*pdc).pdcattr).szlWindowExt.cx;
            (*Point).y = (*(*pdc).pdcattr).szlWindowExt.cy;
        }
        GdiGetViewPortOrg => {
            *Point = (*(*pdc).pdcattr).ptlViewportOrg;
        }
        GdiGetWindowOrg => {
            *Point = (*(*pdc).pdcattr).ptlWindowOrg;
        }
        GdiGetDCOrg => {
            *Point = (*pdc).ptlDCOrig;
        }
        GdiGetAspectRatioFilter => {
            DC_vGetAspectRatioFilter(pdc, &mut Size);
            (*Point).x = Size.cx;
            (*Point).y = Size.cy;
        }
        _ => {
            EngSetLastError(ERROR_INVALID_PARAMETER);
            Ret = 0;
        }
    }

    DC_UnlockDc(pdc);
    Ret
}
#[no_mangle]
pub unsafe extern "system" fn GreSetDCOrg(
    hdc: HDC,
    x: LONG,
    y: LONG,
    Rect: PRECTL,
) -> BOOL {
    let dc: PDC = DC_LockDc(hdc);
    if dc.is_null() {
        return FALSE as BOOL;
    }

    (*dc).ptlDCOrig.x = x;
    (*dc).ptlDCOrig.y = y;

    (*dc).ptlFillOrigin.x = (*dc).dclevel.ptlBrushOrigin.x + x;
    (*dc).ptlFillOrigin.y = (*dc).dclevel.ptlBrushOrigin.y + y;

    if !Rect.is_null() {
        (*dc).erclWindow = *Rect;
    }

    DC_UnlockDc(dc);
    TRUE as BOOL
}
#[no_mangle]
pub unsafe extern "system" fn GreGetDCOrgEx(
    hdc: HDC,
    Point: PPOINTL,
    Rect: PRECTL,
) -> BOOL {
    let dc: PDC;

    dc = DC_LockDc(hdc);
    if dc.is_null() {
        return 0;
    }

    *Rect = (*dc).erclWindow;

    DC_UnlockDc(dc);

    GreGetDCPoint(hdc, GdiGetDCOrg, Point)
}
#[no_mangle]
pub unsafe extern "system" fn GreGetWindowExtEx(
    hdc: HDC,
    lpSize: LPSIZE,
) -> BOOL {
    GreGetDCPoint(hdc, GdiGetWindowExt, lpSize as PPOINTL)
}
#[no_mangle]
pub unsafe extern "system" fn GreGetViewportExtEx(
    hdc: HDC,
    lpSize: LPSIZE,
) -> BOOL {
    GreGetDCPoint(hdc, GdiGetViewPortExt, lpSize as PPOINTL)
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn NtGdiGetDCPoint(
    hDC: HDC,
    iPoint: u32,
    Point: PPOINTL,
) -> i32 {
    let mut Ret: i32;
    let mut SafePoint: POINTL = core::mem::zeroed();
    let Status: i32;

    if Point.is_null() {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    Ret = GreGetDCPoint(hDC, iPoint, &mut SafePoint);
    if Ret != 0 {
        Status = SafeProbeForWrite(Point as *mut _, core::mem::size_of::<POINT>() as u32, 1);
        if !NT_SUCCESS(Status) {
            Ret = 0;
        } else if !crate::copy::publish_to_user(
            Point as *mut core::ffi::c_void,
            core::ptr::addr_of!(SafePoint) as *const core::ffi::c_void,
            core::mem::size_of::<POINT>(),
        ) {
            // Probe is validate-only; the C had probe+store inside one _SEH2_TRY.
            Ret = 0;
        }
    }

    Ret
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn NtGdiGetDCPoint(
    hDC: HDC,
    iPoint: u32,
    Point: PPOINTL,
) -> i32 {
    let mut Ret: i32;
    let mut SafePoint: POINTL = core::mem::zeroed();
    let Status: i32;

    if Point.is_null() {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    Ret = GreGetDCPoint(hDC, iPoint, &mut SafePoint);
    if Ret != 0 {
        Status = SafeProbeForWrite(Point as *mut _, core::mem::size_of::<POINT>() as _, 1);
        if !NT_SUCCESS(Status) {
            Ret = 0;
        } else if !crate::copy::publish_to_user(
            Point as *mut core::ffi::c_void,
            core::ptr::addr_of!(SafePoint) as *const core::ffi::c_void,
            core::mem::size_of::<POINT>(),
        ) {
            // Probe is validate-only; the C had probe+store inside one _SEH2_TRY.
            Ret = 0;
        }
    }

    Ret
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:        GNU GPL, See COPYING in the top level directory
//  * PROJECT:          ReactOS kernel
//  * PURPOSE:          Coordinate systems
//  * FILE:             win32ss/gdi/ntgdi/coord.c
//  * PROGRAMERS:       Timo Kreuzer (timo.kreuzer@rectos.org)
//  *                   Katayama Hirofumi MZ (katayama.hirofumi.mz@gmail.com)
//  */
//
// /* Coordinate translation overview
//  * -------------------------------
//  *
//  * Windows uses 3 different coordinate systems, referred to as world space,
//  * page space and device space.
//  *
//  * Device space:
//  * This is the coordinate system of the physical device that displays the
//  * graphics. One unit matches one pixel of the surface. The coordinate system
//  * is always orthogonal.
//  *
//  * Page space:
//  * This is the coordinate system on the screen or on the paper layout for
//  * printer devices. The coordinate system is also orthogonal but one unit
//  * does not necessarily match one pixel. Instead there are different mapping
//  * modes that can be set using SetMapMode() that specify how page space units
//  * are transformed into device space units. These mapping modes are:
//  * - MM_TEXT: One unit matches one unit in device space (one pixel)
//  * - MM_TWIPS One unit matches 1/20 point (1/1440 inch)
//  * - MM_LOMETRIC: One unit matches 0.1 millimeter
//  * - MM_HIMETRIC: One unit matches 0.01 millimeter
//  * - MM_LOENGLISH: One unit matches 0.01 inch
//  * - MM_HIENGLISH: One unit matches 0.001 inch
//  * - MM_ISOTROPIC:
//  * - MM_ANISOTROPIC:
//  * If the mapping mode is either MM_ISOTROPIC or MM_ANISOTROPIC, the actual
//  * transformation is calculated from the window and viewport extension.
//  * The window extension can be set using SetWindowExtEx() and describes the
//  * extents of an arbitrary window (not to confuse with the gui element!) in
//  * page space coordinates.
//  * The viewport extension can be set using SetViewportExtEx() and describes
//  * the extent of the same window in device space coordinates. If the mapping
//  * mode is MM_ISOTROPIC one of the viewport extensions can be adjusted by GDI
//  * to make sure the mapping stays isotropic, i.e. that it has the same x/y
//  * ratio as the window extension.
//  *
//  * World space:
//  * World space is the coordinate system that is used for all GDI drawing
//  * operations. The metrics of this coordinate system depend on the DCs
//  * graphics mode, which can be set using SetGraphicsMode().
//  * If the graphics mode is GM_COMPATIBLE, world space is identical to page
//  * space and no additional transformation is applied.
//  * If the graphics mode is GM_ADVANCED, an arbitrary coordinate transformation
//  * can be set using SetWorldTransform(), which is applied to transform world
//  * space coordinates into page space coordinates.
//  *
//  * User mode data:
//  * All coordinate translation data is stored in the DC attribute, so the values
//  * might be invalid. This has to be taken into account. Values might also be
//  * zero, so when a division is made, the value has to be read first and then
//  * checked! This is true for both integer and floating point values, even if
//  * we cannot get floating point exceptions on x86, we can get them on all other
//  * architectures that use the FPU directly instead of emulation.
//  * The result of all operations might be completely random and invalid, if it was
//  * messed with in an illegal way in user mode. This is not a problem, since the
//  * result of coordinate transformations are never expected to be "valid" values.
//  * In the worst case, the drawing operation draws rubbish into the DC.
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <win32k.h>
// #include <safe_user_io.h>
//
// #define NDEBUG
// #include <debug.h>
// C_ASSERT(sizeof(XFORML) == sizeof(XFORM));
//
//
// /* GLOBALS *******************************************************************/
//
// const MATRIX gmxIdentity =
// {
//     FLOATOBJ_1, FLOATOBJ_0,
//     FLOATOBJ_0, FLOATOBJ_1,
//     FLOATOBJ_0, FLOATOBJ_0,
//     0, 0, XFORM_NO_TRANSLATION|XFORM_FORMAT_LTOL|XFORM_UNITY|XFORM_SCALE
// };
//
//
// /* FUNCTIONS *****************************************************************/
//
// VOID
// FASTCALL
// DC_vFixIsotropicMapping(PDC pdc)
// {
//     PDC_ATTR pdcattr;
//     LONG64 fx, fy;
//     LONG s;
//     SIZEL szlWindowExt, szlViewportExt;
//     ASSERT(pdc->pdcattr->iMapMode == MM_ISOTROPIC);
//
//     /* Get a pointer to the DC_ATTR */
//     pdcattr = pdc->pdcattr;
//
//     /* Read the extents, we rely on non-null values */
//     szlWindowExt = pdcattr->szlWindowExt;
//     szlViewportExt = pdcattr->szlViewportExt;
//
//     /* Check if all values are valid */
//     if ((szlWindowExt.cx == 0) || (szlWindowExt.cy == 0) ||
//         (szlViewportExt.cx == 0) || (szlViewportExt.cy == 0))
//     {
//         /* Someone put rubbish into the fields, just ignore it. */
//         return;
//     }
//
//     fx = abs((LONG64)szlWindowExt.cx * szlViewportExt.cy);
//     fy = abs((LONG64)szlWindowExt.cy * szlViewportExt.cx);
//
//     if (fx < fy)
//     {
//         s = (szlWindowExt.cy ^ szlViewportExt.cx) > 0 ? 1 : -1;
//         pdcattr->szlViewportExt.cx = (LONG)(fx * s / szlWindowExt.cy);
//     }
//     else if (fx > fy)
//     {
//         s = (szlWindowExt.cx ^ szlViewportExt.cy) > 0 ? 1 : -1;
//         pdcattr->szlViewportExt.cy = (LONG)(fy * s / szlWindowExt.cx);
//     }
//
//     /* Reset the flag */
//     pdc->pdcattr->flXform &= ~PAGE_EXTENTS_CHANGED;
// }
//
// VOID
// FASTCALL
// DC_vGetPageToDevice(PDC pdc, MATRIX *pmx)
// {
//     PDC_ATTR pdcattr = pdc->pdcattr;
//     PSIZEL pszlViewPortExt;
//     SIZEL szlWindowExt;
//
//     /* Get the viewport extension */
//     pszlViewPortExt = DC_pszlViewportExt(pdc);
//
//     /* Copy the window extension, so no one can mess with it */
//     szlWindowExt = pdcattr->szlWindowExt;
//
//     /* No shearing / rotation */
//     FLOATOBJ_SetLong(&pmx->efM12, 0);
//     FLOATOBJ_SetLong(&pmx->efM21, 0);
//
//     /* Calculate scaling */
//     if (szlWindowExt.cx != 0)
//     {
//         FLOATOBJ_SetLong(&pmx->efM11, pszlViewPortExt->cx);
//         FLOATOBJ_DivLong(&pmx->efM11, szlWindowExt.cx);
//     }
//     else
//         FLOATOBJ_SetLong(&pmx->efM11, 1);
//
//     if (szlWindowExt.cy != 0)
//     {
//         FLOATOBJ_SetLong(&pmx->efM22, pszlViewPortExt->cy);
//         FLOATOBJ_DivLong(&pmx->efM22, szlWindowExt.cy);
//     }
//     else
//         FLOATOBJ_SetLong(&pmx->efM22, 1);
//
//     /* Calculate x offset */
//     FLOATOBJ_SetLong(&pmx->efDx, -pdcattr->ptlWindowOrg.x);
//     FLOATOBJ_Mul(&pmx->efDx, &pmx->efM11);
//     FLOATOBJ_AddLong(&pmx->efDx, pdcattr->ptlViewportOrg.x);
//
//     /* Calculate y offset */
//     FLOATOBJ_SetLong(&pmx->efDy, -pdcattr->ptlWindowOrg.y);
//     FLOATOBJ_Mul(&pmx->efDy, &pmx->efM22);
//     FLOATOBJ_AddLong(&pmx->efDy, pdcattr->ptlViewportOrg.y);
// }
//
// VOID
// FASTCALL
// DC_vUpdateWorldToDevice(PDC pdc)
// {
//     XFORMOBJ xoPageToDevice, xoWorldToPage, xoWorldToDevice;
//     MATRIX mxPageToDevice;
//
//     // FIXME: make sure world-to-page is valid!
//
//     /* Construct a transformation to do the page-to-device conversion */
//     DC_vGetPageToDevice(pdc, &mxPageToDevice);
//     XFORMOBJ_vInit(&xoPageToDevice, &mxPageToDevice);
//
//     /* Recalculate the world-to-device xform */
//     XFORMOBJ_vInit(&xoWorldToPage, &pdc->pdcattr->mxWorldToPage);
//     XFORMOBJ_vInit(&xoWorldToDevice, &pdc->pdcattr->mxWorldToDevice);
//     XFORMOBJ_iCombine(&xoWorldToDevice, &xoWorldToPage, &xoPageToDevice);
//
//     /* Reset the flags */
//     pdc->pdcattr->flXform &= ~WORLD_XFORM_CHANGED;
// }
//
// VOID
// FASTCALL
// DC_vUpdateDeviceToWorld(PDC pdc)
// {
//     XFORMOBJ xoWorldToDevice, xoDeviceToWorld;
//     PMATRIX pmxWorldToDevice;
//
//     /* Get the world-to-device translation */
//     pmxWorldToDevice = DC_pmxWorldToDevice(pdc);
//     XFORMOBJ_vInit(&xoWorldToDevice, pmxWorldToDevice);
//
//     /* Create inverse of world-to-device transformation */
//     XFORMOBJ_vInit(&xoDeviceToWorld, &pdc->pdcattr->mxDeviceToWorld);
//     if (XFORMOBJ_iInverse(&xoDeviceToWorld, &xoWorldToDevice) == DDI_ERROR)
//     {
//         MX_Set0(&pdc->pdcattr->mxDeviceToWorld);
//         return;
//     }
//
//     /* Reset the flag */
//     pdc->pdcattr->flXform &= ~DEVICE_TO_WORLD_INVALID;
// }
//
// BOOL
// NTAPI
// GreCombineTransform(
//     XFORML *pxformDest,
//     XFORML *pxform1,
//     XFORML *pxform2)
// {
//     MATRIX mxDest, mx1, mx2;
//     XFORMOBJ xoDest, xo1, xo2;
//
//     /* Check for illegal parameters */
//     if (!pxformDest || !pxform1 || !pxform2) return FALSE;
//
//     /* Initialize XFORMOBJs */
//     XFORMOBJ_vInit(&xoDest, &mxDest);
//     XFORMOBJ_vInit(&xo1, &mx1);
//     XFORMOBJ_vInit(&xo2, &mx2);
//
//     /* Convert the XFORMLs into XFORMOBJs */
//     XFORMOBJ_iSetXform(&xo1, pxform1);
//     XFORMOBJ_iSetXform(&xo2, pxform2);
//
//     /* Combine them */
//     XFORMOBJ_iCombine(&xoDest, &xo1, &xo2);
//
//     /* Translate back into XFORML */
//     XFORMOBJ_iGetXform(&xoDest, pxformDest);
//
//     return TRUE;
// }
//
// BOOL
// APIENTRY
// NtGdiCombineTransform(
//     LPXFORM UnsafeXFormResult,
//     LPXFORM Unsafexform1,
//     LPXFORM Unsafexform2)
// {
//     BOOL Ret;
//
//     _SEH2_TRY
//     {
//         ProbeForWrite(UnsafeXFormResult, sizeof(XFORM), 1);
//         ProbeForRead(Unsafexform1, sizeof(XFORM), 1);
//         ProbeForRead(Unsafexform2, sizeof(XFORM), 1);
//         Ret = GreCombineTransform((XFORML*)UnsafeXFormResult,
//                                   (XFORML*)Unsafexform1,
//                                   (XFORML*)Unsafexform2);
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Ret = FALSE;
//     }
//     _SEH2_END;
//
//     return Ret;
// }
//
// // FIXME: Should be XFORML and use XFORMOBJ functions directly
// BOOL
// APIENTRY
// NtGdiGetTransform(
//     HDC hdc,
//     DWORD iXform,
//     LPXFORM pXForm)
// {
//     PDC pdc;
//     BOOL ret = TRUE;
//     MATRIX mxPageToDevice;
//     XFORMOBJ xo;
//     PMATRIX pmx;
//
//     if (!pXForm)
//     {
//         EngSetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     pdc = DC_LockDc(hdc);
//     if (!pdc)
//     {
//         EngSetLastError(ERROR_INVALID_HANDLE);
//         return FALSE;
//     }
//
//     switch (iXform)
//     {
//         case GdiWorldSpaceToPageSpace:
//             pmx = DC_pmxWorldToPage(pdc);
//             break;
//
//         case GdiWorldSpaceToDeviceSpace:
//             pmx = DC_pmxWorldToDevice(pdc);
//             break;
//
//         case GdiDeviceSpaceToWorldSpace:
//             pmx = DC_pmxDeviceToWorld(pdc);
//             break;
//
//         case GdiPageSpaceToDeviceSpace:
//             DC_vGetPageToDevice(pdc, &mxPageToDevice);
//             pmx = &mxPageToDevice;
//             break;
//
//         default:
//             DPRINT1("Unknown transform %lu\n", iXform);
//             ret = FALSE;
//             goto leave;
//     }
//
//     /* Initialize an XFORMOBJ */
//     XFORMOBJ_vInit(&xo, pmx);
//
//     _SEH2_TRY
//     {
//         ProbeForWrite(pXForm, sizeof(XFORML), 1);
//         XFORMOBJ_iGetXform(&xo, (XFORML*)pXForm);
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         ret = FALSE;
//     }
//     _SEH2_END;
//
// leave:
//     DC_UnlockDc(pdc);
//     return ret;
// }
//
//
// /*!
//  * Converts points from logical coordinates into device coordinates.
//  * Conversion depends on the mapping mode,
//  * world transfrom, viewport origin settings for the given device context.
//  * \param	hDC		device context.
//  * \param	Points	an array of POINT structures (in/out).
//  * \param	Count	number of elements in the array of POINT structures.
//  * \return  TRUE if success, FALSE otherwise.
// */
// BOOL
// APIENTRY
// NtGdiTransformPoints(
//     HDC hDC,
//     PPOINT UnsafePtsIn,
//     PPOINT UnsafePtOut,
//     INT Count,
//     INT iMode)
// {
//     PDC pdc;
//     LPPOINT Points;
//     ULONG Size;
//     BOOL ret = TRUE;
//     NTSTATUS Status;
//
//     if (Count <= 0)
//         return TRUE;
//
//     if (!UnsafePtsIn || !UnsafePtOut)
//     {
//         return FALSE;
//     }
//
//     pdc = DC_LockDc(hDC);
//     if (!pdc)
//     {
//         return FALSE;
//     }
//
//     Size = Count * sizeof(POINT);
//
//     // FIXME: It would be wise to have a small stack buffer as optimization
//     Points = ExAllocatePoolWithTag(PagedPool, Size, GDITAG_TEMP);
//     if (!Points)
//     {
//         DC_UnlockDc(pdc);
//         EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return FALSE;
//     }
//
//     Status = SafeProbeForWrite(UnsafePtOut, Size, 1);
//     if (NT_SUCCESS(Status)) Status = SafeProbeForRead(UnsafePtsIn, Size, 1);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Do not set last error */
//         goto leave;
//     }
//     else
//     {
//         RtlCopyMemory(Points, UnsafePtsIn, Size);
//     }
//
//     switch (iMode)
//     {
//         case GdiDpToLp:
//             ret = INTERNAL_APPLY_MATRIX(DC_pmxDeviceToWorld(pdc), Points, Count);
//             break;
//
//         case GdiLpToDp:
//             ret = INTERNAL_APPLY_MATRIX(DC_pmxWorldToDevice(pdc), Points, Count);
//             break;
//
//         case 2: // Not supported yet. Need testing.
//         default:
//         {
//             EngSetLastError(ERROR_INVALID_PARAMETER);
//             ret = FALSE;
//             goto leave;
//         }
//     }
//
//     if (ret)
//     {
//         _SEH2_TRY
//         {
//             /* Pointer was already probed! */
//             RtlCopyMemory(UnsafePtOut, Points, Size);
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Do not set last error */
//             ret = FALSE;
//         }
//         _SEH2_END;
//     }
//
// //
// // If we are getting called that means User XForms is a mess!
// //
// leave:
//     DC_UnlockDc(pdc);
//     ExFreePoolWithTag(Points, GDITAG_TEMP);
//     return ret;
// }
//
//
// BOOL
// NTAPI
// GreModifyWorldTransform(
//     PDC pdc,
//     const XFORML *pxform,
//     DWORD dwMode)
// {
//     MATRIX mxSrc;
//     XFORMOBJ xoSrc, xoDC;
//
//     switch (dwMode)
//     {
//         case MWT_IDENTITY:
//             pdc->pdcattr->mxWorldToPage = gmxIdentity;
//             break;
//
//         case MWT_LEFTMULTIPLY:
//             XFORMOBJ_vInit(&xoDC, &pdc->pdcattr->mxWorldToPage);
//             XFORMOBJ_vInit(&xoSrc, &mxSrc);
//             if (XFORMOBJ_iSetXform(&xoSrc, pxform) == DDI_ERROR)
//                 return FALSE;
//             XFORMOBJ_iCombine(&xoDC, &xoSrc, &xoDC);
//             break;
//
//         case MWT_RIGHTMULTIPLY:
//             XFORMOBJ_vInit(&xoDC, &pdc->pdcattr->mxWorldToPage);
//             XFORMOBJ_vInit(&xoSrc, &mxSrc);
//             if (XFORMOBJ_iSetXform(&xoSrc, pxform) == DDI_ERROR)
//                 return FALSE;
//             XFORMOBJ_iCombine(&xoDC, &xoDC, &xoSrc);
//             break;
//
//         case MWT_SET:
//             XFORMOBJ_vInit(&xoDC, &pdc->pdcattr->mxWorldToPage);
//             if (XFORMOBJ_iSetXform(&xoDC, pxform) == DDI_ERROR)
//                 return FALSE;
//             break;
//
//         default:
//             return FALSE;
//     }
//
//     /*Set invalidation flags */
//     pdc->pdcattr->flXform |= WORLD_XFORM_CHANGED|DEVICE_TO_WORLD_INVALID;
//
//     return TRUE;
// }
//
// BOOL
// APIENTRY
// NtGdiModifyWorldTransform(
//     HDC hdc,
//     LPXFORM pxformUnsafe,
//     DWORD dwMode)
// {
//     PDC pdc;
//     XFORML xformSafe;
//     BOOL Ret = TRUE;
//     NTSTATUS Status;
//
//     pdc = DC_LockDc(hdc);
//     if (!pdc)
//     {
//         EngSetLastError(ERROR_INVALID_HANDLE);
//         return FALSE;
//     }
//
//     /* The xform is permitted to be NULL for MWT_IDENTITY.
//      * However, if it is not NULL, then it must be valid even
//      * though it is not used. */
//     if ((dwMode != MWT_IDENTITY) && (pxformUnsafe == NULL))
//     {
//         DC_UnlockDc(pdc);
//         return FALSE;
//     }
//
//     if (pxformUnsafe != NULL)
//     {
//         Status = SafeProbeForRead(pxformUnsafe, sizeof(XFORML), 1);
//         if (!NT_SUCCESS(Status))
//         {
//             Ret = FALSE;
//         }
//         else
//         {
//             RtlCopyMemory(&xformSafe, pxformUnsafe, sizeof(XFORML));
//         }
//     }
//
//     /* Safe to handle kernel mode data. */
//     if (Ret) Ret = GreModifyWorldTransform(pdc, &xformSafe, dwMode);
//     DC_UnlockDc(pdc);
//     return Ret;
// }
//
//
// BOOL
// APIENTRY
// NtGdiOffsetViewportOrgEx(
//     HDC hDC,
//     int XOffset,
//     int YOffset,
//     LPPOINT UnsafePoint)
// {
//     PDC      dc;
//     PDC_ATTR pdcattr;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     dc = DC_LockDc(hDC);
//     if (!dc)
//     {
//         EngSetLastError(ERROR_INVALID_HANDLE);
//         return FALSE;
//     }
//     pdcattr = dc->pdcattr;
//
//     if (UnsafePoint)
//     {
//         Status = SafeProbeForWrite(UnsafePoint, sizeof(POINT), 1);
//         if (NT_SUCCESS(Status))
//         {
//             UnsafePoint->x = pdcattr->ptlViewportOrg.x;
//             UnsafePoint->y = pdcattr->ptlViewportOrg.y;
//             if (pdcattr->dwLayout & LAYOUT_RTL)
//             {
//                 UnsafePoint->x = -UnsafePoint->x;
//             }
//         }
//
//         if (!NT_SUCCESS(Status))
//         {
//             SetLastNtError(Status);
//             DC_UnlockDc(dc);
//             return FALSE;
//         }
//     }
//
//     if (pdcattr->dwLayout & LAYOUT_RTL)
//     {
//         XOffset = -XOffset;
//     }
//     pdcattr->ptlViewportOrg.x += XOffset;
//     pdcattr->ptlViewportOrg.y += YOffset;
//     pdcattr->flXform |= PAGE_XLATE_CHANGED | WORLD_XFORM_CHANGED | DEVICE_TO_WORLD_INVALID;
//
//     DC_UnlockDc(dc);
//
//     return TRUE;
// }
//
// BOOL
// APIENTRY
// NtGdiOffsetWindowOrgEx(
//     HDC hDC,
//     int XOffset,
//     int YOffset,
//     LPPOINT Point)
// {
//     PDC dc;
//     PDC_ATTR pdcattr;
//
//     dc = DC_LockDc(hDC);
//     if (!dc)
//     {
//         EngSetLastError(ERROR_INVALID_HANDLE);
//         return FALSE;
//     }
//     pdcattr = dc->pdcattr;
//
//     if (Point)
//     {
//         NTSTATUS Status = STATUS_SUCCESS;
//
//         Status = SafeProbeForWrite(Point, sizeof(POINT), 1);
//         if (NT_SUCCESS(Status))
//         {
//             Point->x = pdcattr->ptlWindowOrg.x;
//             Point->y = pdcattr->ptlWindowOrg.y;
//         }
//
//         if (!NT_SUCCESS(Status))
//         {
//             SetLastNtError(Status);
//             DC_UnlockDc(dc);
//             return FALSE;
//         }
//     }
//
//     pdcattr->ptlWindowOrg.x += XOffset;
//     pdcattr->ptlWindowOrg.y += YOffset;
//     pdcattr->flXform |= PAGE_XLATE_CHANGED | WORLD_XFORM_CHANGED | DEVICE_TO_WORLD_INVALID;
//
//     DC_UnlockDc(dc);
//
//     return TRUE;
// }
//
// BOOL
// APIENTRY
// NtGdiScaleViewportExtEx(
//     HDC hDC,
//     int Xnum,
//     int Xdenom,
//     int Ynum,
//     int Ydenom,
//     LPSIZE pSize)
// {
//     PDC pDC;
//     PDC_ATTR pdcattr;
//     BOOL Ret = FALSE;
//     LONG X, Y;
//     NTSTATUS Status;
//
//     pDC = DC_LockDc(hDC);
//     if (!pDC)
//     {
//         EngSetLastError(ERROR_INVALID_HANDLE);
//         return FALSE;
//     }
//     pdcattr = pDC->pdcattr;
//
//     if (pdcattr->iMapMode > MM_TWIPS)
//     {
//         if (Xdenom && Ydenom)
//         {
//             DC_pszlViewportExt(pDC);
//             X = Xnum * pdcattr->szlViewportExt.cx / Xdenom;
//             if (X)
//             {
//                 Y = Ynum * pdcattr->szlViewportExt.cy / Ydenom;
//                 if (Y)
//                 {
//                     pdcattr->szlViewportExt.cx = X;
//                     pdcattr->szlViewportExt.cy = Y;
//                     pdcattr->flXform |= PAGE_XLATE_CHANGED;
//
//                     IntMirrorWindowOrg(pDC);
//
//                     pdcattr->flXform |= (PAGE_EXTENTS_CHANGED |
//                                          INVALIDATE_ATTRIBUTES |
//                                          WORLD_XFORM_CHANGED |
//                                          DEVICE_TO_WORLD_INVALID);
//
//                     if (pdcattr->iMapMode == MM_ISOTROPIC)
//                     {
//                         DC_vFixIsotropicMapping(pDC);
//                     }
//
//                     Ret = TRUE;
//                 }
//             }
//         }
//     }
//     else
//         Ret = TRUE;
//
//     if (pSize)
//     {
//         Status = SafeProbeForWrite(pSize, sizeof(SIZE), 1);
//         if (!NT_SUCCESS(Status))
//         {
//             SetLastNtError(Status);
//             Ret = FALSE;
//         }
//         else
//         {
//             pSize->cx = pdcattr->szlViewportExt.cx;
//             pSize->cy = pdcattr->szlViewportExt.cy;
//         }
//     }
//
//     DC_UnlockDc(pDC);
//     return Ret;
// }
//
//
// BOOL
// APIENTRY
// NtGdiScaleWindowExtEx(
//     HDC hDC,
//     int Xnum,
//     int Xdenom,
//     int Ynum,
//     int Ydenom,
//     LPSIZE pSize)
// {
//     PDC pDC;
//     PDC_ATTR pdcattr;
//     BOOL Ret = FALSE;
//     LONG X, Y;
//
//     pDC = DC_LockDc(hDC);
//     if (!pDC)
//     {
//         EngSetLastError(ERROR_INVALID_HANDLE);
//         return FALSE;
//     }
//     pdcattr = pDC->pdcattr;
//
//     if (pSize)
//     {
//         NTSTATUS Status = STATUS_SUCCESS;
//
//         Status = SafeProbeForWrite(pSize, sizeof(SIZE), 1);
//         if (NT_SUCCESS(Status))
//         {
//             X = pdcattr->szlWindowExt.cx;
//             if (pdcattr->dwLayout & LAYOUT_RTL) X = -X;
//             pSize->cx = X;
//             pSize->cy = pdcattr->szlWindowExt.cy;
//         }
//
//         if (!NT_SUCCESS(Status))
//         {
//             SetLastNtError(Status);
//             DC_UnlockDc(pDC);
//             return FALSE;
//         }
//     }
//
//     if (pdcattr->iMapMode > MM_TWIPS)
//     {
//         if (Xdenom && Ydenom)
//         {
//             X = Xnum * pdcattr->szlWindowExt.cx / Xdenom;
//             if (X)
//             {
//                 Y = Ynum * pdcattr->szlWindowExt.cy / Ydenom;
//                 if (Y)
//                 {
//                     pdcattr->szlWindowExt.cx = X;
//                     pdcattr->szlWindowExt.cy = Y;
//
//                     IntMirrorWindowOrg(pDC);
//
//                     pdcattr->flXform |= (PAGE_EXTENTS_CHANGED |
//                                          INVALIDATE_ATTRIBUTES |
//                                          WORLD_XFORM_CHANGED |
//                                          DEVICE_TO_WORLD_INVALID);
//
//                     Ret = TRUE;
//                 }
//             }
//         }
//     }
//     else
//         Ret = TRUE;
//
//     DC_UnlockDc(pDC);
//     return Ret;
// }
//
// int
// APIENTRY
// IntGdiSetMapMode(
//     PDC dc,
//     int MapMode)
// {
//     INT iPrevMapMode;
//     FLONG flXform;
//     PDC_ATTR pdcattr = dc->pdcattr;
//
//     if (MapMode == pdcattr->iMapMode)
//         return MapMode;
//
//     flXform = pdcattr->flXform & ~(ISO_OR_ANISO_MAP_MODE|PTOD_EFM22_NEGATIVE|
//         PTOD_EFM11_NEGATIVE|POSITIVE_Y_IS_UP|PAGE_TO_DEVICE_SCALE_IDENTITY|
//         PAGE_TO_DEVICE_IDENTITY);
//
//     switch (MapMode)
//     {
//         case MM_TEXT:
//             pdcattr->szlWindowExt.cx = 1;
//             pdcattr->szlWindowExt.cy = 1;
//             pdcattr->szlViewportExt.cx = 1;
//             pdcattr->szlViewportExt.cy = 1;
//             flXform |= PAGE_TO_DEVICE_SCALE_IDENTITY;
//             break;
//
//         case MM_ISOTROPIC:
//             flXform |= ISO_OR_ANISO_MAP_MODE;
//             /* Fall through */
//
//         case MM_LOMETRIC:
//             pdcattr->szlWindowExt.cx = pdcattr->szlVirtualDeviceMm.cx * 10;
//             pdcattr->szlWindowExt.cy = pdcattr->szlVirtualDeviceMm.cy * 10;
//             pdcattr->szlViewportExt.cx =  pdcattr->szlVirtualDevicePixel.cx;
//             pdcattr->szlViewportExt.cy = -pdcattr->szlVirtualDevicePixel.cy;
//             break;
//
//         case MM_HIMETRIC:
//             pdcattr->szlWindowExt.cx = pdcattr->szlVirtualDeviceMm.cx * 100;
//             pdcattr->szlWindowExt.cy = pdcattr->szlVirtualDeviceMm.cy * 100;
//             pdcattr->szlViewportExt.cx =  pdcattr->szlVirtualDevicePixel.cx;
//             pdcattr->szlViewportExt.cy = -pdcattr->szlVirtualDevicePixel.cy;
//             break;
//
//         case MM_LOENGLISH:
//             pdcattr->szlWindowExt.cx = EngMulDiv(1000, pdcattr->szlVirtualDeviceMm.cx, 254);
//             pdcattr->szlWindowExt.cy = EngMulDiv(1000, pdcattr->szlVirtualDeviceMm.cy, 254);
//             pdcattr->szlViewportExt.cx =  pdcattr->szlVirtualDevicePixel.cx;
//             pdcattr->szlViewportExt.cy = -pdcattr->szlVirtualDevicePixel.cy;
//             break;
//
//         case MM_HIENGLISH:
//             pdcattr->szlWindowExt.cx = EngMulDiv(10000, pdcattr->szlVirtualDeviceMm.cx, 254);
//             pdcattr->szlWindowExt.cy = EngMulDiv(10000, pdcattr->szlVirtualDeviceMm.cy, 254);
//             pdcattr->szlViewportExt.cx =  pdcattr->szlVirtualDevicePixel.cx;
//             pdcattr->szlViewportExt.cy = -pdcattr->szlVirtualDevicePixel.cy;
//             break;
//
//         case MM_TWIPS:
//             pdcattr->szlWindowExt.cx = EngMulDiv(14400, pdcattr->szlVirtualDeviceMm.cx, 254);
//             pdcattr->szlWindowExt.cy = EngMulDiv(14400, pdcattr->szlVirtualDeviceMm.cy, 254);
//             pdcattr->szlViewportExt.cx =  pdcattr->szlVirtualDevicePixel.cx;
//             pdcattr->szlViewportExt.cy = -pdcattr->szlVirtualDevicePixel.cy;
//             break;
//
//         case MM_ANISOTROPIC:
//             flXform &= ~(PAGE_TO_DEVICE_IDENTITY|POSITIVE_Y_IS_UP);
//             flXform |= ISO_OR_ANISO_MAP_MODE;
//             break;
//
//         default:
//             return 0;
//     }
//
//     /* Save the old map mode and set the new one */
//     iPrevMapMode = pdcattr->iMapMode;
//     pdcattr->iMapMode = MapMode;
//
//     /* Update xform flags */
//     pdcattr->flXform = flXform | (PAGE_XLATE_CHANGED | PAGE_EXTENTS_CHANGED |
//                                   INVALIDATE_ATTRIBUTES | DEVICE_TO_PAGE_INVALID |
//                                   WORLD_XFORM_CHANGED | DEVICE_TO_WORLD_INVALID);
//
//     return iPrevMapMode;
// }
//
// BOOL
// FASTCALL
// GreSetViewportOrgEx(
//     HDC hDC,
//     int X,
//     int Y,
//     LPPOINT Point)
// {
//     PDC dc;
//     PDC_ATTR pdcattr;
//
//     dc = DC_LockDc(hDC);
//     if (!dc)
//     {
//         EngSetLastError(ERROR_INVALID_HANDLE);
//         return FALSE;
//     }
//     pdcattr = dc->pdcattr;
//
//     if (Point)
//     {
//        Point->x = pdcattr->ptlViewportOrg.x;
//        Point->y = pdcattr->ptlViewportOrg.y;
//     }
//
//     pdcattr->ptlViewportOrg.x = X;
//     pdcattr->ptlViewportOrg.y = Y;
//     pdcattr->flXform |= PAGE_XLATE_CHANGED | WORLD_XFORM_CHANGED | DEVICE_TO_WORLD_INVALID;
//
//     DC_UnlockDc(dc);
//     return TRUE;
// }
//
// BOOL
// APIENTRY
// NtGdiSetViewportOrgEx(
//     HDC hDC,
//     int X,
//     int Y,
//     LPPOINT Point)
// {
//     PDC dc;
//     PDC_ATTR pdcattr;
//
//     dc = DC_LockDc(hDC);
//     if (!dc)
//     {
//         EngSetLastError(ERROR_INVALID_HANDLE);
//         return FALSE;
//     }
//     pdcattr = dc->pdcattr;
//
//     if (Point)
//     {
//         NTSTATUS Status = STATUS_SUCCESS;
//
//         Status = SafeProbeForWrite(Point, sizeof(POINT), 1);
//         if (NT_SUCCESS(Status))
//         {
//             Point->x = pdcattr->ptlViewportOrg.x;
//             Point->y = pdcattr->ptlViewportOrg.y;
//         }
//
//         if (!NT_SUCCESS(Status))
//         {
//             SetLastNtError(Status);
//             DC_UnlockDc(dc);
//             return FALSE;
//         }
//     }
//
//     pdcattr->ptlViewportOrg.x = X;
//     pdcattr->ptlViewportOrg.y = Y;
//     pdcattr->flXform |= PAGE_XLATE_CHANGED | WORLD_XFORM_CHANGED | DEVICE_TO_WORLD_INVALID;
//
//     DC_UnlockDc(dc);
//
//     return TRUE;
// }
//
// BOOL
// APIENTRY
// NtGdiSetWindowOrgEx(
//     HDC hDC,
//     int X,
//     int Y,
//     LPPOINT Point)
// {
//     PDC dc;
//     PDC_ATTR pdcattr;
//
//     dc = DC_LockDc(hDC);
//     if (!dc)
//     {
//         EngSetLastError(ERROR_INVALID_HANDLE);
//         return FALSE;
//     }
//     pdcattr = dc->pdcattr;
//
//     if (Point)
//     {
//         NTSTATUS Status = STATUS_SUCCESS;
//
//         Status = SafeProbeForWrite(Point, sizeof(POINT), 1);
//         if (NT_SUCCESS(Status))
//         {
//             Point->x = pdcattr->ptlWindowOrg.x;
//             Point->y = pdcattr->ptlWindowOrg.y;
//         }
//
//         if (!NT_SUCCESS(Status))
//         {
//             SetLastNtError(Status);
//             DC_UnlockDc(dc);
//             return FALSE;
//         }
//     }
//
//     pdcattr->ptlWindowOrg.x = X;
//     pdcattr->ptlWindowOrg.y = Y;
//     pdcattr->flXform |= PAGE_XLATE_CHANGED | WORLD_XFORM_CHANGED | DEVICE_TO_WORLD_INVALID;
//
//     DC_UnlockDc(dc);
//
//     return TRUE;
// }
// /* --- pretranslation split: the remaining function groups live in coord.split2..2.c and are #included below so this stays ONE translation unit (zero CMake/linkage change). Reassembled verbatim by cpp. --- */
// /* coord.split2.c — #include fragment of coord.c, carved for pretranslation so rustconv gets a small file. NOT a standalone TU; NOT in CMake; do not compile directly. The preprocessor reassembles the original coord.c TU verbatim. See docs/engineering/win32k-pretranslation-audit.md */
//
//
// //
// // Mirror Window function.
// //
// VOID
// FASTCALL
// IntMirrorWindowOrg(PDC dc)
// {
//     PDC_ATTR pdcattr;
//     LONG X, cx;
//
//     pdcattr = dc->pdcattr;
//
//     if (!(pdcattr->dwLayout & LAYOUT_RTL))
//     {
//         pdcattr->ptlWindowOrg.x = pdcattr->lWindowOrgx; // Flip it back.
//         return;
//     }
//
//     /* Copy the window extension, so no one can mess with it */
//     cx = pdcattr->szlViewportExt.cx;
//     if (cx == 0) return;
//     //
//     // WOrgx = wox - (Width - 1) * WExtx / VExtx
//     //
//     X = (dc->erclWindow.right - dc->erclWindow.left) - 1; // Get device width - 1
//
//     X = (X * pdcattr->szlWindowExt.cx) / cx;
//
//     pdcattr->ptlWindowOrg.x = pdcattr->lWindowOrgx - X; // Now set the inverted win origion.
//     pdcattr->flXform |= PAGE_XLATE_CHANGED | WORLD_XFORM_CHANGED | DEVICE_TO_WORLD_INVALID;
//
//     return;
// }
//
// VOID
// NTAPI
// DC_vSetLayout(
//     IN PDC pdc,
//     IN LONG wox,
//     IN DWORD dwLayout)
// {
//     PDC_ATTR pdcattr = pdc->pdcattr;
//
//     pdcattr->dwLayout = dwLayout;
//
//     if (!(dwLayout & LAYOUT_ORIENTATIONMASK)) return;
//
//     if (dwLayout & LAYOUT_RTL)
//     {
//         pdcattr->iMapMode = MM_ANISOTROPIC;
//     }
//
//     //pdcattr->szlWindowExt.cy = -pdcattr->szlWindowExt.cy;
//     //pdcattr->ptlWindowOrg.x  = -pdcattr->ptlWindowOrg.x;
//
//     //if (wox == -1)
//     //    IntMirrorWindowOrg(pdc);
//     //else
//     //    pdcattr->ptlWindowOrg.x = wox - pdcattr->ptlWindowOrg.x;
//
//     if (!(pdcattr->flTextAlign & TA_CENTER)) pdcattr->flTextAlign |= TA_RIGHT;
//
//     if (pdc->dclevel.flPath & DCPATH_CLOCKWISE)
//         pdc->dclevel.flPath &= ~DCPATH_CLOCKWISE;
//     else
//         pdc->dclevel.flPath |= DCPATH_CLOCKWISE;
//
//     pdcattr->flXform |= (PAGE_EXTENTS_CHANGED |
//                          INVALIDATE_ATTRIBUTES |
//                          WORLD_XFORM_CHANGED |
//                          DEVICE_TO_WORLD_INVALID);
// }
//
// // NtGdiSetLayout
// //
// // The default is left to right. This function changes it to right to left, which
// // is the standard in Arabic and Hebrew cultures.
// //
// /*
//  * @implemented
//  */
// DWORD
// APIENTRY
// NtGdiSetLayout(
//     IN HDC hdc,
//     IN LONG wox,
//     IN DWORD dwLayout)
// {
//     PDC pdc;
//     DWORD dwOldLayout;
//
//     pdc = DC_LockDc(hdc);
//     if (!pdc)
//     {
//         EngSetLastError(ERROR_INVALID_HANDLE);
//         return GDI_ERROR;
//     }
//
//     dwOldLayout = pdc->pdcattr->dwLayout;
//     DC_vSetLayout(pdc, wox, dwLayout);
//
//     DC_UnlockDc(pdc);
//     return dwOldLayout;
// }
//
// /*
//  * @implemented
//  */
// LONG
// APIENTRY
// NtGdiGetDeviceWidth(
//     IN HDC hdc)
// {
//     PDC dc;
//     LONG Ret;
//     dc = DC_LockDc(hdc);
//     if (!dc)
//     {
//         EngSetLastError(ERROR_INVALID_HANDLE);
//         return 0;
//     }
//     Ret = dc->erclWindow.right - dc->erclWindow.left;
//     DC_UnlockDc(dc);
//     return Ret;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// APIENTRY
// NtGdiMirrorWindowOrg(
//     IN HDC hdc)
// {
//     PDC dc;
//     dc = DC_LockDc(hdc);
//     if (!dc)
//     {
//         EngSetLastError(ERROR_INVALID_HANDLE);
//         return FALSE;
//     }
//     IntMirrorWindowOrg(dc);
//     DC_UnlockDc(dc);
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// APIENTRY
// NtGdiSetSizeDevice(
//     IN HDC hdc,
//     IN INT cxVirtualDevice,
//     IN INT cyVirtualDevice)
// {
//     PDC dc;
//     PDC_ATTR pdcattr;
//
//     if (!cxVirtualDevice || !cyVirtualDevice)
//     {
//         return FALSE;
//     }
//
//     dc = DC_LockDc(hdc);
//     if (!dc) return FALSE;
//
//     pdcattr = dc->pdcattr;
//
//     pdcattr->szlVirtualDeviceSize.cx = cxVirtualDevice;
//     pdcattr->szlVirtualDeviceSize.cy = cyVirtualDevice;
//
//     DC_UnlockDc(dc);
//
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// APIENTRY
// NtGdiSetVirtualResolution(
//     IN HDC hdc,
//     IN INT cxVirtualDevicePixel,
//     IN INT cyVirtualDevicePixel,
//     IN INT cxVirtualDeviceMm,
//     IN INT cyVirtualDeviceMm)
// {
//     PDC dc;
//     PDC_ATTR pdcattr;
//
//     /* Check parameters (all zeroes resets to real resolution) */
//     if (cxVirtualDevicePixel == 0 && cyVirtualDevicePixel == 0 &&
//         cxVirtualDeviceMm == 0 && cyVirtualDeviceMm == 0)
//     {
//         cxVirtualDevicePixel = NtGdiGetDeviceCaps(hdc, HORZRES);
//         cyVirtualDevicePixel = NtGdiGetDeviceCaps(hdc, VERTRES);
//         cxVirtualDeviceMm = NtGdiGetDeviceCaps(hdc, HORZSIZE);
//         cyVirtualDeviceMm = NtGdiGetDeviceCaps(hdc, VERTSIZE);
//     }
//     else if (cxVirtualDevicePixel == 0 || cyVirtualDevicePixel == 0 ||
//              cxVirtualDeviceMm == 0 || cyVirtualDeviceMm == 0)
//     {
//         return FALSE;
//     }
//
//     dc = DC_LockDc(hdc);
//     if (!dc) return FALSE;
//
//     pdcattr = dc->pdcattr;
//
//     pdcattr->szlVirtualDevicePixel.cx = cxVirtualDevicePixel;
//     pdcattr->szlVirtualDevicePixel.cy = cyVirtualDevicePixel;
//     pdcattr->szlVirtualDeviceMm.cx = cxVirtualDeviceMm;
//     pdcattr->szlVirtualDeviceMm.cy = cyVirtualDeviceMm;
//
// //    DC_vUpdateXforms(dc);
//     DC_UnlockDc(dc);
//     return TRUE;
// }
//
// static
// VOID FASTCALL
// DC_vGetAspectRatioFilter(PDC pDC, LPSIZE AspectRatio)
// {
//     if (pDC->pdcattr->flFontMapper & 1) // TRUE assume 1.
//     {
//         // "This specifies that Windows should only match fonts that have the
//         // same aspect ratio as the display.", Programming Windows, Fifth Ed.
//         AspectRatio->cx = pDC->ppdev->gdiinfo.ulLogPixelsX;
//         AspectRatio->cy = pDC->ppdev->gdiinfo.ulLogPixelsY;
//     }
//     else
//     {
//         AspectRatio->cx = 0;
//         AspectRatio->cy = 0;
//     }
// }
//
// BOOL APIENTRY
// GreGetDCPoint(
//     HDC hDC,
//     UINT iPoint,
//     PPOINTL Point)
// {
//     BOOL Ret = TRUE;
//     DC *pdc;
//     SIZE Size;
//     PSIZEL pszlViewportExt;
//
//     if (!Point)
//     {
//         EngSetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     pdc = DC_LockDc(hDC);
//     if (!pdc)
//     {
//         EngSetLastError(ERROR_INVALID_HANDLE);
//         return FALSE;
//     }
//
//     switch (iPoint)
//     {
//         case GdiGetViewPortExt:
//             pszlViewportExt = DC_pszlViewportExt(pdc);
//             Point->x = pszlViewportExt->cx;
//             Point->y = pszlViewportExt->cy;
//             break;
//
//         case GdiGetWindowExt:
//             Point->x = pdc->pdcattr->szlWindowExt.cx;
//             Point->y = pdc->pdcattr->szlWindowExt.cy;
//             break;
//
//         case GdiGetViewPortOrg:
//             *Point = pdc->pdcattr->ptlViewportOrg;
//             break;
//
//         case GdiGetWindowOrg:
//             *Point = pdc->pdcattr->ptlWindowOrg;
//             break;
//
//         case GdiGetDCOrg:
//             *Point = pdc->ptlDCOrig;
//             break;
//
//         case GdiGetAspectRatioFilter:
//             DC_vGetAspectRatioFilter(pdc, &Size);
//             Point->x = Size.cx;
//             Point->y = Size.cy;
//             break;
//
//         default:
//             EngSetLastError(ERROR_INVALID_PARAMETER);
//             Ret = FALSE;
//             break;
//     }
//
//     DC_UnlockDc(pdc);
//     return Ret;
// }
//
// BOOL
// WINAPI
// GreSetDCOrg(
//     _In_ HDC hdc,
//     _In_ LONG x,
//     _In_ LONG y,
//     _In_opt_ PRECTL Rect)
// {
//     PDC dc;
//
//     dc = DC_LockDc(hdc);
//     if (!dc) return FALSE;
//
//     /* Set DC Origin */
//     dc->ptlDCOrig.x = x;
//     dc->ptlDCOrig.y = y;
//
//     /* Recalculate Fill Origin */
//     dc->ptlFillOrigin.x = dc->dclevel.ptlBrushOrigin.x + x;
//     dc->ptlFillOrigin.y = dc->dclevel.ptlBrushOrigin.y + y;
//
//     /* Set DC Window Rectangle */
//     if (Rect)
//         dc->erclWindow = *Rect;
//
//     DC_UnlockDc(dc);
//     return TRUE;
// }
//
// BOOL
// WINAPI
// GreGetDCOrgEx(
//     _In_ HDC hdc,
//     _Out_ PPOINTL Point,
//     _Out_ PRECTL Rect)
// {
//     PDC dc;
//
//     dc = DC_LockDc(hdc);
//     if (!dc) return FALSE;
//
//     /* Retrieve DC Window Rectangle without a check */
//     *Rect = dc->erclWindow;
//
//     DC_UnlockDc(dc);
//
//     /* Use default call for DC Origin and parameter checking */
//     return GreGetDCPoint( hdc, GdiGetDCOrg, Point);
// }
//
// BOOL
// WINAPI
// GreGetWindowExtEx(
//     _In_ HDC hdc,
//     _Out_ LPSIZE lpSize)
// {
//     return GreGetDCPoint(hdc, GdiGetWindowExt, (PPOINTL)lpSize);
// }
//
// BOOL
// WINAPI
// GreGetViewportExtEx(
//     _In_ HDC hdc,
//     _Out_ LPSIZE lpSize)
// {
//     return GreGetDCPoint(hdc, GdiGetViewPortExt, (PPOINTL)lpSize);
// }
//
// BOOL APIENTRY
// NtGdiGetDCPoint(
//     HDC hDC,
//     UINT iPoint,
//     PPOINTL Point)
// {
//     BOOL Ret;
//     POINTL SafePoint;
//     NTSTATUS Status;
//
//     if (!Point)
//     {
//         EngSetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     Ret = GreGetDCPoint(hDC, iPoint, &SafePoint);
//     if (Ret)
//     {
//         Status = SafeProbeForWrite(Point, sizeof(POINT), 1);
//         if (!NT_SUCCESS(Status))
//         {
//             Ret = FALSE;
//         }
//         else
//         {
//             *Point = SafePoint;
//         }
//     }
//
//     return Ret;
// }
//
//
// /* EOF */
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
