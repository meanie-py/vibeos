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

/*
 * The GdiBatchCount the CURRENT process's gdi32 actually increments.
 *
 * For a native process that is TEB64's field. For a WoW64 guest it is
 * TEB32's, because the guest runs the 32-bit gdi32 against its own TEB; the
 * 32-bit TEB sits at TEB64 + 0x2000 in the combined WoW64 TEB region (see
 * ARM3/procsup.rs WOW64_TEB32_OFFSET and win32k-rust/src/wow64.rs, which use
 * the same constant).
 */
#define WOW64_TEB32_OFFSET 0x2000

PULONG KiWow64GdiBatchCount(PTEB Teb)
{
    PEPROCESS Process = PsGetCurrentProcess();

    if (Process != NULL && PsGetProcessWow64Process(Process) != NULL)
    {
        PTEB32 Teb32 = (PTEB32)((PUCHAR)Teb + WOW64_TEB32_OFFSET);
        return &Teb32->GdiBatchCount;
    }

    return &Teb->GdiBatchCount;
}

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

/* PL011 UART — use kernel VA from global (identity-mapped during early boot,
 * remapped to kernel VA in Phase 1 via MmMapIoSpace) */
extern volatile PVOID Arm64UartKernelBase;
#define PL011_BASE ((volatile ULONG *)Arm64UartKernelBase)

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

/* SVC SYSTEM CALL DISPATCHER ************************************************/

/*
 * Function pointer type for dispatching system calls with up to 15 arguments.
 * ARM64 passes args 1-8 in x0-x7, args 9+ on the stack.
 * The callee ignores extra args, so passing more than needed is safe.
 */
/* Return type must be ULONG_PTR (not NTSTATUS) because some Win32k syscalls
 * (e.g. NtUserSetWindowLongPtr, NtUserGetWindowLongPtr) return 64-bit values.
 * Using NTSTATUS (32-bit) truncates their return values. */
typedef ULONG_PTR (NTAPI *PSYSCALL_ROUTINE)(
    ULONG_PTR, ULONG_PTR, ULONG_PTR, ULONG_PTR,
    ULONG_PTR, ULONG_PTR, ULONG_PTR, ULONG_PTR,
    ULONG_PTR, ULONG_PTR, ULONG_PTR, ULONG_PTR,
    ULONG_PTR, ULONG_PTR, ULONG_PTR);

extern NTSTATUS NTAPI PsConvertToGuiThread(VOID);

static VOID
KiSvcDispatch(
    _Inout_ PKTRAP_FRAME TrapFrame)
{
    ULONG ServiceNumber = (ULONG)TrapFrame->X[8];
    ULONG TableIndex = (ServiceNumber >> 12) & 1;
    ULONG ServiceIndex = ServiceNumber & 0xFFF;
    PKSERVICE_TABLE_DESCRIPTOR Table;
    PSYSCALL_ROUTINE Routine;
    ULONG_PTR SvcResult;
    NTSTATUS GuiConvertStatus;
    KPROCESSOR_MODE OldMode;
    PKTHREAD Thread;
    PKTRAP_FRAME OldTrapFrame;
    ULONG_PTR Args[15] = {0};
    ULONG ArgCount, i;

    /* Get current thread and its service table */
    Thread = KeGetCurrentThread();
    Table = (PKSERVICE_TABLE_DESCRIPTOR)Thread->ServiceTable;
    if (!Table)
        Table = KeServiceDescriptorTable;
    Table = &Table[TableIndex];

    /* Validate service number */
    if (ServiceIndex >= Table->Limit)
    {
        /* Check if this is a Win32k call on a non-GUI thread */
        if (TableIndex == 1 &&
            Thread->ServiceTable == KeServiceDescriptorTable)
        {
            /* GUI thread conversion trace — disabled for clean console */

            /* Convert to GUI thread (allocates W32Process/W32Thread,
             * switches ServiceTable to KeServiceDescriptorTableShadow) */
            GuiConvertStatus = PsConvertToGuiThread();
            if (NT_SUCCESS(GuiConvertStatus))
            {
                /* Re-fetch from the now-valid Win32k SSDT */
                Table = (PKSERVICE_TABLE_DESCRIPTOR)Thread->ServiceTable;
                Table = &Table[TableIndex];
                if (ServiceIndex < Table->Limit)
                    goto DispatchCall;
            }
            TrapPuts("[SVC] GUI conversion failed: ");
            TrapPutHex32((ULONG)GuiConvertStatus);
            TrapPuts("\r\n");
        }
        TrapFrame->X[0] = (ULONG64)STATUS_INVALID_SYSTEM_SERVICE;
        return;
    }

DispatchCall:

    /* Flush GDI batch before Win32k syscalls (matches amd64 traphandler.c:197).
     * User-mode GDI32 batches PatBlt/DeleteObj commands in TEB->GdiTebBatch.
     * Without this flush, batched painting runs AFTER DeleteObject destroys
     * brushes, causing all brush colors to resolve to WHITE. */
    if (TableIndex == 1 && (TrapFrame->Spsr & 0xF) == 0)
    {
        PTEB Teb = KeGetCurrentThread()->Teb;
        /* A WoW64 guest's gdi32 batches into its TEB32 and bumps
         * TEB32->GdiBatchCount, so testing TEB64's here never sees a guest's
         * batch -- the flush never runs, nothing resets Offset, and gdi32
         * (which writes at Buffer+Offset without re-checking after its own
         * NtGdiFlush) walks out of the buffer into CLIENTINFO. Field manual
         * §9.41, second occurrence; measured on amd64 with Offset reaching
         * 0x508 from the buffer base, which is pDeskInfo exactly. */
        PULONG pGdiBatchCount = Teb ? KiWow64GdiBatchCount(Teb) : NULL;
        if (pGdiBatchCount && MmIsAddressValid(pGdiBatchCount) &&
            *pGdiBatchCount != 0 && KeGdiFlushUserBatch)
        {
            KeGdiFlushUserBatch();
        }
    }

    /* Win32k / NtDelayExecution traces removed — explorer.exe runs */

    /* Get the service routine */
    Routine = (PSYSCALL_ROUTINE)Table->Base[ServiceIndex];
    if (!Routine)
    {
        TrapFrame->X[0] = (ULONG64)STATUS_INVALID_SYSTEM_SERVICE;
        return;
    }

    /* Copy register arguments (x0-x7 = first 8 args) */
    for (i = 0; i < 8 && i < 15; i++)
        Args[i] = TrapFrame->X[i];

    /* Copy stack arguments if needed (args 9+) */
    ArgCount = Table->Number[ServiceIndex] / sizeof(ULONG_PTR);
    if (ArgCount > 8)
    {
        PULONG_PTR CallerStack = (PULONG_PTR)TrapFrame->Sp;
        ULONG StackArgs = ArgCount - 8;
        if (StackArgs > 7) StackArgs = 7; /* Limit to our buffer */
        for (i = 0; i < StackArgs; i++)
            Args[8 + i] = CallerStack[i];
    }

    /* Set PreviousMode from SPSR: M[3:0]==0 is EL0t (user), else kernel */
    OldMode = Thread->PreviousMode;
    if ((TrapFrame->Spsr & 0xF) == 0)
        Thread->PreviousMode = UserMode;
    else
        Thread->PreviousMode = KernelMode;

    /*
     * Link the trap frame to the thread. System calls like NtContinue,
     * NtGetContextThread, etc. need Thread->TrapFrame to find the current
     * user-mode context. Save and restore the old value for nested traps.
     */
    OldTrapFrame = Thread->TrapFrame;
    TrapFrame->TrapFrame = (ULONG64)OldTrapFrame;
    Thread->TrapFrame = TrapFrame;

    /* Dispatch the system call */
    SvcResult = Routine(Args[0], Args[1], Args[2], Args[3],
                        Args[4], Args[5], Args[6], Args[7],
                        Args[8], Args[9], Args[10], Args[11],
                        Args[12], Args[13], Args[14]);

    /* Restore TrapFrame link and PreviousMode */
    Thread->TrapFrame = OldTrapFrame;
    Thread->PreviousMode = OldMode;
    TrapFrame->X[0] = (ULONG64)SvcResult;

    /* ARM64: Voluntary preemption at syscall boundary.
     * Without timer/software interrupts, a user-mode thread calling syscalls
     * in a tight loop (e.g. WaitForSingleObject on a signaled object) can
     * starve higher-priority threads indefinitely.  Check if a higher-priority
     * thread is waiting in the ready queue and yield before returning. */
    if (TableIndex == 0 && (TrapFrame->Spsr & 0xF) == 0)  /* User-mode native syscall */
    {
        PKPRCB Prcb = KeGetCurrentPrcb();
        if (Prcb->ReadySummary)
        {
            ULONG HighBit;
            BitScanReverse(&HighBit, Prcb->ReadySummary);
            if ((SCHAR)HighBit > Thread->Priority)
            {
                NtYieldExecution();
            }
        }
    }

    /* SVC-FAIL trace disabled — too verbose for normal operation.
     * Re-enable for debugging by uncommenting below. */
#if 0
    if (OldMode == UserMode && !NT_SUCCESS((NTSTATUS)SvcResult))
    {
        TrapPuts("[SVC-FAIL] #");
        TrapPutHex32(ServiceNumber);
        TrapPuts("(");
        TrapPutHex64(Args[0]);
        TrapPuts(")=>");
        TrapPutHex32((ULONG)SvcResult);
        TrapPuts("\r\n");
    }
#endif
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
            KPROCESSOR_MODE FaultMode;
            ULONG FaultCode;

            /* Determine fault mode from PreviousMode (set by TRAP_PROLOG_EL0 for EL0) */
            FaultMode = (TrapFrame->PreviousMode == UserMode) ? UserMode : KernelMode;

            /*
             * Encode FaultCode in x86-compatible format for MmAccessFault:
             *   Bit 0: 1 if page present (permission fault), 0 if not present
             *   Bit 1: 1 if write access
             */
            FaultCode = 0;
            if ((DFSC & 0x3C) == 0x0C)  /* Permission fault → page was present */
                FaultCode |= 0x1;
            if (IsWrite)
                FaultCode |= 0x2;

            /*
             * Handle faults that MmAccessFault can resolve:
             *   Translation fault (DFSC 0x04-0x07): page not mapped
             *   Access flag fault (DFSC 0x08-0x0B): access bit not set
             *   Permission fault  (DFSC 0x0C-0x0F): wrong permissions (COW, guard, etc.)
             */
            if ((DFSC & 0x3C) == 0x04 ||
                (DFSC & 0x3C) == 0x08 ||
                (DFSC & 0x3C) == 0x0C)
            {
                NTSTATUS Status;

                Status = MmAccessFault(FaultCode,
                                       (PVOID)TrapFrame->FaultAddress,
                                       FaultMode,
                                       TrapFrame);
                if (NT_SUCCESS(Status))
                {
                    /* Page fault counter — disabled for clean console output */
                    return; /* Resume execution */
                }

                /* MmAccessFault failed — dispatch as access violation
                 * (trace disabled for clean console output) */
            }

            /* Dispatch as access violation exception */
            {
                EXCEPTION_RECORD ExRec;
                ExRec.ExceptionCode = STATUS_ACCESS_VIOLATION;
                ExRec.ExceptionFlags = 0;
                ExRec.ExceptionRecord = NULL;
                ExRec.ExceptionAddress = (PVOID)TrapFrame->Pc;
                ExRec.NumberParameters = 2;
                ExRec.ExceptionInformation[0] = IsWrite ? EXCEPTION_WRITE_FAULT : EXCEPTION_READ_FAULT;
                ExRec.ExceptionInformation[1] = TrapFrame->FaultAddress;
                KiDispatchException(&ExRec, NULL, TrapFrame, FaultMode, TRUE);
                return;
            }
        }

        case 0x21: /* Instruction Abort (same EL) */
        case 0x20: /* Instruction Abort (lower EL) */
        {
            ULONG IFSC = ISS & 0x3F;
            KPROCESSOR_MODE FaultMode2;
            ULONG FaultCode2;

            FaultMode2 = (TrapFrame->PreviousMode == UserMode) ? UserMode : KernelMode;

            /*
             * Encode FaultCode for instruction fetch:
             *   Bit 0: 1 if permission fault (page present), 0 if not present
             *   Bit 4: 1 for instruction fetch
             */
            FaultCode2 = 0x10;  /* Instruction fetch */
            if ((IFSC & 0x3C) == 0x0C)  /* Permission fault → page present */
                FaultCode2 |= 0x1;

            /* Translation/access flag/permission fault — try MmAccessFault */
            if ((IFSC & 0x3C) == 0x04 ||
                (IFSC & 0x3C) == 0x08 ||
                (IFSC & 0x3C) == 0x0C)
            {
                NTSTATUS Status2;
                Status2 = MmAccessFault(FaultCode2,
                                        (PVOID)TrapFrame->FaultAddress,
                                        FaultMode2,
                                        TrapFrame);
                if (NT_SUCCESS(Status2))
                {
                    return;
                }
            }

            /* Dispatch as access violation */
            {
                EXCEPTION_RECORD ExRec;
                ExRec.ExceptionCode = STATUS_ACCESS_VIOLATION;
                ExRec.ExceptionFlags = 0;
                ExRec.ExceptionRecord = NULL;
                ExRec.ExceptionAddress = (PVOID)TrapFrame->Pc;
                ExRec.NumberParameters = 2;
                ExRec.ExceptionInformation[0] = EXCEPTION_EXECUTE_FAULT;
                ExRec.ExceptionInformation[1] = TrapFrame->FaultAddress;
                KiDispatchException(&ExRec, NULL, TrapFrame, FaultMode2, TRUE);
                return;
            }
        }

        case 0x3C: /* BRK (software breakpoint) */
        {
            ULONG BrkImm = ISS & 0xFFFF;

            /* BRK #0xF000 = DebugService: x0=Command, x1=Arg1, x2=Arg2
             * BREAKPOINT_PRINT (1): x1=buffer, x2=length → output to UART */
            if (BrkImm == 0xF000 && TrapFrame->X[0] == 1)
            {
                char *Buf = (char *)(ULONG_PTR)TrapFrame->X[1];
                ULONG Len = (ULONG)(ULONG_PTR)TrapFrame->X[2];
                volatile ULONG *Uart = PL011_BASE;
                ULONG i;

                if (Len > 512) Len = 512; /* Sanity limit */
                for (i = 0; i < Len; i++)
                {
                    if (!MmIsAddressValid(Buf + i))
                    {
                        /* Try to fault in the page (user-mode demand paging) */
                        if ((ULONG_PTR)(Buf + i) < MmUserProbeAddress)
                            MmAccessFault(FALSE, Buf + i, KernelMode, NULL);
                        if (!MmIsAddressValid(Buf + i)) break;
                    }
                    if (Buf[i] == '\n') *Uart = (ULONG)'\r';
                    *Uart = (ULONG)(unsigned char)Buf[i];
                }
            }

            /* Skip past the BRK instruction (4 bytes) */
            TrapFrame->Pc += 4;
            return;
        }

        case 0x15: /* SVC (system call) */
        {
            KiSvcDispatch(TrapFrame);
            return;
        }

        case 0x26: /* SP Alignment fault */
        {
            EXCEPTION_RECORD ExRec;
            KPROCESSOR_MODE FaultMode4;

            FaultMode4 = (TrapFrame->PreviousMode == UserMode) ? UserMode : KernelMode;

            ExRec.ExceptionCode = STATUS_DATATYPE_MISALIGNMENT;
            ExRec.ExceptionFlags = 0;
            ExRec.ExceptionRecord = NULL;
            ExRec.ExceptionAddress = (PVOID)TrapFrame->Pc;
            ExRec.NumberParameters = 0;
            KiDispatchException(&ExRec, NULL, TrapFrame, FaultMode4, TRUE);
            return;
        }

        default:
        {
            EXCEPTION_RECORD ExRec;
            KPROCESSOR_MODE FaultMode3;

            FaultMode3 = (TrapFrame->PreviousMode == UserMode) ? UserMode : KernelMode;

            ExRec.ExceptionCode = STATUS_ILLEGAL_INSTRUCTION;
            ExRec.ExceptionFlags = 0;
            ExRec.ExceptionRecord = NULL;
            ExRec.ExceptionAddress = (PVOID)TrapFrame->Pc;
            ExRec.NumberParameters = 0;
            KiDispatchException(&ExRec, NULL, TrapFrame, FaultMode3, TRUE);
            return;
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
