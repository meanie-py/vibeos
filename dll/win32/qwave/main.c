/*
 * Copyright 2018 Louis Lenders
 * Copyright 2019 Vijay Kiran Kamuju
 *
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Lesser General Public
 * License as published by the Free Software Foundation; either
 * version 2.1 of the License, or (at your option) any later version.
 *
 * This library is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 * Lesser General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public
 * License along with this library; if not, write to the Free Software
 * Foundation, Inc., 51 Franklin St, Fifth Floor, Boston, MA 02110-1301, USA
 */

#include <stdarg.h>

#include "windef.h"
#include "winbase.h"
#include "qos2.h"
#include <stdlib.h>
#include "wine/debug.h"

WINE_DEFAULT_DEBUG_CHANNEL(qwave);

/* ReactOS: a QoS handle is a registered token. Windows hands one out whenever the
 * version is 1.0 and validates it on every later call; Wine fails the create, so
 * nothing past it could be reached. Flows are still not supported. */
struct qos_handle
{
    struct qos_handle *next;
};

static struct qos_handle *qos_handles;
static LONG qos_lock;

/* QoS handles are created and closed rarely; a spinlock keeps the table free of
 * static CRITICAL_SECTION initializers. */
static void lock_handles(void)
{
    while (InterlockedCompareExchange(&qos_lock, 1, 0)) SwitchToThread();
}

static void unlock_handles(void)
{
    InterlockedExchange(&qos_lock, 0);
}

static BOOL is_valid_handle(HANDLE handle)
{
    struct qos_handle *qos;

    lock_handles();
    for (qos = qos_handles; qos; qos = qos->next)
        if ((HANDLE)qos == handle) break;
    unlock_handles();
    return qos != NULL;
}

/* The answer for a flow call: no such handle, or a handle with no flow support. */
static BOOL flow_unsupported(HANDLE handle)
{
    SetLastError(is_valid_handle(handle) ? ERROR_NOT_SUPPORTED : ERROR_INVALID_HANDLE);
    return FALSE;
}

BOOL WINAPI QOSCreateHandle(PQOS_VERSION version, PHANDLE handle)
{
    struct qos_handle *qos;

    TRACE("%p %p\n", version, handle);
    if (!version || !((version->MajorVersion == 1) && (version->MinorVersion == 0)) || !handle)
    {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    if (!(qos = malloc(sizeof(*qos))))
    {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return FALSE;
    }
    lock_handles();
    qos->next = qos_handles;
    qos_handles = qos;
    unlock_handles();
    *handle = (HANDLE)qos;
    return TRUE;
}

BOOL WINAPI QOSAddSocketToFlow(HANDLE handle, SOCKET socket, PSOCKADDR addr,
                               QOS_TRAFFIC_TYPE traffictype, DWORD flags, PQOS_FLOWID flowid)
{
    FIXME("%p, %Ix, %p, %d, 0x%08lx, %p stub!\n", handle, socket, addr, traffictype, flags, flowid);
    return flow_unsupported(handle);
}

BOOL WINAPI QOSCloseHandle(HANDLE handle)
{
    struct qos_handle *qos, **link;

    TRACE("%p\n", handle);

    lock_handles();
    for (link = &qos_handles; (qos = *link); link = &qos->next)
        if ((HANDLE)qos == handle)
        {
            *link = qos->next;
            break;
        }
    unlock_handles();
    if (!qos)
    {
        SetLastError(ERROR_INVALID_HANDLE);
        return FALSE;
    }
    free(qos);
    return TRUE;
}

/* ReactOS: no flow is ever created, so these answer as QOSAddSocketToFlow does. */
BOOL WINAPI QOSRemoveSocketFromFlow(HANDLE handle, SOCKET socket, QOS_FLOWID flowid, DWORD flags)
{
    FIXME("%p, %Ix, %lu, 0x%08lx stub!\n", handle, socket, flowid, flags);
    return flow_unsupported(handle);
}

BOOL WINAPI QOSQueryFlow(HANDLE handle, QOS_FLOWID flowid, QOS_QUERY_FLOW op, PULONG size,
                         PVOID buffer, DWORD flags, LPOVERLAPPED overlap)
{
    FIXME("%p, %lu, %d, %p, %p, 0x%08lx, %p stub!\n", handle, flowid, op, size, buffer, flags, overlap);
    return flow_unsupported(handle);
}

BOOL WINAPI QOSSetFlow(HANDLE handle, QOS_FLOWID flowid, QOS_SET_FLOW op, ULONG size,
                       PVOID buffer, DWORD flags, LPOVERLAPPED overlap)
{
    FIXME("%p, %lu, %d, %lu, %p, 0x%08lx, %p stub!\n", handle, flowid, op, size, buffer, flags, overlap);
    return flow_unsupported(handle);
}
