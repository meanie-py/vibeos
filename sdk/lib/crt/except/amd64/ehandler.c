/*
 * PROJECT:     ReactOS CRT library
 * LICENSE:     MIT (https://spdx.org/licenses/MIT)
 * PURPOSE:     C specific exception/unwind handler for AMD64
 * COPYRIGHT:   Copyright 2018-2021 Timo Kreuzer <timo.kreuzer@reactos.org>
*/

/*
 * SELF-CONTAINED / HEADERLESS (cmake-off WS1, 2026-06-21): was
 * `#include <precomp.h>` + `#include <winnt.h>` (the full crt/psdk header
 * chain) — now compiled directly by cc-rs via reactos-link's `.c_shims()`,
 * NOT replayed from cmake's compile_commands.json. Every type/struct/const
 * winnt.h provided is hand-defined below, LAYOUT-FAITHFUL to the amd64
 * definitions in build-amd64/sdk/include/psdk/winnt.h (DISPATCHER_CONTEXT,
 * SCOPE_TABLE_AMD64, EXCEPTION_RECORD, EXCEPTION_POINTERS) — verified
 * symbol-identical AND disassembly-identical vs the cmake object via the WS1
 * verify harness (nm + objdump -d). RtlUnwind/RtlUnwindEx keep __declspec
 * (dllimport) so the object references the same `__imp_` thunks as before.
 */

/* ---- minimal hand-defined types/macros (was: precomp.h + winnt.h) ---- */
typedef unsigned long      ULONG;      /* 32-bit on LLP64 amd64 */
typedef unsigned long      DWORD;      /* 32-bit */
typedef long               LONG;       /* 32-bit */
typedef unsigned long long ULONG64;
typedef unsigned long long ULONG_PTR;  /* pointer-sized on amd64 */
typedef unsigned char      BOOLEAN;
typedef void               VOID;
typedef void              *PVOID;

#define NTSYSAPI __declspec(dllimport)
#define EXCEPTION_MAXIMUM_PARAMETERS 15
#define EXCEPTION_UNWIND         0x66   /* UNWINDING|EXIT_UNWIND|TARGET|COLLIDED */
#define EXCEPTION_TARGET_UNWIND  0x20
#define EXCEPTION_EXECUTE_HANDLER 1
#define TRUE  1
#define NULL  ((PVOID)0)
#define UlongToPtr(u) ((PVOID)(ULONG_PTR)(ULONG)(u))
#define __debugbreak() __builtin_debugtrap()

typedef enum _EXCEPTION_DISPOSITION
{
    ExceptionContinueExecution = 0,
    ExceptionContinueSearch    = 1
} EXCEPTION_DISPOSITION;

typedef struct _EXCEPTION_RECORD
{
    DWORD ExceptionCode;
    DWORD ExceptionFlags;
    struct _EXCEPTION_RECORD *ExceptionRecord;
    PVOID ExceptionAddress;
    DWORD NumberParameters;
    ULONG_PTR ExceptionInformation[EXCEPTION_MAXIMUM_PARAMETERS];
} EXCEPTION_RECORD, *PEXCEPTION_RECORD;

typedef struct _EXCEPTION_POINTERS
{
    PEXCEPTION_RECORD ExceptionRecord;
    struct _CONTEXT  *ContextRecord;
} EXCEPTION_POINTERS, *PEXCEPTION_POINTERS;

typedef struct _SCOPE_TABLE
{
    DWORD Count;
    struct
    {
        DWORD BeginAddress;
        DWORD EndAddress;
        DWORD HandlerAddress;
        DWORD JumpTarget;
    } ScopeRecord[1];
} SCOPE_TABLE, *PSCOPE_TABLE;

/* DISPATCHER_CONTEXT (amd64): opaque pointers kept as PVOID to preserve the
 * field offsets the handler reads (ControlPc@0 ImageBase@8 TargetIp@32
 * ContextRecord@40 HandlerData@56 HistoryTable@64 ScopeIndex@72). */
typedef struct _DISPATCHER_CONTEXT
{
    ULONG64 ControlPc;
    ULONG64 ImageBase;
    PVOID   FunctionEntry;
    ULONG64 EstablisherFrame;
    ULONG64 TargetIp;
    struct _CONTEXT *ContextRecord;
    PVOID   LanguageHandler;
    PVOID   HandlerData;
    PVOID   HistoryTable;
    ULONG   ScopeIndex;
    ULONG   Fill0;
} DISPATCHER_CONTEXT, *PDISPATCHER_CONTEXT;

typedef LONG (*PEXCEPTION_FILTER)(struct _EXCEPTION_POINTERS *ExceptionPointers,
                                  PVOID EstablisherFrame);
typedef VOID (*PTERMINATION_HANDLER)(BOOLEAN AbnormalTermination,
                                     PVOID EstablisherFrame);

NTSYSAPI VOID RtlUnwindEx(PVOID TargetFrame, PVOID TargetIp,
                          struct _EXCEPTION_RECORD *ExceptionRecord,
                          PVOID ReturnValue, struct _CONTEXT *ContextRecord,
                          PVOID HistoryTable);
NTSYSAPI VOID RtlUnwind(PVOID TargetFrame, PVOID TargetIp,
                        struct _EXCEPTION_RECORD *ExceptionRecord,
                        PVOID ReturnValue);


EXCEPTION_DISPOSITION
__cdecl
__C_specific_handler(
    struct _EXCEPTION_RECORD *ExceptionRecord,
    void *EstablisherFrame,
    struct _CONTEXT *ContextRecord,
    struct _DISPATCHER_CONTEXT *DispatcherContext)
{
    PSCOPE_TABLE ScopeTable;
    ULONG i, BeginAddress, EndAddress, Handler;
    ULONG64 ImageBase, JumpTarget, IpOffset, TargetIpOffset;
    EXCEPTION_POINTERS ExceptionPointers;
    PTERMINATION_HANDLER TerminationHandler;
    PEXCEPTION_FILTER ExceptionFilter;
    LONG FilterResult;

    /* Set up the EXCEPTION_POINTERS */
    ExceptionPointers.ExceptionRecord = ExceptionRecord;
    ExceptionPointers.ContextRecord = ContextRecord;

    /* Get the image base */
    ImageBase = (ULONG64)DispatcherContext->ImageBase;

    /* Get the image base relative instruction pointers */
    IpOffset = DispatcherContext->ControlPc - ImageBase;
    TargetIpOffset = DispatcherContext->TargetIp - ImageBase;

    /* Get the scope table and current index */
    ScopeTable = (PSCOPE_TABLE)DispatcherContext->HandlerData;

    /* Loop while we have scope table entries */
    while (DispatcherContext->ScopeIndex < ScopeTable->Count)
    {
        /* Use i as index and update the dispatcher context */
        i = DispatcherContext->ScopeIndex++;

        /* Get the start and end of the scrope */
        BeginAddress = ScopeTable->ScopeRecord[i].BeginAddress;
        EndAddress = ScopeTable->ScopeRecord[i].EndAddress;

        /* Skip this scope if we are not within the bounds */
        if ((IpOffset < BeginAddress) || (IpOffset >= EndAddress))
        {
            continue;
        }

        /* Check if this is an unwind */
        if (ExceptionRecord->ExceptionFlags & EXCEPTION_UNWIND)
        {
            /* Check if this is a target unwind */
            if (ExceptionRecord->ExceptionFlags & EXCEPTION_TARGET_UNWIND)
            {
                /* Check if the target is within the scope itself */
                if ((TargetIpOffset >= BeginAddress) &&
                    (TargetIpOffset <  EndAddress))
                {
                    return ExceptionContinueSearch;
                }
            }

            /* Check if this is a termination handler / finally function */
            if (ScopeTable->ScopeRecord[i].JumpTarget == 0)
            {
                /* Call the handler */
                Handler = ScopeTable->ScopeRecord[i].HandlerAddress;
                TerminationHandler = (PTERMINATION_HANDLER)(ImageBase + Handler);
                TerminationHandler(TRUE, EstablisherFrame);
            }
            else if (ScopeTable->ScopeRecord[i].JumpTarget == TargetIpOffset)
            {
                return ExceptionContinueSearch;
            }
        }
        else
        {
            /* We are only unterested in exception handlers */
            if (ScopeTable->ScopeRecord[i].JumpTarget == 0)
            {
                continue;
            }

            /* This is an exception filter, get the handler address */
            Handler = ScopeTable->ScopeRecord[i].HandlerAddress;

            /* Check for hardcoded EXCEPTION_EXECUTE_HANDLER */
            if (Handler == EXCEPTION_EXECUTE_HANDLER)
            {
                /* This is our result */
                FilterResult = EXCEPTION_EXECUTE_HANDLER;
            }
            else
            {
                /* Otherwise we need to call the handler */
                ExceptionFilter = (PEXCEPTION_FILTER)(ImageBase + Handler);
                FilterResult = ExceptionFilter(&ExceptionPointers, EstablisherFrame);
            }

            if (FilterResult < 0 /* EXCEPTION_CONTINUE_EXECUTION */)
            {
                return ExceptionContinueExecution;
            }

            if (FilterResult > 0 /* EXCEPTION_EXECUTE_HANDLER */)
            {
                JumpTarget = (ImageBase + ScopeTable->ScopeRecord[i].JumpTarget);

                /* Unwind to the target address (This does not return) */
                RtlUnwindEx(EstablisherFrame,
                            (PVOID)JumpTarget,
                            ExceptionRecord,
                            UlongToPtr(ExceptionRecord->ExceptionCode),
                            DispatcherContext->ContextRecord,
                            DispatcherContext->HistoryTable);

                /* Should not get here */
                __debugbreak();
            }
        }
    }

    /* Reached the end of the scope table */
    return ExceptionContinueSearch;
}

void __cdecl _local_unwind(void* frame, void* target)
{
    RtlUnwind(frame, target, NULL, 0);
}
