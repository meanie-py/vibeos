use crate::regedit::regproc::delete_registry_key;
use crate::regedit::regproc::export_registry_key;
use crate::regedit::regproc::import_registry_file;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_28276425D458: [u16; 2] = [0x002Du16, 0x0000];
pub static RUSTCONV_WSTR_76CFE90FF2FC: [u16; 3] = [0x0072u16, 0x0062, 0x0000];
pub static RUSTCONV_WSTR_E89B9A83645C: [u16; 8] = [0x0072u16, 0x0065, 0x0067, 0x0065, 0x0064, 0x0069, 0x0074, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// WINE_DEFAULT_DEBUG_CHANNEL(regedit): debug channel declaration, handled by the DPRINT*! macros in the crate.

pub type REGEDIT_ACTION = i32;
pub const ACTION_ADD: REGEDIT_ACTION = 0;
pub const ACTION_EXPORT: REGEDIT_ACTION = 1;
pub const ACTION_DELETE: REGEDIT_ACTION = 2;

unsafe extern "C" fn output_writeconsole(str: *const WCHAR, wlen: DWORD) {
    /* This is win32gui application, don't ever try writing to console.
     * For the console version we have a separate reg.exe application. */
    MessageBoxW(
        core::ptr::null_mut(),
        str,
        core::ptr::null(),
        MB_ICONERROR as UINT,
    );
    let _ = wlen;
}

unsafe extern "C" fn output_formatstring(mut fmt: *const WCHAR, mut va_args: va_list) {
    let mut str: *mut WCHAR = core::ptr::null_mut::<WCHAR>();
    let mut len: DWORD = 0;
    SetLastError(NO_ERROR as DWORD);
    len = FormatMessageW(
        (FORMAT_MESSAGE_FROM_STRING | FORMAT_MESSAGE_ALLOCATE_BUFFER) as DWORD,
        fmt as LPCVOID,
        0 as DWORD,
        0 as DWORD,
        &raw mut str as LPWSTR,
        0 as DWORD,
        &raw mut va_args,
    );
    if len == 0 as DWORD
        && GetLastError() != NO_ERROR as DWORD
    {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
            != 0
        {
            WINE_FIXME!(
                b"Could not format string: le=%lu, fmt=%s\n\0" as *const u8
                    as *const ::core::ffi::c_char,
                GetLastError(),
                wine_dbgstr_w(fmt),
            );
        }
        return;
    }
    output_writeconsole(str, len);
    LocalFree(str as HLOCAL);
}
#[no_mangle]
pub unsafe extern "C" fn output_message_v(mut id: ::core::ffi::c_uint, ap_list: *mut ::core::ffi::c_void) {
    let mut fmt: [WCHAR; 1536] = [0 as WCHAR; 1536];
    let mut va_args: va_list = *(ap_list as *mut va_list);
    if LoadStringW(
        GetModuleHandleW(core::ptr::null()) as HINSTANCE,
        id as UINT,
        core::ptr::addr_of_mut!(fmt) as LPWSTR,
        ARRAY_SIZE!(fmt) as ::core::ffi::c_int,
    ) == 0
    {
        WINE_FIXME!(
            b"LoadString failed with %ld\n\0" as *const u8 as *const ::core::ffi::c_char,
            GetLastError(),
        );
        return;
    }

    output_formatstring(core::ptr::addr_of_mut!(fmt) as *mut WCHAR, va_args);

}


#[no_mangle]
pub unsafe extern "C" fn error_exit_v(mut id: ::core::ffi::c_uint, ap_list: *mut ::core::ffi::c_void) {
    let mut fmt: [WCHAR; 1536] = [0; 1536];
    let mut va_args: va_list = *(ap_list as *mut va_list);

    if LoadStringW(
        GetModuleHandleW(core::ptr::null_mut()) as HINSTANCE,
        id as UINT,
        core::ptr::addr_of_mut!(fmt) as LPWSTR,
        ARRAY_SIZE!(fmt) as ::core::ffi::c_int,
    ) == 0
    {
        return;
    }

    output_formatstring(core::ptr::addr_of_mut!(fmt) as *mut WCHAR, va_args);


    exit(0); /* regedit.exe always terminates with error code zero */
}



unsafe extern "C" fn PerformRegAction(action: REGEDIT_ACTION, argv: *mut *mut WCHAR, i: *mut c_int, silent: BOOL) {
    match (action as c_uint) as i32 {
        ACTION_ADD => {
            let mut filename: *mut WCHAR = *argv.offset(*i as isize);
            let mut realname: *mut WCHAR = core::ptr::null_mut();
            let mut reg_file: *mut FILE = core::ptr::null_mut();

            /* Request import confirmation */
            if silent == 0 {
                let mut szText: [WCHAR; 512] = [0; 512];
                let mut choice: c_int = 0;
                let mut mbType: UINT = MB_YESNO as UINT;

                LoadStringW(hInst, IDS_IMPORT_PROMPT as UINT, core::ptr::addr_of_mut!(szText) as LPWSTR, ARRAY_SIZE!(szText) as c_int);

                if !(*argv.offset((*i + 1) as isize)).is_null() {
                    /* Enable three buttons if there's another file coming */
                    mbType = MB_YESNOCANCEL as UINT;
                }

                choice = InfoMessageBox(
                    core::ptr::null_mut(),
                    mbType | (MB_ICONQUESTION as UINT),
                    core::ptr::addr_of_mut!(szTitle) as *mut WCHAR as LPCWSTR,
                    core::ptr::addr_of_mut!(szText) as *mut WCHAR as LPCWSTR,
                    filename,
                );
                match (choice) as u32 {
                    IDNO => {
                        return;
                    }
                    IDCANCEL => {
                        /* The cancel case is useful if the user is importing more than one registry file
                         * at a time, and wants to back out anytime during the import process. This way, the
                         * user doesn't have to resort to ending the regedit process abruptly just to cancel
                         * the operation.
                         * To achieve this, we skip all further command line arguments.
                         */
                        *i = (INT_MAX - 1) as i32;
                        return;
                    }
                    _ => {
                        // default: break
                    }
                }
            }
            if lstrcmpW(filename as LPCWSTR, RUSTCONV_WSTR_28276425D458.as_ptr()) == 0 {
                reg_file = __iob_func().add(0);
            } else {
                let mut size: c_int = 0;

                size = SearchPathW(
                    core::ptr::null(),
                    filename as LPCWSTR,
                    core::ptr::null(),
                    0 as DWORD,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                ) as c_int;
                if size > 0 {
                    realname = malloc(
                        (size as usize).wrapping_mul(core::mem::size_of::<WCHAR>()),
                    ) as *mut WCHAR;
                    size = SearchPathW(
                        core::ptr::null(),
                        filename as LPCWSTR,
                        core::ptr::null(),
                        size as DWORD,
                        realname as LPWSTR,
                        core::ptr::null_mut(),
                    ) as c_int;
                }
                if size == 0 {
                    output_message(STRING_FILE_NOT_FOUND, filename);
                    free(realname as *mut core::ffi::c_void);
                    return;
                }
                reg_file = (_wfopen(realname as *const WCHAR, RUSTCONV_WSTR_76CFE90FF2FC.as_ptr())).cast();
                if reg_file.is_null() {
                    _wperror(RUSTCONV_WSTR_E89B9A83645C.as_ptr());
                    output_message(STRING_CANNOT_OPEN_FILE, filename);
                    free(realname as *mut core::ffi::c_void);
                    return;
                }
            }
            import_registry_file(reg_file);
            if !realname.is_null() {
                free(realname as *mut core::ffi::c_void);
                fclose(reg_file);
            }
            // break
        }
        ACTION_DELETE => {
            delete_registry_key(*argv.offset(*i as isize));
            // break
        }
        ACTION_EXPORT => {
            let mut filename_0: *mut WCHAR = *argv.offset(*i as isize);
            *i += 1;
            let mut key_name: *mut WCHAR = *argv.offset(*i as isize);

            if !key_name.is_null() && *key_name != 0 {
                export_registry_key(filename_0, key_name, REG_FORMAT_5 as DWORD);
            } else {
                export_registry_key(filename_0, core::ptr::null_mut(), REG_FORMAT_5 as DWORD);
            }
            // break
        }
        _ => {
            output_message(STRING_UNHANDLED_ACTION as c_uint);
            // break
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn ProcessCmdLine(mut cmdline: *mut WCHAR) -> BOOL {
    let mut argv: *mut *mut WCHAR = ::core::ptr::null_mut::<*mut WCHAR>();
    let mut argc: ::core::ffi::c_int = 0;
    let mut i: ::core::ffi::c_int = 0;
    let mut action: REGEDIT_ACTION = ACTION_ADD;
    let mut silent: BOOL = (FALSE) as i32;

    argv = CommandLineToArgvW(cmdline as LPCWSTR, core::ptr::addr_of_mut!(argc)) as *mut *mut WCHAR;

    if argv.is_null() {
        return (FALSE) as BOOL;
    }

    if argc == 1 {
        LocalFree(argv as HLOCAL);
        return (FALSE) as BOOL;
    }

    i = 1;
    while i < argc {
        if *(*argv.offset(i as isize)).offset(0 as isize) as ::core::ffi::c_int != '/' as i32
            && *(*argv.offset(i as isize)).offset(0 as isize) as ::core::ffi::c_int != '-' as i32
        {
            break;
        }

        if *(*argv.offset(i as isize)).offset(1 as isize) == 0
            && *(*argv.offset(i as isize)).offset(0 as isize) as ::core::ffi::c_int == '-' as i32
        {
            break;
        }

        if *(*argv.offset(i as isize)).offset(1 as isize) as ::core::ffi::c_int != 0
            && *(*argv.offset(i as isize)).offset(2 as isize) as ::core::ffi::c_int != 0
            && *(*argv.offset(i as isize)).offset(2 as isize) as ::core::ffi::c_int != ':' as i32
        {
            break;
        }

        match towupper((*(*argv.offset(i as isize)).offset(1 as isize) as wint_t) as _)
            as ::core::ffi::c_int
        {
            63 => {
                error_exit(STRING_USAGE as ::core::ffi::c_uint);
            }
            68 => {
                action = ACTION_DELETE;
            }
            69 => {
                action = ACTION_EXPORT;
            }
            82 => {}
            83 => {
                silent = (TRUE) as i32;
            }
            67 | 76 | 77 | 86 => {}
            _ => {
                output_message(STRING_INVALID_SWITCH as ::core::ffi::c_uint, *argv.offset(i as isize));
                error_exit(STRING_HELP as ::core::ffi::c_uint);
            }
        }
        i += 1;
    }

    if i == argc {
        match (action as ::core::ffi::c_uint) as i32 {
            ACTION_ADD | ACTION_EXPORT => {
                output_message(STRING_NO_FILENAME as ::core::ffi::c_uint);
            }
            ACTION_DELETE => {
                output_message(STRING_NO_REG_KEY as ::core::ffi::c_uint);
            }
            _ => {}
        }
        error_exit(STRING_HELP as ::core::ffi::c_uint);
    }

    while i < argc {
        PerformRegAction(action, argv, core::ptr::addr_of_mut!(i), silent);
        i += 1;
    }

    LocalFree(argv as HLOCAL);

    return (TRUE) as BOOL;
}
