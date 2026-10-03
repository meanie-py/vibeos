//! `cprintf.cpp`: the six `__conio_common_vc(w)printf(_s/_p)` entry points --
//! `crate::output`'s processor over `console_output_adapter`, which writes
//! each character with `_putch_nolock_internal` / `_putwch_nolock_internal`.
//!
//! Like the C there is NO conio lock here (the public `_cprintf` inlines call
//! these directly), and the adapter never fails validation.

use core::ffi::c_int;

use super::putch::{_putch_nolock_internal, _putwch_nolock_internal};
use super::{EILSEQ, EOF, WEOF};
use crate::locale::CrtLocalePointers;
use crate::output::{Adapter, Base, OutChar, Processor};
use crate::ptd_host::{Cached, PtdHost};

/// `__acrt_stdio_char_traits<Character>::puttch_nolock_internal` != eof.
pub(crate) trait ConsoleChar: OutChar {
    unsafe fn put(self, host: &mut PtdHost) -> bool;
}

impl ConsoleChar for u8 {
    unsafe fn put(self, host: &mut PtdHost) -> bool {
        // `char` to an `int` parameter: sign-extended, as in the C.
        _putch_nolock_internal(self as i8 as c_int, host) != EOF
    }
}

impl ConsoleChar for u16 {
    unsafe fn put(self, host: &mut PtdHost) -> bool {
        _putwch_nolock_internal(self, host) != WEOF
    }
}

/// `console_output_adapter<Character>`.
pub(crate) struct ConsoleAdapter;

impl<C: ConsoleChar> Adapter<C> for ConsoleAdapter {
    unsafe fn validate(&self, _host: &mut PtdHost) -> bool {
        true
    }

    /// `output_adapter_common::write_character`.
    unsafe fn write_character(&self, c: C, count_written: &mut i32, host: &mut PtdHost) {
        if c.put(host) {
            *count_written += 1;
        } else {
            *count_written = -1;
        }
    }

    /// `write_string_impl`, under its errno guard: an EILSEQ is recoverable
    /// (write '?' and carry on), anything else stops with -1.
    unsafe fn write_string(&self, string: *const C, length: i32, count_written: &mut i32, host: &mut PtdHost) {
        let reset_errno: Cached<c_int> = *host.errno();
        let mut i = 0;
        while i < length {
            if (*string.add(i as usize)).put(host) {
                *count_written += 1;
            } else {
                if !host.errno().check(EILSEQ) {
                    *count_written = -1;
                    break;
                }
                <Self as Adapter<C>>::write_character(self, C::of(b'?' as u32), count_written, host);
            }
            i += 1;
        }
        *host.errno() = reset_errno;
    }
}

/// `common_vcprintf`.
unsafe fn common_vcprintf<C: ConsoleChar>(
    base: Base,
    options: u64,
    format: *const C,
    locale: *mut CrtLocalePointers,
    arglist: *mut u8,
) -> c_int {
    let mut host = PtdHost::new(locale);
    Processor::<C, ConsoleAdapter>::new(ConsoleAdapter, options, format, &mut host, arglist, base).process()
}

macro_rules! vcprintf {
    ($name:ident, $c:ty, $base:expr) => {
        #[no_mangle]
        pub unsafe extern "C" fn $name(
            options: u64,
            format: *const $c,
            locale: *mut CrtLocalePointers,
            arglist: *mut u8,
        ) -> c_int {
            common_vcprintf::<$c>($base, options, format, locale, arglist)
        }
    };
}

vcprintf!(__conio_common_vcprintf, u8, Base::Standard);
vcprintf!(__conio_common_vcprintf_s, u8, Base::Validation);
vcprintf!(__conio_common_vcprintf_p, u8, Base::Positional);
vcprintf!(__conio_common_vcwprintf, u16, Base::Standard);
vcprintf!(__conio_common_vcwprintf_s, u16, Base::Validation);
vcprintf!(__conio_common_vcwprintf_p, u16, Base::Positional);
