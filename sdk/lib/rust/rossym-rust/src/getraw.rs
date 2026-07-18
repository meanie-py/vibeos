//! [correctness-review 2026-06-07 · sonnet] Reviewed vs sdk/lib/rossym/getraw.c.
//! Risk: LOW. Behavior: preserved — logic and layout match C exactly.
//! Functions reviewed: RosSymGetRawDataLength (1/2), RosSymGetRawData (2/2).
//! Findings:
//!   NOTE — `RosSymGetRawDataLength` uses `wrapping_add`/`wrapping_mul` (lines 10-14).
//!     C uses plain arithmetic on ULONG (u32) which wraps on overflow in C's unsigned
//!     arithmetic rules. `wrapping_*` is the correct Rust equivalent; no behavioral change.
//!   NOTE — `RosSymGetRawData` field assignments (SymbolsOffset, SymbolsLength,
//!     StringsOffset, StringsLength) and both `memcpy` calls match C offsets exactly.
//!   NOTE — `extern "C" fn memcpy` is correct cdecl for this library.
//!   NOTE — Return type `u32` for `RosSymGetRawDataLength` matches C `ULONG` (u32). Correct.
//!   NOTE — Return type `()` (void) for `RosSymGetRawData` matches C `VOID`. Correct.
//! Unverified: nothing — this file is complete and self-contained.

use crate::types::*;

extern "C" {
    fn memcpy(dest: *mut core::ffi::c_void, src: *const core::ffi::c_void, n: usize) -> *mut core::ffi::c_void;
}
#[no_mangle]
pub unsafe extern "C" fn RosSymGetRawDataLength(RosSymInfo: *mut ROSSYM_INFO) -> u32 {
    (core::mem::size_of::<ROSSYM_HEADER>() as u32)
        .wrapping_add(
            ((*RosSymInfo).SymbolsCount as u32)
                .wrapping_mul(core::mem::size_of::<ROSSYM_ENTRY>() as u32)
        )
        .wrapping_add((*RosSymInfo).StringsLength)
}
#[no_mangle]
pub unsafe extern "C" fn RosSymGetRawData(RosSymInfo: *mut ROSSYM_INFO, RawData: *mut core::ffi::c_void) {
    let RosSymHeader = RawData as *mut ROSSYM_HEADER;
    (*RosSymHeader).SymbolsOffset = core::mem::size_of::<ROSSYM_HEADER>() as u32;
    (*RosSymHeader).SymbolsLength = ((*RosSymInfo).SymbolsCount as usize * core::mem::size_of::<ROSSYM_ENTRY>()) as u32;
    (*RosSymHeader).StringsOffset = (*RosSymHeader).SymbolsOffset + (*RosSymHeader).SymbolsLength;
    (*RosSymHeader).StringsLength = (*RosSymInfo).StringsLength;

    memcpy(
        (RawData as *mut u8).add((*RosSymHeader).SymbolsOffset as usize) as *mut core::ffi::c_void,
        (*RosSymInfo).Symbols as *const core::ffi::c_void,
        (*RosSymHeader).SymbolsLength as usize,
    );
    memcpy(
        (RawData as *mut u8).add((*RosSymHeader).StringsOffset as usize) as *mut core::ffi::c_void,
        (*RosSymInfo).Strings as *const core::ffi::c_void,
        (*RosSymHeader).StringsLength as usize,
    );
}
