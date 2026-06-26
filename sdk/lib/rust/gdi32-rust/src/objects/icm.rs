//! objects/icm.c → Rust. ICM / color-space client API.
//
// Color-management thin wrappers + A/W string conversion. Mostly UNIMPLEMENTED stubs
// in the C original (faithfully preserved as set-last-error returns).
//
// Imports per Wave-1b/2b contract: win32k_sys glob (scalars + NtGdi* + GDI types) and
// crate::gdihelp (handle macros). NOT crate::types (E0659 collision).
//
// NEEDS-IMPORT: cross-file fns this file calls (decorations matched to their C prototypes):
//     kernel32  VOID WINAPI SetLastError(DWORD)                          → extern "system"
//     kernel32  int  WINAPI WideCharToMultiByte(...)                     → extern "system"
//     ntdll     NTSTATUS NTAPI RtlMultiByteToUnicodeN(...)               → extern "system" (i386 NTAPI=stdcall)
//     CRT       size_t strlen(const char*)                              → extern "C"
//     gdi32     BOOL GdiGetHandleUserData(HGDIOBJ,DWORD,PVOID*)          → extern "C" (misc.rs, cdecl)
//     win32k    HANDLE NtGdiCreateColorSpace(PLOGCOLORSPACEEXW)          → from win32k_sys glob
//     win32k    BOOL   NtGdiSetColorSpace(HDC,HCOLORSPACE)              → from win32k_sys glob
// NEEDS-CHECK: CP_ACP is not in win32k-sys bindgen → defined as a local const (UINT=0),
//   matching font.rs.
// NEEDS-CHECK: RtlMultiByteToUnicodeN passes MAX_PATH as the *byte* count of the unicode
//   dest (the original C quirk — faithfully preserved, not characters).
// NEEDS-CHECK: IntCreateColorSpaceW is FASTCALL and non-static in C → kept #[no_mangle]
//   extern "fastcall" (matches region.rs/text.rs house style; gdi32 is i386).

use win32k_sys::win32k::*;
use crate::gdihelp::*;

// win32k_sys's FALSE/TRUE are bindgen u32 (#define), but BOOL is i32 — shadow with
// BOOL-typed locals so `return FALSE`/`= TRUE` type-check against the i32 BOOL returns.
const FALSE: BOOL = 0;
const TRUE: BOOL = 1;

// CP_ACP (psdk value) — not present in win32k-sys bindgen; mirror font.rs.
const CP_ACP: UINT = 0;

// ── Cross-file functions (defined in other .c → other cohab .rs / C / import libs) ──
extern "system" {
    fn SetLastError(dwErrCode: DWORD);
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
    fn RtlMultiByteToUnicodeN(
        UnicodeString: *mut WCHAR,
        MaxBytesInUnicodeString: ULONG,
        BytesInUnicodeString: *mut ULONG,
        MultiByteString: *const CHAR,
        BytesInMultiByteString: ULONG,
    ) -> NTSTATUS;
}

extern "C" {
    fn strlen(s: *const CHAR) -> usize;
    fn GdiGetHandleUserData(hGdiObj: HGDIOBJ, ObjectType: DWORD, UserData: *mut PVOID) -> BOOL;
}

#[no_mangle]
pub unsafe extern "fastcall" fn IntCreateColorSpaceW(
    lplcpw: LPLOGCOLORSPACEW,
    _Ascii: BOOL,
) -> HCOLORSPACE {
    let mut lcpeexw: LOGCOLORSPACEEXW = core::mem::zeroed();

    if ((*lplcpw).lcsSignature != LCS_SIGNATURE)
        || ((*lplcpw).lcsVersion != 0x400)
        || ((*lplcpw).lcsSize as usize != core::mem::size_of::<LOGCOLORSPACEW>())
    {
        SetLastError(ERROR_INVALID_COLORSPACE);
        return core::ptr::null_mut();
    }
    // RtlCopyMemory(&lcpeexw.lcsColorSpace, lplcpw, sizeof(LOGCOLORSPACEW));
    core::ptr::copy_nonoverlapping(
        lplcpw,
        &mut lcpeexw.lcsColorSpace as *mut LOGCOLORSPACEW,
        1,
    );

    NtGdiCreateColorSpace(&mut lcpeexw) as HCOLORSPACE
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn CreateColorSpaceW(lplcpw: LPLOGCOLORSPACEW) -> HCOLORSPACE {
    IntCreateColorSpaceW(lplcpw, FALSE)
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn CreateColorSpaceA(lplcpa: LPLOGCOLORSPACEA) -> HCOLORSPACE {
    let mut lcpw: LOGCOLORSPACEW = core::mem::zeroed();

    if ((*lplcpa).lcsSignature != LCS_SIGNATURE)
        || ((*lplcpa).lcsVersion != 0x400)
        || ((*lplcpa).lcsSize as usize != core::mem::size_of::<LOGCOLORSPACEA>())
    {
        SetLastError(ERROR_INVALID_COLORSPACE);
        return core::ptr::null_mut();
    }

    lcpw.lcsSignature = (*lplcpa).lcsSignature;
    lcpw.lcsVersion = (*lplcpa).lcsVersion;
    lcpw.lcsSize = core::mem::size_of::<LOGCOLORSPACEW>() as DWORD;
    lcpw.lcsCSType = (*lplcpa).lcsCSType;
    lcpw.lcsIntent = (*lplcpa).lcsIntent;
    lcpw.lcsEndpoints = (*lplcpa).lcsEndpoints;
    lcpw.lcsGammaRed = (*lplcpa).lcsGammaRed;
    lcpw.lcsGammaGreen = (*lplcpa).lcsGammaGreen;
    lcpw.lcsGammaBlue = (*lplcpa).lcsGammaBlue;

    RtlMultiByteToUnicodeN(
        lcpw.lcsFilename.as_mut_ptr(),
        MAX_PATH,
        core::ptr::null_mut(),
        (*lplcpa).lcsFilename.as_ptr(),
        (strlen((*lplcpa).lcsFilename.as_ptr()) + 1) as ULONG,
    );

    IntCreateColorSpaceW(&mut lcpw, FALSE)
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetColorSpace(hDC: HDC) -> HCOLORSPACE {
    let mut pDc_Attr: PDC_ATTR = core::ptr::null_mut();

    if GdiGetHandleUserData(
        hDC as HGDIOBJ,
        GDI_OBJECT_TYPE_DC,
        &mut pDc_Attr as *mut PDC_ATTR as *mut PVOID,
    ) == 0
    {
        SetLastError(ERROR_INVALID_HANDLE);
        return core::ptr::null_mut();
    }
    (*pDc_Attr).hColorSpace
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn SetColorSpace(hDC: HDC, hCS: HCOLORSPACE) -> HCOLORSPACE {
    let rhCS: HCOLORSPACE = GetColorSpace(hDC);

    if GDI_HANDLE_GET_TYPE(hDC as HGDIOBJ) == GDILoObjType_GDILoObjType_LO_DC_TYPE as usize {
        if NtGdiSetColorSpace(hDC, hCS) != 0 {
            return rhCS;
        }
    }
    // #if 0 block (metafile/EMF path) — disabled in C, omitted.
    core::ptr::null_mut()
}

/*
 * @unimplemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetICMProfileA(
    hdc: HDC,
    pBufSize: LPDWORD,
    pszFilename: LPSTR,
) -> BOOL {
    let mut filenameW: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut buflen: DWORD = MAX_PATH;
    let mut ret: BOOL = FALSE;

    if hdc.is_null() || pBufSize.is_null() {
        return FALSE;
    }

    if GetICMProfileW(hdc, &mut buflen, filenameW.as_mut_ptr()) != 0 {
        let len: ULONG = WideCharToMultiByte(
            CP_ACP,
            0,
            filenameW.as_ptr(),
            -1,
            core::ptr::null_mut(),
            0,
            core::ptr::null(),
            core::ptr::null_mut(),
        ) as ULONG;

        if pszFilename.is_null() {
            *pBufSize = len;
            return FALSE;
        }

        if *pBufSize >= len {
            WideCharToMultiByte(
                CP_ACP,
                0,
                filenameW.as_ptr(),
                -1,
                pszFilename,
                *pBufSize as INT,
                core::ptr::null(),
                core::ptr::null_mut(),
            );
            ret = TRUE;
        } else {
            SetLastError(ERROR_INSUFFICIENT_BUFFER);
        }
        *pBufSize = len;
    }

    ret
}

/*
 * @unimplemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetICMProfileW(
    hdc: HDC,
    size: LPDWORD,
    filename: LPWSTR,
) -> BOOL {
    if hdc.is_null() || size.is_null() || filename.is_null() {
        return FALSE;
    }

    // UNIMPLEMENTED;
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    FALSE
}

/*
 * @unimplemented
 */
#[no_mangle]
pub unsafe extern "system" fn SetICMProfileA(_a0: HDC, _a1: LPSTR) -> BOOL {
    // UNIMPLEMENTED;
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    FALSE
}

/*
 * @unimplemented
 */
#[no_mangle]
pub unsafe extern "system" fn SetICMProfileW(_a0: HDC, _a1: LPWSTR) -> BOOL {
    // UNIMPLEMENTED;
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    FALSE
}

/*
 * @unimplemented
 */
#[no_mangle]
pub unsafe extern "system" fn EnumICMProfilesA(
    _a0: HDC,
    _a1: ICMENUMPROCA,
    _a2: LPARAM,
) -> INT {
    /*
     * FIXME - call NtGdiEnumICMProfiles with NULL for lpstrBuffer
     * to find out how big a buffer we need. Then allocate that buffer
     * and call NtGdiEnumICMProfiles again to have the buffer filled.
     *
     * Finally, step through the buffer ( MULTI-SZ recommended for format ),
     * and convert each string to ANSI, calling the user's callback function
     * until we run out of strings or the user returns FALSE
     */

    // UNIMPLEMENTED;
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    0
}

/*
 * @unimplemented
 */
#[no_mangle]
pub unsafe extern "system" fn EnumICMProfilesW(
    _hDC: HDC,
    _lpEnumICMProfilesFunc: ICMENUMPROCW,
    _lParam: LPARAM,
) -> INT {
    /*
     * FIXME - call NtGdiEnumICMProfiles with NULL for lpstrBuffer
     * to find out how big a buffer we need. Then allocate that buffer
     * and call NtGdiEnumICMProfiles again to have the buffer filled.
     *
     * Finally, step through the buffer ( MULTI-SZ recommended for format ),
     * and call the user's callback function until we run out of strings or
     * the user returns FALSE
     */
    // UNIMPLEMENTED;
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    0
}

/*
 * @unimplemented
 */
#[no_mangle]
pub unsafe extern "system" fn UpdateICMRegKeyA(
    _a0: DWORD,
    _a1: LPSTR,
    _a2: LPSTR,
    _a3: UINT,
) -> BOOL {
    // UNIMPLEMENTED;
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    FALSE
}

/*
 * @unimplemented
 */
#[no_mangle]
pub unsafe extern "system" fn UpdateICMRegKeyW(
    _a0: DWORD,
    _a1: LPWSTR,
    _a2: LPWSTR,
    _a3: UINT,
) -> BOOL {
    // UNIMPLEMENTED;
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    FALSE
}

/*
 * @unimplemented
 */
#[no_mangle]
pub unsafe extern "system" fn SetICMMode(_hdc: HDC, iEnableICM: INT) -> INT {
    /*FIXME:  Assume that ICM is always off, and cannot be turned on */
    if iEnableICM == ICM_OFF as INT {
        return ICM_OFF as INT;
    }
    if iEnableICM == ICM_ON as INT {
        return 0;
    }
    if iEnableICM == ICM_QUERY as INT {
        return ICM_OFF as INT;
    }

    // UNIMPLEMENTED;
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    0
}

/*
 * @unimplemented
 */
#[no_mangle]
pub unsafe extern "system" fn GdiConvertBitmapV5(
    _in_format_BitMap: HBITMAP,
    _src_BitMap: HBITMAP,
    _bpp: INT,
    _unuse: INT,
) -> HBITMAP {
    /* FIXME guessing the prototypes */

    /*
     * it have create a new bitmap with desired in format,
     * then convert it src_bitmap to new format
     * and return it as HBITMAP
     */

    // return FALSE;
    core::ptr::null_mut()
}
