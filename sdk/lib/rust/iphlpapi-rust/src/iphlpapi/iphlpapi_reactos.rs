use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const WINE_DEFAULT_DEBUG_CHANNEL: &str = "iphlpapi";
#[no_mangle]
pub unsafe extern "C" fn TCPSendIoctl(mut hDevice: HANDLE, dwIoControlCode: DWORD, lpInBuffer: LPVOID, pInBufferSize: *mut ULONG, lpOutBuffer: LPVOID, pOutBufferSize: *mut ULONG) -> DWORD {
    let mut Hack: u8 = 0u8;
    let mut Event: HANDLE;
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Status: NTSTATUS;

    if hDevice == INVALID_HANDLE_VALUE {
        let dev_name_wide: [u16; 11] = [
            '\\' as u16, 'D' as u16, 'e' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16,
            '\\' as u16, 'I' as u16, 'p' as u16, 0u16,
        ];
        let DevName = _UNICODE_STRING {
            Length: 20u16,
            MaximumLength: 22u16,
            Buffer: dev_name_wide.as_ptr() as *mut u16,
        };
        let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
        ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
        ObjectAttributes.RootDirectory = core::ptr::null_mut();
        ObjectAttributes.ObjectName = &DevName as *const _UNICODE_STRING as *mut _UNICODE_STRING;
        ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
        ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
        ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

        Hack = 1u8;

        Status = NtCreateFile(
            core::ptr::addr_of_mut!(hDevice),
            GENERIC_EXECUTE,
            &mut ObjectAttributes,
            &mut IoStatusBlock,
            core::ptr::null_mut(),
            FILE_ATTRIBUTE_NORMAL,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            FILE_OPEN_IF,
            0,
            core::ptr::null_mut(),
            0,
        );
        if !NT_SUCCESS(Status) {
            return RtlNtStatusToDosError(Status);
        }
    }

    Event = CreateEventW(core::ptr::null_mut(), TRUE, FALSE, core::ptr::null_mut());
    if Event == core::ptr::null_mut() {
        if Hack != 0u8 {
            CloseHandle(hDevice);
        }
        return GetLastError();
    }

    IoStatusBlock.__bindgen_anon_1.Status = STATUS_SUCCESS;
    IoStatusBlock.Information = 0;
    Status = NtDeviceIoControlFile(
        hDevice,
        Event,
        None,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(IoStatusBlock),
        dwIoControlCode,
        lpInBuffer,
        (*pInBufferSize) as ULONG,
        lpOutBuffer,
        (*pOutBufferSize) as ULONG,
    );
    if Status == STATUS_PENDING {
        NtWaitForSingleObject(Event, 0u8, core::ptr::null_mut());
        Status = IoStatusBlock.__bindgen_anon_1.Status;
    }

    CloseHandle(Event);
    *pOutBufferSize = IoStatusBlock.Information as ULONG;

    if Hack != 0u8 {
        CloseHandle(hDevice);
    }

    if !NT_SUCCESS(Status) {
        return RtlNtStatusToDosError(Status);
    }
    ERROR_SUCCESS
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS Networking
//  * LICENSE:     GPL - See COPYING in the top level directory
//  * FILE:        dll/win32/iphlpapi/iphlpapi_reactos.c
//  * PURPOSE:     DHCP helper functions for ReactOS
//  * PROGRAMMERS: Pierre Schweitzer <pierre@reactos.org>
//  */
//
// #include "iphlpapi_private.h"
//
// WINE_DEFAULT_DEBUG_CHANNEL(iphlpapi);
//
// DWORD TCPSendIoctl(HANDLE hDevice, DWORD dwIoControlCode, LPVOID lpInBuffer, PULONG pInBufferSize, LPVOID lpOutBuffer, PULONG pOutBufferSize)
// {
//     BOOL Hack = FALSE;
//     HANDLE Event;
//     IO_STATUS_BLOCK IoStatusBlock;
//     NTSTATUS Status;
//
//     /* FIXME: We don't have a global handle opened to \Device\Ip, so open one each time
//      * we need. In a future, it would be cool, just to pass it to TCPSendIoctl using the first arg
//      */
//     if (hDevice == INVALID_HANDLE_VALUE)
//     {
//         UNICODE_STRING DevName = RTL_CONSTANT_STRING(L"\\Device\\Ip");
//         OBJECT_ATTRIBUTES ObjectAttributes;
//
//         FIXME!("Using the handle hack\n");
//         Hack = TRUE;
//
//         InitializeObjectAttributes(&ObjectAttributes,
//                                    &DevName,
//                                    OBJ_CASE_INSENSITIVE,
//                                    NULL,
//                                    NULL);
//
//         Status = NtCreateFile(&hDevice, GENERIC_EXECUTE, &ObjectAttributes,
//                               &IoStatusBlock, 0, FILE_ATTRIBUTE_NORMAL, FILE_SHARE_READ | FILE_SHARE_WRITE, FILE_OPEN_IF,
//                               0, NULL, 0);
//         if (!NT_SUCCESS(Status))
//         {
//           return RtlNtStatusToDosError(Status);
//         }
//     }
//
//     /* Sync event */
//     Event = CreateEventW(NULL, TRUE, FALSE, NULL);
//     if (Event == NULL)
//     {
//         /* FIXME: See upper */
//         if (Hack)
//         {
//             CloseHandle(hDevice);
//         }
//         return GetLastError();
//     }
//
//     /* Reinit, and call the networking stack */
//     IoStatusBlock.Status = STATUS_SUCCESS;
//     IoStatusBlock.Information = 0;
//     Status = NtDeviceIoControlFile(hDevice, Event, NULL, NULL, &IoStatusBlock, dwIoControlCode, lpInBuffer, *pInBufferSize, lpOutBuffer, *pOutBufferSize);
//     if (Status == STATUS_PENDING)
//     {
//         NtWaitForSingleObject(Event, FALSE, NULL);
//         Status = IoStatusBlock.Status;
//     }
//
//     /* Close & return size info */
//     CloseHandle(Event);
//     *pOutBufferSize = IoStatusBlock.Information;
//
//     /* FIXME: See upper */
//     if (Hack)
//     {
//         CloseHandle(hDevice);
//     }
//
//     /* Return result */
//     if (!NT_SUCCESS(Status))
//     {
//         return RtlNtStatusToDosError(Status);
//     }
//
//     return ERROR_SUCCESS;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
