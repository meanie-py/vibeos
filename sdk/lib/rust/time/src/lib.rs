//! RTL Time Routines — Rust implementation.
//!
//! Replaces `sdk/lib/rtl/time.c` (423 lines).
//! Ten functions: 8 pure arithmetic + 2 with ZwQuerySystemInformation FFI.
//! All architectures.

#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case)]

// ============================================================================
// Constants
// ============================================================================

const TICKSPERSEC: i64 = 10_000_000;
const TICKSPERMSEC: i64 = 10_000;
const SECSPERDAY: i64 = 86400;
const SECSPERHOUR: i64 = 3600;
const SECSPERMIN: i64 = 60;
const EPOCHWEEKDAY: i64 = 1;
const DAYSPERWEEK: i64 = 7;
const EPOCHYEAR: i32 = 1601;
const DAYSPERNORMALYEAR: i32 = 365;
const DAYSPERLEAPYEAR: i32 = 366;
const MONSPERYEAR: usize = 12;

const TICKSTO1970: i64 = 0x019d_b1de_d53e_8000;
const TICKSTO1980: i64 = 0x01a8_e79f_e1d5_8000;

const YEAR_LENGTHS: [i32; 2] = [DAYSPERNORMALYEAR, DAYSPERLEAPYEAR];

const MONTH_LENGTHS: [[u8; MONSPERYEAR]; 2] = [
    [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31],
    [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31],
];

// ============================================================================
// Types
// ============================================================================

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TimeFields {
    pub year: i16,
    pub month: i16,
    pub day: i16,
    pub hour: i16,
    pub minute: i16,
    pub second: i16,
    pub milliseconds: i16,
    pub weekday: i16,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SystemTimeOfDayInformation {
    pub boot_time: i64,
    pub current_time: i64,
    pub time_zone_bias: i64,
    pub time_zone_id: u32,
    pub reserved: u32,
    pub boot_time_bias: u64,
    pub sleep_time_bias: u64,
}

// ============================================================================
// Helpers
// ============================================================================

#[inline]
fn is_leap_year(year: i32) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

#[inline]
fn leap_index(year: i32) -> usize {
    if is_leap_year(year) { 1 } else { 0 }
}

fn days_since_epoch(year: i32) -> i64 {
    let y = (year - 1) as i64;
    let days = y * DAYSPERNORMALYEAR as i64 + y / 4 - y / 100 + y / 400;
    let ey = (EPOCHYEAR - 1) as i64;
    let epoch_days = ey * DAYSPERNORMALYEAR as i64 + ey / 4 - ey / 100 + ey / 400;
    days - epoch_days
}

// ============================================================================
// Pure functions
// ============================================================================

pub fn time_fields_to_time(tf: &TimeFields) -> Option<i64> {
    // Validate fields
    if tf.milliseconds < 0 || tf.milliseconds > 999
        || tf.second < 0 || tf.second > 59
        || tf.minute < 0 || tf.minute > 59
        || tf.hour < 0 || tf.hour > 23
        || tf.month < 1 || tf.month > 12
        || tf.day < 1
        || tf.year < EPOCHYEAR as i16
    {
        return None;
    }

    let li = leap_index(tf.year as i32);
    if tf.day > MONTH_LENGTHS[li][(tf.month - 1) as usize] as i16 {
        return None;
    }

    let mut quad = days_since_epoch(tf.year as i32);
    for m in 1..tf.month as usize {
        quad += MONTH_LENGTHS[li][m - 1] as i64;
    }
    quad += (tf.day - 1) as i64;
    quad *= SECSPERDAY;
    quad += tf.hour as i64 * SECSPERHOUR + tf.minute as i64 * SECSPERMIN + tf.second as i64;
    quad *= TICKSPERSEC;
    quad += tf.milliseconds as i64 * TICKSPERMSEC;

    Some(quad)
}

pub fn time_to_time_fields(time: i64) -> TimeFields {
    let mut tf = TimeFields::default();
    let int_time = time as u64;

    tf.milliseconds = ((int_time % TICKSPERSEC as u64) / TICKSPERMSEC as u64) as i16;
    let int_time = int_time / TICKSPERSEC as u64;

    let mut days = (int_time / SECSPERDAY as u64) as u32;
    let seconds_in_day = (int_time % SECSPERDAY as u64) as u32;

    tf.hour = (seconds_in_day / SECSPERHOUR as u32) as i16;
    let rem = seconds_in_day % SECSPERHOUR as u32;
    tf.minute = (rem / SECSPERMIN as u32) as i16;
    tf.second = (rem % SECSPERMIN as u32) as i16;

    tf.weekday = ((EPOCHWEEKDAY as u32 + days) % DAYSPERWEEK as u32) as i16;

    let mut cur_year = EPOCHYEAR;
    cur_year += days as i32 / DAYSPERLEAPYEAR;
    days -= days_since_epoch(cur_year) as u32;
    loop {
        let li = leap_index(cur_year);
        if days < YEAR_LENGTHS[li] as u32 {
            break;
        }
        cur_year += 1;
        days -= YEAR_LENGTHS[li] as u32;
    }
    tf.year = cur_year as i16;

    let li = leap_index(cur_year);
    let months = &MONTH_LENGTHS[li];
    let mut cur_month = 0usize;
    while days >= months[cur_month] as u32 {
        days -= months[cur_month] as u32;
        cur_month += 1;
    }
    tf.month = (cur_month + 1) as i16;
    tf.day = (days + 1) as i16;

    tf
}

pub fn time_to_elapsed_time_fields(time: i64) -> TimeFields {
    let mut tf = TimeFields::default();

    tf.milliseconds = ((time % TICKSPERSEC) / TICKSPERMSEC) as i16;
    let elapsed_seconds = time as u64 / TICKSPERSEC as u64;
    let seconds_in_day = (elapsed_seconds % SECSPERDAY as u64) as u32;
    let seconds_in_minute = seconds_in_day % SECSPERHOUR as u32;

    tf.hour = (seconds_in_day / SECSPERHOUR as u32) as i16;
    tf.minute = (seconds_in_minute / SECSPERMIN as u32) as i16;
    tf.second = (seconds_in_minute % SECSPERMIN as u32) as i16;
    tf.day = (elapsed_seconds / SECSPERDAY as u64) as i16;
    tf.month = 0;
    tf.year = 0;

    tf
}

pub fn time_to_seconds_since_1970(time: i64) -> Option<u32> {
    let int_time = (time - TICKSTO1970) / TICKSPERSEC;
    // Check if result fits in u32 (high 32 bits must be zero)
    if (int_time >> 32) != 0 {
        return None;
    }
    Some(int_time as u32)
}

pub fn time_to_seconds_since_1980(time: i64) -> Option<u32> {
    let int_time = (time - TICKSTO1980) / TICKSPERSEC;
    if (int_time >> 32) != 0 {
        return None;
    }
    Some(int_time as u32)
}

pub fn seconds_since_1970_to_time(seconds: u32) -> i64 {
    seconds as i64 * TICKSPERSEC + TICKSTO1970
}

pub fn seconds_since_1980_to_time(seconds: u32) -> i64 {
    seconds as i64 * TICKSPERSEC + TICKSTO1980
}

pub fn cutover_time_to_system_time(
    cutover: &TimeFields,
    current_time: i64,
    this_years_cutover_only: bool,
) -> Option<i64> {
    // Fixed cutover time
    if cutover.year != 0 {
        let system_time = time_fields_to_time(cutover)?;
        if system_time < current_time {
            return None;
        }
        return Some(system_time);
    }

    // Recurring cutover time
    if cutover.day == 0 || cutover.day > 5 {
        return None;
    }

    let current_fields = time_to_time_fields(current_time);
    let mut next_years_cutover = false;

    loop {
        let mut adj = TimeFields::default();
        adj.year = current_fields.year;
        if next_years_cutover {
            adj.year += 1;
        }
        adj.month = cutover.month;
        adj.day = 1;
        adj.hour = cutover.hour;
        adj.minute = cutover.minute;
        adj.second = cutover.second;
        adj.milliseconds = cutover.milliseconds;

        let cutover_time = time_fields_to_time(&adj)?;
        let cutover_fields = time_to_time_fields(cutover_time);

        // Adjust to first matching weekday
        if cutover_fields.weekday != cutover.weekday {
            let offset = if cutover_fields.weekday < cutover.weekday {
                cutover.weekday - cutover_fields.weekday
            } else {
                DAYSPERWEEK as i16 - (cutover_fields.weekday - cutover.weekday)
            };
            adj.day += offset;
        }

        // Adjust weeks
        if cutover.day > 1 {
            let extra_days = DAYSPERWEEK as i16 * (cutover.day - 1);
            let month_len = MONTH_LENGTHS[leap_index(adj.year as i32)][(adj.month - 1) as usize] as i16;
            let actual_days = if adj.day + extra_days > month_len {
                extra_days - DAYSPERWEEK as i16
            } else {
                extra_days
            };
            adj.day += actual_days;
        }

        let cutover_system_time = time_fields_to_time(&adj)?;

        if this_years_cutover_only || next_years_cutover || cutover_system_time >= current_time {
            return Some(cutover_system_time);
        }

        next_years_cutover = true;
    }
}

// ============================================================================
// FFI entry points
// ============================================================================

#[cfg(not(test))]
mod ffi {
    use super::*;
    use core::ffi::c_void;

    // SystemTimeOfDayInformation = 3
    const SYSTEM_TIME_OF_DAY_INFORMATION: u32 = 3;

    extern "system" {
        fn ZwQuerySystemInformation(
            system_information_class: u32,
            system_information: *mut c_void,
            system_information_length: u32,
            return_length: *mut u32,
        ) -> i32;
    }

    #[inline]
    fn nt_success(status: i32) -> bool {
        status >= 0
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlTimeFieldsToTime(
        time_fields: *const TimeFields,
        time: *mut i64,
    ) -> u8 {
        match time_fields_to_time(&*time_fields) {
            Some(t) => { *time = t; 1 }
            None => 0,
        }
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlTimeToTimeFields(
        time: *const i64,
        time_fields: *mut TimeFields,
    ) {
        *time_fields = time_to_time_fields(*time);
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlTimeToElapsedTimeFields(
        time: *const i64,
        time_fields: *mut TimeFields,
    ) {
        *time_fields = time_to_elapsed_time_fields(*time);
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlTimeToSecondsSince1970(
        time: *const i64,
        seconds: *mut u32,
    ) -> u8 {
        match time_to_seconds_since_1970(*time) {
            Some(s) => { *seconds = s; 1 }
            None => 0,
        }
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlTimeToSecondsSince1980(
        time: *const i64,
        seconds: *mut u32,
    ) -> u8 {
        match time_to_seconds_since_1980(*time) {
            Some(s) => { *seconds = s; 1 }
            None => 0,
        }
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlSecondsSince1970ToTime(
        seconds: u32,
        time: *mut i64,
    ) {
        *time = seconds_since_1970_to_time(seconds);
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlSecondsSince1980ToTime(
        seconds: u32,
        time: *mut i64,
    ) {
        *time = seconds_since_1980_to_time(seconds);
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlLocalTimeToSystemTime(
        local_time: *const i64,
        system_time: *mut i64,
    ) -> i32 {
        let mut info: SystemTimeOfDayInformation = core::mem::zeroed();
        let status = ZwQuerySystemInformation(
            SYSTEM_TIME_OF_DAY_INFORMATION,
            &mut info as *mut _ as *mut c_void,
            core::mem::size_of::<SystemTimeOfDayInformation>() as u32,
            core::ptr::null_mut(),
        );
        if !nt_success(status) {
            return status;
        }
        *system_time = *local_time + info.time_zone_bias;
        0 // STATUS_SUCCESS
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlSystemTimeToLocalTime(
        system_time: *const i64,
        local_time: *mut i64,
    ) -> i32 {
        let mut info: SystemTimeOfDayInformation = core::mem::zeroed();
        let status = ZwQuerySystemInformation(
            SYSTEM_TIME_OF_DAY_INFORMATION,
            &mut info as *mut _ as *mut c_void,
            core::mem::size_of::<SystemTimeOfDayInformation>() as u32,
            core::ptr::null_mut(),
        );
        if !nt_success(status) {
            return status;
        }
        *local_time = *system_time - info.time_zone_bias;
        0 // STATUS_SUCCESS
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlCutoverTimeToSystemTime(
        cutover_time: *mut TimeFields,
        system_time: *mut i64,
        current_time: *const i64,
        this_years_cutover_only: u8,
    ) -> u8 {
        match cutover_time_to_system_time(&*cutover_time, *current_time, this_years_cutover_only != 0) {
            Some(t) => { *system_time = t; 1 }
            None => 0,
        }
    }
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_epoch_start() {
        // January 1, 1601 00:00:00.000 should be time 0
        let tf = TimeFields {
            year: 1601, month: 1, day: 1,
            hour: 0, minute: 0, second: 0, milliseconds: 0,
            weekday: 0,
        };
        assert_eq!(time_fields_to_time(&tf), Some(0));
    }

    #[test]
    fn test_roundtrip_known_date() {
        // 2000-01-01 12:00:00.000
        let tf = TimeFields {
            year: 2000, month: 1, day: 1,
            hour: 12, minute: 0, second: 0, milliseconds: 0,
            weekday: 0,
        };
        let time = time_fields_to_time(&tf).unwrap();
        let result = time_to_time_fields(time);
        assert_eq!(result.year, 2000);
        assert_eq!(result.month, 1);
        assert_eq!(result.day, 1);
        assert_eq!(result.hour, 12);
        assert_eq!(result.minute, 0);
        assert_eq!(result.second, 0);
        assert_eq!(result.milliseconds, 0);
        assert_eq!(result.weekday, 6); // Saturday
    }

    #[test]
    fn test_leap_year_feb29() {
        let tf = TimeFields {
            year: 2000, month: 2, day: 29,
            hour: 0, minute: 0, second: 0, milliseconds: 0,
            weekday: 0,
        };
        assert!(time_fields_to_time(&tf).is_some());

        // 2001 is not a leap year
        let tf2 = TimeFields {
            year: 2001, month: 2, day: 29,
            hour: 0, minute: 0, second: 0, milliseconds: 0,
            weekday: 0,
        };
        assert!(time_fields_to_time(&tf2).is_none());
    }

    #[test]
    fn test_1970_epoch() {
        // TICKSTO1970 should correspond to 1970-01-01 00:00:00
        let tf = time_to_time_fields(TICKSTO1970);
        assert_eq!(tf.year, 1970);
        assert_eq!(tf.month, 1);
        assert_eq!(tf.day, 1);
        assert_eq!(tf.hour, 0);
    }

    #[test]
    fn test_seconds_since_1970_roundtrip() {
        let secs = 1_000_000_000u32; // ~2001-09-09
        let time = seconds_since_1970_to_time(secs);
        let back = time_to_seconds_since_1970(time).unwrap();
        assert_eq!(back, secs);
    }

    #[test]
    fn test_seconds_since_1980_roundtrip() {
        let secs = 500_000_000u32;
        let time = seconds_since_1980_to_time(secs);
        let back = time_to_seconds_since_1980(time).unwrap();
        assert_eq!(back, secs);
    }

    #[test]
    fn test_time_before_1970_fails() {
        // Time before 1970 should fail for seconds_since_1970
        let tf = TimeFields {
            year: 1960, month: 1, day: 1,
            hour: 0, minute: 0, second: 0, milliseconds: 0,
            weekday: 0,
        };
        let time = time_fields_to_time(&tf).unwrap();
        assert!(time_to_seconds_since_1970(time).is_none());
    }

    #[test]
    fn test_invalid_fields() {
        // Invalid month
        let tf = TimeFields { year: 2000, month: 13, day: 1, ..Default::default() };
        assert!(time_fields_to_time(&tf).is_none());

        // Invalid day
        let tf = TimeFields { year: 2000, month: 1, day: 32, ..Default::default() };
        assert!(time_fields_to_time(&tf).is_none());

        // Year before epoch
        let tf = TimeFields { year: 1600, month: 1, day: 1, ..Default::default() };
        assert!(time_fields_to_time(&tf).is_none());
    }

    #[test]
    fn test_elapsed_time_fields() {
        // 1 day + 2 hours + 3 minutes + 4 seconds + 500 ms
        let time = (SECSPERDAY + 2 * SECSPERHOUR + 3 * SECSPERMIN + 4) * TICKSPERSEC + 500 * TICKSPERMSEC;
        let tf = time_to_elapsed_time_fields(time);
        assert_eq!(tf.day, 1);
        assert_eq!(tf.hour, 2);
        assert_eq!(tf.minute, 3);
        assert_eq!(tf.second, 4);
        assert_eq!(tf.milliseconds, 500);
        assert_eq!(tf.month, 0);
        assert_eq!(tf.year, 0);
    }

    #[test]
    fn test_cutover_fixed() {
        // Fixed cutover: 2000-06-15 02:00:00
        let cutover = TimeFields {
            year: 2000, month: 6, day: 15,
            hour: 2, minute: 0, second: 0, milliseconds: 0,
            weekday: 0,
        };
        let current = time_fields_to_time(&TimeFields {
            year: 2000, month: 1, day: 1,
            hour: 0, minute: 0, second: 0, milliseconds: 0,
            weekday: 0,
        }).unwrap();
        let result = cutover_time_to_system_time(&cutover, current, false).unwrap();
        let tf = time_to_time_fields(result);
        assert_eq!(tf.year, 2000);
        assert_eq!(tf.month, 6);
        assert_eq!(tf.day, 15);
    }
}
