use ntdll_sys::nt::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut __wine_dbch___default: *mut __wine_debug_channel =
    unsafe { &raw const __wine_dbch_cmd as *mut __wine_debug_channel };

#[no_mangle]
pub static mut g_bDynamicTrace: BOOL = FALSE;
#[no_mangle]
pub unsafe extern "C" fn CmdTrace(
    mut type_0: INT,
    mut file: LPCSTR,
    mut line: INT,
    mut func: LPCSTR,
    mut fmt: LPCSTR,
    mut args: ...
) {
    let mut va: ::core::ffi::VaListImpl = args.clone();
    let mut cch: ::core::ffi::c_int = 0;
    let mut szTextA: [::core::ffi::c_char; 800] = [0; 800];
    let mut szTextW: [wchar_t; 800] = [0; 800];
    static mut s_Debug: __wine_debug_functions = unsafe { core::mem::zeroed() };
    va_start(va, fmt);

    /* Console output */
    if g_bDynamicTrace != 0 {
        StringCchPrintfA(
            szTextA.as_mut_ptr() as STRSAFE_LPSTR,
            _countof(szTextA),
            b"%s (%d): \0".as_ptr() as STRSAFE_LPCSTR,
            file,
            line,
        );
        cch = lstrlenA(szTextA.as_ptr() as LPCSTR);
        StringCchVPrintfA(
            szTextA.as_mut_ptr().offset(cch as isize) as STRSAFE_LPSTR,
            _countof(szTextA) - cch as usize,
            fmt as STRSAFE_LPCSTR,
            va.as_va_list(),
        );

        MultiByteToWideChar(
            OutputCodePage,
            0 as DWORD,
            szTextA.as_ptr() as LPCSTR,
            -1 as ::core::ffi::c_int,
            szTextW.as_mut_ptr() as LPWSTR,
            _countof(szTextW) as ::core::ffi::c_int,
        );
        szTextW[_countof(szTextW) - 1] = UNICODE_NULL as wchar_t; /* Avoid buffer overrun */
        ConOutPuts(szTextW.as_mut_ptr() as PCWSTR);
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
        if __WINE_IS_DEBUG_ON(_FIXME, __wine_dbch___default) {
            (*core::ptr::addr_of!(s_Debug)).dbg_vlog.expect("non-null function pointer")(
                __WINE_DBCL_FIXME,
                __wine_dbch___default,
                file as *const ::core::ffi::c_char,
                func as *const ::core::ffi::c_char,
                line as ::core::ffi::c_int,
                fmt as *const ::core::ffi::c_char,
                va.as_va_list(),
            );
        }
    } else if type_0 == __WINE_DBCL_ERR {
        if __WINE_IS_DEBUG_ON(_ERR, __wine_dbch___default) {
            (*core::ptr::addr_of!(s_Debug)).dbg_vlog.expect("non-null function pointer")(
                __WINE_DBCL_ERR,
                __wine_dbch___default,
                file as *const ::core::ffi::c_char,
                func as *const ::core::ffi::c_char,
                line as ::core::ffi::c_int,
                fmt as *const ::core::ffi::c_char,
                va.as_va_list(),
            );
        }
    } else if type_0 == __WINE_DBCL_WARN {
        if __WINE_IS_DEBUG_ON(_WARN, __wine_dbch___default) {
            (*core::ptr::addr_of!(s_Debug)).dbg_vlog.expect("non-null function pointer")(
                __WINE_DBCL_WARN,
                __wine_dbch___default,
                file as *const ::core::ffi::c_char,
                func as *const ::core::ffi::c_char,
                line as ::core::ffi::c_int,
                fmt as *const ::core::ffi::c_char,
                va.as_va_list(),
            );
        }
    } else if type_0 == __WINE_DBCL_TRACE {
        if __WINE_IS_DEBUG_ON(_TRACE, __wine_dbch___default) {
            (*core::ptr::addr_of!(s_Debug)).dbg_vlog.expect("non-null function pointer")(
                __WINE_DBCL_TRACE,
                __wine_dbch___default,
                file as *const ::core::ffi::c_char,
                func as *const ::core::ffi::c_char,
                line as ::core::ffi::c_int,
                fmt as *const ::core::ffi::c_char,
                va.as_va_list(),
            );
        }
    } else {
        /* default: break */
    }

    va_end(va);
}
