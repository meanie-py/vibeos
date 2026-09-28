//! `setbuf.cpp`, `setvbuf.cpp`.
//!
//! Details that are silent if wrong:
//! * `setbuf` is a strict subset of `setvbuf`: a null buffer means
//!   `setvbuf(stream, NULL, _IONBF, 0)`; a non-null one means
//!   `setvbuf(stream, buffer, _IOFBF, BUFSIZ)` -- `_IOLBF` is never reachable
//!   from `setbuf`.
//! * `setvbuf` always flushes and frees any existing buffer FIRST, even for
//!   the `_IONBF` case, before clearing every buffering-related flag.
//! * The `_IONBF` buffer is the stream's own 2-byte `_charbuf` field, not a
//!   heap allocation -- `set_buffer` is handed `&stream->_charbuf` cast to
//!   `char*` with size 2 (the internal two-character pushback buffer every
//!   unbuffered stream still has).
//! * A CRT-allocated buffer (`buffer == nullptr`, `_IOFBF`/`_IOLBF`) forces
//!   the requested size to be EVEN by masking off the low bit, and is
//!   allocated via plain `calloc` (`_calloc_crt_t(char, n).detach()` is just
//!   that, per the batch brief's `_calloc_crt`/`_malloc_crt` note) -- a
//!   user-supplied buffer's size is NOT rounded.
//! * A failed CRT buffer allocation returns nonzero (failure) but otherwise
//!   leaves the stream in the just-cleared, buffer-less state (matching
//!   `_IONBF`'s flags never got set, so the stream is "no buffer, no flags"
//!   rather than falling back to unbuffered).
//! * The lock is released by ordinary sequencing after the buffer-selection
//!   `if`/`else`, not by an RAII guard -- this build's SEH is the dummy one
//!   (see `stdio/mod.rs`'s `with_stream_locked`), so a plain lock/compute/
//!   unlock is equivalent to the C's `__acrt_lock_stream_and_call`.

use core::ffi::{c_char, c_int, c_long, c_void};
use core::ptr::null_mut;

use crate::ptd_host::PtdHost;
use crate::stdio::Stream;
use crate::stdio_layout::flags::*;

const EINVAL: c_int = 22;

/// `_IOFBF`/`_IOLBF`/`_IONBF` (stdio.h).
const IOFBF: c_int = 0x0000;
const IOLBF: c_int = 0x0040;
const IONBF: c_int = 0x0004;
/// `BUFSIZ` (stdio.h).
const BUFSIZ: usize = 512;

extern "C" {
    fn calloc(count: usize, size: usize) -> *mut c_void;
    fn __acrt_stdio_free_buffer_nolock(stream: *mut c_void);
}

#[no_mangle]
pub unsafe extern "C" fn setbuf(stream: *mut c_void, buffer: *mut c_char) {
    debug_assert!(!stream.is_null());

    if buffer.is_null() {
        setvbuf(stream, null_mut(), IONBF, 0);
    } else {
        setvbuf(stream, buffer, IOFBF, BUFSIZ);
    }
}

unsafe fn set_buffer(stream: Stream, buffer: *mut c_char, buffer_size_in_bytes: usize, new_flag_bits: c_long) -> c_int {
    stream.set_flags(new_flag_bits);
    let d = stream.data();
    (*d).bufsiz = buffer_size_in_bytes as c_int;
    (*d).ptr = buffer;
    (*d).base = buffer;
    (*d).cnt = 0;
    0
}

unsafe fn setvbuf_internal(public_stream: *mut c_void, buffer: *mut c_char, kind: c_int, buffer_size_in_bytes: usize, ptd: &mut PtdHost) -> c_int {
    let stream = Stream::from_file(public_stream);

    if !stream.valid() {
        ptd.invalid_parameter(EINVAL);
        return -1;
    }
    if !(kind == IONBF || kind == IOFBF || kind == IOLBF) {
        ptd.invalid_parameter(EINVAL);
        return -1;
    }
    if (kind == IOFBF || kind == IOLBF) && !(2 <= buffer_size_in_bytes && buffer_size_in_bytes <= i32::MAX as usize) {
        ptd.invalid_parameter(EINVAL);
        return -1;
    }

    stream.lock();

    // Force the buffer size to be even by masking the low order bit.
    let usable_buffer_size = buffer_size_in_bytes & !1usize;

    super::flush::__acrt_stdio_flush_nolock(stream.public_stream(), ptd);
    __acrt_stdio_free_buffer_nolock(stream.public_stream());

    stream.unset_flags(STREAM_BUFFER_CRT | STREAM_BUFFER_USER | STREAM_BUFFER_NONE | STREAM_BUFFER_SETVBUF | STREAM_BUFFER_STBUF | STREAM_CTRLZ);

    let result = if kind & IONBF != 0 {
        // Case 1: no buffering.
        let d = stream.data();
        set_buffer(stream, core::ptr::addr_of_mut!((*d).charbuf).cast(), 2, STREAM_BUFFER_NONE)
    } else if buffer.is_null() {
        // Case 2: default buffering, CRT-allocated buffer.
        let crt_buffer = calloc(usable_buffer_size, 1) as *mut c_char;
        if crt_buffer.is_null() {
            -1
        } else {
            set_buffer(stream, crt_buffer, usable_buffer_size, STREAM_BUFFER_CRT | STREAM_BUFFER_SETVBUF)
        }
    } else {
        // Case 3: default buffering, user-provided buffer.
        set_buffer(stream, buffer, usable_buffer_size, STREAM_BUFFER_USER | STREAM_BUFFER_SETVBUF)
    };

    stream.unlock();
    result
}

#[no_mangle]
pub unsafe extern "C" fn setvbuf(public_stream: *mut c_void, buffer: *mut c_char, kind: c_int, buffer_size_in_bytes: usize) -> c_int {
    let mut ptd = PtdHost::new(null_mut());
    setvbuf_internal(public_stream, buffer, kind, buffer_size_in_bytes, &mut ptd)
}
