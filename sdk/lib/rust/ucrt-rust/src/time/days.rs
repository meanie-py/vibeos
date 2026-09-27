//! time/days.cpp: the `_days` and `_lpdays` tables.
//!
//! Internal-linkage in the C's own translation unit is irrelevant here --
//! these are plain `extern "C" int const[]` at namespace scope, so they have
//! external linkage and are read by exact name from `gmtime.cpp` (ported in
//! `super::gmtime`), `mktime.cpp` (`super::mktime`), `loctotime.cpp`
//! (`super::loctotime`) and `tzset.cpp`'s `cvtdate` (`super::tzset`) -- all in
//! this same batch, so nothing outside `time/` reaches them.
//!
//! Byte/relocation-identical to the C's arrays: both are `.rdata`, no
//! relocations (plain `int` literals), so `tools/compare_coff_data.py` need
//! only compare the 13 x 4 = 52 bytes of each.

use core::ffi::c_int;

/// Days elapsed from the beginning of a (non-leap) year to the beginning of
/// each month, one-based index by month (`_days[0]` is unused/-1 sentinel,
/// `_days[mo]` for one-based `mo` in `loctotime.cpp`; zero-based `_days[mo]`
/// for `tb->tm_mon` in `mktime.cpp`/`gmtime.cpp`).
#[no_mangle]
pub static _days: [c_int; 13] = [-1, 30, 58, 89, 119, 150, 180, 211, 242, 272, 303, 333, 364];

/// Same, for a leap year.
#[no_mangle]
pub static _lpdays: [c_int; 13] = [-1, 30, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334, 365];
