use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut __wine_dbch___default: *mut __wine_debug_channel =
    unsafe { &raw const __wine_dbch_cmd as *mut __wine_debug_channel };

#[no_mangle]
pub static mut g_bDynamicTrace: BOOL = (FALSE) as i32;
#[no_mangle]
pub unsafe extern "C" fn CmdTrace_v(
    mut type_0: INT,
    mut file: LPCSTR,
    mut line: INT,
    mut func: LPCSTR,
    mut fmt: LPCSTR,
    ap_list: *mut ::core::ffi::c_void,
) {
    // The trampoline did va_start/va_end; this is its `va_list`, BY VALUE (char * on every
    // Windows target), which C passes twice below -- each callee walks its own copy.
    let va: *mut c_void = *(ap_list as *mut *mut c_void);
    let mut cch: ::core::ffi::c_int = 0;
    let mut szTextA: [::core::ffi::c_char; 800] = [0; 800];
    let mut szTextW: [wchar_t; 800] = [0; 800];
    static mut s_Debug: __wine_debug_functions = unsafe { core::mem::zeroed() };
    /* Console output */
    if g_bDynamicTrace != 0 {
        StringCchPrintfA!(
            szTextA.as_mut_ptr() as STRSAFE_LPSTR,
            _countof!(szTextA),
            b"%s (%d): \0".as_ptr() as STRSAFE_LPCSTR,
            file,
            line,
        );
        cch = lstrlenA(szTextA.as_ptr() as LPCSTR);
        StringCchVPrintfA(
            szTextA.as_mut_ptr().offset(cch as isize) as STRSAFE_LPSTR,
            (_countof!(szTextA) as usize) - (cch as usize),
            fmt as STRSAFE_LPCSTR,
            va,
        );

        MultiByteToWideChar(
            OutputCodePage,
            0 as DWORD,
            szTextA.as_ptr() as LPCSTR,
            -1 as ::core::ffi::c_int,
            szTextW.as_mut_ptr() as LPWSTR,
            _countof!(szTextW) as ::core::ffi::c_int,
        );
        szTextW[(_countof!(szTextW) - 1) as usize] = UNICODE_NULL as wchar_t; /* Avoid buffer overrun */
        ConOutPuts!(szTextW.as_mut_ptr() as PCWSTR);
    }

    if (*core::ptr::addr_of!(s_Debug)).dbg_vlog.is_none() {
        __wine_dbg_set_functions(
            core::ptr::null::<__wine_debug_functions>(),
            core::ptr::addr_of_mut!(s_Debug),
            core::mem::size_of::<__wine_debug_functions>() as size_t,
        );
    }

    /* Debug logging */
    if type_0 == __WINE_DBCL_FIXME {
        if FIXME_ON!(__wine_dbch_cmd) != 0 {
            (*core::ptr::addr_of!(s_Debug)).dbg_vlog.unwrap_unchecked()(
                __WINE_DBCL_FIXME,
                __wine_dbch___default,
                file as *const ::core::ffi::c_char,
                func as *const ::core::ffi::c_char,
                line as ::core::ffi::c_int,
                fmt as *const ::core::ffi::c_char,
                va,
            );
        }
    } else if type_0 == __WINE_DBCL_ERR {
        if ERR_ON!(__wine_dbch_cmd) != 0 {
            (*core::ptr::addr_of!(s_Debug)).dbg_vlog.unwrap_unchecked()(
                __WINE_DBCL_ERR,
                __wine_dbch___default,
                file as *const ::core::ffi::c_char,
                func as *const ::core::ffi::c_char,
                line as ::core::ffi::c_int,
                fmt as *const ::core::ffi::c_char,
                va,
            );
        }
    } else if type_0 == __WINE_DBCL_WARN {
        if WARN_ON!(__wine_dbch_cmd) != 0 {
            (*core::ptr::addr_of!(s_Debug)).dbg_vlog.unwrap_unchecked()(
                __WINE_DBCL_WARN,
                __wine_dbch___default,
                file as *const ::core::ffi::c_char,
                func as *const ::core::ffi::c_char,
                line as ::core::ffi::c_int,
                fmt as *const ::core::ffi::c_char,
                va,
            );
        }
    } else if type_0 == __WINE_DBCL_TRACE {
        if TRACE_ON!(__wine_dbch_cmd) != 0 {
            (*core::ptr::addr_of!(s_Debug)).dbg_vlog.unwrap_unchecked()(
                __WINE_DBCL_TRACE,
                __wine_dbch___default,
                file as *const ::core::ffi::c_char,
                func as *const ::core::ffi::c_char,
                line as ::core::ffi::c_int,
                fmt as *const ::core::ffi::c_char,
                va,
            );
        }
    } else {
        /* default: break */
    }

}
