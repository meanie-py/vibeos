//! main.c — msvcrt.dll's `DllMain`, the last C function in the DLL's exported surface.
//!
//! Everything this orchestrates was already Rust before this file existed: the heap
//! (`heap.rs`), the locks (`lock.rs`), the locale (`locale.rs`), the fd table (`file.rs`),
//! the signal table and C++ exception state (`except.rs`, `cpp.rs`), argv (`data.rs`), the
//! clock (`time.rs`). What was left in C was the *order* they happen in, and the TLS slot
//! index every one of them reaches through.
//!
//! # `msvcrt_tls_index` moved with `DllMain`, and it had to
//!
//! The slot is written exactly once, by `msvcrt_init_tls` at `DLL_PROCESS_ATTACH`, and read
//! by `thread.rs` on every `msvcrt_get_thread_data`. Until now the definition lived in the C
//! and `prelude.rs` imported it — the correct arrangement, and its comment said why: *whoever
//! owns DllMain owns the slot's lifetime, and duplicating the definition on the Rust side
//! would give the reader a second, always-zero index.* That hazard is real and it is the
//! reason the definition moves in the SAME commit as the writer, not before and not after.
//! Cohab resolves both the writer and the reader to this one storage location because
//! `libmsvcrt_crt_cargo.a` precedes `msvcrt_static` in the link; a build that resolved them
//! differently would leave every TLS lookup reading index 0.
//!
//! # What `_MSVCR_VER = 0` deletes
//!
//! Five blocks of the C never compile in this DLL and so are absent here on purpose:
//! `msvcrt_init_handler4` / `msvcrt_attach_handler4` / `msvcrt_free_handler4`
//! (`defined(__x86_64__) && _MSVCR_VER>=140` — and handler4.c itself compiles to an object
//! with no symbols), `msvcrt_init_concurrency` / `msvcrt_free_concurrency` /
//! `msvcrt_free_scheduler_thread` (`_MSVCR_VER >= 100 && <= 120`, likewise concurrency.c),
//! and `_set_printf_count_output(0)` (`_MSVCR_VER >= 80`). The `#if _MSVCR_VER == 0` arm —
//! the `LdrAddRefDll` self-pin — is the one that IS live, and it is the branch a translation
//! working from the wine original rather than this tree's flags would drop.

#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

use crate::prelude::*;
use core::ffi::c_void;

// Every subsystem DllMain drives, imported from the module that owns it rather than reached
// through the prelude — these are OUR definitions, not the C's, and naming the owning module
// is what makes the init order auditable against main.c side by side.
use super::console::msvcrt_free_console;
use super::cpp::msvcrt_init_exception;
use super::data::{msvcrt_free_args, msvcrt_init_args};
use super::except::{msvcrt_free_signals, msvcrt_init_signals};
use super::file::{msvcrt_free_io, msvcrt_init_io};
use super::heap::{msvcrt_destroy_heap, msvcrt_init_heap};
use super::locale::{_free_locale, free_locinfo, msvcrt_init_locale};
use super::lock::{msvcrt_free_locks, msvcrt_init_mt_locks};
use super::math::msvcrt_init_math;
use super::process::msvcrt_free_popen_data;
use super::time::msvcrt_init_clock;

// wine's always-on ERR channel. As in the except modules, the whole message including the
// `err:msvcrt:` prefix is a `c"…"` literal at the call site so the literal-multiset diff
// against main.c sees a channel call on both sides and a macro body with nothing in it.
macro_rules! ERR {
    ($msg:literal $(, $($arg:tt)*)?) => {{
        OutputDebugStringA($msg.as_ptr());
    }};
}
macro_rules! TRACE {
    ($($arg:tt)*) => {{}};
}

/// `psdk/winbase.h:529` — `#define TLS_OUT_OF_INDEXES (DWORD)0xFFFFFFFF`. A macro, so there
/// is no symbol for the island to snapshot.
const TLS_OUT_OF_INDEXES: DWORD = 0xFFFF_FFFF;

/// `wine/winternl.h:4025` — `#define LDR_ADDREF_DLL_PIN 0x00000001`. Same reason.
const LDR_ADDREF_DLL_PIN: ULONG = 0x0000_0001;

/// `msvcrt.h:170` — `DWORD msvcrt_tls_index;`. See the module header: this definition and
/// `msvcrt_init_tls`'s write to it must live on the same side of the cohab boundary as the
/// readers in `thread.rs`, which they now do.
#[no_mangle]
pub static mut msvcrt_tls_index: DWORD = 0;

/// `static inline BOOL msvcrt_init_tls(void)` — inline in the C, so private here.
unsafe fn msvcrt_init_tls() -> BOOL {
    msvcrt_tls_index = TlsAlloc();

    if msvcrt_tls_index == TLS_OUT_OF_INDEXES {
        ERR!(c"err:msvcrt:TlsAlloc() failed!\n");
        return FALSE as BOOL;
    }
    TRUE as BOOL
}

/// `static inline BOOL msvcrt_free_tls(void)`.
unsafe fn msvcrt_free_tls() -> BOOL {
    if TlsFree(msvcrt_tls_index) == 0 {
        ERR!(c"err:msvcrt:TlsFree() failed!\n");
        return FALSE as BOOL;
    }
    TRUE as BOOL
}

/// `static inline void msvcrt_free_tls_mem(void)`.
///
/// Note the `HeapFree` sits OUTSIDE the `if (tls)` in the C, so a NULL `tls` still reaches it.
/// `HeapFree(heap, 0, NULL)` is a documented no-op, so this is harmless and it is kept where
/// the C puts it rather than tidied into the branch — the shapes are not equivalent if
/// `HeapFree` is ever replaced.
unsafe fn msvcrt_free_tls_mem() {
    let tls: *mut thread_data_t = TlsGetValue(msvcrt_tls_index) as *mut thread_data_t;

    if !tls.is_null() {
        free((*tls).efcvt_buffer as *mut c_void);
        free((*tls).asctime_buffer as *mut c_void);
        free((*tls).wasctime_buffer as *mut c_void);
        free((*tls).strerror_buffer as *mut c_void);
        free((*tls).wcserror_buffer as *mut c_void);
        free((*tls).time_buffer as *mut c_void);
        free((*tls).tmpnam_buffer as *mut c_void);
        free((*tls).wtmpnam_buffer as *mut c_void);
        if (*tls).locale_flags & (LOCALE_FREE as c_int) != 0 {
            free_locinfo((*tls).locinfo);
            free_mbcinfo((*tls).mbcinfo);
        }
    }
    HeapFree(GetProcessHeap(), 0, tls as LPVOID);
}

/// `DllMain` — msvcrt.dll's entry point (`set_module_type(msvcrt win32dll ENTRYPOINT
/// DllMain 12)`).
///
/// The teardown order is the reverse of setup and every step of it matters; the one that
/// reads like an oddity is `if (lpvReserved) break;` — a non-NULL `lpvReserved` at
/// DLL_PROCESS_DETACH means the whole process is exiting, so everything after that point is
/// skipped and only `msvcrt_free_io` has run. That is deliberate: at process exit the kernel
/// reclaims it all, and walking the locale/lock/TLS teardown while other threads may have
/// been killed mid-lock is how you hang instead of exit.
#[no_mangle]
pub unsafe extern "system" fn DllMain(
    hinstDLL: HINSTANCE,
    fdwReason: DWORD,
    lpvReserved: LPVOID,
) -> BOOL {
    match fdwReason {
        DLL_PROCESS_ATTACH => {
            // Both take `void*` in msvcrt.h:270/:272, so the C is doing an implicit
            // HINSTANCE -> void* conversion at these two call sites.
            msvcrt_init_exception(hinstDLL as *mut c_void);
            if msvcrt_init_heap() == 0 {
                return FALSE as BOOL;
            }
            if msvcrt_init_tls() == 0 {
                msvcrt_destroy_heap();
                return FALSE as BOOL;
            }
            msvcrt_init_mt_locks();
            if msvcrt_init_locale() == 0 {
                msvcrt_free_locks();
                msvcrt_free_tls_mem();
                msvcrt_destroy_heap();
                return FALSE as BOOL;
            }
            msvcrt_init_math(hinstDLL as *mut c_void);
            msvcrt_init_io();
            msvcrt_init_args();
            msvcrt_init_signals();
            // `#if _MSVCR_VER == 0` — don't allow unloading msvcrt, we can't setup file
            // handles twice.
            LdrAddRefDll(LDR_ADDREF_DLL_PIN, hinstDLL as HMODULE);
            msvcrt_init_clock();
            TRACE!("finished process init\n");
        }
        DLL_THREAD_ATTACH => {}
        DLL_PROCESS_DETACH => {
            msvcrt_free_io();
            if lpvReserved.is_null() {
                msvcrt_free_popen_data();
                msvcrt_free_locks();
                msvcrt_free_console();
                msvcrt_free_args();
                msvcrt_free_signals();
                msvcrt_free_tls_mem();
                if msvcrt_free_tls() == 0 {
                    return FALSE as BOOL;
                }
                _free_locale(MSVCRT_locale);
                msvcrt_destroy_heap();
                TRACE!("finished process free\n");
            }
        }
        DLL_THREAD_DETACH => {
            msvcrt_free_tls_mem();
            TRACE!("finished thread free\n");
        }
        _ => {}
    }
    TRUE as BOOL
}
