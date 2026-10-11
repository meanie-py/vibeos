// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_1006EE11D11A: [u16; 17] = [
    0x0045u16, 0x006E, 0x0061, 0x0062, 0x006C, 0x0065, 0x0045, 0x0078, 0x0074, 0x0065,
    0x006E, 0x0073, 0x0069, 0x006F, 0x006E, 0x0073, 0x0000,
];
pub static RUSTCONV_WSTR_5462AED96D21: [u16; 3] = [0x002Fu16, 0x003F, 0x0000];
pub static RUSTCONV_WSTR_669336782398: [u16; 5] = [0x004Du16, 0x004F, 0x0052, 0x0045, 0x0000];
pub static RUSTCONV_WSTR_8D71B3FAAB82: [u16; 2] = [0x0020u16, 0x0000];
pub static RUSTCONV_WSTR_96A296D224F2: [u16; 1] = [0x0000u16];
pub static RUSTCONV_WSTR_A3F18BB6EFE2: [u16; 27] = [
    0x0045u16, 0x0052, 0x0052, 0x004F, 0x0052, 0x003A, 0x0020, 0x0049, 0x006E, 0x0076,
    0x0061, 0x006C, 0x0069, 0x0064, 0x0020, 0x0066, 0x0069, 0x006C, 0x0065, 0x0020,
    0x0073, 0x0069, 0x007A, 0x0065, 0x0021, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_B5E9C1A0FAD8: [u16; 7] = [0x0043u16, 0x004F, 0x004E, 0x0049, 0x004E, 0x0024, 0x0000];
pub static RUSTCONV_WSTR_CE9E0758CAAD: [u16; 37] = [
    0x0053u16, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x004D,
    0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0043,
    0x006F, 0x006D, 0x006D, 0x0061, 0x006E, 0x0064, 0x0020, 0x0050, 0x0072, 0x006F,
    0x0063, 0x0065, 0x0073, 0x0073, 0x006F, 0x0072, 0x0000,
];
pub static RUSTCONV_WSTR_F5DC9649106A: [u16; 18] = [
    0x0045u16, 0x0072, 0x0072, 0x006F, 0x0072, 0x003A, 0x0020, 0x006E, 0x006F, 0x0020,
    0x006D, 0x0065, 0x006D, 0x006F, 0x0072, 0x0079, 0x000A, 0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut dwFileSize: DWORD = 0;

#[no_mangle]
pub static mut dwSumReadChars: DWORD = 0;

#[no_mangle]
pub static mut dwSumReadBytes: DWORD = 0;

#[no_mangle]
pub static mut hFile: HANDLE = unsafe { INVALID_HANDLE_VALUE };

#[no_mangle]
pub static mut hStdOut: HANDLE =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

#[no_mangle]
pub static mut hStdIn: HANDLE =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

#[no_mangle]
pub static mut hKeyboard: HANDLE =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

#[no_mangle]
pub static mut bEnableExtensions: BOOL = (TRUE as i32);

static mut s_dwFlags: DWORD = 0 as DWORD;

static mut s_nTabWidth: LONG = 8 as LONG;

static mut s_nNextLineNo: DWORD = 0 as DWORD;

static mut s_bPrevLineIsBlank: BOOL = (FALSE as i32);

static mut s_fPrompt: WORD = 0 as WORD;

static mut s_bDoNextFile: BOOL = (FALSE as i32);

pub const FLAG_HELP: DWORD = 1 << 0;
pub const FLAG_E: DWORD = 1 << 1;
pub const FLAG_C: DWORD = 1 << 2;
pub const FLAG_P: DWORD = 1 << 3;
pub const FLAG_S: DWORD = 1 << 4;
pub const FLAG_Tn: DWORD = 1 << 5;
pub const FLAG_PLUSn: DWORD = 1 << 6;

pub const PROMPT_PERCENT: WORD = 1 << 0;
pub const PROMPT_LINE_AT: WORD = 1 << 1;
pub const PROMPT_OPTIONS: WORD = 1 << 2;
pub const PROMPT_LINES: WORD = 1 << 3;

pub type ENCODING = i32;
pub const ENCODING_ANSI: ENCODING = 0;
pub const ENCODING_UTF16LE: ENCODING = 1;
pub const ENCODING_UTF16BE: ENCODING = 2;
pub const ENCODING_UTF8: ENCODING = 3;

unsafe extern "C" fn IsBlankLine(line: PCWCH, cch: SIZE_T) -> BOOL {
    let mut ich: SIZE_T;
    let mut wType: WORD;
    ich = 0;
    while ich < cch {
        /*
         * Explicitly exclude FORM-FEED from the check,
         * so that the pager can handle it.
         */
        if *line.add((ich) as _) == 0x000Cu16 {
            return (FALSE) as BOOL;
        }

        /*
         * Otherwise do the extended blanks check.
         * Note that MS MORE.COM only checks for spaces (\x20) and TABs (\x09).
         * See http://archives.miloush.net/michkap/archive/2007/06/11/3230072.html
         * for more information.
         */
        wType = 0;
        GetStringTypeW(
            CT_CTYPE1,
            line.add((ich) as _),
            1,
            core::ptr::addr_of_mut!(wType),
        );
        if ((wType as u32) & (C1_BLANK | C1_SPACE)) == 0 {
            return (FALSE) as BOOL;
        }
        ich = ich.wrapping_add(1);
    }
    return (TRUE) as BOOL;
}
#[cfg(target_arch = "x86")]
unsafe extern "stdcall" fn MorePagerLine(
    Pager: PCON_PAGER,
    line: PCWCH,
    cch: SIZE_T,
) -> BOOL {
    if (s_dwFlags & FLAG_PLUSn as DWORD) != 0 /* Skip lines */ {
        if (*Pager).lineno < s_nNextLineNo {
            s_bPrevLineIsBlank = (FALSE as i32);
            return (TRUE) as BOOL; /* Handled */
        }
        s_dwFlags &= !(FLAG_PLUSn as DWORD);
    }

    if (s_dwFlags & FLAG_S as DWORD) != 0 /* Shrink blank lines */ {
        if IsBlankLine(line, cch) != 0 {
            if s_bPrevLineIsBlank != 0 {
                return (TRUE) as BOOL; /* Handled */
            }

            /*
             * Display a single blank line, independently of the actual size
             * of the current line, by displaying just one space: this is
             * especially needed in order to force line wrapping when the
             * ENABLE_VIRTUAL_TERMINAL_PROCESSING or DISABLE_NEWLINE_AUTO_RETURN
             * console modes are enabled.
             * Then, reposition the cursor to the next line, first column.
             */
            if (*Pager).PageColumns > 0 {
                ConStreamWrite(
                    ((*(*Pager).Screen).Stream).cast(),
                    ([' ' as u16, 0u16].as_ptr() as PCTCH).cast(),
                    1,
                );
            }
            ConStreamWrite(
                ((*(*Pager).Screen).Stream).cast(),
                (['\n' as u16, 0u16].as_ptr() as PCTCH).cast(),
                1,
            );
            (*Pager).iLine = (*Pager).iLine.wrapping_add(1);
            (*Pager).iColumn = 0;

            s_bPrevLineIsBlank = (TRUE as i32);
            s_nNextLineNo = 0;

            return (TRUE) as BOOL; /* Handled */
        } else {
            s_bPrevLineIsBlank = (FALSE as i32);
        }
    }

    s_nNextLineNo = 0;
    /* Not handled, let the pager do the default action */
    return (FALSE) as BOOL;
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "system" fn MorePagerLine(
    Pager: PCON_PAGER,
    line: PCWCH,
    cch: SIZE_T,
) -> BOOL {
    if (s_dwFlags & FLAG_PLUSn as DWORD) != 0 /* Skip lines */ {
        if (*Pager).lineno < s_nNextLineNo {
            s_bPrevLineIsBlank = (FALSE as i32);
            return (TRUE) as BOOL; /* Handled */
        }
        s_dwFlags &= !(FLAG_PLUSn as DWORD);
    }

    if (s_dwFlags & FLAG_S as DWORD) != 0 /* Shrink blank lines */ {
        if IsBlankLine(line, cch) != 0 {
            if s_bPrevLineIsBlank != 0 {
                return (TRUE) as BOOL; /* Handled */
            }

            /*
             * Display a single blank line, independently of the actual size
             * of the current line, by displaying just one space: this is
             * especially needed in order to force line wrapping when the
             * ENABLE_VIRTUAL_TERMINAL_PROCESSING or DISABLE_NEWLINE_AUTO_RETURN
             * console modes are enabled.
             * Then, reposition the cursor to the next line, first column.
             */
            if (*Pager).PageColumns > 0 {
                ConStreamWrite(
                    (*(*Pager).Screen).Stream,
                    ([' ' as u16, 0u16].as_ptr() as PCTCH).cast(),
                    1,
                );
            }
            ConStreamWrite(
                (*(*Pager).Screen).Stream,
                (['\n' as u16, 0u16].as_ptr() as PCTCH).cast(),
                1,
            );
            (*Pager).iLine = (*Pager).iLine.wrapping_add(1);
            (*Pager).iColumn = 0;

            s_bPrevLineIsBlank = (TRUE as i32);
            s_nNextLineNo = 0;

            return (TRUE) as BOOL; /* Handled */
        } else {
            s_bPrevLineIsBlank = (FALSE as i32);
        }
    }

    s_nNextLineNo = 0;
    /* Not handled, let the pager do the default action */
    return (FALSE) as BOOL;
}
#[cfg(target_arch = "x86")]
unsafe extern "stdcall" fn PagePrompt(Pager: PCON_PAGER, Done: DWORD, Total: DWORD) -> BOOL {
    let mut hInput: HANDLE = ConStreamGetOSHandle(StdIn);
    let mut hOutput: HANDLE = ConStreamGetOSHandle(((*(*Pager).Screen).Stream).cast());
    let mut csbi: CONSOLE_SCREEN_BUFFER_INFO = core::mem::zeroed();
    let mut orgCursorPosition: COORD = core::mem::zeroed();
    let mut dwMode: DWORD = 0;

    let mut KeyEvent: KEY_EVENT_RECORD = core::mem::zeroed();
    let mut fCtrl: BOOL = 0;
    let mut nLines: DWORD = 0;
    let mut chSubCommand: WCHAR = 0;

    /* Prompt strings (small size since the prompt should
     * hold ideally on one <= 80-character line) */
    static mut StrPercent: [WCHAR; 80] = [0 as WCHAR; 80];
    static mut StrLineAt: [WCHAR; 80] = [0 as WCHAR; 80];
    static mut StrOptions: [WCHAR; 80] = [0 as WCHAR; 80];
    static mut StrLines: [WCHAR; 80] = [0 as WCHAR; 80];
    static mut AreStrLoaded: BOOL = (FALSE as i32);

    let mut szPercent: [WCHAR; 80] = [0 as WCHAR; 80];
    let mut szLineAt: [WCHAR; 80] = [0 as WCHAR; 80];

    /* Load the prompt strings */
    if AreStrLoaded == 0 {
        LoadStringW(core::ptr::null_mut(), IDS_CONTINUE_PERCENT, &raw mut StrPercent as LPWSTR, ARRAYSIZE!(StrPercent) as i32);
        LoadStringW(core::ptr::null_mut(), IDS_CONTINUE_LINE_AT, &raw mut StrLineAt as LPWSTR, ARRAYSIZE!(StrLineAt) as i32);
        LoadStringW(core::ptr::null_mut(), IDS_CONTINUE_OPTIONS, &raw mut StrOptions as LPWSTR, ARRAYSIZE!(StrOptions) as i32);
        LoadStringW(core::ptr::null_mut(), IDS_CONTINUE_LINES, &raw mut StrLines as LPWSTR, ARRAYSIZE!(StrLines) as i32);
        AreStrLoaded = (TRUE as i32);
    }

    /*
     * Check whether the pager is prompting, but we have actually finished
     * to display a given file, or no data is present in STDIN anymore.
     * In this case, skip the prompt altogether. The only exception is when
     * we are displaying other files.
     */
    // TODO: Implement!

    'restart: loop {
        nLines = 0;

        /* Do not show the progress percentage when STDIN is being displayed */
        if (s_fPrompt as DWORD & ((PROMPT_PERCENT)) as DWORD) != 0 {
            /*
             * The progress percentage is evaluated as follows.
             * So far we have read a total of 'dwSumReadBytes' bytes from the file.
             * Amongst those is the latest read chunk of 'dwReadBytes' bytes, to which
             * correspond a number of 'dwReadChars' characters with which we have called
             * ConWritePaging who called PagePrompt. We then have: Total == dwReadChars.
             * During this ConWritePaging call the PagePrompt was called after 'Done'
             * number of characters over 'Total'.
             * It should be noted that for 'dwSumReadBytes' number of bytes read it
             * *roughly* corresponds 'dwSumReadChars' number of characters. This is
             * because there may be some failures happening during the conversion of
             * the bytes read to the character string for a given encoding.
             * Therefore the number of characters displayed on screen is equal to:
             *   dwSumReadChars - Total + Done ,
             * but the best corresponding approximated number of bytes would be:
             *   dwSumReadBytes - (Total - Done) * (dwSumReadBytes / dwSumReadChars) ,
             * where the ratio is the average number of bytes per character.
             * The percentage is then computed relative to the total file size.
             */
            let mut dwPercent: DWORD = dwSumReadBytes
                .wrapping_sub(
                    Total
                        .wrapping_sub(Done)
                        .wrapping_mul(dwSumReadBytes.wrapping_div(dwSumReadChars)),
                )
                .wrapping_mul(100)
                .wrapping_div(dwFileSize);
            StringCchPrintfW!(&raw mut szPercent as LPWSTR, ARRAYSIZE!(szPercent), &raw mut StrPercent as LPCWSTR, dwPercent);
        }
        if (s_fPrompt as DWORD & ((PROMPT_LINE_AT)) as DWORD) != 0 {
            StringCchPrintfW!(&raw mut szLineAt as LPWSTR, ARRAYSIZE!(szLineAt), &raw mut StrLineAt as LPCWSTR, (*Pager).lineno);
        }

        /* Suitably format and display the prompt */
        ConResMsgPrintf(((*(*Pager).Screen).Stream).cast(), 0, IDS_CONTINUE_PROMPT,
                        (if (s_fPrompt as DWORD & ((PROMPT_PERCENT)) as DWORD) != 0 { &raw const szPercent as *const WCHAR } else { RUSTCONV_WSTR_96A296D224F2.as_ptr() }),
                        (if (s_fPrompt as DWORD & ((PROMPT_LINE_AT)) as DWORD) != 0 { &raw const szLineAt as *const WCHAR } else { RUSTCONV_WSTR_96A296D224F2.as_ptr() }),
                        (if (s_fPrompt as DWORD & ((PROMPT_OPTIONS)) as DWORD) != 0 { &raw const StrOptions as *const WCHAR } else { RUSTCONV_WSTR_96A296D224F2.as_ptr() }),
                        (if (s_fPrompt as DWORD & ((PROMPT_LINES)) as DWORD) != 0 { &raw const StrLines as *const WCHAR } else { RUSTCONV_WSTR_96A296D224F2.as_ptr() }));

        /* Reset the prompt to a default state */
        s_fPrompt = (s_fPrompt as DWORD & (!((PROMPT_LINE_AT) | (PROMPT_OPTIONS) | (PROMPT_LINES))) as DWORD) as WORD;

        /* RemoveBreakHandler */
        SetConsoleCtrlHandler(None, (TRUE as i32));
        /* ConInDisable */
        GetConsoleMode(hInput, &raw mut dwMode);
        dwMode &= !((ENABLE_PROCESSED_INPUT) as DWORD);
        SetConsoleMode(hInput, dwMode);

        // FIXME: Does not support TTY yet!
        ConGetScreenInfo((*Pager).Screen, &raw mut csbi);
        orgCursorPosition = csbi.dwCursorPosition;
        loop {
            let mut ir: INPUT_RECORD = core::mem::zeroed();
            let mut dwRead: DWORD = 0;
            let mut ch: WCHAR = 0;

            loop {
                ReadConsoleInput(hInput, &raw mut ir, 1, &raw mut dwRead);
                if !((ir.EventType != (KEY_EVENT as u16)) || (ir.Event.KeyEvent.bKeyDown == 0)) {
                    break;
                }
            }

            /* Got our key */
            KeyEvent = ir.Event.KeyEvent;

            /* Ignore any unsupported keyboard press */
            if (KeyEvent.wVirtualKeyCode == (VK_SHIFT as u16)) ||
                (KeyEvent.wVirtualKeyCode == (VK_MENU as u16)) ||
                (KeyEvent.wVirtualKeyCode == (VK_CONTROL as u16))
            {
                continue;
            }

            /* Ctrl key is pressed? */
            fCtrl = ((KeyEvent.dwControlKeyState & ((LEFT_CTRL_PRESSED) | (RIGHT_CTRL_PRESSED))) != 0) as BOOL;

            /* Ctrl+C or Ctrl+Esc? */
            if fCtrl != 0 && ((KeyEvent.wVirtualKeyCode == (VK_ESCAPE as u16)) ||
                              (KeyEvent.wVirtualKeyCode == 0x0043u16))
            {
                chSubCommand = 0;
                break;
            }

            /* If extended features are unavailable, or no
             * pending commands, don't do more processing. */
            if (s_dwFlags & (FLAG_E)) == 0 || (chSubCommand == 0) {
                break;
            }

            ch = KeyEvent.uChar.UnicodeChar;
            if 0x0030u16 <= ch && ch <= 0x0039u16 {
                nLines = nLines.wrapping_mul(10);
                nLines = nLines.wrapping_add((ch as DWORD).wrapping_sub(0x0030u16 as DWORD));
                ConStreamWrite(((*(*Pager).Screen).Stream).cast(), (&raw mut ch as PCTCH).cast(), 1);
                continue;
            } else if KeyEvent.wVirtualKeyCode == (VK_RETURN as u16) {
                /* Validate the line number */
                break;
            } else if KeyEvent.wVirtualKeyCode == (VK_ESCAPE as u16) {
                /* Cancel the current command */
                chSubCommand = 0;
                break;
            } else if KeyEvent.wVirtualKeyCode == (VK_BACK as u16) {
                if nLines != 0 {
                    nLines /= 10;
                }

                /* Erase the current character */
                ConGetScreenInfo((*Pager).Screen, &raw mut csbi);
                if (csbi.dwCursorPosition.Y > orgCursorPosition.Y) ||
                    ((csbi.dwCursorPosition.Y == orgCursorPosition.Y) &&
                     (csbi.dwCursorPosition.X > orgCursorPosition.X))
                {
                    if csbi.dwCursorPosition.X > 0 {
                        csbi.dwCursorPosition.X = csbi.dwCursorPosition.X - 1;
                    } else if csbi.dwCursorPosition.Y > 0 {
                        csbi.dwCursorPosition.Y = csbi.dwCursorPosition.Y - 1;
                        csbi.dwCursorPosition.X = if csbi.dwSize.X != 0 {
                            csbi.dwSize.X - 1
                        } else {
                            0
                        };
                    }

                    SetConsoleCursorPosition(hOutput, csbi.dwCursorPosition);

                    ch = 0x0020u16;
                    ConStreamWrite(((*(*Pager).Screen).Stream).cast(), (&raw mut ch as PCTCH).cast(), 1);
                    SetConsoleCursorPosition(hOutput, csbi.dwCursorPosition);
                }

                continue;
            }
        }

        /* AddBreakHandler */
        SetConsoleCtrlHandler(None, (FALSE as i32));
        /* ConInEnable */
        GetConsoleMode(hInput, &raw mut dwMode);
        dwMode |= (ENABLE_PROCESSED_INPUT) as DWORD;
        SetConsoleMode(hInput, dwMode);

        /* Refresh the screen information, as the console may have been
         * redimensioned. Update also the default number of lines to scroll. */
        ConGetScreenInfo((*Pager).Screen, &raw mut csbi);
        (*Pager).ScrollRows = ((csbi.srWindow.Bottom as i32) - (csbi.srWindow.Top as i32)) as DWORD;

        /*
         * Erase the full line where the cursor is, and move
         * the cursor back to the beginning of the line.
         */
        ConClearLine(((*(*Pager).Screen).Stream).cast());

        /* Ctrl+C or Ctrl+Esc: Control Break */
        if fCtrl != 0 && ((KeyEvent.wVirtualKeyCode == (VK_ESCAPE as u16)) ||
                          (KeyEvent.wVirtualKeyCode == 0x0043u16))
        {
            /* We break, output a newline */
            let mut ch: WCHAR = 0x000Au16;
            ConStreamWrite(((*(*Pager).Screen).Stream).cast(), (&raw mut ch as PCTCH).cast(), 1);
            return (FALSE) as BOOL;
        }

        match chSubCommand {
            c if c == 0x0050u16 => {
                /* If we don't display other lines, just restart the prompt */
                if nLines == 0 {
                    chSubCommand = 0;
                    continue 'restart;
                }
                /* Otherwise tell the pager to display them */
                (*Pager).ScrollRows = nLines;
                return (TRUE) as BOOL;
            }
            c if c == 0x0053u16 => {
                s_dwFlags |= FLAG_PLUSn as DWORD;
                s_nNextLineNo = (*Pager).lineno.wrapping_add(nLines);
                /* Use the default Pager->ScrollRows value */
                return (TRUE) as BOOL;
            }
            _ => {
                chSubCommand = 0;
            }
        }

        /* If extended features are available */
        if (s_dwFlags & (FLAG_E)) != 0 {
            /* Ignore any key presses if Ctrl is pressed */
            if fCtrl != 0 {
                chSubCommand = 0;
                continue 'restart;
            }

            /* 'Q': Quit */
            if KeyEvent.wVirtualKeyCode == 0x0051u16 {
                /* We break, output a newline */
                let mut ch: WCHAR = 0x000Au16;
                ConStreamWrite(((*(*Pager).Screen).Stream).cast(), (&raw mut ch as PCTCH).cast(), 1);
                return (FALSE) as BOOL;
            }

            /* 'F': Next file */
            if KeyEvent.wVirtualKeyCode == 0x0046u16 {
                s_bDoNextFile = (TRUE as i32);
                return (FALSE) as BOOL;
            }

            /* '?': Show Options */
            if KeyEvent.uChar.UnicodeChar == 0x003Fu16 {
                s_fPrompt |= (PROMPT_OPTIONS) as WORD;
                continue 'restart;
            }

            /* [Enter] key: Display one line */
            if KeyEvent.wVirtualKeyCode == (VK_RETURN as u16) {
                (*Pager).ScrollRows = 1;
                return (TRUE) as BOOL;
            }

            /* [Space] key: Display one page */
            if KeyEvent.wVirtualKeyCode == (VK_SPACE as u16) {
                if (s_dwFlags & (FLAG_C)) != 0 {
                    /* Clear the screen */
                    ConClearScreen((*Pager).Screen);
                }
                /* Use the default Pager->ScrollRows value */
                return (TRUE) as BOOL;
            }

            /* 'P': Display n lines */
            if KeyEvent.wVirtualKeyCode == 0x0050u16 {
                s_fPrompt |= (PROMPT_LINES) as WORD;
                chSubCommand = 0x0050u16;
                continue 'restart;
            }

            /* 'S': Skip n lines */
            if KeyEvent.wVirtualKeyCode == 0x0053u16 {
                s_fPrompt |= (PROMPT_LINES) as WORD;
                chSubCommand = 0x0053u16;
                continue 'restart;
            }

            /* '=': Show current line number */
            if KeyEvent.uChar.UnicodeChar == 0x003Du16 {
                s_fPrompt |= (PROMPT_LINE_AT) as WORD;
                continue 'restart;
            }

            chSubCommand = 0;
            continue 'restart;
        } else {
            /* Extended features are unavailable: display one page */
            /* Use the default Pager->ScrollRows value */
            return (TRUE) as BOOL;
        }
    }
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "system" fn PagePrompt(Pager: PCON_PAGER, Done: DWORD, Total: DWORD) -> BOOL {
    let mut hInput: HANDLE = ConStreamGetOSHandle(StdIn);
    let mut hOutput: HANDLE = ConStreamGetOSHandle((*(*Pager).Screen).Stream);
    let mut csbi: CONSOLE_SCREEN_BUFFER_INFO = core::mem::zeroed();
    let mut orgCursorPosition: COORD = core::mem::zeroed();
    let mut dwMode: DWORD = 0;

    let mut KeyEvent: KEY_EVENT_RECORD = core::mem::zeroed();
    let mut fCtrl: BOOL = 0;
    let mut nLines: DWORD = 0;
    let mut chSubCommand: WCHAR = 0;

    /* Prompt strings (small size since the prompt should
     * hold ideally on one <= 80-character line) */
    static mut StrPercent: [WCHAR; 80] = [0 as WCHAR; 80];
    static mut StrLineAt: [WCHAR; 80] = [0 as WCHAR; 80];
    static mut StrOptions: [WCHAR; 80] = [0 as WCHAR; 80];
    static mut StrLines: [WCHAR; 80] = [0 as WCHAR; 80];
    static mut AreStrLoaded: BOOL = (FALSE as i32);

    let mut szPercent: [WCHAR; 80] = [0 as WCHAR; 80];
    let mut szLineAt: [WCHAR; 80] = [0 as WCHAR; 80];

    /* Load the prompt strings */
    if AreStrLoaded == 0 {
        LoadStringW(core::ptr::null_mut(), IDS_CONTINUE_PERCENT, &raw mut StrPercent as LPWSTR, (ARRAYSIZE!(StrPercent)) as i32);
        LoadStringW(core::ptr::null_mut(), IDS_CONTINUE_LINE_AT, &raw mut StrLineAt as LPWSTR, (ARRAYSIZE!(StrLineAt)) as i32);
        LoadStringW(core::ptr::null_mut(), IDS_CONTINUE_OPTIONS, &raw mut StrOptions as LPWSTR, (ARRAYSIZE!(StrOptions)) as i32);
        LoadStringW(core::ptr::null_mut(), IDS_CONTINUE_LINES, &raw mut StrLines as LPWSTR, (ARRAYSIZE!(StrLines)) as i32);
        AreStrLoaded = (TRUE as i32);
    }

    /*
     * Check whether the pager is prompting, but we have actually finished
     * to display a given file, or no data is present in STDIN anymore.
     * In this case, skip the prompt altogether. The only exception is when
     * we are displaying other files.
     */
    // TODO: Implement!

    'restart: loop {
        nLines = 0;

        /* Do not show the progress percentage when STDIN is being displayed */
        if (s_fPrompt as DWORD & ((PROMPT_PERCENT)) as DWORD) != 0 {
            /*
             * The progress percentage is evaluated as follows.
             * So far we have read a total of 'dwSumReadBytes' bytes from the file.
             * Amongst those is the latest read chunk of 'dwReadBytes' bytes, to which
             * correspond a number of 'dwReadChars' characters with which we have called
             * ConWritePaging who called PagePrompt. We then have: Total == dwReadChars.
             * During this ConWritePaging call the PagePrompt was called after 'Done'
             * number of characters over 'Total'.
             * It should be noted that for 'dwSumReadBytes' number of bytes read it
             * *roughly* corresponds 'dwSumReadChars' number of characters. This is
             * because there may be some failures happening during the conversion of
             * the bytes read to the character string for a given encoding.
             * Therefore the number of characters displayed on screen is equal to:
             *   dwSumReadChars - Total + Done ,
             * but the best corresponding approximated number of bytes would be:
             *   dwSumReadBytes - (Total - Done) * (dwSumReadBytes / dwSumReadChars) ,
             * where the ratio is the average number of bytes per character.
             * The percentage is then computed relative to the total file size.
             */
            let mut dwPercent: DWORD = dwSumReadBytes
                .wrapping_sub(
                    Total
                        .wrapping_sub(Done)
                        .wrapping_mul(dwSumReadBytes.wrapping_div(dwSumReadChars)),
                )
                .wrapping_mul(100)
                .wrapping_div(dwFileSize);
            StringCchPrintfW!(&raw mut szPercent as LPWSTR, ARRAYSIZE!(szPercent), &raw mut StrPercent as LPCWSTR, dwPercent);
        }
        if (s_fPrompt as DWORD & ((PROMPT_LINE_AT)) as DWORD) != 0 {
            StringCchPrintfW!(&raw mut szLineAt as LPWSTR, ARRAYSIZE!(szLineAt), &raw mut StrLineAt as LPCWSTR, (*Pager).lineno);
        }

        /* Suitably format and display the prompt */
        ConResMsgPrintf((*(*Pager).Screen).Stream, 0, IDS_CONTINUE_PROMPT,
                        (if (s_fPrompt as DWORD & ((PROMPT_PERCENT)) as DWORD) != 0 { &raw const szPercent as *const WCHAR } else { RUSTCONV_WSTR_96A296D224F2.as_ptr() }),
                        (if (s_fPrompt as DWORD & ((PROMPT_LINE_AT)) as DWORD) != 0 { &raw const szLineAt as *const WCHAR } else { RUSTCONV_WSTR_96A296D224F2.as_ptr() }),
                        (if (s_fPrompt as DWORD & ((PROMPT_OPTIONS)) as DWORD) != 0 { &raw const StrOptions as *const WCHAR } else { RUSTCONV_WSTR_96A296D224F2.as_ptr() }),
                        (if (s_fPrompt as DWORD & ((PROMPT_LINES)) as DWORD) != 0 { &raw const StrLines as *const WCHAR } else { RUSTCONV_WSTR_96A296D224F2.as_ptr() }));

        /* Reset the prompt to a default state */
        s_fPrompt = (s_fPrompt as DWORD & (!((PROMPT_LINE_AT) | (PROMPT_OPTIONS) | (PROMPT_LINES))) as DWORD) as WORD;

        /* RemoveBreakHandler */
        SetConsoleCtrlHandler(None, (TRUE as i32));
        /* ConInDisable */
        GetConsoleMode(hInput, &raw mut dwMode);
        dwMode &= !((ENABLE_PROCESSED_INPUT) as DWORD);
        SetConsoleMode(hInput, dwMode);

        // FIXME: Does not support TTY yet!
        ConGetScreenInfo((*Pager).Screen, &raw mut csbi);
        orgCursorPosition = csbi.dwCursorPosition;
        loop {
            let mut ir: INPUT_RECORD = core::mem::zeroed();
            let mut dwRead: DWORD = 0;
            let mut ch: WCHAR = 0;

            loop {
                ReadConsoleInput(hInput, &raw mut ir, 1, &raw mut dwRead);
                if !((ir.EventType != KEY_EVENT) || (ir.Event.KeyEvent.bKeyDown == 0)) {
                    break;
                }
            }

            /* Got our key */
            KeyEvent = ir.Event.KeyEvent;

            /* Ignore any unsupported keyboard press */
            if (KeyEvent.wVirtualKeyCode == (VK_SHIFT as u16)) ||
                (KeyEvent.wVirtualKeyCode == (VK_MENU as u16)) ||
                (KeyEvent.wVirtualKeyCode == (VK_CONTROL as u16))
            {
                continue;
            }

            /* Ctrl key is pressed? */
            fCtrl = ((KeyEvent.dwControlKeyState & ((LEFT_CTRL_PRESSED) | (RIGHT_CTRL_PRESSED))) != 0) as BOOL;

            /* Ctrl+C or Ctrl+Esc? */
            if fCtrl != 0 && ((KeyEvent.wVirtualKeyCode == (VK_ESCAPE as u16)) ||
                              (KeyEvent.wVirtualKeyCode == 0x0043u16))
            {
                chSubCommand = 0;
                break;
            }

            /* If extended features are unavailable, or no
             * pending commands, don't do more processing. */
            if (s_dwFlags & (FLAG_E)) == 0 || (chSubCommand == 0) {
                break;
            }

            ch = KeyEvent.uChar.UnicodeChar;
            if 0x0030u16 <= ch && ch <= 0x0039u16 {
                nLines = nLines.wrapping_mul(10);
                nLines = nLines.wrapping_add((ch as DWORD).wrapping_sub(0x0030u16 as DWORD));
                ConStreamWrite((*(*Pager).Screen).Stream, (&raw mut ch as PCTCH).cast(), 1);
                continue;
            } else if KeyEvent.wVirtualKeyCode == (VK_RETURN as u16) {
                /* Validate the line number */
                break;
            } else if KeyEvent.wVirtualKeyCode == (VK_ESCAPE as u16) {
                /* Cancel the current command */
                chSubCommand = 0;
                break;
            } else if KeyEvent.wVirtualKeyCode == (VK_BACK as u16) {
                if nLines != 0 {
                    nLines /= 10;
                }

                /* Erase the current character */
                ConGetScreenInfo((*Pager).Screen, &raw mut csbi);
                if (csbi.dwCursorPosition.Y > orgCursorPosition.Y) ||
                    ((csbi.dwCursorPosition.Y == orgCursorPosition.Y) &&
                     (csbi.dwCursorPosition.X > orgCursorPosition.X))
                {
                    if csbi.dwCursorPosition.X > 0 {
                        csbi.dwCursorPosition.X = csbi.dwCursorPosition.X - 1;
                    } else if csbi.dwCursorPosition.Y > 0 {
                        csbi.dwCursorPosition.Y = csbi.dwCursorPosition.Y - 1;
                        csbi.dwCursorPosition.X = if csbi.dwSize.X != 0 {
                            csbi.dwSize.X - 1
                        } else {
                            0
                        };
                    }

                    SetConsoleCursorPosition(hOutput, csbi.dwCursorPosition);

                    ch = 0x0020u16;
                    ConStreamWrite((*(*Pager).Screen).Stream, (&raw mut ch as PCTCH).cast(), 1);
                    SetConsoleCursorPosition(hOutput, csbi.dwCursorPosition);
                }

                continue;
            }
        }

        /* AddBreakHandler */
        SetConsoleCtrlHandler(None, (FALSE as i32));
        /* ConInEnable */
        GetConsoleMode(hInput, &raw mut dwMode);
        dwMode |= (ENABLE_PROCESSED_INPUT) as DWORD;
        SetConsoleMode(hInput, dwMode);

        /* Refresh the screen information, as the console may have been
         * redimensioned. Update also the default number of lines to scroll. */
        ConGetScreenInfo((*Pager).Screen, &raw mut csbi);
        (*Pager).ScrollRows = ((csbi.srWindow.Bottom as i32) - (csbi.srWindow.Top as i32)) as DWORD;

        /*
         * Erase the full line where the cursor is, and move
         * the cursor back to the beginning of the line.
         */
        ConClearLine((*(*Pager).Screen).Stream);

        /* Ctrl+C or Ctrl+Esc: Control Break */
        if fCtrl != 0 && ((KeyEvent.wVirtualKeyCode == (VK_ESCAPE as u16)) ||
                          (KeyEvent.wVirtualKeyCode == 0x0043u16))
        {
            /* We break, output a newline */
            let mut ch: WCHAR = 0x000Au16;
            ConStreamWrite((*(*Pager).Screen).Stream, (&raw mut ch as PCTCH).cast(), 1);
            return (FALSE) as BOOL;
        }

        match chSubCommand {
            c if c == 0x0050u16 => {
                /* If we don't display other lines, just restart the prompt */
                if nLines == 0 {
                    chSubCommand = 0;
                    continue 'restart;
                }
                /* Otherwise tell the pager to display them */
                (*Pager).ScrollRows = nLines;
                return (TRUE) as BOOL;
            }
            c if c == 0x0053u16 => {
                s_dwFlags |= FLAG_PLUSn as DWORD;
                s_nNextLineNo = (*Pager).lineno.wrapping_add(nLines);
                /* Use the default Pager->ScrollRows value */
                return (TRUE) as BOOL;
            }
            _ => {
                chSubCommand = 0;
            }
        }

        /* If extended features are available */
        if (s_dwFlags & (FLAG_E)) != 0 {
            /* Ignore any key presses if Ctrl is pressed */
            if fCtrl != 0 {
                chSubCommand = 0;
                continue 'restart;
            }

            /* 'Q': Quit */
            if KeyEvent.wVirtualKeyCode == 0x0051u16 {
                /* We break, output a newline */
                let mut ch: WCHAR = 0x000Au16;
                ConStreamWrite((*(*Pager).Screen).Stream, (&raw mut ch as PCTCH).cast(), 1);
                return (FALSE) as BOOL;
            }

            /* 'F': Next file */
            if KeyEvent.wVirtualKeyCode == 0x0046u16 {
                s_bDoNextFile = (TRUE as i32);
                return (FALSE) as BOOL;
            }

            /* '?': Show Options */
            if KeyEvent.uChar.UnicodeChar == 0x003Fu16 {
                s_fPrompt |= (PROMPT_OPTIONS) as WORD;
                continue 'restart;
            }

            /* [Enter] key: Display one line */
            if KeyEvent.wVirtualKeyCode == (VK_RETURN as u16) {
                (*Pager).ScrollRows = 1;
                return (TRUE) as BOOL;
            }

            /* [Space] key: Display one page */
            if KeyEvent.wVirtualKeyCode == (VK_SPACE as u16) {
                if (s_dwFlags & (FLAG_C)) != 0 {
                    /* Clear the screen */
                    ConClearScreen((*Pager).Screen);
                }
                /* Use the default Pager->ScrollRows value */
                return (TRUE) as BOOL;
            }

            /* 'P': Display n lines */
            if KeyEvent.wVirtualKeyCode == 0x0050u16 {
                s_fPrompt |= (PROMPT_LINES) as WORD;
                chSubCommand = 0x0050u16;
                continue 'restart;
            }

            /* 'S': Skip n lines */
            if KeyEvent.wVirtualKeyCode == 0x0053u16 {
                s_fPrompt |= (PROMPT_LINES) as WORD;
                chSubCommand = 0x0053u16;
                continue 'restart;
            }

            /* '=': Show current line number */
            if KeyEvent.uChar.UnicodeChar == 0x003Du16 {
                s_fPrompt |= (PROMPT_LINE_AT) as WORD;
                continue 'restart;
            }

            chSubCommand = 0;
            continue 'restart;
        } else {
            /* Extended features are unavailable: display one page */
            /* Use the default Pager->ScrollRows value */
            return (TRUE) as BOOL;
        }
    }
}

unsafe extern "C" fn IsDataUnicode(
    mut Buffer: PVOID,
    mut BufferSize: DWORD,
    mut Encoding: *mut ENCODING,
    mut SkipBytes: PDWORD,
) -> BOOL {
    let mut pBytes: PBYTE = Buffer as PBYTE;
    let mut encFile: ENCODING = ENCODING_ANSI;
    let mut dwPos: DWORD = 0;

    /* Look for Byte Order Marks */
    if BufferSize >= 2
        && *pBytes.add(0 as usize) == 0xFF
        && *pBytes.add(1 as usize) == 0xFE
    {
        encFile = ENCODING_UTF16LE;
        dwPos = 2;
    } else if BufferSize >= 2
        && *pBytes.add(0 as usize) == 0xFE
        && *pBytes.add(1 as usize) == 0xFF
    {
        encFile = ENCODING_UTF16BE;
        dwPos = 2;
    } else if BufferSize >= 3
        && *pBytes.add(0 as usize) == 0xEF
        && *pBytes.add(1 as usize) == 0xBB
        && *pBytes.add(2 as usize) == 0xBF
    {
        encFile = ENCODING_UTF8;
        dwPos = 3;
    } else {
        const IS_TEXT_UNKNOWN_FLAGS_MASK: INT = (7 << 13) | (1 << 11);

        /* Flag out the unknown flags here, the passed tests will not have them either */
        let mut Tests: INT = ((IS_TEXT_UNICODE_NOT_ASCII_MASK
            | IS_TEXT_UNICODE_NOT_UNICODE_MASK
            | IS_TEXT_UNICODE_REVERSE_MASK
            | IS_TEXT_UNICODE_UNICODE_MASK)
            & (!IS_TEXT_UNKNOWN_FLAGS_MASK) as u32) as i32;
        let mut Results: INT;

        IsTextUnicode(
            Buffer as *const ::core::ffi::c_void,
            BufferSize as ::core::ffi::c_int,
            core::ptr::addr_of_mut!(Tests),
        );
        Results = Tests;

        /* Filter out NULL_BYTES / ILLEGAL_CHARS expectations */
        Results &= (!(IS_TEXT_UNICODE_NULL_BYTES | IS_TEXT_UNICODE_ILLEGAL_CHARS)) as i32;

        if (Results & (IS_TEXT_UNICODE_NOT_UNICODE_MASK as i32)) == 0
            && (Results & (IS_TEXT_UNICODE_REVERSE_MASK as i32)) == 0
            && (Results & (IS_TEXT_UNICODE_UNICODE_MASK as i32)) != 0
            && (Tests & (IS_TEXT_UNICODE_NULL_BYTES as i32)) != 0
        {
            encFile = ENCODING_UTF16LE;
            dwPos = if (Results & (IS_TEXT_UNICODE_SIGNATURE as i32)) != 0 {
                2
            } else {
                0
            } as DWORD;
        } else if (Results & (IS_TEXT_UNICODE_NOT_UNICODE_MASK as i32)) == 0
            && (Results & (IS_TEXT_UNICODE_UNICODE_MASK as i32)) == 0
            && (Results & (IS_TEXT_UNICODE_REVERSE_MASK as i32)) != 0
            && (Tests & (IS_TEXT_UNICODE_NULL_BYTES as i32)) != 0
        {
            encFile = ENCODING_UTF16BE;
            dwPos = if (Results & (IS_TEXT_UNICODE_REVERSE_SIGNATURE as i32)) != 0 {
                2
            } else {
                0
            } as DWORD;
        } else {
            /* Invalid / binary data: fall back to ANSI-like default */
            encFile = ENCODING_ANSI; // ENCODING_UTF8;
            dwPos = 0;
        }
    }

    if !Encoding.is_null() {
        *Encoding = encFile;
    }
    if !SkipBytes.is_null() {
        *SkipBytes = dwPos;
    }

    return (encFile as ::core::ffi::c_uint
        != ENCODING_ANSI as ::core::ffi::c_uint) as BOOL;
}

unsafe extern "C" fn FileGetString(
    hFileParam: HANDLE,
    Encoding: ENCODING,
    pCacheBuffer: PVOID,
    CacheBufferLength: DWORD,
    pBuffer: *mut PWCHAR,
    pnBufferLength: PDWORD,
    pdwReadBytes: PDWORD,
    pdwReadChars: PDWORD,
) -> BOOL {
    let mut Success: BOOL = 0;
    let mut CodePage: UINT = -(1 as ::core::ffi::c_int) as UINT;
    let mut dwReadBytes: DWORD = 0;
    let mut len: INT = 0;

    /* Always reset the retrieved number of bytes/characters */
    if !pdwReadBytes.is_null() {
        *pdwReadBytes = 0 as DWORD;
    }
    if !pdwReadChars.is_null() {
        *pdwReadChars = 0 as DWORD;
    }

    Success = ReadFile(
        hFileParam,
        pCacheBuffer,
        CacheBufferLength,
        core::ptr::addr_of_mut!(dwReadBytes),
        core::ptr::null_mut(),
    );
    if Success == 0 || dwReadBytes == 0 as DWORD {
        return FALSE as BOOL;
    }

    if !pdwReadBytes.is_null() {
        *pdwReadBytes = dwReadBytes;
    }

    if Encoding as ::core::ffi::c_uint == ENCODING_ANSI as ::core::ffi::c_uint
        || Encoding as ::core::ffi::c_uint == ENCODING_UTF8 as ::core::ffi::c_uint
    {
        /* Conversion is needed */

        if Encoding as ::core::ffi::c_uint == ENCODING_ANSI as ::core::ffi::c_uint {
            CodePage = GetConsoleCP(); // CP_ACP; // FIXME: Cache GetConsoleCP() value.
        } else {
            // if (Encoding == ENCODING_UTF8)
            CodePage = CP_UTF8 as UINT;
        }

        /* Retrieve the needed buffer size */
        len = MultiByteToWideChar(
            CodePage,
            0 as DWORD,
            pCacheBuffer as LPCSTR,
            dwReadBytes as ::core::ffi::c_int,
            core::ptr::null_mut(),
            0 as ::core::ffi::c_int,
        ) as INT;
        if len == 0 as INT {
            /* Failure, bail out */
            return FALSE as BOOL;
        }

        /* Initialize the conversion buffer if needed... */
        if (*pBuffer).is_null() {
            *pnBufferLength = len as DWORD;
            *pBuffer = HeapAlloc(
                GetProcessHeap(),
                0 as DWORD,
                (*pnBufferLength as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
            ) as PWCHAR;
            if (*pBuffer).is_null() {
                // *pBuffer = NULL;
                *pnBufferLength = 0 as DWORD;
                // WARN("DEBUG: Cannot allocate memory for *pBuffer!\n");
                // ConErrFormatMessage(GetLastError());
                return FALSE as BOOL;
            }
        }
        /* ... or reallocate only if the new length is greater than the old one */
        else if len as DWORD > *pnBufferLength {
            let mut OldBuffer: PWSTR = *pBuffer;

            *pnBufferLength = len as DWORD;
            *pBuffer = HeapReAlloc(
                GetProcessHeap(),
                0 as DWORD,
                *pBuffer as PVOID,
                (*pnBufferLength as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
            ) as PWCHAR;
            if (*pBuffer).is_null() {
                /* Do not leak old buffer */
                HeapFree(
                    GetProcessHeap(),
                    0 as DWORD,
                    OldBuffer as PVOID,
                );
                // *pBuffer = NULL;
                *pnBufferLength = 0 as DWORD;
                // WARN("DEBUG: Cannot reallocate memory for *pBuffer!\n");
                // ConErrFormatMessage(GetLastError());
                return FALSE as BOOL;
            }
        }

        /* Now perform the conversion proper */
        len = MultiByteToWideChar(
            CodePage,
            0 as DWORD,
            pCacheBuffer as LPCSTR,
            dwReadBytes as ::core::ffi::c_int,
            *pBuffer,
            len as ::core::ffi::c_int,
        ) as INT;
        dwReadBytes = len as DWORD;
    } else {
        /*
         * No conversion needed, just convert from big to little endian if needed.
         * pBuffer and pnBufferLength are left untouched and pCacheBuffer can be
         * directly used.
         */
        let mut pWChars: PWCHAR = pCacheBuffer as PWCHAR;
        let mut i: DWORD = 0;

        dwReadBytes /= core::mem::size_of::<WCHAR>() as DWORD;

        if Encoding as ::core::ffi::c_uint == ENCODING_UTF16BE as ::core::ffi::c_uint {
            i = 0 as DWORD;
            while i < dwReadBytes {
                /* Equivalent to RtlUshortByteSwap: reverse high/low bytes */
                *pWChars.offset(i as isize) = MAKEWORD!(
                    HIBYTE!(*pWChars.offset(i as isize)),
                    LOBYTE!(*pWChars.offset(i as isize)),
                ) as WCHAR;
                i = i.wrapping_add(1);
            }
        }
        // else if (Encoding == ENCODING_UTF16LE), we are good, nothing to do.
    }

    /* Return the number of characters (dwReadBytes is converted) */
    if !pdwReadChars.is_null() {
        *pdwReadChars = dwReadBytes;
    }

    return TRUE as BOOL;
}

unsafe extern "C" fn LoadRegistrySettings(hKeyRoot: HKEY) {
    let mut lRet: LONG;
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut dwType: DWORD = 0;
    let mut len: DWORD;
    /*
     * Buffer big enough to hold the string L"4294967295",
     * corresponding to the literal 0xFFFFFFFF (MAXULONG) in decimal.
     */
    let mut Buffer: [WCHAR; 11] = [0; 11];
    C_ASSERT!(core::mem::size_of::<[WCHAR; 11]>() >= core::mem::size_of::<DWORD>());

    lRet = RegOpenKeyExW(
        hKeyRoot,
        RUSTCONV_WSTR_CE9E0758CAAD.as_ptr(),
        0,
        KEY_QUERY_VALUE,
        core::ptr::addr_of_mut!(hKey),
    );
    if (lRet) as u32 != ERROR_SUCCESS {
        return;
    }

    len = core::mem::size_of::<[WCHAR; 11]>() as DWORD;
    lRet = RegQueryValueExW(
        hKey,
        RUSTCONV_WSTR_1006EE11D11A.as_ptr(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(dwType),
        core::ptr::addr_of_mut!(Buffer) as PBYTE,
        core::ptr::addr_of_mut!(len),
    );
    if (lRet) as u32 == ERROR_SUCCESS {
        /* Overwrite the default setting */
        if dwType == REG_DWORD {
            bEnableExtensions = (*(core::ptr::addr_of_mut!(Buffer) as PDWORD) != 0) as BOOL;
        } else if dwType == REG_SZ {
            bEnableExtensions = (_wtol(core::ptr::addr_of_mut!(Buffer) as PWSTR)
                == 1) as BOOL;
        }
    }
    // else, use the default setting set globally.

    RegCloseKey(hKey);
}

unsafe extern "C" fn IsFlag(param: PCWSTR) -> BOOL {
    let mut pch: PCWSTR;
    let mut endptr: PWCHAR = core::ptr::null_mut();

    if *param.offset(0) == 0x002Fu16 {
        return (TRUE) as BOOL;
    }

    if *param.offset(0) == 0x002Bu16 {
        pch = param.offset(1);
        if *pch != 0 {
            let _ = wcstol(pch, core::ptr::addr_of_mut!(endptr), 10);
            return (*endptr == 0) as BOOL;
        }
    }
    return (FALSE) as BOOL;
}

unsafe extern "C" fn ParseArgument(arg: PCWSTR, pbHasFiles: *mut BOOL) -> BOOL {
    let mut endptr: PWCHAR = core::ptr::null_mut();

    if *arg.offset(0 as isize) == 0x002Fu16 {
        let sw: u16 = towupper((*arg.offset(1 as isize) as wint_t) as _) as u16;
        if sw == 0x003Fu16 {
            if *arg.offset(2 as isize) == 0 {
                s_dwFlags |= FLAG_HELP;
                return (TRUE) as BOOL;
            }
        } else if sw == 0x0045u16 {
            if *arg.offset(2 as isize) == 0 {
                s_dwFlags |= FLAG_E;
                return (TRUE) as BOOL;
            }
        } else if sw == 0x0043u16 {
            if *arg.offset(2 as isize) == 0 {
                s_dwFlags |= FLAG_C;
                return (TRUE) as BOOL;
            }
        } else if sw == 0x0050u16 {
            if *arg.offset(2 as isize) == 0 {
                s_dwFlags |= FLAG_P;
                return (TRUE) as BOOL;
            }
        } else if sw == 0x0053u16 {
            if *arg.offset(2 as isize) == 0 {
                s_dwFlags |= FLAG_S;
                return (TRUE) as BOOL;
            }
        } else if sw == 0x0054u16 {
            if *arg.offset(2 as isize) != 0 {
                s_dwFlags |= FLAG_Tn as DWORD;
                s_nTabWidth = wcstol(
                    arg.offset(2 as isize) as *const wchar_t,
                    core::ptr::addr_of_mut!(endptr),
                    10,
                ) as LONG;
                if *endptr == 0 {
                    return (TRUE) as BOOL;
                }
            }
        }
    } else if *arg.offset(0 as isize) == 0x002Bu16 {
        if *arg.offset(1 as isize) != 0 {
            s_dwFlags |= FLAG_PLUSn as DWORD;
            s_nNextLineNo = (wcstol(
                arg.offset(1 as isize) as *const wchar_t,
                core::ptr::addr_of_mut!(endptr),
                10,
            ) + 1) as DWORD;
            if *endptr == 0 {
                return (TRUE) as BOOL;
            }
        }
    }

    if IsFlag(arg) != 0 {
        ConResPrintf(StdErr, IDS_BAD_FLAG, arg);
        return (FALSE) as BOOL;
    } else {
        *pbHasFiles = (TRUE as i32);
    }

    return (TRUE) as BOOL;
}

unsafe extern "C" fn ParseMoreVariable(pbHasFiles: *mut BOOL) -> BOOL {
    let mut ret: BOOL = (TRUE as i32);
    let mut psz: PWSTR = core::ptr::null_mut();
    let mut pch: PWCHAR = core::ptr::null_mut();
    let mut cch: DWORD;

    cch = GetEnvironmentVariableW(RUSTCONV_WSTR_669336782398.as_ptr(), core::ptr::null_mut(), 0);
    if cch == 0 {
        return (TRUE) as BOOL;
    }

    psz = malloc(
        (cch.wrapping_add(1) as usize).wrapping_mul(core::mem::size_of::<WCHAR>()),
    ) as PWSTR;
    if psz.is_null() {
        return (TRUE) as BOOL;
    }

    if GetEnvironmentVariableW(RUSTCONV_WSTR_669336782398.as_ptr(), psz, cch.wrapping_add(1)) == 0 {
        free(psz as *mut core::ffi::c_void);
        return (TRUE) as BOOL;
    }

    pch = wcstok(psz, RUSTCONV_WSTR_8D71B3FAAB82.as_ptr()) as PWCHAR;
    while !pch.is_null() {
        ret = ParseArgument(pch as PCWSTR, pbHasFiles);
        if ret == 0 {
            break;
        }
        pch = wcstok(core::ptr::null_mut(), RUSTCONV_WSTR_8D71B3FAAB82.as_ptr()) as PWCHAR;
    }

    free(psz as *mut core::ffi::c_void);
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn wmain(mut argc: core::ffi::c_int, mut argv: *mut *mut WCHAR) -> core::ffi::c_int {
    // FIXME this stuff!
    let mut Screen: CON_SCREEN = core::mem::zeroed();
    Screen.Stream = StdOut;
    let mut Pager: CON_PAGER = core::mem::zeroed();
    Pager.Screen = core::ptr::addr_of_mut!(Screen);
    Pager.PageColumns = 0;

    let mut i: core::ffi::c_int = 0;

    let mut bRet: BOOL = 0;
    let mut bContinue: BOOL = 0;

    let mut Encoding: ENCODING = ENCODING_ANSI;
    let mut SkipBytes: DWORD = 0;
    let mut HasFiles: BOOL = 0;

    let mut FileCacheBuffer: PVOID = core::ptr::null_mut();
    let mut StringBuffer: PWCHAR = core::ptr::null_mut();
    let mut StringBufferLength: DWORD = 0;
    let mut dwReadBytes: DWORD = 0;
    let mut dwReadChars: DWORD = 0;

    let mut szFullPath: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    hStdIn = GetStdHandle(STD_INPUT_HANDLE);
    hStdOut = GetStdHandle(STD_OUTPUT_HANDLE);

    /* Initialize the Console Standard Streams */
    ConStreamInit(StdIn, GetStdHandle(STD_INPUT_HANDLE), UTF8Text, INVALID_CP);
    ConStreamInit(StdOut, GetStdHandle(STD_OUTPUT_HANDLE), UTF8Text, INVALID_CP);
    ConStreamInit(StdErr, GetStdHandle(STD_ERROR_HANDLE), UTF8Text, INVALID_CP);

    /*
     * Bad usage (too much options) or we use the /? switch.
     * Display help for the MORE command.
     */
    if argc > 1
        && wcscmp(*argv.offset(1 as isize), RUSTCONV_WSTR_5462AED96D21.as_ptr()) == 0
    {
        ConResPuts(StdOut, IDS_USAGE);
        return 0;
    }

    /* Load the registry settings */
    LoadRegistrySettings(HKEY_LOCAL_MACHINE);
    LoadRegistrySettings(HKEY_CURRENT_USER);
    if bEnableExtensions != 0 {
        s_dwFlags |= FLAG_E;
    }

    // NOTE: We might try to duplicate the ConOut for read access... ?
    hKeyboard = CreateFileW(
        RUSTCONV_WSTR_B5E9C1A0FAD8.as_ptr(),
        GENERIC_READ | GENERIC_WRITE,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        core::ptr::null_mut(),
        OPEN_EXISTING,
        0,
        core::ptr::null_mut(),
    );
    FlushConsoleInputBuffer(hKeyboard);
    ConStreamSetOSHandle(StdIn, hKeyboard);

    FileCacheBuffer = HeapAlloc(GetProcessHeap(), 0, 4096);
    if FileCacheBuffer.is_null() {
        ConPuts(StdErr, RUSTCONV_WSTR_F5DC9649106A.as_ptr());
        CloseHandle(hKeyboard);
        return 1;
    }

    /* First, load the "MORE" environment variable and parse it,
     * then parse the command-line parameters. */
    HasFiles = (FALSE as i32);
    if ParseMoreVariable(core::ptr::addr_of_mut!(HasFiles)) == 0 {
        return 1;
    }
    i = 1;
    while i < argc {
        if ParseArgument(*argv.offset(i as isize), core::ptr::addr_of_mut!(HasFiles)) == 0 {
            return 1;
        }
        i += 1;
    }

    if (s_dwFlags & FLAG_HELP) != 0 {
        ConResPuts(StdOut, IDS_USAGE);
        return 0;
    }

    Pager.PagerLine = Some(MorePagerLine);
    Pager.dwFlags |= CON_PAGER_EXPAND_TABS | CON_PAGER_CACHE_INCOMPLETE_LINE;
    if (s_dwFlags & FLAG_P) != 0 {
        Pager.dwFlags |= CON_PAGER_EXPAND_FF;
    }
    Pager.nTabWidth = s_nTabWidth;

    'quit: {
        /* Special case where we run 'MORE' without any argument: we use STDIN */
        if HasFiles == 0 {
            /*
             * Assign STDIN handle to hFile so that the page prompt function will
             * know the data comes from STDIN, and will take different actions.
             */
            hFile = hStdIn;

            /* Update the statistics for PagePrompt */
            dwFileSize = 0;
            dwSumReadChars = 0;
            dwSumReadBytes = dwSumReadChars;

            /* We suppose we read text from the file */

            /* For STDIN we always suppose we are in ANSI mode */
            // SetFilePointer(hFile, 0, NULL, FILE_BEGIN);
            Encoding = ENCODING_ANSI; // ENCODING_UTF8;

            /* Start paging */
            bContinue = ConWritePaging(
                core::ptr::addr_of_mut!(Pager),
                Some(PagePrompt),
                (TRUE as i32),
                core::ptr::null_mut(),
                0,
            );
            if bContinue == 0 {
                break 'quit;
            }

            loop {
                bRet = FileGetString(
                    hFile,
                    Encoding,
                    FileCacheBuffer,
                    4096,
                    core::ptr::addr_of_mut!(StringBuffer),
                    core::ptr::addr_of_mut!(StringBufferLength),
                    core::ptr::addr_of_mut!(dwReadBytes),
                    core::ptr::addr_of_mut!(dwReadChars),
                );
                if bRet == 0 || dwReadBytes == 0 || dwReadChars == 0 {
                    /* We failed at reading the file, bail out */
                    break;
                }

                /* Update the statistics for PagePrompt */
                dwSumReadBytes = dwSumReadBytes.wrapping_add(dwReadBytes);
                dwSumReadChars = dwSumReadChars.wrapping_add(dwReadChars);

                bContinue = ConWritePaging(
                    core::ptr::addr_of_mut!(Pager),
                    Some(PagePrompt),
                    (FALSE as i32),
                    StringBuffer as PCTCH,
                    (dwReadChars as SIZE_T) as u32,
                );
                /* If we Ctrl-C/Ctrl-Break, stop everything */
                if bContinue == 0 {
                    break;
                }
                if !(bRet != 0 && dwReadBytes > 0) {
                    break;
                }
            }

            /* Flush any cached pager buffers */
            if bContinue != 0 {
                bContinue = ConWritePaging(
                    core::ptr::addr_of_mut!(Pager),
                    Some(PagePrompt),
                    (FALSE as i32),
                    core::ptr::null_mut(),
                    0,
                );
            }

            break 'quit;
        }

        /* We have files: read them and output them to STDOUT */
        i = 1;
        while i < argc {
            'body: {
                if IsFlag(*argv.offset(i as isize)) != 0 {
                    break 'body;
                }

                GetFullPathNameW(
                    *argv.offset(i as isize),
                    ARRAYSIZE!(szFullPath),
                    core::ptr::addr_of_mut!(szFullPath) as *mut TCHAR,
                    core::ptr::null_mut(),
                );
                hFile = CreateFileW(
                    core::ptr::addr_of_mut!(szFullPath) as *mut TCHAR,
                    GENERIC_READ,
                    FILE_SHARE_READ,
                    core::ptr::null_mut(),
                    OPEN_EXISTING,
                    0, // FILE_ATTRIBUTE_NORMAL,
                    core::ptr::null_mut(),
                );
                if hFile == INVALID_HANDLE_VALUE {
                    ConResPrintf(
                        StdErr,
                        IDS_FILE_ACCESS,
                        core::ptr::addr_of_mut!(szFullPath) as *mut TCHAR,
                    );
                    break 'quit;
                }

                /* We currently do not support files too big */
                dwFileSize = GetFileSize(hFile, core::ptr::null_mut());
                if dwFileSize == INVALID_FILE_SIZE {
                    ConPuts(StdErr, RUSTCONV_WSTR_A3F18BB6EFE2.as_ptr());
                    CloseHandle(hFile);
                    break 'body;
                }

                /* We suppose we read text from the file */

                /* Check whether the file is UNICODE and retrieve its encoding */
                SetFilePointer(hFile, 0, core::ptr::null_mut(), FILE_BEGIN);
                bRet = ReadFile(
                    hFile,
                    FileCacheBuffer,
                    4096,
                    core::ptr::addr_of_mut!(dwReadBytes),
                    core::ptr::null_mut(),
                );
                IsDataUnicode(
                    FileCacheBuffer,
                    dwReadBytes,
                    core::ptr::addr_of_mut!(Encoding),
                    core::ptr::addr_of_mut!(SkipBytes),
                );
                SetFilePointer(hFile, (SkipBytes) as LONG, core::ptr::null_mut(), FILE_BEGIN);

                /* Reset state for paging */
                s_nNextLineNo = 0;
                s_bPrevLineIsBlank = (FALSE as i32);
                s_fPrompt = PROMPT_PERCENT;
                s_bDoNextFile = (FALSE as i32);

                /* Update the statistics for PagePrompt */
                dwSumReadChars = 0;
                dwSumReadBytes = dwSumReadChars;

                /* Start paging */
                bContinue = ConWritePaging(
                    core::ptr::addr_of_mut!(Pager),
                    Some(PagePrompt),
                    (TRUE as i32),
                    core::ptr::null_mut(),
                    0,
                );
                if bContinue == 0 {
                    /* We stop displaying this file */
                    CloseHandle(hFile);
                    if s_bDoNextFile != 0 {
                        /* Bail out and continue with the other files */
                        break 'body;
                    }

                    /* We Ctrl-C/Ctrl-Break, stop everything */
                    break 'quit;
                }

                loop {
                    bRet = FileGetString(
                        hFile,
                        Encoding,
                        FileCacheBuffer,
                        4096,
                        core::ptr::addr_of_mut!(StringBuffer),
                        core::ptr::addr_of_mut!(StringBufferLength),
                        core::ptr::addr_of_mut!(dwReadBytes),
                        core::ptr::addr_of_mut!(dwReadChars),
                    );
                    if bRet == 0 || dwReadBytes == 0 || dwReadChars == 0 {
                        /*
                         * We failed at reading the file, bail out
                         * and continue with the other files.
                         */
                        break;
                    }

                    /* Update the statistics for PagePrompt */
                    dwSumReadBytes = dwSumReadBytes.wrapping_add(dwReadBytes);
                    dwSumReadChars = dwSumReadChars.wrapping_add(dwReadChars);

                    if (Encoding == ENCODING_UTF16LE) || (Encoding == ENCODING_UTF16BE) {
                        bContinue = ConWritePaging(
                            core::ptr::addr_of_mut!(Pager),
                            Some(PagePrompt),
                            (FALSE as i32),
                            FileCacheBuffer as PCTCH,
                            (dwReadChars as SIZE_T) as u32,
                        );
                    } else {
                        bContinue = ConWritePaging(
                            core::ptr::addr_of_mut!(Pager),
                            Some(PagePrompt),
                            (FALSE as i32),
                            StringBuffer as PCTCH,
                            (dwReadChars as SIZE_T) as u32,
                        );
                    }
                    if bContinue == 0 {
                        /* We stop displaying this file */
                        break;
                    }
                    if !(bRet != 0 && dwReadBytes > 0) {
                        break;
                    }
                }

                /* Flush any cached pager buffers */
                if bContinue != 0 {
                    bContinue = ConWritePaging(
                        core::ptr::addr_of_mut!(Pager),
                        Some(PagePrompt),
                        (FALSE as i32),
                        core::ptr::null_mut(),
                        0,
                    );
                }

                CloseHandle(hFile);

                /* Check whether we should stop displaying this file */
                if bContinue == 0 {
                    if s_bDoNextFile != 0 {
                        /* Bail out and continue with the other files */
                        break 'body;
                    }

                    /* We Ctrl-C/Ctrl-Break, stop everything */
                    break 'quit;
                }
            }
            i += 1;
        }
    }

    // Quit:
    if !StringBuffer.is_null() {
        HeapFree(GetProcessHeap(), 0, StringBuffer as PVOID);
    }
    HeapFree(GetProcessHeap(), 0, FileCacheBuffer);
    CloseHandle(hKeyboard);
    return 0;
}
