//! `internal/initialization.cpp`, `dll/appcrt_dllmain.cpp` and the
//! `initializers/*.cpp` section entries, ported.
//!
//! Details that are silent if wrong:
//! * The initializer table's ORDER is the dependency order (locks before heap
//!   before ptd before lowio ...) and failure unwinds it in reverse; see
//!   `__acrt_execute_initializers`.
//! * `__acrt_uninitialize(terminating = true)` -- process exit rather than
//!   `FreeLibrary` -- only flushes stdio. Tearing the CRT down while other
//!   DLLs' detach handlers may still call into it is what the early return
//!   avoids.
//! * The `.CRT$XIC` / `.CRT$XPX` / `.CRT$XPXA` entries are data nothing calls by
//!   name: `initialize_c` walks `__xi_a..__xi_z` and `uninitialize_c` walks
//!   `__xp_a..__xp_z`, and the linker's `$`-suffix sort puts each entry between
//!   the brackets. Each keeps its C name because Rust modules `force_include!`
//!   it (that was how the deleted C files' `/include:` directives were carried).
//!   `.CRT$XPXA` sorts after `.CRT$XPX`, so stdio is torn down last.

use super::{AcrtInitializer, Pifv, Pvfv, _initterm, _initterm_e};
use core::ffi::{c_int, c_void};
use core::ptr::{addr_of_mut, null_mut};
use core::sync::atomic::{AtomicI32, Ordering};

use crate::locale::CrtMultibyteData;
use crate::locale_abi::{__acrt_current_locale_data, __acrt_initial_locale_data, __acrt_initial_multibyte_data};

extern "C" {
    static __xi_a: [Pifv; 0];
    static __xi_z: [Pifv; 0];
    static __xc_a: [Pvfv; 0];
    static __xc_z: [Pvfv; 0];
    static __xp_a: [Pvfv; 0];
    static __xp_z: [Pvfv; 0];
    static __xt_a: [Pvfv; 0];
    static __xt_z: [Pvfv; 0];

    static mut __acrt_atexit_table: OnexitTable;
    static mut __acrt_at_quick_exit_table: OnexitTable;
    static mut __acrt_stdout_buffer: *mut c_void;
    static mut __acrt_stderr_buffer: *mut c_void;
    static mut __argv: *mut *mut u8;
    static mut __wargv: *mut *mut u16;
    static mut __acrt_current_multibyte_data: *mut CrtMultibyteData;

    fn _initialize_onexit_table(table: *mut OnexitTable) -> c_int;
    fn _initialize_narrow_environment() -> c_int;
    fn _get_initial_narrow_environment() -> *mut *mut u8;
    fn __dcrt_uninitialize_environments_nolock();
    fn _free_base(block: *mut c_void);
    fn _flushall() -> c_int;

    fn __acrt_initialize_invalid_parameter_handler(encoded_null: *mut c_void);
    fn __acrt_initialize_new_handler(encoded_null: *mut c_void);
    fn __acrt_initialize_signal_handlers(encoded_null: *mut c_void);
    fn __acrt_initialize_user_matherr(encoded_null: *mut c_void);
    fn __acrt_initialize_thread_local_exit_callback(encoded_null: *mut c_void);

    fn __acrt_initialize_winapi_thunks() -> bool;
    fn __acrt_uninitialize_winapi_thunks(terminating: bool) -> bool;
    fn __acrt_initialize_heap() -> bool;
    fn __acrt_uninitialize_heap(terminating: bool) -> bool;
    fn __acrt_initialize_lowio() -> bool;
    fn __acrt_uninitialize_lowio(terminating: bool) -> bool;
    fn __acrt_initialize_command_line() -> bool;
    fn __acrt_uninitialize_command_line(terminating: bool) -> bool;
    fn __acrt_initialize_multibyte() -> bool;

    fn __vcrt_initialize() -> bool;
    fn __vcrt_uninitialize(terminating: bool) -> bool;
    fn __isa_available_init() -> c_int;
    fn __security_init_cookie();
    static mut __security_cookie: usize;
}

/// `_onexit_table_t`: three `_PVFV*` (first, last, end).
#[repr(C)]
pub struct OnexitTable {
    first: *mut Pvfv,
    last: *mut Pvfv,
    end: *mut Pvfv,
}

/// `__crt_fast_encode_pointer(nullptr)`: the GS cookie XOR, as in
/// `invalid_parameter.rs`.
unsafe fn encoded_null() -> *mut c_void {
    __security_cookie as *mut c_void
}

unsafe extern "C" fn initialize_global_variables() -> bool {
    __acrt_current_locale_data = addr_of_mut!(__acrt_initial_locale_data);
    true
}

unsafe extern "C" fn initialize_c() -> bool {
    _initialize_onexit_table(addr_of_mut!(__acrt_atexit_table));
    _initialize_onexit_table(addr_of_mut!(__acrt_at_quick_exit_table));
    // Only the C initializers can fail; the C++ ones cannot.
    if _initterm_e(__xi_a.as_ptr(), __xi_z.as_ptr()) != 0 {
        return false;
    }
    _initterm(__xc_a.as_ptr(), __xc_z.as_ptr());
    true
}

unsafe extern "C" fn uninitialize_c(_terminating: bool) -> bool {
    // Pre-terminators, then terminators.
    _initterm(__xp_a.as_ptr(), __xp_z.as_ptr());
    _initterm(__xt_a.as_ptr(), __xt_z.as_ptr());
    true
}

unsafe extern "C" fn initialize_environment() -> bool {
    if _initialize_narrow_environment() < 0 {
        return false;
    }
    !_get_initial_narrow_environment().is_null()
}

unsafe extern "C" fn uninitialize_environment(_terminating: bool) -> bool {
    __dcrt_uninitialize_environments_nolock();
    true
}

unsafe extern "C" fn initialize_pointers() -> bool {
    let encoded_null = encoded_null();
    __acrt_initialize_invalid_parameter_handler(encoded_null);
    __acrt_initialize_new_handler(encoded_null);
    __acrt_initialize_signal_handlers(encoded_null);
    __acrt_initialize_user_matherr(encoded_null);
    __acrt_initialize_thread_local_exit_callback(encoded_null);
    true
}

unsafe extern "C" fn uninitialize_vcruntime(_terminating: bool) -> bool {
    __vcrt_uninitialize(false)
}

unsafe extern "C" fn uninitialize_allocated_memory(_terminating: bool) -> bool {
    let data = __acrt_current_multibyte_data;
    let refcount = &*(addr_of_mut!((*data).refcount) as *const AtomicI32);
    if refcount.fetch_sub(1, Ordering::SeqCst) - 1 == 0 && data != addr_of_mut!(__acrt_initial_multibyte_data) {
        _free_base(data.cast());
        __acrt_current_multibyte_data = addr_of_mut!(__acrt_initial_multibyte_data);
    }
    true
}

unsafe extern "C" fn uninitialize_allocated_io_buffers(_terminating: bool) -> bool {
    _free_base(__acrt_stdout_buffer);
    __acrt_stdout_buffer = null_mut();
    _free_base(__acrt_stderr_buffer);
    __acrt_stderr_buffer = null_mut();
    _free_base(__argv.cast());
    __argv = null_mut();
    _free_base(__wargv.cast());
    __wargv = null_mut();
    true
}

/// The debug CRT's leak report; nothing in a release build.
unsafe extern "C" fn report_memory_leaks(_terminating: bool) -> bool {
    true
}

/// Without global state isolation both of these are no-ops that succeed.
unsafe extern "C" fn initialize_global_state_isolation() -> bool {
    true
}
unsafe extern "C" fn uninitialize_global_state_isolation(_terminating: bool) -> bool {
    true
}

static ACRT_INITIALIZERS: [AcrtInitializer; 16] = [
    AcrtInitializer { initialize: Some(initialize_global_variables), uninitialize: None },
    AcrtInitializer { initialize: Some(initialize_pointers), uninitialize: None },
    AcrtInitializer {
        initialize: Some(__acrt_initialize_winapi_thunks),
        uninitialize: Some(__acrt_uninitialize_winapi_thunks),
    },
    AcrtInitializer {
        initialize: Some(initialize_global_state_isolation),
        uninitialize: Some(uninitialize_global_state_isolation),
    },
    AcrtInitializer {
        initialize: Some(super::__acrt_initialize_locks),
        uninitialize: Some(super::__acrt_uninitialize_locks),
    },
    AcrtInitializer { initialize: Some(__acrt_initialize_heap), uninitialize: Some(__acrt_uninitialize_heap) },
    AcrtInitializer { initialize: None, uninitialize: Some(uninitialize_vcruntime) },
    AcrtInitializer {
        initialize: Some(super::__acrt_initialize_ptd),
        uninitialize: Some(super::__acrt_uninitialize_ptd),
    },
    AcrtInitializer { initialize: Some(__acrt_initialize_lowio), uninitialize: Some(__acrt_uninitialize_lowio) },
    AcrtInitializer {
        initialize: Some(__acrt_initialize_command_line),
        uninitialize: Some(__acrt_uninitialize_command_line),
    },
    AcrtInitializer { initialize: Some(__acrt_initialize_multibyte), uninitialize: None },
    AcrtInitializer { initialize: None, uninitialize: Some(report_memory_leaks) },
    AcrtInitializer { initialize: None, uninitialize: Some(uninitialize_allocated_io_buffers) },
    AcrtInitializer { initialize: None, uninitialize: Some(uninitialize_allocated_memory) },
    AcrtInitializer { initialize: Some(initialize_environment), uninitialize: Some(uninitialize_environment) },
    AcrtInitializer { initialize: Some(initialize_c), uninitialize: Some(uninitialize_c) },
];

fn table_bounds() -> (*const AcrtInitializer, *const AcrtInitializer) {
    let r = ACRT_INITIALIZERS.as_ptr_range();
    (r.start, r.end)
}

#[no_mangle]
pub unsafe extern "C" fn __acrt_initialize() -> bool {
    __isa_available_init();
    let (first, last) = table_bounds();
    super::__acrt_execute_initializers(first, last)
}

#[no_mangle]
pub unsafe extern "C" fn __acrt_uninitialize(terminating: bool) -> bool {
    if terminating {
        if !crate::stdio::__piob.is_null() {
            _flushall();
        }
        return true;
    }
    let (first, last) = table_bounds();
    super::__acrt_execute_uninitializers(first, last)
}

#[no_mangle]
pub unsafe extern "C" fn __acrt_uninitialize_critical(terminating: bool) -> bool {
    super::__acrt_uninitialize_ptd(terminating);
    true
}

#[no_mangle]
pub unsafe extern "C" fn __acrt_thread_attach() -> bool {
    !super::__acrt_getptd_noexit().is_null()
}

#[no_mangle]
pub unsafe extern "C" fn __acrt_thread_detach() -> bool {
    super::__acrt_freeptd();
    true
}

// ── appcrt_dllmain.cpp ──────────────────────────────────────────────────────

const DLL_PROCESS_DETACH: u32 = 0;
const DLL_PROCESS_ATTACH: u32 = 1;
const DLL_THREAD_ATTACH: u32 = 2;
const DLL_THREAD_DETACH: u32 = 3;

static mut ACRT_PROCESS_ATTACHED: c_int = 0;

unsafe fn dll_main_process_attach() -> c_int {
    if !__vcrt_initialize() {
        return 0;
    }
    if !__acrt_initialize() {
        __vcrt_uninitialize(false);
        return 0;
    }
    ACRT_PROCESS_ATTACHED += 1;
    1
}

unsafe fn dll_main_process_detach(terminating: bool) -> c_int {
    // Attach failed or never happened: nothing to undo.
    if ACRT_PROCESS_ATTACHED <= 0 {
        return 0;
    }
    ACRT_PROCESS_ATTACHED -= 1;
    if !__acrt_uninitialize(terminating) {
        return 0;
    }
    1
}

/// A breakpoint target the C keeps out of line: "the CRT has finished booting".
#[no_mangle]
#[inline(never)]
pub extern "C" fn __acrt_end_boot() {
    core::hint::black_box(());
}

#[inline(never)]
unsafe fn dll_main_dispatch(reason: u32, reserved: *mut c_void) -> c_int {
    let result = match reason {
        DLL_PROCESS_ATTACH => dll_main_process_attach(),
        DLL_PROCESS_DETACH => dll_main_process_detach(!reserved.is_null()),
        DLL_THREAD_ATTACH => __acrt_thread_attach() as c_int,
        DLL_THREAD_DETACH => __acrt_thread_detach() as c_int,
        _ => 0,
    };
    __acrt_end_boot();
    result
}

#[no_mangle]
pub unsafe extern "system" fn __acrt_DllMain(_instance: *mut c_void, reason: u32, reserved: *mut c_void) -> c_int {
    if reason == DLL_PROCESS_ATTACH {
        __security_init_cookie();
    }
    dll_main_dispatch(reason, reserved)
}

// ── initializers/*.cpp ──────────────────────────────────────────────────────

macro_rules! crt_entry {
    ($section:literal, $name:ident: $ty:ty = $target:ident, fn() $(-> $ret:ty)?) => {
        #[no_mangle]
        #[used]
        #[link_section = $section]
        pub static $name: $ty = {
            extern "C" {
                fn $target() $(-> $ret)?;
            }
            Some($target)
        };
    };
}

crt_entry!(".CRT$XIC", __acrt_clock_initializer: Pifv = __acrt_initialize_clock, fn() -> c_int);
crt_entry!(".CRT$XPX", __dcrt_console_input_terminator: Pvfv = __dcrt_terminate_console_input, fn());
crt_entry!(".CRT$XPX", __dcrt_console_output_terminator: Pvfv = __dcrt_terminate_console_output, fn());
crt_entry!(".CRT$XIC", __acrt_fmode_initializer: Pifv = __acrt_initialize_fmode, fn() -> c_int);
crt_entry!(".CRT$XPX", __acrt_locale_terminator: Pvfv = __acrt_uninitialize_locale, fn());
crt_entry!(".CRT$XIC", __acrt_stdio_initializer: Pifv = __acrt_initialize_stdio, fn() -> c_int);
crt_entry!(".CRT$XPXA", __acrt_stdio_terminator: Pvfv = __acrt_uninitialize_stdio, fn());
crt_entry!(".CRT$XIC", __acrt_timeset_initializer: Pifv = __acrt_initialize_timeset, fn() -> c_int);
crt_entry!(".CRT$XPX", __acrt_tmpfile_terminator: Pvfv = __acrt_uninitialize_tmpfile, fn());
#[cfg(not(target_arch = "x86"))]
crt_entry!(".CRT$XIC", __acrt_tran_fma3_initializer: Pifv = __acrt_initialize_fma3, fn() -> c_int);

/// `__acrt_initialize_multibyte` returns `bool` (true = success) but a `.CRT$XIC`
/// entry returns `int` (0 = success): the C wraps it to invert.
unsafe extern "C" fn initialize_multibyte() -> c_int {
    (!__acrt_initialize_multibyte()) as c_int
}
#[no_mangle]
#[used]
#[link_section = ".CRT$XIC"]
pub static __acrt_multibyte_initializer: Pifv = Some(initialize_multibyte);

/// i386 only: the SSE2 transcendental switch, which this build never takes.
#[cfg(target_arch = "x86")]
#[no_mangle]
pub extern "C" fn __acrt_initialize_sse2() -> c_int {
    0
}
#[cfg(target_arch = "x86")]
#[no_mangle]
#[used]
#[link_section = ".CRT$XIC"]
pub static __acrt_tran_sse2_initializer: Pifv = Some(__acrt_initialize_sse2);
