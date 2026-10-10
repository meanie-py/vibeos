use crate::prelude::*;
use core::ffi::c_void;

extern "C" {
    fn _errno() -> *mut core::ffi::c_int;
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static RUSTCONV_C2R_WSTR_0DFED3EB75D7: [u16; 5] = [0x0061u16, 0x0075, 0x0074, 0x006F, 0x0000];
pub static RUSTCONV_C2R_WSTR_0EA14E58FC87: [u16; 8] = [0x0066u16, 0x0069, 0x006C, 0x0065, 0x0073, 0x0079, 0x0073, 0x0000];
pub static RUSTCONV_C2R_WSTR_3CCFDAA9AB82: [u16; 7] = [0x0073u16, 0x0065, 0x0076, 0x0065, 0x0072, 0x0065, 0x0000];
pub static RUSTCONV_C2R_WSTR_4445A0F6283A: [u16; 9] = [0x0063u16, 0x0072, 0x0069, 0x0074, 0x0069, 0x0063, 0x0061, 0x006C, 0x0000];
pub static RUSTCONV_C2R_WSTR_52B0C056F710: [u16; 9] = [0x0064u16, 0x0069, 0x0073, 0x0061, 0x0062, 0x006C, 0x0065, 0x0064, 0x0000];
pub static RUSTCONV_C2R_WSTR_568FB8CD9679: [u16; 6] = [0x0073u16, 0x0068, 0x0061, 0x0072, 0x0065, 0x0000];
pub static RUSTCONV_C2R_WSTR_5EBB0A34B8B5: [u16; 9] = [0x0069u16, 0x006E, 0x0074, 0x0065, 0x0072, 0x0061, 0x0063, 0x0074, 0x0000];
pub static RUSTCONV_C2R_WSTR_6D1618CC9381: [u16; 4] = [0x0079u16, 0x0065, 0x0073, 0x0000];
pub static RUSTCONV_C2R_WSTR_833AD7EF6992: [u16; 3] = [0x006Eu16, 0x006F, 0x0000];
pub static RUSTCONV_C2R_WSTR_A34A880B4CF5: [u16; 7] = [0x0064u16, 0x0065, 0x006D, 0x0061, 0x006E, 0x0064, 0x0000];
pub static RUSTCONV_C2R_WSTR_A47CD7311658: [u16; 5] = [0x0062u16, 0x006F, 0x006F, 0x0074, 0x0000];
pub static RUSTCONV_C2R_WSTR_ADDD934D07D5: [u16; 7] = [0x0073u16, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x0000];
pub static RUSTCONV_C2R_WSTR_CE6938C51DB9: [u16; 7] = [0x0069u16, 0x0067, 0x006E, 0x006F, 0x0072, 0x0065, 0x0000];
pub static RUSTCONV_C2R_WSTR_D0989A5D6F7D: [u16; 4] = [0x0072u16, 0x0065, 0x0063, 0x0000];
pub static RUSTCONV_C2R_WSTR_E62BD52B875C: [u16; 4] = [0x006Fu16, 0x0077, 0x006E, 0x0000];
pub static RUSTCONV_C2R_WSTR_E890E6A21D6A: [u16; 7] = [0x006Bu16, 0x0065, 0x0072, 0x006E, 0x0065, 0x006C, 0x0000];
pub static RUSTCONV_C2R_WSTR_F7C4867ABD59: [u16; 7] = [0x006Eu16, 0x006F, 0x0072, 0x006D, 0x0061, 0x006C, 0x0000];

static mut TypeOpts: [OPTION_INFO; 6] = unsafe {
    [
        OPTION_INFO {
            lpOption: RUSTCONV_C2R_WSTR_E62BD52B875C.as_ptr() as LPCTSTR,
            dwValue: SERVICE_WIN32_OWN_PROCESS as DWORD,
        },
        OPTION_INFO {
            lpOption: RUSTCONV_C2R_WSTR_568FB8CD9679.as_ptr() as LPCTSTR,
            dwValue: SERVICE_WIN32_SHARE_PROCESS as DWORD,
        },
        OPTION_INFO {
            lpOption: RUSTCONV_C2R_WSTR_5EBB0A34B8B5.as_ptr() as LPCTSTR,
            dwValue: SERVICE_INTERACTIVE_PROCESS as DWORD,
        },
        OPTION_INFO {
            lpOption: RUSTCONV_C2R_WSTR_E890E6A21D6A.as_ptr() as LPCTSTR,
            dwValue: SERVICE_KERNEL_DRIVER as DWORD,
        },
        OPTION_INFO {
            lpOption: RUSTCONV_C2R_WSTR_0EA14E58FC87.as_ptr() as LPCTSTR,
            dwValue: SERVICE_FILE_SYSTEM_DRIVER as DWORD,
        },
        OPTION_INFO {
            lpOption: RUSTCONV_C2R_WSTR_D0989A5D6F7D.as_ptr() as LPCTSTR,
            dwValue: SERVICE_RECOGNIZER_DRIVER as DWORD,
        },
    ]
};

static mut StartOpts: [OPTION_INFO; 5] = unsafe {
    [
        OPTION_INFO {
            lpOption: RUSTCONV_C2R_WSTR_A47CD7311658.as_ptr() as LPCTSTR,
            dwValue: SERVICE_BOOT_START as DWORD,
        },
        OPTION_INFO {
            lpOption: RUSTCONV_C2R_WSTR_ADDD934D07D5.as_ptr() as LPCTSTR,
            dwValue: SERVICE_SYSTEM_START as DWORD,
        },
        OPTION_INFO {
            lpOption: RUSTCONV_C2R_WSTR_0DFED3EB75D7.as_ptr() as LPCTSTR,
            dwValue: SERVICE_AUTO_START as DWORD,
        },
        OPTION_INFO {
            lpOption: RUSTCONV_C2R_WSTR_A34A880B4CF5.as_ptr() as LPCTSTR,
            dwValue: SERVICE_DEMAND_START as DWORD,
        },
        OPTION_INFO {
            lpOption: RUSTCONV_C2R_WSTR_52B0C056F710.as_ptr() as LPCTSTR,
            dwValue: SERVICE_DISABLED as DWORD,
        },
    ]
};

static mut ErrorOpts: [OPTION_INFO; 4] = unsafe {
    [
        OPTION_INFO {
            lpOption: RUSTCONV_C2R_WSTR_F7C4867ABD59.as_ptr() as LPCTSTR,
            dwValue: SERVICE_ERROR_NORMAL as DWORD,
        },
        OPTION_INFO {
            lpOption: RUSTCONV_C2R_WSTR_3CCFDAA9AB82.as_ptr() as LPCTSTR,
            dwValue: SERVICE_ERROR_SEVERE as DWORD,
        },
        OPTION_INFO {
            lpOption: RUSTCONV_C2R_WSTR_4445A0F6283A.as_ptr() as LPCTSTR,
            dwValue: SERVICE_ERROR_CRITICAL as DWORD,
        },
        OPTION_INFO {
            lpOption: RUSTCONV_C2R_WSTR_CE6938C51DB9.as_ptr() as LPCTSTR,
            dwValue: SERVICE_ERROR_IGNORE as DWORD,
        },
    ]
};

static mut TagOpts: [OPTION_INFO; 2] = unsafe {
    [
        OPTION_INFO {
            lpOption: RUSTCONV_C2R_WSTR_6D1618CC9381.as_ptr() as LPCTSTR,
            dwValue: TRUE as DWORD,
        },
        OPTION_INFO {
            lpOption: RUSTCONV_C2R_WSTR_833AD7EF6992.as_ptr() as LPCTSTR,
            dwValue: FALSE as DWORD,
        },
    ]
};

#[repr(C)]
pub struct OPTION_INFO {
    pub lpOption: LPCTSTR,
    pub dwValue: DWORD,
}

pub type POPTION_INFO = *mut OPTION_INFO;

unsafe impl Sync for OPTION_INFO {}
#[no_mangle]
pub unsafe extern "C" fn ParseCreateConfigArguments(mut ServiceArgs: *mut LPCTSTR, mut ArgCount: INT, mut bChangeService: BOOL, mut lpServiceInfo: LPSERVICE_CREATE_INFO) -> BOOL {
    let mut i: INT = 0;
    let mut ArgIndex: INT = 1 as INT;

    if ArgCount < 1 as INT {
        return FALSE as BOOL;
    }

    ZeroMemory!(
        lpServiceInfo as *mut core::ffi::c_void,
        core::mem::size_of::<SERVICE_CREATE_INFO>(),
    );

    if bChangeService != 0 {
        (*lpServiceInfo).dwServiceType = SERVICE_NO_CHANGE as DWORD;
        (*lpServiceInfo).dwStartType = SERVICE_NO_CHANGE as DWORD;
        (*lpServiceInfo).dwErrorControl = SERVICE_NO_CHANGE as DWORD;
    }

    (*lpServiceInfo).lpServiceName = *ServiceArgs.offset(0 as isize);

    ArgCount -= 1;

    while ArgCount > 1 as INT {
        if lstrcmpi(
            *ServiceArgs.offset(ArgIndex as isize),
            (['t' as u16, 'y' as u16, 'p' as u16, 'e' as u16, '=' as u16, 0].as_ptr()),
        ) == 0
        {
            i = 0 as INT;
            while (i as usize)
                < (core::mem::size_of_val(&TypeOpts)
                    / core::mem::size_of_val(&TypeOpts[0]))
            {
                if lstrcmpi(
                    *ServiceArgs.offset((ArgIndex + 1 as INT) as isize),
                    TypeOpts[i as usize].lpOption,
                ) == 0
                {
                    if (*lpServiceInfo).dwServiceType == SERVICE_NO_CHANGE as DWORD {
                        (*lpServiceInfo).dwServiceType = TypeOpts[i as usize].dwValue;
                    } else {
                        (*lpServiceInfo).dwServiceType |= TypeOpts[i as usize].dwValue;
                    }
                    break;
                }
                i += 1;
            }

            if (i as usize)
                == (core::mem::size_of_val(&TypeOpts)
                    / core::mem::size_of_val(&TypeOpts[0]))
            {
                break;
            }
        } else if lstrcmpi(
            *ServiceArgs.offset(ArgIndex as isize),
            (['s' as u16, 't' as u16, 'a' as u16, 'r' as u16, 't' as u16, '=' as u16, 0].as_ptr()),
        ) == 0
        {
            i = 0 as INT;
            while (i as usize)
                < (core::mem::size_of_val(&StartOpts)
                    / core::mem::size_of_val(&StartOpts[0]))
            {
                if lstrcmpi(
                    *ServiceArgs.offset((ArgIndex + 1 as INT) as isize),
                    StartOpts[i as usize].lpOption,
                ) == 0
                {
                    (*lpServiceInfo).dwStartType = StartOpts[i as usize].dwValue;
                    break;
                }
                i += 1;
            }

            if (i as usize)
                == (core::mem::size_of_val(&StartOpts)
                    / core::mem::size_of_val(&StartOpts[0]))
            {
                break;
            }
        } else if lstrcmpi(
            *ServiceArgs.offset(ArgIndex as isize),
            (['e' as u16, 'r' as u16, 'r' as u16, 'o' as u16, 'r' as u16, '=' as u16, 0].as_ptr()),
        ) == 0
        {
            i = 0 as INT;
            while (i as usize)
                < (core::mem::size_of_val(&ErrorOpts)
                    / core::mem::size_of_val(&ErrorOpts[0]))
            {
                if lstrcmpi(
                    *ServiceArgs.offset((ArgIndex + 1 as INT) as isize),
                    ErrorOpts[i as usize].lpOption,
                ) == 0
                {
                    (*lpServiceInfo).dwErrorControl = ErrorOpts[i as usize].dwValue;
                    break;
                }
                i += 1;
            }

            if (i as usize)
                == (core::mem::size_of_val(&ErrorOpts)
                    / core::mem::size_of_val(&ErrorOpts[0]))
            {
                break;
            }
        } else if lstrcmpi(
            *ServiceArgs.offset(ArgIndex as isize),
            (['t' as u16, 'a' as u16, 'g' as u16, '=' as u16, 0].as_ptr()),
        ) == 0
        {
            i = 0 as INT;
            while (i as usize)
                < (core::mem::size_of_val(&TagOpts)
                    / core::mem::size_of_val(&TagOpts[0]))
            {
                if lstrcmpi(
                    *ServiceArgs.offset((ArgIndex + 1 as INT) as isize),
                    TagOpts[i as usize].lpOption,
                ) == 0
                {
                    (*lpServiceInfo).bTagId = TagOpts[i as usize].dwValue as BOOL;
                    break;
                }
                i += 1;
            }

            if (i as usize)
                == (core::mem::size_of_val(&TagOpts)
                    / core::mem::size_of_val(&TagOpts[0]))
            {
                break;
            }
        } else if lstrcmpi(
            *ServiceArgs.offset(ArgIndex as isize),
            (['b' as u16, 'i' as u16, 'n' as u16, 'p' as u16, 'a' as u16, 't' as u16, 'h' as u16, '=' as u16, 0].as_ptr()),
        ) == 0
        {
            (*lpServiceInfo).lpBinaryPathName =
                *ServiceArgs.offset((ArgIndex + 1 as INT) as isize);
        } else if lstrcmpi(
            *ServiceArgs.offset(ArgIndex as isize),
            (['g' as u16, 'r' as u16, 'o' as u16, 'u' as u16, 'p' as u16, '=' as u16, 0].as_ptr()),
        ) == 0
        {
            (*lpServiceInfo).lpLoadOrderGroup =
                *ServiceArgs.offset((ArgIndex + 1 as INT) as isize);
        } else if lstrcmpi(
            *ServiceArgs.offset(ArgIndex as isize),
            (['d' as u16, 'e' as u16, 'p' as u16, 'e' as u16, 'n' as u16, 'd' as u16, '=' as u16, 0].as_ptr()),
        ) == 0
        {
            (*lpServiceInfo).lpDependencies =
                *ServiceArgs.offset((ArgIndex + 1 as INT) as isize);
        } else if lstrcmpi(
            *ServiceArgs.offset(ArgIndex as isize),
            (['o' as u16, 'b' as u16, 'j' as u16, '=' as u16, 0].as_ptr()),
        ) == 0
        {
            (*lpServiceInfo).lpServiceStartName =
                *ServiceArgs.offset((ArgIndex + 1 as INT) as isize);
        } else if lstrcmpi(
            *ServiceArgs.offset(ArgIndex as isize),
            (['d' as u16, 'i' as u16, 's' as u16, 'p' as u16, 'l' as u16, 'a' as u16, 'y' as u16, 'n' as u16, 'a' as u16, 'm' as u16, 'e' as u16, '=' as u16, 0].as_ptr()),
        ) == 0
        {
            (*lpServiceInfo).lpDisplayName =
                *ServiceArgs.offset((ArgIndex + 1 as INT) as isize);
        } else if lstrcmpi(
            *ServiceArgs.offset(ArgIndex as isize),
            (['p' as u16, 'a' as u16, 's' as u16, 's' as u16, 'w' as u16, 'o' as u16, 'r' as u16, 'd' as u16, '=' as u16, 0].as_ptr()),
        ) == 0
        {
            (*lpServiceInfo).lpPassword =
                *ServiceArgs.offset((ArgIndex + 1 as INT) as isize);
        }

        ArgIndex += 2 as INT;
        ArgCount -= 2 as INT;
    }

    return (ArgCount == 0 as INT) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn ParseFailureActions(
    lpActions: LPCTSTR,
    pcActions: *mut DWORD,
    ppActions: *mut *mut SC_ACTION,
) -> BOOL {
    let mut pActions: *mut SC_ACTION = core::ptr::null_mut();
    let mut pStringBuffer: LPTSTR = core::ptr::null_mut();
    let mut p: LPTSTR;
    let mut nLength: INT_PTR;
    let mut nCount: INT = 0;

    *pcActions = 0;
    *ppActions = core::ptr::null_mut();

    nLength = _tcslen(lpActions) as INT_PTR;

    /* Allocate the string buffer */
    pStringBuffer = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        (((nLength + 2) as usize) * core::mem::size_of::<TCHAR>()) as SIZE_T,
    ) as LPTSTR;
    if pStringBuffer.is_null() {
        return (FALSE) as BOOL;
    }

    /* Copy the actions string into the string buffer */
    CopyMemory!(
        pStringBuffer as *mut core::ffi::c_void,
        lpActions as *const core::ffi::c_void,
        ((nLength as usize) * core::mem::size_of::<TCHAR>()) as usize,
    );

    /* Replace all slashes by null characters */
    p = pStringBuffer;
    while *p != '\0' as u16 {
        if *p == '/' as u16 {
            *p = '\0' as u16;
        }
        p = p.offset(1);
    }

    /* Count the arguments in the buffer */
    p = pStringBuffer;
    while *p != '\0' as u16 {
        nCount += 1;

        nLength = _tcslen(p as *const WCHAR) as INT_PTR;
        p = (p as LONG_PTR
            + ((((nLength + 1) as usize) * core::mem::size_of::<TCHAR>()) as LONG_PTR))
            as LPTSTR;
    }

    /* Allocate the actions buffer */
    pActions = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        ((nCount / 2) * (core::mem::size_of::<SC_ACTION>()) as i32) as SIZE_T,
    ) as *mut SC_ACTION;
    if pActions.is_null() {
        HeapFree(
            GetProcessHeap(),
            0,
            pStringBuffer as PVOID,
        );
        return (FALSE) as BOOL;
    }

    /* Parse the string buffer */
    nCount = 0;
    p = pStringBuffer;
    while *p != '\0' as u16 {
        nLength = _tcslen(p as *const WCHAR) as INT_PTR;

        if nCount % 2 == 0 {
            /* Action */
            if lstrcmpi(
                p as LPCTSTR,
                ['r' as u16, 'e' as u16, 'b' as u16, 'o' as u16, 'o' as u16, 't' as u16, 0u16].as_ptr(),
            ) == 0
            {
                (*pActions.offset((nCount / 2) as isize)).Type =
                    SC_ACTION_REBOOT;
            } else if lstrcmpi(
                p as LPCTSTR,
                ['r' as u16, 'e' as u16, 's' as u16, 't' as u16, 'a' as u16, 'r' as u16, 't' as u16, 0u16]
                    .as_ptr(),
            ) == 0
            {
                (*pActions.offset((nCount / 2) as isize)).Type =
                    SC_ACTION_RESTART;
            } else if lstrcmpi(
                p as LPCTSTR,
                ['r' as u16, 'u' as u16, 'n' as u16, 0u16].as_ptr(),
            ) == 0
            {
                (*pActions.offset((nCount / 2) as isize)).Type =
                    SC_ACTION_RUN_COMMAND;
            } else {
                break;
            }
        } else {
            /* Delay */
            (*pActions.offset((nCount / 2) as isize)).Delay =
                _tcstoul(p as *const WCHAR, core::ptr::null_mut(), 10)
                    as DWORD;
            if (*pActions.offset((nCount / 2) as isize)).Delay
                == 0
                && (*_errno()) as u32 == ERANGE
            {
                break;
            }
        }

        p = (p as LONG_PTR
            + ((((nLength + 1) as usize) * core::mem::size_of::<TCHAR>()) as LONG_PTR))
            as LPTSTR;
        nCount += 1;
    }

    /* Free the string buffer */
    HeapFree(
        GetProcessHeap(),
        0,
        pStringBuffer as PVOID,
    );

    *pcActions = (nCount / 2) as DWORD;
    *ppActions = pActions;

    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn ParseFailureArguments(ServiceArgs: *mut LPCTSTR, mut ArgCount: INT, ppServiceName: *mut LPCTSTR, pFailureActions: LPSERVICE_FAILURE_ACTIONS) -> BOOL {
    let mut ArgIndex: INT = 1;
    let mut lpActions: LPCTSTR = core::ptr::null();
    let mut lpReset: LPCTSTR = core::ptr::null();

    if ArgCount < 1 {
        return (FALSE) as BOOL;
    }

    ZeroMemory!(
        pFailureActions as *mut core::ffi::c_void,
        core::mem::size_of::<SERVICE_FAILURE_ACTIONS>() as usize,
    );

    *ppServiceName = *ServiceArgs.offset(0 as isize);

    ArgCount -= 1;

    while ArgCount > 1 {
        let actions_s = ['a' as u16, 'c' as u16, 't' as u16, 'i' as u16, 'o' as u16, 'n' as u16, 's' as u16, '=' as u16, 0u16];
        let command_s = ['c' as u16, 'o' as u16, 'm' as u16, 'm' as u16, 'a' as u16, 'n' as u16, 'd' as u16, '=' as u16, 0u16];
        let reboot_s = ['r' as u16, 'e' as u16, 'b' as u16, 'o' as u16, 'o' as u16, 't' as u16, '=' as u16, 0u16];
        let reset_s = ['r' as u16, 'e' as u16, 's' as u16, 'e' as u16, 't' as u16, '=' as u16, 0u16];

        if lstrcmpi(
            *ServiceArgs.offset(ArgIndex as isize),
            actions_s.as_ptr() as LPCTSTR,
        ) == 0
        {
            lpActions = *ServiceArgs.offset((ArgIndex + 1) as isize) as LPTSTR as LPCTSTR;
        } else if lstrcmpi(
            *ServiceArgs.offset(ArgIndex as isize),
            command_s.as_ptr() as LPCTSTR,
        ) == 0
        {
            (*pFailureActions).lpCommand = *ServiceArgs.offset((ArgIndex + 1) as isize) as LPTSTR;
        } else if lstrcmpi(
            *ServiceArgs.offset(ArgIndex as isize),
            reboot_s.as_ptr() as LPCTSTR,
        ) == 0
        {
            (*pFailureActions).lpRebootMsg = *ServiceArgs.offset((ArgIndex + 1) as isize) as LPTSTR;
        } else if lstrcmpi(
            *ServiceArgs.offset(ArgIndex as isize),
            reset_s.as_ptr() as LPCTSTR,
        ) == 0
        {
            lpReset = *ServiceArgs.offset((ArgIndex + 1) as isize) as LPTSTR as LPCTSTR;
        }

        ArgIndex += 2;
        ArgCount -= 2;
    }

    if (lpReset.is_null() && !lpActions.is_null()) || (!lpReset.is_null() && lpActions.is_null()) {
        let err_msg = [
            'E' as u16, 'R' as u16, 'R' as u16, 'O' as u16, 'R' as u16, ':' as u16,
            ' ' as u16, ' ' as u16,
            'T' as u16, 'h' as u16, 'e' as u16, ' ' as u16,
            'r' as u16, 'e' as u16, 's' as u16, 'e' as u16, 't' as u16, ' ' as u16,
            'a' as u16, 'n' as u16, 'd' as u16, ' ' as u16,
            'a' as u16, 'c' as u16, 't' as u16, 'i' as u16, 'o' as u16, 'n' as u16, 's' as u16, ' ' as u16,
            'o' as u16, 'p' as u16, 't' as u16, 'i' as u16, 'o' as u16, 'n' as u16, 's' as u16, ' ' as u16,
            'm' as u16, 'u' as u16, 's' as u16, 't' as u16, ' ' as u16,
            'b' as u16, 'e' as u16, ' ' as u16,
            'u' as u16, 's' as u16, 'e' as u16, 'd' as u16, ' ' as u16,
            's' as u16, 'i' as u16, 'm' as u16, 'u' as u16, 'l' as u16, 't' as u16, 'a' as u16,
            'n' as u16, 'e' as u16, 'o' as u16, 'u' as u16, 's' as u16, 'l' as u16, 'y' as u16, '.' as u16,
            '\n' as u16, '\n' as u16,
            0u16,
        ];
        _tprintf(err_msg.as_ptr() as LPCTSTR);
        return (FALSE) as BOOL;
    }

    if !lpReset.is_null() {
        let infinite_s = ['i' as u16, 'n' as u16, 'f' as u16, 'i' as u16, 'n' as u16, 'i' as u16, 't' as u16, 'e' as u16, 0u16];
        if lstrcmpi(lpReset, infinite_s.as_ptr() as LPCTSTR) == 0 {
            (*pFailureActions).dwResetPeriod = INFINITE as DWORD;
        } else {
            (*pFailureActions).dwResetPeriod = _ttoi(lpReset) as DWORD;
        }
    }

    if !lpActions.is_null() {
        if ParseFailureActions(
            lpActions,
            core::ptr::addr_of_mut!((*pFailureActions).cActions),
            core::ptr::addr_of_mut!((*pFailureActions).lpsaActions),
        ) == 0
        {
            return (FALSE) as BOOL;
        }
    }

    return (ArgCount == 0) as BOOL;
}
