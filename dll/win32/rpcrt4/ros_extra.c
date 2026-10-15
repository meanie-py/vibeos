
#include <stdarg.h>

#include "rpc.h"
#include "rpcndr.h"
#include "rpcasync.h"

#include "wine/debug.h"

#include "rpc_binding.h"
#include "rpc_message.h"
#include "ndr_stubless.h"

WINE_DEFAULT_DEBUG_CHANNEL(rpc);

/***********************************************************************
 *           RpcGetAuthorizationContextForClient [RPCRT4.@]
 *
 * Called by RpcFreeAuthorizationContext to return the Authz context.
 *
 * PARAMS
 *  ClientBinding        [I] Binding handle, represents a binding to a client on the server.
 *  ImpersonateOnReturn  [I] Directs this function to be represented the client on return.
 *  Reserved1            [I] Reserved, equal to null.
 *  expiration_time      [I] Points to the exact date and time when the token expires.
 *  Reserved2            [I] Reserved, equal to a LUID structure which has a members,
 *                           each of them is set to zero.
 *  Reserved3            [I] Reserved, equal to zero.
 *  Reserved4            [I] Reserved, equal to null.
 *  authz_client_context [I] Points to an AUTHZ_CLIENT_CONTEXT_HANDLE structure
 *                           that has direct pass to Authz functions.
 *
 * RETURNS
 *  Success: RPC_S_OK.
 *  Failure: Any error code.
 */
RPC_STATUS
WINAPI
RpcGetAuthorizationContextForClient(RPC_BINDING_HANDLE ClientBinding,
                                    BOOL ImpersonateOnReturn,
                                    void * Reserved1,
                                    PLARGE_INTEGER expiration_time,
                                    LUID Reserved2,
                                    DWORD Reserved3,
                                    PVOID Reserved4,
                                    PVOID *authz_client_context)
{
    FIXME("(%p, %d, %p, %p, (%d, %u), %u, %p, %p): stub\n",
          ClientBinding,
          ImpersonateOnReturn,
          Reserved1,
          expiration_time,
          Reserved2.HighPart,
          Reserved2.LowPart,
          Reserved3,
          Reserved4,
          authz_client_context);
    return RPC_S_NO_CONTEXT_AVAILABLE;
}

/***********************************************************************
 *           I_RpcOpenClientProcess (rpcrt4.@)
 *
 * Undocumented. Opens a handle on the process that made the current server call. This tree
 * keeps no record of the client process for a call, so there is nothing to open: the output
 * handle is NULL and RPC_S_CANNOT_SUPPORT is returned. Wine has no body for this export, and
 * the parameter list is inferred from how the export is used, so it is unverified.
 */
RPC_STATUS WINAPI I_RpcOpenClientProcess(void *binding, DWORD desired_access, HANDLE *client_process)
{
    TRACE("(%p, %#lx, %p)\n", binding, desired_access, client_process);

    if (client_process)
        *client_process = NULL;
    return RPC_S_CANNOT_SUPPORT;
}
