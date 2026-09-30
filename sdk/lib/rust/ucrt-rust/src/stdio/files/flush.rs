//! `fflush.cpp`.
//!
//! Details that are silent if wrong:
//! * `is_stream_flushable` (buffered, write-mode) is a strict subset of
//!   `is_stream_flushable_or_commitable` (also true when `_IOCOMMIT` is set,
//!   even for a stream that has nothing buffered to write) -- `fflush(p)`'s
//!   early "would be a no-op" check uses the OR form, but the actual byte
//!   flush (`__acrt_stdio_flush_nolock`) only ever looks at the AND form; the
//!   commit itself is a separate step gated on `_IOCOMMIT` alone.
//! * `common_flush_all(false)` (plain `fflush(NULL)`) returns 0 on success or
//!   `EOF` if ANY stream failed to flush; `common_flush_all(true)`
//!   (`_flushall`) instead returns the COUNT of streams it flushed (or would
//!   have, for a no-op stream) and never fails.
//! * The pre-lock "would this be a no-op" check is re-verified AFTER the lock
//!   is acquired (another thread may have closed/reopened/changed the stream
//!   in between) -- both checks call the exact same predicate.
//! * `__acrt_stdio_flush_nolock`, if the write's byte count doesn't match
//!   what was requested, sets `_IOERROR` and returns `EOF` -- but the buffer
//!   is reset (and `_IOWRITE` cleared for an update-mode stream) EITHER WAY,
//!   even on that failure.

use core::ffi::{c_int, c_long, c_void};

use crate::ptd_host::PtdHost;
use crate::stdio::{reset_buffer, with_stream_locked, Stream, STDIO_INDEX_LOCK};
use crate::stdio_layout::flags::*;

const EOF: c_int = -1;

extern "C" {
    fn _fileno(stream: *mut c_void) -> c_int;
    fn _commit(fh: c_int) -> c_int;
    fn __acrt_lock(lock: c_int);
    fn __acrt_unlock(lock: c_int);
}

fn is_stream_allocated(flags: c_long) -> bool {
    flags & STREAM_ALLOCATED != 0
}

fn is_stream_flushable(flags: c_long) -> bool {
    if flags & (STREAM_READ | STREAM_WRITE) != STREAM_WRITE {
        return false;
    }
    if flags & (STREAM_BUFFER_CRT | STREAM_BUFFER_USER) == 0 {
        return false;
    }
    true
}

fn is_stream_flushable_or_commitable(flags: c_long) -> bool {
    is_stream_flushable(flags) || flags & STREAM_COMMIT != 0
}

unsafe fn common_flush_all_should_try_to_flush_stream(stream: Stream, flushed_stream_count: &mut c_int) -> bool {
    if !stream.valid() {
        return false;
    }
    let flags = stream.get_flags();
    if !is_stream_allocated(flags) {
        return false;
    }
    if !is_stream_flushable_or_commitable(flags) {
        *flushed_stream_count += 1;
        return false;
    }
    true
}

unsafe fn common_flush_all(flush_read_mode_streams: bool) -> c_int {
    let mut count: c_int = 0;
    let mut error: c_int = 0;

    __acrt_lock(STDIO_INDEX_LOCK);

    let nstream = crate::stdio::_nstream;
    for i in 0..nstream {
        let entry = *crate::stdio::__piob.add(i as usize);
        let stream = Stream(entry);

        if !common_flush_all_should_try_to_flush_stream(stream, &mut count) {
            continue;
        }

        with_stream_locked(stream.public_stream(), || {
            if !common_flush_all_should_try_to_flush_stream(stream, &mut count) {
                return;
            }
            if !flush_read_mode_streams && !stream.has_all_of(STREAM_WRITE) {
                return;
            }
            if _fflush_nolock(stream.public_stream()) != EOF {
                count += 1;
            } else {
                error = EOF;
            }
        });
    }

    __acrt_unlock(STDIO_INDEX_LOCK);

    if flush_read_mode_streams {
        count
    } else {
        error
    }
}

#[no_mangle]
pub unsafe extern "C" fn fflush(public_stream: *mut c_void) -> c_int {
    let stream = Stream::from_file(public_stream);

    if !stream.valid() {
        return common_flush_all(false);
    }

    if !is_stream_flushable_or_commitable(stream.get_flags()) {
        return 0;
    }

    with_stream_locked(stream.public_stream(), || _fflush_nolock(stream.public_stream()))
}

unsafe fn fflush_nolock_internal(public_stream: *mut c_void, ptd: &mut PtdHost) -> c_int {
    let stream = Stream::from_file(public_stream);

    if !stream.valid() {
        return common_flush_all(false);
    }

    if __acrt_stdio_flush_nolock(stream.public_stream(), ptd) != 0 {
        return EOF;
    }

    if stream.has_all_of(STREAM_COMMIT) && _commit(_fileno(public_stream)) != 0 {
        return EOF;
    }

    0
}

#[no_mangle]
pub unsafe extern "C" fn _fflush_nolock(public_stream: *mut c_void) -> c_int {
    let mut ptd = PtdHost::new(core::ptr::null_mut());
    fflush_nolock_internal(public_stream, &mut ptd)
}

/// Owned here; called across pieces (`bufio`, `output.rs`'s printf engine)
/// via a transcribed `extern "C"` declaration -- see the batch brief.
#[no_mangle]
pub unsafe extern "C" fn __acrt_stdio_flush_nolock(public_stream: *mut c_void, ptd: *mut PtdHost) -> c_int {
    let ptd = &mut *ptd;
    let stream = Stream::from_file(public_stream);

    if !is_stream_flushable(stream.get_flags()) {
        return 0;
    }

    let d = stream.data();
    let bytes_to_write = (*d).ptr as isize - (*d).base as isize;

    reset_buffer(stream);

    if bytes_to_write <= 0 {
        return 0;
    }

    let bytes_written = super::raw_lowio::write_internal(_fileno(stream.public_stream()), (*d).base.cast(), bytes_to_write as u32, ptd);
    if bytes_to_write != bytes_written as isize {
        stream.set_flags(STREAM_ERROR);
        return EOF;
    }

    // If this is a read/write file, clear _IOWRITE so the next operation can
    // be a read.
    if stream.has_all_of(STREAM_UPDATE) {
        stream.unset_flags(STREAM_WRITE);
    }

    0
}

#[no_mangle]
pub unsafe extern "C" fn _flushall() -> c_int {
    common_flush_all(true)
}
