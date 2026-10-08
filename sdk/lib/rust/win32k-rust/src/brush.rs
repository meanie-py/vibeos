//! Rust translation of `win32ss/gdi/ntgdi/brush.cpp` — BRUSH class.
//! [Phase 2c wave 23: ntgdi brush object — the last "real" ntgdi C file]
//!
//! The C++ `class BRUSH : public BASEOBJECT, protected _BRUSHBODY` has no
//! vtable — it is "C with classes". Every method is a non-virtual wrapper over
//! a C-ABI `GDIOBJ_*` helper (all already Rust, in gdiobj.rs), and the object
//! layout is exactly the bindgen `_BRUSH` struct (`BASEOBJECT` header followed
//! by the `_BRUSHBODY`). So `class BRUSH` becomes `*mut _BRUSH`, the base-object
//! fields are reached via `pbr as POBJ`, and the body fields via the shared
//! `brushbody::pbrushbody()` view. The C++ methods are file-internal; only the
//! `extern "C"` API (which other modules call) gets `#[no_mangle]`.

use win32k_sys::win32k::*;
use ntkrnl_sys::ddk::{ExAllocatePoolWithTag, ExFreePool, ExFreePoolWithTag};
use crate::brushbody::{pbrushbody, BrushBodyView};
use core::sync::atomic::{AtomicU32, Ordering};

// Pool tags (tags.h; C multi-char literals are big-endian).
const GDITAG_HMGR_BRUSH_TYPE: u32 = u32::from_be_bytes(*b"@?hG");
const GDITAG_PENSTYLE: u32 = u32::from_be_bytes(*b"ytsG");
const GDITAG_TEMP: u32 = u32::from_be_bytes(*b"pmtG");

// GDIOBJ / Gre / pool helpers — all NTAPI (stdcall i386 / cdecl else), provided
// as #[no_mangle] by sibling Rust modules (gdiobj.rs, bitmaps.rs, dibobj.rs).
extern "system" {
    fn GDIOBJ_ShareLockObj(hObj: HGDIOBJ, ExpectedType: u32) -> PGDIOBJ;
    fn GDIOBJ_hInsertObject(pobj: POBJ, ulOwner: ULONG) -> HGDIOBJ;
    fn GDIOBJ_vSetObjectAttr(pobj: POBJ, pvObjAttr: PVOID);
    fn GDIOBJ_vUnlockObject(pobj: POBJ);
    fn GDIOBJ_vDereferenceObject(pobj: POBJ);
    fn GDIOBJ_ConvertToStockObj(phObj: *mut HGDIOBJ) -> BOOL;
    fn GDIOBJ_ConvertFromStockObj(phObj: *mut HGDIOBJ) -> BOOL;
    fn GdiPoolAllocate(pPool: PGDI_POOL) -> PVOID;
    fn GdiPoolFree(pPool: PGDI_POOL, pvAlloc: PVOID);
    fn GreIsHandleValid(hobj: HGDIOBJ) -> BOOL;
    fn GreDeleteObject(hobj: HGDIOBJ) -> BOOL;
    fn GreSetBitmapOwner(hbmp: HBITMAP, ulOwner: ULONG) -> BOOL;
    fn GreCreateDIBitmapFromPackedDIB(pvPackedDIB: PVOID, cjPackedDIB: UINT, uUsage: ULONG) -> HBITMAP;
    fn EngSetLastError(iError: ULONG);
}

// GetBrushAttrPool is the cdecl `extern "C" PGDI_POOL GetBrushAttrPool(VOID)`
// HACK declared in brush.hpp and defined elsewhere.
extern "C" {
    fn GetBrushAttrPool() -> PGDI_POOL;
}

// BITMAP_CopyBitmap = FASTCALL on i386, cdecl elsewhere.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn BITMAP_CopyBitmap(hBitmap: HBITMAP) -> HBITMAP;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn BITMAP_CopyBitmap(hBitmap: HBITMAP) -> HBITMAP;
}

/// `static ULONG ulGlobalBrushUnique` — function-local static in the C++ ctor.
static GLOBAL_BRUSH_UNIQUE: AtomicU32 = AtomicU32::new(0);

/* ---- BASEOBJECT inline methods (baseobj.hpp), reached via `pbr as POBJ` ---- */

#[inline]
unsafe fn base_object_init(pbr: PBRUSH, loobjtype: GDILoObjType) {
    let po = pbr as POBJ;
    (*po).hHmgr = (loobjtype as ULONG_PTR) as HGDIOBJ;
    (*po).cExclusiveLock = 0;
    (*po).__bindgen_anon_1.ulShareCount = 1;
    (*po).BaseFlags = 0;
    // DBG_INITLOG / DBG_LOGEVENT — debug-only, omitted.
}

#[inline]
unsafe fn v_unlock(pbr: PBRUSH) {
    let po = pbr as POBJ;
    if (*po).cExclusiveLock > 0 {
        GDIOBJ_vUnlockObject(po);
    } else {
        GDIOBJ_vDereferenceObject(po);
    }
}

#[inline]
unsafe fn h_insert_object(pbr: PBRUSH, owner: ULONG) -> HGDIOBJ {
    GDIOBJ_hInsertObject(pbr as POBJ, owner)
}

#[inline]
unsafe fn v_set_object_attr(pbr: PBRUSH, pv_user_attr: PVOID) {
    GDIOBJ_vSetObjectAttr(pbr as POBJ, pv_user_attr);
}

/// `BRUSH::LockShared` (LockAny / LockForRead / LockForWrite — the owner is a
/// FIXME no-op in C, so all three are the same share-lock).
#[inline]
unsafe fn brush_lock(hbr: HBRUSH) -> PBRUSH {
    GDIOBJ_ShareLockObj(hbr as HGDIOBJ, GDILoObjType_LO_BRUSH_TYPE as u32) as PBRUSH
}

#[inline]
unsafe fn bb(pbr: PBRUSH) -> *mut BrushBodyView {
    pbrushbody(pbr)
}

/* ---- BRUSH class methods ---- */

/// `BRUSH::BRUSH(...)` constructor body.
unsafe fn brush_construct(
    pbr: PBRUSH,
    fl_attrs: FLONG,
    cr_color: COLORREF,
    i_hatch: ULONG,
    hbm_pattern: HBITMAP,
    pv_client: PVOID,
    loobjtype: GDILoObjType,
) {
    base_object_init(pbr, loobjtype);
    let b = bb(pbr);

    /* Get a unique value (InterlockedIncrementUL returns the new value) */
    (*b).ulBrushUnique = GLOBAL_BRUSH_UNIQUE.fetch_add(1, Ordering::Relaxed).wrapping_add(1);

    /* Start with kmode brush attribute */
    (*b).pBrushAttr = core::ptr::addr_of_mut!((*b).BrushAttr);

    /* Set parameters */
    (*b).flAttrs = fl_attrs;
    (*b).iHatch = i_hatch;
    (*b).hbmPattern = hbm_pattern;
    (*b).hbmClient = pv_client as HBITMAP;
    (*(*b).pBrushAttr).lbColor = cr_color;

    /* Initialize the other fields */
    (*b).ptOrigin.x = 0;
    (*b).ptOrigin.y = 0;
    (*b).bCacheGrabbed = FALSE as ULONG;
    (*b).crBack = 0;
    (*b).crFore = 0;
    (*b).ulPalTime = 0;
    (*b).ulSurfTime = 0;
    (*b).pvRBrush = core::ptr::null_mut();
    (*b).hdev = core::ptr::null_mut();

    /* FIXME: should be done only in PEN constructor, but our destructor needs it! */
    (*b).dwStyleCount = 0;
    (*b).pStyle = core::ptr::null_mut();
}

/// `BRUSH::~BRUSH()` destructor body.
unsafe fn brush_destruct(pbr: PBRUSH) {
    let b = bb(pbr);

    /* Check if we have a user mode brush attribute */
    if (*b).pBrushAttr != core::ptr::addr_of_mut!((*b).BrushAttr) {
        GdiPoolFree(GetBrushAttrPool(), (*b).pBrushAttr as PVOID);
    }

    /* Delete the pattern bitmap (may have already been deleted during gdi cleanup) */
    if !(*b).hbmPattern.is_null() && GreIsHandleValid((*b).hbmPattern as HGDIOBJ) != 0 {
        GreSetBitmapOwner((*b).hbmPattern, GDI_OBJ_HMGR_POWNED as ULONG);
        GreDeleteObject((*b).hbmPattern as HGDIOBJ);
    }

    /* Delete styles */
    if !(*b).pStyle.is_null() && ((*b).flAttrs & BR_IS_DEFAULTSTYLE as FLONG) == 0 {
        ExFreePoolWithTag((*b).pStyle as PVOID, GDITAG_PENSTYLE);
    }
}

/// `new BRUSH(...)` — operator new (pool alloc) + constructor.
unsafe fn brush_new(
    fl_attrs: FLONG,
    cr_color: COLORREF,
    i_hatch: ULONG,
    hbm_pattern: HBITMAP,
    pv_client: PVOID,
) -> PBRUSH {
    let pbr = ExAllocatePoolWithTag(
        PagedPool,
        core::mem::size_of::<_BRUSH>() as SIZE_T,
        GDITAG_HMGR_BRUSH_TYPE,
    ) as PBRUSH;
    if pbr.is_null() {
        return core::ptr::null_mut();
    }
    brush_construct(
        pbr,
        fl_attrs,
        cr_color,
        i_hatch,
        hbm_pattern,
        pv_client,
        GDILoObjType_LO_BRUSH_TYPE,
    );
    pbr
}

/// `delete pbr` — destructor + operator delete (ExFreePool).
unsafe fn brush_delete(pbr: PBRUSH) {
    brush_destruct(pbr);
    ExFreePool(pbr as PVOID);
}

/// `BRUSH::vReleaseAttribute()`
unsafe fn brush_v_release_attribute(pbr: PBRUSH) {
    let b = bb(pbr);
    let kmode = core::ptr::addr_of_mut!((*b).BrushAttr);
    if (*b).pBrushAttr != kmode {
        (*b).BrushAttr = *(*b).pBrushAttr;
        GdiPoolFree(GetBrushAttrPool(), (*b).pBrushAttr as PVOID);
        (*b).pBrushAttr = kmode;
    }
}

/// `BRUSH::bAllocateBrushAttr()`
unsafe fn brush_b_allocate_brush_attr(pbr: PBRUSH) -> BOOL {
    let b = bb(pbr);
    /* NT_ASSERT(this->pBrushAttr == &this->BrushAttr); */

    let p_brush_attr = GdiPoolAllocate(GetBrushAttrPool()) as PBRUSH_ATTR;
    if p_brush_attr.is_null() {
        // ERR("Could not allocate brush attr\n");
        return FALSE as BOOL;
    }

    /* Copy the content from the kernel mode brush attribute */
    (*b).pBrushAttr = p_brush_attr;
    *(*b).pBrushAttr = (*b).BrushAttr;

    /* Set the object attribute in the handle table */
    v_set_object_attr(pbr, p_brush_attr as PVOID);

    TRUE as BOOL
}

/// `BRUSH::vSetSolidColor()`
unsafe fn brush_v_set_solid_color(pbr: PBRUSH, cr_color: COLORREF) {
    let b = bb(pbr);
    /* NT_ASSERT(this->flAttrs & BR_IS_SOLID); */
    (*(*b).pBrushAttr).lbColor = cr_color & 0xFFFFFF;
    (*b).ulPalTime = (-1i32) as ULONG;
    (*b).ulSurfTime = (-1i32) as ULONG;
}

/// `BRUSH::hbmGetBitmapHandle()`
unsafe fn brush_hbm_get_bitmap_handle(pbr: PBRUSH, pu_usage: PUINT) -> HBITMAP {
    let b = bb(pbr);
    *pu_usage = if (*b).flAttrs & BR_IS_DIBPALCOLORS as FLONG != 0 {
        DIB_PAL_COLORS as UINT
    } else if (*b).flAttrs & BR_IS_DIBPALINDICES as FLONG != 0 {
        DIB_PAL_INDICES as UINT
    } else {
        DIB_RGB_COLORS as UINT
    };
    (*b).hbmPattern
}

/// `BRUSH::cjGetObject()`
unsafe fn brush_cj_get_object(pbr: PBRUSH, cj_size: UINT, plb: PLOGBRUSH) -> UINT {
    /* Check if only size is requested */
    if plb.is_null() {
        return core::mem::size_of::<LOGBRUSH>() as UINT;
    }

    /* Check if size is ok */
    if cj_size == 0 {
        return 0;
    }

    let b = bb(pbr);

    /* Set color */
    (*plb).lbColor = (*b).BrushAttr.lbColor;

    /* Set style and hatch based on the attribute flags */
    let fl = (*b).flAttrs;
    if fl & BR_IS_SOLID as FLONG != 0 {
        (*plb).lbStyle = BS_SOLID as UINT;
        (*plb).lbHatch = 0;
    } else if fl & BR_IS_HATCH as FLONG != 0 {
        (*plb).lbStyle = BS_HATCHED as UINT;
        (*plb).lbHatch = (*b).iHatch as ULONG_PTR;
    } else if fl & BR_IS_DIB as FLONG != 0 {
        (*plb).lbStyle = BS_DIBPATTERN as UINT;
        (*plb).lbHatch = (*b).hbmClient as ULONG_PTR;
    } else if fl & BR_IS_BITMAP as FLONG != 0 {
        (*plb).lbStyle = BS_PATTERN as UINT;
        (*plb).lbHatch = (*b).hbmClient as ULONG_PTR;
    } else if fl & BR_IS_NULL as FLONG != 0 {
        (*plb).lbStyle = BS_NULL as UINT;
        (*plb).lbHatch = 0;
    } else {
        /* NT_ASSERT(FALSE); */
    }

    core::mem::size_of::<LOGBRUSH>() as UINT
}

/// `CreateBrushInternal()` — static file-local helper.
unsafe fn create_brush_internal(
    fl_attrs: ULONG,
    cr_color: COLORREF,
    i_hatch: ULONG,
    hbm_pattern: HBITMAP,
    pv_client: PVOID,
) -> HBRUSH {
    /* NT_ASSERT(((flAttrs & BR_IS_BITMAP) == 0) || (hbmPattern != NULL)); */

    /* Create the brush (brush takes ownership of the bitmap) */
    let pbr = brush_new(fl_attrs, cr_color, i_hatch, hbm_pattern, pv_client);
    if pbr.is_null() {
        // ERR("Failed to allocate a brush\n");
        GreSetBitmapOwner(hbm_pattern, GDI_OBJ_HMGR_POWNED as ULONG);
        GreDeleteObject(hbm_pattern as HGDIOBJ);
        return core::ptr::null_mut();
    }

    /* Check if this is a global brush */
    if (fl_attrs & BR_IS_GLOBAL as ULONG) == 0 {
        /* Not a global brush, so allocate a user mode brush attribute */
        if brush_b_allocate_brush_attr(pbr) == 0 {
            // ERR("Failed to allocate brush attribute\n");
            brush_delete(pbr);
            return core::ptr::null_mut();
        }
    }

    /* Set the owner, either public or process owned */
    let owner = if (fl_attrs & BR_IS_GLOBAL as ULONG) != 0 {
        GDI_OBJ_HMGR_PUBLIC as ULONG
    } else {
        GDI_OBJ_HMGR_POWNED as ULONG
    };

    /* Insert the object into the GDI handle table */
    let hbr = h_insert_object(pbr, owner) as HBRUSH;
    if hbr.is_null() {
        // ERR("Failed to insert brush\n");
        brush_delete(pbr);
        return core::ptr::null_mut();
    }

    /* Unlock the brush */
    v_unlock(pbr);

    hbr
}

/* ---- C interface ---- */

#[no_mangle]
pub unsafe extern "system" fn BRUSH_vDeleteObject(pvObject: PVOID) {
    let pbr = pvObject as PBRUSH;
    /* NT_ASSERT type is BRUSH/PEN/EXTPEN */
    brush_delete(pbr);
}

#[inline]
unsafe fn brush_getobject_impl(pbr: PBRUSH, cjBuffer: INT, plbBuffer: PLOGBRUSH) -> INT {
    brush_cj_get_object(pbr, cjBuffer as UINT, plbBuffer) as INT
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn BRUSH_GetObject(pbr: PBRUSH, cjBuffer: INT, plbBuffer: PLOGBRUSH) -> INT {
    brush_getobject_impl(pbr, cjBuffer, plbBuffer)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn BRUSH_GetObject(pbr: PBRUSH, cjBuffer: INT, plbBuffer: PLOGBRUSH) -> INT {
    brush_getobject_impl(pbr, cjBuffer, plbBuffer)
}

#[no_mangle]
pub unsafe extern "system" fn IntGdiCreateNullBrush() -> HBRUSH {
    create_brush_internal(
        (BR_IS_NULL | BR_IS_GLOBAL) as ULONG,
        0,
        0,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    )
}

#[no_mangle]
pub unsafe extern "system" fn IntGdiCreateSolidBrush(crColor: COLORREF) -> HBRUSH {
    create_brush_internal(
        (BR_IS_SOLID | BR_IS_GLOBAL) as ULONG,
        crColor,
        0,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    )
}

#[no_mangle]
pub unsafe extern "system" fn IntGdiCreatePatternBrush(hbmPattern: HBITMAP) -> HBRUSH {
    /* NT_ASSERT(hbmPattern != NULL); */
    GreSetBitmapOwner(hbmPattern, GDI_OBJ_HMGR_PUBLIC as ULONG);
    create_brush_internal(
        (BR_IS_BITMAP | BR_IS_GLOBAL) as ULONG,
        0,
        0,
        hbmPattern,
        core::ptr::null_mut(),
    )
}

#[no_mangle]
pub unsafe extern "system" fn IntGdiSetSolidBrushColor(hbr: HBRUSH, crColor: COLORREF) {
    /* Lock the brush */
    let pbr = brush_lock(hbr);
    if pbr.is_null() {
        // ERR("Failed to lock brush %p\n", hbr);
        return;
    }

    /* Call the member function */
    brush_v_set_solid_color(pbr, crColor);

    /* Unlock the brush */
    v_unlock(pbr);
}

#[no_mangle]
pub unsafe extern "system" fn NtGdiCreateSolidBrush(crColor: COLORREF, hbr: HBRUSH) -> HBRUSH {
    if !hbr.is_null() {
        // WARN("hbr is not supported, ignoring\n");
    }
    create_brush_internal(BR_IS_SOLID as ULONG, crColor, 0, core::ptr::null_mut(), core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "system" fn NtGdiCreateHatchBrushInternal(
    iHatch: ULONG,
    crColor: COLORREF,
    bPen: BOOL,
) -> HBRUSH {
    if bPen != 0 {
        // WARN("bPen is not supported, ignoring\n");
    }

    /* Check what kind if hatch style this is */
    let fl_attr: FLONG = if iHatch < HS_DDI_MAX as ULONG {
        BR_IS_HATCH as FLONG
    } else if iHatch < HS_API_MAX as ULONG {
        BR_IS_SOLID as FLONG
    } else {
        // ERR("Invalid iHatch: %lu\n", iHatch);
        return core::ptr::null_mut();
    };

    create_brush_internal(fl_attr, crColor, iHatch, core::ptr::null_mut(), core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "system" fn NtGdiCreatePatternBrushInternal(
    hbmClient: HBITMAP,
    bPen: BOOL,
    b8X8: BOOL,
) -> HBRUSH {
    if b8X8 != 0 {
        // WARN("b8X8 is not supported, ignoring\n");
    }
    if bPen != 0 {
        // WARN("bPen is not supported, ignoring\n");
    }

    /* Copy the bitmap */
    let hbm_pattern = BITMAP_CopyBitmap(hbmClient);
    if hbm_pattern.is_null() {
        // ERR("Failed to copy the bitmap %p\n", hbmPattern);
        return core::ptr::null_mut();
    }

    /* Call the internal function (will delete hbmPattern on failure) */
    create_brush_internal(BR_IS_BITMAP as ULONG, 0, 0, hbm_pattern, hbmClient as PVOID)
}

#[no_mangle]
pub unsafe extern "system" fn NtGdiCreateDIBBrush(
    pv: PVOID,
    uUsage: FLONG,
    cj: UINT,
    b8X8: BOOL,
    bPen: BOOL,
    pvClient: PVOID,
) -> HBRUSH {
    let mut u_usage = uUsage;

    if b8X8 != 0 {
        // WARN("b8X8 is not supported, ignoring\n");
    }
    if bPen != 0 {
        // WARN("bPen is not supported, ignoring\n");
    }

    if u_usage > DIB_PAL_INDICES as FLONG {
        // ERR("Invalid uUsage value: %lu\n", uUsage);
        EngSetLastError(ERROR_INVALID_PARAMETER as ULONG);
        return core::ptr::null_mut();
    }

    /* Allocate a buffer for the packed DIB */
    let pv_packed_dib = ExAllocatePoolWithTag(PagedPool, cj as SIZE_T, GDITAG_TEMP);
    if pv_packed_dib.is_null() {
        // ERR("Failed to allocate temp buffer of %u bytes\n", cj);
        return core::ptr::null_mut();
    }

    let mut hbr: HBRUSH = core::ptr::null_mut();

    /* Probe and copy the packed DIB */
    // C: _SEH2_TRY { ProbeForRead(pv, cj, 1); RtlCopyMemory(..); }
    //    _SEH2_EXCEPT { goto cleanup; } -- free the temp buffer, return NULL.
    if crate::safe_user_io::copy_from_user(pv_packed_dib, pv as *const core::ffi::c_void, cj as usize, 1).is_err() {
        ExFreePoolWithTag(pv_packed_dib, GDITAG_TEMP);
        return hbr;
    }

    let mut fl_attrs: FLONG = (BR_IS_BITMAP | BR_IS_DIB) as FLONG;

    /* Check what kind of color table we have */
    if u_usage == DIB_PAL_COLORS as FLONG {
        /* Remember it and use DIB_PAL_BRUSHHACK to create a "special" palette */
        fl_attrs |= BR_IS_DIBPALCOLORS as FLONG;
        u_usage = DIB_PAL_BRUSHHACK as FLONG;
    } else if u_usage == DIB_PAL_INDICES as FLONG {
        /* No color table, bitmap contains device palette indices */
        fl_attrs |= BR_IS_DIBPALINDICES as FLONG;
        /* FIXME: This makes tests pass, but needs investigation. */
        fl_attrs |= BR_IS_NULL as FLONG;
    }

    /* Create a bitmap from the DIB */
    let hbm = GreCreateDIBitmapFromPackedDIB(pv_packed_dib, cj, u_usage as ULONG);
    if !hbm.is_null() {
        /* Call the internal function (will delete hbm on failure) */
        hbr = create_brush_internal(fl_attrs, 0, 0, hbm, pvClient);
    }

    /* cleanup: */
    ExFreePoolWithTag(pv_packed_dib, GDITAG_TEMP);

    hbr
}

#[no_mangle]
pub unsafe extern "system" fn NtGdiGetObjectBitmapHandle(hbr: HBRUSH, piUsage: *mut UINT) -> HBITMAP {
    /* Lock the brush */
    let pbr = brush_lock(hbr);
    if pbr.is_null() {
        // ERR("Failed to lock brush %p\n", hbr);
        return core::ptr::null_mut();
    }

    /* Call the member function */
    let mut u_usage: UINT = 0;
    let mut hbm = brush_hbm_get_bitmap_handle(pbr, &mut u_usage);

    /* Unlock the brush */
    v_unlock(pbr);

    // C: _SEH2_TRY { ProbeForWrite(piUsage, sizeof(*piUsage), 1); *piUsage = uUsage; }
    //    _SEH2_EXCEPT { hbm = NULL; }  (a NULL piUsage faults into the handler too)
    if crate::safe_user_io::copy_to_user(
        piUsage as *mut core::ffi::c_void,
        core::ptr::addr_of!(u_usage) as *const core::ffi::c_void,
        core::mem::size_of::<UINT>(),
        1,
    )
    .is_err()
    {
        hbm = core::ptr::null_mut();
    }

    hbm
}

#[no_mangle]
pub unsafe extern "system" fn NtGdiSetBrushAttributes(hbr: HBRUSH, dwFlags: DWORD) -> HBRUSH {
    let mut hbr = hbr;
    if (dwFlags & SC_BB_STOCKOBJ as DWORD) != 0 {
        if GDIOBJ_ConvertToStockObj(core::ptr::addr_of_mut!(hbr) as *mut HGDIOBJ) != 0 {
            let pbr = brush_lock(hbr);
            if pbr.is_null() {
                // ERR("Failed to lock brush %p\n", hbr);
                return core::ptr::null_mut();
            }
            brush_v_release_attribute(pbr);
            v_unlock(pbr);
            return hbr;
        }
    }
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn NtGdiClearBrushAttributes(hbr: HBRUSH, dwFlags: DWORD) -> HBRUSH {
    let mut hbr = hbr;
    if (dwFlags & SC_BB_STOCKOBJ as DWORD) != 0 {
        if GDIOBJ_ConvertFromStockObj(core::ptr::addr_of_mut!(hbr) as *mut HGDIOBJ) != 0 {
            let pbr = brush_lock(hbr);
            if pbr.is_null() {
                // ERR("Failed to lock brush %p\n", hbr);
                return core::ptr::null_mut();
            }
            if brush_b_allocate_brush_attr(pbr) == 0 {
                // ERR("Failed to allocate brush attribute\n");
            }
            v_unlock(pbr);
            return hbr;
        }
    }
    core::ptr::null_mut()
}
