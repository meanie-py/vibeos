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
 * raised one frame down in the callee: the same contract as advapi32's EventLog shim.
 * i386 keeps the plain call: clang's i386 native SEH is unreliable, and the DUMMY_PSEH
 * original never caught anything there either.
 */

#include <windef.h>
#include <winbase.h>
#include <objbase.h>

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
    object->lpVtbl->Release(object);
#endif
    return TRUE;
}
