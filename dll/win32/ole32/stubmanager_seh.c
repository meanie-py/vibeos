/*
 * stubmanager_seh.c — the guarded object release ole32-rust's stub manager needs.
 *
 * Wine's stub_manager_delete() releases the server object inside
 * __TRY / __EXCEPT_PAGE_FAULT: "some broken apps crash in object destructors", and
 * Windows XP+ catches and ignores those faults (Wine's marshal test asserts it with an
 * object whose Release raises an access violation). Under the project-wide DUMMY_PSEH
 * the guard compiled to nothing, so the Rust port kept a plain call, and a server
 * whose object faulted on its last release died with it.
 *
 * Rust has no __try, so the guard lives here, in a native-SEH TU (_USE_NATIVE_SEH on
 * amd64/arm64, set in CMakeLists.txt). The try body is only the call, so the fault is
 * raised one frame down in the callee.
 * clang's i386 native SEH is unreliable, so i386 goes through ntdll's RtlSehTryExcept
 * (ntoskrnl/sehtry.c, built on the fs:[0] chain), which catches hardware faults too.
 */

#include <windef.h>
#include <winbase.h>
#include <objbase.h>

#if !defined(_USE_NATIVE_SEH)
/* ntdll's trampoline (dll/ntdll/def/ntdll.spec): TRUE when Filter accepted an exception. */
unsigned char __cdecl RtlSehTryExcept(void (__cdecl *Body)(void *), void *BodyContext,
                                      long (__cdecl *Filter)(void *, void *),
                                      void *FilterContext, long *ExceptionCode);

static void __cdecl release_body(void *context)
{
    IUnknown *object = context;
    object->lpVtbl->Release(object);
}

static long __cdecl access_violation_only(void *context, void *pointers)
{
    return ((EXCEPTION_POINTERS *)pointers)->ExceptionRecord->ExceptionCode == EXCEPTION_ACCESS_VIOLATION
               ? EXCEPTION_EXECUTE_HANDLER
               : EXCEPTION_CONTINUE_SEARCH;
}
#endif

/* Returns FALSE when the release faulted (and the fault was swallowed). */
BOOL __cdecl ole32_release_object_guarded(IUnknown *object)
{
#if defined(_USE_NATIVE_SEH)
    __try
    {
        object->lpVtbl->Release(object);
    }
    __except (GetExceptionCode() == EXCEPTION_ACCESS_VIOLATION ? EXCEPTION_EXECUTE_HANDLER
                                                             : EXCEPTION_CONTINUE_SEARCH)
    {
        return FALSE;
    }
#else
    long code = 0;

    if (RtlSehTryExcept(release_body, object, access_violation_only, NULL, &code))
        return FALSE;
#endif
    return TRUE;
}
