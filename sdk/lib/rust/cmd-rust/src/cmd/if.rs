use crate::cmd::console::ConOutResPaging;
use crate::cmd::cmd::DoDelayedExpansion;
use crate::cmd::cmd::ExecuteCommand;
use crate::cmd::cmd::GetEnvVarOrSpecial;
use crate::cmd::error::error_syntax;
use crate::prelude::*;
use core::ffi::c_void;

unsafe extern "system" fn GenericCmp(
    StringCmp: unsafe extern "system" fn(LPCTSTR, LPCTSTR) -> INT,
    Left: LPCTSTR,
    Right: LPCTSTR,
) -> INT {
    let mut end: LPTSTR = core::ptr::null_mut();
    let mut nLeft: INT = _tcstol(Left, core::ptr::addr_of_mut!(end), 0) as INT;
    if *end == ('\0' as u16) {
        let mut nRight: INT = _tcstol(Right, core::ptr::addr_of_mut!(end), 0) as INT;
        if *end == ('\0' as u16) {
            /* both arguments are numeric */
            return if nLeft < nRight {
                -1
            } else {
                (nLeft > nRight) as INT
            };
        }
    }
    return StringCmp(Left, Right);
}
#[no_mangle]
pub unsafe extern "C" fn cmd_if(param: *mut TCHAR) -> ::core::ffi::c_int {
    if _tcsncmp(param, (&[0x002f, 0x003f, 0x0000u16]).as_ptr() /* _T "/?" */, 2) == 0 {
        ConOutResPaging((TRUE) as BOOL, STRING_IF_HELP1);
        return 0;
    }

    ParseErrorEx(param);
    return 1;
}
#[no_mangle]
pub unsafe extern "C" fn ExecuteIf(Cmd: *mut PARSED_COMMAND) -> INT {
    let mut result: INT = (FALSE) as i32; /* when set cause 'then' clause to be executed */
    let mut param: LPTSTR = core::ptr::null_mut();
    let mut Left: LPTSTR = core::ptr::null_mut();
    let mut Right: LPTSTR = core::ptr::null_mut();

    if !(*Cmd).__bindgen_anon_1.If.LeftArg.is_null() {
        Left = DoDelayedExpansion((*Cmd).__bindgen_anon_1.If.LeftArg);
        if Left.is_null() {
            return 1;
        }
    }
    Right = DoDelayedExpansion((*Cmd).__bindgen_anon_1.If.RightArg);
    if Right.is_null() {
        if !Left.is_null() {
            cmd_free!(Left);
        }
        return 1;
    }

    'fail: {
        if bEnableExtensions != 0 && ((*Cmd).__bindgen_anon_1.If.Operator == IF_CMDEXTVERSION) {
            /* IF CMDEXTVERSION n: check if Command Extensions
             * version is greater or equal to n. */
            let n: DWORD = _tcstoul(
                Right,
                core::ptr::addr_of_mut!(param),
                10,
            );
            if *param != ('\0' as u16) {
                error_syntax(Right);
                break 'fail;
            }
            result = (CMDEXTVERSION >= n) as INT;
        } else if bEnableExtensions != 0 && ((*Cmd).__bindgen_anon_1.If.Operator == IF_DEFINED) {
            /* IF DEFINED var: check if environment variable exists */
            result = (GetEnvVarOrSpecial(Right) != core::ptr::null_mut()) as INT;
        } else if (*Cmd).__bindgen_anon_1.If.Operator == IF_ERRORLEVEL {
            /* IF ERRORLEVEL n: check if last exit code is greater or equal to n */
            let n: INT = _tcstol(
                Right,
                core::ptr::addr_of_mut!(param),
                10,
            );
            if *param != ('\0' as u16) {
                error_syntax(Right);
                break 'fail;
            }
            result = (nErrorLevel >= n) as INT;
        } else if (*Cmd).__bindgen_anon_1.If.Operator == IF_EXIST {
            let mut IsDir: BOOL;
            let mut Size: SIZE_T;
            let mut f: WIN32_FIND_DATA = core::mem::zeroed();
            let mut hFind: HANDLE;
            let mut attrs: DWORD;

            /* IF EXIST filename: check if file exists (wildcards allowed) */
            StripQuotes(Right);

            Size = (_tcslen(Right)) as _;
            IsDir = (*Right.add((Size - 1) as usize) == '\\' as TCHAR) as BOOL;
            if IsDir != 0 {
                *Right.add((Size - 1) as usize) = 0 as TCHAR;
            }

            hFind = FindFirstFile(Right, core::ptr::addr_of_mut!(f));
            if hFind != INVALID_HANDLE_VALUE {
                attrs = f.dwFileAttributes;
                FindClose(hFind);
            } else {
                /* FindFirstFile fails at the root directory. */
                attrs = GetFileAttributes(Right);
            }

            if attrs == INVALID_FILE_ATTRIBUTES {
                result = (FALSE) as i32;
            } else if IsDir != 0 {
                result = ((attrs & FILE_ATTRIBUTE_DIRECTORY) == FILE_ATTRIBUTE_DIRECTORY) as INT;
            } else {
                result = (TRUE) as i32;
            }

            if IsDir != 0 {
                *Right.add((Size - 1) as usize) = '\\' as TCHAR;
            }
        } else {
            /*
             * Do case-insensitive string comparisons if /I specified.
             *
             * Since both strings are user-specific, use kernel32!lstrcmp(i)
             * instead of CRT!_tcs(i)cmp, so as to use the correct
             * current thread locale information.
             */
            let mut StringCmp: unsafe extern "system" fn(
                LPCTSTR,
                LPCTSTR,
            ) -> INT = if ((*Cmd).__bindgen_anon_1.If.Flags & (IFFLAG_IGNORECASE as u8)) != 0 {
                lstrcmpi as _
            } else {
                lstrcmp as _
            };

            if (*Cmd).__bindgen_anon_1.If.Operator == IF_STRINGEQ {
                /* IF str1 == str2 */
                result = (StringCmp(Left, Right) == 0) as INT;
            } else {
                let mut at_unknown_op: bool = false;
                if bEnableExtensions != 0 {
                    result = GenericCmp(StringCmp, Left, Right);
                    match (*Cmd).__bindgen_anon_1.If.Operator {
                        IF_EQU => {
                            result = (result == 0) as INT;
                        }
                        IF_NEQ => {
                            result = (result != 0) as INT;
                        }
                        IF_LSS => {
                            result = (result < 0) as INT;
                        }
                        IF_LEQ => {
                            result = (result <= 0) as INT;
                        }
                        IF_GTR => {
                            result = (result > 0) as INT;
                        }
                        IF_GEQ => {
                            result = (result >= 0) as INT;
                        }
                        _ => {
                            at_unknown_op = true;
                        }
                    }
                } else {
                    at_unknown_op = true;
                }
                if at_unknown_op {
                    /* unknownOp: */
                    ERR!(
                        b"Unknown IF operator 0x%x\n\0" as *const u8 as LPCSTR,
                        (*Cmd).__bindgen_anon_1.If.Operator,
                    );
                    break 'fail;
                }
            }
        }

        if !Left.is_null() {
            cmd_free!(Left);
        }
        cmd_free!(Right);

        if result
            ^ ((((*Cmd).__bindgen_anon_1.If.Flags & (IFFLAG_NEGATE as u8)) != 0) as INT)
            != 0
        {
            /* Full condition was true, do the command */
            return ExecuteCommand((*Cmd).Subcommands);
        } else {
            /* Full condition was false, do the "else" command if there is one */
            return ExecuteCommand((*(*Cmd).Subcommands).Next);
        }
    }

    /* fail: */
    if !Left.is_null() {
        cmd_free!(Left);
    }
    cmd_free!(Right);
    return 1;
}
