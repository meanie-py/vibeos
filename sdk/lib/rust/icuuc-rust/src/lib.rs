//! icuuc-rust: `icuuc.dll` (ICU4C common library, UNVERSIONED exports) as a thin layer over Win32
//! code pages. Implements the 16 entry points that CMake's cmake-gui (Qt 6) imports.
//!
//! ABI (ICU4C `ucnv.h` / `ucnv_err.h` / `utypes.h`; the alias table is ICU's convrtrs.txt and the
//! call pattern Qt 6's qstringconverter.cpp — the client this exists for):
//!  * every export is `U_CAPI ... U_EXPORT2` = cdecl (`extern "C"`), unversioned names;
//!  * UChar = u16, UBool = int8_t, UErrorCode = 32-bit enum, UConverterCallbackReason = 32-bit enum;
//!  * `UConverterToUnicodeArgs` / `UConverterFromUnicodeArgs` are natural-alignment C structs
//!    {u16 size; i8 flush; ptr converter; ptr source; ptr sourceLimit; ptr target; ptr targetLimit;
//!    i32* offsets}, so the pointer block starts at offset 4 (i386) / 8 (amd64, arm64) via
//!    repr(C) padding. `size` is filled with sizeof(struct).
//!
//! Design: `UConverter` is private (allocated from the process heap) and holds the callbacks and
//! contexts, an incomplete-input buffer (DBCS lead byte / partial UTF-8 / partial UTF-16) and an
//! overflow pool of already-converted output for each direction. UTF-8, UTF-16LE/BE, US-ASCII and
//! ISO-8859-1 are implemented natively; every other code page goes through kernel32
//! (MultiByteToWideChar / WideCharToMultiByte / GetCPInfoExW / IsDBCSLeadByteEx), isolated behind the
//! `Platform` trait in plat.rs so engine.rs can be tested on the host with a fake code page.
//!
//! Semantics mirrored from ICU:
//!  * The default callbacks are the SUBSTITUTE ones (U+FFFD / the code page's substitution byte);
//!    `ucnv_get*CallBack` returns real function pointers so a client can compare against them.
//!  * On an invalid unit the error is first stored in *err (U_INVALID_CHAR_FOUND = unassigned,
//!    U_ILLEGAL_CHAR_FOUND, U_TRUNCATED_CHAR_FOUND), the bad input is consumed, then the callback
//!    is invoked with reason UCNV_UNASSIGNED (INVALID) or UCNV_ILLEGAL (others). If it leaves *err
//!    failing the conversion stops with that error.
//!  * Callbacks are also invoked with UCNV_RESET / UCNV_CLOSE from ucnv_reset / ucnv_close when they
//!    are not the built-in default.
//!  * U_BUFFER_OVERFLOW_ERROR is returned when output does not fit; the surplus is held in the
//!    converter and drained first by the next call. toUCountPending / fromUCountPending count pending
//!    INPUT only (bytes / UChars), like ICU.
//!  * Offsets: relative to the start of this call's source; -1 for output that originates in input
//!    of an earlier call.
//!  * getMaxCharSize returns ICU's values (UTF-8 = 3, UTF-16 = 2, EUC-JP = 3, GB18030 = 4 ...).
//!
//! Imports note: memcpy/memmove may be emitted by the compiler (not hand-written here).

#![cfg_attr(not(test), no_std)]
#![allow(non_camel_case_types, non_snake_case, clippy::missing_safety_doc)]

use core::ffi::c_void;

mod codec;
mod engine;
mod names;
mod plat;
mod sys;
#[cfg(test)]
mod tests;

pub use engine::UConverter;
use plat::Plat;

pub type CChar = i8;
pub type UErrorCode = i32;

pub const U_ZERO_ERROR: i32 = 0;
pub const U_ILLEGAL_ARGUMENT_ERROR: i32 = 1;
pub const U_FILE_ACCESS_ERROR: i32 = 4;
pub const U_INTERNAL_PROGRAM_ERROR: i32 = 5;
pub const U_MEMORY_ALLOCATION_ERROR: i32 = 7;
pub const U_INVALID_CHAR_FOUND: i32 = 10;
pub const U_TRUNCATED_CHAR_FOUND: i32 = 11;
pub const U_ILLEGAL_CHAR_FOUND: i32 = 12;
pub const U_BUFFER_OVERFLOW_ERROR: i32 = 15;

pub const UCNV_UNASSIGNED: i32 = 0;
pub const UCNV_ILLEGAL: i32 = 1;
pub const UCNV_IRREGULAR: i32 = 2;
pub const UCNV_RESET: i32 = 3;
pub const UCNV_CLOSE: i32 = 4;
pub const UCNV_CLONE: i32 = 5;

#[repr(C)]
pub struct UConverterToUnicodeArgs {
    pub size: u16,
    pub flush: i8,
    pub converter: *mut UConverter,
    pub source: *const CChar,
    pub source_limit: *const CChar,
    pub target: *mut u16,
    pub target_limit: *const u16,
    pub offsets: *mut i32,
}

#[repr(C)]
pub struct UConverterFromUnicodeArgs {
    pub size: u16,
    pub flush: i8,
    pub converter: *mut UConverter,
    pub source: *const u16,
    pub source_limit: *const u16,
    pub target: *mut CChar,
    pub target_limit: *const CChar,
    pub offsets: *mut i32,
}

pub type UToCb = unsafe extern "C" fn(
    context: *const c_void, args: *mut UConverterToUnicodeArgs, code_units: *const CChar,
    length: i32, reason: i32, err: *mut UErrorCode,
);
pub type UFromCb = unsafe extern "C" fn(
    context: *const c_void, args: *mut UConverterFromUnicodeArgs, code_units: *const u16,
    length: i32, code_point: i32, reason: i32, err: *mut UErrorCode,
);

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

// staticlib needs an eh personality (no_std cohab).
#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}

#[no_mangle]
pub unsafe extern "C" fn ucnv_open(name: *const CChar, err: *mut UErrorCode) -> *mut UConverter {
    engine::open(name, err)
}

#[no_mangle]
pub unsafe extern "C" fn ucnv_close(cnv: *mut UConverter) {
    engine::close(cnv)
}

#[no_mangle]
pub unsafe extern "C" fn ucnv_reset(cnv: *mut UConverter) {
    engine::reset(cnv)
}

#[no_mangle]
pub unsafe extern "C" fn ucnv_getName(cnv: *const UConverter, err: *mut UErrorCode) -> *const CChar {
    if err.is_null() || *err > 0 { return core::ptr::null(); }
    if cnv.is_null() { *err = U_ILLEGAL_ARGUMENT_ERROR; return core::ptr::null(); }
    (*cnv).entry.name.as_ptr() as *const CChar
}

#[no_mangle]
pub unsafe extern "C" fn ucnv_getStandardName(
    name: *const CChar, standard: *const CChar, err: *mut UErrorCode,
) -> *const CChar {
    if err.is_null() || *err > 0 || name.is_null() || standard.is_null() { return core::ptr::null(); }
    let n = core::slice::from_raw_parts(name as *const u8, engine::cstr_len(name));
    let s = core::slice::from_raw_parts(standard as *const u8, engine::cstr_len(standard));
    let e = match names::find(n) { Some(e) => e, None => return core::ptr::null() };
    let tag = if names::names_match(s, b"IANA") {
        e.iana
    } else if names::names_match(s, b"MIME") {
        e.mime
    } else {
        None // JAVA / IBM / WINDOWS / UTR22 tags are not tracked
    };
    match tag { Some(t) => t.as_ptr() as *const CChar, None => core::ptr::null() }
}

#[no_mangle]
pub unsafe extern "C" fn ucnv_getMaxCharSize(cnv: *const UConverter) -> i8 {
    if cnv.is_null() { 0 } else { (*cnv).entry.max }
}

#[no_mangle]
pub unsafe extern "C" fn ucnv_toUnicode(
    cnv: *mut UConverter, target: *mut *mut u16, target_limit: *const u16, source: *mut *const CChar,
    source_limit: *const CChar, offsets: *mut i32, flush: i8, err: *mut UErrorCode,
) {
    engine::to_unicode(cnv, target, target_limit, source, source_limit, offsets, flush, err)
}

#[no_mangle]
pub unsafe extern "C" fn ucnv_fromUnicode(
    cnv: *mut UConverter, target: *mut *mut CChar, target_limit: *const CChar, source: *mut *const u16,
    source_limit: *const u16, offsets: *mut i32, flush: i8, err: *mut UErrorCode,
) {
    engine::from_unicode(cnv, target as *mut *mut u8, target_limit, source, source_limit, offsets, flush, err)
}

#[no_mangle]
pub unsafe extern "C" fn ucnv_toUCountPending(cnv: *const UConverter, err: *mut UErrorCode) -> i32 {
    if err.is_null() || *err > 0 { return -1; }
    if cnv.is_null() { *err = U_ILLEGAL_ARGUMENT_ERROR; return -1; }
    (*cnv).to_in_len as i32
}

#[no_mangle]
pub unsafe extern "C" fn ucnv_fromUCountPending(cnv: *const UConverter, err: *mut UErrorCode) -> i32 {
    if err.is_null() || *err > 0 { return -1; }
    if cnv.is_null() { *err = U_ILLEGAL_ARGUMENT_ERROR; return -1; }
    (*cnv).from_in_len as i32
}

#[no_mangle]
pub unsafe extern "C" fn ucnv_setToUCallBack(
    cnv: *mut UConverter, new_action: Option<UToCb>, new_context: *const c_void,
    old_action: *mut UToCb, old_context: *mut *const c_void, err: *mut UErrorCode,
) {
    if err.is_null() || *err > 0 { return; }
    let new_action = match new_action {
        Some(a) if !cnv.is_null() => a,
        _ => { *err = U_ILLEGAL_ARGUMENT_ERROR; return; }
    };
    if !old_action.is_null() { *old_action = (*cnv).to_cb; }
    if !old_context.is_null() { *old_context = (*cnv).to_ctx; }
    (*cnv).to_cb = new_action;
    (*cnv).to_ctx = new_context;
}

#[no_mangle]
pub unsafe extern "C" fn ucnv_getToUCallBack(cnv: *const UConverter, action: *mut UToCb, context: *mut *const c_void) {
    if cnv.is_null() { return; }
    if !action.is_null() { *action = (*cnv).to_cb; }
    if !context.is_null() { *context = (*cnv).to_ctx; }
}

#[no_mangle]
pub unsafe extern "C" fn ucnv_setFromUCallBack(
    cnv: *mut UConverter, new_action: Option<UFromCb>, new_context: *const c_void,
    old_action: *mut UFromCb, old_context: *mut *const c_void, err: *mut UErrorCode,
) {
    if err.is_null() || *err > 0 { return; }
    let new_action = match new_action {
        Some(a) if !cnv.is_null() => a,
        _ => { *err = U_ILLEGAL_ARGUMENT_ERROR; return; }
    };
    if !old_action.is_null() { *old_action = (*cnv).from_cb; }
    if !old_context.is_null() { *old_context = (*cnv).from_ctx; }
    (*cnv).from_cb = new_action;
    (*cnv).from_ctx = new_context;
}

#[no_mangle]
pub unsafe extern "C" fn ucnv_getFromUCallBack(cnv: *const UConverter, action: *mut UFromCb, context: *mut *const c_void) {
    if cnv.is_null() { return; }
    if !action.is_null() { *action = (*cnv).from_cb; }
    if !context.is_null() { *context = (*cnv).from_ctx; }
}

/// Write UChars to args->target (offsets set to `offset_index`); surplus goes to the converter's
/// overflow pool and U_BUFFER_OVERFLOW_ERROR is set.
#[no_mangle]
pub unsafe extern "C" fn ucnv_cbToUWriteUChars(
    args: *mut UConverterToUnicodeArgs, source: *const u16, length: i32, offset_index: i32, err: *mut UErrorCode,
) {
    if err.is_null() || *err > 0 { return; }
    if args.is_null() || source.is_null() && length > 0 { *err = U_ILLEGAL_ARGUMENT_ERROR; return; }
    engine::write_uchars(args, source, length, offset_index, err)
}

/// Convert the UChars `*source..source_limit` with this converter (no callbacks) into args->target.
#[no_mangle]
pub unsafe extern "C" fn ucnv_cbFromUWriteUChars(
    args: *mut UConverterFromUnicodeArgs, source: *mut *const u16, source_limit: *const u16,
    offset_index: i32, err: *mut UErrorCode,
) {
    engine::cb_from_u_write_uchars(args, source, source_limit, offset_index, err)
}

#[allow(dead_code)]
type _AssertPlat = Plat;

// Layout guards: {u16 size; i8 flush; 6 pointers} => 28 bytes on i386, 56 on amd64/arm64.
const _: () = {
    let want = if core::mem::size_of::<usize>() == 4 { 28 } else { 56 };
    assert!(core::mem::size_of::<UConverterToUnicodeArgs>() == want);
    assert!(core::mem::size_of::<UConverterFromUnicodeArgs>() == want);
    assert!(core::mem::offset_of!(UConverterToUnicodeArgs, converter) == core::mem::size_of::<usize>());
    assert!(core::mem::offset_of!(UConverterToUnicodeArgs, offsets) == want - core::mem::size_of::<usize>());
};
