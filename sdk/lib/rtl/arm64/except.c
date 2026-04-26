/*
 * PROJECT:         ReactOS Run-Time Library
 * LICENSE:         GPL-2.0-or-later
 * FILE:            lib/rtl/arm64/except.c
 * PURPOSE:         User-mode exception support for ARM64
 */

/* INCLUDES *****************************************************************/

#include <rtl.h>
#define NDEBUG
#include <debug.h>

/* ARM64 CONTEXT flags */
#ifndef CONTEXT_ARM64
#define CONTEXT_ARM64           0x00400000L
#endif
#ifndef CONTEXT_CONTROL
#define CONTEXT_CONTROL         (CONTEXT_ARM64 | 0x1L)
#endif
#ifndef CONTEXT_INTEGER
#define CONTEXT_INTEGER         (CONTEXT_ARM64 | 0x2L)
#endif

/* Unwind handler type flags */
#ifndef UNW_FLAG_NHANDLER
#define UNW_FLAG_NHANDLER  0x0
#define UNW_FLAG_EHANDLER  0x1
#define UNW_FLAG_UHANDLER  0x2
#endif

/* Helper macro: convert RVA to pointer (override rtlp.h's swapped-arg version) */
#undef RVA
#define RVA(base, rva) ((PVOID)((ULONG_PTR)(base) + (ULONG_PTR)(rva)))

/* ARM64 exception filter/handler types (may not be defined for this arch) */
typedef LONG (*PARM64_EXCEPTION_FILTER)(
    struct _EXCEPTION_POINTERS *ExceptionPointers,
    PVOID EstablisherFrame);

typedef VOID (*PARM64_TERMINATION_HANDLER)(
    BOOLEAN AbnormalTermination,
    PVOID EstablisherFrame);

/* Forward declarations */
VOID NTAPI RtlRestoreContext(PCONTEXT ContextRecord, PEXCEPTION_RECORD ExceptionRecord);
VOID NTAPI RtlUnwindEx(PVOID TargetFrame, PVOID TargetIp, PEXCEPTION_RECORD ExceptionRecord,
                        PVOID ReturnValue, PCONTEXT ContextRecord, PVOID HistoryTable);

/* PRIVATE FUNCTIONS ********************************************************/

/*
 * RtlpLookupFunctionTable - find the .pdata exception table for a code address.
 */
static PRUNTIME_FUNCTION
RtlpLookupFunctionTable(
    _In_ ULONG64 ControlPc,
    _Out_ PULONG64 ImageBase,
    _Out_ PULONG Length)
{
    PVOID Table;
    ULONG Size;

    /* Find the PE image header for this code address */
    if (!RtlPcToFileHeader((PVOID)ControlPc, (PVOID*)ImageBase))
    {
        return NULL;
    }

    /* Locate the exception directory (.pdata) */
    Table = RtlImageDirectoryEntryToData((PVOID)*ImageBase,
                                         TRUE,
                                         IMAGE_DIRECTORY_ENTRY_EXCEPTION,
                                         &Size);
    if (!Table)
    {
        return NULL;
    }

    *Length = Size / sizeof(RUNTIME_FUNCTION);
    return (PRUNTIME_FUNCTION)Table;
}

/*
 * RtlpGetFunctionEndAddress - compute function end address from RUNTIME_FUNCTION.
 * ARM64 RUNTIME_FUNCTION has no EndAddress field; it must be derived.
 */
static ULONG
RtlpGetFunctionEndAddress(
    _In_ PRUNTIME_FUNCTION FunctionEntry,
    _In_ ULONG64 ImageBase)
{
    if (FunctionEntry->Flag != 0)
    {
        /* Packed format: FunctionLength is in 4-byte units */
        return FunctionEntry->BeginAddress + FunctionEntry->FunctionLength * 4;
    }
    else
    {
        /* .xdata format: first DWORD has FunctionLength in bits [17:0] (in 4-byte units) */
        PULONG XData = (PULONG)RVA(ImageBase, FunctionEntry->UnwindData);
        ULONG FunctionLength = (*XData) & 0x3FFFF;
        return FunctionEntry->BeginAddress + FunctionLength * 4;
    }
}

/* PUBLIC FUNCTIONS **********************************************************/

VOID
NTAPI
RtlInitializeContext(
    IN HANDLE ProcessHandle,
    OUT PCONTEXT ThreadContext,
    IN PVOID ThreadStartParam  OPTIONAL,
    IN PTHREAD_START_ROUTINE ThreadStartAddress,
    IN PINITIAL_TEB StackBase)
{
    RtlZeroMemory(ThreadContext, sizeof(*ThreadContext));

    ThreadContext->Pc = (ULONG64)ThreadStartAddress;
    ThreadContext->Sp = (ULONG64)StackBase;
    ThreadContext->Sp &= ~(ULONG64)15;
    ThreadContext->X0 = (ULONG64)ThreadStartParam;
    ThreadContext->Cpsr = 0; /* EL0t */
    ThreadContext->ContextFlags = CONTEXT_CONTROL | CONTEXT_INTEGER;
}

/*
 * RtlLookupFunctionEntry - binary search of .pdata for a code address.
 */
PRUNTIME_FUNCTION
NTAPI
RtlLookupFunctionEntry(
    _In_ ULONG64 ControlPc,
    _Out_ PULONG64 ImageBase,
    _Inout_opt_ PVOID HistoryTable)
{
    PRUNTIME_FUNCTION FunctionTable, FunctionEntry;
    ULONG TableLength;
    ULONG IndexLo, IndexHi, IndexMid;
    ULONG ControlRva;

    FunctionTable = RtlpLookupFunctionTable(ControlPc, ImageBase, &TableLength);
    if (!FunctionTable)
    {
        *ImageBase = 0;
        return NULL;
    }

    ControlRva = (ULONG)(ControlPc - *ImageBase);

    /* Binary search */
    IndexLo = 0;
    IndexHi = TableLength;
    while (IndexHi > IndexLo)
    {
        IndexMid = (IndexLo + IndexHi) / 2;
        FunctionEntry = &FunctionTable[IndexMid];

        if (ControlRva < FunctionEntry->BeginAddress)
        {
            IndexHi = IndexMid;
        }
        else if (ControlRva >= RtlpGetFunctionEndAddress(FunctionEntry, *ImageBase))
        {
            IndexLo = IndexMid + 1;
        }
        else
        {
            return FunctionEntry;
        }
    }

    return NULL;
}

/*
 * RtlVirtualUnwind - unwind one stack frame using ARM64 packed unwind data.
 *
 * For packed entries (Flag != 0), the prologue follows a canonical pattern:
 *   - If CR==3: stp x29, x30 saved (frame pointer + LR)
 *   - Integer registers x19-x(18+RegI) saved
 *   - FP registers d8-d(7+RegF*2) saved (if RegF > 0)
 *   - Total allocation: FrameSize * 16 bytes
 *
 * Save area layout from FP (when CR==3):
 *   [FP+0]:  saved x29, [FP+8]: saved x30
 *   [FP+16]: x19, [FP+24]: x20, ... [FP+16+(i-1)*8]: x(18+i)
 *
 * For CR==0/1 (no frame pointer): saves at top of allocation.
 */
PEXCEPTION_ROUTINE
NTAPI
RtlVirtualUnwind(
    _In_ ULONG HandlerType,
    _In_ ULONG64 ImageBase,
    _In_ ULONG64 ControlPc,
    _In_ PRUNTIME_FUNCTION FunctionEntry,
    _Inout_ PCONTEXT Context,
    _Out_ PVOID *HandlerData,
    _Out_ PULONG64 EstablisherFrame,
    _Inout_opt_ PVOID ContextPointers)
{
    PKNONVOLATILE_CONTEXT_POINTERS NvContext =
        (PKNONVOLATILE_CONTEXT_POINTERS)ContextPointers;
    ULONG RegI, RegF, CR, H, FrameSize, Flag;
    ULONG64 SaveBase;
    ULONG Offset;
    ULONG IntSaveSize, FpSaveSize, CrSaveSize;
    ULONG SaveAreaSize;
    PEXCEPTION_ROUTINE HandlerRoutine = NULL;

    *HandlerData = NULL;

    if (FunctionEntry->Flag != 0)
    {
        /* ===== Packed unwind format ===== */
        Flag = FunctionEntry->Flag;
        RegI = FunctionEntry->RegI;
        RegF = FunctionEntry->RegF;
        H = FunctionEntry->H;
        CR = FunctionEntry->CR;
        FrameSize = FunctionEntry->FrameSize;

        /* Compute save area sizes */
        CrSaveSize = (CR == 3) ? 16 : ((CR == 1) ? 8 : 0);
        /* Integer regs saved: RegI registers, stored in pairs (last may be unpaired) */
        IntSaveSize = ((RegI + 1) / 2) * 16;
        if (RegI % 2 != 0 && CR != 3) IntSaveSize -= 8; /* unpaired without CR=3 rounding */
        /* For simplicity, use RegI * 8 and round up to 16 for pairing */
        IntSaveSize = ((RegI + 1) / 2) * 16;
        /* FP regs: if RegF > 0, RegF pairs of d-registers */
        FpSaveSize = (RegF > 0) ? (RegF * 16) : 0;
        /* Parameter homes */
        SaveAreaSize = CrSaveSize + IntSaveSize + FpSaveSize + (H ? 64 : 0);

        /* Establisher frame is the function's SP */
        if (CR == 3)
        {
            /* Frame pointer available: FP points to the FP/LR save area */
            *EstablisherFrame = Context->X[29]; /* FP register */
            SaveBase = Context->X[29]; /* saves start at FP */
        }
        else
        {
            /* No frame pointer: saves at top of frame */
            *EstablisherFrame = Context->Sp;
            SaveBase = Context->Sp + (ULONG64)FrameSize * 16 - SaveAreaSize;
        }

        /* Restore registers from save area */
        Offset = 0;

        /* CR==3: restore FP and LR from [SaveBase+0] and [SaveBase+8] */
        if (CR == 3)
        {
            ULONG64 SavedFp = *(PULONG64)(SaveBase + 0);
            ULONG64 SavedLr = *(PULONG64)(SaveBase + 8);
            Context->X[29] = SavedFp;
            Context->X[30] = SavedLr;
            if (NvContext)
            {
                NvContext->Fp = (PDWORD64)(SaveBase + 0);
                NvContext->Lr = (PDWORD64)(SaveBase + 8);
            }
            Offset = 16;
        }

        /* Restore integer registers x19-x(18+RegI) */
        {
            ULONG i;
            PDWORD64 *NvRegs = NvContext ? &NvContext->X19 : NULL;
            for (i = 0; i < RegI; i++)
            {
                ULONG64 *Slot = (PULONG64)(SaveBase + Offset + i * 8);
                Context->X[19 + i] = *Slot;
                if (NvRegs) NvRegs[i] = (PDWORD64)Slot;
            }
            Offset += IntSaveSize;
        }

        /* Skip FP registers for now (RegF handling) */
        Offset += FpSaveSize;

        /* Restore SP: caller's SP = current SP + FrameSize*16 */
        Context->Sp += (ULONG64)FrameSize * 16;

        /* Restore PC from LR */
        if (CR == 3)
        {
            /* LR was saved and restored above */
            Context->Pc = Context->X[30];
        }
        else
        {
            /* LR register holds return address (leaf or LR not saved) */
            Context->Pc = Context->Lr;
        }

        /* Check for exception/unwind handler */
        if (Flag == 1)
        {
            /* Packed short: no handler */
            HandlerRoutine = NULL;
        }
        else if (Flag == 2)
        {
            /* Packed with .xdata containing handler info */
            /* The handler data follows the RUNTIME_FUNCTION in .xdata */
            /* For now, no handler */
            HandlerRoutine = NULL;
        }
    }
    else
    {
        /* ===== .xdata unwind format ===== */
        PULONG XData = (PULONG)RVA(ImageBase, FunctionEntry->UnwindData);
        ULONG XDataHeader = XData[0];
        ULONG FuncLen = XDataHeader & 0x3FFFF;
        ULONG HasHandler = (XDataHeader >> 20) & 1; /* X bit */
        ULONG SingleEpilog = (XDataHeader >> 21) & 1; /* E bit */
        ULONG EpilogCount = (XDataHeader >> 22) & 0x1F;
        ULONG CodeWords = (XDataHeader >> 27) & 0x1F;
        ULONG EpilogScopeCount;
        PULONG HandlerRva;

        (void)FuncLen;
        (void)SingleEpilog;

        /* Establisher frame = current SP */
        *EstablisherFrame = Context->Sp;

        /* For .xdata, we can't easily decode the unwind codes in this initial
         * implementation. Do a simple leaf-function unwind: PC = LR */
        Context->Pc = Context->Lr;

        /* If handler exists, find it after epilog scopes and unwind codes */
        if (HasHandler)
        {
            EpilogScopeCount = SingleEpilog ? 0 : EpilogCount;
            HandlerRva = &XData[1 + EpilogScopeCount + CodeWords];

            if (HandlerType & (UNW_FLAG_EHANDLER | UNW_FLAG_UHANDLER))
            {
                HandlerRoutine = (PEXCEPTION_ROUTINE)RVA(ImageBase, *HandlerRva);
                *HandlerData = (PVOID)(HandlerRva + 1);
            }
        }
    }

    return HandlerRoutine;
}

/*
 * RtlDispatchException - walk the call stack dispatching to exception handlers.
 */
BOOLEAN
NTAPI
RtlDispatchException(
    _In_ PEXCEPTION_RECORD ExceptionRecord,
    _In_ PCONTEXT Context)
{
    DISPATCHER_CONTEXT DispatcherContext;
    PEXCEPTION_ROUTINE ExceptionRoutine;
    EXCEPTION_DISPOSITION Disposition;
    PRUNTIME_FUNCTION FunctionEntry;
    ULONG_PTR StackLow, StackHigh;
    ULONG64 ImageBase, EstablisherFrame;
    CONTEXT UnwindContext;
    ULONG Iterations = 0;

    RtlpGetStackLimits(&StackLow, &StackHigh);

    /* Copy context for unwinding */
    UnwindContext = *Context;

    /* Initialize dispatcher context */
    RtlZeroMemory(&DispatcherContext, sizeof(DispatcherContext));
    DispatcherContext.ContextRecord = Context;

    /* Walk frames */
    while (Iterations++ < 200)
    {
        /* Look up the function entry for current PC */
        FunctionEntry = RtlLookupFunctionEntry(UnwindContext.Pc,
                                                &ImageBase,
                                                NULL);
        if (!FunctionEntry)
        {
            /* Leaf function: PC = LR, done if LR is 0 */
            if (UnwindContext.Lr == 0) break;
            UnwindContext.Pc = UnwindContext.Lr;
            UnwindContext.Lr = 0;
            continue;
        }

        /* Virtual unwind to find handler */
        ExceptionRoutine = RtlVirtualUnwind(UNW_FLAG_EHANDLER,
                                            ImageBase,
                                            UnwindContext.Pc,
                                            FunctionEntry,
                                            &UnwindContext,
                                            &DispatcherContext.HandlerData,
                                            &EstablisherFrame,
                                            NULL);

        /* Validate establisher frame */
        if (EstablisherFrame < StackLow ||
            EstablisherFrame >= StackHigh ||
            (EstablisherFrame & 0x7))
        {
            ExceptionRecord->ExceptionFlags |= EXCEPTION_STACK_INVALID;
            return FALSE;
        }

        /* If there's a handler, call it */
        if (ExceptionRoutine != NULL)
        {
            DispatcherContext.ControlPc = Context->Pc;
            DispatcherContext.ImageBase = ImageBase;
            DispatcherContext.FunctionEntry = FunctionEntry;
            DispatcherContext.EstablisherFrame = EstablisherFrame;
            DispatcherContext.LanguageHandler = ExceptionRoutine;
            DispatcherContext.ScopeIndex = 0;

            Disposition = ExceptionRoutine(ExceptionRecord,
                                           (PVOID)EstablisherFrame,
                                           Context,
                                           &DispatcherContext);

            switch (Disposition)
            {
                case ExceptionContinueExecution:
                    if (ExceptionRecord->ExceptionFlags & EXCEPTION_NONCONTINUABLE)
                    {
                        EXCEPTION_RECORD NestedRecord;
                        NestedRecord.ExceptionCode = STATUS_NONCONTINUABLE_EXCEPTION;
                        NestedRecord.ExceptionFlags = EXCEPTION_NONCONTINUABLE;
                        NestedRecord.ExceptionRecord = ExceptionRecord;
                        NestedRecord.NumberParameters = 0;
                        RtlRaiseException(&NestedRecord);
                    }
                    return TRUE;

                case ExceptionContinueSearch:
                    break;

                case ExceptionNestedException:
                    ExceptionRecord->ExceptionFlags |= EXCEPTION_NESTED_CALL;
                    break;

                default:
                {
                    EXCEPTION_RECORD NestedRecord;
                    NestedRecord.ExceptionCode = STATUS_INVALID_DISPOSITION;
                    NestedRecord.ExceptionFlags = EXCEPTION_NONCONTINUABLE;
                    NestedRecord.ExceptionRecord = ExceptionRecord;
                    NestedRecord.NumberParameters = 0;
                    RtlRaiseException(&NestedRecord);
                }
            }
        }

        /* If PC is 0, we've reached the end of the stack */
        if (UnwindContext.Pc == 0) break;
    }

    return FALSE;
}

/*
 * __C_specific_handler - implements __try/__except/__finally for ARM64.
 */
EXCEPTION_DISPOSITION
__cdecl
__C_specific_handler(
    _In_ struct _EXCEPTION_RECORD *ExceptionRecord,
    _In_ void *EstablisherFrame,
    _Inout_ struct _CONTEXT *ContextRecord,
    _Inout_ struct _DISPATCHER_CONTEXT *DispatcherContext)
{
    PSCOPE_TABLE ScopeTable;
    ULONG ControlRva;
    ULONG i, StartIndex;
    ULONG64 ImageBase;
    LONG FilterResult;

    ImageBase = DispatcherContext->ImageBase;
    ScopeTable = (PSCOPE_TABLE)DispatcherContext->HandlerData;
    ControlRva = (ULONG)(DispatcherContext->ControlPc - ImageBase);
    StartIndex = DispatcherContext->ScopeIndex;

    if (ExceptionRecord->ExceptionFlags & EXCEPTION_UNWIND)
    {
        /* ===== Unwind phase: call __finally handlers ===== */
        for (i = StartIndex; i < ScopeTable->Count; i++)
        {
            if (ControlRva < ScopeTable->ScopeRecord[i].BeginAddress ||
                ControlRva >= ScopeTable->ScopeRecord[i].EndAddress)
            {
                continue;
            }

            /* JumpTarget == 0 means __finally handler */
            if (ScopeTable->ScopeRecord[i].JumpTarget == 0)
            {
                PARM64_TERMINATION_HANDLER TermHandler;
                TermHandler = (PARM64_TERMINATION_HANDLER)
                    RVA(ImageBase, ScopeTable->ScopeRecord[i].HandlerAddress);

                /* Set scope index for potential nested exceptions */
                DispatcherContext->ScopeIndex = i + 1;

                /* Call __finally handler: AbnormalTermination = TRUE */
                TermHandler(TRUE, EstablisherFrame);
            }
        }

        return ExceptionContinueSearch;
    }
    else
    {
        /* ===== Exception dispatch phase: find matching __except handler ===== */
        for (i = StartIndex; i < ScopeTable->Count; i++)
        {
            if (ControlRva < ScopeTable->ScopeRecord[i].BeginAddress ||
                ControlRva >= ScopeTable->ScopeRecord[i].EndAddress)
            {
                continue;
            }

            /* JumpTarget != 0 means __except handler */
            if (ScopeTable->ScopeRecord[i].JumpTarget != 0)
            {
                if (ScopeTable->ScopeRecord[i].HandlerAddress == 1)
                {
                    /* EXCEPTION_EXECUTE_HANDLER (constant filter = 1) */
                    FilterResult = EXCEPTION_EXECUTE_HANDLER;
                }
                else
                {
                    /* Call the filter function */
                    EXCEPTION_POINTERS ExceptionPointers;
                    PARM64_EXCEPTION_FILTER Filter;

                    ExceptionPointers.ExceptionRecord = ExceptionRecord;
                    ExceptionPointers.ContextRecord = ContextRecord;

                    Filter = (PARM64_EXCEPTION_FILTER)
                        RVA(ImageBase, ScopeTable->ScopeRecord[i].HandlerAddress);

                    FilterResult = Filter(&ExceptionPointers, EstablisherFrame);
                }

                if (FilterResult == EXCEPTION_CONTINUE_EXECUTION)
                {
                    return ExceptionContinueExecution;
                }
                else if (FilterResult == EXCEPTION_EXECUTE_HANDLER)
                {
                    PVOID JumpTarget;
                    JumpTarget = RVA(ImageBase, ScopeTable->ScopeRecord[i].JumpTarget);

                    /* Call __finally handlers for unwinding */
                    DispatcherContext->ScopeIndex = i + 1;

                    /* Unwind to the __except block */
                    RtlUnwindEx(EstablisherFrame,
                                JumpTarget,
                                ExceptionRecord,
                                UlongToPtr(ExceptionRecord->ExceptionCode),
                                ContextRecord,
                                DispatcherContext->HistoryTable);

                    /* Should not return */
                    ASSERT(FALSE);
                }
                /* EXCEPTION_CONTINUE_SEARCH: try next scope */
            }
        }

        return ExceptionContinueSearch;
    }
}

/*
 * RtlUnwindEx - unwind the stack to a target frame, calling unwind handlers.
 */
VOID
NTAPI
RtlUnwindEx(
    _In_opt_ PVOID TargetFrame,
    _In_opt_ PVOID TargetIp,
    _In_opt_ PEXCEPTION_RECORD ExceptionRecord,
    _In_ PVOID ReturnValue,
    _In_ PCONTEXT ContextRecord,
    _In_opt_ PVOID HistoryTable)
{
    DISPATCHER_CONTEXT DispatcherContext;
    PEXCEPTION_ROUTINE ExceptionRoutine;
    EXCEPTION_DISPOSITION Disposition;
    PRUNTIME_FUNCTION FunctionEntry;
    EXCEPTION_RECORD LocalRecord;
    ULONG_PTR StackLow, StackHigh;
    ULONG64 ImageBase, EstablisherFrame;
    CONTEXT UnwindContext;
    ULONG Iterations = 0;

    RtlpGetStackLimits(&StackLow, &StackHigh);

    /* Create exception record if not provided */
    if (!ExceptionRecord)
    {
        LocalRecord.ExceptionCode = STATUS_UNWIND;
        LocalRecord.ExceptionFlags = 0;
        LocalRecord.ExceptionRecord = NULL;
        LocalRecord.ExceptionAddress = (PVOID)ContextRecord->Pc;
        LocalRecord.NumberParameters = 0;
        ExceptionRecord = &LocalRecord;
    }

    /* Set unwind flags */
    ExceptionRecord->ExceptionFlags |= EXCEPTION_UNWINDING;
    if (!TargetFrame)
    {
        ExceptionRecord->ExceptionFlags |= EXCEPTION_EXIT_UNWIND;
    }

    /* Capture current context if not provided */
    RtlCaptureContext(&UnwindContext);
    UnwindContext = *ContextRecord;

    RtlZeroMemory(&DispatcherContext, sizeof(DispatcherContext));
    DispatcherContext.ContextRecord = &UnwindContext;
    DispatcherContext.HistoryTable = HistoryTable;
    DispatcherContext.TargetPc = (ULONG_PTR)TargetIp;

    /* Walk frames calling unwind handlers */
    while (Iterations++ < 200)
    {
        FunctionEntry = RtlLookupFunctionEntry(UnwindContext.Pc,
                                                &ImageBase,
                                                NULL);
        if (!FunctionEntry)
        {
            /* Leaf function */
            if (UnwindContext.Lr == 0) break;
            UnwindContext.Pc = UnwindContext.Lr;
            UnwindContext.Lr = 0;
            continue;
        }

        DispatcherContext.ControlPc = UnwindContext.Pc;

        ExceptionRoutine = RtlVirtualUnwind(UNW_FLAG_UHANDLER,
                                            ImageBase,
                                            UnwindContext.Pc,
                                            FunctionEntry,
                                            &UnwindContext,
                                            &DispatcherContext.HandlerData,
                                            &EstablisherFrame,
                                            NULL);

        /* Validate frame */
        if (EstablisherFrame < StackLow ||
            EstablisherFrame >= StackHigh ||
            (EstablisherFrame & 0x7))
        {
            break;
        }

        if (ExceptionRoutine)
        {
            /* Mark target unwind if we reached the target frame */
            if (EstablisherFrame == (ULONG64)TargetFrame)
            {
                ExceptionRecord->ExceptionFlags |= EXCEPTION_TARGET_UNWIND;
            }

            DispatcherContext.ImageBase = ImageBase;
            DispatcherContext.FunctionEntry = FunctionEntry;
            DispatcherContext.EstablisherFrame = EstablisherFrame;
            DispatcherContext.LanguageHandler = ExceptionRoutine;
            DispatcherContext.ScopeIndex = 0;

            Disposition = ExceptionRoutine(ExceptionRecord,
                                           (PVOID)EstablisherFrame,
                                           ContextRecord,
                                           &DispatcherContext);

            ExceptionRecord->ExceptionFlags &= ~(EXCEPTION_TARGET_UNWIND |
                                                  EXCEPTION_COLLIDED_UNWIND);

            if (Disposition == ExceptionContinueSearch)
            {
                /* Continue to next frame */
            }
            else if (Disposition == ExceptionCollidedUnwind)
            {
                /* Collided unwind: restore and continue */
                UnwindContext = *DispatcherContext.ContextRecord;
            }
            else
            {
                break;
            }
        }

        /* Check if we reached the target frame */
        if (EstablisherFrame == (ULONG64)TargetFrame)
        {
            break;
        }
    }

    /* Set return value and target IP */
    ContextRecord->X[0] = (ULONG64)ReturnValue;
    if (ExceptionRecord->ExceptionCode != STATUS_UNWIND_CONSOLIDATE)
    {
        ContextRecord->Pc = (ULONG64)TargetIp;
    }

    /* Restore context (does not return) */
    RtlRestoreContext(ContextRecord, ExceptionRecord);
}

/*
 * RtlUnwind - simplified unwind (no context record parameter).
 */
VOID
NTAPI
RtlUnwind(
    _In_opt_ PVOID TargetFrame,
    _In_opt_ PVOID TargetIp,
    _In_opt_ PEXCEPTION_RECORD ExceptionRecord,
    _In_ PVOID ReturnValue)
{
    CONTEXT Context;
    RtlCaptureContext(&Context);
    RtlUnwindEx(TargetFrame, TargetIp, ExceptionRecord, ReturnValue, &Context, NULL);
}

/*
 * RtlRestoreContext - restore execution from a CONTEXT.
 */
VOID
NTAPI
RtlRestoreContext(
    _In_ PCONTEXT ContextRecord,
    _In_opt_ PEXCEPTION_RECORD ExceptionRecord)
{
    if (ExceptionRecord &&
        ExceptionRecord->ExceptionCode == STATUS_UNWIND_CONSOLIDATE &&
        ExceptionRecord->NumberParameters >= 1)
    {
        PVOID (*Consolidate)(PEXCEPTION_RECORD) =
            (PVOID (*)(PEXCEPTION_RECORD))ExceptionRecord->ExceptionInformation[0];
        ContextRecord->Pc = (ULONG64)Consolidate(ExceptionRecord);
    }

    /* Use NtContinue to restore the context */
    NtContinue(ContextRecord, FALSE);
}

/* Dynamic function table stubs (not needed for initial boot) */

BOOLEAN
NTAPI
RtlAddFunctionTable(
    _In_ PRUNTIME_FUNCTION FunctionTable,
    _In_ ULONG EntryCount,
    _In_ ULONG64 BaseAddress)
{
    return TRUE;
}

BOOLEAN
NTAPI
RtlDeleteFunctionTable(
    _In_ PRUNTIME_FUNCTION FunctionTable)
{
    return TRUE;
}

BOOLEAN
NTAPI
RtlInstallFunctionTableCallback(
    _In_ ULONG64 TableIdentifier,
    _In_ ULONG64 BaseAddress,
    _In_ ULONG Length,
    _In_ PVOID Callback,
    _In_opt_ PVOID Context,
    _In_opt_ PCWSTR OutOfProcessCallbackDll)
{
    return TRUE;
}

/* EOF */
