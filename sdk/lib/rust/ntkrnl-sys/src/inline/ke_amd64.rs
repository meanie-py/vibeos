//! KE subsystem amd64-specific inline functions.
//!
//! Compiler intrinsics and FORCEINLINE functions from intrin_i.h,
//! internal/amd64/ke.h, and related headers.

#![cfg(target_arch = "x86_64")]

use crate::base::*;
use crate::ddk;
use core::arch::asm;

// ============================================================================
// Control register access
// ============================================================================

#[inline]
pub unsafe fn __readcr0() -> ULONG_PTR {
    let val: u64;
    asm!("mov {}, cr0", out(reg) val, options(nomem, nostack));
    val as ULONG_PTR
}

#[inline]
pub unsafe fn __readcr2() -> ULONG_PTR {
    let val: u64;
    asm!("mov {}, cr2", out(reg) val, options(nomem, nostack));
    val as ULONG_PTR
}

#[inline]
pub unsafe fn __readcr3() -> ULONG_PTR {
    let val: u64;
    asm!("mov {}, cr3", out(reg) val, options(nomem, nostack));
    val as ULONG_PTR
}

#[inline]
pub unsafe fn __readcr4() -> ULONG_PTR {
    let val: u64;
    asm!("mov {}, cr4", out(reg) val, options(nomem, nostack));
    val as ULONG_PTR
}

#[inline]
pub unsafe fn __readcr8() -> ULONG_PTR {
    let val: u64;
    asm!("mov {}, cr8", out(reg) val, options(nomem, nostack));
    val as ULONG_PTR
}

#[inline]
pub unsafe fn __writecr0(val: ULONG_PTR) {
    asm!("mov cr0, {}", in(reg) val as u64, options(nomem, nostack));
}

#[inline]
pub unsafe fn __writecr3(val: ULONG_PTR) {
    asm!("mov cr3, {}", in(reg) val as u64, options(nomem, nostack));
}

#[inline]
pub unsafe fn __writecr4(val: ULONG_PTR) {
    asm!("mov cr4, {}", in(reg) val as u64, options(nomem, nostack));
}

#[inline]
pub unsafe fn __writecr8(val: ULONG_PTR) {
    asm!("mov cr8, {}", in(reg) val as u64, options(nomem, nostack));
}

// ============================================================================
// Debug register access
// ============================================================================

#[inline]
pub unsafe fn __readdr(reg: ULONG) -> ULONG_PTR {
    let val: u64;
    match reg {
        0 => asm!("mov {}, dr0", out(reg) val, options(nomem, nostack)),
        1 => asm!("mov {}, dr1", out(reg) val, options(nomem, nostack)),
        2 => asm!("mov {}, dr2", out(reg) val, options(nomem, nostack)),
        3 => asm!("mov {}, dr3", out(reg) val, options(nomem, nostack)),
        6 => asm!("mov {}, dr6", out(reg) val, options(nomem, nostack)),
        7 => asm!("mov {}, dr7", out(reg) val, options(nomem, nostack)),
        _ => { val = 0; }
    }
    val as ULONG_PTR
}

#[inline]
pub unsafe fn __writedr(reg: ULONG, val: ULONG_PTR) {
    match reg {
        0 => asm!("mov dr0, {}", in(reg) val as u64, options(nomem, nostack)),
        1 => asm!("mov dr1, {}", in(reg) val as u64, options(nomem, nostack)),
        2 => asm!("mov dr2, {}", in(reg) val as u64, options(nomem, nostack)),
        3 => asm!("mov dr3, {}", in(reg) val as u64, options(nomem, nostack)),
        6 => asm!("mov dr6, {}", in(reg) val as u64, options(nomem, nostack)),
        7 => asm!("mov dr7, {}", in(reg) val as u64, options(nomem, nostack)),
        _ => {}
    }
}

// ============================================================================
// MSR access
// ============================================================================

#[inline]
pub unsafe fn __readmsr(reg: ULONG) -> u64 {
    let lo: u32;
    let hi: u32;
    asm!(
        "rdmsr",
        in("ecx") reg,
        out("eax") lo,
        out("edx") hi,
        options(nomem, nostack),
    );
    ((hi as u64) << 32) | (lo as u64)
}

#[inline]
pub unsafe fn __writemsr(reg: ULONG, val: u64) {
    asm!(
        "wrmsr",
        in("ecx") reg,
        in("eax") val as u32,
        in("edx") (val >> 32) as u32,
        options(nomem, nostack),
    );
}

// ============================================================================
// Descriptor table operations
// ============================================================================

#[inline]
pub unsafe fn __lidt(source: PVOID) {
    asm!("lidt [{}]", in(reg) source, options(nostack));
}

#[inline]
pub unsafe fn __lgdt(source: PVOID) {
    asm!("lgdt [{}]", in(reg) source, options(nostack));
}

#[inline]
pub unsafe fn __sidt(dest: PVOID) {
    asm!("sidt [{}]", in(reg) dest, options(nostack));
}

#[inline]
pub unsafe fn __sgdt(dest: PVOID) {
    asm!("sgdt [{}]", in(reg) dest, options(nostack));
}

#[inline]
pub unsafe fn __sldt(ldtr: *mut u16) {
    asm!("sldt [{}]", in(reg) ldtr, options(nostack));
}

#[inline]
pub unsafe fn __str(tr: *mut u16) {
    asm!("str [{}]", in(reg) tr, options(nostack));
}

// ============================================================================
// Cache / barrier / misc
// ============================================================================

#[inline]
pub unsafe fn __wbinvd() {
    asm!("wbinvd", options(nostack));
}

#[inline]
pub unsafe fn __swapgs() {
    asm!("swapgs", options(nostack));
}

#[inline]
pub unsafe fn KeMemoryBarrier() {
    asm!("mfence", options(nostack));
}

// ============================================================================
// IDT helpers (FORCEINLINE from internal/amd64/ke.h)
// ============================================================================

/// KeQueryInterruptHandler — read IDT entry handler address.
#[inline]
pub unsafe fn KeQueryInterruptHandler(vector: ULONG) -> PVOID {
    let pcr = super::ke::KeGetPcr();
    let idt = (*pcr).IdtBase;
    let entry = &(*idt.add(vector as usize)).__bindgen_anon_1;
    let low = entry.OffsetLow as u64;
    let mid = entry.OffsetMiddle as u64;
    let high = entry.OffsetHigh as u64;
    (low | (mid << 16) | (high << 32)) as PVOID
}

/// KeRegisterInterruptHandler — write IDT entry handler address.
#[inline]
pub unsafe fn KeRegisterInterruptHandler(vector: ULONG, handler: PVOID) {
    let pcr = super::ke::KeGetPcr();
    let idt = (*pcr).IdtBase;
    let entry = &mut (*idt.add(vector as usize)).__bindgen_anon_1;
    let addr = handler as u64;
    entry.OffsetLow = addr as u16;
    entry.OffsetMiddle = (addr >> 16) as u16;
    entry.OffsetHigh = (addr >> 32) as u32;
}

// ============================================================================
// GDT helpers (FORCEINLINE from internal/amd64/intrin_i.h)
// ============================================================================

/// KiGetGdtEntry — get GDT entry for a given selector.
#[inline]
pub unsafe fn KiGetGdtEntry(gdt: PVOID, selector: u16) -> *mut ddk::_KGDTENTRY64 {
    (gdt as *mut u8).add((selector & !7u16) as usize) as *mut ddk::_KGDTENTRY64
}

/// KiGetGdtDescriptorBase — extract base address from GDT entry.
#[inline]
pub unsafe fn KiGetGdtDescriptorBase(entry: *mut ddk::_KGDTENTRY64) -> PVOID {
    let e = &(*entry).__bindgen_anon_1;
    let base_mid = e.__bindgen_anon_1.Bytes.BaseMiddle as u64;
    let base_hi = e.__bindgen_anon_1.Bytes.BaseHigh as u64;
    let base_upper = e.BaseUpper as u64;
    let base = (e.BaseLow as u64)
        | (base_mid << 16)
        | (base_hi << 24)
        | (base_upper << 32);
    base as PVOID
}

/// KiInitGdtEntry — initialize a GDT64 entry.
#[inline]
pub unsafe fn KiInitGdtEntry(
    entry: *mut ddk::_KGDTENTRY64,
    base: u64,
    size: ULONG,
    type_: u8,
    dpl: u8,
) {
    let e = &mut (*entry).__bindgen_anon_1;
    e.BaseLow = base as u16;
    e.__bindgen_anon_1.Bytes.BaseMiddle = (base >> 16) as u8;
    e.__bindgen_anon_1.Bytes.BaseHigh = (base >> 24) as u8;
    e.BaseUpper = (base >> 32) as u32;
    e.LimitLow = size as u16;
    // Set Bits: Type, Dpl, Present, LimitHi, Granularity
    e.__bindgen_anon_1.Bits.set_Type(type_ as u32);
    e.__bindgen_anon_1.Bits.set_Dpl(dpl as u32);
    e.__bindgen_anon_1.Bits.set_Present(1);
    e.__bindgen_anon_1.Bits.set_LimitHigh(((size >> 16) & 0xF) as u32);
    e.__bindgen_anon_1.Bits.set_Granularity(0);
    e.__bindgen_anon_1.Bits.set_System(0);
    e.__bindgen_anon_1.Bits.set_LongMode(0);
    e.__bindgen_anon_1.Bits.set_DefaultBig(0);
    e.MustBeZero = 0;
}

/// KiSetGdtDescriptorBase — set base address in GDT entry.
#[inline]
pub unsafe fn KiSetGdtDescriptorBase(entry: *mut ddk::_KGDTENTRY64, base: u64) {
    let e = &mut (*entry).__bindgen_anon_1;
    e.BaseLow = base as u16;
    e.__bindgen_anon_1.Bytes.BaseMiddle = (base >> 16) as u8;
    e.__bindgen_anon_1.Bytes.BaseHigh = (base >> 24) as u8;
    e.BaseUpper = (base >> 32) as u32;
}

/// KiSetGdtDescriptorLimit — set limit in GDT entry.
#[inline]
pub unsafe fn KiSetGdtDescriptorLimit(entry: *mut ddk::_KGDTENTRY64, limit: ULONG) {
    let e = &mut (*entry).__bindgen_anon_1;
    e.LimitLow = limit as u16;
    e.__bindgen_anon_1.Bits.set_LimitHigh(((limit >> 16) & 0xF) as u32);
}

// ============================================================================
// XState save/restore (FORCEINLINE from internal/amd64/intrin_i.h)
// ============================================================================

/// KiSaveXState — save extended state (xsave or fxsave).
#[inline]
pub unsafe fn KiSaveXState(buffer: PVOID, component_mask: u64) {
    if component_mask != 0 {
        asm!(
            "xsave [{}]",
            in(reg) buffer,
            in("eax") component_mask as u32,
            in("edx") (component_mask >> 32) as u32,
            options(nostack),
        );
    } else {
        asm!("fxsave [{}]", in(reg) buffer, options(nostack));
    }
}

/// KiRestoreXState — restore extended state (xrstor or fxrstor).
#[inline]
pub unsafe fn KiRestoreXState(buffer: PVOID, component_mask: u64) {
    if component_mask != 0 {
        asm!(
            "xrstor [{}]",
            in(reg) buffer,
            in("eax") component_mask as u32,
            in("edx") (component_mask >> 32) as u32,
            options(nostack),
        );
    } else {
        asm!("fxrstor [{}]", in(reg) buffer, options(nostack));
    }
}

// ============================================================================
// APIC EOI (FORCEINLINE from internal/amd64/ke.h)
// ============================================================================

const APIC_BASE: u64 = 0xFFFFFFFFFFFE0000u64;
const APIC_EOI_REGISTER: u64 = 0xB0;

/// KiSendEOI — write 0 to APIC EOI register.
#[inline]
pub unsafe fn KiSendEOI() {
    let eoi = (APIC_BASE + APIC_EOI_REGISTER) as *mut u32;
    core::ptr::write_volatile(eoi, 0);
}

/// RtlFillMemoryUlong — fill memory with a ULONG pattern. An inline helper for
/// drivers (fastfat), NOT an export: the kernel's `RtlFillMemoryUlong` symbol is the
/// `mem` crate's.
#[inline]
pub unsafe fn RtlFillMemoryUlong(dest: PVOID, length: usize, pattern: ULONG) {
    let count = length / 4;
    asm!(
        "rep stosd",
        inout("rcx") count => _,
        inout("rdi") dest => _,
        in("eax") pattern,
        options(nostack),
    );
}

// ============================================================================
// File-scope asm blocks for compiler builtins
// ============================================================================

core::arch::global_asm!(
    // __cpuid(CPUInfo*, InfoType)
    // MS x64 ABI: rcx=CPUInfo, edx=InfoType
    ".globl __cpuid",
    "__cpuid:",
    "  push rbx",
    "  mov r9, rcx",
    "  mov eax, edx",
    "  xor ecx, ecx",
    "  cpuid",
    "  mov [r9], eax",
    "  mov [r9+4], ebx",
    "  mov [r9+8], ecx",
    "  mov [r9+12], edx",
    "  pop rbx",
    "  ret",

    // KiCpuId(CpuInfo*, Function) — identical body to __cpuid, this is the
    // NTAPI export the kernel uses (was a FORCEINLINE in internal/amd64/ke.h)
    ".globl KiCpuId",
    "KiCpuId:",
    "  push rbx",
    "  mov r9, rcx",
    "  mov eax, edx",
    "  xor ecx, ecx",
    "  cpuid",
    "  mov [r9], eax",
    "  mov [r9+4], ebx",
    "  mov [r9+8], ecx",
    "  mov [r9+12], edx",
    "  pop rbx",
    "  ret",

    // __cpuidex(CPUInfo*, InfoType, ECXValue)
    // MS x64 ABI: rcx=CPUInfo, edx=InfoType, r8d=ECXValue
    ".globl __cpuidex",
    "__cpuidex:",
    "  push rbx",
    "  mov r9, rcx",
    "  mov eax, edx",
    "  mov ecx, r8d",
    "  cpuid",
    "  mov [r9], eax",
    "  mov [r9+4], ebx",
    "  mov [r9+8], ecx",
    "  mov [r9+12], edx",
    "  pop rbx",
    "  ret",

    // _mm_getcsr() -> u32 in eax
    ".globl _mm_getcsr",
    "_mm_getcsr:",
    "  sub rsp, 4",
    "  stmxcsr [rsp]",
    "  mov eax, [rsp]",
    "  add rsp, 4",
    "  ret",

    // _mm_setcsr(val)
    // MS x64 ABI: ecx = val
    ".globl _mm_setcsr",
    "_mm_setcsr:",
    "  sub rsp, 4",
    "  mov [rsp], ecx",
    "  ldmxcsr [rsp]",
    "  add rsp, 4",
    "  ret",

    // _enable / _disable / __debugbreak
    ".globl _enable",
    "_enable:",
    "  sti",
    "  ret",

    ".globl _disable",
    "_disable:",
    "  cli",
    "  ret",

    ".globl __debugbreak",
    "__debugbreak:",
    "  int3",
    "  ret",

    // __addgsdword(offset, value)
    // MS x64 ABI: ecx=offset, edx=value
    ".globl __addgsdword",
    "__addgsdword:",
    "  add gs:[ecx], edx",
    "  ret",

    // _xsetbv(xcr, value)
    // MS x64 ABI: ecx=xcr, rdx=value
    ".globl _xsetbv",
    "_xsetbv:",
    "  mov rax, rdx",
    "  shr rdx, 32",
    "  xsetbv",
    "  ret",

    // InterlockedBitTestAndSet64(Base, Bit)
    // MS x64 ABI: rcx=Base, rdx=Bit
    ".globl InterlockedBitTestAndSet64",
    "InterlockedBitTestAndSet64:",
    "  xor eax, eax",
    "  lock bts [rcx], rdx",
    "  setc al",
    "  ret",

    // InterlockedAnd64(Dest, Value)
    // MS x64 ABI: rcx=Dest, rdx=Value, returns original in rax
    ".globl InterlockedAnd64",
    "InterlockedAnd64:",
    "  mov rax, [rcx]",
    "1:",
    "  mov r8, rax",
    "  and r8, rdx",
    "  lock cmpxchg [rcx], r8",
    "  jnz 1b",
    "  ret",

    // __rdtsc -> u64 in rax
    ".globl __rdtsc",
    "__rdtsc:",
    "  rdtsc",
    "  shl rdx, 32",
    "  or rax, rdx",
    "  ret",
);
