// Phase 2c wave 18: path.c (+ split2/3 = one TU) -> 61 fns. FLIPPED, GREEN x3.
// The FORCEINLINE / macro / arch-divergent eng helpers this file calls (FLOATOBJ_*,
// XFORMOBJ_*, MatrixS2XForm/XFormToMatrix, Coord*/Int*toLP, RECTL_vSetRect/lGet*,
// GDI_HANDLE_GET_TYPE, DC_pszlViewportExt, IntIsEffectiveWidePen,
// KeSave/RestoreFloatingPointState, the gef0/1/2 constants) have no callable symbol
// on one or more arches, so they are reached through plain-cdecl `w32r_*` wrappers in
// gdi/ntgdi/path_rust_shims.c (reuses the C soft-float / matrix logic instead of
// reimplementing it). Exported PATH_* keep their path.h ABI (FASTCALL dual-cfg vs
// cdecl). DbgPrint replaces the TRACE debug macro. ~117 mechanical POINT/POINTL +
// int casts were auto-applied via a rustc-JSON-span fixer; transmute() bridges the
// layout-identical POINT/POINTL struct-value assignments.
use win32k_sys::win32k::*;
extern "C" { fn w32r_KeSaveFloatingPointState(p: *mut core::ffi::c_void) -> NTSTATUS; fn w32r_KeRestoreFloatingPointState(p: *mut core::ffi::c_void); }
use ntkrnl_sys::base::NT_SUCCESS;
use core::ffi::{c_void, c_uint};
use core::cmp::min;

// usize-accepting wrappers over the ddk/rtl primitives (arch SIZE_T = u32 on i386);
// lets the translated `count * size_of()` (usize) call sites pass without per-site casts.
#[inline(always)]
unsafe fn ExAllocatePoolWithTag(pool: POOL_TYPE, n: usize, tag: u32) -> PVOID {
    ntkrnl_sys::ddk::ExAllocatePoolWithTag(pool, n as _, tag)
}
#[inline(always)]
unsafe fn ExFreePoolWithTag(p: PVOID, tag: u32) {
    ntkrnl_sys::ddk::ExFreePoolWithTag(p, tag)
}
#[inline(always)]
unsafe fn RtlZeroMemory(d: *mut c_void, n: usize) {
    ntkrnl_sys::rtl::RtlZeroMemory(d, n as _)
}
#[inline(always)]
unsafe fn RtlCopyMemory(d: *mut c_void, s: *const c_void, n: usize) {
    ntkrnl_sys::rtl::RtlCopyMemory(d, s, n as _)
}

// GDI pool tags (multichar C literal big-endian -> little-endian bytes in memory).
const TAG_PATH: u32 = u32::from_le_bytes(*b"Gh07");   // '70hG'
const TAG_BEZIER: u32 = u32::from_le_bytes(*b"BEZR"); // 'RZEB'

// PATH object macros (path.h) over GDIOBJ_* primitives.
#[inline(always)]
unsafe fn PATH_AllocPathWithHandle() -> PPATH {
    GDIOBJ_AllocObjWithHandle(GDI_OBJECT_TYPE_PATH as _, core::mem::size_of::<PATH>() as _) as PPATH
}
#[inline(always)]
unsafe fn PATH_LockPath(hPath: HPATH) -> PPATH {
    GDIOBJ_ShareLockObj(hPath as HGDIOBJ, GDI_OBJECT_TYPE_PATH as _) as PPATH
}
#[inline(always)]
unsafe fn PATH_UnlockPath(pPath: PPATH) {
    GDIOBJ_vDereferenceObject(pPath as POBJ);
}
// XForm2MatrixS(m, x) = w32r_XFormToMatrix(m, (XFORML*)x)  (macro, coord.h)
#[inline(always)]
unsafe fn XForm2MatrixS(m: *mut MATRIX, x: *mut XFORML) {
    w32r_XFormToMatrix(m, x as *mut XFORML);
}
// ERR(fmt, ...) win32k debug macro -> route to the (already-declared) TRACE.
#[inline(always)]
unsafe fn err_msg(msg: *const u8) {
    DbgPrint(msg as *const i8);
}
// memcpy -> RtlCopyMemory (returns dest like libc).
#[inline(always)]
unsafe fn memcpy(d: *mut c_void, s: *const c_void, n: usize) -> *mut c_void {
    RtlCopyMemory(d, s, n as _);
    d
}

// Helper functions dropped by rustconv (correct ABI fixed at link if needed).
extern "C" {
    fn w32r_RECTL_vSetRect(prcl: *mut RECTL, left: LONG, top: LONG, right: LONG, bottom: LONG);
    fn w32r_DC_pszlViewportExt(pdc: PDC) -> PSIZEL;
    fn w32r_pgef(n: i32) -> *mut FLOATOBJ;
    fn w32r_XFORMOBJ_vInit(pxo: *mut XFORMOBJ, pmx: *mut MATRIX);
    fn w32r_XFORMOBJ_bApplyXform(pxo: *mut XFORMOBJ, iMode: ULONG, cPoints: ULONG, pvOut: PPOINTL, pvIn: PPOINTL) -> BOOL;
    fn w32r_MatrixS2XForm(pxform: *mut XFORML, pmx: *mut MATRIX);
    fn w32r_XFormToMatrix(pmx: *mut MATRIX, pxform: *mut XFORML);
    fn w32r_IntIsEffectiveWidePen(pbrLine: PBRUSH) -> BOOL;
    fn w32r_IntDPtoLP(pdc: PDC, ppt: PPOINTL, count: UINT) -> BOOLEAN;
    fn w32r_ftGdiGetGlyphOutline(pdc: PDC, wch: WCHAR, format: UINT, pgm: LPGLYPHMETRICS, cjBuf: ULONG, pvBuf: PVOID, pmat2: *const MAT2, bIgnoreRotation: BOOL) -> ULONG;
}

extern "C" {
    fn w32r_FLOATOBJ_Add(pf: *mut FLOATOBJ, pf1: *mut FLOATOBJ);
    fn w32r_FLOATOBJ_AddLong(pf: *mut FLOATOBJ, l: LONG);
    fn w32r_FLOATOBJ_Div(pf: *mut FLOATOBJ, pf1: *mut FLOATOBJ);
    fn w32r_FLOATOBJ_MulLong(pf: *mut FLOATOBJ, l: LONG);
    fn w32r_FLOATOBJ_SetFloat(pf: *mut FLOATOBJ, f: FLOATL);
    fn w32r_FLOATOBJ_bConvertToLong(pf: *mut FLOATOBJ, pl: *mut LONG) -> BOOLEAN;
    fn w32r_FLOATOBJ_SetLong(pf: PFLOATOBJ, l: i32);
    fn w32r_FLOATOBJ_DivLong(pf: PFLOATOBJ, l: i32);
    fn w32r_FLOATOBJ_Sub(pf: PFLOATOBJ, pf2: PFLOATOBJ);
    fn w32r_FLOATOBJ_GetFloat(pf: PFLOATOBJ) -> FLOATL;
    fn w32r_SafeProbeForWrite(Address: *mut core::ffi::c_void, Length: u32, Alignment: u32) -> i32;
    fn w32r_IntLPtoDP(dc: *mut DC, points: *mut POINT, count: INT) -> BOOL;
    fn DbgPrint(fmt: *const i8, ...);
    fn w32r_CoordLPtoDP(dc: PDC, pt: *mut POINT);
    fn GDI_Bezier(Points: *const POINT, count: INT, nPtsOut: *mut INT) -> *mut POINT;
    fn w32r_RECTL_lGetWidth(rect: *mut RECTL) -> LONG;
    fn w32r_RECTL_lGetHeight(rect: *mut RECTL) -> LONG;
    fn cos(x: f64) -> f64;
    fn fabs(x: f64) -> f64;
    fn sin(x: f64) -> f64;
    fn atan2(y: f64, x: f64) -> f64;
    fn ceil(x: f64) -> f64;
    fn floor(x: f64) -> f64;
    fn w32r_INTERNAL_LPTODP(dc: PDC, p: *mut POINT, count: INT);
    fn sqrt(x: f64) -> f64;
    fn w32r_GDI_HANDLE_GET_TYPE(hobj: *mut core::ffi::c_void) -> DWORD;
}
// drop_gdiobj_macro_externs: dropped 2 bogus extern decl(s) for GDI-object macro(s): PATH_LockPath, PATH_UnlockPath — these are macros in gdi/eng/<type>.h or gdi/ntgdi/<type>.h, not real fns. Inline-expand via GDIOBJ_* primitives per win32k_translation_prompt.md §6.
// ---- file-scope preamble (translated) ----
pub const NUM_ENTRIES_INITIAL: i32 = 16;
pub const GROW_FACTOR_NUMER: i32 = 2;
pub const GROW_FACTOR_DENOM: i32 = 1;

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn PATH_CreatePath(mut count: core::ffi::c_int) -> PPATH {
    let mut pPath: PPATH = PATH_AllocPathWithHandle();

    if pPath.is_null() {
        EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return core::ptr::null_mut();
    }

    GDIOBJ_vUnlockObject(pPath as POBJ);
    pPath = PATH_LockPath((*pPath).BaseObject.hHmgr);

    PATH_EmptyPath(pPath);

    count = if NUM_ENTRIES_INITIAL > count { NUM_ENTRIES_INITIAL } else { count };

    (*pPath).numEntriesAllocated = count;

    (*pPath).pPoints = (ExAllocatePoolWithTag(
        PagedPool,
        (count as usize) * core::mem::size_of::<POINT>(),
        TAG_PATH,
    ) as *mut POINT as _);
    RtlZeroMemory(
        (*pPath).pPoints as *mut core::ffi::c_void,
        (count as usize) * core::mem::size_of::<POINT>(),
    );
    (*pPath).pFlags = ExAllocatePoolWithTag(
        PagedPool,
        (count as usize) * core::mem::size_of::<u8>(),
        TAG_PATH,
    ) as *mut u8;
    RtlZeroMemory(
        (*pPath).pFlags as *mut core::ffi::c_void,
        (count as usize) * core::mem::size_of::<u8>(),
    );

    (*pPath).numEntriesUsed = 0;
    (*pPath).newStroke = (1u8 as _);
    (*pPath).state = PATH_Open;
    (*pPath).pos.x = 0;
    (*pPath).pos.y = 0;

    pPath
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn PATH_CreatePath(mut count: core::ffi::c_int) -> PPATH {
    let mut pPath: PPATH = PATH_AllocPathWithHandle();

    if pPath.is_null() {
        EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return core::ptr::null_mut();
    }

    GDIOBJ_vUnlockObject(pPath as POBJ);
    pPath = PATH_LockPath((*pPath).BaseObject.hHmgr);

    PATH_EmptyPath(pPath);

    count = if NUM_ENTRIES_INITIAL > count { NUM_ENTRIES_INITIAL } else { count };

    (*pPath).numEntriesAllocated = count;

    (*pPath).pPoints = ExAllocatePoolWithTag(
        PagedPool,
        (count as usize) * core::mem::size_of::<POINT>(),
        TAG_PATH,
    ) as *mut POINT as _;
    RtlZeroMemory(
        (*pPath).pPoints as *mut core::ffi::c_void,
        (count as usize) * core::mem::size_of::<POINT>(),
    );
    (*pPath).pFlags = ExAllocatePoolWithTag(
        PagedPool,
        (count as usize) * core::mem::size_of::<u8>(),
        TAG_PATH,
    ) as *mut u8;
    RtlZeroMemory(
        (*pPath).pFlags as *mut core::ffi::c_void,
        (count as usize) * core::mem::size_of::<u8>(),
    );

    (*pPath).numEntriesUsed = 0;
    (*pPath).newStroke = 1u8 as _;
    (*pPath).state = PATH_Open;
    (*pPath).pos.x = 0;
    (*pPath).pos.y = 0;

    pPath
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn PATH_DestroyGdiPath(pPath: PPATH) {
    if !(*pPath).pPoints.is_null() {
        ExFreePoolWithTag((*pPath).pPoints as *mut core::ffi::c_void, TAG_PATH);
    }
    if !(*pPath).pFlags.is_null() {
        ExFreePoolWithTag((*pPath).pFlags as *mut core::ffi::c_void, TAG_PATH);
    }
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn PATH_DestroyGdiPath(pPath: PPATH) {
    if !(*pPath).pPoints.is_null() {
        ExFreePoolWithTag((*pPath).pPoints as *mut core::ffi::c_void, TAG_PATH);
    }
    if !(*pPath).pFlags.is_null() {
        ExFreePoolWithTag((*pPath).pFlags as *mut core::ffi::c_void, TAG_PATH);
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "fastcall" fn PATH_Delete(hPath: HPATH) -> BOOL {
    let pPath: PPATH;
    if hPath.is_null() {
        return (FALSE as _);
    }
    pPath = PATH_LockPath(hPath);
    if pPath.is_null() {
        return (FALSE as _);
    }
    PATH_DestroyGdiPath(pPath);
    GDIOBJ_vDeleteObject(core::ptr::addr_of_mut!((*pPath).BaseObject));
    (TRUE as _)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
unsafe extern "C" fn PATH_Delete(hPath: HPATH) -> BOOL {
    let pPath: PPATH;
    if hPath.is_null() {
        return FALSE as _;
    }
    pPath = PATH_LockPath(hPath);
    if pPath.is_null() {
        return FALSE as _;
    }
    PATH_DestroyGdiPath(pPath);
    GDIOBJ_vDeleteObject(core::ptr::addr_of_mut!((*pPath).BaseObject));
    TRUE as _
}

#[cfg(target_arch = "x86")]
pub unsafe extern "fastcall" fn IntGdiCloseFigure(pPath: PPATH) {
    if (*pPath).numEntriesUsed != 0 {
        *(*pPath).pFlags.add(((*pPath).numEntriesUsed - 1) as usize) |= PT_CLOSEFIGURE as u8;
        (*pPath).newStroke = (1u8 as _);
    }
}

#[cfg(not(target_arch = "x86"))]
pub unsafe extern "C" fn IntGdiCloseFigure(pPath: PPATH) {
    if (*pPath).numEntriesUsed != 0 {
        *(*pPath).pFlags.add(((*pPath).numEntriesUsed - 1) as usize) |= PT_CLOSEFIGURE as u8;
        (*pPath).newStroke = 1u8 as _;
    }
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn GdiPathDPtoLP(
    pdc: PDC,
    ppt: PPOINT,
    count: INT,
) -> BOOL {
    GdiPathDPtoLP_impl(pdc, ppt, count)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn GdiPathDPtoLP(
    pdc: PDC,
    ppt: PPOINT,
    count: INT,
) -> BOOL {
    GdiPathDPtoLP_impl(pdc, ppt, count)
}

unsafe fn GdiPathDPtoLP_impl(
    pdc: PDC,
    ppt: PPOINT,
    count: INT,
) -> BOOL {
    let mut xo: XFORMOBJ = core::mem::zeroed();

    w32r_XFORMOBJ_vInit(
        core::ptr::addr_of_mut!(xo),
        core::ptr::addr_of_mut!((*(*pdc).pdcattr).mxDeviceToWorld),
    );
    w32r_XFORMOBJ_bApplyXform(
        core::ptr::addr_of_mut!(xo),
        XF_LTOL as _,
        count as _,
        ppt as PPOINTL,
        ppt as PPOINTL,
    )
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn PATH_InitGdiPath(pPath: PPATH) {
    (*pPath).state = PATH_Null;
    (*pPath).pPoints = core::ptr::null_mut();
    (*pPath).pFlags = core::ptr::null_mut();
    (*pPath).numEntriesUsed = 0;
    (*pPath).numEntriesAllocated = 0;
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn PATH_InitGdiPath(pPath: PPATH) {
    (*pPath).state = PATH_Null;
    (*pPath).pPoints = core::ptr::null_mut();
    (*pPath).pFlags = core::ptr::null_mut();
    (*pPath).numEntriesUsed = 0;
    (*pPath).numEntriesAllocated = 0;
}

#[cfg(target_pointer_width = "64")]
#[no_mangle]
pub unsafe extern "C" fn PATH_AssignGdiPath(
    pPathDest: PPATH,
    pPathSrc: PPATH,
) -> BOOL {
    PATH_AssignGdiPath_impl(pPathDest, pPathSrc)
}

#[cfg(not(target_pointer_width = "64"))]
#[no_mangle]
pub unsafe extern "fastcall" fn PATH_AssignGdiPath(
    pPathDest: PPATH,
    pPathSrc: PPATH,
) -> BOOL {
    PATH_AssignGdiPath_impl(pPathDest, pPathSrc)
}

unsafe fn PATH_AssignGdiPath_impl(
    pPathDest: PPATH,
    pPathSrc: PPATH,
) -> BOOL {
    if PATH_ReserveEntries(pPathDest, (*pPathSrc).numEntriesUsed) == 0 {
        return 0;
    }

    core::ptr::copy_nonoverlapping(
        (*pPathSrc).pPoints as *const u8,
        (*pPathDest).pPoints as *mut u8,
        core::mem::size_of::<POINT>() * (*pPathSrc).numEntriesUsed as usize,
    );
    core::ptr::copy_nonoverlapping(
        (*pPathSrc).pFlags as *const u8,
        (*pPathDest).pFlags as *mut u8,
        core::mem::size_of::<BYTE>() * (*pPathSrc).numEntriesUsed as usize,
    );

    (*pPathDest).pos = (*pPathSrc).pos;
    (*pPathDest).state = (*pPathSrc).state;
    (*pPathDest).numEntriesUsed = (*pPathSrc).numEntriesUsed;
    (*pPathDest).newStroke = (*pPathSrc).newStroke;
    1
}

#[no_mangle]
pub unsafe extern "C" fn PATH_SavePath(dst: *mut DC, src: *mut DC) -> BOOL {
    let psrcPath: PPATH = PATH_LockPath((*src).dclevel.hPath);
    let pdstPath: PPATH;
    DbgPrint(b"PATH_SavePath\n\0".as_ptr() as *const i8);
    if !psrcPath.is_null() {
        DbgPrint(b"PATH_SavePath 1\n\0".as_ptr() as *const i8);

        pdstPath = PATH_CreatePath((*psrcPath).numEntriesAllocated);

        (*dst).dclevel.flPath = (*src).dclevel.flPath;

        (*dst).dclevel.hPath = (*pdstPath).BaseObject.hHmgr;

        PATH_AssignGdiPath(pdstPath, psrcPath);

        PATH_UnlockPath(pdstPath);
        PATH_UnlockPath(psrcPath);
    }
    1
}

#[no_mangle]
pub unsafe extern "C" fn PATH_RestorePath(dst: *mut DC, src: *mut DC) -> BOOL {
    DbgPrint(b"PATH_RestorePath\n\0".as_ptr() as *const core::ffi::c_char);

    if (*dst).dclevel.hPath.is_null() {
        let psrcPath: PPATH = PATH_LockPath((*src).dclevel.hPath);
        DbgPrint(b"PATH_RestorePath 1\n\0".as_ptr() as *const core::ffi::c_char);
        let pdstPath: PPATH = PATH_CreatePath((*psrcPath).numEntriesAllocated);
        (*dst).dclevel.flPath = (*src).dclevel.flPath;
        (*dst).dclevel.hPath = (*pdstPath).BaseObject.hHmgr;

        PATH_AssignGdiPath(pdstPath, psrcPath);

        PATH_UnlockPath(pdstPath);
        PATH_UnlockPath(psrcPath);
    } else {
        let psrcPath: PPATH = PATH_LockPath((*src).dclevel.hPath);
        let pdstPath: PPATH = PATH_LockPath((*dst).dclevel.hPath);
        DbgPrint(b"PATH_RestorePath 2\n\0".as_ptr() as *const core::ffi::c_char);
        (*dst).dclevel.flPath = (*src).dclevel.flPath & (DCPATH_CLOCKWISE | DCPATH_ACTIVE);
        PATH_AssignGdiPath(pdstPath, psrcPath);

        PATH_UnlockPath(pdstPath);
        PATH_UnlockPath(psrcPath);
    }
    1
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn PATH_EmptyPath(pPath: PPATH) {
    (*pPath).state = PATH_Null;
    (*pPath).numEntriesUsed = 0;
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn PATH_EmptyPath(pPath: PPATH) {
    (*pPath).state = PATH_Null;
    (*pPath).numEntriesUsed = 0;
}

#[cfg(target_pointer_width = "64")]
pub unsafe extern "C" fn PATH_AddEntry(
    pPath: PPATH,
    pPoint: *const POINT,
    flags: BYTE,
) -> BOOL {
    PATH_AddEntry_impl(pPath, pPoint, flags)
}

#[cfg(not(target_pointer_width = "64"))]
pub unsafe extern "fastcall" fn PATH_AddEntry(
    pPath: PPATH,
    pPoint: *const POINT,
    flags: BYTE,
) -> BOOL {
    PATH_AddEntry_impl(pPath, pPoint, flags)
}

unsafe fn PATH_AddEntry_impl(
    pPath: PPATH,
    pPoint: *const POINT,
    flags: BYTE,
) -> BOOL {
    if PATH_ReserveEntries(pPath, (*pPath).numEntriesUsed + 1) == 0 {
        return 0;
    }

    *(*pPath).pPoints.offset((*pPath).numEntriesUsed as isize) = core::mem::transmute(*pPoint);
    *(*pPath).pFlags.offset((*pPath).numEntriesUsed as isize) = flags;

    (*pPath).numEntriesUsed += 1;

    1
}

#[cfg(target_pointer_width = "64")]
pub unsafe extern "C" fn PATH_ReserveEntries(pPath: PPATH, numEntries: INT) -> BOOL {
    PATH_ReserveEntries_impl(pPath, numEntries)
}

#[cfg(not(target_pointer_width = "64"))]
pub unsafe extern "fastcall" fn PATH_ReserveEntries(pPath: PPATH, numEntries: INT) -> BOOL {
    PATH_ReserveEntries_impl(pPath, numEntries)
}

unsafe fn PATH_ReserveEntries_impl(pPath: PPATH, numEntries: INT) -> BOOL {
    let numEntriesToAllocate: INT;
    let pPointsNew: *mut POINT;
    let pFlagsNew: *mut BYTE;

    if numEntries > (*pPath).numEntriesAllocated {
        if (*pPath).numEntriesAllocated != 0 {
            let mut n = (*pPath).numEntriesAllocated;
            while n < numEntries {
                n = n * GROW_FACTOR_NUMER / GROW_FACTOR_DENOM;
            }
            numEntriesToAllocate = n;
        } else {
            numEntriesToAllocate = numEntries;
        }

        pPointsNew = ExAllocatePoolWithTag(
            PagedPool,
            (numEntriesToAllocate as usize) * core::mem::size_of::<POINT>(),
            TAG_PATH,
        ) as *mut POINT;
        if pPointsNew.is_null() {
            return FALSE as BOOL;
        }

        pFlagsNew = ExAllocatePoolWithTag(
            PagedPool,
            (numEntriesToAllocate as usize) * core::mem::size_of::<BYTE>(),
            TAG_PATH,
        ) as *mut BYTE;
        if pFlagsNew.is_null() {
            ExFreePoolWithTag(pPointsNew as *mut core::ffi::c_void, TAG_PATH);
            return FALSE as BOOL;
        }

        if !(*pPath).pPoints.is_null() {
            memcpy(
                pPointsNew as *mut core::ffi::c_void,
                (*pPath).pPoints as *const core::ffi::c_void,
                core::mem::size_of::<POINT>() * (*pPath).numEntriesUsed as usize,
            );
            memcpy(
                pFlagsNew as *mut core::ffi::c_void,
                (*pPath).pFlags as *const core::ffi::c_void,
                core::mem::size_of::<BYTE>() * (*pPath).numEntriesUsed as usize,
            );

            ExFreePoolWithTag((*pPath).pPoints as *mut core::ffi::c_void, TAG_PATH);
            ExFreePoolWithTag((*pPath).pFlags as *mut core::ffi::c_void, TAG_PATH);
        }

        (*pPath).pPoints = pPointsNew as _;
        (*pPath).pFlags = pFlagsNew;
        (*pPath).numEntriesAllocated = numEntriesToAllocate;
    }

    TRUE as BOOL
}

unsafe fn PATH_ScaleNormalizedPoint(
    corners: *mut POINT,
    x: FLOATL,
    y: FLOATL,
    pPoint: *mut POINT,
) -> BOOLEAN {

    let mut tmp: FLOATOBJ = core::mem::zeroed();

    w32r_FLOATOBJ_SetFloat(&mut tmp, x);
    w32r_FLOATOBJ_Add(&mut tmp, w32r_pgef(1));
    w32r_FLOATOBJ_Div(&mut tmp, w32r_pgef(2));
    w32r_FLOATOBJ_MulLong(&mut tmp, (*corners.add(1)).x - (*corners.add(0)).x);
    w32r_FLOATOBJ_AddLong(&mut tmp, (*corners.add(0)).x);
    if w32r_FLOATOBJ_bConvertToLong(&mut tmp, core::ptr::addr_of_mut!((*pPoint).x)) == 0 {
        return 0u8;
    }

    w32r_FLOATOBJ_SetFloat(&mut tmp, y);
    w32r_FLOATOBJ_Add(&mut tmp, w32r_pgef(1));
    w32r_FLOATOBJ_Div(&mut tmp, w32r_pgef(2));
    w32r_FLOATOBJ_MulLong(&mut tmp, (*corners.add(1)).y - (*corners.add(0)).y);
    w32r_FLOATOBJ_AddLong(&mut tmp, (*corners.add(0)).y);
    if w32r_FLOATOBJ_bConvertToLong(&mut tmp, core::ptr::addr_of_mut!((*pPoint).y)) == 0 {
        return 0u8;
    }
    1u8
}

unsafe fn PATH_NormalizePoint(
    corners: *mut POINTL,
    pPoint: *const POINTL,
    pX: *mut FLOATL,
    pY: *mut FLOATL,
) {

    let mut tmp: FLOATOBJ = core::mem::zeroed();

    w32r_FLOATOBJ_SetLong(
        core::ptr::addr_of_mut!(tmp),
        ((*pPoint).x - (*corners.offset(0)).x) * 2,
    );
    w32r_FLOATOBJ_DivLong(
        core::ptr::addr_of_mut!(tmp),
        (*corners.offset(1)).x - (*corners.offset(0)).x,
    );
    w32r_FLOATOBJ_Sub(core::ptr::addr_of_mut!(tmp), w32r_pgef(1));
    *pX = w32r_FLOATOBJ_GetFloat(core::ptr::addr_of_mut!(tmp));

    w32r_FLOATOBJ_SetLong(
        core::ptr::addr_of_mut!(tmp),
        ((*pPoint).y - (*corners.offset(0)).y) * 2,
    );
    w32r_FLOATOBJ_DivLong(
        core::ptr::addr_of_mut!(tmp),
        (*corners.offset(1)).y - (*corners.offset(0)).y,
    );
    w32r_FLOATOBJ_Sub(core::ptr::addr_of_mut!(tmp), w32r_pgef(1));
    *pY = w32r_FLOATOBJ_GetFloat(core::ptr::addr_of_mut!(tmp));
}

unsafe fn PATH_CheckRect(
    dc: *mut DC,
    rect: *mut RECTL,
    x1: INT,
    y1: INT,
    x2: INT,
    y2: INT,
) -> BOOL {
    let pdcattr: PDC_ATTR = (*dc).pdcattr;

    w32r_RECTL_vSetRect(rect, x1, y1, x2, y2);
    w32r_IntLPtoDP(dc, rect as PPOINT, 2);

    RECTL_vMakeWellOrdered(rect);

    if (*pdcattr).iGraphicsMode == GM_COMPATIBLE as i32 {
        if (*rect).left == (*rect).right {
            return FALSE as BOOL;
        }
        if (*rect).top == (*rect).bottom {
            return FALSE as BOOL;
        }
        (*rect).right -= 1;
        (*rect).bottom -= 1;
    }
    TRUE as BOOL
}

unsafe fn add_log_points(dc: *mut DC, path: PPATH, points: *const POINT, count: DWORD, type_: BYTE) -> *mut BYTE {
    let ret: *mut BYTE;

    if PATH_ReserveEntries(path, (*path).numEntriesUsed + count as INT) == 0 {
        return core::ptr::null_mut();
    }

    ret = core::ptr::addr_of_mut!(*(*path).pFlags.offset((*path).numEntriesUsed as isize));
    core::ptr::copy_nonoverlapping(
        points,
        (*path).pPoints.offset((*path).numEntriesUsed as isize) as _,
        count as usize,
    );
    w32r_IntLPtoDP(dc, (*path).pPoints.offset((*path).numEntriesUsed as isize) as _, count as INT);
    core::ptr::write_bytes(ret, type_, count as usize);
    (*path).numEntriesUsed += count as INT;
    ret
}

unsafe fn add_points(path: PPATH, points: *const POINT, count: DWORD, type_: BYTE) -> *mut BYTE {
    if PATH_ReserveEntries(path, (*path).numEntriesUsed + count as INT) == 0 {
        return core::ptr::null_mut();
    }

    let ret = core::ptr::addr_of_mut!(*(*path).pFlags.offset((*path).numEntriesUsed as isize));
    core::ptr::copy_nonoverlapping(
        points,
        (*path).pPoints.offset((*path).numEntriesUsed as isize) as _,
        count as usize,
    );
    core::ptr::write_bytes(ret, type_, count as usize);
    (*path).numEntriesUsed += count as INT;
    ret
}

unsafe fn reverse_points(points: *mut POINT, count: UINT) {
    let mut i: UINT = 0;
    while i < count / 2 {
        let pt = *points.add(i as usize);
        *points.add(i as usize) = *points.add((count - i - 1) as usize);
        *points.add((count - i - 1) as usize) = pt;
        i += 1;
    }
}

unsafe fn start_new_stroke(path: PPATH) -> BOOL {
    if (*path).newStroke == 0
        && (*path).numEntriesUsed != 0
        && (*(*path).pFlags.offset(((*path).numEntriesUsed - 1) as isize) & PT_CLOSEFIGURE as BYTE) == 0
        && (*(*path).pPoints.offset(((*path).numEntriesUsed - 1) as isize)).x == (*path).pos.x
        && (*(*path).pPoints.offset(((*path).numEntriesUsed - 1) as isize)).y == (*path).pos.y
    {
        return 1;
    }

    (*path).newStroke = 0;
    (add_points(path, core::ptr::addr_of_mut!((*path).pos) as _, 1, PT_MOVETO as BYTE) != core::ptr::null_mut()) as BOOL
}

unsafe fn update_current_pos(path: PPATH) {
    (*path).pos = *(*path).pPoints.add(((*path).numEntriesUsed - 1) as usize);
}

unsafe fn close_figure(path: PPATH) {
    *(*path).pFlags.add(((*path).numEntriesUsed - 1) as usize) |= PT_CLOSEFIGURE as u8;
}

unsafe fn add_log_points_new_stroke(dc: *mut DC, path: PPATH, points: *const POINT, count: DWORD, type_: BYTE) -> BOOL {
    if start_new_stroke(path) == 0 {
        return 0;
    }
    if add_log_points(dc, path, points, count, type_) == core::ptr::null_mut() {
        return 0;
    }
    update_current_pos(path);

    DbgPrint(b"ALPNS : Pos X %d Y %d\n\0".as_ptr() as *const i8, (*path).pos.x, (*path).pos.y);
    IntGdiMoveToEx(dc, (*path).pos.x, (*path).pos.y, core::ptr::null_mut());

    1
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn PATH_MoveTo(dc: PDC, pPath: PPATH) -> BOOL {
    PATH_MoveTo_impl(dc, pPath)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn PATH_MoveTo(dc: PDC, pPath: PPATH) -> BOOL {
    PATH_MoveTo_impl(dc, pPath)
}

unsafe fn PATH_MoveTo_impl(dc: PDC, pPath: PPATH) -> BOOL {
    if pPath.is_null() {
        return 0;
    }

    if (*(*dc).pdcattr).ulDirty_ & DIRTY_STYLESTATE != 0 {
        DbgPrint(b"MoveTo has changed\n\0".as_ptr() as *const i8);
        (*pPath).newStroke = 1;
        IntGetCurrentPositionEx(dc, core::ptr::addr_of_mut!((*pPath).pos));
        w32r_IntLPtoDP(dc, core::ptr::addr_of_mut!((*pPath).pos) as _, 1);
        return 1;
    }

    0
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn PATH_LineTo(dc: PDC, x: INT, y: INT) -> BOOL {
    PATH_LineTo_impl(dc, x, y)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn PATH_LineTo(dc: PDC, x: INT, y: INT) -> BOOL {
    PATH_LineTo_impl(dc, x, y)
}

unsafe fn PATH_LineTo_impl(dc: PDC, x: INT, y: INT) -> BOOL {
    let Ret: BOOL;
    let pPath: PPATH;
    let mut point: POINT = core::mem::zeroed();
    let mut pointCurPos: POINT = core::mem::zeroed();

    pPath = PATH_LockPath((*dc).dclevel.hPath);
    if pPath.is_null() {
        return 0;
    }

    point.x = x;
    point.y = y;

    if PATH_MoveTo(dc, pPath) == 0 {
        if (*pPath).newStroke != 0 {
            DbgPrint(b"Line To : New Stroke\n\0".as_ptr() as *const i8);
            (*pPath).newStroke = 0;
            IntGetCurrentPositionEx(dc, core::ptr::addr_of_mut!(pointCurPos) as _);
            w32r_CoordLPtoDP(dc, &mut pointCurPos);
            if PATH_AddEntry(pPath, &pointCurPos, PT_MOVETO as BYTE) == 0 {
                PATH_UnlockPath(pPath);
                return 0;
            }
        }
    }
    Ret = add_log_points_new_stroke(dc, pPath, &point, 1, PT_LINETO as BYTE);
    PATH_UnlockPath(pPath);
    Ret
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn PATH_Rectangle(
    dc: PDC,
    x1: INT,
    y1: INT,
    x2: INT,
    y2: INT,
) -> BOOL {
    PATH_Rectangle_impl(dc, x1, y1, x2, y2)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn PATH_Rectangle(
    dc: PDC,
    x1: INT,
    y1: INT,
    x2: INT,
    y2: INT,
) -> BOOL {
    PATH_Rectangle_impl(dc, x1, y1, x2, y2)
}

unsafe fn PATH_Rectangle_impl(
    dc: PDC,
    x1: INT,
    y1: INT,
    x2: INT,
    y2: INT,
) -> BOOL {
    let pPath: PPATH;
    let mut rect: RECTL = core::mem::zeroed();
    let mut points: [POINTL; 4] = core::mem::zeroed();
    let type_: *mut BYTE;

    pPath = PATH_LockPath((*dc).dclevel.hPath);
    if pPath.is_null() {
        return 0;
    }

    if PATH_CheckRect(dc, &mut rect, x1, y1, x2, y2) == 0 {
        PATH_UnlockPath(pPath);
        return 1;
    }

    points[0].x = rect.right;
    points[0].y = rect.top;
    points[1].x = rect.left;
    points[1].y = rect.top;
    points[2].x = rect.left;
    points[2].y = rect.bottom;
    points[3].x = rect.right;
    points[3].y = rect.bottom;

    if (*dc).dclevel.flPath & DCPATH_CLOCKWISE != 0 {
        reverse_points(points.as_mut_ptr() as _, 4);
    }

    type_ = add_points(pPath, points.as_ptr() as _, 4, PT_LINETO as BYTE);
    if type_.is_null() {
        PATH_UnlockPath(pPath);
        return 0;
    }
    *type_.add(0) = PT_MOVETO as BYTE;

    IntGdiCloseFigure(pPath);
    PATH_UnlockPath(pPath);
    1
}

#[no_mangle]
pub unsafe extern "C" fn PATH_RoundRect(
    dc: *mut DC,
    x1: INT,
    y1: INT,
    x2: INT,
    y2: INT,
    ell_width: INT,
    ell_height: INT,
) -> BOOL {
    PATH_RoundRect_impl(dc, x1, y1, x2, y2, ell_width, ell_height)
}

unsafe fn PATH_RoundRect_impl(
    dc: *mut DC,
    x1: INT,
    y1: INT,
    x2: INT,
    y2: INT,
    mut ell_width: INT,
    mut ell_height: INT,
) -> BOOL {
    let pPath: PPATH;
    let mut rect: RECTL = core::mem::zeroed();
    let mut ellipse: RECTL = core::mem::zeroed();
    let mut points: [POINT; 16] = core::mem::zeroed();
    let type_: *mut BYTE;
    let xOffset: INT;
    let yOffset: INT;

    if ell_width == 0 || ell_height == 0 {
        return PATH_Rectangle(dc, x1, y1, x2, y2);
    }

    pPath = PATH_LockPath((*dc).dclevel.hPath);
    if pPath.is_null() {
        return 0;
    }

    if PATH_CheckRect(dc, &mut rect, x1, y1, x2, y2) == 0 {
        PATH_UnlockPath(pPath);
        return 1;
    }

    w32r_RECTL_vSetRect(&mut ellipse, 0, 0, ell_width, ell_height);
    w32r_IntLPtoDP(dc, &mut ellipse as *mut RECTL as PPOINT, 2);
    RECTL_vMakeWellOrdered(&mut ellipse);
    ell_width = min(w32r_RECTL_lGetWidth(&mut ellipse), w32r_RECTL_lGetWidth(&mut rect));
    ell_height = min(w32r_RECTL_lGetHeight(&mut ellipse), w32r_RECTL_lGetHeight(&mut rect));

    xOffset = EngMulDiv(ell_width, 44771525, 200000000);
    yOffset = EngMulDiv(ell_height, 44771525, 200000000);
    DbgPrint(
        b"xOffset %d, yOffset %d, Rect WxH: %dx%d.\n\0".as_ptr() as *const i8,
        xOffset,
        yOffset,
        w32r_RECTL_lGetWidth(&mut rect),
        w32r_RECTL_lGetHeight(&mut rect),
    );

    ell_width = EngMulDiv(ell_width, 1, 2);
    ell_height = EngMulDiv(ell_height, 1, 2);

    points[0].x = rect.right;
    points[0].y = rect.top + ell_height;
    points[1].x = rect.right;
    points[1].y = rect.top + yOffset;
    points[2].x = rect.right - xOffset;
    points[2].y = rect.top;
    points[3].x = rect.right - ell_width;
    points[3].y = rect.top;
    points[4].x = rect.left + ell_width;
    points[4].y = rect.top;
    points[5].x = rect.left + xOffset;
    points[5].y = rect.top;
    points[6].x = rect.left;
    points[6].y = rect.top + yOffset;
    points[7].x = rect.left;
    points[7].y = rect.top + ell_height;
    points[8].x = rect.left;
    points[8].y = rect.bottom - ell_height;
    points[9].x = rect.left;
    points[9].y = rect.bottom - yOffset;
    points[10].x = rect.left + xOffset;
    points[10].y = rect.bottom;
    points[11].x = rect.left + ell_width;
    points[11].y = rect.bottom;
    points[12].x = rect.right - ell_width;
    points[12].y = rect.bottom;
    points[13].x = rect.right - xOffset;
    points[13].y = rect.bottom;
    points[14].x = rect.right;
    points[14].y = rect.bottom - yOffset;
    points[15].x = rect.right;
    points[15].y = rect.bottom - ell_height;

    if (*dc).dclevel.flPath & DCPATH_CLOCKWISE as u32 != 0 {
        reverse_points(points.as_mut_ptr(), 16);
    }
    type_ = add_points(pPath, points.as_ptr(), 16, PT_BEZIERTO as BYTE);
    if type_.is_null() {
        PATH_UnlockPath(pPath);
        return 0;
    }
    *type_.add(0) = PT_MOVETO as BYTE;
    *type_.add(4) = PT_LINETO as BYTE;
    *type_.add(8) = PT_LINETO as BYTE;
    *type_.add(12) = PT_LINETO as BYTE;

    IntGdiCloseFigure(pPath);
    PATH_UnlockPath(pPath);
    1
}

#[no_mangle]
pub unsafe extern "C" fn PATH_Ellipse(dc: PDC, x1: INT, y1: INT, x2: INT, y2: INT) -> BOOL {
    let mut points: [POINT; 13] = core::mem::zeroed();
    let mut rect: RECTL = core::mem::zeroed();
    let xRadius: LONG;
    let yRadius: LONG;
    let xOffset: LONG;
    let yOffset: LONG;
    let mut left: POINT = core::mem::zeroed();
    let mut top: POINT = core::mem::zeroed();
    let mut right: POINT = core::mem::zeroed();
    let mut bottom: POINT = core::mem::zeroed();

    if PATH_CheckRect(dc, core::ptr::addr_of_mut!(rect), x1, y1, x2, y2) == 0 {
        return 1;
    }

    xRadius = w32r_RECTL_lGetWidth(core::ptr::addr_of_mut!(rect)) / 2;
    yRadius = w32r_RECTL_lGetHeight(core::ptr::addr_of_mut!(rect)) / 2;

    left.x = rect.left;
    left.y = rect.top + yRadius;
    top.x = rect.left + xRadius;
    top.y = rect.top;
    right.x = rect.right;
    right.y = rect.bottom - yRadius;
    bottom.x = rect.right - xRadius;
    bottom.y = rect.bottom;

    xOffset = EngMulDiv(w32r_RECTL_lGetWidth(core::ptr::addr_of_mut!(rect)), 55428475, 200000000);
    yOffset = EngMulDiv(w32r_RECTL_lGetHeight(core::ptr::addr_of_mut!(rect)), 55428475, 200000000);

    let pPath: PPATH = PATH_LockPath((*dc).dclevel.hPath);
    if pPath.is_null() {
        return 0;
    }

    points[0] = right;

    points[1] = right;
    points[1].y -= yOffset;
    points[2] = top;
    points[2].x += xOffset;

    points[3] = top;

    points[4] = top;
    points[4].x -= xOffset;
    points[5] = left;
    points[5].y -= yOffset;

    points[6] = left;

    points[7] = left;
    points[7].y += yOffset;
    points[8] = bottom;
    points[8].x -= xOffset;

    points[9] = bottom;

    points[10] = bottom;
    points[10].x += xOffset;
    points[11] = right;
    points[11].y += yOffset;

    points[12] = right;

    if (*dc).dclevel.flPath & DCPATH_CLOCKWISE != 0 {
        reverse_points(points.as_mut_ptr(), 13);
    }
    let type_ = add_points(pPath, points.as_ptr(), 13, PT_BEZIERTO as BYTE);
    if type_.is_null() {
        PATH_UnlockPath(pPath);
        return 0;
    }
    *type_.add(0) = PT_MOVETO as BYTE;

    IntGdiCloseFigure(pPath);
    PATH_UnlockPath(pPath);
    1
}

unsafe fn PATH_DoArcPart(
    pPath: PPATH,
    corners: *mut POINT,
    angleStart: f64,
    angleEnd: f64,
    startEntryType: BYTE,
) -> BOOL {

    let halfAngle: f64;
    let a: f64;
    let mut xNorm: [f32; 4] = [0.0; 4];
    let mut yNorm: [f32; 4] = [0.0; 4];
    let mut points: [POINT; 4] = core::mem::zeroed();
    let type_: *mut BYTE;
    let mut i: i32;
    let start: i32;

    halfAngle = (angleEnd - angleStart) / 2.0;
    if fabs(halfAngle) > 1e-8 {
        a = 4.0 / 3.0 * (1.0 - cos(halfAngle)) / sin(halfAngle);
        xNorm[0] = cos(angleStart) as f32;
        yNorm[0] = sin(angleStart) as f32;
        xNorm[1] = (xNorm[0] as f64 - a * yNorm[0] as f64) as f32;
        yNorm[1] = (yNorm[0] as f64 + a * xNorm[0] as f64) as f32;
        xNorm[3] = cos(angleEnd) as f32;
        yNorm[3] = sin(angleEnd) as f32;
        xNorm[2] = (xNorm[3] as f64 + a * yNorm[3] as f64) as f32;
        yNorm[2] = (yNorm[3] as f64 - a * xNorm[3] as f64) as f32;
    } else {
        i = 0;
        while i < 4 {
            xNorm[i as usize] = cos(angleStart) as f32;
            yNorm[i as usize] = sin(angleStart) as f32;
            i += 1;
        }
    }

    start = (startEntryType == 0) as i32;

    i = start;
    while i < 4 {
        if PATH_ScaleNormalizedPoint(
            corners,
            *(&xNorm[i as usize] as *const f32 as *const FLOATL),
            *(&yNorm[i as usize] as *const f32 as *const FLOATL),
            &mut points[i as usize],
        ) == 0
        {
            return 0;
        }
        i += 1;
    }

    type_ = add_points(
        pPath,
        points.as_ptr().offset(start as isize),
        (4 - start) as DWORD,
        PT_BEZIERTO as BYTE,
    );
    if type_.is_null() {
        return 0;
    }
    if start == 0 {
        *type_.offset(0) = startEntryType;
    }

    1
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn PATH_Arc(
    dc: PDC,
    x1: INT,
    y1: INT,
    x2: INT,
    y2: INT,
    xStart: INT,
    yStart: INT,
    xEnd: INT,
    yEnd: INT,
    direction: INT,
    lines: INT,
) -> BOOL {
    PATH_Arc_impl(dc, x1, y1, x2, y2, xStart, yStart, xEnd, yEnd, direction, lines)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn PATH_Arc(
    dc: PDC,
    x1: INT,
    y1: INT,
    x2: INT,
    y2: INT,
    xStart: INT,
    yStart: INT,
    xEnd: INT,
    yEnd: INT,
    direction: INT,
    lines: INT,
) -> BOOL {
    PATH_Arc_impl(dc, x1, y1, x2, y2, xStart, yStart, xEnd, yEnd, direction, lines)
}

unsafe fn PATH_Arc_impl(
    dc: PDC,
    mut x1: INT,
    mut y1: INT,
    mut x2: INT,
    mut y2: INT,
    xStart: INT,
    yStart: INT,
    xEnd: INT,
    yEnd: INT,
    direction: INT,
    lines: INT,
) -> BOOL {

    let mut angleStart: f64;
    let mut angleEnd: f64;
    let mut angleStartQuadrant: f64;
    let mut angleEndQuadrant: f64 = 0.0;
    let mut x: FLOATL = core::mem::zeroed();
    let mut y: FLOATL = core::mem::zeroed();
    let mut corners: [POINT; 2] = core::mem::zeroed();
    let mut pointStart: POINT = core::mem::zeroed();
    let mut pointEnd: POINT = core::mem::zeroed();
    let mut centre: POINT = core::mem::zeroed();
    let mut pointCurPos: POINT = core::mem::zeroed();
    let mut start: BOOL;
    let mut end: BOOL;
    let mut Ret: BOOL = 1;
    let mut temp: INT;
    let clockwise: BOOL;
    let pPath: PPATH;

    pPath = PATH_LockPath((*dc).dclevel.hPath);
    if pPath.is_null() {
        return 0;
    }

    if direction != 0 {
        clockwise = ((direction == AD_CLOCKWISE as INT) != false) as BOOL;
    } else {
        clockwise = (((*dc).dclevel.flPath & DCPATH_CLOCKWISE) != 0) as BOOL;
    }

    'arc: {
        if x1 == x2 || y1 == y2 {
            Ret = 1;
            break 'arc;
        }

        corners[0].x = x1;
        corners[0].y = y1;
        corners[1].x = x2;
        corners[1].y = y2;
        pointStart.x = xStart;
        pointStart.y = yStart;
        pointEnd.x = xEnd;
        pointEnd.y = yEnd;
        w32r_INTERNAL_LPTODP(dc, corners.as_mut_ptr(), 2);
        w32r_INTERNAL_LPTODP(dc, &mut pointStart, 1);
        w32r_INTERNAL_LPTODP(dc, &mut pointEnd, 1);

        if corners[0].x > corners[1].x {
            temp = corners[0].x;
            corners[0].x = corners[1].x;
            corners[1].x = temp;
        }
        if corners[0].y > corners[1].y {
            temp = corners[0].y;
            corners[0].y = corners[1].y;
            corners[1].y = temp;
        }

        PATH_NormalizePoint(corners.as_mut_ptr() as _, core::ptr::addr_of!(pointStart) as _, &mut x, &mut y);
        angleStart = atan2(
            *(&y as *const FLOATL as *const FLOAT) as f64,
            *(&x as *const FLOATL as *const FLOAT) as f64,
        );
        PATH_NormalizePoint(corners.as_mut_ptr() as _, core::ptr::addr_of!(pointEnd) as _, &mut x, &mut y);
        angleEnd = atan2(
            *(&y as *const FLOATL as *const FLOAT) as f64,
            *(&x as *const FLOATL as *const FLOAT) as f64,
        );

        if clockwise != 0 {
            if angleEnd <= angleStart {
                angleEnd += 2.0 * M_PI;
            }
        } else {
            if angleEnd >= angleStart {
                angleEnd -= 2.0 * M_PI;
            }
        }

        if (*(*dc).pdcattr).iGraphicsMode == GM_COMPATIBLE as INT {
            corners[1].x -= 1;
            corners[1].y -= 1;
        }

        if lines == GdiTypeArcTo as INT && (*pPath).newStroke != 0 {
            (*pPath).newStroke = 0;
            IntGetCurrentPositionEx(dc, core::ptr::addr_of_mut!(pointCurPos) as _);
            w32r_CoordLPtoDP(dc, &mut pointCurPos);
            if PATH_AddEntry(pPath, &pointCurPos, PT_MOVETO as BYTE) == 0 {
                Ret = 0;
                break 'arc;
            }
        }

        start = 1;
        end = 0;
        loop {
            if start != 0 {
                angleStartQuadrant = angleStart;
                if clockwise != 0 {
                    angleEndQuadrant = (floor(angleStart / M_PI_2) + 1.0) * M_PI_2;
                } else {
                    angleEndQuadrant = (ceil(angleStart / M_PI_2) - 1.0) * M_PI_2;
                }
            } else {
                angleStartQuadrant = angleEndQuadrant;
                if clockwise != 0 {
                    angleEndQuadrant += M_PI_2;
                } else {
                    angleEndQuadrant -= M_PI_2;
                }
            }

            if (clockwise != 0 && angleEnd < angleEndQuadrant)
                || (clockwise == 0 && angleEnd > angleEndQuadrant)
            {
                angleEndQuadrant = angleEnd;
                end = 1;
            }

            let entryType: BYTE = if start != 0 {
                if lines == GdiTypeArcTo as INT {
                    PT_LINETO as BYTE
                } else {
                    PT_MOVETO as BYTE
                }
            } else {
                0
            };

            PATH_DoArcPart(
                pPath,
                corners.as_mut_ptr(),
                angleStartQuadrant,
                angleEndQuadrant,
                entryType,
            );
            start = 0;

            if end != 0 {
                break;
            }
        }

        if lines == GdiTypeArcTo as INT {
            update_current_pos(pPath);
        } else if lines == GdiTypeChord as INT {
            IntGdiCloseFigure(pPath);
        } else if lines == GdiTypePie as INT {
            centre.x = (corners[0].x + corners[1].x) / 2;
            centre.y = (corners[0].y + corners[1].y) / 2;
            if PATH_AddEntry(pPath, &centre, (PT_LINETO | PT_CLOSEFIGURE) as BYTE) == 0 {
                Ret = 0;
            }
        }
    }

    PATH_UnlockPath(pPath);
    Ret
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn PATH_PolyBezierTo(dc: PDC, pts: *const POINT, cbPoints: DWORD) -> BOOL {
    PATH_PolyBezierTo_impl(dc, pts, cbPoints)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn PATH_PolyBezierTo(dc: PDC, pts: *const POINT, cbPoints: DWORD) -> BOOL {
    PATH_PolyBezierTo_impl(dc, pts, cbPoints)
}

unsafe fn PATH_PolyBezierTo_impl(dc: PDC, pts: *const POINT, cbPoints: DWORD) -> BOOL {
    let pPath: PPATH = PATH_LockPath((*dc).dclevel.hPath);
    if pPath.is_null() {
        return 0;
    }

    let ret = add_log_points_new_stroke(dc, pPath, pts, cbPoints, PT_BEZIERTO as BYTE);

    PATH_UnlockPath(pPath);
    ret
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn PATH_PolyBezier(
    dc: PDC,
    pts: *const POINT,
    cbPoints: DWORD,
) -> BOOL {
    PATH_PolyBezier_impl(dc, pts, cbPoints)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn PATH_PolyBezier(
    dc: PDC,
    pts: *const POINT,
    cbPoints: DWORD,
) -> BOOL {
    PATH_PolyBezier_impl(dc, pts, cbPoints)
}

unsafe fn PATH_PolyBezier_impl(
    dc: PDC,
    pts: *const POINT,
    cbPoints: DWORD,
) -> BOOL {
    let pPath: PPATH;
    let type_: *mut BYTE;

    pPath = PATH_LockPath((*dc).dclevel.hPath);
    if pPath.is_null() {
        return 0;
    }

    type_ = add_log_points(dc, pPath, pts, cbPoints, PT_BEZIERTO as BYTE);
    if type_.is_null() {
        return 0;
    }

    *type_.offset(0) = PT_MOVETO as BYTE;

    PATH_UnlockPath(pPath);
    1
}

#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "fastcall" fn PATH_PolyDraw(
    dc: PDC,
    pts: *const POINT,
    types: *const BYTE,
    cbPoints: DWORD,
) -> BOOL {
    PATH_PolyDraw_impl(dc, pts, types, cbPoints)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
unsafe extern "C" fn PATH_PolyDraw(
    dc: PDC,
    pts: *const POINT,
    types: *const BYTE,
    cbPoints: DWORD,
) -> BOOL {
    PATH_PolyDraw_impl(dc, pts, types, cbPoints)
}

unsafe fn PATH_PolyDraw_impl(
    dc: PDC,
    pts: *const POINT,
    types: *const BYTE,
    cbPoints: DWORD,
) -> BOOL {
    let pPath: PPATH;
    let mut orig_pos: POINT = core::mem::zeroed();
    let mut cur_pos: POINT = core::mem::zeroed();
    let mut i: ULONG;
    let mut lastmove: ULONG = 0;

    pPath = PATH_LockPath((*dc).dclevel.hPath);
    if pPath.is_null() {
        return 0;
    }

    if (*pPath).state != PATH_Open {
        PATH_UnlockPath(pPath);
        return 0;
    }

    i = 0;
    while i < ((*pPath).numEntriesUsed as _) {
        if *(*pPath).pFlags.add(i as usize) == PT_MOVETO as u8 {
            lastmove = i;
        }
        i += 1;
    }
    orig_pos = core::mem::transmute((*pPath).pos);

    IntGetCurrentPositionEx(dc, core::ptr::addr_of_mut!(cur_pos) as _);

    DbgPrint(
        b"PPD : Current pos X %d Y %d\n\0".as_ptr() as *const i8,
        (*pPath).pos.x,
        (*pPath).pos.y,
    );
    DbgPrint(
        b"PPD : last %d pos X %d Y %d\n\0".as_ptr() as *const i8,
        lastmove,
        (*(*pPath).pPoints.add(lastmove as usize)).x,
        (*(*pPath).pPoints.add(lastmove as usize)).y,
    );

    i = 0;
    while i < cbPoints {
        let ty = *types.add(i as usize);
        if ty == PT_MOVETO as u8 {
            (*pPath).newStroke = 1u8 as _;
            (*pPath).pos = core::mem::transmute(*pts.add(i as usize));
            w32r_IntLPtoDP(dc, core::ptr::addr_of_mut!((*pPath).pos) as _, 1);
            lastmove = (*pPath).numEntriesUsed as _;
        } else if ty == PT_LINETO as u8 || ty == (PT_LINETO as u8 | PT_CLOSEFIGURE as u8) {
            if add_log_points_new_stroke(dc, pPath, pts.add(i as usize), 1, PT_LINETO as u8) == 0 {
                PATH_UnlockPath(pPath);
                return 0;
            }
        } else if ty == PT_BEZIERTO as u8
            && (i + 2 < cbPoints)
            && (*types.add((i + 1) as usize) == PT_BEZIERTO as u8)
            && ((*types.add((i + 2) as usize) & !(PT_CLOSEFIGURE as u8)) == PT_BEZIERTO as u8)
        {
            if add_log_points_new_stroke(dc, pPath, pts.add(i as usize), 3, PT_BEZIERTO as u8) == 0
            {
                PATH_UnlockPath(pPath);
                return 0;
            }
            i += 2;
        } else {
            (*pPath).pos = core::mem::transmute(orig_pos);

            DbgPrint(
                b"PPD Bad   : pos X %d Y %d\n\0".as_ptr() as *const i8,
                (*pPath).pos.x,
                (*pPath).pos.y,
            );

            IntGdiMoveToEx(dc, cur_pos.x, cur_pos.y, core::ptr::null_mut());

            PATH_UnlockPath(pPath);
            return 0;
        }

        if (*types.add(i as usize) & PT_CLOSEFIGURE as u8) != 0 {
            close_figure(pPath);
            (*pPath).pos = *(*pPath).pPoints.add(lastmove as usize);
            DbgPrint(
                b"PPD close : pos X %d Y %d\n\0".as_ptr() as *const i8,
                (*pPath).pos.x,
                (*pPath).pos.y,
            );
        }

        i += 1;
    }
    PATH_UnlockPath(pPath);
    1
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn PATH_PolylineTo(
    dc: PDC,
    pts: *const POINT,
    cbPoints: DWORD,
) -> BOOL {
    PATH_PolylineTo_impl(dc, pts, cbPoints)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn PATH_PolylineTo(
    dc: PDC,
    pts: *const POINT,
    cbPoints: DWORD,
) -> BOOL {
    PATH_PolylineTo_impl(dc, pts, cbPoints)
}

unsafe fn PATH_PolylineTo_impl(dc: PDC, pts: *const POINT, cbPoints: DWORD) -> BOOL {
    let pPath: PPATH;
    let ret: BOOL;

    if cbPoints < 1 {
        return 0;
    }

    pPath = PATH_LockPath((*dc).dclevel.hPath);
    if pPath == core::ptr::null_mut() {
        return 0;
    }

    ret = add_log_points_new_stroke(dc, pPath, pts, cbPoints, PT_LINETO as BYTE);
    PATH_UnlockPath(pPath);
    ret
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn PATH_PolyPolygon(
    dc: PDC,
    pts: *const POINT,
    counts: *const INT,
    polygons: UINT,
) -> BOOL {
    PATH_PolyPolygon_impl(dc, pts, counts, polygons)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn PATH_PolyPolygon(
    dc: PDC,
    pts: *const POINT,
    counts: *const INT,
    polygons: UINT,
) -> BOOL {
    PATH_PolyPolygon_impl(dc, pts, counts, polygons)
}

unsafe fn PATH_PolyPolygon_impl(
    dc: PDC,
    pts: *const POINT,
    counts: *const INT,
    polygons: UINT,
) -> BOOL {
    let mut poly: UINT;
    let mut count: UINT;
    let mut type_: *mut BYTE;
    let pPath: PPATH;

    if polygons == 0 {
        return 0;
    }

    pPath = PATH_LockPath((*dc).dclevel.hPath);
    if pPath.is_null() {
        return 0;
    }

    count = 0;
    poly = 0;
    while poly < polygons {
        if *counts.offset(poly as isize) < 2 {
            PATH_UnlockPath(pPath);
            return 0;
        }
        count += *counts.offset(poly as isize) as UINT;
        poly += 1;
    }

    type_ = add_log_points(dc, pPath, pts, count as DWORD, PT_LINETO as BYTE);
    if type_.is_null() {
        PATH_UnlockPath(pPath);
        return 0;
    }

    poly = 0;
    while poly < polygons {
        *type_.offset(0) = PT_MOVETO as BYTE;
        *type_.offset((*counts.offset(poly as isize) - 1) as isize) =
            (PT_LINETO | PT_CLOSEFIGURE) as BYTE;
        type_ = type_.offset(*counts.offset(poly as isize) as isize);
        poly += 1;
    }

    PATH_UnlockPath(pPath);
    1
}

#[cfg(target_pointer_width = "64")]
#[no_mangle]
pub unsafe extern "C" fn PATH_PolyPolyline(
    dc: PDC,
    pts: *const POINT,
    counts: *const DWORD,
    polylines: DWORD,
) -> BOOL {
    PATH_PolyPolyline_impl(dc, pts, counts, polylines)
}

#[cfg(not(target_pointer_width = "64"))]
#[no_mangle]
pub unsafe extern "fastcall" fn PATH_PolyPolyline(
    dc: PDC,
    pts: *const POINT,
    counts: *const DWORD,
    polylines: DWORD,
) -> BOOL {
    PATH_PolyPolyline_impl(dc, pts, counts, polylines)
}

unsafe fn PATH_PolyPolyline_impl(
    dc: PDC,
    pts: *const POINT,
    counts: *const DWORD,
    polylines: DWORD,
) -> BOOL {

    let mut pt: POINT = core::mem::zeroed();
    let mut poly: ULONG;
    let mut point: ULONG;
    let mut i: ULONG = 0;

    let pPath = PATH_LockPath((*dc).dclevel.hPath as *mut core::ffi::c_void);
    if pPath.is_null() {
        return 0;
    }

    poly = 0;
    while poly < polylines {
        point = 0;
        while point < *counts.add(poly as usize) {
            pt = *pts.add(i as usize);
            w32r_CoordLPtoDP(dc, &mut pt);
            PATH_AddEntry(
                pPath,
                &pt,
                if point == 0 { PT_MOVETO as BYTE } else { PT_LINETO as BYTE },
            );
            point += 1;
            i += 1;
        }
        poly += 1;
    }

    PATH_UnlockPath(pPath);
    1
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn PATH_AddFlatBezier(
    pPath: PPATH,
    pt: *mut POINT,
    closed: BOOL,
) -> BOOL {
    PATH_AddFlatBezier_impl(pPath, pt, closed)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn PATH_AddFlatBezier(
    pPath: PPATH,
    pt: *mut POINT,
    closed: BOOL,
) -> BOOL {
    PATH_AddFlatBezier_impl(pPath, pt, closed)
}

unsafe fn PATH_AddFlatBezier_impl(
    pPath: PPATH,
    pt: *mut POINT,
    closed: BOOL,
) -> BOOL {

    let mut ret: BOOL = 0;
    let mut no: INT = 0;

    let pts = GDI_Bezier(pt, 4, &mut no);
    if pts.is_null() {
        return 0;
    }

    let mut i: INT = 1;
    while i < no {
        let flags = if i == no - 1 && closed != 0 {
            (PT_LINETO | PT_CLOSEFIGURE) as BYTE
        } else {
            PT_LINETO as BYTE
        };
        ret = PATH_AddEntry(pPath, &*pts.offset(i as isize), flags);
        if ret == 0 {
            break;
        }
        i += 1;
    }

    ExFreePoolWithTag(pts as *mut core::ffi::c_void, TAG_BEZIER);
    ret
}

#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "fastcall" fn PATH_FlattenPath(pPath: PPATH) -> PPATH {
    PATH_FlattenPath_impl(pPath)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
unsafe extern "C" fn PATH_FlattenPath(pPath: PPATH) -> PPATH {
    PATH_FlattenPath_impl(pPath)
}

unsafe fn PATH_FlattenPath_impl(pPath: PPATH) -> PPATH {
    let newPath: PPATH;
    let mut srcpt: INT;

    newPath = PATH_CreatePath((*pPath).numEntriesUsed);
    if newPath.is_null() {
        return core::ptr::null_mut();
    }

    srcpt = 0;
    while srcpt < (*pPath).numEntriesUsed {
        match (*(*pPath).pFlags.offset(srcpt as isize) as core::ffi::c_int)
            & !(PT_CLOSEFIGURE as core::ffi::c_int)
        {
            x if x == PT_MOVETO as core::ffi::c_int || x == PT_LINETO as core::ffi::c_int => {
                if PATH_AddEntry(
                    newPath,
                    (*pPath).pPoints.offset(srcpt as isize) as _,
                    *(*pPath).pFlags.offset(srcpt as isize),
                ) == 0
                {
                    PATH_UnlockPath(newPath);
                    PATH_Delete((*newPath).BaseObject.hHmgr);
                    return core::ptr::null_mut();
                }
            }
            x if x == PT_BEZIERTO as core::ffi::c_int => {
                if PATH_AddFlatBezier(
                    newPath,
                    (*pPath).pPoints.offset((srcpt - 1) as isize) as _,
                    (*(*pPath).pFlags.offset((srcpt + 2) as isize) & (PT_CLOSEFIGURE as u8))
                        as core::ffi::c_int,
                ) == 0
                {
                    PATH_UnlockPath(newPath);
                    PATH_Delete((*newPath).BaseObject.hHmgr);
                    return core::ptr::null_mut();
                }
                srcpt += 2;
            }
            _ => {}
        }
        srcpt += 1;
    }

    (*newPath).state = (*pPath).state;
    newPath
}

#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe extern "fastcall" fn PATH_PathToRegion(
    pPath: PPATH,
    Mode: INT,
    Rgn: PREGION,
) -> BOOL {
    PATH_PathToRegion_impl(pPath, Mode, Rgn)
}

#[no_mangle]
#[cfg(not(target_arch = "x86"))]
pub unsafe extern "C" fn PATH_PathToRegion(
    pPath: PPATH,
    Mode: INT,
    Rgn: PREGION,
) -> BOOL {
    PATH_PathToRegion_impl(pPath, Mode, Rgn)
}

unsafe fn PATH_PathToRegion_impl(
    pPath: PPATH,
    Mode: INT,
    Rgn: PREGION,
) -> BOOL {
    let mut i: i32;
    let mut pos: i32;
    let mut polygons: i32;
    let counts: PULONG;
    let Ret: i32;

    if (*pPath).numEntriesUsed == 0 {
        return 0;
    }

    counts = ExAllocatePoolWithTag(
        PagedPool,
        (((*pPath).numEntriesUsed / 2) as usize) * core::mem::size_of::<ULONG>(),
        TAG_PATH,
    ) as PULONG;
    if counts.is_null() {
        EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }

    pos = 0;
    polygons = 0;
    i = 1;
    while i < (*pPath).numEntriesUsed as i32 {
        if *(*pPath).pFlags.offset(i as isize) != PT_MOVETO as u8 {
            i += 1;
            continue;
        }
        *counts.offset(polygons as isize) = (i - pos) as ULONG;
        polygons += 1;
        pos = i;
        i += 1;
    }
    if i > pos + 1 {
        *counts.offset(polygons as isize) = (i - pos) as ULONG;
        polygons += 1;
    }

    Ret = REGION_SetPolyPolygonRgn(
        Rgn,
        (*pPath).pPoints,
        counts,
        polygons as _,
        Mode,
    );

    ExFreePoolWithTag(counts as *mut core::ffi::c_void, TAG_PATH);

    Ret
}

#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe extern "fastcall" fn PATH_FillPath(dc: PDC, pPath: PPATH) -> BOOL {
    PATH_FillPathEx(dc, pPath, core::ptr::null_mut())
}

#[no_mangle]
#[cfg(not(target_arch = "x86"))]
pub unsafe extern "C" fn PATH_FillPath(dc: PDC, pPath: PPATH) -> BOOL {
    PATH_FillPathEx(dc, pPath, core::ptr::null_mut())
}

#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe extern "fastcall" fn PATH_FillPathEx(
    dc: PDC,
    pPath: PPATH,
    pbrFill: PBRUSH,
) -> BOOL {
    PATH_FillPathEx_impl(dc, pPath, pbrFill)
}

#[no_mangle]
#[cfg(not(target_arch = "x86"))]
pub unsafe extern "C" fn PATH_FillPathEx(
    dc: PDC,
    pPath: PPATH,
    pbrFill: PBRUSH,
) -> BOOL {
    PATH_FillPathEx_impl(dc, pPath, pbrFill)
}

unsafe fn PATH_FillPathEx_impl(
    dc: PDC,
    pPath: PPATH,
    pbrFill: PBRUSH,
) -> BOOL {
    let mapMode: INT;
    let graphicsMode: INT;
    let ptViewportExt: SIZE;
    let ptWindowExt: SIZE;
    let ptViewportOrg: POINTL;
    let ptWindowOrg: POINTL;
    let mut xform: XFORML = core::mem::zeroed();
    let Rgn: PREGION;
    let pdcattr: PDC_ATTR = (*dc).pdcattr;

    /* Allocate a temporary region */
    Rgn = IntSysCreateRectpRgn(0, 0, 0, 0);
    if Rgn.is_null() {
        EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }

    if PATH_PathToRegion(pPath, (*pdcattr).jFillMode as _, Rgn) == 0 {
        REGION_Delete(Rgn);
        return 0;
    }

    /* Save the information about the old mapping mode */
    mapMode = (*pdcattr).iMapMode;
    ptViewportExt = (*pdcattr).szlViewportExt;
    ptViewportOrg = (*pdcattr).ptlViewportOrg;
    ptWindowExt = (*pdcattr).szlWindowExt;
    ptWindowOrg = (*pdcattr).ptlWindowOrg;

    w32r_MatrixS2XForm(
        core::ptr::addr_of_mut!(xform),
        core::ptr::addr_of_mut!((*(*dc).pdcattr).mxWorldToPage),
    );

    /* Set MM_TEXT */
    IntGdiSetMapMode(dc, MM_TEXT as INT);
    (*pdcattr).ptlViewportOrg.x = 0;
    (*pdcattr).ptlViewportOrg.y = 0;
    (*pdcattr).ptlWindowOrg.x = 0;
    (*pdcattr).ptlWindowOrg.y = 0;

    graphicsMode = (*pdcattr).iGraphicsMode;
    (*pdcattr).iGraphicsMode = GM_ADVANCED as INT;
    GreModifyWorldTransform(dc, core::ptr::addr_of_mut!(xform), MWT_IDENTITY);
    (*pdcattr).iGraphicsMode = graphicsMode;

    /* Paint the region */
    IntGdiFillRgn(dc, Rgn, pbrFill);
    REGION_Delete(Rgn);
    /* Restore the old mapping mode */
    IntGdiSetMapMode(dc, mapMode);
    (*pdcattr).szlViewportExt = ptViewportExt;
    (*pdcattr).ptlViewportOrg = ptViewportOrg;
    (*pdcattr).szlWindowExt = ptWindowExt;
    (*pdcattr).ptlWindowOrg = ptWindowOrg;

    let graphicsMode2 = (*pdcattr).iGraphicsMode;
    (*pdcattr).iGraphicsMode = GM_ADVANCED as INT;
    GreModifyWorldTransform(dc, core::ptr::addr_of_mut!(xform), MWT_SET);
    (*pdcattr).iGraphicsMode = graphicsMode2;
    1
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn PATH_StrokePath(dc: *mut DC, pPath: PPATH) -> BOOL {
    PATH_StrokePath_impl(dc, pPath)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn PATH_StrokePath(dc: *mut DC, pPath: PPATH) -> BOOL {
    PATH_StrokePath_impl(dc, pPath)
}

unsafe fn PATH_StrokePath_impl(dc: *mut DC, pPath: PPATH) -> BOOL {
    let mut ret: BOOL = FALSE as _;
    let mut nLinePts: INT = 0;
    let mut nAlloc: INT;
    let jOldFillMode: INT;
    let mut i: INT = 0;
    let mut pLinePts: *mut POINT = core::ptr::null_mut();
    let ptViewportOrg: POINT;
    let ptWindowOrg: POINT;
    let szViewportExt: SIZE;
    let szWindowExt: SIZE;
    let mapMode: DWORD;
    let graphicsMode: DWORD;
    let mut xform: XFORM = core::mem::zeroed();
    let pdcattr: PDC_ATTR = (*dc).pdcattr;
    let pbrLine: PBRUSH;
    let pNewPath: PPATH;

    pbrLine = (*dc).dclevel.pbrLine;
    if w32r_IntIsEffectiveWidePen(pbrLine) != 0 {
        pNewPath = PATH_WidenPathEx(dc, pPath);
        if !pNewPath.is_null() {
            jOldFillMode = (*pdcattr).jFillMode as _;
            (*pdcattr).jFillMode = WINDING as _;
            PATH_FillPathEx(dc, pNewPath, pbrLine);
            (*pdcattr).jFillMode = jOldFillMode as _;

            PATH_Delete((*pNewPath).BaseObject.hHmgr);
            return TRUE as _;
        }
    }

    mapMode = (*pdcattr).iMapMode as DWORD;

    szViewportExt = *w32r_DC_pszlViewportExt(dc);
    ptViewportOrg = core::mem::transmute((*(*dc).pdcattr).ptlViewportOrg);
    szWindowExt = (*(*dc).pdcattr).szlWindowExt;
    ptWindowOrg = core::mem::transmute((*(*dc).pdcattr).ptlWindowOrg);

    w32r_MatrixS2XForm(core::ptr::addr_of_mut!(xform) as _, core::ptr::addr_of_mut!((*(*dc).pdcattr).mxWorldToPage));

    (*pdcattr).iMapMode = MM_TEXT as _;
    (*pdcattr).ptlViewportOrg.x = 0;
    (*pdcattr).ptlViewportOrg.y = 0;
    (*pdcattr).ptlWindowOrg.x = 0;
    (*pdcattr).ptlWindowOrg.y = 0;
    graphicsMode = (*pdcattr).iGraphicsMode as DWORD;
    (*pdcattr).iGraphicsMode = GM_ADVANCED as _;
    GreModifyWorldTransform(dc, &mut xform as *mut XFORM as *mut XFORML, MWT_IDENTITY);
    (*pdcattr).iGraphicsMode = graphicsMode as _;

    'end: {
        nAlloc = (*pPath).numEntriesUsed as INT + 1 + 300;
        pLinePts = ExAllocatePoolWithTag(
            PagedPool,
            (nAlloc as usize) * core::mem::size_of::<POINT>(),
            TAG_PATH,
        ) as *mut POINT;
        if pLinePts.is_null() {
            EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
            break 'end;
        }
        nLinePts = 0;

        i = 0;
        while i < (*pPath).numEntriesUsed as INT {
            if (i == 0 || ((*(*pPath).pFlags.offset((i - 1) as isize)) & PT_CLOSEFIGURE as u8) != 0)
                && (*(*pPath).pFlags.offset(i as isize)) != PT_MOVETO as u8
            {
                break 'end;
            }

            let flag = *(*pPath).pFlags.offset(i as isize);
            if flag == PT_MOVETO as u8 {
                if nLinePts >= 2 {
                    IntGdiPolyline(dc, pLinePts as _, nLinePts);
                }
                nLinePts = 0;
                *pLinePts.offset(nLinePts as isize) = core::mem::transmute(*(*pPath).pPoints.offset(i as isize));
                nLinePts += 1;
            } else if flag == PT_LINETO as u8
                || flag == (PT_LINETO as u8 | PT_CLOSEFIGURE as u8)
            {
                *pLinePts.offset(nLinePts as isize) = core::mem::transmute(*(*pPath).pPoints.offset(i as isize));
                nLinePts += 1;
            } else if flag == PT_BEZIERTO as u8 {
                if *(*pPath).pFlags.offset((i + 1) as isize) != PT_BEZIERTO as u8
                    || ((*(*pPath).pFlags.offset((i + 2) as isize)) & !(PT_CLOSEFIGURE as u8))
                        != PT_BEZIERTO as u8
                {
                    ret = FALSE as _;
                    break 'end;
                } else {
                    let mut nBzrPts: INT = 0;
                    let nMinAlloc: INT;
                    let pBzrPts: *mut POINT =
                        GDI_Bezier((*pPath).pPoints.offset((i - 1) as isize) as _, 4, &mut nBzrPts);
                    nMinAlloc =
                        nLinePts + ((*pPath).numEntriesUsed as INT - i) + nBzrPts;
                    if nAlloc < nMinAlloc {
                        let Realloc: *mut POINT;
                        nAlloc = nMinAlloc * 2;

                        Realloc = ExAllocatePoolWithTag(
                            PagedPool,
                            (nAlloc as usize) * core::mem::size_of::<POINT>(),
                            TAG_PATH,
                        ) as *mut POINT;

                        if Realloc.is_null() {
                            ExFreePoolWithTag(pBzrPts as *mut _, TAG_BEZIER);
                            break 'end;
                        }

                        core::ptr::copy_nonoverlapping(
                            pLinePts,
                            Realloc,
                            nLinePts as usize,
                        );
                        ExFreePoolWithTag(pLinePts as *mut _, TAG_PATH);
                        pLinePts = Realloc;
                    }
                    core::ptr::copy_nonoverlapping(
                        pBzrPts.offset(1),
                        pLinePts.offset(nLinePts as isize),
                        (nBzrPts - 1) as usize,
                    );
                    nLinePts += nBzrPts - 1;
                    ExFreePoolWithTag(pBzrPts as *mut _, TAG_BEZIER);
                    i += 2;
                }
            } else {
                break 'end;
            }

            if (*(*pPath).pFlags.offset(i as isize)) & PT_CLOSEFIGURE as u8 != 0 {
                *pLinePts.offset(nLinePts as isize) = *pLinePts.offset(0);
                nLinePts += 1;
            }

            i += 1;
        }
        if nLinePts >= 2 {
            IntGdiPolyline(dc, pLinePts as _, nLinePts);
        }

        ret = TRUE as _;
    }

    if !pLinePts.is_null() {
        ExFreePoolWithTag(pLinePts as *mut _, TAG_PATH);
    }

    (*pdcattr).iMapMode = mapMode as _;
    (*pdcattr).szlWindowExt.cx = szWindowExt.cx;
    (*pdcattr).szlWindowExt.cy = szWindowExt.cy;
    (*pdcattr).ptlWindowOrg.x = ptWindowOrg.x;
    (*pdcattr).ptlWindowOrg.y = ptWindowOrg.y;

    (*pdcattr).szlViewportExt.cx = szViewportExt.cx;
    (*pdcattr).szlViewportExt.cy = szViewportExt.cy;
    (*pdcattr).ptlViewportOrg.x = ptViewportOrg.x;
    (*pdcattr).ptlViewportOrg.y = ptViewportOrg.y;

    XForm2MatrixS(core::ptr::addr_of_mut!((*(*dc).pdcattr).mxWorldToPage), core::ptr::addr_of_mut!(xform) as _);

    if i > 0 {
        let mut pt: POINT = core::mem::zeroed();
        IntGetCurrentPositionEx(dc, core::ptr::addr_of_mut!(pt) as _);
        w32r_IntDPtoLP(dc, core::ptr::addr_of_mut!(pt) as _, 1);
        IntGdiMoveToEx(dc, pt.x, pt.y, core::ptr::null_mut());
    }
    ret
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntGdiWidenPath(
    pPath: PPATH,
    penWidth: UINT,
    penStyle: UINT,
    eMiterLimit: FLOAT,
) -> PPATH {
    IntGdiWidenPath_impl(pPath, penWidth, penStyle, eMiterLimit)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntGdiWidenPath(
    pPath: PPATH,
    penWidth: UINT,
    penStyle: UINT,
    eMiterLimit: FLOAT,
) -> PPATH {
    IntGdiWidenPath_impl(pPath, penWidth, penStyle, eMiterLimit)
}

unsafe fn IntGdiWidenPath_impl(
    pPath: PPATH,
    penWidth: UINT,
    penStyle: UINT,
    eMiterLimit: FLOAT,
) -> PPATH {

    macro_rules! round {
        ($x:expr) => {{
            let v: f64 = $x;
            (if v > 0.0 { v + 0.5 } else { v - 0.5 }) as INT
        }};
    }

    let mut i: INT;
    let mut j: INT;
    let mut numStrokes: INT;
    let mut numOldStrokes: INT;
    let penWidthIn: INT;
    let mut penWidthOut: INT;
    let flat_path: PPATH;
    let mut pNewPath: PPATH = core::ptr::null_mut();
    let mut pStrokes: *mut PPATH = core::ptr::null_mut();
    let mut pOldStrokes: *mut PPATH;
    let mut pUpPath: PPATH;
    let mut pDownPath: PPATH;
    let mut type_: *mut BYTE;
    let joint: DWORD;
    let endcap: DWORD;
    let mut fpsave: KFLOATING_SAVE = core::mem::zeroed();

    endcap = PS_ENDCAP_MASK & penStyle;
    joint = PS_JOIN_MASK & penStyle;

    flat_path = PATH_FlattenPath(pPath);
    if flat_path.is_null() {
        return core::ptr::null_mut();
    }

    penWidthIn = (penWidth / 2) as INT;
    penWidthOut = (penWidth / 2) as INT;
    if penWidthIn + penWidthOut < penWidth as INT {
        penWidthOut += 1;
    }

    numStrokes = 0;

    'exit: {
        i = 0;
        j = 0;
        while i < (*flat_path).numEntriesUsed {
            let mut point: POINT = core::mem::zeroed();
            if (i == 0 || (*(*flat_path).pFlags.offset((i - 1) as isize) & PT_CLOSEFIGURE as BYTE) != 0)
                && (*(*flat_path).pFlags.offset(i as isize) != PT_MOVETO as BYTE)
            {
                break 'exit;
            }
            let flag = *(*flat_path).pFlags.offset(i as isize);
            let mut do_lineto = false;
            if flag == PT_MOVETO as BYTE {
                if numStrokes > 0 {
                    (**pStrokes.offset((numStrokes - 1) as isize)).state = PATH_Closed;
                }
                numOldStrokes = numStrokes;
                numStrokes += 1;
                j = 0;
                if numStrokes == 1 {
                    pStrokes = ExAllocatePoolWithTag(
                        PagedPool,
                        core::mem::size_of::<PPATH>(),
                        TAG_PATH,
                    ) as *mut PPATH;
                } else {
                    pOldStrokes = pStrokes;
                    pStrokes = ExAllocatePoolWithTag(
                        PagedPool,
                        numStrokes as usize * core::mem::size_of::<PPATH>(),
                        TAG_PATH,
                    ) as *mut PPATH;
                    if pStrokes.is_null() {
                        ExFreePoolWithTag(pOldStrokes as *mut core::ffi::c_void, TAG_PATH);
                        break 'exit;
                    }
                    RtlCopyMemory(
                        pStrokes as *mut core::ffi::c_void,
                        pOldStrokes as *const core::ffi::c_void,
                        numOldStrokes as usize * core::mem::size_of::<PPATH>(),
                    );
                    ExFreePoolWithTag(pOldStrokes as *mut core::ffi::c_void, TAG_PATH);
                }
                if pStrokes.is_null() {
                    break 'exit;
                }
                *pStrokes.offset((numStrokes - 1) as isize) = ExAllocatePoolWithTag(
                    PagedPool,
                    core::mem::size_of::<PATH>(),
                    TAG_PATH,
                ) as PPATH;
                PATH_InitGdiPath(*pStrokes.offset((numStrokes - 1) as isize));
                (**pStrokes.offset((numStrokes - 1) as isize)).state = PATH_Open;
                do_lineto = true;
            } else if flag == PT_LINETO as BYTE
                || flag == (PT_LINETO | PT_CLOSEFIGURE) as BYTE
            {
                do_lineto = true;
            } else if flag == PT_BEZIERTO as BYTE {
                // Should never happen
            } else {
                break 'exit;
            }

            if do_lineto {
                point.x = (*(*flat_path).pPoints.offset(i as isize)).x;
                point.y = (*(*flat_path).pPoints.offset(i as isize)).y;
                PATH_AddEntry(
                    *pStrokes.offset((numStrokes - 1) as isize),
                    &point,
                    *(*flat_path).pFlags.offset(i as isize),
                );
            }

            i += 1;
            j += 1;
        }

        pNewPath = PATH_CreatePath((*flat_path).numEntriesUsed);
        if pNewPath.is_null() {
            break 'exit;
        }

        w32r_KeSaveFloatingPointState(core::ptr::addr_of_mut!(fpsave) as _);

        i = 0;
        while i < numStrokes {
            pUpPath =
                ExAllocatePoolWithTag(PagedPool, core::mem::size_of::<PATH>(), TAG_PATH) as PPATH;
            PATH_InitGdiPath(pUpPath);
            (*pUpPath).state = PATH_Open;
            pDownPath =
                ExAllocatePoolWithTag(PagedPool, core::mem::size_of::<PATH>(), TAG_PATH) as PPATH;
            PATH_InitGdiPath(pDownPath);
            (*pDownPath).state = PATH_Open;

            let pStroke = *pStrokes.offset(i as isize);

            j = 0;
            while j < (*pStroke).numEntriesUsed {
                if (*(*pStroke).pFlags.offset(((*pStroke).numEntriesUsed - 1) as isize)
                    & PT_CLOSEFIGURE as BYTE)
                    == 0
                    && (j == 0 || j == (*pStroke).numEntriesUsed - 1)
                {
                    let xo: INT;
                    let yo: INT;
                    let xa: INT;
                    let ya: INT;
                    let theta: f64;
                    let mut pt: POINT = core::mem::zeroed();
                    let mut corners: [POINT; 2] = core::mem::zeroed();
                    if j == 0 {
                        xo = (*(*pStroke).pPoints.offset(j as isize)).x;
                        yo = (*(*pStroke).pPoints.offset(j as isize)).y;
                        xa = (*(*pStroke).pPoints.offset(1)).x;
                        ya = (*(*pStroke).pPoints.offset(1)).y;
                    } else {
                        xa = (*(*pStroke).pPoints.offset((j - 1) as isize)).x;
                        ya = (*(*pStroke).pPoints.offset((j - 1) as isize)).y;
                        xo = (*(*pStroke).pPoints.offset(j as isize)).x;
                        yo = (*(*pStroke).pPoints.offset(j as isize)).y;
                    }
                    theta = atan2((ya - yo) as f64, (xa - xo) as f64);
                    if endcap == PS_ENDCAP_SQUARE {
                        pt.x = xo + round!(sqrt(2.0) * penWidthOut as f64 * cos(M_PI_4 + theta));
                        pt.y = yo + round!(sqrt(2.0) * penWidthOut as f64 * sin(M_PI_4 + theta));
                        PATH_AddEntry(
                            pUpPath,
                            &pt,
                            if j == 0 { PT_MOVETO } else { PT_LINETO } as BYTE,
                        );
                        pt.x = xo + round!(sqrt(2.0) * penWidthIn as f64 * cos(-M_PI_4 + theta));
                        pt.y = yo + round!(sqrt(2.0) * penWidthIn as f64 * sin(-M_PI_4 + theta));
                        PATH_AddEntry(pUpPath, &pt, PT_LINETO as BYTE);
                    } else if endcap == PS_ENDCAP_FLAT {
                        pt.x = xo + round!(penWidthOut as f64 * cos(theta + M_PI_2));
                        pt.y = yo + round!(penWidthOut as f64 * sin(theta + M_PI_2));
                        PATH_AddEntry(
                            pUpPath,
                            &pt,
                            if j == 0 { PT_MOVETO } else { PT_LINETO } as BYTE,
                        );
                        pt.x = xo - round!(penWidthIn as f64 * cos(theta + M_PI_2));
                        pt.y = yo - round!(penWidthIn as f64 * sin(theta + M_PI_2));
                        PATH_AddEntry(pUpPath, &pt, PT_LINETO as BYTE);
                    } else {
                        corners[0].x = xo - penWidthIn;
                        corners[0].y = yo - penWidthIn;
                        corners[1].x = xo + penWidthOut;
                        corners[1].y = yo + penWidthOut;
                        PATH_DoArcPart(
                            pUpPath,
                            corners.as_mut_ptr(),
                            theta + M_PI_2,
                            theta + 3.0 * M_PI_4,
                            if j == 0 { PT_MOVETO } else { FALSE as u32 } as BYTE,
                        );
                        PATH_DoArcPart(
                            pUpPath,
                            corners.as_mut_ptr(),
                            theta + 3.0 * M_PI_4,
                            theta + M_PI,
                            FALSE as BYTE,
                        );
                        PATH_DoArcPart(
                            pUpPath,
                            corners.as_mut_ptr(),
                            theta + M_PI,
                            theta + 5.0 * M_PI_4,
                            FALSE as BYTE,
                        );
                        PATH_DoArcPart(
                            pUpPath,
                            corners.as_mut_ptr(),
                            theta + 5.0 * M_PI_4,
                            theta + 3.0 * M_PI_2,
                            FALSE as BYTE,
                        );
                    }
                } else {
                    let previous: INT;
                    let next: INT;
                    let xa: f64;
                    let ya: f64;
                    let xb: f64;
                    let yb: f64;
                    let xo: f64;
                    let yo: f64;
                    let mut alpha: f64;
                    let theta: f64;
                    let miterWidth: f64;
                    let mut _joint: DWORD = joint; // nolint: let-shadow (the C declares its own _joint here)
                    let mut pt: POINT = core::mem::zeroed();
                    let pInsidePath: PPATH;
                    let pOutsidePath: PPATH;
                    if j > 0 && j < (*pStroke).numEntriesUsed - 1 {
                        previous = j - 1;
                        next = j + 1;
                    } else if j == 0 {
                        previous = (*pStroke).numEntriesUsed - 1;
                        next = j + 1;
                    } else {
                        previous = j - 1;
                        next = 0;
                    }
                    xo = (*(*pStroke).pPoints.offset(j as isize)).x as f64;
                    yo = (*(*pStroke).pPoints.offset(j as isize)).y as f64;
                    xa = (*(*pStroke).pPoints.offset(previous as isize)).x as f64;
                    ya = (*(*pStroke).pPoints.offset(previous as isize)).y as f64;
                    xb = (*(*pStroke).pPoints.offset(next as isize)).x as f64;
                    yb = (*(*pStroke).pPoints.offset(next as isize)).y as f64;
                    theta = atan2(yo - ya, xo - xa);
                    alpha = atan2(yb - yo, xb - xo) - theta;
                    if alpha > 0.0 {
                        alpha -= M_PI;
                    } else {
                        alpha += M_PI;
                    }
                    if _joint == PS_JOIN_MITER && (eMiterLimit as f64) < fabs(1.0 / sin(alpha / 2.0))
                    {
                        _joint = PS_JOIN_BEVEL;
                    }
                    if alpha > 0.0 {
                        pInsidePath = pUpPath;
                        pOutsidePath = pDownPath;
                    } else if alpha < 0.0 {
                        pInsidePath = pDownPath;
                        pOutsidePath = pUpPath;
                    } else {
                        j += 1;
                        continue;
                    }
                    if alpha > 0.0 {
                        pt.x = xo as INT - round!(penWidthIn as f64 * cos(theta + M_PI_2));
                        pt.y = yo as INT - round!(penWidthIn as f64 * sin(theta + M_PI_2));
                    } else {
                        pt.x = xo as INT + round!(penWidthIn as f64 * cos(theta + M_PI_2));
                        pt.y = yo as INT + round!(penWidthIn as f64 * sin(theta + M_PI_2));
                    }
                    PATH_AddEntry(pInsidePath, &pt, PT_LINETO as BYTE);
                    if alpha > 0.0 {
                        pt.x = xo as INT + round!(penWidthIn as f64 * cos(M_PI_2 + alpha + theta));
                        pt.y = yo as INT + round!(penWidthIn as f64 * sin(M_PI_2 + alpha + theta));
                    } else {
                        pt.x = xo as INT - round!(penWidthIn as f64 * cos(M_PI_2 + alpha + theta));
                        pt.y = yo as INT - round!(penWidthIn as f64 * sin(M_PI_2 + alpha + theta));
                    }
                    PATH_AddEntry(pInsidePath, &pt, PT_LINETO as BYTE);
                    if _joint == PS_JOIN_MITER {
                        miterWidth = fabs(penWidthOut as f64 / cos(M_PI_2 - fabs(alpha) / 2.0));
                        pt.x = xo as INT + round!(miterWidth * cos(theta + alpha / 2.0));
                        pt.y = yo as INT + round!(miterWidth * sin(theta + alpha / 2.0));
                        PATH_AddEntry(pOutsidePath, &pt, PT_LINETO as BYTE);
                    } else if _joint == PS_JOIN_BEVEL {
                        if alpha > 0.0 {
                            pt.x = xo as INT + round!(penWidthOut as f64 * cos(theta + M_PI_2));
                            pt.y = yo as INT + round!(penWidthOut as f64 * sin(theta + M_PI_2));
                        } else {
                            pt.x = xo as INT - round!(penWidthOut as f64 * cos(theta + M_PI_2));
                            pt.y = yo as INT - round!(penWidthOut as f64 * sin(theta + M_PI_2));
                        }
                        PATH_AddEntry(pOutsidePath, &pt, PT_LINETO as BYTE);
                        if alpha > 0.0 {
                            pt.x = xo as INT
                                - round!(penWidthOut as f64 * cos(M_PI_2 + alpha + theta));
                            pt.y = yo as INT
                                - round!(penWidthOut as f64 * sin(M_PI_2 + alpha + theta));
                        } else {
                            pt.x = xo as INT
                                + round!(penWidthOut as f64 * cos(M_PI_2 + alpha + theta));
                            pt.y = yo as INT
                                + round!(penWidthOut as f64 * sin(M_PI_2 + alpha + theta));
                        }
                        PATH_AddEntry(pOutsidePath, &pt, PT_LINETO as BYTE);
                    } else {
                        if alpha > 0.0 {
                            pt.x = xo as INT + round!(penWidthOut as f64 * cos(theta + M_PI_2));
                            pt.y = yo as INT + round!(penWidthOut as f64 * sin(theta + M_PI_2));
                        } else {
                            pt.x = xo as INT - round!(penWidthOut as f64 * cos(theta + M_PI_2));
                            pt.y = yo as INT - round!(penWidthOut as f64 * sin(theta + M_PI_2));
                        }
                        PATH_AddEntry(pOutsidePath, &pt, PT_BEZIERTO as BYTE);
                        pt.x = xo as INT + round!(penWidthOut as f64 * cos(theta + alpha / 2.0));
                        pt.y = yo as INT + round!(penWidthOut as f64 * sin(theta + alpha / 2.0));
                        PATH_AddEntry(pOutsidePath, &pt, PT_BEZIERTO as BYTE);
                        if alpha > 0.0 {
                            pt.x = xo as INT
                                - round!(penWidthOut as f64 * cos(M_PI_2 + alpha + theta));
                            pt.y = yo as INT
                                - round!(penWidthOut as f64 * sin(M_PI_2 + alpha + theta));
                        } else {
                            pt.x = xo as INT
                                + round!(penWidthOut as f64 * cos(M_PI_2 + alpha + theta));
                            pt.y = yo as INT
                                + round!(penWidthOut as f64 * sin(M_PI_2 + alpha + theta));
                        }
                        PATH_AddEntry(pOutsidePath, &pt, PT_BEZIERTO as BYTE);
                    }
                }
                j += 1;
            }

            type_ = add_points(
                pNewPath,
                (*pUpPath).pPoints as _,
                (*pUpPath).numEntriesUsed as DWORD,
                PT_LINETO as BYTE,
            );
            *type_.offset(0) = PT_MOVETO as BYTE;
            reverse_points((*pDownPath).pPoints as _, (*pDownPath).numEntriesUsed as UINT);
            type_ = add_points(
                pNewPath,
                (*pDownPath).pPoints as _,
                (*pDownPath).numEntriesUsed as DWORD,
                PT_LINETO as BYTE,
            );
            if (*(*pStroke).pFlags.offset(((*pStroke).numEntriesUsed - 1) as isize)
                & PT_CLOSEFIGURE as BYTE)
                != 0
            {
                *type_.offset(0) = PT_MOVETO as BYTE;
            }

            PATH_DestroyGdiPath(pStroke);
            ExFreePoolWithTag(pStroke as *mut core::ffi::c_void, TAG_PATH);
            PATH_DestroyGdiPath(pUpPath);
            ExFreePoolWithTag(pUpPath as *mut core::ffi::c_void, TAG_PATH);
            PATH_DestroyGdiPath(pDownPath);
            ExFreePoolWithTag(pDownPath as *mut core::ffi::c_void, TAG_PATH);

            i += 1;
        }

        (*pNewPath).state = PATH_Closed;
        PATH_UnlockPath(pNewPath);

        w32r_KeRestoreFloatingPointState(core::ptr::addr_of_mut!(fpsave) as _);
    }

    if !pStrokes.is_null() {
        ExFreePoolWithTag(pStrokes as *mut core::ffi::c_void, TAG_PATH);
    }
    let hpathToDelete: HPATH = (*flat_path).BaseObject.hHmgr;
    PATH_UnlockPath(flat_path);
    PATH_Delete(hpathToDelete);

    pNewPath
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn PATH_WidenPath(dc: *mut DC) -> PPATH {
    PATH_WidenPath_impl(dc)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn PATH_WidenPath(dc: *mut DC) -> PPATH {
    PATH_WidenPath_impl(dc)
}

unsafe fn PATH_WidenPath_impl(dc: *mut DC) -> PPATH {
    let pPath: PPATH;
    let pNewPath: PPATH;

    pPath = PATH_LockPath((*dc).dclevel.hPath);
    if pPath.is_null() {
        EngSetLastError(ERROR_CAN_NOT_COMPLETE);
        return core::ptr::null_mut();
    }

    pNewPath = PATH_WidenPathEx(dc, pPath);
    PATH_UnlockPath(pPath);
    pNewPath
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn PATH_WidenPathEx(dc: *mut DC, pPath: PPATH) -> PPATH {
    PATH_WidenPathEx_impl(dc, pPath)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn PATH_WidenPathEx(dc: *mut DC, pPath: PPATH) -> PPATH {
    PATH_WidenPathEx_impl(dc, pPath)
}

unsafe fn PATH_WidenPathEx_impl(dc: *mut DC, pPath: PPATH) -> PPATH {
    let mut size: INT;
    let penWidth: UINT;
    let penStyle: UINT;
    let obj_type: DWORD;
    let elp: LPEXTLOGPEN;
    let pdcattr: PDC_ATTR = (*dc).pdcattr;

    if (*pPath).state != PATH_Closed as _ {
        EngSetLastError(ERROR_CAN_NOT_COMPLETE);
        return core::ptr::null_mut();
    }

    size = GreGetObject((*pdcattr).hpen, 0, core::ptr::null_mut());
    if size == 0 {
        EngSetLastError(ERROR_CAN_NOT_COMPLETE);
        return core::ptr::null_mut();
    }

    elp = ExAllocatePoolWithTag(PagedPool, size as usize, TAG_PATH) as LPEXTLOGPEN;
    if elp.is_null() {
        EngSetLastError(ERROR_OUTOFMEMORY);
        return core::ptr::null_mut();
    }

    GreGetObject((*pdcattr).hpen, size, elp as *mut core::ffi::c_void);

    obj_type = w32r_GDI_HANDLE_GET_TYPE((*pdcattr).hpen);
    if obj_type == GDI_OBJECT_TYPE_PEN {
        penStyle = (*(elp as *mut LOGPEN)).lopnStyle;
    } else if obj_type == GDI_OBJECT_TYPE_EXTPEN {
        penStyle = (*elp).elpPenStyle;
    } else {
        EngSetLastError(ERROR_CAN_NOT_COMPLETE);
        ExFreePoolWithTag(elp as *mut core::ffi::c_void, TAG_PATH);
        return core::ptr::null_mut();
    }

    penWidth = (*elp).elpWidth;
    ExFreePoolWithTag(elp as *mut core::ffi::c_void, TAG_PATH);

    if obj_type == GDI_OBJECT_TYPE_EXTPEN && (PS_TYPE_MASK & penStyle) == PS_COSMETIC {
        EngSetLastError(ERROR_CAN_NOT_COMPLETE);
        return core::ptr::null_mut();
    }

    IntGdiWidenPath(pPath, penWidth, penStyle, (*dc).dclevel.laPath.eMiterLimit as _)
}

unsafe fn int_from_fixed(f: FIXED) -> INT {
    if f.fract >= 0x8000 {
        (f.value + 1) as INT
    } else {
        f.value as INT
    }
}

#[cfg(target_pointer_width = "64")]
unsafe extern "C" fn PATH_BezierTo(pPath: PPATH, lppt: *mut POINT, n: INT) {
    PATH_BezierTo_impl(pPath, lppt, n)
}

#[cfg(not(target_pointer_width = "64"))]
unsafe extern "fastcall" fn PATH_BezierTo(pPath: PPATH, lppt: *mut POINT, n: INT) {
    PATH_BezierTo_impl(pPath, lppt, n)
}

unsafe fn PATH_BezierTo_impl(pPath: PPATH, lppt: *mut POINT, mut n: INT) {
    if n < 2 {
        return;
    }

    if n == 2 {
        PATH_AddEntry(pPath, core::ptr::addr_of!(*lppt.offset(1)), PT_LINETO as BYTE);
    } else if n == 3 {
        add_points(pPath, lppt, 3, PT_BEZIERTO as BYTE);
    } else {
        let mut pt: [POINT; 3] = core::mem::zeroed();
        let mut i: INT = 0;

        pt[2] = *lppt.offset(0);
        n -= 1;

        while n > 2 {
            pt[0] = pt[2];
            pt[1] = *lppt.offset((i + 1) as isize);
            pt[2].x = ((*lppt.offset((i + 2) as isize)).x + (*lppt.offset((i + 1) as isize)).x) / 2;
            pt[2].y = ((*lppt.offset((i + 2) as isize)).y + (*lppt.offset((i + 1) as isize)).y) / 2;
            add_points(pPath, pt.as_ptr(), 3, PT_BEZIERTO as BYTE);
            n -= 1;
            i += 1;
        }

        pt[0] = pt[2];
        pt[1] = *lppt.offset((i + 1) as isize);
        pt[2] = *lppt.offset((i + 2) as isize);
        add_points(pPath, pt.as_ptr(), 3, PT_BEZIERTO as BYTE);
    }
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn PATH_add_outline(
    dc: PDC,
    pPath: PPATH,
    x: INT,
    y: INT,
    header: *mut TTPOLYGONHEADER,
    size: DWORD,
) -> BOOL {
    PATH_add_outline_impl(dc, pPath, x, y, header, size)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn PATH_add_outline(
    dc: PDC,
    pPath: PPATH,
    x: INT,
    y: INT,
    header: *mut TTPOLYGONHEADER,
    size: DWORD,
) -> BOOL {
    PATH_add_outline_impl(dc, pPath, x, y, header, size)
}

unsafe fn PATH_add_outline_impl(
    _dc: PDC,
    pPath: PPATH,
    x: INT,
    y: INT,
    mut header: *mut TTPOLYGONHEADER,
    size: DWORD,
) -> BOOL {
    let mut pt: POINT = core::mem::zeroed();
    let mut bResult: BOOL = 0;

    let start = header;

    'cleanup: {
        while (header as *mut i8) < (start as *mut i8).add(size as usize) {
            if (*header).dwType != TT_POLYGON_TYPE as DWORD {
                break 'cleanup;
            }

            pt.x = x + int_from_fixed((*header).pfxStart.x);
            pt.y = y - int_from_fixed((*header).pfxStart.y);
            PATH_AddEntry(pPath, &pt, PT_MOVETO as BYTE);

            let mut curve = header.add(1) as *mut TTPOLYCURVE;

            while (curve as *mut i8) < (header as *mut i8).add((*header).cb as usize) {
                match (*curve).wType as u32 {
                    TT_PRIM_LINE => {
                        let mut i: WORD = 0;
                        while i < (*curve).cpfx {
                            pt.x = x + int_from_fixed((*curve).apfx.as_ptr().add(i as usize).read().x);
                            pt.y = y - int_from_fixed((*curve).apfx.as_ptr().add(i as usize).read().y);
                            PATH_AddEntry(pPath, &pt, PT_LINETO as BYTE);
                            i += 1;
                        }
                    }
                    TT_PRIM_QSPLINE | TT_PRIM_CSPLINE => {
                        let pts = ExAllocatePoolWithTag(
                            PagedPool,
                            (((*curve).cpfx as usize) + 1) * core::mem::size_of::<POINT>(),
                            TAG_PATH,
                        ) as *mut POINT;

                        if pts.is_null() {
                            break 'cleanup;
                        }

                        let ptfx = ((curve as *mut i8).sub(core::mem::size_of::<POINTFX>())
                            as *mut POINTFX)
                            .read();

                        (*pts.add(0)).x = x + int_from_fixed(ptfx.x);
                        (*pts.add(0)).y = y - int_from_fixed(ptfx.y);

                        let mut i: WORD = 0;
                        while i < (*curve).cpfx {
                            (*pts.add((i + 1) as usize)).x =
                                x + int_from_fixed((*curve).apfx.as_ptr().add(i as usize).read().x);
                            (*pts.add((i + 1) as usize)).y =
                                y - int_from_fixed((*curve).apfx.as_ptr().add(i as usize).read().y);
                            i += 1;
                        }

                        PATH_BezierTo(pPath, pts, ((*curve).cpfx as INT) + 1);

                        ExFreePoolWithTag(pts as *mut core::ffi::c_void, TAG_PATH);
                    }
                    _ => {
                        break 'cleanup;
                    }
                }

                curve = (*curve).apfx.as_mut_ptr().add((*curve).cpfx as usize) as *mut TTPOLYCURVE;
            }
            header = (header as *mut i8).add((*header).cb as usize) as *mut TTPOLYGONHEADER;
        }

        bResult = 1;
    }

    IntGdiCloseFigure(pPath);
    bResult
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn PATH_ExtTextOut(
    dc: PDC,
    x: INT,
    y: INT,
    flags: UINT,
    lprc: *const RECTL,
    str: LPCWSTR,
    count: UINT,
    dx: *const INT,
) -> BOOL {
    PATH_ExtTextOut_impl(dc, x, y, flags, lprc, str, count, dx)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn PATH_ExtTextOut(
    dc: PDC,
    x: INT,
    y: INT,
    flags: UINT,
    lprc: *const RECTL,
    str: LPCWSTR,
    count: UINT,
    dx: *const INT,
) -> BOOL {
    PATH_ExtTextOut_impl(dc, x, y, flags, lprc, str, count, dx)
}

unsafe fn PATH_ExtTextOut_impl(
    dc: PDC,
    x: INT,
    y: INT,
    flags: UINT,
    _lprc: *const RECTL,
    str: LPCWSTR,
    count: UINT,
    dx: *const INT,
) -> BOOL {
    let pPath: PPATH;
    let mut ggo_flags: c_uint = GGO_NATIVE;
    let mut offset: POINT = POINT { x: 0, y: 0 };

    pPath = PATH_LockPath((*dc).dclevel.hPath);
    if pPath.is_null() {
        return FALSE as BOOL;
    }

    if (*pPath).state != PATH_Open {
        err_msg(b"PATH_ExtTextOut not open\n\0".as_ptr());
        return FALSE as BOOL;
    }

    if count == 0 {
        return TRUE as BOOL;
    }
    if (flags & ETO_GLYPH_INDEX) != 0 {
        ggo_flags |= GGO_GLYPH_INDEX;
    }

    let mut idx: c_uint = 0;
    while idx < count {
        let mut identity: MAT2 = MAT2 {
            eM11: FIXED { fract: 0, value: 1 },
            eM12: FIXED { fract: 0, value: 0 },
            eM21: FIXED { fract: 0, value: 0 },
            eM22: FIXED { fract: 0, value: 1 },
        };
        let mut gm: GLYPHMETRICS = core::mem::zeroed();
        let dwSize: DWORD;
        let outline: *mut c_void;

        dwSize = w32r_ftGdiGetGlyphOutline(
            dc,
            *str.offset(idx as isize),
            ggo_flags,
            &mut gm,
            0,
            core::ptr::null_mut(),
            &mut identity,
            TRUE as BOOL,
        );
        if dwSize == GDI_ERROR {
            idx += 1;
            continue;
        }

        if dwSize != 0 {
            outline = ExAllocatePoolWithTag(PagedPool, dwSize as usize, TAG_PATH);
            if outline.is_null() {
                PATH_UnlockPath(pPath);
                return FALSE as BOOL;
            }

            w32r_ftGdiGetGlyphOutline(
                dc,
                *str.offset(idx as isize),
                ggo_flags,
                &mut gm,
                dwSize,
                outline,
                &mut identity,
                TRUE as BOOL,
            );

            PATH_add_outline(
                dc,
                pPath,
                x + offset.x,
                y + offset.y,
                outline as *mut TTPOLYGONHEADER,
                dwSize,
            );

            ExFreePoolWithTag(outline, TAG_PATH);
        }

        if !dx.is_null() {
            if (flags & ETO_PDY) != 0 {
                offset.x += *dx.offset((idx * 2) as isize);
                offset.y += *dx.offset((idx * 2 + 1) as isize);
            } else {
                offset.x += *dx.offset(idx as isize);
            }
        } else {
            offset.x += gm.gmCellIncX as INT;
            offset.y += gm.gmCellIncY as INT;
        }

        idx += 1;
    }
    PATH_UnlockPath(pPath);
    TRUE as BOOL
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn NtGdiAbortPath(hDC: HDC) -> BOOL {
    let dc: PDC = DC_LockDc(hDC);
    if dc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE as u32);
        return (FALSE as _);
    }

    if (*dc).dclevel.hPath.is_null() {
        DC_UnlockDc(dc);
        return (TRUE as _);
    }

    if PATH_Delete((*dc).dclevel.hPath) == (FALSE as _) {
        DC_UnlockDc(dc);
        return (FALSE as _);
    }

    (*dc).dclevel.hPath = core::ptr::null_mut();
    (*dc).dclevel.flPath &= !DCPATH_ACTIVE;

    DC_UnlockDc(dc);
    (TRUE as _)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn NtGdiAbortPath(hDC: HDC) -> BOOL {
    let dc: PDC = DC_LockDc(hDC);
    if dc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE as u32);
        return FALSE as _;
    }

    if (*dc).dclevel.hPath.is_null() {
        DC_UnlockDc(dc);
        return TRUE as _;
    }

    if PATH_Delete((*dc).dclevel.hPath) == (FALSE as _) {
        DC_UnlockDc(dc);
        return FALSE as _;
    }

    (*dc).dclevel.hPath = core::ptr::null_mut();
    (*dc).dclevel.flPath &= !DCPATH_ACTIVE;

    DC_UnlockDc(dc);
    TRUE as _
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn NtGdiBeginPath(hDC: HDC) -> BOOL {
    let pPath: PPATH;
    let dc: PDC;

    dc = DC_LockDc(hDC);
    if dc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return (FALSE as _);
    }

    if ((*dc).dclevel.flPath & DCPATH_ACTIVE as u32) != 0
        && ((*dc).dclevel.flPath & DCPATH_SAVE as u32) == 0
    {
        DC_UnlockDc(dc);
        return (TRUE as _);
    }

    if !(*dc).dclevel.hPath.is_null() {
        if ((*dc).dclevel.flPath & DCPATH_SAVE as u32) == 0 {
            if PATH_Delete((*dc).dclevel.hPath) == 0 {
                DC_UnlockDc(dc);
                return (FALSE as _);
            }
        } else {
            (*dc).dclevel.flPath &= !(DCPATH_SAVE as u32 | DCPATH_ACTIVE as u32);
            (*dc).dclevel.hPath = core::ptr::null_mut();
        }
    }

    pPath = PATH_CreatePath(NUM_ENTRIES_INITIAL as core::ffi::c_int);
    (*dc).dclevel.flPath |= DCPATH_ACTIVE as u32;
    (*dc).dclevel.hPath = (*pPath).BaseObject.hHmgr;
    IntGetCurrentPositionEx(dc, core::ptr::addr_of_mut!((*pPath).pos));
    w32r_IntLPtoDP(dc, (core::ptr::addr_of_mut!((*pPath).pos) as _), 1);
    PATH_UnlockPath(pPath);
    DC_UnlockDc(dc);

    if pPath.is_null() {
        return (FALSE as _);
    }
    (TRUE as _)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn NtGdiBeginPath(hDC: HDC) -> BOOL {
    let pPath: PPATH;
    let dc: PDC;

    dc = DC_LockDc(hDC);
    if dc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return FALSE as _;
    }

    if ((*dc).dclevel.flPath & DCPATH_ACTIVE as u32) != 0
        && ((*dc).dclevel.flPath & DCPATH_SAVE as u32) == 0
    {
        DC_UnlockDc(dc);
        return TRUE as _;
    }

    if !(*dc).dclevel.hPath.is_null() {
        if ((*dc).dclevel.flPath & DCPATH_SAVE as u32) == 0 {
            if PATH_Delete((*dc).dclevel.hPath) == 0 {
                DC_UnlockDc(dc);
                return FALSE as _;
            }
        } else {
            (*dc).dclevel.flPath &= !(DCPATH_SAVE as u32 | DCPATH_ACTIVE as u32);
            (*dc).dclevel.hPath = core::ptr::null_mut();
        }
    }

    pPath = PATH_CreatePath(NUM_ENTRIES_INITIAL as core::ffi::c_int);
    (*dc).dclevel.flPath |= DCPATH_ACTIVE as u32;
    (*dc).dclevel.hPath = (*pPath).BaseObject.hHmgr;
    IntGetCurrentPositionEx(dc, core::ptr::addr_of_mut!((*pPath).pos));
    w32r_IntLPtoDP(dc, core::ptr::addr_of_mut!((*pPath).pos) as _, 1);
    PATH_UnlockPath(pPath);
    DC_UnlockDc(dc);

    if pPath.is_null() {
        return FALSE as _;
    }
    TRUE as _
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiCloseFigure(hDC: HDC) -> BOOL {
    let mut Ret: BOOL = 0;
    let pDc: PDC;
    let pPath: PPATH;

    pDc = DC_LockDc(hDC);
    if pDc.is_null() {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    pPath = PATH_LockPath((*pDc).dclevel.hPath);
    if pPath.is_null() {
        DC_UnlockDc(pDc);
        return 0;
    }

    if (*pPath).state == PATH_Open {
        IntGdiCloseFigure(pPath);
        Ret = 1;
    } else {
        EngSetLastError(ERROR_CAN_NOT_COMPLETE);
    }

    PATH_UnlockPath(pPath);
    DC_UnlockDc(pDc);
    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiEndPath(hDC: HDC) -> BOOL {
    let mut ret: BOOL = 1;
    let pPath: PPATH;
    let dc: PDC;

    dc = DC_LockDc(hDC);
    if dc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }

    pPath = PATH_LockPath((*dc).dclevel.hPath as *mut core::ffi::c_void);
    if pPath.is_null() {
        DC_UnlockDc(dc);
        return 0;
    }

    if (*pPath).state != PATH_Open || ((*dc).dclevel.flPath & DCPATH_ACTIVE) == 0 {
        EngSetLastError(ERROR_CAN_NOT_COMPLETE);
        ret = 0;
    } else {
        (*pPath).state = PATH_Closed;
        (*dc).dclevel.flPath &= !DCPATH_ACTIVE;
    }

    PATH_UnlockPath(pPath);
    DC_UnlockDc(dc);
    ret
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn NtGdiFillPath(hDC: HDC) -> BOOL {
    let mut ret: BOOL = (FALSE as _);
    let pPath: PPATH;
    let pNewPath: PPATH;
    let pdcattr: PDC_ATTR;
    let dc: PDC;

    dc = DC_LockDc(hDC);
    if dc.is_null() {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return (FALSE as _);
    }

    pPath = PATH_LockPath((*dc).dclevel.hPath);
    if pPath.is_null() {
        DC_UnlockDc(dc);
        return (FALSE as _);
    }

    DC_vPrepareDCsForBlit(dc, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut());

    pdcattr = (*dc).pdcattr;

    if (*pdcattr).ulDirty_ & (DIRTY_LINE | DC_PEN_DIRTY) != 0 {
        DC_vUpdateLineBrush(dc);
    }

    if (*pdcattr).ulDirty_ & (DIRTY_FILL | DC_BRUSH_DIRTY) != 0 {
        DC_vUpdateFillBrush(dc);
    }

    pNewPath = PATH_FlattenPath(pPath);

    if (*pNewPath).state != PATH_Closed as _ {
        EngSetLastError(ERROR_CAN_NOT_COMPLETE);
    } else if (*pNewPath).numEntriesUsed != 0 {
        ret = PATH_FillPath(dc, pNewPath);
    } else {
        ret = (TRUE as _);
    }

    PATH_UnlockPath(pNewPath);
    PATH_Delete((*pNewPath).BaseObject.hHmgr);

    PATH_UnlockPath(pPath);
    PATH_Delete((*pPath).BaseObject.hHmgr);
    (*dc).dclevel.hPath = core::ptr::null_mut();
    (*dc).dclevel.flPath &= !DCPATH_ACTIVE;

    DC_vFinishBlit(dc, core::ptr::null_mut());
    DC_UnlockDc(dc);
    ret
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn NtGdiFillPath(hDC: HDC) -> BOOL {
    let mut ret: BOOL = FALSE as _;
    let pPath: PPATH;
    let pNewPath: PPATH;
    let pdcattr: PDC_ATTR;
    let dc: PDC;

    dc = DC_LockDc(hDC);
    if dc.is_null() {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return FALSE as _;
    }

    pPath = PATH_LockPath((*dc).dclevel.hPath);
    if pPath.is_null() {
        DC_UnlockDc(dc);
        return FALSE as _;
    }

    DC_vPrepareDCsForBlit(dc, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut());

    pdcattr = (*dc).pdcattr;

    if (*pdcattr).ulDirty_ & (DIRTY_LINE | DC_PEN_DIRTY) != 0 {
        DC_vUpdateLineBrush(dc);
    }

    if (*pdcattr).ulDirty_ & (DIRTY_FILL | DC_BRUSH_DIRTY) != 0 {
        DC_vUpdateFillBrush(dc);
    }

    pNewPath = PATH_FlattenPath(pPath);

    if (*pNewPath).state != PATH_Closed as _ {
        EngSetLastError(ERROR_CAN_NOT_COMPLETE);
    } else if (*pNewPath).numEntriesUsed != 0 {
        ret = PATH_FillPath(dc, pNewPath);
    } else {
        ret = TRUE as _;
    }

    PATH_UnlockPath(pNewPath);
    PATH_Delete((*pNewPath).BaseObject.hHmgr);

    PATH_UnlockPath(pPath);
    PATH_Delete((*pPath).BaseObject.hHmgr);
    (*dc).dclevel.hPath = core::ptr::null_mut();
    (*dc).dclevel.flPath &= !DCPATH_ACTIVE;

    DC_vFinishBlit(dc, core::ptr::null_mut());
    DC_UnlockDc(dc);
    ret
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn NtGdiFlattenPath(hDC: HDC) -> BOOL {
    let mut Ret: BOOL = (FALSE as _);
    let pDc: *mut DC;
    let pPath: PPATH;
    let mut pNewPath: PPATH = core::ptr::null_mut();

    pDc = DC_LockDc(hDC);
    if pDc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return (FALSE as _);
    }

    pPath = PATH_LockPath((*pDc).dclevel.hPath);
    if pPath.is_null() {
        EngSetLastError(ERROR_CAN_NOT_COMPLETE);
        DC_UnlockDc(pDc);
        return (FALSE as _);
    }

    if (*pPath).state == PATH_Closed {
        pNewPath = PATH_FlattenPath(pPath);
    }

    PATH_UnlockPath(pPath);

    if !pNewPath.is_null() {
        PATH_Delete((*pDc).dclevel.hPath);
        (*pDc).dclevel.hPath = (*pNewPath).BaseObject.hHmgr;
        PATH_UnlockPath(pNewPath);
        Ret = (TRUE as _);
    }

    DC_UnlockDc(pDc);
    Ret
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn NtGdiFlattenPath(hDC: HDC) -> BOOL {
    let mut Ret: BOOL = FALSE as _;
    let pDc: *mut DC;
    let pPath: PPATH;
    let mut pNewPath: PPATH = core::ptr::null_mut();

    pDc = DC_LockDc(hDC);
    if pDc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return FALSE as _;
    }

    pPath = PATH_LockPath((*pDc).dclevel.hPath);
    if pPath.is_null() {
        EngSetLastError(ERROR_CAN_NOT_COMPLETE);
        DC_UnlockDc(pDc);
        return FALSE as _;
    }

    if (*pPath).state == PATH_Closed {
        pNewPath = PATH_FlattenPath(pPath);
    }

    PATH_UnlockPath(pPath);

    if !pNewPath.is_null() {
        PATH_Delete((*pDc).dclevel.hPath);
        (*pDc).dclevel.hPath = (*pNewPath).BaseObject.hHmgr;
        PATH_UnlockPath(pNewPath);
        Ret = TRUE as _;
    }

    DC_UnlockDc(pDc);
    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiGetMiterLimit(hdc: HDC, pdwOut: PDWORD) -> i32 {

    let pDc: *mut DC;
    let mut bResult: i32 = 1;
    let Status: i32;

    pDc = DC_LockDc(hdc);
    if pDc.is_null() {
        EngSetLastError(ERROR_INVALID_PARAMETER as _);
        return 0;
    }

    Status = w32r_SafeProbeForWrite(
        pdwOut as *mut core::ffi::c_void,
        core::mem::size_of::<u32>() as u32,
        1,
    );
    if !NT_SUCCESS(Status) {
        SetLastNtError(Status);
        bResult = 0;
    } else {
        *pdwOut = (*pDc).dclevel.laPath.eMiterLimit as u32;
    }

    DC_UnlockDc(pDc);
    bResult
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn NtGdiGetPath(
    hDC: HDC,
    Points: LPPOINT,
    Types: LPBYTE,
    nSize: INT,
) -> INT {
    let mut ret: INT = -1;
    let pPath: PPATH;
    let dc: *mut DC;
    let mut Status: NTSTATUS;

    Status = w32r_SafeProbeForWrite(
        Points as *mut core::ffi::c_void,
        ((nSize as usize) * core::mem::size_of::<POINT>()) as _,
        core::mem::size_of::<ULONG>() as _,
    );
    if NT_SUCCESS(Status) {
        Status = w32r_SafeProbeForWrite(Types as *mut core::ffi::c_void, (nSize as usize as _), 1);
    }
    if !NT_SUCCESS(Status) {
        SetLastNtError(Status);
        return -1;
    }

    dc = DC_LockDc(hDC);
    if dc.is_null() {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return -1;
    }

    pPath = PATH_LockPath((*dc).dclevel.hPath);
    if pPath.is_null() {
        DC_UnlockDc(dc);
        return -1;
    }

    'done: {
        if (*pPath).state != (PATH_Closed as i32 as _) {
            EngSetLastError(ERROR_CAN_NOT_COMPLETE);
            break 'done;
        }

        if nSize == 0 {
            ret = (*pPath).numEntriesUsed as INT;
        } else if nSize < (*pPath).numEntriesUsed as INT {
            EngSetLastError(ERROR_INVALID_PARAMETER);
            break 'done;
        } else {
            loop {
                core::ptr::copy_nonoverlapping(
                    (*pPath).pPoints as *const u8,
                    Points as *mut u8,
                    core::mem::size_of::<POINT>() * (*pPath).numEntriesUsed as usize,
                );
                core::ptr::copy_nonoverlapping(
                    (*pPath).pFlags as *const u8,
                    Types as *mut u8,
                    core::mem::size_of::<BYTE>() * (*pPath).numEntriesUsed as usize,
                );

                if GdiPathDPtoLP(dc, Points, (*pPath).numEntriesUsed as INT) == 0 {
                    EngSetLastError(ERROR_ARITHMETIC_OVERFLOW);
                    break;
                }

                ret = (*pPath).numEntriesUsed as INT;
                break;
            }
        }
    }

    PATH_UnlockPath(pPath);
    DC_UnlockDc(dc);
    ret
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn NtGdiGetPath(
    hDC: HDC,
    Points: LPPOINT,
    Types: LPBYTE,
    nSize: INT,
) -> INT {
    let mut ret: INT = -1;
    let pPath: PPATH;
    let dc: *mut DC;
    let mut Status: NTSTATUS;

    Status = w32r_SafeProbeForWrite(
        Points as *mut core::ffi::c_void,
        ((nSize as usize) * core::mem::size_of::<POINT>()) as _,
        core::mem::size_of::<ULONG>() as _,
    );
    if NT_SUCCESS(Status) {
        Status = w32r_SafeProbeForWrite(Types as *mut core::ffi::c_void, nSize as usize as _, 1);
    }
    if !NT_SUCCESS(Status) {
        SetLastNtError(Status);
        return -1;
    }

    dc = DC_LockDc(hDC);
    if dc.is_null() {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return -1;
    }

    pPath = PATH_LockPath((*dc).dclevel.hPath);
    if pPath.is_null() {
        DC_UnlockDc(dc);
        return -1;
    }

    'done: {
        if (*pPath).state != (PATH_Closed as i32 as _) {
            EngSetLastError(ERROR_CAN_NOT_COMPLETE);
            break 'done;
        }

        if nSize == 0 {
            ret = (*pPath).numEntriesUsed as INT;
        } else if nSize < (*pPath).numEntriesUsed as INT {
            EngSetLastError(ERROR_INVALID_PARAMETER);
            break 'done;
        } else {
            loop {
                core::ptr::copy_nonoverlapping(
                    (*pPath).pPoints as *const u8,
                    Points as *mut u8,
                    core::mem::size_of::<POINT>() * (*pPath).numEntriesUsed as usize,
                );
                core::ptr::copy_nonoverlapping(
                    (*pPath).pFlags as *const u8,
                    Types as *mut u8,
                    core::mem::size_of::<BYTE>() * (*pPath).numEntriesUsed as usize,
                );

                if GdiPathDPtoLP(dc, Points, (*pPath).numEntriesUsed as INT) == 0 {
                    EngSetLastError(ERROR_ARITHMETIC_OVERFLOW);
                    break;
                }

                ret = (*pPath).numEntriesUsed as INT;
                break;
            }
        }
    }

    PATH_UnlockPath(pPath);
    DC_UnlockDc(dc);
    ret
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiPathToRegion(hDC: HDC) -> HRGN {
    let pPath: PPATH;
    let pNewPath: PPATH;
    let mut hrgnRval: HRGN = core::ptr::null_mut();
    let Ret: INT;
    let Rgn: PREGION;
    let pDc: *mut DC;
    let pdcattr: PDC_ATTR;

    pDc = DC_LockDc(hDC);
    if pDc.is_null() {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return core::ptr::null_mut();
    }

    pdcattr = (*pDc).pdcattr;

    pPath = PATH_LockPath((*pDc).dclevel.hPath);
    if pPath.is_null() {
        DC_UnlockDc(pDc);
        return core::ptr::null_mut();
    }

    if (*pPath).state != PATH_Closed {
        EngSetLastError(ERROR_CAN_NOT_COMPLETE);
    } else {
        Rgn = REGION_AllocUserRgnWithHandle(1);
        if Rgn.is_null() {
            PATH_UnlockPath(pPath);
            DC_UnlockDc(pDc);
            return core::ptr::null_mut();
        }
        hrgnRval = (*Rgn).BaseObject.hHmgr as HRGN;

        pNewPath = PATH_FlattenPath(pPath);
        if pNewPath.is_null() {
            REGION_Delete(Rgn);
            PATH_UnlockPath(pPath);
            DC_UnlockDc(pDc);
            return core::ptr::null_mut();
        }

        Ret = PATH_PathToRegion(pNewPath, (*pdcattr).jFillMode as INT, Rgn);

        PATH_UnlockPath(pNewPath);
        PATH_Delete((*pNewPath).BaseObject.hHmgr as HPATH);

        if Ret == 0 {
            REGION_Delete(Rgn);
            hrgnRval = core::ptr::null_mut();
        } else {
            REGION_UnlockRgn(Rgn);
        }
    }

    PATH_UnlockPath(pPath);
    PATH_Delete((*pDc).dclevel.hPath as HPATH);
    (*pDc).dclevel.hPath = core::ptr::null_mut();
    (*pDc).dclevel.flPath &= !DCPATH_ACTIVE;

    DC_UnlockDc(pDc);
    hrgnRval
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn NtGdiSetMiterLimit(
    hdc: HDC,
    dwNew: u32,
    pdwOut: PDWORD,
) -> i32 {
    let pDc: *mut DC;
    let mut worker: gxf_long = core::mem::zeroed();
    let mut worker1: gxf_long = core::mem::zeroed();
    let mut bResult: i32 = 1;
    let Status: i32;

    pDc = DC_LockDc(hdc);
    if pDc.is_null() {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    worker.l = dwNew;
    worker1.f = ((*pDc).dclevel.laPath.eMiterLimit as _);
    (*pDc).dclevel.laPath.eMiterLimit = (worker.f as _);

    if !pdwOut.is_null() {
        Status = w32r_SafeProbeForWrite(
            pdwOut as *mut core::ffi::c_void,
            core::mem::size_of::<u32>() as u32,
            1,
        );
        if !NT_SUCCESS(Status) {
            SetLastNtError(Status);
            bResult = 0;
        } else {
            *pdwOut = worker1.l;
        }
    }

    DC_UnlockDc(pDc);
    bResult
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn NtGdiSetMiterLimit(
    hdc: HDC,
    dwNew: u32,
    pdwOut: PDWORD,
) -> i32 {
    let pDc: *mut DC;
    let mut worker: gxf_long = core::mem::zeroed();
    let mut worker1: gxf_long = core::mem::zeroed();
    let mut bResult: i32 = 1;
    let Status: i32;

    pDc = DC_LockDc(hdc);
    if pDc.is_null() {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    worker.l = dwNew;
    worker1.f = (*pDc).dclevel.laPath.eMiterLimit;
    (*pDc).dclevel.laPath.eMiterLimit = worker.f;

    if !pdwOut.is_null() {
        Status = w32r_SafeProbeForWrite(
            pdwOut as *mut core::ffi::c_void,
            core::mem::size_of::<u32>() as u32,
            1,
        );
        if !NT_SUCCESS(Status) {
            SetLastNtError(Status);
            bResult = 0;
        } else {
            *pdwOut = worker1.l;
        }
    }

    DC_UnlockDc(pDc);
    bResult
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn NtGdiStrokeAndFillPath(hDC: HDC) -> BOOL {
    let pDc: *mut DC;
    let pdcattr: PDC_ATTR;
    let pPath: PPATH;
    let pNewPath: PPATH;
    let mut bRet: BOOL = (FALSE as _);

    pDc = DC_LockDc(hDC);
    if pDc.is_null() {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return (FALSE as _);
    }
    pPath = PATH_LockPath((*pDc).dclevel.hPath);
    if pPath.is_null() {
        DC_UnlockDc(pDc);
        return (FALSE as _);
    }

    DC_vPrepareDCsForBlit(pDc, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut());

    pdcattr = (*pDc).pdcattr;

    if (*pdcattr).ulDirty_ & (DIRTY_FILL | DC_BRUSH_DIRTY) != 0 {
        DC_vUpdateFillBrush(pDc);
    }

    if (*pdcattr).ulDirty_ & (DIRTY_LINE | DC_PEN_DIRTY) != 0 {
        DC_vUpdateLineBrush(pDc);
    }

    pNewPath = PATH_FlattenPath(pPath);

    if (*pNewPath).state != PATH_Closed as _ {
        EngSetLastError(ERROR_CAN_NOT_COMPLETE);
    } else if (*pNewPath).numEntriesUsed != 0 {
        bRet = PATH_FillPath(pDc, pNewPath);
        if bRet != (FALSE as _) {
            bRet = PATH_StrokePath(pDc, pNewPath);
        }
    } else {
        bRet = (TRUE as _);
    }

    PATH_UnlockPath(pNewPath);
    PATH_Delete((*pNewPath).BaseObject.hHmgr);

    PATH_UnlockPath(pPath);
    PATH_Delete((*pPath).BaseObject.hHmgr);
    (*pDc).dclevel.hPath = core::ptr::null_mut();
    (*pDc).dclevel.flPath &= !DCPATH_ACTIVE;

    DC_vFinishBlit(pDc, core::ptr::null_mut());
    DC_UnlockDc(pDc);
    bRet
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn NtGdiStrokeAndFillPath(hDC: HDC) -> BOOL {
    let pDc: *mut DC;
    let pdcattr: PDC_ATTR;
    let pPath: PPATH;
    let pNewPath: PPATH;
    let mut bRet: BOOL = FALSE as _;

    pDc = DC_LockDc(hDC);
    if pDc.is_null() {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return FALSE as _;
    }
    pPath = PATH_LockPath((*pDc).dclevel.hPath);
    if pPath.is_null() {
        DC_UnlockDc(pDc);
        return FALSE as _;
    }

    DC_vPrepareDCsForBlit(pDc, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut());

    pdcattr = (*pDc).pdcattr;

    if (*pdcattr).ulDirty_ & (DIRTY_FILL | DC_BRUSH_DIRTY) != 0 {
        DC_vUpdateFillBrush(pDc);
    }

    if (*pdcattr).ulDirty_ & (DIRTY_LINE | DC_PEN_DIRTY) != 0 {
        DC_vUpdateLineBrush(pDc);
    }

    pNewPath = PATH_FlattenPath(pPath);

    if (*pNewPath).state != PATH_Closed as _ {
        EngSetLastError(ERROR_CAN_NOT_COMPLETE);
    } else if (*pNewPath).numEntriesUsed != 0 {
        bRet = PATH_FillPath(pDc, pNewPath);
        if bRet != (FALSE as _) {
            bRet = PATH_StrokePath(pDc, pNewPath);
        }
    } else {
        bRet = TRUE as _;
    }

    PATH_UnlockPath(pNewPath);
    PATH_Delete((*pNewPath).BaseObject.hHmgr);

    PATH_UnlockPath(pPath);
    PATH_Delete((*pPath).BaseObject.hHmgr);
    (*pDc).dclevel.hPath = core::ptr::null_mut();
    (*pDc).dclevel.flPath &= !DCPATH_ACTIVE;

    DC_vFinishBlit(pDc, core::ptr::null_mut());
    DC_UnlockDc(pDc);
    bRet
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn NtGdiStrokePath(hDC: HDC) -> BOOL {
    let pDc: *mut DC;
    let pdcattr: PDC_ATTR;
    let pPath: PPATH;
    let pNewPath: PPATH;
    let mut bRet: BOOL = (FALSE as _);

    pDc = DC_LockDc(hDC);
    if pDc.is_null() {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return (FALSE as _);
    }

    pPath = PATH_LockPath((*pDc).dclevel.hPath);
    if pPath.is_null() {
        DC_UnlockDc(pDc);
        return (FALSE as _);
    }

    DC_vPrepareDCsForBlit(pDc, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut());

    pdcattr = (*pDc).pdcattr;

    if (*pdcattr).ulDirty_ & (DIRTY_LINE | DC_PEN_DIRTY) != 0 {
        DC_vUpdateLineBrush(pDc);
    }

    pNewPath = PATH_FlattenPath(pPath);

    if (*pNewPath).state != PATH_Closed {
        EngSetLastError(ERROR_CAN_NOT_COMPLETE);
    } else {
        bRet = PATH_StrokePath(pDc, pNewPath);
    }

    PATH_UnlockPath(pNewPath);
    PATH_Delete((*pNewPath).BaseObject.hHmgr);

    DC_vFinishBlit(pDc, core::ptr::null_mut());

    PATH_UnlockPath(pPath);
    PATH_Delete((*pPath).BaseObject.hHmgr);
    (*pDc).dclevel.hPath = core::ptr::null_mut();
    (*pDc).dclevel.flPath &= !DCPATH_ACTIVE;

    DC_UnlockDc(pDc);
    bRet
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn NtGdiStrokePath(hDC: HDC) -> BOOL {
    let pDc: *mut DC;
    let pdcattr: PDC_ATTR;
    let pPath: PPATH;
    let pNewPath: PPATH;
    let mut bRet: BOOL = FALSE as _;

    pDc = DC_LockDc(hDC);
    if pDc.is_null() {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return FALSE as _;
    }

    pPath = PATH_LockPath((*pDc).dclevel.hPath);
    if pPath.is_null() {
        DC_UnlockDc(pDc);
        return FALSE as _;
    }

    DC_vPrepareDCsForBlit(pDc, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut());

    pdcattr = (*pDc).pdcattr;

    if (*pdcattr).ulDirty_ & (DIRTY_LINE | DC_PEN_DIRTY) != 0 {
        DC_vUpdateLineBrush(pDc);
    }

    pNewPath = PATH_FlattenPath(pPath);

    if (*pNewPath).state != PATH_Closed {
        EngSetLastError(ERROR_CAN_NOT_COMPLETE);
    } else {
        bRet = PATH_StrokePath(pDc, pNewPath);
    }

    PATH_UnlockPath(pNewPath);
    PATH_Delete((*pNewPath).BaseObject.hHmgr);

    DC_vFinishBlit(pDc, core::ptr::null_mut());

    PATH_UnlockPath(pPath);
    PATH_Delete((*pPath).BaseObject.hHmgr);
    (*pDc).dclevel.hPath = core::ptr::null_mut();
    (*pDc).dclevel.flPath &= !DCPATH_ACTIVE;

    DC_UnlockDc(pDc);
    bRet
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn NtGdiWidenPath(hDC: HDC) -> BOOL {
    let pPath: PPATH;
    let mut Ret: BOOL = (FALSE as _);
    let pdc: PDC = DC_LockDc(hDC);
    if pdc.is_null() {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return (FALSE as _);
    }

    pPath = PATH_WidenPath(pdc);
    if !pPath.is_null() {
        PATH_Delete((*pdc).dclevel.hPath);
        (*pdc).dclevel.hPath = (*pPath).BaseObject.hHmgr;
        Ret = (TRUE as _);
    }
    DC_UnlockDc(pdc);
    Ret
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn NtGdiWidenPath(hDC: HDC) -> BOOL {
    let pPath: PPATH;
    let mut Ret: BOOL = FALSE as _;
    let pdc: PDC = DC_LockDc(hDC);
    if pdc.is_null() {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return FALSE as _;
    }

    pPath = PATH_WidenPath(pdc);
    if !pPath.is_null() {
        PATH_Delete((*pdc).dclevel.hPath);
        (*pdc).dclevel.hPath = (*pPath).BaseObject.hHmgr;
        Ret = TRUE as _;
    }
    DC_UnlockDc(pdc);
    Ret
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS win32 kernel mode subsystem
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            win32ss/gdi/ntgdi/path.c
//  * PURPOSE:         Graphics paths (BeginPath, EndPath etc.)
//  * PROGRAMMER:      Copyright 1997, 1998 Martin Boehme
//  *                            1999 Huw D M Davies
//  *                            2005 Dmitry Timoshkov
//  *                            2018 Katayama Hirofumi MZ
//  */
//
// #include <win32k.h>
// #include <safe_user_io.h>
// #include <suppress.h>
//
// DBG_DEFAULT_CHANNEL(GdiPath);
//
// #ifdef _MSC_VER
// #pragma warning(disable:4244)
// #endif
//
// #define NUM_ENTRIES_INITIAL 16  /* Initial size of points / flags arrays  */
//
// #define GROW_FACTOR_NUMER    2  /* Numerator of grow factor for the array */
// #define GROW_FACTOR_DENOM    1  /* Denominator of grow factor             */
//
// #if DBG
// static int PathCount = 0;
// #endif
//
// /***********************************************************************
//  * Internal functions
//  */
//
// PPATH FASTCALL
// PATH_CreatePath(int count)
// {
//     PPATH pPath = PATH_AllocPathWithHandle();
//
//     if (!pPath)
//     {
//         EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return NULL;
//     }
//
//     DbgPrint("CreatePath p 0x%p\n", pPath);
//     // Path handles are shared. Also due to recursion with in the same thread.
//     GDIOBJ_vUnlockObject((POBJ)pPath);       // Unlock
//     pPath = PATH_LockPath(pPath->BaseObject.hHmgr); // Share Lock.
//
//     /* Make sure that path is empty */
//     PATH_EmptyPath(pPath);
//
//     count = max( NUM_ENTRIES_INITIAL, count );
//
//     pPath->numEntriesAllocated = count;
//
//     pPath->pPoints = (POINT *)ExAllocatePoolWithTag(PagedPool, count * sizeof(POINT), TAG_PATH);
//     RtlZeroMemory( pPath->pPoints, count * sizeof(POINT));
//     pPath->pFlags  =  (BYTE *)ExAllocatePoolWithTag(PagedPool, count * sizeof(BYTE),  TAG_PATH);
//     RtlZeroMemory( pPath->pFlags, count * sizeof(BYTE));
//
//     /* Initialize variables for new path */
//     pPath->numEntriesUsed = 0;
//     pPath->newStroke = TRUE;
//     pPath->state     = PATH_Open;
//     pPath->pos.x = pPath->pos.y = 0;
// #if DBG
//     PathCount++;
//     DbgPrint("Create Path %d\n",PathCount);
// #endif
//     return pPath;
// }
//
// /* PATH_DestroyGdiPath
//  *
//  * Destroys a GdiPath structure (frees the memory in the arrays).
//  */
// VOID
// FASTCALL
// PATH_DestroyGdiPath(PPATH pPath)
// {
//     ASSERT(pPath != NULL);
//
//     if (pPath->pPoints) ExFreePoolWithTag(pPath->pPoints, TAG_PATH);
//     if (pPath->pFlags) ExFreePoolWithTag(pPath->pFlags, TAG_PATH);
// }
//
// BOOL
// FASTCALL
// PATH_Delete(HPATH hPath)
// {
//     PPATH pPath;
//     if (!hPath) return FALSE;
//     pPath = PATH_LockPath(hPath);
//     if (!pPath) return FALSE;
//     PATH_DestroyGdiPath(pPath);
//     GDIOBJ_vDeleteObject(&pPath->BaseObject);
// #if DBG
//     PathCount--;
//     DbgPrint("Delete Path %d\n",PathCount);
// #endif
//     return TRUE;
// }
//
//
// VOID
// FASTCALL
// IntGdiCloseFigure(PPATH pPath)
// {
//     ASSERT(pPath->state == PATH_Open);
//
//     // FIXME: Shouldn't we draw a line to the beginning of the figure?
//     // Set PT_CLOSEFIGURE on the last entry and start a new stroke
//     if (pPath->numEntriesUsed)
//     {
//         pPath->pFlags[pPath->numEntriesUsed - 1] |= PT_CLOSEFIGURE;
//         pPath->newStroke = TRUE;
//     }
// }
//
// /* MSDN: This fails if the device coordinates exceed 27 bits, or if the converted
//          logical coordinates exceed 32 bits. */
// BOOL
// FASTCALL
// GdiPathDPtoLP(
//     PDC pdc,
//     PPOINT ppt,
//     INT count)
// {
//     XFORMOBJ xo;
//
//     w32r_XFORMOBJ_vInit(&xo, &pdc->pdcattr->mxDeviceToWorld);
//     return w32r_XFORMOBJ_bApplyXform(&xo, XF_LTOL, count, (PPOINTL)ppt, (PPOINTL)ppt);
// }
//
// /* PATH_InitGdiPath
//  *
//  * Initializes the GdiPath structure.
//  */
// VOID
// FASTCALL
// PATH_InitGdiPath(
//     PPATH pPath)
// {
//     ASSERT(pPath != NULL);
//
//     pPath->state = PATH_Null;
//     pPath->pPoints = NULL;
//     pPath->pFlags = NULL;
//     pPath->numEntriesUsed = 0;
//     pPath->numEntriesAllocated = 0;
// }
//
// /* PATH_AssignGdiPath
//  *
//  * Copies the GdiPath structure "pPathSrc" to "pPathDest". A deep copy is
//  * performed, i.e. the contents of the pPoints and pFlags arrays are copied,
//  * not just the pointers. Since this means that the arrays in pPathDest may
//  * need to be resized, pPathDest should have been initialized using
//  * PATH_InitGdiPath (in C++, this function would be an assignment operator,
//  * not a copy constructor).
//  * Returns TRUE if successful, else FALSE.
//  */
// BOOL
// FASTCALL
// PATH_AssignGdiPath(
//     PPATH pPathDest,
//     const PPATH pPathSrc)
// {
//     ASSERT(pPathDest != NULL && pPathSrc != NULL);
//
//     /* Make sure destination arrays are big enough */
//     if (!PATH_ReserveEntries(pPathDest, pPathSrc->numEntriesUsed))
//         return FALSE;
//
//     /* Perform the copy operation */
//     memcpy(pPathDest->pPoints, pPathSrc->pPoints, sizeof(POINT)*pPathSrc->numEntriesUsed);
//     memcpy(pPathDest->pFlags,  pPathSrc->pFlags,  sizeof(BYTE)*pPathSrc->numEntriesUsed);
//
//     pPathDest->pos = pPathSrc->pos;
//     pPathDest->state = pPathSrc->state;
//     pPathDest->numEntriesUsed = pPathSrc->numEntriesUsed;
//     pPathDest->newStroke = pPathSrc->newStroke;
//     return TRUE;
// }
//
// BOOL PATH_SavePath( DC *dst, DC *src )
// {
//     PPATH pdstPath, psrcPath = PATH_LockPath(src->dclevel.hPath);
//     DbgPrint("PATH_SavePath\n");
//     if (psrcPath)
//     {
//        DbgPrint("PATH_SavePath 1\n");
//
//        pdstPath = PATH_CreatePath(psrcPath->numEntriesAllocated);
//
//        dst->dclevel.flPath = src->dclevel.flPath;
//
//        dst->dclevel.hPath = pdstPath->BaseObject.hHmgr;
//
//        PATH_AssignGdiPath(pdstPath, psrcPath);
//
//        PATH_UnlockPath(pdstPath);
//        PATH_UnlockPath(psrcPath);
//     }
//     return TRUE;
// }
//
// BOOL PATH_RestorePath( DC *dst, DC *src )
// {
//     DbgPrint("PATH_RestorePath\n");
//
//     if (dst->dclevel.hPath == NULL)
//     {
//        PPATH pdstPath, psrcPath = PATH_LockPath(src->dclevel.hPath);
//        DbgPrint("PATH_RestorePath 1\n");
//        pdstPath = PATH_CreatePath(psrcPath->numEntriesAllocated);
//        dst->dclevel.flPath = src->dclevel.flPath;
//        dst->dclevel.hPath = pdstPath->BaseObject.hHmgr;
//
//        PATH_AssignGdiPath(pdstPath, psrcPath);
//
//        PATH_UnlockPath(pdstPath);
//        PATH_UnlockPath(psrcPath);
//     }
//     else
//     {
//        PPATH pdstPath, psrcPath = PATH_LockPath(src->dclevel.hPath);
//        pdstPath = PATH_LockPath(dst->dclevel.hPath);
//        DbgPrint("PATH_RestorePath 2\n");
//        dst->dclevel.flPath = src->dclevel.flPath & (DCPATH_CLOCKWISE|DCPATH_ACTIVE);
//        PATH_AssignGdiPath(pdstPath, psrcPath);
//
//        PATH_UnlockPath(pdstPath);
//        PATH_UnlockPath(psrcPath);
//     }
//     return TRUE;
// }
//
// /* PATH_EmptyPath
//  *
//  * Removes all entries from the path and sets the path state to PATH_Null.
//  */
// VOID
// FASTCALL
// PATH_EmptyPath(PPATH pPath)
// {
//     ASSERT(pPath != NULL);
//
//     pPath->state = PATH_Null;
//     pPath->numEntriesUsed = 0;
// }
//
// /* PATH_AddEntry
//  *
//  * Adds an entry to the path. For "flags", pass either PT_MOVETO, PT_LINETO
//  * or PT_BEZIERTO, optionally ORed with PT_CLOSEFIGURE. Returns TRUE if
//  * successful, FALSE otherwise (e.g. if not enough memory was available).
//  */
// BOOL
// FASTCALL
// PATH_AddEntry(
//     PPATH pPath,
//     const POINT *pPoint,
//     BYTE flags)
// {
//     ASSERT(pPath != NULL);
//
//     /* FIXME: If newStroke is true, perhaps we want to check that we're
//      * getting a PT_MOVETO
//      */
//     DbgPrint("(%d,%d) - %d\n", pPoint->x, pPoint->y, flags);
//
//     /* Reserve enough memory for an extra path entry */
//     if (!PATH_ReserveEntries(pPath, pPath->numEntriesUsed + 1))
//         return FALSE;
//
//     /* Store information in path entry */
//     pPath->pPoints[pPath->numEntriesUsed] = *pPoint;
//     pPath->pFlags[pPath->numEntriesUsed] = flags;
//
//     /* Increment entry count */
//     pPath->numEntriesUsed++;
//
//     return TRUE;
// }
//
// /* PATH_ReserveEntries
//  *
//  * Ensures that at least "numEntries" entries (for points and flags) have
//  * been allocated; allocates larger arrays and copies the existing entries
//  * to those arrays, if necessary. Returns TRUE if successful, else FALSE.
//  */
// BOOL
// FASTCALL
// PATH_ReserveEntries(
//     PPATH pPath,
//     INT numEntries)
// {
//     INT numEntriesToAllocate;
//     POINT *pPointsNew;
//     BYTE *pFlagsNew;
//
//     ASSERT(pPath != NULL);
//     ASSERT(numEntries >= 0);
//
//     /* Do we have to allocate more memory? */
//     if (numEntries > pPath->numEntriesAllocated)
//     {
//         /* Find number of entries to allocate. We let the size of the array
//          * grow exponentially, since that will guarantee linear time
//          * complexity. */
//         if (pPath->numEntriesAllocated)
//         {
//             numEntriesToAllocate = pPath->numEntriesAllocated;
//             while (numEntriesToAllocate < numEntries)
//                 numEntriesToAllocate = numEntriesToAllocate * GROW_FACTOR_NUMER / GROW_FACTOR_DENOM;
//         }
//         else
//             numEntriesToAllocate = numEntries;
//
//         /* Allocate new arrays */
//         pPointsNew = (POINT *)ExAllocatePoolWithTag(PagedPool, numEntriesToAllocate * sizeof(POINT), TAG_PATH);
//         if (!pPointsNew)
//             return FALSE;
//
//         pFlagsNew = (BYTE *)ExAllocatePoolWithTag(PagedPool, numEntriesToAllocate * sizeof(BYTE), TAG_PATH);
//         if (!pFlagsNew)
//         {
//             ExFreePoolWithTag(pPointsNew, TAG_PATH);
//             return FALSE;
//         }
//
//         /* Copy old arrays to new arrays and discard old arrays */
//         if (pPath->pPoints)
//         {
//             ASSERT(pPath->pFlags);
//
//             memcpy(pPointsNew, pPath->pPoints, sizeof(POINT)*pPath->numEntriesUsed);
//             memcpy(pFlagsNew, pPath->pFlags, sizeof(BYTE)*pPath->numEntriesUsed);
//
//             ExFreePoolWithTag(pPath->pPoints, TAG_PATH);
//             ExFreePoolWithTag(pPath->pFlags, TAG_PATH);
//         }
//
//         pPath->pPoints = pPointsNew;
//         pPath->pFlags = pFlagsNew;
//         pPath->numEntriesAllocated = numEntriesToAllocate;
//     }
//
//     return TRUE;
// }
//
// /* PATH_ScaleNormalizedPoint
//  *
//  * Scales a normalized point (x, y) with respect to the box whose corners are
//  * passed in "corners". The point is stored in "*pPoint". The normalized
//  * coordinates (-1.0, -1.0) correspond to corners[0], the coordinates
//  * (1.0, 1.0) correspond to corners[1].
//  */
// static
// BOOLEAN
// PATH_ScaleNormalizedPoint(
//     POINT corners[],
//     FLOATL x,
//     FLOATL y,
//     POINT *pPoint)
// {
//     FLOATOBJ tmp;
//
//     ASSERT(corners);
//     ASSERT(pPoint);
//
//     /* pPoint->x = (double)corners[0].x + (double)(corners[1].x - corners[0].x) * 0.5 * (x + 1.0); */
//     w32r_FLOATOBJ_SetFloat(&tmp, x);
//     w32r_FLOATOBJ_Add(&tmp, (FLOATOBJ*)&gef1);
//     w32r_FLOATOBJ_Div(&tmp, (FLOATOBJ*)&gef2);
//     w32r_FLOATOBJ_MulLong(&tmp, corners[1].x - corners[0].x);
//     w32r_FLOATOBJ_AddLong(&tmp, corners[0].x);
//     if (!w32r_FLOATOBJ_bConvertToLong(&tmp, &pPoint->x))
//         return FALSE;
//
//     /* pPoint->y = (double)corners[0].y + (double)(corners[1].y - corners[0].y) * 0.5 * (y + 1.0); */
//     w32r_FLOATOBJ_SetFloat(&tmp, y);
//     w32r_FLOATOBJ_Add(&tmp, (FLOATOBJ*)&gef1);
//     w32r_FLOATOBJ_Div(&tmp, (FLOATOBJ*)&gef2);
//     w32r_FLOATOBJ_MulLong(&tmp, corners[1].y - corners[0].y);
//     w32r_FLOATOBJ_AddLong(&tmp, corners[0].y);
//     if (!w32r_FLOATOBJ_bConvertToLong(&tmp, &pPoint->y))
//         return FALSE;
//     return TRUE;
// }
//
// /* PATH_NormalizePoint
//  *
//  * Normalizes a point with respect to the box whose corners are passed in
//  * corners. The normalized coordinates are stored in *pX and *pY.
//  */
// static
// VOID
// PATH_NormalizePoint(
//     POINTL corners[],
//     const POINTL *pPoint,
//     FLOATL *pX,
//     FLOATL *pY)
// {
//     FLOATOBJ tmp;
//
//     ASSERT(corners);
//     ASSERT(pPoint);
//     ASSERT(pX);
//     ASSERT(pY);
//
//     /* *pX = (float)(pPoint->x - corners[0].x) / (float)(corners[1].x - corners[0].x) * 2.0 - 1.0; */
//     w32r_FLOATOBJ_SetLong(&tmp, (pPoint->x - corners[0].x) * 2);
//     w32r_FLOATOBJ_DivLong(&tmp, corners[1].x - corners[0].x);
//     w32r_FLOATOBJ_Sub(&tmp, (PFLOATOBJ)&gef1);
//     *pX = w32r_FLOATOBJ_GetFloat(&tmp);
//
//     /* *pY = (float)(pPoint->y - corners[0].y) / (float)(corners[1].y - corners[0].y) * 2.0 - 1.0; */
//     w32r_FLOATOBJ_SetLong(&tmp, (pPoint->y - corners[0].y) * 2);
//     w32r_FLOATOBJ_DivLong(&tmp, corners[1].y - corners[0].y);
//     w32r_FLOATOBJ_Sub(&tmp, (PFLOATOBJ)&gef1);
//     *pY = w32r_FLOATOBJ_GetFloat(&tmp);
// }
//
// /* PATH_CheckCorners
//  *
//  * Helper function for PATH_RoundRect() and PATH_Rectangle()
//  */
// static
// BOOL
// PATH_CheckRect(
//     DC *dc,
//     RECTL* rect,
//     INT x1,
//     INT y1,
//     INT x2,
//     INT y2)
// {
//     PDC_ATTR pdcattr = dc->pdcattr;
//
//     /* Convert points to device coordinates */
//     w32r_RECTL_vSetRect(rect, x1, y1, x2, y2);
//     w32r_IntLPtoDP(dc, (PPOINT)rect, 2);
//
//     /* Make sure first corner is top left and second corner is bottom right */
//     RECTL_vMakeWellOrdered(rect);
//
//     /* In GM_COMPATIBLE, don't include bottom and right edges */
//     if (pdcattr->iGraphicsMode == GM_COMPATIBLE)
//     {
//         if (rect->left == rect->right) return FALSE;
//         if (rect->top == rect->bottom) return FALSE;
//         rect->right--;
//         rect->bottom--;
//     }
//     return TRUE;
// }
//
// /* add a number of points, converting them to device coords */
// /* return a pointer to the first type byte so it can be fixed up if necessary */
// static BYTE *add_log_points( DC *dc, PPATH path, const POINT *points,
//                              DWORD count, BYTE type )
// {
//     BYTE *ret;
//
//     if (!PATH_ReserveEntries( path, path->numEntriesUsed + count )) return NULL;
//
//     ret = &path->pFlags[path->numEntriesUsed];
//     memcpy( &path->pPoints[path->numEntriesUsed], points, count * sizeof(*points) );
//     w32r_IntLPtoDP( dc, &path->pPoints[path->numEntriesUsed], count );
//     memset( ret, type, count );
//     path->numEntriesUsed += count;
//     return ret;
// }
//
// /* add a number of points that are already in device coords */
// /* return a pointer to the first type byte so it can be fixed up if necessary */
// static BYTE *add_points( PPATH path, const POINT *points, DWORD count, BYTE type )
// {
//     BYTE *ret;
//
//     if (!PATH_ReserveEntries( path, path->numEntriesUsed + count )) return NULL;
//
//     ret = &path->pFlags[path->numEntriesUsed];
//     memcpy( &path->pPoints[path->numEntriesUsed], points, count * sizeof(*points) );
//     memset( ret, type, count );
//     path->numEntriesUsed += count;
//     return ret;
// }
//
// /* reverse the order of an array of points */
// static void reverse_points( POINT *points, UINT count )
// {
//     UINT i;
//     for (i = 0; i < count / 2; i++)
//     {
//         POINT pt = points[i];
//         points[i] = points[count - i - 1];
//         points[count - i - 1] = pt;
//     }
// }
//
// /* start a new path stroke if necessary */
// static BOOL start_new_stroke( PPATH path )
// {
//     if (!path->newStroke && path->numEntriesUsed &&
//         !(path->pFlags[path->numEntriesUsed - 1] & PT_CLOSEFIGURE) &&
//         path->pPoints[path->numEntriesUsed - 1].x == path->pos.x &&
//         path->pPoints[path->numEntriesUsed - 1].y == path->pos.y)
//         return TRUE;
//
//     path->newStroke = FALSE;
//     return add_points( path, &path->pos, 1, PT_MOVETO ) != NULL;
// }
//
// /* set current position to the last point that was added to the path */
// static void update_current_pos( PPATH path )
// {
//     ASSERT(path->numEntriesUsed);
//     path->pos = path->pPoints[path->numEntriesUsed - 1];
// }
//
// /* close the current figure */
// static void close_figure( PPATH path )
// {
//     ASSERT(path->numEntriesUsed);
//     path->pFlags[path->numEntriesUsed - 1] |= PT_CLOSEFIGURE;
// }
//
// /* add a number of points, starting a new stroke if necessary */
// static BOOL add_log_points_new_stroke( DC *dc, PPATH path, const POINT *points,
//                                        DWORD count, BYTE type )
// {
//     if (!start_new_stroke( path )) return FALSE;
//     if (!add_log_points( dc, path, points, count, type )) return FALSE;
//     update_current_pos( path );
//
//     DbgPrint("ALPNS : Pos X %d Y %d\n",path->pos.x, path->pos.y);
//     IntGdiMoveToEx(dc, path->pos.x, path->pos.y, NULL);
//
//     return TRUE;
// }
//
// /* PATH_MoveTo
//  *
//  * Should be called when a MoveTo is performed on a DC that has an
//  * open path. This starts a new stroke. Returns TRUE if successful, else
//  * FALSE.
//  */
// BOOL
// FASTCALL
// PATH_MoveTo(
//     PDC dc,
//     PPATH pPath)
// {
//     if (!pPath) return FALSE;
//
//     // GDI32 : Signal from user space of a change in position.
//     if (dc->pdcattr->ulDirty_ & DIRTY_STYLESTATE)
//     {
//        DbgPrint("MoveTo has changed\n");
//        pPath->newStroke = TRUE;
//        // Set position and clear the signal flag.
//        IntGetCurrentPositionEx(dc, &pPath->pos);
//        w32r_IntLPtoDP( dc, &pPath->pos, 1 );
//        return TRUE;
//     }
//
//     return FALSE;
// }
//
// /* PATH_LineTo
//  *
//  * Should be called when a LineTo is performed on a DC that has an
//  * open path. This adds a PT_LINETO entry to the path (and possibly
//  * a PT_MOVETO entry, if this is the first LineTo in a stroke).
//  * Returns TRUE if successful, else FALSE.
//  */
// BOOL
// FASTCALL
// PATH_LineTo(
//     PDC dc,
//     INT x,
//     INT y)
// {
//     BOOL Ret;
//     PPATH pPath;
//     POINT point, pointCurPos;
//
//     pPath = PATH_LockPath(dc->dclevel.hPath);
//     if (!pPath) return FALSE;
//
//     point.x = x;
//     point.y = y;
//
//     // Coalesce a MoveTo point.
//     if ( !PATH_MoveTo(dc, pPath) )
//     {
//        /* Add a PT_MOVETO if necessary */
//        if (pPath->newStroke)
//        {
//            DbgPrint("Line To : New Stroke\n");
//            pPath->newStroke = FALSE;
//            IntGetCurrentPositionEx(dc, &pointCurPos);
//            w32r_CoordLPtoDP(dc, &pointCurPos);
//            if (!PATH_AddEntry(pPath, &pointCurPos, PT_MOVETO))
//            {
//                PATH_UnlockPath(pPath);
//                return FALSE;
//            }
//        }
//     }
//     Ret = add_log_points_new_stroke( dc, pPath, &point, 1, PT_LINETO );
//     PATH_UnlockPath(pPath);
//     return Ret;
// }
//
// /* PATH_Rectangle
//  *
//  * Should be called when a call to Rectangle is performed on a DC that has
//  * an open path. Returns TRUE if successful, else FALSE.
//  */
// BOOL
// FASTCALL
// PATH_Rectangle(
//     PDC dc,
//     INT x1,
//     INT y1,
//     INT x2,
//     INT y2)
// {
//     PPATH pPath;
//     RECTL rect;
//     POINTL points[4];
//     BYTE *type;
//
//     pPath = PATH_LockPath(dc->dclevel.hPath);
//     if (!pPath) return FALSE;
//
//     if (!PATH_CheckRect(dc, &rect, x1, y1, x2, y2))
//     {
//         PATH_UnlockPath(pPath);
//         return TRUE;
//     }
//
//     points[0].x = rect.right; points[0].y = rect.top;
//     points[1].x = rect.left; points[1].y = rect.top;
//     points[2].x = rect.left; points[2].y = rect.bottom;
//     points[3].x = rect.right; points[3].y = rect.bottom;
//
//     if (dc->dclevel.flPath & DCPATH_CLOCKWISE) reverse_points(points, 4 );
//
//     if (!(type = add_points( pPath, points, 4, PT_LINETO )))
//     {
//        PATH_UnlockPath(pPath);
//        return FALSE;
//     }
//     type[0] = PT_MOVETO;
//
//     /* Close the rectangle figure */
//     IntGdiCloseFigure(pPath) ;
//     PATH_UnlockPath(pPath);
//     return TRUE;
// }
//
// /* PATH_RoundRect
//  *
//  * Should be called when a call to RoundRect is performed on a DC that has
//  * an open path. Returns TRUE if successful, else FALSE.
//  *
//  */
// BOOL
// PATH_RoundRect(
//     DC *dc,
//     INT x1,
//     INT y1,
//     INT x2,
//     INT y2,
//     INT ell_width,
//     INT ell_height)
// {
//     PPATH pPath;
//     RECTL rect, ellipse;
//     POINT points[16];
//     BYTE *type;
//     INT xOffset, yOffset;
//
//     if (!ell_width || !ell_height) return PATH_Rectangle( dc, x1, y1, x2, y2 );
//
//     pPath = PATH_LockPath(dc->dclevel.hPath);
//     if (!pPath) return FALSE;
//
//     if (!PATH_CheckRect(dc, &rect, x1, y1, x2, y2))
//     {
//         PATH_UnlockPath(pPath);
//         return TRUE;
//     }
//
//     w32r_RECTL_vSetRect(&ellipse, 0, 0, ell_width, ell_height);
//     w32r_IntLPtoDP( dc, (PPOINT)&ellipse, 2 );
//     RECTL_vMakeWellOrdered(&ellipse);
//     ell_width = min(w32r_RECTL_lGetWidth(&ellipse), w32r_RECTL_lGetWidth(&rect));
//     ell_height = min(w32r_RECTL_lGetHeight(&ellipse), w32r_RECTL_lGetHeight(&rect));
//
//     /*
//      * See here to understand what's happening
//      * https://stackoverflow.com/questions/1734745/how-to-create-circle-with-b%C3%A9zier-curves
//      */
//     xOffset = EngMulDiv(ell_width, 44771525, 200000000); /* w * (1 - 0.5522847) / 2 */
//     yOffset = EngMulDiv(ell_height, 44771525, 200000000); /* h * (1 - 0.5522847) / 2 */
//     DbgPrint("xOffset %d, yOffset %d, Rect WxH: %dx%d.\n",
//         xOffset, yOffset, w32r_RECTL_lGetWidth(&rect), w32r_RECTL_lGetHeight(&rect));
//
//     /*
//      * Get half width & height.
//      * Do not use integer division, we need the rounding made by EngMulDiv.
//      */
//     ell_width = EngMulDiv(ell_width, 1, 2);
//     ell_height = EngMulDiv(ell_height, 1, 2);
//
//     /* starting point */
//     points[0].x  = rect.right;
//     points[0].y  = rect.top + ell_height;
//     /* first curve */
//     points[1].x = rect.right;
//     points[1].y = rect.top + yOffset;
//     points[2].x = rect.right - xOffset;
//     points[2].y = rect.top;
//     points[3].x  = rect.right - ell_width;
//     points[3].y  = rect.top;
//     /* horizontal line */
//     points[4].x  = rect.left + ell_width;
//     points[4].y  = rect.top;
//     /* second curve */
//     points[5].x  = rect.left + xOffset;
//     points[5].y  = rect.top;
//     points[6].x  = rect.left;
//     points[6].y  = rect.top + yOffset;
//     points[7].x  = rect.left;
//     points[7].y  = rect.top + ell_height;
//     /* vertical line */
//     points[8].x  = rect.left;
//     points[8].y  = rect.bottom - ell_height;
//     /* third curve */
//     points[9].x  = rect.left;
//     points[9].y  = rect.bottom - yOffset;
//     points[10].x = rect.left + xOffset;
//     points[10].y = rect.bottom;
//     points[11].x = rect.left + ell_width;
//     points[11].y = rect.bottom;
//     /* horizontal line */
//     points[12].x = rect.right - ell_width;
//     points[12].y = rect.bottom;
//     /* fourth curve */
//     points[13].x = rect.right - xOffset;
//     points[13].y = rect.bottom;
//     points[14].x = rect.right;
//     points[14].y = rect.bottom - yOffset;
//     points[15].x = rect.right;
//     points[15].y = rect.bottom - ell_height;
//
//     if (dc->dclevel.flPath & DCPATH_CLOCKWISE) reverse_points( points, 16 );
//     if (!(type = add_points( pPath, points, 16, PT_BEZIERTO )))
//     {
//        PATH_UnlockPath(pPath);
//        return FALSE;
//     }
//     type[0] = PT_MOVETO;
//     type[4] = type[8] = type[12] = PT_LINETO;
//
//     IntGdiCloseFigure(pPath);
//     PATH_UnlockPath(pPath);
//     return TRUE;
// }
//
// /* PATH_Ellipse
//  *
//  */
// BOOL
// PATH_Ellipse(
//     PDC dc,
//     INT x1,
//     INT y1,
//     INT x2,
//     INT y2)
// {
//     PPATH pPath;
//     POINT points[13];
//     RECTL rect;
//     BYTE *type;
//     LONG xRadius, yRadius, xOffset, yOffset;
//     POINT left, top, right, bottom;
//
//     DbgPrint("PATH_Ellipse: %p -> (%d, %d) - (%d, %d)\n",
//             dc, x1, y1, x2, y2);
//
//     if (!PATH_CheckRect(dc, &rect, x1, y1, x2, y2))
//     {
//         return TRUE;
//     }
//
//     xRadius = w32r_RECTL_lGetWidth(&rect) / 2;
//     yRadius = w32r_RECTL_lGetHeight(&rect) / 2;
//
//     /* Get the four points which box our ellipse */
//     left.x = rect.left; left.y = rect.top + yRadius;
//     top.x = rect.left + xRadius; top.y = rect.top;
//     right.x = rect.right; right.y = rect.bottom - yRadius;
//     bottom.x = rect.right - xRadius; bottom.y = rect.bottom;
//
//     /*
//      * See here to understand what's happening
//      * https://stackoverflow.com/questions/1734745/how-to-create-circle-with-b%C3%A9zier-curves
//      */
//     xOffset = EngMulDiv(w32r_RECTL_lGetWidth(&rect), 55428475, 200000000); /* w * 0.55428475 / 2 */
//     yOffset = EngMulDiv(w32r_RECTL_lGetHeight(&rect), 55428475, 200000000); /* h * 0.55428475 / 2 */
//     DbgPrint("xOffset %d, yOffset %d, Rect WxH: %dx%d.\n",
//         xOffset, yOffset, w32r_RECTL_lGetWidth(&rect), w32r_RECTL_lGetHeight(&rect));
//
//     pPath = PATH_LockPath(dc->dclevel.hPath);
//     if (!pPath)
//         return FALSE;
//
//     /* Starting point: Right */
//     points[0] = right;
//
//     /* first curve - going up, left */
//     points[1] = right;
//     points[1].y -= yOffset;
//     points[2] = top;
//     points[2].x += xOffset;
//
//     /* top */
//     points[3] = top;
//
//     /* second curve - going left, down*/
//     points[4] = top;
//     points[4].x -= xOffset;
//     points[5] = left;
//     points[5].y -= yOffset;
//
//     /* Left */
//     points[6] = left;
//
//     /* Third curve - going down, right */
//     points[7] = left;
//     points[7].y += yOffset;
//     points[8] = bottom;
//     points[8].x -= xOffset;
//
//     /* bottom */
//     points[9] = bottom;
//
//     /* Fourth curve - Going right, up */
//     points[10] = bottom;
//     points[10].x += xOffset;
//     points[11] = right;
//     points[11].y += yOffset;
//
//     /* Back to starting point */
//     points[12] = right;
//
//     if (dc->dclevel.flPath & DCPATH_CLOCKWISE) reverse_points( points, 13 );
//     if (!(type = add_points( pPath, points, 13, PT_BEZIERTO )))
//     {
//         ERR("PATH_Ellipse No add\n");
//         PATH_UnlockPath(pPath);
//         return FALSE;
//     }
//     type[0] = PT_MOVETO;
//
//     IntGdiCloseFigure(pPath);
//     PATH_UnlockPath(pPath);
//     return TRUE;
// }
//
// /* PATH_DoArcPart
//  *
//  * Creates a Bezier spline that corresponds to part of an arc and appends the
//  * corresponding points to the path. The start and end angles are passed in
//  * "angleStart" and "angleEnd"; these angles should span a quarter circle
//  * at most. If "startEntryType" is non-zero, an entry of that type for the first
//  * control point is added to the path; otherwise, it is assumed that the current
//  * position is equal to the first control point.
//  */
// static
// BOOL
// PATH_DoArcPart(
//     PPATH pPath,
//     POINT corners[],
//     double angleStart,
//     double angleEnd,
//     BYTE startEntryType)
// {
//     double  halfAngle, a;
//     float  xNorm[4], yNorm[4];
//     POINT points[4];
//     BYTE *type;
//     int i, start;
//
//     ASSERT(fabs(angleEnd - angleStart) <= M_PI_2);
//
//     /* FIXME: Is there an easier way of computing this? */
//
//     /* Compute control points */
//     halfAngle = (angleEnd - angleStart) / 2.0;
//     if (fabs(halfAngle) > 1e-8)
//     {
//         a = 4.0 / 3.0 * (1 - cos(halfAngle)) / sin(halfAngle);
//         xNorm[0] = cos(angleStart);
//         yNorm[0] = sin(angleStart);
//         xNorm[1] = xNorm[0] - a * yNorm[0];
//         yNorm[1] = yNorm[0] + a * xNorm[0];
//         xNorm[3] = cos(angleEnd);
//         yNorm[3] = sin(angleEnd);
//         xNorm[2] = xNorm[3] + a * yNorm[3];
//         yNorm[2] = yNorm[3] - a * xNorm[3];
//     }
//     else
//         for (i = 0; i < 4; i++)
//         {
//             xNorm[i] = cos(angleStart);
//             yNorm[i] = sin(angleStart);
//         }
//
//     /* Add starting point to path if desired */
//     start = !startEntryType;
//
//     /* Add remaining control points */
//     for (i = start; i < 4; i++)
//     {
//         if (!PATH_ScaleNormalizedPoint(corners, *(FLOATL*)&xNorm[i], *(FLOATL*)&yNorm[i], &points[i]))
//             return FALSE;
//     }
//     if (!(type = add_points( pPath, points + start, 4 - start, PT_BEZIERTO ))) return FALSE;
//     if (!start) type[0] = startEntryType;
//
//     return TRUE;
// }
//
// /* PATH_Arc
//  *
//  * Should be called when a call to Arc is performed on a DC that has
//  * an open path. This adds up to five Bezier splines representing the arc
//  * to the path. When 'lines' is 1, we add 1 extra line to get a chord,
//  * when 'lines' is 2, we add 2 extra lines to get a pie, and when 'lines' is
//  * -1 we add 1 extra line from the current DC position to the starting position
//  * of the arc before drawing the arc itself (arcto). Returns TRUE if successful,
//  * else FALSE.
//  */
// BOOL
// FASTCALL
// PATH_Arc(
//     PDC dc,
//     INT x1,
//     INT y1,
//     INT x2,
//     INT y2,
//     INT xStart,
//     INT yStart,
//     INT xEnd,
//     INT yEnd,
//     INT direction,
//     INT lines)
// {
//     double  angleStart, angleEnd, angleStartQuadrant, angleEndQuadrant = 0.0;
//     /* Initialize angleEndQuadrant to silence gcc's warning */
//     FLOATL  x, y;
//     POINT corners[2], pointStart, pointEnd;
//     POINT   centre, pointCurPos;
//     BOOL    start, end, Ret = TRUE;
//     INT     temp;
//     BOOL    clockwise;
//     PPATH   pPath;
//
//     /* FIXME: This function should check for all possible error returns */
//     /* FIXME: Do we have to respect newStroke? */
//
//     ASSERT(dc);
//
//     pPath = PATH_LockPath(dc->dclevel.hPath);
//     if (!pPath) return FALSE;
//
//     if (direction)
//        clockwise = ((direction == AD_CLOCKWISE) !=0 );
//     else
//        clockwise = ((dc->dclevel.flPath & DCPATH_CLOCKWISE) != 0);
//
//     /* Check for zero height / width */
//     /* FIXME: Only in GM_COMPATIBLE? */
//     if (x1 == x2 || y1 == y2)
//     {
//         Ret = TRUE;
//         goto ArcExit;
//     }
//     /* Convert points to device coordinates */
//     corners[0].x = x1; corners[0].y = y1;
//     corners[1].x = x2; corners[1].y = y2;
//     pointStart.x = xStart; pointStart.y = yStart;
//     pointEnd.x = xEnd; pointEnd.y = yEnd;
//     w32r_INTERNAL_LPTODP(dc, corners, 2);
//     w32r_INTERNAL_LPTODP(dc, &pointStart, 1);
//     w32r_INTERNAL_LPTODP(dc, &pointEnd, 1);
//
//     /* Make sure first corner is top left and second corner is bottom right */
//     if (corners[0].x > corners[1].x)
//     {
//         temp = corners[0].x;
//         corners[0].x = corners[1].x;
//         corners[1].x = temp;
//     }
//     if (corners[0].y > corners[1].y)
//     {
//         temp = corners[0].y;
//         corners[0].y = corners[1].y;
//         corners[1].y = temp;
//     }
//
//     /* Compute start and end angle */
//     PATH_NormalizePoint(corners, &pointStart, &x, &y);
//     angleStart = atan2(*(FLOAT*)&y, *(FLOAT*)&x);
//     PATH_NormalizePoint(corners, &pointEnd, &x, &y);
//     angleEnd = atan2(*(FLOAT*)&y, *(FLOAT*)&x);
//
//     /* Make sure the end angle is "on the right side" of the start angle */
//     if (clockwise)
//     {
//         if (angleEnd <= angleStart)
//         {
//             angleEnd += 2 * M_PI;
//             ASSERT(angleEnd >= angleStart);
//         }
//     }
//     else
//     {
//         if (angleEnd >= angleStart)
//         {
//             angleEnd -= 2 * M_PI;
//             ASSERT(angleEnd <= angleStart);
//         }
//     }
//
//     /* In GM_COMPATIBLE, don't include bottom and right edges */
//     if (dc->pdcattr->iGraphicsMode == GM_COMPATIBLE)
//     {
//         corners[1].x--;
//         corners[1].y--;
//     }
//
//     /* arcto: Add a PT_MOVETO only if this is the first entry in a stroke */
//     if (lines == GdiTypeArcTo && pPath->newStroke) // -1
//     {
//         pPath->newStroke = FALSE;
//         IntGetCurrentPositionEx(dc, &pointCurPos);
//         w32r_CoordLPtoDP(dc, &pointCurPos);
//         if (!PATH_AddEntry(pPath, &pointCurPos, PT_MOVETO))
//         {
//             Ret = FALSE;
//             goto ArcExit;
//         }
//     }
//
//     /* Add the arc to the path with one Bezier spline per quadrant that the
//      * arc spans */
//     start = TRUE;
//     end = FALSE;
//     do
//     {
//         /* Determine the start and end angles for this quadrant */
//         if (start)
//         {
//             angleStartQuadrant = angleStart;
//             if (clockwise)
//                 angleEndQuadrant = (floor(angleStart / M_PI_2) + 1.0) * M_PI_2;
//             else
//                 angleEndQuadrant = (ceil(angleStart / M_PI_2) - 1.0) * M_PI_2;
//         }
//         else
//         {
//             angleStartQuadrant = angleEndQuadrant;
//             if (clockwise)
//                 angleEndQuadrant += M_PI_2;
//             else
//                 angleEndQuadrant -= M_PI_2;
//         }
//
//         /* Have we reached the last part of the arc? */
//         if ((clockwise && angleEnd < angleEndQuadrant) ||
//             (!clockwise && angleEnd > angleEndQuadrant))
//         {
//             /* Adjust the end angle for this quadrant */
//             angleEndQuadrant = angleEnd;
//             end = TRUE;
//         }
//
//         /* Add the Bezier spline to the path */
//         PATH_DoArcPart(pPath,
//                        corners,
//                        angleStartQuadrant,
//                        angleEndQuadrant,
//                        start ? (lines == GdiTypeArcTo ? PT_LINETO : PT_MOVETO) : FALSE); // -1
//         start = FALSE;
//     }
//     while (!end);
//
//     if (lines == GdiTypeArcTo)
//     {
//        update_current_pos( pPath );
//     }
//     else /* chord: close figure. pie: add line and close figure */
//     if (lines == GdiTypeChord) // 1
//     {
//         IntGdiCloseFigure(pPath);
//     }
//     else if (lines == GdiTypePie) // 2
//     {
//         centre.x = (corners[0].x + corners[1].x) / 2;
//         centre.y = (corners[0].y + corners[1].y) / 2;
//         if (!PATH_AddEntry(pPath, &centre, PT_LINETO | PT_CLOSEFIGURE))
//             Ret = FALSE;
//     }
// ArcExit:
//     PATH_UnlockPath(pPath);
//     return Ret;
// }
// /* --- pretranslation split: the remaining function groups live in path.split2..3.c and are #included below so this stays ONE translation unit (zero CMake/linkage change). Reassembled verbatim by cpp. --- */
// /* path.split2.c — #include fragment of path.c, carved for pretranslation so rustconv gets a small file. NOT a standalone TU; NOT in CMake; do not compile directly. The preprocessor reassembles the original path.c TU verbatim. See docs/engineering/win32k-pretranslation-audit.md */
//
//
// BOOL
// FASTCALL
// PATH_PolyBezierTo(
//     PDC dc,
//     const POINT *pts,
//     DWORD cbPoints)
// {
//     PPATH pPath;
//     BOOL ret;
//
//     ASSERT(dc);
//     ASSERT(pts);
//     ASSERT(cbPoints);
//
//     pPath = PATH_LockPath(dc->dclevel.hPath);
//     if (!pPath) return FALSE;
//
//     ret = add_log_points_new_stroke( dc, pPath, pts, cbPoints, PT_BEZIERTO );
//
//     PATH_UnlockPath(pPath);
//     return ret;
// }
//
// BOOL
// FASTCALL
// PATH_PolyBezier(
//     PDC dc,
//     const POINT *pts,
//     DWORD cbPoints)
// {
//     PPATH pPath;
//     BYTE *type;
//
//     ASSERT(dc);
//     ASSERT(pts);
//     ASSERT(cbPoints);
//
//     pPath = PATH_LockPath(dc->dclevel.hPath);
//     if (!pPath) return FALSE;
//
//     type = add_log_points( dc, pPath, pts, cbPoints, PT_BEZIERTO );
//     if (!type) return FALSE;
//
//     type[0] = PT_MOVETO;
//
//     PATH_UnlockPath(pPath);
//     return TRUE;
// }
//
// BOOL
// FASTCALL
// PATH_PolyDraw(
//     PDC dc,
//     const POINT *pts,
//     const BYTE *types,
//     DWORD cbPoints)
// {
//     PPATH pPath;
//     POINT orig_pos, cur_pos;
//     ULONG i, lastmove = 0;
//
//     pPath = PATH_LockPath(dc->dclevel.hPath);
//     if (!pPath) return FALSE;
//
//     if (pPath->state != PATH_Open)
//     {
//         PATH_UnlockPath(pPath);
//         return FALSE;
//     }
//
//     for (i = 0; i < pPath->numEntriesUsed; i++) if (pPath->pFlags[i] == PT_MOVETO) lastmove = i;
//     orig_pos = pPath->pos;
//
//     IntGetCurrentPositionEx(dc, &cur_pos);
//
//     DbgPrint("PPD : Current pos X %d Y %d\n",pPath->pos.x, pPath->pos.y);
//     DbgPrint("PPD : last %d pos X %d Y %d\n",lastmove, pPath->pPoints[lastmove].x, pPath->pPoints[lastmove].y);
//
//
//     for(i = 0; i < cbPoints; i++)
//     {
//         switch (types[i])
//         {
//         case PT_MOVETO:
//             pPath->newStroke = TRUE;
//             pPath->pos = pts[i];
//             w32r_IntLPtoDP( dc, &pPath->pos, 1);
//             lastmove = pPath->numEntriesUsed;
//             break;
//         case PT_LINETO:
//         case PT_LINETO | PT_CLOSEFIGURE:
//             if (!add_log_points_new_stroke( dc, pPath, &pts[i], 1, PT_LINETO ))
//             {
//                PATH_UnlockPath(pPath);
//                return FALSE;
//             }
//             break;
//         case PT_BEZIERTO:
//             if ((i + 2 < cbPoints) && (types[i + 1] == PT_BEZIERTO) &&
//                 (types[i + 2] & ~PT_CLOSEFIGURE) == PT_BEZIERTO)
//             {
//                 if (!add_log_points_new_stroke( dc, pPath, &pts[i], 3, PT_BEZIERTO ))
//                 {
//                    PATH_UnlockPath(pPath);
//                    return FALSE;
//                 }
//                 i += 2;
//                 break;
//             }
//             /* fall through */
//         default:
//             /* restore original position */
//             pPath->pos = orig_pos;
//
//             DbgPrint("PPD Bad   : pos X %d Y %d\n",pPath->pos.x, pPath->pos.y);
//
//             IntGdiMoveToEx(dc, cur_pos.x, cur_pos.y, NULL);
//
//             PATH_UnlockPath(pPath);
//             return FALSE;
//         }
//
//         if (types[i] & PT_CLOSEFIGURE)
//         {
//             close_figure( pPath );
//             pPath->pos = pPath->pPoints[lastmove];
//             DbgPrint("PPD close : pos X %d Y %d\n",pPath->pos.x, pPath->pos.y);
//         }
//     }
//     PATH_UnlockPath(pPath);
//     return TRUE;
// }
//
// BOOL
// FASTCALL
// PATH_PolylineTo(
//     PDC dc,
//     const POINT *pts,
//     DWORD cbPoints)
// {
//     PPATH pPath;
//     BOOL ret;
//
//     ASSERT(dc);
//     ASSERT(pts);
//     ASSERT(cbPoints);
//
//     if (cbPoints < 1) return FALSE;
//
//     pPath = PATH_LockPath(dc->dclevel.hPath);
//     if (!pPath) return FALSE;
//
//     ret = add_log_points_new_stroke( dc, pPath, pts, cbPoints, PT_LINETO );
//     PATH_UnlockPath(pPath);
//     return ret;
// }
//
// BOOL
// FASTCALL
// PATH_PolyPolygon(
//     PDC dc,
//     const POINT* pts,
//     const INT* counts,
//     UINT polygons)
// {
//     UINT poly, count;
//     BYTE *type;
//     PPATH pPath;
//
//     ASSERT(dc);
//     ASSERT(pts);
//     ASSERT(counts);
//     ASSERT(polygons);
//
//     if (!polygons) return FALSE;
//
//     pPath = PATH_LockPath(dc->dclevel.hPath);
//     if (!pPath) return FALSE;
//
//
//     for (poly = count = 0; poly < polygons; poly++)
//     {
//         if (counts[poly] < 2)
//         {
//            PATH_UnlockPath(pPath);
//            return FALSE;
//         }
//         count += counts[poly];
//     }
//
//     type = add_log_points( dc, pPath, pts, count, PT_LINETO );
//     if (!type)
//     {
//        PATH_UnlockPath(pPath);
//        return FALSE;
//     }
//
//     /* make the first point of each polyline a PT_MOVETO, and close the last one */
//     for (poly = 0; poly < polygons; type += counts[poly++])
//     {
//         type[0] = PT_MOVETO;
//         type[counts[poly] - 1] = PT_LINETO | PT_CLOSEFIGURE;
//     }
//     PATH_UnlockPath(pPath);
//     return TRUE;
// }
//
// BOOL
// FASTCALL
// PATH_PolyPolyline(
//     PDC dc,
//     const POINT* pts,
//     const DWORD* counts,
//     DWORD polylines)
// {
//     POINT pt;
//     ULONG poly, point, i;
//     PPATH pPath;
//
//     ASSERT(dc);
//     ASSERT(pts);
//     ASSERT(counts);
//     ASSERT(polylines);
//
//     pPath = PATH_LockPath(dc->dclevel.hPath);
//     if (!pPath)
//     {
//        return FALSE;
//     }
//
//     for (i = 0, poly = 0; poly < polylines; poly++)
//     {
//         for (point = 0; point < counts[poly]; point++, i++)
//         {
//             pt = pts[i];
//             w32r_CoordLPtoDP(dc, &pt);
//             PATH_AddEntry(pPath, &pt, (point == 0) ? PT_MOVETO : PT_LINETO);
//         }
//     }
//     DbgPrint("PATH_PolyPolyline end count %d\n",pPath->numEntriesUsed);
//     PATH_UnlockPath(pPath);
//     return TRUE;
// }
//
// /* PATH_AddFlatBezier
//  *
//  */
// BOOL
// FASTCALL
// PATH_AddFlatBezier(
//     PPATH pPath,
//     POINT *pt,
//     BOOL closed)
// {
//     POINT *pts;
//     BOOL ret = FALSE;
//     INT no, i;
//
//     pts = GDI_Bezier(pt, 4, &no);
//     if (!pts) return FALSE;
//
//     for (i = 1; i < no; i++)
//     {
//         if (!(ret = PATH_AddEntry(pPath, &pts[i],  (i == no - 1 && closed) ? PT_LINETO | PT_CLOSEFIGURE : PT_LINETO)))
//            break;
//     }
//
//     ExFreePoolWithTag(pts, TAG_BEZIER);
//     return ret;
// }
//
// /* PATH_FlattenPath
//  *
//  * Replaces Beziers with line segments
//  *
//  */
// PPATH
// FASTCALL
// PATH_FlattenPath(PPATH pPath)
// {
//     PPATH newPath;
//     INT srcpt;
//     DbgPrint("PATH_FlattenPath\n");
//     if (!(newPath = PATH_CreatePath(pPath->numEntriesUsed))) return NULL;
//
//     for (srcpt = 0; srcpt < pPath->numEntriesUsed; srcpt++)
//     {
//         switch(pPath->pFlags[srcpt] & ~PT_CLOSEFIGURE)
//         {
//             case PT_MOVETO:
//             case PT_LINETO:
//                 if (!PATH_AddEntry(newPath, &pPath->pPoints[srcpt], pPath->pFlags[srcpt]))
//                 {
//                    PATH_UnlockPath(newPath);
//                    PATH_Delete(newPath->BaseObject.hHmgr);
//                    return NULL;
//                 }
//                 break;
//             case PT_BEZIERTO:
//                 if(!PATH_AddFlatBezier(newPath, &pPath->pPoints[srcpt - 1], pPath->pFlags[srcpt + 2] & PT_CLOSEFIGURE))
//                 {
//                    PATH_UnlockPath(newPath);
//                    PATH_Delete(newPath->BaseObject.hHmgr);
//                    return NULL;
//                 }
//                 srcpt += 2;
//                 break;
//         }
//     }
//     DbgPrint("PATH_FlattenPath good\n");
//     newPath->state = pPath->state;
//     return newPath;
// }
//
// /* PATH_PathToRegion
//  *
//  * Fills Rgn from the specified path using the specified polygon
//  * filling mode. The path is left unchanged.
//  */
// BOOL
// FASTCALL
// PATH_PathToRegion(
//     PPATH pPath,
//     INT Mode,
//     PREGION Rgn)
// {
//     int i, pos, polygons;
//     PULONG counts;
//     int Ret;
//
//     if (!pPath->numEntriesUsed) return FALSE;
//
//     counts = ExAllocatePoolWithTag(PagedPool, (pPath->numEntriesUsed / 2) * sizeof(*counts), TAG_PATH);
//     if (!counts)
//     {
//         ERR("Failed to allocate %lu strokes\n", (pPath->numEntriesUsed / 2) * sizeof(*counts));
//         EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return FALSE;
//     }
//
//     pos = polygons = 0;
//     ASSERT( pPath->pFlags[0] == PT_MOVETO );
//     for (i = 1; i < pPath->numEntriesUsed; i++)
//     {
//         if (pPath->pFlags[i] != PT_MOVETO) continue;
//         counts[polygons++] = i - pos;
//         pos = i;
//     }
//     if (i > pos + 1) counts[polygons++] = i - pos;
//
//     ASSERT( polygons <= pPath->numEntriesUsed / 2 );
//
//     /* Fill the region with the strokes */
//     Ret = REGION_SetPolyPolygonRgn(Rgn,
//                                    pPath->pPoints,
//                                    counts,
//                                    polygons,
//                                    Mode);
//     if (!Ret)
//     {
//         ERR("REGION_SetPolyPolygonRgn failed\n");
//     }
//
//     ExFreePoolWithTag(counts, TAG_PATH);
//
//     /* Success! */
//     return Ret;
// }
//
// /* PATH_FillPath
//  *
//  * You can play with this as long as you like, but if you break Area.exe the purge will Begain on Path!!!
//  *
//  */
// BOOL
// FASTCALL
// PATH_FillPath(
//     PDC dc,
//     PPATH pPath)
// {
//     return PATH_FillPathEx(dc, pPath, NULL);
// }
//
// BOOL
// FASTCALL
// PATH_FillPathEx(
//     PDC dc,
//     PPATH pPath,
//     PBRUSH pbrFill)
// {
//     INT   mapMode, graphicsMode;
//     SIZE  ptViewportExt, ptWindowExt;
//     POINTL ptViewportOrg, ptWindowOrg;
//     XFORML xform;
//     PREGION  Rgn;
//     PDC_ATTR pdcattr = dc->pdcattr;
//
//     /* Allocate a temporary region */
//     Rgn = IntSysCreateRectpRgn(0, 0, 0, 0);
//     if (!Rgn)
//     {
//         EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return FALSE;
//     }
//
//     if (!PATH_PathToRegion(pPath, pdcattr->jFillMode, Rgn))
//     {
//         DbgPrint("PFP : Fail P2R\n");
//         /* EngSetLastError ? */
//         REGION_Delete(Rgn);
//         return FALSE;
//     }
//
//     /* Since PaintRgn interprets the region as being in logical coordinates
//      * but the points we store for the path are already in device
//      * coordinates, we have to set the mapping mode to MM_TEXT temporarily.
//      * Using SaveDC to save information about the mapping mode / world
//      * transform would be easier but would require more overhead, especially
//      * now that SaveDC saves the current path.
//      */
//
//     /* Save the information about the old mapping mode */
//     mapMode = pdcattr->iMapMode;
//     ptViewportExt = pdcattr->szlViewportExt;
//     ptViewportOrg = pdcattr->ptlViewportOrg;
//     ptWindowExt   = pdcattr->szlWindowExt;
//     ptWindowOrg   = pdcattr->ptlWindowOrg;
//
//     /* Save world transform
//      * NB: The Windows documentation on world transforms would lead one to
//      * believe that this has to be done only in GM_ADVANCED; however, my
//      * tests show that resetting the graphics mode to GM_COMPATIBLE does
//      * not reset the world transform.
//      */
//     w32r_MatrixS2XForm(&xform, &dc->pdcattr->mxWorldToPage);
//
//     /* Set MM_TEXT */
//     IntGdiSetMapMode(dc, MM_TEXT);
//     pdcattr->ptlViewportOrg.x = 0;
//     pdcattr->ptlViewportOrg.y = 0;
//     pdcattr->ptlWindowOrg.x = 0;
//     pdcattr->ptlWindowOrg.y = 0;
//
//     graphicsMode = pdcattr->iGraphicsMode;
//     pdcattr->iGraphicsMode = GM_ADVANCED;
//     GreModifyWorldTransform(dc, &xform, MWT_IDENTITY);
//     pdcattr->iGraphicsMode =  graphicsMode;
//
//     /* Paint the region */
//     IntGdiFillRgn(dc, Rgn, pbrFill);
//     REGION_Delete(Rgn);
//     /* Restore the old mapping mode */
//     IntGdiSetMapMode(dc, mapMode);
//     pdcattr->szlViewportExt = ptViewportExt;
//     pdcattr->ptlViewportOrg = ptViewportOrg;
//     pdcattr->szlWindowExt   = ptWindowExt;
//     pdcattr->ptlWindowOrg   = ptWindowOrg;
//
//     /* Go to GM_ADVANCED temporarily to restore the world transform */
//     graphicsMode = pdcattr->iGraphicsMode;
//     pdcattr->iGraphicsMode = GM_ADVANCED;
//     GreModifyWorldTransform(dc, &xform, MWT_SET);
//     pdcattr->iGraphicsMode = graphicsMode;
//     return TRUE;
// }
//
// BOOL
// FASTCALL
// PATH_StrokePath(
//     DC *dc,
//     PPATH pPath)
// {
//     BOOL ret = FALSE;
//     INT nLinePts, nAlloc, jOldFillMode, i = 0;
//     POINT *pLinePts = NULL;
//     POINT ptViewportOrg, ptWindowOrg;
//     SIZE szViewportExt, szWindowExt;
//     DWORD mapMode, graphicsMode;
//     XFORM xform;
//     PDC_ATTR pdcattr = dc->pdcattr;
//     PBRUSH pbrLine;
//     PPATH pNewPath;
//
//     DbgPrint("Enter %s\n", __FUNCTION__);
//
//     pbrLine = dc->dclevel.pbrLine;
//     if (w32r_IntIsEffectiveWidePen(pbrLine))
//     {
//         pNewPath = PATH_WidenPathEx(dc, pPath);
//         if (pNewPath)
//         {
//             /* Fill the path with the WINDING fill mode */
//             jOldFillMode = pdcattr->jFillMode;
//             pdcattr->jFillMode = WINDING;
//             PATH_FillPathEx(dc, pNewPath, pbrLine);
//             pdcattr->jFillMode = jOldFillMode;
//
//             PATH_Delete(pNewPath->BaseObject.hHmgr);
//             return TRUE;
//         }
//     }
//
//     /* Save the mapping mode info */
//     mapMode = pdcattr->iMapMode;
//
//     szViewportExt = *w32r_DC_pszlViewportExt(dc);
//     ptViewportOrg = dc->pdcattr->ptlViewportOrg;
//     szWindowExt = dc->pdcattr->szlWindowExt;
//     ptWindowOrg = dc->pdcattr->ptlWindowOrg;
//
//     w32r_MatrixS2XForm(&xform, &dc->pdcattr->mxWorldToPage);
//
//     /* Set MM_TEXT */
//     pdcattr->iMapMode = MM_TEXT;
//     pdcattr->ptlViewportOrg.x = 0;
//     pdcattr->ptlViewportOrg.y = 0;
//     pdcattr->ptlWindowOrg.x = 0;
//     pdcattr->ptlWindowOrg.y = 0;
//     graphicsMode = pdcattr->iGraphicsMode;
//     pdcattr->iGraphicsMode = GM_ADVANCED;
//     GreModifyWorldTransform(dc, (XFORML*)&xform, MWT_IDENTITY);
//     pdcattr->iGraphicsMode = graphicsMode;
//
//     /* Allocate enough memory for the worst case without beziers (one PT_MOVETO
//      * and the rest PT_LINETO with PT_CLOSEFIGURE at the end) plus some buffer
//      * space in case we get one to keep the number of reallocations small. */
//     nAlloc = pPath->numEntriesUsed + 1 + 300;
//     pLinePts = ExAllocatePoolWithTag(PagedPool, nAlloc * sizeof(POINT), TAG_PATH);
//     if (!pLinePts)
//     {
//         ERR("Can't allocate pool!\n");
//         EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         goto end;
//     }
//     nLinePts = 0;
//
//     for (i = 0; i < pPath->numEntriesUsed; i++)
//     {
//         if ((i == 0 || (pPath->pFlags[i - 1] & PT_CLOSEFIGURE))
//                 && (pPath->pFlags[i] != PT_MOVETO))
//         {
//             ERR("Expected PT_MOVETO %s, got path flag %d\n",
//                     i == 0 ? "as first point" : "after PT_CLOSEFIGURE",
//                     (INT)pPath->pFlags[i]);
//             goto end;
//         }
//
//         switch(pPath->pFlags[i])
//         {
//             case PT_MOVETO:
//                 DbgPrint("Got PT_MOVETO (%ld, %ld)\n",
//                        pPath->pPoints[i].x, pPath->pPoints[i].y);
//                 if (nLinePts >= 2) IntGdiPolyline(dc, pLinePts, nLinePts);
//                 nLinePts = 0;
//                 pLinePts[nLinePts++] = pPath->pPoints[i];
//                 break;
//             case PT_LINETO:
//             case (PT_LINETO | PT_CLOSEFIGURE):
//                 DbgPrint("Got PT_LINETO (%ld, %ld)\n",
//                        pPath->pPoints[i].x, pPath->pPoints[i].y);
//                 pLinePts[nLinePts++] = pPath->pPoints[i];
//                 break;
//             case PT_BEZIERTO:
//                 DbgPrint("Got PT_BEZIERTO\n");
//                 if (pPath->pFlags[i + 1] != PT_BEZIERTO ||
//                         (pPath->pFlags[i + 2] & ~PT_CLOSEFIGURE) != PT_BEZIERTO)
//                 {
//                     ERR("Path didn't contain 3 successive PT_BEZIERTOs\n");
//                     ret = FALSE;
//                     goto end;
//                 }
//                 else
//                 {
//                     INT nBzrPts, nMinAlloc;
//                     POINT *pBzrPts = GDI_Bezier(&pPath->pPoints[i - 1], 4, &nBzrPts);
//                     /* Make sure we have allocated enough memory for the lines of
//                      * this bezier and the rest of the path, assuming we won't get
//                      * another one (since we won't reallocate again then). */
//                     nMinAlloc = nLinePts + (pPath->numEntriesUsed - i) + nBzrPts;
//                     if (nAlloc < nMinAlloc)
//                     {
//                         // Reallocate memory
//
//                         POINT *Realloc = NULL;
//                         nAlloc = nMinAlloc * 2;
//
//                         Realloc = ExAllocatePoolWithTag(PagedPool,
//                                                         nAlloc * sizeof(POINT),
//                                                         TAG_PATH);
//
//                         if (!Realloc)
//                         {
//                             ERR("Can't allocate pool!\n");
//                             ExFreePoolWithTag(pBzrPts, TAG_BEZIER);
//                             goto end;
//                         }
//
//                         memcpy(Realloc, pLinePts, nLinePts * sizeof(POINT));
//                         ExFreePoolWithTag(pLinePts, TAG_PATH);
//                         pLinePts = Realloc;
//                     }
//                     memcpy(&pLinePts[nLinePts], &pBzrPts[1], (nBzrPts - 1) * sizeof(POINT));
//                     nLinePts += nBzrPts - 1;
//                     ExFreePoolWithTag(pBzrPts, TAG_BEZIER);
//                     i += 2;
//                 }
//                 break;
//             default:
//                 ERR("Got path flag %d (not supported)\n", (INT)pPath->pFlags[i]);
//                 goto end;
//         }
//
//         if (pPath->pFlags[i] & PT_CLOSEFIGURE)
//         {
//             pLinePts[nLinePts++] = pLinePts[0];
//         }
//     }
//     if (nLinePts >= 2)
//         IntGdiPolyline(dc, pLinePts, nLinePts);
//
//     ret = TRUE;
//
// end:
//     if (pLinePts) ExFreePoolWithTag(pLinePts, TAG_PATH);
//
//     /* Restore the old mapping mode */
//     pdcattr->iMapMode =  mapMode;
//     pdcattr->szlWindowExt.cx = szWindowExt.cx;
//     pdcattr->szlWindowExt.cy = szWindowExt.cy;
//     pdcattr->ptlWindowOrg.x = ptWindowOrg.x;
//     pdcattr->ptlWindowOrg.y = ptWindowOrg.y;
//
//     pdcattr->szlViewportExt.cx = szViewportExt.cx;
//     pdcattr->szlViewportExt.cy = szViewportExt.cy;
//     pdcattr->ptlViewportOrg.x = ptViewportOrg.x;
//     pdcattr->ptlViewportOrg.y = ptViewportOrg.y;
//
//     /* Restore the world transform */
//     XForm2MatrixS(&dc->pdcattr->mxWorldToPage, &xform);
//
//     /* If we've moved the current point then get its new position
//        which will be in device (MM_TEXT) co-ords, convert it to
//        logical co-ords and re-set it.  This basically updates
//        dc->CurPosX|Y so that their values are in the correct mapping
//        mode.
//     */
//     if (i > 0)
//     {
//         POINT pt;
//         IntGetCurrentPositionEx(dc, &pt);
//         w32r_IntDPtoLP(dc, &pt, 1);
//         IntGdiMoveToEx(dc, pt.x, pt.y, NULL);
//     }
//     DbgPrint("Leave %s, ret=%d\n", __FUNCTION__, ret);
//     return ret;
// }
//
// #define round(x) ((int)((x)>0?(x)+0.5:(x)-0.5))
//
// PPATH FASTCALL
// IntGdiWidenPath(PPATH pPath, UINT penWidth, UINT penStyle, FLOAT eMiterLimit)
// {
//     INT i, j, numStrokes, numOldStrokes, penWidthIn, penWidthOut;
//     PPATH flat_path, pNewPath = NULL, *pStrokes = NULL, *pOldStrokes, pUpPath, pDownPath;
//     BYTE *type;
//     DWORD joint, endcap;
//     KFLOATING_SAVE fpsave;
//
//     endcap = (PS_ENDCAP_MASK & penStyle);
//     joint = (PS_JOIN_MASK & penStyle);
//
//     if (!(flat_path = PATH_FlattenPath(pPath)))
//     {
//         ERR("PATH_FlattenPath\n");
//         return NULL;
//     }
//
//     penWidthIn = penWidth / 2;
//     penWidthOut = penWidth / 2;
//     if (penWidthIn + penWidthOut < penWidth)
//         penWidthOut++;
//
//     numStrokes = 0;
//
//     for (i = 0, j = 0; i < flat_path->numEntriesUsed; i++, j++)
//     {
//         POINT point;
//         if ((i == 0 || (flat_path->pFlags[i - 1] & PT_CLOSEFIGURE)) &&
//                 (flat_path->pFlags[i] != PT_MOVETO))
//         {
//             ERR("Expected PT_MOVETO %s, got path flag %c\n",
//                     i == 0 ? "as first point" : "after PT_CLOSEFIGURE",
//                     flat_path->pFlags[i]);
//             goto Exit;
//         }
//         switch(flat_path->pFlags[i])
//         {
//             case PT_MOVETO:
//                 if (numStrokes > 0)
//                 {
//                     pStrokes[numStrokes - 1]->state = PATH_Closed;
//                 }
//                 numOldStrokes = numStrokes;
//                 numStrokes++;
//                 j = 0;
//                 if (numStrokes == 1)
//                     pStrokes = ExAllocatePoolWithTag(PagedPool, sizeof(*pStrokes), TAG_PATH);
//                 else
//                 {
//                     pOldStrokes = pStrokes; // Save old pointer.
//                     pStrokes = ExAllocatePoolWithTag(PagedPool, numStrokes * sizeof(*pStrokes), TAG_PATH);
//                     if (!pStrokes)
//                     {
//                        ExFreePoolWithTag(pOldStrokes, TAG_PATH);
//                        goto Exit;
//                     }
//                     RtlCopyMemory(pStrokes, pOldStrokes, numOldStrokes * sizeof(PPATH));
//                     ExFreePoolWithTag(pOldStrokes, TAG_PATH); // Free old pointer.
//                 }
//                 if (!pStrokes)
//                 {
//                    goto Exit;
//                 }
//                 pStrokes[numStrokes - 1] = ExAllocatePoolWithTag(PagedPool, sizeof(PATH), TAG_PATH);
//                 if (!pStrokes[numStrokes - 1])
//                 {
//                     ASSERT(FALSE); // FIXME
//                 }
//                 PATH_InitGdiPath(pStrokes[numStrokes - 1]);
//                 pStrokes[numStrokes - 1]->state = PATH_Open;
//             case PT_LINETO:
//             case (PT_LINETO | PT_CLOSEFIGURE):
//                 point.x = flat_path->pPoints[i].x;
//                 point.y = flat_path->pPoints[i].y;
//                 PATH_AddEntry(pStrokes[numStrokes - 1], &point, flat_path->pFlags[i]);
//                 break;
//             case PT_BEZIERTO:
//                 /* Should never happen because of the FlattenPath call */
//                 ERR("Should never happen\n");
//                 break;
//             default:
//                 ERR("Got path flag %c\n", flat_path->pFlags[i]);
//                 goto Exit;
//         }
//     }
//
//     pNewPath = PATH_CreatePath( flat_path->numEntriesUsed );
//     if (pNewPath == NULL)
//     {
//         ERR("PATH_CreatePath\n");
//         goto Exit;
//     }
//
//     w32r_KeSaveFloatingPointState(&fpsave);
//
//     for (i = 0; i < numStrokes; i++)
//     {
//         pUpPath = ExAllocatePoolWithTag(PagedPool, sizeof(PATH), TAG_PATH);
//         PATH_InitGdiPath(pUpPath);
//         pUpPath->state = PATH_Open;
//         pDownPath = ExAllocatePoolWithTag(PagedPool, sizeof(PATH), TAG_PATH);
//         PATH_InitGdiPath(pDownPath);
//         pDownPath->state = PATH_Open;
//
//         for (j = 0; j < pStrokes[i]->numEntriesUsed; j++)
//         {
//             /* Beginning or end of the path if not closed */
//             if ((!(pStrokes[i]->pFlags[pStrokes[i]->numEntriesUsed - 1] & PT_CLOSEFIGURE)) && (j == 0 || j == pStrokes[i]->numEntriesUsed - 1))
//             {
//                 /* Compute segment angle */
//                 INT xo, yo, xa, ya;
//                 double theta;
//                 POINT pt;
//                 POINT corners[2];
//                 if (j == 0)
//                 {
//                     xo = pStrokes[i]->pPoints[j].x;
//                     yo = pStrokes[i]->pPoints[j].y;
//                     xa = pStrokes[i]->pPoints[1].x;
//                     ya = pStrokes[i]->pPoints[1].y;
//                 }
//                 else
//                 {
//                     xa = pStrokes[i]->pPoints[j - 1].x;
//                     ya = pStrokes[i]->pPoints[j - 1].y;
//                     xo = pStrokes[i]->pPoints[j].x;
//                     yo = pStrokes[i]->pPoints[j].y;
//                 }
//                 theta = atan2(ya - yo, xa - xo);
//                 switch(endcap)
//                 {
//                     case PS_ENDCAP_SQUARE :
//                         pt.x = xo + round(sqrt(2) * penWidthOut * cos(M_PI_4 + theta));
//                         pt.y = yo + round(sqrt(2) * penWidthOut * sin(M_PI_4 + theta));
//                         PATH_AddEntry(pUpPath, &pt, (j == 0 ? PT_MOVETO : PT_LINETO));
//                         pt.x = xo + round(sqrt(2) * penWidthIn * cos(- M_PI_4 + theta));
//                         pt.y = yo + round(sqrt(2) * penWidthIn * sin(- M_PI_4 + theta));
//                         PATH_AddEntry(pUpPath, &pt, PT_LINETO);
//                         break;
//                     case PS_ENDCAP_FLAT :
//                         pt.x = xo + round(penWidthOut * cos(theta + M_PI_2));
//                         pt.y = yo + round(penWidthOut * sin(theta + M_PI_2));
//                         PATH_AddEntry(pUpPath, &pt, (j == 0 ? PT_MOVETO : PT_LINETO));
//                         pt.x = xo - round(penWidthIn * cos(theta + M_PI_2));
//                         pt.y = yo - round(penWidthIn * sin(theta + M_PI_2));
//                         PATH_AddEntry(pUpPath, &pt, PT_LINETO);
//                         break;
//                     case PS_ENDCAP_ROUND :
//                     default :
//                         corners[0].x = xo - penWidthIn;
//                         corners[0].y = yo - penWidthIn;
//                         corners[1].x = xo + penWidthOut;
//                         corners[1].y = yo + penWidthOut;
//                         PATH_DoArcPart(pUpPath , corners, theta + M_PI_2 , theta + 3 * M_PI_4, (j == 0 ? PT_MOVETO : FALSE));
//                         PATH_DoArcPart(pUpPath , corners, theta + 3 * M_PI_4 , theta + M_PI, FALSE);
//                         PATH_DoArcPart(pUpPath , corners, theta + M_PI, theta +  5 * M_PI_4, FALSE);
//                         PATH_DoArcPart(pUpPath , corners, theta + 5 * M_PI_4 , theta + 3 * M_PI_2, FALSE);
//                         break;
//                 }
//             }
//             /* Corpse of the path */
//             else
//             {
//                 /* Compute angle */
//                 INT previous, next;
//                 double xa, ya, xb, yb, xo, yo;
//                 double alpha, theta, miterWidth;
//                 DWORD _joint = joint;
//                 POINT pt;
//                 PPATH pInsidePath, pOutsidePath;
//                 if (j > 0 && j < pStrokes[i]->numEntriesUsed - 1)
//                 {
//                     previous = j - 1;
//                     next = j + 1;
//                 }
//                 else if (j == 0)
//                 {
//                     previous = pStrokes[i]->numEntriesUsed - 1;
//                     next = j + 1;
//                 }
//                 else
//                 {
//                     previous = j - 1;
//                     next = 0;
//                 }
//                 xo = pStrokes[i]->pPoints[j].x;
//                 yo = pStrokes[i]->pPoints[j].y;
//                 xa = pStrokes[i]->pPoints[previous].x;
//                 ya = pStrokes[i]->pPoints[previous].y;
//                 xb = pStrokes[i]->pPoints[next].x;
//                 yb = pStrokes[i]->pPoints[next].y;
//                 theta = atan2(yo - ya, xo - xa);
//                 alpha = atan2(yb - yo, xb - xo) - theta;
//                 if (alpha > 0) alpha -= M_PI;
//                 else alpha += M_PI;
//                 if (_joint == PS_JOIN_MITER && eMiterLimit < fabs(1 / sin(alpha / 2)))
//                 {
//                     _joint = PS_JOIN_BEVEL;
//                 }
//                 if (alpha > 0)
//                 {
//                     pInsidePath = pUpPath;
//                     pOutsidePath = pDownPath;
//                 }
//                 else if (alpha < 0)
//                 {
//                     pInsidePath = pDownPath;
//                     pOutsidePath = pUpPath;
//                 }
//                 else
//                 {
//                     continue;
//                 }
//                 /* Inside angle points */
//                 if (alpha > 0)
//                 {
//                     pt.x = xo - round(penWidthIn * cos(theta + M_PI_2));
//                     pt.y = yo - round(penWidthIn * sin(theta + M_PI_2));
//                 }
//                 else
//                 {
//                     pt.x = xo + round(penWidthIn * cos(theta + M_PI_2));
//                     pt.y = yo + round(penWidthIn * sin(theta + M_PI_2));
//                 }
//                 PATH_AddEntry(pInsidePath, &pt, PT_LINETO);
//                 if (alpha > 0)
//                 {
//                     pt.x = xo + round(penWidthIn * cos(M_PI_2 + alpha + theta));
//                     pt.y = yo + round(penWidthIn * sin(M_PI_2 + alpha + theta));
//                 }
//                 else
//                 {
//                     pt.x = xo - round(penWidthIn * cos(M_PI_2 + alpha + theta));
//                     pt.y = yo - round(penWidthIn * sin(M_PI_2 + alpha + theta));
//                 }
//                 PATH_AddEntry(pInsidePath, &pt, PT_LINETO);
//                 /* Outside angle point */
//                 switch(_joint)
//                 {
//                     case PS_JOIN_MITER :
//                         miterWidth = fabs(penWidthOut / cos(M_PI_2 - fabs(alpha) / 2));
//                         pt.x = xo + round(miterWidth * cos(theta + alpha / 2));
//                         pt.y = yo + round(miterWidth * sin(theta + alpha / 2));
//                         PATH_AddEntry(pOutsidePath, &pt, PT_LINETO);
//                         break;
//                     case PS_JOIN_BEVEL :
//                         if (alpha > 0)
//                         {
//                             pt.x = xo + round(penWidthOut * cos(theta + M_PI_2));
//                             pt.y = yo + round(penWidthOut * sin(theta + M_PI_2));
//                         }
//                         else
//                         {
//                             pt.x = xo - round(penWidthOut * cos(theta + M_PI_2));
//                             pt.y = yo - round(penWidthOut * sin(theta + M_PI_2));
//                         }
//                         PATH_AddEntry(pOutsidePath, &pt, PT_LINETO);
//                         if (alpha > 0)
//                         {
//                             pt.x = xo - round(penWidthOut * cos(M_PI_2 + alpha + theta));
//                             pt.y = yo - round(penWidthOut * sin(M_PI_2 + alpha + theta));
//                         }
//                         else
//                         {
//                             pt.x = xo + round(penWidthOut * cos(M_PI_2 + alpha + theta));
//                             pt.y = yo + round(penWidthOut * sin(M_PI_2 + alpha + theta));
//                         }
//                         PATH_AddEntry(pOutsidePath, &pt, PT_LINETO);
//                         break;
//                     case PS_JOIN_ROUND :
//                     default :
//                         if (alpha > 0)
//                         {
//                             pt.x = xo + round(penWidthOut * cos(theta + M_PI_2));
//                             pt.y = yo + round(penWidthOut * sin(theta + M_PI_2));
//                         }
//                         else
//                         {
//                             pt.x = xo - round(penWidthOut * cos(theta + M_PI_2));
//                             pt.y = yo - round(penWidthOut * sin(theta + M_PI_2));
//                         }
//                         PATH_AddEntry(pOutsidePath, &pt, PT_BEZIERTO);
//                         pt.x = xo + round(penWidthOut * cos(theta + alpha / 2));
//                         pt.y = yo + round(penWidthOut * sin(theta + alpha / 2));
//                         PATH_AddEntry(pOutsidePath, &pt, PT_BEZIERTO);
//                         if (alpha > 0)
//                         {
//                             pt.x = xo - round(penWidthOut * cos(M_PI_2 + alpha + theta));
//                             pt.y = yo - round(penWidthOut * sin(M_PI_2 + alpha + theta));
//                         }
//                         else
//                         {
//                             pt.x = xo + round(penWidthOut * cos(M_PI_2 + alpha + theta));
//                             pt.y = yo + round(penWidthOut * sin(M_PI_2 + alpha + theta));
//                         }
//                         PATH_AddEntry(pOutsidePath, &pt, PT_BEZIERTO);
//                         break;
//                 }
//             }
//         }
//         type = add_points( pNewPath, pUpPath->pPoints, pUpPath->numEntriesUsed, PT_LINETO );
//         type[0] = PT_MOVETO;
//         reverse_points( pDownPath->pPoints, pDownPath->numEntriesUsed );
//         type = add_points( pNewPath, pDownPath->pPoints, pDownPath->numEntriesUsed, PT_LINETO );
//         if (pStrokes[i]->pFlags[pStrokes[i]->numEntriesUsed - 1] & PT_CLOSEFIGURE) type[0] = PT_MOVETO;
//
//         PATH_DestroyGdiPath(pStrokes[i]);
//         ExFreePoolWithTag(pStrokes[i], TAG_PATH);
//         PATH_DestroyGdiPath(pUpPath);
//         ExFreePoolWithTag(pUpPath, TAG_PATH);
//         PATH_DestroyGdiPath(pDownPath);
//         ExFreePoolWithTag(pDownPath, TAG_PATH);
//     }
//
//     pNewPath->state = PATH_Closed;
//     PATH_UnlockPath(pNewPath);
//
//     w32r_KeRestoreFloatingPointState(&fpsave);
//
// Exit:
//     if (pStrokes) ExFreePoolWithTag(pStrokes, TAG_PATH);
//     HPATH hpathToDelete = flat_path->BaseObject.hHmgr;
//     PATH_UnlockPath(flat_path);
//     PATH_Delete(hpathToDelete);
//
//     return pNewPath;
// }
// /* path.split3.c — #include fragment of path.c, carved for pretranslation so rustconv gets a small file. NOT a standalone TU; NOT in CMake; do not compile directly. The preprocessor reassembles the original path.c TU verbatim. See docs/engineering/win32k-pretranslation-audit.md */
//
//
// static
// PPATH
// FASTCALL
// PATH_WidenPath(DC *dc)
// {
//     PPATH pPath, pNewPath;
//
//     pPath = PATH_LockPath(dc->dclevel.hPath);
//     if (!pPath)
//     {
//         EngSetLastError( ERROR_CAN_NOT_COMPLETE );
//         return NULL;
//     }
//
//     pNewPath = PATH_WidenPathEx(dc, pPath);
//     PATH_UnlockPath(pPath);
//     return pNewPath;
// }
//
// PPATH
// FASTCALL
// PATH_WidenPathEx(DC *dc, PPATH pPath)
// {
//     INT size;
//     UINT penWidth, penStyle;
//     DWORD obj_type;
//     LPEXTLOGPEN elp;
//     PDC_ATTR pdcattr = dc->pdcattr;
//
//     if (pPath->state != PATH_Closed)
//     {
//         DbgPrint("PWP 1\n");
//         EngSetLastError(ERROR_CAN_NOT_COMPLETE);
//         return NULL;
//     }
//
//     size = GreGetObject(pdcattr->hpen, 0, NULL);
//     if (!size)
//     {
//         DbgPrint("PWP 2\n");
//         EngSetLastError(ERROR_CAN_NOT_COMPLETE);
//         return NULL;
//     }
//
//     elp = ExAllocatePoolWithTag(PagedPool, size, TAG_PATH);
//     if (elp == NULL)
//     {
//         DbgPrint("PWP 3\n");
//         EngSetLastError(ERROR_OUTOFMEMORY);
//         return NULL;
//     }
//
//     GreGetObject(pdcattr->hpen, size, elp);
//
//     obj_type = w32r_GDI_HANDLE_GET_TYPE(pdcattr->hpen);
//     if (obj_type == GDI_OBJECT_TYPE_PEN)
//     {
//         penStyle = ((LOGPEN*)elp)->lopnStyle;
//     }
//     else if (obj_type == GDI_OBJECT_TYPE_EXTPEN)
//     {
//         penStyle = elp->elpPenStyle;
//     }
//     else
//     {
//         DbgPrint("PWP 4\n");
//         EngSetLastError(ERROR_CAN_NOT_COMPLETE);
//         ExFreePoolWithTag(elp, TAG_PATH);
//         return NULL;
//     }
//
//     penWidth = elp->elpWidth;
//     ExFreePoolWithTag(elp, TAG_PATH);
//
//     /* The function cannot apply to cosmetic pens */
//     if (obj_type == GDI_OBJECT_TYPE_EXTPEN &&
//         (PS_TYPE_MASK & penStyle) == PS_COSMETIC)
//     {
//         DbgPrint("PWP 5\n");
//         EngSetLastError(ERROR_CAN_NOT_COMPLETE);
//         return FALSE;
//     }
//
//     return IntGdiWidenPath(pPath, penWidth, penStyle, dc->dclevel.laPath.eMiterLimit);
// }
//
// static inline INT int_from_fixed(FIXED f)
// {
//     return (f.fract >= 0x8000) ? (f.value + 1) : f.value;
// }
//
// /**********************************************************************
//  *      PATH_BezierTo
//  *
//  * Internally used by PATH_add_outline
//  */
// static
// VOID
// FASTCALL
// PATH_BezierTo(
//     PPATH pPath,
//     POINT *lppt,
//     INT n)
// {
//     if (n < 2) return;
//
//     if (n == 2)
//     {
//         PATH_AddEntry(pPath, &lppt[1], PT_LINETO);
//     }
//     else if (n == 3)
//     {
//         add_points( pPath, lppt, 3, PT_BEZIERTO );
//     }
//     else
//     {
//         POINT pt[3];
//         INT i = 0;
//
//         pt[2] = lppt[0];
//         n--;
//
//         while (n > 2)
//         {
//             pt[0] = pt[2];
//             pt[1] = lppt[i + 1];
//             pt[2].x = (lppt[i + 2].x + lppt[i + 1].x) / 2;
//             pt[2].y = (lppt[i + 2].y + lppt[i + 1].y) / 2;
//             add_points( pPath, pt, 3, PT_BEZIERTO );
//             n--;
//             i++;
//         }
//
//         pt[0] = pt[2];
//         pt[1] = lppt[i + 1];
//         pt[2] = lppt[i + 2];
//         add_points( pPath, pt, 3, PT_BEZIERTO );
//     }
// }
//
// static
// BOOL
// FASTCALL
// PATH_add_outline(
//     PDC dc,
//     PPATH pPath,
//     INT x,
//     INT y,
//     TTPOLYGONHEADER *header,
//     DWORD size)
// {
//     TTPOLYGONHEADER *start;
//     POINT pt;
//     BOOL bResult = FALSE;
//
//     start = header;
//
//     while ((char *)header < (char *)start + size)
//     {
//         TTPOLYCURVE *curve;
//
//         if (header->dwType != TT_POLYGON_TYPE)
//         {
//             ERR("Unknown header type %lu\n", header->dwType);
//             goto cleanup;
//         }
//
//         pt.x = x + int_from_fixed(header->pfxStart.x);
//         pt.y = y - int_from_fixed(header->pfxStart.y);
//         PATH_AddEntry(pPath, &pt, PT_MOVETO);
//
//         curve = (TTPOLYCURVE *)(header + 1);
//
//         while ((char *)curve < (char *)header + header->cb)
//         {
//             DbgPrint("curve->wType %d\n", curve->wType);
//
//             switch(curve->wType)
//             {
//                 case TT_PRIM_LINE:
//                 {
//                     WORD i;
//
//                     for (i = 0; i < curve->cpfx; i++)
//                     {
//                         pt.x = x + int_from_fixed(curve->apfx[i].x);
//                         pt.y = y - int_from_fixed(curve->apfx[i].y);
//                         PATH_AddEntry(pPath, &pt, PT_LINETO);
//                     }
//                     break;
//                 }
//
//                 case TT_PRIM_QSPLINE:
//                 case TT_PRIM_CSPLINE:
//                 {
//                     WORD i;
//                     POINTFX ptfx;
//                     POINT *pts = ExAllocatePoolWithTag(PagedPool, (curve->cpfx + 1) * sizeof(POINT), TAG_PATH);
//
//                     if (!pts) goto cleanup;
//
//                     ptfx = *(POINTFX *)((char *)curve - sizeof(POINTFX));
//
//                     pts[0].x = x + int_from_fixed(ptfx.x);
//                     pts[0].y = y - int_from_fixed(ptfx.y);
//
//                     for (i = 0; i < curve->cpfx; i++)
//                     {
//                         pts[i + 1].x = x + int_from_fixed(curve->apfx[i].x);
//                         pts[i + 1].y = y - int_from_fixed(curve->apfx[i].y);
//                     }
//
//                     PATH_BezierTo(pPath, pts, curve->cpfx + 1);
//
//                     ExFreePoolWithTag(pts, TAG_PATH);
//                     break;
//                 }
//
//                 default:
//                     ERR("Unknown curve type %04x\n", curve->wType);
//                     goto cleanup;
//             }
//
//             curve = (TTPOLYCURVE *)&curve->apfx[curve->cpfx];
//         }
//         header = (TTPOLYGONHEADER *)((char *)header + header->cb);
//     }
//
//     bResult = TRUE;
//
// cleanup:
//     IntGdiCloseFigure(pPath);
//     return bResult;
// }
//
// /**********************************************************************
//  *      PATH_ExtTextOut
//  */
// BOOL
// FASTCALL
// PATH_ExtTextOut(
//     PDC dc,
//     INT x,
//     INT y,
//     UINT flags,
//     const RECTL *lprc,
//     LPCWSTR str,
//     UINT count,
//     const INT *dx)
// {
//     PPATH pPath;
//     unsigned int idx, ggo_flags = GGO_NATIVE;
//     POINT offset = {0, 0};
//
//     pPath = PATH_LockPath(dc->dclevel.hPath);
//     if (!pPath)
//     {
//         return FALSE;
//     }
//
//     if (pPath->state != PATH_Open)
//     {
//         ERR("PATH_ExtTextOut not open\n");
//         return FALSE;
//     }
//
//     if (!count) return TRUE;
//     if (flags & ETO_GLYPH_INDEX) ggo_flags |= GGO_GLYPH_INDEX;
//
//     for (idx = 0; idx < count; idx++)
//     {
//         MAT2 identity = { {0, 1}, {0, 0}, {0, 0}, {0, 1} };
//         GLYPHMETRICS gm;
//         DWORD dwSize;
//         void *outline;
//
//         dwSize = w32r_ftGdiGetGlyphOutline(dc,
//                                       str[idx],
//                                       ggo_flags,
//                                       &gm,
//                                       0,
//                                       NULL,
//                                       &identity,
//                                       TRUE);
//         if (dwSize == GDI_ERROR)
//         {
//            // With default DC font,,, bitmap font?
//            // ExtTextOut on a path with bitmap font selected shouldn't fail.
//            // This just leads to empty path generated.
//            // Ref : test_emf_ExtTextOut_on_path
//            continue;
//         }
//
//         /* Add outline only if char is printable */
//         if (dwSize)
//         {
//             outline = ExAllocatePoolWithTag(PagedPool, dwSize, TAG_PATH);
//             if (!outline)
//             {
//                PATH_UnlockPath(pPath);
//                return FALSE;
//             }
//
//             w32r_ftGdiGetGlyphOutline(dc,
//                                  str[idx],
//                                  ggo_flags,
//                                  &gm,
//                                  dwSize,
//                                  outline,
//                                  &identity,
//                                  TRUE);
//
//             PATH_add_outline(dc, pPath, x + offset.x, y + offset.y, outline, dwSize);
//
//             ExFreePoolWithTag(outline, TAG_PATH);
//         }
//
//         if (dx)
//         {
//             if (flags & ETO_PDY)
//             {
//                 offset.x += dx[idx * 2];
//                 offset.y += dx[idx * 2 + 1];
//             }
//             else
//                 offset.x += dx[idx];
//         }
//         else
//         {
//             offset.x += gm.gmCellIncX;
//             offset.y += gm.gmCellIncY;
//         }
//     }
//     PATH_UnlockPath(pPath);
//     return TRUE;
// }
//
//
// /***********************************************************************
//  * Exported functions
//  */
//
// BOOL
// __stdcall
// NtGdiAbortPath(HDC hDC)
// {
//     PDC dc = DC_LockDc(hDC);
//     if (!dc)
//     {
//         EngSetLastError(ERROR_INVALID_HANDLE);
//         return FALSE;
//     }
//
//     if (!dc->dclevel.hPath)
//     {
//        DC_UnlockDc(dc);
//        return TRUE;
//     }
//
//     if (!PATH_Delete(dc->dclevel.hPath))
//     {
//        DC_UnlockDc(dc);
//        return FALSE;
//     }
//
//     dc->dclevel.hPath = 0;
//     dc->dclevel.flPath &= ~DCPATH_ACTIVE;
//
//     DC_UnlockDc(dc);
//     return TRUE;
// }
//
// BOOL
// __stdcall
// NtGdiBeginPath(HDC  hDC)
// {
//     PPATH pPath;
//     PDC dc;
//
//     dc = DC_LockDc(hDC);
//     if (!dc)
//     {
//         EngSetLastError(ERROR_INVALID_HANDLE);
//         return FALSE;
//     }
//
//     /* If path is already open, do nothing. Check if not Save DC state */
//     if ((dc->dclevel.flPath & DCPATH_ACTIVE) && !(dc->dclevel.flPath & DCPATH_SAVE))
//     {
//         DC_UnlockDc(dc);
//         return TRUE;
//     }
//
//     if (dc->dclevel.hPath)
//     {
//         DbgPrint("BeginPath 1 0x%p\n", dc->dclevel.hPath);
//         if (!(dc->dclevel.flPath & DCPATH_SAVE))
//         {
//             // Remove previous handle.
//             if (!PATH_Delete(dc->dclevel.hPath))
//             {
//                 DC_UnlockDc(dc);
//                 return FALSE;
//             }
//         }
//         else
//         {
//             // Clear flags and Handle.
//             dc->dclevel.flPath &= ~(DCPATH_SAVE | DCPATH_ACTIVE);
//             dc->dclevel.hPath = NULL;
//         }
//     }
//     pPath = PATH_CreatePath(NUM_ENTRIES_INITIAL);
//     dc->dclevel.flPath |= DCPATH_ACTIVE; // Set active ASAP!
//     dc->dclevel.hPath = pPath->BaseObject.hHmgr;
//     IntGetCurrentPositionEx(dc, &pPath->pos);
//     w32r_IntLPtoDP( dc, &pPath->pos, 1 );
//     DbgPrint("BP : Current pos X %d Y %d\n",pPath->pos.x, pPath->pos.y);
//     PATH_UnlockPath(pPath);
//     DC_UnlockDc(dc);
//
//     if (!pPath)
//     {
//         return FALSE;
//     }
//     return TRUE;
// }
//
// BOOL
// __stdcall
// NtGdiCloseFigure(HDC hDC)
// {
//     BOOL Ret = FALSE; // Default to failure
//     PDC pDc;
//     PPATH pPath;
//
//     DbgPrint("Enter %s\n", __FUNCTION__);
//
//     pDc = DC_LockDc(hDC);
//     if (!pDc)
//     {
//         EngSetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     pPath = PATH_LockPath(pDc->dclevel.hPath);
//     if (!pPath)
//     {
//         DC_UnlockDc(pDc);
//         return FALSE;
//     }
//
//     if (pPath->state == PATH_Open)
//     {
//         IntGdiCloseFigure(pPath);
//         Ret = TRUE;
//     }
//     else
//     {
//         EngSetLastError(ERROR_CAN_NOT_COMPLETE);
//     }
//
//     PATH_UnlockPath(pPath);
//     DC_UnlockDc(pDc);
//     return Ret;
// }
//
// BOOL
// __stdcall
// NtGdiEndPath(HDC  hDC)
// {
//     BOOL ret = TRUE;
//     PPATH pPath;
//     PDC dc;
//
//     dc = DC_LockDc(hDC);
//     if (!dc)
//     {
//         EngSetLastError(ERROR_INVALID_HANDLE);
//         return FALSE;
//     }
//
//     pPath = PATH_LockPath(dc->dclevel.hPath);
//     if (!pPath)
//     {
//         DC_UnlockDc(dc);
//         return FALSE;
//     }
//
//     /* Check that path is currently being constructed */
//     if ((pPath->state != PATH_Open) || !(dc->dclevel.flPath & DCPATH_ACTIVE))
//     {
//         DbgPrint("EndPath ERROR! 0x%p\n", dc->dclevel.hPath);
//         EngSetLastError(ERROR_CAN_NOT_COMPLETE);
//         ret = FALSE;
//     }
//     /* Set flag to indicate that path is finished */
//     else
//     {
//         DbgPrint("EndPath 0x%p\n", dc->dclevel.hPath);
//         pPath->state = PATH_Closed;
//         dc->dclevel.flPath &= ~DCPATH_ACTIVE;
//     }
//
//     PATH_UnlockPath(pPath);
//     DC_UnlockDc(dc);
//     return ret;
// }
//
// BOOL
// __stdcall
// NtGdiFillPath(HDC  hDC)
// {
//     BOOL ret = FALSE;
//     PPATH pPath, pNewPath;
//     PDC_ATTR pdcattr;
//     PDC dc;
//
//     dc = DC_LockDc(hDC);
//     if (!dc)
//     {
//         EngSetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     pPath = PATH_LockPath(dc->dclevel.hPath);
//     if (!pPath)
//     {
//         DC_UnlockDc(dc);
//         return FALSE;
//     }
//
//     DC_vPrepareDCsForBlit(dc, NULL, NULL, NULL);
//
//     pdcattr = dc->pdcattr;
//
//     if (pdcattr->ulDirty_ & (DIRTY_LINE | DC_PEN_DIRTY))
//         DC_vUpdateLineBrush(dc);
//
//     if (pdcattr->ulDirty_ & (DIRTY_FILL | DC_BRUSH_DIRTY))
//         DC_vUpdateFillBrush(dc);
//
//     pNewPath = PATH_FlattenPath(pPath);
//
//     if (pNewPath->state != PATH_Closed)
//     {
//         EngSetLastError(ERROR_CAN_NOT_COMPLETE);
//     }
//     else if (pNewPath->numEntriesUsed)
//     {
//        ret = PATH_FillPath(dc, pNewPath);
//     }
//     else ret = TRUE;
//
//     PATH_UnlockPath(pNewPath);
//     PATH_Delete(pNewPath->BaseObject.hHmgr);
//
//     PATH_UnlockPath(pPath);
//     PATH_Delete(pPath->BaseObject.hHmgr);
//     dc->dclevel.hPath = 0;
//     dc->dclevel.flPath &= ~DCPATH_ACTIVE;
//
//     DC_vFinishBlit(dc, NULL);
//     DC_UnlockDc(dc);
//     return ret;
// }
//
// BOOL
// __stdcall
// NtGdiFlattenPath(HDC hDC)
// {
//     BOOL Ret = FALSE;
//     DC *pDc;
//     PPATH pPath, pNewPath = NULL;
//
//     DbgPrint("Enter %s\n", __FUNCTION__);
//
//     pDc = DC_LockDc(hDC);
//     if (!pDc)
//     {
//         EngSetLastError(ERROR_INVALID_HANDLE);
//         return FALSE;
//     }
//
//     pPath = PATH_LockPath(pDc->dclevel.hPath);
//     if (!pPath)
//     {
//         EngSetLastError( ERROR_CAN_NOT_COMPLETE );
//         DC_UnlockDc(pDc);
//         return FALSE;
//     }
//
//     if (pPath->state == PATH_Closed)
//     {
//         pNewPath = PATH_FlattenPath(pPath);
//     }
//
//     PATH_UnlockPath(pPath);
//
//     if (pNewPath)
//     {
//        PATH_Delete(pDc->dclevel.hPath);
//        pDc->dclevel.hPath = pNewPath->BaseObject.hHmgr;
//        PATH_UnlockPath(pNewPath);
//        Ret = TRUE;
//     }
//
//     DC_UnlockDc(pDc);
//     return Ret;
// }
//
//
// BOOL
// __stdcall
// NtGdiGetMiterLimit(
//      HDC hdc,
//      PDWORD pdwOut)
// {
//     DC *pDc;
//     BOOL bResult = TRUE;
//     NTSTATUS Status;
//
//     if (!(pDc = DC_LockDc(hdc)))
//     {
//         EngSetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     Status = w32r_SafeProbeForWrite(pdwOut, sizeof(DWORD), 1);
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastNtError(Status);
//         bResult = FALSE;
//     }
//     else
//     {
//         *pdwOut = pDc->dclevel.laPath.eMiterLimit;
//     }
//
//     DC_UnlockDc(pDc);
//     return bResult;
//
// }
//
//
// INT
// __stdcall
// NtGdiGetPath(
//     HDC hDC,
//     LPPOINT Points,
//     LPBYTE Types,
//     INT nSize)
// {
//     INT ret = -1;
//     PPATH pPath;
//     DC *dc;
//     NTSTATUS Status;
//
//     Status = w32r_SafeProbeForWrite(Points, nSize * sizeof(*Points), sizeof(ULONG));
//     if (NT_SUCCESS(Status)) Status = w32r_SafeProbeForWrite(Types, nSize, 1);
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastNtError(Status);
//         return -1;
//     }
//
//     dc = DC_LockDc(hDC);
//     DbgPrint("NtGdiGetPath start\n");
//     if (!dc)
//     {
//         ERR("Can't lock dc!\n");
//         EngSetLastError(ERROR_INVALID_PARAMETER);
//         return -1;
//     }
//
//     pPath = PATH_LockPath(dc->dclevel.hPath);
//     if (!pPath)
//     {
//         DC_UnlockDc(dc);
//         return -1;
//     }
//
//     if (pPath->state != PATH_Closed)
//     {
//         EngSetLastError(ERROR_CAN_NOT_COMPLETE);
//         goto done;
//     }
//
//     if (nSize == 0)
//     {
//         ret = pPath->numEntriesUsed;
//     }
//     else if (nSize < pPath->numEntriesUsed)
//     {
//         EngSetLastError(ERROR_INVALID_PARAMETER);
//         goto done;
//     }
//     else
//     {
//         _SEH2_TRY
//         {
//             memcpy(Points, pPath->pPoints, sizeof(POINT)*pPath->numEntriesUsed);
//             memcpy(Types, pPath->pFlags, sizeof(BYTE)*pPath->numEntriesUsed);
//
//             /* Convert the points to logical coordinates */
//             if (!GdiPathDPtoLP(dc, Points, pPath->numEntriesUsed))
//             {
//                 EngSetLastError(ERROR_ARITHMETIC_OVERFLOW);
//                 _SEH2_LEAVE;
//             }
//
//             ret = pPath->numEntriesUsed;
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             SetLastNtError(_SEH2_GetExceptionCode());
//         }
//         _SEH2_END
//     }
//
// done:
//     DbgPrint("NtGdiGetPath exit %d\n",ret);
//     PATH_UnlockPath(pPath);
//     DC_UnlockDc(dc);
//     return ret;
// }
//
//
// HRGN
// __stdcall
// NtGdiPathToRegion(HDC  hDC)
// {
//     PPATH pPath, pNewPath;
//     HRGN  hrgnRval = 0;
//     int Ret;
//     PREGION Rgn;
//     DC *pDc;
//     PDC_ATTR pdcattr;
//
//     DbgPrint("Enter %s\n", __FUNCTION__);
//
//     pDc = DC_LockDc(hDC);
//     if (!pDc)
//     {
//         ERR("Failed to lock DC %p\n", hDC);
//         EngSetLastError(ERROR_INVALID_PARAMETER);
//         return NULL;
//     }
//
//     pdcattr = pDc->pdcattr;
//
//     pPath = PATH_LockPath(pDc->dclevel.hPath);
//     if (!pPath)
//     {
//         ERR("Failed to lock DC path %p\n", pDc->dclevel.hPath);
//         DC_UnlockDc(pDc);
//         return NULL;
//     }
//
//     if (pPath->state != PATH_Closed)
//     {
//         // FIXME: Check that setlasterror is being called correctly
//         ERR("Path is not closed!\n");
//         EngSetLastError(ERROR_CAN_NOT_COMPLETE);
//     }
//     else
//     {
//         /* Create the region and fill it with the path strokes */
//         Rgn = REGION_AllocUserRgnWithHandle(1);
//         if (!Rgn)
//         {
//             ERR("Failed to allocate a region\n");
//             PATH_UnlockPath(pPath);
//             DC_UnlockDc(pDc);
//             return NULL;
//         }
//         hrgnRval = Rgn->BaseObject.hHmgr;
//
//         pNewPath = PATH_FlattenPath(pPath);
//         if (pNewPath == NULL)
//         {
//             ERR("Failed to flatten path %p\n", pDc->dclevel.hPath);
//             REGION_Delete(Rgn);
//             PATH_UnlockPath(pPath);
//             DC_UnlockDc(pDc);
//             return NULL;
//         }
//
//         Ret = PATH_PathToRegion(pNewPath, pdcattr->jFillMode, Rgn);
//
//         PATH_UnlockPath(pNewPath);
//         PATH_Delete(pNewPath->BaseObject.hHmgr);
//
//         if (!Ret)
//         {
//             ERR("PATH_PathToRegion failed\n");
//             REGION_Delete(Rgn);
//             hrgnRval = NULL;
//         }
//         else
//             REGION_UnlockRgn(Rgn);
//     }
//
//     PATH_UnlockPath(pPath);
//     PATH_Delete(pDc->dclevel.hPath);
//     pDc->dclevel.hPath = NULL;
//     pDc->dclevel.flPath &= ~DCPATH_ACTIVE;
//
//     DC_UnlockDc(pDc);
//     return hrgnRval;
// }
//
// BOOL
// __stdcall
// NtGdiSetMiterLimit(
//     IN HDC hdc,
//     IN DWORD dwNew,
//     IN OUT OPTIONAL PDWORD pdwOut)
// {
//     DC *pDc;
//     gxf_long worker, worker1;
//     BOOL bResult = TRUE;
//     NTSTATUS Status;
//
//     if (!(pDc = DC_LockDc(hdc)))
//     {
//         EngSetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     worker.l  = dwNew;
//     worker1.f = pDc->dclevel.laPath.eMiterLimit;
//     pDc->dclevel.laPath.eMiterLimit = worker.f;
//
//     if (pdwOut)
//     {
//         Status = w32r_SafeProbeForWrite(pdwOut, sizeof(DWORD), 1);
//         if (!NT_SUCCESS(Status))
//         {
//             SetLastNtError(Status);
//             bResult = FALSE;
//         }
//         else
//         {
//             *pdwOut = worker1.l;
//         }
//     }
//
//     DC_UnlockDc(pDc);
//     return bResult;
// }
//
//
// BOOL
// __stdcall
// NtGdiStrokeAndFillPath(HDC hDC)
// {
//     DC *pDc;
//     PDC_ATTR pdcattr;
//     PPATH pPath, pNewPath;
//     BOOL bRet = FALSE;
//
//     DbgPrint("Enter %s\n", __FUNCTION__);
//
//     if (!(pDc = DC_LockDc(hDC)))
//     {
//         EngSetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//     pPath = PATH_LockPath(pDc->dclevel.hPath);
//     if (!pPath)
//     {
//         DC_UnlockDc(pDc);
//         return FALSE;
//     }
//
//     DC_vPrepareDCsForBlit(pDc, NULL, NULL, NULL);
//
//     pdcattr = pDc->pdcattr;
//
//     if (pdcattr->ulDirty_ & (DIRTY_FILL | DC_BRUSH_DIRTY))
//         DC_vUpdateFillBrush(pDc);
//
//     if (pdcattr->ulDirty_ & (DIRTY_LINE | DC_PEN_DIRTY))
//         DC_vUpdateLineBrush(pDc);
//
//     pNewPath = PATH_FlattenPath(pPath);
//
//     if (pNewPath->state != PATH_Closed)
//     {
//         EngSetLastError(ERROR_CAN_NOT_COMPLETE);
//     }
//     else if (pNewPath->numEntriesUsed)
//     {
//        bRet = PATH_FillPath(pDc, pNewPath);
//        if (bRet) bRet = PATH_StrokePath(pDc, pNewPath);
//     }
//     else bRet = TRUE;
//
//     PATH_UnlockPath(pNewPath);
//     PATH_Delete(pNewPath->BaseObject.hHmgr);
//
//     PATH_UnlockPath(pPath);
//     PATH_Delete(pPath->BaseObject.hHmgr);
//     pDc->dclevel.hPath = 0;
//     pDc->dclevel.flPath &= ~DCPATH_ACTIVE;
//
//     DC_vFinishBlit(pDc, NULL);
//     DC_UnlockDc(pDc);
//     return bRet;
// }
//
// BOOL
// __stdcall
// NtGdiStrokePath(HDC hDC)
// {
//     DC *pDc;
//     PDC_ATTR pdcattr;
//     PPATH pPath, pNewPath;
//     BOOL bRet = FALSE;
//
//     DbgPrint("Enter %s\n", __FUNCTION__);
//
//     if (!(pDc = DC_LockDc(hDC)))
//     {
//         EngSetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     pPath = PATH_LockPath(pDc->dclevel.hPath);
//     if (!pPath)
//     {
//         DC_UnlockDc(pDc);
//         return FALSE;
//     }
//
//     DC_vPrepareDCsForBlit(pDc, NULL, NULL, NULL);
//
//     pdcattr = pDc->pdcattr;
//
//     if (pdcattr->ulDirty_ & (DIRTY_LINE | DC_PEN_DIRTY))
//         DC_vUpdateLineBrush(pDc);
//
//     pNewPath = PATH_FlattenPath(pPath);
//
//     if (pNewPath->state != PATH_Closed)
//     {
//         EngSetLastError(ERROR_CAN_NOT_COMPLETE);
//     }
//     else bRet = PATH_StrokePath(pDc, pNewPath);
//
//     PATH_UnlockPath(pNewPath);
//     PATH_Delete(pNewPath->BaseObject.hHmgr);
//
//     DC_vFinishBlit(pDc, NULL);
//
//     PATH_UnlockPath(pPath);
//     PATH_Delete(pPath->BaseObject.hHmgr);
//     pDc->dclevel.hPath = 0;
//     pDc->dclevel.flPath &= ~DCPATH_ACTIVE;
//
//     DC_UnlockDc(pDc);
//     return bRet;
// }
//
// BOOL
// __stdcall
// NtGdiWidenPath(HDC  hDC)
// {
//     PPATH pPath;
//     BOOL Ret = FALSE;
//     PDC pdc = DC_LockDc(hDC);
//     DbgPrint("NtGdiWidenPat Enter\n");
//     if (!pdc)
//     {
//         EngSetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     pPath = PATH_WidenPath(pdc);
//     if (pPath)
//     {
//        DbgPrint("WindenPath New Path\n");
//        PATH_Delete(pdc->dclevel.hPath);
//        pdc->dclevel.hPath = pPath->BaseObject.hHmgr;
//        Ret = TRUE;
//     }
//     DC_UnlockDc(pdc);
//     DbgPrint("NtGdiWidenPat Ret %d\n",Ret);
//     return Ret;
// }
//
// /* EOF */
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================


// ABI-corrected extern decls — i386 calling-convention must match the
// definition; a flat extern "C" decl of a stdcall/fastcall fn is UB -> ud2.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn RECTL_vMakeWellOrdered(prcl: *mut RECTL);
    fn SetLastNtError(Status: NTSTATUS);
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn RECTL_vMakeWellOrdered(prcl: *mut RECTL);
    fn SetLastNtError(Status: NTSTATUS);
}