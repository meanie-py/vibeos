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

    // x87 FPU save (FNSAVE then WAIT) — Ke386FnSave(SaveArea)
    ".globl _Ke386FnSave@4",
    "_Ke386FnSave@4:",
    "  mov eax, [esp+4]",
    "  fnsave [eax]",
    "  fwait",
    "  ret 4",

    // x87 + SSE save (FXSAVE) — Ke386FxSave(SaveArea)
    ".globl _Ke386FxSave@4",
    "_Ke386FxSave@4:",
    "  mov eax, [esp+4]",
    "  fxsave [eax]",
    "  ret 4",

    // x87 + SSE restore with MXCSR sanitization (FXRSTOR) — Ke386FxStore(SaveArea)
    // Mirrors C: mask = mxcsr_mask ? mxcsr_mask : 0xFFBF; *mxcsr &= mask
    ".globl _Ke386FxStore@4",
    "_Ke386FxStore@4:",
    "  mov eax, [esp+4]",
    "  mov edx, [eax+28]",
    "  test edx, edx",
    "  jnz 1f",
    "  mov edx, 0xFFBF",
    "1:",
    "  and [eax+24], edx",
    "  fxrstor [eax]",
    "  ret 4",

    // Control register write — Ke386SetCr2(Value)
    ".globl _Ke386SetCr2@4",
    "_Ke386SetCr2@4:",
    "  mov eax, [esp+4]",
    "  mov cr2, eax",
    "  ret 4",

    // Segment register loads (DS / ES / FS / GS) — full 32-bit slot, low 16 used
    ".globl _Ke386SetDs@4",
    "_Ke386SetDs@4:",
    "  mov eax, [esp+4]",
    "  mov ds, ax",
    "  ret 4",

    ".globl _Ke386SetEs@4",
    "_Ke386SetEs@4:",
    "  mov eax, [esp+4]",
    "  mov es, ax",
    "  ret 4",

    ".globl _Ke386SetFs@4",
    "_Ke386SetFs@4:",
    "  mov eax, [esp+4]",
    "  mov fs, ax",
    "  ret 4",

    ".globl _Ke386SetGs@4",
    "_Ke386SetGs@4:",
    "  mov eax, [esp+4]",
    "  mov gs, ax",
    "  ret 4",

    // GDT/LDT/TR loads
    ".globl _Ke386SetGlobalDescriptorTable@4",
    "_Ke386SetGlobalDescriptorTable@4:",
    "  mov eax, [esp+4]",
    "  lgdt [eax]",
    "  ret 4",

    ".globl _Ke386SetLocalDescriptorTable@4",
    "_Ke386SetLocalDescriptorTable@4:",
    "  mov eax, [esp+4]",
    "  lldt ax",
    "  ret 4",

    ".globl _Ke386SetTr@4",
    "_Ke386SetTr@4:",
    "  mov eax, [esp+4]",
    "  ltr ax",
    "  ret 4",

    // Get TR register — Ke386GetTr() -> USHORT
    ".globl _Ke386GetTr@0",
    "_Ke386GetTr@0:",
    "  xor eax, eax",
    "  str ax",
    "  ret",

    // TLB entry invalidate — KeInvalidateTlbEntry(Address) (cdecl, used as
    // extern "C" by the Rust ARM3 paths)
    ".globl _KeInvalidateTlbEntry",
    "_KeInvalidateTlbEntry:",
    "  mov eax, [esp+4]",
    "  invlpg [eax]",
    "  ret",

    // KiCpuId(CpuInfo*, Function) — i386 NTAPI/stdcall, fills 4-ULONG buffer
    ".globl _KiCpuId@8",
    "_KiCpuId@8:",
    "  push edi",
    "  push ebx",
    "  mov edi, [esp+12]",
    "  mov eax, [esp+16]",
    "  xor ecx, ecx",
    "  cpuid",
    "  mov [edi], eax",
    "  mov [edi+4], ebx",
    "  mov [edi+8], ecx",
    "  mov [edi+12], edx",
    "  pop ebx",
    "  pop edi",
    "  ret 8",

    // KeGetPcr — read PCR self-pointer at fs:[0x1C]. Was a #define macro
    // expanding to (PKPCR)__readfsdword(FIELD_OFFSET(KPCR, SelfPcr)).
    // i386 cdecl, 0 args.
    ".globl _KeGetPcr",
    "_KeGetPcr:",
    "  mov eax, fs:[0x1C]",
    "  ret",
);

// ============================================================================
// Pure-logic FORCEINLINEs from internal/i386/ke.h
// ============================================================================

/// Sanitize a segment register selector for the given processor mode.
/// Mirrors the FORCEINLINE in internal/i386/ke.h. KernelMode forces CPL 0,
/// UserMode forces CPL 3.
#[no_mangle]
pub unsafe extern "system" fn Ke386SanitizeSeg(Cs: ULONG, Mode: u8) -> ULONG {
    const RPL_MASK: ULONG = 0x3;
    const KERNEL_MODE: u8 = 0;
    if Mode == KERNEL_MODE {
        Cs & (0xFFFF & !RPL_MASK)
    } else {
        RPL_MASK | (Cs & 0xFFFF)
    }
}

/// `(TrapFrame->SegCs & MODE_MASK) != 0` — was a FORCEINLINE in trap_x.h.
/// Anything but Ring 0 is treated as user mode.
#[no_mangle]
pub unsafe extern "system" fn KiUserTrap(TrapFrame: *const ddk::_KTRAP_FRAME) -> u8 {
    if ((*TrapFrame).SegCs & 1) != 0 { 1 } else { 0 }
}

/// Sanitize EFLAGS for the given processor mode. KernelMode keeps the
/// permitted user bits + interrupt mask; UserMode forces interrupts on
/// and clamps to user-permitted bits.
#[no_mangle]
pub unsafe extern "system" fn Ke386SanitizeFlags(Eflags: ULONG, Mode: u8) -> ULONG {
    const EFLAGS_USER_SANITIZE: ULONG = 0x3F4DD7;
    const EFLAGS_INTERRUPT_MASK: ULONG = 0x200;
    const KERNEL_MODE: u8 = 0;
    if Mode == KERNEL_MODE {
        Eflags & (EFLAGS_USER_SANITIZE | EFLAGS_INTERRUPT_MASK)
    } else {
        EFLAGS_INTERRUPT_MASK | (Eflags & EFLAGS_USER_SANITIZE)
    }
}

/// `Thread->InitialStack - sizeof(FX_SAVE_AREA)` — was a FORCEINLINE in
/// internal/i386/ke.h. Returns the FxSaveArea pointer, which lives just
/// below InitialStack on the kernel stack.
#[no_mangle]
pub unsafe extern "system" fn KiGetThreadNpxArea(
    Thread: *const ddk::_KTHREAD,
) -> *mut ddk::_FX_SAVE_AREA {
    ((*Thread).InitialStack as *mut u8).sub(core::mem::size_of::<ddk::_FX_SAVE_AREA>())
        as *mut ddk::_FX_SAVE_AREA
}

/// Update the per-CPU PCR's self-TEB pointer and patch the GDT TEB entry's
/// base address. Was a FORCEINLINE in internal/i386/ke.h.
///
/// `Pcr->NtTib.Self = TebAddress;`
/// `Ke386SetGdtEntryBase(&Pcr->GDT[KGDT_R3_TEB / sizeof(KGDTENTRY)], TebAddress);`
///
/// The GDT R3 TEB descriptor is at selector 0x38; sizeof(KGDTENTRY)=8 on
/// i386, so the descriptor index is 7.
#[no_mangle]
pub unsafe extern "system" fn KiSetTebBase(
    Pcr: *mut ddk::_KPCR,
    TebAddress: *mut ddk::_NT_TIB,
) {
    (*Pcr).__bindgen_anon_1.NtTib.Self_ = TebAddress;
    let gdt_entry = (*Pcr).GDT.add(7); // 0x38 / sizeof(KGDTENTRY)=8
    KiSetGdtDescriptorBase(gdt_entry, TebAddress as ULONG_PTR);
}

/// Stamp the trap frame's debug fields. Was a FORCEINLINE in trap_x.h.
/// `DbgArgMark = 0xBADB0D00` is the magic that KiExitTrapDebugChecks
/// later validates to confirm the trap frame is real.
#[no_mangle]
pub unsafe extern "system" fn KiFillTrapFrameDebug(TrapFrame: *mut ddk::_KTRAP_FRAME) {
    (*TrapFrame).DbgArgPointer = (*TrapFrame).Edx;
    (*TrapFrame).DbgArgMark = 0xBADB0D00;
    (*TrapFrame).DbgEip = (*TrapFrame).Eip;
    (*TrapFrame).DbgEbp = (*TrapFrame).Ebp;
    (*TrapFrame).PreviousPreviousMode = 0xFFFFFFFF; // (ULONG)-1
}


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
