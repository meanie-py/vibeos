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

use core::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

/// Repeat limiter, keyed by format string. Some warnings are steady state, not
/// events: classpnp's media-change poll of an empty removable disk (the Pi 5's
/// Pico before it serves an image) fails every second with "Device not ready" and
/// a sense breakdown, ~435 lines a minute forever, and at 115200 baud each line
/// holds its CPU until the UART drains. Each message prints its first
/// `REPEAT_FREE` occurrences, then every `REPEAT_EVERY`th with the count, so a
/// first failure is never silent and a stuck one still shows it is alive.
const REPEAT_FREE: u32 = 8;
const REPEAT_EVERY: u32 = 1000;
const REPEAT_SLOTS: usize = 64;
static REPEAT_KEY: [AtomicUsize; REPEAT_SLOTS] = [const { AtomicUsize::new(0) }; REPEAT_SLOTS];
static REPEAT_COUNT: [AtomicU32; REPEAT_SLOTS] = [const { AtomicU32::new(0) }; REPEAT_SLOTS];

/// Whether occurrence `n` (0 = unlimited) is printed.
fn shown(n: u32) -> bool {
    n <= REPEAT_FREE || n % REPEAT_EVERY == 0
}

/// This occurrence's number for `fmt` (1-based), or 0 when the table is full
/// (then nothing is limited).
fn occurrence(fmt: *const i8) -> u32 {
    let key = fmt as usize;
    for i in 0..REPEAT_SLOTS {
        let k = REPEAT_KEY[i].load(Ordering::Acquire);
        let mine = k == key
            || (k == 0
                && match REPEAT_KEY[i].compare_exchange(0, key, Ordering::AcqRel, Ordering::Acquire) {
                    Ok(_) => true,
                    Err(won) => won == key,
                });
        if mine {
            return REPEAT_COUNT[i].fetch_add(1, Ordering::Relaxed).wrapping_add(1);
        }
    }
    0
}

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
        let n = occurrence(fmt);
        if !shown(n) {
            return;
        }
        // Format here rather than passing `fmt` on to DbgPrint, so the prefix and
        // the message stay one call and cannot interleave with another CPU's.
        let mut buf: [i8; 512] = [0; 512];
        va_slots_rust::format_slots_z(
            &mut buf,
            fmt,
            &[a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11],
        );
        if n > REPEAT_FREE {
            DbgPrint(c"[CLASSPNP] (x%u) %s".as_ptr(), n, buf.as_ptr());
        } else {
            DbgPrint(c"[CLASSPNP] %s".as_ptr(), buf.as_ptr());
            if n == REPEAT_FREE {
                DbgPrint(c"[CLASSPNP] (the line above repeats; from now on every %uth is shown)\n".as_ptr(), REPEAT_EVERY);
            }
        }
    }
}

/// Storport debug print: no-op.
#[no_mangle]
pub unsafe extern "C" fn StorDebugPrint(_level: u32, _flags: u32, _fmt: *const i8) {}

// ---------------------------------------------------------------------------
// USB-fleet + serial debug prints: MACROS, never functions.
//
// In C these are `do { if(0) { DbgPrint(...); } } while(0)` or `__noop` in the
// builds we ship, so their ARGUMENTS ARE NEVER EVALUATED. They used to be no-op
// extern functions here, and a function call evaluates every argument first.
// usbport's USBPORT_QueueTransferUrb ends with a DPRINT_URB loop over
// `*Urb->TransferBuffer`, read AFTER the URB was handed to the endpoint. On two
// CPUs the transfer completed on the other one, which set TransferBufferLength
// while usbstor's MDL-only URB had TransferBuffer NULL: a NULL read at
// DISPATCH_LEVEL, raised as an in-page error that cdfs caught and turned into a
// failed paging read. smss or csrss then died on about half of all amd64-bios
// boots at -smp 2. The C never executes that read.
//
// Each macro type-checks its arguments (as the C's `if(0)` does) and runs none
// of them. There are deliberately no same-named symbols any more: a call site
// that slips back to `NAME(...)` fails to link instead of quietly evaluating.
// Names are driver-unique on purpose; do NOT add generic names (DPRINT, TRACE,
// ERR...) — crates define their own.

macro_rules! dead_dprint {
    ($d:tt $($n:ident),* $(,)?) => { $(
        #[macro_export]
        macro_rules! $n {
            ($d($d a:expr),* $d(,)?) => {
                if false {
                    $d( let _ = $d a; )*
                }
            };
        }
    )* }
}

dead_dprint!($
    DPRINT_EHCI, DPRINT_RH,                    // usbehci
    DPRINT_PNP, DPRINT_SCE, DPRINT_IOCTL,      // usbhub
    DPRINT_OHCI,                               // usbohci
    DPRINT_IMPL, DPRINT_UHCI,                  // usbuhci
    DPRINT_CORE, DPRINT_URB, DPRINT_QUEUE,     // usbport
    DPRINT_TIMER, DPRINT_INT, DPRINT_MINIPORT, // usbport
    TRACE_,                                    // serial (ReactOS debug.h)
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_format_is_limited_on_its_own() {
        let (a, b) = (c"a".as_ptr(), c"b".as_ptr());
        let shown_a = (0..3000).filter(|_| shown(occurrence(a))).count();
        assert_eq!(shown_a, REPEAT_FREE as usize + 3); // 1..=8, then 1000, 2000, 3000
        assert_eq!(occurrence(b), 1);
        assert!(shown(0), "a full table limits nothing");
    }
}
