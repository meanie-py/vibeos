use win32k_sys::win32k::*;
use ntkrnl_sys::base::{NT_SUCCESS, STATUS_NOT_IMPLEMENTED};
use ntkrnl_sys::interlocked::InterlockedIncrement;

// FillSolid is APIENTRY (paint.c / paint.rs defines it extern "system"). Declaring it
// extern "C" here collides on i386 (cdecl _FillSolid vs stdcall _FillSolid@12) → same-crate
// ABI mismatch folds the caller (EngEraseSurface) to ud2. Must match the definition.
extern "system" {
    fn FillSolid(pso: *mut SURFOBJ, prcl: *mut RECTL, iColor: ULONG) -> BOOL;
}

extern "system" {
    fn GDIOBJ_vDereferenceObject(pobj: POBJ);
    fn GDIOBJ_vReferenceObjectByPointer(pobj: POBJ);
}

// FORCEINLINE in win32ss/gdi/eng/surface.h:135 — exported as #[no_mangle] cdecl
// so still-C TUs can resolve it (lld error was `_SURFACE_vSetPalette`).
#[no_mangle]
pub unsafe extern "C" fn SURFACE_vSetPalette(psurf: PSURFACE, ppal: PPALETTE) {
    if !(*psurf).ppal.is_null() {
        GDIOBJ_vDereferenceObject((*psurf).ppal as POBJ);
    }
    if !ppal.is_null() {
        GDIOBJ_vReferenceObjectByPointer(ppal as POBJ);
    }
    // *(PVOID*)&psurf->ppal = ppal;  (writing through `const`-cast)
    let ppal_field_ptr = core::ptr::addr_of_mut!((*psurf).ppal) as *mut PPALETTE;
    *ppal_field_ptr = ppal;
}
// drop_gdiobj_macro_externs: dropped 4 bogus extern decl(s) for GDI-object macro(s): PALETTE_ShareLockPalette, PALETTE_ShareUnlockPalette, SURFACE_ShareLockSurface, SURFACE_ShareUnlockSurface — these are macros in gdi/eng/<type>.h or gdi/ntgdi/<type>.h, not real fns. Inline-expand via GDIOBJ_* primitives per win32k_translation_prompt.md §6.
// SURFACE_UnlockSurface(p)        => GDIOBJ_vUnlockObject(p as POBJ)            (surface.h:100)
// SURFACE_ShareLockSurface(h)     => GDIOBJ_ShareLockObj(h as HGDIOBJ, GDI_OBJECT_TYPE_BITMAP) as PSURFACE  (surface.h:91)
// SURFACE_ShareUnlockSurface(p)   => GDIOBJ_vDereferenceObject(p as POBJ)        (surface.h:102)
// PALETTE_ShareLockPalette(h)     => GDIOBJ_ShareLockObj(h as HGDIOBJ, GDI_OBJECT_TYPE_PALETTE) as PPALETTE (palette.h:57)
// PALETTE_ShareUnlockPalette(p)   => GDIOBJ_vDereferenceObject(&(*p).BaseObject as POBJ)                   (palette.h:59)

// WIDTH_BYTES_ALIGN32(cx, bpp) macro (surface.h:111) — inline expansion.
#[inline(always)]
unsafe fn WIDTH_BYTES_ALIGN32(cx: ULONG, bpp: ULONG) -> ULONG {
    ((((cx as u64) * (bpp as u64) + 31) & !31) >> 3) as ULONG
}

// RtlULongMult — __forceinline from ntintsafe.h. Checked u32 multiply.
// Returns STATUS_SUCCESS (0) or STATUS_INTEGER_OVERFLOW (0xC0000095).
#[inline(always)]
unsafe fn RtlULongMult(a: ULONG, b: ULONG, out: *mut ULONG) -> NTSTATUS {
    let result = (a as u64).wrapping_mul(b as u64);
    if result > (ULONG::MAX as u64) {
        *out = ULONG::MAX;
        0xC0000095u32 as NTSTATUS // STATUS_INTEGER_OVERFLOW
    } else {
        *out = result as ULONG;
        0 // STATUS_SUCCESS
    }
}

// ---- file-scope preamble (translated) ----
pub static mut giUniqueSurface: i32 = 0;

#[no_mangle]
pub static gajBitsPerFormat: [u8; 11] = [
    0,
    1,
    4,
    8,
    16,
    24,
    32,
    4,
    8,
    0,
    0,
];

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn BitmapFormat(cBits: ULONG, iCompression: ULONG) -> ULONG {
    match iCompression {
        BI_RGB | BI_BITFIELDS => {
            if cBits <= 1 { return BMF_1BPP; }
            if cBits <= 4 { return BMF_4BPP; }
            if cBits <= 8 { return BMF_8BPP; }
            if cBits <= 16 { return BMF_16BPP; }
            if cBits <= 24 { return BMF_24BPP; }
            if cBits <= 32 { return BMF_32BPP; }
            0
        }
        BI_RLE4 => BMF_4RLE,
        BI_RLE8 => BMF_8RLE,
        _ => 0,
    }
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn BitmapFormat(cBits: ULONG, iCompression: ULONG) -> ULONG {
    match iCompression {
        BI_RGB | BI_BITFIELDS => {
            if cBits <= 1 { return BMF_1BPP; }
            if cBits <= 4 { return BMF_4BPP; }
            if cBits <= 8 { return BMF_8BPP; }
            if cBits <= 16 { return BMF_16BPP; }
            if cBits <= 24 { return BMF_24BPP; }
            if cBits <= 32 { return BMF_32BPP; }
            0
        }
        BI_RLE4 => BMF_4RLE,
        BI_RLE8 => BMF_8RLE,
        _ => 0,
    }
}

// NTAPI in C (surface.h:120) — stdcall on i386, cdecl elsewhere.
#[no_mangle]
pub unsafe extern "system" fn SURFACE_vCleanup(ObjectBody: PVOID) {
    SURFACE_vCleanup_impl(ObjectBody)
}

unsafe fn SURFACE_vCleanup_impl(ObjectBody: PVOID) {
    let psurf = ObjectBody as PSURFACE;
    let pvBits = (*psurf).SurfObj.pvBits;

    if !pvBits.is_null() {
        if !(*psurf).hDIBSection.is_null() {
            EngUnmapSectionView(pvBits, (*psurf).dwOffset, (*psurf).hSecure);
        } else if ((*psurf).SurfObj.fjBitmap & (BMF_USERMEM as u16)) != 0 {
            EngFreeUserMem(pvBits);
        } else if ((*psurf).SurfObj.fjBitmap & (BMF_KMSECTION as u16)) != 0 {
            if EngFreeSectionMem(core::ptr::null_mut(), pvBits) == 0 {
            }
        } else if ((*psurf).SurfObj.fjBitmap & (BMF_POOLALLOC as u16)) != 0 {
            EngFreeMem(pvBits);
        }
    }

    if !(*psurf).ppal.is_null() {
        // PALETTE_ShareUnlockPalette((*psurf).ppal) — macro palette.h:59
        //   => GDIOBJ_vDereferenceObject(&ppal->BaseObject)
        GDIOBJ_vDereferenceObject(core::ptr::addr_of_mut!((*(*psurf).ppal).BaseObject));
    }
}

#[inline(always)]
unsafe fn SURFACE_AllocSurface_impl(
    iType: u16,
    cx: u32,
    cy: u32,
    iFormat: u32,
    fjBitmap: u32,
    cjWidth: u32,
    cjBufSize: u32,
    pvBits: *mut core::ffi::c_void,
) -> PSURFACE {
    let cBitsPixel: u32;
    let mut cjBits: u32 = 0;
    let cjObject: u32;
    let psurf: PSURFACE;
    let pso: *mut SURFOBJ;
    let mut pvSection: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut cjWidth = cjWidth;

    if (iFormat < BMF_1BPP) || (iFormat > BMF_PNG) {
        return core::ptr::null_mut();
    }

    cBitsPixel = gajBitsPerFormat[iFormat as usize] as u32;

    if !pvBits.is_null() && cjWidth != 0 {
        cjWidth = WIDTH_BYTES_ALIGN32((cjWidth << 3) / cBitsPixel, cBitsPixel);
    } else {
        cjWidth = WIDTH_BYTES_ALIGN32(cx, cBitsPixel);
    }

    if iFormat <= BMF_32BPP {
        if !NT_SUCCESS(RtlULongMult(cjWidth, cy, &mut cjBits)) {
            return core::ptr::null_mut();
        }

        if !pvBits.is_null() && cjBufSize != 0 {
            if cjBufSize < cjBits {
                return core::ptr::null_mut();
            }
        }
    } else {
        cjBits = cjBufSize;
    }

    if (iType == STYPE_BITMAP as u16) && pvBits.is_null() &&
        (fjBitmap & BMF_USERMEM) == 0 && (fjBitmap & BMF_KMSECTION) == 0 {
        cjObject = (core::mem::size_of::<SURFACE>() as u32).wrapping_add(cjBits);
    } else {
        cjObject = core::mem::size_of::<SURFACE>() as u32;
    }

    if cjObject < core::mem::size_of::<SURFACE>() as u32 {
        return core::ptr::null_mut();
    }

    psurf = GDIOBJ_AllocObjWithHandle(GDI_OBJECT_TYPE_BITMAP, cjObject) as PSURFACE;
    if psurf.is_null() {
        return core::ptr::null_mut();
    }

    pso = core::ptr::addr_of_mut!((*psurf).SurfObj);
    (*pso).hsurf = (*psurf).BaseObject.hHmgr as HSURF;
    (*pso).sizlBitmap.cx = cx as i32;
    (*pso).sizlBitmap.cy = cy as i32;
    (*pso).iBitmapFormat = iFormat;
    (*pso).iType = iType;
    (*pso).fjBitmap = fjBitmap as u16;
    (*pso).iUniq = InterlockedIncrement(core::ptr::addr_of_mut!(giUniqueSurface)) as u32;
    (*pso).cjBits = cjBits;

    if iType == STYPE_BITMAP as u16 {
        if !pvBits.is_null() {
            (*pso).pvBits = pvBits;
        } else if (fjBitmap & BMF_USERMEM) != 0 {
            (*pso).pvBits = EngAllocUserMem(cjBits as SIZE_T, 0);

            if (*pso).pvBits.is_null() {
                GDIOBJ_vDeleteObject(core::ptr::addr_of_mut!((*psurf).BaseObject));
                return core::ptr::null_mut();
            }
        } else if (fjBitmap & BMF_KMSECTION) != 0 {
            (*pso).pvBits = EngAllocSectionMem(
                &mut pvSection,
                if (fjBitmap & BMF_NOZEROINIT) != 0 { 0 } else { FL_ZERO_MEMORY },
                cjBits as SIZE_T,
                0x20424944u32,
            );

            if (*pso).pvBits.is_null() {
                GDIOBJ_vDeleteObject(core::ptr::addr_of_mut!((*psurf).BaseObject));
                return core::ptr::null_mut();
            }

            EngFreeSectionMem(pvSection, core::ptr::null_mut());
        } else {
            // C: pso->pvBits = psurf + 1  (pointer arithmetic — advance by sizeof(SURFACE))
            (*pso).pvBits = psurf.add(1) as *mut core::ffi::c_void;

            if (fjBitmap & BMF_NOZEROINIT) == 0 {
                core::ptr::write_bytes((*pso).pvBits as *mut u8, 0, cjBits as usize);
            }
        }

        if (fjBitmap & BMF_TOPDOWN) != 0 {
            (*pso).pvScan0 = (*pso).pvBits;
            (*pso).lDelta = cjWidth as i32;
        } else {
            (*pso).pvScan0 = ((*pso).pvBits as *mut u8).add(((*pso).cjBits as usize).wrapping_sub(cjWidth as usize)) as *mut core::ffi::c_void;
            (*pso).lDelta = -(cjWidth as i32);
        }
    } else {
        (*pso).pvScan0 = core::ptr::null_mut();
        (*pso).pvBits = core::ptr::null_mut();
        (*pso).lDelta = 0;
    }

    // appalSurfaceDefault is `[PPALETTE; 0]` (flexible array) in bindgen — read
    // element via raw pointer arithmetic, not direct indexing (see memory
    // feedback_bindgen_flexible_array_index.md: indexing past 0 panic-aborts).
    let appal_base = core::ptr::addr_of_mut!(appalSurfaceDefault) as *mut PPALETTE;
    let default_ppal = *appal_base.add(iFormat as usize);
    SURFACE_vSetPalette(psurf, default_ppal);

    psurf
}

#[no_mangle]
pub unsafe extern "system" fn SURFACE_AllocSurface(
    iType: u16,
    cx: u32,
    cy: u32,
    iFormat: u32,
    fjBitmap: u32,
    cjWidth: u32,
    cjBufSize: u32,
    pvBits: *mut core::ffi::c_void,
) -> PSURFACE {
    SURFACE_AllocSurface_impl(iType, cx, cy, iFormat, fjBitmap, cjWidth, cjBufSize, pvBits)
}

#[no_mangle]
pub unsafe extern "system" fn EngLockSurface(hsurf: HSURF) -> *mut SURFOBJ {
    // SURFACE_ShareLockSurface(hsurf) — macro surface.h:91 expands to
    //   GDIOBJ_ShareLockObj(hsurf as HGDIOBJ, GDI_OBJECT_TYPE_BITMAP) as PSURFACE
    let psurf =
        GDIOBJ_ShareLockObj(hsurf as HGDIOBJ, GDI_OBJECT_TYPE_BITMAP) as PSURFACE;
    if psurf.is_null() {
        core::ptr::null_mut()
    } else {
        core::ptr::addr_of_mut!((*psurf).SurfObj)
    }
}

#[no_mangle]
pub unsafe extern "system" fn NtGdiEngLockSurface(hsurf: HSURF) -> *mut SURFOBJ {
    EngLockSurface(hsurf)
}

#[no_mangle]
pub unsafe extern "system" fn EngCreateBitmap(
    sizl: SIZEL,
    lWidth: i32,
    iFormat: u32,
    fl: u32,
    pvBits: *mut core::ffi::c_void,
) -> HBITMAP {
    let psurf: PSURFACE;
    let hbmp: HBITMAP;

    psurf = SURFACE_AllocSurface(
        STYPE_BITMAP as u16,
        sizl.cx as u32,
        sizl.cy as u32,
        iFormat,
        fl,
        lWidth as u32,
        0,
        pvBits,
    );
    if psurf.is_null() {
        return core::ptr::null_mut();
    }

    hbmp = (*psurf).SurfObj.hsurf as HBITMAP;

    (*psurf).flags = API_BITMAP as FLONG;

    GDIOBJ_vSetObjectOwner(
        core::ptr::addr_of_mut!((*psurf).BaseObject),
        GDI_OBJ_HMGR_PUBLIC,
    );

    // SURFACE_UnlockSurface(psurf) — macro surface.h:100
    //   => GDIOBJ_vUnlockObject((POBJ)pBMObj)
    GDIOBJ_vUnlockObject(core::ptr::addr_of_mut!((*psurf).BaseObject));
    hbmp
}
#[no_mangle]
pub unsafe extern "system" fn EngCreateDeviceBitmap(
    dhsurf: DHSURF,
    sizl: SIZEL,
    iFormat: u32,
) -> HBITMAP {
    let psurf: PSURFACE;
    let hbmp: HBITMAP;

    psurf = SURFACE_AllocSurface(
        STYPE_DEVBITMAP as u16,
        sizl.cx as u32,
        sizl.cy as u32,
        iFormat,
        0,
        0,
        0,
        core::ptr::null_mut(),
    );
    if psurf.is_null() {
        return core::ptr::null_mut();
    }

    (*psurf).SurfObj.dhsurf = dhsurf;

    GDIOBJ_vSetObjectOwner(
        core::ptr::addr_of_mut!((*psurf).BaseObject),
        GDI_OBJ_HMGR_PUBLIC,
    );

    hbmp = (*psurf).SurfObj.hsurf as HBITMAP;

    // SURFACE_UnlockSurface(psurf) — macro surface.h:100
    GDIOBJ_vUnlockObject(core::ptr::addr_of_mut!((*psurf).BaseObject));
    hbmp
}
#[no_mangle]
pub unsafe extern "system" fn EngCreateDeviceSurface(
    dhsurf: DHSURF,
    sizl: SIZEL,
    iFormat: u32,
) -> HSURF {
    let psurf: PSURFACE;
    let hsurf: HSURF;

    psurf = SURFACE_AllocSurface(
        STYPE_DEVICE as u16,
        sizl.cx as u32,
        sizl.cy as u32,
        iFormat,
        0,
        0,
        0,
        core::ptr::null_mut(),
    );
    if psurf.is_null() {
        return core::ptr::null_mut();
    }

    (*psurf).SurfObj.dhsurf = dhsurf;

    GDIOBJ_vSetObjectOwner(
        core::ptr::addr_of_mut!((*psurf).BaseObject),
        GDI_OBJ_HMGR_PUBLIC,
    );

    hsurf = (*psurf).SurfObj.hsurf;

    // SURFACE_UnlockSurface(psurf) — macro surface.h:100
    GDIOBJ_vUnlockObject(core::ptr::addr_of_mut!((*psurf).BaseObject));
    return hsurf;
}
#[no_mangle]
pub unsafe extern "system" fn EngAssociateSurface(
    hsurf: HSURF,
    hdev: HDEV,
    flHooks: FLONG,
) -> i32 {

    let pso: *mut SURFOBJ;
    let psurf: PSURFACE;
    let ppdev: *mut PDEVOBJ;
    let ppal: PPALETTE;

    ppdev = hdev as *mut PDEVOBJ;

    // SURFACE_ShareLockSurface(hsurf) — macro surface.h:91
    //   => (PSURFACE)GDIOBJ_ShareLockObj((HGDIOBJ)hBMObj, GDI_OBJECT_TYPE_BITMAP)
    psurf = GDIOBJ_ShareLockObj(hsurf as HGDIOBJ, GDI_OBJECT_TYPE_BITMAP) as PSURFACE;
    if psurf.is_null() {
        return 0;
    }
    pso = core::ptr::addr_of_mut!((*psurf).SurfObj);

    (*pso).hdev = hdev;
    (*pso).dhpdev = (*ppdev).dhpdev;

    (*psurf).flags &= !HOOK_FLAGS;
    (*psurf).flags |= flHooks & HOOK_FLAGS;

    // PALETTE_ShareLockPalette(hpalDefault) — macro palette.h:57
    //   => (PPALETTE)GDIOBJ_ShareLockObj((HGDIOBJ)hpal, GDI_OBJECT_TYPE_PALETTE)
    ppal = GDIOBJ_ShareLockObj(
        (*ppdev).devinfo.hpalDefault as HGDIOBJ,
        GDI_OBJECT_TYPE_PALETTE,
    ) as PPALETTE;
    SURFACE_vSetPalette(psurf, ppal);
    // PALETTE_ShareUnlockPalette(ppal) — macro palette.h:59
    //   => GDIOBJ_vDereferenceObject(&ppal->BaseObject)
    GDIOBJ_vDereferenceObject(core::ptr::addr_of_mut!((*ppal).BaseObject));

    // SURFACE_ShareUnlockSurface(psurf) — macro surface.h:102
    //   => GDIOBJ_vDereferenceObject((POBJ)pBMObj)
    GDIOBJ_vDereferenceObject(core::ptr::addr_of_mut!((*psurf).BaseObject));

    1
}
#[no_mangle]
pub unsafe extern "system" fn EngModifySurface(
    hsurf: HSURF,
    hdev: HDEV,
    flHooks: FLONG,
    flSurface: FLONG,
    dhsurf: DHSURF,
    pvScan0: *mut core::ffi::c_void,
    lDelta: i32,
    pvReserved: *mut core::ffi::c_void,
) -> i32 {

    // SURFACE_ShareLockSurface(hsurf) — macro surface.h:91
    let psurf = GDIOBJ_ShareLockObj(hsurf as HGDIOBJ, GDI_OBJECT_TYPE_BITMAP) as PSURFACE;
    if psurf.is_null() {
        return 0;
    }

    let ppdev = hdev as *mut PDEVOBJ;
    let pso = core::ptr::addr_of_mut!((*psurf).SurfObj);
    (*pso).dhsurf = dhsurf;

    (*pso).hdev = hdev;
    (*pso).dhpdev = (*ppdev).dhpdev;

    (*psurf).flags &= !HOOK_FLAGS;
    (*psurf).flags |= flHooks & HOOK_FLAGS;

    // PALETTE_ShareLockPalette(hpalDefault) — macro palette.h:57
    let ppal = GDIOBJ_ShareLockObj(
        (*ppdev).devinfo.hpalDefault as HGDIOBJ,
        GDI_OBJECT_TYPE_PALETTE,
    ) as PPALETTE;
    SURFACE_vSetPalette(psurf, ppal);
    // PALETTE_ShareUnlockPalette(ppal) — macro palette.h:59
    GDIOBJ_vDereferenceObject(core::ptr::addr_of_mut!((*ppal).BaseObject));

    if (flSurface & MS_NOTSYSTEMMEMORY) != 0 {
        (*pso).fjBitmap |= BMF_NOTSYSMEM as u16;
    } else {
        (*pso).fjBitmap &= !(BMF_NOTSYSMEM as u16);
    }
    if (flSurface & MS_SHAREDACCESS) != 0 {
        (*psurf).flags |= SHAREACCESS_SURFACE as FLONG;
    } else {
        (*psurf).flags &= !(SHAREACCESS_SURFACE as FLONG);
    }

    if !pvScan0.is_null() && lDelta != 0 {
        (*pso).pvScan0 = pvScan0;
        (*pso).lDelta = lDelta;

        (*pso).iType = STYPE_BITMAP as u16;

        if lDelta > 0 {
            (*pso).cjBits = (lDelta * (*pso).sizlBitmap.cy) as u32;
            (*pso).pvBits = (*pso).pvScan0;
            (*pso).fjBitmap |= BMF_TOPDOWN as u16;
        } else {
            (*pso).cjBits = ((-lDelta) * (*pso).sizlBitmap.cy) as u32;
            (*pso).pvBits = ((*pso).pvScan0 as *mut u8).offset(-((*pso).cjBits as isize) - (lDelta as isize)) as *mut core::ffi::c_void;
            (*pso).fjBitmap &= !(BMF_TOPDOWN as u16);
        }
    } else {
        (*pso).pvBits = core::ptr::null_mut();
        (*pso).pvScan0 = core::ptr::null_mut();
        (*pso).lDelta = 0;

        if (*pso).iType != STYPE_DEVICE as u16 {
            (*pso).iType = STYPE_DEVBITMAP as u16;
        }
    }

    // SURFACE_ShareUnlockSurface(psurf) — macro surface.h:102
    GDIOBJ_vDereferenceObject(core::ptr::addr_of_mut!((*psurf).BaseObject));

    1
}
#[no_mangle]
pub unsafe extern "system" fn EngDeleteSurface(
    hsurf: HSURF
) -> BOOL {
    let psurf: PSURFACE;

    // SURFACE_ShareLockSurface(hsurf) — macro surface.h:91
    psurf = GDIOBJ_ShareLockObj(hsurf as HGDIOBJ, GDI_OBJECT_TYPE_BITMAP) as PSURFACE;
    if psurf.is_null() {
        return 0;
    }

    GDIOBJ_vDeleteObject(core::ptr::addr_of_mut!((*psurf).BaseObject));
    return 1;
}
#[no_mangle]
pub unsafe extern "system" fn EngEraseSurface(
    pso: *mut SURFOBJ,
    prcl: *mut RECTL,
    iColor: ULONG,
) -> BOOL {

    FillSolid(pso, prcl, iColor)
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiEngUnlockSurface(
    pso: *mut SURFOBJ,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}
#[no_mangle]
pub unsafe extern "system" fn EngUnlockSurface(pso: *mut SURFOBJ) {
    if !pso.is_null() {
        let psurf = (pso as *mut u8).offset(-(core::mem::offset_of!(SURFACE, SurfObj) as isize)) as *mut SURFACE;
        // SURFACE_ShareUnlockSurface(psurf) — macro surface.h:102
        GDIOBJ_vDereferenceObject(core::ptr::addr_of_mut!((*psurf).BaseObject));
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:         See COPYING in the top level directory
//  * PROJECT:           ReactOS kernel
//  * PURPOSE:           GDI Driver Surace Functions
//  * FILE:              win32ss/gdi/eng/surface.c
//  * PROGRAMERS:        Jason Filby
//  *                    Timo Kreuzer
//  * TESTING TO BE DONE:
//  * - Create a GDI bitmap with all formats, perform all drawing operations on them, render to VGA surface
//  *   refer to \test\microwin\src\engine\devdraw.c for info on correct pixel plotting for various formats
//  */
//
// #include <win32k.h>
//
// #define NDEBUG
// #include <debug.h>
//
// LONG giUniqueSurface = 0;
//
// UCHAR
// gajBitsPerFormat[11] =
// {
//     0, /*  0: unused */
//     1, /*  1: BMF_1BPP */
//     4, /*  2: BMF_4BPP */
//     8, /*  3: BMF_8BPP */
//    16, /*  4: BMF_16BPP */
//    24, /*  5: BMF_24BPP */
//    32, /*  6: BMF_32BPP */
//     4, /*  7: BMF_4RLE */
//     8, /*  8: BMF_8RLE */
//     0, /*  9: BMF_JPEG */
//     0, /* 10: BMF_PNG */
// };
//
//
// ULONG
// FASTCALL
// BitmapFormat(ULONG cBits, ULONG iCompression)
// {
//     switch (iCompression)
//     {
//         case BI_RGB:
//             /* Fall through */
//         case BI_BITFIELDS:
//             if (cBits <= 1) return BMF_1BPP;
//             if (cBits <= 4) return BMF_4BPP;
//             if (cBits <= 8) return BMF_8BPP;
//             if (cBits <= 16) return BMF_16BPP;
//             if (cBits <= 24) return BMF_24BPP;
//             if (cBits <= 32) return BMF_32BPP;
//             return 0;
//
//         case BI_RLE4:
//             return BMF_4RLE;
//
//         case BI_RLE8:
//             return BMF_8RLE;
//
//         default:
//             return 0;
//     }
// }
//
// VOID
// NTAPI
// SURFACE_vCleanup(PVOID ObjectBody)
// {
//     PSURFACE psurf = (PSURFACE)ObjectBody;
//     PVOID pvBits = psurf->SurfObj.pvBits;
//
//     /* Check if the surface has bits */
//     if (pvBits)
//     {
//         /* Only bitmaps can have bits */
//         ASSERT(psurf->SurfObj.iType == STYPE_BITMAP);
//
//         /* Check if it is a DIB section */
//         if (psurf->hDIBSection)
//         {
//             /* Unmap the section view */
//             EngUnmapSectionView(pvBits, psurf->dwOffset, psurf->hSecure);
//         }
//         else if (psurf->SurfObj.fjBitmap & BMF_USERMEM)
//         {
//             /* Bitmap was allocated from usermode memory */
//             EngFreeUserMem(pvBits);
//         }
//         else if (psurf->SurfObj.fjBitmap & BMF_KMSECTION)
//         {
//             /* Bitmap was allocated from a kernel section */
//             if (!EngFreeSectionMem(NULL, pvBits))
//             {
//                 DPRINT1("EngFreeSectionMem failed for %p!\n", pvBits);
//                 // Should we BugCheck here?
//                 ASSERT(FALSE);
//             }
//         }
//         else if (psurf->SurfObj.fjBitmap & BMF_POOLALLOC)
//         {
//             /* Free a pool allocation */
//             EngFreeMem(pvBits);
//         }
//     }
//
//     /* Free palette */
//     if(psurf->ppal)
//     {
//         PALETTE_ShareUnlockPalette(psurf->ppal);
//     }
// }
//
//
// PSURFACE
// NTAPI
// SURFACE_AllocSurface(
//     _In_ USHORT iType,
//     _In_ ULONG cx,
//     _In_ ULONG cy,
//     _In_ ULONG iFormat,
//     _In_ ULONG fjBitmap,
//     _In_opt_ ULONG cjWidth,
//     _In_opt_ ULONG cjBufSize,
//     _In_opt_ PVOID pvBits)
// {
//     ULONG cBitsPixel, cjBits, cjObject;
//     PSURFACE psurf;
//     SURFOBJ *pso;
//     PVOID pvSection;
//
//     NT_ASSERT(!pvBits || (iType == STYPE_BITMAP));
//     NT_ASSERT((iFormat <= BMF_32BPP) || (cjBufSize != 0));
//     NT_ASSERT((LONG)cy > 0);
//
//     /* Verify format */
//     if ((iFormat < BMF_1BPP) || (iFormat > BMF_PNG))
//     {
//         DPRINT1("Invalid bitmap format: %lu\n", iFormat);
//         return NULL;
//     }
//
//     /* Get bits per pixel from the format */
//     cBitsPixel = gajBitsPerFormat[iFormat];
//
//     /* Are bits and a width in bytes given? */
//     if (pvBits && cjWidth)
//     {
//         /* Align the width (Windows compatibility, drivers expect that) */
//         cjWidth = WIDTH_BYTES_ALIGN32((cjWidth << 3) / cBitsPixel, cBitsPixel);
//     }
//     else
//     {
//         /* Calculate width from the bitmap width in pixels */
//         cjWidth = WIDTH_BYTES_ALIGN32(cx, cBitsPixel);
//     }
//
//     /* Is this an uncompressed format? */
//     if (iFormat <= BMF_32BPP)
//     {
//         /* Calculate the correct bitmap size in bytes */
//         if (!NT_SUCCESS(RtlULongMult(cjWidth, cy, &cjBits)))
//         {
//             DPRINT1("Overflow calculating size: cjWidth %lu, cy %lu\n",
//                     cjWidth, cy);
//             return NULL;
//         }
//
//         /* Did we get a buffer and size? */
//         if ((pvBits != NULL) && (cjBufSize != 0))
//         {
//             /* Make sure the buffer is large enough */
//             if (cjBufSize < cjBits)
//             {
//                 DPRINT1("Buffer is too small, required: %lu, got %lu\n",
//                         cjBits, cjBufSize);
//                 return NULL;
//             }
//         }
//     }
//     else
//     {
//         /* Compressed format, use the provided size */
//         NT_ASSERT(cjBufSize != 0);
//         cjBits = cjBufSize;
//     }
//
//     /* Check if we need an extra large object */
//     if ((iType == STYPE_BITMAP) && (pvBits == NULL) &&
//         !(fjBitmap & BMF_USERMEM) && !(fjBitmap & BMF_KMSECTION))
//     {
//         /* Allocate an object large enough to hold the bits */
//         cjObject = sizeof(SURFACE) + cjBits;
//     }
//     else
//     {
//         /* Otherwise just allocate the SURFACE structure */
//         cjObject = sizeof(SURFACE);
//     }
//
//     /* Check for arithmetic overflow */
//     if (cjObject < sizeof(SURFACE))
//     {
//         /* Fail! */
//         DPRINT1("Overflow calculating cjObject: cjBits %lu\n", cjBits);
//         return NULL;
//     }
//
//     /* Allocate a SURFACE object */
//     psurf = (PSURFACE)GDIOBJ_AllocObjWithHandle(GDI_OBJECT_TYPE_BITMAP, cjObject);
//     if (!psurf)
//     {
//         return NULL;
//     }
//
//     /* Initialize the basic fields */
//     pso = &psurf->SurfObj;
//     pso->hsurf = psurf->BaseObject.hHmgr;
//     pso->sizlBitmap.cx = cx;
//     pso->sizlBitmap.cy = cy;
//     pso->iBitmapFormat = iFormat;
//     pso->iType = iType;
//     pso->fjBitmap = (USHORT)fjBitmap;
//     pso->iUniq = InterlockedIncrement(&giUniqueSurface);
//     pso->cjBits = cjBits;
//
//     /* Check if we need a bitmap buffer */
//     if (iType == STYPE_BITMAP)
//     {
//         /* Check if we got one or if we need to allocate one */
//         if (pvBits != NULL)
//         {
//             /* Use the caller provided buffer */
//             pso->pvBits = pvBits;
//         }
//         else if (fjBitmap & BMF_USERMEM)
//         {
//             /* User mode memory was requested */
//             pso->pvBits = EngAllocUserMem(cjBits, 0);
//
//             /* Check for failure */
//             if (!pso->pvBits)
//             {
//                 GDIOBJ_vDeleteObject(&psurf->BaseObject);
//                 return NULL;
//             }
//         }
//         else if (fjBitmap & BMF_KMSECTION)
//         {
//             /* Use a kernel mode section */
//             pso->pvBits = EngAllocSectionMem(&pvSection,
//                                              (fjBitmap & BMF_NOZEROINIT) ?
//                                                  0 : FL_ZERO_MEMORY,
//                                              cjBits, TAG_DIB);
//
//             /* Check for failure */
//             if (!pso->pvBits)
//             {
//                 GDIOBJ_vDeleteObject(&psurf->BaseObject);
//                 return NULL;
//             }
//
//             /* Free the section already, but keep the mapping */
//             EngFreeSectionMem(pvSection, NULL);
//         }
//         else
//         {
//             /* Buffer is after the object */
//             pso->pvBits = psurf + 1;
//
//             /* Zero the buffer, except requested otherwise */
//             if (!(fjBitmap & BMF_NOZEROINIT))
//             {
//                 RtlZeroMemory(pso->pvBits, cjBits);
//             }
//         }
//
//         /* Set pvScan0 and lDelta */
//         if (fjBitmap & BMF_TOPDOWN)
//         {
//             /* Topdown is the normal way */
//             pso->pvScan0 = pso->pvBits;
//             pso->lDelta = cjWidth;
//         }
//         else
//         {
//             /* Inversed bitmap (bottom up) */
//             pso->pvScan0 = ((PCHAR)pso->pvBits + pso->cjBits - cjWidth);
//             pso->lDelta = -(LONG)cjWidth;
//         }
//     }
//     else
//     {
//         /* There are no bitmap bits */
//         pso->pvScan0 = pso->pvBits = NULL;
//         pso->lDelta = 0;
//     }
//
//     /* Assign a default palette and increment its reference count */
//     SURFACE_vSetPalette(psurf, appalSurfaceDefault[iFormat]);
//
//     return psurf;
// }
//
// HBITMAP
// APIENTRY
// EngCreateBitmap(
//     _In_ SIZEL sizl,
//     _In_ LONG lWidth,
//     _In_ ULONG iFormat,
//     _In_ ULONG fl,
//     _In_opt_ PVOID pvBits)
// {
//     PSURFACE psurf;
//     HBITMAP hbmp;
//
//     /* Allocate a surface */
//     psurf = SURFACE_AllocSurface(STYPE_BITMAP,
//                                  sizl.cx,
//                                  sizl.cy,
//                                  iFormat,
//                                  fl,
//                                  lWidth,
//                                  0,
//                                  pvBits);
//     if (!psurf)
//     {
//         DPRINT1("SURFACE_AllocSurface failed. (STYPE_BITMAP, sizl.cx=%ld, sizl.cy=%ld, iFormat=%lu, fl=%lu, lWidth=%ld, pvBits=0x%p)\n",
//                 sizl.cx, sizl.cy, iFormat, fl, lWidth, pvBits);
//         return NULL;
//     }
//
//     /* Get the handle for the bitmap */
//     hbmp = (HBITMAP)psurf->SurfObj.hsurf;
//
//     /* Mark as API bitmap */
//     psurf->flags = API_BITMAP;
//
//     /* Set public ownership */
//     GDIOBJ_vSetObjectOwner(&psurf->BaseObject, GDI_OBJ_HMGR_PUBLIC);
//
//     /* Unlock the surface and return */
//     SURFACE_UnlockSurface(psurf);
//     return hbmp;
// }
//
// /*
//  * @implemented
//  */
// HBITMAP
// APIENTRY
// EngCreateDeviceBitmap(
//     _In_ DHSURF dhsurf,
//     _In_ SIZEL sizl,
//     _In_ ULONG iFormat)
// {
//     PSURFACE psurf;
//     HBITMAP hbmp;
//
//     /* Allocate a surface */
//     psurf = SURFACE_AllocSurface(STYPE_DEVBITMAP,
//                                  sizl.cx,
//                                  sizl.cy,
//                                  iFormat,
//                                  0,
//                                  0,
//                                  0,
//                                  NULL);
//     if (!psurf)
//     {
//         DPRINT1("SURFACE_AllocSurface failed. (STYPE_DEVBITMAP, sizl.cx=%ld, sizl.cy=%ld, iFormat=%lu)\n",
//                 sizl.cx, sizl.cy, iFormat);
//         return NULL;
//     }
//
//     /* Set the device handle */
//     psurf->SurfObj.dhsurf = dhsurf;
//
//     /* Set public ownership */
//     GDIOBJ_vSetObjectOwner(&psurf->BaseObject, GDI_OBJ_HMGR_PUBLIC);
//
//     /* Get the handle for the bitmap */
//     hbmp = (HBITMAP)psurf->SurfObj.hsurf;
//
//     /* Unlock the surface and return */
//     SURFACE_UnlockSurface(psurf);
//     return hbmp;
// }
//
// HSURF
// APIENTRY
// EngCreateDeviceSurface(
//     _In_ DHSURF dhsurf,
//     _In_ SIZEL sizl,
//     _In_ ULONG iFormat)
// {
//     PSURFACE psurf;
//     HSURF hsurf;
//
//     /* Allocate a surface */
//     psurf = SURFACE_AllocSurface(STYPE_DEVICE,
//                                  sizl.cx,
//                                  sizl.cy,
//                                  iFormat,
//                                  0,
//                                  0,
//                                  0,
//                                  NULL);
//     if (!psurf)
//     {
//         DPRINT1("SURFACE_AllocSurface failed. (STYPE_DEVICE, sizl.cx=%ld, sizl.cy=%ld, iFormat=%lu)\n",
//                 sizl.cx, sizl.cy, iFormat);
//         return NULL;
//     }
//
//     /* Set the device handle */
//     psurf->SurfObj.dhsurf = dhsurf;
//
//     /* Set public ownership */
//     GDIOBJ_vSetObjectOwner(&psurf->BaseObject, GDI_OBJ_HMGR_PUBLIC);
//
//     /* Get the handle for the surface */
//     hsurf = psurf->SurfObj.hsurf;
//
//     /* Unlock the surface and return */
//     SURFACE_UnlockSurface(psurf);
//     return hsurf;
// }
//
// BOOL
// APIENTRY
// EngAssociateSurface(
//     _In_ HSURF hsurf,
//     _In_ HDEV hdev,
//     _In_ FLONG flHooks)
// {
//     SURFOBJ *pso;
//     PSURFACE psurf;
//     PDEVOBJ* ppdev;
//     PPALETTE ppal;
//
//     ppdev = (PDEVOBJ*)hdev;
//
//     /* Lock the surface */
//     psurf = SURFACE_ShareLockSurface(hsurf);
//     if (!psurf)
//     {
//         return FALSE;
//     }
//     pso = &psurf->SurfObj;
//
//     /* Associate the hdev */
//     pso->hdev = hdev;
//     pso->dhpdev = ppdev->dhpdev;
//
//     /* Hook up specified functions */
//     psurf->flags &= ~HOOK_FLAGS;
//     psurf->flags |= (flHooks & HOOK_FLAGS);
//
//     /* Assign the PDEV's palette */
//     ppal = PALETTE_ShareLockPalette(ppdev->devinfo.hpalDefault);
//     SURFACE_vSetPalette(psurf, ppal);
//     PALETTE_ShareUnlockPalette(ppal);
//
//     SURFACE_ShareUnlockSurface(psurf);
//
//     return TRUE;
// }
//
// BOOL
// APIENTRY
// EngModifySurface(
//     _In_ HSURF hsurf,
//     _In_ HDEV hdev,
//     _In_ FLONG flHooks,
//     _In_ FLONG flSurface,
//     _In_ DHSURF dhsurf,
//     _In_ PVOID pvScan0,
//     _In_ LONG lDelta,
//     _Reserved_ PVOID pvReserved)
// {
//     SURFOBJ *pso;
//     PSURFACE psurf;
//     PDEVOBJ* ppdev;
//     PPALETTE ppal;
//
//     /* Lock the surface */
//     psurf = SURFACE_ShareLockSurface(hsurf);
//     if (psurf == NULL)
//     {
//         DPRINT1("Failed to reference surface %p\n", hsurf);
//         return FALSE;
//     }
//
//     ppdev = (PDEVOBJ*)hdev;
//     pso = &psurf->SurfObj;
//     pso->dhsurf = dhsurf;
//
//     /* Associate the hdev */
//     pso->hdev = hdev;
//     pso->dhpdev = ppdev->dhpdev;
//
//     /* Hook up specified functions */
//     psurf->flags &= ~HOOK_FLAGS;
//     psurf->flags |= (flHooks & HOOK_FLAGS);
//
//     /* Assign the PDEV's palette */
//     ppal = PALETTE_ShareLockPalette(ppdev->devinfo.hpalDefault);
//     SURFACE_vSetPalette(psurf, ppal);
//     PALETTE_ShareUnlockPalette(ppal);
//
//     /* Update surface flags */
//     if (flSurface & MS_NOTSYSTEMMEMORY)
//          pso->fjBitmap |= BMF_NOTSYSMEM;
//     else
//         pso->fjBitmap &= ~BMF_NOTSYSMEM;
//     if (flSurface & MS_SHAREDACCESS)
//          psurf->flags |= SHAREACCESS_SURFACE;
//     else
//         psurf->flags &= ~SHAREACCESS_SURFACE;
//
//     /* Check if the caller passed bitmap bits */
//     if ((pvScan0 != NULL) && (lDelta != 0))
//     {
//         /* Update the fields */
//         pso->pvScan0 = pvScan0;
//         pso->lDelta = lDelta;
//
//         /* This is a bitmap now! */
//         pso->iType = STYPE_BITMAP;
//
//         /* Check memory layout */
//         if (lDelta > 0)
//         {
//             /* Topdown is the normal way */
//             pso->cjBits = lDelta * pso->sizlBitmap.cy;
//             pso->pvBits = pso->pvScan0;
//             pso->fjBitmap |= BMF_TOPDOWN;
//         }
//         else
//         {
//             /* Inversed bitmap (bottom up) */
//             pso->cjBits = (-lDelta) * pso->sizlBitmap.cy;
//             pso->pvBits = (PCHAR)pso->pvScan0 - pso->cjBits - lDelta;
//             pso->fjBitmap &= ~BMF_TOPDOWN;
//         }
//     }
//     else
//     {
//         /* Set bits to NULL */
//         pso->pvBits = NULL;
//         pso->pvScan0 = NULL;
//         pso->lDelta = 0;
//
//         /* Set appropriate surface type */
//         if (pso->iType != STYPE_DEVICE)
//             pso->iType = STYPE_DEVBITMAP;
//     }
//
//     SURFACE_ShareUnlockSurface(psurf);
//
//     return TRUE;
// }
//
//
// BOOL
// APIENTRY
// EngDeleteSurface(
//     _In_ _Post_ptr_invalid_ HSURF hsurf)
// {
//     PSURFACE psurf;
//
//     psurf = SURFACE_ShareLockSurface(hsurf);
//     if (!psurf)
//     {
//         DPRINT1("Could not reference surface %p to delete\n", hsurf);
//         return FALSE;
//     }
//
//     GDIOBJ_vDeleteObject(&psurf->BaseObject);
//     return TRUE;
// }
//
// BOOL
// APIENTRY
// EngEraseSurface(
//     _In_ SURFOBJ *pso,
//     _In_ RECTL *prcl,
//     _In_ ULONG iColor)
// {
//     ASSERT(pso);
//     ASSERT(prcl);
//     return FillSolid(pso, prcl, iColor);
// }
//
// /*
//  * @implemented
//  */
// SURFOBJ * APIENTRY
// NtGdiEngLockSurface(IN HSURF hsurf)
// {
//     return EngLockSurface(hsurf);
// }
//
//
// SURFOBJ *
// APIENTRY
// EngLockSurface(
//     _In_ HSURF hsurf)
// {
//     SURFACE *psurf = SURFACE_ShareLockSurface(hsurf);
//
//     return psurf ? &psurf->SurfObj : NULL;
// }
//
// __kernel_entry
// NTSTATUS
// APIENTRY
// NtGdiEngUnlockSurface(
//     _In_ SURFOBJ *pso)
// {
//     UNIMPLEMENTED;
//     ASSERT(FALSE);
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// VOID
// APIENTRY
// EngUnlockSurface(
//     _In_ _Post_ptr_invalid_ SURFOBJ *pso)
// {
//     if (pso != NULL)
//     {
//         SURFACE *psurf = CONTAINING_RECORD(pso, SURFACE, SurfObj);
//         SURFACE_ShareUnlockSurface(psurf);
//     }
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
