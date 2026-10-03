//! `misc/exception_filter.cpp` -- the default exception-action table
//! (`__acrt_exception_action_table` and its four `size_t const` companions)
//! and the SEH filters (`_seh_filter_dll`, `_seh_filter_exe`) that translate
//! OS exceptions into C signals for `signal.rs`/`raise`.
//!
//! `__crt_signal_action_t` (`exception_number: u32, signal_number: i32,
//! action: usize`) is `#[repr(C)]` over plain fixed-width fields, so its
//! layout follows ordinary C struct-layout rules on all three targets without
//! needing a measured/pinned offset (unlike `AcrtPtd`, which has
//! locale/exception-table-derived members whose OWN internal layout is not
//! reproducible without measurement) -- defined in `signal.rs`, which both
//! files share.
//!
//! ## Details that are silent if wrong
//!
//! * Exceptions mapping to the SAME signal are grouped CONTIGUOUSLY in the
//!   table (`STATUS_FLOAT_*` all map to `SIGFPE`, 9 entries starting at index
//!   3) -- `signal()`/`raise()` both walk a contiguous RANGE from the first
//!   match rather than re-searching, so the table's ORDER is load-bearing,
//!   not just its contents.
//! * `_seh_filter_dll` only ever forwards a SPECIFIC C++ exception code
//!   (`'msc' | 0xE0000000` = `0x6D737B00 | 0xE0000000` = `0xE06D7363`, the
//!   MSVC C++ EH signature) to `_seh_filter_exe`; every other exception code
//!   returns `EXCEPTION_CONTINUE_SEARCH` immediately, without EVER consulting
//!   the per-thread exception-action table.
//! * `_seh_filter_exe`'s `SIG_DFL` and unmapped-exception cases both return
//!   `EXCEPTION_CONTINUE_SEARCH` (pass it on) -- indistinguishable to the
//!   caller; only `SIG_DIE` (resets the action to `SIG_DFL` first, "in case
//!   of recursion") returns `EXCEPTION_EXECUTE_HANDLER`, and only `SIG_IGN`
//!   or a real handler return `EXCEPTION_CONTINUE_EXECUTION`.
//! * For `SIGFPE`, ALL NINE floating-point table entries are reset to
//!   `SIG_DFL` before the user handler runs (not just the one that fired),
//!   and `_tfpecode` is saved/restored around the call to support a nested
//!   fault during the handler; the exception-number -> `_FPE_*` mapping is a
//!   `switch` with NO default (an unrecognized `STATUS_FLOAT_*` code --
//!   impossible given the table's own contents, but the C's behavior if it
//!   ever happened -- leaves `_tfpecode` at whatever it was, including a
//!   stale value from a previous fault).
//! * For every OTHER (non-`SIGFPE`) signal, only the ONE matching entry's
//!   `_action` is reset to `SIG_DFL` before the handler runs.
//! * The handler is called through a REINTERPRETED two-argument function
//!   pointer type for `SIGFPE` specifically (`void(int, int)`, passing
//!   `SIGFPE` and the computed `_tfpecode`), matching old two-arg MS SIGFPE
//!   handlers; every other signal calls the ordinary one-argument form.
//! * `ptd->_tpxcptinfoptrs` is swapped to the CURRENT exception's pointer for
//!   the duration of the handler call and restored after, for EVERY handled
//!   signal (not just the FP ones) -- so a nested signal/exception during a
//!   user handler sees the INNER exception's info, and the outer one is
//!   restored correctly on return.

use core::ffi::{c_int, c_ulong};

use crate::misc::signal::{CrtSignalActionT, SIG_DFL, SIG_DIE, SIG_IGN, SIGFPE, SIGILL, SIGSEGV};
use crate::ptd::AcrtPtd;

extern "C" {
    fn __acrt_getptd_noexit() -> *mut AcrtPtd;
}

// `STATUS_*` (`sdk/include/psdk/ntstatus.h`), as plain `NTSTATUS` (`LONG`)
// values reinterpreted unsigned -- matching the table's `unsigned long
// _exception_number` field.
const STATUS_ACCESS_VIOLATION: u32 = 0xC000_0005;
const STATUS_ILLEGAL_INSTRUCTION: u32 = 0xC000_001D;
const STATUS_PRIVILEGED_INSTRUCTION: u32 = 0xC000_0096;
const STATUS_FLOAT_DENORMAL_OPERAND: u32 = 0xC000_008D;
const STATUS_FLOAT_DIVIDE_BY_ZERO: u32 = 0xC000_008E;
const STATUS_FLOAT_INEXACT_RESULT: u32 = 0xC000_008F;
const STATUS_FLOAT_INVALID_OPERATION: u32 = 0xC000_0090;
const STATUS_FLOAT_OVERFLOW: u32 = 0xC000_0091;
const STATUS_FLOAT_STACK_CHECK: u32 = 0xC000_0092;
const STATUS_FLOAT_UNDERFLOW: u32 = 0xC000_0093;
const STATUS_FLOAT_MULTIPLE_FAULTS: u32 = 0xC000_02B4;
const STATUS_FLOAT_MULTIPLE_TRAPS: u32 = 0xC000_02B5;

const _FPE_INVALID: c_int = 0x81;
const _FPE_DENORMAL: c_int = 0x82;
const _FPE_ZERODIVIDE: c_int = 0x83;
const _FPE_OVERFLOW: c_int = 0x84;
const _FPE_UNDERFLOW: c_int = 0x85;
const _FPE_INEXACT: c_int = 0x86;
const _FPE_STACKOVERFLOW: c_int = 0x8A;
const _FPE_MULTIPLE_TRAPS: c_int = 0x8D;
const _FPE_MULTIPLE_FAULTS: c_int = 0x8E;

const EXCEPTION_CONTINUE_EXECUTION: c_int = -1;
const EXCEPTION_CONTINUE_SEARCH: c_int = 0;
const EXCEPTION_EXECUTE_HANDLER: c_int = 1;

/// The MSVC C++ EH exception code: `'msc' | 0xE0000000` = `0x6D737300 |
/// 0xE0000000`. NOTE: `'msc'` as a 4-char multichar literal packs
/// `'\0','m','s','c'` big-endian-in-value (`0x006D7363`) under MSVC's own
/// multichar rules (left-to-right, most significant byte first) -- so the
/// full value is `0x006D7363 | 0xE0000000` = `0xE06D7363`, the well-known
/// `EH_EXCEPTION_NUMBER`.
const MSC_EH_EXCEPTION: u32 = 0xE06D_7363;

/// `__acrt_exception_action_table`. Order matters -- see the module doc.
#[no_mangle]
pub static __acrt_exception_action_table: [CrtSignalActionT; 12] = [
    CrtSignalActionT { exception_number: STATUS_ACCESS_VIOLATION, signal_number: SIGSEGV, action: SIG_DFL },
    CrtSignalActionT { exception_number: STATUS_ILLEGAL_INSTRUCTION, signal_number: SIGILL, action: SIG_DFL },
    CrtSignalActionT { exception_number: STATUS_PRIVILEGED_INSTRUCTION, signal_number: SIGILL, action: SIG_DFL },
    CrtSignalActionT { exception_number: STATUS_FLOAT_DENORMAL_OPERAND, signal_number: SIGFPE, action: SIG_DFL },
    CrtSignalActionT { exception_number: STATUS_FLOAT_DIVIDE_BY_ZERO, signal_number: SIGFPE, action: SIG_DFL },
    CrtSignalActionT { exception_number: STATUS_FLOAT_INEXACT_RESULT, signal_number: SIGFPE, action: SIG_DFL },
    CrtSignalActionT { exception_number: STATUS_FLOAT_INVALID_OPERATION, signal_number: SIGFPE, action: SIG_DFL },
    CrtSignalActionT { exception_number: STATUS_FLOAT_OVERFLOW, signal_number: SIGFPE, action: SIG_DFL },
    CrtSignalActionT { exception_number: STATUS_FLOAT_STACK_CHECK, signal_number: SIGFPE, action: SIG_DFL },
    CrtSignalActionT { exception_number: STATUS_FLOAT_UNDERFLOW, signal_number: SIGFPE, action: SIG_DFL },
    CrtSignalActionT { exception_number: STATUS_FLOAT_MULTIPLE_FAULTS, signal_number: SIGFPE, action: SIG_DFL },
    CrtSignalActionT { exception_number: STATUS_FLOAT_MULTIPLE_TRAPS, signal_number: SIGFPE, action: SIG_DFL },
];

/// `__acrt_signal_action_first_fpe_index`.
#[no_mangle]
pub static __acrt_signal_action_first_fpe_index: usize = 3;
/// `__acrt_signal_action_fpe_count`.
#[no_mangle]
pub static __acrt_signal_action_fpe_count: usize = 9;
/// `__acrt_signal_action_table_size` (bytes).
#[no_mangle]
pub static __acrt_signal_action_table_size: usize = 12 * core::mem::size_of::<CrtSignalActionT>();
/// `__acrt_signal_action_table_count`.
#[no_mangle]
pub static __acrt_signal_action_table_count: usize = 12;

/// `xcptlookup`.
unsafe fn xcptlookup(xcptnum: c_ulong, table: *mut CrtSignalActionT) -> *mut CrtSignalActionT {
    for i in 0..__acrt_signal_action_table_count {
        let p = table.add(i);
        if (*p).exception_number == xcptnum as u32 {
            return p;
        }
    }
    core::ptr::null_mut()
}

/// `_seh_filter_dll`.
#[no_mangle]
pub unsafe extern "C" fn _seh_filter_dll(xcptnum: c_ulong, pxcptinfoptrs: *mut core::ffi::c_void) -> c_int {
    if xcptnum != MSC_EH_EXCEPTION as c_ulong {
        return EXCEPTION_CONTINUE_SEARCH;
    }
    _seh_filter_exe(xcptnum, pxcptinfoptrs)
}

/// `_seh_filter_exe`.
#[no_mangle]
pub unsafe extern "C" fn _seh_filter_exe(xcptnum: c_ulong, pxcptinfoptrs: *mut core::ffi::c_void) -> c_int {
    let ptd = __acrt_getptd_noexit();
    if ptd.is_null() {
        return EXCEPTION_CONTINUE_SEARCH;
    }

    let pxcptact = xcptlookup(xcptnum, (*ptd).pxcptacttab.cast());
    if pxcptact.is_null() {
        return EXCEPTION_CONTINUE_SEARCH;
    }

    let phandler = (*pxcptact).action;

    if phandler == SIG_DFL {
        return EXCEPTION_CONTINUE_SEARCH;
    }
    if phandler == SIG_DIE {
        (*pxcptact).action = SIG_DFL;
        return EXCEPTION_EXECUTE_HANDLER;
    }
    if phandler == SIG_IGN {
        return EXCEPTION_CONTINUE_EXECUTION;
    }

    let old_pxcptinfoptrs = (*ptd).tpxcptinfoptrs;
    (*ptd).tpxcptinfoptrs = pxcptinfoptrs;

    if (*pxcptact).signal_number == SIGFPE {
        let table: *mut CrtSignalActionT = (*ptd).pxcptacttab.cast();
        let first = table.add(__acrt_signal_action_first_fpe_index);
        for i in 0..__acrt_signal_action_fpe_count {
            (*first.add(i)).action = SIG_DFL;
        }

        let old_fpecode = (*ptd).tfpecode;

        match (*pxcptact).exception_number {
            STATUS_FLOAT_DIVIDE_BY_ZERO => (*ptd).tfpecode = _FPE_ZERODIVIDE,
            STATUS_FLOAT_INVALID_OPERATION => (*ptd).tfpecode = _FPE_INVALID,
            STATUS_FLOAT_OVERFLOW => (*ptd).tfpecode = _FPE_OVERFLOW,
            STATUS_FLOAT_UNDERFLOW => (*ptd).tfpecode = _FPE_UNDERFLOW,
            STATUS_FLOAT_DENORMAL_OPERAND => (*ptd).tfpecode = _FPE_DENORMAL,
            STATUS_FLOAT_INEXACT_RESULT => (*ptd).tfpecode = _FPE_INEXACT,
            STATUS_FLOAT_STACK_CHECK => (*ptd).tfpecode = _FPE_STACKOVERFLOW,
            STATUS_FLOAT_MULTIPLE_TRAPS => (*ptd).tfpecode = _FPE_MULTIPLE_TRAPS,
            STATUS_FLOAT_MULTIPLE_FAULTS => (*ptd).tfpecode = _FPE_MULTIPLE_FAULTS,
            _ => {}
        }

        let handler: unsafe extern "C" fn(c_int, c_int) = core::mem::transmute(phandler);
        handler(SIGFPE, (*ptd).tfpecode);

        (*ptd).tfpecode = old_fpecode;
    } else {
        (*pxcptact).action = SIG_DFL;
        let handler: unsafe extern "C" fn(c_int) = core::mem::transmute(phandler);
        handler((*pxcptact).signal_number);
    }

    (*ptd).tpxcptinfoptrs = old_pxcptinfoptrs;

    EXCEPTION_CONTINUE_EXECUTION
}
