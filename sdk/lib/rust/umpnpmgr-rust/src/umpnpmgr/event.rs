// rustconv-assembly: whole-tu
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
unsafe extern "C" fn ProcessTargetDeviceEvent(
    PnpEvent: PPLUGPLAY_EVENT_BLOCK) {
    let mut RpcStatus: RPC_STATUS = 0;

    // DPRINT removed

    if UuidEqual(&mut (*PnpEvent).EventGuid, &GUID_DEVICE_ARRIVAL as *const GUID as *mut UUID, &mut RpcStatus) != 0
    {
        // DPRINT removed

        SendMessageW(HWND_BROADCAST, (WM_DEVICECHANGE) as u32, DBT_DEVNODES_CHANGED as WPARAM, 0);
    }
    else if UuidEqual(&mut (*PnpEvent).EventGuid, &GUID_DEVICE_EJECT_VETOED as *const GUID as *mut UUID, &mut RpcStatus) != 0
    {
        // DPRINT removed
    }
    else if UuidEqual(&mut (*PnpEvent).EventGuid, &GUID_DEVICE_KERNEL_INITIATED_EJECT as *const GUID as *mut UUID, &mut RpcStatus) != 0
    {
        // DPRINT removed
    }
    else if UuidEqual(&mut (*PnpEvent).EventGuid, &GUID_DEVICE_SAFE_REMOVAL as *const GUID as *mut UUID, &mut RpcStatus) != 0
    {
        // DPRINT removed

        SendMessageW(HWND_BROADCAST, (WM_DEVICECHANGE) as u32, DBT_DEVNODES_CHANGED as WPARAM, 0);
    }
    else if UuidEqual(&mut (*PnpEvent).EventGuid, &GUID_DEVICE_SURPRISE_REMOVAL as *const GUID as *mut UUID, &mut RpcStatus) != 0
    {
        // DPRINT removed

        SendMessageW(HWND_BROADCAST, (WM_DEVICECHANGE) as u32, DBT_DEVNODES_CHANGED as WPARAM, 0);
    }
    else if UuidEqual(&mut (*PnpEvent).EventGuid, &GUID_DEVICE_REMOVAL_VETOED as *const GUID as *mut UUID, &mut RpcStatus) != 0
    {
        // DPRINT removed
    }
    else if UuidEqual(&mut (*PnpEvent).EventGuid, &GUID_DEVICE_REMOVE_PENDING as *const GUID as *mut UUID, &mut RpcStatus) != 0
    {
        // DPRINT removed
    }
    else
    {
        // DPRINT removed
    }
}
unsafe extern "C" fn ProcessDeviceClassChangeEvent(
    PnpEvent: PPLUGPLAY_EVENT_BLOCK,
) {
    let mut RpcStatus: RPC_STATUS = 0;
    let mut Current: PLIST_ENTRY;
    let mut pNotifyData: PNOTIFY_ENTRY;
    let mut pEventData: PDEV_BROADCAST_DEVICEINTERFACE_W;
    let mut dwSize: DWORD;
    let mut dwEventType: DWORD;

    // DPRINT removed
    // DPRINT removed
    // DPRINT removed

    RtlAcquireResourceShared(&mut NotificationListLock, (TRUE) as u8);

    Current = NotificationListHead.Flink;
    while Current != &mut NotificationListHead {
        pNotifyData = ((Current as *const u8).offset(-(core::mem::offset_of!(NOTIFY_ENTRY, ListEntry) as isize))) as PNOTIFY_ENTRY;
        if ((*pNotifyData).dwType == CLASS_NOTIFICATION) &&
            (((*pNotifyData).ulFlags & (DEVICE_NOTIFY_ALL_INTERFACE_CLASSES) as u32) != 0 ||
             UuidEqual(
                 core::ptr::addr_of_mut!((*PnpEvent).__bindgen_anon_1.DeviceClass.ClassGuid),
                 core::ptr::addr_of_mut!((*pNotifyData).ClassGuid),
                 &mut RpcStatus,
             ) != 0)
        {
            if UuidEqual(
                core::ptr::addr_of_mut!((*PnpEvent).EventGuid),
                &GUID_DEVICE_INTERFACE_ARRIVAL as *const GUID as *mut UUID,
                &mut RpcStatus,
            ) != 0
            {
                // DPRINT removed
                dwEventType = (DBT_DEVICEARRIVAL) as u32;
            } else if UuidEqual(
                core::ptr::addr_of_mut!((*PnpEvent).EventGuid),
                &GUID_DEVICE_INTERFACE_REMOVAL as *const GUID as *mut UUID,
                &mut RpcStatus,
            ) != 0
            {
                // DPRINT removed
                dwEventType = (DBT_DEVICEREMOVECOMPLETE) as u32;
            } else {
                dwEventType = 0;
            }

            dwSize = core::mem::size_of::<DEV_BROADCAST_DEVICEINTERFACE_W>() as DWORD + (wcslen((*PnpEvent).__bindgen_anon_1.DeviceClass.SymbolicLinkName.as_ptr()) * (core::mem::size_of::<WCHAR>() as DWORD) as usize) as u32;
            pEventData = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, dwSize as SIZE_T) as PDEV_BROADCAST_DEVICEINTERFACE_W;
            if !pEventData.is_null() {
                (*pEventData).dbcc_size = dwSize;
                (*pEventData).dbcc_devicetype = (DBT_DEVTYP_DEVICEINTERFACE) as u32;
                CopyMemory!(
                    core::ptr::addr_of_mut!((*pEventData).dbcc_classguid) as LPVOID,
                    core::ptr::addr_of_mut!((*PnpEvent).__bindgen_anon_1.DeviceClass.ClassGuid) as LPVOID,
                    core::mem::size_of::<GUID>(),
                );
                wcscpy((*pEventData).dbcc_name.as_mut_ptr(), (*PnpEvent).__bindgen_anon_1.DeviceClass.SymbolicLinkName.as_ptr());
            }

            if ((*pNotifyData).ulFlags & (DEVICE_NOTIFY_SERVICE_HANDLE) as u32) == (DEVICE_NOTIFY_WINDOW_HANDLE) as u32 {
                SendMessageW(
                    ((*pNotifyData).hRecipient as HANDLE).cast(),
                    (WM_DEVICECHANGE) as u32,
                    dwEventType as WPARAM,
                    pEventData as LPARAM,
                );
            } else if ((*pNotifyData).ulFlags & (DEVICE_NOTIFY_SERVICE_HANDLE) as u32) == (DEVICE_NOTIFY_SERVICE_HANDLE) as u32 {
                I_ScSendPnPMessage(
                    (*pNotifyData).hRecipient as SERVICE_STATUS_HANDLE,
                    (SERVICE_CONTROL_DEVICEEVENT) as u32,
                    dwEventType,
                    pEventData as LPVOID,
                );
            }

            if !pEventData.is_null() {
                HeapFree(GetProcessHeap(), 0, pEventData as LPVOID);
            }
        }

        Current = (*Current).Flink;
    }

    RtlReleaseResource(&mut NotificationListLock);
}
unsafe extern "C" fn ProcessDeviceInstallEvent(
    PnpEvent: PPLUGPLAY_EVENT_BLOCK,
) {
    let mut Params: *mut DeviceInstallParams;
    let mut len: u32;
    let mut DeviceIdLength: u32;

    DeviceIdLength = (lstrlenW((*PnpEvent).__bindgen_anon_1.InstallDevice.DeviceId.as_ptr())) as u32;
    if DeviceIdLength != 0 {
        len = (FIELD_OFFSET!(DeviceInstallParams, DeviceIds) + ((DeviceIdLength + 1) * core::mem::size_of::<u16>() as u32) as i32) as u32;
        Params = HeapAlloc(GetProcessHeap(), 0, len as SIZE_T) as *mut DeviceInstallParams;
        if !Params.is_null() {
            wcscpy((*Params).DeviceIds.as_mut_ptr(), (*PnpEvent).__bindgen_anon_1.InstallDevice.DeviceId.as_ptr());

            WaitForSingleObject(hDeviceInstallListMutex, (INFINITE) as u32);
            InsertTailList(&mut DeviceInstallListHead, core::ptr::addr_of_mut!((*Params).ListEntry));
            ReleaseMutex(hDeviceInstallListMutex);

            SetEvent(hDeviceInstallListNotEmpty);

            SendMessageW(HWND_BROADCAST, (WM_DEVICECHANGE) as u32, DBT_DEVNODES_CHANGED as WPARAM, 0);
        }
    }
}
#[no_mangle]
pub unsafe extern "system" fn PnpEventThread(
    lpParameter: LPVOID,
) -> u32 {
    // event.c:241 is `PLUGPLAY_CONTROL_USER_RESPONSE_DATA ResponseData = {0, 0, 0, 0};` — a
    // POSITIONAL zero initializer. The translation named the members instead, and invented the
    // names (the real struct is Unknown1..Unknown4). Note `Unknown4` happens to line up, which
    // is what makes a partial invention worse than a total one: three of four would have been
    // caught, and the fourth would have looked reviewed.
    let mut ResponseData: PLUGPLAY_CONTROL_USER_RESPONSE_DATA = core::mem::zeroed();
    let mut dwRet: u32 = ERROR_SUCCESS;
    let mut Status: i32;
    let mut PnpEvent: PPLUGPLAY_EVENT_BLOCK;
    let mut NewPnpEvent: PPLUGPLAY_EVENT_BLOCK;
    let mut PnpEventSize: u32;

    UNREFERENCED_PARAMETER!(lpParameter);

    PnpEventSize = 0x1000;
    PnpEvent = HeapAlloc(GetProcessHeap(), 0, PnpEventSize as SIZE_T) as PPLUGPLAY_EVENT_BLOCK;
    if PnpEvent.is_null() {
        return ERROR_OUTOFMEMORY;
    }

    loop {
        // DPRINT removed

        Status = NtGetPlugPlayEvent(0, 0, PnpEvent, PnpEventSize);

        if Status == STATUS_BUFFER_TOO_SMALL {
            PnpEventSize += 0x400;
            NewPnpEvent = HeapReAlloc(GetProcessHeap(), 0, PnpEvent as LPVOID, PnpEventSize as SIZE_T) as PPLUGPLAY_EVENT_BLOCK;
            if NewPnpEvent.is_null() {
                dwRet = ERROR_OUTOFMEMORY;
                break;
            }
            PnpEvent = NewPnpEvent;
            continue;
        }

        if !NT_SUCCESS(Status) {
            // DPRINT removed
            break;
        }

        // DPRINT removed
        match (*PnpEvent).EventCategory {
            TargetDeviceChangeEvent => {
                ProcessTargetDeviceEvent(PnpEvent);
            }

            DeviceClassChangeEvent => {
                ProcessDeviceClassChangeEvent(PnpEvent);
            }

            DeviceInstallEvent => {
                ProcessDeviceInstallEvent(PnpEvent);
            }

            _ => {
                // DPRINT removed
            }
        }

        Status = NtPlugPlayControl(PlugPlayControlUserResponse,
                                   &mut ResponseData as *mut _ as LPVOID,
                                   core::mem::size_of::<PLUGPLAY_CONTROL_USER_RESPONSE_DATA>() as u32);
        if !NT_SUCCESS(Status) {
            // DPRINT removed
            break;
        }
    }

    HeapFree(GetProcessHeap(), 0, PnpEvent as LPVOID);

    return dwRet;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  *  ReactOS kernel
//  *  Copyright (C) 2005 ReactOS Team
//  *
//  *  This program is free software; you can redistribute it and/or modify
//  *  it under the terms of the GNU General Public License as published by
//  *  the Free Software Foundation; either version 2 of the License, or
//  *  (at your option) any later version.
//  *
//  *  This program is distributed in the hope that it will be useful,
//  *  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  *  GNU General Public License for more details.
//  *
//  *  You should have received a copy of the GNU General Public License along
//  *  with this program; if not, write to the Free Software Foundation, Inc.,
//  *  51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
//  */
// /*
//  * COPYRIGHT:        See COPYING in the top level directory
//  * PROJECT:          ReactOS kernel
//  * FILE:             base/services/umpnpmgr/event.c
//  * PURPOSE:          PNP Event thread
//  * PROGRAMMER:       Eric Kohl (eric.kohl@reactos.org)
//  *                   Hervé Poussineau (hpoussin@reactos.org)
//  *                   Colin Finck (colin@reactos.org)
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include "precomp.h"
//
// #define NDEBUG
// #include <debug.h>
//
// /* FUNCTIONS *****************************************************************/
//
// static
// VOID
// ProcessTargetDeviceEvent(
//      PPLUGPLAY_EVENT_BLOCK PnpEvent)
// {
//     RPC_STATUS RpcStatus;
//
//     DPRINT("ProcessTargetDeviceEvent(%p)\n", PnpEvent);
//
//     if (UuidEqual(&PnpEvent->EventGuid, (UUID*)&GUID_DEVICE_ARRIVAL, &RpcStatus))
//     {
// //        DWORD dwRecipient;
//
//         DPRINT("Device arrival: %S\n", PnpEvent->TargetDevice.DeviceIds.as_ptr());
//
// //       dwRecipient = BSM_ALLDESKTOPS | BSM_APPLICATIONS;
// //       BroadcastSystemMessageW(BSF_POSTMESSAGE,
// //                               &dwRecipient,
// //                               WM_DEVICECHANGE,
// //                               DBT_DEVNODES_CHANGED,
// //                               0);
//         SendMessageW(HWND_BROADCAST, WM_DEVICECHANGE, DBT_DEVNODES_CHANGED, 0);
//     }
//     else if (UuidEqual(&PnpEvent->EventGuid, (UUID*)&GUID_DEVICE_EJECT_VETOED, &RpcStatus))
//     {
//         DPRINT1("Eject vetoed: %S\n", PnpEvent->TargetDevice.DeviceIds.as_ptr());
//     }
//     else if (UuidEqual(&PnpEvent->EventGuid, (UUID*)&GUID_DEVICE_KERNEL_INITIATED_EJECT, &RpcStatus))
//     {
//         DPRINT1("Kernel initiated eject: %S\n", PnpEvent->TargetDevice.DeviceIds.as_ptr());
//     }
//     else if (UuidEqual(&PnpEvent->EventGuid, (UUID*)&GUID_DEVICE_SAFE_REMOVAL, &RpcStatus))
//     {
// //        DWORD dwRecipient;
//
//         DPRINT1("Safe removal: %S\n", PnpEvent->TargetDevice.DeviceIds.as_ptr());
//
// //        dwRecipient = BSM_ALLDESKTOPS | BSM_APPLICATIONS;
// //        BroadcastSystemMessageW(BSF_POSTMESSAGE,
// //                                &dwRecipient,
// //                                WM_DEVICECHANGE,
// //                                DBT_DEVNODES_CHANGED,
// //                                0);
//         SendMessageW(HWND_BROADCAST, WM_DEVICECHANGE, DBT_DEVNODES_CHANGED, 0);
//     }
//     else if (UuidEqual(&PnpEvent->EventGuid, (UUID*)&GUID_DEVICE_SURPRISE_REMOVAL, &RpcStatus))
//     {
// //        DWORD dwRecipient;
//
//         DPRINT1("Surprise removal: %S\n", PnpEvent->TargetDevice.DeviceIds.as_ptr());
//
// //        dwRecipient = BSM_ALLDESKTOPS | BSM_APPLICATIONS;
// //        BroadcastSystemMessageW(BSF_POSTMESSAGE,
// //                                &dwRecipient,
// //                                WM_DEVICECHANGE,
// //                                DBT_DEVNODES_CHANGED,
// //                                0);
//         SendMessageW(HWND_BROADCAST, WM_DEVICECHANGE, DBT_DEVNODES_CHANGED, 0);
//     }
//     else if (UuidEqual(&PnpEvent->EventGuid, (UUID*)&GUID_DEVICE_REMOVAL_VETOED, &RpcStatus))
//     {
//         DPRINT1("Removal vetoed: %S\n", PnpEvent->TargetDevice.DeviceIds.as_ptr());
//     }
//     else if (UuidEqual(&PnpEvent->EventGuid, (UUID*)&GUID_DEVICE_REMOVE_PENDING, &RpcStatus))
//     {
//         DPRINT1("Removal pending: %S\n", PnpEvent->TargetDevice.DeviceIds.as_ptr());
//     }
//     else
//     {
//         DPRINT1("Unknown event, GUID {%08X-%04X-%04X-%02X%02X-%02X%02X%02X%02X%02X%02X}\n",
//                 PnpEvent->EventGuid.Data1, PnpEvent->EventGuid.Data2, PnpEvent->EventGuid.Data3,
//                 PnpEvent->EventGuid.Data4[0], PnpEvent->EventGuid.Data4[1], PnpEvent->EventGuid.Data4[2],
//                 PnpEvent->EventGuid.Data4[3], PnpEvent->EventGuid.Data4[4], PnpEvent->EventGuid.Data4[5],
//                 PnpEvent->EventGuid.Data4[6], PnpEvent->EventGuid.Data4[7]);
//     }
// }
//
//
// static
// VOID
// ProcessDeviceClassChangeEvent(
//      PPLUGPLAY_EVENT_BLOCK PnpEvent)
// {
//     RPC_STATUS RpcStatus;
//     PLIST_ENTRY Current;
//     PNOTIFY_ENTRY pNotifyData;
//     PDEV_BROADCAST_DEVICEINTERFACE_W pEventData;
//     DWORD dwSize;
//     DWORD dwEventType;
//
//     DPRINT("ProcessDeviceClassChangeEvent(%p)\n", PnpEvent);
//     DPRINT("SymbolicLink: %S\n", PnpEvent->DeviceClass.SymbolicLinkName.as_ptr());
//     DPRINT("ClassGuid: {%08X-%04X-%04X-%02X%02X-%02X%02X%02X%02X%02X%02X}\n",
//            PnpEvent->DeviceClass.ClassGuid.Data1, PnpEvent->DeviceClass.ClassGuid.Data2, PnpEvent->DeviceClass.ClassGuid.Data3,
//            PnpEvent->DeviceClass.ClassGuid.Data4[0], PnpEvent->DeviceClass.ClassGuid.Data4[1], PnpEvent->DeviceClass.ClassGuid.Data4[2],
//            PnpEvent->DeviceClass.ClassGuid.Data4[3], PnpEvent->DeviceClass.ClassGuid.Data4[4], PnpEvent->DeviceClass.ClassGuid.Data4[5],
//            PnpEvent->DeviceClass.ClassGuid.Data4[6], PnpEvent->DeviceClass.ClassGuid.Data4[7]);
//
//     RtlAcquireResourceShared(&NotificationListLock, TRUE);
//
//     Current = NotificationListHead.Flink;
//     while (Current != &NotificationListHead)
//     {
//         pNotifyData = CONTAINING_RECORD(Current, NOTIFY_ENTRY, ListEntry);
//         if ((pNotifyData->dwType == CLASS_NOTIFICATION) &&
//             ((pNotifyData->ulFlags & DEVICE_NOTIFY_ALL_INTERFACE_CLASSES) ||
//              (UuidEqual(&PnpEvent->DeviceClass.ClassGuid, &pNotifyData->ClassGuid, &RpcStatus))))
//         {
//             if (UuidEqual(&PnpEvent->EventGuid, (UUID*)&GUID_DEVICE_INTERFACE_ARRIVAL, &RpcStatus))
//             {
//                 DPRINT("Interface arrival: %S\n", PnpEvent->DeviceClass.SymbolicLinkName.as_ptr());
//                 dwEventType = DBT_DEVICEARRIVAL;
//             }
//             else if (UuidEqual(&PnpEvent->EventGuid, (UUID*)&GUID_DEVICE_INTERFACE_REMOVAL, &RpcStatus))
//             {
//                 DPRINT("Interface removal: %S\n", PnpEvent->DeviceClass.SymbolicLinkName.as_ptr());
//                 dwEventType = DBT_DEVICEREMOVECOMPLETE;
//             }
//             else
//             {
//                 dwEventType = 0;
//             }
//
//             dwSize = sizeof(DEV_BROADCAST_DEVICEINTERFACE_W) + wcslen(PnpEvent->DeviceClass.SymbolicLinkName.as_ptr()) * sizeof(WCHAR);
//             pEventData = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, dwSize);
//             if (pEventData)
//             {
//                 pEventData->dbcc_size = dwSize;
//                 pEventData->dbcc_devicetype = DBT_DEVTYP_DEVICEINTERFACE;
//                 CopyMemory!(&pEventData->dbcc_classguid, &PnpEvent->DeviceClass.ClassGuid, sizeof(GUID));
//                 wcscpy(pEventData->dbcc_name, PnpEvent->DeviceClass.SymbolicLinkName.as_ptr());
//             }
//
//             if ((pNotifyData->ulFlags & DEVICE_NOTIFY_SERVICE_HANDLE) == DEVICE_NOTIFY_WINDOW_HANDLE)
//             {
//                 SendMessageW((HANDLE)pNotifyData->hRecipient, WM_DEVICECHANGE, (WPARAM)dwEventType, (LPARAM)pEventData);
//             }
//             else if ((pNotifyData->ulFlags & DEVICE_NOTIFY_SERVICE_HANDLE) == DEVICE_NOTIFY_SERVICE_HANDLE)
//             {
//                 I_ScSendPnPMessage((SERVICE_STATUS_HANDLE)pNotifyData->hRecipient,
//                                    SERVICE_CONTROL_DEVICEEVENT,
//                                    dwEventType,
//                                    pEventData);
//             }
//
//             if (pEventData)
//                 HeapFree(GetProcessHeap(), 0, pEventData);
//         }
//
//         Current = Current->Flink;
//     }
//
//     RtlReleaseResource(&NotificationListLock);
// }
//
//
// static
// VOID
// ProcessDeviceInstallEvent(
//      PPLUGPLAY_EVENT_BLOCK PnpEvent)
// {
//     DeviceInstallParams* Params;
//     DWORD len;
//     DWORD DeviceIdLength;
// //    DWORD dwRecipient;
//
//     DPRINT("ProcessDeviceInstallEvent(%p)\n", PnpEvent);
//     DPRINT("Device enumerated: %S\n", PnpEvent->InstallDevice.DeviceId);
//
//     DeviceIdLength = lstrlenW(PnpEvent->InstallDevice.DeviceId);
//     if (DeviceIdLength)
//     {
//         /* Allocate a new device-install event */
//         len = FIELD_OFFSET!(DeviceInstallParams, DeviceIds) + (DeviceIdLength + 1) * sizeof(WCHAR);
//         Params = HeapAlloc(GetProcessHeap(), 0, len);
//         if (Params)
//         {
//             wcscpy(Params->DeviceIds, PnpEvent->InstallDevice.DeviceId);
//
//             /* Queue the event (will be dequeued by DeviceInstallThread) */
//             WaitForSingleObject(hDeviceInstallListMutex, INFINITE);
//             InsertTailList(&DeviceInstallListHead, &Params->ListEntry);
//             ReleaseMutex(hDeviceInstallListMutex);
//
//             SetEvent(hDeviceInstallListNotEmpty);
//
// //            dwRecipient = BSM_ALLDESKTOPS | BSM_APPLICATIONS;
// //            BroadcastSystemMessageW(BSF_POSTMESSAGE,
// //                                    &dwRecipient,
// //                                    WM_DEVICECHANGE,
// //                                    DBT_DEVNODES_CHANGED,
// //                                    0);
//             SendMessageW(HWND_BROADCAST, WM_DEVICECHANGE, DBT_DEVNODES_CHANGED, 0);
//         }
//     }
// }
//
//
// DWORD
// WINAPI
// PnpEventThread(
//      LPVOID lpParameter)
// {
//     PLUGPLAY_CONTROL_USER_RESPONSE_DATA ResponseData = {0, 0, 0, 0};
//     DWORD dwRet = ERROR_SUCCESS;
//     NTSTATUS Status;
//     PPLUGPLAY_EVENT_BLOCK PnpEvent, NewPnpEvent;
//     ULONG PnpEventSize;
//
//     UNREFERENCED_PARAMETER!(lpParameter);
//
//     PnpEventSize = 0x1000;
//     PnpEvent = HeapAlloc(GetProcessHeap(), 0, PnpEventSize);
//     if (PnpEvent == NULL)
//         return ERROR_OUTOFMEMORY;
//
//     for (;;)
//     {
//         DPRINT("Calling NtGetPlugPlayEvent()\n");
//
//         /* Wait for the next PnP event */
//         Status = NtGetPlugPlayEvent(0, 0, PnpEvent, PnpEventSize);
//
//         /* Resize the buffer for the PnP event if it's too small */
//         if (Status == STATUS_BUFFER_TOO_SMALL)
//         {
//             PnpEventSize += 0x400;
//             NewPnpEvent = HeapReAlloc(GetProcessHeap(), 0, PnpEvent, PnpEventSize);
//             if (NewPnpEvent == NULL)
//             {
//                 dwRet = ERROR_OUTOFMEMORY;
//                 break;
//             }
//             PnpEvent = NewPnpEvent;
//             continue;
//         }
//
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT1("NtGetPlugPlayEvent() failed (Status 0x%08lx)\n", Status);
//             break;
//         }
//
//         /* Process the PnP event */
//         DPRINT("Received PnP Event\n");
//         switch (PnpEvent->EventCategory)
//         {
// //            case HardwareProfileChangeEvent:
//
//             case TargetDeviceChangeEvent:
//                 ProcessTargetDeviceEvent(PnpEvent);
//                 break;
//
//             case DeviceClassChangeEvent:
//                 ProcessDeviceClassChangeEvent(PnpEvent);
//                 break;
//
// //            case CustomDeviceEvent:
//
//             case DeviceInstallEvent:
//                 ProcessDeviceInstallEvent(PnpEvent);
//                 break;
//
// //            case DeviceArrivalEvent:
// //            case PowerEvent:
// //            case VetoEvent:
// //            case BlockedDriverEvent:
//
//             default:
//                 DPRINT1("Unsupported Event Category: %lu\n", PnpEvent->EventCategory);
//                 break;
//         }
//
//         /* Dequeue the current PnP event and signal the next one */
//         Status = NtPlugPlayControl(PlugPlayControlUserResponse,
//                                    &ResponseData,
//                                    sizeof(ResponseData));
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT1("NtPlugPlayControl(PlugPlayControlUserResponse) failed (Status 0x%08lx)\n", Status);
//             break;
//         }
//     }
//
//     HeapFree(GetProcessHeap(), 0, PnpEvent);
//
//     return dwRet;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
