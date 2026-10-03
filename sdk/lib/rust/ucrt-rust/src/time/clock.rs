//! time/clock.cpp: `clock`, `__acrt_initialize_clock`.
//!
//! ## Details that are silent if wrong
//! * **`start_count`/`source_frequency` are set to `-1` (not `0`) on ANY
//!   `QueryPerformance*` failure**, including a reported frequency of
//!   exactly `0` -- `clock()` checks `start_count == -1` as its own failure
//!   sentinel, so this is load-bearing, not just a diagnostic value.
//! * **`scale_count` avoids overflow by splitting the scaling into a
//!   whole-seconds term and a remainder term**, each independently
//!   multiplied by `CLOCKS_PER_SEC` before dividing -- not a single
//!   `count * CLOCKS_PER_SEC / source_frequency`, which the C's own comment
//!   says would overflow for a long-running process on a high-frequency
//!   counter.
//! * **A negative `current - start_count` (clock going backwards) and a
//!   scaled result exceeding `LONG_MAX` both return `-1`**, per C11
//!   7.27.2.1/3 for the latter.
//! * **`__acrt_initialize_clock` is registered via `_CRT_LINKER_FORCE_INCLUDE`**
//!   (`force_include!` here) so it still runs even though nothing calls it
//!   by name -- deleting this file without keeping the force-include would
//!   silently leave `start_count`/`source_frequency` at their zero
//!   defaults, making every `clock()` call return `-1` forever (0 divides
//!   by zero in `scale_count`, avoided only because `start_count == -1`
//!   short-circuits first).

use core::ffi::c_long;

crate::force_include!(__acrt_clock_initializer);

const CLOCKS_PER_SEC: i64 = 1000;
const LONG_MAX: i64 = 0x7fff_ffff;

extern "system" {
    fn QueryPerformanceFrequency(freq: *mut i64) -> i32;
    fn QueryPerformanceCounter(count: *mut i64) -> i32;
}

static mut SOURCE_FREQUENCY: i64 = 0;
static mut START_COUNT: i64 = 0;

fn scale_count(mut count: i64, source_frequency: i64) -> i64 {
    let mut scaled_count = (count / source_frequency) * CLOCKS_PER_SEC;
    count %= source_frequency;
    scaled_count += (count * CLOCKS_PER_SEC) / source_frequency;
    scaled_count
}

#[no_mangle]
pub unsafe extern "C" fn __acrt_initialize_clock() -> core::ffi::c_int {
    let mut local_frequency: i64 = 0;
    let mut local_start_count: i64 = 0;
    if QueryPerformanceFrequency(&mut local_frequency) == 0 || QueryPerformanceCounter(&mut local_start_count) == 0 || local_frequency == 0 {
        SOURCE_FREQUENCY = -1;
        START_COUNT = -1;
        return 0;
    }

    SOURCE_FREQUENCY = local_frequency;
    START_COUNT = local_start_count;
    0
}

#[no_mangle]
pub unsafe extern "C" fn clock() -> c_long {
    if START_COUNT == -1 {
        return -1;
    }

    let mut current_count: i64 = 0;
    if QueryPerformanceCounter(&mut current_count) == 0 {
        return -1;
    }

    let result = current_count - START_COUNT;
    if result < 0 {
        return -1;
    }

    let scaled_result = scale_count(result, SOURCE_FREQUENCY);
    if scaled_result > LONG_MAX {
        return -1;
    }

    scaled_result as c_long
}
