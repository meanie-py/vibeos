//! `boot/rtl/nlsboot.c` — the loader's "poor-man" NLS, used at boot time when no NLS tables
//! are loaded: a single-byte code page whose only case mapping is ASCII a-z, plus stubbed
//! OEM conversions that unicode.c needs to link. All NTAPI (`extern "system"`).
//!
//! One divergence from the C, in `RtlMultiByteToUnicodeN`: the C says "Trivially
//! zero-extend" and then writes `(WCHAR)MbString[i]` from a signed `CHAR`, which SIGN-extends
//! (0xE9 -> 0xFFE9). This zero-extends, as the comment and the single-byte code page mean.

#![allow(non_snake_case, non_upper_case_globals)]

use core::ffi::c_char;
use core::ptr::null_mut;

type NTSTATUS = i32;
const STATUS_SUCCESS: NTSTATUS = 0;
const STATUS_NOT_IMPLEMENTED: NTSTATUS = 0xC000_0002u32 as i32;

// GLOBALS: unicode.c reads these; at boot no code-page table is loaded.
#[unsafe(no_mangle)]
pub static mut NlsMbCodePageTag: u8 = 0; // BOOLEAN FALSE
#[unsafe(no_mangle)]
pub static mut NlsMbOemCodePageTag: u8 = 0;
#[unsafe(no_mangle)]
pub static mut NlsOemToUnicodeTable: *mut u16 = null_mut();
#[unsafe(no_mangle)]
pub static mut NlsUnicodeToOemTable: *mut c_char = null_mut();
#[unsafe(no_mangle)]
pub static mut NlsUnicodeToMbOemTable: *mut u16 = null_mut();
#[unsafe(no_mangle)]
pub static mut NlsLeadByteInfo: *mut u16 = null_mut();
#[unsafe(no_mangle)]
pub static mut NlsOemLeadByteInfo: *mut u16 = null_mut();
#[unsafe(no_mangle)]
pub static mut NlsOemDefaultChar: u16 = 0;
#[unsafe(no_mangle)]
pub static mut NlsUnicodeDefaultChar: u16 = 0;

#[unsafe(no_mangle)]
pub extern "system" fn RtlpDowncaseUnicodeChar(Source: u16) -> u16 {
    if Source < b'A' as u16 {
        return Source;
    }
    if Source <= b'Z' as u16 {
        return Source + (b'a' - b'A') as u16;
    }
    Source
}

#[unsafe(no_mangle)]
pub extern "system" fn RtlDowncaseUnicodeChar(Source: u16) -> u16 {
    RtlpDowncaseUnicodeChar(Source)
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn RtlMultiByteToUnicodeN(
    UnicodeString: *mut u16,
    UnicodeSize: u32,
    ResultSize: *mut u32,
    MbString: *const c_char,
    MbSize: u32,
) -> NTSTATUS {
    // single-byte code page
    let size = MbSize.min(UnicodeSize / 2);
    if !ResultSize.is_null() {
        *ResultSize = size * 2;
    }
    for i in 0..size as usize {
        *UnicodeString.add(i) = *MbString.add(i) as u8 as u16;
    }
    STATUS_SUCCESS
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn RtlMultiByteToUnicodeSize(
    UnicodeSize: *mut u32,
    _MbString: *const c_char,
    MbSize: u32,
) -> NTSTATUS {
    // single-byte code page
    *UnicodeSize = MbSize.wrapping_mul(2);
    STATUS_SUCCESS
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn RtlUnicodeToMultiByteN(
    mut MbString: *mut c_char,
    MbSize: u32,
    ResultSize: *mut u32,
    mut UnicodeString: *const u16,
    UnicodeSize: u32,
) -> NTSTATUS {
    // single-byte code page; `MbSize * sizeof(WCHAR)` is SIZE_T arithmetic in the C
    let size = if UnicodeSize as usize > (MbSize as usize).wrapping_mul(2) { MbSize } else { UnicodeSize / 2 };
    if !ResultSize.is_null() {
        *ResultSize = size;
    }
    for _ in 0..size {
        // A character whose high byte is set cannot be trivially demoted to ANSI.
        let c = *UnicodeString;
        *MbString = if c >> 8 == 0 { c as u8 as c_char } else { NlsOemDefaultChar as u8 as c_char };
        MbString = MbString.add(1);
        UnicodeString = UnicodeString.add(1);
    }
    STATUS_SUCCESS
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn RtlUnicodeToMultiByteSize(
    MbSize: *mut u32,
    _UnicodeString: *const u16,
    UnicodeSize: u32,
) -> NTSTATUS {
    // single-byte code page
    *MbSize = UnicodeSize / 2;
    STATUS_SUCCESS
}

#[unsafe(no_mangle)]
pub extern "system" fn RtlpUpcaseUnicodeChar(Source: u16) -> u16 {
    if Source < b'a' as u16 {
        return Source;
    }
    if Source <= b'z' as u16 {
        return Source - (b'a' - b'A') as u16;
    }
    Source
}

#[unsafe(no_mangle)]
pub extern "system" fn RtlUpcaseUnicodeChar(Source: u16) -> u16 {
    RtlpUpcaseUnicodeChar(Source)
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn RtlUpcaseUnicodeToMultiByteN(
    mut MbString: *mut c_char,
    MbSize: u32,
    ResultSize: *mut u32,
    mut UnicodeString: *const u16,
    UnicodeSize: u32,
) -> NTSTATUS {
    // single-byte code page
    let size = if UnicodeSize as usize > (MbSize as usize).wrapping_mul(2) { MbSize } else { UnicodeSize / 2 };
    if !ResultSize.is_null() {
        *ResultSize = size;
    }
    for _ in 0..size {
        let up = RtlpUpcaseUnicodeChar(*UnicodeString);
        *MbString = if up >> 8 == 0 { up as u8 as c_char } else { NlsOemDefaultChar as u8 as c_char };
        MbString = MbString.add(1);
        UnicodeString = UnicodeString.add(1);
    }
    STATUS_SUCCESS
}

#[unsafe(no_mangle)]
pub extern "system" fn RtlUpperChar(Source: c_char) -> c_char {
    // CHAR is signed, so every byte >= 0x80 takes the first branch and comes back unchanged.
    if Source <= b'z' as c_char {
        if Source >= b'a' as c_char {
            Source ^ (b'a' - b'A') as c_char
        } else {
            Source
        }
    } else {
        RtlpUpcaseUnicodeChar(Source as u16) as c_char
    }
}

// Stubbed OEM helpers that should not be used in the boot loader, but that the rest of the
// RTL's unicode.c needs to link.

#[unsafe(no_mangle)]
pub unsafe extern "system" fn RtlUnicodeToOemN(
    OemString: *mut c_char,
    OemSize: u32,
    ResultSize: *mut u32,
    _UnicodeString: *const u16,
    _UnicodeSize: u32,
) -> NTSTATUS {
    if OemSize != 0 {
        *OemString = 0;
    }
    if !ResultSize.is_null() {
        *ResultSize = 0;
    }
    STATUS_NOT_IMPLEMENTED
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn RtlOemToUnicodeN(
    UnicodeString: *mut u16,
    _UnicodeSize: u32,
    ResultSize: *mut u32,
    _OemString: *const c_char,
    _OemSize: u32,
) -> NTSTATUS {
    if !UnicodeString.is_null() {
        *UnicodeString = 0;
    }
    if !ResultSize.is_null() {
        *ResultSize = 0;
    }
    STATUS_NOT_IMPLEMENTED
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn RtlUpcaseUnicodeToOemN(
    OemString: *mut c_char,
    OemSize: u32,
    ResultSize: *mut u32,
    _UnicodeString: *const u16,
    _UnicodeSize: u32,
) -> NTSTATUS {
    if OemSize != 0 {
        *OemString = 0;
    }
    if !ResultSize.is_null() {
        *ResultSize = 0;
    }
    STATUS_NOT_IMPLEMENTED
}
