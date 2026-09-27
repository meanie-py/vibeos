//! `_aligned_*` -- the eight exports plus their `*_base` internals. Replaces
//! `sdk/lib/ucrt/heap/align.cpp` in full.
//!
//! ## The block layout (must be bit-identical to Microsoft's)
//!
//! ```text
//! |1|___6___|2|3|4|_________5__________|_6_|
//!
//! 1 -> pointer to the start of the block `malloc` returned.
//! 2 -> the SAME value as 1, stored just behind the returned pointer.
//! 3 -> gap used to get 1 aligned on sizeof(void*).
//! 4 -> the returned (user) pointer.
//! 4+5 -> the user's data.
//! 6 -> wasted memory at front (between `malloc`'s block and the stored
//!      pointer) and rear (past the user's requested size, up to `align`).
//! ```
//!
//! Concretely: `malloc` gives back `ptr`. The returned pointer `retptr` is the
//! highest address `<= ptr + PTR_SZ + gap + align` (relative to `offset`) that
//! satisfies the alignment, and the `usize` immediately behind
//! `retptr - gap` (i.e. at `retptr - gap - PTR_SZ`) holds `ptr` verbatim. Every
//! other export walks backward through that one word to find the real block,
//! so this layout is a cross-module ABI: memory `_aligned_malloc`'d on one
//! side must be walkable by `_aligned_free`/`_aligned_msize` on the other, and
//! vice versa.
//!
//! ## Details that are silent if wrong
//!
//! * **Alignment must be a power of two** (`IS_2_POW_N`), checked before
//!   anything else; `offset` must be `0` or `< size`. Both violations are
//!   `_VALIDATE_RETURN` (EINVAL, iph called, `nullptr`) -- not
//!   `_VALIDATE_RETURN_NOEXC`.
//! * **The effective alignment is `max(align, PTR_SZ) - 1`** used as a mask,
//!   so the mask is always all-ones in its low bits -- an `align` less than a
//!   pointer never narrows anything.
//! * **A `size`/`block_size` overflow is detected by the NOEXC comparison
//!   `size <= block_size`**, not by any explicit overflow check: on wraparound
//!   `block_size` becomes smaller than `size`, which is caught. This is a
//!   `_VALIDATE_RETURN_NOEXC` (ENOMEM, no iph call), unlike the two checks
//!   above.
//! * **`_aligned_offset_realloc_base(NULL, ...)` forwards to
//!   `_aligned_offset_malloc_base`**, and `_aligned_offset_realloc_base(p, 0,
//!   ...)` frees `p` and returns `NULL` -- exactly `realloc`'s own special
//!   cases, reimplemented rather than shared.
//! * **Realloc prefers `_expand` in place** when the existing block already
//!   has room for the worst-case new footprint (`stptr + align + PTR_SZ + gap
//!   < block`, i.e. there is no risk the new alignment/offset would need data
//!   that starts before the old block); only then does it try `_expand`, and
//!   only falls back to `malloc` + `memmove` + `free` if `_expand` fails
//!   (restoring `errno` first, since `_expand` may have clobbered it) or if
//!   the layout couldn't be preserved in place to begin with.
//! * **Even after a successful in-place `_expand`, the data may still need to
//!   move** within the (possibly bigger) block, if the new `align`/`offset`
//!   put the aligned point somewhere else. The C checks this explicitly
//!   (`ptr == block - diff && no realignment needed`) and returns `block`
//!   unchanged only then; every other path `memmove`s.
//! * **`_aligned_msize_base` recomputes the header/footer split from
//!   `align`/`offset` as given by the CALLER**, not from anything stored in
//!   the block -- passing the wrong `alignment`/`offset` to `_aligned_msize`
//!   silently returns a wrong size rather than erroring (there is nothing in
//!   the block to check against; this is Microsoft's contract, not a bug we
//!   introduced).
//! * **`_aligned_recalloc`/`_aligned_offset_recalloc` zero-fill using the OLD
//!   size (via `_aligned_msize_base` on the block BEFORE reallocating)**, so
//!   the fill starts exactly where the previous user data ended, never at 0.
//!   `count == 0` is not rejected up front; the overflow check
//!   `_HEAP_MAXREQ / count >= size` is skipped entirely when `count == 0`
//!   (division by zero avoided the same way the C avoids it), so
//!   `size * 0 == 0` sails through.
//! * **`_aligned_free_base(NULL)` is a no-op**; every other function's NULL
//!   handling is documented above per-function, not implied.
//! * `_aligned_offset_malloc`/`_aligned_offset_realloc`/
//!   `_aligned_offset_recalloc`/`_aligned_realloc`/`_aligned_recalloc` are all
//!   `_CRTRESTRICT` (no aliasing contract to honor in Rust, noted for parity).
//! * The `#ifdef _DEBUG` arms (`_aligned_free_dbg` etc.) are NOT ported: this
//!   tree never defines `_DEBUG` for ucrtbase, so the C's release path
//!   (straight to `*_base`) is the only one that can ever run.

use core::ffi::c_void;

/// errno.h `EINVAL`.
const EINVAL: i32 = 22;
/// errno.h `ENOMEM`.
const ENOMEM: i32 = 12;

/// `PTR_SZ` -- `sizeof(void*)`, i.e. `size_of::<usize>()` for a pointer-sized
/// scalar exactly as the C's `uintptr_t` arithmetic assumes.
const PTR_SZ: usize = core::mem::size_of::<usize>();

extern "C" {
    fn _errno() -> *mut i32;
    fn _invalid_parameter_noinfo();
    fn malloc(size: usize) -> *mut c_void;
    fn free(p: *mut c_void);
    /// `sdk/lib/ucrt/heap/expand.cpp` (still C): may clobber `errno` even on
    /// failure, which is why every caller here saves/restores it around the
    /// call, exactly as the C does.
    fn _expand(block: *mut c_void, size: usize) -> *mut c_void;
    /// `sdk/lib/ucrt/heap/msize.cpp` (still C).
    fn _msize_base(block: *mut c_void) -> usize;
}

/// `IS_2_POW_N(X)`.
#[inline]
fn is_2_pow_n(x: usize) -> bool {
    x != 0 && (x & (x.wrapping_sub(1))) == 0
}

/// `_VALIDATE_RETURN(expr, EINVAL, nullptr)`'s failure arm.
#[cold]
unsafe fn invalid_parameter_einval() -> *mut c_void {
    *_errno() = EINVAL;
    _invalid_parameter_noinfo();
    core::ptr::null_mut()
}

/// `_VALIDATE_RETURN_NOEXC(expr, ENOMEM, nullptr)`'s failure arm.
#[cold]
unsafe fn errno_enomem() -> *mut c_void {
    *_errno() = ENOMEM;
    core::ptr::null_mut()
}

/// The word stored immediately behind `retptr - gap`: `((uintptr_t
/// *)(retptr - gap))[-1]`.
#[inline]
unsafe fn header_slot(retptr: usize, gap: usize) -> *mut usize {
    (retptr - gap - PTR_SZ) as *mut usize
}

/// `_aligned_offset_malloc_base`.
unsafe fn aligned_offset_malloc_base(size: usize, mut align: usize, offset: usize) -> *mut c_void {
    if !is_2_pow_n(align) {
        return invalid_parameter_einval();
    }
    if !(offset == 0 || offset < size) {
        return invalid_parameter_einval();
    }

    align = (if align > PTR_SZ { align } else { PTR_SZ }) - 1;

    // gap = number of bytes needed to round up offset to align with PTR_SZ.
    let gap = 0usize.wrapping_sub(offset) & (PTR_SZ - 1);

    let nonuser_size = PTR_SZ + gap + align;
    let block_size = nonuser_size.wrapping_add(size);
    if !(size <= block_size) {
        return errno_enomem();
    }

    let ptr = malloc(block_size) as usize;
    if ptr == 0 {
        return core::ptr::null_mut();
    }

    let retptr = ((ptr + nonuser_size + offset) & !align).wrapping_sub(offset);
    *header_slot(retptr, gap) = ptr;

    retptr as *mut c_void
}

/// `_aligned_malloc_base`.
#[inline]
unsafe fn aligned_malloc_base(size: usize, alignment: usize) -> *mut c_void {
    aligned_offset_malloc_base(size, alignment, 0)
}

/// `_aligned_msize_base`.
unsafe fn aligned_msize_base(block: *mut c_void, mut align: usize, offset: usize) -> usize {
    if block.is_null() {
        *_errno() = EINVAL;
        _invalid_parameter_noinfo();
        return usize::MAX; // static_cast<size_t>(-1)
    }

    let mut p = block as usize;
    // p is one position behind memblock; the value there is the start of the
    // real allocated block.
    p = (p & !(PTR_SZ - 1)) - PTR_SZ;
    let ptr = *(p as *const usize);

    let total_size = _msize_base(ptr as *mut c_void);
    let header_size = block as usize - ptr;
    let gap = 0usize.wrapping_sub(offset) & (PTR_SZ - 1);
    align = (if align > PTR_SZ { align } else { PTR_SZ }) - 1;
    let footer_size = gap + align + PTR_SZ - header_size;
    total_size - header_size - footer_size
}

/// `_aligned_offset_realloc_base`.
unsafe fn aligned_offset_realloc_base(
    block: *mut c_void,
    size: usize,
    mut align: usize,
    offset: usize,
) -> *mut c_void {
    // Special cases, exactly as `realloc` itself:
    if block.is_null() {
        return aligned_offset_malloc_base(size, align, offset);
    }
    if size == 0 {
        aligned_free_base(block);
        return core::ptr::null_mut();
    }

    if !is_2_pow_n(align) {
        return invalid_parameter_einval();
    }
    if !(offset == 0 || offset < size) {
        return invalid_parameter_einval();
    }

    let block_addr = block as usize;

    // stptr -> pointer to the start of the pointer-to-block-start slot, then
    // dereferenced to the real block start.
    let mut stptr = (block_addr & !(PTR_SZ - 1)) - PTR_SZ;
    stptr = *(stptr as *const usize);

    align = (if align > PTR_SZ { align } else { PTR_SZ }) - 1;
    let gap = 0usize.wrapping_sub(offset) & (PTR_SZ - 1);

    let diff = block_addr - stptr;
    // Amount of the existing block that must be preserved: the lesser of
    // what's there and what's wanted.
    let movsz_avail = _msize_base(stptr as *mut c_void) - (block_addr - stptr);
    let movsz = if movsz_avail > size { size } else { movsz_avail };
    let reqsz = PTR_SZ + gap + align + size;

    if !(size <= reqsz) {
        return errno_enomem();
    }

    let ptr;
    let mut b_free = false;

    if stptr + align + PTR_SZ + gap < block_addr {
        ptr = malloc(reqsz) as usize;
        if ptr == 0 {
            return core::ptr::null_mut();
        }
        b_free = true;
    } else {
        // _expand may clobber errno even on failure; save/restore around it.
        let save_errno = *_errno();
        let expanded = _expand(stptr as *mut c_void, reqsz);
        if expanded.is_null() {
            *_errno() = save_errno;
            ptr = malloc(reqsz) as usize;
            if ptr == 0 {
                return core::ptr::null_mut();
            }
            b_free = true;
        } else {
            ptr = expanded as usize;
            stptr = ptr;
        }
    }

    if ptr == block_addr.wrapping_sub(diff)
        && (block_addr.wrapping_add(gap).wrapping_add(offset) & !align) == 0
    {
        return block;
    }

    let retptr = ((ptr + PTR_SZ + gap + align + offset) & !align).wrapping_sub(offset);
    core::ptr::copy((stptr + diff) as *const u8, retptr as *mut u8, movsz);
    if b_free {
        free(stptr as *mut c_void);
    }

    *header_slot(retptr, gap) = ptr;
    retptr as *mut c_void
}

/// `_aligned_realloc_base`.
#[inline]
unsafe fn aligned_realloc_base(block: *mut c_void, size: usize, alignment: usize) -> *mut c_void {
    aligned_offset_realloc_base(block, size, alignment, 0)
}

/// `_HEAP_MAXREQ` (`corecrt_internal.h`): the largest single allocation UCRT
/// will attempt, used only to bound the `count * size` overflow check.
#[cfg(target_pointer_width = "32")]
const HEAP_MAXREQ: usize = 0xFFFF_FFE0;
#[cfg(target_pointer_width = "64")]
const HEAP_MAXREQ: usize = 0xFFFF_FFFF_FFFF_FFE0;

/// `_aligned_offset_recalloc_base`.
unsafe fn aligned_offset_recalloc_base(
    block: *mut c_void,
    count: usize,
    size: usize,
    align: usize,
    offset: usize,
) -> *mut c_void {
    if count > 0 && !(HEAP_MAXREQ / count >= size) {
        return errno_enomem();
    }

    let user_size = size.wrapping_mul(count);

    let start_fill = if !block.is_null() {
        aligned_msize_base(block, align, offset)
    } else {
        0
    };

    let retptr = aligned_offset_realloc_base(block, user_size, align, offset);

    if !retptr.is_null() && start_fill < user_size {
        core::ptr::write_bytes(
            (retptr as *mut u8).add(start_fill),
            0,
            user_size - start_fill,
        );
    }
    retptr
}

/// `_aligned_recalloc_base`.
#[inline]
unsafe fn aligned_recalloc_base(
    block: *mut c_void,
    count: usize,
    size: usize,
    alignment: usize,
) -> *mut c_void {
    aligned_offset_recalloc_base(block, count, size, alignment, 0)
}

/// `_aligned_free_base`.
unsafe fn aligned_free_base(block: *mut c_void) {
    if block.is_null() {
        return;
    }
    let mut ptr = block as usize;
    ptr = (ptr & !(PTR_SZ - 1)) - PTR_SZ;
    ptr = *(ptr as *const usize);
    free(ptr as *mut c_void);
}

// ── the eight exports ───────────────────────────────────────────────────────
// Each is `__declspec(noinline)` in the C "so there is enough space in each
// export to host a patch"; that's an MSVC hotpatching concern with no Rust
// analogue, so it is not reproduced.

#[no_mangle]
pub unsafe extern "C" fn _aligned_free(block: *mut c_void) {
    aligned_free_base(block);
}

#[no_mangle]
pub unsafe extern "C" fn _aligned_malloc(size: usize, alignment: usize) -> *mut c_void {
    aligned_malloc_base(size, alignment)
}

#[no_mangle]
pub unsafe extern "C" fn _aligned_msize(block: *mut c_void, alignment: usize, offset: usize) -> usize {
    aligned_msize_base(block, alignment, offset)
}

#[no_mangle]
pub unsafe extern "C" fn _aligned_offset_malloc(
    size: usize,
    alignment: usize,
    offset: usize,
) -> *mut c_void {
    aligned_offset_malloc_base(size, alignment, offset)
}

#[no_mangle]
pub unsafe extern "C" fn _aligned_offset_realloc(
    block: *mut c_void,
    size: usize,
    alignment: usize,
    offset: usize,
) -> *mut c_void {
    aligned_offset_realloc_base(block, size, alignment, offset)
}

#[no_mangle]
pub unsafe extern "C" fn _aligned_offset_recalloc(
    block: *mut c_void,
    count: usize,
    size: usize,
    alignment: usize,
    offset: usize,
) -> *mut c_void {
    aligned_offset_recalloc_base(block, count, size, alignment, offset)
}

#[no_mangle]
pub unsafe extern "C" fn _aligned_realloc(
    block: *mut c_void,
    size: usize,
    alignment: usize,
) -> *mut c_void {
    aligned_realloc_base(block, size, alignment)
}

#[no_mangle]
pub unsafe extern "C" fn _aligned_recalloc(
    block: *mut c_void,
    count: usize,
    size: usize,
    alignment: usize,
) -> *mut c_void {
    aligned_recalloc_base(block, count, size, alignment)
}
