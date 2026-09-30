//! `malloc`/`calloc`/`realloc`/`free`/`_msize`/`_expand`/`_recalloc` and their
//! `*_base` bodies (eleven C files, one function pair each).
//!
//! Every public name forwards to its `*_base`, which is `#[inline(never)]`
//! like the C's `__declspec(noinline)`: the pairs must stay distinct bodies so
//! a user patch of `malloc` never lands in `_malloc_base` (and the CRT's own
//! calls to `_malloc_base` never reach a patched `malloc`).

use super::new_handler::{_callnewh, _query_new_mode};
use super::{
    acrt_heap, invalid_einval, noexc_enomem, GetLastError, HeapAlloc, HeapFree,
    HeapQueryInformation, HeapReAlloc, HeapSize, __acrt_errno_from_os_error, _errno, ENOMEM,
    HEAP_MAXREQ,
};
use core::ffi::{c_int, c_void};
use core::ptr::null_mut;

/// winnt.h `HEAP_ZERO_MEMORY`.
const HEAP_ZERO_MEMORY: u32 = 0x0000_0008;
/// winnt.h `HEAP_REALLOC_IN_PLACE_ONLY`.
const HEAP_REALLOC_IN_PLACE_ONLY: u32 = 0x0000_0010;
/// `HeapCompatibilityInformation`, member 0 of `HEAP_INFORMATION_CLASS`.
const HEAP_COMPATIBILITY_INFORMATION: c_int = 0;

// ── malloc.cpp / malloc_base.cpp ────────────────────────────────────────────

/// `void* _malloc_base(size_t)`.
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _malloc_base(size: usize) -> *mut c_void {
    // _VALIDATE_RETURN_NOEXC(_HEAP_MAXREQ >= size, ENOMEM, nullptr)
    if !(HEAP_MAXREQ >= size) {
        return noexc_enomem();
    }

    let actual_size = if size == 0 { 1 } else { size };

    loop {
        let block = HeapAlloc(acrt_heap(), 0, actual_size);
        if !block.is_null() {
            return block;
        }

        if _query_new_mode() == 0 || _callnewh(actual_size) == 0 {
            *_errno() = ENOMEM;
            return null_mut();
        }
    }
}

/// `void* malloc(size_t)`.
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn malloc(size: usize) -> *mut c_void {
    _malloc_base(size)
}

// ── calloc.cpp / calloc_base.cpp ────────────────────────────────────────────

/// `void* _calloc_base(size_t count, size_t size)`.
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _calloc_base(count: usize, size: usize) -> *mut c_void {
    // _VALIDATE_RETURN_NOEXC(count == 0 || (_HEAP_MAXREQ / count) >= size, ENOMEM, nullptr)
    if !(count == 0 || (HEAP_MAXREQ / count) >= size) {
        return noexc_enomem();
    }

    let requested_block_size = count.wrapping_mul(size);
    let actual_block_size = if requested_block_size == 0 { 1 } else { requested_block_size };

    loop {
        let block = HeapAlloc(acrt_heap(), HEAP_ZERO_MEMORY, actual_block_size);
        if !block.is_null() {
            return block;
        }

        if _query_new_mode() == 0 || _callnewh(actual_block_size) == 0 {
            *_errno() = ENOMEM;
            return null_mut();
        }
    }
}

/// `void* calloc(size_t count, size_t size)`.
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn calloc(count: usize, size: usize) -> *mut c_void {
    _calloc_base(count, size)
}

// ── realloc.cpp / realloc_base.cpp ──────────────────────────────────────────

/// `void* _realloc_base(void* block, size_t size)`.
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _realloc_base(block: *mut c_void, size: usize) -> *mut c_void {
    if block.is_null() {
        return _malloc_base(size);
    }

    if size == 0 {
        _free_base(block);
        return null_mut();
    }

    if !(HEAP_MAXREQ >= size) {
        return noexc_enomem();
    }

    loop {
        let new_block = HeapReAlloc(acrt_heap(), 0, block, size);
        if !new_block.is_null() {
            return new_block;
        }

        if _query_new_mode() == 0 || _callnewh(size) == 0 {
            *_errno() = ENOMEM;
            return null_mut();
        }
    }
}

/// `void* realloc(void* block, size_t size)`.
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn realloc(block: *mut c_void, size: usize) -> *mut c_void {
    _realloc_base(block, size)
}

// ── free.cpp / free_base.cpp ────────────────────────────────────────────────

/// `void _free_base(void*)`. `select_heap` is `__acrt_heap` (heap-mismatch
/// recovery is compiled out). A failing `HeapFree` sets errno.
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _free_base(block: *mut c_void) {
    if block.is_null() {
        return;
    }

    if HeapFree(acrt_heap(), 0, block) == 0 {
        let e = __acrt_errno_from_os_error(GetLastError());
        *_errno() = e;
    }
}

/// `void free(void*)`. The C's `volatile int extra_instructions_for_patching_
/// libraries` pads the body so hooking libraries (libtcmalloc) patch `free`
/// rather than `_free_base`; kept as the same volatile store.
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn free(block: *mut c_void) {
    let mut extra_instructions_for_patching_libraries: c_int = 0;
    core::ptr::write_volatile(&mut extra_instructions_for_patching_libraries, 0);
    _free_base(block);
}

// ── msize.cpp ───────────────────────────────────────────────────────────────

/// `size_t _msize_base(void*)`: NULL is EINVAL + handler, `(size_t)-1`.
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _msize_base(block: *mut c_void) -> usize {
    if block.is_null() {
        invalid_einval();
        return usize::MAX;
    }

    HeapSize(acrt_heap(), 0, block)
}

/// `size_t _msize(void*)`.
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _msize(block: *mut c_void) -> usize {
    _msize_base(block)
}

// ── expand.cpp ──────────────────────────────────────────────────────────────

/// `is_contraction_possible` (non-enclave arm): a block <= 16KB may be on the
/// low-fragmentation heap (compatibility value 2), which cannot shrink in
/// place; a failed query says "not possible" too.
unsafe fn is_contraction_possible(old_size: usize) -> bool {
    if old_size <= 0x4000 {
        let mut heap_type: i32 = -1; // LONG
        if HeapQueryInformation(
            acrt_heap(),
            HEAP_COMPATIBILITY_INFORMATION,
            (&mut heap_type as *mut i32).cast::<c_void>(),
            core::mem::size_of::<i32>(),
            null_mut(),
        ) == 0
        {
            return false;
        }
        return heap_type != 2;
    }
    true
}

/// `void* _expand_base(void* block, size_t size)` -- resizes strictly in
/// place; never calls the new handler.
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _expand_base(block: *mut c_void, size: usize) -> *mut c_void {
    if block.is_null() {
        invalid_einval();
        return null_mut();
    }
    if !(size <= HEAP_MAXREQ) {
        return noexc_enomem();
    }

    let old_size = HeapSize(acrt_heap(), 0, block);
    let new_size = if size == 0 { 1 } else { size };

    let new_block = HeapReAlloc(acrt_heap(), HEAP_REALLOC_IN_PLACE_ONLY, block, new_size);
    if !new_block.is_null() {
        return new_block;
    }

    // A refused contraction that the platform cannot do anyway (LFH block)
    // returns the original block.
    if new_size <= old_size && !is_contraction_possible(old_size) {
        return block;
    }

    let e = __acrt_errno_from_os_error(GetLastError());
    *_errno() = e;
    null_mut()
}

/// `void* _expand(void* block, size_t size)`.
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _expand(block: *mut c_void, size: usize) -> *mut c_void {
    _expand_base(block, size)
}

// ── recalloc.cpp ────────────────────────────────────────────────────────────

/// `void* _recalloc_base(void* block, size_t count, size_t size)`.
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _recalloc_base(block: *mut c_void, count: usize, size: usize) -> *mut c_void {
    if !(count == 0 || (HEAP_MAXREQ / count) >= size) {
        return noexc_enomem();
    }

    let old_block_size = if !block.is_null() { _msize_base(block) } else { 0 };
    let new_block_size = count.wrapping_mul(size);

    let new_block = _realloc_base(block, new_block_size);

    // Zero-fill from the OLD heap size, not the caller's old count*size.
    if !new_block.is_null() && old_block_size < new_block_size {
        core::ptr::write_bytes(
            new_block.cast::<u8>().add(old_block_size),
            0,
            new_block_size - old_block_size,
        );
    }

    new_block
}

/// `void* _recalloc(void* block, size_t count, size_t size)`.
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _recalloc(block: *mut c_void, count: usize, size: usize) -> *mut c_void {
    _recalloc_base(block, count, size)
}
