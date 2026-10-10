use crate::cmd::console::ConOutChar;
use crate::cmd::console::ConOutResPaging;
use crate::cmd::console::GetCursorY;
use crate::cmd::locale::GetDateString;
use crate::cmd::locale::GetTimeString;
use ntdll_sys::nt::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut DefaultPrompt: [TCHAR; 5] =
    unsafe { ::core::mem::transmute::<[u8; 10], [TCHAR; 5]>(*b"$\0P\0$\0G\0\0\0") };
#[no_mangle]
pub unsafe extern "C" fn InitPrompt() {
    let mut Buffer: [TCHAR; 2] = core::mem::zeroed();
    let PromptName: [u16; 7] = ['P' as u16, 'R' as u16, 'O' as u16, 'M' as u16, 'P' as u16, 'T' as u16, 0];

    /*
     * Set the PROMPT environment variable if it doesn't exist already.
     * You can change the PROMPT environment variable before cmd starts.
     */
    if GetEnvironmentVariable(
        core::ptr::addr_of!(PromptName) as LPTSTR,
        core::ptr::addr_of_mut!(Buffer) as LPTSTR,
        _countof(Buffer)
    ) == 0
    {
        SetEnvironmentVariable(
            core::ptr::addr_of!(PromptName) as LPTSTR,
            core::ptr::addr_of!(DefaultPrompt) as LPTSTR
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn HasInfoLine() -> BOOL {
    let mut pr: LPTSTR = core::ptr::null_mut();
    let mut szPrompt: [TCHAR; 256] = [0; 256];
    let PromptName: [u16; 7] = ['P' as u16, 'R' as u16, 'O' as u16, 'M' as u16, 'P' as u16, 'T' as u16, 0];
    if GetEnvironmentVariable(
        PromptName.as_ptr(),
        szPrompt.as_mut_ptr(),
        _countof(szPrompt) as DWORD,
    ) != 0
    {
        pr = szPrompt.as_mut_ptr();
        while *pr != 0 {
            let fresh31 = pr;
            pr = pr.add(1);
            if *fresh31 != _T('$') {
                continue;
            }
            if *pr == 0 || {
                let fresh32 = pr;
                pr = pr.add(1);
                _totupper(*fresh32) != _T('I')
            } {
                continue;
            }
            return TRUE;
        }
    }
    return FALSE;
}
#[no_mangle]
pub unsafe extern "C" fn PrintInfoLine() {
    let mut hOutput: HANDLE = GetStdHandle(STD_OUTPUT_HANDLE);
    let mut csbi: CONSOLE_SCREEN_BUFFER_INFO = core::mem::zeroed();
    let mut coPos: COORD = core::mem::zeroed();
    let mut dwWritten: DWORD = 0;

    let mut pszInfoLine: PTSTR = core::ptr::null_mut();
    let mut iInfoLineLen: INT = 0;

    /* Return directly if the output handle is not a console handle */
    if GetConsoleScreenBufferInfo(hOutput, &raw mut csbi) == 0 {
        return;
    }

    iInfoLineLen = LoadString(CMD_ModuleHandle, STRING_CMD_INFOLINE, &raw mut pszInfoLine as PTSTR, 0);
    if pszInfoLine.is_null() || iInfoLineLen == 0 {
        return;
    }

    /* Display the localized information line */
    coPos.X = 0;
    coPos.Y = 0;
    FillConsoleOutputAttribute(hOutput, BACKGROUND_BLUE | FOREGROUND_WHITE,
                               csbi.dwSize.X,
                               coPos, &raw mut dwWritten);
    FillConsoleOutputCharacter(hOutput, _T(' '),
                               csbi.dwSize.X,
                               coPos, &raw mut dwWritten);

    WriteConsoleOutputCharacter(hOutput, pszInfoLine, iInfoLineLen,
                                coPos, &raw mut dwWritten);
}
#[no_mangle]
pub unsafe extern "C" fn PrintPrompt() {
    let mut pr: LPTSTR = core::ptr::null_mut();
    let mut Prompt: LPTSTR = core::ptr::null_mut();
    let mut szPrompt: [TCHAR; 256] = [0 as TCHAR; 256];
    let mut szPath: [TCHAR; MAX_PATH] = [0 as TCHAR; MAX_PATH];

    if GetEnvironmentVariable(
        ['P' as TCHAR, 'R' as TCHAR, 'O' as TCHAR, 'M' as TCHAR, 'P' as TCHAR, 'T' as TCHAR, 0 as TCHAR].as_ptr(),
        szPrompt.as_mut_ptr(),
        _countof(szPrompt),
    ) != 0
    {
        Prompt = szPrompt.as_mut_ptr();
    } else {
        Prompt = core::ptr::addr_of_mut!(DefaultPrompt) as *mut TCHAR;
    }

    /*
     * Special pre-handling for $I: If the information line is displayed
     * on top of the screen, ensure that the prompt won't be hidden below it.
     */
    if HasInfoLine() != 0 && GetCursorY() as i32 == 0 {
        ConOutChar('\n' as TCHAR);
    }

    /* Parse the prompt string */
    pr = Prompt;
    while *pr != 0 {
        if *pr != '$' as TCHAR {
            ConOutChar(*pr);
        } else {
            pr = pr.add(1);
            if *pr == 0 {
                break;
            }
            match _totupper(*pr) as u32 {
                65 => {
                    ConOutChar('&' as TCHAR);
                }
                66 => {
                    ConOutChar('|' as TCHAR);
                }
                67 => {
                    ConOutChar('(' as TCHAR);
                }
                68 => {
                    ConOutPrintf(
                        ['%' as TCHAR, 's' as TCHAR, 0 as TCHAR].as_ptr(),
                        GetDateString(),
                    );
                }
                69 => {
                    ConOutChar('\u{1b}' as TCHAR);
                }
                70 => {
                    ConOutChar(')' as TCHAR);
                }
                71 => {
                    ConOutChar('>' as TCHAR);
                }
                72 => {
                    ConOutPuts(
                        [0x08 as TCHAR, ' ' as TCHAR, 0x08 as TCHAR, 0 as TCHAR].as_ptr(),
                    );
                }
                73 => {
                    PrintInfoLine();
                }
                76 => {
                    ConOutChar('<' as TCHAR);
                }
                78 => {
                    GetCurrentDirectory(_countof(szPath), szPath.as_mut_ptr());
                    ConOutChar(szPath[0 as usize]);
                }
                80 => {
                    GetCurrentDirectory(_countof(szPath), szPath.as_mut_ptr());
                    ConOutPrintf(
                        ['%' as TCHAR, 's' as TCHAR, 0 as TCHAR].as_ptr(),
                        szPath.as_mut_ptr(),
                    );
                }
                81 => {
                    ConOutChar('=' as TCHAR);
                }
                83 => {
                    ConOutChar(' ' as TCHAR);
                }
                84 => {
                    ConOutPrintf(
                        ['%' as TCHAR, 's' as TCHAR, 0 as TCHAR].as_ptr(),
                        GetTimeString(),
                    );
                }
                86 => {
                    PrintOSVersion();
                }
                95 => {
                    ConOutChar('\n' as TCHAR);
                }
                36 => {
                    ConOutChar('$' as TCHAR);
                }
                43 => {
                    let mut i: INT = 0;
                    while i < GetDirectoryStackDepth() {
                        ConOutChar('+' as TCHAR);
                        i += 1;
                    }
                }
                _ => {}
            }
        }
        pr = pr.add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn cmd_prompt(param: LPTSTR) -> INT {
    let mut retval: INT = 0;

    if _tcsncmp(param, _T("/?"), 2) == 0 {
        ConOutResPaging(TRUE, STRING_PROMPT_HELP1);
        ConOutResPaging(FALSE, STRING_PROMPT_HELP2);
        ConOutResPaging(FALSE, STRING_PROMPT_HELP3);
        return 0;
    }

    /*
     * Set the PROMPT environment variable. If 'param' is NULL or is
     * an empty string (the user entered "prompt" only), then remove
     * the environment variable and therefore use the default prompt.
     * Otherwise, use the new prompt.
     */
    if SetEnvironmentVariable(
        _T("PROMPT"),
        (if !param.is_null() && *param.add(0) != _T('\0') {
            param
        } else {
            core::ptr::null_mut()
        }),
    ) == 0
    {
        retval = 1;
    }

    if BatType != CMD_TYPE {
        if retval != 0 {
            nErrorLevel = retval;
        }
    } else {
        nErrorLevel = retval;
    }

    return retval;
}
