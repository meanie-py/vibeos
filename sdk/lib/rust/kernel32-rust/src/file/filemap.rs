use ntdll_sys::nt::*;
use crate::prelude::*;
use core::ffi::c_void;
use crate::k32_externs::{BaseFormatObjectAttributes, SetLastError};

pub const STATUS_INVALID_PAGE_PROTECTION: NTSTATUS = 0xC000_0045u32 as NTSTATUS;
pub const STATUS_NOT_MAPPED_DATA: NTSTATUS = 0xC000_0088u32 as NTSTATUS;

extern "system" {
    fn BasepGetNamedObjectDirectory() -> HANDLE;
}

// ---- file-scope preamble (translated) ----

#[no_mangle]
pub unsafe extern "system" fn CreateFileMappingA(
    hFile: HANDLE,
    lpFileMappingAttributes: LPSECURITY_ATTRIBUTES,
    flProtect: DWORD,
    dwMaximumSizeHigh: DWORD,
    dwMaximumSizeLow: DWORD,
    lpName: LPCSTR,
) -> HANDLE {
    // ConvertWin32AnsiObjectApiToUnicodeApi(FileMapping, lpName, ...) — expanded.
    if lpName.is_null() {
        return CreateFileMappingW(
            hFile,
            lpFileMappingAttributes,
            flProtect,
            dwMaximumSizeHigh,
            dwMaximumSizeLow,
            core::ptr::null(),
        );
    }

    let UnicodeCache: PUNICODE_STRING = &raw mut (*NtCurrentTeb()).StaticUnicodeString;
    let mut AnsiName: ANSI_STRING = core::mem::zeroed();
    RtlInitAnsiString(&mut AnsiName, lpName);
    let Status: NTSTATUS = RtlAnsiStringToUnicodeString(UnicodeCache, &mut AnsiName, 0);
    if NT_SUCCESS(Status) {
        return CreateFileMappingW(
            hFile,
            lpFileMappingAttributes,
            flProtect,
            dwMaximumSizeHigh,
            dwMaximumSizeLow,
            (*UnicodeCache).Buffer,
        );
    }

    if Status == STATUS_BUFFER_OVERFLOW {
        SetLastError(ERROR_FILENAME_EXCED_RANGE);
    } else {
        BaseSetLastNTError(Status);
    }
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn CreateFileMappingW(
    mut hFile: HANDLE,
    lpFileMappingAttributes: LPSECURITY_ATTRIBUTES,
    mut flProtect: DWORD,
    dwMaximumSizeHigh: DWORD,
    dwMaximumSizeLow: DWORD,
    lpName: LPCWSTR,
) -> HANDLE {
    let mut Status: NTSTATUS;
    let mut SectionHandle: HANDLE = core::ptr::null_mut();
    let mut LocalAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let ObjectAttributes: POBJECT_ATTRIBUTES;
    let mut SectionName: _UNICODE_STRING = core::mem::zeroed();
    let mut DesiredAccess: ACCESS_MASK;
    let mut LocalSize: LARGE_INTEGER = core::mem::zeroed();
    let mut SectionSize: PLARGE_INTEGER;
    let mut Attributes: ULONG;

    /* Set default access */
    DesiredAccess = (STANDARD_RIGHTS_REQUIRED | SECTION_QUERY | SECTION_MAP_READ) as ACCESS_MASK;

    /* Get the attributes for the actual allocation and cleanup flProtect */
    Attributes = (flProtect & (SEC_FILE | SEC_IMAGE | SEC_RESERVE | SEC_NOCACHE | SEC_COMMIT | SEC_LARGE_PAGES)) as ULONG;
    flProtect ^= Attributes as DWORD;

    /* If the caller didn't say anything, assume SEC_COMMIT */
    if Attributes == 0 {
        Attributes = SEC_COMMIT as ULONG;
    }

    /* Now check if the caller wanted write access */
    if flProtect == PAGE_READWRITE as DWORD {
        /* Give it */
        DesiredAccess |= SECTION_MAP_WRITE as ACCESS_MASK;
    } else if flProtect == PAGE_EXECUTE_READWRITE as DWORD {
        /* Give it */
        DesiredAccess |= (SECTION_MAP_WRITE | SECTION_MAP_EXECUTE) as ACCESS_MASK;
    } else if flProtect == PAGE_EXECUTE_READ as DWORD {
        /* Give it */
        DesiredAccess |= SECTION_MAP_EXECUTE as ACCESS_MASK;
    } else if (flProtect == PAGE_EXECUTE_WRITECOPY as DWORD) &&
              ((*NtCurrentPeb()).OSMajorVersion >= 6)
    {
        /* Give it */
        DesiredAccess |= SECTION_MAP_EXECUTE as ACCESS_MASK;
    } else if (flProtect != PAGE_READONLY as DWORD) && (flProtect != PAGE_WRITECOPY as DWORD) {
        SetLastError(ERROR_INVALID_PARAMETER);
        return core::ptr::null_mut();
    }

    /* Now check if we got a name */
    if !lpName.is_null() {
        RtlInitUnicodeString(&mut SectionName, lpName);
    }

    /* Now convert the object attributes */
    ObjectAttributes = BaseFormatObjectAttributes(
        &mut LocalAttributes,
        lpFileMappingAttributes,
        if !lpName.is_null() { &mut SectionName } else { core::ptr::null_mut() },
    );

    /* Check if we got a size */
    if dwMaximumSizeLow != 0 || dwMaximumSizeHigh != 0 {
        /* Use a LARGE_INTEGER and convert */
        SectionSize = &mut LocalSize;
        (*SectionSize).__bindgen_anon_1.LowPart = dwMaximumSizeLow;
        (*SectionSize).__bindgen_anon_1.HighPart = dwMaximumSizeHigh as LONG;
    } else {
        SectionSize = core::ptr::null_mut();
    }

    /* Make sure the handle is valid */
    if hFile == INVALID_HANDLE_VALUE {
        /* It's not, we'll only go on if we have a size */
        // C-exact: REASSIGN the param (C `hFile = NULL`) so NtCreateSection
        // creates a pagefile-backed section. The translation had `let hFile`
        // here — a shadow that died at the block end, so NtCreateSection got
        // INVALID_HANDLE_VALUE (-1, the current-process pseudo-handle) as a
        // *file* handle and EVERY anonymous CreateFileMapping in the OS
        // failed (kbswitch "SetHooks failed", SHAllocShared "Out of memory").
        hFile = core::ptr::null_mut();
        if SectionSize.is_null() {
            /* No size, so this isn't a valid non-mapped section */
            SetLastError(ERROR_INVALID_PARAMETER);
            return core::ptr::null_mut();
        }
    }

    /* Now create the actual section */
    Status = NtCreateSection(
        &mut SectionHandle,
        DesiredAccess,
        ObjectAttributes,
        SectionSize,
        flProtect as ULONG,
        Attributes,
        hFile,
    );
    if !NT_SUCCESS(Status) {
        /* We failed */
        BaseSetLastNTError(Status);
        return core::ptr::null_mut();
    } else if Status == STATUS_OBJECT_NAME_EXISTS {
        SetLastError(ERROR_ALREADY_EXISTS);
    } else {
        SetLastError(ERROR_SUCCESS);
    }

    /* Return the section */
    SectionHandle
}
#[no_mangle]
pub unsafe extern "system" fn MapViewOfFileEx(
    hFileMappingObject: HANDLE,
    dwDesiredAccess: DWORD,
    dwFileOffsetHigh: DWORD,
    dwFileOffsetLow: DWORD,
    dwNumberOfBytesToMap: SIZE_T,
    lpBaseAddress: LPVOID,
) -> LPVOID {
    let mut Status: NTSTATUS;
    let mut SectionOffset: LARGE_INTEGER = core::mem::zeroed();
    let mut ViewSize: SIZE_T;
    let mut Protect: ULONG;
    let mut ViewBase: LPVOID;

    /* Convert the offset */
    SectionOffset.u.LowPart = dwFileOffsetLow;
    SectionOffset.u.HighPart = dwFileOffsetHigh as LONG;

    /* Save the size and base */
    ViewBase = lpBaseAddress;
    ViewSize = dwNumberOfBytesToMap;

    /* Convert flags to NT Protection Attributes */
    if dwDesiredAccess == FILE_MAP_COPY as DWORD
    {
        Protect = PAGE_WRITECOPY as ULONG;
    }
    else if dwDesiredAccess as DWORD & FILE_MAP_WRITE as DWORD != 0
    {
        Protect = if dwDesiredAccess as DWORD & FILE_MAP_EXECUTE as DWORD != 0
        {
            PAGE_EXECUTE_READWRITE as ULONG
        }
        else
        {
            PAGE_READWRITE as ULONG
        };
    }
    else if dwDesiredAccess as DWORD & FILE_MAP_READ as DWORD != 0
    {
        Protect = if dwDesiredAccess as DWORD & FILE_MAP_EXECUTE as DWORD != 0
        {
            PAGE_EXECUTE_READ as ULONG
        }
        else
        {
            PAGE_READONLY as ULONG
        };
    }
    else
    {
        Protect = PAGE_NOACCESS as ULONG;
    }

    /* Map the section */
    Status = NtMapViewOfSection(
        hFileMappingObject,
        NtCurrentProcess(),
        &mut ViewBase as *mut LPVOID,
        0,
        0,
        &mut SectionOffset as *mut LARGE_INTEGER,
        &mut ViewSize as *mut SIZE_T,
        ViewShare,
        0,
        Protect,
    );
    if !NT_SUCCESS(Status)
    {
        /* We failed */
        BaseSetLastNTError(Status);
        return core::ptr::null_mut();
    }

    /* Return the base */
    return ViewBase;
}

#[no_mangle]
pub unsafe extern "system" fn MapViewOfFile(
    hFileMappingObject: HANDLE,
    dwDesiredAccess: DWORD,
    dwFileOffsetHigh: DWORD,
    dwFileOffsetLow: DWORD,
    dwNumberOfBytesToMap: SIZE_T,
) -> LPVOID {
    MapViewOfFileEx(
        hFileMappingObject,
        dwDesiredAccess,
        dwFileOffsetHigh,
        dwFileOffsetLow,
        dwNumberOfBytesToMap,
        core::ptr::null_mut(),
    )
}

#[no_mangle]
pub unsafe extern "system" fn UnmapViewOfFile(lpBaseAddress: LPCVOID) -> BOOL {
    let mut Status: NTSTATUS;
    Status = NtUnmapViewOfSection(NtCurrentProcess(), lpBaseAddress as PVOID);
    if !NT_SUCCESS(Status) {
        if Status == STATUS_INVALID_PAGE_PROTECTION {
            if RtlFlushSecureMemoryCache(lpBaseAddress as PVOID, 0) != 0 {
                Status = NtUnmapViewOfSection(NtCurrentProcess(), lpBaseAddress as PVOID);
                if NT_SUCCESS(Status) {
                    return TRUE as BOOL;
                }
            }
        }
        BaseSetLastNTError(Status);
        return FALSE as BOOL;
    }
    return TRUE as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn OpenFileMappingA(
    dwDesiredAccess: DWORD,
    bInheritHandle: BOOL,
    lpName: LPCSTR,
) -> HANDLE {
    // ConvertOpenWin32AnsiObjectApiToUnicodeApi(FileMapping, dwDesiredAccess, bInheritHandle, lpName) — expanded.
    if lpName.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return core::ptr::null_mut();
    }

    let UnicodeCache: PUNICODE_STRING = &raw mut (*NtCurrentTeb()).StaticUnicodeString;
    let mut AnsiName: ANSI_STRING = core::mem::zeroed();
    RtlInitAnsiString(&mut AnsiName, lpName);
    let Status: NTSTATUS = RtlAnsiStringToUnicodeString(UnicodeCache, &mut AnsiName, 0);
    if NT_SUCCESS(Status) {
        return OpenFileMappingW(dwDesiredAccess, bInheritHandle, (*UnicodeCache).Buffer);
    }

    if Status == STATUS_BUFFER_OVERFLOW {
        SetLastError(ERROR_FILENAME_EXCED_RANGE);
    } else {
        BaseSetLastNTError(Status);
    }
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn OpenFileMappingW(
    mut dwDesiredAccess: DWORD,
    bInheritHandle: BOOL,
    lpName: LPCWSTR,
) -> HANDLE {
    let mut Status: NTSTATUS;
    let mut SectionHandle: HANDLE = core::ptr::null_mut();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut UnicodeName: _UNICODE_STRING = core::mem::zeroed();

    if lpName.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return core::ptr::null_mut();
    }

    RtlInitUnicodeString(&mut UnicodeName, lpName);
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = BasepGetNamedObjectDirectory();
    ObjectAttributes.Attributes = if bInheritHandle != 0 { OBJ_INHERIT as ULONG } else { 0 };
    ObjectAttributes.ObjectName = &mut UnicodeName as *mut _UNICODE_STRING;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    if dwDesiredAccess == FILE_MAP_COPY as DWORD {
        dwDesiredAccess = SECTION_MAP_READ as DWORD;
    } else if dwDesiredAccess as u32 & FILE_MAP_EXECUTE as u32 != 0 {
        dwDesiredAccess = (dwDesiredAccess as u32 & !FILE_MAP_EXECUTE as u32 | SECTION_MAP_EXECUTE as u32) as DWORD;
    }

    Status = NtOpenSection(
        &mut SectionHandle,
        dwDesiredAccess as ACCESS_MASK,
        &mut ObjectAttributes,
    );
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return core::ptr::null_mut();
    }

    SectionHandle
}
#[no_mangle]
pub unsafe extern "system" fn FlushViewOfFile(
    lpBaseAddress: LPCVOID,
    dwNumberOfBytesToFlush: SIZE_T,
) -> BOOL {
    let mut Status: NTSTATUS;
    let mut BaseAddress: PVOID = lpBaseAddress as PVOID;
    let mut NumberOfBytesToFlush: SIZE_T = dwNumberOfBytesToFlush;
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();

    Status = NtFlushVirtualMemory(
        NtCurrentProcess(),
        &mut BaseAddress as *mut PVOID,
        &mut NumberOfBytesToFlush as *mut SIZE_T,
        &mut IoStatusBlock as *mut _IO_STATUS_BLOCK,
    );
    if !NT_SUCCESS(Status) && Status != STATUS_NOT_MAPPED_DATA {
        BaseSetLastNTError(Status);
        return 0;
    }

    return 1;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Win32 Base API
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            dll/win32/kernel32/client/file/filemap.c
//  * PURPOSE:         Handles virtual memory APIs
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
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
// NTAPI
// CreateFileMappingA(IN HANDLE hFile,
//                    IN LPSECURITY_ATTRIBUTES lpFileMappingAttributes,
//                    IN DWORD flProtect,
//                    IN DWORD dwMaximumSizeHigh,
//                    IN DWORD dwMaximumSizeLow,
//                    IN LPCSTR lpName)
// {
//     /* Call the W(ide) function */
//     ConvertWin32AnsiObjectApiToUnicodeApi(FileMapping,
//                                           lpName,
//                                           hFile,
//                                           lpFileMappingAttributes,
//                                           flProtect,
//                                           dwMaximumSizeHigh,
//                                           dwMaximumSizeLow);
// }
//
// /*
//  * @implemented
//  */
// HANDLE
// NTAPI
// CreateFileMappingW(HANDLE hFile,
//                    LPSECURITY_ATTRIBUTES lpFileMappingAttributes,
//                    DWORD flProtect,
//                    DWORD dwMaximumSizeHigh,
//                    DWORD dwMaximumSizeLow,
//                    LPCWSTR lpName)
// {
//     NTSTATUS Status;
//     HANDLE SectionHandle;
//     OBJECT_ATTRIBUTES LocalAttributes;
//     POBJECT_ATTRIBUTES ObjectAttributes;
//     UNICODE_STRING SectionName;
//     ACCESS_MASK DesiredAccess;
//     LARGE_INTEGER LocalSize;
//     PLARGE_INTEGER SectionSize = NULL;
//     ULONG Attributes;
//
//     /* Set default access */
//     DesiredAccess = STANDARD_RIGHTS_REQUIRED | SECTION_QUERY | SECTION_MAP_READ;
//
//     /* Get the attributes for the actual allocation and cleanup flProtect */
//     Attributes = flProtect & (SEC_FILE | SEC_IMAGE | SEC_RESERVE | SEC_NOCACHE | SEC_COMMIT | SEC_LARGE_PAGES);
//     flProtect ^= Attributes;
//
//     /* If the caller didn't say anything, assume SEC_COMMIT */
//     if (!Attributes) Attributes = SEC_COMMIT;
//
//     /* Now check if the caller wanted write access */
//     if (flProtect == PAGE_READWRITE)
//     {
//         /* Give it */
//         DesiredAccess |= SECTION_MAP_WRITE;
//     }
//     else if (flProtect == PAGE_EXECUTE_READWRITE)
//     {
//         /* Give it */
//         DesiredAccess |= (SECTION_MAP_WRITE | SECTION_MAP_EXECUTE);
//     }
//     else if (flProtect == PAGE_EXECUTE_READ)
//     {
//         /* Give it */
//         DesiredAccess |= SECTION_MAP_EXECUTE;
//     }
//     else if ((flProtect == PAGE_EXECUTE_WRITECOPY) &&
//              (NtCurrentPeb()->OSMajorVersion >= 6))
//     {
//         /* Give it */
//         DesiredAccess |= SECTION_MAP_EXECUTE;
//     }
//     else if ((flProtect != PAGE_READONLY) && (flProtect != PAGE_WRITECOPY))
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return NULL;
//     }
//
//     /* Now check if we got a name */
//     if (lpName) RtlInitUnicodeString(&SectionName, lpName);
//
//     /* Now convert the object attributes */
//     ObjectAttributes = BaseFormatObjectAttributes(&LocalAttributes,
//                                                     lpFileMappingAttributes,
//                                                     lpName ? &SectionName : NULL);
//
//     /* Check if we got a size */
//     if (dwMaximumSizeLow || dwMaximumSizeHigh)
//     {
//         /* Use a LARGE_INTEGER and convert */
//         SectionSize = &LocalSize;
//         SectionSize->LowPart = dwMaximumSizeLow;
//         SectionSize->HighPart = dwMaximumSizeHigh;
//     }
//
//     /* Make sure the handle is valid */
//     if (hFile == INVALID_HANDLE_VALUE)
//     {
//         /* It's not, we'll only go on if we have a size */
//         hFile = NULL;
//         if (!SectionSize)
//         {
//             /* No size, so this isn't a valid non-mapped section */
//             SetLastError(ERROR_INVALID_PARAMETER);
//             return NULL;
//         }
//     }
//
//     /* Now create the actual section */
//     Status = NtCreateSection(&SectionHandle,
//                              DesiredAccess,
//                              ObjectAttributes,
//                              SectionSize,
//                              flProtect,
//                              Attributes,
//                              hFile);
//     if (!NT_SUCCESS(Status))
//     {
//         /* We failed */
//         BaseSetLastNTError(Status);
//         return NULL;
//     }
//     else if (Status == STATUS_OBJECT_NAME_EXISTS)
//     {
//         SetLastError(ERROR_ALREADY_EXISTS);
//     }
//     else
//     {
//         SetLastError(ERROR_SUCCESS);
//     }
//
//     /* Return the section */
//     return SectionHandle;
// }
//
// /*
//  * @implemented
//  */
// LPVOID
// NTAPI
// MapViewOfFileEx(HANDLE hFileMappingObject,
//                 DWORD dwDesiredAccess,
//                 DWORD dwFileOffsetHigh,
//                 DWORD dwFileOffsetLow,
//                 SIZE_T dwNumberOfBytesToMap,
//                 LPVOID lpBaseAddress)
// {
//     NTSTATUS Status;
//     LARGE_INTEGER SectionOffset;
//     SIZE_T ViewSize;
//     ULONG Protect;
//     LPVOID ViewBase;
//
//     /* Convert the offset */
//     SectionOffset.LowPart = dwFileOffsetLow;
//     SectionOffset.HighPart = dwFileOffsetHigh;
//
//     /* Save the size and base */
//     ViewBase = lpBaseAddress;
//     ViewSize = dwNumberOfBytesToMap;
//
//     /* Convert flags to NT Protection Attributes */
//     if (dwDesiredAccess == FILE_MAP_COPY)
//     {
//         Protect = PAGE_WRITECOPY;
//     }
//     else if (dwDesiredAccess & FILE_MAP_WRITE)
//     {
//         Protect = (dwDesiredAccess & FILE_MAP_EXECUTE) ?
//                    PAGE_EXECUTE_READWRITE : PAGE_READWRITE;
//     }
//     else if (dwDesiredAccess & FILE_MAP_READ)
//     {
//         Protect = (dwDesiredAccess & FILE_MAP_EXECUTE) ?
//                    PAGE_EXECUTE_READ : PAGE_READONLY;
//     }
//     else
//     {
//         Protect = PAGE_NOACCESS;
//     }
//
//     /* Map the section */
//     Status = NtMapViewOfSection(hFileMappingObject,
//                                 NtCurrentProcess(),
//                                 &ViewBase,
//                                 0,
//                                 0,
//                                 &SectionOffset,
//                                 &ViewSize,
//                                 ViewShare,
//                                 0,
//                                 Protect);
//     if (!NT_SUCCESS(Status))
//     {
//         /* We failed */
//         BaseSetLastNTError(Status);
//         return NULL;
//     }
//
//     /* Return the base */
//     return ViewBase;
// }
//
// /*
//  * @implemented
//  */
// LPVOID
// NTAPI
// MapViewOfFile(HANDLE hFileMappingObject,
//               DWORD dwDesiredAccess,
//               DWORD dwFileOffsetHigh,
//               DWORD dwFileOffsetLow,
//               SIZE_T dwNumberOfBytesToMap)
// {
//     /* Call the extended API */
//     return MapViewOfFileEx(hFileMappingObject,
//                            dwDesiredAccess,
//                            dwFileOffsetHigh,
//                            dwFileOffsetLow,
//                            dwNumberOfBytesToMap,
//                            NULL);
// }
//
// /*
//  * @implemented
//  */
// BOOL
// NTAPI
// UnmapViewOfFile(LPCVOID lpBaseAddress)
// {
//     NTSTATUS Status;
//
//     /* Unmap the section */
//     Status = NtUnmapViewOfSection(NtCurrentProcess(), (PVOID)lpBaseAddress);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Check if the pages were protected */
//         if (Status == STATUS_INVALID_PAGE_PROTECTION)
//         {
//             /* Flush the region if it was a "secure memory cache" */
//             if (RtlFlushSecureMemoryCache((PVOID)lpBaseAddress, 0))
//             {
//                 /* Now try to unmap again */
//                 Status = NtUnmapViewOfSection(NtCurrentProcess(), (PVOID)lpBaseAddress);
//                 if (NT_SUCCESS(Status)) return TRUE;
//             }
//         }
//
//         /* We failed */
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     /* Otherwise, return success */
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// HANDLE
// NTAPI
// OpenFileMappingA(IN DWORD dwDesiredAccess,
//                  IN BOOL bInheritHandle,
//                  IN LPCSTR lpName)
// {
//     ConvertOpenWin32AnsiObjectApiToUnicodeApi(FileMapping, dwDesiredAccess, bInheritHandle, lpName);
// }
//
// /*
//  * @implemented
//  */
//  /* FIXME: Convert to the new macros */
// HANDLE
// NTAPI
// OpenFileMappingW(IN DWORD dwDesiredAccess,
//                  IN BOOL bInheritHandle,
//                  IN LPCWSTR lpName)
// {
//     NTSTATUS Status;
//     HANDLE SectionHandle;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     UNICODE_STRING UnicodeName;
//
//     /* We need a name */
//     if (!lpName)
//     {
//         /* Otherwise, fail */
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return NULL;
//     }
//
//     /* Convert attributes */
//     RtlInitUnicodeString(&UnicodeName, lpName);
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &UnicodeName,
//                                (bInheritHandle ? OBJ_INHERIT : 0),
//                                BaseGetNamedObjectDirectory(),
//                                NULL);
//
//     /* Convert COPY to READ */
//     if (dwDesiredAccess == FILE_MAP_COPY)
//     {
//         /* Fixup copy */
//         dwDesiredAccess = SECTION_MAP_READ;
//     }
//     else if (dwDesiredAccess & FILE_MAP_EXECUTE)
//     {
//         /* Fixup execute */
//         dwDesiredAccess = (dwDesiredAccess & ~FILE_MAP_EXECUTE) | SECTION_MAP_EXECUTE;
//     }
//
//     /* Open the section */
//     Status = NtOpenSection(&SectionHandle, dwDesiredAccess, &ObjectAttributes);
//     if (!NT_SUCCESS(Status))
//     {
//         /* We failed */
//         BaseSetLastNTError(Status);
//         return NULL;
//     }
//
//     /* Otherwise, return the handle */
//     return SectionHandle;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// NTAPI
// FlushViewOfFile(IN LPCVOID lpBaseAddress,
//                 IN SIZE_T dwNumberOfBytesToFlush)
// {
//     NTSTATUS Status;
//     PVOID BaseAddress = (PVOID)lpBaseAddress;
//     SIZE_T NumberOfBytesToFlush = dwNumberOfBytesToFlush;
//     IO_STATUS_BLOCK IoStatusBlock;
//
//     /* Flush the view */
//     Status = NtFlushVirtualMemory(NtCurrentProcess(),
//                                   &BaseAddress,
//                                   &NumberOfBytesToFlush,
//                                   &IoStatusBlock);
//     if (!NT_SUCCESS(Status) && (Status != STATUS_NOT_MAPPED_DATA))
//     {
//         /* We failed */
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     /* Return success */
//     return TRUE;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
