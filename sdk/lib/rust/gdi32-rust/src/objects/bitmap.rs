//! objects/bitmap.c → Rust. Bitmap/DIB creation & blitting client API.
//
// NEEDS-TYPE: none — all scalar aliases + GDI structs come from `win32k_sys::win32k`
//   (BOOL/INT/UINT/LONG/WORD/DWORD/ULONG/UCHAR/USHORT/ULONG_PTR/SIZE_T/COLORREF/FLONG/
//   PVOID/HANDLE/HDC/HBITMAP/HGDIOBJ/HPALETTE/HPEN/HBRUSH/LPVOID/LPBYTE/PLDC/
//   BITMAP/BITMAPINFO/PBITMAPINFO/LPBITMAPINFO/BITMAPINFOHEADER/BITMAPCOREHEADER/
//   PBITMAPCOREHEADER/BITMAPCOREINFO/PBITMAPCOREINFO/BITMAPV4HEADER/BITMAPV5HEADER/
//   RGBQUAD/DIBSECTION/PDC_ATTR). All `NtGdi*` declarations come from win32k_sys too.
//
// NEEDS-IMPORT: cross-file gdi32-private / kernel32 / ntdll fns, decorations matched to
//   their C prototypes (flagged for central confirmation they stay exported in gdi32.dll):
//     objects/utils.c   LPBITMAPINFO WINAPI ConvertBitmapInfo(...)             → extern "system"
//     misc/misc.c       BOOL         WINAPI GdiValidateHandle(HGDIOBJ)         → extern "system"
//     misc/misc.c       VOID         WINAPI GdiSetLastError(DWORD)             → extern "system"
//     misc/misc.c       BOOL         (cdecl) GdiGetHandleUserData(...)         → extern "C"
//     gdi32p.h          PLDC         FASTCALL GdiGetLDC(HDC)                   → extern "fastcall"
//     gdi32p.h          VOID         (cdecl) GdiSAPCallback(PLDC)              → extern "C"
//     objects/dc.c      HDC          WINAPI GdiConvertAndCheckDC(HDC)          → extern "system"
//     objects/dc.c      HDC          WINAPI CreateCompatibleDC(HDC)            → extern "system"
//     objects/dc.c      INT          WINAPI SaveDC(HDC) / BOOL RestoreDC(HDC,INT) → extern "system"
//     objects/dc.c      BOOL         WINAPI DeleteDC(HDC)                      → extern "system"
//     objects/gdiobj.c  HGDIOBJ      WINAPI GetStockObject(INT)               → extern "system"
//     objects/gdiobj.c  INT          WINAPI GetObjectA(HGDIOBJ,INT,LPVOID)    → extern "system"
//     objects/gdiobj.c  HGDIOBJ      WINAPI SelectObject(HDC,HGDIOBJ)         → extern "system"
//     objects/dc.c      HGDIOBJ      WINAPI GetCurrentObject(HDC,UINT)        → extern "system"
//     objects/palette.c HPALETTE     WINAPI SelectPalette(HDC,HPALETTE,BOOL)  → extern "system"
//     objects/dc.c      UINT         WINAPI GetDIBColorTable(HDC,UINT,UINT,RGBQUAD*) → extern "system"
//     objects/printdrv  INT          WINAPI StartPage(HDC)                     → extern "system"
//     kernel32          VOID         WINAPI SetLastError(DWORD)               → extern "system"
//     kernel32          HANDLE       WINAPI GetProcessHeap()                  → extern "system"
//     ntdll             PVOID        NTAPI  RtlAllocateHeap(PVOID,ULONG,SIZE_T) → extern "system"
//     ntdll             BOOLEAN      NTAPI  RtlFreeHeap(PVOID,ULONG,PVOID)    → extern "system"
//
// NEEDS-CHECK: `RtlULongMult` (ntintsafe.h __forceinline, NOT a real ntdll export) is
//   reimplemented locally below as `rtl_ulong_mult` (checked 32-bit multiply returning an
//   NTSTATUS) — it must NOT be declared `extern`, that would be an undefined symbol at link.
// NEEDS-CHECK: `gajBitsPerFormat[11]` is a non-static gdi32-local global (copied from
//   eng/surface.c). Emitted `#[no_mangle] static mut`; lives only in gdi32.dll so it does
//   not collide with win32k's same-named symbol.
// NEEDS-CHECK: the two `_SEH2_TRY { RtlCopyMemory } _SEH2_EXCEPT { Hit = TRUE }` blocks are
//   translated under USE_DUMMY_PSEH: the except handler is DEAD CODE, so `Hit` stays FALSE
//   and the `if (Hit)` recovery path is unreachable — faithful to how the C compiles here.

use win32k_sys::win32k::*;
use crate::gdihelp::*;

// win32k_sys's FALSE/TRUE are bindgen u32 (#define), but BOOL is i32 — shadow with
// BOOL-typed locals so `== FALSE`/`return TRUE` type-check against the i32 BOOL returns.
const FALSE: BOOL = 0;
const TRUE: BOOL = 1;

// handle-type constants (ntgdihdl.h / GDILoObjType), as `usize` for compare.
const LO_DC_TYPE: usize = GDILoObjType_GDILoObjType_LO_DC_TYPE as usize;
const LO_METADC16_TYPE: usize = GDILoObjType_GDILoObjType_LO_METADC16_TYPE as usize;

// NTSTATUS bits we need for the local RtlULongMult reimplementation.
const STATUS_SUCCESS: LONG = 0;
const STATUS_INTEGER_OVERFLOW: LONG = 0xC000_0095u32 as LONG;
const ULONG_ERROR: ULONG = 0xFFFF_FFFF;

// ── Cross-file gdi32 / kernel32 / ntdll fns (WINAPI / NTAPI → extern "system") ───
extern "system" {
    fn ConvertBitmapInfo(
        BitmapInfo: PBITMAPINFO,
        ColorSpec: UINT,
        BitmapInfoSize: *mut UINT,
        FollowedByData: BOOL,
    ) -> LPBITMAPINFO;
    fn GdiValidateHandle(hGdiObj: HGDIOBJ) -> BOOL;
    fn GdiSetLastError(dwErrCode: DWORD);
    fn GdiConvertAndCheckDC(hdc: HDC) -> HDC;
    fn CreateCompatibleDC(hdc: HDC) -> HDC;
    fn SaveDC(hdc: HDC) -> INT;
    fn RestoreDC(hdc: HDC, nSavedDC: INT) -> BOOL;
    fn DeleteDC(hdc: HDC) -> BOOL;
    fn GetStockObject(i: INT) -> HGDIOBJ;
    fn GetObjectA(h: HGDIOBJ, c: INT, pv: LPVOID) -> INT;
    fn SelectObject(hdc: HDC, h: HGDIOBJ) -> HGDIOBJ;
    fn GetCurrentObject(hdc: HDC, uObjectType: UINT) -> HGDIOBJ;
    fn SelectPalette(hdc: HDC, hpal: HPALETTE, bForceBackground: BOOL) -> HPALETTE;
    fn GetDIBColorTable(hdc: HDC, iStart: UINT, cEntries: UINT, prgbq: *mut RGBQUAD) -> UINT;
    fn StartPage(hdc: HDC) -> INT;
    fn SetLastError(dwErrCode: DWORD);
    fn GetProcessHeap() -> HANDLE;
    fn RtlAllocateHeap(HeapHandle: PVOID, Flags: ULONG, Size: SIZE_T) -> PVOID;
    fn RtlFreeHeap(HeapHandle: PVOID, Flags: ULONG, BaseAddress: PVOID) -> BOOLEAN;
}

// ── GdiGetLDC (FASTCALL → extern "fastcall", i386 `@GdiGetLDC@4`) ────────────────
extern "fastcall" {
    fn GdiGetLDC(hdc: HDC) -> PLDC;
}

// ── plain-cdecl gdi32-private fns + metafile recorders (no WINAPI) → extern "C" ──
extern "C" {
    fn GdiGetHandleUserData(hGdiObj: HGDIOBJ, ObjectType: DWORD, UserData: *mut PVOID) -> BOOL;
    fn GdiSAPCallback(pldc: PLDC);
    fn METADC_SetDIBitsToDevice(
        hdc: HDC,
        x_dest: INT,
        y_dest: INT,
        width: DWORD,
        height: DWORD,
        x_src: INT,
        y_src: INT,
        startscan: UINT,
        lines: UINT,
        bits: *const core::ffi::c_void,
        info: *const BITMAPINFO,
        coloruse: UINT,
    ) -> INT;
    fn EMFDC_SetDIBitsToDevice(
        dc_attr: PLDC,
        x_dest: INT,
        y_dest: INT,
        width: DWORD,
        height: DWORD,
        x_src: INT,
        y_src: INT,
        startscan: UINT,
        lines: UINT,
        bits: *const core::ffi::c_void,
        info: *const BITMAPINFO,
        coloruse: UINT,
    ) -> BOOL;
    fn METADC_StretchDIBits(
        hdc: HDC,
        x_dst: INT,
        y_dst: INT,
        width_dst: INT,
        height_dst: INT,
        x_src: INT,
        y_src: INT,
        width_src: INT,
        height_src: INT,
        bits: *const core::ffi::c_void,
        info: *const BITMAPINFO,
        coloruse: UINT,
        rop: DWORD,
    ) -> BOOL;
    fn EMFDC_StretchDIBits(
        dc_attr: PLDC,
        x_dst: INT,
        y_dst: INT,
        width_dst: INT,
        height_dst: INT,
        x_src: INT,
        y_src: INT,
        width_src: INT,
        height_src: INT,
        bits: *const core::ffi::c_void,
        info: *const BITMAPINFO,
        coloruse: UINT,
        rop: DWORD,
    ) -> BOOL;
}

/// `RtlGetProcessHeap()` — FORCEINLINE macro `NtCurrentPeb()->ProcessHeap` (rtlfuncs.h),
/// reproduced over the win32k_sys _TEB/_PEB (NtCurrentTeb lives in crate::gdihelp).
#[inline]
unsafe fn RtlGetProcessHeap() -> PVOID {
    (*(*NtCurrentTeb()).ProcessEnvironmentBlock).ProcessHeap
}

/// `RtlULongMult` (ntintsafe.h __forceinline) — NOT a real export; reimplemented locally.
/// Multiplies two ULONGs with overflow detection: on overflow sets *out = ULONG_ERROR and
/// returns STATUS_INTEGER_OVERFLOW; otherwise stores the product and returns STATUS_SUCCESS.
#[inline]
unsafe fn rtl_ulong_mult(a: ULONG, b: ULONG, out: *mut ULONG) -> LONG {
    match a.checked_mul(b) {
        Some(p) => {
            *out = p;
            STATUS_SUCCESS
        }
        None => {
            *out = ULONG_ERROR;
            STATUS_INTEGER_OVERFLOW
        }
    }
}

/// `NT_SUCCESS(Status) = ((NTSTATUS)(Status) >= 0)`.
#[inline]
fn NT_SUCCESS(status: LONG) -> bool {
    status >= 0
}

/// `WIDTH_BYTES_ALIGN32(cx, bpp) = ((((cx)*(bpp) + 31) & ~31) >> 3)` (Yuan / amvideo.h).
/// Reproduced over 32-bit signed int arithmetic, matching the C macro's `int` expansion.
#[inline]
fn width_bytes_align32(cx: LONG, bpp: LONG) -> LONG {
    ((cx.wrapping_mul(bpp)).wrapping_add(31) & !31) >> 3
}

/* Copied from win32ss/gdi/eng/surface.c */
#[no_mangle]
pub unsafe extern "fastcall" fn BitmapFormat(cBits: ULONG, iCompression: ULONG) -> ULONG {
    match iCompression {
        // BI_RGB / BI_BITFIELDS — fall through
        x if x == BI_RGB || x == BI_BITFIELDS => {
            if cBits <= 1 {
                return BMF_1BPP;
            }
            if cBits <= 4 {
                return BMF_4BPP;
            }
            if cBits <= 8 {
                return BMF_8BPP;
            }
            if cBits <= 16 {
                return BMF_16BPP;
            }
            if cBits <= 24 {
                return BMF_24BPP;
            }
            if cBits <= 32 {
                return BMF_32BPP;
            }
            0
        }
        x if x == BI_RLE4 => BMF_4RLE,
        x if x == BI_RLE8 => BMF_8RLE,
        _ => 0,
    }
}

/* Copied from win32ss/gdi/eng/surface.c */
#[no_mangle]
pub static mut gajBitsPerFormat: [UCHAR; 11] = [
    0,  /*  0: unused */
    1,  /*  1: BMF_1BPP */
    4,  /*  2: BMF_4BPP */
    8,  /*  3: BMF_8BPP */
    16, /*  4: BMF_16BPP */
    24, /*  5: BMF_24BPP */
    32, /*  6: BMF_32BPP */
    4,  /*  7: BMF_4RLE */
    8,  /*  8: BMF_8RLE */
    0,  /*  9: BMF_JPEG */
    0,  /* 10: BMF_PNG */
];

/*
 *           DIB_BitmapInfoSize
 *
 * Return the size of the bitmap info structure including color table.
 * 11/16/1999 (RJJ) lifted from wine
 */
#[no_mangle]
pub unsafe extern "fastcall" fn DIB_BitmapInfoSize(
    info: *const BITMAPINFO,
    coloruse: WORD,
    max: BOOL,
) -> INT {
    let mut masks: UINT = 0;

    if (*info).bmiHeader.biSize == core::mem::size_of::<BITMAPCOREHEADER>() as DWORD {
        let core: *const BITMAPCOREHEADER = info as *const BITMAPCOREHEADER;
        let mut size = core::mem::size_of::<BITMAPCOREHEADER>() as UINT;
        if (*core).bcBitCount <= 8 {
            let colors = 1u32 << (*core).bcBitCount;
            if coloruse as UINT == DIB_RGB_COLORS {
                size += colors * core::mem::size_of::<RGBTRIPLE>() as UINT;
            } else {
                size += colors * core::mem::size_of::<WORD>() as UINT;
            }
        }
        return size as INT;
    } else
    /* assume BITMAPINFOHEADER */
    {
        let mut colors = if max != 0 {
            1u32 << (*info).bmiHeader.biBitCount
        } else {
            (*info).bmiHeader.biClrUsed
        };
        if colors > 256 {
            colors = 256;
        }
        if colors == 0 && ((*info).bmiHeader.biBitCount <= 8) {
            colors = 1u32 << (*info).bmiHeader.biBitCount;
        }
        if (*info).bmiHeader.biCompression == BI_BITFIELDS {
            masks = 3;
        }
        let size = core::cmp::max(
            (*info).bmiHeader.biSize,
            core::mem::size_of::<BITMAPINFOHEADER>() as UINT + masks * core::mem::size_of::<DWORD>() as UINT,
        );
        return (size
            + colors
                * (if coloruse as UINT == DIB_RGB_COLORS {
                    core::mem::size_of::<RGBQUAD>() as UINT
                } else {
                    core::mem::size_of::<WORD>() as UINT
                })) as INT;
    }
}

/*
 * Return the full scan size for a bitmap.
 *
 * Based on Wine, Utils.c and Windows Graphics Prog pg 595, SDK amvideo.h.
 */
#[no_mangle]
pub unsafe extern "fastcall" fn DIB_BitmapMaxBitsSize(Info: PBITMAPINFO, ScanLines: UINT) -> UINT {
    let Ret: UINT;

    if Info.is_null() {
        return 0;
    }

    if (*Info).bmiHeader.biSize == core::mem::size_of::<BITMAPCOREHEADER>() as DWORD {
        let Core: PBITMAPCOREHEADER = Info as PBITMAPCOREHEADER;
        Ret = (width_bytes_align32(
            (*Core).bcWidth as LONG * (*Core).bcPlanes as LONG,
            (*Core).bcBitCount as LONG,
        ) as UINT)
            .wrapping_mul(ScanLines);
    } else
    /* assume BITMAPINFOHEADER */
    {
        if ((*Info).bmiHeader.biCompression == BI_RGB)
            || ((*Info).bmiHeader.biCompression == BI_BITFIELDS)
        {
            Ret = (width_bytes_align32(
                (*Info).bmiHeader.biWidth * (*Info).bmiHeader.biPlanes as LONG,
                (*Info).bmiHeader.biBitCount as LONG,
            ) as UINT)
                .wrapping_mul(ScanLines);
        } else {
            Ret = (*Info).bmiHeader.biSizeImage;
        }
    }
    Ret
}

/*
 * DIB_GetBitmapInfo is complete copy of wine cvs 2/9-2006
 * from file dib.c from gdi32.dll or orginal version
 * did not calc the info right for some headers.
 */
#[no_mangle]
pub unsafe extern "system" fn DIB_GetBitmapInfo(
    header: *const BITMAPINFOHEADER,
    width: PLONG,
    height: PLONG,
    planes: PWORD,
    bpp: PWORD,
    compr: PLONG,
    size: PLONG,
) -> INT {
    if (*header).biSize == core::mem::size_of::<BITMAPCOREHEADER>() as DWORD {
        let core: *const BITMAPCOREHEADER = header as *const BITMAPCOREHEADER;
        *width = (*core).bcWidth as LONG;
        *height = (*core).bcHeight as LONG;
        *planes = (*core).bcPlanes;
        *bpp = (*core).bcBitCount;
        *compr = 0;
        *size = 0;
        return 0;
    }

    if (*header).biSize == core::mem::size_of::<BITMAPINFOHEADER>() as DWORD {
        *width = (*header).biWidth;
        *height = (*header).biHeight;
        *planes = (*header).biPlanes;
        *bpp = (*header).biBitCount;
        *compr = (*header).biCompression as LONG;
        *size = (*header).biSizeImage as LONG;
        return 1;
    }

    if (*header).biSize == core::mem::size_of::<BITMAPV4HEADER>() as DWORD {
        let v4hdr: *const BITMAPV4HEADER = header as *const BITMAPV4HEADER;
        *width = (*v4hdr).bV4Width;
        *height = (*v4hdr).bV4Height;
        *planes = (*v4hdr).bV4Planes;
        *bpp = (*v4hdr).bV4BitCount;
        *compr = (*v4hdr).bV4V4Compression as LONG;
        *size = (*v4hdr).bV4SizeImage as LONG;
        return 4;
    }

    if (*header).biSize == core::mem::size_of::<BITMAPV5HEADER>() as DWORD {
        let v5hdr: *const BITMAPV5HEADER = header as *const BITMAPV5HEADER;
        *width = (*v5hdr).bV5Width;
        *height = (*v5hdr).bV5Height;
        *planes = (*v5hdr).bV5Planes;
        *bpp = (*v5hdr).bV5BitCount;
        *compr = (*v5hdr).bV5Compression as LONG;
        *size = (*v5hdr).bV5SizeImage as LONG;
        return 5;
    }
    // DPRINT("(%lu): wrong size for header\n", header->biSize);
    -1
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GdiGetBitmapBitsSize(lpbmi: *mut BITMAPINFO) -> INT {
    let Ret: UINT;

    if lpbmi.is_null() {
        return 0;
    }

    if (*lpbmi).bmiHeader.biSize == core::mem::size_of::<BITMAPCOREHEADER>() as DWORD {
        let Core: PBITMAPCOREHEADER = lpbmi as PBITMAPCOREHEADER;
        Ret = (width_bytes_align32(
            (*Core).bcWidth as LONG * (*Core).bcPlanes as LONG,
            (*Core).bcBitCount as LONG,
        ) as UINT)
            .wrapping_mul((*Core).bcHeight as UINT);
    } else
    /* assume BITMAPINFOHEADER */
    {
        if ((*lpbmi).bmiHeader.biCompression == 0)
            || ((*lpbmi).bmiHeader.biCompression == BI_BITFIELDS)
        {
            Ret = (width_bytes_align32(
                (*lpbmi).bmiHeader.biWidth * (*lpbmi).bmiHeader.biPlanes as LONG,
                (*lpbmi).bmiHeader.biBitCount as LONG,
            ) as UINT)
                .wrapping_mul((*lpbmi).bmiHeader.biHeight.wrapping_abs() as UINT);
        } else {
            Ret = (*lpbmi).bmiHeader.biSizeImage;
        }
    }
    Ret as INT
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn CreateDIBSection(
    hDC: HDC,
    BitmapInfo: *const BITMAPINFO,
    Usage: UINT,
    Bits: *mut *mut core::ffi::c_void,
    hSection: HANDLE,
    dwOffset: DWORD,
) -> HBITMAP {
    let pConvertedInfo: PBITMAPINFO;
    let mut ConvertedInfoSize: UINT = 0;
    let mut hBitmap: HBITMAP = core::ptr::null_mut();
    let mut bmBits: PVOID = core::ptr::null_mut();

    pConvertedInfo = ConvertBitmapInfo(
        BitmapInfo as PBITMAPINFO,
        Usage,
        &mut ConvertedInfoSize,
        FALSE,
    );
    if !pConvertedInfo.is_null() {
        // Verify header due to converted may == info.
        if (*pConvertedInfo).bmiHeader.biSize >= core::mem::size_of::<BITMAPINFOHEADER>() as DWORD {
            if (*pConvertedInfo).bmiHeader.biCompression == BI_JPEG
                || (*pConvertedInfo).bmiHeader.biCompression == BI_PNG
            {
                SetLastError(ERROR_INVALID_PARAMETER);
                return core::ptr::null_mut();
            }
        }
        hBitmap = NtGdiCreateDIBSection(
            hDC,
            hSection,
            dwOffset,
            pConvertedInfo,
            Usage,
            ConvertedInfoSize,
            0, // fl
            0, // dwColorSpace
            &mut bmBits,
        );
        if hBitmap.is_null() {
            bmBits = core::ptr::null_mut();
        }

        if BitmapInfo as PBITMAPINFO != pConvertedInfo {
            RtlFreeHeap(RtlGetProcessHeap(), 0, pConvertedInfo as PVOID);
        }
    }

    if !Bits.is_null() {
        *Bits = bmBits;
    }

    hBitmap
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn CreateBitmap(
    Width: INT,
    Height: INT,
    Planes: UINT,
    BitsPixel: UINT,
    pUnsafeBits: *const core::ffi::c_void,
) -> HBITMAP {
    if Width != 0 && Height != 0 {
        NtGdiCreateBitmap(Width, Height, Planes, BitsPixel, pUnsafeBits as LPBYTE)
    } else {
        /* Return 1x1 bitmap */
        GetStockObject(DEFAULT_BITMAP as INT) as HBITMAP
    }
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn CreateBitmapIndirect(pbm: *const BITMAP) -> HBITMAP {
    let mut bitmap: HBITMAP = core::ptr::null_mut();

    /* Note windows xp/2003 does not check if pbm is NULL or not */
    if ((*pbm).bmWidthBytes != 0) && ((((*pbm).bmWidthBytes) & 1) == 0) {
        bitmap = CreateBitmap(
            (*pbm).bmWidth,
            (*pbm).bmHeight,
            (*pbm).bmPlanes as UINT,
            (*pbm).bmBitsPixel as UINT,
            (*pbm).bmBits,
        );
    } else {
        SetLastError(ERROR_INVALID_PARAMETER);
    }

    bitmap
}

#[no_mangle]
pub unsafe extern "system" fn CreateDiscardableBitmap(hDC: HDC, Width: INT, Height: INT) -> HBITMAP {
    CreateCompatibleBitmap(hDC, Width, Height)
}

#[repr(C)]
struct CompatBuffer {
    bitmap: BITMAP,
    bmih: BITMAPINFOHEADER,
    rgbquad: [RGBQUAD; 256],
}

#[no_mangle]
pub unsafe extern "system" fn CreateCompatibleBitmap(hDC: HDC, Width: INT, Height: INT) -> HBITMAP {
    let mut pDc_Attr: PDC_ATTR = core::ptr::null_mut();

    if GdiGetHandleUserData(
        hDC as HGDIOBJ,
        GDI_OBJECT_TYPE_DC,
        &mut pDc_Attr as *mut PDC_ATTR as *mut PVOID,
    ) == FALSE
    {
        return core::ptr::null_mut();
    }

    if Width == 0 || Height == 0 {
        return GetStockObject(DEFAULT_BITMAP as INT) as HBITMAP;
    }

    if ((*pDc_Attr).ulDirty_ & DC_DIBSECTION) == 0 {
        NtGdiCreateCompatibleBitmap(hDC, Width, Height)
    } else {
        let hBmp: HBITMAP;
        let mut buffer: CompatBuffer = core::mem::zeroed();
        let pDIBs: *mut DIBSECTION = &mut buffer as *mut CompatBuffer as *mut DIBSECTION;
        let pbmi: *mut BITMAPINFO = &mut buffer.bmih as *mut BITMAPINFOHEADER as *mut BITMAPINFO;

        hBmp = NtGdiGetDCObject(hDC, GDI_OBJECT_TYPE_BITMAP as INT) as HBITMAP;

        if GetObjectA(
            hBmp as HGDIOBJ,
            core::mem::size_of::<DIBSECTION>() as INT,
            pDIBs as LPVOID,
        ) != core::mem::size_of::<DIBSECTION>() as INT
        {
            return core::ptr::null_mut();
        }

        if (*pDIBs).dsBm.bmBitsPixel <= 8 {
            GetDIBColorTable(hDC, 0, 256, buffer.rgbquad.as_mut_ptr());
        }

        (*pDIBs).dsBmih.biWidth = Width;
        (*pDIBs).dsBmih.biHeight = Height;

        CreateDIBSection(
            hDC,
            pbmi,
            DIB_RGB_COLORS,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            0,
        )
    }
}

#[no_mangle]
pub unsafe extern "system" fn GetDIBits(
    hDC: HDC,
    hbmp: HBITMAP,
    uStartScan: UINT,
    cScanLines: UINT,
    lpvBits: LPVOID,
    lpbmi: LPBITMAPINFO,
    uUsage: UINT,
) -> INT {
    let cjBmpScanSize: UINT;
    let cjInfoSize: UINT;

    if hDC.is_null() || GdiValidateHandle(hDC as HGDIOBJ) == FALSE || lpbmi.is_null() {
        GdiSetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    cjBmpScanSize = DIB_BitmapMaxBitsSize(lpbmi, cScanLines);
    /* Caller must provide maximum size possible */
    cjInfoSize = DIB_BitmapInfoSize(lpbmi, uUsage as WORD, TRUE) as UINT;

    if !lpvBits.is_null() {
        if (*lpbmi).bmiHeader.biSize >= core::mem::size_of::<BITMAPINFOHEADER>() as DWORD {
            if (*lpbmi).bmiHeader.biCompression == BI_JPEG
                || (*lpbmi).bmiHeader.biCompression == BI_PNG
            {
                SetLastError(ERROR_INVALID_PARAMETER);
                return 0;
            }
        }
    }

    NtGdiGetDIBitsInternal(
        hDC,
        hbmp,
        uStartScan,
        cScanLines,
        lpvBits as LPBYTE,
        lpbmi,
        uUsage,
        cjBmpScanSize,
        cjInfoSize,
    )
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn CreateDIBitmap(
    hDC: HDC,
    Header: *const BITMAPINFOHEADER,
    Init: DWORD,
    Bits: *const core::ffi::c_void,
    Data: *const BITMAPINFO,
    ColorUse: UINT,
) -> HBITMAP {
    let mut Width: LONG = 0;
    let mut Height: LONG = 0;
    let mut Compression: LONG = 0;
    let mut DibSize: LONG = 0;
    let mut Planes: WORD = 0;
    let mut BitsPerPixel: WORD = 0;
    //  PDC_ATTR pDc_Attr;
    let cjBmpScanSize: UINT;
    let mut hBitmap: HBITMAP = core::ptr::null_mut();
    let pbmiConverted: PBITMAPINFO;
    let mut cjInfoSize: UINT = 0;

    let mut Header = Header;
    let mut Bits = Bits;
    let mut Init = Init;

    /* Convert the BITMAPINFO if it is a COREINFO */
    pbmiConverted = ConvertBitmapInfo(Data as PBITMAPINFO, ColorUse, &mut cjInfoSize, FALSE);

    /* Check for CBM_CREATDIB */
    if (Init & CBM_CREATDIB) != 0 {
        if cjInfoSize == 0 {
            // goto Exit;
            if (!pbmiConverted.is_null()) && (pbmiConverted != Data as PBITMAPINFO) {
                RtlFreeHeap(RtlGetProcessHeap(), 0, pbmiConverted as PVOID);
            }
            return hBitmap;
        } else if (Init & CBM_INIT) != 0 {
            if Bits.is_null() {
                // goto Exit;
                if (!pbmiConverted.is_null()) && (pbmiConverted != Data as PBITMAPINFO) {
                    RtlFreeHeap(RtlGetProcessHeap(), 0, pbmiConverted as PVOID);
                }
                return hBitmap;
            }
        } else {
            Bits = core::ptr::null();
        }

        /* CBM_CREATDIB needs Data. */
        if pbmiConverted.is_null() {
            // DPRINT1("CBM_CREATDIB needs a BITMAPINFO!\n");
            // goto Exit;
            if (!pbmiConverted.is_null()) && (pbmiConverted != Data as PBITMAPINFO) {
                RtlFreeHeap(RtlGetProcessHeap(), 0, pbmiConverted as PVOID);
            }
            return hBitmap;
        }

        /* It only works with PAL or RGB */
        if ColorUse > DIB_PAL_COLORS {
            // DPRINT1("Invalid ColorUse: %lu\n", ColorUse);
            GdiSetLastError(ERROR_INVALID_PARAMETER);
            // goto Exit;
            if (!pbmiConverted.is_null()) && (pbmiConverted != Data as PBITMAPINFO) {
                RtlFreeHeap(RtlGetProcessHeap(), 0, pbmiConverted as PVOID);
            }
            return hBitmap;
        }

        /* Use the header from the data */
        Header = &(*Data).bmiHeader as *const BITMAPINFOHEADER;
    } else {
        if (Init & CBM_INIT) != 0 {
            if !Bits.is_null() {
                if cjInfoSize == 0 {
                    // goto Exit;
                    if (!pbmiConverted.is_null()) && (pbmiConverted != Data as PBITMAPINFO) {
                        RtlFreeHeap(RtlGetProcessHeap(), 0, pbmiConverted as PVOID);
                    }
                    return hBitmap;
                }
            } else {
                Init &= !CBM_INIT;
            }
        }
    }

    /* Header is required */
    if Header.is_null() {
        // DPRINT1("Header is NULL\n");
        GdiSetLastError(ERROR_INVALID_PARAMETER);
        // goto Exit;
        if (!pbmiConverted.is_null()) && (pbmiConverted != Data as PBITMAPINFO) {
            RtlFreeHeap(RtlGetProcessHeap(), 0, pbmiConverted as PVOID);
        }
        return hBitmap;
    }

    /* Get the bitmap format and dimensions */
    if DIB_GetBitmapInfo(
        Header,
        &mut Width,
        &mut Height,
        &mut Planes,
        &mut BitsPerPixel,
        &mut Compression,
        &mut DibSize,
    ) == -1
    {
        // DPRINT1("DIB_GetBitmapInfo failed!\n");
        GdiSetLastError(ERROR_INVALID_PARAMETER);
        // goto Exit;
        if (!pbmiConverted.is_null()) && (pbmiConverted != Data as PBITMAPINFO) {
            RtlFreeHeap(RtlGetProcessHeap(), 0, pbmiConverted as PVOID);
        }
        return hBitmap;
    }

    /* Check if the Compr is incompatible */
    if (Compression == BI_JPEG as LONG) || (Compression == BI_PNG as LONG) {
        // DPRINT1("Invalid compression: %lu!\n", Compression);
        // goto Exit;
        if (!pbmiConverted.is_null()) && (pbmiConverted != Data as PBITMAPINFO) {
            RtlFreeHeap(RtlGetProcessHeap(), 0, pbmiConverted as PVOID);
        }
        return hBitmap;
    }

    /* Only DIB_RGB_COLORS (0), DIB_PAL_COLORS (1) and 2 are valid. */
    if ColorUse > DIB_PAL_COLORS + 1 {
        // DPRINT1("Invalid compression: %lu!\n", Compression);
        GdiSetLastError(ERROR_INVALID_PARAMETER);
        // goto Exit;
        if (!pbmiConverted.is_null()) && (pbmiConverted != Data as PBITMAPINFO) {
            RtlFreeHeap(RtlGetProcessHeap(), 0, pbmiConverted as PVOID);
        }
        return hBitmap;
    }

    /* If some Bits are given, only DIB_PAL_COLORS and DIB_RGB_COLORS are valid */
    if !Bits.is_null() && (ColorUse > DIB_PAL_COLORS) {
        // DPRINT1("Invalid ColorUse: %lu\n", ColorUse);
        GdiSetLastError(ERROR_INVALID_PARAMETER);
        // goto Exit;
        if (!pbmiConverted.is_null()) && (pbmiConverted != Data as PBITMAPINFO) {
            RtlFreeHeap(RtlGetProcessHeap(), 0, pbmiConverted as PVOID);
        }
        return hBitmap;
    }

    /* Negative width is not allowed */
    if Width < 0 {
        // DPRINT1("Negative width: %li\n", Width);
        // goto Exit;
        if (!pbmiConverted.is_null()) && (pbmiConverted != Data as PBITMAPINFO) {
            RtlFreeHeap(RtlGetProcessHeap(), 0, pbmiConverted as PVOID);
        }
        return hBitmap;
    }

    /* Top-down DIBs have a negative height. */
    Height = Height.wrapping_abs();

    // For Icm support.
    // GdiGetHandleUserData(hdc, GDI_OBJECT_TYPE_DC, (PVOID)&pDc_Attr))

    cjBmpScanSize = GdiGetBitmapBitsSize(pbmiConverted) as UINT;

    // DPRINT("pBMI %p, Size bpp %u, dibsize %d, Conv %u, BSS %u\n", ...);

    if Width == 0 || Height == 0 {
        hBitmap = GetStockObject(DEFAULT_BITMAP as INT) as HBITMAP;
    } else {
        hBitmap = NtGdiCreateDIBitmapInternal(
            hDC,
            Width,
            Height,
            Init,
            Bits as LPBYTE,
            pbmiConverted as LPBITMAPINFO,
            ColorUse,
            cjInfoSize,
            cjBmpScanSize,
            0,
            core::ptr::null_mut(),
        );
    }

    // Exit:
    /* Cleanup converted BITMAPINFO */
    if (!pbmiConverted.is_null()) && (pbmiConverted != Data as PBITMAPINFO) {
        RtlFreeHeap(RtlGetProcessHeap(), 0, pbmiConverted as PVOID);
    }

    hBitmap
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn SetDIBits(
    hDC: HDC,
    hBitmap: HBITMAP,
    uStartScan: UINT,
    cScanLines: UINT,
    lpvBits: *const core::ffi::c_void,
    lpbmi: *const BITMAPINFO,
    fuColorUse: UINT,
) -> INT {
    let hDCc: HDC;
    let mut SavehDC: HDC;
    let nhDC: HDC;
    let dwWidth: DWORD;
    let dwHeight: DWORD;
    let hOldBitmap: HGDIOBJ;
    let mut hPal: HPALETTE = core::ptr::null_mut();
    let mut LinesCopied: INT = 0;
    let mut newDC: BOOL = FALSE;

    if fuColorUse != DIB_RGB_COLORS && fuColorUse != DIB_PAL_COLORS {
        return 0;
    }

    if lpvBits.is_null()
        || (GDI_HANDLE_GET_TYPE(hBitmap as HGDIOBJ) != GDI_OBJECT_TYPE_BITMAP as usize)
    {
        return 0;
    }

    if (*lpbmi).bmiHeader.biSize >= core::mem::size_of::<BITMAPINFOHEADER>() as DWORD {
        if (*lpbmi).bmiHeader.biCompression == BI_JPEG
            || (*lpbmi).bmiHeader.biCompression == BI_PNG
        {
            SetLastError(ERROR_INVALID_PARAMETER);
            return 0;
        }
    }

    if (*lpbmi).bmiHeader.biCompression == BI_BITFIELDS {
        let masks: *const DWORD = (*lpbmi).bmiColors.as_ptr() as *const DWORD;
        if *masks.add(0) == 0 || *masks.add(1) == 0 || *masks.add(2) == 0 {
            SetLastError(ERROR_INVALID_PARAMETER);
            return 0;
        }
    }

    hDCc = NtGdiGetDCforBitmap(hBitmap); // hDC can be NULL, so, get it from the bitmap.
    SavehDC = hDCc;
    if hDCc.is_null()
    // No DC associated with bitmap, Clone or Create one.
    {
        nhDC = CreateCompatibleDC(hDC);
        if nhDC.is_null() {
            return 0;
        }
        newDC = TRUE;
        SavehDC = nhDC;
    } else if SaveDC(hDCc) == 0 {
        return 0;
    }

    hOldBitmap = SelectObject(SavehDC, hBitmap as HGDIOBJ);

    if !hOldBitmap.is_null() {
        if !hDC.is_null() {
            hPal = SelectPalette(
                SavehDC,
                GetCurrentObject(hDC, OBJ_PAL) as HPALETTE,
                FALSE,
            );
        }

        if (*lpbmi).bmiHeader.biSize < core::mem::size_of::<BITMAPINFOHEADER>() as DWORD {
            let pbci: PBITMAPCOREINFO = lpbmi as PBITMAPCOREINFO;
            dwWidth = (*pbci).bmciHeader.bcWidth as DWORD;
            dwHeight = (*pbci).bmciHeader.bcHeight as DWORD;
        } else {
            dwWidth = (*lpbmi).bmiHeader.biWidth as DWORD;
            dwHeight = (*lpbmi).bmiHeader.biHeight.wrapping_abs() as DWORD;
        }

        LinesCopied = SetDIBitsToDevice(
            SavehDC,
            0,
            0,
            dwWidth,
            dwHeight,
            0,
            0,
            uStartScan,
            cScanLines,
            lpvBits,
            lpbmi,
            fuColorUse,
        );

        if !hDC.is_null() {
            SelectPalette(SavehDC, hPal, FALSE);
        }

        SelectObject(SavehDC, hOldBitmap);
    }

    if newDC != 0 {
        DeleteDC(SavehDC);
    } else {
        RestoreDC(SavehDC, -1);
    }

    LinesCopied
}

/*
 * @implemented
 *
 */
const MAX_SCAN_LINES: UINT = 1000;
const MAX_HEIGHT: DWORD = 2000;
const MAX_SOURCE_HEIGHT: LONG = 2000;

/// `IS_ALIGNED(Pointer, Alignment) = (((ULONG_PTR)(void*)(Pointer)) % (Alignment) == 0)`.
#[inline]
fn is_aligned(p: ULONG_PTR, alignment: ULONG_PTR) -> bool {
    p % alignment == 0
}

#[no_mangle]
pub unsafe extern "system" fn SetDIBitsToDevice(
    hdc: HDC,
    XDest: core::ffi::c_int,
    YDest: core::ffi::c_int,
    Width: DWORD,
    Height: DWORD,
    XSrc: core::ffi::c_int,
    mut YSrc: core::ffi::c_int,
    mut StartScan: UINT,
    mut ScanLines: UINT,
    Bits: *const core::ffi::c_void,
    lpbmi: *const BITMAPINFO,
    ColorUse: UINT,
) -> INT {
    let mut pDc_Attr: PDC_ATTR = core::ptr::null_mut();
    let pConvertedInfo: PBITMAPINFO;
    let mut ConvertedInfoSize: UINT = 0;
    let mut LinesCopied: INT = 0;
    let mut cjBmpScanSize: UINT;
    let mut Hit: BOOL = FALSE;
    let mut pvSafeBits: PVOID = Bits as PVOID;
    let bmiHeight: UINT;
    let top_down: BOOL;
    let mut src_y: INT = 0;
    let iFormat: ULONG;
    let cBitsPixel: ULONG;
    let mut cjBits: ULONG = 0;
    let cjWidth: ULONG;

    if ScanLines == 0 || lpbmi.is_null() || Bits.is_null() {
        return 0;
    }

    // DPRINT("ScanLines ... biCompression ... biBitCount ...");

    if (*lpbmi).bmiHeader.biWidth < 0 {
        return 0;
    }

    if ColorUse != 0 && ColorUse != DIB_PAL_COLORS && ColorUse != DIB_PAL_COLORS + 1 {
        return 0;
    }

    pConvertedInfo = ConvertBitmapInfo(lpbmi as PBITMAPINFO, ColorUse, &mut ConvertedInfoSize, FALSE);
    if pConvertedInfo.is_null() {
        return 0;
    }

    if ScanLines > MAX_SCAN_LINES {
        LinesCopied = 0;
        // goto Exit;
        return set_dib_to_dev_exit(Bits, pvSafeBits, lpbmi, pConvertedInfo, LinesCopied);
    }

    bmiHeight = (*pConvertedInfo).bmiHeader.biHeight.wrapping_abs() as UINT;
    top_down = ((*pConvertedInfo).bmiHeader.biHeight < 0) as BOOL;
    if (StartScan > bmiHeight) && (ScanLines > bmiHeight) {
        // DPRINT("Returning ScanLines of '%d'\n", ScanLines);
        LinesCopied = ScanLines as INT;
        return set_dib_to_dev_exit(Bits, pvSafeBits, lpbmi, pConvertedInfo, LinesCopied);
    }

    if (*pConvertedInfo).bmiHeader.biHeight == 0 {
        LinesCopied = 0;
        return set_dib_to_dev_exit(Bits, pvSafeBits, lpbmi, pConvertedInfo, LinesCopied);
    }

    if !is_aligned(Bits as ULONG_PTR, 2) && (ScanLines > Height) {
        LinesCopied = 0;
        return set_dib_to_dev_exit(Bits, pvSafeBits, lpbmi, pConvertedInfo, LinesCopied);
    }

    /* Below code modeled after Wine's nulldrv_SetDIBitsToDevice */
    if StartScan <= (YSrc as u32).wrapping_add(bmiHeight) {
        if ((*pConvertedInfo).bmiHeader.biCompression == BI_RLE8)
            || ((*pConvertedInfo).bmiHeader.biCompression == BI_RLE4)
        {
            StartScan = 0;
            ScanLines = bmiHeight;
        } else {
            if StartScan >= bmiHeight {
                LinesCopied = 0;
                return set_dib_to_dev_exit(Bits, pvSafeBits, lpbmi, pConvertedInfo, LinesCopied);
            }
            if top_down == 0 && ScanLines > bmiHeight - StartScan {
                ScanLines = bmiHeight - StartScan;
            }
            src_y = StartScan
                .wrapping_add(ScanLines)
                .wrapping_sub((YSrc as u32).wrapping_add(Height)) as INT;
            if top_down == 0 {
                /* get rid of unnecessary lines */
                if (src_y < 0 || src_y >= ScanLines as INT)
                    && (*pConvertedInfo).bmiHeader.biCompression != BI_BITFIELDS
                {
                    LinesCopied = ScanLines as INT;
                    return set_dib_to_dev_exit(Bits, pvSafeBits, lpbmi, pConvertedInfo, LinesCopied);
                }
                if YDest >= 0 {
                    LinesCopied = (ScanLines.wrapping_add(StartScan)) as INT;
                    ScanLines = ScanLines.wrapping_sub(src_y as u32);
                } else {
                    LinesCopied = (ScanLines.wrapping_sub(src_y as u32)) as INT;
                }
            } else if src_y < 0 || src_y >= ScanLines as INT {
                if (*lpbmi).bmiHeader.biHeight < 0 && StartScan > MAX_SCAN_LINES {
                    ScanLines = ((*lpbmi).bmiHeader.biHeight as u32).wrapping_sub(StartScan);
                }
                // DPRINT("Returning ScanLines of '%d'\n", ScanLines);
                LinesCopied = ScanLines as INT;
                return set_dib_to_dev_exit(Bits, pvSafeBits, lpbmi, pConvertedInfo, LinesCopied);
            } else {
                LinesCopied = ScanLines as INT;
            }
        }
    }

    // HANDLE_METADC(INT, SetDIBitsToDevice, 0, hdc, XDest, YDest, Width, Height, XSrc, YSrc,
    //               StartScan, ScanLines, Bits, lpbmi, ColorUse);
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != LO_DC_TYPE {
        if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) == LO_METADC16_TYPE {
            return METADC_SetDIBitsToDevice(
                hdc, XDest, YDest, Width, Height, XSrc, YSrc, StartScan, ScanLines, Bits, lpbmi,
                ColorUse,
            );
        } else {
            let pLDC: PLDC = GdiGetLDC(hdc);
            let _Ret: INT = 0;
            if pLDC.is_null() {
                SetLastError(ERROR_INVALID_HANDLE);
                return _Ret;
            }
            if (*pLDC).iType == LDC_EMFLDC as INT
                && EMFDC_SetDIBitsToDevice(
                    pLDC, XDest, YDest, Width, Height, XSrc, YSrc, StartScan, ScanLines, Bits,
                    lpbmi, ColorUse,
                ) == 0
            {
                return _Ret;
            }
            /* Fall through to support information DC's. */
        }
    }

    // Handle the "Special Case"!
    {
        let pldc: PLDC;
        let hType: ULONG = GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) as ULONG;
        if hType as usize != LO_DC_TYPE && hType as usize != LO_METADC16_TYPE {
            pldc = GdiGetLDC(hdc);
            if !pldc.is_null() {
                if ((*pldc).Flags & LDC_STARTPAGE) != 0 {
                    StartPage(hdc);
                }

                if ((*pldc).Flags & LDC_SAPCALLBACK) != 0 {
                    GdiSAPCallback(pldc);
                }

                if ((*pldc).Flags & LDC_KILL_DOCUMENT) != 0 {
                    LinesCopied = 0;
                    return set_dib_to_dev_exit(Bits, pvSafeBits, lpbmi, pConvertedInfo, LinesCopied);
                }
            } else {
                SetLastError(ERROR_INVALID_HANDLE);
                LinesCopied = 0;
                return set_dib_to_dev_exit(Bits, pvSafeBits, lpbmi, pConvertedInfo, LinesCopied);
            }
        }
    }

    if ((*pConvertedInfo).bmiHeader.biCompression == BI_RLE8)
        || ((*pConvertedInfo).bmiHeader.biCompression == BI_RLE4)
    {
        /* For compressed data, we must set the whole thing */
        StartScan = 0;
        ScanLines = (*pConvertedInfo).bmiHeader.biHeight as UINT;
    }

    cjBmpScanSize = DIB_BitmapMaxBitsSize(lpbmi as LPBITMAPINFO, ScanLines);

    pvSafeBits = RtlAllocateHeap(GetProcessHeap() as PVOID, 0, cjBmpScanSize as SIZE_T);
    if !pvSafeBits.is_null() {
        // _SEH2_TRY
        core::ptr::copy_nonoverlapping(
            Bits as *const u8,
            pvSafeBits as *mut u8,
            cjBmpScanSize as usize,
        );
        // _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER) { Hit = TRUE; } — dead under DUMMY_PSEH
        let _ = &mut Hit;

        if Hit != 0 {
            // We don't die, we continue on with a allocated safe pointer to kernel
            // space.....
            if !is_aligned(Bits as ULONG_PTR, 2) {
                // If both Read Exception and mis-aligned
                LinesCopied = 0;
                return set_dib_to_dev_exit(Bits, pvSafeBits, lpbmi, pConvertedInfo, LinesCopied);
            }
            // DPRINT1("SetDIBitsToDevice fail to read BitMapInfo ...");
        }
        // DPRINT("SetDIBitsToDevice Allocate Bits %u!!!\n", cjBmpScanSize);
    }

    if GdiGetHandleUserData(
        hdc as HGDIOBJ,
        GDI_OBJECT_TYPE_DC,
        &mut pDc_Attr as *mut PDC_ATTR as *mut PVOID,
    ) == FALSE
    {
        // DPRINT1("SetDIBitsToDevice called on invalid DC %p (not owned?)\n", hdc);
        SetLastError(ERROR_INVALID_PARAMETER);
        LinesCopied = 0;
        return set_dib_to_dev_exit(Bits, pvSafeBits, lpbmi, pConvertedInfo, LinesCopied);
    }

    /* Calculation of ScanLines for NtGdiSetDIBitsToDeviceInternal */
    if YDest >= 0 {
        ScanLines = core::cmp::min((Height as i32).wrapping_abs() as UINT, ScanLines);
        if YSrc > 0 {
            ScanLines = ScanLines.wrapping_add(YSrc as u32);
        }
    } else {
        ScanLines = core::cmp::min(
            ScanLines,
            ((*pConvertedInfo).bmiHeader.biHeight.wrapping_abs() as u32).wrapping_sub(StartScan),
        );
    }

    if YDest >= 0 && YSrc > 0 && bmiHeight <= MAX_HEIGHT {
        ScanLines = ScanLines.wrapping_add(YSrc as u32);
    }

    /* Find Format from lpbmi which is now pConvertedInfo */
    iFormat = BitmapFormat(
        (*pConvertedInfo).bmiHeader.biBitCount as ULONG,
        (*pConvertedInfo).bmiHeader.biCompression,
    );

    /* Get bits per pixel from the format */
    cBitsPixel = gajBitsPerFormat[iFormat as usize] as ULONG;

    cjWidth = width_bytes_align32((*pConvertedInfo).bmiHeader.biWidth, cBitsPixel as LONG) as ULONG;

    /* Calculate the correct bitmap size in bytes */
    if !NT_SUCCESS(rtl_ulong_mult(
        cjWidth,
        core::cmp::max(ScanLines, LinesCopied as u32),
        &mut cjBits,
    )) {
        // DPRINT1("Overflow calculating size ...");
        // goto Exit;
        return set_dib_to_dev_exit(Bits, pvSafeBits, lpbmi, pConvertedInfo, LinesCopied);
    }

    /* Make sure the buffer is large enough */
    if (*pConvertedInfo).bmiHeader.biSizeImage < cjBits {
        // DPRINT("Buffer is too small ...");
        if (*pConvertedInfo).bmiHeader.biCompression == BI_RGB {
            (*pConvertedInfo).bmiHeader.biSizeImage = cjBits;
        }
    }

    if YDest >= 0 {
        ScanLines = core::cmp::min((Height as i32).wrapping_abs() as UINT, ScanLines);
        if YSrc > 0 {
            ScanLines = ScanLines.wrapping_add(YSrc as u32);
            if Height.wrapping_add(YDest as u32).wrapping_add(1) < ScanLines {
                YSrc = 0;
            }
        }
    }

    /*
     if ( !pDc_Attr || ... )*/
    {
        LinesCopied = NtGdiSetDIBitsToDeviceInternal(
            hdc,
            XDest,
            YDest,
            Width,
            Height,
            XSrc,
            YSrc,
            StartScan,
            ScanLines,
            pvSafeBits as LPBYTE,
            pConvertedInfo as LPBITMAPINFO,
            ColorUse,
            cjBmpScanSize,
            ConvertedInfoSize,
            TRUE,
            core::ptr::null_mut(),
        );
    }

    if bmiHeight > MAX_SCAN_LINES {
        LinesCopied = ScanLines as INT;
    }

    if YDest < 0 {
        if top_down != 0 {
            LinesCopied = ScanLines as INT;
        } else {
            LinesCopied = (ScanLines.wrapping_sub(src_y as u32)) as INT;
        }
    }

    if (*pConvertedInfo).bmiHeader.biCompression == BI_RLE8
        || (*pConvertedInfo).bmiHeader.biCompression == BI_RLE4
    {
        LinesCopied = bmiHeight as INT;
    }

    if (*pConvertedInfo).bmiHeader.biHeight < 0 {
        if (*pConvertedInfo).bmiHeader.biHeight < -MAX_SOURCE_HEIGHT || YDest >= 0 {
            LinesCopied = (ScanLines as INT).wrapping_add(src_y);
        } else {
            LinesCopied = ScanLines as INT;
        }
    }

    // Exit:
    let _ = cjBmpScanSize;
    set_dib_to_dev_exit(Bits, pvSafeBits, lpbmi, pConvertedInfo, LinesCopied)
}

/// The `Exit:` epilogue of SetDIBitsToDevice — frees the safe-bits copy and the converted
/// BITMAPINFO (each only when it was actually allocated), then returns LinesCopied.
#[inline]
unsafe fn set_dib_to_dev_exit(
    Bits: *const core::ffi::c_void,
    pvSafeBits: PVOID,
    lpbmi: *const BITMAPINFO,
    pConvertedInfo: PBITMAPINFO,
    LinesCopied: INT,
) -> INT {
    if Bits as PVOID != pvSafeBits {
        RtlFreeHeap(RtlGetProcessHeap(), 0, pvSafeBits);
    }
    if lpbmi as PBITMAPINFO != pConvertedInfo {
        RtlFreeHeap(RtlGetProcessHeap(), 0, pConvertedInfo as PVOID);
    }
    LinesCopied
}

/*
 * @unimplemented
 */
#[no_mangle]
pub unsafe extern "system" fn StretchDIBits(
    hdc: HDC,
    XDest: core::ffi::c_int,
    YDest: core::ffi::c_int,
    nDestWidth: core::ffi::c_int,
    nDestHeight: core::ffi::c_int,
    XSrc: core::ffi::c_int,
    YSrc: core::ffi::c_int,
    nSrcWidth: core::ffi::c_int,
    nSrcHeight: core::ffi::c_int,
    lpBits: *const core::ffi::c_void,
    lpBitsInfo: *const BITMAPINFO,
    iUsage: UINT,
    dwRop: DWORD,
) -> core::ffi::c_int {
    let mut pDc_Attr: PDC_ATTR = core::ptr::null_mut();
    let pConvertedInfo: PBITMAPINFO;
    let mut ConvertedInfoSize: UINT = 0;
    let mut LinesCopied: INT = 0;
    let cjBmpScanSize: UINT;
    let mut pvSafeBits: PVOID = core::ptr::null_mut();
    let mut Hit: BOOL = FALSE;

    // DPRINT("StretchDIBits %p : %p : %u\n", lpBits, lpBitsInfo, iUsage);

    // HANDLE_METADC(int, StretchDIBits, 0, hdc, XDest, YDest, nDestWidth, nDestHeight, XSrc,
    //               YSrc, nSrcWidth, nSrcHeight, lpBits, lpBitsInfo, iUsage, dwRop);
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != LO_DC_TYPE {
        if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) == LO_METADC16_TYPE {
            return METADC_StretchDIBits(
                hdc, XDest, YDest, nDestWidth, nDestHeight, XSrc, YSrc, nSrcWidth, nSrcHeight,
                lpBits, lpBitsInfo, iUsage, dwRop,
            ) as core::ffi::c_int;
        } else {
            let pLDC: PLDC = GdiGetLDC(hdc);
            let _Ret: INT = 0;
            if pLDC.is_null() {
                SetLastError(ERROR_INVALID_HANDLE);
                return _Ret;
            }
            if (*pLDC).iType == LDC_EMFLDC as INT
                && EMFDC_StretchDIBits(
                    pLDC, XDest, YDest, nDestWidth, nDestHeight, XSrc, YSrc, nSrcWidth, nSrcHeight,
                    lpBits, lpBitsInfo, iUsage, dwRop,
                ) == 0
            {
                return _Ret;
            }
        }
    }

    if GdiConvertAndCheckDC(hdc).is_null() {
        return 0;
    }

    pConvertedInfo = ConvertBitmapInfo(lpBitsInfo as PBITMAPINFO, iUsage, &mut ConvertedInfoSize, FALSE);
    if pConvertedInfo.is_null() {
        return 0;
    }

    cjBmpScanSize = GdiGetBitmapBitsSize(pConvertedInfo as *mut BITMAPINFO) as UINT;

    if !lpBits.is_null() {
        pvSafeBits = RtlAllocateHeap(GetProcessHeap() as PVOID, 0, cjBmpScanSize as SIZE_T);
        if !pvSafeBits.is_null() {
            // _SEH2_TRY
            core::ptr::copy_nonoverlapping(
                lpBits as *const u8,
                pvSafeBits as *mut u8,
                cjBmpScanSize as usize,
            );
            // _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER) { Hit = TRUE; } — dead under DUMMY_PSEH
            let _ = &mut Hit;

            if Hit != 0 {
                // We don't die, we continue on with a allocated safe pointer to kernel
                // space.....
                // DPRINT1("StretchDIBits fail to read BitMapInfo ...");
            }
            // DPRINT("StretchDIBits Allocate Bits %u!!!\n", cjBmpScanSize);
        }
    }

    if GdiGetHandleUserData(
        hdc as HGDIOBJ,
        GDI_OBJECT_TYPE_DC,
        &mut pDc_Attr as *mut PDC_ATTR as *mut PVOID,
    ) == FALSE
    {
        // DPRINT1("StretchDIBits called on invalid DC %p (not owned?)\n", hdc);
        SetLastError(ERROR_INVALID_PARAMETER);
        LinesCopied = 0;
        // goto Exit;
        if !pvSafeBits.is_null() {
            RtlFreeHeap(RtlGetProcessHeap(), 0, pvSafeBits);
        }
        if lpBitsInfo as PBITMAPINFO != pConvertedInfo {
            RtlFreeHeap(RtlGetProcessHeap(), 0, pConvertedInfo as PVOID);
        }
        return LinesCopied;
    }
    /*
     if ( !pDc_Attr || ... )*/
    {
        LinesCopied = NtGdiStretchDIBitsInternal(
            hdc,
            XDest,
            YDest,
            nDestWidth,
            nDestHeight,
            XSrc,
            YSrc,
            nSrcWidth,
            nSrcHeight,
            pvSafeBits as LPBYTE,
            pConvertedInfo as LPBITMAPINFO,
            iUsage as DWORD,
            dwRop,
            ConvertedInfoSize,
            cjBmpScanSize,
            core::ptr::null_mut(),
        );
    }
    // Exit:
    if !pvSafeBits.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, pvSafeBits);
    }
    if lpBitsInfo as PBITMAPINFO != pConvertedInfo {
        RtlFreeHeap(RtlGetProcessHeap(), 0, pConvertedInfo as PVOID);
    }

    LinesCopied
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetBitmapAttributes(hbm: HBITMAP) -> DWORD {
    if GDI_HANDLE_IS_STOCKOBJ(hbm as HGDIOBJ) {
        return SC_BB_STOCKOBJ;
    }
    0
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn SetBitmapAttributes(hbm: HBITMAP, dwFlags: DWORD) -> HBITMAP {
    if (dwFlags & !SC_BB_STOCKOBJ) != 0 {
        return core::ptr::null_mut();
    }
    NtGdiSetBitmapAttributes(hbm, dwFlags)
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn ClearBitmapAttributes(hbm: HBITMAP, dwFlags: DWORD) -> HBITMAP {
    if (dwFlags & !SC_BB_STOCKOBJ) != 0 {
        return core::ptr::null_mut();
    }
    NtGdiClearBitmapAttributes(hbm, dwFlags)
}
