//! `pipe.cpp`: `_pipe`, an anonymous pipe as two CRT file handles
//! (`phandles[0]` reads, `phandles[1]` writes).
//!
//! Both slots start `FOPEN | FPIPE | FTEXT`, ANSI, not Unicode; the text flag
//! is dropped when `_O_BINARY` is asked for OR when neither mode is given and
//! the global `_fmode` is `_O_BINARY`. `_O_NOINHERIT` makes the OS handles
//! non-inheritable AND sets `FNOINHERIT`.

use core::ffi::{c_int, c_void};
use core::ptr::null_mut;

use super::{errcheck, invalid_clear_oserr, set_errno, GetLastError, __acrt_errno_map_os_error, __doserrno, EINVAL, EMFILE};
use crate::lowio::{
    HandleData, _alloc_osfhnd, __acrt_lowio_set_os_handle, __acrt_lowio_unlock_fh, pioinfo, FNOINHERIT, FOPEN, FPIPE,
    FTEXT, TEXT_MODE_ANSI,
};

const O_TEXT: c_int = 0x4000;
const O_BINARY: c_int = 0x8000;
const O_NOINHERIT: c_int = 0x0080;

/// `SECURITY_ATTRIBUTES`.
#[repr(C)]
struct SecurityAttributes {
    length: u32,
    security_descriptor: *mut c_void,
    inherit_handle: c_int,
}

extern "system" {
    fn CreatePipe(read: *mut *mut c_void, write: *mut *mut c_void, sa: *mut SecurityAttributes, size: u32) -> c_int;
    fn CloseHandle(handle: *mut c_void) -> i32;
}

extern "C" {
    fn _get_fmode(pmode: *mut c_int) -> c_int;
}

/// `_osfile(fh) = FOPEN | FPIPE | FTEXT; _textmode(fh) = ansi; _tm_unicode(fh) = false;`
unsafe fn init_pipe_slot(fh: c_int) {
    let pio = pioinfo(fh);
    (*pio).osfile = FOPEN | FPIPE | FTEXT;
    (*pio).textmode = TEXT_MODE_ANSI;
    (*pio).set_bit(HandleData::BIT_UNICODE, false);
}

#[no_mangle]
pub unsafe extern "C" fn _pipe(phandles: *mut c_int, psize: u32, textmode: c_int) -> c_int {
    if phandles.is_null() {
        invalid_clear_oserr(EINVAL);
        return -1;
    }
    *phandles = -1;
    *phandles.add(1) = -1;

    if textmode & !(O_NOINHERIT | O_BINARY | O_TEXT) != 0 {
        invalid_clear_oserr(EINVAL);
        return -1;
    }
    if textmode & (O_BINARY | O_TEXT) == (O_BINARY | O_TEXT) {
        invalid_clear_oserr(EINVAL);
        return -1;
    }

    let mut security_attributes = SecurityAttributes {
        length: core::mem::size_of::<SecurityAttributes>() as u32,
        security_descriptor: null_mut(),
        inherit_handle: (textmode & O_NOINHERIT == 0) as c_int,
    };

    let mut read_handle: *mut c_void = null_mut();
    let mut write_handle: *mut c_void = null_mut();
    if CreatePipe(&mut read_handle, &mut write_handle, &mut security_attributes, psize) == 0 {
        __acrt_errno_map_os_error(GetLastError());
        return -1;
    }

    // The read end's CRT handle (returned locked).
    let crt_read_handle = _alloc_osfhnd();
    if crt_read_handle == -1 {
        set_errno(EMFILE);
        *__doserrno() = 0;
        CloseHandle(read_handle);
        CloseHandle(write_handle);
        return -1;
    }
    init_pipe_slot(crt_read_handle);
    __acrt_lowio_unlock_fh(crt_read_handle);

    // The write end's.
    let crt_write_handle = _alloc_osfhnd();
    if crt_write_handle == -1 {
        (*pioinfo(crt_read_handle)).osfile = 0;
        set_errno(EMFILE);
        *__doserrno() = 0;
        CloseHandle(read_handle);
        CloseHandle(write_handle);
        return -1;
    }
    init_pipe_slot(crt_write_handle);
    __acrt_lowio_unlock_fh(crt_write_handle);

    let mut fmode: c_int = 0;
    errcheck(_get_fmode(&mut fmode));

    // Binary if asked for, or if nothing was asked and the default is binary.
    if textmode & O_BINARY != 0 || (textmode & O_TEXT == 0 && fmode == O_BINARY) {
        (*pioinfo(crt_read_handle)).osfile &= !FTEXT;
        (*pioinfo(crt_write_handle)).osfile &= !FTEXT;
    }

    if textmode & O_NOINHERIT != 0 {
        (*pioinfo(crt_read_handle)).osfile |= FNOINHERIT;
        (*pioinfo(crt_write_handle)).osfile |= FNOINHERIT;
    }

    __acrt_lowio_set_os_handle(crt_read_handle, read_handle as isize);
    __acrt_lowio_set_os_handle(crt_write_handle, write_handle as isize);

    *phandles = crt_read_handle;
    *phandles.add(1) = crt_write_handle;

    0
}
