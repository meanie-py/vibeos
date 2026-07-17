use ntkrnl_sys::base::*;
use crate::frldr::*;
use ntkrnl_sys::ddk::{ARC_STATUS, FILEINFORMATION, PCCHAR, PCSTR, PSTR};
// freeldr_ddk_imports dropped (not in ddk): Next
use core::ffi::c_void;
pub use ntkrnl_sys::ddk::_ARC_CODES_ESUCCESS as ESUCCESS;
pub use ntkrnl_sys::ddk::_OPENMODE_OpenReadOnly as OpenReadOnly;

extern "C" {
    fn _stricmp(s1: PCSTR, s2: PCSTR) -> i32;
    fn memset(s: *mut core::ffi::c_void, c: i32, n: usize) -> *mut core::ffi::c_void;
    fn strcpy(dest: *mut i8, src: *const i8) -> *mut i8;
    fn strlen(s: *const i8) -> usize;
    fn atoi(s: *const i8) -> i32;
    fn isspace(c: i32) -> i32;
    fn ArcClose(FileId: ULONG) -> ARC_STATUS;
    fn ArcGetFileInformation(FileId: ULONG, Information: *mut FILEINFORMATION) -> ARC_STATUS;
    fn ArcOpen(FileName: PSTR, OpenMode: ULONG, FileId: *mut ULONG) -> ARC_STATUS;
    fn ArcRead(FileId: ULONG, Buffer: *mut core::ffi::c_void, Length: ULONG, Count: *mut ULONG) -> ARC_STATUS;
}

// ---- rustconv dropped file-local content from inffile.c / inffile.h, restored faithfully ----

pub const STATUS_BAD_SECTION_NAME_LINE: ULONG = 0xC0700001;
pub const STATUS_SECTION_NAME_TOO_LONG: ULONG = 0xC0700002;
pub const STATUS_WRONG_INF_STYLE: ULONG = 0xC0700003;
pub const STATUS_NOT_ENOUGH_MEMORY: ULONG = 0xC0700004;

pub const MAX_INF_STRING_LENGTH: usize = 512;

pub const CONTROL_Z: CHAR = 0x1a;
pub const MAX_SECTION_NAME_LEN: ULONG = 255;
pub const MAX_FIELD_LEN: ULONG = 511;
pub const MAX_STRING_LEN: usize = MAX_INF_STRING_LENGTH + 1;

// C multi-char literals are big-endian: 'KfnI' => bytes K,f,n,I read as a 32-bit BE int.
pub const TAG_INF_KEY: ULONG = u32::from_be_bytes(*b"KfnI");
pub const TAG_INF_FIELD: ULONG = u32::from_be_bytes(*b"ffnI");
pub const TAG_INF_LINE: ULONG = u32::from_be_bytes(*b"LfnI");
pub const TAG_INF_SECTION: ULONG = u32::from_be_bytes(*b"SfnI");
pub const TAG_INF_CACHE: ULONG = u32::from_be_bytes(*b"CfnI");
pub const TAG_INF_FILE: ULONG = u32::from_be_bytes(*b"FfnI");

#[repr(C)]
pub struct _INFCACHEFIELD {
    pub Next: *mut _INFCACHEFIELD,
    pub Prev: *mut _INFCACHEFIELD,
    pub Data: [CHAR; 1],
}
pub type INFCACHEFIELD = _INFCACHEFIELD;
pub type PINFCACHEFIELD = *mut _INFCACHEFIELD;

#[repr(C)]
pub struct _INFCACHELINE {
    pub Next: *mut _INFCACHELINE,
    pub Prev: *mut _INFCACHELINE,
    pub FieldCount: ULONG,
    pub Key: PCHAR,
    pub FirstField: PINFCACHEFIELD,
    pub LastField: PINFCACHEFIELD,
}
pub type INFCACHELINE = _INFCACHELINE;
pub type PINFCACHELINE = *mut _INFCACHELINE;

#[repr(C)]
pub struct _INFCACHESECTION {
    pub Next: *mut _INFCACHESECTION,
    pub Prev: *mut _INFCACHESECTION,
    pub FirstLine: PINFCACHELINE,
    pub LastLine: PINFCACHELINE,
    pub LineCount: LONG,
    pub Name: [CHAR; 1],
}
pub type INFCACHESECTION = _INFCACHESECTION;
pub type PINFCACHESECTION = *mut _INFCACHESECTION;

#[repr(C)]
pub struct _INFCACHE {
    pub FirstSection: PINFCACHESECTION,
    pub LastSection: PINFCACHESECTION,
    pub StringsSection: PINFCACHESECTION,
}
pub type INFCACHE = _INFCACHE;
pub type PINFCACHE = *mut _INFCACHE;

// typedef PULONG HINF, *PHINF;
pub type HINF = PULONG;
pub type PHINF = *mut PULONG;

#[repr(C)]
pub struct _INFCONTEXT {
    pub Inf: PVOID,
    pub CurrentInf: PVOID,
    pub Section: PVOID,
    pub Line: PVOID,
}
pub type INFCONTEXT = _INFCONTEXT;
pub type PINFCONTEXT = *mut _INFCONTEXT;

pub type parser_state = i32;
pub const parser_state_LINE_START: parser_state = 0;
pub const parser_state_SECTION_NAME: parser_state = 1;
pub const parser_state_KEY_NAME: parser_state = 2;
pub const parser_state_VALUE_NAME: parser_state = 3;
pub const parser_state_EOL_BACKSLASH: parser_state = 4;
pub const parser_state_QUOTES: parser_state = 5;
pub const parser_state_LEADING_SPACES: parser_state = 6;
pub const parser_state_TRAILING_SPACES: parser_state = 7;
pub const parser_state_COMMENT: parser_state = 8;
pub const parser_state_NB_PARSER_STATES: parser_state = 9;

// bare aliases (rustconv emits both prefixed and unprefixed forms)
pub const LINE_START: parser_state = parser_state_LINE_START;
pub const SECTION_NAME: parser_state = parser_state_SECTION_NAME;
pub const KEY_NAME: parser_state = parser_state_KEY_NAME;
pub const VALUE_NAME: parser_state = parser_state_VALUE_NAME;
pub const EOL_BACKSLASH: parser_state = parser_state_EOL_BACKSLASH;
pub const QUOTES: parser_state = parser_state_QUOTES;
pub const LEADING_SPACES: parser_state = parser_state_LEADING_SPACES;
pub const TRAILING_SPACES: parser_state = parser_state_TRAILING_SPACES;
pub const COMMENT: parser_state = parser_state_COMMENT;
pub const NB_PARSER_STATES: usize = 9;

#[repr(C)]
pub struct parser {
    pub start: *const CHAR,
    pub end: *const CHAR,
    pub file: PINFCACHE,
    pub state: parser_state,
    pub stack: [parser_state; 4],
    pub stack_pos: i32,
    pub cur_section: PINFCACHESECTION,
    pub line: PINFCACHELINE,
    pub line_pos: u32,
    pub error: u32,
    pub token_len: u32,
    pub token: [CHAR; (MAX_FIELD_LEN as usize) + 1],
}

pub type parser_state_func =
    unsafe extern "C" fn(parser: *mut parser, pos: *const CHAR) -> *const CHAR;

#[no_mangle]
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

#[inline]

unsafe fn InfpCacheFreeLine(
    Line: PINFCACHELINE
) -> PINFCACHELINE
{
    let Next: PINFCACHELINE;
    let mut Field: PINFCACHEFIELD;

    if Line == core::ptr::null_mut() {
        return core::ptr::null_mut();
    }

    Next = (*Line).Next;
    if (*Line).Key != core::ptr::null_mut() {
        FrLdrTempFree((*Line).Key as *mut core::ffi::c_void, TAG_INF_KEY);
        (*Line).Key = core::ptr::null_mut();
    }

    while (*Line).FirstField != core::ptr::null_mut() {
        Field = (*(*Line).FirstField).Next;
        FrLdrTempFree((*Line).FirstField as *mut core::ffi::c_void, TAG_INF_FIELD);
        (*Line).FirstField = Field;
    }
    (*Line).LastField = core::ptr::null_mut();

    FrLdrTempFree(Line as *mut core::ffi::c_void, TAG_INF_LINE);

    return Next;
}

unsafe fn InfpCacheFreeSection(
    Section: PINFCACHESECTION
) -> PINFCACHESECTION
{
    let Next: PINFCACHESECTION;

    if Section == core::ptr::null_mut() {
        return core::ptr::null_mut();
    }

    Next = (*Section).Next;
    while (*Section).FirstLine != core::ptr::null_mut() {
        (*Section).FirstLine = InfpCacheFreeLine((*Section).FirstLine);
    }
    (*Section).LastLine = core::ptr::null_mut();

    FrLdrTempFree(Section as *mut core::ffi::c_void, TAG_INF_SECTION);

    return Next;
}

unsafe fn InfpCacheFindSection(
    Cache: PINFCACHE,
    Name: PCSTR,
) -> PINFCACHESECTION {
    let mut Section: PINFCACHESECTION = core::ptr::null_mut();

    if Cache.is_null() || Name.is_null() {
        return core::ptr::null_mut();
    }

    Section = (*Cache).FirstSection;
    while !Section.is_null() {
        if _stricmp((*Section).Name.as_ptr(), Name) == 0 {
            return Section;
        }

        Section = (*Section).Next;
    }

    core::ptr::null_mut()
}

unsafe fn InfpCacheAddSection(
    Cache: PINFCACHE,
    Name: PCSTR,
) -> PINFCACHESECTION {
    let mut Section: PINFCACHESECTION = core::ptr::null_mut();
    let Size: usize;

    if Cache.is_null() || Name.is_null() {
        return core::ptr::null_mut();
    }

    Size = core::mem::size_of::<INFCACHESECTION>() + strlen(Name);
    Section = FrLdrTempAlloc(Size, TAG_INF_SECTION) as PINFCACHESECTION;
    if Section.is_null() {
        return core::ptr::null_mut();
    }
    memset(Section as *mut core::ffi::c_void, 0, Size);

    strcpy((*Section).Name.as_mut_ptr(), Name);

    if (*Cache).FirstSection.is_null() {
        (*Cache).FirstSection = Section;
        (*Cache).LastSection = Section;
    } else {
        (*(*Cache).LastSection).Next = Section;
        (*Section).Prev = (*Cache).LastSection;
        (*Cache).LastSection = Section;
    }

    Section
}

unsafe fn InfpCacheAddLine(Section: PINFCACHESECTION) -> PINFCACHELINE {

    if Section.is_null() {
        return core::ptr::null_mut();
    }

    let Line = FrLdrTempAlloc(core::mem::size_of::<INFCACHELINE>(), TAG_INF_LINE) as PINFCACHELINE;
    if Line.is_null() {
        return core::ptr::null_mut();
    }
    memset(Line as *mut core::ffi::c_void, 0, core::mem::size_of::<INFCACHELINE>());

    if (*Section).FirstLine.is_null() {
        (*Section).FirstLine = Line;
        (*Section).LastLine = Line;
    } else {
        (*(*Section).LastLine).Next = Line;
        (*Line).Prev = (*Section).LastLine;
        (*Section).LastLine = Line;
    }
    (*Section).LineCount += 1;

    Line
}

unsafe fn InfpAddKeyToLine(
    Line: PINFCACHELINE,
    Key: PCSTR,
) -> PVOID {
    if Line.is_null() {
        return core::ptr::null_mut();
    }

    if !(*Line).Key.is_null() {
        return core::ptr::null_mut();
    }

    (*Line).Key = FrLdrTempAlloc(strlen(Key) + 1, TAG_INF_KEY) as PCHAR;
    if (*Line).Key.is_null() {
        return core::ptr::null_mut();
    }

    strcpy((*Line).Key, Key);

    return (*Line).Key as PVOID;
}

unsafe fn InfpAddFieldToLine(
    Line: PINFCACHELINE,
    Data: PCSTR,
) -> PVOID {
    let Field: PINFCACHEFIELD;
    let Size: usize;

    Size = core::mem::size_of::<INFCACHEFIELD>() + strlen(Data);
    Field = FrLdrTempAlloc(Size, TAG_INF_FIELD) as PINFCACHEFIELD;
    if Field.is_null() {
        return core::ptr::null_mut();
    }
    memset(Field as *mut core::ffi::c_void, 0, Size);

    strcpy((*Field).Data.as_mut_ptr(), Data);

    if (*Line).FirstField.is_null() {
        (*Line).FirstField = Field;
        (*Line).LastField = Field;
    } else {
        (*(*Line).LastField).Next = Field;
        (*Field).Prev = (*Line).LastField;
        (*Line).LastField = Field;
    }
    (*Line).FieldCount += 1;

    Field as PVOID
}

unsafe fn InfpCacheFindKeyLine(
    Section: PINFCACHESECTION,
    Key: PCSTR,
) -> PINFCACHELINE {

    let mut Line: PINFCACHELINE = (*Section).FirstLine;
    while !Line.is_null() {
        if !(*Line).Key.is_null() && _stricmp((*Line).Key, Key) == 0 {
            return Line;
        }

        Line = (*Line).Next;
    }

    core::ptr::null_mut()
}

unsafe fn push_state(parser: *mut parser, state: parser_state) {
    (*parser).stack[(*parser).stack_pos as usize] = state;
    (*parser).stack_pos = (*parser).stack_pos.wrapping_add(1);
}

unsafe fn pop_state(parser: *mut parser) {
    (*parser).stack_pos = (*parser).stack_pos.wrapping_sub(1);
    (*parser).state = (*parser).stack[(*parser).stack_pos as usize];
}

unsafe fn set_state(parser: *mut parser, state: parser_state) -> parser_state {
    let ret = (*parser).state;
    (*parser).state = state;
    ret
}

unsafe fn is_eof(parser: *const parser, ptr: *const CHAR) -> i32 {
    if ptr >= (*parser).end || *ptr == CONTROL_Z {
        1
    } else {
        0
    }
}

unsafe fn is_eol(parser: *mut parser, ptr: *const CHAR) -> i32 {
    if ptr >= (*parser).end ||
       *ptr == CONTROL_Z ||
       *ptr == b'\n' as i8 ||
       (*ptr == b'\r' as i8 && *ptr.offset(1) == b'\n' as i8) {
        1
    } else {
        0
    }
}

unsafe fn push_token(
    parser: *mut parser,
    pos: *const CHAR,
) -> i32 {
    let mut len: SIZE_T = pos.offset_from((*parser).start) as SIZE_T;
    let mut src: *const CHAR = (*parser).start;
    let mut dst: *mut CHAR = (*parser).token.as_mut_ptr().add((*parser).token_len as usize);

    if len > (MAX_FIELD_LEN as SIZE_T).wrapping_sub((*parser).token_len as SIZE_T) {
        len = (MAX_FIELD_LEN as SIZE_T).wrapping_sub((*parser).token_len as SIZE_T);
    }

    (*parser).token_len = ((*parser).token_len as SIZE_T).wrapping_add(len) as ULONG;
    while len > 0 {
        *dst = if *src != 0 { *src } else { b' ' as CHAR };
        len = len.wrapping_sub(1);
        dst = dst.add(1);
        src = src.add(1);
    }
    *dst = 0;
    (*parser).start = pos;

    0
}

unsafe fn add_section_from_token(parser: *mut parser) -> PVOID {
    let mut Section: PINFCACHESECTION;

    if (*parser).token_len > MAX_SECTION_NAME_LEN {
        (*parser).error = (0u8) as u32 ;
        return core::ptr::null_mut();
    }

    Section = InfpCacheFindSection((*parser).file, (*parser).token.as_ptr());
    if Section.is_null() {
        Section = InfpCacheAddSection((*parser).file, (*parser).token.as_ptr());
        if Section.is_null() {
            (*parser).error = (0u8) as u32 ;
            return core::ptr::null_mut();
        }
    }

    (*parser).token_len = 0;
    (*parser).cur_section = Section;

    Section as PVOID
}

unsafe fn add_field_from_token(
    parser: *mut parser,
    is_key: i32,
) -> PVOID {
    let field: PVOID;

    if (*parser).line.is_null() {
        if (*parser).cur_section.is_null() {
            (*parser).error = STATUS_WRONG_INF_STYLE;
            return core::ptr::null_mut();
        }

        (*parser).line = InfpCacheAddLine((*parser).cur_section);
        if (*parser).line.is_null() {
            (*parser).error = 0;
            return core::ptr::null_mut();
        }
    }

    if is_key != 0 {
        field = InfpAddKeyToLine((*parser).line, (*parser).token.as_ptr());
    } else {
        field = InfpAddFieldToLine((*parser).line, (*parser).token.as_ptr());
    }

    if !field.is_null() {
        (*parser).token_len = 0;
        return field;
    }

    (*parser).error = 0;
    core::ptr::null_mut()
}

unsafe fn close_current_line(parser: *mut parser) {
    (*parser).line = core::ptr::null_mut();
}

unsafe extern "C" fn line_start_state(
    parser: *mut parser,
    pos: *const CHAR,
) -> *const CHAR {

    let mut p = pos;
    while is_eof(parser, p) == 0 {
        let c = *p as u8;
        if c == b'\r' {
            p = p.offset(1);
            continue;
        } else if c == b'\n' {
            (*parser).line_pos = (*parser).line_pos.wrapping_add(1);
            close_current_line(parser);
        } else if c == b';' {
            push_state(parser, LINE_START);
            set_state(parser, COMMENT);
            return p.offset(1);
        } else if c == b'[' {
            (*parser).start = p.offset(1);
            set_state(parser, SECTION_NAME);
            return p.offset(1);
        } else {
            if isspace(*p as u8 as i32) == 0 {
                (*parser).start = p;
                set_state(parser, KEY_NAME);
                return p;
            }
        }
        p = p.offset(1);
    }
    close_current_line(parser);
    core::ptr::null()
}

unsafe extern "C" fn section_name_state(
    parser: *mut parser,
    pos: *const CHAR,
) -> *const CHAR {
    let mut p: *const CHAR;

    p = pos;
    while is_eol(parser, p) == 0 {
        if *p == b']' as i8 {
            push_token(parser, p);
            if add_section_from_token(parser).is_null() {
                return core::ptr::null();
            }
            push_state(parser, parser_state_LINE_START);
            set_state(parser, parser_state_COMMENT);
            return p.offset(1);
        }
        p = p.offset(1);
    }
    (*parser).error = 0xC0000341u32;
    core::ptr::null()
}

unsafe extern "C" fn key_name_state(
    parser: *mut parser,
    pos: *const CHAR,
) -> *const CHAR {
    let mut p: *const CHAR;
    let mut token_end: *const CHAR = (*parser).start;

    p = pos;
    while is_eol(parser, p) == 0 {
        if *p == b',' as i8 {
            break;
        }
        match *p as u8 {
            b'=' => {
                push_token(parser, token_end);
                if add_field_from_token(parser, 1).is_null() {
                    return core::ptr::null();
                }
                (*parser).start = p.offset(1);
                push_state(parser, parser_state_VALUE_NAME);
                set_state(parser, parser_state_LEADING_SPACES);
                return p.offset(1);
            }
            b';' => {
                push_token(parser, token_end);
                if add_field_from_token(parser, 0).is_null() {
                    return core::ptr::null();
                }
                push_state(parser, parser_state_LINE_START);
                set_state(parser, parser_state_COMMENT);
                return p.offset(1);
            }
            b'"' => {
                push_token(parser, token_end);
                (*parser).start = p.offset(1);
                push_state(parser, parser_state_KEY_NAME);
                set_state(parser, parser_state_QUOTES);
                return p.offset(1);
            }
            b'\\' => {
                push_token(parser, token_end);
                (*parser).start = p;
                push_state(parser, parser_state_KEY_NAME);
                set_state(parser, parser_state_EOL_BACKSLASH);
                return p;
            }
            _ => {
                if isspace(*p as u8 as i32) == 0 {
                    token_end = p.offset(1);
                } else {
                    push_token(parser, p);
                    push_state(parser, parser_state_KEY_NAME);
                    set_state(parser, parser_state_TRAILING_SPACES);
                    return p;
                }
            }
        }
        p = p.offset(1);
    }
    push_token(parser, token_end);
    set_state(parser, parser_state_VALUE_NAME);
    p
}

unsafe extern "C" fn value_name_state(
    parser: *mut parser,
    pos: *const CHAR,
) -> *const CHAR {

    let mut p: *const CHAR;
    let mut token_end: *const CHAR = (*parser).start;

    p = pos;
    while is_eol(parser, p) == 0 {
        match *p as u8 {
            b';' => {
                push_token(parser, token_end);
                if add_field_from_token(parser, 0).is_null() {
                    return core::ptr::null();
                }
                push_state(parser, parser_state_COMMENT);
                set_state(parser, parser_state_COMMENT);
                return p.offset(1);
            }
            b',' => {
                push_token(parser, token_end);
                if add_field_from_token(parser, 0).is_null() {
                    return core::ptr::null();
                }
                (*parser).start = p.offset(1);
                push_state(parser, parser_state_VALUE_NAME);
                set_state(parser, parser_state_LEADING_SPACES);
                return p.offset(1);
            }
            b'"' => {
                push_token(parser, token_end);
                (*parser).start = p.offset(1);
                push_state(parser, parser_state_VALUE_NAME);
                set_state(parser, parser_state_QUOTES);
                return p.offset(1);
            }
            b'\\' => {
                push_token(parser, token_end);
                (*parser).start = p;
                push_state(parser, parser_state_VALUE_NAME);
                set_state(parser, parser_state_EOL_BACKSLASH);
                return p;
            }
            _ => {
                if isspace(*p as u8 as i32) == 0 {
                    token_end = p.offset(1);
                } else {
                    push_token(parser, p);
                    push_state(parser, parser_state_VALUE_NAME);
                    set_state(parser, parser_state_TRAILING_SPACES);
                    return p;
                }
            }
        }
        p = p.offset(1);
    }
    push_token(parser, token_end);
    if add_field_from_token(parser, 0).is_null() {
        return core::ptr::null();
    }
    set_state(parser, parser_state_LINE_START);
    p
}

unsafe extern "C" fn eol_backslash_state(
    parser: *mut parser,
    pos: *const CHAR,
) -> *const CHAR {
    let mut p: *const CHAR;

    p = pos;
    while is_eof(parser, p) == 0 {
        match *p as u8 {
            b'\r' => {
                p = p.offset(1);
                continue;
            }
            b'\n' => {
                (*parser).line_pos = (*parser).line_pos.wrapping_add(1);
                (*parser).start = p.offset(1);
                set_state(parser, parser_state_LEADING_SPACES);
                return p.offset(1);
            }
            b'\\' => {
                p = p.offset(1);
                continue;
            }
            b';' => {
                push_state(parser, parser_state_EOL_BACKSLASH);
                set_state(parser, parser_state_COMMENT);
                return p.offset(1);
            }
            _ => {
                if isspace(*p as u8 as i32) != 0 {
                    p = p.offset(1);
                    continue;
                }
                push_token(parser, p);
                pop_state(parser);
                return p;
            }
        }
    }
    (*parser).start = p;
    pop_state(parser);

    return p;
}

unsafe extern "C" fn quotes_state(
    parser: *mut parser,
    pos: *const CHAR,
) -> *const CHAR {
    let mut p: *const CHAR;
    let mut token_end: *const CHAR = (*parser).start;

    p = pos;
    while is_eol(parser, p) == 0 {
        if *p == b'"' as i8 {
            if p.offset(1) < (*parser).end && *p.offset(1) == b'"' as i8 {
                push_token(parser, p.offset(1));
                (*parser).start = p.offset(2);
                token_end = p.offset(2);
                p = p.offset(1);
            } else {
                push_token(parser, p);
                (*parser).start = p.offset(1);
                pop_state(parser);
                return p.offset(1);
            }
        }
        p = p.offset(1);
    }
    push_token(parser, p);
    pop_state(parser);
    p
}

unsafe extern "C" fn leading_spaces_state(
    parser: *mut parser,
    pos: *const CHAR
) -> *const CHAR {

    let mut p = pos;
    while is_eol(parser, p) == 0 {
        if *p == b'\\' as i8 {
            (*parser).start = p;
            set_state(parser, parser_state_EOL_BACKSLASH);
            return p;
        }
        if isspace(*p as u8 as i32) == 0 {
            break;
        }
        p = p.offset(1);
    }
    (*parser).start = p;
    pop_state(parser);
    p
}

unsafe extern "C" fn trailing_spaces_state(
    parser: *mut parser,
    pos: *const CHAR
) -> *const CHAR {

    let mut p = pos;
    while is_eol(parser, p) == 0 {
        if *p == b'\\' as i8 {
            set_state(parser, parser_state_EOL_BACKSLASH);
            return p;
        }
        if isspace(*p as u8 as i32) == 0 {
            break;
        }
        p = p.offset(1);
    }
    pop_state(parser);
    p
}

unsafe extern "C" fn comment_state(parser: *mut parser, pos: *const CHAR) -> *const CHAR {
    let mut p = pos;
    
    while is_eol(parser, p) == 0 {
        p = p.offset(1);
    }
    pop_state(parser);
    p
}

unsafe fn InfpParseBuffer(
    file: PINFCACHE,
    buffer: PCCHAR,
    end: PCCHAR,
    error_line: PULONG,
) -> BOOLEAN {
    let mut parser: parser = core::mem::zeroed();
    let mut pos: *const CHAR = buffer as *const CHAR;

    parser.start = buffer;
    parser.end = end;
    parser.file = file;
    parser.line = core::ptr::null_mut();
    parser.state = LINE_START;
    parser.stack_pos = 0;
    parser.cur_section = core::ptr::null_mut();
    parser.line_pos = 1;
    parser.error = (1u8) as u32 ;
    parser.token_len = 0;

    while !pos.is_null() {
        pos = (parser_funcs[parser.state as usize])(&mut parser as *mut parser, pos);
    }

    if parser.error != 0 {
        if !error_line.is_null() {
            *error_line = parser.line_pos;
        }
        return (parser.error) as u8 ;
    }

    (*file).StringsSection = InfpCacheFindSection(file, b"Strings\0".as_ptr() as PCSTR);

    return 1u8;
}

#[no_mangle]
pub unsafe extern "C" fn InfOpenFile(
    InfHandle: PHINF,
    FileName: PCSTR,
    ErrorLine: PULONG,
) -> BOOLEAN {

    let mut Information: FILEINFORMATION = core::mem::zeroed();
    let mut FileId: ULONG = 0;
    let FileBuffer: PCHAR;
    let FileSize: ULONG;
    let mut Count: ULONG = 0;
    let mut Cache: PINFCACHE;
    let Success: BOOLEAN;
    let mut Status: ARC_STATUS;

    *InfHandle = core::ptr::null_mut();
    *ErrorLine = 0xFFFFFFFF_u32;

    Status = ArcOpen(FileName as PSTR, OpenReadOnly, &mut FileId);
    if Status != ESUCCESS {
        return 0;
    }

    Status = ArcGetFileInformation(FileId, &mut Information);
    if (Status != ESUCCESS) || (Information.EndingAddress.__bindgen_anon_1.HighPart != 0) {
        ArcClose(FileId);
        return 0;
    }
    FileSize = Information.EndingAddress.__bindgen_anon_1.LowPart;

    FileBuffer = FrLdrTempAlloc((FileSize + 1) as usize , TAG_INF_FILE) as PCHAR;
    if FileBuffer.is_null() {
        ArcClose(FileId);
        return 0;
    }

    Status = ArcRead(FileId, FileBuffer as *mut core::ffi::c_void, FileSize, &mut Count);
    if (Status != ESUCCESS) || (Count != FileSize) {
        ArcClose(FileId);
        FrLdrTempFree(FileBuffer as *mut core::ffi::c_void, TAG_INF_FILE);
        return 0;
    }

    ArcClose(FileId);

    *FileBuffer.add(FileSize as usize) = 0;

    Cache = FrLdrTempAlloc(core::mem::size_of::<INFCACHE>(), TAG_INF_CACHE) as PINFCACHE;
    if Cache.is_null() {
        FrLdrTempFree(FileBuffer as *mut core::ffi::c_void, TAG_INF_FILE);
        return 0;
    }

    RtlZeroMemory(Cache as *mut core::ffi::c_void, core::mem::size_of::<INFCACHE>());

    Success = InfpParseBuffer(Cache, FileBuffer, FileBuffer.add(FileSize as usize), ErrorLine);
    if Success == 0 {
        FrLdrTempFree(Cache as *mut core::ffi::c_void, TAG_INF_CACHE);
        Cache = core::ptr::null_mut();
    }

    FrLdrTempFree(FileBuffer as *mut core::ffi::c_void, TAG_INF_FILE);

    *InfHandle = Cache as HINF;

    Success
}

#[no_mangle]
pub unsafe extern "C" fn InfCloseFile(InfHandle: HINF) {
    let Cache: PINFCACHE;

    Cache = InfHandle as PINFCACHE;

    if Cache == core::ptr::null_mut() {
        return;
    }

    while (*Cache).FirstSection != core::ptr::null_mut() {
        (*Cache).FirstSection = InfpCacheFreeSection((*Cache).FirstSection);
    }
    (*Cache).LastSection = core::ptr::null_mut();

    FrLdrTempFree(Cache as *mut core::ffi::c_void, TAG_INF_CACHE);
}

#[no_mangle]
pub unsafe extern "C" fn InfFindFirstLine(
    InfHandle: HINF,
    Section: PCSTR,
    Key: PCSTR,
    Context: PINFCONTEXT,
) -> BOOLEAN {

    if InfHandle.is_null() || Section.is_null() || Context.is_null() {
        return 0u8;
    }

    let Cache: PINFCACHE = InfHandle as PINFCACHE;

    let mut CacheSection: PINFCACHESECTION = (*Cache).FirstSection;
    while !CacheSection.is_null() {
        if _stricmp((*CacheSection).Name.as_ptr(), Section) == 0 {
            let CacheLine: PINFCACHELINE = if !Key.is_null() {
                InfpCacheFindKeyLine(CacheSection, Key)
            } else {
                (*CacheSection).FirstLine
            };

            if CacheLine.is_null() {
                return 0u8;
            }

            (*Context).Inf = Cache as PVOID;
            (*Context).Section = CacheSection as PVOID;
            (*Context).Line = CacheLine as PVOID;

            return 1u8;
        }

        CacheSection = (*CacheSection).Next;
    }

    0u8
}

#[no_mangle]
pub unsafe extern "C" fn InfFindNextLine(
    ContextIn: PINFCONTEXT,
    ContextOut: PINFCONTEXT
) -> BOOLEAN {
    let CacheLine: PINFCACHELINE;

    if ContextIn.is_null() || ContextOut.is_null() {
        return 0u8;
    }

    if (*ContextIn).Line.is_null() {
        return 0u8;
    }

    CacheLine = (*ContextIn).Line as PINFCACHELINE;
    if (*CacheLine).Next.is_null() {
        return 0u8;
    }

    if ContextIn != ContextOut {
        (*ContextOut).Inf = (*ContextIn).Inf;
        (*ContextOut).Section = (*ContextIn).Section;
    }
    (*ContextOut).Line = (*CacheLine).Next as PVOID;

    return 1u8;
}

#[no_mangle]
pub unsafe extern "C" fn InfFindFirstMatchLine(
    ContextIn: PINFCONTEXT,
    Key: PCSTR,
    ContextOut: PINFCONTEXT
) -> BOOLEAN {

    if ContextIn.is_null() || ContextOut.is_null() || Key.is_null() || *Key == 0 {
        return 0u8;
    }

    if (*ContextIn).Inf.is_null() || (*ContextIn).Section.is_null() {
        return 0u8;
    }

    let mut CacheLine = (*((*ContextIn).Section as PINFCACHESECTION)).FirstLine;
    while !CacheLine.is_null() {
        if !(*CacheLine).Key.is_null() && _stricmp((*CacheLine).Key, Key) == 0 {
            if ContextIn != ContextOut {
                (*ContextOut).Inf = (*ContextIn).Inf;
                (*ContextOut).Section = (*ContextIn).Section;
            }
            (*ContextOut).Line = CacheLine as PVOID;

            return 1u8;
        }

        CacheLine = (*CacheLine).Next;
    }

    0u8
}

#[no_mangle]
pub unsafe extern "C" fn InfFindNextMatchLine(
    ContextIn: PINFCONTEXT,
    Key: PCSTR,
    ContextOut: PINFCONTEXT
) -> BOOLEAN {

    if ContextIn.is_null() || ContextOut.is_null() || Key.is_null() || *Key == 0 {
        return 0u8;
    }

    if (*ContextIn).Inf.is_null() || (*ContextIn).Section.is_null() || (*ContextIn).Line.is_null() {
        return 0u8;
    }

    let mut CacheLine = (*ContextIn).Line as PINFCACHELINE;
    while !CacheLine.is_null() {
        if !(*CacheLine).Key.is_null() && _stricmp((*CacheLine).Key, Key) == 0 {
            if ContextIn != ContextOut {
                (*ContextOut).Inf = (*ContextIn).Inf;
                (*ContextOut).Section = (*ContextIn).Section;
            }
            (*ContextOut).Line = CacheLine as PVOID;

            return 1u8;
        }

        CacheLine = (*CacheLine).Next;
    }

    0u8
}

#[no_mangle]
pub unsafe extern "C" fn InfGetLineCount(
    InfHandle: HINF,
    Section: PCSTR,
) -> LONG {
    let Cache: PINFCACHE;
    let mut CacheSection: PINFCACHESECTION;

    if InfHandle.is_null() || Section.is_null() {
        return -1;
    }

    Cache = InfHandle as PINFCACHE;

    CacheSection = (*Cache).FirstSection;
    while !CacheSection.is_null() {
        if _stricmp((*CacheSection).Name.as_ptr(), Section) == 0 {
            return (*CacheSection).LineCount;
        }

        CacheSection = (*CacheSection).Next;
    }

    return -1;
}

#[no_mangle]
pub unsafe extern "C" fn InfGetFieldCount(Context: PINFCONTEXT) -> LONG {
    if Context.is_null() || (*Context).Line.is_null() {
        return 0;
    }

    ((*((*Context).Line as PINFCACHELINE)).FieldCount) as i32
}

#[no_mangle]
pub unsafe extern "C" fn InfGetBinaryField(
    Context: PINFCONTEXT,
    FieldIndex: ULONG,
    ReturnBuffer: PUCHAR,
    ReturnBufferSize: ULONG,
    RequiredSize: PULONG,
) -> BOOLEAN {

    let CacheLine: PINFCACHELINE;
    let mut CacheField: PINFCACHEFIELD;
    let mut Index: ULONG;
    let Size: ULONG;
    let mut Ptr: PUCHAR;

    if Context.is_null() || (*Context).Line.is_null() || FieldIndex == 0 {
        return 0u8;
    }

    if !RequiredSize.is_null() {
        *RequiredSize = 0;
    }

    CacheLine = (*Context).Line as PINFCACHELINE;

    if FieldIndex > (*CacheLine).FieldCount {
        return 0u8;
    }

    CacheField = (*CacheLine).FirstField;
    Index = 1;
    while Index < FieldIndex {
        CacheField = (*CacheField).Next;
        Index += 1;
    }

    Size = (*CacheLine).FieldCount - FieldIndex + 1;

    if !RequiredSize.is_null() {
        *RequiredSize = Size;
    }

    if !ReturnBuffer.is_null() {
        if ReturnBufferSize < Size {
            return 0u8;
        }

        Ptr = ReturnBuffer;
        while !CacheField.is_null() {
            *Ptr = atoi((*CacheField).Data.as_ptr()) as UCHAR;

            Ptr = Ptr.add(1);
            CacheField = (*CacheField).Next;
        }
    }

    1u8
}

#[no_mangle]
pub unsafe extern "C" fn InfGetIntField(
    Context: PINFCONTEXT,
    FieldIndex: ULONG,
    IntegerValue: PLONG,
) -> BOOLEAN {
    let CacheLine: PINFCACHELINE;
    let mut CacheField: PINFCACHEFIELD;
    let mut Index: ULONG;
    let Ptr: PCHAR;

    if Context.is_null() || (*Context).Line.is_null() || IntegerValue.is_null() {
        return 0u8;
    }

    CacheLine = (*Context).Line as PINFCACHELINE;

    if FieldIndex > (*CacheLine).FieldCount {
        return 0u8;
    }

    if FieldIndex == 0 {
        Ptr = (*CacheLine).Key;
    } else {
        CacheField = (*CacheLine).FirstField;
        Index = 1;
        while Index < FieldIndex {
            CacheField = (*CacheField).Next;
            Index += 1;
        }

        Ptr = (*CacheField).Data.as_mut_ptr();
    }

    *IntegerValue = atoi(Ptr);

    return 1u8;
}

#[no_mangle]
pub unsafe extern "C" fn InfGetMultiSzField(
    Context: PINFCONTEXT,
    FieldIndex: ULONG,
    ReturnBuffer: PCHAR,
    ReturnBufferSize: ULONG,
    RequiredSize: PULONG,
) -> BOOLEAN {

    let CacheLine: PINFCACHELINE;
    let mut CacheField: PINFCACHEFIELD;
    let mut FieldPtr: PINFCACHEFIELD;
    let mut Index: ULONG;
    let mut Size: usize;
    let mut Ptr: PCHAR;

    if Context.is_null() || (*Context).Line.is_null() || FieldIndex == 0 {
        return 0u8;
    }

    if !RequiredSize.is_null() {
        *RequiredSize = 0;
    }

    CacheLine = (*Context).Line as PINFCACHELINE;

    if FieldIndex > (*CacheLine).FieldCount {
        return 0u8;
    }

    CacheField = (*CacheLine).FirstField;
    Index = 1;
    while Index < FieldIndex {
        CacheField = (*CacheField).Next;
        Index += 1;
    }

    FieldPtr = CacheField;
    Size = 0;
    while !FieldPtr.is_null() {
        Size += strlen((*FieldPtr).Data.as_ptr()) + 1;
        FieldPtr = (*FieldPtr).Next;
    }
    Size += 1;

    if !RequiredSize.is_null() {
        *RequiredSize = Size as ULONG;
    }

    if !ReturnBuffer.is_null() {
        if (ReturnBufferSize as usize) < Size {
            return 0u8;
        }

        Ptr = ReturnBuffer;
        FieldPtr = CacheField;
        while !FieldPtr.is_null() {
            Size = strlen((*FieldPtr).Data.as_ptr()) + 1;

            strcpy(Ptr, (*FieldPtr).Data.as_ptr());

            Ptr = Ptr.add(Size);
            FieldPtr = (*FieldPtr).Next;
        }
        *Ptr = 0;
    }

    1u8
}

#[no_mangle]
pub unsafe extern "C" fn InfGetStringField(
    Context: PINFCONTEXT,
    FieldIndex: ULONG,
    ReturnBuffer: PCHAR,
    ReturnBufferSize: ULONG,
    RequiredSize: PULONG,
) -> BOOLEAN {

    let CacheLine: PINFCACHELINE;
    let mut CacheField: PINFCACHEFIELD;
    let mut Index: ULONG;
    let Ptr: PCHAR;
    let Size: usize;

    if Context.is_null() || (*Context).Line.is_null() {
        return 0u8;
    }

    if !RequiredSize.is_null() {
        *RequiredSize = 0;
    }

    CacheLine = (*Context).Line as PINFCACHELINE;

    if FieldIndex > (*CacheLine).FieldCount {
        return 0u8;
    }

    if FieldIndex == 0 {
        Ptr = (*CacheLine).Key;
    } else {
        CacheField = (*CacheLine).FirstField;
        Index = 1;
        while Index < FieldIndex {
            CacheField = (*CacheField).Next;
            Index += 1;
        }

        Ptr = (*CacheField).Data.as_mut_ptr();
    }

    Size = strlen(Ptr) + 1;

    if !RequiredSize.is_null() {
        *RequiredSize = Size as ULONG;
    }

    if !ReturnBuffer.is_null() {
        if ReturnBufferSize < Size as ULONG {
            return 0u8;
        }

        strcpy(ReturnBuffer, Ptr);
    }

    1u8
}

#[no_mangle]
pub unsafe extern "C" fn InfGetData(
    Context: PINFCONTEXT,
    Key: *mut PCSTR,
    Data: *mut PCSTR,
) -> BOOLEAN {
    if Context.is_null() || (*Context).Line.is_null() || Data.is_null() {
        return 0u8;
    }

    let CacheKey = (*Context).Line as PINFCACHELINE;
    if !Key.is_null() {
        *Key = (*CacheKey).Key;
    }

    *Data = if (*CacheKey).FirstField.is_null() {
        core::ptr::null()
    } else {
        (*(*CacheKey).FirstField).Data.as_ptr()
    };

    1u8
}

#[no_mangle]
pub unsafe extern "C" fn InfGetDataField(
    Context: PINFCONTEXT,
    FieldIndex: u32,
    Data: *mut PCSTR,
) -> u8 {
    let CacheLine: PINFCACHELINE;
    let mut CacheField: PINFCACHEFIELD;
    let mut Index: u32;

    if Context.is_null() || (*Context).Line.is_null() || Data.is_null() {
        return 0u8;
    }

    CacheLine = (*Context).Line as PINFCACHELINE;

    if FieldIndex > (*CacheLine).FieldCount {
        return 0u8;
    }

    if FieldIndex == 0 {
        *Data = (*CacheLine).Key;
    } else {
        CacheField = (*CacheLine).FirstField;
        Index = 1;
        while Index < FieldIndex {
            CacheField = (*CacheField).Next;
            Index += 1;
        }

        *Data = (*CacheField).Data.as_ptr();
    }

    return 1u8;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     FreeLoader
//  * LICENSE:     GPL-2.0+ (https://spdx.org/licenses/GPL-2.0+)
//  * PURPOSE:     INF file parser that caches contents of INF file in memory.
//  * COPYRIGHT:   Copyright 2002-2006 Royce Mitchell III
//  *              Copyright 2003-2019 Eric Kohl
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include <freeldr.h>
// #include "inffile.h"
//
// #define CONTROL_Z  '\x1a'
// #define MAX_SECTION_NAME_LEN  255
// #define MAX_FIELD_LEN         511  /* larger fields get silently truncated */
// /* actual string limit is MAX_INF_STRING_LENGTH+1 (plus terminating null) under Windows */
// #define MAX_STRING_LEN        (MAX_INF_STRING_LENGTH+1)
//
// #define TAG_INF_KEY 'KfnI'
// #define TAG_INF_FIELD 'ffnI'
// #define TAG_INF_LINE 'LfnI'
// #define TAG_INF_SECTION 'SfnI'
// #define TAG_INF_CACHE 'CfnI'
// #define TAG_INF_FILE 'FfnI'
//
// typedef struct _INFCACHEFIELD
// {
//     struct _INFCACHEFIELD *Next;
//     struct _INFCACHEFIELD *Prev;
//
//     CHAR Data[1];
// } INFCACHEFIELD, *PINFCACHEFIELD;
//
// typedef struct _INFCACHELINE
// {
//     struct _INFCACHELINE *Next;
//     struct _INFCACHELINE *Prev;
//
//     ULONG FieldCount;
//
//     PCHAR Key;
//
//     PINFCACHEFIELD FirstField;
//     PINFCACHEFIELD LastField;
//
// } INFCACHELINE, *PINFCACHELINE;
//
// typedef struct _INFCACHESECTION
// {
//     struct _INFCACHESECTION *Next;
//     struct _INFCACHESECTION *Prev;
//
//     PINFCACHELINE FirstLine;
//     PINFCACHELINE LastLine;
//
//     LONG LineCount;
//
//     CHAR Name[1];
// } INFCACHESECTION, *PINFCACHESECTION;
//
// typedef struct _INFCACHE
// {
//     PINFCACHESECTION FirstSection;
//     PINFCACHESECTION LastSection;
//
//     PINFCACHESECTION StringsSection;
// } INFCACHE, *PINFCACHE;
//
// /* parser definitions */
// enum parser_state
// {
//     LINE_START,      /* at beginning of a line */
//     SECTION_NAME,    /* parsing a section name */
//     KEY_NAME,        /* parsing a key name */
//     VALUE_NAME,      /* parsing a value name */
//     EOL_BACKSLASH,   /* backslash at end of line */
//     QUOTES,          /* inside quotes */
//     LEADING_SPACES,  /* leading spaces */
//     TRAILING_SPACES, /* trailing spaces */
//     COMMENT,         /* inside a comment */
//     NB_PARSER_STATES
// };
//
// struct parser
// {
//     const CHAR        *start;       /* start position of item being parsed */
//     const CHAR        *end;         /* end of buffer */
//     PINFCACHE         file;         /* file being built */
//     enum parser_state state;        /* current parser state */
//     enum parser_state stack[4];     /* state stack */
//     int               stack_pos;    /* current pos in stack */
//
//     PINFCACHESECTION cur_section;   /* pointer to the section being parsed*/
//     PINFCACHELINE    line;          /* current line */
//     unsigned int     line_pos;      /* current line position in file */
//     unsigned int     error;         /* error code */
//     unsigned int     token_len;     /* current token len */
//     CHAR token[MAX_FIELD_LEN + 1]; /* current token */
// };
//
// typedef const CHAR * (*parser_state_func)(struct parser *parser, const CHAR *pos);
//
// /* parser state machine functions */
// static const CHAR *line_start_state(struct parser *parser, const CHAR *pos);
// static const CHAR *section_name_state(struct parser *parser, const CHAR *pos);
// static const CHAR *key_name_state(struct parser *parser, const CHAR *pos);
// static const CHAR *value_name_state(struct parser *parser, const CHAR *pos);
// static const CHAR *eol_backslash_state(struct parser *parser, const CHAR *pos);
// static const CHAR *quotes_state(struct parser *parser, const CHAR *pos);
// static const CHAR *leading_spaces_state(struct parser *parser, const CHAR *pos);
// static const CHAR *trailing_spaces_state(struct parser *parser, const CHAR *pos);
// static const CHAR *comment_state(struct parser *parser, const CHAR *pos);
//
// static
// const parser_state_func
// parser_funcs[NB_PARSER_STATES] =
// {
//     line_start_state,      /* LINE_START */
//     section_name_state,    /* SECTION_NAME */
//     key_name_state,        /* KEY_NAME */
//     value_name_state,      /* VALUE_NAME */
//     eol_backslash_state,   /* EOL_BACKSLASH */
//     quotes_state,          /* QUOTES */
//     leading_spaces_state,  /* LEADING_SPACES */
//     trailing_spaces_state, /* TRAILING_SPACES */
//     comment_state          /* COMMENT */
// };
//
//
// /* PRIVATE FUNCTIONS ********************************************************/
//
// static
// PINFCACHELINE
// InfpCacheFreeLine(
//     PINFCACHELINE Line)
// {
//     PINFCACHELINE Next;
//     PINFCACHEFIELD Field;
//
//     if (Line == NULL)
//     {
//         return NULL;
//     }
//
//     Next = Line->Next;
//     if (Line->Key != NULL)
//     {
//         FrLdrTempFree(Line->Key, TAG_INF_KEY);
//         Line->Key = NULL;
//     }
//
//     /* Remove data fields */
//     while (Line->FirstField != NULL)
//     {
//         Field = Line->FirstField->Next;
//         FrLdrTempFree(Line->FirstField, TAG_INF_FIELD);
//         Line->FirstField = Field;
//     }
//     Line->LastField = NULL;
//
//     FrLdrTempFree(Line, TAG_INF_LINE);
//
//     return Next;
// }
//
//
// static
// PINFCACHESECTION
// InfpCacheFreeSection(
//     PINFCACHESECTION Section)
// {
//     PINFCACHESECTION Next;
//
//     if (Section == NULL)
//     {
//         return NULL;
//     }
//
//     /* Release all keys */
//     Next = Section->Next;
//     while (Section->FirstLine != NULL)
//     {
//         Section->FirstLine = InfpCacheFreeLine(Section->FirstLine);
//     }
//     Section->LastLine = NULL;
//
//     FrLdrTempFree(Section, TAG_INF_SECTION);
//
//     return Next;
// }
//
//
// static
// PINFCACHESECTION
// InfpCacheFindSection(
//     PINFCACHE Cache,
//     PCSTR Name)
// {
//     PINFCACHESECTION Section = NULL;
//
//     if (Cache == NULL || Name == NULL)
//     {
//         return NULL;
//     }
//
//     /* iterate through list of sections */
//     Section = Cache->FirstSection;
//     while (Section != NULL)
//     {
//         if (_stricmp(Section->Name, Name) == 0)
//         {
//             return Section;
//         }
//
//         /* get the next section*/
//         Section = Section->Next;
//     }
//
//     return NULL;
// }
//
//
// static
// PINFCACHESECTION
// InfpCacheAddSection(
//     PINFCACHE Cache,
//     PCSTR Name)
// {
//     PINFCACHESECTION Section = NULL;
//     SIZE_T Size;
//
//     if ((Cache == NULL) || (Name == NULL))
//     {
// //      DPRINT("Invalid parameter\n");
//         return NULL;
//     }
//
//     /* Allocate and initialize the new section */
//     Size = sizeof(INFCACHESECTION) + strlen(Name);
//     Section = (PINFCACHESECTION)FrLdrTempAlloc(Size, TAG_INF_SECTION);
//     if (Section == NULL)
//     {
// //      DPRINT("RtlAllocateHeap() failed\n");
//         return NULL;
//     }
//     memset(Section, 0, Size);
//
//     /* Copy section name */
//     strcpy(Section->Name, Name);
//
//     /* Append section */
//     if (Cache->FirstSection == NULL)
//     {
//         Cache->FirstSection = Section;
//         Cache->LastSection = Section;
//     }
//     else
//     {
//         Cache->LastSection->Next = Section;
//         Section->Prev = Cache->LastSection;
//         Cache->LastSection = Section;
//     }
//
//     return Section;
// }
//
//
// static
// PINFCACHELINE
// InfpCacheAddLine(PINFCACHESECTION Section)
// {
//     PINFCACHELINE Line;
//
//     if (Section == NULL)
//     {
// //      DPRINT("Invalid parameter\n");
//         return NULL;
//     }
//
//     Line = (PINFCACHELINE)FrLdrTempAlloc(sizeof(INFCACHELINE), TAG_INF_LINE);
//     if (Line == NULL)
//     {
// //      DPRINT("RtlAllocateHeap() failed\n");
//         return NULL;
//     }
//     memset(Line, 0, sizeof(INFCACHELINE));
//
//     /* Append line */
//     if (Section->FirstLine == NULL)
//     {
//         Section->FirstLine = Line;
//         Section->LastLine = Line;
//     }
//     else
//     {
//         Section->LastLine->Next = Line;
//         Line->Prev = Section->LastLine;
//         Section->LastLine = Line;
//     }
//     Section->LineCount++;
//
//     return Line;
// }
//
//
// static
// PVOID
// InfpAddKeyToLine(
//     PINFCACHELINE Line,
//     PCSTR Key)
// {
//     if (Line == NULL)
//         return NULL;
//
//     if (Line->Key != NULL)
//         return NULL;
//
//     Line->Key = FrLdrTempAlloc(strlen(Key) + 1, TAG_INF_KEY);
//     if (Line->Key == NULL)
//         return NULL;
//
//     strcpy(Line->Key, Key);
//
//     return (PVOID)Line->Key;
// }
//
//
// static
// PVOID
// InfpAddFieldToLine(
//     PINFCACHELINE Line,
//     PCSTR Data)
// {
//     PINFCACHEFIELD Field;
//     SIZE_T Size;
//
//     Size = sizeof(INFCACHEFIELD) + strlen(Data);
//     Field = (PINFCACHEFIELD)FrLdrTempAlloc(Size, TAG_INF_FIELD);
//     if (Field == NULL)
//     {
//         return NULL;
//     }
//     memset(Field, 0, Size);
//
//     strcpy(Field->Data, Data);
//
//     /* Append key */
//     if (Line->FirstField == NULL)
//     {
//         Line->FirstField = Field;
//         Line->LastField = Field;
//     }
//     else
//     {
//         Line->LastField->Next = Field;
//         Field->Prev = Line->LastField;
//         Line->LastField = Field;
//     }
//     Line->FieldCount++;
//
//     return (PVOID)Field;
// }
//
//
// static
// PINFCACHELINE
// InfpCacheFindKeyLine(
//     PINFCACHESECTION Section,
//     PCSTR Key)
// {
//     PINFCACHELINE Line;
//
//     Line = Section->FirstLine;
//     while (Line != NULL)
//     {
//         if ((Line->Key != NULL) && (_stricmp(Line->Key, Key) == 0))
//         {
//             return Line;
//         }
//
//         Line = Line->Next;
//     }
//
//     return NULL;
// }
//
//
// /* push the current state on the parser stack */
// __inline static void push_state(struct parser *parser, enum parser_state state)
// {
// //  assert(parser->stack_pos < sizeof(parser->stack)/sizeof(parser->stack[0]));
//     parser->stack[parser->stack_pos++] = state;
// }
//
//
// /* pop the current state */
// __inline static void pop_state(struct parser *parser)
// {
// //  assert( parser->stack_pos );
//     parser->state = parser->stack[--parser->stack_pos];
// }
//
//
// /* set the parser state and return the previous one */
// __inline static enum parser_state set_state(struct parser *parser, enum parser_state state)
// {
//     enum parser_state ret = parser->state;
//     parser->state = state;
//     return ret;
// }
//
//
// /* check if the pointer points to an end of file */
// __inline static int is_eof(struct parser *parser, const CHAR *ptr)
// {
//     return (ptr >= parser->end || *ptr == CONTROL_Z);
// }
//
//
// /* check if the pointer points to an end of line */
// __inline static int is_eol(struct parser *parser, const CHAR *ptr)
// {
//     return ((ptr >= parser->end) ||
//             (*ptr == CONTROL_Z) ||
//             (*ptr == '\n') ||
//             ((*ptr == '\r') && (*(ptr + 1) == '\n')));
// }
//
//
// /* push data from current token start up to pos into the current token */
// static
// int
// push_token(
//     struct parser *parser,
//     const CHAR *pos)
// {
//     SIZE_T len = pos - parser->start;
//     const CHAR *src = parser->start;
//     CHAR *dst = parser->token + parser->token_len;
//
//     if (len > MAX_FIELD_LEN - parser->token_len)
//         len = MAX_FIELD_LEN - parser->token_len;
//
//     parser->token_len += (ULONG)len;
//     for ( ; len > 0; len--, dst++, src++)
//         *dst = *src ? (CHAR)*src : L' ';
//     *dst = 0;
//     parser->start = pos;
//
//     return 0;
// }
//
//
// /* add a section with the current token as name */
// static
// PVOID
// add_section_from_token(struct parser *parser)
// {
//     PINFCACHESECTION Section;
//
//     if (parser->token_len > MAX_SECTION_NAME_LEN)
//     {
//         parser->error = FALSE;
//         return NULL;
//     }
//
//     Section = InfpCacheFindSection(parser->file, parser->token);
//     if (Section == NULL)
//     {
//         /* need to create a new one */
//         Section = InfpCacheAddSection(parser->file, parser->token);
//         if (Section == NULL)
//         {
//             parser->error = FALSE;
//             return NULL;
//         }
//     }
//
//     parser->token_len = 0;
//     parser->cur_section = Section;
//
//     return (PVOID)Section;
// }
//
//
// /* add a field containing the current token to the current line */
// static
// struct field*
// add_field_from_token(
//     struct parser *parser,
//     int is_key)
// {
//     PVOID field;
//
//     if (!parser->line)  /* need to start a new line */
//     {
//         if (parser->cur_section == NULL)  /* got a line before the first section */
//         {
//             parser->error = STATUS_WRONG_INF_STYLE;
//             return NULL;
//         }
//
//         parser->line = InfpCacheAddLine(parser->cur_section);
//         if (parser->line == NULL)
//             goto error;
//     }
//     else
//     {
// //      assert(!is_key);
//     }
//
//     if (is_key)
//     {
//         field = InfpAddKeyToLine(parser->line, parser->token);
//     }
//     else
//     {
//         field = InfpAddFieldToLine(parser->line, parser->token);
//     }
//
//     if (field != NULL)
//     {
//         parser->token_len = 0;
//         return field;
//     }
//
// error:
//     parser->error = FALSE;
//     return NULL;
// }
//
//
// /* close the current line and prepare for parsing a new one */
// static
// VOID
// close_current_line(struct parser *parser)
// {
//     parser->line = NULL;
// }
//
//
// /* handler for parser LINE_START state */
// static
// const CHAR*
// line_start_state(
//     struct parser *parser,
//     const CHAR *pos)
// {
//     const CHAR *p;
//
//     for (p = pos; !is_eof(parser, p); p++)
//     {
//         switch(*p)
//         {
//         case '\r':
//             continue;
//
//         case '\n':
//             parser->line_pos++;
//             close_current_line(parser);
//             break;
//
//         case ';':
//             push_state(parser, LINE_START);
//             set_state(parser, COMMENT);
//             return p + 1;
//
//         case '[':
//             parser->start = p + 1;
//             set_state(parser, SECTION_NAME);
//             return p + 1;
//
//         default:
//             if (!isspace((unsigned char)*p))
//             {
//                 parser->start = p;
//                 set_state(parser, KEY_NAME);
//                 return p;
//             }
//             break;
//         }
//     }
//     close_current_line(parser);
//     return NULL;
// }
//
//
// /* handler for parser SECTION_NAME state */
// static
// const CHAR*
// section_name_state(
//     struct parser *parser,
//     const CHAR *pos)
// {
//     const CHAR *p;
//
//     for (p = pos; !is_eol(parser, p); p++)
//     {
//         if (*p == ']')
//         {
//             push_token(parser, p);
//             if (add_section_from_token(parser) == NULL)
//                 return NULL;
//             push_state(parser, LINE_START);
//             set_state(parser, COMMENT);  /* ignore everything else on the line */
//             return p + 1;
//         }
//     }
//     parser->error = STATUS_BAD_SECTION_NAME_LINE; /* unfinished section name */
//     return NULL;
// }
//
//
// /* handler for parser KEY_NAME state */
// static
// const CHAR*
// key_name_state(
//     struct parser *parser,
//     const CHAR *pos)
// {
//     const CHAR *p, *token_end = parser->start;
//
//     for (p = pos; !is_eol(parser, p); p++)
//     {
//         if (*p == ',') break;
//         switch(*p)
//         {
//
//         case '=':
//             push_token(parser, token_end);
//             if (!add_field_from_token(parser, 1)) return NULL;
//             parser->start = p + 1;
//             push_state(parser, VALUE_NAME);
//             set_state(parser, LEADING_SPACES);
//             return p + 1;
//         case ';':
//             push_token(parser, token_end);
//             if (!add_field_from_token(parser, 0)) return NULL;
//             push_state(parser, LINE_START);
//             set_state(parser, COMMENT);
//             return p + 1;
//         case '"':
//             push_token(parser, token_end);
//             parser->start = p + 1;
//             push_state(parser, KEY_NAME);
//             set_state(parser, QUOTES);
//             return p + 1;
//         case '\\':
//             push_token(parser, token_end);
//             parser->start = p;
//             push_state(parser, KEY_NAME);
//             set_state(parser, EOL_BACKSLASH);
//             return p;
//         default:
//             if (!isspace((unsigned char)*p)) token_end = p + 1;
//             else
//             {
//                 push_token(parser, p);
//                 push_state(parser, KEY_NAME);
//                 set_state(parser, TRAILING_SPACES);
//                 return p;
//             }
//             break;
//         }
//     }
//     push_token(parser, token_end);
//     set_state(parser, VALUE_NAME);
//     return p;
// }
//
//
// /* handler for parser VALUE_NAME state */
// static
// const CHAR*
// value_name_state(
//     struct parser *parser,
//     const CHAR *pos)
// {
//     const CHAR *p, *token_end = parser->start;
//
//     for (p = pos; !is_eol(parser, p); p++)
//     {
//         switch(*p)
//         {
//         case ';':
//             push_token(parser, token_end);
//             if (!add_field_from_token(parser, 0)) return NULL;
//             push_state(parser, LINE_START);
//             set_state(parser, COMMENT);
//             return p + 1;
//         case ',':
//             push_token(parser, token_end);
//             if (!add_field_from_token(parser, 0)) return NULL;
//             parser->start = p + 1;
//             push_state(parser, VALUE_NAME);
//             set_state(parser, LEADING_SPACES);
//             return p + 1;
//         case '"':
//             push_token(parser, token_end);
//             parser->start = p + 1;
//             push_state(parser, VALUE_NAME);
//             set_state(parser, QUOTES);
//             return p + 1;
//         case '\\':
//             push_token(parser, token_end);
//             parser->start = p;
//             push_state(parser, VALUE_NAME);
//             set_state(parser, EOL_BACKSLASH);
//             return p;
//         default:
//             if (!isspace((unsigned char)*p)) token_end = p + 1;
//             else
//             {
//                 push_token(parser, p);
//                 push_state(parser, VALUE_NAME);
//                 set_state(parser, TRAILING_SPACES);
//                 return p;
//             }
//             break;
//         }
//     }
//     push_token(parser, token_end);
//     if (!add_field_from_token(parser, 0)) return NULL;
//     set_state(parser, LINE_START);
//     return p;
// }
//
//
// /* handler for parser EOL_BACKSLASH state */
// static
// const CHAR*
// eol_backslash_state(
//     struct parser *parser,
//     const CHAR *pos )
// {
//     const CHAR *p;
//
//     for (p = pos; !is_eof(parser, p); p++)
//     {
//         switch(*p)
//         {
//         case '\r':
//             continue;
//
//         case '\n':
//             parser->line_pos++;
//             parser->start = p + 1;
//             set_state(parser, LEADING_SPACES);
//             return p + 1;
//
//         case '\\':
//             continue;
//
//         case ';':
//             push_state(parser, EOL_BACKSLASH);
//             set_state(parser, COMMENT);
//             return p + 1;
//
//         default:
//             if (isspace((unsigned char)*p))
//                 continue;
//             push_token(parser, p);
//             pop_state(parser);
//             return p;
//         }
//     }
//     parser->start = p;
//     pop_state(parser);
//
//     return p;
// }
//
//
// /* handler for parser QUOTES state */
// static
// const CHAR*
// quotes_state(
//     struct parser *parser,
//     const CHAR *pos )
// {
//     const CHAR *p, *token_end = parser->start;
//
//     for (p = pos; !is_eol(parser, p); p++)
//     {
//         if (*p == '"')
//         {
//             if (p + 1 < parser->end && p[1] == '"') /* double quotes */
//             {
//                 push_token(parser, p + 1);
//                 parser->start = token_end = p + 2;
//                 p++;
//             }
//             else  /* end of quotes */
//             {
//                 push_token(parser, p);
//                 parser->start = p + 1;
//                 pop_state(parser);
//                 return p + 1;
//             }
//         }
//     }
//     push_token(parser, p);
//     pop_state(parser);
//     return p;
// }
//
//
// /* handler for parser LEADING_SPACES state */
// static
// const CHAR*
// leading_spaces_state(
//     struct parser *parser,
//     const CHAR *pos )
// {
//     const CHAR *p;
//
//     for (p = pos; !is_eol(parser, p); p++)
//     {
//         if (*p == '\\')
//         {
//             parser->start = p;
//             set_state(parser, EOL_BACKSLASH);
//             return p;
//         }
//         if (!isspace((unsigned char)*p))
//             break;
//     }
//     parser->start = p;
//     pop_state(parser);
//     return p;
// }
//
//
// /* handler for parser TRAILING_SPACES state */
// static
// const CHAR*
// trailing_spaces_state(
//     struct parser *parser,
//     const CHAR *pos )
// {
//     const CHAR *p;
//
//     for (p = pos; !is_eol(parser, p); p++)
//     {
//         if (*p == '\\')
//         {
//             set_state(parser, EOL_BACKSLASH);
//             return p;
//         }
//         if (!isspace((unsigned char)*p))
//             break;
//     }
//     pop_state(parser);
//     return p;
// }
//
//
// /* handler for parser COMMENT state */
// static
// const CHAR*
// comment_state(
//     struct parser *parser,
//     const CHAR *pos )
// {
//     const CHAR *p = pos;
//
//     while (!is_eol(parser, p))
//         p++;
//     pop_state(parser);
//     return p;
// }
//
//
// /* parse a complete buffer */
// static
// BOOLEAN
// InfpParseBuffer (
//     PINFCACHE file,
//     PCCHAR buffer,
//     PCCHAR end,
//     PULONG error_line)
// {
//     struct parser parser;
//     const CHAR* pos = buffer;
//
//     parser.start       = buffer;
//     parser.end         = end;
//     parser.file        = file;
//     parser.line        = NULL;
//     parser.state       = LINE_START;
//     parser.stack_pos   = 0;
//     parser.cur_section = NULL;
//     parser.line_pos    = 1;
//     parser.error       = TRUE;
//     parser.token_len   = 0;
//
//     /* parser main loop */
//     while (pos)
//         pos = (parser_funcs[parser.state])(&parser, pos);
//
//     if (parser.error)
//     {
//         if (error_line)
//             *error_line = parser.line_pos;
//         return parser.error;
//     }
//
//     /* find the [strings] section */
//     file->StringsSection = InfpCacheFindSection(file, "Strings");
//
//     return TRUE;
// }
//
// /* PUBLIC FUNCTIONS *********************************************************/
//
// BOOLEAN
// InfOpenFile(
//     PHINF InfHandle,
//     PCSTR FileName,
//     PULONG ErrorLine)
// {
//     FILEINFORMATION Information;
//     ULONG FileId;
//     PCHAR FileBuffer;
//     ULONG FileSize, Count;
//     PINFCACHE Cache;
//     BOOLEAN Success;
//     ARC_STATUS Status;
//
//     *InfHandle = NULL;
//     *ErrorLine = (ULONG) - 1;
//
//     //
//     // Open the .inf file
//     //
//     Status = ArcOpen((PSTR)FileName, OpenReadOnly, &FileId);
//     if (Status != ESUCCESS)
//     {
//         return FALSE;
//     }
//
//     //
//     // Query file size
//     //
//     Status = ArcGetFileInformation(FileId, &Information);
//     if ((Status != ESUCCESS) || (Information.EndingAddress.HighPart != 0))
//     {
//         ArcClose(FileId);
//         return FALSE;
//     }
//     FileSize = Information.EndingAddress.LowPart;
//
//     //
//     // Allocate buffer to cache the file
//     //
//     FileBuffer = FrLdrTempAlloc(FileSize + 1, TAG_INF_FILE);
//     if (!FileBuffer)
//     {
//         ArcClose(FileId);
//         return FALSE;
//     }
//
//     //
//     // Read file into memory
//     //
//     Status = ArcRead(FileId, FileBuffer, FileSize, &Count);
//     if ((Status != ESUCCESS) || (Count != FileSize))
//     {
//         ArcClose(FileId);
//         FrLdrTempFree(FileBuffer, TAG_INF_FILE);
//         return FALSE;
//     }
//
//     //
//     // We don't need the file anymore. Close it
//     //
//     ArcClose(FileId);
//
//     //
//     // Append string terminator
//     //
//     FileBuffer[FileSize] = 0;
//
//     //
//     // Allocate infcache header
//     //
//     Cache = (PINFCACHE)FrLdrTempAlloc(sizeof(INFCACHE), TAG_INF_CACHE);
//     if (!Cache)
//     {
//         FrLdrTempFree(FileBuffer, TAG_INF_FILE);
//         return FALSE;
//     }
//
//     //
//     // Initialize inicache header
//     //
//     RtlZeroMemory(Cache, sizeof(INFCACHE));
//
//     //
//     // Parse the inf buffer
//     //
//     Success = InfpParseBuffer(Cache,
//                               FileBuffer,
//                               FileBuffer + FileSize,
//                               ErrorLine);
//     if (!Success)
//     {
//         FrLdrTempFree(Cache, TAG_INF_CACHE);
//         Cache = NULL;
//     }
//
//     //
//     // Free file buffer, as it has been parsed
//     //
//     FrLdrTempFree(FileBuffer, TAG_INF_FILE);
//
//     //
//     // Return .inf parsed contents
//     //
//     *InfHandle = (HINF)Cache;
//
//     return Success;
// }
//
//
// VOID
// InfCloseFile(HINF InfHandle)
// {
//     PINFCACHE Cache;
//
//     Cache = (PINFCACHE)InfHandle;
//
//     if (Cache == NULL)
//     {
//         return;
//     }
//
//     while (Cache->FirstSection != NULL)
//     {
//         Cache->FirstSection = InfpCacheFreeSection(Cache->FirstSection);
//     }
//     Cache->LastSection = NULL;
//
//     FrLdrTempFree(Cache, TAG_INF_CACHE);
// }
//
//
// BOOLEAN
// InfFindFirstLine (
//     HINF InfHandle,
//     PCSTR Section,
//     PCSTR Key,
//     PINFCONTEXT Context)
// {
//     PINFCACHE Cache;
//     PINFCACHESECTION CacheSection;
//     PINFCACHELINE CacheLine;
//
//     if ((InfHandle == NULL) || (Section == NULL) || (Context == NULL))
//     {
// //      DPRINT("Invalid parameter\n");
//         return FALSE;
//     }
//
//     Cache = (PINFCACHE)InfHandle;
//
//     /* Iterate through list of sections */
//     CacheSection = Cache->FirstSection;
//     while (CacheSection != NULL)
//     {
// //      DPRINT("Comparing '%s' and '%s'\n", CacheSection->Name, Section);
//
//         /* Are the section names the same? */
//         if (_stricmp(CacheSection->Name, Section) == 0)
//         {
//             if (Key != NULL)
//             {
//                 CacheLine = InfpCacheFindKeyLine(CacheSection, Key);
//             }
//             else
//             {
//                 CacheLine = CacheSection->FirstLine;
//             }
//
//             if (CacheLine == NULL)
//                 return FALSE;
//
//             Context->Inf = (PVOID)Cache;
//             Context->Section = (PVOID)CacheSection;
//             Context->Line = (PVOID)CacheLine;
//
//             return TRUE;
//         }
//
//         /* Get the next section */
//         CacheSection = CacheSection->Next;
//     }
//
// //  DPRINT("Section not found\n");
//
//     return FALSE;
// }
//
//
// BOOLEAN
// InfFindNextLine (
//     PINFCONTEXT ContextIn,
//     PINFCONTEXT ContextOut)
// {
//     PINFCACHELINE CacheLine;
//
//     if ((ContextIn == NULL) || (ContextOut == NULL))
//         return FALSE;
//
//     if (ContextIn->Line == NULL)
//         return FALSE;
//
//     CacheLine = (PINFCACHELINE)ContextIn->Line;
//     if (CacheLine->Next == NULL)
//         return FALSE;
//
//     if (ContextIn != ContextOut)
//     {
//         ContextOut->Inf = ContextIn->Inf;
//         ContextOut->Section = ContextIn->Section;
//     }
//     ContextOut->Line = (PVOID)(CacheLine->Next);
//
//     return TRUE;
// }
//
//
// BOOLEAN
// InfFindFirstMatchLine (
//     PINFCONTEXT ContextIn,
//     PCSTR Key,
//     PINFCONTEXT ContextOut)
// {
//     PINFCACHELINE CacheLine;
//
//     if ((ContextIn == NULL) || (ContextOut == NULL) || (Key == NULL) || (*Key == 0))
//         return FALSE;
//
//     if (ContextIn->Inf == NULL || ContextIn->Section == NULL)
//         return FALSE;
//
//     CacheLine = ((PINFCACHESECTION)(ContextIn->Section))->FirstLine;
//     while (CacheLine != NULL)
//     {
//         if ((CacheLine->Key != NULL) && (_stricmp(CacheLine->Key, Key) == 0))
//         {
//
//             if (ContextIn != ContextOut)
//             {
//                 ContextOut->Inf = ContextIn->Inf;
//                 ContextOut->Section = ContextIn->Section;
//             }
//             ContextOut->Line = (PVOID)CacheLine;
//
//             return TRUE;
//         }
//
//         CacheLine = CacheLine->Next;
//     }
//
//     return FALSE;
// }
//
//
// BOOLEAN
// InfFindNextMatchLine (
//     PINFCONTEXT ContextIn,
//     PCSTR Key,
//     PINFCONTEXT ContextOut)
// {
//     PINFCACHELINE CacheLine;
//
//     if ((ContextIn == NULL) || (ContextOut == NULL) || (Key == NULL) || (*Key == 0))
//         return FALSE;
//
//     if ((ContextIn->Inf == NULL) || (ContextIn->Section == NULL) || (ContextIn->Line == NULL))
//         return FALSE;
//
//     CacheLine = (PINFCACHELINE)ContextIn->Line;
//     while (CacheLine != NULL)
//     {
//         if ((CacheLine->Key != NULL) && (_stricmp(CacheLine->Key, Key) == 0))
//         {
//
//             if (ContextIn != ContextOut)
//             {
//                 ContextOut->Inf = ContextIn->Inf;
//                 ContextOut->Section = ContextIn->Section;
//             }
//             ContextOut->Line = (PVOID)CacheLine;
//
//             return TRUE;
//         }
//
//         CacheLine = CacheLine->Next;
//     }
//
//     return FALSE;
// }
//
//
// LONG
// InfGetLineCount(
//     HINF InfHandle,
//     PCSTR Section)
// {
//     PINFCACHE Cache;
//     PINFCACHESECTION CacheSection;
//
//     if ((InfHandle == NULL) || (Section == NULL))
//     {
// //      DPRINT("Invalid parameter\n");
//         return -1;
//     }
//
//     Cache = (PINFCACHE)InfHandle;
//
//     /* Iterate through list of sections */
//     CacheSection = Cache->FirstSection;
//     while (CacheSection != NULL)
//     {
// //      DPRINT("Comparing '%s' and '%s'\n", CacheSection->Name, Section);
//
//         /* Are the section names the same? */
//         if (_stricmp(CacheSection->Name, Section) == 0)
//         {
//             return CacheSection->LineCount;
//         }
//
//         /* Get the next section */
//         CacheSection = CacheSection->Next;
//     }
//
// //  DPRINT("Section not found\n");
//
//     return -1;
// }
//
//
// /* InfGetLineText */
//
//
// LONG
// InfGetFieldCount(PINFCONTEXT Context)
// {
//     if ((Context == NULL) || (Context->Line == NULL))
//         return 0;
//
//     return ((PINFCACHELINE)Context->Line)->FieldCount;
// }
//
//
// BOOLEAN
// InfGetBinaryField (
//     PINFCONTEXT Context,
//     ULONG FieldIndex,
//     PUCHAR ReturnBuffer,
//     ULONG ReturnBufferSize,
//     PULONG RequiredSize)
// {
//     PINFCACHELINE CacheLine;
//     PINFCACHEFIELD CacheField;
//     ULONG Index;
//     ULONG Size;
//     PUCHAR Ptr;
//
//     if ((Context == NULL) || (Context->Line == NULL) || (FieldIndex == 0))
//     {
// //      DPRINT("Invalid parameter\n");
//         return FALSE;
//     }
//
//     if (RequiredSize != NULL)
//         *RequiredSize = 0;
//
//     CacheLine = (PINFCACHELINE)Context->Line;
//
//     if (FieldIndex > CacheLine->FieldCount)
//         return FALSE;
//
//     CacheField = CacheLine->FirstField;
//     for (Index = 1; Index < FieldIndex; Index++)
//         CacheField = CacheField->Next;
//
//     Size = CacheLine->FieldCount - FieldIndex + 1;
//
//     if (RequiredSize != NULL)
//         *RequiredSize = Size;
//
//     if (ReturnBuffer != NULL)
//     {
//         if (ReturnBufferSize < Size)
//             return FALSE;
//
//         /* Copy binary data */
//         Ptr = ReturnBuffer;
//         while (CacheField != NULL)
//         {
//             *Ptr = (UCHAR)atoi(CacheField->Data); //strtoul(CacheField->Data, NULL, 16);
//
//             Ptr++;
//             CacheField = CacheField->Next;
//         }
//     }
//
//     return TRUE;
// }
//
//
// BOOLEAN
// InfGetIntField (
//     PINFCONTEXT Context,
//     ULONG FieldIndex,
//     PLONG IntegerValue)
// {
//     PINFCACHELINE CacheLine;
//     PINFCACHEFIELD CacheField;
//     ULONG Index;
//     PCHAR Ptr;
//
//     if ((Context == NULL) || (Context->Line == NULL) || (IntegerValue == NULL))
//     {
// //      DPRINT("Invalid parameter\n");
//         return FALSE;
//     }
//
//     CacheLine = (PINFCACHELINE)Context->Line;
//
//     if (FieldIndex > CacheLine->FieldCount)
//     {
// //      DPRINT("Invalid parameter\n");
//         return FALSE;
//     }
//
//     if (FieldIndex == 0)
//     {
//         Ptr = CacheLine->Key;
//     }
//     else
//     {
//         CacheField = CacheLine->FirstField;
//         for (Index = 1; Index < FieldIndex; Index++)
//             CacheField = CacheField->Next;
//
//         Ptr = CacheField->Data;
//     }
//
//     *IntegerValue = atoi(Ptr); //strtol(Ptr, NULL, 0);
//
//     return TRUE;
// }
//
//
// BOOLEAN
// InfGetMultiSzField (
//     PINFCONTEXT Context,
//     ULONG FieldIndex,
//     PCHAR ReturnBuffer,
//     ULONG ReturnBufferSize,
//     PULONG RequiredSize)
// {
//     PINFCACHELINE CacheLine;
//     PINFCACHEFIELD CacheField;
//     PINFCACHEFIELD FieldPtr;
//     ULONG Index;
//     SIZE_T Size;
//     PCHAR Ptr;
//
//     if ((Context == NULL) || (Context->Line == NULL) || (FieldIndex == 0))
//     {
// //      DPRINT("Invalid parameter\n");
//         return FALSE;
//     }
//
//     if (RequiredSize != NULL)
//         *RequiredSize = 0;
//
//     CacheLine = (PINFCACHELINE)Context->Line;
//
//     if (FieldIndex > CacheLine->FieldCount)
//         return FALSE;
//
//     CacheField = CacheLine->FirstField;
//     for (Index = 1; Index < FieldIndex; Index++)
//         CacheField = CacheField->Next;
//
//     /* Calculate the required buffer size */
//     FieldPtr = CacheField;
//     Size = 0;
//     while (FieldPtr != NULL)
//     {
//         Size += (strlen(FieldPtr->Data) + 1);
//         FieldPtr = FieldPtr->Next;
//     }
//     Size++;
//
//     if (RequiredSize != NULL)
//         *RequiredSize = (ULONG)Size;
//
//     if (ReturnBuffer != NULL)
//     {
//         if (ReturnBufferSize < Size)
//             return FALSE;
//
//         /* Copy multi-sz string */
//         Ptr = ReturnBuffer;
//         FieldPtr = CacheField;
//         while (FieldPtr != NULL)
//         {
//             Size = strlen(FieldPtr->Data) + 1;
//
//             strcpy(Ptr, FieldPtr->Data);
//
//             Ptr = Ptr + Size;
//             FieldPtr = FieldPtr->Next;
//         }
//         *Ptr = 0;
//     }
//
//     return TRUE;
// }
//
//
// BOOLEAN
// InfGetStringField (
//     PINFCONTEXT Context,
//     ULONG FieldIndex,
//     PCHAR ReturnBuffer,
//     ULONG ReturnBufferSize,
//     PULONG RequiredSize)
// {
//     PINFCACHELINE CacheLine;
//     PINFCACHEFIELD CacheField;
//     ULONG Index;
//     PCHAR Ptr;
//     SIZE_T Size;
//
//     if ((Context == NULL) || (Context->Line == NULL))
//     {
// //      DPRINT("Invalid parameter\n");
//         return FALSE;
//     }
//
//     if (RequiredSize != NULL)
//         *RequiredSize = 0;
//
//     CacheLine = (PINFCACHELINE)Context->Line;
//
//     if (FieldIndex > CacheLine->FieldCount)
//         return FALSE;
//
//     if (FieldIndex == 0)
//     {
//         Ptr = CacheLine->Key;
//     }
//     else
//     {
//         CacheField = CacheLine->FirstField;
//         for (Index = 1; Index < FieldIndex; Index++)
//             CacheField = CacheField->Next;
//
//         Ptr = CacheField->Data;
//     }
//
//     Size = strlen(Ptr) + 1;
//
//     if (RequiredSize != NULL)
//         *RequiredSize = (ULONG)Size;
//
//     if (ReturnBuffer != NULL)
//     {
//         if (ReturnBufferSize < Size)
//             return FALSE;
//
//         strcpy(ReturnBuffer, Ptr);
//     }
//
//     return TRUE;
// }
//
//
//
//
// BOOLEAN
// InfGetData (
//     PINFCONTEXT Context,
//     PCSTR *Key,
//     PCSTR *Data)
// {
//     PINFCACHELINE CacheKey;
//
//     if ((Context == NULL) || (Context->Line == NULL) || (Data == NULL))
//     {
// //      DPRINT("Invalid parameter\n");
//         return FALSE;
//     }
//
//     CacheKey = (PINFCACHELINE)Context->Line;
//     if (Key != NULL)
//         *Key = CacheKey->Key;
//
//     *Data = (CacheKey->FirstField == NULL) ? NULL : CacheKey->FirstField->Data;
//
//     return TRUE;
// }
//
//
// BOOLEAN
// InfGetDataField (
//     PINFCONTEXT Context,
//     ULONG FieldIndex,
//     PCSTR *Data)
// {
//     PINFCACHELINE CacheLine;
//     PINFCACHEFIELD CacheField;
//     ULONG Index;
//
//     if ((Context == NULL) || (Context->Line == NULL) || (Data == NULL))
//     {
// //      DPRINT("Invalid parameter\n");
//         return FALSE;
//     }
//
//     CacheLine = (PINFCACHELINE)Context->Line;
//
//     if (FieldIndex > CacheLine->FieldCount)
//         return FALSE;
//
//     if (FieldIndex == 0)
//     {
//         *Data = CacheLine->Key;
//     }
//     else
//     {
//         CacheField = CacheLine->FirstField;
//         for (Index = 1; Index < FieldIndex; Index++)
//             CacheField = CacheField->Next;
//
//         *Data = CacheField->Data;
//     }
//
//     return TRUE;
// }
//
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
