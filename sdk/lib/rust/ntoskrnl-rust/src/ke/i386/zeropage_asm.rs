//! Fast page zeroing — `@KeZeroPages@8` (i386 fastcall).
//!
//! Translated from `ntoskrnl/ke/i386/zeropage.S` (32 LOC of MASM/GAS) to
//! `core::arch::global_asm!` so the cargo PoC stops needing
//! `ntoskrnl/include` and `sdk/include/asm` `-I` paths just for `asm.inc`.

#[cfg(target_arch = "x86")]
core::arch::global_asm!(
    ".intel_syntax noprefix",
    ".text",
    ".globl @KeZeroPages@8",
    "@KeZeroPages@8:",
    ".cfi_startproc",
    "    push edi",
    "    mov edi, ecx",
    "    mov ecx, edx",
    "    shr ecx, 2",
    "    xor eax, eax",
    "    rep stosd",
    "    pop edi",
    "    ret",
    ".cfi_endproc",
);

// Force-pull: ensure the rlib's `.text` section for `@KeZeroPages@8`
// survives lld archive-stripping. Decoration `@Name@N` is fastcall.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn KeZeroPages(ptr: *mut core::ffi::c_void, size: u32);
}
#[cfg(target_arch = "x86")]
#[used]
static KE_ZERO_PAGES_REF: unsafe extern "fastcall" fn(*mut core::ffi::c_void, u32) =
    KeZeroPages;
