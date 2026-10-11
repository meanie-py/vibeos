// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_181895D51D74: [u16; 6] = [0x0025u16, 0x0030, 0x0038, 0x006C, 0x0078, 0x0000];
pub static RUSTCONV_WSTR_2F006FE06135: [u16; 9] = [0x0053u16, 0x0063, 0x0068, 0x0065, 0x0064, 0x0075, 0x006C, 0x0065, 0x0000];
pub static RUSTCONV_WSTR_425FC060B288: [u16; 8] = [0x0043u16, 0x006F, 0x006D, 0x006D, 0x0061, 0x006E, 0x0064, 0x0000];
pub static RUSTCONV_WSTR_5A2E2E7B1604: [u16; 48] = [
    0x0053u16, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x005C, 0x0043, 0x0075, 0x0072,
    0x0072, 0x0065, 0x006E, 0x0074, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F,
    0x006C, 0x0053, 0x0065, 0x0074, 0x005C, 0x0053, 0x0065, 0x0072, 0x0076, 0x0069,
    0x0063, 0x0065, 0x0073, 0x005C, 0x0053, 0x0063, 0x0068, 0x0065, 0x0064, 0x0075,
    0x006C, 0x0065, 0x005C, 0x004A, 0x006F, 0x0062, 0x0073, 0x0000,
];
pub static RUSTCONV_WSTR_A3C18557FFAF: [u16; 16] = [
    0x0057u16, 0x0069, 0x006E, 0x0053, 0x0074, 0x0061, 0x0030, 0x005C, 0x0044, 0x0065,
    0x0066, 0x0061, 0x0075, 0x006C, 0x0074, 0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut dwNextJobId: DWORD = 0 as DWORD;

#[no_mangle]
pub static mut dwJobCount: DWORD = 0 as DWORD;

#[no_mangle]
pub static mut JobListHead: LIST_ENTRY = _LIST_ENTRY {
    Flink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
    Blink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
};

#[no_mangle]
pub static mut JobListLock: RTL_RESOURCE = _RTL_RESOURCE {
    Lock: _RTL_CRITICAL_SECTION {
        DebugInfo: ::core::ptr::null::<_RTL_CRITICAL_SECTION_DEBUG>()
            as *mut _RTL_CRITICAL_SECTION_DEBUG,
        LockCount: 0,
        RecursionCount: 0,
        OwningThread: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
        LockSemaphore: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
        SpinCount: 0,
    },
    SharedSemaphore: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    SharedWaiters: 0,
    ExclusiveSemaphore: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    ExclusiveWaiters: 0,
    NumberActive: 0,
    OwningThread: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    TimeoutBoost: 0,
    DebugInfo: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
};

#[no_mangle]
pub static mut StartListHead: LIST_ENTRY = _LIST_ENTRY {
    Flink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
    Blink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
};

#[no_mangle]
pub static mut StartListLock: RTL_RESOURCE = _RTL_RESOURCE {
    Lock: _RTL_CRITICAL_SECTION {
        DebugInfo: ::core::ptr::null::<_RTL_CRITICAL_SECTION_DEBUG>()
            as *mut _RTL_CRITICAL_SECTION_DEBUG,
        LockCount: 0,
        RecursionCount: 0,
        OwningThread: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
        LockSemaphore: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
        SpinCount: 0,
    },
    SharedSemaphore: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    SharedWaiters: 0,
    ExclusiveSemaphore: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    ExclusiveWaiters: 0,
    NumberActive: 0,
    OwningThread: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    TimeoutBoost: 0,
    DebugInfo: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
};

#[no_mangle]
pub static mut NextJobStartTime: FILETIME = _FILETIME {
    dwLowDateTime: 0,
    dwHighDateTime: 0,
};

#[no_mangle]
pub static mut bValidNextJobStartTime: BOOL = (FALSE) as i32;

static mut wDaysArray: [WORD; 13] = [
    0 as ::core::ffi::c_int as WORD,
    31 as ::core::ffi::c_int as WORD,
    28 as ::core::ffi::c_int as WORD,
    31 as ::core::ffi::c_int as WORD,
    30 as ::core::ffi::c_int as WORD,
    31 as ::core::ffi::c_int as WORD,
    30 as ::core::ffi::c_int as WORD,
    31 as ::core::ffi::c_int as WORD,
    31 as ::core::ffi::c_int as WORD,
    30 as ::core::ffi::c_int as WORD,
    31 as ::core::ffi::c_int as WORD,
    30 as ::core::ffi::c_int as WORD,
    31 as ::core::ffi::c_int as WORD,
];

// WINE_DEFAULT_DEBUG_CHANNEL(schedsvc): expands to the Wine debug channel
// declaration for this translation unit. Recorded here as the channel name.

#[repr(C)]
pub struct SCHEDULE {
    pub JobTime: DWORD,
    pub DaysOfMonth: DWORD,
    pub DaysOfWeek: UCHAR,
    pub Flags: UCHAR,
    pub Reserved: WORD,
}

// C: `typedef struct _SCHEDULE {...} SCHEDULE, PSCHEDULE;` -- PSCHEDULE names the
// struct itself (not a pointer), so it is an alias of SCHEDULE.
pub type PSCHEDULE = SCHEDULE;
#[no_mangle]
pub unsafe extern "C" fn GetNextJobTimeout(hTimer: HANDLE) {
    let mut CurrentEntry: PLIST_ENTRY;
    let mut DueTime: FILETIME = core::mem::zeroed();
    let mut CurrentJob: PJOB;

    bValidNextJobStartTime = FALSE as BOOL;
    CurrentEntry = (*core::ptr::addr_of_mut!(JobListHead)).Flink;
    while CurrentEntry != core::ptr::addr_of_mut!(JobListHead) {
        CurrentJob = CONTAINING_RECORD!(CurrentEntry, JOB, JobEntry);

        if bValidNextJobStartTime == FALSE as BOOL {
            CopyMemory!(
                core::ptr::addr_of_mut!(NextJobStartTime) as *mut core::ffi::c_void,
                core::ptr::addr_of_mut!((*CurrentJob).StartTime) as *const core::ffi::c_void,
                core::mem::size_of::<FILETIME>(),
            );
            bValidNextJobStartTime = TRUE as BOOL;
        } else {
            if CompareFileTime(
                core::ptr::addr_of_mut!(NextJobStartTime),
                core::ptr::addr_of_mut!((*CurrentJob).StartTime),
            ) > 0
            {
                CopyMemory!(
                    core::ptr::addr_of_mut!(NextJobStartTime) as *mut core::ffi::c_void,
                    core::ptr::addr_of_mut!((*CurrentJob).StartTime) as *const core::ffi::c_void,
                    core::mem::size_of::<FILETIME>(),
                );
            }
        }

        CurrentEntry = (*CurrentEntry).Flink;
    }

    if bValidNextJobStartTime == FALSE as BOOL {
        return;
    }

    LocalFileTimeToFileTime(
        core::ptr::addr_of_mut!(DueTime),
        core::ptr::addr_of_mut!(NextJobStartTime),
    );

    SetWaitableTimer(
        hTimer,
        core::ptr::addr_of_mut!(DueTime) as PLARGE_INTEGER,
        0,
        None,
        core::ptr::null_mut(),
        (TRUE) as BOOL,
    );
}
#[no_mangle]
pub unsafe extern "C" fn RunCurrentJobs() {
    let mut ProcessInformation: PROCESS_INFORMATION = core::mem::zeroed();
    let mut StartupInfo: STARTUPINFOW = core::mem::zeroed();
    let mut CurrentEntry: PLIST_ENTRY;
    let mut CurrentJob: PJOB;
    let mut bRet: BOOL;

    CurrentEntry = (*core::ptr::addr_of_mut!(JobListHead)).Flink;
    while CurrentEntry != core::ptr::addr_of_mut!(JobListHead) {
        CurrentJob = CONTAINING_RECORD!(CurrentEntry, JOB, JobEntry);

        if CompareFileTime(
            core::ptr::addr_of_mut!(NextJobStartTime),
            core::ptr::addr_of_mut!((*CurrentJob).StartTime),
        ) == 0
        {
            ZeroMemory!(
                core::ptr::addr_of_mut!(StartupInfo) as *mut core::ffi::c_void,
                core::mem::size_of::<STARTUPINFOW>() as usize,
            );
            StartupInfo.cb = core::mem::size_of::<STARTUPINFOW>() as DWORD;
            StartupInfo.lpTitle = core::ptr::addr_of_mut!((*CurrentJob).Command) as *mut WCHAR as LPWSTR;
            StartupInfo.dwFlags = STARTF_USESHOWWINDOW;
            StartupInfo.wShowWindow = SW_SHOWDEFAULT as u16;

            if ((*CurrentJob).Flags as u32 & JOB_NONINTERACTIVE) == 0
            {
                StartupInfo.dwFlags |= STARTF_INHERITDESKTOP;
                StartupInfo.lpDesktop = RUSTCONV_WSTR_A3C18557FFAF.as_ptr() as LPWSTR;
            }

            bRet = CreateProcessW(
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!((*CurrentJob).Command) as LPWSTR,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                FALSE as BOOL,
                CREATE_NEW_CONSOLE,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(StartupInfo),
                core::ptr::addr_of_mut!(ProcessInformation),
            );
            if bRet == FALSE as BOOL
            {
            }
            else
            {
                CloseHandle(ProcessInformation.hThread);
                CloseHandle(ProcessInformation.hProcess);
            }
        }

        CurrentEntry = (*CurrentEntry).Flink;
    }
}

unsafe extern "C" fn GetJobName(hJobsKey: HKEY, pszJobName: PWSTR) {
    let mut szNameBuffer: [WCHAR; JOB_NAME_LENGTH as usize] = [0; JOB_NAME_LENGTH as usize];
    let mut SystemTime: FILETIME = FILETIME {
        dwLowDateTime: 0,
        dwHighDateTime: 0,
    };
    let mut ulSeed: ULONG = 0;
    let mut ulValue: ULONG = 0;
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut lError: LONG = 0;

    GetSystemTimeAsFileTime(core::ptr::addr_of_mut!(SystemTime));
    ulSeed = SystemTime.dwLowDateTime as ULONG;
    loop {
        ulValue = RtlRandomEx(core::ptr::addr_of_mut!(ulSeed));
        _swprintf(
            core::ptr::addr_of_mut!(szNameBuffer) as *mut WCHAR,
            RUSTCONV_WSTR_181895D51D74.as_ptr(),
            ulValue,
        );

        hKey = core::ptr::null_mut();
        lError = RegOpenKeyEx(
            hJobsKey,
            core::ptr::addr_of_mut!(szNameBuffer) as *mut WCHAR as LPCWSTR,
            0,
            KEY_READ,
            core::ptr::addr_of_mut!(hKey),
        );
        if lError != ERROR_SUCCESS as LONG {
            wcscpy(pszJobName, core::ptr::addr_of_mut!(szNameBuffer) as *mut WCHAR);
            return;
        }

        RegCloseKey(hKey);
    }
}
#[no_mangle]
pub unsafe extern "C" fn SaveJob(pJob: PJOB) -> LONG {
    let mut Schedule: SCHEDULE = core::mem::zeroed();
    let mut hJobsKey: HKEY = core::ptr::null_mut::<HKEY__>();
    let mut hJobKey: HKEY = core::ptr::null_mut::<HKEY__>();
    let mut lError: LONG;

    'done: {
        lError = RegCreateKeyExW(
            HKEY_LOCAL_MACHINE,
            RUSTCONV_WSTR_5A2E2E7B1604.as_ptr(),
            0,
            core::ptr::null_mut::<WCHAR>(),
            REG_OPTION_NON_VOLATILE,
            KEY_WRITE,
            core::ptr::null_mut::<_SECURITY_ATTRIBUTES>(),
            &raw mut hJobsKey,
            core::ptr::null_mut::<DWORD>(),
        );
        if lError != ERROR_SUCCESS as LONG {
            break 'done;
        }

        GetJobName(hJobsKey, &raw mut (*pJob).Name as PWSTR);

        lError = RegCreateKeyExW(
            hJobsKey,
            &raw mut (*pJob).Name as PWSTR,
            0,
            core::ptr::null_mut::<WCHAR>(),
            REG_OPTION_NON_VOLATILE,
            KEY_WRITE,
            core::ptr::null_mut::<_SECURITY_ATTRIBUTES>(),
            &raw mut hJobKey,
            core::ptr::null_mut::<DWORD>(),
        );
        if lError != ERROR_SUCCESS as LONG {
            break 'done;
        }

        Schedule.JobTime = (*pJob).JobTime as DWORD;
        Schedule.DaysOfMonth = (*pJob).DaysOfMonth;
        Schedule.DaysOfWeek = (*pJob).DaysOfWeek;
        Schedule.Flags = (*pJob).Flags;

        lError = RegSetValueEx(
            hJobKey,
            RUSTCONV_WSTR_2F006FE06135.as_ptr(),
            0,
            REG_BINARY,
            &raw mut Schedule as PBYTE,
            core::mem::size_of::<SCHEDULE>() as DWORD,
        );
        if lError != ERROR_SUCCESS as LONG {
            break 'done;
        }

        lError = RegSetValueEx(
            hJobKey,
            RUSTCONV_WSTR_425FC060B288.as_ptr(),
            0,
            REG_SZ,
            &raw mut (*pJob).Command as *mut WCHAR as PBYTE,
            ((wcslen(&raw mut (*pJob).Command as *mut WCHAR) + 1) * core::mem::size_of::<WCHAR>()) as DWORD,
        );
        if lError != ERROR_SUCCESS as LONG {
            break 'done;
        }
    }

    if !hJobKey.is_null() {
        RegCloseKey(hJobKey);
    }

    if !hJobsKey.is_null() {
        RegCloseKey(hJobsKey);
    }

    return lError;
}
#[no_mangle]
pub unsafe extern "C" fn DeleteJob(pJob: PJOB) -> LONG {
    let mut hJobsKey: HKEY = core::ptr::null_mut();
    let mut lError: LONG = 0;

    'done: {
        lError = RegCreateKeyExW(
            HKEY_LOCAL_MACHINE,
            RUSTCONV_WSTR_5A2E2E7B1604.as_ptr(),
            0,
            core::ptr::null_mut(),
            REG_OPTION_NON_VOLATILE,
            KEY_WRITE,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(hJobsKey),
            core::ptr::null_mut(),
        );
        if lError != ERROR_SUCCESS as LONG {
            break 'done;
        }

        lError = RegDeleteKey(
            hJobsKey,
            core::ptr::addr_of_mut!((*pJob).Name) as *mut WCHAR as LPCWSTR,
        );
        if lError != ERROR_SUCCESS as LONG {
            break 'done;
        }
    }

    if !hJobsKey.is_null() {
        RegCloseKey(hJobsKey);
    }

    return lError;
}
#[no_mangle]
pub unsafe extern "C" fn LoadJobs() -> LONG {
    let mut Schedule: SCHEDULE = core::mem::zeroed();
    let mut szNameBuffer: [WCHAR; JOB_NAME_LENGTH as usize] = [0; JOB_NAME_LENGTH as usize];
    let mut dwNameLength: DWORD = 0;
    let mut dwIndex: DWORD = 0;
    let mut dwSize: DWORD = 0;
    let mut hJobsKey: HKEY = core::ptr::null_mut();
    let mut hJobKey: HKEY = core::ptr::null_mut();
    let mut pJob: PJOB = core::ptr::null_mut();
    let mut lError: LONG;

    'done: {
        lError = RegCreateKeyExW(
            HKEY_LOCAL_MACHINE as HKEY,
            RUSTCONV_WSTR_5A2E2E7B1604.as_ptr(),
            0,
            core::ptr::null_mut(),
            REG_OPTION_NON_VOLATILE,
            KEY_READ,
            core::ptr::null_mut(),
            &raw mut hJobsKey,
            core::ptr::null_mut(),
        );
        if lError != ERROR_SUCCESS as LONG {
            break 'done;
        }

        dwIndex = 0;
        while dwIndex < 1000 {
            dwNameLength = JOB_NAME_LENGTH;
            lError = RegEnumKeyEx(
                hJobsKey,
                dwIndex,
                &raw mut szNameBuffer as *mut WCHAR,
                &raw mut dwNameLength,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );
            if lError != ERROR_SUCCESS as LONG {
                lError = ERROR_SUCCESS as LONG;
                break;
            }

            lError = RegOpenKeyEx(
                hJobsKey,
                &raw mut szNameBuffer as *mut WCHAR,
                0,
                KEY_READ,
                &raw mut hJobKey,
            );
            if lError != ERROR_SUCCESS as LONG {
                break;
            }

            dwSize = core::mem::size_of::<SCHEDULE>() as DWORD;
            lError = RegQueryValueEx(
                hJobKey,
                RUSTCONV_WSTR_2F006FE06135.as_ptr(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                &raw mut Schedule as PBYTE,
                &raw mut dwSize,
            );
            if lError == ERROR_SUCCESS as LONG {
                dwSize = 0;
                RegQueryValueEx(
                    hJobKey,
                    RUSTCONV_WSTR_425FC060B288.as_ptr(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    &raw mut dwSize,
                );
                if dwSize != 0 {
                    /* Allocate a new job object */
                    pJob = HeapAlloc(
                        GetProcessHeap(),
                        HEAP_ZERO_MEMORY as DWORD,
                        ((core::mem::size_of::<JOB>())
                            .wrapping_add(dwSize as usize)
                            .wrapping_sub(core::mem::size_of::<WCHAR>())) as SIZE_T,
                    ) as PJOB;
                    if pJob.is_null() {
                        lError = ERROR_OUTOFMEMORY as LONG;
                        break;
                    }

                    lError = RegQueryValueEx(
                        hJobKey,
                        RUSTCONV_WSTR_425FC060B288.as_ptr(),
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                        (&raw mut (*pJob).Command) as *mut WCHAR as PBYTE,
                        &raw mut dwSize,
                    );
                    if lError != ERROR_SUCCESS as LONG {
                        break;
                    }

                    wcscpy(
                        (&raw mut (*pJob).Name) as *mut WCHAR,
                        &raw mut szNameBuffer as *mut WCHAR,
                    );
                    (*pJob).JobTime = Schedule.JobTime as _;
                    (*pJob).DaysOfMonth = Schedule.DaysOfMonth;
                    (*pJob).DaysOfWeek = Schedule.DaysOfWeek;
                    (*pJob).Flags = Schedule.Flags;

                    /* Acquire the job list lock exclusively */
                    RtlAcquireResourceExclusive(&raw mut JobListLock, TRUE as BOOLEAN);

                    /* Assign a new job ID */
                    let fresh_job_id = dwNextJobId;
                    dwNextJobId = dwNextJobId.wrapping_add(1);
                    (*pJob).JobId = fresh_job_id;
                    dwJobCount = dwJobCount.wrapping_add(1);

                    /* Append the new job to the job list */
                    InsertTailList(&raw mut JobListHead, &raw mut (*pJob).JobEntry);

                    /* Calculate the next start time */
                    CalculateNextStartTime(pJob);

                    /* Release the job list lock */
                    RtlReleaseResource(&raw mut JobListLock);

                    pJob = core::ptr::null_mut();
                }
            }

            RegCloseKey(hJobKey);
            hJobKey = core::ptr::null_mut();
            dwIndex = dwIndex.wrapping_add(1);
        }
    }

    if pJob != core::ptr::null_mut() {
        HeapFree(GetProcessHeap(), 0 as DWORD, pJob as PVOID);
    }

    if hJobKey != core::ptr::null_mut() {
        RegCloseKey(hJobKey);
    }

    if hJobsKey != core::ptr::null_mut() {
        RegCloseKey(hJobsKey);
    }

    return lError;
}

unsafe extern "C" fn DaysOfMonth(wMonth: WORD, wYear: WORD) -> WORD {
    if wMonth == 2 && wYear % 4 == 0 && wYear % 400 != 0 {
        return 29;
    }

    return wDaysArray[wMonth as usize];
}
#[no_mangle]
pub unsafe extern "C" fn CalculateNextStartTime(pJob: PJOB) {
    let mut CurrentSystemTime: SYSTEMTIME = core::mem::zeroed();
    let mut StartSystemTime: SYSTEMTIME = core::mem::zeroed();
    let mut StartFileTime: FILETIME = core::mem::zeroed();
    let mut wDaysOffset: WORD = 0;
    let mut wTempOffset: WORD = 0;
    let mut i: WORD = 0;
    let mut wJobDayOfWeek: WORD = 0;
    let mut wJobDayOfMonth: WORD = 0;
    let mut CurrentTimeMs: DWORD_PTR = 0;
    let mut bDaysOffsetValid: BOOL = 0;
    let mut LocalStartTime: ULARGE_INTEGER = core::mem::zeroed();

    GetLocalTime(core::ptr::addr_of_mut!(CurrentSystemTime));

    CurrentTimeMs = (CurrentSystemTime.wHour as DWORD_PTR) * 3600000
        + (CurrentSystemTime.wMinute as DWORD_PTR) * 60000;

    bDaysOffsetValid = FALSE as BOOL;
    wDaysOffset = 0;
    if ((*pJob).DaysOfWeek == 0) && ((*pJob).DaysOfMonth == 0) {
        if CurrentTimeMs >= (*pJob).JobTime {
            wDaysOffset = 1;
        }

        bDaysOffsetValid = TRUE as BOOL;
    } else {
        if (*pJob).DaysOfWeek != 0 {
            i = 0;
            while i < 7 {
                if ((((*pJob).DaysOfWeek as DWORD) & (1 << (i as DWORD))) != 0) {
                    /* Adjust the range */
                    wJobDayOfWeek = (i + 1) % 7;

                    /* Calculate the days offset */
                    if (CurrentSystemTime.wDayOfWeek > wJobDayOfWeek)
                        || ((CurrentSystemTime.wDayOfWeek == wJobDayOfWeek) && (CurrentTimeMs >= (*pJob).JobTime))
                    {
                        wTempOffset = 7 - CurrentSystemTime.wDayOfWeek + wJobDayOfWeek;
                    } else {
                        wTempOffset = wJobDayOfWeek - CurrentSystemTime.wDayOfWeek;
                    }

                    /* Use the smallest offset */
                    if bDaysOffsetValid == FALSE as BOOL {
                        wDaysOffset = wTempOffset;
                        bDaysOffsetValid = TRUE as BOOL;
                    } else {
                        if wTempOffset < wDaysOffset {
                            wDaysOffset = wTempOffset;
                        }
                    }
                }
                i += 1;
            }
        }

        if (*pJob).DaysOfMonth != 0 {
            i = 0;
            while i < 31 {
                if ((*pJob).DaysOfMonth & (1 << (i as DWORD))) != 0 {
                    /* Adjust the range */
                    wJobDayOfMonth = i + 1;

                    if (CurrentSystemTime.wDay > wJobDayOfMonth)
                        || ((CurrentSystemTime.wDay == wJobDayOfMonth) && (CurrentTimeMs >= (*pJob).JobTime))
                    {
                        wTempOffset = DaysOfMonth(CurrentSystemTime.wMonth, CurrentSystemTime.wYear)
                            - CurrentSystemTime.wDay
                            + wJobDayOfMonth;
                    } else {
                        wTempOffset = wJobDayOfMonth - CurrentSystemTime.wDay;
                    }

                    /* Use the smallest offset */
                    if bDaysOffsetValid == FALSE as BOOL {
                        wDaysOffset = wTempOffset;
                        bDaysOffsetValid = TRUE as BOOL;
                    } else {
                        if wTempOffset < wDaysOffset {
                            wDaysOffset = wTempOffset;
                        }
                    }
                }
                i += 1;
            }
        }
    }

    CopyMemory!(
        core::ptr::addr_of_mut!(StartSystemTime) as *mut core::ffi::c_void,
        core::ptr::addr_of_mut!(CurrentSystemTime) as *const core::ffi::c_void,
        core::mem::size_of::<SYSTEMTIME>() as usize,
    );

    StartSystemTime.wMilliseconds = 0;
    StartSystemTime.wSecond = 0;
    StartSystemTime.wHour = ((*pJob).JobTime / 3600000) as WORD;
    StartSystemTime.wMinute = ((((*pJob).JobTime % 3600000) / 60000)) as WORD;

    SystemTimeToFileTime(
        core::ptr::addr_of_mut!(StartSystemTime),
        core::ptr::addr_of_mut!(StartFileTime),
    );

    LocalStartTime.u.LowPart = StartFileTime.dwLowDateTime;
    LocalStartTime.u.HighPart = StartFileTime.dwHighDateTime;
    if (bDaysOffsetValid != 0) && (wDaysOffset != 0) {
        LocalStartTime.QuadPart += (wDaysOffset as ULONGLONG)
            * 24
            * 60
            * 60
            * 10000;
    }

    (*pJob).StartTime.dwLowDateTime = LocalStartTime.u.LowPart;
    (*pJob).StartTime.dwHighDateTime = LocalStartTime.u.HighPart;
}
