use ntdll_sys::nt::*;
use core::ffi::c_void;
use crate::conutils::stream::CON_STREAM;
use crate::conutils::outstream::CONSOLE_SCREEN_BUFFER_INFO;
use crate::conutils::screen::CON_SCREEN;

pub const CP_SHIFTJIS: u32 = 932;
pub const CP_HANGUL: u32 = 949;
pub const CP_JOHAB: u32 = 1361;
pub const CP_GB2312: u32 = 936;
pub const CP_BIG5: u32 = 950;

// CON_PAGER flags
pub const CON_PAGER_EXPAND_TABS: u32 = 1 << 0;
pub const CON_PAGER_EXPAND_FF: u32 = 1 << 1;
pub const CON_PAGER_CACHE_INCOMPLETE_LINE: u32 = 1 << 2;

// `cch` is SIZE_T (pager.h): declared u32 here, the 64-bit callers left the upper half of the
// register undefined for a callee (more's MorePagerLine) that reads all of it.
pub type CON_PAGER_LINE_FN = Option<unsafe extern "system" fn(Pager: *mut CON_PAGER, line: *const u16, cch: usize) -> i32>;
pub type PAGE_PROMPT = Option<unsafe extern "system" fn(Pager: *mut CON_PAGER, Done: u32, Total: u32) -> i32>;

#[repr(C)]
pub struct CON_PAGER {
    pub Screen: *mut CON_SCREEN,
    pub PageColumns: u32,
    pub PageRows: u32,
    pub PagerLine: CON_PAGER_LINE_FN,
    pub dwFlags: u32,
    pub nTabWidth: i32,
    pub ScrollRows: u32,
    pub CachedLine: *mut u16,
    pub cchCachedLine: usize,
    pub ich: usize,
    pub CurrentLine: *mut u16,
    pub ichCurr: usize,
    pub iEndLine: usize,
    pub nSpacePending: u32,
    pub iColumn: u32,
    pub iLine: u32,
    pub lineno: u32,
}

extern "system" {
    fn WideCharToMultiByte(CodePage: u32, dwFlags: u32, lpWideCharStr: *const u16, cchWideChar: i32, lpMultiByteStr: *mut u8, cbMultiByte: i32, lpDefaultChar: *const u8, lpUsedDefaultChar: *mut i32) -> i32;
    fn GetProcessHeap() -> *mut c_void;
    fn HeapAlloc(hHeap: *mut c_void, dwFlags: u32, dwBytes: usize) -> *mut c_void;
    fn HeapFree(hHeap: *mut c_void, dwFlags: u32, lpMem: *mut c_void) -> i32;
    fn HeapReAlloc(hHeap: *mut c_void, dwFlags: u32, lpMem: *mut c_void, dwBytes: usize) -> *mut c_void;
    fn SetLastError(dwErrCode: u32);
    fn GetConsoleOutputCP() -> u32;
    fn K32LoadStringW(hInstance: HINSTANCE, uID: u32, lpBuffer: PWSTR, cchBufferMax: i32) -> i32;
}
unsafe extern "C" {
    fn ConGetScreenInfo(screen: *mut CON_SCREEN, csbi: *mut CONSOLE_SCREEN_BUFFER_INFO) -> i32;
}
unsafe extern "C" {
    fn ConStreamWrite(Stream: *mut CON_STREAM, Buffer: *const u16, nLength: u32) -> i32;
}

extern "C" {
    fn wcslen(s: *const u16) -> usize;
}

unsafe fn GetWidthOfCharCJK(nCodePage: u32, ch: u16) -> i32 {
    let mut ret = WideCharToMultiByte(
        nCodePage,
        0,
        core::ptr::addr_of!(ch) as *const u16,
        1,
        core::ptr::null_mut(),
        0,
        core::ptr::null::<u8>(),
        core::ptr::null_mut::<i32>(),
    );
    if ret == 0 {
        ret = 1;
    } else if ret > 2 {
        ret = 2;
    }
    ret
}

unsafe fn GetNextLine(Pager: *mut CON_PAGER, TextBuff: *const u16, cch: usize) -> i32 {
    let mut ich: usize = (*Pager).ich;
    let mut ichStart: usize = 0;
    let mut cchLine: usize = 0;
    let bCacheLine: i32;

    (*Pager).ichCurr = 0;
    (*Pager).iEndLine = 0;

    if !(*Pager).CurrentLine.is_null() {
        if !(*Pager).CachedLine.is_null() {
            HeapFree(GetProcessHeap(), 0, (*Pager).CachedLine as *mut c_void);
            (*Pager).CachedLine = core::ptr::null_mut();
            (*Pager).cchCachedLine = 0;
        }
        (*Pager).CurrentLine = core::ptr::null_mut();
    }

    if ich >= cch {
        if !(*Pager).CachedLine.is_null() {
            // goto TerminateLine
        } else {
            return 0;
        }
    } else {
        ichStart = ich;

        while ich < cch {
            if *TextBuff.add(ich) == 0x000A {
                ich += 1;
                break;
            }
            ich += 1;
        }
        (*Pager).ich = ich;

        cchLine = ich - ichStart;

        bCacheLine = if ((*Pager).dwFlags & CON_PAGER_CACHE_INCOMPLETE_LINE) != 0
            && ich >= cch
            && *TextBuff.add(ich.wrapping_sub(1)) != 0x000A
        {
            1
        } else {
            0
        };

        if bCacheLine != 0 && (*Pager).CachedLine.is_null() {
            let alloc_size = cchLine * core::mem::size_of::<u16>();
            (*Pager).CachedLine = HeapAlloc(
                GetProcessHeap(),
                8, // HEAP_ZERO_MEMORY
                alloc_size,
            ) as *mut u16;
            (*Pager).cchCachedLine = 0;

            if (*Pager).CachedLine.is_null() {
                SetLastError(8); // ERROR_NOT_ENOUGH_MEMORY
                return 0;
            }
        } else if !(*Pager).CachedLine.is_null() {
            let ptr = HeapReAlloc(
                GetProcessHeap(),
                8, // HEAP_ZERO_MEMORY
                (*Pager).CachedLine as *mut c_void,
                ((*Pager).cchCachedLine + cchLine) * core::mem::size_of::<u16>(),
            );
            if ptr.is_null() {
                HeapFree(GetProcessHeap(), 0, (*Pager).CachedLine as *mut c_void);
                (*Pager).CachedLine = core::ptr::null_mut();
                (*Pager).cchCachedLine = 0;
                SetLastError(8); // ERROR_NOT_ENOUGH_MEMORY
                return 0;
            }
            (*Pager).CachedLine = ptr as *mut u16;
        }

        if !(*Pager).CachedLine.is_null() {
            core::ptr::copy_nonoverlapping(
                TextBuff.add(ichStart),
                (*Pager).CachedLine.add((*Pager).cchCachedLine),
                cchLine,
            );
            (*Pager).cchCachedLine += cchLine;
        }

        if bCacheLine != 0 {
            return 0;
        }
    }

    // TerminateLine:
    (*Pager).ichCurr = 0;
    if !(*Pager).CachedLine.is_null() {
        (*Pager).iEndLine = (*Pager).cchCachedLine;
        (*Pager).CurrentLine = (*Pager).CachedLine;
    } else {
        (*Pager).iEndLine = cchLine;
        (*Pager).CurrentLine = TextBuff.add(ichStart) as *mut u16;
    }

    if (*Pager).iEndLine > 0 && *((*Pager).CurrentLine.add((*Pager).iEndLine.wrapping_sub(1))) == 0x000A {
        (*Pager).lineno += 1;
    }

    1
}

unsafe fn ConPagerWorker(
    pager: *mut CON_PAGER,
    text_buff: *const u16,
    cch: u32,
) -> i32 {
    let page_columns: u32 = (*pager).PageColumns;
    let scroll_rows: u32 = (*pager).ScrollRows;
    let b_finite_paging: i32 = if (page_columns > 0) && ((*pager).PageRows > 0) { 1 } else { 0 };
    let mut n_tab_width: i32 = (*pager).nTabWidth;

    let mut line: *mut u16 = (*pager).CurrentLine;
    let mut ich_start: usize = (*pager).ichCurr;
    let mut i_end_line: usize = (*pager).iEndLine;
    let mut i_column: u32 = (*pager).iColumn;

    let n_code_page: u32 = GetConsoleOutputCP();
    let is_cjk: i32 = if n_code_page == CP_SHIFTJIS || n_code_page == CP_HANGUL || n_code_page == CP_BIG5 || n_code_page == CP_GB2312  { 1 } else { 0 };
    let mut n_width_of_char: u32 = 1;
    let mut is_double_width_char_trailing: i32 = 0;

    if page_columns > 0 {
        if n_tab_width < 0 {
            n_tab_width = (page_columns - 1) as i32;
        } else if n_tab_width >= (page_columns as i32 - 1) {
            n_tab_width = page_columns as i32 - 1;
        }
    } else {
        if n_tab_width < 0 {
            n_tab_width = 8;
        }
    }

    let mut goto_expand_tab: bool = false;
    'process: loop {
        // ProcessLine: the C label's first action is the stop-check, BEFORE the
        // ExpandTab block below (which C reaches via `goto ExpandTab` only AFTER
        // this check). Each `continue` here maps to C's `goto ProcessLine`, so the
        // page-limit must be re-tested before any further tab expansion.
        if b_finite_paging != 0 && (*pager).iLine >= scroll_rows {
            break;
        }

        // ExpandTab
        if (*pager).nSpacePending > 0 || goto_expand_tab {
            goto_expand_tab = false;
            while (*pager).nSpacePending > 0 {
                let space: [u16; 2] = [0x0020, 0];
                ConStreamWrite((*(*pager).Screen).Stream, space.as_ptr() as *const u16, 1);
                (*pager).nSpacePending -= 1;
                i_column += 1;

                if (page_columns > 0) && (i_column % page_columns == 0) {
                    if !(b_finite_paging != 0) || (page_columns < (*(*pager).Screen).csbi.dwSize.X as u32) {
                        let newline: [u16; 2] = [0x000A, 0];
                        ConStreamWrite((*(*pager).Screen).Stream, newline.as_ptr() as *const u16, 1);
                    }
                    (*pager).iLine += 1;
                    continue 'process; // C: goto ProcessLine — re-check stop before more expansion
                }
            }
        }

        if line.is_null() || (ich_start >= i_end_line) {
            if GetNextLine(pager, text_buff, cch as usize) == 0 {
                break;
            }
            line = (*pager).CurrentLine;
            ich_start = (*pager).ichCurr;
            i_end_line = (*pager).iEndLine;
        }

        (*pager).iColumn = i_column;
        {
            let pager_line_fn = (*pager).PagerLine;
            if let Some(f) = pager_line_fn {
                if f(pager, line.add(ich_start) as *const u16, (i_end_line - ich_start) as usize) != 0 {
                    i_column = (*pager).iColumn;
                    (*pager).nSpacePending = 0;
                    ich_start = i_end_line;
                    continue;
                }
            }
        }

        let mut ich: usize = ich_start;
        while ich < i_end_line {
            let ch: u16 = *line.add(ich);
            if ch == 0x000A {
                break;
            }
            if ch == 0x0009 && ((*pager).dwFlags & CON_PAGER_EXPAND_TABS) != 0 {
                break;
            }
            if ch == 0x000C && ((*pager).dwFlags & CON_PAGER_EXPAND_FF) != 0 {
                break;
            }

            if is_cjk != 0 {
                n_width_of_char = GetWidthOfCharCJK(n_code_page, ch) as u32;
            }

            if (page_columns > 0) && (is_cjk != 0) {
                is_double_width_char_trailing = if (n_width_of_char == 2) && ((i_column + 1) % page_columns == 0) { 1 } else { 0 };
                if is_double_width_char_trailing != 0 {
                    i_column += 1;
                    break;
                }
            }

            i_column += n_width_of_char;

            if (page_columns > 0) && (i_column % page_columns == 0) {
                ich += 1;
                break;
            }
            ich += 1;
        }

        if ich - ich_start > 0 {
            ConStreamWrite((*(*pager).Screen).Stream, line.add(ich_start) as *const u16, (ich - ich_start) as u32);
        }

        if ich >= i_end_line {
            ich_start = ich;
            continue;
        }

        let ch: u16 = *line.add(ich);
        if ch == 0x000A {
            let newline: [u16; 2] = [0x000A, 0];
            ConStreamWrite((*(*pager).Screen).Stream, newline.as_ptr() as *const u16, 1);
            (*pager).iLine += 1;
            i_column = 0;
            (*pager).nSpacePending = 0;
            ich_start = i_end_line;
            continue;
        }
        if ch == 0x0009 && ((*pager).dwFlags & CON_PAGER_EXPAND_TABS) != 0 {
            if n_tab_width == 0 {
                ich_start = ich + 1;
                continue;
            }
            ich_start = ich + 1;
            (*pager).nSpacePending = n_tab_width as u32 - (i_column % n_tab_width as u32);
            goto_expand_tab = true;
            continue;
        }
        if ch == 0x000C && ((*pager).dwFlags & CON_PAGER_EXPAND_FF) != 0 {
            if b_finite_paging != 0 {
                while (*pager).iLine < scroll_rows {
                    (*pager).iColumn = i_column;
                    // C: `if (PagerLine && PagerLine(Pager,"\n",1)) { iLine++; break; }
                    //     else { CON_STREAM_WRITE("\n",1); iLine++; }`
                    // The else covers BOTH PagerLine==NULL and PagerLine returning 0
                    // — in both cases write a newline + advance (or the loop spins forever).
                    let nl: u16 = 0x000A;
                    let paused = match (*pager).PagerLine {
                        Some(f) => f(pager, &nl as *const u16, 1) != 0,
                        None => false,
                    };
                    if paused {
                        (*pager).iLine += 1;
                        break;
                    } else {
                        ConStreamWrite((*(*pager).Screen).Stream, &nl as *const u16, 1);
                        (*pager).iLine += 1;
                    }
                }
            } else {
                let formfeed_newline: [u16; 3] = [0x000C, 0x000A, 0];
                ConStreamWrite((*(*pager).Screen).Stream, formfeed_newline.as_ptr() as *const u16, 2);
                (*pager).iLine += 1;
            }

            i_column = 0;
            (*pager).nSpacePending = 0;
            ich_start = ich + 1;
            continue;
        }

        if is_double_width_char_trailing != 0 {
            is_double_width_char_trailing = 0;
            let space: [u16; 2] = [0x0020, 0];
            ConStreamWrite((*(*pager).Screen).Stream, space.as_ptr() as *const u16, 1);
        }

        if (page_columns > 0) && (i_column % page_columns == 0) {
            if !(b_finite_paging != 0) || (page_columns < (*(*pager).Screen).csbi.dwSize.X as u32) {
                let newline: [u16; 2] = [0x000A, 0];
                ConStreamWrite((*(*pager).Screen).Stream, newline.as_ptr() as *const u16, 1);
            }
            (*pager).iLine += 1;
        }

        ich_start = ich;
        continue;
    }

    // End
    (*pager).ichCurr = ich_start;
    (*pager).iColumn = i_column;

    if b_finite_paging != 0 && (*pager).iLine >= scroll_rows {
        (*pager).iLine = 0;
        return 1;
    } else {
        return 0;
    }
}

#[no_mangle]
pub unsafe extern "C" fn ConWritePaging(
    Pager: *mut CON_PAGER,
    PagePrompt: PAGE_PROMPT,
    StartPaging: i32,
    szStr: *const u16,
    len: u32,
) -> i32 {
    let mut csbi: CONSOLE_SCREEN_BUFFER_INFO = core::mem::zeroed();
    let bIsConsole: i32;

    if Pager.is_null() {
        return 0;
    }

    bIsConsole = ConGetScreenInfo((*Pager).Screen, &mut csbi);
    if bIsConsole != 0 {
        (*Pager).PageColumns = csbi.dwSize.X as u32;
        (*Pager).PageRows = (csbi.srWindow.Bottom - csbi.srWindow.Top) as u32 + 1;
    } else {
        (*Pager).PageColumns = 0;
        (*Pager).PageRows = 0;
    }

    if StartPaging != 0 {
        if bIsConsole != 0 && ((*Pager).PageRows >= 2) {
            (*Pager).ScrollRows = (*Pager).PageRows - 1;
        } else {
            (*Pager).ScrollRows = 0;
        }

        (*Pager).CachedLine = core::ptr::null_mut();
        (*Pager).cchCachedLine = 0;
        (*Pager).CurrentLine = core::ptr::null_mut();
        (*Pager).ichCurr = 0;
        (*Pager).iEndLine = 0;
        (*Pager).nSpacePending = 0;
        (*Pager).iColumn = 0;
        (*Pager).iLine = 0;
        (*Pager).lineno = 0;
    }

    (*Pager).ich = 0;

    if (*Pager).CachedLine.is_null() {
        if len == 0 || szStr.is_null() {
            return 1;
        }
    }

    while ConPagerWorker(Pager, szStr, len) != 0 {
        if bIsConsole != 0 && ((*Pager).PageRows >= 2) {
            (*Pager).ScrollRows = (*Pager).PageRows - 1;

            if let Some(prompt) = PagePrompt {
                if prompt(Pager, (*Pager).ich as u32, len) == 0 {
                    return 0;
                }
            }
        }

        if bIsConsole != 0 && ConGetScreenInfo((*Pager).Screen, &mut csbi) != 0 {
            (*Pager).PageColumns = csbi.dwSize.X as u32;
            (*Pager).PageRows = (csbi.srWindow.Bottom - csbi.srWindow.Top) as u32 + 1;
        }
    }

    1
}

#[no_mangle]
pub unsafe extern "C" fn ConPutsPaging(
    Pager: *mut CON_PAGER,
    PagePrompt: PAGE_PROMPT,
    StartPaging: i32,
    szStr: *const u16,
) -> i32 {
    let len: u32;

    if szStr.is_null() {
        return 1;
    }

    len = wcslen(szStr) as u32;
    ConWritePaging(Pager, PagePrompt, StartPaging, szStr, len)
}

#[no_mangle]
pub unsafe extern "C" fn ConResPagingEx(
    Pager: *mut CON_PAGER,
    PagePrompt: PAGE_PROMPT,
    StartPaging: i32,
    hInstance: HINSTANCE,
    uID: u32,
) -> i32 {
    let Len: i32;
    let mut szStr: *mut u16 = core::ptr::null_mut();

    Len = K32LoadStringW(
        hInstance,
        uID,
        core::ptr::addr_of_mut!(szStr) as PWSTR,
        0,
    );

    if !szStr.is_null() && Len != 0 {
        ConWritePaging(Pager, PagePrompt, StartPaging, szStr, Len as u32)
    } else {
        1
    }
}

#[no_mangle]
pub unsafe extern "C" fn ConResPaging(
    Pager: *mut CON_PAGER,
    PagePrompt: PAGE_PROMPT,
    StartPaging: i32,
    uID: u32,
) -> i32 {
    ConResPagingEx(Pager, PagePrompt, StartPaging, core::ptr::null_mut(), uID)
}
