//! Empty bin shell shared by all 95 layout bins: a layout DLL has no entry point
//! (`-e __ImageBase`) and nothing here runs. Cargo needs a `main.rs` to drive the link.

#![no_std]
#![no_main]

/// A layout DLL imports nothing: win32k maps it and reads its tables in kernel context,
/// so the handler cannot call out to report. It fails fast instead (int 0x29 / brk
/// #0xF003, FAST_FAIL_FATAL_APP_EXIT), which the kernel turns into a loud bugcheck.
/// Unreachable in practice: the body is tables plus one accessor that cannot panic.
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    const FAST_FAIL_FATAL_APP_EXIT: u32 = 7;
    // SAFETY: __fastfail never returns and touches nothing but the code register.
    unsafe {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        core::arch::asm!("int 0x29", in("ecx") FAST_FAIL_FATAL_APP_EXIT, options(noreturn));
        #[cfg(target_arch = "aarch64")]
        core::arch::asm!("brk #0xf003", in("x0") FAST_FAIL_FATAL_APP_EXIT as u64, options(noreturn));
    }
}
