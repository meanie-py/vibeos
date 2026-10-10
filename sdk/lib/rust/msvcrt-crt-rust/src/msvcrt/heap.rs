// rustconv-assembly: whole-tu
use crate::prelude::*;
use msvcrt_sys::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----

// heap.c:32-33 — `#define LOCK_HEAP _lock(_HEAP_LOCK)` / `UNLOCK_HEAP _unlock(...)`.
// `macro_rules!`, not a helper `fn`: an out-of-line wrapper for a C macro is an
// optimization barrier that has uncloaked latent ABI mismatches into `ud2` before now.
// The translation emitted these as `pub const LOCK_HEAP: () = _lock(_HEAP_LOCK);` — a
// const initialised by a call, which is neither valid nor a lock.
macro_rules! LOCK_HEAP {
    () => {
        _lock(_HEAP_LOCK as core::ffi::c_int)
    };
}
macro_rules! UNLOCK_HEAP {
    () => {
        _unlock(_HEAP_LOCK as core::ffi::c_int)
    };
}

// heap.c:36-40, the `_aligned` pointer algebra, both `macro_rules!` for the same reason.
// SAVED_PTR steps back one pointer-width from the user block and rounds DOWN to the
// pointer alignment; ALIGN_PTR rounds a raw allocation UP past `alignment + sizeof(void*)
// + offset` and then subtracts the offset back off, so the pointer HANDED OUT is aligned
// rather than the block. Kept as the C's own bit arithmetic rather than reached for with
// `align_offset`, which is a different computation.
macro_rules! SAVED_PTR {
    ($x:expr) => {
        ((($x as usize) - core::mem::size_of::<*mut core::ffi::c_void>())
            & !(core::mem::size_of::<*mut core::ffi::c_void>() - 1))
            as *mut core::ffi::c_void
    };
}
macro_rules! ALIGN_PTR {
    ($ptr:expr, $alignment:expr, $offset:expr) => {
        (((($ptr as usize)
            + $alignment
            + core::mem::size_of::<*mut core::ffi::c_void>()
            + $offset)
            & !($alignment - 1))
            - $offset) as *mut core::ffi::c_void
    };
}

// `NtCurrentTeb()` is a FORCEINLINE macro in C reading the TEB self-pointer, so there is no
// symbol to import and no bindgen output to take. Same per-arch asm as the established copy
// in rtl-actctx/src/shim.rs; `pure` + `nomem` because the self-pointer is fixed for the
// lifetime of the thread.
#[inline(always)]
#[allow(non_snake_case)]
unsafe fn NtCurrentTeb() -> *mut _TEB {
    let teb: *mut _TEB;
    #[cfg(target_arch = "x86")]
    core::arch::asm!("mov {}, fs:[0x18]", out(reg) teb, options(nostack, nomem, pure));
    #[cfg(target_arch = "x86_64")]
    core::arch::asm!("mov {}, gs:[0x30]", out(reg) teb, options(nostack, nomem, pure));
    #[cfg(target_arch = "aarch64")]
    core::arch::asm!("mov {}, x18", out(reg) teb, options(nostack, nomem, pure));
    teb
}

pub static mut heap: HANDLE = core::ptr::null_mut();
pub static mut sb_heap: HANDLE = core::ptr::null_mut();

pub type MSVCRT_new_handler_func = Option<unsafe extern "C" fn(size: usize) -> i32>;

pub static mut MSVCRT_new_handler: MSVCRT_new_handler_func = None;
pub static mut MSVCRT_new_mode: LONG = 0;
pub static mut MSVCRT_amblksiz: u32 = 16;
pub static mut MSVCRT_sbh_threshold: usize = 0;

pub const SB_HEAP_ALIGN: usize = 16;

unsafe extern "C" fn msvcrt_heap_alloc(flags: u32, size: usize) -> *mut core::ffi::c_void {
    if size < MSVCRT_sbh_threshold {
        let temp = RtlAllocateHeap(
            sb_heap,
            flags,
            size.wrapping_add(core::mem::size_of::<*mut core::ffi::c_void>())
                .wrapping_add(SB_HEAP_ALIGN) as SIZE_T,
        );
        if temp.is_null() {
            return core::ptr::null_mut();
        }

        let memblock = ALIGN_PTR!(temp, SB_HEAP_ALIGN, 0);
        let saved = SAVED_PTR!(memblock) as *mut *mut core::ffi::c_void;
        *saved = temp;
        return memblock;
    }

    // `heap = NtCurrentTeb()->Peb->ProcessHeap;` — TWO hops. The paid translation dropped
    // the second one here and wrote the PEB pointer itself as the heap handle, while
    // getting the SAME expression right in msvcrt_heap_realloc below. The guarded offsets
    // cannot see this: they prove where `Peb` and `ProcessHeap` LIVE, not how many times
    // the translation dereferenced.
    if heap.is_null() {
        heap = (*(*NtCurrentTeb()).Peb).ProcessHeap;
    }

    RtlAllocateHeap(heap, flags, size as SIZE_T)
}
unsafe extern "C" fn msvcrt_heap_realloc(flags: u32, ptr: *mut core::ffi::c_void, size: usize) -> *mut core::ffi::c_void {
    // Spelled exactly as in msvcrt_heap_alloc above. The translation reached this one via
    // `.as_ref().unwrap()`, which is a panic path the C does not have — msvcrt has no
    // unwinder and a panic here is a dead process, so the raw deref is the faithful form.
    if heap.is_null() {
        heap = (*(*NtCurrentTeb()).Peb).ProcessHeap;
    }
    if !sb_heap.is_null() && !ptr.is_null() && RtlValidateHeap(heap, 0, ptr) == 0 {
        let memblock: *mut core::ffi::c_void;
        let temp: *mut core::ffi::c_void;
        let mut saved: *mut *mut core::ffi::c_void;
        let old_padding: usize;
        let new_padding: usize;
        let old_size: usize;

        saved = SAVED_PTR!(ptr) as *mut *mut core::ffi::c_void;
        old_padding = (ptr as *mut u8).offset_from(*saved as *mut u8) as usize;
        old_size = RtlSizeHeap(sb_heap, 0, *saved) as usize;
        if old_size == (0usize).wrapping_sub(1) {
            return core::ptr::null_mut();
        }
        let old_size = old_size - old_padding;

        temp = RtlReAllocateHeap(
            sb_heap,
            flags,
            *saved,
            (size + core::mem::size_of::<*mut core::ffi::c_void>() + SB_HEAP_ALIGN) as SIZE_T,
        );
        if temp.is_null() {
            return core::ptr::null_mut();
        }

        memblock = ALIGN_PTR!(temp, SB_HEAP_ALIGN, 0);
        saved = SAVED_PTR!(memblock) as *mut *mut core::ffi::c_void;
        new_padding = (memblock as *mut u8).offset_from(temp as *mut u8) as usize;

        if new_padding != old_padding {
            memmove(memblock, (temp as *mut u8).add(old_padding) as *mut core::ffi::c_void, if old_size > size { size } else { old_size });
        }

        *saved = temp;
        return memblock;
    }

    return RtlReAllocateHeap(heap, flags, ptr, size as SIZE_T);
}
unsafe extern "C" fn msvcrt_heap_free(ptr: *mut core::ffi::c_void) -> i32 {
    // Third spelling of the same two-hop C, and the worst of them: the translation walked
    // the PEB by a HARD-CODED 0x18, which is ProcessHeap's i386 offset. On amd64 and arm64
    // it is 0x30, so this read a neighbouring PEB field as the heap handle on two of three
    // arches. Offsets belong to the guarded island, never to a literal in a body.
    if heap.is_null() {
        heap = (*(*NtCurrentTeb()).Peb).ProcessHeap;
    }
    if !sb_heap.is_null() && !ptr.is_null() && RtlValidateHeap(heap, 0, ptr) == 0 {
        let saved = SAVED_PTR!(ptr) as *mut *mut core::ffi::c_void;
        return RtlFreeHeap(sb_heap, 0, *saved) as i32;
    }

    // RtlFreeHeap returns BOOLEAN (a uchar); the C's `static int msvcrt_heap_free` widens it
    // on return, so the cast is the implicit conversion made explicit, not a reinterpretation.
    RtlFreeHeap(heap, 0, ptr) as i32
}
unsafe extern "C" fn msvcrt_heap_size(ptr: *mut core::ffi::c_void) -> usize {
    // Fourth and last spelling of the same two-hop C; this one simply stopped at the PEB.
    if heap.is_null() {
        heap = (*(*NtCurrentTeb()).Peb).ProcessHeap;
    }
    if !sb_heap.is_null() && !ptr.is_null() && RtlValidateHeap(heap, 0, ptr) == 0 {
        let saved = SAVED_PTR!(ptr) as *mut *mut core::ffi::c_void;
        return RtlSizeHeap(sb_heap, 0, *saved) as usize;
    }

    RtlSizeHeap(heap, 0, ptr) as usize
}
#[no_mangle]
pub unsafe extern "C" fn _callnewh(size: usize) -> i32 {
    let mut ret = 0;
    let handler = MSVCRT_new_handler;
    // `MSVCRT_new_handler_func` is a nullable C function pointer, which Rust spells
    // `Option<fn>` — so the C's `if(handler)` is `is_some()`, and the call is
    // `unwrap_unchecked()`. Plain `unwrap()` would add a panic path the C does not have,
    // in a function reached from every failing malloc.
    if handler.is_some() {
        ret = if (handler.unwrap_unchecked())(size) != 0 {
            1
        } else {
            0
        };
    }
    ret
}

#[no_mangle]
pub unsafe extern "C" fn operator_new(size: usize) -> *mut core::ffi::c_void {
    let mut retval: *mut core::ffi::c_void;

    loop {
        retval = msvcrt_heap_alloc(0, size);
        if !retval.is_null() {
            return retval;
        }
        if _callnewh(size) == 0 {
            break;
        }
    }

    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn operator_new_dbg(
    size: usize,
    r#type: i32,
    file: *const i8,
    line: i32,
) -> *mut core::ffi::c_void {
    operator_new(size)
}

#[no_mangle]
pub unsafe extern "C" fn operator_delete(mem: *mut core::ffi::c_void) {
    msvcrt_heap_free(mem);
}

#[no_mangle]
pub unsafe extern "C" fn _query_new_handler() -> MSVCRT_new_handler_func {
    MSVCRT_new_handler
}

#[no_mangle]
pub unsafe extern "C" fn _query_new_mode() -> i32 {
    MSVCRT_new_mode
}
#[no_mangle]
pub unsafe extern "C" fn _set_new_handler(func: MSVCRT_new_handler_func) -> MSVCRT_new_handler_func {
    let old_handler: MSVCRT_new_handler_func;
    LOCK_HEAP!();
    old_handler = MSVCRT_new_handler;
    MSVCRT_new_handler = func;
    UNLOCK_HEAP!();
    return old_handler;
}
#[no_mangle]
pub unsafe extern "C" fn set_new_handler(func: *mut core::ffi::c_void) -> MSVCRT_new_handler_func {
    // Faithful to the C, which ignores `func` entirely: it clears the handler and returns
    // NULL. `None` is the nullable function pointer, not `null_mut()`.
    let _ = func;
    _set_new_handler(None);
    None
}
#[no_mangle]
pub unsafe extern "C" fn _set_new_mode(mode: i32) -> i32 {
    if MSVCRT_CHECK_PMT!(mode == 0 || mode == 1) == 0 {
        return -1;
    }
    InterlockedExchange(core::ptr::addr_of_mut!(MSVCRT_new_mode), mode)
}

#[no_mangle]
pub unsafe extern "C" fn _expand(mem: *mut core::ffi::c_void, size: usize) -> *mut core::ffi::c_void {
    return msvcrt_heap_realloc(HEAP_REALLOC_IN_PLACE_ONLY, mem, size);
}
#[no_mangle]
pub unsafe extern "C" fn _heapchk() -> i32 {
    if HeapValidate(heap, 0, core::ptr::null_mut()) == 0 ||
       (!sb_heap.is_null() && HeapValidate(sb_heap, 0, core::ptr::null_mut()) == 0)
    {
        msvcrt_set_errno(GetLastError() as i32);
        return _HEAPBADNODE;
    }
    return _HEAPOK;
}

#[no_mangle]
pub unsafe extern "C" fn _heapmin() -> i32 {
    if HeapCompact(heap, 0) == 0 ||
        (!sb_heap.is_null() && HeapCompact(sb_heap, 0) == 0)
    {
        if GetLastError() != ERROR_CALL_NOT_IMPLEMENTED {
            msvcrt_set_errno(GetLastError() as i32);
        }
        return -1;
    }
    0
}
#[no_mangle]
pub unsafe extern "C" fn _heapwalk(next: *mut _HEAPINFO) -> i32 {
    let mut phe: PROCESS_HEAP_ENTRY = core::mem::zeroed();

    if !sb_heap.is_null() {
        // FIXME: small blocks heap not supported
    }

    // The `_HEAPINFO` the caller owns and the `PROCESS_HEAP_ENTRY` HeapWalk advances hold the
    // same three facts at different widths — `int*` vs `PVOID`, `size_t` vs `DWORD`, `int` vs
    // `WORD` — so each hop is a narrowing the C makes implicitly. Written out rather than
    // papered over with a blanket cast, because `cbData` is the one that actually narrows:
    // a >4GB block reports truncated on 64-bit, exactly as msvcrt's own C does.
    LOCK_HEAP!();
    phe.lpData = (*next)._pentry as PVOID;
    phe.cbData = (*next)._size as DWORD;
    phe.wFlags = if (*next)._useflag == _USEDENTRY as core::ffi::c_int {
        PROCESS_HEAP_ENTRY_BUSY as msvcrt_sys::WORD
    } else {
        0
    };

    if !phe.lpData.is_null() && (phe.wFlags & PROCESS_HEAP_ENTRY_BUSY as msvcrt_sys::WORD) != 0 &&
        HeapValidate(heap, 0, phe.lpData) == 0
    {
        UNLOCK_HEAP!();
        msvcrt_set_errno(GetLastError() as i32);
        return _HEAPBADNODE;
    }

    loop {
        if HeapWalk(heap, &mut phe) == 0 {
            UNLOCK_HEAP!();
            if GetLastError() == ERROR_NO_MORE_ITEMS {
                return _HEAPEND;
            }
            msvcrt_set_errno(GetLastError() as i32);
            if phe.lpData.is_null() {
                return _HEAPBADBEGIN;
            }
            return _HEAPBADNODE;
        }
        if (phe.wFlags & (PROCESS_HEAP_REGION | PROCESS_HEAP_UNCOMMITTED_RANGE) as msvcrt_sys::WORD) == 0 {
            break;
        }
    }

    UNLOCK_HEAP!();
    (*next)._pentry = phe.lpData as *mut core::ffi::c_int;
    (*next)._size = phe.cbData as usize;
    (*next)._useflag = if (phe.wFlags & PROCESS_HEAP_ENTRY_BUSY as msvcrt_sys::WORD) != 0 {
        _USEDENTRY as core::ffi::c_int
    } else {
        _FREEENTRY as core::ffi::c_int
    };
    _HEAPOK
}
#[no_mangle]
pub unsafe extern "C" fn _heapset(value: u32) -> i32 {
    let mut retval: i32;
    // The C names this local `heap`, which would shadow this file's `static mut heap`; Rust
    // forbids that outright (E0530) rather than silently taking the local, which is the one
    // mercy in this function.
    let mut hinfo: _HEAPINFO = core::mem::zeroed();

    memset(&mut hinfo as *mut _ as *mut core::ffi::c_void, 0, core::mem::size_of::<_HEAPINFO>());
    LOCK_HEAP!();
    // Every constant below arrived from the paid call as an INVENTED LITERAL: `!= 1` for
    // `!= _HEAPOK` (-2), `== 5` for `== _HEAPEND` (-5), and `1` for `_HEAPOK` (-2). Only
    // `_FREEENTRY` happened to be right, because it is 0. Shipped as written, the loop body
    // would never execute — _heapwalk returns -2 and the guard breaks immediately — so
    // `_heapset` would quietly fill nothing and return a value from the wrong namespace.
    //
    // These were NOT sealed, and that is the lesson: at the time of the call none of the
    // `_HEAP*` family was in the msvcrt-sys island, and `--seal-facts` can only seal a
    // constant the island carries. An absent constant is not protected, it is invented. The
    // island now carries all of them, so a re-translation of this file would be sealed.
    loop {
        retval = _heapwalk(&mut hinfo);
        if retval != _HEAPOK {
            break;
        }
        if hinfo._useflag == _FREEENTRY as core::ffi::c_int {
            memset(hinfo._pentry as *mut core::ffi::c_void, value as i32, hinfo._size);
        }
    }
    UNLOCK_HEAP!();
    if retval == _HEAPEND { _HEAPOK } else { retval }
}

#[no_mangle]
pub unsafe extern "C" fn _heapadd(mem: *mut core::ffi::c_void, size: usize) -> i32 {
    *_errno() = ENOSYS as core::ffi::c_int;
    return -1;
}

#[no_mangle]
pub unsafe extern "C" fn _get_heap_handle() -> isize {
    heap as isize
}

#[no_mangle]
pub unsafe extern "C" fn _msize(mem: *mut core::ffi::c_void) -> usize {
    let size = msvcrt_heap_size(mem);
    if size == !0 {
    }
    return size;
}

#[no_mangle]
pub unsafe extern "C" fn calloc(count: usize, size: usize) -> *mut core::ffi::c_void {
    let bytes = count.wrapping_mul(size);

    if size != 0 && bytes / size != count {
        *_errno() = ENOMEM as core::ffi::c_int;
        return core::ptr::null_mut();
    }

    msvcrt_heap_alloc(HEAP_ZERO_MEMORY, bytes)
}

#[no_mangle]
pub unsafe extern "C" fn free(ptr: *mut core::ffi::c_void) {
    msvcrt_heap_free(ptr);
}

#[no_mangle]
pub unsafe extern "C" fn malloc(size: usize) -> *mut core::ffi::c_void {
    let mut ret: *mut core::ffi::c_void;

    loop {
        ret = msvcrt_heap_alloc(0, size);
        if !ret.is_null() || MSVCRT_new_mode == 0 {
            break;
        }
        if _callnewh(size) == 0 {
            break;
        }
    }

    if ret.is_null() {
        *_errno() = ENOMEM as core::ffi::c_int;
    }
    ret
}

#[no_mangle]
pub unsafe extern "C" fn realloc(ptr: *mut core::ffi::c_void, size: usize) -> *mut core::ffi::c_void {
    if ptr.is_null() {
        return malloc(size);
    }
    if size != 0 {
        return msvcrt_heap_realloc(0, ptr, size);
    }
    free(ptr);
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn __p__amblksiz() -> *mut u32 {
    core::ptr::addr_of_mut!(MSVCRT_amblksiz)
}

#[no_mangle]
pub unsafe extern "C" fn _get_sbh_threshold() -> usize {
    MSVCRT_sbh_threshold
}

// The assembler split this function: its signature was emitted at the top of the file, in
// the file-scope preamble and with no body, where its `{` swallowed every following item
// into itself; the body was left here, headerless. Rejoined by hand.
//
// `#ifdef _WIN64` in the C — so the small-block heap exists on i386 only, and
// `target_pointer_width` is the honest spelling of that condition for all three arches.
#[no_mangle]
pub unsafe extern "C" fn _set_sbh_threshold(threshold: usize) -> i32 {
    #[cfg(target_pointer_width = "64")]
    {
        0
    }
    #[cfg(not(target_pointer_width = "64"))]
    {
        if threshold > 1016 {
            return 0;
        }

        if sb_heap.is_null() {
            sb_heap = HeapCreate(0, 0, 0);
            if sb_heap.is_null() {
                return 0;
            }
        }

        MSVCRT_sbh_threshold = (threshold + 0xf) & !0xf;
        1
    }
}

#[no_mangle]
pub unsafe extern "C" fn _aligned_free(memblock: *mut core::ffi::c_void) {
    if !memblock.is_null() {
        let saved = SAVED_PTR!(memblock) as *mut *mut core::ffi::c_void;
        free(*saved);
    }
}

#[no_mangle]
pub unsafe extern "C" fn _aligned_offset_malloc(
    size: usize,
    alignment: usize,
    offset: usize,
) -> *mut core::ffi::c_void {
    let memblock: *mut core::ffi::c_void;
    let temp: *mut core::ffi::c_void;
    let saved: *mut *mut core::ffi::c_void;

    if (alignment & (alignment.wrapping_sub(1))) != 0 {
        *_errno() = (EINVAL) as i32;
        return core::ptr::null_mut();
    }

    if offset != 0 && offset >= size {
        *_errno() = (EINVAL) as i32;
        return core::ptr::null_mut();
    }

    let mut alignment = alignment;
    if alignment < core::mem::size_of::<*mut core::ffi::c_void>() {
        alignment = core::mem::size_of::<*mut core::ffi::c_void>();
    }

    temp = malloc(size + alignment + core::mem::size_of::<*mut core::ffi::c_void>());

    if temp.is_null() {
        return core::ptr::null_mut();
    }

    memblock = ALIGN_PTR!(temp, alignment, offset);

    saved = SAVED_PTR!(memblock) as *mut *mut core::ffi::c_void;
    *saved = temp;

    memblock
}

#[no_mangle]
pub unsafe extern "C" fn _aligned_malloc(
    size: usize,
    alignment: usize,
) -> *mut core::ffi::c_void {
    _aligned_offset_malloc(size, alignment, 0)
}

#[no_mangle]
pub unsafe extern "C" fn _aligned_offset_realloc(
    memblock: *mut core::ffi::c_void,
    size: usize,
    alignment: usize,
    offset: usize,
) -> *mut core::ffi::c_void {
    let temp: *mut core::ffi::c_void;
    let mut saved: *mut *mut core::ffi::c_void;
    let old_padding: usize;
    let new_padding: usize;
    let mut old_size: usize;

    if memblock.is_null() {
        return _aligned_offset_malloc(size, alignment, offset);
    }

    if (alignment & (alignment - 1)) != 0 {
        *_errno() = (EINVAL) as i32;
        return core::ptr::null_mut();
    }

    if offset >= size {
        *_errno() = (EINVAL) as i32;
        return core::ptr::null_mut();
    }

    if size == 0 {
        _aligned_free(memblock);
        return core::ptr::null_mut();
    }

    let alignment = if alignment < core::mem::size_of::<*mut core::ffi::c_void>() {
        core::mem::size_of::<*mut core::ffi::c_void>()
    } else {
        alignment
    };

    saved = SAVED_PTR!(memblock) as *mut *mut core::ffi::c_void;
    if memblock != ALIGN_PTR!(*saved, alignment, offset) {
        *_errno() = (EINVAL) as i32;
        return core::ptr::null_mut();
    }

    old_padding = (memblock as *mut u8).offset_from(*saved as *mut u8) as usize;

    old_size = _msize(*saved);
    if old_size == (-1 as isize) as usize {
        return core::ptr::null_mut();
    }

    if old_size < old_padding {
        return core::ptr::null_mut();
    }
    old_size -= old_padding;

    temp = realloc(*saved, size + alignment + core::mem::size_of::<*mut core::ffi::c_void>());

    if temp.is_null() {
        return core::ptr::null_mut();
    }

    let memblock = ALIGN_PTR!(temp, alignment, offset);

    saved = SAVED_PTR!(memblock) as *mut *mut core::ffi::c_void;

    new_padding = (memblock as *mut u8).offset_from(temp as *mut u8) as usize;

    if new_padding != old_padding {
        memmove(
            memblock,
            (temp as *mut u8).add(old_padding) as *mut core::ffi::c_void,
            if old_size < size { old_size } else { size },
        );
    }

    *saved = temp;

    memblock
}

#[no_mangle]
pub unsafe extern "C" fn _aligned_realloc(
    memblock: *mut core::ffi::c_void,
    size: usize,
    alignment: usize,
) -> *mut core::ffi::c_void {
    _aligned_offset_realloc(memblock, size, alignment, 0)
}

#[no_mangle]
pub unsafe extern "C" fn memmove_s(
    dest: *mut core::ffi::c_void,
    numberOfElements: usize,
    src: *const core::ffi::c_void,
    count: usize,
) -> core::ffi::c_int {
    if count == 0 {
        return 0;
    }

    if MSVCRT_CHECK_PMT!(!dest.is_null()) == 0 {
        return (EINVAL) as i32;
    }
    if MSVCRT_CHECK_PMT!(!src.is_null()) == 0 {
        return (EINVAL) as i32;
    }
    if MSVCRT_CHECK_PMT_ERR!(count <= numberOfElements, (ERANGE) as i32) == 0 {
        return (ERANGE) as i32;
    }

    memmove(dest, src, count);
    0
}

#[no_mangle]
pub unsafe extern "C" fn memcpy_s(
    dest: *mut core::ffi::c_void,
    numberOfElements: usize,
    src: *const core::ffi::c_void,
    count: usize,
) -> i32 {
    if count == 0 {
        return 0;
    }

    if MSVCRT_CHECK_PMT!(dest != core::ptr::null_mut()) == 0 {
        return (EINVAL) as i32;
    }
    if MSVCRT_CHECK_PMT!(src != core::ptr::null()) == 0 {
        memset(dest, 0, numberOfElements);
        return (EINVAL) as i32;
    }
    if MSVCRT_CHECK_PMT_ERR!(count <= numberOfElements, (ERANGE) as i32) == 0 {
        memset(dest, 0, numberOfElements);
        return (ERANGE) as i32;
    }

    memmove(dest, src, count);
    return 0;
}
#[no_mangle]
pub unsafe extern "C" fn msvcrt_init_heap() -> BOOL {
    // The C guards this with `#if _MSVCR_VER <= 100 && !defined(__REACTOS__)`. Here
    // `_MSVCR_VER` is 0 but `__REACTOS__` IS defined, so the condition is FALSE and the
    // `#else` arm is taken on ALL THREE ARCHES — there is no HeapCreate on this path.
    //
    // The paid translation invented `target_pointer_width` for that condition, which has
    // nothing to do with it, and then took the one-hop PEB shortcut in the 64-bit arm. The
    // result booted on i386 (a private heap is still a valid heap, so the wrong arm worked)
    // and wedged amd64 and arm64: `heap` became the PEB pointer, every RtlAllocateHeap
    // failed, malloc returned NULL forever and user-mode starved with threads parked on
    // WrFreePage. A `#if` whose condition the translation does not model is not a
    // formatting detail — it silently selects dead code.
    heap = (*(*NtCurrentTeb()).Peb).ProcessHeap;
    if !heap.is_null() { 1 } else { 0 }
}
#[no_mangle]
pub unsafe extern "C" fn msvcrt_destroy_heap() {
    // Same `#if` as msvcrt_init_heap, same verdict: the `HeapDestroy(heap)` arm is NOT
    // compiled here, because this build never creates that heap — it borrows the process
    // heap. Destroying it would tear down the heap the whole process allocates from. The
    // translation invented the same `target_pointer_width` condition here, which on i386
    // paired with its bogus HeapCreate to at least be self-consistent; the C is neither.
    // Only the small-block heap, which this file really does own, is destroyed.
    if !sb_heap.is_null() {
        HeapDestroy(sb_heap);
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * msvcrt.dll heap functions
//  *
//  * Copyright 2000 Jon Griffiths
//  *
//  * This library is free software; you can redistribute it and/or
//  * modify it under the terms of the GNU Lesser General Public
//  * License as published by the Free Software Foundation; either
//  * version 2.1 of the License, or (at your option) any later version.
//  *
//  * This library is distributed in the hope that it will be useful,
//  * but WITHOUT ANY WARRANTY; without even the implied warranty of
//  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
//  * Lesser General Public License for more details.
//  *
//  * You should have received a copy of the GNU Lesser General Public
//  * License along with this library; if not, write to the Free Software
//  * Foundation, Inc., 51 Franklin St, Fifth Floor, Boston, MA 02110-1301, USA
//  *
//  * Note: Win32 heap operations are MT safe. We only lock the new
//  *       handler and non atomic heap operations
//  */
//
// #include <malloc.h>
// #include "msvcrt.h"
// #include "mtdll.h"
// #include "winternl.h"
// #include "wine/debug.h"
//
// WINE_DEFAULT_DEBUG_CHANNEL(msvcrt);
//
// /* MT */
// #define LOCK_HEAP   _lock( _HEAP_LOCK )
// #define UNLOCK_HEAP _unlock( _HEAP_LOCK )
//
// /* _aligned */
// #define SAVED_PTR(x) ((void *)((DWORD_PTR)((char *)x - sizeof(void *)) & \
//                                ~(sizeof(void *) - 1)))
// #define ALIGN_PTR(ptr, alignment, offset) ((void *) \
//     ((((DWORD_PTR)((char *)ptr + alignment + sizeof(void *) + offset)) & \
//       ~(alignment - 1)) - offset))
//
// #define SB_HEAP_ALIGN 16
//
// static HANDLE heap, sb_heap;
//
// typedef int (CDECL *MSVCRT_new_handler_func)(size_t size);
//
// static MSVCRT_new_handler_func MSVCRT_new_handler;
// static LONG MSVCRT_new_mode;
//
// /* FIXME - According to documentation it should be 8*1024, at runtime it returns 16 */
// static unsigned int MSVCRT_amblksiz = 16;
// /* FIXME - According to documentation it should be 480 bytes, at runtime default is 0 */
// static size_t MSVCRT_sbh_threshold = 0;
//
// static void* msvcrt_heap_alloc(DWORD flags, size_t size)
// {
//     if(size < MSVCRT_sbh_threshold)
//     {
//         void *memblock, *temp, **saved;
//
//         temp = RtlAllocateHeap(sb_heap, flags, size+sizeof(void*)+SB_HEAP_ALIGN);
//         if(!temp) return NULL;
//
//         memblock = ALIGN_PTR(temp, SB_HEAP_ALIGN, 0);
//         saved = SAVED_PTR(memblock);
//         *saved = temp;
//         return memblock;
//     }
//
//     /* Lazy init if DllMain was never called (e.g. DLL mapped but not initialized) */
//     if(!heap)
//         heap = NtCurrentTeb()->Peb->ProcessHeap;
//
//     return RtlAllocateHeap(heap, flags, size);
// }
//
// static void* msvcrt_heap_realloc(DWORD flags, void *ptr, size_t size)
// {
//     if(!heap) heap = NtCurrentTeb()->Peb->ProcessHeap;
//     if(sb_heap && ptr && !RtlValidateHeap(heap, 0, ptr))
//     {
//         /* TODO: move data to normal heap if it exceeds sbh_threshold limit */
//         void *memblock, *temp, **saved;
//         size_t old_padding, new_padding, old_size;
//
//         saved = SAVED_PTR(ptr);
//         old_padding = (char*)ptr - (char*)*saved;
//         old_size = RtlSizeHeap(sb_heap, 0, *saved);
//         if(old_size == -1)
//             return NULL;
//         old_size -= old_padding;
//
//         temp = RtlReAllocateHeap(sb_heap, flags, *saved, size+sizeof(void*)+SB_HEAP_ALIGN);
//         if(!temp) return NULL;
//
//         memblock = ALIGN_PTR(temp, SB_HEAP_ALIGN, 0);
//         saved = SAVED_PTR(memblock);
//         new_padding = (char*)memblock - (char*)temp;
//
//         if(new_padding != old_padding)
//             memmove(memblock, (char*)temp+old_padding, old_size>size ? size : old_size);
//
//         *saved = temp;
//         return memblock;
//     }
//
//     return RtlReAllocateHeap(heap, flags, ptr, size);
// }
//
// static BOOL msvcrt_heap_free(void *ptr)
// {
//     if(!heap) heap = NtCurrentTeb()->Peb->ProcessHeap;
//     if(sb_heap && ptr && !RtlValidateHeap(heap, 0, ptr))
//     {
//         void **saved = SAVED_PTR(ptr);
//         return RtlFreeHeap(sb_heap, 0, *saved);
//     }
//
//     return RtlFreeHeap(heap, 0, ptr);
// }
//
// static size_t msvcrt_heap_size(void *ptr)
// {
//     if(!heap) heap = NtCurrentTeb()->Peb->ProcessHeap;
//     if(sb_heap && ptr && !RtlValidateHeap(heap, 0, ptr))
//     {
//         void **saved = SAVED_PTR(ptr);
//         return RtlSizeHeap(sb_heap, 0, *saved);
//     }
//
//     return RtlSizeHeap(heap, 0, ptr);
// }
//
// /*********************************************************************
//  *		_callnewh (MSVCRT.@)
//  */
// int CDECL _callnewh(size_t size)
// {
//   int ret = 0;
//   MSVCRT_new_handler_func handler = MSVCRT_new_handler;
//   if(handler)
//     ret = (*handler)(size) ? 1 : 0;
//   return ret;
// }
//
// /*********************************************************************
//  *		??2@YAPAXI@Z (MSVCRT.@)
//  */
// void*  DECLSPEC_HOTPATCH operator_new(size_t size)
// {
//   void *retval;
//
//   do
//   {
//     retval = msvcrt_heap_alloc(0, size);
//     if(retval)
//     {
//       TRACE("(%Iu) returning %p\n", size, retval);
//       return retval;
//     }
//   } while(_callnewh(size));
//
//   TRACE("(%Iu) out of memory\n", size);
//   return NULL;
// }
//
//
// /*********************************************************************
//  *		??2@YAPAXIHPBDH@Z (MSVCRT.@)
//  */
// void*  operator_new_dbg(size_t size, int type, const char *file, int line)
// {
//     return operator_new( size );
// }
//
//
// /*********************************************************************
//  *		??3@YAXPAX@Z (MSVCRT.@)
//  */
// void CDECL DECLSPEC_HOTPATCH operator_delete(void *mem)
// {
//   TRACE("(%p)\n", mem);
//   msvcrt_heap_free(mem);
// }
//
//
// /*********************************************************************
//  *		?_query_new_handler@@YAP6AHI@ZXZ (MSVCRT.@)
//  */
// MSVCRT_new_handler_func CDECL _query_new_handler(void)
// {
//   return MSVCRT_new_handler;
// }
//
//
// /*********************************************************************
//  *		?_query_new_mode@@YAHXZ (MSVCRT.@)
//  */
// int CDECL _query_new_mode(void)
// {
//   return MSVCRT_new_mode;
// }
//
// /*********************************************************************
//  *		?_set_new_handler@@YAP6AHI@ZP6AHI@Z@Z (MSVCRT.@)
//  */
// MSVCRT_new_handler_func CDECL _set_new_handler(MSVCRT_new_handler_func func)
// {
//   MSVCRT_new_handler_func old_handler;
//   LOCK_HEAP;
//   old_handler = MSVCRT_new_handler;
//   MSVCRT_new_handler = func;
//   UNLOCK_HEAP;
//   return old_handler;
// }
//
// /*********************************************************************
//  *		?set_new_handler@@YAP6AXXZP6AXXZ@Z (MSVCRT.@)
//  */
// MSVCRT_new_handler_func CDECL set_new_handler(void *func)
// {
//   TRACE("(%p)\n",func);
//   _set_new_handler(NULL);
//   return NULL;
// }
//
// /*********************************************************************
//  *		?_set_new_mode@@YAHH@Z (MSVCRT.@)
//  */
// int CDECL _set_new_mode(int mode)
// {
//   if(!MSVCRT_CHECK_PMT(mode == 0 || mode == 1)) return -1;
//   return InterlockedExchange(&MSVCRT_new_mode, mode);
// }
//
// /*********************************************************************
//  *		_expand (MSVCRT.@)
//  */
// void*  _expand(void* mem, size_t size)
// {
//   return msvcrt_heap_realloc(HEAP_REALLOC_IN_PLACE_ONLY, mem, size);
// }
//
// /*********************************************************************
//  *		_heapchk (MSVCRT.@)
//  */
// int CDECL _heapchk(void)
// {
//   if (!HeapValidate(heap, 0, NULL) ||
//           (sb_heap && !HeapValidate(sb_heap, 0, NULL)))
//   {
//     msvcrt_set_errno(GetLastError());
//     return _HEAPBADNODE;
//   }
//   return _HEAPOK;
// }
//
// /*********************************************************************
//  *		_heapmin (MSVCRT.@)
//  */
// int CDECL _heapmin(void)
// {
//   if (!HeapCompact( heap, 0 ) ||
//           (sb_heap && !HeapCompact( sb_heap, 0 )))
//   {
//     if (GetLastError() != ERROR_CALL_NOT_IMPLEMENTED)
//       msvcrt_set_errno(GetLastError());
//     return -1;
//   }
//   return 0;
// }
//
// /*********************************************************************
//  *		_heapwalk (MSVCRT.@)
//  */
// int CDECL _heapwalk(_HEAPINFO *next)
// {
//   PROCESS_HEAP_ENTRY phe;
//
//   if (sb_heap)
//       FIXME("small blocks heap not supported\n");
//
//   LOCK_HEAP;
//   phe.lpData = next->_pentry;
//   phe.cbData = next->_size;
//   phe.wFlags = next->_useflag == _USEDENTRY ? PROCESS_HEAP_ENTRY_BUSY : 0;
//
//   if (phe.lpData && phe.wFlags & PROCESS_HEAP_ENTRY_BUSY &&
//       !HeapValidate( heap, 0, phe.lpData ))
//   {
//     UNLOCK_HEAP;
//     msvcrt_set_errno(GetLastError());
//     return _HEAPBADNODE;
//   }
//
//   do
//   {
//     if (!HeapWalk( heap, &phe ))
//     {
//       UNLOCK_HEAP;
//       if (GetLastError() == ERROR_NO_MORE_ITEMS)
//          return _HEAPEND;
//       msvcrt_set_errno(GetLastError());
//       if (!phe.lpData)
//         return _HEAPBADBEGIN;
//       return _HEAPBADNODE;
//     }
//   } while (phe.wFlags & (PROCESS_HEAP_REGION|PROCESS_HEAP_UNCOMMITTED_RANGE));
//
//   UNLOCK_HEAP;
//   next->_pentry = phe.lpData;
//   next->_size = phe.cbData;
//   next->_useflag = phe.wFlags & PROCESS_HEAP_ENTRY_BUSY ? _USEDENTRY : _FREEENTRY;
//   return _HEAPOK;
// }
//
// /*********************************************************************
//  *		_heapset (MSVCRT.@)
//  */
// int CDECL _heapset(unsigned int value)
// {
//   int retval;
//   _HEAPINFO heap;
//
//   memset( &heap, 0, sizeof(heap) );
//   LOCK_HEAP;
//   while ((retval = _heapwalk(&heap)) == _HEAPOK)
//   {
//     if (heap._useflag == _FREEENTRY)
//       memset(heap._pentry, value, heap._size);
//   }
//   UNLOCK_HEAP;
//   return retval == _HEAPEND ? _HEAPOK : retval;
// }
//
// /*********************************************************************
//  *		_heapadd (MSVCRT.@)
//  */
// int CDECL _heapadd(void* mem, size_t size)
// {
//   TRACE("(%p,%Iu) unsupported in Win32\n", mem,size);
//   *_errno() = ENOSYS as core::ffi::c_int;
//   return -1;
// }
//
// /*********************************************************************
//  *		_get_heap_handle (MSVCRT.@)
//  */
// intptr_t CDECL _get_heap_handle(void)
// {
//     return (intptr_t)heap;
// }
//
// /*********************************************************************
//  *		_msize (MSVCRT.@)
//  */
// size_t CDECL _msize(void* mem)
// {
//   size_t size = msvcrt_heap_size(mem);
//   if (size == ~(size_t)0)
//   {
//     WARN(":Probably called with non wine-allocated memory, ret = -1\n");
//     /* At least the Win32 crtdll/msvcrt also return -1 in this case */
//   }
//   return size;
// }
//
//
// /*********************************************************************
//  *		calloc (MSVCRT.@)
//  */
// void*  DECLSPEC_HOTPATCH calloc(size_t count, size_t size)
// {
//   size_t bytes = count*size;
//
//   if (size && bytes / size != count)
//   {
//     *_errno() = ENOMEM as core::ffi::c_int;
//     return NULL;
//   }
//
//   return msvcrt_heap_alloc(HEAP_ZERO_MEMORY, bytes);
// }
//
//
// /*********************************************************************
//  *		free (MSVCRT.@)
//  */
// void CDECL DECLSPEC_HOTPATCH free(void* ptr)
// {
//   msvcrt_heap_free(ptr);
// }
//
//
// /*********************************************************************
//  *                  malloc (MSVCRT.@)
//  */
// void*  malloc(size_t size)
// {
//     void *ret;
//
//     do
//     {
//         ret = msvcrt_heap_alloc(0, size);
//         if (ret || !MSVCRT_new_mode)
//             break;
//     } while(_callnewh(size));
//
//     if (!ret)
//         *_errno() = ENOMEM as core::ffi::c_int;
//     return ret;
// }
//
//
// /*********************************************************************
//  *		realloc (MSVCRT.@)
//  */
// void*  DECLSPEC_HOTPATCH realloc(void* ptr, size_t size)
// {
//   if (!ptr) return malloc(size);
//   if (size) return msvcrt_heap_realloc(0, ptr, size);
//   free(ptr);
//   return NULL;
// }
//
//
//
// /*********************************************************************
//  *		__p__amblksiz (MSVCRT.@)
//  */
// unsigned int*  __p__amblksiz(void)
// {
//   return &MSVCRT_amblksiz;
// }
//
// /*********************************************************************
//  *		_get_sbh_threshold (MSVCRT.@)
//  */
// size_t CDECL _get_sbh_threshold(void)
// {
//   return MSVCRT_sbh_threshold;
// }
//
// /*********************************************************************
//  *		_set_sbh_threshold (MSVCRT.@)
//  */
// int CDECL _set_sbh_threshold(size_t threshold)
// {
// #ifdef _WIN64
//   return 0;
// #else
//   if(threshold > 1016)
//      return 0;
//
//   if(!sb_heap)
//   {
//       sb_heap = HeapCreate(0, 0, 0);
//       if(!sb_heap)
//           return 0;
//   }
//
//   MSVCRT_sbh_threshold = (threshold+0xf) & ~0xf;
//   return 1;
// #endif
// }
//
// /*********************************************************************
//  *		_aligned_free (MSVCRT.@)
//  */
// void CDECL _aligned_free(void *memblock)
// {
//     TRACE("(%p)\n", memblock);
//
//     if (memblock)
//     {
//         void **saved = SAVED_PTR(memblock);
//         free(*saved);
//     }
// }
//
// /*********************************************************************
//  *		_aligned_offset_malloc (MSVCRT.@)
//  */
// void *  _aligned_offset_malloc(size_t size, size_t alignment, size_t offset)
// {
//     void *memblock, *temp, **saved;
//     TRACE("(%Iu, %Iu, %Iu)\n", size, alignment, offset);
//
//     /* alignment must be a power of 2 */
//     if ((alignment & (alignment - 1)) != 0)
//     {
//         *_errno() = EINVAL as core::ffi::c_int;
//         return NULL;
//     }
//
//     /* offset must be less than size */
//     if (offset && offset >= size)
//     {
//         *_errno() = EINVAL as core::ffi::c_int;
//         return NULL;
//     }
//
//     /* don't align to less than void pointer size */
//     if (alignment < sizeof(void *))
//         alignment = sizeof(void *);
//
//     /* allocate enough space for void pointer and alignment */
//     temp = malloc(size + alignment + sizeof(void *));
//
//     if (!temp)
//         return NULL;
//
//     /* adjust pointer for proper alignment and offset */
//     memblock = ALIGN_PTR(temp, alignment, offset);
//
//     /* Save the real allocation address below returned address */
//     /* so it can be found later to free. */
//     saved = SAVED_PTR(memblock);
//     *saved = temp;
//
//     return memblock;
// }
//
// /*********************************************************************
//  *		_aligned_malloc (MSVCRT.@)
//  */
// void *  _aligned_malloc(size_t size, size_t alignment)
// {
//     TRACE("(%Iu, %Iu)\n", size, alignment);
//     return _aligned_offset_malloc(size, alignment, 0);
// }
//
// /*********************************************************************
//  *		_aligned_offset_realloc (MSVCRT.@)
//  */
// void *  _aligned_offset_realloc(void *memblock, size_t size,
//                                      size_t alignment, size_t offset)
// {
//     void * temp, **saved;
//     size_t old_padding, new_padding, old_size;
//     TRACE("(%p, %Iu, %Iu, %Iu)\n", memblock, size, alignment, offset);
//
//     if (!memblock)
//         return _aligned_offset_malloc(size, alignment, offset);
//
//     /* alignment must be a power of 2 */
//     if ((alignment & (alignment - 1)) != 0)
//     {
//         *_errno() = EINVAL as core::ffi::c_int;
//         return NULL;
//     }
//
//     /* offset must be less than size */
//     if (offset >= size)
//     {
//         *_errno() = EINVAL as core::ffi::c_int;
//         return NULL;
//     }
//
//     if (size == 0)
//     {
//         _aligned_free(memblock);
//         return NULL;
//     }
//
//     /* don't align to less than void pointer size */
//     if (alignment < sizeof(void *))
//         alignment = sizeof(void *);
//
//     /* make sure alignment and offset didn't change */
//     saved = SAVED_PTR(memblock);
//     if (memblock != ALIGN_PTR(*saved, alignment, offset))
//     {
//         *_errno() = EINVAL as core::ffi::c_int;
//         return NULL;
//     }
//
//     old_padding = (char *)memblock - (char *)*saved;
//
//     /* Get previous size of block */
//     old_size = _msize(*saved);
//     if (old_size == -1)
//     {
//         /* It seems this function was called with an invalid pointer. Bail out. */
//         return NULL;
//     }
//
//     /* Adjust old_size to get amount of actual data in old block. */
//     if (old_size < old_padding)
//     {
//         /* Shouldn't happen. Something's weird, so bail out. */
//         return NULL;
//     }
//     old_size -= old_padding;
//
//     temp = realloc(*saved, size + alignment + sizeof(void *));
//
//     if (!temp)
//         return NULL;
//
//     /* adjust pointer for proper alignment and offset */
//     memblock = ALIGN_PTR(temp, alignment, offset);
//
//     /* Save the real allocation address below returned address */
//     /* so it can be found later to free. */
//     saved = SAVED_PTR(memblock);
//
//     new_padding = (char *)memblock - (char *)temp;
//
// /*
//    Memory layout of old block is as follows:
//    +-------+---------------------+-+--------------------------+-----------+
//    |  ...  | "old_padding" bytes | | ... "old_size" bytes ... |    ...    |
//    +-------+---------------------+-+--------------------------+-----------+
//            ^                     ^ ^
//            |                     | |
//         *saved               saved memblock
//
//    Memory layout of new block is as follows:
//    +-------+-----------------------------+-+----------------------+-------+
//    |  ...  |    "new_padding" bytes      | | ... "size" bytes ... |  ...  |
//    +-------+-----------------------------+-+----------------------+-------+
//            ^                             ^ ^
//            |                             | |
//           temp                       saved memblock
//
//    However, in the new block, actual data is still written as follows
//    (because it was copied by realloc):
//    +-------+---------------------+--------------------------------+-------+
//    |  ...  | "old_padding" bytes |   ... "old_size" bytes ...     |  ...  |
//    +-------+---------------------+--------------------------------+-------+
//            ^                             ^ ^
//            |                             | |
//           temp                       saved memblock
//
//    Therefore, min(old_size,size) bytes of actual data have to be moved
//    from the offset they were at in the old block (temp + old_padding),
//    to the offset they have to be in the new block (temp + new_padding == memblock).
// */
//     if (new_padding != old_padding)
//         memmove((char *)memblock, (char *)temp + old_padding, (old_size < size) ? old_size : size);
//
//     *saved = temp;
//
//     return memblock;
// }
//
// /*********************************************************************
//  *		_aligned_realloc (MSVCRT.@)
//  */
// void *  _aligned_realloc(void *memblock, size_t size, size_t alignment)
// {
//     TRACE("(%p, %Iu, %Iu)\n", memblock, size, alignment);
//     return _aligned_offset_realloc(memblock, size, alignment, 0);
// }
//
// /*********************************************************************
//  *		memmove_s (MSVCRT.@)
//  */
// int CDECL memmove_s(void *dest, size_t numberOfElements, const void *src, size_t count)
// {
//     TRACE("(%p %Iu %p %Iu)\n", dest, numberOfElements, src, count);
//
//     if(!count)
//         return 0;
//
//     if (!MSVCRT_CHECK_PMT(dest != NULL)) return EINVAL;
//     if (!MSVCRT_CHECK_PMT(src != NULL)) return EINVAL;
//     if (!MSVCRT_CHECK_PMT_ERR( count <= numberOfElements, ERANGE )) return ERANGE;
//
//     memmove(dest, src, count);
//     return 0;
// }
//
//
// /*********************************************************************
//  *		memcpy_s (MSVCRT.@)
//  */
// int CDECL memcpy_s(void *dest, size_t numberOfElements, const void *src, size_t count)
// {
//     TRACE("(%p %Iu %p %Iu)\n", dest, numberOfElements, src, count);
//
//     if(!count)
//         return 0;
//
//     if (!MSVCRT_CHECK_PMT(dest != NULL)) return EINVAL;
//     if (!MSVCRT_CHECK_PMT(src != NULL))
//     {
//         memset(dest, 0, numberOfElements);
//         return EINVAL;
//     }
//     if (!MSVCRT_CHECK_PMT_ERR( count <= numberOfElements, ERANGE ))
//     {
//         memset(dest, 0, numberOfElements);
//         return ERANGE;
//     }
//
//     memmove(dest, src, count);
//     return 0;
// }
//
//
// BOOL msvcrt_init_heap(void)
// {
// #if _MSVCR_VER <= 100 && !defined(__REACTOS__)
//     heap = HeapCreate(0, 0, 0);
// #else
//     heap = NtCurrentTeb()->Peb->ProcessHeap;
// #endif
//     return heap != NULL;
// }
//
// void msvcrt_destroy_heap(void)
// {
// #if _MSVCR_VER <= 100 && !defined(__REACTOS__)
//     HeapDestroy(heap);
// #endif
//     if(sb_heap)
//         HeapDestroy(sb_heap);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
