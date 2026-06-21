use crate::prelude::*;
use crate::msgina_externs::*;
use crate::msgina_types::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const MSV1_0_PACKAGE_NAME: [::core::ffi::c_char; 38] = unsafe {
    ::core::mem::transmute::<[u8; 38], [::core::ffi::c_char; 38]>(
        *b"MICROSOFT_AUTHENTICATION_PACKAGE_V1_0\0",
    )
};

static mut __wine_dbch_msgina: __wine_debug_channel = unsafe {
    __wine_debug_channel {
        flags: !(0 as ::core::ffi::c_int) as ::core::ffi::c_uchar,
        name: ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(
            *b"msgina\0\0\0\0\0\0\0\0\0",
        ),
    }
};

static mut __wine_dbch___default: *mut __wine_debug_channel =
    unsafe { &raw const __wine_dbch_msgina as *mut __wine_debug_channel };

static mut User32TokenSourceName: [CHAR; 9] =
    unsafe { ::core::mem::transmute::<[u8; 9], [CHAR; 9]>(*b"User32  \0") };
#[no_mangle]
pub unsafe extern "C" fn ConnectToLsa(pgContext: *mut GINA_CONTEXT) -> NTSTATUS {
    let mut LogonProcessName: LSA_STRING = LSA_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut PackageName: LSA_STRING = LSA_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut SecurityMode: LSA_OPERATIONAL_MODE = 0;
    let mut Status: NTSTATUS = 0;

    if !(*pgContext).LsaHandle.is_null() {
        return STATUS_SUCCESS;
    }

    RtlInitAnsiString(
        core::ptr::addr_of_mut!(LogonProcessName) as PANSI_STRING,
        b"MSGINA\0".as_ptr() as *const i8,
    );

    Status = LsaRegisterLogonProcess(
        core::ptr::addr_of_mut!(LogonProcessName) as PANSI_STRING,
        core::ptr::addr_of_mut!((*pgContext).LsaHandle),
        core::ptr::addr_of_mut!(SecurityMode),
    );

    if !(Status >= 0) {
        if ((*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_ERR as i32)) != 0 {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/msgina/lsa.c\0".as_ptr() as *const i8,
                b"ConnectToLsa\0".as_ptr() as *const i8,
                32,
                b"LsaRegisterLogonProcess failed (Status 0x%08lx)\n\0".as_ptr() as *const i8,
                Status,
            );
        }
        return Status;
    }

    RtlInitAnsiString(
        core::ptr::addr_of_mut!(PackageName) as PANSI_STRING,
        MSV1_0_PACKAGE_NAME.as_ptr() as *const i8,
    );

    Status = LsaLookupAuthenticationPackage(
        (*pgContext).LsaHandle,
        core::ptr::addr_of_mut!(PackageName) as PANSI_STRING,
        core::ptr::addr_of_mut!((*pgContext).AuthenticationPackage),
    );

    if !(Status >= 0) {
        if ((*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_ERR as i32)) != 0 {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl_0,
                __dbch_0,
                b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/msgina/lsa.c\0".as_ptr() as *const i8,
                b"ConnectToLsa\0".as_ptr() as *const i8,
                45,
                b"LsaLookupAuthenticationPackage failed (Status 0x%08lx)\n\0".as_ptr() as *const i8,
                Status,
            );
        }
    }

    Status
}
