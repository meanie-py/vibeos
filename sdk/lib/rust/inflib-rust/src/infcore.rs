//! infcore.rs — INF parser core: section/line/field allocation + parser state machine.
//! Translated from sdk/lib/inflib/infcore.c.
//!
//! Fixes applied:
//!  - `_SPACE = 0` placeholder in line_start_state / leading_spaces_state removed;
//!    replaced with inline `isspaceW()` that matches iswspace() semantics
//!    (space/tab/CR/LF/VT/FF: 0x09/0x0A/0x0B/0x0C/0x0D/0x20).
//!  - `offset_of!(INFCACHEFIELD, Data[wcslen+1])` (invalid Rust) replaced with
//!    `offset_of!(..,Data) + (wcslen(Data)+1)*size_of::<u16>()`.
//!  - Double `#[no_mangle] #[no_mangle]` on InfpParseBuffer collapsed to one.
//!  - `null_mut::<PINFCACHE>()` double-pointer in InfpGetSectionForContext fixed
//!    to `null_mut::<_INFCACHE>()`.
//!  - extern "C" blocks for FREE/MALLOC/ZEROMEMORY/DPRINT/DPRINT1 eliminated;
//!    those are resolved via the kernel heap globals and DbgPrint below.
//!  - All sibling function cross-calls are by direct Rust path (no extern {}).
//!  - `INFSTATUS` used from crate::types (signed i32, correct NTSTATUS values).
//!  - `isspaceW` / `iswctype` extern decl replaced with local inline fn.

use crate::types::*;
use ntkrnl_sys::base::*;
use core::ffi::c_void;
use core::mem::{offset_of, size_of};

// ──────────────────────────────────────────────────────────────────────────────
// Kernel heap + runtime helpers (resolved from ntoskrnl exports)
// ──────────────────────────────────────────────────────────────────────────────

extern "C" {
    pub static mut InfpHeap: *mut c_void;

    fn RtlAllocateHeap(HeapHandle: *mut c_void, Flags: ULONG, Size: SIZE_T) -> *mut c_void;
    fn RtlFreeHeap(HeapHandle: *mut c_void, Flags: ULONG, BaseAddress: *mut c_void);
    fn DbgPrint(fmt: *const u8, ...) -> i32;
    fn wcslen(s: *const u16) -> usize;
    fn wcscpy(dst: *mut u16, src: *const u16) -> *mut u16;
    fn _wcsicmp(s1: *const u16, s2: *const u16) -> i32;
}

// ──────────────────────────────────────────────────────────────────────────────
// Macro-replacement helpers (kernel build: MALLOC = RtlAllocateHeap, etc.)
// ──────────────────────────────────────────────────────────────────────────────

#[inline(always)]
unsafe fn MALLOC(size: usize) -> *mut c_void {
    RtlAllocateHeap(InfpHeap, 0, size)
}
#[inline(always)]
unsafe fn FREE(ptr: *mut c_void) {
    RtlFreeHeap(InfpHeap, 0, ptr);
}
#[inline(always)]
unsafe fn ZEROMEMORY(ptr: *mut c_void, size: usize) {
    core::ptr::write_bytes(ptr as *mut u8, 0, size);
}
/// Inline equivalent of wine's `isspaceW(n)` = `iswspace(n)`.
/// Matches characters: 0x09 HT, 0x0A LF, 0x0B VT, 0x0C FF, 0x0D CR, 0x20 SPACE.
#[inline(always)]
fn isspaceW(c: u16) -> bool {
    matches!(c, 0x09 | 0x0A | 0x0B | 0x0C | 0x0D | 0x20)
}

// ──────────────────────────────────────────────────────────────────────────────
// Parser state machine constants / types
// ──────────────────────────────────────────────────────────────────────────────

pub const CONTROL_Z: u16 = 0x1a;
pub const MAX_SECTION_NAME_LEN: u32 = 255;
pub const MAX_FIELD_LEN: u32 = 511;

pub type parser_state = u32;
pub const LINE_START:       parser_state = 0;
pub const SECTION_NAME:     parser_state = 1;
pub const KEY_NAME:         parser_state = 2;
pub const VALUE_NAME:       parser_state = 3;
pub const EOL_BACKSLASH:    parser_state = 4;
pub const QUOTES:           parser_state = 5;
pub const LEADING_SPACES:   parser_state = 6;
pub const TRAILING_SPACES:  parser_state = 7;
pub const COMMENT:          parser_state = 8;
pub const NB_PARSER_STATES: usize = 9;

pub type parser_state_func = unsafe extern "C" fn(*mut parser, *const u16) -> *const u16;

#[repr(C)]
pub struct parser {
    pub start:       *const u16,
    pub end:         *const u16,
    pub file:        PINFCACHE,
    pub state:       parser_state,
    pub stack:       [parser_state; 4],
    pub stack_pos:   i32,
    pub cur_section: PINFCACHESECTION,
    pub line:        PINFCACHELINE,
    pub line_pos:    u32,
    pub error:       INFSTATUS,
    pub token_len:   u32,
    pub token:       [u16; MAX_FIELD_LEN as usize + 1],
}

pub static parser_funcs: [parser_state_func; NB_PARSER_STATES] = [
    line_start_state,
    section_name_state,
    key_name_state,
    value_name_state,
    eol_backslash_state,
    quotes_state,
    leading_spaces_state,
    trailing_spaces_state,
    comment_state,
];

// ──────────────────────────────────────────────────────────────────────────────
// Private section/line/field helpers
// ──────────────────────────────────────────────────────────────────────────────

unsafe fn InfpFreeLine(Line: *mut INFCACHELINE) -> *mut INFCACHELINE {
    if Line.is_null() {
        return core::ptr::null_mut();
    }
    let next = (*Line).Next;
    if !(*Line).Key.is_null() {
        FREE((*Line).Key as *mut c_void);
        (*Line).Key = core::ptr::null_mut();
    }
    while !(*Line).FirstField.is_null() {
        let field_next = (*(*Line).FirstField).Next;
        FREE((*Line).FirstField as *mut c_void);
        (*Line).FirstField = field_next;
    }
    (*Line).LastField = core::ptr::null_mut();
    FREE(Line as *mut c_void);
    next
}

#[no_mangle]
pub unsafe extern "C" fn InfpFreeSection(Section: PINFCACHESECTION) -> PINFCACHESECTION {
    if Section.is_null() {
        return core::ptr::null_mut();
    }
    let next = (*Section).Next;
    while !(*Section).FirstLine.is_null() {
        (*Section).FirstLine = InfpFreeLine((*Section).FirstLine);
    }
    (*Section).LastLine = core::ptr::null_mut();
    FREE(Section as *mut c_void);
    next
}

#[no_mangle]
pub unsafe extern "C" fn InfpFindSection(Cache: PINFCACHE, Name: PCWSTR) -> PINFCACHESECTION {
    if Cache.is_null() || Name.is_null() {
        return core::ptr::null_mut();
    }
    let mut Section = (*Cache).FirstSection;
    while !Section.is_null() {
        if _wcsicmp(
            core::ptr::addr_of!((*Section).Name) as *const u16,
            Name,
        ) == 0
        {
            return Section;
        }
        Section = (*Section).Next;
    }
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn InfpAddSection(Cache: PINFCACHE, Name: PCWSTR) -> PINFCACHESECTION {
    if Cache.is_null() || Name.is_null() {
        return core::ptr::null_mut();
    }
    let nlen = wcslen(Name) + 1;
    // FIELD_OFFSET(INFCACHESECTION, Name[nlen]) = offset_of Name + nlen * sizeof(u16)
    let size = offset_of!(_INFCACHESECTION, Name) + nlen * size_of::<u16>();
    let Section = MALLOC(size) as PINFCACHESECTION;
    if Section.is_null() {
        return core::ptr::null_mut();
    }
    ZEROMEMORY(Section as *mut c_void, size);
    (*Cache).NextSectionId = (*Cache).NextSectionId.wrapping_add(1);
    (*Section).Id = (*Cache).NextSectionId;
    wcscpy(
        core::ptr::addr_of_mut!((*Section).Name) as *mut u16,
        Name,
    );
    if (*Cache).FirstSection.is_null() {
        (*Cache).FirstSection = Section;
        (*Cache).LastSection  = Section;
    } else {
        (*(*Cache).LastSection).Next = Section;
        (*Section).Prev = (*Cache).LastSection;
        (*Cache).LastSection = Section;
    }
    Section
}

#[no_mangle]
pub unsafe extern "C" fn InfpAddLine(Section: PINFCACHESECTION) -> PINFCACHELINE {
    if Section.is_null() {
        return core::ptr::null_mut();
    }
    let Line = MALLOC(size_of::<INFCACHELINE>()) as PINFCACHELINE;
    if Line.is_null() {
        return core::ptr::null_mut();
    }
    ZEROMEMORY(Line as *mut c_void, size_of::<INFCACHELINE>());
    (*Section).NextLineId = (*Section).NextLineId.wrapping_add(1);
    (*Line).Id = (*Section).NextLineId;
    if (*Section).FirstLine.is_null() {
        (*Section).FirstLine = Line;
        (*Section).LastLine  = Line;
    } else {
        (*(*Section).LastLine).Next = Line;
        (*Line).Prev = (*Section).LastLine;
        (*Section).LastLine = Line;
    }
    (*Section).LineCount = (*Section).LineCount.wrapping_add(1);
    Line
}

#[no_mangle]
pub unsafe extern "C" fn InfpFindSectionById(Cache: PINFCACHE, Id: UINT) -> PINFCACHESECTION {
    let mut Section = (*Cache).FirstSection;
    while !Section.is_null() {
        if (*Section).Id == Id {
            return Section;
        }
        Section = (*Section).Next;
    }
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn InfpGetSectionForContext(Context: PINFCONTEXT) -> PINFCACHESECTION {
    if Context.is_null() {
        return core::ptr::null_mut::<_INFCACHESECTION>();
    }
    // FIXED: was null_mut::<PINFCACHE>() (double-pointer); correct: null_mut::<_INFCACHE>()
    let Cache: PINFCACHE = (*Context).Inf;
    if Cache.is_null() {
        return core::ptr::null_mut::<_INFCACHESECTION>();
    }
    InfpFindSectionById(Cache, (*Context).Section)
}

#[no_mangle]
pub unsafe extern "C" fn InfpFindLineById(Section: PINFCACHESECTION, Id: UINT) -> PINFCACHELINE {
    let mut Line = (*Section).FirstLine;
    while !Line.is_null() {
        if (*Line).Id == Id {
            return Line;
        }
        Line = (*Line).Next;
    }
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn InfpGetLineForContext(Context: PINFCONTEXT) -> PINFCACHELINE {
    let Section = InfpGetSectionForContext(Context);
    if Section.is_null() {
        return core::ptr::null_mut();
    }
    InfpFindLineById(Section, (*Context).Line)
}

#[no_mangle]
pub unsafe extern "C" fn InfpAddKeyToLine(Line: PINFCACHELINE, Key: PCWSTR) -> PVOID {
    if Line.is_null() {
        DbgPrint(b"InfpAddKeyToLine: Invalid Line\n\0".as_ptr());
        return core::ptr::null_mut();
    }
    if !(*Line).Key.is_null() {
        DbgPrint(b"InfpAddKeyToLine: Line already has a key\n\0".as_ptr());
        return core::ptr::null_mut();
    }
    let klen = wcslen(Key) + 1;
    (*Line).Key = MALLOC(klen * size_of::<u16>()) as PWCHAR;
    if (*Line).Key.is_null() {
        DbgPrint(b"InfpAddKeyToLine: MALLOC() failed\n\0".as_ptr());
        return core::ptr::null_mut();
    }
    wcscpy((*Line).Key, Key);
    (*Line).Key as PVOID
}

/// FIXED: offset_of!(INFCACHEFIELD, Data[wcslen+1]) → proper arithmetic.
#[no_mangle]
pub unsafe extern "C" fn InfpAddFieldToLine(Line: PINFCACHELINE, Data: PCWSTR) -> PVOID {
    // FIELD_OFFSET(INFCACHEFIELD, Data[wcslen(Data)+1])
    let dlen = wcslen(Data);
    let size = offset_of!(_INFCACHEFIELD, Data) + (dlen + 1) * size_of::<u16>();
    let Field = MALLOC(size) as PINFCACHEFIELD;
    if Field.is_null() {
        DbgPrint(b"InfpAddFieldToLine: MALLOC() failed\n\0".as_ptr());
        return core::ptr::null_mut();
    }
    ZEROMEMORY(Field as *mut c_void, size);
    wcscpy(
        core::ptr::addr_of_mut!((*Field).Data) as *mut u16,
        Data,
    );
    if (*Line).FirstField.is_null() {
        (*Line).FirstField = Field;
        (*Line).LastField  = Field;
    } else {
        (*(*Line).LastField).Next = Field;
        (*Field).Prev = (*Line).LastField;
        (*Line).LastField = Field;
    }
    (*Line).FieldCount = (*Line).FieldCount.wrapping_add(1);
    Field as PVOID
}

#[no_mangle]
pub unsafe extern "C" fn InfpFindKeyLine(Section: PINFCACHESECTION, Key: PCWSTR) -> PINFCACHELINE {
    let mut Line = (*Section).FirstLine;
    while !Line.is_null() {
        if !(*Line).Key.is_null() && _wcsicmp((*Line).Key, Key) == 0 {
            return Line;
        }
        Line = (*Line).Next;
    }
    core::ptr::null_mut()
}

// ──────────────────────────────────────────────────────────────────────────────
// Parser helpers (push/pop/set state, is_eof, is_eol, push_token)
// ──────────────────────────────────────────────────────────────────────────────

#[inline(always)]
unsafe fn push_state(p: *mut parser, state: parser_state) {
    let idx = (*p).stack_pos as usize;
    (*p).stack[idx] = state;
    (*p).stack_pos += 1;
}
#[inline(always)]
unsafe fn pop_state(p: *mut parser) {
    (*p).stack_pos -= 1;
    (*p).state = (*p).stack[(*p).stack_pos as usize];
}
#[inline(always)]
unsafe fn set_state(p: *mut parser, state: parser_state) -> parser_state {
    let ret = (*p).state;
    (*p).state = state;
    ret
}
#[inline(always)]
unsafe fn is_eof(p: *mut parser, ptr: *const u16) -> bool {
    ptr >= (*p).end || *ptr == CONTROL_Z || *ptr == 0
}
#[inline(always)]
unsafe fn is_eol(p: *mut parser, ptr: *const u16) -> bool {
    ptr >= (*p).end
        || *ptr == CONTROL_Z
        || *ptr == 0x0A
        || (*ptr == 0x0D && *ptr.add(1) == 0x0A)
        || *ptr == 0
}

unsafe fn push_token(p: *mut parser, pos: *const u16) -> i32 {
    let mut len = pos.offset_from((*p).start) as u32;
    let mut src = (*p).start;
    let dst_base = (*p).token.as_mut_ptr().add((*p).token_len as usize);
    let room = MAX_FIELD_LEN.wrapping_sub((*p).token_len);
    if len > room { len = room; }
    (*p).token_len += len;
    let mut dst = dst_base;
    for _ in 0..len {
        *dst = if *src != 0 { *src } else { 0x20 };
        dst = dst.add(1);
        src = src.add(1);
    }
    *dst = 0;
    (*p).start = pos;
    0
}

unsafe fn add_section_from_token(p: *mut parser) -> *mut c_void {
    if (*p).token_len > MAX_SECTION_NAME_LEN {
        (*p).error = INF_STATUS_SECTION_NAME_TOO_LONG;
        return core::ptr::null_mut();
    }
    let mut Section = InfpFindSection((*p).file, (*p).token.as_ptr());
    if Section.is_null() {
        Section = InfpAddSection((*p).file, (*p).token.as_ptr());
        if Section.is_null() {
            (*p).error = INF_STATUS_NOT_ENOUGH_MEMORY;
            return core::ptr::null_mut();
        }
    }
    (*p).token_len = 0;
    (*p).cur_section = Section;
    Section as *mut c_void
}

unsafe fn add_field_from_token(p: *mut parser, is_key: i32) -> *mut c_void {
    if (*p).line.is_null() {
        if (*p).cur_section.is_null() {
            (*p).error = INF_STATUS_WRONG_INF_STYLE;
            return core::ptr::null_mut();
        }
        (*p).line = InfpAddLine((*p).cur_section);
        if (*p).line.is_null() {
            (*p).error = INF_STATUS_NOT_ENOUGH_MEMORY;
            return core::ptr::null_mut();
        }
    }
    let field = if is_key != 0 {
        InfpAddKeyToLine((*p).line, (*p).token.as_ptr())
    } else {
        InfpAddFieldToLine((*p).line, (*p).token.as_ptr())
    };
    if !field.is_null() {
        (*p).token_len = 0;
        return field;
    }
    (*p).error = INF_STATUS_NOT_ENOUGH_MEMORY;
    core::ptr::null_mut()
}

unsafe fn close_current_line(p: *mut parser) {
    (*p).line = core::ptr::null_mut();
}

// ──────────────────────────────────────────────────────────────────────────────
// Parser state functions
// ──────────────────────────────────────────────────────────────────────────────

unsafe extern "C" fn line_start_state(p: *mut parser, pos: *const u16) -> *const u16 {
    let mut ptr = pos;
    while !is_eof(p, ptr) {
        match *ptr {
            0x0D => {} // CR: skip
            0x0A => {
                (*p).line_pos = (*p).line_pos.wrapping_add(1);
                close_current_line(p);
            }
            0x3B => {
                // ';' — comment
                push_state(p, LINE_START);
                set_state(p, COMMENT);
                return ptr.add(1);
            }
            0x5B => {
                // '[' — section start
                (*p).start = ptr.add(1);
                set_state(p, SECTION_NAME);
                return ptr.add(1);
            }
            c => {
                // FIXED: was iswctype(*p, _SPACE=0); now correct isspaceW
                if !isspaceW(c) {
                    (*p).start = ptr;
                    set_state(p, KEY_NAME);
                    return ptr;
                }
            }
        }
        ptr = ptr.add(1);
    }
    close_current_line(p);
    core::ptr::null()
}

unsafe extern "C" fn section_name_state(p: *mut parser, pos: *const u16) -> *const u16 {
    let mut ptr = pos;
    while !is_eol(p, ptr) {
        if *ptr == b']' as u16 {
            push_token(p, ptr);
            if add_section_from_token(p).is_null() {
                return core::ptr::null();
            }
            push_state(p, LINE_START);
            set_state(p, COMMENT);
            return ptr.add(1);
        }
        ptr = ptr.add(1);
    }
    (*p).error = INF_STATUS_BAD_SECTION_NAME_LINE;
    core::ptr::null()
}

unsafe extern "C" fn key_name_state(p: *mut parser, pos: *const u16) -> *const u16 {
    let mut ptr = pos;
    let mut token_end = (*p).start;
    while !is_eol(p, ptr) {
        if *ptr == b',' as u16 { break; }
        match *ptr {
            0x3D => {
                // '='
                push_token(p, token_end);
                if add_field_from_token(p, 1).is_null() { return core::ptr::null(); }
                (*p).start = ptr.add(1);
                push_state(p, VALUE_NAME);
                set_state(p, LEADING_SPACES);
                return ptr.add(1);
            }
            0x3B => {
                // ';'
                push_token(p, token_end);
                if add_field_from_token(p, 0).is_null() { return core::ptr::null(); }
                push_state(p, LINE_START);
                set_state(p, COMMENT);
                return ptr.add(1);
            }
            0x22 => {
                // '"'
                push_token(p, token_end);
                (*p).start = ptr.add(1);
                push_state(p, KEY_NAME);
                set_state(p, QUOTES);
                return ptr.add(1);
            }
            0x5C => {
                // '\'
                push_token(p, token_end);
                (*p).start = ptr;
                push_state(p, KEY_NAME);
                set_state(p, EOL_BACKSLASH);
                return ptr;
            }
            c => {
                if !isspaceW(c) {
                    token_end = ptr.add(1);
                } else {
                    push_token(p, ptr);
                    push_state(p, KEY_NAME);
                    set_state(p, TRAILING_SPACES);
                    return ptr;
                }
            }
        }
        ptr = ptr.add(1);
    }
    push_token(p, token_end);
    set_state(p, VALUE_NAME);
    ptr
}

unsafe extern "C" fn value_name_state(p: *mut parser, pos: *const u16) -> *const u16 {
    let mut ptr = pos;
    let mut token_end = (*p).start;
    while !is_eol(p, ptr) {
        match *ptr {
            0x3B => {
                // ';'
                push_token(p, token_end);
                if add_field_from_token(p, 0).is_null() { return core::ptr::null(); }
                push_state(p, LINE_START);
                set_state(p, COMMENT);
                return ptr.add(1);
            }
            0x2C => {
                // ','
                push_token(p, token_end);
                if add_field_from_token(p, 0).is_null() { return core::ptr::null(); }
                (*p).start = ptr.add(1);
                push_state(p, VALUE_NAME);
                set_state(p, LEADING_SPACES);
                return ptr.add(1);
            }
            0x22 => {
                // '"'
                push_token(p, token_end);
                (*p).start = ptr.add(1);
                push_state(p, VALUE_NAME);
                set_state(p, QUOTES);
                return ptr.add(1);
            }
            0x5C => {
                // '\'
                push_token(p, token_end);
                (*p).start = ptr;
                push_state(p, VALUE_NAME);
                set_state(p, EOL_BACKSLASH);
                return ptr;
            }
            c => {
                if !isspaceW(c) {
                    token_end = ptr.add(1);
                } else {
                    push_token(p, ptr);
                    push_state(p, VALUE_NAME);
                    set_state(p, TRAILING_SPACES);
                    return ptr;
                }
            }
        }
        ptr = ptr.add(1);
    }
    push_token(p, token_end);
    if add_field_from_token(p, 0).is_null() { return core::ptr::null(); }
    set_state(p, LINE_START);
    ptr
}

unsafe extern "C" fn eol_backslash_state(p: *mut parser, pos: *const u16) -> *const u16 {
    let mut ptr = pos;
    while !is_eof(p, ptr) {
        match *ptr {
            0x0A => {
                (*p).line_pos = (*p).line_pos.wrapping_add(1);
                (*p).start = ptr.add(1);
                set_state(p, LEADING_SPACES);
                return ptr.add(1);
            }
            0x0D | 0x5C => {}
            0x3B => {
                push_state(p, EOL_BACKSLASH);
                set_state(p, COMMENT);
                return ptr.add(1);
            }
            c => {
                if !isspaceW(c) {
                    push_token(p, ptr);
                    pop_state(p);
                    return ptr;
                }
            }
        }
        ptr = ptr.add(1);
    }
    (*p).start = ptr;
    pop_state(p);
    ptr
}

unsafe extern "C" fn quotes_state(p: *mut parser, pos: *const u16) -> *const u16 {
    let mut ptr = pos;
    while !is_eol(p, ptr) {
        if *ptr == b'"' as u16 {
            if ptr.add(1) < (*p).end && *ptr.add(1) == b'"' as u16 {
                push_token(p, ptr.add(1));
                (*p).start = ptr.add(2);
                ptr = ptr.add(1);
            } else {
                push_token(p, ptr);
                (*p).start = ptr.add(1);
                pop_state(p);
                return ptr.add(1);
            }
        }
        ptr = ptr.add(1);
    }
    push_token(p, ptr);
    pop_state(p);
    ptr
}

/// FIXED: was using `iswctype(*p, _SPACE=0)` (always 0 = never matches whitespace).
/// Now uses correct isspaceW inline.
unsafe extern "C" fn leading_spaces_state(p: *mut parser, pos: *const u16) -> *const u16 {
    let mut ptr = pos;
    while !is_eol(p, ptr) {
        if *ptr == b'\\' as u16 {
            (*p).start = ptr;
            set_state(p, EOL_BACKSLASH);
            return ptr;
        }
        if !isspaceW(*ptr) {
            break;
        }
        ptr = ptr.add(1);
    }
    (*p).start = ptr;
    pop_state(p);
    ptr
}

unsafe extern "C" fn trailing_spaces_state(p: *mut parser, pos: *const u16) -> *const u16 {
    let mut ptr = pos;
    while !is_eol(p, ptr) {
        if *ptr == b'\\' as u16 {
            set_state(p, EOL_BACKSLASH);
            return ptr;
        }
        if !isspaceW(*ptr) {
            break;
        }
        ptr = ptr.add(1);
    }
    pop_state(p);
    ptr
}

unsafe extern "C" fn comment_state(p: *mut parser, pos: *const u16) -> *const u16 {
    let mut ptr = pos;
    while !is_eol(p, ptr) {
        ptr = ptr.add(1);
    }
    pop_state(p);
    ptr
}

// ──────────────────────────────────────────────────────────────────────────────
// Main parser entry
// ──────────────────────────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn InfpParseBuffer(
    file: PINFCACHE,
    buffer: *const u16,
    end: *const u16,
    error_line: PULONG,
) -> INFSTATUS {
    let mut p = parser {
        start:       buffer,
        end,
        file,
        state:       LINE_START,
        stack:       [LINE_START; 4],
        stack_pos:   0,
        cur_section: core::ptr::null_mut(),
        line:        core::ptr::null_mut(),
        line_pos:    1,
        error:       0,
        token_len:   0,
        token:       [0u16; MAX_FIELD_LEN as usize + 1],
    };

    let mut pos: *const u16 = buffer;
    while !pos.is_null() {
        pos = parser_funcs[p.state as usize](&mut p, pos);
    }

    if p.error != 0 {
        if !error_line.is_null() {
            *error_line = p.line_pos;
        }
        return p.error;
    }

    // Find the [Strings] section
    let strings_w: [u16; 8] = [b'S' as u16, b't' as u16, b'r' as u16, b'i' as u16,
                                b'n' as u16, b'g' as u16, b's' as u16, 0u16];
    (*file).StringsSection = InfpFindSection(file, strings_w.as_ptr());
    INF_STATUS_SUCCESS
}
