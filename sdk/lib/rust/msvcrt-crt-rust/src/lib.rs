//! msvcrt-crt-rust — scoped Rust cohab body for msvcrt.dll (the user-mode CRT).
//!
//! Each `#[no_mangle]` here overrides the C copy in `dll/win32/msvcrt/*.c`
//! via lld first-match (`--allow-multiple-definition`, this .a linked before
//! `msvcrt_static`). Only the families we've flipped are defined; everything
//! else stays C. The formatting logic is NOT reimplemented — it delegates to
//! `cnt_rust::core_rs`, the difftest-pinned single source of truth shared with
//! libcntpr. The thin ptr/NUL marshalling below mirrors cnt-rust's generated
//! `cabi` shims verbatim (`cnt-rust/src/cabi/generated.rs`); the eventual
//! zero-duplication mechanism is generator-emitted family scoping (tracked in
//! the lane plan, §8).
//!
//! Contract note: these preserve the CRT "misfeatures" exactly — no radix
//! validation, no buffer-size check, no NULL guard on the narrow variants;
//! sign only for radix==10 (else the value's unsigned bit pattern at the
//! declared width). Verified against msvcrt's `ltoa_helper`.
#![no_std]
#![allow(clippy::missing_safety_doc)]

use cnt_rust::core_rs::radix::format_radix;

// ── Wave 1: int↔string radix family ─────────────────────────────────
// Narrow, signed 64-bit.
#[no_mangle]
pub unsafe extern "C" fn _i64toa(value: i64, string: *mut u8, radix: i32) -> *mut u8 {
    let neg = radix == 10 && value < 0;
    let mag = if neg { value.unsigned_abs() } else { value as u64 };
    let f = format_radix(mag, radix as u32, neg);
    let mut o = 0usize;
    if f.negative { *string = b'-'; o = 1; }
    let mut i = 0;
    while i < f.ndigits { *string.add(o + i) = f.digits[i]; i += 1; }
    *string.add(o + f.ndigits) = 0;
    string
}

// Wide, signed 64-bit (NULL dst accepted/returned).
#[no_mangle]
pub unsafe extern "C" fn _i64tow(value: i64, string: *mut u16, radix: i32) -> *mut u16 {
    let neg = radix == 10 && value < 0;
    let mag = if neg { value.unsigned_abs() } else { value as u64 };
    let f = format_radix(mag, radix as u32, neg);
    if !string.is_null() {
        let mut o = 0usize;
        if f.negative { *string = b'-' as u16; o = 1; }
        let mut i = 0;
        while i < f.ndigits { *string.add(o + i) = f.digits[i] as u16; i += 1; }
        *string.add(o + f.ndigits) = 0;
    }
    string
}

// Narrow, signed 32-bit (_itoa == _ltoa: LP32 width).
#[no_mangle]
pub unsafe extern "C" fn _itoa(value: i32, string: *mut u8, radix: i32) -> *mut u8 {
    _ltoa(value, string, radix)
}

// Wide, signed 32-bit.
#[no_mangle]
pub unsafe extern "C" fn _itow(value: i32, string: *mut u16, radix: i32) -> *mut u16 {
    _ltow(value, string, radix)
}

// Narrow, signed long (32-bit LP32): neg+radix!=10 → u32 reinterpret.
#[no_mangle]
pub unsafe extern "C" fn _ltoa(value: i32, string: *mut u8, radix: i32) -> *mut u8 {
    let neg = radix == 10 && value < 0;
    let mag = if neg { value.unsigned_abs() as u64 } else { value as u32 as u64 };
    let f = format_radix(mag, radix as u32, neg);
    let mut o = 0usize;
    if f.negative { *string = b'-'; o = 1; }
    let mut i = 0;
    while i < f.ndigits { *string.add(o + i) = f.digits[i]; i += 1; }
    *string.add(o + f.ndigits) = 0;
    string
}

// Wide, signed long (32-bit LP32; NULL dst accepted/returned).
#[no_mangle]
pub unsafe extern "C" fn _ltow(value: i32, string: *mut u16, radix: i32) -> *mut u16 {
    let neg = radix == 10 && value < 0;
    let mag = if neg { value.unsigned_abs() as u64 } else { value as u32 as u64 };
    let f = format_radix(mag, radix as u32, neg);
    if !string.is_null() {
        let mut o = 0usize;
        if f.negative { *string = b'-' as u16; o = 1; }
        let mut i = 0;
        while i < f.ndigits { *string.add(o + i) = f.digits[i] as u16; i += 1; }
        *string.add(o + f.ndigits) = 0;
    }
    string
}

// Narrow, unsigned 64-bit (pure unsigned, never negative).
#[no_mangle]
pub unsafe extern "C" fn _ui64toa(value: u64, string: *mut u8, radix: i32) -> *mut u8 {
    let f = format_radix(value, radix as u32, false);
    let mut i = 0;
    while i < f.ndigits { *string.add(i) = f.digits[i]; i += 1; }
    *string.add(f.ndigits) = 0;
    string
}

// Wide, unsigned 64-bit (NULL dst accepted/returned).
#[no_mangle]
pub unsafe extern "C" fn _ui64tow(value: u64, string: *mut u16, radix: i32) -> *mut u16 {
    let f = format_radix(value, radix as u32, false);
    if !string.is_null() {
        let mut i = 0;
        while i < f.ndigits { *string.add(i) = f.digits[i] as u16; i += 1; }
        *string.add(f.ndigits) = 0;
    }
    string
}

// Narrow, unsigned long (32-bit).
#[no_mangle]
pub unsafe extern "C" fn _ultoa(value: u32, string: *mut u8, radix: i32) -> *mut u8 {
    let f = format_radix(value as u64, radix as u32, false);
    let mut i = 0;
    while i < f.ndigits { *string.add(i) = f.digits[i]; i += 1; }
    *string.add(f.ndigits) = 0;
    string
}

// Wide, unsigned long (32-bit; NULL dst accepted/returned).
#[no_mangle]
pub unsafe extern "C" fn _ultow(value: u32, string: *mut u16, radix: i32) -> *mut u16 {
    let f = format_radix(value as u64, radix as u32, false);
    if !string.is_null() {
        let mut i = 0;
        while i < f.ndigits { *string.add(i) = f.digits[i] as u16; i += 1; }
        *string.add(f.ndigits) = 0;
    }
    string
}

// ── no_std staticlib runtime bits ───────────────────────────────────
// Required to compile standalone; the dup vs any other Rust staticlib in the
// module is absorbed by --allow-multiple-definition (mirrors crt-dllmain-rust).
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}
