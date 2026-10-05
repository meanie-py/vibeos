use win32k_sys::win32k::*;
use ntkrnl_sys::ddk::{ZwMapViewOfSection, ZwQuerySystemInformation, ZwUnmapViewOfSection, ExAllocatePoolWithTag, ExFreePoolWithTag};
use ntkrnl_sys::base::STATUS_SUCCESS;
use core::ffi::c_void;
use crate::floatobj::*;
use crate::surface::gajBitsPerFormat;
use crate::dib_helpers::abs;

// File-local, matching copy.rs / safe_user_io.rs in this crate.
const STATUS_ACCESS_VIOLATION: NTSTATUS = 0xC0000005_u32 as NTSTATUS;
const STATUS_INVALID_PARAMETER: NTSTATUS = 0xC000000D_u32 as NTSTATUS;
const STATUS_INSUFFICIENT_RESOURCES: NTSTATUS = 0xC000009A_u32 as NTSTATUS;

// ---- pool tags + small helpers ----
// C: win32ss/user/ntuser/tags.h `#define TAG_DIB ' BID'` (big-endian).
const TAG_DIB: u32 = u32::from_be_bytes(*b" BID");
const TAG_GBMI: u32 = u32::from_le_bytes(*b"GBmi");
#[inline(always)] fn abs_i32(x: i32) -> i32 { x.wrapping_abs() }
#[inline(always)] fn min_u32(a: u32, b: u32) -> u32 { if a < b { a } else { b } }
#[inline(always)] fn NT_SUCCESS(s: NTSTATUS) -> bool { s >= 0 }
#[inline(always)] unsafe fn RtlCopyMemory(d: *mut c_void, sc: *const c_void, n: usize) { core::ptr::copy_nonoverlapping(sc as *const u8, d as *mut u8, n); }
#[inline(always)] unsafe fn RtlZeroMemory(d: *mut c_void, n: usize) { core::ptr::write_bytes(d as *mut u8, 0, n); }

// ---- FORCEINLINE / macro helpers (no exported symbol) ----
#[inline(always)] fn RGB(r: u8, g: u8, b: u8) -> COLORREF { ((r as u32) | ((g as u32) << 8) | ((b as u32) << 16)) as COLORREF }
#[inline(always)] fn WIN32_ROP3_TO_ENG_ROP4(dwRop4: u32) -> u32 { ((dwRop4 & 0x00FF0000) >> 16) | ((dwRop4 & 0x00FF0000) >> 8) }
#[inline(always)] unsafe fn RECTL_vOffsetRect(prcl: *mut RECTL, cx: i32, cy: i32) { (*prcl).left += cx; (*prcl).right += cx; (*prcl).top += cy; (*prcl).bottom += cy; }
#[inline(always)] unsafe fn SURFACE_ShareLockSurface(h: HBITMAP) -> PSURFACE { GDIOBJ_ShareLockObj(h as HGDIOBJ, GDI_OBJECT_TYPE_BITMAP as DWORD) as PSURFACE }
#[inline(always)] unsafe fn SURFACE_ShareUnlockSurface(p: PSURFACE) { GDIOBJ_vDereferenceObject(p as POBJ); }
#[inline(always)] unsafe fn PALETTE_ShareUnlockPalette(p: PPALETTE) { GDIOBJ_vDereferenceObject(core::ptr::addr_of_mut!((*p).BaseObject)); }
#[inline(always)] unsafe fn PALETTE_UnlockPalette(p: PPALETTE) { GDIOBJ_vUnlockObject(p as POBJ); }
// IntLPtoDP (coord.h static inline) + INTERNAL_APPLY_MATRIX (FORCEINLINE) — no symbols; mirror dcutil.rs.
#[inline(always)] unsafe fn INTERNAL_APPLY_MATRIX(matrix: PMATRIX, points: LPPOINT, mut count: u32) -> u8 {
    while count != 0 {
        count -= 1;
        let p = points.offset(count as isize);
        let mut x: FLOATOBJ = core::mem::zeroed();
        let mut y: FLOATOBJ = core::mem::zeroed();
        let mut tmp: FLOATOBJ;
        FLOATOBJ_SetLong(core::ptr::addr_of_mut!(x), (*p).x);
        FLOATOBJ_Mul(core::ptr::addr_of_mut!(x), core::ptr::addr_of_mut!((*matrix).efM11));
        tmp = (*matrix).efM21;
        FLOATOBJ_MulLong(core::ptr::addr_of_mut!(tmp), (*p).y);
        FLOATOBJ_Add(core::ptr::addr_of_mut!(x), core::ptr::addr_of_mut!(tmp));
        FLOATOBJ_Add(core::ptr::addr_of_mut!(x), core::ptr::addr_of_mut!((*matrix).efDx));
        FLOATOBJ_SetLong(core::ptr::addr_of_mut!(y), (*p).y);
        FLOATOBJ_Mul(core::ptr::addr_of_mut!(y), core::ptr::addr_of_mut!((*matrix).efM22));
        tmp = (*matrix).efM12;
        FLOATOBJ_MulLong(core::ptr::addr_of_mut!(tmp), (*p).x);
        FLOATOBJ_Add(core::ptr::addr_of_mut!(y), core::ptr::addr_of_mut!(tmp));
        FLOATOBJ_Add(core::ptr::addr_of_mut!(y), core::ptr::addr_of_mut!((*matrix).efDy));
        if FLOATOBJ_bConvertToLong(core::ptr::addr_of!(x), core::ptr::addr_of_mut!((*p).x)) == 0 { return 0; }
        if FLOATOBJ_bConvertToLong(core::ptr::addr_of!(y), core::ptr::addr_of_mut!((*p).y)) == 0 { return 0; }
    }
    1
}
#[inline(always)] unsafe fn IntLPtoDP(pdc: PDC, ppt: *mut POINTL, count: u32) -> BOOL {
    DC_vUpdateWorldToDevice(pdc);
    INTERNAL_APPLY_MATRIX(core::ptr::addr_of_mut!((*(*pdc).pdcattr).mxWorldToDevice) as PMATRIX, ppt as LPPOINT, count) as BOOL
}

// more macro/FORCEINLINE helpers
#[inline(always)] fn BitsPerFormat<T: Into<i64>>(fmt: T) -> ULONG { gajBitsPerFormat[Into::<i64>::into(fmt) as usize] as ULONG }
#[inline(always)] fn WIDTH_BYTES_ALIGN16<A: Into<i64>, B: Into<i64>>(cx: A, bpp: B) -> ULONG { let cx: i64 = cx.into(); let bpp: i64 = bpp.into(); (((cx * bpp + 15) & !15) >> 3) as ULONG }
#[inline(always)] fn WIDTH_BYTES_ALIGN32<A: Into<i64>, B: Into<i64>>(cx: A, bpp: B) -> ULONG { let cx: i64 = cx.into(); let bpp: i64 = bpp.into(); (((cx * bpp + 31) & !31) >> 3) as ULONG }
#[inline(always)] fn UlongToPtr(ul: u32) -> *mut c_void { ul as usize as *mut c_void }
#[inline(always)] unsafe fn NtCurrentProcess() -> HANDLE { (-1isize as usize) as HANDLE }
#[inline(always)] unsafe fn PALETTE_ShareLockPalette(h: HPALETTE) -> PPALETTE { GDIOBJ_ShareLockObj(h as HGDIOBJ, GDI_OBJECT_TYPE_PALETTE as DWORD) as PPALETTE }
#[inline(always)] fn ROP4_FROM_INDEX(index: u32) -> u32 { index | (index << 8) }
#[inline(always)] unsafe fn RECTL_vSetRect(prcl: *mut RECTL, l: i32, t: i32, r: i32, b: i32) { (*prcl).left = l; (*prcl).top = t; (*prcl).right = r; (*prcl).bottom = b; }

// ---- real exported symbols ----
// BitmapFormat, SetLastNtError, RECTL_vMakeWellOrdered = FASTCALL.
#[cfg(target_arch = "x86")] extern "fastcall" { fn BitmapFormat(cBits: ULONG, iCompression: ULONG) -> ULONG; fn SetLastNtError(Status: NTSTATUS); fn RECTL_vMakeWellOrdered(prcl: *mut RECTL); }
#[cfg(not(target_arch = "x86"))] extern "C" { fn BitmapFormat(cBits: ULONG, iCompression: ULONG) -> ULONG; fn SetLastNtError(Status: NTSTATUS); fn RECTL_vMakeWellOrdered(prcl: *mut RECTL); }
// Real DIB engine helpers (still-C / sibling Rust) + Mm probe.
// dib1bpp.rs defines DIB_1BPP_HLine cdecl (`extern "C"`) — decl must match
// (§9.30 UB-fold class).
extern "C" {
    fn DIB_1BPP_HLine(so: *mut SURFOBJ, x1: LONG, x2: LONG, y: LONG, c: ULONG);
}

// EXLATEOBJ_* = ENGAPI/NTAPI; SafeProbe* = NTAPI, SIZE_T length.
extern "system" {
    fn EXLATEOBJ_vInitialize(pexlo: *mut EXLATEOBJ, ppalSrc: PPALETTE, ppalDst: PPALETTE, crBackgroundClr: COLORREF, crForegroundClr: COLORREF, iForeIndex: ULONG);
    fn EXLATEOBJ_vCleanup(pexlo: *mut EXLATEOBJ);
    fn SafeProbeForRead(a: *const core::ffi::c_void, l: usize, al: u32) -> i32;
    fn SafeProbeForWrite(a: *mut core::ffi::c_void, l: usize, al: u32) -> i32;
}

pub static DefLogPaletteQuads: [RGBQUAD; 20] = [
    RGBQUAD { rgbBlue: 0x00, rgbGreen: 0x00, rgbRed: 0x00, rgbReserved: 0x00 },
    RGBQUAD { rgbBlue: 0x00, rgbGreen: 0x00, rgbRed: 0x80, rgbReserved: 0x00 },
    RGBQUAD { rgbBlue: 0x00, rgbGreen: 0x80, rgbRed: 0x00, rgbReserved: 0x00 },
    RGBQUAD { rgbBlue: 0x00, rgbGreen: 0x80, rgbRed: 0x80, rgbReserved: 0x00 },
    RGBQUAD { rgbBlue: 0x80, rgbGreen: 0x00, rgbRed: 0x00, rgbReserved: 0x00 },
    RGBQUAD { rgbBlue: 0x80, rgbGreen: 0x00, rgbRed: 0x80, rgbReserved: 0x00 },
    RGBQUAD { rgbBlue: 0x80, rgbGreen: 0x80, rgbRed: 0x00, rgbReserved: 0x00 },
    RGBQUAD { rgbBlue: 0xc0, rgbGreen: 0xc0, rgbRed: 0xc0, rgbReserved: 0x00 },
    RGBQUAD { rgbBlue: 0xc0, rgbGreen: 0xdc, rgbRed: 0xc0, rgbReserved: 0x00 },
    RGBQUAD { rgbBlue: 0xf0, rgbGreen: 0xca, rgbRed: 0xa6, rgbReserved: 0x00 },
    RGBQUAD { rgbBlue: 0xf0, rgbGreen: 0xfb, rgbRed: 0xff, rgbReserved: 0x00 },
    RGBQUAD { rgbBlue: 0xa4, rgbGreen: 0xa0, rgbRed: 0xa0, rgbReserved: 0x00 },
    RGBQUAD { rgbBlue: 0x80, rgbGreen: 0x80, rgbRed: 0x80, rgbReserved: 0x00 },
    RGBQUAD { rgbBlue: 0x00, rgbGreen: 0x00, rgbRed: 0xff, rgbReserved: 0x00 },
    RGBQUAD { rgbBlue: 0x00, rgbGreen: 0xff, rgbRed: 0x00, rgbReserved: 0x00 },
    RGBQUAD { rgbBlue: 0x00, rgbGreen: 0xff, rgbRed: 0xff, rgbReserved: 0x00 },
    RGBQUAD { rgbBlue: 0xff, rgbGreen: 0x00, rgbRed: 0x00, rgbReserved: 0x00 },
    RGBQUAD { rgbBlue: 0xff, rgbGreen: 0x00, rgbRed: 0xff, rgbReserved: 0x00 },
    RGBQUAD { rgbBlue: 0xff, rgbGreen: 0xff, rgbRed: 0x00, rgbReserved: 0x00 },
    RGBQUAD { rgbBlue: 0xff, rgbGreen: 0xff, rgbRed: 0xff, rgbReserved: 0x00 },
];

#[no_mangle]
pub unsafe extern "system" fn CreateDIBPalette(
    pbmi: *const BITMAPINFO,
    pdc: PDC,
    iUsage: u32,
) -> PPALETTE {
    let mut ppal: PPALETTE;
    let mut i: u32;
    let cBitsPixel: u32;
    let mut cColors: u32;

    if (*pbmi).bmiHeader.biSize < core::mem::size_of::<BITMAPINFOHEADER>() as u32 {
        let pbci = pbmi as PBITMAPCOREINFO;
        cBitsPixel = (*pbci).bmciHeader.bcBitCount as u32;
    } else {
        cBitsPixel = (*pbmi).bmiHeader.biBitCount as u32;
    }

    if cBitsPixel <= 8 {
        cColors = 1u32 << cBitsPixel;

        ppal = PALETTE_AllocPalette(
            (PAL_INDEXED | PAL_DIBSECTION) as u32,
            cColors,
            core::ptr::null_mut(),
            0,
            0,
            0,
        );
        if ppal.is_null() {
            return core::ptr::null_mut();
        }

        if ((*pbmi).bmiHeader.biSize >= core::mem::size_of::<BITMAPINFOHEADER>() as u32)
            && ((*pbmi).bmiHeader.biClrUsed != 0)
        {
            cColors = min_u32(cColors, (*pbmi).bmiHeader.biClrUsed);
        }

        if iUsage == DIB_PAL_COLORS as u32 {
            let crColor: COLORREF;
            let pwColors = (pbmi as *const u8).add((*pbmi).bmiHeader.biSize as usize) as PWORD;

            let ppalDC: PPALETTE = if !pdc.is_null() {
                (*pdc).dclevel.ppal
            } else {
                gppalDefault
            };

            i = 0;
            while i < cColors {
                let wIndex = (*pwColors.add(i as usize)) % (*ppalDC).NumColors as u16;
                let crColor = PALETTE_ulGetRGBColorFromIndex(ppalDC, wIndex as u32);
                PALETTE_vSetRGBColorForIndex(ppal, i, crColor);
                i += 1;
            }
            let _ = crColor;
        } else if iUsage == DIB_PAL_BRUSHHACK as u32 {
            let pwColors = (pbmi as *const u8).add((*pbmi).bmiHeader.biSize as usize) as PWORD;

            i = 0;
            while i < cColors {
                PALETTE_vSetRGBColorForIndex(ppal, i, *pwColors.add(i as usize) as COLORREF);
                i += 1;
            }

            (*ppal).flFlags |= PAL_BRUSHHACK as u32;
        } else if (*pbmi).bmiHeader.biSize >= core::mem::size_of::<BITMAPINFOHEADER>() as u32 {
            let prgb = (pbmi as *const u8).add((*pbmi).bmiHeader.biSize as usize) as *const RGBQUAD;
            let mut colors: [RGBQUAD; 256] = [RGBQUAD {
                rgbBlue: 0,
                rgbGreen: 0,
                rgbRed: 0,
                rgbReserved: 0,
            }; 256];

            RtlCopyMemory(
                colors.as_mut_ptr() as *mut _,
                prgb as *mut _,
                (cColors as usize) * core::mem::size_of::<RGBQUAD>(),
            );

            i = 0;
            while i < cColors {
                let crColor = RGB(
                    colors[i as usize].rgbRed,
                    colors[i as usize].rgbGreen,
                    colors[i as usize].rgbBlue,
                );
                PALETTE_vSetRGBColorForIndex(ppal, i, crColor);
                i += 1;
            }
        } else {
            let prgb = (pbmi as *const u8).add((*pbmi).bmiHeader.biSize as usize) as *const RGBTRIPLE;

            i = 0;
            while i < cColors {
                let rgb = *prgb.add(i as usize);
                let crColor = RGB(rgb.rgbtRed, rgb.rgbtGreen, rgb.rgbtBlue);
                PALETTE_vSetRGBColorForIndex(ppal, i, crColor);
                i += 1;
            }
        }
    } else {
        let flRedMask: u32;
        let flGreenMask: u32;
        let flBlueMask: u32;

        if ((*pbmi).bmiHeader.biSize >= core::mem::size_of::<BITMAPINFOHEADER>() as u32)
            && ((*pbmi).bmiHeader.biCompression == 3 /* BI_BITFIELDS */)
        {
            if (*pbmi).bmiHeader.biSize >= 108 /* sizeof(BITMAPV4HEADER) */ {
                let pbmV4Header = core::ptr::addr_of!((*pbmi).bmiHeader) as *const u8;
                // bV4RedMask offset = 40, bV4GreenMask = 44, bV4BlueMask = 48
                flRedMask = *(pbmV4Header.add(40) as *const u32);
                flGreenMask = *(pbmV4Header.add(44) as *const u32);
                flBlueMask = *(pbmV4Header.add(48) as *const u32);
            } else {
                let pdwColors = (pbmi as *const u8).add((*pbmi).bmiHeader.biSize as usize) as *const u32;
                flRedMask = *pdwColors;
                flGreenMask = *pdwColors.add(1);
                flBlueMask = *pdwColors.add(2);
            }
        } else {
            if cBitsPixel == 16 {
                flRedMask = 0x7C00;
                flGreenMask = 0x03E0;
                flBlueMask = 0x001F;
            } else {
                flRedMask = 0xFF0000;
                flGreenMask = 0x00FF00;
                flBlueMask = 0x0000FF;
            }
        }

        ppal = PALETTE_AllocPalette(
            (0x2 /* PAL_BITFIELDS */ | PAL_DIBSECTION) as u32,
            0,
            core::ptr::null_mut(),
            flRedMask,
            flGreenMask,
            flBlueMask,
        );
    }

    ppal
}

#[cfg(target_pointer_width = "32")]
unsafe extern "fastcall" fn IntSetDIBits(
    DC: PDC,
    hBitmap: HBITMAP,
    StartScan: u32,
    ScanLines: u32,
    Bits: *const core::ffi::c_void,
    cjMaxBits: u32,
    bmi: *const BITMAPINFO,
    ColorUse: u32,
) -> i32 {
    IntSetDIBits_impl(DC, hBitmap, StartScan, ScanLines, Bits, cjMaxBits, bmi, ColorUse)
}

#[cfg(not(target_pointer_width = "32"))]
unsafe extern "C" fn IntSetDIBits(
    DC: PDC,
    hBitmap: HBITMAP,
    StartScan: u32,
    ScanLines: u32,
    Bits: *const core::ffi::c_void,
    cjMaxBits: u32,
    bmi: *const BITMAPINFO,
    ColorUse: u32,
) -> i32 {
    IntSetDIBits_impl(DC, hBitmap, StartScan, ScanLines, Bits, cjMaxBits, bmi, ColorUse)
}

unsafe fn IntSetDIBits_impl(
    DC: PDC,
    hBitmap: HBITMAP,
    StartScan: u32,
    ScanLines: u32,
    Bits: *const core::ffi::c_void,
    cjMaxBits: u32,
    bmi: *const BITMAPINFO,
    ColorUse: u32,
) -> i32 {
    let SourceBitmap: HBITMAP;
    let psurfDst: PSURFACE;
    let psurfSrc: PSURFACE;
    let mut result: i32 = 0;
    let mut rcDst: RECT = core::mem::zeroed();
    let mut ptSrc: POINTL = core::mem::zeroed();
    let mut exlo: EXLATEOBJ = core::mem::zeroed();
    let mut ppalDIB: PPALETTE = core::ptr::null_mut();
    let cjSizeImage: u32;

    if bmi.is_null() || Bits.is_null() {
        return 0;
    }

    if ((*bmi).bmiHeader.biCompression == BI_RGB)
        || ((*bmi).bmiHeader.biCompression == BI_BITFIELDS)
    {
        cjSizeImage = DIB_GetDIBImageBytes(
            (*bmi).bmiHeader.biWidth,
            ScanLines as i32,
            (*bmi).bmiHeader.biBitCount as i32,
        ) as u32;
    } else if (*bmi).bmiHeader.biSizeImage != 0 {
        cjSizeImage = (*bmi).bmiHeader.biSizeImage;
    } else {
        return 0;
    }

    if cjSizeImage > cjMaxBits || cjSizeImage == 0 {
        return 0;
    }

    SourceBitmap = GreCreateBitmapEx(
        (*bmi).bmiHeader.biWidth as u32,
        ScanLines as u32,
        0,
        BitmapFormat((*bmi).bmiHeader.biBitCount as u32, (*bmi).bmiHeader.biCompression),
        if (*bmi).bmiHeader.biHeight < 0 { BMF_TOPDOWN as USHORT } else { 0 },
        cjSizeImage,
        Bits as *mut core::ffi::c_void,
        0,
    );
    if SourceBitmap.is_null() {
        EngSetLastError(ERROR_NO_SYSTEM_RESOURCES);
        return 0;
    }

    psurfDst = SURFACE_ShareLockSurface(hBitmap);
    psurfSrc = SURFACE_ShareLockSurface(SourceBitmap);

    'cleanup: {
        if !(!psurfSrc.is_null() && !psurfDst.is_null()) {
            break 'cleanup;
        }

        ppalDIB = CreateDIBPalette(bmi, DC, ColorUse);
        if ppalDIB.is_null() {
            EngSetLastError(ERROR_NO_SYSTEM_RESOURCES);
            break 'cleanup;
        }

        EXLATEOBJ_vInitialize(
            core::ptr::addr_of_mut!(exlo),
            ppalDIB,
            (*psurfDst).ppal,
            RGB(0xff, 0xff, 0xff),
            RGB(0xff, 0xff, 0xff),
            0,
        );

        rcDst.top = StartScan as i32;
        rcDst.left = 0;
        rcDst.bottom = rcDst.top + ScanLines as i32;
        rcDst.right = (*psurfDst).SurfObj.sizlBitmap.cx;
        ptSrc.x = 0;
        ptSrc.y = 0;

        result = IntEngCopyBits(
            core::ptr::addr_of_mut!((*psurfDst).SurfObj),
            core::ptr::addr_of_mut!((*psurfSrc).SurfObj),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(exlo.xlo),
            core::ptr::addr_of_mut!(rcDst) as *mut RECTL,
            core::ptr::addr_of_mut!(ptSrc),
        ) as i32;
        if result != 0 {
            result = ScanLines as i32;
        }

        EXLATEOBJ_vCleanup(core::ptr::addr_of_mut!(exlo));
    }

    if !ppalDIB.is_null() {
        PALETTE_ShareUnlockPalette(ppalDIB);
    }
    if !psurfSrc.is_null() {
        SURFACE_ShareUnlockSurface(psurfSrc);
    }
    if !psurfDst.is_null() {
        SURFACE_ShareUnlockSurface(psurfDst);
    }
    GreDeleteObject(SourceBitmap as *mut core::ffi::c_void);

    result
}

unsafe fn IntGdiCreateMaskFromRLE(
    Width: u32,
    Height: u32,
    Compression: u32,
    Bits: *const u8,
    BitsSize: u32,
) -> HBITMAP {
    let Mask: HBITMAP;
    let mut x: u32;
    let mut y: u32;
    let SurfObj: *mut SURFOBJ;
    let mut i: u32 = 0;
    let mut Data: u8;
    let mut NumPixels: u8;
    let mut ToSkip: u8;

    Mask = GreCreateBitmapEx(Width, Height, 0, BMF_1BPP, 0, 0, core::ptr::null_mut(), 0);
    if Mask.is_null() {
        return core::ptr::null_mut();
    }

    SurfObj = EngLockSurface(Mask as HSURF);
    if SurfObj.is_null() {
        GreDeleteObject(Mask as *mut _);
        return core::ptr::null_mut();
    }

    x = 0;
    y = 0;

    'done: loop {
        while i < BitsSize {
            NumPixels = *Bits.offset(i as isize);
            Data = *Bits.offset((i + 1) as isize);
            i += 2;

            if NumPixels != 0 {
                if (x + NumPixels as u32) > Width {
                    NumPixels = (Width - x) as u8;
                }

                if NumPixels == 0 {
                    continue;
                }

                DIB_1BPP_HLine(SurfObj, x as i32, (x + NumPixels as u32) as i32, y as i32, 1);
                x += NumPixels as u32;
                continue;
            }

            if Data < 3 {
                match Data {
                    0 => {
                        y += 1;
                        if y == Height {
                            break 'done;
                        }
                        x = 0;
                    }
                    1 => {
                        break 'done;
                    }
                    2 => {
                        if i >= (BitsSize - 1) {
                            break 'done;
                        }
                        x += *Bits.offset(i as isize) as u32;
                        if x > Width {
                            x = Width;
                        }
                        y += *Bits.offset((i + 1) as isize) as u32;
                        if y >= Height {
                            break 'done;
                        }
                        i += 2;
                    }
                    _ => {}
                }
                continue;
            }

            NumPixels = Data;
            if Compression == BI_RLE8 {
                ToSkip = NumPixels;
            } else {
                ToSkip = (NumPixels / 2) + (NumPixels & 1);
            }

            if (i + ToSkip as u32) > BitsSize {
                break 'done;
            }
            ToSkip = (ToSkip + 1) & !1;

            if (x + NumPixels as u32) > Width {
                NumPixels = (Width - x) as u8;
            }

            if NumPixels != 0 {
                DIB_1BPP_HLine(SurfObj, x as i32, (x + NumPixels as u32) as i32, y as i32, 1);
                x += NumPixels as u32;
            }
            i += ToSkip as u32;
        }
        break;
    }

    EngUnlockSurface(SurfObj);
    Mask
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiSetDIBitsToDeviceInternal(
    hDC: HDC,
    XDest: i32,
    YDest: i32,
    Width: u32,
    Height: u32,
    XSrc: i32,
    YSrc: i32,
    StartScan: u32,
    mut ScanLines: u32,
    mut Bits: LPBYTE,
    mut bmi: LPBITMAPINFO,
    ColorUse: u32,
    cjMaxBits: u32,
    cjMaxInfo: u32,
    bTransformCoordinates: i32,
    _hcmXform: *mut core::ffi::c_void,
) -> i32 {
    let mut ret: i32;
    // Kernel copy of the caller's pixel bits; see the capture below.
    let mut pjBitsSafe: PBYTE = core::ptr::null_mut();
    let mut pDC: PDC = core::ptr::null_mut();
    let mut hSourceBitmap: HBITMAP = core::ptr::null_mut();
    let mut hMaskBitmap: HBITMAP = core::ptr::null_mut();
    let pDestSurf: *mut SURFOBJ;
    let mut pSourceSurf: *mut SURFOBJ = core::ptr::null_mut();
    let mut pMaskSurf: *mut SURFOBJ = core::ptr::null_mut();
    let pSurf: *mut SURFACE;
    let mut rcDest: RECTL = core::mem::zeroed();
    let mut ptSource: POINTL = core::mem::zeroed();
    let mut SourceSize: SIZEL = core::mem::zeroed();
    let mut exlo: EXLATEOBJ = core::mem::zeroed();
    let mut ppalDIB: PPALETTE = core::ptr::null_mut();
    let pbmiSafe: LPBITMAPINFO;
    let bResult: i32;
    let mut Status: i32;


    const ROP4_MASK: u32 = 0xAACC; // inteng.h: SRCCOPY | (NOOP<<8) — was wrongly 0x00AA0000
    const CBM_INIT_CREATDIB: u32 = 0;
    const TAG_GDITMP: u32 = u32::from_le_bytes(*b"GTmp");

    if Bits.is_null() {
        return 0;
    }

    pbmiSafe = ExAllocatePoolWithTag(PagedPool, cjMaxInfo as _, u32::from_le_bytes(*b"GTmp")) as LPBITMAPINFO;
    if pbmiSafe.is_null() {
        return 0;
    }

    'exit: {
        Status = SafeProbeForRead(bmi as *const core::ffi::c_void, cjMaxInfo as usize, 1);
        if NT_SUCCESS(Status) {
            Status = SafeProbeForRead(Bits as *const core::ffi::c_void, cjMaxBits as usize, 1);
        }
        // The probes VALIDATE only; this capture is the access. Once it succeeds `bmi`
        // names the KERNEL copy, so every later (*bmi) read is a kernel load.
        if !NT_SUCCESS(Status)
            || !crate::copy::capture_from_user(
                pbmiSafe as *mut core::ffi::c_void,
                bmi as *const core::ffi::c_void,
                cjMaxInfo as usize,
            )
        {
            ret = 0;
            break 'exit;
        }
        bmi = pbmiSafe;

        // `Bits` was probed above, but the probe only validates. Both consumers below
        // read it directly out of the caller's page -- GreCreateBitmapEx as the pixel
        // buffer, and IntGdiCreateMaskFromRLE by WALKING the RLE stream, which is a
        // length-driven walk over user memory. Capture once and hand the copy to both.
        if cjMaxBits != 0 && !Bits.is_null() {
            pjBitsSafe = ExAllocatePoolWithTag(PagedPool, cjMaxBits as _, TAG_DIB) as PBYTE;
            if pjBitsSafe.is_null() {
                EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
                ret = 0;
                break 'exit;
            }
            if !crate::copy::capture_from_user(
                pjBitsSafe as *mut core::ffi::c_void,
                Bits as *const core::ffi::c_void,
                cjMaxBits as usize,
            ) {
                ret = 0;
                break 'exit;
            }
            Bits = pjBitsSafe as _;
        }

        if YDest < 0 {
            let h = (*bmi).bmiHeader.biHeight;
            let abs_h = if h < 0 { (-h) as u32 } else { h as u32 };
            let avail = abs_h.saturating_sub(StartScan);
            if ScanLines > avail { ScanLines = avail; }
        }

        if ScanLines == 0 {
            ret = 0;
            break 'exit;
        }

        pDC = DC_LockDc(hDC);
        if pDC.is_null() {
            EngSetLastError(ERROR_INVALID_HANDLE);
            ret = 0;
            break 'exit;
        }

        if (*pDC).dctype as u32 == DCTYPE_INFO {
            ret = 0;
            break 'exit;
        }

        rcDest.left = XDest;
        rcDest.top = YDest;
        if bTransformCoordinates != 0 {
            IntLPtoDP(pDC, core::ptr::addr_of_mut!(rcDest) as *mut POINTL, 2);
        }
        rcDest.left += (*pDC).ptlDCOrig.x;
        rcDest.top += (*pDC).ptlDCOrig.y;
        rcDest.right = rcDest.left + Width as i32;
        rcDest.bottom = rcDest.top + Height as i32;
        rcDest.top += StartScan as i32;

        if ((*pDC).fs as u32 & (DC_ACCUM_APP | DC_ACCUM_WMGR)) != 0 {
            IntUpdateBoundsRect(pDC, core::ptr::addr_of_mut!(rcDest));
        }

        ptSource.x = XSrc;
        ptSource.y = YSrc;

        SourceSize.cx = (*bmi).bmiHeader.biWidth;
        SourceSize.cy = ScanLines as i32;

        hSourceBitmap = GreCreateBitmapEx(
            (*bmi).bmiHeader.biWidth as u32,
            ScanLines,
            0,
            BitmapFormat((*bmi).bmiHeader.biBitCount as u32, (*bmi).bmiHeader.biCompression),
            if (*bmi).bmiHeader.biHeight < 0 { BMF_TOPDOWN as USHORT } else { 0 },
            (*bmi).bmiHeader.biSizeImage,
            Bits as *mut core::ffi::c_void,
            0,
        );

        if hSourceBitmap.is_null() {
            EngSetLastError(ERROR_NO_SYSTEM_RESOURCES);
            ret = 0;
            break 'exit;
        }

        pSourceSurf = EngLockSurface(hSourceBitmap as HSURF);
        if pSourceSurf.is_null() {
            ret = 0;
            break 'exit;
        }

        if (*bmi).bmiHeader.biCompression == BI_RLE8 || (*bmi).bmiHeader.biCompression == BI_RLE4 {
            hMaskBitmap = IntGdiCreateMaskFromRLE(
                (*bmi).bmiHeader.biWidth as u32,
                ScanLines,
                (*bmi).bmiHeader.biCompression,
                Bits,
                cjMaxBits,
            );
            if hMaskBitmap.is_null() {
                EngSetLastError(ERROR_NO_SYSTEM_RESOURCES);
                ret = 0;
                break 'exit;
            }
            pMaskSurf = EngLockSurface(hMaskBitmap as HSURF);
            if pMaskSurf.is_null() {
                ret = 0;
                break 'exit;
            }
        }

        ppalDIB = CreateDIBPalette(bmi, pDC, ColorUse);
        if ppalDIB.is_null() {
            EngSetLastError(ERROR_NO_SYSTEM_RESOURCES);
            ret = 0;
            break 'exit;
        }

        DC_vPrepareDCsForBlit(pDC, core::ptr::addr_of_mut!(rcDest), core::ptr::null_mut(), core::ptr::null_mut());
        pSurf = (*pDC).dclevel.pSurface;
        if pSurf.is_null() {
            DC_vFinishBlit(pDC, core::ptr::null_mut());
            ret = ScanLines as i32;
            break 'exit;
        }

        EXLATEOBJ_vInitialize(
            core::ptr::addr_of_mut!(exlo),
            ppalDIB,
            (*pSurf).ppal,
            RGB(0xff, 0xff, 0xff),
            (*(*pDC).pdcattr).crBackgroundClr,
            (*(*pDC).pdcattr).crForegroundClr,
        );

        pDestSurf = core::ptr::addr_of_mut!((*pSurf).SurfObj);

        if rcDest.top > rcDest.bottom {
            RECTL_vMakeWellOrdered(core::ptr::addr_of_mut!(rcDest));
            ptSource.y -= SourceSize.cy;
        }

        bResult = IntEngBitBlt(
            pDestSurf,
            pSourceSurf,
            pMaskSurf,
            core::ptr::addr_of_mut!((*pDC).co) as *mut CLIPOBJ,
            core::ptr::addr_of_mut!(exlo.xlo),
            core::ptr::addr_of_mut!(rcDest),
            core::ptr::addr_of_mut!(ptSource),
            if !pMaskSurf.is_null() { core::ptr::addr_of_mut!(ptSource) } else { core::ptr::null_mut() },
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            if !pMaskSurf.is_null() { ROP4_MASK } else { ROP4_FROM_INDEX(R3_OPINDEX_SRCCOPY) },
        );

        EXLATEOBJ_vCleanup(core::ptr::addr_of_mut!(exlo));

        DC_vFinishBlit(pDC, core::ptr::null_mut());

        ret = if bResult != 0 { ScanLines as i32 } else { 0 };
    }

    if !ppalDIB.is_null() { PALETTE_ShareUnlockPalette(ppalDIB); }
    if !pSourceSurf.is_null() { EngUnlockSurface(pSourceSurf); }
    if !hSourceBitmap.is_null() { EngDeleteSurface(hSourceBitmap as HSURF); }
    if !pMaskSurf.is_null() { EngUnlockSurface(pMaskSurf); }
    if !hMaskBitmap.is_null() { EngDeleteSurface(hMaskBitmap as HSURF); }
    if !pDC.is_null() { DC_UnlockDc(pDC); }
    ExFreePoolWithTag(pbmiSafe as *mut core::ffi::c_void, u32::from_le_bytes(*b"GTmp"));
    if !pjBitsSafe.is_null() {
        ExFreePoolWithTag(pjBitsSafe as *mut core::ffi::c_void, TAG_DIB);
    }

    ret
}
#[no_mangle]
pub unsafe extern "system" fn GreGetDIBitsInternal(
    hDC: HDC,
    hBitmap: HBITMAP,
    mut StartScan: UINT,
    mut ScanLines: UINT,
    Bits: LPBYTE,
    mut Info: LPBITMAPINFO,
    Usage: UINT,
    _MaxBits: UINT,
    _MaxInfo: UINT,
) -> INT {
    let mut pbmci: *mut BITMAPCOREINFO = core::ptr::null_mut();
    let mut psurf: PSURFACE = core::ptr::null_mut();
    let mut pDC: PDC;
    let mut width: LONG = 0;
    let mut height: LONG = 0;
    let mut planes: WORD = 0;
    let mut bpp: WORD = 0;
    let mut compr: DWORD = 0;
    let mut size: DWORD = 0;
    let mut i: USHORT;
    let bitmap_type: i32;
    let mut rgbQuads: *mut RGBQUAD;
    let colorPtr: *mut core::ffi::c_void;

    if (Usage != 0 && Usage != DIB_PAL_COLORS) || Info.is_null() || hBitmap.is_null() {
        return 0;
    }

    pDC = DC_LockDc(hDC);

    'done: loop {
        if pDC.is_null() || (*pDC).dctype as u32 == DCTYPE_INFO {
            ScanLines = 0;
            break 'done;
        }

        psurf = SURFACE_ShareLockSurface(hBitmap);
        if psurf.is_null() {
            ScanLines = 0;
            break 'done;
        }

        colorPtr = (Info as LPBYTE).add((*Info).bmiHeader.biSize as usize) as *mut core::ffi::c_void;
        rgbQuads = colorPtr as *mut RGBQUAD;

        bitmap_type = DIB_GetBitmapInfo(
            core::ptr::addr_of_mut!((*Info).bmiHeader),
            &mut width,
            &mut height,
            &mut planes,
            &mut bpp,
            &mut compr,
            &mut size,
        );

        if bitmap_type == -1 {
            EngSetLastError(ERROR_INVALID_PARAMETER);
            ScanLines = 0;
            break 'done;
        } else if bitmap_type == 0 {
            pbmci = Info as *mut BITMAPCOREINFO;
            (*pbmci).bmciHeader.bcBitCount = BitsPerFormat((*psurf).SurfObj.iBitmapFormat) as WORD;
            Info = DIB_ConvertBitmapInfo(pbmci as *const BITMAPINFO, Usage);
            if Info.is_null() {
                ScanLines = 0;
                break 'done;
            }
            rgbQuads = (*Info).bmiColors.as_mut_ptr();
        }

        if width < 0 || (bpp == 0 && !Bits.is_null()) {
            ScanLines = 0;
            break 'done;
        }

        if !Bits.is_null() || bpp != 0 {
            if (height == 0 || width == 0)
                || (compr != 0 && compr != BI_BITFIELDS && compr != BI_RGB)
            {
                ScanLines = 0;
                break 'done;
            }
        }

        (*Info).bmiHeader.biClrUsed = 0;
        (*Info).bmiHeader.biClrImportant = 0;

        match bpp {
            0 => {
                (*Info).bmiHeader.biWidth = (*psurf).SurfObj.sizlBitmap.cx;
                (*Info).bmiHeader.biHeight = if ((*psurf).SurfObj.fjBitmap as u32 & BMF_TOPDOWN) != 0 {
                    -(*psurf).SurfObj.sizlBitmap.cy
                } else {
                    (*psurf).SurfObj.sizlBitmap.cy
                };
                (*Info).bmiHeader.biPlanes = 1;
                (*Info).bmiHeader.biBitCount = BitsPerFormat((*psurf).SurfObj.iBitmapFormat) as WORD;
                (*Info).bmiHeader.biSizeImage = DIB_GetDIBImageBytes(
                    (*Info).bmiHeader.biWidth,
                    (*Info).bmiHeader.biHeight,
                    (*Info).bmiHeader.biBitCount as INT,
                ) as DWORD;
                (*Info).bmiHeader.biCompression =
                    if (*Info).bmiHeader.biBitCount == 16 || (*Info).bmiHeader.biBitCount == 32 {
                        BI_BITFIELDS
                    } else {
                        BI_RGB
                    };
                (*Info).bmiHeader.biXPelsPerMeter = 0;
                (*Info).bmiHeader.biYPelsPerMeter = 0;

                if (*Info).bmiHeader.biBitCount <= 8 && (*Info).bmiHeader.biClrUsed == 0 {
                    (*Info).bmiHeader.biClrUsed = 1u32 << (*Info).bmiHeader.biBitCount;
                }

                ScanLines = 1;
                break 'done;
            }
            1 | 4 | 8 => {
                (*Info).bmiHeader.biClrUsed = 1u32 << bpp;

                if !(*psurf).hSecure.is_null()
                    && BitsPerFormat((*psurf).SurfObj.iBitmapFormat) as WORD == bpp
                {
                    if Usage == DIB_RGB_COLORS {
                        let colors = core::cmp::min((*(*psurf).ppal).NumColors, 256);
                        if colors != 256 {
                            (*Info).bmiHeader.biClrUsed = colors;
                        }
                        i = 0;
                        while (i as u32) < colors {
                            (*rgbQuads.add(i as usize)).rgbRed =
                                (*(*(*psurf).ppal).IndexedColors.add(i as usize)).peRed;
                            (*rgbQuads.add(i as usize)).rgbGreen =
                                (*(*(*psurf).ppal).IndexedColors.add(i as usize)).peGreen;
                            (*rgbQuads.add(i as usize)).rgbBlue =
                                (*(*(*psurf).ppal).IndexedColors.add(i as usize)).peBlue;
                            (*rgbQuads.add(i as usize)).rgbReserved = 0;
                            i += 1;
                        }
                    } else {
                        let words = rgbQuads as *mut WORD;
                        i = 0;
                        while i < 256 {
                            *words.add(i as usize) = i;
                            i += 1;
                        }
                    }
                } else {
                    if Usage == DIB_PAL_COLORS {
                        let words = rgbQuads as *mut WORD;
                        i = 0;
                        while i < 256 {
                            *words.add(i as usize) = i;
                            i += 1;
                        }
                    } else if bpp > 1
                        && bpp == BitsPerFormat((*psurf).SurfObj.iBitmapFormat) as WORD
                    {
                        let pDcPal: PPALETTE =
                            PALETTE_ShareLockPalette((*pDC).dclevel.hpal);
                        if pDcPal.is_null() {
                            ScanLines = 0;
                            break 'done;
                        }
                        i = 0;
                        while (i as u32) < (*pDcPal).NumColors {
                            (*rgbQuads.add(i as usize)).rgbRed =
                                (*(*pDcPal).IndexedColors.add(i as usize)).peRed;
                            (*rgbQuads.add(i as usize)).rgbGreen =
                                (*(*pDcPal).IndexedColors.add(i as usize)).peGreen;
                            (*rgbQuads.add(i as usize)).rgbBlue =
                                (*(*pDcPal).IndexedColors.add(i as usize)).peBlue;
                            (*rgbQuads.add(i as usize)).rgbReserved = 0;
                            i += 1;
                        }
                        PALETTE_ShareUnlockPalette(pDcPal);
                    } else {
                        match bpp {
                            1 => {
                                (*rgbQuads.add(0)).rgbRed = 0;
                                (*rgbQuads.add(0)).rgbGreen = 0;
                                (*rgbQuads.add(0)).rgbBlue = 0;
                                (*rgbQuads.add(0)).rgbReserved = 0;
                                (*rgbQuads.add(1)).rgbRed = 0xff;
                                (*rgbQuads.add(1)).rgbGreen = 0xff;
                                (*rgbQuads.add(1)).rgbBlue = 0xff;
                                (*rgbQuads.add(1)).rgbReserved = 0;
                            }
                            4 => {
                                RtlCopyMemory(
                                    rgbQuads as *mut core::ffi::c_void,
                                    DefLogPaletteQuads.as_ptr() as *const core::ffi::c_void,
                                    7 * core::mem::size_of::<RGBQUAD>(),
                                );
                                RtlCopyMemory(
                                    rgbQuads.add(7) as *mut core::ffi::c_void,
                                    DefLogPaletteQuads.as_ptr().add(12) as *const core::ffi::c_void,
                                    core::mem::size_of::<RGBQUAD>(),
                                );
                                RtlCopyMemory(
                                    rgbQuads.add(8) as *mut core::ffi::c_void,
                                    DefLogPaletteQuads.as_ptr().add(7) as *const core::ffi::c_void,
                                    core::mem::size_of::<RGBQUAD>(),
                                );
                                RtlCopyMemory(
                                    rgbQuads.add(9) as *mut core::ffi::c_void,
                                    DefLogPaletteQuads.as_ptr().add(13) as *const core::ffi::c_void,
                                    7 * core::mem::size_of::<RGBQUAD>(),
                                );
                            }
                            8 => {
                                core::ptr::copy_nonoverlapping(
                                    DefLogPaletteQuads.as_ptr(),
                                    rgbQuads,
                                    10,
                                );
                                core::ptr::copy_nonoverlapping(
                                    DefLogPaletteQuads.as_ptr().add(10),
                                    rgbQuads.add(246),
                                    10,
                                );
                                let mut j: INT = 10;
                                while j < 246 {
                                    (*rgbQuads.add(j as usize)).rgbRed = ((j & 0x07) << 5) as u8;
                                    (*rgbQuads.add(j as usize)).rgbGreen = ((j & 0x38) << 2) as u8;
                                    (*rgbQuads.add(j as usize)).rgbBlue = (j & 0xc0) as u8;
                                    (*rgbQuads.add(j as usize)).rgbReserved = 0;
                                    j += 1;
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            15 => {
                if (*Info).bmiHeader.biCompression == BI_BITFIELDS {
                    let p = (*Info).bmiColors.as_mut_ptr() as *mut DWORD;
                    *p.add(0) = 0x7c00;
                    *p.add(1) = 0x03e0;
                    *p.add(2) = 0x001f;
                }
            }
            16 => {
                if (*Info).bmiHeader.biCompression == BI_BITFIELDS {
                    let p = (*Info).bmiColors.as_mut_ptr() as *mut DWORD;
                    if !(*psurf).hSecure.is_null() {
                        *p.add(0) = (*(*psurf).ppal).RedMask;
                        *p.add(1) = (*(*psurf).ppal).GreenMask;
                        *p.add(2) = (*(*psurf).ppal).BlueMask;
                    } else {
                        *p.add(0) = 0xf800;
                        *p.add(1) = 0x07e0;
                        *p.add(2) = 0x001f;
                    }
                }
            }
            24 | 32 => {
                if (*Info).bmiHeader.biCompression == BI_BITFIELDS {
                    let p = (*Info).bmiColors.as_mut_ptr() as *mut DWORD;
                    if !(*psurf).hSecure.is_null() {
                        *p.add(0) = (*(*psurf).ppal).RedMask;
                        *p.add(1) = (*(*psurf).ppal).GreenMask;
                        *p.add(2) = (*(*psurf).ppal).BlueMask;
                    } else {
                        *p.add(0) = 0xff0000;
                        *p.add(1) = 0x00ff00;
                        *p.add(2) = 0x0000ff;
                    }
                }
            }
            _ => {
                ScanLines = 0;
                break 'done;
            }
        }

        (*Info).bmiHeader.biSizeImage =
            DIB_GetDIBImageBytes(width, height, bpp as INT) as DWORD;
        (*Info).bmiHeader.biPlanes = 1;

        if !Bits.is_null() && ScanLines != 0 {
            let mut pDIBits: *mut core::ffi::c_void = core::ptr::null_mut();
            let hBmpDest: HBITMAP;
            let psurfDest: PSURFACE;
            let mut exlo: EXLATEOBJ = core::mem::zeroed();
            let mut rcDest: RECT = core::mem::zeroed();
            let mut srcPoint: POINTL = core::mem::zeroed();
            let ret: i32;
            let mut newLines: i32 = -1;

            let abs_h = if (*Info).bmiHeader.biHeight < 0 {
                -(*Info).bmiHeader.biHeight
            } else {
                (*Info).bmiHeader.biHeight
            } as u32;

            if StartScan >= abs_h {
                ScanLines = 1;
                break 'done;
            } else {
                ScanLines = core::cmp::min(ScanLines, abs_h - StartScan);
            }

            if abs_h < (*psurf).SurfObj.sizlBitmap.cy as u32 {
                StartScan += (*psurf).SurfObj.sizlBitmap.cy as u32 - abs_h;
            }

            (*Info).bmiHeader.biHeight = if height < 0 {
                -(ScanLines as LONG)
            } else {
                ScanLines as LONG
            };

            hBmpDest = DIB_CreateDIBSection(
                pDC,
                Info,
                Usage,
                &mut pDIBits,
                core::ptr::null_mut(),
                0,
                0,
            );

            (*Info).bmiHeader.biHeight = height;

            if hBmpDest.is_null() {
                EngSetLastError(ERROR_INVALID_PARAMETER);
                ScanLines = 0;
                break 'done;
            }

            psurfDest = SURFACE_ShareLockSurface(hBmpDest);

            RECTL_vSetRect(
                core::ptr::addr_of_mut!(rcDest) as *mut RECTL,
                0,
                0,
                (*Info).bmiHeader.biWidth,
                ScanLines as LONG,
            );
            (*Info).bmiHeader.biWidth = width;
            srcPoint.x = 0;

            let abs_h2 = if (*Info).bmiHeader.biHeight < 0 {
                -(*Info).bmiHeader.biHeight
            } else {
                (*Info).bmiHeader.biHeight
            };

            if abs_h2 <= (*psurf).SurfObj.sizlBitmap.cy {
                srcPoint.y =
                    (*psurf).SurfObj.sizlBitmap.cy - StartScan as LONG - ScanLines as LONG;
            } else {
                newLines = abs_h2 - (*psurf).SurfObj.sizlBitmap.cy;
                newLines = core::cmp::min(
                    (StartScan + ScanLines) as i32 - newLines,
                    (*psurf).SurfObj.sizlBitmap.cy,
                );
                if newLines > 0 {
                    srcPoint.y = (*psurf).SurfObj.sizlBitmap.cy - newLines;
                    if StartScan as LONG > (*psurf).SurfObj.sizlBitmap.cy {
                        newLines -= StartScan as i32 - (*psurf).SurfObj.sizlBitmap.cy;
                    }
                } else {
                    newLines = 0;
                    srcPoint.y = (*psurf).SurfObj.sizlBitmap.cy;
                }
            }

            EXLATEOBJ_vInitialize(
                &mut exlo,
                (*psurf).ppal,
                (*psurfDest).ppal,
                0xffffff,
                0xffffff,
                0,
            );

            ret = IntEngCopyBits(
                &mut (*psurfDest).SurfObj,
                &mut (*psurf).SurfObj,
                core::ptr::null_mut(),
                &mut exlo.xlo,
                core::ptr::addr_of_mut!(rcDest) as *mut RECTL,
                &mut srcPoint,
            );

            SURFACE_ShareUnlockSurface(psurfDest);

            if ret == 0 {
                ScanLines = 0;
            } else {
                RtlCopyMemory(
                    Bits as *mut core::ffi::c_void,
                    pDIBits,
                    DIB_GetDIBImageBytes(width, ScanLines as INT, bpp as INT) as usize,
                );
            }

            if newLines != -1 {
                ScanLines = newLines as UINT;
            }
            GreDeleteObject(hBmpDest as *mut core::ffi::c_void);
            EXLATEOBJ_vCleanup(&mut exlo);
        } else {
            ScanLines = 1;
        }

        break 'done;
    }

    if !pbmci.is_null() {
        DIB_FreeConvertedBitmapInfo(Info, pbmci as *mut BITMAPINFO, Usage);
    }

    if !psurf.is_null() {
        SURFACE_ShareUnlockSurface(psurf);
    }

    if !pDC.is_null() {
        DC_UnlockDc(pDC);
    }

    ScanLines as INT
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiGetDIBitsInternal(
    hdc: HDC,
    hbm: HBITMAP,
    iStartScan: UINT,
    cScans: UINT,
    pjBits: LPBYTE,
    pbmi: LPBITMAPINFO,
    iUsage: UINT,
    cjMaxBits: UINT,
    mut cjMaxInfo: UINT,
) -> INT {
    let pbmiSafe: PBITMAPINFO;
    let mut hSecure: HANDLE = core::ptr::null_mut();
    let mut iResult: INT = 0;
    let cjAlloc: UINT;

    if iUsage > 2 {
        return 0;
    }

    if (cjMaxInfo as usize) < core::mem::size_of::<BITMAPCOREHEADER>() {
        return 0;
    }

    let maxSize = (core::mem::size_of::<BITMAPV5HEADER>() + 256 * core::mem::size_of::<RGBQUAD>()) as UINT;
    if cjMaxInfo > maxSize {
        cjMaxInfo = maxSize;
    }

    cjAlloc = maxSize;

    pbmiSafe = ExAllocatePoolWithTag(PagedPool, cjAlloc as _, u32::from_le_bytes(*b"GBmi")) as PBITMAPINFO;
    if pbmiSafe.is_null() {
        return 0;
    }

    'cleanup: {
        {
            let __SehStatus = SafeProbeForRead(pbmi as *mut _, cjMaxInfo as usize, 1);
            if !NT_SUCCESS(__SehStatus) {
                break 'cleanup;
            }
        }

        // The probe VALIDATES only; this capture is the access, and takes the same exit.
        if !crate::copy::capture_from_user(
            pbmiSafe as *mut core::ffi::c_void,
            pbmi as *const core::ffi::c_void,
            cjMaxInfo as usize,
        ) {
            break 'cleanup;
        }

        if ((*pbmiSafe).bmiHeader.biSize < core::mem::size_of::<BITMAPCOREHEADER>() as u32)
            || ((*pbmiSafe).bmiHeader.biSize > cjMaxInfo)
        {
            break 'cleanup;
        }

        if !pjBits.is_null() {
            hSecure = EngSecureMem(pjBits as *mut _, cjMaxBits);
            if hSecure.is_null() {
                break 'cleanup;
            }
        }

        iResult = GreGetDIBitsInternal(
            hdc,
            hbm,
            iStartScan,
            cScans,
            pjBits,
            pbmiSafe,
            iUsage,
            cjMaxBits,
            cjMaxInfo,
        );

        if iResult != 0 {
            let infoSize = DIB_BitmapInfoSize(pbmiSafe, iUsage as u16) as UINT;
            if cjMaxInfo > infoSize {
                cjMaxInfo = infoSize;
            }
            // C: _SEH2_TRY { ProbeForWrite(pbmi, cjMaxInfo, 1);
            //                RtlCopyMemory(pbmi, pbmiSafe, cjMaxInfo); }
            //      _SEH2_EXCEPT { /* Ignore */ (VOID)0; }
            // The handler deliberately ignores the fault: the scan lines already
            // went to pjBits (secured user memory), only the header copy-back
            // failed, so iResult stands and no last-error is set.
            let _ = crate::safe_user_io::copy_to_user(
                pbmi as *mut core::ffi::c_void,
                pbmiSafe as *const core::ffi::c_void,
                cjMaxInfo as usize,
                1,
            );
        }
    }

    if !hSecure.is_null() {
        EngUnsecureMem(hSecure);
    }
    ExFreePoolWithTag(pbmiSafe as *mut _, u32::from_le_bytes(*b"GBmi"));

    iResult
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiStretchDIBitsInternal(
    hdc: HDC,
    xDst: i32,
    yDst: i32,
    cxDst: i32,
    cyDst: i32,
    xSrc: i32,
    ySrc: i32,
    cxSrc: i32,
    cySrc: i32,
    pjInit: LPBYTE,
    pbmi: LPBITMAPINFO,
    dwUsage: u32,
    dwRop: u32,
    cjMaxInfo: u32,
    cjMaxBits: u32,
    _hcmXform: *mut core::ffi::c_void,
) -> i32 {
    let mut sizel: SIZEL = core::mem::zeroed();
    let mut rcSrc: RECTL = core::mem::zeroed();
    let mut rcDst: RECTL = core::mem::zeroed();
    let mut pdc: PDC;
    let mut hbmTmp: HBITMAP = core::ptr::null_mut();
    let mut psurfTmp: PSURFACE = core::ptr::null_mut();
    let mut psurfDst: PSURFACE;
    let mut ppalDIB: PPALETTE = core::ptr::null_mut();
    let mut exlo: EXLATEOBJ = core::mem::zeroed();
    let mut pvBits: PBYTE;

    let pbmiSafe: LPBITMAPINFO;
    let cjAlloc: u32;
    let hBitmap: HBITMAP;
    let mut hOldBitmap: HBITMAP = core::ptr::null_mut();
    let hdcMem: HDC;
    let mut hPal: HPALETTE = core::ptr::null_mut();
    let mut BmpFormat: u32 = 0;
    let LinesCopied: i32;

    let mut cjMaxInfo = cjMaxInfo;

    if dwUsage > 2 {
        return 0;
    }

    if pbmi.is_null() {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    if (cjMaxInfo as usize) < core::mem::size_of::<BITMAPCOREHEADER>() {
        return 0;
    }

    cjMaxInfo = min_u32(
        cjMaxInfo,
        (core::mem::size_of::<BITMAPV5HEADER>() + 256 * core::mem::size_of::<RGBQUAD>()) as u32,
    );

    cjAlloc = (core::mem::size_of::<BITMAPV5HEADER>() + 256 * core::mem::size_of::<RGBQUAD>()) as u32;

    pbmiSafe = ExAllocatePoolWithTag(PagedPool, cjAlloc as _, TAG_GBMI) as LPBITMAPINFO;
    if pbmiSafe.is_null() {
        return 0;
    }

    {
        let __SehStatus = SafeProbeForRead(pbmi as *const core::ffi::c_void, cjMaxInfo as usize, 1);
        if !NT_SUCCESS(__SehStatus) {
            ExFreePoolWithTag(pbmiSafe as *mut core::ffi::c_void, TAG_GBMI);
            return 0;
        }
    }

    // The probe VALIDATES only; this capture is the access, and takes the same exit.
    if !crate::copy::capture_from_user(
        pbmiSafe as *mut core::ffi::c_void,
        pbmi as *const core::ffi::c_void,
        cjMaxInfo as usize,
    ) {
        ExFreePoolWithTag(pbmiSafe as *mut core::ffi::c_void, TAG_GBMI);
        return 0;
    }

    if ((*pbmiSafe).bmiHeader.biSize < core::mem::size_of::<BITMAPCOREHEADER>() as u32)
        || ((*pbmiSafe).bmiHeader.biSize > cjMaxInfo)
    {
        ExFreePoolWithTag(pbmiSafe as *mut core::ffi::c_void, TAG_GBMI);
        return 0;
    }

    pdc = DC_LockDc(hdc);
    if pdc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }

    if (*pdc).dclevel.pSurface.is_null() {
        DC_UnlockDc(pdc);
        return 1;
    }

    sizel.cx = cxDst;
    sizel.cy = cyDst;
    IntLPtoDP(pdc, core::ptr::addr_of_mut!(sizel) as *mut POINTL, 1);
    DC_UnlockDc(pdc);

    if !pjInit.is_null() && cjMaxBits > 0 {
        pvBits = ExAllocatePoolWithTag(PagedPool, cjMaxBits as _, TAG_DIB) as PBYTE;
        if pvBits.is_null() {
            return 0;
        }

        {
            let __SehStatus = SafeProbeForRead(pjInit as *const core::ffi::c_void, cjMaxBits as usize, 1);
            if !NT_SUCCESS(__SehStatus) {
                ExFreePoolWithTag(pvBits as *mut core::ffi::c_void, TAG_DIB);
                return 0;
            }
        }
        // The probe VALIDATES only; this capture is the access, and takes the same exit.
        if !crate::copy::capture_from_user(
            pvBits as *mut core::ffi::c_void,
            pjInit as *const core::ffi::c_void,
            cjMaxBits as usize,
        ) {
            ExFreePoolWithTag(pvBits as *mut core::ffi::c_void, TAG_DIB);
            return 0;
        }
    } else {
        pvBits = core::ptr::null_mut();
    }

    if dwRop == SRCCOPY {
        hdcMem = NtGdiCreateCompatibleDC(hdc);
        if hdcMem.is_null() {
            EngSetLastError(ERROR_NO_SYSTEM_RESOURCES);
            return 0;
        }

        hBitmap = NtGdiCreateCompatibleBitmap(
            hdc,
            abs_i32((*pbmiSafe).bmiHeader.biWidth),
            abs_i32((*pbmiSafe).bmiHeader.biHeight),
        );
        if hBitmap.is_null() {
            EngSetLastError(ERROR_NO_SYSTEM_RESOURCES);
            return 0;
        }

        hOldBitmap = NtGdiSelectBitmap(hdcMem, hBitmap);

        if dwUsage == DIB_PAL_COLORS {
            hPal = NtGdiGetDCObject(hdc, GDI_OBJECT_TYPE_PALETTE as i32) as HPALETTE;
            hPal = GdiSelectPalette(hdcMem, hPal, 0);
        }

        pdc = DC_LockDc(hdcMem);
        if !pdc.is_null() {
            IntSetDIBits(
                pdc,
                hBitmap,
                0,
                abs_i32((*pbmiSafe).bmiHeader.biHeight) as u32,
                pvBits as *const core::ffi::c_void,
                cjMaxBits,
                pbmiSafe,
                dwUsage,
            );
            DC_UnlockDc(pdc);
        }

        if cxSrc == cxDst && cySrc == cyDst {
            NtGdiBitBlt(
                hdc, xDst, yDst, cxDst, cyDst,
                hdcMem, xSrc,
                abs_i32((*pbmiSafe).bmiHeader.biHeight) - cySrc - ySrc,
                dwRop, CLR_INVALID, 0,
            );
        } else {
            NtGdiStretchBlt(
                hdc, xDst, yDst, cxDst, cyDst,
                hdcMem, xSrc,
                abs_i32((*pbmiSafe).bmiHeader.biHeight) - cySrc - ySrc,
                cxSrc, cySrc, dwRop, CLR_INVALID,
            );
        }

        if !hPal.is_null() {
            GdiSelectPalette(hdcMem, hPal, 0);
        }

        if !hOldBitmap.is_null() {
            NtGdiSelectBitmap(hdcMem, hOldBitmap);
        }

        NtGdiDeleteObjectApp(hdcMem as *mut core::ffi::c_void);
        GreDeleteObject(hBitmap as *mut core::ffi::c_void);
    } else {
        'cleanup: {
            pdc = DC_LockDc(hdc);
            if pdc.is_null() {
                EngSetLastError(ERROR_INVALID_HANDLE);
                break 'cleanup;
            }

            rcSrc.left = xSrc;
            rcSrc.top = ySrc;
            rcSrc.right = xSrc + abs_i32(cxSrc);
            rcSrc.bottom = ySrc + abs_i32(cySrc);
            rcDst.left = xDst;
            rcDst.top = yDst;
            rcDst.right = rcDst.left + cxDst;
            rcDst.bottom = rcDst.top + cyDst;
            IntLPtoDP(pdc, core::ptr::addr_of_mut!(rcDst) as *mut POINTL, 2);
            RECTL_vOffsetRect(
                core::ptr::addr_of_mut!(rcDst),
                (*pdc).ptlDCOrig.x,
                (*pdc).ptlDCOrig.y,
            );

            if ((*pdc).fs as u32 & (DC_ACCUM_APP | DC_ACCUM_WMGR)) != 0 {
                IntUpdateBoundsRect(pdc, core::ptr::addr_of_mut!(rcDst));
            }

            BmpFormat = BitmapFormat(
                (*pbmiSafe).bmiHeader.biBitCount as u32,
                (*pbmiSafe).bmiHeader.biCompression,
            );

            hbmTmp = GreCreateBitmapEx(
                (*pbmiSafe).bmiHeader.biWidth as u32,
                abs_i32((*pbmiSafe).bmiHeader.biHeight) as u32,
                0,
                BmpFormat,
                if (*pbmiSafe).bmiHeader.biHeight < 0 { BMF_TOPDOWN as USHORT } else { 0 },
                cjMaxBits,
                pvBits as *mut core::ffi::c_void,
                0,
            );

            if hbmTmp.is_null() {
                break 'cleanup;
            }

            psurfTmp = SURFACE_ShareLockSurface(hbmTmp);
            if psurfTmp.is_null() {
                break 'cleanup;
            }

            ppalDIB = CreateDIBPalette(pbmiSafe, pdc, dwUsage);
            if ppalDIB.is_null() {
                break 'cleanup;
            }

            DC_vPrepareDCsForBlit(pdc, core::ptr::addr_of_mut!(rcDst), core::ptr::null_mut(), core::ptr::null_mut());

            psurfDst = (*pdc).dclevel.pSurface;

            EXLATEOBJ_vInitialize(
                core::ptr::addr_of_mut!(exlo),
                ppalDIB,
                (*psurfDst).ppal,
                RGB(0xff, 0xff, 0xff),
                (*(*pdc).pdcattr).crBackgroundClr,
                (*(*pdc).pdcattr).crForegroundClr,
            );

            IntEngStretchBlt(
                core::ptr::addr_of_mut!((*psurfDst).SurfObj),
                core::ptr::addr_of_mut!((*psurfTmp).SurfObj),
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!((*pdc).co) as *mut CLIPOBJ,
                core::ptr::addr_of_mut!(exlo.xlo),
                core::ptr::addr_of_mut!((*pdc).dclevel.ca),
                core::ptr::addr_of_mut!(rcDst),
                core::ptr::addr_of_mut!(rcSrc),
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!((*pdc).eboFill.BrushObject),
                core::ptr::null_mut(),
                WIN32_ROP3_TO_ENG_ROP4(dwRop),
            );

            DC_vFinishBlit(pdc, core::ptr::null_mut());
            EXLATEOBJ_vCleanup(core::ptr::addr_of_mut!(exlo));
        }

        if !ppalDIB.is_null() { PALETTE_ShareUnlockPalette(ppalDIB); }
        if !psurfTmp.is_null() { SURFACE_ShareUnlockSurface(psurfTmp); }
        if !hbmTmp.is_null() { GreDeleteObject(hbmTmp as *mut core::ffi::c_void); }
        if !pdc.is_null() { DC_UnlockDc(pdc); }
    }

    if !pvBits.is_null() {
        ExFreePoolWithTag(pvBits as *mut core::ffi::c_void, TAG_DIB);
    }

    if dwRop == SRCCOPY {
        LinesCopied = abs_i32((*pbmiSafe).bmiHeader.biHeight);
    } else {
        LinesCopied = (*pbmiSafe).bmiHeader.biHeight;
    }

    ExFreePoolWithTag(pbmiSafe as *mut core::ffi::c_void, TAG_GBMI);

    LinesCopied
}

#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe extern "fastcall" fn IntCreateDIBitmap(
    Dc: PDC,
    width: i32,
    height: i32,
    planes: u32,
    bpp: u32,
    compression: u32,
    init: u32,
    bits: LPBYTE,
    cjMaxBits: u32,
    data: PBITMAPINFO,
    coloruse: u32,
) -> HBITMAP {
    IntCreateDIBitmap_impl(Dc, width, height, planes, bpp, compression, init, bits, cjMaxBits, data, coloruse)
}

#[no_mangle]
#[cfg(not(target_arch = "x86"))]
pub unsafe extern "C" fn IntCreateDIBitmap(
    Dc: PDC,
    width: i32,
    height: i32,
    planes: u32,
    bpp: u32,
    compression: u32,
    init: u32,
    bits: LPBYTE,
    cjMaxBits: u32,
    data: PBITMAPINFO,
    coloruse: u32,
) -> HBITMAP {
    IntCreateDIBitmap_impl(Dc, width, height, planes, bpp, compression, init, bits, cjMaxBits, data, coloruse)
}

unsafe fn IntCreateDIBitmap_impl(
    Dc: PDC,
    width: i32,
    mut height: i32,
    planes: u32,
    bpp: u32,
    compression: u32,
    init: u32,
    bits: LPBYTE,
    cjMaxBits: u32,
    data: PBITMAPINFO,
    coloruse: u32,
) -> HBITMAP {
    let handle: HBITMAP;
    let fColor: i32;
    let mut BmpFormat: u32 = 0;

    if planes != 0 && bpp != 0 {
        BmpFormat = BitmapFormat(planes * bpp, compression);
    }

    if BmpFormat != BMF_1BPP {
        fColor = 1;
    } else if coloruse > DIB_RGB_COLORS || (init & CBM_INIT) == 0 || data.is_null() {
        fColor = 0;
    } else {
        let mut rgb = (data as PBYTE).add((*data).bmiHeader.biSize as usize) as *const RGBQUAD;
        let mut col = RGB((*rgb).rgbRed, (*rgb).rgbGreen, (*rgb).rgbBlue);

        if col == RGB(0, 0, 0) {
            rgb = rgb.add(1);
            col = RGB((*rgb).rgbRed, (*rgb).rgbGreen, (*rgb).rgbBlue);
            fColor = if col != RGB(0xff, 0xff, 0xff) { 1 } else { 0 };
        } else {
            fColor = 1;
        }
    }

    if fColor != 0 {
        if (init & CBM_CREATDIB) != 0 {
            let Surface: PSURFACE;
            let Palette: PPALETTE;

            handle = IntCreateCompatibleBitmap(Dc, width, height, planes, bpp);
            if handle.is_null() {
                return core::ptr::null_mut();
            }

            Surface = SURFACE_ShareLockSurface(handle);
            Palette = CreateDIBPalette(data, Dc, coloruse);
            if Palette.is_null() {
                SURFACE_ShareUnlockSurface(Surface);
                GreDeleteObject(handle as _);
                return core::ptr::null_mut();
            }

            SURFACE_vSetPalette(Surface, Palette);

            PALETTE_ShareUnlockPalette(Palette);
            SURFACE_ShareUnlockSurface(Surface);
        } else {
            handle = IntCreateCompatibleBitmap(Dc, width, height, 0, 0);
        }
    } else {
        handle = GreCreateBitmap(width as u32, abs(height) as u32, 1, 1, core::ptr::null_mut());
    }

    if height < 0 {
        height = -height;
    }

    if !handle.is_null() && (CBM_INIT & init) != 0 {
        IntSetDIBits(Dc, handle, 0, height as u32, bits as *const core::ffi::c_void, cjMaxBits, data, coloruse);
    }

    handle
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiCreateDIBitmapInternal(
    hDc: HDC,
    cx: i32,
    cy: i32,
    fInit: u32,
    pjInit: LPBYTE,
    pbmi: LPBITMAPINFO,
    iUsage: u32,
    cjMaxInitInfo: u32,
    cjMaxBits: u32,
    fl: FLONG,
    hcmXform: *mut core::ffi::c_void,
) -> HBITMAP {
    let mut Status: i32 = STATUS_SUCCESS;
    let mut safeBits: PBYTE = core::ptr::null_mut();
    let mut pbmiSafe: LPBITMAPINFO = core::ptr::null_mut();
    let mut hbmResult: HBITMAP = core::ptr::null_mut();
    let mut fInit = fInit;

    if pjInit.is_null() {
        fInit &= !CBM_INIT;
    }

    if !pjInit.is_null() && (fInit & CBM_INIT) != 0 {
        if cjMaxBits == 0 {
            return core::ptr::null_mut();
        }
        safeBits = ExAllocatePoolWithTag(PagedPool, cjMaxBits as _, TAG_DIB) as PBYTE;
        if safeBits.is_null() {
            EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
            return core::ptr::null_mut();
        }
    }

    'cleanup: {
        if !pbmi.is_null() {
            Status = SafeProbeForRead(pbmi as *mut core::ffi::c_void, cjMaxInitInfo as usize, 1);
        }
        if NT_SUCCESS(Status) && !pjInit.is_null() && (fInit & CBM_INIT) != 0 {
            Status = SafeProbeForRead(pjInit as *mut core::ffi::c_void, cjMaxBits as usize, 1);
        }
        if !NT_SUCCESS(Status) {
            SetLastNtError(Status);
            break 'cleanup;
        }

        // The probes above VALIDATE only; this capture is the access.
        if !pjInit.is_null()
            && (fInit & CBM_INIT) != 0
            && !crate::copy::capture_from_user(
                safeBits as *mut core::ffi::c_void,
                pjInit as *const core::ffi::c_void,
                cjMaxBits as usize,
            )
        {
            SetLastNtError(STATUS_ACCESS_VIOLATION);
            break 'cleanup;
        }

        // `pbmi` must be CAPTURED, not forwarded. GreCreateDIBitmapInternal reads the
        // header and the colour table out of it, and it was being handed the caller's
        // raw pointer -- OUTSIDE the probe, so a fault there has no handler and takes
        // the machine down from user mode. Upstream has the same hole: it probes pbmi
        // inside the _SEH2_TRY and then passes the user pointer to the same callee
        // after the _SEH2_END, so this is not a translation divergence but a bug we
        // inherited and are not obliged to keep.
        if !pbmi.is_null() {
            // A bitmap-info pointer with a zero declared maximum has nothing that can
            // be validated or copied, so there is no safe way to honour it.
            if cjMaxInitInfo == 0 {
                EngSetLastError(ERROR_INVALID_PARAMETER);
                break 'cleanup;
            }
            pbmiSafe =
                ExAllocatePoolWithTag(PagedPool, cjMaxInitInfo as _, TAG_DIB) as LPBITMAPINFO;
            if pbmiSafe.is_null() {
                EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
                break 'cleanup;
            }
            if !crate::copy::capture_from_user(
                pbmiSafe as *mut core::ffi::c_void,
                pbmi as *const core::ffi::c_void,
                cjMaxInitInfo as usize,
            ) {
                SetLastNtError(STATUS_ACCESS_VIOLATION);
                break 'cleanup;
            }
        }

        hbmResult = GreCreateDIBitmapInternal(
            hDc,
            cx,
            cy,
            fInit,
            safeBits,
            pbmiSafe,
            iUsage,
            fl,
            cjMaxBits,
            hcmXform,
        );
    }

    if !safeBits.is_null() {
        ExFreePoolWithTag(safeBits as *mut core::ffi::c_void, TAG_DIB);
    }
    if !pbmiSafe.is_null() {
        ExFreePoolWithTag(pbmiSafe as *mut core::ffi::c_void, TAG_DIB);
    }
    hbmResult
}
#[no_mangle]
pub unsafe extern "system" fn GreCreateDIBitmapInternal(
    hDc: HDC,
    cx: i32,
    cy: i32,
    fInit: u32,
    pjInit: LPBYTE,
    pbmi: PBITMAPINFO,
    iUsage: u32,
    fl: FLONG,
    cjMaxBits: u32,
    hcmXform: *mut core::ffi::c_void,
) -> HBITMAP {
    let Dc: PDC;
    let Bmp: HBITMAP;
    let bpp: u16;
    let planes: u16;
    let compression: u32;
    let hdcDest: HDC;

    if hDc.is_null() {
        hdcDest = NtGdiCreateCompatibleDC(core::ptr::null_mut());
        if hdcDest.is_null() {
            return core::ptr::null_mut();
        }
    } else {
        hdcDest = hDc;
    }

    Dc = DC_LockDc(hdcDest);
    if Dc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return core::ptr::null_mut();
    }

    if !pbmi.is_null() {
        if (*pbmi).bmiHeader.biSize as usize == core::mem::size_of::<BITMAPCOREHEADER>() {
            let CoreHeader = core::ptr::addr_of!((*pbmi).bmiHeader) as *const BITMAPCOREHEADER;
            bpp = (*CoreHeader).bcBitCount;
            planes = if (*CoreHeader).bcPlanes != 0 { (*CoreHeader).bcPlanes } else { 1 };
            compression = BI_RGB;
        } else {
            bpp = (*pbmi).bmiHeader.biBitCount;
            planes = if (*pbmi).bmiHeader.biPlanes != 0 { (*pbmi).bmiHeader.biPlanes } else { 1 };
            compression = (*pbmi).bmiHeader.biCompression;
        }
    } else {
        bpp = 0;
        planes = 0;
        compression = 0;
    }

    Bmp = IntCreateDIBitmap(Dc, cx, cy, planes as u32, bpp as u32, compression, fInit, pjInit, cjMaxBits, pbmi, iUsage);
    DC_UnlockDc(Dc);

    if hDc.is_null() {
        NtGdiDeleteObjectApp(hdcDest as *mut core::ffi::c_void);
    }
    Bmp
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiCreateDIBSection(
    mut hDC: HDC,
    hSection: HANDLE,
    dwOffset: u32,
    bmi: *mut BITMAPINFO,
    Usage: u32,
    cjHeader: u32,
    fl: FLONG,
    dwColorSpace: usize,
    Bits: *mut PVOID,
) -> HBITMAP {
    let mut hbitmap: HBITMAP = core::ptr::null_mut();
    let dc: *mut DC;
    let mut bDesktopDC: u8 = 0;
    let mut Status: i32 = STATUS_SUCCESS;

    if bmi.is_null() {
        return hbitmap;
    }

    // What stood here was the comment "ProbeForRead calls are no-ops in this
    // translation context". They are not no-ops — they are the only bounds check on
    // `bmi`, which is a user pointer. Dropping them left `Status` a constant
    // STATUS_SUCCESS, so the `if (!NT_SUCCESS(Status))` below could never fire, and
    // DIB_BitmapInfoSize plus DIB_CreateDIBSection read the caller's pointer freely.
    //
    // DUMMY_PSEH is the reason the handler is omitted, and also the reason the probe
    // must stay: with no handler, an unvalidated access is not an exception, it is the
    // machine. Validate-first, in the C's three progressive steps — the size field
    // first, then the header it declares, then the full palette-dependent extent —
    // because each length is read out of the buffer being validated.
    Status = SafeProbeForRead(
        core::ptr::addr_of!((*bmi).bmiHeader.biSize) as *const core::ffi::c_void,
        core::mem::size_of::<DWORD>(),
        1,
    );
    // CAPTURE biSize rather than loading it inline. The probe above validated
    // the RANGE of this field and not its MAPPING -- it cannot touch-test under
    // DUMMY_PSEH -- so `(*bmi).bmiHeader.biSize` was a raw load of a user
    // pointer with no handler behind it. That exact instruction is
    // win32k.sys+0xFAD1A (NtGdiCreateDIBSection+0x4A, `movl (%r9), %r10d`),
    // where `win32u_apitest NtGdiCreateDIBSection` BUGCHECKs 0x1E.
    // Reading it once into a local is also what stops the size being re-read
    // from user memory after it has been validated.
    let mut biSize: DWORD = 0;
    if NT_SUCCESS(Status)
        && !crate::copy::capture_from_user(
            core::ptr::addr_of_mut!(biSize) as *mut core::ffi::c_void,
            core::ptr::addr_of!((*bmi).bmiHeader.biSize) as *const core::ffi::c_void,
            core::mem::size_of::<DWORD>(),
        )
    {
        Status = STATUS_ACCESS_VIOLATION;
    }
    if NT_SUCCESS(Status) {
        Status = SafeProbeForRead(bmi as *const core::ffi::c_void, biSize as usize, 1);
    }

    // CAPTURE the header, then compute its full extent from the COPY. The third probe
    // used `DIB_BitmapInfoSize(bmi, ..)`, which walks the caller's buffer to work out
    // how long it is -- so the length used to validate user memory was itself read out
    // of that same unvalidated user memory, and the walk was a raw read with no handler
    // behind it. Deriving the extent from a kernel copy removes both: the walk cannot
    // fault, and the size cannot change between the call that computes it and the probe
    // that trusts it. `biSize` bounds this first capture and was validated above.
    let mut pbmiFull: LPBITMAPINFO = core::ptr::null_mut();
    if NT_SUCCESS(Status) {
        if (biSize as usize) < core::mem::size_of::<BITMAPCOREHEADER>() {
            Status = STATUS_INVALID_PARAMETER;
        } else {
            let hdr = ExAllocatePoolWithTag(PagedPool, biSize as _, TAG_DIB) as LPBITMAPINFO;
            if hdr.is_null() {
                Status = STATUS_INSUFFICIENT_RESOURCES;
            } else {
                if !crate::copy::capture_from_user(
                    hdr as *mut core::ffi::c_void,
                    bmi as *const core::ffi::c_void,
                    biSize as usize,
                ) {
                    Status = STATUS_ACCESS_VIOLATION;
                }
                // The palette-dependent extent, computed from the kernel copy.
                let full = if NT_SUCCESS(Status) {
                    DIB_BitmapInfoSize(hdr, Usage as u16)
                } else {
                    0
                };
                if NT_SUCCESS(Status) && full <= 0 {
                    Status = STATUS_INVALID_PARAMETER;
                }
                if NT_SUCCESS(Status) {
                    let full = full as usize;
                    if full <= biSize as usize {
                        // Nothing beyond the header to fetch; reuse what we have.
                        pbmiFull = hdr;
                    } else {
                        pbmiFull =
                            ExAllocatePoolWithTag(PagedPool, full as _, TAG_DIB) as LPBITMAPINFO;
                        if pbmiFull.is_null() {
                            Status = STATUS_INSUFFICIENT_RESOURCES;
                        } else {
                            Status =
                                SafeProbeForRead(bmi as *const core::ffi::c_void, full, 1);
                            if NT_SUCCESS(Status)
                                && !crate::copy::capture_from_user(
                                    pbmiFull as *mut core::ffi::c_void,
                                    bmi as *const core::ffi::c_void,
                                    full,
                                )
                            {
                                Status = STATUS_ACCESS_VIOLATION;
                            }
                        }
                    }
                }
                if pbmiFull != hdr {
                    ExFreePoolWithTag(hdr as *mut core::ffi::c_void, TAG_DIB);
                }
            }
        }
    }

    if !NT_SUCCESS(Status) {
        if !pbmiFull.is_null() {
            ExFreePoolWithTag(pbmiFull as *mut core::ffi::c_void, TAG_DIB);
        }
        SetLastNtError(Status);
        return core::ptr::null_mut();
    }

    // Everything downstream reads the KERNEL copy from here on.
    let bmi = pbmiFull;

    if hDC.is_null() {
        hDC = NtGdiCreateCompatibleDC(core::ptr::null_mut());
        bDesktopDC = 1;
    }

    dc = DC_LockDc(hDC);
    if !dc.is_null() {
        hbitmap = DIB_CreateDIBSection(
            dc,
            bmi,
            Usage,
            Bits,
            hSection,
            dwOffset,
            0,
        );
        DC_UnlockDc(dc);
    } else {
        EngSetLastError(ERROR_INVALID_HANDLE);
    }

    if bDesktopDC != 0 {
        NtGdiDeleteObjectApp(hDC as *mut core::ffi::c_void);
    }

    ExFreePoolWithTag(bmi as *mut core::ffi::c_void, TAG_DIB);

    hbitmap
}

// DIB_CreateDIBSection is APIENTRY (stdcall on i386 → _DIB_CreateDIBSection@28);
// consumers (bitmaps.rs, dib.h) expect stdcall. extern "system" gives the right
// decoration per arch (stdcall i386 / cdecl amd64+arm64).
#[no_mangle]
pub unsafe extern "system" fn DIB_CreateDIBSection(
    dc: PDC,
    bmi: *const BITMAPINFO,
    usage: u32,
    bits: *mut LPVOID,
    section: HANDLE,
    mut offset: u32,
    ovr_pitch: u32,
) -> HBITMAP {
    let mut res: HBITMAP = core::ptr::null_mut();
    let mut bmp: *mut SURFACE = core::ptr::null_mut();
    let mut mapBits: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut ppalDIB: PPALETTE = core::ptr::null_mut();

    let bi: *const BITMAPINFOHEADER = core::ptr::addr_of!((*bmi).bmiHeader);
    let effHeight: i32;
    let totalSize: u32;
    let mut bm: BITMAP = core::mem::zeroed();
    let hSecure: HANDLE;

    if (*bi).biCompression == BI_RLE4 || (*bi).biCompression == BI_RLE8 {
        return core::ptr::null_mut();
    }

    effHeight = if (*bi).biHeight >= 0 { (*bi).biHeight } else { -(*bi).biHeight };
    bm.bmType = 0;
    bm.bmWidth = (*bi).biWidth;
    bm.bmHeight = effHeight;
    bm.bmWidthBytes = if ovr_pitch != 0 {
        ovr_pitch as i32
    } else {
        WIDTH_BYTES_ALIGN32(bm.bmWidth, (*bi).biBitCount as u32) as i32
    };

    bm.bmPlanes = (*bi).biPlanes;
    bm.bmBitsPixel = (*bi).biBitCount;
    bm.bmBits = core::ptr::null_mut();

    totalSize = if (*bi).biSizeImage != 0
        && (*bi).biCompression != BI_RGB
        && (*bi).biCompression != BI_BITFIELDS
    {
        (*bi).biSizeImage
    } else {
        (bm.bmWidthBytes * effHeight) as u32
    };

    if !section.is_null() {
        let mut Sbi: SYSTEM_BASIC_INFORMATION = core::mem::zeroed();
        let mut Status: i32;
        let mapOffset: u32;
        let mut SectionOffset: LARGE_INTEGER = core::mem::zeroed();
        let mut mapSize: usize;

        Status = ZwQuerySystemInformation(
            SystemBasicInformation,
            core::ptr::addr_of_mut!(Sbi) as *mut _,
            core::mem::size_of::<SYSTEM_BASIC_INFORMATION>() as u32,
            core::ptr::null_mut(),
        );
        if !NT_SUCCESS(Status) {
            return core::ptr::null_mut();
        }

        mapOffset = offset - (offset % Sbi.AllocationGranularity);
        mapSize = (totalSize + (offset - mapOffset)) as usize;

        SectionOffset.__bindgen_anon_1.LowPart = mapOffset;
        SectionOffset.__bindgen_anon_1.HighPart = 0;

        Status = ZwMapViewOfSection(
            section,
            NtCurrentProcess(),
            core::ptr::addr_of_mut!(mapBits),
            0,
            0,
            core::ptr::addr_of_mut!(SectionOffset) as *mut _,
            core::ptr::addr_of_mut!(mapSize) as *mut _,
            ViewShare,
            0,
            PAGE_READWRITE,
        );
        if !NT_SUCCESS(Status) {
            EngSetLastError(ERROR_INVALID_PARAMETER);
            return core::ptr::null_mut();
        }

        if !mapBits.is_null() {
            bm.bmBits = (mapBits as *mut u8).add((offset - mapOffset) as usize) as *mut _;
        }
    } else if ovr_pitch != 0 && offset != 0 {
        bm.bmBits = UlongToPtr(offset);
    } else {
        offset = 0;
        bm.bmBits = EngAllocUserMem(totalSize as _, 0);
        if bm.bmBits.is_null() {
            // goto cleanup
            'cleanup: {
                break 'cleanup;
            }
            return cleanup_and_return(
                res, bmp, &mut bm, ppalDIB, section, mapBits, offset, bits,
            );
        }
    }

    hSecure = 0x1 as HANDLE;

    res = GreCreateBitmapEx(
        bm.bmWidth as u32,
        abs(bm.bmHeight) as u32,
        bm.bmWidthBytes as u32,
        BitmapFormat((*bi).biBitCount as u32 * (*bi).biPlanes as u32, (*bi).biCompression),
        (BMF_DONTCACHE | BMF_USERMEM | BMF_NOZEROINIT
            | (if (*bi).biHeight < 0 { BMF_TOPDOWN } else { 0 })) as USHORT,
        totalSize,
        bm.bmBits,
        0,
    );

    'cleanup: {
        if res.is_null() {
            EngSetLastError(ERROR_NO_SYSTEM_RESOURCES);
            break 'cleanup;
        }
        bmp = SURFACE_ShareLockSurface(res);
        if bmp.is_null() {
            EngSetLastError(ERROR_INVALID_HANDLE);
            break 'cleanup;
        }

        (*bmp).hDIBSection = section;
        (*bmp).hSecure = hSecure;
        (*bmp).dwOffset = offset;
        (*bmp).flags = API_BITMAP;
        (*bmp).biClrImportant = (*bi).biClrImportant;

        ppalDIB = CreateDIBPalette(bmi, dc, usage);
    }

    if res.is_null() || bmp.is_null() || bm.bmBits.is_null() || ppalDIB.is_null() {
        if !bm.bmBits.is_null() {
            if !section.is_null() {
                ZwUnmapViewOfSection(NtCurrentProcess(), mapBits);
                bm.bmBits = core::ptr::null_mut();
            } else if offset == 0 {
                EngFreeUserMem(bm.bmBits);
                bm.bmBits = core::ptr::null_mut();
            }
        }

        if !bmp.is_null() {
            SURFACE_ShareUnlockSurface(bmp);
            bmp = core::ptr::null_mut();
        }

        if !res.is_null() {
            GreDeleteObject(res as *mut core::ffi::c_void);
            res = core::ptr::null_mut();
        }

        if !ppalDIB.is_null() {
            PALETTE_ShareUnlockPalette(ppalDIB);
        }
    }

    if !bmp.is_null() {
        SURFACE_vSetPalette(bmp, ppalDIB);
        PALETTE_ShareUnlockPalette(ppalDIB);
        SURFACE_ShareUnlockSurface(bmp);
    }

    if !bm.bmBits.is_null() && !bits.is_null() {
        *bits = bm.bmBits;
    }

    res
}


#[inline]
unsafe fn cleanup_and_return(
    _res: HBITMAP,
    _bmp: *mut SURFACE,
    _bm: *mut BITMAP,
    _ppalDIB: PPALETTE,
    _section: HANDLE,
    _mapBits: *mut core::ffi::c_void,
    _offset: u32,
    _bits: *mut LPVOID,
) -> HBITMAP {
    core::ptr::null_mut()
}

#[cfg(target_pointer_width = "32")]
unsafe extern "fastcall" fn DIB_GetBitmapInfo(
    header: *const BITMAPINFOHEADER,
    width: *mut i32,
    height: *mut i32,
    planes: *mut u16,
    bpp: *mut u16,
    compr: *mut u32,
    size: *mut u32,
) -> i32 {
    DIB_GetBitmapInfo_impl(header, width, height, planes, bpp, compr, size)
}

#[cfg(not(target_pointer_width = "32"))]
unsafe extern "C" fn DIB_GetBitmapInfo(
    header: *const BITMAPINFOHEADER,
    width: *mut i32,
    height: *mut i32,
    planes: *mut u16,
    bpp: *mut u16,
    compr: *mut u32,
    size: *mut u32,
) -> i32 {
    DIB_GetBitmapInfo_impl(header, width, height, planes, bpp, compr, size)
}

#[inline]
unsafe fn DIB_GetBitmapInfo_impl(
    header: *const BITMAPINFOHEADER,
    width: *mut i32,
    height: *mut i32,
    planes: *mut u16,
    bpp: *mut u16,
    compr: *mut u32,
    size: *mut u32,
) -> i32 {
    if (*header).biSize as usize == core::mem::size_of::<BITMAPCOREHEADER>() {
        let core_hdr = header as *const BITMAPCOREHEADER;
        *width = (*core_hdr).bcWidth as i32;
        *height = (*core_hdr).bcHeight as i32;
        *planes = (*core_hdr).bcPlanes;
        *bpp = (*core_hdr).bcBitCount;
        *compr = BI_RGB;
        *size = 0;
        return 0;
    }
    if (*header).biSize as usize >= core::mem::size_of::<BITMAPINFOHEADER>() {
        *width = (*header).biWidth;
        *height = (*header).biHeight;
        *planes = (*header).biPlanes;
        *bpp = (*header).biBitCount;
        *compr = (*header).biCompression;
        *size = (*header).biSizeImage;
        return 1;
    }
    -1
}

#[no_mangle]
pub unsafe extern "system" fn DIB_GetDIBImageBytes(width: INT, height: INT, depth: INT) -> INT {
    WIDTH_BYTES_ALIGN32(width, depth) as i32 * (if height < 0 { -height } else { height })
}

#[cfg(target_pointer_width = "32")]
#[no_mangle]
unsafe extern "fastcall" fn DIB_BitmapInfoSize(info: *const BITMAPINFO, coloruse: u16) -> i32 {
    DIB_BitmapInfoSize_impl(info, coloruse)
}

#[cfg(not(target_pointer_width = "32"))]
#[no_mangle]
unsafe extern "C" fn DIB_BitmapInfoSize(info: *const BITMAPINFO, coloruse: u16) -> i32 {
    DIB_BitmapInfoSize_impl(info, coloruse)
}

#[inline(always)]
unsafe fn DIB_BitmapInfoSize_impl(info: *const BITMAPINFO, coloruse: u16) -> i32 {
    let mut colors: u32;
    let size: u32;
    let mut masks: u32 = 0;
    let colorsize: u32;

    colorsize = if coloruse as u32 == DIB_RGB_COLORS {
        core::mem::size_of::<RGBQUAD>() as u32
    } else if coloruse as u32 == DIB_PAL_INDICES {
        0
    } else {
        core::mem::size_of::<u16>() as u32
    };

    if (*info).bmiHeader.biSize as usize == core::mem::size_of::<BITMAPCOREHEADER>() {
        let core_hdr = info as *const BITMAPCOREHEADER;
        colors = if (*core_hdr).bcBitCount <= 8 {
            1u32 << (*core_hdr).bcBitCount
        } else {
            0
        };
        return (core::mem::size_of::<BITMAPCOREHEADER>() as u32 + colors * colorsize) as i32;
    } else {
        colors = (*info).bmiHeader.biClrUsed;
        if colors > 256 {
            colors = 256;
        }
        if colors == 0 && (*info).bmiHeader.biBitCount <= 8 {
            colors = 1u32 << (*info).bmiHeader.biBitCount;
        }
        if (*info).bmiHeader.biCompression == BI_BITFIELDS {
            masks = 3;
        }
        let a = (*info).bmiHeader.biSize;
        let b = (core::mem::size_of::<BITMAPINFOHEADER>() as u32) + masks * (core::mem::size_of::<u32>() as u32);
        size = if a > b { a } else { b };
        return (size + colors * colorsize) as i32;
    }
}

fastcall_fn! {
unsafe fn DIB_MapPaletteColors(ppalDc: PPALETTE, lpbmi: *const BITMAPINFO) -> HPALETTE {
    let ppalNew: PPALETTE;
    let mut nNumColors: ULONG;
    let mut lpIndex: *mut USHORT;
    let hpal: HPALETTE;

    if ((*ppalDc).flFlags & PAL_INDEXED) == 0 {
        return core::ptr::null_mut();
    }

    nNumColors = 1u32 << (*lpbmi).bmiHeader.biBitCount;
    if (*lpbmi).bmiHeader.biClrUsed != 0 {
        if (*lpbmi).bmiHeader.biClrUsed < nNumColors {
            nNumColors = (*lpbmi).bmiHeader.biClrUsed;
        }
    }

    ppalNew = PALETTE_AllocPalWithHandle(PAL_INDEXED, nNumColors, core::ptr::null_mut(), 0, 0, 0);
    if ppalNew.is_null() {
        return core::ptr::null_mut();
    }

    lpIndex = ((lpbmi as PBYTE).add((*lpbmi).bmiHeader.biSize as usize)) as *mut USHORT;

    let mut i: ULONG = 0;
    while i < nNumColors {
        let iColorIndex: ULONG = (*lpIndex as ULONG) % (*ppalDc).NumColors;
        *(*ppalNew).IndexedColors.add(i as usize) =
            *(*ppalDc).IndexedColors.add(iColorIndex as usize);
        lpIndex = lpIndex.add(1);
        i += 1;
    }

    hpal = (*ppalNew).BaseObject.hHmgr as HPALETTE;
    PALETTE_UnlockPalette(ppalNew);

    hpal
}
}

#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "fastcall" fn DIB_FreeConvertedBitmapInfo(
    converted: *mut BITMAPINFO,
    orig: *mut BITMAPINFO,
    usage: u32,
) {
    DIB_FreeConvertedBitmapInfo_impl(converted, orig, usage)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
unsafe extern "C" fn DIB_FreeConvertedBitmapInfo(
    converted: *mut BITMAPINFO,
    orig: *mut BITMAPINFO,
    usage: u32,
) {
    DIB_FreeConvertedBitmapInfo_impl(converted, orig, usage)
}

unsafe fn DIB_FreeConvertedBitmapInfo_impl(
    converted: *mut BITMAPINFO,
    orig: *mut BITMAPINFO,
    usage: u32,
) {

    if converted == orig {
        return;
    }

    if usage == 0xFFFFFFFF {
        ExFreePoolWithTag(converted as *mut _, TAG_DIB);
        return;
    }

    let pbmci = orig as *mut BITMAPCOREINFO;

    (*pbmci).bmciHeader.bcBitCount = (*converted).bmiHeader.biBitCount;
    (*pbmci).bmciHeader.bcWidth = (*converted).bmiHeader.biWidth as u16;
    (*pbmci).bmciHeader.bcHeight = (*converted).bmiHeader.biHeight as u16;
    (*pbmci).bmciHeader.bcPlanes = (*converted).bmiHeader.biPlanes;

    if (*pbmci).bmciHeader.bcBitCount <= 8 {
        let mut numColors: u32 = (*converted).bmiHeader.biClrUsed;
        if numColors == 0 {
            numColors = 1u32 << (*pbmci).bmciHeader.bcBitCount;
        }
        let totalColors: u32 = 1u32 << (*pbmci).bmciHeader.bcBitCount;

        if usage == DIB_PAL_COLORS {
            core::ptr::write_bytes(
                core::ptr::addr_of_mut!((*pbmci).bmciColors) as *mut u8,
                0,
                (totalColors as usize) * core::mem::size_of::<u16>(),
            );
            core::ptr::copy_nonoverlapping(
                core::ptr::addr_of!((*converted).bmiColors) as *const u8,
                core::ptr::addr_of_mut!((*pbmci).bmciColors) as *mut u8,
                (numColors as usize) * core::mem::size_of::<u16>(),
            );
        } else {
            core::ptr::write_bytes(
                core::ptr::addr_of_mut!((*pbmci).bmciColors) as *mut u8,
                0,
                (totalColors as usize) * core::mem::size_of::<RGBTRIPLE>(),
            );
            let dst = core::ptr::addr_of_mut!((*pbmci).bmciColors) as *mut RGBTRIPLE;
            let src = core::ptr::addr_of!((*converted).bmiColors) as *const u32;
            let mut i: u32 = 0;
            while i < numColors {
                let src_quad = src.add(i as usize) as *const u8;
                // RGBQUAD: rgbBlue, rgbGreen, rgbRed, rgbReserved
                let rgbBlue = *src_quad.add(0);
                let rgbGreen = *src_quad.add(1);
                let rgbRed = *src_quad.add(2);
                (*dst.add(i as usize)).rgbtRed = rgbRed;
                (*dst.add(i as usize)).rgbtGreen = rgbGreen;
                (*dst.add(i as usize)).rgbtBlue = rgbBlue;
                i += 1;
            }
        }
    }

    ExFreePoolWithTag(converted as *mut _, TAG_DIB);
}

// GreCreateDIBitmapFromPackedDIB (dibobj.split2.c) — missed by the rustconv scanner
// (multi-line _In_reads_ SAL signature); hand-translated. NTAPI → extern "system".
#[no_mangle]
pub unsafe extern "system" fn GreCreateDIBitmapFromPackedDIB(
    pvPackedDIB: PVOID,
    cjPackedDIB: UINT,
    uUsage: ULONG,
) -> HBITMAP {
    // We only support BITMAPINFOHEADER, make sure the size is ok
    if (cjPackedDIB as usize) < core::mem::size_of::<BITMAPINFOHEADER>() {
        return core::ptr::null_mut();
    }
    let pbmi = pvPackedDIB as PBITMAPINFO;
    if cjPackedDIB < (*pbmi).bmiHeader.biSize {
        return core::ptr::null_mut();
    }
    // Calculate the info size and make sure the packed DIB is large enough
    let cjInfo = DIB_BitmapInfoSize(pbmi, uUsage as u16) as UINT;
    if cjPackedDIB <= cjInfo {
        return core::ptr::null_mut();
    }
    // The bitmap bits start after the header
    let pjBits = (pvPackedDIB as PBYTE).add(cjInfo as usize);
    let cjBits = cjPackedDIB - cjInfo;
    GreCreateDIBitmapInternal(
        core::ptr::null_mut(),
        (*pbmi).bmiHeader.biWidth,
        abs((*pbmi).bmiHeader.biHeight),
        (CBM_INIT | CBM_CREATDIB) as DWORD,
        pjBits,
        pbmi,
        uUsage,
        0,
        cjBits,
        core::ptr::null_mut(),
    )
}


// DIB_ConvertBitmapInfo (dibobj.split2.c) — scanner-missed (FASTCALL + CONST signature).
// BITMAPCOREINFO → BITMAPINFO converter; hand-translated.
#[cfg(target_pointer_width = "32")]
#[no_mangle]
pub unsafe extern "fastcall" fn DIB_ConvertBitmapInfo(pbmi: *const BITMAPINFO, Usage: DWORD) -> *mut BITMAPINFO { DIB_ConvertBitmapInfo_impl(pbmi, Usage) }
#[cfg(not(target_pointer_width = "32"))]
#[no_mangle]
pub unsafe extern "C" fn DIB_ConvertBitmapInfo(pbmi: *const BITMAPINFO, Usage: DWORD) -> *mut BITMAPINFO { DIB_ConvertBitmapInfo_impl(pbmi, Usage) }
#[inline(always)]
unsafe fn DIB_ConvertBitmapInfo_impl(pbmi: *const BITMAPINFO, Usage: DWORD) -> *mut BITMAPINFO {
    let pbmci = pbmi as *const BITMAPCOREINFO;
    let mut numColors: UINT = 0;
    let mut ColorsSize: UINT = 0;

    if (*pbmi).bmiHeader.biSize >= core::mem::size_of::<BITMAPINFOHEADER>() as u32 {
        return pbmi as *mut BITMAPINFO;
    }
    if (*pbmi).bmiHeader.biSize != core::mem::size_of::<BITMAPCOREHEADER>() as u32 {
        return core::ptr::null_mut();
    }

    if (*pbmci).bmciHeader.bcBitCount <= 8 {
        numColors = 1u32 << (*pbmci).bmciHeader.bcBitCount;
        if Usage == DIB_PAL_COLORS {
            ColorsSize = numColors * core::mem::size_of::<WORD>() as u32;
        } else {
            ColorsSize = numColors * core::mem::size_of::<RGBQUAD>() as u32;
        }
    } else if Usage == DIB_PAL_COLORS {
        return core::ptr::null_mut();
    }

    let pNewBmi = ExAllocatePoolWithTag(PagedPool, (core::mem::size_of::<BITMAPINFOHEADER>() as u32 + ColorsSize) as _, TAG_DIB) as *mut BITMAPINFO;
    if pNewBmi.is_null() { return core::ptr::null_mut(); }

    RtlZeroMemory(pNewBmi as *mut c_void, core::mem::size_of::<BITMAPINFOHEADER>() + ColorsSize as usize);

    (*pNewBmi).bmiHeader.biSize = core::mem::size_of::<BITMAPINFOHEADER>() as u32;
    (*pNewBmi).bmiHeader.biBitCount = (*pbmci).bmciHeader.bcBitCount;
    (*pNewBmi).bmiHeader.biWidth = (*pbmci).bmciHeader.bcWidth as i32;
    (*pNewBmi).bmiHeader.biHeight = (*pbmci).bmciHeader.bcHeight as i32;
    (*pNewBmi).bmiHeader.biPlanes = (*pbmci).bmciHeader.bcPlanes;
    (*pNewBmi).bmiHeader.biCompression = BI_RGB;
    (*pNewBmi).bmiHeader.biSizeImage = DIB_GetDIBImageBytes((*pNewBmi).bmiHeader.biWidth, (*pNewBmi).bmiHeader.biHeight, (*pNewBmi).bmiHeader.biBitCount as i32) as u32;
    (*pNewBmi).bmiHeader.biClrUsed = numColors;

    if Usage == DIB_PAL_COLORS {
        RtlCopyMemory((*pNewBmi).bmiColors.as_mut_ptr() as *mut c_void, (*pbmci).bmciColors.as_ptr() as *const c_void, ColorsSize as usize);
    } else {
        let mut i: UINT = 0;
        while i < numColors {
            (*(*pNewBmi).bmiColors.as_mut_ptr().add(i as usize)).rgbRed = (*(*pbmci).bmciColors.as_ptr().add(i as usize)).rgbtRed;
            (*(*pNewBmi).bmiColors.as_mut_ptr().add(i as usize)).rgbGreen = (*(*pbmci).bmciColors.as_ptr().add(i as usize)).rgbtGreen;
            (*(*pNewBmi).bmiColors.as_mut_ptr().add(i as usize)).rgbBlue = (*(*pbmci).bmciColors.as_ptr().add(i as usize)).rgbtBlue;
            i += 1;
        }
    }
    pNewBmi
}



// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS win32 kernel mode subsystem
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            win32ss/gdi/ntgdi/dibobj.c
//  * PURPOSE:         Dib object functions
//  * PROGRAMMER:
//  */
//
// #include <win32k.h>
// #include <safe_user_io.h>
//
// #define NDEBUG
// #include <debug.h>
//
// static const RGBQUAD DefLogPaletteQuads[20] =   /* Copy of Default Logical Palette */
// {
//     /* rgbBlue, rgbGreen, rgbRed, rgbReserved */
//     { 0x00, 0x00, 0x00, 0x00 },
//     { 0x00, 0x00, 0x80, 0x00 },
//     { 0x00, 0x80, 0x00, 0x00 },
//     { 0x00, 0x80, 0x80, 0x00 },
//     { 0x80, 0x00, 0x00, 0x00 },
//     { 0x80, 0x00, 0x80, 0x00 },
//     { 0x80, 0x80, 0x00, 0x00 },
//     { 0xc0, 0xc0, 0xc0, 0x00 },
//     { 0xc0, 0xdc, 0xc0, 0x00 },
//     { 0xf0, 0xca, 0xa6, 0x00 },
//     { 0xf0, 0xfb, 0xff, 0x00 },
//     { 0xa4, 0xa0, 0xa0, 0x00 },
//     { 0x80, 0x80, 0x80, 0x00 },
//     { 0x00, 0x00, 0xff, 0x00 },
//     { 0x00, 0xff, 0x00, 0x00 },
//     { 0x00, 0xff, 0xff, 0x00 },
//     { 0xff, 0x00, 0x00, 0x00 },
//     { 0xff, 0x00, 0xff, 0x00 },
//     { 0xff, 0xff, 0x00, 0x00 },
//     { 0xff, 0xff, 0xff, 0x00 }
// };
//
// PPALETTE
// NTAPI
// CreateDIBPalette(
//     _In_ const BITMAPINFO *pbmi,
//     _In_ PDC pdc,
//     _In_ ULONG iUsage)
// {
//     PPALETTE ppal;
//     ULONG i, cBitsPixel, cColors;
//
//     if (pbmi->bmiHeader.biSize < sizeof(BITMAPINFOHEADER))
//     {
//         PBITMAPCOREINFO pbci = (PBITMAPCOREINFO)pbmi;
//         cBitsPixel = pbci->bmciHeader.bcBitCount;
//     }
//     else
//     {
//         cBitsPixel = pbmi->bmiHeader.biBitCount;
//     }
//
//     /* Check if the colors are indexed */
//     if (cBitsPixel <= 8)
//     {
//         /* We create a "full" palette */
//         cColors = 1 << cBitsPixel;
//
//         /* Allocate the palette */
//         ppal = PALETTE_AllocPalette(PAL_INDEXED | PAL_DIBSECTION,
//                                     cColors,
//                                     NULL,
//                                     0,
//                                     0,
//                                     0);
//         if (ppal == NULL)
//         {
//             DPRINT1("Failed to allocate palette.\n");
//             return NULL;
//         }
//
//         /* Check if the BITMAPINFO specifies how many colors to use */
//         if ((pbmi->bmiHeader.biSize >= sizeof(BITMAPINFOHEADER)) &&
//             (pbmi->bmiHeader.biClrUsed != 0))
//         {
//             /* This is how many colors we can actually process */
//             cColors = min(cColors, pbmi->bmiHeader.biClrUsed);
//         }
//
//         /* Check how to use the colors */
//         if (iUsage == DIB_PAL_COLORS)
//         {
//             COLORREF crColor;
//
//             /* The colors are an array of WORD indices into the DC palette */
//             PWORD pwColors = (PWORD)((PCHAR)pbmi + pbmi->bmiHeader.biSize);
//
//             /* Use the DCs palette or, if no DC is given, the default one */
//             PPALETTE ppalDC = pdc ? pdc->dclevel.ppal : gppalDefault;
//
//             /* Loop all color indices in the DIB */
//             for (i = 0; i < cColors; i++)
//             {
//                 /* Get the palette index and handle wraparound when exceeding
//                    the number of colors in the DC palette */
//                 WORD wIndex = pwColors[i] % ppalDC->NumColors;
//
//                 /* USe the RGB value from the DC palette */
//                 crColor = PALETTE_ulGetRGBColorFromIndex(ppalDC, wIndex);
//                 PALETTE_vSetRGBColorForIndex(ppal, i, crColor);
//             }
//         }
//         else if (iUsage == DIB_PAL_BRUSHHACK)
//         {
//             /* The colors are an array of WORD indices into the DC palette */
//             PWORD pwColors = (PWORD)((PCHAR)pbmi + pbmi->bmiHeader.biSize);
//
//             /* Loop all color indices in the DIB */
//             for (i = 0; i < cColors; i++)
//             {
//                 /* Set the index directly as the RGB color, the real palette
//                    containing RGB values will be calculated when the brush is
//                    realized */
//                 PALETTE_vSetRGBColorForIndex(ppal, i, pwColors[i]);
//             }
//
//             /* Mark the palette as a brush hack palette */
//             ppal->flFlags |= PAL_BRUSHHACK;
//         }
// //        else if (iUsage == 2)
// //        {
//             // FIXME: this one is undocumented
// //            ASSERT(FALSE);
// //        }
//         else if (pbmi->bmiHeader.biSize >= sizeof(BITMAPINFOHEADER))
//         {
//             /* The colors are an array of RGBQUAD values */
//             RGBQUAD *prgb = (RGBQUAD*)((PCHAR)pbmi + pbmi->bmiHeader.biSize);
//             RGBQUAD colors[256] = {{0}};
//
//             // FIXME: do we need to handle PALETTEINDEX / PALETTERGB macro?
//
//             /* Use SEH to verify we can READ prgb[] succesfully */
//             _SEH2_TRY
//             {
//                 RtlCopyMemory(colors, prgb, cColors * sizeof(colors[0]));
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//               /* Do Nothing */
//             }
//             _SEH2_END;
//
//             for (i = 0; i < cColors; ++i)
//             {
//                 /* Get the color value and translate it to a COLORREF */
//                 COLORREF crColor = RGB(colors[i].rgbRed, colors[i].rgbGreen, colors[i].rgbBlue);
//
//                 /* Set the RGB value in the palette */
//                 PALETTE_vSetRGBColorForIndex(ppal, i, crColor);
//             }
//         }
//         else
//         {
//             /* The colors are an array of RGBTRIPLE values */
//             RGBTRIPLE *prgb = (RGBTRIPLE*)((PCHAR)pbmi + pbmi->bmiHeader.biSize);
//
//             /* Loop all color indices in the DIB */
//             for (i = 0; i < cColors; i++)
//             {
//                 /* Get the color value and translate it to a COLORREF */
//                 RGBTRIPLE rgb = prgb[i];
//                 COLORREF crColor = RGB(rgb.rgbtRed, rgb.rgbtGreen, rgb.rgbtBlue);
//
//                 /* Set the RGB value in the palette */
//                 PALETTE_vSetRGBColorForIndex(ppal, i, crColor);
//             }
//         }
//     }
//     else
//     {
//         /* This is a bitfield / RGB palette */
//         ULONG flRedMask, flGreenMask, flBlueMask;
//
//         /* Check if the DIB contains bitfield values */
//         if ((pbmi->bmiHeader.biSize >= sizeof(BITMAPINFOHEADER)) &&
//             (pbmi->bmiHeader.biCompression == BI_BITFIELDS))
//         {
//             /* Check if we have a v4/v5 header */
//             if (pbmi->bmiHeader.biSize >= sizeof(BITMAPV4HEADER))
//             {
//                 /* The masks are dedicated fields in the header */
//                 PBITMAPV4HEADER pbmV4Header = (PBITMAPV4HEADER)&pbmi->bmiHeader;
//                 flRedMask = pbmV4Header->bV4RedMask;
//                 flGreenMask = pbmV4Header->bV4GreenMask;
//                 flBlueMask = pbmV4Header->bV4BlueMask;
//             }
//             else
//             {
//                 /* The masks are the first 3 values in the DIB color table */
//                 PDWORD pdwColors = (PVOID)((PCHAR)pbmi + pbmi->bmiHeader.biSize);
//                 flRedMask = pdwColors[0];
//                 flGreenMask = pdwColors[1];
//                 flBlueMask = pdwColors[2];
//             }
//         }
//         else
//         {
//             /* Check what bit depth we have. Note: optimization flags are
//                calculated in PALETTE_AllocPalette()  */
//             if (cBitsPixel == 16)
//             {
//                 /* This is an RGB 555 palette */
//                 flRedMask = 0x7C00;
//                 flGreenMask = 0x03E0;
//                 flBlueMask = 0x001F;
//             }
//             else
//             {
//                 /* This is an RGB 888 palette */
//                 flRedMask = 0xFF0000;
//                 flGreenMask = 0x00FF00;
//                 flBlueMask = 0x0000FF;
//             }
//         }
//
//         /* Allocate the bitfield palette */
//         ppal = PALETTE_AllocPalette(PAL_BITFIELDS | PAL_DIBSECTION,
//                                     0,
//                                     NULL,
//                                     flRedMask,
//                                     flGreenMask,
//                                     flBlueMask);
//     }
//
//     /* We're done, return the palette */
//     return ppal;
// }
//
// // Converts a DIB to a device-dependent bitmap
// static INT
// FASTCALL
// IntSetDIBits(
//     PDC   DC,
//     HBITMAP  hBitmap,
//     UINT  StartScan,
//     UINT  ScanLines,
//     CONST VOID  *Bits,
//     ULONG cjMaxBits,
//     CONST BITMAPINFO  *bmi,
//     UINT  ColorUse)
// {
//     HBITMAP     SourceBitmap;
//     PSURFACE    psurfDst, psurfSrc;
//     INT         result = 0;
//     RECT		rcDst;
//     POINTL		ptSrc;
//     EXLATEOBJ	exlo;
//     PPALETTE    ppalDIB = 0;
//     ULONG cjSizeImage;
//
//     if (!bmi || !Bits) return 0;
//
//     /* Check for uncompressed formats */
//     if ((bmi->bmiHeader.biCompression == BI_RGB) ||
//              (bmi->bmiHeader.biCompression == BI_BITFIELDS))
//     {
//         /* Calculate the image size */
//         cjSizeImage = DIB_GetDIBImageBytes(bmi->bmiHeader.biWidth,
//                                            ScanLines,
//                                            bmi->bmiHeader.biBitCount);
//     }
//     /* Check if the header provided an image size */
//     else if (bmi->bmiHeader.biSizeImage != 0)
//     {
//         /* Use the given size */
//         cjSizeImage = bmi->bmiHeader.biSizeImage;
//     }
//     else
//     {
//         /* Compressed format without a size. This is invalid. */
//         DPRINT1("Compressed format without a size\n");
//         return 0;
//     }
//
//     /* Check if the size that we have is ok */
//     if ((cjSizeImage > cjMaxBits) || (cjSizeImage == 0))
//     {
//         DPRINT1("Invalid bitmap size! cjSizeImage = %lu, cjMaxBits = %lu\n",
//                 cjSizeImage, cjMaxBits);
//         return 0;
//     }
//
//     SourceBitmap = GreCreateBitmapEx(bmi->bmiHeader.biWidth,
//                                      ScanLines,
//                                      0,
//                                      BitmapFormat(bmi->bmiHeader.biBitCount,
//                                                   bmi->bmiHeader.biCompression),
//                                      bmi->bmiHeader.biHeight < 0 ? BMF_TOPDOWN : 0,
//                                      cjSizeImage,
//                                      (PVOID)Bits,
//                                      0);
//     if (!SourceBitmap)
//     {
//         DPRINT1("Error: Could not create a bitmap.\n");
//         EngSetLastError(ERROR_NO_SYSTEM_RESOURCES);
//         return 0;
//     }
//
//     psurfDst = SURFACE_ShareLockSurface(hBitmap);
//     psurfSrc = SURFACE_ShareLockSurface(SourceBitmap);
//
//     if(!(psurfSrc && psurfDst))
//     {
//         DPRINT1("Error: Could not lock surfaces\n");
//         goto cleanup;
//     }
//
//     /* Create a palette for the DIB */
//     ppalDIB = CreateDIBPalette(bmi, DC, ColorUse);
//     if (!ppalDIB)
//     {
//         EngSetLastError(ERROR_NO_SYSTEM_RESOURCES);
//         goto cleanup;
//     }
//
//     /* Initialize EXLATEOBJ */
//     EXLATEOBJ_vInitialize(&exlo,
//                           ppalDIB,
//                           psurfDst->ppal,
//                           RGB(0xff, 0xff, 0xff),
//                           RGB(0xff, 0xff, 0xff), //DC->pdcattr->crBackgroundClr,
//                           0); // DC->pdcattr->crForegroundClr);
//
//     rcDst.top = StartScan;
//     rcDst.left = 0;
//     rcDst.bottom = rcDst.top + ScanLines;
//     rcDst.right = psurfDst->SurfObj.sizlBitmap.cx;
//     ptSrc.x = 0;
//     ptSrc.y = 0;
//
//     result = IntEngCopyBits(&psurfDst->SurfObj,
//                             &psurfSrc->SurfObj,
//                             NULL,
//                             &exlo.xlo,
//                             &rcDst,
//                             &ptSrc);
//     if(result)
//         result = ScanLines;
//
//     EXLATEOBJ_vCleanup(&exlo);
//
// cleanup:
//     if (ppalDIB) PALETTE_ShareUnlockPalette(ppalDIB);
//     if(psurfSrc) SURFACE_ShareUnlockSurface(psurfSrc);
//     if(psurfDst) SURFACE_ShareUnlockSurface(psurfDst);
//     GreDeleteObject(SourceBitmap);
//
//     return result;
// }
//
// static
// HBITMAP
// IntGdiCreateMaskFromRLE(
//     DWORD Width,
//     DWORD Height,
//     ULONG Compression,
//     const BYTE* Bits,
//     DWORD BitsSize)
// {
//     HBITMAP Mask;
//     DWORD x, y;
//     SURFOBJ* SurfObj;
//     UINT i = 0;
//     BYTE Data, NumPixels, ToSkip;
//
//     ASSERT((Compression == BI_RLE8) || (Compression == BI_RLE4));
//
//     /* Create the bitmap */
//     Mask = GreCreateBitmapEx(Width, Height, 0, BMF_1BPP, 0, 0, NULL, 0);
//     if (!Mask)
//         return NULL;
//
//     SurfObj = EngLockSurface((HSURF)Mask);
//     if (!SurfObj)
//     {
//         GreDeleteObject(Mask);
//         return NULL;
//     }
//     ASSERT(SurfObj->pvBits != NULL);
//
//     x = y = 0;
//
//     while (i < BitsSize)
//     {
//         NumPixels = Bits[i];
//         Data = Bits[i + 1];
//         i += 2;
//
//         if (NumPixels != 0)
//         {
//             if ((x + NumPixels) > Width)
//                 NumPixels = Width - x;
//
//             if (NumPixels == 0)
//                 continue;
//
//             DIB_1BPP_HLine(SurfObj, x, x + NumPixels, y, 1);
//             x += NumPixels;
//             continue;
//         }
//
//         if (Data < 3)
//         {
//             switch (Data)
//             {
//                 case 0:
//                     /* End of line */
//                     y++;
//                     if (y == Height)
//                         goto done;
//                     x = 0;
//                     break;
//                 case 1:
//                     /* End of file */
//                     goto done;
//                 case 2:
//                     /* Jump */
//                     if (i >= (BitsSize - 1))
//                         goto done;
//                     x += Bits[i];
//                     if (x > Width)
//                         x = Width;
//                     y += Bits[i + 1];
//                     if (y >= Height)
//                         goto done;
//                     i += 2;
//                     break;
//             }
//             /* Done for this run */
//             continue;
//         }
//
//         /* Embedded data into the RLE */
//         NumPixels = Data;
//         if (Compression == BI_RLE8)
//             ToSkip = NumPixels;
//         else
//             ToSkip = (NumPixels / 2) + (NumPixels & 1);
//
//         if ((i + ToSkip) > BitsSize)
//             goto done;
//         ToSkip = (ToSkip + 1) & ~1;
//
//         if ((x + NumPixels) > Width)
//             NumPixels = Width - x;
//
//         if (NumPixels != 0)
//         {
//             DIB_1BPP_HLine(SurfObj, x, x + NumPixels, y, 1);
//             x += NumPixels;
//         }
//         i += ToSkip;
//     }
//
// done:
//     EngUnlockSurface(SurfObj);
//     return Mask;
// }
//
// W32KAPI
// INT
// APIENTRY
// NtGdiSetDIBitsToDeviceInternal(
//     IN HDC hDC,
//     IN INT XDest,
//     IN INT YDest,
//     IN DWORD Width,
//     IN DWORD Height,
//     IN INT XSrc,
//     IN INT YSrc,
//     IN DWORD StartScan,
//     IN DWORD ScanLines,
//     IN LPBYTE Bits,
//     IN LPBITMAPINFO bmi,
//     IN DWORD ColorUse,
//     IN UINT cjMaxBits,
//     IN UINT cjMaxInfo,
//     IN BOOL bTransformCoordinates,
//     IN OPTIONAL HANDLE hcmXform)
// {
//     INT ret;
//     PDC pDC = NULL;
//     HBITMAP hSourceBitmap = NULL, hMaskBitmap = NULL;
//     SURFOBJ *pDestSurf, *pSourceSurf = NULL, *pMaskSurf = NULL;
//     SURFACE *pSurf;
//     RECTL rcDest;
//     POINTL ptSource;
//     //INT DIBWidth;
//     SIZEL SourceSize;
//     EXLATEOBJ exlo;
//     PPALETTE ppalDIB = NULL;
//     LPBITMAPINFO pbmiSafe;
//     BOOL bResult;
//     NTSTATUS Status;
//
//     if (!Bits) return 0;
//
//     pbmiSafe = ExAllocatePoolWithTag(PagedPool, cjMaxInfo, 'pmTG');
//     if (!pbmiSafe) return 0;
//
//     Status = SafeProbeForRead(bmi, cjMaxInfo as usize, 1);
//     if (NT_SUCCESS(Status)) Status = SafeProbeForRead(Bits, cjMaxBits as usize, 1);
//     if (!NT_SUCCESS(Status))
//     {
//         ret = 0;
//         goto Exit;
//     }
//     else
//     {
//         RtlCopyMemory(pbmiSafe, bmi, cjMaxInfo);
//         bmi = pbmiSafe;
//     }
//
//     DPRINT("StartScan %d ScanLines %d Bits %p bmi %p ColorUse %d\n"
//            "    Height %d Width %d biSizeImage %d\n"
//            "    biHeight %d biWidth %d biBitCount %d\n"
//            "    XSrc %d YSrc %d XDest %d YDest %d\n",
//            StartScan, ScanLines, Bits, bmi, ColorUse,
//            Height, Width, bmi->bmiHeader.biSizeImage,
//            bmi->bmiHeader.biHeight, bmi->bmiHeader.biWidth,
//            bmi->bmiHeader.biBitCount,
//            XSrc, YSrc, XDest, YDest);
//
//     if (YDest < 0)
//     {
//         ScanLines = min(ScanLines, abs(bmi->bmiHeader.biHeight) - StartScan);
//     }
//
//     if (ScanLines == 0)
//     {
//         DPRINT1("ScanLines == 0\n");
//         ret = 0;
//         goto Exit;
//     }
//
//     pDC = DC_LockDc(hDC);
//     if (!pDC)
//     {
//         EngSetLastError(ERROR_INVALID_HANDLE);
//         ret = 0;
//         goto Exit;
//     }
//
//     if (pDC->dctype == DCTYPE_INFO)
//     {
//         ret = 0;
//         goto Exit;
//     }
//
//     rcDest.left = XDest;
//     rcDest.top = YDest;
//     if (bTransformCoordinates)
//     {
//         IntLPtoDP(pDC, (LPPOINT)&rcDest, 2);
//     }
//     rcDest.left += pDC->ptlDCOrig.x;
//     rcDest.top += pDC->ptlDCOrig.y;
//     rcDest.right = rcDest.left + Width;
//     rcDest.bottom = rcDest.top + Height;
//     rcDest.top += StartScan;
//
//     if (pDC->fs & (DC_ACCUM_APP|DC_ACCUM_WMGR))
//     {
//        IntUpdateBoundsRect(pDC, &rcDest);
//     }
//
//     ptSource.x = XSrc;
//     ptSource.y = YSrc;
//
//     SourceSize.cx = bmi->bmiHeader.biWidth;
//     SourceSize.cy = ScanLines;
//
//     //DIBWidth = WIDTH_BYTES_ALIGN32(SourceSize.cx, bmi->bmiHeader.biBitCount);
//
//     hSourceBitmap = GreCreateBitmapEx(bmi->bmiHeader.biWidth,
//                                       ScanLines,
//                                       0,
//                                       BitmapFormat(bmi->bmiHeader.biBitCount,
//                                                    bmi->bmiHeader.biCompression),
//                                       bmi->bmiHeader.biHeight < 0 ? BMF_TOPDOWN : 0,
//                                       bmi->bmiHeader.biSizeImage,
//                                       Bits,
//                                       0);
//
//     if (!hSourceBitmap)
//     {
//         EngSetLastError(ERROR_NO_SYSTEM_RESOURCES);
//         ret = 0;
//         goto Exit;
//     }
//
//     pSourceSurf = EngLockSurface((HSURF)hSourceBitmap);
//     if (!pSourceSurf)
//     {
//         ret = 0;
//         goto Exit;
//     }
//
//     /* HACK: If this is a RLE bitmap, only the relevant pixels must be set. */
//     if ((bmi->bmiHeader.biCompression == BI_RLE8) || (bmi->bmiHeader.biCompression == BI_RLE4))
//     {
//         hMaskBitmap = IntGdiCreateMaskFromRLE(bmi->bmiHeader.biWidth,
//             ScanLines,
//             bmi->bmiHeader.biCompression,
//             Bits,
//             cjMaxBits);
//         if (!hMaskBitmap)
//         {
//             EngSetLastError(ERROR_NO_SYSTEM_RESOURCES);
//             ret = 0;
//             goto Exit;
//         }
//         pMaskSurf = EngLockSurface((HSURF)hMaskBitmap);
//         if (!pMaskSurf)
//         {
//             ret = 0;
//             goto Exit;
//         }
//     }
//
//     /* Create a palette for the DIB */
//     ppalDIB = CreateDIBPalette(bmi, pDC, ColorUse);
//     if (!ppalDIB)
//     {
//         EngSetLastError(ERROR_NO_SYSTEM_RESOURCES);
//         ret = 0;
//         goto Exit;
//     }
//
//     /* This is actually a blit */
//     DC_vPrepareDCsForBlit(pDC, &rcDest, NULL, NULL);
//     pSurf = pDC->dclevel.pSurface;
//     if (!pSurf)
//     {
//         DC_vFinishBlit(pDC, NULL);
//         ret = ScanLines;
//         goto Exit;
//     }
//
//     ASSERT(pSurf->ppal);
//
//     /* Initialize EXLATEOBJ */
//     EXLATEOBJ_vInitialize(&exlo,
//                           ppalDIB,
//                           pSurf->ppal,
//                           RGB(0xff, 0xff, 0xff),
//                           pDC->pdcattr->crBackgroundClr,
//                           pDC->pdcattr->crForegroundClr);
//
//     pDestSurf = &pSurf->SurfObj;
//
//     /* Copy the bits */
//     DPRINT("BitsToDev with rcDest=(%d|%d) (%d|%d), ptSource=(%d|%d) w=%d h=%d\n",
//            rcDest.left, rcDest.top, rcDest.right, rcDest.bottom,
//            ptSource.x, ptSource.y, SourceSize.cx, SourceSize.cy);
//
//     /* This fixes the large Google text on Google.com from being upside down */
//     if (rcDest.top > rcDest.bottom)
//     {
//         RECTL_vMakeWellOrdered(&rcDest);
//         ptSource.y -= SourceSize.cy;
//     }
//
//     bResult = IntEngBitBlt(pDestSurf,
//                           pSourceSurf,
//                           pMaskSurf,
//                           (CLIPOBJ *)&pDC->co,
//                           &exlo.xlo,
//                           &rcDest,
//                           &ptSource,
//                           pMaskSurf ? &ptSource : NULL,
//                           NULL,
//                           NULL,
//                           pMaskSurf ? ROP4_MASK : ROP4_FROM_INDEX(R3_OPINDEX_SRCCOPY));
//
//     /* Cleanup EXLATEOBJ */
//     EXLATEOBJ_vCleanup(&exlo);
//
//     /* We're done */
//     DC_vFinishBlit(pDC, NULL);
//
//     ret = bResult ? ScanLines : 0;
//
// Exit:
//
//     if (ppalDIB) PALETTE_ShareUnlockPalette(ppalDIB);
//     if (pSourceSurf) EngUnlockSurface(pSourceSurf);
//     if (hSourceBitmap) EngDeleteSurface((HSURF)hSourceBitmap);
//     if (pMaskSurf) EngUnlockSurface(pMaskSurf);
//     if (hMaskBitmap) EngDeleteSurface((HSURF)hMaskBitmap);
//     if (pDC) DC_UnlockDc(pDC);
//     ExFreePoolWithTag(pbmiSafe, 'pmTG');
//
//     return ret;
// }
//
//
// /* Converts a device-dependent bitmap to a DIB */
// INT
// APIENTRY
// GreGetDIBitsInternal(
//     HDC hDC,
//     HBITMAP hBitmap,
//     UINT StartScan,
//     UINT ScanLines,
//     LPBYTE Bits,
//     LPBITMAPINFO Info,
//     UINT Usage,
//     UINT MaxBits,
//     UINT MaxInfo)
// {
//     BITMAPCOREINFO* pbmci = NULL;
//     PSURFACE psurf = NULL;
//     PDC pDC;
//     LONG width, height;
//     WORD planes, bpp;
//     DWORD compr, size ;
//     USHORT i;
//     int bitmap_type;
//     RGBQUAD* rgbQuads;
//     VOID* colorPtr;
//
//     DPRINT("Entered GreGetDIBitsInternal()\n");
//
//     if ((Usage && Usage != DIB_PAL_COLORS) || !Info || !hBitmap)
//         return 0;
//
//     pDC = DC_LockDc(hDC);
//     if (pDC == NULL || pDC->dctype == DCTYPE_INFO)
//     {
//         ScanLines = 0;
//         goto done;
//     }
//
//     /* Get a pointer to the source bitmap object */
//     psurf = SURFACE_ShareLockSurface(hBitmap);
//     if (psurf == NULL)
//     {
//         ScanLines = 0;
//         goto done;
//     }
//
//     colorPtr = (LPBYTE)Info + Info->bmiHeader.biSize;
//     rgbQuads = colorPtr;
//
//     bitmap_type = DIB_GetBitmapInfo(&Info->bmiHeader,
//                                     &width,
//                                     &height,
//                                     &planes,
//                                     &bpp,
//                                     &compr,
//                                     &size);
//     if(bitmap_type == -1)
//     {
//         DPRINT1("Wrong bitmap format\n");
//         EngSetLastError(ERROR_INVALID_PARAMETER);
//         ScanLines = 0;
//         goto done;
//     }
//     else if(bitmap_type == 0)
//     {
//         /* We need a BITMAPINFO to create a DIB, but we have to fill
//          * the BITMAPCOREINFO we're provided */
//         pbmci = (BITMAPCOREINFO*)Info;
//         /* fill in the the bit count, so we can calculate the right ColorsSize during the conversion */
//         pbmci->bmciHeader.bcBitCount = BitsPerFormat(psurf->SurfObj.iBitmapFormat);
//         Info = DIB_ConvertBitmapInfo((BITMAPINFO*)pbmci, Usage);
//         if(Info == NULL)
//         {
//             DPRINT1("Error, could not convert the BITMAPCOREINFO!\n");
//             ScanLines = 0;
//             goto done;
//         }
//         rgbQuads = Info->bmiColors;
//     }
//
//     /* Validate input:
//        - negative width is always an invalid value
//        - non-null Bits and zero bpp is an invalid combination
//        - only check the rest of the input params if either bpp is non-zero or Bits are set */
//     if (width < 0 || (bpp == 0 && Bits))
//     {
//         ScanLines = 0;
//         goto done;
//     }
//
//     if (Bits || bpp)
//     {
//         if ((height == 0 || width == 0) || (compr && compr != BI_BITFIELDS && compr != BI_RGB))
//         {
//             ScanLines = 0;
//             goto done;
//         }
//     }
//
//     Info->bmiHeader.biClrUsed = 0;
//     Info->bmiHeader.biClrImportant = 0;
//
//     /* Fill in the structure */
//     switch(bpp)
//     {
//     case 0: /* Only info */
//         Info->bmiHeader.biWidth = psurf->SurfObj.sizlBitmap.cx;
//         Info->bmiHeader.biHeight = (psurf->SurfObj.fjBitmap & BMF_TOPDOWN) ?
//                                    -psurf->SurfObj.sizlBitmap.cy :
//                                    psurf->SurfObj.sizlBitmap.cy;
//         Info->bmiHeader.biPlanes = 1;
//         Info->bmiHeader.biBitCount = BitsPerFormat(psurf->SurfObj.iBitmapFormat);
//         Info->bmiHeader.biSizeImage = DIB_GetDIBImageBytes( Info->bmiHeader.biWidth,
//                                       Info->bmiHeader.biHeight,
//                                       Info->bmiHeader.biBitCount);
//         Info->bmiHeader.biCompression = (Info->bmiHeader.biBitCount == 16 || Info->bmiHeader.biBitCount == 32) ?
//                                         BI_BITFIELDS : BI_RGB;
//         Info->bmiHeader.biXPelsPerMeter = 0;
//         Info->bmiHeader.biYPelsPerMeter = 0;
//
//         if (Info->bmiHeader.biBitCount <= 8 && Info->bmiHeader.biClrUsed == 0)
//             Info->bmiHeader.biClrUsed = 1 << Info->bmiHeader.biBitCount;
//
//         ScanLines = 1;
//         goto done;
//
//     case 1:
//     case 4:
//     case 8:
//         Info->bmiHeader.biClrUsed = 1 << bpp;
//
//         /* If the bitmap is a DIB section and has the same format as what
//          * is requested, go ahead! */
//         if((psurf->hSecure) &&
//                 (BitsPerFormat(psurf->SurfObj.iBitmapFormat) == bpp))
//         {
//             if(Usage == DIB_RGB_COLORS)
//             {
//                 ULONG colors = min(psurf->ppal->NumColors, 256);
//                 if(colors != 256) Info->bmiHeader.biClrUsed = colors;
//                 for(i = 0; i < colors; i++)
//                 {
//                     rgbQuads[i].rgbRed = psurf->ppal->IndexedColors[i].peRed;
//                     rgbQuads[i].rgbGreen = psurf->ppal->IndexedColors[i].peGreen;
//                     rgbQuads[i].rgbBlue = psurf->ppal->IndexedColors[i].peBlue;
//                     rgbQuads[i].rgbReserved = 0;
//                 }
//             }
//             else
//             {
//                 for(i = 0; i < 256; i++)
//                     ((WORD*)rgbQuads)[i] = i;
//             }
//         }
//         else
//         {
//             if(Usage == DIB_PAL_COLORS)
//             {
//                 for(i = 0; i < 256; i++)
//                 {
//                     ((WORD*)rgbQuads)[i] = i;
//                 }
//             }
//             else if(bpp > 1 && bpp == BitsPerFormat(psurf->SurfObj.iBitmapFormat))
//             {
//                 /* For color DDBs in native depth (mono DDBs always have
//                    a black/white palette):
//                    Generate the color map from the selected palette */
//                 PPALETTE pDcPal = PALETTE_ShareLockPalette(pDC->dclevel.hpal);
//                 if(!pDcPal)
//                 {
//                     ScanLines = 0 ;
//                     goto done ;
//                 }
//                 for (i = 0; i < pDcPal->NumColors; i++)
//                 {
//                     rgbQuads[i].rgbRed      = pDcPal->IndexedColors[i].peRed;
//                     rgbQuads[i].rgbGreen    = pDcPal->IndexedColors[i].peGreen;
//                     rgbQuads[i].rgbBlue     = pDcPal->IndexedColors[i].peBlue;
//                     rgbQuads[i].rgbReserved = 0;
//                 }
//                 PALETTE_ShareUnlockPalette(pDcPal);
//             }
//             else
//             {
//                 switch (bpp)
//                 {
//                 case 1:
//                     rgbQuads[0].rgbRed = rgbQuads[0].rgbGreen = rgbQuads[0].rgbBlue = 0;
//                     rgbQuads[0].rgbReserved = 0;
//                     rgbQuads[1].rgbRed = rgbQuads[1].rgbGreen = rgbQuads[1].rgbBlue = 0xff;
//                     rgbQuads[1].rgbReserved = 0;
//                     break;
//
//                 case 4:
//                     /* The EGA palette is the first and last 8 colours of the default palette
//                        with the innermost pair swapped */
//                     RtlCopyMemory(rgbQuads,     DefLogPaletteQuads,      7 * sizeof(RGBQUAD));
//                     RtlCopyMemory(rgbQuads + 7, DefLogPaletteQuads + 12, 1 * sizeof(RGBQUAD));
//                     RtlCopyMemory(rgbQuads + 8, DefLogPaletteQuads +  7, 1 * sizeof(RGBQUAD));
//                     RtlCopyMemory(rgbQuads + 9, DefLogPaletteQuads + 13, 7 * sizeof(RGBQUAD));
//                     break;
//
//                 case 8:
//                 {
//                     INT i;
//
//                     memcpy(rgbQuads, DefLogPaletteQuads, 10 * sizeof(RGBQUAD));
//                     memcpy(rgbQuads + 246, DefLogPaletteQuads + 10, 10 * sizeof(RGBQUAD));
//
//                     for (i = 10; i < 246; i++)
//                     {
//                         rgbQuads[i].rgbRed = (i & 0x07) << 5;
//                         rgbQuads[i].rgbGreen = (i & 0x38) << 2;
//                         rgbQuads[i].rgbBlue = i & 0xc0;
//                         rgbQuads[i].rgbReserved = 0;
//                     }
//                 }
//                 }
//             }
//         }
//         break;
//
//     case 15:
//         if (Info->bmiHeader.biCompression == BI_BITFIELDS)
//         {
//             ((PDWORD)Info->bmiColors)[0] = 0x7c00;
//             ((PDWORD)Info->bmiColors)[1] = 0x03e0;
//             ((PDWORD)Info->bmiColors)[2] = 0x001f;
//         }
//         break;
//
//     case 16:
//         if (Info->bmiHeader.biCompression == BI_BITFIELDS)
//         {
//             if (psurf->hSecure)
//             {
//                 ((PDWORD)Info->bmiColors)[0] = psurf->ppal->RedMask;
//                 ((PDWORD)Info->bmiColors)[1] = psurf->ppal->GreenMask;
//                 ((PDWORD)Info->bmiColors)[2] = psurf->ppal->BlueMask;
//             }
//             else
//             {
//                 ((PDWORD)Info->bmiColors)[0] = 0xf800;
//                 ((PDWORD)Info->bmiColors)[1] = 0x07e0;
//                 ((PDWORD)Info->bmiColors)[2] = 0x001f;
//             }
//         }
//         break;
//
//     case 24:
//     case 32:
//         if (Info->bmiHeader.biCompression == BI_BITFIELDS)
//         {
//             if (psurf->hSecure)
//             {
//                 ((PDWORD)Info->bmiColors)[0] = psurf->ppal->RedMask;
//                 ((PDWORD)Info->bmiColors)[1] = psurf->ppal->GreenMask;
//                 ((PDWORD)Info->bmiColors)[2] = psurf->ppal->BlueMask;
//             }
//             else
//             {
//                 ((PDWORD)Info->bmiColors)[0] = 0xff0000;
//                 ((PDWORD)Info->bmiColors)[1] = 0x00ff00;
//                 ((PDWORD)Info->bmiColors)[2] = 0x0000ff;
//             }
//         }
//         break;
//
//     default:
//         ScanLines = 0;
//         goto done;
//     }
//
//     Info->bmiHeader.biSizeImage = DIB_GetDIBImageBytes(width, height, bpp);
//     Info->bmiHeader.biPlanes = 1;
//
//     if(Bits && ScanLines)
//     {
//         /* Create a DIBSECTION, blt it, profit */
//         PVOID pDIBits ;
//         HBITMAP hBmpDest;
//         PSURFACE psurfDest;
//         EXLATEOBJ exlo;
//         RECT rcDest;
//         POINTL srcPoint;
//         BOOL ret ;
//         int newLines = -1;
//
//         if (StartScan >= abs(Info->bmiHeader.biHeight))
//         {
//             ScanLines = 1;
//             goto done;
//         }
//         else
//         {
//             ScanLines = min(ScanLines, abs(Info->bmiHeader.biHeight) - StartScan);
//         }
//
//         if (abs(Info->bmiHeader.biHeight) < psurf->SurfObj.sizlBitmap.cy)
//         {
//             StartScan += psurf->SurfObj.sizlBitmap.cy - abs(Info->bmiHeader.biHeight);
//         }
//         /* Fixup values */
//         Info->bmiHeader.biHeight = (height < 0) ?
//                                    -(LONG)ScanLines : ScanLines;
//         /* Create the DIB */
//         hBmpDest = DIB_CreateDIBSection(pDC, Info, Usage, &pDIBits, NULL, 0, 0);
//         /* Restore them */
//         Info->bmiHeader.biHeight = height;
//
//         if(!hBmpDest)
//         {
//             DPRINT1("Unable to create a DIB Section!\n");
//             EngSetLastError(ERROR_INVALID_PARAMETER);
//             ScanLines = 0;
//             goto done ;
//         }
//
//         psurfDest = SURFACE_ShareLockSurface(hBmpDest);
//
//         RECTL_vSetRect(&rcDest, 0, 0, Info->bmiHeader.biWidth, ScanLines);
//         Info->bmiHeader.biWidth = width;
//         srcPoint.x = 0;
//
//         if (abs(Info->bmiHeader.biHeight) <= psurf->SurfObj.sizlBitmap.cy)
//         {
//             srcPoint.y = psurf->SurfObj.sizlBitmap.cy - StartScan - ScanLines;
//         }
//         else
//         {
//             /*  Determine the actual number of lines copied from the  */
//             /*  original bitmap. It might be different from ScanLines. */
//             newLines = abs(Info->bmiHeader.biHeight) - psurf->SurfObj.sizlBitmap.cy;
//             newLines = min((int)(StartScan + ScanLines - newLines), psurf->SurfObj.sizlBitmap.cy);
//             if (newLines > 0)
//             {
//                 srcPoint.y = psurf->SurfObj.sizlBitmap.cy - newLines;
//                 if (StartScan > psurf->SurfObj.sizlBitmap.cy)
//                 {
//                     newLines -= (StartScan - psurf->SurfObj.sizlBitmap.cy);
//                 }
//             }
//             else
//             {
//                 newLines = 0;
//                 srcPoint.y = psurf->SurfObj.sizlBitmap.cy;
//             }
//         }
//
//         EXLATEOBJ_vInitialize(&exlo, psurf->ppal, psurfDest->ppal, 0xffffff, 0xffffff, 0);
//
//         ret = IntEngCopyBits(&psurfDest->SurfObj,
//                              &psurf->SurfObj,
//                              NULL,
//                              &exlo.xlo,
//                              &rcDest,
//                              &srcPoint);
//
//         SURFACE_ShareUnlockSurface(psurfDest);
//
//         if(!ret)
//             ScanLines = 0;
//         else
//         {
//             RtlCopyMemory(Bits, pDIBits, DIB_GetDIBImageBytes (width, ScanLines, bpp));
//         }
//         /* Update if line count has changed */
//         if (newLines != -1)
//         {
//             ScanLines = (UINT)newLines;
//         }
//         GreDeleteObject(hBmpDest as *mut core::ffi::c_void);
//         EXLATEOBJ_vCleanup(&exlo);
//     }
//     else
//     {
//         /* Signals success and not the actual number of scan lines*/
//         ScanLines = 1;
//     }
//
// done:
//
//     if (pbmci)
//         DIB_FreeConvertedBitmapInfo(Info, (BITMAPINFO*)pbmci, Usage);
//
//     if (psurf)
//         SURFACE_ShareUnlockSurface(psurf);
//
//     if (pDC)
//         DC_UnlockDc(pDC);
//
//     return ScanLines;
// }
// /* --- pretranslation split: the remaining function groups live in dibobj.split2..2.c and are #included below so this stays ONE translation unit (zero CMake/linkage change). Reassembled verbatim by cpp. --- */
// /* dibobj.split2.c — #include fragment of dibobj.c, carved for pretranslation so rustconv gets a small file. NOT a standalone TU; NOT in CMake; do not compile directly. The preprocessor reassembles the original dibobj.c TU verbatim. See docs/engineering/win32k-pretranslation-audit.md */
//
//
// _Success_(return!=0)
// __kernel_entry
// INT
// APIENTRY
// NtGdiGetDIBitsInternal(
//     _In_ HDC hdc,
//     _In_ HBITMAP hbm,
//     _In_ UINT iStartScan,
//     _In_ UINT cScans,
//     _Out_writes_bytes_opt_(cjMaxBits) LPBYTE pjBits,
//     _Inout_ LPBITMAPINFO pbmi,
//     _In_ UINT iUsage,
//     _In_ UINT cjMaxBits,
//     _In_ UINT cjMaxInfo)
// {
//     PBITMAPINFO pbmiSafe;
//     HANDLE hSecure = NULL;
//     INT iResult = 0;
//     UINT cjAlloc;
//
//     /* Check for bad iUsage */
//     if (iUsage > 2) return 0;
//
//     /* Check if the size of the bitmap info is large enough */
//     if (cjMaxInfo < sizeof(BITMAPCOREHEADER))
//     {
//         return 0;
//     }
//
//     /* Use maximum size */
//     cjMaxInfo = min(cjMaxInfo, sizeof(BITMAPV5HEADER) + 256 * sizeof(RGBQUAD));
//
//     // HACK: the underlying code sucks and doesn't care for the size, so we
//     // give it the maximum ever needed
//     cjAlloc = sizeof(BITMAPV5HEADER) + 256 * sizeof(RGBQUAD);
//
//     /* Allocate a buffer the bitmapinfo */
//     pbmiSafe = ExAllocatePoolWithTag(PagedPool, cjAlloc, 'imBG');
//     if (!pbmiSafe)
//     {
//         /* Fail */
//         return 0;
//     }
//
//     /* Use SEH */
//     {
//         NTSTATUS __SehStatus = SafeProbeForRead(pbmi, cjMaxInfo as usize, 1);
//         if (!NT_SUCCESS(__SehStatus))
//         {
//             goto cleanup;
//         }
//     }
//     /* Probe and copy the BITMAPINFO */
//
//     RtlCopyMemory(pbmiSafe, pbmi, cjMaxInfo);
//
//     /* Check if the header size is large enough */
//     if ((pbmiSafe->bmiHeader.biSize < sizeof(BITMAPCOREHEADER)) ||
//         (pbmiSafe->bmiHeader.biSize > cjMaxInfo))
//     {
//         goto cleanup;
//     }
//
//     /* Check if the caller provided bitmap bits */
//     if (pjBits)
//     {
//         /* Secure the user mode memory */
//         hSecure = EngSecureMem(pjBits, cjMaxBits);
//         if (!hSecure)
//         {
//             goto cleanup;
//         }
//     }
//
//     /* Now call the internal function */
//     iResult = GreGetDIBitsInternal(hdc,
//                                    hbm,
//                                    iStartScan,
//                                    cScans,
//                                    pjBits,
//                                    pbmiSafe,
//                                    iUsage,
//                                    cjMaxBits,
//                                    cjMaxInfo);
//
//     /* Check for success */
//     if (iResult)
//     {
//         /* Use SEH to copy back to user mode */
//         _SEH2_TRY
//         {
//             /* Copy the data back */
//             cjMaxInfo = min(cjMaxInfo, (UINT)DIB_BitmapInfoSize(pbmiSafe, (WORD)iUsage));
//             ProbeForWrite(pbmi, cjMaxInfo as usize, 1);
//             RtlCopyMemory(pbmi, pbmiSafe, cjMaxInfo);
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Ignore */
//             (VOID)0;
//         }
//         _SEH2_END;
//     }
//
// cleanup:
//     if (hSecure) EngUnsecureMem(hSecure);
//     ExFreePoolWithTag(pbmiSafe, 'imBG');
//
//     return iResult;
// }
//
// W32KAPI
// INT
// APIENTRY
// NtGdiStretchDIBitsInternal(
//     IN HDC hdc,
//     IN INT xDst,
//     IN INT yDst,
//     IN INT cxDst,
//     IN INT cyDst,
//     IN INT xSrc,
//     IN INT ySrc,
//     IN INT cxSrc,
//     IN INT cySrc,
//     IN OPTIONAL LPBYTE pjInit,
//     IN LPBITMAPINFO pbmi,
//     IN DWORD dwUsage,
//     IN DWORD dwRop, // MS ntgdi.h says dwRop4(?)
//     IN UINT cjMaxInfo,
//     IN UINT cjMaxBits,
//     IN HANDLE hcmXform)
// {
//     SIZEL sizel;
//     RECTL rcSrc, rcDst;
//     PDC pdc;
//     HBITMAP hbmTmp = 0;
//     PSURFACE psurfTmp = 0, psurfDst = 0;
//     PPALETTE ppalDIB = 0;
//     EXLATEOBJ exlo;
//     PBYTE pvBits;
//
//     LPBITMAPINFO pbmiSafe;
//     UINT cjAlloc;
//     HBITMAP hBitmap, hOldBitmap = NULL;
//     HDC hdcMem;
//     HPALETTE hPal = NULL;
//     ULONG BmpFormat = 0;
//     INT LinesCopied = 0;
//
//     /* Check for bad iUsage */
//     if (dwUsage > 2) return 0;
//
//     /* We must have LPBITMAPINFO */
//     if (!pbmi)
//     {
//         DPRINT1("Error, Invalid Parameter.\n");
//         EngSetLastError(ERROR_INVALID_PARAMETER);
//         return 0;
//     }
//
//     /* Check if the size of the bitmap info is large enough */
//     if (cjMaxInfo < sizeof(BITMAPCOREHEADER))
//     {
//         return 0;
//     }
//
//     /* Use maximum size */
//     cjMaxInfo = min(cjMaxInfo, sizeof(BITMAPV5HEADER) + 256 * sizeof(RGBQUAD));
//
//     // HACK: the underlying code sucks and doesn't care for the size, so we
//     // give it the maximum ever needed
//     cjAlloc = sizeof(BITMAPV5HEADER) + 256 * sizeof(RGBQUAD);
//
//     /* Allocate a buffer the bitmapinfo */
//     pbmiSafe = ExAllocatePoolWithTag(PagedPool, cjAlloc, 'imBG');
//     if (!pbmiSafe)
//     {
//         /* Fail */
//         return 0;
//     }
//
//     /* Use SEH */
//     {
//         NTSTATUS __SehStatus = SafeProbeForRead(pbmi, cjMaxInfo as usize, 1);
//         if (!NT_SUCCESS(__SehStatus))
//         {
//             ExFreePoolWithTag(pbmiSafe, 'imBG');
//             return 0;
//         }
//     }
//     /* Probe and copy the BITMAPINFO */
//
//     RtlCopyMemory(pbmiSafe, pbmi, cjMaxInfo);
//
//     /* Check if the header size is large enough */
//     if ((pbmiSafe->bmiHeader.biSize < sizeof(BITMAPCOREHEADER)) ||
//         (pbmiSafe->bmiHeader.biSize > cjMaxInfo))
//     {
//         ExFreePoolWithTag(pbmiSafe, 'imBG');
//         return 0;
//     }
//
//     if (!(pdc = DC_LockDc(hdc)))
//     {
//         EngSetLastError(ERROR_INVALID_HANDLE);
//         return 0;
//     }
//
//     /* Check for info / mem DC without surface */
//     if (!pdc->dclevel.pSurface)
//     {
//         DC_UnlockDc(pdc);
//         // CHECKME
//         return TRUE;
//     }
//
//     /* Transform dest size */
//     sizel.cx = cxDst;
//     sizel.cy = cyDst;
//     IntLPtoDP(pdc, (POINTL*)&sizel, 1);
//     DC_UnlockDc(pdc);
//
//     if (pjInit && (cjMaxBits > 0))
//     {
//         pvBits = ExAllocatePoolWithTag(PagedPool, cjMaxBits, TAG_DIB);
//         if (!pvBits)
//         {
//             return 0;
//         }
//
//         {
//             NTSTATUS __SehStatus = SafeProbeForRead(pjInit, cjMaxBits as usize, 1);
//             if (!NT_SUCCESS(__SehStatus))
//             {
//                 ExFreePoolWithTag(pvBits, TAG_DIB);
//                 return 0;
//             }
//         }
//         RtlCopyMemory(pvBits, pjInit, cjMaxBits);
//     }
//     else
//     {
//         pvBits = NULL;
//     }
//
//     /* Here we select between the dwRop with SRCCOPY or not. */
//     if (dwRop == SRCCOPY)
//     {
//         hdcMem = NtGdiCreateCompatibleDC(hdc);
//         if (hdcMem == NULL)
//         {
//             DPRINT1("NtGdiCreateCompatibleDC failed to create hdc.\n");
//             EngSetLastError(ERROR_NO_SYSTEM_RESOURCES);
//             return 0;
//         }
//
//         hBitmap = NtGdiCreateCompatibleBitmap(hdc,
//                                               abs(pbmiSafe->bmiHeader.biWidth),
//                                               abs(pbmiSafe->bmiHeader.biHeight));
//         if (hBitmap == NULL)
//         {
//             DPRINT1("NtGdiCreateCompatibleBitmap failed to create bitmap.\n");
//             DPRINT1("hdc : 0x%08x \n", hdc);
//             DPRINT1("width : 0x%08x \n", pbmiSafe->bmiHeader.biWidth);
//             DPRINT1("height : 0x%08x \n", pbmiSafe->bmiHeader.biHeight);
//             EngSetLastError(ERROR_NO_SYSTEM_RESOURCES);
//             return 0;
//         }
//
//         /* Select the bitmap into hdcMem, and save a handle to the old bitmap */
//         hOldBitmap = NtGdiSelectBitmap(hdcMem, hBitmap);
//
//         if (dwUsage == DIB_PAL_COLORS)
//         {
//             hPal = NtGdiGetDCObject(hdc, GDI_OBJECT_TYPE_PALETTE as i32);
//             hPal = GdiSelectPalette(hdcMem, hPal, FALSE);
//         }
//
//         pdc = DC_LockDc(hdcMem);
//         if (pdc != NULL)
//         {
//             IntSetDIBits(pdc, hBitmap, 0, abs(pbmiSafe->bmiHeader.biHeight), pvBits,
//                          cjMaxBits, pbmiSafe, dwUsage);
//             DC_UnlockDc(pdc);
//         }
//
//         /* Origin for DIBitmap may be bottom left (positive biHeight) or top
//            left (negative biHeight) */
//         if (cxSrc == cxDst && cySrc == cyDst)
//         {
//             NtGdiBitBlt(hdc, xDst, yDst, cxDst, cyDst,
//                         hdcMem, xSrc, abs(pbmiSafe->bmiHeader.biHeight) - cySrc - ySrc,
//                         dwRop, CLR_INVALID, 0);
//         }
//         else
//         {
//             NtGdiStretchBlt(hdc, xDst, yDst, cxDst, cyDst,
//                             hdcMem, xSrc, abs(pbmiSafe->bmiHeader.biHeight) - cySrc - ySrc,
//                             cxSrc, cySrc, dwRop, CLR_INVALID);
//         }
//
//         /* cleanup */
//         if (hPal)
//             GdiSelectPalette(hdcMem, hPal, FALSE);
//
//         if (hOldBitmap)
//             NtGdiSelectBitmap(hdcMem, hOldBitmap);
//
//         NtGdiDeleteObjectApp(hdcMem);
//         GreDeleteObject(hBitmap);
//
//     } /* End of dwRop == SRCCOPY */
//     else
//     { /* Start of dwRop != SRCCOPY */
//         /* FIXME: Locking twice is cheesy, coord tranlation in UM will fix it */
//         if (!(pdc = DC_LockDc(hdc)))
//         {
//             DPRINT1("Could not lock dc\n");
//             EngSetLastError(ERROR_INVALID_HANDLE);
//             goto cleanup;
//         }
//
//         /* Calculate source and destination rect */
//         rcSrc.left = xSrc;
//         rcSrc.top = ySrc;
//         rcSrc.right = xSrc + abs(cxSrc);
//         rcSrc.bottom = ySrc + abs(cySrc);
//         rcDst.left = xDst;
//         rcDst.top = yDst;
//         rcDst.right = rcDst.left + cxDst;
//         rcDst.bottom = rcDst.top + cyDst;
//         IntLPtoDP(pdc, (POINTL*)&rcDst, 2);
//         RECTL_vOffsetRect(&rcDst, pdc->ptlDCOrig.x, pdc->ptlDCOrig.y);
//
//         if (pdc->fs & (DC_ACCUM_APP|DC_ACCUM_WMGR))
//         {
//            IntUpdateBoundsRect(pdc, &rcDst);
//         }
//
//         BmpFormat = BitmapFormat(pbmiSafe->bmiHeader.biBitCount,
//                                  pbmiSafe->bmiHeader.biCompression);
//
//         hbmTmp = GreCreateBitmapEx(pbmiSafe->bmiHeader.biWidth,
//                                    abs(pbmiSafe->bmiHeader.biHeight),
//                                    0,
//                                    BmpFormat,
//                                    pbmiSafe->bmiHeader.biHeight < 0 ? BMF_TOPDOWN : 0,
//                                    cjMaxBits,
//                                    pvBits,
//                                    0);
//
//         if (!hbmTmp)
//         {
//             goto cleanup;
//         }
//
//         psurfTmp = SURFACE_ShareLockSurface(hbmTmp);
//         if (!psurfTmp)
//         {
//             goto cleanup;
//         }
//
//         /* Create a palette for the DIB */
//         ppalDIB = CreateDIBPalette(pbmiSafe, pdc, dwUsage);
//         if (!ppalDIB)
//         {
//             goto cleanup;
//         }
//
//         /* Prepare DC for blit */
//         DC_vPrepareDCsForBlit(pdc, &rcDst, NULL, NULL);
//
//         psurfDst = pdc->dclevel.pSurface;
//
//         /* Initialize XLATEOBJ */
//         EXLATEOBJ_vInitialize(&exlo,
//                               ppalDIB,
//                               psurfDst->ppal,
//                               RGB(0xff, 0xff, 0xff),
//                               pdc->pdcattr->crBackgroundClr,
//                               pdc->pdcattr->crForegroundClr);
//
//         /* Perform the stretch operation */
//         IntEngStretchBlt(&psurfDst->SurfObj,
//                          &psurfTmp->SurfObj,
//                          NULL,
//                          (CLIPOBJ *)&pdc->co,
//                          &exlo.xlo,
//                          &pdc->dclevel.ca,
//                          &rcDst,
//                          &rcSrc,
//                          NULL,
//                          &pdc->eboFill.BrushObject,
//                          NULL,
//                          WIN32_ROP3_TO_ENG_ROP4(dwRop));
//
//         /* Cleanup */
//         DC_vFinishBlit(pdc, NULL);
//         EXLATEOBJ_vCleanup(&exlo);
//
//     cleanup:
//         if (ppalDIB) PALETTE_ShareUnlockPalette(ppalDIB);
//         if (psurfTmp) SURFACE_ShareUnlockSurface(psurfTmp);
//         if (hbmTmp) GreDeleteObject(hbmTmp);
//         if (pdc) DC_UnlockDc(pdc);
//     }
//
//     if (pvBits) ExFreePoolWithTag(pvBits, TAG_DIB);
//
//     /* This is not what MSDN says is returned from this function, but it
//      * follows Wine's dlls/gdi32/dib.c function nulldrv_StretchDIBits
//      * and it fixes over 100 gdi32:dib regression tests. */
//     if (dwRop == SRCCOPY)
//     {
//         LinesCopied = abs(pbmiSafe->bmiHeader.biHeight);
//     }
//     else
//     {
//         LinesCopied = pbmiSafe->bmiHeader.biHeight;
//     }
//
//     ExFreePoolWithTag(pbmiSafe, 'imBG');
//
//     return LinesCopied;
// }
//
// HBITMAP
// FASTCALL
// IntCreateDIBitmap(
//     PDC Dc,
//     INT width,
//     INT height,
//     UINT planes,
//     UINT bpp,
//     ULONG compression,
//     DWORD init,
//     LPBYTE bits,
//     ULONG cjMaxBits,
//     PBITMAPINFO data,
//     DWORD coloruse)
// {
//     HBITMAP handle;
//     BOOL fColor;
//     ULONG BmpFormat = 0;
//
//     if (planes && bpp)
//         BmpFormat = BitmapFormat(planes * bpp, compression);
//
//     // Check if we should create a monochrome or color bitmap. We create a monochrome bitmap only if it has exactly 2
//     // colors, which are black followed by white, nothing else. In all other cases, we create a color bitmap.
//
//     if (BmpFormat != BMF_1BPP) fColor = TRUE;
//     else if ((coloruse > DIB_RGB_COLORS) || ((init & CBM_INIT) == 0) || !data) fColor = FALSE;
//     else
//     {
//         const RGBQUAD *rgb = (RGBQUAD*)((PBYTE)data + data->bmiHeader.biSize);
//         DWORD col = RGB(rgb->rgbRed, rgb->rgbGreen, rgb->rgbBlue);
//
//         // Check if the first color of the colormap is black
//         if (col == RGB(0, 0, 0))
//         {
//             rgb++;
//             col = RGB(rgb->rgbRed, rgb->rgbGreen, rgb->rgbBlue);
//
//             // If the second color is white, create a monochrome bitmap
//             fColor = (col != RGB(0xff,0xff,0xff));
//         }
//         else fColor = TRUE;
//     }
//
//     // Now create the bitmap
//     if (fColor)
//     {
//         if (init & CBM_CREATDIB)
//         {
//             PSURFACE Surface;
//             PPALETTE Palette;
//
//             /* Undocumented flag which creates a DDB of the format specified by the bitmap info. */
//             handle = IntCreateCompatibleBitmap(Dc, width, height, planes, bpp);
//             if (!handle)
//             {
//                 DPRINT1("IntCreateCompatibleBitmap() failed!\n");
//                 return NULL;
//             }
//
//             /* The palette must also match the given data */
//             Surface = SURFACE_ShareLockSurface(handle);
//             ASSERT(Surface);
//             Palette = CreateDIBPalette(data, Dc, coloruse);
//             if (Palette == NULL)
//             {
//                 SURFACE_ShareUnlockSurface(Surface);
//                 GreDeleteObject(handle);
//                 return NULL;
//             }
//
//             SURFACE_vSetPalette(Surface, Palette);
//
//             PALETTE_ShareUnlockPalette(Palette);
//             SURFACE_ShareUnlockSurface(Surface);
//         }
//         else
//         {
//             /* Create a regular compatible bitmap, in the same format as the device */
//             handle = IntCreateCompatibleBitmap(Dc, width, height, 0, 0);
//         }
//     }
//     else
//     {
//         handle = GreCreateBitmap(width,
//                                  abs(height),
//                                  1,
//                                  1,
//                                  NULL);
//     }
//
//     if (height < 0)
//         height = -height;
//
//     if ((NULL != handle) && (CBM_INIT & init))
//     {
//         IntSetDIBits(Dc, handle, 0, height, bits, cjMaxBits, data, coloruse);
//     }
//
//     return handle;
// }
//
// // The CreateDIBitmap function creates a device-dependent bitmap (DDB) from a DIB and, optionally, sets the bitmap bits
// // The DDB that is created will be whatever bit depth your reference DC is
// HBITMAP
// APIENTRY
// NtGdiCreateDIBitmapInternal(
//     IN HDC hDc,
//     IN INT cx,
//     IN INT cy,
//     IN DWORD fInit,
//     IN OPTIONAL LPBYTE pjInit,
//     IN OPTIONAL LPBITMAPINFO pbmi,
//     IN DWORD iUsage,
//     IN UINT cjMaxInitInfo,
//     IN UINT cjMaxBits,
//     IN FLONG fl,
//     IN HANDLE hcmXform)
// {
//     NTSTATUS Status = STATUS_SUCCESS;
//     PBYTE safeBits = NULL;
//     HBITMAP hbmResult = NULL;
//
//     if (pjInit == NULL)
//     {
//         fInit &= ~CBM_INIT;
//     }
//
//     if(pjInit && (fInit & CBM_INIT))
//     {
//         if (cjMaxBits == 0) return NULL;
//         safeBits = ExAllocatePoolWithTag(PagedPool, cjMaxBits, TAG_DIB);
//         if(!safeBits)
//         {
//             DPRINT1("Failed to allocate %lu bytes\n", cjMaxBits);
//             EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
//             return NULL;
//         }
//     }
//
//     if (pbmi)
//     {
//         Status = SafeProbeForRead(pbmi, cjMaxInitInfo as usize, 1);
//     }
//     if (NT_SUCCESS(Status) && pjInit && (fInit & CBM_INIT))
//     {
//         Status = SafeProbeForRead(pjInit, cjMaxBits as usize, 1);
//     }
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Got an exception! pjInit = %p\n", pjInit);
//         SetLastNtError(Status);
//         goto cleanup;
//     }
//     else
//     {
//         if(pjInit && (fInit & CBM_INIT))
//         {
//             RtlCopyMemory(safeBits, pjInit, cjMaxBits);
//         }
//     }
//
//     hbmResult =  GreCreateDIBitmapInternal(hDc,
//                                            cx,
//                                            cy,
//                                            fInit,
//                                            safeBits,
//                                            pbmi,
//                                            iUsage,
//                                            fl,
//                                            cjMaxBits,
//                                            hcmXform);
//
// cleanup:
//     if (safeBits) ExFreePoolWithTag(safeBits, TAG_DIB);
//     return hbmResult;
// }
//
//
// HBITMAP
// NTAPI
// GreCreateDIBitmapInternal(
//     IN HDC hDc,
//     IN INT cx,
//     IN INT cy,
//     IN DWORD fInit,
//     IN OPTIONAL LPBYTE pjInit,
//     IN OPTIONAL PBITMAPINFO pbmi,
//     IN DWORD iUsage,
//     IN FLONG fl,
//     IN UINT cjMaxBits,
//     IN HANDLE hcmXform)
// {
//     PDC Dc;
//     HBITMAP Bmp;
//     USHORT bpp, planes;
//     DWORD compression;
//     HDC hdcDest;
//
//     if (!hDc) /* 1bpp monochrome bitmap */
//     {
//         // Should use System Bitmap DC hSystemBM, with CreateCompatibleDC for this.
//         hdcDest = NtGdiCreateCompatibleDC(0);
//         if(!hdcDest)
//         {
//             DPRINT1("NtGdiCreateCompatibleDC failed\n");
//             return NULL;
//         }
//     }
//     else
//     {
//         hdcDest = hDc;
//     }
//
//     Dc = DC_LockDc(hdcDest);
//     if (!Dc)
//     {
//         DPRINT1("Failed to lock hdcDest %p\n", hdcDest);
//         EngSetLastError(ERROR_INVALID_HANDLE);
//         return NULL;
//     }
//     /* It's OK to set bpp=0 here, as IntCreateDIBitmap will create a compatible Bitmap
//      * if bpp != 1 and ignore the real value that was passed */
//     if (pbmi)
//     {
//         if (pbmi->bmiHeader.biSize == sizeof(BITMAPCOREHEADER))
//         {
//             BITMAPCOREHEADER* CoreHeader = (BITMAPCOREHEADER*)&pbmi->bmiHeader;
//             bpp = CoreHeader->bcBitCount;
//             planes = CoreHeader->bcPlanes ? CoreHeader->bcPlanes : 1;
//             compression = BI_RGB;
//         }
//         else
//         {
//             bpp = pbmi->bmiHeader.biBitCount;
//             planes = pbmi->bmiHeader.biPlanes ? pbmi->bmiHeader.biPlanes : 1;
//             compression = pbmi->bmiHeader.biCompression;
//         }
//     }
//     else
//     {
//         bpp = 0;
//         planes = 0;
//         compression = 0;
//     }
//     Bmp = IntCreateDIBitmap(Dc, cx, cy, planes, bpp, compression, fInit, pjInit, cjMaxBits, pbmi, iUsage);
//     DC_UnlockDc(Dc);
//
//     if(!hDc)
//     {
//         NtGdiDeleteObjectApp(hdcDest);
//     }
//     return Bmp;
// }
//
// HBITMAP
// NTAPI
// GreCreateDIBitmapFromPackedDIB(
//     _In_reads_(cjPackedDIB )PVOID pvPackedDIB,
//     _In_ UINT cjPackedDIB,
//     _In_ ULONG uUsage)
// {
//     PBITMAPINFO pbmi;
//     PBYTE pjBits;
//     UINT cjInfo, cjBits;
//     HBITMAP hbm;
//
//     /* We only support BITMAPINFOHEADER, make sure the size is ok */
//     if (cjPackedDIB < sizeof(BITMAPINFOHEADER))
//     {
//         return NULL;
//     }
//
//     /* The packed DIB starts with the BITMAPINFOHEADER */
//     pbmi = pvPackedDIB;
//
//     if (cjPackedDIB < pbmi->bmiHeader.biSize)
//     {
//         return NULL;
//     }
//
//     /* Calculate the info size and make sure the packed DIB is large enough */
//     cjInfo = DIB_BitmapInfoSize(pbmi, uUsage);
//     if (cjPackedDIB <= cjInfo)
//     {
//         return NULL;
//     }
//
//     /* The bitmap bits start after the header */
//     pjBits = (PBYTE)pvPackedDIB + cjInfo;
//     cjBits = cjPackedDIB - cjInfo;
//
//     hbm = GreCreateDIBitmapInternal(NULL,
//                                     pbmi->bmiHeader.biWidth,
//                                     abs(pbmi->bmiHeader.biHeight),
//                                     CBM_INIT | CBM_CREATDIB,
//                                     pjBits,
//                                     pbmi,
//                                     uUsage,
//                                     0,
//                                     cjBits,
//                                     NULL);
//
//     return hbm;
// }
//
// HBITMAP
// APIENTRY
// NtGdiCreateDIBSection(
//     IN HDC hDC,
//     IN OPTIONAL HANDLE hSection,
//     IN DWORD dwOffset,
//     IN BITMAPINFO* bmi,
//     IN DWORD Usage,
//     IN UINT cjHeader,
//     IN FLONG fl,
//     IN ULONG_PTR dwColorSpace,
//     OUT PVOID *Bits)
// {
//     HBITMAP hbitmap = 0;
//     DC *dc;
//     BOOL bDesktopDC = FALSE;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     if (!bmi) return hbitmap; // Make sure.
//
//     _SEH2_TRY
//     {
//         ProbeForRead(&bmi->bmiHeader.biSize, sizeof(DWORD), 1);
//         ProbeForRead(bmi, bmi->bmiHeader.biSize, 1);
//         ProbeForRead(bmi, DIB_BitmapInfoSize(bmi, (WORD)Usage), 1);
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = _SEH2_GetExceptionCode();
//     }
//     _SEH2_END;
//
//     if(!NT_SUCCESS(Status))
//     {
//         SetLastNtError(Status);
//         return NULL;
//     }
//
//     // If the reference hdc is null, take the desktop dc
//     if (hDC == 0)
//     {
//         hDC = NtGdiCreateCompatibleDC(0);
//         bDesktopDC = TRUE;
//     }
//
//     if ((dc = DC_LockDc(hDC)))
//     {
//         hbitmap = DIB_CreateDIBSection(dc,
//                                        bmi,
//                                        Usage,
//                                        Bits,
//                                        hSection,
//                                        dwOffset,
//                                        0);
//         DC_UnlockDc(dc);
//     }
//     else
//     {
//         EngSetLastError(ERROR_INVALID_HANDLE);
//     }
//
//     if (bDesktopDC)
//         NtGdiDeleteObjectApp(hDC as *mut core::ffi::c_void);
//
//     return hbitmap;
// }
//
// HBITMAP
// APIENTRY
// DIB_CreateDIBSection(
//     PDC dc,
//     CONST BITMAPINFO *bmi,
//     UINT usage,
//     LPVOID *bits,
//     HANDLE section,
//     DWORD offset,
//     DWORD ovr_pitch)
// {
//     HBITMAP res = 0;
//     SURFACE *bmp = NULL;
//     void *mapBits = NULL;
//     PPALETTE ppalDIB = NULL;
//
//     // Fill BITMAP32 structure with DIB data
//     CONST BITMAPINFOHEADER *bi = &bmi->bmiHeader;
//     INT effHeight;
//     ULONG totalSize;
//     BITMAP bm;
//     //SIZEL Size;
//     HANDLE hSecure;
//
//     DPRINT("format (%ld,%ld), planes %u, bpp %u, size %lu, colors %lu (%s)\n",
//            bi->biWidth, bi->biHeight, bi->biPlanes, bi->biBitCount,
//            bi->biSizeImage, bi->biClrUsed, usage == DIB_PAL_COLORS? "PAL" : "RGB");
//
//     /* CreateDIBSection should fail for compressed formats */
//     if (bi->biCompression == BI_RLE4 || bi->biCompression == BI_RLE8)
//     {
//         DPRINT1("no compressed format allowed\n");
//         return (HBITMAP)NULL;
//     }
//
//     effHeight = bi->biHeight >= 0 ? bi->biHeight : -bi->biHeight;
//     bm.bmType = 0;
//     bm.bmWidth = bi->biWidth;
//     bm.bmHeight = effHeight;
//     bm.bmWidthBytes = ovr_pitch ? ovr_pitch : WIDTH_BYTES_ALIGN32(bm.bmWidth, bi->biBitCount);
//
//     bm.bmPlanes = bi->biPlanes;
//     bm.bmBitsPixel = bi->biBitCount;
//     bm.bmBits = NULL;
//
//     // Get storage location for DIB bits.  Only use biSizeImage if it's valid and
//     // we're dealing with a compressed bitmap.  Otherwise, use width * height.
//     totalSize = (bi->biSizeImage && (bi->biCompression != BI_RGB) && (bi->biCompression != BI_BITFIELDS))
//                 ? bi->biSizeImage : (ULONG)(bm.bmWidthBytes * effHeight);
//
//     if (section)
//     {
//         SYSTEM_BASIC_INFORMATION Sbi;
//         NTSTATUS Status;
//         DWORD mapOffset;
//         LARGE_INTEGER SectionOffset;
//         SIZE_T mapSize;
//
//         Status = ZwQuerySystemInformation(SystemBasicInformation,
//                                           &Sbi,
//                                           sizeof Sbi,
//                                           0);
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT1("ZwQuerySystemInformation failed (0x%lx)\n", Status);
//             return NULL;
//         }
//
//         mapOffset = offset - (offset % Sbi.AllocationGranularity);
//         mapSize = totalSize + (offset - mapOffset);
//
//         SectionOffset.LowPart  = mapOffset;
//         SectionOffset.HighPart = 0;
//
//         Status = ZwMapViewOfSection(section,
//                                     NtCurrentProcess(),
//                                     &mapBits,
//                                     0,
//                                     0,
//                                     &SectionOffset,
//                                     &mapSize,
//                                     ViewShare,
//                                     0,
//                                     PAGE_READWRITE);
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT1("ZwMapViewOfSection failed (0x%lx)\n", Status);
//             EngSetLastError(ERROR_INVALID_PARAMETER);
//             return NULL;
//         }
//
//         if (mapBits) bm.bmBits = (char *)mapBits + (offset - mapOffset);
//     }
//     else if (ovr_pitch && offset)
//         bm.bmBits = UlongToPtr(offset);
//     else
//     {
//         offset = 0;
//         bm.bmBits = EngAllocUserMem(totalSize, 0);
//         if(!bm.bmBits)
//         {
//             DPRINT1("Failed to allocate memory\n");
//             goto cleanup;
//         }
//     }
//
// //  hSecure = MmSecureVirtualMemory(bm.bmBits, totalSize, PAGE_READWRITE);
//     hSecure = (HANDLE)0x1; // HACK OF UNIMPLEMENTED KERNEL STUFF !!!!
//
//     // Create Device Dependent Bitmap and add DIB pointer
//     //Size.cx = bm.bmWidth;
//     //Size.cy = abs(bm.bmHeight);
//     res = GreCreateBitmapEx(bm.bmWidth,
//                             abs(bm.bmHeight),
//                             bm.bmWidthBytes,
//                             BitmapFormat(bi->biBitCount * bi->biPlanes, bi->biCompression),
//                             BMF_DONTCACHE | BMF_USERMEM | BMF_NOZEROINIT |
//                             ((bi->biHeight < 0) ? BMF_TOPDOWN : 0),
//                             totalSize,
//                             bm.bmBits,
//                             0);
//     if (!res)
//     {
//         DPRINT1("GreCreateBitmapEx failed\n");
//         EngSetLastError(ERROR_NO_SYSTEM_RESOURCES);
//         goto cleanup;
//     }
//     bmp = SURFACE_ShareLockSurface(res); // HACK
//     if (NULL == bmp)
//     {
//         DPRINT1("SURFACE_LockSurface failed\n");
//         EngSetLastError(ERROR_INVALID_HANDLE);
//         goto cleanup;
//     }
//
//     /* WINE NOTE: WINE makes use of a colormap, which is a color translation
//                   table between the DIB and the X physical device. Obviously,
//                   this is left out of the ReactOS implementation. Instead,
//                   we call NtGdiSetDIBColorTable. */
//     bmp->hDIBSection = section;
//     bmp->hSecure = hSecure;
//     bmp->dwOffset = offset;
//     bmp->flags = API_BITMAP;
//     bmp->biClrImportant = bi->biClrImportant;
//
//     /* Create a palette for the DIB */
//     ppalDIB = CreateDIBPalette(bmi, dc, usage);
//
//     // Clean up in case of errors
// cleanup:
//     if (!res || !bmp || !bm.bmBits || !ppalDIB)
//     {
//         DPRINT("Got an error res=%p, bmp=%p, bm.bmBits=%p\n", res, bmp, bm.bmBits);
//         if (bm.bmBits)
//         {
//             // MmUnsecureVirtualMemory(hSecure); // FIXME: Implement this!
//             if (section)
//             {
//                 ZwUnmapViewOfSection(NtCurrentProcess(), mapBits);
//                 bm.bmBits = NULL;
//             }
//             else if (!offset)
//                 EngFreeUserMem(bm.bmBits), bm.bmBits = NULL;
//         }
//
//         if (bmp)
//         {
//             SURFACE_ShareUnlockSurface(bmp);
//             bmp = NULL;
//         }
//
//         if (res)
//         {
//             GreDeleteObject(res as *mut core::ffi::c_void);
//             res = 0;
//         }
//
//         if(ppalDIB)
//         {
//             PALETTE_ShareUnlockPalette(ppalDIB);
//         }
//     }
//
//     if (bmp)
//     {
//         /* If we're here, everything went fine */
//         SURFACE_vSetPalette(bmp, ppalDIB);
//         PALETTE_ShareUnlockPalette(ppalDIB);
//         SURFACE_ShareUnlockSurface(bmp);
//     }
//
//     // Return BITMAP handle and storage location
//     if (NULL != bm.bmBits && NULL != bits)
//     {
//         *bits = bm.bmBits;
//     }
//
//     return res;
// }
//
// /***********************************************************************
//  *           DIB_GetBitmapInfo
//  *
//  * Get the info from a bitmap header.
//  * Return 0 for COREHEADER, 1 for INFOHEADER, -1 for error.
//  */
// int
// FASTCALL
// DIB_GetBitmapInfo( const BITMAPINFOHEADER *header, LONG *width,
//                    LONG *height, WORD *planes, WORD *bpp, DWORD *compr, DWORD *size )
// {
//     if (header->biSize == sizeof(BITMAPCOREHEADER))
//     {
//         const BITMAPCOREHEADER *core = (const BITMAPCOREHEADER *)header;
//         *width  = core->bcWidth;
//         *height = core->bcHeight;
//         *planes = core->bcPlanes;
//         *bpp    = core->bcBitCount;
//         *compr  = BI_RGB;
//         *size   = 0;
//         return 0;
//     }
//     if (header->biSize >= sizeof(BITMAPINFOHEADER)) /* Assume BITMAPINFOHEADER */
//     {
//         *width  = header->biWidth;
//         *height = header->biHeight;
//         *planes = header->biPlanes;
//         *bpp    = header->biBitCount;
//         *compr  = header->biCompression;
//         *size   = header->biSizeImage;
//         return 1;
//     }
//     DPRINT1("(%u): unknown/wrong size for header\n", header->biSize );
//     return -1;
// }
//
// /***********************************************************************
//  *           DIB_GetDIBImageBytes
//  *
//  * Return the number of bytes used to hold the image in a DIB bitmap.
//  * 11/16/1999 (RJJ) lifted from wine
//  */
//
// INT APIENTRY DIB_GetDIBImageBytes(INT  width, INT height, INT depth)
// {
//     return WIDTH_BYTES_ALIGN32(width, depth) * (height < 0 ? -height : height);
// }
//
// /***********************************************************************
//  *           DIB_BitmapInfoSize
//  *
//  * Return the size of the bitmap info structure including color table.
//  * 11/16/1999 (RJJ) lifted from wine
//  */
//
// INT FASTCALL DIB_BitmapInfoSize(const BITMAPINFO * info, WORD coloruse)
// {
//     unsigned int colors, size, masks = 0;
//     unsigned int colorsize;
//
//     colorsize = (coloruse == DIB_RGB_COLORS) ? sizeof(RGBQUAD) :
//                 (coloruse == DIB_PAL_INDICES) ? 0 :
//                 sizeof(WORD);
//
//     if (info->bmiHeader.biSize == sizeof(BITMAPCOREHEADER))
//     {
//         const BITMAPCOREHEADER *core = (const BITMAPCOREHEADER *)info;
//         colors = (core->bcBitCount <= 8) ? 1 << core->bcBitCount : 0;
//         return sizeof(BITMAPCOREHEADER) + colors * colorsize;
//     }
//     else  /* Assume BITMAPINFOHEADER */
//     {
//         colors = info->bmiHeader.biClrUsed;
//         if (colors > 256) colors = 256;
//         if (!colors && (info->bmiHeader.biBitCount <= 8))
//             colors = 1 << info->bmiHeader.biBitCount;
//         if (info->bmiHeader.biCompression == BI_BITFIELDS) masks = 3;
//         size = max( info->bmiHeader.biSize, sizeof(BITMAPINFOHEADER) + masks * sizeof(DWORD) );
//         return size + colors * colorsize;
//     }
// }
//
// HPALETTE
// FASTCALL
// DIB_MapPaletteColors(PPALETTE ppalDc, CONST BITMAPINFO* lpbmi)
// {
//     PPALETTE ppalNew;
//     ULONG nNumColors,i;
//     USHORT *lpIndex;
//     HPALETTE hpal;
//
//     if (!(ppalDc->flFlags & PAL_INDEXED))
//     {
//         return NULL;
//     }
//
//     nNumColors = 1 << lpbmi->bmiHeader.biBitCount;
//     if (lpbmi->bmiHeader.biClrUsed)
//     {
//         nNumColors = min(nNumColors, lpbmi->bmiHeader.biClrUsed);
//     }
//
//     ppalNew = PALETTE_AllocPalWithHandle(PAL_INDEXED, nNumColors, NULL, 0, 0, 0);
//     if (ppalNew == NULL)
//     {
//         DPRINT1("Could not allocate palette\n");
//         return NULL;
//     }
//
//     lpIndex = (USHORT *)((PBYTE)lpbmi + lpbmi->bmiHeader.biSize);
//
//     for (i = 0; i < nNumColors; i++)
//     {
//         ULONG iColorIndex = *lpIndex % ppalDc->NumColors;
//         ppalNew->IndexedColors[i] = ppalDc->IndexedColors[iColorIndex];
//         lpIndex++;
//     }
//
//     hpal = ppalNew->BaseObject.hHmgr;
//     PALETTE_UnlockPalette(ppalNew);
//
//     return hpal;
// }
//
// /* Converts a BITMAPCOREINFO to a BITMAPINFO structure,
//  * or does nothing if it's already a BITMAPINFO (or V4 or V5) */
// BITMAPINFO*
// FASTCALL
// DIB_ConvertBitmapInfo (CONST BITMAPINFO* pbmi, DWORD Usage)
// {
//     CONST BITMAPCOREINFO* pbmci = (BITMAPCOREINFO*)pbmi;
//     BITMAPINFO* pNewBmi ;
//     UINT numColors = 0, ColorsSize = 0;
//
//     if(pbmi->bmiHeader.biSize >= sizeof(BITMAPINFOHEADER)) return (BITMAPINFO*)pbmi;
//     if(pbmi->bmiHeader.biSize != sizeof(BITMAPCOREHEADER)) return NULL;
//
//     if(pbmci->bmciHeader.bcBitCount <= 8)
//     {
//         numColors = 1 << pbmci->bmciHeader.bcBitCount;
//         if(Usage == DIB_PAL_COLORS)
//         {
//             ColorsSize = numColors * sizeof(WORD);
//         }
//         else
//         {
//             ColorsSize = numColors * sizeof(RGBQUAD);
//         }
//     }
//     else if (Usage == DIB_PAL_COLORS)
//     {
//         /* Invalid at high-res */
//         return NULL;
//     }
//
//     pNewBmi = ExAllocatePoolWithTag(PagedPool, sizeof(BITMAPINFOHEADER) + ColorsSize, TAG_DIB);
//     if(!pNewBmi) return NULL;
//
//     RtlZeroMemory(pNewBmi, sizeof(BITMAPINFOHEADER) + ColorsSize);
//
//     pNewBmi->bmiHeader.biSize = sizeof(BITMAPINFOHEADER);
//     pNewBmi->bmiHeader.biBitCount = pbmci->bmciHeader.bcBitCount;
//     pNewBmi->bmiHeader.biWidth = pbmci->bmciHeader.bcWidth;
//     pNewBmi->bmiHeader.biHeight = pbmci->bmciHeader.bcHeight;
//     pNewBmi->bmiHeader.biPlanes = pbmci->bmciHeader.bcPlanes;
//     pNewBmi->bmiHeader.biCompression = BI_RGB ;
//     pNewBmi->bmiHeader.biSizeImage = DIB_GetDIBImageBytes(pNewBmi->bmiHeader.biWidth,
//                                      pNewBmi->bmiHeader.biHeight,
//                                      pNewBmi->bmiHeader.biBitCount);
//     pNewBmi->bmiHeader.biClrUsed = numColors;
//
//     if(Usage == DIB_PAL_COLORS)
//     {
//         RtlCopyMemory(pNewBmi->bmiColors, pbmci->bmciColors, ColorsSize);
//     }
//     else
//     {
//         UINT i;
//         for(i=0; i<numColors; i++)
//         {
//             pNewBmi->bmiColors[i].rgbRed = pbmci->bmciColors[i].rgbtRed;
//             pNewBmi->bmiColors[i].rgbGreen = pbmci->bmciColors[i].rgbtGreen;
//             pNewBmi->bmiColors[i].rgbBlue = pbmci->bmciColors[i].rgbtBlue;
//         }
//     }
//
//     return pNewBmi ;
// }
//
// /* Frees a BITMAPINFO created with DIB_ConvertBitmapInfo */
// VOID
// FASTCALL
// DIB_FreeConvertedBitmapInfo(BITMAPINFO* converted, BITMAPINFO* orig, DWORD usage)
// {
//     BITMAPCOREINFO* pbmci;
//     if(converted == orig)
//         return;
//
//     if(usage == -1)
//     {
//         /* Caller don't want any conversion */
//         ExFreePoolWithTag(converted, TAG_DIB);
//         return;
//     }
//
//     /* Perform inverse conversion */
//     pbmci = (BITMAPCOREINFO*)orig;
//
//     ASSERT(pbmci->bmciHeader.bcSize == sizeof(BITMAPCOREHEADER));
//     pbmci->bmciHeader.bcBitCount = converted->bmiHeader.biBitCount;
//     pbmci->bmciHeader.bcWidth = converted->bmiHeader.biWidth;
//     pbmci->bmciHeader.bcHeight = converted->bmiHeader.biHeight;
//     pbmci->bmciHeader.bcPlanes = converted->bmiHeader.biPlanes;
//
//     if(pbmci->bmciHeader.bcBitCount <= 8)
//     {
//         UINT numColors = converted->bmiHeader.biClrUsed;
//         if(!numColors) numColors = 1 << pbmci->bmciHeader.bcBitCount;
//         if(usage == DIB_PAL_COLORS)
//         {
//             RtlZeroMemory(pbmci->bmciColors, (1 << pbmci->bmciHeader.bcBitCount) * sizeof(WORD));
//             RtlCopyMemory(pbmci->bmciColors, converted->bmiColors, numColors * sizeof(WORD));
//         }
//         else
//         {
//             UINT i;
//             RtlZeroMemory(pbmci->bmciColors, (1 << pbmci->bmciHeader.bcBitCount) * sizeof(RGBTRIPLE));
//             for(i=0; i<numColors; i++)
//             {
//                 pbmci->bmciColors[i].rgbtRed = converted->bmiColors[i].rgbRed;
//                 pbmci->bmciColors[i].rgbtGreen = converted->bmiColors[i].rgbGreen;
//                 pbmci->bmciColors[i].rgbtBlue = converted->bmiColors[i].rgbBlue;
//             }
//         }
//     }
//     /* Now free it, it's not needed anymore */
//     ExFreePoolWithTag(converted, TAG_DIB);
// }
//
// /* EOF */
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
