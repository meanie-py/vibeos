//! The stdio stream core: `stdio/_file.cpp` and `stdio/stream.cpp`, ported, plus
//! the header-inline machinery every other stdio file uses (`__crt_stdio_stream`,
//! `__acrt_stdio_reset_buffer`, `__crt_stdio_path_requires_backslash`).
//!
//! The rest of `stdio/` is ported in three pieces over this one, each its own
//! feature and submodule (see Cargo.toml). The pieces call one another through
//! the C names the functions have always had, declared `extern "C"` from the
//! header prototypes -- the same boundary the Microsoft C used, so a piece can be
//! built, graded and landed against the others still being C. What stays C++-only
//! (`__acrt_stdio_allocate_stream` / `__acrt_stdio_free_stream`, which return a
//! class by value) is plain Rust here: every caller is inside `stdio/`, and the
//! directory goes as one unit.
//!
//! Details that are silent if wrong:
//! * `_iob`, `__piob` and `_nstream` keep their C names and types. lowio's
//!   `ioinit.cpp` reads `__piob` by name, and `_iob` is `__acrt_iob_func`'s storage.
//! * ucrtbase is built with `CRTDLL`, so `_nstream` starts at `_NSTREAM_` (512),
//!   there is no `_cflush`, and `__acrt_initialize_stdio` never rewrites it.
//! * A stream's flags are updated with interlocked operations (`set_flags`,
//!   `try_allocate`), and allocation is decided by `_IOALLOCATED` alone.
//! * `__acrt_uninitialize_stdio` frees the three standard streams' buffers and
//!   deletes their locks but leaves `_iob` otherwise alone; the others are freed
//!   by `_fcloseall`.
//! * `_file.cpp`'s `_CRT_LINKER_FORCE_INCLUDE` of the stdio initializer and
//!   terminator is carried by `force_include!`, or deleting the C would silently
//!   stop stdio being initialized at all.

use crate::stdio_layout::flags::*;
use crate::stdio_layout::{lowio, CriticalSection, StreamData};
use core::ffi::{c_char, c_int, c_long, c_uint, c_void};
use core::ptr::{addr_of_mut, null_mut};
use core::sync::atomic::{AtomicI32, Ordering};

#[cfg(feature = "stdio-bufio")]
pub mod bufio;
#[cfg(feature = "stdio-files")]
pub mod files;
#[cfg(feature = "stdio-input")]
pub mod input;

/// `_IOB_ENTRIES`: stdin, stdout, stderr.
pub const IOB_ENTRIES: usize = 3;
/// `_NSTREAM_`.
pub const NSTREAM: c_int = 512;
/// `_INTERNAL_BUFSIZ` / `_SMALL_BUFSIZ` (corecrt_internal_lowio.h).
pub const INTERNAL_BUFSIZ: c_int = 4096;
pub const SMALL_BUFSIZ: c_int = 512;
/// `_NO_CONSOLE_FILENO`.
pub const NO_CONSOLE_FILENO: isize = -2;
/// `_CORECRT_SPINCOUNT`.
const CORECRT_SPINCOUNT: u32 = 4000;
/// `__acrt_lock_id` positions (corecrt_internal.h:936).
pub const STDIO_INDEX_LOCK: c_int = 8;
pub const TEMPNAM_LOCK: c_int = 12;

const EINVAL: c_int = 22;
const INVALID_HANDLE_VALUE: isize = -1;

extern "C" {
    fn calloc(count: usize, size: usize) -> *mut c_void;
    fn free(p: *mut c_void);
    fn __acrt_lock(lock: c_int);
    fn __acrt_unlock(lock: c_int);
    fn _errno() -> *mut c_int;
    fn _invalid_parameter_noinfo();
    // Owned by the other stdio pieces (fflush.cpp, closeall.cpp, _freebuf.cpp).
    fn _flushall() -> c_int;
    fn _fcloseall() -> c_int;
    fn __acrt_stdio_free_buffer_nolock(stream: *mut c_void);
}

extern "system" {
    fn __acrt_InitializeCriticalSectionEx(cs: *mut CriticalSection, spin_count: u32, flags: u32) -> c_int;
    fn EnterCriticalSection(cs: *mut CriticalSection);
    fn LeaveCriticalSection(cs: *mut CriticalSection);
    fn DeleteCriticalSection(cs: *mut CriticalSection);
}

/// `__crt_stdio_stream`: a `FILE*` seen as the record behind it.
#[derive(Clone, Copy)]
pub struct Stream(pub *mut StreamData);

impl Stream {
    pub const NULL: Stream = Stream(null_mut());

    pub fn from_file(file: *mut c_void) -> Stream {
        Stream(file.cast())
    }
    pub fn valid(self) -> bool {
        !self.0.is_null()
    }
    pub fn public_stream(self) -> *mut c_void {
        self.0.cast()
    }
    pub fn data(self) -> *mut StreamData {
        self.0
    }

    unsafe fn flags_atomic(self) -> &'static AtomicI32 {
        // `long` is 32 bits on every Windows target.
        &*(addr_of_mut!((*self.0).flags) as *const AtomicI32)
    }

    /// `__crt_interlocked_read(&_stream->_flags)`.
    pub unsafe fn get_flags(self) -> c_long {
        self.flags_atomic().load(Ordering::SeqCst)
    }
    pub unsafe fn is_in_use(self) -> bool {
        self.get_flags() & STREAM_ALLOCATED != 0
    }
    pub unsafe fn try_allocate(self) -> bool {
        self.flags_atomic().fetch_or(STREAM_ALLOCATED, Ordering::SeqCst) & STREAM_ALLOCATED == 0
    }
    /// Clears ALL flags, so the record is clean the next time it is allocated.
    pub unsafe fn deallocate(self) {
        self.flags_atomic().swap(0, Ordering::SeqCst);
    }
    pub unsafe fn lock(self) {
        _lock_file(self.public_stream());
    }
    pub unsafe fn unlock(self) {
        _unlock_file(self.public_stream());
    }
    pub unsafe fn has_any_of(self, flags: c_long) -> bool {
        self.get_flags() & flags != 0
    }
    pub unsafe fn has_all_of(self, flags: c_long) -> bool {
        self.get_flags() & flags == flags
    }
    /// Returns whether any of `flags` was ALREADY set.
    pub unsafe fn set_flags(self, flags: c_long) -> bool {
        self.flags_atomic().fetch_or(flags, Ordering::SeqCst) & flags != 0
    }
    /// Returns whether any of `flags` WAS set.
    pub unsafe fn unset_flags(self, flags: c_long) -> bool {
        self.flags_atomic().fetch_and(!flags, Ordering::SeqCst) & flags != 0
    }
    pub unsafe fn eof(self) -> bool {
        self.has_any_of(STREAM_EOF)
    }
    pub unsafe fn error(self) -> bool {
        self.has_any_of(STREAM_ERROR)
    }
    pub unsafe fn ctrl_z(self) -> bool {
        self.has_any_of(STREAM_CTRLZ)
    }
    pub unsafe fn has_crt_buffer(self) -> bool {
        self.has_any_of(STREAM_BUFFER_CRT)
    }
    pub unsafe fn has_user_buffer(self) -> bool {
        self.has_any_of(STREAM_BUFFER_USER)
    }
    pub unsafe fn has_temporary_buffer(self) -> bool {
        self.has_any_of(STREAM_BUFFER_STBUF)
    }
    pub unsafe fn has_setvbuf_buffer(self) -> bool {
        self.has_any_of(STREAM_BUFFER_SETVBUF)
    }
    pub unsafe fn has_big_buffer(self) -> bool {
        self.has_any_of(STREAM_BUFFER_CRT | STREAM_BUFFER_USER)
    }
    pub unsafe fn has_any_buffer(self) -> bool {
        self.has_any_of(STREAM_BUFFER_CRT | STREAM_BUFFER_USER | STREAM_BUFFER_NONE)
    }
    /// `__crt_interlocked_read(&_stream->_file)`.
    pub unsafe fn lowio_handle(self) -> c_int {
        (*(addr_of_mut!((*self.0).file) as *const AtomicI32)).load(Ordering::SeqCst)
    }
    pub unsafe fn is_string_backed(self) -> bool {
        self.get_flags() & STREAM_STRING != 0
    }
}

/// `__acrt_lock_stream_and_call`: `action` with the stream locked. The C guards
/// the unlock with SEH; this build's SEH is the dummy one, so the unlock is simply
/// sequenced after.
pub unsafe fn with_stream_locked<R>(stream: *mut c_void, action: impl FnOnce() -> R) -> R {
    _lock_file(stream);
    let r = action();
    _unlock_file(stream);
    r
}

/// `__acrt_stdio_is_initialized()`.
pub unsafe fn is_initialized() -> bool {
    !__piob.is_null()
}

/// `__acrt_stdio_reset_buffer`.
pub unsafe fn reset_buffer(stream: Stream) {
    (*stream.0).ptr = (*stream.0).base;
    (*stream.0).cnt = 0;
}

/// `__crt_stdio_path_requires_backslash(char const*)`. A trailing `\` that is the
/// trail byte of a DBCS character does not count, which is what the `_mbsrchr`
/// comparison decides.
pub unsafe fn path_requires_backslash_narrow(first: *const c_char) -> bool {
    extern "C" {
        fn strlen(s: *const c_char) -> usize;
        fn _mbsrchr(s: *const u8, c: c_uint) -> *mut u8;
    }
    let len = strlen(first);
    if len == 0 {
        return false;
    }
    let last = first.add(len - 1);
    if *last as u8 == b'\\' {
        return last.cast::<u8>() != _mbsrchr(first.cast(), b'\\' as c_uint).cast_const();
    }
    if *last as u8 == b'/' {
        return false;
    }
    true
}

/// `__crt_stdio_path_requires_backslash(wchar_t const*)`.
pub unsafe fn path_requires_backslash_wide(first: *const u16) -> bool {
    extern "C" {
        fn wcslen(s: *const u16) -> usize;
    }
    let len = wcslen(first);
    if len == 0 {
        return false;
    }
    let last = *first.add(len - 1);
    !(last == b'\\' as u16 || last == b'/' as u16)
}

// ── _file.cpp ───────────────────────────────────────────────────────────────

const fn std_stream(flags: c_long, file: c_long) -> StreamData {
    StreamData {
        ptr: null_mut(),
        base: null_mut(),
        cnt: 0,
        flags,
        file,
        charbuf: 0,
        bufsiz: 0,
        tmpfname: null_mut(),
        lock: CriticalSection::ZERO,
    }
}

/// `__crt_stdio_stream_data _iob[_IOB_ENTRIES]`: stdin, stdout, stderr.
#[no_mangle]
pub static mut _iob: [StreamData; IOB_ENTRIES] = [
    std_stream(STREAM_ALLOCATED | STREAM_READ, 0),
    std_stream(STREAM_ALLOCATED | STREAM_WRITE, 1),
    std_stream(STREAM_ALLOCATED | STREAM_WRITE, 2),
];

/// `__crt_stdio_stream_data** __piob`.
#[no_mangle]
pub static mut __piob: *mut *mut StreamData = null_mut();

/// `int _nstream` -- `_NSTREAM_` from the start under `CRTDLL`.
#[no_mangle]
pub static mut _nstream: c_int = NSTREAM;

crate::force_include!(__acrt_stdio_initializer);
crate::force_include!(__acrt_stdio_terminator);

#[no_mangle]
pub unsafe extern "C" fn __acrt_iob_func(id: c_uint) -> *mut c_void {
    addr_of_mut!(_iob[id as usize]).cast()
}

#[no_mangle]
pub unsafe extern "C" fn __acrt_initialize_stdio() -> c_int {
    // Try for `_nstream` slots; failing that, `_IOB_ENTRIES`; failing that, fail
    // CRT initialization.
    __piob = calloc(_nstream as usize, core::mem::size_of::<*mut StreamData>()).cast();
    if __piob.is_null() {
        _nstream = IOB_ENTRIES as c_int;
        __piob = calloc(_nstream as usize, core::mem::size_of::<*mut StreamData>()).cast();
        if __piob.is_null() {
            return -1;
        }
    }
    for i in 0..IOB_ENTRIES {
        let s = addr_of_mut!(_iob[i]);
        __acrt_InitializeCriticalSectionEx(addr_of_mut!((*s).lock), CORECRT_SPINCOUNT, 0);
        *__piob.add(i) = s;
        // _NO_CONSOLE_FILENO lets a caller tell "no console" from "failed to open".
        let os_handle = lowio::osfhnd(i as c_int);
        if os_handle == INVALID_HANDLE_VALUE || os_handle == NO_CONSOLE_FILENO || os_handle == 0 {
            (*s).file = NO_CONSOLE_FILENO as c_long;
        }
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn __acrt_uninitialize_stdio() {
    _flushall();
    _fcloseall();
    for i in 0..IOB_ENTRIES {
        let s = *__piob.add(i);
        __acrt_stdio_free_buffer_nolock(s.cast());
        DeleteCriticalSection(addr_of_mut!((*s).lock));
    }
    free(__piob.cast());
    __piob = null_mut();
}

#[no_mangle]
pub unsafe extern "C" fn _lock_file(stream: *mut c_void) {
    EnterCriticalSection(addr_of_mut!((*stream.cast::<StreamData>()).lock));
}

#[no_mangle]
pub unsafe extern "C" fn _unlock_file(stream: *mut c_void) {
    LeaveCriticalSection(addr_of_mut!((*stream.cast::<StreamData>()).lock));
}

#[no_mangle]
pub unsafe extern "C" fn _get_stream_buffer_pointers(
    public_stream: *mut c_void,
    base: *mut *mut *mut c_char,
    ptr: *mut *mut *mut c_char,
    count: *mut *mut c_int,
) -> c_int {
    if public_stream.is_null() {
        *_errno() = EINVAL;
        _invalid_parameter_noinfo();
        return EINVAL;
    }
    let s = public_stream.cast::<StreamData>();
    if !base.is_null() {
        *base = addr_of_mut!((*s).base);
    }
    if !ptr.is_null() {
        *ptr = addr_of_mut!((*s).ptr);
    }
    if !count.is_null() {
        *count = addr_of_mut!((*s).cnt);
    }
    0
}

// ── stream.cpp ──────────────────────────────────────────────────────────────

unsafe fn find_or_allocate_unused_stream_nolock() -> Stream {
    let first = __piob.add(IOB_ENTRIES);
    let last = first.add(_nstream as usize - IOB_ENTRIES);
    let mut it = first;
    while it != last {
        let existing = Stream(*it);
        if existing.valid() {
            if existing.is_in_use() {
                it = it.add(1);
                continue;
            }
            existing.lock();
            if !existing.try_allocate() {
                existing.unlock();
                it = it.add(1);
                continue;
            }
            return existing;
        }
        // No stream at this index yet. Everything is zero from calloc except the
        // handle and the lock.
        *it = calloc(1, core::mem::size_of::<StreamData>()).cast();
        if (*it).is_null() {
            break;
        }
        (**it).file = -1;
        __acrt_InitializeCriticalSectionEx(addr_of_mut!((**it).lock), CORECRT_SPINCOUNT, 0);
        let fresh = Stream(*it);
        // Always succeeds: nobody else has a pointer to it yet.
        fresh.try_allocate();
        fresh.lock();
        return fresh;
    }
    Stream::NULL
}

/// `__acrt_stdio_allocate_stream`: a free stream, allocated and LOCKED, or an
/// invalid one.
pub unsafe fn allocate_stream() -> Stream {
    __acrt_lock(STDIO_INDEX_LOCK);
    let stream = find_or_allocate_unused_stream_nolock();
    if stream.valid() {
        (*stream.0).cnt = 0;
        (*stream.0).tmpfname = null_mut();
        (*stream.0).ptr = null_mut();
        (*stream.0).base = null_mut();
        (*stream.0).file = -1;
    }
    __acrt_unlock(STDIO_INDEX_LOCK);
    stream
}

/// `__acrt_stdio_free_stream`. The caller still holds the lock.
pub unsafe fn free_stream(stream: Stream) {
    let d = stream.0;
    (*d).ptr = null_mut();
    (*d).base = null_mut();
    (*d).cnt = 0;
    (*d).file = -1;
    (*d).charbuf = 0;
    (*d).bufsiz = 0;
    (*d).tmpfname = null_mut();
    stream.deallocate();
}
