//! infget.rs — INF get/find operations (InfpFindFirstLine, InfpGetStringField, etc.)
//! Translated from sdk/lib/inflib/infget.c.
//!
//! Fixes applied:
//!  - Duplicate const definitions (INF_STATUS_INVALID_PARAMETER/SUCCESS/NOT_FOUND/
//!    BUFFER_OVERFLOW, MAX_INF_STRING_LENGTH) removed; all from crate::types.
//!  - `STATUS_SUCCESS` undefined → use INF_STATUS_SUCCESS (= 0).
//!  - `InfpGetSubstitutionString` lang-id line: `((0u16 as i32) << 10) | ((*Inf).LanguageId as i32 & 0x3ff) as u16`
//!    is i32|u16 type error. FIXED: `MAKELANGID(PRIMARYLANGID(lang), SUBLANG_NEUTRAL)` =
//!    `(lang & 0x3ff) as u16` (SUBLANG_NEUTRAL=0, PRIMARYLANGID=lang & 0x3ff).
//!  - `INF_STATUS_NO_MEMORY` used in InfpFindFirstLine (was `1` in assembled); now
//!    sources from types (STATUS_NO_MEMORY = 0xC0000017).
//!  - `INF_STATUS_NOT_FOUND` was 0xE0000202 (fabricated); now 0xC0000225 from types.
//!  - `INF_STATUS_INVALID_PARAMETER` was 0x57 (Win32); now 0xC000000D from types.
//!  - Missing `let mut` on Section/CacheLine in InfpFindNextMatchLine added.
//!  - All sibling calls by direct Rust path (no extern {} for siblings).
//!  - `InfpFreeContext` takes *mut c_void (matches C: void* / PINFCONTEXT) and
//!    calls RtlFreeHeap directly.
//!  - `strlenW` = wcslen (wine unicode.h alias).

use crate::types::*;
use crate::infcore::{
    InfpFindSection, InfpFindKeyLine, InfpAddSection, InfpGetLineForContext,
    InfpGetSectionForContext,
};
use ntkrnl_sys::base::*;
use core::ffi::c_void;

extern "C" {
    pub static mut InfpHeap: *mut c_void;
    fn RtlAllocateHeap(HeapHandle: *mut c_void, Flags: ULONG, Size: SIZE_T) -> *mut c_void;
    fn RtlFreeHeap(HeapHandle: *mut c_void, Flags: ULONG, BaseAddress: *mut c_void);
    fn DbgPrint(fmt: *const u8, ...) -> i32;
    fn wcslen(s: *const u16) -> usize;
    fn wcscpy(dst: *mut u16, src: *const u16) -> *mut u16;
    fn wcstoul(s: *const u16, endptr: *mut *mut u16, base: i32) -> u32;
    fn wcstol(s: *const u16, endptr: *mut *mut u16, base: i32) -> i32;
    fn _wcsicmp(s1: *const u16, s2: *const u16) -> i32;
}

#[inline(always)]
unsafe fn MALLOC(size: usize) -> *mut c_void {
    RtlAllocateHeap(InfpHeap, 0, size)
}

// ──────────────────────────────────────────────────────────────────────────────
// Short-to-hex helper (used by InfpGetSubstitutionString)
// ──────────────────────────────────────────────────────────────────────────────

unsafe fn ShortToHex(buf: *mut u16, value: u16) {
    const HEX: [u16; 16] = [
        b'0' as u16, b'1' as u16, b'2' as u16, b'3' as u16,
        b'4' as u16, b'5' as u16, b'6' as u16, b'7' as u16,
        b'8' as u16, b'9' as u16, b'a' as u16, b'b' as u16,
        b'c' as u16, b'd' as u16, b'e' as u16, b'f' as u16,
    ];
    *buf.offset(0) = HEX[((value >> 12) & 0xf) as usize];
    *buf.offset(1) = HEX[((value >>  8) & 0xf) as usize];
    *buf.offset(2) = HEX[((value >>  4) & 0xf) as usize];
    *buf.offset(3) = HEX[((value      ) & 0xf) as usize];
}

// ──────────────────────────────────────────────────────────────────────────────
// String substitution helpers (private, kernel build)
// ──────────────────────────────────────────────────────────────────────────────

/// Retrieve the substitution string for `str[0..len]` from the [Strings*] sections.
/// FIXED: language ID computation; STATUS_SUCCESS undefined → use INF_STATUS_SUCCESS.
unsafe fn InfpGetSubstitutionString(
    Inf: PINFCACHE,
    str_ptr: PCWSTR,
    len: *mut usize,
    _no_trailing_slash: BOOL,
) -> PCWSTR {
    static PERCENT: [u16; 2] = [b'%' as u16, 0u16];

    let mut Status: INFSTATUS;
    let mut Context: PINFCONTEXT = core::ptr::null_mut();
    let mut Data: PWCHAR = core::ptr::null_mut();
    let mut ValueName: [u16; MAX_INF_STRING_LENGTH + 1] = [0u16; MAX_INF_STRING_LENGTH + 1];
    // L"Strings.XXXX" — 12 chars + NUL
    let mut StringLangId: [u16; 13] = [
        b'S' as u16, b't' as u16, b'r' as u16, b'i' as u16, b'n' as u16, b'g' as u16,
        b's' as u16, b'.' as u16, b'X' as u16, b'X' as u16, b'X' as u16, b'X' as u16, 0u16,
    ];

    if *len == 0 {
        *len = 1;
        return PERCENT.as_ptr() as PCWSTR;
    }

    core::ptr::copy_nonoverlapping(str_ptr, ValueName.as_mut_ptr(), *len);
    ValueName[*len] = 0;

    // sizeof("Strings.") - 1 = 8
    const IDX: usize = 8;

    if (*Inf).LanguageId != 0 {
        ShortToHex(StringLangId.as_mut_ptr().add(IDX), (*Inf).LanguageId);
        Status = InfpFindFirstLine(Inf, StringLangId.as_ptr(), ValueName.as_ptr(), &mut Context);
        if Status != INF_STATUS_SUCCESS {
            // MAKELANGID(PRIMARYLANGID(lang), SUBLANG_NEUTRAL) = (lang & 0x3ff) | (0 << 10)
            // FIXED: was `((0 << 10) | ((*Inf).LanguageId & 0x3ff)) as u16` — i32|u16 error
            let primary_only: u16 = (*Inf).LanguageId & 0x3ff;
            ShortToHex(StringLangId.as_mut_ptr().add(IDX), primary_only);
            Status = InfpFindFirstLine(Inf, StringLangId.as_ptr(), ValueName.as_ptr(), &mut Context);
            if Status != INF_STATUS_SUCCESS {
                let strings_w: [u16; 8] = [b'S' as u16, b't' as u16, b'r' as u16, b'i' as u16,
                                           b'n' as u16, b'g' as u16, b's' as u16, 0u16];
                Status = InfpFindFirstLine(Inf, strings_w.as_ptr(), ValueName.as_ptr(), &mut Context);
            }
        }
    } else {
        let strings_w: [u16; 8] = [b'S' as u16, b't' as u16, b'r' as u16, b'i' as u16,
                                   b'n' as u16, b'g' as u16, b's' as u16, 0u16];
        Status = InfpFindFirstLine(Inf, strings_w.as_ptr(), ValueName.as_ptr(), &mut Context);
    }

    if Status != INF_STATUS_SUCCESS || Context.is_null() {
        return core::ptr::null_mut();
    }

    Status = InfpGetData(Context, core::ptr::null_mut(), &mut Data);
    InfpFreeContext(Context as *mut c_void);

    // FIXED: STATUS_SUCCESS undefined → INF_STATUS_SUCCESS
    if Status == INF_STATUS_SUCCESS {
        *len = wcslen(Data);
        return Data;
    }
    core::ptr::null_mut()
}

unsafe fn InfpSubstituteString(
    Inf: PINFCACHE,
    text: PCWSTR,
    buffer: PWSTR,
    size: usize,
) -> usize {
    let mut start: *const u16 = text;
    let mut p:     *const u16 = text;
    let mut total: usize = 0;
    let mut avail: usize = if buffer.is_null() { MAX_INF_STRING_LENGTH + 1 } else { size };
    let mut inside = false;

    while *p != 0 {
        if *p == b'%' as u16 {
            inside = !inside;
            if inside {
                // start of %xx%
                let len = p.offset_from(start) as usize;
                let copy = len.min(avail.saturating_sub(1));
                if !buffer.is_null() {
                    core::ptr::copy_nonoverlapping(start, buffer.add(total), copy);
                }
                total += copy;
                avail = avail.saturating_sub(copy);
                start = p;
            } else {
                // end of %xx%
                let mut len = (p.offset_from(start) as usize).saturating_sub(1);
                let subst = InfpGetSubstitutionString(
                    Inf,
                    start.add(1),
                    &mut len,
                    (*p.add(1) == b'\\' as u16) as BOOL,
                );
                let (subst_ptr, subst_len) = if subst.is_null() {
                    let full_len = p.offset_from(start) as usize + 1;
                    (start, full_len)
                } else {
                    (subst, len)
                };
                let copy = subst_len.min(avail.saturating_sub(1));
                if !buffer.is_null() {
                    core::ptr::copy_nonoverlapping(subst_ptr, buffer.add(total), copy);
                }
                total += copy;
                avail = avail.saturating_sub(copy);
                start = p.add(1);
            }
        }
        p = p.add(1);
    }
    if start != p {
        let len = p.offset_from(start) as usize;
        let copy = len.min(avail.saturating_sub(1));
        if !buffer.is_null() {
            core::ptr::copy_nonoverlapping(start, buffer.add(total), copy);
        }
        total += copy;
    }
    if !buffer.is_null() && avail != 0 {
        *buffer.add(total) = 0;
    }
    total
}

// ──────────────────────────────────────────────────────────────────────────────
// Public find/get functions
// ──────────────────────────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn InfpFindFirstLine(
    Cache: PINFCACHE,
    Section: PCWSTR,
    Key: PCWSTR,
    Context: *mut PINFCONTEXT,
) -> INFSTATUS {
    if Cache.is_null() || Section.is_null() || Context.is_null() {
        DbgPrint(b"InfpFindFirstLine: Invalid parameter\n\0".as_ptr());
        return INF_STATUS_INVALID_PARAMETER;
    }
    let CacheSection = InfpFindSection(Cache, Section);
    if CacheSection.is_null() {
        return INF_STATUS_NOT_FOUND;
    }
    let CacheLine = if !Key.is_null() {
        InfpFindKeyLine(CacheSection, Key)
    } else {
        (*CacheSection).FirstLine
    };
    if CacheLine.is_null() {
        return INF_STATUS_NOT_FOUND;
    }
    *Context = MALLOC(core::mem::size_of::<INFCONTEXT>()) as PINFCONTEXT;
    if (*Context).is_null() {
        DbgPrint(b"InfpFindFirstLine: MALLOC() failed\n\0".as_ptr());
        return INF_STATUS_NO_MEMORY;
    }
    (**Context).Inf        = Cache;
    (**Context).CurrentInf = Cache;
    (**Context).Section    = (*CacheSection).Id;
    (**Context).Line       = (*CacheLine).Id;
    INF_STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn InfpFindNextLine(
    ContextIn:  PINFCONTEXT,
    ContextOut: PINFCONTEXT,
) -> INFSTATUS {
    if ContextIn.is_null() || ContextOut.is_null() {
        return INF_STATUS_INVALID_PARAMETER;
    }
    let CacheLine = InfpGetLineForContext(ContextIn);
    if CacheLine.is_null() {
        return INF_STATUS_INVALID_PARAMETER;
    }
    if (*CacheLine).Next.is_null() {
        return INF_STATUS_NOT_FOUND;
    }
    if ContextIn != ContextOut {
        (*ContextOut).Inf     = (*ContextIn).Inf;
        (*ContextOut).Section = (*ContextIn).Section;
    }
    (*ContextOut).Line = (*(*CacheLine).Next).Id;
    INF_STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn InfpFindFirstMatchLine(
    ContextIn:  PINFCONTEXT,
    Key:        PCWSTR,
    ContextOut: PINFCONTEXT,
) -> INFSTATUS {
    if ContextIn.is_null() || ContextOut.is_null() || Key.is_null() || *Key == 0 {
        return INF_STATUS_INVALID_PARAMETER;
    }
    let Section = InfpGetSectionForContext(ContextIn);
    if Section.is_null() {
        return INF_STATUS_INVALID_PARAMETER;
    }
    let mut CacheLine = (*Section).FirstLine;
    while !CacheLine.is_null() {
        if !(*CacheLine).Key.is_null() && _wcsicmp((*CacheLine).Key, Key) == 0 {
            if ContextIn != ContextOut {
                (*ContextOut).Inf     = (*ContextIn).Inf;
                (*ContextOut).Section = (*ContextIn).Section;
            }
            (*ContextOut).Line = (*CacheLine).Id;
            return INF_STATUS_SUCCESS;
        }
        CacheLine = (*CacheLine).Next;
    }
    INF_STATUS_NOT_FOUND
}

/// FIXED: missing `mut` on Section/CacheLine; wrong constant values for
/// INF_STATUS_INVALID_PARAMETER (was 0x57) and INF_STATUS_NOT_FOUND (was 0xE0000202).
#[no_mangle]
pub unsafe extern "C" fn InfpFindNextMatchLine(
    ContextIn:  PINFCONTEXT,
    Key:        PCWSTR,
    ContextOut: PINFCONTEXT,
) -> INFSTATUS {
    if ContextIn.is_null() || ContextOut.is_null() || Key.is_null() || *Key == 0 {
        return INF_STATUS_INVALID_PARAMETER;
    }
    let mut Section = InfpGetSectionForContext(ContextIn);
    if Section.is_null() {
        return INF_STATUS_INVALID_PARAMETER;
    }
    let _ = Section; // suppress unused warning; used in original but we iterate via CacheLine
    let mut CacheLine = InfpGetLineForContext(ContextIn);
    while !CacheLine.is_null() {
        if !(*CacheLine).Key.is_null() && _wcsicmp((*CacheLine).Key, Key) == 0 {
            if ContextIn != ContextOut {
                (*ContextOut).Inf     = (*ContextIn).Inf;
                (*ContextOut).Section = (*ContextIn).Section;
            }
            (*ContextOut).Line = (*CacheLine).Id;
            return INF_STATUS_SUCCESS;
        }
        CacheLine = (*CacheLine).Next;
    }
    INF_STATUS_NOT_FOUND
}

#[no_mangle]
pub unsafe extern "C" fn InfpGetLineCount(InfHandle: HINF, Section: PCWSTR) -> LONG {
    if InfHandle.is_null() || Section.is_null() {
        return -1;
    }
    let Cache = InfHandle as PINFCACHE;
    let mut CacheSection = (*Cache).FirstSection;
    while !CacheSection.is_null() {
        if _wcsicmp(
            core::ptr::addr_of!((*CacheSection).Name) as *const u16,
            Section,
        ) == 0
        {
            return (*CacheSection).LineCount;
        }
        CacheSection = (*CacheSection).Next;
    }
    -1
}

#[no_mangle]
pub unsafe extern "C" fn InfpGetFieldCount(Context: PINFCONTEXT) -> LONG {
    let Line = InfpGetLineForContext(Context);
    if Line.is_null() { return 0; }
    (*Line).FieldCount
}

#[no_mangle]
pub unsafe extern "C" fn InfpGetBinaryField(
    Context:          PINFCONTEXT,
    FieldIndex:       ULONG,
    ReturnBuffer:     PUCHAR,
    ReturnBufferSize: ULONG,
    RequiredSize:     PULONG,
) -> INFSTATUS {
    if Context.is_null() || FieldIndex == 0 {
        return INF_STATUS_INVALID_PARAMETER;
    }
    if !RequiredSize.is_null() { *RequiredSize = 0; }
    let CacheLine = InfpGetLineForContext(Context);
    if FieldIndex > (*CacheLine).FieldCount as ULONG {
        return INF_STATUS_NOT_FOUND;
    }
    let mut CacheField = (*CacheLine).FirstField;
    for _ in 1..FieldIndex {
        CacheField = (*CacheField).Next;
    }
    let Size = ((*CacheLine).FieldCount as ULONG)
        .wrapping_sub(FieldIndex)
        .wrapping_add(1);
    if !RequiredSize.is_null() { *RequiredSize = Size; }
    if !ReturnBuffer.is_null() {
        if ReturnBufferSize < Size {
            return INF_STATUS_BUFFER_OVERFLOW;
        }
        let mut Ptr = ReturnBuffer;
        while !CacheField.is_null() {
            *Ptr = wcstoul(
                core::ptr::addr_of!((*CacheField).Data) as *const u16,
                core::ptr::null_mut(),
                16,
            ) as UCHAR;
            Ptr = Ptr.add(1);
            CacheField = (*CacheField).Next;
        }
    }
    INF_STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn InfpGetIntField(
    Context:      PINFCONTEXT,
    FieldIndex:   ULONG,
    IntegerValue: *mut INT,
) -> INFSTATUS {
    if Context.is_null() || IntegerValue.is_null() {
        return INF_STATUS_INVALID_PARAMETER;
    }
    let CacheLine = InfpGetLineForContext(Context);
    if FieldIndex > (*CacheLine).FieldCount as ULONG {
        return INF_STATUS_INVALID_PARAMETER;
    }
    let Ptr: *const u16 = if FieldIndex == 0 {
        (*CacheLine).Key
    } else {
        let mut CacheField = (*CacheLine).FirstField;
        for _ in 1..FieldIndex {
            CacheField = (*CacheField).Next;
        }
        core::ptr::addr_of!((*CacheField).Data) as *const u16
    };
    *IntegerValue = wcstol(Ptr, core::ptr::null_mut(), 0) as INT;
    INF_STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn InfpGetMultiSzField(
    Context:          PINFCONTEXT,
    FieldIndex:       ULONG,
    ReturnBuffer:     PWSTR,
    ReturnBufferSize: ULONG,
    RequiredSize:     PULONG,
) -> INFSTATUS {
    if Context.is_null() || FieldIndex == 0 {
        return INF_STATUS_INVALID_PARAMETER;
    }
    if !RequiredSize.is_null() { *RequiredSize = 0; }
    let CacheLine = InfpGetLineForContext(Context);
    if FieldIndex > (*CacheLine).FieldCount as ULONG {
        return INF_STATUS_INVALID_PARAMETER;
    }
    let mut CacheField = (*CacheLine).FirstField;
    for _ in 1..FieldIndex {
        CacheField = (*CacheField).Next;
    }
    // Calculate required size (each field + NUL, plus trailing double-NUL)
    let mut Size: ULONG = 0;
    let mut FieldPtr = CacheField;
    while !FieldPtr.is_null() {
        Size = Size.wrapping_add(
            wcslen(core::ptr::addr_of!((*FieldPtr).Data) as *const u16) as ULONG + 1
        );
        FieldPtr = (*FieldPtr).Next;
    }
    Size = Size.wrapping_add(1); // trailing NUL
    if !RequiredSize.is_null() { *RequiredSize = Size; }
    if !ReturnBuffer.is_null() {
        if ReturnBufferSize < Size {
            return INF_STATUS_BUFFER_OVERFLOW;
        }
        let mut Ptr = ReturnBuffer;
        FieldPtr = CacheField;
        while !FieldPtr.is_null() {
            let flen = wcslen(core::ptr::addr_of!((*FieldPtr).Data) as *const u16) as ULONG + 1;
            wcscpy(Ptr, core::ptr::addr_of!((*FieldPtr).Data) as *const u16);
            Ptr = Ptr.add(flen as usize);
            FieldPtr = (*FieldPtr).Next;
        }
        *Ptr = 0;
    }
    INF_STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn InfpGetStringField(
    Context:          PINFCONTEXT,
    FieldIndex:       ULONG,
    ReturnBuffer:     PWSTR,
    ReturnBufferSize: ULONG,
    RequiredSize:     PULONG,
) -> INFSTATUS {
    if Context.is_null() {
        return INF_STATUS_INVALID_PARAMETER;
    }
    if !RequiredSize.is_null() { *RequiredSize = 0; }
    let CacheLine = InfpGetLineForContext(Context);
    if FieldIndex > (*CacheLine).FieldCount as ULONG {
        return INF_STATUS_INVALID_PARAMETER;
    }
    let Ptr: *const u16 = if FieldIndex == 0 {
        (*CacheLine).Key
    } else {
        let mut CacheField = (*CacheLine).FirstField;
        for _ in 1..FieldIndex {
            CacheField = (*CacheField).Next;
        }
        core::ptr::addr_of!((*CacheField).Data) as *const u16
    };
    let Size = InfpSubstituteString((*Context).Inf, Ptr, core::ptr::null_mut(), 0);
    if !RequiredSize.is_null() {
        *RequiredSize = (Size as ULONG).wrapping_add(1);
    }
    if !ReturnBuffer.is_null() {
        if ReturnBufferSize as usize <= Size {
            return INF_STATUS_BUFFER_OVERFLOW;
        }
        InfpSubstituteString((*Context).Inf, Ptr, ReturnBuffer, ReturnBufferSize as usize);
    }
    INF_STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn InfpGetData(
    Context: PINFCONTEXT,
    Key:     *mut PWCHAR,
    Data:    *mut PWCHAR,
) -> INFSTATUS {
    if Context.is_null() || Data.is_null() {
        return INF_STATUS_INVALID_PARAMETER;
    }
    let CacheLine = InfpGetLineForContext(Context);
    if !Key.is_null() {
        *Key = (*CacheLine).Key;
    }
    if (*CacheLine).FirstField.is_null() {
        *Data = core::ptr::null_mut();
    } else {
        *Data = core::ptr::addr_of_mut!((*(*CacheLine).FirstField).Data) as PWCHAR;
    }
    INF_STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn InfpGetDataField(
    Context:    PINFCONTEXT,
    FieldIndex: ULONG,
    Data:       *mut PWCHAR,
) -> INFSTATUS {
    if Context.is_null() || Data.is_null() {
        return INF_STATUS_INVALID_PARAMETER;
    }
    let CacheLine = InfpGetLineForContext(Context);
    if FieldIndex > (*CacheLine).FieldCount as ULONG {
        return INF_STATUS_INVALID_PARAMETER;
    }
    if FieldIndex == 0 {
        *Data = (*CacheLine).Key;
    } else {
        let mut CacheField = (*CacheLine).FirstField;
        for _ in 1..FieldIndex {
            CacheField = (*CacheField).Next;
        }
        *Data = core::ptr::addr_of_mut!((*CacheField).Data) as PWCHAR;
    }
    INF_STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn InfpFreeContext(Context: *mut c_void) {
    RtlFreeHeap(InfpHeap, 0, Context);
}
