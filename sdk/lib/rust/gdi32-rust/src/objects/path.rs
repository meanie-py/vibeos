//! objects/path.c → Rust. Path bracket/fill/stroke client API.
//
// gdi32.dll path client wrappers: AbortPath/BeginPath/CloseFigure/EndPath/FillPath/
// FlattenPath/GetPath/PathToRegion/SetMiterLimit/StrokeAndFillPath/StrokePath/
// WidenPath/SelectClipPath. Mostly thin WINAPI→NtGdi* wrappers fronted by the
// HANDLE_METADC0P / HANDLE_EMETAFDC macro fast-path (meta/EMF DC handling).
//
// NEEDS-TYPE: none — HDC/HRGN/HGDIOBJ/PLDC/BOOL/INT/DWORD/PDWORD/LPPOINT/LPBYTE/
//   PPOINTL/FLOAT/PFLOAT come from `win32k_sys::win32k`. PLDC.iType compared as INT
//   exactly as the landed painting.rs does (same crate/struct).
// NEEDS-IMPORT: kernel32 SetLastError (WINAPI → extern "system"); GdiGetLDC (gdi32
//   misc.rs, FASTCALL → extern "fastcall"); EMFDC_* path recorders (gdi32p.h, plain
//   cdecl → extern "C") — flagged for central confirmation they stay exported within
//   gdi32.dll (emfdc.c still C, or later cohab .rs).
// NEEDS-CHECK: PathToRegion uses HANDLE_METADC0P with _Func = `AbortPath` (so it calls
//   EMFDC_AbortPath, NOT EMFDC_PathToRegion) — faithful to the C source quirk.

use win32k_sys::win32k::*;
use crate::gdihelp::*;

// win32k_sys's FALSE/TRUE are bindgen u32 (#define), but BOOL is i32 — shadow with
// BOOL-typed locals so `return FALSE`/`return TRUE` type-check against i32 BOOL.
const FALSE: BOOL = 0;

// ── handle-type constants (ntgdihdl.h / GDILoObjType), as `usize` for compare ──
const LO_DC_TYPE: usize = GDILoObjType_GDILoObjType_LO_DC_TYPE as usize;
const LO_METADC16_TYPE: usize = GDILoObjType_GDILoObjType_LO_METADC16_TYPE as usize;

// ── kernel32 SetLastError (WINAPI → extern "system") ────────────────────────────
extern "system" {
    fn SetLastError(dwErrCode: DWORD);
}

// ── GdiGetLDC (FASTCALL → extern "fastcall", i386 `@GdiGetLDC@4`) ────────────────
extern "fastcall" {
    fn GdiGetLDC(hdc: HDC) -> PLDC;
}

// ── EMF metafile path recorders (gdi32p.h, plain cdecl → extern "C") ─────────────
extern "C" {
    fn EMFDC_AbortPath(dc_attr: PLDC) -> BOOL;
    fn EMFDC_BeginPath(dc_attr: PLDC) -> BOOL;
    fn EMFDC_CloseFigure(dc_attr: PLDC) -> BOOL;
    fn EMFDC_EndPath(dc_attr: PLDC) -> BOOL;
    fn EMFDC_FillPath(dc_attr: PLDC) -> BOOL;
    fn EMFDC_FlattenPath(dc_attr: PLDC) -> BOOL;
    fn EMFDC_StrokeAndFillPath(dc_attr: PLDC) -> BOOL;
    fn EMFDC_StrokePath(dc_attr: PLDC) -> BOOL;
    fn EMFDC_WidenPath(dc_attr: PLDC) -> BOOL;
    fn EMFDC_SelectClipPath(dc_attr: PLDC, mode: INT) -> BOOL;
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn AbortPath(hdc: HDC) -> BOOL {
    // HANDLE_METADC0P(BOOL, AbortPath, FALSE, hdc);
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != LO_DC_TYPE {
        let _ret: BOOL = FALSE;
        if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) == LO_METADC16_TYPE {
            return _ret;
        }
        let pLDC: PLDC = GdiGetLDC(hdc);
        if pLDC.is_null() {
            SetLastError(ERROR_INVALID_HANDLE);
            return _ret;
        }
        if (*pLDC).iType == LDC_EMFLDC as INT && EMFDC_AbortPath(pLDC) == 0 {
            return _ret;
        }
        // Fall through to support information DC's.
    }
    NtGdiAbortPath(hdc)
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn BeginPath(hdc: HDC) -> BOOL {
    // HANDLE_METADC0P(BOOL, BeginPath, FALSE, hdc);
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != LO_DC_TYPE {
        let _ret: BOOL = FALSE;
        if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) == LO_METADC16_TYPE {
            return _ret;
        }
        let pLDC: PLDC = GdiGetLDC(hdc);
        if pLDC.is_null() {
            SetLastError(ERROR_INVALID_HANDLE);
            return _ret;
        }
        if (*pLDC).iType == LDC_EMFLDC as INT && EMFDC_BeginPath(pLDC) == 0 {
            return _ret;
        }
        // Fall through to support information DC's.
    }
    NtGdiBeginPath(hdc)
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn CloseFigure(hdc: HDC) -> BOOL {
    // HANDLE_METADC0P(BOOL, CloseFigure, FALSE, hdc);
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != LO_DC_TYPE {
        let _ret: BOOL = FALSE;
        if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) == LO_METADC16_TYPE {
            return _ret;
        }
        let pLDC: PLDC = GdiGetLDC(hdc);
        if pLDC.is_null() {
            SetLastError(ERROR_INVALID_HANDLE);
            return _ret;
        }
        if (*pLDC).iType == LDC_EMFLDC as INT && EMFDC_CloseFigure(pLDC) == 0 {
            return _ret;
        }
        // Fall through to support information DC's.
    }
    NtGdiCloseFigure(hdc)
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn EndPath(hdc: HDC) -> BOOL {
    // HANDLE_METADC0P(BOOL, EndPath, FALSE, hdc);
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != LO_DC_TYPE {
        let _ret: BOOL = FALSE;
        if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) == LO_METADC16_TYPE {
            return _ret;
        }
        let pLDC: PLDC = GdiGetLDC(hdc);
        if pLDC.is_null() {
            SetLastError(ERROR_INVALID_HANDLE);
            return _ret;
        }
        if (*pLDC).iType == LDC_EMFLDC as INT && EMFDC_EndPath(pLDC) == 0 {
            return _ret;
        }
        // Fall through to support information DC's.
    }
    NtGdiEndPath(hdc)
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn FillPath(hdc: HDC) -> BOOL {
    // HANDLE_METADC0P(BOOL, FillPath, FALSE, hdc);
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != LO_DC_TYPE {
        let _ret: BOOL = FALSE;
        if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) == LO_METADC16_TYPE {
            return _ret;
        }
        let pLDC: PLDC = GdiGetLDC(hdc);
        if pLDC.is_null() {
            SetLastError(ERROR_INVALID_HANDLE);
            return _ret;
        }
        if (*pLDC).iType == LDC_EMFLDC as INT && EMFDC_FillPath(pLDC) == 0 {
            return _ret;
        }
        // Fall through to support information DC's.
    }
    NtGdiFillPath(hdc)
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn FlattenPath(hdc: HDC) -> BOOL {
    // HANDLE_METADC0P(BOOL, FlattenPath, FALSE, hdc);
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != LO_DC_TYPE {
        let _ret: BOOL = FALSE;
        if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) == LO_METADC16_TYPE {
            return _ret;
        }
        let pLDC: PLDC = GdiGetLDC(hdc);
        if pLDC.is_null() {
            SetLastError(ERROR_INVALID_HANDLE);
            return _ret;
        }
        if (*pLDC).iType == LDC_EMFLDC as INT && EMFDC_FlattenPath(pLDC) == 0 {
            return _ret;
        }
        // Fall through to support information DC's.
    }
    NtGdiFlattenPath(hdc)
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetPath(
    hdc: HDC,
    pptlBuf: LPPOINT,
    pjTypes: LPBYTE,
    cptBuf: INT,
) -> INT {
    let mut ret_value: INT = -1;

    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) == GDI_OBJECT_TYPE_METADC as usize {
        SetLastError(ERROR_INVALID_PARAMETER);
    } else {
        ret_value = NtGdiGetPath(hdc, pptlBuf as PPOINTL, pjTypes, cptBuf);
    }

    ret_value
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn PathToRegion(hdc: HDC) -> HRGN {
    // HANDLE_METADC0P(HRGN, AbortPath, NULL, hdc);  -- _Func is AbortPath in the C source.
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != LO_DC_TYPE {
        let _ret: HRGN = core::ptr::null_mut();
        if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) == LO_METADC16_TYPE {
            return _ret;
        }
        let pLDC: PLDC = GdiGetLDC(hdc);
        if pLDC.is_null() {
            SetLastError(ERROR_INVALID_HANDLE);
            return _ret;
        }
        if (*pLDC).iType == LDC_EMFLDC as INT && EMFDC_AbortPath(pLDC) == 0 {
            return _ret;
        }
        // Fall through to support information DC's.
    }
    NtGdiPathToRegion(hdc)
}

/*
 * @implemented
 *
 * SetMiterLimit(HDC, FLOAT a1, PFLOAT a2): the by-value FLOAT export case.
 * C does: worker.f = a1; NtGdiSetMiterLimit(hdc, worker.l, a2 ? &worker1.l : NULL);
 *         if (a2 && Ret) *a2 = worker1.f;  — pure bit-reinterpret, no float math.
 * Per-arch cfg split so the ABI matches the C export on every arch (a plain f32 param
 * fails to compile on the i386 -sse,-sse2 baseline; a plain u32 would corrupt the
 * amd64/arm64 vector-register float ABI). NtGdiSetMiterLimit(hdc, dwNew: DWORD, pdwOut: PDWORD).
 */
// i386: stdcall passes FLOAT as 4 bytes ON THE STACK — bit-identical to a u32 on the
// stack. Take the bits directly (they ARE worker.l); never materialize an f32 here.
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn SetMiterLimit(hdc: HDC, a1: u32 /*FLOAT bits = worker.l*/, a2: PFLOAT) -> BOOL {
    let mut worker1_l: DWORD = 0;
    let ret = NtGdiSetMiterLimit(
        hdc,
        a1 as DWORD,
        if !a2.is_null() { &mut worker1_l } else { core::ptr::null_mut() },
    );
    if !a2.is_null() && ret != 0 {
        // *a2 = worker1.f — store the raw bits (PFLOAT is *mut f32; write its u32 bits).
        *(a2 as *mut u32) = worker1_l as u32;
    }
    ret
}

// amd64/arm64: SSE/NEON present — FLOAT passes in a vector reg, so the native f32 sig
// is correct.
#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn SetMiterLimit(hdc: HDC, a1: FLOAT, a2: PFLOAT) -> BOOL {
    let mut worker1_l: DWORD = 0;
    let ret = NtGdiSetMiterLimit(
        hdc,
        a1.to_bits() as DWORD,
        if !a2.is_null() { &mut worker1_l } else { core::ptr::null_mut() },
    );
    if !a2.is_null() && ret != 0 {
        *a2 = f32::from_bits(worker1_l as u32);
    }
    ret
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn StrokeAndFillPath(hdc: HDC) -> BOOL {
    // HANDLE_METADC0P(BOOL, StrokeAndFillPath, FALSE, hdc);
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != LO_DC_TYPE {
        let _ret: BOOL = FALSE;
        if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) == LO_METADC16_TYPE {
            return _ret;
        }
        let pLDC: PLDC = GdiGetLDC(hdc);
        if pLDC.is_null() {
            SetLastError(ERROR_INVALID_HANDLE);
            return _ret;
        }
        if (*pLDC).iType == LDC_EMFLDC as INT && EMFDC_StrokeAndFillPath(pLDC) == 0 {
            return _ret;
        }
        // Fall through to support information DC's.
    }
    NtGdiStrokeAndFillPath(hdc)
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn StrokePath(hdc: HDC) -> BOOL {
    // HANDLE_METADC0P(BOOL, StrokePath, FALSE, hdc);
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != LO_DC_TYPE {
        let _ret: BOOL = FALSE;
        if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) == LO_METADC16_TYPE {
            return _ret;
        }
        let pLDC: PLDC = GdiGetLDC(hdc);
        if pLDC.is_null() {
            SetLastError(ERROR_INVALID_HANDLE);
            return _ret;
        }
        if (*pLDC).iType == LDC_EMFLDC as INT && EMFDC_StrokePath(pLDC) == 0 {
            return _ret;
        }
        // Fall through to support information DC's.
    }
    NtGdiStrokePath(hdc)
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn WidenPath(hdc: HDC) -> BOOL {
    // HANDLE_METADC0P(BOOL, WidenPath, FALSE, hdc);
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != LO_DC_TYPE {
        let _ret: BOOL = FALSE;
        if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) == LO_METADC16_TYPE {
            return _ret;
        }
        let pLDC: PLDC = GdiGetLDC(hdc);
        if pLDC.is_null() {
            SetLastError(ERROR_INVALID_HANDLE);
            return _ret;
        }
        if (*pLDC).iType == LDC_EMFLDC as INT && EMFDC_WidenPath(pLDC) == 0 {
            return _ret;
        }
        // Fall through to support information DC's.
    }
    NtGdiWidenPath(hdc)
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn SelectClipPath(hdc: HDC, iMode: INT) -> BOOL {
    // HANDLE_EMETAFDC(BOOL, SelectClipPath, FALSE, hdc, iMode);
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != LO_DC_TYPE {
        let _ret: BOOL = FALSE;
        if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) == LO_METADC16_TYPE {
            return _ret;
        }
        let pLDC: PLDC = GdiGetLDC(hdc);
        if pLDC.is_null() {
            SetLastError(ERROR_INVALID_HANDLE);
            return _ret;
        }
        if (*pLDC).iType == LDC_EMFLDC as INT && EMFDC_SelectClipPath(pLDC, iMode) == 0 {
            return _ret;
        }
        // Fall through to support information DC's.
    }
    NtGdiSelectClipPath(hdc, iMode)
}
