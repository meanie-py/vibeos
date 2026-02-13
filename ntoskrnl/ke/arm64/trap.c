/*
 * PROJECT:         ReactOS Kernel
 * LICENSE:         GPL-2.0-or-later
 * FILE:            ntoskrnl/ke/arm64/trap.c
 * PURPOSE:         ARM64 Exception Handlers (Synchronous, IRQ, SError)
 */

/* INCLUDES ******************************************************************/

#include <ntoskrnl.h>
#define NDEBUG
#include <debug.h>

/* Verify KTRAP_FRAME offsets match assembly constants in trap.S */
_Static_assert(__builtin_offsetof(KTRAP_FRAME, ExceptionActive) == 0x00,  "TfExceptionActive");
_Static_assert(__builtin_offsetof(KTRAP_FRAME, PreviousMode)    == 0x03,  "TfPreviousMode");
_Static_assert(__builtin_offsetof(KTRAP_FRAME, PreviousIrql)    == 0x04,  "TfPreviousIrql");
_Static_assert(__builtin_offsetof(KTRAP_FRAME, FaultAddress)    == 0x10,  "TfFaultAddress");
_Static_assert(__builtin_offsetof(KTRAP_FRAME, Spsr)            == 0xA0,  "TfSpsr");
_Static_assert(__builtin_offsetof(KTRAP_FRAME, Esr)             == 0xA4,  "TfEsr");
_Static_assert(__builtin_offsetof(KTRAP_FRAME, Sp)              == 0xA8,  "TfSp");
_Static_assert(__builtin_offsetof(KTRAP_FRAME, X0)              == 0xB0,  "TfX0");
_Static_assert(__builtin_offsetof(KTRAP_FRAME, X18)             == 0x140, "TfX18");
_Static_assert(__builtin_offsetof(KTRAP_FRAME, Lr)              == 0x148, "TfLr");
_Static_assert(__builtin_offsetof(KTRAP_FRAME, Fp)              == 0x150, "TfFp");
_Static_assert(__builtin_offsetof(KTRAP_FRAME, Pc)              == 0x158, "TfPc");
_Static_assert(sizeof(KTRAP_FRAME)                              == 0x160, "KTRAP_FRAME_LENGTH");

/* UART HELPERS **************************************************************/

/* PL011 UART at QEMU virt address - identity mapped during boot */
#define PL011_BASE ((volatile ULONG *)0x09000000ULL)

static void TrapPutChar(char c)
{
    *PL011_BASE = (ULONG)c;
}

static void TrapPuts(const char *s)
{
    while (*s)
        TrapPutChar(*s++);
}

static void TrapPutHex4(ULONG nibble)
{
    TrapPutChar("0123456789ABCDEF"[nibble & 0xF]);
}

static void TrapPutHex32(ULONG val)
{
    int i;
    TrapPuts("0x");
    for (i = 28; i >= 0; i -= 4)
        TrapPutHex4(val >> i);
}

static void TrapPutHex64(ULONG64 val)
{
    int i;
    TrapPuts("0x");
    for (i = 60; i >= 0; i -= 4)
        TrapPutHex4((ULONG)(val >> i));
}

/* ESR exception class names */
static const char *KiGetExceptionClassName(ULONG EC)
{
    switch (EC)
    {
        case 0x00: return "Unknown";
        case 0x01: return "WFx trap";
        case 0x0E: return "Illegal Execution";
        case 0x15: return "SVC (AArch64)";
        case 0x18: return "MSR/MRS trap";
        case 0x20: return "Instruction Abort (lower EL)";
        case 0x21: return "Instruction Abort (same EL)";
        case 0x22: return "PC Alignment";
        case 0x24: return "Data Abort (lower EL)";
        case 0x25: return "Data Abort (same EL)";
        case 0x26: return "SP Alignment";
        case 0x2C: return "FP Exception";
        case 0x3C: return "BRK (AArch64)";
        default:   return "Other";
    }
}

static void KiDumpTrapFrame(PKTRAP_FRAME Tf)
{
    ULONG EC = (Tf->Esr >> 26) & 0x3F;

    TrapPuts("\r\n*** ARM64 EXCEPTION ***\r\n");
    TrapPuts("Type: ");
    TrapPuts(KiGetExceptionClassName(EC));
    TrapPuts(" (EC=");
    TrapPutHex32(EC);
    TrapPuts(")\r\n");

    TrapPuts("PC:   "); TrapPutHex64(Tf->Pc);   TrapPuts("\r\n");
    TrapPuts("LR:   "); TrapPutHex64(Tf->Lr);   TrapPuts("\r\n");
    TrapPuts("SP:   "); TrapPutHex64(Tf->Sp);   TrapPuts("\r\n");
    TrapPuts("FP:   "); TrapPutHex64(Tf->Fp);   TrapPuts("\r\n");
    TrapPuts("ESR:  "); TrapPutHex32(Tf->Esr);  TrapPuts("\r\n");
    TrapPuts("FAR:  "); TrapPutHex64(Tf->FaultAddress); TrapPuts("\r\n");
    TrapPuts("SPSR: "); TrapPutHex32(Tf->Spsr); TrapPuts("\r\n");

    TrapPuts("X0:   "); TrapPutHex64(Tf->X0);  TrapPuts("  X1:  "); TrapPutHex64(Tf->X1);  TrapPuts("\r\n");
    TrapPuts("X2:   "); TrapPutHex64(Tf->X2);  TrapPuts("  X3:  "); TrapPutHex64(Tf->X3);  TrapPuts("\r\n");
    TrapPuts("X4:   "); TrapPutHex64(Tf->X4);  TrapPuts("  X5:  "); TrapPutHex64(Tf->X5);  TrapPuts("\r\n");
    TrapPuts("X6:   "); TrapPutHex64(Tf->X6);  TrapPuts("  X7:  "); TrapPutHex64(Tf->X7);  TrapPuts("\r\n");
    TrapPuts("X8:   "); TrapPutHex64(Tf->X8);  TrapPuts("  X9:  "); TrapPutHex64(Tf->X9);  TrapPuts("\r\n");
    TrapPuts("X10:  "); TrapPutHex64(Tf->X10); TrapPuts("  X11: "); TrapPutHex64(Tf->X11); TrapPuts("\r\n");
    TrapPuts("X12:  "); TrapPutHex64(Tf->X12); TrapPuts("  X13: "); TrapPutHex64(Tf->X13); TrapPuts("\r\n");
    TrapPuts("X14:  "); TrapPutHex64(Tf->X14); TrapPuts("  X15: "); TrapPutHex64(Tf->X15); TrapPuts("\r\n");
    TrapPuts("X16:  "); TrapPutHex64(Tf->X16); TrapPuts("  X17: "); TrapPutHex64(Tf->X17); TrapPuts("\r\n");
    TrapPuts("X18:  "); TrapPutHex64(Tf->X18); TrapPuts("\r\n");
}

/* EXCEPTION HANDLERS ********************************************************/

VOID
KiSynchronousExceptionHandler(
    _Inout_ PKTRAP_FRAME TrapFrame)
{
    ULONG EC = (TrapFrame->Esr >> 26) & 0x3F;
    ULONG ISS = TrapFrame->Esr & 0x1FFFFFF;

    switch (EC)
    {
        case 0x25: /* Data Abort (same EL) */
        case 0x24: /* Data Abort (lower EL) */
        {
            ULONG DFSC = ISS & 0x3F;
            BOOLEAN IsWrite = (ISS >> 6) & 1;

            /* Translation fault (DFSC 0x04-0x07) — try MmAccessFault */
            if ((DFSC & 0x3C) == 0x04)
            {
                NTSTATUS Status;
                TrapPuts("[TRAP] Data Abort (translation fault): FAR=");
                TrapPutHex64(TrapFrame->FaultAddress);
                TrapPuts(IsWrite ? " WRITE" : " READ");
                TrapPuts(" PC=");
                TrapPutHex64(TrapFrame->Pc);
                TrapPuts("\r\n");

                Status = MmAccessFault(IsWrite ? 1 : 0,
                                       (PVOID)TrapFrame->FaultAddress,
                                       KernelMode,
                                       TrapFrame);
                if (NT_SUCCESS(Status))
                {
                    TrapPuts("[TRAP] MmAccessFault resolved\r\n");
                    return; /* Resume execution */
                }

                TrapPuts("[TRAP] MmAccessFault FAILED: ");
                TrapPutHex32((ULONG)Status);
                TrapPuts("\r\n");
            }

            /* Fall through to dump + bugcheck for unhandled data aborts */
            KiDumpTrapFrame(TrapFrame);
            KeBugCheckEx(KMODE_EXCEPTION_NOT_HANDLED,
                         TrapFrame->Esr,
                         TrapFrame->FaultAddress,
                         TrapFrame->Pc,
                         0);
            break;
        }

        case 0x21: /* Instruction Abort (same EL) */
        case 0x20: /* Instruction Abort (lower EL) */
        {
            TrapPuts("[TRAP] Instruction Abort: FAR=");
            TrapPutHex64(TrapFrame->FaultAddress);
            TrapPuts(" PC=");
            TrapPutHex64(TrapFrame->Pc);
            TrapPuts("\r\n");
            KiDumpTrapFrame(TrapFrame);
            KeBugCheckEx(KMODE_EXCEPTION_NOT_HANDLED,
                         TrapFrame->Esr,
                         TrapFrame->FaultAddress,
                         TrapFrame->Pc,
                         1);
            break;
        }

        case 0x3C: /* BRK (software breakpoint) */
        {
            USHORT BrkImm = (USHORT)(ISS & 0xFFFF);
            TrapPuts("[BRK #");
            TrapPutHex32(BrkImm);
            TrapPuts("] PC=");
            TrapPutHex64(TrapFrame->Pc);
            TrapPuts("\r\n");

            /* Skip past the BRK instruction (4 bytes) */
            TrapFrame->Pc += 4;
            return;
        }

        case 0x15: /* SVC (system call) */
        {
            /* Stub — no user-mode syscalls yet */
            return;
        }

        case 0x26: /* SP Alignment fault */
        {
            TrapPuts("[TRAP] SP Alignment Fault\r\n");
            KiDumpTrapFrame(TrapFrame);
            KeBugCheckEx(KMODE_EXCEPTION_NOT_HANDLED,
                         TrapFrame->Esr,
                         TrapFrame->Sp,
                         TrapFrame->Pc,
                         2);
            break;
        }

        default:
        {
            KiDumpTrapFrame(TrapFrame);
            KeBugCheckEx(KMODE_EXCEPTION_NOT_HANDLED,
                         TrapFrame->Esr,
                         TrapFrame->FaultAddress,
                         TrapFrame->Pc,
                         3);
            break;
        }
    }
}

VOID
KiIrqHandler(
    _Inout_ PKTRAP_FRAME TrapFrame)
{
    TrapPuts("[IRQ] PC=");
    TrapPutHex64(TrapFrame->Pc);
    TrapPuts("\r\n");
    /* No GIC configured yet — just return */
}

VOID
KiSErrorHandler(
    _Inout_ PKTRAP_FRAME TrapFrame)
{
    TrapPuts("[SError] FATAL\r\n");
    KiDumpTrapFrame(TrapFrame);
    KeBugCheckEx(NMI_HARDWARE_FAILURE,
                 TrapFrame->Esr,
                 TrapFrame->FaultAddress,
                 TrapFrame->Pc,
                 0);
}

VOID
KiUnexpectedHandler(
    _Inout_ PKTRAP_FRAME TrapFrame)
{
    TrapPuts("[UNEXPECTED EXCEPTION]\r\n");
    KiDumpTrapFrame(TrapFrame);
    KeBugCheckEx(UNEXPECTED_KERNEL_MODE_TRAP,
                 TrapFrame->Esr,
                 TrapFrame->FaultAddress,
                 TrapFrame->Pc,
                 0);
}

/* EOF */
