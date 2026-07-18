//! objects/metafile.c → Rust. Classic-metafile (WMF) client API.
//
// Wave-3b. Same crate/substrate as Waves 1a/1b/2/2b/3a. Imports:
// `win32k_sys::win32k::*` + `crate::gdihelp::*`.
//
// NOTE: metafile.c contains exactly 5 functions —
//   GdiIsPlayMetafileDC, GdiIsMetaFileDC, GdiIsMetaPrintDC,
//   GdiCreateLocalMetaFilePict, GdiConvertMetaFilePict.
//   (The generic GetMetaFileA/W / PlayMetaFile / CopyMetaFileA/W / DeleteMetaFile /
//    SetMetaFileBitsEx / GetMetaFileBitsEx wrappers are NOT in this file — they live in
//    the gdi32 wine/ sublib. They appear here only as cross-file `extern` callees.)
//
// NEEDS-IMPORT (constants absent from win32k_sys bindgen → local consts, psdk/ntgdityp.h values):
//   GHND (GMEM_MOVEABLE|GMEM_ZEROINIT = 0x0042), GMEM_SHARE (0x2000), LPTR (0x0040);
//   GDITAG_TYPE_EMF ('XEFM' = 0x5845464D), GDITAG_TYPE_MFP ('_PFM' = 0x5F50464D)
//   from win32ss/include/ntgdityp.h (multi-char DWORD tags; big-endian char order).
// NEEDS-CHECK: cross-file ABI — gdihelp::GdiGetDcAttr (in-crate). GdiGetLDC is gdi32-private
//   FASTCALL (misc.rs) → extern "fastcall". The metafile/EMF/coord client wrappers
//   (SetMetaFileBitsEx/GetMetaFileBitsEx/SetEnhMetaFileBits/GetEnhMetaFileHeader/
//   DeleteEnhMetaFile/GetWinMetaFileBits/CreateDCW/DeleteDC) and kernel32 Global*/Local*
//   are all WINAPI/cdecl-on-the-export-table → extern "system". NtGdi* from win32k_sys glob.

use win32k_sys::win32k::*;
use crate::gdihelp::*;
use crate::dcattr_f;

// win32k_sys FALSE/TRUE are bindgen u32; BOOL is i32 — BOOL-typed shadows so
// `return FALSE`/`return TRUE` type-check against the i32 BOOL returns.
const FALSE: BOOL = 0;
const TRUE: BOOL = 1;

// ── GlobalAlloc/LocalAlloc flags (winbase.h) — absent from win32k_sys bindgen ────
const GHND: UINT = 0x0042; // GMEM_MOVEABLE | GMEM_ZEROINIT
const GMEM_SHARE: UINT = 0x2000;
const LPTR: UINT = 0x0040; // LMEM_FIXED | LMEM_ZEROINIT

// ── GDI server-metafile tags (win32ss/include/ntgdityp.h) — multi-char DWORD ─────
const GDITAG_TYPE_EMF: DWORD = 0x5845_464D; // 'XEFM' — EnhMetaFile
const GDITAG_TYPE_MFP: DWORD = 0x5F50_464D; // '_PFM' — MetaFile Picture

// ── Cross-file fns ───────────────────────────────────────────────────────────────
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn GdiGetLDC(hdc: HDC) -> PLDC; // gdi32-private (misc.rs)
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn GdiGetLDC(hdc: HDC) -> PLDC; // gdi32-private (misc.rs)
}

extern "system" {
    // kernel32
    fn SetLastError(dwErrCode: DWORD);
    fn GlobalAlloc(uFlags: UINT, dwBytes: SIZE_T) -> HGLOBAL;
    fn GlobalLock(hMem: HGLOBAL) -> LPVOID;
    fn GlobalUnlock(hMem: HGLOBAL) -> BOOL;
    fn GlobalFree(hMem: HGLOBAL) -> HGLOBAL;
    fn LocalAlloc(uFlags: UINT, uBytes: SIZE_T) -> HLOCAL;
    fn LocalFree(hMem: HLOCAL) -> HLOCAL;
    // gdi32 (other objects / wine sublib)
    fn SetMetaFileBitsEx(cbBuffer: UINT, lpData: *const BYTE) -> HMETAFILE;
    fn GetMetaFileBitsEx(hMF: HMETAFILE, cbBuffer: UINT, lpData: LPVOID) -> UINT;
    fn SetEnhMetaFileBits(cbBuffer: UINT, lpData: *const BYTE) -> HENHMETAFILE;
    fn GetEnhMetaFileHeader(hemf: HENHMETAFILE, cbBuffer: UINT, lpemh: LPENHMETAHEADER) -> UINT;
    fn DeleteEnhMetaFile(hemf: HENHMETAFILE) -> BOOL;
    fn GetWinMetaFileBits(
        hemf: HENHMETAFILE,
        cbData16: UINT,
        pData16: LPBYTE,
        iMapMode: INT,
        hdcRef: HDC,
    ) -> UINT;
    fn CreateDCW(
        pwszDriver: LPCWSTR,
        pwszDevice: LPCWSTR,
        pszPort: LPCWSTR,
        pdm: *const DEVMODEW,
    ) -> HDC;
    fn DeleteDC(hdc: HDC) -> BOOL;
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GdiIsPlayMetafileDC(hdc: HDC) -> BOOL {
    let pdcattr: PDC_ATTR = GdiGetDcAttr(hdc);

    if !pdcattr.is_null() {
        return (((*dcattr_f!(pdcattr, ulDirty_)) & DC_PLAYMETAFILE) != 0) as BOOL;
    }
    FALSE
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GdiIsMetaFileDC(hdc: HDC) -> BOOL {
    let ulObjType: ULONG;

    ulObjType = GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) as ULONG;
    if ulObjType == GDILoObjType_LO_METADC16_TYPE as ULONG {
        return TRUE;
    }

    if ulObjType == GDILoObjType_LO_ALTDC_TYPE as ULONG {
        let pLDC: PLDC = GdiGetLDC(hdc);
        if pLDC.is_null() {
            SetLastError(ERROR_INVALID_HANDLE);
            return FALSE;
        }
        return ((*pLDC).iType == LDC_EMFLDC as INT) as BOOL;
    }

    FALSE
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GdiIsMetaPrintDC(hdc: HDC) -> BOOL {
    let hType: ULONG = GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) as ULONG;

    if hType != GDILoObjType_LO_DC_TYPE as ULONG {
        if hType == GDILoObjType_LO_METADC16_TYPE as ULONG {
            return FALSE;
        } else {
            let pLDC: PLDC = GdiGetLDC(hdc);
            if pLDC.is_null() {
                SetLastError(ERROR_INVALID_HANDLE);
                return FALSE;
            }
            return (((*pLDC).Flags & LDC_META_PRINT) != 0) as BOOL;
        }
    }
    FALSE
}

// NOTE: I wanna use GdiCreateLocalMetaFilePict and GdiConvertMetaFilePict
//       functions for clipboard data conversion. --- katahiromz

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GdiCreateLocalMetaFilePict(hmo: HANDLE) -> HGLOBAL {
    let mut hMetaFilePict: HGLOBAL;
    let pInfo: *mut METAFILEPICT;
    let mut hMF: HMETAFILE = core::ptr::null_mut();
    let mut Buffer: *mut BYTE = core::ptr::null_mut();
    let mut BufNew: *mut BYTE = core::ptr::null_mut();
    let mut hDC: HDC;
    let mut nSize: UINT;
    let cSize: UINT;
    let mut iType: DWORD = 0;

    // NOTE: On Win32, there is no difference between the local heap and
    //       the global heap. GlobalAlloc and LocalAlloc have same effect.

    // allocate for METAFILEPICT
    hMetaFilePict = GlobalAlloc(GHND | GMEM_SHARE, core::mem::size_of::<METAFILEPICT>() as SIZE_T);
    pInfo = GlobalLock(hMetaFilePict) as *mut METAFILEPICT;

    'exit: {
        if pInfo.is_null() {
            break 'exit;
        }

        nSize = NtGdiGetServerMetaFileBits(
            hmo,
            0,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        ) as UINT;

        // allocate buffer
        Buffer = LocalAlloc(LPTR, nSize as SIZE_T) as *mut BYTE;
        if Buffer.is_null() {
            break 'exit;
        }

        // store to buffer
        nSize = NtGdiGetServerMetaFileBits(
            hmo,
            nSize as ULONG,
            Buffer as LPBYTE,
            &mut iType,
            &mut (*pInfo).mm as *mut LONG as PDWORD,
            &mut (*pInfo).xExt as *mut LONG as PDWORD,
            &mut (*pInfo).yExt as *mut LONG as PDWORD,
        ) as UINT;
        if nSize == 0 {
            break 'exit;
        }

        if iType == GDITAG_TYPE_EMF {
            // handle conversion to MFP
            static SZ_DISPLAY_W: [WCHAR; 8] = [
                b'D' as WCHAR,
                b'I' as WCHAR,
                b'S' as WCHAR,
                b'P' as WCHAR,
                b'L' as WCHAR,
                b'A' as WCHAR,
                b'Y' as WCHAR,
                0,
            ];
            let hEMF: HENHMETAFILE;
            let mut emh: ENHMETAHEADER = core::mem::zeroed();

            hEMF = SetEnhMetaFileBits(nSize, Buffer as *const BYTE);
            if hEMF.is_null() {
                break 'exit;
            }

            if GetEnhMetaFileHeader(hEMF, core::mem::size_of::<ENHMETAHEADER>() as UINT, &mut emh)
                == 0
            {
                DeleteEnhMetaFile(hEMF);
                break 'exit;
            }

            (*pInfo).mm = MM_ANISOTROPIC as LONG; // wine uses MM_ISOTROPIC.
            (*pInfo).xExt = emh.rclFrame.right - emh.rclFrame.left; // Width
            (*pInfo).yExt = emh.rclFrame.bottom - emh.rclFrame.top; // Height

            hDC = CreateDCW(
                SZ_DISPLAY_W.as_ptr(),
                core::ptr::null(),
                core::ptr::null(),
                core::ptr::null(),
            );
            if !hDC.is_null() {
                cSize = GetWinMetaFileBits(
                    hEMF,
                    0,
                    core::ptr::null_mut(),
                    MM_ANISOTROPIC as INT,
                    hDC,
                );
                if cSize != 0 {
                    BufNew = LocalAlloc(LPTR, cSize as SIZE_T) as *mut BYTE;
                    if !BufNew.is_null() {
                        nSize = GetWinMetaFileBits(
                            hEMF,
                            cSize,
                            BufNew as LPBYTE,
                            MM_ANISOTROPIC as INT,
                            hDC,
                        );
                        if nSize == cSize {
                            if !Buffer.is_null() {
                                LocalFree(Buffer as HLOCAL);
                            }
                            Buffer = BufNew;
                        }
                    }
                }
                DeleteDC(hDC);
            }
            DeleteEnhMetaFile(hEMF);

            if Buffer != BufNew {
                break 'exit;
            }
        }

        // create metafile from buffer
        hMF = SetMetaFileBitsEx(nSize, Buffer as *const BYTE);
        if hMF.is_null() {
            break 'exit;
        }

        // set metafile handle
        (*pInfo).hMF = hMF;
    }

    // Exit: clean up
    if !Buffer.is_null() {
        LocalFree(Buffer as HLOCAL);
    }
    if !pInfo.is_null() {
        GlobalUnlock(hMetaFilePict);
    }
    if hMF.is_null() {
        // failure
        GlobalFree(hMetaFilePict);
        hMetaFilePict = core::ptr::null_mut();
    }

    hMetaFilePict // success if non-NULL
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GdiConvertMetaFilePict(hMetaFilePict: HGLOBAL) -> HANDLE {
    let hMF: HMETAFILE;
    let mut nSize: UINT;
    let mut hmo: HANDLE = core::ptr::null_mut();
    let mut Buffer: *mut BYTE = core::ptr::null_mut();
    let pInfo: *mut METAFILEPICT;

    // get METAFILEPICT pointer
    pInfo = GlobalLock(hMetaFilePict) as *mut METAFILEPICT;

    'exit: {
        if pInfo.is_null() {
            break 'exit;
        }

        // get metafile handle
        hMF = (*pInfo).hMF;

        // get size of buffer
        nSize = GetMetaFileBitsEx(hMF, 0, core::ptr::null_mut());
        if nSize == 0 {
            break 'exit;
        }

        // allocate buffer
        Buffer = LocalAlloc(LPTR, nSize as SIZE_T) as *mut BYTE;
        if Buffer.is_null() {
            break 'exit;
        }

        // store to buffer
        nSize = GetMetaFileBitsEx(hMF, nSize, Buffer as LPVOID);
        if nSize == 0 {
            break 'exit;
        }

        hmo = NtGdiCreateServerMetaFile(
            GDITAG_TYPE_MFP,
            nSize as ULONG,
            Buffer as LPBYTE,
            (*pInfo).mm as DWORD,
            (*pInfo).xExt as DWORD,
            (*pInfo).yExt as DWORD,
        );
    }

    // Exit: clean up
    if !pInfo.is_null() {
        GlobalUnlock(hMetaFilePict);
    }
    if !Buffer.is_null() {
        LocalFree(Buffer as HLOCAL);
    }
    hmo // success if non-NULL
}
