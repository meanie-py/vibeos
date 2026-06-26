//! misc/historic.c → Rust. Legacy/compat GDI client APIs.
//!
//! GDI32.DLL stubs: APIs that do basically nothing, but exist for backwards
//! compatibility with older Windows. Every body here is a thin pass-through or a
//! fixed return value (all @implemented in the C source).
//!
//! NEEDS-CHECK: `EngQueryEMFInfo` is ALSO declared as an `extern fn` in the
//!   win32k_sys bindgen surface (bindgen_i386.rs:78299). The `#[no_mangle]`
//!   definition below provides the gdi32.dll export (the C historic.c stub); the
//!   bindgen item is only a declaration, so there's no Rust-level clash, but the
//!   integrator should confirm gdi32.dll really exports this symbol (it does in
//!   ReactOS — gdi32.spec lists EngQueryEMFInfo).
//! NEEDS-CHECK: `DdSwapTextureHandles` returns DWORD but the C body returns the
//!   BOOL literal `TRUE` (== 1). Preserved verbatim as `TRUE as DWORD`.

use win32k_sys::win32k::*;
use crate::gdihelp::*;

// win32k_sys FALSE/TRUE are u32; BOOL is i32. Provide BOOL-typed shadows so the
// `return TRUE`/`return FALSE` bodies type-check (these shadow the glob imports).
const TRUE: BOOL = 1;
const FALSE: BOOL = 0;

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn EngQueryEMFInfo(hdev: HDEV, pEMFInfo: *mut EMFINFO) -> BOOL {
    let _ = (hdev, pEMFInfo);
    FALSE
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GdiPlayDCScript(
    a0: DWORD,
    a1: DWORD,
    a2: DWORD,
    a3: DWORD,
    a4: DWORD,
    a5: DWORD,
) -> BOOL {
    /* FIXME fix the prototype right */
    let _ = (a0, a1, a2, a3, a4, a5);
    FALSE
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GdiPlayJournal(
    a0: DWORD,
    a1: DWORD,
    a2: DWORD,
    a3: DWORD,
    a4: DWORD,
) -> BOOL {
    /* FIXME fix the prototype right */
    let _ = (a0, a1, a2, a3, a4);
    FALSE
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GdiPlayScript(
    a0: DWORD,
    a1: DWORD,
    a2: DWORD,
    a3: DWORD,
    a4: DWORD,
    a5: DWORD,
    a6: DWORD,
) -> BOOL {
    /* FIXME fix the prototype right */
    let _ = (a0, a1, a2, a3, a4, a5, a6);
    FALSE
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GdiConvertBitmap(hbm: HBITMAP) -> HBITMAP {
    /* Note Windows 2000/XP/VISTA always returns hbm */
    hbm
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GdiConvertBrush(hbr: HBRUSH) -> HBRUSH {
    /* Note Windows 2000/XP/VISTA always returns hbr */
    hbr
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GdiConvertDC(hdc: HDC) -> HDC {
    /* Note Windows 2000/XP/VISTA always returns hdc */
    hdc
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GdiConvertFont(hfont: HFONT) -> HFONT {
    /* Note Windows 2000/XP/VISTA always returns hfont */
    hfont
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GdiConvertPalette(hpal: HPALETTE) -> HPALETTE {
    /* Note Windows 2000/XP/VISTA always returns hpal */
    hpal
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GdiConvertRegion(hregion: HRGN) -> HRGN {
    /* Note Windows 2000/XP/VISTA always returns hregion */
    hregion
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GdiSetAttrs(hdc: HDC) -> BOOL {
    /* Note Windows 2000/XP/VISTA always returns TRUE */
    let _ = hdc;
    TRUE
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GdiDeleteLocalDC(hdc: HDC) -> BOOL {
    /* Note Windows 2000/XP/VISTA always returns TRUE */
    let _ = hdc;
    TRUE
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GdiSetServerAttr(hdc: HDC, attr: DWORD) {
    /* it does do nothing */
    let _ = (hdc, attr);
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn DeviceCapabilitiesExA(
    pDevice: LPCSTR,
    pPort: LPCSTR,
    fwCapability: WORD,
    pOutput: LPSTR,
    pDevMode: *const DEVMODEA,
) -> INT {
    /* Note Windows 2000/XP/VISTA always returns -1 */
    let _ = (pDevice, pPort, fwCapability, pOutput, pDevMode);
    -1
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn DeviceCapabilitiesExW(
    pDevice: LPCWSTR,
    pPort: LPCWSTR,
    fwCapability: WORD,
    pOutput: LPWSTR,
    pDevMode: *const DEVMODEW,
) -> INT {
    /* Note Windows 2000/XP/VISTA always returns -1 */
    let _ = (pDevice, pPort, fwCapability, pOutput, pDevMode);
    -1
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn FixBrushOrgEx(
    hDC: HDC,
    nXOrg: INT,
    nYOrg: INT,
    lpPoint: LPPOINT,
) -> BOOL {
    /* Note Windows 2000/XP/VISTA always returns FALSE */
    let _ = (hDC, nXOrg, nYOrg, lpPoint);
    FALSE
}

/*
 * @implemented
 *
 * GDIEntry 16
 */
#[no_mangle]
pub unsafe extern "system" fn DdSwapTextureHandles(
    pDDraw: LPDDRAWI_DIRECTDRAW_LCL,
    pDDSLcl1: LPDDRAWI_DDRAWSURFACE_LCL,
    pDDSLcl2: LPDDRAWI_DDRAWSURFACE_LCL,
) -> DWORD {
    /* Note Windows 2000/XP/VISTA always returns success */
    let _ = (pDDraw, pDDSLcl1, pDDSLcl2);
    TRUE as DWORD
}

/// gdi32.spec exports ordinal 273 `GdiEntry16` (stdcall, 3 ptr args), which the C historic.c
/// TU provided as an obj-level alias of `DdSwapTextureHandles` (the "GDIEntry 16" comment above).
/// The cohab flip drops that hidden alias, so re-provide it explicitly as an export forwarding to
/// DdSwapTextureHandles — `_GdiEntry16@12` on i386 / `GdiEntry16` on amd64/arm64 (all-arch; the
/// spec export is arch-independent). NB: gdientry.c's GdiEntry1..15 are aliased the same way →
/// Wave 4 will need equivalent explicit exports for the DdXxx entries.
#[no_mangle]
pub unsafe extern "system" fn GdiEntry16(
    pDDraw: LPDDRAWI_DIRECTDRAW_LCL,
    pDDSLcl1: LPDDRAWI_DDRAWSURFACE_LCL,
    pDDSLcl2: LPDDRAWI_DDRAWSURFACE_LCL,
) -> DWORD {
    DdSwapTextureHandles(pDDraw, pDDSLcl1, pDDSLcl2)
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GdiReleaseLocalDC(hdc: HDC) -> BOOL {
    /* Note Windows 2000/XP/VISTA always returns TRUE */
    let _ = hdc;
    TRUE
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn SelectBrushLocal(Currenthbm: HBRUSH, Newhbm: HBRUSH) -> HBRUSH {
    let _ = Currenthbm;
    Newhbm
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn SelectFontLocal(Currenthfnt: HFONT, newhfnt: HFONT) -> HFONT {
    let _ = Currenthfnt;
    newhfnt
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GdiGetLocalBrush(hbr: HBRUSH) -> HBRUSH {
    hbr
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GdiGetLocalDC(hdc: HDC) -> HDC {
    hdc
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GdiGetLocalFont(hfont: HFONT) -> HFONT {
    hfont
}
