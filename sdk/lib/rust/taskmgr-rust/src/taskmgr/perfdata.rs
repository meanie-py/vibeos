// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_59A543594151: [u16; 4] = [0x002Eu16, 0x002E, 0x002E, 0x0000];
pub static RUSTCONV_WSTR_839B5A81141C: [u16; 5] = [0x0020u16, 0x002A, 0x0033, 0x0032, 0x0000];
pub static RUSTCONV_WSTR_96A296D224F2: [u16; 1] = [0x0000u16];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_3 {
    pub LowPart: DWORD,
    pub HighPart: LONG,
}

#[no_mangle]
pub static mut PerfDataCriticalSection: CRITICAL_SECTION = _RTL_CRITICAL_SECTION {
    DebugInfo: ::core::ptr::null::<_RTL_CRITICAL_SECTION_DEBUG>()
        as *mut _RTL_CRITICAL_SECTION_DEBUG,
    LockCount: 0,
    RecursionCount: 0,
    OwningThread: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    LockSemaphore: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    SpinCount: 0,
};

#[no_mangle]
pub static mut pPerfDataOld: PPERFDATA = ::core::ptr::null::<_PERFDATA>() as *mut _PERFDATA;

#[no_mangle]
pub static mut pPerfData: PPERFDATA = ::core::ptr::null::<_PERFDATA>() as *mut _PERFDATA;

#[no_mangle]
pub static mut ProcessCountOld: ULONG = 0 as ULONG;

#[no_mangle]
pub static mut ProcessCount: ULONG = 0 as ULONG;

#[no_mangle]
pub static mut dbIdleTime: ::core::ffi::c_double = 0.;

#[no_mangle]
pub static mut dbKernelTime: ::core::ffi::c_double = 0.;

#[no_mangle]
pub static mut dbSystemTime: ::core::ffi::c_double = 0.;

#[no_mangle]
pub static mut liOldIdleTime: LARGE_INTEGER = _LARGE_INTEGER {
    __bindgen_anon_1: _LARGE_INTEGER__bindgen_ty_1 { LowPart: 0 as DWORD,
        HighPart: 0 as LONG,
    },
};

#[no_mangle]
pub static mut OldKernelTime: ::core::ffi::c_double =
    0 as ::core::ffi::c_int as ::core::ffi::c_double;

#[no_mangle]
pub static mut liOldSystemTime: LARGE_INTEGER = _LARGE_INTEGER {
    __bindgen_anon_1: _LARGE_INTEGER__bindgen_ty_1 { LowPart: 0 as DWORD,
        HighPart: 0 as LONG,
    },
};

#[no_mangle]
pub static mut SystemPerfInfo: SYSTEM_PERFORMANCE_INFORMATION = _SYSTEM_PERFORMANCE_INFORMATION {
    IdleProcessTime: _LARGE_INTEGER {
        __bindgen_anon_1: _LARGE_INTEGER__bindgen_ty_1 { LowPart: 0,
            HighPart: 0,
        },
    },
    IoReadTransferCount: _LARGE_INTEGER {
        __bindgen_anon_1: _LARGE_INTEGER__bindgen_ty_1 { LowPart: 0,
            HighPart: 0,
        },
    },
    IoWriteTransferCount: _LARGE_INTEGER {
        __bindgen_anon_1: _LARGE_INTEGER__bindgen_ty_1 { LowPart: 0,
            HighPart: 0,
        },
    },
    IoOtherTransferCount: _LARGE_INTEGER {
        __bindgen_anon_1: _LARGE_INTEGER__bindgen_ty_1 { LowPart: 0,
            HighPart: 0,
        },
    },
    IoReadOperationCount: 0,
    IoWriteOperationCount: 0,
    IoOtherOperationCount: 0,
    AvailablePages: 0,
    CommittedPages: 0,
    CommitLimit: 0,
    PeakCommitment: 0,
    PageFaultCount: 0,
    CopyOnWriteCount: 0,
    TransitionCount: 0,
    CacheTransitionCount: 0,
    DemandZeroCount: 0,
    PageReadCount: 0,
    PageReadIoCount: 0,
    CacheReadCount: 0,
    CacheIoCount: 0,
    DirtyPagesWriteCount: 0,
    DirtyWriteIoCount: 0,
    MappedPagesWriteCount: 0,
    MappedWriteIoCount: 0,
    PagedPoolPages: 0,
    NonPagedPoolPages: 0,
    PagedPoolAllocs: 0,
    PagedPoolFrees: 0,
    NonPagedPoolAllocs: 0,
    NonPagedPoolFrees: 0,
    FreeSystemPtes: 0,
    ResidentSystemCodePage: 0,
    TotalSystemDriverPages: 0,
    TotalSystemCodePages: 0,
    NonPagedPoolLookasideHits: 0,
    PagedPoolLookasideHits: 0,
    Spare3Count: 0,
    ResidentSystemCachePage: 0,
    ResidentPagedPoolPage: 0,
    ResidentSystemDriverPage: 0,
    CcFastReadNoWait: 0,
    CcFastReadWait: 0,
    CcFastReadResourceMiss: 0,
    CcFastReadNotPossible: 0,
    CcFastMdlReadNoWait: 0,
    CcFastMdlReadWait: 0,
    CcFastMdlReadResourceMiss: 0,
    CcFastMdlReadNotPossible: 0,
    CcMapDataNoWait: 0,
    CcMapDataWait: 0,
    CcMapDataNoWaitMiss: 0,
    CcMapDataWaitMiss: 0,
    CcPinMappedDataCount: 0,
    CcPinReadNoWait: 0,
    CcPinReadWait: 0,
    CcPinReadNoWaitMiss: 0,
    CcPinReadWaitMiss: 0,
    CcCopyReadNoWait: 0,
    CcCopyReadWait: 0,
    CcCopyReadNoWaitMiss: 0,
    CcCopyReadWaitMiss: 0,
    CcMdlReadNoWait: 0,
    CcMdlReadWait: 0,
    CcMdlReadNoWaitMiss: 0,
    CcMdlReadWaitMiss: 0,
    CcReadAheadIos: 0,
    CcLazyWriteIos: 0,
    CcLazyWritePages: 0,
    CcDataFlushes: 0,
    CcDataPages: 0,
    ContextSwitches: 0,
    FirstLevelTbFills: 0,
    SecondLevelTbFills: 0,
    SystemCalls: 0,
};

#[no_mangle]
pub static mut SystemBasicInfo: SYSTEM_BASIC_INFORMATION = _SYSTEM_BASIC_INFORMATION {
    Reserved: 0,
    TimerResolution: 0,
    PageSize: 0,
    NumberOfPhysicalPages: 0,
    LowestPhysicalPageNumber: 0,
    HighestPhysicalPageNumber: 0,
    AllocationGranularity: 0,
    MinimumUserModeAddress: 0,
    MaximumUserModeAddress: 0,
    ActiveProcessorsAffinityMask: 0,
    NumberOfProcessors: 0,
};

#[no_mangle]
pub static mut SystemCacheInfo: SYSTEM_FILECACHE_INFORMATION = _SYSTEM_FILECACHE_INFORMATION {
    CurrentSize: 0,
    PeakSize: 0,
    PageFaultCount: 0,
    MinimumWorkingSet: 0,
    MaximumWorkingSet: 0,
    CurrentSizeIncludingTransitionInPages: 0,
    PeakSizeIncludingTransitionInPages: 0,
    TransitionRePurposeCount: 0,
    Flags: 0,
};

#[no_mangle]
pub static mut SystemNumberOfHandles: ULONG = 0;

#[no_mangle]
pub static mut SystemProcessorTimeInfo: PSYSTEM_PROCESSOR_PERFORMANCE_INFORMATION =
    ::core::ptr::null::<_SYSTEM_PROCESSOR_PERFORMANCE_INFORMATION>()
        as *mut _SYSTEM_PROCESSOR_PERFORMANCE_INFORMATION;

#[no_mangle]
pub static mut SystemUserSid: PSID = NULL;

#[no_mangle]
pub static mut global_cache: PCMD_LINE_CACHE =
    ::core::ptr::null::<_CMD_LINE_CACHE>() as *mut _CMD_LINE_CACHE;

static mut SidToUserNameHead: LIST_ENTRY = unsafe {
    _LIST_ENTRY {
        Flink: &raw const SidToUserNameHead as *mut _LIST_ENTRY,
        Blink: &raw const SidToUserNameHead as *mut _LIST_ENTRY,
    }
};

#[repr(C)]
pub struct _SIDTOUSERNAME {
    pub List: LIST_ENTRY,
    pub pszName: LPWSTR,
    pub Data: [BYTE; 0],
}
pub type SIDTOUSERNAME = _SIDTOUSERNAME;
pub type PSIDTOUSERNAME = *mut _SIDTOUSERNAME;
#[no_mangle]
pub unsafe extern "C" fn PerfDataInitialize() -> BOOL {
    let mut NtSidAuthority: SID_IDENTIFIER_AUTHORITY = SID_IDENTIFIER_AUTHORITY {
        Value: [0, 0, 0, 0, 0, SECURITY_NT_AUTHORITY as u8],
    };
    let mut status: NTSTATUS;

    InitializeCriticalSection(core::ptr::addr_of_mut!(PerfDataCriticalSection));

    /*
     * Get number of processors in the system
     */
    status = NtQuerySystemInformation(
        SystemBasicInformation,
        core::ptr::addr_of_mut!(SystemBasicInfo) as PVOID,
        core::mem::size_of::<SYSTEM_BASIC_INFORMATION>() as ULONG,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS!(status) {
        return (FALSE) as BOOL;
    }

    /*
     * Create the SYSTEM Sid
     */
    AllocateAndInitializeSid(
        core::ptr::addr_of_mut!(NtSidAuthority),
        1,
        SECURITY_LOCAL_SYSTEM_RID,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        core::ptr::addr_of_mut!(SystemUserSid),
    );

    /*
     * Set up global info storage
     */
    SystemProcessorTimeInfo = HeapAlloc(
        GetProcessHeap(),
        0,
        (core::mem::size_of::<SYSTEM_PROCESSOR_PERFORMANCE_INFORMATION>() as SIZE_T)
            .wrapping_mul(SystemBasicInfo.NumberOfProcessors as SIZE_T),
    ) as PSYSTEM_PROCESSOR_PERFORMANCE_INFORMATION;

    return (SystemProcessorTimeInfo != core::ptr::null_mut()) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn PerfDataUninitialize() {
    let mut pCur: PLIST_ENTRY = core::ptr::null_mut();
    let mut pEntry: PSIDTOUSERNAME = core::ptr::null_mut();

    if !pPerfData.is_null() {
        HeapFree(GetProcessHeap(), 0, pPerfData as LPVOID);
    }

    DeleteCriticalSection(core::ptr::addr_of_mut!(PerfDataCriticalSection));

    if !SystemUserSid.is_null() {
        FreeSid(SystemUserSid);
        SystemUserSid = core::ptr::null_mut();
    }

    /* Free user names cache list */
    pCur = (*core::ptr::addr_of_mut!(SidToUserNameHead)).Flink;
    while pCur != core::ptr::addr_of_mut!(SidToUserNameHead) {
        pEntry = CONTAINING_RECORD!(pCur, SIDTOUSERNAME, List);
        pCur = (*pCur).Flink;
        HeapFree(GetProcessHeap(), 0, pEntry as LPVOID);
    }

    if !SystemProcessorTimeInfo.is_null() {
        HeapFree(GetProcessHeap(), 0, SystemProcessorTimeInfo as LPVOID);
    }
}

unsafe extern "C" fn SidToUserName(mut Sid: PSID, mut szBuffer: LPWSTR, mut BufferSize: DWORD) {
    static mut szDomainNameUnused: [WCHAR; 255] = [0; 255];
    let mut DomainNameLen: DWORD = _countof!(szDomainNameUnused) as DWORD;
    let mut Use: SID_NAME_USE = 0 as SID_NAME_USE;

    if !Sid.is_null() {
        LookupAccountSidW(
            core::ptr::null::<WCHAR>(),
            Sid,
            szBuffer,
            core::ptr::addr_of_mut!(BufferSize),
            core::ptr::addr_of_mut!(szDomainNameUnused) as LPWSTR,
            core::ptr::addr_of_mut!(DomainNameLen),
            core::ptr::addr_of_mut!(Use),
        );
    }
}
#[no_mangle]
pub unsafe extern "system" fn CachedGetUserFromSid(
    pSid: PSID,
    pUserName: LPWSTR,
    pcwcUserName: PULONG,
) {
    let mut pCur: PLIST_ENTRY = core::ptr::null_mut();
    let mut pEntry: *mut SIDTOUSERNAME = core::ptr::null_mut();
    let mut cbSid: ULONG = 0;
    let mut cwcUserName: ULONG = 0;

    cwcUserName = *pcwcUserName;

    /* Walk through the list */
    pCur = SidToUserNameHead.Flink;
    while pCur != core::ptr::addr_of_mut!(SidToUserNameHead) {
        pEntry = ((pCur as ULONG_PTR)
            .wrapping_sub(core::mem::offset_of!(SIDTOUSERNAME, List) as ULONG_PTR))
            as *mut SIDTOUSERNAME;
        if EqualSid(
            core::ptr::addr_of_mut!((*pEntry).Data) as PSID,
            pSid,
        ) != 0
        {
            wcsncpy(pUserName, (*pEntry).pszName, (cwcUserName) as _);
            *pcwcUserName = wcslen(pUserName) as ULONG;
            return;
        }
        pCur = (*pCur).Flink;
    }

    /* We didn't find the SID in the list, get the name conventional */
    SidToUserName(pSid, pUserName, cwcUserName as DWORD);
    *pcwcUserName = wcslen(pUserName) as ULONG;

    /* Allocate a new entry */
    cwcUserName = (*pcwcUserName).wrapping_add(1);
    cbSid = GetLengthSid(pSid) as ULONG;
    pEntry = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        (core::mem::size_of::<SIDTOUSERNAME>() as SIZE_T)
            .wrapping_add(cbSid as SIZE_T)
            .wrapping_add(
                (cwcUserName as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
            ),
    ) as *mut SIDTOUSERNAME;

    /* Copy the Sid and name to our entry */
    CopySid(
        cbSid as DWORD,
        core::ptr::addr_of_mut!((*pEntry).Data) as PSID,
        pSid,
    );
    (*pEntry).pszName =
        (core::ptr::addr_of_mut!((*pEntry).Data) as *mut BYTE).offset(cbSid as isize) as LPWSTR;
    wcsncpy((*pEntry).pszName, pUserName, (cwcUserName) as _);

    /* Insert the new entry */
    (*pEntry).List.Flink = core::ptr::addr_of_mut!(SidToUserNameHead);
    (*pEntry).List.Blink = SidToUserNameHead.Blink;
    (*SidToUserNameHead.Blink).Flink = core::ptr::addr_of_mut!((*pEntry).List);
    SidToUserNameHead.Blink = core::ptr::addr_of_mut!((*pEntry).List);
}
#[no_mangle]
pub unsafe extern "C" fn PerfDataRefresh() {
    let mut ulSize: ULONG = 0;
    let mut status: NTSTATUS = 0;
    let mut pBuffer: LPBYTE = core::ptr::null_mut();
    let mut BufferSize: ULONG = 0;
    let mut pSPI: PSYSTEM_PROCESS_INFORMATION = core::ptr::null_mut();
    let mut pPDOld: PPERFDATA = core::ptr::null_mut();
    let mut Idx: ULONG = 0;
    let mut Idx2: ULONG = 0;
    let mut hProcess: HANDLE = core::ptr::null_mut();
    let mut hProcessToken: HANDLE = core::ptr::null_mut();
    let mut SysPerfInfo: SYSTEM_PERFORMANCE_INFORMATION = core::mem::zeroed();
    let mut SysTimeInfo: SYSTEM_TIMEOFDAY_INFORMATION = core::mem::zeroed();
    let mut SysCacheInfo: SYSTEM_FILECACHE_INFORMATION = core::mem::zeroed();
    let mut SysHandleInfoData: SYSTEM_HANDLE_INFORMATION = core::mem::zeroed();
    let mut SysProcessorTimeInfo: PSYSTEM_PROCESSOR_PERFORMANCE_INFORMATION = core::ptr::null_mut();
    let mut CurrentKernelTime: f64 = 0.0;
    let mut ProcessSD: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
    let mut ProcessUser: PSID = core::ptr::null_mut();
    let mut Buffer: [ULONG; 64] = [0; 64]; /* must be 4 bytes aligned! */
    let mut cwcUserName: ULONG = 0;
    let mut bIsWow64: BOOL = 0;

    /* Get new system time */
    status = NtQuerySystemInformation(SystemTimeOfDayInformation, addr_of_mut!(SysTimeInfo) as PVOID, core::mem::size_of::<SYSTEM_TIMEOFDAY_INFORMATION>() as ULONG, core::ptr::null_mut());
    if !NT_SUCCESS!(status) {
        return;
    }

    /* Get new CPU's idle time */
    status = NtQuerySystemInformation(SystemPerformanceInformation, addr_of_mut!(SysPerfInfo) as PVOID, core::mem::size_of::<SYSTEM_PERFORMANCE_INFORMATION>() as ULONG, core::ptr::null_mut());
    if !NT_SUCCESS!(status) {
        return;
    }

    /* Get system cache information */
    status = NtQuerySystemInformation(SystemFileCacheInformation, addr_of_mut!(SysCacheInfo) as PVOID, core::mem::size_of::<SYSTEM_FILECACHE_INFORMATION>() as ULONG, core::ptr::null_mut());
    if !NT_SUCCESS!(status) {
        return;
    }

    /* Get processor time information */
    SysProcessorTimeInfo = HeapAlloc(GetProcessHeap(), 0 as DWORD, (core::mem::size_of::<SYSTEM_PROCESSOR_PERFORMANCE_INFORMATION>() as SIZE_T).wrapping_mul(SystemBasicInfo.NumberOfProcessors as SIZE_T)) as PSYSTEM_PROCESSOR_PERFORMANCE_INFORMATION;
    status = NtQuerySystemInformation(SystemProcessorPerformanceInformation, SysProcessorTimeInfo as PVOID, (core::mem::size_of::<SYSTEM_PROCESSOR_PERFORMANCE_INFORMATION>() as ULONG).wrapping_mul(SystemBasicInfo.NumberOfProcessors as ULONG), addr_of_mut!(ulSize));

    if !NT_SUCCESS!(status) {
        if !SysProcessorTimeInfo.is_null() {
            HeapFree(GetProcessHeap(), 0 as DWORD, SysProcessorTimeInfo as PVOID);
        }
        return;
    }

    /* Get handle information
     * Number of handles is enough, no need for data array.
     */
    status = NtQuerySystemInformation(SystemHandleInformation, addr_of_mut!(SysHandleInfoData) as PVOID, core::mem::size_of::<SYSTEM_HANDLE_INFORMATION>() as ULONG, core::ptr::null_mut());
    /* On unexpected error, reuse previous value.
     * STATUS_SUCCESS (0-1 handle) should never happen.
     */
    if status != STATUS_INFO_LENGTH_MISMATCH {
        SysHandleInfoData.NumberOfHandles = SystemNumberOfHandles;
    }

    /* Get process information
     * We don't know how much data there is so just keep
     * increasing the buffer size until the call succeeds
     */
    BufferSize = 0;
    loop {
        BufferSize += 0x10000;
        pBuffer = HeapAlloc(GetProcessHeap(), 0 as DWORD, BufferSize as SIZE_T) as LPBYTE;

        status = NtQuerySystemInformation(SystemProcessInformation, pBuffer as PVOID, BufferSize, addr_of_mut!(ulSize));

        if status == STATUS_INFO_LENGTH_MISMATCH {
            HeapFree(GetProcessHeap(), 0 as DWORD, pBuffer as LPVOID);
        }

        if !(status == STATUS_INFO_LENGTH_MISMATCH) {
            break;
        }
    }

    EnterCriticalSection(addr_of_mut!(PerfDataCriticalSection));

    /*
     * Save system performance info
     */
    memcpy(addr_of_mut!(SystemPerfInfo) as *mut ::core::ffi::c_void, addr_of_mut!(SysPerfInfo) as *const ::core::ffi::c_void, core::mem::size_of::<SYSTEM_PERFORMANCE_INFORMATION>() as usize);

    /*
     * Save system cache info
     */
    memcpy(addr_of_mut!(SystemCacheInfo) as *mut ::core::ffi::c_void, addr_of_mut!(SysCacheInfo) as *const ::core::ffi::c_void, core::mem::size_of::<SYSTEM_FILECACHE_INFORMATION>() as usize);

    /*
     * Save system processor time info
     */
    memcpy(SystemProcessorTimeInfo as *mut ::core::ffi::c_void, SysProcessorTimeInfo as *const ::core::ffi::c_void,
           (core::mem::size_of::<SYSTEM_PROCESSOR_PERFORMANCE_INFORMATION>() as usize).wrapping_mul(SystemBasicInfo.NumberOfProcessors as usize));

    if !SysProcessorTimeInfo.is_null() {
        HeapFree(GetProcessHeap(), 0 as DWORD, SysProcessorTimeInfo as PVOID);
    }

    /*
     * Save system handle info
     */
    SystemNumberOfHandles = SysHandleInfoData.NumberOfHandles;

    CurrentKernelTime = 0 as f64;
    Idx = 0 as ULONG;
    while Idx < SystemBasicInfo.NumberOfProcessors as ULONG {
        CurrentKernelTime += Li2Double!((*SystemProcessorTimeInfo.offset(Idx as isize)).KernelTime);
        CurrentKernelTime += Li2Double!((*SystemProcessorTimeInfo.offset(Idx as isize)).DpcTime);
        CurrentKernelTime += Li2Double!((*SystemProcessorTimeInfo.offset(Idx as isize)).InterruptTime);
        Idx = Idx.wrapping_add(1);
    }

    /* If it's a first call - skip idle time calcs */
    if liOldIdleTime.QuadPart != 0 as LONGLONG {
        /*  CurrentValue = NewValue - OldValue */
        dbIdleTime = Li2Double!(SysPerfInfo.IdleProcessTime) - Li2Double!(liOldIdleTime);
        dbKernelTime = CurrentKernelTime - OldKernelTime;
        dbSystemTime = Li2Double!(SysTimeInfo.CurrentTime) - Li2Double!(liOldSystemTime);

        /*  CurrentCpuIdle = IdleTime / SystemTime */
        dbIdleTime = dbIdleTime / dbSystemTime;
        dbKernelTime = dbKernelTime / dbSystemTime;

        /*  CurrentCpuUsage% = 100 - (CurrentCpuIdle * 100) / NumberOfProcessors */
        dbIdleTime = 100.0 - dbIdleTime * 100.0 / (SystemBasicInfo.NumberOfProcessors as f64); /* + 0.5; */
        dbKernelTime = 100.0 - dbKernelTime * 100.0 / (SystemBasicInfo.NumberOfProcessors as f64); /* + 0.5; */
    }

    /* Store new CPU's idle and system time */
    liOldIdleTime = SysPerfInfo.IdleProcessTime;
    liOldSystemTime = SysTimeInfo.CurrentTime;
    OldKernelTime = CurrentKernelTime;

    /* Determine the process count
     * We loop through the data we got from NtQuerySystemInformation
     * and count how many structures there are (until RelativeOffset is 0)
     */
    ProcessCountOld = ProcessCount;
    ProcessCount = 0 as ULONG;
    pSPI = pBuffer as PSYSTEM_PROCESS_INFORMATION;
    while !pSPI.is_null() {
        ProcessCount = ProcessCount.wrapping_add(1);
        if (*pSPI).NextEntryOffset == 0 as ULONG {
            break;
        }
        pSPI = (pSPI as LPBYTE).offset((*pSPI).NextEntryOffset as isize) as PSYSTEM_PROCESS_INFORMATION;
    }

    /* Now alloc a new PERFDATA array and fill in the data */
    pPerfData = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY as DWORD, (core::mem::size_of::<PERFDATA>() as SIZE_T).wrapping_mul(ProcessCount as SIZE_T)) as PPERFDATA;

    pSPI = pBuffer as PSYSTEM_PROCESS_INFORMATION;
    Idx = 0 as ULONG;
    while Idx < ProcessCount {
        /* Get the old perf data for this process (if any) */
        /* so that we can establish delta values */
        pPDOld = core::ptr::null_mut();
        if !pPerfDataOld.is_null() {
            Idx2 = 0 as ULONG;
            while Idx2 < ProcessCountOld {
                if (*pPerfDataOld.offset(Idx2 as isize)).ProcessId == (*pSPI).UniqueProcessId {
                    pPDOld = pPerfDataOld.offset(Idx2 as isize);
                    break;
                }
                Idx2 = Idx2.wrapping_add(1);
            }
        }

        if !(*pSPI).ImageName.Buffer.is_null() {
            /* Don't assume a _UNICODE_STRING Buffer is zero terminated: */
            let mut len: ::core::ffi::c_int = (*pSPI).ImageName.Length as ::core::ffi::c_int / 2;
            /* Check against max size and allow for terminating zero (already zeroed): */
            if (len) as u32 >= MAX_PATH {
                len = (MAX_PATH - 1) as i32;
            }
            wcsncpy(addr_of_mut!((*pPerfData.offset(Idx as isize)).ImageName) as LPWSTR, (*pSPI).ImageName.Buffer as *const u16, len as usize);
        } else {
            LoadStringW(hInst, IDS_IDLE_PROCESS as UINT, addr_of_mut!((*pPerfData.offset(Idx as isize)).ImageName) as LPWSTR,
                       _countof!((*pPerfData.offset(Idx as isize)).ImageName) as ::core::ffi::c_int);
        }

        (*pPerfData.offset(Idx as isize)).ProcessId = (*pSPI).UniqueProcessId;

        if !pPDOld.is_null() {
            let mut CurTime: f64 = Li2Double!((*pSPI).KernelTime) + Li2Double!((*pSPI).UserTime);
            let mut OldTime: f64 = Li2Double!((*pPDOld).KernelTime) + Li2Double!((*pPDOld).UserTime);
            let mut CpuTime: f64 = (CurTime - OldTime) / dbSystemTime;
            CpuTime = CpuTime * 100.0 / (SystemBasicInfo.NumberOfProcessors as f64); /* + 0.5; */
            (*pPerfData.offset(Idx as isize)).CPUUsage = CpuTime as ULONG;
        }
        (*pPerfData.offset(Idx as isize)).CPUTime.QuadPart = (*pSPI).UserTime.QuadPart + (*pSPI).KernelTime.QuadPart;
        (*pPerfData.offset(Idx as isize)).WorkingSetSizeBytes = (*pSPI).WorkingSetSize as ULONG;
        (*pPerfData.offset(Idx as isize)).PeakWorkingSetSizeBytes = (*pSPI).PeakWorkingSetSize as ULONG;
        if !pPDOld.is_null() {
            (*pPerfData.offset(Idx as isize)).WorkingSetSizeDelta = labs((*pSPI).WorkingSetSize as LONG - (*pPDOld).WorkingSetSizeBytes as LONG) as ULONG;
        } else {
            (*pPerfData.offset(Idx as isize)).WorkingSetSizeDelta = 0;
        }
        (*pPerfData.offset(Idx as isize)).PageFaultCount = (*pSPI).PageFaultCount;
        if !pPDOld.is_null() {
            (*pPerfData.offset(Idx as isize)).PageFaultCountDelta = labs((*pSPI).PageFaultCount as LONG - (*pPDOld).PageFaultCount as LONG) as ULONG;
        } else {
            (*pPerfData.offset(Idx as isize)).PageFaultCountDelta = 0;
        }
        (*pPerfData.offset(Idx as isize)).VirtualMemorySizeBytes = (*pSPI).VirtualSize as ULONG;
        (*pPerfData.offset(Idx as isize)).PagedPoolUsagePages = (*pSPI).QuotaPeakPagedPoolUsage as ULONG;
        (*pPerfData.offset(Idx as isize)).NonPagedPoolUsagePages = (*pSPI).QuotaPeakNonPagedPoolUsage as ULONG;
        (*pPerfData.offset(Idx as isize)).BasePriority = (*pSPI).BasePriority as ULONG;
        (*pPerfData.offset(Idx as isize)).HandleCount = (*pSPI).HandleCount;
        (*pPerfData.offset(Idx as isize)).ThreadCount = (*pSPI).NumberOfThreads;
        (*pPerfData.offset(Idx as isize)).SessionId = (*pSPI).SessionId;
        (*pPerfData.offset(Idx as isize)).UserName[0 as usize] = UNICODE_NULL;
        (*pPerfData.offset(Idx as isize)).USERObjectCount = 0 as ULONG;
        (*pPerfData.offset(Idx as isize)).GDIObjectCount = 0 as ULONG;
        ProcessUser = SystemUserSid;
        ProcessSD = core::ptr::null_mut();

        // C's `goto ClearInfo` reaches the ZeroMemory only from the two failure paths; the
        // success path leaves past it ('fetched), or every process's I/O counters read 0.
        'fetched: {
        'clear_info: {
            if !(*pSPI).UniqueProcessId.is_null() {
                hProcess = OpenProcess((PROCESS_QUERY_INFORMATION | READ_CONTROL) as DWORD, (FALSE) as BOOL, PtrToUlong!((*pSPI).UniqueProcessId as *const ::core::ffi::c_void) as DWORD);
                if !hProcess.is_null() {
                    /* don't query the information of the system process. It's possible but
                       returns Administrators as the owner of the process instead of SYSTEM */
                    if (*pSPI).UniqueProcessId != 0x4 as usize as HANDLE
                    {
                        // Likewise `goto ReadProcOwner` is the token query's FAILURE path: the
                        // object owner (often Administrators) must not overwrite the token user.
                        'owner_known: {
                        'read_owner: {
                            if OpenProcessToken(hProcess, TOKEN_QUERY, addr_of_mut!(hProcessToken)) != 0
                            {
                                let mut RetLen: DWORD = 0;
                                let mut Ret: BOOL;

                                Ret = GetTokenInformation(hProcessToken, TokenUser, addr_of_mut!(Buffer) as LPVOID, core::mem::size_of::<[ULONG; 64]>() as DWORD, addr_of_mut!(RetLen));
                                CloseHandle(hProcessToken);

                                if Ret != 0 {
                                    ProcessUser = (*(addr_of_mut!(Buffer) as PTOKEN_USER)).User.Sid;
                                    break 'owner_known;
                                }
                                /* else goto ReadProcOwner: falls through to the owner query below */
                            }
                        }
                        /* ReadProcOwner: */
                        GetSecurityInfo(hProcess, SE_KERNEL_OBJECT, OWNER_SECURITY_INFORMATION, addr_of_mut!(ProcessUser), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), addr_of_mut!(ProcessSD));
                        }

                        (*pPerfData.offset(Idx as isize)).USERObjectCount = GetGuiResources(hProcess, GR_USEROBJECTS) as ULONG;
                        (*pPerfData.offset(Idx as isize)).GDIObjectCount = GetGuiResources(hProcess, GR_GDIOBJECTS) as ULONG;
                    }

                    if IsWow64Process(hProcess, addr_of_mut!(bIsWow64)) != 0 && bIsWow64 != 0
                    {
                        wcscat(addr_of_mut!((*pPerfData.offset(Idx as isize)).ImageName) as LPWSTR, RUSTCONV_WSTR_839B5A81141C.as_ptr());
                    }

                    GetProcessIoCounters(hProcess, addr_of_mut!((*pPerfData.offset(Idx as isize)).IOCounters));
                    CloseHandle(hProcess);
                    break 'fetched;
                }
                /* else goto ClearInfo: falls through */
            }
        }
        /* ClearInfo: */
        /* clear information we were unable to fetch */
        ZeroMemory!(addr_of_mut!((*pPerfData.offset(Idx as isize)).IOCounters) as PVOID, core::mem::size_of::<IO_COUNTERS>());
        }

        cwcUserName = _countof!((*pPerfData.offset(0 as isize)).UserName) as ULONG;
        CachedGetUserFromSid(ProcessUser, addr_of_mut!((*pPerfData.offset(Idx as isize)).UserName) as LPWSTR, addr_of_mut!(cwcUserName));

        if !ProcessSD.is_null()
        {
            LocalFree(ProcessSD as HLOCAL);
        }

        (*pPerfData.offset(Idx as isize)).UserTime.QuadPart = (*pSPI).UserTime.QuadPart;
        (*pPerfData.offset(Idx as isize)).KernelTime.QuadPart = (*pSPI).KernelTime.QuadPart;
        pSPI = (pSPI as LPBYTE).offset((*pSPI).NextEntryOffset as isize) as PSYSTEM_PROCESS_INFORMATION;
        Idx = Idx.wrapping_add(1);
    }
    HeapFree(GetProcessHeap(), 0 as DWORD, pBuffer as LPVOID);
    if !pPerfDataOld.is_null() {
        HeapFree(GetProcessHeap(), 0 as DWORD, pPerfDataOld as LPVOID);
    }
    pPerfDataOld = pPerfData;
    LeaveCriticalSection(addr_of_mut!(PerfDataCriticalSection));
}
#[no_mangle]
pub unsafe extern "C" fn PerfDataGetProcessIndex(mut pid: ULONG) -> ULONG {
    let mut idx: ULONG = 0;

    EnterCriticalSection(core::ptr::addr_of_mut!(PerfDataCriticalSection));

    while idx < ProcessCount {
        if PtrToUlong!((*pPerfData.offset(idx as isize)).ProcessId as *const ::core::ffi::c_void) == pid {
            break;
        }
        idx = idx.wrapping_add(1);
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!(PerfDataCriticalSection));

    if idx == ProcessCount {
        return -(1 as ::core::ffi::c_int) as ULONG;
    }
    return idx;
}
#[no_mangle]
pub unsafe extern "C" fn PerfDataGetProcessCount() -> ULONG {
    let mut Result: ULONG;
    EnterCriticalSection(core::ptr::addr_of_mut!(PerfDataCriticalSection));
    Result = ProcessCount;
    LeaveCriticalSection(core::ptr::addr_of_mut!(PerfDataCriticalSection));
    return Result;
}

#[no_mangle]
pub unsafe extern "C" fn PerfDataGetProcessorUsage() -> ULONG {
    let mut Result: ULONG;
    EnterCriticalSection(core::ptr::addr_of_mut!(PerfDataCriticalSection));
    Result = (min!(max!(dbIdleTime, (0 as f64)), (100 as f64))) as ULONG;
    LeaveCriticalSection(core::ptr::addr_of_mut!(PerfDataCriticalSection));
    return Result;
}
#[no_mangle]
pub unsafe extern "C" fn PerfDataGetProcessorSystemUsage() -> ULONG {
    let mut Result: ULONG;
    EnterCriticalSection(core::ptr::addr_of_mut!(PerfDataCriticalSection));
    Result = min!(max!(dbKernelTime, 0.), 100.) as ULONG;
    LeaveCriticalSection(core::ptr::addr_of_mut!(PerfDataCriticalSection));
    return Result;
}
#[no_mangle]
pub unsafe extern "C" fn PerfDataGetImageName(Index: ULONG, lpImageName: LPWSTR, nMaxCount: ULONG) -> BOOL {
    let mut bSuccessful: BOOL;

    EnterCriticalSection(core::ptr::addr_of_mut!(PerfDataCriticalSection));

    if Index < ProcessCount {
        wcsncpy(
            lpImageName as *mut u16,
            core::ptr::addr_of_mut!((*pPerfData.add(Index as usize)).ImageName) as *mut u16,
            nMaxCount as usize,
        );
        bSuccessful = (TRUE) as i32;
    } else {
        bSuccessful = (FALSE) as i32;
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!(PerfDataCriticalSection));
    return bSuccessful;
}
#[no_mangle]
pub unsafe extern "C" fn PerfDataGetProcessId(Index: ULONG) -> ULONG {
    let ProcessId: ULONG;

    EnterCriticalSection(core::ptr::addr_of_mut!(PerfDataCriticalSection));

    if Index < ProcessCount {
        ProcessId = PtrToUlong!((*pPerfData.offset(Index as isize)).ProcessId as *const ::core::ffi::c_void) as ULONG;
    } else {
        ProcessId = 0;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!(PerfDataCriticalSection));

    return ProcessId;
}
#[no_mangle]
pub unsafe extern "C" fn PerfDataGetUserName(Index: ULONG, lpUserName: LPWSTR, nMaxCount: ULONG) -> BOOL {
    let mut bSuccessful: BOOL;

    EnterCriticalSection(core::ptr::addr_of_mut!(PerfDataCriticalSection));

    if Index < ProcessCount {
        wcsncpy(
            lpUserName,
            core::ptr::addr_of_mut!((*pPerfData.offset(Index as isize)).UserName) as LPWSTR,
            nMaxCount as usize,
        );
        bSuccessful = (TRUE) as i32;
    } else {
        bSuccessful = (FALSE) as i32;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!(PerfDataCriticalSection));

    return bSuccessful;
}
#[no_mangle]
pub unsafe extern "C" fn PerfDataGetCommandLine(Index: ULONG, lpCommandLine: LPWSTR, nMaxCount: ULONG) -> BOOL {
    let ellipsis: LPWSTR = RUSTCONV_WSTR_59A543594151.as_ptr() as LPWSTR;

    let mut pbi: PROCESS_BASIC_INFORMATION = core::mem::zeroed();
    let mut CommandLineStr: _UNICODE_STRING = core::mem::zeroed();

    let mut ProcessParams: PVOID = core::ptr::null_mut();
    let mut hProcess: HANDLE = core::ptr::null_mut();
    let mut ProcessId: ULONG = 0;

    let mut Status: NTSTATUS = 0;
    let mut result: BOOL = 0;

    let mut new_entry: PCMD_LINE_CACHE = core::ptr::null_mut();
    let mut new_string: LPWSTR = core::ptr::null_mut();

    let mut cache: PCMD_LINE_CACHE = global_cache;
    let mut tail: PCMD_LINE_CACHE = core::ptr::null_mut();

    if nMaxCount == 0 {
        return (TRUE) as BOOL;
    }

    // Divergence from the C: it keyed the cache by the list-view `Index`, which shifts whenever
    // a process above it starts or exits (so a row showed another process's command line), and
    // its walk stopped BEFORE the last entry (`cache->pnext != NULL`), so the newest entry was
    // never found and was re-read and appended again. Entries are keyed by process id, every
    // entry is searched, and `tail` remembers where to append.
    ProcessId = PerfDataGetProcessId(Index);

    /* [A] Search for a string already in cache? If so, use it */
    while !cache.is_null() {
        if (*cache).idx == ProcessId && !(*cache).str_.is_null() {
            CmdLineCopy(lpCommandLine, nMaxCount, (*cache).str_, (*cache).len, ellipsis);
            return (TRUE) as BOOL;
        }

        tail = cache;
        cache = (*cache).pnext;
    }

    /* [B] We don't; let's allocate and load a value from the process mem... and cache it */

    /* Default blank command line in case things don't work out */
    wcsncpy(lpCommandLine as *mut u16, RUSTCONV_WSTR_96A296D224F2.as_ptr(), nMaxCount as usize);

    /* Ask for a handle to the target process so that we can read its memory and query stuff */
    hProcess = OpenProcess(
        PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
        (FALSE) as BOOL,
        ProcessId,
    );

    'cleanup: {
        if hProcess.is_null() {
            break 'cleanup;
        }

        /* First off, get the ProcessEnvironmentBlock location in that process' address space */
        Status = NtQueryInformationProcess(
            hProcess,
            0,
            core::ptr::addr_of_mut!(pbi) as PVOID,
            core::mem::size_of::<PROCESS_BASIC_INFORMATION>() as ULONG,
            core::ptr::null_mut(),
        );
        if !NT_SUCCESS!(Status) {
            break 'cleanup;
        }

        /* Then get the PEB.ProcessParameters member pointer */
        result = ReadProcessMemory(
            hProcess,
            ((pbi.PebBaseAddress as ULONG_PTR)
                .wrapping_add(FIELD_OFFSET!(PEB, ProcessParameters) as ULONG_PTR))
                as PVOID,
            core::ptr::addr_of_mut!(ProcessParams) as LPVOID,
            (core::mem::size_of::<PVOID>()) as SIZE_T,
            core::ptr::null_mut(),
        );
        if result == 0 {
            break 'cleanup;
        }

        /* Then copy the PEB->ProcessParameters.CommandLine member
           to get the pointer to the string buffer and its size */
        result = ReadProcessMemory(
            hProcess,
            ((ProcessParams as ULONG_PTR)
                .wrapping_add(FIELD_OFFSET!(RTL_USER_PROCESS_PARAMETERS, CommandLine) as ULONG_PTR))
                as PVOID,
            core::ptr::addr_of_mut!(CommandLineStr) as LPVOID,
            (core::mem::size_of::<_UNICODE_STRING>()) as SIZE_T,
            core::ptr::null_mut(),
        );
        if result == 0 {
            break 'cleanup;
        }

        /* Allocate the next cache entry and its accompanying string in one go */
        new_entry = HeapAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY,
            ((core::mem::size_of::<CMD_LINE_CACHE>() as usize)
                .wrapping_add(CommandLineStr.Length as usize)
                .wrapping_add(core::mem::size_of_val(&UNICODE_NULL) as usize)) as SIZE_T,
        ) as PCMD_LINE_CACHE;
        if new_entry.is_null() {
            break 'cleanup;
        }

        new_string = ((new_entry as ULONG_PTR)
            .wrapping_add(core::mem::size_of::<CMD_LINE_CACHE>() as ULONG_PTR))
            as LPWSTR;

        /* Bingo, the command line should be stored there,
           copy the string from the other process */
        result = ReadProcessMemory(
            hProcess,
            CommandLineStr.Buffer as LPCVOID,
            new_string as LPVOID,
            (CommandLineStr.Length) as SIZE_T,
            core::ptr::null_mut(),
        );
        if result == 0 {
            /* Weird, after successfully reading the mem of that process
               various times it fails now, forget it and bail out */
            HeapFree(
                GetProcessHeap(),
                0,
                new_entry as PVOID,
            );
            break 'cleanup;
        }

        /* Add our pointer to the cache... */
        // `len` is in WCHARs; the C stored the UNICODE_STRING's byte Length and compared it
        // against the WCHAR count `nMaxCount`.
        (*new_entry).idx = ProcessId;
        (*new_entry).str_ = new_string;
        (*new_entry).len = (CommandLineStr.Length as ULONG) / core::mem::size_of::<WCHAR>() as ULONG;

        if global_cache.is_null() {
            global_cache = new_entry;
        } else {
            (*tail).pnext = new_entry;
        }

        /* ... and print the buffer for the first time */
        CmdLineCopy(lpCommandLine, nMaxCount, new_string, (*new_entry).len, ellipsis);
    }

    // cleanup:
    if !hProcess.is_null() {
        CloseHandle(hProcess);
    }
    return (TRUE) as BOOL;
}
/// Copies a cached command line of `len` WCHARs into a `nMaxCount`-WCHAR buffer, always
/// terminated, ending in "..." only when it had to be cut. Divergence from the C, which mixed
/// bytes and WCHARs here (`CMD_LINE_MIN` subtracted `sizeof(WCHAR)` from a WCHAR count) and
/// wrote the ellipsis at `min(..) - 3` unconditionally: a command line under three bytes put it
/// before the start of the caller's buffer.
unsafe fn CmdLineCopy(lpCommandLine: LPWSTR, nMaxCount: ULONG, str_: LPWSTR, len: ULONG, ellipsis: LPWSTR) {
    let ellipsis_len = wcslen(ellipsis as *const u16) as ULONG;
    if len < nMaxCount {
        wcsncpy(lpCommandLine as *mut u16, str_ as *const u16, len as usize);
        *lpCommandLine.add(len as usize) = 0;
        return;
    }
    let keep = nMaxCount - 1;
    wcsncpy(lpCommandLine as *mut u16, str_ as *const u16, keep as usize);
    *lpCommandLine.add(keep as usize) = 0;
    if keep >= ellipsis_len {
        wcscpy(lpCommandLine.add((keep - ellipsis_len) as usize) as *mut u16, ellipsis as *const u16);
    }
}
#[no_mangle]
pub unsafe extern "C" fn PerfDataDeallocCommandLineCache() {
    let mut cache: PCMD_LINE_CACHE = core::ptr::null_mut();
    let mut pnext: PCMD_LINE_CACHE = core::ptr::null_mut();
    cache = global_cache;
    while !cache.is_null() {
        pnext = (*cache).pnext as PCMD_LINE_CACHE;
        HeapFree(GetProcessHeap(), 0 as DWORD, cache as PVOID);
        cache = pnext;
    }
    global_cache = core::ptr::null_mut();
}
#[no_mangle]
pub unsafe extern "C" fn PerfDataGetSessionId(mut Index: ULONG) -> ULONG {
    let mut SessionId_local: ULONG = 0;

    EnterCriticalSection(core::ptr::addr_of_mut!(PerfDataCriticalSection));

    if Index < ProcessCount {
        SessionId_local = (*pPerfData.add(Index as usize)).SessionId;
    } else {
        SessionId_local = 0;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!(PerfDataCriticalSection));

    return SessionId_local;
}
#[no_mangle]
pub unsafe extern "C" fn PerfDataGetCPUUsage(Index: ULONG) -> ULONG {
    let mut CpuUsage: ULONG;

    EnterCriticalSection(core::ptr::addr_of_mut!(PerfDataCriticalSection));

    if Index < ProcessCount {
        CpuUsage = (*pPerfData.offset(Index as isize)).CPUUsage;
    } else {
        CpuUsage = 0;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!(PerfDataCriticalSection));

    return CpuUsage;
}
#[no_mangle]
pub unsafe extern "C" fn PerfDataGetCPUTime(mut Index: ULONG) -> LARGE_INTEGER {
    let mut CpuTime: LARGE_INTEGER = LARGE_INTEGER {
        __bindgen_anon_1: _LARGE_INTEGER__bindgen_ty_1 { LowPart: 0,
            HighPart: 0,
        },
    };
    EnterCriticalSection(core::ptr::addr_of_mut!(PerfDataCriticalSection));
    if Index < ProcessCount {
        CpuTime = (*pPerfData.offset(Index as isize)).CPUTime;
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!(PerfDataCriticalSection));
    return CpuTime;
}
#[no_mangle]
pub unsafe extern "C" fn PerfDataGetWorkingSetSizeBytes(Index: ULONG) -> ULONG {
    let mut WorkingSetSizeBytes: ULONG;

    EnterCriticalSection(core::ptr::addr_of_mut!(PerfDataCriticalSection));

    if Index < ProcessCount {
        WorkingSetSizeBytes = (*pPerfData.offset(Index as isize)).WorkingSetSizeBytes;
    } else {
        WorkingSetSizeBytes = 0;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!(PerfDataCriticalSection));

    return WorkingSetSizeBytes;
}
#[no_mangle]
pub unsafe extern "C" fn PerfDataGetPeakWorkingSetSizeBytes(Index: ULONG) -> ULONG {
    let mut PeakWorkingSetSizeBytes: ULONG;

    EnterCriticalSection(core::ptr::addr_of_mut!(PerfDataCriticalSection));

    if Index < ProcessCount {
        PeakWorkingSetSizeBytes = (*pPerfData.offset(Index as isize)).PeakWorkingSetSizeBytes;
    } else {
        PeakWorkingSetSizeBytes = 0;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!(PerfDataCriticalSection));

    return PeakWorkingSetSizeBytes;
}
#[no_mangle]
pub unsafe extern "C" fn PerfDataGetWorkingSetSizeDelta(Index: ULONG) -> ULONG {
    let mut WorkingSetSizeDelta: ULONG;

    EnterCriticalSection(core::ptr::addr_of_mut!(PerfDataCriticalSection));

    if Index < ProcessCount {
        WorkingSetSizeDelta = (*pPerfData.add(Index as usize)).WorkingSetSizeDelta;
    } else {
        WorkingSetSizeDelta = 0;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!(PerfDataCriticalSection));

    return WorkingSetSizeDelta;
}
#[no_mangle]
pub unsafe extern "C" fn PerfDataGetPageFaultCount(Index: ULONG) -> ULONG {
    let mut PageFaultCount: ULONG;

    EnterCriticalSection(core::ptr::addr_of_mut!(PerfDataCriticalSection));

    if Index < ProcessCount {
        PageFaultCount = (*pPerfData.add(Index as usize)).PageFaultCount;
    } else {
        PageFaultCount = 0;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!(PerfDataCriticalSection));

    return PageFaultCount;
}
#[no_mangle]
pub unsafe extern "C" fn PerfDataGetPageFaultCountDelta(Index: ULONG) -> ULONG {
    let mut PageFaultCountDelta: ULONG;

    EnterCriticalSection(core::ptr::addr_of_mut!(PerfDataCriticalSection));

    if Index < ProcessCount {
        PageFaultCountDelta = (*pPerfData.add(Index as usize)).PageFaultCountDelta;
    } else {
        PageFaultCountDelta = 0;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!(PerfDataCriticalSection));

    return PageFaultCountDelta;
}
#[no_mangle]
pub unsafe extern "C" fn PerfDataGetVirtualMemorySizeBytes(Index: ULONG) -> ULONG {
    let mut VirtualMemorySizeBytes: ULONG;

    EnterCriticalSection(core::ptr::addr_of_mut!(PerfDataCriticalSection));

    if Index < ProcessCount {
        VirtualMemorySizeBytes = (*pPerfData.add(Index as usize)).VirtualMemorySizeBytes;
    } else {
        VirtualMemorySizeBytes = 0;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!(PerfDataCriticalSection));

    return VirtualMemorySizeBytes;
}
#[no_mangle]
pub unsafe extern "C" fn PerfDataGetPagedPoolUsagePages(Index: ULONG) -> ULONG {
    let mut PagedPoolUsage: ULONG;

    EnterCriticalSection(core::ptr::addr_of_mut!(PerfDataCriticalSection));

    if Index < ProcessCount {
        PagedPoolUsage = (*pPerfData.offset(Index as isize)).PagedPoolUsagePages;
    } else {
        PagedPoolUsage = 0;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!(PerfDataCriticalSection));

    return PagedPoolUsage;
}
#[no_mangle]
pub unsafe extern "C" fn PerfDataGetNonPagedPoolUsagePages(Index: ULONG) -> ULONG {
    let mut NonPagedPoolUsage: ULONG;

    EnterCriticalSection(core::ptr::addr_of_mut!(PerfDataCriticalSection));

    if Index < ProcessCount {
        NonPagedPoolUsage = (*pPerfData.offset(Index as isize)).NonPagedPoolUsagePages;
    } else {
        NonPagedPoolUsage = 0;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!(PerfDataCriticalSection));

    return NonPagedPoolUsage;
}
#[no_mangle]
pub unsafe extern "C" fn PerfDataGetBasePriority(Index: ULONG) -> ULONG {
    let mut BasePriority: ULONG;

    EnterCriticalSection(core::ptr::addr_of_mut!(PerfDataCriticalSection));

    if Index < ProcessCount {
        BasePriority = (*pPerfData.offset(Index as isize)).BasePriority;
    } else {
        BasePriority = 0;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!(PerfDataCriticalSection));

    return BasePriority;
}
#[no_mangle]
pub unsafe extern "C" fn PerfDataGetHandleCount(Index: ULONG) -> ULONG {
    let mut HandleCount: ULONG;

    EnterCriticalSection(core::ptr::addr_of_mut!(PerfDataCriticalSection));

    if Index < ProcessCount {
        HandleCount = (*pPerfData.offset(Index as isize)).HandleCount;
    } else {
        HandleCount = 0;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!(PerfDataCriticalSection));

    return HandleCount;
}
#[no_mangle]
pub unsafe extern "C" fn PerfDataGetThreadCount(Index: ULONG) -> ULONG {
    let mut ThreadCount: ULONG;

    EnterCriticalSection(core::ptr::addr_of_mut!(PerfDataCriticalSection));

    if Index < ProcessCount {
        ThreadCount = (*pPerfData.offset(Index as isize)).ThreadCount;
    } else {
        ThreadCount = 0;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!(PerfDataCriticalSection));

    return ThreadCount;
}
#[no_mangle]
pub unsafe extern "C" fn PerfDataGetUSERObjectCount(Index: ULONG) -> ULONG {
    let mut USERObjectCount: ULONG;

    EnterCriticalSection(core::ptr::addr_of_mut!(PerfDataCriticalSection));

    if Index < ProcessCount {
        USERObjectCount = (*pPerfData.offset(Index as isize)).USERObjectCount;
    } else {
        USERObjectCount = 0;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!(PerfDataCriticalSection));

    return USERObjectCount;
}
#[no_mangle]
pub unsafe extern "C" fn PerfDataGetGDIObjectCount(Index: ULONG) -> ULONG {
    let mut GDIObjectCount: ULONG;

    EnterCriticalSection(core::ptr::addr_of_mut!(PerfDataCriticalSection));

    if Index < ProcessCount {
        GDIObjectCount = (*pPerfData.offset(Index as isize)).GDIObjectCount;
    } else {
        GDIObjectCount = 0;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!(PerfDataCriticalSection));

    return GDIObjectCount;
}
#[no_mangle]
pub unsafe extern "C" fn PerfDataGetIOCounters(Index: ULONG, pIoCounters: PIO_COUNTERS) -> BOOL {
    let mut bSuccessful: BOOL = 0;

    EnterCriticalSection(core::ptr::addr_of_mut!(PerfDataCriticalSection));

    if Index < ProcessCount {
        memcpy(
            pIoCounters as *mut core::ffi::c_void,
            core::ptr::addr_of_mut!((*pPerfData.add(Index as usize)).IOCounters) as *const core::ffi::c_void,
            core::mem::size_of::<IO_COUNTERS>() as size_t,
        );
        bSuccessful = TRUE as BOOL;
    } else {
        bSuccessful = FALSE as BOOL;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!(PerfDataCriticalSection));

    return bSuccessful;
}
#[no_mangle]
pub unsafe extern "C" fn PerfDataGetCommitChargeTotalK() -> ULONG {
    let mut Total: ULONG;
    let mut PageSize: ULONG;

    EnterCriticalSection(core::ptr::addr_of_mut!(PerfDataCriticalSection));

    Total = SystemPerfInfo.CommittedPages;
    PageSize = SystemBasicInfo.PageSize;

    LeaveCriticalSection(core::ptr::addr_of_mut!(PerfDataCriticalSection));

    Total = Total.wrapping_mul(PageSize / 1024);

    return Total;
}

#[no_mangle]
pub unsafe extern "C" fn PerfDataGetCommitChargeLimitK() -> ULONG {
    let mut Limit: ULONG;
    let mut PageSize: ULONG;

    EnterCriticalSection(core::ptr::addr_of_mut!(PerfDataCriticalSection));

    Limit = SystemPerfInfo.CommitLimit;
    PageSize = SystemBasicInfo.PageSize;

    LeaveCriticalSection(core::ptr::addr_of_mut!(PerfDataCriticalSection));

    Limit = Limit.wrapping_mul(PageSize.wrapping_div(1024));

    return Limit;
}
#[no_mangle]
pub unsafe extern "C" fn PerfDataGetCommitChargePeakK() -> ULONG {
    let mut Peak: ULONG;
    let mut PageSize: ULONG;

    EnterCriticalSection(core::ptr::addr_of_mut!(PerfDataCriticalSection));

    Peak = (*core::ptr::addr_of!(SystemPerfInfo)).PeakCommitment;
    PageSize = (*core::ptr::addr_of!(SystemBasicInfo)).PageSize;

    LeaveCriticalSection(core::ptr::addr_of_mut!(PerfDataCriticalSection));

    Peak = Peak.wrapping_mul(PageSize / 1024);

    return Peak;
}
#[no_mangle]
pub unsafe extern "C" fn PerfDataGetKernelMemoryTotalK() -> ULONG {
    let mut Total: ULONG;
    let mut Paged: ULONG;
    let mut NonPaged: ULONG;
    let mut PageSize: ULONG;

    EnterCriticalSection(core::ptr::addr_of_mut!(PerfDataCriticalSection));

    Paged = SystemPerfInfo.PagedPoolPages;
    NonPaged = SystemPerfInfo.NonPagedPoolPages;
    PageSize = SystemBasicInfo.PageSize;

    LeaveCriticalSection(core::ptr::addr_of_mut!(PerfDataCriticalSection));

    Paged = Paged.wrapping_mul(PageSize / 1024);
    NonPaged = NonPaged.wrapping_mul(PageSize / 1024);

    Total = Paged.wrapping_add(NonPaged);

    return Total;
}

#[no_mangle]
pub unsafe extern "C" fn PerfDataGetKernelMemoryPagedK() -> ULONG {
    let mut Paged: ULONG;
    let mut PageSize: ULONG;

    EnterCriticalSection(core::ptr::addr_of_mut!(PerfDataCriticalSection));

    Paged = SystemPerfInfo.PagedPoolPages;
    PageSize = SystemBasicInfo.PageSize;

    LeaveCriticalSection(core::ptr::addr_of_mut!(PerfDataCriticalSection));

    Paged = Paged.wrapping_mul(PageSize.wrapping_div(1024));

    return Paged;
}
#[no_mangle]
pub unsafe extern "C" fn PerfDataGetKernelMemoryNonPagedK() -> ULONG {
    let mut NonPaged: ULONG;
    let mut PageSize: ULONG;

    EnterCriticalSection(core::ptr::addr_of_mut!(PerfDataCriticalSection));

    NonPaged = SystemPerfInfo.NonPagedPoolPages;
    PageSize = SystemBasicInfo.PageSize;

    LeaveCriticalSection(core::ptr::addr_of_mut!(PerfDataCriticalSection));

    NonPaged = NonPaged.wrapping_mul(PageSize / 1024);

    return NonPaged;
}
#[no_mangle]
pub unsafe extern "C" fn PerfDataGetPhysicalMemoryTotalK() -> ULONG {
    let mut Total: ULONG;
    let mut PageSize: ULONG;

    EnterCriticalSection(core::ptr::addr_of_mut!(PerfDataCriticalSection));

    Total = SystemBasicInfo.NumberOfPhysicalPages;
    PageSize = SystemBasicInfo.PageSize;

    LeaveCriticalSection(core::ptr::addr_of_mut!(PerfDataCriticalSection));

    Total = Total.wrapping_mul(PageSize / (1024 as ULONG));

    return Total;
}
#[no_mangle]
pub unsafe extern "C" fn PerfDataGetPhysicalMemoryAvailableK() -> ULONG {
    let mut Available: ULONG;
    let mut PageSize: ULONG;

    EnterCriticalSection(core::ptr::addr_of_mut!(PerfDataCriticalSection));

    Available = (*core::ptr::addr_of!(SystemPerfInfo)).AvailablePages;
    PageSize = (*core::ptr::addr_of!(SystemBasicInfo)).PageSize;

    LeaveCriticalSection(core::ptr::addr_of_mut!(PerfDataCriticalSection));

    Available = Available.wrapping_mul(PageSize / 1024);

    return Available;
}
#[no_mangle]
pub unsafe extern "C" fn PerfDataGetPhysicalMemorySystemCacheK() -> ULONG {
    let mut SystemCache: ULONG = 0;
    let mut PageSize: ULONG = 0;

    EnterCriticalSection(core::ptr::addr_of_mut!(PerfDataCriticalSection));

    PageSize = SystemBasicInfo.PageSize;
    SystemCache = SystemCacheInfo
        .CurrentSizeIncludingTransitionInPages
        .wrapping_mul(PageSize as SIZE_T) as ULONG;

    LeaveCriticalSection(core::ptr::addr_of_mut!(PerfDataCriticalSection));

    return SystemCache / 1024;
}
#[no_mangle]
pub unsafe extern "C" fn PerfDataGetSystemHandleCount() -> ULONG {
    let mut HandleCount: ULONG;

    EnterCriticalSection(core::ptr::addr_of_mut!(PerfDataCriticalSection));

    HandleCount = SystemNumberOfHandles;

    LeaveCriticalSection(core::ptr::addr_of_mut!(PerfDataCriticalSection));

    return HandleCount;
}
#[no_mangle]
pub unsafe extern "C" fn PerfDataGetTotalThreadCount() -> ULONG {
    let mut ThreadCount: ULONG = 0;
    let mut i: ULONG;

    EnterCriticalSection(core::ptr::addr_of_mut!(PerfDataCriticalSection));

    i = 0;
    while i < ProcessCount {
        ThreadCount = ThreadCount.wrapping_add((*pPerfData.offset(i as isize)).ThreadCount);
        i = i.wrapping_add(1);
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!(PerfDataCriticalSection));

    return ThreadCount;
}
#[no_mangle]
pub unsafe extern "C" fn PerfDataGet(Index: ULONG, lppData: *mut PPERFDATA) -> BOOL {
    let mut bSuccessful: BOOL = (FALSE) as i32;

    EnterCriticalSection(core::ptr::addr_of_mut!(PerfDataCriticalSection));
    if Index < ProcessCount {
        *lppData = pPerfData.add(Index as usize);
        bSuccessful = (TRUE) as i32;
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!(PerfDataCriticalSection));
    return bSuccessful;
}
