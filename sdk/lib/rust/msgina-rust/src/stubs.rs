use crate::prelude::*;
use crate::msgina_externs::*;
use crate::msgina_types::*;
use crate::resources::*;
use crate::msgina_consts::*;
use crate::strsafe::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut __wine_dbch_msgina: __wine_debug_channel = unsafe {
    __wine_debug_channel {
        flags: WINE_DBG_DEFAULT_FLAGS,
        name: ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(
            *b"msgina\0\0\0\0\0\0\0\0\0",
        ),
    }
};

static mut __wine_dbch___default: *mut __wine_debug_channel =
    unsafe { &raw const __wine_dbch_msgina as *mut __wine_debug_channel };
#[no_mangle]
pub unsafe extern "system" fn WlxIsLockOk(pWlxContext: PVOID) -> BOOL {
    if (*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_FIXME) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/msgina/stubs.c\0"
                .as_ptr() as *const core::ffi::c_char,
            b"WlxIsLockOk\0".as_ptr() as *const core::ffi::c_char,
            23i32,
            b"%s is unimplemented!()\n\0".as_ptr() as *const core::ffi::c_char,
            b"WlxIsLockOk\0".as_ptr() as *const core::ffi::c_char,
        );
    }
    TRUE.try_into().unwrap()
}

#[no_mangle]
pub unsafe extern "system" fn WlxGetStatusMessage(
    pWlxContext: PVOID,
    pdwOptions: *mut DWORD,
    pMessage: PWSTR,
    dwBufferSize: DWORD,
) -> BOOL {
    let _ = pWlxContext;
    let _ = pdwOptions;
    let _ = pMessage;
    let _ = dwBufferSize;

    if ((*__wine_dbch___default).flags as i32 & (1 << __WINE_DBCL_FIXME as i32)) != 0 {
        ros_dbg_log(
            __WINE_DBCL_FIXME,
            __wine_dbch___default,
            b"dll/win32/msgina/stubs.c\0".as_ptr() as *const i8,
            b"WlxGetStatusMessage\0".as_ptr() as *const i8,
            43,
            b"%s is unimplemented!()\n\0".as_ptr() as *const i8,
            b"WlxGetStatusMessage\0".as_ptr() as *const i8,
        );
    }

    0
}

#[no_mangle]
pub unsafe extern "system" fn WlxNetworkProviderLoad(
    pWlxContext: PVOID,
    pNprNotifyInfo: PWLX_MPR_NOTIFY_INFO,
) -> BOOL {
    let _ = pWlxContext;
    let _ = pNprNotifyInfo;

    if ((*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_FIXME as i32)) != 0 {
        ros_dbg_log(
            __WINE_DBCL_FIXME,
            __wine_dbch___default,
            b"dll/win32/msgina/stubs.c\0"
                .as_ptr() as *const core::ffi::c_char,
            b"WlxNetworkProviderLoad\0".as_ptr() as *const core::ffi::c_char,
            59i32,
            b"%s is unimplemented!()\n\0".as_ptr() as *const core::ffi::c_char,
            b"WlxNetworkProviderLoad\0".as_ptr() as *const core::ffi::c_char,
        );
    }
    0i32
}
#[no_mangle]
pub unsafe extern "system" fn WlxDisconnectNotify(pWlxContext: PVOID) {
    if ((*__wine_dbch___default).flags as u32 & (1u32 << __WINE_DBCL_FIXME as u32)) != 0 {
        ros_dbg_log(
            __WINE_DBCL_FIXME,
            __wine_dbch___default,
            b"dll/win32/msgina/stubs.c\0"
                .as_ptr() as *const core::ffi::c_char,
            b"WlxDisconnectNotify\0".as_ptr() as *const core::ffi::c_char,
            73i32,
            b"%s is unimplemented!()\n\0".as_ptr() as *const core::ffi::c_char,
            b"WlxDisconnectNotify\0".as_ptr() as *const core::ffi::c_char,
        );
    }
}
#[no_mangle]
pub unsafe extern "system" fn WlxGetConsoleSwitchCredentials(
    _pWlxContext: *mut ::core::ffi::c_void,
    _pCredInfo: *mut ::core::ffi::c_void,
) -> i32 {
    if ((*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_FIXME as i32)) != 0 {
        ros_dbg_log(
            __WINE_DBCL_FIXME,
            __wine_dbch___default,
            b"dll/win32/msgina/stubs.c\0"
                .as_ptr() as *const i8,
            b"WlxGetConsoleSwitchCredentials\0".as_ptr() as *const i8,
            88i32,
            b"%s is unimplemented!()\n\0".as_ptr() as *const i8,
            b"WlxGetConsoleSwitchCredentials\0".as_ptr() as *const i8,
        );
    }
    0i32
}
