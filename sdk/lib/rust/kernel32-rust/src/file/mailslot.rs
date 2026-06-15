use ntdll_sys::nt::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----

#[no_mangle]
pub unsafe extern "system" fn CreateMailslotA(
    lpName: LPCSTR,
    nMaxMessageSize: DWORD,
    lReadTimeout: DWORD,
    lpSecurityAttributes: LPSECURITY_ATTRIBUTES,
) -> HANDLE {
    ConvertWin32AnsiObjectApiToUnicodeApi2(Mailslot, lpName, nMaxMessageSize, lReadTimeout, lpSecurityAttributes)
}

#[no_mangle]
pub unsafe extern "system" fn CreateMailslotW(
    lpName: LPCWSTR,
    nMaxMessageSize: DWORD,
    lReadTimeout: DWORD,
    lpSecurityAttributes: LPSECURITY_ATTRIBUTES,
) -> HANDLE {
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut MailslotName: _UNICODE_STRING = core::mem::zeroed();
    let mut MailslotHandle: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS = 0;
    let mut Result: BOOLEAN = 0;
    let mut DefaultTimeOut: LARGE_INTEGER = core::mem::zeroed();
    let mut Iosb: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Attributes: ULONG = OBJ_CASE_INSENSITIVE;
    let mut SecurityDescriptor: PSECURITY_DESCRIPTOR = core::ptr::null_mut();

    Result = RtlDosPathNameToNtPathName_U(
        lpName,
        &mut MailslotName,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    if Result == 0 {
        SetLastError(ERROR_PATH_NOT_FOUND);
        return INVALID_HANDLE_VALUE;
    }

    if !lpSecurityAttributes.is_null() {
        SecurityDescriptor = (*lpSecurityAttributes).lpSecurityDescriptor;
        if (*lpSecurityAttributes).bInheritHandle != 0 {
            Attributes |= OBJ_INHERIT;
        }
    }

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.Attributes = Attributes;
    ObjectAttributes.ObjectName = &mut MailslotName;
    ObjectAttributes.SecurityDescriptor = SecurityDescriptor as *mut core::ffi::c_void;
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    if lReadTimeout == MAILSLOT_WAIT_FOREVER {
        DefaultTimeOut.QuadPart = 0xFFFFFFFFFFFFFFFFi64;
    } else {
        DefaultTimeOut.QuadPart = (lReadTimeout as i64) * -10000i64;
    }

    Status = NtCreateMailslotFile(
        &mut MailslotHandle,
        GENERIC_READ | SYNCHRONIZE | WRITE_DAC,
        &mut ObjectAttributes,
        &mut Iosb,
        FILE_WRITE_THROUGH,
        0,
        nMaxMessageSize,
        &mut DefaultTimeOut,
    );

    if Status == STATUS_INVALID_DEVICE_REQUEST || Status == STATUS_NOT_SUPPORTED {
        Status = STATUS_OBJECT_NAME_INVALID;
    }

    RtlFreeHeap(
        RtlGetProcessHeap(),
        0,
        MailslotName.Buffer as *mut core::ffi::c_void,
    );

    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return INVALID_HANDLE_VALUE;
    }

    MailslotHandle
}

#[no_mangle]
pub unsafe extern "system" fn GetMailslotInfo(
    hMailslot: HANDLE,
    lpMaxMessageSize: LPDWORD,
    lpNextSize: LPDWORD,
    lpMessageCount: LPDWORD,
    lpReadTimeout: LPDWORD,
) -> BOOL {
    let mut Buffer: FILE_MAILSLOT_QUERY_INFORMATION = core::mem::zeroed();
    let mut Iosb: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Status: NTSTATUS = 0;
    let mut Timeout: LARGE_INTEGER = core::mem::zeroed();

    Status = NtQueryInformationFile(
        hMailslot,
        &mut Iosb,
        &mut Buffer as *mut FILE_MAILSLOT_QUERY_INFORMATION as *mut core::ffi::c_void,
        core::mem::size_of::<FILE_MAILSLOT_QUERY_INFORMATION>() as u32,
        FileMailslotQueryInformation,
    );

    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return 0;
    }

    if !lpMaxMessageSize.is_null() {
        *lpMaxMessageSize = Buffer.MaximumMessageSize;
    }
    if !lpNextSize.is_null() {
        *lpNextSize = Buffer.NextMessageSize;
    }
    if !lpMessageCount.is_null() {
        *lpMessageCount = Buffer.MessagesAvailable;
    }

    if !lpReadTimeout.is_null() {
        if Buffer.ReadTimeout.QuadPart == -1i64 as u64 {
            *lpReadTimeout = 0xFFFFFFFF;
        } else {
            Timeout.QuadPart = -(Buffer.ReadTimeout.QuadPart);
            Timeout = RtlExtendedLargeIntegerDivide(Timeout, 10000, core::ptr::null_mut());
            if Timeout.u.__bindgen_anon_1.HighPart == 0 {
                *lpReadTimeout = Timeout.u.__bindgen_anon_1.LowPart;
            } else {
                *lpReadTimeout = 0xFFFFFFFE;
            }
        }
    }

    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn SetMailslotInfo(
    hMailslot: HANDLE,
    lReadTimeout: DWORD,
) -> BOOL {
    let mut Buffer: FILE_MAILSLOT_SET_INFORMATION = FILE_MAILSLOT_SET_INFORMATION {
        ReadTimeout: core::ptr::null_mut(),
    };
    let mut Timeout: LARGE_INTEGER = LARGE_INTEGER {
        QuadPart: 0,
    };
    let mut Iosb: _IO_STATUS_BLOCK = _IO_STATUS_BLOCK {
        Status: 0,
        Information: 0,
    };
    let mut Status: NTSTATUS = 0;

    if lReadTimeout == MAILSLOT_WAIT_FOREVER {
        Timeout.QuadPart = 0xFFFFFFFFFFFFFFFFi64;
    } else {
        Timeout.QuadPart = (lReadTimeout as i64) * -10000i64;
    }

    Buffer.ReadTimeout = &mut Timeout;

    Status = NtSetInformationFile(
        hMailslot,
        &mut Iosb,
        &mut Buffer as *mut FILE_MAILSLOT_SET_INFORMATION as PVOID,
        core::mem::size_of::<FILE_MAILSLOT_SET_INFORMATION>() as ULONG,
        FileMailslotSetInformation,
    );

    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return FALSE;
    }

    return TRUE;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS system libraries
//  * FILE:            dll/win32/kernel32/client/file/mailslot.c
//  * PURPOSE:         Mailslot functions
//  * PROGRAMMER:      Ariadne ( ariadne@xs4all.nl)
//  * UPDATE HISTORY:
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include <k32.h>
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
// CreateMailslotA(IN LPCSTR lpName,
//                 IN DWORD nMaxMessageSize,
//                 IN DWORD lReadTimeout,
//                 IN LPSECURITY_ATTRIBUTES lpSecurityAttributes)
// {
//     /* Call the W(ide) function */
//     ConvertWin32AnsiObjectApiToUnicodeApi2(Mailslot, lpName, nMaxMessageSize, lReadTimeout, lpSecurityAttributes);
// }
//
// /*
//  * @implemented
//  */
// HANDLE
// WINAPI
// CreateMailslotW(IN LPCWSTR lpName,
//                 IN DWORD nMaxMessageSize,
//                 IN DWORD lReadTimeout,
//                 IN LPSECURITY_ATTRIBUTES lpSecurityAttributes)
// {
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     UNICODE_STRING MailslotName;
//     HANDLE MailslotHandle;
//     NTSTATUS Status;
//     BOOLEAN Result;
//     LARGE_INTEGER DefaultTimeOut;
//     IO_STATUS_BLOCK Iosb;
//     ULONG Attributes = OBJ_CASE_INSENSITIVE;
//     PSECURITY_DESCRIPTOR SecurityDescriptor = NULL;
//
//     Result = RtlDosPathNameToNtPathName_U(lpName, &MailslotName, NULL, NULL);
//     if (!Result)
//     {
//         SetLastError(ERROR_PATH_NOT_FOUND);
//         return INVALID_HANDLE_VALUE;
//     }
//
//     DPRINT("Mailslot name: %wZ\n", &MailslotName);
//
//     if (lpSecurityAttributes)
//     {
//         SecurityDescriptor = lpSecurityAttributes->lpSecurityDescriptor;
//         if(lpSecurityAttributes->bInheritHandle) Attributes |= OBJ_INHERIT;
//     }
//
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &MailslotName,
//                                Attributes,
//                                NULL,
//                                SecurityDescriptor);
//
//     if (lReadTimeout == MAILSLOT_WAIT_FOREVER)
//     {
//         /* Set the max */
//         DefaultTimeOut.QuadPart = 0xFFFFFFFFFFFFFFFFLL;
//     }
//     else
//     {
//         /* Convert to NT format */
//         DefaultTimeOut.QuadPart = lReadTimeout * -10000LL;
//     }
//
//     Status = NtCreateMailslotFile(&MailslotHandle,
//                                   GENERIC_READ | SYNCHRONIZE | WRITE_DAC,
//                                   &ObjectAttributes,
//                                   &Iosb,
//                                   FILE_WRITE_THROUGH,
//                                   0,
//                                   nMaxMessageSize,
//                                   &DefaultTimeOut);
//
//     if ((Status == STATUS_INVALID_DEVICE_REQUEST) ||
//         (Status == STATUS_NOT_SUPPORTED))
//     {
//         Status = STATUS_OBJECT_NAME_INVALID;
//     }
//
//     RtlFreeHeap(RtlGetProcessHeap(), 0, MailslotName.Buffer);
//
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("NtCreateMailslot failed (Status %x)!\n", Status);
//         BaseSetLastNTError(Status);
//         return INVALID_HANDLE_VALUE;
//     }
//
//     return MailslotHandle;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// GetMailslotInfo(IN HANDLE hMailslot,
//                 IN LPDWORD lpMaxMessageSize,
//                 IN LPDWORD lpNextSize,
//                 IN LPDWORD lpMessageCount,
//                 IN LPDWORD lpReadTimeout)
// {
//     FILE_MAILSLOT_QUERY_INFORMATION Buffer;
//     IO_STATUS_BLOCK Iosb;
//     NTSTATUS Status;
//     LARGE_INTEGER Timeout;
//
//     Status = NtQueryInformationFile(hMailslot,
//                                     &Iosb,
//                                     &Buffer,
//                                     sizeof(FILE_MAILSLOT_QUERY_INFORMATION),
//                                     FileMailslotQueryInformation);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("NtQueryInformationFile failed (Status %x)!\n", Status);
//         BaseSetLastNTError(Status);
//         return FALSE;
//      }
//
//      if (lpMaxMessageSize) *lpMaxMessageSize = Buffer.MaximumMessageSize;
//      if (lpNextSize) *lpNextSize = Buffer.NextMessageSize;
//      if (lpMessageCount) *lpMessageCount = Buffer.MessagesAvailable;
//
//      if (lpReadTimeout)
//      {
//          if (Buffer.ReadTimeout.QuadPart == 0xFFFFFFFFFFFFFFFFLL)
//          {
//              *lpReadTimeout = MAILSLOT_WAIT_FOREVER;
//          }
//          else
//          {
//              Timeout.QuadPart = -Buffer.ReadTimeout.QuadPart;
//              Timeout = RtlExtendedLargeIntegerDivide(Timeout, 10000, NULL);
//              if (Timeout.HighPart == 0)
//              {
//                  *lpReadTimeout = Timeout.LowPart;
//              }
//              else
//              {
//                  *lpReadTimeout = 0xFFFFFFFE;
//              }
//          }
//      }
//
//    return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// SetMailslotInfo(IN HANDLE hMailslot,
//                 IN DWORD lReadTimeout)
// {
//     FILE_MAILSLOT_SET_INFORMATION Buffer;
//     LARGE_INTEGER Timeout;
//     IO_STATUS_BLOCK Iosb;
//     NTSTATUS Status;
//
//     if (lReadTimeout == MAILSLOT_WAIT_FOREVER)
//     {
//         /* Set the max */
//         Timeout.QuadPart = 0xFFFFFFFFFFFFFFFFLL;
//     }
//     else
//     {
//         /* Convert to NT format */
//         Timeout.QuadPart = lReadTimeout * -10000LL;
//     }
//
//     Buffer.ReadTimeout = &Timeout;
//
//     Status = NtSetInformationFile(hMailslot,
//                                   &Iosb,
//                                   &Buffer,
//                                   sizeof(FILE_MAILSLOT_SET_INFORMATION),
//                                   FileMailslotSetInformation);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("NtSetInformationFile failed (Status %x)!\n", Status);
//         BaseSetLastNTError(Status);
//         return FALSE;
//      }
//
//      return TRUE;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
