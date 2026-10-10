use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// No file-scope declarations: the preamble contains only the `CommandColor` function (under `INCLUDE_CMD_COLOR`), which is excluded from this pass. No constants, typedefs, enums, or static tables to emit.
#[no_mangle]
pub unsafe extern "C" fn CommandColor(mut rest: LPTSTR) -> INT {
    let mut w_color: WORD = 0x00 as WORD;
    let mut SlashQ: [u16; 3] = ['/' as u16, '?' as u16, 0];

    /* The user asked for help */
    if _tcsncmp(rest, SlashQ.as_ptr(), 2 as usize) == 0 as INT {
        ConOutResPaging(TRUE as BOOL, STRING_COLOR_HELP1 as UINT);
        return 0 as INT;
    }

    /* Let's prepare %ERRORLEVEL% */
    nErrorLevel = 0 as INT;

    /* No parameter: Set the default colors */
    if *rest.offset(0 as isize) == '\0' as u16 {
        ConSetScreenColor(ConStreamGetOSHandle(StdOut), wDefColor, TRUE as BOOL);
        return 0 as INT;
    }

    /* The parameter is just one character: Set color text */
    if _tcslen(rest) == 1 as usize {
        if (*rest.offset(0 as isize) >= '0' as u16)
            && (*rest.offset(0 as isize) <= '9' as u16)
        {
            w_color = _ttoi(rest) as WORD;
        } else if (*rest.offset(0 as isize) >= 'a' as u16)
            && (*rest.offset(0 as isize) <= 'f' as u16)
        {
            w_color = (*rest.offset(0 as isize) as INT
                + 10 as INT
                - 'a' as INT) as WORD;
        } else if (*rest.offset(0 as isize) >= 'A' as u16)
            && (*rest.offset(0 as isize) <= 'F' as u16)
        {
            w_color = (*rest.offset(0 as isize) as INT
                + 10 as INT
                - 'A' as INT) as WORD;
        } else {
            /* Invalid character */
            ConOutResPaging(TRUE as BOOL, STRING_COLOR_HELP1 as UINT);
            nErrorLevel = 1 as INT;
            return 1 as INT;
        }
    }
    /* Color string: advanced choice: two-digits, "Color ON Color", "Foreground ON Background" */
    else if StringToColor(core::ptr::addr_of_mut!(w_color), core::ptr::addr_of_mut!(rest)) == FALSE as BOOL {
        /* Invalid color string */
        ConOutResPaging(TRUE as BOOL, STRING_COLOR_HELP1 as UINT);
        nErrorLevel = 1 as INT;
        return 1 as INT;
    }

    /* Trace (TRACE macro): Color %02x\n */
    CmdTrace(
        1 as INT,
        b"base/shell/cmd/color.c\0".as_ptr() as LPCSTR,
        84 as INT,
        b"CommandColor\0".as_ptr() as LPCSTR,
        b"Color %02x\n\0".as_ptr() as LPCSTR,
        w_color as INT,
    );

    let mut SlashF: [u16; 4] = ['/' as u16, '-' as u16, 'F' as u16, 0];
    let mut Slashf: [u16; 4] = ['/' as u16, '-' as u16, 'f' as u16, 0];

    /*
     * Set the chosen color. Use also the following advanced flag:
     * /-F to avoid changing already buffered foreground/background.
     */
    if ConSetScreenColor(
        ConStreamGetOSHandle(StdOut),
        w_color,
        (_tcsstr(rest, SlashF.as_ptr()).is_null()
            && _tcsstr(rest, Slashf.as_ptr()).is_null()) as BOOL,
    ) == FALSE as BOOL
    {
        /* Failed because foreground and background colors were the same */
        ConErrResPuts!(STRING_COLOR_ERROR1 as UINT);
        nErrorLevel = 1 as INT;
        return 1 as INT;
    }

    /* Return success */
    return 0 as INT;
}
