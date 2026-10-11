use crate::net::main::PrintErrorMessage;
use crate::net::main::PrintMessageString;
use crate::net::main::PrintNetMessage;
use crate::net::main::PrintPaddedMessageString;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_075DE2B906DB: [u16; 2] = [0x000Au16, 0x0000];
pub static RUSTCONV_WSTR_1A749FECCEE7: [u16; 6] = [0x002Fu16, 0x0068, 0x0065, 0x006C, 0x0070, 0x0000];
pub static RUSTCONV_WSTR_2D08E5CF381F: [u16; 11] = [
    0x0020u16, 0x0020, 0x0020, 0x0053, 0x0065, 0x0072, 0x0076, 0x0065, 0x0072, 0x000A,
    0x0000,
];
pub static RUSTCONV_WSTR_2FBF874A4098: [u16; 7] = [0x0073u16, 0x0065, 0x0072, 0x0076, 0x0065, 0x0072, 0x0000];
pub static RUSTCONV_WSTR_321CF6D32384: [u16; 6] = [0x0025u16, 0x006C, 0x0075, 0x000A, 0x000A, 0x0000];
pub static RUSTCONV_WSTR_6509121F99F7: [u16; 5] = [0x0068u16, 0x0065, 0x006C, 0x0070, 0x0000];
pub static RUSTCONV_WSTR_89B544410E96: [u16; 10] = [0x0020u16, 0x0025, 0x0073, 0x0020, 0x0025, 0x0073, 0x000A, 0x000A, 0x000A, 0x0000];
pub static RUSTCONV_WSTR_99397581246D: [u16; 5] = [0x0025u16, 0x006C, 0x0075, 0x000A, 0x0000];
pub static RUSTCONV_WSTR_9D313EA1711F: [u16; 3] = [0x000Au16, 0x000A, 0x0000];
pub static RUSTCONV_WSTR_A0D69491DF0E: [u16; 7] = [0x0025u16, 0x0049, 0x0036, 0x0034, 0x0075, 0x000A, 0x0000];
pub static RUSTCONV_WSTR_A2FF2DDCAEDE: [u16; 3] = [0x0020u16, 0x0020, 0x0000];
pub static RUSTCONV_WSTR_CA01375376F7: [u16; 12] = [
    0x0077u16, 0x006F, 0x0072, 0x006B, 0x0073, 0x0074, 0x0061, 0x0074, 0x0069, 0x006F,
    0x006E, 0x0000,
];
pub static RUSTCONV_WSTR_E92A90E19836: [u16; 16] = [
    0x0020u16, 0x0020, 0x0020, 0x0057, 0x006F, 0x0072, 0x006B, 0x0073, 0x0074, 0x0061,
    0x0074, 0x0069, 0x006F, 0x006E, 0x000A, 0x0000,
];

use core::ffi::c_void;

unsafe extern "C" fn DisplayServerStatistics() -> INT {
    let mut ServerInfo: PSERVER_INFO_100 = core::ptr::null_mut();
    let mut StatisticsInfo: PSTAT_SERVER_0 = core::ptr::null_mut();
    let mut LargeValue: LARGE_INTEGER = core::mem::zeroed();
    let mut FileTime: FILETIME = core::mem::zeroed();
    let mut LocalFileTime: FILETIME = core::mem::zeroed();
    let mut SystemTime: SYSTEMTIME = core::mem::zeroed();
    let mut DateBuffer: [WCHAR; 32] = [0; 32];
    let mut TimeBuffer: [WCHAR; 32] = [0; 32];
    let nPaddedLength: DWORD = 35;
    let mut Status: NET_API_STATUS;

    'done: {
        Status = NetServerGetInfo(
            core::ptr::null_mut(),
            100,
            core::ptr::addr_of_mut!(ServerInfo) as *mut PBYTE,
        );
        if Status != NERR_Success {
            break 'done;
        }

        Status = NetStatisticsGet(
            core::ptr::null_mut(),
            SERVICE_SERVER as LPWSTR,
            0,
            0,
            core::ptr::addr_of_mut!(StatisticsInfo) as *mut LPBYTE,
        );
        if Status != NERR_Success {
            break 'done;
        }

        PrintMessageStringV(4624, (*ServerInfo).sv100_name);
        ConPrintf(StdOut, RUSTCONV_WSTR_9D313EA1711F.as_ptr());

        RtlSecondsSince1970ToTime(
            (*StatisticsInfo).sts0_start,
            core::ptr::addr_of_mut!(LargeValue),
        );
        FileTime.dwLowDateTime = LargeValue.u.LowPart;
        FileTime.dwHighDateTime = LargeValue.u.HighPart as DWORD;
        FileTimeToLocalFileTime(
            core::ptr::addr_of_mut!(FileTime),
            core::ptr::addr_of_mut!(LocalFileTime),
        );
        FileTimeToSystemTime(
            core::ptr::addr_of_mut!(LocalFileTime),
            core::ptr::addr_of_mut!(SystemTime),
        );

        GetDateFormatW(
            LOCALE_USER_DEFAULT,
            DATE_SHORTDATE,
            core::ptr::addr_of_mut!(SystemTime),
            core::ptr::null(),
            core::ptr::addr_of_mut!(DateBuffer) as LPWSTR,
            (ARRAYSIZE!(DateBuffer)) as i32,
        );

        GetTimeFormatW(
            LOCALE_USER_DEFAULT,
            0,
            core::ptr::addr_of_mut!(SystemTime),
            core::ptr::null(),
            core::ptr::addr_of_mut!(TimeBuffer) as LPWSTR,
            (ARRAYSIZE!(TimeBuffer)) as i32,
        );

        PrintMessageString(4600);
        ConPrintf(
            StdOut,
            RUSTCONV_WSTR_89B544410E96.as_ptr(),
            core::ptr::addr_of_mut!(DateBuffer) as *mut WCHAR,
            core::ptr::addr_of_mut!(TimeBuffer) as *mut WCHAR,
        );

        PrintPaddedMessageString(4601, nPaddedLength);
        ConPrintf(StdOut, RUSTCONV_WSTR_99397581246D.as_ptr(), (*StatisticsInfo).sts0_sopens);

        PrintPaddedMessageString(4602, nPaddedLength);
        ConPrintf(StdOut, RUSTCONV_WSTR_99397581246D.as_ptr(), (*StatisticsInfo).sts0_stimedout);

        PrintPaddedMessageString(4603, nPaddedLength);
        ConPrintf(StdOut, RUSTCONV_WSTR_321CF6D32384.as_ptr(), (*StatisticsInfo).sts0_serrorout);

        LargeValue.u.LowPart = (*StatisticsInfo).sts0_bytessent_low;
        LargeValue.u.HighPart = (*StatisticsInfo).sts0_bytessent_high as LONG;
        PrintPaddedMessageString(4604, nPaddedLength);
        ConPrintf(
            StdOut,
            RUSTCONV_WSTR_A0D69491DF0E.as_ptr(),
            LargeValue.QuadPart / 1024,
        );

        LargeValue.u.LowPart = (*StatisticsInfo).sts0_bytesrcvd_low;
        LargeValue.u.HighPart = (*StatisticsInfo).sts0_bytesrcvd_high as LONG;
        PrintPaddedMessageString(4605, nPaddedLength);
        ConPrintf(
            StdOut,
            RUSTCONV_WSTR_A0D69491DF0E.as_ptr(),
            LargeValue.QuadPart / 1024,
        );

        PrintPaddedMessageString(4606, nPaddedLength);
        ConPrintf(StdOut, RUSTCONV_WSTR_321CF6D32384.as_ptr(), (*StatisticsInfo).sts0_avresponse);

        PrintPaddedMessageString(4610, nPaddedLength);
        ConPrintf(StdOut, RUSTCONV_WSTR_99397581246D.as_ptr(), (*StatisticsInfo).sts0_syserrors);

        PrintPaddedMessageString(4612, nPaddedLength);
        ConPrintf(StdOut, RUSTCONV_WSTR_99397581246D.as_ptr(), (*StatisticsInfo).sts0_permerrors);

        PrintPaddedMessageString(4611, nPaddedLength);
        ConPrintf(StdOut, RUSTCONV_WSTR_321CF6D32384.as_ptr(), (*StatisticsInfo).sts0_pwerrors);

        PrintPaddedMessageString(4608, nPaddedLength);
        ConPrintf(StdOut, RUSTCONV_WSTR_99397581246D.as_ptr(), (*StatisticsInfo).sts0_fopens);

        PrintPaddedMessageString(4613, nPaddedLength);
        ConPrintf(StdOut, RUSTCONV_WSTR_99397581246D.as_ptr(), (*StatisticsInfo).sts0_devopens);

        PrintPaddedMessageString(4609, nPaddedLength);
        ConPrintf(StdOut, RUSTCONV_WSTR_321CF6D32384.as_ptr(), (*StatisticsInfo).sts0_jobsqueued);

        PrintMessageString(4620);
        ConPrintf(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());

        ConPrintf(StdOut, RUSTCONV_WSTR_A2FF2DDCAEDE.as_ptr());
        PrintPaddedMessageString(
            4621,
            nPaddedLength.wrapping_sub(2),
        );
        ConPrintf(StdOut, RUSTCONV_WSTR_99397581246D.as_ptr(), (*StatisticsInfo).sts0_bigbufneed);

        ConPrintf(StdOut, RUSTCONV_WSTR_A2FF2DDCAEDE.as_ptr());
        PrintPaddedMessageString(
            4622,
            nPaddedLength.wrapping_sub(2),
        );
        ConPrintf(StdOut, RUSTCONV_WSTR_321CF6D32384.as_ptr(), (*StatisticsInfo).sts0_reqbufneed);
    }

    if !StatisticsInfo.is_null() {
        NetApiBufferFree(StatisticsInfo as PVOID);
    }

    if !ServerInfo.is_null() {
        NetApiBufferFree(ServerInfo as PVOID);
    }

    return 0;
}

unsafe extern "C" fn DisplayWorkstationStatistics() -> INT {
    let mut WorkstationInfo: PWKSTA_INFO_100 = core::ptr::null_mut();
    let mut StatisticsInfo: PSTAT_WORKSTATION_0 = core::ptr::null_mut();
    let mut LargeValue: LARGE_INTEGER = core::mem::zeroed();
    let mut FileTime: FILETIME = core::mem::zeroed();
    let mut LocalFileTime: FILETIME = core::mem::zeroed();
    let mut SystemTime: SYSTEMTIME = core::mem::zeroed();
    let mut DateBuffer: [WCHAR; 32] = [0; 32];
    let mut TimeBuffer: [WCHAR; 32] = [0; 32];
    let nPaddedLength: DWORD = 47;
    let mut Status: NET_API_STATUS;

    'done: {
        Status = NetWkstaGetInfo(
            core::ptr::null_mut(),
            100,
            core::ptr::addr_of_mut!(WorkstationInfo) as *mut PBYTE,
        );
        if Status != NERR_Success {
            break 'done;
        }

        Status = NetStatisticsGet(
            core::ptr::null_mut(),
            SERVICE_WORKSTATION as LPWSTR,
            0,
            0,
            (core::ptr::addr_of_mut!(StatisticsInfo) as LPBYTE).cast(),
        );
        if Status != NERR_Success {
            break 'done;
        }

        PrintMessageStringV(4623, (*WorkstationInfo).wki100_computername);
        ConPrintf(StdOut, RUSTCONV_WSTR_9D313EA1711F.as_ptr());

        RtlSecondsSince1970ToTime(
            (*StatisticsInfo).StatisticsStartTime.u.LowPart,
            core::ptr::addr_of_mut!(LargeValue),
        );
        FileTime.dwLowDateTime = LargeValue.u.LowPart;
        FileTime.dwHighDateTime = (LargeValue.u.HighPart) as u32;
        FileTimeToLocalFileTime(
            core::ptr::addr_of_mut!(FileTime),
            core::ptr::addr_of_mut!(LocalFileTime),
        );
        FileTimeToSystemTime(
            core::ptr::addr_of_mut!(LocalFileTime),
            core::ptr::addr_of_mut!(SystemTime),
        );

        GetDateFormatW(
            LOCALE_USER_DEFAULT,
            DATE_SHORTDATE,
            core::ptr::addr_of_mut!(SystemTime),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(DateBuffer) as LPWSTR,
            (ARRAYSIZE!(DateBuffer)) as i32,
        );

        GetTimeFormatW(
            LOCALE_USER_DEFAULT,
            0,
            core::ptr::addr_of_mut!(SystemTime),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(TimeBuffer) as LPWSTR,
            (ARRAYSIZE!(TimeBuffer)) as i32,
        );

        PrintMessageString(4600);
        ConPrintf(
            StdOut,
            RUSTCONV_WSTR_89B544410E96.as_ptr(),
            core::ptr::addr_of_mut!(DateBuffer) as *mut WCHAR,
            core::ptr::addr_of_mut!(TimeBuffer) as *mut WCHAR,
        );

        PrintPaddedMessageString(4630, nPaddedLength);
        ConPrintf(StdOut, RUSTCONV_WSTR_A0D69491DF0E.as_ptr(), (*StatisticsInfo).BytesReceived.QuadPart);

        PrintPaddedMessageString(4631, nPaddedLength);
        ConPrintf(StdOut, RUSTCONV_WSTR_A0D69491DF0E.as_ptr(), (*StatisticsInfo).SmbsReceived.QuadPart);

        PrintPaddedMessageString(4632, nPaddedLength);
        ConPrintf(StdOut, RUSTCONV_WSTR_A0D69491DF0E.as_ptr(), (*StatisticsInfo).BytesTransmitted.QuadPart);

        PrintPaddedMessageString(4633, nPaddedLength);
        ConPrintf(StdOut, RUSTCONV_WSTR_A0D69491DF0E.as_ptr(), (*StatisticsInfo).SmbsTransmitted.QuadPart);

        PrintPaddedMessageString(4634, nPaddedLength);
        ConPrintf(StdOut, RUSTCONV_WSTR_99397581246D.as_ptr(), (*StatisticsInfo).ReadOperations);

        PrintPaddedMessageString(4635, nPaddedLength);
        ConPrintf(StdOut, RUSTCONV_WSTR_99397581246D.as_ptr(), (*StatisticsInfo).WriteOperations);

        PrintPaddedMessageString(4636, nPaddedLength);
        ConPrintf(StdOut, RUSTCONV_WSTR_99397581246D.as_ptr(), (*StatisticsInfo).RawReadsDenied);

        PrintPaddedMessageString(4637, nPaddedLength);
        ConPrintf(StdOut, RUSTCONV_WSTR_321CF6D32384.as_ptr(), (*StatisticsInfo).RawWritesDenied);

        PrintPaddedMessageString(4638, nPaddedLength);
        ConPrintf(StdOut, RUSTCONV_WSTR_99397581246D.as_ptr(), (*StatisticsInfo).NetworkErrors);

        PrintPaddedMessageString(4639, nPaddedLength);
        ConPrintf(
            StdOut,
            RUSTCONV_WSTR_99397581246D.as_ptr(),
            (*StatisticsInfo)
                .CoreConnects
                .wrapping_add((*StatisticsInfo).Lanman20Connects)
                .wrapping_add((*StatisticsInfo).Lanman21Connects)
                .wrapping_add((*StatisticsInfo).LanmanNtConnects),
        );

        PrintPaddedMessageString(4640, nPaddedLength);
        ConPrintf(StdOut, RUSTCONV_WSTR_99397581246D.as_ptr(), (*StatisticsInfo).Reconnects);

        PrintPaddedMessageString(4641, nPaddedLength);
        ConPrintf(StdOut, RUSTCONV_WSTR_321CF6D32384.as_ptr(), (*StatisticsInfo).ServerDisconnects);

        PrintPaddedMessageString(4642, nPaddedLength);
        ConPrintf(StdOut, RUSTCONV_WSTR_99397581246D.as_ptr(), (*StatisticsInfo).Sessions);

        PrintPaddedMessageString(4643, nPaddedLength);
        ConPrintf(StdOut, RUSTCONV_WSTR_99397581246D.as_ptr(), (*StatisticsInfo).HungSessions);

        PrintPaddedMessageString(4644, nPaddedLength);
        ConPrintf(StdOut, RUSTCONV_WSTR_99397581246D.as_ptr(), (*StatisticsInfo).FailedSessions);

        PrintPaddedMessageString(4645, nPaddedLength);
        ConPrintf(
            StdOut,
            RUSTCONV_WSTR_99397581246D.as_ptr(),
            (*StatisticsInfo)
                .InitiallyFailedOperations
                .wrapping_add((*StatisticsInfo).FailedCompletionOperations),
        );

        PrintPaddedMessageString(4646, nPaddedLength);
        ConPrintf(StdOut, RUSTCONV_WSTR_99397581246D.as_ptr(), (*StatisticsInfo).UseCount);

        PrintPaddedMessageString(4647, nPaddedLength);
        ConPrintf(StdOut, RUSTCONV_WSTR_321CF6D32384.as_ptr(), (*StatisticsInfo).FailedUseCount);
    }

    if !StatisticsInfo.is_null() {
        NetApiBufferFree(StatisticsInfo as PVOID);
    }

    if !WorkstationInfo.is_null() {
        NetApiBufferFree(WorkstationInfo as PVOID);
    }

    return 0;
}
#[no_mangle]
pub unsafe extern "C" fn cmdStatistics(mut argc: INT, mut argv: *mut *mut WCHAR) -> INT {
    let mut i: INT = 2;
    let mut result: INT = 0;
    let mut bServer: BOOL = (FALSE) as i32;
    let mut bWorkstation: BOOL = (FALSE) as i32;

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
                PrintMessageString(4381);
                ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
                PrintNetMessage(MSG_STATISTICS_SYNTAX);
                return 0;
            }

            if _wcsicmp(*argv.offset(i as isize), RUSTCONV_WSTR_1A749FECCEE7.as_ptr()) == 0 {
                /* Print full help text*/
                PrintMessageString(4381);
                ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
                PrintNetMessage(MSG_STATISTICS_SYNTAX);
                PrintNetMessage(MSG_STATISTICS_HELP);
                return 0;
            }
        }
        i += 1;
    }

    if bServer != 0 {
        result = DisplayServerStatistics();
    } else if bWorkstation != 0 {
        result = DisplayWorkstationStatistics();
    } else {
        PrintMessageString(4379);
        ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
        ConPuts(StdOut, RUSTCONV_WSTR_2D08E5CF381F.as_ptr());
        ConPuts(StdOut, RUSTCONV_WSTR_E92A90E19836.as_ptr());
        ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
    }

    if result == 0 {
        PrintErrorMessage(ERROR_SUCCESS);
    }

    return result;
}
