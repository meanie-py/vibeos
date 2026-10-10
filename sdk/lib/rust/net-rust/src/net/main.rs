use crate::net::cmdAccounts::cmdAccounts;
use crate::net::cmdComputer::cmdComputer;
use crate::net::cmdConfig::cmdConfig;
use crate::net::cmdContinue::cmdContinue;
use crate::net::cmdGroup::cmdGroup;
use crate::net::cmdHelp::cmdHelp;
use crate::net::cmdHelpMsg::cmdHelpMsg;
use crate::net::cmdLocalGroup::cmdLocalGroup;
use crate::net::cmdPause::cmdPause;
use crate::net::cmdSession::cmdSession;
use crate::net::cmdShare::cmdShare;
use crate::net::cmdStart::cmdStart;
use crate::net::cmdStatistics::cmdStatistics;
use crate::net::cmdStop::cmdStop;
use crate::net::cmdHelp::cmdSyntax;
use crate::net::cmdUse::cmdUse;
use crate::net::cmdUser::cmdUser;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_075DE2B906DB: [u16; 2] = [0x000Au16, 0x0000];
pub static RUSTCONV_WSTR_4BA5D7E4F16E: [u16; 4] = [0x0025u16, 0x0073, 0x000A, 0x0000];
pub static RUSTCONV_WSTR_6A034B9F8DB0: [u16; 27] = [
    0x0046u16, 0x0061, 0x0069, 0x006C, 0x0065, 0x0064, 0x0020, 0x0074, 0x006F, 0x0020,
    0x006C, 0x006F, 0x0061, 0x0064, 0x0020, 0x006E, 0x0065, 0x0074, 0x006D, 0x0073,
    0x0067, 0x002E, 0x0064, 0x006C, 0x006C, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_DE49107DC011: [u16; 37] = [
    0x0054u16, 0x0068, 0x0069, 0x0073, 0x0020, 0x0063, 0x006F, 0x006D, 0x006D, 0x0061,
    0x006E, 0x0064, 0x0020, 0x0069, 0x0073, 0x0020, 0x006E, 0x006F, 0x0074, 0x0020,
    0x0069, 0x006D, 0x0070, 0x006C, 0x0065, 0x006D, 0x0065, 0x006E, 0x0074, 0x0065,
    0x0064, 0x0020, 0x0079, 0x0065, 0x0074, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_EB66FEF147EA: [u16; 12] = [
    0x005Cu16, 0x006E, 0x0065, 0x0074, 0x006D, 0x0073, 0x0067, 0x002E, 0x0064, 0x006C,
    0x006C, 0x0000,
];
pub static RUSTCONV_WSTR_F3C6A8F2AC47: [u16; 4] = [0x0025u16, 0x006C, 0x0075, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static RUSTCONV_C2R_WSTR_08784EFC934C: [u16; 5] = [0x0066u16, 0x0069, 0x006C, 0x0065, 0x0000];
pub static RUSTCONV_C2R_WSTR_17754B0C67BE: [u16; 9] = [0x0063u16, 0x006F, 0x006E, 0x0074, 0x0069, 0x006E, 0x0075, 0x0065, 0x0000];
pub static RUSTCONV_C2R_WSTR_188462D6BDEB: [u16; 9] = [0x0061u16, 0x0063, 0x0063, 0x006F, 0x0075, 0x006E, 0x0074, 0x0073, 0x0000];
pub static RUSTCONV_C2R_WSTR_242288A71293: [u16; 7] = [0x0063u16, 0x006F, 0x006E, 0x0066, 0x0069, 0x0067, 0x0000];
pub static RUSTCONV_C2R_WSTR_3C78D9218DED: [u16; 11] = [
    0x0073u16, 0x0074, 0x0061, 0x0074, 0x0069, 0x0073, 0x0074, 0x0069, 0x0063, 0x0073,
    0x0000,
];
pub static RUSTCONV_C2R_WSTR_41C78B5E6C8F: [u16; 11] = [
    0x006Cu16, 0x006F, 0x0063, 0x0061, 0x006C, 0x0067, 0x0072, 0x006F, 0x0075, 0x0070,
    0x0000,
];
pub static RUSTCONV_C2R_WSTR_4F9E8005A358: [u16; 6] = [0x0070u16, 0x0061, 0x0075, 0x0073, 0x0065, 0x0000];
pub static RUSTCONV_C2R_WSTR_5478FC1EF918: [u16; 7] = [0x0073u16, 0x0079, 0x006E, 0x0074, 0x0061, 0x0078, 0x0000];
pub static RUSTCONV_C2R_WSTR_568FB8CD9679: [u16; 6] = [0x0073u16, 0x0068, 0x0061, 0x0072, 0x0065, 0x0000];
pub static RUSTCONV_C2R_WSTR_5E820C407186: [u16; 9] = [0x0063u16, 0x006F, 0x006D, 0x0070, 0x0075, 0x0074, 0x0065, 0x0072, 0x0000];
pub static RUSTCONV_C2R_WSTR_6509121F99F7: [u16; 5] = [0x0068u16, 0x0065, 0x006C, 0x0070, 0x0000];
pub static RUSTCONV_C2R_WSTR_6CACA0C30AFA: [u16; 5] = [0x0075u16, 0x0073, 0x0065, 0x0072, 0x0000];
pub static RUSTCONV_C2R_WSTR_9910056FAA91: [u16; 8] = [0x0073u16, 0x0065, 0x0073, 0x0073, 0x0069, 0x006F, 0x006E, 0x0000];
pub static RUSTCONV_C2R_WSTR_9E442C20507D: [u16; 5] = [0x0074u16, 0x0069, 0x006D, 0x0065, 0x0000];
pub static RUSTCONV_C2R_WSTR_9F2D459FAC2D: [u16; 5] = [0x0073u16, 0x0074, 0x006F, 0x0070, 0x0000];
pub static RUSTCONV_C2R_WSTR_B289B6819D9A: [u16; 8] = [0x0068u16, 0x0065, 0x006C, 0x0070, 0x006D, 0x0073, 0x0067, 0x0000];
pub static RUSTCONV_C2R_WSTR_BC7846468928: [u16; 4] = [0x0075u16, 0x0073, 0x0065, 0x0000];
pub static RUSTCONV_C2R_WSTR_BEB8E1991CF9: [u16; 6] = [0x0073u16, 0x0074, 0x0061, 0x0072, 0x0074, 0x0000];
pub static RUSTCONV_C2R_WSTR_F4506A259167: [u16; 5] = [0x0076u16, 0x0069, 0x0065, 0x0077, 0x0000];
pub static RUSTCONV_C2R_WSTR_FC6CCB328C4F: [u16; 6] = [0x0067u16, 0x0072, 0x006F, 0x0075, 0x0070, 0x0000];

#[no_mangle]
pub static mut cmds: [COMMAND; 21] = unsafe {
    [
        _COMMAND {
            name: RUSTCONV_C2R_WSTR_188462D6BDEB.as_ptr() as *mut WCHAR,
            func: Some(cmdAccounts as unsafe extern "C" fn(INT, *mut *mut WCHAR) -> INT),
        },
        _COMMAND {
            name: RUSTCONV_C2R_WSTR_5E820C407186.as_ptr() as *mut WCHAR,
            func: Some(cmdComputer as unsafe extern "C" fn(INT, *mut *mut WCHAR) -> INT),
        },
        _COMMAND {
            name: RUSTCONV_C2R_WSTR_242288A71293.as_ptr() as *mut WCHAR,
            func: Some(cmdConfig as unsafe extern "C" fn(INT, *mut *mut WCHAR) -> INT),
        },
        _COMMAND {
            name: RUSTCONV_C2R_WSTR_17754B0C67BE.as_ptr() as *mut WCHAR,
            func: Some(cmdContinue as unsafe extern "C" fn(INT, *mut *mut WCHAR) -> INT),
        },
        _COMMAND {
            name: RUSTCONV_C2R_WSTR_08784EFC934C.as_ptr() as *mut WCHAR,
            func: Some(unimplemented as unsafe extern "C" fn(INT, *mut *mut WCHAR) -> INT),
        },
        _COMMAND {
            name: RUSTCONV_C2R_WSTR_FC6CCB328C4F.as_ptr() as *mut WCHAR,
            func: Some(cmdGroup as unsafe extern "C" fn(INT, *mut *mut WCHAR) -> INT),
        },
        _COMMAND {
            name: RUSTCONV_C2R_WSTR_6509121F99F7.as_ptr() as *mut WCHAR,
            func: Some(cmdHelp as unsafe extern "C" fn(INT, *mut *mut WCHAR) -> INT),
        },
        _COMMAND {
            name: RUSTCONV_C2R_WSTR_B289B6819D9A.as_ptr() as *mut WCHAR,
            func: Some(cmdHelpMsg as unsafe extern "C" fn(INT, *mut *mut WCHAR) -> INT),
        },
        _COMMAND {
            name: RUSTCONV_C2R_WSTR_41C78B5E6C8F.as_ptr() as *mut WCHAR,
            func: Some(cmdLocalGroup as unsafe extern "C" fn(INT, *mut *mut WCHAR) -> INT),
        },
        _COMMAND {
            name: RUSTCONV_C2R_WSTR_4F9E8005A358.as_ptr() as *mut WCHAR,
            func: Some(cmdPause as unsafe extern "C" fn(INT, *mut *mut WCHAR) -> INT),
        },
        _COMMAND {
            name: RUSTCONV_C2R_WSTR_9910056FAA91.as_ptr() as *mut WCHAR,
            func: Some(cmdSession as unsafe extern "C" fn(INT, *mut *mut WCHAR) -> INT),
        },
        _COMMAND {
            name: RUSTCONV_C2R_WSTR_568FB8CD9679.as_ptr() as *mut WCHAR,
            func: Some(cmdShare as unsafe extern "C" fn(INT, *mut *mut WCHAR) -> INT),
        },
        _COMMAND {
            name: RUSTCONV_C2R_WSTR_BEB8E1991CF9.as_ptr() as *mut WCHAR,
            func: Some(cmdStart as unsafe extern "C" fn(INT, *mut *mut WCHAR) -> INT),
        },
        _COMMAND {
            name: RUSTCONV_C2R_WSTR_3C78D9218DED.as_ptr() as *mut WCHAR,
            func: Some(cmdStatistics as unsafe extern "C" fn(INT, *mut *mut WCHAR) -> INT),
        },
        _COMMAND {
            name: RUSTCONV_C2R_WSTR_9F2D459FAC2D.as_ptr() as *mut WCHAR,
            func: Some(cmdStop as unsafe extern "C" fn(INT, *mut *mut WCHAR) -> INT),
        },
        _COMMAND {
            name: RUSTCONV_C2R_WSTR_5478FC1EF918.as_ptr() as *mut WCHAR,
            func: Some(cmdSyntax as unsafe extern "C" fn(INT, *mut *mut WCHAR) -> INT),
        },
        _COMMAND {
            name: RUSTCONV_C2R_WSTR_9E442C20507D.as_ptr() as *mut WCHAR,
            func: Some(unimplemented as unsafe extern "C" fn(INT, *mut *mut WCHAR) -> INT),
        },
        _COMMAND {
            name: RUSTCONV_C2R_WSTR_BC7846468928.as_ptr() as *mut WCHAR,
            func: Some(cmdUse as unsafe extern "C" fn(INT, *mut *mut WCHAR) -> INT),
        },
        _COMMAND {
            name: RUSTCONV_C2R_WSTR_6CACA0C30AFA.as_ptr() as *mut WCHAR,
            func: Some(cmdUser as unsafe extern "C" fn(INT, *mut *mut WCHAR) -> INT),
        },
        _COMMAND {
            name: RUSTCONV_C2R_WSTR_F4506A259167.as_ptr() as *mut WCHAR,
            func: Some(unimplemented as unsafe extern "C" fn(INT, *mut *mut WCHAR) -> INT),
        },
        _COMMAND {
            name: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            func: None,
        },
    ]
};

#[no_mangle]
pub static mut hModuleNetMsg: HMODULE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

pub const MAX_BUFFER_SIZE: i32 = 4096;

#[repr(C)]
pub struct _COMMAND {
    pub name: *const u16,
    pub func: Option<unsafe extern "C" fn(i32, *mut PWSTR) -> i32>,
}
pub type COMMAND = _COMMAND;
pub type PCOMMAND = *mut _COMMAND;
#[no_mangle]
pub unsafe extern "C" fn PrintPaddedResourceString(uID: UINT, nPaddedLength: DWORD) {
    let mut dwLength: DWORD;

    dwLength = ConResPuts(StdOut, uID) as DWORD;
    if dwLength < nPaddedLength {
        PrintPadding(0x0020u16, nPaddedLength - dwLength);
    }
}
#[no_mangle]
pub unsafe extern "C" fn PrintPadding(mut chr: WCHAR, mut nPaddedLength: DWORD) {
    let mut i: DWORD;
    let mut szMsgBuffer: [WCHAR; (MAX_BUFFER_SIZE) as usize] = [0; (MAX_BUFFER_SIZE) as usize];

    i = 0;
    while i < nPaddedLength {
        szMsgBuffer[i as usize] = chr;
        i = i.wrapping_add(1);
    }
    szMsgBuffer[nPaddedLength as usize] = UNICODE_NULL;

    ConPuts(StdOut, core::ptr::addr_of_mut!(szMsgBuffer) as *mut WCHAR);
}
#[no_mangle]
pub unsafe extern "C" fn TranslateAppMessage(dwMessage: DWORD) -> DWORD {
    if dwMessage == NERR_Success {
        return 3500; // APPERR_3500
    }
    if dwMessage == ERROR_MORE_DATA {
        return 3513; // APPERR_3513
    }
    return dwMessage;
}
#[no_mangle]
pub unsafe extern "C" fn PrintMessageStringV_v(dwMessage: DWORD, ap_list: *mut ::core::ffi::c_void) {
    let mut args: va_list = *(ap_list as *mut va_list);
    ConMsgPrintfV(
        StdOut,
        FORMAT_MESSAGE_FROM_HMODULE,
        hModuleNetMsg as LPCVOID,
        dwMessage,
        LANG_USER_DEFAULT,
        core::ptr::addr_of_mut!(args),
    );
}


#[no_mangle]
pub unsafe extern "C" fn PrintMessageString(mut dwMessage: DWORD) {
    ConMsgPuts(
        &raw mut csStdOut,
        FORMAT_MESSAGE_FROM_HMODULE as DWORD,
        hModuleNetMsg as LPCVOID,
        dwMessage,
        LANG_USER_DEFAULT as DWORD,
    );
}
#[no_mangle]
pub unsafe extern "C" fn PrintPaddedMessageString(mut dwMessage: DWORD, mut nPaddedLength: DWORD) {
    let mut dwLength: DWORD = 0;
    dwLength = ConMsgPuts(
        StdOut,
        FORMAT_MESSAGE_FROM_HMODULE as DWORD,
        hModuleNetMsg as LPCVOID,
        dwMessage,
        LANG_USER_DEFAULT as DWORD,
    ) as DWORD;
    if dwLength < nPaddedLength {
        PrintPadding(0x0020u16, nPaddedLength.wrapping_sub(dwLength));
    }
}
#[no_mangle]
pub unsafe extern "C" fn PrintErrorMessage(dwError: DWORD) {
    let mut szErrorBuffer: [WCHAR; 16] = [0 as WCHAR; 16];
    let mut pBuffer: PWSTR = core::ptr::null_mut::<WCHAR>();
    let mut pErrorInserts: [PWSTR; 2] = [core::ptr::null_mut::<WCHAR>(), core::ptr::null_mut::<WCHAR>()];

    if dwError >= (MIN_LANMAN_MESSAGE_ID as u32) && dwError <= MAX_LANMAN_MESSAGE_ID {
        FormatMessageW(
            FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_HMODULE | FORMAT_MESSAGE_IGNORE_INSERTS,
            hModuleNetMsg as LPCVOID,
            dwError,
            LANG_USER_DEFAULT,
            &raw mut pBuffer as LPWSTR,
            0,
            core::ptr::null_mut::<va_list>(),
        );
        if !pBuffer.is_null() {
            ConPrintf(StdErr, RUSTCONV_WSTR_4BA5D7E4F16E.as_ptr(), pBuffer);
            LocalFree(pBuffer as HLOCAL);
            pBuffer = core::ptr::null_mut::<WCHAR>();
        }
    } else {
        FormatMessageW(
            FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_IGNORE_INSERTS,
            core::ptr::null::<core::ffi::c_void>(),
            dwError,
            LANG_USER_DEFAULT,
            &raw mut pBuffer as LPWSTR,
            0,
            core::ptr::null_mut::<va_list>(),
        );
        if !pBuffer.is_null() {
            ConPrintf(StdErr, RUSTCONV_WSTR_4BA5D7E4F16E.as_ptr(), pBuffer);
            LocalFree(pBuffer as HLOCAL);
            pBuffer = core::ptr::null_mut::<WCHAR>();
        }
    }

    if dwError != ERROR_SUCCESS {
        /* Format insert for the 3514 message */
        _swprintf(szErrorBuffer.as_mut_ptr(), RUSTCONV_WSTR_F3C6A8F2AC47.as_ptr(), dwError);
        pErrorInserts[0] = szErrorBuffer.as_mut_ptr() as PWSTR;

        /* Format and print the 3514 message */
        FormatMessageW(
            FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_HMODULE | FORMAT_MESSAGE_ARGUMENT_ARRAY,
            hModuleNetMsg as LPCVOID,
            3514,
            LANG_USER_DEFAULT,
            &raw mut pBuffer as LPWSTR,
            0,
            pErrorInserts.as_mut_ptr() as *mut va_list,
        );
        if !pBuffer.is_null() {
            ConPrintf(StdErr, RUSTCONV_WSTR_4BA5D7E4F16E.as_ptr(), pBuffer);
            LocalFree(pBuffer as HLOCAL);
            pBuffer = core::ptr::null_mut::<WCHAR>();
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn PrintNetMessage(dwMessage: DWORD) {
    let mut pBuffer: PWSTR = core::ptr::null_mut();
    FormatMessageW(
        (FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_HMODULE | FORMAT_MESSAGE_IGNORE_INSERTS) as DWORD,
        GetModuleHandleW(core::ptr::null()) as LPCVOID,
        dwMessage,
        LANG_USER_DEFAULT as DWORD,
        &raw mut pBuffer as LPWSTR,
        0 as DWORD,
        core::ptr::null_mut(),
    );
    if !pBuffer.is_null() {
        ConPuts(&raw mut csStdOut, pBuffer as PCWSTR);
        ConPuts(&raw mut csStdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr() as PCWSTR);
        LocalFree(pBuffer as HLOCAL);
        pBuffer = core::ptr::null_mut();
    }
}
#[no_mangle]
pub unsafe extern "C" fn ReadFromConsole(lpInput: LPWSTR, dwLength: DWORD, bEcho: BOOL) {
    let mut dwOldMode: DWORD = core::mem::zeroed();
    let mut dwRead: DWORD = 0;
    let mut hFile: HANDLE;
    let mut p: LPWSTR;
    let mut pBuf: PCHAR;

    pBuf = HeapAlloc(GetProcessHeap(), 0, (dwLength - 1) as SIZE_T) as PCHAR;
    ZeroMemory!(lpInput as PVOID, (dwLength as SIZE_T) * (core::mem::size_of::<WCHAR>() as SIZE_T));
    hFile = GetStdHandle(STD_INPUT_HANDLE);
    GetConsoleMode(hFile, core::ptr::addr_of_mut!(dwOldMode));

    SetConsoleMode(hFile, (ENABLE_LINE_INPUT | (if bEcho != 0 { ENABLE_ECHO_INPUT } else { 0 })) as DWORD);

    ReadFile(hFile, pBuf as PVOID, dwLength - 1, core::ptr::addr_of_mut!(dwRead), core::ptr::null_mut());

    MultiByteToWideChar(CP_OEMCP, 0, pBuf as LPCSTR, dwRead as core::ffi::c_int, lpInput, (dwLength - 1) as core::ffi::c_int);
    HeapFree(GetProcessHeap(), 0, pBuf as PVOID);

    p = lpInput;
    while *p != 0 {
        if *p == 0x000Du16 {
            *p = 0x0000u16;
            break;
        } else {
            p = p.add(1);
        }
    }

    SetConsoleMode(hFile, dwOldMode);
}

#[no_mangle]
pub unsafe extern "C" fn wmain(mut argc: c_int, mut argv: *mut *mut WCHAR) -> c_int {
    let mut szDllBuffer: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut cmdptr: PCOMMAND = core::ptr::null_mut();
    let mut nResult: c_int = 0;
    let mut bRun: BOOL = (FALSE) as i32;

    /* Initialize the Console Standard Streams */
    ConInitStdStreams();

    /* Load netmsg.dll */
    GetSystemDirectoryW(
        &raw mut szDllBuffer as LPWSTR,
        ARRAYSIZE!(szDllBuffer) as UINT,
    );
    wcscat(
        &raw mut szDllBuffer as *mut WCHAR,
        RUSTCONV_WSTR_EB66FEF147EA.as_ptr(),
    );

    hModuleNetMsg = LoadLibrary(&raw mut szDllBuffer as *mut WCHAR as LPCWSTR) as HMODULE;
    if hModuleNetMsg.is_null() {
        ConPrintf(StdErr, RUSTCONV_WSTR_6A034B9F8DB0.as_ptr());
        return 1;
    }

    'done: {
        if argc < 2 {
            nResult = 1;
            break 'done;
        }

        /* Scan the command table */
        cmdptr = &raw mut cmds as PCOMMAND;
        while !(*cmdptr).name.is_null() {
            if _wcsicmp(
                *argv.offset(1 as isize),
                (*cmdptr).name,
            ) == 0
            {
                nResult = (*cmdptr).func.unwrap_unchecked()(argc as INT, argv)
                    as c_int;
                bRun = (TRUE) as i32;
                break;
            } else {
                cmdptr = cmdptr.offset(1);
            }
        }
    }

    /* done: */
    if (bRun) as u32 == FALSE {
        PrintMessageString(4381);
        ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
        PrintNetMessage(MSG_NET_SYNTAX);
    }

    if !hModuleNetMsg.is_null() {
        FreeLibrary(hModuleNetMsg);
    }

    return nResult;
}
#[no_mangle]
pub unsafe extern "C" fn unimplemented(mut argc: INT, mut argv: *mut *mut WCHAR) -> INT {
    ConPuts(&raw mut csStdOut, RUSTCONV_WSTR_DE49107DC011.as_ptr() as PCWSTR);
    return 1 as INT;
}
