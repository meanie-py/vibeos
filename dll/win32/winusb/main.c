/*
 * Copyright (C) 2022 Mohamad Al-Jaf
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
#include "winerror.h"
#include "usb.h"
#include "usbdi.h"
#include "winusb.h"

#include "wine/debug.h"

WINE_DEFAULT_DEBUG_CHANNEL(winusb);

/* Declarations the PSDK winusb.h does not carry (only WinUsb_Free is there). */
typedef struct _WINUSB_SETUP_PACKET
{
    UCHAR  RequestType;
    UCHAR  Request;
    USHORT Value;
    USHORT Index;
    USHORT Length;
} WINUSB_SETUP_PACKET, *PWINUSB_SETUP_PACKET;

typedef struct _WINUSB_PIPE_INFORMATION
{
    USBD_PIPE_TYPE PipeType;
    UCHAR          PipeId;
    USHORT         MaximumPacketSize;
    UCHAR          Interval;
} WINUSB_PIPE_INFORMATION, *PWINUSB_PIPE_INFORMATION;

/*
 * vibeos has no winusb.sys function driver, so no device can be bound to WinUSB.
 * WinUsb_Initialize therefore cannot produce an interface handle, and every
 * function that takes one fails with ERROR_INVALID_HANDLE because no valid
 * interface handle can exist. Wine has no bodies for these exports (its
 * winusb.spec marks them all as stubs), so the honest failures are written here.
 */
static BOOL winusb_no_interface(void)
{
    SetLastError(ERROR_INVALID_HANDLE);
    return FALSE;
}

/***********************************************************************
 *           WinUsb_AbortPipe (winusb.@)
 */
BOOL WINAPI WinUsb_AbortPipe(WINUSB_INTERFACE_HANDLE handle, UCHAR pipe_id)
{
    TRACE("(%p, %#x)\n", handle, pipe_id);
    return winusb_no_interface();
}

/***********************************************************************
 *           WinUsb_ControlTransfer (winusb.@)
 */
BOOL WINAPI WinUsb_ControlTransfer(WINUSB_INTERFACE_HANDLE handle, WINUSB_SETUP_PACKET setup,
                                   PUCHAR buffer, ULONG length, PULONG transferred, LPOVERLAPPED overlapped)
{
    TRACE("(%p, %p, %p, %lu, %p, %p)\n", handle, &setup, buffer, length, transferred, overlapped);
    return winusb_no_interface();
}

/***********************************************************************
 *           WinUsb_FlushPipe (winusb.@)
 */
BOOL WINAPI WinUsb_FlushPipe(WINUSB_INTERFACE_HANDLE handle, UCHAR pipe_id)
{
    TRACE("(%p, %#x)\n", handle, pipe_id);
    return winusb_no_interface();
}

/***********************************************************************
 *           WinUsb_Free (winusb.@)
 */
BOOL WINAPI WinUsb_Free(WINUSB_INTERFACE_HANDLE handle)
{
    TRACE("(%p)\n", handle);
    return winusb_no_interface();
}

/***********************************************************************
 *           WinUsb_GetAssociatedInterface (winusb.@)
 */
BOOL WINAPI WinUsb_GetAssociatedInterface(WINUSB_INTERFACE_HANDLE handle, UCHAR index,
                                          PWINUSB_INTERFACE_HANDLE associated)
{
    TRACE("(%p, %#x, %p)\n", handle, index, associated);
    if (associated) *associated = NULL;
    return winusb_no_interface();
}

/***********************************************************************
 *           WinUsb_GetCurrentAlternateSetting (winusb.@)
 */
BOOL WINAPI WinUsb_GetCurrentAlternateSetting(WINUSB_INTERFACE_HANDLE handle, PUCHAR setting)
{
    TRACE("(%p, %p)\n", handle, setting);
    return winusb_no_interface();
}

/***********************************************************************
 *           WinUsb_GetDescriptor (winusb.@)
 */
BOOL WINAPI WinUsb_GetDescriptor(WINUSB_INTERFACE_HANDLE handle, UCHAR type, UCHAR index,
                                 USHORT language_id, PUCHAR buffer, ULONG length, PULONG transferred)
{
    TRACE("(%p, %#x, %#x, %#x, %p, %lu, %p)\n", handle, type, index, language_id, buffer,
          length, transferred);
    return winusb_no_interface();
}

/***********************************************************************
 *           WinUsb_GetOverlappedResult (winusb.@)
 */
BOOL WINAPI WinUsb_GetOverlappedResult(WINUSB_INTERFACE_HANDLE handle, LPOVERLAPPED overlapped,
                                       LPDWORD transferred, BOOL wait)
{
    TRACE("(%p, %p, %p, %d)\n", handle, overlapped, transferred, wait);
    return winusb_no_interface();
}

/***********************************************************************
 *           WinUsb_GetPipePolicy (winusb.@)
 */
BOOL WINAPI WinUsb_GetPipePolicy(WINUSB_INTERFACE_HANDLE handle, UCHAR pipe_id, ULONG type,
                                 PULONG length, PVOID value)
{
    TRACE("(%p, %#x, %lu, %p, %p)\n", handle, pipe_id, type, length, value);
    return winusb_no_interface();
}

/***********************************************************************
 *           WinUsb_GetPowerPolicy (winusb.@)
 */
BOOL WINAPI WinUsb_GetPowerPolicy(WINUSB_INTERFACE_HANDLE handle, ULONG type, PULONG length,
                                  PVOID value)
{
    TRACE("(%p, %lu, %p, %p)\n", handle, type, length, value);
    return winusb_no_interface();
}

/***********************************************************************
 *           WinUsb_Initialize (winusb.@)
 */
BOOL WINAPI WinUsb_Initialize(HANDLE device, PWINUSB_INTERFACE_HANDLE interface_handle)
{
    TRACE("(%p, %p)\n", device, interface_handle);
    /* No winusb.sys function driver exists, so there is no interface to hand back. */
    if (interface_handle) *interface_handle = NULL;
    SetLastError(ERROR_NOT_SUPPORTED);
    return FALSE;
}

/***********************************************************************
 *           WinUsb_QueryDeviceInformation (winusb.@)
 */
BOOL WINAPI WinUsb_QueryDeviceInformation(WINUSB_INTERFACE_HANDLE handle, ULONG type,
                                          PULONG length, PVOID buffer)
{
    TRACE("(%p, %lu, %p, %p)\n", handle, type, length, buffer);
    return winusb_no_interface();
}

/***********************************************************************
 *           WinUsb_QueryInterfaceSettings (winusb.@)
 */
BOOL WINAPI WinUsb_QueryInterfaceSettings(WINUSB_INTERFACE_HANDLE handle, UCHAR alt_setting,
                                          PUSB_INTERFACE_DESCRIPTOR descriptor)
{
    TRACE("(%p, %#x, %p)\n", handle, alt_setting, descriptor);
    return winusb_no_interface();
}

/***********************************************************************
 *           WinUsb_QueryPipe (winusb.@)
 */
BOOL WINAPI WinUsb_QueryPipe(WINUSB_INTERFACE_HANDLE handle, UCHAR alt_setting, UCHAR index,
                             PWINUSB_PIPE_INFORMATION info)
{
    TRACE("(%p, %#x, %#x, %p)\n", handle, alt_setting, index, info);
    return winusb_no_interface();
}

/***********************************************************************
 *           WinUsb_ReadPipe (winusb.@)
 */
BOOL WINAPI WinUsb_ReadPipe(WINUSB_INTERFACE_HANDLE handle, UCHAR pipe_id, PUCHAR buffer,
                            ULONG length, PULONG transferred, LPOVERLAPPED overlapped)
{
    TRACE("(%p, %#x, %p, %lu, %p, %p)\n", handle, pipe_id, buffer, length, transferred, overlapped);
    return winusb_no_interface();
}

/***********************************************************************
 *           WinUsb_ResetPipe (winusb.@)
 */
BOOL WINAPI WinUsb_ResetPipe(WINUSB_INTERFACE_HANDLE handle, UCHAR pipe_id)
{
    TRACE("(%p, %#x)\n", handle, pipe_id);
    return winusb_no_interface();
}

/***********************************************************************
 *           WinUsb_SetCurrentAlternateSetting (winusb.@)
 */
BOOL WINAPI WinUsb_SetCurrentAlternateSetting(WINUSB_INTERFACE_HANDLE handle, UCHAR setting)
{
    TRACE("(%p, %#x)\n", handle, setting);
    return winusb_no_interface();
}

/***********************************************************************
 *           WinUsb_SetPipePolicy (winusb.@)
 */
BOOL WINAPI WinUsb_SetPipePolicy(WINUSB_INTERFACE_HANDLE handle, UCHAR pipe_id, ULONG type,
                                 ULONG length, PVOID value)
{
    TRACE("(%p, %#x, %lu, %lu, %p)\n", handle, pipe_id, type, length, value);
    return winusb_no_interface();
}

/***********************************************************************
 *           WinUsb_SetPowerPolicy (winusb.@)
 */
BOOL WINAPI WinUsb_SetPowerPolicy(WINUSB_INTERFACE_HANDLE handle, ULONG type, ULONG length,
                                  PVOID value)
{
    TRACE("(%p, %lu, %lu, %p)\n", handle, type, length, value);
    return winusb_no_interface();
}

/***********************************************************************
 *           WinUsb_WritePipe (winusb.@)
 */
BOOL WINAPI WinUsb_WritePipe(WINUSB_INTERFACE_HANDLE handle, UCHAR pipe_id, PUCHAR buffer,
                             ULONG length, PULONG transferred, LPOVERLAPPED overlapped)
{
    TRACE("(%p, %#x, %p, %lu, %p, %p)\n", handle, pipe_id, buffer, length, transferred, overlapped);
    return winusb_no_interface();
}
