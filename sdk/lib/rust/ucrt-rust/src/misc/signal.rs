//! `misc/signal.cpp` -- `signal()`, `raise()`, `__acrt_initialize_signal_handlers`,
//! `__acrt_get_sigabrt_handler`, `__fpecode`, `__pxcptinfoptrs`.
//!
//! `__crt_signal_handler_t` (`void(__cdecl*)(int)`) is represented as a plain
//! `usize` bit pattern throughout, INCLUDING at the `extern "C"` boundary for
//! `signal`/`raise`'s public parameter/return -- a pointer-sized value is
//! passed identically in every calling convention regardless of whether its
//! Rust-level type is `usize` or a function-pointer type, so this is
//! ABI-compatible with any caller declaring the "real" signature, and it
//! sidesteps constructing `Option<fn(..)>` values from the sentinel codes
//! (`SIG_IGN`=1, `SIG_GET`=2, ...) that are never actually valid code
//! addresses -- exactly as the C's own `reinterpret_cast`s never call through
//! them either, only compare.
//!
//! `CrtSignalActionT` (shared with `exception_filter.rs`, which owns and
//! iterates the actual table) is `#[repr(C)]` over three fixed-width fields,
//! so its layout follows ordinary C struct-layout rules without needing a
//! measured/pinned offset.
//!
//! ## Details that are silent if wrong
//!
//! * `ctrlc_action`/`ctrlbreak_action`/`abort_action`/`term_action` are
//!   PROCESS-wide (one plain `static`, no per-thread storage) and store the
//!   handler ENCODED (`ptr ^ __security_cookie`, `__crt_fast_encode_pointer`
//!   -- see `invalid_parameter.rs`'s module doc for why this is XOR, not
//!   `EncodePointer`), so a raw memory dump of these statics never shows a
//!   real code address directly.
//! * `SIGABRT` and `SIGABRT_COMPAT` are DIFFERENT constants in real UCRT
//!   headers (22 and 6 respectively -- `sdk/include/ucrt/signal.h`), NOT the
//!   mingw `crt/signal.h` value (6 for both) other C/Rust in this tree may
//!   use; both map to the SAME `abort_action` global here, matching the C's
//!   `case SIGABRT: case SIGABRT_COMPAT:` (distinct case labels only because
//!   the UCRT header's values differ).
//! * `is_unsupported_signal`/`is_console_signal`: this build is NOT
//!   `_UCRT_ENCLAVE_BUILD`, so `is_unsupported_signal` only ever rejects
//!   `SIG_ACK`/`SIG_SGE` as the ACTION (never rejects a signum), and
//!   `is_console_signal` is true only for `SIGINT`/`SIGBREAK`.
//! * `signal()` for `SIGINT`/`SIGBREAK`/`SIGABRT`/`SIGABRT_COMPAT`/`SIGTERM`
//!   installs `ctrlevent_capture` via `SetConsoleCtrlHandler` on the FIRST
//!   `SIGINT`/`SIGBREAK` call ever made (any of the two), never uninstalls
//!   it, and if that install fails sets `_doserrno = GetLastError()` and
//!   fails the WHOLE call (`signal_failed`) even though the requested
//!   action's global WAS still read out first -- but the write of the new
//!   action never happens in that branch (checked in the same critical
//!   section, unlock happens before the `signal_failed` return).
//! * `sigact == SIG_GET` reads the current action WITHOUT installing
//!   anything (skips the "if `sigact != SIG_GET`" write both in the
//!   console-signal branch and the exception-table branch) -- but STILL
//!   goes through the `SetConsoleCtrlHandler`-install-on-first-use logic for
//!   console signals, since that check doesn't look at `sigact` at all.
//! * For `SIGFPE`/`SIGILL`/`SIGSEGV`, the per-thread exception-action table
//!   is lazily `malloc`'d (from the SHARED, `static` default table) the
//!   FIRST time `signal()` is called for ANY of these three on that thread
//!   -- `ptd->_pxcptacttab == __acrt_exception_action_table` (pointer
//!   equality against the process-wide default) is the "not yet allocated"
//!   test; once allocated, it never shrinks back.
//! * Setting a handler for `SIGFPE`/`SIGILL`/`SIGSEGV` updates EVERY
//!   contiguous table entry mapped to that signal (the whole `STATUS_FLOAT_*`
//!   run for `SIGFPE`, both `SIGILL` entries), not just the first found.
//! * `raise()`'s SIGFPE handling ALSO resets every FP table entry to
//!   `SIG_DFL` before calling the handler (matching `_seh_filter_exe`), and
//!   swaps `_tpxcptinfoptrs` to NULL (not to a real exception pointer, since
//!   there IS no OS exception here) around the call; every other exception
//!   signal (SIGILL/SIGSEGV) does the pointer swap but NOT the table reset.
//! * `raise()` for a `SIG_DFL` action calls `_exit(3)` -- WITHOUT running
//!   any registered `atexit` handlers via the normal `exit()` path (a real
//!   divergence from a "normal" process exit, transcribed as-is) -- and
//!   unlocks the signal lock FIRST if the action was global (console/
//!   abort/term signals), since `_exit` never returns to run the `__finally`.
//! * `raise()` returns 0 for `SIG_IGN` WITHOUT ever calling the handler, and
//!   returns 0 after a real handler call too -- raise() only returns -1 for
//!   an invalid signal number/action-not-found (both via the
//!   invalid-parameter path), never for "handler ran".

use core::ffi::{c_int, c_void};

use crate::ptd::AcrtPtd;

// ── constants ────────────────────────────────────────────────────────────

/// UCRT's OWN `signal.h` values (`sdk/include/ucrt/signal.h`), which this
/// file (a UCRT source) uses -- NOT mingw's `crt/signal.h`, where `SIGABRT`
/// is 6. See the module doc's `SIGABRT`/`SIGABRT_COMPAT` entry.
pub const SIGINT: c_int = 2;
pub const SIGILL: c_int = 4;
pub const SIGFPE: c_int = 8;
pub const SIGSEGV: c_int = 11;
pub const SIGTERM: c_int = 15;
pub const SIGBREAK: c_int = 21;
pub const UCRT_SIGABRT: c_int = 22;
pub const UCRT_SIGABRT_COMPAT: c_int = 6;

pub const SIG_DFL: usize = 0;
pub const SIG_IGN: usize = 1;
pub const SIG_GET: usize = 2;
pub const SIG_SGE: usize = 3;
pub const SIG_ACK: usize = 4;
/// Internal-only (`_CORECRT_BUILD`); never a valid argument to `signal()`.
pub const SIG_DIE: usize = 5;
pub const SIG_ERR: usize = usize::MAX;

const _SIGHUP_IGNORE: c_int = 1;
const _SIGQUIT_IGNORE: c_int = 3;
const _SIGPIPE_IGNORE: c_int = 13;
const _SIGIOINT_IGNORE: c_int = 16;
const _SIGSTOP_IGNORE: c_int = 17;

const EINVAL: c_int = 22;

/// `__acrt_signal_lock` (`corecrt_internal.h`'s `__acrt_lock_id` enum: heap=0,
/// debug=1, exit=2, signal=3).
const ACRT_SIGNAL_LOCK: c_int = 3;

const CTRL_C_EVENT: u32 = 0;
// `CTRL_BREAK_EVENT` (1) is the only other value `ctrlevent_capture` ever
// receives (it's registered only for that pair via `SetConsoleCtrlHandler`),
// and the C never checks for it explicitly either -- see `ctrlevent_capture`.

// ── the shared exception-action record (also used by `exception_filter.rs`) ─

/// `__crt_signal_action_t` (`corecrt_internal.h:1398`): plain fixed-width
/// fields, so ordinary `#[repr(C)]` layout matches the C's on all three
/// targets without a measured/pinned offset.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct CrtSignalActionT {
    pub exception_number: u32,
    pub signal_number: c_int,
    /// `__crt_signal_handler_t` as a bit pattern -- see the module doc.
    pub action: usize,
}

extern "C" {
    fn __acrt_getptd_noexit() -> *mut AcrtPtd;
    fn _invalid_parameter_noinfo();
    fn malloc(size: usize) -> *mut c_void;
    fn _exit(code: c_int) -> !;
    fn __acrt_lock(lock: c_int);
    fn __acrt_unlock(lock: c_int);
    static mut __security_cookie: usize;

    static __acrt_exception_action_table: [CrtSignalActionT; 12];
    static __acrt_signal_action_table_count: usize;
    static __acrt_signal_action_table_size: usize;
    static __acrt_signal_action_first_fpe_index: usize;
    static __acrt_signal_action_fpe_count: usize;
}

extern "system" {
    /// `BOOL WINAPI SetConsoleCtrlHandler(PHANDLER_ROUTINE, BOOL)`.
    fn SetConsoleCtrlHandler(handler: unsafe extern "system" fn(u32) -> c_int, add: c_int) -> c_int;
    fn GetLastError() -> u32;
}

#[inline]
unsafe fn encode(ptr: usize) -> usize {
    ptr ^ __security_cookie
}
#[inline]
unsafe fn decode(ptr: usize) -> usize {
    ptr ^ __security_cookie
}

unsafe fn validate(ok: bool, err: c_int) {
    if !ok {
        *crate::misc::errno::_errno() = err;
        _invalid_parameter_noinfo();
    }
}

// ── process-wide console/runtime-only signal actions ────────────────────────

static mut CTRLC_ACTION: usize = 0;
static mut CTRLBREAK_ACTION: usize = 0;
static mut ABORT_ACTION: usize = 0;
static mut TERM_ACTION: usize = 0;
static mut CONSOLE_CTRL_HANDLER_INSTALLED: bool = false;

/// `__acrt_initialize_signal_handlers`. Stores the ALREADY-ENCODED null
/// verbatim in all four globals, matching
/// `dual_state_global::initialize` exactly (no re-encoding here either --
/// see `invalid_parameter.rs`'s module doc for the same pattern).
#[no_mangle]
pub unsafe extern "C" fn __acrt_initialize_signal_handlers(encoded_nullptr: *mut c_void) {
    let v = encoded_nullptr as usize;
    CTRLC_ACTION = v;
    CTRLBREAK_ACTION = v;
    ABORT_ACTION = v;
    TERM_ACTION = v;
}

/// `get_global_action_nolock`.
unsafe fn global_action_nolock(signum: c_int) -> *mut usize {
    match signum {
        SIGINT => core::ptr::addr_of_mut!(CTRLC_ACTION),
        SIGBREAK => core::ptr::addr_of_mut!(CTRLBREAK_ACTION),
        UCRT_SIGABRT => core::ptr::addr_of_mut!(ABORT_ACTION),
        UCRT_SIGABRT_COMPAT => core::ptr::addr_of_mut!(ABORT_ACTION),
        SIGTERM => core::ptr::addr_of_mut!(TERM_ACTION),
        _ => core::ptr::null_mut(),
    }
}

/// `siglookup`.
unsafe fn siglookup(signum: c_int, table: *mut CrtSignalActionT) -> *mut CrtSignalActionT {
    for i in 0..__acrt_signal_action_table_count {
        let p = table.add(i);
        if (*p).signal_number == signum {
            return p;
        }
    }
    core::ptr::null_mut()
}

/// `signal_failed`.
unsafe fn signal_failed(signum: c_int) -> usize {
    match signum {
        _SIGHUP_IGNORE | _SIGQUIT_IGNORE | _SIGPIPE_IGNORE | _SIGIOINT_IGNORE | _SIGSTOP_IGNORE => SIG_ERR,
        _ => {
            *crate::misc::errno::_errno() = EINVAL;
            SIG_ERR
        }
    }
}

/// `is_unsupported_signal` (non-enclave build: only the action matters).
fn is_unsupported_signal(_signum: c_int, sigact: usize) -> bool {
    sigact == SIG_ACK || sigact == SIG_SGE
}

/// `is_console_signal` (non-enclave build).
fn is_console_signal(signum: c_int) -> bool {
    signum == SIGINT || signum == SIGBREAK
}

/// `ctrlevent_capture`.
unsafe extern "system" fn ctrlevent_capture(ctrl_type: u32) -> c_int {
    let ctrl_action: usize;
    let signal_code: c_int;

    __acrt_lock(ACRT_SIGNAL_LOCK);
    // The C never explicitly checks `CTRL_BREAK_EVENT` either -- anything
    // that isn't `CTRL_C_EVENT` falls into the break-event arm.
    let pctrl_action: *mut usize;
    if ctrl_type == CTRL_C_EVENT {
        pctrl_action = core::ptr::addr_of_mut!(CTRLC_ACTION);
        ctrl_action = decode(*pctrl_action);
        signal_code = SIGINT;
    } else {
        pctrl_action = core::ptr::addr_of_mut!(CTRLBREAK_ACTION);
        ctrl_action = decode(*pctrl_action);
        signal_code = SIGBREAK;
    }
    if ctrl_action != SIG_DFL && ctrl_action != SIG_IGN {
        *pctrl_action = encode(0);
    }
    __acrt_unlock(ACRT_SIGNAL_LOCK);

    if ctrl_action == SIG_DFL {
        return 0; // FALSE
    }
    if ctrl_action != SIG_IGN {
        let handler: unsafe extern "C" fn(c_int) = core::mem::transmute(ctrl_action);
        handler(signal_code);
    }
    1 // TRUE
}

/// `signal`. See the module doc for why the public parameter/return is
/// `usize` rather than a function-pointer type.
#[no_mangle]
pub unsafe extern "C" fn signal(signum: c_int, sigact: usize) -> usize {
    if is_unsupported_signal(signum, sigact) {
        return signal_failed(signum);
    }

    if signum == SIGINT || signum == SIGBREAK || signum == UCRT_SIGABRT || signum == UCRT_SIGABRT_COMPAT || signum == SIGTERM {
        let mut set_console_ctrl_error = false;
        let mut old_action: usize = 0;

        __acrt_lock(ACRT_SIGNAL_LOCK);

        if is_console_signal(signum) && !CONSOLE_CTRL_HANDLER_INSTALLED {
            if SetConsoleCtrlHandler(ctrlevent_capture, 1) != 0 {
                CONSOLE_CTRL_HANDLER_INSTALLED = true;
            } else {
                *crate::misc::errno::__doserrno() = GetLastError();
                set_console_ctrl_error = true;
            }
        }

        let action_pointer = global_action_nolock(signum);
        if !action_pointer.is_null() {
            old_action = decode(*action_pointer);
            if sigact != SIG_GET {
                *action_pointer = encode(sigact);
            }
        }

        __acrt_unlock(ACRT_SIGNAL_LOCK);

        if set_console_ctrl_error {
            return signal_failed(signum);
        }
        return old_action;
    }

    if signum != SIGFPE && signum != SIGILL && signum != SIGSEGV {
        return signal_failed(signum);
    }

    let ptd = __acrt_getptd_noexit();
    if ptd.is_null() {
        return signal_failed(signum);
    }

    if (*ptd).pxcptacttab == core::ptr::addr_of!(__acrt_exception_action_table).cast::<c_void>().cast_mut() {
        let table = malloc(__acrt_signal_action_table_size) as *mut CrtSignalActionT;
        if table.is_null() {
            return signal_failed(signum);
        }
        core::ptr::copy_nonoverlapping(
            core::ptr::addr_of!(__acrt_exception_action_table).cast::<CrtSignalActionT>(),
            table,
            __acrt_signal_action_table_count,
        );
        (*ptd).pxcptacttab = table.cast();
    }

    let table: *mut CrtSignalActionT = (*ptd).pxcptacttab.cast();
    let xcpt_action = siglookup(signum, table);
    if xcpt_action.is_null() {
        return signal_failed(signum);
    }

    let old_action = (*xcpt_action).action;

    if sigact != SIG_GET {
        let last = table.add(__acrt_signal_action_table_count);
        let mut p = xcpt_action;
        while p != last && (*p).signal_number == signum {
            (*p).action = sigact;
            p = p.add(1);
        }
    }

    old_action
}

/// `raise`.
#[no_mangle]
pub unsafe extern "C" fn raise(signum: c_int) -> c_int {
    let mut ptd: *mut AcrtPtd = core::ptr::null_mut();
    let mut old_fpecode: c_int = 0;

    let action_pointer: *mut usize;
    let mut action_is_global = true;

    match signum {
        SIGINT | SIGBREAK | UCRT_SIGABRT | UCRT_SIGABRT_COMPAT | SIGTERM => {
            action_pointer = global_action_nolock(signum);
        }
        SIGFPE | SIGILL | SIGSEGV => {
            ptd = __acrt_getptd_noexit();
            if ptd.is_null() {
                return -1;
            }
            let local_action = siglookup(signum, (*ptd).pxcptacttab.cast());
            if local_action.is_null() {
                validate(false, EINVAL);
                return -1;
            }
            action_pointer = core::ptr::addr_of_mut!((*local_action).action);
            action_is_global = false;
        }
        _ => {
            validate(false, EINVAL);
            return -1;
        }
    }

    let mut old_pxcptinfoptrs: *mut c_void = core::ptr::null_mut();

    if action_is_global {
        __acrt_lock(ACRT_SIGNAL_LOCK);
    }

    let action: usize;
    let return0: bool;
    {
        let raw = if action_is_global { decode(*action_pointer) } else { *action_pointer };
        return0 = raw == SIG_IGN;
        action = raw;

        if !return0 {
            if action == SIG_DFL {
                if action_is_global {
                    __acrt_unlock(ACRT_SIGNAL_LOCK);
                }
                _exit(3);
            }

            if signum == SIGFPE || signum == SIGSEGV || signum == SIGILL {
                old_pxcptinfoptrs = (*ptd).tpxcptinfoptrs;
                (*ptd).tpxcptinfoptrs = core::ptr::null_mut();
                if signum == SIGFPE {
                    old_fpecode = (*ptd).tfpecode;
                    (*ptd).tfpecode = 0x8c; // _FPE_EXPLICITGEN
                }
            }

            if signum == SIGFPE {
                let table: *mut CrtSignalActionT = (*ptd).pxcptacttab.cast();
                let first = table.add(__acrt_signal_action_first_fpe_index);
                for i in 0..__acrt_signal_action_fpe_count {
                    (*first.add(i)).action = SIG_DFL;
                }
            } else {
                *action_pointer = encode(0);
            }
        }
    }

    if action_is_global {
        __acrt_unlock(ACRT_SIGNAL_LOCK);
    }

    if return0 {
        return 0;
    }

    if signum == SIGFPE {
        let handler: unsafe extern "C" fn(c_int, c_int) = core::mem::transmute(action);
        handler(SIGFPE, (*ptd).tfpecode);
    } else {
        let handler: unsafe extern "C" fn(c_int) = core::mem::transmute(action);
        handler(signum);
    }

    if signum == SIGFPE || signum == SIGSEGV || signum == SIGILL {
        (*ptd).tpxcptinfoptrs = old_pxcptinfoptrs;
        if signum == SIGFPE {
            (*ptd).tfpecode = old_fpecode;
        }
    }

    0
}

/// `__acrt_get_sigabrt_handler`.
#[no_mangle]
pub unsafe extern "C" fn __acrt_get_sigabrt_handler() -> usize {
    __acrt_lock(ACRT_SIGNAL_LOCK);
    let v = decode(ABORT_ACTION);
    __acrt_unlock(ACRT_SIGNAL_LOCK);
    v
}

/// `__fpecode`.
#[no_mangle]
pub unsafe extern "C" fn __fpecode() -> *mut c_int {
    let ptd = crate::locale_update::__acrt_getptd();
    core::ptr::addr_of_mut!((*ptd).tfpecode)
}

/// `__pxcptinfoptrs`.
#[no_mangle]
pub unsafe extern "C" fn __pxcptinfoptrs() -> *mut *mut c_void {
    let ptd = crate::locale_update::__acrt_getptd();
    core::ptr::addr_of_mut!((*ptd).tpxcptinfoptrs)
}
