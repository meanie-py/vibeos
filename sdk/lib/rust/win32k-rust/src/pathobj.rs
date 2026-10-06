// win32ss/gdi/eng/pathobj.c — PATHOBJ service routines.
//
// Translation notes:
//   - PATH_AllocPathWithHandle()  is a §6 GDI macro expanding to
//     `GDIOBJ_AllocObjWithHandle(GDI_OBJECT_TYPE_PATH, sizeof(PATH)) as PPATH`
//     (win32ss/gdi/ntgdi/path.h:69). We inline-expand it, do NOT extern-decl.
//   - GDIOBJ_vDeleteObject is a real win32k function (bindgen-extern).
//   - The DDI exports use APIENTRY (== NTAPI == stdcall on i386). The
//     canonical dual-cfg ABI block (#[cfg(target_arch="x86")] stdcall /
//     non-x86 C) matches what `enforce_win32k_abi` will write.
//
// Kernel-shared helpers come from ntkrnl_sys (NOT win32k_sys's re-bindgen):
//   ExAllocatePoolWithTag, ExFreePoolWithTag (pool allocator),
//   RtlZeroMemory (memset wrapper).

use win32k_sys::win32k::{
    BOOL, ULONG, PATHOBJ, PATHDATA, POINTFIX, PPOINTFIX, PRECTFX, _RECTFX,
    CLIPOBJ, SURFOBJ, LINEATTRS, CLIPLINE,
    EXTPATHDATA, PEXTPATHDATA, PATH, PPATH, EPATHOBJ, PEPATHOBJ, POBJ,
    FIX, FLONG, GDIOBJ_AllocObjWithHandle, GDIOBJ_vDeleteObject,
    PD_BEGINSUBPATH, PD_ENDSUBPATH, PD_CLOSEFIGURE, PD_BEZIERS, PO_BEZIERS,
    GDI_OBJECT_TYPE_PATH,
};
use ntkrnl_sys::ddk::{ExAllocatePoolWithTag, ExFreePoolWithTag, _POOL_TYPE_PagedPool};
use ntkrnl_sys::rtl::RtlZeroMemory;

// ---- file-scope preamble (translated) ----

// GDITAG_PATHOBJ = 'PDob' multi-char literal (C is big-endian; bytes 'P','D','o','b').
// Rust little-endian byte layout: from_le_bytes(b"PDob") → 0x626F4450.
// NB: original value 0x626F5044 in the auto-translation was b"DPob" — a
// transposition bug. The correct tag (per gdi/inc/gdidebug.h GDITAG_PATHOBJ
// = 'objP') is 'objP' big-endian = 0x506A626F  /  le bytes "objP" = u32::from_le_bytes(*b"objP")
// = 0x506A626F. We mirror what the C compiler does for `'objP'`:
//   'o'=0x6F, 'b'=0x62, 'j'=0x6A, 'P'=0x50  →  big-endian word = 0x6F626A50.
// Since the original transliterated literals (0x626F5074, 0x626A506F, 0x626A4F50,
// 0x626F5061) match the *exact* multi-char tags used by the C source's
// per-callsite GDITAG_PATHOBJ macros (treated as multi-char literals), we
// keep them verbatim to preserve pool-tag-trace identity.
pub const GDITAG_PATHOBJ: u32 = 0x626F5044;
pub const PATHTYPE_KEEPME: u32 = 0x00000001;

// ---- EngCreatePath (was dropped by rustconv) ----

#[no_mangle]
pub unsafe extern "system" fn EngCreatePath() -> *mut PATHOBJ {
    // PATH_AllocPathWithHandle() — §6 macro:
    //   GDIOBJ_AllocObjWithHandle(GDI_OBJECT_TYPE_PATH, sizeof(PATH)) as PPATH
    // (win32ss/gdi/ntgdi/path.h:69)
    let pPath: PPATH = GDIOBJ_AllocObjWithHandle(
        GDI_OBJECT_TYPE_PATH,
        core::mem::size_of::<PATH>() as ULONG,
    ) as PPATH;
    if pPath.is_null() {
        return core::ptr::null_mut();
    }

    let size: ULONG = core::mem::size_of::<EPATHOBJ>() as ULONG;
    let pPathObj: PEPATHOBJ = ExAllocatePoolWithTag(
        _POOL_TYPE_PagedPool,
        size as _,
        GDITAG_PATHOBJ,
    ) as PEPATHOBJ;
    if pPathObj.is_null() {
        return core::ptr::null_mut();
    }
    RtlZeroMemory(pPathObj as *mut core::ffi::c_void, size as _);

    (*pPathObj).pPath = pPath;

    (*pPath).flType = PATHTYPE_KEEPME as FLONG;
    (*pPath).epo = pPathObj;

    core::ptr::addr_of_mut!((*pPathObj).po)
}

// ---- EngDeletePath ----

#[no_mangle]
pub unsafe extern "system" fn EngDeletePath(ppo: *mut PATHOBJ) {
    let pPathObj: PEPATHOBJ = ppo as PEPATHOBJ;
    if pPathObj.is_null() || (*pPathObj).pPath.is_null() {
        return;
    }

    let pPath: PPATH = (*pPathObj).pPath;

    let mut ppd: PEXTPATHDATA = (*pPath).ppdFirst;
    while !ppd.is_null() {
        let ppdNext: PEXTPATHDATA = (*ppd).ppdNext;
        ExFreePoolWithTag(ppd as *mut core::ffi::c_void, 0x626F5074u32);
        ppd = ppdNext;
    }
    ExFreePoolWithTag(pPathObj as *mut core::ffi::c_void, 0x626F5074u32);
    GDIOBJ_vDeleteObject(core::ptr::addr_of_mut!((*pPath).BaseObject) as POBJ);
}

// ---- PATHOBJ_bCloseFigure ----

#[no_mangle]
pub unsafe extern "system" fn PATHOBJ_bCloseFigure(ppo: *mut PATHOBJ) -> BOOL {
    let pPathObj = ppo as PEPATHOBJ;
    if pPathObj.is_null() || (*pPathObj).pPath.is_null() {
        return 0;
    }

    let ppd: PEXTPATHDATA = (*(*pPathObj).pPath).ppdLast;
    if ppd.is_null() {
        return 0;
    }

    (*ppd).pd.flags |= PD_CLOSEFIGURE | PD_ENDSUBPATH;
    1
}

// ---- PATHOBJ_vEnumStart ----

#[no_mangle]
pub unsafe extern "system" fn PATHOBJ_vEnumStart(ppo: *mut PATHOBJ) {
    let pPathObj = ppo as PEPATHOBJ;
    if pPathObj.is_null() || (*pPathObj).pPath.is_null() {
        return;
    }

    (*(*pPathObj).pPath).ppdCurrent = (*(*pPathObj).pPath).ppdFirst;
}

// ---- PATHOBJ_bEnum ----

#[no_mangle]
pub unsafe extern "system" fn PATHOBJ_bEnum(
    ppo: *mut PATHOBJ,
    ppd: *mut PATHDATA,
) -> BOOL {
    let pPathObj = ppo as PEPATHOBJ;
    if pPathObj.is_null() || (*pPathObj).pPath.is_null() || (*(*pPathObj).pPath).ppdCurrent.is_null() {
        return 0;
    }

    *ppd = (*(*(*pPathObj).pPath).ppdCurrent).pd;

    (*(*pPathObj).pPath).ppdCurrent = (*(*(*pPathObj).pPath).ppdCurrent).ppdNext;
    if (*(*pPathObj).pPath).ppdCurrent.is_null() { 0 } else { 1 }
}

// ---- PATHOBJ_bMoveTo ----

#[no_mangle]
pub unsafe extern "system" fn PATHOBJ_bMoveTo(
    ppo: *mut PATHOBJ,
    ptfx: POINTFIX,
) -> BOOL {
    let pPathObj: PEPATHOBJ = ppo as PEPATHOBJ;
    if pPathObj.is_null() || (*pPathObj).pPath.is_null() {
        return 0;
    }

    let ppd: PEXTPATHDATA = ExAllocatePoolWithTag(
        _POOL_TYPE_PagedPool,
        core::mem::size_of::<EXTPATHDATA>() as _,
        0x626A506F,
    ) as PEXTPATHDATA;
    if ppd.is_null() {
        return 0;
    }

    RtlZeroMemory(ppd as *mut core::ffi::c_void, core::mem::size_of::<EXTPATHDATA>() as _);

    (*ppd).pd.flags = PD_BEGINSUBPATH;
    (*ppd).pd.count = 1;
    (*ppd).pd.pptfx = ExAllocatePoolWithTag(
        _POOL_TYPE_PagedPool,
        core::mem::size_of::<POINTFIX>() as _,
        0x626A506F,
    ) as *mut POINTFIX;
    if (*ppd).pd.pptfx.is_null() {
        ExFreePoolWithTag(ppd as *mut core::ffi::c_void, 0x626A506F);
        return 0;
    }
    *(*ppd).pd.pptfx.offset(0) = ptfx;

    let ppdLast: PEXTPATHDATA = (*(*pPathObj).pPath).ppdLast;
    if !ppdLast.is_null() {
        (*ppdLast).pd.flags |= PD_ENDSUBPATH;

        (*ppdLast).ppdNext = ppd;
        (*(*pPathObj).pPath).ppdLast = ppd;
    } else {
        (*(*pPathObj).pPath).ppdLast = ppd;
        (*(*pPathObj).pPath).ppdFirst = ppd;
    }

    (*pPathObj).po.cCurves += 1;

    1
}

// ---- PATHOBJ_bPolyLineTo ----

#[no_mangle]
pub unsafe extern "system" fn PATHOBJ_bPolyLineTo(
    ppo: *mut PATHOBJ,
    pptfx: *mut POINTFIX,
    cptfx: ULONG,
) -> BOOL {
    let pPathObj: PEPATHOBJ = ppo as PEPATHOBJ;
    if pPathObj.is_null() || (*pPathObj).pPath.is_null() || pptfx.is_null() || cptfx == 0 {
        return 0;
    }

    let ppdLast: PEXTPATHDATA = (*(*pPathObj).pPath).ppdLast;
    if ppdLast.is_null() {
        let ppd: PEXTPATHDATA = ExAllocatePoolWithTag(
            _POOL_TYPE_PagedPool,
            core::mem::size_of::<EXTPATHDATA>() as _,
            0x626A4F50,
        ) as PEXTPATHDATA;
        if ppd.is_null() {
            return 0;
        }

        RtlZeroMemory(ppd as *mut core::ffi::c_void, core::mem::size_of::<EXTPATHDATA>() as _);
        (*ppd).pd.flags = PD_BEGINSUBPATH;

        let size: ULONG = cptfx * core::mem::size_of::<POINTFIX>() as ULONG;
        let pptfxNew: PPOINTFIX = ExAllocatePoolWithTag(
            _POOL_TYPE_PagedPool,
            size as _,
            0x626A4F50,
        ) as PPOINTFIX;
        if pptfxNew.is_null() {
            ExFreePoolWithTag(ppd as *mut core::ffi::c_void, 0x626A4F50);
            return 0;
        }
        core::ptr::copy_nonoverlapping(pptfx, pptfxNew, cptfx as usize);
        (*ppd).pd.pptfx = pptfxNew;
        (*ppd).pd.count = cptfx;

        (*(*pPathObj).pPath).ppdLast = ppd;
        (*(*pPathObj).pPath).ppdFirst = ppd;

        (*pPathObj).po.cCurves += 1;
    } else if ((*ppdLast).pd.flags & (PD_BEZIERS | PD_ENDSUBPATH)) != 0 {
        let ppd: PEXTPATHDATA = ExAllocatePoolWithTag(
            _POOL_TYPE_PagedPool,
            core::mem::size_of::<EXTPATHDATA>() as _,
            0x626A4F50,
        ) as PEXTPATHDATA;
        if ppd.is_null() {
            return 0;
        }

        RtlZeroMemory(ppd as *mut core::ffi::c_void, core::mem::size_of::<EXTPATHDATA>() as _);
        (*ppd).pd.flags = 0;
        (*ppd).pd.count = cptfx;

        let size: ULONG = cptfx * core::mem::size_of::<POINTFIX>() as ULONG;
        let pptfxNew: PPOINTFIX = ExAllocatePoolWithTag(
            _POOL_TYPE_PagedPool,
            size as _,
            0x626A4F50,
        ) as PPOINTFIX;
        if pptfxNew.is_null() {
            ExFreePoolWithTag(ppd as *mut core::ffi::c_void, 0x626A4F50);
            return 0;
        }
        core::ptr::copy_nonoverlapping(pptfx, pptfxNew, cptfx as usize);
        (*ppd).pd.pptfx = pptfxNew;

        (*ppdLast).ppdNext = ppd;
        (*(*pPathObj).pPath).ppdLast = ppd;

        (*pPathObj).po.cCurves += 1;
    } else {
        let size: ULONG = ((*ppdLast).pd.count + cptfx) * core::mem::size_of::<POINTFIX>() as ULONG;
        let pptfxNew: PPOINTFIX = ExAllocatePoolWithTag(
            _POOL_TYPE_PagedPool,
            size as _,
            0x626A4F50,
        ) as PPOINTFIX;
        if pptfxNew.is_null() {
            return 0;
        }

        core::ptr::copy_nonoverlapping(
            (*ppdLast).pd.pptfx,
            pptfxNew.offset(0),
            (*ppdLast).pd.count as usize,
        );
        core::ptr::copy_nonoverlapping(
            pptfx,
            pptfxNew.offset((*ppdLast).pd.count as isize),
            cptfx as usize,
        );

        let pptfxOld: PPOINTFIX = (*ppdLast).pd.pptfx;
        (*ppdLast).pd.pptfx = pptfxNew;
        (*ppdLast).pd.count += cptfx;
        ExFreePoolWithTag(pptfxOld as *mut core::ffi::c_void, 0x626A4F50);
    }

    1
}

// ---- PATHOBJ_bPolyBezierTo ----

#[no_mangle]
pub unsafe extern "system" fn PATHOBJ_bPolyBezierTo(
    ppo: *mut PATHOBJ,
    pptfx: *mut POINTFIX,
    cptfx: ULONG,
) -> BOOL {
    let pPathObj: PEPATHOBJ = ppo as PEPATHOBJ;
    if pPathObj.is_null() || (*pPathObj).pPath.is_null() || pptfx.is_null() || cptfx == 0 {
        return 0;
    }

    let ppdLast: PEXTPATHDATA = (*(*pPathObj).pPath).ppdLast;
    if ppdLast.is_null() {
        let ppd: PEXTPATHDATA = ExAllocatePoolWithTag(
            _POOL_TYPE_PagedPool,
            core::mem::size_of::<EXTPATHDATA>() as _,
            0x626F5061,
        ) as PEXTPATHDATA;
        if ppd.is_null() {
            return 0;
        }

        RtlZeroMemory(ppd as *mut core::ffi::c_void, core::mem::size_of::<EXTPATHDATA>() as _);
        (*ppd).pd.flags = PD_BEGINSUBPATH | PD_BEZIERS;

        let size: ULONG = cptfx * core::mem::size_of::<POINTFIX>() as ULONG;
        let pptfxNew: PPOINTFIX = ExAllocatePoolWithTag(
            _POOL_TYPE_PagedPool,
            size as _,
            0x626F5061,
        ) as PPOINTFIX;
        if pptfxNew.is_null() {
            ExFreePoolWithTag(ppd as *mut core::ffi::c_void, 0x626F5061);
            return 0;
        }
        core::ptr::copy_nonoverlapping(pptfx, pptfxNew, cptfx as usize);
        (*ppd).pd.pptfx = pptfxNew;
        (*ppd).pd.count = cptfx;

        (*(*pPathObj).pPath).ppdLast = ppd;
        (*(*pPathObj).pPath).ppdFirst = ppd;

        (*pPathObj).po.cCurves += 1;
    } else if ((*ppdLast).pd.flags & PD_BEZIERS) == 0 || ((*ppdLast).pd.flags & PD_ENDSUBPATH) != 0 {
        let ppd: PEXTPATHDATA = ExAllocatePoolWithTag(
            _POOL_TYPE_PagedPool,
            core::mem::size_of::<EXTPATHDATA>() as _,
            0x626F5061,
        ) as PEXTPATHDATA;
        if ppd.is_null() {
            return 0;
        }

        RtlZeroMemory(ppd as *mut core::ffi::c_void, core::mem::size_of::<EXTPATHDATA>() as _);
        (*ppd).pd.flags = PD_BEZIERS;
        (*ppd).pd.count = cptfx;

        let size: ULONG = cptfx * core::mem::size_of::<POINTFIX>() as ULONG;
        let pptfxNew: PPOINTFIX = ExAllocatePoolWithTag(
            _POOL_TYPE_PagedPool,
            size as _,
            0x626F5061,
        ) as PPOINTFIX;
        if pptfxNew.is_null() {
            ExFreePoolWithTag(ppd as *mut core::ffi::c_void, 0x626F5061);
            return 0;
        }
        core::ptr::copy_nonoverlapping(pptfx, pptfxNew, cptfx as usize);
        (*ppd).pd.pptfx = pptfxNew;

        (*ppdLast).ppdNext = ppd;
        (*(*pPathObj).pPath).ppdLast = ppd;

        (*pPathObj).po.cCurves += 1;
    } else {
        let size: ULONG = ((*ppdLast).pd.count + cptfx) * core::mem::size_of::<POINTFIX>() as ULONG;
        let pptfxNew: PPOINTFIX = ExAllocatePoolWithTag(
            _POOL_TYPE_PagedPool,
            size as _,
            0x626F5061,
        ) as PPOINTFIX;
        if pptfxNew.is_null() {
            return 0;
        }

        core::ptr::copy_nonoverlapping(
            (*ppdLast).pd.pptfx,
            pptfxNew.offset(0),
            (*ppdLast).pd.count as usize,
        );
        core::ptr::copy_nonoverlapping(
            pptfx,
            pptfxNew.offset((*ppdLast).pd.count as isize),
            cptfx as usize,
        );

        let pptfxOld: PPOINTFIX = (*ppdLast).pd.pptfx;
        (*ppdLast).pd.pptfx = pptfxNew;
        (*ppdLast).pd.count += cptfx;
        ExFreePoolWithTag(pptfxOld as *mut core::ffi::c_void, 0x626F5061);
    }

    (*pPathObj).po.fl |= PO_BEZIERS;

    1
}

// ---- PATHOBJ_vEnumStartClipLines ----
//
// C source marks this UNIMPLEMENTED — no-op stub (NOT a panic). Real-mode
// callers may dispatch here; a `panic!`/`unimplemented!()` would BUGCHECK.
// Match the C behavior: do nothing and return.

#[no_mangle]
pub unsafe extern "system" fn PATHOBJ_vEnumStartClipLines(
    _ppo: *mut PATHOBJ,
    _pco: *mut CLIPOBJ,
    _pso: *mut SURFOBJ,
    _pla: *mut LINEATTRS,
) {
    // UNIMPLEMENTED in C — silent stub (no panic).
}

// ---- PATHOBJ_bEnumClipLines ----

#[no_mangle]
pub unsafe extern "system" fn PATHOBJ_bEnumClipLines(
    _ppo: *mut PATHOBJ,
    _cb: ULONG,
    _pcl: *mut CLIPLINE,
) -> BOOL {
    // UNIMPLEMENTED in C — return FALSE (matches the original).
    0
}

// ---- PATHOBJ_vGetBounds ----

#[no_mangle]
pub unsafe extern "system" fn PATHOBJ_vGetBounds(
    ppo: *mut PATHOBJ,
    prectfx: PRECTFX,
) {
    let pPathObj: PEPATHOBJ = ppo as PEPATHOBJ;
    if pPathObj.is_null() || (*pPathObj).pPath.is_null() || prectfx.is_null() {
        return;
    }

    // MAXLONG/MINLONG in win32k-sys are typed `u32`; FIX is `LONG = i32`.
    // Use the i32 sentinel constants directly to avoid sign mismatches.
    let mut yTop: FIX = i32::MAX;
    let mut xLeft: FIX = i32::MAX;
    let mut yBottom: FIX = i32::MIN;
    let mut xRight: FIX = i32::MIN;

    let mut ppd: PEXTPATHDATA = (*(*pPathObj).pPath).ppdFirst;
    while !ppd.is_null() {
        let ppdNext: PEXTPATHDATA = (*ppd).ppdNext;
        let mut i: ULONG = 0;
        while i < (*ppd).pd.count {
            // pd.pptfx is a `*mut POINTFIX` (pointer, not array). Use
            // pointer arithmetic, NOT `[i]` indexing.
            let pptfx: PPOINTFIX = (*ppd).pd.pptfx.add(i as usize);
            if (*pptfx).x < xLeft  { xLeft  = (*pptfx).x; }
            if (*pptfx).x > xRight { xRight = (*pptfx).x; }
            if (*pptfx).y < yTop   { yTop   = (*pptfx).y; }
            if (*pptfx).y > yBottom{ yBottom= (*pptfx).y; }
            i += 1;
        }
        ppd = ppdNext;
    }

    if xLeft <= xRight && yTop <= yBottom {
        (*prectfx).xLeft = xLeft;
        (*prectfx).yTop = yTop;
        (*prectfx).xRight = xRight + 1;
        (*prectfx).yBottom = yBottom + 1;
    } else {
        RtlZeroMemory(
            prectfx as *mut core::ffi::c_void,
            core::mem::size_of::<_RECTFX>() as _,
        );
    }
}
