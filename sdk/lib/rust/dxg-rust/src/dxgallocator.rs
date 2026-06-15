//! win32ss/reactx/dxg/dxgallocator.c — DirectDraw video-memory allocator.

use win32k_sys::win32k::*;
use ntkrnl_sys::driver_ddk::DbgPrint;
use crate::types::*;

/// `ALIGN_UP(value, alignment)` from the DDK — round `value` up to a multiple
/// of `alignment` (a power of two). Reproduced locally (no macro export).
#[inline]
fn ALIGN_UP(value: DWORD, alignment: DWORD) -> DWORD {
    (value + (alignment - 1)) & !(alignment - 1)
}

#[no_mangle]
pub unsafe extern "system" fn DdrawMemAlloc(
    pvmh: LPVMEMHEAP,
    Width: DWORD,
    Height: DWORD,
    AllocSize: LPDWORD,
    _Alignment: LPSURFACEALIGNMENT,
    ResolvedPitch: LPDWORD,
) -> FLATPTR {
    let MemPtr: FLATPTR;
    let RequiredBytes: DWORD;
    let mut CurrentPos: DWORD;
    let mut Width = Width;

    if pvmh.is_null() {
        if !AllocSize.is_null() {
            *AllocSize = 0;
        }
        return 0;
    }

    let MemBase: FLATPTR = (*pvmh).freeList as FLATPTR;
    let MemLimit: FLATPTR = (*pvmh).allocList as FLATPTR;

    if MemBase == 0 || MemLimit == 0 || MemLimit <= MemBase {
        DbgPrint(b"No valid video memory range\n\0".as_ptr() as *const i8);
        if !AllocSize.is_null() {
            *AllocSize = 0;
        }
        return 0;
    }

    CurrentPos = (*pvmh).dwCommitedSize;
    CurrentPos = ALIGN_UP(CurrentPos, core::mem::size_of::<ULONG>() as DWORD);

    if (*pvmh).dwFlags & VMEMHEAP_LINEAR != 0 {
        RequiredBytes = Width;
    } else {
        Width = if (*pvmh).stride != 0 { (*pvmh).stride } else { Width };
        RequiredBytes = Width * Height;
    }

    MemPtr = MemBase + CurrentPos as FLATPTR;

    if MemPtr + RequiredBytes as FLATPTR > MemLimit {
        DbgPrint(b"Out of memory\n\0".as_ptr() as *const i8);
        if !AllocSize.is_null() {
            *AllocSize = 0;
        }
        return 0;
    }

    (*pvmh).dwCommitedSize = CurrentPos + RequiredBytes;

    if !ResolvedPitch.is_null() {
        *ResolvedPitch = Width;
    }
    if !AllocSize.is_null() {
        *AllocSize = RequiredBytes;
    }

    MemPtr
}

#[no_mangle]
pub unsafe extern "system" fn DxDdHeapDdrawMemAlloc(
    DdrawVidMem: LPVIDMEM,
    Width: DWORD,
    Height: DWORD,
    Alignment: LPSURFACEALIGNMENT,
    ResolvedPitch: LPDWORD,
    AllocSize: PDWORD,
) -> FLATPTR {
    let Result: FLATPTR;
    let mut ActualSize: DWORD = 0;

    if DdrawVidMem.is_null() || (*DdrawVidMem).__bindgen_anon_2.lpHeap.is_null() {
        if !AllocSize.is_null() {
            *AllocSize = 0;
        }
        return 0;
    }

    if ((*DdrawVidMem).dwFlags & VIDMEM_ISNONLOCAL) != 0
        && (*(*DdrawVidMem).__bindgen_anon_2.lpHeap).pvPhysRsrv.is_null()
    {
        if !AllocSize.is_null() {
            *AllocSize = 0;
        }
        return 0;
    }

    let MemBegin: FLATPTR = (*DdrawVidMem).fpStart;
    let mut MemEnd: FLATPTR;

    if ((*DdrawVidMem).dwFlags & VIDMEM_ISLINEAR) != 0 {
        MemEnd = (*DdrawVidMem).__bindgen_anon_1.fpEnd;
    } else if ((*DdrawVidMem).dwFlags & VIDMEM_ISRECTANGULAR) != 0 {
        MemEnd = MemBegin
            .wrapping_add(((*DdrawVidMem).__bindgen_anon_1.dwWidth * (*DdrawVidMem).__bindgen_anon_2.dwHeight) as FLATPTR);
    } else {
        MemEnd = (*DdrawVidMem).__bindgen_anon_1.fpEnd;
    }

    if MemBegin == 0 {
        if !AllocSize.is_null() {
            *AllocSize = 0;
        }
        return 0;
    }

    if MemEnd == 0 || MemEnd <= MemBegin {
        if !(*DdrawVidMem).__bindgen_anon_2.lpHeap.is_null()
            && (*(*DdrawVidMem).__bindgen_anon_2.lpHeap).dwTotalSize > 0
        {
            MemEnd = MemBegin.wrapping_add((*(*DdrawVidMem).__bindgen_anon_2.lpHeap).dwTotalSize as FLATPTR);
        } else {
            if !AllocSize.is_null() {
                *AllocSize = 0;
            }
            return 0;
        }
    }

    let lpHeap = (*DdrawVidMem).__bindgen_anon_2.lpHeap;
    let SavedBegin: FLATPTR = (*lpHeap).freeList as FLATPTR;
    let SavedEnd: FLATPTR = (*lpHeap).allocList as FLATPTR;
    (*lpHeap).freeList = MemBegin as LPVOID;
    (*lpHeap).allocList = MemEnd as LPVOID;

    Result = DdrawMemAlloc(lpHeap, Width, Height, &mut ActualSize, Alignment, ResolvedPitch);

    if Result == 0 {
        if !AllocSize.is_null() {
            *AllocSize = 0;
        }
        return Result;
    }

    (*lpHeap).freeList = SavedBegin as LPVOID;
    (*lpHeap).allocList = SavedEnd as LPVOID;

    if !AllocSize.is_null() {
        *AllocSize = ActualSize;
    }

    Result
}

#[no_mangle]
pub unsafe extern "system" fn DxDdHeapVidMemAllocAligned(
    DdrawVidMem: LPVIDMEM,
    Width: DWORD,
    Height: DWORD,
    Alignment: LPSURFACEALIGNMENT,
    ResolvedPitch: LPDWORD,
) -> FLATPTR {
    let mut SizeOut: DWORD = 0;

    if DdrawVidMem.is_null()
        || (*DdrawVidMem).__bindgen_anon_2.lpHeap.is_null()
        || ((*DdrawVidMem).dwFlags & VIDMEM_HEAPDISABLED) != 0
    {
        return 0;
    }

    if ((*DdrawVidMem).dwFlags & VIDMEM_ISNONLOCAL) != 0 {
        if (*(*DdrawVidMem).__bindgen_anon_2.lpHeap).pvPhysRsrv.is_null() {
            return 0;
        }
        DbgPrint(b"AGP memory not supported\n\0".as_ptr() as *const i8);
        return 0;
    }

    DxDdHeapDdrawMemAlloc(DdrawVidMem, Width, Height, Alignment, ResolvedPitch, &mut SizeOut)
}
