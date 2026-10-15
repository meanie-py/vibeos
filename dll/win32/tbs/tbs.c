/*
 * Trusted Platform Module Base Services
 *
 * Copyright 2021 Alex Henrie
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

#include "windef.h"
#include "tbs.h"
#include "wine/debug.h"

WINE_DEFAULT_DEBUG_CHANNEL(tbs);

TBS_RESULT WINAPI Tbsi_Context_Create(const TBS_CONTEXT_PARAMS *params, TBS_HCONTEXT *out)
{
    FIXME("(%p, %p) stub\n", params, out);
    return TBS_E_TPM_NOT_FOUND;
}

TBS_RESULT WINAPI Tbsi_GetDeviceInfo(UINT32 size, void *info)
{
    FIXME("(%u, %p) stub\n", size, info);
    return TBS_E_TPM_NOT_FOUND;
}

HRESULT WINAPI GetDeviceIDString(WCHAR *out, UINT32 size, UINT32 *used, BOOL *tpm)
{
    FIXME("(%p, %u, %p, %p) stub\n", out, size, used, tpm);
    return E_NOTIMPL;
}

/* ReactOS: Tbsi_Context_Create never succeeds (there is no TPM), so no valid
 * context can reach these, and TBS_E_INVALID_CONTEXT is what Windows answers
 * for a context it did not hand out. */
TBS_RESULT WINAPI Tbsip_Context_Close(TBS_HCONTEXT context)
{
    FIXME("(%p) stub\n", context);
    return TBS_E_INVALID_CONTEXT;
}

TBS_RESULT WINAPI Tbsip_Cancel_Commands(TBS_HCONTEXT context)
{
    FIXME("(%p) stub\n", context);
    return TBS_E_INVALID_CONTEXT;
}

TBS_RESULT WINAPI Tbsip_Submit_Command(TBS_HCONTEXT context, TBS_COMMAND_LOCALITY locality,
                                       TBS_COMMAND_PRIORITY priority, const BYTE *command,
                                       UINT32 command_size, BYTE *result, UINT32 *result_size)
{
    FIXME("(%p, %u, %u, %p, %u, %p, %p) stub\n", context, locality, priority, command,
          command_size, result, result_size);
    return TBS_E_INVALID_CONTEXT;
}
