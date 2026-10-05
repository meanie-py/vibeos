//! `__chkstk`, the arm64 stack probe clang calls for a frame over a page (x15 = size / 16;
//! probes each page below sp, clobbers x16/x17). The CRT DLLs link with no clang runtime
//! archive (link_strict, sdk/cmake/gcc.cmake), and Rust's compiler_builtins has no arm64
//! `__chkstk`, so without this the name went to ntdll's import library. These are the seven
//! instructions of the libclang_rt.builtins-aarch64 `chkstk.S.obj` that was linked before,
//! as llvm-objdump shows them.
core::arch::global_asm!(
    ".text",
    ".globl __chkstk",
    ".p2align 2",
    "__chkstk:",
    "    lsl x16, x15, #4",
    "    mov x17, sp",
    "1:",
    "    sub x17, x17, #1, lsl #12",
    "    subs x16, x16, #1, lsl #12",
    "    ldr xzr, [x17]",
    "    b.gt 1b",
    "    ret",
);
