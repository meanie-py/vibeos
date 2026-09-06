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

/// classpnp/disk WPP TracePrint: error/warning levels reach the debug serial so
/// failure paths are not silent. Stable rustc cannot DEFINE a C-variadic, so the
/// trailing slots are declared as fixed parameters and replayed as the va_list
/// (see `va_slots_rust`); this used to hand the raw format string to DbgPrint's
/// `%s` and every specifier printed literally with its argument dropped, which
/// on these call sites means the status code you actually wanted was the part
/// thrown away.
#[no_mangle]
pub unsafe extern "C" fn TracePrint(
    lvl: u32,
    _flags: u32,
    fmt: *const i8,
    a0: usize, a1: usize, a2: usize, a3: usize, a4: usize, a5: usize,
    a6: usize, a7: usize, a8: usize, a9: usize, a10: usize, a11: usize,
) {
    if lvl <= TRACE_LEVEL_WARNING && !fmt.is_null() {
        unsafe extern "C" {
            fn DbgPrint(Format: *const i8, ...) -> u32;
        }
        // Format here rather than passing `fmt` on to DbgPrint, so the prefix and
        // the message stay one call and cannot interleave with another CPU's.
        let mut buf: [i8; 512] = [0; 512];
        va_slots_rust::format_slots_z(
            &mut buf,
            fmt,
            &[a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11],
        );
        DbgPrint(c"[CLASSPNP] %s".as_ptr(), buf.as_ptr());
    }
}

/// Storport debug print: no-op.
#[no_mangle]
pub unsafe extern "C" fn StorDebugPrint(_level: u32, _flags: u32, _fmt: *const i8) {}

// ---------------------------------------------------------------------------
// USB-fleet + serial DPRINT-family no-ops (2026-07-18). Same §9.99 story as
// TracePrint above: call sites declare these C-variadic, the defs are
// non-variadic no-ops — fine at machine level (cdecl caller-clean), UB to LLVM
// in one module. Living here keeps def and callers in different LLVM modules,
// so the pattern is fold-proof. Names are driver-unique on purpose; do NOT add
// generic names (DPRINT, TRACE, ERR…) here — they'd leak into every binary
// that links this shim and collide with crates' own defs.

macro_rules! dprint_noop {
    ($($n:ident),* $(,)?) => { $(
        #[no_mangle]
        pub unsafe extern "C" fn $n(_fmt: *const i8) {}
    )* }
}

dprint_noop!(
    DPRINT_EHCI, DPRINT_RH,                    // usbehci
    DPRINT_PNP, DPRINT_SCE,                    // usbhub
    DPRINT_OHCI,                               // usbohci
    DPRINT_IMPL, DPRINT_UHCI,                  // usbuhci
    DPRINT_CORE, DPRINT_URB, DPRINT_QUEUE,     // usbport
    DPRINT_TIMER, DPRINT_INT, DPRINT_MINIPORT, // usbport
);

/// serial.sys TRACE_ (ReactOS debug.h, __noop in retail).
#[no_mangle]
pub unsafe extern "C" fn TRACE_(_component: u32, _fmt: *const i8) {}
