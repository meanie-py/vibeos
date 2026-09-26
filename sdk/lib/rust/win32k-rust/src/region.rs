use win32k_sys::win32k::*;
use ntkrnl_sys::ddk::PsGetCurrentProcessWin32Process;
use ntkrnl_sys::driver_ddk::DbgPrint;
use ntkrnl_sys::ddk::{ExAllocatePoolWithTag, ExFreePoolWithTag};
use ntkrnl_sys::base::STATUS_SUCCESS;
const STATUS_ACCESS_VIOLATION: i32 = 0xC0000005_u32 as i32;
use core::ffi::c_void;

// Sibling Rust modules (same crate) — `use`, do NOT re-extern: re-declaring an
// in-crate #[no_mangle] fn with a different ABI string causes an IR-level
// available_externally collision (feedback_dup_abi_name_collision).
// (GDIOBJ_*/Gre*/GdiPool*/EngSetLastError/SetLastNtError resolve via the
// win32k_sys glob with bindgen's correct per-arch ABI — proven by waves 1-3.)
use crate::rect::{RECTL_bUnionRect, RECTL_vMakeWellOrdered};
use crate::xformobj::{EXFORMOBJ_iSetXform, EXFORMOBJ_bApplyXform};
use crate::error::SetLastNtError;

// GDITAG_REGION = 'ngrG' (C multichar literal, big-endian) = 0x6E677247.
const TAG_REGION: u32 = 0x6E677247;
// bindgen types these as u32; region uses them as INT (i32). Shadow locally.
const INT_MAX: INT = i32::MAX;
const INT_MIN: INT = i32::MIN;
const MAXLONG: INT = i32::MAX;
const MINLONG: INT = i32::MIN;

extern "C" {
    fn abs(x: INT) -> INT;
}
// SafeProbeForRead/Write are NTAPI (probe.c) → stdcall on i386 = extern "system".
extern "system" {
    fn SafeProbeForRead(Address: *const c_void, Length: usize, Alignment: u32) -> NTSTATUS;
    fn SafeProbeForWrite(Address: *mut c_void, Length: usize, Alignment: u32) -> NTSTATUS;
}

// ---- inline helpers (C macros / FORCEINLINEs that have no exported symbol) ----
#[inline(always)] fn NT_SUCCESS(s: NTSTATUS) -> bool { s >= 0 }
#[inline(always)] unsafe fn RtlCopyMemory(dst: *mut c_void, src: *const c_void, n: usize) {
    core::ptr::copy_nonoverlapping(src as *const u8, dst as *mut u8, n);
}
#[inline(always)] fn max(a: INT, b: INT) -> INT { if a > b { a } else { b } }
#[inline(always)] fn min(a: INT, b: INT) -> INT { if a < b { a } else { b } }
// EXTENTCHECK(r1,r2): do two rectangles overlap? (region.c macro)
#[inline(always)] unsafe fn EXTENTCHECK(r1: *const RECTL, r2: *const RECTL) -> i32 {
    (((*r1).right > (*r2).left) && ((*r1).left < (*r2).right)
        && ((*r1).bottom > (*r2).top) && ((*r1).top < (*r2).bottom)) as i32
}
// GDI_HANDLE_GET_TYPE(h as HGDIOBJ) = ((ULONG_PTR)h) & GDI_HANDLE_TYPE_MASK (ntgdihdl.h)
#[inline(always)] fn GDI_HANDLE_GET_TYPE(h: HGDIOBJ) -> u32 { ((h as usize) & 0x007f0000) as u32 }
// RECTL_vSetEmptyRect is FORCEINLINE (rect.h): zero all four edges.
#[inline(always)] unsafe fn RECTL_vSetEmptyRect(prcl: *mut RECTL) {
    (*prcl).left = 0; (*prcl).top = 0; (*prcl).right = 0; (*prcl).bottom = 0;
}
// XFORMOBJ_vInit is FORCEINLINE (xformobj.h): pxo->pmx = pmx.
#[inline(always)] unsafe fn XFORMOBJ_vInit(pxo: *mut XFORMOBJ, pmx: *mut MATRIX) { (*(pxo as *mut _EXFORMOBJ)).pmx = pmx; }

// NT_ASSERT is debug-only and stays silent. DbgPrint is NOT: region.c calls the
// real cdecl-variadic ntoskrnl export directly (IntDumpRegion), so shadowing it
// with a silent macro discarded the dump it exists to produce.
macro_rules! NT_ASSERT { ($($t:tt)*) => { () }; }

// ---- file-scope preamble (translated) ----
#[no_mangle]
pub static mut prgnDefault: PREGION = core::ptr::null_mut();
pub static mut hrgnDefault: HRGN = core::ptr::null_mut();

#[repr(C)]
pub struct BRESINFO {
    pub minor_axis: INT,
    pub d: INT,
    pub m: INT,
    pub m1: INT,
    pub incr1: INT,
    pub incr2: INT,
}

pub const CLOCKWISE: INT = 1;
pub const COUNTERCLOCKWISE: INT = -1;

#[repr(C)]
pub struct _EDGE_TABLE_ENTRY {
    pub ymax: INT,
    pub bres: BRESINFO,
    pub next: *mut _EDGE_TABLE_ENTRY,
    pub back: *mut _EDGE_TABLE_ENTRY,
    pub nextWETE: *mut _EDGE_TABLE_ENTRY,
    pub ClockWise: INT,
}
pub type EDGE_TABLE_ENTRY = _EDGE_TABLE_ENTRY;

#[repr(C)]
pub struct _SCANLINE_LIST {
    pub scanline: INT,
    pub edgelist: *mut EDGE_TABLE_ENTRY,
    pub next: *mut _SCANLINE_LIST,
}
pub type SCANLINE_LIST = _SCANLINE_LIST;

#[repr(C)]
pub struct EDGE_TABLE {
    pub ymax: INT,
    pub ymin: INT,
    pub scanlines: SCANLINE_LIST,
}

pub const SLLSPERBLOCK: usize = 25;

#[repr(C)]
pub struct _SCANLINE_LISTBLOCK {
    pub SLLs: [SCANLINE_LIST; SLLSPERBLOCK],
    pub next: *mut _SCANLINE_LISTBLOCK,
}
pub type SCANLINE_LISTBLOCK = _SCANLINE_LISTBLOCK;

pub const LARGE_COORDINATE: INT = INT_MAX;
pub const SMALL_COORDINATE: INT = INT_MIN;

pub type overlapProcp = Option<unsafe fn(PREGION, PRECT, PRECT, PRECT, PRECT, INT, INT) -> BOOL>;
pub type nonOverlapProcp = Option<unsafe fn(PREGION, PRECT, PRECT, INT, INT) -> BOOL>;

pub const NUMPTSTOBUFFER: usize = 200;
pub const RGN_DEFAULT_RECTS: usize = 2;

#[repr(C)]
pub struct _POINTBLOCK {
    pub pts: [POINT; NUMPTSTOBUFFER],
    pub next: *mut _POINTBLOCK,
}
pub type POINTBLOCK = _POINTBLOCK;

#[inline(always)]


unsafe fn REGION_bGrowBufferSize(
    prgn: PREGION,
    cRects: u32,
) -> u8 {
    let mut cjNewSize: u32;
    let pvBuffer: *mut core::ffi::c_void;

    if cRects > MAXULONG / core::mem::size_of::<RECTL>() as u32 {
        return 0u8;
    }

    cjNewSize = cRects * core::mem::size_of::<RECTL>() as u32;

    if 2 * (*prgn).rdh.nRgnSize > cjNewSize {
        cjNewSize = 2 * (*prgn).rdh.nRgnSize;
    }

    pvBuffer = ExAllocatePoolWithTag(PagedPool, cjNewSize as SIZE_T, 0x6E676552u32);
    if pvBuffer.is_null() {
        return 0u8;
    }

    {
        let xDest = pvBuffer as *mut RECTL;
        let xSrc = (*prgn).Buffer;
        let mut xRects = (*prgn).rdh.nCount;
        let mut dest_ptr = xDest;
        let mut src_ptr = xSrc;
        while xRects > 0 {
            *dest_ptr = *src_ptr;
            dest_ptr = dest_ptr.offset(1);
            src_ptr = src_ptr.offset(1);
            xRects -= 1;
        }
    }

    if (*prgn).Buffer != core::ptr::addr_of_mut!((*prgn).rdh.rcBound) as *mut RECTL {
        ExFreePoolWithTag((*prgn).Buffer as *mut core::ffi::c_void, 0x6E676552u32);
    }

    (*prgn).Buffer = pvBuffer as *mut RECTL;
    (*prgn).rdh.nRgnSize = cjNewSize;

    return 1u8;
}

unsafe fn REGION_bEnsureBufferSize(
    prgn: PREGION,
    cRects: u32,
) -> u8 {
    if cRects > (*prgn).rdh.nRgnSize / core::mem::size_of::<RECTL>() as u32 {
        return REGION_bGrowBufferSize(prgn, cRects);
    }

    1u8
}

unsafe fn REGION_vAddRect(
    prgn: PREGION,
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
) {
    NT_ASSERT!((((*prgn).rdh.nCount + 1) * core::mem::size_of::<RECT>() as u32) <= (*prgn).rdh.nRgnSize);

    let prcl = &mut *(*prgn).Buffer.add((*prgn).rdh.nCount as usize);
    (*prcl).left = left;
    (*prcl).top = top;
    (*prcl).right = right;
    (*prcl).bottom = bottom;
    (*prgn).rdh.nCount += 1;
}

unsafe fn REGION_bAddRect(
    prgn: PREGION,
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
) -> u8 {
    if REGION_bEnsureBufferSize(prgn, (*prgn).rdh.nCount + 1) == 0 {
        return 0u8;
    }

    REGION_vAddRect(prgn, left, top, right, bottom);
    1u8
}

#[cfg(target_arch = "x86")]
unsafe fn IntDumpRegion(hRgn: HRGN) {
    let Data: PREGION;

    Data = REGION_LockRgn(hRgn);
    if Data.is_null() {
        DbgPrint(b"IntDumpRegion called with invalid region!\n\0".as_ptr() as PCHAR);
        return;
    }

    DbgPrint(
        b"IntDumpRegion(%x): %d,%d-%d,%d %d\n\0".as_ptr() as PCHAR,
        hRgn,
        (*Data).rdh.rcBound.left,
        (*Data).rdh.rcBound.top,
        (*Data).rdh.rcBound.right,
        (*Data).rdh.rcBound.bottom,
        (*Data).rdh.iType,
    );

    REGION_UnlockRgn(Data);
}

#[cfg(not(target_arch = "x86"))]
unsafe fn IntDumpRegion(hRgn: HRGN) {
    let Data: PREGION;

    Data = REGION_LockRgn(hRgn);
    if Data.is_null() {
        DbgPrint(b"IntDumpRegion called with invalid region!\n\0".as_ptr() as PCHAR);
        return;
    }

    DbgPrint(
        b"IntDumpRegion(%x): %d,%d-%d,%d %d\n\0".as_ptr() as PCHAR,
        hRgn,
        (*Data).rdh.rcBound.left,
        (*Data).rdh.rcBound.top,
        (*Data).rdh.rcBound.right,
        (*Data).rdh.rcBound.bottom,
        (*Data).rdh.iType,
    );

    REGION_UnlockRgn(Data);
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn REGION_Complexity(prgn: PREGION) -> INT {
    if prgn.is_null() {
        return NULLREGION as INT;
    }

    match (*prgn).rdh.nCount {
        0 => NULLREGION as INT,
        1 => SIMPLEREGION as INT,
        _ => COMPLEXREGION as INT,
    }
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn REGION_Complexity(prgn: PREGION) -> INT {
    if prgn.is_null() {
        return NULLREGION as INT;
    }

    match (*prgn).rdh.nCount {
        0 => NULLREGION as INT,
        1 => SIMPLEREGION as INT,
        _ => COMPLEXREGION as INT,
    }
}

unsafe fn REGION_CopyRegion(
    dst: PREGION,
    src: PREGION) -> u8
{
    if dst != src
    {
        if (*dst).rdh.nRgnSize < (*src).rdh.nCount * core::mem::size_of::<RECT>() as u32
        {
            let temp = ExAllocatePoolWithTag(
                PagedPool,
                ((*src).rdh.nCount * core::mem::size_of::<RECT>() as u32) as SIZE_T,
                0x52676e54u32) as PRECTL;
            
            if temp.is_null() {
                return 0u8;
            }

            if !(*dst).Buffer.is_null() && (*dst).Buffer != core::ptr::addr_of_mut!((*dst).rdh.rcBound) {
                ExFreePoolWithTag((*dst).Buffer as *mut core::ffi::c_void, 0x52676e54u32);
            }

            (*dst).Buffer = temp;
            (*dst).rdh.nRgnSize = (*src).rdh.nCount * core::mem::size_of::<RECT>() as u32;
        }

        (*dst).rdh.nCount = (*src).rdh.nCount;
        (*dst).rdh.rcBound.left = (*src).rdh.rcBound.left;
        (*dst).rdh.rcBound.top = (*src).rdh.rcBound.top;
        (*dst).rdh.rcBound.right = (*src).rdh.rcBound.right;
        (*dst).rdh.rcBound.bottom = (*src).rdh.rcBound.bottom;
        (*dst).rdh.iType = (*src).rdh.iType;
        
        let mut xDest = (*dst).Buffer;
        let mut xSrc = (*src).Buffer;
        let mut xRects = (*src).rdh.nCount;
        while xRects > 0 {
            xRects -= 1;
            *xDest = *xSrc;
            xDest = xDest.offset(1);
            xSrc = xSrc.offset(1);
        }
    }

    return 1u8;
}

unsafe fn REGION_SetExtents(pReg: PREGION) {
    let mut pRect: *mut RECTL;
    let mut pRectEnd: *mut RECTL;
    let mut pExtents: *mut RECTL;

    if (*pReg).rdh.nCount == 0 {
        (*pReg).rdh.rcBound.left = 0;
        (*pReg).rdh.rcBound.top = 0;
        (*pReg).rdh.rcBound.right = 0;
        (*pReg).rdh.rcBound.bottom = 0;
        (*pReg).rdh.iType = RDH_RECTANGLES;
        return;
    }

    pExtents = core::ptr::addr_of_mut!((*pReg).rdh.rcBound);
    pRect = (*pReg).Buffer;
    pRectEnd = (*pReg).Buffer.add(((*pReg).rdh.nCount - 1) as usize);

    (*pExtents).left = (*pRect).left;
    (*pExtents).top = (*pRect).top;
    (*pExtents).right = (*pRectEnd).right;
    (*pExtents).bottom = (*pRectEnd).bottom;

    while pRect <= pRectEnd {
        if (*pRect).left < (*pExtents).left {
            (*pExtents).left = (*pRect).left;
        }
        if (*pRect).right > (*pExtents).right {
            (*pExtents).right = (*pRect).right;
        }
        pRect = pRect.add(1);
    }

    (*pReg).rdh.iType = RDH_RECTANGLES;
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn REGION_CropRegion(
    rgnDst: PREGION,
    rgnSrc: PREGION,
    rect: *const RECTL,
) -> INT {
    let mut lpr: PRECTL;
    let mut rpr: PRECTL;
    let mut i: ULONG;
    let mut j: ULONG;
    let mut clipa: ULONG;
    let mut clipb: ULONG;
    let mut nRgnSize: ULONG;
    let mut left: INT = MAXLONG;
    let mut right: INT = MINLONG;
    let mut top: INT = MAXLONG;
    let mut bottom: INT = MINLONG;

    if ((*rect).left >= (*rect).right) ||
       ((*rect).top >= (*rect).bottom) ||
       (((*rect).right > (*rgnSrc).rdh.rcBound.left &&
         (*rect).left < (*rgnSrc).rdh.rcBound.right &&
         (*rect).bottom > (*rgnSrc).rdh.rcBound.top &&
         (*rect).top < (*rgnSrc).rdh.rcBound.bottom) == false)
    {
        'empty: loop {
            if (*rgnDst).Buffer.is_null() {
                (*rgnDst).Buffer = core::ptr::addr_of_mut!((*rgnDst).rdh.rcBound);
            }

            (*rgnDst).rdh.nCount = 0;
            (*rgnDst).rdh.rcBound.left = 0;
            (*rgnDst).rdh.rcBound.top = 0;
            (*rgnDst).rdh.rcBound.right = 0;
            (*rgnDst).rdh.rcBound.bottom = 0;
            (*rgnDst).rdh.iType = RDH_RECTANGLES;

            return NULLREGION as INT;
        }
    }

    clipa = 0;
    while clipa < (*rgnSrc).rdh.nCount {
        if (*(*rgnSrc).Buffer.offset(clipa as isize)).bottom > (*rect).top {
            break;
        }
        clipa += 1;
    }

    if clipa == (*rgnSrc).rdh.nCount {
        'empty: loop {
            if (*rgnDst).Buffer.is_null() {
                (*rgnDst).Buffer = core::ptr::addr_of_mut!((*rgnDst).rdh.rcBound);
            }

            (*rgnDst).rdh.nCount = 0;
            (*rgnDst).rdh.rcBound.left = 0;
            (*rgnDst).rdh.rcBound.top = 0;
            (*rgnDst).rdh.rcBound.right = 0;
            (*rgnDst).rdh.rcBound.bottom = 0;
            (*rgnDst).rdh.iType = RDH_RECTANGLES;

            return NULLREGION as INT;
        }
    }

    clipb = clipa;
    while clipb < (*rgnSrc).rdh.nCount {
        if (*(*rgnSrc).Buffer.offset(clipb as isize)).top >= (*rect).bottom {
            break;
        }
        clipb += 1;
    }

    if clipb == clipa {
        'empty: loop {
            if (*rgnDst).Buffer.is_null() {
                (*rgnDst).Buffer = core::ptr::addr_of_mut!((*rgnDst).rdh.rcBound);
            }

            (*rgnDst).rdh.nCount = 0;
            (*rgnDst).rdh.rcBound.left = 0;
            (*rgnDst).rdh.rcBound.top = 0;
            (*rgnDst).rdh.rcBound.right = 0;
            (*rgnDst).rdh.rcBound.bottom = 0;
            (*rgnDst).rdh.iType = RDH_RECTANGLES;

            return NULLREGION as INT;
        }
    }

    nRgnSize = (clipb - clipa) * core::mem::size_of::<RECT>() as ULONG;
    if (rgnDst != rgnSrc) && ((*rgnDst).rdh.nRgnSize < nRgnSize) {
        let temp: PRECTL = ExAllocatePoolWithTag(
            PagedPool,
            nRgnSize as SIZE_T,
            0x52676e54u32,
        ) as PRECTL;
        if temp.is_null() {
            return ERROR as INT;
        }

        if !(*rgnDst).Buffer.is_null() && ((*rgnDst).Buffer != core::ptr::addr_of_mut!((*rgnDst).rdh.rcBound)) {
            ExFreePoolWithTag((*rgnDst).Buffer as *mut core::ffi::c_void, 0x52676e54u32);
        }

        (*rgnDst).Buffer = temp;
        (*rgnDst).rdh.nCount = 0;
        (*rgnDst).rdh.nRgnSize = nRgnSize;
        (*rgnDst).rdh.iType = RDH_RECTANGLES;
    }

    i = clipa;
    j = 0;
    while i < clipb {
        lpr = &mut (*(*rgnSrc).Buffer.offset(i as isize));

        if ((*lpr).left < (*rect).right) && ((*lpr).right > (*rect).left) {
            rpr = &mut (*(*rgnDst).Buffer.offset(j as isize));

            (*rpr).top = if (*lpr).top > (*rect).top { (*lpr).top } else { (*rect).top };
            (*rpr).bottom = if (*lpr).bottom < (*rect).bottom { (*lpr).bottom } else { (*rect).bottom };
            (*rpr).left = if (*lpr).left > (*rect).left { (*lpr).left } else { (*rect).left };
            (*rpr).right = if (*lpr).right < (*rect).right { (*lpr).right } else { (*rect).right };

            if (*rpr).left < left {
                left = (*rpr).left;
            }
            if (*rpr).right > right {
                right = (*rpr).right;
            }
            if (*rpr).top < top {
                top = (*rpr).top;
            }
            if (*rpr).bottom > bottom {
                bottom = (*rpr).bottom;
            }

            j += 1;
        }

        i += 1;
    }

    if j == 0 {
        'empty: loop {
            if (*rgnDst).Buffer.is_null() {
                (*rgnDst).Buffer = core::ptr::addr_of_mut!((*rgnDst).rdh.rcBound);
            }

            (*rgnDst).rdh.nCount = 0;
            (*rgnDst).rdh.rcBound.left = 0;
            (*rgnDst).rdh.rcBound.top = 0;
            (*rgnDst).rdh.rcBound.right = 0;
            (*rgnDst).rdh.rcBound.bottom = 0;
            (*rgnDst).rdh.iType = RDH_RECTANGLES;

            return NULLREGION as INT;
        }
    }

    (*rgnDst).rdh.rcBound.left = left;
    (*rgnDst).rdh.rcBound.right = right;
    (*rgnDst).rdh.rcBound.top = top;
    (*rgnDst).rdh.rcBound.bottom = bottom;

    (*rgnDst).rdh.nCount = j;

    REGION_Complexity(rgnDst)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn REGION_CropRegion(
    rgnDst: PREGION,
    rgnSrc: PREGION,
    rect: *const RECTL,
) -> INT {
    let mut lpr: PRECTL;
    let mut rpr: PRECTL;
    let mut i: ULONG;
    let mut j: ULONG;
    let mut clipa: ULONG;
    let mut clipb: ULONG;
    let mut nRgnSize: ULONG;
    let mut left: INT = MAXLONG;
    let mut right: INT = MINLONG;
    let mut top: INT = MAXLONG;
    let mut bottom: INT = MINLONG;

    if ((*rect).left >= (*rect).right) ||
       ((*rect).top >= (*rect).bottom) ||
       (((*rect).right > (*rgnSrc).rdh.rcBound.left &&
         (*rect).left < (*rgnSrc).rdh.rcBound.right &&
         (*rect).bottom > (*rgnSrc).rdh.rcBound.top &&
         (*rect).top < (*rgnSrc).rdh.rcBound.bottom) == false)
    {
        'empty: loop {
            if (*rgnDst).Buffer.is_null() {
                (*rgnDst).Buffer = core::ptr::addr_of_mut!((*rgnDst).rdh.rcBound);
            }

            (*rgnDst).rdh.nCount = 0;
            (*rgnDst).rdh.rcBound.left = 0;
            (*rgnDst).rdh.rcBound.top = 0;
            (*rgnDst).rdh.rcBound.right = 0;
            (*rgnDst).rdh.rcBound.bottom = 0;
            (*rgnDst).rdh.iType = RDH_RECTANGLES;

            return NULLREGION as INT;
        }
    }

    clipa = 0;
    while clipa < (*rgnSrc).rdh.nCount {
        if (*(*rgnSrc).Buffer.offset(clipa as isize)).bottom > (*rect).top {
            break;
        }
        clipa += 1;
    }

    if clipa == (*rgnSrc).rdh.nCount {
        'empty: loop {
            if (*rgnDst).Buffer.is_null() {
                (*rgnDst).Buffer = core::ptr::addr_of_mut!((*rgnDst).rdh.rcBound);
            }

            (*rgnDst).rdh.nCount = 0;
            (*rgnDst).rdh.rcBound.left = 0;
            (*rgnDst).rdh.rcBound.top = 0;
            (*rgnDst).rdh.rcBound.right = 0;
            (*rgnDst).rdh.rcBound.bottom = 0;
            (*rgnDst).rdh.iType = RDH_RECTANGLES;

            return NULLREGION as INT;
        }
    }

    clipb = clipa;
    while clipb < (*rgnSrc).rdh.nCount {
        if (*(*rgnSrc).Buffer.offset(clipb as isize)).top >= (*rect).bottom {
            break;
        }
        clipb += 1;
    }

    if clipb == clipa {
        'empty: loop {
            if (*rgnDst).Buffer.is_null() {
                (*rgnDst).Buffer = core::ptr::addr_of_mut!((*rgnDst).rdh.rcBound);
            }

            (*rgnDst).rdh.nCount = 0;
            (*rgnDst).rdh.rcBound.left = 0;
            (*rgnDst).rdh.rcBound.top = 0;
            (*rgnDst).rdh.rcBound.right = 0;
            (*rgnDst).rdh.rcBound.bottom = 0;
            (*rgnDst).rdh.iType = RDH_RECTANGLES;

            return NULLREGION as INT;
        }
    }

    nRgnSize = (clipb - clipa) * core::mem::size_of::<RECT>() as ULONG;
    if (rgnDst != rgnSrc) && ((*rgnDst).rdh.nRgnSize < nRgnSize) {
        let temp: PRECTL = ExAllocatePoolWithTag(
            PagedPool,
            nRgnSize as SIZE_T,
            0x52676e54u32,
        ) as PRECTL;
        if temp.is_null() {
            return ERROR as INT;
        }

        if !(*rgnDst).Buffer.is_null() && ((*rgnDst).Buffer != core::ptr::addr_of_mut!((*rgnDst).rdh.rcBound)) {
            ExFreePoolWithTag((*rgnDst).Buffer as *mut core::ffi::c_void, 0x52676e54u32);
        }

        (*rgnDst).Buffer = temp;
        (*rgnDst).rdh.nCount = 0;
        (*rgnDst).rdh.nRgnSize = nRgnSize;
        (*rgnDst).rdh.iType = RDH_RECTANGLES;
    }

    i = clipa;
    j = 0;
    while i < clipb {
        lpr = &mut (*(*rgnSrc).Buffer.offset(i as isize));

        if ((*lpr).left < (*rect).right) && ((*lpr).right > (*rect).left) {
            rpr = &mut (*(*rgnDst).Buffer.offset(j as isize));

            (*rpr).top = if (*lpr).top > (*rect).top { (*lpr).top } else { (*rect).top };
            (*rpr).bottom = if (*lpr).bottom < (*rect).bottom { (*lpr).bottom } else { (*rect).bottom };
            (*rpr).left = if (*lpr).left > (*rect).left { (*lpr).left } else { (*rect).left };
            (*rpr).right = if (*lpr).right < (*rect).right { (*lpr).right } else { (*rect).right };

            if (*rpr).left < left {
                left = (*rpr).left;
            }
            if (*rpr).right > right {
                right = (*rpr).right;
            }
            if (*rpr).top < top {
                top = (*rpr).top;
            }
            if (*rpr).bottom > bottom {
                bottom = (*rpr).bottom;
            }

            j += 1;
        }

        i += 1;
    }

    if j == 0 {
        'empty: loop {
            if (*rgnDst).Buffer.is_null() {
                (*rgnDst).Buffer = core::ptr::addr_of_mut!((*rgnDst).rdh.rcBound);
            }

            (*rgnDst).rdh.nCount = 0;
            (*rgnDst).rdh.rcBound.left = 0;
            (*rgnDst).rdh.rcBound.top = 0;
            (*rgnDst).rdh.rcBound.right = 0;
            (*rgnDst).rdh.rcBound.bottom = 0;
            (*rgnDst).rdh.iType = RDH_RECTANGLES;

            return NULLREGION as INT;
        }
    }

    (*rgnDst).rdh.rcBound.left = left;
    (*rgnDst).rdh.rcBound.right = right;
    (*rgnDst).rdh.rcBound.top = top;
    (*rgnDst).rdh.rcBound.bottom = bottom;

    (*rgnDst).rdh.nCount = j;

    REGION_Complexity(rgnDst)
}

unsafe fn REGION_Coalesce(
    pReg: PREGION,
    prevStart: i32,
    curStart: i32,
) -> i32 {
    let mut pPrevRect: *mut RECTL;
    let mut pCurRect: *mut RECTL;
    let mut pRegEnd: *mut RECTL;
    let mut curNumRects: i32;
    let mut prevNumRects: i32;
    let bandtop: i32;

    pRegEnd = (*pReg).Buffer.offset((*pReg).rdh.nCount as isize);
    pPrevRect = (*pReg).Buffer.offset(prevStart as isize);
    prevNumRects = curStart - prevStart;

    pCurRect = (*pReg).Buffer.offset(curStart as isize);
    bandtop = (*pCurRect).top;
    curNumRects = 0;
    while pCurRect != pRegEnd && (*pCurRect).top == bandtop {
        curNumRects += 1;
        pCurRect = pCurRect.offset(1);
    }

    if pCurRect != pRegEnd {
        pRegEnd = pRegEnd.offset(-1);
        while (*pRegEnd.offset(-1)).top == (*pRegEnd).top {
            pRegEnd = pRegEnd.offset(-1);
        }

        let mut curStart = curStart;
        curStart = pRegEnd.offset_from((*pReg).Buffer) as i32;
        pRegEnd = (*pReg).Buffer.offset((*pReg).rdh.nCount as isize);
    }

    let mut curStart = curStart;

    if curNumRects == prevNumRects && curNumRects != 0 {
        pCurRect = pCurRect.offset(-(curNumRects as isize));

        if (*pPrevRect).bottom == (*pCurRect).top {
            let mut prevNumRects_local = prevNumRects;
            loop {
                if (*pPrevRect).left != (*pCurRect).left ||
                   (*pPrevRect).right != (*pCurRect).right {
                    return curStart;
                }

                pPrevRect = pPrevRect.offset(1);
                pCurRect = pCurRect.offset(1);
                prevNumRects_local -= 1;

                if prevNumRects_local == 0 {
                    break;
                }
            }

            (*pReg).rdh.nCount -= curNumRects as u32;
            pCurRect = pCurRect.offset(-(curNumRects as isize));
            pPrevRect = pPrevRect.offset(-(curNumRects as isize));

            let mut curNumRects_local = curNumRects;
            loop {
                (*pPrevRect).bottom = (*pCurRect).bottom;
                pPrevRect = pPrevRect.offset(1);
                pCurRect = pCurRect.offset(1);
                curNumRects_local -= 1;

                if curNumRects_local == 0 {
                    break;
                }
            }

            if pCurRect == pRegEnd {
                curStart = prevStart;
            } else {
                loop {
                    *pPrevRect = *pCurRect;
                    pPrevRect = pPrevRect.offset(1);
                    pCurRect = pCurRect.offset(1);

                    if pCurRect == pRegEnd {
                        break;
                    }
                }
            }
        }
    }

    curStart
}

unsafe fn REGION_RegionOp(
    newReg: PREGION,
    reg1: PREGION,
    reg2: PREGION,
    overlapFunc: Option<unsafe fn(PREGION, *mut RECTL, *mut RECTL, *mut RECTL, *mut RECTL, i32, i32) -> u8>,
    nonOverlap1Func: Option<unsafe fn(PREGION, *mut RECTL, *mut RECTL, i32, i32) -> u8>,
    nonOverlap2Func: Option<unsafe fn(PREGION, *mut RECTL, *mut RECTL, i32, i32) -> u8>,
) -> u8 {
    let mut r1: *mut RECTL;
    let mut r2: *mut RECTL;
    let mut r1End: *mut RECTL;
    let mut r2End: *mut RECTL;
    let mut ybot: i32;
    let mut ytop: i32;
    let oldRects: *mut RECTL;
    let mut prevBand: u32;
    let mut curBand: u32;
    let mut r1BandEnd: *mut RECTL;
    let mut r2BandEnd: *mut RECTL;
    let mut top: u32;
    let mut bot: u32;

    r1 = (*reg1).Buffer;
    r2 = (*reg2).Buffer;
    r1End = r1.offset((*reg1).rdh.nCount as isize);
    r2End = r2.offset((*reg2).rdh.nCount as isize);

    oldRects = (*newReg).Buffer;
    (*newReg).rdh.nCount = 0;

    (*newReg).rdh.nRgnSize = core::cmp::max((*reg1).rdh.nCount + 1, (*reg2).rdh.nCount) * 2 * core::mem::size_of::<RECT>() as u32;

    (*newReg).Buffer = ExAllocatePoolWithTag(
        PagedPool,
        (*newReg).rdh.nRgnSize as SIZE_T,
        0x52676e54,
    ) as *mut RECTL;
    if (*newReg).Buffer.is_null() {
        (*newReg).rdh.nRgnSize = 0;
        return 0u8;
    }

    if (*reg1).rdh.rcBound.top < (*reg2).rdh.rcBound.top {
        ybot = (*reg1).rdh.rcBound.top;
    } else {
        ybot = (*reg2).rdh.rcBound.top;
    }

    prevBand = 0;
    loop {
        curBand = (*newReg).rdh.nCount;

        r1BandEnd = r1;
        while r1BandEnd != r1End && (*r1BandEnd).top == (*r1).top {
            r1BandEnd = r1BandEnd.offset(1);
        }

        r2BandEnd = r2;
        while r2BandEnd != r2End && (*r2BandEnd).top == (*r2).top {
            r2BandEnd = r2BandEnd.offset(1);
        }

        if (*r1).top < (*r2).top {
            top = core::cmp::max((*r1).top, ybot) as u32;
            bot = core::cmp::min((*r1).bottom, (*r2).top) as u32;

            if top != bot && nonOverlap1Func.is_some() {
                if nonOverlap1Func.unwrap()(newReg, r1, r1BandEnd, top as i32, bot as i32) == 0 {
                    return 0u8;
                }
            }

            ytop = (*r2).top;
        } else if (*r2).top < (*r1).top {
            top = core::cmp::max((*r2).top, ybot) as u32;
            bot = core::cmp::min((*r2).bottom, (*r1).top) as u32;

            if top != bot && nonOverlap2Func.is_some() {
                if nonOverlap2Func.unwrap()(newReg, r2, r2BandEnd, top as i32, bot as i32) == 0 {
                    return 0u8;
                }
            }

            ytop = (*r1).top;
        } else {
            ytop = (*r1).top;
        }

        if (*newReg).rdh.nCount != curBand {
            prevBand = REGION_Coalesce(newReg, prevBand as i32, curBand as i32) as u32;
        }

        ybot = core::cmp::min((*r1).bottom, (*r2).bottom);
        curBand = (*newReg).rdh.nCount;
        if ybot > ytop {
            if overlapFunc.unwrap()(newReg, r1, r1BandEnd, r2, r2BandEnd, ytop, ybot) == 0 {
                return 0u8;
            }
        }

        if (*newReg).rdh.nCount != curBand {
            prevBand = REGION_Coalesce(newReg, prevBand as i32, curBand as i32) as u32;
        }

        if (*r1).bottom == ybot {
            r1 = r1BandEnd;
        }
        if (*r2).bottom == ybot {
            r2 = r2BandEnd;
        }

        if r1 == r1End || r2 == r2End {
            break;
        }
    }

    curBand = (*newReg).rdh.nCount;
    if r1 != r1End {
        if nonOverlap1Func.is_some() {
            loop {
                r1BandEnd = r1;
                while r1BandEnd < r1End && (*r1BandEnd).top == (*r1).top {
                    r1BandEnd = r1BandEnd.offset(1);
                }

                if nonOverlap1Func.unwrap()(
                    newReg,
                    r1,
                    r1BandEnd,
                    core::cmp::max((*r1).top, ybot),
                    (*r1).bottom,
                ) == 0
                {
                    return 0u8;
                }
                r1 = r1BandEnd;

                if r1 == r1End {
                    break;
                }
            }
        }
    } else if r2 != r2End && nonOverlap2Func.is_some() {
        loop {
            r2BandEnd = r2;
            while r2BandEnd < r2End && (*r2BandEnd).top == (*r2).top {
                r2BandEnd = r2BandEnd.offset(1);
            }

            if nonOverlap2Func.unwrap()(
                newReg,
                r2,
                r2BandEnd,
                core::cmp::max((*r2).top, ybot),
                (*r2).bottom,
            ) == 0
            {
                return 0u8;
            }
            r2 = r2BandEnd;

            if r2 == r2End {
                break;
            }
        }
    }

    if (*newReg).rdh.nCount != curBand {
        REGION_Coalesce(newReg, prevBand as i32, curBand as i32);
    }

    if (*newReg).rdh.nRgnSize > (2 * (*newReg).rdh.nCount * core::mem::size_of::<RECT>() as u32)
        && (*newReg).rdh.nCount > 2
    {
        if (*newReg).rdh.nCount != 0 {
            let prev_rects = (*newReg).Buffer;
            (*newReg).Buffer = ExAllocatePoolWithTag(
                PagedPool,
                ((*newReg).rdh.nCount * core::mem::size_of::<RECT>() as u32) as SIZE_T,
                0x52676e54,
            ) as *mut RECTL;

            if (*newReg).Buffer.is_null() {
                (*newReg).Buffer = prev_rects;
            } else {
                (*newReg).rdh.nRgnSize = (*newReg).rdh.nCount * core::mem::size_of::<RECT>() as u32;
                let mut xDest = (*newReg).Buffer;
                let mut xSrc = prev_rects;
                let mut xRects = (*newReg).rdh.nCount;
                while xRects > 0 {
                    *xDest = *xSrc;
                    xDest = xDest.offset(1);
                    xSrc = xSrc.offset(1);
                    xRects -= 1;
                }
                if prev_rects != core::ptr::addr_of_mut!((*newReg).rdh.rcBound) as *mut RECTL {
                    ExFreePoolWithTag(prev_rects as *mut core::ffi::c_void, 0x52676e54);
                }
            }
        } else {
            (*newReg).rdh.nRgnSize = core::mem::size_of::<RECT>() as u32;
            if (*newReg).Buffer != core::ptr::addr_of_mut!((*newReg).rdh.rcBound) as *mut RECTL {
                ExFreePoolWithTag((*newReg).Buffer as *mut core::ffi::c_void, 0x52676e54);
            }

            (*newReg).Buffer = ExAllocatePoolWithTag(
                PagedPool,
                core::mem::size_of::<RECT>() as SIZE_T,
                0x52676e54,
            ) as *mut RECTL;
        }
    }

    (*newReg).rdh.iType = RDH_RECTANGLES;

    if oldRects != core::ptr::addr_of_mut!((*newReg).rdh.rcBound) as *mut RECTL {
        ExFreePoolWithTag(oldRects as *mut core::ffi::c_void, 0x52676e54);
    }
    1u8
}

unsafe fn REGION_IntersectO(
    pReg: PREGION,
    mut r1: PRECTL,
    r1End: PRECTL,
    mut r2: PRECTL,
    r2End: PRECTL,
    top: i32,
    bottom: i32,
) -> u8 {
    let mut left: i32;
    let mut right: i32;

    while (r1 != r1End) && (r2 != r2End) {
        left = if (*r1).left > (*r2).left { (*r1).left } else { (*r2).left };
        right = if (*r1).right < (*r2).right { (*r1).right } else { (*r2).right };

        if left < right {
            if REGION_bAddRect(pReg, left, top, right, bottom) == 0 {
                return 0;
            }
        }

        if (*r1).right < (*r2).right {
            r1 = r1.offset(1);
        } else if (*r2).right < (*r1).right {
            r2 = r2.offset(1);
        } else {
            r1 = r1.offset(1);
            r2 = r2.offset(1);
        }
    }

    1
}

unsafe fn REGION_IntersectRegion(
    newReg: PREGION,
    reg1: PREGION,
    reg2: PREGION,
) -> u8 {
    if ((*reg1).rdh.nCount == 0) ||
        ((*reg2).rdh.nCount == 0) ||
        (EXTENTCHECK(
            core::ptr::addr_of_mut!((*reg1).rdh.rcBound),
            core::ptr::addr_of_mut!((*reg2).rdh.rcBound)
        ) == 0)
    {
        (*newReg).rdh.nCount = 0;
    }
    else
    {
        if REGION_RegionOp(
            newReg,
            reg1,
            reg2,
            Some(REGION_IntersectO),
            None,
            None,
        ) == 0
        {
            return 0u8;
        }
    }

    REGION_SetExtents(newReg);
    return 1u8;
}

unsafe fn REGION_UnionNonO(
    pReg: PREGION,
    mut r: PRECTL,
    rEnd: PRECTL,
    top: i32,
    bottom: i32,
) -> u8 {
    if r != rEnd {
        if REGION_bEnsureBufferSize(
            pReg,
            (*pReg).rdh.nCount.wrapping_add(rEnd.offset_from(r) as u32),
        ) == 0
        {
            return 0u8;
        }

        loop {
            REGION_vAddRect(pReg, (*r).left, top, (*r).right, bottom);
            r = r.offset(1);
            if r == rEnd {
                break;
            }
        }
    }

    1u8
}

unsafe fn REGION_bMergeRect(
    prgn: PREGION,
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
) -> u8 {
    if ((*prgn).rdh.nCount != 0) &&
        ((*(*prgn).Buffer.offset(((*prgn).rdh.nCount - 1) as isize)).top == top) &&
        ((*(*prgn).Buffer.offset(((*prgn).rdh.nCount - 1) as isize)).bottom == bottom) &&
        ((*(*prgn).Buffer.offset(((*prgn).rdh.nCount - 1) as isize)).right >= left)
    {
        if (*(*prgn).Buffer.offset(((*prgn).rdh.nCount - 1) as isize)).right < right
        {
            (*(*prgn).Buffer.offset(((*prgn).rdh.nCount - 1) as isize)).right = right;
        }
    }
    else
    {
        if REGION_bAddRect(prgn, left, top, right, bottom) == 0
        {
            return 0u8;
        }
    }

    1u8
}

#[cfg(target_arch = "x86")]
unsafe fn REGION_UnionO(
    pReg: PREGION,
    mut r1: PRECTL,
    r1End: PRECTL,
    mut r2: PRECTL,
    r2End: PRECTL,
    top: i32,
    bottom: i32,
) -> u8 {
    while (r1 != r1End) && (r2 != r2End) {
        if (*r1).left < (*r2).left {
            if REGION_bMergeRect(pReg, (*r1).left, top, (*r1).right, bottom) == 0 {
                return 0;
            }
            r1 = r1.offset(1);
        } else {
            if REGION_bMergeRect(pReg, (*r2).left, top, (*r2).right, bottom) == 0 {
                return 0;
            }
            r2 = r2.offset(1);
        }
    }

    if r1 != r1End {
        loop {
            if REGION_bMergeRect(pReg, (*r1).left, top, (*r1).right, bottom) == 0 {
                return 0;
            }
            r1 = r1.offset(1);
            if r1 == r1End {
                break;
            }
        }
    } else {
        while r2 != r2End {
            if REGION_bMergeRect(pReg, (*r2).left, top, (*r2).right, bottom) == 0 {
                return 0;
            }
            r2 = r2.offset(1);
        }
    }

    1
}

#[cfg(not(target_arch = "x86"))]
unsafe fn REGION_UnionO(
    pReg: PREGION,
    mut r1: PRECTL,
    r1End: PRECTL,
    mut r2: PRECTL,
    r2End: PRECTL,
    top: i32,
    bottom: i32,
) -> u8 {
    while (r1 != r1End) && (r2 != r2End) {
        if (*r1).left < (*r2).left {
            if REGION_bMergeRect(pReg, (*r1).left, top, (*r1).right, bottom) == 0 {
                return 0;
            }
            r1 = r1.offset(1);
        } else {
            if REGION_bMergeRect(pReg, (*r2).left, top, (*r2).right, bottom) == 0 {
                return 0;
            }
            r2 = r2.offset(1);
        }
    }

    if r1 != r1End {
        loop {
            if REGION_bMergeRect(pReg, (*r1).left, top, (*r1).right, bottom) == 0 {
                return 0;
            }
            r1 = r1.offset(1);
            if r1 == r1End {
                break;
            }
        }
    } else {
        while r2 != r2End {
            if REGION_bMergeRect(pReg, (*r2).left, top, (*r2).right, bottom) == 0 {
                return 0;
            }
            r2 = r2.offset(1);
        }
    }

    1
}

unsafe fn REGION_UnionRegion(
    newReg: PREGION,
    reg1: PREGION,
    reg2: PREGION) -> u8
{
    let mut ret: u8 = 1u8;

    if (reg1 == reg2) || ((*reg1).rdh.nCount == 0) ||
        ((*reg1).rdh.rcBound.right <= (*reg1).rdh.rcBound.left) ||
        ((*reg1).rdh.rcBound.bottom <= (*reg1).rdh.rcBound.top)
    {
        if newReg != reg2
        {
            ret = REGION_CopyRegion(newReg, reg2);
        }

        return ret;
    }

    if ((*reg2).rdh.nCount == 0) ||
        ((*reg2).rdh.rcBound.right <= (*reg2).rdh.rcBound.left) ||
        ((*reg2).rdh.rcBound.bottom <= (*reg2).rdh.rcBound.top)
    {
        if newReg != reg1
        {
            ret = REGION_CopyRegion(newReg, reg1);
        }

        return ret;
    }

    if ((*reg1).rdh.nCount == 1) &&
        ((*reg1).rdh.rcBound.left <= (*reg2).rdh.rcBound.left) &&
        ((*reg1).rdh.rcBound.top <= (*reg2).rdh.rcBound.top) &&
        ((*reg2).rdh.rcBound.right <= (*reg1).rdh.rcBound.right) &&
        ((*reg2).rdh.rcBound.bottom <= (*reg1).rdh.rcBound.bottom)
    {
        if newReg != reg1
        {
            ret = REGION_CopyRegion(newReg, reg1);
        }

        return ret;
    }

    if ((*reg2).rdh.nCount == 1) &&
        ((*reg2).rdh.rcBound.left <= (*reg1).rdh.rcBound.left) &&
        ((*reg2).rdh.rcBound.top <= (*reg1).rdh.rcBound.top) &&
        ((*reg1).rdh.rcBound.right <= (*reg2).rdh.rcBound.right) &&
        ((*reg1).rdh.rcBound.bottom <= (*reg2).rdh.rcBound.bottom)
    {
        if newReg != reg2
        {
            ret = REGION_CopyRegion(newReg, reg2);
        }

        return ret;
    }

    ret = REGION_RegionOp(newReg,
                    reg1,
                    reg2,
                    Some(REGION_UnionO as _),
                    Some(REGION_UnionNonO as _),
                    Some(REGION_UnionNonO as _));
    if ret != 0
    {
        (*newReg).rdh.rcBound.left = min((*reg1).rdh.rcBound.left, (*reg2).rdh.rcBound.left);
        (*newReg).rdh.rcBound.top = min((*reg1).rdh.rcBound.top, (*reg2).rdh.rcBound.top);
        (*newReg).rdh.rcBound.right = max((*reg1).rdh.rcBound.right, (*reg2).rdh.rcBound.right);
        (*newReg).rdh.rcBound.bottom = max((*reg1).rdh.rcBound.bottom, (*reg2).rdh.rcBound.bottom);
    }
    ret
}

unsafe fn REGION_SubtractNonO1(
    pReg: PREGION,
    mut r: PRECTL,
    rEnd: PRECTL,
    top: i32,
    bottom: i32,
) -> u8 {
    if r != rEnd {
        if REGION_bEnsureBufferSize(pReg, (*pReg).rdh.nCount + (rEnd.offset_from(r) as u32)) == 0 {
            return 0u8;
        }

        loop {
            REGION_vAddRect(pReg, (*r).left, top, (*r).right, bottom);
            r = r.offset(1);
            if r == rEnd {
                break;
            }
        }
    }

    1u8
}

unsafe fn REGION_SubtractO(
    pReg: PREGION,
    mut r1: PRECTL,
    r1End: PRECTL,
    mut r2: PRECTL,
    r2End: PRECTL,
    top: i32,
    bottom: i32,
) -> u8 {
    let mut left: i32;

    left = (*r1).left;

    while (r1 != r1End) && (r2 != r2End) {
        if (*r2).right <= left {
            r2 = r2.offset(1);
        } else if (*r2).left <= left {
            left = (*r2).right;
            if left >= (*r1).right {
                r1 = r1.offset(1);
                if r1 != r1End {
                    left = (*r1).left;
                }
            } else {
                r2 = r2.offset(1);
            }
        } else if (*r2).left < (*r1).right {
            if REGION_bAddRect(pReg, left, top, (*r2).left, bottom) == 0 {
                return 0;
            }

            left = (*r2).right;
            if left >= (*r1).right {
                r1 = r1.offset(1);
                if r1 != r1End {
                    left = (*r1).left;
                }
            } else {
                r2 = r2.offset(1);
            }
        } else {
            if (*r1).right > left {
                if REGION_bAddRect(pReg, left, top, (*r1).right, bottom) == 0 {
                    return 0;
                }
            }

            r1 = r1.offset(1);
            if r1 != r1End {
                left = (*r1).left;
            }
        }
    }

    if r1 != r1End {
        if REGION_bEnsureBufferSize(pReg, (*pReg).rdh.nCount + (r1End.offset_from(r1) as u32)) == 0 {
            return 0;
        }

        loop {
            REGION_vAddRect(pReg, left, top, (*r1).right, bottom);
            r1 = r1.offset(1);
            if r1 != r1End {
                left = (*r1).left;
            }

            if r1 == r1End {
                break;
            }
        }
    }

    1
}

unsafe fn REGION_SubtractRegion(
    regD: PREGION,
    regM: PREGION,
    regS: PREGION) -> u8
{
    if ((*regM).rdh.nCount == 0) ||
       ((*regS).rdh.nCount == 0) ||
       (EXTENTCHECK(core::ptr::addr_of_mut!((*regM).rdh.rcBound), core::ptr::addr_of_mut!((*regS).rdh.rcBound)) == 0)
    {
        return REGION_CopyRegion(regD, regM);
    }

    if REGION_RegionOp(regD,
                       regM,
                       regS,
                       Some(REGION_SubtractO),
                       Some(REGION_SubtractNonO1 as _),
                       None) == 0
    {
        return 0u8;
    }

    REGION_SetExtents(regD);
    return 1u8;
}

unsafe fn REGION_XorRegion(
    dr: PREGION,
    sra: PREGION,
    srb: PREGION) -> u8
{
    let htra: HRGN;
    let htrb: HRGN;
    let tra: PREGION;
    let trb: PREGION;
    let ret: u8;

    tra = REGION_AllocRgnWithHandle((*sra).rdh.nCount as INT + 1);
    if tra.is_null()
    {
        return 0u8;
    }
    htra = (*tra).BaseObject.hHmgr as HRGN;

    trb = REGION_AllocRgnWithHandle((*srb).rdh.nCount as INT + 1);
    if trb.is_null()
    {
        REGION_UnlockRgn(tra);
        GreDeleteObject(htra as HGDIOBJ);
        return 0u8;
    }
    htrb = (*trb).BaseObject.hHmgr as HRGN;

    ret = if REGION_SubtractRegion(tra, sra, srb) != 0 &&
             REGION_SubtractRegion(trb, srb, sra) != 0 &&
             REGION_UnionRegion(dr, tra, trb) != 0 { 1u8 } else { 0u8 };
    REGION_UnlockRgn(tra);
    REGION_UnlockRgn(trb);

    GreDeleteObject(htra as HGDIOBJ);
    GreDeleteObject(htrb as HGDIOBJ);
    ret
}

unsafe fn REGION_UnionRectWithRgn(
    rgn: PREGION,
    rect: *const RECTL) -> u8
{
    let mut region: REGION = core::mem::zeroed();

    region.Buffer = core::ptr::addr_of_mut!(region.rdh.rcBound) as *mut RECTL;
    region.rdh.nCount = 1;
    region.rdh.nRgnSize = core::mem::size_of::<RECT>() as u32;
    region.rdh.rcBound = *rect;
    REGION_UnionRegion(rgn, rgn, core::ptr::addr_of_mut!(region))
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn REGION_SubtractRectFromRgn(
    prgnDest: PREGION,
    prgnSrc: PREGION,
    prcl: *const RECTL,
) -> INT {
    let mut rgnLocal: REGION = core::mem::zeroed();

    rgnLocal.Buffer = core::ptr::addr_of_mut!(rgnLocal.rdh.rcBound);
    rgnLocal.rdh.nCount = 1;
    rgnLocal.rdh.nRgnSize = core::mem::size_of::<RECT>() as u32;
    rgnLocal.rdh.rcBound = *prcl;
    REGION_SubtractRegion(prgnDest, prgnSrc, core::ptr::addr_of_mut!(rgnLocal));
    REGION_Complexity(prgnDest)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn REGION_SubtractRectFromRgn(
    prgnDest: PREGION,
    prgnSrc: PREGION,
    prcl: *const RECTL,
) -> INT {
    let mut rgnLocal: REGION = core::mem::zeroed();

    rgnLocal.Buffer = core::ptr::addr_of_mut!(rgnLocal.rdh.rcBound);
    rgnLocal.rdh.nCount = 1;
    rgnLocal.rdh.nRgnSize = core::mem::size_of::<RECT>() as u32;
    rgnLocal.rdh.rcBound = *prcl;
    REGION_SubtractRegion(prgnDest, prgnSrc, core::ptr::addr_of_mut!(rgnLocal));
    REGION_Complexity(prgnDest)
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn REGION_bCopy(dst: PREGION, src: PREGION) -> BOOL { REGION_bCopy_impl(dst, src) as BOOL }
#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn REGION_bCopy(dst: PREGION, src: PREGION) -> BOOL { REGION_bCopy_impl(dst, src) as BOOL }
unsafe fn REGION_bCopy_impl(
    dst: PREGION,
    src: PREGION) -> u8
{
    if dst.is_null() || src.is_null() {
        return 0u8;
    }
    return REGION_CopyRegion(dst, src);
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn REGION_bIntersectRegion(
    newReg: PREGION,
    reg1: PREGION,
    reg2: PREGION,
) -> BOOL {
    if newReg.is_null() || reg1.is_null() || reg2.is_null() {
        return 0;
    }
    return REGION_IntersectRegion(newReg, reg1, reg2) as BOOL;
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn REGION_bIntersectRegion(
    newReg: PREGION,
    reg1: PREGION,
    reg2: PREGION,
) -> BOOL {
    if newReg.is_null() || reg1.is_null() || reg2.is_null() {
        return 0;
    }
    return REGION_IntersectRegion(newReg, reg1, reg2) as BOOL;
}

unsafe fn REGION_bMakeSimpleFrameRgn(
    prgn: PREGION,
    prclSrc: PRECTL,
    cx: i32,
    cy: i32,
) -> u8 {
    let mut arcl: [RECTL; 4] = core::mem::zeroed();
    let mut i: u32;

    NT_ASSERT!((cx >= 0) && (cy >= 0));
    NT_ASSERT!(((*prclSrc).bottom > (*prclSrc).top) &&
              ((*prclSrc).right > (*prclSrc).left));

    (*prgn).rdh.nCount = 0;
    (*prgn).rdh.rcBound.left = 0;
    (*prgn).rdh.rcBound.top = 0;
    (*prgn).rdh.rcBound.right = 0;
    (*prgn).rdh.rcBound.bottom = 0;
    (*prgn).rdh.iType = 1;

    if (((*prclSrc).bottom - (*prclSrc).top) <= cy * 2) ||
       (((*prclSrc).right - (*prclSrc).left) <= cx * 2) {
        (*prgn).rdh.rcBound = *prclSrc;
        *(*prgn).Buffer = *prclSrc;
        (*prgn).rdh.nCount = 1;
        return 1u8;
    }

    i = 0;

    if cy != 0 {
        arcl[i as usize].left = (*prclSrc).left;
        arcl[i as usize].top = (*prclSrc).top;
        arcl[i as usize].right = (*prclSrc).right;
        arcl[i as usize].bottom = (*prclSrc).top + cy;
        i += 1;
    }

    if cx != 0 {
        arcl[i as usize].left = (*prclSrc).left;
        arcl[i as usize].top = (*prclSrc).top + cy;
        arcl[i as usize].right = (*prclSrc).left + cx;
        arcl[i as usize].bottom = (*prclSrc).bottom - cy;
        i += 1;

        arcl[i as usize].left = (*prclSrc).right - cx;
        arcl[i as usize].top = (*prclSrc).top + cy;
        arcl[i as usize].right = (*prclSrc).right;
        arcl[i as usize].bottom = (*prclSrc).bottom - cy;
        i += 1;
    }

    if cy != 0 {
        arcl[i as usize].left = (*prclSrc).left;
        arcl[i as usize].top = (*prclSrc).bottom - cy;
        arcl[i as usize].right = (*prclSrc).right;
        arcl[i as usize].bottom = (*prclSrc).bottom;
        i += 1;
    }

    if i != 0 {
        (*prgn).rdh.nCount = i;
        NT_ASSERT!((*prgn).rdh.nCount > 1);
        (*prgn).rdh.nRgnSize = (*prgn).rdh.nCount * core::mem::size_of::<RECT>() as u32;
        NT_ASSERT!((*prgn).Buffer == core::ptr::addr_of_mut!((*prgn).rdh.rcBound) as *mut RECTL);
        (*prgn).Buffer = ExAllocatePoolWithTag(
            PagedPool,
            (*prgn).rdh.nRgnSize as SIZE_T,
            0x52676e54u32,
        ) as *mut RECTL;
        if (*prgn).Buffer.is_null() {
            (*prgn).rdh.nRgnSize = 0;
            return 0u8;
        }

        let mut xDest = (*prgn).Buffer;
        let mut xSrc = arcl.as_ptr();
        let mut xRects = (*prgn).rdh.nCount;
        while xRects > 0 {
            *xDest = *xSrc;
            xDest = xDest.offset(1);
            xSrc = xSrc.offset(1);
            xRects -= 1;
        }
    }

    1u8
}

unsafe fn REGION_bMakeFrameRegion(
    prgnDest: PREGION,
    prgnSrc: PREGION,
    cx: INT,
    cy: INT) -> BOOL
{
    let mut cx = cx.abs();
    let mut cy = cy.abs();

    if ((cx as UINT) > MAX_COORD) || ((cy as UINT) > MAX_COORD)
    {
        return 0;
    }

    if (*prgnSrc).rdh.nCount == 0
    {
        return 0;
    }

    if (cx == 0) && (cy == 0)
    {
        (*prgnDest).rdh.nCount = 0;
        (*prgnDest).rdh.rcBound.left = 0;
        (*prgnDest).rdh.rcBound.top = 0;
        (*prgnDest).rdh.rcBound.right = 0;
        (*prgnDest).rdh.rcBound.bottom = 0;
        (*prgnDest).rdh.iType = RDH_RECTANGLES;
        return 1;
    }

    if REGION_Complexity(prgnSrc) == SIMPLEREGION as INT
    {
        return REGION_bMakeSimpleFrameRgn(prgnDest, core::ptr::addr_of_mut!((*prgnSrc).rdh.rcBound), cx, cy) as BOOL;
    }

    if ((*prgnSrc).rdh.rcBound.left < (MIN_COORD + cx)) ||
        ((*prgnSrc).rdh.rcBound.top < (MIN_COORD + cy)) ||
        ((*prgnSrc).rdh.rcBound.right > (MAX_COORD as INT - cx)) ||
        ((*prgnSrc).rdh.rcBound.bottom > (MAX_COORD as INT - cy))
    {
        return 0;
    }

    if REGION_CopyRegion(prgnDest, prgnSrc) == 0
    {
        return 0;
    }

    REGION_bOffsetRgn(prgnSrc, cx, cy);

    REGION_IntersectRegion(prgnDest, prgnDest, prgnSrc);

    REGION_bOffsetRgn(prgnSrc, -2 * cx, 0);

    REGION_IntersectRegion(prgnDest, prgnDest, prgnSrc);

    REGION_bOffsetRgn(prgnSrc, 0, -2 * cy);

    REGION_IntersectRegion(prgnDest, prgnDest, prgnSrc);

    REGION_bOffsetRgn(prgnSrc, 2 * cx, 0);

    REGION_IntersectRegion(prgnDest, prgnDest, prgnSrc);

    REGION_bOffsetRgn(prgnSrc, -cx, cy);

    REGION_SubtractRegion(prgnDest, prgnSrc, prgnDest);

    1
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn GreCreateFrameRgn(
    hrgn: HRGN,
    cx: INT,
    cy: INT,
) -> HRGN {
    let prgnFrame: PREGION;
    let prgnSrc: PREGION;
    let hrgnFrame: HRGN;

    prgnFrame = REGION_AllocUserRgnWithHandle(1);
    if prgnFrame.is_null() {
        EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return core::ptr::null_mut();
    }

    prgnSrc = REGION_LockRgn(hrgn);
    if prgnSrc.is_null() {
        REGION_Delete(prgnFrame);
        return 0 as HRGN;
    }

    if REGION_bMakeFrameRegion(prgnFrame, prgnSrc, cx, cy) != 0 {
        hrgnFrame = (*prgnFrame).BaseObject.hHmgr as HRGN;
        REGION_UnlockRgn(prgnFrame);
    } else {
        REGION_Delete(prgnFrame);
        hrgnFrame = core::ptr::null_mut();
    }

    REGION_UnlockRgn(prgnSrc);
    hrgnFrame
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn GreCreateFrameRgn(
    hrgn: HRGN,
    cx: INT,
    cy: INT,
) -> HRGN {
    let prgnFrame: PREGION;
    let prgnSrc: PREGION;
    let hrgnFrame: HRGN;

    prgnFrame = REGION_AllocUserRgnWithHandle(1);
    if prgnFrame.is_null() {
        EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return core::ptr::null_mut();
    }

    prgnSrc = REGION_LockRgn(hrgn);
    if prgnSrc.is_null() {
        REGION_Delete(prgnFrame);
        return 0 as HRGN;
    }

    if REGION_bMakeFrameRegion(prgnFrame, prgnSrc, cx, cy) != 0 {
        hrgnFrame = (*prgnFrame).BaseObject.hHmgr as HRGN;
        REGION_UnlockRgn(prgnFrame);
    } else {
        REGION_Delete(prgnFrame);
        hrgnFrame = core::ptr::null_mut();
    }

    REGION_UnlockRgn(prgnSrc);
    hrgnFrame
}

#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "fastcall" fn REGION_bXformRgn(
    prgn: PREGION,
    pmx: PMATRIX,
) -> BOOL {
    let mut xo: EXFORMOBJ = core::mem::zeroed(); // 8B on 64-bit: holds the pmx XFORMOBJ_vInit writes (XFORMOBJ itself is 4B → OOB)
    let mut i: ULONG;
    let mut cjSize: ULONG;
    let mut ppt: PPOINT;
    let mut pcPoints: PULONG;
    let mut rect: RECT = core::mem::zeroed();
    let bResult: BOOL;

    if (*prgn).rdh.nCount < 1 {
        return if ((*pmx).flAccel & XFORM_UNITY) != 0 { 1 } else { 0 };
    }

    if ((*pmx).flAccel & XFORM_SCALE) != 0 {
        if ((*pmx).flAccel & XFORM_UNITY) != 0 {
            return REGION_bOffsetRgn(prgn, ((*pmx).fxDx + 8) / 16, ((*pmx).fxDy + 8) / 16) as BOOL;
        } else {
            XFORMOBJ_vInit((core::ptr::addr_of_mut!(xo) as *mut XFORMOBJ), pmx);

            if EXFORMOBJ_bApplyXform(
                (core::ptr::addr_of_mut!(xo) as *mut XFORMOBJ),
                XF_LTOL,
                2,
                core::ptr::addr_of_mut!((*prgn).rdh.rcBound) as *mut core::ffi::c_void,
                core::ptr::addr_of_mut!(rect) as *mut core::ffi::c_void,
            ) == 0
            {
                return 0;
            }

            if EXFORMOBJ_bApplyXform(
                (core::ptr::addr_of_mut!(xo) as *mut XFORMOBJ),
                XF_LTOL,
                (*prgn).rdh.nCount * 2,
                (*prgn).Buffer as *mut core::ffi::c_void,
                (*prgn).Buffer as *mut core::ffi::c_void,
            ) == 0
            {
                NT_ASSERT!(0);
            }

            RECTL_vSetEmptyRect(core::ptr::addr_of_mut!((*prgn).rdh.rcBound));

            i = 0;
            while i < (*prgn).rdh.nCount {
                RECTL_vMakeWellOrdered((*prgn).Buffer.add(i as usize));

                if RECTL_bUnionRect(
                    core::ptr::addr_of_mut!((*prgn).rdh.rcBound),
                    core::ptr::addr_of_mut!((*prgn).rdh.rcBound),
                    (*prgn).Buffer.add(i as usize),
                ) == 0
                {
                    return 0;
                }
                i += 1;
            }

            i = 0;
            while i < (*prgn).rdh.nCount - 1 {
                NT_ASSERT!((*(*prgn).Buffer.add(i as usize)).top <= (*(*prgn).Buffer.add(i as usize)).bottom);
                NT_ASSERT!((*(*prgn).Buffer.add((i + 1) as usize)).top >= (*(*prgn).Buffer.add(i as usize)).top);
                i += 1;
            }

            return 1;
        }
    } else {
        cjSize = (*prgn).rdh.nCount * (4 * core::mem::size_of::<POINT>() as u32 + core::mem::size_of::<ULONG>() as u32);
        ppt = ExAllocatePoolWithTag(PagedPool, cjSize as SIZE_T, 0x6e526747u32) as PPOINT;
        if ppt.is_null() {
            return 0;
        }

        pcPoints = ppt.add((4 * (*prgn).rdh.nCount) as usize) as PULONG;
        i = 0;
        while i < (*prgn).rdh.nCount {
            *pcPoints.add(i as usize) = 4;
            (*ppt.add((4 * i + 0) as usize)).x = (*(*prgn).Buffer.add(i as usize)).left;
            (*ppt.add((4 * i + 0) as usize)).y = (*(*prgn).Buffer.add(i as usize)).top;
            (*ppt.add((4 * i + 1) as usize)).x = (*(*prgn).Buffer.add(i as usize)).right;
            (*ppt.add((4 * i + 1) as usize)).y = (*(*prgn).Buffer.add(i as usize)).top;
            (*ppt.add((4 * i + 2) as usize)).x = (*(*prgn).Buffer.add(i as usize)).right;
            (*ppt.add((4 * i + 2) as usize)).y = (*(*prgn).Buffer.add(i as usize)).bottom;
            (*ppt.add((4 * i + 3) as usize)).x = (*(*prgn).Buffer.add(i as usize)).left;
            (*ppt.add((4 * i + 3) as usize)).y = (*(*prgn).Buffer.add(i as usize)).bottom;
            i += 1;
        }

        XFORMOBJ_vInit((core::ptr::addr_of_mut!(xo) as *mut XFORMOBJ), pmx);

        if EXFORMOBJ_bApplyXform(
            (core::ptr::addr_of_mut!(xo) as *mut XFORMOBJ),
            XF_LTOL,
            (*prgn).rdh.nCount * 2,
            ppt as *mut core::ffi::c_void,
            ppt as *mut core::ffi::c_void,
        ) == 0
        {
            ExFreePoolWithTag(ppt as *mut core::ffi::c_void, 0x6e526747u32);
            return 0;
        }

        bResult = REGION_SetPolyPolygonRgn(
            prgn,
            ppt,
            pcPoints,
            (*prgn).rdh.nCount,
            WINDING as INT,
        );

        ExFreePoolWithTag(ppt as *mut core::ffi::c_void, 0x6e526747u32);

        return bResult;
    }
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
unsafe extern "C" fn REGION_bXformRgn(
    prgn: PREGION,
    pmx: PMATRIX,
) -> BOOL {
    let mut xo: EXFORMOBJ = core::mem::zeroed(); // 8B on 64-bit: holds the pmx XFORMOBJ_vInit writes (XFORMOBJ itself is 4B → OOB)
    let mut i: ULONG;
    let mut cjSize: ULONG;
    let mut ppt: PPOINT;
    let mut pcPoints: PULONG;
    let mut rect: RECT = core::mem::zeroed();
    let bResult: BOOL;

    if (*prgn).rdh.nCount < 1 {
        return if ((*pmx).flAccel & XFORM_UNITY) != 0 { 1 } else { 0 };
    }

    if ((*pmx).flAccel & XFORM_SCALE) != 0 {
        if ((*pmx).flAccel & XFORM_UNITY) != 0 {
            return REGION_bOffsetRgn(prgn, ((*pmx).fxDx + 8) / 16, ((*pmx).fxDy + 8) / 16) as BOOL;
        } else {
            XFORMOBJ_vInit(core::ptr::addr_of_mut!(xo) as *mut XFORMOBJ , pmx);

            if EXFORMOBJ_bApplyXform(
                core::ptr::addr_of_mut!(xo) as *mut XFORMOBJ ,
                XF_LTOL,
                2,
                core::ptr::addr_of_mut!((*prgn).rdh.rcBound) as *mut core::ffi::c_void,
                core::ptr::addr_of_mut!(rect) as *mut core::ffi::c_void,
            ) == 0
            {
                return 0;
            }

            if EXFORMOBJ_bApplyXform(
                core::ptr::addr_of_mut!(xo) as *mut XFORMOBJ ,
                XF_LTOL,
                (*prgn).rdh.nCount * 2,
                (*prgn).Buffer as *mut core::ffi::c_void,
                (*prgn).Buffer as *mut core::ffi::c_void,
            ) == 0
            {
                NT_ASSERT!(0);
            }

            RECTL_vSetEmptyRect(core::ptr::addr_of_mut!((*prgn).rdh.rcBound));

            i = 0;
            while i < (*prgn).rdh.nCount {
                RECTL_vMakeWellOrdered((*prgn).Buffer.add(i as usize));

                if RECTL_bUnionRect(
                    core::ptr::addr_of_mut!((*prgn).rdh.rcBound),
                    core::ptr::addr_of_mut!((*prgn).rdh.rcBound),
                    (*prgn).Buffer.add(i as usize),
                ) == 0
                {
                    return 0;
                }
                i += 1;
            }

            i = 0;
            while i < (*prgn).rdh.nCount - 1 {
                NT_ASSERT!((*(*prgn).Buffer.add(i as usize)).top <= (*(*prgn).Buffer.add(i as usize)).bottom);
                NT_ASSERT!((*(*prgn).Buffer.add((i + 1) as usize)).top >= (*(*prgn).Buffer.add(i as usize)).top);
                i += 1;
            }

            return 1;
        }
    } else {
        cjSize = (*prgn).rdh.nCount * (4 * core::mem::size_of::<POINT>() as u32 + core::mem::size_of::<ULONG>() as u32);
        ppt = ExAllocatePoolWithTag(PagedPool, cjSize as SIZE_T, 0x6e526747u32) as PPOINT;
        if ppt.is_null() {
            return 0;
        }

        pcPoints = ppt.add((4 * (*prgn).rdh.nCount) as usize) as PULONG;
        i = 0;
        while i < (*prgn).rdh.nCount {
            *pcPoints.add(i as usize) = 4;
            (*ppt.add((4 * i + 0) as usize)).x = (*(*prgn).Buffer.add(i as usize)).left;
            (*ppt.add((4 * i + 0) as usize)).y = (*(*prgn).Buffer.add(i as usize)).top;
            (*ppt.add((4 * i + 1) as usize)).x = (*(*prgn).Buffer.add(i as usize)).right;
            (*ppt.add((4 * i + 1) as usize)).y = (*(*prgn).Buffer.add(i as usize)).top;
            (*ppt.add((4 * i + 2) as usize)).x = (*(*prgn).Buffer.add(i as usize)).right;
            (*ppt.add((4 * i + 2) as usize)).y = (*(*prgn).Buffer.add(i as usize)).bottom;
            (*ppt.add((4 * i + 3) as usize)).x = (*(*prgn).Buffer.add(i as usize)).left;
            (*ppt.add((4 * i + 3) as usize)).y = (*(*prgn).Buffer.add(i as usize)).bottom;
            i += 1;
        }

        XFORMOBJ_vInit(core::ptr::addr_of_mut!(xo) as *mut XFORMOBJ , pmx);

        if EXFORMOBJ_bApplyXform(
            core::ptr::addr_of_mut!(xo) as *mut XFORMOBJ ,
            XF_LTOL,
            (*prgn).rdh.nCount * 2,
            ppt as *mut core::ffi::c_void,
            ppt as *mut core::ffi::c_void,
        ) == 0
        {
            ExFreePoolWithTag(ppt as *mut core::ffi::c_void, 0x6e526747u32);
            return 0;
        }

        bResult = REGION_SetPolyPolygonRgn(
            prgn,
            ppt,
            pcPoints,
            (*prgn).rdh.nCount,
            WINDING as INT,
        );

        ExFreePoolWithTag(ppt as *mut core::ffi::c_void, 0x6e526747u32);

        return bResult;
    }
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn REGION_AllocRgnWithHandle(
    nReg: INT,
) -> PREGION {
    let pReg: PREGION = GDIOBJ_AllocateObject(
        GDIObjType_RGN_TYPE as u8,
        core::mem::size_of::<REGION>() as ULONG,
        BASEFLAG_LOOKASIDE,
    ) as PREGION;
    
    if pReg.is_null() {
        return core::ptr::null_mut();
    }

    if nReg == 0 || nReg == 1 {
        (*pReg).Buffer = core::ptr::addr_of_mut!((*pReg).rdh.rcBound);
    } else {
        (*pReg).Buffer = ExAllocatePoolWithTag(
            PagedPool,
            (nReg as usize * core::mem::size_of::<RECT>()) as SIZE_T,
            0x52676e20u32,
        ) as *mut RECTL;
        
        if (*pReg).Buffer.is_null() {
            GDIOBJ_vDeleteObject(core::ptr::addr_of_mut!((*pReg).BaseObject));
            return core::ptr::null_mut();
        }
    }

    (*pReg).rdh.nCount = 0;
    (*pReg).rdh.rcBound.left = 0;
    (*pReg).rdh.rcBound.top = 0;
    (*pReg).rdh.rcBound.right = 0;
    (*pReg).rdh.rcBound.bottom = 0;
    (*pReg).rdh.iType = 1;

    (*pReg).rdh.dwSize = core::mem::size_of::<RGNDATAHEADER>() as DWORD;
    (*pReg).rdh.nCount = nReg as DWORD;
    (*pReg).rdh.nRgnSize = (nReg as usize * core::mem::size_of::<RECT>()) as DWORD;
    (*pReg).prgnattr = core::ptr::addr_of_mut!((*pReg).rgnattr);

    (*pReg).rgnattr.AttrFlags = 0;
    (*pReg).rgnattr.iComplexity = SIMPLEREGION;
    (*pReg).rgnattr.Rect = (*pReg).rdh.rcBound;

    if GDIOBJ_hInsertObject(core::ptr::addr_of_mut!((*pReg).BaseObject), GDI_OBJ_HMGR_POWNED).is_null() {
        GDIOBJ_vFreeObject(core::ptr::addr_of_mut!((*pReg).BaseObject));
        return core::ptr::null_mut();
    }

    pReg
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn REGION_AllocRgnWithHandle(
    nReg: INT,
) -> PREGION {
    let pReg: PREGION = GDIOBJ_AllocateObject(
        GDIObjType_RGN_TYPE as u8,
        core::mem::size_of::<REGION>() as ULONG,
        BASEFLAG_LOOKASIDE,
    ) as PREGION;
    
    if pReg.is_null() {
        return core::ptr::null_mut();
    }

    if nReg == 0 || nReg == 1 {
        (*pReg).Buffer = core::ptr::addr_of_mut!((*pReg).rdh.rcBound);
    } else {
        (*pReg).Buffer = ExAllocatePoolWithTag(
            PagedPool,
            (nReg as usize * core::mem::size_of::<RECT>()) as SIZE_T,
            0x52676e20u32,
        ) as *mut RECTL;
        
        if (*pReg).Buffer.is_null() {
            GDIOBJ_vDeleteObject(core::ptr::addr_of_mut!((*pReg).BaseObject));
            return core::ptr::null_mut();
        }
    }

    (*pReg).rdh.nCount = 0;
    (*pReg).rdh.rcBound.left = 0;
    (*pReg).rdh.rcBound.top = 0;
    (*pReg).rdh.rcBound.right = 0;
    (*pReg).rdh.rcBound.bottom = 0;
    (*pReg).rdh.iType = 1;

    (*pReg).rdh.dwSize = core::mem::size_of::<RGNDATAHEADER>() as DWORD;
    (*pReg).rdh.nCount = nReg as DWORD;
    (*pReg).rdh.nRgnSize = (nReg as usize * core::mem::size_of::<RECT>()) as DWORD;
    (*pReg).prgnattr = core::ptr::addr_of_mut!((*pReg).rgnattr);

    (*pReg).rgnattr.AttrFlags = 0;
    (*pReg).rgnattr.iComplexity = SIMPLEREGION;
    (*pReg).rgnattr.Rect = (*pReg).rdh.rcBound;

    if GDIOBJ_hInsertObject(core::ptr::addr_of_mut!((*pReg).BaseObject), GDI_OBJ_HMGR_POWNED).is_null() {
        GDIOBJ_vFreeObject(core::ptr::addr_of_mut!((*pReg).BaseObject));
        return core::ptr::null_mut();
    }

    pReg
}

#[no_mangle]
pub unsafe extern "system" fn REGION_bAllocRgnAttr(
    prgn: PREGION
) -> u8 {
    let ppi: PPROCESSINFO;
    let prgnattr: PRGN_ATTR;

    ppi = PsGetCurrentProcessWin32Process() as PPROCESSINFO;

    prgnattr = GdiPoolAllocate((*ppi).pPoolRgnAttr) as PRGN_ATTR;
    if prgnattr.is_null() {
        return 0u8;
    }

    *prgnattr = (*prgn).rgnattr;

    (*prgn).prgnattr = prgnattr;
    GDIOBJ_vSetObjectAttr(core::ptr::addr_of_mut!((*prgn).BaseObject), prgnattr as *mut core::ffi::c_void);

    return 1u8;
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn REGION_AllocUserRgnWithHandle(
    nRgn: INT,
) -> PREGION {
    let prgn: PREGION;

    prgn = REGION_AllocRgnWithHandle(nRgn);
    if prgn.is_null() {
        return core::ptr::null_mut();
    }

    if REGION_bAllocRgnAttr(prgn) == 0 {
        NT_ASSERT!(0);
    }

    return prgn;
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn REGION_AllocUserRgnWithHandle(
    nRgn: INT,
) -> PREGION {
    let prgn: PREGION;

    prgn = REGION_AllocRgnWithHandle(nRgn);
    if prgn.is_null() {
        return core::ptr::null_mut();
    }

    if REGION_bAllocRgnAttr(prgn) == 0 {
        NT_ASSERT!(0);
    }

    return prgn;
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn REGION_vSyncRegion(
    prgn: PREGION,
) {
    let prgnattr: PRGN_ATTR;

    prgnattr = (*prgn).prgnattr;
    if (*prgnattr).AttrFlags & ATTR_RGN_DIRTY != 0 {
        if (*prgnattr).iComplexity == NULLREGION {
            (*prgn).rdh.nCount = 0;
            (*prgn).rdh.rcBound.left = 0;
            (*prgn).rdh.rcBound.top = 0;
            (*prgn).rdh.rcBound.right = 0;
            (*prgn).rdh.rcBound.bottom = 0;
            (*prgn).rdh.iType = 1;
        } else if (*prgnattr).iComplexity == SIMPLEREGION {
            REGION_SetRectRgn(
                prgn,
                (*prgnattr).Rect.left,
                (*prgnattr).Rect.top,
                (*prgnattr).Rect.right,
                (*prgnattr).Rect.bottom,
            );
        }
    }

    (*prgnattr).AttrFlags &= !(ATTR_RGN_DIRTY | ATTR_RGN_VALID);
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn REGION_vSyncRegion(
    prgn: PREGION,
) {
    let prgnattr: PRGN_ATTR;

    prgnattr = (*prgn).prgnattr;
    if (*prgnattr).AttrFlags & ATTR_RGN_DIRTY != 0 {
        if (*prgnattr).iComplexity == NULLREGION {
            (*prgn).rdh.nCount = 0;
            (*prgn).rdh.rcBound.left = 0;
            (*prgn).rdh.rcBound.top = 0;
            (*prgn).rdh.rcBound.right = 0;
            (*prgn).rdh.rcBound.bottom = 0;
            (*prgn).rdh.iType = 1;
        } else if (*prgnattr).iComplexity == SIMPLEREGION {
            REGION_SetRectRgn(
                prgn,
                (*prgnattr).Rect.left,
                (*prgnattr).Rect.top,
                (*prgnattr).Rect.right,
                (*prgnattr).Rect.bottom,
            );
        }
    }

    (*prgnattr).AttrFlags &= !(ATTR_RGN_DIRTY | ATTR_RGN_VALID);
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn REGION_LockRgn(
    hrgn: HRGN,
) -> PREGION {
    let prgn: PREGION;

    prgn = GDIOBJ_LockObject(hrgn as HGDIOBJ, GDIObjType_RGN_TYPE as u8) as PREGION;
    if prgn.is_null() {
        return core::ptr::null_mut();
    }

    REGION_vSyncRegion(prgn);
    prgn
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn REGION_LockRgn(
    hrgn: HRGN,
) -> PREGION {
    let prgn: PREGION;

    prgn = GDIOBJ_LockObject(hrgn as HGDIOBJ, GDIObjType_RGN_TYPE as u8) as PREGION;
    if prgn.is_null() {
        return core::ptr::null_mut();
    }

    REGION_vSyncRegion(prgn);
    prgn
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn REGION_UnlockRgn(prgn: PREGION) {
    let prgnattr: PRGN_ATTR;

    NT_ASSERT!(!prgn.is_null());
    NT_ASSERT!(!(*prgn).prgnattr.is_null());

    prgnattr = (*prgn).prgnattr;
    if prgnattr != core::ptr::addr_of_mut!((*prgn).rgnattr) {
        NT_ASSERT!(GreGetObjectOwner((*prgn).BaseObject.hHmgr) == GDI_OBJ_HMGR_POWNED);
        (*prgnattr).iComplexity = REGION_Complexity(prgn) as u32;
        (*prgnattr).Rect.left = (*prgn).rdh.rcBound.left;
        (*prgnattr).Rect.top = (*prgn).rdh.rcBound.top;
        (*prgnattr).Rect.right = (*prgn).rdh.rcBound.right;
        (*prgnattr).Rect.bottom = (*prgn).rdh.rcBound.bottom;
        (*prgnattr).AttrFlags |= ATTR_RGN_VALID;
    }

    GDIOBJ_vUnlockObject(core::ptr::addr_of_mut!((*prgn).BaseObject));
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn REGION_UnlockRgn(prgn: PREGION) {
    let prgnattr: PRGN_ATTR;

    NT_ASSERT!(!prgn.is_null());
    NT_ASSERT!(!(*prgn).prgnattr.is_null());

    prgnattr = (*prgn).prgnattr;
    if prgnattr != core::ptr::addr_of_mut!((*prgn).rgnattr) {
        NT_ASSERT!(GreGetObjectOwner((*prgn).BaseObject.hHmgr) == GDI_OBJ_HMGR_POWNED);
        (*prgnattr).iComplexity = REGION_Complexity(prgn) as u32;
        (*prgnattr).Rect.left = (*prgn).rdh.rcBound.left;
        (*prgnattr).Rect.top = (*prgn).rdh.rcBound.top;
        (*prgnattr).Rect.right = (*prgn).rdh.rcBound.right;
        (*prgnattr).Rect.bottom = (*prgn).rdh.rcBound.bottom;
        (*prgnattr).AttrFlags |= ATTR_RGN_VALID;
    }

    GDIOBJ_vUnlockObject(core::ptr::addr_of_mut!((*prgn).BaseObject));
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntSysCreateRectpRgn(
    LeftRect: INT,
    TopRect: INT,
    RightRect: INT,
    BottomRect: INT,
) -> PREGION {
    let prgn: PREGION = GDIOBJ_AllocateObject(
        GDIObjType_RGN_TYPE as u8,
        core::mem::size_of::<REGION>() as ULONG,
        BASEFLAG_LOOKASIDE,
    ) as PREGION;
    
    if prgn.is_null() {
        return core::ptr::null_mut();
    }
    
    (*prgn).Buffer = core::ptr::addr_of_mut!((*prgn).rdh.rcBound);
    (*prgn).prgnattr = core::ptr::addr_of_mut!((*prgn).rgnattr);
    (*(*prgn).prgnattr).AttrFlags = ATTR_RGN_VALID;
    REGION_SetRectRgn(prgn, LeftRect, TopRect, RightRect, BottomRect);
    
    prgn
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntSysCreateRectpRgn(
    LeftRect: INT,
    TopRect: INT,
    RightRect: INT,
    BottomRect: INT,
) -> PREGION {
    let prgn: PREGION = GDIOBJ_AllocateObject(
        GDIObjType_RGN_TYPE as u8,
        core::mem::size_of::<REGION>() as ULONG,
        BASEFLAG_LOOKASIDE,
    ) as PREGION;
    
    if prgn.is_null() {
        return core::ptr::null_mut();
    }
    
    (*prgn).Buffer = core::ptr::addr_of_mut!((*prgn).rdh.rcBound);
    (*prgn).prgnattr = core::ptr::addr_of_mut!((*prgn).rgnattr);
    (*(*prgn).prgnattr).AttrFlags = ATTR_RGN_VALID;
    REGION_SetRectRgn(prgn, LeftRect, TopRect, RightRect, BottomRect);
    
    prgn
}
#[no_mangle]
pub unsafe extern "system" fn REGION_vCleanup(ObjectBody: PVOID) {
    let pRgn = ObjectBody as PREGION;
    let ppi = PsGetCurrentProcessWin32Process() as PPROCESSINFO;

    if (*pRgn).prgnattr != core::ptr::addr_of_mut!((*pRgn).rgnattr) as *mut _ {
        GdiPoolFree((*ppi).pPoolRgnAttr, (*pRgn).prgnattr as PVOID);
    }

    if !(*pRgn).Buffer.is_null() && (*pRgn).Buffer != core::ptr::addr_of_mut!((*pRgn).rdh.rcBound) as PRECTL {
        ExFreePoolWithTag((*pRgn).Buffer as PVOID, TAG_REGION);
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "fastcall" fn REGION_Delete(pRgn: PREGION) {
    if pRgn == prgnDefault {
        return;
    }

    GDIOBJ_vDeleteObject(core::ptr::addr_of_mut!((*pRgn).BaseObject));
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
unsafe extern "C" fn REGION_Delete(pRgn: PREGION) {
    if pRgn == prgnDefault {
        return;
    }

    GDIOBJ_vDeleteObject(core::ptr::addr_of_mut!((*pRgn).BaseObject));
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntGdiSetRegionOwner(
    hRgn: HRGN,
    OwnerMask: u32,
) -> i32 {
    let prgn: PREGION;
    let prgnattr: PRGN_ATTR;
    let ppi: PPROCESSINFO;

    prgn = REGION_LockRgn(hRgn);
    if prgn.is_null() {
        return 0;
    }

    prgnattr = (*prgn).prgnattr;
    if prgnattr != core::ptr::addr_of_mut!((*prgn).rgnattr) {
        GDIOBJ_vSetObjectAttr(core::ptr::addr_of_mut!((*prgn).BaseObject), core::ptr::null_mut());
        (*prgn).prgnattr = core::ptr::addr_of_mut!((*prgn).rgnattr);
        ppi = PsGetCurrentProcessWin32Process() as PPROCESSINFO;
        GdiPoolFree((*ppi).pPoolRgnAttr, prgnattr as *mut core::ffi::c_void);
    }

    REGION_UnlockRgn(prgn);

    GreSetObjectOwner(hRgn as HGDIOBJ, OwnerMask)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntGdiSetRegionOwner(
    hRgn: HRGN,
    OwnerMask: u32,
) -> i32 {
    let prgn: PREGION;
    let prgnattr: PRGN_ATTR;
    let ppi: PPROCESSINFO;

    prgn = REGION_LockRgn(hRgn);
    if prgn.is_null() {
        return 0;
    }

    prgnattr = (*prgn).prgnattr;
    if prgnattr != core::ptr::addr_of_mut!((*prgn).rgnattr) {
        GDIOBJ_vSetObjectAttr(core::ptr::addr_of_mut!((*prgn).BaseObject), core::ptr::null_mut());
        (*prgn).prgnattr = core::ptr::addr_of_mut!((*prgn).rgnattr);
        ppi = PsGetCurrentProcessWin32Process() as PPROCESSINFO;
        GdiPoolFree((*ppi).pPoolRgnAttr, prgnattr as *mut core::ffi::c_void);
    }

    REGION_UnlockRgn(prgn);

    GreSetObjectOwner(hRgn as HGDIOBJ, OwnerMask)
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntGdiCombineRgn(
    prgnDest: PREGION,
    prgnSrc1: PREGION,
    prgnSrc2: PREGION,
    iCombineMode: INT,
) -> INT {
    let mut Ret: u8 = 1u8;

    if prgnDest.is_null() {
        return ERROR as INT;
    }

    if prgnSrc1.is_null() {
        return ERROR as INT;
    }

    if iCombineMode == RGN_COPY as INT {
        if REGION_CopyRegion(prgnDest, prgnSrc1) == 0 {
            return ERROR as INT;
        }

        return REGION_Complexity(prgnDest);
    }

    if prgnSrc2.is_null() {
        return ERROR as INT;
    }

    match iCombineMode {
        _ if iCombineMode == RGN_AND as INT => {
            Ret = REGION_IntersectRegion(prgnDest, prgnSrc1, prgnSrc2);
        }
        _ if iCombineMode == RGN_OR as INT => {
            Ret = REGION_UnionRegion(prgnDest, prgnSrc1, prgnSrc2);
        }
        _ if iCombineMode == RGN_XOR as INT => {
            Ret = REGION_XorRegion(prgnDest, prgnSrc1, prgnSrc2);
        }
        _ if iCombineMode == RGN_DIFF as INT => {
            Ret = REGION_SubtractRegion(prgnDest, prgnSrc1, prgnSrc2);
        }
        _ => {}
    }

    if Ret != 0 {
        REGION_Complexity(prgnDest)
    } else {
        ERROR as INT
    }
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntGdiCombineRgn(
    prgnDest: PREGION,
    prgnSrc1: PREGION,
    prgnSrc2: PREGION,
    iCombineMode: INT,
) -> INT {
    let mut Ret: u8 = 1u8;

    if prgnDest.is_null() {
        return ERROR as INT;
    }

    if prgnSrc1.is_null() {
        return ERROR as INT;
    }

    if iCombineMode == RGN_COPY as INT {
        if REGION_CopyRegion(prgnDest, prgnSrc1) == 0 {
            return ERROR as INT;
        }

        return REGION_Complexity(prgnDest);
    }

    if prgnSrc2.is_null() {
        return ERROR as INT;
    }

    match iCombineMode {
        _ if iCombineMode == RGN_AND as INT => {
            Ret = REGION_IntersectRegion(prgnDest, prgnSrc1, prgnSrc2);
        }
        _ if iCombineMode == RGN_OR as INT => {
            Ret = REGION_UnionRegion(prgnDest, prgnSrc1, prgnSrc2);
        }
        _ if iCombineMode == RGN_XOR as INT => {
            Ret = REGION_XorRegion(prgnDest, prgnSrc1, prgnSrc2);
        }
        _ if iCombineMode == RGN_DIFF as INT => {
            Ret = REGION_SubtractRegion(prgnDest, prgnSrc1, prgnSrc2);
        }
        _ => {}
    }

    if Ret != 0 {
        REGION_Complexity(prgnDest)
    } else {
        ERROR as INT
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn REGION_GetRgnBox(
    Rgn: PREGION,
    pRect: PRECTL,
) -> INT {
    let mut ret: DWORD;

    if !Rgn.is_null() {
        *pRect = (*Rgn).rdh.rcBound;
        ret = REGION_Complexity(Rgn) as DWORD;

        return ret as INT;
    }
    return 0;
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn REGION_GetRgnBox(
    Rgn: PREGION,
    pRect: PRECTL,
) -> INT {
    let mut ret: DWORD;

    if !Rgn.is_null() {
        *pRect = (*Rgn).rdh.rcBound;
        ret = REGION_Complexity(Rgn) as DWORD;

        return ret as INT;
    }
    return 0;
}
#[no_mangle]
pub unsafe extern "system" fn IntGdiGetRgnBox(
    hRgn: HRGN,
    pRect: PRECTL,
) -> INT {
    let Rgn: PREGION;
    let ret: DWORD;

    Rgn = REGION_LockRgn(hRgn);
    if Rgn.is_null() {
        return ERROR as _;
    }

    ret = REGION_GetRgnBox(Rgn, pRect) as DWORD;
    REGION_UnlockRgn(Rgn);

    return ret as INT;
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn REGION_PtInRegion(
    prgn: PREGION,
    X: i32,
    Y: i32,
) -> BOOL {
    let mut i: u32;
    let r: PRECT;

    if (*prgn).rdh.nCount > 0 
        && (*prgn).rdh.rcBound.right > X 
        && (*prgn).rdh.rcBound.left <= X 
        && (*prgn).rdh.rcBound.bottom > Y 
        && (*prgn).rdh.rcBound.top <= Y 
    {
        r = (*prgn).Buffer as PRECT;
        i = 0;
        while i < (*prgn).rdh.nCount {
            if (*r.offset(i as isize)).right > X 
                && (*r.offset(i as isize)).left <= X 
                && (*r.offset(i as isize)).bottom > Y 
                && (*r.offset(i as isize)).top <= Y 
            {
                return 1;
            }
            i += 1;
        }
    }

    0
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn REGION_PtInRegion(
    prgn: PREGION,
    X: i32,
    Y: i32,
) -> BOOL {
    let mut i: u32;
    let r: PRECT;

    if (*prgn).rdh.nCount > 0 
        && (*prgn).rdh.rcBound.right > X 
        && (*prgn).rdh.rcBound.left <= X 
        && (*prgn).rdh.rcBound.bottom > Y 
        && (*prgn).rdh.rcBound.top <= Y 
    {
        r = (*prgn).Buffer as PRECT;
        i = 0;
        while i < (*prgn).rdh.nCount {
            if (*r.offset(i as isize)).right > X 
                && (*r.offset(i as isize)).left <= X 
                && (*r.offset(i as isize)).bottom > Y 
                && (*r.offset(i as isize)).top <= Y 
            {
                return 1;
            }
            i += 1;
        }
    }

    0
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn REGION_RectInRegion(
    Rgn: PREGION,
    rect: *const RECTL,
) -> BOOL {
    let mut pCurRect: PRECTL;
    let pRectEnd: PRECTL;
    let mut rc: RECT = core::mem::zeroed();

    if (*rect).top > (*rect).bottom {
        rc.top = (*rect).bottom;
        rc.bottom = (*rect).top;
    } else {
        rc.top = (*rect).top;
        rc.bottom = (*rect).bottom;
    }

    if (*rect).right < (*rect).left {
        rc.right = (*rect).left;
        rc.left = (*rect).right;
    } else {
        rc.right = (*rect).right;
        rc.left = (*rect).left;
    }

    if ((*Rgn).rdh.nCount > 0) && 
       ((*Rgn).rdh.rcBound.right > rc.left &&
        (*Rgn).rdh.rcBound.left < rc.right &&
        (*Rgn).rdh.rcBound.bottom > rc.top &&
        (*Rgn).rdh.rcBound.top < rc.bottom) {
        pCurRect = (*Rgn).Buffer;
        pRectEnd = pCurRect.add((*Rgn).rdh.nCount as usize);
        while pCurRect < pRectEnd {
            if (*pCurRect).bottom <= rc.top {
                pCurRect = pCurRect.add(1);
                continue;
            }

            if (*pCurRect).top >= rc.bottom {
                break;
            }

            if (*pCurRect).right <= rc.left {
                pCurRect = pCurRect.add(1);
                continue;
            }

            if (*pCurRect).left >= rc.right {
                pCurRect = pCurRect.add(1);
                continue;
            }

            return 1;
        }
    }

    return 0;
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn REGION_RectInRegion(
    Rgn: PREGION,
    rect: *const RECTL,
) -> BOOL {
    let mut pCurRect: PRECTL;
    let pRectEnd: PRECTL;
    let mut rc: RECT = core::mem::zeroed();

    if (*rect).top > (*rect).bottom {
        rc.top = (*rect).bottom;
        rc.bottom = (*rect).top;
    } else {
        rc.top = (*rect).top;
        rc.bottom = (*rect).bottom;
    }

    if (*rect).right < (*rect).left {
        rc.right = (*rect).left;
        rc.left = (*rect).right;
    } else {
        rc.right = (*rect).right;
        rc.left = (*rect).left;
    }

    if ((*Rgn).rdh.nCount > 0) && 
       ((*Rgn).rdh.rcBound.right > rc.left &&
        (*Rgn).rdh.rcBound.left < rc.right &&
        (*Rgn).rdh.rcBound.bottom > rc.top &&
        (*Rgn).rdh.rcBound.top < rc.bottom) {
        pCurRect = (*Rgn).Buffer;
        pRectEnd = pCurRect.add((*Rgn).rdh.nCount as usize);
        while pCurRect < pRectEnd {
            if (*pCurRect).bottom <= rc.top {
                pCurRect = pCurRect.add(1);
                continue;
            }

            if (*pCurRect).top >= rc.bottom {
                break;
            }

            if (*pCurRect).right <= rc.left {
                pCurRect = pCurRect.add(1);
                continue;
            }

            if (*pCurRect).left >= rc.right {
                pCurRect = pCurRect.add(1);
                continue;
            }

            return 1;
        }
    }

    return 0;
}

#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "fastcall" fn REGION_SetRectRgn(
    rgn: PREGION,
    LeftRect: INT,
    TopRect: INT,
    RightRect: INT,
    BottomRect: INT,
) {
    let mut LeftRect = LeftRect;
    let mut TopRect = TopRect;
    let mut RightRect = RightRect;
    let mut BottomRect = BottomRect;

    if LeftRect > RightRect {
        let tmp = LeftRect;
        LeftRect = RightRect;
        RightRect = tmp;
    }

    if TopRect > BottomRect {
        let tmp = TopRect;
        TopRect = BottomRect;
        BottomRect = tmp;
    }

    if (LeftRect != RightRect) && (TopRect != BottomRect) {
        let firstRect = (*rgn).Buffer;
        (*firstRect).left = LeftRect;
        (*rgn).rdh.rcBound.left = LeftRect;
        (*firstRect).top = TopRect;
        (*rgn).rdh.rcBound.top = TopRect;
        (*firstRect).right = RightRect;
        (*rgn).rdh.rcBound.right = RightRect;
        (*firstRect).bottom = BottomRect;
        (*rgn).rdh.rcBound.bottom = BottomRect;
        (*rgn).rdh.nCount = 1;
        (*rgn).rdh.iType = RDH_RECTANGLES;
    } else {
        (*rgn).rdh.nCount = 0;
        (*rgn).rdh.rcBound.left = 0;
        (*rgn).rdh.rcBound.top = 0;
        (*rgn).rdh.rcBound.right = 0;
        (*rgn).rdh.rcBound.bottom = 0;
        (*rgn).rdh.iType = RDH_RECTANGLES;
    }
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
unsafe extern "C" fn REGION_SetRectRgn(
    rgn: PREGION,
    LeftRect: INT,
    TopRect: INT,
    RightRect: INT,
    BottomRect: INT,
) {
    let mut LeftRect = LeftRect;
    let mut TopRect = TopRect;
    let mut RightRect = RightRect;
    let mut BottomRect = BottomRect;

    if LeftRect > RightRect {
        let tmp = LeftRect;
        LeftRect = RightRect;
        RightRect = tmp;
    }

    if TopRect > BottomRect {
        let tmp = TopRect;
        TopRect = BottomRect;
        BottomRect = tmp;
    }

    if (LeftRect != RightRect) && (TopRect != BottomRect) {
        let firstRect = (*rgn).Buffer;
        (*firstRect).left = LeftRect;
        (*rgn).rdh.rcBound.left = LeftRect;
        (*firstRect).top = TopRect;
        (*rgn).rdh.rcBound.top = TopRect;
        (*firstRect).right = RightRect;
        (*rgn).rdh.rcBound.right = RightRect;
        (*firstRect).bottom = BottomRect;
        (*rgn).rdh.rcBound.bottom = BottomRect;
        (*rgn).rdh.nCount = 1;
        (*rgn).rdh.iType = RDH_RECTANGLES;
    } else {
        (*rgn).rdh.nCount = 0;
        (*rgn).rdh.rcBound.left = 0;
        (*rgn).rdh.rcBound.top = 0;
        (*rgn).rdh.rcBound.right = 0;
        (*rgn).rdh.rcBound.bottom = 0;
        (*rgn).rdh.iType = RDH_RECTANGLES;
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn REGION_bOffsetRgn(prgn: PREGION, cx: i32, cy: i32) -> BOOL { REGION_bOffsetRgn_impl(prgn, cx, cy) as BOOL }
#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn REGION_bOffsetRgn(prgn: PREGION, cx: i32, cy: i32) -> BOOL { REGION_bOffsetRgn_impl(prgn, cx, cy) as BOOL }
unsafe fn REGION_bOffsetRgn_impl(
    prgn: PREGION,
    cx: i32,
    cy: i32,
) -> u8 {
    let mut prcl: PRECTL;
    let mut i: u32;

    if (cx == 0) && (cy == 0) {
        return 1u8;
    }

    if (*prgn).rdh.nCount == 0 {
        return 1u8;
    }

    if (cx > MAX_COORD as INT) || (cx < MIN_COORD as INT) ||
       (cy > MAX_COORD as INT) || (cy < MIN_COORD as INT) {
        return 0u8;
    }

    if cx > 0 {
        if (*prgn).rdh.rcBound.right > (MAX_COORD as INT - cx) {
            return 0u8;
        }
    } else {
        if (*prgn).rdh.rcBound.left < (MIN_COORD - cx) {
            return 0u8;
        }
    }

    if cy > 0 {
        if (*prgn).rdh.rcBound.bottom > (MAX_COORD as INT - cy) {
            return 0u8;
        }
    } else {
        if (*prgn).rdh.rcBound.top < (MIN_COORD - cy) {
            return 0u8;
        }
    }

    prcl = (*prgn).Buffer;
    i = 0;
    while i < (*prgn).rdh.nCount {
        (*prcl.offset(i as isize)).left += cx;
        (*prcl.offset(i as isize)).right += cx;
        (*prcl.offset(i as isize)).top += cy;
        (*prcl.offset(i as isize)).bottom += cy;
        i += 1;
    }

    if (*prgn).Buffer != core::ptr::addr_of_mut!((*prgn).rdh.rcBound) {
        (*prgn).rdh.rcBound.left += cx;
        (*prgn).rdh.rcBound.right += cx;
        (*prgn).rdh.rcBound.top += cy;
        (*prgn).rdh.rcBound.bottom += cy;
    }

    1u8
}

unsafe fn REGION_InsertEdgeInET(
    ET: *mut EDGE_TABLE,
    ETE: *mut EDGE_TABLE_ENTRY,
    scanline: INT,
    SLLBlock: *mut *mut SCANLINE_LISTBLOCK,
    iSLLBlock: *mut INT)
{
    let mut start: *mut EDGE_TABLE_ENTRY;
    let mut prev: *mut EDGE_TABLE_ENTRY;
    let mut pSLL: *mut SCANLINE_LIST;
    let mut pPrevSLL: *mut SCANLINE_LIST;
    let mut tmpSLLBlock: *mut SCANLINE_LISTBLOCK;

    pPrevSLL = core::ptr::addr_of_mut!((*ET).scanlines);
    pSLL = (*pPrevSLL).next;
    while !pSLL.is_null() && (*pSLL).scanline < scanline
    {
        pPrevSLL = pSLL;
        pSLL = (*pSLL).next;
    }

    if pSLL.is_null() || (*pSLL).scanline > scanline
    {
        if *iSLLBlock > SLLSPERBLOCK as INT - 1
        {
            tmpSLLBlock = ExAllocatePoolWithTag(
                PagedPool,
                core::mem::size_of::<SCANLINE_LISTBLOCK>() as SIZE_T,
                TAG_REGION
            ) as *mut SCANLINE_LISTBLOCK;
            if tmpSLLBlock.is_null()
            {
                return;
            }

            (**SLLBlock).next = tmpSLLBlock;
            (*tmpSLLBlock).next = core::ptr::null_mut();
            *SLLBlock = tmpSLLBlock;
            *iSLLBlock = 0;
        }

        pSLL = core::ptr::addr_of_mut!((**SLLBlock).SLLs[*iSLLBlock as usize]);
        *iSLLBlock += 1;

        (*pSLL).next = (*pPrevSLL).next;
        (*pSLL).edgelist = core::ptr::null_mut();
        (*pPrevSLL).next = pSLL;
    }

    (*pSLL).scanline = scanline;

    prev = core::ptr::null_mut();
    start = (*pSLL).edgelist;
    while !start.is_null() && (*start).bres.minor_axis < (*ETE).bres.minor_axis
    {
        prev = start;
        start = (*start).next;
    }

    (*ETE).next = start;

    if !prev.is_null() {
        (*prev).next = ETE;
    } else {
        (*pSLL).edgelist = ETE;
    }
}

unsafe fn REGION_loadAET(
    mut AET: *mut EDGE_TABLE_ENTRY,
    mut ETEs: *mut EDGE_TABLE_ENTRY)
{
    let mut pPrevAET: *mut EDGE_TABLE_ENTRY;
    let mut tmp: *mut EDGE_TABLE_ENTRY;

    pPrevAET = AET;
    AET = (*AET).next;
    while !ETEs.is_null()
    {
        while !AET.is_null() && ((*AET).bres.minor_axis < (*ETEs).bres.minor_axis)
        {
            pPrevAET = AET;
            AET = (*AET).next;
        }

        tmp = (*ETEs).next;
        (*ETEs).next = AET;
        if !AET.is_null() {
            (*AET).back = ETEs;
        }

        (*ETEs).back = pPrevAET;
        (*pPrevAET).next = ETEs;
        pPrevAET = ETEs;

        ETEs = tmp;
    }
}

unsafe fn REGION_computeWAET(
    AET: *mut EDGE_TABLE_ENTRY)
{
    let mut pWETE: *mut EDGE_TABLE_ENTRY;
    let mut inside: INT = 1;
    let mut isInside: INT = 0;

    (*AET).nextWETE = core::ptr::null_mut();
    pWETE = AET;
    let mut AET = (*AET).next;
    while !AET.is_null()
    {
        if (*AET).ClockWise != 0
        {
            isInside += 1;
        }
        else
        {
            isInside -= 1;
        }

        if (inside == 0 && isInside == 0) ||
           (inside != 0 && isInside != 0)
        {
            (*pWETE).nextWETE = AET;
            pWETE = AET;
            inside = if inside != 0 { 0 } else { 1 };
        }
        AET = (*AET).next;
    }

    (*pWETE).nextWETE = core::ptr::null_mut();
}

unsafe fn REGION_InsertionSort(
    mut AET: *mut EDGE_TABLE_ENTRY
) -> BOOLEAN {
    let mut pETEchase: *mut EDGE_TABLE_ENTRY;
    let mut pETEinsert: *mut EDGE_TABLE_ENTRY;
    let mut pETEchaseBackTMP: *mut EDGE_TABLE_ENTRY;
    let mut changed: BOOLEAN = 0u8;

    AET = (*AET).next;
    while !AET.is_null() {
        pETEinsert = AET;
        pETEchase = AET;
        while (*(*pETEchase).back).bres.minor_axis > (*AET).bres.minor_axis {
            pETEchase = (*pETEchase).back;
        }

        AET = (*AET).next;
        if pETEchase != pETEinsert {
            pETEchaseBackTMP = (*pETEchase).back;
            (*(*pETEinsert).back).next = AET;
            if !AET.is_null() {
                (*AET).back = (*pETEinsert).back;
            }

            (*pETEinsert).next = pETEchase;
            (*(*pETEchase).back).next = pETEinsert;
            (*pETEchase).back = pETEinsert;
            (*pETEinsert).back = pETEchaseBackTMP;
            changed = 1u8;
        }
    }

    changed
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn REGION_FreeStorage(
    pSLLBlock: *mut SCANLINE_LISTBLOCK)
{
    let mut pSLLBlock = pSLLBlock;
    let mut tmpSLLBlock: *mut SCANLINE_LISTBLOCK;

    while !pSLLBlock.is_null()
    {
        tmpSLLBlock = (*pSLLBlock).next;
        ExFreePoolWithTag(pSLLBlock as *mut core::ffi::c_void, TAG_REGION);
        pSLLBlock = tmpSLLBlock;
    }
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn REGION_FreeStorage(
    pSLLBlock: *mut SCANLINE_LISTBLOCK)
{
    let mut pSLLBlock = pSLLBlock;
    let mut tmpSLLBlock: *mut SCANLINE_LISTBLOCK;

    while !pSLLBlock.is_null()
    {
        tmpSLLBlock = (*pSLLBlock).next;
        ExFreePoolWithTag(pSLLBlock as *mut core::ffi::c_void, TAG_REGION);
        pSLLBlock = tmpSLLBlock;
    }
}

unsafe fn REGION_PtsToRegion(
    mut numFullPtBlocks: INT,
    iCurPtBlock: INT,
    FirstPtBlock: *mut POINTBLOCK,
    reg: PREGION,
) -> INT {
    let mut rects: *mut RECTL;
    let mut pts: *mut POINT;
    let mut CurPtBlock: *mut POINTBLOCK;
    let mut i: INT;
    let extents: *mut RECTL;
    let temp: *mut RECTL;
    let mut numRects: INT;

    extents = core::ptr::addr_of_mut!((*reg).rdh.rcBound);

    numRects = ((numFullPtBlocks * NUMPTSTOBUFFER as INT) + iCurPtBlock) >> 1;

    if numRects == 0 {
        numRects = 1;
    }

    temp = ExAllocatePoolWithTag(PagedPool, (numRects * core::mem::size_of::<RECT>() as INT) as SIZE_T, TAG_REGION) as *mut RECTL;
    if temp.is_null() {
        return 0;
    }

    if !(*reg).Buffer.is_null() {
        let mut xDest = temp;
        let mut xSrc = (*reg).Buffer;
        let mut xRects = (*reg).rdh.nCount as UINT;
        while xRects > 0 {
            *xDest = *xSrc;
            xDest = xDest.offset(1);
            xSrc = xSrc.offset(1);
            xRects -= 1;
        }
        if (*reg).Buffer != core::ptr::addr_of_mut!((*reg).rdh.rcBound) {
            ExFreePoolWithTag((*reg).Buffer as *mut core::ffi::c_void, TAG_REGION);
        }
    }
    (*reg).Buffer = temp;

    (*reg).rdh.nCount = numRects as u32;
    CurPtBlock = FirstPtBlock;
    rects = (*reg).Buffer.offset(-1);
    numRects = 0;
    (*extents).left = LARGE_COORDINATE;
    (*extents).right = SMALL_COORDINATE;

    while numFullPtBlocks >= 0 {
        i = NUMPTSTOBUFFER as INT >> 1;
        if numFullPtBlocks == 0 {
            i = iCurPtBlock >> 1;
        }

        pts = (*CurPtBlock).pts.as_mut_ptr();
        while i > 0 {
            if (*pts).x == (*pts.offset(1)).x {
                pts = pts.offset(2);
                i -= 1;
                continue;
            }

            if (numRects != 0 && (*pts).x == (*rects).left) &&
                ((*pts).y == (*rects).bottom) &&
                ((*pts.offset(1)).x == (*rects).right) &&
                ((numRects == 1) || ((*rects.offset(-1)).top != (*rects).top)) &&
                (i != 0 && (*pts.offset(2)).y > (*pts.offset(1)).y) {
                (*rects).bottom = (*pts.offset(1)).y + 1;
                pts = pts.offset(2);
                i -= 1;
                continue;
            }

            numRects += 1;
            rects = rects.offset(1);
            (*rects).left = (*pts).x;
            (*rects).top = (*pts).y;
            (*rects).right = (*pts.offset(1)).x;
            (*rects).bottom = (*pts.offset(1)).y + 1;

            if (*rects).left < (*extents).left {
                (*extents).left = (*rects).left;
            }
            if (*rects).right > (*extents).right {
                (*extents).right = (*rects).right;
            }

            pts = pts.offset(2);
            i -= 1;
        }

        CurPtBlock = (*CurPtBlock).next;
        numFullPtBlocks -= 1;
    }

    if numRects != 0 {
        (*extents).top = (*(*reg).Buffer).top;
        (*extents).bottom = (*rects).bottom;
    } else {
        (*extents).left = 0;
        (*extents).top = 0;
        (*extents).right = 0;
        (*extents).bottom = 0;
    }

    (*reg).rdh.nCount = numRects as u32;

    1
}

unsafe fn REGION_CreateETandAET(
    Count: *const ULONG,
    nbpolygons: INT,
    pts: *const POINT,
    ET: *mut EDGE_TABLE,
    AET: *mut EDGE_TABLE_ENTRY,
    pETEs: *mut EDGE_TABLE_ENTRY,
    mut pSLLBlock: *mut SCANLINE_LISTBLOCK)
{
    let mut top: *const POINT;
    let mut bottom: *const POINT;
    let mut PrevPt: *const POINT;
    let mut CurrPt: *const POINT;
    let mut EndPt: *const POINT;
    let mut poly: INT;
    let mut count: INT;
    let mut iSLLBlock: INT = 0;
    let mut dy: INT;
    let mut pts_iter = pts;
    let mut pETEs_iter = pETEs;

    (*AET).next = core::ptr::null_mut();
    (*AET).back = core::ptr::null_mut();
    (*AET).nextWETE = core::ptr::null_mut();
    (*AET).bres.minor_axis = SMALL_COORDINATE;

    (*ET).scanlines.next = core::ptr::null_mut();
    (*ET).ymax = SMALL_COORDINATE;
    (*ET).ymin = LARGE_COORDINATE;
    (*pSLLBlock).next = core::ptr::null_mut();

    EndPt = pts.offset(-1);
    poly = 0;
    while poly < nbpolygons {
        count = *Count.offset(poly as isize) as INT;
        EndPt = EndPt.offset(count as isize);
        if count < 2 {
            poly += 1;
            continue;
        }

        PrevPt = EndPt;

        while count != 0 {
            count -= 1;
            CurrPt = pts_iter;
            pts_iter = pts_iter.offset(1);

            if (*PrevPt).y > (*CurrPt).y {
                bottom = PrevPt;
                top = CurrPt;
                (*pETEs_iter).ClockWise = 0;
            } else {
                bottom = CurrPt;
                top = PrevPt;
                (*pETEs_iter).ClockWise = 1;
            }

            if (*bottom).y != (*top).y {
                (*pETEs_iter).ymax = (*bottom).y - 1;

                dy = (*bottom).y - (*top).y;
                {
                    let dmaj = dy;
                    let min1 = (*top).x;
                    let min2 = (*bottom).x;
                    let bres = core::ptr::addr_of_mut!((*pETEs_iter).bres);
                    let mut dx: INT;
                    if dmaj != 0 {
                        (*bres).minor_axis = min1;
                        dx = min2 - (*bres).minor_axis;
                        if dx < 0 {
                            (*bres).m = dx / dmaj;
                            (*bres).m1 = (*bres).m - 1;
                            (*bres).incr1 = -2 * dx + 2 * dmaj * (*bres).m1;
                            (*bres).incr2 = -2 * dx + 2 * dmaj * (*bres).m;
                            (*bres).d = 2 * (*bres).m * dmaj - 2 * dx - 2 * dmaj;
                        } else {
                            (*bres).m = dx / dmaj;
                            (*bres).m1 = (*bres).m + 1;
                            (*bres).incr1 = 2 * dx - 2 * dmaj * (*bres).m1;
                            (*bres).incr2 = 2 * dx - 2 * dmaj * (*bres).m;
                            (*bres).d = -2 * (*bres).m * dmaj + 2 * dx;
                        }
                    }
                }

                REGION_InsertEdgeInET(ET,
                                      pETEs_iter,
                                      (*top).y,
                                      core::ptr::addr_of_mut!(pSLLBlock),
                                      core::ptr::addr_of_mut!(iSLLBlock));

                if (*PrevPt).y > (*ET).ymax {
                    (*ET).ymax = (*PrevPt).y;
                }
                if (*PrevPt).y < (*ET).ymin {
                    (*ET).ymin = (*PrevPt).y;
                }
                pETEs_iter = pETEs_iter.offset(1);
            }

            PrevPt = CurrPt;
        }
        poly += 1;
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "fastcall" fn REGION_SetPolyPolygonRgn(
    prgn: PREGION,
    ppt: *const POINT,
    pcPoints: *const ULONG,
    cPolygons: ULONG,
    iMode: INT,
) -> BOOL {
    let mut pAET: *mut EDGE_TABLE_ENTRY;
    let mut y: INT;
    let mut iPts: INT = 0;
    let mut pWETE: *mut EDGE_TABLE_ENTRY;
    let mut pSLL: *mut SCANLINE_LIST;
    let mut pts: *mut POINT;
    let mut pPrevAET: *mut EDGE_TABLE_ENTRY;
    let mut ET: EDGE_TABLE = core::mem::zeroed();
    let mut AET: EDGE_TABLE_ENTRY = core::mem::zeroed();
    let mut pETEs: *mut EDGE_TABLE_ENTRY;
    let mut SLLBlock: SCANLINE_LISTBLOCK = core::mem::zeroed();
    let mut fixWAET: INT = 0;
    let mut FirstPtBlock: POINTBLOCK = core::mem::zeroed();
    let mut curPtBlock: *mut POINTBLOCK;
    let mut tmpPtBlock: *mut POINTBLOCK;
    let mut numFullPtBlocks: UINT = 0;
    let mut poly: UINT;
    let mut total: UINT;
    let mut bResult: BOOL = 0;

    if (iMode != ALTERNATE as INT) && (iMode != WINDING as INT) {
        return 0;
    }

    if ((cPolygons == 1) && (((*pcPoints.offset(0) == 4) ||
         (((*pcPoints.offset(0) == 5) && ((*ppt.offset(4)).x == (*ppt.offset(0)).x) && ((*ppt.offset(4)).y == (*ppt.offset(0)).y)))) &&
        ((((*ppt.offset(0)).y == (*ppt.offset(1)).y) &&
          ((*ppt.offset(1)).x == (*ppt.offset(2)).x) &&
          ((*ppt.offset(2)).y == (*ppt.offset(3)).y) &&
          ((*ppt.offset(3)).x == (*ppt.offset(0)).x)) ||
         (((*ppt.offset(0)).x == (*ppt.offset(1)).x) &&
          ((*ppt.offset(1)).y == (*ppt.offset(2)).y) &&
          ((*ppt.offset(2)).x == (*ppt.offset(3)).x) &&
          ((*ppt.offset(3)).y == (*ppt.offset(0)).y)))))
    {
        REGION_SetRectRgn(prgn,
                          core::cmp::min((*ppt.offset(0)).x, (*ppt.offset(2)).x),
                          core::cmp::min((*ppt.offset(0)).y, (*ppt.offset(2)).y),
                          core::cmp::max((*ppt.offset(0)).x, (*ppt.offset(2)).x),
                          core::cmp::max((*ppt.offset(0)).y, (*ppt.offset(2)).y));
        return 1;
    }

    'Cleanup: {
    poly = 0;
    total = 0;
    while poly < cPolygons {
        total += *pcPoints.offset(poly as isize);
        poly += 1;
    }

    pETEs = ExAllocatePoolWithTag(PagedPool,
                                  (core::mem::size_of::<EDGE_TABLE_ENTRY>() * total as usize) as SIZE_T,
                                  TAG_REGION) as *mut EDGE_TABLE_ENTRY;
    if pETEs.is_null() {
        return 0;
    }

    pts = FirstPtBlock.pts.as_mut_ptr();
    REGION_CreateETandAET(pcPoints, cPolygons as INT, ppt, &mut ET, &mut AET, pETEs, &mut SLLBlock);
    pSLL = ET.scanlines.next;
    curPtBlock = &mut FirstPtBlock;

    if iMode != WINDING as INT {
        y = ET.ymin;
        while y < ET.ymax {
            if !pSLL.is_null() && y == (*pSLL).scanline {
                REGION_loadAET(&mut AET, (*pSLL).edgelist);
                pSLL = (*pSLL).next;
            }
            pPrevAET = &mut AET;
            pAET = AET.next;

            while !pAET.is_null() {
                (*pts).x = (*pAET).bres.minor_axis;
                (*pts).y = y;
                pts = pts.offset(1);
                iPts += 1;

                if iPts == NUMPTSTOBUFFER as INT {
                    tmpPtBlock = ExAllocatePoolWithTag(PagedPool,
                                                       core::mem::size_of::<POINTBLOCK>() as SIZE_T,
                                                       TAG_REGION) as *mut POINTBLOCK;
                    if tmpPtBlock.is_null() {
                        bResult = 0;
                        break 'Cleanup;
                    }

                    (*curPtBlock).next = tmpPtBlock;
                    curPtBlock = tmpPtBlock;
                    pts = (*curPtBlock).pts.as_mut_ptr();
                    numFullPtBlocks += 1;
                    iPts = 0;
                }

                if (*pAET).ymax == y {
                    (*pPrevAET).next = (*pAET).next;
                    pAET = (*pPrevAET).next;
                    if !pAET.is_null() {
                        (*pAET).back = pPrevAET;
                    }
                } else {
                    if (*pAET).bres.m1 > 0 {
                        if (*pAET).bres.d > 0 {
                            (*pAET).bres.minor_axis += (*pAET).bres.m1;
                            (*pAET).bres.d += (*pAET).bres.incr1;
                        } else {
                            (*pAET).bres.minor_axis += (*pAET).bres.m;
                            (*pAET).bres.d += (*pAET).bres.incr2;
                        }
                    } else {
                        if (*pAET).bres.d >= 0 {
                            (*pAET).bres.minor_axis += (*pAET).bres.m1;
                            (*pAET).bres.d += (*pAET).bres.incr1;
                        } else {
                            (*pAET).bres.minor_axis += (*pAET).bres.m;
                            (*pAET).bres.d += (*pAET).bres.incr2;
                        }
                    }
                    pPrevAET = pAET;
                    pAET = (*pAET).next;
                }
            }

            REGION_InsertionSort(&mut AET);
            y += 1;
        }
    } else {
        y = ET.ymin;
        while y < ET.ymax {
            if !pSLL.is_null() && y == (*pSLL).scanline {
                REGION_loadAET(&mut AET, (*pSLL).edgelist);
                REGION_computeWAET(&mut AET);
                pSLL = (*pSLL).next;
            }

            pPrevAET = &mut AET;
            pAET = AET.next;
            pWETE = pAET;

            while !pAET.is_null() {
                if pWETE == pAET {
                    (*pts).x = (*pAET).bres.minor_axis;
                    (*pts).y = y;
                    pts = pts.offset(1);
                    iPts += 1;

                    if iPts == NUMPTSTOBUFFER as INT {
                        tmpPtBlock = ExAllocatePoolWithTag(PagedPool,
                                                           core::mem::size_of::<POINTBLOCK>() as SIZE_T,
                                                           TAG_REGION) as *mut POINTBLOCK;
                        if tmpPtBlock.is_null() {
                            bResult = 0;
                            break 'Cleanup;
                        }
                        (*curPtBlock).next = tmpPtBlock;
                        curPtBlock = tmpPtBlock;
                        pts = (*curPtBlock).pts.as_mut_ptr();
                        numFullPtBlocks += 1;
                        iPts = 0;
                    }

                    pWETE = (*pWETE).nextWETE;
                }

                if (*pAET).ymax == y {
                    (*pPrevAET).next = (*pAET).next;
                    pAET = (*pPrevAET).next;
                    fixWAET = 1;
                    if !pAET.is_null() {
                        (*pAET).back = pPrevAET;
                    }
                } else {
                    if (*pAET).bres.m1 > 0 {
                        if (*pAET).bres.d > 0 {
                            (*pAET).bres.minor_axis += (*pAET).bres.m1;
                            (*pAET).bres.d += (*pAET).bres.incr1;
                        } else {
                            (*pAET).bres.minor_axis += (*pAET).bres.m;
                            (*pAET).bres.d += (*pAET).bres.incr2;
                        }
                    } else {
                        if (*pAET).bres.d >= 0 {
                            (*pAET).bres.minor_axis += (*pAET).bres.m1;
                            (*pAET).bres.d += (*pAET).bres.incr1;
                        } else {
                            (*pAET).bres.minor_axis += (*pAET).bres.m;
                            (*pAET).bres.d += (*pAET).bres.incr2;
                        }
                    }
                    pPrevAET = pAET;
                    pAET = (*pAET).next;
                }
            }

            if REGION_InsertionSort(&mut AET) != 0 || fixWAET != 0 {
                REGION_computeWAET(&mut AET);
                fixWAET = 0;
            }
            y += 1;
        }
    }

    REGION_PtsToRegion(numFullPtBlocks as INT, iPts, &mut FirstPtBlock, prgn);
    bResult = 1;
    }

    {
        REGION_FreeStorage(SLLBlock.next);

        curPtBlock = FirstPtBlock.next;
        while numFullPtBlocks > 0 {
            tmpPtBlock = (*curPtBlock).next;
            ExFreePoolWithTag(curPtBlock as *mut core::ffi::c_void, TAG_REGION);
            curPtBlock = tmpPtBlock;
            numFullPtBlocks -= 1;
        }

        ExFreePoolWithTag(pETEs as *mut core::ffi::c_void, TAG_REGION);
    }

    bResult
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
unsafe extern "C" fn REGION_SetPolyPolygonRgn(
    prgn: PREGION,
    ppt: *const POINT,
    pcPoints: *const ULONG,
    cPolygons: ULONG,
    iMode: INT,
) -> BOOL {
    let mut pAET: *mut EDGE_TABLE_ENTRY;
    let mut y: INT;
    let mut iPts: INT = 0;
    let mut pWETE: *mut EDGE_TABLE_ENTRY;
    let mut pSLL: *mut SCANLINE_LIST;
    let mut pts: *mut POINT;
    let mut pPrevAET: *mut EDGE_TABLE_ENTRY;
    let mut ET: EDGE_TABLE = core::mem::zeroed();
    let mut AET: EDGE_TABLE_ENTRY = core::mem::zeroed();
    let mut pETEs: *mut EDGE_TABLE_ENTRY;
    let mut SLLBlock: SCANLINE_LISTBLOCK = core::mem::zeroed();
    let mut fixWAET: INT = 0;
    let mut FirstPtBlock: POINTBLOCK = core::mem::zeroed();
    let mut curPtBlock: *mut POINTBLOCK;
    let mut tmpPtBlock: *mut POINTBLOCK;
    let mut numFullPtBlocks: UINT = 0;
    let mut poly: UINT;
    let mut total: UINT;
    let mut bResult: BOOL = 0;

    if (iMode != ALTERNATE as INT) && (iMode != WINDING as INT) {
        return 0;
    }

    if (cPolygons == 1) && (((*pcPoints.offset(0) == 4) ||
         (((*pcPoints.offset(0) == 5) && ((*ppt.offset(4)).x == (*ppt.offset(0)).x) && ((*ppt.offset(4)).y == (*ppt.offset(0)).y)))) &&
        ((((*ppt.offset(0)).y == (*ppt.offset(1)).y) &&
          ((*ppt.offset(1)).x == (*ppt.offset(2)).x) &&
          ((*ppt.offset(2)).y == (*ppt.offset(3)).y) &&
          ((*ppt.offset(3)).x == (*ppt.offset(0)).x)) ||
         (((*ppt.offset(0)).x == (*ppt.offset(1)).x) &&
          ((*ppt.offset(1)).y == (*ppt.offset(2)).y) &&
          ((*ppt.offset(2)).x == (*ppt.offset(3)).x) &&
          ((*ppt.offset(3)).y == (*ppt.offset(0)).y))))
    {
        REGION_SetRectRgn(prgn,
                          core::cmp::min((*ppt.offset(0)).x, (*ppt.offset(2)).x),
                          core::cmp::min((*ppt.offset(0)).y, (*ppt.offset(2)).y),
                          core::cmp::max((*ppt.offset(0)).x, (*ppt.offset(2)).x),
                          core::cmp::max((*ppt.offset(0)).y, (*ppt.offset(2)).y));
        return 1;
    }

    'Cleanup: {
    poly = 0;
    total = 0;
    while poly < cPolygons {
        total += *pcPoints.offset(poly as isize);
        poly += 1;
    }

    pETEs = ExAllocatePoolWithTag(PagedPool,
                                  (core::mem::size_of::<EDGE_TABLE_ENTRY>() * total as usize) as SIZE_T,
                                  TAG_REGION) as *mut EDGE_TABLE_ENTRY;
    if pETEs.is_null() {
        return 0;
    }

    pts = FirstPtBlock.pts.as_mut_ptr();
    REGION_CreateETandAET(pcPoints, cPolygons as INT, ppt, &mut ET, &mut AET, pETEs, &mut SLLBlock);
    pSLL = ET.scanlines.next;
    curPtBlock = &mut FirstPtBlock;

    if iMode != WINDING as INT {
        y = ET.ymin;
        while y < ET.ymax {
            if !pSLL.is_null() && y == (*pSLL).scanline {
                REGION_loadAET(&mut AET, (*pSLL).edgelist);
                pSLL = (*pSLL).next;
            }
            pPrevAET = &mut AET;
            pAET = AET.next;

            while !pAET.is_null() {
                (*pts).x = (*pAET).bres.minor_axis;
                (*pts).y = y;
                pts = pts.offset(1);
                iPts += 1;

                if iPts == NUMPTSTOBUFFER as INT {
                    tmpPtBlock = ExAllocatePoolWithTag(PagedPool,
                                                       core::mem::size_of::<POINTBLOCK>() as SIZE_T,
                                                       TAG_REGION) as *mut POINTBLOCK;
                    if tmpPtBlock.is_null() {
                        bResult = 0;
                        break 'Cleanup;
                    }

                    (*curPtBlock).next = tmpPtBlock;
                    curPtBlock = tmpPtBlock;
                    pts = (*curPtBlock).pts.as_mut_ptr();
                    numFullPtBlocks += 1;
                    iPts = 0;
                }

                if (*pAET).ymax == y {
                    (*pPrevAET).next = (*pAET).next;
                    pAET = (*pPrevAET).next;
                    if !pAET.is_null() {
                        (*pAET).back = pPrevAET;
                    }
                } else {
                    if (*pAET).bres.m1 > 0 {
                        if (*pAET).bres.d > 0 {
                            (*pAET).bres.minor_axis += (*pAET).bres.m1;
                            (*pAET).bres.d += (*pAET).bres.incr1;
                        } else {
                            (*pAET).bres.minor_axis += (*pAET).bres.m;
                            (*pAET).bres.d += (*pAET).bres.incr2;
                        }
                    } else {
                        if (*pAET).bres.d >= 0 {
                            (*pAET).bres.minor_axis += (*pAET).bres.m1;
                            (*pAET).bres.d += (*pAET).bres.incr1;
                        } else {
                            (*pAET).bres.minor_axis += (*pAET).bres.m;
                            (*pAET).bres.d += (*pAET).bres.incr2;
                        }
                    }
                    pPrevAET = pAET;
                    pAET = (*pAET).next;
                }
            }

            REGION_InsertionSort(&mut AET);
            y += 1;
        }
    } else {
        y = ET.ymin;
        while y < ET.ymax {
            if !pSLL.is_null() && y == (*pSLL).scanline {
                REGION_loadAET(&mut AET, (*pSLL).edgelist);
                REGION_computeWAET(&mut AET);
                pSLL = (*pSLL).next;
            }

            pPrevAET = &mut AET;
            pAET = AET.next;
            pWETE = pAET;

            while !pAET.is_null() {
                if pWETE == pAET {
                    (*pts).x = (*pAET).bres.minor_axis;
                    (*pts).y = y;
                    pts = pts.offset(1);
                    iPts += 1;

                    if iPts == NUMPTSTOBUFFER as INT {
                        tmpPtBlock = ExAllocatePoolWithTag(PagedPool,
                                                           core::mem::size_of::<POINTBLOCK>() as SIZE_T,
                                                           TAG_REGION) as *mut POINTBLOCK;
                        if tmpPtBlock.is_null() {
                            bResult = 0;
                            break 'Cleanup;
                        }
                        (*curPtBlock).next = tmpPtBlock;
                        curPtBlock = tmpPtBlock;
                        pts = (*curPtBlock).pts.as_mut_ptr();
                        numFullPtBlocks += 1;
                        iPts = 0;
                    }

                    pWETE = (*pWETE).nextWETE;
                }

                if (*pAET).ymax == y {
                    (*pPrevAET).next = (*pAET).next;
                    pAET = (*pPrevAET).next;
                    fixWAET = 1;
                    if !pAET.is_null() {
                        (*pAET).back = pPrevAET;
                    }
                } else {
                    if (*pAET).bres.m1 > 0 {
                        if (*pAET).bres.d > 0 {
                            (*pAET).bres.minor_axis += (*pAET).bres.m1;
                            (*pAET).bres.d += (*pAET).bres.incr1;
                        } else {
                            (*pAET).bres.minor_axis += (*pAET).bres.m;
                            (*pAET).bres.d += (*pAET).bres.incr2;
                        }
                    } else {
                        if (*pAET).bres.d >= 0 {
                            (*pAET).bres.minor_axis += (*pAET).bres.m1;
                            (*pAET).bres.d += (*pAET).bres.incr1;
                        } else {
                            (*pAET).bres.minor_axis += (*pAET).bres.m;
                            (*pAET).bres.d += (*pAET).bres.incr2;
                        }
                    }
                    pPrevAET = pAET;
                    pAET = (*pAET).next;
                }
            }

            if REGION_InsertionSort(&mut AET) != 0 || fixWAET != 0 {
                REGION_computeWAET(&mut AET);
                fixWAET = 0;
            }
            y += 1;
        }
    }

    REGION_PtsToRegion(numFullPtBlocks as INT, iPts, &mut FirstPtBlock, prgn);
    bResult = 1;
    }

    {
        REGION_FreeStorage(SLLBlock.next);

        curPtBlock = FirstPtBlock.next;
        while numFullPtBlocks > 0 {
            tmpPtBlock = (*curPtBlock).next;
            ExFreePoolWithTag(curPtBlock as *mut core::ffi::c_void, TAG_REGION);
            curPtBlock = tmpPtBlock;
            numFullPtBlocks -= 1;
        }

        ExFreePoolWithTag(pETEs as *mut core::ffi::c_void, TAG_REGION);
    }

    bResult
}
#[no_mangle]
pub unsafe extern "system" fn GreCreatePolyPolygonRgn(
    ppt: *const POINT,
    pcPoints: *const ULONG,
    cPolygons: ULONG,
    iMode: INT,
) -> HRGN {
    let prgn: PREGION;
    let hrgn: HRGN;

    prgn = REGION_AllocUserRgnWithHandle(0);
    if prgn.is_null() {
        EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return core::ptr::null_mut();
    }

    if REGION_SetPolyPolygonRgn(prgn, ppt, pcPoints, cPolygons, iMode) != 0 {
        hrgn = (*prgn).BaseObject.hHmgr as HRGN;
        REGION_UnlockRgn(prgn);
    } else {
        REGION_Delete(prgn);
        hrgn = core::ptr::null_mut();
    }

    return hrgn;
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntRectInRegion(
    hRgn: HRGN,
    rc: LPRECTL,
) -> BOOL {
    let Rgn: PREGION;
    let Ret: BOOL;

    Rgn = REGION_LockRgn(hRgn);
    if Rgn.is_null() {
        return ERROR as _;
    }

    Ret = REGION_RectInRegion(Rgn, rc);
    REGION_UnlockRgn(Rgn);
    return Ret;
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntRectInRegion(
    hRgn: HRGN,
    rc: LPRECTL,
) -> BOOL {
    let Rgn: PREGION;
    let Ret: BOOL;

    Rgn = REGION_LockRgn(hRgn);
    if Rgn.is_null() {
        return ERROR as _;
    }

    Ret = REGION_RectInRegion(Rgn, rc);
    REGION_UnlockRgn(Rgn);
    return Ret;
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiCombineRgn(
    hrgnDst: HRGN,
    hrgnSrc1: HRGN,
    hrgnSrc2: HRGN,
    iMode: INT,
) -> INT {
    let mut ahrgn: [HRGN; 3] = [core::ptr::null_mut(); 3];
    let mut aprgn: [PREGION; 3] = [core::ptr::null_mut(); 3];
    let iResult: INT;

    if (iMode < RGN_AND as INT) || (iMode > RGN_COPY as INT) {
        return ERROR as INT;
    }

    if hrgnDst.is_null() ||
       hrgnSrc1.is_null() ||
       ((iMode != RGN_COPY as INT) && hrgnSrc2.is_null()) {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return ERROR as INT;
    }

    ahrgn[0] = hrgnDst;
    ahrgn[1] = hrgnSrc1;
    ahrgn[2] = if iMode != RGN_COPY as INT { hrgnSrc2 } else { core::ptr::null_mut() };
    if GDIOBJ_bLockMultipleObjects(
        3,
        ahrgn.as_mut_ptr() as *mut HGDIOBJ,
        aprgn.as_mut_ptr() as *mut *mut core::ffi::c_void,
        GDIObjType_RGN_TYPE as u8,
    ) == 0 {
        return ERROR as INT;
    }

    REGION_vSyncRegion(aprgn[0]);
    if aprgn[1] != aprgn[0] {
        REGION_vSyncRegion(aprgn[1]);
    }
    if !aprgn[2].is_null() && (aprgn[2] != aprgn[0]) && (aprgn[2] != aprgn[1]) {
        REGION_vSyncRegion(aprgn[2]);
    }

    iResult = IntGdiCombineRgn(aprgn[0], aprgn[1], aprgn[2], iMode);

    REGION_UnlockRgn(aprgn[0]);
    REGION_UnlockRgn(aprgn[1]);
    if !aprgn[2].is_null() {
        REGION_UnlockRgn(aprgn[2]);
    }

    iResult
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiCreateEllipticRgn(
    Left: INT,
    Top: INT,
    Right: INT,
    Bottom: INT,
) -> HRGN {
    NtGdiCreateRoundRectRgn(
        Left,
        Top,
        Right,
        Bottom,
        Right - Left,
        Bottom - Top,
    )
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiCreateRectRgn(
    LeftRect: INT,
    TopRect: INT,
    RightRect: INT,
    BottomRect: INT,
) -> HRGN {
    let pRgn: PREGION;
    let hRgn: HRGN;

    pRgn = REGION_AllocUserRgnWithHandle(1);
    if pRgn.is_null() {
        EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return core::ptr::null_mut();
    }

    hRgn = (*pRgn).BaseObject.hHmgr as HRGN;

    REGION_SetRectRgn(pRgn, LeftRect, TopRect, RightRect, BottomRect);
    REGION_UnlockRgn(pRgn);

    return hRgn;
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiCreateRoundRectRgn(
    left: INT,
    top: INT,
    right: INT,
    bottom: INT,
    ellipse_width: INT,
    ellipse_height: INT,
) -> HRGN {
    let obj: PREGION;
    let hrgn: HRGN;
    let mut a: i32;
    let mut b: i32;
    let mut i: i32;
    let mut x: i32;
    let mut y: i32;
    let mut asq: INT64;
    let mut bsq: INT64;
    let mut dx: INT64;
    let mut dy: INT64;
    let mut err: INT64;
    let rects: *mut RECT;
    let mut left = left;
    let mut top = top;
    let mut right = right;
    let mut bottom = bottom;
    let mut ellipse_width = ellipse_width;
    let mut ellipse_height = ellipse_height;

    if left > right {
        let tmp = left;
        left = right;
        right = tmp;
    }

    if top > bottom {
        let tmp = top;
        top = bottom;
        bottom = tmp;
    }

    right -= 1;
    bottom -= 1;

    ellipse_width = min(right - left, abs(ellipse_width));
    ellipse_height = min(bottom - top, abs(ellipse_height));

    if (ellipse_width < 2) || (ellipse_height < 2) {
        return NtGdiCreateRectRgn(left, top, right, bottom);
    }

    obj = REGION_AllocUserRgnWithHandle(ellipse_height);
    if obj.is_null() {
        return core::ptr::null_mut();
    }

    hrgn = (*obj).BaseObject.hHmgr as HRGN;

    (*obj).rdh.rcBound.left = left;
    (*obj).rdh.rcBound.top = top;
    (*obj).rdh.rcBound.right = right;
    (*obj).rdh.rcBound.bottom = bottom;
    rects = (*obj).Buffer as PRECT;

    a = ellipse_width - 1;
    b = ellipse_height - 1;
    asq = 8i64 * a as INT64 * a as INT64;
    bsq = 8i64 * b as INT64 * b as INT64;
    dx = 4i64 * b as INT64 * b as INT64 * (1 - a) as INT64;
    dy = 4i64 * a as INT64 * a as INT64 * (1 + (b % 2)) as INT64;
    err = dx + dy + a as INT64 * a as INT64 * (b % 2) as INT64;

    x = 0;
    y = ellipse_height / 2;

    (*rects.offset(y as isize)).left = left;
    (*rects.offset(y as isize)).right = right;

    while x <= ellipse_width / 2 {
        let e2 = 2 * err;
        if e2 >= dx {
            x += 1;
            dx += bsq;
            err += dx;
        }
        if e2 <= dy {
            y += 1;
            dy += asq;
            err += dy;
            (*rects.offset(y as isize)).left = left + x;
            (*rects.offset(y as isize)).right = right - x;
        }
    }

    i = 0;
    while i < ellipse_height / 2 {
        (*rects.offset(i as isize)).left = (*rects.offset((b - i) as isize)).left;
        (*rects.offset(i as isize)).right = (*rects.offset((b - i) as isize)).right;
        (*rects.offset(i as isize)).top = top + i;
        (*rects.offset(i as isize)).bottom = (*rects.offset(i as isize)).top + 1;
        i += 1;
    }

    while i < ellipse_height {
        (*rects.offset(i as isize)).top = bottom - ellipse_height + i;
        (*rects.offset(i as isize)).bottom = (*rects.offset(i as isize)).top + 1;
        i += 1;
    }

    (*rects.offset((ellipse_height / 2) as isize)).top = top + ellipse_height / 2;

    REGION_UnlockRgn(obj);
    hrgn
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiEqualRgn(
    hSrcRgn1: HRGN,
    hSrcRgn2: HRGN,
) -> BOOL {
    let mut ahrgn: [HRGN; 2] = [core::ptr::null_mut(); 2];
    let mut aprgn: [PREGION; 2] = [core::ptr::null_mut(); 2];
    let rgn1: PREGION;
    let rgn2: PREGION;
    let tRect1: PRECTL;
    let tRect2: PRECTL;
    let mut i: ULONG;
    let mut bRet: BOOL = 0;

    if hSrcRgn1.is_null() || hSrcRgn2.is_null() {
        return 0;
    }

    if hSrcRgn1 == hSrcRgn2 {
        if (GDI_HANDLE_GET_TYPE(hSrcRgn1 as HGDIOBJ) == GDILoObjType_LO_REGION_TYPE) &&
            GreIsHandleValid(hSrcRgn1 as HGDIOBJ) != 0 {
            return 1;
        }
        return 0;
    }

    ahrgn[0] = hSrcRgn1;
    ahrgn[1] = hSrcRgn2;
    if GDIOBJ_bLockMultipleObjects(2, ahrgn.as_mut_ptr() as *mut HGDIOBJ, aprgn.as_mut_ptr() as *mut PVOID, GDIObjType_RGN_TYPE as u8) == 0 {
        return 0;
    }

    REGION_vSyncRegion(aprgn[0]);
    REGION_vSyncRegion(aprgn[1]);

    rgn1 = aprgn[0];
    rgn2 = aprgn[1];

    if (*rgn1).rdh.nCount != (*rgn2).rdh.nCount {
        REGION_UnlockRgn(rgn1);
        REGION_UnlockRgn(rgn2);
        return bRet;
    }

    if (*rgn1).rdh.nCount == 0 {
        bRet = 1;
        REGION_UnlockRgn(rgn1);
        REGION_UnlockRgn(rgn2);
        return bRet;
    }

    if ((*rgn1).rdh.rcBound.left   != (*rgn2).rdh.rcBound.left)  ||
       ((*rgn1).rdh.rcBound.right  != (*rgn2).rdh.rcBound.right) ||
       ((*rgn1).rdh.rcBound.top    != (*rgn2).rdh.rcBound.top)   ||
       ((*rgn1).rdh.rcBound.bottom != (*rgn2).rdh.rcBound.bottom) {
        REGION_UnlockRgn(rgn1);
        REGION_UnlockRgn(rgn2);
        return bRet;
    }

    tRect1 = (*rgn1).Buffer;
    tRect2 = (*rgn2).Buffer;

    if tRect1.is_null() || tRect2.is_null() {
        REGION_UnlockRgn(rgn1);
        REGION_UnlockRgn(rgn2);
        return bRet;
    }

    i = 0;
    while i < (*rgn1).rdh.nCount {
        if ((*tRect1.offset(i as isize)).left   != (*tRect2.offset(i as isize)).left)  ||
           ((*tRect1.offset(i as isize)).right  != (*tRect2.offset(i as isize)).right) ||
           ((*tRect1.offset(i as isize)).top    != (*tRect2.offset(i as isize)).top)   ||
           ((*tRect1.offset(i as isize)).bottom != (*tRect2.offset(i as isize)).bottom) {
            REGION_UnlockRgn(rgn1);
            REGION_UnlockRgn(rgn2);
            return bRet;
        }
        i += 1;
    }

    bRet = 1;

    REGION_UnlockRgn(rgn1);
    REGION_UnlockRgn(rgn2);
    bRet
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn NtGdiExtCreateRegion(
    Xform: LPXFORM,
    Count: u32,
    RgnData: LPRGNDATA,
) -> HRGN {
    let mut hRgn: HRGN;
    let Region: PREGION;
    let mut nCount: u32 = 0;
    let mut iType: u32 = 0;
    let mut dwSize: u32 = 0;
    let mut rects: *mut RECT = core::ptr::null_mut();
    let mut Status: i32 = STATUS_SUCCESS;
    let mut matrix: MATRIX = core::mem::zeroed();
    let mut xo: EXFORMOBJ = core::mem::zeroed(); // 8B on 64-bit: holds the pmx XFORMOBJ_vInit writes (XFORMOBJ itself is 4B → OOB)

    Status = SafeProbeForRead(RgnData as *const core::ffi::c_void, Count as usize, 1);
    if NT_SUCCESS(Status) {
        // Probe is validate-only (USE_DUMMY_PSEH); the C wrapped probe AND these
        // loads in one _SEH2_TRY, so the loads must be the fault-tolerant copy.
        let mut rdh: RGNDATAHEADER = core::mem::zeroed();
        if crate::copy::capture_from_user(
            core::ptr::addr_of_mut!(rdh) as *mut core::ffi::c_void,
            core::ptr::addr_of!((*RgnData).rdh) as *const core::ffi::c_void,
            core::mem::size_of::<RGNDATAHEADER>(),
        ) {
            nCount = rdh.nCount;
            iType = rdh.iType;
            dwSize = rdh.dwSize;
            // address-of only, never a load
            rects = core::ptr::addr_of_mut!((*RgnData).Buffer) as *mut RECT;
        } else {
            Status = STATUS_ACCESS_VIOLATION;
        }
    }

    if !NT_SUCCESS(Status) {
        SetLastNtError(Status);
        return core::ptr::null_mut();
    }

    if (Count < core::mem::size_of::<RGNDATAHEADER>() as u32 + nCount * core::mem::size_of::<RECT>() as u32) ||
       (iType != RDH_RECTANGLES) ||
       (dwSize != core::mem::size_of::<RGNDATAHEADER>() as u32) {
        return core::ptr::null_mut();
    }

    Region = REGION_AllocUserRgnWithHandle(nCount as i32);

    if Region.is_null() {
        EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0 as HRGN;
    }
    hRgn = (*Region).BaseObject.hHmgr as HRGN;

    {
        for i in 0..nCount {
            if (*rects.offset(i as isize)).left < (*rects.offset(i as isize)).right &&
               (*rects.offset(i as isize)).top < (*rects.offset(i as isize)).bottom {
                if REGION_UnionRectWithRgn(Region, rects.offset(i as isize) as *const _) == 0 {
                    REGION_UnlockRgn(Region);
                    GreDeleteObject(hRgn as HGDIOBJ);
                    hRgn = core::ptr::null_mut();
                    Status = STATUS_SUCCESS;
                    break;
                }
            }
        }

        if !Xform.is_null() && !hRgn.is_null() {
            let ret: u32;

            Status = ERROR_INVALID_PARAMETER as i32;
            XFORMOBJ_vInit((core::ptr::addr_of_mut!(xo) as *mut XFORMOBJ), core::ptr::addr_of_mut!(matrix));
            ret = EXFORMOBJ_iSetXform((core::ptr::addr_of_mut!(xo) as *mut XFORMOBJ), Xform as *mut XFORML);

            if ret != DDI_ERROR {
                if EXFORMOBJ_bApplyXform((core::ptr::addr_of_mut!(xo) as *mut XFORMOBJ),
                                        XF_LTOL,
                                        (*Region).rdh.nCount * 2,
                                        (*Region).Buffer as *mut core::ffi::c_void,
                                        (*Region).Buffer as *mut core::ffi::c_void) != 0 {
                    Status = STATUS_SUCCESS;
                }
            }
        }
    }

    if !NT_SUCCESS(Status) {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        REGION_UnlockRgn(Region);
        GreDeleteObject(hRgn as HGDIOBJ);
        return core::ptr::null_mut();
    }

    if !hRgn.is_null() {
        REGION_UnlockRgn(Region);
    }

    hRgn
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn NtGdiExtCreateRegion(
    Xform: LPXFORM,
    Count: u32,
    RgnData: LPRGNDATA,
) -> HRGN {
    let mut hRgn: HRGN;
    let Region: PREGION;
    let mut nCount: u32 = 0;
    let mut iType: u32 = 0;
    let mut dwSize: u32 = 0;
    let mut rects: *mut RECT = core::ptr::null_mut();
    let mut Status: i32 = STATUS_SUCCESS;
    let mut matrix: MATRIX = core::mem::zeroed();
    let mut xo: EXFORMOBJ = core::mem::zeroed(); // 8B on 64-bit: holds the pmx XFORMOBJ_vInit writes (XFORMOBJ itself is 4B → OOB)

    Status = SafeProbeForRead(RgnData as *const core::ffi::c_void, Count as usize, 1);
    if NT_SUCCESS(Status) {
        // Probe is validate-only (USE_DUMMY_PSEH); the C wrapped probe AND these
        // loads in one _SEH2_TRY, so the loads must be the fault-tolerant copy.
        let mut rdh: RGNDATAHEADER = core::mem::zeroed();
        if crate::copy::capture_from_user(
            core::ptr::addr_of_mut!(rdh) as *mut core::ffi::c_void,
            core::ptr::addr_of!((*RgnData).rdh) as *const core::ffi::c_void,
            core::mem::size_of::<RGNDATAHEADER>(),
        ) {
            nCount = rdh.nCount;
            iType = rdh.iType;
            dwSize = rdh.dwSize;
            // address-of only, never a load
            rects = core::ptr::addr_of_mut!((*RgnData).Buffer) as *mut RECT;
        } else {
            Status = STATUS_ACCESS_VIOLATION;
        }
    }

    if !NT_SUCCESS(Status) {
        SetLastNtError(Status);
        return core::ptr::null_mut();
    }

    if (Count < core::mem::size_of::<RGNDATAHEADER>() as u32 + nCount * core::mem::size_of::<RECT>() as u32) ||
       (iType != RDH_RECTANGLES) ||
       (dwSize != core::mem::size_of::<RGNDATAHEADER>() as u32) {
        return core::ptr::null_mut();
    }

    Region = REGION_AllocUserRgnWithHandle(nCount as i32);

    if Region.is_null() {
        EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0 as HRGN;
    }
    hRgn = (*Region).BaseObject.hHmgr as HRGN;

    {
        for i in 0..nCount {
            if (*rects.offset(i as isize)).left < (*rects.offset(i as isize)).right &&
               (*rects.offset(i as isize)).top < (*rects.offset(i as isize)).bottom {
                if REGION_UnionRectWithRgn(Region, rects.offset(i as isize) as *const _) == 0 {
                    REGION_UnlockRgn(Region);
                    GreDeleteObject(hRgn as HGDIOBJ);
                    hRgn = core::ptr::null_mut();
                    Status = STATUS_SUCCESS;
                    break;
                }
            }
        }

        if !Xform.is_null() && !hRgn.is_null() {
            let ret: u32;

            Status = ERROR_INVALID_PARAMETER as i32;
            XFORMOBJ_vInit(core::ptr::addr_of_mut!(xo) as *mut XFORMOBJ , core::ptr::addr_of_mut!(matrix));
            ret = EXFORMOBJ_iSetXform(core::ptr::addr_of_mut!(xo) as *mut XFORMOBJ , Xform as *mut XFORML);

            if ret != DDI_ERROR {
                if EXFORMOBJ_bApplyXform(core::ptr::addr_of_mut!(xo) as *mut XFORMOBJ ,
                                        XF_LTOL,
                                        (*Region).rdh.nCount * 2,
                                        (*Region).Buffer as *mut core::ffi::c_void,
                                        (*Region).Buffer as *mut core::ffi::c_void) != 0 {
                    Status = STATUS_SUCCESS;
                }
            }
        }
    }

    if !NT_SUCCESS(Status) {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        REGION_UnlockRgn(Region);
        GreDeleteObject(hRgn as HGDIOBJ);
        return core::ptr::null_mut();
    }

    if !hRgn.is_null() {
        REGION_UnlockRgn(Region);
    }

    hRgn
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiGetRgnBox(
    hRgn: HRGN,
    pRect: PRECTL,
) -> INT {
    let Rgn: PREGION;
    let mut SafeRect: RECTL = core::mem::zeroed();
    let ret: DWORD;
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    Rgn = REGION_LockRgn(hRgn);
    if Rgn.is_null() {
        return ERROR as _;
    }

    ret = REGION_GetRgnBox(Rgn, &mut SafeRect) as u32;
    REGION_UnlockRgn(Rgn);
    if ret == ERROR {
        return ret as INT;
    }

    Status = SafeProbeForWrite(pRect as *mut core::ffi::c_void, core::mem::size_of::<RECT>(), 1);
    if NT_SUCCESS(Status)
        // Probe is validate-only; the C had probe AND store in one _SEH2_TRY.
        && !crate::copy::publish_to_user(
            pRect as *mut core::ffi::c_void,
            core::ptr::addr_of!(SafeRect) as *const core::ffi::c_void,
            core::mem::size_of::<RECT>(),
        )
    {
        Status = STATUS_ACCESS_VIOLATION;
    }
    if !NT_SUCCESS(Status) {
        return ERROR as _;
    }

    return ret as INT;
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiOffsetRgn(
    hrgn: HRGN,
    cx: INT,
    cy: INT,
) -> INT {
    let prgn: PREGION;
    let iResult: INT;

    prgn = REGION_LockRgn(hrgn);
    if prgn.is_null() {
        return ERROR as _;
    }

    if REGION_bOffsetRgn(prgn, cx, cy) == 0 {
        iResult = ERROR as INT;
    } else {
        iResult = REGION_Complexity(prgn);
    }

    REGION_UnlockRgn(prgn);
    return iResult;
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiPtInRegion(
    hrgn: HRGN,
    x: i32,
    y: i32,
) -> i32 {
    let prgn: PREGION;
    let bResult: BOOL;

    prgn = REGION_LockRgn(hrgn);
    if prgn.is_null() {
        return 0;
    }

    bResult = REGION_PtInRegion(prgn, x, y);

    REGION_UnlockRgn(prgn);
    bResult as i32
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiRectInRegion(
    hrgn: HRGN,
    prclUnsafe: LPRECT,
) -> BOOL {
    let mut rcTemp: RECTL = core::mem::zeroed();

    {
        let __SehStatus = SafeProbeForRead(prclUnsafe as *const core::ffi::c_void, core::mem::size_of::<RECT>(), 1);
        if !NT_SUCCESS(__SehStatus) {
            return 0;
        }
    }
    // Probe is validate-only; the C had probe AND load in one _SEH2_TRY.
    if !crate::copy::capture_from_user(
        core::ptr::addr_of_mut!(rcTemp) as *mut core::ffi::c_void,
        prclUnsafe as *const core::ffi::c_void,
        core::mem::size_of::<RECTL>(),
    ) {
        return 0;
    }

    #[cfg(target_arch = "x86")]
    {
        IntRectInRegion(hrgn, core::ptr::addr_of_mut!(rcTemp))
    }
    #[cfg(not(target_arch = "x86"))]
    {
        IntRectInRegion(hrgn, core::ptr::addr_of_mut!(rcTemp))
    }
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiSetRectRgn(
    hrgn: HRGN,
    xLeft: INT,
    yTop: INT,
    xRight: INT,
    yBottom: INT,
) -> BOOL {
    let prgn: PREGION;

    prgn = REGION_LockRgn(hrgn);
    if prgn.is_null() {
        return 0;
    }

    REGION_SetRectRgn(prgn, xLeft, yTop, xRight, yBottom);

    REGION_UnlockRgn(prgn);
    1
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiGetRegionData(
    hrgn: HRGN,
    cjBuffer: u32,
    lpRgnData: LPRGNDATA,
) -> u32 {
    let cjRects: u32;
    let mut cjSize: u32;
    let prgn: PREGION;
    let Status: i32;

    prgn = REGION_LockRgn(hrgn);
    if prgn.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }

    cjRects = (*prgn).rdh.nCount * core::mem::size_of::<RECT>() as u32;
    cjSize = cjRects + core::mem::size_of::<RGNDATAHEADER>() as u32;

    if !lpRgnData.is_null() {
        if cjBuffer >= cjSize {
            Status = SafeProbeForWrite(
                lpRgnData as *mut core::ffi::c_void,
                cjSize as usize,
                core::mem::size_of::<u32>() as u32,
            );
            if !NT_SUCCESS(Status) {
                EngSetLastError(ERROR_INVALID_PARAMETER);
                cjSize = 0;
            } else {
                // Probe is validate-only (USE_DUMMY_PSEH); the C wrapped probe AND
                // these stores in one _SEH2_TRY. Patch the header in KERNEL memory
                // and publish once, so nothing reads back through the user pointer.
                let mut rdh: RGNDATAHEADER = *core::ptr::addr_of!((*prgn).rdh);
                rdh.iType = RDH_RECTANGLES;
                rdh.nRgnSize = cjRects;
                if !crate::copy::publish_to_user(
                    lpRgnData as *mut core::ffi::c_void,
                    core::ptr::addr_of!(rdh) as *const core::ffi::c_void,
                    core::mem::size_of::<RGNDATAHEADER>(),
                ) || !crate::copy::publish_to_user(
                    core::ptr::addr_of_mut!((*lpRgnData).Buffer) as *mut core::ffi::c_void,
                    (*prgn).Buffer as *const core::ffi::c_void,
                    cjRects as usize,
                ) {
                    EngSetLastError(ERROR_INVALID_PARAMETER);
                    cjSize = 0;
                }
            }
        } else {
            EngSetLastError(ERROR_INVALID_PARAMETER);
            cjSize = 0;
        }
    }

    REGION_UnlockRgn(prgn);
    cjSize
}
