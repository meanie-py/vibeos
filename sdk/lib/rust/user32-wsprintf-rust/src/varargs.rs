//! C-variadic exports as naked thunks.
//!
//! Stable Rust cannot DEFINE `int f(T a, U b, ...)`. On every Windows ABI this tree builds,
//! though, a `va_list` is a plain pointer to the variadic arguments laid out contiguously in
//! memory, so `f` only has to put its register-passed variadic arguments next to its
//! stack-passed ones and hand a pointer to the first of them to `vf(a, b, va)`:
//!
//! * i386 (cdecl): everything is already on the stack; `va` = the address after the last fixed
//!   argument. The fixed arguments are re-pushed: each push moves the stack by one slot, so
//!   every one of them is read from the same offset, `[esp + 4*N + 4]`.
//! * x86_64 (Microsoft x64): the first four arguments travel in rcx/rdx/r8/r9 and the caller
//!   reserves their 32-byte home area right below the stack arguments. The variadic register
//!   arguments are spilled into their own home slots, which makes them contiguous with the
//!   rest; the call then gets a fresh frame so `vf` owns a home area of its own.
//! * aarch64 (Windows ARM64, variadic convention): x0-x7 then the stack, floating point in
//!   integer registers too. x(N)..x7 are stored so they end exactly at the caller's stack
//!   arguments.
//!
//! Each 64-bit thunk calls out, so it carries SEH unwind directives: rustc emits none for a
//! naked function, and an exception unwinding through a frame without .pdata is lost.

/// `va_thunk!(N, export, helper)`: `export(fixed_1..fixed_N, ...)` calls
/// `helper(fixed_1..fixed_N, va_list)` (all cdecl) and returns its result. Only N = 2 exists
/// so far (wsprintfA/W); other arities follow the layouts in the module docs.
macro_rules! va_thunk {
    (2, $export:ident, $helper:path) => {
        #[cfg(target_arch = "x86")]
        #[unsafe(naked)]
        #[no_mangle]
        pub unsafe extern "C" fn $export() {
            core::arch::naked_asm!(
                "lea eax, [esp + 12]",
                "push eax",
                "push dword ptr [esp + 12]",
                "push dword ptr [esp + 12]",
                "call {h}",
                "add esp, 12",
                "ret",
                h = sym $helper,
            );
        }
        #[cfg(target_arch = "x86_64")]
        #[unsafe(naked)]
        #[no_mangle]
        pub unsafe extern "C" fn $export() {
            core::arch::naked_asm!(
                concat!(".seh_proc ", stringify!($export)),
                "mov [rsp + 24], r8",
                "mov [rsp + 32], r9",
                "lea r8, [rsp + 24]",
                "sub rsp, 40",
                ".seh_stackalloc 40",
                ".seh_endprologue",
                "call {h}",
                "add rsp, 40",
                "ret",
                ".seh_endproc",
                h = sym $helper,
            );
        }
        #[cfg(target_arch = "aarch64")]
        #[unsafe(naked)]
        #[no_mangle]
        pub unsafe extern "C" fn $export() {
            core::arch::naked_asm!(
                concat!(".seh_proc ", stringify!($export)),
                // 16 bytes fp/lr + x2..x7 (48), ending at the caller's stack arguments.
                "stp x29, x30, [sp, #-64]!",
                ".seh_save_fplr_x 64",
                "mov x29, sp",
                ".seh_set_fp",
                ".seh_endprologue",
                "stp x2, x3, [sp, #16]",
                "stp x4, x5, [sp, #32]",
                "stp x6, x7, [sp, #48]",
                "add x2, sp, #16",
                "bl {h}",
                ".seh_startepilogue",
                "ldp x29, x30, [sp], #64",
                ".seh_save_fplr_x 64",
                ".seh_endepilogue",
                "ret",
                ".seh_endproc",
                h = sym $helper,
            );
        }
    };
}

// user32.spec: `@ varargs wsprintfA(ptr str)` / `@ varargs wsprintfW(ptr wstr)`.
va_thunk!(2, wsprintfA, crate::wsprintf_va_a);
va_thunk!(2, wsprintfW, crate::wsprintf_va_w);
