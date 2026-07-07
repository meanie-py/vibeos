//! `@RtlPrefetchMemoryNonTemporal@8` (i386 fastcall).
//!
//! Translated from `ntoskrnl/rtl/i386/prefetch.S` (44 LOC of MASM/GAS).
//! The first instruction is `ret`; on init the kernel patches that byte
//! to `nop` (in `kiinit.rs`) when CPUID reports prefetchnta is available.
//! Until then, every call returns immediately. Same trick the original
//! asm used.

#[cfg(target_arch = "x86")]
core::arch::global_asm!(
    ".intel_syntax noprefix",
    ".text",
    ".globl @RtlPrefetchMemoryNonTemporal@8",
    "@RtlPrefetchMemoryNonTemporal@8:",
    ".cfi_startproc",
    // Kernel overwrites this to `nop` during init if prefetchnta is
    // available. Slight optimization vs. checking KeI386XMMIPresent on
    // every call.
    "    ret",
    "    mov eax, [_KePrefetchNTAGranularity]",
    "FetchLine:",
    "    prefetchnta byte ptr [ecx]",
    "    add ecx, eax",
    "    sub edx, eax",
    "    ja FetchLine",
    "    ret",
    ".cfi_endproc",
);

// Force-pull so lld doesn't strip the symbol out of libkernel_rust.a.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn RtlPrefetchMemoryNonTemporal(src: *const core::ffi::c_void, len: usize);
}
#[cfg(target_arch = "x86")]
#[used]
static RTL_PREFETCH_REF: unsafe extern "fastcall" fn(*const core::ffi::c_void, usize) =
    RtlPrefetchMemoryNonTemporal;
