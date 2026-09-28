//! `cscanf.cpp`: `__conio_common_vcscanf`/`__conio_common_vcwscanf` --
//! `crate::stdio::input`'s processor over `console_input_adapter`, which reads
//! with `_getche_nolock`/`_getwche_nolock` (so input is ECHOED, and a pushed
//! back character is consumed first, unechoed) and ungets with
//! `_ungetch_nolock`/`_ungetwch_nolock` -- which can FAIL (a full narrow
//! push-back, an occupied wide one), silently losing the character.
//!
//! No conio lock is taken, as in the C.

use core::ffi::c_int;

use super::getch::{_getche_nolock, _ungetch_nolock};
use super::getwch::{_getwche_nolock, _ungetwch_nolock};
use crate::locale::CrtLocalePointers;
use crate::locale_update::LocaleUpdate;
use crate::stdio::input::engine::InputProcessor;
use crate::stdio::input::{Adapter, Char};

/// `console_input_adapter<Character>`.
pub(crate) struct ConsoleInputAdapter<C> {
    characters_read: u64,
    _marker: core::marker::PhantomData<C>,
}

impl<C> ConsoleInputAdapter<C> {
    fn new() -> Self {
        Self { characters_read: 0, _marker: core::marker::PhantomData }
    }
}

impl Adapter<u8> for ConsoleInputAdapter<u8> {
    unsafe fn validate(&mut self) -> bool {
        true
    }
    unsafe fn get(&mut self) -> i32 {
        let c = _getche_nolock();
        if c != u8::EOF_CODE {
            self.characters_read += 1;
        }
        c
    }
    unsafe fn unget(&mut self, c: i32) {
        if c == u8::EOF_CODE {
            return;
        }
        self.characters_read -= 1;
        _ungetch_nolock(c);
    }
    fn characters_read(&self) -> u64 {
        self.characters_read
    }
}

impl Adapter<u16> for ConsoleInputAdapter<u16> {
    unsafe fn validate(&mut self) -> bool {
        true
    }
    unsafe fn get(&mut self) -> i32 {
        let c = _getwche_nolock() as i32;
        if c != u16::EOF_CODE {
            self.characters_read += 1;
        }
        c
    }
    unsafe fn unget(&mut self, c: i32) {
        if c == u16::EOF_CODE {
            return;
        }
        self.characters_read -= 1;
        _ungetwch_nolock(c as u16);
    }
    fn characters_read(&self) -> u64 {
        self.characters_read
    }
}

/// `common_cscanf`.
unsafe fn common_cscanf<C: Char>(
    options: u64,
    format: *const C,
    locale: *mut CrtLocalePointers,
    arglist: *mut u8,
) -> c_int
where
    ConsoleInputAdapter<C>: Adapter<C>,
{
    let mut locale_update = LocaleUpdate::new(locale);
    let mut processor =
        InputProcessor::new(ConsoleInputAdapter::<C>::new(), options, format, locale_update.locale_t(), arglist);
    processor.process()
}

#[no_mangle]
pub unsafe extern "C" fn __conio_common_vcscanf(
    options: u64,
    format: *const u8,
    locale: *mut CrtLocalePointers,
    arglist: *mut u8,
) -> c_int {
    common_cscanf::<u8>(options, format, locale, arglist)
}

#[no_mangle]
pub unsafe extern "C" fn __conio_common_vcwscanf(
    options: u64,
    format: *const u16,
    locale: *mut CrtLocalePointers,
    arglist: *mut u8,
) -> c_int {
    common_cscanf::<u16>(options, format, locale, arglist)
}
