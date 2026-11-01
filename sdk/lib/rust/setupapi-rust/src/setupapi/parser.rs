use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut BackSlash: [WCHAR; 2] = ['\\' as i32 as WCHAR, 0 as ::core::ffi::c_int as WCHAR];

static mut Class: [WCHAR; 6] = [
    'C' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    's' as i32 as WCHAR,
    's' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut ClassGUID: [WCHAR; 10] = [
    'C' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    's' as i32 as WCHAR,
    's' as i32 as WCHAR,
    'G' as i32 as WCHAR,
    'U' as i32 as WCHAR,
    'I' as i32 as WCHAR,
    'D' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut InfDirectory: [WCHAR; 5] = [
    'i' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    'f' as i32 as WCHAR,
    '\\' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut InfFileSpecification: [WCHAR; 6] = [
    '*' as i32 as WCHAR,
    '.' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    'f' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

pub const CONTROL_Z: ::core::ffi::c_int = '\u{1a}' as i32;

static mut parser_funcs: [parser_state_func; 9] = unsafe {
    [
        Some(line_start_state as unsafe extern "C" fn(*mut parser, *const WCHAR) -> *const WCHAR),
        Some(section_name_state as unsafe extern "C" fn(*mut parser, *const WCHAR) -> *const WCHAR),
        Some(key_name_state as unsafe extern "C" fn(*mut parser, *const WCHAR) -> *const WCHAR),
        Some(value_name_state as unsafe extern "C" fn(*mut parser, *const WCHAR) -> *const WCHAR),
        Some(
            eol_backslash_state as unsafe extern "C" fn(*mut parser, *const WCHAR) -> *const WCHAR,
        ),
        Some(quotes_state as unsafe extern "C" fn(*mut parser, *const WCHAR) -> *const WCHAR),
        Some(
            leading_spaces_state as unsafe extern "C" fn(*mut parser, *const WCHAR) -> *const WCHAR,
        ),
        Some(
            trailing_spaces_state
                as unsafe extern "C" fn(*mut parser, *const WCHAR) -> *const WCHAR,
        ),
        Some(comment_state as unsafe extern "C" fn(*mut parser, *const WCHAR) -> *const WCHAR),
    ]
};

static mut Version: [WCHAR; 8] = [
    'V' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    's' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut Signature: [WCHAR; 10] = [
    'S' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'g' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'u' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut Chicago: [WCHAR; 10] = [
    '$' as i32 as WCHAR,
    'C' as i32 as WCHAR,
    'h' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'c' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'g' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    '$' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut WindowsNT: [WCHAR; 13] = [
    '$' as i32 as WCHAR,
    'W' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    'd' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'w' as i32 as WCHAR,
    's' as i32 as WCHAR,
    ' ' as i32 as WCHAR,
    'N' as i32 as WCHAR,
    'T' as i32 as WCHAR,
    '$' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut Windows95: [WCHAR; 13] = [
    '$' as i32 as WCHAR,
    'W' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    'd' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'w' as i32 as WCHAR,
    's' as i32 as WCHAR,
    ' ' as i32 as WCHAR,
    '9' as i32 as WCHAR,
    '5' as i32 as WCHAR,
    '$' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut LayoutFile: [WCHAR; 11] = [
    'L' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'y' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'u' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'F' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

pub const MAX_SECTION_NAME_LEN: i32 = 255;

pub const MAX_FIELD_LEN: i32 = 511;

// MAX_INF_STRING_LENGTH (4096) + 1
pub const MAX_STRING_LEN: i32 = 4096 + 1;

pub type parser_state = i32;
pub const LINE_START: parser_state = 0;
pub const SECTION_NAME: parser_state = 1;
pub const KEY_NAME: parser_state = 2;
pub const VALUE_NAME: parser_state = 3;
pub const EOL_BACKSLASH: parser_state = 4;
pub const QUOTES: parser_state = 5;
pub const LEADING_SPACES: parser_state = 6;
pub const TRAILING_SPACES: parser_state = 7;
pub const COMMENT: parser_state = 8;
pub const NB_PARSER_STATES: parser_state = 9;

#[repr(C)]
pub struct parser {
    pub start: *const WCHAR,
    pub end: *const WCHAR,
    pub file: *mut inf_file,
    pub state: parser_state,
    pub stack: [parser_state; 4],
    pub stack_pos: i32,
    pub cur_section: i32,
    pub line: *mut line,
    pub line_pos: u32,
    pub broken_line: u32,
    pub error: u32,
    pub token_len: u32,
    pub token: [WCHAR; (MAX_FIELD_LEN + 1) as usize],
}

pub type parser_state_func =
    Option<unsafe extern "C" fn(parser: *mut parser, pos: *const WCHAR) -> *const WCHAR>;

unsafe extern "C" fn grow_array(
    mut array: *mut ::core::ffi::c_void,
    mut count: *mut ::core::ffi::c_uint,
    mut elem: size_t,
) -> *mut ::core::ffi::c_void {
    let mut new_array: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
    let mut new_count: ::core::ffi::c_uint =
        (*count).wrapping_add((*count).wrapping_div(2 as ::core::ffi::c_uint));
    if new_count < 32 as ::core::ffi::c_uint {
        new_count = 32 as ::core::ffi::c_uint;
    }
    if !array.is_null() {
        new_array = HeapReAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY as DWORD,
            array as PVOID,
            (new_count as SIZE_T).wrapping_mul(elem as SIZE_T),
        ) as *mut ::core::ffi::c_void;
    } else {
        new_array = HeapAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY as DWORD,
            (new_count as SIZE_T).wrapping_mul(elem as SIZE_T),
        ) as *mut ::core::ffi::c_void;
    }
    if !new_array.is_null() {
        *count = new_count;
    } else {
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            array as PVOID,
        );
    }
    return new_array;
}

unsafe extern "C" fn get_inf_dir(file: *const inf_file, len: *mut u32) -> *const WCHAR {
    let p: *const WCHAR = strrchrW((*file).filename, '\\' as WCHAR);
    *len = if !p.is_null() {
        (p.offset(1 as isize).offset_from((*file).filename)) as u32
    } else {
        0 as u32
    };
    return (*file).filename;
}

unsafe extern "C" fn find_section(file: *const inf_file, name: *const WCHAR) -> ::core::ffi::c_int {
    let mut i: ::core::ffi::c_uint;
    i = 0 as ::core::ffi::c_uint;
    while i < (*file).nb_sections {
        if strcmpiW(name, (**(*file).sections.add(i as usize)).name) == 0 {
            return i as ::core::ffi::c_int;
        }
        i = i.wrapping_add(1);
    }
    return -(1 as ::core::ffi::c_int);
}

unsafe extern "C" fn find_line(
    file: *mut inf_file,
    section_index: ::core::ffi::c_int,
    name: *const WCHAR,
) -> *mut line {
    let mut section: *mut section = ::core::ptr::null_mut::<section>();
    let mut line: *mut line = ::core::ptr::null_mut::<line>();
    let mut i: ::core::ffi::c_uint = 0;
    if section_index < 0 || section_index as ::core::ffi::c_uint >= (*file).nb_sections {
        return ::core::ptr::null_mut::<line>();
    }
    section = *(*file).sections.offset(section_index as isize);
    i = 0;
    line = ::core::ptr::addr_of_mut!((*section).lines) as *mut line;
    while i < (*section).nb_lines {
        if !((*line).key_field == -1) {
            if strcmpiW(
                name,
                (*(*file).fields.offset((*line).key_field as isize)).text,
            ) == 0
            {
                return line;
            }
        }
        i = i.wrapping_add(1);
        line = line.offset(1);
    }
    return ::core::ptr::null_mut::<line>();
}

unsafe extern "C" fn add_section(file: *mut inf_file, name: *const WCHAR) -> ::core::ffi::c_int {
    let mut section: *mut section = ::core::ptr::null_mut::<section>();
    if (*file).nb_sections >= (*file).alloc_sections {
        let _elem_idx: usize = 0;
        (*file).sections = grow_array(
            (*file).sections as *mut ::core::ffi::c_void,
            &raw mut (*file).alloc_sections,
            ::core::mem::size_of::<section>() as size_t,
        ) as *mut *mut section;
        if (*file).sections.is_null() {
            return -1;
        }
    }
    section = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        ::core::mem::size_of::<section>() as SIZE_T,
    ) as *mut section;
    if section.is_null() {
        return -1;
    }
    (*section).name = name;
    (*section).nb_lines = 0 as ::core::ffi::c_uint;
    let _lines_idx: usize = 0;
    (*section).alloc_lines = (::core::mem::size_of::<[line; 16]>() as usize)
        .wrapping_div(::core::mem::size_of::<line>() as usize)
        as ::core::ffi::c_uint;
    let ref mut fresh19 = *(*file).sections.offset((*file).nb_sections as isize);
    *fresh19 = section;
    let fresh20 = (*file).nb_sections;
    (*file).nb_sections = (*file).nb_sections.wrapping_add(1);
    return fresh20 as ::core::ffi::c_int;
}

unsafe extern "C" fn add_line(file: *mut inf_file, section_index: i32) -> *mut line {
    let mut section: *mut section = core::ptr::null_mut();
    let mut line: *mut line = core::ptr::null_mut();
    ASSERT!(section_index >= 0 && (section_index as u32) < (*file).nb_sections);
    section = *(*file).sections.offset(section_index as isize);
    if (*section).nb_lines == (*section).alloc_lines {
        let mut size: i32 = (core::mem::size_of::<section>()
            - core::mem::size_of::<[line; 16]>()
            + (2 as usize) * ((*section).alloc_lines as usize) * core::mem::size_of::<line>()) as i32;
        section = HeapReAlloc(
            GetProcessHeap(),
            0 as u32,
            section as *mut core::ffi::c_void,
            (size) as SIZE_T,
        ) as *mut section;
        if section.is_null() {
            return core::ptr::null_mut();
        }
        (*section).alloc_lines *= 2;
        *(*file).sections.offset(section_index as isize) = section;
    }
    let fresh18 = (*section).nb_lines;
    (*section).nb_lines = fresh18 + 1;
    line = (core::ptr::addr_of_mut!((*section).lines) as *mut line).offset(fresh18 as isize);
    (*line).first_field = (*file).nb_fields as i32;
    (*line).nb_fields = 0;
    (*line).key_field = -(1 as i32);
    return line;
}

unsafe extern "C" fn get_line(
    file: *mut inf_file,
    section_index: ::core::ffi::c_uint,
    line_index: ::core::ffi::c_uint,
) -> *mut line {
    let mut section: *mut section;

    if section_index >= (*file).nb_sections {
        return core::ptr::null_mut();
    }
    section = *(*file).sections.offset(section_index as isize);
    if line_index >= (*section).nb_lines {
        return core::ptr::null_mut();
    }
    return core::ptr::addr_of_mut!((*section).lines).cast::<line>().offset(line_index as isize);
}

unsafe extern "C" fn get_field(
    file: *mut inf_file,
    section_index: ::core::ffi::c_int,
    line_index: ::core::ffi::c_int,
    mut field_index: ::core::ffi::c_int,
) -> *mut field {
    let line: *mut line = get_line(
        file,
        section_index as ::core::ffi::c_uint,
        line_index as ::core::ffi::c_uint,
    );

    if line.is_null() {
        return ::core::ptr::null_mut::<field>();
    }
    if field_index == 0 {
        if (*line).key_field == -1 {
            return ::core::ptr::null_mut::<field>();
        }
        return (*file).fields.offset((*line).key_field as isize) as *mut field;
    }
    field_index -= 1;
    if field_index >= (*line).nb_fields {
        return ::core::ptr::null_mut::<field>();
    }
    return (*file)
        .fields
        .offset(((*line).first_field + field_index) as isize) as *mut field;
}

unsafe extern "C" fn add_field(file: *mut inf_file, text: *const WCHAR) -> *mut field {
    let mut field: *mut field = core::ptr::null_mut::<field>();
    if (*file).nb_fields >= (*file).alloc_fields {
        (*file).fields = grow_array(
            (*file).fields as *mut core::ffi::c_void,
            core::ptr::addr_of_mut!((*file).alloc_fields),
            core::mem::size_of::<field>() + 0 * (0 as usize),
        ) as *mut field;
        if (*file).fields.is_null() {
            return core::ptr::null_mut::<field>();
        }
    }
    let fresh16 = (*file).nb_fields;
    (*file).nb_fields = (*file).nb_fields.wrapping_add(1);
    field = (*file).fields.offset(fresh16 as isize) as *mut field;
    (*field).text = text;
    return field;
}

unsafe extern "C" fn get_dirid_subst(file: *const inf_file, dirid: i32, len: *mut u32) -> *const WCHAR {
    let mut ret: *const WCHAR = core::ptr::null();

    if dirid == (DIRID_SRCPATH as i32) {
        return get_inf_dir(file, len) as *const WCHAR;
    }
    ret = DIRID_get_string(dirid);
    if !ret.is_null() {
        *len = strlenW(ret) as u32;
    }
    return ret;
}

unsafe extern "C" fn get_string_subst(
    file: *const inf_file,
    str: *const WCHAR,
    len: *mut core::ffi::c_uint,
    no_trailing_slash: BOOL,
) -> *const WCHAR {
    static percent: WCHAR = '%' as WCHAR;

    let mut strings_section: *mut section = core::ptr::null_mut();
    let mut line: *mut line = core::ptr::null_mut();
    let mut field: *mut field = core::ptr::null_mut();
    let mut i: core::ffi::c_uint = 0;
    let mut j: core::ffi::c_uint = 0;
    let mut dirid: core::ffi::c_int = 0;
    let mut dirid_str: *mut WCHAR = core::ptr::null_mut();
    let mut end: *mut WCHAR = core::ptr::null_mut();
    let mut ret: *const WCHAR = core::ptr::null();
    let mut StringLangId: [WCHAR; 13] = [
    0x0053u16, 0x0074, 0x0072, 0x0069, 0x006E, 0x0067, 0x0073, 0x002E, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000,
];
    let mut Lang: [TCHAR; 5] = [0; 5];

    if *len == 0 {
        *len = 1 as core::ffi::c_uint;
        return core::ptr::addr_of!(percent);
    }
    'not_found: {
        if (*file).strings_section == -(1) {
            break 'not_found;
        }
        strings_section = *(*file).sections.offset((*file).strings_section as isize);
        j = 0 as core::ffi::c_uint;
        line = core::ptr::addr_of_mut!((*strings_section).lines) as *mut line;
        'lp476: while j < (*strings_section).nb_lines {
            'next: {
                if (*line).key_field == -(1) {
                    break 'next;
                }
                if strncmpiW(
                    str,
                    (*(*file).fields.offset((*line).key_field as isize)).text,
                    *len as usize,
                ) != 0
                {
                    break 'next;
                }
                if *(*(*file).fields.offset((*line).key_field as isize))
                    .text
                    .offset(*len as isize)
                    == 0
                {
                    break 'lp476;
                }
            }
            j = j.wrapping_add(1);
            line = line.offset(1);
        }
        if j == (*strings_section).nb_lines || (*line).nb_fields == 0 {
            break 'not_found;
        }
        field = (*file).fields.offset((*line).first_field as isize) as *mut field;
        GetLocaleInfo(
            LOCALE_SYSTEM_DEFAULT,
            LOCALE_ILANGUAGE,
            core::ptr::addr_of_mut!(Lang) as LPWSTR,
            (core::mem::size_of::<[TCHAR; 5]>()
                / core::mem::size_of::<TCHAR>() as usize) as core::ffi::c_int,
        ); // get the current system locale for translated strings

        strcpyW(
            (core::ptr::addr_of_mut!(StringLangId) as *mut WCHAR)
                .offset(8 as isize),
            (core::ptr::addr_of_mut!(Lang) as *mut TCHAR).offset(2 as isize),
        );
        // now you have e.g. Strings.07 for german neutral translations
        i = 0 as core::ffi::c_uint;
        while i < (*file).nb_sections {
            // search in all sections
            if strcmpiW(
                (**(*file).sections.offset(i as isize)).name,
                core::ptr::addr_of_mut!(StringLangId) as *mut WCHAR,
            ) == 0
            {
                // if the section is a Strings.* section
                strings_section = *(*file).sections.offset(i as isize); // select this section for further use
                j = 0 as core::ffi::c_uint;
                line = core::ptr::addr_of_mut!((*strings_section).lines) as *mut line;
                // process all lines in this section
                'lp536: while j < (*strings_section).nb_lines {
                    'next: {
                        if (*line).key_field == -(1) {
                            // if no key then skip
                            break 'next;
                        }
                        if strncmpiW(
                            str,
                            (*(*file).fields.offset((*line).key_field as isize)).text,
                            *len as usize,
                        ) != 0
                        {
                            // if wrong key name, then skip
                            break 'next;
                        }
                        if *(*(*file).fields.offset((*line).key_field as isize))
                            .text
                            .offset(*len as isize)
                            == 0
                        {
                            // if value exist
                            field = (*file).fields.offset((*line).first_field as isize)
                                as *mut field; // then extract value and
                            break 'lp536; // no more search necessary
                        }
                    }
                    j = j.wrapping_add(1);
                    line = line.offset(1);
                }
            }
            i = i.wrapping_add(1);
        }

        strcpyW(
            (core::ptr::addr_of_mut!(StringLangId) as *mut WCHAR)
                .offset(8 as isize),
            core::ptr::addr_of_mut!(Lang) as *mut TCHAR,
        ); // append the Language identifier from GetLocaleInfo
        // now you have e.g. Strings.0407 for german translations
        i = 0 as core::ffi::c_uint;
        while i < (*file).nb_sections {
            // search in all sections
            if strcmpiW(
                (**(*file).sections.offset(i as isize)).name,
                core::ptr::addr_of_mut!(StringLangId) as *mut WCHAR,
            ) == 0
            {
                // if the section is a Strings.* section
                strings_section = *(*file).sections.offset(i as isize); // select this section for further use
                j = 0 as core::ffi::c_uint;
                line = core::ptr::addr_of_mut!((*strings_section).lines) as *mut line;
                // process all lines in this section
                'lp588: while j < (*strings_section).nb_lines {
                    'next: {
                        if (*line).key_field == -(1) {
                            // if no key then skip
                            break 'next;
                        }
                        if strncmpiW(
                            str,
                            (*(*file).fields.offset((*line).key_field as isize)).text,
                            *len as usize,
                        ) != 0
                        {
                            // if wrong key name, then skip
                            break 'next;
                        }
                        if *(*(*file).fields.offset((*line).key_field as isize))
                            .text
                            .offset(*len as isize)
                            == 0
                        {
                            // if value exist
                            field = (*file).fields.offset((*line).first_field as isize)
                                as *mut field; // then extract value and
                            break 'lp588; // no more search necessary
                        }
                    }
                    j = j.wrapping_add(1);
                    line = line.offset(1);
                }
            }
            i = i.wrapping_add(1);
        }
        *len = strlenW((*field).text) as core::ffi::c_uint; // set length
        ret = (*field).text; // return the english or translated string
        return ret;
    }

    // not_found:  check for integer id
    dirid_str = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        ((*len).wrapping_add(1 as core::ffi::c_uint) as SIZE_T)
            .wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
    ) as *mut WCHAR;
    if !dirid_str.is_null() {
        memcpy(
            dirid_str as *mut core::ffi::c_void,
            str as *const core::ffi::c_void,
            (*len as usize).wrapping_mul(core::mem::size_of::<WCHAR>() as usize),
        );
        *dirid_str.offset(*len as isize) = 0 as WCHAR;
        dirid = strtolW(
            dirid_str,
            core::ptr::addr_of_mut!(end),
            10 as core::ffi::c_int,
        ) as core::ffi::c_int;
        if *end == 0 {
            ret = get_dirid_subst(file, dirid, len) as *const WCHAR;
        }
        if no_trailing_slash != 0
            && !ret.is_null()
            && *len != 0
            && *ret.offset((*len).wrapping_sub(1 as core::ffi::c_uint) as isize)
                as core::ffi::c_int
                == '\\' as core::ffi::c_int
        {
            *len = (*len).wrapping_sub(1 as core::ffi::c_uint);
        }
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            dirid_str as PVOID,
        );
        return ret;
    }
    return core::ptr::null();
}

unsafe extern "C" fn PARSER_string_substW(file: *const inf_file, text: *const WCHAR, buffer: *mut WCHAR, mut size: c_uint) -> c_uint {
    let mut start: *const WCHAR;
    let mut subst: *const WCHAR;
    let mut p: *const WCHAR;
    let mut len: c_uint;
    let mut total: c_uint = 0;
    let mut inside: BOOL = (FALSE as i32);

    if buffer.is_null() {
        size = (MAX_STRING_LEN + 1) as c_uint;
    }
    start = text;
    p = start;
    while *p != 0 {
        'body: {
            if *p != '%' as WCHAR {
                break 'body;
            }
            inside = (inside == 0) as BOOL;
            if inside != 0 {
                /* start of a %xx% string */
                len = p.offset_from(start) as c_uint;
                if len > size.wrapping_sub(1) {
                    len = size.wrapping_sub(1);
                }
                if !buffer.is_null() {
                    memcpy(
                        buffer.add(total as usize) as *mut ::core::ffi::c_void,
                        start as *const ::core::ffi::c_void,
                        (len as usize).wrapping_mul(core::mem::size_of::<WCHAR>()),
                    );
                }
                total = total.wrapping_add(len);
                size = size.wrapping_sub(len);
                start = p;
            } else {
                /* end of the %xx% string, find substitution */
                len = (p.offset_from(start) - 1) as c_uint;
                subst = get_string_subst(
                    file,
                    start.offset(1),
                    core::ptr::addr_of_mut!(len),
                    (*p.offset(1) == '\\' as WCHAR) as BOOL,
                );
                if subst.is_null() {
                    subst = start;
                    len = (p.offset_from(start) + 1) as c_uint;
                }
                if len > size.wrapping_sub(1) {
                    len = size.wrapping_sub(1);
                }
                if !buffer.is_null() {
                    memcpy(
                        buffer.add(total as usize) as *mut ::core::ffi::c_void,
                        subst as *const ::core::ffi::c_void,
                        (len as usize).wrapping_mul(core::mem::size_of::<WCHAR>()),
                    );
                }
                total = total.wrapping_add(len);
                size = size.wrapping_sub(len);
                start = p.offset(1);
            }
        }
        p = p.offset(1);
    }

    if start != p {
        /* unfinished string, copy it */
        len = p.offset_from(start) as c_uint;
        if len > size.wrapping_sub(1) {
            len = size.wrapping_sub(1);
        }
        if !buffer.is_null() {
            memcpy(
                buffer.add(total as usize) as *mut ::core::ffi::c_void,
                start as *const ::core::ffi::c_void,
                (len as usize).wrapping_mul(core::mem::size_of::<WCHAR>()),
            );
        }
        total = total.wrapping_add(len);
    }
    if !buffer.is_null() && size != 0 {
        *buffer.add(total as usize) = 0 as WCHAR;
    }
    return total;
}

unsafe extern "C" fn PARSER_string_substA(
    file: *const inf_file,
    text: *const WCHAR,
    buffer: *mut c_char,
    size: c_uint,
) -> c_uint {
    let mut buffW: [WCHAR; (MAX_STRING_LEN + 1) as usize] = [0; (MAX_STRING_LEN + 1) as usize];
    let mut ret: DWORD = 0;

    let len: c_uint = PARSER_string_substW(
        file,
        text,
        core::ptr::addr_of_mut!(buffW) as *mut WCHAR,
        (core::mem::size_of::<[WCHAR; (MAX_STRING_LEN + 1) as usize]>()
            / core::mem::size_of::<WCHAR>()) as c_uint,
    );
    if buffer.is_null() {
        RtlUnicodeToMultiByteSize(
            core::ptr::addr_of_mut!(ret),
            core::ptr::addr_of_mut!(buffW) as *mut WCHAR as PCWCH,
            (len as usize).wrapping_mul(core::mem::size_of::<WCHAR>()) as ULONG,
        );
    } else {
        RtlUnicodeToMultiByteN(
            buffer as PCHAR,
            size.wrapping_sub(1 as c_uint) as ULONG,
            core::ptr::addr_of_mut!(ret),
            core::ptr::addr_of_mut!(buffW) as *mut WCHAR as PCWCH,
            (len as usize).wrapping_mul(core::mem::size_of::<WCHAR>()) as ULONG,
        );
        *buffer.offset(ret as isize) = 0 as c_char;
    }
    return ret as c_uint;
}

unsafe extern "C" fn push_string(file: *mut inf_file, string: *const WCHAR) -> *mut WCHAR {
    let mut ret: *mut WCHAR = (*file).string_pos;
    strcpyW(ret, string);
    (*file).string_pos = (*file)
        .string_pos
        .offset((strlenW(ret) + 1) as isize);
    return ret;
}

unsafe extern "C" fn push_state(parser: *mut parser, state: parser_state) {
    ASSERT!(
        ((*parser).stack_pos as usize)
            < (core::mem::size_of::<[parser_state; 4]>())
                / core::mem::size_of_val(&(*parser).stack[0 as usize])
    );
    let fresh = (*parser).stack_pos;
    (*parser).stack_pos = (*parser).stack_pos + 1;
    (*parser).stack[fresh as usize] = state;
}

unsafe extern "C" fn pop_state(parser: *mut parser) {
    if (*parser).stack_pos == 0 {
        ASSERT!((*parser).stack_pos);
    } else {
    }
    (*parser).stack_pos -= 1;
    (*parser).state = (*parser).stack[(*parser).stack_pos as usize];
}

unsafe extern "C" fn set_state(parser: *mut parser, state: parser_state) -> parser_state {
    let mut ret: parser_state = (*parser).state;
    (*parser).state = state;
    return ret;
}

unsafe extern "C" fn is_eof(parser: *const parser, ptr: *const WCHAR) -> BOOL {
    return (ptr >= (*parser).end || (*ptr) as i32 == CONTROL_Z) as BOOL;
}

unsafe extern "C" fn is_eol(parser: *const parser, ptr: *const WCHAR) -> BOOL {
    return (ptr >= (*parser).end
        || *ptr == (CONTROL_Z as WCHAR)
        || *ptr == ('\n' as WCHAR)) as BOOL;
}

unsafe extern "C" fn push_token(parser: *mut parser, pos: *const WCHAR) -> ::core::ffi::c_int {
    let mut len: ::core::ffi::c_int =
        pos.offset_from((*parser).start) as ::core::ffi::c_longlong as ::core::ffi::c_int;
    let mut src: *const WCHAR = (*parser).start;
    let mut dst: *mut WCHAR =
        (core::ptr::addr_of_mut!((*parser).token) as *mut WCHAR).offset((*parser).token_len as isize);
    if len as ::core::ffi::c_uint
        > (MAX_FIELD_LEN as ::core::ffi::c_uint).wrapping_sub((*parser).token_len)
    {
        len = (MAX_FIELD_LEN as ::core::ffi::c_uint).wrapping_sub((*parser).token_len)
            as ::core::ffi::c_int;
    }
    (*parser).token_len = (*parser).token_len.wrapping_add(len as ::core::ffi::c_uint);
    while len > 0 as ::core::ffi::c_int {
        *dst = (if *src as ::core::ffi::c_int != 0 {
            *src as ::core::ffi::c_int
        } else {
            ' ' as i32
        }) as WCHAR;
        len -= 1;
        dst = dst.offset(1);
        src = src.offset(1);
    }
    *dst = 0 as WCHAR;
    (*parser).start = pos;
    return 0 as ::core::ffi::c_int;
}

unsafe extern "C" fn add_section_from_token(parser: *mut parser) -> ::core::ffi::c_int {
    let mut section_index: ::core::ffi::c_int;
    if (*parser).token_len > (MAX_SECTION_NAME_LEN as u32) {
        (*parser).error = ERROR_SECTION_NAME_TOO_LONG;
        return -1;
    }
    section_index = find_section((*parser).file, core::ptr::addr_of_mut!((*parser).token) as *mut WCHAR);
    if section_index == -1 {
        /* need to create a new one */
        let name: *const WCHAR = push_string((*parser).file, core::ptr::addr_of_mut!((*parser).token) as *mut WCHAR);
        section_index = add_section((*parser).file, name);
        if section_index == -1 {
            (*parser).error = ERROR_NOT_ENOUGH_MEMORY;
            return -1;
        }
    }
    (*parser).token_len = 0;
    (*parser).cur_section = section_index;
    return section_index;
}

unsafe extern "C" fn add_field_from_token(parser: *mut parser, is_key: BOOL) -> *mut field {
    let mut field: *mut field = ::core::ptr::null_mut::<field>();
    let mut text: *mut WCHAR = ::core::ptr::null_mut::<WCHAR>();
    'error: {
        if (*parser).line.is_null() {
            if (*parser).cur_section == -(1 as ::core::ffi::c_int) {
                (*parser).error = ERROR_EXPECTED_SECTION_NAME as ::core::ffi::c_uint;
                return ::core::ptr::null_mut::<field>();
            }
            (*parser).line = add_line((*parser).file, (*parser).cur_section);
            if (*parser).line.is_null() {
                break 'error;
            }
        } else {
            ASSERT!(!is_key);
        }

        text = push_string((*parser).file, &raw mut (*parser).token as *mut WCHAR);
        field = add_field((*parser).file, text);
        if !field.is_null() {
            if is_key == 0 {
                (*(*parser).line).nb_fields += 1;
            } else {
                (*(*parser).line).key_field = (*(*parser).line).first_field;
                (*(*parser).line).first_field += 1;
            }
            (*parser).token_len = 0 as ::core::ffi::c_uint;
            return field;
        }
    }
    (*parser).error = ERROR_NOT_ENOUGH_MEMORY as ::core::ffi::c_uint;
    return ::core::ptr::null_mut::<field>();
}

unsafe extern "C" fn close_current_line(parser: *mut parser) {
    let cur_line: *mut line = (*parser).line;
    if !cur_line.is_null() {
        /* if line has a single field and no key, the field is the key too */
        if (*cur_line).nb_fields == 1 && (*cur_line).key_field == -1 {
            (*cur_line).key_field = (*cur_line).first_field;
        }
    }
    (*parser).line = core::ptr::null_mut();
}

unsafe extern "C" fn line_start_state(parser: *mut parser, pos: *const WCHAR) -> *const WCHAR {
    let mut p: *const WCHAR = core::ptr::null::<WCHAR>();
    p = pos;
    while is_eof(parser, p) == 0 {
        'sw: {
            match *p {
                10 => {
                    (*parser).line_pos = (*parser).line_pos.wrapping_add(1);
                    close_current_line(parser);
                    break 'sw;
                }
                59 => {
                    push_state(parser, LINE_START);
                    set_state(parser, COMMENT);
                    return p.offset(1 as isize);
                }
                91 => {
                    (*parser).start = p.offset(1 as isize);
                    set_state(parser, SECTION_NAME);
                    return p.offset(1 as isize);
                }
                _ => {
                    if isspaceW(*p) != 0 {
                        break 'sw;
                    }
                    if (*parser).cur_section != -(1 as ::core::ffi::c_int) {
                        (*parser).start = p;
                        set_state(parser, KEY_NAME);
                        return p;
                    }
                    if (*parser).broken_line == 0 {
                        (*parser).broken_line = (*parser).line_pos;
                    }
                    break 'sw;
                }
            }
        }
        p = p.offset(1);
    }
    close_current_line(parser);
    return core::ptr::null::<WCHAR>();
}

unsafe extern "C" fn section_name_state(parser: *mut parser, pos: *const WCHAR) -> *const WCHAR {
    let mut p: *const WCHAR;
    p = pos;
    while is_eol(parser, p) == 0 {
        if *p == ']' as WCHAR {
            push_token(parser, p);
            if add_section_from_token(parser) == -1 {
                return core::ptr::null::<WCHAR>();
            }
            push_state(parser, LINE_START);
            set_state(parser, COMMENT);
            return p.offset(1 as isize);
        }
        p = p.offset(1);
    }
    (*parser).error = ERROR_BAD_SECTION_NAME_LINE;
    return core::ptr::null::<WCHAR>();
}

unsafe extern "C" fn key_name_state(
    mut parser: *mut parser,
    mut pos: *const WCHAR,
) -> *const WCHAR {
    let mut p: *const WCHAR = core::ptr::null::<WCHAR>();
    let mut token_end: *const WCHAR = (*parser).start;
    p = pos;
    while is_eol(parser, p) == 0 {
        if *p as i32 == ',' as i32 {
            break;
        }
        match *p as i32 {
            61 => {
                push_token(parser, token_end);
                if add_field_from_token(parser, (TRUE) as BOOL).is_null() {
                    return core::ptr::null::<WCHAR>();
                }
                (*parser).start = p.offset(1 as isize);
                push_state(parser, VALUE_NAME);
                set_state(parser, LEADING_SPACES);
                return p.offset(1 as isize);
            }
            59 => {
                push_token(parser, token_end);
                if add_field_from_token(parser, (FALSE) as BOOL).is_null() {
                    return core::ptr::null::<WCHAR>();
                }
                push_state(parser, LINE_START);
                set_state(parser, COMMENT);
                return p.offset(1 as isize);
            }
            34 => {
                push_token(parser, p);
                (*parser).start = p.offset(1 as isize);
                push_state(parser, KEY_NAME);
                set_state(parser, QUOTES);
                return p.offset(1 as isize);
            }
            92 => {
                push_token(parser, token_end);
                (*parser).start = p;
                push_state(parser, KEY_NAME);
                set_state(parser, EOL_BACKSLASH);
                return p;
            }
            _ => {
                if isspaceW(*p) == 0 {
                    token_end = p.offset(1 as isize);
                } else {
                    push_token(parser, p);
                    push_state(parser, KEY_NAME);
                    set_state(parser, TRAILING_SPACES);
                    return p;
                }
            }
        }
        p = p.offset(1);
    }
    push_token(parser, token_end);
    set_state(parser, VALUE_NAME);
    return p;
}

unsafe extern "C" fn value_name_state(parser: *mut parser, pos: *const WCHAR) -> *const WCHAR {
    let mut p: *const WCHAR = core::ptr::null::<WCHAR>();
    let mut token_end: *const WCHAR = (*parser).start;
    p = pos;
    while is_eol(parser, p) == 0 {
        match *p as i32 {
            59 => {
                push_token(parser, token_end);
                if add_field_from_token(parser, (FALSE) as BOOL).is_null() {
                    return core::ptr::null::<WCHAR>();
                }
                push_state(parser, LINE_START);
                set_state(parser, COMMENT);
                return p.offset(1 as isize);
            }
            44 => {
                push_token(parser, token_end);
                if add_field_from_token(parser, (FALSE) as BOOL).is_null() {
                    return core::ptr::null::<WCHAR>();
                }
                (*parser).start = p.offset(1 as isize);
                push_state(parser, VALUE_NAME);
                set_state(parser, LEADING_SPACES);
                return p.offset(1 as isize);
            }
            34 => {
                push_token(parser, p);
                (*parser).start = p.offset(1 as isize);
                push_state(parser, VALUE_NAME);
                set_state(parser, QUOTES);
                return p.offset(1 as isize);
            }
            92 => {
                push_token(parser, token_end);
                (*parser).start = p;
                push_state(parser, VALUE_NAME);
                set_state(parser, EOL_BACKSLASH);
                return p;
            }
            _ => {
                if isspaceW(*p) == 0 {
                    token_end = p.offset(1 as isize);
                } else {
                    push_token(parser, p);
                    push_state(parser, VALUE_NAME);
                    set_state(parser, TRAILING_SPACES);
                    return p;
                }
            }
        }
        p = p.offset(1);
    }
    push_token(parser, token_end);
    if add_field_from_token(parser, (FALSE) as BOOL).is_null() {
        return core::ptr::null::<WCHAR>();
    }
    set_state(parser, LINE_START);
    return p;
}

unsafe extern "C" fn eol_backslash_state(parser: *mut parser, pos: *const WCHAR) -> *const WCHAR {
    let mut p: *const WCHAR;
    p = pos;
    while is_eof(parser, p) == 0 {
        match *p {
            10 => {
                (*parser).line_pos = (*parser).line_pos.wrapping_add(1);
                (*parser).start = p.offset(1 as isize);
                set_state(parser, LEADING_SPACES);
                return p.offset(1 as isize);
            }
            92 => {}
            59 => {
                push_state(parser, EOL_BACKSLASH);
                set_state(parser, COMMENT);
                return p.offset(1 as isize);
            }
            _ => {
                if isspaceW(*p) == 0 {
                    push_token(parser, p);
                    pop_state(parser);
                    return p;
                }
            }
        }
        p = p.offset(1);
    }
    (*parser).start = p;
    pop_state(parser);
    return p;
}

unsafe extern "C" fn quotes_state(parser: *mut parser, pos: *const WCHAR) -> *const WCHAR {
    let mut p: *const WCHAR = pos;
    while is_eol(parser, p) == 0 {
        if *p == '"' as u16 {
            if p.offset(1 as isize) < (*parser).end
                && *p.offset(1 as isize) == '"' as u16
            {
                push_token(parser, p.offset(1 as isize));
                (*parser).start = p.offset(2 as isize);
                p = p.offset(1);
            } else {
                push_token(parser, p);
                (*parser).start = p.offset(1 as isize);
                pop_state(parser);
                return p.offset(1 as isize);
            }
        }
        p = p.offset(1);
    }
    push_token(parser, p);
    pop_state(parser);
    p
}

unsafe extern "C" fn leading_spaces_state(parser: *mut parser, pos: *const WCHAR) -> *const WCHAR {
    let mut p: *const WCHAR;
    p = pos;
    while is_eol(parser, p) == 0 {
        if *p == '\\' as WCHAR {
            (*parser).start = p;
            set_state(parser, EOL_BACKSLASH);
            return p;
        }
        if isspaceW(*p) == 0 {
            break;
        }
        p = p.offset(1);
    }
    (*parser).start = p;
    pop_state(parser);
    return p;
}

unsafe extern "C" fn trailing_spaces_state(
    mut parser: *mut parser,
    mut pos: *const WCHAR,
) -> *const WCHAR {
    let mut p: *const WCHAR = core::ptr::null::<WCHAR>();
    p = pos;
    while is_eol(parser, p) == 0 {
        if *p == '\\' as WCHAR {
            set_state(parser, EOL_BACKSLASH);
            return p;
        }
        if isspaceW(*p) == 0 {
            break;
        }
        p = p.add(1);
    }
    pop_state(parser);
    return p;
}

unsafe extern "C" fn comment_state(parser: *mut parser, pos: *const WCHAR) -> *const WCHAR {
    let mut p: *const WCHAR = pos;
    while is_eol(parser, p) == 0 {
        p = p.offset(1);
    }
    pop_state(parser);
    return p;
}

unsafe extern "C" fn free_inf_file(file: *mut inf_file) {
    let mut i: ::core::ffi::c_uint;
    i = 0;
    while i < (*file).nb_sections {
        HeapFree(GetProcessHeap(), 0, *(*file).sections.offset(i as isize) as PVOID);
        i = i.wrapping_add(1);
    }
    HeapFree(GetProcessHeap(), 0, (*file).filename as PVOID);
    HeapFree(GetProcessHeap(), 0, (*file).sections as PVOID);
    HeapFree(GetProcessHeap(), 0, (*file).fields as PVOID);
    HeapFree(GetProcessHeap(), 0, (*file).strings as PVOID);
    HeapFree(GetProcessHeap(), 0, file as PVOID);
}

unsafe extern "C" fn parse_buffer(
    mut file: *mut inf_file,
    mut buffer: *const WCHAR,
    mut end: *const WCHAR,
    mut error_line: *mut UINT,
) -> DWORD {
    static mut Strings: [u16; 8] = [0x0053u16, 0x0074, 0x0072, 0x0069, 0x006E, 0x0067, 0x0073, 0x0000];

    let mut parser: parser = core::mem::zeroed();
    let mut pos: *const WCHAR = buffer;

    parser.start = buffer;
    parser.end = end;
    parser.file = file;
    parser.line = core::ptr::null_mut();
    parser.state = LINE_START as _;
    parser.stack_pos = 0 as _;
    parser.cur_section = -(1 as i32);
    parser.line_pos = 1 as _;
    parser.broken_line = 0 as _;
    parser.error = 0 as _;
    parser.token_len = 0 as _;

    /* parser main loop */
    while !pos.is_null() {
        pos = ((parser_funcs[parser.state as usize]).unwrap_unchecked())(core::ptr::addr_of_mut!(parser), pos);
    }

    /* trim excess buffer space */
    if (*file).alloc_sections > (*file).nb_sections {
        let _elem_sections = (*file).sections.wrapping_add(0 as usize);
        (*file).sections = HeapReAlloc(
            GetProcessHeap(),
            0 as DWORD,
            (*file).sections as PVOID,
            ((*file).nb_sections as SIZE_T)
                .wrapping_mul(core::mem::size_of::<section>() as SIZE_T),
        ) as *mut *mut section;
        (*file).alloc_sections = (*file).nb_sections;
    }
    if (*file).alloc_fields > (*file).nb_fields {
        let _elem_fields = (*file).fields.wrapping_add(0 as usize);
        (*file).fields = HeapReAlloc(
            GetProcessHeap(),
            0 as DWORD,
            (*file).fields as PVOID,
            ((*file).nb_fields as SIZE_T).wrapping_mul(core::mem::size_of::<field>() as SIZE_T),
        ) as *mut field;
        (*file).alloc_fields = (*file).nb_fields;
    }
    (*file).strings = HeapReAlloc(
        GetProcessHeap(),
        HEAP_REALLOC_IN_PLACE_ONLY as DWORD,
        (*file).strings as PVOID,
        ((*file).string_pos.offset_from((*file).strings) as core::ffi::c_longlong as SIZE_T)
            .wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
    ) as *mut WCHAR;

    if parser.error != 0 {
        if !error_line.is_null() {
            *error_line = parser.line_pos as UINT;
        }
        return parser.error as DWORD;
    }

    /* find the [strings] section */
    (*file).strings_section =
        find_section(file, core::ptr::addr_of!(Strings) as *const WCHAR);

    if (*file).strings_section == -(1 as i32) && parser.broken_line != 0 {
        if !error_line.is_null() {
            *error_line = parser.broken_line as UINT;
        }
        return ERROR_EXPECTED_SECTION_NAME as DWORD;
    }

    return 0 as DWORD;
}

unsafe extern "C" fn append_inf_file(parent: *mut inf_file, child: *mut inf_file) {
    let mut ppnext: *mut *mut inf_file = core::ptr::addr_of_mut!((*parent).next);
    (*child).next = core::ptr::null_mut();
    loop {
        let next: *mut inf_file = InterlockedCompareExchangePointer(
            ppnext as *mut *mut core::ffi::c_void,
            child as *mut core::ffi::c_void,
            core::ptr::null_mut(),
        ) as *mut inf_file;
        if next.is_null() {
            return;
        }
        ppnext = core::ptr::addr_of_mut!((*next).next);
    }
}

unsafe extern "C" fn parse_file(
    handle: HANDLE,
    error_line: *mut UINT,
    style: DWORD,
) -> *mut inf_file {
    let mut buffer: *mut ::core::ffi::c_void = ::core::ptr::null_mut();
    let mut err: DWORD = 0;
    let mut file: *mut inf_file = ::core::ptr::null_mut();

    let size: DWORD = GetFileSize(handle, ::core::ptr::null_mut());
    let mapping: HANDLE = CreateFileMappingW(
        handle,
        ::core::ptr::null_mut(),
        PAGE_READONLY,
        0,
        size,
        ::core::ptr::null_mut(),
    );
    if mapping.is_null() {
        return ::core::ptr::null_mut();
    }
    buffer = MapViewOfFile(
        mapping,
        FILE_MAP_READ,
        0,
        0,
        (size) as SIZE_T,
    ) as *mut ::core::ffi::c_void;
    NtClose(mapping);
    if buffer.is_null() {
        return ::core::ptr::null_mut();
    }

    'done: {
        file = HeapAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY,
            ::core::mem::size_of::<inf_file>() as SIZE_T,
        ) as *mut inf_file;
        if file.is_null() {
            err = ERROR_NOT_ENOUGH_MEMORY;
            break 'done;
        }

        /* we won't need more strings space than the size of the file,
         * so we can preallocate it here
         */
        (*file).strings = HeapAlloc(
            GetProcessHeap(),
            0,
            (size as SIZE_T).wrapping_mul(::core::mem::size_of::<WCHAR>() as SIZE_T),
        ) as *mut WCHAR;
        if (*file).strings.is_null() {
            err = ERROR_NOT_ENOUGH_MEMORY;
            break 'done;
        }
        (*file).string_pos = (*file).strings;
        (*file).strings_section = -1;

        if RtlIsTextUnicode(buffer, size as INT, ::core::ptr::null_mut()) == 0 {
            let mut utf8_bom: [BYTE; 3] = [
                0xef as BYTE,
                0xbb as BYTE,
                0xbf as BYTE,
            ];
            let mut new_buff: *mut WCHAR = ::core::ptr::null_mut();
            let mut codepage: UINT = CP_ACP as UINT;
            let mut offset: UINT = 0 as UINT;

            if (size as usize) > ::core::mem::size_of::<[BYTE; 3]>() as usize
                && memcmp(
                    buffer,
                    ::core::ptr::addr_of!(utf8_bom) as *const ::core::ffi::c_void,
                    ::core::mem::size_of::<[BYTE; 3]>() as size_t,
                ) == 0
            {
                codepage = CP_UTF8 as UINT;
                offset = ::core::mem::size_of::<[BYTE; 3]>() as UINT;
            }

            new_buff = HeapAlloc(
                GetProcessHeap(),
                0,
                (size as SIZE_T).wrapping_mul(::core::mem::size_of::<WCHAR>() as SIZE_T),
            ) as *mut WCHAR;
            if !new_buff.is_null() {
                let len: DWORD = MultiByteToWideChar(
                    codepage,
                    0,
                    (buffer as *mut ::core::ffi::c_char).offset(offset as isize) as LPCSTR,
                    size.wrapping_sub(offset as DWORD) as ::core::ffi::c_int,
                    new_buff as LPWSTR,
                    size as ::core::ffi::c_int,
                ) as DWORD;
                err = parse_buffer(
                    file,
                    new_buff,
                    new_buff.offset(len as isize),
                    error_line,
                );
                HeapFree(
                    GetProcessHeap(),
                    0,
                    new_buff as PVOID,
                );
            }
        } else {
            let mut new_buff_0: *mut WCHAR = buffer as *mut WCHAR;
            /* UCS-16 files should start with the Unicode BOM; we should skip it */
            if *new_buff_0 as ::core::ffi::c_int == 0xfeff {
                new_buff_0 = new_buff_0.offset(1);
            }
            err = parse_buffer(
                file,
                new_buff_0,
                (buffer as *mut ::core::ffi::c_char).offset(size as isize) as *mut WCHAR,
                error_line,
            );
        }

        if err == 0 {
            /* now check signature */
            let version_index: ::core::ffi::c_int =
                find_section(file, ::core::ptr::addr_of!(Version) as *const WCHAR);
            if version_index != -1 {
                let line: *mut line = find_line(
                    file,
                    version_index,
                    ::core::ptr::addr_of!(Signature) as *const WCHAR,
                );
                if !line.is_null() && (*line).nb_fields > 0 {
                    let field: *mut field = (*file).fields.offset((*line).first_field as isize);
                    if strcmpiW((*field).text, ::core::ptr::addr_of!(Chicago) as *const WCHAR) == 0 {
                        break 'done;
                    }
                    if strcmpiW((*field).text, ::core::ptr::addr_of!(WindowsNT) as *const WCHAR) == 0 {
                        break 'done;
                    }
                    if strcmpiW((*field).text, ::core::ptr::addr_of!(Windows95) as *const WCHAR) == 0 {
                        break 'done;
                    }
                }
            }
            if !error_line.is_null() {
                *error_line = 0 as UINT;
            }
            if (style & INF_STYLE_WIN4 as DWORD) != 0 {
                err = ERROR_WRONG_INF_STYLE as DWORD;
            }
        }
    }

    /* done: */
    UnmapViewOfFile(buffer as LPCVOID);
    if err != 0 {
        if !file.is_null() {
            free_inf_file(file);
        }
        SetLastError(err);
        file = ::core::ptr::null_mut();
    }
    return file;
}
#[no_mangle]
pub unsafe extern "C" fn PARSER_get_inf_filename(hinf: HINF) -> *const WCHAR {
    let mut file: *mut inf_file = hinf as *mut inf_file;
    return (*file).filename as *const WCHAR;
}
#[no_mangle]
pub unsafe extern "C" fn PARSER_get_src_root(hinf: HINF) -> *mut WCHAR {
    let mut len: core::ffi::c_uint = 0;
    let dir: *const WCHAR = get_inf_dir(hinf as *const inf_file, &mut len as *mut core::ffi::c_uint);
    let ret: *mut WCHAR = HeapAlloc(
        GetProcessHeap(),
        0,
        ((len + 1) as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
    ) as *mut WCHAR;
    if !ret.is_null() {
        memcpy(
            ret as *mut core::ffi::c_void,
            dir as *const core::ffi::c_void,
            (len as usize).wrapping_mul(core::mem::size_of::<WCHAR>()),
        );
        *ret.offset(len as isize) = 0 as WCHAR;
    }
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn PARSER_get_dest_dir(context: *mut INFCONTEXT) -> *mut WCHAR {
    let mut dir: *const WCHAR = core::ptr::null();
    let mut ptr: *mut WCHAR = core::ptr::null_mut();
    let mut ret: *mut WCHAR = core::ptr::null_mut();
    let mut dirid: INT = 0;
    let mut len1: u32 = 0;
    let mut len2: DWORD = 0;

    if SetupGetIntField(context as PINFCONTEXT, 1, core::ptr::addr_of_mut!(dirid)) == 0 {
        return core::ptr::null_mut();
    }
    dir = get_dirid_subst(
        (*context).Inf as *const inf_file,
        dirid as core::ffi::c_int,
        core::ptr::addr_of_mut!(len1),
    );
    if dir.is_null() {
        return core::ptr::null_mut();
    }
    if SetupGetStringFieldW(
        context as PINFCONTEXT,
        2,
        core::ptr::null_mut(),
        0,
        core::ptr::addr_of_mut!(len2),
    ) == 0
    {
        len2 = 0;
    }
    ret = HeapAlloc(
        GetProcessHeap(),
        0,
        ((len1 as DWORD)
            .wrapping_add(len2)
            .wrapping_add(1 as DWORD) as SIZE_T)
            .wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
    ) as *mut WCHAR;
    if ret.is_null() {
        return core::ptr::null_mut();
    }
    memcpy(
        ret as *mut core::ffi::c_void,
        dir as *const core::ffi::c_void,
        (len1 as size_t).wrapping_mul(core::mem::size_of::<WCHAR>() as size_t),
    );
    ptr = ret.offset(len1 as isize);
    if len2 != 0
        && ptr > ret
        && *ptr.offset(-(1 as isize)) as core::ffi::c_int != '\\' as core::ffi::c_int
    {
        let fresh22 = ptr;
        ptr = ptr.offset(1);
        *fresh22 = '\\' as u16;
    }
    if SetupGetStringFieldW(
        context as PINFCONTEXT,
        2,
        ptr as PWSTR,
        len2,
        core::ptr::null_mut(),
    ) == 0
    {
        *ptr = 0 as WCHAR;
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupOpenInfFileA(
    name: PCSTR,
    class: PCSTR,
    style: DWORD,
    error: *mut UINT,
) -> HINF {
    let mut nameW: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut::<WCHAR>(),
    };
    let mut classW: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut::<WCHAR>(),
    };
    let mut ret: HINF = INVALID_HANDLE_VALUE;

    classW.Buffer = core::ptr::null_mut::<WCHAR>();
    if !class.is_null() && RtlCreateUnicodeStringFromAsciiz(core::ptr::addr_of_mut!(classW), class as PCSZ) == 0 {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY as DWORD);
        return ret;
    }
    if RtlCreateUnicodeStringFromAsciiz(core::ptr::addr_of_mut!(nameW), name as PCSZ) != 0 {
        ret = SetupOpenInfFileW(
            nameW.Buffer as PCWSTR,
            classW.Buffer as PCWSTR,
            style,
            error,
        );
        RtlFreeUnicodeString(core::ptr::addr_of_mut!(nameW));
    }
    RtlFreeUnicodeString(core::ptr::addr_of_mut!(classW));
    return ret;
}

unsafe extern "C" fn PARSER_GetInfClassW(
    mut hInf: HINF,
    mut ClassGuid: LPGUID,
    mut ClassName: PWSTR,
    mut ClassNameSize: DWORD,
    mut RequiredSize: PDWORD,
) -> BOOL {
    let mut requiredSize: DWORD = 0;
    let mut guidW: [WCHAR; (MAX_GUID_STRING_LEN + 1) as _] =
        [0; (MAX_GUID_STRING_LEN + 1) as _];
    let mut ret: BOOL = FALSE as BOOL;

    'cleanup: {
        /* Read class Guid */
        if SetupGetLineTextW(
            ::core::ptr::null_mut::<_INFCONTEXT>(),
            hInf,
            &raw const Version as PCWSTR,
            &raw const ClassGUID as PCWSTR,
            &raw mut guidW as PWSTR,
            ::core::mem::size_of::<[WCHAR; (MAX_GUID_STRING_LEN + 1) as _]>() as DWORD,
            ::core::ptr::null_mut::<DWORD>(),
        ) == 0
        {
            break 'cleanup;
        }
        guidW[37 as usize] = '\0' as WCHAR; /* Replace the } by a NULL character */
        if UuidFromStringW(
            (&raw mut guidW as *mut WCHAR).offset(1 as isize) as RPC_WSTR,
            ClassGuid as *mut UUID,
        ) != RPC_S_OK as RPC_STATUS
        {
            break 'cleanup;
        }

        /* Read class name */
        ret = SetupGetLineTextW(
            ::core::ptr::null_mut::<_INFCONTEXT>(),
            hInf,
            &raw const Version as PCWSTR,
            &raw const Class as PCWSTR,
            ClassName,
            ClassNameSize,
            &raw mut requiredSize,
        );
        if ret != 0 && ClassName.is_null() && ClassNameSize == 0 as DWORD {
            if !RequiredSize.is_null() {
                *RequiredSize = requiredSize;
            }
            SetLastError(ERROR_INSUFFICIENT_BUFFER as DWORD);
            ret = FALSE as BOOL;
            break 'cleanup;
        }
        if ret == 0 {
            if GetLastError() == ERROR_INSUFFICIENT_BUFFER as DWORD {
                if !RequiredSize.is_null() {
                    *RequiredSize = requiredSize;
                }
                break 'cleanup;
            } else if SetupDiClassNameFromGuidW(
                ClassGuid as *const GUID,
                ClassName,
                ClassNameSize,
                &raw mut requiredSize,
            ) == 0
            {
                if GetLastError() == ERROR_INSUFFICIENT_BUFFER as DWORD {
                    if !RequiredSize.is_null() {
                        *RequiredSize = requiredSize;
                    }
                    break 'cleanup;
                }
                /* Return a NULL class name */
                if !RequiredSize.is_null() {
                    *RequiredSize = 1 as DWORD;
                }
                if ClassNameSize < 1 as DWORD {
                    SetLastError(ERROR_INSUFFICIENT_BUFFER as DWORD);
                    break 'cleanup;
                }
                memcpy(
                    ClassGuid as *mut ::core::ffi::c_void,
                    &raw const GUID_NULL as *const ::core::ffi::c_void,
                    ::core::mem::size_of::<GUID>() as size_t,
                );
                *ClassName = UNICODE_NULL;
            }
        }

        ret = TRUE as BOOL;
    }

    TRACE!("Returning %d\n", ret);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupOpenInfFileW(
    name: PCWSTR,
    class: PCWSTR,
    style: DWORD,
    error: *mut UINT,
) -> HINF {
    let mut file: *mut inf_file = core::ptr::null_mut();
    let mut handle: HANDLE = core::ptr::null_mut();
    let mut path: *mut WCHAR = core::ptr::null_mut();
    let mut p: *mut WCHAR = core::ptr::null_mut();
    let mut len: UINT = 0;

    {
    }

    if style & !(INF_STYLE_OLDNT | INF_STYLE_WIN4) != 0 {
        SetLastError(ERROR_INVALID_PARAMETER);
        return (INVALID_HANDLE_VALUE) as HINF;
    }
    if name.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return (INVALID_HANDLE_VALUE) as HINF;
    }

    if strchrW(name, '\\' as WCHAR) != core::ptr::null_mut()
        || strchrW(name, '/' as WCHAR) != core::ptr::null_mut()
    {
        len = GetFullPathNameW(name, 0, core::ptr::null_mut(), core::ptr::null_mut());
        if len == 0 {
            return INVALID_HANDLE_VALUE;
        }
        path = HeapAlloc(
            GetProcessHeap(),
            0,
            (len as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
        ) as *mut WCHAR;
        if path.is_null() {
            SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            return INVALID_HANDLE_VALUE;
        }
        GetFullPathNameW(name, len, path, core::ptr::null_mut());
        handle = CreateFileW(
            path,
            GENERIC_READ,
            FILE_SHARE_READ,
            core::ptr::null_mut(),
            OPEN_EXISTING,
            0,
            core::ptr::null_mut(),
        );
    } else {
        static Inf: [u16; 6] = [0x005Cu16, 0x0069, 0x006E, 0x0066, 0x005C, 0x0000];
        static System32: [u16; 11] = [
    0x005Cu16, 0x0073, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x0033, 0x0032, 0x005C,
    0x0000,
];

        len = (GetWindowsDirectoryW(core::ptr::null_mut(), 0) as usize
            + strlenW(name) as usize
            + 12 as usize) as UINT;
        path = HeapAlloc(
            GetProcessHeap(),
            0,
            (len as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
        ) as *mut WCHAR;
        if path.is_null() {
            SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            return INVALID_HANDLE_VALUE;
        }
        GetWindowsDirectoryW(path, len);
        p = path.offset(strlenW(path) as isize);
        strcpyW(p, Inf.as_ptr());
        strcatW(p, name);
        handle = CreateFileW(
            path,
            GENERIC_READ,
            FILE_SHARE_READ,
            core::ptr::null_mut(),
            OPEN_EXISTING,
            0,
            core::ptr::null_mut(),
        );
        if handle == INVALID_HANDLE_VALUE {
            strcpyW(p, System32.as_ptr());
            strcatW(p, name);
            handle = CreateFileW(
                path,
                GENERIC_READ,
                FILE_SHARE_READ,
                core::ptr::null_mut(),
                OPEN_EXISTING,
                0,
                core::ptr::null_mut(),
            );
        }
    }

    if handle != INVALID_HANDLE_VALUE {
        file = parse_file(handle, error, style);
        CloseHandle(handle);
    }
    if file.is_null() {
        HeapFree(GetProcessHeap(), 0, path as PVOID);
        return INVALID_HANDLE_VALUE;
    }

    (*file).filename = path;

    if !class.is_null() {
        let mut ClassGuid: GUID = core::mem::zeroed();
        let mut ClassName: LPWSTR = HeapAlloc(
            GetProcessHeap(),
            0,
            ((strlenW(class) as SIZE_T).wrapping_add(1 as SIZE_T))
                .wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
        ) as LPWSTR;
        if ClassName.is_null() {
            /* Not enough memory */
            SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            SetupCloseInfFile(file as HINF);
            return INVALID_HANDLE_VALUE;
        } else if PARSER_GetInfClassW(
            file as HINF,
            core::ptr::addr_of_mut!(ClassGuid),
            ClassName,
            (strlenW(class) + 1) as DWORD,
            core::ptr::null_mut(),
        ) == 0
        {
            /* Unable to get class name in .inf file */
            HeapFree(GetProcessHeap(), 0, ClassName as PVOID);
            SetLastError(ERROR_CLASS_MISMATCH);
            SetupCloseInfFile(file as HINF);
            return INVALID_HANDLE_VALUE;
        } else if strcmpW(class, ClassName) != 0 {
            /* Provided name name is not the expected one */
            HeapFree(GetProcessHeap(), 0, ClassName as PVOID);
            SetLastError(ERROR_CLASS_MISMATCH);
            SetupCloseInfFile(file as HINF);
            return INVALID_HANDLE_VALUE;
        }
        HeapFree(GetProcessHeap(), 0, ClassName as PVOID);
    }

    SetLastError(0);
    return file as HINF;
}

#[no_mangle]
pub unsafe extern "system" fn SetupOpenAppendInfFileA(
    name: PCSTR,
    parent_hinf: HINF,
    error: *mut UINT,
) -> BOOL {
    let mut child_hinf: HINF = core::ptr::null_mut();
    if name.is_null() {
        return SetupOpenAppendInfFileW(core::ptr::null(), parent_hinf, error);
    }
    child_hinf = SetupOpenInfFileA(
        name,
        core::ptr::null(),
        INF_STYLE_WIN4 as DWORD,
        error,
    );
    if child_hinf == INVALID_HANDLE_VALUE {
        return (FALSE) as BOOL;
    }
    append_inf_file(parent_hinf as *mut inf_file, child_hinf as *mut inf_file);
    if (*__wine_dbch___default).flags as core::ffi::c_int
        & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/setupapi/parser.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"SetupOpenAppendInfFileA\0" as *const u8 as *const core::ffi::c_char,
            1345 as core::ffi::c_int,
            b"%p: appended %s (%p)\n\0" as *const u8 as *const core::ffi::c_char,
            parent_hinf,
            debugstr_a(name as *const core::ffi::c_char),
            child_hinf,
        );
    }
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetupOpenAppendInfFileW(
    name: PCWSTR,
    parent_hinf: HINF,
    error: *mut UINT,
) -> BOOL {
    let mut child_hinf: HINF = core::ptr::null_mut::<core::ffi::c_void>();
    if name.is_null() {
        let mut context: INFCONTEXT = _INFCONTEXT {
            Inf: core::ptr::null_mut::<core::ffi::c_void>(),
            CurrentInf: core::ptr::null_mut::<core::ffi::c_void>(),
            Section: 0,
            Line: 0,
        };
        let mut filename: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
        let mut idx: core::ffi::c_int = 1;

        if SetupFindFirstLineW(
            parent_hinf,
            core::ptr::addr_of!(Version) as PCWSTR,
            core::ptr::addr_of!(LayoutFile) as PCWSTR,
            core::ptr::addr_of_mut!(context),
        ) == 0
        {
            return (FALSE) as BOOL;
        }
        loop {
            let fresh = idx;
            idx = idx + 1;
            if !(SetupGetStringFieldW(
                core::ptr::addr_of_mut!(context),
                fresh as DWORD,
                core::ptr::addr_of_mut!(filename) as PWSTR,
                (core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>() as usize)
                    .wrapping_div(core::mem::size_of::<WCHAR>() as usize) as DWORD,
                core::ptr::null_mut::<DWORD>(),
            ) != 0)
            {
                break;
            }
            child_hinf = SetupOpenInfFileW(
                core::ptr::addr_of_mut!(filename) as *mut WCHAR as PCWSTR,
                core::ptr::null::<WCHAR>(),
                INF_STYLE_WIN4 as DWORD,
                error,
            );
            if child_hinf == INVALID_HANDLE_VALUE {
                return (FALSE) as BOOL;
            }
            append_inf_file(parent_hinf as *mut inf_file, child_hinf as *mut inf_file);
            if (*__wine_dbch___default).flags as core::ffi::c_int
                & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int
                != 0
            {
                let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl,
                    __dbch,
                    b"dll/win32/setupapi/parser.c\0"
                        as *const u8 as *const core::ffi::c_char,
                    b"SetupOpenAppendInfFileW\0" as *const u8 as *const core::ffi::c_char,
                    1370 as core::ffi::c_int,
                    b"%p: appended %s (%p)\n\0" as *const u8 as *const core::ffi::c_char,
                    parent_hinf,
                    debugstr_w(core::ptr::addr_of_mut!(filename) as *mut WCHAR),
                    child_hinf,
                );
            }
        }
        return (TRUE) as BOOL;
    }
    child_hinf = SetupOpenInfFileW(
        name,
        core::ptr::null::<WCHAR>(),
        INF_STYLE_WIN4 as DWORD,
        error,
    );
    if child_hinf == INVALID_HANDLE_VALUE {
        return (FALSE) as BOOL;
    }
    append_inf_file(parent_hinf as *mut inf_file, child_hinf as *mut inf_file);
    if (*__wine_dbch___default).flags as core::ffi::c_int
        & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int
        != 0
    {
        let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl_0,
            __dbch_0,
            b"dll/win32/setupapi/parser.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"SetupOpenAppendInfFileW\0" as *const u8 as *const core::ffi::c_char,
            1377 as core::ffi::c_int,
            b"%p: appended %s (%p)\n\0" as *const u8 as *const core::ffi::c_char,
            parent_hinf,
            debugstr_w(name as *const WCHAR),
            child_hinf,
        );
    }
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetupOpenMasterInf() -> HINF {
    static Layout: [WCHAR; 16] = [
    0x005Cu16, 0x0069, 0x006E, 0x0066, 0x005C, 0x006C, 0x0061, 0x0079, 0x006F, 0x0075,
    0x0074, 0x002E, 0x0069, 0x006E, 0x0066, 0x0000,
];
    let mut Buffer: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    GetWindowsDirectoryW(Buffer.as_mut_ptr(), MAX_PATH);
    strcatW(Buffer.as_mut_ptr(), Layout.as_ptr() as PCWSTR);
    return SetupOpenInfFileW(
        Buffer.as_ptr() as PCWSTR,
        core::ptr::null(),
        INF_STYLE_WIN4,
        core::ptr::null_mut(),
    );
}

#[no_mangle]
pub unsafe extern "system" fn SetupCloseInfFile(hinf: HINF) {
    let mut file: *mut inf_file = hinf as *mut inf_file;

    if hinf.is_null() || hinf == INVALID_HANDLE_VALUE as HINF {
        return;
    }

    free_inf_file(file);
}

#[no_mangle]
pub unsafe extern "system" fn SetupEnumInfSectionsA(
    mut hinf: HINF,
    mut index: UINT,
    mut buffer: PSTR,
    mut size: DWORD,
    mut need: *mut DWORD,
) -> BOOL {
    let mut file: *mut inf_file = hinf as *mut inf_file;

    file = hinf as *mut inf_file;
    while !file.is_null() {
        if index < (*file).nb_sections {
            let mut len: DWORD = WideCharToMultiByte(
                CP_ACP as UINT,
                0 as DWORD,
                (**(*file).sections.offset(index as isize)).name as LPCWSTR,
                -(1 as ::core::ffi::c_int),
                ::core::ptr::null_mut::<CHAR>(),
                0 as ::core::ffi::c_int,
                ::core::ptr::null::<CHAR>(),
                ::core::ptr::null_mut::<BOOL>(),
            ) as DWORD;
            if !need.is_null() {
                *need = len;
            }
            if buffer.is_null() {
                if size == 0 {
                    return (TRUE) as BOOL;
                }
                SetLastError(ERROR_INVALID_USER_BUFFER as DWORD);
                return (FALSE) as BOOL;
            }
            if len > size {
                SetLastError(ERROR_INSUFFICIENT_BUFFER as DWORD);
                return (FALSE) as BOOL;
            }
            WideCharToMultiByte(
                CP_ACP as UINT,
                0 as DWORD,
                (**(*file).sections.offset(index as isize)).name as LPCWSTR,
                -(1 as ::core::ffi::c_int),
                buffer as LPSTR,
                size as ::core::ffi::c_int,
                ::core::ptr::null::<CHAR>(),
                ::core::ptr::null_mut::<BOOL>(),
            );
            return (TRUE) as BOOL;
        }
        index = (index as ::core::ffi::c_uint).wrapping_sub((*file).nb_sections) as UINT as UINT;
        file = (*file).next;
    }
    SetLastError(ERROR_NO_MORE_ITEMS as DWORD);
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetupEnumInfSectionsW(
    mut hinf: HINF,
    mut index: UINT,
    mut buffer: PWSTR,
    mut size: DWORD,
    mut need: *mut DWORD,
) -> BOOL {
    let mut file: *mut inf_file = hinf as *mut inf_file;

    file = hinf as *mut inf_file;
    while !file.is_null() {
        if index < (*file).nb_sections {
            let mut len: DWORD = (strlenW((**(*file).sections.add(index as usize)).name) + 1) as _;
            if !need.is_null() {
                *need = len;
            }
            if buffer.is_null() {
                if size == 0 {
                    return (TRUE) as BOOL;
                }
                SetLastError(ERROR_INVALID_USER_BUFFER);
                return (FALSE) as BOOL;
            }
            if len > size {
                SetLastError(ERROR_INSUFFICIENT_BUFFER);
                return (FALSE) as BOOL;
            }
            memcpy(
                buffer as *mut core::ffi::c_void,
                (**(*file).sections.add(index as usize)).name as *const core::ffi::c_void,
                (len as usize) * core::mem::size_of::<WCHAR>(),
            );
            return (TRUE) as BOOL;
        }
        index = index.wrapping_sub((*file).nb_sections);
        file = (*file).next;
    }
    SetLastError(ERROR_NO_MORE_ITEMS);
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetupGetLineCountA(hinf: HINF, name: PCSTR) -> LONG {
    let mut sectionW: _UNICODE_STRING = core::mem::zeroed();
    let mut ret: LONG = -(1 as LONG);

    if RtlCreateUnicodeStringFromAsciiz(core::ptr::addr_of_mut!(sectionW), name) == 0 {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY as DWORD);
    } else {
        ret = SetupGetLineCountW(hinf, sectionW.Buffer as PCWSTR);
        RtlFreeUnicodeString(core::ptr::addr_of_mut!(sectionW));
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupGetLineCountW(mut hinf: HINF, mut section: PCWSTR) -> LONG {
    let mut file: *mut inf_file = hinf as *mut inf_file;
    let mut section_index: ::core::ffi::c_int = 0;
    let mut ret: LONG = -1 as LONG;
    file = hinf as *mut inf_file;
    while !file.is_null() {
        section_index = find_section(file, section as *const WCHAR);
        if !(section_index == -1) {
            if ret == -1 as LONG {
                ret = 0 as LONG;
            }
            ret = (ret as ::core::ffi::c_uint).wrapping_add(
                (**(*file).sections.offset(section_index as isize)).nb_lines as ::core::ffi::c_uint,
            ) as LONG;
        }
        file = (*file).next;
    }
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/setupapi/parser.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"SetupGetLineCountW\0" as *const u8 as *const ::core::ffi::c_char,
            1513 as ::core::ffi::c_int,
            b"(%p,%s) returning %d\n\0" as *const u8 as *const ::core::ffi::c_char,
            hinf,
            debugstr_w(section as *const WCHAR),
            ret,
        );
    }
    SetLastError(
        (if ret == -1 as LONG {
            ERROR_SECTION_NOT_FOUND
        } else {
            0
        }) as DWORD,
    );
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupGetLineByIndexA(hinf: HINF, section: PCSTR, index: DWORD, context: *mut INFCONTEXT) -> BOOL {
    let mut sectionW: _UNICODE_STRING = core::mem::zeroed();
    let mut ret: BOOL = (FALSE as i32);

    if RtlCreateUnicodeStringFromAsciiz(core::ptr::addr_of_mut!(sectionW), section as PCSZ) == 0 {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY as DWORD);
    } else {
        ret = SetupGetLineByIndexW(hinf, sectionW.Buffer as PCWSTR, index, context);
        RtlFreeUnicodeString(core::ptr::addr_of_mut!(sectionW));
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupGetLineByIndexW(
    hinf: HINF,
    section: PCWSTR,
    mut index: DWORD,
    context: *mut INFCONTEXT,
) -> BOOL {
    let mut file: *mut inf_file = hinf as *mut inf_file;
    let mut section_index: i32 = 0;
    file = hinf as *mut inf_file;
    while !file.is_null() {
        section_index = find_section(file, section as *const WCHAR);
        if !(section_index == -1) {
            if index < (*(*(*file).sections.offset(section_index as isize))).nb_lines as DWORD {
                (*context).Inf = hinf as PVOID;
                (*context).CurrentInf = file as PVOID;
                (*context).Section = section_index as UINT;
                (*context).Line = index as UINT;
                SetLastError(0 as DWORD);
                return (TRUE) as BOOL;
            }
            index = index.wrapping_sub(
                (*(*(*file).sections.offset(section_index as isize))).nb_lines as DWORD,
            );
        }
        file = (*file).next;
    }
    SetLastError(ERROR_LINE_NOT_FOUND as DWORD);
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetupFindFirstLineA(hinf: HINF, section: PCSTR, key: PCSTR, context: *mut INFCONTEXT) -> BOOL {
    let mut sectionW: _UNICODE_STRING = core::mem::zeroed();
    let mut keyW: _UNICODE_STRING = core::mem::zeroed();
    let mut ret: BOOL = (FALSE as i32);

    if RtlCreateUnicodeStringFromAsciiz(core::ptr::addr_of_mut!(sectionW), section as PCSZ) == 0 {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY as DWORD);
        return (FALSE) as BOOL;
    }

    if key.is_null() {
        ret = SetupFindFirstLineW(hinf, sectionW.Buffer as PCWSTR, core::ptr::null_mut(), context);
    } else {
        if RtlCreateUnicodeStringFromAsciiz(core::ptr::addr_of_mut!(keyW), key as PCSZ) != 0 {
            ret = SetupFindFirstLineW(hinf, sectionW.Buffer as PCWSTR, keyW.Buffer as PCWSTR, context);
            RtlFreeUnicodeString(core::ptr::addr_of_mut!(keyW));
        } else {
            SetLastError(ERROR_NOT_ENOUGH_MEMORY as DWORD);
        }
    }
    RtlFreeUnicodeString(core::ptr::addr_of_mut!(sectionW));
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupFindFirstLineW(
    hinf: HINF,
    section: PCWSTR,
    key: PCWSTR,
    context: *mut INFCONTEXT,
) -> BOOL {
    let mut file: *mut inf_file = ::core::ptr::null_mut::<inf_file>();
    let mut section_index: ::core::ffi::c_int = 0;
    file = hinf as *mut inf_file;
    while !file.is_null() {
        'next: {
            section_index = find_section(file, section);
            if section_index == -1 {
                break 'next;
            }
            if !key.is_null() {
                let mut ctx: INFCONTEXT = core::mem::zeroed();
                ctx.Inf = hinf as PVOID;
                ctx.CurrentInf = file as PVOID;
                ctx.Section = section_index as UINT;
                ctx.Line = (-(1 as ::core::ffi::c_int)) as UINT;
                return SetupFindNextMatchLineW(core::ptr::addr_of_mut!(ctx), key, context as PINFCONTEXT);
            }
            if (**(*file).sections.offset(section_index as isize)).nb_lines != 0 {
                (*context).Inf = hinf as PVOID;
                (*context).CurrentInf = file as PVOID;
                (*context).Section = section_index as UINT;
                (*context).Line = 0 as UINT;
                SetLastError(0 as DWORD);
                if ((*__wine_dbch___default).flags as ::core::ffi::c_int
                    & ((1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int))
                    != 0
                {
                    let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
                    ros_dbg_log(
                        __dbcl,
                        __dbch,
                        b"dll/win32/setupapi/parser.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"SetupFindFirstLineW\0" as *const u8 as *const ::core::ffi::c_char,
                        1625 as ::core::ffi::c_int,
                        b"(%p,%s,%s): returning %d/0\n\0" as *const u8 as *const ::core::ffi::c_char,
                        hinf,
                        debugstr_w(section as *const WCHAR),
                        debugstr_w(key as *const WCHAR),
                        section_index,
                    );
                }
                return TRUE as BOOL;
            }
        }
        file = (*file).next;
    }
    if ((*__wine_dbch___default).flags as ::core::ffi::c_int
        & ((1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int))
        != 0
    {
        let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl_0,
            __dbch_0,
            b"dll/win32/setupapi/parser.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"SetupFindFirstLineW\0" as *const u8 as *const ::core::ffi::c_char,
            1629 as ::core::ffi::c_int,
            b"(%p,%s,%s): not found\n\0" as *const u8 as *const ::core::ffi::c_char,
            hinf,
            debugstr_w(section as *const WCHAR),
            debugstr_w(key as *const WCHAR),
        );
    }
    SetLastError(ERROR_LINE_NOT_FOUND as DWORD);
    return FALSE as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetupFindNextLine(context_in: PINFCONTEXT, context_out: PINFCONTEXT) -> BOOL {
    let mut file: *mut inf_file = (*context_in).CurrentInf as *mut inf_file;
    let mut section: *mut section = core::ptr::null_mut();
    'body: {
        if (*context_in).Section >= (*file).nb_sections {
            break 'body;
        }
        section = *(*file).sections.offset((*context_in).Section as isize);
        if (*context_in).Line.wrapping_add(1) < (*section).nb_lines {
            if context_out != context_in {
                *context_out = *context_in;
            }
            (*context_out).Line = (*context_out).Line.wrapping_add(1);
            SetLastError(0);
            return (TRUE) as BOOL;
        }
        /* now search the appended files */
        file = (*file).next;
        while !file.is_null() {
            let mut section_index: core::ffi::c_int = find_section(file, (*section).name);
            if section_index != -1 {
                if (*(*(*file).sections.offset(section_index as isize))).nb_lines != 0 {
                    (*context_out).Inf = (*context_in).Inf;
                    (*context_out).CurrentInf = file as PVOID;
                    (*context_out).Section = section_index as UINT;
                    (*context_out).Line = 0;
                    SetLastError(0);
                    return (TRUE) as BOOL;
                }
            }
            file = (*file).next;
        }
    }
    /* error: */
    SetLastError(ERROR_LINE_NOT_FOUND);
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetupFindNextMatchLineA(context_in: PINFCONTEXT, key: PCSTR, context_out: PINFCONTEXT) -> BOOL {
    let mut keyW: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut ret: BOOL = (FALSE as i32);
    if key.is_null() {
        return SetupFindNextLine(context_in, context_out);
    }
    if RtlCreateUnicodeStringFromAsciiz(core::ptr::addr_of_mut!(keyW), key as PCSZ) == 0 {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY as DWORD);
    } else {
        ret = SetupFindNextMatchLineW(context_in, keyW.Buffer as PCWSTR, context_out);
        RtlFreeUnicodeString(core::ptr::addr_of_mut!(keyW));
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupFindNextMatchLineW(
    context_in: PINFCONTEXT,
    key: PCWSTR,
    context_out: PINFCONTEXT,
) -> BOOL {
    let mut file: *mut inf_file = (*context_in).CurrentInf as *mut inf_file;
    let mut section: *mut section = core::ptr::null_mut();
    let mut line: *mut line = core::ptr::null_mut();
    let mut i: u32 = 0;

    if key.is_null() {
        return SetupFindNextLine(context_in, context_out);
    }

    'error: {
        if (*context_in).Section >= (*file).nb_sections {
            break 'error;
        }

        section = *(*file).sections.offset((*context_in).Section as isize);

        i = (*context_in).Line.wrapping_add(1);
        line = (core::ptr::addr_of_mut!((*section).lines) as *mut line).offset(i as isize);
        while i < (*section).nb_lines {
            if !((*line).key_field == -(1)) {
                if strcmpiW(key, (*(*file).fields.offset((*line).key_field as isize)).text) == 0 {
                    if context_out != context_in {
                        *context_out = *context_in;
                    }
                    (*context_out).Line = i;
                    SetLastError(0 as DWORD);
                    if ((*__wine_dbch___default).flags as ::core::ffi::c_int
                        & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_TRACE as ::core::ffi::c_int)))
                        != 0
                    {
                        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
                        ros_dbg_log(
                            __dbcl,
                            __dbch,
                            b"dll/win32/setupapi/parser.c\0"
                                as *const u8 as *const ::core::ffi::c_char,
                            b"SetupFindNextMatchLineW\0" as *const u8 as *const ::core::ffi::c_char,
                            1724 as ::core::ffi::c_int,
                            b"(%p,%s,%s): returning %d\n\0" as *const u8 as *const ::core::ffi::c_char,
                            file,
                            debugstr_w((*section).name),
                            debugstr_w(key as *const WCHAR),
                            i,
                        );
                    }
                    return (TRUE) as BOOL;
                }
            }
            i = i.wrapping_add(1);
            line = line.offset(1);
        }

        /* now search the appended files */

        file = (*file).next;
        while !file.is_null() {
            let section_index: ::core::ffi::c_int = find_section(file, (*section).name);
            if !(section_index == -(1)) {
                section = *(*file).sections.offset(section_index as isize);
                i = 0;
                line = core::ptr::addr_of_mut!((*section).lines) as *mut line;
                while i < (*section).nb_lines {
                    if !((*line).key_field == -(1)) {
                        if strcmpiW(key, (*(*file).fields.offset((*line).key_field as isize)).text) == 0 {
                            (*context_out).Inf = (*context_in).Inf;
                            (*context_out).CurrentInf = file as PVOID;
                            (*context_out).Section = section_index as UINT;
                            (*context_out).Line = i;
                            SetLastError(0 as DWORD);
                            if ((*__wine_dbch___default).flags as ::core::ffi::c_int
                                & ((1 as ::core::ffi::c_int)
                                    << (__WINE_DBCL_TRACE as ::core::ffi::c_int)))
                                != 0
                            {
                                let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                                let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
                                ros_dbg_log(
                                    __dbcl_0,
                                    __dbch_0,
                                    b"dll/win32/setupapi/parser.c\0"
                                        as *const u8 as *const ::core::ffi::c_char,
                                    b"SetupFindNextMatchLineW\0" as *const u8 as *const ::core::ffi::c_char,
                                    1747 as ::core::ffi::c_int,
                                    b"(%p,%s,%s): returning %d/%d\n\0" as *const u8 as *const ::core::ffi::c_char,
                                    file,
                                    debugstr_w((*section).name),
                                    debugstr_w(key as *const WCHAR),
                                    section_index,
                                    i,
                                );
                            }
                            return (TRUE) as BOOL;
                        }
                    }
                    i = i.wrapping_add(1);
                    line = line.offset(1);
                }
            }
            file = (*file).next;
        }

        if ((*__wine_dbch___default).flags as ::core::ffi::c_int
            & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_TRACE as ::core::ffi::c_int)))
            != 0
        {
            let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_1: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl_1,
                __dbch_1,
                b"dll/win32/setupapi/parser.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"SetupFindNextMatchLineW\0" as *const u8 as *const ::core::ffi::c_char,
                1753 as ::core::ffi::c_int,
                b"(%p,%s,%s): not found\n\0" as *const u8 as *const ::core::ffi::c_char,
                (*context_in).CurrentInf,
                debugstr_w((*section).name),
                debugstr_w(key as *const WCHAR),
            );
        }
    }

    SetLastError(ERROR_LINE_NOT_FOUND as DWORD);
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetupGetLineTextW(
    mut context: PINFCONTEXT,
    mut hinf: HINF,
    mut section_name: PCWSTR,
    mut key_name: PCWSTR,
    mut buffer: PWSTR,
    mut size: DWORD,
    mut required: PDWORD,
) -> BOOL {
    let mut file: *mut inf_file = core::ptr::null_mut::<inf_file>();
    let mut line: *mut line = core::ptr::null_mut::<line>();
    let mut field: *mut field = core::ptr::null_mut::<field>();
    let mut i: core::ffi::c_int = 0;
    let mut total: DWORD = 0;

    if context.is_null() {
        let mut new_context: INFCONTEXT = _INFCONTEXT {
            Inf: core::ptr::null_mut::<core::ffi::c_void>(),
            CurrentInf: core::ptr::null_mut::<core::ffi::c_void>(),
            Section: 0,
            Line: 0,
        };
        if SetupFindFirstLineW(hinf, section_name, key_name, core::ptr::addr_of_mut!(new_context)) == 0 {
            return (FALSE) as BOOL;
        }
        file = new_context.CurrentInf as *mut inf_file;
        line = get_line(
            file,
            new_context.Section as core::ffi::c_uint,
            new_context.Line as core::ffi::c_uint,
        );
    } else {
        file = (*context).CurrentInf as *mut inf_file;
        line = get_line(
            file,
            (*context).Section as core::ffi::c_uint,
            (*context).Line as core::ffi::c_uint,
        );
        if line.is_null() {
            SetLastError(ERROR_LINE_NOT_FOUND as DWORD);
            return (FALSE) as BOOL;
        }
    }

    i = 0 as core::ffi::c_int;
    field = (*file).fields.offset((*line).first_field as isize) as *mut field;
    while i < (*line).nb_fields {
        total = total.wrapping_add(
            PARSER_string_substW(
                file,
                (*field).text,
                core::ptr::null_mut::<WCHAR>(),
                0,
            )
            .wrapping_add(1) as DWORD,
        );
        i += 1;
        field = field.offset(1);
    }

    if !required.is_null() {
        *required = total;
    }
    if !buffer.is_null() {
        if total > size {
            SetLastError(ERROR_INSUFFICIENT_BUFFER as DWORD);
            return (FALSE) as BOOL;
        }
        i = 0 as core::ffi::c_int;
        field = (*file).fields.offset((*line).first_field as isize) as *mut field;
        while i < (*line).nb_fields {
            let mut len: core::ffi::c_uint = PARSER_string_substW(
                file,
                (*field).text,
                buffer as *mut WCHAR,
                size as core::ffi::c_uint,
            );
            if (i + 1 as core::ffi::c_int) < (*line).nb_fields {
                *buffer.offset(len as isize) = ',' as WCHAR;
            }
            buffer = buffer.offset(len.wrapping_add(1 as core::ffi::c_uint) as isize);
            i += 1;
            field = field.offset(1);
        }
    }
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetupGetLineTextA(
    context: PINFCONTEXT,
    hinf: HINF,
    section_name: PCSTR,
    key_name: PCSTR,
    mut buffer: PSTR,
    size: DWORD,
    required: PDWORD,
) -> BOOL {
    let mut file: *mut inf_file = core::ptr::null_mut();
    let mut line: *mut line = core::ptr::null_mut();
    let mut field: *mut field = core::ptr::null_mut();
    let mut i: core::ffi::c_int = 0;
    let mut total: DWORD = 0;

    if context.is_null() {
        let mut new_context: INFCONTEXT = core::mem::zeroed();
        if SetupFindFirstLineA(hinf, section_name, key_name, core::ptr::addr_of_mut!(new_context)) == 0 {
            return (FALSE) as BOOL;
        }
        file = new_context.CurrentInf as *mut inf_file;
        line = get_line(file, new_context.Section, new_context.Line);
    } else {
        file = (*context).CurrentInf as *mut inf_file;
        line = get_line(file, (*context).Section, (*context).Line);
        if line.is_null() {
            SetLastError(ERROR_LINE_NOT_FOUND);
            return (FALSE) as BOOL;
        }
    }

    i = 0;
    field = (*file).fields.offset((*line).first_field as isize);
    while i < (*line).nb_fields {
        total = total.wrapping_add(
            PARSER_string_substA(
                file,
                (*field).text,
                core::ptr::null_mut(),
                0,
            )
            .wrapping_add(1) as DWORD,
        );
        i += 1;
        field = field.offset(1);
    }

    if !required.is_null() {
        *required = total;
    }
    if !buffer.is_null() {
        if total > size {
            SetLastError(ERROR_INSUFFICIENT_BUFFER);
            return (FALSE) as BOOL;
        }
        i = 0;
        field = (*file).fields.offset((*line).first_field as isize);
        while i < (*line).nb_fields {
            let len: core::ffi::c_uint = PARSER_string_substA(
                file,
                (*field).text,
                buffer as *mut core::ffi::c_char,
                size as core::ffi::c_uint,
            );
            if (i + 1) < (*line).nb_fields {
                *buffer.offset(len as isize) = ',' as i32 as CHAR;
            }
            buffer = buffer.offset(len.wrapping_add(1) as isize);
            i += 1;
            field = field.offset(1);
        }
    }
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetupGetFieldCount(context: PINFCONTEXT) -> DWORD {
    let file: *mut inf_file = (*context).CurrentInf as *mut inf_file;
    let line: *mut line = get_line(
        file,
        (*context).Section as ::core::ffi::c_uint,
        (*context).Line as ::core::ffi::c_uint,
    );

    if line.is_null() {
        return 0;
    }
    return (*line).nb_fields as DWORD;
}

#[no_mangle]
pub unsafe extern "system" fn SetupGetStringFieldA(
    mut context: PINFCONTEXT,
    mut index: DWORD,
    mut buffer: PSTR,
    mut size: DWORD,
    mut required: PDWORD,
) -> BOOL {
    let mut file: *mut inf_file = (*context).CurrentInf as *mut inf_file;
    let mut field: *mut field = get_field(
        file,
        (*context).Section as core::ffi::c_int,
        (*context).Line as core::ffi::c_int,
        index as core::ffi::c_int,
    );
    let mut len: core::ffi::c_uint;

    SetLastError(0);
    if field.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return (FALSE) as BOOL;
    }
    len = PARSER_string_substA(
        file,
        (*field).text,
        core::ptr::null_mut::<core::ffi::c_char>(),
        0 as core::ffi::c_uint,
    );
    if !required.is_null() {
        *required = len.wrapping_add(1 as core::ffi::c_uint) as DWORD;
    }
    if !buffer.is_null() {
        if size <= len as DWORD {
            SetLastError(ERROR_INSUFFICIENT_BUFFER);
            return (FALSE) as BOOL;
        }
        PARSER_string_substA(
            file,
            (*field).text,
            buffer as *mut core::ffi::c_char,
            size as core::ffi::c_uint,
        );
        if ((*__wine_dbch___default).flags as core::ffi::c_int
            & ((1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int))
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/win32/setupapi/parser.c\0"
                    as *const u8 as *const core::ffi::c_char,
                b"SetupGetStringFieldA\0" as *const u8 as *const core::ffi::c_char,
                1900 as core::ffi::c_int,
                b"context %p/%p/%d/%d index %d returning %s\n\0" as *const u8
                    as *const core::ffi::c_char,
                (*context).Inf,
                (*context).CurrentInf,
                (*context).Section,
                (*context).Line,
                index,
                debugstr_a(buffer as *const core::ffi::c_char),
            );
        }
    }
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetupGetStringFieldW(
    context: PINFCONTEXT,
    index: DWORD,
    buffer: PWSTR,
    size: DWORD,
    required: PDWORD,
) -> BOOL {
    let file: *mut inf_file = (*context).CurrentInf as *mut inf_file;
    let field: *mut field = get_field(
        file,
        (*context).Section as ::core::ffi::c_int,
        (*context).Line as ::core::ffi::c_int,
        index as ::core::ffi::c_int,
    );
    let mut len: ::core::ffi::c_uint;

    SetLastError(0);
    if field.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return (FALSE) as BOOL;
    }
    len = PARSER_string_substW(
        file,
        (*field).text,
        ::core::ptr::null_mut::<WCHAR>(),
        0 as ::core::ffi::c_uint,
    );
    if !required.is_null() {
        *required = len.wrapping_add(1 as ::core::ffi::c_uint) as DWORD;
    }
    if !buffer.is_null() {
        if size <= len as DWORD {
            SetLastError(ERROR_INSUFFICIENT_BUFFER);
            return (FALSE) as BOOL;
        }
        PARSER_string_substW(
            file,
            (*field).text,
            buffer as *mut WCHAR,
            size as ::core::ffi::c_uint,
        );

        if ((*__wine_dbch___default).flags as ::core::ffi::c_int
            & ((1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int))
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/win32/setupapi/parser.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"SetupGetStringFieldW\0" as *const u8 as *const ::core::ffi::c_char,
                1931 as ::core::ffi::c_int,
                b"context %p/%p/%d/%d index %d returning %s\n\0" as *const u8
                    as *const ::core::ffi::c_char,
                (*context).Inf,
                (*context).CurrentInf,
                (*context).Section,
                (*context).Line,
                index,
                debugstr_w(buffer as *const WCHAR),
            );
        }
    }
    (TRUE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn SetupGetIntField(
    context: PINFCONTEXT,
    index: DWORD,
    result: PINT,
) -> BOOL {
    let mut localbuff: [::core::ffi::c_char; 20] = [0; 20];
    let mut end: *mut ::core::ffi::c_char = core::ptr::null_mut();
    let mut buffer: *mut ::core::ffi::c_char = &raw mut localbuff as *mut ::core::ffi::c_char;
    let mut required: DWORD = 0;
    let mut res: INT = 0;
    let mut ret: BOOL;

    'done: {
        ret = SetupGetStringFieldA(
            context,
            index,
            &raw mut localbuff as PSTR,
            core::mem::size_of::<[::core::ffi::c_char; 20]>() as DWORD,
            &raw mut required,
        );
        if ret == 0 {
            if GetLastError() != ERROR_INSUFFICIENT_BUFFER as DWORD {
                return FALSE as BOOL;
            }
            buffer = HeapAlloc(
                GetProcessHeap(),
                0 as DWORD,
                required as SIZE_T,
            ) as *mut ::core::ffi::c_char;
            if buffer.is_null() {
                return FALSE as BOOL;
            }
            ret = SetupGetStringFieldA(
                context,
                index,
                buffer as PSTR,
                required,
                core::ptr::null_mut::<DWORD>(),
            );
            if ret == 0 {
                break 'done;
            }
        }
        /* The call to SetupGetStringFieldA succeeded. If buffer is empty we have an optional field */
        if *buffer == 0 {
            *result = 0 as INT;
        } else {
            res = strtol(
                buffer,
                &raw mut end,
                0 as ::core::ffi::c_int,
            ) as INT;
            if end != buffer && *end == 0 {
                *result = res;
            } else {
                SetLastError(ERROR_INVALID_DATA as DWORD);
                ret = FALSE as BOOL;
            }
        }
    }

    if buffer != &raw mut localbuff as *mut ::core::ffi::c_char {
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            buffer as PVOID,
        );
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupGetBinaryField(
    mut context: PINFCONTEXT,
    mut index: DWORD,
    mut buffer: *mut BYTE,
    mut size: DWORD,
    mut required: LPDWORD,
) -> BOOL {
    let mut file: *mut inf_file = (*context).CurrentInf as *mut inf_file;
    let mut line: *mut line = get_line(
        file,
        (*context).Section as ::core::ffi::c_uint,
        (*context).Line as ::core::ffi::c_uint,
    );
    let mut field: *mut field = ::core::ptr::null_mut::<field>();
    let mut i: ::core::ffi::c_int = 0;

    if line.is_null() {
        SetLastError(ERROR_LINE_NOT_FOUND as DWORD);
        return (FALSE) as BOOL;
    }
    if index == 0 || index > (*line).nb_fields as DWORD {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return (FALSE) as BOOL;
    }
    index = index.wrapping_sub(1); // fields start at 0
    if !required.is_null() {
        *required = ((*line).nb_fields as DWORD).wrapping_sub(index);
    }
    if buffer.is_null() {
        return (TRUE) as BOOL;
    }
    if size < ((*line).nb_fields as DWORD).wrapping_sub(index) {
        SetLastError(ERROR_INSUFFICIENT_BUFFER as DWORD);
        return (FALSE) as BOOL;
    }
    field = (*file)
        .fields
        .offset(((*line).first_field as DWORD).wrapping_add(index) as isize);
    i = index as ::core::ffi::c_int;
    while i < (*line).nb_fields {
        *_errno() = 0;
        let mut value: ULONG = wcstoul(
            (*field).text,
            ::core::ptr::null_mut::<*mut WCHAR>(),
            16,
        ) as ULONG;
        if value > 255 || *_errno() != 0 {
            SetLastError(ERROR_INVALID_DATA as DWORD);
            return (FALSE) as BOOL;
        }
        *buffer.offset((i as DWORD).wrapping_sub(index) as isize) = value as BYTE;
        i += 1;
        field = field.offset(1);
    }
    if ((TRACE_ON!(__wine_dbch_setupapi)) != 0) {
        i = index as ::core::ffi::c_int;
        while i < (*line).nb_fields {
            TRACE!(
                " %02x\n",
                *buffer.offset((i as DWORD).wrapping_sub(index) as isize) as ::core::ffi::c_int
            );
            i += 1;
        }
    }
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetupGetMultiSzFieldA(
    mut context: PINFCONTEXT,
    mut index: DWORD,
    mut buffer: PSTR,
    mut size: DWORD,
    mut required: LPDWORD,
) -> BOOL {
    let mut file: *mut inf_file = (*context).CurrentInf as *mut inf_file;
    let mut line: *mut line = get_line(
        file,
        (*context).Section as ::core::ffi::c_uint,
        (*context).Line as ::core::ffi::c_uint,
    );
    let mut field: *mut field = ::core::ptr::null_mut::<field>();
    let mut len: ::core::ffi::c_uint = 0;
    let mut i: ::core::ffi::c_int = 0;
    let mut total: DWORD = 1;

    if line.is_null() {
        SetLastError(ERROR_LINE_NOT_FOUND as DWORD);
        return (FALSE) as BOOL;
    }
    if index == 0 || index > (*line).nb_fields as DWORD {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return (FALSE) as BOOL;
    }
    index = index.wrapping_sub(1); /* fields start at 0 */
    field = (*file)
        .fields
        .offset(((*line).first_field as DWORD).wrapping_add(index) as isize)
        as *mut field;
    i = index as ::core::ffi::c_int;
    while i < (*line).nb_fields {
        len = PARSER_string_substA(
            file,
            (*field).text,
            0 as *mut ::core::ffi::c_char,
            0 as ::core::ffi::c_uint,
        );
        if len == 0 {
            break;
        }
        total = total.wrapping_add(len.wrapping_add(1 as ::core::ffi::c_uint) as DWORD);
        i += 1;
        field = field.offset(1);
    }

    if !required.is_null() {
        *required = total;
    }
    if buffer.is_null() {
        return (TRUE) as BOOL;
    }
    if total > size {
        SetLastError(ERROR_INSUFFICIENT_BUFFER as DWORD);
        return (FALSE) as BOOL;
    }
    field = (*file)
        .fields
        .offset(((*line).first_field as DWORD).wrapping_add(index) as isize)
        as *mut field;
    i = index as ::core::ffi::c_int;
    while i < (*line).nb_fields {
        len = PARSER_string_substA(
            file,
            (*field).text,
            buffer as *mut ::core::ffi::c_char,
            size as ::core::ffi::c_uint,
        );
        if len == 0 {
            break;
        }
        buffer = buffer.offset(len.wrapping_add(1 as ::core::ffi::c_uint) as isize);
        i += 1;
        field = field.offset(1);
    }
    *buffer = 0 as CHAR; /* add final null */
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetupGetMultiSzFieldW(
    context: PINFCONTEXT,
    index: DWORD,
    buffer: PWSTR,
    size: DWORD,
    required: LPDWORD,
) -> BOOL {
    let mut index = index;
    let mut buffer = buffer;
    let file: *mut inf_file = (*context).CurrentInf as *mut inf_file;
    let line: *mut line = get_line(
        file,
        (*context).Section as u32,
        (*context).Line as u32,
    );
    let mut field: *mut field = core::ptr::null_mut();
    let mut len: u32;
    let mut i: i32;
    let mut total: DWORD = 1;

    if line.is_null() {
        SetLastError(ERROR_LINE_NOT_FOUND);
        return (FALSE) as BOOL;
    }
    if index == 0 || index > (*line).nb_fields as DWORD {
        SetLastError(ERROR_INVALID_PARAMETER);
        return (FALSE) as BOOL;
    }
    index = index.wrapping_sub(1); // fields start at 0
    field = (*file)
        .fields
        .offset(((*line).first_field as DWORD).wrapping_add(index) as isize);
    i = index as i32;
    while i < (*line).nb_fields {
        len = PARSER_string_substW(
            file,
            (*field).text,
            core::ptr::null_mut(),
            0,
        );
        if len == 0 {
            break;
        }
        total = total.wrapping_add(len.wrapping_add(1 as u32) as DWORD);
        i += 1;
        field = field.offset(1);
    }

    if !required.is_null() {
        *required = total;
    }
    if buffer.is_null() {
        return (TRUE) as BOOL;
    }
    if total > size {
        SetLastError(ERROR_INSUFFICIENT_BUFFER);
        return (FALSE) as BOOL;
    }
    field = (*file)
        .fields
        .offset(((*line).first_field as DWORD).wrapping_add(index) as isize);
    i = index as i32;
    while i < (*line).nb_fields {
        len = PARSER_string_substW(file, (*field).text, buffer, size);
        if len == 0 {
            break;
        }
        buffer = buffer.offset(len.wrapping_add(1 as u32) as isize);
        i += 1;
        field = field.offset(1);
    }
    *buffer = 0; // add final null
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn pSetupGetField(context: PINFCONTEXT, index: DWORD) -> LPCWSTR {
    let file: *mut inf_file = (*context).CurrentInf as *mut inf_file;
    let field: *mut field = get_field(
        file,
        (*context).Section as ::core::ffi::c_int,
        (*context).Line as ::core::ffi::c_int,
        index as ::core::ffi::c_int,
    );

    if field.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return ::core::ptr::null::<WCHAR>();
    }
    return (*field).text as LPCWSTR;
}

#[no_mangle]
pub unsafe extern "system" fn SetupGetInfFileListW(
    DirectoryPath: PCWSTR,
    InfStyle: DWORD,
    ReturnBuffer: PWSTR,
    ReturnBufferSize: DWORD,
    RequiredSize: PDWORD,
) -> BOOL {
    let mut hSearch: HANDLE = core::ptr::null_mut();
    let mut pFullFileName: LPWSTR = core::ptr::null_mut();
    let mut pFileName: LPWSTR = core::ptr::null_mut();
    let mut pBuffer: LPWSTR = ReturnBuffer;
    let mut wfdFileInfo: WIN32_FIND_DATAW = core::mem::zeroed();
    let mut len: usize;
    let mut requiredSize: DWORD = 0;
    let mut ret: BOOL = (FALSE as i32);

    'cleanup: {
        if InfStyle & !(INF_STYLE_OLDNT | INF_STYLE_WIN4) != 0 {
            SetLastError(ERROR_INVALID_PARAMETER);
            break 'cleanup;
        } else if ReturnBufferSize == 0 && !ReturnBuffer.is_null() {
            SetLastError(ERROR_INVALID_PARAMETER);
            break 'cleanup;
        } else if ReturnBufferSize > 0 && ReturnBuffer.is_null() {
            SetLastError(ERROR_INVALID_PARAMETER);
            break 'cleanup;
        }

        /* Allocate memory for file filter */
        if !DirectoryPath.is_null() {
            /* "DirectoryPath\" form */
            len = strlenW(DirectoryPath) + 1 + 1;
        } else {
            /* "%SYSTEMROOT%\Inf\" form */
            len = (MAX_PATH
                + 1
                + (strlenW(&raw const InfDirectory as *const WCHAR)) as u32
                + 1) as _;
        }
        len = len.wrapping_add((MAX_PATH) as _); /* To contain file name or "*.inf" string */
        pFullFileName = MyMalloc((len * core::mem::size_of::<WCHAR>()) as DWORD)
            as LPWSTR;
        if pFullFileName.is_null() {
            SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            break 'cleanup;
        }

        /* Fill file filter buffer */
        if !DirectoryPath.is_null() {
            strcpyW(pFullFileName, DirectoryPath);
            if *pFullFileName != 0
                && *pFullFileName.add(strlenW(pFullFileName) - 1)
                    != '\\' as WCHAR
            {
                strcatW(pFullFileName, &raw const BackSlash as *const WCHAR);
            }
        } else {
            len = GetSystemWindowsDirectoryW(pFullFileName, MAX_PATH) as usize;
            if len == 0 || len > (MAX_PATH) as _ {
                break 'cleanup;
            }
            if *pFullFileName.add(strlenW(pFullFileName) - 1)
                != '\\' as WCHAR
            {
                strcatW(pFullFileName, &raw const BackSlash as *const WCHAR);
            }
            strcatW(pFullFileName, &raw const InfDirectory as *const WCHAR);
        }
        pFileName = pFullFileName.add(strlenW(pFullFileName));

        /* Search for the first file */
        strcpyW(pFileName, &raw const InfFileSpecification as *const WCHAR);
        hSearch = FindFirstFileW(pFullFileName as LPCWSTR, &raw mut wfdFileInfo);
        if hSearch == INVALID_HANDLE_VALUE {
            /* No file returned by FindFirstFileW */
            break 'cleanup;
        }

        loop {
            'body: {
                let mut hInf: HINF = core::ptr::null_mut();

                strcpyW(pFileName, &raw mut wfdFileInfo.cFileName as *mut WCHAR);
                hInf = SetupOpenInfFileW(
                    pFullFileName as PCWSTR,
                    core::ptr::null::<WCHAR>(),
                    InfStyle,
                    core::ptr::null_mut::<UINT>(),
                );
                if hInf == INVALID_HANDLE_VALUE {
                    if GetLastError() == ERROR_CLASS_MISMATCH {
                        /* InfStyle was not correct. Skip this file */
                        break 'body;
                    }
                    break 'body;
                }

                len = strlenW(&raw mut wfdFileInfo.cFileName as *mut WCHAR) + 1;
                requiredSize = requiredSize.wrapping_add(len as DWORD);
                if requiredSize <= ReturnBufferSize {
                    strcpyW(pBuffer, &raw mut wfdFileInfo.cFileName as *mut WCHAR);
                    pBuffer = pBuffer.add(len);
                }
                SetupCloseInfFile(hInf);
            }
            if !(FindNextFileW(hSearch, &raw mut wfdFileInfo) != 0) {
                break;
            }
        }
        FindClose(hSearch);

        requiredSize = requiredSize.wrapping_add(1); /* Final NULL char */
        if requiredSize <= ReturnBufferSize {
            *pBuffer = '\0' as WCHAR;
            ret = (TRUE as i32);
        } else {
            SetLastError(ERROR_INSUFFICIENT_BUFFER);
            ret = (FALSE as i32);
        }
        if !RequiredSize.is_null() {
            *RequiredSize = requiredSize;
        }
    }

    MyFree(pFullFileName as PVOID);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupGetInfFileListA(
    DirectoryPath: PCSTR,
    InfStyle: DWORD,
    ReturnBuffer: PSTR,
    ReturnBufferSize: DWORD,
    RequiredSize: PDWORD,
) -> BOOL {
    let mut DirectoryPathW: PWSTR = core::ptr::null_mut();
    let mut ReturnBufferW: PWSTR = core::ptr::null_mut();
    let mut ret: BOOL = (FALSE as i32);

    'Cleanup: {
        if !DirectoryPath.is_null() {
            DirectoryPathW = pSetupMultiByteToUnicode(DirectoryPath, CP_ACP);
            if DirectoryPathW.is_null() {
                break 'Cleanup;
            }
        }

        if !ReturnBuffer.is_null() && ReturnBufferSize != 0 {
            ReturnBufferW = MyMalloc(
                (ReturnBufferSize as usize).wrapping_mul(core::mem::size_of::<WCHAR>()) as DWORD,
            ) as PWSTR;
            if ReturnBufferW.is_null() {
                break 'Cleanup;
            }
        }

        ret = SetupGetInfFileListW(
            DirectoryPathW as PCWSTR,
            InfStyle,
            ReturnBufferW,
            ReturnBufferSize,
            RequiredSize,
        );

        if ret != 0 && !ReturnBufferW.is_null() {
            ret = (WideCharToMultiByte(
                CP_ACP,
                0,
                ReturnBufferW as LPCWSTR,
                -(1),
                ReturnBuffer as LPSTR,
                ReturnBufferSize as core::ffi::c_int,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            ) != 0) as BOOL;
        }
    }

    // Cleanup:
    MyFree(DirectoryPathW as PVOID);
    MyFree(ReturnBufferW as PVOID);

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiGetINFClassW(
    InfName: PCWSTR,
    ClassGuid: LPGUID,
    ClassName: PWSTR,
    ClassNameSize: DWORD,
    RequiredSize: PDWORD,
) -> BOOL {
    let mut hInf: HINF = INVALID_HANDLE_VALUE;
    let mut ret: BOOL = (FALSE as i32);

    if InfName.is_null() || ClassGuid.is_null() || ClassName.is_null() || ClassNameSize == 0
    {
        SetLastError(ERROR_INVALID_PARAMETER);
        return (FALSE) as BOOL;
    }

    /* Open .inf file */
    hInf = SetupOpenInfFileW(InfName, core::ptr::null(), INF_STYLE_WIN4, core::ptr::null_mut());
    if hInf == INVALID_HANDLE_VALUE {
        // goto cleanup;
        ret = ret;
    } else {
        ret = PARSER_GetInfClassW(hInf, ClassGuid, ClassName, ClassNameSize, RequiredSize);
    }

    // cleanup:
    if hInf != INVALID_HANDLE_VALUE {
        SetupCloseInfFile(hInf);
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiGetINFClassA(
    InfName: PCSTR,
    ClassGuid: LPGUID,
    ClassName: PSTR,
    ClassNameSize: DWORD,
    RequiredSize: PDWORD,
) -> BOOL {
    let mut InfNameW: PWSTR = core::ptr::null_mut();
    let mut ClassNameW: PWSTR = core::ptr::null_mut();
    let mut ret: BOOL = (FALSE as i32);

    'Cleanup: {
        if !InfName.is_null() {
            InfNameW = pSetupMultiByteToUnicode(InfName, CP_ACP) as PWSTR;
            if InfNameW.is_null() {
                break 'Cleanup;
            }
        }

        if !ClassName.is_null() && ClassNameSize != 0 {
            ClassNameW = MyMalloc(
                (ClassNameSize as usize).wrapping_mul(core::mem::size_of::<WCHAR>()) as DWORD,
            ) as PWSTR;
            if ClassNameW.is_null() {
                break 'Cleanup;
            }
        }

        ret = SetupDiGetINFClassW(
            InfNameW as PCWSTR,
            ClassGuid,
            ClassNameW,
            ClassNameSize,
            RequiredSize,
        );

        if ret != 0 && !ClassNameW.is_null() {
            ret = (WideCharToMultiByte(
                CP_ACP,
                0,
                ClassNameW as LPCWSTR,
                -1,
                ClassName as LPSTR,
                ClassNameSize as i32,
                core::ptr::null_mut(),
                core::ptr::null_mut::<BOOL>(),
            ) != 0) as i32 as BOOL;
        }
    }

    MyFree(InfNameW as PVOID);
    MyFree(ClassNameW as PVOID);

    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn EnumerateSectionsStartingWith(
    hInf: HINF,
    pStr: LPCWSTR,
    Callback: FIND_CALLBACK,
    Context: PVOID,
) -> BOOL {
    let file: *mut inf_file = hInf as *mut inf_file;
    let len: size_t = strlenW(pStr) as size_t;
    let mut i: ::core::ffi::c_uint;

    i = 0 as ::core::ffi::c_uint;
    while i < (*file).nb_sections {
        if strncmpiW(pStr, (**(*file).sections.offset(i as isize)).name, len) == 0 {
            if (Callback.unwrap_unchecked())((**(*file).sections.offset(i as isize)).name as LPCWSTR, Context) == 0 {
                return (FALSE) as BOOL;
            }
        }
        i = i.wrapping_add(1);
    }
    return (TRUE) as BOOL;
}
