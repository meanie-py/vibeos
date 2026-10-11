use crate::cmd::console::ConOutChar;
use crate::cmd::console::SetCursorXY;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut AutoCompletionChar: TCHAR = 0x20 as TCHAR;

#[no_mangle]
pub static mut PathCompletionChar: TCHAR = 0x20 as TCHAR;

#[no_mangle]
pub static mut maxx: SHORT = 0;

#[no_mangle]
pub static mut maxy: SHORT = 0;

static mut bInsert: BOOL = (TRUE) as i32;

unsafe extern "C" fn ClearCommandLine(str: LPTSTR, maxlen: INT, orgx: SHORT, orgy: SHORT) {
    let mut count: INT;
    SetCursorXY(orgx, orgy);
    count = 0;
    while count < (_tcslen(str) as INT) {
        ConOutChar((' ' as u16));
        count += 1;
    }
    _tcsnset(str, ('\0' as u16), (maxlen) as usize);
    SetCursorXY(orgx, orgy);
}

// ---- ReadCommand (hand-ported from cmdinput.c) ----
//
// Build config (base/shell/cmd/config.h): FEATURE_HISTORY, FEATURE_ALIASES and
// FEATURE_4NT_FILENAME_COMPLETION are ON; FEATURE_UNIX_FILENAME_COMPLETION is OFF.
// Only the enabled side of each #ifdef is ported. _UNICODE is on (TCHAR = WCHAR).

// winuser.h virtual-key codes (absent from this crate's bindgen island).
const VK_BACK: WORD = 0x08;
const VK_TAB: WORD = 0x09;
const VK_RETURN: WORD = 0x0D;
const VK_ESCAPE: WORD = 0x1B;
const VK_END: WORD = 0x23;
const VK_HOME: WORD = 0x24;
const VK_LEFT: WORD = 0x25;
const VK_UP: WORD = 0x26;
const VK_RIGHT: WORD = 0x27;
const VK_DOWN: WORD = 0x28;
const VK_INSERT: WORD = 0x2D;
const VK_DELETE: WORD = 0x2E;
const VK_F3: WORD = 0x72;

// C `int / int` and `int % int`. Division by a zero screen width (or INT_MIN / -1)
// is UB in the C and a panic in Rust; with no console width there is nothing
// meaningful to compute, so yield 0 rather than panicking in live code.
#[inline(always)]
fn idiv(a: INT, b: INT) -> INT {
    a.checked_div(b).unwrap_or(0)
}
#[inline(always)]
fn imod(a: INT, b: INT) -> INT {
    a.checked_rem(b).unwrap_or(0)
}
// C `size_t / size_t` and `size_t % size_t` (same zero-divisor guard).
#[inline(always)]
fn udiv(a: SIZE_T, b: SIZE_T) -> SIZE_T {
    a.checked_div(b).unwrap_or(0)
}
#[inline(always)]
fn umod(a: SIZE_T, b: SIZE_T) -> SIZE_T {
    a.checked_rem(b).unwrap_or(0)
}

/* _T "%s" */
const FMT_S: [u16; 3] = [0x0025, 0x0073, 0x0000u16];
/* _T "%s " */
const FMT_S_SP: [u16; 4] = [0x0025, 0x0073, 0x0020, 0x0000u16];
/* _T "\b \b" */
const FMT_BS_SP_BS: [u16; 4] = [0x0008, 0x0020, 0x0008, 0x0000u16];

/* read in a command line */
#[no_mangle]
pub unsafe extern "C" fn ReadCommand(str: LPTSTR, maxlen: INT) -> BOOL {
    use crate::cmd::alias::ExpandAlias;
    use crate::cmd::console::{GetCursorX, GetCursorXY, GetCursorY, SetCursorType};
    use crate::cmd::filecomp::CompleteFilename; // 4NT variant
    use crate::cmd::history::{History, History_del_current_entry, History_move_to_bottom, PeekHistory};

    let mut csbi: CONSOLE_SCREEN_BUFFER_INFO = core::mem::zeroed();
    let mut orgx: SHORT; /* origin x/y */
    let mut orgy: SHORT;
    let mut curx: SHORT; /*current x/y cursor position*/
    let mut cury: SHORT;
    let mut tempscreen: SIZE_T;
    let mut count: INT; /*used in some for loops*/
    let mut current: INT = 0; /*the position of the cursor in the string (str)*/
    let mut charcount: INT = 0; /*chars in the string (str)*/
    let mut ir: INPUT_RECORD = core::mem::zeroed();
    let mut dwControlKeyState: DWORD;
    let mut ch: TCHAR;
    let mut bReturn: BOOL;
    let mut bCharInput: BOOL;
    /* FEATURE_4NT_FILENAME_COMPLETION */
    let mut szPath: [TCHAR; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];
    /* FEATURE_HISTORY */
    let mut PreviousChar: TCHAR;

    // &str[i] for a possibly-negative INT index, as C pointer arithmetic.
    macro_rules! at {
        ($i:expr) => {
            str.offset(($i) as isize)
        };
    }

    if GetConsoleScreenBufferInfo(GetStdHandle(STD_OUTPUT_HANDLE), core::ptr::addr_of_mut!(csbi)) == 0 {
        /* No console */
        let hStdin: HANDLE = GetStdHandle(STD_INPUT_HANDLE);
        let mut dwRead: DWORD = 0;
        let mut chr: CHAR = 0;
        loop {
            if ReadFile(
                hStdin,
                core::ptr::addr_of_mut!(chr) as PVOID,
                1,
                core::ptr::addr_of_mut!(dwRead),
                core::ptr::null_mut(),
            ) == 0
                || dwRead == 0
            {
                return FALSE as BOOL;
            }
            /* _UNICODE */
            let dst = at!(charcount);
            charcount += 1;
            MultiByteToWideChar(InputCodePage, 0, core::ptr::addr_of!(chr), 1, dst, 1);
            if !(chr != (b'\n' as CHAR) && charcount < maxlen) {
                break;
            }
        }
        *at!(charcount) = '\0' as u16;
        return TRUE as BOOL;
    }

    /* get screen size */
    maxx = csbi.dwSize.X;
    maxy = csbi.dwSize.Y;

    orgx = csbi.dwCursorPosition.X;
    curx = orgx;
    orgy = csbi.dwCursorPosition.Y;
    cury = orgy;

    memset(
        str as *mut c_void,
        0,
        (maxlen as isize as usize).wrapping_mul(core::mem::size_of::<TCHAR>()),
    );

    SetCursorType(bInsert, TRUE as BOOL);

    loop {
        bReturn = FALSE as BOOL;
        ConInKey(core::ptr::addr_of_mut!(ir));

        dwControlKeyState = ir.Event.KeyEvent.dwControlKeyState;

        // C: `goto DoBackSpace;` from the ^H case into the VK_BACK case below.
        let mut goto_backspace = false;

        if dwControlKeyState & (RIGHT_ALT_PRESSED | LEFT_ALT_PRESSED | RIGHT_CTRL_PRESSED | LEFT_CTRL_PRESSED) as DWORD != 0 {
            let vk = ir.Event.KeyEvent.wVirtualKeyCode;
            if vk == 'K' as u16 {
                /* FEATURE_HISTORY */
                /* add the current command line to the history */
                if dwControlKeyState & (LEFT_CTRL_PRESSED | RIGHT_CTRL_PRESSED) as DWORD != 0 {
                    if *str != 0 {
                        History(0, str);
                    }

                    ClearCommandLine(str, maxlen, orgx, orgy);
                    charcount = 0;
                    current = 0;
                    curx = orgx;
                    cury = orgy;
                }
            } else if vk == 'D' as u16 {
                /* FEATURE_HISTORY */
                /* delete current history entry */
                if dwControlKeyState & (LEFT_CTRL_PRESSED | RIGHT_CTRL_PRESSED) as DWORD != 0 {
                    ClearCommandLine(str, maxlen, orgx, orgy);
                    History_del_current_entry(str);
                    charcount = _tcslen(str) as INT;
                    current = charcount;
                    ConOutPrintf!(FMT_S.as_ptr() /* _T "%s" */, str);
                    GetCursorXY(core::ptr::addr_of_mut!(curx), core::ptr::addr_of_mut!(cury));
                }
            } else if vk == 'M' as u16 {
                /* ^M does the same as return */
                if dwControlKeyState & (LEFT_CTRL_PRESSED | RIGHT_CTRL_PRESSED) as DWORD != 0 {
                    /* end input, return to main */
                    /* FEATURE_HISTORY: add to the history */
                    if *str != 0 {
                        History(0, str);
                    }
                    *at!(charcount) = '\n' as u16;
                    charcount += 1;
                    *at!(charcount) = '\0' as u16;
                    ConOutChar('\n' as u16);
                    bReturn = TRUE as BOOL;
                }
            } else if vk == 'H' as u16 {
                /* ^H does the same as VK_BACK */
                if dwControlKeyState & (LEFT_CTRL_PRESSED | RIGHT_CTRL_PRESSED) as DWORD != 0 {
                    bCharInput = FALSE as BOOL;
                    goto_backspace = true;
                }
            }
        }

        // Skipped by the goto in the C, but ^H already set it FALSE, so this is equivalent.
        bCharInput = FALSE as BOOL;

        let vk = ir.Event.KeyEvent.wVirtualKeyCode;

        if goto_backspace || vk == VK_BACK {
            // DoBackSpace:
            /* <BACKSPACE> - delete character to left of cursor */
            if current > 0 && charcount > 0 {
                if current == charcount {
                    /* if at end of line */
                    *at!(current - 1) = '\0' as u16;
                    if GetCursorX() != 0 {
                        ConOutPrintf!(FMT_BS_SP_BS.as_ptr() /* _T "\b \b" */);
                        curx = curx.wrapping_sub(1);
                    } else {
                        SetCursorXY((maxx as INT - 1) as SHORT, (GetCursorY() as INT - 1) as SHORT);
                        ConOutChar(' ' as u16);
                        SetCursorXY((maxx as INT - 1) as SHORT, (GetCursorY() as INT - 1) as SHORT);
                        cury = cury.wrapping_sub(1);
                        curx = (maxx as INT - 1) as SHORT;
                    }
                } else {
                    count = current - 1;
                    while count < charcount {
                        *at!(count) = *at!(count + 1);
                        count += 1;
                    }
                    if GetCursorX() != 0 {
                        SetCursorXY((GetCursorX() as INT - 1) as SHORT, GetCursorY());
                        curx = curx.wrapping_sub(1);
                    } else {
                        SetCursorXY((maxx as INT - 1) as SHORT, (GetCursorY() as INT - 1) as SHORT);
                        cury = cury.wrapping_sub(1);
                        curx = (maxx as INT - 1) as SHORT;
                    }
                    GetCursorXY(core::ptr::addr_of_mut!(curx), core::ptr::addr_of_mut!(cury));
                    ConOutPrintf!(FMT_S_SP.as_ptr() /* _T "%s " */, at!(current - 1));
                    SetCursorXY(curx, cury);
                }
                charcount -= 1;
                current -= 1;
            }
        } else if vk == VK_INSERT {
            /* toggle insert/overstrike mode */
            bInsert ^= TRUE as BOOL;
            SetCursorType(bInsert, TRUE as BOOL);
        } else if vk == VK_DELETE {
            /* delete character under cursor */
            if current != charcount && charcount > 0 {
                count = current;
                while count < charcount {
                    *at!(count) = *at!(count + 1);
                    count += 1;
                }
                charcount -= 1;
                GetCursorXY(core::ptr::addr_of_mut!(curx), core::ptr::addr_of_mut!(cury));
                ConOutPrintf!(FMT_S_SP.as_ptr() /* _T "%s " */, at!(current));
                SetCursorXY(curx, cury);
            }
        } else if vk == VK_HOME {
            /* goto beginning of string */
            if current != 0 {
                SetCursorXY(orgx, orgy);
                curx = orgx;
                cury = orgy;
                current = 0;
            }
        } else if vk == VK_END {
            /* goto end of string */
            if current != charcount {
                SetCursorXY(orgx, orgy);
                ConOutPrintf!(FMT_S.as_ptr() /* _T "%s" */, str);
                GetCursorXY(core::ptr::addr_of_mut!(curx), core::ptr::addr_of_mut!(cury));
                current = charcount;
            }
        } else if vk == VK_TAB {
            /* FEATURE_4NT_FILENAME_COMPLETION */
            /* used to later see if we went down to the next line */
            tempscreen = charcount as isize as SIZE_T;
            szPath[0] = '\0' as u16;

            /* str is the whole things that is on the current line
               that is and and out.  arg 2 is weather it goes back
                one file or forward one file */
            CompleteFilename(
                str,
                (ir.Event.KeyEvent.dwControlKeyState & SHIFT_PRESSED as DWORD == 0) as BOOL,
                szPath.as_mut_ptr(),
                current as UINT,
            );
            /* Attempt to clear the line */
            ClearCommandLine(str, maxlen, orgx, orgy);
            curx = orgx;
            cury = orgy;
            charcount = 0;
            current = 0;

            /* Everything is deleted, lets add it back in */
            _tcscpy(str, szPath.as_ptr());

            /* Figure out where cusor is going to be after we print it */
            charcount = _tcslen(str) as INT;
            current = charcount;

            SetCursorXY(orgx, orgy);
            /* Print out what we have now */
            ConOutPrintf!(FMT_S.as_ptr() /* _T "%s" */, str);

            /* Move cursor accordingly */
            if tempscreen > charcount as isize as SIZE_T {
                GetCursorXY(core::ptr::addr_of_mut!(curx), core::ptr::addr_of_mut!(cury));
                count = tempscreen.wrapping_sub(charcount as isize as SIZE_T) as INT;
                // for (; count--; )
                loop {
                    let c = count;
                    count = count.wrapping_sub(1);
                    if c == 0 {
                        break;
                    }
                    ConOutChar(' ' as u16);
                }
                SetCursorXY(curx, cury);
            } else {
                if idiv(charcount + orgx as INT, maxx as INT) + orgy as INT > maxy as INT - 1 {
                    orgy = (orgy as INT
                        + (maxy as INT - (idiv(charcount + orgx as INT, maxx as INT) + orgy as INT + 1)))
                        as SHORT;
                }
            }
            SetCursorXY(
                imod(orgx as INT + current, maxx as INT) as SHORT,
                (orgy as INT + idiv(orgx as INT + current, maxx as INT)) as SHORT,
            );
            GetCursorXY(core::ptr::addr_of_mut!(curx), core::ptr::addr_of_mut!(cury));
        } else if vk == 'C' as u16 {
            if ir.Event.KeyEvent.dwControlKeyState & (RIGHT_CTRL_PRESSED | LEFT_CTRL_PRESSED) as DWORD != 0 {
                /* Ignore the Ctrl-C key event if it has already been handled */
                if bCtrlBreak != 0 {
                    /*
                     * Fully print the entered string
                     * so the command prompt would not overwrite it.
                     */
                    SetCursorXY(orgx, orgy);
                    ConOutPrintf!(FMT_S.as_ptr() /* _T "%s" */, str);

                    /*
                     * A Ctrl-C. Do not clear the command line,
                     * but return an empty string in str.
                     */
                    *str = '\0' as u16;
                    curx = orgx;
                    cury = orgy;
                    charcount = 0;
                    current = 0;
                    bReturn = TRUE as BOOL;
                }
            } else {
                /* Just a normal 'C' character */
                bCharInput = TRUE as BOOL;
            }
        } else if vk == VK_RETURN {
            /* end input, return to main */
            /* FEATURE_HISTORY: add to the history */
            if *str != 0 {
                History(0, str);
            }
            *at!(charcount) = '\n' as u16;
            charcount += 1;
            *at!(charcount) = '\0' as u16;
            ConOutChar('\n' as u16);
            bReturn = TRUE as BOOL;
        } else if vk == VK_ESCAPE {
            /* clear str  Make this callable! */
            ClearCommandLine(str, maxlen, orgx, orgy);
            curx = orgx;
            cury = orgy;
            charcount = 0;
            current = 0;
        } else if vk == VK_F3 || vk == VK_UP {
            /* FEATURE_HISTORY: VK_F3 moves to the bottom, then falls through to VK_UP */
            if vk == VK_F3 {
                History_move_to_bottom();
            }
            /* get previous command from buffer */
            ClearCommandLine(str, maxlen, orgx, orgy);
            History(-1, str);
            charcount = _tcslen(str) as INT;
            current = charcount;
            if idiv(charcount + orgx as INT, maxx as INT) + orgy as INT > maxy as INT - 1 {
                orgy = (orgy as INT + (maxy as INT - (idiv(charcount + orgx as INT, maxx as INT) + orgy as INT + 1)))
                    as SHORT;
            }
            ConOutPrintf!(FMT_S.as_ptr() /* _T "%s" */, str);
            GetCursorXY(core::ptr::addr_of_mut!(curx), core::ptr::addr_of_mut!(cury));
        } else if vk == VK_DOWN {
            /* FEATURE_HISTORY */
            /* get next command from buffer */
            ClearCommandLine(str, maxlen, orgx, orgy);
            History(1, str);
            charcount = _tcslen(str) as INT;
            current = charcount;
            if idiv(charcount + orgx as INT, maxx as INT) + orgy as INT > maxy as INT - 1 {
                orgy = (orgy as INT + (maxy as INT - (idiv(charcount + orgx as INT, maxx as INT) + orgy as INT + 1)))
                    as SHORT;
            }
            ConOutPrintf!(FMT_S.as_ptr() /* _T "%s" */, str);
            GetCursorXY(core::ptr::addr_of_mut!(curx), core::ptr::addr_of_mut!(cury));
        } else if vk == VK_LEFT {
            if dwControlKeyState & (RIGHT_CTRL_PRESSED | LEFT_CTRL_PRESSED) as DWORD != 0 {
                /* move cursor to the previous word */
                if current > 0 {
                    while current > 0 && *at!(current - 1) == ' ' as u16 {
                        current -= 1;
                        if curx == 0 {
                            cury = cury.wrapping_sub(1);
                            curx = (maxx as INT - 1) as SHORT;
                        } else {
                            curx = curx.wrapping_sub(1);
                        }
                    }

                    while current > 0 && *at!(current - 1) != ' ' as u16 {
                        current -= 1;
                        if curx == 0 {
                            cury = cury.wrapping_sub(1);
                            curx = (maxx as INT - 1) as SHORT;
                        } else {
                            curx = curx.wrapping_sub(1);
                        }
                    }

                    SetCursorXY(curx, cury);
                }
            } else {
                /* move cursor left */
                if current > 0 {
                    current -= 1;
                    if GetCursorX() == 0 {
                        SetCursorXY((maxx as INT - 1) as SHORT, (GetCursorY() as INT - 1) as SHORT);
                        curx = (maxx as INT - 1) as SHORT;
                        cury = cury.wrapping_sub(1);
                    } else {
                        SetCursorXY((GetCursorX() as INT - 1) as SHORT, GetCursorY());
                        curx = curx.wrapping_sub(1);
                    }
                } else {
                    MessageBeep((-(1 as i32)) as UINT);
                }
            }
        } else if vk == VK_RIGHT {
            if dwControlKeyState & (RIGHT_CTRL_PRESSED | LEFT_CTRL_PRESSED) as DWORD != 0 {
                /* move cursor to the next word */
                if current != charcount {
                    while current != charcount && *at!(current) != ' ' as u16 {
                        current += 1;
                        if curx as INT == maxx as INT - 1 {
                            cury = cury.wrapping_add(1);
                            curx = 0;
                        } else {
                            curx = curx.wrapping_add(1);
                        }
                    }

                    while current != charcount && *at!(current) == ' ' as u16 {
                        current += 1;
                        if curx as INT == maxx as INT - 1 {
                            cury = cury.wrapping_add(1);
                            curx = 0;
                        } else {
                            curx = curx.wrapping_add(1);
                        }
                    }

                    SetCursorXY(curx, cury);
                }
            } else {
                /* move cursor right */
                if current != charcount {
                    current += 1;
                    if GetCursorX() as INT == maxx as INT - 1 {
                        SetCursorXY(0, (GetCursorY() as INT + 1) as SHORT);
                        curx = 0;
                        cury = cury.wrapping_add(1);
                    } else {
                        SetCursorXY((GetCursorX() as INT + 1) as SHORT, GetCursorY());
                        curx = curx.wrapping_add(1);
                    }
                } else {
                    /* FEATURE_HISTORY */
                    let last: LPCTSTR = PeekHistory(-1);
                    if !last.is_null() && charcount < (_tcslen(last) as INT) {
                        PreviousChar = *last.offset(current as isize);
                        ConOutChar(PreviousChar);
                        GetCursorXY(core::ptr::addr_of_mut!(curx), core::ptr::addr_of_mut!(cury));
                        *at!(current) = PreviousChar;
                        current += 1;
                        charcount += 1;
                    }
                }
            }
        } else {
            /* This input is just a normal char */
            bCharInput = TRUE as BOOL;
        }

        /* _UNICODE */
        ch = ir.Event.KeyEvent.uChar.UnicodeChar;
        if ch >= 32 && (charcount != (maxlen - 2)) && bCharInput != 0 {
            /* insert character into string... */
            if bInsert != 0 && current != charcount {
                /* If this character insertion will cause screen scrolling,
                 * adjust the saved origin of the command prompt. */
                tempscreen = (_tcslen(at!(current)) as SIZE_T).wrapping_add(curx as isize as SIZE_T);
                let mx = maxx as INT as isize as SIZE_T;
                if umod(tempscreen, mx) == (maxx as INT - 1) as isize as SIZE_T
                    && udiv(tempscreen, mx).wrapping_add(cury as INT as isize as SIZE_T)
                        == (maxy as INT - 1) as isize as SIZE_T
                {
                    orgy = orgy.wrapping_sub(1);
                    cury = cury.wrapping_sub(1);
                }

                count = charcount;
                while count > current {
                    *at!(count) = *at!(count - 1);
                    count -= 1;
                }
                *at!(current) = ch;
                current += 1;
                if curx as INT == maxx as INT - 1 {
                    curx = 0;
                    cury = cury.wrapping_add(1);
                } else {
                    curx = curx.wrapping_add(1);
                }
                ConOutPrintf!(FMT_S.as_ptr() /* _T "%s" */, at!(current - 1));
                SetCursorXY(curx, cury);
                charcount += 1;
            } else {
                if current == charcount {
                    charcount += 1;
                }
                *at!(current) = ch;
                current += 1;
                if GetCursorX() as INT == maxx as INT - 1 && GetCursorY() as INT == maxy as INT - 1 {
                    orgy = orgy.wrapping_sub(1);
                    cury = cury.wrapping_sub(1);
                }
                if GetCursorX() as INT == maxx as INT - 1 {
                    curx = 0;
                    cury = cury.wrapping_add(1);
                } else {
                    curx = curx.wrapping_add(1);
                }
                ConOutChar(ch);
            }
        }

        if bReturn != 0 {
            break;
        }
    }

    SetCursorType(bInsert, TRUE as BOOL);

    /* FEATURE_ALIASES: expand all aliases */
    ExpandAlias(str, maxlen);
    return TRUE as BOOL;
}
