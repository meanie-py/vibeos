//! `heapchk.cpp`, `heapmin.cpp`, `heapwalk.cpp`: `HeapValidate`,
//! `HeapCompact` and `HeapWalk` mapped onto the `_HEAP*` status codes.

use super::{acrt_heap, invalid_einval, GetLastError, HeapCompact, HeapValidate, HeapWalk};
use core::ffi::{c_int, c_void};
use core::mem::offset_of;
use core::ptr::null;

/// `malloc.h` status codes.
const HEAPOK: c_int = -2;
const HEAPBADBEGIN: c_int = -3;
const HEAPBADNODE: c_int = -4;
const HEAPEND: c_int = -5;
const HEAPBADPTR: c_int = -6;
/// `malloc.h` `_USEDENTRY`.
const USEDENTRY: c_int = 1;
/// winbase.h `PROCESS_HEAP_ENTRY_BUSY`.
const PROCESS_HEAP_ENTRY_BUSY: u16 = 0x0004;
/// winerror.h `ERROR_NO_MORE_ITEMS`.
const ERROR_NO_MORE_ITEMS: u32 = 259;

/// `malloc.h` `_HEAPINFO`.
#[repr(C)]
pub struct HeapInfo {
    pub pentry: *mut c_int,
    pub size: usize,
    pub useflag: c_int,
}

/// winbase.h `PROCESS_HEAP_ENTRY`. The trailing union (`Block { hMem,
/// dwReserved[3] }` / `Region { dwCommittedSize, dwUnCommittedSize,
/// lpFirstBlock, lpLastBlock }`) is never read here, only zeroed; it is two
/// DWORDs + two pointers, which covers both arms on every target.
#[repr(C)]
pub struct ProcessHeapEntry {
    pub lp_data: *mut c_void,
    pub cb_data: u32,
    pub cb_overhead: u8,
    pub i_region_index: u8,
    pub w_flags: u16,
    pub region: [u32; 2],
    pub region_blocks: [*mut c_void; 2],
}

// Measured from Microsoft's compiled heapwalk.cpp.obj: the entry is zeroed as
// 7 dwords on i386 (`wFlags` at +0xa) and 40 bytes on amd64/arm64 (`wFlags` at
// +0xe); `_HEAPINFO` is read at 0/4/8 and 0/8/0x10.
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<ProcessHeapEntry>() == 28);
    assert!(offset_of!(ProcessHeapEntry, cb_data) == 4);
    assert!(offset_of!(ProcessHeapEntry, w_flags) == 0xa);
    assert!(core::mem::size_of::<HeapInfo>() == 12);
    assert!(offset_of!(HeapInfo, size) == 4);
    assert!(offset_of!(HeapInfo, useflag) == 8);
};
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<ProcessHeapEntry>() == 40);
    assert!(offset_of!(ProcessHeapEntry, cb_data) == 8);
    assert!(offset_of!(ProcessHeapEntry, w_flags) == 0xe);
    assert!(core::mem::size_of::<HeapInfo>() == 24);
    assert!(offset_of!(HeapInfo, size) == 8);
    assert!(offset_of!(HeapInfo, useflag) == 0x10);
};

/// `int _heapchk()`: `HeapValidate` over the whole heap.
#[no_mangle]
pub unsafe extern "C" fn _heapchk() -> c_int {
    if HeapValidate(acrt_heap(), 0, null()) == 0 {
        return HEAPBADNODE;
    }
    HEAPOK
}

/// `int _heapmin()`: 0, or -1 when `HeapCompact` returns 0.
#[no_mangle]
pub unsafe extern "C" fn _heapmin() -> c_int {
    if HeapCompact(acrt_heap(), 0) == 0 {
        return -1;
    }
    0
}

/// `try_walk`. The C wraps `HeapWalk` in `__try`/`__except(ACCESS_VIOLATION)
/// -> _HEAPBADNODE`, but ucrt is built with `_USE_DUMMY_PSEH=1`, where the
/// except arm is dead code (the compiled object calls `HeapWalk` bare, with no
/// handler registered): a fault propagates. Same here.
unsafe fn try_walk(win32_entry: *mut ProcessHeapEntry) -> c_int {
    if HeapWalk(acrt_heap(), win32_entry) != 0 {
        return HEAPOK;
    }
    if GetLastError() == ERROR_NO_MORE_ITEMS {
        return HEAPEND;
    }
    HEAPBADNODE
}

/// `int _heapwalk(_HEAPINFO*)`.
#[no_mangle]
pub unsafe extern "C" fn _heapwalk(entry: *mut HeapInfo) -> c_int {
    if entry.is_null() {
        invalid_einval();
        return HEAPBADPTR;
    }

    let mut win32_entry = ProcessHeapEntry {
        lp_data: (*entry).pentry.cast::<c_void>(),
        cb_data: 0,
        cb_overhead: 0,
        i_region_index: 0,
        w_flags: 0,
        region: [0; 2],
        region_blocks: [core::ptr::null_mut(); 2],
    };

    if win32_entry.lp_data.is_null() {
        // Starting the walk.
        if HeapWalk(acrt_heap(), &mut win32_entry) == 0 {
            return HEAPBADBEGIN;
        }
    } else {
        if (*entry).useflag == USEDENTRY {
            if HeapValidate(acrt_heap(), 0, (*entry).pentry.cast::<c_void>()) == 0 {
                return HEAPBADNODE;
            }
            win32_entry.w_flags = PROCESS_HEAP_ENTRY_BUSY;
        }

        let status = try_walk(&mut win32_entry);
        if status != HEAPOK {
            return status;
        }
    }

    loop {
        if win32_entry.w_flags & PROCESS_HEAP_ENTRY_BUSY != 0 {
            (*entry).pentry = win32_entry.lp_data.cast::<c_int>();
            (*entry).size = win32_entry.cb_data as usize;
            (*entry).useflag = USEDENTRY;
            return HEAPOK;
        }

        let status = try_walk(&mut win32_entry);
        if status != HEAPOK {
            return status;
        }
    }
}
