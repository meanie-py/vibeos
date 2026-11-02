//! The i386 `_CI*` functions: x87-stack math thunks every Windows CRT exports and no C
//! compiler emits portably. Each pops its operand(s) off the x87 stack, spills whatever else
//! the caller left there, calls the plain cdecl function of the same name (`_cos`, `_pow`,
//! ...) which the linker resolves to the binary's own body, and restores the stack around
//! the result. Wine's msvcrt math.c:477-566, moved here from crt-shared-rust's
//! `crtmath_i386.rs` so the i386 kernel and ntdll (which export five of them) can drop
//! crtmath's `math/i386/ci*.c`. Those C bodies only popped the operand; these also keep
//! the rest of the caller's x87 stack intact, which is the contract MSVC's callers assume.
//!
//! The `.def/.scl/.endef` directive is COFF, so this only assembles for a PE target.
#![no_std]
#![cfg_attr(not(all(target_arch = "x86", target_os = "windows")), allow(unused_macros))]

// macro_rules! mirror of CREATE_FPU_FUNC1(name, call) — math.c:477-511.
// One double popped off the FPU stack in, one pushed back out.
#[allow(unused_macros)]
macro_rules! create_fpu_func1 {
    ($name:ident, $call:ident) => {
        core::arch::global_asm!(
            ".text",
            ".align 4",
            concat!(".globl _", stringify!($name)),
            concat!(".def _", stringify!($name), "; .scl 2; .type 32; .endef"),
            concat!("_", stringify!($name), ":"),
            "pushl   %ebp",
            "movl    %esp, %ebp",
            "subl    $68, %esp", // sizeof(double)*8 + sizeof(int)
            "fstpl   (%esp)",    // store function argument
            "fwait",
            "movl    $1, %ecx", // empty FPU stack
            "1:",
            "fxam",
            "fstsw   %ax",
            "and     $0x4500, %ax",
            "cmp     $0x4100, %ax",
            "je      2f",
            "fstpl    (%esp,%ecx,8)",
            "fwait",
            "incl    %ecx",
            "jmp     1b",
            "2:",
            "movl    %ecx, -4(%ebp)",
            concat!("call    _", stringify!($call)),
            "movl    -4(%ebp), %ecx",
            "fstpl   (%esp)", // save result
            "3:",             // restore FPU stack
            "decl    %ecx",
            "fldl    (%esp,%ecx,8)",
            "cmpl    $0, %ecx",
            "jne     3b",
            "leave",
            "ret",
            options(att_syntax)
        );
    };
}

// macro_rules! mirror of CREATE_FPU_FUNC2(name, call) — math.c:513-549.
// Two doubles popped off the FPU stack in (2nd arg deeper), one pushed back out.
#[allow(unused_macros)]
macro_rules! create_fpu_func2 {
    ($name:ident, $call:ident) => {
        core::arch::global_asm!(
            ".text",
            ".align 4",
            concat!(".globl _", stringify!($name)),
            concat!(".def _", stringify!($name), "; .scl 2; .type 32; .endef"),
            concat!("_", stringify!($name), ":"),
            "pushl   %ebp",
            "movl    %esp, %ebp",
            "subl    $68, %esp", // sizeof(double)*8 + sizeof(int)
            "fstpl   8(%esp)",   // store function argument
            "fwait",
            "fstpl   (%esp)",
            "fwait",
            "movl    $2, %ecx", // empty FPU stack
            "1:",
            "fxam",
            "fstsw   %ax",
            "and     $0x4500, %ax",
            "cmp     $0x4100, %ax",
            "je      2f",
            "fstpl    (%esp,%ecx,8)",
            "fwait",
            "incl    %ecx",
            "jmp     1b",
            "2:",
            "movl    %ecx, -4(%ebp)",
            concat!("call    _", stringify!($call)),
            "movl    -4(%ebp), %ecx",
            "fstpl   8(%esp)", // save result
            "3:",              // restore FPU stack
            "decl    %ecx",
            "fldl    (%esp,%ecx,8)",
            "cmpl    $1, %ecx",
            "jne     3b",
            "leave",
            "ret",
            options(att_syntax)
        );
    };
}

#[cfg(all(target_arch = "x86", target_os = "windows", feature = "kernel"))]
mod kernel {
    // ntoskrnl.spec exports _CIcos/_CIsin/_CIsqrt and ntdll.spec those plus _CIlog/_CIpow.
    create_fpu_func1!(_CIcos, cos);
    create_fpu_func1!(_CIlog, log);
    create_fpu_func2!(_CIpow, pow);
    create_fpu_func1!(_CIsin, sin);
    create_fpu_func1!(_CIsqrt, sqrt);
}

#[cfg(all(target_arch = "x86", target_os = "windows", feature = "all"))]
mod all {
    // math.c:551-566's other eleven; `kernel` above holds the rest of the sixteen.
    create_fpu_func1!(_CIacos, acos);
    create_fpu_func1!(_CIasin, asin);
    create_fpu_func1!(_CIatan, atan);
    create_fpu_func2!(_CIatan2, atan2);
    create_fpu_func1!(_CIcosh, cosh);
    create_fpu_func1!(_CIexp, exp);
    create_fpu_func2!(_CIfmod, fmod);
    create_fpu_func1!(_CIlog10, log10);
    create_fpu_func1!(_CIsinh, sinh);
    create_fpu_func1!(_CItan, tan);
    create_fpu_func1!(_CItanh, tanh);
}
