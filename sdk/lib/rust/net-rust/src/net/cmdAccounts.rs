use crate::net::main::PrintMessageString;
use crate::net::main::PrintNetMessage;
use crate::net::main::PrintPaddedMessageString;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_075DE2B906DB: [u16; 2] = [0x000Au16, 0x0000];
pub static RUSTCONV_WSTR_1A749FECCEE7: [u16; 6] = [0x002Fu16, 0x0068, 0x0065, 0x006C, 0x0070, 0x0000];
pub static RUSTCONV_WSTR_209DAB04A41C: [u16; 10] = [0x002Fu16, 0x004D, 0x0041, 0x0058, 0x0050, 0x0057, 0x004C, 0x0045, 0x004E, 0x0000];
pub static RUSTCONV_WSTR_29E4CF19D7D8: [u16; 10] = [0x002Fu16, 0x004D, 0x0049, 0x004E, 0x0050, 0x0057, 0x004C, 0x0045, 0x004E, 0x0000];
pub static RUSTCONV_WSTR_34FF81DA9F6B: [u16; 11] = [
    0x002Fu16, 0x006D, 0x0069, 0x006E, 0x0070, 0x0077, 0x0061, 0x0067, 0x0065, 0x003A,
    0x0000,
];
pub static RUSTCONV_WSTR_3D8DEDAB2E93: [u16; 14] = [
    0x002Fu16, 0x0066, 0x006F, 0x0072, 0x0063, 0x0065, 0x006C, 0x006F, 0x0067, 0x006F,
    0x0066, 0x0066, 0x003A, 0x0000,
];
pub static RUSTCONV_WSTR_582BA4648587: [u16; 42] = [
    0x0054u16, 0x0068, 0x0065, 0x0020, 0x002F, 0x0044, 0x004F, 0x004D, 0x0041, 0x0049,
    0x004E, 0x0020, 0x006F, 0x0070, 0x0074, 0x0069, 0x006F, 0x006E, 0x0020, 0x0069,
    0x0073, 0x0020, 0x006E, 0x006F, 0x0074, 0x0020, 0x0073, 0x0075, 0x0070, 0x0070,
    0x006F, 0x0072, 0x0074, 0x0065, 0x0064, 0x0020, 0x0079, 0x0065, 0x0074, 0x002E,
    0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_58469F90D8DA: [u16; 10] = [0x002Fu16, 0x004D, 0x0049, 0x004E, 0x0050, 0x0057, 0x0041, 0x0047, 0x0045, 0x0000];
pub static RUSTCONV_WSTR_6509121F99F7: [u16; 5] = [0x0068u16, 0x0065, 0x006C, 0x0070, 0x0000];
pub static RUSTCONV_WSTR_833AD7EF6992: [u16; 3] = [0x006Eu16, 0x006F, 0x0000];
pub static RUSTCONV_WSTR_99397581246D: [u16; 5] = [0x0025u16, 0x006C, 0x0075, 0x000A, 0x0000];
pub static RUSTCONV_WSTR_9BB9C2E30775: [u16; 8] = [0x002Fu16, 0x0064, 0x006F, 0x006D, 0x0061, 0x0069, 0x006E, 0x0000];
pub static RUSTCONV_WSTR_9D41BE8E6281: [u16; 11] = [
    0x002Fu16, 0x006D, 0x0061, 0x0078, 0x0070, 0x0077, 0x0061, 0x0067, 0x0065, 0x003A,
    0x0000,
];
pub static RUSTCONV_WSTR_A34B1A691119: [u16; 10] = [0x002Fu16, 0x0055, 0x004E, 0x0049, 0x0051, 0x0055, 0x0045, 0x0050, 0x0057, 0x0000];
pub static RUSTCONV_WSTR_C0C460A19204: [u16; 11] = [
    0x002Fu16, 0x0075, 0x006E, 0x0069, 0x0071, 0x0075, 0x0065, 0x0070, 0x0077, 0x003A,
    0x0000,
];
pub static RUSTCONV_WSTR_C61098AC3158: [u16; 10] = [0x0075u16, 0x006E, 0x006C, 0x0069, 0x006D, 0x0069, 0x0074, 0x0065, 0x0064, 0x0000];
pub static RUSTCONV_WSTR_D6B15A6F44A6: [u16; 11] = [
    0x002Fu16, 0x006D, 0x0069, 0x006E, 0x0070, 0x0077, 0x006C, 0x0065, 0x006E, 0x003A,
    0x0000,
];
pub static RUSTCONV_WSTR_EBC5772084BE: [u16; 13] = [
    0x002Fu16, 0x0046, 0x004F, 0x0052, 0x0043, 0x0045, 0x004C, 0x004F, 0x0047, 0x004F,
    0x0046, 0x0046, 0x0000,
];
pub static RUSTCONV_WSTR_F3C6A8F2AC47: [u16; 4] = [0x0025u16, 0x006C, 0x0075, 0x0000];

use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn cmdAccounts(argc: INT, argv: *mut *mut WCHAR) -> INT {
    let mut Info0: PUSER_MODALS_INFO_0 = core::ptr::null_mut();
    let mut Info1: PUSER_MODALS_INFO_1 = core::ptr::null_mut();
    let mut Info3: PUSER_MODALS_INFO_3 = core::ptr::null_mut();
    let mut ProductType: NT_PRODUCT_TYPE = NtProductWinNt;
    let mut p: LPWSTR;
    let mut endptr: LPWSTR = core::ptr::null_mut();
    let mut ParamErr: DWORD = 0;
    let mut value: ULONG = 0;
    let mut i: INT;
    let mut Modified: BOOL = (FALSE) as i32;
    let nPaddedLength: DWORD = 58;
    let mut Status: NET_API_STATUS;
    let mut result: INT = 0;

    i = 2;
    while i < argc {
        if _wcsicmp(*argv.offset(i as isize), RUSTCONV_WSTR_6509121F99F7.as_ptr()) == 0
        {
            /* Print short syntax help */
            PrintMessageString(4381);
            ConPuts(core::ptr::addr_of_mut!(csStdOut), RUSTCONV_WSTR_075DE2B906DB.as_ptr());
            PrintNetMessage(MSG_ACCOUNTS_SYNTAX);
            return 0;
        }

        if _wcsicmp(*argv.offset(i as isize), RUSTCONV_WSTR_1A749FECCEE7.as_ptr()) == 0
        {
            /* Print full help text*/
            PrintMessageString(4381);
            ConPuts(core::ptr::addr_of_mut!(csStdOut), RUSTCONV_WSTR_075DE2B906DB.as_ptr());
            PrintNetMessage(MSG_ACCOUNTS_SYNTAX);
            PrintNetMessage(MSG_ACCOUNTS_HELP);
            return 0;
        }

        if _wcsicmp(*argv.offset(i as isize), RUSTCONV_WSTR_9BB9C2E30775.as_ptr()) == 0
        {
            ConPuts(core::ptr::addr_of_mut!(csStdErr), RUSTCONV_WSTR_582BA4648587.as_ptr());
        }
        i += 1;
    }

    'done: {
        Status = NetUserModalsGet(
            core::ptr::null(),
            0,
            core::ptr::addr_of_mut!(Info0) as *mut LPBYTE,
        );
        if Status != NERR_Success {
            break 'done;
        }

        i = 2;
        while i < argc {
            if _wcsnicmp(
                *argv.offset(i as isize),
                RUSTCONV_WSTR_3D8DEDAB2E93.as_ptr(),
                13,
            ) == 0
            {
                p = (*argv.offset(i as isize)).offset(13 as isize);
                if _wcsicmp(p, RUSTCONV_WSTR_833AD7EF6992.as_ptr()) == 0 {
                    (*Info0).usrmod0_force_logoff = TIMEQ_FOREVER;
                    Modified = (TRUE) as i32;
                } else {
                    value = wcstoul(p, core::ptr::addr_of_mut!(endptr), 10);
                    if *endptr != 0 {
                        PrintMessageStringV(3952, RUSTCONV_WSTR_EBC5772084BE.as_ptr());
                        result = 1;
                        break 'done;
                    }

                    (*Info0).usrmod0_force_logoff = value.wrapping_mul(60);
                    Modified = (TRUE) as i32;
                }
            } else if _wcsnicmp(
                *argv.offset(i as isize),
                RUSTCONV_WSTR_D6B15A6F44A6.as_ptr(),
                10,
            ) == 0
            {
                p = (*argv.offset(i as isize)).offset(10 as isize);
                value = wcstoul(p, core::ptr::addr_of_mut!(endptr), 10);
                if *endptr != 0 {
                    PrintMessageStringV(3952, RUSTCONV_WSTR_29E4CF19D7D8.as_ptr());
                    result = 1;
                    break 'done;
                }

                (*Info0).usrmod0_min_passwd_len = value;
                Modified = (TRUE) as i32;
            } else if _wcsnicmp(
                *argv.offset(i as isize),
                RUSTCONV_WSTR_9D41BE8E6281.as_ptr(),
                10,
            ) == 0
            {
                p = (*argv.offset(i as isize)).offset(10 as isize);

                if _wcsicmp(p, RUSTCONV_WSTR_C61098AC3158.as_ptr()) != 0 {
                    (*Info0).usrmod0_max_passwd_age = ULONG_MAX;
                    Modified = (TRUE) as i32;
                } else {
                    value = wcstoul(p, core::ptr::addr_of_mut!(endptr), 10);
                    if *endptr != 0 {
                        PrintMessageStringV(3952, RUSTCONV_WSTR_209DAB04A41C.as_ptr());
                        result = 1;
                        break 'done;
                    }

                    (*Info0).usrmod0_max_passwd_age = value.wrapping_mul(86400);
                    Modified = (TRUE) as i32;
                }
            } else if _wcsnicmp(
                *argv.offset(i as isize),
                RUSTCONV_WSTR_34FF81DA9F6B.as_ptr(),
                10,
            ) == 0
            {
                p = (*argv.offset(i as isize)).offset(10 as isize);
                value = wcstoul(p, core::ptr::addr_of_mut!(endptr), 10);
                if *endptr != 0 {
                    PrintMessageStringV(3952, RUSTCONV_WSTR_58469F90D8DA.as_ptr());
                    result = 1;
                    break 'done;
                }

                (*Info0).usrmod0_min_passwd_age = value.wrapping_mul(86400);
                Modified = (TRUE) as i32;
            } else if _wcsnicmp(
                *argv.offset(i as isize),
                RUSTCONV_WSTR_C0C460A19204.as_ptr(),
                10,
            ) == 0
            {
                p = (*argv.offset(i as isize)).offset(10 as isize);
                value = wcstoul(p, core::ptr::addr_of_mut!(endptr), 10);
                if *endptr != 0 {
                    PrintMessageStringV(3952, RUSTCONV_WSTR_A34B1A691119.as_ptr());
                    result = 1;
                    break 'done;
                }

                (*Info0).usrmod0_password_hist_len = value;
                Modified = (TRUE) as i32;
            }
            i += 1;
        }

        if (Modified) as u32 == TRUE {
            Status = NetUserModalsSet(
                core::ptr::null(),
                0,
                Info0 as LPBYTE,
                core::ptr::addr_of_mut!(ParamErr),
            );
            if Status != NERR_Success {
                break 'done;
            }
        } else {
            Status = NetUserModalsGet(
                core::ptr::null(),
                1,
                core::ptr::addr_of_mut!(Info1) as *mut LPBYTE,
            );
            if Status != NERR_Success {
                break 'done;
            }

            Status = NetUserModalsGet(
                core::ptr::null(),
                3,
                core::ptr::addr_of_mut!(Info3) as *mut LPBYTE,
            );
            if Status != NERR_Success {
                break 'done;
            }

            RtlGetNtProductType(core::ptr::addr_of_mut!(ProductType));

            PrintPaddedMessageString(4570, nPaddedLength);
            if (*Info0).usrmod0_force_logoff == TIMEQ_FOREVER {
                PrintMessageString(4305);
            } else {
                ConPrintf(
                    core::ptr::addr_of_mut!(csStdOut),
                    RUSTCONV_WSTR_F3C6A8F2AC47.as_ptr(),
                    (*Info0).usrmod0_force_logoff,
                );
            }
            ConPuts(core::ptr::addr_of_mut!(csStdOut), RUSTCONV_WSTR_075DE2B906DB.as_ptr());

            PrintPaddedMessageString(4572, nPaddedLength);
            ConPrintf(
                core::ptr::addr_of_mut!(csStdOut),
                RUSTCONV_WSTR_99397581246D.as_ptr(),
                (*Info0).usrmod0_min_passwd_age.wrapping_div(86400),
            );

            PrintPaddedMessageString(4573, nPaddedLength);
            ConPrintf(
                core::ptr::addr_of_mut!(csStdOut),
                RUSTCONV_WSTR_99397581246D.as_ptr(),
                (*Info0).usrmod0_max_passwd_age.wrapping_div(86400),
            );

            PrintPaddedMessageString(4574, nPaddedLength);
            ConPrintf(
                core::ptr::addr_of_mut!(csStdOut),
                RUSTCONV_WSTR_99397581246D.as_ptr(),
                (*Info0).usrmod0_min_passwd_len,
            );

            PrintPaddedMessageString(4575, nPaddedLength);
            if (*Info0).usrmod0_password_hist_len == 0 {
                PrintMessageString(4303);
            } else {
                ConPrintf(
                    core::ptr::addr_of_mut!(csStdOut),
                    RUSTCONV_WSTR_F3C6A8F2AC47.as_ptr(),
                    (*Info0).usrmod0_password_hist_len,
                );
            }
            ConPuts(core::ptr::addr_of_mut!(csStdOut), RUSTCONV_WSTR_075DE2B906DB.as_ptr());

            PrintPaddedMessageString(4578, nPaddedLength);
            if (*Info3).usrmod3_lockout_threshold == 0 {
                PrintMessageString(4305);
            } else {
                ConPrintf(
                    core::ptr::addr_of_mut!(csStdOut),
                    RUSTCONV_WSTR_F3C6A8F2AC47.as_ptr(),
                    (*Info3).usrmod3_lockout_threshold,
                );
            }
            ConPuts(core::ptr::addr_of_mut!(csStdOut), RUSTCONV_WSTR_075DE2B906DB.as_ptr());

            PrintPaddedMessageString(4579, nPaddedLength);
            ConPrintf(
                core::ptr::addr_of_mut!(csStdOut),
                RUSTCONV_WSTR_99397581246D.as_ptr(),
                (*Info3).usrmod3_lockout_duration.wrapping_div(60),
            );

            PrintPaddedMessageString(4580, nPaddedLength);
            ConPrintf(
                core::ptr::addr_of_mut!(csStdOut),
                RUSTCONV_WSTR_99397581246D.as_ptr(),
                (*Info3).usrmod3_lockout_observation_window.wrapping_div(60),
            );

            PrintPaddedMessageString(4576, nPaddedLength);
            if (*Info1).usrmod1_role == UAS_ROLE_PRIMARY {
                if ProductType as u32 == NtProductLanManNt {
                    PrintMessageString(5070);
                } else if ProductType as u32 == NtProductServer {
                    PrintMessageString(5073);
                } else {
                    PrintMessageString(5072);
                }
            } else {
                PrintMessageString(5071);
            }
            ConPuts(core::ptr::addr_of_mut!(csStdOut), RUSTCONV_WSTR_075DE2B906DB.as_ptr());
        }
    }

    // done:
    if !Info3.is_null() {
        NetApiBufferFree(Info3 as PVOID);
    }

    if !Info1.is_null() {
        NetApiBufferFree(Info1 as PVOID);
    }

    if !Info0.is_null() {
        NetApiBufferFree(Info0 as PVOID);
    }

    return result;
}
