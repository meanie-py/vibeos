use ntdll_sys::nt::*;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut MonthLengths: [[::core::ffi::c_int; 12]; 2] = [
    [
        31 as ::core::ffi::c_int,
        28 as ::core::ffi::c_int,
        31 as ::core::ffi::c_int,
        30 as ::core::ffi::c_int,
        31 as ::core::ffi::c_int,
        30 as ::core::ffi::c_int,
        31 as ::core::ffi::c_int,
        31 as ::core::ffi::c_int,
        30 as ::core::ffi::c_int,
        31 as ::core::ffi::c_int,
        30 as ::core::ffi::c_int,
        31 as ::core::ffi::c_int,
    ],
    [
        31 as ::core::ffi::c_int,
        29 as ::core::ffi::c_int,
        31 as ::core::ffi::c_int,
        30 as ::core::ffi::c_int,
        31 as ::core::ffi::c_int,
        30 as ::core::ffi::c_int,
        31 as ::core::ffi::c_int,
        31 as ::core::ffi::c_int,
        30 as ::core::ffi::c_int,
        31 as ::core::ffi::c_int,
        30 as ::core::ffi::c_int,
        31 as ::core::ffi::c_int,
    ],
];

pub const NDEBUG: () = ();
pub const TICKSPERMIN: i32 = 600000000;

pub unsafe fn LL2FILETIME(ll: u64, pft: *mut FILETIME) {
    (*pft).dwLowDateTime = ll as u32;
    (*pft).dwHighDateTime = (ll >> 32) as u32;
}

pub unsafe fn FILETIME2LL(pft: *const FILETIME) -> i64 {
    ((((*pft).dwHighDateTime as i64) << 32) + (*pft).dwLowDateTime as i64)
}
unsafe fn IsLeapYear(Year: i32) -> i32 {
    return if Year % 4 == 0 && (Year % 100 != 0 || Year % 400 == 0) {
        1
    } else {
        0
    };
}
unsafe fn TIME_DayLightCompareDate(
    date: *const SYSTEMTIME,
    compareDate: *const SYSTEMTIME,
) -> i32 {
    let mut limit_day: i32;
    let mut dayinsecs: i32;

    if (*date).wMonth < (*compareDate).wMonth {
        return -1;
    }

    if (*date).wMonth > (*compareDate).wMonth {
        return 1;
    }

    if (*compareDate).wYear == 0 {
        let mut First: WORD;
        let weekofmonth: i32 = (*compareDate).wDay as i32;
        First = ((6i32 + (*compareDate).wDayOfWeek as i32 - (*date).wDayOfWeek as i32 + (*date).wDay as i32) % 7 + 1) as WORD;
        limit_day = First as i32 + 7 * (weekofmonth - 1);
        if limit_day > MonthLengths[if (*date).wMonth == 2 && IsLeapYear((*date).wYear as i32) != 0 { 1 } else { 0 }][(*date).wMonth as usize - 1] {
            limit_day -= 7;
        }
    } else {
        limit_day = (*compareDate).wDay as i32;
    }

    limit_day = ((limit_day * 24 + (*compareDate).wHour as i32) * 60 + (*compareDate).wMinute as i32) * 60;
    dayinsecs = (((*date).wDay as i32 * 24 + (*date).wHour as i32) * 60 + (*date).wMinute as i32) * 60 + (*date).wSecond as i32;

    return if dayinsecs < limit_day {
        -1
    } else if dayinsecs > limit_day {
        1
    } else {
        0
    };
}
unsafe fn TIME_CompTimeZoneID(
    pTZinfo: *const TIME_ZONE_INFORMATION,
    lpFileTime: *mut FILETIME,
    islocal: BOOL,
) -> DWORD {
    let mut ret: i32 = 0;
    let mut year: i32 = 0;
    let mut beforeStandardDate: BOOL = 0;
    let mut afterDaylightDate: BOOL = 0;
    let mut retval: DWORD = TIME_ZONE_ID_INVALID as DWORD;
    let mut llTime: LONGLONG = 0;
    let mut SysTime: SYSTEMTIME = SYSTEMTIME {
        wYear: 0,
        wMonth: 0,
        wDayOfWeek: 0,
        wDay: 0,
        wHour: 0,
        wMinute: 0,
        wSecond: 0,
        wMilliseconds: 0,
    };
    let mut ftTemp: FILETIME = FILETIME {
        dwLowDateTime: 0,
        dwHighDateTime: 0,
    };

    if (*pTZinfo).DaylightDate.wMonth != 0 {
        if (*pTZinfo).StandardDate.wMonth == 0
            || ((*pTZinfo).StandardDate.wYear == 0
                && ((*pTZinfo).StandardDate.wDay < 1
                    || (*pTZinfo).StandardDate.wDay > 5
                    || (*pTZinfo).DaylightDate.wDay < 1
                    || (*pTZinfo).DaylightDate.wDay > 5))
        {
            SetLastError(ERROR_INVALID_PARAMETER as DWORD);
            return TIME_ZONE_ID_INVALID as DWORD;
        }

        if islocal == 0 {
            llTime = ((((*lpFileTime).dwHighDateTime as LONGLONG) << 32)
                + (*lpFileTime).dwLowDateTime as LONGLONG) as LONGLONG;
            llTime -= (*pTZinfo).Bias as LONGLONG * TICKSPERMIN as LONGLONG;
            ftTemp.dwLowDateTime = llTime as DWORD;
            ftTemp.dwHighDateTime = (llTime >> 32) as DWORD;
            lpFileTime = &raw mut ftTemp;
        }

        FileTimeToSystemTime(lpFileTime, &raw mut SysTime);
        year = SysTime.wYear as i32;

        if islocal == 0 {
            llTime -= (*pTZinfo).DaylightBias as LONGLONG * TICKSPERMIN as LONGLONG;
            ftTemp.dwLowDateTime = llTime as DWORD;
            ftTemp.dwHighDateTime = (llTime >> 32) as DWORD;
            FileTimeToSystemTime(lpFileTime, &raw mut SysTime);
        }

        if year == SysTime.wYear as i32 {
            ret = TIME_DayLightCompareDate(&raw mut SysTime, &raw const (*pTZinfo).StandardDate);
            if ret == -2 {
                return TIME_ZONE_ID_INVALID as DWORD;
            }
            beforeStandardDate = (ret < 0) as BOOL;
        } else {
            beforeStandardDate = ((SysTime.wYear as i32) < year) as BOOL;
        }

        if islocal == 0 {
            llTime -= ((*pTZinfo).StandardBias - (*pTZinfo).DaylightBias) as LONGLONG * TICKSPERMIN as LONGLONG;
            ftTemp.dwLowDateTime = llTime as DWORD;
            ftTemp.dwHighDateTime = (llTime >> 32) as DWORD;
            FileTimeToSystemTime(lpFileTime, &raw mut SysTime);
        }

        if year == SysTime.wYear as i32 {
            ret = TIME_DayLightCompareDate(&raw mut SysTime, &raw const (*pTZinfo).DaylightDate);
            if ret == -2 {
                return TIME_ZONE_ID_INVALID as DWORD;
            }
            afterDaylightDate = (ret >= 0) as BOOL;
        } else {
            afterDaylightDate = (SysTime.wYear as i32 > year) as BOOL;
        }

        retval = TIME_ZONE_ID_STANDARD as DWORD;
        if (*pTZinfo).DaylightDate.wMonth < (*pTZinfo).StandardDate.wMonth {
            if beforeStandardDate != 0 && afterDaylightDate != 0 {
                retval = TIME_ZONE_ID_DAYLIGHT as DWORD;
            }
        } else if beforeStandardDate != 0 || afterDaylightDate != 0 {
            retval = TIME_ZONE_ID_DAYLIGHT as DWORD;
        }
    } else {
        retval = TIME_ZONE_ID_UNKNOWN as DWORD;
    }

    return retval;
}
unsafe fn TIME_ZoneID(pTzi: *const TIME_ZONE_INFORMATION) -> DWORD {
    let mut ftTime: FILETIME = FILETIME {
        dwLowDateTime: 0,
        dwHighDateTime: 0,
    };
    GetSystemTimeAsFileTime(&mut ftTime as *mut FILETIME);
    return TIME_CompTimeZoneID(pTzi, &mut ftTime as *mut FILETIME, 0i32);
}
unsafe fn TIME_GetTimezoneBias(
    pTZinfo: *const TIME_ZONE_INFORMATION,
    lpFileTime: *mut FILETIME,
    islocal: BOOL,
    pBias: *mut LONG,
) -> BOOL {
    let mut bias: LONG = (*pTZinfo).Bias;
    let tzid: DWORD = TIME_CompTimeZoneID(pTZinfo, lpFileTime, islocal);

    if tzid == TIME_ZONE_ID_INVALID {
        return FALSE;
    }
    if tzid == TIME_ZONE_ID_DAYLIGHT {
        bias += (*pTZinfo).DaylightBias;
    } else if tzid == TIME_ZONE_ID_STANDARD {
        bias += (*pTZinfo).StandardBias;
    }
    *pBias = bias;
    return TRUE;
}

#[no_mangle]
pub unsafe extern "system" fn GetTimeZoneInformation(
    lpTimeZoneInformation: LPTIME_ZONE_INFORMATION,
) -> DWORD {
    let mut TimeZoneInformation: RTL_TIME_ZONE_INFORMATION = core::mem::zeroed();
    let mut Status: NTSTATUS;

    Status = NtQuerySystemInformation(
        SystemCurrentTimeZoneInformation,
        &mut TimeZoneInformation as *mut RTL_TIME_ZONE_INFORMATION as *mut core::ffi::c_void,
        core::mem::size_of::<RTL_TIME_ZONE_INFORMATION>() as ULONG,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return TIME_ZONE_ID_INVALID;
    }

    (*lpTimeZoneInformation).Bias = TimeZoneInformation.Bias;

    wcsncpy(
        (*lpTimeZoneInformation).StandardName.as_mut_ptr(),
        TimeZoneInformation.StandardName.as_ptr(),
        core::mem::size_of::<[WCHAR; 32]>() as size_t / core::mem::size_of::<WCHAR>() as size_t,
    );
    (*lpTimeZoneInformation).StandardDate.wYear = TimeZoneInformation.StandardDate.Year as WORD;
    (*lpTimeZoneInformation).StandardDate.wMonth = TimeZoneInformation.StandardDate.Month as WORD;
    (*lpTimeZoneInformation).StandardDate.wDay = TimeZoneInformation.StandardDate.Day as WORD;
    (*lpTimeZoneInformation).StandardDate.wHour = TimeZoneInformation.StandardDate.Hour as WORD;
    (*lpTimeZoneInformation).StandardDate.wMinute = TimeZoneInformation.StandardDate.Minute as WORD;
    (*lpTimeZoneInformation).StandardDate.wSecond = TimeZoneInformation.StandardDate.Second as WORD;
    (*lpTimeZoneInformation).StandardDate.wMilliseconds = TimeZoneInformation.StandardDate.Milliseconds as WORD;
    (*lpTimeZoneInformation).StandardDate.wDayOfWeek = TimeZoneInformation.StandardDate.Weekday as WORD;
    (*lpTimeZoneInformation).StandardBias = TimeZoneInformation.StandardBias;

    wcsncpy(
        (*lpTimeZoneInformation).DaylightName.as_mut_ptr(),
        TimeZoneInformation.DaylightName.as_ptr(),
        core::mem::size_of::<[WCHAR; 32]>() as size_t / core::mem::size_of::<WCHAR>() as size_t,
    );
    (*lpTimeZoneInformation).DaylightDate.wYear = TimeZoneInformation.DaylightDate.Year as WORD;
    (*lpTimeZoneInformation).DaylightDate.wMonth = TimeZoneInformation.DaylightDate.Month as WORD;
    (*lpTimeZoneInformation).DaylightDate.wDay = TimeZoneInformation.DaylightDate.Day as WORD;
    (*lpTimeZoneInformation).DaylightDate.wHour = TimeZoneInformation.DaylightDate.Hour as WORD;
    (*lpTimeZoneInformation).DaylightDate.wMinute = TimeZoneInformation.DaylightDate.Minute as WORD;
    (*lpTimeZoneInformation).DaylightDate.wSecond = TimeZoneInformation.DaylightDate.Second as WORD;
    (*lpTimeZoneInformation).DaylightDate.wMilliseconds = TimeZoneInformation.DaylightDate.Milliseconds as WORD;
    (*lpTimeZoneInformation).DaylightDate.wDayOfWeek = TimeZoneInformation.DaylightDate.Weekday as WORD;
    (*lpTimeZoneInformation).DaylightBias = TimeZoneInformation.DaylightBias;

    return TIME_ZoneID(lpTimeZoneInformation as *const TIME_ZONE_INFORMATION);
}

#[no_mangle]
pub unsafe extern "system" fn SetTimeZoneInformation(
    lpTimeZoneInformation: *const TIME_ZONE_INFORMATION,
) -> BOOL {
    let mut TimeZoneInformation: RTL_TIME_ZONE_INFORMATION = core::mem::zeroed();
    let mut Status: NTSTATUS;

    TimeZoneInformation.Bias = (*lpTimeZoneInformation).Bias;

    wcsncpy(
        TimeZoneInformation.StandardName.as_mut_ptr(),
        (*lpTimeZoneInformation).StandardName.as_ptr(),
        core::mem::size_of::<[WCHAR; 32]>() / core::mem::size_of::<WCHAR>(),
    );
    TimeZoneInformation.StandardDate.Year = (*lpTimeZoneInformation).StandardDate.wYear as CSHORT;
    TimeZoneInformation.StandardDate.Month = (*lpTimeZoneInformation).StandardDate.wMonth as CSHORT;
    TimeZoneInformation.StandardDate.Day = (*lpTimeZoneInformation).StandardDate.wDay as CSHORT;
    TimeZoneInformation.StandardDate.Hour = (*lpTimeZoneInformation).StandardDate.wHour as CSHORT;
    TimeZoneInformation.StandardDate.Minute = (*lpTimeZoneInformation).StandardDate.wMinute as CSHORT;
    TimeZoneInformation.StandardDate.Second = (*lpTimeZoneInformation).StandardDate.wSecond as CSHORT;
    TimeZoneInformation.StandardDate.Milliseconds = (*lpTimeZoneInformation).StandardDate.wMilliseconds as CSHORT;
    TimeZoneInformation.StandardDate.Weekday = (*lpTimeZoneInformation).StandardDate.wDayOfWeek as CSHORT;
    TimeZoneInformation.StandardBias = (*lpTimeZoneInformation).StandardBias;

    wcsncpy(
        TimeZoneInformation.DaylightName.as_mut_ptr(),
        (*lpTimeZoneInformation).DaylightName.as_ptr(),
        core::mem::size_of::<[WCHAR; 32]>() / core::mem::size_of::<WCHAR>(),
    );
    TimeZoneInformation.DaylightDate.Year = (*lpTimeZoneInformation).DaylightDate.wYear as CSHORT;
    TimeZoneInformation.DaylightDate.Month = (*lpTimeZoneInformation).DaylightDate.wMonth as CSHORT;
    TimeZoneInformation.DaylightDate.Day = (*lpTimeZoneInformation).DaylightDate.wDay as CSHORT;
    TimeZoneInformation.DaylightDate.Hour = (*lpTimeZoneInformation).DaylightDate.wHour as CSHORT;
    TimeZoneInformation.DaylightDate.Minute = (*lpTimeZoneInformation).DaylightDate.wMinute as CSHORT;
    TimeZoneInformation.DaylightDate.Second = (*lpTimeZoneInformation).DaylightDate.wSecond as CSHORT;
    TimeZoneInformation.DaylightDate.Milliseconds = (*lpTimeZoneInformation).DaylightDate.wMilliseconds as CSHORT;
    TimeZoneInformation.DaylightDate.Weekday = (*lpTimeZoneInformation).DaylightDate.wDayOfWeek as CSHORT;
    TimeZoneInformation.DaylightBias = (*lpTimeZoneInformation).DaylightBias;

    Status = RtlSetTimeZoneInformation(&mut TimeZoneInformation);
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return FALSE;
    }

    Status = NtSetSystemInformation(
        SystemCurrentTimeZoneInformation,
        &mut TimeZoneInformation as *mut RTL_TIME_ZONE_INFORMATION as PVOID,
        core::mem::size_of::<RTL_TIME_ZONE_INFORMATION>() as ULONG,
    );
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return FALSE;
    }

    return TRUE;
}

#[no_mangle]
pub unsafe extern "system" fn SystemTimeToTzSpecificLocalTime(
    lpTimeZoneInformation: *const TIME_ZONE_INFORMATION,
    lpUniversalTime: *const SYSTEMTIME,
    lpLocalTime: LPSYSTEMTIME,
) -> BOOL {
    let mut TzInfo: TIME_ZONE_INFORMATION = core::mem::zeroed();
    let mut FileTime: FILETIME = core::mem::zeroed();
    let mut llTime: LONGLONG = 0;
    let mut lBias: LONG = 0;

    if !lpTimeZoneInformation.is_null() {
        TzInfo = *lpTimeZoneInformation;
    } else {
        if GetTimeZoneInformation(&mut TzInfo) == TIME_ZONE_ID_INVALID {
            return FALSE;
        }
    }

    if lpUniversalTime.is_null() || lpLocalTime.is_null() {
        return FALSE;
    }

    if SystemTimeToFileTime(lpUniversalTime, &mut FileTime) == 0 {
        return FALSE;
    }

    llTime = (((FileTime.dwHighDateTime as LONGLONG) << 32) + FileTime.dwLowDateTime as LONGLONG) as LONGLONG;

    if TIME_GetTimezoneBias(&mut TzInfo, &mut FileTime, FALSE, &mut lBias) == 0 {
        return FALSE;
    }

    llTime -= lBias as LONGLONG * TICKSPERMIN as LONGLONG;

    FileTime.dwLowDateTime = llTime as DWORD;
    FileTime.dwHighDateTime = (llTime >> 32) as DWORD;

    return FileTimeToSystemTime(&mut FileTime, lpLocalTime);
}

#[no_mangle]
pub unsafe extern "system" fn TzSpecificLocalTimeToSystemTime(
    lpTimeZoneInformation: LPTIME_ZONE_INFORMATION,
    lpLocalTime: LPSYSTEMTIME,
    lpUniversalTime: LPSYSTEMTIME,
) -> BOOL {
    let mut ft: FILETIME = core::mem::zeroed();
    let mut lBias: LONG = 0;
    let mut t: LONGLONG = 0;
    let mut tzinfo: TIME_ZONE_INFORMATION = core::mem::zeroed();

    if !lpTimeZoneInformation.is_null() {
        tzinfo = *lpTimeZoneInformation;
    } else {
        if GetTimeZoneInformation(&mut tzinfo) == TIME_ZONE_ID_INVALID {
            return FALSE;
        }
    }

    if SystemTimeToFileTime(lpLocalTime, &mut ft) == 0 {
        return FALSE;
    }

    t = (((ft.dwHighDateTime as LONGLONG) << 32) + ft.dwLowDateTime as LONGLONG) as LONGLONG;

    if TIME_GetTimezoneBias(&mut tzinfo, &mut ft, TRUE, &mut lBias) == 0 {
        return FALSE;
    }

    t += lBias as LONGLONG * TICKSPERMIN as LONGLONG;

    ft.dwLowDateTime = t as DWORD;
    ft.dwHighDateTime = (t >> 32) as DWORD;

    return FileTimeToSystemTime(&mut ft, lpUniversalTime);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS system libraries
//  * FILE:            dll/win32/kernel32/wine/timezone.c
//  * PURPOSE:         Time conversion functions
//  * PROGRAMMER:      Ariadne
//  *                  DOSDATE and DOSTIME structures from Onno Hovers
//  * UPDATE HISTORY:
//  *                  Created 19/01/99
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <k32.h>
//
// #define NDEBUG
// #include <debug.h>
//
// /* TYPES *********************************************************************/
//
// #define TICKSPERMIN 600000000
//
// #define LL2FILETIME( ll, pft )\
//     (pft)->dwLowDateTime = (UINT)(ll); \
//     (pft)->dwHighDateTime = (UINT)((ll) >> 32);
// #define FILETIME2LL( pft, ll) \
//     ll = (((LONGLONG)((pft)->dwHighDateTime))<<32) + (pft)-> dwLowDateTime ;
//
// static const int MonthLengths[2][12] =
// {
//     { 31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31 },
//     { 31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31 }
// };
//
// /* STATIC FUNCTIONS **********************************************************/
//
// static inline int IsLeapYear(int Year)
// {
//     return Year % 4 == 0 && (Year % 100 != 0 || Year % 400 == 0) ? 1 : 0;
// }
//
// /***********************************************************************
//  *  TIME_DayLightCompareDate
//  *
//  * Compares two dates without looking at the year.
//  *
//  * PARAMS
//  *   date        [in] The local time to compare.
//  *   compareDate [in] The daylight savings begin or end date.
//  *
//  * RETURNS
//  *
//  *  -1 if date < compareDate
//  *   0 if date == compareDate
//  *   1 if date > compareDate
//  *  -2 if an error occurs
//  */
// static int
// TIME_DayLightCompareDate(const SYSTEMTIME *date, const SYSTEMTIME *compareDate)
// {
//     int limit_day, dayinsecs;
//
//     if (date->wMonth < compareDate->wMonth)
//         return -1; /* We are in a month before the date limit. */
//
//     if (date->wMonth > compareDate->wMonth)
//         return 1; /* We are in a month after the date limit. */
//
//     /* if year is 0 then date is in day-of-week format, otherwise
//      * it's absolute date.
//      */
//     if (compareDate->wYear == 0)
//     {
//         WORD First;
//         /* compareDate->wDay is interpreted as number of the week in the month
//          * 5 means: the last week in the month */
//         int weekofmonth = compareDate->wDay;
//           /* calculate the day of the first DayOfWeek in the month */
//         First = ( 6 + compareDate->wDayOfWeek - date->wDayOfWeek + date->wDay
//                ) % 7 + 1;
//         limit_day = First + 7 * (weekofmonth - 1);
//         /* check needed for the 5th weekday of the month */
//         if(limit_day > MonthLengths[date->wMonth==2 && IsLeapYear(date->wYear)]
//                 [date->wMonth - 1])
//             limit_day -= 7;
//     }
//     else
//     {
//        limit_day = compareDate->wDay;
//     }
//
//     /* convert to seconds */
//     limit_day = ((limit_day * 24  + compareDate->wHour) * 60 +
//             compareDate->wMinute ) * 60;
//     dayinsecs = ((date->wDay * 24  + date->wHour) * 60 +
//             date->wMinute ) * 60 + date->wSecond;
//     /* and compare */
//     return dayinsecs < limit_day ? -1 :
//            dayinsecs > limit_day ? 1 :
//            0;   /* date is equal to the date limit. */
// }
//
// /***********************************************************************
//  *  TIME_CompTimeZoneID
//  *
//  *  Computes the local time bias for a given time and time zone.
//  *
//  *  PARAMS
//  *      pTZinfo     [in] The time zone data.
//  *      lpFileTime  [in] The system or local time.
//  *      islocal     [in] it is local time.
//  *
//  *  RETURNS
//  *      TIME_ZONE_ID_INVALID    An error occurred
//  *      TIME_ZONE_ID_UNKNOWN    There are no transition time known
//  *      TIME_ZONE_ID_STANDARD   Current time is standard time
//  *      TIME_ZONE_ID_DAYLIGHT   Current time is daylight savings time
//  */
// static
// DWORD
// TIME_CompTimeZoneID( const TIME_ZONE_INFORMATION *pTZinfo, FILETIME *lpFileTime, BOOL islocal )
// {
//     int ret, year;
//     BOOL beforeStandardDate, afterDaylightDate;
//     DWORD retval = TIME_ZONE_ID_INVALID;
//     LONGLONG llTime = 0; /* initialized to prevent gcc complaining */
//     SYSTEMTIME SysTime;
//     FILETIME ftTemp;
//
//     if (pTZinfo->DaylightDate.wMonth != 0)
//     {
//         /* if year is 0 then date is in day-of-week format, otherwise
//          * it's absolute date.
//          */
//         if (pTZinfo->StandardDate.wMonth == 0 ||
//             (pTZinfo->StandardDate.wYear == 0 &&
//             (pTZinfo->StandardDate.wDay<1 ||
//             pTZinfo->StandardDate.wDay>5 ||
//             pTZinfo->DaylightDate.wDay<1 ||
//             pTZinfo->DaylightDate.wDay>5)))
//         {
//             SetLastError(ERROR_INVALID_PARAMETER);
//             return TIME_ZONE_ID_INVALID;
//         }
//
//         if (!islocal) {
//             FILETIME2LL( lpFileTime, llTime );
//             llTime -= pTZinfo->Bias * (LONGLONG)TICKSPERMIN;
//             LL2FILETIME( llTime, &ftTemp)
//             lpFileTime = &ftTemp;
//         }
//
//         FileTimeToSystemTime(lpFileTime, &SysTime);
//         year = SysTime.wYear;
//
//         if (!islocal) {
//             llTime -= pTZinfo->DaylightBias * (LONGLONG)TICKSPERMIN;
//             LL2FILETIME( llTime, &ftTemp)
//             FileTimeToSystemTime(lpFileTime, &SysTime);
//         }
//
//         /* check for daylight savings */
//         if(year == SysTime.wYear) {
//             ret = TIME_DayLightCompareDate( &SysTime, &pTZinfo->StandardDate);
//             if (ret == -2)
//                 return TIME_ZONE_ID_INVALID;
//
//             beforeStandardDate = ret < 0;
//         } else
//             beforeStandardDate = SysTime.wYear < year;
//
//         if (!islocal) {
//             llTime -= ( pTZinfo->StandardBias - pTZinfo->DaylightBias )
//                 * (LONGLONG)TICKSPERMIN;
//             LL2FILETIME( llTime, &ftTemp)
//             FileTimeToSystemTime(lpFileTime, &SysTime);
//         }
//
//         if(year == SysTime.wYear) {
//             ret = TIME_DayLightCompareDate( &SysTime, &pTZinfo->DaylightDate);
//             if (ret == -2)
//                 return TIME_ZONE_ID_INVALID;
//
//             afterDaylightDate = ret >= 0;
//         } else
//             afterDaylightDate = SysTime.wYear > year;
//
//         retval = TIME_ZONE_ID_STANDARD;
//         if( pTZinfo->DaylightDate.wMonth <  pTZinfo->StandardDate.wMonth ) {
//             /* Northern hemisphere */
//             if( beforeStandardDate && afterDaylightDate )
//                 retval = TIME_ZONE_ID_DAYLIGHT;
//         } else    /* Down south */
//             if( beforeStandardDate || afterDaylightDate )
//             retval = TIME_ZONE_ID_DAYLIGHT;
//     } else
//         /* No transition date */
//         retval = TIME_ZONE_ID_UNKNOWN;
//
//     return retval;
// }
//
// /***********************************************************************
//  *  TIME_TimeZoneID
//  *
//  *  Calculates whether daylight savings is on now.
//  *
//  *  PARAMS
//  *      pTzi [in] Timezone info.
//  *
//  *  RETURNS
//  *      TIME_ZONE_ID_INVALID    An error occurred
//  *      TIME_ZONE_ID_UNKNOWN    There are no transition time known
//  *      TIME_ZONE_ID_STANDARD   Current time is standard time
//  *      TIME_ZONE_ID_DAYLIGHT   Current time is daylight savings time
//  */
// static DWORD TIME_ZoneID( const TIME_ZONE_INFORMATION *pTzi )
// {
//     FILETIME ftTime;
//     GetSystemTimeAsFileTime( &ftTime);
//     return TIME_CompTimeZoneID( pTzi, &ftTime, FALSE);
// }
//
// /***********************************************************************
//  *  TIME_GetTimezoneBias
//  *
//  *  Calculates the local time bias for a given time zone.
//  *
//  * PARAMS
//  *  pTZinfo    [in]  The time zone data.
//  *  lpFileTime [in]  The system or local time.
//  *  islocal    [in]  It is local time.
//  *  pBias      [out] The calculated bias in minutes.
//  *
//  * RETURNS
//  *  TRUE when the time zone bias was calculated.
//  */
// static BOOL
// TIME_GetTimezoneBias(const TIME_ZONE_INFORMATION *pTZinfo, FILETIME *lpFileTime, BOOL islocal, LONG *pBias)
// {
//     LONG bias = pTZinfo->Bias;
//     DWORD tzid = TIME_CompTimeZoneID(pTZinfo, lpFileTime, islocal);
//
//     if( tzid == TIME_ZONE_ID_INVALID)
//         return FALSE;
//     if (tzid == TIME_ZONE_ID_DAYLIGHT)
//         bias += pTZinfo->DaylightBias;
//     else if (tzid == TIME_ZONE_ID_STANDARD)
//         bias += pTZinfo->StandardBias;
//     *pBias = bias;
//     return TRUE;
// }
//
//
// /* FUNCTIONS ****************************************************************/
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// GetTimeZoneInformation(LPTIME_ZONE_INFORMATION lpTimeZoneInformation)
// {
//     RTL_TIME_ZONE_INFORMATION TimeZoneInformation;
//     NTSTATUS Status;
//
//     DPRINT("GetTimeZoneInformation()\n");
//
//     Status = NtQuerySystemInformation(SystemCurrentTimeZoneInformation,
//                                       &TimeZoneInformation,
//                                       sizeof(RTL_TIME_ZONE_INFORMATION),
//                                       NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError(Status);
//         return TIME_ZONE_ID_INVALID;
//     }
//
//     lpTimeZoneInformation->Bias = TimeZoneInformation.Bias;
//
//     wcsncpy(lpTimeZoneInformation->StandardName,
//             TimeZoneInformation.StandardName,
//             ARRAYSIZE(lpTimeZoneInformation->StandardName));
//     lpTimeZoneInformation->StandardDate.wYear = TimeZoneInformation.StandardDate.Year;
//     lpTimeZoneInformation->StandardDate.wMonth = TimeZoneInformation.StandardDate.Month;
//     lpTimeZoneInformation->StandardDate.wDay = TimeZoneInformation.StandardDate.Day;
//     lpTimeZoneInformation->StandardDate.wHour = TimeZoneInformation.StandardDate.Hour;
//     lpTimeZoneInformation->StandardDate.wMinute = TimeZoneInformation.StandardDate.Minute;
//     lpTimeZoneInformation->StandardDate.wSecond = TimeZoneInformation.StandardDate.Second;
//     lpTimeZoneInformation->StandardDate.wMilliseconds = TimeZoneInformation.StandardDate.Milliseconds;
//     lpTimeZoneInformation->StandardDate.wDayOfWeek = TimeZoneInformation.StandardDate.Weekday;
//     lpTimeZoneInformation->StandardBias = TimeZoneInformation.StandardBias;
//
//     wcsncpy(lpTimeZoneInformation->DaylightName,
//             TimeZoneInformation.DaylightName,
//             ARRAYSIZE(lpTimeZoneInformation->DaylightName));
//     lpTimeZoneInformation->DaylightDate.wYear = TimeZoneInformation.DaylightDate.Year;
//     lpTimeZoneInformation->DaylightDate.wMonth = TimeZoneInformation.DaylightDate.Month;
//     lpTimeZoneInformation->DaylightDate.wDay = TimeZoneInformation.DaylightDate.Day;
//     lpTimeZoneInformation->DaylightDate.wHour = TimeZoneInformation.DaylightDate.Hour;
//     lpTimeZoneInformation->DaylightDate.wMinute = TimeZoneInformation.DaylightDate.Minute;
//     lpTimeZoneInformation->DaylightDate.wSecond = TimeZoneInformation.DaylightDate.Second;
//     lpTimeZoneInformation->DaylightDate.wMilliseconds = TimeZoneInformation.DaylightDate.Milliseconds;
//     lpTimeZoneInformation->DaylightDate.wDayOfWeek = TimeZoneInformation.DaylightDate.Weekday;
//     lpTimeZoneInformation->DaylightBias = TimeZoneInformation.DaylightBias;
//
//     return TIME_ZoneID(lpTimeZoneInformation);
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// SetTimeZoneInformation(CONST TIME_ZONE_INFORMATION *lpTimeZoneInformation)
// {
//     RTL_TIME_ZONE_INFORMATION TimeZoneInformation;
//     NTSTATUS Status;
//
//     DPRINT("SetTimeZoneInformation()\n");
//
//     TimeZoneInformation.Bias = lpTimeZoneInformation->Bias;
//
//     wcsncpy(TimeZoneInformation.StandardName,
//             lpTimeZoneInformation->StandardName,
//             ARRAYSIZE(TimeZoneInformation.StandardName));
//     TimeZoneInformation.StandardDate.Year = lpTimeZoneInformation->StandardDate.wYear;
//     TimeZoneInformation.StandardDate.Month = lpTimeZoneInformation->StandardDate.wMonth;
//     TimeZoneInformation.StandardDate.Day = lpTimeZoneInformation->StandardDate.wDay;
//     TimeZoneInformation.StandardDate.Hour = lpTimeZoneInformation->StandardDate.wHour;
//     TimeZoneInformation.StandardDate.Minute = lpTimeZoneInformation->StandardDate.wMinute;
//     TimeZoneInformation.StandardDate.Second = lpTimeZoneInformation->StandardDate.wSecond;
//     TimeZoneInformation.StandardDate.Milliseconds = lpTimeZoneInformation->StandardDate.wMilliseconds;
//     TimeZoneInformation.StandardDate.Weekday = lpTimeZoneInformation->StandardDate.wDayOfWeek;
//     TimeZoneInformation.StandardBias = lpTimeZoneInformation->StandardBias;
//
//     wcsncpy(TimeZoneInformation.DaylightName,
//             lpTimeZoneInformation->DaylightName,
//             ARRAYSIZE(TimeZoneInformation.DaylightName));
//     TimeZoneInformation.DaylightDate.Year = lpTimeZoneInformation->DaylightDate.wYear;
//     TimeZoneInformation.DaylightDate.Month = lpTimeZoneInformation->DaylightDate.wMonth;
//     TimeZoneInformation.DaylightDate.Day = lpTimeZoneInformation->DaylightDate.wDay;
//     TimeZoneInformation.DaylightDate.Hour = lpTimeZoneInformation->DaylightDate.wHour;
//     TimeZoneInformation.DaylightDate.Minute = lpTimeZoneInformation->DaylightDate.wMinute;
//     TimeZoneInformation.DaylightDate.Second = lpTimeZoneInformation->DaylightDate.wSecond;
//     TimeZoneInformation.DaylightDate.Milliseconds = lpTimeZoneInformation->DaylightDate.wMilliseconds;
//     TimeZoneInformation.DaylightDate.Weekday = lpTimeZoneInformation->DaylightDate.wDayOfWeek;
//     TimeZoneInformation.DaylightBias = lpTimeZoneInformation->DaylightBias;
//
//     Status = RtlSetTimeZoneInformation(&TimeZoneInformation);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("RtlSetTimeZoneInformation() failed (Status %lx)\n", Status);
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     Status = NtSetSystemInformation(SystemCurrentTimeZoneInformation,
//                                     (PVOID)&TimeZoneInformation,
//                                     sizeof(RTL_TIME_ZONE_INFORMATION));
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("NtSetSystemInformation() failed (Status %lx)\n", Status);
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
// SystemTimeToTzSpecificLocalTime(CONST TIME_ZONE_INFORMATION *lpTimeZoneInformation,
//                                 CONST SYSTEMTIME *lpUniversalTime,
//                                 LPSYSTEMTIME lpLocalTime)
// {
//     TIME_ZONE_INFORMATION TzInfo;
//     FILETIME FileTime;
//     LONGLONG llTime;
//     LONG lBias;
//
//     if (lpTimeZoneInformation != NULL)
//     {
//         TzInfo = *lpTimeZoneInformation;
//     }
//     else
//     {
//         if (GetTimeZoneInformation(&TzInfo) == TIME_ZONE_ID_INVALID)
//             return FALSE;
//     }
//
//     if (!lpUniversalTime || !lpLocalTime)
//         return FALSE;
//
//     if (!SystemTimeToFileTime(lpUniversalTime, &FileTime))
//         return FALSE;
//
//     FILETIME2LL(&FileTime, llTime)
//
//     if (!TIME_GetTimezoneBias(&TzInfo, &FileTime, FALSE, &lBias))
//         return FALSE;
//
//     /* convert minutes to 100-nanoseconds-ticks */
//     llTime -= (LONGLONG)lBias * TICKSPERMIN;
//
//     LL2FILETIME( llTime, &FileTime)
//
//     return FileTimeToSystemTime(&FileTime, lpLocalTime);
// }
//
//
// /*
//  * @implemented (Wine 13 sep 2008)
//  */
// BOOL
// WINAPI
// TzSpecificLocalTimeToSystemTime(LPTIME_ZONE_INFORMATION lpTimeZoneInformation,
//                                 LPSYSTEMTIME lpLocalTime,
//                                 LPSYSTEMTIME lpUniversalTime)
// {
//     FILETIME ft;
//     LONG lBias;
//     LONGLONG t;
//     TIME_ZONE_INFORMATION tzinfo;
//
//     if (lpTimeZoneInformation != NULL)
//     {
//         tzinfo = *lpTimeZoneInformation;
//     }
//     else
//     {
//         if (GetTimeZoneInformation(&tzinfo) == TIME_ZONE_ID_INVALID)
//             return FALSE;
//     }
//
//     if (!SystemTimeToFileTime(lpLocalTime, &ft))
//         return FALSE;
//     FILETIME2LL( &ft, t)
//     if (!TIME_GetTimezoneBias(&tzinfo, &ft, TRUE, &lBias))
//         return FALSE;
//     /* convert minutes to 100-nanoseconds-ticks */
//     t += (LONGLONG)lBias * TICKSPERMIN;
//     LL2FILETIME( t, &ft)
//     return FileTimeToSystemTime(&ft, lpUniversalTime);
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
