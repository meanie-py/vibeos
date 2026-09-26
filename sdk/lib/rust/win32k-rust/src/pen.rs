use win32k_sys::win32k::*;

// Not in ntkrnl_sys::base; declared per-file, as the crate does elsewhere.
const STATUS_ACCESS_VIOLATION: i32 = 0xC0000005_u32 as i32;
#[allow(unused_imports)]
use core::ffi::c_void;
// bindgen models _BRUSH/_PEN as opaque (BaseObject + padding); the BRUSHBODY
// fields (lWidth/eWidth/ulPenStyle/pStyle/flAttrs/…) are reached via this view.
use crate::brushbody::pbrushbody;
use crate::floatobj::FLOATOBJ_SetLong;
use ntkrnl_sys::base::STATUS_SUCCESS;
use ntkrnl_sys::ddk::{ExAllocatePoolWithTag, ExFreePoolWithTag};

// SafeProbeForRead is NTAPI (win32ss/safe_user_io.c) — stdcall on i386.
extern "system" {
    fn SafeProbeForRead(address: *const core::ffi::c_void, length: usize, alignment: u32) -> i32;
}
// SetLastNtError is FASTCALL (win32k) — @-decorated on i386, cdecl elsewhere.
#[cfg(target_arch = "x86")]
extern "fastcall" { fn SetLastNtError(Status: NTSTATUS); }
#[cfg(not(target_arch = "x86"))]
extern "C" { fn SetLastNtError(Status: NTSTATUS); }

// NT_SUCCESS / GDI_HANDLE_GET_TYPE are C macros (gap functions); GDITAG_PENSTYLE
// is 'ytsG' (tags.h) — pool tags are stored little-endian, so b"Gsty".
#[inline(always)]
fn NT_SUCCESS(s: NTSTATUS) -> bool { s >= 0 }
#[inline(always)]
fn GDI_HANDLE_GET_TYPE(h: HPEN) -> u32 { ((h as usize) & 0x007f0000) as u32 }
const GDITAG_PENSTYLE: u32 = u32::from_le_bytes(*b"Gsty");

// PEN_UnlockPen(p)      = GDIOBJ_vUnlockObject((POBJ)p)        (pen.h macro)
// PEN_ShareUnlockPen(p) = GDIOBJ_vDereferenceObject((POBJ)p)  (pen.h macro)

unsafe fn PEN_vInit(ppen: PPEN) {
    let bb = pbrushbody(ppen);
    (*bb).pBrushAttr = core::ptr::addr_of_mut!((*bb).BrushAttr);
}

#[no_mangle]
pub unsafe extern "system" fn PEN_AllocPenWithHandle() -> PBRUSH {
    let ppen: PPEN = GDIOBJ_AllocObjWithHandle(GDILoObjType_LO_PEN_TYPE as _, core::mem::size_of::<PEN>() as _) as PPEN;
    if ppen.is_null() {
        return core::ptr::null_mut();
    }

    PEN_vInit(ppen);
    ppen
}

#[no_mangle]
pub unsafe extern "system" fn PEN_AllocExtPenWithHandle() -> PBRUSH {
    let ppen: PPEN = GDIOBJ_AllocObjWithHandle(GDILoObjType_LO_EXTPEN_TYPE as _, core::mem::size_of::<PEN>() as _) as PPEN;
    if ppen.is_null() {
        return core::ptr::null_mut();
    }

    PEN_vInit(ppen);
    ppen
}

#[cfg(target_pointer_width = "32")]
#[no_mangle]
pub unsafe extern "fastcall" fn PEN_ShareLockPen(hobj: HPEN) -> PBRUSH {
    if (GDI_HANDLE_GET_TYPE(hobj) != GDILoObjType_LO_PEN_TYPE)
        && (GDI_HANDLE_GET_TYPE(hobj) != GDILoObjType_LO_EXTPEN_TYPE)
    {
        return core::ptr::null_mut();
    }

    GDIOBJ_ReferenceObjectByHandle(hobj as _, GDIObjType_BRUSH_TYPE as _) as PBRUSH
}

#[cfg(not(target_pointer_width = "32"))]
#[no_mangle]
pub unsafe extern "C" fn PEN_ShareLockPen(hobj: HPEN) -> PBRUSH {
    if (GDI_HANDLE_GET_TYPE(hobj) != GDILoObjType_LO_PEN_TYPE)
        && (GDI_HANDLE_GET_TYPE(hobj) != GDILoObjType_LO_EXTPEN_TYPE)
    {
        return core::ptr::null_mut();
    }

    GDIOBJ_ReferenceObjectByHandle(hobj as _, GDIObjType_BRUSH_TYPE as _) as PBRUSH
}

#[no_mangle]
pub unsafe extern "system" fn IntGdiExtCreatePen(
    mut dwPenStyle: DWORD,
    mut dwWidth: DWORD,
    ulBrushStyle: ULONG,
    ulColor: ULONG,
    ulClientHatch: ULONG_PTR,
    _ulHatch: ULONG_PTR,
    dwStyleCount: DWORD,
    pStyle: PULONG,
    _cjDIB: ULONG,
    bOldStylePen: BOOL,
    _hbrush: HBRUSH,
) -> HPEN {
    static mut aulStyleAlternate: [ULONG; 2] = [1, 1];
    static mut aulStyleDash: [ULONG; 2] = [6, 2];
    static mut aulStyleDot: [ULONG; 2] = [1, 1];
    static mut aulStyleDashDot: [ULONG; 4] = [3, 2, 1, 2];
    static mut aulStyleDashDotDot: [ULONG; 6] = [3, 1, 1, 1, 1, 1];

    dwWidth = (dwWidth as i32).unsigned_abs() as DWORD;

    if (dwPenStyle & PS_STYLE_MASK) == PS_NULL {
        return (*(*(&raw const StockObjects)).as_ptr().add(NULL_PEN as usize)) as HPEN;
    }

    let pbrushPen: PBRUSH = if bOldStylePen != 0 {
        PEN_AllocPenWithHandle()
    } else {
        PEN_AllocExtPenWithHandle()
    };

    if pbrushPen.is_null() {
        EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return core::ptr::null_mut();
    }

    let hPen: HPEN = (*pbrushPen).BaseObject.hHmgr as HPEN;
    let bb = pbrushbody(pbrushPen);

    'cleanup: {
        if bOldStylePen != 0 {
            if dwWidth == 0 && (dwPenStyle & PS_STYLE_MASK) != PS_SOLID {
                dwWidth = 1;
            }
        } else {
            match dwPenStyle & PS_ENDCAP_MASK {
                x if x == PS_ENDCAP_ROUND || x == PS_ENDCAP_SQUARE || x == PS_ENDCAP_FLAT => {}
                _ => break 'cleanup,
            }

            match dwPenStyle & PS_JOIN_MASK {
                x if x == PS_JOIN_ROUND || x == PS_JOIN_BEVEL || x == PS_JOIN_MITER => {}
                _ => break 'cleanup,
            }

            match dwPenStyle & PS_TYPE_MASK {
                x if x == PS_COSMETIC => {
                    if dwWidth != 1 || ulBrushStyle != BS_SOLID {
                        break 'cleanup;
                    }
                }
                x if x == PS_GEOMETRIC => {}
                _ => break 'cleanup,
            }
        }

        (*bb).lWidth = dwWidth as _;
        FLOATOBJ_SetLong(core::ptr::addr_of_mut!((*bb).eWidth), (*bb).lWidth);
        (*bb).ulPenStyle = dwPenStyle;
        (*bb).BrushAttr.lbColor = ulColor;
        (*bb).iBrushStyle = ulBrushStyle as INT;
        (*bb).hbmClient = ulClientHatch as HBITMAP;
        (*bb).dwStyleCount = 0;
        (*bb).pStyle = core::ptr::null_mut();
        (*bb).ulStyleSize = 0;
        (*bb).flAttrs = if bOldStylePen != 0 { BR_IS_OLDSTYLEPEN } else { BR_IS_PEN };

        match dwPenStyle & PS_STYLE_MASK {
            x if x == PS_NULL => {
                (*bb).flAttrs |= BR_IS_NULL;
            }
            x if x == PS_SOLID => {
                (*bb).flAttrs |= BR_IS_SOLID;
            }
            x if x == PS_ALTERNATE => {
                (*bb).flAttrs |= BR_IS_SOLID | BR_IS_DEFAULTSTYLE;
                (*bb).pStyle = (*(&raw mut aulStyleAlternate)).as_mut_ptr();
                (*bb).dwStyleCount = (*(&raw const aulStyleAlternate)).len() as DWORD;
            }
            x if x == PS_DOT => {
                (*bb).flAttrs |= BR_IS_SOLID | BR_IS_DEFAULTSTYLE;
                (*bb).pStyle = (*(&raw mut aulStyleDot)).as_mut_ptr();
                (*bb).dwStyleCount = (*(&raw const aulStyleDot)).len() as DWORD;
            }
            x if x == PS_DASH => {
                (*bb).flAttrs |= BR_IS_SOLID | BR_IS_DEFAULTSTYLE;
                (*bb).pStyle = (*(&raw mut aulStyleDash)).as_mut_ptr();
                (*bb).dwStyleCount = (*(&raw const aulStyleDash)).len() as DWORD;
            }
            x if x == PS_DASHDOT => {
                (*bb).flAttrs |= BR_IS_SOLID | BR_IS_DEFAULTSTYLE;
                (*bb).pStyle = (*(&raw mut aulStyleDashDot)).as_mut_ptr();
                (*bb).dwStyleCount = (*(&raw const aulStyleDashDot)).len() as DWORD;
            }
            x if x == PS_DASHDOTDOT => {
                (*bb).flAttrs |= BR_IS_SOLID | BR_IS_DEFAULTSTYLE;
                (*bb).pStyle = (*(&raw mut aulStyleDashDotDot)).as_mut_ptr();
                (*bb).dwStyleCount = (*(&raw const aulStyleDashDotDot)).len() as DWORD;
            }
            x if x == PS_INSIDEFRAME => {
                (*bb).flAttrs |= BR_IS_SOLID | BR_IS_INSIDEFRAME;
            }
            x if x == PS_USERSTYLE => {
                let mut has_neg: BOOL = 0;
                let mut all_zero: BOOL = 1;
                let mut j: UINT = 0;
                while j < dwStyleCount && has_neg == 0 {
                    if (*pStyle.add(j as usize) as i32) < 0 {
                        has_neg = 1;
                    }
                    if *pStyle.add(j as usize) != 0 {
                        all_zero = 0;
                    }
                    j += 1;
                }

                if all_zero != 0 || has_neg != 0 {
                    break 'cleanup;
                }

                (*bb).flAttrs |= BR_IS_SOLID;
                (*bb).dwStyleCount = dwStyleCount;
                (*bb).pStyle = pStyle;
            }
            _ => {
                break 'cleanup;
            }
        }

        if !(*bb).pStyle.is_null() {
            let mut i: ULONG = 0;
            while i < (*bb).dwStyleCount {
                (*bb).ulStyleSize += *(*bb).pStyle.add(i as usize);
                i += 1;
            }
        }

        GDIOBJ_vUnlockObject(pbrushPen as POBJ);
        return hPen;
    }

    EngSetLastError(ERROR_INVALID_PARAMETER);
    (*bb).pStyle = core::ptr::null_mut();
    GDIOBJ_vDeleteObject(core::ptr::addr_of_mut!((*pbrushPen).BaseObject));

    core::ptr::null_mut()
}

#[cfg(target_pointer_width = "32")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntGdiSetSolidPenColor(hPen: HPEN, Color: COLORREF) {
    let pbrPen: PBRUSH = PEN_ShareLockPen(hPen);
    if !pbrPen.is_null() {
        let bb = pbrushbody(pbrPen);
        if ((*bb).flAttrs & BR_IS_SOLID) != 0 {
            (*bb).BrushAttr.lbColor = Color & 0xFFFFFF;
        }
        GDIOBJ_vDereferenceObject(pbrPen as POBJ);
    }
}

#[cfg(not(target_pointer_width = "32"))]
#[no_mangle]
pub unsafe extern "C" fn IntGdiSetSolidPenColor(hPen: HPEN, Color: COLORREF) {
    let pbrPen: PBRUSH = PEN_ShareLockPen(hPen);
    if !pbrPen.is_null() {
        let bb = pbrushbody(pbrPen);
        if ((*bb).flAttrs & BR_IS_SOLID) != 0 {
            (*bb).BrushAttr.lbColor = Color & 0xFFFFFF;
        }
        GDIOBJ_vDereferenceObject(pbrPen as POBJ);
    }
}

// APIENTRY = stdcall on i386 (extern "system"); cdecl elsewhere. gdiobj.rs declares it extern "system".
#[no_mangle]
pub unsafe extern "system" fn PEN_GetObject(pbrushPen: PBRUSH, cbCount: i32, pBuffer: PLOGPEN) -> i32 {
    let pLogPen: PLOGPEN;
    let pExtLogPen: PEXTLOGPEN;
    let cbRetCount: i32;
    let bb = pbrushbody(pbrushPen);

    if ((*bb).flAttrs & BR_IS_OLDSTYLEPEN) != 0 {
        cbRetCount = core::mem::size_of::<LOGPEN>() as i32;
        if !pBuffer.is_null() {
            if cbCount < cbRetCount {
                return 0;
            }

            if ((*bb).ulPenStyle & PS_STYLE_MASK) == PS_NULL
                && cbCount == core::mem::size_of::<EXTLOGPEN>() as i32
            {
                pExtLogPen = pBuffer as PEXTLOGPEN;
                (*pExtLogPen).elpPenStyle = (*bb).ulPenStyle;
                (*pExtLogPen).elpWidth = 0;
                (*pExtLogPen).elpBrushStyle = (*bb).iBrushStyle as u32;
                (*pExtLogPen).elpColor = (*bb).BrushAttr.lbColor;
                (*pExtLogPen).elpHatch = 0;
                (*pExtLogPen).elpNumEntries = 0;
                return core::mem::size_of::<EXTLOGPEN>() as i32;
            } else {
                pLogPen = pBuffer as PLOGPEN;
                (*pLogPen).lopnWidth.x = (*bb).lWidth;
                (*pLogPen).lopnWidth.y = 0;
                (*pLogPen).lopnStyle = (*bb).ulPenStyle as u32;
                (*pLogPen).lopnColor = (*bb).BrushAttr.lbColor;
            }
        }
    } else {
        let dwStyleCount: u32 = if ((*bb).flAttrs & BR_IS_DEFAULTSTYLE) != 0 {
            0
        } else {
            (*bb).dwStyleCount
        };
        cbRetCount = core::mem::size_of::<EXTLOGPEN>() as i32
            - core::mem::size_of::<u32>() as i32
            + (dwStyleCount as i32) * core::mem::size_of::<u32>() as i32;
        if !pBuffer.is_null() {
            if cbCount < cbRetCount {
                return 0;
            }
            pExtLogPen = pBuffer as PEXTLOGPEN;
            (*pExtLogPen).elpPenStyle = (*bb).ulPenStyle;
            (*pExtLogPen).elpWidth = (*bb).lWidth as u32;
            (*pExtLogPen).elpBrushStyle = (*bb).iBrushStyle as u32;
            (*pExtLogPen).elpColor = (*bb).BrushAttr.lbColor;
            (*pExtLogPen).elpHatch = (*bb).hbmClient as _;
            (*pExtLogPen).elpNumEntries = dwStyleCount;
            let mut i: u32 = 0;
            while i < dwStyleCount {
                let entry_ptr = core::ptr::addr_of_mut!((*pExtLogPen).elpStyleEntry) as *mut u32;
                *entry_ptr.offset(i as isize) = *(*bb).pStyle.offset(i as isize);
                i += 1;
            }
        }
    }

    cbRetCount
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiCreatePen(
    PenStyle: INT,
    Width: INT,
    Color: COLORREF,
    hbr: HBRUSH,
) -> HPEN {
    if (PenStyle < PS_SOLID as INT) || (PenStyle > PS_INSIDEFRAME as INT) {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return core::ptr::null_mut();
    }

    IntGdiExtCreatePen(
        PenStyle as DWORD,
        Width as DWORD,
        BS_SOLID as ULONG,
        Color as ULONG,
        0,
        0,
        0,
        core::ptr::null_mut(),
        0,
        1 as BOOL,
        hbr,
    )
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiExtCreatePen(
    dwPenStyle: DWORD,
    ulWidth: DWORD,
    ulBrushStyle: ULONG,
    mut ulColor: ULONG,
    ulClientHatch: ULONG_PTR,
    ulHatch: ULONG_PTR,
    mut dwStyleCount: DWORD,
    mut pUnsafeStyle: PULONG,
    cjDIB: ULONG,
    bOldStylePen: BOOL,
    hBrush: HBRUSH,
) -> HPEN {
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut pSafeStyle: *mut DWORD = core::ptr::null_mut();
    let hPen: HPEN;

    if (dwStyleCount as i32) < 0 {
        return 0 as HPEN;
    }
    if dwStyleCount > 16 {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return 0 as HPEN;
    }

    if ((dwPenStyle & PS_TYPE_MASK) == PS_COSMETIC) && (ulBrushStyle != BS_SOLID) {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return 0 as HPEN;
    }

    if ((dwPenStyle & PS_STYLE_MASK) == PS_NULL) || (ulBrushStyle == BS_NULL) {
        return (*(*(&raw const StockObjects)).as_ptr().add(NULL_PEN as usize)) as HPEN;
    }

    if (ulBrushStyle == BS_PATTERN)
        || (ulBrushStyle == BS_DIBPATTERN)
        || (ulBrushStyle == BS_DIBPATTERNPT)
    {
        ulColor = 0;
    } else if (ulBrushStyle != BS_SOLID) && (ulBrushStyle != BS_HATCHED) {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return 0 as HPEN;
    }

    if (dwPenStyle & PS_STYLE_MASK) != PS_USERSTYLE {
        dwStyleCount = 0;
        pUnsafeStyle = core::ptr::null_mut();
    }

    if dwStyleCount > 0 {
        if pUnsafeStyle.is_null() {
            EngSetLastError(ERROR_INVALID_PARAMETER);
            return 0 as HPEN;
        }

        pSafeStyle = ExAllocatePoolWithTag(
            NonPagedPool,
            ((dwStyleCount as usize) * core::mem::size_of::<DWORD>()) as SIZE_T,
            GDITAG_PENSTYLE,
        ) as *mut DWORD;
        if pSafeStyle.is_null() {
            SetLastNtError(ERROR_NOT_ENOUGH_MEMORY as NTSTATUS);
            return 0 as HPEN;
        }
        Status = SafeProbeForRead(
            pUnsafeStyle as PVOID,
            (dwStyleCount as usize) * core::mem::size_of::<DWORD>(),
            1,
        );
        // Validate-only probe; the capture is the access. A fault lands in Status,
        // which is checked below and frees pSafeStyle.
        if NT_SUCCESS(Status)
            && !crate::copy::capture_from_user(
                pSafeStyle as *mut core::ffi::c_void,
                pUnsafeStyle as *const core::ffi::c_void,
                (dwStyleCount as usize) * core::mem::size_of::<DWORD>(),
            )
        {
            Status = STATUS_ACCESS_VIOLATION;
        }
        if !NT_SUCCESS(Status) {
            SetLastNtError(Status);
            ExFreePoolWithTag(pSafeStyle as PVOID, GDITAG_PENSTYLE);
            return 0 as HPEN;
        }
    }

    if ulBrushStyle == BS_PATTERN {
        Status = SafeProbeForRead(ulHatch as PVOID, cjDIB as usize, 1);
        if !NT_SUCCESS(Status) {
            SetLastNtError(Status);
            if !pSafeStyle.is_null() {
                ExFreePoolWithTag(pSafeStyle as PVOID, GDITAG_PENSTYLE);
            }
            return 0 as HPEN;
        }
    }

    hPen = IntGdiExtCreatePen(
        dwPenStyle,
        ulWidth,
        ulBrushStyle,
        ulColor,
        ulClientHatch,
        ulHatch,
        dwStyleCount,
        pSafeStyle,
        cjDIB,
        bOldStylePen,
        hBrush,
    );

    if hPen.is_null() && !pSafeStyle.is_null() {
        ExFreePoolWithTag(pSafeStyle as PVOID, GDITAG_PENSTYLE);
    }

    hPen
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:         See COPYING in the top level directory
//  * PROJECT:           ReactOS kernel
//  * PURPOSE:           Pen functiona
//  * FILE:              win32ss/gdi/ntgdi/pen.c
//  * PROGRAMER:
//  */
//
// #include <win32k.h>
// #include <safe_user_io.h>
//
// #define NDEBUG
// #include <debug.h>
//
// /* PRIVATE FUNCTIONS **********************************************************/
//
// static
// VOID
// PEN_vInit(
//     PPEN ppen)
// {
//     /* Start with kmode brush attribute */
//     ppen->pBrushAttr = &ppen->BrushAttr;
// }
//
// PBRUSH
// NTAPI
// PEN_AllocPenWithHandle(
//     VOID)
// {
//     PPEN ppen;
//
//     ppen = (PBRUSH)GDIOBJ_AllocObjWithHandle(GDILoObjType_LO_PEN_TYPE, sizeof(PEN));
//     if (ppen == NULL)
//     {
//         return NULL;
//     }
//
//     PEN_vInit(ppen);
//     return ppen;
// }
//
// PBRUSH
// NTAPI
// PEN_AllocExtPenWithHandle(
//     VOID)
// {
//     PPEN ppen;
//
//     ppen = (PBRUSH)GDIOBJ_AllocObjWithHandle(GDILoObjType_LO_EXTPEN_TYPE, sizeof(PEN));
//     if (ppen == NULL)
//     {
//         return NULL;
//     }
//
//     PEN_vInit(ppen);
//     return ppen;
// }
//
// PBRUSH
// FASTCALL
// PEN_ShareLockPen(HPEN hobj)
// {
//     if ((GDI_HANDLE_GET_TYPE(hobj) != GDILoObjType_LO_PEN_TYPE) &&
//         (GDI_HANDLE_GET_TYPE(hobj) != GDILoObjType_LO_EXTPEN_TYPE))
//     {
//         return NULL;
//     }
//
//     return (PBRUSH)GDIOBJ_ReferenceObjectByHandle(hobj, GDIObjType_BRUSH_TYPE);
// }
//
// HPEN
// APIENTRY
// IntGdiExtCreatePen(
//     DWORD dwPenStyle,
//     DWORD dwWidth,
//     IN ULONG ulBrushStyle,
//     IN ULONG ulColor,
//     IN ULONG_PTR ulClientHatch,
//     IN ULONG_PTR ulHatch,
//     DWORD dwStyleCount,
//     PULONG pStyle,
//     IN ULONG cjDIB,
//     IN BOOL bOldStylePen,
//     IN OPTIONAL HBRUSH hbrush)
// {
//     HPEN hPen;
//     PBRUSH pbrushPen;
//     static ULONG aulStyleAlternate[] = { 1, 1 };
//     static ULONG aulStyleDash[] = { 6, 2 };
//     static ULONG aulStyleDot[] = { 1, 1 };
//     static ULONG aulStyleDashDot[] = { 3, 2, 1, 2 };
//     static ULONG aulStyleDashDotDot[] = { 3, 1, 1, 1, 1, 1 };
//     ULONG i;
//
//     dwWidth = abs(dwWidth);
//
//     if ( (dwPenStyle & PS_STYLE_MASK) == PS_NULL)
//     {
//         return StockObjects[NULL_PEN];
//     }
//
//     if (bOldStylePen)
//     {
//         pbrushPen = PEN_AllocPenWithHandle();
//     }
//     else
//     {
//         pbrushPen = PEN_AllocExtPenWithHandle();
//     }
//
//     if (!pbrushPen)
//     {
//         EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         DPRINT("Can't allocate pen\n");
//         return 0;
//     }
//
//     hPen = pbrushPen->BaseObject.hHmgr;
//
//     if (bOldStylePen)
//     {
//         // If nWidth is zero, the pen is a single pixel wide, regardless of the current transformation.
//         if (!dwWidth && (dwPenStyle & PS_STYLE_MASK) != PS_SOLID)
//             dwWidth = 1;
//     }
//     else
//     {
//         switch (dwPenStyle & PS_ENDCAP_MASK)
//         {
//             case PS_ENDCAP_ROUND:
//             case PS_ENDCAP_SQUARE:
//             case PS_ENDCAP_FLAT:
//                 break;
//
//             default:
//                 goto ExitCleanup;
//         }
//
//         switch (dwPenStyle & PS_JOIN_MASK)
//         {
//             case PS_JOIN_ROUND:
//             case PS_JOIN_BEVEL:
//             case PS_JOIN_MITER:
//                 break;
//
//             default:
//                 goto ExitCleanup;
//         }
//
//         switch (dwPenStyle & PS_TYPE_MASK)
//         {
//             case PS_COSMETIC:
//                 if (dwWidth != 1 || ulBrushStyle != BS_SOLID)
//                     goto ExitCleanup;
//
//                 break;
//
//             case PS_GEOMETRIC:
//                 break;
//
//             default:
//                 goto ExitCleanup;
//         }
//     }
//
//     pbrushPen->lWidth = dwWidth;
//     FLOATOBJ_SetLong(&pbrushPen->eWidth, pbrushPen->lWidth);
//     pbrushPen->ulPenStyle = dwPenStyle;
//     pbrushPen->BrushAttr.lbColor = ulColor;
//     pbrushPen->iBrushStyle = ulBrushStyle;
//     // FIXME: Copy the bitmap first ?
//     pbrushPen->hbmClient = (HANDLE)ulClientHatch;
//     pbrushPen->dwStyleCount = 0;
//     pbrushPen->pStyle = NULL;
//     pbrushPen->ulStyleSize = 0;
//     pbrushPen->flAttrs = bOldStylePen ? BR_IS_OLDSTYLEPEN : BR_IS_PEN;
//
//     switch (dwPenStyle & PS_STYLE_MASK)
//     {
//     case PS_NULL:
//         pbrushPen->flAttrs |= BR_IS_NULL;
//         break;
//
//     case PS_SOLID:
//         pbrushPen->flAttrs |= BR_IS_SOLID;
//         break;
//
//     case PS_ALTERNATE:
//         pbrushPen->flAttrs |= BR_IS_SOLID | BR_IS_DEFAULTSTYLE;
//         pbrushPen->pStyle = aulStyleAlternate;
//         pbrushPen->dwStyleCount = _countof(aulStyleAlternate);
//         break;
//
//     case PS_DOT:
//         pbrushPen->flAttrs |= BR_IS_SOLID | BR_IS_DEFAULTSTYLE;
//         pbrushPen->pStyle = aulStyleDot;
//         pbrushPen->dwStyleCount = _countof(aulStyleDot);
//         break;
//
//     case PS_DASH:
//         pbrushPen->flAttrs |= BR_IS_SOLID | BR_IS_DEFAULTSTYLE;
//         pbrushPen->pStyle = aulStyleDash;
//         pbrushPen->dwStyleCount = _countof(aulStyleDash);
//         break;
//
//     case PS_DASHDOT:
//         pbrushPen->flAttrs |= BR_IS_SOLID | BR_IS_DEFAULTSTYLE;
//         pbrushPen->pStyle = aulStyleDashDot;
//         pbrushPen->dwStyleCount = _countof(aulStyleDashDot);
//         break;
//
//     case PS_DASHDOTDOT:
//         pbrushPen->flAttrs |= BR_IS_SOLID | BR_IS_DEFAULTSTYLE;
//         pbrushPen->pStyle = aulStyleDashDotDot;
//         pbrushPen->dwStyleCount = _countof(aulStyleDashDotDot);
//         break;
//
//     case PS_INSIDEFRAME:
//         pbrushPen->flAttrs |= (BR_IS_SOLID | BR_IS_INSIDEFRAME);
//         break;
//
//     case PS_USERSTYLE:
//         {
//             UINT i;
//             BOOL has_neg = FALSE, all_zero = TRUE;
//
//             for(i = 0; (i < dwStyleCount) && !has_neg; i++)
//             {
//                 has_neg = has_neg || (((INT)(pStyle[i])) < 0);
//                 all_zero = all_zero && (pStyle[i] == 0);
//             }
//
//             if(all_zero || has_neg)
//             {
//                 goto ExitCleanup;
//             }
//         }
//         /* FIXME: What style here? */
//         pbrushPen->flAttrs |= BR_IS_SOLID;
//         pbrushPen->dwStyleCount = dwStyleCount;
//         pbrushPen->pStyle = pStyle;
//         break;
//
//     default:
//         DPRINT1("IntGdiExtCreatePen unknown penstyle %x\n", dwPenStyle);
//         goto ExitCleanup;
//     }
//
//     if (pbrushPen->pStyle != NULL)
//     {
//         for (i = 0; i < pbrushPen->dwStyleCount; i++)
//         {
//             pbrushPen->ulStyleSize += pbrushPen->pStyle[i];
//         }
//     }
//
//     NT_ASSERT((pbrushPen->dwStyleCount == 0) || (pbrushPen->pStyle != NULL));
//
//     PEN_UnlockPen(pbrushPen);
//     return hPen;
//
// ExitCleanup:
//     EngSetLastError(ERROR_INVALID_PARAMETER);
//     pbrushPen->pStyle = NULL;
//     GDIOBJ_vDeleteObject(&pbrushPen->BaseObject);
//
//     return NULL;
// }
//
// VOID
// FASTCALL
// IntGdiSetSolidPenColor(HPEN hPen, COLORREF Color)
// {
//     PBRUSH pbrPen;
//
//     pbrPen = PEN_ShareLockPen(hPen);
//     if (pbrPen)
//     {
//         if (pbrPen->flAttrs & BR_IS_SOLID)
//         {
//             pbrPen->BrushAttr.lbColor = Color & 0xFFFFFF;
//         }
//         GDIOBJ_vDereferenceObject(pbrPen as POBJ);
//     }
// }
//
// INT
// APIENTRY
// PEN_GetObject(PBRUSH pbrushPen, INT cbCount, PLOGPEN pBuffer)
// {
//     PLOGPEN pLogPen;
//     PEXTLOGPEN pExtLogPen;
//     INT cbRetCount;
//
//     if (pbrushPen->flAttrs & BR_IS_OLDSTYLEPEN)
//     {
//         cbRetCount = sizeof(LOGPEN);
//         if (pBuffer)
//         {
//             if (cbCount < cbRetCount) return 0;
//
//             if (((pbrushPen->ulPenStyle & PS_STYLE_MASK) == PS_NULL) &&
//                 (cbCount == sizeof(EXTLOGPEN)))
//             {
//                 pExtLogPen = (PEXTLOGPEN)pBuffer;
//                 pExtLogPen->elpPenStyle = pbrushPen->ulPenStyle;
//                 pExtLogPen->elpWidth = 0;
//                 pExtLogPen->elpBrushStyle = pbrushPen->iBrushStyle;
//                 pExtLogPen->elpColor = pbrushPen->BrushAttr.lbColor;
//                 pExtLogPen->elpHatch = 0;
//                 pExtLogPen->elpNumEntries = 0;
//                 cbRetCount = sizeof(EXTLOGPEN);
//             }
//             else
//             {
//                 pLogPen = (PLOGPEN)pBuffer;
//                 pLogPen->lopnWidth.x = pbrushPen->lWidth;
//                 pLogPen->lopnWidth.y = 0;
//                 pLogPen->lopnStyle = pbrushPen->ulPenStyle;
//                 pLogPen->lopnColor = pbrushPen->BrushAttr.lbColor;
//             }
//         }
//     }
//     else
//     {
//         DWORD dwStyleCount = (pbrushPen->flAttrs & BR_IS_DEFAULTSTYLE) ?
//             0 : pbrushPen->dwStyleCount;
//         cbRetCount = sizeof(EXTLOGPEN) - sizeof(DWORD) + dwStyleCount * sizeof(DWORD);
//         if (pBuffer)
//         {
//             ULONG i;
//
//             if (cbCount < cbRetCount) return 0;
//             pExtLogPen = (PEXTLOGPEN)pBuffer;
//             pExtLogPen->elpPenStyle = pbrushPen->ulPenStyle;
//             pExtLogPen->elpWidth = pbrushPen->lWidth;
//             pExtLogPen->elpBrushStyle = pbrushPen->iBrushStyle;
//             pExtLogPen->elpColor = pbrushPen->BrushAttr.lbColor;
//             pExtLogPen->elpHatch = (ULONG_PTR)pbrushPen->hbmClient;
//             pExtLogPen->elpNumEntries = dwStyleCount;
//             for (i = 0; i < dwStyleCount; i++)
//             {
//                 pExtLogPen->elpStyleEntry[i] = pbrushPen->pStyle[i];
//             }
//         }
//     }
//
//     return cbRetCount;
// }
//
//
// /* PUBLIC FUNCTIONS ***********************************************************/
//
// HPEN
// APIENTRY
// NtGdiCreatePen(
//     INT PenStyle,
//     INT Width,
//     COLORREF Color,
//     IN HBRUSH hbr)
// {
//     if ((PenStyle < PS_SOLID) ||( PenStyle > PS_INSIDEFRAME))
//     {
//         EngSetLastError(ERROR_INVALID_PARAMETER);
//         return NULL;
//     }
//
//     return IntGdiExtCreatePen(PenStyle,
//                               Width,
//                               BS_SOLID,
//                               Color,
//                               0,
//                               0,
//                               0,
//                               NULL,
//                               0,
//                               TRUE,
//                               hbr);
// }
//
// HPEN
// APIENTRY
// NtGdiExtCreatePen(
//     DWORD dwPenStyle,
//     DWORD ulWidth,
//     IN ULONG ulBrushStyle,
//     IN ULONG ulColor,
//     IN ULONG_PTR ulClientHatch,
//     IN ULONG_PTR ulHatch,
//     DWORD dwStyleCount,
//     PULONG pUnsafeStyle,
//     IN ULONG cjDIB,
//     IN BOOL bOldStylePen,
//     IN OPTIONAL HBRUSH hBrush)
// {
//     NTSTATUS Status = STATUS_SUCCESS;
//     DWORD* pSafeStyle = NULL;
//     HPEN hPen;
//
//     if ((int)dwStyleCount < 0) return 0;
//     if (dwStyleCount > 16)
//     {
//         EngSetLastError(ERROR_INVALID_PARAMETER);
//         return 0;
//     }
//
//     if (((dwPenStyle & PS_TYPE_MASK) == PS_COSMETIC) &&
//         (ulBrushStyle != BS_SOLID))
//     {
//         EngSetLastError(ERROR_INVALID_PARAMETER);
//         return 0;
//     }
//
//     if (((dwPenStyle & PS_STYLE_MASK) == PS_NULL) ||
//         (ulBrushStyle == BS_NULL))
//     {
//         return StockObjects[NULL_PEN];
//     }
//
//
//     if ((ulBrushStyle == BS_PATTERN) ||
//         (ulBrushStyle == BS_DIBPATTERN) ||
//         (ulBrushStyle == BS_DIBPATTERNPT))
//     {
//         ulColor = 0;
//     }
//     else if ((ulBrushStyle != BS_SOLID) &&
//              (ulBrushStyle != BS_HATCHED))
//     {
//         EngSetLastError(ERROR_INVALID_PARAMETER);
//         return 0;
//     }
//
//     if ((dwPenStyle & PS_STYLE_MASK) != PS_USERSTYLE)
//     {
//         dwStyleCount = 0;
//         pUnsafeStyle = NULL;
//     }
//
//     if (dwStyleCount > 0)
//     {
//         if (pUnsafeStyle == NULL)
//         {
//             EngSetLastError(ERROR_INVALID_PARAMETER);
//             return 0;
//         }
//
//         pSafeStyle = ExAllocatePoolWithTag(NonPagedPool,
//                                            dwStyleCount * sizeof(DWORD),
//                                            GDITAG_PENSTYLE);
//         if (!pSafeStyle)
//         {
//             SetLastNtError(ERROR_NOT_ENOUGH_MEMORY);
//             return 0;
//         }
//         Status = SafeProbeForRead(pUnsafeStyle, dwStyleCount * sizeof(DWORD), 1);
//         if (NT_SUCCESS(Status))
//         {
//             RtlCopyMemory(pSafeStyle,
//                           pUnsafeStyle,
//                           dwStyleCount * sizeof(DWORD));
//         }
//         if(!NT_SUCCESS(Status))
//         {
//             SetLastNtError(Status);
//             ExFreePoolWithTag(pSafeStyle, GDITAG_PENSTYLE);
//             return 0;
//         }
//     }
//
//     if (ulBrushStyle == BS_PATTERN)
//     {
//         Status = SafeProbeForRead((PVOID)ulHatch, cjDIB, 1);
//         if(!NT_SUCCESS(Status))
//         {
//             SetLastNtError(Status);
//             if (pSafeStyle) ExFreePoolWithTag(pSafeStyle, GDITAG_PENSTYLE);
//             return 0;
//         }
//     }
//
//     hPen = IntGdiExtCreatePen(dwPenStyle,
//                               ulWidth,
//                               ulBrushStyle,
//                               ulColor,
//                               ulClientHatch,
//                               ulHatch,
//                               dwStyleCount,
//                               pSafeStyle,
//                               cjDIB,
//                               bOldStylePen,
//                               hBrush);
//
//     if (!hPen && pSafeStyle)
//     {
//         ExFreePoolWithTag(pSafeStyle, GDITAG_PENSTYLE);
//     }
//
//     return hPen;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
