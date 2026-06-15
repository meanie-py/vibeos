use ntdll_sys::nt::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----

#[no_mangle]
pub unsafe extern "system" fn CreateMemoryResourceNotification(
    NotificationType: MEMORY_RESOURCE_NOTIFICATION_TYPE,
) -> HANDLE {
    let mut EventName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = _OBJECT_ATTRIBUTES {
        Length: 0,
        RootDirectory: core::ptr::null_mut(),
        ObjectName: core::ptr::null_mut(),
        Attributes: 0,
        SecurityDescriptor: core::ptr::null_mut(),
        SecurityQualityOfService: core::ptr::null_mut(),
    };
    let mut hEvent: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS = 0;

    if NotificationType > HighMemoryResourceNotification {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return core::ptr::null_mut();
    }

    RtlInitUnicodeString(
        &mut EventName,
        if NotificationType != 0 {
            b"\\\0K\0e\0r\0n\0e\0l\0O\0b\0j\0e\0c\0t\0s\0\\\0H\0i\0g\0h\0M\0e\0m\0o\0r\0y\0C\0o\0n\0d\0i\0t\0i\0o\0n\0\0\0".as_ptr() as PCWSTR
        } else {
            b"\\\0K\0e\0r\0n\0e\0l\0O\0b\0j\0e\0c\0t\0s\0\\\0L\0o\0w\0M\0e\0m\0o\0r\0y\0C\0o\0n\0d\0i\0t\0i\0o\0n\0\0\0".as_ptr() as PCWSTR
        },
    );

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.Attributes = 0;
    ObjectAttributes.ObjectName = &mut EventName;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtOpenEvent(
        &mut hEvent,
        (EVENT_QUERY_STATE | SYNCHRONIZE) as ACCESS_MASK,
        &mut ObjectAttributes,
    );

    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return core::ptr::null_mut();
    }

    hEvent
}
#[no_mangle]
pub unsafe extern "system" fn QueryMemoryResourceNotification(
    ResourceNotificationHandle: HANDLE,
    ResourceState: PBOOL,
) -> BOOL {
    let mut EventInfo: EVENT_BASIC_INFORMATION = core::mem::zeroed();
    let mut Status: NTSTATUS = 0;

    if !ResourceNotificationHandle.is_null()
        && ResourceNotificationHandle != INVALID_HANDLE_VALUE
        && !ResourceState.is_null()
    {
        Status = NtQueryEvent(
            ResourceNotificationHandle,
            EventBasicInformation,
            &mut EventInfo as *mut EVENT_BASIC_INFORMATION as *mut core::ffi::c_void,
            core::mem::size_of::<EVENT_BASIC_INFORMATION>() as u32,
            core::ptr::null_mut(),
        );
        if NT_SUCCESS(Status) {
            *ResourceState = (EventInfo.EventState == 1) as i32;
            return 1;
        }
        BaseSetLastNTError(Status);
    } else {
        SetLastError(ERROR_INVALID_PARAMETER as u32);
    }

    0
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:            See COPYING in the top level directory
//  * PROJECT:              ReactOS Win32 Base API
//  * FILE:                 dll/win32/kernel32/client/resntfy.c
//  * PURPOSE:              Memory Resource Notifications
//  * PROGRAMMER:           Thomas Weidenmueller <w3seek@reactos.com>
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include <k32.h>
//
// #define NDEBUG
// #include <debug.h>
//
// /* FUNCTIONS ******************************************************************/
//
// /*
//  * @implemented
//  */
// HANDLE
// WINAPI
// CreateMemoryResourceNotification(IN MEMORY_RESOURCE_NOTIFICATION_TYPE NotificationType)
// {
//     UNICODE_STRING EventName;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     HANDLE hEvent;
//     NTSTATUS Status;
//
//     if (NotificationType > HighMemoryResourceNotification)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return NULL;
//     }
//
//     RtlInitUnicodeString(&EventName,
//                          NotificationType ?
//                          L"\\KernelObjects\\HighMemoryCondition" :
//                          L"\\KernelObjects\\LowMemoryCondition");
//
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &EventName,
//                                0,
//                                NULL,
//                                NULL);
//
//     Status = NtOpenEvent(&hEvent,
//                          EVENT_QUERY_STATE | SYNCHRONIZE,
//                          &ObjectAttributes);
//     if (!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError(Status);
//         return NULL;
//     }
//
//     return hEvent;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// QueryMemoryResourceNotification(IN HANDLE ResourceNotificationHandle,
//                                 OUT PBOOL ResourceState)
// {
//     EVENT_BASIC_INFORMATION EventInfo;
//     NTSTATUS Status;
//
//     if ((ResourceNotificationHandle) &&
//         (ResourceNotificationHandle != INVALID_HANDLE_VALUE) &&
//         (ResourceState))
//     {
//         Status = NtQueryEvent(ResourceNotificationHandle,
//                               EventBasicInformation,
//                               &EventInfo,
//                               sizeof(EventInfo),
//                               NULL);
//         if (NT_SUCCESS(Status))
//         {
//             *ResourceState = (EventInfo.EventState == 1);
//             return TRUE;
//         }
//
//         BaseSetLastNTError(Status);
//     }
//     else
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//     }
//
//     return FALSE;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
