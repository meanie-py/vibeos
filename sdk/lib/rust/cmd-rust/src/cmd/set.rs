use crate::cmd::console::ConInString;
use crate::cmd::console::ConOutChar;
use crate::cmd::console::ConOutResPaging;
use crate::cmd::cmd::GetEnvVarOrSpecial;
use crate::cmd::misc::StripQuotes;
use ntdll_sys::nt::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
/// Initial size of environment variable buffer
pub const ENV_BUFFER_SIZE: usize = 1024;

unsafe extern "C" fn skip_ws(mut p: LPCTSTR) -> LPCTSTR {
    while *p as ::core::ffi::c_int != 0
        && *p as ::core::ffi::c_int <= ' ' as i32 as ::core::ffi::c_int
    {
        p = p.offset(1);
    }
    return p;
}

unsafe extern "C" fn GetQuotedString(mut p: LPTSTR) -> LPTSTR {
    let mut end: LPTSTR = core::ptr::null_mut();
    if *p == _T('"') {
        p = skip_ws(p.add(1 as usize)) as LPTSTR;
        /* If a matching quote is found, truncate the string */
        end = _tcsrchr(p, _T('"')) as LPTSTR;
        if !end.is_null() {
            *end = _T('\0');
        }
    }
    return p;
}

#[no_mangle]
pub unsafe extern "C" fn cmd_set(mut param: LPTSTR) -> INT {
    let mut retval: INT = 0;
    let mut p: LPTSTR;
    let mut lpEnv: LPTSTR;
    let mut lpOutput: LPTSTR;

    'quit: {
        if _tcsncmp(param, _T("/?"), 2) == 0 {
            ConOutResPaging(TRUE, STRING_SET_HELP);
            return 0;
        }

        param = skip_ws(param) as LPTSTR;

        /* If no parameters, show the environment */
        if *param.add(0) == _T('\0') {
            lpEnv = GetEnvironmentStrings() as LPTSTR;
            if !lpEnv.is_null() {
                lpOutput = lpEnv;
                while *lpOutput != 0 {
                    /* Do not display the special '=X:' environment variables */
                    if *lpOutput != _T('=') {
                        ConOutPuts(lpOutput);
                        ConOutChar(_T('\n'));
                    }
                    lpOutput = lpOutput.add(_tcslen(lpOutput) + 1);
                }
                FreeEnvironmentStrings(lpEnv);
            }

            retval = 0;
            break 'quit;
        }

        /* The /A does *NOT* have to be followed by a whitespace */
        if _tcsnicmp(param, _T("/A"), 2) == 0 {
            let mut Success: BOOL;

            /* Save error level since seta_eval() modifies it, as
             * we need to set it later according to specific rules. */
            let mut nOldErrorLevel: INT = nErrorLevel;

            StripQuotes(param);
            Success = seta_eval(skip_ws(param.add(2)));
            if Success == 0 {
                retval = nErrorLevel;
                nErrorLevel = nOldErrorLevel;
            } else {
                retval = 0;
            }
            break 'quit;
        }

        if _tcsnicmp(param, _T("/P"), 2) == 0 {
            let mut value: [TCHAR; 1023] = [0; 1023];
            param = GetQuotedString(
                skip_ws(param.add(2)) as LPTSTR,
            );
            p = _tcschr(param, _T('='));
            if p.is_null() {
                ConErrResPuts(STRING_SYNTAX_COMMAND_INCORRECT);
                retval = 1;
                break 'quit;
            }

            let fresh0 = p;
            p = p.add(1);
            *fresh0 = _T('\0');
            ConOutPrintf(_T("%s"), GetQuotedString(p));
            ConInString(value.as_mut_ptr(), ARRAYSIZE(value));

            if *value.as_ptr() == 0 || SetEnvironmentVariable(param, value.as_ptr()) == 0 {
                retval = 1;
                break 'quit;
            }
            retval = 0;
            break 'quit;
        }

        param = GetQuotedString(param) as LPTSTR;

        p = _tcschr(param, _T('='));
        if !p.is_null() {
            /* Set or remove the environment variable */
            if p == param {
                /* Handle set =val case */
                ConErrResPuts(STRING_SYNTAX_COMMAND_INCORRECT);
                retval = 1;
                break 'quit;
            }

            let fresh1 = p;
            p = p.add(1);
            *fresh1 = _T('\0');

            /* Check for dynamic TRACE ON/OFF */
            if _tcsicmp(param, _T("CMDTRACE")) == 0 {
                g_bDynamicTrace = (_tcsicmp(p, _T("ON")) == 0) as BOOL;
            }

            if SetEnvironmentVariable(
                param,
                (if *p != 0 { p } else { core::ptr::null_mut() }) as LPCTSTR,
            ) == 0
            {
                retval = 1;
                break 'quit;
            }
        } else {
            /* Display all the environment variables with the given prefix */
            let mut pOrgParam: LPTSTR = param;
            let mut bFound: BOOLEAN = FALSE;
            let mut bRestoreSpace: BOOLEAN;

            /*
             * Trim the prefix from "special" characters (only when displaying the
             * environment variables), so that e.g. "SET ,; ,;FOO" will display all
             * the variables starting by "FOO".
             * The SET command allows as well to set an environment variable whose name
             * actually contains these characters (e.g. "SET ,; ,;FOO=42"); however,
             * by trimming the characters, doing "SET ,; ,;FOO" would not allow seeing
             * such variables.
             * Thus, we also save a pointer to the original variable name prefix, that
             * we will look it up as well below.
             */
            while _istspace(*param) != 0 || *param == _T(',') || *param == _T(';') {
                param = param.add(1);
            }

            /* Just remove the very last space, if present */
            p = _tcsrchr(param, _T(' '));
            bRestoreSpace = (!p.is_null()) as BOOLEAN;
            if p.is_null() {
                p = param.add(_tcslen(param));
            }
            *p = _T('\0');

            lpEnv = GetEnvironmentStrings() as LPTSTR;
            if !lpEnv.is_null() {
                lpOutput = lpEnv;
                while *lpOutput != 0 {
                    /* Look up for both the original and truncated variable name prefix */
                    if _tcsnicmp(lpOutput, pOrgParam, p.offset_from(pOrgParam) as usize) == 0
                        || _tcsnicmp(lpOutput, param, p.offset_from(param) as usize) == 0
                    {
                        ConOutPuts(lpOutput);
                        ConOutChar(_T('\n'));
                        bFound = TRUE;
                    }
                    lpOutput = lpOutput.add(_tcslen(lpOutput) + 1);
                }
                FreeEnvironmentStrings(lpEnv);
            }

            /* Restore the truncated space for correctly
             * displaying the error message, if any. */
            if bRestoreSpace != 0 {
                *p = _T(' ');
            }

            if bFound == 0 {
                ConErrResPrintf(STRING_SET_ENV_ERROR, param);
                retval = 1;
                break 'quit;
            }
        }
    }

    // Quit:
    if BatType != CMD_TYPE {
        if retval != 0 {
            nErrorLevel = retval;
        }
    } else {
        nErrorLevel = retval;
    }

    return retval;
}

unsafe extern "C" fn ident_len(p: LPCTSTR) -> INT {
    let mut p2: LPCTSTR = p;
    if __iscsymf(*p as ::core::ffi::c_int) != 0 {
        p2 = p2.offset(1);
        while __iscsym(*p2 as ::core::ffi::c_int) != 0 {
            p2 = p2.offset(1);
        }
    }
    return p2.offset_from(p) as INT;
}

unsafe extern "C" fn seta_identval(ident: LPCTSTR) -> INT {
    let identVal: LPCTSTR = GetEnvVarOrSpecial(ident);
    if identVal.is_null() {
        return 0;
    } else {
        return _tcstol(identVal, core::ptr::null_mut(), 0) as INT;
    }
}

unsafe extern "C" fn calc(lval: *mut INT, op: TCHAR, rval: INT) -> BOOL {
    match op as i32 {
        42 => {
            *lval *= rval;
        }
        47 => {
            if rval == 0 {
                ConErrResPuts(STRING_ERROR_DIVISION_BY_ZERO);
                nErrorLevel = 0x400023D1; // 1073750993;
                return FALSE;
            }
            *lval /= rval;
        }
        37 => {
            if rval == 0 {
                ConErrResPuts(STRING_ERROR_DIVISION_BY_ZERO);
                nErrorLevel = 0x400023D1; // 1073750993;
                return FALSE;
            }
            *lval %= rval;
        }
        43 => {
            *lval += rval;
        }
        45 => {
            *lval -= rval;
        }
        38 => {
            *lval &= rval;
        }
        94 => {
            *lval ^= rval;
        }
        124 => {
            *lval |= rval;
        }
        _ => {
            ConErrResPuts(STRING_INVALID_OPERAND);
            nErrorLevel = 0x400023CE; // 1073750990;
            return FALSE;
        }
    }
    return TRUE;
}

unsafe extern "C" fn seta_unaryTerm(p_: *mut LPCTSTR, result: *mut INT) -> BOOL {
    let mut alloca_allocations: Vec<Vec<u8>> = Vec::new();
    let mut p: LPCTSTR = *p_;
    let mut rval: INT = 0;

    if *p == _T('(') {
        p = skip_ws(p.offset(1 as isize));
        if seta_stmt(&raw mut p, &raw mut rval) == 0 {
            return FALSE;
        }
        let fresh33 = p;
        p = p.offset(1);
        if *fresh33 != _T(')') {
            ConErrResPuts(STRING_EXPECTED_CLOSE_PAREN);
            nErrorLevel = 0x400023CC; // 1073750988;
            return FALSE;
        }
        *result = rval;
    } else if _istdigit(*p) != 0 {
        *_errno() = 0;
        rval = _tcstol(
            p,
            &raw mut p as *mut LPTSTR,
            0,
        );

        /* Check for overflow / underflow */
        if *_errno() == ERANGE {
            ConErrResPuts(STRING_ERROR_INVALID_NUMBER2);
            nErrorLevel = 0x400023D0; // 1073750992;
            return FALSE;
        }
        /*
         * _tcstol() stopped at the first non-digit character. If it's not a whitespace,
         * or if it's the start of a possible identifier, this means the number being
         * interpreted was invalid.
         */
        else if *p != 0 && _istspace(*p) == 0 && __iscsymf(*p) != 0 {
            ConErrResPuts(STRING_ERROR_INVALID_NUMBER1);
            nErrorLevel = 0x400023CF; // 1073750991;
            return FALSE;
        }
        *result = rval;
    } else if __iscsymf(*p) != 0 {
        let mut ident: LPTSTR = core::ptr::null_mut();
        let mut identlen: INT = 0;
        // PARSE_IDENT(ident, identlen, p) expanded inline:
        identlen = ident_len(p);
        alloca_allocations.push(vec![
            0u8;
            ((identlen + 1) as usize).wrapping_mul(core::mem::size_of::<TCHAR>())
        ]);
        ident = alloca_allocations.last_mut().unwrap().as_mut_ptr() as LPTSTR;
        core::ptr::copy(p, ident, identlen as usize);
        *ident.offset(identlen as isize) = 0;
        p = p.offset(identlen as isize);
        *result = seta_identval(ident);
    } else {
        ConErrResPuts(STRING_EXPECTED_NUMBER_OR_VARIABLE);
        nErrorLevel = 0x400023CD; // 1073750989;
        return FALSE;
    }
    *p_ = skip_ws(p);
    return TRUE;
}

unsafe extern "C" fn seta_mulTerm(p_: *mut LPCTSTR, result: *mut INT) -> BOOL {
    let mut p: LPCTSTR = *p_;
    let mut op: TCHAR = 0;
    let mut rval: INT = 0;

    if !_tcschr(_T("!~-+"), *p).is_null() {
        op = *p;
        p = skip_ws(p.add(1 as usize));

        if seta_mulTerm(core::ptr::addr_of_mut!(p), core::ptr::addr_of_mut!(rval)) == 0 {
            return FALSE;
        }

        match op as INT {
            x if x == '!' as INT => {
                rval = (rval == 0) as INT;
            }
            x if x == '~' as INT => {
                rval = !rval;
            }
            x if x == '-' as INT => {
                rval = -rval;
            }
            _ => {}
        }
    } else {
        if seta_unaryTerm(core::ptr::addr_of_mut!(p), core::ptr::addr_of_mut!(rval)) == 0 {
            return FALSE;
        }
    }

    *result = rval;
    *p_ = p;
    return TRUE;
}

unsafe extern "C" fn seta_ltorTerm(
    p_: *mut LPCTSTR,
    result: *mut INT,
    ops: LPCTSTR,
    subTerm: Option<unsafe extern "C" fn(*mut LPCTSTR, *mut INT) -> BOOL>,
) -> BOOL {
    let mut p: LPCTSTR = *p_;
    let mut lval: INT = 0;

    /* Evaluate the left-hand side */
    if subTerm(core::ptr::addr_of_mut!(p), core::ptr::addr_of_mut!(lval)) == 0 {
        return FALSE;
    }

    while *p != 0 && !_tcschr(ops as *const wchar_t, *p).is_null() {
        let mut rval: INT = 0;
        let mut op: TCHAR = *p;

        p = skip_ws(p.offset(1 as isize));

        /* Evaluate the immediate right-hand side */
        if subTerm(core::ptr::addr_of_mut!(p), core::ptr::addr_of_mut!(rval)) == 0 {
            return FALSE;
        }

        /* This becomes the new left-hand side for the next iteration */
        if calc(core::ptr::addr_of_mut!(lval), op, rval) == 0 {
            return FALSE;
        }
    }

    *result = lval;
    *p_ = p;
    return TRUE;
}

unsafe extern "C" fn seta_addTerm(p_: *mut LPCTSTR, result: *mut INT) -> BOOL {
    static OPS: [u16; 4] = ['*' as u16, '/' as u16, '%' as u16, 0];
    return seta_ltorTerm(
        p_,
        result,
        OPS.as_ptr() as LPCTSTR,
        seta_mulTerm as unsafe extern "C" fn(*mut LPCTSTR, *mut INT) -> BOOL,
    );
}

unsafe extern "C" fn seta_logShiftTerm(mut p_: *mut LPCTSTR, mut result: *mut INT) -> BOOL {
    static OPS: [u16; 3] = ['+' as u16, '-' as u16, 0];
    return seta_ltorTerm(
        p_,
        result,
        core::ptr::addr_of!(OPS) as *const u16 as LPCTSTR,
        Some(seta_addTerm as unsafe extern "C" fn(*mut LPCTSTR, *mut INT) -> BOOL),
    );
}

unsafe extern "C" fn seta_bitAndTerm(p_: *mut LPCTSTR, result: *mut INT) -> BOOL {
    let mut p: LPCTSTR = *p_;
    let mut lval: INT = 0;

    /* Evaluate the left-hand side */
    if seta_logShiftTerm(core::ptr::addr_of_mut!(p), core::ptr::addr_of_mut!(lval)) == 0 {
        return FALSE;
    }

    /* Handle << >> operators */
    while *p != 0 && !_tcschr(_T("<>"), *p).is_null() {
        let mut rval: INT = 0;
        let mut op: TCHAR = *p;

        /* Check whether the next non-whitespace character is the same operator */
        p = skip_ws(p.offset(1 as isize));
        if *p != op {
            break;
        }

        /* Skip it */
        p = skip_ws(p.offset(1 as isize));

        /* Evaluate the immediate right-hand side */
        if seta_logShiftTerm(core::ptr::addr_of_mut!(p), core::ptr::addr_of_mut!(rval)) == 0 {
            return FALSE;
        }

        /* This becomes the new left-hand side for the next iteration */
        match op as u32 {
            60 => {
                /* Shift left has to be a positive number, 0-31 otherwise 0 is returned,
                 * which differs from the compiler (for example gcc) so being explicit. */
                if rval < 0
                    || (rval as usize) >= (8 as usize) * core::mem::size_of::<INT>()
                {
                    lval = 0;
                } else {
                    lval <<= rval;
                }
            }
            62 => {
                lval >>= rval;
            }
            _ => {
                ConErrResPuts(STRING_INVALID_OPERAND);
                nErrorLevel = 0x400023CE; // 1073750990;
                return FALSE;
            }
        }
    }

    *result = lval;
    *p_ = p;
    return TRUE;
}

unsafe extern "C" fn seta_bitExclOrTerm(mut p_: *mut LPCTSTR, mut result: *mut INT) -> BOOL {
    return seta_ltorTerm(
        p_,
        result,
        _T("&"),
        Some(seta_bitAndTerm as unsafe extern "C" fn(*mut LPCTSTR, *mut INT) -> BOOL),
    );
}

unsafe extern "C" fn seta_bitOrTerm(p_: *mut LPCTSTR, result: *mut INT) -> BOOL {
    return seta_ltorTerm(p_, result, _T("^"), seta_bitExclOrTerm);
}

unsafe extern "C" fn seta_expr(p_: *mut LPCTSTR, result: *mut INT) -> BOOL {
    let ops: [u16; 2] = ['|' as u16, 0];
    return seta_ltorTerm(p_, result, ops.as_ptr() as LPCTSTR, Some(seta_bitOrTerm));
}

unsafe extern "C" fn seta_assignment(p_: *mut LPCTSTR, result: *mut INT) -> BOOL {
    let mut p: LPCTSTR = *p_;
    let mut ident: LPTSTR = core::ptr::null_mut();
    let mut op: TCHAR = 0;
    let mut identlen: INT = 0;
    let mut exprval: INT = 0;

    // PARSE_IDENT(ident, identlen, p) expanded inline
    identlen = ident_len(p);
    ident = _alloca(((identlen + 1) as usize) * core::mem::size_of::<TCHAR>()) as LPTSTR;
    memmove(
        ident as *mut core::ffi::c_void,
        p as *const core::ffi::c_void,
        (identlen as usize) * core::mem::size_of::<TCHAR>(),
    );
    *ident.offset(identlen as isize) = 0;
    p = p.offset(identlen as isize);

    'evaluate: {
        if identlen != 0 {
            p = skip_ws(p);

            /* Handle = assignment */
            if *p == _T('=') {
                op = *p;
                p = skip_ws(p.offset(1 as isize));
            }
            /* Handle *= /= %= += -= &= ^= |= assignments */
            else if !_tcschr(_T("*/%+-&^|"), *p).is_null() {
                op = *p;

                /* Find the '=', there may be some spaces before it */
                p = skip_ws(p.offset(1 as isize));
                if *p != _T('=') {
                    op = 0;
                    break 'evaluate;
                }

                /* Skip it */
                p = skip_ws(p.offset(1 as isize));
            }
            /* Handle <<= >>= assignments */
            else if !_tcschr(_T("<>"), *p).is_null() {
                op = *p;

                /* Check whether the next non-whitespace character is the same operator */
                p = skip_ws(p.offset(1 as isize));
                if *p != op {
                    op = 0;
                    break 'evaluate;
                }

                /* Find the '=', there may be some spaces before it */
                p = skip_ws(p.offset(1 as isize));
                if *p != _T('=') {
                    op = 0;
                    break 'evaluate;
                }

                /* Skip it */
                p = skip_ws(p.offset(1 as isize));
            }
        }
    }

    /* Allow to chain multiple assignments, such as: a=b=1 */
    if !ident.is_null() && op != 0 {
        let mut identval: INT = 0;
        let mut buf: LPTSTR = core::ptr::null_mut();

        if seta_assignment(core::ptr::addr_of_mut!(p), core::ptr::addr_of_mut!(exprval)) == 0 {
            return FALSE;
        }

        identval = seta_identval(ident as LPCTSTR);

        match op as i32 {
            /* Handle = assignment */
            61 => {
                identval = exprval;
            }
            /* Handle <<= assignment */
            60 => {
                /* Shift left has to be a positive number, 0-31 otherwise 0 is returned,
                 * which differs from the compiler (for example gcc) so being explicit. */
                if exprval < 0
                    || (exprval as usize)
                        >= (8 as usize) * core::mem::size_of::<INT>()
                {
                    identval = 0;
                } else {
                    identval <<= exprval;
                }
            }
            /* Handle >>= assignment */
            62 => {
                identval >>= exprval;
            }
            /* Other assignments */
            _ => {
                if calc(core::ptr::addr_of_mut!(identval), op, exprval) == 0 {
                    return FALSE;
                }
            }
        }

        buf = _alloca(32 * core::mem::size_of::<TCHAR>()) as LPTSTR;
        _sntprintf(buf, 32 as usize, _T("%i"), identval);
        SetEnvironmentVariable(ident as LPCTSTR, buf as LPCTSTR); // TODO FIXME - check return value
        exprval = identval;
    } else {
        /* Restore p in case we found an identifier but not an operator */
        p = *p_;
        if seta_expr(core::ptr::addr_of_mut!(p), core::ptr::addr_of_mut!(exprval)) == 0 {
            return FALSE;
        }
    }

    *result = exprval;
    *p_ = p;
    return TRUE;
}

unsafe extern "C" fn seta_stmt(p_: *mut LPCTSTR, result: *mut INT) -> BOOL {
    let mut p: LPCTSTR = *p_;
    let mut rval: INT = 0;
    if seta_assignment(core::ptr::addr_of_mut!(p), core::ptr::addr_of_mut!(rval)) == 0 {
        return FALSE;
    }
    /* Loop over each statement */
    while *p == _T(',') {
        p = skip_ws(p.add(1));

        if seta_assignment(core::ptr::addr_of_mut!(p), core::ptr::addr_of_mut!(rval)) == 0 {
            return FALSE;
        }
    }

    *result = rval;
    *p_ = p;
    return TRUE;
}

unsafe extern "C" fn seta_eval(mut p: LPCTSTR) -> BOOL {
    let mut rval: INT = 0;

    if *p == 0 {
        ConErrResPuts(STRING_SYNTAX_COMMAND_INCORRECT);
        nErrorLevel = 1;
        return FALSE;
    }
    if seta_stmt(core::ptr::addr_of_mut!(p), core::ptr::addr_of_mut!(rval)) == 0 {
        return FALSE;
    }

    /* If unparsed data remains, fail and bail out */
    if *p != 0 {
        ConErrResPuts(STRING_SYNTAX_COMMAND_INCORRECT); // Actually syntax error / missing operand.
        nErrorLevel = 0x400023CE; // 1073750990;
        return FALSE;
    }

    /* Echo the result of the evaluation only in interactive (non-batch) mode */
    if bc.is_null() {
        ConOutPrintf(_T("%i"), rval);
    }

    return TRUE;
}
