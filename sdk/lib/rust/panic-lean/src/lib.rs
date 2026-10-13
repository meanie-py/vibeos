//! The panic handler for a cargo cell whose body cannot panic.
//!
//! A cargo cell is an empty `#![no_std]` bin that drives a module's link, and rustc
//! insists it carry a `#[panic_handler]`. With `panic_common` there, every cell loaded
//! the whole of `core` whether or not anything could panic: rustc roots each
//! dependency's `#[no_mangle]` symbol in an executable link, panic_common's
//! `_Unwind_Resume` fallback is one, and `core` for the builtin windows-gnu triples
//! is one non-COMDAT section that lld's COFF gc cannot split. w32time.dll, 10 KB of
//! code from cmake, came out at 147 KB; across 94 modules amd64 grew 18.5 MB, and
//! kmtest MmMdl found one fewer 32 MB block of free memory.
//!
//! cmake pulled the body's own handler lazily, only when something referenced a
//! panic path. A cell whose cmake-built module carried no handler (the generator reads
//! that from the module's imports) uses this one instead. It references nothing in
//! `core`, not even `PanicInfo::location`, which i386's build-std `core` keeps out of
//! line. If the body ever gains a panic path the handler still reports and exits; the
//! cell should then go back to `panic_common` for the message and the stack.
#![no_std]

/// The exit status panic_common uses too ('RST').
pub const RUST_PANIC: u32 = 0xE052_5354;

extern "C" {
    fn DbgPrint(fmt: *const u8, ...) -> u32;
}

extern "system" {
    fn NtTerminateProcess(handle: isize, status: u32) -> i32;
}

/// Report the panic by module name and end the process. `module` is NUL-terminated.
#[inline(always)]
pub fn panic_lean(module: &'static [u8], _info: &core::panic::PanicInfo) -> ! {
    unsafe {
        DbgPrint(
            b"*** RUST PANIC in %s (panic-lean: rebuild the cell with panic_common for detail)\n\0"
                .as_ptr(),
            module.as_ptr(),
        );
        NtTerminateProcess(-1, RUST_PANIC);
    }
    #[allow(clippy::empty_loop)]
    loop {}
}
