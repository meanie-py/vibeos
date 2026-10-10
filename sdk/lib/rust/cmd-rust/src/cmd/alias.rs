use ntdll_sys::nt::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// File-scope preamble of alias.c: no structs, typedefs, #define constants, enums, or static tables. The only top-level items are functions (partstrlwr, PrintAlias, ExpandAlias, CommandAlias), which belong to the function pass.

unsafe extern "C" fn partstrlwr(str: LPTSTR) {
    let mut c: LPTSTR = str;
    while *c != 0 && _istspace(*c) == 0 && *c != _T('=') {
        *c = _totlower(*c);
        c = c.offset(1);
    }
}

unsafe extern "C" fn PrintAlias() {
    let mut Aliases: LPTSTR = core::ptr::null_mut();
    let mut ptr: LPTSTR = core::ptr::null_mut();
    let mut len: DWORD = 0;
    let cmd_exe: [u16; 8] = ['c' as u16, 'm' as u16, 'd' as u16, '.' as u16, 'e' as u16, 'x' as u16, 'e' as u16, 0];
    let fmt_s: [u16; 4] = ['%' as u16, 's' as u16, '\n' as u16, 0];

    len = GetConsoleAliasesLength(_T(cmd_exe.as_ptr()));
    if len == 0 as DWORD {
        return;
    }

    /* allocate memory for an extra \0 char to make parsing easier */
    ptr = cmd_alloc(len + core::mem::size_of::<TCHAR>() as DWORD);
    if ptr.is_null() {
        return;
    }

    Aliases = ptr;

    ZeroMemory(Aliases, len + core::mem::size_of::<TCHAR>() as DWORD);

    if GetConsoleAliases(Aliases, len, _T(cmd_exe.as_ptr())) != 0 as DWORD {
        while *Aliases != '\0' as TCHAR {
            ConOutPrintf(_T(fmt_s.as_ptr()), Aliases);
            Aliases = Aliases.offset(lstrlen(Aliases) as isize);
            Aliases = Aliases.offset(1);
        }
    }
    cmd_free(ptr);
}
#[no_mangle]
pub unsafe extern "C" fn ExpandAlias(cmd: LPTSTR, maxlen: INT) {
    let mut buffer: LPTSTR = core::ptr::null_mut();
    let mut position: LPTSTR = core::ptr::null_mut();
    let mut in_: LPTSTR = core::ptr::null_mut();
    let mut out: LPTSTR = core::ptr::null_mut();
    let mut Token: LPTSTR = core::ptr::null_mut();
    let mut tmp: LPTSTR = core::ptr::null_mut();
    let sep_nl: [TCHAR; 3] = [' ' as i32 as TCHAR, '\n' as i32 as TCHAR, 0 as TCHAR];
    let sep_sp: [TCHAR; 2] = [' ' as i32 as TCHAR, 0 as TCHAR];
    let exe_name: [TCHAR; 8] = [
        'c' as i32 as TCHAR,
        'm' as i32 as TCHAR,
        'd' as i32 as TCHAR,
        '.' as i32 as TCHAR,
        'e' as i32 as TCHAR,
        'x' as i32 as TCHAR,
        'e' as i32 as TCHAR,
        0 as TCHAR,
    ];

    tmp = cmd_dup(cmd);
    if tmp.is_null() {
        return;
    }

    /* first part is the macro name */
    position = tmp.offset(_tcscspn(tmp, sep_nl.as_ptr()) as isize);
    if position == tmp {
        cmd_free(tmp);
        return;
    }
    let fresh0 = position;
    position = position.offset(1);
    *fresh0 = '\0' as i32 as TCHAR;
    position = position.offset(_tcsspn(position, sep_sp.as_ptr()) as isize);

    buffer = cmd_alloc(maxlen as usize);
    if buffer.is_null() {
        cmd_free(tmp);
        return;
    }

    if GetConsoleAlias(
        tmp,
        buffer,
        maxlen as DWORD,
        exe_name.as_ptr(),
    ) == 0
    {
        cmd_free(tmp);
        cmd_free(buffer);
        return;
    }

    in_ = buffer;
    out = cmd;
    while *in_ != 0 {
        if *in_ == '$' as i32 as TCHAR {
            Token = position;
            if *in_.offset(1 as isize) >= '1' as i32 as TCHAR
                && *in_.offset(1 as isize) <= '9' as i32 as TCHAR
            {
                /* Copy a single space-delimited token from the input line */
                let mut num: INT;
                num = (*in_.offset(1 as isize) as INT
                    - '1' as i32) as INT;
                while num > 0 {
                    Token = Token.offset(
                        _tcscspn(Token, sep_nl.as_ptr()) as isize,
                    );
                    Token = Token.offset(
                        _tcsspn(Token, sep_sp.as_ptr()) as isize,
                    );
                    num -= 1;
                }
                while _tcschr(sep_nl.as_ptr(), *Token).is_null() {
                    if out >= cmd.offset((maxlen - 1) as isize) {
                        break;
                    }
                    let fresh1 = Token;
                    Token = Token.offset(1);
                    let fresh2 = out;
                    out = out.offset(1);
                    *fresh2 = *fresh1;
                }
                in_ = in_.offset(2 as isize);
                continue;
            } else if *in_.offset(1 as isize) == '*' as i32 as TCHAR {
                /* Copy the entire remainder of the line */
                while *Token != 0 && *Token != '\n' as i32 as TCHAR {
                    if out >= cmd.offset((maxlen - 1) as isize) {
                        break;
                    }
                    let fresh3 = Token;
                    Token = Token.offset(1);
                    let fresh4 = out;
                    out = out.offset(1);
                    *fresh4 = *fresh3;
                }
                in_ = in_.offset(2 as isize);
                continue;
            }
        }
        if out >= cmd.offset((maxlen - 1) as isize) {
            break;
        }
        let fresh5 = in_;
        in_ = in_.offset(1);
        let fresh6 = out;
        out = out.offset(1);
        *fresh6 = *fresh5;
    }
    let fresh7 = out;
    out = out.offset(1);
    *fresh7 = '\n' as i32 as TCHAR;
    *out = '\0' as i32 as TCHAR;

    cmd_free(buffer);
    cmd_free(tmp);
}

#[no_mangle]
pub unsafe extern "C" fn CommandAlias(param: LPTSTR) -> INT {
    let mut ptr: LPTSTR;

    if _tcsncmp(param, _T("/?"), 2) == 0 {
        ConOutResPaging(TRUE, STRING_ALIAS_HELP);
        return 0;
    }

    nErrorLevel = 0 as INT;

    if *param.offset(0 as isize) == _T('\0') {
        PrintAlias();
        return 0;
    }

    nErrorLevel = 0 as INT;

    /* error if no '=' found */
    ptr = _tcschr(param, _T('=')) as LPTSTR;
    if ptr == (0 as LPTSTR) {
        nErrorLevel = 1 as INT;
        return 1;
    }

    /* Split rest into name and substitute */
    let fresh39 = ptr;
    ptr = ptr.offset(1);
    *fresh39 = _T('\0');

    partstrlwr(param);

    if *ptr.offset(0 as isize) == _T('\0') {
        AddConsoleAlias(param, core::ptr::null_mut(), _T("cmd.exe"));
    } else {
        AddConsoleAlias(param, ptr, _T("cmd.exe"));
    }

    return 0;
}
