//! `lowio/write.cpp`: `_write`, `_write_internal` and `_write_nolock` (which,
//! unusually, takes the PTD host itself -- `chsize.cpp` and misc's `perror`
//! call it that way).
//!
//! Details that are silent if wrong:
//! * The return is the number of SOURCE bytes consumed, not bytes written:
//!   text mode writes CRLF for each LF but subtracts the added CRs
//!   (`char_count - lf_count`). In UTF-16 mode each LF counts 2 bytes of
//!   `lf_count`; in UTF-8 mode `char_count` is the source position after the
//!   last fully committed chunk and `lf_count` stays 0.
//! * Nothing written: an OS error is mapped (`ERROR_ACCESS_DENIED` -> EBADF
//!   with `_doserrno = 5`); no error but a device and a leading Ctrl-Z is a
//!   plain 0; otherwise ENOSPC with `_doserrno = 0`.
//! * `_O_U8TEXT` / `_O_U16TEXT` / `_O_WTEXT` handles require an EVEN byte count
//!   (EINVAL, handler, `_doserrno = 0`), checked after `count == 0` returns 0
//!   and after the NULL-buffer check.
//! * `FAPPEND` seeks to the end first, ignoring failure.
//! * The UTF-8 file path converts in chunks of at most 852 UTF-16 units (853
//!   with a CRLF) and loops `WriteFile` until each converted chunk is fully
//!   written, so it never leaves half a UTF-8 sequence.
//! * Console (double translation, only when `_isatty`, text mode, a console
//!   mode is readable, and not (C locale and ANSI mode)): Microsoft writes the
//!   CR AFTER the LF ("LFCR"), one character at a time; ANSI goes through
//!   mbtowc then WideCharToMultiByte to the console code page, buffering a
//!   split DBCS lead byte in `_dbcsBuffer` or a split UTF-8 sequence in
//!   `_mbBuffer` and reporting it as written.

use super::common::*;
use super::lseek::_lseeki64_nolock_internal;
use super::small::_isatty;
use super::{pioinfo, with_fh_locked, HandleData, CR, CTRLZ, ENOSPC, FAPPEND, FDEV, FOPEN, FTEXT, LF, MB_LEN_MAX,
    TEXT_MODE_ANSI, TEXT_MODE_UTF16LE, TEXT_MODE_UTF8};
use crate::ptd_host::PtdHost;
use core::ffi::{c_char, c_int, c_void};
use core::ptr::null_mut;

/// `LC_CTYPE`.
const CRT_LC_CTYPE: usize = 2;
/// `_LEADBYTE` in `_locale_pctype`.
const CRT_LEADBYTE: u16 = 0x8000;

/// `BUF_SIZE`: the LF => CRLF translation buffer size.
const BUF_SIZE: usize = 5 * 1024;

extern "C" {
    fn _mbtowc_internal(dst: *mut u16, src: *const u8, src_size: usize, ptd: *mut PtdHost) -> c_int;
    /// conio/putwch.cpp.
    fn _putwch_nolock(c: u16) -> u16;
    /// `__crt_mbstring::__mbsrtowcs_utf8` (convert/mbrtowc.cpp): reached, as
    /// write.cpp reaches it, by its Itanium-mangled name (`j`/`y` is the
    /// `size_t` parameter; i386's extra `_` is added by rustc).
    #[cfg_attr(
        target_pointer_width = "32",
        link_name = "_ZN14__crt_mbstring16__mbsrtowcs_utf8EPwPPKcjP9_MbstatetR21__crt_cached_ptd_host"
    )]
    #[cfg_attr(
        target_pointer_width = "64",
        link_name = "_ZN14__crt_mbstring16__mbsrtowcs_utf8EPwPPKcyP9_MbstatetR21__crt_cached_ptd_host"
    )]
    fn mbsrtowcs_utf8(dst: *mut u16, src: *mut *const c_char, len: usize, ps: *mut Mbstate, ptd: *mut PtdHost) -> usize;
}

extern "system" {
    fn GetConsoleOutputCP() -> u32;
}

/// `mbstate_t` (`_Mbstatet`: `unsigned long _Wchar; unsigned short _Byte, _State;`).
#[repr(C)]
struct Mbstate {
    wchar: u32,
    byte: u16,
    state: u16,
}

#[derive(Clone, Copy, Default)]
struct WriteResult {
    error_code: u32,
    char_count: u32,
    lf_count: u32,
}

#[no_mangle]
pub unsafe extern "C" fn _write_internal(fh: c_int, buffer: *const c_void, size: u32, ptd: *mut PtdHost) -> c_int {
    let ptd = &mut *ptd;
    if !ptd_validate_fh(ptd, fh) {
        return -1;
    }
    with_fh_locked(fh, || {
        if (*pioinfo(fh)).osfile & FOPEN == 0 {
            ptd.errno().set(EBADF);
            ptd.doserrno().set(0);
            return -1;
        }
        _write_nolock(fh, buffer, size, ptd)
    })
}

#[no_mangle]
pub unsafe extern "C" fn _write(fh: c_int, buffer: *const c_void, size: u32) -> c_int {
    let mut ptd = PtdHost::new(null_mut());
    _write_internal(fh, buffer, size, &mut ptd)
}

unsafe fn write_requires_double_translation_nolock(fh: c_int, ptd: &mut PtdHost) -> bool {
    // Not a TTY: not the console.
    if _isatty(fh) == 0 {
        return false;
    }
    let pio = pioinfo(fh);
    if (*pio).osfile & FTEXT == 0 {
        return false;
    }
    // C locale and ANSI mode: no double translation.
    let is_c_locale = (*(*ptd.get_locale()).locinfo).locale_name[CRT_LC_CTYPE].is_null();
    if is_c_locale && (*pio).textmode == TEXT_MODE_ANSI {
        return false;
    }
    // If we can't get the console mode, it's not the console.
    let mut mode: u32 = 0;
    if GetConsoleMode((*pio).osfhnd as *mut c_void, &mut mode) == 0 {
        return false;
    }
    true
}

/// `_isleadbyte_fast_internal(c, locale)`.
unsafe fn isleadbyte_fast(c: u8, ptd: &mut PtdHost) -> bool {
    let pctype = (*(*(*ptd.get_locale()).locinfo).public_ref()).locale_pctype;
    *pctype.add(c as usize) & CRT_LEADBYTE != 0
}

unsafe fn write_double_translated_ansi_nolock(fh: c_int, buffer: *const u8, buffer_size: u32, ptd: &mut PtdHost) -> WriteResult {
    let pio = pioinfo(fh);
    let os_handle = (*pio).osfhnd as *mut c_void;
    let buffer_end = buffer.add(buffer_size as usize);
    let console_cp = GetConsoleOutputCP();
    let is_utf8 = (*(*(*ptd.get_locale()).locinfo).public_ref()).locale_lc_codepage == CP_UTF8_W;

    let mut result = WriteResult::default();

    let mut source_it = buffer;
    while source_it < buffer_end {
        let c = *source_it;

        // Double conversion: source multibyte -> Unicode -> console code page.
        // _write may be called byte by byte, so a split character is buffered
        // and reported as written.
        let mut wc: [u16; 2] = [0; 2];
        let mut wc_used: c_int = 1;
        if is_utf8 {
            let mb_buf_size = MB_LEN_MAX;
            let mut mb_buf_used = 0usize;
            while mb_buf_used < mb_buf_size && (*pio).mb_buffer[mb_buf_used] != 0 {
                mb_buf_used += 1;
            }

            if mb_buf_used > 0 {
                let mb_len = utf8_no_of_trailbytes((*pio).mb_buffer[0]) as c_int + 1;
                let remaining_bytes = mb_len - mb_buf_used as c_int;
                if remaining_bytes as isize <= buffer_end.offset_from(source_it) {
                    // Enough bytes to complete the code point.
                    let mut mb_buffer: [u8; MB_LEN_MAX] = [0; MB_LEN_MAX];
                    core::ptr::copy_nonoverlapping((*pio).mb_buffer.as_ptr(), mb_buffer.as_mut_ptr(), mb_buf_used);
                    core::ptr::copy_nonoverlapping(
                        source_it,
                        mb_buffer.as_mut_ptr().add(mb_buf_used),
                        remaining_bytes as usize,
                    );
                    // Clear out the temp buffer.
                    core::ptr::write_bytes((*pio).mb_buffer.as_mut_ptr(), 0, mb_buf_used);

                    let mut state = Mbstate { wchar: 0, byte: 0, state: 0 };
                    let mut str_: *const c_char = mb_buffer.as_ptr().cast();
                    if mb_len == 4 {
                        wc_used = 2;
                    }
                    if mbsrtowcs_utf8(wc.as_mut_ptr(), &mut str_, wc_used as usize, &mut state, ptd) == usize::MAX {
                        return result;
                    }
                    source_it = source_it.add((remaining_bytes - 1) as usize);
                } else {
                    // Not enough yet: add what there is and pretend it was written.
                    let bytes_to_add = buffer_end.offset_from(source_it) as usize;
                    core::ptr::copy_nonoverlapping(
                        source_it,
                        (*pio).mb_buffer.as_mut_ptr().add(mb_buf_used),
                        bytes_to_add,
                    );
                    result.char_count = result.char_count.wrapping_add(bytes_to_add as u32);
                    return result;
                }
            } else {
                let mb_len = utf8_no_of_trailbytes(*source_it) as c_int + 1;
                let available_bytes = buffer_end.offset_from(source_it);
                if mb_len as isize <= available_bytes {
                    // Enough bytes for the whole code point.
                    let mut state = Mbstate { wchar: 0, byte: 0, state: 0 };
                    let mut str_: *const c_char = source_it.cast();
                    if mb_len == 4 {
                        wc_used = 2;
                    }
                    if mbsrtowcs_utf8(wc.as_mut_ptr(), &mut str_, wc_used as usize, &mut state, ptd) == usize::MAX {
                        return result;
                    }
                    source_it = source_it.add((mb_len - 1) as usize);
                } else {
                    // Not enough bytes for this code point: buffer them.
                    core::ptr::copy_nonoverlapping(source_it, (*pio).mb_buffer.as_mut_ptr(), available_bytes as usize);
                    result.char_count = result.char_count.wrapping_add(available_bytes as u32);
                    return result;
                }
            }
        } else if (*pio).bits & HandleData::BIT_DBCS_BUFFER_USED != 0 {
            // A buffered DBCS lead byte: combine it with this byte.
            let mut mb_buffer: [u8; MB_LEN_MAX] = [0; MB_LEN_MAX];
            mb_buffer[0] = (*pio).mb_buffer[0];
            mb_buffer[1] = *source_it;
            (*pio).set_bit(HandleData::BIT_DBCS_BUFFER_USED, false);
            if _mbtowc_internal(wc.as_mut_ptr(), mb_buffer.as_ptr(), 2, ptd) == -1 {
                return result;
            }
        } else if isleadbyte_fast(*source_it, ptd) {
            if source_it.add(1) < buffer_end {
                // More bytes: convert the pair.
                if _mbtowc_internal(wc.as_mut_ptr(), source_it, 2, ptd) == -1 {
                    return result;
                }
                source_it = source_it.add(1);
            } else {
                // Out of bytes: buffer the lead byte and claim it was written.
                (*pio).mb_buffer[0] = *source_it;
                (*pio).set_bit(HandleData::BIT_DBCS_BUFFER_USED, true);
                result.char_count = result.char_count.wrapping_add(1);
                return result;
            }
        } else if _mbtowc_internal(wc.as_mut_ptr(), source_it, 1, ptd) == -1 {
            // single character conversion failed
            return result;
        }

        source_it = source_it.add(1);

        // Unicode -> console code page, then write.
        let mut mb_buffer: [u8; MB_LEN_MAX] = [0; MB_LEN_MAX];
        let size = __acrt_WideCharToMultiByte(
            console_cp,
            0,
            wc.as_ptr(),
            wc_used,
            mb_buffer.as_mut_ptr().cast(),
            MB_LEN_MAX as c_int,
            core::ptr::null(),
            null_mut(),
        ) as u32;

        if size == 0 {
            return result;
        }

        let mut written: u32 = 0;
        if WriteFile(os_handle, mb_buffer.as_ptr().cast(), size, &mut written, null_mut()) == 0 {
            result.error_code = GetLastError();
            return result;
        }

        // Count source bytes consumed plus LFs, not bytes written (2 MBCS ->
        // 1 wide -> 1 MBCS is possible).
        result.char_count = result.lf_count.wrapping_add(source_it.offset_from(buffer) as u32);

        if written < size {
            return result;
        }

        // An LF gets a CR written AFTER it (Microsoft's own TODO: "LFCR").
        if c == LF {
            let cr: u16 = CR as u16;
            if WriteFile(os_handle, (&cr as *const u16).cast(), 1, &mut written, null_mut()) == 0 {
                result.error_code = GetLastError();
                return result;
            }
            if written < 1 {
                return result;
            }
            result.lf_count = result.lf_count.wrapping_add(1);
            result.char_count = result.char_count.wrapping_add(1);
        }
    }

    result
}

unsafe fn write_double_translated_unicode_nolock(buffer: *const u8, buffer_size: u32) -> WriteResult {
    // Console + Unicode mode: print each character with _putwch_nolock.
    let mut result = WriteResult::default();
    let buffer_end = buffer.add(buffer_size as usize);
    let mut pch = buffer;
    while pch < buffer_end {
        let c = (pch as *const u16).read_unaligned();

        if _putwch_nolock(c) == c {
            result.char_count = result.char_count.wrapping_add(2);
        } else {
            result.error_code = GetLastError();
            return result;
        }

        // An LF gets a CR after it ("LFCR", as in the ANSI path).
        if c == LF as u16 {
            if _putwch_nolock(CR as u16) != CR as u16 {
                result.error_code = GetLastError();
                return result;
            }
            result.char_count = result.char_count.wrapping_add(1);
            result.lf_count = result.lf_count.wrapping_add(1);
        }
        pch = pch.add(2);
    }
    result
}

unsafe fn write_text_ansi_nolock(fh: c_int, buffer: *const u8, buffer_size: u32) -> WriteResult {
    let os_handle = (*pioinfo(fh)).osfhnd as *mut c_void;
    let buffer_end = buffer.add(buffer_size as usize);

    let mut result = WriteResult::default();

    let mut source_it = buffer;
    while source_it < buffer_end {
        let mut lfbuf: [u8; BUF_SIZE] = [0; BUF_SIZE];
        // One short of the end: an LF needs room for both CR and LF.
        let lfbuf_end = BUF_SIZE - 1;

        let mut lfbuf_it = 0usize;
        while lfbuf_it < lfbuf_end && source_it < buffer_end {
            let c = *source_it;
            source_it = source_it.add(1);
            if c == LF {
                result.lf_count = result.lf_count.wrapping_add(1);
                lfbuf[lfbuf_it] = CR;
                lfbuf_it += 1;
            }
            lfbuf[lfbuf_it] = c;
            lfbuf_it += 1;
        }

        let lfbuf_length = lfbuf_it as u32;

        let mut written: u32 = 0;
        if WriteFile(os_handle, lfbuf.as_ptr().cast(), lfbuf_length, &mut written, null_mut()) == 0 {
            result.error_code = GetLastError();
            return result;
        }

        result.char_count = result.char_count.wrapping_add(written);
        if written < lfbuf_length {
            return result; // The write succeeded but didn't write everything
        }
    }

    result
}

unsafe fn write_text_utf16le_nolock(fh: c_int, buffer: *const u8, buffer_size: u32) -> WriteResult {
    let os_handle = (*pioinfo(fh)).osfhnd as *mut c_void;
    // An odd byte count was rejected by the caller; the end is in u16 units.
    let buffer_end = buffer.add(buffer_size as usize) as *const u16;

    let mut result = WriteResult::default();

    let mut source_it = buffer as *const u16;
    while source_it < buffer_end {
        const N: usize = BUF_SIZE / core::mem::size_of::<u16>();
        let mut lfbuf: [u16; N] = [0; N];
        let lfbuf_end = N - 1;

        let mut lfbuf_it = 0usize;
        while lfbuf_it < lfbuf_end && source_it < buffer_end {
            let c = source_it.read_unaligned();
            source_it = source_it.add(1);
            if c == LF as u16 {
                result.lf_count = result.lf_count.wrapping_add(2);
                lfbuf[lfbuf_it] = CR as u16;
                lfbuf_it += 1;
            }
            lfbuf[lfbuf_it] = c;
            lfbuf_it += 1;
        }

        // In bytes, for WriteFile.
        let lfbuf_length = (lfbuf_it * core::mem::size_of::<u16>()) as u32;

        let mut written: u32 = 0;
        if WriteFile(os_handle, lfbuf.as_ptr().cast(), lfbuf_length, &mut written, null_mut()) == 0 {
            result.error_code = GetLastError();
            return result;
        }

        result.char_count = result.char_count.wrapping_add(written);
        if written < lfbuf_length {
            return result; // The write succeeded, but didn't write everything
        }
    }

    result
}

unsafe fn write_text_utf8_nolock(fh: c_int, buffer: *const u8, buffer_size: u32) -> WriteResult {
    let os_handle = (*pioinfo(fh)).osfhnd as *mut c_void;
    let buffer_end = buffer.add(buffer_size as usize) as *const u16;

    let mut result = WriteResult::default();

    let mut source_it = buffer as *const u16;
    while source_it < buffer_end {
        // UTF-16 LF => CRLF buffer, sized so its UTF-8 fits utf8_buf.
        const N16: usize = BUF_SIZE / 6;
        let mut utf16_buf: [u16; N16] = [0; N16];
        let utf16_buf_end = N16 - 1;

        let mut utf16_buf_it = 0usize;
        while utf16_buf_it < utf16_buf_end && source_it < buffer_end {
            let c = source_it.read_unaligned();
            source_it = source_it.add(1);
            if c == LF as u16 {
                // LFs are not counted: char_count is taken from the source
                // position below.
                utf16_buf[utf16_buf_it] = CR as u16;
                utf16_buf_it += 1;
            }
            utf16_buf[utf16_buf_it] = c;
            utf16_buf_it += 1;
        }

        // In characters, not bytes.
        let utf16_buf_length = utf16_buf_it as u32;

        const N8: usize = (BUF_SIZE * 2) / 3;
        let mut utf8_buf: [u8; N8] = [0; N8];
        let bytes_converted = __acrt_WideCharToMultiByte(
            CP_UTF8_W,
            0,
            utf16_buf.as_ptr(),
            utf16_buf_length as c_int,
            utf8_buf.as_mut_ptr().cast(),
            N8 as c_int,
            core::ptr::null(),
            null_mut(),
        ) as u32;

        if bytes_converted == 0 {
            result.error_code = GetLastError();
            return result;
        }

        // Write ALL the converted bytes (a partial UTF-8 sequence would
        // corrupt the stream), unless there is an error.
        let mut bytes_written: u32 = 0;
        while bytes_written < bytes_converted {
            let current = utf8_buf.as_ptr().add(bytes_written as usize);
            let current_size = bytes_converted - bytes_written;

            let mut written: u32 = 0;
            if WriteFile(os_handle, current.cast(), current_size, &mut written, null_mut()) == 0 {
                result.error_code = GetLastError();
                return result;
            }
            bytes_written = bytes_written.wrapping_add(written);
        }

        // This chunk was committed: count the source consumed so far.
        result.char_count = (source_it as usize).wrapping_sub(buffer as usize) as u32;
    }

    result
}

unsafe fn write_binary_nolock(fh: c_int, buffer: *const u8, buffer_size: u32) -> WriteResult {
    let os_handle = (*pioinfo(fh)).osfhnd as *mut c_void;
    let mut result = WriteResult::default();
    if WriteFile(os_handle, buffer.cast(), buffer_size, &mut result.char_count, null_mut()) == 0 {
        result.error_code = GetLastError();
    }
    result
}

#[no_mangle]
pub unsafe extern "C" fn _write_nolock(fh: c_int, buffer: *const c_void, buffer_size: u32, ptd: *mut PtdHost) -> c_int {
    let ptd = &mut *ptd;

    // Nothing to write.
    if buffer_size == 0 {
        return 0;
    }

    if buffer.is_null() {
        ptd_invalid_clear_oserr(ptd, EINVAL);
        return -1;
    }

    let pio = pioinfo(fh);
    let fh_textmode = (*pio).textmode;

    // Unicode modes need an even byte count.
    if (fh_textmode == TEXT_MODE_UTF16LE || fh_textmode == TEXT_MODE_UTF8) && buffer_size % 2 != 0 {
        ptd_invalid_clear_oserr(ptd, EINVAL);
        return -1;
    }

    // Appending: seek to the end, ignoring failure (the file may not seek).
    if (*pio).osfile & FAPPEND != 0 {
        let _ = _lseeki64_nolock_internal(fh, 0, SEEK_END_W, ptd);
    }

    let char_buffer = buffer as *const u8;

    let mut result = WriteResult::default();
    if write_requires_double_translation_nolock(fh, ptd) {
        match fh_textmode {
            TEXT_MODE_ANSI => result = write_double_translated_ansi_nolock(fh, char_buffer, buffer_size, ptd),
            TEXT_MODE_UTF16LE | TEXT_MODE_UTF8 => result = write_double_translated_unicode_nolock(char_buffer, buffer_size),
            _ => {}
        }
    } else if (*pio).osfile & FTEXT != 0 {
        match fh_textmode {
            TEXT_MODE_ANSI => result = write_text_ansi_nolock(fh, char_buffer, buffer_size),
            TEXT_MODE_UTF16LE => result = write_text_utf16le_nolock(fh, char_buffer, buffer_size),
            TEXT_MODE_UTF8 => result = write_text_utf8_nolock(fh, char_buffer, buffer_size),
            _ => {}
        }
    } else {
        result = write_binary_nolock(fh, char_buffer, buffer_size);
    }

    // Why did we not write anything?
    if result.char_count == 0 {
        if result.error_code != 0 {
            // ERROR_ACCESS_DENIED is EBADF here, not EACCES.
            if result.error_code == ERROR_ACCESS_DENIED_W {
                ptd.errno().set(EBADF);
                ptd.doserrno().set(result.error_code as _);
            } else {
                __acrt_errno_map_os_error_ptd(result.error_code as _, ptd);
            }
            return -1;
        }

        // A device and a leading Ctrl+Z: writing nothing is expected.
        if (*pio).osfile & FDEV != 0 && *char_buffer == CTRLZ {
            return 0;
        }

        // Otherwise, ENOSPC.
        ptd.errno().set(ENOSPC);
        ptd.doserrno().set(0);
        return -1;
    }

    // The adjusted number of bytes written.
    result.char_count.wrapping_sub(result.lf_count) as c_int
}

/// `_utf8_no_of_trailbytes(c)`, over read.rs's exported table.
unsafe fn utf8_no_of_trailbytes(c: u8) -> i8 {
    (*core::ptr::addr_of!(super::read::_lookuptrailbytes))[c as usize]
}
