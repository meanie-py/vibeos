//! win32ss/gdi/gdi32/objects/arc.c → Rust. GDI arc client wrappers:
//! Arc / AngleArc / ArcTo / Chord / Pie (metadc/emf-dc dispatch + NtGdiArcInternal).
//
// NEEDS-TYPE: none — all scalar aliases + GDI types come from `win32k_sys::win32k`
//   (BOOL/INT/DWORD/FLOAT/HDC/PLDC).
// NEEDS-IMPORT (cross-file gdi32-private, resolved within gdi32.dll):
//   include/gdi32p.h  HDC  WINAPI    GdiConvertAndCheckDC(HDC)            → extern "system"
//   include/gdi32p.h  PLDC FASTCALL  GdiGetLDC(HDC)                       → extern "fastcall"
//   include/gdi32p.h  BOOL (cdecl)   METADC_Arc/Chord/Pie(HDC,...)        → extern "C"
//   include/gdi32p.h  BOOL (cdecl)   EMFDC_ArcChordPie(LDC*,...,DWORD)    → extern "C"
//   include/gdi32p.h  BOOL (cdecl)   EMFDC_AngleArc(LDC*,INT,INT,DWORD,FLOAT,FLOAT) → extern "C"
//   kernel32          VOID WINAPI    SetLastError(DWORD)                  → extern "system"
//   EMFDC_Arc/ArcTo/Chord/Pie are gdi32p.h FORCEINLINE → reproduced as local #[inline].
//
// NOTE: arc.c contains exactly 5 functions — Arc, AngleArc, ArcTo, Chord, Pie.
//   (SetArcDirection/GetArcDirection are NOT in this file; they live elsewhere.)

use win32k_sys::win32k::*;
use crate::gdihelp::*;

// win32k_sys FALSE/TRUE are bindgen u32; BOOL is i32 — BOOL-typed shadows.
const FALSE: BOOL = 0;

extern "system" {
    fn GdiConvertAndCheckDC(hdc: HDC) -> HDC;
    fn SetLastError(dwErrCode: DWORD);
}

#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn GdiGetLDC(hdc: HDC) -> PLDC;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn GdiGetLDC(hdc: HDC) -> PLDC;
}

extern "C" {
    fn METADC_Arc(
        hdc: HDC,
        left: INT,
        top: INT,
        right: INT,
        bottom: INT,
        xstart: INT,
        ystart: INT,
        xend: INT,
        yend: INT,
    ) -> BOOL;
    fn METADC_Chord(
        hdc: HDC,
        left: INT,
        top: INT,
        right: INT,
        bottom: INT,
        xstart: INT,
        ystart: INT,
        xend: INT,
        yend: INT,
    ) -> BOOL;
    fn METADC_Pie(
        hdc: HDC,
        left: INT,
        top: INT,
        right: INT,
        bottom: INT,
        xstart: INT,
        ystart: INT,
        xend: INT,
        yend: INT,
    ) -> BOOL;
    fn EMFDC_ArcChordPie(
        dc_attr: PLDC,
        left: INT,
        top: INT,
        right: INT,
        bottom: INT,
        xstart: INT,
        ystart: INT,
        xend: INT,
        yend: INT,
        type_: DWORD,
    ) -> BOOL;
    fn EMFDC_AngleArc(
        dc_attr: PLDC,
        x: INT,
        y: INT,
        radius: DWORD,
        start: FLOAT,
        sweep: FLOAT,
    ) -> BOOL;
}

// ── gdi32p.h FORCEINLINE EMFDC_{Arc,ArcTo,Chord,Pie} → EMFDC_ArcChordPie(.., EMR_*) ──
#[inline]
unsafe fn EMFDC_Arc(
    dc_attr: PLDC,
    left: INT,
    top: INT,
    right: INT,
    bottom: INT,
    xstart: INT,
    ystart: INT,
    xend: INT,
    yend: INT,
) -> BOOL {
    EMFDC_ArcChordPie(dc_attr, left, top, right, bottom, xstart, ystart, xend, yend, EMR_ARC)
}

#[inline]
unsafe fn EMFDC_ArcTo(
    dc_attr: PLDC,
    left: INT,
    top: INT,
    right: INT,
    bottom: INT,
    xstart: INT,
    ystart: INT,
    xend: INT,
    yend: INT,
) -> BOOL {
    EMFDC_ArcChordPie(dc_attr, left, top, right, bottom, xstart, ystart, xend, yend, EMR_ARCTO)
}

#[inline]
unsafe fn EMFDC_Chord(
    dc_attr: PLDC,
    left: INT,
    top: INT,
    right: INT,
    bottom: INT,
    xstart: INT,
    ystart: INT,
    xend: INT,
    yend: INT,
) -> BOOL {
    EMFDC_ArcChordPie(dc_attr, left, top, right, bottom, xstart, ystart, xend, yend, EMR_CHORD)
}

#[inline]
unsafe fn EMFDC_Pie(
    dc_attr: PLDC,
    left: INT,
    top: INT,
    right: INT,
    bottom: INT,
    xstart: INT,
    ystart: INT,
    xend: INT,
    yend: INT,
) -> BOOL {
    EMFDC_ArcChordPie(dc_attr, left, top, right, bottom, xstart, ystart, xend, yend, EMR_PIE)
}

#[no_mangle]
pub unsafe extern "system" fn Arc(
    hdc: HDC,
    xLeft: INT,
    yTop: INT,
    xRight: INT,
    yBottom: INT,
    xStartArc: INT,
    yStartArc: INT,
    xEndArc: INT,
    yEndArc: INT,
) -> BOOL {
    // HANDLE_METADC(BOOL, Arc, FALSE, hdc, ...)
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != GDILoObjType_LO_DC_TYPE as usize {
        if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) == GDILoObjType_LO_METADC16_TYPE as usize {
            return METADC_Arc(
                hdc, xLeft, yTop, xRight, yBottom, xStartArc, yStartArc, xEndArc, yEndArc,
            ) as BOOL;
        } else {
            let pLDC: PLDC = GdiGetLDC(hdc);
            let _Ret: BOOL = FALSE;
            if pLDC.is_null() {
                SetLastError(ERROR_INVALID_HANDLE);
                return _Ret;
            }
            if (*pLDC).iType == LDC_EMFLDC as INT
                && EMFDC_Arc(
                    pLDC, xLeft, yTop, xRight, yBottom, xStartArc, yStartArc, xEndArc, yEndArc,
                ) == 0
            {
                return _Ret;
            }
            /* Fall through to support information DC's. */
        }
    }

    if GdiConvertAndCheckDC(hdc).is_null() {
        return FALSE;
    }

    NtGdiArcInternal(
        _ARCTYPE_GdiTypeArc,
        hdc,
        xLeft,
        yTop,
        xRight,
        yBottom,
        xStartArc,
        yStartArc,
        xEndArc,
        yEndArc,
    )
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn AngleArc(
    hdc: HDC,
    x: INT,
    y: INT,
    dwRadius: DWORD,
    eStartAngle: FLOAT,
    eSweepAngle: FLOAT,
) -> BOOL {
    // HANDLE_EMETAFDC(BOOL, AngleArc, FALSE, hdc, ...)
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != GDILoObjType_LO_DC_TYPE as usize {
        let pLDC: PLDC;
        let _Ret: BOOL = FALSE;
        if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) == GDILoObjType_LO_METADC16_TYPE as usize {
            return _Ret;
        }
        pLDC = GdiGetLDC(hdc);
        if pLDC.is_null() {
            SetLastError(ERROR_INVALID_HANDLE);
            return _Ret;
        }
        if (*pLDC).iType == LDC_EMFLDC as INT
            && EMFDC_AngleArc(pLDC, x, y, dwRadius, eStartAngle, eSweepAngle) == 0
        {
            return _Ret;
        }
        /* Fall through to support information DC's. */
    }

    if GdiConvertAndCheckDC(hdc).is_null() {
        return FALSE;
    }

    NtGdiAngleArc(
        hdc,
        x,
        y,
        dwRadius,
        eStartAngle.to_bits() as DWORD,
        eSweepAngle.to_bits() as DWORD,
    )
}

#[no_mangle]
pub unsafe extern "system" fn ArcTo(
    hdc: HDC,
    xLeft: INT,
    yTop: INT,
    xRight: INT,
    yBottom: INT,
    xRadial1: INT,
    yRadial1: INT,
    xRadial2: INT,
    yRadial2: INT,
) -> BOOL {
    // HANDLE_EMETAFDC(BOOL, ArcTo, FALSE, hdc, ...)
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != GDILoObjType_LO_DC_TYPE as usize {
        let pLDC: PLDC;
        let _Ret: BOOL = FALSE;
        if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) == GDILoObjType_LO_METADC16_TYPE as usize {
            return _Ret;
        }
        pLDC = GdiGetLDC(hdc);
        if pLDC.is_null() {
            SetLastError(ERROR_INVALID_HANDLE);
            return _Ret;
        }
        if (*pLDC).iType == LDC_EMFLDC as INT
            && EMFDC_ArcTo(
                pLDC, xLeft, yTop, xRight, yBottom, xRadial1, yRadial1, xRadial2, yRadial2,
            ) == 0
        {
            return _Ret;
        }
        /* Fall through to support information DC's. */
    }

    if GdiConvertAndCheckDC(hdc).is_null() {
        return FALSE;
    }

    NtGdiArcInternal(
        _ARCTYPE_GdiTypeArcTo,
        hdc,
        xLeft,
        yTop,
        xRight,
        yBottom,
        xRadial1,
        yRadial1,
        xRadial2,
        yRadial2,
    )
}

#[no_mangle]
pub unsafe extern "system" fn Chord(
    hdc: HDC,
    xLeft: INT,
    yTop: INT,
    xRight: INT,
    yBottom: INT,
    xRadial1: INT,
    yRadial1: INT,
    xRadial2: INT,
    yRadial2: INT,
) -> BOOL {
    // HANDLE_METADC(BOOL, Chord, FALSE, hdc, ...)
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != GDILoObjType_LO_DC_TYPE as usize {
        if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) == GDILoObjType_LO_METADC16_TYPE as usize {
            return METADC_Chord(
                hdc, xLeft, yTop, xRight, yBottom, xRadial1, yRadial1, xRadial2, yRadial2,
            ) as BOOL;
        } else {
            let pLDC: PLDC = GdiGetLDC(hdc);
            let _Ret: BOOL = FALSE;
            if pLDC.is_null() {
                SetLastError(ERROR_INVALID_HANDLE);
                return _Ret;
            }
            if (*pLDC).iType == LDC_EMFLDC as INT
                && EMFDC_Chord(
                    pLDC, xLeft, yTop, xRight, yBottom, xRadial1, yRadial1, xRadial2, yRadial2,
                ) == 0
            {
                return _Ret;
            }
            /* Fall through to support information DC's. */
        }
    }

    if GdiConvertAndCheckDC(hdc).is_null() {
        return FALSE;
    }

    NtGdiArcInternal(
        _ARCTYPE_GdiTypeChord,
        hdc,
        xLeft,
        yTop,
        xRight,
        yBottom,
        xRadial1,
        yRadial1,
        xRadial2,
        yRadial2,
    )
}

/*
 * @unimplemented
 */
#[no_mangle]
pub unsafe extern "system" fn Pie(
    hdc: HDC,
    xLeft: INT,
    yTop: INT,
    xRight: INT,
    yBottom: INT,
    xRadial1: INT,
    yRadial1: INT,
    xRadial2: INT,
    yRadial2: INT,
) -> BOOL {
    // HANDLE_METADC(BOOL, Pie, FALSE, hdc, ...)
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != GDILoObjType_LO_DC_TYPE as usize {
        if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) == GDILoObjType_LO_METADC16_TYPE as usize {
            return METADC_Pie(
                hdc, xLeft, yTop, xRight, yBottom, xRadial1, yRadial1, xRadial2, yRadial2,
            ) as BOOL;
        } else {
            let pLDC: PLDC = GdiGetLDC(hdc);
            let _Ret: BOOL = FALSE;
            if pLDC.is_null() {
                SetLastError(ERROR_INVALID_HANDLE);
                return _Ret;
            }
            if (*pLDC).iType == LDC_EMFLDC as INT
                && EMFDC_Pie(
                    pLDC, xLeft, yTop, xRight, yBottom, xRadial1, yRadial1, xRadial2, yRadial2,
                ) == 0
            {
                return _Ret;
            }
            /* Fall through to support information DC's. */
        }
    }

    if GdiConvertAndCheckDC(hdc).is_null() {
        return FALSE;
    }

    NtGdiArcInternal(
        _ARCTYPE_GdiTypePie,
        hdc,
        xLeft,
        yTop,
        xRight,
        yBottom,
        xRadial1,
        yRadial1,
        xRadial2,
        yRadial2,
    )
}
