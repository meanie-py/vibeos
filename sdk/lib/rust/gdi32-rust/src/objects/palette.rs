//! win32ss/gdi/gdi32/objects/palette.c → Rust. Palette client wrappers:
//! AnimatePalette / CreatePalette / GetPaletteEntries / SetPaletteEntries /
//! GetSystemPaletteEntries / GetDIBColorTable / RealizePalette / ResizePalette /
//! SetDIBColorTable / UpdateColors / ColorCorrectPalette.
//
// Imports per Wave-1b contract: win32k_sys glob (scalars + NtGdi* + GDI types) and
// crate::gdihelp (handle macros + NtCurrentTeb). NOT crate::types (E0659 collision).
//
// NEEDS-IMPORT: cross-file gdi32 client fns this file calls (decorations matched to
//   their C prototypes — flag for central integration if any differ):
//     wingdi (gdi32 export) int  WINAPI GetDeviceCaps(HDC,int)        → extern "system" (_GetDeviceCaps@8)
//     gdi32p.h               UINT WINAPI UserRealizePalette(HDC)       → extern "system" (_UserRealizePalette@4)
//     gdi32p.h (METADC_*)    BOOL        METADC_RealizePalette(HDC)    → extern "C" (plain cdecl, per contract)
//     kernel32               VOID WINAPI SetLastError(DWORD)           → extern "system" (_SetLastError@4)
//
// NEEDS-IMPORT: `NtCurrentTeb` referenced via `crate::gdihelp::NtCurrentTeb()` (TEB
//   substrate; NOT in win32k-sys bindgen). Mirrors gdiobj.rs.

use win32k_sys::win32k::*;
use crate::gdihelp::*;

// win32k_sys's FALSE/TRUE are bindgen u32 (#define), but BOOL is i32 — shadow with
// BOOL-typed locals so the `iFunc`/`bInbound` BOOL args and BOOL returns type-check.
const FALSE: BOOL = 0;
const TRUE: BOOL = 1;

// ── Cross-file gdi32 client functions (defined in other .c → other cohab .rs / C) ──
extern "system" {
    fn GetDeviceCaps(hdc: HDC, index: INT) -> INT;
    fn UserRealizePalette(hDC: HDC) -> UINT;
    fn SetLastError(dwErrCode: DWORD);
}
extern "C" {
    fn METADC_RealizePalette(hdc: HDC) -> BOOL;
}

/// number of fixed colors in system palette
const NB_RESERVED_COLORS: usize = 20;

const fn pe(r: BYTE, g: BYTE, b: BYTE, f: BYTE) -> PALETTEENTRY {
    PALETTEENTRY { peRed: r, peGreen: g, peBlue: b, peFlags: f }
}

static SYS_PAL_TEMPLATE: [PALETTEENTRY; NB_RESERVED_COLORS] = [
    /* first 10 entries in the system palette */
    /* red  green blue  flags */
    pe(0x00, 0x00, 0x00, 0),
    pe(0x80, 0x00, 0x00, 0),
    pe(0x00, 0x80, 0x00, 0),
    pe(0x80, 0x80, 0x00, 0),
    pe(0x00, 0x00, 0x80, 0),
    pe(0x80, 0x00, 0x80, 0),
    pe(0x00, 0x80, 0x80, 0),
    pe(0xc0, 0xc0, 0xc0, 0),
    pe(0xc0, 0xdc, 0xc0, 0),
    pe(0xa6, 0xca, 0xf0, 0),

    /* ... c_min/2 dynamic colorcells */
    /* ... gap (for sparse palettes) */
    /* ... c_min/2 dynamic colorcells */

    pe(0xff, 0xfb, 0xf0, 0),
    pe(0xa0, 0xa0, 0xa4, 0),
    pe(0x80, 0x80, 0x80, 0),
    pe(0xff, 0x00, 0x00, 0),
    pe(0x00, 0xff, 0x00, 0),
    pe(0xff, 0xff, 0x00, 0),
    pe(0x00, 0x00, 0xff, 0),
    pe(0xff, 0x00, 0xff, 0),
    pe(0x00, 0xff, 0xff, 0),
    pe(0xff, 0xff, 0xff, 0),    /* last 10 */
];

#[no_mangle]
pub unsafe extern "system" fn AnimatePalette(
    hpal: HPALETTE,
    iStartIndex: UINT,
    cEntries: UINT,
    ppe: *const PALETTEENTRY,
) -> BOOL {
    NtGdiDoPalette(
        hpal as HGDIOBJ,
        iStartIndex as WORD,
        cEntries as WORD,
        ppe as *mut PALETTEENTRY as LPVOID,
        _PALFUNCTYPE_GdiPalAnimate as DWORD,
        TRUE,
    ) as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn CreatePalette(plpal: *const LOGPALETTE) -> HPALETTE {
    NtGdiCreatePaletteInternal(plpal as LPLOGPALETTE, (*plpal).palNumEntries as UINT)
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetPaletteEntries(
    hpal: HPALETTE,
    iStartIndex: UINT,
    cEntries: UINT,
    ppe: LPPALETTEENTRY,
) -> UINT {
    NtGdiDoPalette(
        hpal as HGDIOBJ,
        iStartIndex as WORD,
        cEntries as WORD,
        ppe as LPVOID,
        _PALFUNCTYPE_GdiPalGetEntries as DWORD,
        FALSE,
    ) as UINT
}

#[no_mangle]
pub unsafe extern "system" fn SetPaletteEntries(
    hpal: HPALETTE,
    iStartIndex: UINT,
    cEntries: UINT,
    ppe: *const PALETTEENTRY,
) -> UINT {
    NtGdiDoPalette(
        hpal as HGDIOBJ,
        iStartIndex as WORD,
        cEntries as WORD,
        ppe as *mut PALETTEENTRY as LPVOID,
        _PALFUNCTYPE_GdiPalSetEntries as DWORD,
        TRUE,
    ) as UINT
}

#[no_mangle]
pub unsafe extern "system" fn GetSystemPaletteEntries(
    hDC: HDC,
    iStartIndex: UINT,
    cEntries: UINT,
    ppe: LPPALETTEENTRY,
) -> UINT {
    let mut ippe: [PALETTEENTRY; 256] = [pe(0, 0, 0, 0); 256];

    if (cEntries as INT) >= 0 {
        if (GetDeviceCaps(hDC, RASTERCAPS as INT) & RC_PALETTE as INT) != 0 {
            return NtGdiDoPalette(
                hDC as HGDIOBJ,
                iStartIndex as WORD,
                cEntries as WORD,
                ppe as LPVOID,
                _PALFUNCTYPE_GdiPalGetSystemEntries as DWORD,
                FALSE,
            ) as UINT;
        } else if !ppe.is_null() {
            // RtlCopyMemory(ippe, sys_pal_template, 10 * sizeof(PALETTEENTRY));
            core::ptr::copy_nonoverlapping(SYS_PAL_TEMPLATE.as_ptr(), ippe.as_mut_ptr(), 10);
            // RtlCopyMemory(&ippe[246], &sys_pal_template[10], 10 * sizeof(PALETTEENTRY));
            core::ptr::copy_nonoverlapping(
                SYS_PAL_TEMPLATE.as_ptr().add(10),
                ippe.as_mut_ptr().add(246),
                10,
            );
            // RtlZeroMemory(&ippe[10], sizeof(ippe) - 20 * sizeof(PALETTEENTRY));
            core::ptr::write_bytes(ippe.as_mut_ptr().add(10), 0, 256 - 20);

            if iStartIndex < 256 {
                // min(256 - iStartIndex, cEntries) entries
                let count = core::cmp::min(256 - iStartIndex, cEntries) as usize;
                core::ptr::copy_nonoverlapping(
                    ippe.as_ptr().add(iStartIndex as usize),
                    ppe,
                    count,
                );
            }
        }
    }

    0
}

#[no_mangle]
pub unsafe extern "system" fn GetDIBColorTable(
    hDC: HDC,
    iStartIndex: UINT,
    cEntries: UINT,
    pColors: *mut RGBQUAD,
) -> UINT {
    if cEntries != 0 {
        return NtGdiDoPalette(
            hDC as HGDIOBJ,
            iStartIndex as WORD,
            cEntries as WORD,
            pColors as LPVOID,
            _PALFUNCTYPE_GdiPalGetColorTable as DWORD,
            FALSE,
        ) as UINT;
    }
    0
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn RealizePalette(hdc: HDC) -> UINT {
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ)
        == GDILoObjType_GDILoObjType_LO_METADC16_TYPE as usize
    {
        return METADC_RealizePalette(hdc) as UINT;
    }

    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != GDILoObjType_GDILoObjType_LO_DC_TYPE as usize {
        return GDI_ERROR;
    }

    UserRealizePalette(hdc)
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn ResizePalette(hPalette: HPALETTE, nEntries: UINT) -> BOOL {
    NtGdiResizePalette(hPalette, nEntries)
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn SetDIBColorTable(
    hDC: HDC,
    iStartIndex: UINT,
    cEntries: UINT,
    pColors: *const RGBQUAD,
) -> UINT {
    let mut ret_value: UINT = 0;

    if cEntries != 0 {
        ret_value = NtGdiDoPalette(
            hDC as HGDIOBJ,
            iStartIndex as WORD,
            cEntries as WORD,
            pColors as *mut RGBQUAD as LPVOID,
            _PALFUNCTYPE_GdiPalSetColorTable as DWORD,
            TRUE,
        ) as UINT;
    }

    ret_value
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn UpdateColors(hdc: HDC) -> BOOL {
    // ((PW32CLIENTINFO)NtCurrentTeb()->Win32ClientInfo)->cSpins = 0;
    let ci = core::ptr::addr_of_mut!((*NtCurrentTeb()).Win32ClientInfo) as PW32CLIENTINFO;
    (*ci).cSpins = 0;
    NtGdiUpdateColors(hdc)
}

/*
 * @unimplemented
 */
#[no_mangle]
pub unsafe extern "system" fn ColorCorrectPalette(
    _hDC: HDC,
    _hPalette: HPALETTE,
    _dwFirstEntry: DWORD,
    _dwNumOfEntries: DWORD,
) -> BOOL {
    // UNIMPLEMENTED; (DPRINT no-op)
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    0
}

/* EOF */
