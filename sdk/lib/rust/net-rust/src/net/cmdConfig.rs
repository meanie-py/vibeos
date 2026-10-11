use crate::net::main::PrintErrorMessage;
use crate::net::main::PrintMessageString;
use crate::net::main::PrintNetMessage;
use crate::net::main::PrintPaddedMessageString;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_075DE2B906DB: [u16; 2] = [0x000Au16, 0x0000];
pub static RUSTCONV_WSTR_0903D887FBED: [u16; 17] = [
    0x002Fu16, 0x0061, 0x0075, 0x0074, 0x006F, 0x0064, 0x0069, 0x0073, 0x0063, 0x006F,
    0x006E, 0x006E, 0x0065, 0x0063, 0x0074, 0x003A, 0x0000,
];
pub static RUSTCONV_WSTR_1A749FECCEE7: [u16; 6] = [0x002Fu16, 0x0068, 0x0065, 0x006C, 0x0070, 0x0000];
pub static RUSTCONV_WSTR_2D08E5CF381F: [u16; 11] = [
    0x0020u16, 0x0020, 0x0020, 0x0053, 0x0065, 0x0072, 0x0076, 0x0065, 0x0072, 0x000A,
    0x0000,
];
pub static RUSTCONV_WSTR_2FBF874A4098: [u16; 7] = [0x0073u16, 0x0065, 0x0072, 0x0076, 0x0065, 0x0072, 0x0000];
pub static RUSTCONV_WSTR_4B9295395356: [u16; 15] = [
    0x0020u16, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0025, 0x0073, 0x0020, 0x0028,
    0x0025, 0x0073, 0x0029, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_4BA5D7E4F16E: [u16; 4] = [0x0025u16, 0x0073, 0x000A, 0x0000];
pub static RUSTCONV_WSTR_5BFB8DE9D124: [u16; 5] = [0x002Eu16, 0x002E, 0x002E, 0x000A, 0x0000];
pub static RUSTCONV_WSTR_6509121F99F7: [u16; 5] = [0x0068u16, 0x0065, 0x006C, 0x0070, 0x0000];
pub static RUSTCONV_WSTR_6D1618CC9381: [u16; 4] = [0x0079u16, 0x0065, 0x0073, 0x0000];
pub static RUSTCONV_WSTR_833AD7EF6992: [u16; 3] = [0x006Eu16, 0x006F, 0x0000];
pub static RUSTCONV_WSTR_91EE49139047: [u16; 9] = [0x0025u16, 0x006C, 0x0075, 0x002E, 0x0025, 0x006C, 0x0075, 0x000A, 0x0000];
pub static RUSTCONV_WSTR_99397581246D: [u16; 5] = [0x0025u16, 0x006C, 0x0075, 0x000A, 0x0000];
pub static RUSTCONV_WSTR_9D907FB934C0: [u16; 13] = [
    0x002Fu16, 0x0073, 0x0072, 0x0076, 0x0063, 0x006F, 0x006D, 0x006D, 0x0065, 0x006E,
    0x0074, 0x003A, 0x0000,
];
pub static RUSTCONV_WSTR_A6777530FA58: [u16; 16] = [
    0x002Fu16, 0x0041, 0x0055, 0x0054, 0x004F, 0x0044, 0x0049, 0x0053, 0x0043, 0x004F,
    0x004E, 0x004E, 0x0045, 0x0043, 0x0054, 0x0000,
];
pub static RUSTCONV_WSTR_A9EAD68DFCD4: [u16; 9] = [0x002Fu16, 0x0068, 0x0069, 0x0064, 0x0064, 0x0065, 0x006E, 0x003A, 0x0000];
pub static RUSTCONV_WSTR_C3D39A2E4D96: [u16; 6] = [0x005Cu16, 0x005C, 0x0025, 0x0073, 0x000A, 0x0000];
pub static RUSTCONV_WSTR_CA01375376F7: [u16; 12] = [
    0x0077u16, 0x006F, 0x0072, 0x006B, 0x0073, 0x0074, 0x0061, 0x0074, 0x0069, 0x006F,
    0x006E, 0x0000,
];
pub static RUSTCONV_WSTR_DCE10C604406: [u16; 8] = [0x002Fu16, 0x0048, 0x0049, 0x0044, 0x0044, 0x0045, 0x004E, 0x0000];
pub static RUSTCONV_WSTR_E92A90E19836: [u16; 16] = [
    0x0020u16, 0x0020, 0x0020, 0x0057, 0x006F, 0x0072, 0x006B, 0x0073, 0x0074, 0x0061,
    0x0074, 0x0069, 0x006F, 0x006E, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_F5E1A713F1CC: [u16; 6] = [0x002Eu16, 0x002E, 0x002E, 0x000A, 0x000A, 0x0000];
pub static RUSTCONV_WSTR_FD671CC5B2EA: [u16; 5] = [0x0025u16, 0x0073, 0x000A, 0x000A, 0x0000];

use core::ffi::c_void;

unsafe extern "C" fn DisplayServerConfig(ServerInfo: PSERVER_INFO_102) -> INT {
    let mut TransportInfo: PSERVER_TRANSPORT_INFO_0 = core::ptr::null_mut();
    let mut dwRead: DWORD = 0;
    let mut dwTotal: DWORD = 0;
    let mut i: DWORD = 0;
    let nPaddedLength: DWORD = 38;
    let mut Status: NET_API_STATUS;

    'done: {
        Status = NetServerTransportEnum(
            core::ptr::null_mut(),
            0,
            core::ptr::addr_of_mut!(TransportInfo) as *mut PBYTE,
            MAX_PREFERRED_LENGTH,
            core::ptr::addr_of_mut!(dwRead),
            core::ptr::addr_of_mut!(dwTotal),
            core::ptr::null_mut(),
        );
        if Status != NERR_Success {
            break 'done;
        }

        PrintPaddedMessageString(4481, nPaddedLength);
        ConPrintf(StdOut, RUSTCONV_WSTR_C3D39A2E4D96.as_ptr(), (*ServerInfo).sv102_name);

        PrintPaddedMessageString(4482, nPaddedLength);
        ConPrintf(StdOut, RUSTCONV_WSTR_FD671CC5B2EA.as_ptr(), (*ServerInfo).sv102_comment);

        PrintPaddedMessageString(4484, nPaddedLength);
        ConPrintf(
            StdOut,
            RUSTCONV_WSTR_91EE49139047.as_ptr(),
            (*ServerInfo).sv102_version_major,
            (*ServerInfo).sv102_version_minor,
        );

        PrintPaddedMessageString(4489, nPaddedLength);
        ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
        i = 0;
        while i < dwRead {
            ConPrintf(
                StdOut,
                RUSTCONV_WSTR_4B9295395356.as_ptr(),
                (*TransportInfo.add(i as usize)).svti0_transportname.add(8),
                (*TransportInfo.add(i as usize)).svti0_networkaddress,
            );
            i += 1;
        }
        ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());

        PrintPaddedMessageString(4492, nPaddedLength);
        PrintMessageString(
            if (*ServerInfo).sv102_hidden == (SV_HIDDEN as i32) {
                4300
            } else {
                4301
            },
        );
        ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());

        PrintPaddedMessageString(4506, nPaddedLength);
        ConPrintf(StdOut, RUSTCONV_WSTR_99397581246D.as_ptr(), (*ServerInfo).sv102_users);

        PrintPaddedMessageString(4511, nPaddedLength);
        ConPuts(StdOut, RUSTCONV_WSTR_F5E1A713F1CC.as_ptr());

        PrintPaddedMessageString(4520, nPaddedLength);
        if (*ServerInfo).sv102_disc == SV_NODISC {
            PrintMessageString(4306);
        } else {
            ConPrintf(StdOut, RUSTCONV_WSTR_99397581246D.as_ptr(), (*ServerInfo).sv102_disc);
        }
    }

    if !TransportInfo.is_null() {
        NetApiBufferFree(TransportInfo as PVOID);
    }

    return 0;
}

unsafe extern "C" fn DisplayWorkstationConfig() -> INT {
    let mut WorkstationInfo: PWKSTA_INFO_100 = core::ptr::null_mut();
    let mut UserInfo: PWKSTA_USER_INFO_1 = core::ptr::null_mut();
    let mut TransportInfo: PWKSTA_TRANSPORT_INFO_0 = core::ptr::null_mut();
    let mut dwRead: DWORD = 0 as DWORD;
    let mut dwTotal: DWORD = 0 as DWORD;
    let mut i: DWORD;
    let nPaddedLength: DWORD = 38 as DWORD;
    let mut Status: DWORD;

    'done: {
        Status = NetWkstaGetInfo(
            core::ptr::null_mut(),
            100 as DWORD,
            core::ptr::addr_of_mut!(WorkstationInfo) as *mut PBYTE,
        );
        if Status != NERR_Success as DWORD {
            break 'done;
        }

        Status = NetWkstaUserGetInfo(
            core::ptr::null_mut(),
            1 as DWORD,
            core::ptr::addr_of_mut!(UserInfo) as *mut PBYTE,
        );
        if Status != NERR_Success as DWORD {
            break 'done;
        }

        Status = NetWkstaTransportEnum(
            core::ptr::null_mut(),
            0 as DWORD,
            core::ptr::addr_of_mut!(TransportInfo) as *mut PBYTE,
            MAX_PREFERRED_LENGTH as DWORD,
            core::ptr::addr_of_mut!(dwRead),
            core::ptr::addr_of_mut!(dwTotal),
            core::ptr::null_mut(),
        );
        if Status != NERR_Success as DWORD {
            break 'done;
        }

        PrintPaddedMessageString(4450 as DWORD, nPaddedLength);
        ConPrintf(
            StdOut,
            RUSTCONV_WSTR_C3D39A2E4D96.as_ptr(),
            ((*WorkstationInfo).wki100_computername) as *mut WCHAR,
        );

        PrintPaddedMessageString(4468 as DWORD, nPaddedLength);
        ConPuts(StdOut, RUSTCONV_WSTR_5BFB8DE9D124.as_ptr());

        PrintPaddedMessageString(4451 as DWORD, nPaddedLength);
        ConPrintf(
            StdOut,
            RUSTCONV_WSTR_4BA5D7E4F16E.as_ptr(),
            ((*UserInfo).wkui1_username) as *mut WCHAR,
        );

        ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());

        PrintPaddedMessageString(4453 as DWORD, nPaddedLength);
        ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
        i = 0 as DWORD;
        while i < dwRead {
            ConPrintf(
                StdOut,
                RUSTCONV_WSTR_4B9295395356.as_ptr(),
                (*TransportInfo.add(i as usize)).wkti0_transport_name.add(8),
                (*TransportInfo.add(i as usize)).wkti0_transport_address,
            );
            i = i.wrapping_add(1);
        }
        ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());

        PrintPaddedMessageString(4452 as DWORD, nPaddedLength);
        ConPrintf(
            StdOut,
            RUSTCONV_WSTR_91EE49139047.as_ptr(),
            (*WorkstationInfo).wki100_ver_major,
            (*WorkstationInfo).wki100_ver_minor,
        );

        ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());

        PrintPaddedMessageString(4455 as DWORD, nPaddedLength);
        ConPrintf(
            StdOut,
            RUSTCONV_WSTR_4BA5D7E4F16E.as_ptr(),
            ((*WorkstationInfo).wki100_langroup) as *mut WCHAR,
        );

        PrintPaddedMessageString(4469 as DWORD, nPaddedLength);
        ConPuts(StdOut, RUSTCONV_WSTR_5BFB8DE9D124.as_ptr());

        PrintPaddedMessageString(4456 as DWORD, nPaddedLength);
        ConPrintf(
            StdOut,
            RUSTCONV_WSTR_4BA5D7E4F16E.as_ptr(),
            ((*UserInfo).wkui1_logon_domain) as *mut WCHAR,
        );

        ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());

        PrintPaddedMessageString(4458 as DWORD, nPaddedLength);
        ConPuts(StdOut, RUSTCONV_WSTR_5BFB8DE9D124.as_ptr());

        PrintPaddedMessageString(4459 as DWORD, nPaddedLength);
        ConPuts(StdOut, RUSTCONV_WSTR_5BFB8DE9D124.as_ptr());

        PrintPaddedMessageString(4460 as DWORD, nPaddedLength);
        ConPuts(StdOut, RUSTCONV_WSTR_5BFB8DE9D124.as_ptr());
    }

    if !TransportInfo.is_null() {
        NetApiBufferFree(TransportInfo as PVOID);
    }

    if !UserInfo.is_null() {
        NetApiBufferFree(UserInfo as PVOID);
    }

    if !WorkstationInfo.is_null() {
        NetApiBufferFree(WorkstationInfo as PVOID);
    }

    return 0 as INT;
}
#[no_mangle]
pub unsafe extern "C" fn cmdConfig(argc: INT, argv: *mut *mut WCHAR) -> INT {
    let mut i: INT = 0;
    let mut result: INT = 0;
    let mut bServer: BOOL = (FALSE) as i32;
    let mut bWorkstation: BOOL = (FALSE) as i32;
    let mut p: PWSTR = core::ptr::null_mut();
    let mut endptr: PWSTR = core::ptr::null_mut();
    let mut bModify: BOOL = (FALSE) as i32;
    let mut lValue: LONG = 0;
    let mut ServerInfo: PSERVER_INFO_102 = core::ptr::null_mut();
    let mut Status: NET_API_STATUS = 0;

    'done: {
        i = 2;
        while i < argc {
            'body: {
                if _wcsicmp(*argv.offset(i as isize), RUSTCONV_WSTR_2FBF874A4098.as_ptr()) == 0 {
                    if (bWorkstation) as u32 == FALSE {
                        bServer = (TRUE) as i32;
                    }
                    break 'body;
                }

                if _wcsicmp(*argv.offset(i as isize), RUSTCONV_WSTR_CA01375376F7.as_ptr()) == 0 {
                    if (bServer) as u32 == FALSE {
                        bWorkstation = (TRUE) as i32;
                    }
                    break 'body;
                }

                if _wcsicmp(*argv.offset(i as isize), RUSTCONV_WSTR_6509121F99F7.as_ptr()) == 0 {
                    /* Print short syntax help */
                    if (bServer) as u32 == TRUE {
                        PrintMessageString(4381);
                        ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
                        PrintNetMessage(MSG_CONFIG_SERVER_SYNTAX);
                    } else {
                        PrintMessageString(4381);
                        ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
                        PrintNetMessage(MSG_CONFIG_SYNTAX);
                    }
                    return 0;
                }

                if _wcsicmp(*argv.offset(i as isize), RUSTCONV_WSTR_1A749FECCEE7.as_ptr()) == 0 {
                    /* Print full help text*/
                    if (bServer) as u32 == TRUE {
                        PrintMessageString(4381);
                        ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
                        PrintNetMessage(MSG_CONFIG_SERVER_SYNTAX);
                        PrintNetMessage(MSG_CONFIG_SERVER_HELP);
                    } else {
                        PrintMessageString(4381);
                        ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
                        PrintNetMessage(MSG_CONFIG_SYNTAX);
                        PrintNetMessage(MSG_CONFIG_HELP);
                    }
                    return 0;
                }
            }
            i += 1;
        }

        if bServer != 0 {
            Status = NetServerGetInfo(core::ptr::null_mut(), 102, core::ptr::addr_of_mut!(ServerInfo) as *mut PBYTE);
            if Status != NERR_Success {
                break 'done;
            }

            i = 2;
            while i < argc {
                'body2: {
                    if *(*argv.offset(i as isize)).offset(0 as isize) != 0x002Fu16 {
                        break 'body2;
                    }

                    if _wcsnicmp(*argv.offset(i as isize), RUSTCONV_WSTR_0903D887FBED.as_ptr(), 16) == 0 {
                        p = (*argv.offset(i as isize)).offset(16 as isize);
                        lValue = wcstol(p, core::ptr::addr_of_mut!(endptr), 10) as LONG;
                        if *endptr != 0 {
                            PrintMessageStringV(3952, RUSTCONV_WSTR_A6777530FA58.as_ptr());
                            result = 1;
                            break 'done;
                        }

                        if lValue < -1 || lValue > 65535 {
                            PrintMessageStringV(3952, RUSTCONV_WSTR_A6777530FA58.as_ptr());
                            result = 1;
                            break 'done;
                        }

                        (*ServerInfo).sv102_disc = lValue;
                        bModify = (TRUE) as i32;
                    } else if _wcsnicmp(*argv.offset(i as isize), RUSTCONV_WSTR_9D907FB934C0.as_ptr(), 12) == 0 {
                        (*ServerInfo).sv102_comment = (*argv.offset(i as isize)).offset(12 as isize);
                        bModify = (TRUE) as i32;
                    } else if _wcsnicmp(*argv.offset(i as isize), RUSTCONV_WSTR_A9EAD68DFCD4.as_ptr(), 8) == 0 {
                        p = (*argv.offset(i as isize)).offset(8 as isize);
                        if _wcsicmp(p, RUSTCONV_WSTR_6D1618CC9381.as_ptr()) != 0 && _wcsicmp(p, RUSTCONV_WSTR_833AD7EF6992.as_ptr()) != 0 {
                            PrintMessageStringV(3952, RUSTCONV_WSTR_DCE10C604406.as_ptr());
                            result = 1;
                            break 'done;
                        }

                        (*ServerInfo).sv102_hidden = if _wcsicmp(p, RUSTCONV_WSTR_6D1618CC9381.as_ptr()) == 0 {
                            (TRUE) as i32
                        } else {
                            (FALSE) as i32
                        };
                        bModify = (TRUE) as i32;
                    } else {
                        PrintMessageString(4381);
                        ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
                        PrintNetMessage(MSG_CONFIG_SERVER_SYNTAX);
                        result = 1;
                        break 'done;
                    }
                }
                i += 1;
            }

            if bModify != 0 {
                Status = NetServerSetInfo(core::ptr::null_mut(), 102, core::ptr::addr_of_mut!(ServerInfo) as PBYTE, core::ptr::null_mut());
                if Status != NERR_Success {
                    result = 1;
                }
            } else {
                result = DisplayServerConfig(ServerInfo);
            }
        } else if bWorkstation != 0 {
            result = DisplayWorkstationConfig();
        } else {
            PrintMessageString(4378);
            ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
            ConPuts(StdOut, RUSTCONV_WSTR_2D08E5CF381F.as_ptr());
            ConPuts(StdOut, RUSTCONV_WSTR_E92A90E19836.as_ptr());
            ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
        }
    }

    if !ServerInfo.is_null() {
        NetApiBufferFree(ServerInfo as PVOID);
    }

    if result == 0 {
        PrintErrorMessage(ERROR_SUCCESS);
    }

    return result;
}
