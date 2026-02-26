/*
 * PROJECT:     ReactOS Run-Time Library
 * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
 * PURPOSE:     User-mode exception support for AMD64
 * COPYRIGHT:   Copyright 2018-2021 Timo Kreuzer <timo.kreuzer@reactos.org>
 */

/* INCLUDES *****************************************************************/

#include <rtl.h>
#define NDEBUG
#include <debug.h>

/* PUBLIC FUNCTIONS **********************************************************/

/*
* @unimplemented
*/
PVOID
NTAPI
RtlpGetExceptionAddress(VOID)
{
    UNIMPLEMENTED;
    return NULL;
}

BOOLEAN
NTAPI
RtlpUnwindInternal(
    _In_opt_ PVOID TargetFrame,
    _In_opt_ PVOID TargetIp,
    _In_ PEXCEPTION_RECORD ExceptionRecord,
    _In_ PVOID ReturnValue,
    _In_ PCONTEXT ContextRecord,
    _In_opt_ struct _UNWIND_HISTORY_TABLE *HistoryTable,
    _In_ ULONG Flags);

/*
 * @unimplemented
 */
BOOLEAN
NTAPI
RtlDispatchException(
    _In_ PEXCEPTION_RECORD ExceptionRecord,
    _In_ PCONTEXT ContextRecord)
{
    BOOLEAN Handled;

    /*
     * Perform vectored exception handling for user mode.
     * In kernel mode (ntoskrnl), skip VEH — the VEH lock uses
     * NtCurrentTeb() which is not valid in kernel context and causes
     * a recursive page fault → stack overflow → double fault.
     */
#ifndef _NTOSKRNL_
    if (RtlCallVectoredExceptionHandlers(ExceptionRecord, ContextRecord))
    {
        /* Exception handled, now call vectored continue handlers */
        RtlCallVectoredContinueHandlers(ExceptionRecord, ContextRecord);

        /* Continue execution */
        return TRUE;
    }
#endif

    /* Call the internal unwind routine */
    Handled = RtlpUnwindInternal(NULL, // TargetFrame
                                 NULL, // TargetIp
                                 ExceptionRecord,
                                 0, // ReturnValue
                                 ContextRecord,
                                 NULL, // HistoryTable
                                 UNW_FLAG_EHANDLER);

#ifndef _NTOSKRNL_
    /* In user mode, call any registered vectored continue handlers */
    RtlCallVectoredContinueHandlers(ExceptionRecord, ContextRecord);
#endif

    return Handled;
}
