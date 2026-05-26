use win32k_sys::win32k::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// No file-scope declarations to translate - only a preprocessor directive
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn STROBJ_bEnum(
    pstro: *mut STROBJ,
    pc: *mut ULONG,
    ppgpos: *mut PGLYPHPOS,
) -> BOOL {
    0
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn STROBJ_bEnum(
    pstro: *mut STROBJ,
    pc: *mut ULONG,
    ppgpos: *mut PGLYPHPOS,
) -> BOOL {
    0
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn STROBJ_dwGetCodePage(
    pstro: *mut STROBJ
) -> u32 {
    let pStrGdi = pstro as PSTRGDI;
    (*pStrGdi).dwCodePage
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn STROBJ_dwGetCodePage(
    pstro: *mut STROBJ
) -> u32 {
    let pStrGdi = pstro as PSTRGDI;
    (*pStrGdi).dwCodePage
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn STROBJ_vEnumStart(
    pstro: *mut STROBJ) {
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn STROBJ_vEnumStart(
    pstro: *mut STROBJ) {
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn STROBJ_bEnumPositionsOnly(
    StringObj: *mut STROBJ,
    Count: *mut u32,
    Pos: *mut *mut _GLYPHPOS,  // PGLYPHPOS = *mut _GLYPHPOS; spelled out to disambiguate
) -> BOOL {
    DDI_ERROR as BOOL
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn STROBJ_bEnumPositionsOnly(
    StringObj: *mut STROBJ,
    Count: *mut u32,
    Pos: *mut *mut _GLYPHPOS,  // PGLYPHPOS = *mut _GLYPHPOS; spelled out to disambiguate
) -> BOOL {
    DDI_ERROR as BOOL
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn STROBJ_bGetAdvanceWidths(
    StringObj: *mut STROBJ,
    First: ULONG,
    Count: ULONG,
    Widths: *mut POINTQF,
) -> BOOL {
    0
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn STROBJ_bGetAdvanceWidths(
    StringObj: *mut STROBJ,
    First: ULONG,
    Count: ULONG,
    Widths: *mut POINTQF,
) -> BOOL {
    0
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn STROBJ_fxBreakExtra(
    StringObj: *mut STROBJ,
) -> FIX {
    let pStrGdi = StringObj as PSTRGDI;
    if ((*pStrGdi).StrObj.flAccel & SO_BREAK_EXTRA) != 0 {
        return (*pStrGdi).fxBreakExtra;
    }
    return 0 as FIX;
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn STROBJ_fxBreakExtra(
    StringObj: *mut STROBJ,
) -> FIX {
    let pStrGdi = StringObj as PSTRGDI;
    if ((*pStrGdi).StrObj.flAccel & SO_BREAK_EXTRA) != 0 {
        return (*pStrGdi).fxBreakExtra;
    }
    return 0 as FIX;
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn STROBJ_fxCharacterExtra(
    StringObj: *mut STROBJ,
) -> FIX {
    let pStrGdi = StringObj as PSTRGDI;
    if (*pStrGdi).StrObj.flAccel & SO_CHARACTER_EXTRA != 0 {
        return (*pStrGdi).fxExtra;
    }
    return 0 as FIX;
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn STROBJ_fxCharacterExtra(
    StringObj: *mut STROBJ,
) -> FIX {
    let pStrGdi = StringObj as PSTRGDI;
    if (*pStrGdi).StrObj.flAccel & SO_CHARACTER_EXTRA != 0 {
        return (*pStrGdi).fxExtra;
    }
    return 0 as FIX;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
//
// #include <win32k.h>
//
// #define NDEBUG
// #include <debug.h>
//
//
// BOOL
// APIENTRY
// STROBJ_bEnum(
//     IN STROBJ *pstro,
//     OUT ULONG *pc,
//     OUT PGLYPHPOS *ppgpos)
// {
//     // www.osr.com/ddk/graphics/gdifncs_65uv.htm
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// DWORD
// APIENTRY
// STROBJ_dwGetCodePage(
//     IN STROBJ *pstro)
// {
//     // www.osr.com/ddk/graphics/gdifncs_9jmv.htm
//     PSTRGDI pStrGdi = (PSTRGDI) pstro;
//     return pStrGdi->dwCodePage;
// }
//
// VOID
// APIENTRY
// STROBJ_vEnumStart(
//     IN STROBJ *pstro)
// {
//     // www.osr.com/ddk/graphics/gdifncs_32uf.htm
//     UNIMPLEMENTED;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// APIENTRY
// STROBJ_bEnumPositionsOnly(
//     IN STROBJ *StringObj,
//     OUT ULONG *Count,
//     OUT PGLYPHPOS *Pos)
// {
//     UNIMPLEMENTED;
//     return (BOOL) DDI_ERROR;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// APIENTRY
// STROBJ_bGetAdvanceWidths(
//     IN STROBJ *StringObj,
//     IN ULONG First,
//     IN ULONG Count,
//     OUT POINTQF *Widths)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// /*
//  * @implemented
//  */
// FIX
// APIENTRY
// STROBJ_fxBreakExtra(
//     IN STROBJ *StringObj)
// {
//     PSTRGDI pStrGdi = (PSTRGDI) StringObj;
//     if (pStrGdi->StrObj.flAccel & SO_BREAK_EXTRA) return pStrGdi->fxBreakExtra;
//     return (FIX) 0;
// }
//
// /*
//  * @implemented
//  */
// FIX
// APIENTRY
// STROBJ_fxCharacterExtra(
//     IN STROBJ *StringObj)
// {
//     PSTRGDI pStrGdi = (PSTRGDI) StringObj;
//     if (pStrGdi->StrObj.flAccel & SO_CHARACTER_EXTRA) return pStrGdi->fxExtra;
//     return (FIX)0;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
