use win32k_sys::win32k::*;
use ntkrnl_sys::interlocked::InterlockedIncrement;

// ---- C macro/intrinsic expansions (wingdi.h, palette.h, msvc intrinsics) ----
// GetRValue/GetGValue/GetBValue/RGB are wingdi.h macros — inline-expand.
#[inline(always)]
unsafe fn GetRValue(rgb: ULONG) -> ULONG { rgb & 0xFF }
#[inline(always)]
unsafe fn GetGValue(rgb: ULONG) -> ULONG { (rgb >> 8) & 0xFF }
#[inline(always)]
unsafe fn GetBValue(rgb: ULONG) -> ULONG { (rgb >> 16) & 0xFF }
// RGB(r,g,b) = ((BYTE)(r)|((WORD)(BYTE)(g)<<8)|((DWORD)(BYTE)(b)<<16))
#[inline(always)]
unsafe fn RGB(r: ULONG, g: ULONG, b: ULONG) -> ULONG {
    (r & 0xFF) | ((g & 0xFF) << 8) | ((b & 0xFF) << 16)
}
// _rotl — MSVC intrinsic: left-rotate u32 by `shift` (bit count).
// In C, _rotl(x, 0) == x; _rotl(x, n) where n>=32 wraps mod 32.
// Used here with shift values that come from CalculateShift's (b - a + 32) mod 32 logic
// (i.e. always within 0..32). Use rotate_left with masking to be safe.
#[inline(always)]
unsafe fn _rotl(value: u32, shift: i32) -> u32 {
    value.rotate_left((shift as u32) & 31)
}
// CalculateShift — FORCEINLINE in palette.h. Computes the rotate-left count
// that maps the high bit of ulMask1 onto the high bit of ulMask2.
#[inline(always)]
unsafe fn CalculateShift(ulMask1: ULONG, ulMask2: ULONG) -> ULONG {
    // BitScanReverse returns the index of the high set bit (0..31).
    // If a mask is 0 the C BitScanReverse leaves the output undefined; the
    // callers here only feed in nonzero masks (RGB(0xFF,0,0) etc.).
    let s1: u32 = if ulMask1 == 0 { 0 } else { 31 - ulMask1.leading_zeros() };
    let s2: u32 = if ulMask2 == 0 { 0 } else { 31 - ulMask2.leading_zeros() };
    let mut shift = s2.wrapping_sub(s1);
    if (shift as i32) < 0 { shift = shift.wrapping_add(32); }
    shift
}
// `min` is a C macro / stdlib helper; use core::cmp::min via a thin shim
// to keep the call sites byte-for-byte identical with the C.
#[inline(always)]
unsafe fn min(a: u32, b: u32) -> u32 { if a < b { a } else { b } }

// ---- file-scope preamble (translated) ----
// gexloTrivial is exported (declared `extern static mut gexloTrivial` in win32k-sys).
#[no_mangle]
pub static mut gexloTrivial: EXLATEOBJ = EXLATEOBJ {
    xlo: XLATEOBJ {
        iUniq: 0,
        flXlate: XO_TRIVIAL,
        iSrcType: 0,
        iDstType: 0,
        cEntries: 0,
        pulXlate: core::ptr::null_mut(),
    },
    pfnXlate: Some(EXLATEOBJ_iXlateTrivial),
    ppalSrc: core::ptr::null_mut(),
    ppalDst: core::ptr::null_mut(),
    ppalDstDc: core::ptr::null_mut(),
    hColorTransform: core::ptr::null_mut(),
    __bindgen_anon_1: _EXLATEOBJ__bindgen_ty_1 { aulXlate: [0; 6] },
};

pub static mut giUniqueXlate: ULONG = 0;

pub static gajXlate5to8: [BYTE; 32] = [
    0, 8, 16, 25, 33, 41, 49, 58, 66, 74, 82, 90, 99, 107, 115, 123,
    132, 140, 148, 156, 165, 173, 181, 189, 197, 206, 214, 222, 231, 239, 247, 255,
];

pub static gajXlate6to8: [BYTE; 64] = [
    0, 4, 8, 12, 16, 20, 24, 28, 32, 36, 40, 45, 49, 52, 57, 61,
    65, 69, 73, 77, 81, 85, 89, 93, 97, 101, 105, 109, 113, 117, 121, 125,
    130, 134, 138, 142, 146, 150, 154, 158, 162, 166, 170, 174, 178, 182, 186, 190,
    194, 198, 202, 207, 210, 215, 219, 223, 227, 231, 235, 239, 243, 247, 251, 255,
];

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn EXLATEOBJ_iXlateTrivial(
    pexlo: PEXLATEOBJ,
    iColor: ULONG,
) -> ULONG {
    iColor
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn EXLATEOBJ_iXlateTrivial(
    pexlo: PEXLATEOBJ,
    iColor: ULONG,
) -> ULONG {
    iColor
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn EXLATEOBJ_iXlateMonoInvert(
    _pexlo: PEXLATEOBJ,
    iColor: ULONG,
) -> ULONG {
    iColor ^ 1
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn EXLATEOBJ_iXlateMonoInvert(
    _pexlo: PEXLATEOBJ,
    iColor: ULONG,
) -> ULONG {
    iColor ^ 1
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn EXLATEOBJ_iXlateMonoTo0(
    _pexlo: PEXLATEOBJ,
    _iColor: ULONG,
) -> ULONG {
    0
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn EXLATEOBJ_iXlateMonoTo0(
    _pexlo: PEXLATEOBJ,
    _iColor: ULONG,
) -> ULONG {
    0
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn EXLATEOBJ_iXlateMonoTo1(
    pexlo: PEXLATEOBJ,
    iColor: ULONG,
) -> ULONG {
    1
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn EXLATEOBJ_iXlateMonoTo1(
    pexlo: PEXLATEOBJ,
    iColor: ULONG,
) -> ULONG {
    1
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn EXLATEOBJ_iXlateRGBToBW(pexlo: PEXLATEOBJ, iColor: u32) -> u32 {
    let r = GetRValue(iColor);
    let g = GetGValue(iColor);
    let b = GetBValue(iColor);
    ((r + g + b) >= 383) as u32
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn EXLATEOBJ_iXlateRGBToBW(pexlo: PEXLATEOBJ, iColor: u32) -> u32 {
    let r = GetRValue(iColor);
    let g = GetGValue(iColor);
    let b = GetBValue(iColor);
    ((r + g + b) >= 383) as u32
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn EXLATEOBJ_iXlateRGBToWB(pexlo: PEXLATEOBJ, iColor: ULONG) -> ULONG {
    
    let r = GetRValue(iColor);
    let g = GetGValue(iColor);
    let b = GetBValue(iColor);
    return if (r + g + b) < 383 { 1 } else { 0 };
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn EXLATEOBJ_iXlateRGBToWB(pexlo: PEXLATEOBJ, iColor: ULONG) -> ULONG {
    
    let r = GetRValue(iColor);
    let g = GetGValue(iColor);
    let b = GetBValue(iColor);
    return if (r + g + b) < 383 { 1 } else { 0 };
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn EXLATEOBJ_iXlateRGBToMono(
    pexlo: PEXLATEOBJ,
    rgbColor: ULONG
) -> ULONG {
    let r = GetRValue(rgbColor) as LONG;
    let g = GetGValue(rgbColor) as LONG;
    let b = GetBValue(rgbColor) as LONG;

    let lDist = r * (*pexlo).__bindgen_anon_1.ToMono.lDeltaR +
                g * (*pexlo).__bindgen_anon_1.ToMono.lDeltaG +
                b * (*pexlo).__bindgen_anon_1.ToMono.lDeltaB;

    (lDist > (*pexlo).__bindgen_anon_1.ToMono.lHalfDist) as ULONG
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn EXLATEOBJ_iXlateRGBToMono(
    pexlo: PEXLATEOBJ,
    rgbColor: ULONG
) -> ULONG {
    let r = GetRValue(rgbColor) as LONG;
    let g = GetGValue(rgbColor) as LONG;
    let b = GetBValue(rgbColor) as LONG;

    let lDist = r * (*pexlo).__bindgen_anon_1.ToMono.lDeltaR +
                g * (*pexlo).__bindgen_anon_1.ToMono.lDeltaG +
                b * (*pexlo).__bindgen_anon_1.ToMono.lDeltaB;

    (lDist > (*pexlo).__bindgen_anon_1.ToMono.lHalfDist) as ULONG
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn EXLATEOBJ_iXlateBGRToMono(pexlo: PEXLATEOBJ, rgbColor: ULONG) -> ULONG {

    let b = GetRValue(rgbColor) as LONG;
    let g = GetGValue(rgbColor) as LONG;
    let r = GetBValue(rgbColor) as LONG;

    let lDist = r * (*pexlo).__bindgen_anon_1.ToMono.lDeltaR +
                g * (*pexlo).__bindgen_anon_1.ToMono.lDeltaG +
                b * (*pexlo).__bindgen_anon_1.ToMono.lDeltaB;

    (lDist > (*pexlo).__bindgen_anon_1.ToMono.lHalfDist) as ULONG
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn EXLATEOBJ_iXlateBGRToMono(pexlo: PEXLATEOBJ, rgbColor: ULONG) -> ULONG {

    let b = GetRValue(rgbColor) as LONG;
    let g = GetGValue(rgbColor) as LONG;
    let r = GetBValue(rgbColor) as LONG;

    let lDist = r * (*pexlo).__bindgen_anon_1.ToMono.lDeltaR +
                g * (*pexlo).__bindgen_anon_1.ToMono.lDeltaG +
                b * (*pexlo).__bindgen_anon_1.ToMono.lDeltaB;

    (lDist > (*pexlo).__bindgen_anon_1.ToMono.lHalfDist) as ULONG
}

unsafe fn EXLATEOBJ_vInitRGBToMono(
    pexlo: PEXLATEOBJ,
    rgbBack: ULONG,
    rgbFore: ULONG,
    bIsBGR: BOOL,
) {
    if (rgbBack == 0x000000) && (rgbFore == 0xFFFFFF) {
        (*pexlo).pfnXlate = Some(EXLATEOBJ_iXlateRGBToBW);
    } else if (rgbBack == 0xFFFFFF) && (rgbFore == 0x000000) {
        (*pexlo).pfnXlate = Some(EXLATEOBJ_iXlateRGBToWB);
    } else {
        if bIsBGR != 0 {
            (*pexlo).pfnXlate = Some(EXLATEOBJ_iXlateBGRToMono);
        } else {
            (*pexlo).pfnXlate = Some(EXLATEOBJ_iXlateRGBToMono);
        }

        let lForeR = GetRValue(rgbFore) as LONG;
        let lForeG = GetGValue(rgbFore) as LONG;
        let lForeB = GetBValue(rgbFore) as LONG;
        let lBackR = GetRValue(rgbBack) as LONG;
        let lBackG = GetGValue(rgbBack) as LONG;
        let lBackB = GetBValue(rgbBack) as LONG;
        (*pexlo).__bindgen_anon_1.ToMono.lDeltaR = lForeR - lBackR;
        (*pexlo).__bindgen_anon_1.ToMono.lDeltaG = lForeG - lBackG;
        (*pexlo).__bindgen_anon_1.ToMono.lDeltaB = lForeB - lBackB;
        (*pexlo).__bindgen_anon_1.ToMono.lHalfDist =
            (((lForeR * lForeR) + (lForeG * lForeG) + (lForeB * lForeB)) -
             ((lBackR * lBackR) + (lBackG * lBackG) + (lBackB * lBackB))) / 2;
    }
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn EXLATEOBJ_iXlateToMono(pexlo: PEXLATEOBJ, iColor: ULONG) -> ULONG {
    (iColor == (*(*pexlo).xlo.pulXlate.offset(0))) as ULONG
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn EXLATEOBJ_iXlateToMono(pexlo: PEXLATEOBJ, iColor: ULONG) -> ULONG {
    (iColor == (*(*pexlo).xlo.pulXlate.offset(0))) as ULONG
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn EXLATEOBJ_iXlateTable(pexlo: PEXLATEOBJ, iColor: ULONG) -> ULONG {
    if iColor >= (*pexlo).xlo.cEntries {
        return 0;
    }
    *(*pexlo).xlo.pulXlate.offset(iColor as isize)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn EXLATEOBJ_iXlateTable(pexlo: PEXLATEOBJ, iColor: ULONG) -> ULONG {
    if iColor >= (*pexlo).xlo.cEntries {
        return 0;
    }
    *(*pexlo).xlo.pulXlate.offset(iColor as isize)
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn EXLATEOBJ_iXlateRGBtoBGR(pxlo: PEXLATEOBJ, iColor: ULONG) -> ULONG {
    let mut iNewColor: ULONG;

    iNewColor = iColor & 0xff00ff00;
    let mut iColor = iColor;
    iColor &= 0x00ff00ff;
    iNewColor |= iColor >> 16;
    iNewColor |= iColor << 16;

    iNewColor
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn EXLATEOBJ_iXlateRGBtoBGR(pxlo: PEXLATEOBJ, iColor: ULONG) -> ULONG {
    let mut iNewColor: ULONG;

    iNewColor = iColor & 0xff00ff00;
    let mut iColor = iColor;
    iColor &= 0x00ff00ff;
    iNewColor |= iColor >> 16;
    iNewColor |= iColor << 16;

    iNewColor
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn EXLATEOBJ_iXlateRGBto555(pxlo: PEXLATEOBJ, iColor: ULONG) -> ULONG {
    let mut iColor = iColor;
    let mut iNewColor: ULONG;

    iColor <<= 7;
    iNewColor = iColor & 0x7C00;

    iColor >>= 13;
    iNewColor |= iColor & 0x3E0;

    iColor >>= 13;
    iNewColor |= iColor & 0x1F;

    iNewColor
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn EXLATEOBJ_iXlateRGBto555(pxlo: PEXLATEOBJ, iColor: ULONG) -> ULONG {
    let mut iColor = iColor;
    let mut iNewColor: ULONG;

    iColor <<= 7;
    iNewColor = iColor & 0x7C00;

    iColor >>= 13;
    iNewColor |= iColor & 0x3E0;

    iColor >>= 13;
    iNewColor |= iColor & 0x1F;

    iNewColor
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn EXLATEOBJ_iXlateBGRto555(pxlo: PEXLATEOBJ, iColor: ULONG) -> ULONG {
    let mut iColor = iColor;
    let mut iNewColor: ULONG;

    iColor >>= 3;
    iNewColor = iColor & 0x1f;

    iColor >>= 3;
    iNewColor |= iColor & 0x3E0;

    iColor >>= 3;
    iNewColor |= iColor & 0x7C00;

    iNewColor
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn EXLATEOBJ_iXlateBGRto555(pxlo: PEXLATEOBJ, iColor: ULONG) -> ULONG {
    let mut iColor = iColor;
    let mut iNewColor: ULONG;

    iColor >>= 3;
    iNewColor = iColor & 0x1f;

    iColor >>= 3;
    iNewColor |= iColor & 0x3E0;

    iColor >>= 3;
    iNewColor |= iColor & 0x7C00;

    iNewColor
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn EXLATEOBJ_iXlateRGBto565(
    pxlo: PEXLATEOBJ,
    iColor: ULONG
) -> ULONG {
    let mut iColor = iColor;
    let mut iNewColor: ULONG;

    iColor <<= 8;
    iNewColor = iColor & 0xF800;

    iColor >>= 13;
    iNewColor |= iColor & 0x7E0;

    iColor >>= 14;
    iNewColor |= iColor & 0x1F;

    iNewColor
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn EXLATEOBJ_iXlateRGBto565(
    pxlo: PEXLATEOBJ,
    iColor: ULONG
) -> ULONG {
    let mut iColor = iColor;
    let mut iNewColor: ULONG;

    iColor <<= 8;
    iNewColor = iColor & 0xF800;

    iColor >>= 13;
    iNewColor |= iColor & 0x7E0;

    iColor >>= 14;
    iNewColor |= iColor & 0x1F;

    iNewColor
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn EXLATEOBJ_iXlateBGRto565(pxlo: PEXLATEOBJ, iColor: ULONG) -> ULONG {
    let mut iColor = iColor;
    let mut iNewColor: ULONG;

    iColor >>= 3;
    iNewColor = iColor & 0x1f;

    iColor >>= 2;
    iNewColor |= iColor & 0x7E0;

    iColor >>= 3;
    iNewColor |= iColor & 0xF800;

    iNewColor
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn EXLATEOBJ_iXlateBGRto565(pxlo: PEXLATEOBJ, iColor: ULONG) -> ULONG {
    let mut iColor = iColor;
    let mut iNewColor: ULONG;

    iColor >>= 3;
    iNewColor = iColor & 0x1f;

    iColor >>= 2;
    iNewColor |= iColor & 0x7E0;

    iColor >>= 3;
    iNewColor |= iColor & 0xF800;

    iNewColor
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn EXLATEOBJ_iXlateRGBtoPal(pexlo: PEXLATEOBJ, iColor: ULONG) -> ULONG {
    PALETTE_ulGetNearestPaletteIndex((*pexlo).ppalDst, iColor)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn EXLATEOBJ_iXlateRGBtoPal(pexlo: PEXLATEOBJ, iColor: ULONG) -> ULONG {
    PALETTE_ulGetNearestPaletteIndex((*pexlo).ppalDst, iColor)
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn EXLATEOBJ_iXlate555toRGB(pxlo: PEXLATEOBJ, iColor: ULONG) -> ULONG {
    let mut iNewColor: ULONG;
    let mut iColor = iColor;

    iNewColor = (gajXlate5to8[(iColor & 0x1F) as usize] as ULONG) << 16;

    iColor >>= 5;
    iNewColor |= (gajXlate5to8[(iColor & 0x1F) as usize] as ULONG) << 8;

    iColor >>= 5;
    iNewColor |= gajXlate5to8[(iColor & 0x1F) as usize] as ULONG;

    return iNewColor;
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn EXLATEOBJ_iXlate555toRGB(pxlo: PEXLATEOBJ, iColor: ULONG) -> ULONG {
    let mut iNewColor: ULONG;
    let mut iColor = iColor;

    iNewColor = (gajXlate5to8[(iColor & 0x1F) as usize] as ULONG) << 16;

    iColor >>= 5;
    iNewColor |= (gajXlate5to8[(iColor & 0x1F) as usize] as ULONG) << 8;

    iColor >>= 5;
    iNewColor |= gajXlate5to8[(iColor & 0x1F) as usize] as ULONG;

    return iNewColor;
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn EXLATEOBJ_iXlate555toBGR(pxlo: PEXLATEOBJ, iColor: ULONG) -> ULONG {
    let mut iNewColor: ULONG;
    let mut iColor = iColor;

    iNewColor = gajXlate5to8[(iColor & 0x1F) as usize] as ULONG;

    iColor >>= 5;
    iNewColor |= (gajXlate5to8[(iColor & 0x1F) as usize] as ULONG) << 8;

    iColor >>= 5;
    iNewColor |= (gajXlate5to8[(iColor & 0x1F) as usize] as ULONG) << 16;

    iNewColor
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn EXLATEOBJ_iXlate555toBGR(pxlo: PEXLATEOBJ, iColor: ULONG) -> ULONG {
    let mut iNewColor: ULONG;
    let mut iColor = iColor;

    iNewColor = gajXlate5to8[(iColor & 0x1F) as usize] as ULONG;

    iColor >>= 5;
    iNewColor |= (gajXlate5to8[(iColor & 0x1F) as usize] as ULONG) << 8;

    iColor >>= 5;
    iNewColor |= (gajXlate5to8[(iColor & 0x1F) as usize] as ULONG) << 16;

    iNewColor
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn EXLATEOBJ_iXlate555to565(pxlo: PEXLATEOBJ, iColor: ULONG) -> ULONG {
    let mut iNewColor: ULONG;
    let mut iColor = iColor;

    iNewColor = iColor & 0x1f;

    iColor <<= 1;
    iNewColor |= iColor & 0xFFC0;

    iColor >>= 5;
    iNewColor |= iColor & 0x20;

    iNewColor
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn EXLATEOBJ_iXlate555to565(pxlo: PEXLATEOBJ, iColor: ULONG) -> ULONG {
    let mut iNewColor: ULONG;
    let mut iColor = iColor;

    iNewColor = iColor & 0x1f;

    iColor <<= 1;
    iNewColor |= iColor & 0xFFC0;

    iColor >>= 5;
    iNewColor |= iColor & 0x20;

    iNewColor
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn EXLATEOBJ_iXlate555toPal(pexlo: PEXLATEOBJ, iColor: ULONG) -> ULONG {
    let iColor = EXLATEOBJ_iXlate555toRGB(pexlo, iColor);
    
    return PALETTE_ulGetNearestPaletteIndex((*pexlo).ppalDst, iColor);
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn EXLATEOBJ_iXlate555toPal(pexlo: PEXLATEOBJ, iColor: ULONG) -> ULONG {
    let iColor = EXLATEOBJ_iXlate555toRGB(pexlo, iColor);
    
    return PALETTE_ulGetNearestPaletteIndex((*pexlo).ppalDst, iColor);
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn EXLATEOBJ_iXlate565to555(pxlo: PEXLATEOBJ, iColor: ULONG) -> ULONG {
    let mut iNewColor: ULONG;

    iNewColor = iColor & 0x1f;

    let mut iColor = iColor;
    iColor >>= 1;
    iNewColor |= iColor & 0x7FE0;

    return iNewColor;
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn EXLATEOBJ_iXlate565to555(pxlo: PEXLATEOBJ, iColor: ULONG) -> ULONG {
    let mut iNewColor: ULONG;

    iNewColor = iColor & 0x1f;

    let mut iColor = iColor;
    iColor >>= 1;
    iNewColor |= iColor & 0x7FE0;

    return iNewColor;
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn EXLATEOBJ_iXlate565toRGB(pexlo: PEXLATEOBJ, iColor: ULONG) -> ULONG {
    let mut iNewColor: ULONG;
    let mut iColor = iColor;

    iNewColor = (gajXlate5to8[(iColor & 0x1F) as usize] as ULONG) << 16;

    iColor >>= 5;
    iNewColor |= (gajXlate6to8[(iColor & 0x3F) as usize] as ULONG) << 8;

    iColor >>= 6;
    iNewColor |= gajXlate5to8[(iColor & 0x1F) as usize] as ULONG;

    return iNewColor;
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn EXLATEOBJ_iXlate565toRGB(pexlo: PEXLATEOBJ, iColor: ULONG) -> ULONG {
    let mut iNewColor: ULONG;
    let mut iColor = iColor;

    iNewColor = (gajXlate5to8[(iColor & 0x1F) as usize] as ULONG) << 16;

    iColor >>= 5;
    iNewColor |= (gajXlate6to8[(iColor & 0x3F) as usize] as ULONG) << 8;

    iColor >>= 6;
    iNewColor |= gajXlate5to8[(iColor & 0x1F) as usize] as ULONG;

    return iNewColor;
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn EXLATEOBJ_iXlate565toBGR(pexlo: PEXLATEOBJ, iColor: ULONG) -> ULONG {
    let mut iNewColor: ULONG;
    let mut iColor = iColor;

    iNewColor = gajXlate5to8[(iColor & 0x1F) as usize] as ULONG;

    iColor >>= 5;
    iNewColor |= (gajXlate6to8[(iColor & 0x3F) as usize] as ULONG) << 8;

    iColor >>= 6;
    iNewColor |= (gajXlate5to8[(iColor & 0x1F) as usize] as ULONG) << 16;

    return iNewColor;
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn EXLATEOBJ_iXlate565toBGR(pexlo: PEXLATEOBJ, iColor: ULONG) -> ULONG {
    let mut iNewColor: ULONG;
    let mut iColor = iColor;

    iNewColor = gajXlate5to8[(iColor & 0x1F) as usize] as ULONG;

    iColor >>= 5;
    iNewColor |= (gajXlate6to8[(iColor & 0x3F) as usize] as ULONG) << 8;

    iColor >>= 6;
    iNewColor |= (gajXlate5to8[(iColor & 0x1F) as usize] as ULONG) << 16;

    return iNewColor;
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn EXLATEOBJ_iXlate565toPal(pexlo: PEXLATEOBJ, iColor: ULONG) -> ULONG {
    let iColor = EXLATEOBJ_iXlate565toRGB(pexlo, iColor);

    return PALETTE_ulGetNearestPaletteIndex((*pexlo).ppalDst, iColor);
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn EXLATEOBJ_iXlate565toPal(pexlo: PEXLATEOBJ, iColor: ULONG) -> ULONG {
    let iColor = EXLATEOBJ_iXlate565toRGB(pexlo, iColor);

    return PALETTE_ulGetNearestPaletteIndex((*pexlo).ppalDst, iColor);
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn EXLATEOBJ_iXlateShiftAndMask(pexlo: PEXLATEOBJ, iColor: ULONG) -> ULONG {
    let mut iNewColor: ULONG;
    let shifts = &(*pexlo).__bindgen_anon_1.__bindgen_anon_1;

    iNewColor = _rotl(iColor, shifts.ulRedShift as i32) & shifts.ulRedMask;
    iNewColor |= _rotl(iColor, shifts.ulGreenShift as i32) & shifts.ulGreenMask;
    iNewColor |= _rotl(iColor, shifts.ulBlueShift as i32) & shifts.ulBlueMask;

    return iNewColor;
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn EXLATEOBJ_iXlateShiftAndMask(pexlo: PEXLATEOBJ, iColor: ULONG) -> ULONG {
    let mut iNewColor: ULONG;
    let shifts = &(*pexlo).__bindgen_anon_1.__bindgen_anon_1;

    iNewColor = _rotl(iColor, shifts.ulRedShift as i32) & shifts.ulRedMask;
    iNewColor |= _rotl(iColor, shifts.ulGreenShift as i32) & shifts.ulGreenMask;
    iNewColor |= _rotl(iColor, shifts.ulBlueShift as i32) & shifts.ulBlueMask;

    return iNewColor;
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn EXLATEOBJ_iXlateBitfieldsToPal(pexlo: PEXLATEOBJ, iColor: ULONG) -> ULONG {
    let mut iColor = iColor;

    iColor = EXLATEOBJ_iXlateShiftAndMask(pexlo, iColor);

    return PALETTE_ulGetNearestPaletteIndex((*pexlo).ppalDst, iColor);
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn EXLATEOBJ_iXlateBitfieldsToPal(pexlo: PEXLATEOBJ, iColor: ULONG) -> ULONG {
    let mut iColor = iColor;

    iColor = EXLATEOBJ_iXlateShiftAndMask(pexlo, iColor);

    return PALETTE_ulGetNearestPaletteIndex((*pexlo).ppalDst, iColor);
}

#[no_mangle]
pub unsafe extern "system" fn EXLATEOBJ_vInitialize(
    pexlo: PEXLATEOBJ,
    ppalSrc: *mut PALETTE,
    ppalDst: *mut PALETTE,
    crSrcBackColor: COLORREF,
    crDstBackColor: COLORREF,
    crDstForeColor: COLORREF,
) {
    let mut cEntries: ULONG;
    let mut i: ULONG;
    let mut ulColor: ULONG;

    let ppalSrc = if ppalSrc.is_null() { core::ptr::addr_of_mut!(gpalRGB) } else { ppalSrc };
    let ppalDst = if ppalDst.is_null() { core::ptr::addr_of_mut!(gpalRGB) } else { ppalDst };

    (*pexlo).xlo.iUniq = InterlockedIncrement(core::ptr::addr_of_mut!(giUniqueXlate) as *mut i32) as ULONG;
    (*pexlo).xlo.cEntries = 0;
    (*pexlo).xlo.flXlate = 0;
    (*pexlo).xlo.pulXlate = (*pexlo).__bindgen_anon_1.aulXlate.as_mut_ptr();
    (*pexlo).pfnXlate = Some(EXLATEOBJ_iXlateTrivial);
    (*pexlo).hColorTransform = core::ptr::null_mut();
    (*pexlo).ppalSrc = ppalSrc;
    (*pexlo).ppalDst = ppalDst;
    (*pexlo).xlo.iSrcType = (*ppalSrc).flFlags as u16;
    (*pexlo).xlo.iDstType = (*ppalDst).flFlags as u16;
    (*pexlo).ppalDstDc = core::ptr::addr_of_mut!(gpalRGB);

    if ppalDst == ppalSrc {
        (*pexlo).xlo.flXlate |= XO_TRIVIAL;
        return;
    }

    if ((*ppalSrc).flFlags & PAL_INDEXED) == 0 || ((*ppalDst).flFlags & PAL_INDEXED) == 0 {
        let mut aulMasksSrc: [ULONG; 3] = [0; 3];
        let mut aulMasksDst: [ULONG; 3] = [0; 3];

        PALETTE_vGetBitMasks(ppalSrc, aulMasksSrc.as_mut_ptr());
        PALETTE_vGetBitMasks(ppalDst, aulMasksDst.as_mut_ptr());

        let shifts = &mut (*pexlo).__bindgen_anon_1.__bindgen_anon_1;
        shifts.ulRedMask = aulMasksDst[0];
        shifts.ulGreenMask = aulMasksDst[1];
        shifts.ulBlueMask = aulMasksDst[2];

        shifts.ulRedShift = CalculateShift(aulMasksSrc[0], aulMasksDst[0]);
        shifts.ulGreenShift = CalculateShift(aulMasksSrc[1], aulMasksDst[1]);
        shifts.ulBlueShift = CalculateShift(aulMasksSrc[2], aulMasksDst[2]);
    }

    if (*ppalSrc).flFlags & PAL_MONOCHROME != 0 {
        if (*ppalDst).flFlags & PAL_MONOCHROME != 0 {
            let mut iColors: [ULONG; 2] = [0; 2];

            if ((*ppalSrc).flFlags & PAL_DIBSECTION) != 0 && ((*ppalDst).flFlags & PAL_DIBSECTION) == 0 {
                iColors[1] = PALETTE_ulGetNearestPaletteIndex(ppalSrc, crSrcBackColor);
                iColors[0] = iColors[1] ^ 1;
            } else if ((*ppalSrc).flFlags & PAL_DIBSECTION) == 0 && ((*ppalDst).flFlags & PAL_DIBSECTION) != 0 {
                let iBlack = PALETTE_ulGetNearestPaletteIndex(ppalSrc, 0x000000);
                let iWhite = iBlack ^ 1;
                iColors[iBlack as usize] = PALETTE_ulGetNearestPaletteIndex(ppalDst, crDstForeColor);
                iColors[iWhite as usize] = PALETTE_ulGetNearestPaletteIndex(ppalDst, crDstBackColor);
            } else {
                let rgbColor0 = PALETTE_ulGetRGBColorFromIndex(ppalSrc, 0);
                let rgbColor1 = PALETTE_ulGetRGBColorFromIndex(ppalSrc, 1);
                iColors[0] = PALETTE_ulGetNearestPaletteIndex(ppalDst, rgbColor0);
                iColors[1] = PALETTE_ulGetNearestPaletteIndex(ppalDst, rgbColor1);
            }

            if iColors[0] == 0 && iColors[1] == 1 {
                (*pexlo).pfnXlate = Some(EXLATEOBJ_iXlateTrivial);
                (*pexlo).xlo.flXlate = XO_TRIVIAL;
            } else if iColors[0] == 1 && iColors[1] == 0 {
                (*pexlo).pfnXlate = Some(EXLATEOBJ_iXlateMonoInvert);
            } else if iColors[0] == 0 {
                (*pexlo).pfnXlate = Some(EXLATEOBJ_iXlateMonoTo0);
            } else {
                (*pexlo).pfnXlate = Some(EXLATEOBJ_iXlateMonoTo1);
            }
        } else {
            (*pexlo).pfnXlate = Some(EXLATEOBJ_iXlateTable);
            (*pexlo).xlo.flXlate |= XO_TABLE;
            (*pexlo).xlo.cEntries = 2;

            if ((*ppalSrc).flFlags & PAL_DIBSECTION) != 0 || crDstBackColor == CLR_INVALID {
                let rgbColor0 = PALETTE_ulGetRGBColorFromIndex(ppalSrc, 0);
                let rgbColor1 = PALETTE_ulGetRGBColorFromIndex(ppalSrc, 1);
                (*pexlo).__bindgen_anon_1.aulXlate[0] = PALETTE_ulGetNearestIndex(ppalDst, rgbColor0);
                (*pexlo).__bindgen_anon_1.aulXlate[1] = PALETTE_ulGetNearestIndex(ppalDst, rgbColor1);
            } else {
                let iBlack = PALETTE_ulGetNearestPaletteIndex(ppalSrc, 0x000000);
                let iWhite = iBlack ^ 1;
                (*(*pexlo).xlo.pulXlate.offset(iBlack as isize)) = PALETTE_ulGetNearestIndex(ppalDst, crDstForeColor);
                (*(*pexlo).xlo.pulXlate.offset(iWhite as isize)) = PALETTE_ulGetNearestIndex(ppalDst, crDstBackColor);
            }
        }
    } else if (*ppalDst).flFlags & PAL_MONOCHROME != 0 {
        (*pexlo).pfnXlate = Some(EXLATEOBJ_iXlateToMono);
        (*pexlo).xlo.flXlate |= XO_TO_MONO;
        (*pexlo).xlo.cEntries = 1;

        if (*ppalSrc).flFlags & PAL_INDEXED != 0 {
            (*pexlo).__bindgen_anon_1.aulXlate[0] = PALETTE_ulGetNearestPaletteIndex(ppalSrc, crSrcBackColor);
        } else if (*ppalSrc).flFlags & PAL_RGB != 0 {
            (*pexlo).__bindgen_anon_1.aulXlate[0] = crSrcBackColor;

            if crSrcBackColor == CLR_INVALID {
                EXLATEOBJ_vInitRGBToMono(
                    pexlo,
                    PALETTE_ulGetRGBColorFromIndex(ppalDst, 0),
                    PALETTE_ulGetRGBColorFromIndex(ppalDst, 1),
                    0,
                );
            }
        } else if (*ppalSrc).flFlags & PAL_BGR != 0 {
            (*pexlo).__bindgen_anon_1.aulXlate[0] = RGB(
                GetBValue(crSrcBackColor),
                GetGValue(crSrcBackColor),
                GetRValue(crSrcBackColor),
            );

            if crSrcBackColor == CLR_INVALID {
                EXLATEOBJ_vInitRGBToMono(
                    pexlo,
                    PALETTE_ulGetRGBColorFromIndex(ppalDst, 0),
                    PALETTE_ulGetRGBColorFromIndex(ppalDst, 1),
                    1,
                );
            }
        } else if (*ppalSrc).flFlags & PAL_BITFIELDS != 0 {
            // C: PALETTE_vGetBitMasks(ppalSrc, &pexlo->ulRedMask);  -- writes 3 ULONGs starting at ulRedMask.
            let shifts = &mut (*pexlo).__bindgen_anon_1.__bindgen_anon_1;
            PALETTE_vGetBitMasks(ppalSrc, core::ptr::addr_of_mut!(shifts.ulRedMask));
            shifts.ulRedShift = CalculateShift(RGB(0xFF, 0, 0), shifts.ulRedMask);
            shifts.ulGreenShift = CalculateShift(RGB(0, 0xFF, 0), shifts.ulGreenMask);
            shifts.ulBlueShift = CalculateShift(RGB(0, 0, 0xFF), shifts.ulBlueMask);

            (*pexlo).__bindgen_anon_1.aulXlate[0] = EXLATEOBJ_iXlateShiftAndMask(pexlo, crSrcBackColor);
        }
    } else if (*ppalSrc).flFlags & PAL_INDEXED != 0 {
        cEntries = (*ppalSrc).NumColors;

        if cEntries > 6 {
            (*pexlo).xlo.pulXlate = EngAllocMem(0, cEntries * core::mem::size_of::<ULONG>() as u32, 0x20616C58) as *mut ULONG;
            if (*pexlo).xlo.pulXlate.is_null() {
                (*pexlo).pfnXlate = Some(EXLATEOBJ_iXlateTrivial);
                (*pexlo).xlo.flXlate = XO_TRIVIAL;
                return;
            }
        }

        (*pexlo).pfnXlate = Some(EXLATEOBJ_iXlateTable);
        (*pexlo).xlo.cEntries = cEntries;
        (*pexlo).xlo.flXlate |= XO_TABLE;

        if (*ppalDst).flFlags & PAL_INDEXED != 0 {
            let mut cDiff: ULONG = 0;

            i = 0;
            while i < cEntries {
                let entry = (*ppalSrc).IndexedColors.add(i as usize);
                ulColor = RGB(
                    (*entry).peRed as ULONG,
                    (*entry).peGreen as ULONG,
                    (*entry).peBlue as ULONG,
                );

                (*(*pexlo).xlo.pulXlate.offset(i as isize)) = PALETTE_ulGetNearestPaletteIndex(ppalDst, ulColor);

                if (*(*pexlo).xlo.pulXlate.offset(i as isize)) != i {
                    cDiff += 1;
                }
                i += 1;
            }

            if cDiff == 0 {
                if (*pexlo).xlo.pulXlate != (*pexlo).__bindgen_anon_1.aulXlate.as_mut_ptr() {
                    EngFreeMem((*pexlo).xlo.pulXlate as *mut core::ffi::c_void);
                    (*pexlo).xlo.pulXlate = (*pexlo).__bindgen_anon_1.aulXlate.as_mut_ptr();
                }
                (*pexlo).pfnXlate = Some(EXLATEOBJ_iXlateTrivial);
                (*pexlo).xlo.flXlate = XO_TRIVIAL;
                (*pexlo).xlo.cEntries = 0;
                return;
            }
        } else {
            i = 0;
            while i < (*pexlo).xlo.cEntries {
                let entry = (*ppalSrc).IndexedColors.add(i as usize);
                ulColor = RGB(
                    (*entry).peRed as ULONG,
                    (*entry).peGreen as ULONG,
                    (*entry).peBlue as ULONG,
                );
                (*(*pexlo).xlo.pulXlate.offset(i as isize)) = PALETTE_ulGetNearestIndex(ppalDst, ulColor);
                i += 1;
            }
        }
    } else if (*ppalSrc).flFlags & PAL_RGB != 0 {
        if (*ppalDst).flFlags & PAL_INDEXED != 0 {
            (*pexlo).pfnXlate = Some(EXLATEOBJ_iXlateRGBtoPal);
        } else if (*ppalDst).flFlags & PAL_BGR != 0 {
            (*pexlo).pfnXlate = Some(EXLATEOBJ_iXlateRGBtoBGR);
        } else if (*ppalDst).flFlags & PAL_RGB16_555 != 0 {
            (*pexlo).pfnXlate = Some(EXLATEOBJ_iXlateRGBto555);
        } else if (*ppalDst).flFlags & PAL_RGB16_565 != 0 {
            (*pexlo).pfnXlate = Some(EXLATEOBJ_iXlateRGBto565);
        } else if (*ppalDst).flFlags & PAL_BITFIELDS != 0 {
            (*pexlo).pfnXlate = Some(EXLATEOBJ_iXlateShiftAndMask);
        }
    } else if (*ppalSrc).flFlags & PAL_BGR != 0 {
        if (*ppalDst).flFlags & PAL_INDEXED != 0 {
            (*pexlo).pfnXlate = Some(EXLATEOBJ_iXlateBitfieldsToPal);
        } else if (*ppalDst).flFlags & PAL_RGB != 0 {
            (*pexlo).pfnXlate = Some(EXLATEOBJ_iXlateRGBtoBGR);
        } else if (*ppalDst).flFlags & PAL_RGB16_555 != 0 {
            (*pexlo).pfnXlate = Some(EXLATEOBJ_iXlateBGRto555);
        } else if (*ppalDst).flFlags & PAL_RGB16_565 != 0 {
            (*pexlo).pfnXlate = Some(EXLATEOBJ_iXlateBGRto565);
        } else if (*ppalDst).flFlags & PAL_BITFIELDS != 0 {
            (*pexlo).pfnXlate = Some(EXLATEOBJ_iXlateShiftAndMask);
        }
    } else if (*ppalSrc).flFlags & PAL_RGB16_555 != 0 {
        if (*ppalDst).flFlags & PAL_INDEXED != 0 {
            (*pexlo).pfnXlate = Some(EXLATEOBJ_iXlate555toPal);
        } else if (*ppalDst).flFlags & PAL_RGB != 0 {
            (*pexlo).pfnXlate = Some(EXLATEOBJ_iXlate555toRGB);
        } else if (*ppalDst).flFlags & PAL_BGR != 0 {
            (*pexlo).pfnXlate = Some(EXLATEOBJ_iXlate555toBGR);
        } else if (*ppalDst).flFlags & PAL_RGB16_565 != 0 {
            (*pexlo).pfnXlate = Some(EXLATEOBJ_iXlate555to565);
        } else if (*ppalDst).flFlags & PAL_BITFIELDS != 0 {
            (*pexlo).pfnXlate = Some(EXLATEOBJ_iXlateShiftAndMask);
        }
    } else if (*ppalSrc).flFlags & PAL_RGB16_565 != 0 {
        if (*ppalDst).flFlags & PAL_INDEXED != 0 {
            (*pexlo).pfnXlate = Some(EXLATEOBJ_iXlate565toPal);
        } else if (*ppalDst).flFlags & PAL_RGB != 0 {
            (*pexlo).pfnXlate = Some(EXLATEOBJ_iXlate565toRGB);
        } else if (*ppalDst).flFlags & PAL_BGR != 0 {
            (*pexlo).pfnXlate = Some(EXLATEOBJ_iXlate565toBGR);
        } else if (*ppalDst).flFlags & PAL_RGB16_555 != 0 {
            (*pexlo).pfnXlate = Some(EXLATEOBJ_iXlate565to555);
        } else if (*ppalDst).flFlags & PAL_BITFIELDS != 0 {
            (*pexlo).pfnXlate = Some(EXLATEOBJ_iXlateShiftAndMask);
        }
    } else if (*ppalSrc).flFlags & PAL_BITFIELDS != 0 {
        if (*ppalDst).flFlags & PAL_INDEXED != 0 {
            (*pexlo).pfnXlate = Some(EXLATEOBJ_iXlateBitfieldsToPal);
        } else {
            (*pexlo).pfnXlate = Some(EXLATEOBJ_iXlateShiftAndMask);
        }
    }

    if (*pexlo).pfnXlate.unwrap() as *const () == EXLATEOBJ_iXlateShiftAndMask as *const ()
        && (*pexlo).__bindgen_anon_1.__bindgen_anon_1.ulRedShift == 0
        && (*pexlo).__bindgen_anon_1.__bindgen_anon_1.ulGreenShift == 0
        && (*pexlo).__bindgen_anon_1.__bindgen_anon_1.ulBlueShift == 0
    {
        (*pexlo).pfnXlate = Some(EXLATEOBJ_iXlateTrivial);
    }

    if (*pexlo).pfnXlate.unwrap() as *const () == EXLATEOBJ_iXlateTrivial as *const () {
        (*pexlo).xlo.flXlate = XO_TRIVIAL;
    } else {
        (*pexlo).xlo.flXlate &= !XO_TRIVIAL;
    }
}

// NTAPI in C (xlateobj.h) — stdcall on i386, cdecl elsewhere.
#[no_mangle]
pub unsafe extern "system" fn EXLATEOBJ_vInitXlateFromDCs(
    pexlo: PEXLATEOBJ,
    pdcSrc: PDC,
    pdcDst: PDC,
) {
    EXLATEOBJ_vInitXlateFromDCs_impl(pexlo, pdcSrc, pdcDst)
}

unsafe fn EXLATEOBJ_vInitXlateFromDCs_impl(
    pexlo: PEXLATEOBJ,
    pdcSrc: PDC,
    pdcDst: PDC,
) {
    let psurfDst: PSURFACE;
    let psurfSrc: PSURFACE;

    psurfDst = (*pdcDst).dclevel.pSurface;
    psurfSrc = (*pdcSrc).dclevel.pSurface;

    EXLATEOBJ_vInitialize(
        pexlo,
        if !psurfSrc.is_null() { (*psurfSrc).ppal } else { gppalMono },
        if !psurfDst.is_null() { (*psurfDst).ppal } else { gppalMono },
        (*(*pdcSrc).pdcattr).crBackgroundClr,
        (*(*pdcDst).pdcattr).crBackgroundClr,
        (*(*pdcDst).pdcattr).crForegroundClr,
    );

    (*pexlo).ppalDstDc = (*pdcDst).dclevel.ppal;
}

// NTAPI in C (xlateobj.h) — stdcall on i386, cdecl elsewhere.
#[no_mangle]
pub unsafe extern "system" fn EXLATEOBJ_vInitXlateFromDCsEx(
    pexlo: *mut EXLATEOBJ,
    pdcSrc: PDC,
    pdcDst: PDC,
    crBackColor: COLORREF,
) {
    EXLATEOBJ_vInitXlateFromDCsEx_impl(pexlo, pdcSrc, pdcDst, crBackColor)
}

unsafe fn EXLATEOBJ_vInitXlateFromDCsEx_impl(
    pexlo: *mut EXLATEOBJ,
    pdcSrc: PDC,
    pdcDst: PDC,
    crBackColor: COLORREF,
) {
    let psurfDst: PSURFACE;
    let psurfSrc: PSURFACE;
    let mut crBackColor = crBackColor;

    psurfDst = (*pdcDst).dclevel.pSurface;
    psurfSrc = (*pdcSrc).dclevel.pSurface;

    if crBackColor == CLR_INVALID {
        crBackColor = (*(*pdcSrc).pdcattr).crBackgroundClr;
    }

    EXLATEOBJ_vInitialize(
        pexlo,
        if !psurfSrc.is_null() { (*psurfSrc).ppal } else { gppalMono },
        if !psurfDst.is_null() { (*psurfDst).ppal } else { gppalMono },
        crBackColor,
        (*(*pdcDst).pdcattr).crBackgroundClr,
        (*(*pdcDst).pdcattr).crForegroundClr,
    );

    (*pexlo).ppalDstDc = (*pdcDst).dclevel.ppal;
}

#[no_mangle]
pub unsafe extern "system" fn EXLATEOBJ_vInitSrcMonoXlate(
    pexlo: PEXLATEOBJ,
    ppalDst: PPALETTE,
    crBackgroundClr: COLORREF,
    crForegroundClr: COLORREF,
) {
    EXLATEOBJ_vInitialize(
        pexlo,
        gppalMono,
        ppalDst,
        0,
        crBackgroundClr,
        crForegroundClr,
    );
}

#[inline(always)]
unsafe fn EXLATEOBJ_vCleanup_impl(pexlo: PEXLATEOBJ) {
    if (*pexlo).xlo.pulXlate != (*pexlo).__bindgen_anon_1.aulXlate.as_mut_ptr() {
        EngFreeMem((*pexlo).xlo.pulXlate as *mut core::ffi::c_void);
    }
    (*pexlo).xlo.pulXlate = (*pexlo).__bindgen_anon_1.aulXlate.as_mut_ptr();
}

#[no_mangle]
pub unsafe extern "system" fn EXLATEOBJ_vCleanup(pexlo: PEXLATEOBJ) {
    EXLATEOBJ_vCleanup_impl(pexlo)
}
#[no_mangle]
pub unsafe extern "system" fn XLATEOBJ_iXlate(
    pxlo: *mut XLATEOBJ,
    iColor: u32,
) -> u32 {
    let pexlo = pxlo as PEXLATEOBJ;

    if pxlo.is_null() {
        return iColor;
    }

    ((*pexlo).pfnXlate.unwrap())(pexlo, iColor)
}
#[no_mangle]
pub unsafe extern "system" fn XLATEOBJ_cGetPalette(
    pxlo: *mut XLATEOBJ,
    iPal: u32,
    cPal: u32,
    pPalOut: *mut u32,
) -> u32 {

    let pexlo = pxlo as PEXLATEOBJ;
    let ppal: PPALETTE;
    let mut i: u32;

    if pxlo.is_null() {
        return 0;
    }

    if iPal > 5 {
        return 0;
    }

    if iPal == XO_DESTDCPALETTE {
        ppal = (*pexlo).ppalDstDc;
    } else if iPal == XO_SRCPALETTE || iPal == XO_SRCBITFIELDS {
        ppal = (*pexlo).ppalSrc;
    } else {
        ppal = (*pexlo).ppalDst;
    }

    if ppal.is_null()
        || ((iPal == XO_SRCPALETTE || iPal == XO_DESTPALETTE)
            && ((*ppal).flFlags & PAL_INDEXED) == 0)
        || ((iPal == XO_SRCBITFIELDS || iPal == XO_DESTBITFIELDS)
            && ((*ppal).flFlags & PAL_BITFIELDS) == 0)
    {
        return 0;
    }

    if pPalOut.is_null() {
        return (*ppal).NumColors;
    }

    let mut cPal = cPal;

    if ((*ppal).flFlags & PAL_INDEXED) != 0 {
        cPal = min(cPal, (*ppal).NumColors);
        i = 0;
        while i < cPal {
            let entry = (*ppal).IndexedColors.add(i as usize);
            *pPalOut.add(i as usize) = RGB(
                (*entry).peRed as ULONG,
                (*entry).peGreen as ULONG,
                (*entry).peBlue as ULONG,
            );
            i += 1;
        }
    } else {
        *pPalOut.add(0) = (*ppal).RedMask;
        *pPalOut.add(1) = (*ppal).GreenMask;
        *pPalOut.add(2) = (*ppal).BlueMask;
    }

    cPal
}
#[no_mangle]
pub unsafe extern "system" fn XLATEOBJ_hGetColorTransform(
    pxlo: *mut XLATEOBJ,
) -> HANDLE {
    let pexlo = pxlo as PEXLATEOBJ;
    (*pexlo).hColorTransform
}
#[no_mangle]
pub unsafe extern "system" fn XLATEOBJ_piVector(
    pxlo: *mut XLATEOBJ
) -> *mut u32 {
    if (*pxlo).iSrcType as u32 == PAL_INDEXED {
        return (*pxlo).pulXlate;
    }

    core::ptr::null_mut()
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:        See COPYING in the top level directory
//  * PROJECT:          ReactOS Win32k subsystem
//  * PURPOSE:          GDI Color Translation Functions
//  * FILE:             win32ss/gdi/eng/xlateobj.c
//  * PROGRAMER:        Timo Kreuzer (timo.kreuzer@reactos.org)
//  */
//
// #include <win32k.h>
//
// #define NDEBUG
// #include <debug.h>
//
// _Post_satisfies_(return==iColor)
// _Function_class_(FN_XLATE)
// ULONG
// FASTCALL
// EXLATEOBJ_iXlateTrivial(
//     _In_ PEXLATEOBJ pexlo,
//     _In_ ULONG iColor);
//
// /** Globals *******************************************************************/
//
// EXLATEOBJ gexloTrivial = {{0, XO_TRIVIAL, 0, 0, 0, 0}, EXLATEOBJ_iXlateTrivial};
//
// static ULONG giUniqueXlate = 0;
//
// static const BYTE gajXlate5to8[32] =
// {  0,  8, 16, 25, 33, 41, 49, 58, 66, 74, 82, 90, 99,107,115,123,
//  132,140,148,156,165,173,181,189,197,206,214,222,231,239,247,255};
//
// static const BYTE gajXlate6to8[64] =
// { 0,  4,  8, 12, 16, 20, 24, 28, 32, 36, 40, 45, 49, 52, 57, 61,
//  65, 69, 73, 77, 81, 85, 89, 93, 97,101,105,109,113,117,121,125,
// 130,134,138,142,146,150,154,158,162,166,170,174,178,182,186,190,
// 194,198,202,207,210,215,219,223,227,231,235,239,243,247,251,255};
//
//
// /** iXlate functions **********************************************************/
//
// _Post_satisfies_(return==iColor)
// _Function_class_(FN_XLATE)
// ULONG
// FASTCALL
// EXLATEOBJ_iXlateTrivial(
//     _In_ PEXLATEOBJ pexlo,
//     _In_ ULONG iColor)
// {
//     return iColor;
// }
//
// _Function_class_(FN_XLATE)
// ULONG
// FASTCALL
// EXLATEOBJ_iXlateMonoInvert(
//     _In_ PEXLATEOBJ pexlo,
//     _In_ ULONG iColor)
// {
//     return iColor ^ 1;
// }
//
// _Function_class_(FN_XLATE)
// ULONG
// FASTCALL
// EXLATEOBJ_iXlateMonoTo0(
//     _In_ PEXLATEOBJ pexlo,
//     _In_ ULONG iColor)
// {
//     return 0;
// }
//
// _Function_class_(FN_XLATE)
// ULONG
// FASTCALL
// EXLATEOBJ_iXlateMonoTo1(
//     _In_ PEXLATEOBJ pexlo,
//     _In_ ULONG iColor)
// {
//     return 1;
// }
//
// _Function_class_(FN_XLATE)
// ULONG
// FASTCALL
// EXLATEOBJ_iXlateRGBToBW(_In_ PEXLATEOBJ pexlo, ULONG iColor)
// {
//     ULONG r = GetRValue(iColor);
//     ULONG g = GetGValue(iColor);
//     ULONG b = GetBValue(iColor);
//     return ((r + g + b) >= 383);
// }
//
// _Function_class_(FN_XLATE)
// ULONG
// FASTCALL
// EXLATEOBJ_iXlateRGBToWB(_In_ PEXLATEOBJ pexlo, ULONG iColor)
// {
//     ULONG r = GetRValue(iColor);
//     ULONG g = GetGValue(iColor);
//     ULONG b = GetBValue(iColor);
//     return ((r + g + b) < 383);
// }
//
// _Function_class_(FN_XLATE)
// ULONG
// FASTCALL
// EXLATEOBJ_iXlateRGBToMono(_In_ PEXLATEOBJ pexlo, ULONG rgbColor)
// {
//     /* Get the RGB values */
//     LONG r = GetRValue(rgbColor);
//     LONG g = GetGValue(rgbColor);
//     LONG b = GetBValue(rgbColor);
//
//     /* Calculate the dot product of the color vector and the
//        back-to-fore distance vector. */
//     LONG lDist = r * pexlo->ToMono.lDeltaR +
//                  g * pexlo->ToMono.lDeltaG +
//                  b * pexlo->ToMono.lDeltaB;
//
//     return (lDist > pexlo->ToMono.lHalfDist);
// }
//
// _Function_class_(FN_XLATE)
// ULONG
// FASTCALL
// EXLATEOBJ_iXlateBGRToMono(_In_ PEXLATEOBJ pexlo, ULONG rgbColor)
// {
//     /* Inverted usage of the color macros compared to RGB! */
//     LONG b = GetRValue(rgbColor);
//     LONG g = GetGValue(rgbColor);
//     LONG r = GetBValue(rgbColor);
//
//     /* Calculate the dot product of the color vector and the
//        back-to-fore distance vector. */
//     LONG lDist = r * pexlo->ToMono.lDeltaR +
//                  g * pexlo->ToMono.lDeltaG +
//                  b * pexlo->ToMono.lDeltaB;
//
//     return (lDist > pexlo->ToMono.lHalfDist);
// }
//
// static
// VOID
// EXLATEOBJ_vInitRGBToMono(
//     _Inout_ PEXLATEOBJ pexlo,
//     _In_ ULONG rgbBack,
//     _In_ ULONG rgbFore,
//     _In_ BOOL bIsBGR)
// {
//     /* Check if dest durface is B/W or W/B */
//     if ((rgbBack == 0x000000) && (rgbFore == 0xFFFFFF))
//     {
//         /* Black/White (symmetric between RGB and BGR) */
//         pexlo->pfnXlate = EXLATEOBJ_iXlateRGBToBW;
//     }
//     else if ((rgbBack == 0xFFFFFF) && (rgbFore == 0x000000))
//     {
//         /* White/Black (symmetric between RGB and BGR) */
//         pexlo->pfnXlate = EXLATEOBJ_iXlateRGBToWB;
//     }
//     else
//     {
//         /* Use generic mono translation function */
//         if (bIsBGR)
//             pexlo->pfnXlate = EXLATEOBJ_iXlateBGRToMono;
//         else
//             pexlo->pfnXlate = EXLATEOBJ_iXlateRGBToMono;
//
//         /* Precompute evaluation constants */
//         LONG lForeR = GetRValue(rgbFore);
//         LONG lForeG = GetGValue(rgbFore);
//         LONG lForeB = GetBValue(rgbFore);
//         LONG lBackR = GetRValue(rgbBack);
//         LONG lBackG = GetGValue(rgbBack);
//         LONG lBackB = GetBValue(rgbBack);
//         pexlo->ToMono.lDeltaR = lForeR - lBackR;
//         pexlo->ToMono.lDeltaG = lForeG - lBackG;
//         pexlo->ToMono.lDeltaB = lForeB - lBackB;
//         pexlo->ToMono.lHalfDist =
//             (((lForeR * lForeR) + (lForeG * lForeG) + (lForeB * lForeB)) -
//              ((lBackR * lBackR) + (lBackG * lBackG) + (lBackB * lBackB))) / 2;
//     }
// }
//
// _Function_class_(FN_XLATE)
// ULONG
// FASTCALL
// EXLATEOBJ_iXlateToMono(_In_ PEXLATEOBJ pexlo, ULONG iColor)
// {
//     return (iColor == pexlo->xlo.pulXlate[0]);
// }
//
// _Function_class_(FN_XLATE)
// ULONG
// FASTCALL
// EXLATEOBJ_iXlateTable(PEXLATEOBJ pexlo, ULONG iColor)
// {
//     if (iColor >= pexlo->xlo.cEntries) return 0;
//     return pexlo->xlo.pulXlate[iColor];
// }
//
// _Function_class_(FN_XLATE)
// ULONG
// FASTCALL
// EXLATEOBJ_iXlateRGBtoBGR(PEXLATEOBJ pxlo, ULONG iColor)
// {
//     ULONG iNewColor;
//
//     /* Copy green */
//     iNewColor = iColor & 0xff00ff00;
//
//     /* Mask red and blue */
//     iColor &= 0x00ff00ff;
//
//     /* Shift and copy red and blue */
//     iNewColor |= iColor >> 16;
//     iNewColor |= iColor << 16;
//
//     return iNewColor;
// }
//
// _Function_class_(FN_XLATE)
// ULONG
// FASTCALL
// EXLATEOBJ_iXlateRGBto555(PEXLATEOBJ pxlo, ULONG iColor)
// {
//     ULONG iNewColor;
//
//     /* Copy red */
//     iColor <<= 7;
//     iNewColor = iColor & 0x7C00;
//
//     /* Copy green */
//     iColor >>= 13;
//     iNewColor |= iColor & 0x3E0;
//
//     /* Copy blue */
//     iColor >>= 13;
//     iNewColor |= iColor & 0x1F;
//
//     return iNewColor;
// }
//
// _Function_class_(FN_XLATE)
// ULONG
// FASTCALL
// EXLATEOBJ_iXlateBGRto555(PEXLATEOBJ pxlo, ULONG iColor)
// {
//     ULONG iNewColor;
//
//     /* Copy blue */
//     iColor >>= 3;
//     iNewColor = iColor & 0x1f;
//
//     /* Copy green */
//     iColor >>= 3;
//     iNewColor |= (iColor & 0x3E0);
//
//     /* Copy red */
//     iColor >>= 3;
//     iNewColor |= (iColor & 0x7C00);
//
//     return iNewColor;
// }
//
// _Function_class_(FN_XLATE)
// ULONG
// FASTCALL
// EXLATEOBJ_iXlateRGBto565(PEXLATEOBJ pxlo, ULONG iColor)
// {
//     ULONG iNewColor;
//
//     /* Copy red */
//     iColor <<= 8;
//     iNewColor = iColor & 0xF800;
//
//     /* Copy green */
//     iColor >>= 13;
//     iNewColor |= iColor & 0x7E0;
//
//     /* Copy green */
//     iColor >>= 14;
//     iNewColor |= iColor & 0x1F;
//
//     return iNewColor;
// }
//
// _Function_class_(FN_XLATE)
// ULONG
// FASTCALL
// EXLATEOBJ_iXlateBGRto565(PEXLATEOBJ pxlo, ULONG iColor)
// {
//     ULONG iNewColor;
//
//     /* Copy blue */
//     iColor >>= 3;
//     iNewColor = iColor & 0x1f;
//
//     /* Copy green */
//     iColor >>= 2;
//     iNewColor |= (iColor & 0x7E0);
//
//     /* Copy red */
//     iColor >>= 3;
//     iNewColor |= (iColor & 0xF800);
//
//     return iNewColor;
// }
//
// _Function_class_(FN_XLATE)
// ULONG
// FASTCALL
// EXLATEOBJ_iXlateRGBtoPal(PEXLATEOBJ pexlo, ULONG iColor)
// {
//     return PALETTE_ulGetNearestPaletteIndex(pexlo->ppalDst, iColor);
// }
//
// _Function_class_(FN_XLATE)
// ULONG
// FASTCALL
// EXLATEOBJ_iXlate555toRGB(PEXLATEOBJ pxlo, ULONG iColor)
// {
//     ULONG iNewColor;
//
//     /* Copy blue */
//     iNewColor = gajXlate5to8[iColor & 0x1F] << 16;
//
//     /* Copy green */
//     iColor >>= 5;
//     iNewColor |= gajXlate5to8[iColor & 0x1F] << 8;
//
//     /* Copy red */
//     iColor >>= 5;
//     iNewColor |= gajXlate5to8[iColor & 0x1F];
//
//     return iNewColor;
// }
//
// _Function_class_(FN_XLATE)
// ULONG
// FASTCALL
// EXLATEOBJ_iXlate555toBGR(PEXLATEOBJ pxlo, ULONG iColor)
// {
//     ULONG iNewColor;
//
//     /* Copy blue */
//     iNewColor = gajXlate5to8[iColor & 0x1F];
//
//     /* Copy green */
//     iColor >>= 5;
//     iNewColor |= gajXlate5to8[iColor & 0x1F] << 8;
//
//     /* Copy red */
//     iColor >>= 5;
//     iNewColor |= gajXlate5to8[iColor & 0x1F] << 16;
//
//     return iNewColor;
// }
//
// _Function_class_(FN_XLATE)
// ULONG
// FASTCALL
// EXLATEOBJ_iXlate555to565(PEXLATEOBJ pxlo, ULONG iColor)
// {
//     ULONG iNewColor;
//
//     /* Copy blue */
//     iNewColor = iColor & 0x1f;
//
//     /* Copy red and green */
//     iColor <<= 1;
//     iNewColor |= iColor & 0xFFC0;
//
//     /* Duplicate highest green bit */
//     iColor >>= 5;
//     iNewColor |= (iColor & 0x20);
//
//     return iNewColor;
// }
//
// _Function_class_(FN_XLATE)
// ULONG
// FASTCALL
// EXLATEOBJ_iXlate555toPal(PEXLATEOBJ pexlo, ULONG iColor)
// {
//     iColor = EXLATEOBJ_iXlate555toRGB(pexlo, iColor);
//
//     return PALETTE_ulGetNearestPaletteIndex(pexlo->ppalDst, iColor);
// }
//
// _Function_class_(FN_XLATE)
// ULONG
// FASTCALL
// EXLATEOBJ_iXlate565to555(PEXLATEOBJ pxlo, ULONG iColor)
// {
//     ULONG iNewColor;
//
//     /* Copy blue */
//     iNewColor = iColor & 0x1f;
//
//     /* Copy red and green */
//     iColor >>= 1;
//     iNewColor |= iColor & 0x7FE0;
//
//     return iNewColor;
// }
//
// _Function_class_(FN_XLATE)
// ULONG
// FASTCALL
// EXLATEOBJ_iXlate565toRGB(PEXLATEOBJ pexlo, ULONG iColor)
// {
//     ULONG iNewColor;
//
//     /* Copy blue */
//     iNewColor = gajXlate5to8[iColor & 0x1F] << 16;
//
//     /* Copy green */
//     iColor >>= 5;
//     iNewColor |= gajXlate6to8[iColor & 0x3F] << 8;
//
//     /* Copy red */
//     iColor >>= 6;
//     iNewColor |= gajXlate5to8[iColor & 0x1F];
//
//     return iNewColor;
// }
//
// _Function_class_(FN_XLATE)
// ULONG
// FASTCALL
// EXLATEOBJ_iXlate565toBGR(PEXLATEOBJ pexlo, ULONG iColor)
// {
//     ULONG iNewColor;
//
//     /* Copy blue */
//     iNewColor = gajXlate5to8[iColor & 0x1F];
//
//     /* Copy green */
//     iColor >>= 5;
//     iNewColor |= gajXlate6to8[iColor & 0x3F] << 8;
//
//     /* Copy blue */
//     iColor >>= 6;
//     iNewColor |= gajXlate5to8[iColor & 0x1F] << 16;
//
//     return iNewColor;
// }
//
// _Function_class_(FN_XLATE)
// ULONG
// FASTCALL
// EXLATEOBJ_iXlate565toPal(EXLATEOBJ *pexlo, ULONG iColor)
// {
//     iColor = EXLATEOBJ_iXlate565toRGB(pexlo, iColor);
//
//     return PALETTE_ulGetNearestPaletteIndex(pexlo->ppalDst, iColor);
// }
//
// _Function_class_(FN_XLATE)
// ULONG
// FASTCALL
// EXLATEOBJ_iXlateShiftAndMask(PEXLATEOBJ pexlo, ULONG iColor)
// {
//     ULONG iNewColor;
//
//     iNewColor = _rotl(iColor, pexlo->ulRedShift) & pexlo->ulRedMask;
//     iNewColor |= _rotl(iColor, pexlo->ulGreenShift) & pexlo->ulGreenMask;
//     iNewColor |= _rotl(iColor, pexlo->ulBlueShift) & pexlo->ulBlueMask;
//
//     return iNewColor;
// }
//
// _Function_class_(FN_XLATE)
// ULONG
// FASTCALL
// EXLATEOBJ_iXlateBitfieldsToPal(PEXLATEOBJ pexlo, ULONG iColor)
// {
//     /* Convert bitfields to RGB */
//     iColor = EXLATEOBJ_iXlateShiftAndMask(pexlo, iColor);
//
//     /* Return nearest index */
//     return PALETTE_ulGetNearestPaletteIndex(pexlo->ppalDst, iColor);
// }
//
//
// /** Private Functions *********************************************************/
//
// VOID
// NTAPI
// EXLATEOBJ_vInitialize(
//     _Out_ PEXLATEOBJ pexlo,
//     _In_opt_ PALETTE *ppalSrc,
//     _In_opt_ PALETTE *ppalDst,
//     _In_ COLORREF crSrcBackColor,
//     _In_ COLORREF crDstBackColor,
//     _In_ COLORREF crDstForeColor)
// {
//     ULONG cEntries;
//     ULONG i, ulColor;
//
//     if (!ppalSrc) ppalSrc = &gpalRGB;
//     if (!ppalDst) ppalDst = &gpalRGB;
//
//     pexlo->xlo.iUniq = InterlockedIncrement((LONG*)&giUniqueXlate);
//     pexlo->xlo.cEntries = 0;
//     pexlo->xlo.flXlate = 0;
//     pexlo->xlo.pulXlate = pexlo->aulXlate;
//     pexlo->pfnXlate = EXLATEOBJ_iXlateTrivial;
//     pexlo->hColorTransform = NULL;
//     pexlo->ppalSrc = ppalSrc;
//     pexlo->ppalDst = ppalDst;
//     pexlo->xlo.iSrcType = (USHORT)ppalSrc->flFlags;
//     pexlo->xlo.iDstType = (USHORT)ppalDst->flFlags;
//     pexlo->ppalDstDc = &gpalRGB;
//
//     if (ppalDst == ppalSrc)
//     {
//         pexlo->xlo.flXlate |= XO_TRIVIAL;
//         return;
//     }
//
//     /* Check if both of the palettes are indexed */
//     if (!(ppalSrc->flFlags & PAL_INDEXED) || !(ppalDst->flFlags & PAL_INDEXED))
//     {
//         /* At least one palette is not indexed, calculate shifts/masks */
//         ULONG aulMasksSrc[3], aulMasksDst[3];
//
//         PALETTE_vGetBitMasks(ppalSrc, aulMasksSrc);
//         PALETTE_vGetBitMasks(ppalDst, aulMasksDst);
//
//         pexlo->ulRedMask = aulMasksDst[0];
//         pexlo->ulGreenMask = aulMasksDst[1];
//         pexlo->ulBlueMask = aulMasksDst[2];
//
//         pexlo->ulRedShift = CalculateShift(aulMasksSrc[0], aulMasksDst[0]);
//         pexlo->ulGreenShift = CalculateShift(aulMasksSrc[1], aulMasksDst[1]);
//         pexlo->ulBlueShift = CalculateShift(aulMasksSrc[2], aulMasksDst[2]);
//     }
//
//     if (ppalSrc->flFlags & PAL_MONOCHROME)
//     {
//         if (ppalDst->flFlags & PAL_MONOCHROME)
//         {
//             ULONG iColors[2];
//
//             /* Color mapping depends on whether the source or dest are DIBs */
//             if ((ppalSrc->flFlags & PAL_DIBSECTION) && !(ppalDst->flFlags & PAL_DIBSECTION))
//             {
//                 /* DIB -> DDB: Use the source back color */
//                 iColors[1] = PALETTE_ulGetNearestPaletteIndex(ppalSrc, crSrcBackColor);
//                 iColors[0] = iColors[1] ^ 1;
//             }
//             else if (!(ppalSrc->flFlags & PAL_DIBSECTION) && (ppalDst->flFlags & PAL_DIBSECTION))
//             {
//                 /* DDB -> DIB: Use the dest DC's back and fore color (black -> fore, white -> back) */
//                 ULONG iBlack = PALETTE_ulGetNearestPaletteIndex(ppalSrc, 0x000000);
//                 ULONG iWhite = iBlack ^ 1;
//                 iColors[iBlack] = PALETTE_ulGetNearestPaletteIndex(ppalDst, crDstForeColor);
//                 iColors[iWhite] = PALETTE_ulGetNearestPaletteIndex(ppalDst, crDstBackColor);
//             }
//             else
//             {
//                 /* Either both are DIBs or both are DDBs, fore/back colors are ignored */
//                 ULONG rgbColor0 = PALETTE_ulGetRGBColorFromIndex(ppalSrc, 0);
//                 ULONG rgbColor1 = PALETTE_ulGetRGBColorFromIndex(ppalSrc, 1);
//                 iColors[0] = PALETTE_ulGetNearestPaletteIndex(ppalDst, rgbColor0);
//                 iColors[1] = PALETTE_ulGetNearestPaletteIndex(ppalDst, rgbColor1);
//             }
//
//             /* Check for one of 4 cases: trivial, invert, to-0, to-1 */
//             if ((iColors[0] == 0) && (iColors[1] == 1))
//             {
//                 pexlo->pfnXlate = EXLATEOBJ_iXlateTrivial;
//                 pexlo->xlo.flXlate = XO_TRIVIAL;
//             }
//             else if ((iColors[0] == 1) && (iColors[1] == 0))
//             {
//                 pexlo->pfnXlate = EXLATEOBJ_iXlateMonoInvert;
//             }
//             else if (iColors[0] == 0) // && (iColors[1] == 0)
//             {
//                 pexlo->pfnXlate = EXLATEOBJ_iXlateMonoTo0;
//             }
//             else // if ((iColors[0] == 1) && (iColors[1] == 1))
//             {
//                 pexlo->pfnXlate = EXLATEOBJ_iXlateMonoTo1;
//             }
//         }
//         else
//         {
//             /* Mono to color translation uses a 2-entry table */
//             pexlo->pfnXlate = EXLATEOBJ_iXlateTable;
//             pexlo->xlo.flXlate |= XO_TABLE;
//             pexlo->xlo.cEntries = 2;
//
//             /* Check if source is a DIB or direct translation was explicitly
//                requested by passing CLR_INALID as crDstBackColor */
//             if ((ppalSrc->flFlags & PAL_DIBSECTION) ||
//                 (crDstBackColor == CLR_INVALID))
//             {
//                 /* Direct translation, use the DIB color table */
//                 ULONG rgbColor0 = PALETTE_ulGetRGBColorFromIndex(ppalSrc, 0);
//                 ULONG rgbColor1 = PALETTE_ulGetRGBColorFromIndex(ppalSrc, 1);
//                 pexlo->aulXlate[0] =
//                     PALETTE_ulGetNearestIndex(ppalDst, rgbColor0);
//                 pexlo->aulXlate[1] =
//                     PALETTE_ulGetNearestIndex(ppalDst, rgbColor1);
//             }
//             else
//             {
//                 /* Use the dest DCs back and fore color */
//                 ULONG iBlack = PALETTE_ulGetNearestPaletteIndex(ppalSrc, 0x000000);
//                 ULONG iWhite = iBlack ^ 1;
//                 pexlo->xlo.pulXlate[iBlack] = PALETTE_ulGetNearestIndex(ppalDst, crDstForeColor);
//                 pexlo->xlo.pulXlate[iWhite] = PALETTE_ulGetNearestIndex(ppalDst, crDstBackColor);
//             }
//         }
//     }
//     else if (ppalDst->flFlags & PAL_MONOCHROME)
//     {
//         pexlo->pfnXlate = EXLATEOBJ_iXlateToMono;
//         pexlo->xlo.flXlate |= XO_TO_MONO;
//         pexlo->xlo.cEntries = 1;
//
//         if (ppalSrc->flFlags & PAL_INDEXED)
//         {
//             pexlo->aulXlate[0] =
//                 PALETTE_ulGetNearestPaletteIndex(ppalSrc, crSrcBackColor);
//         }
//         else if (ppalSrc->flFlags & PAL_RGB)
//         {
//             pexlo->aulXlate[0] = crSrcBackColor;
//
//             /* Check if direct translation was requested (e.g. NtGdiSetPixel) */
//             if (crSrcBackColor == CLR_INVALID)
//             {
//                 EXLATEOBJ_vInitRGBToMono(pexlo,
//                                          PALETTE_ulGetRGBColorFromIndex(ppalDst, 0),
//                                          PALETTE_ulGetRGBColorFromIndex(ppalDst, 1),
//                                          FALSE);
//             }
//         }
//         else if (ppalSrc->flFlags & PAL_BGR)
//         {
//             pexlo->aulXlate[0] = RGB(GetBValue(crSrcBackColor),
//                                      GetGValue(crSrcBackColor),
//                                      GetRValue(crSrcBackColor));
//
//             /* Check if direct translation was requested (e.g. NtGdiSetPixel) */
//             if (crSrcBackColor == CLR_INVALID)
//             {
//                 EXLATEOBJ_vInitRGBToMono(pexlo,
//                                          PALETTE_ulGetRGBColorFromIndex(ppalDst, 0),
//                                          PALETTE_ulGetRGBColorFromIndex(ppalDst, 1),
//                                          TRUE);
//             }
//         }
//         else if (ppalSrc->flFlags & PAL_BITFIELDS)
//         {
//             PALETTE_vGetBitMasks(ppalSrc, &pexlo->ulRedMask);
//             pexlo->ulRedShift = CalculateShift(RGB(0xFF,0,0), pexlo->ulRedMask);
//             pexlo->ulGreenShift = CalculateShift(RGB(0,0xFF,0), pexlo->ulGreenMask);
//             pexlo->ulBlueShift = CalculateShift(RGB(0,0,0xFF), pexlo->ulBlueMask);
//
//             pexlo->aulXlate[0] = EXLATEOBJ_iXlateShiftAndMask(pexlo, crSrcBackColor);
//         }
//     }
//     else if (ppalSrc->flFlags & PAL_INDEXED)
//     {
//         cEntries = ppalSrc->NumColors;
//
//         /* Allocate buffer if needed */
//         if (cEntries > 6)
//         {
//             pexlo->xlo.pulXlate = EngAllocMem(0,
//                                               cEntries * sizeof(ULONG),
//                                               GDITAG_PXLATE);
//             if (!pexlo->xlo.pulXlate)
//             {
//                 DPRINT1("Could not allocate pulXlate buffer.\n");
//                 pexlo->pfnXlate = EXLATEOBJ_iXlateTrivial;
//                 pexlo->xlo.flXlate = XO_TRIVIAL;
//                 return;
//             }
//         }
//
//         pexlo->pfnXlate = EXLATEOBJ_iXlateTable;
//         pexlo->xlo.cEntries = cEntries;
//         pexlo->xlo.flXlate |= XO_TABLE;
//
//         if (ppalDst->flFlags & PAL_INDEXED)
//         {
//             ULONG cDiff = 0;
//
//             for (i = 0; i < cEntries; i++)
//             {
//                 ulColor = RGB(ppalSrc->IndexedColors[i].peRed,
//                               ppalSrc->IndexedColors[i].peGreen,
//                               ppalSrc->IndexedColors[i].peBlue);
//
//                 pexlo->xlo.pulXlate[i] =
//                     PALETTE_ulGetNearestPaletteIndex(ppalDst, ulColor);
//
//                 if (pexlo->xlo.pulXlate[i] != i) cDiff++;
//             }
//
//             /* Check if we have only trivial mappings */
//             if (cDiff == 0)
//             {
//                 if (pexlo->xlo.pulXlate != pexlo->aulXlate)
//                 {
//                     EngFreeMem(pexlo->xlo.pulXlate);
//                     pexlo->xlo.pulXlate = pexlo->aulXlate;
//                 }
//                 pexlo->pfnXlate = EXLATEOBJ_iXlateTrivial;
//                 pexlo->xlo.flXlate = XO_TRIVIAL;
//                 pexlo->xlo.cEntries = 0;
//                 return;
//             }
//         }
//         else
//         {
//             for (i = 0; i < pexlo->xlo.cEntries; i++)
//             {
//                 ulColor = RGB(ppalSrc->IndexedColors[i].peRed,
//                               ppalSrc->IndexedColors[i].peGreen,
//                               ppalSrc->IndexedColors[i].peBlue);
//                 pexlo->xlo.pulXlate[i] = PALETTE_ulGetNearestIndex(ppalDst, ulColor);
//             }
//         }
//     }
//     else if (ppalSrc->flFlags & PAL_RGB)
//     {
//         if (ppalDst->flFlags & PAL_INDEXED)
//             pexlo->pfnXlate = EXLATEOBJ_iXlateRGBtoPal;
//
//         else if (ppalDst->flFlags & PAL_BGR)
//             pexlo->pfnXlate = EXLATEOBJ_iXlateRGBtoBGR;
//
//         else if (ppalDst->flFlags & PAL_RGB16_555)
//             pexlo->pfnXlate = EXLATEOBJ_iXlateRGBto555;
//
//         else if (ppalDst->flFlags & PAL_RGB16_565)
//             pexlo->pfnXlate = EXLATEOBJ_iXlateRGBto565;
//
//         else if (ppalDst->flFlags & PAL_BITFIELDS)
//             pexlo->pfnXlate = EXLATEOBJ_iXlateShiftAndMask;
//     }
//     else if (ppalSrc->flFlags & PAL_BGR)
//     {
//         if (ppalDst->flFlags & PAL_INDEXED)
//             pexlo->pfnXlate = EXLATEOBJ_iXlateBitfieldsToPal;
//
//         else if (ppalDst->flFlags & PAL_RGB)
//             /* The inverse function works the same */
//             pexlo->pfnXlate = EXLATEOBJ_iXlateRGBtoBGR;
//
//         else if (ppalDst->flFlags & PAL_RGB16_555)
//             pexlo->pfnXlate = EXLATEOBJ_iXlateBGRto555;
//
//         else if (ppalDst->flFlags & PAL_RGB16_565)
//             pexlo->pfnXlate = EXLATEOBJ_iXlateBGRto565;
//
//         else if (ppalDst->flFlags & PAL_BITFIELDS)
//             pexlo->pfnXlate = EXLATEOBJ_iXlateShiftAndMask;
//     }
//     else if (ppalSrc->flFlags & PAL_RGB16_555)
//     {
//         if (ppalDst->flFlags & PAL_INDEXED)
//             pexlo->pfnXlate = EXLATEOBJ_iXlate555toPal;
//
//         else if (ppalDst->flFlags & PAL_RGB)
//             pexlo->pfnXlate = EXLATEOBJ_iXlate555toRGB;
//
//         else if (ppalDst->flFlags & PAL_BGR)
//             pexlo->pfnXlate = EXLATEOBJ_iXlate555toBGR;
//
//         else if (ppalDst->flFlags & PAL_RGB16_565)
//             pexlo->pfnXlate = EXLATEOBJ_iXlate555to565;
//
//         else if (ppalDst->flFlags & PAL_BITFIELDS)
//             pexlo->pfnXlate = EXLATEOBJ_iXlateShiftAndMask;
//     }
//     else if (ppalSrc->flFlags & PAL_RGB16_565)
//     {
//         if (ppalDst->flFlags & PAL_INDEXED)
//             pexlo->pfnXlate = EXLATEOBJ_iXlate565toPal;
//
//         else if (ppalDst->flFlags & PAL_RGB)
//             pexlo->pfnXlate = EXLATEOBJ_iXlate565toRGB;
//
//         else if (ppalDst->flFlags & PAL_BGR)
//             pexlo->pfnXlate = EXLATEOBJ_iXlate565toBGR;
//
//         else if (ppalDst->flFlags & PAL_RGB16_555)
//             pexlo->pfnXlate = EXLATEOBJ_iXlate565to555;
//
//         else if (ppalDst->flFlags & PAL_BITFIELDS)
//             pexlo->pfnXlate = EXLATEOBJ_iXlateShiftAndMask;
//     }
//     else if (ppalSrc->flFlags & PAL_BITFIELDS)
//     {
//         if (ppalDst->flFlags & PAL_INDEXED)
//             pexlo->pfnXlate = EXLATEOBJ_iXlateBitfieldsToPal;
//         else
//             pexlo->pfnXlate = EXLATEOBJ_iXlateShiftAndMask;
//     }
//
//     /* Check for a trivial shift and mask operation */
//     if (pexlo->pfnXlate == EXLATEOBJ_iXlateShiftAndMask &&
//         !pexlo->ulRedShift && !pexlo->ulGreenShift && !pexlo->ulBlueShift)
//     {
//         pexlo->pfnXlate = EXLATEOBJ_iXlateTrivial;
//     }
//
//     /* Check for trivial xlate */
//     if (pexlo->pfnXlate == EXLATEOBJ_iXlateTrivial)
//         pexlo->xlo.flXlate = XO_TRIVIAL;
//     else
//         pexlo->xlo.flXlate &= ~XO_TRIVIAL;
// }
//
// VOID
// NTAPI
// EXLATEOBJ_vInitXlateFromDCs(
//     _Out_ EXLATEOBJ* pexlo,
//     _In_ PDC pdcSrc,
//     _In_ PDC pdcDst)
// {
//     PSURFACE psurfDst, psurfSrc;
//
//     psurfDst = pdcDst->dclevel.pSurface;
//     psurfSrc = pdcSrc->dclevel.pSurface;
//
//     /* Normal initialisation. No surface means DEFAULT_BITMAP */
//     EXLATEOBJ_vInitialize(pexlo,
//                           psurfSrc ? psurfSrc->ppal : gppalMono,
//                           psurfDst ? psurfDst->ppal : gppalMono,
//                           pdcSrc->pdcattr->crBackgroundClr,
//                           pdcDst->pdcattr->crBackgroundClr,
//                           pdcDst->pdcattr->crForegroundClr);
//
//     pexlo->ppalDstDc = pdcDst->dclevel.ppal;
// }
//
// VOID
// NTAPI
// EXLATEOBJ_vInitXlateFromDCsEx(
//     _Out_ EXLATEOBJ* pexlo,
//     _In_ PDC pdcSrc,
//     _In_ PDC pdcDst,
//     _In_ COLORREF crBackColor)
// {
//     PSURFACE psurfDst, psurfSrc;
//
//     psurfDst = pdcDst->dclevel.pSurface;
//     psurfSrc = pdcSrc->dclevel.pSurface;
//
//     if (crBackColor == CLR_INVALID)
//     {
//         crBackColor = pdcSrc->pdcattr->crBackgroundClr;
//     }
//
//     /* Normal initialisation. No surface means DEFAULT_BITMAP */
//     EXLATEOBJ_vInitialize(pexlo,
//                           psurfSrc ? psurfSrc->ppal : gppalMono,
//                           psurfDst ? psurfDst->ppal : gppalMono,
//                           crBackColor,
//                           pdcDst->pdcattr->crBackgroundClr,
//                           pdcDst->pdcattr->crForegroundClr);
//
//     pexlo->ppalDstDc = pdcDst->dclevel.ppal;
// }
//
//
// VOID NTAPI EXLATEOBJ_vInitSrcMonoXlate(
//     PEXLATEOBJ pexlo,
//     PPALETTE ppalDst,
//     COLORREF crBackgroundClr,
//     COLORREF crForegroundClr)
// {
//     /* Normal initialisation, with mono palette as source */
//     EXLATEOBJ_vInitialize(pexlo,
//                           gppalMono,
//                           ppalDst,
//                           0,
//                           crBackgroundClr,
//                           crForegroundClr);
// }
//
// VOID
// NTAPI
// EXLATEOBJ_vCleanup(
//     _Inout_ PEXLATEOBJ pexlo)
// {
//     if (pexlo->xlo.pulXlate != pexlo->aulXlate)
//     {
//         EngFreeMem(pexlo->xlo.pulXlate);
//     }
//     pexlo->xlo.pulXlate = pexlo->aulXlate;
// }
//
// /** Public DDI Functions ******************************************************/
//
// #undef XLATEOBJ_iXlate
// ULONG
// NTAPI
// XLATEOBJ_iXlate(
//     _In_ XLATEOBJ *pxlo,
//     _In_ ULONG iColor)
// {
//     PEXLATEOBJ pexlo = (PEXLATEOBJ)pxlo;
//
//     if (!pxlo)
//         return iColor;
//
//     /* Call the iXlate function */
//     return pexlo->pfnXlate(pexlo, iColor);
// }
//
// ULONG
// NTAPI
// XLATEOBJ_cGetPalette(
//     _In_ XLATEOBJ *pxlo,
//     _In_ ULONG iPal,
//     _In_ ULONG cPal,
//     _Out_cap_(cPal) ULONG *pPalOut)
// {
//     PEXLATEOBJ pexlo = (PEXLATEOBJ)pxlo;
//     PPALETTE ppal;
//     ULONG i;
//
//     if (!pxlo)
//     {
//         return 0;
//     }
//
//     if (iPal > 5)
//     {
//        DPRINT1("XLATEOBJ_cGetPalette called with wrong iPal: %lu\n", iPal);
//        return 0;
//     }
//
//     /* Get the requested palette */
//     if (iPal == XO_DESTDCPALETTE)
//     {
//         ppal = pexlo->ppalDstDc;
//     }
//     else if (iPal == XO_SRCPALETTE || iPal == XO_SRCBITFIELDS)
//     {
//         ppal = pexlo->ppalSrc;
//     }
//     else
//     {
//         ppal = pexlo->ppalDst;
//     }
//
//     /* Verify palette type match */
//     if (!ppal ||
//         ((iPal == XO_SRCPALETTE || iPal == XO_DESTPALETTE)
//             && !(ppal->flFlags & PAL_INDEXED)) ||
//         ((iPal == XO_SRCBITFIELDS || iPal == XO_DESTBITFIELDS)
//             && !(ppal->flFlags & PAL_BITFIELDS)))
//     {
//         return 0;
//     }
//
//     if(!pPalOut)
//     {
//         return ppal->NumColors;
//     }
//
//     /* Copy the values into the buffer */
//     if (ppal->flFlags & PAL_INDEXED)
//     {
//         cPal = min(cPal, ppal->NumColors);
//         for (i = 0; i < cPal; i++)
//         {
//             pPalOut[i] = RGB(ppal->IndexedColors[i].peRed,
//                              ppal->IndexedColors[i].peGreen,
//                              ppal->IndexedColors[i].peBlue);
//         }
//     }
//     else
//     {
//         // FIXME: should use the above code
//         pPalOut[0] = ppal->RedMask;
//         pPalOut[1] = ppal->GreenMask;
//         pPalOut[2] = ppal->BlueMask;
//     }
//
//     return cPal;
// }
//
// HANDLE
// NTAPI
// XLATEOBJ_hGetColorTransform(
//     _In_ XLATEOBJ *pxlo)
// {
//     PEXLATEOBJ pexlo = (PEXLATEOBJ)pxlo;
//     return pexlo->hColorTransform;
// }
//
// PULONG
// NTAPI
// XLATEOBJ_piVector(
//     _In_ XLATEOBJ *pxlo)
// {
//     if (pxlo->iSrcType == PAL_INDEXED)
//     {
//         return pxlo->pulXlate;
//     }
//
//     return NULL;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
