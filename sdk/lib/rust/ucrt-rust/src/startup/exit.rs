//! `startup/exit.cpp` -- `exit`, `_exit`, `_Exit`, `quick_exit`, `_cexit`,
//! `_c_exit`, `_is_c_termination_complete`,
//! `_register_thread_local_exe_atexit_callback`, and
//! `__acrt_initialize_thread_local_exit_callback` (called by
//! `crate::internal::init`'s `initialize_pointers` with the encoded null).
//!
//! Details that are silent if wrong:
//! * `common_exit`'s three cleanup modes: FULL runs the main thread's
//!   thread_local destructor callback (with `DLL_PROCESS_DETACH` = 0 and two
//!   NULLs) and then `__acrt_atexit_table`; QUICK runs only
//!   `__acrt_at_quick_exit_table`; NONE runs nothing.
//! * All of that happens under the exit lock, guarded by a function-local
//!   `c_exit_complete` flag that is set ONLY on the terminate-process path.
//!   So `_cexit` then `exit` runs the (now emptied) tables twice, while a
//!   second `exit` -- from an atexit callback on this thread (the lock is
//!   recursive), or from another thread after the first finished -- skips the
//!   termination and goes straight to process exit. `c_termination_complete`
//!   is set (interlocked) on every entry that gets past the flag, including
//!   `_c_exit`'s.
//! * Process exit, in `exit_or_terminate_process`: `TerminateProcess` unless
//!   the process-end policy says `ExitProcess` or Application Verifier is on;
//!   then (should that return) `CorExitProcess` from an already-loaded
//!   mscoree.dll; then `ExitProcess`. Before any of that, a MANAGED exe (a PE
//!   with a COM descriptor directory) first tries `CorExitProcess` so the CLR
//!   can re-enter with return-to-caller.
//! * The thread-local exit callback is stored encoded; registering it twice
//!   is `terminate()`.
//! * The C's `__try`/`__except(atexit_exception_filter)` -> `terminate()`
//!   around the table execution is dead under this build's dummy PSEH (see
//!   `super`'s module doc); the ported body is the protected region alone.

use core::ffi::{c_int, c_long, c_void};
use core::sync::atomic::{AtomicI32, Ordering};

use super::onexit::{_execute_onexit_table, __acrt_at_quick_exit_table, __acrt_atexit_table};
use super::{fast_code, with_exit_lock};

/// `_tls_callback_type` (`process.h`): `void (__stdcall*)(void*, unsigned long, void*)`.
pub type TlsCallbackType = Option<unsafe extern "system" fn(*mut c_void, u32, *mut c_void)>;

/// `_crt_exit_cleanup_mode` (`vcruntime_startup.h`).
#[derive(Clone, Copy, PartialEq, Eq)]
enum CleanupMode {
    Full,
    Quick,
    NoCleanup,
}

/// `_crt_exit_return_mode`.
#[derive(Clone, Copy, PartialEq, Eq)]
enum ReturnMode {
    TerminateProcess,
    ReturnToCaller,
}

/// `process_end_policy_exit_process` (`corecrt_internal.h`: after
/// `..._terminate_process` = 0).
const PROCESS_END_POLICY_EXIT_PROCESS: c_int = 1;
/// `DLL_PROCESS_DETACH`.
const DLL_PROCESS_DETACH: u32 = 0;

const IMAGE_DOS_SIGNATURE: u16 = 0x5A4D;
const IMAGE_NT_SIGNATURE: u32 = 0x0000_4550;
const IMAGE_DIRECTORY_ENTRY_COM_DESCRIPTOR: u32 = 14;
/// `IMAGE_NT_OPTIONAL_HDR_MAGIC` for this build's pointer width.
#[cfg(target_pointer_width = "64")]
const IMAGE_NT_OPTIONAL_HDR_MAGIC: u16 = 0x20B;
#[cfg(target_pointer_width = "32")]
const IMAGE_NT_OPTIONAL_HDR_MAGIC: u16 = 0x10B;
/// `IMAGE_DOS_HEADER::e_lfanew`, a LONG.
const E_LFANEW_AT: usize = 0x3C;
/// `IMAGE_NT_HEADERS` field offsets from the NT header (Signature 0,
/// OptionalHeader 0x18; then `NumberOfRvaAndSizes` / `DataDirectory` inside
/// the optional header). Read off the compiled C on all three arches:
/// i386 0x74/0xE8, amd64 and arm64 0x84/0xF8 for [COM_DESCRIPTOR].
const NT_OPTIONAL_MAGIC_AT: usize = 0x18;
#[cfg(target_pointer_width = "64")]
const NT_NUMBER_OF_RVA_AND_SIZES_AT: usize = 0x18 + 0x6C;
#[cfg(target_pointer_width = "32")]
const NT_NUMBER_OF_RVA_AND_SIZES_AT: usize = 0x18 + 0x5C;
#[cfg(target_pointer_width = "64")]
const NT_DATA_DIRECTORY_AT: usize = 0x18 + 0x70;
#[cfg(target_pointer_width = "32")]
const NT_DATA_DIRECTORY_AT: usize = 0x18 + 0x60;

static C_TERMINATION_COMPLETE: AtomicI32 = AtomicI32::new(0);

/// `thread_local_exit_callback_func`: ENCODED. Zero until
/// `__acrt_initialize_thread_local_exit_callback` seeds it with the encoded
/// null at CRT initialization.
static mut THREAD_LOCAL_EXIT_CALLBACK_FUNC: usize = 0;

/// `common_exit`'s lambda-local `static bool c_exit_complete`.
static mut C_EXIT_COMPLETE: bool = false;

extern "C" {
    fn __acrt_get_process_end_policy() -> c_int;
    fn __acrt_app_verifier_enabled() -> bool;
    /// `__declspec(noreturn) void __cdecl terminate(void)` (`corecrt_terminate.h`).
    fn terminate() -> !;
}

extern "system" {
    fn GetModuleHandleW(module_name: *const u16) -> *mut c_void;
    fn GetModuleHandleExW(flags: u32, module_name: *const u16, module: *mut *mut c_void) -> c_int;
    fn GetProcAddress(module: *mut c_void, name: *const core::ffi::c_char) -> *mut c_void;
    fn FreeLibrary(module: *mut c_void) -> c_int;
    fn GetCurrentProcess() -> *mut c_void;
    fn TerminateProcess(process: *mut c_void, exit_code: u32) -> i32;
    fn ExitProcess(exit_code: u32) -> !;
}

/// `exit_process_pft`: `void (WINAPI*)(UINT)`.
type ExitProcessPft = unsafe extern "system" fn(u32);

/// `is_managed_app`: does the EXE carry a COM descriptor (CLR header)?
unsafe fn is_managed_app() -> bool {
    let dos_header = GetModuleHandleW(core::ptr::null()) as *const u8;
    if dos_header.is_null() {
        return false;
    }

    if (dos_header as *const u16).read_unaligned() != IMAGE_DOS_SIGNATURE {
        return false;
    }

    let e_lfanew = (dos_header.add(E_LFANEW_AT) as *const c_long).read_unaligned();
    let pe_header = dos_header.offset(e_lfanew as isize);

    if (pe_header as *const u32).read_unaligned() != IMAGE_NT_SIGNATURE {
        return false;
    }

    if (pe_header.add(NT_OPTIONAL_MAGIC_AT) as *const u16).read_unaligned() != IMAGE_NT_OPTIONAL_HDR_MAGIC {
        return false;
    }

    if (pe_header.add(NT_NUMBER_OF_RVA_AND_SIZES_AT) as *const u32).read_unaligned()
        <= IMAGE_DIRECTORY_ENTRY_COM_DESCRIPTOR
    {
        return false;
    }

    // DataDirectory[COM_DESCRIPTOR].VirtualAddress (each entry is 8 bytes).
    let com = pe_header.add(NT_DATA_DIRECTORY_AT + 8 * IMAGE_DIRECTORY_ENTRY_COM_DESCRIPTOR as usize);
    if (com as *const u32).read_unaligned() == 0 {
        return false;
    }

    true
}

/// `try_cor_exit_process`: `CorExitProcess` from an ALREADY-loaded
/// mscoree.dll (`GetModuleHandleExW(0, ...)` takes a reference, which the
/// `__crt_unique_hmodule` releases -- only reached if the call returns).
unsafe fn try_cor_exit_process(return_code: u32) {
    let mut mscoree: *mut c_void = core::ptr::null_mut();
    if GetModuleHandleExW(0, super::w!("mscoree.dll").as_ptr(), &mut mscoree) != 0 {
        let cor_exit_process = GetProcAddress(mscoree, c"CorExitProcess".as_ptr());
        if !cor_exit_process.is_null() {
            let cor_exit_process = core::mem::transmute::<*mut c_void, ExitProcessPft>(cor_exit_process);
            cor_exit_process(return_code);
        }
    }

    // ~__crt_unique_hmodule
    if !mscoree.is_null() {
        FreeLibrary(mscoree);
    }
}

/// `should_call_terminate_process`.
unsafe fn should_call_terminate_process() -> bool {
    if __acrt_get_process_end_policy() == PROCESS_END_POLICY_EXIT_PROCESS {
        return false;
    }

    // Application Verifier wants DLLs unloaded cleanly: ExitProcess.
    if __acrt_app_verifier_enabled() {
        return false;
    }

    true
}

/// `exit_or_terminate_process`. Never returns.
unsafe fn exit_or_terminate_process(return_code: u32) -> ! {
    if should_call_terminate_process() {
        TerminateProcess(GetCurrentProcess(), return_code);
    }

    try_cor_exit_process(return_code);

    // Not managed, or CorExitProcess failed: exit the normal way.
    ExitProcess(return_code)
}

/// `__acrt_initialize_thread_local_exit_callback`: seed with the encoded null.
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn __acrt_initialize_thread_local_exit_callback(encoded_null: *mut c_void) {
    THREAD_LOCAL_EXIT_CALLBACK_FUNC = encoded_null as usize;
}

/// `_register_thread_local_exe_atexit_callback`: set once, by the EXE's
/// startup; a second registration is `terminate()`.
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _register_thread_local_exe_atexit_callback(callback: TlsCallbackType) {
    if THREAD_LOCAL_EXIT_CALLBACK_FUNC != fast_code(0) {
        terminate();
    }

    THREAD_LOCAL_EXIT_CALLBACK_FUNC = fast_code(callback.map_or(0, |f| f as usize));
}

/// The C termination half of `common_exit`: everything under the exit lock.
/// `return_mode` is a constant at every call site (clang clones it the same
/// way), which is what lets the process-exit tail be typed `!` below.
unsafe fn run_c_termination(cleanup_mode: CleanupMode, return_mode: ReturnMode) {
    with_exit_lock(|| {
        if C_EXIT_COMPLETE {
            return;
        }

        C_TERMINATION_COMPLETE.swap(1, Ordering::SeqCst);

        if cleanup_mode == CleanupMode::Full {
            // thread_local destructors for the primary thread come before any
            // atexit call (3.6.3/1).
            let encoded = THREAD_LOCAL_EXIT_CALLBACK_FUNC;
            if encoded != fast_code(0) {
                let callback = core::mem::transmute::<usize, unsafe extern "system" fn(*mut c_void, u32, *mut c_void)>(
                    fast_code(encoded),
                );
                callback(core::ptr::null_mut(), DLL_PROCESS_DETACH, core::ptr::null_mut());
            }

            _execute_onexit_table(core::ptr::addr_of_mut!(__acrt_atexit_table));
        } else if cleanup_mode == CleanupMode::Quick {
            _execute_onexit_table(core::ptr::addr_of_mut!(__acrt_at_quick_exit_table));
        }

        // (CRTDLL: no _initterm(__xp_a/__xt_a) here -- see `super`.)

        if return_mode == ReturnMode::TerminateProcess {
            C_EXIT_COMPLETE = true;
            // crt_uninitialization_required = true: only acted on by the
            // static debug CRT, which this is not.
        }
    });
}

/// `common_exit(return_code, cleanup_mode, _crt_exit_terminate_process)`.
unsafe fn common_exit_terminate(return_code: c_int, cleanup_mode: CleanupMode) -> ! {
    // In a managed app, let the CLR run the termination: a successful
    // CorExitProcess comes back through here with return-to-caller.
    if is_managed_app() {
        try_cor_exit_process(return_code as u32);
    }

    run_c_termination(cleanup_mode, ReturnMode::TerminateProcess);

    exit_or_terminate_process(return_code as u32)
}

/// `_is_c_termination_complete`.
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _is_c_termination_complete() -> c_int {
    C_TERMINATION_COMPLETE.load(Ordering::SeqCst)
}

/// `exit`: full cleanup, then end the process.
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn exit(return_code: c_int) -> ! {
    common_exit_terminate(return_code, CleanupMode::Full)
}

/// `_exit`: no cleanup.
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _exit(return_code: c_int) -> ! {
    common_exit_terminate(return_code, CleanupMode::NoCleanup)
}

/// `_Exit`: identical to `_exit`.
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _Exit(return_code: c_int) -> ! {
    common_exit_terminate(return_code, CleanupMode::NoCleanup)
}

/// `quick_exit`: the at_quick_exit table only.
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn quick_exit(return_code: c_int) -> ! {
    common_exit_terminate(return_code, CleanupMode::Quick)
}

/// `_cexit`: full cleanup, return to the caller.
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _cexit() {
    run_c_termination(CleanupMode::Full, ReturnMode::ReturnToCaller);
}

/// `_c_exit`: no cleanup, return to the caller.
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _c_exit() {
    run_c_termination(CleanupMode::NoCleanup, ReturnMode::ReturnToCaller);
}
