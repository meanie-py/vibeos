/*
 * rpcss_seh.c — the guarded irpcss calls ole32-rust needs.
 *
 * wine-10's combase wraps every rpcss call in __TRY / __EXCEPT(rpc_filter): an RPC
 * client stub reports failure by RAISING (e.g. RPC_S_SERVER_UNAVAILABLE before rpcss
 * is up), and the filter turns that into an HRESULT the caller can retry on. Rust
 * has no __try, so the call lives here in a native-SEH TU (_USE_NATIVE_SEH on
 * amd64/arm64, set in CMakeLists.txt) with a calls-only try body. clang's i386 native
 * SEH is unreliable, so i386 goes through ntdll's RtlSehTryExcept (ntoskrnl/sehtry.c,
 * built on the fs:[0] chain) instead. It was a plain call there, which is how advapi32's
 * identical EventLog thunk let an RPC raise kill csrss (BUGCHECK 0xF4, 2026-10-17).
 */

#include <windef.h>
#include <winbase.h>
#include <rpc.h>
#include <objbase.h>
#include "irpcss.h"

static RPC_BINDING_HANDLE get_irpcss_handle(void)
{
    static RPC_BINDING_HANDLE irpcss_handle;

    if (!irpcss_handle)
    {
        unsigned short protseq[] = IRPCSS_PROTSEQ;
        unsigned short endpoint[] = IRPCSS_ENDPOINT;
        RPC_BINDING_HANDLE new_handle = NULL;
        RPC_WSTR binding;

        if (RpcStringBindingComposeW(NULL, protseq, NULL, endpoint, NULL, &binding) == RPC_S_OK)
        {
            RpcBindingFromStringBindingW(binding, &new_handle);
            RpcStringFreeW(&binding);
        }
        if (InterlockedCompareExchangePointer(&irpcss_handle, new_handle, NULL))
            /* another thread beat us to it */
            RpcBindingFree(&new_handle);
    }
    return irpcss_handle;
}

#if !defined(_USE_NATIVE_SEH)
/* ntdll's trampoline (dll/ntdll/def/ntdll.spec): TRUE when Filter accepted an exception. */
unsigned char __cdecl RtlSehTryExcept(void (__cdecl *Body)(void *), void *BodyContext,
                                      long (__cdecl *Filter)(void *, void *),
                                      void *FilterContext, long *ExceptionCode);

struct seqid_call
{
    RPC_BINDING_HANDLE handle;
    DWORD *id;
    HRESULT hr;
};

static void __cdecl seqid_body(void *context)
{
    struct seqid_call *call = context;
    call->hr = irpcss_get_thread_seq_id(call->handle, call->id);
}

static long __cdecl seqid_filter(void *context, void *pointers)
{
    return I_RpcExceptionFilter(((EXCEPTION_POINTERS *)pointers)->ExceptionRecord->ExceptionCode);
}
#endif

/* wine-10 rpcss_get_next_seqid, minus the start-rpcss retry (the Rust caller owns it). */
HRESULT __cdecl ole32_rpcss_get_next_seqid(DWORD *id)
{
    RPC_BINDING_HANDLE handle = get_irpcss_handle();
    HRESULT hr;

#if defined(_USE_NATIVE_SEH)
    __try
    {
        hr = irpcss_get_thread_seq_id(handle, id);
    }
    __except (I_RpcExceptionFilter(GetExceptionCode()))
    {
        hr = HRESULT_FROM_WIN32(GetExceptionCode());
    }
#else
    struct seqid_call call = { handle, id, S_OK };
    long code = 0;

    if (RtlSehTryExcept(seqid_body, &call, seqid_filter, NULL, &code))
        hr = HRESULT_FROM_WIN32(code);
    else
        hr = call.hr;
#endif
    return hr;
}
