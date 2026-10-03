//! `fclose.cpp`, `closeall.cpp`.
//!
//! Details that are silent if wrong:
//! * `_fclose_nolock_internal` returns `EOF` if the underlying `_close`
//!   fails, even though the flush and buffer-free already happened -- and it
//!   STILL frees `_tmpfname` and the stream itself either way; a failed close
//!   does not leak the stream slot.
//! * `_fclose_internal` short-circuits for a string-backed stream: no lock,
//!   no flush, straight to `free_stream`, and returns `EOF` unconditionally
//!   (not 0) -- closing a `sprintf`-style fake stream is always reported as a
//!   "failure", by design.
//! * `_fcloseall` iterates from `_IOB_ENTRIES` (skips stdin/stdout/stderr)
//!   through `_nstream`, closes every stream still in use via the public
//!   `fclose` (not `_fclose_nolock` -- it wants the full validation and
//!   locking path), then unconditionally deletes the slot's critical section
//!   and frees the slot itself, whether or not it was in use.

use core::ffi::{c_int, c_void};
use core::ptr::{addr_of_mut, null_mut};

use crate::ptd_host::PtdHost;
use crate::stdio::{free_stream, Stream, IOB_ENTRIES, STDIO_INDEX_LOCK};
use crate::stdio_layout::CriticalSection;

const EINVAL: c_int = 22;
const EOF: c_int = -1;

extern "C" {
    fn free(p: *mut c_void);
    fn __acrt_lock(lock: c_int);
    fn __acrt_unlock(lock: c_int);
    fn __acrt_stdio_free_buffer_nolock(stream: *mut c_void);
}

extern "system" {
    fn DeleteCriticalSection(cs: *mut CriticalSection);
}

unsafe fn fclose_nolock_internal(public_stream: *mut c_void, ptd: &mut PtdHost) -> c_int {
    let stream = Stream::from_file(public_stream);

    if !stream.valid() {
        ptd.invalid_parameter(EINVAL);
        return EOF;
    }

    let mut result = EOF;

    if stream.is_in_use() {
        result = super::flush::__acrt_stdio_flush_nolock(stream.public_stream(), ptd);
        __acrt_stdio_free_buffer_nolock(stream.public_stream());

        if super::raw_lowio::close_internal(stream.lowio_handle(), ptd) < 0 {
            result = EOF;
        } else {
            let d = stream.data();
            if !(*d).tmpfname.is_null() {
                free((*d).tmpfname.cast());
                (*d).tmpfname = null_mut();
            }
        }
    }

    free_stream(stream);

    result
}

#[no_mangle]
pub unsafe extern "C" fn _fclose_nolock(public_stream: *mut c_void) -> c_int {
    let mut ptd = PtdHost::new(null_mut());
    fclose_nolock_internal(public_stream, &mut ptd)
}

unsafe fn fclose_internal(public_stream: *mut c_void, ptd: &mut PtdHost) -> c_int {
    let stream = Stream::from_file(public_stream);

    if !stream.valid() {
        ptd.invalid_parameter(EINVAL);
        return EOF;
    }

    // A string-backed stream requires no synchronization/flushing: freeing
    // it resets all of its data to defaults.
    if stream.is_string_backed() {
        free_stream(stream);
        return EOF;
    }

    stream.lock();
    let return_value = fclose_nolock_internal(stream.public_stream(), ptd);
    stream.unlock();

    return_value
}

#[no_mangle]
pub unsafe extern "C" fn fclose(public_stream: *mut c_void) -> c_int {
    let mut ptd = PtdHost::new(null_mut());
    fclose_internal(public_stream, &mut ptd)
}

// ── closeall.cpp ─────────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn _fcloseall() -> c_int {
    let mut count: c_int = 0;

    __acrt_lock(STDIO_INDEX_LOCK);

    let nstream = crate::stdio::_nstream;
    for i in IOB_ENTRIES as c_int..nstream {
        let entry = *crate::stdio::__piob.add(i as usize);
        if entry.is_null() {
            continue;
        }

        let stream = Stream(entry);
        if stream.is_in_use() && fclose(stream.public_stream()) != EOF {
            count += 1;
        }

        DeleteCriticalSection(addr_of_mut!((*entry).lock));
        free(entry.cast());
        *crate::stdio::__piob.add(i as usize) = null_mut();
    }

    __acrt_unlock(STDIO_INDEX_LOCK);

    count
}
