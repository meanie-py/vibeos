use ntdll_sys::nt::*;
use crate::prelude::*;
use crate::k32_externs::{BaseFormatObjectAttributes, RtlCopyMemory};
use core::ffi::c_void;

extern "system" {
    fn BasepGetNamedObjectDirectory() -> HANDLE;
}

// ---- file-scope preamble (translated) ----
#[no_mangle]
pub unsafe extern "system" fn CreateJobObjectA(
    lpJobAttributes: LPSECURITY_ATTRIBUTES,
    lpName: LPCSTR,
) -> HANDLE {
    let mut Status: NTSTATUS = 0;
    let mut UnicodeCache: PUNICODE_STRING = core::ptr::null_mut();
    let mut AnsiName: ANSI_STRING = ANSI_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    if lpName.is_null() {
        return CreateJobObjectW(lpJobAttributes, core::ptr::null());
    }
    UnicodeCache = &raw mut (*NtCurrentTeb())
        .StaticUnicodeString as PUNICODE_STRING;
    RtlInitAnsiString(&raw mut AnsiName, lpName as PCSZ);
    Status = RtlAnsiStringToUnicodeString(UnicodeCache, &raw mut AnsiName, 0);
    if Status >= 0 {
        return CreateJobObjectW(lpJobAttributes, (*UnicodeCache).Buffer as LPCWSTR);
    }
    if Status == STATUS_BUFFER_OVERFLOW {
        SetLastError(ERROR_FILENAME_EXCED_RANGE as DWORD);
    } else {
        BaseSetLastNTError(Status);
    }
    return core::ptr::null_mut();
}
#[no_mangle]
pub unsafe extern "system" fn CreateJobObjectW(
    lpJobAttributes: LPSECURITY_ATTRIBUTES,
    lpName: LPCWSTR,
) -> HANDLE {
    let mut Status: NTSTATUS = 0;
    let mut Handle: HANDLE = core::ptr::null_mut();
    let mut ObjectName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut LocalAttributes: _OBJECT_ATTRIBUTES = _OBJECT_ATTRIBUTES {
        Length: 0,
        RootDirectory: core::ptr::null_mut(),
        ObjectName: core::ptr::null_mut(),
        Attributes: 0,
        SecurityDescriptor: core::ptr::null_mut(),
        SecurityQualityOfService: core::ptr::null_mut(),
    };
    let mut ObjectAttributes: POBJECT_ATTRIBUTES = core::ptr::addr_of_mut!(LocalAttributes);
    if !lpName.is_null() {
        RtlInitUnicodeString(core::ptr::addr_of_mut!(ObjectName), lpName as PCWSTR);
    }
    ObjectAttributes = BaseFormatObjectAttributes(
        core::ptr::addr_of_mut!(LocalAttributes),
        lpJobAttributes as PSECURITY_ATTRIBUTES,
        if !lpName.is_null() {
            core::ptr::addr_of_mut!(ObjectName)
        } else {
            core::ptr::null_mut()
        },
    );
    Status = NtCreateJobObject(
        core::ptr::addr_of_mut!(Handle),
        (0xf0000i32 | 0x100000i32 | 31i32) as ACCESS_MASK,
        ObjectAttributes,
    );
    if Status >= 0i32 {
        if Status == STATUS_OBJECT_NAME_EXISTS {
            SetLastError(ERROR_ALREADY_EXISTS as DWORD);
        } else {
            SetLastError(ERROR_SUCCESS as DWORD);
        }
        return Handle;
    }
    BaseSetLastNTError(Status);
    return core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "system" fn OpenJobObjectW(
    dwDesiredAccess: DWORD,
    bInheritHandle: BOOL,
    lpName: LPCWSTR,
) -> HANDLE {
    let mut Status: NTSTATUS = 0;
    let mut Handle: HANDLE = core::ptr::null_mut();
    let mut ObjectName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut LocalAttributes: _OBJECT_ATTRIBUTES = _OBJECT_ATTRIBUTES {
        Length: 0,
        RootDirectory: core::ptr::null_mut(),
        ObjectName: core::ptr::null_mut(),
        Attributes: 0,
        SecurityDescriptor: core::ptr::null_mut(),
        SecurityQualityOfService: core::ptr::null_mut(),
    };
    let mut ObjectAttributes: POBJECT_ATTRIBUTES = core::ptr::addr_of_mut!(LocalAttributes);

    if lpName.is_null() {
        BaseSetLastNTError(STATUS_INVALID_PARAMETER);
        return core::ptr::null_mut();
    }

    RtlInitUnicodeString(core::ptr::addr_of_mut!(ObjectName), lpName as PCWSTR);

    (*ObjectAttributes).Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    (*ObjectAttributes).RootDirectory = BasepGetNamedObjectDirectory();
    (*ObjectAttributes).Attributes = (if bInheritHandle != 0 {
        0x2 as ::core::ffi::c_long
    } else {
        0 as ::core::ffi::c_long
    }) as ULONG;
    (*ObjectAttributes).ObjectName = core::ptr::addr_of_mut!(ObjectName) as PUNICODE_STRING;
    (*ObjectAttributes).SecurityDescriptor = core::ptr::null_mut() as PVOID;
    (*ObjectAttributes).SecurityQualityOfService = core::ptr::null_mut() as PVOID;

    Status = NtOpenJobObject(
        core::ptr::addr_of_mut!(Handle),
        dwDesiredAccess as ACCESS_MASK,
        ObjectAttributes,
    );

    if !(Status >= 0 as ::core::ffi::c_long) {
        BaseSetLastNTError(Status);
        return core::ptr::null_mut();
    }

    return Handle;
}
#[no_mangle]
pub unsafe extern "system" fn OpenJobObjectA(
    dwDesiredAccess: DWORD,
    bInheritHandle: BOOL,
    lpName: LPCSTR,
) -> HANDLE {
    let mut Status: NTSTATUS = 0;
    let mut UnicodeCache: PUNICODE_STRING = core::ptr::null_mut();
    let mut AnsiName: ANSI_STRING = ANSI_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    if lpName.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return core::ptr::null_mut();
    }
    UnicodeCache = &raw mut (*NtCurrentTeb())
        .StaticUnicodeString as PUNICODE_STRING;
    RtlInitAnsiString(&raw mut AnsiName, lpName as PCSZ);
    Status = RtlAnsiStringToUnicodeString(UnicodeCache, &raw mut AnsiName, FALSE as BOOLEAN);
    if Status >= 0 as ::core::ffi::c_long {
        return OpenJobObjectW(
            dwDesiredAccess,
            bInheritHandle,
            (*UnicodeCache).Buffer as LPCWSTR,
        );
    }
    if Status == STATUS_BUFFER_OVERFLOW {
        SetLastError(ERROR_FILENAME_EXCED_RANGE as DWORD);
    } else {
        BaseSetLastNTError(Status);
    }
    return core::ptr::null_mut::<core::ffi::c_void>();
}

#[no_mangle]
pub unsafe extern "system" fn IsProcessInJob(
    ProcessHandle: HANDLE,
    JobHandle: HANDLE,
    Result: PBOOL,
) -> BOOL {
    let Status: NTSTATUS;
    Status = NtIsProcessInJob(ProcessHandle, JobHandle);
    if NT_SUCCESS(Status) {
        *Result = (Status == STATUS_PROCESS_IN_JOB) as BOOL;
        return 1i32;
    }
    BaseSetLastNTError(Status);
    return 0i32;
}

#[no_mangle]
pub unsafe extern "system" fn AssignProcessToJobObject(
    hJob: HANDLE,
    hProcess: HANDLE,
) -> BOOL {
    let mut Status: NTSTATUS = 0;
    Status = NtAssignProcessToJobObject(hJob, hProcess);
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return 0;
    }
    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn QueryInformationJobObject(
    hJob: HANDLE,
    JobObjectInformationClass: JOBOBJECTINFOCLASS,
    lpJobObjectInformation: LPVOID,
    cbJobObjectInformationLength: DWORD,
    lpReturnLength: LPDWORD,
) -> BOOL {
    let mut Status: NTSTATUS;
    let mut JobInfo: PVOID;
    let mut LocalInfo: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = core::mem::zeroed();
    let mut ExpectedSize: ULONG;

    if JobObjectInformationClass == JobObjectBasicLimitInformation {
        ExpectedSize = core::mem::size_of::<JOBOBJECT_BASIC_LIMIT_INFORMATION>() as ULONG;
        JobInfo = core::ptr::addr_of_mut!(LocalInfo) as PVOID;
    } else if JobObjectInformationClass == JobObjectExtendedLimitInformation {
        ExpectedSize = core::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as ULONG;
        JobInfo = core::ptr::addr_of_mut!(LocalInfo) as PVOID;
    } else {
        ExpectedSize = cbJobObjectInformationLength;
        JobInfo = lpJobObjectInformation;
    }

    if cbJobObjectInformationLength != ExpectedSize {
        BaseSetLastNTError(STATUS_INFO_LENGTH_MISMATCH);
        return 0;
    }

    Status = NtQueryInformationJobObject(
        hJob,
        JobObjectInformationClass,
        JobInfo,
        ExpectedSize,
        lpReturnLength,
    );
    if NT_SUCCESS(Status) {
        if JobInfo != core::ptr::addr_of_mut!(LocalInfo) as PVOID {
            return 1;
        }

        match LocalInfo.BasicLimitInformation.PriorityClass {
            PROCESS_PRIORITY_CLASS_IDLE => {
                LocalInfo.BasicLimitInformation.PriorityClass = IDLE_PRIORITY_CLASS;
            }
            PROCESS_PRIORITY_CLASS_BELOW_NORMAL => {
                LocalInfo.BasicLimitInformation.PriorityClass = BELOW_NORMAL_PRIORITY_CLASS;
            }
            PROCESS_PRIORITY_CLASS_NORMAL => {
                LocalInfo.BasicLimitInformation.PriorityClass = NORMAL_PRIORITY_CLASS;
            }
            PROCESS_PRIORITY_CLASS_ABOVE_NORMAL => {
                LocalInfo.BasicLimitInformation.PriorityClass = ABOVE_NORMAL_PRIORITY_CLASS;
            }
            PROCESS_PRIORITY_CLASS_HIGH => {
                LocalInfo.BasicLimitInformation.PriorityClass = HIGH_PRIORITY_CLASS;
            }
            PROCESS_PRIORITY_CLASS_REALTIME => {
                LocalInfo.BasicLimitInformation.PriorityClass = REALTIME_PRIORITY_CLASS;
            }
            _ => {
                LocalInfo.BasicLimitInformation.PriorityClass = NORMAL_PRIORITY_CLASS;
            }
        }

        RtlCopyMemory(
            lpJobObjectInformation,
            core::ptr::addr_of!(LocalInfo) as *const _,
            ExpectedSize as SIZE_T,
        );
        return 1;
    }

    BaseSetLastNTError(Status);
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn SetInformationJobObject(
    hJob: HANDLE,
    JobObjectInformationClass: JOBOBJECTINFOCLASS,
    lpJobObjectInformation: LPVOID,
    cbJobObjectInformationLength: DWORD,
) -> BOOL {
    let mut Status: NTSTATUS = 0;
    let mut JobInfo: PVOID = core::ptr::null_mut();
    let mut LocalInfo: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = core::mem::zeroed();
    let mut ExpectedSize: ULONG = 0;
    let mut State: PVOID = core::ptr::null_mut();
    let mut Privilege: ULONG = SE_INC_BASE_PRIORITY_PRIVILEGE as ULONG;

    if JobObjectInformationClass == JobObjectBasicLimitInformation {
        ExpectedSize = core::mem::size_of::<JOBOBJECT_BASIC_LIMIT_INFORMATION>() as ULONG;
        JobInfo = core::ptr::addr_of_mut!(LocalInfo) as PVOID;
    } else if JobObjectInformationClass == JobObjectExtendedLimitInformation {
        ExpectedSize = core::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as ULONG;
        JobInfo = core::ptr::addr_of_mut!(LocalInfo) as PVOID;
    } else {
        ExpectedSize = cbJobObjectInformationLength;
        JobInfo = lpJobObjectInformation;
    }

    if cbJobObjectInformationLength != ExpectedSize {
        BaseSetLastNTError(STATUS_INFO_LENGTH_MISMATCH);
        return 0;
    }

    if JobInfo == core::ptr::addr_of_mut!(LocalInfo) as PVOID {
        core::ptr::copy_nonoverlapping(
            lpJobObjectInformation as *const u8,
            core::ptr::addr_of_mut!(LocalInfo) as *mut u8,
            ExpectedSize as usize,
        );

        if LocalInfo.BasicLimitInformation.LimitFlags & JOB_OBJECT_LIMIT_PRIORITY_CLASS != 0 {
            match LocalInfo.BasicLimitInformation.PriorityClass {
                IDLE_PRIORITY_CLASS => {
                    LocalInfo.BasicLimitInformation.PriorityClass = PROCESS_PRIORITY_CLASS_IDLE;
                }
                BELOW_NORMAL_PRIORITY_CLASS => {
                    LocalInfo.BasicLimitInformation.PriorityClass = PROCESS_PRIORITY_CLASS_BELOW_NORMAL;
                }
                NORMAL_PRIORITY_CLASS => {
                    LocalInfo.BasicLimitInformation.PriorityClass = PROCESS_PRIORITY_CLASS_NORMAL;
                }
                ABOVE_NORMAL_PRIORITY_CLASS => {
                    LocalInfo.BasicLimitInformation.PriorityClass = PROCESS_PRIORITY_CLASS_ABOVE_NORMAL;
                }
                HIGH_PRIORITY_CLASS => {
                    LocalInfo.BasicLimitInformation.PriorityClass = PROCESS_PRIORITY_CLASS_HIGH;
                }
                REALTIME_PRIORITY_CLASS => {
                    LocalInfo.BasicLimitInformation.PriorityClass = PROCESS_PRIORITY_CLASS_REALTIME;
                }
                _ => {
                    LocalInfo.BasicLimitInformation.PriorityClass = PROCESS_PRIORITY_CLASS_NORMAL;
                }
            }
        }

        if LocalInfo.BasicLimitInformation.LimitFlags & JOB_OBJECT_LIMIT_WORKINGSET != 0 {
            Status = RtlAcquirePrivilege(
                core::ptr::addr_of_mut!(Privilege),
                1,
                0,
                core::ptr::addr_of_mut!(State),
            );
        }
    }

    Status = NtSetInformationJobObject(
        hJob,
        JobObjectInformationClass,
        JobInfo,
        ExpectedSize,
    );

    if NT_SUCCESS(Status) {
        if !State.is_null() {
            RtlReleasePrivilege(State);
        }
        return 1;
    }

    BaseSetLastNTError(Status);
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn TerminateJobObject(
    hJob: HANDLE,
    uExitCode: UINT,
) -> BOOL {
    let mut Status: NTSTATUS = 0;
    Status = NtTerminateJobObject(hJob, uExitCode as NTSTATUS);
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return 0; // FALSE
    }
    return 1; // TRUE
}

#[no_mangle]
pub unsafe extern "system" fn CreateJobSet(
    NumJob: ULONG,
    UserJobSet: PJOB_SET_ARRAY,
    Flags: ULONG,
) -> BOOL {
    let mut Status: NTSTATUS;
    Status = NtCreateJobSet(NumJob, UserJobSet, Flags);
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return 0i32;
    }
    return 1i32;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS system libraries
//  * FILE:            dll/win32/kernel32/client/job.c
//  * PURPOSE:         Job functions
//  * PROGRAMMER:      Thomas Weidenmueller <w3seek@reactos.com>
//  * UPDATE HISTORY:
//  *                  Created 9/23/2004
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include <k32.h>
// #include <winspool.h>
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
// CreateJobObjectA(IN LPSECURITY_ATTRIBUTES lpJobAttributes,
//                  IN LPCSTR lpName)
// {
//     /* Call the W(ide) function */
//     ConvertWin32AnsiObjectApiToUnicodeApi(JobObject, lpName, lpJobAttributes);
// }
//
// /*
//  * @implemented
//  */
// HANDLE
// WINAPI
// CreateJobObjectW(IN LPSECURITY_ATTRIBUTES lpJobAttributes,
//                  IN LPCWSTR lpName)
// {
//     /* Create the NT object */
//     CreateNtObjectFromWin32Api(JobObject, JobObject, JOB_OBJECT_ALL_ACCESS, lpJobAttributes, lpName);
// }
//
// /*
//  * @implemented
//  */
// HANDLE
// WINAPI
// OpenJobObjectW(IN DWORD dwDesiredAccess,
//                IN BOOL bInheritHandle,
//                IN LPCWSTR lpName)
// {
//     /* Open the NT object */
//     OpenNtObjectFromWin32Api(JobObject, dwDesiredAccess, bInheritHandle, lpName);
// }
//
//
// /*
//  * @implemented
//  */
// HANDLE
// WINAPI
// OpenJobObjectA(IN DWORD dwDesiredAccess,
//                IN BOOL bInheritHandle,
//                IN LPCSTR lpName)
// {
//     /* Call the W(ide) function */
//     ConvertOpenWin32AnsiObjectApiToUnicodeApi(JobObject, dwDesiredAccess, bInheritHandle, lpName);
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// IsProcessInJob(IN HANDLE ProcessHandle,
//                IN HANDLE JobHandle,
//                OUT PBOOL Result)
// {
//     NTSTATUS Status;
//
//     Status = NtIsProcessInJob(ProcessHandle, JobHandle);
//     if (NT_SUCCESS(Status))
//     {
//         *Result = (Status == STATUS_PROCESS_IN_JOB);
//         return TRUE;
//     }
//
//     BaseSetLastNTError(Status);
//     return FALSE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// AssignProcessToJobObject(IN HANDLE hJob,
//                          IN HANDLE hProcess)
// {
//     NTSTATUS Status;
//
//     Status = NtAssignProcessToJobObject(hJob, hProcess);
//     if (!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// QueryInformationJobObject(IN HANDLE hJob,
//                           IN JOBOBJECTINFOCLASS JobObjectInformationClass,
//                           IN LPVOID lpJobObjectInformation,
//                           IN DWORD cbJobObjectInformationLength,
//                           OUT LPDWORD lpReturnLength)
// {
//     NTSTATUS Status;
//     PVOID JobInfo;
//     JOBOBJECT_EXTENDED_LIMIT_INFORMATION LocalInfo;
//     ULONG ExpectedSize;
//
//     if (JobObjectInformationClass == JobObjectBasicLimitInformation)
//     {
//         ExpectedSize = sizeof(JOBOBJECT_BASIC_LIMIT_INFORMATION);
//         JobInfo = &LocalInfo;
//     }
//     else if (JobObjectInformationClass == JobObjectExtendedLimitInformation)
//     {
//         ExpectedSize = sizeof(JOBOBJECT_EXTENDED_LIMIT_INFORMATION);
//         JobInfo = &LocalInfo;
//     }
//     else
//     {
//         ExpectedSize = cbJobObjectInformationLength;
//         JobInfo = lpJobObjectInformation;
//     }
//
//     if (cbJobObjectInformationLength != ExpectedSize)
//     {
//         BaseSetLastNTError(STATUS_INFO_LENGTH_MISMATCH);
//         return FALSE;
//     }
//
//     Status = NtQueryInformationJobObject(hJob,
//                                          JobObjectInformationClass,
//                                          JobInfo,
//                                          ExpectedSize,
//                                          lpReturnLength);
//     if (NT_SUCCESS(Status))
//     {
//         if (JobInfo != &LocalInfo) return TRUE;
//
//         switch (LocalInfo.BasicLimitInformation.PriorityClass)
//         {
//             case PROCESS_PRIORITY_CLASS_IDLE:
//                 LocalInfo.BasicLimitInformation.PriorityClass =
//                 IDLE_PRIORITY_CLASS;
//                 break;
//
//             case PROCESS_PRIORITY_CLASS_BELOW_NORMAL:
//                 LocalInfo.BasicLimitInformation.PriorityClass =
//                 BELOW_NORMAL_PRIORITY_CLASS;
//                 break;
//
//             case PROCESS_PRIORITY_CLASS_NORMAL:
//                 LocalInfo.BasicLimitInformation.PriorityClass =
//                 NORMAL_PRIORITY_CLASS;
//                 break;
//
//             case PROCESS_PRIORITY_CLASS_ABOVE_NORMAL:
//                 LocalInfo.BasicLimitInformation.PriorityClass =
//                 ABOVE_NORMAL_PRIORITY_CLASS;
//                 break;
//
//             case PROCESS_PRIORITY_CLASS_HIGH:
//                 LocalInfo.BasicLimitInformation.PriorityClass =
//                 HIGH_PRIORITY_CLASS;
//                 break;
//
//             case PROCESS_PRIORITY_CLASS_REALTIME:
//                 LocalInfo.BasicLimitInformation.PriorityClass =
//                 REALTIME_PRIORITY_CLASS;
//                 break;
//
//             default:
//                 LocalInfo.BasicLimitInformation.PriorityClass =
//                 NORMAL_PRIORITY_CLASS;
//                 break;
//         }
//
//         RtlCopyMemory(lpJobObjectInformation, &LocalInfo, ExpectedSize);
//         return TRUE;
//     }
//
//     BaseSetLastNTError(Status);
//     return FALSE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// SetInformationJobObject(IN HANDLE hJob,
//                         IN JOBOBJECTINFOCLASS JobObjectInformationClass,
//                         IN LPVOID lpJobObjectInformation,
//                         IN DWORD cbJobObjectInformationLength)
// {
//     NTSTATUS Status;
//     PVOID JobInfo;
//     JOBOBJECT_EXTENDED_LIMIT_INFORMATION LocalInfo;
//     ULONG ExpectedSize;
//     PVOID State = NULL;
//     ULONG Privilege = SE_INC_BASE_PRIORITY_PRIVILEGE;
//
//     if (JobObjectInformationClass == JobObjectBasicLimitInformation)
//     {
//         ExpectedSize = sizeof(JOBOBJECT_BASIC_LIMIT_INFORMATION);
//         JobInfo = &LocalInfo;
//     }
//     else if (JobObjectInformationClass == JobObjectExtendedLimitInformation)
//     {
//         ExpectedSize = sizeof(JOBOBJECT_EXTENDED_LIMIT_INFORMATION);
//         JobInfo = &LocalInfo;
//     }
//     else
//     {
//         ExpectedSize = cbJobObjectInformationLength;
//         JobInfo = lpJobObjectInformation;
//     }
//
//     if (cbJobObjectInformationLength != ExpectedSize)
//     {
//         BaseSetLastNTError(STATUS_INFO_LENGTH_MISMATCH);
//         return FALSE;
//     }
//
//     if (JobInfo == &LocalInfo)
//     {
//         RtlCopyMemory(&LocalInfo, lpJobObjectInformation, ExpectedSize);
//
//         if (LocalInfo.BasicLimitInformation.LimitFlags &
//             JOB_OBJECT_LIMIT_PRIORITY_CLASS)
//         {
//             switch (LocalInfo.BasicLimitInformation.PriorityClass)
//             {
//                 case IDLE_PRIORITY_CLASS:
//                     LocalInfo.BasicLimitInformation.PriorityClass =
//                     PROCESS_PRIORITY_CLASS_IDLE;
//                     break;
//
//                 case BELOW_NORMAL_PRIORITY_CLASS:
//                     LocalInfo.BasicLimitInformation.PriorityClass =
//                     PROCESS_PRIORITY_CLASS_BELOW_NORMAL;
//                     break;
//
//                 case NORMAL_PRIORITY_CLASS:
//                     LocalInfo.BasicLimitInformation.PriorityClass =
//                     PROCESS_PRIORITY_CLASS_NORMAL;
//                     break;
//
//                 case ABOVE_NORMAL_PRIORITY_CLASS:
//                     LocalInfo.BasicLimitInformation.PriorityClass =
//                     PROCESS_PRIORITY_CLASS_ABOVE_NORMAL;
//                     break;
//
//                 case HIGH_PRIORITY_CLASS:
//                     LocalInfo.BasicLimitInformation.PriorityClass =
//                     PROCESS_PRIORITY_CLASS_HIGH;
//                     break;
//
//                 case REALTIME_PRIORITY_CLASS:
//                     LocalInfo.BasicLimitInformation.PriorityClass =
//                     PROCESS_PRIORITY_CLASS_REALTIME;
//                     break;
//
//                 default:
//                     LocalInfo.BasicLimitInformation.PriorityClass =
//                     PROCESS_PRIORITY_CLASS_NORMAL;
//                     break;
//             }
//         }
//
//         if (LocalInfo.BasicLimitInformation.LimitFlags &
//             JOB_OBJECT_LIMIT_WORKINGSET)
//         {
//             Status = RtlAcquirePrivilege(&Privilege, 1, 0, &State);
//         }
//     }
//
//     Status = NtSetInformationJobObject(hJob,
//                                        JobObjectInformationClass,
//                                        JobInfo,
//                                        ExpectedSize);
//     if (NT_SUCCESS(Status))
//     {
//         if (State != NULL) RtlReleasePrivilege(State);
//         return TRUE;
//     }
//
//     BaseSetLastNTError(Status);
//     return FALSE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// TerminateJobObject(IN HANDLE hJob,
//                    IN UINT uExitCode)
// {
//     NTSTATUS Status;
//
//     Status = NtTerminateJobObject(hJob, uExitCode);
//     if (!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// CreateJobSet(IN ULONG NumJob,
//              IN PJOB_SET_ARRAY UserJobSet,
//              IN ULONG Flags)
// {
//     NTSTATUS Status;
//
//     Status = NtCreateJobSet(NumJob, UserJobSet, Flags);
//     if (!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
