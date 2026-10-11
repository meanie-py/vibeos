use crate::net::main::PrintErrorMessage;
use crate::net::main::PrintMessageString;
use crate::net::main::PrintNetMessage;
use crate::net::main::PrintPadding;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_075DE2B906DB: [u16; 2] = [0x000Au16, 0x0000];
pub static RUSTCONV_WSTR_1A749FECCEE7: [u16; 6] = [0x002Fu16, 0x0068, 0x0065, 0x006C, 0x0070, 0x0000];
pub static RUSTCONV_WSTR_3B87373938C1: [u16; 18] = [
    0x0025u16, 0x0030, 0x0032, 0x006C, 0x0075, 0x003A, 0x0025, 0x0030, 0x0032, 0x006C,
    0x0075, 0x003A, 0x0025, 0x0030, 0x0032, 0x006C, 0x0075, 0x0000,
];
pub static RUSTCONV_WSTR_5FD7D108481C: [u16; 33] = [
    0x0053u16, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x0020, 0x0065, 0x0072, 0x0072,
    0x006F, 0x0072, 0x0020, 0x0025, 0x006C, 0x0075, 0x0020, 0x0068, 0x0061, 0x0073,
    0x0020, 0x006F, 0x0063, 0x0063, 0x0075, 0x0072, 0x0072, 0x0065, 0x0064, 0x002E,
    0x000A, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_62636DC08541: [u16; 41] = [
    0x0025u16, 0x002D, 0x0032, 0x0032, 0x002E, 0x0032, 0x0032, 0x0073, 0x0020, 0x0025,
    0x002D, 0x0032, 0x0030, 0x002E, 0x0032, 0x0030, 0x0073, 0x0020, 0x0025, 0x002D,
    0x0031, 0x0037, 0x002E, 0x0031, 0x0037, 0x0073, 0x0020, 0x0025, 0x002D, 0x0035,
    0x006C, 0x0075, 0x0020, 0x0025, 0x002D, 0x0038, 0x002E, 0x0038, 0x0073, 0x000A,
    0x0000,
];
pub static RUSTCONV_WSTR_7E7219CF4FF7: [u16; 8] = [0x002Fu16, 0x0064, 0x0065, 0x006C, 0x0065, 0x0074, 0x0065, 0x0000];
pub static RUSTCONV_WSTR_A441D303E301: [u16; 6] = [0x002Fu16, 0x006C, 0x0069, 0x0073, 0x0074, 0x0000];

use core::ffi::c_void;

unsafe extern "C" fn SecondsToDurationString(
    mut DurationString: PWSTR,
    mut DurationStringSize: size_t,
    mut dwDuration: DWORD,
) {
    let mut dwHours: DWORD = 0;
    let mut dwRemainingSeconds: DWORD = 0;
    let mut dwMinutes: DWORD = 0;
    let mut dwSeconds: DWORD = 0;

    dwHours = dwDuration / 3600;
    dwRemainingSeconds = dwDuration % 3600;
    dwMinutes = dwRemainingSeconds / 60;
    dwSeconds = dwRemainingSeconds % 60;

    StringCchPrintfW!(DurationString, DurationStringSize, RUSTCONV_WSTR_3B87373938C1.as_ptr(), dwHours, dwMinutes, dwSeconds);
}
#[no_mangle]
pub unsafe extern "C" fn EnumSessions(mut pszComputerName: PWSTR, mut bList: BOOL) -> DWORD {
    let mut pBuffer: PSESSION_INFO_2 = core::ptr::null_mut();
    let mut DurationBuffer: [WCHAR; 10] = [0; 10];
    let mut dwRead: DWORD = 0;
    let mut dwTotal: DWORD = 0;
    let mut i: DWORD = 0;
    let mut ResumeHandle: DWORD = 0;
    let mut Status: DWORD;

    Status = NetSessionEnum(
        pszComputerName,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        2,
        core::ptr::addr_of_mut!(pBuffer) as *mut LPBYTE,
        MAX_PREFERRED_LENGTH,
        core::ptr::addr_of_mut!(dwRead),
        core::ptr::addr_of_mut!(dwTotal),
        core::ptr::addr_of_mut!(ResumeHandle),
    );
    if (Status != 0) && (Status != ERROR_MORE_DATA) {
        // PrintMessageStringV(3502, Status);
        ConPrintf(
            core::ptr::addr_of_mut!(csStdOut),
            RUSTCONV_WSTR_5FD7D108481C.as_ptr(),
            Status,
        );
        return Status;
    }

    if dwTotal == 0 {
        PrintMessageString(3683);
    } else {
        ConPuts(
            core::ptr::addr_of_mut!(csStdOut),
            RUSTCONV_WSTR_075DE2B906DB.as_ptr(),
        );
        PrintMessageString(4750);
        PrintPadding(0x002Du16, 79);
        ConPuts(
            core::ptr::addr_of_mut!(csStdOut),
            RUSTCONV_WSTR_075DE2B906DB.as_ptr(),
        );

        i = 0;
        while i < dwRead {
            if !(*pBuffer.offset(i as isize)).sesi2_cname.is_null() {
                SecondsToDurationString(
                    DurationBuffer.as_mut_ptr(),
                    (ARRAYSIZE!(DurationBuffer)) as size_t,
                    (*pBuffer.offset(i as isize)).sesi2_idle_time,
                );

                ConPrintf(
                    core::ptr::addr_of_mut!(csStdOut),
                    RUSTCONV_WSTR_62636DC08541.as_ptr(),
                    (*pBuffer.offset(i as isize)).sesi2_cname,
                    (*pBuffer.offset(i as isize)).sesi2_username,
                    (*pBuffer.offset(i as isize)).sesi2_cltype_name,
                    (*pBuffer.offset(i as isize)).sesi2_num_opens,
                    DurationBuffer.as_mut_ptr(),
                );
            }
            i = i.wrapping_add(1);
        }
    }

    NetApiBufferFree(pBuffer as PVOID);

    return 0;
}
#[no_mangle]
pub unsafe extern "C" fn cmdSession(argc: INT, argv: *mut *mut WCHAR) -> INT {
    let mut pszComputerName: PWSTR = core::ptr::null_mut();
    let mut bList: BOOL = (FALSE) as i32;
    let mut bDelete: BOOL = (FALSE) as i32;
    let mut i: INT = 0;
    let mut Status: NET_API_STATUS;
    let mut result: INT = 0;

    i = 2;
    while i < argc {
        if *(*argv.offset(i as isize)).offset(0 as isize) == 0x005Cu16
            && *(*argv.offset(i as isize)).offset(1 as isize) == 0x005Cu16
            && pszComputerName.is_null()
        {
            pszComputerName = *argv.offset(i as isize);
            i += 1;
        } else if _wcsicmp(*argv.offset(i as isize), RUSTCONV_WSTR_A441D303E301.as_ptr()) == 0 {
            bList = (TRUE) as i32;
        } else if _wcsicmp(*argv.offset(i as isize), RUSTCONV_WSTR_7E7219CF4FF7.as_ptr()) == 0 {
            bDelete = (TRUE) as i32;
        } else if _wcsicmp(*argv.offset(i as isize), RUSTCONV_WSTR_1A749FECCEE7.as_ptr()) == 0 {
            PrintMessageString(4381);
            ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
            PrintNetMessage(MSG_SESSION_SYNTAX);
            PrintNetMessage(MSG_SESSION_HELP);
            return 0;
        } else {
            PrintMessageString(4381);
            ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
            PrintNetMessage(MSG_SESSION_SYNTAX);
            return 1;
        }
        i += 1;
    }

    if bDelete != 0 {
        Status = NetSessionDel(pszComputerName, core::ptr::null_mut(), core::ptr::null_mut());
    } else {
        Status = EnumSessions(pszComputerName, bList);
    }

    if Status == NERR_Success {
        PrintErrorMessage(ERROR_SUCCESS);
    } else {
        PrintErrorMessage(Status);
        result = 1;
    }

    return result;
}
