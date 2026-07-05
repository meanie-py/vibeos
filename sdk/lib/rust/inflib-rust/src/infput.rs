//! infput.rs — INF write/put operations (InfpBuildFileBuffer, InfpFindOrAddSection, etc.)
//! Translated from sdk/lib/inflib/infput.c.
//!
//! Fixes applied:
//!  - `OUTPUTBUFFER` has a plain `Status: INFSTATUS` field — no union, no `__bindgen_anon_1`.
//!    All `(*OutBuf).__bindgen_anon_1.Status` → `(*OutBuf).Status`.
//!  - `EOL` as `*const u16` → local inline static.
//!  - `INFSTATUS` is i32 (signed); `INF_STATUS_NO_MEMORY` correct from types.
//!  - `DPRINT(...)` extern block eliminated (debug macro, no-op in NDEBUG build).
//!  - `INF_STATUS_NO_MEMORY = 1` duplicate const removed; use crate::types.
//!  - Sibling calls (InfpFindSection, InfpAddSection, etc.) by direct Rust path.
//!  - `Output` copy-loop fixed: Length was computed as byte count, but loop was
//!    decrementing by 1 per iteration — correct for u16 copy since the original
//!    C code does `Length = strlenW(Text) * sizeof(WCHAR)` then copies sizeof(WCHAR)
//!    sized chars.  We keep it faithful: Length in bytes, loop steps WCHARs.
//!  - `extern "C" fn DPRINT(...)` extern block removed (NDEBUG = no-op).

use crate::types::*;
use crate::infcore::{
    InfpFindSection, InfpAddSection, InfpAddLine, InfpAddKeyToLine,
    InfpAddFieldToLine, InfpGetSectionForContext, InfpGetLineForContext,
};
use ntkrnl_sys::base::*;
use core::ffi::c_void;

extern "system" {
    fn RtlAllocateHeap(HeapHandle: *mut c_void, Flags: ULONG, Size: SIZE_T) -> *mut c_void;
    fn RtlFreeHeap(HeapHandle: *mut c_void, Flags: ULONG, BaseAddress: *mut c_void);
}

extern "C" {
    pub static mut InfpHeap: *mut c_void;
    fn DbgPrint(fmt: *const u8, ...) -> i32;
    fn wcslen(s: *const u16) -> usize;
}

#[inline(always)]
unsafe fn MALLOC(size: usize) -> *mut c_void {
    RtlAllocateHeap(InfpHeap, 0, size)
}

// EOL = L"\r\n" as a static wide-string
static EOL: [u16; 3] = [0x0D, 0x0A, 0x00];

// ──────────────────────────────────────────────────────────────────────────────
// OUTPUTBUFFER — plain struct (no union)
// ──────────────────────────────────────────────────────────────────────────────

#[repr(C)]
struct OUTPUTBUFFER {
    Buffer:    PWCHAR,
    Current:   PWCHAR,
    TotalSize: ULONG,
    FreeSize:  ULONG,
    // FIXED: no __bindgen_anon_1 — plain field (INFSTATUS = i32)
    Status:    INFSTATUS,
}
type POUTPUTBUFFER = *mut OUTPUTBUFFER;

const SIZE_INC: u32 = 1024;

unsafe fn Output(OutBuf: *mut OUTPUTBUFFER, Text: *const u16) {
    // Skip mode?
    if (*OutBuf).Status < 0 {
        return;
    }
    // Length in bytes (not chars)
    let text_chars = wcslen(Text) as u32;
    let length_bytes: u32 = text_chars.wrapping_mul(core::mem::size_of::<u16>() as u32);
    if (*OutBuf).FreeSize < length_bytes.wrapping_add(1) {
        // Round up to SIZE_INC
        let need = length_bytes.wrapping_add(1).wrapping_sub((*OutBuf).FreeSize);
        let inc   = need.wrapping_add(SIZE_INC - 1) / SIZE_INC * SIZE_INC;
        let new_size = (*OutBuf).TotalSize.wrapping_add(inc);
        let new_buf = MALLOC(new_size as usize) as *mut u16;
        if new_buf.is_null() {
            DbgPrint(b"Output: MALLOC() failed\n\0".as_ptr());
            (*OutBuf).Status = INF_STATUS_NO_MEMORY;
            return;
        }
        if !(*OutBuf).Buffer.is_null() {
            let used = (*OutBuf).TotalSize.wrapping_sub((*OutBuf).FreeSize);
            core::ptr::copy_nonoverlapping(
                (*OutBuf).Buffer as *const u8,
                new_buf as *mut u8,
                used as usize,
            );
            let offset = (*OutBuf).Current.offset_from((*OutBuf).Buffer) as isize;
            (*OutBuf).Current = new_buf.offset(offset);
            RtlFreeHeap(InfpHeap, 0, (*OutBuf).Buffer as *mut c_void);
        } else {
            (*OutBuf).Current = new_buf;
        }
        (*OutBuf).Buffer    = new_buf;
        (*OutBuf).FreeSize  = (*OutBuf).FreeSize.wrapping_add(inc);
        (*OutBuf).TotalSize = new_size;
    }
    // Copy char by char (WCHAR)
    let mut src = Text;
    let mut remaining = text_chars;
    while remaining > 0 {
        *(*OutBuf).Current = *src;
        (*OutBuf).Current  = (*OutBuf).Current.add(1);
        (*OutBuf).FreeSize = (*OutBuf).FreeSize.wrapping_sub(core::mem::size_of::<u16>() as u32);
        src      = src.add(1);
        remaining -= 1;
    }
    *(*OutBuf).Current = 0;
}

// ──────────────────────────────────────────────────────────────────────────────
// Public functions
// ──────────────────────────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn InfpBuildFileBuffer(
    Cache:      PINFCACHE,
    Buffer:     *mut PWCHAR,
    BufferSize: PULONG,
) -> INFSTATUS {
    let mut OutBuf = OUTPUTBUFFER {
        Buffer:    core::ptr::null_mut(),
        Current:   core::ptr::null_mut(),
        TotalSize: 0,
        FreeSize:  0,
        Status:    INF_STATUS_SUCCESS,
    };

    let mut CacheSection = (*Cache).FirstSection;
    while !CacheSection.is_null() {
        if CacheSection != (*Cache).FirstSection {
            Output(&mut OutBuf, EOL.as_ptr());
        }
        let lbracket:  [u16; 2] = [b'[' as u16, 0];
        let rbracket:  [u16; 2] = [b']' as u16, 0];
        Output(&mut OutBuf, lbracket.as_ptr());
        Output(&mut OutBuf, core::ptr::addr_of!((*CacheSection).Name) as *const u16);
        Output(&mut OutBuf, rbracket.as_ptr());
        Output(&mut OutBuf, EOL.as_ptr());

        let mut CacheLine = (*CacheSection).FirstLine;
        while !CacheLine.is_null() {
            if !(*CacheLine).Key.is_null() {
                Output(&mut OutBuf, (*CacheLine).Key);
                let eq_space: [u16; 4] = [b' ' as u16, b'=' as u16, b' ' as u16, 0];
                Output(&mut OutBuf, eq_space.as_ptr());
            }

            let mut CacheField = (*CacheLine).FirstField;
            while !CacheField.is_null() {
                if CacheField != (*CacheLine).FirstField {
                    let comma: [u16; 2] = [b',' as u16, 0];
                    Output(&mut OutBuf, comma.as_ptr());
                }
                let mut p = core::ptr::addr_of_mut!((*CacheField).Data) as *mut u16;
                let mut need_quotes: bool = false;
                while *p != 0 && !need_quotes {
                    need_quotes = *p == b',' as u16 || *p == b';' as u16 || *p == b'\\' as u16;
                    p = p.add(1);
                }
                let quote: [u16; 2] = [b'"' as u16, 0];
                if need_quotes {
                    Output(&mut OutBuf, quote.as_ptr());
                    Output(&mut OutBuf, core::ptr::addr_of!((*CacheField).Data) as *const u16);
                    Output(&mut OutBuf, quote.as_ptr());
                } else {
                    Output(&mut OutBuf, core::ptr::addr_of!((*CacheField).Data) as *const u16);
                }
                CacheField = (*CacheField).Next;
            }
            Output(&mut OutBuf, EOL.as_ptr());
            CacheLine = (*CacheLine).Next;
        }
        CacheSection = (*CacheSection).Next;
    }

    if OutBuf.Status >= 0 {
        *Buffer     = OutBuf.Buffer;
        *BufferSize = OutBuf.TotalSize.wrapping_sub(OutBuf.FreeSize);
    } else if !OutBuf.Buffer.is_null() {
        RtlFreeHeap(InfpHeap, 0, OutBuf.Buffer as *mut c_void);
    }
    INF_STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn InfpFindOrAddSection(
    Cache:   PINFCACHE,
    Section: PCWSTR,
    Context: *mut PINFCONTEXT,
) -> INFSTATUS {
    *Context = MALLOC(core::mem::size_of::<INFCONTEXT>()) as PINFCONTEXT;
    if (*Context).is_null() {
        DbgPrint(b"InfpFindOrAddSection: MALLOC() failed\n\0".as_ptr());
        return INF_STATUS_NO_MEMORY;
    }
    (**Context).Inf  = Cache;
    (**Context).Line = 0;

    let mut CacheSection = InfpFindSection(Cache, Section);
    if CacheSection.is_null() {
        CacheSection = InfpAddSection(Cache, Section);
        if CacheSection.is_null() {
            RtlFreeHeap(InfpHeap, 0, *Context as *mut c_void);
            return INF_STATUS_NO_MEMORY;
        }
    }
    (**Context).Section = (*CacheSection).Id;
    INF_STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn InfpAddLineWithKey(Context: PINFCONTEXT, Key: PCWSTR) -> INFSTATUS {
    if Context.is_null() {
        DbgPrint(b"InfpAddLineWithKey: Invalid parameter\n\0".as_ptr());
        return INF_STATUS_INVALID_PARAMETER;
    }
    let Section = InfpGetSectionForContext(Context);
    let Line = InfpAddLine(Section);
    if Line.is_null() {
        return INF_STATUS_NO_MEMORY;
    }
    (*Context).Line = (*Line).Id;
    if !Key.is_null() && InfpAddKeyToLine(Line, Key).is_null() {
        return INF_STATUS_NO_MEMORY;
    }
    INF_STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn InfpAddField(Context: PINFCONTEXT, Data: PCWSTR) -> INFSTATUS {
    if Context.is_null() {
        DbgPrint(b"InfpAddField: Invalid parameter\n\0".as_ptr());
        return INF_STATUS_INVALID_PARAMETER;
    }
    let Line = InfpGetLineForContext(Context);
    if InfpAddFieldToLine(Line, Data).is_null() {
        return INF_STATUS_NO_MEMORY;
    }
    INF_STATUS_SUCCESS
}
