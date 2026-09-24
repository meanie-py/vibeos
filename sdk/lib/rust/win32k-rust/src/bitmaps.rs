use win32k_sys::win32k::*;

use core::ffi::c_void;

use crate::surface::gajBitsPerFormat;
use crate::dib_helpers::abs;

// surface.h macros / FORCEINLINE without exported symbols → reimplement inline.
// Generic over Into<i64> so the mixed LONG(i32)/ULONG(u32) call sites all typecheck.
#[inline(always)] fn BitsPerFormat<T: Into<i64>>(fmt: T) -> ULONG { gajBitsPerFormat[Into::<i64>::into(fmt) as usize] as ULONG }
#[inline(always)] fn WIDTH_BYTES_ALIGN16<A: Into<i64>, B: Into<i64>>(cx: A, bpp: B) -> ULONG { let cx: i64 = cx.into(); let bpp: i64 = bpp.into(); (((cx * bpp + 15) & !15) >> 3) as ULONG }
#[inline(always)] fn WIDTH_BYTES_ALIGN32<A: Into<i64>, B: Into<i64>>(cx: A, bpp: B) -> ULONG { let cx: i64 = cx.into(); let bpp: i64 = bpp.into(); (((cx * bpp + 31) & !31) >> 3) as ULONG }
#[inline(always)] fn labs(x: LONG) -> LONG { x.wrapping_abs() }
#[inline(always)] fn NT_SUCCESS(s: NTSTATUS) -> bool { s >= 0 }
#[inline(always)] fn min<T: Ord>(a: T, b: T) -> T { if a < b { a } else { b } }
// GDI handle macros (ntgdihdl.h)
#[inline(always)] fn GDI_HANDLE_GET_TYPE(h: HGDIOBJ) -> u32 { (h as usize as u32) & GDI_HANDLE_TYPE_MASK }
#[inline(always)] fn GDI_HANDLE_IS_STOCKOBJ(h: HGDIOBJ) -> BOOL { (((h as usize as u32) & 0x00800000) != 0) as BOOL }
#[inline(always)] unsafe fn RtlCopyMemory(d: *mut c_void, s: *const c_void, n: usize) { core::ptr::copy_nonoverlapping(s as *const u8, d as *mut u8, n); }
#[inline(always)] unsafe fn RtlZeroMemory(d: *mut c_void, n: usize) { core::ptr::write_bytes(d as *mut u8, 0, n); }
// SURFACE_*/PALETTE_* lock macros (surface.h/palette.h) → GDIOBJ primitives.
#[inline(always)] unsafe fn SURFACE_ShareLockSurface(h: HBITMAP) -> PSURFACE { GDIOBJ_ShareLockObj(h as HGDIOBJ, GDI_OBJECT_TYPE_BITMAP as DWORD) as PSURFACE }
#[inline(always)] unsafe fn SURFACE_ShareUnlockSurface(p: PSURFACE) { GDIOBJ_vDereferenceObject(p as POBJ); }
#[inline(always)] unsafe fn SURFACE_UnlockSurface(p: PSURFACE) { GDIOBJ_vUnlockObject(p as POBJ); }
#[inline(always)] unsafe fn PALETTE_ShareLockPalette(h: HGDIOBJ) -> PPALETTE { GDIOBJ_ShareLockObj(h, GDI_OBJECT_TYPE_PALETTE as DWORD) as PPALETTE }
#[inline(always)] unsafe fn PALETTE_ShareUnlockPalette(p: PPALETTE) { GDIOBJ_vDereferenceObject(core::ptr::addr_of_mut!((*p).BaseObject)); }

// BitmapFormat — real FASTCALL export (surface.rs). DecompressBitmap — cdecl VOID (rlecomp.c).
#[cfg(target_arch = "x86")] extern "fastcall" { fn BitmapFormat(cBits: ULONG, iCompression: ULONG) -> ULONG; }
#[cfg(not(target_arch = "x86"))] extern "C" { fn BitmapFormat(cBits: ULONG, iCompression: ULONG) -> ULONG; }
extern "C" {
    fn DecompressBitmap(sizl: SIZEL, pvCompressedBits: PVOID, pvBits: PVOID, lDelta: LONG, iFormat: ULONG, cjSizeImage: ULONG);
}
extern "system" {
    fn SafeProbeForWrite(a: *mut core::ffi::c_void, l: usize, al: u32) -> i32;
    fn SafeProbeForRead(a: *const core::ffi::c_void, l: usize, al: u32) -> i32;
}
// ProbeForRead/Write (Mm, NTAPI) — raise on bad user pointer (DUMMY_PSEH: validate-then-deref).
extern "system" {
    fn ProbeForRead(a: *const core::ffi::c_void, l: usize, al: u32);
    fn ProbeForWrite(a: *mut core::ffi::c_void, l: usize, al: u32);
}
// DIB_CreateDIBSection (dib.h) — HBITMAP APIENTRY; still-C dibobj.c symbol.
extern "system" {
    fn DIB_CreateDIBSection(dc: PDC, bmi: *const BITMAPINFO, usage: UINT, bits: *mut LPVOID, section: HANDLE, offset: DWORD, ovr_pitch: DWORD) -> HBITMAP;
}
// ---- file-scope preamble (translated) ----
//
// (NDEBUG is a debug-print suppression flag; no Rust equivalent needed at file scope.)
//
#[no_mangle]
pub unsafe extern "system" fn GreSetBitmapOwner(
    hbmp: HBITMAP,
    ulOwner: ULONG,
) -> BOOL {
    if GDI_HANDLE_GET_TYPE(hbmp as HGDIOBJ) != GDILoObjType_LO_BITMAP_TYPE as ULONG {
        return 0;
    }

    if (ulOwner == GDI_OBJ_HMGR_PUBLIC) || (ulOwner == GDI_OBJ_HMGR_NONE) {
        if GreGetObjectOwner(hbmp as HGDIOBJ) != GDI_OBJ_HMGR_POWNED {
            return 0;
        }
    }

    GreSetObjectOwner(hbmp as HGDIOBJ, ulOwner)
}

#[no_mangle]
pub unsafe extern "system" fn UnsafeSetBitmapBits(
    psurf: PSURFACE,
    mut cjBits: ULONG,
    pvBits: *const core::ffi::c_void,
) -> LONG {

    let nWidth: ULONG = (*psurf).SurfObj.sizlBitmap.cx as ULONG;
    let nHeight: ULONG = (*psurf).SurfObj.sizlBitmap.cy as ULONG;
    let cBitsPixel: ULONG = BitsPerFormat((*psurf).SurfObj.iBitmapFormat);

    let mut pjDst: PUCHAR = (*psurf).SurfObj.pvScan0 as PUCHAR;
    let mut pjSrc: *const UCHAR = pvBits as *const UCHAR;
    let lDeltaDst: LONG = (*psurf).SurfObj.lDelta;
    let lDeltaDstAbs: LONG = labs(lDeltaDst as core::ffi::c_long) as LONG;
    let lDeltaSrc: LONG = (((nWidth * cBitsPixel + 15) & !15) / 8) as LONG;

    let cbDst: ULONG = (lDeltaDstAbs as ULONG) * nHeight;
    let cbSrc: ULONG = (lDeltaSrc as ULONG) * nHeight;
    if cbSrc < cjBits {
        cjBits = cbSrc;
    }

    let mut iSrc: ULONG = 0;
    let mut iDst: ULONG = 0;
    let mut Y: ULONG = 0;
    while Y < nHeight {
        if iSrc + (lDeltaSrc as ULONG) > cjBits || iDst + (lDeltaDstAbs as ULONG) > cbDst {
            let a = cjBits - iSrc;
            let b = cbDst - iDst;
            let lDelta: LONG = if a < b { a as LONG } else { b as LONG };
            RtlCopyMemory(
                pjDst as *mut core::ffi::c_void,
                pjSrc as *const core::ffi::c_void,
                lDelta as usize,
            );
            iSrc += lDelta as ULONG;
            break;
        }

        RtlCopyMemory(
            pjDst as *mut core::ffi::c_void,
            pjSrc as *const core::ffi::c_void,
            lDeltaSrc as usize,
        );
        pjSrc = pjSrc.add(lDeltaSrc as usize);
        pjDst = pjDst.offset(lDeltaDst as isize);
        iSrc += lDeltaSrc as ULONG;
        iDst += lDeltaDstAbs as ULONG;

        Y += 1;
    }

    iSrc as LONG
}
#[no_mangle]
pub unsafe extern "system" fn GreCreateBitmapEx(
    nWidth: ULONG,
    nHeight: ULONG,
    cjWidthBytes: ULONG,
    mut iFormat: ULONG,
    fjBitmap: USHORT,
    cjSizeImage: ULONG,
    mut pvBits: PVOID,
    flags: FLONG,
) -> HBITMAP {
    let psurf: PSURFACE;
    let hbmp: HBITMAP;
    let mut pvCompressedBits: PVOID = core::ptr::null_mut();

    if iFormat < BMF_1BPP || iFormat > BMF_PNG {
        return core::ptr::null_mut();
    }

    if (iFormat == BMF_4RLE) || (iFormat == BMF_8RLE) {
        pvCompressedBits = pvBits;
        pvBits = core::ptr::null_mut();
        iFormat = if iFormat == BMF_4RLE { BMF_4BPP } else { BMF_8BPP };
    }

    psurf = SURFACE_AllocSurface(
        STYPE_BITMAP as USHORT,
        nWidth,
        nHeight,
        iFormat,
        fjBitmap as FLONG,
        cjWidthBytes,
        if !pvCompressedBits.is_null() { 0 } else { cjSizeImage },
        pvBits,
    );
    if psurf.is_null() {
        return core::ptr::null_mut();
    }

    if !pvCompressedBits.is_null() {
        let mut sizl: SIZEL = core::mem::zeroed();
        let lDelta: LONG;

        sizl.cx = nWidth as LONG;
        sizl.cy = nHeight as LONG;
        lDelta = WIDTH_BYTES_ALIGN32(nWidth, gajBitsPerFormat[iFormat as usize] as ULONG) as LONG;

        pvBits = (*psurf).SurfObj.pvBits;
        DecompressBitmap(sizl, pvCompressedBits, pvBits, lDelta, iFormat, cjSizeImage);
    }

    hbmp = (*psurf).SurfObj.hsurf as HBITMAP;

    (*psurf).flags |= flags | API_BITMAP as FLONG;

    SURFACE_UnlockSurface(psurf);
    hbmp
}
#[no_mangle]
pub unsafe extern "system" fn GreCreateBitmap(
    nWidth: ULONG,
    nHeight: ULONG,
    cPlanes: ULONG,
    cBitsPixel: ULONG,
    pvBits: PVOID,
) -> HBITMAP {
    GreCreateBitmapEx(
        nWidth,
        nHeight,
        0,
        BitmapFormat((cBitsPixel * cPlanes) as _, BI_RGB as _) as ULONG,
        0,
        0,
        pvBits,
        DDB_SURFACE as FLONG,
    )
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiCreateBitmap(
    nWidth: INT,
    nHeight: INT,
    cPlanes: UINT,
    cBitsPixel: UINT,
    pUnsafeBits: LPBYTE,
) -> HBITMAP {

    let iFormat: ULONG = BitmapFormat(cBitsPixel * cPlanes, BI_RGB);
    let cRealBpp: ULONG = gajBitsPerFormat[iFormat as usize] as ULONG;

    let cjWidthBytes: ULONG = WIDTH_BYTES_ALIGN16(nWidth, cRealBpp);
    let cjSize: ULONGLONG = cjWidthBytes as ULONGLONG * nHeight as ULONGLONG;

    if (iFormat == 0)
        || (nWidth <= 0)
        || (nWidth >= 0x8000000)
        || (nHeight <= 0)
        || (cBitsPixel > 32)
        || (cPlanes > 32)
        || (cjSize >= 0x100000000u64)
    {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return core::ptr::null_mut();
    }

    let psurf: PSURFACE = SURFACE_AllocSurface(
        STYPE_BITMAP as USHORT,
        nWidth as ULONG,
        nHeight as ULONG,
        iFormat,
        0,
        0,
        0,
        core::ptr::null_mut(),
    );
    if psurf.is_null() {
        return core::ptr::null_mut();
    }

    (*psurf).flags |= (API_BITMAP | DDB_SURFACE) as ULONG;

    if !pUnsafeBits.is_null() {
        // The C probes pUnsafeBits for READ here and, from the handler, deletes the
        // surface it just allocated and returns NULL. The probe was dropped, so
        // UnsafeSetBitmapBits copied cjSize bytes off the caller's raw pointer with
        // nothing to catch a fault — and cjSize is caller-influenced (width x height).
        if !NT_SUCCESS(SafeProbeForRead(
            pUnsafeBits as *const core::ffi::c_void,
            cjSize as usize,
            1,
        )) {
            GDIOBJ_vDeleteObject(core::ptr::addr_of_mut!((*psurf).BaseObject));
            return core::ptr::null_mut();
        }
        UnsafeSetBitmapBits(psurf, cjSize as ULONG, pUnsafeBits as *const core::ffi::c_void);
    } else {
        RtlZeroMemory(
            (*psurf).SurfObj.pvBits,
            (*psurf).SurfObj.cjBits as usize,
        );
    }

    let hbmp: HBITMAP = (*psurf).SurfObj.hsurf as HBITMAP;

    SURFACE_UnlockSurface(psurf);

    hbmp
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntCreateCompatibleBitmap(
    Dc: PDC,
    Width: INT,
    Height: INT,
    Planes: UINT,
    Bpp: UINT,
) -> HBITMAP {
    let mut Bmp: HBITMAP = core::ptr::null_mut();
    let ppal: PPALETTE;

    if Width == 0 || Height == 0 {
        return NtGdiGetStockObject(DEFAULT_BITMAP as INT) as HBITMAP;
    }

    if (*Dc).dctype != DCTYPE_MEMORY as u32 {
        let psurf: PSURFACE;

        Bmp = GreCreateBitmap(
            abs(Width) as ULONG,
            abs(Height) as ULONG,
            if Planes != 0 { Planes } else { 1 },
            if Bpp != 0 { Bpp } else { (*(*Dc).ppdev).gdiinfo.cBitsPixel },
            core::ptr::null_mut(),
        );
        if Bmp.is_null() {
            return core::ptr::null_mut();
        }

        psurf = SURFACE_ShareLockSurface(Bmp);

        ppal = PALETTE_ShareLockPalette((*(*Dc).ppdev).devinfo.hpalDefault as HGDIOBJ);
        SURFACE_vSetPalette(psurf, ppal);
        PALETTE_ShareUnlockPalette(ppal);

        (*psurf).flags = API_BITMAP;
        (*psurf).hdc = core::ptr::null_mut();
        (*psurf).SurfObj.hdev = (*Dc).ppdev as HDEV;
        SURFACE_ShareUnlockSurface(psurf);
    } else {
        let mut dibs: DIBSECTION = core::mem::zeroed();
        let Count: INT;
        let mut psurf: PSURFACE = (*Dc).dclevel.pSurface;
        if psurf.is_null() {
            psurf = psurfDefaultBitmap;
        }
        Count = BITMAP_GetObject(psurf, core::mem::size_of::<DIBSECTION>() as INT, core::ptr::addr_of_mut!(dibs) as LPVOID);

        if Count == core::mem::size_of::<BITMAP>() as INT {
            let psurfBmp: PSURFACE;

            Bmp = GreCreateBitmap(
                abs(Width) as ULONG,
                abs(Height) as ULONG,
                if Planes != 0 { Planes } else { 1 },
                if Bpp != 0 { Bpp } else { dibs.dsBm.bmBitsPixel as UINT },
                core::ptr::null_mut(),
            );
            if Bmp.is_null() {
                return core::ptr::null_mut();
            }
            psurfBmp = SURFACE_ShareLockSurface(Bmp);

            SURFACE_vSetPalette(psurfBmp, (*psurf).ppal);

            (*psurfBmp).flags = API_BITMAP;
            (*psurfBmp).hdc = core::ptr::null_mut();
            (*psurfBmp).SurfObj.hdev = (*Dc).ppdev as HDEV;
            SURFACE_ShareUnlockSurface(psurfBmp);
        } else if Count == core::mem::size_of::<DIBSECTION>() as INT {
            let mut buf: [BYTE; core::mem::size_of::<BITMAPINFOHEADER>() + 256 * core::mem::size_of::<RGBQUAD>()] =
                [0; core::mem::size_of::<BITMAPINFOHEADER>() + 256 * core::mem::size_of::<RGBQUAD>()];
            let mut Bits: PVOID = core::ptr::null_mut();
            let bi: *mut BITMAPINFO = buf.as_mut_ptr() as *mut BITMAPINFO;

            (*bi).bmiHeader.biSize = core::mem::size_of::<BITMAPINFOHEADER>() as DWORD;
            (*bi).bmiHeader.biWidth = Width as LONG;
            (*bi).bmiHeader.biHeight = Height as LONG;
            (*bi).bmiHeader.biPlanes = if Planes != 0 { Planes as WORD } else { dibs.dsBmih.biPlanes };
            (*bi).bmiHeader.biBitCount = if Bpp != 0 { Bpp as WORD } else { dibs.dsBmih.biBitCount };
            (*bi).bmiHeader.biCompression = dibs.dsBmih.biCompression;
            (*bi).bmiHeader.biSizeImage = 0;
            (*bi).bmiHeader.biXPelsPerMeter = dibs.dsBmih.biXPelsPerMeter;
            (*bi).bmiHeader.biYPelsPerMeter = dibs.dsBmih.biYPelsPerMeter;
            (*bi).bmiHeader.biClrUsed = dibs.dsBmih.biClrUsed;
            (*bi).bmiHeader.biClrImportant = dibs.dsBmih.biClrImportant;

            if (*bi).bmiHeader.biCompression == BI_BITFIELDS as DWORD {
                RtlCopyMemory(
                    core::ptr::addr_of_mut!((*bi).bmiColors) as PVOID,
                    core::ptr::addr_of!(dibs.dsBitfields) as *const _ as PVOID,
                    3 * core::mem::size_of::<RGBQUAD>(),
                );
            } else if (*bi).bmiHeader.biBitCount <= 8 {
                let mut Index: UINT;
                let PalGDI: PPALETTE;

                if (*psurf).ppal.is_null() {
                    EngSetLastError(ERROR_INVALID_HANDLE);
                    return core::ptr::null_mut();
                }

                PalGDI = (*psurf).ppal;

                Index = 0;
                while Index < 256 && Index < (*PalGDI).NumColors {
                    let colors = core::ptr::addr_of_mut!((*bi).bmiColors) as *mut RGBQUAD;
                    (*colors.add(Index as usize)).rgbRed = (*(*PalGDI).IndexedColors.add(Index as usize)).peRed;
                    (*colors.add(Index as usize)).rgbGreen = (*(*PalGDI).IndexedColors.add(Index as usize)).peGreen;
                    (*colors.add(Index as usize)).rgbBlue = (*(*PalGDI).IndexedColors.add(Index as usize)).peBlue;
                    (*colors.add(Index as usize)).rgbReserved = 0;
                    Index += 1;
                }
            }

            Bmp = DIB_CreateDIBSection(
                Dc,
                bi,
                DIB_RGB_COLORS,
                core::ptr::addr_of_mut!(Bits),
                core::ptr::null_mut(),
                0,
                0,
            );
            return Bmp;
        }
    }
    Bmp
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntCreateCompatibleBitmap(
    Dc: PDC,
    Width: INT,
    Height: INT,
    Planes: UINT,
    Bpp: UINT,
) -> HBITMAP {
    let mut Bmp: HBITMAP = core::ptr::null_mut();
    let ppal: PPALETTE;

    if Width == 0 || Height == 0 {
        return NtGdiGetStockObject(DEFAULT_BITMAP as INT) as HBITMAP;
    }

    if (*Dc).dctype != DCTYPE_MEMORY as u32 {
        let psurf: PSURFACE;

        Bmp = GreCreateBitmap(
            abs(Width) as ULONG,
            abs(Height) as ULONG,
            if Planes != 0 { Planes } else { 1 },
            if Bpp != 0 { Bpp } else { (*(*Dc).ppdev).gdiinfo.cBitsPixel },
            core::ptr::null_mut(),
        );
        if Bmp.is_null() {
            return core::ptr::null_mut();
        }

        psurf = SURFACE_ShareLockSurface(Bmp);

        ppal = PALETTE_ShareLockPalette((*(*Dc).ppdev).devinfo.hpalDefault as HGDIOBJ);
        SURFACE_vSetPalette(psurf, ppal);
        PALETTE_ShareUnlockPalette(ppal);

        (*psurf).flags = API_BITMAP;
        (*psurf).hdc = core::ptr::null_mut();
        (*psurf).SurfObj.hdev = (*Dc).ppdev as HDEV;
        SURFACE_ShareUnlockSurface(psurf);
    } else {
        let mut dibs: DIBSECTION = core::mem::zeroed();
        let Count: INT;
        let mut psurf: PSURFACE = (*Dc).dclevel.pSurface;
        if psurf.is_null() {
            psurf = psurfDefaultBitmap;
        }
        Count = BITMAP_GetObject(psurf, core::mem::size_of::<DIBSECTION>() as INT, core::ptr::addr_of_mut!(dibs) as LPVOID);

        if Count == core::mem::size_of::<BITMAP>() as INT {
            let psurfBmp: PSURFACE;

            Bmp = GreCreateBitmap(
                abs(Width) as ULONG,
                abs(Height) as ULONG,
                if Planes != 0 { Planes } else { 1 },
                if Bpp != 0 { Bpp } else { dibs.dsBm.bmBitsPixel as UINT },
                core::ptr::null_mut(),
            );
            if Bmp.is_null() {
                return core::ptr::null_mut();
            }
            psurfBmp = SURFACE_ShareLockSurface(Bmp);

            SURFACE_vSetPalette(psurfBmp, (*psurf).ppal);

            (*psurfBmp).flags = API_BITMAP;
            (*psurfBmp).hdc = core::ptr::null_mut();
            (*psurfBmp).SurfObj.hdev = (*Dc).ppdev as HDEV;
            SURFACE_ShareUnlockSurface(psurfBmp);
        } else if Count == core::mem::size_of::<DIBSECTION>() as INT {
            let mut buf: [BYTE; core::mem::size_of::<BITMAPINFOHEADER>() + 256 * core::mem::size_of::<RGBQUAD>()] =
                [0; core::mem::size_of::<BITMAPINFOHEADER>() + 256 * core::mem::size_of::<RGBQUAD>()];
            let mut Bits: PVOID = core::ptr::null_mut();
            let bi: *mut BITMAPINFO = buf.as_mut_ptr() as *mut BITMAPINFO;

            (*bi).bmiHeader.biSize = core::mem::size_of::<BITMAPINFOHEADER>() as DWORD;
            (*bi).bmiHeader.biWidth = Width as LONG;
            (*bi).bmiHeader.biHeight = Height as LONG;
            (*bi).bmiHeader.biPlanes = if Planes != 0 { Planes as WORD } else { dibs.dsBmih.biPlanes };
            (*bi).bmiHeader.biBitCount = if Bpp != 0 { Bpp as WORD } else { dibs.dsBmih.biBitCount };
            (*bi).bmiHeader.biCompression = dibs.dsBmih.biCompression;
            (*bi).bmiHeader.biSizeImage = 0;
            (*bi).bmiHeader.biXPelsPerMeter = dibs.dsBmih.biXPelsPerMeter;
            (*bi).bmiHeader.biYPelsPerMeter = dibs.dsBmih.biYPelsPerMeter;
            (*bi).bmiHeader.biClrUsed = dibs.dsBmih.biClrUsed;
            (*bi).bmiHeader.biClrImportant = dibs.dsBmih.biClrImportant;

            if (*bi).bmiHeader.biCompression == BI_BITFIELDS as DWORD {
                RtlCopyMemory(
                    core::ptr::addr_of_mut!((*bi).bmiColors) as PVOID,
                    core::ptr::addr_of!(dibs.dsBitfields) as *const _ as PVOID,
                    3 * core::mem::size_of::<RGBQUAD>(),
                );
            } else if (*bi).bmiHeader.biBitCount <= 8 {
                let mut Index: UINT;
                let PalGDI: PPALETTE;

                if (*psurf).ppal.is_null() {
                    EngSetLastError(ERROR_INVALID_HANDLE);
                    return core::ptr::null_mut();
                }

                PalGDI = (*psurf).ppal;

                Index = 0;
                while Index < 256 && Index < (*PalGDI).NumColors {
                    let colors = core::ptr::addr_of_mut!((*bi).bmiColors) as *mut RGBQUAD;
                    (*colors.add(Index as usize)).rgbRed = (*(*PalGDI).IndexedColors.add(Index as usize)).peRed;
                    (*colors.add(Index as usize)).rgbGreen = (*(*PalGDI).IndexedColors.add(Index as usize)).peGreen;
                    (*colors.add(Index as usize)).rgbBlue = (*(*PalGDI).IndexedColors.add(Index as usize)).peBlue;
                    (*colors.add(Index as usize)).rgbReserved = 0;
                    Index += 1;
                }
            }

            Bmp = DIB_CreateDIBSection(
                Dc,
                bi,
                DIB_RGB_COLORS,
                core::ptr::addr_of_mut!(Bits),
                core::ptr::null_mut(),
                0,
                0,
            );
            return Bmp;
        }
    }
    Bmp
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiCreateCompatibleBitmap(
    hDC: HDC,
    Width: INT,
    Height: INT,
) -> HBITMAP {
    let Bmp: HBITMAP;
    let Dc: PDC;

    if (Width <= 0) || (Height <= 0) || ((Width * Height) > 0x3FFFFFFF) {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return core::ptr::null_mut();
    }

    if hDC.is_null() {
        return GreCreateBitmap(Width as ULONG, Height as ULONG, 1, 1, core::ptr::null_mut());
    }

    Dc = DC_LockDc(hDC);

    if Dc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return core::ptr::null_mut();
    }

    Bmp = IntCreateCompatibleBitmap(Dc, Width, Height, 0, 0);

    DC_UnlockDc(Dc);
    Bmp
}
#[no_mangle]
pub unsafe extern "system" fn GreGetBitmapDimension(
    hBitmap: HBITMAP,
    psizDim: LPSIZE,
) -> BOOL {

    if hBitmap == core::ptr::null_mut() {
        return 0;
    }

    let psurfBmp = SURFACE_ShareLockSurface(hBitmap);
    if psurfBmp == core::ptr::null_mut() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }

    *psizDim = (*psurfBmp).sizlDim;

    SURFACE_ShareUnlockSurface(psurfBmp);

    1
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiGetBitmapDimension(
    hBitmap: HBITMAP,
    psizDim: LPSIZE,
) -> BOOL {

    let mut dim: SIZE = core::mem::zeroed();

    if GreGetBitmapDimension(hBitmap, &mut dim) == 0 {
        return 0;
    }

    let __SehStatus = SafeProbeForWrite(
        psizDim as *mut core::ffi::c_void,
        core::mem::size_of::<SIZE>(),
        1,
    );
    if !NT_SUCCESS(__SehStatus) {
        return 0;
    }

    // Probe is validate-only (USE_DUMMY_PSEH); the C wrapped probe AND access
    // in one _SEH2_TRY, so the access itself must be the fault-tolerant copy.
    if !crate::copy::publish_to_user(
        psizDim as *mut core::ffi::c_void,
        core::ptr::addr_of!(dim) as *const core::ffi::c_void,
        core::mem::size_of::<SIZE>(),
    ) {
        return 0;
    }

    1
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn UnsafeGetBitmapBits(psurf: PSURFACE, mut Bytes: u32, pvBits: PBYTE) -> i32 {
    UnsafeGetBitmapBits_impl(psurf, Bytes, pvBits)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn UnsafeGetBitmapBits(psurf: PSURFACE, Bytes: u32, pvBits: PBYTE) -> i32 {
    UnsafeGetBitmapBits_impl(psurf, Bytes, pvBits)
}

unsafe fn UnsafeGetBitmapBits_impl(psurf: PSURFACE, mut Bytes: u32, pvBits: PBYTE) -> i32 {

    let nWidth: u32 = (*psurf).SurfObj.sizlBitmap.cx as u32;
    let nHeight: u32 = (*psurf).SurfObj.sizlBitmap.cy as u32;
    let cBitsPixel: u32 = BitsPerFormat((*psurf).SurfObj.iBitmapFormat);

    let mut pjSrc: *mut u8 = (*psurf).SurfObj.pvScan0 as *mut u8;
    let mut pjDst: *mut u8 = pvBits as *mut u8;
    let lDeltaSrc: i32 = (*psurf).SurfObj.lDelta;
    let lDeltaSrcAbs: i32 = if lDeltaSrc < 0 { -lDeltaSrc } else { lDeltaSrc };
    let lDeltaDst: i32 = (((nWidth * cBitsPixel + 15) & !15) / 8) as i32;

    let cbSrc: u32 = nHeight * (lDeltaSrcAbs as u32);
    let cbDst: u32 = nHeight * (lDeltaDst as u32);
    Bytes = if Bytes < cbDst { Bytes } else { cbDst };

    let mut iSrc: u32 = 0;
    let mut iDst: u32 = 0;
    let mut Y: u32 = 0;
    while Y < nHeight {
        if iSrc + (lDeltaSrcAbs as u32) > cbSrc || iDst + (lDeltaDst as u32) > Bytes {
            let a = cbSrc - iSrc;
            let b = Bytes - iDst;
            let lDelta: i32 = (if a < b { a } else { b }) as i32;
            RtlCopyMemory(pjDst as *mut _, pjSrc as *const _, lDelta as usize);
            iDst += lDelta as u32;
            break;
        }

        RtlCopyMemory(pjDst as *mut _, pjSrc as *const _, lDeltaDst as usize);
        pjSrc = pjSrc.offset(lDeltaSrc as isize);
        pjDst = pjDst.offset(lDeltaDst as isize);
        iSrc += lDeltaSrcAbs as u32;
        iDst += lDeltaDst as u32;
        Y += 1;
    }

    iDst as i32
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiGetBitmapBits(
    hBitmap: HBITMAP,
    mut cjBuffer: u32,
    pUnsafeBits: PBYTE,
) -> i32 {

    let psurf: PSURFACE;
    let cjSize: u32;
    let ret: i32;

    if !pUnsafeBits.is_null() && cjBuffer == 0 {
        return 0;
    }

    psurf = SURFACE_ShareLockSurface(hBitmap);
    if psurf.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }

    cjSize = WIDTH_BYTES_ALIGN16(
        (*psurf).SurfObj.sizlBitmap.cx,
        BitsPerFormat((*psurf).SurfObj.iBitmapFormat),
    ) * (*psurf).SurfObj.sizlBitmap.cy as u32;

    if pUnsafeBits.is_null() {
        SURFACE_ShareUnlockSurface(psurf);
        return cjSize as i32;
    }

    if cjSize < cjBuffer {
        cjBuffer = cjSize;
    }

    ProbeForWrite(pUnsafeBits as *mut core::ffi::c_void, cjBuffer as usize, 1);
    ret = UnsafeGetBitmapBits(psurf, cjBuffer, pUnsafeBits);

    SURFACE_ShareUnlockSurface(psurf);

    ret
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiSetBitmapBits(
    hBitmap: HBITMAP,
    Bytes: DWORD,
    pUnsafeBits: PBYTE,
) -> LONG {

    let ret: LONG;
    let psurf: PSURFACE;

    if pUnsafeBits.is_null() || Bytes == 0 {
        return 0;
    }

    if GDI_HANDLE_IS_STOCKOBJ(hBitmap as HGDIOBJ) != 0 {
        return 0;
    }

    psurf = SURFACE_ShareLockSurface(hBitmap);
    if psurf.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }

    if ((*psurf).flags & API_BITMAP) == 0
        || (*psurf).SurfObj.iBitmapFormat > BMF_32BPP
    {
        EngSetLastError(ERROR_INVALID_HANDLE);
        SURFACE_ShareUnlockSurface(psurf);
        return 0;
    }

    // The C probes pUnsafeBits inside a _SEH2_TRY and returns 0 from the handler. It
    // probes for WRITE even though this call only READS the buffer — upstream's choice,
    // kept because ProbeForWrite is the stricter of the two and matching the C is what
    // makes the oracle meaningful. Dropped in translation, so UnsafeSetBitmapBits read
    // `Bytes` (a caller-supplied DWORD) off the caller's raw pointer unchecked.
    if !NT_SUCCESS(SafeProbeForWrite(
        pUnsafeBits as *mut core::ffi::c_void,
        Bytes as usize,
        1,
    )) {
        SURFACE_ShareUnlockSurface(psurf);
        return 0;
    }

    ret = UnsafeSetBitmapBits(psurf, Bytes, pUnsafeBits as *const core::ffi::c_void);

    SURFACE_ShareUnlockSurface(psurf);

    ret
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiSetBitmapDimension(
    hBitmap: HBITMAP,
    Width: i32,
    Height: i32,
    Size: LPSIZE,
) -> u32 {

    let psurf: PSURFACE;
    let mut Ret: u32 = 1;
    let Status: i32;

    if hBitmap.is_null() {
        return 0;
    }

    psurf = SURFACE_ShareLockSurface(hBitmap);
    if psurf.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }

    if !Size.is_null() {
        Status = SafeProbeForWrite(Size as *mut core::ffi::c_void, core::mem::size_of::<SIZE>(), 1);
        if !NT_SUCCESS(Status) {
            Ret = 0;
        } else if !crate::copy::publish_to_user(
            Size as *mut core::ffi::c_void,
            core::ptr::addr_of!((*psurf).sizlDim) as *const core::ffi::c_void,
            core::mem::size_of::<SIZE>(),
        ) {
            // Probe is validate-only; the C had probe AND store in one _SEH2_TRY.
            Ret = 0;
        }
    }

    (*psurf).sizlDim.cx = Width;
    (*psurf).sizlDim.cy = Height;

    SURFACE_ShareUnlockSurface(psurf);

    Ret
}

#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe extern "fastcall" fn BITMAP_CopyBitmap(hBitmap: HBITMAP) -> HBITMAP {
    BITMAP_CopyBitmap_impl(hBitmap)
}

#[no_mangle]
#[cfg(not(target_arch = "x86"))]
pub unsafe extern "C" fn BITMAP_CopyBitmap(hBitmap: HBITMAP) -> HBITMAP {
    BITMAP_CopyBitmap_impl(hBitmap)
}

unsafe fn BITMAP_CopyBitmap_impl(hBitmap: HBITMAP) -> HBITMAP {

    let mut hbmNew: HBITMAP;
    let psurfSrc: *mut SURFACE;
    let psurfNew: *mut SURFACE;

    if hBitmap.is_null() {
        return core::ptr::null_mut();
    }

    psurfSrc = SURFACE_ShareLockSurface(hBitmap);
    if psurfSrc.is_null() {
        return core::ptr::null_mut();
    }

    hbmNew = GreCreateBitmapEx(
        (*psurfSrc).SurfObj.sizlBitmap.cx as ULONG,
        (*psurfSrc).SurfObj.sizlBitmap.cy as ULONG,
        abs((*psurfSrc).SurfObj.lDelta) as ULONG,
        (*psurfSrc).SurfObj.iBitmapFormat,
        ((*psurfSrc).SurfObj.fjBitmap as u32 & BMF_TOPDOWN) as USHORT,
        (*psurfSrc).SurfObj.cjBits,
        core::ptr::null_mut(),
        (*psurfSrc).flags,
    );

    if !hbmNew.is_null() {
        psurfNew = SURFACE_ShareLockSurface(hbmNew);
        if !psurfNew.is_null() {
            RtlCopyMemory(
                (*psurfNew).SurfObj.pvBits,
                (*psurfSrc).SurfObj.pvBits,
                (*psurfNew).SurfObj.cjBits as usize,
            );

            SURFACE_vSetPalette(psurfNew, (*psurfSrc).ppal);

            SURFACE_ShareUnlockSurface(psurfNew);
        } else {
            GreDeleteObject(hbmNew as _);
            hbmNew = core::ptr::null_mut();
        }
    }

    SURFACE_ShareUnlockSurface(psurfSrc);
    hbmNew
}
#[no_mangle]
pub unsafe extern "system" fn BITMAP_GetObject(
    psurf: *mut SURFACE,
    Count: i32,
    buffer: LPVOID,
) -> i32 {
    let pBitmap: PBITMAP;

    if buffer.is_null() {
        return core::mem::size_of::<BITMAP>() as i32;
    }
    if (Count as u32) < core::mem::size_of::<BITMAP>() as u32 {
        return 0;
    }

    pBitmap = buffer as PBITMAP;
    (*pBitmap).bmType = 0;
    (*pBitmap).bmWidth = (*psurf).SurfObj.sizlBitmap.cx;
    (*pBitmap).bmHeight = (*psurf).SurfObj.sizlBitmap.cy;
    (*pBitmap).bmPlanes = 1;
    (*pBitmap).bmBitsPixel = BitsPerFormat((*psurf).SurfObj.iBitmapFormat) as u16;
    (*pBitmap).bmWidthBytes = WIDTH_BYTES_ALIGN16((*pBitmap).bmWidth, (*pBitmap).bmBitsPixel as i32) as i32;

    if !(*psurf).hSecure.is_null() {
        (*pBitmap).bmBits = (*psurf).SurfObj.pvBits;
        (*pBitmap).bmWidthBytes = WIDTH_BYTES_ALIGN32((*pBitmap).bmWidth, (*pBitmap).bmBitsPixel as i32) as i32;

        if Count >= core::mem::size_of::<DIBSECTION>() as i32 {
            let pds: PDIBSECTION = buffer as PDIBSECTION;

            (*pds).dsBmih.biSize = core::mem::size_of::<BITMAPINFOHEADER>() as u32;
            (*pds).dsBmih.biWidth = (*pds).dsBm.bmWidth;
            (*pds).dsBmih.biHeight = (*pds).dsBm.bmHeight;
            (*pds).dsBmih.biPlanes = (*pds).dsBm.bmPlanes;
            (*pds).dsBmih.biBitCount = (*pds).dsBm.bmBitsPixel;

            match (*psurf).SurfObj.iBitmapFormat {
                x if x == BMF_1BPP as _ || x == BMF_4BPP as _ || x == BMF_8BPP as _ => {
                    (*pds).dsBmih.biCompression = BI_RGB;
                }
                x if x == BMF_16BPP as _ => {
                    if ((*(*psurf).ppal).flFlags & PAL_RGB16_555) != 0 {
                        (*pds).dsBmih.biCompression = BI_RGB;
                    } else {
                        (*pds).dsBmih.biCompression = BI_BITFIELDS;
                    }
                }
                x if x == BMF_24BPP as _ || x == BMF_32BPP as _ => {
                    if ((*(*psurf).ppal).flFlags & PAL_BGR) != 0 {
                        (*pds).dsBmih.biCompression = BI_RGB;
                    } else {
                        (*pds).dsBmih.biCompression = BI_BITFIELDS;
                    }
                }
                x if x == BMF_4RLE as _ => {
                    (*pds).dsBmih.biCompression = BI_RLE4;
                }
                x if x == BMF_8RLE as _ => {
                    (*pds).dsBmih.biCompression = BI_RLE8;
                }
                x if x == BMF_JPEG as _ => {
                    (*pds).dsBmih.biCompression = BI_JPEG;
                }
                x if x == BMF_PNG as _ => {
                    (*pds).dsBmih.biCompression = BI_PNG;
                }
                _ => {}
            }

            (*pds).dsBmih.biSizeImage = (*psurf).SurfObj.cjBits;
            (*pds).dsBmih.biXPelsPerMeter = 0;
            (*pds).dsBmih.biYPelsPerMeter = 0;
            (*pds).dsBmih.biClrUsed = (*(*psurf).ppal).NumColors;
            (*pds).dsBmih.biClrImportant = (*psurf).biClrImportant;
            (*pds).dsBitfields[0] = (*(*psurf).ppal).RedMask;
            (*pds).dsBitfields[1] = (*(*psurf).ppal).GreenMask;
            (*pds).dsBitfields[2] = (*(*psurf).ppal).BlueMask;
            (*pds).dshSection = (*psurf).hDIBSection;
            (*pds).dsOffset = (*psurf).dwOffset;

            return core::mem::size_of::<DIBSECTION>() as i32;
        }
    } else {
        (*pBitmap).bmBits = core::ptr::null_mut();
    }

    core::mem::size_of::<BITMAP>() as i32
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiGetDCforBitmap(hsurf: HBITMAP) -> HDC {
    let mut hdc: HDC = core::ptr::null_mut();
    let psurf: PSURFACE = SURFACE_ShareLockSurface(hsurf);
    if !psurf.is_null() {
        hdc = (*psurf).hdc;
        SURFACE_ShareUnlockSurface(psurf);
    }
    hdc
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:        GNU GPL, See COPYING in the top level directory
//  * PROJECT:          ReactOS kernel
//  * PURPOSE:          Bitmap functions
//  * FILE:             win32ss/gdi/ntgdi/bitmaps.c
//  * PROGRAMERS:       Timo Kreuzer <timo.kreuzer@reactos.org>
//  *                   Katayama Hirofumi MZ <katayama.hirofumi.mz@gmail.com>
//  */
//
// #include <win32k.h>
// #include <safe_user_io.h>
//
// #define NDEBUG
// #include <debug.h>
//
// BOOL
// NTAPI
// GreSetBitmapOwner(
//     _In_ HBITMAP hbmp,
//     _In_ ULONG ulOwner)
// {
//     /* Check if we have the correct object type */
//     if (GDI_HANDLE_GET_TYPE(hbmp as HGDIOBJ) != GDILoObjType_LO_BITMAP_TYPE)
//     {
//         DPRINT1("Incorrect type for hbmp: %p\n", hbmp);
//         return FALSE;
//     }
//
//     /// FIXME: this is a hack and doesn't handle a race condition properly.
//     /// It needs to be done in GDIOBJ_vSetObjectOwner atomically.
//
//     /* Check if we set public or none */
//     if ((ulOwner == GDI_OBJ_HMGR_PUBLIC) ||
//         (ulOwner == GDI_OBJ_HMGR_NONE))
//     {
//         /* Only allow this for owned objects */
//         if (GreGetObjectOwner(hbmp as HGDIOBJ) != GDI_OBJ_HMGR_POWNED)
//         {
//             DPRINT1("Cannot change owner for non-powned hbmp\n");
//             return FALSE;
//         }
//     }
//
//     return GreSetObjectOwner(hbmp as HGDIOBJ, ulOwner);
// }
//
// LONG
// NTAPI
// UnsafeSetBitmapBits(
//     _Inout_ PSURFACE psurf,
//     _In_ ULONG cjBits,
//     _In_ const VOID *pvBits)
// {
//     PUCHAR pjDst;
//     const UCHAR *pjSrc;
//     LONG lDeltaDst, lDeltaSrc, lDeltaDstAbs;
//     ULONG Y, iSrc, iDst, cbSrc, cbDst, nWidth, nHeight, cBitsPixel;
//
//     NT_ASSERT(psurf->flags & API_BITMAP);
//     NT_ASSERT(psurf->SurfObj.iBitmapFormat <= BMF_32BPP);
//
//     nWidth = psurf->SurfObj.sizlBitmap.cx;
//     nHeight = psurf->SurfObj.sizlBitmap.cy;
//     cBitsPixel = BitsPerFormat(psurf->SurfObj.iBitmapFormat);
//
//     pjDst = psurf->SurfObj.pvScan0;
//     pjSrc = pvBits;
//     lDeltaDst = psurf->SurfObj.lDelta;
//     lDeltaDstAbs = labs(lDeltaDst);
//     lDeltaSrc = WIDTH_BYTES_ALIGN16(nWidth, cBitsPixel);
//     NT_ASSERT(lDeltaSrc <= lDeltaDstAbs);
//
//     cbDst = lDeltaDstAbs * nHeight;
//     cbSrc = lDeltaSrc * nHeight;
//     cjBits = min(cjBits, cbSrc);
//
//     iSrc = iDst = 0;
//     for (Y = 0; Y < nHeight; Y++)
//     {
//         if (iSrc + lDeltaSrc > cjBits || iDst + lDeltaDstAbs > cbDst)
//         {
//             LONG lDelta = min(cjBits - iSrc, cbDst - iDst);
//             NT_ASSERT(lDelta >= 0);
//             RtlCopyMemory(pjDst, pjSrc, lDelta);
//             iSrc += lDelta;
//             break;
//         }
//
//         /* Copy one line */
//         RtlCopyMemory(pjDst, pjSrc, lDeltaSrc);
//         pjSrc += lDeltaSrc;
//         pjDst += lDeltaDst;
//         iSrc += lDeltaSrc;
//         iDst += lDeltaDstAbs;
//     }
//
//     return iSrc;
// }
//
// HBITMAP
// NTAPI
// GreCreateBitmapEx(
//     _In_ ULONG nWidth,
//     _In_ ULONG nHeight,
//     _In_ ULONG cjWidthBytes,
//     _In_ ULONG iFormat,
//     _In_ USHORT fjBitmap,
//     _In_ ULONG cjSizeImage,
//     _In_opt_ PVOID pvBits,
//     _In_ FLONG flags)
// {
//     PSURFACE psurf;
//     HBITMAP hbmp;
//     PVOID pvCompressedBits = NULL;
//
//     /* Verify format */
//     if (iFormat < BMF_1BPP || iFormat > BMF_PNG) return NULL;
//
//     /* The infamous RLE hack */
//     if ((iFormat == BMF_4RLE) || (iFormat == BMF_8RLE))
//     {
//         pvCompressedBits = pvBits;
//         pvBits = NULL;
//         iFormat = (iFormat == BMF_4RLE) ? BMF_4BPP : BMF_8BPP;
//     }
//
//     /* Allocate a surface */
//     psurf = SURFACE_AllocSurface(STYPE_BITMAP,
//                                  nWidth,
//                                  nHeight,
//                                  iFormat,
//                                  fjBitmap,
//                                  cjWidthBytes,
//                                  pvCompressedBits ? 0 : cjSizeImage,
//                                  pvBits);
//     if (!psurf)
//     {
//         DPRINT1("SURFACE_AllocSurface failed.\n");
//         return NULL;
//     }
//
//     /* The infamous RLE hack */
//     if (pvCompressedBits)
//     {
//         SIZEL sizl;
//         LONG lDelta;
//
//         sizl.cx = nWidth;
//         sizl.cy = nHeight;
//         lDelta = WIDTH_BYTES_ALIGN32(nWidth, gajBitsPerFormat[iFormat]);
//
//         pvBits = psurf->SurfObj.pvBits;
//         DecompressBitmap(sizl, pvCompressedBits, pvBits, lDelta, iFormat, cjSizeImage);
//     }
//
//     /* Get the handle for the bitmap */
//     hbmp = (HBITMAP)psurf->SurfObj.hsurf;
//
//     /* Mark as API bitmap */
//     psurf->flags |= (flags | API_BITMAP);
//
//     /* Unlock the surface and return */
//     SURFACE_UnlockSurface(psurf);
//     return hbmp;
// }
//
// /* Creates a DDB surface,
//  * as in CreateCompatibleBitmap or CreateBitmap.
//  * Note that each scanline must be 32bit aligned!
//  */
// HBITMAP
// NTAPI
// GreCreateBitmap(
//     _In_ ULONG nWidth,
//     _In_ ULONG nHeight,
//     _In_ ULONG cPlanes,
//     _In_ ULONG cBitsPixel,
//     _In_opt_ PVOID pvBits)
// {
//     /* Call the extended function */
//     return GreCreateBitmapEx(nWidth,
//                              nHeight,
//                              0, /* Auto width */
//                              BitmapFormat(cBitsPixel * cPlanes, BI_RGB),
//                              0, /* No bitmap flags */
//                              0, /* Auto size */
//                              pvBits,
//                              DDB_SURFACE /* DDB */);
// }
//
// HBITMAP
// APIENTRY
// NtGdiCreateBitmap(
//     IN INT nWidth,
//     IN INT nHeight,
//     IN UINT cPlanes,
//     IN UINT cBitsPixel,
//     IN OPTIONAL LPBYTE pUnsafeBits)
// {
//     HBITMAP hbmp;
//     ULONG cRealBpp, cjWidthBytes, iFormat;
//     ULONGLONG cjSize;
//     PSURFACE psurf;
//
//     /* Calculate bitmap format and real bits per pixel. */
//     iFormat = BitmapFormat(cBitsPixel * cPlanes, BI_RGB);
//     cRealBpp = gajBitsPerFormat[iFormat];
//
//     /* Calculate width and image size in bytes */
//     cjWidthBytes = WIDTH_BYTES_ALIGN16(nWidth, cRealBpp);
//     cjSize = (ULONGLONG)cjWidthBytes * nHeight;
//
//     /* Check parameters (possible overflow of cjSize!) */
//     if ((iFormat == 0) || (nWidth <= 0) || (nWidth >= 0x8000000) || (nHeight <= 0) ||
//         (cBitsPixel > 32) || (cPlanes > 32) || (cjSize >= 0x100000000ULL))
//     {
//         DPRINT1("Invalid bitmap format! Width=%d, Height=%d, Bpp=%u, Planes=%u\n",
//                 nWidth, nHeight, cBitsPixel, cPlanes);
//         EngSetLastError(ERROR_INVALID_PARAMETER);
//         return NULL;
//     }
//
//     /* Allocate the surface (but don't set the bits) */
//     psurf = SURFACE_AllocSurface(STYPE_BITMAP,
//                                  nWidth,
//                                  nHeight,
//                                  iFormat,
//                                  0,
//                                  0,
//                                  0,
//                                  NULL);
//     if (!psurf)
//     {
//         DPRINT1("SURFACE_AllocSurface failed.\n");
//         return NULL;
//     }
//
//     /* Mark as API and DDB bitmap */
//     psurf->flags |= (API_BITMAP | DDB_SURFACE);
//
//     /* Check if we have bits to set */
//     if (pUnsafeBits)
//     {
//         /* Protect with SEH and copy the bits */
//         _SEH2_TRY
//         {
//             ProbeForRead(pUnsafeBits, (SIZE_T)cjSize, 1);
//             UnsafeSetBitmapBits(psurf, cjSize, pUnsafeBits);
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             GDIOBJ_vDeleteObject(&psurf->BaseObject);
//             _SEH2_YIELD(return NULL;)
//         }
//         _SEH2_END
//     }
//     else
//     {
//         /* Zero the bits */
//         RtlZeroMemory(psurf->SurfObj.pvBits, psurf->SurfObj.cjBits);
//     }
//
//     /* Get the handle for the bitmap */
//     hbmp = (HBITMAP)psurf->SurfObj.hsurf;
//
//     /* Unlock the surface */
//     SURFACE_UnlockSurface(psurf);
//
//     return hbmp;
// }
//
//
// HBITMAP FASTCALL
// IntCreateCompatibleBitmap(
//     PDC Dc,
//     INT Width,
//     INT Height,
//     UINT Planes,
//     UINT Bpp)
// {
//     HBITMAP Bmp = NULL;
//     PPALETTE ppal;
//
//     /* MS doc says if width or height is 0, return 1-by-1 pixel, monochrome bitmap */
//     if (0 == Width || 0 == Height)
//     {
//         return NtGdiGetStockObject(DEFAULT_BITMAP);
//     }
//
//     if (Dc->dctype != DCTYPE_MEMORY)
//     {
//         PSURFACE psurf;
//
//         Bmp = GreCreateBitmap(abs(Width),
//                               abs(Height),
//                               Planes ? Planes : 1,
//                               Bpp ? Bpp : Dc->ppdev->gdiinfo.cBitsPixel,
//                               NULL);
//         if (Bmp == NULL)
//         {
//             DPRINT1("Failed to allocate a bitmap!\n");
//             return NULL;
//         }
//
//         psurf = SURFACE_ShareLockSurface(Bmp);
//         ASSERT(psurf);
//
//         /* Dereference old palette and set new palette */
//         ppal = PALETTE_ShareLockPalette(Dc->ppdev->devinfo.hpalDefault);
//         ASSERT(ppal);
//         SURFACE_vSetPalette(psurf, ppal);
//         PALETTE_ShareUnlockPalette(ppal);
//
//         /* Set flags */
//         psurf->flags = API_BITMAP;
//         psurf->hdc = NULL; // FIXME:
//         psurf->SurfObj.hdev = (HDEV)Dc->ppdev;
//         SURFACE_ShareUnlockSurface(psurf);
//     }
//     else
//     {
//         DIBSECTION dibs;
//         INT Count;
//         PSURFACE psurf = Dc->dclevel.pSurface;
//         if(!psurf) psurf = psurfDefaultBitmap;
//         Count = BITMAP_GetObject(psurf, sizeof(dibs), &dibs);
//
//         if (Count == sizeof(BITMAP))
//         {
//             PSURFACE psurfBmp;
//
//             Bmp = GreCreateBitmap(abs(Width),
//                           abs(Height),
//                           Planes ? Planes : 1,
//                           Bpp ? Bpp : dibs.dsBm.bmBitsPixel,
//                           NULL);
//             if (Bmp == NULL)
//             {
//                 DPRINT1("Failed to allocate a bitmap!\n");
//                 return NULL;
//             }
//             psurfBmp = SURFACE_ShareLockSurface(Bmp);
//             ASSERT(psurfBmp);
//
//             /* Dereference old palette and set new palette */
//             SURFACE_vSetPalette(psurfBmp, psurf->ppal);
//
//             /* Set flags */
//             psurfBmp->flags = API_BITMAP;
//             psurfBmp->hdc = NULL; // FIXME:
//             psurfBmp->SurfObj.hdev = (HDEV)Dc->ppdev;
//             SURFACE_ShareUnlockSurface(psurfBmp);
//         }
//         else if (Count == sizeof(DIBSECTION))
//         {
//             /* A DIB section is selected in the DC */
//             BYTE buf[sizeof(BITMAPINFOHEADER) + 256*sizeof(RGBQUAD)] = {0};
//             PVOID Bits;
//             BITMAPINFO* bi = (BITMAPINFO*)buf;
//
//             bi->bmiHeader.biSize          = sizeof(bi->bmiHeader);
//             bi->bmiHeader.biWidth         = Width;
//             bi->bmiHeader.biHeight        = Height;
//             bi->bmiHeader.biPlanes        = Planes ? Planes : dibs.dsBmih.biPlanes;
//             bi->bmiHeader.biBitCount      = Bpp ? Bpp : dibs.dsBmih.biBitCount;
//             bi->bmiHeader.biCompression   = dibs.dsBmih.biCompression;
//             bi->bmiHeader.biSizeImage     = 0;
//             bi->bmiHeader.biXPelsPerMeter = dibs.dsBmih.biXPelsPerMeter;
//             bi->bmiHeader.biYPelsPerMeter = dibs.dsBmih.biYPelsPerMeter;
//             bi->bmiHeader.biClrUsed       = dibs.dsBmih.biClrUsed;
//             bi->bmiHeader.biClrImportant  = dibs.dsBmih.biClrImportant;
//
//             if (bi->bmiHeader.biCompression == BI_BITFIELDS)
//             {
//                 /* Copy the color masks */
//                 RtlCopyMemory(bi->bmiColors, dibs.dsBitfields, 3*sizeof(RGBQUAD));
//             }
//             else if (bi->bmiHeader.biBitCount <= 8)
//             {
//                 /* Copy the color table */
//                 UINT Index;
//                 PPALETTE PalGDI;
//
//                 if (!psurf->ppal)
//                 {
//                     EngSetLastError(ERROR_INVALID_HANDLE);
//                     return 0;
//                 }
//
//                 PalGDI = psurf->ppal;
//
//                 for (Index = 0;
//                         Index < 256 && Index < PalGDI->NumColors;
//                         Index++)
//                 {
//                     bi->bmiColors[Index].rgbRed   = PalGDI->IndexedColors[Index].peRed;
//                     bi->bmiColors[Index].rgbGreen = PalGDI->IndexedColors[Index].peGreen;
//                     bi->bmiColors[Index].rgbBlue  = PalGDI->IndexedColors[Index].peBlue;
//                     bi->bmiColors[Index].rgbReserved = 0;
//                 }
//             }
//
//             Bmp = DIB_CreateDIBSection(Dc,
//                                        bi,
//                                        DIB_RGB_COLORS,
//                                        &Bits,
//                                        NULL,
//                                        0,
//                                        0);
//             return Bmp;
//         }
//     }
//     return Bmp;
// }
//
// HBITMAP APIENTRY
// NtGdiCreateCompatibleBitmap(
//     HDC hDC,
//     INT Width,
//     INT Height)
// {
//     HBITMAP Bmp;
//     PDC Dc;
//
//     /* Check parameters */
//     if ((Width <= 0) || (Height <= 0) || ((Width * Height) > 0x3FFFFFFF))
//     {
//         EngSetLastError(ERROR_INVALID_PARAMETER);
//         return NULL;
//     }
//
//     if (!hDC)
//         return GreCreateBitmap(Width, Height, 1, 1, 0);
//
//     Dc = DC_LockDc(hDC);
//
//     DPRINT("NtGdiCreateCompatibleBitmap(%p,%d,%d, bpp:%u) = \n",
//            hDC, Width, Height, Dc->ppdev->gdiinfo.cBitsPixel);
//
//     if (NULL == Dc)
//     {
//         EngSetLastError(ERROR_INVALID_HANDLE);
//         return NULL;
//     }
//
//     Bmp = IntCreateCompatibleBitmap(Dc, Width, Height, 0, 0);
//
//     DC_UnlockDc(Dc);
//     return Bmp;
// }
//
// BOOL
// NTAPI
// GreGetBitmapDimension(
//     _In_ HBITMAP hBitmap,
//     _Out_ LPSIZE psizDim)
// {
//     PSURFACE psurfBmp;
//
//     if (hBitmap == NULL)
//         return FALSE;
//
//     /* Lock the bitmap */
//     psurfBmp = SURFACE_ShareLockSurface(hBitmap);
//     if (psurfBmp == NULL)
//     {
//         EngSetLastError(ERROR_INVALID_HANDLE);
//         return FALSE;
//     }
//
//     *psizDim = psurfBmp->sizlDim;
//
//     /* Unlock the bitmap */
//     SURFACE_ShareUnlockSurface(psurfBmp);
//
//     return TRUE;
// }
//
// BOOL
// APIENTRY
// NtGdiGetBitmapDimension(
//     HBITMAP hBitmap,
//     LPSIZE psizDim)
// {
//     SIZE dim;
//
//     if (!GreGetBitmapDimension(hBitmap, &dim))
//         return FALSE;
//
//     /* Use SEH to copy the data to the caller */
//     {
//         NTSTATUS __SehStatus = SafeProbeForWrite(psizDim, sizeof(*psizDim), 1);
//         if (!NT_SUCCESS(__SehStatus))
//         {
//             return FALSE;
//         }
//     }
//     *psizDim = dim;
//
//     return TRUE;
// }
//
//
// LONG
// FASTCALL
// UnsafeGetBitmapBits(
//     PSURFACE psurf,
//     DWORD Bytes,
//     OUT PBYTE pvBits)
// {
//     PUCHAR pjDst, pjSrc;
//     LONG lDeltaDst, lDeltaSrc, lDeltaSrcAbs;
//     ULONG Y, iSrc, iDst, cbSrc, cbDst, nWidth, nHeight, cBitsPixel;
//
//     nWidth = psurf->SurfObj.sizlBitmap.cx;
//     nHeight = psurf->SurfObj.sizlBitmap.cy;
//     cBitsPixel = BitsPerFormat(psurf->SurfObj.iBitmapFormat);
//
//     /* Get pointers */
//     pjSrc = psurf->SurfObj.pvScan0;
//     pjDst = pvBits;
//     lDeltaSrc = psurf->SurfObj.lDelta;
//     lDeltaSrcAbs = labs(lDeltaSrc);
//     lDeltaDst = WIDTH_BYTES_ALIGN16(nWidth, cBitsPixel);
//     NT_ASSERT(lDeltaSrcAbs >= lDeltaDst);
//
//     cbSrc = nHeight * lDeltaSrcAbs;
//     cbDst = nHeight * lDeltaDst;
//     Bytes = min(Bytes, cbDst);
//
//     iSrc = iDst = 0;
//     for (Y = 0; Y < nHeight; Y++)
//     {
//         if (iSrc + lDeltaSrcAbs > cbSrc || iDst + lDeltaDst > Bytes)
//         {
//             LONG lDelta = min(cbSrc - iSrc, Bytes - iDst);
//             NT_ASSERT(lDelta >= 0);
//             RtlCopyMemory(pjDst, pjSrc, lDelta);
//             iDst += lDelta;
//             break;
//         }
//
//         /* Copy one line */
//         RtlCopyMemory(pjDst, pjSrc, lDeltaDst);
//         pjSrc += lDeltaSrc;
//         pjDst += lDeltaDst;
//         iSrc += lDeltaSrcAbs;
//         iDst += lDeltaDst;
//     }
//
//     return iDst;
// }
//
// LONG
// APIENTRY
// NtGdiGetBitmapBits(
//     HBITMAP hBitmap,
//     ULONG cjBuffer,
//     OUT OPTIONAL PBYTE pUnsafeBits)
// {
//     PSURFACE psurf;
//     ULONG cjSize;
//     LONG ret;
//
//     /* Check parameters */
//     if (pUnsafeBits != NULL && cjBuffer == 0)
//     {
//         return 0;
//     }
//
//     /* Lock the bitmap */
//     psurf = SURFACE_ShareLockSurface(hBitmap);
//     if (!psurf)
//     {
//         EngSetLastError(ERROR_INVALID_HANDLE);
//         return 0;
//     }
//
//     /* Calculate the size of the bitmap in bytes */
//     cjSize = WIDTH_BYTES_ALIGN16(psurf->SurfObj.sizlBitmap.cx,
//                 BitsPerFormat(psurf->SurfObj.iBitmapFormat)) *
//                 psurf->SurfObj.sizlBitmap.cy;
//
//     /* If the bits vector is null, the function should return the read size */
//     if (pUnsafeBits == NULL)
//     {
//         SURFACE_ShareUnlockSurface(psurf);
//         return cjSize;
//     }
//
//     /* Don't copy more bytes than the buffer has */
//     cjBuffer = min(cjBuffer, cjSize);
//
//     // FIXME: Use MmSecureVirtualMemory
//     _SEH2_TRY
//     {
//         ProbeForWrite(pUnsafeBits, cjBuffer, 1);
//         ret = UnsafeGetBitmapBits(psurf, cjBuffer, pUnsafeBits);
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         ret = 0;
//     }
//     _SEH2_END
//
//     SURFACE_ShareUnlockSurface(psurf);
//
//     return ret;
// }
//
//
// LONG APIENTRY
// NtGdiSetBitmapBits(
//     HBITMAP hBitmap,
//     DWORD Bytes,
//     IN PBYTE pUnsafeBits)
// {
//     LONG ret;
//     PSURFACE psurf;
//
//     if (pUnsafeBits == NULL || Bytes == 0)
//     {
//         return 0;
//     }
//
//     if (GDI_HANDLE_IS_STOCKOBJ(hBitmap as HGDIOBJ))
//     {
//         return 0;
//     }
//
//     psurf = SURFACE_ShareLockSurface(hBitmap);
//     if (psurf == NULL)
//     {
//         EngSetLastError(ERROR_INVALID_HANDLE);
//         return 0;
//     }
//
//     if (((psurf->flags & API_BITMAP) == 0) ||
//         (psurf->SurfObj.iBitmapFormat > BMF_32BPP))
//     {
//         DPRINT1("Invalid bitmap: iBitmapFormat = %lu, flags = 0x%lx\n",
//                 psurf->SurfObj.iBitmapFormat,
//                 psurf->flags);
//         EngSetLastError(ERROR_INVALID_HANDLE);
//         SURFACE_ShareUnlockSurface(psurf);
//         return 0;
//     }
//
//     _SEH2_TRY
//     {
//         /* NOTE: Win2k3 doesn't check WORD alignment here. */
//         ProbeForWrite(pUnsafeBits, Bytes, 1);
//         ret = UnsafeSetBitmapBits(psurf, Bytes, pUnsafeBits);
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         ret = 0;
//     }
//     _SEH2_END
//
//     SURFACE_ShareUnlockSurface(psurf);
//
//     return ret;
// }
//
// BOOL APIENTRY
// NtGdiSetBitmapDimension(
//     HBITMAP hBitmap,
//     INT Width,
//     INT Height,
//     LPSIZE Size)
// {
//     PSURFACE psurf;
//     BOOL Ret = TRUE;
//     NTSTATUS Status;
//
//     if (hBitmap == NULL)
//         return FALSE;
//
//     psurf = SURFACE_ShareLockSurface(hBitmap);
//     if (psurf == NULL)
//     {
//         EngSetLastError(ERROR_INVALID_HANDLE);
//         return FALSE;
//     }
//
//     if (Size)
//     {
//         Status = SafeProbeForWrite(Size, sizeof(SIZE), 1);
//         if (!NT_SUCCESS(Status))
//         {
//             Ret = FALSE;
//         }
//         else
//         {
//             *Size = psurf->sizlDim;
//         }
//     }
//
//     /* The dimension is changed even if writing the old value failed */
//     psurf->sizlDim.cx = Width;
//     psurf->sizlDim.cy = Height;
//
//     SURFACE_ShareUnlockSurface(psurf);
//
//     return Ret;
// }
//
//
// /*  Internal Functions  */
//
// HBITMAP
// FASTCALL
// BITMAP_CopyBitmap(HBITMAP hBitmap)
// {
//     HBITMAP hbmNew;
//     SURFACE *psurfSrc, *psurfNew;
//
//     /* Fail, if no source bitmap is given */
//     if (hBitmap == NULL) return 0;
//
//     /* Lock the source bitmap */
//     psurfSrc = SURFACE_ShareLockSurface(hBitmap);
//     if (psurfSrc == NULL)
//     {
//         return 0;
//     }
//
//     /* Allocate a new bitmap with the same dimensions as the source bmp */
//     hbmNew = GreCreateBitmapEx(psurfSrc->SurfObj.sizlBitmap.cx,
//                                psurfSrc->SurfObj.sizlBitmap.cy,
//                                abs(psurfSrc->SurfObj.lDelta),
//                                psurfSrc->SurfObj.iBitmapFormat,
//                                psurfSrc->SurfObj.fjBitmap & BMF_TOPDOWN,
//                                psurfSrc->SurfObj.cjBits,
//                                NULL,
//                                psurfSrc->flags);
//
//     if (hbmNew)
//     {
//         /* Lock the new bitmap */
//         psurfNew = SURFACE_ShareLockSurface(hbmNew);
//         if (psurfNew)
//         {
//             /* Copy the bitmap bits to the new bitmap buffer */
//             RtlCopyMemory(psurfNew->SurfObj.pvBits,
//                           psurfSrc->SurfObj.pvBits,
//                           psurfNew->SurfObj.cjBits);
//
//
//             /* Reference the palette of the source bitmap and use it */
//             SURFACE_vSetPalette(psurfNew, psurfSrc->ppal);
//
//             /* Unlock the new surface */
//             SURFACE_ShareUnlockSurface(psurfNew);
//         }
//         else
//         {
//             /* Failed to lock the bitmap, shouldn't happen */
//             GreDeleteObject(hbmNew);
//             hbmNew = NULL;
//         }
//     }
//
//     /* Unlock the source bitmap and return the handle of the new bitmap */
//     SURFACE_ShareUnlockSurface(psurfSrc);
//     return hbmNew;
// }
//
// INT APIENTRY
// BITMAP_GetObject(SURFACE *psurf, INT Count, LPVOID buffer)
// {
//     PBITMAP pBitmap;
//
//     if (!buffer) return sizeof(BITMAP);
//     if ((UINT)Count < sizeof(BITMAP)) return 0;
//
//     /* Always fill a basic BITMAP structure */
//     pBitmap = buffer;
//     pBitmap->bmType = 0;
//     pBitmap->bmWidth = psurf->SurfObj.sizlBitmap.cx;
//     pBitmap->bmHeight = psurf->SurfObj.sizlBitmap.cy;
//     pBitmap->bmPlanes = 1;
//     pBitmap->bmBitsPixel = BitsPerFormat(psurf->SurfObj.iBitmapFormat);
//     pBitmap->bmWidthBytes = WIDTH_BYTES_ALIGN16(pBitmap->bmWidth, pBitmap->bmBitsPixel);
//
//     /* Check for DIB section */
//     if (psurf->hSecure)
//     {
//         /* Set bmBits in this case */
//         pBitmap->bmBits = psurf->SurfObj.pvBits;
//         /* DIBs data are 32 bits aligned */
//         pBitmap->bmWidthBytes = WIDTH_BYTES_ALIGN32(pBitmap->bmWidth, pBitmap->bmBitsPixel);
//
//         if (Count >= sizeof(DIBSECTION))
//         {
//             /* Fill rest of DIBSECTION */
//             PDIBSECTION pds = buffer;
//
//             pds->dsBmih.biSize = sizeof(BITMAPINFOHEADER);
//             pds->dsBmih.biWidth = pds->dsBm.bmWidth;
//             pds->dsBmih.biHeight = pds->dsBm.bmHeight;
//             pds->dsBmih.biPlanes = pds->dsBm.bmPlanes;
//             pds->dsBmih.biBitCount = pds->dsBm.bmBitsPixel;
//
//             switch (psurf->SurfObj.iBitmapFormat)
//             {
//                 case BMF_1BPP:
//                 case BMF_4BPP:
//                 case BMF_8BPP:
//                    pds->dsBmih.biCompression = BI_RGB;
//                    break;
//
//                 case BMF_16BPP:
//                     if (psurf->ppal->flFlags & PAL_RGB16_555)
//                         pds->dsBmih.biCompression = BI_RGB;
//                     else
//                         pds->dsBmih.biCompression = BI_BITFIELDS;
//                     break;
//
//                 case BMF_24BPP:
//                 case BMF_32BPP:
//                     /* 24/32bpp BI_RGB is actually BGR format */
//                     if (psurf->ppal->flFlags & PAL_BGR)
//                         pds->dsBmih.biCompression = BI_RGB;
//                     else
//                         pds->dsBmih.biCompression = BI_BITFIELDS;
//                     break;
//
//                 case BMF_4RLE:
//                    pds->dsBmih.biCompression = BI_RLE4;
//                    break;
//                 case BMF_8RLE:
//                    pds->dsBmih.biCompression = BI_RLE8;
//                    break;
//                 case BMF_JPEG:
//                    pds->dsBmih.biCompression = BI_JPEG;
//                    break;
//                 case BMF_PNG:
//                    pds->dsBmih.biCompression = BI_PNG;
//                    break;
//                 default:
//                     ASSERT(FALSE); /* This shouldn't happen */
//             }
//
//             pds->dsBmih.biSizeImage = psurf->SurfObj.cjBits;
//             pds->dsBmih.biXPelsPerMeter = 0;
//             pds->dsBmih.biYPelsPerMeter = 0;
//             pds->dsBmih.biClrUsed = psurf->ppal->NumColors;
//             pds->dsBmih.biClrImportant = psurf->biClrImportant;
//             pds->dsBitfields[0] = psurf->ppal->RedMask;
//             pds->dsBitfields[1] = psurf->ppal->GreenMask;
//             pds->dsBitfields[2] = psurf->ppal->BlueMask;
//             pds->dshSection = psurf->hDIBSection;
//             pds->dsOffset = psurf->dwOffset;
//
//             return sizeof(DIBSECTION);
//         }
//     }
//     else
//     {
//         /* Not set according to wine test, confirmed in win2k */
//         pBitmap->bmBits = NULL;
//     }
//
//     return sizeof(BITMAP);
// }
//
// /*
//  * @implemented
//  */
// HDC
// APIENTRY
// NtGdiGetDCforBitmap(
//     IN HBITMAP hsurf)
// {
//     HDC hdc = NULL;
//     PSURFACE psurf = SURFACE_ShareLockSurface(hsurf);
//     if (psurf)
//     {
//         hdc = psurf->hdc;
//         SURFACE_ShareUnlockSurface(psurf);
//     }
//     return hdc;
// }
//
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
