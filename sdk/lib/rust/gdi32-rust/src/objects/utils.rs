//! win32ss/gdi/gdi32/objects/utils.c → Rust. Shared GDI client utilities.
//!
//! Owns the LPK.DLL loader hook globals (`hLpk` / `LpkExtTextOut` /
//! `LpkGetCharacterPlacement` / `LpkGetTextExtentExPoint`) that objects/text.c (text.rs)
//! links against, plus the DIB-info helpers `CalculateColorTableSize` / `ConvertBitmapInfo`
//! and the LOGFONT A↔W conversion helpers.
//
// NEEDS-TYPE: the LPK hook fn-pointer typedefs `LPKETO`/`LPKGCP`/`LPKGTEP` (gdi32p.h) are
//   not in win32k_sys; reproduced locally as nullable `Option<extern "system" fn>` so the
//   owned globals can initialize to NULL (text.rs declares them as bare-fn `LPKETO` —
//   link-compatible: both are pointer-sized, null == None).
// NEEDS-IMPORT: cross-file gdi32/kernel32/ntdll fns are declared at the extern blocks below:
//   - GdiGetBitmapBitsSize  (objects/bitmap.c, WINAPI)   → extern "system" (do NOT redefine)
//   - RtlAllocateHeap       (ntdll, NTAPI)               → extern "system"
//   - MultiByteToWideChar / WideCharToMultiByte / LoadLibraryW / GetProcAddress /
//     FreeLibrary           (kernel32, WINAPI)           → extern "system"
// NEEDS-CHECK: `RtlCopyMemory` (a memcpy macro) is rendered as
//   `core::ptr::copy_nonoverlapping` (note Rust's (src,dst,len) arg order vs the macro's
//   (dst,src,len)).

use win32k_sys::win32k::*;
use crate::gdihelp::*;

// win32k_sys's FALSE/TRUE are bindgen u32 (#define), but BOOL is i32 — shadow with
// BOOL-typed locals so `== FALSE`/`return TRUE` type-check against the i32 BOOL returns.
const FALSE: BOOL = 0;
const TRUE: BOOL = 1;

// winnls.h — not in the win32k_sys bindgen surface.
const CP_THREAD_ACP: UINT = 3;

// gdi32p.h LoadLPK() function-id selectors.
const LPK_INIT: INT = 1;
const LPK_ETO: INT = 2;
const LPK_GCP: INT = 3;
const LPK_GTEP: INT = 4;

// ── LPK hook fn-pointer types (gdi32p.h LPKETO/LPKGCP/LPKGTEP) ──────────────────
// Reproduced as nullable Option<fn> so the owned globals can init to NULL.
type LPKETO = Option<
    unsafe extern "system" fn(
        hdc: HDC,
        x: INT,
        y: INT,
        fuOptions: UINT,
        lprc: *const RECT,
        lpString: LPCWSTR,
        uCount: UINT,
        lpDx: *const INT,
        unknown: INT,
    ) -> BOOL,
>;
type LPKGCP = Option<
    unsafe extern "system" fn(
        hdc: HDC,
        lpString: LPCWSTR,
        uCount: INT,
        nMaxExtent: INT,
        lpResults: LPGCP_RESULTSW,
        dwFlags: DWORD,
        dwUnused: DWORD,
    ) -> DWORD,
>;
type LPKGTEP = Option<
    unsafe extern "system" fn(
        hdc: HDC,
        lpString: LPCWSTR,
        cString: INT,
        nMaxExtent: INT,
        lpnFit: LPINT,
        lpnDx: LPINT,
        lpSize: LPSIZE,
        dwUnused: DWORD,
        unknown: INT,
    ) -> BOOL,
>;

// ── LoadLPK global variables (this TU OWNS them) ───────────────────────────────
#[no_mangle]
pub static mut hLpk: HINSTANCE = core::ptr::null_mut();
#[no_mangle]
pub static mut LpkExtTextOut: LPKETO = None;
#[no_mangle]
pub static mut LpkGetCharacterPlacement: LPKGCP = None;
#[no_mangle]
pub static mut LpkGetTextExtentExPoint: LPKGTEP = None;

// ── Cross-file gdi32 / kernel32 / ntdll fns (WINAPI / NTAPI → extern "system") ──
extern "system" {
    // objects/bitmap.c — provided by bitmap.rs; do NOT redefine here.
    fn GdiGetBitmapBitsSize(lpbmi: *mut BITMAPINFO) -> INT;
    // ntdll
    fn RtlAllocateHeap(HeapHandle: PVOID, Flags: ULONG, Size: SIZE_T) -> PVOID;
    // kernel32
    fn MultiByteToWideChar(
        CodePage: UINT,
        dwFlags: DWORD,
        lpMultiByteStr: LPCSTR,
        cbMultiByte: INT,
        lpWideCharStr: LPWSTR,
        cchWideChar: INT,
    ) -> INT;
    fn WideCharToMultiByte(
        CodePage: UINT,
        dwFlags: DWORD,
        lpWideCharStr: LPCWSTR,
        cchWideChar: INT,
        lpMultiByteStr: LPSTR,
        cbMultiByte: INT,
        lpDefaultChar: LPCSTR,
        lpUsedDefaultChar: *mut BOOL,
    ) -> INT;
    fn LoadLibraryW(lpLibFileName: LPCWSTR) -> HINSTANCE;
    fn GetProcAddress(hModule: HINSTANCE, lpProcName: LPCSTR) -> FARPROC;
    fn FreeLibrary(hLibModule: HINSTANCE) -> BOOL;
}

/// `RtlGetProcessHeap()` — FORCEINLINE macro `NtCurrentPeb()->ProcessHeap` (rtlfuncs.h),
/// reproduced over the win32k_sys _TEB/_PEB (NtCurrentTeb lives in crate::gdihelp).
#[inline]
unsafe fn RtlGetProcessHeap() -> PVOID {
    (*(*NtCurrentTeb()).ProcessEnvironmentBlock).ProcessHeap
}

/// Internal routine to calculate the number of color table entries.
///
/// Returns TRUE if the input values together form a valid image, FALSE otherwise.
#[no_mangle]
pub unsafe extern "system" fn CalculateColorTableSize(
    BitmapInfoHeader: *const BITMAPINFOHEADER,
    ColorSpec: *mut UINT,
    ColorTableSize: *mut UINT,
) -> BOOL {
    let BitCount: WORD;
    let ClrUsed: DWORD;
    let Compression: DWORD;

    // At first get some basic parameters from the passed BitmapInfoHeader.
    if (*BitmapInfoHeader).biSize == core::mem::size_of::<BITMAPCOREHEADER>() as DWORD {
        BitCount = (*(BitmapInfoHeader as LPBITMAPCOREHEADER)).bcBitCount;
        ClrUsed = 0;
        Compression = BI_RGB;
    } else {
        BitCount = (*BitmapInfoHeader).biBitCount;
        ClrUsed = (*BitmapInfoHeader).biClrUsed;
        Compression = (*BitmapInfoHeader).biCompression;
    }

    match Compression {
        x if x == BI_BITFIELDS => {
            if *ColorSpec == DIB_PAL_COLORS {
                *ColorSpec = DIB_RGB_COLORS;
            }

            if BitCount != 16 && BitCount != 32 {
                return FALSE;
            }

            // For BITMAPV4HEADER/BITMAPV5HEADER the masks are in the structure itself;
            // for BITMAPINFOHEADER the color masks are stored in the palette.
            if (*BitmapInfoHeader).biSize > core::mem::size_of::<BITMAPINFOHEADER>() as DWORD {
                *ColorTableSize = 0;
            } else {
                *ColorTableSize = 3;
            }

            TRUE
        }

        x if x == BI_RGB => match BitCount {
            1 => {
                *ColorTableSize = if ClrUsed != 0 { core::cmp::min(ClrUsed, 2) } else { 2 };
                TRUE
            }
            4 => {
                *ColorTableSize = if ClrUsed != 0 { core::cmp::min(ClrUsed, 16) } else { 16 };
                TRUE
            }
            8 => {
                *ColorTableSize = if ClrUsed != 0 { core::cmp::min(ClrUsed, 256) } else { 256 };
                TRUE
            }
            _ => {
                if *ColorSpec == DIB_PAL_COLORS {
                    *ColorSpec = DIB_RGB_COLORS;
                }
                if BitCount != 16 && BitCount != 24 && BitCount != 32 {
                    return FALSE;
                }
                *ColorTableSize = ClrUsed;
                TRUE
            }
        },

        x if x == BI_RLE4 => {
            if BitCount == 4 {
                *ColorTableSize = if ClrUsed != 0 { core::cmp::min(ClrUsed, 16) } else { 16 };
                return TRUE;
            }
            FALSE
        }

        x if x == BI_RLE8 => {
            if BitCount == 8 {
                *ColorTableSize = if ClrUsed != 0 { core::cmp::min(ClrUsed, 256) } else { 256 };
                return TRUE;
            }
            FALSE
        }

        x if x == BI_JPEG || x == BI_PNG => {
            *ColorTableSize = ClrUsed;
            TRUE
        }

        _ => FALSE,
    }
}

/// Internal routine to convert a user-passed BITMAPINFO structure into a unified
/// BITMAPINFO structure. Returns either the original BitmapInfo or a newly allocated
/// structure (caller frees the latter with RtlFreeHeap on the current process heap).
#[no_mangle]
pub unsafe extern "system" fn ConvertBitmapInfo(
    BitmapInfo: *const BITMAPINFO,
    mut ColorSpec: UINT,
    BitmapInfoSize: *mut UINT,
    FollowedByData: BOOL,
) -> LPBITMAPINFO {
    let mut NewBitmapInfo: LPBITMAPINFO = BitmapInfo as LPBITMAPINFO;
    let CoreBitmapInfo: LPBITMAPCOREINFO = BitmapInfo as LPBITMAPCOREINFO;
    let mut Size: DWORD = 0;
    let mut DataSize: ULONG = 0;
    let mut PaletteEntryCount: UINT = 0;

    // At first check if the passed BitmapInfo structure has a valid size.
    if BitmapInfo.is_null()
        || ((*BitmapInfo).bmiHeader.biSize != core::mem::size_of::<BITMAPCOREHEADER>() as DWORD
            && ((*BitmapInfo).bmiHeader.biSize < core::mem::size_of::<BITMAPINFOHEADER>() as DWORD
                || (*BitmapInfo).bmiHeader.biSize
                    > core::mem::size_of::<BITMAPV5HEADER>() as DWORD))
    {
        return core::ptr::null_mut();
    }

    // Now calculate the color table size. Invalid color information is rejected here.
    if CalculateColorTableSize(
        &(*BitmapInfo).bmiHeader,
        &mut ColorSpec,
        &mut PaletteEntryCount,
    ) == FALSE
    {
        return core::ptr::null_mut();
    }

    // Calculate the size of image data if applicable.
    if FollowedByData != FALSE {
        DataSize = GdiGetBitmapBitsSize(BitmapInfo as PBITMAPINFO) as ULONG;
    }

    // If BitmapInfo was originally BITMAPCOREINFO convert it to the standard BITMAPINFO layout.
    if (*BitmapInfo).bmiHeader.biSize == core::mem::size_of::<BITMAPCOREHEADER>() as DWORD {
        Size = core::mem::size_of::<BITMAPINFOHEADER>() as DWORD;
        if ColorSpec == DIB_RGB_COLORS {
            Size = Size
                .wrapping_add(PaletteEntryCount.wrapping_mul(core::mem::size_of::<RGBQUAD>() as DWORD));
        } else {
            Size = Size
                .wrapping_add(PaletteEntryCount.wrapping_mul(core::mem::size_of::<USHORT>() as DWORD));
        }
        Size = Size.wrapping_add(DataSize);

        NewBitmapInfo = RtlAllocateHeap(RtlGetProcessHeap(), 0, Size as SIZE_T) as LPBITMAPINFO;
        if NewBitmapInfo.is_null() {
            return core::ptr::null_mut();
        }

        (*NewBitmapInfo).bmiHeader.biSize = core::mem::size_of::<BITMAPINFOHEADER>() as DWORD;
        (*NewBitmapInfo).bmiHeader.biWidth = (*CoreBitmapInfo).bmciHeader.bcWidth as LONG;
        (*NewBitmapInfo).bmiHeader.biHeight = (*CoreBitmapInfo).bmciHeader.bcHeight as LONG;
        (*NewBitmapInfo).bmiHeader.biPlanes = (*CoreBitmapInfo).bmciHeader.bcPlanes;
        (*NewBitmapInfo).bmiHeader.biBitCount = (*CoreBitmapInfo).bmciHeader.bcBitCount;
        (*NewBitmapInfo).bmiHeader.biCompression = BI_RGB;
        (*NewBitmapInfo).bmiHeader.biSizeImage = 0;
        (*NewBitmapInfo).bmiHeader.biXPelsPerMeter = 0;
        (*NewBitmapInfo).bmiHeader.biYPelsPerMeter = 0;
        (*NewBitmapInfo).bmiHeader.biClrUsed = 0;
        (*NewBitmapInfo).bmiHeader.biClrImportant = 0;

        if PaletteEntryCount != 0 {
            if ColorSpec == DIB_RGB_COLORS {
                let mut Index: ULONG = 0;
                while Index < PaletteEntryCount {
                    let dst = (*NewBitmapInfo).bmiColors.as_mut_ptr().add(Index as usize);
                    let src = (*CoreBitmapInfo).bmciColors.as_ptr().add(Index as usize);
                    (*dst).rgbRed = (*src).rgbtRed;
                    (*dst).rgbGreen = (*src).rgbtGreen;
                    (*dst).rgbBlue = (*src).rgbtBlue;
                    (*dst).rgbReserved = 0;
                    Index += 1;
                }
            } else {
                core::ptr::copy_nonoverlapping(
                    (*CoreBitmapInfo).bmciColors.as_ptr() as *const u8,
                    (*NewBitmapInfo).bmiColors.as_mut_ptr() as *mut u8,
                    (PaletteEntryCount as usize) * core::mem::size_of::<USHORT>(),
                );
            }
        }

        if FollowedByData != FALSE {
            let NewDataPtr: ULONG_PTR;
            let OldDataPtr: ULONG_PTR;

            if ColorSpec == DIB_RGB_COLORS {
                NewDataPtr = (*NewBitmapInfo)
                    .bmiColors
                    .as_mut_ptr()
                    .add(PaletteEntryCount as usize) as ULONG_PTR;
                OldDataPtr = (*CoreBitmapInfo)
                    .bmciColors
                    .as_ptr()
                    .add(PaletteEntryCount as usize) as ULONG_PTR;
            } else {
                NewDataPtr = ((*NewBitmapInfo).bmiColors.as_mut_ptr() as ULONG_PTR)
                    .wrapping_add(
                        (PaletteEntryCount as ULONG_PTR)
                            .wrapping_mul(core::mem::size_of::<USHORT>() as ULONG_PTR),
                    );
                OldDataPtr = ((*CoreBitmapInfo).bmciColors.as_ptr() as ULONG_PTR)
                    .wrapping_add(
                        (PaletteEntryCount as ULONG_PTR)
                            .wrapping_mul(core::mem::size_of::<USHORT>() as ULONG_PTR),
                    );
            }

            core::ptr::copy_nonoverlapping(
                OldDataPtr as *const u8,
                NewDataPtr as *mut u8,
                DataSize as usize,
            );
        }
    } else {
        // Verify some data validity.
        match (*BitmapInfo).bmiHeader.biCompression {
            x if x == BI_RLE8 => {
                if (*BitmapInfo).bmiHeader.biBitCount != 8 {
                    return core::ptr::null_mut();
                }
                if (*BitmapInfo).bmiHeader.biHeight < 0 {
                    return core::ptr::null_mut();
                }
            }
            x if x == BI_RLE4 => {
                if (*BitmapInfo).bmiHeader.biBitCount != 4 {
                    return core::ptr::null_mut();
                }
                if (*BitmapInfo).bmiHeader.biHeight < 0 {
                    return core::ptr::null_mut();
                }
            }
            _ => {}
        }

        // Non "standard" formats must have a valid size set.
        if (*BitmapInfo).bmiHeader.biCompression != BI_RGB
            && (*BitmapInfo).bmiHeader.biCompression != BI_BITFIELDS
        {
            if (*BitmapInfo).bmiHeader.biSizeImage == 0 {
                return core::ptr::null_mut();
            }
        }
    }

    Size = (*NewBitmapInfo).bmiHeader.biSize;
    if ColorSpec == DIB_RGB_COLORS {
        Size = Size
            .wrapping_add(PaletteEntryCount.wrapping_mul(core::mem::size_of::<RGBQUAD>() as DWORD));
    } else {
        Size = Size
            .wrapping_add(PaletteEntryCount.wrapping_mul(core::mem::size_of::<USHORT>() as DWORD));
    }
    Size = Size.wrapping_add(DataSize);
    *BitmapInfoSize = Size;

    NewBitmapInfo
}

#[no_mangle]
pub unsafe extern "system" fn LogFontA2W(pW: LPLOGFONTW, pA: *const LOGFONTA) {
    // COPYN(f): pW->f = pA->f
    (*pW).lfHeight = (*pA).lfHeight;
    (*pW).lfWidth = (*pA).lfWidth;
    (*pW).lfEscapement = (*pA).lfEscapement;
    (*pW).lfOrientation = (*pA).lfOrientation;
    (*pW).lfWeight = (*pA).lfWeight;
    (*pW).lfItalic = (*pA).lfItalic;
    (*pW).lfUnderline = (*pA).lfUnderline;
    (*pW).lfStrikeOut = (*pA).lfStrikeOut;
    (*pW).lfCharSet = (*pA).lfCharSet;
    (*pW).lfOutPrecision = (*pA).lfOutPrecision;
    (*pW).lfClipPrecision = (*pA).lfClipPrecision;
    (*pW).lfQuality = (*pA).lfQuality;
    (*pW).lfPitchAndFamily = (*pA).lfPitchAndFamily;

    // COPYS(lfFaceName, LF_FACESIZE):
    //   MultiByteToWideChar(CP_THREAD_ACP, 0, pA->f, len, pW->f, len)
    MultiByteToWideChar(
        CP_THREAD_ACP,
        0,
        (*pA).lfFaceName.as_ptr() as LPCSTR,
        LF_FACESIZE as INT,
        (*pW).lfFaceName.as_mut_ptr(),
        LF_FACESIZE as INT,
    );
    (*pW).lfFaceName[(LF_FACESIZE - 1) as usize] = 0;
}

#[no_mangle]
pub unsafe extern "system" fn LogFontW2A(pA: LPLOGFONTA, pW: *const LOGFONTW) {
    // COPYN(f): pA->f = pW->f
    (*pA).lfHeight = (*pW).lfHeight;
    (*pA).lfWidth = (*pW).lfWidth;
    (*pA).lfEscapement = (*pW).lfEscapement;
    (*pA).lfOrientation = (*pW).lfOrientation;
    (*pA).lfWeight = (*pW).lfWeight;
    (*pA).lfItalic = (*pW).lfItalic;
    (*pA).lfUnderline = (*pW).lfUnderline;
    (*pA).lfStrikeOut = (*pW).lfStrikeOut;
    (*pA).lfCharSet = (*pW).lfCharSet;
    (*pA).lfOutPrecision = (*pW).lfOutPrecision;
    (*pA).lfClipPrecision = (*pW).lfClipPrecision;
    (*pA).lfQuality = (*pW).lfQuality;
    (*pA).lfPitchAndFamily = (*pW).lfPitchAndFamily;

    // COPYS(lfFaceName, LF_FACESIZE):
    //   WideCharToMultiByte(CP_THREAD_ACP, 0, pW->f, len, pA->f, len, NULL, NULL)
    WideCharToMultiByte(
        CP_THREAD_ACP,
        0,
        (*pW).lfFaceName.as_ptr(),
        LF_FACESIZE as INT,
        (*pA).lfFaceName.as_mut_ptr(),
        LF_FACESIZE as INT,
        core::ptr::null(),
        core::ptr::null_mut(),
    );
    (*pA).lfFaceName[(LF_FACESIZE - 1) as usize] = 0;
}

#[no_mangle]
pub unsafe extern "system" fn EnumLogFontExW2A(
    fontA: LPENUMLOGFONTEXA,
    fontW: *const ENUMLOGFONTEXW,
) {
    LogFontW2A(fontA as LPLOGFONTA, fontW as *const LOGFONTW);

    WideCharToMultiByte(
        CP_THREAD_ACP,
        0,
        (*fontW).elfFullName.as_ptr(),
        -1,
        (*fontA).elfFullName.as_mut_ptr() as LPSTR,
        LF_FULLFACESIZE as INT,
        core::ptr::null(),
        core::ptr::null_mut(),
    );
    (*fontA).elfFullName[(LF_FULLFACESIZE - 1) as usize] = 0;
    WideCharToMultiByte(
        CP_THREAD_ACP,
        0,
        (*fontW).elfStyle.as_ptr(),
        -1,
        (*fontA).elfStyle.as_mut_ptr() as LPSTR,
        LF_FACESIZE as INT,
        core::ptr::null(),
        core::ptr::null_mut(),
    );
    (*fontA).elfStyle[(LF_FACESIZE - 1) as usize] = 0;
    WideCharToMultiByte(
        CP_THREAD_ACP,
        0,
        (*fontW).elfScript.as_ptr(),
        -1,
        (*fontA).elfScript.as_mut_ptr() as LPSTR,
        LF_FACESIZE as INT,
        core::ptr::null(),
        core::ptr::null_mut(),
    );
    (*fontA).elfScript[(LF_FACESIZE - 1) as usize] = 0;
}

/// LPK.DLL loader function.
///
/// Returns TRUE if a valid parameter was passed and loading was successful, FALSE otherwise.
#[no_mangle]
pub unsafe extern "system" fn LoadLPK(LpkFunctionID: INT) -> BOOL {
    // L"lpk.dll"
    const LPK_DLL: [WCHAR; 8] = [
        b'l' as WCHAR,
        b'p' as WCHAR,
        b'k' as WCHAR,
        b'.' as WCHAR,
        b'd' as WCHAR,
        b'l' as WCHAR,
        b'l' as WCHAR,
        0,
    ];

    if hLpk.is_null() {
        // Check if the DLL is already loaded
        hLpk = LoadLibraryW(LPK_DLL.as_ptr());
    }

    if !hLpk.is_null() {
        match LpkFunctionID {
            LPK_INIT => TRUE,

            LPK_ETO => {
                if (*(&raw const LpkExtTextOut)).is_none() {
                    // Check if the function is already loaded
                    LpkExtTextOut = core::mem::transmute::<FARPROC, LPKETO>(GetProcAddress(
                        hLpk,
                        b"LpkExtTextOut\0".as_ptr() as LPCSTR,
                    ));
                }

                if (*(&raw const LpkExtTextOut)).is_none() {
                    FreeLibrary(hLpk);
                    return FALSE;
                }

                TRUE
            }

            LPK_GCP => {
                if (*(&raw const LpkGetCharacterPlacement)).is_none() {
                    // Check if the function is already loaded
                    LpkGetCharacterPlacement = core::mem::transmute::<FARPROC, LPKGCP>(
                        GetProcAddress(hLpk, b"LpkGetCharacterPlacement\0".as_ptr() as LPCSTR),
                    );
                }

                if (*(&raw const LpkGetCharacterPlacement)).is_none() {
                    FreeLibrary(hLpk);
                    return FALSE;
                }

                TRUE
            }

            LPK_GTEP => {
                if (*(&raw const LpkGetTextExtentExPoint)).is_none() {
                    // Check if the function is already loaded
                    LpkGetTextExtentExPoint = core::mem::transmute::<FARPROC, LPKGTEP>(
                        GetProcAddress(hLpk, b"LpkGetTextExtentExPoint\0".as_ptr() as LPCSTR),
                    );
                }

                if (*(&raw const LpkGetTextExtentExPoint)).is_none() {
                    FreeLibrary(hLpk);
                    return FALSE;
                }

                TRUE
            }

            _ => {
                FreeLibrary(hLpk);
                FALSE
            }
        }
    } else {
        FALSE
    }
}
