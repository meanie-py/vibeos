use crate::schedsvc::job::CalculateNextStartTime;
use crate::schedsvc::job::DeleteJob;
use crate::schedsvc::job::SaveJob;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_12A6BF85117F: [u16; 9] = [0x006Eu16, 0x0063, 0x0061, 0x0063, 0x006E, 0x005F, 0x006E, 0x0070, 0x0000];
pub static RUSTCONV_WSTR_D54FDAF080C8: [u16; 12] = [
    0x005Cu16, 0x0070, 0x0069, 0x0070, 0x0065, 0x005C, 0x0061, 0x0074, 0x0073, 0x0076,
    0x0063, 0x0000,
];

use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn RpcThreadRoutine(lpParameter: LPVOID) -> DWORD {
    let mut Status: RPC_STATUS = 0;

    Status = RpcServerUseProtseqEpW(RUSTCONV_WSTR_12A6BF85117F.as_ptr() as *mut u16, 20, RUSTCONV_WSTR_D54FDAF080C8.as_ptr() as *mut u16, core::ptr::null_mut());
    if Status != RPC_S_OK
    {
        return 0;
    }

    Status = RpcServerRegisterIf(atsvc_v1_0_s_ifspec, core::ptr::null_mut(), core::ptr::null_mut());
    if Status != RPC_S_OK
    {
        return 0;
    }

    Status = RpcServerListen(1, RPC_C_LISTEN_MAX_CALLS_DEFAULT, FALSE);
    if Status != RPC_S_OK
    {
    }

    return 0;
}
#[export_name = "MIDL_user_allocate"]
pub unsafe extern "system" fn midl_user_allocate(len: SIZE_T) -> *mut core::ffi::c_void {
    return HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, len) as *mut core::ffi::c_void;
}
#[export_name = "MIDL_user_free"]
pub unsafe extern "system" fn midl_user_free(ptr: *mut core::ffi::c_void) {
    HeapFree(GetProcessHeap(), 0, ptr);
}
#[no_mangle]
pub unsafe extern "system" fn NetrJobAdd(
    ServerName: ATSVC_HANDLE,
    pAtInfo: LPAT_INFO,
    pJobId: LPDWORD,
) -> NET_API_STATUS {
    let mut pJob: PJOB;

    /* Allocate a new job object */
    pJob = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY as DWORD,
        (core::mem::size_of::<JOB>() as SIZE_T).wrapping_add(
            (wcslen(((*pAtInfo).Command) as *const WCHAR) as SIZE_T)
                .wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
        ),
    ) as PJOB;
    if pJob.is_null() {
        return ERROR_OUTOFMEMORY as NET_API_STATUS;
    }

    /* Initialize the job object */
    (*pJob).JobTime = (*pAtInfo).JobTime;
    (*pJob).DaysOfMonth = (*pAtInfo).DaysOfMonth;
    (*pJob).DaysOfWeek = (*pAtInfo).DaysOfWeek as UCHAR;
    (*pJob).Flags = (*pAtInfo).Flags as UCHAR;
    wcscpy(
        core::ptr::addr_of_mut!((*pJob).Command) as *mut WCHAR,
        ((*pAtInfo).Command) as *const WCHAR,
    );

    /* Acquire the job list lock exclusively */
    RtlAcquireResourceExclusive(
        core::ptr::addr_of_mut!(JobListLock),
        TRUE as BOOLEAN,
    );

    /* Assign a new job ID */
    (*pJob).JobId = dwNextJobId;
    dwNextJobId = dwNextJobId.wrapping_add(1);
    dwJobCount = dwJobCount.wrapping_add(1);

    /* Append the new job to the job list */
    InsertTailList(
        core::ptr::addr_of_mut!(JobListHead),
        core::ptr::addr_of_mut!((*pJob).JobEntry),
    );

    /* Save the job in the registry */
    SaveJob(pJob);

    /* Calculate the next start time */
    CalculateNextStartTime(pJob);

    /* Release the job list lock */
    RtlReleaseResource(core::ptr::addr_of_mut!(JobListLock));

    /* Set the update event */
    if !Events[1 as usize].is_null() {
        SetEvent(Events[1 as usize]);
    }

    /* Return the new job ID */
    *pJobId = (*pJob).JobId;

    return ERROR_SUCCESS as NET_API_STATUS;
}
#[no_mangle]
pub unsafe extern "system" fn NetrJobDel(mut ServerName: ATSVC_HANDLE, mut MinJobId: DWORD, mut MaxJobId: DWORD) -> NET_API_STATUS {
    let mut JobEntry: PLIST_ENTRY = ::core::ptr::null_mut::<_LIST_ENTRY>();
    let mut NextEntry: PLIST_ENTRY = ::core::ptr::null_mut::<_LIST_ENTRY>();
    let mut CurrentJob: PJOB = ::core::ptr::null_mut::<_JOB>();
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"base/services/schedsvc/rpcserver.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"NetrJobDel\0" as *const u8 as *const ::core::ffi::c_char,
            154 as ::core::ffi::c_int,
            b"NetrJobDel(%S %lu %lu)\n\0" as *const u8 as *const ::core::ffi::c_char,
            ServerName,
            MinJobId,
            MaxJobId,
        );
    }

    /* Check the job IDs */
    if MinJobId > MaxJobId {
        return ERROR_INVALID_PARAMETER;
    }

    /* Acquire the job list lock exclusively */
    RtlAcquireResourceExclusive(&raw mut JobListLock, TRUE as BOOLEAN);

    JobEntry = JobListHead.Flink as PLIST_ENTRY;
    while JobEntry != &raw mut JobListHead {
        CurrentJob = CONTAINING_RECORD!(JobEntry, JOB, JobEntry);

        if (*CurrentJob).JobId >= MinJobId && (*CurrentJob).JobId <= MaxJobId {
            /* Remove the job from the registry */
            DeleteJob(CurrentJob);

            NextEntry = (*JobEntry).Flink as PLIST_ENTRY;
            if RemoveEntryList(JobEntry) {
                dwJobCount = dwJobCount.wrapping_sub(1);
                HeapFree(
                    GetProcessHeap(),
                    0,
                    CurrentJob as PVOID,
                );
                JobEntry = NextEntry;
                continue;
            }
        }

        JobEntry = (*JobEntry).Flink as PLIST_ENTRY;
    }

    /* Release the job list lock */
    RtlReleaseResource(&raw mut JobListLock);

    /* Set the update event */
    if !Events[1 as usize].is_null() {
        SetEvent(Events[1 as usize]);
    }

    return ERROR_SUCCESS;
}
#[no_mangle]
pub unsafe extern "system" fn NetrJobEnum(
    ServerName: ATSVC_HANDLE,
    pEnumContainer: LPAT_ENUM_CONTAINER,
    PreferedMaximumLength: DWORD,
    pTotalEntries: LPDWORD,
    pResumeHandle: LPDWORD,
) -> NET_API_STATUS {
    let mut JobEntry: PLIST_ENTRY = core::ptr::null_mut();
    let mut CurrentJob: PJOB = core::ptr::null_mut();
    let mut pEnum: PAT_ENUM = core::ptr::null_mut();
    let mut dwStartIndex: DWORD = 0;
    let mut dwIndex: DWORD = 0;
    let mut dwEntriesToRead: DWORD = 0;
    let mut dwEntriesRead: DWORD = 0;
    let mut dwRequiredSize: DWORD = 0;
    let mut dwEntrySize: DWORD = 0;
    let mut pString: PWSTR = core::ptr::null_mut();
    let mut dwError: DWORD = ERROR_SUCCESS as DWORD;

    if pEnumContainer.is_null() {
        *pTotalEntries = 0 as DWORD;
        return ERROR_INVALID_PARAMETER as NET_API_STATUS;
    }

    if *pResumeHandle >= dwJobCount {
        *pTotalEntries = 0 as DWORD;
        return ERROR_SUCCESS as NET_API_STATUS;
    }

    dwStartIndex = *pResumeHandle;

    'done: {
        /* Acquire the job list lock exclusively */
        RtlAcquireResourceShared(core::ptr::addr_of_mut!(JobListLock), TRUE as BOOLEAN);

        dwEntriesToRead = 0 as DWORD;
        dwRequiredSize = 0 as DWORD;
        dwIndex = 0 as DWORD;
        JobEntry = JobListHead.Flink;
        while JobEntry != core::ptr::addr_of_mut!(JobListHead) {
            CurrentJob = CONTAINING_RECORD!(JobEntry, JOB, JobEntry);

            if dwIndex >= dwStartIndex {
                dwEntrySize = (core::mem::size_of::<AT_ENUM>() as DWORD).wrapping_add(
                    ((wcslen(core::ptr::addr_of_mut!((*CurrentJob).Command) as *mut WCHAR) as DWORD)
                        .wrapping_add(1 as DWORD))
                        .wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD),
                );

                if PreferedMaximumLength != ULONG_MAX as DWORD
                    && dwRequiredSize.wrapping_add(dwEntrySize) > PreferedMaximumLength
                {
                    break;
                }

                dwRequiredSize = dwRequiredSize.wrapping_add(dwEntrySize);
                dwEntriesToRead = dwEntriesToRead.wrapping_add(1);
            }

            JobEntry = (*JobEntry).Flink;
            dwIndex = dwIndex.wrapping_add(1);
        }

        if PreferedMaximumLength != ULONG_MAX as DWORD {
            dwRequiredSize = PreferedMaximumLength;
        }

        pEnum = midl_user_allocate(dwRequiredSize as SIZE_T) as PAT_ENUM;
        if pEnum.is_null() {
            dwError = ERROR_OUTOFMEMORY as DWORD;
            break 'done;
        }

        pString = ((pEnum as usize).wrapping_add(
            (dwEntriesToRead as usize).wrapping_mul(core::mem::size_of::<AT_ENUM>() as usize),
        )) as PWSTR;

        dwEntriesRead = 0 as DWORD;
        dwIndex = 0 as DWORD;
        JobEntry = JobListHead.Flink;
        while JobEntry != core::ptr::addr_of_mut!(JobListHead) {
            CurrentJob = CONTAINING_RECORD!(JobEntry, JOB, JobEntry);

            if dwIndex >= dwStartIndex {
                (*pEnum.offset(dwIndex as isize)).JobId = (*CurrentJob).JobId;
                (*pEnum.offset(dwIndex as isize)).JobTime = (*CurrentJob).JobTime;
                (*pEnum.offset(dwIndex as isize)).DaysOfMonth = (*CurrentJob).DaysOfMonth;
                (*pEnum.offset(dwIndex as isize)).DaysOfWeek = (*CurrentJob).DaysOfWeek;
                (*pEnum.offset(dwIndex as isize)).Flags = (*CurrentJob).Flags;
                (*pEnum.offset(dwIndex as isize)).Command = pString;
                wcscpy(pString, core::ptr::addr_of_mut!((*CurrentJob).Command) as *mut WCHAR);

                pString = ((pString as usize).wrapping_add(
                    ((wcslen(core::ptr::addr_of_mut!((*CurrentJob).Command) as *mut WCHAR) as usize)
                        .wrapping_add(1 as usize))
                        .wrapping_mul(core::mem::size_of::<WCHAR>() as usize),
                )) as PWSTR;

                dwEntriesRead = dwEntriesRead.wrapping_add(1);
            }

            if dwEntriesRead == dwEntriesToRead {
                break;
            }

            /* Next job */
            JobEntry = (*JobEntry).Flink;
            dwIndex = dwIndex.wrapping_add(1);
        }

        (*pEnumContainer).EntriesRead = dwEntriesRead;
        (*pEnumContainer).Buffer = pEnum as LPAT_ENUM;

        *pTotalEntries = dwJobCount;
        *pResumeHandle = dwIndex;

        if dwEntriesRead.wrapping_add(dwStartIndex) < dwJobCount {
            dwError = ERROR_MORE_DATA as DWORD;
        } else {
            dwError = ERROR_SUCCESS as DWORD;
        }
    }

    /* Release the job list lock */
    RtlReleaseResource(core::ptr::addr_of_mut!(JobListLock));

    return dwError as NET_API_STATUS;
}

#[no_mangle]
pub unsafe extern "system" fn NetrJobGetInfo(
    ServerName: ATSVC_HANDLE,
    JobId: DWORD,
    ppAtInfo: *mut LPAT_INFO,
) -> NET_API_STATUS {
    let mut JobEntry: PLIST_ENTRY = core::ptr::null_mut();
    let mut CurrentJob: PJOB = core::ptr::null_mut();
    let mut pInfo: PAT_INFO = core::ptr::null_mut();
    let mut dwError: DWORD = ERROR_FILE_NOT_FOUND;

    /* Acquire the job list lock exclusively */
    RtlAcquireResourceShared(core::ptr::addr_of_mut!(JobListLock), TRUE as BOOLEAN);

    'done: {
        /* Traverse the job list */
        JobEntry = JobListHead.Flink;
        while JobEntry != core::ptr::addr_of_mut!(JobListHead) {
            CurrentJob = CONTAINING_RECORD!(JobEntry, JOB, JobEntry);

            /* Do we have the right job? */
            if (*CurrentJob).JobId == JobId {
                pInfo = midl_user_allocate(
                    core::mem::size_of::<AT_INFO>() as SIZE_T,
                ) as PAT_INFO;
                if pInfo.is_null() {
                    dwError = ERROR_OUTOFMEMORY;
                    break 'done;
                }

                (*pInfo).Command = midl_user_allocate(
                    ((wcslen(core::ptr::addr_of_mut!((*CurrentJob).Command) as PWSTR)
                        as SIZE_T
                        + 1)
                        * core::mem::size_of::<WCHAR>() as SIZE_T),
                ) as PWSTR;
                if (*pInfo).Command.is_null() {
                    midl_user_free(pInfo as LPVOID);
                    dwError = ERROR_OUTOFMEMORY;
                    break 'done;
                }

                (*pInfo).JobTime = (*CurrentJob).JobTime;
                (*pInfo).DaysOfMonth = (*CurrentJob).DaysOfMonth;
                (*pInfo).DaysOfWeek = (*CurrentJob).DaysOfWeek;
                (*pInfo).Flags = (*CurrentJob).Flags;
                wcscpy(
                    (*pInfo).Command,
                    core::ptr::addr_of_mut!((*CurrentJob).Command) as PWSTR,
                );

                *ppAtInfo = pInfo as LPAT_INFO;

                dwError = ERROR_SUCCESS;
                break 'done;
            }

            /* Next job */
            JobEntry = (*JobEntry).Flink;
        }
    }

    /* Release the job list lock */
    RtlReleaseResource(core::ptr::addr_of_mut!(JobListLock));

    return dwError;
}
