use ntdll_sys::nt::*;
use crate::prelude::*;
use crate::k32_externs::{RtlZeroMemory, RtlCopyMemory};
use crate::proc_getters::GetProcessVersion;
use core::ffi::c_void;

extern "system" {
    fn GetFirmwareEnvironmentVariableExW( lpName: LPCWSTR, lpGuid: LPCWSTR, pBuffer: PVOID, nSize: DWORD, pdwAttribubutes: PDWORD, ) -> DWORD;
    fn GetFirmwareEnvironmentVariableExA( lpName: LPCSTR, lpGuid: LPCSTR, pBuffer: PVOID, nSize: DWORD, pdwAttribubutes: PDWORD, ) -> DWORD;
    fn SetFirmwareEnvironmentVariableExW( lpName: LPCWSTR, lpGuid: LPCWSTR, pValue: PVOID, nSize: DWORD, dwAttributes: DWORD, ) -> BOOL;
    fn SetFirmwareEnvironmentVariableExA( lpName: LPCSTR, lpGuid: LPCSTR, pValue: PVOID, nSize: DWORD, dwAttributes: DWORD, ) -> BOOL;
}

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct _SYSTEM_INFO {
    pub wProcessorArchitecture: u16,
    pub wReserved: u16,
    pub dwPageSize: u32,
    pub lpMinimumApplicationAddress: *mut core::ffi::c_void,
    pub lpMaximumApplicationAddress: *mut core::ffi::c_void,
    pub dwActiveProcessorMask: usize,
    pub dwNumberOfProcessors: u32,
    pub dwProcessorType: u32,
    pub dwAllocationGranularity: u32,
    pub wProcessorLevel: u16,
    pub wProcessorRevision: u16,
}

pub type LPSYSTEM_INFO = *mut _SYSTEM_INFO;

pub const PV_NT351: u32 = 0x00030033;

pub const PROCESSOR_ARCHITECTURE_INTEL: u16 = 0;
pub const PROCESSOR_ARCHITECTURE_AMD64: u16 = 9;
pub const PROCESSOR_ARCHITECTURE_IA64: u16 = 6;

pub const PROCESSOR_INTEL_386: u32 = 386;
pub const PROCESSOR_INTEL_486: u32 = 486;
pub const PROCESSOR_INTEL_PENTIUM: u32 = 586;
pub const PROCESSOR_AMD_X8664: u32 = 8664;
pub const PROCESSOR_INTEL_IA64: u32 = 2200;

pub const PROCESSOR_FEATURE_MAX: u32 = 64;

pub const MAXIMUM_PROCESSORS: u32 = 32;

pub const STATUS_BUFFER_TOO_SMALL: i32 = 0xC0000023u32 as i32;
pub const STATUS_INFO_LENGTH_MISMATCH: i32 = 0xC0000004u32 as i32;

pub const ERROR_INVALID_PARAMETER: u32 = 87;
pub const APPMODEL_ERROR_NO_PACKAGE: u32 = 15700;

pub const HEAP_ZERO_MEMORY: u32 = 0x00000008;

pub const VARIABLE_ATTRIBUTE_NON_VOLATILE: u32 = 0x00000001;

// SYSTEM_INFORMATION_CLASS values come from ntdll_sys::nt (enum-typed); local
// shadows removed so the NtQuerySystemInformation enum param type-checks.
pub const SystemFirmwareTable_Enumerate: u32 = 0;
pub const SystemFirmwareTable_Get: u32 = 1;

#[repr(C)]
pub struct _SYSTEM_BASIC_INFORMATION {
    pub Reserved: ULONG,
    pub TimerResolution: ULONG,
    pub PageSize: ULONG,
    pub NumberOfPhysicalPages: ULONG,
    pub LowestPhysicalPageNumber: ULONG,
    pub HighestPhysicalPageNumber: ULONG,
    pub AllocationGranularity: ULONG,
    pub MinimumUserModeAddress: ULONG_PTR,
    pub MaximumUserModeAddress: ULONG_PTR,
    pub ActiveProcessorsAffinityMask: ULONG_PTR,
    pub NumberOfProcessors: CCHAR,
}

pub type PSYSTEM_BASIC_INFORMATION = *mut _SYSTEM_BASIC_INFORMATION;

#[repr(C)]
pub struct _SYSTEM_PROCESSOR_INFORMATION {
    pub ProcessorArchitecture: u16,
    pub ProcessorLevel: u16,
    pub ProcessorRevision: u16,
    pub Reserved: u16,
    pub ProcessorFeatureBits: u32,
}

pub type PSYSTEM_PROCESSOR_INFORMATION = *mut _SYSTEM_PROCESSOR_INFORMATION;

#[repr(C)]
pub struct _SYSTEM_LOGICAL_PROCESSOR_INFORMATION {
    pub ProcessorMask: usize,
    pub Relationship: u32,
    pub Reserved: [u8; 2],
    pub ProcessorCoreCount: u8,
    pub ProcessorCacheCount: u8,
    pub ProcessorCacheType: u32,
    pub ProcessorCacheLevel: u8,
    pub ProcessorCacheAssociativity: u8,
    pub ProcessorCacheLineSize: u16,
    pub ProcessorCacheSize: u32,
    pub Reserved2: [u8; 4],
}

pub type PSYSTEM_LOGICAL_PROCESSOR_INFORMATION = *mut _SYSTEM_LOGICAL_PROCESSOR_INFORMATION;

#[repr(C)]
pub struct _SYSTEM_NUMA_INFORMATION {
    pub HighestNodeNumber: u32,
    pub AvailableMemory: [u64; 64],
}

#[repr(C)]
pub struct _SYSTEM_REGISTRY_QUOTA_INFORMATION {
    pub RegistryQuotaAllowed: u32,
    pub RegistryQuotaUsed: u32,
    pub Reserved: [u32; 3],
}

#[repr(C)]
pub struct _SYSTEM_FIRMWARE_TABLE_INFORMATION {
    pub ProviderSignature: u32,
    pub Action: u32,
    pub TableID: u32,
    pub TableBufferLength: u32,
    pub TableBuffer: *mut u8,
}

pub type PDWORD = *mut u32;
pub type PULONGLONG = *mut u64;
pub type LPCSTR = *const u8;
pub type LPCWSTR = *const u16;
pub type PSIZE_T = *mut usize;

#[no_mangle]
pub unsafe extern "system" fn GetSystemInfoInternal(
    BasicInfo: PSYSTEM_BASIC_INFORMATION,
    ProcInfo: PSYSTEM_PROCESSOR_INFORMATION,
    SystemInfo: LPSYSTEM_INFO,
) {
    RtlZeroMemory(SystemInfo as *mut _, core::mem::size_of::<SYSTEM_INFO>() as SIZE_T);
    (*SystemInfo).wProcessorArchitecture = (*ProcInfo).ProcessorArchitecture;
    (*SystemInfo).wReserved = 0;
    (*SystemInfo).dwPageSize = (*BasicInfo).PageSize;
    (*SystemInfo).lpMinimumApplicationAddress = (*BasicInfo).MinimumUserModeAddress as *mut _;
    (*SystemInfo).lpMaximumApplicationAddress = (*BasicInfo).MaximumUserModeAddress as *mut _;
    (*SystemInfo).dwActiveProcessorMask = (*BasicInfo).ActiveProcessorsAffinityMask as usize;
    (*SystemInfo).dwNumberOfProcessors = (*BasicInfo).NumberOfProcessors as u32;
    (*SystemInfo).wProcessorLevel = (*ProcInfo).ProcessorLevel;
    (*SystemInfo).wProcessorRevision = (*ProcInfo).ProcessorRevision;
    (*SystemInfo).dwAllocationGranularity = (*BasicInfo).AllocationGranularity;

    match (*ProcInfo).ProcessorArchitecture {
        PROCESSOR_ARCHITECTURE_INTEL => {
            match (*ProcInfo).ProcessorLevel {
                3 => {
                    (*SystemInfo).dwProcessorType = PROCESSOR_INTEL_386;
                }
                4 => {
                    (*SystemInfo).dwProcessorType = PROCESSOR_INTEL_486;
                }
                _ => {
                    (*SystemInfo).dwProcessorType = PROCESSOR_INTEL_PENTIUM;
                }
            }
        }
        PROCESSOR_ARCHITECTURE_AMD64 => {
            (*SystemInfo).dwProcessorType = PROCESSOR_AMD_X8664;
        }
        PROCESSOR_ARCHITECTURE_IA64 => {
            (*SystemInfo).dwProcessorType = PROCESSOR_INTEL_IA64;
        }
        _ => {
            (*SystemInfo).dwProcessorType = 0;
        }
    }

    if PV_NT351 > GetProcessVersion(0) {
        (*SystemInfo).wProcessorLevel = 0;
        (*SystemInfo).wProcessorRevision = 0;
    }
}
unsafe fn BaseQuerySystemFirmware(
    FirmwareTableProviderSignature: DWORD,
    FirmwareTableID: DWORD,
    pFirmwareTableBuffer: PVOID,
    BufferSize: DWORD,
    Action: SYSTEM_FIRMWARE_TABLE_ACTION,
) -> UINT {
    let SysFirmwareInfo: *mut SYSTEM_FIRMWARE_TABLE_INFORMATION;
    let mut Result: ULONG = 0;
    let mut ReturnedSize: ULONG = 0;
    let TotalSize: ULONG = BufferSize + core::mem::size_of::<SYSTEM_FIRMWARE_TABLE_INFORMATION>() as ULONG;
    let Status: NTSTATUS;

    SysFirmwareInfo = RtlAllocateHeap(RtlGetProcessHeap(), HEAP_ZERO_MEMORY, TotalSize as SIZE_T) as *mut SYSTEM_FIRMWARE_TABLE_INFORMATION;
    if SysFirmwareInfo.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    // _SEH2_TRY
    {
        (*SysFirmwareInfo).ProviderSignature = FirmwareTableProviderSignature;
        (*SysFirmwareInfo).TableID = FirmwareTableID;
        (*SysFirmwareInfo).Action = Action;
        (*SysFirmwareInfo).TableBufferLength = BufferSize;

        Status = NtQuerySystemInformation(
            SystemFirmwareTableInformation,
            SysFirmwareInfo as *mut core::ffi::c_void,
            TotalSize,
            &mut ReturnedSize as *mut ULONG,
        );

        if NT_SUCCESS(Status) || Status == STATUS_BUFFER_TOO_SMALL {
            Result = (*SysFirmwareInfo).TableBufferLength;
        }

        if NT_SUCCESS(Status) && !pFirmwareTableBuffer.is_null() {
            RtlCopyMemory(
                pFirmwareTableBuffer,
                (*SysFirmwareInfo).TableBuffer.as_ptr() as *const core::ffi::c_void,
                (*SysFirmwareInfo).TableBufferLength as SIZE_T,
            );
        }
    }

    // _SEH2_FINALLY
    {
        RtlFreeHeap(RtlGetProcessHeap(), 0, SysFirmwareInfo as *mut core::ffi::c_void);
    }
    // _SEH2_END

    BaseSetLastNTError(Status);
    return Result;
}

#[no_mangle]
pub unsafe extern "system" fn GetLargePageMinimum() -> SIZE_T {
    return (*SharedUserData).LargePageMinimum as SIZE_T;
}

#[no_mangle]
pub unsafe extern "system" fn GetSystemInfo(
    lpSystemInfo: LPSYSTEM_INFO,
) {
    let mut BasicInfo: SYSTEM_BASIC_INFORMATION = core::mem::zeroed();
    let mut ProcInfo: SYSTEM_PROCESSOR_INFORMATION = core::mem::zeroed();
    let Status: NTSTATUS;

    let Status = NtQuerySystemInformation(
        SystemBasicInformation,
        &mut BasicInfo as *mut SYSTEM_BASIC_INFORMATION as *mut core::ffi::c_void,
        core::mem::size_of::<SYSTEM_BASIC_INFORMATION>() as u32,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        return;
    }

    let Status = NtQuerySystemInformation(
        SystemProcessorInformation,
        &mut ProcInfo as *mut SYSTEM_PROCESSOR_INFORMATION as *mut core::ffi::c_void,
        core::mem::size_of::<SYSTEM_PROCESSOR_INFORMATION>() as u32,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        return;
    }

    GetSystemInfoInternal(
        &mut BasicInfo as *mut _ as PSYSTEM_BASIC_INFORMATION,
        &mut ProcInfo as *mut _ as PSYSTEM_PROCESSOR_INFORMATION,
        lpSystemInfo,
    );
}

#[no_mangle]
pub unsafe extern "system" fn IsProcessorFeaturePresent(
    ProcessorFeature: DWORD,
) -> BOOL {
    if ProcessorFeature >= PROCESSOR_FEATURE_MAX {
        return 0i32;
    }
    return (*SharedUserData).ProcessorFeatures[ProcessorFeature as usize] as i32;
}

#[no_mangle]
pub unsafe extern "system" fn GetSystemRegistryQuota(
    pdwQuotaAllowed: PDWORD,
    pdwQuotaUsed: PDWORD,
) -> BOOL {
    let mut QuotaInfo: SYSTEM_REGISTRY_QUOTA_INFORMATION = core::mem::zeroed();
    let mut BytesWritten: ULONG = 0;
    let Status: NTSTATUS;

    Status = NtQuerySystemInformation(
        SystemRegistryQuotaInformation,
        &mut QuotaInfo as *mut SYSTEM_REGISTRY_QUOTA_INFORMATION as *mut core::ffi::c_void,
        core::mem::size_of::<SYSTEM_REGISTRY_QUOTA_INFORMATION>() as ULONG,
        &mut BytesWritten,
    );
    if NT_SUCCESS(Status) {
        if !pdwQuotaAllowed.is_null() {
            *pdwQuotaAllowed = QuotaInfo.RegistryQuotaAllowed;
        }
        if !pdwQuotaUsed.is_null() {
            *pdwQuotaUsed = QuotaInfo.RegistryQuotaUsed;
        }
        return 1;
    }

    BaseSetLastNTError(Status);
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn GetNativeSystemInfo(
    lpSystemInfo: LPSYSTEM_INFO,
) {
    let mut BasicInfo: SYSTEM_BASIC_INFORMATION = core::mem::zeroed();
    let mut ProcInfo: SYSTEM_PROCESSOR_INFORMATION = core::mem::zeroed();
    let mut Status: NTSTATUS;

    Status = RtlGetNativeSystemInformation(
        SystemBasicInformation,
        &mut BasicInfo as *mut SYSTEM_BASIC_INFORMATION as PVOID,
        core::mem::size_of::<SYSTEM_BASIC_INFORMATION>() as ULONG,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        return;
    }

    Status = RtlGetNativeSystemInformation(
        SystemProcessorInformation,
        &mut ProcInfo as *mut SYSTEM_PROCESSOR_INFORMATION as PVOID,
        core::mem::size_of::<SYSTEM_PROCESSOR_INFORMATION>() as ULONG,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        return;
    }

    GetSystemInfoInternal(
        &mut BasicInfo as *mut _ as PSYSTEM_BASIC_INFORMATION,
        &mut ProcInfo as *mut _ as PSYSTEM_PROCESSOR_INFORMATION,
        lpSystemInfo,
    );
}

#[no_mangle]
pub unsafe extern "system" fn GetLogicalProcessorInformation(
    Buffer: PSYSTEM_LOGICAL_PROCESSOR_INFORMATION,
    ReturnLength: PDWORD,
) -> BOOL {
    let mut Status: NTSTATUS;

    if ReturnLength.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    Status = NtQuerySystemInformation(
        SystemLogicalProcessorInformation,
        Buffer as *mut core::ffi::c_void,
        *ReturnLength,
        ReturnLength,
    );

    /* Normalize the error to what Win32 expects */
    if Status == STATUS_INFO_LENGTH_MISMATCH {
        Status = STATUS_BUFFER_TOO_SMALL;
    }
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return 0;
    }

    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn GetNumaHighestNodeNumber(
    HighestNodeNumber: PULONG,
) -> BOOL {
    let Status: NTSTATUS;
    let mut Length: ULONG = 0;
    let mut PartialInfo: [ULONG; 2] = [0; 2];

    Status = NtQuerySystemInformation(
        SystemNumaProcessorMap,
        PartialInfo.as_mut_ptr() as *mut core::ffi::c_void,
        core::mem::size_of::<[ULONG; 2]>() as ULONG,
        &mut Length,
    );
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return 0;
    }

    if Length < core::mem::size_of::<ULONG>() as ULONG {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    *HighestNodeNumber = PartialInfo[0];
    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn GetNumaNodeProcessorMask(
    Node: UCHAR,
    ProcessorMask: PULONGLONG,
) -> BOOL {
    let mut Status: NTSTATUS;
    let mut NumaInformation: SYSTEM_NUMA_INFORMATION = core::mem::zeroed();
    let mut Length: ULONG = 0;

    Status = NtQuerySystemInformation(
        SystemNumaProcessorMap,
        &mut NumaInformation as *mut SYSTEM_NUMA_INFORMATION as *mut core::ffi::c_void,
        core::mem::size_of::<SYSTEM_NUMA_INFORMATION>() as ULONG,
        &mut Length as *mut ULONG,
    );
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return 0;
    }

    if Node as ULONG > NumaInformation.HighestNodeNumber {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    *ProcessorMask = NumaInformation.__bindgen_anon_1.ActiveProcessorsAffinityMask[Node as usize];
    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn GetNumaProcessorNode(
    Processor: UCHAR,
    NodeNumber: PUCHAR,
) -> BOOL {
    let Status: NTSTATUS;
    let mut NumaInformation: SYSTEM_NUMA_INFORMATION = core::mem::zeroed();
    let Length: ULONG = 0;
    let mut Node: ULONG;
    let Proc: ULONGLONG;

    /* Can't handle processor number >= 32 */
    if Processor >= MAXIMUM_PROCESSORS as UCHAR {
        *NodeNumber = !0u8;
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    /* Query NUMA information */
    Status = NtQuerySystemInformation(
        SystemNumaProcessorMap,
        &mut NumaInformation as *mut SYSTEM_NUMA_INFORMATION as *mut core::ffi::c_void,
        core::mem::size_of::<SYSTEM_NUMA_INFORMATION>() as ULONG,
        &Length as *const ULONG as *mut ULONG,
    );
    if !NT_SUCCESS(Status) {
        *NodeNumber = !0u8;
        BaseSetLastNTError(Status);
        return 0;
    }

    /* Find ourselves */
    Node = 0;
    Proc = (1u64 as ULONGLONG) << Processor;
    while (Proc & NumaInformation.__bindgen_anon_1.ActiveProcessorsAffinityMask[Node as usize]) == 0u64 {
        Node += 1;
        /* Out of options */
        if Node as ULONG > NumaInformation.HighestNodeNumber {
            *NodeNumber = !0u8;
            SetLastError(ERROR_INVALID_PARAMETER);
            return 0;
        }
    }

    /* Return found node */
    *NodeNumber = Node as UCHAR;
    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn GetNumaAvailableMemoryNode(
    Node: UCHAR,
    AvailableBytes: PULONGLONG,
) -> BOOL {
    let mut Status: NTSTATUS;
    let mut NumaInformation: SYSTEM_NUMA_INFORMATION = core::mem::zeroed();
    let mut Length: ULONG = 0;

    Status = NtQuerySystemInformation(
        SystemNumaAvailableMemory,
        &mut NumaInformation as *mut SYSTEM_NUMA_INFORMATION as *mut core::ffi::c_void,
        core::mem::size_of::<SYSTEM_NUMA_INFORMATION>() as ULONG,
        &mut Length as *mut ULONG,
    );
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return 0;
    }

    if Node as ULONG > NumaInformation.HighestNodeNumber {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    *AvailableBytes = NumaInformation.__bindgen_anon_1.AvailableMemory[Node as usize];
    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn GetFirmwareEnvironmentVariableW(
    lpName: LPCWSTR,
    lpGuid: LPCWSTR,
    pBuffer: PVOID,
    nSize: DWORD,
) -> DWORD {
    GetFirmwareEnvironmentVariableExW(lpName, lpGuid, pBuffer, nSize, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "system" fn GetFirmwareEnvironmentVariableA(
    lpName: LPCSTR,
    lpGuid: LPCSTR,
    pBuffer: PVOID,
    nSize: DWORD,
) -> DWORD {
    return GetFirmwareEnvironmentVariableExA(lpName, lpGuid, pBuffer, nSize, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn SetFirmwareEnvironmentVariableW(
    lpName: LPCWSTR,
    lpGuid: LPCWSTR,
    pValue: PVOID,
    nSize: DWORD,
) -> BOOL {
    return SetFirmwareEnvironmentVariableExW(
        lpName,
        lpGuid,
        pValue,
        nSize,
        VARIABLE_ATTRIBUTE_NON_VOLATILE,
    );
}

#[no_mangle]
pub unsafe extern "system" fn SetFirmwareEnvironmentVariableA(
    lpName: LPCSTR,
    lpGuid: LPCSTR,
    pValue: PVOID,
    nSize: DWORD,
) -> BOOL {
    SetFirmwareEnvironmentVariableExA(
        lpName,
        lpGuid,
        pValue,
        nSize,
        VARIABLE_ATTRIBUTE_NON_VOLATILE,
    )
}

#[no_mangle]
pub unsafe extern "system" fn EnumSystemFirmwareTables(
    FirmwareTableProviderSignature: DWORD,
    pFirmwareTableEnumBuffer: PVOID,
    BufferSize: DWORD,
) -> UINT {
    return BaseQuerySystemFirmware(
        FirmwareTableProviderSignature,
        0,
        pFirmwareTableEnumBuffer,
        BufferSize,
        SystemFirmwareTable_Enumerate,
    );
}

#[no_mangle]
pub unsafe extern "system" fn GetSystemFirmwareTable(
    FirmwareTableProviderSignature: DWORD,
    FirmwareTableID: DWORD,
    pFirmwareTableBuffer: PVOID,
    BufferSize: DWORD,
) -> UINT {
    return BaseQuerySystemFirmware(
        FirmwareTableProviderSignature,
        FirmwareTableID,
        pFirmwareTableBuffer,
        BufferSize,
        SystemFirmwareTable_Get,
    );
}

#[no_mangle]
pub unsafe extern "system" fn GetSystemFileCacheSize(
    lpMinimumFileCacheSize: PSIZE_T,
    lpMaximumFileCacheSize: PSIZE_T,
    lpFlags: PDWORD,
) -> BOOL {
    STUB!();
    return 0i32;
}

#[no_mangle]
pub unsafe extern "system" fn SetSystemFileCacheSize(
    MinimumFileCacheSize: SIZE_T,
    MaximumFileCacheSize: SIZE_T,
    Flags: DWORD,
) -> BOOL {
    STUB!();
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn GetCurrentPackageId(
    BufferLength: *mut UINT32,
    Buffer: *mut BYTE,
) -> LONG {
    STUB!();
    return APPMODEL_ERROR_NO_PACKAGE as LONG;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Win32 Base API
//  * LICENSE:         GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
//  * PURPOSE:         System Information Functions
//  * COPYRIGHT:       Emanuele Aliberti
//  *                  Christoph von Wittich
//  *                  Thomas Weidenmueller
//  *                  Gunnar Andre Dalsnes
//  *                  Stanislav Motylkov (x86corez@gmail.com)
//  *                  Mark Jansen (mark.jansen@reactos.org)
//  *                  Copyright 2023 Ratin Gao <ratin@knsoft.org>
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include <k32.h>
//
// #define NDEBUG
// #include <debug.h>
//
// #define PV_NT351 0x00030033
//
// /* PRIVATE FUNCTIONS **********************************************************/
//
// VOID
// WINAPI
// GetSystemInfoInternal(IN PSYSTEM_BASIC_INFORMATION BasicInfo,
//                       IN PSYSTEM_PROCESSOR_INFORMATION ProcInfo,
//                       OUT LPSYSTEM_INFO SystemInfo)
// {
//     RtlZeroMemory(SystemInfo, sizeof (SYSTEM_INFO));
//     SystemInfo->wProcessorArchitecture = ProcInfo->ProcessorArchitecture;
//     SystemInfo->wReserved = 0;
//     SystemInfo->dwPageSize = BasicInfo->PageSize;
//     SystemInfo->lpMinimumApplicationAddress = (PVOID)BasicInfo->MinimumUserModeAddress;
//     SystemInfo->lpMaximumApplicationAddress = (PVOID)BasicInfo->MaximumUserModeAddress;
//     SystemInfo->dwActiveProcessorMask = BasicInfo->ActiveProcessorsAffinityMask;
//     SystemInfo->dwNumberOfProcessors = BasicInfo->NumberOfProcessors;
//     SystemInfo->wProcessorLevel = ProcInfo->ProcessorLevel;
//     SystemInfo->wProcessorRevision = ProcInfo->ProcessorRevision;
//     SystemInfo->dwAllocationGranularity = BasicInfo->AllocationGranularity;
//
//     switch (ProcInfo->ProcessorArchitecture)
//     {
//         case PROCESSOR_ARCHITECTURE_INTEL:
//             switch (ProcInfo->ProcessorLevel)
//             {
//                 case 3:
//                     SystemInfo->dwProcessorType = PROCESSOR_INTEL_386;
//                     break;
//                 case 4:
//                     SystemInfo->dwProcessorType = PROCESSOR_INTEL_486;
//                     break;
//                 default:
//                     SystemInfo->dwProcessorType = PROCESSOR_INTEL_PENTIUM;
//             }
//             break;
//
//         case PROCESSOR_ARCHITECTURE_AMD64:
//             SystemInfo->dwProcessorType = PROCESSOR_AMD_X8664;
//             break;
//
//         case PROCESSOR_ARCHITECTURE_IA64:
//             SystemInfo->dwProcessorType = PROCESSOR_INTEL_IA64;
//             break;
//
//         default:
//             SystemInfo->dwProcessorType = 0;
//             break;
//     }
//
//     if (PV_NT351 > GetProcessVersion(0))
//     {
//         SystemInfo->wProcessorLevel = 0;
//         SystemInfo->wProcessorRevision = 0;
//     }
// }
//
// static
// UINT
// BaseQuerySystemFirmware(
//      DWORD FirmwareTableProviderSignature,
//      DWORD FirmwareTableID,
//      PVOID pFirmwareTableBuffer,
//      DWORD BufferSize,
//      SYSTEM_FIRMWARE_TABLE_ACTION Action)
// {
//     SYSTEM_FIRMWARE_TABLE_INFORMATION* SysFirmwareInfo;
//     ULONG Result = 0, ReturnedSize;
//     ULONG TotalSize = BufferSize + sizeof(SYSTEM_FIRMWARE_TABLE_INFORMATION);
//     NTSTATUS Status;
//
//     SysFirmwareInfo = RtlAllocateHeap(RtlGetProcessHeap(), HEAP_ZERO_MEMORY, TotalSize);
//     if (!SysFirmwareInfo)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return 0;
//     }
//     _SEH2_TRY
//     {
//         SysFirmwareInfo->ProviderSignature = FirmwareTableProviderSignature;
//         SysFirmwareInfo->TableID = FirmwareTableID;
//         SysFirmwareInfo->Action = Action;
//         SysFirmwareInfo->TableBufferLength = BufferSize;
//
//         Status = NtQuerySystemInformation(SystemFirmwareTableInformation, SysFirmwareInfo, TotalSize, &ReturnedSize);
//
//         if (NT_SUCCESS(Status) || Status == STATUS_BUFFER_TOO_SMALL)
//             Result = SysFirmwareInfo->TableBufferLength;
//
//         if (NT_SUCCESS(Status) && pFirmwareTableBuffer)
//         {
//             RtlCopyMemory(pFirmwareTableBuffer, SysFirmwareInfo->TableBuffer, SysFirmwareInfo->TableBufferLength);
//         }
//     }
//     _SEH2_FINALLY
//     {
//         RtlFreeHeap(RtlGetProcessHeap(), 0, SysFirmwareInfo);
//     }
//     _SEH2_END;
//
//     BaseSetLastNTError(Status);
//     return Result;
// }
//
// /* PUBLIC FUNCTIONS ***********************************************************/
//
// /*
//  * @implemented
//  */
// SIZE_T
// WINAPI
// GetLargePageMinimum(VOID)
// {
//     return SharedUserData->LargePageMinimum;
// }
//
// /*
//  * @implemented
//  */
// VOID
// WINAPI
// GetSystemInfo(IN LPSYSTEM_INFO lpSystemInfo)
// {
//     SYSTEM_BASIC_INFORMATION BasicInfo;
//     SYSTEM_PROCESSOR_INFORMATION ProcInfo;
//     NTSTATUS Status;
//
//     Status = NtQuerySystemInformation(SystemBasicInformation,
//                                       &BasicInfo,
//                                       sizeof(BasicInfo),
//                                       0);
//     if (!NT_SUCCESS(Status)) return;
//
//     Status = NtQuerySystemInformation(SystemProcessorInformation,
//                                       &ProcInfo,
//                                       sizeof(ProcInfo),
//                                       0);
//     if (!NT_SUCCESS(Status)) return;
//
//     GetSystemInfoInternal(&BasicInfo, &ProcInfo, lpSystemInfo);
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// IsProcessorFeaturePresent(IN DWORD ProcessorFeature)
// {
//     if (ProcessorFeature >= PROCESSOR_FEATURE_MAX) return FALSE;
//     return ((BOOL)SharedUserData->ProcessorFeatures[ProcessorFeature]);
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// GetSystemRegistryQuota(OUT PDWORD pdwQuotaAllowed,
//                        OUT PDWORD pdwQuotaUsed)
// {
//     SYSTEM_REGISTRY_QUOTA_INFORMATION QuotaInfo;
//     ULONG BytesWritten;
//     NTSTATUS Status;
//
//     Status = NtQuerySystemInformation(SystemRegistryQuotaInformation,
//                                       &QuotaInfo,
//                                       sizeof(QuotaInfo),
//                                       &BytesWritten);
//     if (NT_SUCCESS(Status))
//     {
//       if (pdwQuotaAllowed) *pdwQuotaAllowed = QuotaInfo.RegistryQuotaAllowed;
//       if (pdwQuotaUsed) *pdwQuotaUsed = QuotaInfo.RegistryQuotaUsed;
//       return TRUE;
//     }
//
//     BaseSetLastNTError(Status);
//     return FALSE;
// }
//
// /*
//  * @implemented
//  */
// VOID
// WINAPI
// GetNativeSystemInfo(IN LPSYSTEM_INFO lpSystemInfo)
// {
//     SYSTEM_BASIC_INFORMATION BasicInfo;
//     SYSTEM_PROCESSOR_INFORMATION ProcInfo;
//     NTSTATUS Status;
//
//     Status = RtlGetNativeSystemInformation(SystemBasicInformation,
//                                            &BasicInfo,
//                                            sizeof(BasicInfo),
//                                            0);
//     if (!NT_SUCCESS(Status)) return;
//
//     Status = RtlGetNativeSystemInformation(SystemProcessorInformation,
//                                            &ProcInfo,
//                                            sizeof(ProcInfo),
//                                            0);
//     if (!NT_SUCCESS(Status)) return;
//
//     GetSystemInfoInternal(&BasicInfo, &ProcInfo, lpSystemInfo);
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// GetLogicalProcessorInformation(OUT PSYSTEM_LOGICAL_PROCESSOR_INFORMATION Buffer,
//                                IN OUT PDWORD ReturnLength)
// {
//     NTSTATUS Status;
//
//     if (!ReturnLength)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     Status = NtQuerySystemInformation(SystemLogicalProcessorInformation,
//                                       Buffer,
//                                       *ReturnLength,
//                                       ReturnLength);
//
//     /* Normalize the error to what Win32 expects */
//     if (Status == STATUS_INFO_LENGTH_MISMATCH) Status = STATUS_BUFFER_TOO_SMALL;
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
// GetNumaHighestNodeNumber(OUT PULONG HighestNodeNumber)
// {
//     NTSTATUS Status;
//     ULONG Length;
//     ULONG PartialInfo[2]; // First two members of SYSTEM_NUMA_INFORMATION
//
//     /* Query partial NUMA info */
//     Status = NtQuerySystemInformation(SystemNumaProcessorMap,
//                                       PartialInfo,
//                                       sizeof(PartialInfo),
//                                       &Length);
//     if (!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     if (Length < sizeof(ULONG))
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     /* First member of the struct is the highest node number */
//     *HighestNodeNumber = PartialInfo[0];
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// GetNumaNodeProcessorMask(IN UCHAR Node,
//                          OUT PULONGLONG ProcessorMask)
// {
//     NTSTATUS Status;
//     SYSTEM_NUMA_INFORMATION NumaInformation;
//     ULONG Length;
//
//     /* Query NUMA information */
//     Status = NtQuerySystemInformation(SystemNumaProcessorMap,
//                                       &NumaInformation,
//                                       sizeof(NumaInformation),
//                                       &Length);
//     if (!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     /* Validate input node number */
//     if (Node > NumaInformation.HighestNodeNumber)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     /* Return mask for that node */
//     *ProcessorMask = NumaInformation.ActiveProcessorsAffinityMask[Node];
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// GetNumaProcessorNode(IN UCHAR Processor,
//                      OUT PUCHAR NodeNumber)
// {
//     NTSTATUS Status;
//     SYSTEM_NUMA_INFORMATION NumaInformation;
//     ULONG Length;
//     ULONG Node;
//     ULONGLONG Proc;
//
//     /* Can't handle processor number >= 32 */
//     if (Processor >= MAXIMUM_PROCESSORS)
//     {
//         *NodeNumber = -1;
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     /* Query NUMA information */
//     Status = NtQuerySystemInformation(SystemNumaProcessorMap,
//                                       &NumaInformation,
//                                       sizeof(NumaInformation),
//                                       &Length);
//     if (!NT_SUCCESS(Status))
//     {
//         *NodeNumber = -1;
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     /* Find ourselves */
//     Node = 0;
//     Proc = 1ULL << Processor;
//     while ((Proc & NumaInformation.ActiveProcessorsAffinityMask[Node]) == 0ULL)
//     {
//         ++Node;
//         /* Out of options */
//         if (Node > NumaInformation.HighestNodeNumber)
//         {
//             *NodeNumber = -1;
//             SetLastError(ERROR_INVALID_PARAMETER);
//             return FALSE;
//         }
//     }
//
//     /* Return found node */
//     *NodeNumber = Node;
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// GetNumaAvailableMemoryNode(IN UCHAR Node,
//                            OUT PULONGLONG AvailableBytes)
// {
//     NTSTATUS Status;
//     SYSTEM_NUMA_INFORMATION NumaInformation;
//     ULONG Length;
//
//     /* Query NUMA information */
//     Status = NtQuerySystemInformation(SystemNumaAvailableMemory,
//                                       &NumaInformation,
//                                       sizeof(NumaInformation),
//                                       &Length);
//     if (!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     /* Validate input node number */
//     if (Node > NumaInformation.HighestNodeNumber)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     /* Return available memory for that node */
//     *AvailableBytes = NumaInformation.AvailableMemory[Node];
//     return TRUE;
// }
//
//
// DWORD
// WINAPI
// GetFirmwareEnvironmentVariableExW(
//      LPCWSTR lpName,
//      LPCWSTR lpGuid,
//      PVOID pBuffer,
//      DWORD nSize,
//      PDWORD pdwAttribubutes);
//
//
// DWORD
// WINAPI
// GetFirmwareEnvironmentVariableExA(
//      LPCSTR lpName,
//      LPCSTR lpGuid,
//      PVOID pBuffer,
//      DWORD nSize,
//      PDWORD pdwAttribubutes);
//
// BOOL
// WINAPI
// SetFirmwareEnvironmentVariableExW(
//      LPCWSTR lpName,
//      LPCWSTR lpGuid,
//      PVOID pValue,
//      DWORD nSize,
//      DWORD dwAttributes);
//
// BOOL
// WINAPI
// SetFirmwareEnvironmentVariableExA(
//      LPCSTR lpName,
//      LPCSTR lpGuid,
//      PVOID pValue,
//      DWORD nSize,
//      DWORD dwAttributes);
//
//
// DWORD
// WINAPI
// GetFirmwareEnvironmentVariableW(
//      LPCWSTR lpName,
//      LPCWSTR lpGuid,
//      PVOID pBuffer,
//      DWORD nSize)
// {
//     return GetFirmwareEnvironmentVariableExW(lpName, lpGuid, pBuffer, nSize, NULL);
// }
//
//
// DWORD
// WINAPI
// GetFirmwareEnvironmentVariableA(
//      LPCSTR lpName,
//      LPCSTR lpGuid,
//      PVOID pBuffer,
//      DWORD nSize)
// {
//     return GetFirmwareEnvironmentVariableExA(lpName, lpGuid, pBuffer, nSize, NULL);
// }
//
// BOOL
// WINAPI
// SetFirmwareEnvironmentVariableW(
//      LPCWSTR lpName,
//      LPCWSTR lpGuid,
//      PVOID pValue,
//      DWORD nSize)
// {
//     return SetFirmwareEnvironmentVariableExW(lpName,
//                                              lpGuid,
//                                              pValue,
//                                              nSize,
//                                              VARIABLE_ATTRIBUTE_NON_VOLATILE);
// }
//
// BOOL
// WINAPI
// SetFirmwareEnvironmentVariableA(
//      LPCSTR lpName,
//      LPCSTR lpGuid,
//      PVOID pValue,
//      DWORD nSize)
// {
//     return SetFirmwareEnvironmentVariableExA(lpName,
//                                              lpGuid,
//                                              pValue,
//                                              nSize,
//                                              VARIABLE_ATTRIBUTE_NON_VOLATILE);
// }
//
// /**
//  * @name EnumSystemFirmwareTables
//  * @implemented
//  *
//  * Obtains firmware table identifiers.
//  * https://learn.microsoft.com/en-us/windows/win32/api/sysinfoapi/nf-sysinfoapi-enumsystemfirmwaretables
//  *
//  * @param FirmwareTableProviderSignature
//  * Can be either ACPI, FIRM, or RSMB.
//  *
//  * @param pFirmwareTableBuffer
//  * Pointer to the output buffer, can be NULL.
//  *
//  * @param BufferSize
//  * Size of the output buffer.
//  *
//  * @return
//  * Actual size of the data in case of success, 0 otherwise.
//  *
//  * @remarks
//  * Data would be written to buffer only if the specified size is
//  * larger or equal to the actual size, in the other case Last Error
//  * value would be set to ERROR_INSUFFICIENT_BUFFER.
//  * In case of incorrect provider signature, Last Error value would be
//  * set to ERROR_INVALID_FUNCTION.
//  *
//  */
// UINT
// WINAPI
// EnumSystemFirmwareTables(
//      DWORD FirmwareTableProviderSignature,
//      PVOID pFirmwareTableEnumBuffer,
//      DWORD BufferSize)
// {
//     return BaseQuerySystemFirmware(FirmwareTableProviderSignature,
//                                    0,
//                                    pFirmwareTableEnumBuffer,
//                                    BufferSize,
//                                    SystemFirmwareTable_Enumerate);
// }
//
// /**
//  * @name GetSystemFirmwareTable
//  * @implemented
//  *
//  * Obtains the firmware table data.
//  * https://learn.microsoft.com/en-us/windows/win32/api/sysinfoapi/nf-sysinfoapi-getsystemfirmwaretable
//  *
//  * @param FirmwareTableProviderSignature
//  * Can be either ACPI, FIRM, or RSMB.
//  *
//  * @param FirmwareTableID
//  * Correct table identifier.
//  *
//  * @param pFirmwareTableBuffer
//  * Pointer to the output buffer, can be NULL.
//  *
//  * @param BufferSize
//  * Size of the output buffer.
//  *
//  * @return
//  * Actual size of the data in case of success, 0 otherwise.
//  *
//  * @remarks
//  * Data would be written to buffer only if the specified size is
//  * larger or equal to the actual size, in the other case Last Error
//  * value would be set to ERROR_INSUFFICIENT_BUFFER.
//  * In case of incorrect provider signature, Last Error value would be
//  * set to ERROR_INVALID_FUNCTION.
//  * Also Last Error value becomes ERROR_NOT_FOUND if incorrect
//  * table identifier was specified along with ACPI provider, and
//  * ERROR_INVALID_PARAMETER along with FIRM provider. The RSMB provider
//  * accepts any table identifier.
//  *
//  */
// UINT
// WINAPI
// GetSystemFirmwareTable(
//      DWORD FirmwareTableProviderSignature,
//      DWORD FirmwareTableID,
//      PVOID pFirmwareTableBuffer,
//      DWORD BufferSize)
// {
//     return BaseQuerySystemFirmware(FirmwareTableProviderSignature,
//                                    FirmwareTableID,
//                                    pFirmwareTableBuffer,
//                                    BufferSize,
//                                    SystemFirmwareTable_Get);
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// WINAPI
// GetSystemFileCacheSize(OUT PSIZE_T lpMinimumFileCacheSize,
//                        OUT PSIZE_T lpMaximumFileCacheSize,
//                        OUT PDWORD lpFlags)
// {
//     STUB;
//     return FALSE;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// WINAPI
// SetSystemFileCacheSize(IN SIZE_T MinimumFileCacheSize,
//                        IN SIZE_T MaximumFileCacheSize,
//                        IN DWORD Flags)
// {
//     STUB;
//     return FALSE;
// }
//
// /*
//  * @unimplemented
//  */
// LONG
// WINAPI
// GetCurrentPackageId(UINT32 *BufferLength,
//                     BYTE *Buffer)
// {
//     STUB;
//     return APPMODEL_ERROR_NO_PACKAGE;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
