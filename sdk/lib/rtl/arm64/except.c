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

/* PUBLIC FUNCTIONS **********************************************************/

BOOLEAN
NTAPI
RtlDispatchException(
    _In_ PEXCEPTION_RECORD ExceptionRecord,
    _In_ PCONTEXT Context)
{
    UNIMPLEMENTED;
    return FALSE;
}

VOID
NTAPI
RtlInitializeContext(
    IN HANDLE ProcessHandle,
    OUT PCONTEXT ThreadContext,
    IN PVOID ThreadStartParam  OPTIONAL,
    IN PTHREAD_START_ROUTINE ThreadStartAddress,
    IN PINITIAL_TEB StackBase)
{
    /* Initialize everything to 0 */
    RtlZeroMemory(ThreadContext, sizeof(*ThreadContext));

    /* Set the entry point (PC) and stack pointer */
    ThreadContext->Pc = (ULONG64)ThreadStartAddress;
    ThreadContext->Sp = (ULONG64)StackBase;

    /* ARM64 SP must be 16-byte aligned */
    ThreadContext->Sp &= ~(ULONG64)15;

    /* Set first argument (X0) to the start parameter */
    ThreadContext->X0 = (ULONG64)ThreadStartParam;

    /* CPSR for user mode: EL0t (M=0), all interrupts enabled (DAIF=0) */
    ThreadContext->Cpsr = 0;

    /* Set context flags */
    ThreadContext->ContextFlags = CONTEXT_CONTROL | CONTEXT_INTEGER;
}

PRUNTIME_FUNCTION
NTAPI
RtlLookupFunctionEntry(
    _In_ ULONG64 ControlPc,
    _Out_ PULONG64 ImageBase,
    _Inout_opt_ PVOID HistoryTable)
{
    UNIMPLEMENTED;
    *ImageBase = 0;
    return NULL;
}

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
    UNIMPLEMENTED;
    *HandlerData = NULL;
    *EstablisherFrame = 0;
    return NULL;
}

VOID
NTAPI
RtlUnwind(
    _In_opt_ PVOID TargetFrame,
    _In_opt_ PVOID TargetIp,
    _In_opt_ PEXCEPTION_RECORD ExceptionRecord,
    _In_ PVOID ReturnValue)
{
    UNIMPLEMENTED;
}

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
    UNIMPLEMENTED;
}

VOID
NTAPI
RtlRestoreContext(
    _In_ PCONTEXT ContextRecord,
    _In_opt_ PEXCEPTION_RECORD ExceptionRecord)
{
    UNIMPLEMENTED;
}

BOOLEAN
NTAPI
RtlAddFunctionTable(
    _In_ PRUNTIME_FUNCTION FunctionTable,
    _In_ ULONG EntryCount,
    _In_ ULONG64 BaseAddress)
{
    UNIMPLEMENTED;
    return TRUE;
}

BOOLEAN
NTAPI
RtlDeleteFunctionTable(
    _In_ PRUNTIME_FUNCTION FunctionTable)
{
    UNIMPLEMENTED;
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
    UNIMPLEMENTED;
    return TRUE;
}

EXCEPTION_DISPOSITION
__cdecl
__C_specific_handler(
    _In_ struct _EXCEPTION_RECORD *ExceptionRecord,
    _In_ void *EstablisherFrame,
    _Inout_ struct _CONTEXT *ContextRecord,
    _Inout_ struct _DISPATCHER_CONTEXT *DispatcherContext)
{
    UNIMPLEMENTED;
    return ExceptionContinueSearch;
}

/* EOF */
