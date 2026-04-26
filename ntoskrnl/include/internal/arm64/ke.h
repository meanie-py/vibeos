/*
 *  ReactOS kernel
 *  Copyright (C) 1998, 1999, 2000, 2001 ReactOS Team
 *
 *  This program is free software; you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation; either version 2 of the License, or
 *  (at your option) any later version.
 *
 *  This program is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License along
 *  with this program; if not, write to the Free Software Foundation, Inc.,
 *  51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
 */

#ifndef __NTOSKRNL_INCLUDE_INTERNAL_ARM64_KE_H
#define __NTOSKRNL_INCLUDE_INTERNAL_ARM64_KE_H

#ifdef __cplusplus
extern "C" {
#endif

#include "intrin_i.h"

//
// KiExceptionExit - restores state from trap/exception frame and returns
// This will be implemented in assembly; for now declare it
//
DECLSPEC_NORETURN
VOID
KiExceptionExit(
    _In_ PKTRAP_FRAME TrapFrame,
    _In_ PKEXCEPTION_FRAME ExceptionFrame);

#define KiServiceExit2 KiExceptionExit

/* ARM64 uses same IRQL as ARM */
#define SYNCH_LEVEL DISPATCH_LEVEL

//
// Shared user data location (same as AMD64)
//
#define KI_USER_SHARED_DATA         0xFFFFF78000000000ULL
#define SharedInterruptTime         (KI_USER_SHARED_DATA + 0x8)
#define SharedSystemTime            (KI_USER_SHARED_DATA + 0x14)
#define SharedTickCount             (KI_USER_SHARED_DATA + 0x320)

//
// Query functions using shared user data
//
#define KeQueryInterruptTime() \
    (*(volatile ULONG64*)SharedInterruptTime)

#define KeQuerySystemTime(CurrentCount) \
    (*(ULONG64*)(CurrentCount) = *(volatile ULONG64*)SharedSystemTime)

#define KeQueryTickCount(CurrentCount) \
    (*(ULONG64*)(CurrentCount) = *(volatile ULONG64*)SharedTickCount)

//
// ARM64 yield instruction for spin-wait loops
//
#define YieldProcessor __yield

//
// BRK is 4 bytes long on ARM64
//
#define KD_BREAKPOINT_TYPE        ULONG
#define KD_BREAKPOINT_SIZE        sizeof(ULONG)
#define KD_BREAKPOINT_VALUE       0xD4200000  /* BRK #0 */

//
// Maximum IRQs - same as ARM for now
//
#define MAXIMUM_VECTOR          16

//
// Macros for getting and setting special purpose registers in portable code
//
#define KeGetContextPc(Context) \
    ((Context)->Pc)

#define KeSetContextPc(Context, ProgramCounter) \
    ((Context)->Pc = (ProgramCounter))

#define KeGetTrapFramePc(TrapFrame) \
    ((TrapFrame)->Pc)

#define KeGetContextReturnRegister(Context) \
    ((Context)->X0)

#define KeSetContextReturnRegister(Context, ReturnValue) \
    ((Context)->X0 = (ReturnValue))

#define KeGetContextStackRegister(Context) \
    ((Context)->Sp)

#define KeGetContextFrameRegister(Context) \
    ((Context)->Fp)

#define KeSetContextFrameRegister(Context, Frame) \
    ((Context)->Fp = (Frame))

//
// Macro to get trap and exception frame from a thread stack
//
#define KeGetTrapFrame(Thread) \
    (PKTRAP_FRAME)((ULONG_PTR)((Thread)->InitialStack) - \
                   sizeof(KTRAP_FRAME))

#define KeGetExceptionFrame(Thread) \
    (PKEXCEPTION_FRAME)((ULONG_PTR)KeGetTrapFrame(Thread) - \
                        sizeof(KEXCEPTION_FRAME))

//
// Macro to get context switches from the PRCB
// All architectures but x86 have it in the PRCB's KeContextSwitches
//
#define KeGetContextSwitches(Prcb)  \
    (Prcb)->KeContextSwitches

//
// Macro to get the second level cache size field name which differs between
// CISC and RISC architectures, as the former has unified I/D cache
//
#define KiGetSecondLevelDCacheSize() ((PKIPCR)KeGetPcr())->SecondLevelDcacheSize

//
// Returns the Interrupt State from a Trap Frame.
// ON = TRUE, OFF = FALSE
//
#define KeGetTrapFrameInterruptState(TrapFrame) \
    (!((TrapFrame)->Spsr & 0x80))  /* Check PSTATE.DAIF.I bit */

FORCEINLINE
BOOLEAN
KeDisableInterrupts(VOID)
{
    ULONG64 Flags;

    /* Read DAIF and disable interrupts */
    Flags = __getReg(18);  /* DAIF register */
    _disable();

    /* Return whether IRQs were enabled (I bit clear) */
    return !(Flags & 0x80);
}

FORCEINLINE
VOID
KeRestoreInterrupts(BOOLEAN WereEnabled)
{
    if (WereEnabled) _enable();
}

//
// Invalidates the TLB entry for a specified address
//
FORCEINLINE
VOID
KeInvalidateTlbEntry(IN PVOID Address)
{
    /* ARM64: TLBI VALE1IS, Xt — invalidate by VA, last level, EL1, inner shareable.
     * The register holds VA >> 12 (the page-aligned virtual address shifted right). */
    ULONG64 TlbiArg = (ULONG64)Address >> 12;
    __asm__ __volatile__("tlbi vale1is, %0" : : "r"(TlbiArg) : "memory");
    __asm__ __volatile__("dsb ish" ::: "memory");
    __asm__ __volatile__("isb" ::: "memory");
}

FORCEINLINE
VOID
KeFlushProcessTb(VOID)
{
    /* ARM64: TLBI VMALLE1IS — invalidate all EL1 TLB entries, inner shareable */
    __asm__ __volatile__("tlbi vmalle1is" ::: "memory");
    __asm__ __volatile__("dsb ish" ::: "memory");
    __asm__ __volatile__("isb" ::: "memory");
}

FORCEINLINE
VOID
KeSweepICache(IN PVOID BaseAddress,
              IN SIZE_T FlushSize)
{
    /*
     * ARM64 has non-coherent I-cache and D-cache. After writing code to
     * memory (stores go to D-cache), we must:
     * 1. Clean D-cache to Point of Unification (DC CVAU)
     * 2. DSB to ensure clean completes
     * 3. Invalidate I-cache (IC IALLU for simplicity)
     * 4. DSB to ensure invalidation completes
     * 5. ISB to flush pipeline
     *
     * Without this, newly written code (e.g. ATL thunks, JIT) executes
     * stale I-cache contents → STATUS_ILLEGAL_INSTRUCTION.
     */
    if (BaseAddress != NULL && FlushSize > 0)
    {
        /* Clean D-cache by VA to PoU for each cache line (64 bytes) */
        ULONG_PTR Addr = (ULONG_PTR)BaseAddress & ~(ULONG_PTR)63;
        ULONG_PTR End = ((ULONG_PTR)BaseAddress + FlushSize + 63) & ~(ULONG_PTR)63;
        for (; Addr < End; Addr += 64)
            __asm__ volatile("dc cvau, %0" :: "r"(Addr));
    }

    __dsb(_ARM64_BARRIER_ISH);
    __asm__ volatile("ic iallu");
    __dsb(_ARM64_BARRIER_ISH);
    __isb(_ARM64_BARRIER_SY);
}

FORCEINLINE
VOID
KiRundownThread(IN PKTHREAD Thread)
{
    /* Nothing to do on ARM64 */
    UNREFERENCED_PARAMETER(Thread);
}

VOID
KiPassiveRelease(
    VOID
);

VOID
KiSystemService(IN PKTHREAD Thread,
                IN PKTRAP_FRAME TrapFrame,
                IN ULONG Instruction);

VOID
KiApcInterrupt(
    VOID
);

#include "mm.h"

VOID
KeFlushTb(
    VOID
);

#define Ki386PerfEnd()
#define KiEndInterrupt(x,y)

#define KiGetLinkedTrapFrame(x) \
    (PKTRAP_FRAME)((x)->TrapFrame)

/* ARM64 mode check - EL0 is user mode */
#define KiGetPreviousMode(tf) \
    (((tf)->Spsr & 0x0F) == 0) ? UserMode : KernelMode

// HACK
extern NTKERNELAPI volatile KSYSTEM_TIME KeTickCount;

VOID
KeArm64InitThreadWithContext(PKTHREAD Thread,
                              PKSYSTEM_ROUTINE SystemRoutine,
                              PKSTART_ROUTINE StartRoutine,
                              PVOID StartContext,
                              PCONTEXT Context);

#define KeArchInitThreadWithContext(Thread,SystemRoutine,StartRoutine,StartContext,Context) \
    KeArm64InitThreadWithContext(Thread,SystemRoutine,StartRoutine,StartContext,Context)

#ifdef _NTOSKRNL_
/* ARM64 KiThreadStartup takes no parameters - it reads KSTART_FRAME from the stack */
VOID KiThreadStartup(VOID);
#endif

FORCEINLINE
PULONG_PTR
KiGetUserModeStackAddress(void)
{
    return &PsGetCurrentThread()->Tcb.TrapFrame->Sp;
}

/*
 * ARM64 UART debug trace — writes to PL011 UART via Arm64UartKernelBase.
 * Initially identity-mapped at 0x09000000 (works during early boot in System
 * process). Remapped to kernel VA via MmMapIoSpace in Phase 1 init so traces
 * work from any process context.
 *
 * Toggle with ARM64_UART_TRACE_ENABLE (define before including this header,
 * or pass -DARM64_UART_TRACE_ENABLE via cmake).
 * Each trace acts as a full memory barrier (device-memory MMIO stores).
 */
extern volatile PVOID Arm64UartKernelBase;
/* Uncomment to enable UART traces (also acts as memory barriers) */
/* #define ARM64_UART_TRACE_ENABLE */
#ifdef ARM64_UART_TRACE_ENABLE
#define ARM64_UART_TRACE(s) do { \
    volatile ULONG *_uart = (volatile ULONG *)Arm64UartKernelBase; \
    const char *_str = (s); \
    if (!_uart) break; \
    while (*_str) { *_uart = (ULONG)*_str++; } \
} while(0)
#define ARM64_UART_TRACE_HEX(prefix, val) do { \
    volatile ULONG *_uart = (volatile ULONG *)Arm64UartKernelBase; \
    const char *_str = (prefix); \
    ULONG_PTR _v = (ULONG_PTR)(val); \
    int _i; \
    if (!_uart) break; \
    while (*_str) { *_uart = (ULONG)*_str++; } \
    *_uart = '0'; *_uart = 'x'; \
    for (_i = 60; _i >= 0; _i -= 4) { \
        int _d = (_v >> _i) & 0xF; \
        *_uart = (ULONG)(_d < 10 ? '0' + _d : 'A' + _d - 10); \
    } \
    *_uart = '\r'; *_uart = '\n'; \
} while(0)
#else
#define ARM64_UART_TRACE(s) ((void)0)
#define ARM64_UART_TRACE_HEX(prefix, val) ((void)0)
#endif

/* ARM64 full memory barrier — use where UART traces masked timing bugs */
#define ARM64_BARRIER() __asm__ __volatile__("dsb sy\nisb" ::: "memory")

/* ARM64 QEMU TCG barrier — performs a minimal MMIO write to force QEMU's JIT
 * to break the translation block. Required around indirect function calls
 * through pointers loaded from freshly-allocated memory, where QEMU TCG
 * generates incorrect code without the TB break. */
#define ARM64_MMIO_BARRIER() do { \
    volatile ULONG *_u = (volatile ULONG *)Arm64UartKernelBase; \
    if (_u) *_u = '.'; \
    __asm__ __volatile__("dsb sy\nisb" ::: "memory"); \
} while(0)

#ifdef __cplusplus
} // extern "C"
#endif

#endif /* __NTOSKRNL_INCLUDE_INTERNAL_ARM64_KE_H */

/* EOF */
