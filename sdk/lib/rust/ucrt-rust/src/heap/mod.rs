//! `sdk/lib/ucrt/heap/` -- the allocator every other part of ucrtbase (C and
//! Rust) calls. Replaces, in full, the 17 release-build files of
//! `UCRT_HEAP_SOURCES` (`heap/heap.cmake`): `calloc calloc_base expand free
//! free_base heapchk heapmin heapwalk heap_handle malloc malloc_base msize
//! new_handler new_mode realloc realloc_base recalloc`. `debug_heap.cpp` and
//! `debug_heap_hook.cpp` are `UCRTD_HEAP_SOURCES` -- the debug CRT only, never
//! part of ucrtbase -- and are left alone. `align.cpp` is `crate::align`.
//!
//! | C file | Rust |
//! |---|---|
//! | heap_handle.cpp | [`handle`]: `__acrt_heap`, `__acrt_initialize_heap`, `__acrt_uninitialize_heap`, `_get_heap_handle`, `__acrt_getheap` |
//! | malloc.cpp, malloc_base.cpp, calloc.cpp, calloc_base.cpp, realloc.cpp, realloc_base.cpp, free.cpp, free_base.cpp, msize.cpp, expand.cpp, recalloc.cpp | [`alloc`] |
//! | heapchk.cpp, heapmin.cpp, heapwalk.cpp | [`check`] |
//! | new_handler.cpp, new_mode.cpp | [`new_handler`] |
//!
//! The process heap is the ONLY heap: `__acrt_heap` is `GetProcessHeap()`, and
//! every body goes straight to the `Heap*` Win32 API. `_UCRT_HEAP_MISMATCH_*`
//! are all 0 in `corecrt_internal.h`, so `free_base.cpp`'s msvcrt-heap
//! recovery (`__acrt_get_msvcrt_heap_handle`, the `HeapValidate` probing
//! `select_heap`) is compiled out, and `select_heap` is `__acrt_heap`. The
//! `#ifdef _DEBUG` arms (`_malloc_dbg`...) are not ported: ucrtbase never
//! defines `_DEBUG`.
//!
//! ## Details that are silent if wrong
//!
//! * **Size 0 allocates 1 byte** in `_malloc_base`/`_calloc_base`/`_expand_base`
//!   (so `_msize(malloc(0)) == 1`), but `_realloc_base(p, 0)` FREES `p` and
//!   returns NULL (without touching errno) and `_realloc_base(NULL, n)` is
//!   `_malloc_base(n)`. `_recalloc_base(p, 0, n)` therefore frees too.
//! * **Oversize is `_VALIDATE_RETURN_NOEXC`**: `size > _HEAP_MAXREQ` (and the
//!   `calloc`/`_recalloc` product check `_HEAP_MAXREQ / count >= size`) sets
//!   ENOMEM and returns NULL WITHOUT the invalid-parameter handler. A NULL
//!   block to `_msize`/`_expand` is the other kind -- EINVAL WITH the handler,
//!   `(size_t)-1` / NULL.
//! * **The `_callnewh` retry loop runs only when `_query_new_mode() != 0`**, and
//!   calls the handler with the ADJUSTED size (1 for a 0 request); a handler
//!   returning nonzero retries the allocation, zero ends it with ENOMEM. The
//!   oversize check is before the loop, so the handler never sees an oversize
//!   request. `_expand` never calls the handler.
//! * **`_recalloc` zero-fills from the OLD `_msize`** (the heap's rounded size,
//!   not the caller's original count*size) up to the new count*size.
//! * **`_expand` returns the ORIGINAL block for a shrink the heap refused**,
//!   but only when the block may live on the low-fragmentation heap (old size
//!   <= 0x4000 and `HeapCompatibilityInformation` == 2) -- anything else that
//!   `HEAP_REALLOC_IN_PLACE_ONLY` refuses is NULL with errno from
//!   `GetLastError()`. `_free_base` also maps a `HeapFree` failure to errno.
//! * **The new handler is stored ENCODED** (`__crt_fast_encode_pointer`, i.e.
//!   XOR `__security_cookie`), and CRT startup seeds it with the encoded NULL
//!   through `__acrt_initialize_new_handler`. Storing it raw would make the
//!   seeded value decode to the cookie -- a garbage handler `malloc` would call.
//! * **`_heapwalk` resumes from the caller's `_pentry`**, validating it with
//!   `HeapValidate` only when `_useflag == _USEDENTRY`, and skips non-BUSY
//!   (free/region) entries. A NULL `_pentry` starts a walk; `HeapWalk` failing
//!   there is `_HEAPBADBEGIN`, later `ERROR_NO_MORE_ITEMS` is `_HEAPEND`.

use core::ffi::{c_int, c_ulong, c_void};

pub mod alloc;
pub mod check;
pub mod handle;
pub mod new_handler;

/// errno.h `EINVAL`.
const EINVAL: c_int = 22;
/// errno.h `ENOMEM`.
const ENOMEM: c_int = 12;

/// `_HEAP_MAXREQ` (`malloc.h`): the largest single allocation UCRT attempts.
#[cfg(target_pointer_width = "32")]
const HEAP_MAXREQ: usize = 0xFFFF_FFE0;
#[cfg(target_pointer_width = "64")]
const HEAP_MAXREQ: usize = 0xFFFF_FFFF_FFFF_FFE0;

/// `__acrt_heap_lock`, member 0 of `enum __acrt_lock_id`.
const ACRT_HEAP_LOCK: c_int = 0;

type Handle = *mut c_void;

extern "system" {
    fn GetProcessHeap() -> Handle;
    fn HeapAlloc(heap: Handle, flags: u32, bytes: usize) -> *mut c_void;
    fn HeapReAlloc(heap: Handle, flags: u32, mem: *mut c_void, bytes: usize) -> *mut c_void;
    fn HeapFree(heap: Handle, flags: u32, mem: *mut c_void) -> c_int;
    fn HeapSize(heap: Handle, flags: u32, mem: *const c_void) -> usize;
    fn HeapValidate(heap: Handle, flags: u32, mem: *const c_void) -> c_int;
    fn HeapCompact(heap: Handle, flags: u32) -> usize;
    fn HeapWalk(heap: Handle, entry: *mut check::ProcessHeapEntry) -> c_int;
    /// `HEAP_INFORMATION_CLASS` is a C enum: `int`.
    fn HeapQueryInformation(
        heap: Handle,
        class: c_int,
        info: *mut c_void,
        info_len: usize,
        return_len: *mut usize,
    ) -> c_int;
    fn GetLastError() -> u32;
}

extern "C" {
    fn _errno() -> *mut c_int;
    fn _invalid_parameter_noinfo();
    /// `corecrt_internal.h`: `int __cdecl __acrt_errno_from_os_error(unsigned long)`.
    fn __acrt_errno_from_os_error(err: c_ulong) -> c_int;
    fn __acrt_lock(lock: c_int);
    fn __acrt_unlock(lock: c_int);
}

/// The current value of `__acrt_heap`.
#[inline(always)]
unsafe fn acrt_heap() -> Handle {
    core::ptr::addr_of!(handle::__acrt_heap).read()
}

/// `_VALIDATE_RETURN_NOEXC(expr, ENOMEM, nullptr)`'s failure arm: errno only.
#[cold]
unsafe fn noexc_enomem() -> *mut c_void {
    *_errno() = ENOMEM;
    core::ptr::null_mut()
}

/// `_VALIDATE_RETURN(expr, EINVAL, ...)`'s failure arm, before the return.
#[cold]
unsafe fn invalid_einval() {
    *_errno() = EINVAL;
    _invalid_parameter_noinfo();
}
