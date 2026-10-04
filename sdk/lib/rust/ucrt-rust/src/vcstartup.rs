//! The C ucrtbase linked from sdk/lib/vcstartup and sdk/lib/vcruntime, in Rust:
//! initializers.cpp (the `.CRT$X?A`/`X?Z` bracket arrays), tlssup.c (the TLS directory),
//! __security_init_cookie.c, __report_gsfailure.c, isa_available.cpp, purecall.cpp and
//! __vcrt_init.c. Those libraries stay on ucrtbase's link line (and keep serving the programs
//! that link vcstartup statically); this archive is linked whole ahead of them, so these
//! definitions win first-match and none of their objects is pulled into ucrtbase.

use core::ffi::{c_int, c_void};
use core::ptr::{addr_of, null, null_mut};
use core::sync::atomic::{AtomicPtr, Ordering};

type Pifv = Option<unsafe extern "C" fn() -> c_int>;
type Pvfv = Option<unsafe extern "C" fn()>;
type TlsCallback = Option<unsafe extern "system" fn(*mut c_void, u32, *mut c_void)>;

extern "system" {
    fn IsProcessorFeaturePresent(feature: u32) -> c_int;
    fn GetTickCount() -> u32;
    fn QueryPerformanceCounter(counter: *mut i64) -> c_int;
    fn GetCurrentThreadId() -> u32;
    fn GetCurrentProcessId() -> u32;
    fn GetSystemTimeAsFileTime(time: *mut u64); // FILETIME, as time/time_get.rs declares it
    fn GetModuleHandleW(name: *const u16) -> *mut c_void;
}
extern "C" {
    fn abort() -> !;
    fn msvcrt_init_exception(base: *mut c_void);
    fn msvcrt_init_tls() -> c_int;
    fn msvcrt_free_tls() -> c_int;
    fn msvcrt_free_tls_mem();
}

// ---- initializers.cpp ------------------------------------------------------------------

macro_rules! bracket {
    ($($name:ident: $ty:ty = $sect:literal;)*) => {$(
        #[no_mangle]
        #[link_section = $sect]
        #[used]
        pub static $name: [$ty; 1] = [None];
    )*};
}
bracket! {
    __xi_a: Pifv = ".CRT$XIA";
    __xi_z: Pifv = ".CRT$XIZ";
    __xc_a: Pvfv = ".CRT$XCA";
    __xc_z: Pvfv = ".CRT$XCZ";
    __xp_a: Pvfv = ".CRT$XPA";
    __xp_z: Pvfv = ".CRT$XPZ";
    __xt_a: Pvfv = ".CRT$XTA";
    __xt_z: Pvfv = ".CRT$XTZ";
}

// ---- tlssup.c --------------------------------------------------------------------------

#[no_mangle]
pub static mut _tls_index: u32 = 0;
#[no_mangle]
#[link_section = ".tls"]
#[used]
pub static mut _tls_start: *mut u8 = null_mut();
#[no_mangle]
#[link_section = ".tls$ZZZ"]
#[used]
pub static mut _tls_end: *mut u8 = null_mut();
#[no_mangle]
#[link_section = ".CRT$XLA"]
#[used]
pub static __xl_a: TlsCallback = None;
#[no_mangle]
#[link_section = ".CRT$XLZ"]
#[used]
pub static __xl_z: TlsCallback = None;

/// IMAGE_TLS_DIRECTORY.
#[repr(C)]
pub struct TlsDirectory {
    start: *const u8,
    end: *const u8,
    index: *const u32,
    callbacks: *const TlsCallback,
    size_of_zero_fill: u32,
    characteristics: u32,
}
unsafe impl Sync for TlsDirectory {}

#[no_mangle]
#[used]
pub static _tls_used: TlsDirectory = TlsDirectory {
    start: addr_of!(_tls_start) as *const u8,
    end: addr_of!(_tls_end) as *const u8,
    index: addr_of!(_tls_index),
    callbacks: addr_of!(__xl_a).wrapping_add(1),
    size_of_zero_fill: 0,
    characteristics: 0,
};

// ---- __vcrt_init.c ---------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "C" fn __vcrt_initialize() -> bool {
    msvcrt_init_exception(GetModuleHandleW(null()));
    msvcrt_init_tls() != 0
}
#[no_mangle]
pub unsafe extern "C" fn __vcrt_uninitialize(_terminating: bool) -> bool {
    msvcrt_free_tls() != 0
}
#[no_mangle]
pub extern "C" fn __vcrt_uninitialize_critical() -> bool {
    true
}
/// Not called by UCRT.
#[no_mangle]
pub extern "C" fn __vcrt_thread_attach() -> bool {
    true
}
/// Not called by UCRT.
#[no_mangle]
pub extern "C" fn __vcrt_thread_detach() -> bool {
    true
}

#[no_mangle]
pub static __p_tls_used: &TlsDirectory = &_tls_used;

/// Frees the thread's msvcrt_shared data; for the last thread only DLL_PROCESS_DETACH comes.
unsafe extern "system" fn wine_tls_cleanup_callback(_dll: *mut c_void, reason: u32, _reserved: *mut c_void) {
    const DLL_PROCESS_DETACH: u32 = 0;
    const DLL_THREAD_DETACH: u32 = 3;
    if reason == DLL_THREAD_DETACH || reason == DLL_PROCESS_DETACH {
        msvcrt_free_tls_mem();
    }
}
#[no_mangle]
#[link_section = ".CRT$XLD"]
#[used]
pub static wine_tls_cleanup_ptr: TlsCallback = Some(wine_tls_cleanup_callback);

// ---- purecall.cpp ----------------------------------------------------------------------

static PURECALL_HANDLER: AtomicPtr<c_void> = AtomicPtr::new(null_mut());

#[no_mangle]
pub extern "C" fn _get_purecall_handler() -> Pvfv {
    unsafe { core::mem::transmute::<*mut c_void, Pvfv>(PURECALL_HANDLER.load(Ordering::SeqCst)) }
}
#[no_mangle]
pub extern "C" fn _set_purecall_handler(handler: Pvfv) -> Pvfv {
    let new = unsafe { core::mem::transmute::<Pvfv, *mut c_void>(handler) };
    unsafe { core::mem::transmute::<*mut c_void, Pvfv>(PURECALL_HANDLER.swap(new, Ordering::SeqCst)) }
}
#[no_mangle]
pub unsafe extern "C" fn _purecall() -> c_int {
    if let Some(handler) = _get_purecall_handler() {
        handler();
    }
    abort()
}

// ---- __report_gsfailure.c --------------------------------------------------------------

/// `__fastfail(FAST_FAIL_STACK_COOKIE_CHECK_FAILURE)`: code 2, raised the way the
/// intrinsic is (int 0x29 with the code in ecx/rcx; brk #0xF003 with it in x0). Not
/// `options(noreturn)`: rustc follows a noreturn asm with a `ud2` trap, which scan_ud2
/// (rightly) reads as a miscompile; the kernel never returns from a fast fail, and the
/// loop after it costs one jump that never runs.
#[inline(always)]
unsafe fn fastfail_stack_cookie() -> ! {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    core::arch::asm!("int 0x29", in("ecx") 2u32, options(nostack));
    #[cfg(target_arch = "aarch64")]
    core::arch::asm!("brk #0xF003", in("x0") 2u64, options(nostack));
    #[allow(clippy::empty_loop)]
    loop {} // nolint: silent-failure (after a fast fail, which the kernel never returns from)
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "C" fn __report_gsfailure() -> ! {
    fastfail_stack_cookie()
}
#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn __report_gsfailure(_stack_cookie: usize) -> ! {
    fastfail_stack_cookie()
}

// ---- __security_init_cookie.c ----------------------------------------------------------

#[cfg(target_pointer_width = "64")]
const DEFAULT_SECURITY_COOKIE: usize = 0x0000_2B99_2DDF_A232;
#[cfg(target_pointer_width = "32")]
const DEFAULT_SECURITY_COOKIE: usize = 0xBB40_E64E;

#[no_mangle]
pub static mut __security_cookie: usize = DEFAULT_SECURITY_COOKIE;
#[no_mangle]
pub static mut __security_cookie_complement: usize = !DEFAULT_SECURITY_COOKIE;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
unsafe fn rdrand() -> usize {
    loop {
        let v: usize;
        let ok: u8;
        core::arch::asm!("rdrand {v}", "setc {ok}", v = out(reg) v, ok = out(reg_byte) ok, options(nomem, nostack));
        if ok != 0 {
            return v;
        }
        core::arch::asm!("pause", options(nomem, nostack, preserves_flags));
    }
}

#[no_mangle]
pub unsafe extern "C" fn __security_init_cookie() {
    let mut random_value: usize = 0x27E3_0B2C_16B0_7297u64 as usize;

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        const PF_RDTSC_INSTRUCTION_AVAILABLE: u32 = 8;
        const PF_RDRAND_INSTRUCTION_AVAILABLE: u32 = 28;
        if IsProcessorFeaturePresent(PF_RDRAND_INSTRUCTION_AVAILABLE) != 0 {
            random_value = rdrand();
        }
        if IsProcessorFeaturePresent(PF_RDTSC_INSTRUCTION_AVAILABLE) != 0 {
            #[cfg(target_arch = "x86_64")]
            let tsc = core::arch::x86_64::_rdtsc();
            #[cfg(target_arch = "x86")]
            let tsc = core::arch::x86::_rdtsc();
            random_value = random_value.wrapping_add(tsc as usize);
        }
    }
    random_value = random_value.wrapping_add(addr_of!(random_value) as usize);
    random_value ^= GetTickCount() as usize;
    let mut counter = 0i64;
    QueryPerformanceCounter(&mut counter);
    #[cfg(target_pointer_width = "64")]
    {
        random_value ^= counter as usize;
    }
    #[cfg(target_pointer_width = "32")]
    {
        random_value ^= counter as u32 as usize;
        random_value ^= (counter >> 32) as u32 as usize;
    }
    random_value = random_value.wrapping_add(GetCurrentThreadId() as usize);
    random_value = random_value.rotate_left(GetCurrentThreadId() >> 2);
    // _WIN32_WINNT is 0x502 here, so the C took GetSystemTimeAsFileTime, not the Precise one.
    let mut file_time = 0u64;
    GetSystemTimeAsFileTime(&mut file_time);
    random_value = random_value.wrapping_add(file_time as u32 as usize); // dwLowDateTime
    random_value = random_value.wrapping_add((file_time >> 32) as u32 as usize); // dwHighDateTime
    random_value = random_value.wrapping_add(GetCurrentProcessId() as usize);
    random_value = random_value.rotate_left(GetCurrentProcessId() >> 2);
    #[cfg(target_pointer_width = "64")]
    {
        random_value &= 0x0000_FFFF_FFFF_FFFF; // zero out the highest 16 bits
    }
    if random_value == DEFAULT_SECURITY_COOKIE {
        random_value += 1; // avoid the default security cookie
    }
    __security_cookie = random_value;
    __security_cookie_complement = !random_value;
}

// ---- isa_available.cpp -----------------------------------------------------------------

#[no_mangle]
pub static mut __isa_available: c_int = 0;

#[no_mangle]
pub unsafe extern "C" fn __isa_available_init() -> c_int {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        // (feature, __ISA_AVAILABLE_*), best first; __ISA_AVAILABLE_X86 = 0 otherwise.
        const LEVELS: [(u32, c_int); 5] = [(41, 6), (40, 5), (39, 3), (38, 2), (10, 1)];
        __isa_available = 0;
        for (feature, level) in LEVELS {
            if IsProcessorFeaturePresent(feature) != 0 {
                __isa_available = level;
                break;
            }
        }
    }
    #[cfg(target_arch = "aarch64")]
    {
        const PF_ARM_V8_INSTRUCTIONS_AVAILABLE: u32 = 29;
        // __ISA_AVAILABLE_NEON_ARM64 = 2, else __ISA_AVAILABLE_ARMNT = 0 (the C's CHECKME).
        __isa_available = if IsProcessorFeaturePresent(PF_ARM_V8_INSTRUCTIONS_AVAILABLE) != 0 { 2 } else { 0 };
    }
    0
}
