use crate::prelude::*;
use core::ffi::c_void;

crate::wine_default_debug_channel!(__wine_dbch_edit);

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const WC_EDITW: [::core::ffi::c_ushort; 5] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_ushort; 5]>(*b"E\0d\0i\0t\0\0\0") };

// WINE_DEFAULT_DEBUG_CHANNEL(edit): debug-channel macro, no Rust declaration emitted.

pub const BUFLIMIT_INITIAL: i32 = 30000; // initial buffer size

pub const GROWLENGTH: i32 = 32; // buffers granularity in bytes: must be power of 2

pub const HSCROLL_FRACTION: i32 = 3; // scroll window by 1/3 width

pub const EF_MODIFIED: i32 = 0x0001; // text has been modified

pub const EF_FOCUSED: i32 = 0x0002; // we have input focus

pub const EF_UPDATE: i32 = 0x0004; // notify parent of changed state

pub const EF_VSCROLL_TRACK: i32 = 0x0008; // don't SetScrollPos() since we are tracking the thumb

pub const EF_HSCROLL_TRACK: i32 = 0x0010; // don't SetScrollPos() since we are tracking the thumb

pub const EF_AFTER_WRAP: i32 = 0x0080; // the caret is displayed after the last character of a wrapped line, instead of in front of the next character

pub const EF_USE_SOFTBRK: i32 = 0x0100; // Enable soft breaks in text.

pub const EF_DIALOGMODE: i32 = 0x0200; // Indicates that we are inside a dialog window

pub const ID_CB_LISTBOX: i32 = 1000;

pub type LINE_END = i32;
pub const END_0: LINE_END = 0; // line ends with terminating '\0' character
pub const END_WRAP: LINE_END = 1; // line is wrapped
pub const END_HARD: LINE_END = 2; // line ends with a hard return '\r\n'
pub const END_SOFT: LINE_END = 3; // line ends with a soft return '\r\r\n'
pub const END_RICH: LINE_END = 4; // line ends with a single '\n'

#[repr(C)]
pub struct tagLINEDEF {
    pub length: i32, // bruto length of a line in bytes
    pub net_length: i32, // netto length of a line in visible characters
    pub ending: LINE_END,
    pub width: i32, // width of the line in pixels
    pub index: i32, // line index into the buffer
    pub ssa: SCRIPT_STRING_ANALYSIS, // Uniscribe Data
    pub next: *mut tagLINEDEF,
}
pub type LINEDEF = tagLINEDEF;

#[repr(C)]
pub struct EDITSTATE {
    pub text: LPWSTR, // the actual contents of the control
    pub text_length: u32, // cached length of text buffer (in WCHARs)
    pub buffer_size: u32, // the size of the buffer in characters
    pub buffer_limit: u32, // the maximum size to which the buffer may grow in characters
    pub font: HFONT, // NULL means standard system font
    pub x_offset: i32, // scroll offset: pixels for multi line, characters for single line
    pub dwCaretWidth: DWORD,
    pub line_height: i32, // height of a screen line in pixels
    pub char_width: i32, // average character width in pixels
    pub style: DWORD, // sane version of wnd->dwStyle
    pub flags: WORD, // flags that are not in es->style or wnd->flags (EF_XXX)
    pub undo_insert_count: i32, // number of characters inserted in sequence
    pub undo_position: u32, // character index of the insertion and deletion
    pub undo_text: LPWSTR, // deleted text
    pub undo_buffer_size: u32, // size of the deleted text buffer
    pub selection_start: i32, // == selection_end if no selection
    pub selection_end: i32, // == current caret position
    pub password_char: WCHAR, // == 0 if no password char, and for multi line controls
    pub left_margin: i32, // in pixels
    pub right_margin: i32, // in pixels
    pub format_rect: RECT,
    pub text_width: i32, // width of the widest line in pixels (multi line) or line width (single line)
    pub region_posx: i32, // Position of cursor relative to region: -1 left, 0 within, 1 right
    pub region_posy: i32, // -1: to left, 0: within, 1: to right
    pub word_break_proc: EDITWORDBREAKPROCW,
    pub line_count: i32, // number of lines
    pub y_offset: i32, // scroll offset in number of lines
    pub bCaptureState: BOOL, // flag indicating whether mouse was captured
    pub bEnableState: BOOL, // flag keeping the enable state
    pub hwndSelf: HWND, // the our window handle
    pub hwndParent: HWND, // Handle of parent for sending EN_* messages
    pub hwndListBox: HWND, // handle of ComboBox's listbox or NULL
    pub wheelDeltaRemainder: i32, // scroll wheel delta left over after scrolling whole lines
    pub cue_banner_text: *mut WCHAR,
    pub cue_banner_draw_focused: BOOL,

    // only for multi line controls
    pub lock_count: i32, // amount of re-entries in the EditWndProc
    pub tabs_count: i32,
    pub tabs: LPINT,
    pub first_line_def: *mut LINEDEF, // linked list of (soft) linebreaks
    pub hloc32W: HLOCAL, // our unicode local memory block
    pub hlocapp: HLOCAL, // The text buffer handle belongs to the app

    // Uniscribe Data
    pub logAttr: *mut SCRIPT_LOGATTR,
    pub ssa: SCRIPT_STRING_ANALYSIS, // Uniscribe Data for single line controls
}

// #define ROUND_TO_GROW(size), SWAP_UINT32(x,y), ORDER_UINT(x,y): function-like macros, skipped.
// static LRESULT EDIT_EM_PosFromChar(...): prototype, skipped (function emitted separately).
// extern BOOL WINAPI GetCharWidthInfo(...): extern declaration, skipped (caller imports).

#[repr(C)]
pub struct char_width_info {
    pub min_lsb: i32,
    pub min_rsb: i32,
    pub unknown: i32,
}

#[allow(non_snake_case)]

static mut __RUSTCONV_NOTUSED__: () = ();

unsafe extern "C" fn notify_parent(es: *const EDITSTATE, code: INT) -> BOOL {
    let hwnd: HWND = (*es).hwndSelf;
    SendMessageW((*es).hwndParent, WM_COMMAND as UINT, MAKEWPARAM!(GetWindowLongPtrW((*es).hwndSelf, GWLP_ID) as WORD, code as WORD) as WPARAM, (*es).hwndSelf as LPARAM);
    return IsWindow(hwnd);
}

unsafe extern "C" fn EDIT_EM_CanUndo(es: *const EDITSTATE) -> BOOL {
    return ((*es).undo_insert_count != 0 || lstrlenW((*es).undo_text as LPCWSTR) != 0) as BOOL;
}

unsafe extern "C" fn EDIT_EM_EmptyUndoBuffer(mut es: *mut EDITSTATE) {
    (*es).undo_insert_count = 0 as INT;
    *(*es).undo_text = '\0' as WCHAR;
}

unsafe extern "C" fn EDIT_NotifyCtlColor(es: *mut EDITSTATE, hdc: HDC) -> HBRUSH {
    let mut hbrush: HBRUSH = ::core::ptr::null_mut::<HBRUSH__>();
    let mut msg: UINT = 0;
    if (*es).bEnableState == 0 || ((*es).style & ES_READONLY) != 0 {
        msg = WM_CTLCOLORSTATIC;
    } else {
        msg = WM_CTLCOLOREDIT;
    }
    hbrush = SendMessageW(
        GetParent((*es).hwndSelf),
        msg,
        hdc as WPARAM,
        (*es).hwndSelf as LPARAM,
    ) as HBRUSH;
    if hbrush.is_null() {
        hbrush = DefWindowProcW(
            GetParent((*es).hwndSelf),
            msg,
            hdc as WPARAM,
            (*es).hwndSelf as LPARAM,
        ) as HBRUSH;
    }
    return hbrush;
}

unsafe extern "C" fn get_text_length(mut es: *mut EDITSTATE) -> UINT {
    if (*es).text_length == (-(1 as i32)) as UINT {
        (*es).text_length = lstrlenW((*es).text as LPCWSTR) as UINT;
    }
    return (*es).text_length;
}

unsafe extern "C" fn EDIT_WordBreakProc(
    mut es: *mut EDITSTATE,
    mut s: LPWSTR,
    mut index: INT,
    mut count: INT,
    mut action: INT,
) -> INT {
    let mut ret: INT = 0 as INT;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/edit.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"EDIT_WordBreakProc\0" as *const u8 as *const ::core::ffi::c_char,
            241 as ::core::ffi::c_int,
            b"s=%p, index=%d, count=%d, action=%d\n\0" as *const u8 as *const ::core::ffi::c_char,
            s,
            index,
            count,
            action,
        );
    }
    if s.is_null() {
        return 0 as INT;
    }
    if (*es).logAttr.is_null() {
        let mut psa: SCRIPT_ANALYSIS = core::mem::zeroed();
        memset(
            core::ptr::addr_of_mut!(psa) as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            core::mem::size_of::<SCRIPT_ANALYSIS>() as size_t,
        );
        psa.set_eScript(SCRIPT_UNDEFINED as WORD as WORD);
        (*es).logAttr = heap_alloc(
            (::core::mem::size_of::<SCRIPT_LOGATTR>() as SIZE_T)
                .wrapping_mul(get_text_length(es) as SIZE_T),
        ) as *mut SCRIPT_LOGATTR;
        ScriptBreak(
            (*es).text as *const WCHAR,
            get_text_length(es) as ::core::ffi::c_int,
            core::ptr::addr_of_mut!(psa),
            (*es).logAttr,
        );
    }
    match (action) as u32 {
        WB_LEFT => {
            if index != 0 {
                index -= 1;
            }
            while index != 0 && (*(*es).logAttr.offset(index as isize)).fSoftBreak() == 0 {
                index -= 1;
            }
            ret = index;
        }
        WB_RIGHT => {
            if !(count == 0) {
                while index < count
                    && *s.offset(index as isize) as ::core::ffi::c_int != 0
                    && (*(*es).logAttr.offset(index as isize)).fSoftBreak() == 0
                {
                    index += 1;
                }
                ret = index;
            }
        }
        WB_ISDELIMITER => {
            ret = (*(*es).logAttr.offset(index as isize)).fWhiteSpace() as INT;
        }
        _ => {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                != 0
            {
                let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_0: __wine_debug_class = __WINE_DBCL_ERR;
                ros_dbg_log(
                    __dbcl_0,
                    __dbch_0,
                    b"dll/win32/comctl32/edit.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"EDIT_WordBreakProc\0" as *const u8 as *const ::core::ffi::c_char,
                    275 as ::core::ffi::c_int,
                    b"unknown action code, please report !\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                );
            }
        }
    }
    return ret;
}

unsafe extern "C" fn EDIT_CallWordBreakProc(es: *mut EDITSTATE, start: INT, index: INT, count: INT, action: INT) -> INT {
    let mut ret: INT;

    if (*es).word_break_proc.is_some() {
        ret = (*es).word_break_proc.unwrap_unchecked()(
            (*es).text.offset(start as isize),
            index,
            count,
            action,
        ) as INT;
    } else {
        ret = EDIT_WordBreakProc(es, (*es).text, index + start, count + start, action) - start;
    }

    return ret;
}

unsafe extern "C" fn EDIT_InvalidateUniscribeData_linedef(line_def: *mut LINEDEF) {
    if !(*line_def).ssa.is_null() {
        ScriptStringFree(core::ptr::addr_of_mut!((*line_def).ssa));
        (*line_def).ssa = core::ptr::null_mut();
    }
}

unsafe extern "C" fn EDIT_InvalidateUniscribeData(mut es: *mut EDITSTATE) {
    let mut line_def: *mut LINEDEF = (*es).first_line_def;
    while !line_def.is_null() {
        EDIT_InvalidateUniscribeData_linedef(line_def);
        line_def = (*line_def).next as *mut LINEDEF;
    }
    if !(*es).ssa.is_null() {
        ScriptStringFree(core::ptr::addr_of_mut!((*es).ssa));
        (*es).ssa = core::ptr::null_mut() as SCRIPT_STRING_ANALYSIS;
    }
}

unsafe extern "C" fn EDIT_UpdateUniscribeData_linedef(es: *mut EDITSTATE, dc: HDC, line_def: *mut LINEDEF) -> SCRIPT_STRING_ANALYSIS {
    if line_def.is_null() {
        return core::ptr::null_mut();
    }

    if (*line_def).net_length != 0 && (*line_def).ssa.is_null() {
        let mut index: i32 = (*line_def).index;
        let mut old_font: HFONT = core::ptr::null_mut();
        let mut udc: HDC = dc;
        let mut tabdef: SCRIPT_TABDEF = core::mem::zeroed();
        let mut hr: HRESULT = 0;

        if udc.is_null() {
            udc = GetDC((*es).hwndSelf);
        }
        if !(*es).font.is_null() {
            old_font = (SelectObject(udc, ((*es).font).cast())).cast();
        }

        tabdef.cTabStops = (*es).tabs_count;
        tabdef.iScale = GdiGetCharDimensions(udc, core::ptr::null_mut(), core::ptr::null_mut());
        tabdef.pTabStops = (*es).tabs;
        tabdef.iTabOrigin = 0;

        hr = ScriptStringAnalyse(
            udc,
            (*es).text.offset(index as isize) as *const core::ffi::c_void,
            (*line_def).net_length,
            (1.5 * (*line_def).net_length as f64 + 16 as f64) as i32,
            -(1),
            (SSA_LINK | SSA_FALLBACK | SSA_GLYPHS | SSA_TAB) as DWORD,
            -(1),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null(),
            core::ptr::addr_of_mut!(tabdef),
            core::ptr::null(),
            core::ptr::addr_of_mut!((*line_def).ssa),
        );
        if hr < 0 {
            (*line_def).ssa = core::ptr::null_mut();
        }

        if !(*es).font.is_null() {
            SelectObject(udc, (old_font).cast());
        }
        if udc != dc {
            ReleaseDC((*es).hwndSelf, udc);
        }
    }

    (*line_def).ssa
}

unsafe extern "C" fn EDIT_UpdateUniscribeData(es: *mut EDITSTATE, dc: HDC, mut line: INT) -> SCRIPT_STRING_ANALYSIS {
    let mut line_def: *mut LINEDEF = core::ptr::null_mut();
    if ((*es).style & (ES_MULTILINE as DWORD)) == 0 {
        if (*es).ssa.is_null() {
            let mut length: INT = get_text_length(es) as INT;
            let mut old_font: HFONT = core::ptr::null_mut();
            let mut udc: HDC = dc;
            if udc.is_null() {
                udc = GetDC((*es).hwndSelf);
            }
            if !(*es).font.is_null() {
                old_font = SelectObject(udc, (*es).font as HGDIOBJ) as HFONT;
            }
            if ((*es).style & (ES_PASSWORD as DWORD)) != 0 {
                ScriptStringAnalyse(
                    udc,
                    core::ptr::addr_of_mut!((*es).password_char) as *const core::ffi::c_void,
                    length as core::ffi::c_int,
                    ((1.5 as f64) * (length as f64)
                        + (16 as f64)) as core::ffi::c_int,
                    -(1 as core::ffi::c_int),
                    (SSA_LINK | SSA_FALLBACK | SSA_GLYPHS | SSA_PASSWORD) as DWORD,
                    -(1 as core::ffi::c_int),
                    (core::ptr::null_mut::<SCRIPT_CONTROL>()).cast(),
                    (core::ptr::null_mut::<SCRIPT_STATE>()).cast(),
                    core::ptr::null::<core::ffi::c_int>(),
                    core::ptr::null_mut::<SCRIPT_TABDEF>(),
                    core::ptr::null::<BYTE>(),
                    core::ptr::addr_of_mut!((*es).ssa),
                );
            } else {
                ScriptStringAnalyse(
                    udc,
                    (*es).text as *const core::ffi::c_void,
                    length as core::ffi::c_int,
                    ((1.5 as f64) * (length as f64)
                        + (16 as f64)) as core::ffi::c_int,
                    -(1 as core::ffi::c_int),
                    (SSA_LINK | SSA_FALLBACK | SSA_GLYPHS) as DWORD,
                    -(1 as core::ffi::c_int),
                    (core::ptr::null_mut::<SCRIPT_CONTROL>()).cast(),
                    (core::ptr::null_mut::<SCRIPT_STATE>()).cast(),
                    core::ptr::null::<core::ffi::c_int>(),
                    core::ptr::null_mut::<SCRIPT_TABDEF>(),
                    core::ptr::null::<BYTE>(),
                    core::ptr::addr_of_mut!((*es).ssa),
                );
            }
            if !(*es).font.is_null() {
                SelectObject(udc, old_font as HGDIOBJ);
            }
            if udc != dc {
                ReleaseDC((*es).hwndSelf, udc);
            }
        }
        return (*es).ssa;
    } else {
        line_def = (*es).first_line_def;
        while !line_def.is_null() && line != 0 {
            line_def = (*line_def).next as *mut LINEDEF;
            line -= 1;
        }
        return EDIT_UpdateUniscribeData_linedef(es, dc, line_def);
    }
}

unsafe extern "C" fn get_vertical_line_count(es: *mut EDITSTATE) -> INT {
    let vlc: INT = ((*es).format_rect.bottom - (*es).format_rect.top) / (*es).line_height;
    return max(1, vlc);
}

unsafe extern "C" fn EDIT_BuildLineDefs_ML(
    mut es: *mut EDITSTATE,
    mut istart: INT,
    mut iend: INT,
    mut delta: INT,
    mut hrgn: HRGN,
) {
    let mut current_position: LPWSTR = core::ptr::null_mut::<WCHAR>();
    let mut cp: LPWSTR = core::ptr::null_mut::<WCHAR>();
    let mut fw: INT = 0;
    let mut current_line: *mut LINEDEF = core::ptr::null_mut::<LINEDEF>();
    let mut previous_line: *mut LINEDEF = core::ptr::null_mut::<LINEDEF>();
    let mut start_line: *mut LINEDEF = core::ptr::null_mut::<LINEDEF>();
    let mut line_index: INT = 0;
    let mut nstart_line: INT = 0;
    let mut nstart_index: INT = 0;
    let mut line_count: INT = (*es).line_count;
    let mut orig_net_length: INT = 0;
    let mut rc: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut vlc: INT = 0;

    if istart == iend && delta == 0 {
        return;
    }

    previous_line = core::ptr::null_mut::<LINEDEF>();
    current_line = (*es).first_line_def as *mut LINEDEF;

    loop {
        if istart < (*current_line).index + (*current_line).length
            || (*current_line).ending as c_uint == END_0 as c_uint
        {
            break;
        }
        previous_line = current_line;
        current_line = (*current_line).next as *mut LINEDEF;
        line_index += 1;
        if current_line.is_null() {
            break;
        }
    }

    if current_line.is_null() {
        if (*__wine_dbch___default).flags as c_int & (1 as c_int) << __WINE_DBCL_FIXME as c_int != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/win32/comctl32/edit.c\0"
                    as *const u8 as *const c_char,
                b"EDIT_BuildLineDefs_ML\0" as *const u8 as *const c_char,
                463 as c_int,
                b" modification occurred outside buffer\n\0" as *const u8 as *const c_char,
            );
        }
        return;
    }

    nstart_line = line_index;
    nstart_index = (*current_line).index;

    if ((*es).style & (ES_AUTOHSCROLL as DWORD)) == 0
        && line_index > 0
    {
        line_index -= 1;
        current_line = previous_line;
    }
    start_line = current_line;

    fw = ((*es).format_rect.right - (*es).format_rect.left) as INT;
    current_position = (*es).text.offset((*current_line).index as isize);
    vlc = get_vertical_line_count(es);

    let mut current_block_143: u64;
    loop {
        if current_line != start_line {
            if current_line.is_null()
                || ((*current_line).index + delta) as c_longlong
                    > current_position.offset_from((*es).text) as c_longlong
            {
                let mut new_line: *mut LINEDEF =
                    heap_alloc_zero(core::mem::size_of::<LINEDEF>() as SIZE_T)
                        as *mut LINEDEF;
                (*new_line).next = (*previous_line).next;
                (*previous_line).next = new_line as *mut tagLINEDEF;
                current_line = new_line;
                (*es).line_count += 1;
                current_block_143 = 6417057564578538666;
            } else if (((*current_line).index + delta) as c_longlong)
                < current_position.offset_from((*es).text) as c_longlong
            {
                (*previous_line).next = (*current_line).next;
                heap_free(current_line as *mut c_void);
                current_line = (*previous_line).next as *mut LINEDEF;
                (*es).line_count -= 1;
                current_block_143 = 9828876828309294594;
            } else {
                if current_position.offset_from((*es).text) as c_longlong > iend as c_longlong {
                    break;
                }
                current_block_143 = 6417057564578538666;
            }
        } else {
            current_block_143 = 6417057564578538666;
        }

        match current_block_143 {
            6417057564578538666 => {
                (*current_line).index =
                    current_position.offset_from((*es).text) as c_longlong as INT;
                orig_net_length = (*current_line).net_length;

                cp = current_position;
                while *cp != 0 {
                    if *cp as c_int == '\n' as i32 {
                        break;
                    }
                    if *cp as c_int == '\r' as i32
                        && *cp.offset(1 as isize) as c_int == '\n' as i32
                    {
                        break;
                    }
                    cp = cp.offset(1);
                }

                if *cp == 0 {
                    (*current_line).ending = END_0;
                    (*current_line).net_length =
                        lstrlenW(current_position as LPCWSTR) as INT;
                } else if cp > current_position
                    && *cp.offset(-(1 as isize)) as c_int == '\r' as i32
                {
                    (*current_line).ending = END_SOFT;
                    (*current_line).net_length = (cp.offset_from(current_position) as c_longlong
                        - 1 as c_longlong)
                        as INT;
                } else if *cp as c_int == '\n' as i32 {
                    (*current_line).ending = END_RICH;
                    (*current_line).net_length =
                        cp.offset_from(current_position) as c_longlong as INT;
                } else {
                    (*current_line).ending = END_HARD;
                    (*current_line).net_length =
                        cp.offset_from(current_position) as c_longlong as INT;
                }

                if (*current_line).net_length != 0 {
                    let mut sz: *const SIZE = core::ptr::null::<SIZE>();
                    EDIT_InvalidateUniscribeData_linedef(current_line);
                    EDIT_UpdateUniscribeData_linedef(es, core::ptr::null_mut::<HDC__>(), current_line);
                    if !(*current_line).ssa.is_null() {
                        sz = ScriptString_pSize((*current_line).ssa);
                        (*current_line).width = (*sz).cx as INT;
                    } else {
                        (*current_line).width = (*es).char_width * (*current_line).net_length;
                    }
                } else {
                    (*current_line).width = 0;
                }

                if ((*es).style & (ES_AUTOHSCROLL as DWORD)) == 0 {
                    if (*current_line).width > fw && fw > (*es).char_width {
                        let mut prev: INT = 0;
                        let mut next: INT = 0;
                        let mut w: c_int = 0;
                        let mut sz_0: *const SIZE = core::ptr::null::<SIZE>();
                        let mut d: f32 = 0.;

                        prev = ((*current_line).net_length as c_int
                            - 1 as c_int) as INT;
                        w = (*current_line).net_length as c_int;
                        d = (*current_line).width as f32 / fw as f32;
                        if d > (1 as f32 + 0.2f32) {
                            d -= 0 as f32 + 0.2f32;
                        }
                        next = (prev as f32 / d) as INT;
                        if next >= prev {
                            next = (prev as c_int - 1 as c_int) as INT;
                        }

                        loop {
                            prev = EDIT_CallWordBreakProc(
                                es,
                                current_position.offset_from((*es).text) as c_longlong as INT,
                                next,
                                (*current_line).net_length,
                                (WB_LEFT) as INT,
                            );
                            (*current_line).net_length = prev;
                            EDIT_InvalidateUniscribeData_linedef(current_line);
                            EDIT_UpdateUniscribeData_linedef(es, core::ptr::null_mut::<HDC__>(), current_line);
                            if !(*current_line).ssa.is_null() {
                                sz_0 = ScriptString_pSize((*current_line).ssa);
                            } else {
                                sz_0 = (0 as usize) as *const SIZE;
                            }
                            if !sz_0.is_null() {
                                (*current_line).width = (*sz_0).cx as INT;
                            } else {
                                prev = 0;
                            }
                            next = (prev as c_int - 1 as c_int) as INT;
                            if !(prev != 0 && (*current_line).width > fw) {
                                break;
                            }
                        }
                        (*current_line).net_length = w as INT;

                        if prev == 0 {
                            let mut piDx: *mut INT = core::ptr::null_mut::<INT>();
                            let mut count: *const INT = core::ptr::null::<INT>();

                            EDIT_InvalidateUniscribeData_linedef(current_line);
                            EDIT_UpdateUniscribeData_linedef(es, core::ptr::null_mut::<HDC__>(), current_line);

                            if !(*current_line).ssa.is_null() {
                                count = ScriptString_pcOutChars((*current_line).ssa) as *const INT;
                                piDx = heap_alloc(
                                    (core::mem::size_of::<INT>() as SIZE_T)
                                        .wrapping_mul(*count as SIZE_T),
                                ) as *mut INT;
                                ScriptStringGetLogicalWidths((*current_line).ssa, piDx);

                                prev = ((*current_line).net_length as c_int
                                    - 1 as c_int)
                                    as INT;
                                loop {
                                    (*current_line).width -= *piDx.offset(prev as isize);
                                    prev -= 1;
                                    if !(prev > 0 && (*current_line).width > fw)
                                    {
                                        break;
                                    }
                                }
                                if prev <= 0 {
                                    prev = 1;
                                }
                                heap_free(piDx as *mut c_void);
                            } else {
                                prev = fw / (*es).char_width;
                            }
                        }

                        if (*current_line).index == nstart_index
                            && istart > (*current_line).index + prev
                        {
                            istart = (*current_line).index + prev;
                        } else if current_line == start_line
                            && (*current_line).index != nstart_index
                            && orig_net_length < prev
                        {
                            nstart_line = line_index;
                            nstart_index = (*current_line).index;
                            istart = (*current_line).index + orig_net_length;
                        }

                        (*current_line).net_length = prev;
                        (*current_line).ending = END_WRAP;

                        if (*current_line).net_length > 0 {
                            EDIT_UpdateUniscribeData_linedef(es, core::ptr::null_mut::<HDC__>(), current_line);
                            if !(*current_line).ssa.is_null() {
                                sz_0 = ScriptString_pSize((*current_line).ssa);
                                (*current_line).width = (*sz_0).cx as INT;
                            } else {
                                (*current_line).width = 0;
                            }
                        } else {
                            (*current_line).width = 0;
                        }
                    } else if current_line == start_line
                        && (*current_line).index != nstart_index
                        && orig_net_length < (*current_line).net_length
                    {
                        nstart_line = line_index;
                        nstart_index = (*current_line).index;
                        istart = (*current_line).index + orig_net_length;
                    }
                }

                match ((*current_line).ending as c_uint) as i32 {
                    END_SOFT => {
                        (*current_line).length =
                            ((*current_line).net_length as c_int + 3 as c_int)
                                as INT;
                    }
                    END_RICH => {
                        (*current_line).length =
                            ((*current_line).net_length as c_int + 1 as c_int)
                                as INT;
                    }
                    END_HARD => {
                        (*current_line).length =
                            ((*current_line).net_length as c_int + 2 as c_int)
                                as INT;
                    }
                    END_WRAP | END_0 => {
                        (*current_line).length = (*current_line).net_length;
                    }
                    _ => {}
                }

                (*es).text_width = max((*es).text_width, (*current_line).width);
                current_position = current_position.offset((*current_line).length as isize);
                previous_line = current_line;

                if line_index < (*es).y_offset || line_index > (*es).y_offset + vlc {
                    EDIT_InvalidateUniscribeData_linedef(current_line);
                }

                current_line = (*current_line).next as *mut LINEDEF;
                line_index += 1;
            }
            _ => {}
        }

        if !((*previous_line).ending as c_uint != END_0 as c_uint) {
            break;
        }
    }

    if (*previous_line).ending as c_uint == END_0 as c_uint {
        let mut pnext: *mut LINEDEF = core::ptr::null_mut::<LINEDEF>();
        (*previous_line).next = core::ptr::null_mut::<tagLINEDEF>();
        while !current_line.is_null() {
            pnext = (*current_line).next as *mut LINEDEF;
            EDIT_InvalidateUniscribeData_linedef(current_line);
            heap_free(current_line as *mut c_void);
            current_line = pnext;
            (*es).line_count -= 1;
        }
    } else if delta != 0 {
        while !current_line.is_null() {
            (*current_line).index += delta;
            current_line = (*current_line).next as *mut LINEDEF;
        }
    }

    if !hrgn.is_null() {
        let mut tmphrgn: HRGN = core::ptr::null_mut::<HRGN__>();

        rc.top = ((*es).format_rect.top as INT + nstart_line * (*es).line_height
            - (*es).y_offset * (*es).line_height) as LONG;
        rc.bottom = (rc.top as INT + (*es).line_height) as LONG;

        if ((*es).style & (ES_CENTER as DWORD)) != 0
            || ((*es).style & (ES_RIGHT as DWORD)) != 0
        {
            rc.left = (*es).format_rect.left;
        } else {
            rc.left = LOWORD!(EDIT_EM_PosFromChar(
                es,
                nstart_index,
                FALSE as BOOL,
            )) as i16 as LONG;
        }
        rc.right = (*es).format_rect.right;
        SetRectRgn(
            hrgn,
            rc.left as c_int,
            rc.top as c_int,
            rc.right as c_int,
            rc.bottom as c_int,
        );

        rc.top = rc.bottom;
        rc.left = (*es).format_rect.left;
        rc.right = (*es).format_rect.right;

        if line_count < (*es).line_count {
            rc.bottom = ((*es).line_count * (*es).line_height) as LONG;
        } else if line_count > (*es).line_count {
            rc.bottom = (line_count * (*es).line_height) as LONG;
        } else {
            rc.bottom = (line_index * (*es).line_height) as LONG;
        }
        rc.bottom += (*es).format_rect.top;
        rc.bottom -= ((*es).y_offset * (*es).line_height) as c_int;

        tmphrgn = CreateRectRgn(
            rc.left as c_int,
            rc.top as c_int,
            rc.right as c_int,
            rc.bottom as c_int,
        );
        CombineRgn(hrgn, hrgn, tmphrgn, RGN_OR as c_int);
        DeleteObject(tmphrgn as HGDIOBJ);
    }
}

unsafe extern "C" fn EDIT_CalcLineWidth_SL(mut es: *mut EDITSTATE) {
    EDIT_UpdateUniscribeData(es, core::ptr::null_mut(), 0);
    if !(*es).ssa.is_null() {
        let mut size: *const SIZE = core::ptr::null();
        size = ScriptString_pSize((*es).ssa);
        (*es).text_width = (*size).cx;
    } else {
        (*es).text_width = 0;
    }
}

unsafe extern "C" fn EDIT_CharFromPos(
    mut es: *mut EDITSTATE,
    mut x: INT,
    mut y: INT,
    mut after_wrap: LPBOOL,
) -> INT {
    let mut index: INT = 0;
    if (*es).style & ES_MULTILINE as DWORD != 0 {
        let mut trailing: ::core::ffi::c_int = 0;
        let mut line: INT = (y - (*es).format_rect.top as INT) / (*es).line_height + (*es).y_offset;
        let mut line_index: INT = 0 as INT;
        let mut line_def: *mut LINEDEF = (*es).first_line_def;
        EDIT_UpdateUniscribeData(es, ::core::ptr::null_mut::<HDC__>(), line);
        while line > 0 as ::core::ffi::c_int && !(*line_def).next.is_null() {
            line_index += (*line_def).length;
            line_def = (*line_def).next as *mut LINEDEF;
            line -= 1;
        }
        x += ((*es).x_offset as LONG - (*es).format_rect.left) as ::core::ffi::c_int;
        if (*es).style & ES_RIGHT as DWORD != 0 {
            x -= ((*es).format_rect.right as INT
                - (*es).format_rect.left as INT
                - (*line_def).width) as ::core::ffi::c_int;
        } else if (*es).style & ES_CENTER as DWORD != 0 {
            x -= ((*es).format_rect.right as ::core::ffi::c_int
                - (*es).format_rect.left as ::core::ffi::c_int
                - (*line_def).width as ::core::ffi::c_int)
                / 2 as ::core::ffi::c_int;
        }
        if x >= (*line_def).width {
            if !after_wrap.is_null() {
                *after_wrap = ((*line_def).ending as ::core::ffi::c_uint
                    == END_WRAP as ::core::ffi::c_uint)
                    as ::core::ffi::c_int as BOOL;
            }
            return line_index + (*line_def).net_length;
        }
        if x <= 0 as ::core::ffi::c_int || (*line_def).ssa.is_null() {
            if !after_wrap.is_null() {
                *after_wrap = FALSE as BOOL;
            }
            return line_index;
        }
        ScriptStringXtoCP(
            (*line_def).ssa,
            x as ::core::ffi::c_int,
            core::ptr::addr_of_mut!(index),
            core::ptr::addr_of_mut!(trailing),
        );
        if trailing != 0 {
            index += 1;
        }
        index += line_index;
        if !after_wrap.is_null() {
            *after_wrap = (index == line_index + (*line_def).net_length
                && (*line_def).ending as ::core::ffi::c_uint
                    == END_WRAP as ::core::ffi::c_uint)
                as ::core::ffi::c_int as BOOL;
        }
    } else {
        let mut xoff: INT = 0 as INT;
        let mut trailing_0: INT = 0;
        if !after_wrap.is_null() {
            *after_wrap = FALSE as BOOL;
        }
        x -= (*es).format_rect.left as ::core::ffi::c_int;
        if x == 0 {
            return (*es).x_offset;
        }
        if (*es).x_offset == 0 {
            let mut indent: INT =
                (*es).format_rect.right as INT - (*es).format_rect.left as INT - (*es).text_width;
            if (*es).style & ES_RIGHT as DWORD != 0 {
                x -= indent;
            } else if (*es).style & ES_CENTER as DWORD != 0 {
                x -= indent as ::core::ffi::c_int / 2 as ::core::ffi::c_int;
            }
        }
        EDIT_UpdateUniscribeData(es, ::core::ptr::null_mut::<HDC__>(), 0 as INT);
        if (*es).x_offset != 0 {
            if !(*es).ssa.is_null() {
                if (*es).x_offset as UINT >= get_text_length(es) {
                    let mut size: *const SIZE = ::core::ptr::null::<SIZE>();
                    size = ScriptString_pSize((*es).ssa);
                    xoff = (*size).cx as INT;
                }
                ScriptStringCPtoX(
                    (*es).ssa,
                    (*es).x_offset as ::core::ffi::c_int,
                    FALSE as BOOL,
                    core::ptr::addr_of_mut!(xoff),
                );
            } else {
                xoff = 0 as INT;
            }
        }
        if x < 0 as ::core::ffi::c_int {
            if x + xoff > 0 as ::core::ffi::c_int || (*es).ssa.is_null() {
                ScriptStringXtoCP(
                    (*es).ssa,
                    x as ::core::ffi::c_int + xoff as ::core::ffi::c_int,
                    core::ptr::addr_of_mut!(index),
                    core::ptr::addr_of_mut!(trailing_0),
                );
                if trailing_0 != 0 {
                    index += 1;
                }
            } else {
                index = 0 as INT;
            }
        } else if x != 0 {
            let mut size_0: *const SIZE = ::core::ptr::null::<SIZE>();
            if !(*es).ssa.is_null() {
                size_0 = ScriptString_pSize((*es).ssa);
            }
            if size_0.is_null() {
                index = 0 as INT;
            } else if x > (*size_0).cx {
                index = get_text_length(es) as INT;
            } else if !(*es).ssa.is_null() {
                ScriptStringXtoCP(
                    (*es).ssa,
                    x as ::core::ffi::c_int + xoff as ::core::ffi::c_int,
                    core::ptr::addr_of_mut!(index),
                    core::ptr::addr_of_mut!(trailing_0),
                );
                if trailing_0 != 0 {
                    index += 1;
                }
            } else {
                index = 0 as INT;
            }
        } else {
            index = (*es).x_offset;
        }
    }
    return index;
}

unsafe extern "C" fn EDIT_ConfinePoint(es: *const EDITSTATE, x: LPINT, y: LPINT) {
    *x = min(max(*x, (*es).format_rect.left), (*es).format_rect.right - 1);
    *y = min(max(*y, (*es).format_rect.top), (*es).format_rect.bottom - 1);
}

unsafe extern "C" fn EDIT_EM_LineFromChar(es: *mut EDITSTATE, mut index: INT) -> INT {
    let mut line: INT = 0;
    let mut line_def: *mut LINEDEF = core::ptr::null_mut::<LINEDEF>();
    if (*es).style & (ES_MULTILINE as DWORD) == 0 {
        return 0 as INT;
    }
    if index > get_text_length(es) as INT {
        return (*es).line_count - 1;
    }
    if index == -(1) {
        index = min((*es).selection_start, (*es).selection_end);
    }
    line = 0;
    line_def = (*es).first_line_def;
    index -= (*line_def).length;
    while (index >= 0) && !(*line_def).next.is_null() {
        line += 1;
        line_def = (*line_def).next as *mut LINEDEF;
        index -= (*line_def).length;
    }
    return line;
}

unsafe extern "C" fn EDIT_EM_LineIndex(es: *const EDITSTATE, mut line: INT) -> INT {
    let mut line_index: INT;
    let mut line_def: *const LINEDEF;

    if !((*es).style & (ES_MULTILINE as DWORD) != 0) {
        return 0 as INT;
    }
    if line >= (*es).line_count {
        return -(1 as INT);
    }

    line_index = 0 as INT;
    line_def = (*es).first_line_def;
    if line == -(1 as INT) {
        let mut index: INT = (*es).selection_end - (*line_def).length;
        while (index >= (0 as INT)) && !(*line_def).next.is_null() {
            line_index += (*line_def).length;
            line_def = (*line_def).next;
            index -= (*line_def).length;
        }
    } else {
        while line > (0 as INT) {
            line_index += (*line_def).length;
            line_def = (*line_def).next;
            line -= 1;
        }
    }
    return line_index;
}

unsafe extern "C" fn EDIT_EM_LineLength(es: *mut EDITSTATE, mut index: INT) -> INT {
    let mut line_def: *mut LINEDEF = core::ptr::null_mut();
    if ((*es).style & (ES_MULTILINE as DWORD)) == 0 {
        return get_text_length(es) as INT;
    }
    if index == -(1 as INT) {
        let mut l: INT32 = 0;
        let mut li: INT32 = 0;
        let mut count: INT32 = 0;
        l = EDIT_EM_LineFromChar(es, (*es).selection_start) as INT32;
        count = ((*es).selection_start - EDIT_EM_LineIndex(es, l as INT)) as INT32;
        l = EDIT_EM_LineFromChar(es, (*es).selection_end) as INT32;
        li = EDIT_EM_LineIndex(es, l as INT) as INT32;
        count += (li as INT + EDIT_EM_LineLength(es, li as INT) - (*es).selection_end) as INT32;
        return count as INT;
    }
    line_def = (*es).first_line_def;
    index -= (*line_def).length;
    while index >= (0 as INT) && !(*line_def).next.is_null() {
        line_def = (*line_def).next;
        index -= (*line_def).length;
    }
    return (*line_def).net_length;
}

unsafe extern "C" fn EDIT_EM_PosFromChar(mut es: *mut EDITSTATE, mut index: INT, mut after_wrap: BOOL) -> LRESULT {
    let mut len: INT = get_text_length(es) as INT;
    let mut l: INT = 0;
    let mut li: INT = 0;
    let mut x: INT = 0;
    let mut y: INT = 0;
    let mut w: INT = 0;
    let mut lw: INT = 0;
    let mut line_def: *mut LINEDEF = core::ptr::null_mut::<LINEDEF>();
    index = min(index, len);
    if (*es).style & ES_MULTILINE != 0 {
        l = EDIT_EM_LineFromChar(es, index);
        EDIT_UpdateUniscribeData(es, core::ptr::null_mut::<HDC__>(), l);
        y = (l - (*es).y_offset) * (*es).line_height;
        li = EDIT_EM_LineIndex(es, l);
        if after_wrap != 0 && li == index && l != 0 {
            let mut l2: INT = l - 1;
            line_def = (*es).first_line_def;
            while l2 != 0 {
                line_def = (*line_def).next as *mut LINEDEF;
                l2 -= 1;
            }
            if (*line_def).ending as u32 == (END_WRAP as u32) {
                l -= 1;
                y -= (*es).line_height;
                li = EDIT_EM_LineIndex(es, l);
            }
        }
        line_def = (*es).first_line_def;
        while (*line_def).index != li {
            line_def = (*line_def).next as *mut LINEDEF;
        }
        lw = (*line_def).width;
        w = ((*es).format_rect.right - (*es).format_rect.left) as INT;
        if !(*line_def).ssa.is_null() {
            ScriptStringCPtoX(
                (*line_def).ssa,
                (index - 1) - li,
                TRUE,
                core::ptr::addr_of_mut!(x),
            );
            x -= (*es).x_offset;
        } else {
            x = if lw > 0 {
                (*es).x_offset
            } else {
                x - (*es).x_offset
            };
        }
        if (*es).style & ES_RIGHT != 0 {
            x = w - (lw - x);
        } else if (*es).style & ES_CENTER != 0 {
            x += (w - lw) / 2;
        }
    } else {
        let mut xoff: INT = 0;
        let mut xi: INT = 0;
        EDIT_UpdateUniscribeData(es, core::ptr::null_mut::<HDC__>(), 0);
        if (*es).x_offset != 0 {
            if !(*es).ssa.is_null() {
                if (*es).x_offset as UINT >= get_text_length(es) {
                    let mut leftover: INT = ((*es).x_offset as UINT)
                        .wrapping_sub(get_text_length(es)) as INT;
                    if !(*es).ssa.is_null() {
                        let mut size: *const SIZE = core::ptr::null::<SIZE>();
                        size = ScriptString_pSize((*es).ssa);
                        xoff = (*size).cx as INT;
                    } else {
                        xoff = 0;
                    }
                    xoff += (*es).char_width * leftover;
                } else {
                    ScriptStringCPtoX(
                        (*es).ssa,
                        (*es).x_offset,
                        FALSE,
                        core::ptr::addr_of_mut!(xoff),
                    );
                }
            } else {
                xoff = 0;
            }
        }
        if index != 0 {
            if index as UINT >= get_text_length(es) {
                if !(*es).ssa.is_null() {
                    let mut size_0: *const SIZE = core::ptr::null::<SIZE>();
                    size_0 = ScriptString_pSize((*es).ssa);
                    xi = (*size_0).cx as INT;
                } else {
                    xi = 0;
                }
            } else if !(*es).ssa.is_null() {
                ScriptStringCPtoX(
                    (*es).ssa,
                    index,
                    FALSE,
                    core::ptr::addr_of_mut!(xi),
                );
            } else {
                xi = 0;
            }
        }
        x = xi - xoff;
        if index >= (*es).x_offset {
            if (*es).x_offset == 0
                && (*es).style & (ES_RIGHT | ES_CENTER) != 0
            {
                w = ((*es).format_rect.right - (*es).format_rect.left) as INT;
                if w > (*es).text_width {
                    if (*es).style & ES_RIGHT != 0 {
                        x += w - (*es).text_width;
                    } else if (*es).style & ES_CENTER != 0 {
                        x += (w - (*es).text_width) / 2;
                    }
                }
            }
        }
        y = 0;
    }
    x += (*es).format_rect.left as INT;
    y += (*es).format_rect.top as INT;
    return MAKELONG!(x as INT16, y as INT16) as LRESULT;
}

unsafe extern "C" fn EDIT_GetLineRect(
    es: *mut EDITSTATE,
    mut line: INT,
    scol: INT,
    ecol: INT,
    rc: LPRECT,
) {
    let mut ssa: SCRIPT_STRING_ANALYSIS = core::ptr::null_mut();
    let mut line_index: INT = 0;
    let mut pt1: INT = 0;
    let mut pt2: INT = 0;
    let mut pt3: INT = 0;

    if ((*es).style & (ES_MULTILINE as DWORD)) != 0 {
        let mut line_def: *const LINEDEF = core::ptr::null();
        (*rc).top =
            ((*es).format_rect.top as INT + (line - (*es).y_offset) * (*es).line_height) as LONG;
        if line >= (*es).line_count {
            return;
        }

        line_def = (*es).first_line_def;
        if line == -(1) {
            let mut index: INT = (*es).selection_end - (*line_def).length;
            while (index >= 0) && !(*line_def).next.is_null() {
                line_index += (*line_def).length;
                line_def = (*line_def).next;
                index -= (*line_def).length;
            }
        } else {
            while line > 0 {
                line_index += (*line_def).length;
                line_def = (*line_def).next;
                line -= 1;
            }
        }
        ssa = (*line_def).ssa;
    } else {
        line_index = 0;
        (*rc).top = (*es).format_rect.top;
        ssa = (*es).ssa;
    }

    (*rc).bottom = ((*rc).top as INT + (*es).line_height) as LONG;
    pt1 = (if scol == 0 {
        (*es).format_rect.left as INT
    } else {
        LOWORD!(EDIT_EM_PosFromChar(es, line_index + scol, TRUE) as DWORD_PTR)
            as i16 as INT
    }) as INT;
    pt2 = (if ecol == -(1) {
        (*es).format_rect.right as INT
    } else {
        LOWORD!(EDIT_EM_PosFromChar(es, line_index + ecol, TRUE) as DWORD_PTR)
            as i16 as INT
    }) as INT;
    if !ssa.is_null() {
        ScriptStringCPtoX(ssa, scol, FALSE, core::ptr::addr_of_mut!(pt3));
        pt3 += (*es).format_rect.left as INT;
    } else {
        pt3 = pt1;
    }
    (*rc).right = max(max(pt1, pt2), pt3) as LONG;
    (*rc).left = min(min(pt1, pt2), pt3) as LONG;
}

unsafe extern "C" fn text_buffer_changed(es: *mut EDITSTATE) {
    (*es).text_length = (-(1 as i32)) as UINT;
    heap_free((*es).logAttr as *mut core::ffi::c_void);
    (*es).logAttr = core::ptr::null_mut::<SCRIPT_LOGATTR>();
    EDIT_InvalidateUniscribeData(es);
}

unsafe extern "C" fn EDIT_LockBuffer(mut es: *mut EDITSTATE) {
    if (*es).text.is_null() {
        if (*es).hloc32W.is_null() {
            return;
        }
        (*es).text = LocalLock((*es).hloc32W) as LPWSTR;
    }
    (*es).lock_count += 1;
}

unsafe extern "C" fn EDIT_UnlockBuffer(es: *mut EDITSTATE, force: BOOL) {
    if IsWindow((*es).hwndSelf) == 0 {
        if ((*__wine_dbch___default).flags as ::core::ffi::c_int
            & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_WARN as ::core::ffi::c_int))) != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_WARN;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/win32/comctl32/edit.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"EDIT_UnlockBuffer\0" as *const u8 as *const ::core::ffi::c_char,
                1236 as ::core::ffi::c_int,
                b"edit hwnd %p already destroyed\n\0" as *const u8
                    as *const ::core::ffi::c_char,
                (*es).hwndSelf,
            );
        }
        return;
    }
    if (*es).lock_count == 0 {
        if ((*__wine_dbch___default).flags as ::core::ffi::c_int
            & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_ERR as ::core::ffi::c_int))) != 0
        {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl_0,
                __dbch_0,
                b"dll/win32/comctl32/edit.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"EDIT_UnlockBuffer\0" as *const u8 as *const ::core::ffi::c_char,
                1242 as ::core::ffi::c_int,
                b"lock_count == 0 ... please report\n\0" as *const u8
                    as *const ::core::ffi::c_char,
            );
        }
        return;
    }
    if (*es).text.is_null() {
        if ((*__wine_dbch___default).flags as ::core::ffi::c_int
            & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_ERR as ::core::ffi::c_int))) != 0
        {
            let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_1: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl_1,
                __dbch_1,
                b"dll/win32/comctl32/edit.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"EDIT_UnlockBuffer\0" as *const u8 as *const ::core::ffi::c_char,
                1248 as ::core::ffi::c_int,
                b"es->text == 0 ... please report\n\0" as *const u8
                    as *const ::core::ffi::c_char,
            );
        }
        return;
    }
    if force != 0 || (*es).lock_count == 1 {
        if !(*es).hloc32W.is_null() {
            LocalUnlock((*es).hloc32W);
            (*es).text = ::core::ptr::null_mut::<WCHAR>();
        } else {
            if ((*__wine_dbch___default).flags as ::core::ffi::c_int
                & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_ERR as ::core::ffi::c_int))) != 0
            {
                let __dbch_2: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_2: __wine_debug_class = __WINE_DBCL_ERR;
                ros_dbg_log(
                    __dbcl_2,
                    __dbch_2,
                    b"dll/win32/comctl32/edit.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"EDIT_UnlockBuffer\0" as *const u8 as *const ::core::ffi::c_char,
                    1261 as ::core::ffi::c_int,
                    b"no buffer ... please report\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                );
            }
            return;
        }
    }
    (*es).lock_count -= 1;
}

unsafe extern "C" fn EDIT_MakeFit(es: *mut EDITSTATE, size: UINT) -> BOOL {
    let mut hNew32W: HLOCAL = core::ptr::null_mut();

    if size <= (*es).buffer_size {
        return TRUE;
    }

    /* Force edit to unlock its buffer. es->text now NULL */
    EDIT_UnlockBuffer(es, TRUE);

    if !(*es).hloc32W.is_null() {
        let alloc_size: UINT = ROUND_TO_GROW!((size + 1) * core::mem::size_of::<WCHAR>() as UINT) as UINT;
        hNew32W = LocalReAlloc(
            (*es).hloc32W,
            alloc_size as SIZE_T,
            (LMEM_MOVEABLE | LMEM_ZEROINIT) as UINT,
        );
        if !hNew32W.is_null() {
            (*es).hloc32W = hNew32W;
            (*es).buffer_size = ((LocalSize(hNew32W) as usize)
                / core::mem::size_of::<WCHAR>() as usize
                - 1 as usize) as UINT;
        }
    }

    EDIT_LockBuffer(es);

    if (*es).buffer_size < size {
        notify_parent(es, (EN_ERRSPACE) as INT);
        return FALSE;
    } else {
        return TRUE;
    }
}

unsafe extern "C" fn EDIT_MakeUndoFit(es: *mut EDITSTATE, size: UINT) -> BOOL {
    let mut alloc_size: UINT;

    if size <= (*es).undo_buffer_size {
        return TRUE;
    }

    alloc_size = ROUND_TO_GROW!((size + 1) * core::mem::size_of::<WCHAR>() as UINT);
    (*es).undo_text = HeapReAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        (*es).undo_text as PVOID,
        alloc_size as SIZE_T,
    ) as LPWSTR;
    if !(*es).undo_text.is_null() {
        (*es).undo_buffer_size = alloc_size / core::mem::size_of::<WCHAR>() as UINT - 1;
        return TRUE;
    } else {
        return FALSE;
    }
}

unsafe extern "C" fn EDIT_UpdateTextRegion(es: *mut EDITSTATE, hrgn: HRGN, bErase: BOOL) {
    if ((*es).flags as i32 & EF_UPDATE) != 0 {
        (*es).flags = ((*es).flags as i32 & !EF_UPDATE) as WORD;
        if notify_parent(es, (EN_UPDATE) as INT) == 0 {
            return;
        }
    }
    InvalidateRgn((*es).hwndSelf, hrgn, bErase);
}

unsafe extern "C" fn EDIT_UpdateText(mut es: *mut EDITSTATE, mut rc: *const RECT, mut bErase: BOOL) {
    if ((*es).flags as i32 & EF_UPDATE) != 0 {
        (*es).flags = ((*es).flags as i32 & !EF_UPDATE) as WORD;
        if notify_parent(es, (EN_UPDATE) as INT) == 0 {
            return;
        }
    }
    InvalidateRect((*es).hwndSelf, rc as LPCRECT, bErase);
}

unsafe extern "C" fn EDIT_SL_InvalidateText(es: *mut EDITSTATE, start: INT, end: INT) {
    let mut line_rect: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut rc: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    EDIT_GetLineRect(es, 0, start, end, core::ptr::addr_of_mut!(line_rect));
    if IntersectRect(
        core::ptr::addr_of_mut!(rc),
        core::ptr::addr_of!(line_rect),
        core::ptr::addr_of!((*es).format_rect),
    ) != 0
    {
        EDIT_UpdateText(es, core::ptr::addr_of_mut!(rc), TRUE);
    }
}

unsafe extern "C" fn EDIT_ML_InvalidateText(es: *mut EDITSTATE, start: INT, end: INT) {
    let mut vlc: INT = get_vertical_line_count(es);
    let mut sl: INT = EDIT_EM_LineFromChar(es, start);
    let mut el: INT = EDIT_EM_LineFromChar(es, end);
    let mut sc: INT = 0;
    let mut ec: INT = 0;
    let mut rc1: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut rcWnd: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut rcLine: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut rcUpdate: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut l: INT = 0;

    if (el < (*es).y_offset) || (sl > (*es).y_offset + vlc) {
        return;
    }

    sc = start - EDIT_EM_LineIndex(es, sl);
    ec = end - EDIT_EM_LineIndex(es, el);
    if sl < (*es).y_offset {
        sl = (*es).y_offset;
        sc = 0;
    }
    if el > (*es).y_offset + vlc {
        el = (*es).y_offset + vlc;
        ec = EDIT_EM_LineLength(es, EDIT_EM_LineIndex(es, el));
    }
    GetClientRect((*es).hwndSelf, core::ptr::addr_of_mut!(rc1));
    IntersectRect(
        core::ptr::addr_of_mut!(rcWnd),
        core::ptr::addr_of_mut!(rc1) as LPCRECT,
        core::ptr::addr_of_mut!((*es).format_rect) as LPCRECT,
    );
    if sl == el {
        EDIT_GetLineRect(es, sl, sc, ec, core::ptr::addr_of_mut!(rcLine));
        if IntersectRect(
            core::ptr::addr_of_mut!(rcUpdate),
            core::ptr::addr_of_mut!(rcWnd) as LPCRECT,
            core::ptr::addr_of_mut!(rcLine) as LPCRECT,
        ) != 0
        {
            EDIT_UpdateText(es, core::ptr::addr_of_mut!(rcUpdate), TRUE);
        }
    } else {
        EDIT_GetLineRect(
            es,
            sl,
            sc,
            EDIT_EM_LineLength(es, EDIT_EM_LineIndex(es, sl)),
            core::ptr::addr_of_mut!(rcLine),
        );
        if IntersectRect(
            core::ptr::addr_of_mut!(rcUpdate),
            core::ptr::addr_of_mut!(rcWnd) as LPCRECT,
            core::ptr::addr_of_mut!(rcLine) as LPCRECT,
        ) != 0
        {
            EDIT_UpdateText(es, core::ptr::addr_of_mut!(rcUpdate), TRUE);
        }
        l = sl + 1;
        while l < el {
            EDIT_GetLineRect(
                es,
                l,
                0,
                EDIT_EM_LineLength(es, EDIT_EM_LineIndex(es, l)),
                core::ptr::addr_of_mut!(rcLine),
            );
            if IntersectRect(
                core::ptr::addr_of_mut!(rcUpdate),
                core::ptr::addr_of_mut!(rcWnd) as LPCRECT,
                core::ptr::addr_of_mut!(rcLine) as LPCRECT,
            ) != 0
            {
                EDIT_UpdateText(es, core::ptr::addr_of_mut!(rcUpdate), TRUE);
            }
            l += 1;
        }
        EDIT_GetLineRect(es, el, 0, ec, core::ptr::addr_of_mut!(rcLine));
        if IntersectRect(
            core::ptr::addr_of_mut!(rcUpdate),
            core::ptr::addr_of_mut!(rcWnd) as LPCRECT,
            core::ptr::addr_of_mut!(rcLine) as LPCRECT,
        ) != 0
        {
            EDIT_UpdateText(es, core::ptr::addr_of_mut!(rcUpdate), TRUE);
        }
    }
}

unsafe extern "C" fn EDIT_InvalidateText(mut es: *mut EDITSTATE, mut start: INT, mut end: INT) {
    if end == start {
        return;
    }
    if end == -1 {
        end = get_text_length(es) as INT;
    }
    if end < start {
        let mut tmp: INT = start;
        start = end;
        end = tmp;
    }
    if ((*es).style & (ES_MULTILINE as DWORD)) != 0 {
        EDIT_ML_InvalidateText(es, start, end);
    } else {
        EDIT_SL_InvalidateText(es, start, end);
    }
}

unsafe extern "C" fn EDIT_EM_SetSel(es: *mut EDITSTATE, mut start: UINT, mut end: UINT, after_wrap: BOOL) -> BOOL {
    let mut old_start: UINT = (*es).selection_start as UINT;
    let mut old_end: UINT = (*es).selection_end as UINT;
    let mut len: UINT = get_text_length(es);

    if start == old_start && end == old_end {
        return FALSE;
    }

    if start == (-(1 as ::core::ffi::c_int)) as UINT {
        start = (*es).selection_end as UINT;
        end = (*es).selection_end as UINT;
    } else {
        start = min(start, len);
        end = min(end, len);
    }
    (*es).selection_start = start as INT;
    (*es).selection_end = end as INT;
    if after_wrap != 0 {
        (*es).flags = ((*es).flags as ::core::ffi::c_int | EF_AFTER_WRAP) as WORD;
    } else {
        (*es).flags = ((*es).flags as ::core::ffi::c_int & !EF_AFTER_WRAP) as WORD;
    }

    /* ORDER_UINT(end, old_end); */
    if old_end < end {
        let mut temp: UINT = end;
        end = old_end;
        old_end = temp;
    }
    /* ORDER_UINT(start, old_start); */
    if old_start < start {
        let mut temp_0: UINT = start;
        start = old_start;
        old_start = temp_0;
    }
    /* ORDER_UINT(old_start, old_end); */
    if old_end < old_start {
        let mut temp_1: UINT = old_start;
        old_start = old_end;
        old_end = temp_1;
    }
    /* ORDER_UINT(start, end); */
    if end < start {
        let mut temp_2: UINT = start;
        start = end;
        end = temp_2;
    }

    if end != old_start {
        if old_start > end {
            EDIT_InvalidateText(es, start as INT, end as INT);
            EDIT_InvalidateText(es, old_start as INT, old_end as INT);
        } else {
            EDIT_InvalidateText(es, start as INT, old_start as INT);
            EDIT_InvalidateText(es, end as INT, old_end as INT);
        }
    } else {
        EDIT_InvalidateText(es, start as INT, old_end as INT);
    }

    return TRUE;
}

unsafe extern "C" fn EDIT_UpdateScrollInfo(es: *mut EDITSTATE) {
    if ((*es).style & (WS_VSCROLL as u32)) != 0
        && ((*es).flags as u32 & (EF_VSCROLL_TRACK as u32)) == 0
    {
        let mut si: SCROLLINFO = core::mem::zeroed();
        si.cbSize = core::mem::size_of::<SCROLLINFO>() as u32;
        si.fMask = (SIF_PAGE as u32)
            | (SIF_POS as u32)
            | (SIF_RANGE as u32)
            | (SIF_DISABLENOSCROLL as u32);
        si.nMin = 0 as i32;
        si.nMax = (*es).line_count - (1 as i32);
        si.nPage = (((*es).format_rect.bottom - (*es).format_rect.top) / (*es).line_height) as u32;
        si.nPos = (*es).y_offset;

        SetScrollInfo(
            (*es).hwndSelf,
            SB_VERT as i32,
            core::ptr::addr_of_mut!(si) as LPCSCROLLINFO,
            TRUE as i32,
        );
    }

    if ((*es).style & (WS_HSCROLL as u32)) != 0
        && ((*es).flags as u32 & (EF_HSCROLL_TRACK as u32)) == 0
    {
        let mut si: SCROLLINFO = core::mem::zeroed();
        si.cbSize = core::mem::size_of::<SCROLLINFO>() as u32;
        si.fMask = (SIF_PAGE as u32)
            | (SIF_POS as u32)
            | (SIF_RANGE as u32)
            | (SIF_DISABLENOSCROLL as u32);
        si.nMin = 0 as i32;
        si.nMax = (*es).text_width - (1 as i32);
        si.nPage = ((*es).format_rect.right - (*es).format_rect.left) as u32;
        si.nPos = (*es).x_offset;

        SetScrollInfo(
            (*es).hwndSelf,
            SB_HORZ as i32,
            core::ptr::addr_of_mut!(si) as LPCSCROLLINFO,
            TRUE as i32,
        );
    }
}

unsafe extern "C" fn EDIT_EM_LineScroll_internal(mut es: *mut EDITSTATE, mut dx: INT, mut dy: INT) -> BOOL {
    let mut nyoff: INT = 0;
    let mut x_offset_in_pixels: INT = 0;
    let mut lines_per_page: INT =
        ((*es).format_rect.bottom - (*es).format_rect.top) / (*es).line_height;

    if ((*es).style & (ES_MULTILINE as DWORD)) != 0 {
        x_offset_in_pixels = (*es).x_offset;
    } else {
        dy = 0 as INT;
        x_offset_in_pixels = (LOWORD!(
            EDIT_EM_PosFromChar(es, (*es).x_offset, FALSE as BOOL) as DWORD_PTR,
        ) as i16) as INT;
    }

    if -dx > x_offset_in_pixels {
        dx = -x_offset_in_pixels;
    }
    if dx > (*es).text_width - x_offset_in_pixels {
        dx = (*es).text_width - x_offset_in_pixels;
    }
    nyoff = max(0 as INT, (*es).y_offset + dy);
    if nyoff >= (*es).line_count - lines_per_page {
        nyoff = max(
            0 as INT,
            (*es).line_count - lines_per_page,
        );
    }
    dy = ((*es).y_offset - nyoff) * (*es).line_height;
    if dx != 0 || dy != 0 {
        let mut rc1: RECT = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        let mut rc: RECT = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };

        (*es).y_offset = nyoff;
        if ((*es).style & (ES_MULTILINE as DWORD)) != 0 {
            (*es).x_offset += dx;
        } else {
            (*es).x_offset += dx / (*es).char_width;
        }

        GetClientRect((*es).hwndSelf, core::ptr::addr_of_mut!(rc1));
        IntersectRect(
            core::ptr::addr_of_mut!(rc),
            core::ptr::addr_of_mut!(rc1),
            core::ptr::addr_of_mut!((*es).format_rect),
        );
        ScrollWindowEx(
            (*es).hwndSelf,
            -dx,
            dy,
            core::ptr::null(),
            core::ptr::addr_of_mut!(rc),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            SW_INVALIDATE as UINT,
        );
        /* force scroll info update */
        EDIT_UpdateScrollInfo(es);
    }
    if dx != 0 && ((*es).flags as DWORD & (EF_HSCROLL_TRACK as DWORD)) == 0 {
        notify_parent(es, EN_HSCROLL as INT);
    }
    if dy != 0 && ((*es).flags as DWORD & (EF_VSCROLL_TRACK as DWORD)) == 0 {
        notify_parent(es, EN_VSCROLL as INT);
    }
    return TRUE as BOOL;
}

unsafe extern "C" fn EDIT_EM_LineScroll(mut es: *mut EDITSTATE, mut dx: INT, mut dy: INT) -> BOOL {
    if ((*es).style & ES_MULTILINE) == 0 {
        return FALSE;
    }
    dx *= (*es).char_width;
    return EDIT_EM_LineScroll_internal(es, dx, dy);
}

unsafe extern "C" fn EDIT_EM_Scroll(es: *mut EDITSTATE, action: INT) -> LRESULT {
    let mut dy: INT;

    if ((*es).style & ES_MULTILINE) == 0 {
        return FALSE as LRESULT;
    }

    dy = 0;

    if (action) as u32 == SB_LINEUP {
        if (*es).y_offset != 0 {
            dy = -1;
        }
    } else if (action) as u32 == SB_LINEDOWN {
        if (*es).y_offset < (*es).line_count - 1 {
            dy = 1;
        }
    } else if (action) as u32 == SB_PAGEUP {
        if (*es).y_offset != 0 {
            dy = -((*es).format_rect.bottom - (*es).format_rect.top) / (*es).line_height;
        }
    } else if (action) as u32 == SB_PAGEDOWN {
        if (*es).y_offset < (*es).line_count - 1 {
            dy = ((*es).format_rect.bottom - (*es).format_rect.top) / (*es).line_height;
        }
    } else {
        return FALSE as LRESULT;
    }

    if dy != 0 {
        let vlc: INT = get_vertical_line_count(es);
        /* check if we are going to move too far */
        if (*es).y_offset + dy > (*es).line_count - vlc {
            dy = max((*es).line_count - vlc, 0) - (*es).y_offset;
        }

        /* Notification is done in EDIT_EM_LineScroll */
        if dy != 0 {
            EDIT_EM_LineScroll(es, 0, dy);
            return MAKELONG!(dy as WORD, TRUE as WORD) as LRESULT;
        }
    }
    return FALSE as LRESULT;
}

unsafe extern "C" fn EDIT_ImmSetCompositionWindow(es: *mut EDITSTATE, pt: POINT) {
    let mut CompForm: COMPOSITIONFORM = core::mem::zeroed();
    let hIMC: HIMC = ImmGetContext((*es).hwndSelf);
    if hIMC.is_null() {
        return;
    }

    CompForm.ptCurrentPos = pt;
    if ((*es).style & ES_MULTILINE) != 0 {
        CompForm.dwStyle = CFS_RECT;
        CompForm.rcArea = (*es).format_rect;
    } else {
        CompForm.dwStyle = CFS_POINT;
        SetRectEmpty(core::ptr::addr_of_mut!(CompForm.rcArea));
    }

    ImmSetCompositionWindow(hIMC, core::ptr::addr_of_mut!(CompForm));
    ImmReleaseContext((*es).hwndSelf, hIMC);
}

unsafe extern "C" fn EDIT_SetCaretPos(es: *mut EDITSTATE, pos: INT, after_wrap: BOOL) {
    let res: LRESULT = EDIT_EM_PosFromChar(es, pos, after_wrap);
    let hKL: HKL = GetKeyboardLayout(0);
    let pt: POINT = POINT {
        x: (LOWORD!(res as DWORD_PTR) as WORD as ::core::ffi::c_short) as LONG,
        y: (HIWORD!(res as DWORD_PTR) as WORD as ::core::ffi::c_short) as LONG,
    };

    /* Don't set caret if not focused */
    if (*es).flags as ::core::ffi::c_int & EF_FOCUSED == 0 {
        return;
    }

    SetCaretPos(pt.x as ::core::ffi::c_int, pt.y as ::core::ffi::c_int);

    if ImmIsIME(hKL) != 0 {
        EDIT_ImmSetCompositionWindow(es, pt);
    }
}

unsafe extern "C" fn EDIT_EM_ScrollCaret(mut es: *mut EDITSTATE) {
    if (*es).style & ES_MULTILINE as DWORD != 0 {
        let mut l: INT = 0;
        let mut vlc: INT = 0;
        let mut ww: INT = 0;
        let mut cw: INT = (*es).char_width;
        let mut x: INT = 0;
        let mut dy: INT = 0 as INT;
        let mut dx: INT = 0 as INT;
        l = EDIT_EM_LineFromChar(es, (*es).selection_end);
        x = LOWORD!(EDIT_EM_PosFromChar(es, (*es).selection_end, ((*es).flags as BOOL & EF_AFTER_WRAP)) as DWORD_PTR) as WORD as ::core::ffi::c_short as INT;
        vlc = get_vertical_line_count(es);
        if l >= (*es).y_offset + vlc {
            dy = l - vlc + 1 as INT - (*es).y_offset;
        }
        if l < (*es).y_offset {
            dy = l - (*es).y_offset;
        }
        ww = ((*es).format_rect.right - (*es).format_rect.left) as INT;
        if x < (*es).format_rect.left as INT {
            dx = x - (*es).format_rect.left as INT - ww / HSCROLL_FRACTION / cw * cw;
        }
        if x > (*es).format_rect.right as INT {
            dx = x - (*es).format_rect.left as INT - (HSCROLL_FRACTION - 1) * ww / HSCROLL_FRACTION / cw * cw;
        }
        if dy != 0 || dx != 0 || ((*es).y_offset != 0 && ((*es).line_count - (*es).y_offset < vlc)) {
            /* check if we are going to move too far */
            if (*es).x_offset + dx + ww > (*es).text_width {
                dx = (*es).text_width - ww - (*es).x_offset;
            }
            if dx != 0 || dy != 0 || ((*es).y_offset != 0 && ((*es).line_count - (*es).y_offset < vlc)) {
                EDIT_EM_LineScroll_internal(es, dx, dy);
            }
        }
    } else {
        let mut x_0: INT = 0;
        let mut goal: INT = 0;
        let mut format_width: INT = 0;
        x_0 = LOWORD!(EDIT_EM_PosFromChar(es, (*es).selection_end, FALSE) as DWORD_PTR) as WORD as ::core::ffi::c_short as INT;
        format_width = ((*es).format_rect.right - (*es).format_rect.left) as INT;
        if x_0 < (*es).format_rect.left as INT {
            goal = ((*es).format_rect.left as INT + format_width / HSCROLL_FRACTION);
            loop {
                (*es).x_offset -= 1;
                x_0 = LOWORD!(EDIT_EM_PosFromChar(es, (*es).selection_end, FALSE) as DWORD_PTR) as WORD as ::core::ffi::c_short as INT;
                if !((x_0 < goal) && (*es).x_offset != 0) {
                    break;
                }
            }
            /* FIXME: use ScrollWindow() somehow to improve performance */
            EDIT_UpdateText(es, ::core::ptr::null::<RECT>() as *mut RECT as *const RECT, TRUE);
        } else if x_0 > (*es).format_rect.right as INT {
            let mut x_last: INT = 0;
            let mut len: INT = get_text_length(es) as INT;
            goal = ((*es).format_rect.right as INT - format_width / HSCROLL_FRACTION);
            loop {
                (*es).x_offset += 1;
                x_0 = LOWORD!(EDIT_EM_PosFromChar(es, (*es).selection_end, FALSE) as DWORD_PTR) as WORD as ::core::ffi::c_short as INT;
                x_last = LOWORD!(EDIT_EM_PosFromChar(es, len, FALSE) as DWORD_PTR) as WORD as ::core::ffi::c_short as INT;
                if !((x_0 > goal) && (x_last > (*es).format_rect.right as INT)) {
                    break;
                }
            }
            /* FIXME: use ScrollWindow() somehow to improve performance */
            EDIT_UpdateText(es, ::core::ptr::null::<RECT>() as *mut RECT as *const RECT, TRUE);
        }
    }

    EDIT_SetCaretPos(es, (*es).selection_end, ((*es).flags as BOOL & EF_AFTER_WRAP));
}

unsafe extern "C" fn EDIT_MoveBackward(es: *mut EDITSTATE, extend: BOOL) {
    let mut e: INT = (*es).selection_end;
    if e != 0 {
        e -= 1;
        if ((*es).style & ES_MULTILINE as DWORD) != 0
            && e != 0
            && *(*es).text.offset((e - 1 as INT) as isize) as i32 == '\r' as i32
            && *(*es).text.offset(e as isize) as i32 == '\n' as i32
        {
            e -= 1;
            if e != 0
                && *(*es).text.offset((e - 1 as INT) as isize) as i32 == '\r' as i32
            {
                e -= 1;
            }
        }
    }
    EDIT_EM_SetSel(
        es,
        (if extend != 0 {
            (*es).selection_start
        } else {
            e
        }) as UINT,
        e as UINT,
        FALSE,
    );
    EDIT_EM_ScrollCaret(es);
}

unsafe extern "C" fn EDIT_MoveDown_ML(es: *mut EDITSTATE, extend: BOOL) {
    let mut s: INT = (*es).selection_start;
    let mut e: INT = (*es).selection_end;
    let mut after_wrap: BOOL = ((*es).flags as BOOL & EF_AFTER_WRAP);
    let pos: LRESULT = EDIT_EM_PosFromChar(es, e, after_wrap);
    let x: INT = (LOWORD!(pos) as i16) as INT;
    let y: INT = (HIWORD!(pos) as i16) as INT;

    e = EDIT_CharFromPos(es, x, y + (*es).line_height, core::ptr::addr_of_mut!(after_wrap));
    if extend == 0 {
        s = e;
    }
    EDIT_EM_SetSel(es, s as UINT, e as UINT, after_wrap);
    EDIT_EM_ScrollCaret(es);
}

unsafe extern "C" fn EDIT_MoveEnd(es: *mut EDITSTATE, extend: BOOL, ctrl: BOOL) {
    let mut after_wrap: BOOL = FALSE;
    let mut e: INT;

    /* Pass a high value in x to make sure of receiving the end of the line */
    if ctrl == 0 && ((*es).style & ES_MULTILINE as DWORD) != 0 {
        e = EDIT_CharFromPos(
            es,
            0x3fffffff as INT,
            HIWORD!(EDIT_EM_PosFromChar(
                es,
                (*es).selection_end,
                (*es).flags as BOOL & EF_AFTER_WRAP as BOOL,
            ) as DWORD) as WORD as INT,
            core::ptr::addr_of_mut!(after_wrap),
        );
    } else {
        e = get_text_length(es) as INT;
    }
    EDIT_EM_SetSel(
        es,
        (if extend != 0 { (*es).selection_start } else { e }) as UINT,
        e as UINT,
        after_wrap,
    );
    EDIT_EM_ScrollCaret(es);
}

unsafe extern "C" fn EDIT_MoveForward(mut es: *mut EDITSTATE, mut extend: BOOL) {
    let mut e: INT = (*es).selection_end;
    if *(*es).text.offset(e as isize) != 0 {
        e += 1;
        if ((*es).style & ES_MULTILINE as DWORD) != 0
            && *(*es).text.offset((e - 1) as isize) == '\r' as u16
        {
            if *(*es).text.offset(e as isize) == '\n' as u16 {
                e += 1;
            } else if *(*es).text.offset(e as isize) == '\r' as u16
                && *(*es).text.offset((e + 1) as isize) == '\n' as u16
            {
                e += 2;
            }
        }
    }
    EDIT_EM_SetSel(
        es,
        (if extend != 0 {
            (*es).selection_start
        } else {
            e
        }) as UINT,
        e as UINT,
        FALSE as BOOL,
    );
    EDIT_EM_ScrollCaret(es);
}

unsafe extern "C" fn EDIT_MoveHome(mut es: *mut EDITSTATE, mut extend: BOOL, mut ctrl: BOOL) {
    let mut e: INT = 0;
    if ctrl == 0 && ((*es).style & ES_MULTILINE as DWORD) != 0 {
        e = EDIT_CharFromPos(
            es,
            -(*es).x_offset,
            HIWORD!(EDIT_EM_PosFromChar(
                es,
                (*es).selection_end,
                ((*es).flags as BOOL & EF_AFTER_WRAP as BOOL),
            ) as DWORD) as WORD as INT,
            core::ptr::null_mut::<BOOL>(),
        );
    } else {
        e = 0 as INT;
    }
    EDIT_EM_SetSel(
        es,
        (if extend != 0 {
            (*es).selection_start
        } else {
            e
        }) as UINT,
        e as UINT,
        FALSE as BOOL,
    );
    EDIT_EM_ScrollCaret(es);
}

unsafe extern "C" fn EDIT_MovePageDown_ML(es: *mut EDITSTATE, extend: BOOL) {
    let mut s: INT = (*es).selection_start;
    let mut e: INT = (*es).selection_end;
    let mut after_wrap: BOOL = ((*es).flags as BOOL) & EF_AFTER_WRAP;
    let mut pos: LRESULT = EDIT_EM_PosFromChar(es, e, after_wrap);
    let mut x: INT = LOWORD!(pos) as WORD as ::core::ffi::c_short as INT;
    let mut y: INT = HIWORD!(pos) as WORD as ::core::ffi::c_short as INT;

    e = EDIT_CharFromPos(
        es,
        x,
        y + ((*es).format_rect.bottom - (*es).format_rect.top),
        core::ptr::addr_of_mut!(after_wrap),
    );
    if extend == 0 {
        s = e;
    }
    EDIT_EM_SetSel(es, s as UINT, e as UINT, after_wrap);
    EDIT_EM_ScrollCaret(es);
}

unsafe extern "C" fn EDIT_MovePageUp_ML(es: *mut EDITSTATE, extend: BOOL) {
    let mut s: INT = (*es).selection_start;
    let mut e: INT = (*es).selection_end;
    let mut after_wrap: BOOL = (*es).flags as BOOL & EF_AFTER_WRAP;
    let pos: LRESULT = EDIT_EM_PosFromChar(es, e, after_wrap);
    let x: INT = LOWORD!(pos) as ::core::ffi::c_short as INT;
    let y: INT = HIWORD!(pos) as ::core::ffi::c_short as INT;

    e = EDIT_CharFromPos(
        es,
        x,
        y - ((*es).format_rect.bottom - (*es).format_rect.top),
        core::ptr::addr_of_mut!(after_wrap),
    );
    if extend == 0 {
        s = e;
    }
    EDIT_EM_SetSel(es, (s) as UINT, (e) as UINT, after_wrap);
    EDIT_EM_ScrollCaret(es);
}

unsafe extern "C" fn EDIT_MoveUp_ML(es: *mut EDITSTATE, extend: BOOL) {
    let mut s: INT = (*es).selection_start;
    let mut e: INT = (*es).selection_end;
    let mut after_wrap: BOOL = ((*es).flags as BOOL & EF_AFTER_WRAP);
    let pos: LRESULT = EDIT_EM_PosFromChar(es, e, after_wrap);
    let mut x: INT = LOWORD!(pos as DWORD) as WORD as ::core::ffi::c_short as INT;
    let mut y: INT = HIWORD!(pos as DWORD) as WORD as ::core::ffi::c_short as INT;

    e = EDIT_CharFromPos(es, x, y - (*es).line_height, core::ptr::addr_of_mut!(after_wrap));
    if extend == 0 {
        s = e;
    }
    EDIT_EM_SetSel(es, s as UINT, e as UINT, after_wrap);
    EDIT_EM_ScrollCaret(es);
}

unsafe extern "C" fn EDIT_MoveWordBackward(mut es: *mut EDITSTATE, mut extend: BOOL) {
    let mut s: INT = (*es).selection_start;
    let mut e: INT = (*es).selection_end;
    let mut l: INT = 0;
    let mut ll: INT = 0;
    let mut li: INT = 0;
    l = EDIT_EM_LineFromChar(es, e);
    ll = EDIT_EM_LineLength(es, e);
    li = EDIT_EM_LineIndex(es, l);
    if e - li == 0 {
        if l != 0 {
            li = EDIT_EM_LineIndex(es, l - 1);
            e = li + EDIT_EM_LineLength(es, li);
        }
    } else {
        e = li + EDIT_CallWordBreakProc(es, li, e - li, ll, (WB_LEFT) as INT);
    }
    if extend == 0 {
        s = e;
    }
    EDIT_EM_SetSel(es, s as UINT, e as UINT, FALSE);
    EDIT_EM_ScrollCaret(es);
}

unsafe extern "C" fn EDIT_MoveWordForward(es: *mut EDITSTATE, extend: BOOL) {
    let mut s: INT = (*es).selection_start;
    let mut e: INT = (*es).selection_end;
    let mut l: INT;
    let mut ll: INT;
    let mut li: INT;

    l = EDIT_EM_LineFromChar(es, e);
    ll = EDIT_EM_LineLength(es, e);
    li = EDIT_EM_LineIndex(es, l);
    if e - li == ll {
        if ((*es).style & (ES_MULTILINE as DWORD)) != 0
            && l != (*es).line_count - 1
        {
            e = EDIT_EM_LineIndex(es, l + 1);
        }
    } else {
        e = li + EDIT_CallWordBreakProc(
            es,
            li,
            e - li + 1,
            ll,
            (WB_RIGHT) as INT,
        );
    }
    if extend == 0 {
        s = e;
    }
    EDIT_EM_SetSel(es, s as UINT, e as UINT, FALSE);
    EDIT_EM_ScrollCaret(es);
}

unsafe extern "C" fn EDIT_PaintText(es: *mut EDITSTATE, dc: HDC, x: INT, y: INT, line: INT, col: INT, count: INT, rev: BOOL) -> INT {
	let mut BkColor: COLORREF = 0;
	let mut TextColor: COLORREF = 0;
	let mut underline_font: LOGFONTW = core::mem::zeroed();
	let mut hUnderline: HFONT = core::ptr::null_mut();
	let mut old_font: HFONT = core::ptr::null_mut();
	let mut ret: INT;
	let mut li: INT;
	let mut BkMode: INT;
	let mut size: SIZE = core::mem::zeroed();

	if count == 0 {
		return 0;
	}
	BkMode = GetBkMode(dc);
	BkColor = GetBkColor(dc);
	TextColor = GetTextColor(dc);
	if rev != 0 {
		if 1 != 0 {
			SetBkColor(dc, GetSysColor((COLOR_HIGHLIGHT) as i32) as COLORREF);
			SetTextColor(dc, GetSysColor((COLOR_HIGHLIGHTTEXT) as i32) as COLORREF);
			SetBkMode(dc, (OPAQUE as i32));
		} else {
			let current: HFONT = (GetCurrentObject(dc, OBJ_FONT)).cast();
			GetObjectW((current).cast(), core::mem::size_of::<LOGFONTW>() as INT, core::ptr::addr_of_mut!(underline_font) as *mut core::ffi::c_void);
			underline_font.lfUnderline = TRUE as BYTE;
			hUnderline = CreateFontIndirectW(core::ptr::addr_of_mut!(underline_font));
			old_font = (SelectObject(dc, (hUnderline).cast())).cast();
		}
	}
	li = EDIT_EM_LineIndex(es, line);
	if (*es).style & (ES_MULTILINE as DWORD) != 0 {
		ret = (TabbedTextOutW(
			dc,
			x as ::core::ffi::c_int,
			y as ::core::ffi::c_int,
			(*es).text.offset(li as isize).offset(col as isize) as LPCWSTR,
			count as ::core::ffi::c_int,
			(*es).tabs_count as ::core::ffi::c_int,
			(*es).tabs as *const INT,
			(*es).format_rect.left as ::core::ffi::c_int - (*es).x_offset as ::core::ffi::c_int,
		) as DWORD_PTR & 0xffff as DWORD_PTR) as WORD as INT;
	} else {
		TextOutW(
			dc,
			x as ::core::ffi::c_int,
			y as ::core::ffi::c_int,
			(*es).text.offset(li as isize).offset(col as isize) as LPCWSTR,
			count as ::core::ffi::c_int,
		);
		GetTextExtentPoint32W(
			dc,
			(*es).text.offset(li as isize).offset(col as isize) as LPCWSTR,
			count as ::core::ffi::c_int,
			core::ptr::addr_of_mut!(size),
		);
		ret = size.cx as INT;
	}
	if rev != 0 {
		if 1 != 0 {
			SetBkColor(dc, BkColor);
			SetTextColor(dc, TextColor);
			SetBkMode(dc, BkMode);
		} else {
			if old_font != core::ptr::null_mut() {
				SelectObject(dc, (old_font).cast());
			}
			if hUnderline != core::ptr::null_mut() {
				DeleteObject((hUnderline).cast());
			}
		}
	}
	return ret;
}

unsafe extern "C" fn EDIT_PaintLine(es: *mut EDITSTATE, dc: HDC, line: INT, rev: BOOL) {
    let mut s: INT = 0;
    let mut e: INT = 0;
    let mut li: INT = 0;
    let mut ll: INT = 0;
    let mut x: INT;
    let mut y: INT;
    let mut pos: LRESULT;
    let mut ssa: SCRIPT_STRING_ANALYSIS;

    if ((*es).style & (ES_MULTILINE as DWORD)) != 0 {
        let vlc: INT = get_vertical_line_count(es);

        if line < (*es).y_offset || line > (*es).y_offset + vlc || line >= (*es).line_count {
            return;
        }
    } else if line != 0 {
        return;
    }

    ssa = EDIT_UpdateUniscribeData(es, dc, line);
    pos = EDIT_EM_PosFromChar(
        es,
        EDIT_EM_LineIndex(es, line),
        FALSE,
    );
    x = (LOWORD!(pos as DWORD) as WORD) as i16 as INT;
    y = (HIWORD!(pos as DWORD) as WORD) as i16 as INT;

    if ((*es).style & (ES_MULTILINE as DWORD)) != 0 {
        let mut line_idx: INT = line;
        x = -(*es).x_offset;
        if ((*es).style & (ES_RIGHT as DWORD)) != 0
            || ((*es).style & (ES_CENTER as DWORD)) != 0
        {
            let mut line_def: *mut LINEDEF = (*es).first_line_def;
            let mut w: INT;
            let mut lw: INT;

            while !line_def.is_null() && line_idx != 0 {
                line_def = (*line_def).next as *mut LINEDEF;
                line_idx -= 1;
            }
            w = (*es).format_rect.right - (*es).format_rect.left;
            lw = (*line_def).width;

            if ((*es).style & (ES_RIGHT as DWORD)) != 0 {
                x = w - (lw - x);
            } else if ((*es).style & (ES_CENTER as DWORD)) != 0 {
                x += (w - lw) / 2;
            }
        }
        x += (*es).format_rect.left;
    }

    if rev != 0 {
        li = EDIT_EM_LineIndex(es, line);
        ll = EDIT_EM_LineLength(es, li);
        s = min((*es).selection_start, (*es).selection_end);
        e = max((*es).selection_start, (*es).selection_end);
        s = min(li + ll, max(li, s));
        e = min(li + ll, max(li, e));
    }

    if !ssa.is_null() {
        ScriptStringOut(
            ssa,
            x,
            y,
            0,
            core::ptr::addr_of_mut!((*es).format_rect),
            s - li,
            e - li,
            FALSE,
        );
    } else if rev != 0
        && s != e
        && (((*es).flags as INT & (EF_FOCUSED as INT)) != 0
            || ((*es).style & (ES_NOHIDESEL as DWORD)) != 0)
    {
        x += EDIT_PaintText(es, dc, x, y, line, 0, s - li, FALSE);
        x += EDIT_PaintText(es, dc, x, y, line, s - li, e - s, TRUE);
        x += EDIT_PaintText(es, dc, x, y, line, e - li, li + ll - e, FALSE);
    } else {
        x += EDIT_PaintText(es, dc, x, y, line, 0, ll, FALSE);
    }

    if !(*es).cue_banner_text.is_null()
        && (*es).text_length == 0
        && (((*es).flags as INT & (EF_FOCUSED as INT)) == 0
            || (*es).cue_banner_draw_focused != 0)
    {
        SetTextColor(dc, GetSysColor((COLOR_GRAYTEXT) as i32) as COLORREF);
        TextOutW(
            dc,
            x,
            y,
            (*es).cue_banner_text,
            lstrlenW((*es).cue_banner_text as LPCWSTR),
        );
    }
}

unsafe extern "C" fn EDIT_AdjustFormatRect(es: *mut EDITSTATE) {
    let mut ClientRect: RECT = tagRECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    (*es).format_rect.right = max(
        (*es).format_rect.right,
        (*es).format_rect.left + (*es).char_width,
    );
    if ((*es).style & (ES_MULTILINE as DWORD)) != 0 {
        let mut fw: INT = 0;
        let mut vlc: INT = 0;
        let mut max_x_offset: INT = 0;
        let mut max_y_offset: INT = 0;

        vlc = get_vertical_line_count(es);
        (*es).format_rect.bottom = (*es).format_rect.top + vlc * (*es).line_height;

        /* correct es->x_offset */
        fw = (*es).format_rect.right - (*es).format_rect.left;
        max_x_offset = (*es).text_width - fw;
        if max_x_offset < 0 {
            max_x_offset = 0;
        }
        if (*es).x_offset > max_x_offset {
            (*es).x_offset = max_x_offset;
        }

        /* correct es->y_offset */
        max_y_offset = (*es).line_count - vlc;
        if max_y_offset < 0 {
            max_y_offset = 0;
        }
        if (*es).y_offset > max_y_offset {
            (*es).y_offset = max_y_offset;
        }

        /* force scroll info update */
        EDIT_UpdateScrollInfo(es);
    } else {
        /* Windows doesn't care to fix text placement for SL controls */
        (*es).format_rect.bottom = (*es).format_rect.top + (*es).line_height;
    }

    /* Always stay within the client area */
    GetClientRect((*es).hwndSelf, core::ptr::addr_of_mut!(ClientRect));
    (*es).format_rect.bottom = min((*es).format_rect.bottom, ClientRect.bottom);

    if ((*es).style & (ES_MULTILINE as DWORD)) != 0
        && ((*es).style & (ES_AUTOHSCROLL as DWORD)) == 0
    {
        EDIT_BuildLineDefs_ML(
            es,
            0 as INT,
            get_text_length(es) as INT,
            0 as INT,
            core::ptr::null_mut(),
        );
    }

    EDIT_SetCaretPos(
        es,
        (*es).selection_end,
        (*es).flags as BOOL & (EF_AFTER_WRAP as BOOL),
    );
}

unsafe extern "C" fn EDIT_SetRectNP(es: *mut EDITSTATE, rc: *const RECT) {
    let mut ExStyle: LONG_PTR = 0;
    let mut bw: INT = 0;
    let mut bh: INT = 0;
    ExStyle = GetWindowLongPtrW((*es).hwndSelf, GWL_EXSTYLE);

    CopyRect(core::ptr::addr_of_mut!((*es).format_rect), rc);

    if (ExStyle & (WS_EX_CLIENTEDGE) as LONG_PTR) != 0 {
        (*es).format_rect.left += 1;
        (*es).format_rect.right -= 1;

        if (*es).format_rect.bottom - (*es).format_rect.top
            >= (*es).line_height + 2
        {
            (*es).format_rect.top += 1;
            (*es).format_rect.bottom -= 1;
        }
    }
    else if ((*es).style & WS_BORDER) != 0 {
        bw = GetSystemMetrics((SM_CXBORDER) as i32) + 1;
        bh = GetSystemMetrics((SM_CYBORDER) as i32) + 1;
        InflateRect(core::ptr::addr_of_mut!((*es).format_rect), -bw, 0);
        if (*es).format_rect.bottom - (*es).format_rect.top >= (*es).line_height + 2 * bh {
            InflateRect(core::ptr::addr_of_mut!((*es).format_rect), 0, -bh);
        }
    }

    (*es).format_rect.left += (*es).left_margin;
    (*es).format_rect.right -= (*es).right_margin;
    EDIT_AdjustFormatRect(es);
}

unsafe extern "C" fn EDIT_EM_CharFromPos(mut es: *mut EDITSTATE, mut x: INT, mut y: INT) -> LRESULT {
    let mut pt: POINT = POINT { x: 0, y: 0 };
    let mut rc: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut index: INT;

    pt.x = x;
    pt.y = y;
    GetClientRect((*es).hwndSelf, core::ptr::addr_of_mut!(rc));
    if PtInRect(core::ptr::addr_of_mut!(rc), pt) == 0 {
        return -(1 as LRESULT);
    }

    index = EDIT_CharFromPos(es, x, y, core::ptr::null_mut());
    return MAKELONG!(index, EDIT_EM_LineFromChar(es, index)) as LRESULT;
}

unsafe extern "C" fn EDIT_EM_FmtLines(mut es: *mut EDITSTATE, mut add_eol: BOOL) -> BOOL {
    (*es).flags = ((*es).flags as ::core::ffi::c_int & !EF_USE_SOFTBRK) as WORD;
    if add_eol != 0 {
        (*es).flags = ((*es).flags as ::core::ffi::c_int | EF_USE_SOFTBRK) as WORD;
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/win32/comctl32/edit.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"EDIT_EM_FmtLines\0" as *const u8 as *const ::core::ffi::c_char,
                2395 as ::core::ffi::c_int,
                b"soft break enabled, not implemented\n\0" as *const u8
                    as *const ::core::ffi::c_char,
            );
        }
    }
    return add_eol;
}

unsafe extern "C" fn EDIT_EM_GetHandle(es: *mut EDITSTATE) -> HLOCAL {
    if !(((*es).style & ES_MULTILINE) != 0) {
        return core::ptr::null_mut();
    }

    EDIT_UnlockBuffer(es, TRUE);

    /* The text buffer handle belongs to the app */
    (*es).hlocapp = (*es).hloc32W;

    return (*es).hlocapp;
}

unsafe extern "C" fn EDIT_EM_GetLine(es: *mut EDITSTATE, mut line: INT, dst: LPWSTR) -> INT {
    let mut line_len: INT;
    let mut dst_len: INT;
    let mut src: LPWSTR;
    let mut i: INT;

    if (*es).style & ES_MULTILINE as DWORD != 0 {
        if line >= (*es).line_count {
            return 0 as INT;
        }
    } else {
        line = 0 as INT;
    }

    i = EDIT_EM_LineIndex(es, line);
    src = (*es).text.offset(i as isize);
    line_len = EDIT_EM_LineLength(es, i);
    dst_len = *(dst as *mut WORD) as INT;

    if dst_len <= line_len {
        memcpy(
            dst as *mut core::ffi::c_void,
            src as *const core::ffi::c_void,
            (dst_len as usize).wrapping_mul(core::mem::size_of::<WCHAR>()),
        );
        return dst_len;
    } else {
        memcpy(
            dst as *mut core::ffi::c_void,
            src as *const core::ffi::c_void,
            (line_len as usize).wrapping_mul(core::mem::size_of::<WCHAR>()),
        );
        *dst.offset(line_len as isize) = 0 as WCHAR;
        return line_len;
    }
}

unsafe extern "C" fn EDIT_EM_GetSel(es: *const EDITSTATE, start: PUINT, end: PUINT) -> LRESULT {
    let mut s: UINT = (*es).selection_start as UINT;
    let mut e: UINT = (*es).selection_end as UINT;
    // ORDER_UINT(s, e) expanded inline: if ((UINT)e < (UINT)s) SWAP_UINT32(s, e);
    if e < s {
        let temp: UINT = s;
        s = e;
        e = temp;
    }
    if !start.is_null() {
        *start = s;
    }
    if !end.is_null() {
        *end = e;
    }
    return MAKELONG!(s as WORD, e as WORD) as LRESULT;
}

unsafe extern "C" fn EDIT_EM_ReplaceSel(mut es: *mut EDITSTATE, mut can_undo: BOOL, mut lpsz_replace: *const WCHAR, mut strl: UINT, mut send_update: BOOL, mut honor_limit: BOOL) {
    let mut tl: UINT = get_text_length(es);
    let mut utl: UINT = 0;
    let mut s: UINT = 0;
    let mut e: UINT = 0;
    let mut i: UINT = 0;
    let mut size: UINT = 0;
    let mut p: LPWSTR = ::core::ptr::null_mut::<WCHAR>();
    let mut hrgn: HRGN = 0 as HRGN;
    let mut buf: LPWSTR = ::core::ptr::null_mut::<WCHAR>();
    let mut bufl: UINT = 0;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/edit.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"EDIT_EM_ReplaceSel\0" as *const u8 as *const ::core::ffi::c_char,
            2506 as ::core::ffi::c_int,
            b"%s, can_undo %d, send_update %d\n\0" as *const u8 as *const ::core::ffi::c_char,
            debugstr_wn(lpsz_replace, strl as ::core::ffi::c_int),
            can_undo,
            send_update,
        );
    }
    s = (*es).selection_start as UINT;
    e = (*es).selection_end as UINT;
    EDIT_InvalidateUniscribeData(es);
    if s == e && strl == 0 {
        return;
    }
    if e < s {
        let mut temp: UINT = s;
        s = e;
        e = temp;
    }
    size = tl.wrapping_sub(e.wrapping_sub(s)).wrapping_add(strl);
    if size == 0 {
        (*es).text_width = 0 as INT;
    }
    if honor_limit != 0 && size > (*es).buffer_limit {
        if notify_parent(es, (EN_MAXTEXT) as INT) == 0 {
            return;
        }
        if (*es).buffer_limit < tl.wrapping_sub(e.wrapping_sub(s)) {
            strl = 0 as UINT;
        } else {
            strl = if strl
                < (*es)
                    .buffer_limit
                    .wrapping_sub(tl.wrapping_sub(e.wrapping_sub(s)))
            {
                strl
            } else {
                (*es)
                    .buffer_limit
                    .wrapping_sub(tl.wrapping_sub(e.wrapping_sub(s)))
            };
        }
    }
    if EDIT_MakeFit(es, tl.wrapping_sub(e.wrapping_sub(s)).wrapping_add(strl)) == 0 {
        return;
    }
    if e != s {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
        {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl_0,
                __dbch_0,
                b"dll/win32/comctl32/edit.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"EDIT_EM_ReplaceSel\0" as *const u8 as *const ::core::ffi::c_char,
                2538 as ::core::ffi::c_int,
                b"deleting stuff.\n\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        bufl = e.wrapping_sub(s);
        buf = heap_alloc(
            (bufl.wrapping_add(1 as UINT) as SIZE_T)
                .wrapping_mul(::core::mem::size_of::<WCHAR>() as SIZE_T),
        ) as LPWSTR;
        if buf.is_null() {
            return;
        }
        memcpy(
            buf as *mut ::core::ffi::c_void,
            (*es).text.offset(s as isize) as *const ::core::ffi::c_void,
            (bufl as size_t).wrapping_mul(::core::mem::size_of::<WCHAR>() as size_t),
        );
        *buf.offset(bufl as isize) = 0 as WCHAR;
        lstrcpyW(
            (*es).text.offset(s as isize),
            (*es).text.offset(e as isize) as LPCWSTR,
        );
        text_buffer_changed(es);
    }
    if strl != 0 {
        tl = get_text_length(es);
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
        {
            let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_1: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl_1,
                __dbch_1,
                b"dll/win32/comctl32/edit.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"EDIT_EM_ReplaceSel\0" as *const u8 as *const ::core::ffi::c_char,
                2551 as ::core::ffi::c_int,
                b"inserting stuff (tl %d, strl %d, selstart %d (%s), text %s)\n\0"
                    as *const u8 as *const ::core::ffi::c_char,
                tl,
                strl,
                s,
                debugstr_w((*es).text.offset(s as isize) as *const WCHAR),
                debugstr_w((*es).text as *const WCHAR),
            );
        }
        p = (*es).text.offset(tl as isize);
        while p >= (*es).text.offset(s as isize) {
            *p.offset(strl as isize) = *p.offset(0 as isize);
            p = p.offset(-1);
        }
        i = 0;
        p = (*es).text.offset(s as isize);
        while i < strl {
            *p.offset(i as isize) = *lpsz_replace.offset(i as isize);
            i = i.wrapping_add(1);
        }
        if (*es).style & ES_UPPERCASE as DWORD != 0 {
            CharUpperBuffW(p, strl as DWORD);
        } else if (*es).style & ES_LOWERCASE as DWORD != 0 {
            CharLowerBuffW(p, strl as DWORD);
        }
        text_buffer_changed(es);
    }
    if (*es).style & ES_MULTILINE as DWORD != 0 {
        let mut st: INT = if (*es).selection_start < (*es).selection_end {
            (*es).selection_start
        } else {
            (*es).selection_end
        };
        let mut vlc: INT = get_vertical_line_count(es);
        hrgn = CreateRectRgn(
            0,
            0,
            0,
            0,
        );
        EDIT_BuildLineDefs_ML(
            es,
            st,
            (st as UINT).wrapping_add(strl) as INT,
            strl.wrapping_sub(abs((*es).selection_end as ::core::ffi::c_int
                - (*es).selection_start as ::core::ffi::c_int) as UINT) as INT,
            hrgn,
        );
        if honor_limit != 0 && (*es).style & ES_AUTOVSCROLL as DWORD == 0 && (*es).line_count > vlc {
            if strl != 0 {
                lstrcpyW(
                    (*es).text.offset(e as isize),
                    (*es).text.offset(e as isize).offset(strl as isize) as LPCWSTR,
                );
            }
            if e != s {
                i = 0;
                p = (*es).text;
                while i < e.wrapping_sub(s) {
                    *p.offset(i.wrapping_add(s) as isize) = *buf.offset(i as isize);
                    i = i.wrapping_add(1);
                }
            }
            text_buffer_changed(es);
            EDIT_BuildLineDefs_ML(
                es,
                s as INT,
                e as INT,
                (abs((*es).selection_end as ::core::ffi::c_int
                    - (*es).selection_start as ::core::ffi::c_int) as UINT)
                    .wrapping_sub(strl) as INT,
                hrgn,
            );
            strl = 0 as UINT;
            e = s;
            SetRectRgn(
                hrgn,
                0,
                0,
                0,
                0,
            );
            if notify_parent(es, (EN_MAXTEXT) as INT) == 0 {
                return;
            }
        }
    } else {
        let mut fw: INT = (*es).format_rect.right as INT - (*es).format_rect.left as INT;
        EDIT_InvalidateUniscribeData(es);
        EDIT_CalcLineWidth_SL(es);
        if honor_limit != 0 && (*es).style & ES_AUTOHSCROLL as DWORD == 0 && (*es).text_width > fw {
            while (*es).text_width > fw && s.wrapping_add(strl) > 0 as UINT {
                lstrcpyW(
                    (*es)
                        .text
                        .offset(s as isize)
                        .offset(strl as isize)
                        .offset(-(1 as isize)),
                    (*es).text.offset(s as isize).offset(strl as isize) as LPCWSTR,
                );
                strl = strl.wrapping_sub(1);
                (*es).text_length = (-(1 as INT)) as UINT;
                EDIT_InvalidateUniscribeData(es);
                EDIT_CalcLineWidth_SL(es);
            }
            text_buffer_changed(es);
            if notify_parent(es, (EN_MAXTEXT) as INT) == 0 {
                return;
            }
        }
    }
    if e != s {
        if can_undo != 0 {
            utl = lstrlenW((*es).undo_text as LPCWSTR) as UINT;
            if (*es).undo_insert_count == 0
                && (*(*es).undo_text as ::core::ffi::c_int != 0 && s == (*es).undo_position)
            {
                EDIT_MakeUndoFit(es, utl.wrapping_add(e).wrapping_sub(s));
                memcpy(
                    (*es).undo_text.offset(utl as isize) as *mut ::core::ffi::c_void,
                    buf as *const ::core::ffi::c_void,
                    (e.wrapping_sub(s) as size_t)
                        .wrapping_mul(::core::mem::size_of::<WCHAR>() as size_t),
                );
                *(*es)
                    .undo_text
                    .offset(utl as isize)
                    .offset(e.wrapping_sub(s) as isize) = 0 as WCHAR;
            } else if (*es).undo_insert_count == 0
                && (*(*es).undo_text as ::core::ffi::c_int != 0 && e == (*es).undo_position)
            {
                EDIT_MakeUndoFit(es, utl.wrapping_add(e).wrapping_sub(s));
                p = (*es).undo_text.offset(utl as isize);
                while p >= (*es).undo_text {
                    *p.offset(e.wrapping_sub(s) as isize) =
                        *p.offset(0 as isize);
                    p = p.offset(-1);
                }
                i = 0;
                p = (*es).undo_text;
                while i < e.wrapping_sub(s) {
                    *p.offset(i as isize) = *buf.offset(i as isize);
                    i = i.wrapping_add(1);
                }
                (*es).undo_position = s;
            } else {
                EDIT_MakeUndoFit(es, e.wrapping_sub(s));
                memcpy(
                    (*es).undo_text as *mut ::core::ffi::c_void,
                    buf as *const ::core::ffi::c_void,
                    (e.wrapping_sub(s) as size_t)
                        .wrapping_mul(::core::mem::size_of::<WCHAR>() as size_t),
                );
                *(*es).undo_text.offset(e.wrapping_sub(s) as isize) = 0 as WCHAR;
                (*es).undo_position = s;
            }
            (*es).undo_insert_count = 0 as INT;
        } else {
            EDIT_EM_EmptyUndoBuffer(es);
        }
    }
    if strl != 0 {
        if can_undo != 0 {
            if s == (*es).undo_position
                || (*es).undo_insert_count != 0
                    && s == (*es)
                        .undo_position
                        .wrapping_add((*es).undo_insert_count as UINT)
            {
                (*es).undo_insert_count =
                    ((*es).undo_insert_count as UINT).wrapping_add(strl) as INT as INT;
            } else {
                (*es).undo_position = s;
                (*es).undo_insert_count = strl as INT;
                *(*es).undo_text = '\0' as i32 as WCHAR;
            }
        } else {
            EDIT_EM_EmptyUndoBuffer(es);
        }
    }
    heap_free(buf as *mut ::core::ffi::c_void);
    s = s.wrapping_add(strl);
    if (*es).style & (ES_RIGHT | ES_CENTER) as DWORD != 0 {
        let mut delta: INT = strl.wrapping_sub(abs((*es).selection_end as ::core::ffi::c_int
            - (*es).selection_start as ::core::ffi::c_int) as UINT) as INT;
        if delta < 0 as INT && (*es).x_offset != 0 {
            if abs(delta as ::core::ffi::c_int) > (*es).x_offset {
                (*es).x_offset = 0 as INT;
            } else {
                (*es).x_offset += delta;
            }
        }
    }
    EDIT_EM_SetSel(es, s, s, FALSE);
    (*es).flags = ((*es).flags as ::core::ffi::c_int | EF_MODIFIED as ::core::ffi::c_int) as WORD;
    if send_update != 0 {
        (*es).flags = ((*es).flags as ::core::ffi::c_int | EF_UPDATE as ::core::ffi::c_int) as WORD;
    }
    if !hrgn.is_null() {
        EDIT_UpdateTextRegion(es, hrgn, TRUE);
        DeleteObject(hrgn as HGDIOBJ);
    } else {
        EDIT_UpdateText(es, ::core::ptr::null::<RECT>(), TRUE);
    }
    EDIT_EM_ScrollCaret(es);
    EDIT_UpdateScrollInfo(es);
    if send_update != 0 || (*es).flags as ::core::ffi::c_int & EF_UPDATE as ::core::ffi::c_int != 0 {
        (*es).flags = ((*es).flags as ::core::ffi::c_int & !(EF_UPDATE as ::core::ffi::c_int)) as WORD;
        if notify_parent(es, (EN_CHANGE) as INT) == 0 {
            return;
        }
    }
    EDIT_InvalidateUniscribeData(es);
}

unsafe extern "C" fn EDIT_EM_SetHandle(mut es: *mut EDITSTATE, mut hloc: HLOCAL) {
    if (*es).style & ES_MULTILINE as DWORD == 0 {
        return;
    }
    if hloc.is_null() {
        return;
    }
    if !(*es).text.is_null() {
        EDIT_UnlockBuffer(es, TRUE);
    }
    (*es).hloc32W = hloc;
    (*es).buffer_size = (LocalSize((*es).hloc32W) as usize)
        .wrapping_div(::core::mem::size_of::<WCHAR>() as usize)
        .wrapping_sub(1 as usize) as UINT;
    (*es).hlocapp = core::ptr::null_mut();
    EDIT_LockBuffer(es);
    text_buffer_changed(es);
    (*es).y_offset = 0 as INT;
    (*es).x_offset = (*es).y_offset;
    (*es).selection_end = 0 as INT;
    (*es).selection_start = (*es).selection_end;
    EDIT_EM_EmptyUndoBuffer(es);
    (*es).flags = ((*es).flags as ::core::ffi::c_int & !EF_MODIFIED) as WORD;
    (*es).flags = ((*es).flags as ::core::ffi::c_int & !EF_UPDATE) as WORD;
    EDIT_BuildLineDefs_ML(
        es,
        0 as INT,
        get_text_length(es) as INT,
        0 as INT,
        core::ptr::null_mut(),
    );
    EDIT_UpdateText(es, core::ptr::null(), TRUE);
    EDIT_EM_ScrollCaret(es);
    EDIT_UpdateScrollInfo(es);
}

unsafe extern "C" fn EDIT_EM_SetLimitText(mut es: *mut EDITSTATE, mut limit: UINT) {
    if limit == 0 {
        limit = !(0 as UINT);
    }
    if ((*es).style & (ES_MULTILINE as DWORD)) == 0 {
        limit = min(limit, 0x7ffffffe as UINT);
    }
    (*es).buffer_limit = limit;
}

unsafe extern "C" fn is_cjk(dc: HDC) -> BOOL {
    const FS_DBCS_MASK: DWORD = (FS_JISJAPAN | FS_CHINESESIMP | FS_WANSUNG | FS_CHINESETRAD | FS_JOHAB) as DWORD;
    let mut fs: FONTSIGNATURE = core::mem::zeroed();

    match GdiGetCodePage(dc) {
        932 | 936 | 949 | 950 | 1361 => {
            return TRUE;
        }
        _ => {
            return (GetTextCharsetInfo(dc, core::ptr::addr_of_mut!(fs), 0) != (DEFAULT_CHARSET as i32)
                && (fs.fsCsb[0] & FS_DBCS_MASK) != 0) as BOOL;
        }
    }
}

unsafe extern "C" fn get_cjk_fontinfo_margin(width: i32, side_bearing: i32) -> i32 {
    let mut margin: i32;
    if side_bearing < 0 {
        margin = min(-side_bearing, width / 2);
    } else {
        margin = 0;
    }
    return margin;
}

unsafe extern "C" fn EDIT_EM_SetMargins(
    mut es: *mut EDITSTATE,
    mut action: INT,
    mut left: WORD,
    mut right: WORD,
    mut repaint: BOOL,
) {
    let mut tm: TEXTMETRICW = tagTEXTMETRICW {
        tmHeight: 0,
        tmAscent: 0,
        tmDescent: 0,
        tmInternalLeading: 0,
        tmExternalLeading: 0,
        tmAveCharWidth: 0,
        tmMaxCharWidth: 0,
        tmWeight: 0,
        tmOverhang: 0,
        tmDigitizedAspectX: 0,
        tmDigitizedAspectY: 0,
        tmFirstChar: 0,
        tmLastChar: 0,
        tmDefaultChar: 0,
        tmBreakChar: 0,
        tmItalic: 0,
        tmUnderlined: 0,
        tmStruckOut: 0,
        tmPitchAndFamily: 0,
        tmCharSet: 0,
    };
    let mut default_left_margin: INT = 0 as INT;
    let mut default_right_margin: INT = 0 as INT;
    if !(*es).font.is_null()
        && ((left as ::core::ffi::c_int) as u32 == EC_USEFONTINFO
            || (right as ::core::ffi::c_int) as u32 == EC_USEFONTINFO)
    {
        let mut dc: HDC = GetDC((*es).hwndSelf);
        let mut old_font: HFONT = SelectObject(dc, (*es).font as HGDIOBJ) as HFONT;
        let mut width: LONG =
            GdiGetCharDimensions(dc, core::ptr::addr_of_mut!(tm), ::core::ptr::null_mut::<LONG>());
        let mut rc_width: LONG = 0;
        let mut rc: RECT = tagRECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        if tm.tmPitchAndFamily as u32
            & (TMPF_VECTOR | TMPF_TRUETYPE)
            != 0
        {
            let mut width_info: char_width_info = char_width_info {
                min_lsb: 0,
                min_rsb: 0,
                unknown: 0,
            };
            if is_cjk(dc) != 0
                && GetCharWidthInfo(dc, core::ptr::addr_of_mut!(width_info)) != 0
            {
                default_left_margin = get_cjk_fontinfo_margin(
                    width as ::core::ffi::c_int,
                    width_info.min_lsb as ::core::ffi::c_int,
                ) as INT;
                default_right_margin = get_cjk_fontinfo_margin(
                    width as ::core::ffi::c_int,
                    width_info.min_rsb as ::core::ffi::c_int,
                ) as INT;
            } else {
                default_left_margin = (width as ::core::ffi::c_int
                    / 2 as ::core::ffi::c_int)
                    as INT;
                default_right_margin = (width as ::core::ffi::c_int
                    / 2 as ::core::ffi::c_int)
                    as INT;
            }
            GetClientRect((*es).hwndSelf, core::ptr::addr_of_mut!(rc));
            rc_width = (if IsRectEmpty(core::ptr::addr_of_mut!(rc) as LPCRECT) == 0 {
                rc.right as ::core::ffi::c_int - rc.left as ::core::ffi::c_int
            } else {
                80 as ::core::ffi::c_int
            }) as LONG;
            if rc_width
                < default_left_margin as ::core::ffi::c_int
                    + default_right_margin as ::core::ffi::c_int
                    + width as ::core::ffi::c_int * 2 as ::core::ffi::c_int
            {
                default_left_margin = (*es).left_margin;
                default_right_margin = (*es).right_margin;
            }
        }
        SelectObject(dc, old_font as HGDIOBJ);
        ReleaseDC((*es).hwndSelf, dc);
    }
    if action as u32 & EC_LEFTMARGIN != 0 {
        (*es).format_rect.left -= (*es).left_margin as ::core::ffi::c_int;
        if (left as ::core::ffi::c_int) as u32 != EC_USEFONTINFO {
            (*es).left_margin = left as INT;
        } else {
            (*es).left_margin = default_left_margin;
        }
        (*es).format_rect.left += (*es).left_margin as ::core::ffi::c_int;
    }
    if action as u32 & EC_RIGHTMARGIN != 0 {
        (*es).format_rect.right += (*es).right_margin as ::core::ffi::c_int;
        if (right as ::core::ffi::c_int) as u32 != EC_USEFONTINFO {
            (*es).right_margin = right as INT;
        } else {
            (*es).right_margin = default_right_margin;
        }
        (*es).format_rect.right -= (*es).right_margin as ::core::ffi::c_int;
    }
    if action as u32
        & (EC_LEFTMARGIN | EC_RIGHTMARGIN)
        != 0
    {
        EDIT_AdjustFormatRect(es);
        if repaint != 0 {
            EDIT_UpdateText(es, ::core::ptr::null::<RECT>(), TRUE);
        }
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
            b"dll/win32/comctl32/edit.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"EDIT_EM_SetMargins\0" as *const u8 as *const ::core::ffi::c_char,
            2863 as ::core::ffi::c_int,
            b"left=%d, right=%d\n\0" as *const u8 as *const ::core::ffi::c_char,
            (*es).left_margin,
            (*es).right_margin,
        );
    }
}

unsafe extern "C" fn EDIT_EM_SetPasswordChar(es: *mut EDITSTATE, c: WCHAR) {
    let mut style: LONG = 0;
    if ((*es).style & (ES_MULTILINE as DWORD)) != 0 {
        return;
    }
    if (*es).password_char == c {
        return;
    }
    style = GetWindowLongW((*es).hwndSelf, GWL_STYLE);
    (*es).password_char = c;
    if c != 0 {
        SetWindowLongW((*es).hwndSelf, GWL_STYLE, style | (ES_PASSWORD as LONG));
        (*es).style |= ES_PASSWORD as DWORD;
    } else {
        SetWindowLongW((*es).hwndSelf, GWL_STYLE, style & !(ES_PASSWORD as LONG));
        (*es).style &= !(ES_PASSWORD as DWORD);
    }
    EDIT_InvalidateUniscribeData(es);
    EDIT_UpdateText(es, core::ptr::null::<RECT>(), TRUE);
}

unsafe extern "C" fn EDIT_EM_SetTabStops(
    mut es: *mut EDITSTATE,
    mut count: INT,
    mut tabs: *const INT,
) -> BOOL {
    if (*es).style & (ES_MULTILINE as DWORD) == 0 {
        return FALSE as BOOL;
    }
    heap_free((*es).tabs as *mut core::ffi::c_void);
    (*es).tabs_count = count;
    if count == 0 {
        (*es).tabs = core::ptr::null_mut::<INT>();
    } else {
        (*es).tabs = heap_alloc(
            (count as SIZE_T).wrapping_mul(core::mem::size_of::<INT>() as SIZE_T),
        ) as LPINT;
        memcpy(
            (*es).tabs as *mut core::ffi::c_void,
            tabs as *const core::ffi::c_void,
            (count as usize).wrapping_mul(core::mem::size_of::<INT>()),
        );
    }
    EDIT_InvalidateUniscribeData(es);
    return TRUE as BOOL;
}

unsafe extern "C" fn EDIT_EM_SetWordBreakProc(mut es: *mut EDITSTATE, mut wbp: EDITWORDBREAKPROCW) {
    if (*es).word_break_proc == wbp {
        return;
    }
    (*es).word_break_proc = wbp;
    if ((*es).style & (ES_MULTILINE as DWORD)) != 0 && ((*es).style & (ES_AUTOHSCROLL as DWORD)) == 0 {
        EDIT_BuildLineDefs_ML(
            es,
            0 as INT,
            get_text_length(es) as INT,
            0 as INT,
            core::ptr::null_mut::<HRGN__>(),
        );
        EDIT_UpdateText(es, core::ptr::null::<RECT>(), TRUE as BOOL);
    }
}

unsafe extern "C" fn EDIT_EM_Undo(es: *mut EDITSTATE) -> BOOL {
    let mut ulength: INT = 0;
    let mut utext: LPWSTR = core::ptr::null_mut::<WCHAR>();

    /* As per MSDN spec, for a single-line edit control,
       the return value is always TRUE */
    if ((*es).style & (ES_READONLY as DWORD)) != 0 {
        return (((*es).style & (ES_MULTILINE as DWORD)) == 0) as BOOL;
    }

    ulength = lstrlenW((*es).undo_text as LPCWSTR) as INT;

    utext = heap_alloc(
        ((ulength + 1) as SIZE_T)
            .wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
    ) as LPWSTR;

    lstrcpyW(utext, (*es).undo_text as LPCWSTR);

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/edit.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"EDIT_EM_Undo\0" as *const u8 as *const ::core::ffi::c_char,
            2959 as ::core::ffi::c_int,
            b"before UNDO:insertion length = %d, deletion buffer = %s\n\0" as *const u8
                as *const ::core::ffi::c_char,
            (*es).undo_insert_count,
            debugstr_w(utext as *const WCHAR),
        );
    }

    EDIT_EM_SetSel(
        es,
        (*es).undo_position,
        (*es).undo_position.wrapping_add((*es).undo_insert_count as UINT),
        FALSE,
    );
    EDIT_EM_EmptyUndoBuffer(es);
    EDIT_EM_ReplaceSel(
        es,
        TRUE,
        utext as *const WCHAR,
        ulength as UINT,
        TRUE,
        TRUE,
    );
    EDIT_EM_SetSel(
        es,
        (*es).undo_position,
        (*es).undo_position.wrapping_add((*es).undo_insert_count as UINT),
        FALSE,
    );
    /* send the notification after the selection start and end are set */
    if notify_parent(es, (EN_CHANGE) as INT) == 0 {
        return TRUE;
    }
    EDIT_EM_ScrollCaret(es);
    heap_free(utext as *mut ::core::ffi::c_void);

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl_0,
            __dbch_0,
            b"dll/win32/comctl32/edit.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"EDIT_EM_Undo\0" as *const u8 as *const ::core::ffi::c_char,
            2971 as ::core::ffi::c_int,
            b"after UNDO:insertion length = %d, deletion buffer = %s\n\0" as *const u8
                as *const ::core::ffi::c_char,
            (*es).undo_insert_count,
            debugstr_w((*es).undo_text as *const WCHAR),
        );
    }
    return TRUE;
}

unsafe extern "C" fn EDIT_IsInsideDialog(es: *mut EDITSTATE) -> BOOL {
    return ((*es).flags as BOOL & EF_DIALOGMODE);
}

unsafe extern "C" fn EDIT_WM_Paste(mut es: *mut EDITSTATE) {
    let mut hsrc: HGLOBAL = ::core::ptr::null_mut::<::core::ffi::c_void>();
    let mut src: LPWSTR = ::core::ptr::null_mut::<WCHAR>();
    let mut ptr: LPWSTR = ::core::ptr::null_mut::<WCHAR>();
    let mut len: ::core::ffi::c_int = 0;

    /* Protect read-only edit control from modification */
    if (*es).style & ES_READONLY as DWORD != 0 {
        return;
    }

    OpenClipboard((*es).hwndSelf);
    hsrc = GetClipboardData(CF_UNICODETEXT as UINT) as HGLOBAL;
    if !hsrc.is_null() {
        src = GlobalLock(hsrc) as LPWSTR;
        len = lstrlenW(src as LPCWSTR) as ::core::ffi::c_int;
        /* Protect single-line edit against pasting new line character */
        if (*es).style & ES_MULTILINE as DWORD == 0 && {
            ptr = wcschr(src as *const WCHAR, '\n' as WCHAR) as LPWSTR;
            !ptr.is_null()
        } {
            len = ptr.offset_from(src) as ::core::ffi::c_longlong as ::core::ffi::c_int;
            if len != 0
                && *src.offset((len - 1 as ::core::ffi::c_int) as isize)
                    as ::core::ffi::c_int
                    == '\r' as i32
            {
                len -= 1;
            }
        }
        EDIT_EM_ReplaceSel(
            es,
            TRUE as BOOL,
            src as *const WCHAR,
            len as UINT,
            TRUE as BOOL,
            TRUE as BOOL,
        );
        GlobalUnlock(hsrc);
    } else if (*es).style & ES_PASSWORD as DWORD != 0 {
        /* clear selected text in password edit box even with empty clipboard */
        EDIT_EM_ReplaceSel(
            es,
            TRUE as BOOL,
            ::core::ptr::null::<WCHAR>(),
            0 as UINT,
            TRUE as BOOL,
            TRUE as BOOL,
        );
    }
    CloseClipboard();
}

unsafe extern "C" fn EDIT_WM_Copy(es: *mut EDITSTATE) {
    let s: INT = min((*es).selection_start, (*es).selection_end);
    let e: INT = max((*es).selection_start, (*es).selection_end);
    let mut hdst: HGLOBAL = core::ptr::null_mut::<core::ffi::c_void>();
    let mut dst: LPWSTR = core::ptr::null_mut::<WCHAR>();
    let mut len: DWORD = 0;

    if e == s {
        return;
    }

    len = (e - s) as DWORD;
    hdst = GlobalAlloc(
        (GMEM_MOVEABLE | GMEM_DDESHARE) as UINT,
        ((len + 1) as SIZE_T)
            .wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
    );
    dst = GlobalLock(hdst) as LPWSTR;
    memcpy(
        dst as *mut core::ffi::c_void,
        (*es).text.offset(s as isize) as *const core::ffi::c_void,
        (len as size_t).wrapping_mul(core::mem::size_of::<WCHAR>() as size_t),
    );
    *dst.offset(len as isize) = 0; /* ensure 0 termination */

    if ((*__wine_dbch___default).flags as core::ffi::c_int
        & ((1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int))
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/edit.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"EDIT_WM_Copy\0" as *const u8 as *const core::ffi::c_char,
            3045 as core::ffi::c_int,
            b"%s\n\0" as *const u8 as *const core::ffi::c_char,
            debugstr_w(dst as *const WCHAR),
        );
    }

    GlobalUnlock(hdst);
    OpenClipboard((*es).hwndSelf);
    EmptyClipboard();
    SetClipboardData(CF_UNICODETEXT, hdst as HANDLE);
    CloseClipboard();
}

unsafe extern "C" fn EDIT_WM_Clear(es: *mut EDITSTATE) {
    /* Protect read-only edit control from modification */
    if ((*es).style & (ES_READONLY as DWORD)) != 0 {
        return;
    }

    EDIT_EM_ReplaceSel(
        es,
        TRUE as BOOL,
        core::ptr::null::<WCHAR>(),
        0 as UINT,
        TRUE as BOOL,
        TRUE as BOOL,
    );
}

unsafe extern "C" fn EDIT_WM_Cut(es: *mut EDITSTATE) {
    EDIT_WM_Copy(es);
    EDIT_WM_Clear(es);
}

unsafe extern "C" fn EDIT_WM_Char(mut es: *mut EDITSTATE, mut c: WCHAR) -> LRESULT {
    let mut control: BOOL = 0;
    if (*es).bCaptureState != 0 {
        return 0;
    }
    control = (GetKeyState((VK_CONTROL) as i32) as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) as BOOL;

    // at: 0 = plain break out of switch, 1 = fall through to the '\n' tail
    let mut at: u8 = 0;
    match (c as ::core::ffi::c_int) as u32 {
        13 => {
            at = 1;
            if ((*es).style & ES_MULTILINE) != 0 && ((*es).style & ES_WANTRETURN) == 0 {
                if EDIT_IsInsideDialog(es) != 0 {
                    at = 0;
                }
            }
        }
        10 => {
            at = 1;
        }
        9 => {
            if ((*es).style & ES_MULTILINE) != 0 && ((*es).style & ES_READONLY) == 0 {
                static tabW: [WCHAR; 1] = [0x0009u16];
                if EDIT_IsInsideDialog(es) == 0 {
                    EDIT_EM_ReplaceSel(
                        es,
                        TRUE,
                        core::ptr::addr_of!(tabW) as *const WCHAR,
                        1,
                        TRUE,
                        TRUE,
                    );
                }
            }
        }
        VK_BACK => {
            if ((*es).style & ES_READONLY) == 0 && control == 0 {
                if (*es).selection_start != (*es).selection_end {
                    EDIT_WM_Clear(es);
                } else {
                    EDIT_EM_SetSel(
                        es,
                        (-(1 as ::core::ffi::c_int)) as UINT,
                        0,
                        FALSE,
                    );
                    EDIT_MoveBackward(es, TRUE);
                    EDIT_WM_Clear(es);
                }
            }
        }
        0x03 => {
            if ((*es).style & ES_PASSWORD) == 0 {
                SendMessageW(
                    (*es).hwndSelf,
                    WM_COPY,
                    0,
                    0,
                );
            }
        }
        0x16 => {
            if ((*es).style & ES_READONLY) == 0 {
                SendMessageW(
                    (*es).hwndSelf,
                    WM_PASTE,
                    0,
                    0,
                );
            }
        }
        0x18 => {
            if !(((*es).style & ES_READONLY) != 0 || ((*es).style & ES_PASSWORD) != 0) {
                SendMessageW(
                    (*es).hwndSelf,
                    WM_CUT,
                    0,
                    0,
                );
            }
        }
        0x1A => {
            if ((*es).style & ES_READONLY) == 0 {
                SendMessageW(
                    (*es).hwndSelf,
                    WM_UNDO,
                    0,
                    0,
                );
            }
        }
        _ => {
            if ((*es).style & ES_NUMBER) != 0
                && !((c as ::core::ffi::c_int) >= ('0' as ::core::ffi::c_int) && (c as ::core::ffi::c_int) <= ('9' as ::core::ffi::c_int))
            {
                // break
            } else {
                if ((*es).style & ES_READONLY) == 0
                    && ((c as ::core::ffi::c_int) >= (' ' as ::core::ffi::c_int))
                    && ((c as ::core::ffi::c_int) != 127)
                {
                    EDIT_EM_ReplaceSel(
                        es,
                        TRUE,
                        core::ptr::addr_of_mut!(c),
                        1,
                        TRUE,
                        TRUE,
                    );
                }
            }
        }
    }

    if at == 1 {
        if ((*es).style & ES_MULTILINE) != 0 {
            if ((*es).style & ES_READONLY) != 0 {
                EDIT_MoveHome(es, FALSE, FALSE);
                EDIT_MoveDown_ML(es, FALSE);
            } else {
                static cr_lfW: [WCHAR; 2] = [0x000Du16, 0x000A];
                EDIT_EM_ReplaceSel(
                    es,
                    TRUE,
                    core::ptr::addr_of!(cr_lfW) as *const WCHAR,
                    2,
                    TRUE,
                    TRUE,
                );
            }
        }
    }
    return 1;
}

unsafe extern "C" fn EDIT_ContextMenuCommand(mut es: *mut EDITSTATE, mut id: UINT) {
    match id {
        EM_UNDO => {
            SendMessageW((*es).hwndSelf, WM_UNDO as UINT, 0 as WPARAM, 0 as LPARAM);
        }
        WM_CUT => {
            SendMessageW((*es).hwndSelf, WM_CUT as UINT, 0 as WPARAM, 0 as LPARAM);
        }
        WM_COPY => {
            SendMessageW((*es).hwndSelf, WM_COPY as UINT, 0 as WPARAM, 0 as LPARAM);
        }
        WM_PASTE => {
            SendMessageW((*es).hwndSelf, WM_PASTE as UINT, 0 as WPARAM, 0 as LPARAM);
        }
        WM_CLEAR => {
            SendMessageW((*es).hwndSelf, WM_CLEAR as UINT, 0 as WPARAM, 0 as LPARAM);
        }
        EM_SETSEL => {
            SendMessageW((*es).hwndSelf, EM_SETSEL as UINT, 0 as WPARAM, -(1 as ::core::ffi::c_int) as LPARAM);
        }
        _ => {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                != 0
            {
                let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
                ros_dbg_log(
                    __dbcl,
                    __dbch,
                    b"dll/win32/comctl32/edit.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"EDIT_ContextMenuCommand\0" as *const u8
                        as *const ::core::ffi::c_char,
                    3195 as ::core::ffi::c_int,
                    b"unknown menu item, please report\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                );
            }
        }
    }
}

unsafe extern "C" fn EDIT_WM_ContextMenu(es: *mut EDITSTATE, x: INT, y: INT) {
    let menu: HMENU = LoadMenuA(
        GetModuleHandleA(b"user32.dll\0" as *const u8 as LPCSTR) as HINSTANCE,
        b"EDITMENU\0" as *const u8 as LPCSTR,
    );
    let popup: HMENU = GetSubMenu(menu, 0);
    let mut start: UINT = (*es).selection_start as UINT;
    let mut end: UINT = (*es).selection_end as UINT;
    let mut cmd: UINT;
    let mut pt: POINT = POINT { x: 0, y: 0 };

    if end < start {
        let temp: UINT = start;
        start = end;
        end = temp;
    }

    /* undo */
    EnableMenuItem(
        popup,
        0,
        MF_BYPOSITION
            | (if EDIT_EM_CanUndo(es) != 0 && ((*es).style & ES_READONLY) == 0 {
                MF_ENABLED
            } else {
                MF_GRAYED
            }),
    );
    /* cut */
    EnableMenuItem(
        popup,
        2,
        MF_BYPOSITION
            | (if end.wrapping_sub(start) != 0
                && ((*es).style & ES_PASSWORD) == 0
                && ((*es).style & ES_READONLY) == 0
            {
                MF_ENABLED
            } else {
                MF_GRAYED
            }),
    );
    /* copy */
    EnableMenuItem(
        popup,
        3,
        MF_BYPOSITION
            | (if end.wrapping_sub(start) != 0 && ((*es).style & ES_PASSWORD) == 0 {
                MF_ENABLED
            } else {
                MF_GRAYED
            }),
    );
    /* paste */
    EnableMenuItem(
        popup,
        4,
        MF_BYPOSITION
            | (if IsClipboardFormatAvailable(CF_UNICODETEXT as UINT) != 0
                && ((*es).style & ES_READONLY) == 0
            {
                MF_ENABLED
            } else {
                MF_GRAYED
            }),
    );
    /* delete */
    EnableMenuItem(
        popup,
        5,
        MF_BYPOSITION
            | (if end.wrapping_sub(start) != 0 && ((*es).style & ES_READONLY) == 0 {
                MF_ENABLED
            } else {
                MF_GRAYED
            }),
    );
    /* select all */
    EnableMenuItem(
        popup,
        7,
        MF_BYPOSITION
            | (if start != 0 || end != get_text_length(es) {
                MF_ENABLED
            } else {
                MF_GRAYED
            }),
    );

    pt.x = x as LONG;
    pt.y = y as LONG;

    if pt.x == -(1 as LONG) && pt.y == -(1 as LONG) {
        /* passed via VK_APPS press/release */
        let mut rc: RECT = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };

        /* Windows places the menu at the edit's center in this case */
        GetClientRect((*es).hwndSelf, core::ptr::addr_of_mut!(rc));
        pt.x = (rc.left + (rc.right - rc.left) / 2) as LONG;
        pt.y = (rc.top + (rc.bottom - rc.top) / 2) as LONG;
        ClientToScreen((*es).hwndSelf, core::ptr::addr_of_mut!(pt));
    }

    if ((*es).flags as INT & EF_FOCUSED) == 0 {
        SetFocus((*es).hwndSelf);
    }

    cmd = TrackPopupMenu(
        popup,
        (TPM_LEFTALIGN
            | TPM_RIGHTBUTTON
            | TPM_RETURNCMD
            | TPM_NONOTIFY) as UINT,
        pt.x as INT,
        pt.y as INT,
        0 as INT,
        (*es).hwndSelf,
        core::ptr::null::<RECT>(),
    ) as UINT;

    if cmd != 0 {
        EDIT_ContextMenuCommand(es, cmd);
    }

    DestroyMenu(menu);
}

unsafe extern "C" fn EDIT_WM_GetText(es: *const EDITSTATE, count: INT, dst: LPWSTR) -> INT {
    if count == 0 {
        return 0;
    }
    lstrcpynW(dst, (*es).text as LPCWSTR, count);
    return lstrlenW(dst as LPCWSTR) as INT;
}

unsafe extern "C" fn EDIT_CheckCombo(es: *mut EDITSTATE, msg: UINT, mut key: INT) -> BOOL {
    let hLBox: HWND = (*es).hwndListBox;
    let mut hCombo: HWND = core::ptr::null_mut();
    let mut bDropped: BOOL = 0;
    let mut nEUI: ::core::ffi::c_int = 0;

    if hLBox.is_null() {
        return FALSE as BOOL;
    }

    hCombo = GetParent((*es).hwndSelf);
    bDropped = TRUE as BOOL;
    nEUI = 0 as ::core::ffi::c_int;

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/edit.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"EDIT_CheckCombo\0" as *const u8 as *const ::core::ffi::c_char,
            3301 as ::core::ffi::c_int,
            b"[%p]: handling msg %x (%x)\n\0" as *const u8 as *const ::core::ffi::c_char,
            (*es).hwndSelf,
            msg,
            key,
        );
    }

    if key == VK_UP as INT || key == VK_DOWN as INT {
        if SendMessageW(
            hCombo,
            CB_GETEXTENDEDUI as UINT,
            0 as WPARAM,
            0 as LPARAM,
        ) != 0
        {
            nEUI = 1 as ::core::ffi::c_int;
        }

        if msg == WM_KEYDOWN as UINT || nEUI != 0 {
            bDropped = SendMessageW(
                hCombo,
                CB_GETDROPPEDSTATE as UINT,
                0 as WPARAM,
                0 as LPARAM,
            ) as BOOL;
        }
    }

    if msg == WM_KEYDOWN as UINT {
        if bDropped == 0
            && nEUI != 0
            && (key == VK_UP as INT || key == VK_DOWN as INT)
        {
            /* make sure ComboLBox pops up */
            SendMessageW(
                hCombo,
                CB_SETEXTENDEDUI as UINT,
                FALSE as WPARAM,
                0 as LPARAM,
            );
            key = VK_F4 as INT;
            nEUI = 2 as ::core::ffi::c_int;
        }

        SendMessageW(
            hLBox,
            WM_KEYDOWN as UINT,
            key as WPARAM,
            0 as LPARAM,
        );
    } else if msg == WM_SYSKEYDOWN as UINT {
        /* Handle Alt+up/down arrows */
        if nEUI != 0 {
            SendMessageW(
                hCombo,
                CB_SHOWDROPDOWN as UINT,
                ((bDropped == 0) as ::core::ffi::c_int) as WPARAM,
                0 as LPARAM,
            );
        } else {
            SendMessageW(
                hLBox,
                WM_KEYDOWN as UINT,
                VK_F4 as WPARAM,
                0 as LPARAM,
            );
        }
    }

    if nEUI == 2 as ::core::ffi::c_int {
        SendMessageW(
            hCombo,
            CB_SETEXTENDEDUI as UINT,
            TRUE as WPARAM,
            0 as LPARAM,
        );
    }

    return TRUE as BOOL;
}

unsafe extern "C" fn EDIT_WM_KeyDown(mut es: *mut EDITSTATE, mut key: INT) -> LRESULT {
    let mut shift: BOOL;
    let mut control: BOOL;

    if (GetKeyState((VK_MENU) as i32) as INT & 0x8000) != 0 {
        return 0 as LRESULT;
    }

    shift = (GetKeyState((VK_SHIFT) as i32) as INT & 0x8000) as BOOL;
    control = (GetKeyState((VK_CONTROL) as i32) as INT & 0x8000) as BOOL;

    // blk: 0 = break (leave switch), 1 = VK_LEFT body, 2 = VK_RIGHT body
    // (C fall-through from the earlier case labels is expressed by setting blk)
    let mut blk: u8 = 0;

    match (key) as u32 {
        VK_F4 | VK_UP => {
            if EDIT_CheckCombo(es, WM_KEYDOWN as UINT, key) != 0
                || (key) as u32 == VK_F4
            {
                blk = 0;
            } else {
                blk = 1;
            }
        }
        VK_LEFT => {
            blk = 1;
        }
        VK_DOWN => {
            if EDIT_CheckCombo(es, WM_KEYDOWN as UINT, key) != 0 {
                blk = 0;
            } else {
                blk = 2;
            }
        }
        VK_RIGHT => {
            blk = 2;
        }
        VK_HOME => {
            EDIT_MoveHome(es, shift, control);
            blk = 0;
        }
        VK_END => {
            EDIT_MoveEnd(es, shift, control);
            blk = 0;
        }
        VK_PRIOR => {
            if ((*es).style & ES_MULTILINE as DWORD) != 0 {
                EDIT_MovePageUp_ML(es, shift);
            } else {
                EDIT_CheckCombo(es, WM_KEYDOWN as UINT, key);
            }
            blk = 0;
        }
        VK_NEXT => {
            if ((*es).style & ES_MULTILINE as DWORD) != 0 {
                EDIT_MovePageDown_ML(es, shift);
            } else {
                EDIT_CheckCombo(es, WM_KEYDOWN as UINT, key);
            }
            blk = 0;
        }
        VK_DELETE => {
            if ((*es).style & ES_READONLY as DWORD) == 0
                && !(shift != 0 && control != 0)
            {
                if (*es).selection_start != (*es).selection_end {
                    if shift != 0 {
                        EDIT_WM_Cut(es);
                    } else {
                        EDIT_WM_Clear(es);
                    }
                } else {
                    EDIT_EM_SetSel(
                        es,
                        !(0 as UINT),
                        0 as UINT,
                        FALSE as BOOL,
                    );
                    if shift != 0 {
                        /* delete character left of caret */
                        EDIT_MoveBackward(es, TRUE as BOOL);
                    } else if control != 0 {
                        /* delete to end of line */
                        EDIT_MoveEnd(
                            es,
                            TRUE as BOOL,
                            FALSE as BOOL,
                        );
                    } else {
                        /* delete character right of caret */
                        EDIT_MoveForward(es, TRUE as BOOL);
                    }
                    EDIT_WM_Clear(es);
                }
            }
            blk = 0;
        }
        VK_INSERT => {
            if shift != 0 {
                if ((*es).style & ES_READONLY as DWORD) == 0 {
                    EDIT_WM_Paste(es);
                }
            } else if control != 0 {
                EDIT_WM_Copy(es);
            }
            blk = 0;
        }
        VK_RETURN => {
            /* If the edit doesn't want the return send a message to the default object */
            if ((*es).style & ES_MULTILINE as DWORD) == 0
                || ((*es).style & ES_WANTRETURN as DWORD) == 0
            {
                let mut dw: DWORD = 0;

                if EDIT_IsInsideDialog(es) == 0 {
                    blk = 0;
                } else if control != 0 {
                    blk = 0;
                } else {
                    dw = SendMessageW(
                        (*es).hwndParent,
                        DM_GETDEFID as UINT,
                        0 as WPARAM,
                        0 as LPARAM,
                    ) as DWORD;
                    if ((HIWORD!(dw) as INT)) as u32 == DC_HASDEFID {
                        let hwDefCtrl: HWND = GetDlgItem(
                            (*es).hwndParent,
                            LOWORD!(dw) as INT,
                        );
                        if !hwDefCtrl.is_null() {
                            SendMessageW(
                                (*es).hwndParent,
                                WM_NEXTDLGCTL as UINT,
                                hwDefCtrl as WPARAM,
                                TRUE as LPARAM,
                            );
                            PostMessageW(
                                hwDefCtrl,
                                WM_KEYDOWN as UINT,
                                VK_RETURN as WPARAM,
                                0 as LPARAM,
                            );
                        }
                    }
                    blk = 0;
                }
            }
            blk = 0;
        }
        VK_ESCAPE => {
            if ((*es).style & ES_MULTILINE as DWORD) != 0
                && EDIT_IsInsideDialog(es) != 0
            {
                PostMessageW(
                    (*es).hwndParent,
                    WM_CLOSE as UINT,
                    0 as WPARAM,
                    0 as LPARAM,
                );
            }
            blk = 0;
        }
        VK_TAB => {
            if ((*es).style & ES_MULTILINE as DWORD) != 0
                && EDIT_IsInsideDialog(es) != 0
            {
                SendMessageW(
                    (*es).hwndParent,
                    WM_NEXTDLGCTL as UINT,
                    shift as WPARAM,
                    0 as LPARAM,
                );
            }
            blk = 0;
        }
        65 => {
            /* 'A' */
            if control != 0 {
                if EDIT_EM_SetSel(
                    es,
                    0 as UINT,
                    get_text_length(es),
                    FALSE as BOOL,
                ) != 0
                {
                    if notify_parent(es, EN_UPDATE as INT) == 0 {
                        blk = 0;
                    } else {
                        notify_parent(es, EN_CHANGE as INT);
                        EDIT_EM_ScrollCaret(es);
                        blk = 0;
                    }
                }
            }
            blk = 0;
        }
        _ => {
            blk = 0;
        }
    }

    match blk {
        1 => {
            if ((*es).style & ES_MULTILINE as DWORD) != 0 && (key) as u32 == VK_UP {
                EDIT_MoveUp_ML(es, shift);
            } else if control != 0 {
                EDIT_MoveWordBackward(es, shift);
            } else {
                EDIT_MoveBackward(es, shift);
            }
        }
        2 => {
            if ((*es).style & ES_MULTILINE as DWORD) != 0 && (key) as u32 == VK_DOWN {
                EDIT_MoveDown_ML(es, shift);
            } else if control != 0 {
                EDIT_MoveWordForward(es, shift);
            } else {
                EDIT_MoveForward(es, shift);
            }
        }
        _ => {}
    }

    return TRUE as LRESULT;
}

unsafe extern "C" fn EDIT_WM_KillFocus(mut theme: HTHEME, mut es: *mut EDITSTATE) -> LRESULT {
    let mut flags: UINT = RDW_INVALIDATE as UINT | RDW_UPDATENOW as UINT;
    let mut hwndSelf: HWND = (*es).hwndSelf;
    (*es).flags = (((*es).flags as ::core::ffi::c_int) & !(EF_FOCUSED as ::core::ffi::c_int)) as WORD;
    DestroyCaret();
    if ((*es).style & (ES_NOHIDESEL as DWORD)) == 0 {
        EDIT_InvalidateText(es, (*es).selection_start, (*es).selection_end);
    }
    if notify_parent(es, EN_KILLFOCUS as INT) == 0 {
        return 0 as LRESULT;
    }
    /* Throw away left over scroll when we lose focus */
    (*es).wheelDeltaRemainder = 0 as INT;

    if !theme.is_null() {
        flags |= RDW_FRAME as UINT;
    }

    RedrawWindow(
        hwndSelf,
        ::core::ptr::null::<RECT>(),
        ::core::ptr::null_mut::<HRGN__>(),
        flags,
    );
    return 0 as LRESULT;
}

unsafe extern "C" fn EDIT_WM_LButtonDblClk(es: *mut EDITSTATE) -> LRESULT {
    let mut s: INT = 0;
    let mut e: INT = (*es).selection_end;
    let mut l: INT = 0;
    let mut li: INT = 0;
    let mut ll: INT = 0;

    (*es).bCaptureState = TRUE;
    SetCapture((*es).hwndSelf);

    l = EDIT_EM_LineFromChar(es, e);
    li = EDIT_EM_LineIndex(es, l);
    ll = EDIT_EM_LineLength(es, e);
    s = li + EDIT_CallWordBreakProc(es, li, e - li, ll, (WB_LEFT) as INT);
    e = li + EDIT_CallWordBreakProc(es, li, e - li, ll, (WB_RIGHT) as INT);
    EDIT_EM_SetSel(es, s as UINT, e as UINT, FALSE);
    EDIT_EM_ScrollCaret(es);
    (*es).region_posy = 0;
    (*es).region_posx = (*es).region_posy;
    SetTimer((*es).hwndSelf, 0, 100, None);
    return 0;
}

unsafe extern "C" fn EDIT_WM_LButtonDown(es: *mut EDITSTATE, keys: DWORD, x: INT, y: INT) -> LRESULT {
	let mut x = x;
	let mut y = y;
	let mut e: INT;
	let mut after_wrap: BOOL = core::mem::zeroed();

	(*es).bCaptureState = TRUE as BOOL;
	SetCapture((*es).hwndSelf);
	EDIT_ConfinePoint(es, core::ptr::addr_of_mut!(x), core::ptr::addr_of_mut!(y));
	e = EDIT_CharFromPos(es, x, y, core::ptr::addr_of_mut!(after_wrap));
	EDIT_EM_SetSel(
		es,
		(if (keys & (MK_SHIFT as DWORD)) != 0 {
			(*es).selection_start
		} else {
			e
		}) as UINT,
		e as UINT,
		after_wrap,
	);
	EDIT_EM_ScrollCaret(es);
	(*es).region_posy = 0 as INT;
	(*es).region_posx = (*es).region_posy;
	SetTimer(
		(*es).hwndSelf,
		0 as UINT_PTR,
		100 as UINT,
		None,
	);

	if ((((*es).flags as INT) & (EF_FOCUSED as INT)) == 0) {
		SetFocus((*es).hwndSelf);
	}

	return 0 as LRESULT;
}

unsafe extern "C" fn EDIT_WM_LButtonUp(mut es: *mut EDITSTATE) -> LRESULT {
    if (*es).bCaptureState != 0 {
        KillTimer((*es).hwndSelf, 0 as UINT_PTR);
        if GetCapture() == (*es).hwndSelf {
            ReleaseCapture();
        }
    }
    (*es).bCaptureState = FALSE as BOOL;
    return 0 as LRESULT;
}

unsafe extern "C" fn EDIT_WM_MButtonDown(es: *mut EDITSTATE) -> LRESULT {
    SendMessageW((*es).hwndSelf, WM_PASTE as UINT, 0 as WPARAM, 0 as LPARAM);
    return 0 as LRESULT;
}

unsafe extern "C" fn EDIT_WM_MouseMove(es: *mut EDITSTATE, mut x: INT, mut y: INT) -> LRESULT {
    let mut e: INT;
    let mut after_wrap: BOOL = 0;
    let mut prex: INT;
    let mut prey: INT;

    if (*es).bCaptureState == 0 || GetCapture() != (*es).hwndSelf {
        return 0 as LRESULT;
    }

    prex = x;
    prey = y;
    EDIT_ConfinePoint(es, core::ptr::addr_of_mut!(x), core::ptr::addr_of_mut!(y));
    (*es).region_posx = (if prex < x {
        -1
    } else if prex > x {
        1
    } else {
        0
    }) as INT;
    (*es).region_posy = (if prey < y {
        -1
    } else if prey > y {
        1
    } else {
        0
    }) as INT;
    e = EDIT_CharFromPos(es, x, y, core::ptr::addr_of_mut!(after_wrap));
    EDIT_EM_SetSel(es, (*es).selection_start as UINT, e as UINT, after_wrap);
    EDIT_SetCaretPos(es, (*es).selection_end, (*es).flags as BOOL & EF_AFTER_WRAP);
    return 0 as LRESULT;
}

unsafe extern "C" fn EDIT_WM_Paint(es: *mut EDITSTATE, hdc: HDC) {
    let mut ps: PAINTSTRUCT = core::mem::zeroed();
    let mut i: INT;
    let mut dc: HDC;
    let mut old_font: HFONT = core::ptr::null_mut();
    let mut rc: RECT = core::mem::zeroed();
    let mut rcClient: RECT = core::mem::zeroed();
    let mut rcLine: RECT = core::mem::zeroed();
    let mut rcRgn: RECT = core::mem::zeroed();
    let mut brush: HBRUSH;
    let mut old_brush: HBRUSH;
    let mut bw: INT;
    let mut bh: INT;
    let mut rev: BOOL = (((*es).bEnableState != 0)
        && ((((*es).flags as DWORD) & (EF_FOCUSED as u32)) != 0
            || (((*es).style) & ES_NOHIDESEL) != 0)) as BOOL;
    dc = if !hdc.is_null() {
        hdc
    } else {
        BeginPaint((*es).hwndSelf, core::ptr::addr_of_mut!(ps))
    };

    /* The dc we use for calculating may not be the one we paint into.
       This is the safest action. */
    EDIT_InvalidateUniscribeData(es);
    GetClientRect((*es).hwndSelf, core::ptr::addr_of_mut!(rcClient));

    /* get the background brush */
    brush = EDIT_NotifyCtlColor(es, dc);

    /* paint the border and the background */
    IntersectClipRect(dc, rcClient.left, rcClient.top, rcClient.right, rcClient.bottom);

    if ((*es).style & WS_BORDER) != 0 {
        bw = GetSystemMetrics((SM_CXBORDER) as i32);
        bh = GetSystemMetrics((SM_CYBORDER) as i32);
        rc = rcClient;
        if ((*es).style & ES_MULTILINE) != 0 {
            if ((*es).style & WS_HSCROLL) != 0 {
                rc.bottom += bh;
            }
            if ((*es).style & WS_VSCROLL) != 0 {
                rc.right += bw;
            }
        }

        /* Draw the frame. Same code as in nonclient.c */
        old_brush = SelectObject(dc, GetSysColorBrush((COLOR_WINDOWFRAME) as i32) as HGDIOBJ) as HBRUSH;
        PatBlt(dc, rc.left, rc.top, rc.right - rc.left, bh, PATCOPY);
        PatBlt(dc, rc.left, rc.top, bw, rc.bottom - rc.top, PATCOPY);
        PatBlt(dc, rc.left, rc.bottom - 1, rc.right - rc.left, -bw, PATCOPY);
        PatBlt(dc, rc.right - 1, rc.top, -bw, rc.bottom - rc.top, PATCOPY);
        SelectObject(dc, old_brush as HGDIOBJ);

        /* Keep the border clean */
        IntersectClipRect(dc, rc.left + bw, rc.top + bh,
            max(rc.right - bw, rc.left + bw), max(rc.bottom - bh, rc.top + bh));
    }

    GetClipBox(dc, core::ptr::addr_of_mut!(rc));
    FillRect(dc, core::ptr::addr_of!(rc) as LPCRECT, brush);

    IntersectClipRect(dc, (*es).format_rect.left,
        (*es).format_rect.top,
        (*es).format_rect.right,
        (*es).format_rect.bottom);
    if ((*es).style & ES_MULTILINE) != 0 {
        rc = rcClient;
        IntersectClipRect(dc, rc.left, rc.top, rc.right, rc.bottom);
    }
    if !(*es).font.is_null() {
        old_font = SelectObject(dc, (*es).font as HGDIOBJ) as HFONT;
    }

    if (*es).bEnableState == 0 {
        SetTextColor(dc, GetSysColor((COLOR_GRAYTEXT) as i32) as COLORREF);
    }
    GetClipBox(dc, core::ptr::addr_of_mut!(rcRgn));
    if ((*es).style & ES_MULTILINE) != 0 {
        let vlc: INT = get_vertical_line_count(es);
        i = (*es).y_offset;
        while i <= min((*es).y_offset + vlc, (*es).y_offset + (*es).line_count - 1) {
            EDIT_UpdateUniscribeData(es, dc, i);
            EDIT_GetLineRect(es, i, 0, -1, core::ptr::addr_of_mut!(rcLine));
            if IntersectRect(core::ptr::addr_of_mut!(rc), core::ptr::addr_of!(rcRgn) as LPCRECT, core::ptr::addr_of!(rcLine) as LPCRECT) != 0 {
                EDIT_PaintLine(es, dc, i, rev);
            }
            i += 1;
        }
    } else {
        EDIT_UpdateUniscribeData(es, dc, 0);
        EDIT_GetLineRect(es, 0, 0, -1, core::ptr::addr_of_mut!(rcLine));
        if IntersectRect(core::ptr::addr_of_mut!(rc), core::ptr::addr_of!(rcRgn) as LPCRECT, core::ptr::addr_of!(rcLine) as LPCRECT) != 0 {
            EDIT_PaintLine(es, dc, 0, rev);
        }
    }
    if !(*es).font.is_null() {
        SelectObject(dc, old_font as HGDIOBJ);
    }

    if hdc.is_null() {
        EndPaint((*es).hwndSelf, core::ptr::addr_of_mut!(ps));
    }
}

unsafe extern "C" fn EDIT_WM_NCPaint(mut hwnd: HWND, mut region: HRGN) {
    let mut exStyle: DWORD = GetWindowLongW(hwnd, GWL_EXSTYLE) as DWORD;
    let mut theme: HTHEME = GetWindowTheme(hwnd);
    let mut cliprgn: HRGN = region;
    if !theme.is_null() && exStyle & WS_EX_CLIENTEDGE as DWORD != 0 {
        let mut dc: HDC = core::ptr::null_mut();
        let mut r: RECT = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        let mut cxEdge: ::core::ffi::c_int =
            GetSystemMetrics((SM_CXEDGE) as i32) as ::core::ffi::c_int;
        let mut cyEdge: ::core::ffi::c_int =
            GetSystemMetrics((SM_CYEDGE) as i32) as ::core::ffi::c_int;
        let part: ::core::ffi::c_int = EP_EDITTEXT as ::core::ffi::c_int;
        let mut state: ::core::ffi::c_int = ETS_NORMAL as ::core::ffi::c_int;
        let mut dwStyle: DWORD =
            GetWindowLongW(hwnd, GWL_STYLE) as DWORD;
        if IsWindowEnabled(hwnd) == 0 {
            state = ETS_DISABLED as ::core::ffi::c_int;
        } else if dwStyle & ES_READONLY as DWORD != 0 {
            state = ETS_READONLY as ::core::ffi::c_int;
        } else if GetFocus() == hwnd {
            state = ETS_FOCUSED as ::core::ffi::c_int;
        }
        GetWindowRect(hwnd, &raw mut r);
        /* New clipping region passed to default proc to exclude border */
        cliprgn = CreateRectRgn(
            r.left as ::core::ffi::c_int + cxEdge,
            r.top as ::core::ffi::c_int + cyEdge,
            r.right as ::core::ffi::c_int - cxEdge,
            r.bottom as ::core::ffi::c_int - cyEdge,
        );
        if region != (1 as usize as HRGN) {
            CombineRgn(cliprgn, cliprgn, region, RGN_AND as ::core::ffi::c_int);
        }
        OffsetRect(
            &raw mut r,
            -(r.left as ::core::ffi::c_int),
            -(r.top as ::core::ffi::c_int),
        );
        dc = GetWindowDC(hwnd);
        /* Exclude client part */
        ExcludeClipRect(
            dc,
            r.left as ::core::ffi::c_int + cxEdge,
            r.top as ::core::ffi::c_int + cyEdge,
            r.right as ::core::ffi::c_int - cxEdge,
            r.bottom as ::core::ffi::c_int - cyEdge,
        );
        if IsThemeBackgroundPartiallyTransparent(theme, part, state) != 0 {
            DrawThemeParentBackground(hwnd, dc, &raw mut r);
        }
        DrawThemeBackground(
            theme,
            dc,
            part,
            state,
            &raw mut r,
            0 as usize as *const RECT,
        );
        ReleaseDC(hwnd, dc);
    }
    /* Call default proc to get the scrollbars etc. also painted */
    DefWindowProcW(
        hwnd,
        WM_NCPAINT as UINT,
        cliprgn as WPARAM,
        0 as LPARAM,
    );
    if cliprgn != region {
        DeleteObject(cliprgn as HGDIOBJ);
    }
}

unsafe extern "C" fn EDIT_WM_SetFocus(theme: HTHEME, es: *mut EDITSTATE) {
    let mut flags: UINT = (RDW_INVALIDATE | RDW_UPDATENOW) as UINT;

    (*es).flags = ((*es).flags as ::core::ffi::c_int | EF_FOCUSED) as WORD;

    if ((*es).style & ES_NOHIDESEL as DWORD) == 0 {
        EDIT_InvalidateText(es, (*es).selection_start, (*es).selection_end);
    }

    SystemParametersInfo(
        SPI_GETCARETWIDTH as UINT,
        0 as UINT,
        core::ptr::addr_of_mut!((*es).dwCaretWidth) as PVOID,
        0 as UINT,
    );
    CreateCaret(
        (*es).hwndSelf,
        core::ptr::null_mut(),
        (*es).dwCaretWidth as ::core::ffi::c_int,
        (*es).line_height as ::core::ffi::c_int,
    );
    EDIT_SetCaretPos(
        es,
        (*es).selection_end,
        (((*es).flags as ::core::ffi::c_int & EF_AFTER_WRAP) as BOOL),
    );
    ShowCaret((*es).hwndSelf);
    if notify_parent(es, (EN_SETFOCUS) as INT) == 0 {
        return;
    }

    if !theme.is_null() {
        flags |= (RDW_FRAME | RDW_ERASE) as UINT;
    }

    RedrawWindow(
        (*es).hwndSelf,
        core::ptr::null(),
        core::ptr::null_mut(),
        flags,
    );
}

unsafe extern "C" fn get_font_margins(hdc: HDC, tm: *const TEXTMETRICW) -> DWORD {
    let mut abc: [ABC; 256] = core::mem::zeroed();
    let mut left: SHORT = 0;
    let mut right: SHORT = 0;
    let mut i: UINT = 0;

    if ((*tm).tmPitchAndFamily as u32) & (TMPF_VECTOR | TMPF_TRUETYPE) == 0 {
        return MAKELONG!(EC_USEFONTINFO as WORD, EC_USEFONTINFO as WORD) as DWORD;
    }

    if is_cjk(hdc) == 0 {
        return MAKELONG!(EC_USEFONTINFO as WORD, EC_USEFONTINFO as WORD) as DWORD;
    }

    if GetCharABCWidthsW(hdc, 0 as UINT, 255 as UINT, core::ptr::addr_of_mut!(abc) as LPABC) == 0 {
        return 0 as DWORD;
    }

    right = 0 as SHORT;
    left = right;
    i = 0 as UINT;
    while (i as usize) < (ARRAY_SIZE!(abc) as usize) {
        if (-abc[i as usize].abcA) > right as i32 {
            right = (-abc[i as usize].abcA) as SHORT;
        }
        if (-abc[i as usize].abcC) > left as i32 {
            left = (-abc[i as usize].abcC) as SHORT;
        }
        i = i.wrapping_add(1);
    }
    return MAKELONG!(left as WORD, right as WORD) as DWORD;
}

unsafe extern "C" fn EDIT_WM_SetFont(mut es: *mut EDITSTATE, mut font: HFONT, mut redraw: BOOL) {
    let mut tm: TEXTMETRICW = core::mem::zeroed();
    let mut dc: HDC;
    let mut old_font: HFONT = core::ptr::null_mut();
    let mut clientRect: RECT = core::mem::zeroed();
    let mut margins: DWORD;

    (*es).font = font;
    EDIT_InvalidateUniscribeData(es);
    dc = GetDC((*es).hwndSelf);
    if !font.is_null() {
        old_font = SelectObject(dc, font as HGDIOBJ) as HFONT;
    }
    GetTextMetricsW(dc, &raw mut tm);
    (*es).line_height = tm.tmHeight as INT;
    (*es).char_width = tm.tmAveCharWidth as INT;
    margins = get_font_margins(dc, &raw mut tm);
    if !font.is_null() {
        SelectObject(dc, old_font as HGDIOBJ);
    }
    ReleaseDC((*es).hwndSelf, dc);

    /* Reset the format rect and the margins */
    GetClientRect((*es).hwndSelf, &raw mut clientRect);
    EDIT_SetRectNP(es, &raw mut clientRect);
    if margins != 0 {
        EDIT_EM_SetMargins(
            es,
            (EC_LEFTMARGIN | EC_RIGHTMARGIN) as INT,
            LOWORD!(margins),
            HIWORD!(margins),
            FALSE,
        );
    }

    if ((*es).style & ES_MULTILINE) != 0 {
        EDIT_BuildLineDefs_ML(
            es,
            0,
            get_text_length(es) as INT,
            0,
            core::ptr::null_mut(),
        );
    } else {
        EDIT_CalcLineWidth_SL(es);
    }

    if redraw != 0 {
        EDIT_UpdateText(es, core::ptr::null(), TRUE);
    }
    if ((*es).flags as INT & EF_FOCUSED) != 0 {
        DestroyCaret();
        CreateCaret(
            (*es).hwndSelf,
            core::ptr::null_mut(),
            (*es).dwCaretWidth as INT,
            (*es).line_height as INT,
        );
        EDIT_SetCaretPos(
            es,
            (*es).selection_end,
            ((*es).flags as BOOL & EF_AFTER_WRAP),
        );
        ShowCaret((*es).hwndSelf);
    }
    if ImmIsIME(GetKeyboardLayout(0)) != 0 {
        let mut lf: LOGFONTW = core::mem::zeroed();
        let mut hIMC: HIMC = ImmGetContext((*es).hwndSelf);
        if font.is_null() {
            font = GetStockObject((DEFAULT_GUI_FONT as i32)) as HFONT;
        }
        GetObjectW(
            font as HANDLE,
            core::mem::size_of::<LOGFONTW>() as INT,
            &raw mut lf as LPVOID,
        );
        ImmSetCompositionFontW(hIMC, &raw mut lf);
        ImmReleaseContext((*es).hwndSelf, hIMC);
    }
}

unsafe extern "C" fn EDIT_WM_SetText(mut es: *mut EDITSTATE, mut text: LPCWSTR) {
    if (*es).flags as ::core::ffi::c_int & EF_UPDATE != 0 {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/win32/comctl32/edit.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"EDIT_WM_SetText\0" as *const u8 as *const ::core::ffi::c_char,
                3916 as ::core::ffi::c_int,
                b"SetSel may generate UPDATE message whose handler may reset selection.\n\0"
                    as *const u8 as *const ::core::ffi::c_char,
            );
        }
    }
    EDIT_EM_SetSel(
        es,
        0 as UINT,
        -(1 as ::core::ffi::c_int) as UINT,
        FALSE,
    );
    if !text.is_null() {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
        {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl_0,
                __dbch_0,
                b"dll/win32/comctl32/edit.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"EDIT_WM_SetText\0" as *const u8 as *const ::core::ffi::c_char,
                3921 as ::core::ffi::c_int,
                b"%s\n\0" as *const u8 as *const ::core::ffi::c_char,
                debugstr_w(text as *const WCHAR),
            );
        }
        EDIT_EM_ReplaceSel(
            es,
            FALSE,
            text as *const WCHAR,
            lstrlenW(text) as UINT,
            FALSE,
            FALSE,
        );
    } else {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
        {
            let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_1: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl_1,
                __dbch_1,
                b"dll/win32/comctl32/edit.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"EDIT_WM_SetText\0" as *const u8 as *const ::core::ffi::c_char,
                3926 as ::core::ffi::c_int,
                b"<NULL>\n\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        EDIT_EM_ReplaceSel(
            es,
            FALSE,
            ::core::ptr::null::<WCHAR>(),
            0 as UINT,
            FALSE,
            FALSE,
        );
    }
    (*es).x_offset = 0 as INT;
    (*es).flags = ((*es).flags as ::core::ffi::c_int & !(EF_MODIFIED as ::core::ffi::c_int)) as WORD;
    EDIT_EM_SetSel(
        es,
        0 as UINT,
        0 as UINT,
        FALSE,
    );

    /* Send the notification after the selection start and end have been set
     * edit control doesn't send notification on WM_SETTEXT
     * if it is multiline, or it is part of combobox
     */
    if !((*es).style & (ES_MULTILINE as DWORD) != 0 || !(*es).hwndListBox.is_null()) {
        if notify_parent(es, (EN_UPDATE) as INT) == 0 {
            return;
        }
        if notify_parent(es, (EN_CHANGE) as INT) == 0 {
            return;
        }
    }
    EDIT_EM_ScrollCaret(es);
    EDIT_UpdateScrollInfo(es);
    EDIT_InvalidateUniscribeData(es);
}

unsafe extern "C" fn EDIT_WM_Size(mut es: *mut EDITSTATE, mut action: UINT) {
    if action == SIZE_MAXIMIZED || action == SIZE_RESTORED {
        let mut rc: RECT = core::mem::zeroed();
        GetClientRect((*es).hwndSelf, core::ptr::addr_of_mut!(rc));
        EDIT_SetRectNP(es, core::ptr::addr_of_mut!(rc));
        EDIT_UpdateText(es, core::ptr::null_mut(), TRUE);
    }
}

unsafe extern "C" fn EDIT_WM_StyleChanged(
    mut es: *mut EDITSTATE,
    mut which: WPARAM,
    mut style: *const STYLESTRUCT,
) -> LRESULT {
    if GWL_STYLE as WPARAM == which {
        let mut style_change_mask: DWORD = 0;
        let mut new_style: DWORD = 0;
        style_change_mask = (ES_UPPERCASE
            | ES_LOWERCASE
            | ES_NUMBER
            | ES_LEFT
            | ES_RIGHT
            | ES_CENTER) as DWORD;
        if (*es).style & ES_MULTILINE as DWORD != 0 {
            style_change_mask |= ES_WANTRETURN as DWORD;
        }
        new_style = (*style).styleNew & style_change_mask;
        if !(new_style & ES_NUMBER as DWORD != 0) {
            if new_style & ES_LOWERCASE as DWORD != 0 {
                new_style &= !ES_UPPERCASE as DWORD;
            }
        }
        (*es).style = (*es).style & !style_change_mask | new_style;
    } else if !(GWL_EXSTYLE as WPARAM == which) {
        if (*__wine_dbch___default).flags as core::ffi::c_int
            & (1 as core::ffi::c_int) << __WINE_DBCL_WARN as core::ffi::c_int
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_WARN;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/win32/comctl32/edit.c\0"
                    as *const u8 as *const core::ffi::c_char,
                b"EDIT_WM_StyleChanged\0" as *const u8 as *const core::ffi::c_char,
                4020 as core::ffi::c_int,
                b"Invalid style change %ld\n\0" as *const u8 as *const core::ffi::c_char,
                which,
            );
        }
    }
    return 0 as LRESULT;
}

unsafe extern "C" fn EDIT_WM_SysKeyDown(es: *mut EDITSTATE, key: INT, key_data: DWORD) -> LRESULT {
    if ((key) as u32 == VK_BACK) && ((key_data & 0x2000) != 0) {
        if EDIT_EM_CanUndo(es) != 0 {
            EDIT_EM_Undo(es);
        }
        return 0 as LRESULT;
    } else if (key) as u32 == VK_UP || (key) as u32 == VK_DOWN {
        if EDIT_CheckCombo(es, WM_SYSKEYDOWN as UINT, key) != 0 {
            return 0 as LRESULT;
        }
    }
    return DefWindowProcW(
        (*es).hwndSelf,
        WM_SYSKEYDOWN as UINT,
        key as WPARAM,
        key_data as LPARAM,
    );
}

unsafe extern "C" fn EDIT_WM_Timer(es: *mut EDITSTATE) {
    if (*es).region_posx < 0 {
        EDIT_MoveBackward(es, TRUE);
    } else if (*es).region_posx > 0 {
        EDIT_MoveForward(es, TRUE);
    }
    /*
     *	FIXME: gotta do some vertical scrolling here, like
     *		EDIT_EM_LineScroll(hwnd, 0, 1);
     */
}

unsafe extern "C" fn EDIT_WM_HScroll(es: *mut EDITSTATE, action: INT, pos: INT) -> LRESULT {
    let mut dx: INT = 0;
    let mut fw: INT = 0;

    if (*es).style & ES_MULTILINE as DWORD == 0 {
        return 0 as LRESULT;
    }

    if (*es).style & ES_AUTOHSCROLL as DWORD == 0 {
        return 0 as LRESULT;
    }

    dx = 0 as INT;
    fw = ((*es).format_rect.right - (*es).format_rect.left) as INT;
    match (action) as u32 {
        SB_LINELEFT => {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                != 0
            {
                let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl,
                    __dbch,
                    b"dll/win32/comctl32/edit.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"EDIT_WM_HScroll\0" as *const u8 as *const ::core::ffi::c_char,
                    4083 as ::core::ffi::c_int,
                    b"SB_LINELEFT\n\0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
            if (*es).x_offset != 0 {
                dx = -(*es).char_width;
            }
        }
        SB_LINERIGHT => {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                != 0
            {
                let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl_0,
                    __dbch_0,
                    b"dll/win32/comctl32/edit.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"EDIT_WM_HScroll\0" as *const u8 as *const ::core::ffi::c_char,
                    4088 as ::core::ffi::c_int,
                    b"SB_LINERIGHT\n\0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
            if (*es).x_offset < (*es).text_width {
                dx = (*es).char_width;
            }
        }
        SB_PAGELEFT => {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                != 0
            {
                let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_1: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl_1,
                    __dbch_1,
                    b"dll/win32/comctl32/edit.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"EDIT_WM_HScroll\0" as *const u8 as *const ::core::ffi::c_char,
                    4093 as ::core::ffi::c_int,
                    b"SB_PAGELEFT\n\0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
            if (*es).x_offset != 0 {
                dx = -fw / HSCROLL_FRACTION / (*es).char_width * (*es).char_width;
            }
        }
        SB_PAGERIGHT => {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                != 0
            {
                let __dbch_2: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_2: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl_2,
                    __dbch_2,
                    b"dll/win32/comctl32/edit.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"EDIT_WM_HScroll\0" as *const u8 as *const ::core::ffi::c_char,
                    4098 as ::core::ffi::c_int,
                    b"SB_PAGERIGHT\n\0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
            if (*es).x_offset < (*es).text_width {
                dx = fw / HSCROLL_FRACTION / (*es).char_width * (*es).char_width;
            }
        }
        SB_LEFT => {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                != 0
            {
                let __dbch_3: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_3: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl_3,
                    __dbch_3,
                    b"dll/win32/comctl32/edit.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"EDIT_WM_HScroll\0" as *const u8 as *const ::core::ffi::c_char,
                    4103 as ::core::ffi::c_int,
                    b"SB_LEFT\n\0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
            if (*es).x_offset != 0 {
                dx = -(*es).x_offset;
            }
        }
        SB_RIGHT => {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                != 0
            {
                let __dbch_4: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_4: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl_4,
                    __dbch_4,
                    b"dll/win32/comctl32/edit.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"EDIT_WM_HScroll\0" as *const u8 as *const ::core::ffi::c_char,
                    4108 as ::core::ffi::c_int,
                    b"SB_RIGHT\n\0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
            if (*es).x_offset < (*es).text_width {
                dx = (*es).text_width - (*es).x_offset;
            }
        }
        SB_THUMBTRACK => {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                != 0
            {
                let __dbch_5: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_5: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl_5,
                    __dbch_5,
                    b"dll/win32/comctl32/edit.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"EDIT_WM_HScroll\0" as *const u8 as *const ::core::ffi::c_char,
                    4113 as ::core::ffi::c_int,
                    b"SB_THUMBTRACK %d\n\0" as *const u8 as *const ::core::ffi::c_char,
                    pos,
                );
            }
            (*es).flags = ((*es).flags as ::core::ffi::c_int | EF_HSCROLL_TRACK as ::core::ffi::c_int) as WORD;
            if (*es).style & WS_HSCROLL as DWORD != 0 {
                dx = pos - (*es).x_offset;
            } else {
                let mut fw_0: INT = 0;
                let mut new_x: INT = 0;
                if pos < 0 || pos > 100 {
                    return 0 as LRESULT;
                }
                fw_0 = ((*es).format_rect.right - (*es).format_rect.left) as INT;
                new_x = (pos as ::core::ffi::c_int
                    * ((*es).text_width as ::core::ffi::c_int - fw_0 as ::core::ffi::c_int)
                    / 100 as ::core::ffi::c_int) as INT;
                dx = (if (*es).text_width != 0 {
                    new_x as ::core::ffi::c_int - (*es).x_offset as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                }) as INT;
            }
        }
        SB_THUMBPOSITION => {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                != 0
            {
                let __dbch_6: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_6: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl_6,
                    __dbch_6,
                    b"dll/win32/comctl32/edit.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"EDIT_WM_HScroll\0" as *const u8 as *const ::core::ffi::c_char,
                    4129 as ::core::ffi::c_int,
                    b"SB_THUMBPOSITION %d\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                    pos,
                );
            }
            (*es).flags = ((*es).flags as ::core::ffi::c_int & !(EF_HSCROLL_TRACK as ::core::ffi::c_int)) as WORD;
            if (GetWindowLongW((*es).hwndSelf, GWL_STYLE) as ::core::ffi::c_int & WS_HSCROLL as ::core::ffi::c_int) != 0 {
                dx = pos - (*es).x_offset;
            } else {
                let mut fw_1: INT = 0;
                let mut new_x_0: INT = 0;
                if pos < 0 || pos > 100 {
                    return 0 as LRESULT;
                }
                fw_1 = ((*es).format_rect.right - (*es).format_rect.left) as INT;
                new_x_0 = (pos as ::core::ffi::c_int
                    * ((*es).text_width as ::core::ffi::c_int - fw_1 as ::core::ffi::c_int)
                    / 100 as ::core::ffi::c_int) as INT;
                dx = (if (*es).text_width != 0 {
                    new_x_0 as ::core::ffi::c_int - (*es).x_offset as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                }) as INT;
            }
            if dx == 0 {
                EDIT_UpdateScrollInfo(es);
                notify_parent(es, (EN_HSCROLL) as INT);
            }
        }
        SB_ENDSCROLL => {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                != 0
            {
                let __dbch_7: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_7: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl_7,
                    __dbch_7,
                    b"dll/win32/comctl32/edit.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"EDIT_WM_HScroll\0" as *const u8 as *const ::core::ffi::c_char,
                    4150 as ::core::ffi::c_int,
                    b"SB_ENDSCROLL\n\0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
        }
        EM_GETTHUMB => {
            let mut ret: LRESULT = 0;
            if (GetWindowLongW((*es).hwndSelf, GWL_STYLE) as ::core::ffi::c_int & WS_HSCROLL as ::core::ffi::c_int) != 0 {
                ret = GetScrollPos((*es).hwndSelf, (SB_HORZ) as i32) as LRESULT;
            } else {
                let mut fw_2: INT = (*es).format_rect.right as INT - (*es).format_rect.left as INT;
                ret = (if (*es).text_width != 0 {
                    (*es).x_offset as ::core::ffi::c_int * 100 as ::core::ffi::c_int
                        / ((*es).text_width as ::core::ffi::c_int - fw_2 as ::core::ffi::c_int)
                } else {
                    0 as ::core::ffi::c_int
                }) as LRESULT;
            }
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                != 0
            {
                let __dbch_8: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_8: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl_8,
                    __dbch_8,
                    b"dll/win32/comctl32/edit.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"EDIT_WM_HScroll\0" as *const u8 as *const ::core::ffi::c_char,
                    4169 as ::core::ffi::c_int,
                    b"EM_GETTHUMB: returning %ld\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                    ret,
                );
            }
            return ret;
        }
        EM_LINESCROLL => {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                != 0
            {
                let __dbch_9: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_9: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl_9,
                    __dbch_9,
                    b"dll/win32/comctl32/edit.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"EDIT_WM_HScroll\0" as *const u8 as *const ::core::ffi::c_char,
                    4173 as ::core::ffi::c_int,
                    b"EM_LINESCROLL16\n\0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
            dx = pos;
        }
        _ => {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                != 0
            {
                let __dbch_10: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_10: __wine_debug_class = __WINE_DBCL_ERR;
                ros_dbg_log(
                    __dbcl_10,
                    __dbch_10,
                    b"dll/win32/comctl32/edit.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"EDIT_WM_HScroll\0" as *const u8 as *const ::core::ffi::c_char,
                    4179 as ::core::ffi::c_int,
                    b"undocumented WM_HSCROLL action %d (0x%04x), please report\n\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    action,
                    action,
                );
            }
            return 0 as LRESULT;
        }
    }
    if dx != 0 {
        let mut fw_3: INT = (*es).format_rect.right as INT - (*es).format_rect.left as INT;
        if (*es).x_offset + dx + fw_3 > (*es).text_width {
            dx = (*es).text_width - fw_3 - (*es).x_offset;
        }
        if dx != 0 {
            EDIT_EM_LineScroll_internal(es, dx, 0 as INT);
        }
    }
    return 0 as LRESULT;
}

unsafe extern "C" fn EDIT_WM_VScroll(es: *mut EDITSTATE, action: INT, pos: INT) -> LRESULT {
    let mut dy: INT = 0;
    if (*es).style & (ES_MULTILINE as DWORD) == 0 {
        return 0;
    }
    if (*es).style & (ES_AUTOVSCROLL as DWORD) == 0 {
        return 0;
    }
    dy = 0;
    match (action) as u32 {
        SB_LINEUP | SB_LINEDOWN | SB_PAGEUP | SB_PAGEDOWN => {
            if (*__wine_dbch___default).flags as INT & (1 as INT) << __WINE_DBCL_TRACE as INT != 0 {
                ros_dbg_log(
                    __WINE_DBCL_TRACE,
                    __wine_dbch___default,
                    b"dll/win32/comctl32/edit.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"EDIT_WM_VScroll\0" as *const u8 as *const ::core::ffi::c_char,
                    4219 as ::core::ffi::c_int,
                    b"action %d (%s)\n\0" as *const u8 as *const ::core::ffi::c_char,
                    action,
                    if (action) as u32 == SB_LINEUP {
                        b"SB_LINEUP\0" as *const u8 as *const ::core::ffi::c_char
                    } else if (action) as u32 == SB_LINEDOWN {
                        b"SB_LINEDOWN\0" as *const u8 as *const ::core::ffi::c_char
                    } else if (action) as u32 == SB_PAGEUP {
                        b"SB_PAGEUP\0" as *const u8 as *const ::core::ffi::c_char
                    } else {
                        b"SB_PAGEDOWN\0" as *const u8 as *const ::core::ffi::c_char
                    },
                );
            }
            EDIT_EM_Scroll(es, action);
            return 0;
        }
        SB_TOP => {
            if (*__wine_dbch___default).flags as INT & (1 as INT) << __WINE_DBCL_TRACE as INT != 0 {
                ros_dbg_log(
                    __WINE_DBCL_TRACE,
                    __wine_dbch___default,
                    b"dll/win32/comctl32/edit.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"EDIT_WM_VScroll\0" as *const u8 as *const ::core::ffi::c_char,
                    4223 as ::core::ffi::c_int,
                    b"SB_TOP\n\0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
            dy = -(*es).y_offset;
        }
        SB_BOTTOM => {
            if (*__wine_dbch___default).flags as INT & (1 as INT) << __WINE_DBCL_TRACE as INT != 0 {
                ros_dbg_log(
                    __WINE_DBCL_TRACE,
                    __wine_dbch___default,
                    b"dll/win32/comctl32/edit.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"EDIT_WM_VScroll\0" as *const u8 as *const ::core::ffi::c_char,
                    4227 as ::core::ffi::c_int,
                    b"SB_BOTTOM\n\0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
            dy = (*es).line_count - 1 - (*es).y_offset;
        }
        SB_THUMBTRACK => {
            if (*__wine_dbch___default).flags as INT & (1 as INT) << __WINE_DBCL_TRACE as INT != 0 {
                ros_dbg_log(
                    __WINE_DBCL_TRACE,
                    __wine_dbch___default,
                    b"dll/win32/comctl32/edit.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"EDIT_WM_VScroll\0" as *const u8 as *const ::core::ffi::c_char,
                    4231 as ::core::ffi::c_int,
                    b"SB_THUMBTRACK %d\n\0" as *const u8 as *const ::core::ffi::c_char,
                    pos,
                );
            }
            (*es).flags = ((*es).flags as INT | (EF_VSCROLL_TRACK as INT)) as WORD;
            if (*es).style & (WS_VSCROLL as DWORD) != 0 {
                dy = pos - (*es).y_offset;
            } else {
                let mut vlc: INT = 0;
                let mut new_y: INT = 0;
                if pos < 0 || pos > 100 {
                    return 0;
                }
                vlc = get_vertical_line_count(es);
                new_y = pos * ((*es).line_count - vlc) / 100;
                dy = if (*es).line_count != 0 {
                    new_y - (*es).y_offset
                } else {
                    0
                };
                if (*__wine_dbch___default).flags as INT & (1 as INT) << __WINE_DBCL_TRACE as INT != 0 {
                    ros_dbg_log(
                        __WINE_DBCL_TRACE,
                        __wine_dbch___default,
                        b"dll/win32/comctl32/edit.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"EDIT_WM_VScroll\0" as *const u8 as *const ::core::ffi::c_char,
                        4245 as ::core::ffi::c_int,
                        b"line_count=%d, y_offset=%d, pos=%d, dy = %d\n\0" as *const u8
                            as *const ::core::ffi::c_char,
                        (*es).line_count,
                        (*es).y_offset,
                        pos,
                        dy,
                    );
                }
            }
        }
        SB_THUMBPOSITION => {
            if (*__wine_dbch___default).flags as INT & (1 as INT) << __WINE_DBCL_TRACE as INT != 0 {
                ros_dbg_log(
                    __WINE_DBCL_TRACE,
                    __wine_dbch___default,
                    b"dll/win32/comctl32/edit.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"EDIT_WM_VScroll\0" as *const u8 as *const ::core::ffi::c_char,
                    4249 as ::core::ffi::c_int,
                    b"SB_THUMBPOSITION %d\n\0" as *const u8 as *const ::core::ffi::c_char,
                    pos,
                );
            }
            (*es).flags = ((*es).flags as INT & !(EF_VSCROLL_TRACK as INT)) as WORD;
            if (*es).style & (WS_VSCROLL as DWORD) != 0 {
                dy = pos - (*es).y_offset;
            } else {
                let mut vlc_0: INT = 0;
                let mut new_y_0: INT = 0;
                if pos < 0 || pos > 100 {
                    return 0;
                }
                vlc_0 = get_vertical_line_count(es);
                new_y_0 = pos * ((*es).line_count - vlc_0) / 100;
                dy = if (*es).line_count != 0 {
                    new_y_0 - (*es).y_offset
                } else {
                    0
                };
                if (*__wine_dbch___default).flags as INT & (1 as INT) << __WINE_DBCL_TRACE as INT != 0 {
                    ros_dbg_log(
                        __WINE_DBCL_TRACE,
                        __wine_dbch___default,
                        b"dll/win32/comctl32/edit.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"EDIT_WM_VScroll\0" as *const u8 as *const ::core::ffi::c_char,
                        4263 as ::core::ffi::c_int,
                        b"line_count=%d, y_offset=%d, pos=%d, dy = %d\n\0" as *const u8
                            as *const ::core::ffi::c_char,
                        (*es).line_count,
                        (*es).y_offset,
                        pos,
                        dy,
                    );
                }
            }
            if dy == 0 {
                EDIT_UpdateScrollInfo(es);
                notify_parent(es, (EN_VSCROLL) as INT);
            }
        }
        SB_ENDSCROLL => {
            if (*__wine_dbch___default).flags as INT & (1 as INT) << __WINE_DBCL_TRACE as INT != 0 {
                ros_dbg_log(
                    __WINE_DBCL_TRACE,
                    __wine_dbch___default,
                    b"dll/win32/comctl32/edit.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"EDIT_WM_VScroll\0" as *const u8 as *const ::core::ffi::c_char,
                    4273 as ::core::ffi::c_int,
                    b"SB_ENDSCROLL\n\0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
        }
        EM_GETTHUMB => {
            let mut ret: LRESULT = 0;
            if (GetWindowLongW((*es).hwndSelf, GWL_STYLE) as INT)
                & (WS_VSCROLL as INT)
                != 0
            {
                ret = GetScrollPos((*es).hwndSelf, (SB_VERT) as i32) as LRESULT;
            } else {
                let mut vlc_1: INT = get_vertical_line_count(es);
                ret = (if (*es).line_count != 0 {
                    (*es).y_offset * 100 / ((*es).line_count - vlc_1)
                } else {
                    0
                }) as LRESULT;
            }
            if (*__wine_dbch___default).flags as INT & (1 as INT) << __WINE_DBCL_TRACE as INT != 0 {
                ros_dbg_log(
                    __WINE_DBCL_TRACE,
                    __wine_dbch___default,
                    b"dll/win32/comctl32/edit.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"EDIT_WM_VScroll\0" as *const u8 as *const ::core::ffi::c_char,
                    4292 as ::core::ffi::c_int,
                    b"EM_GETTHUMB: returning %ld\n\0" as *const u8 as *const ::core::ffi::c_char,
                    ret,
                );
            }
            return ret;
        }
        EM_LINESCROLL => {
            if (*__wine_dbch___default).flags as INT & (1 as INT) << __WINE_DBCL_TRACE as INT != 0 {
                ros_dbg_log(
                    __WINE_DBCL_TRACE,
                    __wine_dbch___default,
                    b"dll/win32/comctl32/edit.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"EDIT_WM_VScroll\0" as *const u8 as *const ::core::ffi::c_char,
                    4296 as ::core::ffi::c_int,
                    b"EM_LINESCROLL %d\n\0" as *const u8 as *const ::core::ffi::c_char,
                    pos,
                );
            }
            dy = pos;
        }
        _ => {
            if (*__wine_dbch___default).flags as INT & (1 as INT) << __WINE_DBCL_ERR as INT != 0 {
                ros_dbg_log(
                    __WINE_DBCL_ERR,
                    __wine_dbch___default,
                    b"dll/win32/comctl32/edit.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"EDIT_WM_VScroll\0" as *const u8 as *const ::core::ffi::c_char,
                    4302 as ::core::ffi::c_int,
                    b"undocumented WM_VSCROLL action %d (0x%04x), please report\n\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    action,
                    action,
                );
            }
            return 0;
        }
    }
    if dy != 0 {
        EDIT_EM_LineScroll(es, 0, dy);
    }
    return 0;
}

unsafe extern "C" fn EDIT_EM_GetThumb(es: *mut EDITSTATE) -> LRESULT {
    return MAKELONG!(
        EDIT_WM_VScroll(es, (EM_GETTHUMB) as INT, 0),
        EDIT_WM_HScroll(es, (EM_GETTHUMB) as INT, 0),
    ) as LRESULT;
}

unsafe extern "C" fn heap_strdupW(str: *const WCHAR) -> *mut WCHAR {
    let len: ::core::ffi::c_int = lstrlenW(str as LPCWSTR) + 1;
    let ret: *mut WCHAR = heap_alloc((len as SIZE_T).wrapping_mul(::core::mem::size_of::<WCHAR>() as SIZE_T)) as *mut WCHAR;
    lstrcpyW(ret as LPWSTR, str as LPCWSTR);
    return ret;
}

unsafe extern "C" fn EDIT_EM_SetCueBanner(es: *mut EDITSTATE, draw_focused: BOOL, cue_text: *const WCHAR) -> BOOL {
    if (((*es).style & ES_MULTILINE) != 0) || cue_text.is_null() {
        return FALSE;
    }

    heap_free((*es).cue_banner_text as *mut core::ffi::c_void);
    (*es).cue_banner_text = heap_strdupW(cue_text);
    (*es).cue_banner_draw_focused = draw_focused;

    return TRUE;
}

unsafe extern "C" fn EDIT_EM_GetCueBanner(mut es: *mut EDITSTATE, mut buf: *mut WCHAR, mut size: DWORD) -> BOOL {
    if (*es).style & (ES_MULTILINE as DWORD) != 0 {
        return FALSE;
    }
    if (*es).cue_banner_text.is_null() {
        if !buf.is_null() && size != 0 {
            *buf = 0 as WCHAR;
        }
        return FALSE;
    } else {
        if !buf.is_null() {
            lstrcpynW(
                buf as LPWSTR,
                (*es).cue_banner_text as LPCWSTR,
                size as ::core::ffi::c_int,
            );
        }
        return TRUE;
    }
}

unsafe extern "C" fn EDIT_GetCompositionStr(
    mut hIMC: HIMC,
    mut CompFlag: LPARAM,
    mut es: *mut EDITSTATE,
) {
    let mut buflen: LONG = 0;
    let mut lpCompStr: LPWSTR = core::ptr::null_mut();
    let mut lpCompStrAttr: LPSTR = core::ptr::null_mut();
    let mut dwBufLenAttr: DWORD = 0;
    let _ = es;

    buflen = ImmGetCompositionStringW(
        hIMC,
        GCS_COMPSTR as DWORD,
        core::ptr::null_mut(),
        0 as DWORD,
    );

    if buflen < 0 as LONG {
        return;
    }

    lpCompStr = heap_alloc(buflen as SIZE_T) as LPWSTR;
    if lpCompStr.is_null() {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/win32/comctl32/edit.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"EDIT_GetCompositionStr\0" as *const u8 as *const ::core::ffi::c_char,
                4398 as ::core::ffi::c_int,
                b"Unable to allocate IME CompositionString\n\0" as *const u8
                    as *const ::core::ffi::c_char,
            );
        }
        return;
    }

    if buflen != 0 {
        ImmGetCompositionStringW(
            hIMC,
            GCS_COMPSTR as DWORD,
            lpCompStr as LPVOID,
            buflen as DWORD,
        );
    }

    if (CompFlag & (GCS_COMPATTR as LPARAM)) != 0 {
        dwBufLenAttr = ImmGetCompositionStringW(
            hIMC,
            GCS_COMPATTR as DWORD,
            core::ptr::null_mut(),
            0 as DWORD,
        ) as DWORD;
        if dwBufLenAttr != 0 {
            dwBufLenAttr = dwBufLenAttr.wrapping_add(1);
            lpCompStrAttr = heap_alloc(
                dwBufLenAttr.wrapping_add(1 as DWORD) as SIZE_T,
            ) as LPSTR;
            if lpCompStrAttr.is_null() {
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                    != 0
                {
                    let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl_0: __wine_debug_class = __WINE_DBCL_ERR;
                    ros_dbg_log(
                        __dbcl_0,
                        __dbch_0,
                        b"dll/win32/comctl32/edit.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"EDIT_GetCompositionStr\0" as *const u8
                            as *const ::core::ffi::c_char,
                        4418 as ::core::ffi::c_int,
                        b"Unable to allocate IME Attribute String\n\0" as *const u8
                            as *const ::core::ffi::c_char,
                    );
                }
                heap_free(lpCompStr as *mut ::core::ffi::c_void);
                return;
            }
            ImmGetCompositionStringW(
                hIMC,
                GCS_COMPATTR as DWORD,
                lpCompStrAttr as LPVOID,
                dwBufLenAttr,
            );
            *lpCompStrAttr.offset(dwBufLenAttr as isize) = 0 as CHAR;
        }
    }

    heap_free(lpCompStrAttr as *mut ::core::ffi::c_void);
    heap_free(lpCompStr as *mut ::core::ffi::c_void);
}

unsafe extern "C" fn EDIT_GetResultStr(hIMC: HIMC, es: *mut EDITSTATE) {
    let mut buflen: LONG;
    let mut lpResultStr: LPWSTR;

    buflen = ImmGetCompositionStringW(hIMC, GCS_RESULTSTR, core::ptr::null_mut(), 0);
    if buflen <= 0 {
        return;
    }

    lpResultStr = heap_alloc(buflen as SIZE_T) as LPWSTR;
    if lpResultStr.is_null() {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/win32/comctl32/edit.c\0"
                    .as_ptr() as *const ::core::ffi::c_char,
                b"EDIT_GetResultStr\0".as_ptr() as *const ::core::ffi::c_char,
                4466 as ::core::ffi::c_int,
                b"Unable to alloc buffer for IME string\n\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        return;
    }

    ImmGetCompositionStringW(hIMC, GCS_RESULTSTR, lpResultStr as LPVOID, buflen as DWORD);

    heap_free(lpResultStr as *mut ::core::ffi::c_void);
}

unsafe extern "C" fn EDIT_ImeComposition(hwnd: HWND, CompFlag: LPARAM, es: *mut EDITSTATE) {
    let mut hIMC: HIMC = core::ptr::null_mut();
    let mut cursor: core::ffi::c_int;
    if (*es).selection_start != (*es).selection_end {
        EDIT_EM_ReplaceSel(
            es,
            TRUE,
            core::ptr::null(),
            0,
            TRUE,
            TRUE,
        );
    }
    hIMC = ImmGetContext(hwnd);
    if hIMC.is_null() {
        return;
    }
    if (CompFlag & (GCS_RESULTSTR as LPARAM)) != 0 {
        EDIT_GetResultStr(hIMC, es);
        cursor = 0 as core::ffi::c_int;
    } else {
        if (CompFlag & (GCS_COMPSTR as LPARAM)) != 0 {
            EDIT_GetCompositionStr(hIMC, CompFlag, es);
        }
        cursor = 0 as core::ffi::c_int;
    }
    ImmReleaseContext(hwnd, hIMC);
    EDIT_SetCaretPos(
        es,
        (*es).selection_start + cursor as INT,
        (((*es).flags as BOOL & EF_AFTER_WRAP) != 0) as i32,
    );
}

unsafe extern "C" fn EDIT_WM_NCCreate(hwnd: HWND, lpcs: LPCREATESTRUCTW) -> LRESULT {
    let mut es: *mut EDITSTATE = core::ptr::null_mut();
    let mut alloc_size: UINT = 0;

    es = heap_alloc_zero(core::mem::size_of::<EDITSTATE>() as SIZE_T) as *mut EDITSTATE;
    if es.is_null() {
        return FALSE as LRESULT;
    }
    SetWindowLongPtrW(hwnd, 0 as ::core::ffi::c_int, es as LONG_PTR);

    (*es).style = (*lpcs).style as DWORD;

    (*es).bEnableState = (((*es).style & (WS_DISABLED as DWORD)) == 0) as ::core::ffi::c_int as BOOL;

    (*es).hwndSelf = hwnd;
    /* Save parent, which will be notified by EN_* messages */
    (*es).hwndParent = (*lpcs).hwndParent;

    if ((*es).style & (ES_COMBO as DWORD)) != 0 {
        (*es).hwndListBox = GetDlgItem((*es).hwndParent, ID_CB_LISTBOX as ::core::ffi::c_int);
    }

    /* FIXME: should we handle changes to WS_EX_RIGHT style after creation? */
    if ((*lpcs).dwExStyle & (WS_EX_RIGHT as DWORD)) != 0 {
        (*es).style |= ES_RIGHT as DWORD;
    }

    /* Number overrides lowercase overrides uppercase (at least it
     * does in Win95).  However I'll bet that ES_NUMBER would be
     * invalid under Win 3.1.
     */
    if ((*es).style & (ES_NUMBER as DWORD)) != 0 {
        /* do not override the ES_NUMBER */
    } else if ((*es).style & (ES_LOWERCASE as DWORD)) != 0 {
        (*es).style &= !(ES_UPPERCASE as DWORD);
    }

    if ((*es).style & (ES_MULTILINE as DWORD)) != 0 {
        (*es).buffer_limit = BUFLIMIT_INITIAL as UINT;
        if ((*es).style & (WS_VSCROLL as DWORD)) != 0 {
            (*es).style |= ES_AUTOVSCROLL as DWORD;
        }
        if ((*es).style & (WS_HSCROLL as DWORD)) != 0 {
            (*es).style |= ES_AUTOHSCROLL as DWORD;
        }
        (*es).style &= !(ES_PASSWORD as DWORD);
        if ((*es).style & (ES_CENTER as DWORD)) != 0
            || ((*es).style & (ES_RIGHT as DWORD)) != 0
        {
            /* Confirmed - RIGHT overrides CENTER */
            if ((*es).style & (ES_RIGHT as DWORD)) != 0 {
                (*es).style &= !(ES_CENTER as DWORD);
            }
            (*es).style &= !(WS_HSCROLL as DWORD);
            (*es).style &= !(ES_AUTOHSCROLL as DWORD);
        }
    } else {
        (*es).buffer_limit = BUFLIMIT_INITIAL as UINT;
        if ((*es).style & (ES_RIGHT as DWORD)) != 0
            && ((*es).style & (ES_CENTER as DWORD)) != 0
        {
            (*es).style &= !(ES_CENTER as DWORD);
        }
        (*es).style &= !(WS_HSCROLL as DWORD);
        (*es).style &= !(WS_VSCROLL as DWORD);
        if ((*es).style & (ES_PASSWORD as DWORD)) != 0 {
            (*es).password_char = '*' as i32 as WCHAR;
        }
    }

    alloc_size = ROUND_TO_GROW!(
        ((*es).buffer_size.wrapping_add(1 as UINT) as usize)
            .wrapping_mul(core::mem::size_of::<WCHAR>() as usize),
    ) as UINT;

    'cleanup: {
        (*es).hloc32W = LocalAlloc(
            (LMEM_MOVEABLE | LMEM_ZEROINIT) as UINT,
            alloc_size as SIZE_T,
        );
        if (*es).hloc32W.is_null() {
            break 'cleanup;
        }
        (*es).buffer_size = (LocalSize((*es).hloc32W) as usize)
            .wrapping_div(core::mem::size_of::<WCHAR>() as usize)
            .wrapping_sub(1 as usize) as UINT;

        (*es).undo_text = heap_alloc_zero(
            ((*es).buffer_size.wrapping_add(1 as UINT) as SIZE_T)
                .wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
        ) as LPWSTR;
        if (*es).undo_text.is_null() {
            break 'cleanup;
        }
        (*es).undo_buffer_size = (*es).buffer_size;

        if ((*es).style & (ES_MULTILINE as DWORD)) != 0 {
            (*es).first_line_def =
                heap_alloc_zero(core::mem::size_of::<LINEDEF>() as SIZE_T) as *mut LINEDEF;
            if (*es).first_line_def.is_null() {
                break 'cleanup;
            }
        }
        (*es).line_count = 1 as INT;

        /*
         * In Win95 look and feel, the WS_BORDER style is replaced by the
         * WS_EX_CLIENTEDGE style for the edit control. This gives the edit
         * control a nonclient area so we don't need to draw the border.
         * If WS_BORDER without WS_EX_CLIENTEDGE is specified we shouldn't have
         * a nonclient area and we should handle painting the border ourselves.
         *
         * When making modifications please ensure that the code still works
         * for edit controls created directly with style 0x50800000, exStyle 0
         * (which should have a single pixel border)
         */
        if ((*lpcs).dwExStyle & (WS_EX_CLIENTEDGE as DWORD)) != 0 {
            (*es).style &= !(WS_BORDER as DWORD);
        } else if ((*es).style & (WS_BORDER as DWORD)) != 0 {
            SetWindowLongW(
                hwnd,
                GWL_STYLE as ::core::ffi::c_int,
                ((*es).style & !(WS_BORDER as DWORD)) as LONG,
            );
        }

        return TRUE as LRESULT;
    }

    /* cleanup: */
    SetWindowLongPtrW((*es).hwndSelf, 0 as ::core::ffi::c_int, 0 as LONG_PTR);
    EDIT_InvalidateUniscribeData(es);
    heap_free((*es).first_line_def as *mut ::core::ffi::c_void);
    heap_free((*es).undo_text as *mut ::core::ffi::c_void);
    if !(*es).hloc32W.is_null() {
        LocalFree((*es).hloc32W);
    }
    heap_free((*es).logAttr as *mut ::core::ffi::c_void);
    heap_free(es as *mut ::core::ffi::c_void);
    return FALSE as LRESULT;
}

unsafe extern "C" fn EDIT_WM_Create(es: *mut EDITSTATE, name: *const WCHAR) -> LRESULT {
    let mut clientRect: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    if ((*__wine_dbch___default).flags as i32 & ((1i32) << __WINE_DBCL_TRACE as i32)) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/edit.c\0"
                as *const u8 as *const i8,
            b"EDIT_WM_Create\0" as *const u8 as *const i8,
            4650 as i32,
            b"%s\n\0" as *const u8 as *const i8,
            debugstr_w(name),
        );
    }
    EDIT_WM_SetFont(es, core::ptr::null_mut(), FALSE);
    EDIT_EM_EmptyUndoBuffer(es);

    /* We need to calculate the format rect
       (applications may send EM_SETMARGINS before the control gets visible) */
    GetClientRect((*es).hwndSelf, core::ptr::addr_of_mut!(clientRect));
    EDIT_SetRectNP(es, core::ptr::addr_of_mut!(clientRect));

    if !name.is_null() && *name != 0 {
        EDIT_EM_ReplaceSel(
            es,
            FALSE,
            name,
            lstrlenW(name) as UINT,
            FALSE,
            FALSE,
        );
        /* if we insert text to the editline, the text scrolls out
         * of the window, as the caret is placed after the insert
         * pos normally; thus we reset es->selection... to 0 and
         * update caret
         */
        (*es).selection_end = 0;
        (*es).selection_start = (*es).selection_end;
        /* Adobe Photoshop does NOT like this. and MSDN says that EN_CHANGE
         * Messages are only to be sent when the USER does something to
         * change the contents. So I am removing this EN_CHANGE
         *
         * EDIT_NOTIFY_PARENT(es, EN_CHANGE);
         */
        EDIT_EM_ScrollCaret(es);
    }

    /* force scroll info update */
    EDIT_UpdateScrollInfo(es);
    OpenThemeData((*es).hwndSelf, (WC_EDITW).as_ptr());

    /* The rule seems to return 1 here for success */
    /* Power Builder masked edit controls will crash  */
    /* if not. */
    /* FIXME: is that in all cases so ? */
    return 1;
}

unsafe extern "C" fn EDIT_WM_NCDestroy(es: *mut EDITSTATE) -> LRESULT {
    let mut pc: *mut LINEDEF = core::ptr::null_mut();
    let mut pp: *mut LINEDEF = core::ptr::null_mut();
    let mut theme: HTHEME;

    theme = GetWindowTheme((*es).hwndSelf);
    CloseThemeData(theme);

    /* The app can own the text buffer handle */
    if !(*es).hloc32W.is_null() && (*es).hloc32W != (*es).hlocapp {
        LocalFree((*es).hloc32W);
    }

    EDIT_InvalidateUniscribeData(es);

    pc = (*es).first_line_def;
    while !pc.is_null() {
        pp = (*pc).next as *mut LINEDEF;
        heap_free(pc as *mut core::ffi::c_void);
        pc = pp;
    }

    SetWindowLongPtrW(
        (*es).hwndSelf,
        0 as ::core::ffi::c_int,
        0 as LONG_PTR
    );
    heap_free((*es).undo_text as *mut core::ffi::c_void);
    heap_free((*es).cue_banner_text as *mut core::ffi::c_void);
    heap_free(es as *mut core::ffi::c_void);

    return 0;
}

unsafe extern "system" fn EDIT_WindowProc(
    mut hwnd: HWND,
    mut msg: UINT,
    mut wParam: WPARAM,
    mut lParam: LPARAM,
) -> LRESULT {
    let mut es: *mut EDITSTATE = GetWindowLongPtrW(hwnd, 0) as *mut EDITSTATE;
    let mut theme: HTHEME = GetWindowTheme(hwnd);
    let mut result: LRESULT = 0 as LRESULT;
    let mut rect: *mut RECT = core::ptr::null_mut();

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/edit.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"EDIT_WindowProc\0" as *const u8 as *const ::core::ffi::c_char,
            4738 as ::core::ffi::c_int,
            b"hwnd=%p msg=%#x wparam=%lx lparam=%lx\n\0" as *const u8 as *const ::core::ffi::c_char,
            hwnd,
            msg,
            wParam,
            lParam,
        );
    }

    if es.is_null() && msg != WM_NCCREATE {
        return DefWindowProcW(hwnd, msg, wParam, lParam);
    }

    if !es.is_null() && msg != WM_NCDESTROY {
        EDIT_LockBuffer(es);
    }

    'switch: {
        match msg {
            EM_GETSEL => {
                result = EDIT_EM_GetSel(es, wParam as *mut UINT, lParam as *mut UINT) as LRESULT;
            }

            EM_SETSEL => {
                EDIT_EM_SetSel(es, wParam as UINT, lParam as UINT, FALSE);
                EDIT_EM_ScrollCaret(es);
                result = 1 as LRESULT;
            }

            EM_GETRECT => {
                rect = lParam as *mut RECT;
                if !rect.is_null() {
                    *rect = (*es).format_rect;
                }
            }

            EM_SETRECT => {
                if ((*es).style & ES_MULTILINE as DWORD) != 0 && lParam != 0 {
                    EDIT_SetRectNP(es, lParam as *mut RECT);
                    EDIT_UpdateText(es, core::ptr::null(), TRUE);
                }
            }

            EM_SETRECTNP => {
                if ((*es).style & ES_MULTILINE as DWORD) != 0 && lParam != 0 {
                    EDIT_SetRectNP(es, lParam as LPRECT);
                }
            }

            EM_SCROLL => {
                result = EDIT_EM_Scroll(es, wParam as INT) as LRESULT;
            }

            EM_LINESCROLL => {
                result = EDIT_EM_LineScroll(es, wParam as INT, lParam as INT) as LRESULT;
            }

            EM_SCROLLCARET => {
                EDIT_EM_ScrollCaret(es);
                result = 1 as LRESULT;
            }

            EM_GETMODIFY => {
                result = (((*es).flags as ::core::ffi::c_int & EF_MODIFIED)
                    != 0) as LRESULT;
            }

            EM_SETMODIFY => {
                if wParam != 0 {
                    (*es).flags |= EF_MODIFIED as WORD;
                } else {
                    (*es).flags &= !((EF_MODIFIED | EF_UPDATE) as WORD); /* reset pending updates */
                }
            }

            EM_GETLINECOUNT => {
                result = (if ((*es).style & ES_MULTILINE as DWORD) != 0 {
                    (*es).line_count as LRESULT
                } else {
                    1 as LRESULT
                }) as LRESULT;
            }

            EM_LINEINDEX => {
                result = EDIT_EM_LineIndex(es, wParam as INT) as LRESULT;
            }

            EM_SETHANDLE => {
                EDIT_EM_SetHandle(es, wParam as HLOCAL);
            }

            EM_GETHANDLE => {
                result = EDIT_EM_GetHandle(es) as LRESULT;
            }

            EM_GETTHUMB => {
                result = EDIT_EM_GetThumb(es) as LRESULT;
            }

            /* these messages missing from specs */
            0x00bf | 0x00c0 | 0x00c3 | 0x00ca => {
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
                    != 0
                {
                    let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl_0: __wine_debug_class = __WINE_DBCL_FIXME;
                    ros_dbg_log(
                        __dbcl_0,
                        __dbch_0,
                        b"dll/win32/comctl32/edit.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"EDIT_WindowProc\0" as *const u8 as *const ::core::ffi::c_char,
                        4826 as ::core::ffi::c_int,
                        b"undocumented message 0x%x, please report\n\0" as *const u8
                            as *const ::core::ffi::c_char,
                        msg,
                    );
                }
                result = DefWindowProcW(hwnd, msg, wParam, lParam) as LRESULT;
            }

            EM_LINELENGTH => {
                result = EDIT_EM_LineLength(es, wParam as INT) as LRESULT;
            }

            EM_REPLACESEL => {
                let mut textW: *const WCHAR = lParam as *const WCHAR;

                EDIT_EM_ReplaceSel(
                    es,
                    wParam as BOOL,
                    textW,
                    lstrlenW(textW as LPCWSTR) as UINT,
                    TRUE,
                    TRUE,
                );
                result = 1 as LRESULT;
            }

            EM_GETLINE => {
                result = EDIT_EM_GetLine(es, wParam as INT, lParam as LPWSTR) as LRESULT;
            }

            EM_SETLIMITTEXT => {
                EDIT_EM_SetLimitText(es, wParam as UINT);
            }

            EM_CANUNDO => {
                result = EDIT_EM_CanUndo(es) as LRESULT;
            }

            EM_UNDO | WM_UNDO => {
                result = EDIT_EM_Undo(es) as LRESULT;
            }

            EM_FMTLINES => {
                result = EDIT_EM_FmtLines(es, wParam as BOOL) as LRESULT;
            }

            EM_LINEFROMCHAR => {
                result = EDIT_EM_LineFromChar(es, wParam as INT) as LRESULT;
            }

            EM_SETTABSTOPS => {
                result = EDIT_EM_SetTabStops(es, wParam as INT, lParam as LPINT) as LRESULT;
            }

            EM_SETPASSWORDCHAR => {
                EDIT_EM_SetPasswordChar(es, (wParam as UINT) as WCHAR);
            }

            EM_EMPTYUNDOBUFFER => {
                EDIT_EM_EmptyUndoBuffer(es);
            }

            EM_GETFIRSTVISIBLELINE => {
                result = (if ((*es).style & ES_MULTILINE as DWORD) != 0 {
                    (*es).y_offset
                } else {
                    (*es).x_offset
                }) as LRESULT;
            }

            EM_SETREADONLY => {
                let mut old_style: DWORD = (*es).style;

                if wParam != 0 {
                    SetWindowLongW(
                        hwnd,
                        GWL_STYLE,
                        GetWindowLongW(hwnd, GWL_STYLE) | (ES_READONLY) as i32,
                    );
                    (*es).style |= ES_READONLY as DWORD;
                } else {
                    SetWindowLongW(
                        hwnd,
                        GWL_STYLE,
                        GetWindowLongW(hwnd, GWL_STYLE) & (!(ES_READONLY)) as i32,
                    );
                    (*es).style &= !(ES_READONLY as DWORD);
                }

                if (old_style ^ (*es).style) != 0 {
                    InvalidateRect((*es).hwndSelf, core::ptr::null_mut(), TRUE);
                }

                result = 1 as LRESULT;
            }

            EM_SETWORDBREAKPROC => {
                EDIT_EM_SetWordBreakProc(
                    es,
                    *(core::ptr::addr_of!(lParam) as *const EDITWORDBREAKPROCW),
                );
                result = 1 as LRESULT;
            }

            EM_GETWORDBREAKPROC => {
                result = *(core::ptr::addr_of!((*es).word_break_proc) as *const LRESULT);
            }

            EM_GETPASSWORDCHAR => {
                result = (*es).password_char as LRESULT;
            }

            EM_SETMARGINS => {
                EDIT_EM_SetMargins(
                    es,
                    wParam as INT,
                    LOWORD!(lParam),
                    HIWORD!(lParam),
                    TRUE,
                );
            }

            EM_GETMARGINS => {
                result = MAKELONG!((*es).left_margin, (*es).right_margin) as LRESULT;
            }

            EM_GETLIMITTEXT => {
                result = (*es).buffer_limit as LRESULT;
            }

            EM_POSFROMCHAR => {
                if wParam as INT as UINT >= get_text_length(es) {
                    result = -(1) as LRESULT;
                } else {
                    result = EDIT_EM_PosFromChar(es, wParam as INT, FALSE) as LRESULT;
                }
            }

            EM_CHARFROMPOS => {
                result = EDIT_EM_CharFromPos(
                    es,
                    LOWORD!(lParam) as i16 as INT,
                    HIWORD!(lParam) as i16 as INT,
                ) as LRESULT;
            }

            EM_SETCUEBANNER => {
                result = EDIT_EM_SetCueBanner(es, wParam as BOOL, lParam as *const WCHAR) as LRESULT;
            }

            EM_GETCUEBANNER => {
                result = EDIT_EM_GetCueBanner(es, wParam as *mut WCHAR, lParam as DWORD) as LRESULT;
            }

            /* End of the EM_ messages which were in numerical order; what order
             * are these in?  vaguely alphabetical?
             */

            WM_NCCREATE => {
                result = EDIT_WM_NCCreate(hwnd, lParam as LPCREATESTRUCTW) as LRESULT;
            }

            WM_NCDESTROY => {
                result = EDIT_WM_NCDestroy(es) as LRESULT;
                es = core::ptr::null_mut();
            }

            WM_GETDLGCODE => {
                result = (DLGC_HASSETSEL | DLGC_WANTCHARS | DLGC_WANTARROWS) as LRESULT;

                if ((*es).style & ES_MULTILINE as DWORD) != 0 {
                    result |= DLGC_WANTALLKEYS as LRESULT;
                }

                if lParam != 0 {
                    let msg: *mut MSG = lParam as *mut MSG;
                    (*es).flags |= EF_DIALOGMODE as WORD;

                    if (*msg).message == WM_KEYDOWN as UINT {
                        let vk: ::core::ffi::c_int = (*msg).wParam as ::core::ffi::c_int;

                        if !(*es).hwndListBox.is_null() {
                            if (vk) as u32 == VK_RETURN || (vk) as u32 == VK_ESCAPE {
                                if SendMessageW(
                                    GetParent(hwnd),
                                    CB_GETDROPPEDSTATE,
                                    0,
                                    0,
                                ) != 0
                                {
                                    result |= DLGC_WANTMESSAGE as LRESULT;
                                }
                            }
                        }
                    }
                }
            }

            WM_IME_CHAR => {
                result = DefWindowProcW(hwnd, msg, wParam, lParam) as LRESULT;
            }

            WM_CHAR => {
                let mut charW: WCHAR = wParam as WCHAR;

                if !(*es).hwndListBox.is_null() {
                    if (charW as ::core::ffi::c_int) as u32 == VK_RETURN
                        || (charW as ::core::ffi::c_int) as u32 == VK_ESCAPE
                    {
                        if SendMessageW(
                            GetParent(hwnd),
                            CB_GETDROPPEDSTATE,
                            0,
                            0,
                        ) != 0
                        {
                            SendMessageW(
                                GetParent(hwnd),
                                WM_KEYDOWN,
                                charW as WPARAM,
                                0,
                            );
                        }
                        break 'switch;
                    }
                }
                result = EDIT_WM_Char(es, charW) as LRESULT;
            }

            WM_UNICHAR => {
                if wParam == UNICODE_NOCHAR as WPARAM {
                    return TRUE as LRESULT;
                }
                if wParam <= 0x000fffff as WPARAM {
                    if wParam > 0xffff as WPARAM {
                        /* convert to surrogates */
                        wParam -= 0x10000 as WPARAM;
                        EDIT_WM_Char(
                            es,
                            ((wParam >> 10) + 0xd800 as WPARAM) as WCHAR,
                        );
                        EDIT_WM_Char(
                            es,
                            ((wParam & 0x03ff as WPARAM) + 0xdc00 as WPARAM) as WCHAR,
                        );
                    } else {
                        EDIT_WM_Char(es, wParam as WCHAR);
                    }
                }
                return 0 as LRESULT;
            }

            WM_CLEAR => {
                EDIT_WM_Clear(es);
            }

            WM_CONTEXTMENU => {
                EDIT_WM_ContextMenu(es, LOWORD!(lParam) as i16 as INT, HIWORD!(lParam) as i16 as INT);
            }

            WM_COPY => {
                EDIT_WM_Copy(es);
            }

            WM_CREATE => {
                result = EDIT_WM_Create(
                    es,
                    (*(lParam as LPCREATESTRUCTW)).lpszName as *const WCHAR,
                ) as LRESULT;
            }

            WM_CUT => {
                EDIT_WM_Cut(es);
            }

            WM_ENABLE => {
                (*es).bEnableState = wParam as BOOL;
                EDIT_UpdateText(es, core::ptr::null(), TRUE);
                if !theme.is_null() {
                    RedrawWindow(
                        hwnd,
                        core::ptr::null(),
                        core::ptr::null_mut(),
                        RDW_FRAME | RDW_INVALIDATE | RDW_UPDATENOW,
                    );
                }
            }

            WM_ERASEBKGND => {
                /* we do the proper erase in EDIT_WM_Paint */
                result = 1 as LRESULT;
            }

            WM_GETFONT => {
                result = (*es).font as LRESULT;
            }

            WM_GETTEXT => {
                result = EDIT_WM_GetText(es, wParam as INT, lParam as LPWSTR) as LRESULT;
            }

            WM_GETTEXTLENGTH => {
                result = get_text_length(es) as LRESULT;
            }

            WM_HSCROLL => {
                result = EDIT_WM_HScroll(es, (LOWORD!(wParam)) as INT, (HIWORD!(wParam)) as INT) as LRESULT;
            }

            WM_KEYDOWN => {
                result = EDIT_WM_KeyDown(es, wParam as INT) as LRESULT;
            }

            WM_KILLFOCUS => {
                result = EDIT_WM_KillFocus(theme, es) as LRESULT;
            }

            WM_LBUTTONDBLCLK => {
                result = EDIT_WM_LButtonDblClk(es) as LRESULT;
            }

            WM_LBUTTONDOWN => {
                result = EDIT_WM_LButtonDown(
                    es,
                    (wParam) as DWORD,
                    (LOWORD!(lParam)) as INT,
                    (HIWORD!(lParam)) as INT,
                ) as LRESULT;
            }

            WM_LBUTTONUP => {
                result = EDIT_WM_LButtonUp(es) as LRESULT;
            }

            WM_MBUTTONDOWN => {
                result = EDIT_WM_MButtonDown(es) as LRESULT;
            }

            WM_MOUSEMOVE => {
                result = EDIT_WM_MouseMove(
                    es,
                    LOWORD!(lParam) as i16 as INT,
                    HIWORD!(lParam) as i16 as INT,
                ) as LRESULT;
            }

            WM_PRINTCLIENT | WM_PAINT => {
                EDIT_WM_Paint(es, wParam as HDC);
            }

            WM_NCPAINT => {
                EDIT_WM_NCPaint(hwnd, wParam as HRGN);
            }

            WM_PASTE => {
                EDIT_WM_Paste(es);
            }

            WM_SETFOCUS => {
                EDIT_WM_SetFocus(theme, es);
            }

            WM_SETFONT => {
                EDIT_WM_SetFont(
                    es,
                    wParam as HFONT,
                    (LOWORD!(lParam) != 0) as BOOL,
                );
            }

            WM_SETREDRAW => {
                /* FIXME: actually set an internal flag and behave accordingly */
            }

            WM_SETTEXT => {
                EDIT_WM_SetText(es, lParam as *const WCHAR);
                result = TRUE as LRESULT;
            }

            WM_SIZE => {
                EDIT_WM_Size(es, wParam as UINT);
            }

            WM_STYLECHANGED => {
                result = EDIT_WM_StyleChanged(es, wParam, lParam as *const STYLESTRUCT) as LRESULT;
            }

            WM_STYLECHANGING => {
                result = 0 as LRESULT; /* See EDIT_WM_StyleChanged */
            }

            WM_SYSKEYDOWN => {
                result = EDIT_WM_SysKeyDown(es, wParam as INT, lParam as DWORD) as LRESULT;
            }

            WM_TIMER => {
                EDIT_WM_Timer(es);
            }

            WM_VSCROLL => {
                let mut si: SCROLLINFO = core::mem::zeroed::<SCROLLINFO>();

                si.cbSize = core::mem::size_of::<SCROLLINFO>() as UINT;
                si.fMask = SIF_TRACKPOS as UINT;
                if GetScrollInfo(hwnd, SB_VERT as ::core::ffi::c_int, core::ptr::addr_of_mut!(si)) == 0 {
                    result = 1 as LRESULT;
                    break 'switch;
                }
                result = EDIT_WM_VScroll(es, (LOWORD!(wParam)) as INT, si.nTrackPos) as LRESULT;
            }

            WM_MOUSEWHEEL => {
                let mut wheelDelta: ::core::ffi::c_int;
                let mut pulScrollLines: INT = 3;
                SystemParametersInfoW(
                    SPI_GETWHEELSCROLLLINES,
                    0,
                    core::ptr::addr_of_mut!(pulScrollLines) as PVOID,
                    0,
                );

                if (wParam & ((MK_SHIFT | MK_CONTROL) as WPARAM)) != 0 {
                    result = DefWindowProcW(hwnd, msg, wParam, lParam) as LRESULT;
                    break 'switch;
                }

                wheelDelta = GET_WHEEL_DELTA_WPARAM!(wParam) as ::core::ffi::c_int;
                /* if scrolling changes direction, ignore left overs */
                if (wheelDelta < 0 && (*es).wheelDeltaRemainder < 0)
                    || (wheelDelta > 0 && (*es).wheelDeltaRemainder > 0)
                {
                    (*es).wheelDeltaRemainder += wheelDelta as INT;
                } else {
                    (*es).wheelDeltaRemainder = wheelDelta as INT;
                }

                if (*es).wheelDeltaRemainder != 0 && pulScrollLines != 0 {
                    let mut cLineScroll: ::core::ffi::c_int;
                    pulScrollLines = min((*es).line_count, pulScrollLines);
                    cLineScroll = pulScrollLines * (*es).wheelDeltaRemainder / (WHEEL_DELTA) as i32;
                    (*es).wheelDeltaRemainder -= 120 /* WHEEL_DELTA */ * cLineScroll / pulScrollLines;
                    result = EDIT_EM_LineScroll(es, 0, -cLineScroll) as LRESULT;
                }
            }

            /* IME messages to make the edit control IME aware */
            WM_IME_SETCONTEXT => {
                let hKL: HKL = GetKeyboardLayout(0);

                /* Korean doesn't want composition window */
                if (PRIMARYLANGID!(LOWORD!(hKL as DWORD_PTR))) as u32 == LANG_KOREAN {
                    lParam &= !(ISC_SHOWUICOMPOSITIONWINDOW as LPARAM);
                }

                if wParam != 0 {
                    let hIMC: HIMC = ImmGetContext(hwnd);
                    let pIC: LPINPUTCONTEXTDX = ImmLockIMC(hIMC) as LPINPUTCONTEXTDX;
                    if !pIC.is_null() {
                        (*pIC).dwUIFlags &= !(0x40000 as DWORD);
                        ImmUnlockIMC(hIMC);
                    }
                    if FALSE != 0 {
                        /* FIXME: Condition */
                        ImmNotifyIME(hIMC, NI_COMPOSITIONSTR, CPS_CANCEL, 0);
                    }
                    ImmReleaseContext(hwnd, hIMC);
                }

                result = DefWindowProcW(hwnd, msg, wParam, lParam) as LRESULT;
            }

            WM_IME_STARTCOMPOSITION => {
                let hKL: HKL = GetKeyboardLayout(0);

                /* Korean doesn't want composition window */
                if (PRIMARYLANGID!(LOWORD!(hKL as DWORD_PTR))) as u32 == LANG_KOREAN {
                    return TRUE as LRESULT;
                }

                result = DefWindowProcW(hwnd, msg, wParam, lParam) as LRESULT;
            }

            WM_IME_COMPOSITION => {
                EDIT_ImeComposition(hwnd, lParam, es);
                result = DefWindowProcW(hwnd, msg, wParam, lParam) as LRESULT;
            }

            WM_IME_ENDCOMPOSITION => {
                result = DefWindowProcW(hwnd, msg, wParam, lParam) as LRESULT;
            }

            WM_IME_COMPOSITIONFULL => {}

            WM_IME_SELECT => {
                result = DefWindowProcW(hwnd, msg, wParam, lParam) as LRESULT;
            }

            WM_IME_CONTROL => {
                result = DefWindowProcW(hwnd, msg, wParam, lParam) as LRESULT;
            }

            WM_IME_REQUEST => {
                match wParam {
                    _ if wParam == IMR_QUERYCHARPOSITION as WPARAM => {
                        let chpos: *mut IMECHARPOSITION = lParam as *mut IMECHARPOSITION;
                        let pos: LRESULT = EDIT_EM_PosFromChar(
                            es,
                            ((*es).selection_start as DWORD).wrapping_add((*chpos).dwCharPos) as INT,
                            FALSE,
                        );
                        (*chpos).pt.x = LOWORD!(pos as DWORD_PTR) as LONG;
                        (*chpos).pt.y = HIWORD!(pos as DWORD_PTR) as LONG;
                        (*chpos).cLineHeight = (*es).line_height as UINT;
                        (*chpos).rcDocument = (*es).format_rect;
                        MapWindowPoints(
                            hwnd,
                            core::ptr::null_mut(),
                            core::ptr::addr_of_mut!((*chpos).pt),
                            1,
                        );
                        MapWindowPoints(
                            hwnd,
                            core::ptr::null_mut(),
                            core::ptr::addr_of_mut!((*chpos).rcDocument) as *mut POINT,
                            2,
                        );
                        result = 1 as LRESULT;
                    }
                    _ => {}
                }
            }

            WM_THEMECHANGED => {
                CloseThemeData(theme);
                OpenThemeData(hwnd, (WC_EDITW).as_ptr());
            }

            _ => {
                result = DefWindowProcW(hwnd, msg, wParam, lParam) as LRESULT;
            }
        }
    }

    if IsWindow(hwnd) != 0 && !es.is_null() && msg != EM_GETHANDLE {
        EDIT_UnlockBuffer(es, FALSE);
    }

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_1: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl_1,
            __dbch_1,
            b"dll/win32/comctl32/edit.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"EDIT_WindowProc\0" as *const u8 as *const ::core::ffi::c_char,
            5314 as ::core::ffi::c_int,
            b"hwnd=%p msg=%x -- 0x%08lx\n\0" as *const u8 as *const ::core::ffi::c_char,
            hwnd,
            msg,
            result,
        );
    }

    return result;
}

#[no_mangle]
pub unsafe extern "C" fn EDIT_Register() {
    let mut wndClass: WNDCLASSW = core::mem::zeroed();

    memset(
        core::ptr::addr_of_mut!(wndClass) as *mut c_void,
        0 as c_int,
        core::mem::size_of::<WNDCLASSW>() as size_t,
    );
    wndClass.style = (CS_PARENTDC | CS_GLOBALCLASS | CS_DBLCLKS) as UINT;
    wndClass.lpfnWndProc = Some(EDIT_WindowProc);
    wndClass.cbClsExtra = 0 as c_int;
    #[cfg(target_arch = "x86")]
    {
        wndClass.cbWndExtra = (core::mem::size_of::<*mut EDITSTATE>()
            + core::mem::size_of::<WORD>()) as c_int;
    }
    #[cfg(not(target_arch = "x86"))]
    {
        wndClass.cbWndExtra = core::mem::size_of::<*mut EDITSTATE>() as c_int;
    }
    wndClass.hCursor = LoadCursorW(
        core::ptr::null_mut(),
        (IDC_IBEAM as LPWSTR),
    );
    wndClass.hbrBackground = core::ptr::null_mut();
    wndClass.lpszClassName = (WC_EDITW).as_ptr();
    RegisterClassW(core::ptr::addr_of_mut!(wndClass));
}
#[no_mangle]
pub unsafe extern "C" fn EDIT_Unregister() {
    UnregisterClassW((WC_EDITW).as_ptr(), core::ptr::null_mut());
}
