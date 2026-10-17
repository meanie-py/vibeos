//! KE subsystem i386-specific inline functions.
//!
//! Compiler intrinsics and FORCEINLINE functions from intrin_i.h,
//! internal/i386/ke.h, and related headers.

#![cfg(target_arch = "x86")]

use crate::base::*;
use crate::ddk;
use core::arch::asm;

/// ndk/i386/ketypes.h: the Requested Privilege Level bits of a segment selector.
/// Module-level so both users share it — a per-function copy is how one of them
/// drifted to 0x7 (see `KiGetGdtEntry`).
const RPL_MASK: ULONG = 0x3;

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

    // ___debugbreak@0 — STDCALL-decorated ONLY, and that is deliberate.
    //
    // Consumers declare __debugbreak `extern "C"`, so on i386 they ask the linker
    // for the CDECL name `___debugbreak`. It is tempting to "fix" the mismatch by
    // also emitting `.globl ___debugbreak` here. DO NOT — it was tried, and it
    // makes driver links worse, not better.
    //
    // This whole `global_asm!` is ONE object, and further down it carries the
    // kernel-only trap trampolines (`call _KiDispatchExceptionFromTrapFrame@32`,
    // `jmp @KiServiceExit2@4`, `jmp _KiSystemStartupBootStack@0`). Archive
    // semantics pull an object in only to satisfy a symbol it DEFINES. Because no
    // driver can resolve `___debugbreak` here, no driver drags this object — and
    // therefore never inherits its ntoskrnl-internal references. Adding the cdecl
    // label made ksecdd.sys pull the object and fail with four *new* undefined
    // kernel symbols instead of one.
    //
    // A driver that needs __debugbreak defines it locally as an inline `int3` (see
    // fltmgr-rust/src/glue.rs, scsiport-rust/src/inlines.rs, ksecdd-rust). An
    // inline body needs no symbol at all, so it cannot pull anything.
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

    // KfRaiseIrql stdcall wrapper → fastcall HAL.
    // MUST call + ret 4, NOT tail-jmp: the fastcall impl's `ret` cleans 0 stack
    // bytes, so a tail-jmp would return to the caller with the pushed arg still
    // on the stack (ESP-4). That imbalance crashed KiCheckForApcDelivery's
    // esp-relative loop → BUGCHECK 0x0A Eip=0x10c (field-manual §9.26).
    ".globl _KfRaiseIrql@4",
    "_KfRaiseIrql@4:",
    "  mov ecx, [esp+4]",
    "  call @KfRaiseIrql@4",
    "  ret 4",

    // KfLowerIrql stdcall wrapper → fastcall HAL (same ret-4 rule as above)
    ".globl _KfLowerIrql@4",
    "_KfLowerIrql@4:",
    "  mov ecx, [esp+4]",
    "  call @KfLowerIrql@4",
    "  ret 4",

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

    // KeInvalidateTlbEntry is NOT here: ntoskrnl-rust's shim_exports.rs is its one
    // provider, and it also does the SMP remote shootdown. A local-only `invlpg`
    // label used to sit here under the same name; which one the kernel got was
    // decided by archive order alone.

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

    // KiHandleDebugRegistersOnTrapEntry(TrapFrame) — was a FORCEINLINE in
    // trap_x.h. Saves DR0–7 into the trap frame, disables active debugging,
    // then loads the kernel DR values from the PRCB. NTAPI/stdcall.
    //
    // Field offsets (from sdk/include/asm/ks386.inc):
    //   KTRAP_FRAME.Dr0..Dr7   = 0x18, 0x1C, 0x20, 0x24, 0x28, 0x2C
    //   KPCR.Prcb              = 0x20  (i.e. fs:[0x20] is the PRCB pointer)
    //   PRCB.ProcessorState    = 0x1C
    //   KPROCESSOR_STATE.SpecialRegisters = 0x2CC
    //   KSPECIAL_REGISTERS.KernelDr0..7 = 0x10, 0x14, 0x18, 0x1C, 0x20, 0x24
    //   ⇒ PRCB.KernelDr0 lives at 0x1C + 0x2CC + 0x10 = 0x2F8
    ".globl _KiHandleDebugRegistersOnTrapEntry@4",
    "_KiHandleDebugRegistersOnTrapEntry@4:",
    "  mov ecx, [esp+4]",        // ecx = TrapFrame
    "  mov eax, dr0",
    "  mov [ecx+0x18], eax",
    "  mov eax, dr1",
    "  mov [ecx+0x1C], eax",
    "  mov eax, dr2",
    "  mov [ecx+0x20], eax",
    "  mov eax, dr3",
    "  mov [ecx+0x24], eax",
    "  mov eax, dr6",
    "  mov [ecx+0x28], eax",
    "  mov eax, dr7",
    "  mov [ecx+0x2C], eax",
    "  xor eax, eax",
    "  mov dr7, eax",            // disable all active debugging first
    "  mov eax, fs:[0x20]",      // eax = current PRCB
    "  mov edx, [eax+0x2F8]",    // KernelDr0
    "  mov dr0, edx",
    "  mov edx, [eax+0x2FC]",    // KernelDr1
    "  mov dr1, edx",
    "  mov edx, [eax+0x300]",    // KernelDr2
    "  mov dr2, edx",
    "  mov edx, [eax+0x304]",    // KernelDr3
    "  mov dr3, edx",
    "  mov edx, [eax+0x308]",    // KernelDr6
    "  mov dr6, edx",
    "  mov edx, [eax+0x30C]",    // KernelDr7 (last so it can re-arm BPs)
    "  mov dr7, edx",
    "  ret 4",

    // KiHandleDebugRegistersOnTrapExit(TrapFrame) — disable then restore
    // DR0–7 from the trap frame.
    ".globl _KiHandleDebugRegistersOnTrapExit@4",
    "_KiHandleDebugRegistersOnTrapExit@4:",
    "  mov ecx, [esp+4]",        // ecx = TrapFrame
    "  xor eax, eax",
    "  mov dr7, eax",            // disable first
    "  mov eax, [ecx+0x18]",
    "  mov dr0, eax",
    "  mov eax, [ecx+0x1C]",
    "  mov dr1, eax",
    "  mov eax, [ecx+0x20]",
    "  mov dr2, eax",
    "  mov eax, [ecx+0x24]",
    "  mov dr3, eax",
    "  mov eax, [ecx+0x28]",
    "  mov dr6, eax",
    "  mov eax, [ecx+0x2C]",
    "  mov dr7, eax",
    "  ret 4",

    // KiEnterTrap(TrapFrame) — was a FORCEINLINE in trap_x.h. FASTCALL, so
    // arg is in ECX. Body:
    //   TrapFrame->Dr7 = 0;
    //   if (KiUserTrap(TrapFrame) || (EFlags & V86_MASK))
    //       if (CurrentThread->Header.DebugActive)
    //           KiHandleDebugRegistersOnTrapEntry(TrapFrame);
    //   KiFillTrapFrameDebug(TrapFrame);
    //
    // Field offsets: SegCs=0x6C, EFlags=0x70, Dr7=0x2C, KPCR.CurrentThread=0x124,
    // KTHREAD.Header.DebugActive=0x3.
    ".globl @KiEnterTrap@4",
    "@KiEnterTrap@4:",
    "  mov dword ptr [ecx+0x2C], 0",
    "  push esi",
    "  mov esi, ecx",
    "  test byte ptr [esi+0x6C], 1",
    "  jnz 1f",
    "  test dword ptr [esi+0x70], 0x20000",
    "  jz 2f",
    "1:",
    "  mov eax, fs:[0x124]",
    "  test byte ptr [eax+3], 0xFF",
    "  jz 2f",
    "  push esi",
    "  call _KiHandleDebugRegistersOnTrapEntry@4",
    "2:",
    "  push esi",
    "  call _KiFillTrapFrameDebug@4",
    "  pop esi",
    "  ret",

    // KiEnterInterruptTrap(TrapFrame) — was a FORCEINLINE in trap_x.h.
    // Same shape as KiEnterTrap, but additionally terminates the SEH
    // exception list (PCR.NtTib.ExceptionList = EXCEPTION_CHAIN_END = -1).
    // cdecl on i386 (extern "C" in irqobj.rs).
    ".globl _KiEnterInterruptTrap",
    "_KiEnterInterruptTrap:",
    "  mov ecx, [esp+4]",
    "  mov dword ptr fs:[0], 0xFFFFFFFF",  // ExceptionList = EXCEPTION_CHAIN_END
    "  mov dword ptr [ecx+0x2C], 0",       // Dr7 = 0
    "  push esi",
    "  mov esi, ecx",
    "  test byte ptr [esi+0x6C], 1",
    "  jnz 1f",
    "  test dword ptr [esi+0x70], 0x20000",
    "  jz 2f",
    "1:",
    "  mov eax, fs:[0x124]",
    "  test byte ptr [eax+3], 0xFF",
    "  jz 2f",
    "  push esi",
    "  call _KiHandleDebugRegistersOnTrapEntry@4",
    "2:",
    "  push esi",
    "  call _KiFillTrapFrameDebug@4",
    "  pop esi",
    "  ret",                                // cdecl: caller cleans

    // KiEnterV86Trap(TrapFrame) — was a FORCEINLINE in trap_x.h. FASTCALL.
    //   TrapFrame->Dr7 = __readdr(7);
    //   if (Dr7 & ~DR7_RESERVED_MASK)  // any non-reserved bit set
    //       KiHandleDebugRegistersOnTrapEntry(TrapFrame);
    // DR7_RESERVED_MASK = 0xDC00, so the inverse mask is 0xFFFF23FF.
    ".globl @KiEnterV86Trap@4",
    "@KiEnterV86Trap@4:",
    "  mov eax, dr7",
    "  mov [ecx+0x2C], eax",
    "  test eax, 0xFFFF23FF",
    "  jz 1f",
    "  push ecx",
    "  call _KiHandleDebugRegistersOnTrapEntry@4",
    "1:",
    "  ret",                                // FASTCALL: no stack args

    // KiDispatchException{0,1,2}Args — DECLSPEC_NORETURN FASTCALL helpers
    // that delegate to KiDispatchExceptionFromTrapFrame (NTAPI, 8 args).
    // Per FORCEINLINE in internal/i386/ke.h:
    //   0Args: KiDEFTF(Code, 0, Address, 0,        0,  0,  0, TrapFrame)
    //   1Args: KiDEFTF(Code, 0, Address, 1,        P1, 0,  0, TrapFrame)
    //   2Args: KiDEFTF(Code, 0, Address, 2,        P1, P2, 0, TrapFrame)
    //
    // Fastcall: ecx=Code, edx=Address, [esp+4]=P1/TrapFrame depending on N.
    // Stdcall callee KiDispatchExceptionFromTrapFrame@32 cleans 32 bytes.
    // None of these return (NORETURN), so no caller-side cleanup matters.
    ".globl @KiDispatchException0Args@12",
    "@KiDispatchException0Args@12:",
    "  push dword ptr [esp+4]",   // TrapFrame
    "  push 0",                    // P3
    "  push 0",                    // P2
    "  push 0",                    // P1
    "  push 0",                    // NumberParameters
    "  push edx",                  // Address
    "  push 0",                    // ExceptionRecord
    "  push ecx",                  // ExceptionCode
    "  call _KiDispatchExceptionFromTrapFrame@32",

    ".globl @KiDispatchException1Args@16",
    "@KiDispatchException1Args@16:",
    "  push dword ptr [esp+8]",   // TrapFrame (orig [esp+8])
    "  push 0",                    // P3
    "  push 0",                    // P2
    "  push dword ptr [esp+16]",  // P1 (orig [esp+4], 3 pushes done = +12)
    "  push 1",                    // NumberParameters
    "  push edx",                  // Address
    "  push 0",                    // ExceptionRecord
    "  push ecx",                  // ExceptionCode
    "  call _KiDispatchExceptionFromTrapFrame@32",

    ".globl @KiDispatchException2Args@20",
    "@KiDispatchException2Args@20:",
    "  push dword ptr [esp+12]",  // TrapFrame (orig [esp+12])
    "  push 0",                    // P3
    "  push dword ptr [esp+16]",  // P2 (orig [esp+8], 2 pushes done = +8 → +16)
    "  push dword ptr [esp+16]",  // P1 (orig [esp+4], 3 pushes done = +12 → +16)
    "  push 2",                    // NumberParameters
    "  push edx",                  // Address
    "  push 0",                    // ExceptionRecord
    "  push ecx",                  // ExceptionCode
    "  call _KiDispatchExceptionFromTrapFrame@32",

    // KiExceptionExit(TrapFrame, ExceptionFrame) — cdecl, NORETURN. On i386
    // ExceptionFrame is unused; body just tail-calls KiServiceExit2.
    ".globl _KiExceptionExit",
    "_KiExceptionExit:",
    "  mov ecx, [esp+4]",          // TrapFrame → ecx (fastcall arg)
    "  jmp @KiServiceExit2@4",

    // KiSwitchToBootStack(InitialStack) — NTAPI/stdcall. Switches ESP and
    // jumps into KiSystemStartupBootStack. Body from internal/i386/ke.h:
    //   esp = InitialStack;
    //   esp -= NPX_FRAME_LENGTH + KTRAP_FRAME_ALIGN + KTRAP_FRAME_LENGTH;
    //   push CR0_EM | CR0_TS | CR0_MP;
    //   jmp KiSystemStartupBootStack;
    //
    // Constants: 0x210 + 0x4 + 0x8C = 0x2A0 ; 0x4|0x8|0x2 = 0xE
    ".globl _KiSwitchToBootStack@4",
    "_KiSwitchToBootStack@4:",
    "  mov esp, [esp+4]",
    "  sub esp, 0x2A0",
    "  push 0xE",
    "  jmp _KiSystemStartupBootStack@0",

    // KiCheckForApcDelivery(TrapFrame) — was a FORCEINLINE in
    // internal/i386/ke.h. NTAPI/stdcall. Body:
    //   if ((TrapFrame->EFlags & EFLAGS_V86_MASK) || KiUserTrap(TrapFrame))
    //     while (1) {
    //       Thread->Alerted[KernelMode] = 0;
    //       if (!Thread->ApcState.UserApcPending) break;
    //       OldIrql = KfRaiseIrql(APC_LEVEL);  // APC_LEVEL = 1
    //       sti;
    //       KiDeliverApc(UserMode, NULL, TrapFrame);  // UserMode = 1
    //       KfLowerIrql(OldIrql);
    //       cli;
    //     }
    //
    // Field offsets: TrapFrame.SegCs=0x6C, EFlags=0x70; KTHREAD.Alerted[0]=0x5E,
    // KTHREAD.ApcState.UserApcPending = ThApcState(0x28) + AsUserApcPending(0x16)
    // = 0x3E. KPCR.CurrentThread = fs:[0x124].
    //
    // Stack layout in the loop: we save TrapFrame in esi (callee-saved),
    // and OldIrql in ebx (callee-saved).
    ".globl _KiCheckForApcDelivery@4",
    "_KiCheckForApcDelivery@4:",
    "  mov ecx, [esp+4]",                  // ecx = TrapFrame
    "  test byte ptr [ecx+0x6C], 1",       // user mode?
    "  jnz 1f",
    "  test dword ptr [ecx+0x70], 0x20000", // V86?
    "  jz 9f",                              // neither → return
    "1:",
    "  push esi",
    "  push edi",
    "  push ebx",
    "  mov esi, ecx",                      // esi = TrapFrame (preserved)
    "2:",                                   // top of loop
    "  mov edi, fs:[0x124]",                // edi = current thread
    "  mov byte ptr [edi+0x5E], 0",         // Thread.Alerted[KernelMode] = FALSE
    "  cmp byte ptr [edi+0x3E], 0",         // Thread.ApcState.UserApcPending
    "  je 8f",                              // 0 → break
    "  push 1",                             // APC_LEVEL = 1
    "  call _KfRaiseIrql@4",
    "  mov ebx, eax",                      // ebx = OldIrql (preserved across KiDeliverApc)
    "  sti",
    "  push esi",                          // TrapFrame
    "  push 0",                            // ExceptionFrame = NULL
    "  push 1",                            // UserMode = 1
    "  call _KiDeliverApc@12",
    "  push ebx",                          // OldIrql
    "  call _KfLowerIrql@4",
    "  cli",
    "  jmp 2b",
    "8:",
    "  pop ebx",
    "  pop edi",
    "  pop esi",
    "9:",
    "  ret 4",
);

// Compile-time guards: if bindgen layout drifts away from the hardcoded
// offsets above, fail the build instead of silently corrupting registers.
const _: () = assert!(core::mem::offset_of!(ddk::_KTRAP_FRAME, Dr0) == 0x18);
const _: () = assert!(core::mem::offset_of!(ddk::_KTRAP_FRAME, Dr7) == 0x2C);
const _: () = assert!(core::mem::offset_of!(ddk::_KPCR, Prcb) == 0x20);
const _: () = assert!(core::mem::offset_of!(ddk::_KPRCB, ProcessorState) == 0x1C);
const _: () = assert!(core::mem::offset_of!(ddk::_KPROCESSOR_STATE, SpecialRegisters) == 0x2CC);
const _: () = assert!(core::mem::offset_of!(ddk::_KSPECIAL_REGISTERS, KernelDr0) == 0x10);
const _: () = assert!(core::mem::offset_of!(ddk::_KSPECIAL_REGISTERS, KernelDr7) == 0x24);
const _: () = assert!(core::mem::offset_of!(ddk::_KTRAP_FRAME, SegCs) == 0x6C);
const _: () = assert!(core::mem::offset_of!(ddk::_KTRAP_FRAME, EFlags) == 0x70);

// ============================================================================
// Pure-logic FORCEINLINEs from internal/i386/ke.h
// ============================================================================

/// Sanitize a segment register selector for the given processor mode.
/// Mirrors the FORCEINLINE in internal/i386/ke.h. KernelMode forces CPL 0,
/// UserMode forces CPL 3.
#[no_mangle]
pub unsafe extern "system" fn Ke386SanitizeSeg(Cs: ULONG, Mode: u8) -> ULONG {
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
    crate::ASSERT!((*Thread).InitialStack as usize % 16 == 0);
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

extern "C" {
    fn KeGetPcr() -> *mut ddk::_KPCR;
}

extern "system" {
    fn HalVectorToIDTEntry(Vector: ULONG) -> u8;
}

/// Read the registered interrupt handler for an IDT vector. Was a
/// FORCEINLINE in internal/i386/ke.h. The PCR.IDT entry at index
/// `HalVectorToIDTEntry(Vector)` stores a 32-bit address split across
/// two 16-bit fields (`Offset` low, `ExtendedOffset` high).
#[no_mangle]
pub unsafe extern "system" fn KeQueryInterruptHandler(Vector: ULONG) -> PVOID {
    let pcr = KeGetPcr() as *mut ddk::_KIPCR;
    let entry = HalVectorToIDTEntry(Vector) as usize;
    let e = &*((*pcr).IDT).add(entry);
    let addr = ((e.ExtendedOffset as u32) << 16) | (e.Offset as u32);
    addr as PVOID
}

/// Patch the IDT entry for a given vector to dispatch to a new handler.
/// Was a FORCEINLINE in internal/i386/ke.h.
#[no_mangle]
pub unsafe extern "system" fn KeRegisterInterruptHandler(Vector: ULONG, Handler: PVOID) {
    let pcr = KeGetPcr() as *mut ddk::_KIPCR;
    let entry = HalVectorToIDTEntry(Vector) as usize;
    let e = &mut *((*pcr).IDT).add(entry);
    let addr = Handler as usize;
    e.ExtendedOffset = (addr >> 16) as u16;
    e.Offset = addr as u16;
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
    // RPL_MASK (0x3), not 0x7: the extra bit is TI, and masking it too would
    // silently alias an LDT selector onto the GDT entry of the same index.
    // Equivalent for every KGDT_* caller (all TI=0), wrong the moment one isn't.
    (gdt as *mut u8).add((selector & !(RPL_MASK as u16)) as usize) as *mut ddk::_KGDTENTRY
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
