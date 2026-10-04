use ntdll_sys::nt::*;
use crate::prelude::*;
use core::ffi::c_long;

const FALSE: BOOL = 0;
const TRUE: BOOL = 1;
const STATUS_INTERNAL_ERROR: NTSTATUS = 0xC00000E5_u32 as i32;

// ---- file-scope preamble (translated) ----
#[no_mangle]
pub unsafe extern "system" fn IsTimeZoneRedirectionEnabled() -> BOOL {
    return ((*BaseStaticServerData).TermsrvClientTimeZoneId != TIME_ZONE_ID_INVALID as u32) as i32;
}

#[no_mangle]
pub unsafe extern "system" fn FileTimeToDosDateTime(
    lpFileTime: *const FILETIME,
    lpFatDate: LPWORD,
    lpFatTime: LPWORD,
) -> BOOL {
    let mut FileTime: LARGE_INTEGER = core::mem::zeroed();
    let mut TimeFields: TIME_FIELDS = TIME_FIELDS {
        Year: 0,
        Month: 0,
        Day: 0,
        Hour: 0,
        Minute: 0,
        Second: 0,
        Milliseconds: 0,
        Weekday: 0,
    };

    FileTime.u.HighPart = (*lpFileTime).dwHighDateTime as LONG;
    FileTime.u.LowPart = (*lpFileTime).dwLowDateTime;

    if FileTime.QuadPart < 0 {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }

    RtlTimeToTimeFields(&raw mut FileTime, &raw mut TimeFields);

    if (TimeFields.Year < 1980) || (TimeFields.Year > 2107) {
        BaseSetLastNTError(STATUS_INVALID_PARAMETER);
        return FALSE;
    }

    *lpFatDate = (TimeFields.Day
        | (TimeFields.Month << 5)
        | ((TimeFields.Year - 1980) << 9)) as WORD;

    *lpFatTime = ((TimeFields.Second >> 1)
        | (TimeFields.Minute << 5)
        | (TimeFields.Hour << 11)) as WORD;

    return TRUE;
}
#[no_mangle]
pub unsafe extern "system" fn DosDateTimeToFileTime(
    wFatDate: WORD,
    wFatTime: WORD,
    lpFileTime: LPFILETIME,
) -> BOOL {
    let mut TimeFields: TIME_FIELDS = core::mem::zeroed();
    let mut SystemTime: LARGE_INTEGER = core::mem::zeroed();

    TimeFields.Year = ((wFatDate as i32 >> 9) + 1980) as CSHORT;
    TimeFields.Month = ((wFatDate as i32 >> 5) & 0xF) as CSHORT;
    TimeFields.Day = ((wFatDate as i32) & 0x1F) as CSHORT;
    TimeFields.Hour = ((wFatTime as i32) >> 11) as CSHORT;
    TimeFields.Minute = ((wFatTime as i32 >> 5) & 0x3F) as CSHORT;
    TimeFields.Second = ((wFatTime as i32 & 0x1F) << 1) as CSHORT;
    TimeFields.Milliseconds = 0;

    if RtlTimeFieldsToTime(&mut TimeFields, &mut SystemTime) != 0 {
        (*lpFileTime).dwLowDateTime = unsafe { SystemTime.u.LowPart };
        (*lpFileTime).dwHighDateTime = unsafe { SystemTime.u.HighPart as DWORD };
        return 1;
    }

    BaseSetLastNTError(STATUS_INVALID_PARAMETER);
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn CompareFileTime(
    lpFileTime1: *const FILETIME,
    lpFileTime2: *const FILETIME,
) -> LONG {
    let mut Time1: LARGE_INTEGER = core::mem::zeroed();
    let mut Time2: LARGE_INTEGER = core::mem::zeroed();
    let mut Diff: LARGE_INTEGER = core::mem::zeroed();

    Time1.u.LowPart = (*lpFileTime1).dwLowDateTime;
    Time2.u.LowPart = (*lpFileTime2).dwLowDateTime;
    Time1.u.HighPart = (*lpFileTime1).dwHighDateTime as LONG;
    Time2.u.HighPart = (*lpFileTime2).dwHighDateTime as LONG;

    Diff.QuadPart = Time1.QuadPart - Time2.QuadPart;

    if Diff.u.HighPart < 0 {
        return -1;
    }
    if Diff.QuadPart == 0 {
        return 0;
    }
    return 1;
}

/// Volatile seqlock capture of a KSYSTEM_TIME (the SharedUserData time fields
/// are C-`volatile`). Plain loads get hoisted out of the retry loop by LLVM:
/// entered mid-update, the condition never re-evaluates and the function spins
/// forever (caught as arm64 `b .` spins by scan_ud2's driver sweep, 2026-07-18).
unsafe fn read_ksystem_time(p: *const KSYSTEM_TIME) -> LARGE_INTEGER {
    let mut t: LARGE_INTEGER = core::mem::zeroed();
    loop {
        let high1 = core::ptr::read_volatile(core::ptr::addr_of!((*p).High1Time));
        let low = core::ptr::read_volatile(core::ptr::addr_of!((*p).LowPart));
        let high2 = core::ptr::read_volatile(core::ptr::addr_of!((*p).High2Time));
        if high1 == high2 {
            t.u.HighPart = high1;
            t.u.LowPart = low;
            return t;
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn GetSystemTimeAsFileTime(lpFileTime: PFILETIME) {
    let mut SystemTime: LARGE_INTEGER = core::mem::zeroed();

    SystemTime = read_ksystem_time(core::ptr::addr_of!((*SharedUserData).SystemTime));

    (*lpFileTime).dwLowDateTime = SystemTime.u.LowPart;
    (*lpFileTime).dwHighDateTime = SystemTime.u.HighPart as DWORD;
}

#[no_mangle]
pub unsafe extern "system" fn GetSystemTimePreciseAsFileTime(
    lpFileTime: PFILETIME,
) {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED as u32);
    if DbgPrint(
        b"(%s:%d) %s() is unimplemented!()\n\0" as *const u8 as *const i8,
        b"dll/win32/kernel32/client/time.c\0"
            as *const u8 as *const i8,
        150i32,
        b"GetSystemTimePreciseAsFileTime\0" as *const u8 as *const i8,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const i8,
            b"dll/win32/kernel32/client/time.c\0"
                as *const u8 as *const i8,
            150i32,
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn SystemTimeToFileTime(
    lpSystemTime: *const SYSTEMTIME,
    lpFileTime: LPFILETIME,
) -> BOOL {
    let mut TimeFields: TIME_FIELDS = core::mem::zeroed();
    let mut liTime: LARGE_INTEGER = core::mem::zeroed();

    TimeFields.Year = (*lpSystemTime).wYear as CSHORT;
    TimeFields.Month = (*lpSystemTime).wMonth as CSHORT;
    TimeFields.Day = (*lpSystemTime).wDay as CSHORT;
    TimeFields.Hour = (*lpSystemTime).wHour as CSHORT;
    TimeFields.Minute = (*lpSystemTime).wMinute as CSHORT;
    TimeFields.Second = (*lpSystemTime).wSecond as CSHORT;
    TimeFields.Milliseconds = (*lpSystemTime).wMilliseconds as CSHORT;

    if RtlTimeFieldsToTime(&mut TimeFields, &mut liTime) != 0 {
        (*lpFileTime).dwLowDateTime = liTime.u.LowPart;
        (*lpFileTime).dwHighDateTime = liTime.u.HighPart as DWORD;
        return 1;
    }

    BaseSetLastNTError(STATUS_INVALID_PARAMETER);
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn FileTimeToSystemTime(
    lpFileTime: *const FILETIME,
    lpSystemTime: LPSYSTEMTIME,
) -> BOOL {
    let mut TimeFields: TIME_FIELDS = core::mem::zeroed();
    let mut liTime: LARGE_INTEGER = core::mem::zeroed();

    liTime.u.LowPart = (*lpFileTime).dwLowDateTime;
    liTime.u.HighPart = (*lpFileTime).dwHighDateTime as LONG;
    if liTime.QuadPart < 0i64 {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return FALSE;
    }

    RtlTimeToTimeFields(&raw mut liTime, &raw mut TimeFields);

    (*lpSystemTime).wYear = TimeFields.Year as WORD;
    (*lpSystemTime).wMonth = TimeFields.Month as WORD;
    (*lpSystemTime).wDay = TimeFields.Day as WORD;
    (*lpSystemTime).wHour = TimeFields.Hour as WORD;
    (*lpSystemTime).wMinute = TimeFields.Minute as WORD;
    (*lpSystemTime).wSecond = TimeFields.Second as WORD;
    (*lpSystemTime).wMilliseconds = TimeFields.Milliseconds as WORD;
    (*lpSystemTime).wDayOfWeek = TimeFields.Weekday as WORD;

    return TRUE;
}

#[no_mangle]
pub unsafe extern "system" fn FileTimeToLocalFileTime(
    lpFileTime: *const FILETIME,
    lpLocalFileTime: LPFILETIME,
) -> BOOL {
    let mut TimeZoneBias: LARGE_INTEGER = core::mem::zeroed();
    let mut FileTime: LARGE_INTEGER = core::mem::zeroed();
    let mut TimePtr: *mut KSYSTEM_TIME = core::ptr::null_mut();

    TimePtr = if IsTimeZoneRedirectionEnabled() != 0 {
        core::ptr::addr_of_mut!((*BaseStaticServerData).ktTermsrvClientBias)
    } else {
        core::ptr::addr_of_mut!((*SharedUserData).TimeZoneBias)
    };

    TimeZoneBias = read_ksystem_time(TimePtr);

    FileTime.u.LowPart = (*lpFileTime).dwLowDateTime;
    FileTime.u.HighPart = (*lpFileTime).dwHighDateTime as i32;
    FileTime.QuadPart -= TimeZoneBias.QuadPart as i64;

    (*lpLocalFileTime).dwLowDateTime = FileTime.u.LowPart;
    (*lpLocalFileTime).dwHighDateTime = FileTime.u.HighPart as u32;

    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn LocalFileTimeToFileTime(
    lpLocalFileTime: *const FILETIME,
    lpFileTime: LPFILETIME,
) -> BOOL {
    let mut TimeZoneBias: LARGE_INTEGER = core::mem::zeroed();
    let mut FileTime: LARGE_INTEGER = core::mem::zeroed();
    let mut TimePtr: *mut KSYSTEM_TIME = core::ptr::null_mut();

    TimePtr = if IsTimeZoneRedirectionEnabled() != 0 {
        &raw mut (*BaseStaticServerData).ktTermsrvClientBias
    } else {
        &raw mut (*SharedUserData).TimeZoneBias
    };

    TimeZoneBias = read_ksystem_time(TimePtr);

    FileTime.u.LowPart = (*lpLocalFileTime).dwLowDateTime;
    FileTime.u.HighPart = (*lpLocalFileTime).dwHighDateTime as LONG;

    FileTime.QuadPart += TimeZoneBias.QuadPart;

    (*lpFileTime).dwLowDateTime = FileTime.u.LowPart;
    (*lpFileTime).dwHighDateTime = FileTime.u.HighPart as DWORD;

    return TRUE;
}
#[no_mangle]
pub unsafe extern "system" fn GetLocalTime(lpSystemTime: LPSYSTEMTIME) {
    let mut SystemTime: LARGE_INTEGER = core::mem::zeroed();
    let mut TimeZoneBias: LARGE_INTEGER = core::mem::zeroed();
    let mut TimeFields: TIME_FIELDS = core::mem::zeroed();
    let mut TimePtr: *mut KSYSTEM_TIME = core::ptr::null_mut();

    SystemTime = read_ksystem_time(core::ptr::addr_of!((*SharedUserData).SystemTime));

    TimePtr = if IsTimeZoneRedirectionEnabled() != 0 {
        &raw mut (*BaseStaticServerData).ktTermsrvClientBias
    } else {
        &raw mut (*SharedUserData).TimeZoneBias
    };

    TimeZoneBias = read_ksystem_time(TimePtr);

    SystemTime.QuadPart -= TimeZoneBias.QuadPart;
    RtlTimeToTimeFields(&raw mut SystemTime, &raw mut TimeFields);

    (*lpSystemTime).wYear = TimeFields.Year as u16;
    (*lpSystemTime).wMonth = TimeFields.Month as u16;
    (*lpSystemTime).wDay = TimeFields.Day as u16;
    (*lpSystemTime).wHour = TimeFields.Hour as u16;
    (*lpSystemTime).wMinute = TimeFields.Minute as u16;
    (*lpSystemTime).wSecond = TimeFields.Second as u16;
    (*lpSystemTime).wMilliseconds = TimeFields.Milliseconds as u16;
    (*lpSystemTime).wDayOfWeek = TimeFields.Weekday as u16;
}

#[no_mangle]
pub unsafe extern "system" fn GetSystemTime(lpSystemTime: LPSYSTEMTIME) {
    let mut SystemTime: LARGE_INTEGER = core::mem::zeroed();
    let mut TimeFields: TIME_FIELDS = core::mem::zeroed();

    SystemTime = read_ksystem_time(core::ptr::addr_of!((*SharedUserData).SystemTime));

    RtlTimeToTimeFields(&raw mut SystemTime, &raw mut TimeFields);

    (*lpSystemTime).wYear = TimeFields.Year as u16;
    (*lpSystemTime).wMonth = TimeFields.Month as u16;
    (*lpSystemTime).wDay = TimeFields.Day as u16;
    (*lpSystemTime).wHour = TimeFields.Hour as u16;
    (*lpSystemTime).wMinute = TimeFields.Minute as u16;
    (*lpSystemTime).wSecond = TimeFields.Second as u16;
    (*lpSystemTime).wMilliseconds = TimeFields.Milliseconds as u16;
    (*lpSystemTime).wDayOfWeek = TimeFields.Weekday as u16;
}

#[no_mangle]
pub unsafe extern "system" fn SetLocalTime(lpSystemTime: *const SYSTEMTIME) -> BOOL {
    let mut NewSystemTime: LARGE_INTEGER = core::mem::zeroed();
    let mut TimeZoneBias: LARGE_INTEGER = core::mem::zeroed();
    let mut Status: NTSTATUS = 0;
    let mut Privilege: ULONG = SE_SYSTEMTIME_PRIVILEGE as ULONG;
    let mut TimeFields: TIME_FIELDS = TIME_FIELDS {
        Year: 0,
        Month: 0,
        Day: 0,
        Hour: 0,
        Minute: 0,
        Second: 0,
        Milliseconds: 0,
        Weekday: 0,
    };
    let mut State: PVOID = core::ptr::null_mut();
    let mut TimePtr: *mut KSYSTEM_TIME = core::ptr::null_mut();

    TimePtr = if IsTimeZoneRedirectionEnabled() != 0 {
        core::ptr::addr_of_mut!((*BaseStaticServerData).ktTermsrvClientBias) as *mut KSYSTEM_TIME
    } else {
        core::ptr::addr_of_mut!((*SharedUserData).TimeZoneBias)
    };

    TimeZoneBias = read_ksystem_time(TimePtr);

    TimeFields.Year = (*lpSystemTime).wYear as CSHORT;
    TimeFields.Month = (*lpSystemTime).wMonth as CSHORT;
    TimeFields.Day = (*lpSystemTime).wDay as CSHORT;
    TimeFields.Hour = (*lpSystemTime).wHour as CSHORT;
    TimeFields.Minute = (*lpSystemTime).wMinute as CSHORT;
    TimeFields.Second = (*lpSystemTime).wSecond as CSHORT;
    TimeFields.Milliseconds = (*lpSystemTime).wMilliseconds as CSHORT;

    if RtlTimeFieldsToTime(core::ptr::addr_of_mut!(TimeFields), core::ptr::addr_of_mut!(NewSystemTime)) == 0 {
        BaseSetLastNTError(STATUS_INVALID_PARAMETER);
        return FALSE;
    }

    NewSystemTime.QuadPart += TimeZoneBias.QuadPart as i64;

    Status = RtlAcquirePrivilege(core::ptr::addr_of_mut!(Privilege), 1 as ULONG, 0 as ULONG, core::ptr::addr_of_mut!(State));
    if NT_SUCCESS(Status) {
        Status = NtSetSystemTime(core::ptr::addr_of_mut!(NewSystemTime), core::ptr::null_mut());
        RtlReleasePrivilege(State);
    }

    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return FALSE;
    }

    return TRUE;
}

#[no_mangle]
pub unsafe extern "system" fn SetSystemTime(lpSystemTime: *const SYSTEMTIME) -> BOOL {
    let mut NewSystemTime: LARGE_INTEGER = core::mem::zeroed();
    let mut Status: NTSTATUS = 0;
    let mut Privilege: ULONG = SE_SYSTEMTIME_PRIVILEGE as ULONG;
    let mut TimeFields: TIME_FIELDS = TIME_FIELDS {
        Year: 0,
        Month: 0,
        Day: 0,
        Hour: 0,
        Minute: 0,
        Second: 0,
        Milliseconds: 0,
        Weekday: 0,
    };
    let mut State: PVOID = core::ptr::null_mut();

    TimeFields.Year = (*lpSystemTime).wYear as CSHORT;
    TimeFields.Month = (*lpSystemTime).wMonth as CSHORT;
    TimeFields.Day = (*lpSystemTime).wDay as CSHORT;
    TimeFields.Hour = (*lpSystemTime).wHour as CSHORT;
    TimeFields.Minute = (*lpSystemTime).wMinute as CSHORT;
    TimeFields.Second = (*lpSystemTime).wSecond as CSHORT;
    TimeFields.Milliseconds = (*lpSystemTime).wMilliseconds as CSHORT;

    if RtlTimeFieldsToTime(&raw mut TimeFields, &raw mut NewSystemTime) == 0 {
        BaseSetLastNTError(STATUS_INVALID_PARAMETER);
        return FALSE;
    }

    Status = RtlAcquirePrivilege(&raw mut Privilege, 1 as ULONG, 0 as ULONG, &raw mut State);
    if Status >= 0 as c_long {
        Status = NtSetSystemTime(&raw mut NewSystemTime, core::ptr::null_mut());
        RtlReleasePrivilege(State);
    }

    if !(Status >= 0 as c_long) {
        BaseSetLastNTError(Status);
        return FALSE;
    }

    return TRUE;
}

#[no_mangle]
pub unsafe extern "system" fn GetTickCount() -> DWORD {
    let mut TickCount: ULARGE_INTEGER = core::mem::zeroed();
    let TickCountPtr = &raw const (*SharedUserData).__bindgen_anon_2.TickCount;

    // KiReadSystemTime: one atomic 64-bit read on _WIN64; on 32-bit the
    // High1Time/LowPart/High2Time retry loop. The translation took the _WIN64
    // read on every arch, which i386 splits into two loads that can tear.
    #[cfg(target_pointer_width = "64")]
    {
        TickCount.QuadPart = core::ptr::read_volatile(TickCountPtr as *const ULONG64) as ULONGLONG;
    }
    #[cfg(not(target_pointer_width = "64"))]
    {
        TickCount.QuadPart = read_ksystem_time(TickCountPtr).QuadPart as ULONGLONG;
    }

    return ((TickCount.__bindgen_anon_1.LowPart as u64)
        .wrapping_mul((*SharedUserData).TickCountMultiplier as u64)
        >> 24)
        .wrapping_add(
            (((TickCount.__bindgen_anon_1.HighPart as u32) << 8 & 0xFFFFFFFF) as u64)
                .wrapping_mul((*SharedUserData).TickCountMultiplier as u64),
        ) as DWORD;
}

#[no_mangle]
pub unsafe extern "system" fn GetSystemTimeAdjustment(
    lpTimeAdjustment: PDWORD,
    lpTimeIncrement: PDWORD,
    lpTimeAdjustmentDisabled: PBOOL,
) -> BOOL {
    let mut TimeInfo: SYSTEM_QUERY_TIME_ADJUST_INFORMATION = SYSTEM_QUERY_TIME_ADJUST_INFORMATION {
        TimeAdjustment: 0,
        TimeIncrement: 0,
        Enable: 0,
    };
    let mut Status: NTSTATUS = 0;

    Status = NtQuerySystemInformation(
        SystemTimeAdjustmentInformation,
        &mut TimeInfo as *mut SYSTEM_QUERY_TIME_ADJUST_INFORMATION as *mut core::ffi::c_void,
        core::mem::size_of::<SYSTEM_QUERY_TIME_ADJUST_INFORMATION>() as u32,
        core::ptr::null_mut(),
    );

    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return 0;
    }

    *lpTimeAdjustment = TimeInfo.TimeAdjustment as u32;
    *lpTimeIncrement = TimeInfo.TimeIncrement as u32;
    *lpTimeAdjustmentDisabled = TimeInfo.Enable as i32;

    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn SetSystemTimeAdjustment(
    dwTimeAdjustment: DWORD,
    bTimeAdjustmentDisabled: BOOL,
) -> BOOL {
    let mut Status: NTSTATUS;
    let mut TimeInfo: SYSTEM_SET_TIME_ADJUST_INFORMATION = core::mem::zeroed();

    TimeInfo.TimeAdjustment = dwTimeAdjustment;
    TimeInfo.Enable = bTimeAdjustmentDisabled as BOOLEAN;

    Status = NtSetSystemInformation(
        SystemTimeAdjustmentInformation,
        &raw mut TimeInfo as *mut core::ffi::c_void,
        core::mem::size_of::<SYSTEM_SET_TIME_ADJUST_INFORMATION>() as ULONG,
    );

    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return FALSE;
    }

    return TRUE;
}

#[no_mangle]
pub unsafe extern "system" fn GetSystemTimes(
    lpIdleTime: LPFILETIME,
    lpKernelTime: LPFILETIME,
    lpUserTime: LPFILETIME,
) -> BOOL {
    let mut ProcPerfInfo: PSYSTEM_PROCESSOR_PERFORMANCE_INFORMATION = core::ptr::null_mut();
    let mut TotalUserTime: LARGE_INTEGER = LARGE_INTEGER { QuadPart: 0 };
    let mut TotalKernTime: LARGE_INTEGER = LARGE_INTEGER { QuadPart: 0 };
    let mut TotalIdleTime: LARGE_INTEGER = LARGE_INTEGER { QuadPart: 0 };
    let mut BufferSize: ULONG = 0;
    let mut ReturnLength: ULONG = 0;
    let mut i: CCHAR = 0;
    let mut Status: NTSTATUS = 0;

    TotalIdleTime.QuadPart = 0;
    TotalKernTime.QuadPart = TotalIdleTime.QuadPart;
    TotalUserTime.QuadPart = TotalKernTime.QuadPart;

    BufferSize = (core::mem::size_of::<SYSTEM_PROCESSOR_PERFORMANCE_INFORMATION>() as ULONG)
        .wrapping_mul((*BaseStaticServerData).SysInfo.NumberOfProcessors as ULONG);

    ProcPerfInfo = RtlAllocateHeap(
        RtlGetProcessHeap(),
        0,
        BufferSize as SIZE_T,
    ) as PSYSTEM_PROCESSOR_PERFORMANCE_INFORMATION;

    if ProcPerfInfo.is_null() {
        BaseSetLastNTError(STATUS_NO_MEMORY);
        return 0;
    }

    Status = NtQuerySystemInformation(
        SystemProcessorPerformanceInformation,
        ProcPerfInfo as PVOID,
        BufferSize,
        &mut ReturnLength as *mut ULONG,
    );

    if NT_SUCCESS(Status) && ReturnLength == BufferSize {
        if !lpIdleTime.is_null() {
            i = 0;
            while (i as i32) < (*BaseStaticServerData).SysInfo.NumberOfProcessors as i32 {
                TotalIdleTime.QuadPart += (*ProcPerfInfo.offset(i as isize)).IdleTime.QuadPart;
                i += 1;
            }
            (*lpIdleTime).dwLowDateTime = TotalIdleTime.__bindgen_anon_1.LowPart;
            (*lpIdleTime).dwHighDateTime = TotalIdleTime.__bindgen_anon_1.HighPart as DWORD;
        }

        if !lpKernelTime.is_null() {
            i = 0;
            while (i as i32) < (*BaseStaticServerData).SysInfo.NumberOfProcessors as i32 {
                TotalKernTime.QuadPart += (*ProcPerfInfo.offset(i as isize)).KernelTime.QuadPart;
                i += 1;
            }
            (*lpKernelTime).dwLowDateTime = TotalKernTime.__bindgen_anon_1.LowPart;
            (*lpKernelTime).dwHighDateTime = TotalKernTime.__bindgen_anon_1.HighPart as DWORD;
        }

        if !lpUserTime.is_null() {
            i = 0;
            while (i as i32) < (*BaseStaticServerData).SysInfo.NumberOfProcessors as i32 {
                TotalUserTime.QuadPart += (*ProcPerfInfo.offset(i as isize)).UserTime.QuadPart;
                i += 1;
            }
            (*lpUserTime).dwLowDateTime = TotalUserTime.__bindgen_anon_1.LowPart;
            (*lpUserTime).dwHighDateTime = TotalUserTime.__bindgen_anon_1.HighPart as DWORD;
        }
    } else if NT_SUCCESS(Status) {
        Status = STATUS_INTERNAL_ERROR;
    }

    RtlFreeHeap(
        RtlGetProcessHeap(),
        0,
        ProcPerfInfo as PVOID,
    );

    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return 0;
    }

    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn SetClientTimeZoneInformation(
    lpTimeZoneInformation: *const TIME_ZONE_INFORMATION,
) -> BOOL {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED as DWORD);
    if DbgPrint(
        b"(%s:%d) %s() is unimplemented!()\n\0".as_ptr() as PCSTR,
        b"dll/win32/kernel32/client/time.c\0"
            .as_ptr() as *const ::core::ffi::c_char,
        624 as ::core::ffi::c_int,
        b"SetClientTimeZoneInformation\0".as_ptr() as *const ::core::ffi::c_char,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
            b"dll/win32/kernel32/client/time.c\0"
                .as_ptr() as *const ::core::ffi::c_char,
            624 as ::core::ffi::c_int,
        );
    }
    return 0 as BOOL;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS system libraries
//  * FILE:            dll/win32/kernel32/client/time.c
//  * PURPOSE:         Time conversion functions
//  * PROGRAMMER:      Ariadne
//  *                  DOSDATE and DOSTIME structures from Onno Hovers
//  * UPDATE HISTORY:
//  *                  Created 19/01/99
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
// BOOL
// WINAPI
// IsTimeZoneRedirectionEnabled(VOID)
// {
//     /* Return if a TS Timezone ID is active */
//     return (BaseStaticServerData->TermsrvClientTimeZoneId != TIME_ZONE_ID_INVALID);
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// FileTimeToDosDateTime(IN CONST FILETIME *lpFileTime,
//                       OUT LPWORD lpFatDate,
//                       OUT LPWORD lpFatTime)
// {
//     LARGE_INTEGER FileTime;
//     TIME_FIELDS TimeFields;
//
//     FileTime.HighPart = lpFileTime->dwHighDateTime;
//     FileTime.LowPart = lpFileTime->dwLowDateTime;
//
//     if (FileTime.QuadPart < 0)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     RtlTimeToTimeFields(&FileTime, &TimeFields);
//     if ((TimeFields.Year < 1980) || (TimeFields.Year > 2107))
//     {
//         BaseSetLastNTError(STATUS_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     *lpFatDate = (TimeFields.Day) |
//                  (TimeFields.Month << 5) |
//                  ((TimeFields.Year - 1980) << 9);
//     *lpFatTime = (TimeFields.Second >> 1) |
//                  (TimeFields.Minute << 5) |
//                  (TimeFields.Hour << 11);
//
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// DosDateTimeToFileTime(IN WORD wFatDate,
//                       IN WORD wFatTime,
//                       OUT LPFILETIME lpFileTime)
// {
//     TIME_FIELDS TimeFields;
//     LARGE_INTEGER SystemTime;
//
//     TimeFields.Year = (wFatDate >> 9) + 1980;
//     TimeFields.Month = (wFatDate >> 5) & 0xF;
//     TimeFields.Day = (wFatDate & 0x1F);
//     TimeFields.Hour = (wFatTime >> 11);
//     TimeFields.Minute = (wFatTime >> 5) & 0x3F;
//     TimeFields.Second = (wFatTime & 0x1F) << 1;
//     TimeFields.Milliseconds = 0;
//
//     if (RtlTimeFieldsToTime(&TimeFields, &SystemTime))
//     {
//         lpFileTime->dwLowDateTime = SystemTime.LowPart;
//         lpFileTime->dwHighDateTime = SystemTime.HighPart;
//         return TRUE;
//     }
//
//     BaseSetLastNTError(STATUS_INVALID_PARAMETER);
//     return FALSE;
// }
//
// /*
//  * @implemented
//  */
// LONG
// WINAPI
// CompareFileTime(IN CONST FILETIME *lpFileTime1,
//                 IN CONST FILETIME *lpFileTime2)
// {
//     LARGE_INTEGER Time1, Time2, Diff;
//
//     Time1.LowPart = lpFileTime1->dwLowDateTime;
//     Time2.LowPart = lpFileTime2->dwLowDateTime;
//     Time1.HighPart = lpFileTime1->dwHighDateTime;
//     Time2.HighPart = lpFileTime2->dwHighDateTime;
//
//     Diff.QuadPart = Time1.QuadPart - Time2.QuadPart;
//
//     if (Diff.HighPart < 0) return -1;
//     if (Diff.QuadPart == 0) return 0;
//     return 1;
// }
//
// /*
//  * @implemented
//  */
// VOID
// WINAPI
// GetSystemTimeAsFileTime(OUT PFILETIME lpFileTime)
// {
//     LARGE_INTEGER SystemTime;
//
//     do
//     {
//         SystemTime.HighPart = SharedUserData->SystemTime.High1Time;
//         SystemTime.LowPart = SharedUserData->SystemTime.LowPart;
//     }
//     while (SystemTime.HighPart != SharedUserData->SystemTime.High2Time);
//
//     lpFileTime->dwLowDateTime = SystemTime.LowPart;
//     lpFileTime->dwHighDateTime = SystemTime.HighPart;
// }
//
// /*
//  * @unimplemented
//  */
// VOID
// WINAPI
// GetSystemTimePreciseAsFileTime(OUT PFILETIME lpFileTime)
// {
//     STUB;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// SystemTimeToFileTime(IN CONST SYSTEMTIME *lpSystemTime,
//                      OUT LPFILETIME lpFileTime)
// {
//     TIME_FIELDS TimeFields;
//     LARGE_INTEGER liTime;
//
//     TimeFields.Year = lpSystemTime->wYear;
//     TimeFields.Month = lpSystemTime->wMonth;
//     TimeFields.Day = lpSystemTime->wDay;
//     TimeFields.Hour = lpSystemTime->wHour;
//     TimeFields.Minute = lpSystemTime->wMinute;
//     TimeFields.Second = lpSystemTime->wSecond;
//     TimeFields.Milliseconds = lpSystemTime->wMilliseconds;
//
//     if (RtlTimeFieldsToTime(&TimeFields, &liTime))
//     {
//         lpFileTime->dwLowDateTime = liTime.u.LowPart;
//         lpFileTime->dwHighDateTime = liTime.u.HighPart;
//         return TRUE;
//     }
//
//     BaseSetLastNTError(STATUS_INVALID_PARAMETER);
//     return FALSE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// FileTimeToSystemTime(IN CONST FILETIME *lpFileTime,
//                      OUT LPSYSTEMTIME lpSystemTime)
// {
//     TIME_FIELDS TimeFields;
//     LARGE_INTEGER liTime;
//
//     liTime.u.LowPart = lpFileTime->dwLowDateTime;
//     liTime.u.HighPart = lpFileTime->dwHighDateTime;
//     if (liTime.QuadPart < 0)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     RtlTimeToTimeFields(&liTime, &TimeFields);
//
//     lpSystemTime->wYear = TimeFields.Year;
//     lpSystemTime->wMonth = TimeFields.Month;
//     lpSystemTime->wDay = TimeFields.Day;
//     lpSystemTime->wHour = TimeFields.Hour;
//     lpSystemTime->wMinute = TimeFields.Minute;
//     lpSystemTime->wSecond = TimeFields.Second;
//     lpSystemTime->wMilliseconds = TimeFields.Milliseconds;
//     lpSystemTime->wDayOfWeek = TimeFields.Weekday;
//
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// FileTimeToLocalFileTime(IN CONST FILETIME *lpFileTime,
//                         OUT LPFILETIME lpLocalFileTime)
// {
//     LARGE_INTEGER TimeZoneBias, FileTime;
//     volatile KSYSTEM_TIME *TimePtr;
//
//     TimePtr = IsTimeZoneRedirectionEnabled() ?
//               &BaseStaticServerData->ktTermsrvClientBias :
//               &SharedUserData->TimeZoneBias;
//     do
//     {
//         TimeZoneBias.HighPart = TimePtr->High1Time;
//         TimeZoneBias.LowPart = TimePtr->LowPart;
//     }
//     while (TimeZoneBias.HighPart != TimePtr->High2Time);
//
//     FileTime.LowPart = lpFileTime->dwLowDateTime;
//     FileTime.HighPart = lpFileTime->dwHighDateTime;
//
//     FileTime.QuadPart -= TimeZoneBias.QuadPart;
//
//     lpLocalFileTime->dwLowDateTime = FileTime.LowPart;
//     lpLocalFileTime->dwHighDateTime = FileTime.HighPart;
//
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// LocalFileTimeToFileTime(IN CONST FILETIME *lpLocalFileTime,
//                         OUT LPFILETIME lpFileTime)
// {
//     LARGE_INTEGER TimeZoneBias, FileTime;
//     volatile KSYSTEM_TIME *TimePtr;
//
//     TimePtr = IsTimeZoneRedirectionEnabled() ?
//               &BaseStaticServerData->ktTermsrvClientBias :
//               &SharedUserData->TimeZoneBias;
//
//     do
//     {
//         TimeZoneBias.HighPart = TimePtr->High1Time;
//         TimeZoneBias.LowPart = TimePtr->LowPart;
//     }
//     while (TimeZoneBias.HighPart != TimePtr->High2Time);
//
//     FileTime.LowPart = lpLocalFileTime->dwLowDateTime;
//     FileTime.HighPart = lpLocalFileTime->dwHighDateTime;
//
//     FileTime.QuadPart += TimeZoneBias.QuadPart;
//
//     lpFileTime->dwLowDateTime = FileTime.LowPart;
//     lpFileTime->dwHighDateTime = FileTime.HighPart;
//
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// VOID
// WINAPI
// GetLocalTime(OUT LPSYSTEMTIME lpSystemTime)
// {
//     LARGE_INTEGER SystemTime, TimeZoneBias;
//     TIME_FIELDS TimeFields;
//     volatile KSYSTEM_TIME *TimePtr;
//
//     do
//     {
//         SystemTime.HighPart = SharedUserData->SystemTime.High1Time;
//         SystemTime.LowPart = SharedUserData->SystemTime.LowPart;
//     }
//     while (SystemTime.HighPart != SharedUserData->SystemTime.High2Time);
//
//     TimePtr = IsTimeZoneRedirectionEnabled() ?
//               &BaseStaticServerData->ktTermsrvClientBias :
//               &SharedUserData->TimeZoneBias;
//     do
//     {
//         TimeZoneBias.HighPart = TimePtr->High1Time;
//         TimeZoneBias.LowPart = TimePtr->LowPart;
//     }
//     while (TimeZoneBias.HighPart != TimePtr->High2Time);
//
//     SystemTime.QuadPart -= TimeZoneBias.QuadPart;
//     RtlTimeToTimeFields(&SystemTime, &TimeFields);
//
//     lpSystemTime->wYear = TimeFields.Year;
//     lpSystemTime->wMonth = TimeFields.Month;
//     lpSystemTime->wDay = TimeFields.Day;
//     lpSystemTime->wHour = TimeFields.Hour;
//     lpSystemTime->wMinute = TimeFields.Minute;
//     lpSystemTime->wSecond = TimeFields.Second;
//     lpSystemTime->wMilliseconds = TimeFields.Milliseconds;
//     lpSystemTime->wDayOfWeek = TimeFields.Weekday;
// }
//
// /*
//  * @implemented
//  */
// VOID
// WINAPI
// GetSystemTime(OUT LPSYSTEMTIME lpSystemTime)
// {
//     LARGE_INTEGER SystemTime;
//     TIME_FIELDS TimeFields;
//
//     do
//     {
//         SystemTime.HighPart = SharedUserData->SystemTime.High1Time;
//         SystemTime.LowPart = SharedUserData->SystemTime.LowPart;
//     }
//     while (SystemTime.HighPart != SharedUserData->SystemTime.High2Time);
//
//     RtlTimeToTimeFields(&SystemTime, &TimeFields);
//
//     lpSystemTime->wYear = TimeFields.Year;
//     lpSystemTime->wMonth = TimeFields.Month;
//     lpSystemTime->wDay = TimeFields.Day;
//     lpSystemTime->wHour = TimeFields.Hour;
//     lpSystemTime->wMinute = TimeFields.Minute;
//     lpSystemTime->wSecond = TimeFields.Second;
//     lpSystemTime->wMilliseconds = TimeFields.Milliseconds;
//     lpSystemTime->wDayOfWeek = TimeFields.Weekday;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// SetLocalTime(IN CONST SYSTEMTIME *lpSystemTime)
// {
//     LARGE_INTEGER NewSystemTime, TimeZoneBias;
//     NTSTATUS Status;
//     ULONG Privilege = SE_SYSTEMTIME_PRIVILEGE;
//     TIME_FIELDS TimeFields;
//     PVOID State;
//     volatile KSYSTEM_TIME *TimePtr;
//
//     TimePtr = IsTimeZoneRedirectionEnabled() ?
//               &BaseStaticServerData->ktTermsrvClientBias :
//               &SharedUserData->TimeZoneBias;
//     do
//     {
//         TimeZoneBias.HighPart = TimePtr->High1Time;
//         TimeZoneBias.LowPart = TimePtr->LowPart;
//     }
//     while (TimeZoneBias.HighPart != TimePtr->High2Time);
//
//     TimeFields.Year = lpSystemTime->wYear;
//     TimeFields.Month = lpSystemTime->wMonth;
//     TimeFields.Day = lpSystemTime->wDay;
//     TimeFields.Hour = lpSystemTime->wHour;
//     TimeFields.Minute = lpSystemTime->wMinute;
//     TimeFields.Second = lpSystemTime->wSecond;
//     TimeFields.Milliseconds = lpSystemTime->wMilliseconds;
//
//     if (!RtlTimeFieldsToTime(&TimeFields, &NewSystemTime))
//     {
//         BaseSetLastNTError(STATUS_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     NewSystemTime.QuadPart += TimeZoneBias.QuadPart;
//
//     Status = RtlAcquirePrivilege(&Privilege, 1, 0, &State);
//     if (NT_SUCCESS(Status))
//     {
//         Status = NtSetSystemTime(&NewSystemTime, NULL);
//         RtlReleasePrivilege(State);
//     }
//
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
// SetSystemTime(IN CONST SYSTEMTIME *lpSystemTime)
// {
//     LARGE_INTEGER NewSystemTime;
//     NTSTATUS Status;
//     ULONG Privilege = SE_SYSTEMTIME_PRIVILEGE;
//     TIME_FIELDS TimeFields;
//     PVOID State;
//
//     TimeFields.Year = lpSystemTime->wYear;
//     TimeFields.Month = lpSystemTime->wMonth;
//     TimeFields.Day = lpSystemTime->wDay;
//     TimeFields.Hour = lpSystemTime->wHour;
//     TimeFields.Minute = lpSystemTime->wMinute;
//     TimeFields.Second = lpSystemTime->wSecond;
//     TimeFields.Milliseconds = lpSystemTime->wMilliseconds;
//
//     if (!RtlTimeFieldsToTime(&TimeFields, &NewSystemTime))
//     {
//         BaseSetLastNTError(STATUS_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     Status = RtlAcquirePrivilege(&Privilege, 1, 0, &State);
//     if (NT_SUCCESS(Status))
//     {
//         Status = NtSetSystemTime(&NewSystemTime, NULL);
//         RtlReleasePrivilege(State);
//     }
//
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
// DWORD
// WINAPI
// GetTickCount(VOID)
// {
//     ULARGE_INTEGER TickCount;
//
// #ifdef _WIN64
//     TickCount.QuadPart = *((volatile ULONG64*)&SharedUserData->TickCount);
// #else
//     while (TRUE)
//     {
//         TickCount.HighPart = (ULONG)SharedUserData->TickCount.High1Time;
//         TickCount.LowPart = SharedUserData->TickCount.LowPart;
//
//         if (TickCount.HighPart == (ULONG)SharedUserData->TickCount.High2Time)
//             break;
//
//         YieldProcessor();
//     }
// #endif
//
//     return (ULONG)((UInt32x32To64(TickCount.LowPart,
//                                   SharedUserData->TickCountMultiplier) >> 24) +
//                     UInt32x32To64((TickCount.HighPart << 8) & 0xFFFFFFFF,
//                                   SharedUserData->TickCountMultiplier));
//
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// GetSystemTimeAdjustment(OUT PDWORD lpTimeAdjustment,
//                         OUT PDWORD lpTimeIncrement,
//                         OUT PBOOL lpTimeAdjustmentDisabled)
// {
//     SYSTEM_QUERY_TIME_ADJUST_INFORMATION TimeInfo;
//     NTSTATUS Status;
//
//     Status = NtQuerySystemInformation(SystemTimeAdjustmentInformation,
//                                       &TimeInfo,
//                                       sizeof(TimeInfo),
//                                       NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     *lpTimeAdjustment = (DWORD)TimeInfo.TimeAdjustment;
//     *lpTimeIncrement = (DWORD)TimeInfo.TimeIncrement;
//     *lpTimeAdjustmentDisabled = (BOOL)TimeInfo.Enable;
//
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// SetSystemTimeAdjustment(IN DWORD dwTimeAdjustment,
//                         IN BOOL bTimeAdjustmentDisabled)
// {
//     NTSTATUS Status;
//     SYSTEM_SET_TIME_ADJUST_INFORMATION TimeInfo;
//
//     TimeInfo.TimeAdjustment = (ULONG)dwTimeAdjustment;
//     TimeInfo.Enable = (BOOLEAN)bTimeAdjustmentDisabled;
//
//     Status = NtSetSystemInformation(SystemTimeAdjustmentInformation,
//                                     &TimeInfo,
//                                     sizeof(TimeInfo));
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
// GetSystemTimes(OUT LPFILETIME lpIdleTime OPTIONAL,
//                OUT LPFILETIME lpKernelTime OPTIONAL,
//                OUT LPFILETIME lpUserTime OPTIONAL)
// {
//     PSYSTEM_PROCESSOR_PERFORMANCE_INFORMATION ProcPerfInfo;
//     LARGE_INTEGER TotalUserTime, TotalKernTime, TotalIdleTime;
//     ULONG BufferSize, ReturnLength;
//     CCHAR i;
//     NTSTATUS Status;
//
//     TotalUserTime.QuadPart = TotalKernTime.QuadPart = TotalIdleTime.QuadPart = 0;
//
//     BufferSize = sizeof(SYSTEM_PROCESSOR_PERFORMANCE_INFORMATION) *
//                  BaseStaticServerData->SysInfo.NumberOfProcessors;
//
//     ProcPerfInfo = RtlAllocateHeap(RtlGetProcessHeap(), 0, BufferSize);
//     if (!ProcPerfInfo)
//     {
//         BaseSetLastNTError(STATUS_NO_MEMORY);
//         return FALSE;
//     }
//
//     Status = NtQuerySystemInformation(SystemProcessorPerformanceInformation,
//                                       ProcPerfInfo,
//                                       BufferSize,
//                                       &ReturnLength);
//     if ((NT_SUCCESS(Status)) && (ReturnLength == BufferSize))
//     {
//         if (lpIdleTime)
//         {
//             for (i = 0; i < BaseStaticServerData->SysInfo.NumberOfProcessors; i++)
//             {
//                 TotalIdleTime.QuadPart += ProcPerfInfo[i].IdleTime.QuadPart;
//             }
//
//             lpIdleTime->dwLowDateTime = TotalIdleTime.LowPart;
//             lpIdleTime->dwHighDateTime = TotalIdleTime.HighPart;
//         }
//
//         if (lpKernelTime)
//         {
//             for (i = 0; i < BaseStaticServerData->SysInfo.NumberOfProcessors; i++)
//             {
//                 TotalKernTime.QuadPart += ProcPerfInfo[i].KernelTime.QuadPart;
//             }
//
//             lpKernelTime->dwLowDateTime = TotalKernTime.LowPart;
//             lpKernelTime->dwHighDateTime = TotalKernTime.HighPart;
//         }
//
//         if (lpUserTime)
//         {
//             for (i = 0; i < BaseStaticServerData->SysInfo.NumberOfProcessors; i++)
//             {
//                 TotalUserTime.QuadPart += ProcPerfInfo[i].UserTime.QuadPart;
//             }
//
//             lpUserTime->dwLowDateTime = TotalUserTime.LowPart;
//             lpUserTime->dwHighDateTime = TotalUserTime.HighPart;
//         }
//     }
//     else if (NT_SUCCESS(Status))
//     {
//          Status = STATUS_INTERNAL_ERROR;
//     }
//
//     RtlFreeHeap(RtlGetProcessHeap(), 0, ProcPerfInfo);
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
//  * @unimplemented
//  */
// BOOL
// WINAPI
// SetClientTimeZoneInformation(IN CONST TIME_ZONE_INFORMATION *lpTimeZoneInformation)
// {
//     STUB;
//     return 0;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
