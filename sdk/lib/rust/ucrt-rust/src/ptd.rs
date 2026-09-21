//! UCRT's per-thread data — `__acrt_ptd`.
//!
//! The root dependency under most of the remaining work, which is why it is
//! pinned early. Nearly every locale-aware CRT function reaches this struct:
//! `_LocaleUpdate` with a NULL `_locale_t` resolves the *current thread's*
//! locale through it, so even the non-`_l` spellings of `_ismbb*`, `strtod`,
//! `_mbs*` and the rest are downstream of it, not just the `_l` twins.
//!
//! It is also where `_thread_local_iph` lives, which is the slot the
//! thread-local half of the invalid-parameter handler needs — that work is
//! parked on a Rust TLS mechanism this tree does not have yet, and this layout
//! is half of what unparks it.
//!
//! Same argument as `locale.rs` for why the numbers are here rather than
//! derived on demand: `sdk/lib/ucrt/inc/corecrt_internal.h` is vendored Windows
//! SDK source and is going away, and after it goes nothing in the tree can
//! re-derive them. Measured with the build's own compiler flags, pinned as
//! `const` assertions so they are checked on every build of every arch. The
//! probe added 1_000_000 to each value before recording it, so that a
//! genuinely-zero offset lands in `.data` and is readable rather than being
//! optimised into `.bss` and silently missing from the output.
//!
//! WE DO NOT IMPLEMENT ANY OF THIS YET. It is layout only: no `#[no_mangle]`,
//! so it takes over nothing and needs no entry in `tools/ucrt-takeover.yaml`.

use core::ffi::{c_int, c_long, c_uint, c_ushort, c_void};

/// `MB_LEN_MAX` — the `_putch_buffer` length, confirmed by the measured gap
/// between `_putch_buffer` and `_putch_buffer_used` (6 bytes, 5 used + 1 pad).
pub const MB_LEN_MAX: usize = 5;

/// `__crt_qualified_locale_data`, embedded BY VALUE and deliberately opaque.
///
/// Its internals are not needed by anything we are implementing, and pinning a
/// layout nobody reads is a maintenance cost with no benefit — but its SIZE is
/// load-bearing, because every field after it shifts if it is wrong. So it is
/// carried as a pointer-aligned block of exactly the measured width: 764 bytes
/// on 32-bit (191 words), 776 on 64-bit (97 words). Both divide exactly, which
/// is itself a check on the measurement.
#[cfg(target_pointer_width = "32")]
pub type QualifiedLocaleData = [usize; 191];
#[cfg(target_pointer_width = "64")]
pub type QualifiedLocaleData = [usize; 97];

/// UCRT's `struct __acrt_ptd` (`sdk/lib/ucrt/inc/corecrt_internal.h:864`).
///
/// NOT interchangeable with msvcrt's per-thread block. Our Rust msvcrt has its
/// own thread data behind `msvcrt_get_thread_data()`
/// (`sdk/lib/rust/msvcrt-crt-rust/src/msvcrt/`), shaped to ReactOS's msvcrt.dll;
/// this is UCRT's, and the two agree on very little beyond both having an
/// errno. Reading one through the other's layout is the same silent-wrong-answer
/// class as the `_locale_t` mismatch documented in `locale.rs`.
#[repr(C)]
pub struct AcrtPtd {
    pub pxcptacttab: *mut c_void,
    pub tpxcptinfoptrs: *mut c_void,
    pub tfpecode: c_int,
    pub terminate: Option<unsafe extern "C" fn()>,
    pub terrno: c_int,
    pub tdoserrno: c_long,
    pub rand_state: c_uint,
    pub strtok_token: *mut u8,
    pub mbstok_token: *mut u8,
    pub wcstok_token: *mut u16,
    pub tmpnam_narrow_buffer: *mut u8,
    pub tmpnam_wide_buffer: *mut u16,
    pub asctime_buffer: *mut u8,
    pub wasctime_buffer: *mut u16,
    pub gmtime_buffer: *mut c_void,
    pub cvtbuf: *mut u8,
    pub strerror_buffer: *mut u8,
    pub wcserror_buffer: *mut u16,
    pub multibyte_info: *mut crate::locale::CrtMultibyteData,
    pub locale_info: *mut crate::locale::CrtLocaleData,
    pub setloc_data: QualifiedLocaleData,
    pub setloc_downlevel_data: *mut c_void,
    pub own_locale: c_int,
    pub putch_buffer: [u8; MB_LEN_MAX],
    pub putch_buffer_used: c_ushort,
    pub thread_local_iph: *mut c_void,
    pub beginthread_context: *mut c_void,
}

impl AcrtPtd {
    /// Accessors used by `locale_update`; named to make the read sites read
    /// like the C they mirror rather than like field pokes.
    pub fn _locale_info_ptr(&self) -> *mut crate::locale::CrtLocaleData {
        self.locale_info
    }
    pub fn _multibyte_info_ptr(&self) -> *mut crate::locale::CrtMultibyteData {
        self.multibyte_info
    }
}

macro_rules! pin {
    ($field:ident, $want:expr) => {
        const _: () = assert!(
            core::mem::offset_of!(AcrtPtd, $field) == $want,
            concat!("__acrt_ptd.", stringify!($field), " is not where UCRT puts it"),
        );
    };
}

#[cfg(target_pointer_width = "32")]
mod pinned {
    use super::*;
    pin!(pxcptacttab, 0);
    pin!(tpxcptinfoptrs, 4);
    pin!(tfpecode, 8);
    pin!(terminate, 12);
    pin!(terrno, 16);
    pin!(tdoserrno, 20);
    pin!(rand_state, 24);
    pin!(strtok_token, 28);
    pin!(mbstok_token, 32);
    pin!(wcstok_token, 36);
    pin!(tmpnam_narrow_buffer, 40);
    pin!(tmpnam_wide_buffer, 44);
    pin!(asctime_buffer, 48);
    pin!(wasctime_buffer, 52);
    pin!(gmtime_buffer, 56);
    pin!(cvtbuf, 60);
    pin!(strerror_buffer, 64);
    pin!(wcserror_buffer, 68);
    pin!(multibyte_info, 72);
    pin!(locale_info, 76);
    pin!(setloc_data, 80);
    pin!(setloc_downlevel_data, 844);
    pin!(own_locale, 848);
    pin!(putch_buffer, 852);
    pin!(putch_buffer_used, 858);
    pin!(thread_local_iph, 860);
    pin!(beginthread_context, 864);
    const _: () = assert!(core::mem::size_of::<AcrtPtd>() == 868);
}

#[cfg(target_pointer_width = "64")]
mod pinned {
    use super::*;
    pin!(pxcptacttab, 0);
    pin!(tpxcptinfoptrs, 8);
    pin!(tfpecode, 16);
    pin!(terminate, 24);
    pin!(terrno, 32);
    pin!(tdoserrno, 36);
    pin!(rand_state, 40);
    pin!(strtok_token, 48);
    pin!(mbstok_token, 56);
    pin!(wcstok_token, 64);
    pin!(tmpnam_narrow_buffer, 72);
    pin!(tmpnam_wide_buffer, 80);
    pin!(asctime_buffer, 88);
    pin!(wasctime_buffer, 96);
    pin!(gmtime_buffer, 104);
    pin!(cvtbuf, 112);
    pin!(strerror_buffer, 120);
    pin!(wcserror_buffer, 128);
    pin!(multibyte_info, 136);
    pin!(locale_info, 144);
    pin!(setloc_data, 152);
    pin!(setloc_downlevel_data, 928);
    pin!(own_locale, 936);
    pin!(putch_buffer, 940);
    pin!(putch_buffer_used, 946);
    pin!(thread_local_iph, 952);
    pin!(beginthread_context, 960);
    const _: () = assert!(core::mem::size_of::<AcrtPtd>() == 968);
}
