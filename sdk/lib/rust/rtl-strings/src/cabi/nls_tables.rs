//! NLS code-page-table machinery — the *structural* sub-bucket of
//! `sdk/lib/rtl/nls.c` that is NOT byte-diffable (it walks
//! caller-supplied `CPTABLEINFO` / `NLSTABLEINFO` pointers and the raw
//! NLS-file format, and mutates process-global `Nls*` state).
//!
//! This is a faithful 1:1 unsafe port (same shape as the `uniprefix`
//! port): behavioral-equivalent, validated by the boot smoke tests,
//! NOT the `core_rs` byte-diff pattern. It exists so the librtl
//! cohabitation swap can comment out the *whole* `nls.c` (the
//! whole-file rule — librtl has no `--allow-multiple-definition`).
//!
//! Gated behind the `kernel` feature: the byte-diff harness
//! (`rtl-difftest`, `cabi` only) supplies its own modelled `Nls*`
//! globals + `RtlpUpcase/DowncaseUnicodeChar`, so this module must be
//! invisible there to avoid duplicate-symbol at the host link.
//!
//! Faithful quirks pinned verbatim from nls.c:
//!  - DBCS branches are `ASSERT(FALSE)` / FIXME in the genuine code
//!    (unreachable on the default single-byte install) — ported as an
//!    `unreachable!()`-free no-op return matching the genuine
//!    fall-through (genuine `ASSERT` is a no-op in release builds, the
//!    function then returns `STATUS_SUCCESS` with `ResultSize`/output
//!    untouched).
//!  - `RtlpDowncaseUnicodeChar` does *not* mask `Source >> 8` (genuine
//!    relies on `WCHAR` already being 16-bit); `RtlpUpcaseUnicodeChar`
//!    *does* `& 0xFF`. Replicated exactly.
//!  - `Source + (SHORT)Offset` = `src.wrapping_add(off)` (low-16 of
//!    the C int arithmetic is identical to the u16 wrapping add).

use core::ffi::c_void;
use core::ptr;


const STATUS_SUCCESS: i32 = 0x0000_0000;
const STATUS_BUFFER_OVERFLOW: i32 = 0x8000_0005u32 as i32;

const MAXIMUM_LEADBYTES: usize = 12;

/// `CPTABLEINFO` (ddk/ntnls.h == wine/winternl.h — identical layout).
#[repr(C)]
pub struct CPTABLEINFO {
    pub CodePage: u16,
    pub MaximumCharacterSize: u16,
    pub DefaultChar: u16,
    pub UniDefaultChar: u16,
    pub TransDefaultChar: u16,
    pub TransUniDefaultChar: u16,
    pub DBCSCodePage: u16,
    pub LeadByte: [u8; MAXIMUM_LEADBYTES],
    pub MultiByteTable: *mut u16,
    pub WideCharTable: *mut c_void,
    pub DBCSRanges: *mut u16,
    pub DBCSOffsets: *mut u16,
}

/// `NLSTABLEINFO` (Oem, Ansi, Upper, Lower — both headers agree).
#[repr(C)]
pub struct NLSTABLEINFO {
    pub OemTableInfo: CPTABLEINFO,
    pub AnsiTableInfo: CPTABLEINFO,
    pub UpperCaseTable: *mut u16,
    pub LowerCaseTable: *mut u16,
}

/// `NLS_FILE_HEADER` (ndk/rtltypes.h).
#[repr(C)]
pub struct NLS_FILE_HEADER {
    pub HeaderSize: u16,
    pub CodePage: u16,
    pub MaximumCharacterSize: u16,
    pub DefaultChar: u16,
    pub UniDefaultChar: u16,
    pub TransDefaultChar: u16,
    pub TransUniDefaultChar: u16,
    pub LeadByte: [u8; MAXIMUM_LEADBYTES],
}

// ---------------------------------------------------------------------------
// Process-global NLS state (nls.c lines 18-36). nls.c is the genuine
// definition site; commenting it out for the cohabitation swap removes
// these, so rtl-strings must own the storage. Six are `@ extern` in
// ntoskrnl.spec (NlsAnsiCodePage, NlsLeadByteInfo, NlsMbCodePageTag,
// NlsMbOemCodePageTag, NlsOemCodePage, NlsOemLeadByteInfo) — the
// `#[no_mangle]` symbol names must match exactly.
// ---------------------------------------------------------------------------

#[no_mangle]
pub static mut NlsUnicodeUpcaseTable: *mut u16 = ptr::null_mut();
#[no_mangle]
pub static mut NlsUnicodeLowercaseTable: *mut u16 = ptr::null_mut();

#[no_mangle]
pub static mut NlsAnsiCodePage: u16 = 0;
#[no_mangle]
pub static mut NlsMbCodePageTag: u8 = 0; // BOOLEAN (FALSE)
#[no_mangle]
pub static mut NlsAnsiToUnicodeTable: *mut u16 = ptr::null_mut();
#[no_mangle]
pub static mut NlsUnicodeToAnsiTable: *mut i8 = ptr::null_mut(); // PCHAR
#[no_mangle]
pub static mut NlsUnicodeToMbAnsiTable: *mut u16 = ptr::null_mut();
#[no_mangle]
pub static mut NlsLeadByteInfo: *mut u16 = ptr::null_mut();

#[no_mangle]
pub static mut NlsOemCodePage: u16 = 0;
#[no_mangle]
pub static mut NlsMbOemCodePageTag: u8 = 0; // BOOLEAN (FALSE)
#[no_mangle]
pub static mut NlsOemToUnicodeTable: *mut u16 = ptr::null_mut();
#[no_mangle]
pub static mut NlsUnicodeToOemTable: *mut i8 = ptr::null_mut(); // PCHAR
#[no_mangle]
pub static mut NlsUnicodeToMbOemTable: *mut u16 = ptr::null_mut();
#[no_mangle]
pub static mut NlsOemLeadByteInfo: *mut u16 = ptr::null_mut();

#[no_mangle]
pub static mut NlsOemDefaultChar: u16 = 0;
#[no_mangle]
pub static mut NlsUnicodeDefaultChar: u16 = 0;

// ---------------------------------------------------------------------------
// Private case helpers (nls.c 695-715 / 89-125). Exported symbols —
// path.c (still C) and the `uniprefix` crate call them via `extern`.
// Faithful table-walk: NULL table before RtlResetRtlTranslations is the
// genuine pre-NLS-init state (C global is also NULL); ntoskrnl init
// order guarantees the reset runs first.
// ---------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "system" fn RtlpUpcaseUnicodeChar(source: u16) -> u16 {
    if source < b'a' as u16 {
        return source;
    }
    if source <= b'z' as u16 {
        return source - (b'a' - b'A') as u16;
    }

    let mut offset: u16 = (source >> 8) & 0xFF;
    offset = *NlsUnicodeUpcaseTable.add(offset as usize);

    offset = offset.wrapping_add((source >> 4) & 0xF);
    offset = *NlsUnicodeUpcaseTable.add(offset as usize);

    offset = offset.wrapping_add(source & 0xF);
    offset = *NlsUnicodeUpcaseTable.add(offset as usize);

    source.wrapping_add(offset)
}

#[no_mangle]
pub unsafe extern "system" fn RtlpDowncaseUnicodeChar(source: u16) -> u16 {
    if source < b'A' as u16 {
        return source;
    }
    if source <= b'Z' as u16 {
        return source + (b'a' - b'A') as u16;
    }
    if source < 0x80 {
        return source;
    }

    let mut offset: u16 = source >> 8; // genuine: no & 0xFF here
    offset = *NlsUnicodeLowercaseTable.add(offset as usize);

    offset = offset.wrapping_add((source & 0x00F0) >> 4);
    offset = *NlsUnicodeLowercaseTable.add(offset as usize);

    offset = offset.wrapping_add(source & 0x000F);
    offset = *NlsUnicodeLowercaseTable.add(offset as usize);

    source.wrapping_add(offset)
}

// ---------------------------------------------------------------------------
// Code-page-table initialisation.
// ---------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "system" fn RtlInitCodePageTable(
    table_base: *mut u16,
    code_page_table: *mut CPTABLEINFO,
) {
    let hdr = table_base as *const NLS_FILE_HEADER;
    let cpt = &mut *code_page_table;

    cpt.CodePage = (*hdr).CodePage;
    cpt.MaximumCharacterSize = (*hdr).MaximumCharacterSize;
    cpt.DefaultChar = (*hdr).DefaultChar;
    cpt.UniDefaultChar = (*hdr).UniDefaultChar;
    cpt.TransDefaultChar = (*hdr).TransDefaultChar;
    cpt.TransUniDefaultChar = (*hdr).TransUniDefaultChar;

    // Faithful to C's `RtlCopyMemory(.., NlsFileHeader->LeadByte, MAXIMUM_LEADBYTES)`,
    // but copied byte-wise via volatile so the optimizer cannot coalesce the 12-byte
    // copy into a wide unaligned load. `table_base` here lands at +0x0e (2-byte
    // aligned, not 4/8-aligned); on arm64 a vectorized `ldur w`/`ldur x` of that
    // source alignment-faults (DFSC 0x21) because the NLS region is mapped with
    // natural-alignment-required attributes — a clean cargo rebuild emitted exactly
    // that wide load and bugchecked 0x1E in RtlInitCodePageTable during Phase-1 init.
    let lb_src = (*hdr).LeadByte.as_ptr();
    let lb_dst = cpt.LeadByte.as_mut_ptr();
    let mut i = 0usize;
    while i < MAXIMUM_LEADBYTES {
        ptr::write_volatile(lb_dst.add(i), ptr::read_volatile(lb_src.add(i)));
        i += 1;
    }

    let header_size = (*hdr).HeaderSize as usize;

    // WideCharTable = TableBase + HeaderSize + 1 + TableBase[HeaderSize]
    cpt.WideCharTable = table_base
        .add(header_size + 1 + *table_base.add(header_size) as usize)
        as *mut c_void;

    // MultiByteTable = TableBase + HeaderSize + 1
    cpt.MultiByteTable = table_base.add(header_size + 1);

    // Glyph table (256 wchars) present?
    if *cpt.MultiByteTable.add(256) == 0 {
        cpt.DBCSRanges = cpt.MultiByteTable.add(256 + 1);
    } else {
        cpt.DBCSRanges = cpt.MultiByteTable.add(256 + 1 + 256);
    }

    if *cpt.DBCSRanges != 0 {
        cpt.DBCSCodePage = 1;
        cpt.DBCSOffsets = cpt.DBCSRanges.add(1);
    } else {
        cpt.DBCSCodePage = 0;
        cpt.DBCSOffsets = ptr::null_mut();
    }
}

#[no_mangle]
pub unsafe extern "system" fn RtlInitNlsTables(
    ansi_table_base: *mut u16,
    oem_table_base: *mut u16,
    case_table_base: *mut u16,
    nls_table: *mut NLSTABLEINFO,
) {
    if !ansi_table_base.is_null() && !oem_table_base.is_null() && !case_table_base.is_null() {
        let nt = &mut *nls_table;
        RtlInitCodePageTable(ansi_table_base, &mut nt.AnsiTableInfo);
        RtlInitCodePageTable(oem_table_base, &mut nt.OemTableInfo);

        nt.UpperCaseTable = case_table_base.add(2);
        nt.LowerCaseTable = case_table_base.add(*case_table_base.add(1) as usize + 2);
    }
}

#[no_mangle]
pub unsafe extern "system" fn RtlResetRtlTranslations(nls_table: *mut NLSTABLEINFO) {
    let nt = &*nls_table;

    // ANSI data
    NlsAnsiToUnicodeTable = nt.AnsiTableInfo.MultiByteTable;
    NlsUnicodeToAnsiTable = nt.AnsiTableInfo.WideCharTable as *mut i8;
    NlsUnicodeToMbAnsiTable = nt.AnsiTableInfo.WideCharTable as *mut u16;
    NlsMbCodePageTag = (nt.AnsiTableInfo.DBCSCodePage != 0) as u8;
    NlsLeadByteInfo = nt.AnsiTableInfo.DBCSOffsets;
    NlsAnsiCodePage = nt.AnsiTableInfo.CodePage;

    // OEM data
    NlsOemToUnicodeTable = nt.OemTableInfo.MultiByteTable;
    NlsUnicodeToOemTable = nt.OemTableInfo.WideCharTable as *mut i8;
    NlsUnicodeToMbOemTable = nt.OemTableInfo.WideCharTable as *mut u16;
    NlsMbOemCodePageTag = (nt.OemTableInfo.DBCSCodePage != 0) as u8;
    NlsOemLeadByteInfo = nt.OemTableInfo.DBCSOffsets;
    NlsOemCodePage = nt.OemTableInfo.CodePage;

    // Unicode case map data
    NlsUnicodeUpcaseTable = nt.UpperCaseTable;
    NlsUnicodeLowercaseTable = nt.LowerCaseTable;

    // Default chars for RtlpDidUnicodeToOemWork
    NlsOemDefaultChar = nt.OemTableInfo.DefaultChar;
    NlsUnicodeDefaultChar = nt.OemTableInfo.TransDefaultChar;
}

// ---------------------------------------------------------------------------
// Custom-CP single-byte converters. DBCS path = genuine ASSERT(FALSE)
// FIXME (release ASSERT is a no-op → falls through to STATUS_SUCCESS
// with output untouched). Replicated as that exact fall-through.
// ---------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "system" fn RtlCustomCPToUnicodeN(
    custom_cp: *mut CPTABLEINFO,
    unicode_string: *mut u16,
    unicode_size: u32,
    result_size: *mut u32,
    custom_string: *const i8,
    custom_size: u32,
) -> i32 {
    let cp = &*custom_cp;

    if cp.DBCSCodePage == 0 {
        let size = if custom_size > unicode_size / 2 {
            unicode_size / 2
        } else {
            custom_size
        };

        if !result_size.is_null() {
            *result_size = size * 2;
        }

        let mut u = unicode_string;
        let mut c = custom_string;
        for _ in 0..size {
            *u = *cp.MultiByteTable.add(*(c as *const u8) as usize);
            u = u.add(1);
            c = c.add(1);
        }
    }
    // else: genuine ASSERT(FALSE) — release no-op, fall through.

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn RtlUnicodeToCustomCPN(
    custom_cp: *mut CPTABLEINFO,
    custom_string: *mut i8,
    custom_size: u32,
    result_size: *mut u32,
    unicode_string: *mut u16,
    unicode_size: u32,
) -> i32 {
    let cp = &*custom_cp;

    if cp.DBCSCodePage == 0 {
        let size = if unicode_size > custom_size * 2 {
            custom_size
        } else {
            unicode_size / 2
        };

        if !result_size.is_null() {
            *result_size = size;
        }

        let wct = cp.WideCharTable as *const i8;
        let mut c = custom_string;
        let mut u = unicode_string;
        for _ in 0..size {
            *c = *wct.add(*u as usize);
            c = c.add(1);
            u = u.add(1);
        }
    }
    // else: genuine ASSERT(FALSE) — release no-op, fall through.

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn RtlUpcaseUnicodeToCustomCPN(
    custom_cp: *mut CPTABLEINFO,
    custom_string: *mut i8,
    custom_size: u32,
    result_size: *mut u32,
    unicode_string: *mut u16,
    unicode_size: u32,
) -> i32 {
    let cp = &*custom_cp;

    if cp.DBCSCodePage == 0 {
        let size = if unicode_size > custom_size * 2 {
            custom_size
        } else {
            unicode_size / 2
        };

        if !result_size.is_null() {
            *result_size = size;
        }

        let wct = cp.WideCharTable as *const i8;
        let mut c = custom_string;
        let mut u = unicode_string;
        for _ in 0..size {
            let upcase = RtlpUpcaseUnicodeChar(*u);
            *c = *wct.add(upcase as usize);
            c = c.add(1);
            u = u.add(1);
        }
    }
    // else: genuine ASSERT(FALSE) — release no-op, fall through.

    STATUS_SUCCESS
}

// ---------------------------------------------------------------------------
// RtlConsoleMultiByteToUnicodeN — genuine UNIMPLEMENTED stub: set
// *Unknown = 1, delegate to RtlMultiByteToUnicodeN.
// ---------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "system" fn RtlConsoleMultiByteToUnicodeN(
    unicode_string: *mut u16,
    unicode_size: u32,
    result_size: *mut u32,
    mb_string: *const i8,
    mb_size: u32,
    unknown: *mut u32,
) -> i32 {
    *unknown = 1;
    RtlMultiByteToUnicodeN(
        unicode_string,
        unicode_size,
        result_size,
        mb_string as *const u8,
        mb_size,
    )
}

// RtlpDidUnicodeToOemWork is unicode.c's, not nls.c's: it lives in nlsapi.rs, which the
// boot loaders' build shares.

// ---------------------------------------------------------------------------
// The nls.c converters and case helpers, table-driven as the C is. These were the
// shared CP1252 model in cabi/mod.rs, which ignored the tables RtlResetRtlTranslations
// loads: with the OEM code page 437, every byte >= 0x80 came out of RtlUnicodeToOemN and
// friends as the CP1252 byte instead. The model survives only for the host byte-diff
// harness (no `kernel` feature), where both sides model the table identically.
//
// `MbSize * sizeof(WCHAR)` is SIZE_T arithmetic in the C, so the single-byte size
// compares widen to usize. The multi-byte branches are the C's own FIXME code, ported as
// it stands (unreachable on a single-byte install).
// ---------------------------------------------------------------------------

#[inline(always)]
fn lobyte(w: u16) -> u8 {
    w as u8
}
#[inline(always)]
fn hibyte(w: u16) -> u8 {
    (w >> 8) as u8
}

#[no_mangle]
pub unsafe extern "system" fn RtlDowncaseUnicodeChar(source: u16) -> u16 {
    RtlpDowncaseUnicodeChar(source)
}

#[no_mangle]
pub unsafe extern "system" fn RtlUpcaseUnicodeChar(source: u16) -> u16 {
    RtlpUpcaseUnicodeChar(source)
}

#[no_mangle]
pub unsafe extern "system" fn RtlGetDefaultCodePage(ansi_code_page: *mut u16, oem_code_page: *mut u16) {
    *ansi_code_page = NlsAnsiCodePage;
    *oem_code_page = NlsOemCodePage;
}

#[no_mangle]
pub unsafe extern "system" fn RtlMultiByteToUnicodeN(
    mut unicode_string: *mut u16,
    unicode_size: u32,
    result_size: *mut u32,
    mut mb_string: *const u8,
    mb_size: u32,
) -> i32 {
    if NlsMbCodePageTag == 0 {
        let size = if mb_size > unicode_size / 2 { unicode_size / 2 } else { mb_size };
        if !result_size.is_null() {
            *result_size = size * 2;
        }
        for i in 0..size as usize {
            *unicode_string.add(i) = *NlsAnsiToUnicodeTable.add(*mb_string.add(i) as usize);
        }
    } else {
        let mb_end = mb_string.add(mb_size as usize);
        let mut i: u32 = 0;
        while i < unicode_size / 2 && mb_string < mb_end {
            let ch = *mb_string;
            mb_string = mb_string.add(1);
            i += 1;
            if ch < 0x80 {
                *unicode_string = ch as u16;
                unicode_string = unicode_string.add(1);
                continue;
            }
            let lead_byte_info = *NlsLeadByteInfo.add(ch as usize);
            if lead_byte_info == 0 {
                *unicode_string = *NlsAnsiToUnicodeTable.add(ch as usize);
                unicode_string = unicode_string.add(1);
                continue;
            }
            if mb_string < mb_end {
                *unicode_string = *NlsLeadByteInfo.add(lead_byte_info as usize + *mb_string as usize);
                unicode_string = unicode_string.add(1);
                mb_string = mb_string.add(1);
            }
        }
        if !result_size.is_null() {
            *result_size = i * 2;
        }
    }
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn RtlMultiByteToUnicodeSize(
    unicode_size: *mut u32,
    mut mb_string: *const u8,
    mut mb_size: u32,
) -> i32 {
    if NlsMbCodePageTag == 0 {
        *unicode_size = mb_size.wrapping_mul(2);
    } else {
        let mut length: u32 = 0;
        while mb_size != 0 {
            mb_size -= 1;
            let ch = *mb_string;
            mb_string = mb_string.add(1);
            if ch >= 0x80 && *NlsLeadByteInfo.add(ch as usize) != 0 && mb_size != 0 {
                mb_size -= 1;
                mb_string = mb_string.add(1);
            }
            length += 1;
        }
        *unicode_size = length * 2;
    }
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn RtlOemToUnicodeN(
    mut unicode_string: *mut u16,
    unicode_size: u32,
    result_size: *mut u32,
    mut oem_string: *const u8,
    oem_size: u32,
) -> i32 {
    if NlsMbOemCodePageTag == 0 {
        let size = if oem_size > unicode_size / 2 { unicode_size / 2 } else { oem_size };
        if !result_size.is_null() {
            *result_size = size * 2;
        }
        for _ in 0..size {
            *unicode_string = *NlsOemToUnicodeTable.add(*oem_string as usize);
            unicode_string = unicode_string.add(1);
            oem_string = oem_string.add(1);
        }
    } else {
        let oem_end = oem_string.add(oem_size as usize);
        let mut i: u32 = 0;
        while i < unicode_size / 2 && oem_string < oem_end {
            let ch = *oem_string;
            oem_string = oem_string.add(1);
            i += 1;
            if ch < 0x80 {
                *unicode_string = ch as u16;
                unicode_string = unicode_string.add(1);
                continue;
            }
            let lead_byte_info = *NlsOemLeadByteInfo.add(ch as usize);
            if lead_byte_info == 0 {
                *unicode_string = *NlsOemToUnicodeTable.add(ch as usize);
                unicode_string = unicode_string.add(1);
                continue;
            }
            if oem_string < oem_end {
                *unicode_string = *NlsOemLeadByteInfo.add(lead_byte_info as usize + *oem_string as usize);
                unicode_string = unicode_string.add(1);
                oem_string = oem_string.add(1);
            }
        }
        if !result_size.is_null() {
            *result_size = i * 2;
        }
    }
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn RtlUnicodeToMultiByteN(
    mut mb_string: *mut u8,
    mb_size: u32,
    result_size: *mut u32,
    mut unicode_string: *const u16,
    unicode_size: u32,
) -> i32 {
    if NlsMbCodePageTag == 0 {
        let size = if unicode_size as usize > mb_size as usize * 2 { mb_size } else { unicode_size / 2 };
        if !result_size.is_null() {
            *result_size = size;
        }
        for _ in 0..size {
            *mb_string = *NlsUnicodeToAnsiTable.add(*unicode_string as usize) as u8;
            mb_string = mb_string.add(1);
            unicode_string = unicode_string.add(1);
        }
    } else {
        let mut i = mb_size;
        let mut size = unicode_size / 2;
        while i != 0 && size != 0 {
            let wide_char = *unicode_string;
            unicode_string = unicode_string.add(1);
            if wide_char < 0x80 {
                *mb_string = lobyte(wide_char);
                mb_string = mb_string.add(1);
            } else {
                let mb_char = *NlsUnicodeToMbAnsiTable.add(wide_char as usize);
                if hibyte(mb_char) == 0 {
                    *mb_string = lobyte(mb_char);
                    mb_string = mb_string.add(1);
                } else if i >= 2 {
                    *mb_string = hibyte(mb_char);
                    *mb_string.add(1) = lobyte(mb_char);
                    mb_string = mb_string.add(2);
                    i -= 1;
                } else {
                    break;
                }
            }
            i -= 1;
            size -= 1;
        }
        if !result_size.is_null() {
            *result_size = mb_size - i;
        }
    }
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn RtlUnicodeToMultiByteSize(
    mb_size: *mut u32,
    mut unicode_string: *const u16,
    unicode_size: u32,
) -> i32 {
    let mut unicode_length = unicode_size / 2;
    if NlsMbCodePageTag == 0 {
        *mb_size = unicode_length;
    } else {
        let mut mb_length: u32 = 0;
        while unicode_length != 0 {
            unicode_length -= 1;
            let wide_char = *unicode_string;
            unicode_string = unicode_string.add(1);
            if wide_char >= 0x80 && hibyte(*NlsUnicodeToMbAnsiTable.add(wide_char as usize)) != 0 {
                mb_length += 2;
            } else {
                mb_length += 1;
            }
        }
        *mb_size = mb_length;
    }
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn RtlUnicodeToOemN(
    oem_string: *mut u8,
    mut oem_size: u32,
    result_size: *mut u32,
    mut unicode_string: *const u16,
    unicode_size: u32,
) -> i32 {
    let mut size: u32 = 0;
    // Bytes -> chars
    let mut unicode_size = unicode_size / 2;
    if NlsMbOemCodePageTag == 0 {
        while oem_size != 0 && unicode_size != 0 {
            *oem_string.add(size as usize) = *NlsUnicodeToOemTable.add(*unicode_string as usize) as u8;
            unicode_string = unicode_string.add(1);
            size += 1;
            oem_size -= 1;
            unicode_size -= 1;
        }
    } else {
        while oem_size != 0 && unicode_size != 0 {
            let oem_char = *NlsUnicodeToMbOemTable.add(*unicode_string as usize);
            unicode_string = unicode_string.add(1);
            if hibyte(oem_char) != 0 {
                if oem_size < 2 {
                    break;
                }
                *oem_string.add(size as usize) = hibyte(oem_char);
                size += 1;
                oem_size -= 1;
            }
            *oem_string.add(size as usize) = lobyte(oem_char);
            size += 1;
            oem_size -= 1;
            unicode_size -= 1;
        }
    }
    if !result_size.is_null() {
        *result_size = size;
    }
    if unicode_size != 0 {
        STATUS_BUFFER_OVERFLOW
    } else {
        STATUS_SUCCESS
    }
}

#[no_mangle]
pub unsafe extern "system" fn RtlUpcaseUnicodeToMultiByteN(
    mut mb_string: *mut u8,
    mb_size: u32,
    result_size: *mut u32,
    mut unicode_string: *const u16,
    unicode_size: u32,
) -> i32 {
    if NlsMbCodePageTag == 0 {
        let size = if unicode_size as usize > mb_size as usize * 2 { mb_size } else { unicode_size / 2 };
        if !result_size.is_null() {
            *result_size = size;
        }
        for _ in 0..size {
            let upcase_char = RtlpUpcaseUnicodeChar(*unicode_string);
            *mb_string = *NlsUnicodeToAnsiTable.add(upcase_char as usize) as u8;
            mb_string = mb_string.add(1);
            unicode_string = unicode_string.add(1);
        }
    }
    // else: the C's multi-byte branch is ASSERT(FALSE) -- a release no-op.
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn RtlUpcaseUnicodeToOemN(
    mut oem_string: *mut u8,
    oem_size: u32,
    result_size: *mut u32,
    mut unicode_string: *const u16,
    unicode_size: u32,
) -> i32 {
    if NlsMbOemCodePageTag == 0 {
        let size = if unicode_size as usize > oem_size as usize * 2 { oem_size } else { unicode_size / 2 };
        if !result_size.is_null() {
            *result_size = size;
        }
        for _ in 0..size {
            let upcase_char = RtlpUpcaseUnicodeChar(*unicode_string);
            *oem_string = *NlsUnicodeToOemTable.add(upcase_char as usize) as u8;
            oem_string = oem_string.add(1);
            unicode_string = unicode_string.add(1);
        }
    } else {
        let mut i = oem_size;
        let mut size = unicode_size / 2;
        while i != 0 && size != 0 {
            let wide_char = RtlpUpcaseUnicodeChar(*unicode_string);
            unicode_string = unicode_string.add(1);
            if wide_char < 0x80 {
                *oem_string = lobyte(wide_char);
                oem_string = oem_string.add(1);
            } else {
                let oem_char = *NlsUnicodeToMbOemTable.add(wide_char as usize);
                if hibyte(oem_char) == 0 {
                    *oem_string = lobyte(oem_char);
                    oem_string = oem_string.add(1);
                } else if i >= 2 {
                    *oem_string = hibyte(oem_char);
                    *oem_string.add(1) = lobyte(oem_char);
                    oem_string = oem_string.add(2);
                    i -= 1;
                } else {
                    break;
                }
            }
            i -= 1;
            size -= 1;
        }
        if !result_size.is_null() {
            *result_size = oem_size - i;
        }
    }
    STATUS_SUCCESS
}

/// `CHAR RtlUpperChar(CHAR)`. CHAR is signed, so every byte >= 0x80 takes the `<= 'z'`
/// branch and comes back unchanged; only 0x7B-0x7F reach the tables.
#[no_mangle]
pub unsafe extern "system" fn RtlUpperChar(source: u8) -> u8 {
    let s = source as i8;
    if s <= b'z' as i8 {
        if s >= b'a' as i8 {
            return source ^ (b'a' - b'A');
        }
        return source;
    }
    if NlsMbCodePageTag == 0 {
        let unicode = *NlsAnsiToUnicodeTable.add(source as usize);
        let unicode = RtlpUpcaseUnicodeChar(unicode);
        *NlsUnicodeToAnsiTable.add(unicode as usize) as u8
    } else {
        source
    }
}
