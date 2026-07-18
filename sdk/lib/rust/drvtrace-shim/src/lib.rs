//! Storage-driver trace shims (TracePrint / StorDebugPrint), isolated in their
//! OWN crate on purpose.
//!
//! The call sites are C-variadic (`fn TracePrint(lvl, flags, fmt, ...)`); these
//! definitions are non-variadic and simply ignore the extra args — safe at the
//! machine level on all our ABIs (caller-cleaned varargs), but **undefined
//! behavior to LLVM** when the mismatched def and a variadic decl land in the
//! same LLVM module: instcombine folds every such call site to `ud2` (the
//! 2026-07-18 i386 classpnp bomb set — ClassPnpStartDevice & friends died at
//! their first TracePrint). The workspace builds with `lto = false`, so a
//! separate crate is a separate LLVM module — the def is invisible to callers'
//! modules and the calls stay ordinary symbol references. Do NOT move these
//! back into a driver crate, and do NOT enable LTO over them.
#![no_std]
#![allow(non_snake_case)]

const TRACE_LEVEL_WARNING: u32 = 3; // ETW: 1=critical 2=error 3=warning

/// classpnp/disk WPP TracePrint: error/warning levels forward the (raw) format
/// string to the debug serial so failure paths are not silent; %-specifiers
/// print literally — enough to identify the site.
#[no_mangle]
pub unsafe extern "C" fn TracePrint(lvl: u32, _flags: u32, fmt: *const i8) {
    if lvl <= TRACE_LEVEL_WARNING && !fmt.is_null() {
        unsafe extern "C" {
            fn DbgPrint(Format: *const i8, ...) -> u32;
        }
        DbgPrint(c"[CLASSPNP] %s".as_ptr(), fmt);
    }
}

/// Storport debug print: no-op.
#[no_mangle]
pub unsafe extern "C" fn StorDebugPrint(_level: u32, _flags: u32, _fmt: *const i8) {}
