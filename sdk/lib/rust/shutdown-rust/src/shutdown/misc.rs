// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_9F0256D484A7: [u16; 17] = [
    0x0045u16, 0x0072, 0x0072, 0x006F, 0x0072, 0x0020, 0x0063, 0x006F, 0x0064, 0x0065,
    0x003A, 0x0020, 0x0025, 0x006C, 0x0075, 0x000A, 0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static RUSTCONV_C2R_WSTR_0009EA1DF10E: [u16; 2] = [0x0055u16, 0x0000];
pub static RUSTCONV_C2R_WSTR_29374CFDD45D: [u16; 2] = [0x0050u16, 0x0000];
pub static RUSTCONV_C2R_WSTR_4FD33DCDE470: [u16; 2] = [0x0045u16, 0x0000];
pub static RUSTCONV_C2R_WSTR_DAD14AC9087B: [u16; 3] = [0x0045u16, 0x0050, 0x0000];

#[no_mangle]
pub static mut defaultReason: DWORD =
    (SHTDN_REASON_MAJOR_OTHER | SHTDN_REASON_MINOR_OTHER) as DWORD;

#[no_mangle]
pub static mut shutdownReason: [REASON; 29] = unsafe {
    [
        _REASON {
            prefix: RUSTCONV_C2R_WSTR_0009EA1DF10E.as_ptr() as LPWSTR,
            major: 0 as ::core::ffi::c_int,
            minor: 0 as ::core::ffi::c_int,
            flag: (SHTDN_REASON_MAJOR_OTHER | SHTDN_REASON_MINOR_OTHER) as DWORD,
        },
        _REASON {
            prefix: RUSTCONV_C2R_WSTR_4FD33DCDE470.as_ptr() as LPWSTR,
            major: 0 as ::core::ffi::c_int,
            minor: 0 as ::core::ffi::c_int,
            flag: (SHTDN_REASON_MAJOR_OTHER | SHTDN_REASON_MINOR_OTHER) as DWORD,
        },
        _REASON {
            prefix: RUSTCONV_C2R_WSTR_DAD14AC9087B.as_ptr() as LPWSTR,
            major: 0 as ::core::ffi::c_int,
            minor: 0 as ::core::ffi::c_int,
            flag: ((SHTDN_REASON_MAJOR_OTHER | SHTDN_REASON_MINOR_OTHER) as ::core::ffi::c_uint
                | SHTDN_REASON_FLAG_PLANNED) as DWORD,
        },
        _REASON {
            prefix: RUSTCONV_C2R_WSTR_0009EA1DF10E.as_ptr() as LPWSTR,
            major: 0 as ::core::ffi::c_int,
            minor: 5 as ::core::ffi::c_int,
            flag: (SHTDN_REASON_MAJOR_OTHER | SHTDN_REASON_MINOR_HUNG) as DWORD,
        },
        _REASON {
            prefix: RUSTCONV_C2R_WSTR_4FD33DCDE470.as_ptr() as LPWSTR,
            major: 1 as ::core::ffi::c_int,
            minor: 1 as ::core::ffi::c_int,
            flag: (SHTDN_REASON_MAJOR_HARDWARE | SHTDN_REASON_MINOR_MAINTENANCE) as DWORD,
        },
        _REASON {
            prefix: RUSTCONV_C2R_WSTR_DAD14AC9087B.as_ptr() as LPWSTR,
            major: 1 as ::core::ffi::c_int,
            minor: 1 as ::core::ffi::c_int,
            flag: ((SHTDN_REASON_MAJOR_HARDWARE | SHTDN_REASON_MINOR_MAINTENANCE)
                as ::core::ffi::c_uint
                | SHTDN_REASON_FLAG_PLANNED) as DWORD,
        },
        _REASON {
            prefix: RUSTCONV_C2R_WSTR_4FD33DCDE470.as_ptr() as LPWSTR,
            major: 1 as ::core::ffi::c_int,
            minor: 2 as ::core::ffi::c_int,
            flag: (SHTDN_REASON_MAJOR_HARDWARE | SHTDN_REASON_MINOR_INSTALLATION) as DWORD,
        },
        _REASON {
            prefix: RUSTCONV_C2R_WSTR_DAD14AC9087B.as_ptr() as LPWSTR,
            major: 1 as ::core::ffi::c_int,
            minor: 2 as ::core::ffi::c_int,
            flag: ((SHTDN_REASON_MAJOR_HARDWARE | SHTDN_REASON_MINOR_INSTALLATION)
                as ::core::ffi::c_uint
                | SHTDN_REASON_FLAG_PLANNED) as DWORD,
        },
        _REASON {
            prefix: RUSTCONV_C2R_WSTR_29374CFDD45D.as_ptr() as LPWSTR,
            major: 2 as ::core::ffi::c_int,
            minor: 3 as ::core::ffi::c_int,
            flag: ((SHTDN_REASON_MAJOR_OPERATINGSYSTEM | SHTDN_REASON_MINOR_UPGRADE)
                as ::core::ffi::c_uint
                | SHTDN_REASON_FLAG_PLANNED) as DWORD,
        },
        _REASON {
            prefix: RUSTCONV_C2R_WSTR_4FD33DCDE470.as_ptr() as LPWSTR,
            major: 2 as ::core::ffi::c_int,
            minor: 4 as ::core::ffi::c_int,
            flag: (SHTDN_REASON_MAJOR_OPERATINGSYSTEM | SHTDN_REASON_MINOR_RECONFIG) as DWORD,
        },
        _REASON {
            prefix: RUSTCONV_C2R_WSTR_DAD14AC9087B.as_ptr() as LPWSTR,
            major: 2 as ::core::ffi::c_int,
            minor: 4 as ::core::ffi::c_int,
            flag: ((SHTDN_REASON_MAJOR_OPERATINGSYSTEM | SHTDN_REASON_MINOR_RECONFIG)
                as ::core::ffi::c_uint
                | SHTDN_REASON_FLAG_PLANNED) as DWORD,
        },
        _REASON {
            prefix: RUSTCONV_C2R_WSTR_29374CFDD45D.as_ptr() as LPWSTR,
            major: 2 as ::core::ffi::c_int,
            minor: 16 as ::core::ffi::c_int,
            flag: ((SHTDN_REASON_MAJOR_OPERATINGSYSTEM | SHTDN_REASON_MINOR_RECONFIG)
                as ::core::ffi::c_uint
                | SHTDN_REASON_FLAG_PLANNED) as DWORD,
        },
        _REASON {
            prefix: RUSTCONV_C2R_WSTR_0009EA1DF10E.as_ptr() as LPWSTR,
            major: 2 as ::core::ffi::c_int,
            minor: 17 as ::core::ffi::c_int,
            flag: (SHTDN_REASON_MAJOR_OPERATINGSYSTEM | SHTDN_REASON_MINOR_HOTFIX) as DWORD,
        },
        _REASON {
            prefix: RUSTCONV_C2R_WSTR_29374CFDD45D.as_ptr() as LPWSTR,
            major: 2 as ::core::ffi::c_int,
            minor: 17 as ::core::ffi::c_int,
            flag: ((SHTDN_REASON_MAJOR_OPERATINGSYSTEM | SHTDN_REASON_MINOR_HOTFIX)
                as ::core::ffi::c_uint
                | SHTDN_REASON_FLAG_PLANNED) as DWORD,
        },
        _REASON {
            prefix: RUSTCONV_C2R_WSTR_0009EA1DF10E.as_ptr() as LPWSTR,
            major: 2 as ::core::ffi::c_int,
            minor: 18 as ::core::ffi::c_int,
            flag: (SHTDN_REASON_MAJOR_OPERATINGSYSTEM | SHTDN_REASON_MINOR_SECURITYFIX) as DWORD,
        },
        _REASON {
            prefix: RUSTCONV_C2R_WSTR_29374CFDD45D.as_ptr() as LPWSTR,
            major: 2 as ::core::ffi::c_int,
            minor: 18 as ::core::ffi::c_int,
            flag: ((SHTDN_REASON_MAJOR_OPERATINGSYSTEM | SHTDN_REASON_MINOR_SECURITYFIX)
                as ::core::ffi::c_uint
                | SHTDN_REASON_FLAG_PLANNED) as DWORD,
        },
        _REASON {
            prefix: RUSTCONV_C2R_WSTR_4FD33DCDE470.as_ptr() as LPWSTR,
            major: 4 as ::core::ffi::c_int,
            minor: 1 as ::core::ffi::c_int,
            flag: (SHTDN_REASON_MAJOR_APPLICATION | SHTDN_REASON_MINOR_MAINTENANCE) as DWORD,
        },
        _REASON {
            prefix: RUSTCONV_C2R_WSTR_DAD14AC9087B.as_ptr() as LPWSTR,
            major: 4 as ::core::ffi::c_int,
            minor: 1 as ::core::ffi::c_int,
            flag: ((SHTDN_REASON_MAJOR_APPLICATION | SHTDN_REASON_MINOR_MAINTENANCE)
                as ::core::ffi::c_uint
                | SHTDN_REASON_FLAG_PLANNED) as DWORD,
        },
        _REASON {
            prefix: RUSTCONV_C2R_WSTR_DAD14AC9087B.as_ptr() as LPWSTR,
            major: 4 as ::core::ffi::c_int,
            minor: 2 as ::core::ffi::c_int,
            flag: ((SHTDN_REASON_MAJOR_APPLICATION | SHTDN_REASON_MINOR_INSTALLATION)
                as ::core::ffi::c_uint
                | SHTDN_REASON_FLAG_PLANNED) as DWORD,
        },
        _REASON {
            prefix: RUSTCONV_C2R_WSTR_4FD33DCDE470.as_ptr() as LPWSTR,
            major: 4 as ::core::ffi::c_int,
            minor: 5 as ::core::ffi::c_int,
            flag: (SHTDN_REASON_MAJOR_APPLICATION | SHTDN_REASON_MINOR_HUNG) as DWORD,
        },
        _REASON {
            prefix: RUSTCONV_C2R_WSTR_4FD33DCDE470.as_ptr() as LPWSTR,
            major: 4 as ::core::ffi::c_int,
            minor: 6 as ::core::ffi::c_int,
            flag: (SHTDN_REASON_MAJOR_APPLICATION | SHTDN_REASON_MINOR_UNSTABLE) as DWORD,
        },
        _REASON {
            prefix: RUSTCONV_C2R_WSTR_0009EA1DF10E.as_ptr() as LPWSTR,
            major: 5 as ::core::ffi::c_int,
            minor: 15 as ::core::ffi::c_int,
            flag: (SHTDN_REASON_MAJOR_SYSTEM | SHTDN_REASON_MINOR_BLUESCREEN) as DWORD,
        },
        _REASON {
            prefix: RUSTCONV_C2R_WSTR_4FD33DCDE470.as_ptr() as LPWSTR,
            major: 5 as ::core::ffi::c_int,
            minor: 19 as ::core::ffi::c_int,
            flag: (SHTDN_REASON_MAJOR_SYSTEM | SHTDN_REASON_MINOR_SECURITY) as DWORD,
        },
        _REASON {
            prefix: RUSTCONV_C2R_WSTR_0009EA1DF10E.as_ptr() as LPWSTR,
            major: 5 as ::core::ffi::c_int,
            minor: 19 as ::core::ffi::c_int,
            flag: (SHTDN_REASON_MAJOR_SYSTEM | SHTDN_REASON_MINOR_SECURITY) as DWORD,
        },
        _REASON {
            prefix: RUSTCONV_C2R_WSTR_DAD14AC9087B.as_ptr() as LPWSTR,
            major: 5 as ::core::ffi::c_int,
            minor: 19 as ::core::ffi::c_int,
            flag: ((SHTDN_REASON_MAJOR_SYSTEM | SHTDN_REASON_MINOR_SECURITY) as ::core::ffi::c_uint
                | SHTDN_REASON_FLAG_PLANNED) as DWORD,
        },
        _REASON {
            prefix: RUSTCONV_C2R_WSTR_4FD33DCDE470.as_ptr() as LPWSTR,
            major: 5 as ::core::ffi::c_int,
            minor: 20 as ::core::ffi::c_int,
            flag: (SHTDN_REASON_MAJOR_SYSTEM | SHTDN_REASON_MINOR_NETWORK_CONNECTIVITY) as DWORD,
        },
        _REASON {
            prefix: RUSTCONV_C2R_WSTR_0009EA1DF10E.as_ptr() as LPWSTR,
            major: 6 as ::core::ffi::c_int,
            minor: 11 as ::core::ffi::c_int,
            flag: (SHTDN_REASON_MAJOR_POWER | SHTDN_REASON_MINOR_CORDUNPLUGGED) as DWORD,
        },
        _REASON {
            prefix: RUSTCONV_C2R_WSTR_0009EA1DF10E.as_ptr() as LPWSTR,
            major: 6 as ::core::ffi::c_int,
            minor: 12 as ::core::ffi::c_int,
            flag: (SHTDN_REASON_MAJOR_POWER | SHTDN_REASON_MINOR_ENVIRONMENT) as DWORD,
        },
        _REASON {
            prefix: RUSTCONV_C2R_WSTR_29374CFDD45D.as_ptr() as LPWSTR,
            major: 7 as ::core::ffi::c_int,
            minor: 0 as ::core::ffi::c_int,
            flag: (SHTDN_REASON_MAJOR_POWER | SHTDN_REASON_MINOR_ENVIRONMENT) as DWORD,
        },
    ]
};

// Field names are assumed (the C REASON typedef is not in this preamble):
// name = wide string, f1/f2 = the two small integers, reason = SHTDN_REASON_* flags.
#[no_mangle]
pub unsafe extern "C" fn ParseReasonCode(code: LPCWSTR) -> DWORD {
    let mut reasonptr: PREASON = core::ptr::null_mut();
    let mut majorCode: c_int = 0;
    let mut minorCode: c_int = 0;
    let mut tmpPrefix: LPWSTR = core::ptr::null_mut();
    let mut codeSize: size_t;

    /* If no reason code is specified, use "Other (Unplanned)" as the default option */
    if code.is_null() {
        return defaultReason;
    } else {
        /* Store the size of the code so we can use it later */
        codeSize = wcslen(code as *const wchar_t) as size_t;

        /* A colon cannot be the first or last character in the reason code */
        if (*code.offset(0 as isize) == 0x003Au16)
            || (*code.offset(codeSize as isize) == 0x003Au16)
        {
            return defaultReason;
        }

        /* The minimum length that a reason code can be is 5-7 characters in length */
        if (codeSize < 5) || (codeSize > 7) {
            return defaultReason;
        }

        /* TODO: Add code for reason parsing here. */

        /* Make sure that the major and minor codes are within size limits */
        if (majorCode > 7)
            || (majorCode < 0)
            || (minorCode > 20)
            || (minorCode < 0)
        {
            return defaultReason;
        }

        /* Figure out what flags to return */
        reasonptr = core::ptr::addr_of_mut!(shutdownReason) as *mut REASON as PREASON;
        // The C walks until a NULL prefix, but shutdownReason[] has none: past the last entry
        // it reads off the table. Bounded by the array, and a NULL tmpPrefix (a /d code the
        // parser above did not fill) is not handed to _wcsicmp.
        let reasonEnd: PREASON =
            (core::ptr::addr_of_mut!(shutdownReason) as PREASON).add(shutdownReason.len());
        while reasonptr < reasonEnd && !(*reasonptr).prefix.is_null() {
            if (majorCode == (*reasonptr).major)
                && (minorCode == (*reasonptr).minor)
                && !tmpPrefix.is_null()
                && (_wcsicmp(
                    tmpPrefix as *const wchar_t,
                    (*reasonptr).prefix as *const wchar_t,
                ) != 0)
            {
                return (*reasonptr).flag;
            }
            reasonptr = reasonptr.offset(1);
        }
    }

    return defaultReason;
}
#[no_mangle]
pub unsafe extern "C" fn DisplayError(dwError: DWORD) {
    ConMsgPuts(StdErr, FORMAT_MESSAGE_FROM_SYSTEM as DWORD, core::ptr::null_mut(), dwError, LANG_USER_DEFAULT as DWORD);
    ConPrintf(StdErr, RUSTCONV_WSTR_9F0256D484A7.as_ptr(), dwError);
}
