use crate::cmd::console::ConOutResPaging;
use crate::cmd::cmd::DoDelayedExpansion;
use crate::cmd::misc::StripQuotes;
use crate::cmd::misc::_stpcpy;
use crate::cmd::error::error_bad_command;
use crate::cmd::error::error_out_of_memory;
use crate::cmd::error::error_sfile_not_found;
use crate::cmd::error::error_syntax;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut fc: PFOR_CONTEXT = ::core::ptr::null::<_FOR_CONTEXT>() as *mut _FOR_CONTEXT;

// MSCMD_FOR_QUIRKS: empty object-like macro (no value), no Rust constant emitted.
// RunInstance, Exiting, ExitingOrGoto: function-like macros, inlined by the functions that use them.
#[no_mangle]
pub unsafe extern "C" fn cmd_for(mut param: LPTSTR) -> INT {
    if _tcsncmp(param, (&[0x002f, 0x003f, 0x0000u16]).as_ptr() /* _T "/?" */, 2) == 0
    {
        ConOutResPaging((TRUE) as BOOL, STRING_FOR_HELP1);
        return 0;
    }

    ParseErrorEx(param);
    return 1;
}

unsafe extern "C" fn GetNextElement(pStart: *mut *mut TCHAR, pEnd: *mut *mut TCHAR) -> BOOL {
    let mut p: *mut TCHAR = *pEnd;
    let mut InQuotes: BOOL = (FALSE) as i32;
    while _istspace!(*p) != 0 {
        p = p.offset(1);
    }
    if *p == 0 {
        return (FALSE) as BOOL;
    }
    *pStart = p;
    while *p != 0 && (InQuotes != 0 || _istspace!(*p) == 0) {
        let fresh31 = p;
        p = p.offset(1);
        InQuotes ^= (*fresh31 == ('"' as u16)) as BOOL;
    }
    *pEnd = p;
    return (TRUE) as BOOL;
}

unsafe extern "C" fn ReadFileContents(mut InputFile: *mut FILE, mut Buffer: *mut TCHAR) -> LPTSTR {
    let mut Len: SIZE_T = 0 as SIZE_T;
    let mut AllocLen: SIZE_T = 1000 as SIZE_T;

    let mut Contents: LPTSTR = cmd_alloc!(
        AllocLen.wrapping_mul(core::mem::size_of::<TCHAR>() as SIZE_T),
    ) as LPTSTR;
    if Contents.is_null() {
        return core::ptr::null_mut();
    }

    while !_fgetts(Buffer, CMDLINE_LENGTH as INT, InputFile).is_null() {
        let mut CharsRead: ULONG_PTR = _tcslen(Buffer) as ULONG_PTR;
        while (Len as ULONG_PTR).wrapping_add(CharsRead) >= AllocLen as ULONG_PTR {
            let mut OldContents: LPTSTR = Contents;
            Contents = cmd_realloc!(
                Contents,
                {
                    AllocLen = AllocLen.wrapping_mul(2 as SIZE_T);
                    AllocLen
                }
                .wrapping_mul(core::mem::size_of::<TCHAR>() as SIZE_T),
            ) as LPTSTR;
            if Contents.is_null() {
                cmd_free!(OldContents);
                return core::ptr::null_mut();
            }
        }
        _tcscpy(Contents.add(Len as usize), Buffer);
        Len = (Len as ULONG_PTR).wrapping_add(CharsRead) as SIZE_T;
    }

    *Contents.add(Len as usize) = ('\0' as TCHAR);
    return Contents;
}

unsafe extern "C" fn ForF(Cmd: *mut PARSED_COMMAND, List: LPTSTR, Buffer: *mut TCHAR) -> INT {
    let mut DelimsLit: [TCHAR; 3] = [' ' as TCHAR, '\t' as TCHAR, 0];
    let mut DelimsKw: [TCHAR; 8] = [
        'd' as TCHAR, 'e' as TCHAR, 'l' as TCHAR, 'i' as TCHAR,
        'm' as TCHAR, 's' as TCHAR, '=' as TCHAR, 0,
    ];
    let mut SpaceLit: [TCHAR; 2] = [' ' as TCHAR, 0];
    let mut EolKw: [TCHAR; 5] = ['e' as TCHAR, 'o' as TCHAR, 'l' as TCHAR, '=' as TCHAR, 0];
    let mut SkipKw: [TCHAR; 6] = [
        's' as TCHAR, 'k' as TCHAR, 'i' as TCHAR, 'p' as TCHAR, '=' as TCHAR, 0,
    ];
    let mut TokensKw: [TCHAR; 8] = [
        't' as TCHAR, 'o' as TCHAR, 'k' as TCHAR, 'e' as TCHAR,
        'n' as TCHAR, 's' as TCHAR, '=' as TCHAR, 0,
    ];
    let mut UsebackKw: [TCHAR; 8] = [
        'u' as TCHAR, 's' as TCHAR, 'e' as TCHAR, 'b' as TCHAR,
        'a' as TCHAR, 'c' as TCHAR, 'k' as TCHAR, 0,
    ];
    let mut ModeR: [TCHAR; 2] = ['r' as TCHAR, 0];

    let mut Delims: LPTSTR = DelimsLit.as_mut_ptr();
    let mut DelimsEndPtr: PTCHAR = core::ptr::null_mut();
    let mut DelimsEndChr: TCHAR = '\0' as TCHAR;
    let mut Eol: TCHAR = ';' as TCHAR;
    let mut SkipLines: INT = 0;
    let mut TokensMask: DWORD = (1 << 1);
    let mut NumTokens: DWORD = 1;
    let mut RemainderVar: DWORD = 0;
    let mut StringQuote: TCHAR = '"' as TCHAR;
    let mut CommandQuote: TCHAR = '\'' as TCHAR;
    let mut Variables: [LPTSTR; 32] =
        [core::ptr::null_mut(); 32];
    let mut Start: PTCHAR = core::ptr::null_mut();
    let mut End: PTCHAR = core::ptr::null_mut();
    let mut Ret: INT = 0;
    let mut SingleElementJump: bool = false;

    if !(*Cmd).__bindgen_anon_1.For.Params.is_null() {
        let mut Quote: TCHAR = 0;
        let mut Param: PTCHAR = (*Cmd).__bindgen_anon_1.For.Params as PTCHAR;
        if *Param == '"' as TCHAR || *Param == '\'' as TCHAR {
            let FreshQ: PTCHAR = Param;
            Param = Param.offset(1);
            Quote = *FreshQ;
        }

        while *Param != 0 && *Param != Quote {
            if *Param <= (' ' as TCHAR) {
                Param = Param.offset(1);
            } else if _tcsnicmp(Param, DelimsKw.as_mut_ptr(), 7)
                == 0
            {
                Param = Param.offset(7 as isize);
                Delims = Param;
                DelimsEndPtr = core::ptr::null_mut();
                while *Param != 0 && *Param != Quote {
                    if *Param == (' ' as TCHAR) {
                        let FirstSpace: PTCHAR = Param;
                        Param = Param.offset(
                            _tcsspn(Param, SpaceLit.as_mut_ptr()) as isize,
                        );
                        if *Param != 0 && *Param != Quote {
                            DelimsEndPtr = FirstSpace;
                        }
                        break;
                    }
                    Param = Param.offset(1);
                }
                if *Param == Quote {
                    DelimsEndPtr = Param;
                    Param = Param.offset(1);
                }
            } else if _tcsnicmp(Param, EolKw.as_mut_ptr(), 4)
                == 0
            {
                Param = Param.offset(4 as isize);
                Eol = *Param;
                if Eol != ('\0' as TCHAR) {
                    Param = Param.offset(1);
                }
            } else if _tcsnicmp(Param, SkipKw.as_mut_ptr(), 5)
                == 0
            {
                SkipLines = _tcstol(
                    Param.offset(5 as isize),
                    core::ptr::addr_of_mut!(Param),
                    0,
                ) as INT;
                if SkipLines <= 0 {
                    error_syntax(Param);
                    return 1;
                }
            } else if _tcsnicmp(Param, TokensKw.as_mut_ptr(), 7)
                == 0
            {
                let mut NumToksInSpec: DWORD = 0;
                Param = Param.offset(7 as isize);
                TokensMask = 0;
                NumToksInSpec = 0;

                while *Param != 0 && *Param != Quote && *Param != ('*' as TCHAR) {
                    let First: INT = _tcstol(
                        Param,
                        core::ptr::addr_of_mut!(Param),
                        0,
                    ) as INT;
                    let mut Last: INT = First;
                    if First < 1 {
                        error_syntax(Param);
                        return 1;
                    }
                    if *Param == ('-' as TCHAR) {
                        Last = _tcstol(
                            Param.offset(1 as isize),
                            core::ptr::addr_of_mut!(Param),
                            0,
                        ) as INT;
                        if Last < 1 {
                            error_syntax(Param);
                            return 1;
                        }
                    }
                    if (First <= Last) && (Last <= 31) {
                        TokensMask |= ((2 << Last)
                            - (1 << First)) as DWORD;
                        NumToksInSpec = NumToksInSpec.wrapping_add(
                            (Last - First + 1 as INT) as DWORD,
                        );
                    }

                    if *Param != (',' as TCHAR) {
                        break;
                    }
                    Param = Param.offset(1);
                }
                if *Param == ('*' as TCHAR) {
                    NumToksInSpec = NumToksInSpec.wrapping_add(1);
                    RemainderVar = NumToksInSpec;
                    Param = Param.offset(1);
                }
                NumTokens = max!(NumTokens, NumToksInSpec);
            } else if _tcsnicmp(Param, UsebackKw.as_mut_ptr(), 7)
                == 0
            {
                Param = Param.offset(7 as isize);
                StringQuote = '\'' as TCHAR;
                CommandQuote = '`' as TCHAR;
                if _totlower(*Param) as TCHAR == ('q' as TCHAR) {
                    Param = Param.offset(1);
                }
            } else {
                error_syntax(Param);
                return 1;
            }
        }
    }

    (*fc).varcount = NumTokens;
    if (NumTokens as usize) <= (ARRAYSIZE!(Variables) as usize) {
        (*fc).values = Variables.as_mut_ptr();
    } else {
        (*fc).values = cmd_alloc!(
            ((*fc).varcount as usize) * core::mem::size_of::<LPTSTR>(),
        ) as *mut LPTSTR;
        if (*fc).values.is_null() {
            error_out_of_memory();
            return 1;
        }
    }

    if *List == StringQuote || *List == CommandQuote {
        Start = List;
        End = List.offset(_tcslen(List) as isize);
        SingleElementJump = true;
    } else {
        End = List;
    }

    'quit: {
        while SingleElementJump
            || (ExitingOrGoto!(Cmd) == false
                && GetNextElement(
                    core::ptr::addr_of_mut!(Start),
                    core::ptr::addr_of_mut!(End),
                ) != 0)
        {
            SingleElementJump = false;

            let mut InputFile: *mut FILE = core::ptr::null_mut();
            let mut FullInput: LPTSTR = core::ptr::null_mut();
            let mut In: LPTSTR = core::ptr::null_mut();
            let mut NextLine: LPTSTR = core::ptr::null_mut();
            let mut Skip: INT = 0;

            if *Start == StringQuote
                && *End.offset(-(1 as isize)) == StringQuote
            {
                *End.offset(-(1 as isize)) = '\0' as TCHAR;
                FullInput = cmd_dup!(Start.offset(1 as isize));
            } else if *Start == CommandQuote
                && *End.offset(-(1 as isize)) == CommandQuote
            {
                *End.offset(-(1 as isize)) = '\0' as TCHAR;
                InputFile = _tpopen(
                    Start.offset(1 as isize),
                    ModeR.as_mut_ptr(),
                );
                if InputFile.is_null() {
                    error_bad_command(Start.offset(1 as isize));
                    Ret = 1;
                    break 'quit;
                }
                FullInput = ReadFileContents(InputFile, Buffer);
                _pclose(InputFile);
            } else {
                let mut Temp: TCHAR = *End;
                *End = '\0' as TCHAR;
                StripQuotes(Start);
                InputFile = _tfopen(Start, ModeR.as_mut_ptr());
                *End = Temp;
                if InputFile.is_null() {
                    error_sfile_not_found(Start);
                    Ret = 1;
                    break 'quit;
                }
                FullInput = ReadFileContents(InputFile, Buffer);
                fclose(InputFile);
            }

            if FullInput.is_null() {
                error_out_of_memory();
                Ret = 1;
                break 'quit;
            }

            if !DelimsEndPtr.is_null() {
                DelimsEndChr = *DelimsEndPtr;
                *DelimsEndPtr = '\0' as TCHAR;
            }

            In = FullInput;
            Skip = SkipLines;
            loop {
                if !(ExitingOrGoto!(Cmd) == false && !In.is_null()) {
                    break;
                }
                'body: {
                    let mut RemainingTokens: DWORD = TokensMask;
                    let mut CurVar: *mut LPTSTR = (*fc).values;

                    ZeroMemory!(
                        (*fc).values as *mut core::ffi::c_void,
                        ((*fc).varcount as usize) * core::mem::size_of::<LPTSTR>(),
                    );
                    NumTokens = (*fc).varcount;

                    NextLine = _tcschr(In, '\n' as TCHAR);
                    if !NextLine.is_null() {
                        let FreshN: LPTSTR = NextLine;
                        NextLine = NextLine.offset(1);
                        *FreshN = '\0' as TCHAR;
                    }

                    Skip -= 1;
                    if Skip >= 0 {
                        break 'body;
                    }

                    In = In.offset(_tcsspn(In, Delims) as isize);
                    if *In == Eol {
                        break 'body;
                    }

                    while *In != 0 && {
                        RemainingTokens >>= 1;
                        RemainingTokens != 0
                    } {
                        if (RemainingTokens & 1) != 0 {
                            NumTokens = NumTokens.wrapping_sub(1);
                            let FreshC: *mut LPTSTR = CurVar;
                            CurVar = CurVar.offset(1);
                            *FreshC = In;
                        }
                        In = In.offset(_tcscspn(In, Delims) as isize);
                        if *In != 0 {
                            let FreshT: LPTSTR = In;
                            In = In.offset(1);
                            *FreshT = '\0' as TCHAR;
                            In = In.offset(_tcsspn(In, Delims) as isize);
                        }
                    }

                    if *In != 0 {
                        if RemainderVar != 0
                            && (*fc)
                                .varcount
                                .wrapping_sub(NumTokens)
                                .wrapping_add(1 as DWORD)
                                == RemainderVar
                        {
                            *CurVar = In;
                        }
                    }

                    if !(*(*fc).values.offset(0 as isize)).is_null() {
                        Ret = RunInstance!(Cmd);
                    }
                }
                In = NextLine;
            }

            if !DelimsEndPtr.is_null() {
                *DelimsEndPtr = DelimsEndChr;
            }

            cmd_free!(FullInput);
        }
    }

    if !(*fc).values.is_null() && (*fc).values != Variables.as_mut_ptr() {
        cmd_free!((*fc).values);
    }
    (*fc).values = core::ptr::null_mut();

    return Ret;
}

unsafe extern "C" fn ForLoop(Cmd: *mut PARSED_COMMAND, List: LPTSTR, Buffer: *mut TCHAR) -> INT {
    // for.c's function-local `enum { START, STEP, END };`
    const START: usize = 0;
    const STEP: usize = 1;
    const END: usize = 2;
    let mut params: [INT; 3] = [0, 0, 0];
    let mut i: INT;
    let mut Ret: INT = 0;
    let mut Start: *mut TCHAR = core::ptr::null_mut();
    let mut End: *mut TCHAR = List;

    i = 0;
    while i < 3 && GetNextElement(core::ptr::addr_of_mut!(Start), core::ptr::addr_of_mut!(End)) != 0 {
        params[i as usize] = _tcstol(Start, core::ptr::null_mut(), 0) as INT;
        i += 1;
    }

    i = params[START as usize];
    /*
     * Windows' CMD compatibility:
     * Contrary to the other FOR-loops, FOR /L does not check
     * whether a GOTO has been done, and will continue to loop.
     */
    while !Exiting!(Cmd)
        && (if params[STEP as usize] >= 0 {
            (i <= params[END as usize]) as INT
        } else {
            (i >= params[END as usize]) as INT
        }) != 0
    {
        _itot(i, Buffer, 10);
        Ret = RunInstance!(Cmd);
        i += params[STEP as usize];
    }

    return Ret;
}

unsafe extern "C" fn ForDir(
    Cmd: *mut PARSED_COMMAND,
    List: LPTSTR,
    Buffer: *mut TCHAR,
    BufPos: *mut TCHAR,
) -> INT {
    let mut Ret: INT = 0;
    let mut Start: *mut TCHAR = core::ptr::null_mut();
    let mut End: *mut TCHAR = List as *mut TCHAR;
    while ExitingOrGoto!(Cmd) == false
        && GetNextElement(core::ptr::addr_of_mut!(Start), core::ptr::addr_of_mut!(End)) != 0
    {
        if BufPos.offset(End.offset_from(Start) as isize)
            > Buffer.offset(CMDLINE_LENGTH as isize)
        {
            continue;
        }
        memcpy(
            BufPos as *mut core::ffi::c_void,
            Start as *const core::ffi::c_void,
            (End.offset_from(Start) as usize) * (core::mem::size_of::<TCHAR>() as usize),
        );
        *BufPos.offset(End.offset_from(Start) as isize) = ('\0' as u16);

        if !_tcschr(BufPos, ('?' as u16)).is_null()
            || !_tcschr(BufPos, ('*' as u16)).is_null()
        {
            let mut w32fd: WIN32_FIND_DATA = core::mem::zeroed();
            let mut hFind: HANDLE = core::ptr::null_mut();
            let mut FilePart: *mut TCHAR = core::ptr::null_mut();

            StripQuotes(BufPos);
            hFind = FindFirstFile(Buffer, core::ptr::addr_of_mut!(w32fd));
            if hFind == INVALID_HANDLE_VALUE {
                continue;
            }
            FilePart = _tcsrchr(BufPos, ('\\' as u16));
            FilePart = if !FilePart.is_null() {
                FilePart.offset(1 as isize)
            } else {
                BufPos
            };
            loop {
                'body: {
                    if (w32fd.dwFileAttributes & FILE_ATTRIBUTE_HIDDEN) != 0 {
                        break 'body;
                    }
                    if ((w32fd.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) == 0)
                        != (((*Cmd).__bindgen_anon_1.For.Switches & (FOR_DIRS as u8)) == 0)
                    {
                        break 'body;
                    }
                    if _tcscmp(
                        core::ptr::addr_of_mut!(w32fd.cFileName) as *const TCHAR,
                        (&[0x002e, 0x0000u16]).as_ptr() /* _T "." */,
                    ) == 0
                        || _tcscmp(
                            core::ptr::addr_of_mut!(w32fd.cFileName) as *const TCHAR,
                            (&[0x002e, 0x002e, 0x0000u16]).as_ptr() /* _T ".." */,
                        ) == 0
                    {
                        break 'body;
                    }
                    _tcscpy(
                        FilePart,
                        core::ptr::addr_of_mut!(w32fd.cFileName) as *const TCHAR,
                    );
                    Ret = RunInstance!(Cmd);
                }
                if !(ExitingOrGoto!(Cmd) == false
                    && FindNextFile(hFind, core::ptr::addr_of_mut!(w32fd)) != 0)
                {
                    break;
                }
            }
            FindClose(hFind);
        } else {
            Ret = RunInstance!(Cmd);
        }
    }
    return Ret;
}

unsafe extern "C" fn ForRecursive(
    Cmd: *mut PARSED_COMMAND,
    List: LPTSTR,
    Buffer: *mut TCHAR,
    BufPos: *mut TCHAR,
) -> INT {
    let mut Ret: INT = 0;
    let mut hFind: HANDLE = core::ptr::null_mut();
    let mut w32fd: WIN32_FIND_DATA = core::mem::zeroed();
    let mut BufPos = BufPos;

    if *BufPos.offset(-(1 as isize)) != ('\\' as TCHAR) {
        let fresh = BufPos;
        BufPos = BufPos.add(1);
        *fresh = '\\' as TCHAR;
        *BufPos = '\0' as TCHAR;
    }

    Ret = ForDir(Cmd, List, Buffer, BufPos);

    /* NOTE (We don't apply Windows' CMD compatibility here):
     * Windows' CMD does not check whether a GOTO has been done,
     * and will continue to loop. */
    if ExitingOrGoto!(Cmd) != false {
        return Ret;
    }

    let star: [TCHAR; 2] = ['*' as TCHAR, 0];
    _tcscpy(BufPos, star.as_ptr());
    hFind = FindFirstFile(Buffer, core::ptr::addr_of_mut!(w32fd));
    if hFind == INVALID_HANDLE_VALUE {
        return Ret;
    }
    loop {
        'body: {
            if (w32fd.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) == 0 {
                break 'body;
            }
            let dot: [TCHAR; 2] = ['.' as TCHAR, 0];
            let dotdot: [TCHAR; 3] = ['.' as TCHAR, '.' as TCHAR, 0];
            if _tcscmp(
                core::ptr::addr_of_mut!(w32fd.cFileName) as *mut TCHAR,
                dot.as_ptr(),
            ) == 0
                || _tcscmp(
                    core::ptr::addr_of_mut!(w32fd.cFileName) as *mut TCHAR,
                    dotdot.as_ptr(),
                ) == 0
            {
                break 'body;
            }
            Ret = ForRecursive(
                Cmd,
                List,
                Buffer,
                _stpcpy(
                    BufPos,
                    core::ptr::addr_of_mut!(w32fd.cFileName) as *mut TCHAR,
                ),
            );
        }
        /* NOTE (We don't apply Windows' CMD compatibility here):
         * Windows' CMD does not check whether a GOTO has been done,
         * and will continue to loop. */
        if !(ExitingOrGoto!(Cmd) == false
            && FindNextFile(hFind, core::ptr::addr_of_mut!(w32fd)) != 0)
        {
            break;
        }
    }
    FindClose(hFind);

    return Ret;
}
#[no_mangle]
pub unsafe extern "C" fn ExecuteFor(mut Cmd: *mut PARSED_COMMAND) -> INT {
    let mut Ret: INT = 0;
    let mut List: LPTSTR = core::ptr::null_mut();
    let mut lpNew: PFOR_CONTEXT = core::ptr::null_mut();
    let mut Buffer: [TCHAR; (CMDLINE_LENGTH) as _] = [0; (CMDLINE_LENGTH) as _];
    let mut BufferPtr: LPTSTR = core::ptr::addr_of_mut!(Buffer) as LPTSTR;

    List = DoDelayedExpansion((*Cmd).__bindgen_anon_1.For.List) as LPTSTR;
    if List.is_null() {
        return 1;
    }

    /* Create our FOR context */
    lpNew = cmd_alloc!(core::mem::size_of::<FOR_CONTEXT>() as usize) as PFOR_CONTEXT;
    if lpNew.is_null() {

        cmd_free!(List);
        return 1;
    }
    (*lpNew).prev = fc;
    (*lpNew).firstvar = (*Cmd).__bindgen_anon_1.For.Variable;
    (*lpNew).varcount = 1;
    (*lpNew).values = core::ptr::addr_of_mut!(BufferPtr);

    (*Cmd).__bindgen_anon_1.For.Context = lpNew;
    fc = lpNew;

    /* Run the extended FOR syntax only if extensions are enabled */
    if bEnableExtensions != 0 {
        if (*Cmd).__bindgen_anon_1.For.Switches & (FOR_F as u8) != 0 {
            Ret = ForF(Cmd, List, core::ptr::addr_of_mut!(Buffer) as LPTSTR);
        } else if (*Cmd).__bindgen_anon_1.For.Switches & (FOR_LOOP as u8) != 0 {
            Ret = ForLoop(Cmd, List, core::ptr::addr_of_mut!(Buffer) as LPTSTR);
        } else if (*Cmd).__bindgen_anon_1.For.Switches & (FOR_RECURSIVE as u8) != 0 {
            let Len: DWORD = GetFullPathName(
                if !(*Cmd).__bindgen_anon_1.For.Params.is_null() {
                    (*Cmd).__bindgen_anon_1.For.Params
                } else {
                    (&[0x002e, 0x0000u16]).as_ptr() /* _T "." */
                },
                MAX_PATH,
                core::ptr::addr_of_mut!(Buffer) as LPTSTR,
                core::ptr::null_mut(),
            );
            Ret = ForRecursive(
                Cmd,
                List,
                core::ptr::addr_of_mut!(Buffer) as LPTSTR,
                (core::ptr::addr_of_mut!(Buffer) as LPTSTR).add(Len as usize),
            );
        } else {
            Ret = ForDir(
                Cmd,
                List,
                core::ptr::addr_of_mut!(Buffer) as LPTSTR,
                core::ptr::addr_of_mut!(Buffer) as LPTSTR,
            );
        }
    } else {
        Ret = ForDir(
            Cmd,
            List,
            core::ptr::addr_of_mut!(Buffer) as LPTSTR,
            core::ptr::addr_of_mut!(Buffer) as LPTSTR,
        );
    }

    /* Remove our context, unless someone already did that */
    if fc == lpNew {
        fc = (*lpNew).prev;
    }

    cmd_free!(lpNew);
    cmd_free!(List);
    return Ret;
}
