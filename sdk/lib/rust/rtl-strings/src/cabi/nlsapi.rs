//! The NLS layer unicode.c calls through, routed the way the C routes it.
//!
//! unicode.c never converts or case-maps by itself: it calls nls.c's exports
//! (`RtlUnicodeToOemN`, `RtlpUpcaseUnicodeChar`, ...) and reads its `Nls*` globals. Each
//! build supplies that layer differently, and this module is the one place that knows:
//!
//! - `kernel` (ntoskrnl and ntdll): nls_tables.rs, the table-driven port of nls.c, over the
//!   tables `RtlResetRtlTranslations` loads.
//! - `bldr` (the boot loaders' blrtl-rust): boot/rtl/nlsboot.c's Rust port, linked by name
//!   from blrtl-rust (ASCII-only case mapping, no tables).
//! - neither (the host byte-diff harness, rtl-difftest): the shared CP1252 model in
//!   cabi/mod.rs, which the harness's C side models identically.
//!
//! The unicode.c wrappers used to call the model directly in every build, so the kernel
//! ignored the loaded tables (OEM is code page 437, so every byte >= 0x80 differed from
//! the C) and the loaders could not have used nlsboot at all.

#![allow(non_snake_case, non_upper_case_globals)]

#[cfg(any(feature = "kernel", feature = "bldr"))]
use super::{STRING, UNICODE_STRING};

#[cfg(feature = "kernel")]
pub use super::nls_tables::{
    NlsLeadByteInfo, NlsMbOemCodePageTag, NlsOemDefaultChar, NlsOemLeadByteInfo,
    NlsOemToUnicodeTable, NlsUnicodeDefaultChar, NlsUnicodeToMbOemTable, NlsUnicodeToOemTable,
    RtlMultiByteToUnicodeN, RtlOemToUnicodeN, RtlUnicodeToMultiByteN, RtlUnicodeToOemN,
    RtlUpcaseUnicodeToMultiByteN, RtlUpcaseUnicodeToOemN, RtlUpperChar, RtlpDowncaseUnicodeChar,
    RtlpUpcaseUnicodeChar,
};

#[cfg(all(feature = "bldr", not(feature = "kernel")))]
extern "system" {
    pub fn RtlMultiByteToUnicodeN(u: *mut u16, us: u32, rs: *mut u32, mb: *const u8, mbs: u32) -> i32;
    pub fn RtlOemToUnicodeN(u: *mut u16, us: u32, rs: *mut u32, oem: *const u8, oems: u32) -> i32;
    pub fn RtlUnicodeToMultiByteN(mb: *mut u8, mbs: u32, rs: *mut u32, u: *const u16, us: u32) -> i32;
    pub fn RtlUnicodeToOemN(oem: *mut u8, oems: u32, rs: *mut u32, u: *const u16, us: u32) -> i32;
    pub fn RtlUpcaseUnicodeToMultiByteN(mb: *mut u8, mbs: u32, rs: *mut u32, u: *const u16, us: u32) -> i32;
    pub fn RtlUpcaseUnicodeToOemN(oem: *mut u8, oems: u32, rs: *mut u32, u: *const u16, us: u32) -> i32;
    pub fn RtlUpperChar(c: u8) -> u8;
    pub fn RtlpUpcaseUnicodeChar(c: u16) -> u16;
    pub fn RtlpDowncaseUnicodeChar(c: u16) -> u16;
    // nlsboot.c's globals (BOOLEAN / PUSHORT / PCHAR / USHORT).
    pub static mut NlsLeadByteInfo: *mut u16;
    pub static mut NlsMbOemCodePageTag: u8;
    pub static mut NlsOemLeadByteInfo: *mut u16;
    pub static mut NlsOemToUnicodeTable: *mut u16;
    pub static mut NlsUnicodeToMbOemTable: *mut u16;
    pub static mut NlsUnicodeToOemTable: *mut i8;
    pub static mut NlsOemDefaultChar: u16;
    pub static mut NlsUnicodeDefaultChar: u16;
}

#[cfg(not(any(feature = "kernel", feature = "bldr")))]
pub use super::{
    RtlMultiByteToUnicodeN, RtlOemToUnicodeN, RtlUnicodeToMultiByteN, RtlUnicodeToOemN,
    RtlUpcaseUnicodeToMultiByteN, RtlUpcaseUnicodeToOemN,
};

// Safe entry points for core_rs (which forbids `unsafe`). The case helpers read the NLS
// tables, which the kernel/ntdll load in their init before any string op, and nlsboot
// reads none; so a call is sound wherever the C's would be.
#[cfg(any(feature = "kernel", feature = "bldr"))]
pub fn upcase(c: u16) -> u16 {
    unsafe { RtlpUpcaseUnicodeChar(c) }
}
#[cfg(any(feature = "kernel", feature = "bldr"))]
pub fn downcase(c: u16) -> u16 {
    unsafe { RtlpDowncaseUnicodeChar(c) }
}
#[cfg(any(feature = "kernel", feature = "bldr"))]
pub fn upper(c: u8) -> u8 {
    unsafe { RtlUpperChar(c) }
}

/// `BOOLEAN RtlpDidUnicodeToOemWork(PCUNICODE_STRING, POEM_STRING)` (unicode.c): FALSE
/// when a byte came out as the OEM default char from a source char that was not the
/// Unicode default, i.e. a real unmappable substitution happened. `Buffer[i]` is a signed
/// CHAR promoted to int, so a byte >= 0x80 never equals the USHORT default.
#[cfg(any(feature = "kernel", feature = "bldr"))]
#[no_mangle]
pub unsafe extern "system" fn RtlpDidUnicodeToOemWork(
    unicode_string: *const UNICODE_STRING,
    oem_string: *const STRING,
) -> u8 {
    if NlsMbOemCodePageTag == 0 {
        let os = &*oem_string;
        let us = &*unicode_string;
        let mut i: u32 = 0;
        while i < os.Length as u32 {
            let oem_ch = *os.Buffer.add(i as usize) as i8 as i32;
            let uni_ch = *us.Buffer.add(i as usize);
            if oem_ch == NlsOemDefaultChar as i32 && uni_ch != NlsUnicodeDefaultChar {
                return 0;
            }
            i += 1;
        }
        return 1;
    }
    // multi-byte: the C's FIXME returns TRUE
    1
}

/// The host model of `RtlpDidUnicodeToOemWork`, over the same `OemString->Length` the C
/// walks.
#[cfg(not(any(feature = "kernel", feature = "bldr")))]
pub unsafe fn RtlpDidUnicodeToOemWork(
    unicode_string: *const super::UNICODE_STRING,
    oem_string: *const super::STRING,
) -> u8 {
    let n = (*oem_string).Length as usize;
    let oem = if n == 0 { &[][..] } else { core::slice::from_raw_parts((*oem_string).Buffer, n) };
    let un = (*unicode_string).Length as usize / 2;
    let uni = if un == 0 || (*unicode_string).Buffer.is_null() {
        &[][..]
    } else {
        core::slice::from_raw_parts((*unicode_string).Buffer, un)
    };
    crate::core_rs::nls::did_oem_work(uni, oem) as u8
}
