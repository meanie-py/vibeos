/*
 * PROJECT:     ReactOS system libraries
 * LICENSE:     GNU GPL - See COPYING in the top level directory
 * PURPOSE:     Setjmp-based PSEH for i386 Clang (llvm-mingw)
 * PROGRAMMER:  ReactOS ARM64 port
 *
 * This implements a "always-catch" SEH handler for i386 Clang builds.
 * Instead of evaluating exception filters in the handler context (where
 * Clang's ESP-relative locals are wrong), we longjmp back to the original
 * function context and evaluate the filter there.
 */

#include <stdarg.h>
#include <windef.h>
#include <winnt.h>

/* Import from pseh_clang_asm.S */
void __cdecl _PsehLongjmp(void *frame, int value);
void __cdecl _PsehCallRtlUnwind(void *frame);

/*
 * Frame layout - first 8 bytes match EXCEPTION_REGISTRATION_RECORD:
 *   +0  Prev (void*)
 *   +4  Handler (void*)
 */

/*
 * _PsehExceptionHandler - Registered on fs:[0] as the SEH handler.
 *
 * On unwind calls: return ExceptionContinueSearch (let RtlUnwind proceed).
 * On first-chance exceptions: copy the EXCEPTION_RECORD, call RtlUnwind
 * to fire intermediate FINALLY blocks, then longjmp back to setjmp point.
 *
 * Signature matches pseh2.h declaration: int(void*,void*,void*,void*)
 */
int
__cdecl
_PsehExceptionHandler(
    void *ExceptionRecordPtr,
    void *EstablisherFrame,
    void *ContextRecordPtr,
    void *DispatcherContext)
{
    struct _EXCEPTION_RECORD *ExceptionRecord = (struct _EXCEPTION_RECORD *)ExceptionRecordPtr;

    /* During unwind, just continue the search */
    if (ExceptionRecord->ExceptionFlags & (EXCEPTION_UNWINDING | EXCEPTION_EXIT_UNWIND))
    {
        return ExceptionContinueSearch;
    }

    /* Frame layout: +0 Prev, +4 Handler, +8 SavedEbp, ... +32 ExceptionCode, +36 ExceptionRecordCopy, +116 ExceptionPointersCopy */
    char *frame = (char *)EstablisherFrame;

    /* Save the exception code at offset 32 */
    *(DWORD *)(frame + 32) = ExceptionRecord->ExceptionCode;

    /* Copy the full EXCEPTION_RECORD (80 bytes) at offset 36 */
    RtlCopyMemory(frame + 36, ExceptionRecord, sizeof(EXCEPTION_RECORD));

    /* Set up ExceptionPointers copy at offset 116: .ExceptionRecord points to our copy, .ContextRecord = NULL */
    *(void **)(frame + 116) = (void *)(frame + 36);   /* ExceptionRecord ptr */
    *(void **)(frame + 120) = NULL;                     /* ContextRecord ptr */

    /* Call RtlUnwind to fire intermediate FINALLY handlers on frames
       between the current fs:[0] head and our frame */
    _PsehCallRtlUnwind(EstablisherFrame);

    /* longjmp back to the setjmp point with value=1 */
    _PsehLongjmp(EstablisherFrame, 1);

    /* Should not be reached */
    return ExceptionContinueSearch;
}

/*
 * _PsehPopFrame - Remove our frame from the fs:[0] SEH chain.
 *
 * If our frame is the head of the chain, simple pop.
 * Otherwise walk the chain to find and unlink it (handles goto-out-of-nested-try).
 *
 * Uses raw pointer math since EXCEPTION_REGISTRATION_RECORD layout is:
 *   +0 Next (void*), +4 Handler (void*)
 */
void
__cdecl
_PsehPopFrame(void *framePtr)
{
    void **frame = (void **)framePtr;        /* frame[0] = Prev/Next */
    void **current;

    current = (void **)(ULONG_PTR)__readfsdword(0);

    if (current == frame)
    {
        /* Simple case: we're at the head */
        __writefsdword(0, (DWORD)(ULONG_PTR)frame[0]);
        return;
    }

    /* Walk the chain to find the link pointing to our frame */
    while ((void **)current[0] != frame)
    {
        current = (void **)current[0];
        /* Safety: if we hit the end without finding our frame, bail */
        if (current == (void **)(ULONG_PTR)-1)
            return;
    }

    /* Unlink our frame */
    current[0] = frame[0];
}

/*
 * _PsehReRaise - Re-raise a caught exception to propagate to outer handlers.
 *
 * Called from _SEH2_END when _pseh_abnormal is still set, meaning:
 * - _SEH2_FINALLY: exception occurred, finally body ran, need to propagate
 * - _SEH2_EXCEPT: exception occurred but filter rejected, need to propagate
 *
 * Uses the saved EXCEPTION_RECORD copy at frame offset 36.
 *
 * Instead of calling RtlRaiseException (which requires ntdll, not linked by
 * all user-mode binaries), we manually walk the fs:[0] SEH chain and call
 * each handler directly — same as RtlDispatchException does internally.
 * When a handler catches (via longjmp), execution never returns here.
 */
typedef int (__cdecl *_PSEH_HANDLER_FUNC)(void *, void *, void *, void *);

void
__cdecl
_PsehReRaise(void *framePtr)
{
    EXCEPTION_RECORD *Record = (EXCEPTION_RECORD *)(((char *)framePtr) + 36);
    void **current;

    /* Zero the chain pointer to prevent infinite loops */
    Record->ExceptionRecord = NULL;

    /* Walk the fs:[0] SEH chain and dispatch to each handler */
    current = (void **)(ULONG_PTR)__readfsdword(0);

    while (current != (void **)(ULONG_PTR)-1)
    {
        _PSEH_HANDLER_FUNC handler = (_PSEH_HANDLER_FUNC)current[1];
        void *next = current[0];
        ULONG DispatcherContext = 0;

        /* Call the handler. For PSEH_CLANG handlers (_PsehExceptionHandler),
         * this will longjmp and never return. For other handlers (e.g., kernel
         * default), they return ExceptionContinueSearch (1) to skip. */
        int result = handler(
            (void *)Record,     /* ExceptionRecord */
            (void *)current,    /* EstablisherFrame */
            NULL,               /* ContextRecord (unused by PSEH_CLANG handlers) */
            &DispatcherContext  /* DispatcherContext */
        );

        if (result != 1 /* ExceptionContinueSearch */)
        {
            /* Handler handled it — shouldn't reach here for PSEH_CLANG
             * (longjmp doesn't return), but break for safety */
            break;
        }

        current = (void **)next;
    }

    /* If we reach here, no handler caught the exception.
     * This shouldn't happen in practice — there's always a kernel
     * or process-level handler at the end of the chain. */
}
