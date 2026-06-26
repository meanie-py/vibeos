//! objects/enhmfile.c → Rust. Enhanced-metafile (EMF) client API.
//
// Wave-3b. Same crate/substrate as Waves 1a/1b/2/2b/3a (all landed/in-crate).
// Imports: `win32k_sys::win32k::*` + `crate::gdihelp::*`.
//
// NEEDS-IMPORT: GDITAG_TYPE_EMF / GDITAG_TYPE_MFP are NOT in win32k-sys bindgen — they are
//   FOURCC-style multi-char literals from win32ss/include/ntgdityp.h. Reproduced here as local
//   consts with the exact byte values: 'XEFM' = 0x5845464D, '_PFM' = 0x5F50464D.
// NEEDS-IMPORT: LPTR (winbase.h LMEM_FIXED|LMEM_ZEROINIT) = 0x0040 — local const.
// NEEDS-CHECK: GDI_HANDLE_GET_TYPE returns usize; compared against
//   `GDILoObjType_GDILoObjType_LO_ALTDC_TYPE as usize` (the real bindgen const, value 0x210000).
// NEEDS-CHECK: cross-file ABI. gdi32 client EMF wrappers (GetEnhMetaFileBits / GetEnhMetaFileHeader
//   / SetEnhMetaFileBits / SetWinMetaFileBits) are WINAPI → extern "system". kernel32 LocalAlloc /
//   LocalFree are WINAPI → extern "system". EMFDC_GdiComment is a plain-cdecl EMF recorder (wine/
//   sublib / dc.c) → extern "C". NtGdi* come from the win32k_sys glob (stdcall).
// NEEDS-CHECK: GdiCreateLocalEnhMetaFile — the C declares `hEMF` uninitialized and returns it on
//   the early `nSize == 0` goto (a latent C bug: returns garbage). Rust forces initialization, so
//   hEMF is seeded to NULL; behavior is identical on every reachable success path, and the early-
//   exit now deterministically returns NULL instead of stack garbage.

use win32k_sys::win32k::*;
use crate::gdihelp::*;

// ── Constants absent from win32k_sys bindgen ────────────────────────────────────
// win32ss/include/ntgdityp.h: GDITAG_TYPE_EMF 'XEFM', GDITAG_TYPE_MFP '_PFM'.
const GDITAG_TYPE_EMF: DWORD = 0x5845_464D; // 'XEFM' (EnhMetaFile)
const GDITAG_TYPE_MFP: DWORD = 0x5F50_464D; // '_PFM' (MetaFile Picture)
// winbase.h: LPTR = LMEM_FIXED (0x0000) | LMEM_ZEROINIT (0x0040)
const LPTR: UINT = 0x0040;

// ── Cross-file gdi32 client / kernel32 functions (WINAPI → extern "system") ─────
extern "system" {
    fn SetLastError(dwErrCode: DWORD);
    // kernel32
    fn LocalAlloc(uFlags: UINT, uBytes: SIZE_T) -> HLOCAL;
    fn LocalFree(hMem: HLOCAL) -> HLOCAL;
    // gdi32 EMF client (other objects/enhmfile-family .c)
    fn GetEnhMetaFileBits(hEMF: HENHMETAFILE, nSize: UINT, lpData: LPBYTE) -> UINT;
    fn GetEnhMetaFileHeader(hemf: HENHMETAFILE, nSize: UINT, lpEnhMetaHeader: LPENHMETAHEADER)
        -> UINT;
    fn SetEnhMetaFileBits(nSize: UINT, lpData: *const BYTE) -> HENHMETAFILE;
    fn SetWinMetaFileBits(
        nSize: UINT,
        lpMeta16Data: *const BYTE,
        hdcRef: HDC,
        lpMFP: *const METAFILEPICT,
    ) -> HENHMETAFILE;
}

// ── EMF recorder (wine/emfdc.c) — declared `BOOL WINAPI EMFDC_GdiComment` → stdcall
// (`_EMFDC_GdiComment@12` on i386), NOT cdecl. extern "system" matches the wine def.
extern "system" {
    fn EMFDC_GdiComment(hdc: HDC, cbSize: UINT, lpData: *const BYTE) -> BOOL;
}

/// `UlongToPtr(ul)` (basetsd.h) — `(PVOID)(ULONG_PTR)(ULONG)ul`.
#[inline]
fn UlongToPtr(ul: ULONG) -> PVOID {
    ul as usize as PVOID
}

/*
 * @unimplemented
 */
#[no_mangle]
pub unsafe extern "system" fn IsValidEnhMetaRecord(_pv0: PVOID, _pv1: PVOID) -> BOOL {
    // UNIMPLEMENTED;
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    FALSE as BOOL
}

/*
 * @unimplemented
 */
#[no_mangle]
pub unsafe extern "system" fn IsValidEnhMetaRecordOffExt(
    _pv0: PVOID,
    _pv1: PVOID,
    _dwOffset: DWORD,
    _dwExtends: DWORD,
) -> BOOL {
    // UNIMPLEMENTED;
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    FALSE as BOOL
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GdiConvertEnhMetaFile(hemf: HENHMETAFILE) -> HANDLE {
    let mut hmo: HANDLE = core::ptr::null_mut();
    let mut Buffer: *mut BYTE = core::ptr::null_mut();
    let mut nSize: UINT;

    'exit: {
        nSize = GetEnhMetaFileBits(hemf, 0, core::ptr::null_mut());
        if nSize == 0 {
            break 'exit;
        }

        // allocate buffer
        Buffer = LocalAlloc(LPTR, nSize as SIZE_T) as *mut BYTE;
        if Buffer.is_null() {
            break 'exit;
        }

        nSize = GetEnhMetaFileBits(hemf, nSize, Buffer);
        if nSize == 0 {
            break 'exit;
        }

        hmo = NtGdiCreateServerMetaFile(GDITAG_TYPE_EMF, nSize, Buffer, 0, 0, 0);
    }

    // clean up
    if !Buffer.is_null() {
        LocalFree(Buffer as HLOCAL);
    }
    hmo
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GdiCreateLocalEnhMetaFile(hmo: HANDLE) -> HENHMETAFILE {
    // C declares hEMF uninitialized and returns it on the early nSize==0 exit (latent C bug).
    // Rust requires initialization; seed to NULL (see NEEDS-CHECK at top).
    let mut hEMF: HENHMETAFILE = core::ptr::null_mut();
    let mut Buffer: *mut BYTE = core::ptr::null_mut();
    let mut nSize: UINT;
    let mut iType: DWORD = 0;
    let mut mm: DWORD = 0;
    let mut xExt: DWORD = 0;
    let mut yExt: DWORD = 0;

    'exit: {
        nSize = NtGdiGetServerMetaFileBits(
            hmo,
            0,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        if nSize == 0 {
            break 'exit;
        }

        // allocate buffer
        Buffer = LocalAlloc(LPTR, nSize as SIZE_T) as *mut BYTE;
        if Buffer.is_null() {
            break 'exit;
        }

        // store to buffer
        nSize = NtGdiGetServerMetaFileBits(
            hmo,
            nSize,
            Buffer,
            &mut iType,
            &mut mm,
            &mut xExt,
            &mut yExt,
        );
        if nSize == 0 {
            break 'exit;
        }

        if iType == GDITAG_TYPE_MFP {
            // handle conversion to EMF
            let mut Info: METAFILEPICT = core::mem::zeroed();

            Info.hMF = core::ptr::null_mut();
            Info.mm = mm as LONG;
            Info.xExt = xExt as LONG;
            Info.yExt = yExt as LONG;

            // Translate from old style to new style.
            hEMF = SetWinMetaFileBits(nSize, Buffer, core::ptr::null_mut(), &Info);
            if hEMF.is_null() {
                break 'exit;
            }
        } else {
            hEMF = SetEnhMetaFileBits(nSize, Buffer);
            if hEMF.is_null() {
                break 'exit;
            }
        }
    }

    // clean up
    if !Buffer.is_null() {
        LocalFree(Buffer as HLOCAL);
    }
    hEMF
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GdiComment(hdc: HDC, cbSize: UINT, lpData: *const BYTE) -> BOOL {
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != GDILoObjType_GDILoObjType_LO_ALTDC_TYPE as usize {
        return TRUE as BOOL;
    }

    EMFDC_GdiComment(hdc, cbSize, lpData)
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetEnhMetaFilePixelFormat(
    hemf: HENHMETAFILE,
    cbBuffer: UINT,
    ppfd: *mut PIXELFORMATDESCRIPTOR,
) -> UINT {
    let mut pemh: ENHMETAHEADER = core::mem::zeroed();

    if GetEnhMetaFileHeader(
        hemf,
        core::mem::size_of::<ENHMETAHEADER>() as UINT,
        &mut pemh,
    ) != 0
    {
        if pemh.bOpenGL != 0 {
            if pemh.cbPixelFormat != 0 {
                core::ptr::copy_nonoverlapping(
                    UlongToPtr(pemh.offPixelFormat) as *const u8,
                    ppfd as *mut u8,
                    cbBuffer as usize,
                );
                return pemh.cbPixelFormat;
            }
        }
    }
    0
}
