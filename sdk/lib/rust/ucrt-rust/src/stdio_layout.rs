//! The two stdio/lowio records the formatted-output family reads directly:
//! UCRT's `FILE` (`__crt_stdio_stream_data`) and a lowio handle
//! (`__crt_lowio_handle_data`).
//!
//! Only the fields a caller here actually reads are named; the rest is carried
//! as padding of the measured size, so a wrong guess cannot hide behind a field
//! nobody touches. Offsets were measured with the build's own compiler flags on
//! all three targets and are asserted on every build, for the same reason as
//! `ptd.rs`: the headers they came from are vendored source that is going away.

use core::ffi::{c_char, c_int, c_long};

/// `_IOSTRING` (corecrt_internal_stdio.h): the stream is backed by a caller's
/// string, not a file -- `sprintf`'s fake FILE, and `_vsnprintf(NULL, 0, ...)`'s
/// counting-only one, which has a null `_base`.
pub const IOSTRING: c_long = 0x1000;

/// `__crt_stdio_stream_data`, through its public `FILE*`.
#[repr(C)]
pub struct StreamData {
    pub ptr: *mut c_char,
    pub base: *mut c_char,
    pub cnt: c_int,
    pub flags: c_long,
    pub file: c_long,
}

impl StreamData {
    /// `__crt_stdio_stream::is_string_backed()`.
    pub fn is_string_backed(&self) -> bool {
        self.flags & IOSTRING != 0
    }
}

macro_rules! pin_stream {
    ($field:ident, $want:expr) => {
        const _: () = assert!(
            core::mem::offset_of!(StreamData, $field) == $want,
            concat!("__crt_stdio_stream_data.", stringify!($field), " is not where UCRT puts it"),
        );
    };
}

#[cfg(target_pointer_width = "32")]
mod pinned_stream {
    use super::*;
    pin_stream!(ptr, 0);
    pin_stream!(base, 4);
    pin_stream!(cnt, 8);
    pin_stream!(flags, 12);
    pin_stream!(file, 16);
}

#[cfg(target_pointer_width = "64")]
mod pinned_stream {
    use super::*;
    pin_stream!(ptr, 0);
    pin_stream!(base, 8);
    pin_stream!(cnt, 16);
    pin_stream!(flags, 20);
    pin_stream!(file, 24);
}

/// `__crt_lowio_text_mode::ansi` -- the other two are utf8 (1) and utf16le (2).
pub const TEXT_MODE_ANSI: u8 = 0;

/// `IOINFO_L2E`: each `__pioinfo` array holds `1 << 6` handles.
const IOINFO_L2E: c_int = 6;
const IOINFO_ARRAY_ELTS: c_int = 1 << IOINFO_L2E;

/// Measured `sizeof(__crt_lowio_handle_data)` and the offsets of the two
/// fields read here. `unicode` is bit 0 of the bitfield byte that follows the
/// three-byte `_pipe_lookahead` (checked by compiling `p->unicode = 1`, which
/// the build's compiler lowers to `orb $1, 45(%eax)` on i386).
mod handle {
    const WIDE: bool = cfg!(target_pointer_width = "64");
    pub const HANDLE_DATA_SIZE: usize = if WIDE { 72 } else { 56 };
    pub const HANDLE_TEXTMODE_OFFSET: usize = if WIDE { 57 } else { 41 };
    pub const HANDLE_UNICODE_BYTE: usize = if WIDE { 61 } else { 45 };
}

/// `_osfile(fh)` / `_osfhnd(fh)`: the UNCHECKED accessors, for a descriptor
/// the caller has already range-checked against `_nhandle`. `osfhnd` follows
/// the `CRITICAL_SECTION` (24 / 40 bytes); `osfile` follows the 8-aligned
/// `startpos`, one byte before `textmode`.
pub mod lowio {
    use super::{handle, IOINFO_ARRAY_ELTS, IOINFO_L2E, __pioinfo};
    use core::ffi::c_int;

    const WIDE: bool = cfg!(target_pointer_width = "64");
    const HANDLE_OSFHND_OFFSET: usize = if WIDE { 40 } else { 24 };
    const HANDLE_OSFILE_OFFSET: usize = if WIDE { 56 } else { 40 };
    const _: () = assert!(HANDLE_OSFILE_OFFSET + 1 == handle::HANDLE_TEXTMODE_OFFSET);

    unsafe fn pioinfo(fh: c_int) -> *const u8 {
        let array = __pioinfo[(fh >> IOINFO_L2E) as usize];
        array.add((fh & (IOINFO_ARRAY_ELTS - 1)) as usize * handle::HANDLE_DATA_SIZE)
    }

    pub unsafe fn osfile(fh: c_int) -> u8 {
        *pioinfo(fh).add(HANDLE_OSFILE_OFFSET)
    }

    pub unsafe fn osfhnd(fh: c_int) -> isize {
        (pioinfo(fh).add(HANDLE_OSFHND_OFFSET) as *const isize).read()
    }
}

extern "C" {
    /// `__crt_lowio_handle_data* __pioinfo[IOINFO_ARRAYS]` (lowio/ioinit.cpp).
    static __pioinfo: [*mut u8; 128];
    /// The record `_pioinfo_safe` substitutes for handles -1 and -2.
    static __badioinfo: u8;
}

/// `_pioinfo_safe(fh)`: the handle record, or `__badioinfo` for -1 and -2.
/// Like the C, it does not range-check any other value.
unsafe fn pioinfo_safe(fh: c_int) -> *const u8 {
    if fh == -1 || fh == -2 {
        return core::ptr::addr_of!(__badioinfo);
    }
    let array = __pioinfo[(fh >> IOINFO_L2E) as usize];
    array.add((fh & (IOINFO_ARRAY_ELTS - 1)) as usize * handle::HANDLE_DATA_SIZE)
}

/// `_textmode_safe(fh)`.
pub unsafe fn textmode_safe(fh: c_int) -> u8 {
    *pioinfo_safe(fh).add(handle::HANDLE_TEXTMODE_OFFSET)
}

/// `_tm_unicode_safe(fh)`.
pub unsafe fn tm_unicode_safe(fh: c_int) -> bool {
    *pioinfo_safe(fh).add(handle::HANDLE_UNICODE_BYTE) & 1 != 0
}
