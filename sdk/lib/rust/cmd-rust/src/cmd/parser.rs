use crate::cmd::console::ConOutChar;
use crate::cmd::redir::FreeRedirection;
use crate::cmd::cmd::ReadLine;
use crate::cmd::misc::StripQuotes;
use crate::cmd::cmd::SubstituteForVars;
use crate::cmd::cmd::SubstituteVars;
use crate::cmd::error::error_syntax;
use crate::prelude::*;
use core::ffi::c_void;

extern "C" {
    fn _snwprintf(buf: *mut TCHAR, count: usize, fmt: *const TCHAR, ...) -> i32;
    fn wcslen(s: *const TCHAR) -> usize;
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static RUSTCONV_C2R_WSTR_43054F1DC5A5: [u16; 4] = [0x0065u16, 0x0071, 0x0075, 0x0000];
pub static RUSTCONV_C2R_WSTR_5777410BDD22: [u16; 4] = [0x006Eu16, 0x0065, 0x0071, 0x0000];
pub static RUSTCONV_C2R_WSTR_5780FDC2DDEC: [u16; 6] = [0x0065u16, 0x0078, 0x0069, 0x0073, 0x0074, 0x0000];
pub static RUSTCONV_C2R_WSTR_6AF7BDD845B6: [u16; 4] = [0x0067u16, 0x0074, 0x0072, 0x0000];
pub static RUSTCONV_C2R_WSTR_888F2CC94A1A: [u16; 4] = [0x006Cu16, 0x0065, 0x0071, 0x0000];
pub static RUSTCONV_C2R_WSTR_C966DC48D2F4: [u16; 4] = [0x0067u16, 0x0065, 0x0071, 0x0000];
pub static RUSTCONV_C2R_WSTR_D2B12FD6E127: [u16; 4] = [0x006Cu16, 0x0073, 0x0073, 0x0000];
pub static RUSTCONV_C2R_WSTR_DD9F2FC392A6: [u16; 8] = [0x0064u16, 0x0065, 0x0066, 0x0069, 0x006E, 0x0065, 0x0064, 0x0000];
pub static RUSTCONV_C2R_WSTR_E3126470BC03: [u16; 11] = [
    0x0065u16, 0x0072, 0x0072, 0x006F, 0x0072, 0x006C, 0x0065, 0x0076, 0x0065, 0x006C,
    0x0000,
];
pub static RUSTCONV_C2R_WSTR_E5B0B8CDE0AD: [u16; 14] = [
    0x0063u16, 0x006D, 0x0064, 0x0065, 0x0078, 0x0074, 0x0076, 0x0065, 0x0072, 0x0073,
    0x0069, 0x006F, 0x006E, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_F3A89E2DDBE3: [u16; 3] = [0x003Du16, 0x003D, 0x0000];

#[no_mangle]
pub static mut fDumpTokens: BOOLEAN = FALSE as BOOLEAN;

#[no_mangle]
pub static mut fDumpParse: BOOLEAN = FALSE as BOOLEAN;

static mut OpString: [[TCHAR; 3]; 4] = unsafe {
    [
        ::core::mem::transmute::<[u8; 6], [TCHAR; 3]>(*b"&\0\0\0\0\0"),
        ::core::mem::transmute::<[u8; 6], [TCHAR; 3]>(*b"|\0|\0\0\0"),
        ::core::mem::transmute::<[u8; 6], [TCHAR; 3]>(*b"&\0&\0\0\0"),
        ::core::mem::transmute::<[u8; 6], [TCHAR; 3]>(*b"|\0\0\0\0\0"),
    ]
};

static mut RedirString: [[TCHAR; 3]; 3] = unsafe {
    [
        ::core::mem::transmute::<[u8; 6], [TCHAR; 3]>(*b"<\0\0\0\0\0"),
        ::core::mem::transmute::<[u8; 6], [TCHAR; 3]>(*b">\0\0\0\0\0"),
        ::core::mem::transmute::<[u8; 6], [TCHAR; 3]>(*b">\0>\0\0\0"),
    ]
};

static mut IfOperatorString: [*const TCHAR; 11] = unsafe {
    [
        RUSTCONV_C2R_WSTR_E3126470BC03.as_ptr(),
        RUSTCONV_C2R_WSTR_5780FDC2DDEC.as_ptr(),
        RUSTCONV_C2R_WSTR_E5B0B8CDE0AD.as_ptr(),
        RUSTCONV_C2R_WSTR_DD9F2FC392A6.as_ptr(),
        RUSTCONV_C2R_WSTR_F3A89E2DDBE3.as_ptr(),
        RUSTCONV_C2R_WSTR_43054F1DC5A5.as_ptr(),
        RUSTCONV_C2R_WSTR_5777410BDD22.as_ptr(),
        RUSTCONV_C2R_WSTR_D2B12FD6E127.as_ptr(),
        RUSTCONV_C2R_WSTR_888F2CC94A1A.as_ptr(),
        RUSTCONV_C2R_WSTR_6AF7BDD845B6.as_ptr(),
        RUSTCONV_C2R_WSTR_C966DC48D2F4.as_ptr(),
    ]
};

static mut TempBuf: [TCHAR; 8192] = [0; 8192];

#[no_mangle]
pub static mut bParseError: BOOL = 0;

static mut bLineContinuations: BOOL = 0;

#[no_mangle]
pub static mut ParseLine: [TCHAR; 8192] = [0; 8192];

static mut ParsePos: PTCHAR = ::core::ptr::null::<wchar_t>() as *mut wchar_t;

static mut OldParsePos: PTCHAR = ::core::ptr::null::<wchar_t>() as *mut wchar_t;

#[no_mangle]
pub static mut bIgnoreParserComments: BOOL = (TRUE) as i32;

#[no_mangle]
pub static mut bHandleContinuations: BOOL = (TRUE) as i32;

static mut CurrentToken: [TCHAR; 8192] = [0; 8192];

static mut CurrentTokenType: TOK_TYPE = TOK_END;

static mut InsideBlock: INT = 0 as INT;

pub const C_OP_LOWEST: i32 = (C_MULTI) as i32;
pub const C_OP_HIGHEST: i32 = (C_PIPE) as i32;

pub static IfOperatorString_0: [TCHAR; 11] = [
    b'e' as TCHAR, b'r' as TCHAR, b'r' as TCHAR, b'o' as TCHAR, b'r' as TCHAR,
    b'l' as TCHAR, b'e' as TCHAR, b'v' as TCHAR, b'e' as TCHAR, b'l' as TCHAR, 0,
];
pub static IfOperatorString_1: [TCHAR; 6] = [
    b'e' as TCHAR, b'x' as TCHAR, b'i' as TCHAR, b's' as TCHAR, b't' as TCHAR, 0,
];
pub static IfOperatorString_2: [TCHAR; 14] = [
    b'c' as TCHAR, b'm' as TCHAR, b'd' as TCHAR, b'e' as TCHAR, b'x' as TCHAR,
    b't' as TCHAR, b'v' as TCHAR, b'e' as TCHAR, b'r' as TCHAR, b's' as TCHAR,
    b'i' as TCHAR, b'o' as TCHAR, b'n' as TCHAR, 0,
];
pub static IfOperatorString_3: [TCHAR; 8] = [
    b'd' as TCHAR, b'e' as TCHAR, b'f' as TCHAR, b'i' as TCHAR, b'n' as TCHAR,
    b'e' as TCHAR, b'd' as TCHAR, 0,
];
pub static IfOperatorString_4: [TCHAR; 3] = [b'=' as TCHAR, b'=' as TCHAR, 0];
pub static IfOperatorString_5: [TCHAR; 4] = [b'e' as TCHAR, b'q' as TCHAR, b'u' as TCHAR, 0];
pub static IfOperatorString_6: [TCHAR; 4] = [b'n' as TCHAR, b'e' as TCHAR, b'q' as TCHAR, 0];
pub static IfOperatorString_7: [TCHAR; 4] = [b'l' as TCHAR, b's' as TCHAR, b's' as TCHAR, 0];
pub static IfOperatorString_8: [TCHAR; 4] = [b'l' as TCHAR, b'e' as TCHAR, b'q' as TCHAR, 0];
pub static IfOperatorString_9: [TCHAR; 4] = [b'g' as TCHAR, b't' as TCHAR, b'r' as TCHAR, 0];
pub static IfOperatorString_10: [TCHAR; 4] = [b'g' as TCHAR, b'e' as TCHAR, b'q' as TCHAR, 0];

pub const IF_MAX_UNARY: i32 = (IF_DEFINED) as i32;
pub const IF_MAX_COMPARISON: i32 = (IF_GEQ) as i32;

pub type TOK_TYPE = i32;
pub const TOK_END: i32 = 0;
pub const TOK_NORMAL: i32 = 1;
pub const TOK_OPERATOR: i32 = 2;
pub const TOK_REDIRECTION: i32 = 3;
pub const TOK_BEGIN_BLOCK: i32 = 4;
pub const TOK_END_BLOCK: i32 = 5;

// __WINE_DBCL_WARN: debug class constant (value assumed; WARN class index)
const __WINE_DBCL_WARN: i32 = 2;

unsafe extern "C" fn IsSeparator(Char: TCHAR) -> BOOL {
    return (_istspace!(Char) != 0
        || (Char != 0 && !_tcschr(STANDARD_SEPS, Char).is_null())) as BOOL;
}

unsafe extern "C" fn ResetParser(Pos_: PTCHAR) {
    // C's parameter `Pos` (renamed: the island has a constant of that name).
    bParseError = FALSE as BOOL;
    ParsePos = Pos_;
    OldParsePos = ParsePos;
}

unsafe extern "C" fn UnParseToken() {
    ParsePos = OldParsePos;

    /* Debugging support */
    if fDumpTokens != 0 {
        let fmt: [u16; 17] = [
            'U' as u16, 'n' as u16, 'g' as u16, 'e' as u16, 't' as u16, 't' as u16,
            'i' as u16, 'n' as u16, 'g' as u16, ':' as u16, ' ' as u16, '\'' as u16,
            '%' as u16, 's' as u16, '\'' as u16, '\n' as u16, 0,
        ];
        ConOutPrintf!(fmt.as_ptr(), ParsePos);
    }
}

unsafe extern "C" fn InitParser() {
    *(core::ptr::addr_of_mut!(CurrentToken) as *mut TCHAR) = 0 as TCHAR;
    CurrentTokenType = TOK_END;
    InsideBlock = 0 as INT;
    ResetParser(core::ptr::addr_of_mut!(ParseLine) as PTCHAR);
}

unsafe extern "C" fn ParseChar() -> TCHAR {
    let mut Char: TCHAR = 0;
    if bParseError != 0 {
        return 0;
    }
    loop {
        loop {
            let fresh43 = ParsePos;
            ParsePos = ParsePos.offset(1);
            Char = *fresh43;
            if !(Char == ('\r' as u16)) {
                break;
            }
        }
        if Char == 0 {
            ParsePos = ParsePos.offset(-1);
        }
        if !(Char == 0 && bLineContinuations != 0) {
            break;
        }
        if ReadLine(&raw mut ParseLine as *mut TCHAR, (TRUE) as BOOL) == 0 {
            bParseError = TRUE as BOOL;
            break;
        } else {
            ResetParser(&raw mut ParseLine as PTCHAR);
            if !(*ParsePos != 0) {
                break;
            }
        }
    }
    return Char;
}
#[no_mangle]
pub unsafe extern "C" fn ParseErrorEx(s: PCTSTR) {
    // C's `if (!bParseError)`: a logical not. `!bParseError` on the i32 is bitwise, true for
    // 0 AND 1, so every syntax error after the first was reported again.
    if bParseError == 0 {
        error_syntax(s);
    }
    bParseError = (TRUE) as i32;
}

unsafe extern "C" fn ParseError() {
    ParseErrorEx(if CurrentTokenType != TOK_END {
        core::ptr::addr_of_mut!(CurrentToken) as *mut TCHAR as PCTSTR
    } else {
        core::ptr::null_mut::<TCHAR>() as PCTSTR
    });
}

unsafe extern "C" fn ParseTokenEx(
    mut PrefixOperator: TCHAR,
    ExtraEnd: TCHAR,
    Separators: PCTSTR,
    bHandleContinuations_param: BOOL,
) -> TOK_TYPE {
    let mut Type: TOK_TYPE = TOK_END;
    let mut CurrentTokStart: PTCHAR = core::ptr::addr_of_mut!(CurrentToken) as PTCHAR;
    let mut Out: PTCHAR = CurrentTokStart;
    let mut Char: TCHAR = 0;
    let mut bInQuote: BOOL = (FALSE) as i32;
    let ParenOps: [TCHAR; 6] = [
        '(' as TCHAR,
        ')' as TCHAR,
        '&' as TCHAR,
        '|' as TCHAR,
        '"' as TCHAR,
        0,
    ];
    let RedirOps: [TCHAR; 5] = [
        '&' as TCHAR,
        '|' as TCHAR,
        '<' as TCHAR,
        '>' as TCHAR,
        0,
    ];

    /* Start with what we have at current ParsePos */
    OldParsePos = ParsePos;

    Char = ParseChar();
    'scan: while Char != 0 && Char != ('\n' as TCHAR) {
        'body: {
            bInQuote ^= (Char == ('"' as TCHAR)) as BOOL;
            if bInQuote == 0 {
                if !Separators.is_null() {
                    if _istspace!(Char) != 0 || !_tcschr(Separators, Char).is_null() {
                        /* Skip leading separators */
                        if Out == CurrentTokStart {
                            break 'body;
                        }
                        break 'scan;
                    }
                }

                /* Check for prefix operator */
                if Out == CurrentTokStart && Char == PrefixOperator {
                    break 'scan;
                }

                /* Check for numbered redirection. */
                if _istdigit!(Char) != 0
                    && (ParsePos == OldParsePos.offset(1 as isize)
                        || IsSeparator(
                            *ParsePos.offset(-(2 as isize)),
                        ) != 0
                        || !_tcschr(
                            ParenOps.as_ptr(),
                            *ParsePos.offset(-(2 as isize)),
                        )
                        .is_null())
                    && (*ParsePos == ('<' as TCHAR) || *ParsePos == ('>' as TCHAR))
                {
                    break 'scan;
                }

                /* Check for other delimiters / operators */
                if Char == ExtraEnd {
                    break 'scan;
                }
                if InsideBlock != 0 && Char == (')' as TCHAR) {
                    break 'scan;
                }
                if !_tcschr(RedirOps.as_ptr(), Char).is_null() {
                    break 'scan;
                }

                if bHandleContinuations_param != 0 && Char == ('^' as TCHAR) {
                    Char = ParseChar();
                    /* Eat up a \n, allowing line continuation */
                    if Char == ('\n' as TCHAR) {
                        Char = ParseChar();
                    }
                    /* Next character is a forced literal */

                    if Out == CurrentTokStart {
                        /* Ignore any prefix operator if we don't start a new command block */
                        if CurrentTokenType != TOK_BEGIN_BLOCK {
                            PrefixOperator = 0;
                        }
                    }
                }
            }
            if Out
                == (core::ptr::addr_of_mut!(CurrentToken) as PTCHAR).offset(
                    (CMDLINE_LENGTH - 1) as isize,
                )
            {
                break 'scan;
            }
            *Out = Char;
            Out = Out.offset(1);
        }
        Char = ParseChar();
    }

    /*
     * We exited the parsing loop. If the current character is the first one
     * (Out == CurrentTokStart), interpret it as an operator. Otherwise,
     * terminate the current token (type TOK_NORMAL) and keep the current
     * character so that it can be refetched as an operator at the next call.
     */

    if Out != CurrentTokStart {
        Type = TOK_NORMAL;
    }
    /*
     * Else we have an operator.
     */
    else if Char == ('@' as TCHAR) {
        Type = TOK_OPERATOR; // TOK_QUIET / TOK_PREFIX_OPERATOR
        *Out = Char;
        Out = Out.offset(1);
        Char = ParseChar();
    } else if Char == ('(' as TCHAR) {
        Type = TOK_BEGIN_BLOCK;
        *Out = Char;
        Out = Out.offset(1);
        Char = ParseChar();
    } else if Char == (')' as TCHAR) {
        Type = TOK_END_BLOCK;
        *Out = Char;
        Out = Out.offset(1);
        Char = ParseChar();
    } else if Char == ('&' as TCHAR) || Char == ('|' as TCHAR) {
        Type = TOK_OPERATOR;
        *Out = Char;
        Out = Out.offset(1);
        Char = ParseChar();
        /* Check for '&&' or '||' */
        if Char == *Out.offset(-(1 as isize)) {
            *Out = Char;
            Out = Out.offset(1);
            Char = ParseChar();
        }
    } else if _istdigit!(Char) != 0 || (Char == ('<' as TCHAR) || Char == ('>' as TCHAR)) {
        Type = TOK_REDIRECTION;
        if _istdigit!(Char) != 0 {
            *Out = Char;
            Out = Out.offset(1);
            Char = ParseChar();
        }
        /* By construction (see the while-loop above),
         * the next character must be a redirection. */

        *Out = Char;
        Out = Out.offset(1);
        Char = ParseChar();
        if Char == *Out.offset(-(1 as isize)) {
            /* Strangely, the tokenizer allows << as well as >>... (it
             * will cause an error when trying to parse it though) */
            *Out = Char;
            Out = Out.offset(1);
            Char = ParseChar();
        }
        if Char == ('&' as TCHAR) {
            *Out = Char;
            Out = Out.offset(1);
            loop {
                Char = ParseChar();
                if !(IsSeparator(Char) != 0) {
                    break;
                }
            }
            if _istdigit!(Char) != 0 {
                *Out = Char;
                Out = Out.offset(1);
                Char = ParseChar();
            }
        }
    } else {
        Type = TOK_END;
        *Out = Char;
        Out = Out.offset(1);
    }
    *Out = '\0' as TCHAR;

    /*
     * Rewind the parsing position, so that the current character can be
     * refetched later on. However do this only if it is not NULL and if
     * this is not TOK_END, since we do not want to reparse later the line
     * termination (we could enter into infinite loops, or, in case of line
     * continuation, get unwanted "More?" prompts).
     */
    if Char != 0 && Type != TOK_END {
        ParsePos = ParsePos.offset(-1);
    }

    /* Debugging support */
    if fDumpTokens != 0 {
        let FmtTokens: [TCHAR; 23] = [
            'P' as TCHAR,
            'a' as TCHAR,
            'r' as TCHAR,
            's' as TCHAR,
            'e' as TCHAR,
            'T' as TCHAR,
            'o' as TCHAR,
            'k' as TCHAR,
            'e' as TCHAR,
            'n' as TCHAR,
            ':' as TCHAR,
            ' ' as TCHAR,
            '(' as TCHAR,
            '%' as TCHAR,
            'd' as TCHAR,
            ')' as TCHAR,
            ' ' as TCHAR,
            '\'' as TCHAR,
            '%' as TCHAR,
            's' as TCHAR,
            '\'' as TCHAR,
            '\n' as TCHAR,
            0,
        ];
        ConOutPrintf!(
            FmtTokens.as_ptr(),
            Type as core::ffi::c_int,
            core::ptr::addr_of_mut!(CurrentToken) as PTCHAR,
        );
    }

    CurrentTokenType = Type;
    return CurrentTokenType;
}

unsafe extern "C" fn ParseToken(ExtraEnd: TCHAR, Separators: PCTSTR) -> INT {
    return ParseTokenEx(0, ExtraEnd, Separators, bHandleContinuations) as INT;
}

unsafe extern "C" fn AllocCommand(mut Type: COMMAND_TYPE, mut CmdHead: PCTSTR, mut CmdTail: PCTSTR) -> *mut PARSED_COMMAND {
    let mut Cmd: *mut PARSED_COMMAND;

    match Type {
        C_COMMAND | C_REM => {
            let mut CmdHeadLen: SIZE_T = (_tcslen(CmdHead) + 1) as _;
            let mut CmdTailLen: SIZE_T = (_tcslen(CmdTail) + 1) as _;

            Cmd = cmd_alloc!((FIELD_OFFSET!(PARSED_COMMAND, __bindgen_anon_1.Command.First) as SIZE_T)
                + (CmdHeadLen + CmdTailLen) * (core::mem::size_of::<TCHAR>() as SIZE_T))
                as *mut PARSED_COMMAND;
            if Cmd.is_null() {
                return core::ptr::null_mut();
            }

            (*Cmd).Type = Type;
            (*Cmd).Next = core::ptr::null_mut();
            (*Cmd).Subcommands = core::ptr::null_mut();
            (*Cmd).Redirections = core::ptr::null_mut(); // Is assigned by the calling function
            memcpy(
                core::ptr::addr_of_mut!((*Cmd).__bindgen_anon_1.Command.First) as *mut TCHAR as *mut core::ffi::c_void,
                CmdHead as *const core::ffi::c_void,
                (CmdHeadLen as usize) * core::mem::size_of::<TCHAR>(),
            );
            (*Cmd).__bindgen_anon_1.Command.Rest =
                (core::ptr::addr_of_mut!((*Cmd).__bindgen_anon_1.Command.First) as *mut TCHAR)
                    .offset(CmdHeadLen as isize) as PTSTR;
            memcpy(
                (*Cmd).__bindgen_anon_1.Command.Rest as *mut core::ffi::c_void,
                CmdTail as *const core::ffi::c_void,
                (CmdTailLen as usize) * core::mem::size_of::<TCHAR>(),
            );
            return Cmd;
        }

        C_QUIET | C_BLOCK | C_MULTI
        | C_OR | C_AND | C_PIPE => {
            Cmd = cmd_alloc!(core::mem::size_of::<PARSED_COMMAND>())
                as *mut PARSED_COMMAND;
            if Cmd.is_null() {
                return core::ptr::null_mut();
            }

            (*Cmd).Type = Type;
            (*Cmd).Next = core::ptr::null_mut();
            (*Cmd).Subcommands = core::ptr::null_mut();
            (*Cmd).Redirections = core::ptr::null_mut(); // For C_BLOCK only: is assigned by the calling function
            return Cmd;
        }

        C_FOR | C_IF => {
            Cmd = cmd_alloc!(core::mem::size_of::<PARSED_COMMAND>())
                as *mut PARSED_COMMAND;
            if Cmd.is_null() {
                return core::ptr::null_mut();
            }

            memset(
                Cmd as *mut core::ffi::c_void,
                0,
                core::mem::size_of::<PARSED_COMMAND>(),
            );
            (*Cmd).Type = Type;
            return Cmd;
        }

        _ => {
            CmdTrace(
                __WINE_DBCL_ERR as INT, // channel constant (not in import list)
                b"base/shell/cmd/parser.c\0".as_ptr() as LPCSTR,
                520 as INT,
                b"AllocCommand\0".as_ptr() as LPCSTR,
                b"Unknown command type 0x%x\n\0".as_ptr() as LPCSTR,
                Type as core::ffi::c_uint,
            );
            if 0 as core::ffi::c_int == 0 {
                RtlAssert(
                    b"FALSE\0".as_ptr() as *const core::ffi::c_char as PVOID,
                    b"base/shell/cmd/parser.c\0".as_ptr()
                        as *const core::ffi::c_char as PVOID,
                    521 as ULONG,
                    core::ptr::null_mut(),
                );
            }
            return core::ptr::null_mut();
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn FreeCommand(Cmd: *mut PARSED_COMMAND) {
    if !(*Cmd).Subcommands.is_null() {
        FreeCommand((*Cmd).Subcommands);
    }
    if !(*Cmd).Next.is_null() {
        FreeCommand((*Cmd).Next);
    }
    FreeRedirection((*Cmd).Redirections);
    if (*Cmd).Type == C_FOR {
        cmd_free!((*Cmd).__bindgen_anon_1.For.Params as *mut core::ffi::c_void);
        cmd_free!((*Cmd).__bindgen_anon_1.For.List as *mut core::ffi::c_void);
    } else if (*Cmd).Type == C_IF {
        cmd_free!((*Cmd).__bindgen_anon_1.If.LeftArg as *mut core::ffi::c_void);
        cmd_free!((*Cmd).__bindgen_anon_1.If.RightArg as *mut core::ffi::c_void);
    }
    cmd_free!(Cmd as *mut core::ffi::c_void);
}
unsafe extern "C" fn ParseRedirection(mut List: *mut *mut REDIRECTION) -> BOOL {
    let mut Tok: PTSTR = core::ptr::addr_of_mut!(CurrentToken) as PTSTR;
    let mut Redir: *mut REDIRECTION = core::ptr::null_mut();
    let mut RedirMode: REDIR_MODE = REDIR_READ;
    let mut Number: BYTE = 0;

    if !(*Tok == '<' as TCHAR || *Tok == '>' as TCHAR)
        && !(_istdigit!(*Tok as i32) != 0
            && (*Tok.offset(1) == '<' as TCHAR
                || *Tok.offset(1) == '>' as TCHAR))
    {
        if !(CurrentTokenType as ::core::ffi::c_uint
            != TOK_REDIRECTION as ::core::ffi::c_int as ::core::ffi::c_uint)
        {
            RtlAssert(
                b"CurrentTokenType != TOK_REDIRECTION\0" as *const u8 as *const ::core::ffi::c_char
                    as PVOID,
                b"base/shell/cmd/parser.c\0"
                    as *const u8 as *const ::core::ffi::c_char as PVOID,
                563 as ULONG,
                core::ptr::null_mut::<CHAR>(),
            );
        } else {
        }
        return (FALSE) as BOOL;
    }

    if !(CurrentTokenType as ::core::ffi::c_uint
        == TOK_REDIRECTION as ::core::ffi::c_int as ::core::ffi::c_uint
        || CurrentTokenType as ::core::ffi::c_uint
            == TOK_NORMAL as ::core::ffi::c_int as ::core::ffi::c_uint)
    {
        RtlAssert(
            b"(CurrentTokenType == TOK_REDIRECTION) || (CurrentTokenType == TOK_NORMAL)\0"
                as *const u8 as *const ::core::ffi::c_char as PVOID,
            b"base/shell/cmd/parser.c\0" as *const u8
                as *const ::core::ffi::c_char as PVOID,
            567 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    }

    if _istdigit!(*Tok as i32) != 0 {
        let fresh44 = Tok;
        Tok = Tok.offset(1);
        Number = ((*fresh44 as i32) - ('0' as i32)) as BYTE;
    } else {
        Number = (if *Tok == '<' as TCHAR {
            0
        } else {
            1
        }) as BYTE;
    }

    'fail: {
        let fresh45 = Tok;
        Tok = Tok.offset(1);
        if *fresh45 == '<' as TCHAR {
            RedirMode = REDIR_READ;
            /* Forbid '<<' */
            if *Tok == '<' as TCHAR {
                break 'fail;
            }
        } else {
            RedirMode = REDIR_WRITE;
            if *Tok == '>' as TCHAR {
                RedirMode = REDIR_APPEND;
                Tok = Tok.offset(1);
            }
        }

        if *Tok == '&' as TCHAR {
            /* This is a handle redirection: the next character must be one single digit */
            if !(_istdigit!(*Tok.offset(1) as i32) != 0
                && *Tok.offset(2) == 0)
            {
                break 'fail;
            }
        } else {
            /* Get rid of what possibly remains in the token, and retrieve the next one */
            if ParseToken(0, STANDARD_SEPS)
                != TOK_NORMAL
            {
                break 'fail;
            }
            Tok = core::ptr::addr_of_mut!(CurrentToken) as PTSTR;
        }

        /* If a redirection for this handle number already exists, delete it */
        loop {
            Redir = *List;
            if Redir.is_null() {
                break;
            }
            if (*Redir).Number == Number {
                *List = (*Redir).Next as *mut REDIRECTION;
                cmd_free!(Redir as *mut ::core::ffi::c_void);
                continue;
            }
            List = core::ptr::addr_of_mut!((*Redir).Next) as *mut *mut REDIRECTION;
        }

        Redir = cmd_alloc!((FIELD_OFFSET!(REDIRECTION, Filename) as usize + (_tcslen(Tok as *const wchar_t) + 1) * core::mem::size_of::<TCHAR>()) as usize) as *mut REDIRECTION;
        if Redir.is_null() {
            CmdTrace(
                __WINE_DBCL_WARN as ::core::ffi::c_int as INT,
                b"base/shell/cmd/parser.c\0"
                    as *const u8 as LPCSTR,
                625 as INT,
                b"ParseRedirection\0" as *const u8 as LPCSTR,
                b"Cannot allocate memory for Redir!\n\0" as *const u8 as LPCSTR,
            );
        } else {
            (*Redir).Next = core::ptr::null_mut::<_REDIRECTION>();
            (*Redir).OldHandle = INVALID_HANDLE_VALUE;
            (*Redir).Number = Number;
            (*Redir).Mode = RedirMode;
            _tcscpy(
                core::ptr::addr_of_mut!((*Redir).Filename) as *mut TCHAR,
                Tok,
            );
            *List = Redir as *mut REDIRECTION;
            return (TRUE) as BOOL;
        }
    }

    /* fail: */
    ParseError();
    FreeRedirection(*List);
    *List = core::ptr::null_mut::<REDIRECTION>();
    return (FALSE) as BOOL;
}

unsafe extern "C" fn ParseBlock(RedirList: *mut *mut REDIRECTION) -> *mut PARSED_COMMAND {
    let mut Cmd: *mut PARSED_COMMAND = core::ptr::null_mut::<PARSED_COMMAND>();
    let mut Sub: *mut PARSED_COMMAND = core::ptr::null_mut::<PARSED_COMMAND>();
    let mut NextPtr: *mut *mut PARSED_COMMAND = core::ptr::null_mut::<*mut PARSED_COMMAND>();

    Cmd = AllocCommand(C_BLOCK, core::ptr::null::<WCHAR>(), core::ptr::null::<WCHAR>());
    if Cmd.is_null() {
        ParseError();
        return core::ptr::null_mut::<PARSED_COMMAND>();
    }

    /* Read the block contents */
    NextPtr = core::ptr::addr_of_mut!((*Cmd).Subcommands) as *mut *mut PARSED_COMMAND;
    InsideBlock += 1;
    loop {
        while ParseToken(0, STANDARD_SEPS) == TOK_END
            && *(core::ptr::addr_of_mut!(CurrentToken) as *mut TCHAR) == ('\n' as u16)
        {}
        if *(core::ptr::addr_of_mut!(CurrentToken) as *mut TCHAR) != 0
            && *(core::ptr::addr_of_mut!(CurrentToken) as *mut TCHAR) != ('\n' as u16)
        {
            UnParseToken();
        }

        /* Break early if we have nothing else to read. */
        if *(core::ptr::addr_of_mut!(CurrentToken) as *mut TCHAR) == 0 {
            break;
        }

        Sub = ParseCommandOp((C_OP_LOWEST) as COMMAND_TYPE);
        if !Sub.is_null() {
            *NextPtr = Sub;
            NextPtr = core::ptr::addr_of_mut!((*Sub).Next) as *mut *mut PARSED_COMMAND;
        } else if bParseError != 0 {
            InsideBlock -= 1;
            FreeCommand(Cmd);
            return core::ptr::null_mut::<PARSED_COMMAND>();
        }

        if CurrentTokenType == TOK_END_BLOCK {
            break;
        }
    }
    InsideBlock -= 1;

    /* Fail if the block was not terminated, or if we have an empty block. */
    if CurrentTokenType != TOK_END_BLOCK || (*Cmd).Subcommands.is_null() {
        ParseError();
        FreeCommand(Cmd);
        return core::ptr::null_mut::<PARSED_COMMAND>();
    }

    /* Process any trailing redirections and append them to the list */
    while ParseToken(0, STANDARD_SEPS) != TOK_END {
        if ParseRedirection(RedirList) == 0 {
            /* If an actual error happened in ParseRedirection(), bail out */
            if bParseError != 0 {
                FreeCommand(Cmd);
                return core::ptr::null_mut::<PARSED_COMMAND>();
            }
            /* Otherwise it just returned FALSE; unparse the token and refetch it. */
            break;
        }
    }
    if CurrentTokenType != TOK_END {
        UnParseToken();

        /* Call ParseToken() again to compensate. */
        ParseToken(0, STANDARD_SEPS);
    }

    return Cmd;
}

unsafe extern "C" fn ParseIf() -> *mut PARSED_COMMAND {
    let mut Cmd: *mut PARSED_COMMAND = core::ptr::null_mut();

    Cmd = AllocCommand(C_IF, core::ptr::null_mut(), core::ptr::null_mut());
    if Cmd.is_null() {
        ParseError();
        return core::ptr::null_mut();
    }

    'error: {
        if bEnableExtensions != 0
            && (_tcsicmp(core::ptr::addr_of_mut!(CurrentToken) as *mut TCHAR, (&[0x002f, 0x0049, 0x0000u16]).as_ptr() /* _T "/I" */)
                == 0)
        {
            (*Cmd).__bindgen_anon_1.If.Flags = ((*Cmd).__bindgen_anon_1.If.Flags as i32 | (IFFLAG_IGNORECASE as i32)) as _;
            ParseToken(0, STANDARD_SEPS);
        }
        if _tcsicmp(core::ptr::addr_of_mut!(CurrentToken) as *mut TCHAR, (&[0x006e, 0x006f, 0x0074, 0x0000u16]).as_ptr() /* _T "not" */)
            == 0
        {
            (*Cmd).__bindgen_anon_1.If.Flags = ((*Cmd).__bindgen_anon_1.If.Flags as i32 | (IFFLAG_NEGATE as i32)) as _;
            ParseToken(0, STANDARD_SEPS);
        }

        if CurrentTokenType != TOK_NORMAL {
            break 'error;
        }

        'condition_done: {
            /* Check for unary operators */
            while ((*Cmd).__bindgen_anon_1.If.Operator as i32) <= (IF_MAX_UNARY as i32) {
                'body: {
                    /* Skip the extended operators if the extensions are disabled */
                    if bEnableExtensions == 0 && ((*Cmd).__bindgen_anon_1.If.Operator as i32) >= (IF_CMDEXTVERSION as i32) {
                        break 'body;
                    }

                    if _tcsicmp(
                        core::ptr::addr_of_mut!(CurrentToken) as *mut TCHAR,
                        IfOperatorString[(*Cmd).__bindgen_anon_1.If.Operator as usize],
                    ) == 0
                    {
                        if ParseToken(0, STANDARD_SEPS)
                            != TOK_NORMAL
                        {
                            break 'error;
                        }
                        (*Cmd).__bindgen_anon_1.If.RightArg = cmd_dup!(core::ptr::addr_of_mut!(CurrentToken) as *mut TCHAR);
                        break 'condition_done;
                    }
                }
                (*Cmd).__bindgen_anon_1.If.Operator += 1;
            }

            /* It must be a two-argument (comparison) operator. It could be ==, so
             * the equals sign can't be treated as whitespace here. */
            (*Cmd).__bindgen_anon_1.If.LeftArg = cmd_dup!(core::ptr::addr_of_mut!(CurrentToken) as *mut TCHAR);
            ParseToken(0, (&[0x002c, 0x003b, 0x0000u16]).as_ptr() /* _T ",;" */);

            /* The right argument can come immediately after == */
            if _tcsnicmp(
                core::ptr::addr_of_mut!(CurrentToken) as *mut TCHAR,
                (&[0x003d, 0x003d, 0x0000u16]).as_ptr() /* _T "==" */,
                2,
            ) == 0
                && CurrentToken[2 as usize] != 0
            {
                (*Cmd).__bindgen_anon_1.If.RightArg = cmd_dup!(
                    (core::ptr::addr_of_mut!(CurrentToken) as *mut TCHAR)
                        .offset(2 as isize),
                );
                break 'condition_done;
            }

            // Cmd->If.Operator == IF_MAX_UNARY + 1;
            while ((*Cmd).__bindgen_anon_1.If.Operator as i32) <= (IF_MAX_COMPARISON as i32) {
                'body2: {
                    /* Skip the extended operators if the extensions are disabled */
                    if bEnableExtensions == 0 && ((*Cmd).__bindgen_anon_1.If.Operator as i32) >= (IF_EQU as i32) {
                        // (Cmd->If.Operator > IF_STRINGEQ)
                        break 'body2;
                    }

                    if _tcsicmp(
                        core::ptr::addr_of_mut!(CurrentToken) as *mut TCHAR,
                        IfOperatorString[(*Cmd).__bindgen_anon_1.If.Operator as usize],
                    ) == 0
                    {
                        if ParseToken(0, STANDARD_SEPS)
                            != TOK_NORMAL
                        {
                            break 'error;
                        }
                        (*Cmd).__bindgen_anon_1.If.RightArg = cmd_dup!(core::ptr::addr_of_mut!(CurrentToken) as *mut TCHAR);
                        break 'condition_done;
                    }
                }
                (*Cmd).__bindgen_anon_1.If.Operator += 1;
            }
            break 'error;
        }

        // condition_done:
        (*Cmd).Subcommands = ParseCommandOp((C_OP_LOWEST) as COMMAND_TYPE);
        if (*Cmd).Subcommands.is_null() {
            break 'error;
        }
        if _tcsicmp(core::ptr::addr_of_mut!(CurrentToken) as *mut TCHAR, (&[0x0065, 0x006c, 0x0073, 0x0065, 0x0000u16]).as_ptr() /* _T "else" */)
            == 0
        {
            (*(*Cmd).Subcommands).Next = ParseCommandOp((C_OP_LOWEST) as COMMAND_TYPE);
            if (*(*Cmd).Subcommands).Next.is_null() {
                break 'error;
            }
        }

        return Cmd;
    }

    // error:
    FreeCommand(Cmd);
    ParseError();
    return core::ptr::null_mut();
}

unsafe extern "C" fn ParseFor() -> *mut PARSED_COMMAND {
    let mut Cmd: *mut PARSED_COMMAND;

    /* Use the scratch buffer */
    let mut List: PTSTR = core::ptr::addr_of_mut!(TempBuf) as PTSTR;
    let mut Pos_local: PTCHAR = List as PTCHAR;

    Cmd = AllocCommand(C_FOR, core::ptr::null(), core::ptr::null());
    if Cmd.is_null() {
        ParseError();
        return core::ptr::null_mut();
    }

    'error: {
        /* Skip the extended FOR syntax if extensions are disabled */
        if bEnableExtensions != 0 {
            loop {
                if _tcsicmp(
                    core::ptr::addr_of_mut!(CurrentToken) as PTSTR,
                    ['/' as TCHAR, 'D' as TCHAR, 0 as TCHAR].as_ptr(),
                ) == 0
                {
                    (*Cmd).__bindgen_anon_1.For.Switches |= (FOR_DIRS as u8);
                } else if _tcsicmp(
                    core::ptr::addr_of_mut!(CurrentToken) as PTSTR,
                    ['/' as TCHAR, 'F' as TCHAR, 0 as TCHAR].as_ptr(),
                ) == 0
                {
                    (*Cmd).__bindgen_anon_1.For.Switches |= (FOR_F as u8);
                    if (*Cmd).__bindgen_anon_1.For.Params.is_null() {
                        ParseToken(0, STANDARD_SEPS);
                        if CurrentToken[0] == '/' as TCHAR
                            || CurrentToken[0] == '%' as TCHAR
                        {
                            break;
                        }
                        (*Cmd).__bindgen_anon_1.For.Params = cmd_dup!(core::ptr::addr_of_mut!(CurrentToken) as PTSTR);
                    }
                } else if _tcsicmp(
                    core::ptr::addr_of_mut!(CurrentToken) as PTSTR,
                    ['/' as TCHAR, 'L' as TCHAR, 0 as TCHAR].as_ptr(),
                ) == 0
                {
                    (*Cmd).__bindgen_anon_1.For.Switches |= (FOR_LOOP as u8);
                } else if _tcsicmp(
                    core::ptr::addr_of_mut!(CurrentToken) as PTSTR,
                    ['/' as TCHAR, 'R' as TCHAR, 0 as TCHAR].as_ptr(),
                ) == 0
                {
                    (*Cmd).__bindgen_anon_1.For.Switches |= (FOR_RECURSIVE as u8);
                    if (*Cmd).__bindgen_anon_1.For.Params.is_null() {
                        ParseToken(0, STANDARD_SEPS);
                        if CurrentToken[0] == '/' as TCHAR
                            || CurrentToken[0] == '%' as TCHAR
                        {
                            break;
                        }
                        StripQuotes(core::ptr::addr_of_mut!(CurrentToken) as PTSTR);
                        (*Cmd).__bindgen_anon_1.For.Params = cmd_dup!(core::ptr::addr_of_mut!(CurrentToken) as PTSTR);
                    }
                } else {
                    break;
                }

                ParseToken(0, STANDARD_SEPS);
            }

            /* Make sure there aren't two different switches specified
             * at the same time, unless they're /D and /R */
            if (((*Cmd).__bindgen_anon_1.For.Switches as i32)
                & ((*Cmd).__bindgen_anon_1.For.Switches as i32 - 1))
                != 0
                && (((*Cmd).__bindgen_anon_1.For.Switches as i32)) as u32
                    != (FOR_DIRS | FOR_RECURSIVE)
            {
                break 'error;
            }
        }

        /* parseForBody */

        /* Variable name should be % and just one other character */
        if CurrentToken[0] != '%' as TCHAR
            || _tcslen(core::ptr::addr_of_mut!(CurrentToken) as PCTSTR)
                != 2
        {
            break 'error;
        }
        (*Cmd).__bindgen_anon_1.For.Variable = CurrentToken[1];

        ParseToken(0, STANDARD_SEPS);
        if _tcsicmp(
            core::ptr::addr_of_mut!(CurrentToken) as PTSTR,
            ['i' as TCHAR, 'n' as TCHAR, 0 as TCHAR].as_ptr(),
        ) != 0
        {
            break 'error;
        }

        if ParseToken('(' as TCHAR, STANDARD_SEPS) != TOK_BEGIN_BLOCK {
            break 'error;
        }

        loop {
            /* Pretend we're inside a block so the tokenizer will stop on ')' */
            InsideBlock += 1;
            ParseToken(0, STANDARD_SEPS);
            InsideBlock -= 1;

            if CurrentTokenType == TOK_END_BLOCK {
                break;
            }

            /* Skip past the \n */
            if (CurrentTokenType == TOK_END)
                && CurrentToken[0] == '\n' as TCHAR
            {
                continue;
            }

            if CurrentTokenType != TOK_NORMAL {
                break 'error;
            }

            if Pos_local != List {
                let fresh46 = Pos_local;
                Pos_local = Pos_local.add(1);
                *fresh46 = ' ' as TCHAR;
            }

            if Pos_local.add(_tcslen(core::ptr::addr_of_mut!(CurrentToken) as PCTSTR) as usize)
                >= List.add(CMDLINE_LENGTH as usize)
            {
                break 'error;
            }
            Pos_local = _stpcpy(
                Pos_local as LPTSTR,
                core::ptr::addr_of_mut!(CurrentToken) as *mut TCHAR as LPCTSTR,
            ) as PTCHAR;
        }
        *Pos_local = 0 as TCHAR;
        (*Cmd).__bindgen_anon_1.For.List = cmd_dup!(List);

        ParseToken(0, STANDARD_SEPS);
        if _tcsicmp(
            core::ptr::addr_of_mut!(CurrentToken) as PTSTR,
            ['d' as TCHAR, 'o' as TCHAR, 0 as TCHAR].as_ptr(),
        ) != 0
        {
            break 'error;
        }

        (*Cmd).Subcommands = ParseCommandOp((C_OP_LOWEST) as COMMAND_TYPE);
        if (*Cmd).Subcommands.is_null() {
            break 'error;
        }

        return Cmd;
    }

    /* error: */
    FreeCommand(Cmd);
    ParseError();
    return core::ptr::null_mut();
}

unsafe extern "C" fn ParseRem() -> *mut PARSED_COMMAND {
    let mut Cmd: *mut PARSED_COMMAND = ::core::ptr::null_mut::<PARSED_COMMAND>();
    /* The common scratch buffer already contains the name of the command */
    let mut ParsedLine: PTSTR = core::ptr::addr_of_mut!(TempBuf) as PTSTR;
    let mut Pos_local: PTCHAR = ParsedLine
        .offset(_tcslen(ParsedLine as *const TCHAR) as isize)
        .offset(1 as isize);
    let mut TailOffset: SIZE_T = Pos_local.offset_from(ParsedLine) as ::core::ffi::c_longlong as SIZE_T;
    /* Build a minimal command for REM, so that it can still get through the batch echo unparsing */
    /* Unparse the current token, so as to emulate the REM command parsing
     * behaviour of Windows' CMD, that discards everything before the last
     * line continuation. */
    UnParseToken();
    /*
     * Ignore the rest of the line, without any line continuation (but eat the caret).
     * We cannot simply set bLineContinuations to TRUE or FALSE, because we want (only
     * for the REM command), even when bLineContinuations == FALSE, to get the caret,
     * otherwise it would be ignored.
     */
    while ParseTokenEx(
        0 as TCHAR,
        0 as TCHAR,
        ::core::ptr::null::<WCHAR>(),
        (FALSE) as BOOL,
    ) as ::core::ffi::c_uint
        != TOK_END as ::core::ffi::c_uint
    {
        if Pos_local.add(_tcslen(core::ptr::addr_of_mut!(CurrentToken) as *mut TCHAR))
            >= ParsedLine.offset(CMDLINE_LENGTH as isize)
        {
            ParseError();
            return ::core::ptr::null_mut::<PARSED_COMMAND>();
        }
        Pos_local = _stpcpy(
            Pos_local as LPTSTR,
            core::ptr::addr_of_mut!(CurrentToken) as *mut TCHAR as LPCTSTR,
        ) as PTCHAR;
    }
    *Pos_local = ('\0' as u16);
    Cmd = AllocCommand(
        C_REM as COMMAND_TYPE,
        ParsedLine as PCTSTR,
        ParsedLine.offset(TailOffset as isize) as PCTSTR,
    ) as *mut PARSED_COMMAND;
    if Cmd.is_null() {
        ParseError();
        return ::core::ptr::null_mut::<PARSED_COMMAND>();
    }
    return Cmd;
}

unsafe extern "C" fn ParseCommandPart(RedirList: *mut *mut REDIRECTION) -> *mut PARSED_COMMAND {
    let mut Cmd: *mut PARSED_COMMAND;
    let mut Func: unsafe extern "C" fn() -> *mut PARSED_COMMAND =
        ParseFor as unsafe extern "C" fn() -> *mut PARSED_COMMAND;

    /* Use the scratch buffer */
    let ParsedLine: PTSTR = core::ptr::addr_of_mut!(TempBuf) as *mut TCHAR as PTSTR;

    /* We need to copy the current token because it's going to be changed below by the ParseToken() calls */
    let mut Pos_local: PTCHAR = _stpcpy(ParsedLine as LPTSTR, core::ptr::addr_of_mut!(CurrentToken) as *mut TCHAR as LPCTSTR)
        .offset(1 as isize);
    let TailOffset: SIZE_T = Pos_local.offset_from(ParsedLine) as SIZE_T;

    /* Check for special forms */
    if ({
        Func = ParseFor as unsafe extern "C" fn() -> *mut PARSED_COMMAND;
        _tcsicmp(ParsedLine as LPCTSTR, (&[0x0046, 0x004f, 0x0052, 0x0000u16]).as_ptr() /* _T "FOR" */) == 0
    }) || ({
        Func = ParseIf as unsafe extern "C" fn() -> *mut PARSED_COMMAND;
        _tcsicmp(ParsedLine as LPCTSTR, (&[0x0049, 0x0046, 0x0000u16]).as_ptr() /* _T "IF" */) == 0
    }) || ({
        Func = ParseRem as unsafe extern "C" fn() -> *mut PARSED_COMMAND;
        _tcsicmp(ParsedLine as LPCTSTR, (&[0x0052, 0x0045, 0x004d, 0x0000u16]).as_ptr() /* _T "REM" */) == 0
    })
    {
        let mut pHelp: PTCHAR;

        ParseToken(0, STANDARD_SEPS);

        pHelp = _tcsstr(core::ptr::addr_of_mut!(CurrentToken) as *mut TCHAR, (&[0x002f, 0x003f, 0x0000u16]).as_ptr() /* _T "/?" */) as PTCHAR;
        if !pHelp.is_null()
            && (if Func == ParseIf as unsafe extern "C" fn() -> *mut PARSED_COMMAND {
                *pHelp.offset(2 as isize) == ('/' as u16)
                    || *pHelp.offset(2 as isize) == 0 as TCHAR
            } else {
                TRUE != 0
            })
        {
            /* /? was found within the first token */
            ParseToken(0, STANDARD_SEPS);
        } else {
            pHelp = core::ptr::null_mut::<TCHAR>();
        }
        if !pHelp.is_null() && CurrentTokenType == TOK_NORMAL {
            /* We encountered /? first, but is followed
             * by another token: that's an error. */
            ParseError();
            return core::ptr::null_mut::<PARSED_COMMAND>();
        }

        /* Do actual parsing only if no help is present */
        if pHelp.is_null() {
            /* FOR and IF commands cannot have leading redirection, but REM can */
            if !(*RedirList).is_null() && (Func == ParseFor as unsafe extern "C" fn() -> *mut PARSED_COMMAND
                || Func == ParseIf as unsafe extern "C" fn() -> *mut PARSED_COMMAND)
            {
                /* Display the culprit command and fail */
                ParseErrorEx(ParsedLine as PCTSTR);
                return core::ptr::null_mut::<PARSED_COMMAND>();
            }

            return Func();
        }

        /* Otherwise, run FOR,IF,REM as regular commands only for help support */
        if Pos_local.offset(_tcslen((&[0x002f, 0x003f, 0x0000u16]).as_ptr() /* _T "/?" */) as isize) >= ParsedLine.offset(CMDLINE_LENGTH as isize) {
            ParseError();
            return core::ptr::null_mut::<PARSED_COMMAND>();
        }
        Pos_local = _stpcpy(Pos_local as LPTSTR, (&[0x002f, 0x003f, 0x0000u16]).as_ptr() /* _T "/?" */) as PTCHAR;
    } else {
        ParseToken(0, core::ptr::null::<WCHAR>());
    }

    /* Now get the tail */
    while CurrentTokenType != TOK_END {
        if CurrentTokenType == TOK_NORMAL {
            if Pos_local.offset(_tcslen(core::ptr::addr_of_mut!(CurrentToken) as *mut TCHAR) as isize)
                >= ParsedLine.offset(CMDLINE_LENGTH as isize)
            {
                ParseError();
                return core::ptr::null_mut::<PARSED_COMMAND>();
            }
            Pos_local = _stpcpy(Pos_local as LPTSTR, core::ptr::addr_of_mut!(CurrentToken) as *mut TCHAR as LPCTSTR) as PTCHAR;
        } else {
            /* Process any trailing redirections and append them to the list */
            let mut bSuccess: BOOL = (FALSE) as i32;

            while CurrentTokenType != TOK_END {
                if ParseRedirection(RedirList) == 0 {
                    /* If an actual error happened in ParseRedirection(), bail out */
                    if bParseError != 0 {
                        return core::ptr::null_mut::<PARSED_COMMAND>();
                    }

                    /* Otherwise it just returned FALSE because the current token
                     * is not a redirection. Unparse the token and refetch it. */
                    break;
                }
                bSuccess = (TRUE) as i32;

                ParseToken(0, STANDARD_SEPS);
            }
            if CurrentTokenType == TOK_END {
                break;
            }

            /* Unparse the current token, and reparse it below with no separators */
            UnParseToken();

            /* If bSuccess == FALSE, we know that it's still the old fetched token, but
             * it has been unparsed, so we need to refetch it before quitting the loop. */
            if bSuccess == 0 {
                ParseToken(0, core::ptr::null::<WCHAR>());
                break;
            }
        }

        ParseToken(0, core::ptr::null::<WCHAR>());
    }
    *Pos_local = ('\0' as u16);

    Cmd = AllocCommand(
        C_COMMAND,
        ParsedLine as PCTSTR,
        ParsedLine.offset(TailOffset as isize) as PCTSTR,
    );
    if Cmd.is_null() {
        CmdTrace(
            __WINE_DBCL_WARN as INT,
            b"parser.c\0".as_ptr() as LPCSTR,
            1201 as INT,
            b"ParseCommandPart\0".as_ptr() as LPCSTR,
            b"Cannot allocate memory for Cmd!\n\0".as_ptr() as LPCSTR,
        );
        ParseError();
        return core::ptr::null_mut::<PARSED_COMMAND>();
    }
    return Cmd;
}

unsafe extern "C" fn ParsePrimary() -> *mut PARSED_COMMAND {
    let mut Cmd: *mut PARSED_COMMAND = ::core::ptr::null_mut();
    let mut RedirList: *mut REDIRECTION = ::core::ptr::null_mut();

    /* In this context, '@' is considered as a separate token */
    if (*(core::ptr::addr_of_mut!(CurrentToken) as *mut TCHAR) == ('@' as u16))
        && (CurrentTokenType as ::core::ffi::c_uint == TOK_OPERATOR as ::core::ffi::c_uint)
    {
        Cmd = AllocCommand(
            C_QUIET,
            core::ptr::null(),
            core::ptr::null(),
        );
        if Cmd.is_null() {
            CmdTrace(
                __WINE_DBCL_WARN as ::core::ffi::c_int as INT,
                b"base/shell/cmd/parser.c\0"
                    as *const u8 as LPCSTR,
                1220 as INT,
                b"ParsePrimary\0" as *const u8 as LPCSTR,
                b"Cannot allocate memory for Cmd!\n\0" as *const u8 as LPCSTR,
            );
            ParseError();
            return core::ptr::null_mut();
        }
        /* @ acts like a unary operator with low precedence,
         * so call the top-level parser */
        (*Cmd).Subcommands = ParseCommandOp((C_OP_LOWEST) as COMMAND_TYPE) as *mut _PARSED_COMMAND;
        return Cmd;
    }

    /* Process leading redirections and get the head of the command */
    {
        let mut bSuccess: BOOL = (FALSE) as i32;
        while CurrentTokenType as ::core::ffi::c_uint
            != TOK_END as ::core::ffi::c_uint
        {
            if ParseRedirection(core::ptr::addr_of_mut!(RedirList)) == 0 {
                /* If an actual error happened in ParseRedirection(), bail out */
                if bParseError != 0 {
                    return core::ptr::null_mut();
                }

                /* Otherwise it just returned FALSE because
                 * the current token is not a redirection. */
                break;
            }
            bSuccess = (TRUE) as i32;

            ParseToken(0, STANDARD_SEPS);
        }
        if bSuccess != 0 {
            /* Unparse the current token, and reparse it with support for parenthesis */
            if CurrentTokenType as ::core::ffi::c_uint
                != TOK_END as ::core::ffi::c_uint
            {
                UnParseToken();
            }

            ParseToken(('(' as u16), STANDARD_SEPS);
        }
    }

    if CurrentTokenType as ::core::ffi::c_uint == TOK_NORMAL as ::core::ffi::c_uint {
        Cmd = ParseCommandPart(core::ptr::addr_of_mut!(RedirList));
    } else if CurrentTokenType as ::core::ffi::c_uint
        == TOK_BEGIN_BLOCK as ::core::ffi::c_uint
    {
        Cmd = ParseBlock(core::ptr::addr_of_mut!(RedirList));
    } else if CurrentTokenType as ::core::ffi::c_uint
        == TOK_END_BLOCK as ::core::ffi::c_uint
        && RedirList.is_null()
    {
        return core::ptr::null_mut();
    }

    if !Cmd.is_null() {
        /* FOR and IF commands cannot have leading redirection
         * (checked by ParseCommandPart(), errors out if so). */
        if !(RedirList.is_null()
            || ((*Cmd).Type as ::core::ffi::c_uint != C_FOR as ::core::ffi::c_uint
                && (*Cmd).Type as ::core::ffi::c_uint != C_IF as ::core::ffi::c_uint))
        {
            RtlAssert(
                b"!RedirList || (Cmd->Type != C_FOR && Cmd->Type != C_IF)\0" as *const u8
                    as *const ::core::ffi::c_char as PVOID,
                b"base/shell/cmd/parser.c\0"
                    as *const u8 as *const ::core::ffi::c_char as PVOID,
                1280 as ULONG,
                core::ptr::null_mut::<CHAR>(),
            );
        }

        /* Save the redirection list in the command */
        (*Cmd).Redirections = RedirList as *mut _REDIRECTION;

        /* Return the new command */
        return Cmd;
    }

    ParseError();
    FreeRedirection(RedirList);
    return core::ptr::null_mut();
}

unsafe extern "C" fn ParseCommandBinaryOp(mut OpType: COMMAND_TYPE) -> *mut PARSED_COMMAND {
    let mut Cmd: *mut PARSED_COMMAND = core::ptr::null_mut();
    if (OpType as u32) == (C_OP_LOWEST as u32) {
        /* Ignore any parser-level comments */
        if bIgnoreParserComments != 0
            && *(core::ptr::addr_of_mut!(CurrentToken) as *mut TCHAR) == (':' as u16)
        {
            /* Ignore the rest of the line, including line continuations */
            while ParseToken(0 as TCHAR, core::ptr::null())
                != TOK_END as i32
            {}
            return core::ptr::null_mut();
        }

        /*
         * Ignore single closing parenthesis outside of command blocks,
         * thus interpreted as a command.
         */
        if (_tcscmp(
            core::ptr::addr_of_mut!(CurrentToken) as *mut TCHAR,
            (&[0x0029, 0x0000u16]).as_ptr() /* _T ")" */,
        ) == 0)
            && ((CurrentTokenType as i32) != (TOK_END_BLOCK as i32))
        {
            /* Ignore the rest of the line, including line continuations */
            while ParseToken(0 as TCHAR, core::ptr::null())
                != TOK_END as i32
            {}
            return core::ptr::null_mut();
        }

        if *(core::ptr::addr_of_mut!(CurrentToken) as *mut TCHAR) == 0
            || *(core::ptr::addr_of_mut!(CurrentToken) as *mut TCHAR) == ('\n' as u16)
        {
            return core::ptr::null_mut();
        }
    }

    if (OpType as u32) == (C_OP_HIGHEST as u32) {
        Cmd = ParsePrimary();
    } else {
        Cmd = ParseCommandBinaryOp(
            ((OpType as u32).wrapping_add(1 as u32)) as COMMAND_TYPE,
        );
    }

    if !Cmd.is_null()
        && _tcscmp(
            core::ptr::addr_of_mut!(CurrentToken) as *mut TCHAR,
            core::ptr::addr_of!(OpString[((OpType as u32).wrapping_sub(C_OP_LOWEST as u32)) as usize]) as *const TCHAR,
        ) == 0
    {
        let mut Left: *mut PARSED_COMMAND = Cmd as *mut PARSED_COMMAND;
        let mut Right: *mut PARSED_COMMAND = core::ptr::null_mut();

        Right = ParseCommandOp(OpType);
        if Right.is_null() {
            /*
             * The '&' operator is allowed to have an empty RHS.
             * In this case, we directly return the LHS only.
             */
            if bParseError == 0 && ((OpType as u32) != (C_MULTI as u32)) {
                ParseError();
            }
            if bParseError != 0 {
                FreeCommand(Left as *mut _PARSED_COMMAND);
                return core::ptr::null_mut();
            }

            return Left as *mut PARSED_COMMAND;
        }

        Cmd = AllocCommand(OpType, core::ptr::null(), core::ptr::null());
        if Cmd.is_null() {
            if !(InsideBlock == 0) {
                RtlAssert(
                    b"InsideBlock == 0\0" as *const u8 as *const ::core::ffi::c_char as PVOID,
                    b"base/shell/cmd/parser.c\0"
                        as *const u8 as *const ::core::ffi::c_char as PVOID,
                    1374 as ULONG,
                    core::ptr::null_mut::<CHAR>(),
                );
            } else {
            };
            CmdTrace(
                __WINE_DBCL_WARN as INT,
                b"base/shell/cmd/parser.c\0"
                    as *const u8 as LPCSTR,
                1430 as INT,
                b"ParseCommandBinaryOp\0" as *const u8 as LPCSTR,
                b"Cannot allocate memory for Cmd!\n\0" as *const u8 as LPCSTR,
            );
            ParseError();
            FreeCommand(Left as *mut _PARSED_COMMAND);
            FreeCommand(Right as *mut _PARSED_COMMAND);
            return core::ptr::null_mut();
        }
        (*Cmd).Subcommands = Left as *mut _PARSED_COMMAND;
        (*Left).Next = Right as *mut _PARSED_COMMAND;
        (*Right).Next = core::ptr::null_mut::<_PARSED_COMMAND>();
    }

    return Cmd as *mut PARSED_COMMAND;
}

unsafe extern "C" fn ParseCommandOp(OpType: COMMAND_TYPE) -> *mut PARSED_COMMAND {
    /* Start parsing: initialize the first token */

    /* Parse the prefix "quiet" operator '@' as a separate command.
     * Thus, @@foo@bar is parsed as: '@', '@', 'foo@bar'. */
    ParseTokenEx(('@' as u16), ('(' as u16), STANDARD_SEPS, bHandleContinuations);

    return ParseCommandBinaryOp(OpType);
}
#[no_mangle]
pub unsafe extern "C" fn ParseCommand(Line: PCTSTR) -> *mut PARSED_COMMAND {
    let mut Cmd: *mut PARSED_COMMAND = core::ptr::null_mut();

    if !Line.is_null() {
        if SubstituteVars(Line, &raw mut ParseLine as PTSTR, ('%' as u16)) == 0 {
            return core::ptr::null_mut();
        }
        bLineContinuations = (FALSE) as i32;
    } else {
        if ReadLine(&raw mut ParseLine as PTSTR, (FALSE) as BOOL) == 0 {
            return core::ptr::null_mut();
        }
        bLineContinuations = (TRUE) as i32;
    }

    InitParser();

    Cmd = ParseCommandOp((C_OP_LOWEST) as COMMAND_TYPE);
    if !Cmd.is_null() {
        bIgnoreEcho = (FALSE) as i32;

        if (CurrentTokenType as core::ffi::c_uint != TOK_END as core::ffi::c_uint)
            && (_tcscmp(&raw mut CurrentToken as PTSTR, (&[0x000a, 0x0000u16]).as_ptr() /* _T "\n" */) != 0)
        {
            ParseError();
        }
        if bParseError != 0 {
            FreeCommand(Cmd);
            return core::ptr::null_mut();
        }

        /* Debugging support */
        if fDumpParse != 0 {
            DumpCommand(Cmd, 0);
        }
    } else {
        bIgnoreEcho = (TRUE) as i32;
    }
    return Cmd;
}

unsafe extern "C" fn DumpRedir(Redirections: *mut REDIRECTION) {
    let mut Redir: *mut REDIRECTION = core::ptr::null_mut();

    if !Redirections.is_null() {
        ConOutPuts!((&[0x0052, 0x0065, 0x0064, 0x0069, 0x0072, 0x003a, 0x0020, 0x0000u16]).as_ptr() /* _T "Redir: " */);
    }
    Redir = Redirections as *mut REDIRECTION;
    while !Redir.is_null() {
        ConOutPrintf!(
            (&[0x0020, 0x0025, 0x0078, 0x0020, 0x0025, 0x0073, 0x0025, 0x0073, 0x0000u16]).as_ptr() /* _T " %x %s%s" */,
            (*Redir).Number as core::ffi::c_int,
            (&raw const RedirString as *const [TCHAR; 3]).offset((*Redir).Mode as isize) as *const TCHAR,
            &raw mut (*Redir).Filename as *mut TCHAR,
        );
        Redir = (*Redir).Next as *mut REDIRECTION;
    }
}
#[no_mangle]
pub unsafe extern "C" fn DumpCommand(mut Cmd: *mut PARSED_COMMAND, mut SpacePad: ULONG) {
    let mut Sub: *mut PARSED_COMMAND = core::ptr::null_mut();
    loop {
        if Cmd.is_null() {
            return;
        }

        /* Space padding */
        ConOutPrintf!((&[0x0025, 0x002a, 0x0073, 0x0000u16]).as_ptr() /* _T "%*s" */, SpacePad, (&[0x0000u16]).as_ptr() /* _T "" */);

        match (*Cmd).Type as ::core::ffi::c_uint {
            C_COMMAND | C_REM => {
                /* Generic command name, and Type */
                ConOutPrintf!(
                    (&[0x0043, 0x006d, 0x0064, 0x003a, 0x0020, 0x0025, 0x0073, 0x0020, 0x0020, 0x0054, 0x0079, 0x0070, 0x0065, 0x003a, 0x0020, 0x0025, 0x0078, 0x0020, 0x0000u16]).as_ptr() /* _T "Cmd: %s  Type: %x " */,
                    (*Cmd).__bindgen_anon_1.Command.First,
                    (*Cmd).Type,
                );
                /* Arguments */
                if !(*Cmd).__bindgen_anon_1.Command.Rest.is_null()
                    && *(*Cmd).__bindgen_anon_1.Command.Rest as ::core::ffi::c_int != 0
                {
                    ConOutPrintf!((&[0x0041, 0x0072, 0x0067, 0x0073, 0x003a, 0x0020, 0x0060, 0x0025, 0x0073, 0x0027, 0x0020, 0x0000u16]).as_ptr() /* _T "Args: `%s' " */, (*Cmd).__bindgen_anon_1.Command.Rest);
                }
                /* Redirections */
                DumpRedir((*Cmd).Redirections as *mut REDIRECTION);

                ConOutChar(('\n' as u16));
                return;
            }

            C_QUIET => {
                ConOutPuts!((&[0x0040, 0x0020, 0x0000u16]).as_ptr() /* _T "@ " */);
                DumpRedir((*Cmd).Redirections as *mut REDIRECTION); // FIXME: Can we have leading redirections??
                ConOutChar(('\n' as u16));

                /*DumpCommand*/
                // DUMP((*Cmd).Subcommands, SpacePad + FACT_0004)
                Cmd = (*Cmd).Subcommands as *mut PARSED_COMMAND;
                SpacePad = SpacePad.wrapping_add(2 as ULONG);
                continue;
            }

            C_BLOCK => {
                ConOutPuts!((&[0x0028, 0x0020, 0x0000u16]).as_ptr() /* _T "( " */);
                DumpRedir((*Cmd).Redirections as *mut REDIRECTION);
                ConOutChar(('\n' as u16));

                SpacePad = SpacePad.wrapping_add(2 as ULONG);

                Sub = (*Cmd).Subcommands as *mut PARSED_COMMAND;
                while !Sub.is_null() {
                    /*
                     * We will emulate Windows' CMD handling of "CRLF" and "&" multi-command
                     * enumeration within parenthesized command blocks.
                     */
                    'body: {
                        if (*Sub).Next.is_null() {
                            DumpCommand(Sub as *mut PARSED_COMMAND, SpacePad);
                            break 'body;
                        }

                        if (*Sub).Type as ::core::ffi::c_uint
                            != C_MULTI as ::core::ffi::c_uint
                        {
                            ConOutPrintf!((&[0x0025, 0x002a, 0x0073, 0x0000u16]).as_ptr() /* _T "%*s" */, SpacePad, (&[0x0000u16]).as_ptr() /* _T "" */);
                            ConOutPuts!((&[0x0043, 0x0052, 0x004c, 0x0046, 0x0020, 0x000a, 0x0000u16]).as_ptr() /* _T "CRLF \n" */);
                            DumpCommand(Sub as *mut PARSED_COMMAND, SpacePad);
                            break 'body;
                        }

                        /* Now, Sub->Type == C_MULTI */

                        Cmd = Sub as *mut PARSED_COMMAND;

                        ConOutPrintf!((&[0x0025, 0x002a, 0x0073, 0x0000u16]).as_ptr() /* _T "%*s" */, SpacePad, (&[0x0000u16]).as_ptr() /* _T "" */);
                        ConOutPrintf!(
                            (&[0x0025, 0x0073, 0x0020, 0x000a, 0x0000u16]).as_ptr() /* _T "%s \n" */,
                            &raw const *(&raw const OpString as *const [TCHAR; 3]).offset(
                                ((*Cmd).Type as ::core::ffi::c_uint).wrapping_sub(
                                    C_OP_LOWEST as ::core::ffi::c_uint,
                                ) as isize,
                            ) as *const TCHAR,
                        );
                        // FIXME: Can we have redirections on these operator-type commands?

                        SpacePad = SpacePad.wrapping_add(2 as ULONG);

                        Cmd = (*Cmd).Subcommands as *mut PARSED_COMMAND;
                        DumpCommand(Cmd, SpacePad);
                        ConOutPrintf!((&[0x0025, 0x002a, 0x0073, 0x0000u16]).as_ptr() /* _T "%*s" */, SpacePad, (&[0x0000u16]).as_ptr() /* _T "" */);
                        ConOutPuts!((&[0x0043, 0x0052, 0x004c, 0x0046, 0x0020, 0x000a, 0x0000u16]).as_ptr() /* _T "CRLF \n" */);
                        DumpCommand((*Cmd).Next as *mut PARSED_COMMAND, SpacePad);

                        // NOTE: Next commands will remain indented.
                    }
                    Sub = (*Sub).Next as *mut PARSED_COMMAND;
                }

                return;
            }

            C_MULTI
            | C_OR
            | C_AND
            | C_PIPE => {
                ConOutPrintf!(
                    (&[0x0025, 0x0073, 0x0020, 0x000a, 0x0000u16]).as_ptr() /* _T "%s \n" */,
                    &raw const *(&raw const OpString as *const [TCHAR; 3]).offset(
                        ((*Cmd).Type as ::core::ffi::c_uint).wrapping_sub(
                            C_OP_LOWEST as ::core::ffi::c_uint,
                        ) as isize,
                    ) as *const TCHAR,
                );
                // FIXME: Can we have redirections on these operator-type commands?

                SpacePad = SpacePad.wrapping_add(2 as ULONG);

                Sub = (*Cmd).Subcommands as *mut PARSED_COMMAND;
                DumpCommand(Sub as *mut PARSED_COMMAND, SpacePad);
                /*DumpCommand*/
                // DUMP(Sub->Next, SpacePad)
                Cmd = (*Sub).Next as *mut PARSED_COMMAND;
                SpacePad = SpacePad;
                continue;
            }

            C_FOR => {
                ConOutPuts!((&[0x0066, 0x006f, 0x0072, 0x0000u16]).as_ptr() /* _T "for" */);
                /* NOTE: FOR cannot have leading redirections */

                if ((*Cmd).__bindgen_anon_1.For.Switches as ::core::ffi::c_int & FOR_DIRS as ::core::ffi::c_int) != 0 {
                    ConOutPuts!((&[0x0020, 0x002f, 0x0044, 0x0000u16]).as_ptr() /* _T " /D" */);
                }
                if ((*Cmd).__bindgen_anon_1.For.Switches as ::core::ffi::c_int & FOR_F as ::core::ffi::c_int) != 0 {
                    ConOutPuts!((&[0x0020, 0x002f, 0x0046, 0x0000u16]).as_ptr() /* _T " /F" */);
                }
                if ((*Cmd).__bindgen_anon_1.For.Switches as ::core::ffi::c_int & FOR_LOOP as ::core::ffi::c_int) != 0 {
                    ConOutPuts!((&[0x0020, 0x002f, 0x004c, 0x0000u16]).as_ptr() /* _T " /L" */);
                }
                if ((*Cmd).__bindgen_anon_1.For.Switches as ::core::ffi::c_int & FOR_RECURSIVE as ::core::ffi::c_int) != 0 {
                    ConOutPuts!((&[0x0020, 0x002f, 0x0052, 0x0000u16]).as_ptr() /* _T " /R" */);
                }
                if !(*Cmd).__bindgen_anon_1.For.Params.is_null() {
                    ConOutPrintf!((&[0x0020, 0x0025, 0x0073, 0x0000u16]).as_ptr() /* _T " %s" */, (*Cmd).__bindgen_anon_1.For.Params);
                }
                ConOutPrintf!(
                    (&[0x0020, 0x0025, 0x0025, 0x0025, 0x0063, 0x0020, 0x0069, 0x006e, 0x0020, 0x0028, 0x0025, 0x0073, 0x0029, 0x0020, 0x0064, 0x006f, 0x000a, 0x0000u16]).as_ptr() /* _T " %%%c in (%s) do\n" */,
                    (*Cmd).__bindgen_anon_1.For.Variable as ::core::ffi::c_int,
                    (*Cmd).__bindgen_anon_1.For.List,
                );
                /*DumpCommand*/
                // DUMP((*Cmd).Subcommands, SpacePad + FACT_0021)
                Cmd = (*Cmd).Subcommands as *mut PARSED_COMMAND;
                SpacePad = SpacePad.wrapping_add(2 as ULONG);
                continue;
            }

            C_IF => {
                ConOutPuts!((&[0x0069, 0x0066, 0x0000u16]).as_ptr() /* _T "if" */);
                /* NOTE: IF cannot have leading redirections */

                if ((*Cmd).__bindgen_anon_1.If.Flags as ::core::ffi::c_int & IFFLAG_IGNORECASE as ::core::ffi::c_int) != 0 {
                    ConOutPuts!((&[0x0020, 0x002f, 0x0049, 0x0000u16]).as_ptr() /* _T " /I" */);
                }

                ConOutChar(('\n' as u16));

                SpacePad = SpacePad.wrapping_add(2 as ULONG);

                /*
                 * Show the IF command condition as a command.
                 * If it is negated, indent the command more.
                 */
                if ((*Cmd).__bindgen_anon_1.If.Flags as ::core::ffi::c_int & IFFLAG_NEGATE as ::core::ffi::c_int) != 0 {
                    ConOutPrintf!((&[0x0025, 0x002a, 0x0073, 0x0000u16]).as_ptr() /* _T "%*s" */, SpacePad, (&[0x0000u16]).as_ptr() /* _T "" */);
                    ConOutPuts!((&[0x006e, 0x006f, 0x0074, 0x000a, 0x0000u16]).as_ptr() /* _T "not\n" */);
                    SpacePad = SpacePad.wrapping_add(2 as ULONG);
                }

                ConOutPrintf!((&[0x0025, 0x002a, 0x0073, 0x0000u16]).as_ptr() /* _T "%*s" */, SpacePad, (&[0x0000u16]).as_ptr() /* _T "" */);

                /*
                 * Command name:
                 * - Unary operator: its name is the command name, and its argument is the command argument.
                 * - Binary operator: its LHS is the command name, its RHS is the command argument.
                 *
                 * Type:
                 * Windows' CMD (Win2k3 / Win7-10) values are as follows:
                 *   CMDEXTVERSION  Type: 0x32 / 0x34
                 *   ERRORLEVEL     Type: 0x33 / 0x35
                 *   DEFINED        Type: 0x34 / 0x36
                 *   EXIST          Type: 0x35 / 0x37
                 *   ==             Type: 0x37 / 0x39 (String Comparison)
                 *
                 * For the following command:
                 *   NOT            Type: 0x36 / 0x38
                 * Windows only prints it without any type / redirection.
                 *
                 * For the following command:
                 *   EQU, NEQ, etc. Type: 0x38 / 0x3a (Generic Comparison)
                 * Windows displays it as command of unknown type.
                 */
                ConOutPrintf!(
                    (&[0x0043, 0x006d, 0x0064, 0x003a, 0x0020, 0x0025, 0x0073, 0x0020, 0x0020, 0x0054, 0x0079, 0x0070, 0x0065, 0x003a, 0x0020, 0x0025, 0x0078, 0x0020, 0x0000u16]).as_ptr() /* _T "Cmd: %s  Type: %x " */,
                    if ((*Cmd).__bindgen_anon_1.If.Operator as ::core::ffi::c_uint)
                        <= IF_MAX_UNARY as ::core::ffi::c_uint
                    {
                        IfOperatorString[(*Cmd).__bindgen_anon_1.If.Operator as usize]
                    } else {
                        (*Cmd).__bindgen_anon_1.If.LeftArg as *const TCHAR
                    },
                    (*Cmd).__bindgen_anon_1.If.Operator as ::core::ffi::c_uint,
                );
                /* Arguments */
                ConOutPrintf!((&[0x0041, 0x0072, 0x0067, 0x0073, 0x003a, 0x0020, 0x0060, 0x0025, 0x0073, 0x0027, 0x0020, 0x0000u16]).as_ptr() /* _T "Args: `%s' " */, (*Cmd).__bindgen_anon_1.If.RightArg);

                ConOutChar(('\n' as u16));

                if ((*Cmd).__bindgen_anon_1.If.Flags as ::core::ffi::c_int & IFFLAG_NEGATE as ::core::ffi::c_int) != 0 {
                    SpacePad = SpacePad.wrapping_sub(2 as ULONG);
                }

                Sub = (*Cmd).Subcommands as *mut PARSED_COMMAND;
                DumpCommand(Sub as *mut PARSED_COMMAND, SpacePad);
                if !(*Sub).Next.is_null() {
                    ConOutPrintf!(
                        (&[0x0025, 0x002a, 0x0073, 0x0000u16]).as_ptr() /* _T "%*s" */,
                        SpacePad.wrapping_sub(2 as ULONG),
                        (&[0x0000u16]).as_ptr() /* _T "" */,
                    );
                    ConOutPuts!((&[0x0065, 0x006c, 0x0073, 0x0065, 0x000a, 0x0000u16]).as_ptr() /* _T "else\n" */);
                    DumpCommand((*Sub).Next as *mut PARSED_COMMAND, SpacePad);
                }
                return;
            }

            _ => {
                ConOutPrintf!((&[0x002a, 0x002a, 0x002a, 0x0020, 0x0055, 0x006e, 0x006b, 0x006e, 0x006f, 0x0077, 0x006e, 0x0020, 0x0074, 0x0079, 0x0070, 0x0065, 0x003a, 0x0020, 0x0025, 0x0078, 0x000a, 0x0000u16]).as_ptr() /* _T "*** Unknown type: %x\n" */, (*Cmd).Type);
                break;
            }
        }
        return;
    }
}
#[no_mangle]
pub unsafe extern "C" fn EchoCommand(mut Cmd: *mut PARSED_COMMAND) {
    let mut Sub: *mut PARSED_COMMAND = core::ptr::null_mut::<PARSED_COMMAND>();
    let mut Redir: *mut REDIRECTION = core::ptr::null_mut::<REDIRECTION>();
    if Cmd.is_null() {
        return;
    }
    match (*Cmd).Type {
        C_COMMAND | C_REM => {
            if SubstituteForVars(
                core::ptr::addr_of!((*Cmd).__bindgen_anon_1.Command.First) as PCTSTR,
                core::ptr::addr_of_mut!(TempBuf) as PTSTR,
            ) != 0
            {
                ConOutPrintf!(
                    ['%' as u16, 's' as u16, 0u16].as_ptr() as PCTSTR,
                    core::ptr::addr_of_mut!(TempBuf) as *mut TCHAR,
                );
            }
            if SubstituteForVars(
                (*Cmd).__bindgen_anon_1.Command.Rest as PCTSTR,
                core::ptr::addr_of_mut!(TempBuf) as PTSTR,
            ) != 0
            {
                ConOutPrintf!(
                    ['%' as u16, 's' as u16, 0u16].as_ptr() as PCTSTR,
                    core::ptr::addr_of_mut!(TempBuf) as *mut TCHAR,
                );
                /* NOTE: For Windows compatibility, add a trailing space after printing the command parameter, if present */
                if *(core::ptr::addr_of_mut!(TempBuf) as *mut TCHAR) != 0 {
                    ConOutChar((' ') as TCHAR);
                }
            }

        }

        C_QUIET => {
            return;
        }

        C_BLOCK => {
            let mut bIsFirstCmdCRLF: BOOLEAN = 0;

            ConOutChar(('(') as TCHAR);

            Sub = (*Cmd).Subcommands as *mut PARSED_COMMAND;

            bIsFirstCmdCRLF = (!Sub.is_null() && !(*Sub).Next.is_null()) as i32 as BOOLEAN;

            /*
             * We will emulate Windows' CMD handling of "CRLF" and "&" multi-command
             * enumeration within parenthesized command blocks.
             */
            bIsFirstCmdCRLF = (bIsFirstCmdCRLF as i32 != 0
                && (*Sub).Type != C_MULTI) as i32 as BOOLEAN;

            /*
             * Single-command block: display all on one line.
             * Multi-command block: display commands on separate lines.
             */
            if bIsFirstCmdCRLF != 0 {
                ConOutChar(('\n') as TCHAR);
            }

            while !Sub.is_null() {
                EchoCommand(Sub);
                if !(*Sub).Next.is_null() {
                    ConOutPuts!([' ' as u16, '\n' as u16, ' ' as u16, 0u16].as_ptr() as PCTSTR);
                }
                Sub = (*Sub).Next as *mut PARSED_COMMAND;
            }

            if bIsFirstCmdCRLF != 0 {
                ConOutChar(('\n') as TCHAR);
            }

            /* NOTE: For Windows compatibility, add a trailing space after printing the closing parenthesis */
            ConOutPuts!([')' as u16, ' ' as u16, 0u16].as_ptr() as PCTSTR);

        }

        C_MULTI | C_OR | C_AND | C_PIPE => {
            Sub = (*Cmd).Subcommands as *mut PARSED_COMMAND;
            EchoCommand(Sub);
            ConOutPrintf!(
                [' ' as u16, '%' as u16, 's' as u16, ' ' as u16, 0u16].as_ptr() as PCTSTR,
                core::ptr::addr_of!(OpString[((*Cmd).Type - (C_OP_LOWEST) as u32) as usize]) as *const TCHAR,
            );
            EchoCommand((*Sub).Next as *mut PARSED_COMMAND);

        }

        C_FOR => {
            ConOutPuts!(['f' as u16, 'o' as u16, 'r' as u16, 0u16].as_ptr() as PCTSTR);
            if ((*Cmd).__bindgen_anon_1.For.Switches & (FOR_DIRS as u8)) != 0 {
                ConOutPuts!([' ' as u16, '/' as u16, 'D' as u16, 0u16].as_ptr() as PCTSTR);
            }
            if ((*Cmd).__bindgen_anon_1.For.Switches & (FOR_F as u8)) != 0 {
                ConOutPuts!([' ' as u16, '/' as u16, 'F' as u16, 0u16].as_ptr() as PCTSTR);
            }
            if ((*Cmd).__bindgen_anon_1.For.Switches & (FOR_LOOP as u8)) != 0 {
                ConOutPuts!([' ' as u16, '/' as u16, 'L' as u16, 0u16].as_ptr() as PCTSTR);
            }
            if ((*Cmd).__bindgen_anon_1.For.Switches & (FOR_RECURSIVE as u8)) != 0 {
                ConOutPuts!([' ' as u16, '/' as u16, 'R' as u16, 0u16].as_ptr() as PCTSTR);
            }
            if !(*Cmd).__bindgen_anon_1.For.Params.is_null() {
                ConOutPrintf!(
                    [' ' as u16, '%' as u16, 's' as u16, 0u16].as_ptr() as PCTSTR,
                    (*Cmd).__bindgen_anon_1.For.Params,
                );
            }
            if !(*Cmd).__bindgen_anon_1.For.List.is_null()
                && SubstituteForVars(
                    (*Cmd).__bindgen_anon_1.For.List as PCTSTR,
                    core::ptr::addr_of_mut!(TempBuf) as PTSTR,
                ) != 0
            {
                ConOutPrintf!(
                    [' ' as u16, '%' as u16, '%' as u16, '%' as u16, 'c' as u16, ' ' as u16, 'i' as u16, 'n' as u16, ' ' as u16, '(' as u16, '%' as u16, 's' as u16, ')' as u16, ' ' as u16, 'd' as u16, 'o' as u16, ' ' as u16, 0u16].as_ptr() as PCTSTR,
                    (*Cmd).__bindgen_anon_1.For.Variable as i32,
                    core::ptr::addr_of_mut!(TempBuf) as *mut TCHAR,
                );
            } else {
                ConOutPrintf!(
                    [' ' as u16, '%' as u16, '%' as u16, '%' as u16, 'c' as u16, ' ' as u16, 'i' as u16, 'n' as u16, ' ' as u16, '(' as u16, '%' as u16, 's' as u16, ')' as u16, ' ' as u16, 'd' as u16, 'o' as u16, ' ' as u16, 0u16].as_ptr() as PCTSTR,
                    (*Cmd).__bindgen_anon_1.For.Variable as i32,
                    (*Cmd).__bindgen_anon_1.For.List,
                );
            }
            EchoCommand((*Cmd).Subcommands as *mut PARSED_COMMAND);

        }

        C_IF => {
            ConOutPuts!(['i' as u16, 'f' as u16, 0u16].as_ptr() as PCTSTR);
            if ((*Cmd).__bindgen_anon_1.If.Flags & (IFFLAG_IGNORECASE as u8)) != 0 {
                ConOutPuts!([' ' as u16, '/' as u16, 'I' as u16, 0u16].as_ptr() as PCTSTR);
            }
            if ((*Cmd).__bindgen_anon_1.If.Flags & (IFFLAG_NEGATE as u8)) != 0 {
                ConOutPuts!([' ' as u16, 'n' as u16, 'o' as u16, 't' as u16, 0u16].as_ptr() as PCTSTR);
            }
            if !(*Cmd).__bindgen_anon_1.If.LeftArg.is_null()
                && SubstituteForVars(
                    (*Cmd).__bindgen_anon_1.If.LeftArg as PCTSTR,
                    core::ptr::addr_of_mut!(TempBuf) as PTSTR,
                ) != 0
            {
                ConOutPrintf!(
                    [' ' as u16, '%' as u16, 's' as u16, 0u16].as_ptr() as PCTSTR,
                    core::ptr::addr_of_mut!(TempBuf) as *mut TCHAR,
                );
            }
            ConOutPrintf!(
                [' ' as u16, '%' as u16, 's' as u16, 0u16].as_ptr() as PCTSTR,
                IfOperatorString[(*Cmd).__bindgen_anon_1.If.Operator as usize],
            );
            if SubstituteForVars(
                (*Cmd).__bindgen_anon_1.If.RightArg as PCTSTR,
                core::ptr::addr_of_mut!(TempBuf) as PTSTR,
            ) != 0
            {
                ConOutPrintf!(
                    [' ' as u16, '%' as u16, 's' as u16, ' ' as u16, 0u16].as_ptr() as PCTSTR,
                    core::ptr::addr_of_mut!(TempBuf) as *mut TCHAR,
                );
            }
            Sub = (*Cmd).Subcommands as *mut PARSED_COMMAND;
            EchoCommand(Sub);
            if !(*Sub).Next.is_null() {
                ConOutPuts!([' ' as u16, 'e' as u16, 'l' as u16, 's' as u16, 'e' as u16, ' ' as u16, 0u16].as_ptr() as PCTSTR);
                EchoCommand((*Sub).Next as *mut PARSED_COMMAND);
            }

        }

        _ => {
            ASSERT!(FALSE);
        }
    }

    Redir = (*Cmd).Redirections as *mut REDIRECTION;
    while !Redir.is_null() {
        if SubstituteForVars(
            core::ptr::addr_of_mut!((*Redir).Filename) as *mut TCHAR as PCTSTR,
            core::ptr::addr_of_mut!(TempBuf) as PTSTR,
        ) != 0
        {
            ConOutPrintf!(
                ['%' as u16, 'c' as u16, '%' as u16, 's' as u16, '%' as u16, 's' as u16, ' ' as u16, 0u16].as_ptr() as PCTSTR,
                ('0' as i32) + (*Redir).Number as i32,
                core::ptr::addr_of!(RedirString[(*Redir).Mode as usize]) as *const TCHAR,
                core::ptr::addr_of_mut!(TempBuf) as *mut TCHAR,
            );
        }
        Redir = (*Redir).Next as *mut REDIRECTION;
    }
}

#[no_mangle]
pub unsafe extern "C" fn UnparseCommand(
    Cmd: *mut PARSED_COMMAND,
    mut Out: PTCHAR,
    OutEnd: PTCHAR,
) -> PTCHAR {
    let mut Sub: *mut PARSED_COMMAND = core::ptr::null_mut();
    let mut Redir: *mut REDIRECTION = core::ptr::null_mut();

    if Cmd.is_null() {
        return Out;
    }

    match (*Cmd).Type {
        C_COMMAND | C_REM => {
            if SubstituteForVars(
                core::ptr::addr_of_mut!((*Cmd).__bindgen_anon_1.Command.First) as *mut TCHAR as PCTSTR,
                core::ptr::addr_of_mut!(TempBuf) as PTSTR,
            ) == 0
            {
                return core::ptr::null_mut();
            }
            // STRING(TempBuf)
            if Out.add(wcslen(core::ptr::addr_of!(TempBuf) as *const TCHAR)) > OutEnd {
                return core::ptr::null_mut();
            }
            Out = _stpcpy(Out, core::ptr::addr_of!(TempBuf) as *const TCHAR);
            if SubstituteForVars(
                (*Cmd).__bindgen_anon_1.Command.Rest as PCTSTR,
                core::ptr::addr_of_mut!(TempBuf) as PTSTR,
            ) == 0
            {
                return core::ptr::null_mut();
            }
            // STRING(TempBuf)
            if Out.add(wcslen(core::ptr::addr_of!(TempBuf) as *const TCHAR)) > OutEnd {
                return core::ptr::null_mut();
            }
            Out = _stpcpy(Out, core::ptr::addr_of!(TempBuf) as *const TCHAR);
        }

        C_QUIET => {
            // CHAR(('@' as u16))
            if Out == OutEnd {
                return core::ptr::null_mut();
            }
            *Out = '@' as TCHAR;
            Out = Out.add(1);
            // RECURSE(Cmd->Subcommands)
            Out = UnparseCommand((*Cmd).Subcommands, Out, OutEnd);
            if Out.is_null() {
                return core::ptr::null_mut();
            }
        }

        C_BLOCK => {
            // CHAR(('(' as u16))
            if Out == OutEnd {
                return core::ptr::null_mut();
            }
            *Out = '(' as TCHAR;
            Out = Out.add(1);
            Sub = (*Cmd).Subcommands;
            while !Sub.is_null() {
                // RECURSE(Sub)
                Out = UnparseCommand(Sub, Out, OutEnd);
                if Out.is_null() {
                    return core::ptr::null_mut();
                }
                if !(*Sub).Next.is_null() {
                    // CHAR(('&' as u16))
                    if Out == OutEnd {
                        return core::ptr::null_mut();
                    }
                    *Out = '&' as TCHAR;
                    Out = Out.add(1);
                }
                Sub = (*Sub).Next;
            }
            // CHAR((')' as u16))
            if Out == OutEnd {
                return core::ptr::null_mut();
            }
            *Out = ')' as TCHAR;
            Out = Out.add(1);
        }

        C_MULTI | C_OR | C_AND | C_PIPE => {
            Sub = (*Cmd).Subcommands;
            // RECURSE(Sub)
            Out = UnparseCommand(Sub, Out, OutEnd);
            if Out.is_null() {
                return core::ptr::null_mut();
            }
            // PRINTF((&[0x0020, 0x0025, 0x0073, 0x0020, 0x0000u16]).as_ptr() /* _T " %s " */, OpString[Cmd->Type - C_OP_LOWEST])
            let fmt_op: [TCHAR; 5] = [' ' as TCHAR, '%' as TCHAR, 's' as TCHAR, ' ' as TCHAR, 0];
            let Len: UINT = _snwprintf(
                Out,
                OutEnd.offset_from(Out) as usize,
                fmt_op.as_ptr(),
                core::ptr::addr_of!(OpString[((*Cmd).Type as u32).wrapping_sub(C_OP_LOWEST as u32) as usize]) as *const TCHAR,
            ) as UINT;
            if Len > OutEnd.offset_from(Out) as UINT {
                return core::ptr::null_mut();
            }
            Out = Out.add(Len as usize);
            // RECURSE(Sub->Next)
            Out = UnparseCommand((*Sub).Next, Out, OutEnd);
            if Out.is_null() {
                return core::ptr::null_mut();
            }
        }

        C_FOR => {
            // STRING((&[0x0066, 0x006f, 0x0072, 0x0000u16]).as_ptr() /* _T "for" */)
            let s_for: [TCHAR; 4] = ['f' as TCHAR, 'o' as TCHAR, 'r' as TCHAR, 0];
            if Out.add(wcslen(s_for.as_ptr())) > OutEnd {
                return core::ptr::null_mut();
            }
            Out = _stpcpy(Out, s_for.as_ptr());

            if ((*Cmd).__bindgen_anon_1.For.Switches as u32 & FOR_DIRS as u32) != 0 {
                let s_d: [TCHAR; 4] = [' ' as TCHAR, '/' as TCHAR, 'D' as TCHAR, 0];
                if Out.add(wcslen(s_d.as_ptr())) > OutEnd {
                    return core::ptr::null_mut();
                }
                Out = _stpcpy(Out, s_d.as_ptr());
            }
            if ((*Cmd).__bindgen_anon_1.For.Switches as u32 & FOR_F as u32) != 0 {
                let s_f: [TCHAR; 4] = [' ' as TCHAR, '/' as TCHAR, 'F' as TCHAR, 0];
                if Out.add(wcslen(s_f.as_ptr())) > OutEnd {
                    return core::ptr::null_mut();
                }
                Out = _stpcpy(Out, s_f.as_ptr());
            }
            if ((*Cmd).__bindgen_anon_1.For.Switches as u32 & FOR_LOOP as u32) != 0 {
                let s_l: [TCHAR; 4] = [' ' as TCHAR, '/' as TCHAR, 'L' as TCHAR, 0];
                if Out.add(wcslen(s_l.as_ptr())) > OutEnd {
                    return core::ptr::null_mut();
                }
                Out = _stpcpy(Out, s_l.as_ptr());
            }
            if ((*Cmd).__bindgen_anon_1.For.Switches as u32 & FOR_RECURSIVE as u32) != 0 {
                let s_r: [TCHAR; 4] = [' ' as TCHAR, '/' as TCHAR, 'R' as TCHAR, 0];
                if Out.add(wcslen(s_r.as_ptr())) > OutEnd {
                    return core::ptr::null_mut();
                }
                Out = _stpcpy(Out, s_r.as_ptr());
            }
            if !(*Cmd).__bindgen_anon_1.For.Params.is_null() {
                // PRINTF((&[0x0020, 0x0025, 0x0073, 0x0000u16]).as_ptr() /* _T " %s" */, Cmd->For.Params)
                let fmt_p: [TCHAR; 4] = [' ' as TCHAR, '%' as TCHAR, 's' as TCHAR, 0];
                let LenP: UINT = _snwprintf(
                    Out,
                    OutEnd.offset_from(Out) as usize,
                    fmt_p.as_ptr(),
                    (*Cmd).__bindgen_anon_1.For.Params,
                ) as UINT;
                if LenP > OutEnd.offset_from(Out) as UINT {
                    return core::ptr::null_mut();
                }
                Out = Out.add(LenP as usize);
            }
            if !(*Cmd).__bindgen_anon_1.For.List.is_null()
                && SubstituteForVars(
                    (*Cmd).__bindgen_anon_1.For.List as PCTSTR,
                    core::ptr::addr_of_mut!(TempBuf) as PTSTR,
                ) != 0
            {
                // PRINTF((&[0x0020, 0x0025, 0x0025, 0x0025, 0x0063, 0x0020, 0x0069, 0x006e, 0x0020, 0x0028, 0x0025, 0x0073, 0x0029, 0x0020, 0x0064, 0x006f, 0x0020, 0x0000u16]).as_ptr() /* _T " %%%c in (%s) do " */, Variable, TempBuf)
                let fmt_f1: [TCHAR; 18] = [
                    ' ' as TCHAR, '%' as TCHAR, '%' as TCHAR, '%' as TCHAR, 'c' as TCHAR,
                    ' ' as TCHAR, 'i' as TCHAR, 'n' as TCHAR, ' ' as TCHAR, '(' as TCHAR,
                    '%' as TCHAR, 's' as TCHAR, ')' as TCHAR, ' ' as TCHAR, 'd' as TCHAR,
                    'o' as TCHAR, ' ' as TCHAR, 0,
                ];
                let Len1: UINT = _snwprintf(
                    Out,
                    OutEnd.offset_from(Out) as usize,
                    fmt_f1.as_ptr(),
                    (*Cmd).__bindgen_anon_1.For.Variable as i32,
                    core::ptr::addr_of_mut!(TempBuf) as *mut TCHAR,
                ) as UINT;
                if Len1 > OutEnd.offset_from(Out) as UINT {
                    return core::ptr::null_mut();
                }
                Out = Out.add(Len1 as usize);
            } else {
                // PRINTF((&[0x0020, 0x0025, 0x0025, 0x0025, 0x0063, 0x0020, 0x0069, 0x006e, 0x0020, 0x0028, 0x0025, 0x0073, 0x0029, 0x0020, 0x0064, 0x006f, 0x0020, 0x0000u16]).as_ptr() /* _T " %%%c in (%s) do " */, Variable, List)
                let fmt_f2: [TCHAR; 18] = [
                    ' ' as TCHAR, '%' as TCHAR, '%' as TCHAR, '%' as TCHAR, 'c' as TCHAR,
                    ' ' as TCHAR, 'i' as TCHAR, 'n' as TCHAR, ' ' as TCHAR, '(' as TCHAR,
                    '%' as TCHAR, 's' as TCHAR, ')' as TCHAR, ' ' as TCHAR, 'd' as TCHAR,
                    'o' as TCHAR, ' ' as TCHAR, 0,
                ];
                let Len2: UINT = _snwprintf(
                    Out,
                    OutEnd.offset_from(Out) as usize,
                    fmt_f2.as_ptr(),
                    (*Cmd).__bindgen_anon_1.For.Variable as i32,
                    (*Cmd).__bindgen_anon_1.For.List,
                ) as UINT;
                if Len2 > OutEnd.offset_from(Out) as UINT {
                    return core::ptr::null_mut();
                }
                Out = Out.add(Len2 as usize);
            }
            // RECURSE(Cmd->Subcommands)
            Out = UnparseCommand((*Cmd).Subcommands, Out, OutEnd);
            if Out.is_null() {
                return core::ptr::null_mut();
            }
        }

        C_IF => {
            // STRING((&[0x0069, 0x0066, 0x0000u16]).as_ptr() /* _T "if" */)
            let s_if: [TCHAR; 3] = ['i' as TCHAR, 'f' as TCHAR, 0];
            if Out.add(wcslen(s_if.as_ptr())) > OutEnd {
                return core::ptr::null_mut();
            }
            Out = _stpcpy(Out, s_if.as_ptr());

            if ((*Cmd).__bindgen_anon_1.If.Flags as u32 & IFFLAG_IGNORECASE as u32) != 0 {
                let s_i: [TCHAR; 4] = [' ' as TCHAR, '/' as TCHAR, 'I' as TCHAR, 0];
                if Out.add(wcslen(s_i.as_ptr())) > OutEnd {
                    return core::ptr::null_mut();
                }
                Out = _stpcpy(Out, s_i.as_ptr());
            }
            if ((*Cmd).__bindgen_anon_1.If.Flags as u32 & IFFLAG_NEGATE as u32) != 0 {
                let s_not: [TCHAR; 5] = [' ' as TCHAR, 'n' as TCHAR, 'o' as TCHAR, 't' as TCHAR, 0];
                if Out.add(wcslen(s_not.as_ptr())) > OutEnd {
                    return core::ptr::null_mut();
                }
                Out = _stpcpy(Out, s_not.as_ptr());
            }
            if !(*Cmd).__bindgen_anon_1.If.LeftArg.is_null()
                && SubstituteForVars(
                    (*Cmd).__bindgen_anon_1.If.LeftArg as PCTSTR,
                    core::ptr::addr_of_mut!(TempBuf) as PTSTR,
                ) != 0
            {
                // PRINTF((&[0x0020, 0x0025, 0x0073, 0x0000u16]).as_ptr() /* _T " %s" */, TempBuf)
                let fmt_l: [TCHAR; 4] = [' ' as TCHAR, '%' as TCHAR, 's' as TCHAR, 0];
                let LenL: UINT = _snwprintf(
                    Out,
                    OutEnd.offset_from(Out) as usize,
                    fmt_l.as_ptr(),
                    core::ptr::addr_of_mut!(TempBuf) as *mut TCHAR,
                ) as UINT;
                if LenL > OutEnd.offset_from(Out) as UINT {
                    return core::ptr::null_mut();
                }
                Out = Out.add(LenL as usize);
            }
            // PRINTF((&[0x0020, 0x0025, 0x0073, 0x0000u16]).as_ptr() /* _T " %s" */, IfOperatorString[Cmd->If.Operator])
            let fmt_o: [TCHAR; 4] = [' ' as TCHAR, '%' as TCHAR, 's' as TCHAR, 0];
            let LenO: UINT = _snwprintf(
                Out,
                OutEnd.offset_from(Out) as usize,
                fmt_o.as_ptr(),
                IfOperatorString[(*Cmd).__bindgen_anon_1.If.Operator as usize],
            ) as UINT;
            if LenO > OutEnd.offset_from(Out) as UINT {
                return core::ptr::null_mut();
            }
            Out = Out.add(LenO as usize);

            if SubstituteForVars(
                (*Cmd).__bindgen_anon_1.If.RightArg as PCTSTR,
                core::ptr::addr_of_mut!(TempBuf) as PTSTR,
            ) == 0
            {
                return core::ptr::null_mut();
            }
            // PRINTF((&[0x0020, 0x0025, 0x0073, 0x0020, 0x0000u16]).as_ptr() /* _T " %s " */, TempBuf)
            let fmt_r: [TCHAR; 5] = [' ' as TCHAR, '%' as TCHAR, 's' as TCHAR, ' ' as TCHAR, 0];
            let LenR: UINT = _snwprintf(
                Out,
                OutEnd.offset_from(Out) as usize,
                fmt_r.as_ptr(),
                core::ptr::addr_of_mut!(TempBuf) as *mut TCHAR,
            ) as UINT;
            if LenR > OutEnd.offset_from(Out) as UINT {
                return core::ptr::null_mut();
            }
            Out = Out.add(LenR as usize);

            Sub = (*Cmd).Subcommands;
            // RECURSE(Sub)
            Out = UnparseCommand(Sub, Out, OutEnd);
            if Out.is_null() {
                return core::ptr::null_mut();
            }
            if !(*Sub).Next.is_null() {
                // STRING((&[0x0020, 0x0065, 0x006c, 0x0073, 0x0065, 0x0020, 0x0000u16]).as_ptr() /* _T " else " */)
                let s_else: [TCHAR; 7] = [
                    ' ' as TCHAR, 'e' as TCHAR, 'l' as TCHAR, 's' as TCHAR,
                    'e' as TCHAR, ' ' as TCHAR, 0,
                ];
                if Out.add(wcslen(s_else.as_ptr())) > OutEnd {
                    return core::ptr::null_mut();
                }
                Out = _stpcpy(Out, s_else.as_ptr());
                // RECURSE(Sub->Next)
                Out = UnparseCommand((*Sub).Next, Out, OutEnd);
                if Out.is_null() {
                    return core::ptr::null_mut();
                }
            }
        }

        _ => {
            // ASSERT(COMMENT_TOKEN_ECHO____)
            if FALSE == 0 {
                RtlAssert(
                    b"FALSE\0".as_ptr() as PVOID,
                    b"base/shell/cmd/parser.c\0".as_ptr() as PVOID,
                    2090 as ULONG,
                    core::ptr::null_mut::<CHAR>(),
                );
            }
        }
    }

    Redir = (*Cmd).Redirections;
    while !Redir.is_null() {
        if SubstituteForVars(
            core::ptr::addr_of_mut!((*Redir).Filename) as *mut TCHAR as PCTSTR,
            core::ptr::addr_of_mut!(TempBuf) as PTSTR,
        ) == 0
        {
            return core::ptr::null_mut();
        }
        // PRINTF((&[0x0020, 0x0025, 0x0063, 0x0025, 0x0073, 0x0025, 0x0073, 0x0000u16]).as_ptr() /* _T " %c%s%s" */, ('0' as u16) + Redir->Number, RedirString[Redir->Mode], TempBuf)
        let fmt_redir: [TCHAR; 8] = [
            ' ' as TCHAR, '%' as TCHAR, 'c' as TCHAR, '%' as TCHAR,
            's' as TCHAR, '%' as TCHAR, 's' as TCHAR, 0,
        ];
        let LenRd: UINT = _snwprintf(
            Out,
            OutEnd.offset_from(Out) as usize,
            fmt_redir.as_ptr(),
            '0' as i32 + (*Redir).Number as i32,
            core::ptr::addr_of!(RedirString[(*Redir).Mode as usize]) as *const TCHAR,
            core::ptr::addr_of_mut!(TempBuf) as *mut TCHAR,
        ) as UINT;
        if LenRd > OutEnd.offset_from(Out) as UINT {
            return core::ptr::null_mut();
        }
        Out = Out.add(LenRd as usize);
        Redir = (*Redir).Next;
    }
    return Out;
}
// enforce_import_abi: moved 1 import decl(s) into the extern block their
// exporting DLL's .spec requires: RtlAssert->system.
// The CRT is cdecl, Win32 is stdcall, and the C source does not say which is which —
// a wrong guess is a silent i386 stack walk on every call. The .spec is the truth.
extern "system" {
    fn RtlAssert(assertion: PVOID, file: PVOID, line: ULONG, message: LPCSTR) -> !;
}
