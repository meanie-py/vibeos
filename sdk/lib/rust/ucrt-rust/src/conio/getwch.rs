//! `getwch.cpp`: `_getwch`/`_getwche` (read one key as a wide character,
//! without/with echo) and `_ungetwch`. The wide push-back is ONE character,
//! separate from `_getch`'s, and keeps only the low byte of what is pushed.

use core::ptr::{addr_of, addr_of_mut};

use super::getch::_getextendedkeycode;
use super::initconin::{
    __dcrt_get_input_console_mode, __dcrt_lowio_ensure_console_input_initialized, __dcrt_read_console_input,
    __dcrt_set_input_console_mode,
};
use super::putch::_putwch_nolock;
use super::{with_conio_lock, InputRecord, EOF, KEY_EVENT, WEOF};

/// `static wint_t wchbuf = WEOF;`
static mut WCHBUF: u16 = WEOF;

unsafe fn wchbuf() -> u16 {
    *addr_of!(WCHBUF)
}

#[no_mangle]
pub unsafe extern "C" fn _getwch() -> u16 {
    with_conio_lock(|| unsafe { _getwch_nolock() })
}

#[no_mangle]
pub unsafe extern "C" fn _getwche() -> u16 {
    with_conio_lock(|| unsafe { _getwche_nolock() })
}

/// The read loop of `_getwch_nolock`, between the mode switch and its restore.
unsafe fn getwch_read_loop() -> u16 {
    loop {
        let mut input_record: InputRecord = core::mem::zeroed();
        let mut num_read: u32 = 0;
        if __dcrt_read_console_input(&mut input_record, 1, &mut num_read) == 0 {
            return WEOF;
        }
        if num_read == 0 {
            return WEOF;
        }

        if input_record.event_type == KEY_EVENT && input_record.event.key_event.key_down != 0 {
            let c = input_record.event.key_event.u_char;
            if c != 0 {
                return c;
            }

            let cp = _getextendedkeycode(&mut input_record.event.key_event);
            if !cp.is_null() {
                *addr_of_mut!(WCHBUF) = (*cp).second_char as u16;
                return (*cp).lead_char as u16;
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn _getwch_nolock() -> u16 {
    // A pushed-back character first, without echo.
    if wchbuf() != WEOF {
        let buffered_wchar = wchbuf() & 0xFFFF;
        *addr_of_mut!(WCHBUF) = WEOF;
        return buffered_wchar;
    }

    if __dcrt_lowio_ensure_console_input_initialized() == 0 {
        return WEOF;
    }

    // Raw mode (see `_getch_nolock` for the failed-query note).
    let mut old_console_mode: u32 = 0;
    __dcrt_get_input_console_mode(&mut old_console_mode);
    __dcrt_set_input_console_mode(0);

    let result = getwch_read_loop();

    __dcrt_set_input_console_mode(old_console_mode);
    result
}

#[no_mangle]
pub unsafe extern "C" fn _getwche_nolock() -> u16 {
    if wchbuf() != WEOF {
        let buffered_wchar = wchbuf() & 0xFFFF;
        *addr_of_mut!(WCHBUF) = WEOF;
        return buffered_wchar;
    }

    let gotten_wchar = _getwch_nolock();
    if gotten_wchar == WEOF {
        return WEOF;
    }

    if _putwch_nolock(gotten_wchar) == WEOF {
        return WEOF;
    }

    gotten_wchar
}

#[no_mangle]
pub unsafe extern "C" fn _ungetwch(c: u16) -> u16 {
    with_conio_lock(|| unsafe { _ungetwch_nolock(c) })
}

#[no_mangle]
pub unsafe extern "C" fn _ungetwch_nolock(c: u16) -> u16 {
    // `static_cast<wint_t>(EOF)` is 0xFFFF.
    if c == WEOF || wchbuf() != WEOF {
        return EOF as u16;
    }

    // Only the low byte is kept.
    *addr_of_mut!(WCHBUF) = c & 0xFF;
    wchbuf()
}
