//! `lowio/read.cpp`: `_read`, `_read_nolock`, and the `_lookuptrailbytes`
//! table (exported data: the `_utf8_no_of_trailbytes` inline in
//! `corecrt_internal_lowio.h` indexes it, and stdio's ftell reads it by name).
//!
//! Details that are silent if wrong:
//! * Text mode maps CRLF to LF IN PLACE and stops at Ctrl-Z. For a file Ctrl-Z
//!   is dropped and sets `FEOFLAG` (later reads return 0 until a seek clears
//!   it); for a device it is copied and nothing is flagged.
//! * `FCRLF` is set when the raw buffer STARTS with LF (stdio's ftell uses it),
//!   cleared otherwise.
//! * A CR at the very end of the buffer is resolved with a one-character
//!   `ReadFile` peek. Seekable file: seek back one character and drop the CR
//!   if the peek was LF -- unless the CR is the only output so far, in which
//!   case the LF is stored (so a 1-byte read of "\r\n" makes progress). Pipe
//!   or device: store LF, or store the CR and park the peeked character in
//!   `_pipe_lookahead`. A failed or empty peek keeps the CR.
//! * `_O_U16TEXT`/`_O_WTEXT` and `_O_U8TEXT` reads need an EVEN count (EINVAL
//!   with the handler), checked after the `count == 0` / `FEOFLAG` early 0.
//! * UTF-8 mode reads into a heap buffer of `max(count/2, 4)` bytes and
//!   converts; an incomplete trailing sequence is seeked back over (file) or
//!   parked in `_pipe_lookahead` (pipe/device), and a trailing byte that is not
//!   a lead byte is EILSEQ. `startpos` is set to the pre-read position, and
//!   `utf8translations` records whether the wide count differs from the byte count.
//! * `ERROR_ACCESS_DENIED` from `ReadFile` (a write-only handle) is EBADF with
//!   `_doserrno = 5`, not EACCES; `ERROR_BROKEN_PIPE` is a plain 0 (EOF).
//! * Microsoft quirk kept: after the pipe lookahead bytes are copied in,
//!   `internal_buffer` has been advanced past them, yet the ANSI/UTF-8
//!   translation is handed that ADVANCED pointer with a count that INCLUDES
//!   them (the UTF-16 path uses the pointer saved before the advance). Only
//!   reachable on pipes/devices with a parked lookahead.
//! * Microsoft quirk kept: a UTF-8 handle switched to `_O_BINARY` still reads
//!   into (and returns the count of) the private heap buffer, which is then
//!   freed -- the caller's buffer is left untouched.
//! * The UTF-8 lead-byte walk-back evaluates `*result_it` before its
//!   `result_it >= source_buffer` bound, so it can read one byte before the
//!   buffer, as in the C.

use super::common::*;
use super::lseek::_lseeki64_nolock;
use super::small::_isatty;
use super::{
    __acrt_errno_map_os_error, pioinfo, set_doserrno, set_errno, with_fh_locked, CR, CTRLZ, FCRLF, FDEV, FEOFLAG,
    FOPEN, FPIPE, FTEXT, HandleData, LF, TEXT_MODE_ANSI, TEXT_MODE_UTF16LE, TEXT_MODE_UTF8, ENOMEM,
};
use core::ffi::{c_char, c_int, c_void};
use core::ptr::null_mut;

extern "system" {
    fn ReadConsoleW(handle: *mut c_void, buffer: *mut c_void, chars_to_read: u32, chars_read: *mut u32, control: *mut c_void) -> c_int;
}

const fn build_lookuptrailbytes() -> [i8; 256] {
    let mut t = [0i8; 256];
    let mut i = 0xC0;
    while i < 0xE0 {
        t[i] = 1; // 110xxxxx
        i += 1;
    }
    while i < 0xF0 {
        t[i] = 2; // 1110xxxx
        i += 1;
    }
    while i < 0xF8 {
        t[i] = 3; // 11110xxx
        i += 1;
    }
    t
}

/// `char _lookuptrailbytes[256]`: trail-byte count per UTF-8 lead byte, 0 for
/// everything else (invalid patterns included -- MultiByteToWideChar
/// validates).
#[no_mangle]
pub static mut _lookuptrailbytes: [i8; 256] = build_lookuptrailbytes();

/// `_utf8_no_of_trailbytes(c)`.
unsafe fn utf8_no_of_trailbytes(c: u8) -> i8 {
    (*core::ptr::addr_of!(_lookuptrailbytes))[c as usize]
}

/// `_utf8_is_leadbyte(c)`.
unsafe fn utf8_is_leadbyte(c: u8) -> bool {
    utf8_no_of_trailbytes(c) != 0
}

/// A `Character` of `translate_text_mode_nolock<Character>`: `char` or `wchar_t`.
trait Unit: Copy + PartialEq {
    const CR: Self;
    const LF: Self;
    const CTRLZ: Self;
    const ZERO: Self;
    /// `store_lookahead(fh, c)`.
    unsafe fn store_lookahead(self, pio: *mut HandleData);
}

impl Unit for u8 {
    const CR: Self = CR;
    const LF: Self = LF;
    const CTRLZ: Self = CTRLZ;
    const ZERO: Self = 0;
    unsafe fn store_lookahead(self, pio: *mut HandleData) {
        (*pio).pipe_lookahead[0] = self;
    }
}

impl Unit for u16 {
    const CR: Self = CR as u16;
    const LF: Self = LF as u16;
    const CTRLZ: Self = CTRLZ as u16;
    const ZERO: Self = 0;
    unsafe fn store_lookahead(self, pio: *mut HandleData) {
        let bytes = self.to_le_bytes();
        (*pio).pipe_lookahead[0] = bytes[0];
        (*pio).pipe_lookahead[1] = bytes[1];
        (*pio).pipe_lookahead[2] = LF; // Mark as empty
    }
}

unsafe fn translate_utf16_from_console_nolock(fh: c_int, buffer: *mut u16, count: usize) -> c_int {
    let buffer_end = buffer.wrapping_add(count);
    let mut source_it = buffer;
    let mut result_it = buffer;

    while source_it < buffer_end {
        // Ctrl+Z stops the translation immediately.
        if *source_it == CTRLZ as u16 {
            (*pioinfo(fh)).osfile |= FEOFLAG;
            break;
        }
        // CRLF => LF.
        if *source_it == CR as u16 && source_it.wrapping_add(1) < buffer_end && *source_it.add(1) == LF as u16 {
            source_it = source_it.add(2);
            *result_it = LF as u16;
            result_it = result_it.add(1);
            continue;
        }
        *result_it = *source_it;
        result_it = result_it.add(1);
        source_it = source_it.add(1);
    }

    (result_it.offset_from(buffer) as usize * core::mem::size_of::<u16>()) as c_int
}

unsafe fn translate_text_mode_nolock<T: Unit>(fh: c_int, buffer: *mut T, count: usize) -> c_int {
    let pio = pioinfo(fh);
    let os_handle = (*pio).osfhnd;

    // If there is an LF at the beginning of the buffer, set the CRLF flag.
    if count != 0 && *buffer == T::LF {
        (*pio).osfile |= FCRLF;
    } else {
        (*pio).osfile &= !FCRLF;
    }

    let buffer_end = buffer.wrapping_add(count);
    let mut source_it = buffer;
    let mut result_it = buffer;

    while source_it < buffer_end {
        // Ctrl+Z: a file stops here and is flagged; a device gets the Ctrl+Z
        // copied through.
        if *source_it == T::CTRLZ {
            if (*pio).osfile & FDEV == 0 {
                (*pio).osfile |= FEOFLAG;
            } else {
                // (The C also advances source_it here; the loop is left anyway.)
                *result_it = *source_it;
                result_it = result_it.add(1);
            }
            break;
        }

        if *source_it != T::CR {
            *result_it = *source_it;
            result_it = result_it.add(1);
            source_it = source_it.add(1);
            continue;
        }

        // A CR not at the end of the buffer: look at the next character.
        if source_it.wrapping_add(1) < buffer_end {
            if *source_it.add(1) == T::LF {
                source_it = source_it.add(2);
                *result_it = T::LF; // Convert CRLF => LF
                result_it = result_it.add(1);
            } else {
                *result_it = *source_it;
                result_it = result_it.add(1);
                source_it = source_it.add(1);
            }
            continue;
        }

        // The CR is at the end of the buffer: peek at the next character.
        source_it = source_it.add(1);

        // The C leaves `peek` uninitialized; a short (1-byte) UTF-16 peek
        // therefore compares stack garbage in its high byte. Here it is 0.
        let mut peek: T = T::ZERO;
        let mut peek_size: u32 = 0;
        if ReadFile(
            os_handle,
            (&mut peek as *mut T).cast(),
            core::mem::size_of::<T>() as u32,
            &mut peek_size,
            null_mut(),
        ) == 0
            || peek_size == 0
        {
            // Couldn't peek: just store the CR.
            *result_it = T::CR;
            result_it = result_it.add(1);
            continue;
        }

        if (*pio).osfile & (FDEV | FPIPE) != 0 {
            // Not seekable: keep the LF, or keep the CR and park the peek.
            if peek == T::LF {
                *result_it = T::LF;
                result_it = result_it.add(1);
            } else {
                *result_it = T::CR;
                result_it = result_it.add(1);
                peek.store_lookahead(pio);
            }
        } else if peek == T::LF && result_it == buffer {
            // A one-character read: store the LF so the read makes progress.
            *result_it = T::LF;
            result_it = result_it.add(1);
        } else {
            // Seekable: unpeek, and keep the CR only if no LF followed.
            _lseeki64_nolock(fh, -(core::mem::size_of::<T>() as c_int) as i64, SEEK_CUR_W);
            if peek != T::LF {
                *result_it = T::CR;
                result_it = result_it.add(1);
            }
        }
    }

    (result_it.offset_from(buffer) as usize * core::mem::size_of::<T>()) as c_int
}

unsafe fn translate_ansi_or_utf8_nolock(
    fh: c_int,
    source_buffer: *mut u8,
    source_count: usize,
    result_buffer: *mut u16,
    result_count: usize,
) -> c_int {
    let text_mode_translation_result_size = translate_text_mode_nolock(fh, source_buffer, source_count);

    if text_mode_translation_result_size == 0 {
        return 0;
    }

    let pio = pioinfo(fh);

    // ANSI: nothing further to do.
    if (*pio).textmode == TEXT_MODE_ANSI {
        return text_mode_translation_result_size;
    }

    // UTF-8: find the end of the last complete character.
    let mut result_it = source_buffer.wrapping_offset(text_mode_translation_result_size as isize - 1);

    if *result_it & 0x80 == 0 {
        // `_utf8_is_independent`: the whole buffer converts.
        result_it = result_it.wrapping_add(1);
    } else {
        // Walk back to the last lead byte.
        let mut counter: u32 = 1;
        while !utf8_is_leadbyte(*result_it) && counter <= 4 && result_it >= source_buffer {
            result_it = result_it.wrapping_sub(1);
            counter += 1;
        }

        let trailbyte_count = utf8_no_of_trailbytes(*result_it) as u32;
        if trailbyte_count == 0 {
            // Not a lead byte after all: invalid UTF-8.
            set_errno(CRT_EILSEQ);
            return -1;
        }

        if trailbyte_count + 1 == counter {
            // The last character is complete.
            result_it = result_it.wrapping_add(counter as usize);
        } else if (*pio).osfile & (FDEV | FPIPE) != 0 {
            // Incomplete, not seekable: park the bytes in the lookahead.
            (*pio).pipe_lookahead[0] = *result_it;
            result_it = result_it.wrapping_add(1);
            if counter >= 2 {
                (*pio).pipe_lookahead[1] = *result_it;
                result_it = result_it.wrapping_add(1);
            }
            if counter == 3 {
                (*pio).pipe_lookahead[2] = *result_it;
                result_it = result_it.wrapping_add(1);
            }
            // Back to the actual end of the translatable sequence.
            result_it = result_it.wrapping_sub(counter as usize);
        } else {
            // Incomplete, seekable: seek back so the next read gets it.
            _lseeki64_nolock(fh, -(counter as c_int) as i64, SEEK_CUR_W);
        }
    }

    let characters_translated = __acrt_MultiByteToWideChar(
        CP_UTF8_W,
        0,
        source_buffer as *const c_char,
        (result_it as usize).wrapping_sub(source_buffer as usize) as u32 as c_int,
        result_buffer,
        result_count as u32 as c_int,
    );

    if characters_translated == 0 {
        __acrt_errno_map_os_error(GetLastError());
        return -1;
    }

    let converted_bytes = (result_it as usize).wrapping_sub(source_buffer as usize) as c_int;
    (*pio).set_bit(HandleData::BIT_UTF8_TRANSLATIONS, characters_translated != converted_bytes);

    // Bytes, not wide characters.
    (characters_translated as usize * core::mem::size_of::<u16>()) as c_int
}

#[no_mangle]
pub unsafe extern "C" fn _read(fh: c_int, buffer: *mut c_void, buffer_size: u32) -> c_int {
    if !validate_fh_clear_oserr(fh) {
        return -1;
    }
    if buffer_size > c_int::MAX as u32 {
        super::invalid_clear_oserr(EINVAL);
        return -1;
    }
    with_fh_locked(fh, || {
        if (*pioinfo(fh)).osfile & FOPEN == 0 {
            set_errno(EBADF);
            set_doserrno(0);
            return -1;
        }
        _read_nolock(fh, buffer, buffer_size)
    })
}

#[no_mangle]
pub unsafe extern "C" fn _read_nolock(fh: c_int, result_buffer: *mut c_void, result_buffer_size: u32) -> c_int {
    if !validate_fh_clear_oserr(fh) {
        return -1;
    }
    if result_buffer_size > c_int::MAX as u32 {
        super::invalid_clear_oserr(EINVAL);
        return -1;
    }

    let pio = pioinfo(fh);

    // Nothing to read, or already at Ctrl+Z EOF.
    if result_buffer_size == 0 || (*pio).osfile & FEOFLAG != 0 {
        return 0;
    }

    if result_buffer.is_null() {
        super::invalid_clear_oserr(EINVAL);
        return -1;
    }

    let os_handle = (*pio).osfhnd;
    let text_mode = (*pio).textmode;

    let mut owned_internal_buffer = HeapPtr(null_mut());

    let mut internal_buffer: *mut u8;
    let mut internal_buffer_remaining: u32;
    match text_mode {
        TEXT_MODE_UTF8 => {
            // UTF-8 needs its own buffer, converted into the caller's after.
            if result_buffer_size % 2 != 0 {
                super::invalid_clear_oserr(EINVAL);
                return -1;
            }
            internal_buffer_remaining = if result_buffer_size / 2 < 4 { 4 } else { result_buffer_size / 2 };
            owned_internal_buffer.0 = malloc(internal_buffer_remaining as usize);
            internal_buffer = owned_internal_buffer.0.cast();
            if internal_buffer.is_null() {
                set_errno(ENOMEM);
                set_doserrno(ERROR_NOT_ENOUGH_MEMORY_W as _);
                return -1;
            }
            (*pio).startpos = _lseeki64_nolock(fh, 0, SEEK_CUR_W);
        }
        TEXT_MODE_UTF16LE => {
            if result_buffer_size % 2 != 0 {
                super::invalid_clear_oserr(EINVAL);
                return -1;
            }
            internal_buffer_remaining = result_buffer_size;
            internal_buffer = result_buffer.cast();
        }
        _ => {
            internal_buffer_remaining = result_buffer_size;
            internal_buffer = result_buffer.cast();
        }
    }

    let wide_internal_buffer: *mut u16 = internal_buffer.cast();

    let mut bytes_read: c_int = 0;

    // Characters parked in the lookahead by the previous read come first; the
    // slots go back to empty (LF).
    if (*pio).osfile & (FPIPE | FDEV) != 0 && (*pio).pipe_lookahead[0] != LF && internal_buffer_remaining != 0 {
        *internal_buffer = (*pio).pipe_lookahead[0];
        internal_buffer = internal_buffer.add(1);
        bytes_read += 1;
        internal_buffer_remaining -= 1;
        (*pio).pipe_lookahead[0] = LF;

        // UTF-16 may have one more; UTF-8 two more.
        if text_mode != TEXT_MODE_ANSI && (*pio).pipe_lookahead[1] != LF && internal_buffer_remaining != 0 {
            *internal_buffer = (*pio).pipe_lookahead[1];
            internal_buffer = internal_buffer.add(1);
            bytes_read += 1;
            internal_buffer_remaining -= 1;
            (*pio).pipe_lookahead[1] = LF;

            if text_mode == TEXT_MODE_UTF8 && (*pio).pipe_lookahead[2] != LF && internal_buffer_remaining != 0 {
                *internal_buffer = (*pio).pipe_lookahead[2];
                internal_buffer = internal_buffer.add(1);
                bytes_read += 1;
                internal_buffer_remaining -= 1;
                (*pio).pipe_lookahead[2] = LF;
            }
        }
    }

    let mut console_mode: u32 = 0;
    let from_console = _isatty(fh) != 0
        && (*pio).osfile & FTEXT != 0
        && GetConsoleMode(os_handle as *mut c_void, &mut console_mode) != 0;

    if from_console && text_mode == TEXT_MODE_UTF16LE {
        // Read directly from the console.
        let mut console_characters_read: u32 = 0;
        if ReadConsoleW(
            os_handle as *mut c_void,
            internal_buffer.cast(),
            internal_buffer_remaining / core::mem::size_of::<u16>() as u32,
            &mut console_characters_read,
            null_mut(),
        ) == 0
        {
            __acrt_errno_map_os_error(GetLastError());
            return -1;
        }
        // Wide characters read; we need bytes.
        bytes_read = bytes_read.wrapping_add((console_characters_read as usize * core::mem::size_of::<u16>()) as c_int);
    } else {
        let mut bytes_read_from_file: u32 = 0;
        if ReadFile(
            os_handle,
            internal_buffer.cast(),
            internal_buffer_remaining,
            &mut bytes_read_from_file,
            null_mut(),
        ) == 0
            || bytes_read_from_file > result_buffer_size
        {
            let last_error = GetLastError();
            if last_error == ERROR_ACCESS_DENIED_W {
                // Wrong read/write mode: EBADF, not the EACCES the map gives.
                set_errno(EBADF);
                set_doserrno(last_error as _);
                return -1;
            } else if last_error == ERROR_BROKEN_PIPE_W {
                // All write handles of the pipe are closed: EOF.
                return 0;
            } else {
                __acrt_errno_map_os_error(last_error);
                return -1;
            }
        }
        bytes_read = bytes_read.wrapping_add(bytes_read_from_file as c_int);
    }

    // Binary: no translation.
    if (*pio).osfile & FTEXT == 0 {
        return bytes_read;
    }

    if text_mode != TEXT_MODE_UTF16LE {
        // ANSI or UTF-8 (see the module doc for the advanced-pointer quirk).
        translate_ansi_or_utf8_nolock(
            fh,
            internal_buffer,
            bytes_read as usize,
            result_buffer.cast(),
            result_buffer_size as usize / core::mem::size_of::<u16>(),
        )
    } else if from_console {
        translate_utf16_from_console_nolock(fh, wide_internal_buffer, bytes_read as usize / core::mem::size_of::<u16>())
    } else {
        translate_text_mode_nolock(fh, wide_internal_buffer, bytes_read as usize / core::mem::size_of::<u16>())
    }
}
