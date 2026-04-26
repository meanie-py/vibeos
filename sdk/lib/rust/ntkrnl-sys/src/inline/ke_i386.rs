//! KE subsystem i386-specific inline functions.
//!
//! Compiler intrinsics and FORCEINLINE functions from intrin_i.h,
//! internal/i386/ke.h, and related headers.

#![cfg(target_arch = "x86")]

use crate::base::*;
use crate::ddk;
use core::arch::asm;

// ============================================================================
// Control register access (stdcall-decorated global asm symbols)
// ============================================================================

core::arch::global_asm!(
    // ___readcr0@0
    ".globl ___readcr0@0",
    "___readcr0@0:",
    "  mov eax, cr0",
    "  ret",

    // ___writecr0@4
    ".globl ___writecr0@4",
    "___writecr0@4:",
    "  mov eax, [esp+4]",
    "  mov cr0, eax",
    "  ret 4",

    // ___readcr2@0
    ".globl ___readcr2@0",
    "___readcr2@0:",
    "  mov eax, cr2",
    "  ret",

    // ___readcr3@0
    ".globl ___readcr3@0",
    "___readcr3@0:",
    "  mov eax, cr3",
    "  ret",

    // ___readcr4@0
    ".globl ___readcr4@0",
    "___readcr4@0:",
    "  mov eax, cr4",
    "  ret",

    // ___writecr3@4
    ".globl ___writecr3@4",
    "___writecr3@4:",
    "  mov eax, [esp+4]",
    "  mov cr3, eax",
    "  ret 4",

    // ___sgdt@4
    ".globl ___sgdt@4",
    "___sgdt@4:",
    "  mov eax, [esp+4]",
    "  sgdt [eax]",
    "  ret 4",

    // ___sidt@4
    ".globl ___sidt@4",
    "___sidt@4:",
    "  mov eax, [esp+4]",
    "  sidt [eax]",
    "  ret 4",

    // ___invlpg@4
    ".globl ___invlpg@4",
    "___invlpg@4:",
    "  mov eax, [esp+4]",
    "  invlpg [eax]",
    "  ret 4",

    // ___readfsdword@4
    ".globl ___readfsdword@4",
    "___readfsdword@4:",
    "  mov ecx, [esp+4]",
    "  mov eax, fs:[ecx]",
    "  ret 4",

    // ___writefsdword@8
    ".globl ___writefsdword@8",
    "___writefsdword@8:",
    "  mov ecx, [esp+4]",
    "  mov eax, [esp+8]",
    "  mov fs:[ecx], eax",
    "  ret 8",

    // __disable@0
    ".globl __disable@0",
    "__disable@0:",
    "  cli",
    "  ret",

    // __enable@0
    ".globl __enable@0",
    "__enable@0:",
    "  sti",
    "  ret",

    // ___debugbreak@0
    ".globl ___debugbreak@0",
    "___debugbreak@0:",
    "  int3",
    "  ret",

    // ___readeflags@0
    ".globl ___readeflags@0",
    "___readeflags@0:",
    "  pushfd",
    "  pop eax",
    "  ret",

    // ___writeeflags@4
    ".globl ___writeeflags@4",
    "___writeeflags@4:",
    "  mov eax, [esp+4]",
    "  push eax",
    "  popfd",
    "  ret 4",

    // ___rdtsc@0
    ".globl ___rdtsc@0",
    "___rdtsc@0:",
    "  rdtsc",
    "  ret",

    // _InterlockedBitTestAndSet@8
    ".globl _InterlockedBitTestAndSet@8",
    "_InterlockedBitTestAndSet@8:",
    "  mov ecx, [esp+4]",
    "  mov edx, [esp+8]",
    "  xor eax, eax",
    "  lock bts [ecx], edx",
    "  setc al",
    "  ret 8",

    // _InterlockedBitTestAndReset@8
    ".globl _InterlockedBitTestAndReset@8",
    "_InterlockedBitTestAndReset@8:",
    "  mov ecx, [esp+4]",
    "  mov edx, [esp+8]",
    "  xor eax, eax",
    "  lock btr [ecx], edx",
    "  setc al",
    "  ret 8",

    // _Ke386FnInit@0
    ".globl _Ke386FnInit@0",
    "_Ke386FnInit@0:",
    "  fninit",
    "  ret",

    // _Ke386SaveFpuState@4
    ".globl _Ke386SaveFpuState@4",
    "_Ke386SaveFpuState@4:",
    "  mov eax, [esp+4]",
    "  fxsave [eax]",
    "  ret 4",

    // _Ke386RestoreFpuState@4
    ".globl _Ke386RestoreFpuState@4",
    "_Ke386RestoreFpuState@4:",
    "  mov eax, [esp+4]",
    "  fxrstor [eax]",
    "  ret 4",

    // _Ke386ClearFpExceptions@0
    ".globl _Ke386ClearFpExceptions@0",
    "_Ke386ClearFpExceptions@0:",
    "  fnclex",
    "  ret",

    // KfRaiseIrql stdcall wrapper → fastcall HAL
    ".globl _KfRaiseIrql@4",
    "_KfRaiseIrql@4:",
    "  mov ecx, [esp+4]",
    "  jmp @KfRaiseIrql@4",

    // KfLowerIrql stdcall wrapper → fastcall HAL
    ".globl _KfLowerIrql@4",
    "_KfLowerIrql@4:",
    "  mov ecx, [esp+4]",
    "  jmp @KfLowerIrql@4",
);

// ============================================================================
// cdecl versions of intrinsics (used by Rust callers via extern "C")
// The stdcall versions above are kept for any C code using @N decoration.
// ============================================================================

core::arch::global_asm!(
    ".intel_syntax noprefix",

    // __readcr0 (cdecl, 0 args — same as stdcall)
    ".globl ___readcr0",
    "___readcr0:",
    "  mov eax, cr0",
    "  ret",

    // __readcr3 (cdecl, 0 args)
    ".globl ___readcr3",
    "___readcr3:",
    "  mov eax, cr3",
    "  ret",

    // __sgdt (cdecl, 1 arg — caller cleans stack)
    ".globl ___sgdt",
    "___sgdt:",
    "  mov eax, [esp+4]",
    "  sgdt [eax]",
    "  ret",

    // __sidt (cdecl, 1 arg)
    ".globl ___sidt",
    "___sidt:",
    "  mov eax, [esp+4]",
    "  sidt [eax]",
    "  ret",

    // __readcr2 (cdecl, 0 args)
    ".globl ___readcr2",
    "___readcr2:",
    "  mov eax, cr2",
    "  ret",

    // __readcr4 (cdecl, 0 args)
    ".globl ___readcr4",
    "___readcr4:",
    "  mov eax, cr4",
    "  ret",

    // __writecr0 (cdecl, 1 arg)
    ".globl ___writecr0",
    "___writecr0:",
    "  mov eax, [esp+4]",
    "  mov cr0, eax",
    "  ret",

    // __writecr3 (cdecl, 1 arg)
    ".globl ___writecr3",
    "___writecr3:",
    "  mov eax, [esp+4]",
    "  mov cr3, eax",
    "  ret",
);

// ============================================================================
// GDT helpers (i386 — uses KGDTENTRY, not KGDTENTRY64)
// ============================================================================

/// KiGetGdtEntry — get GDT entry for a given selector (i386 version).
/// C: `return (PKGDTENTRY)((ULONG_PTR)pGdt + (Selector & ~RPL_MASK));`
#[inline]
pub unsafe fn KiGetGdtEntry(gdt: PVOID, selector: u16) -> *mut ddk::_KGDTENTRY {
    (gdt as *mut u8).add((selector & !7u16) as usize) as *mut ddk::_KGDTENTRY
}

/// KiSetGdtDescriptorBase — set base address in i386 GDT entry.
#[inline]
pub unsafe fn KiSetGdtDescriptorBase(entry: *mut ddk::_KGDTENTRY, base: ULONG_PTR) {
    let e = &mut *entry;
    e.BaseLow = base as u16;
    e.HighWord.Bytes.BaseMid = ((base >> 16) & 0xFF) as u8;
    e.HighWord.Bytes.BaseHi = ((base >> 24) & 0xFF) as u8;
}

// ============================================================================
// RtlFillMemoryUlong (i386 — rep stosd)
// ============================================================================

/// RtlFillMemoryUlong — fill memory with a ULONG pattern.
#[inline]
pub unsafe fn RtlFillMemoryUlong(dest: PVOID, length: usize, pattern: ULONG) {
    let count = length / 4;
    asm!(
        "rep stosd",
        inout("ecx") count => _,
        inout("edi") dest => _,
        in("eax") pattern,
        options(nostack),
    );
}

// ============================================================================
// KeMemoryBarrier — i386 uses xchg-based full fence
// ============================================================================

static mut KE_MEMORY_BARRIER_DUMMY: i32 = 0;

#[inline]
pub unsafe fn KeMemoryBarrier() {
    asm!(
        "xchg [{dummy}], {tmp:e}",
        dummy = sym KE_MEMORY_BARRIER_DUMMY,
        tmp = inout(reg) 0i32 => _,
        options(nostack),
    );
}
