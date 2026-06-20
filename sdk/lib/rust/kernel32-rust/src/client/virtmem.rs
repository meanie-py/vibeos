use ntdll_sys::nt::*;
use core::ffi::c_void;
use crate::k32_externs::{BaseSetLastNTError, SetLastError};
use crate::string::nls_subst::NT_SUCCESS;

extern "system" {
    fn GetCurrentProcess() -> HANDLE;
}

// UlongToPtr — winnt.h inline (zero-extend a ULONG to a pointer-sized value).
#[inline]
unsafe fn UlongToPtr(ul: u32) -> PVOID {
    ul as usize as PVOID
}

// ---- file-scope preamble (translated) ----
#[cfg_attr(not(debug_assertions), allow(unused_imports))]
#[no_mangle]
pub unsafe extern "system" fn VirtualAllocEx(
    hProcess: HANDLE,
    mut lpAddress: LPVOID,
    mut dwSize: SIZE_T,
    flAllocationType: DWORD,
    flProtect: DWORD,
) -> LPVOID {
    let mut Status: NTSTATUS;

    /* Make sure the address is within the granularity of the system (64K) */
    if !lpAddress.is_null()
        && (lpAddress as usize)
            < UlongToPtr((*BaseStaticServerData).SysInfo.AllocationGranularity as u32) as usize
    {
        /* Fail the call */
        SetLastError(ERROR_INVALID_PARAMETER);
        return core::ptr::null_mut();
    }

    /* Allocate the memory */
    Status = NtAllocateVirtualMemory(
        hProcess,
        &raw mut lpAddress,
        0,
        &raw mut dwSize,
        flAllocationType,
        flProtect,
    );

    /* Check for status */
    if !NT_SUCCESS(Status) {
        /* We failed */
        BaseSetLastNTError(Status);
        return core::ptr::null_mut();
    }

    /* Return the allocated address */
    lpAddress
}

#[no_mangle]
pub unsafe extern "system" fn VirtualAlloc(
    lpAddress: LPVOID,
    dwSize: SIZE_T,
    flAllocationType: DWORD,
    flProtect: DWORD,
) -> LPVOID {
    return VirtualAllocEx(
        GetCurrentProcess(),
        lpAddress,
        dwSize,
        flAllocationType,
        flProtect,
    );
}
#[no_mangle]
pub unsafe extern "system" fn VirtualFreeEx(
    hProcess: HANDLE,
    mut lpAddress: LPVOID,
    mut dwSize: SIZE_T,
    dwFreeType: DWORD,
) -> BOOL {
    let mut Status: NTSTATUS;
    if dwSize == 0 || dwFreeType & MEM_RELEASE == 0 {
        Status = NtFreeVirtualMemory(
            hProcess,
            &raw mut lpAddress,
            &raw mut dwSize,
            dwFreeType,
        );
        if !NT_SUCCESS(Status) {
            BaseSetLastNTError(Status);
            return FALSE as BOOL;
        }
        return TRUE as BOOL;
    }
    BaseSetLastNTError(STATUS_INVALID_PARAMETER);
    return FALSE as BOOL;
}
#[no_mangle]
pub unsafe extern "system" fn VirtualFree(
    lpAddress: LPVOID,
    dwSize: SIZE_T,
    dwFreeType: DWORD,
) -> BOOL {
    return VirtualFreeEx(GetCurrentProcess(), lpAddress, dwSize, dwFreeType);
}

#[no_mangle]
pub unsafe extern "system" fn VirtualProtect(
    lpAddress: LPVOID,
    dwSize: SIZE_T,
    flNewProtect: DWORD,
    lpflOldProtect: PDWORD,
) -> BOOL {
    VirtualProtectEx(
        GetCurrentProcess(),
        lpAddress,
        dwSize,
        flNewProtect,
        lpflOldProtect,
    )
}
#[no_mangle]
pub unsafe extern "system" fn VirtualProtectEx(
    hProcess: HANDLE,
    mut lpAddress: LPVOID,
    mut dwSize: SIZE_T,
    flNewProtect: DWORD,
    lpflOldProtect: PDWORD,
) -> BOOL {
    let mut Status: NTSTATUS;
    Status = NtProtectVirtualMemory(
        hProcess,
        core::ptr::addr_of_mut!(lpAddress),
        core::ptr::addr_of_mut!(dwSize),
        flNewProtect,
        lpflOldProtect as PULONG,
    );
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return 0;
    }
    return 1;
}
#[no_mangle]
pub unsafe extern "system" fn VirtualLock(
    lpAddress: LPVOID,
    dwSize: SIZE_T,
) -> BOOL {
    let mut Status: NTSTATUS = 0;
    let mut RegionSize: SIZE_T = dwSize;
    let mut BaseAddress: PVOID = lpAddress as PVOID;

    Status = NtLockVirtualMemory(
        NtCurrentProcess(),
        &raw mut BaseAddress,
        &raw mut RegionSize,
        MAP_PROCESS as ULONG,
    );

    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return FALSE as BOOL;
    }

    return TRUE as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn VirtualQuery(
    lpAddress: LPCVOID,
    lpBuffer: PMEMORY_BASIC_INFORMATION,
    dwLength: SIZE_T,
) -> SIZE_T {
    return VirtualQueryEx(
        -(1i32) as isize as HANDLE,
        lpAddress,
        lpBuffer,
        dwLength,
    );
}

#[no_mangle]
pub unsafe extern "system" fn VirtualQueryEx(
    hProcess: HANDLE,
    lpAddress: LPCVOID,
    lpBuffer: PMEMORY_BASIC_INFORMATION,
    dwLength: SIZE_T,
) -> SIZE_T {
    let mut Status: NTSTATUS = 0;
    let mut ResultLength: SIZE_T = 0;

    Status = NtQueryVirtualMemory(
        hProcess,
        lpAddress as LPVOID,
        MemoryBasicInformation,
        lpBuffer as *mut _,
        dwLength,
        &mut ResultLength as *mut SIZE_T,
    );
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return 0;
    }
    return ResultLength;
}

#[no_mangle]
pub unsafe extern "system" fn VirtualUnlock(
    lpAddress: LPVOID,
    dwSize: SIZE_T,
) -> BOOL {
    let mut Status: NTSTATUS = 0;
    let mut RegionSize: SIZE_T = dwSize;
    let mut BaseAddress: PVOID = lpAddress as PVOID;

    Status = NtUnlockVirtualMemory(
        NtCurrentProcess(),
        &mut BaseAddress as *mut PVOID,
        &mut RegionSize as *mut SIZE_T,
        MAP_PROCESS,
    );

    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return FALSE as BOOL;
    }

    return TRUE as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn GetWriteWatch(
    dwFlags: DWORD,
    lpBaseAddress: PVOID,
    dwRegionSize: SIZE_T,
    lpAddresses: *mut PVOID,
    lpdwCount: PULONG_PTR,
    lpdwGranularity: PULONG,
) -> UINT {
    let mut Status: NTSTATUS;
    Status = NtGetWriteWatch(
        GetCurrentProcess(),
        dwFlags,
        lpBaseAddress,
        dwRegionSize,
        lpAddresses,
        lpdwCount,
        lpdwGranularity,
    );
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return !0u32;
    }
    return 0u32;
}
#[no_mangle]
pub unsafe extern "system" fn ResetWriteWatch(
    lpBaseAddress: LPVOID,
    dwRegionSize: SIZE_T,
) -> UINT {
    let mut Status: NTSTATUS = 0;
    Status = NtResetWriteWatch(
        NtCurrentProcess(),
        lpBaseAddress,
        dwRegionSize,
    );
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return !0u32;
    }
    return 0u32;
}

#[no_mangle]
pub unsafe extern "system" fn AllocateUserPhysicalPages(
    hProcess: HANDLE,
    NumberOfPages: PULONG_PTR,
    UserPfnArray: PULONG_PTR,
) -> BOOL {
    let mut Status: NTSTATUS;
    Status = NtAllocateUserPhysicalPages(hProcess, NumberOfPages, UserPfnArray);
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return 0i32;
    }
    return 1i32;
}

#[no_mangle]
pub unsafe extern "system" fn FreeUserPhysicalPages(
    hProcess: HANDLE,
    NumberOfPages: PULONG_PTR,
    PageArray: PULONG_PTR,
) -> BOOL {
    let mut Status: NTSTATUS;
    Status = NtFreeUserPhysicalPages(hProcess, NumberOfPages, PageArray);
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return 0i32;
    }
    return 1i32;
}

#[no_mangle]
pub unsafe extern "system" fn MapUserPhysicalPages(
    VirtualAddress: PVOID,
    NumberOfPages: ULONG_PTR,
    PageArray: PULONG_PTR,
) -> BOOL {
    let Status: NTSTATUS;
    Status = NtMapUserPhysicalPages(VirtualAddress, NumberOfPages, PageArray);
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return 0;
    }
    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn MapUserPhysicalPagesScatter(
    VirtualAddresses: *mut *mut core::ffi::c_void,
    NumberOfPages: ULONG_PTR,
    PageArray: PULONG_PTR,
) -> BOOL {
    let Status: NTSTATUS;
    Status = NtMapUserPhysicalPagesScatter(VirtualAddresses, NumberOfPages, PageArray);
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
//  * PROJECT:         ReactOS Win32 Base API
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            dll/win32/kernel32/client/virtmem.c
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
// LPVOID
// NTAPI
// VirtualAllocEx(IN HANDLE hProcess,
//                IN LPVOID lpAddress,
//                IN SIZE_T dwSize,
//                IN DWORD flAllocationType,
//                IN DWORD flProtect)
// {
//     NTSTATUS Status;
//
//     /* Make sure the address is within the granularity of the system (64K) */
//     if ((lpAddress != NULL) &&
//         (lpAddress < UlongToPtr(BaseStaticServerData->SysInfo.AllocationGranularity)))
//     {
//         /* Fail the call */
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return NULL;
//     }
//
//     /* Allocate the memory */
//     Status = NtAllocateVirtualMemory(hProcess,
//                                      &lpAddress,
//                                      0,
//                                      &dwSize,
//                                      flAllocationType,
//                                      flProtect);
//
//     /* Check for status */
//     if (!NT_SUCCESS(Status))
//     {
//         /* We failed */
//         BaseSetLastNTError(Status);
//         return NULL;
//     }
//
//     /* Return the allocated address */
//     return lpAddress;
// }
//
// /*
//  * @implemented
//  */
// LPVOID
// NTAPI
// VirtualAlloc(IN LPVOID lpAddress,
//              IN SIZE_T dwSize,
//              IN DWORD flAllocationType,
//              IN DWORD flProtect)
// {
//     /* Call the extended API */
//     return VirtualAllocEx(GetCurrentProcess(),
//                           lpAddress,
//                           dwSize,
//                           flAllocationType,
//                           flProtect);
// }
//
// /*
//  * @implemented
//  */
// BOOL
// NTAPI
// VirtualFreeEx(IN HANDLE hProcess,
//               IN LPVOID lpAddress,
//               IN SIZE_T dwSize,
//               IN DWORD dwFreeType)
// {
//     NTSTATUS Status;
//
//     /* Validate size and flags */
//     if (!(dwSize) || !(dwFreeType & MEM_RELEASE))
//     {
//         /* Free the memory */
//         Status = NtFreeVirtualMemory(hProcess,
//                                      &lpAddress,
//                                      &dwSize,
//                                      dwFreeType);
//         if (!NT_SUCCESS(Status))
//         {
//             /* We failed */
//             BaseSetLastNTError(Status);
//             return FALSE;
//         }
//
//         /* Return success */
//         return TRUE;
//     }
//
//     /* Invalid combo */
//     BaseSetLastNTError(STATUS_INVALID_PARAMETER);
//     return FALSE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// NTAPI
// VirtualFree(IN LPVOID lpAddress,
//             IN SIZE_T dwSize,
//             IN DWORD dwFreeType)
// {
//     /* Call the extended API */
//     return VirtualFreeEx(GetCurrentProcess(),
//                          lpAddress,
//                          dwSize,
//                          dwFreeType);
// }
//
// /*
//  * @implemented
//  */
// BOOL
// NTAPI
// VirtualProtect(IN LPVOID lpAddress,
//                IN SIZE_T dwSize,
//                IN DWORD flNewProtect,
//                OUT PDWORD lpflOldProtect)
// {
//     /* Call the extended API */
//     return VirtualProtectEx(GetCurrentProcess(),
//                             lpAddress,
//                             dwSize,
//                             flNewProtect,
//                             lpflOldProtect);
// }
//
// /*
//  * @implemented
//  */
// BOOL
// NTAPI
// VirtualProtectEx(IN HANDLE hProcess,
//                  IN LPVOID lpAddress,
//                  IN SIZE_T dwSize,
//                  IN DWORD flNewProtect,
//                  OUT PDWORD lpflOldProtect)
// {
//     NTSTATUS Status;
//
//     /* Change the protection */
//     Status = NtProtectVirtualMemory(hProcess,
//                                     &lpAddress,
//                                     &dwSize,
//                                     flNewProtect,
//                                     (PULONG)lpflOldProtect);
//     if (!NT_SUCCESS(Status))
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
// /*
//  * @implemented
//  */
// BOOL
// NTAPI
// VirtualLock(IN LPVOID lpAddress,
//             IN SIZE_T dwSize)
// {
//     NTSTATUS Status;
//     SIZE_T RegionSize = dwSize;
//     PVOID BaseAddress = lpAddress;
//
//     /* Lock the memory */
//     Status = NtLockVirtualMemory(NtCurrentProcess(),
//                                  &BaseAddress,
//                                  &RegionSize,
//                                  MAP_PROCESS);
//     if (!NT_SUCCESS(Status))
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
// /*
//  * @implemented
//  */
// SIZE_T
// NTAPI
// VirtualQuery(IN LPCVOID lpAddress,
//              OUT PMEMORY_BASIC_INFORMATION lpBuffer,
//              IN SIZE_T dwLength)
// {
//     /* Call the extended API */
//     return VirtualQueryEx(NtCurrentProcess(),
//                           lpAddress,
//                           lpBuffer,
//                           dwLength);
// }
//
// /*
//  * @implemented
//  */
// SIZE_T
// NTAPI
// VirtualQueryEx(IN HANDLE hProcess,
//                IN LPCVOID lpAddress,
//                OUT PMEMORY_BASIC_INFORMATION lpBuffer,
//                IN SIZE_T dwLength)
// {
//     NTSTATUS Status;
//     SIZE_T ResultLength;
//
//     /* Query basic information */
//     Status = NtQueryVirtualMemory(hProcess,
//                                   (LPVOID)lpAddress,
//                                   MemoryBasicInformation,
//                                   lpBuffer,
//                                   dwLength,
//                                   &ResultLength);
//     if (!NT_SUCCESS(Status))
//     {
//         /* We failed */
//         BaseSetLastNTError(Status);
//         return 0;
//     }
//
//     /* Return the length returned */
//     return ResultLength;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// NTAPI
// VirtualUnlock(IN LPVOID lpAddress,
//               IN SIZE_T dwSize)
// {
//     NTSTATUS Status;
//     SIZE_T RegionSize = dwSize;
//     PVOID BaseAddress = lpAddress;
//
//     /* Lock the memory */
//     Status = NtUnlockVirtualMemory(NtCurrentProcess(),
//                                    &BaseAddress,
//                                    &RegionSize,
//                                    MAP_PROCESS);
//     if (!NT_SUCCESS(Status))
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
// /*
//  * @implemented
//  */
// UINT
// WINAPI
// GetWriteWatch(IN DWORD dwFlags,
//               IN PVOID lpBaseAddress,
//               IN SIZE_T dwRegionSize,
//               IN PVOID *lpAddresses,
//               OUT PULONG_PTR lpdwCount,
//               OUT PULONG lpdwGranularity)
// {
//     NTSTATUS Status;
//
//     Status = NtGetWriteWatch(GetCurrentProcess(),
//                              dwFlags,
//                              lpBaseAddress,
//                              dwRegionSize,
//                              lpAddresses,
//                              lpdwCount,
//                              lpdwGranularity);
//     if (!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError(Status);
//         return -1;
//     }
//
//     return 0;
// }
//
// /*
//  * @implemented
//  */
// UINT
// WINAPI
// ResetWriteWatch(IN LPVOID lpBaseAddress,
//                 IN SIZE_T dwRegionSize)
// {
//     NTSTATUS Status;
//
//     Status = NtResetWriteWatch(NtCurrentProcess(),
//                                lpBaseAddress,
//                                dwRegionSize);
//     if (!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError(Status);
//         return -1;
//     }
//
//     return 0;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// AllocateUserPhysicalPages(IN HANDLE hProcess,
//                           IN PULONG_PTR NumberOfPages,
//                           OUT PULONG_PTR UserPfnArray)
// {
//     NTSTATUS Status;
//
//     Status = NtAllocateUserPhysicalPages(hProcess, NumberOfPages, UserPfnArray);
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
// FreeUserPhysicalPages(IN HANDLE hProcess,
//                       IN PULONG_PTR NumberOfPages,
//                       IN PULONG_PTR PageArray)
// {
//     NTSTATUS Status;
//
//     Status = NtFreeUserPhysicalPages(hProcess, NumberOfPages, PageArray);
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
// MapUserPhysicalPages(IN PVOID VirtualAddress,
//                      IN ULONG_PTR NumberOfPages,
//                      OUT PULONG_PTR PageArray OPTIONAL)
// {
//     NTSTATUS Status;
//
//     Status = NtMapUserPhysicalPages(VirtualAddress, NumberOfPages, PageArray);
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
// MapUserPhysicalPagesScatter(IN PVOID *VirtualAddresses,
//                             IN ULONG_PTR NumberOfPages,
//                             OUT PULONG_PTR PageArray OPTIONAL)
// {
//     NTSTATUS Status;
//
//     Status = NtMapUserPhysicalPagesScatter(VirtualAddresses,
//                                            NumberOfPages,
//                                            PageArray);
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
