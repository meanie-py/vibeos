use crate::net::main::PrintErrorMessage;
use crate::net::main::PrintMessageString;
use crate::net::main::PrintNetMessage;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_075DE2B906DB: [u16; 2] = [0x000Au16, 0x0000];
pub static RUSTCONV_WSTR_1A749FECCEE7: [u16; 6] = [0x002Fu16, 0x0068, 0x0065, 0x006C, 0x0070, 0x0000];
pub static RUSTCONV_WSTR_28B08D2B6613: [u16; 5] = [0x002Fu16, 0x0064, 0x0065, 0x006C, 0x0000];
pub static RUSTCONV_WSTR_5D12D1C1410E: [u16; 5] = [0x002Fu16, 0x0061, 0x0064, 0x0064, 0x0000];
pub static RUSTCONV_WSTR_6509121F99F7: [u16; 5] = [0x0068u16, 0x0065, 0x006C, 0x0070, 0x0000];
pub static RUSTCONV_WSTR_DE6D696C194F: [u16; 2] = [0x0024u16, 0x0000];

use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn cmdComputer(mut argc: INT, mut argv: *mut *mut WCHAR) -> INT {
    let mut ComputerAccountName: [WCHAR; (MAX_PATH + 2) as _] = [0; (MAX_PATH + 2) as _];
    let mut ComputerPassword: [WCHAR; (LM20_PWLEN + 1) as _] = [0; (LM20_PWLEN + 1) as _];
    let mut UserInfo: USER_INFO_1 = core::mem::zeroed();
    let mut i: INT;
    let mut result: INT = 0;
    let mut bAdd: BOOL = (FALSE) as i32;
    let mut bDelete: BOOL = (FALSE) as i32;
    let mut pComputerName: PWSTR = core::ptr::null_mut();
    let mut Status: NET_API_STATUS = NERR_Success;
    /*
    OSVERSIONINFOEX VersionInfo;

    VersionInfo.dwOSVersionInfoSize = sizeof(VersionInfo);
    if (!GetVersionEx((LPOSVERSIONINFO)&VersionInfo))
    {
        PrintErrorMessage(GetLastError());
        return 1;
    }

    if (VersionInfo.wProductType != VER_NT_DOMAIN_CONTROLLER)
    {
        PrintErrorMessage(3515);
        return 1;
    }
    */

    i = 2;
    if argc > 2
        && *(*argv.offset(i as isize)).offset(0 as isize) == 0x005Cu16
        && *(*argv.offset(i as isize)).offset(1 as isize) == 0x005Cu16
    {
        pComputerName = *argv.offset(i as isize);
        i += 1;
    }

    while i < argc {
        if _wcsicmp(*argv.offset(i as isize), RUSTCONV_WSTR_6509121F99F7.as_ptr()) == 0
        {
            /* Print short syntax help */
            PrintMessageString(4381);
            ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
            PrintNetMessage(MSG_COMPUTER_SYNTAX);
            return 0;
        }

        if _wcsicmp(*argv.offset(i as isize), RUSTCONV_WSTR_1A749FECCEE7.as_ptr()) == 0
        {
            /* Print full help text*/
            PrintMessageString(4381);
            ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
            PrintNetMessage(MSG_COMPUTER_SYNTAX);
            PrintNetMessage(MSG_COMPUTER_HELP);
            return 0;
        }

        if _wcsicmp(*argv.offset(i as isize), RUSTCONV_WSTR_5D12D1C1410E.as_ptr()) == 0
        {
            bAdd = (TRUE) as i32;
            i += 1;
            continue;
        } else if _wcsicmp(*argv.offset(i as isize), RUSTCONV_WSTR_28B08D2B6613.as_ptr()) == 0
        {
            bDelete = (TRUE) as i32;
            i += 1;
            continue;
        } else {
            PrintErrorMessage(3506/*, argv[i]*/);
            return 1;
        }
    }

    if pComputerName.is_null()
        || ((bAdd) as u32 == FALSE && (bDelete) as u32 == FALSE)
        || ((bAdd) as u32 == TRUE && (bDelete) as u32 == TRUE)
    {
        PrintMessageString(4381);
        ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
        PrintNetMessage(MSG_COMPUTER_SYNTAX);
        return 1;
    }

    /*
     * Create the computer account name:
     *  Skip the leading '\\' and appand a '$'.
     */
    wcscpy(
        &raw mut ComputerAccountName as *mut WCHAR,
        pComputerName.offset(2 as isize),
    );
    wcscat(&raw mut ComputerAccountName as *mut WCHAR, RUSTCONV_WSTR_DE6D696C194F.as_ptr());

    if bAdd != 0 {
        /*
         * Create the computer password:
         *   Skip the leading '\\', shorten to a maximum of 14 characters
         *   and convert to lower case
         */
        wcsncpy(
            &raw mut ComputerPassword as *mut WCHAR,
            pComputerName.offset(2 as isize),
            (LM20_PWLEN as usize),
        );
        ComputerPassword[LM20_PWLEN as usize] = UNICODE_NULL;
        _wcslwr(&raw mut ComputerPassword as *mut WCHAR);

        /* Set the account data */
        UserInfo.usri1_name = &raw mut ComputerAccountName as *mut WCHAR as LPWSTR;
        UserInfo.usri1_password = &raw mut ComputerPassword as *mut WCHAR as LPWSTR;
        UserInfo.usri1_password_age = 0;
        UserInfo.usri1_priv = USER_PRIV_USER;
        UserInfo.usri1_home_dir = core::ptr::null_mut();
        UserInfo.usri1_comment = core::ptr::null_mut();
        UserInfo.usri1_flags = UF_SCRIPT | UF_WORKSTATION_TRUST_ACCOUNT;
        UserInfo.usri1_script_path = core::ptr::null_mut();

        /* Add the computer account */
        Status = NetUserAdd(
            core::ptr::null_mut(),
            1,
            &raw mut UserInfo as LPBYTE,
            core::ptr::null_mut(),
        );
    } else if bDelete != 0 {
        /* Delete the coputer account */
        Status = NetUserDel(
            core::ptr::null_mut(),
            &raw mut ComputerAccountName as *mut WCHAR,
        );
    }

    if Status == NERR_Success {
        PrintErrorMessage(ERROR_SUCCESS);
    } else {
        PrintErrorMessage(Status);
        result = 1;
    }

    return result;
}
