/*
 * RpcServerInqCallAttributes{A,W}
 *
 * Query attributes of the RPC call the calling server thread is dispatching
 * (RPC_CALL_ATTRIBUTES_V1/V2/V3, see MS docs rpcasync.h). Wine leaves both as
 * stubs. Everything here is derived from state this runtime really keeps on
 * the server connection (RpcBinding / RpcConnection); what it does not track
 * is reported with the documented "not available" value and listed below.
 *
 * Filled from real state:
 *   AuthenticationLevel/Service  connection's inquire_auth_client hook (same
 *                                source as RpcBindingInqAuthClientEx); a call
 *                                with no auth is reported as LEVEL_NONE / AUTHN_NONE
 *   ProtocolSequence             the binding's protseq name
 *   IsClientLocal                Local for ncalrpc, else "unknown locality"
 *                                (we do not compare the peer address)
 *   ClientPID                    ncalrpc only (connection's inquire_client_pid),
 *                                only when RPC_QUERY_CLIENT_PID is requested
 *   InterfaceUuid                conn->ActiveInterface of the connection
 *   ServerPrincipalName          AuthInfo->server_principal_name when present
 *   CallStatus / CallType        IN_PROGRESS / rctNormal (the caller is, by
 *                                construction, inside the server routine)
 * NOT tracked by this runtime (left at the documented empty value):
 *   ClientPrincipalName          length set to 0 ("protocol sequence does not
 *                                support retrieving"); no SID->name lookup here
 *   NullSession                  FALSE (anonymous sessions are not distinguished)
 *   KernelModeCaller             FALSE
 *   OpNum                        0 (the dispatcher keeps msg->ProcNum on its own stack only)
 *   CallLocalAddress             BufferSize 0, AddressFormat rlafInvalid
 *   V3 ClientIdentifier          length 0
 *
 * Errors: RPC_S_INVALID_ARG (NULL struct, Version outside 1..3, V2-only flags
 * on a V1 struct, CALL_LOCAL_ADDRESS without a CallLocalAddress), RPC_S_NO_CALL_ACTIVE
 * (no binding given and thread has no current call), RPC_S_INVALID_BINDING
 * (not a server-side binding), RPC_S_CANNOT_SUPPORT (ncadg_*, per MS docs),
 * ERROR_INVALID_PARAMETER (name length nonzero with NULL buffer),
 * ERROR_MORE_DATA (name buffer too small; required size written back, other
 * fields still filled).
 *
 * Copyright: LGPL 2.1+, in the style of the neighbouring Wine-derived sources.
 */

#include <stdarg.h>

#include "rpc.h"
#include "rpcndr.h"
#include "rpcasync.h"

#include "wine/debug.h"

#include "rpc_binding.h"
#include "rpc_message.h"

WINE_DEFAULT_DEBUG_CHANNEL(rpc);

/* The SDK header in this tree lacks RPC_CALL_ATTRIBUTES, so use a private
 * V3 layout (V1 and V2 are exact prefixes of it; A and W differ only in the
 * pointee type of the two name pointers, so one layout serves both). */
#define CA_QUERY_SERVER_PRINCIPAL_NAME 0x02
#define CA_QUERY_CLIENT_PRINCIPAL_NAME 0x04
#define CA_QUERY_CALL_LOCAL_ADDRESS    0x08
#define CA_QUERY_CLIENT_PID            0x10

#define CA_CALL_STATUS_IN_PROGRESS 1
#define CA_CALL_TYPE_NORMAL        1   /* rctNormal */
#define CA_LOCALITY_LOCAL          1   /* rcclLocal */
#define CA_LOCALITY_UNKNOWN        3   /* rcclClientUnknownLocality */

typedef struct
{
    unsigned int  Version;
    ULONG         Flags;
    ULONG         ServerPrincipalNameBufferLength;
    void         *ServerPrincipalName;
    ULONG         ClientPrincipalNameBufferLength;
    void         *ClientPrincipalName;
    ULONG         AuthenticationLevel;
    ULONG         AuthenticationService;
    BOOL          NullSession;
    /* ---- end of V1 ---- */
    BOOL          KernelModeCaller;
    ULONG         ProtocolSequence;
    ULONG         IsClientLocal;
    HANDLE        ClientPID;
    ULONG         CallStatus;
    ULONG         CallType;
    struct { unsigned int Version; void *Buffer; ULONG BufferSize; ULONG AddressFormat; } *CallLocalAddress;
    unsigned short OpNum;
    UUID          InterfaceUuid;
    /* ---- end of V2 ---- */
    ULONG         ClientIdentifierBufferLength;
    unsigned char *ClientIdentifier;
} rpc_call_attrs;

/* Copy a wide name into the caller's buffer using the MS length protocol. */
static RPC_STATUS fill_name(const WCHAR *src, ULONG *buflen, void *buf, BOOL unicode)
{
    ULONG needed;

    if (*buflen && !buf)
        return ERROR_INVALID_PARAMETER;

    if (!src)
    {
        *buflen = 0;
        return RPC_S_OK;
    }

    if (unicode)
        needed = (lstrlenW(src) + 1) * sizeof(WCHAR);
    else
    {
        int n = WideCharToMultiByte(CP_ACP, 0, src, -1, NULL, 0, NULL, NULL);
        if (n <= 0) return GetLastError() ? GetLastError() : ERROR_INVALID_PARAMETER;
        needed = n;
    }

    if (*buflen < needed)
    {
        *buflen = needed;
        return ERROR_MORE_DATA;
    }

    if (unicode)
        memcpy(buf, src, needed);
    else if (!WideCharToMultiByte(CP_ACP, 0, src, -1, buf, needed, NULL, NULL))
        return GetLastError();
    *buflen = needed;
    return RPC_S_OK;
}

static ULONG protseq_to_id(const char *protseq)
{
    if (!protseq) return 0;
    if (!strcmp(protseq, "ncacn_ip_tcp")) return RPC_PROTSEQ_TCP;
    if (!strcmp(protseq, "ncacn_np"))     return RPC_PROTSEQ_NMP;
    if (!strcmp(protseq, "ncalrpc"))      return RPC_PROTSEQ_LRPC;
    if (!strcmp(protseq, "ncacn_http"))   return RPC_PROTSEQ_HTTP;
    return 0;
}

static RPC_STATUS inq_call_attributes(RPC_BINDING_HANDLE ClientBinding, void *RpcCallAttributes, BOOL unicode)
{
    rpc_call_attrs *attrs = RpcCallAttributes;
    RpcBinding *bind = ClientBinding;
    RpcConnection *conn;
    RPC_STATUS status, ret = RPC_S_OK;
    unsigned int version;
    ULONG flags;

    if (!attrs) return RPC_S_INVALID_ARG;

    version = attrs->Version;
    flags = attrs->Flags;
    if (version < 1 || version > 3) return RPC_S_INVALID_ARG;
    /* V2-only queries cannot be honoured in the smaller V1 struct (and would
     * write past it), so refuse them there. */
    if (version < 2 && (flags & (CA_QUERY_CALL_LOCAL_ADDRESS | CA_QUERY_CLIENT_PID)))
        return RPC_S_INVALID_ARG;
    if (version >= 2 && (flags & CA_QUERY_CALL_LOCAL_ADDRESS) && !attrs->CallLocalAddress)
        return RPC_S_INVALID_ARG;

    if (!bind) bind = RPCRT4_GetThreadCurrentCallHandle();
    if (!bind) return ClientBinding ? RPC_S_INVALID_BINDING : RPC_S_NO_CALL_ACTIVE;
    if (!bind->server || !(conn = bind->FromConn)) return RPC_S_INVALID_BINDING;

    if (bind->Protseq && !strncmp(bind->Protseq, "ncadg_", 6))
        return RPC_S_CANNOT_SUPPORT;

    /* Names. Server name only if the connection recorded one; client name is
     * not tracked, so the documented "unsupported" result (length 0) applies. */
    if (flags & CA_QUERY_SERVER_PRINCIPAL_NAME)
    {
        const WCHAR *name = conn->AuthInfo ? conn->AuthInfo->server_principal_name : NULL;
        status = fill_name(name, &attrs->ServerPrincipalNameBufferLength, attrs->ServerPrincipalName, unicode);
        if (status == ERROR_MORE_DATA) ret = status;
        else if (status) return status;
    }
    if (flags & CA_QUERY_CLIENT_PRINCIPAL_NAME)
    {
        status = fill_name(NULL, &attrs->ClientPrincipalNameBufferLength, attrs->ClientPrincipalName, unicode);
        if (status) return status;
    }

    attrs->AuthenticationLevel = RPC_C_AUTHN_LEVEL_NONE;
    attrs->AuthenticationService = RPC_C_AUTHN_NONE;
    status = rpcrt4_conn_inquire_auth_client(conn, NULL, NULL, &attrs->AuthenticationLevel,
                                             &attrs->AuthenticationService, NULL, 0);
    if (status == RPC_S_BINDING_HAS_NO_AUTH)
    {
        attrs->AuthenticationLevel = RPC_C_AUTHN_LEVEL_NONE;
        attrs->AuthenticationService = RPC_C_AUTHN_NONE;
    }
    else if (status)
        return status;
    attrs->NullSession = FALSE;

    if (version >= 2)
    {
        BOOL is_lrpc = bind->Protseq && !strcmp(bind->Protseq, "ncalrpc");

        attrs->KernelModeCaller = FALSE;
        attrs->ProtocolSequence = protseq_to_id(bind->Protseq);
        attrs->IsClientLocal = is_lrpc ? CA_LOCALITY_LOCAL : CA_LOCALITY_UNKNOWN;
        attrs->ClientPID = NULL;
        if ((flags & CA_QUERY_CLIENT_PID) && is_lrpc && conn->ops->inquire_client_pid)
        {
            ULONG pid = 0;
            status = conn->ops->inquire_client_pid(conn, &pid);
            if (status) return status;
            attrs->ClientPID = ULongToHandle(pid);
        }
        attrs->CallStatus = CA_CALL_STATUS_IN_PROGRESS;
        attrs->CallType = CA_CALL_TYPE_NORMAL;
        if ((flags & CA_QUERY_CALL_LOCAL_ADDRESS))
        {
            attrs->CallLocalAddress->BufferSize = 0;   /* local address not tracked */
            attrs->CallLocalAddress->AddressFormat = 0; /* rlafInvalid */
        }
        attrs->OpNum = 0; /* not tracked */
        attrs->InterfaceUuid = conn->ActiveInterface.SyntaxGUID;
    }
    if (version >= 3)
        attrs->ClientIdentifierBufferLength = 0;

    return ret;
}

/***********************************************************************
 *             RpcServerInqCallAttributesW (RPCRT4.@)
 */
RPC_STATUS RPC_ENTRY RpcServerInqCallAttributesW(RPC_BINDING_HANDLE ClientBinding, void *RpcCallAttributes)
{
    TRACE("(%p, %p)\n", ClientBinding, RpcCallAttributes);
    return inq_call_attributes(ClientBinding, RpcCallAttributes, TRUE);
}

/***********************************************************************
 *             RpcServerInqCallAttributesA (RPCRT4.@)
 */
RPC_STATUS RPC_ENTRY RpcServerInqCallAttributesA(RPC_BINDING_HANDLE ClientBinding, void *RpcCallAttributes)
{
    TRACE("(%p, %p)\n", ClientBinding, RpcCallAttributes);
    return inq_call_attributes(ClientBinding, RpcCallAttributes, FALSE);
}
