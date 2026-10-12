use ntkrnl_sys::globals::*;
use crate::externs::DbgPrint;
// One declaration per external symbol: externs.rs.
use crate::externs::RtlAssert;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{ExLocalTimeToSystemTime, ExSystemTimeToLocalTime, LARGE_INTEGER, PLARGE_INTEGER, RtlTimeFieldsToTime, RtlTimeToTimeFields, TIME_FIELDS};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{AlmostTwoSeconds, FAT_DATE, FAT_TIME_STAMP, PFAT_TIME_STAMP, PIRP_CONTEXT};
use ntkrnl_sys::ddk::{CSHORT, PCSTR};

extern "system" {
    fn KeGetCurrentIrql() -> KIRQL;
    } extern "system" {
    static mut SharedSystemTime: u64;
}

// ---- file-scope preamble (translated) ----
// No file-scope declarations to translate; the original C preamble contains only #pragma directives,
// which are not types, constants, or tables and have no Rust equivalent in this context.

const APC_LEVEL: KIRQL = 1;



// Forward declarations for missing externs

#[no_mangle]
pub unsafe extern "system" fn FatNtTimeToFatTime(
    IrpContext: PIRP_CONTEXT,
    NtTime: PLARGE_INTEGER,
    Rounding: BOOLEAN,
    FatTime: PFAT_TIME_STAMP,
    TenMsecs: PUCHAR,
) -> BOOLEAN {
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

    // PAGED_CODE macro expansion
    if KeGetCurrentIrql() as i32 > APC_LEVEL as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as PCSTR,
            KeGetCurrentIrql() as i32,
        );
        RtlAssert(
            b"0\0".as_ptr() as *const i8 as PVOID,
            b"drivers/filesystems/fastfat/timesup.c\0"
                .as_ptr() as *const i8 as PVOID,
            66 as ULONG,
            (core::ptr::null_mut::<i8>()).cast(),
        );
    }

    if Rounding != 0 {
        (*NtTime).QuadPart = ((*NtTime).QuadPart as i64 + AlmostTwoSeconds as i64) as LONGLONG;
    }

    ExSystemTimeToLocalTime(NtTime, NtTime);
    RtlTimeToTimeFields(NtTime, &raw mut TimeFields);

    if (TimeFields.Year as i32) < 1980
        || (TimeFields.Year as i32) > (1980 + 127)
    {
        ExLocalTimeToSystemTime(NtTime, NtTime);
        return 0; // FALSE
    }

    (*FatTime).Time.set_DoubleSeconds((TimeFields.Second as i32 / 2) as USHORT);
    (*FatTime).Time.set_Minute(TimeFields.Minute as USHORT);
    (*FatTime).Time.set_Hour(TimeFields.Hour as USHORT);
    (*FatTime).Date.set_Year((TimeFields.Year as i32 - 1980) as USHORT);
    (*FatTime).Date.set_Month(TimeFields.Month as USHORT);
    (*FatTime).Date.set_Day(TimeFields.Day as USHORT);

    if !TenMsecs.is_null() {
        if Rounding == 0 {
            *TenMsecs = (TimeFields.Milliseconds as i32 / 10
                + (TimeFields.Second as i32 % 2) * 100) as u8;
        } else {
            *TenMsecs = 0u8;
        }
    }

    if Rounding != 0 {
        TimeFields.Milliseconds = 0 as CSHORT;
        TimeFields.Second = (TimeFields.Second as i32
            - (TimeFields.Second as i32 % 2)) as CSHORT;
    } else {
        TimeFields.Milliseconds = (TimeFields.Milliseconds as i32
            - (TimeFields.Milliseconds as i32 % 10)) as CSHORT;
    }

    RtlTimeFieldsToTime(&raw mut TimeFields, NtTime);
    ExLocalTimeToSystemTime(NtTime, NtTime);

    let _ = IrpContext; // UNREFERENCED_PARAMETER

    1 // TRUE
}

#[no_mangle]
pub unsafe extern "system" fn FatFatDateToNtTime(
    IrpContext: PIRP_CONTEXT,
    FatDate: FAT_DATE,
) -> LARGE_INTEGER {
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
    let mut Time: LARGE_INTEGER = unsafe { core::mem::zeroed() };

    if KeGetCurrentIrql() as i32 > (APC_LEVEL) as i32 {
        DbgPrint(
            (b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const u8).cast(),
            KeGetCurrentIrql() as i32,
        );
        if 0i32 == 0 {
            RtlAssert(
                b"0\0".as_ptr() as *mut core::ffi::c_void,
                b"drivers/filesystems/fastfat/timesup.c\0"
                    .as_ptr() as *mut core::ffi::c_void,
                197u32,
                core::ptr::null_mut(),
            );
        }
    }

    TimeFields.Year = (FatDate.Year() as i32 + 1980i32) as u16 as i16;
    TimeFields.Month = FatDate.Month() as i16;
    TimeFields.Day = FatDate.Day() as i16;
    TimeFields.Hour = 0i32 as u16 as i16;
    TimeFields.Minute = 0i32 as u16 as i16;
    TimeFields.Second = 0i32 as u16 as i16;
    TimeFields.Milliseconds = 0i32 as u16 as i16;

    if RtlTimeFieldsToTime(&raw mut TimeFields, &raw mut Time) == 0 {
        Time.__bindgen_anon_1.LowPart = 0u32;
        Time.__bindgen_anon_1.HighPart = 0i32;
    } else {
        ExLocalTimeToSystemTime(&raw mut Time, &raw mut Time);
    }

    return Time;
}
#[no_mangle]
pub unsafe extern "system" fn FatFatTimeToNtTime(
    IrpContext: PIRP_CONTEXT,
    FatTime: FAT_TIME_STAMP,
    TenMilliSeconds: u8,
) -> LARGE_INTEGER {
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
    let mut Time: LARGE_INTEGER = core::mem::zeroed();

    if KeGetCurrentIrql() > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            RtlAssert(
                b"0\0" as *const u8 as *mut core::ffi::c_void,
                b"drivers/filesystems/fastfat/timesup.c\0"
                    as *const u8 as *mut core::ffi::c_void,
                262u32,
                (core::ptr::null_mut::<u8>() as *mut core::ffi::c_void).cast(),
            );
        }
    }

    TimeFields.Year = ((FatTime.Date.Year() + 1980) as u16) as i16;
    TimeFields.Month = (FatTime.Date.Month() as u16) as i16;
    TimeFields.Day = (FatTime.Date.Day() as u16) as i16;
    TimeFields.Hour = (FatTime.Time.Hour() as u16) as i16;
    TimeFields.Minute = (FatTime.Time.Minute() as u16) as i16;
    TimeFields.Second = ((FatTime.Time.DoubleSeconds() * 2) as u16) as i16;

    if TenMilliSeconds != 0 {
        TimeFields.Second += ((TenMilliSeconds / 100) as u16) as i16;
        TimeFields.Milliseconds = (((TenMilliSeconds % 100) * 10) as u16) as i16;
    } else {
        TimeFields.Milliseconds = (0u16) as i16;
    }

    if TimeFields.Second > 59 {
        TimeFields.Second = 0;
    }

    if RtlTimeFieldsToTime(
        core::ptr::addr_of_mut!(TimeFields),
        core::ptr::addr_of_mut!(Time),
    ) == 0
    {
        Time.u.LowPart = 0;
        Time.u.HighPart = 0;
    } else {
        ExLocalTimeToSystemTime(
            core::ptr::addr_of_mut!(Time),
            core::ptr::addr_of_mut!(Time),
        );
    }

    UNREFERENCED_PARAMETER!(IrpContext as *mut core::ffi::c_void);

    return Time;
}

#[no_mangle]
pub unsafe extern "system" fn FatGetCurrentFatTime(IrpContext: PIRP_CONTEXT) -> FAT_TIME_STAMP {
    // Zero-initialize locals as C would (all fields set to 0)
    let mut Time: LARGE_INTEGER = LARGE_INTEGER { QuadPart: 0 };
    let mut TimeFields: TIME_FIELDS = core::mem::zeroed();
    let mut FatTime: FAT_TIME_STAMP = core::mem::zeroed();

    // PAGED_CODE() expansion
    if KeGetCurrentIrql() as u8 > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            RtlAssert(
                b"0\0".as_ptr() as *mut core::ffi::c_void,
                b"drivers/filesystems/fastfat/timesup.c\0"
                    .as_ptr() as *mut core::ffi::c_void,
                340,
                (core::ptr::null_mut::<i8>() as *mut core::ffi::c_void).cast(),
            );
        }
    }

    // KeQuerySystemTime(&Time) inline:
    // Copy the whole 64-bit value of SharedSystemTime into Time
    *(core::ptr::addr_of_mut!(Time) as *mut u64) =
        *(core::ptr::addr_of!(SharedSystemTime) as *const u64);

    ExSystemTimeToLocalTime(
        core::ptr::addr_of_mut!(Time),
        core::ptr::addr_of_mut!(Time),
    );

    // AlmostTwoSeconds addition
    Time.QuadPart = Time.QuadPart + AlmostTwoSeconds as LONGLONG;

    RtlTimeToTimeFields(
        core::ptr::addr_of_mut!(Time),
        core::ptr::addr_of_mut!(TimeFields),
    );

    // Assign FAT time fields using the same setter methods as the anchor
    FatTime.Time.set_DoubleSeconds((TimeFields.Second / 2) as u16);
    FatTime.Time.set_Minute(TimeFields.Minute as u16);
    FatTime.Time.set_Hour(TimeFields.Hour as u16);

    FatTime.Date.set_Year((TimeFields.Year - 1980) as u16);
    FatTime.Date.set_Month(TimeFields.Month as u16);
    FatTime.Date.set_Day(TimeFields.Day as u16);

    // UNREFERENCED_PARAMETER macro (suppress warning)
    let _ = IrpContext;

    return FatTime;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*++
//
// Copyright (c) 1989-2000 Microsoft Corporation
//
// Module Name:
//
//     TimeSup.c
//
// Abstract:
//
//     This module implements the Fat Time conversion support routines
//
//
// --*/
//
// #include "fatprocs.h"
//
// #ifdef ALLOC_PRAGMA
// #pragma alloc_text(PAGE, FatNtTimeToFatTime)
// #pragma alloc_text(PAGE, FatFatDateToNtTime)
// #pragma alloc_text(PAGE, FatFatTimeToNtTime)
// #pragma alloc_text(PAGE, FatGetCurrentFatTime)
// #endif
//
//
// BOOLEAN
// FatNtTimeToFatTime (
//      PIRP_CONTEXT IrpContext,
//      PLARGE_INTEGER NtTime,
//      BOOLEAN Rounding,
//      PFAT_TIME_STAMP FatTime,
//      PUCHAR TenMsecs
//     )
//
// /*++
//
// Routine Description:
//
//     This routine converts an NtTime value to its corresponding Fat time value.
//
// Arguments:
//
//     NtTime - Supplies the Nt GMT Time value to convert from
//
//     Rounding - Indicates whether the NT time should be rounded up to a FAT boundary.
//         This should only be done *once* in the lifetime of a timestamp (important
//         for tunneling, which will cause a timestamp to pass through at least twice).
//         If true, rounded up. If false, rounded down to 10ms boundary. This obeys
//         the rules for non-creation time and creation times (respectively).
//
//     FatTime - Receives the equivalent Fat time value
//
//     TenMsecs - Optionally receive the number of tens of milliseconds the NtTime, after
//         any rounding, is greater than the FatTime
//
// Return Value:
//
//     BOOLEAN - TRUE if the Nt time value is within the range of Fat's
//         time range, and FALSE otherwise
//
// --*/
//
// {
//     TIME_FIELDS TimeFields;
//
//     PAGED_CODE();
//
//     //
//     //  Convert the input to the a time field record.
//     //
//
//     if (Rounding) {
//
//         //
//         //   Add almost two seconds to round up to the nearest double second.
//         //
//
//         NtTime->QuadPart = NtTime->QuadPart + AlmostTwoSeconds;
//     }
//
//     ExSystemTimeToLocalTime( NtTime, NtTime );
//
//     RtlTimeToTimeFields( NtTime, &TimeFields );
//
//     //
//     //  Check the range of the date found in the time field record
//     //
//
//     if ((TimeFields.Year < 1980) || (TimeFields.Year > (1980 + 127))) {
//
//         ExLocalTimeToSystemTime( NtTime, NtTime );
//
//         return FALSE;
//     }
//
//     //
//     //  The year will fit in Fat so simply copy over the information
//     //
//
//     FatTime->Time.DoubleSeconds = (USHORT)(TimeFields.Second / 2);
//     FatTime->Time.Minute        = (USHORT)(TimeFields.Minute);
//     FatTime->Time.Hour          = (USHORT)(TimeFields.Hour);
//
//     FatTime->Date.Year          = (USHORT)(TimeFields.Year - 1980);
//     FatTime->Date.Month         = (USHORT)(TimeFields.Month);
//     FatTime->Date.Day           = (USHORT)(TimeFields.Day);
//
//     if (TenMsecs) {
//
//         if (!Rounding) {
//
//             //
//             //  If the number of seconds was not divisible by two, then there
//             //  is another second of time (1 sec, 3 sec, etc.) Note we round down
//             //  the number of milleconds onto tens of milleseconds boundaries.
//             //
//
// #ifdef _MSC_VER
// #pragma warning( push )
// #pragma warning( disable: 4244 )
// #endif
//             *TenMsecs = (TimeFields.Milliseconds / 10) +
//                 ((TimeFields.Second % 2) * 100);
// #ifdef _MSC_VER
// #pragma warning( pop )
// #endif
//         } else {
//
//             //
//             //  If we rounded up, we have in effect changed the NT time. Therefore,
//             //  it does not differ from the FAT time.
//             //
//
//             *TenMsecs = 0;
//         }
//     }
//
//     if (Rounding) {
//
//         //
//         //  Slice off non-FAT boundary time and convert back to 64bit form
//         //
//
//         TimeFields.Milliseconds = 0;
//         TimeFields.Second -= TimeFields.Second % 2;
//
//     } else {
//
//         //
//         //  Round down to 10ms boundary
//         //
//
//         TimeFields.Milliseconds -= TimeFields.Milliseconds % 10;
//     }
//
//     //
//     //  Convert back to NT time
//     //
//
//     (VOID) RtlTimeFieldsToTime(&TimeFields, NtTime);
//
//     ExLocalTimeToSystemTime( NtTime, NtTime );
//
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     return TRUE;
// }
//
//
// LARGE_INTEGER
// FatFatDateToNtTime (
//      PIRP_CONTEXT IrpContext,
//      FAT_DATE FatDate
//     )
//
// /*++
//
// Routine Description:
//
//     This routine converts a Fat datev value to its corresponding Nt GMT
//     Time value.
//
// Arguments:
//
//     FatDate - Supplies the Fat Date to convert from
//
// Return Value:
//
//     LARGE_INTEGER - Receives the corresponding Nt Time value
//
// --*/
//
// {
//     TIME_FIELDS TimeFields;
//     LARGE_INTEGER Time;
//
//     PAGED_CODE();
//
//     //
//     //  Pack the input time/date into a time field record
//     //
//
//     TimeFields.Year         = (USHORT)(FatDate.Year + 1980);
//     TimeFields.Month        = (USHORT)(FatDate.Month);
//     TimeFields.Day          = (USHORT)(FatDate.Day);
//     TimeFields.Hour         = (USHORT)0;
//     TimeFields.Minute       = (USHORT)0;
//     TimeFields.Second       = (USHORT)0;
//     TimeFields.Milliseconds = (USHORT)0;
//
//     //
//     //  Convert the time field record to Nt LARGE_INTEGER, and set it to zero
//     //  if we were given a bogus time.
//     //
//
//     if (!RtlTimeFieldsToTime( &TimeFields, &Time )) {
//
//         Time.LowPart = 0;
//         Time.HighPart = 0;
//
//     } else {
//
//         ExLocalTimeToSystemTime( &Time, &Time );
//     }
//
//     return Time;
//
//     UNREFERENCED_PARAMETER( IrpContext );
// }
//
//
// LARGE_INTEGER
// FatFatTimeToNtTime (
//      PIRP_CONTEXT IrpContext,
//      FAT_TIME_STAMP FatTime,
//      UCHAR TenMilliSeconds
//     )
//
// /*++
//
// Routine Description:
//
//     This routine converts a Fat time value pair to its corresponding Nt GMT
//     Time value.
//
// Arguments:
//
//     FatTime - Supplies the Fat Time to convert from
//
//     TenMilliSeconds - A 10 Milisecond resolution
//
// Return Value:
//
//     LARGE_INTEGER - Receives the corresponding Nt GMT Time value
//
// --*/
//
// {
//     TIME_FIELDS TimeFields;
//     LARGE_INTEGER Time;
//
//     PAGED_CODE();
//
//     //
//     //  Pack the input time/date into a time field record
//     //
//
//     TimeFields.Year         = (USHORT)(FatTime.Date.Year + 1980);
//     TimeFields.Month        = (USHORT)(FatTime.Date.Month);
//     TimeFields.Day          = (USHORT)(FatTime.Date.Day);
//     TimeFields.Hour         = (USHORT)(FatTime.Time.Hour);
//     TimeFields.Minute       = (USHORT)(FatTime.Time.Minute);
//     TimeFields.Second       = (USHORT)(FatTime.Time.DoubleSeconds * 2);
//
//     if (TenMilliSeconds != 0) {
//
//         TimeFields.Second      += (USHORT)(TenMilliSeconds / 100);
//         TimeFields.Milliseconds = (USHORT)((TenMilliSeconds % 100) * 10);
//
//     } else {
//
//         TimeFields.Milliseconds = (USHORT)0;
//     }
//
//     //
//     //  If the second value is greater than 59 then we truncate it to 0.
//     //  Note that this can't happen with a proper FAT timestamp.
//     //
//
//     if (TimeFields.Second > 59) {
//
//         TimeFields.Second = 0;
//     }
//
//     //
//     //  Convert the time field record to Nt LARGE_INTEGER, and set it to zero
//     //  if we were given a bogus time.
//     //
//
//     if (!RtlTimeFieldsToTime( &TimeFields, &Time )) {
//
//         Time.LowPart = 0;
//         Time.HighPart = 0;
//
//     } else {
//
//         ExLocalTimeToSystemTime( &Time, &Time );
//     }
//
//     return Time;
//
//     UNREFERENCED_PARAMETER( IrpContext );
// }
//
//
// FAT_TIME_STAMP
// FatGetCurrentFatTime (
//      PIRP_CONTEXT IrpContext
//     )
//
// /*++
//
// Routine Description:
//
//     This routine returns the current system time in Fat time
//
// Arguments:
//
// Return Value:
//
//     FAT_TIME_STAMP - Receives the current system time
//
// --*/
//
// {
//     LARGE_INTEGER Time;
//     TIME_FIELDS TimeFields;
//     FAT_TIME_STAMP FatTime;
//
//     PAGED_CODE();
//
//     //
//     //  Get the current system time, and map it into a time field record.
//     //
//
//     KeQuerySystemTime( &Time );
//
//     ExSystemTimeToLocalTime( &Time, &Time );
//
//     //
//     //  Always add almost two seconds to round up to the nearest double second.
//     //
//
//     Time.QuadPart = Time.QuadPart + AlmostTwoSeconds;
//
//     (VOID)RtlTimeToTimeFields( &Time, &TimeFields );
//
//     //
//     //  Now simply copy over the information
//     //
//
//     FatTime.Time.DoubleSeconds = (USHORT)(TimeFields.Second / 2);
//     FatTime.Time.Minute        = (USHORT)(TimeFields.Minute);
//     FatTime.Time.Hour          = (USHORT)(TimeFields.Hour);
//
//     FatTime.Date.Year          = (USHORT)(TimeFields.Year - 1980);
//     FatTime.Date.Month         = (USHORT)(TimeFields.Month);
//     FatTime.Date.Day           = (USHORT)(TimeFields.Day);
//
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     return FatTime;
// }
//
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
