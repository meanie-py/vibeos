use crate::net::main::PrintMessageString;
use crate::net::main::PrintNetMessage;
use crate::net::main::PrintPaddedMessageString;
use crate::net::main::PrintPadding;
use crate::net::main::ReadFromConsole;
use crate::net::main::TranslateAppMessage;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_00C0DB009DDA: [u16; 10] = [0x002Fu16, 0x0063, 0x006F, 0x006D, 0x006D, 0x0065, 0x006E, 0x0074, 0x003A, 0x0000];
pub static RUSTCONV_WSTR_075DE2B906DB: [u16; 2] = [0x000Au16, 0x0000];
pub static RUSTCONV_WSTR_0AD474BF6806: [u16; 8] = [0x002Fu16, 0x0074, 0x0069, 0x006D, 0x0065, 0x0073, 0x003A, 0x0000];
pub static RUSTCONV_WSTR_0BE94F48150F: [u16; 6] = [0x006Eu16, 0x0065, 0x0076, 0x0065, 0x0072, 0x0000];
pub static RUSTCONV_WSTR_0DE3843AC6EC: [u16; 15] = [
    0x002Fu16, 0x0077, 0x006F, 0x0072, 0x006B, 0x0073, 0x0074, 0x0061, 0x0074, 0x0069,
    0x006F, 0x006E, 0x0073, 0x003A, 0x0000,
];
pub static RUSTCONV_WSTR_0F163FD255A4: [u16; 8] = [0x002Fu16, 0x0072, 0x0061, 0x006E, 0x0064, 0x006F, 0x006D, 0x0000];
pub static RUSTCONV_WSTR_17084575BCAC: [u16; 14] = [
    0x002Fu16, 0x0070, 0x0061, 0x0073, 0x0073, 0x0077, 0x006F, 0x0072, 0x0064, 0x0072,
    0x0065, 0x0071, 0x003A, 0x0000,
];
pub static RUSTCONV_WSTR_1A52D437A1CE: [u16; 9] = [0x002Fu16, 0x0061, 0x0063, 0x0074, 0x0069, 0x0076, 0x0065, 0x003A, 0x0000];
pub static RUSTCONV_WSTR_1A749FECCEE7: [u16; 6] = [0x002Fu16, 0x0068, 0x0065, 0x006C, 0x0070, 0x0000];
pub static RUSTCONV_WSTR_1B0FB29105DF: [u16; 10] = [0x002Fu16, 0x0068, 0x006F, 0x006D, 0x0065, 0x0064, 0x0069, 0x0072, 0x003A, 0x0000];
pub static RUSTCONV_WSTR_329A042C9CFB: [u16; 13] = [
    0x002Fu16, 0x0073, 0x0063, 0x0072, 0x0069, 0x0070, 0x0074, 0x0070, 0x0061, 0x0074,
    0x0068, 0x003A, 0x0000,
];
pub static RUSTCONV_WSTR_447438009695: [u16; 6] = [0x0025u16, 0x0073, 0x0020, 0x0025, 0x0073, 0x0000];
pub static RUSTCONV_WSTR_4BA5D7E4F16E: [u16; 4] = [0x0025u16, 0x0073, 0x000A, 0x0000];
pub static RUSTCONV_WSTR_5197BE528E07: [u16; 11] = [
    0x002Fu16, 0x0066, 0x0075, 0x006C, 0x006C, 0x006E, 0x0061, 0x006D, 0x0065, 0x003A,
    0x0000,
];
pub static RUSTCONV_WSTR_5295D3F8988C: [u16; 4] = [0x0061u16, 0x006C, 0x006C, 0x0000];
pub static RUSTCONV_WSTR_580D24B22F7C: [u16; 13] = [
    0x002Fu16, 0x0043, 0x004F, 0x0055, 0x004E, 0x0054, 0x0052, 0x0059, 0x0043, 0x004F,
    0x0044, 0x0045, 0x0000,
];
pub static RUSTCONV_WSTR_582BA4648587: [u16; 42] = [
    0x0054u16, 0x0068, 0x0065, 0x0020, 0x002F, 0x0044, 0x004F, 0x004D, 0x0041, 0x0049,
    0x004E, 0x0020, 0x006F, 0x0070, 0x0074, 0x0069, 0x006F, 0x006E, 0x0020, 0x0069,
    0x0073, 0x0020, 0x006E, 0x006F, 0x0074, 0x0020, 0x0073, 0x0075, 0x0070, 0x0070,
    0x006F, 0x0072, 0x0074, 0x0065, 0x0064, 0x0020, 0x0079, 0x0065, 0x0074, 0x002E,
    0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_5D12D1C1410E: [u16; 5] = [0x002Fu16, 0x0061, 0x0064, 0x0064, 0x0000];
pub static RUSTCONV_WSTR_6D1618CC9381: [u16; 4] = [0x0079u16, 0x0065, 0x0073, 0x0000];
pub static RUSTCONV_WSTR_74DDC21DC0C7: [u16; 12] = [
    0x0025u16, 0x0030, 0x0033, 0x006C, 0x0064, 0x0020, 0x0028, 0x0025, 0x0073, 0x0029,
    0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_7E7219CF4FF7: [u16; 8] = [0x002Fu16, 0x0064, 0x0065, 0x006C, 0x0065, 0x0074, 0x0065, 0x0000];
pub static RUSTCONV_WSTR_833AD7EF6992: [u16; 3] = [0x006Eu16, 0x006F, 0x0000];
pub static RUSTCONV_WSTR_8CE542C39882: [u16; 14] = [
    0x002Fu16, 0x0075, 0x0073, 0x0065, 0x0072, 0x0063, 0x006F, 0x006D, 0x006D, 0x0065,
    0x006E, 0x0074, 0x003A, 0x0000,
];
pub static RUSTCONV_WSTR_8D71B3FAAB82: [u16; 2] = [0x0020u16, 0x0000];
pub static RUSTCONV_WSTR_96A296D224F2: [u16; 1] = [0x0000u16];
pub static RUSTCONV_WSTR_9BB9C2E30775: [u16; 8] = [0x002Fu16, 0x0064, 0x006F, 0x006D, 0x0061, 0x0069, 0x006E, 0x0000];
pub static RUSTCONV_WSTR_9D313EA1711F: [u16; 3] = [0x000Au16, 0x000A, 0x0000];
pub static RUSTCONV_WSTR_A032BD847C02: [u16; 14] = [
    0x002Fu16, 0x0070, 0x0061, 0x0073, 0x0073, 0x0077, 0x006F, 0x0072, 0x0064, 0x0063,
    0x0068, 0x0067, 0x003A, 0x0000,
];
pub static RUSTCONV_WSTR_A5AB18086796: [u16; 10] = [0x002Fu16, 0x0065, 0x0078, 0x0070, 0x0069, 0x0072, 0x0065, 0x0073, 0x003A, 0x0000];
pub static RUSTCONV_WSTR_A8F12694C205: [u16; 4] = [0x0020u16, 0x002D, 0x0020, 0x0000];
pub static RUSTCONV_WSTR_A9A634858003: [u16; 9] = [0x002Fu16, 0x0045, 0x0058, 0x0050, 0x0049, 0x0052, 0x0045, 0x0053, 0x0000];
pub static RUSTCONV_WSTR_BCE2AEEE903F: [u16; 10] = [0x002Fu16, 0x005B, 0x005D, 0x003D, 0x003F, 0x005C, 0x002B, 0x003A, 0x002E, 0x0000];
pub static RUSTCONV_WSTR_C441979A38EC: [u16; 13] = [
    0x002Fu16, 0x0050, 0x0041, 0x0053, 0x0053, 0x0057, 0x004F, 0x0052, 0x0044, 0x0043,
    0x0048, 0x0047, 0x0000,
];
pub static RUSTCONV_WSTR_CBFCEEE03B06: [u16; 14] = [
    0x002Fu16, 0x0070, 0x0072, 0x006F, 0x0066, 0x0069, 0x006C, 0x0065, 0x0070, 0x0061,
    0x0074, 0x0068, 0x003A, 0x0000,
];
pub static RUSTCONV_WSTR_D1DAA4B12576: [u16; 14] = [
    0x002Fu16, 0x0063, 0x006F, 0x0075, 0x006E, 0x0074, 0x0072, 0x0079, 0x0063, 0x006F,
    0x0064, 0x0065, 0x003A, 0x0000,
];
pub static RUSTCONV_WSTR_D90EC374D009: [u16; 13] = [
    0x002Fu16, 0x0050, 0x0041, 0x0053, 0x0053, 0x0057, 0x004F, 0x0052, 0x0044, 0x0052,
    0x0045, 0x0051, 0x0000,
];
pub static RUSTCONV_WSTR_E8A4B2EE7EDE: [u16; 2] = [0x002Au16, 0x0000];
pub static RUSTCONV_WSTR_ECEFEF436E2E: [u16; 8] = [0x002Fu16, 0x0041, 0x0043, 0x0054, 0x0049, 0x0056, 0x0045, 0x0000];
pub static RUSTCONV_WSTR_F118FF86CE3B: [u16; 5] = [0x002Au16, 0x0025, 0x0073, 0x000A, 0x0000];
pub static RUSTCONV_WSTR_FB47CCC40FE6: [u16; 3] = [0x0025u16, 0x0073, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut szPasswordChars: [WCHAR; 71] = unsafe {
    ::core::mem::transmute::<
        [u8; 142],
        [WCHAR; 71],
    >(
        *b"0\x001\x002\x003\x004\x005\x006\x007\08\09\0a\0b\0c\0d\0e\0f\0g\0h\0i\0j\0k\0l\0m\0n\0o\0p\0q\0r\0s\0t\0u\0v\0w\0x\0y\0z\0A\0B\0C\0D\0E\0F\0G\0H\0I\0J\0K\0L\0M\0N\0O\0P\0Q\0R\0S\0T\0U\0V\0W\0X\0Y\0Z\0@\0#\0$\0%\0_\0-\0+\0:\0\0\0",
    )
};

static mut CountryTable: [COUNTRY_TABLE; 25] = [
    _COUNTY_TABLE {
        dwCountryCode: 0 as DWORD,
        dwMessageId: 5080 as DWORD,
    },
    _COUNTY_TABLE {
        dwCountryCode: 1 as DWORD,
        dwMessageId: 5081 as DWORD,
    },
    _COUNTY_TABLE {
        dwCountryCode: 2 as DWORD,
        dwMessageId: 5082 as DWORD,
    },
    _COUNTY_TABLE {
        dwCountryCode: 3 as DWORD,
        dwMessageId: 5083 as DWORD,
    },
    _COUNTY_TABLE {
        dwCountryCode: 31 as DWORD,
        dwMessageId: 5084 as DWORD,
    },
    _COUNTY_TABLE {
        dwCountryCode: 32 as DWORD,
        dwMessageId: 5085 as DWORD,
    },
    _COUNTY_TABLE {
        dwCountryCode: 33 as DWORD,
        dwMessageId: 5086 as DWORD,
    },
    _COUNTY_TABLE {
        dwCountryCode: 34 as DWORD,
        dwMessageId: 5090 as DWORD,
    },
    _COUNTY_TABLE {
        dwCountryCode: 39 as DWORD,
        dwMessageId: 5087 as DWORD,
    },
    _COUNTY_TABLE {
        dwCountryCode: 41 as DWORD,
        dwMessageId: 5088 as DWORD,
    },
    _COUNTY_TABLE {
        dwCountryCode: 44 as DWORD,
        dwMessageId: 5089 as DWORD,
    },
    _COUNTY_TABLE {
        dwCountryCode: 45 as DWORD,
        dwMessageId: 5091 as DWORD,
    },
    _COUNTY_TABLE {
        dwCountryCode: 46 as DWORD,
        dwMessageId: 5092 as DWORD,
    },
    _COUNTY_TABLE {
        dwCountryCode: 47 as DWORD,
        dwMessageId: 5093 as DWORD,
    },
    _COUNTY_TABLE {
        dwCountryCode: 49 as DWORD,
        dwMessageId: 5094 as DWORD,
    },
    _COUNTY_TABLE {
        dwCountryCode: 61 as DWORD,
        dwMessageId: 5095 as DWORD,
    },
    _COUNTY_TABLE {
        dwCountryCode: 81 as DWORD,
        dwMessageId: 5096 as DWORD,
    },
    _COUNTY_TABLE {
        dwCountryCode: 82 as DWORD,
        dwMessageId: 5097 as DWORD,
    },
    _COUNTY_TABLE {
        dwCountryCode: 86 as DWORD,
        dwMessageId: 5098 as DWORD,
    },
    _COUNTY_TABLE {
        dwCountryCode: 88 as DWORD,
        dwMessageId: 5099 as DWORD,
    },
    _COUNTY_TABLE {
        dwCountryCode: 99 as DWORD,
        dwMessageId: 5100 as DWORD,
    },
    _COUNTY_TABLE {
        dwCountryCode: 351 as DWORD,
        dwMessageId: 5101 as DWORD,
    },
    _COUNTY_TABLE {
        dwCountryCode: 358 as DWORD,
        dwMessageId: 5102 as DWORD,
    },
    _COUNTY_TABLE {
        dwCountryCode: 785 as DWORD,
        dwMessageId: 5103 as DWORD,
    },
    _COUNTY_TABLE {
        dwCountryCode: 972 as DWORD,
        dwMessageId: 5104 as DWORD,
    },
];

pub const SECONDS_PER_DAY: i32 = 60 * 60 * 24;

pub const SECONDS_PER_HOUR: i32 = 60 * 60;

pub const HOURS_PER_DAY: i32 = 24;

pub const DAYS_PER_WEEK: i32 = 7;

#[repr(C)]
pub struct _COUNTY_TABLE {
    pub dwCountryCode: DWORD,
    pub dwMessageId: DWORD,
}
pub type COUNTRY_TABLE = _COUNTY_TABLE;
pub type PCOUNTRY_TABLE = *mut COUNTRY_TABLE;


unsafe extern "C" fn CompareInfo(a: *const ::core::ffi::c_void, b: *const ::core::ffi::c_void) -> ::core::ffi::c_int {
    return _wcsicmp((*(a as PUSER_INFO_0)).usri0_name, (*(b as PUSER_INFO_0)).usri0_name);
}

unsafe extern "C" fn EnumerateUsers() -> DWORD {
    let mut pBuffer: PUSER_INFO_0 = core::ptr::null_mut();
    let mut pServer: PSERVER_INFO_100 = core::ptr::null_mut();
    let mut dwRead: DWORD = 0;
    let mut dwTotal: DWORD = 0;
    let mut i: DWORD;
    let mut ResumeHandle: DWORD = 0;
    let mut Status: NET_API_STATUS;

    Status = NetServerGetInfo(
        core::ptr::null_mut::<WCHAR>(),
        100,
        &raw mut pServer as *mut LPBYTE,
    );
    if Status != NERR_Success {
        return Status;
    }

    ConPuts(&raw mut csStdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
    PrintMessageStringV(4410, (*pServer).sv100_name);
    ConPuts(&raw mut csStdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
    PrintPadding(0x002Du16, 79);
    ConPuts(&raw mut csStdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());

    NetApiBufferFree(pServer as PVOID);

    loop {
        Status = NetUserEnum(
            core::ptr::null::<WCHAR>(),
            0,
            0,
            &raw mut pBuffer as *mut LPBYTE,
            MAX_PREFERRED_LENGTH,
            &raw mut dwRead,
            &raw mut dwTotal,
            &raw mut ResumeHandle,
        );
        if (Status != NERR_Success) && (Status != ERROR_MORE_DATA) {
            return Status;
        }

        qsort(
            pBuffer as *mut core::ffi::c_void,
            dwRead as size_t,
            core::mem::size_of::<PUSER_INFO_0>() as size_t,
            Some(
                CompareInfo
                    as unsafe extern "C" fn(
                        *const core::ffi::c_void,
                        *const core::ffi::c_void,
                    ) -> core::ffi::c_int,
            ),
        );

        i = 0;
        while i < dwRead {
            if !(*pBuffer.add(i as usize)).usri0_name.is_null() {
                ConPrintf(
                    &raw mut csStdOut,
                    RUSTCONV_WSTR_4BA5D7E4F16E.as_ptr(),
                    (*pBuffer.add(i as usize)).usri0_name,
                );
            }
            i = i.wrapping_add(1);
        }

        NetApiBufferFree(pBuffer as PVOID);
        pBuffer = core::ptr::null_mut();

        if !(Status == ERROR_MORE_DATA) {
            break;
        }
    }

    return NERR_Success;
}

unsafe extern "C" fn PrintDateTime(dwSeconds: DWORD) {
    let mut Time: LARGE_INTEGER = core::mem::zeroed();
    let mut FileTime: FILETIME = core::mem::zeroed();
    let mut SystemTime: SYSTEMTIME = core::mem::zeroed();
    let mut DateBuffer: [WCHAR; 80] = [0; 80];
    let mut TimeBuffer: [WCHAR; 80] = [0; 80];

    RtlSecondsSince1970ToTime(dwSeconds, core::ptr::addr_of_mut!(Time));
    FileTime.dwLowDateTime = Time.u.LowPart;
    FileTime.dwHighDateTime = (Time.u.HighPart) as u32;
    FileTimeToLocalFileTime(core::ptr::addr_of_mut!(FileTime), core::ptr::addr_of_mut!(FileTime));
    FileTimeToSystemTime(core::ptr::addr_of_mut!(FileTime), core::ptr::addr_of_mut!(SystemTime));

    GetDateFormatW(
        LOCALE_USER_DEFAULT,
        DATE_SHORTDATE,
        core::ptr::addr_of_mut!(SystemTime),
        core::ptr::null(),
        core::ptr::addr_of_mut!(DateBuffer) as LPWSTR,
        ARRAYSIZE!(DateBuffer) as i32,
    );

    GetTimeFormatW(
        LOCALE_USER_DEFAULT,
        TIME_NOSECONDS,
        core::ptr::addr_of_mut!(SystemTime),
        core::ptr::null(),
        core::ptr::addr_of_mut!(TimeBuffer) as LPWSTR,
        ARRAYSIZE!(TimeBuffer) as i32,
    );

    ConPrintf(
        StdOut,
        RUSTCONV_WSTR_447438009695.as_ptr(),
        core::ptr::addr_of_mut!(DateBuffer) as *mut WCHAR,
        core::ptr::addr_of_mut!(TimeBuffer) as *mut WCHAR,
    );
}

unsafe extern "C" fn PrintLocalTime(dwSeconds: DWORD) {
    let mut Time: LARGE_INTEGER = core::mem::zeroed();
    let mut FileTime: FILETIME = core::mem::zeroed();
    let mut SystemTime: SYSTEMTIME = core::mem::zeroed();
    let mut TimeBuffer: [WCHAR; 80] = [0; 80];

    RtlSecondsSince1970ToTime(dwSeconds as ULONG, core::ptr::addr_of_mut!(Time));
    FileTime.dwLowDateTime = Time.u.LowPart;
    FileTime.dwHighDateTime = Time.u.HighPart as DWORD;
    FileTimeToSystemTime(core::ptr::addr_of_mut!(FileTime), core::ptr::addr_of_mut!(SystemTime));

    GetTimeFormatW(
        LOCALE_USER_DEFAULT,
        TIME_NOSECONDS,
        core::ptr::addr_of_mut!(SystemTime),
        core::ptr::null_mut(),
        TimeBuffer.as_mut_ptr(),
        (ARRAYSIZE!(TimeBuffer)) as i32
    );

    ConPrintf(StdOut, RUSTCONV_WSTR_FB47CCC40FE6.as_ptr(), TimeBuffer.as_mut_ptr());
}

unsafe extern "C" fn GetTimeInSeconds() -> DWORD {
    let mut Time: LARGE_INTEGER = core::mem::zeroed();
    let mut FileTime: FILETIME = core::mem::zeroed();
    let mut dwSeconds: DWORD = 0;
    GetSystemTimeAsFileTime(core::ptr::addr_of_mut!(FileTime));
    Time.u.LowPart = FileTime.dwLowDateTime;
    Time.u.HighPart = FileTime.dwHighDateTime as LONG;
    RtlTimeToSecondsSince1970(core::ptr::addr_of_mut!(Time), core::ptr::addr_of_mut!(dwSeconds));

    return dwSeconds;
}

unsafe extern "C" fn GetCountryFromCountryCode(
    mut dwCountryCode: DWORD,
    mut dwCountryLength: DWORD,
    mut szCountryBuffer: PWSTR,
) -> BOOL {
    let mut i: DWORD = 0;
    i = 0 as DWORD;
    loop {
        if !((i as usize) < (ARRAYSIZE!(CountryTable) as usize)) {
            break;
        }
        if CountryTable[i as usize].dwCountryCode == dwCountryCode {
            if !szCountryBuffer.is_null() && dwCountryLength > 0 as DWORD {
                FormatMessageW(
                    (FORMAT_MESSAGE_FROM_HMODULE | FORMAT_MESSAGE_IGNORE_INSERTS) as DWORD,
                    hModuleNetMsg as LPCVOID,
                    CountryTable[i as usize].dwMessageId,
                    LANG_USER_DEFAULT as DWORD,
                    szCountryBuffer as LPWSTR,
                    dwCountryLength,
                    core::ptr::null_mut(),
                );
            }
            return (TRUE) as BOOL;
        }
        i = i.wrapping_add(1);
    }
    return (FALSE) as BOOL;
}

unsafe extern "C" fn GetBitValue(pBitmap: PBYTE, dwBitNumber: DWORD) -> BOOL {
    let dwIndex: DWORD = dwBitNumber / 8;
    let Mask: BYTE = ((1 as ::core::ffi::c_int) << (dwBitNumber & 7)) as BYTE;

    return ((*pBitmap.offset(dwIndex as isize) as ::core::ffi::c_int & Mask as ::core::ffi::c_int) != 0) as BOOL;
}

unsafe extern "C" fn SetBitValue(pBitmap: PBYTE, dwBitNumber: DWORD) {
    let dwIndex: DWORD = dwBitNumber / 8;
    let Mask: BYTE = ((1 as ::core::ffi::c_int) << (dwBitNumber & 7 as DWORD)) as BYTE;

    *pBitmap.offset(dwIndex as isize) = (*pBitmap.offset(dwIndex as isize) as ::core::ffi::c_int | Mask as ::core::ffi::c_int) as BYTE;
}

unsafe extern "C" fn PrintLogonHours(
    mut dwUnitsPerWeek: DWORD,
    mut pLogonHours: PBYTE,
    mut nPaddedLength: DWORD,
) {
    let mut dwUnitsPerDay: DWORD = 0;
    let mut dwBitNumber: DWORD = 0;
    let mut dwSecondsPerUnit: DWORD = 0;
    let mut dwStartTime: DWORD = 0;
    let mut dwEndTime: DWORD = 0;
    let mut dwStartDay: DWORD = 0;
    let mut dwEndDay: DWORD = 0;
    let mut dwBias: DWORD = 0;
    let mut bBitValue: BOOL = 0;
    let mut bFirst: BOOL = (TRUE) as i32;
    let mut TimeZoneInformation: TIME_ZONE_INFORMATION = core::mem::zeroed();

    GetTimeZoneInformation(core::ptr::addr_of_mut!(TimeZoneInformation));
    dwBias = ((TimeZoneInformation.Bias / (60 as LONG))
        * (SECONDS_PER_HOUR as LONG)) as DWORD;

    if (dwUnitsPerWeek == 0)
        || ((dwUnitsPerWeek % 7) != 0)
    {
        return;
    }

    dwUnitsPerDay = dwUnitsPerWeek / 7;

    if ((dwUnitsPerDay % 24) != 0)
        || ((dwUnitsPerDay / 24) > 6)
    {
        return;
    }

    dwSecondsPerUnit = ((60 * 60 * 24) as DWORD) / dwUnitsPerDay;

    dwBitNumber = 0;
    while dwBitNumber < dwUnitsPerWeek {
        bBitValue = GetBitValue(pLogonHours, dwBitNumber);
        if bBitValue != 0 {
            dwStartTime = dwSecondsPerUnit.wrapping_mul(dwBitNumber);

            while bBitValue != 0 && dwBitNumber < dwUnitsPerWeek {
                dwBitNumber = dwBitNumber.wrapping_add(1);
                if dwBitNumber < dwUnitsPerWeek {
                    bBitValue = GetBitValue(pLogonHours, dwBitNumber);
                }
            }

            dwEndTime = dwSecondsPerUnit.wrapping_mul(dwBitNumber);

            if bFirst == 0 {
                PrintPadding(0x0020u16, nPaddedLength);
            }

            if dwStartTime == 0
                && dwEndTime == ((SECONDS_PER_DAY * 7)) as u32
            {
                PrintMessageString(4302);
                ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
            } else {
                dwStartDay = dwStartTime / (SECONDS_PER_DAY) as u32;
                dwEndDay = (dwEndTime / (SECONDS_PER_DAY) as u32) % 7;

                PrintMessageString((4307 as DWORD).wrapping_add(dwStartDay));
                ConPuts(StdOut, RUSTCONV_WSTR_8D71B3FAAB82.as_ptr());

                /* Convert from GMT to local timezone */
                PrintLocalTime(
                    (dwStartTime % (SECONDS_PER_DAY) as u32).wrapping_sub(dwBias),
                );

                ConPrintf(StdOut, RUSTCONV_WSTR_A8F12694C205.as_ptr());
                if dwStartDay != dwEndDay {
                    PrintMessageString((4307 as DWORD).wrapping_add(dwEndDay));
                    ConPuts(StdOut, RUSTCONV_WSTR_8D71B3FAAB82.as_ptr());
                }

                /* Convert from GMT to local timezone */
                PrintLocalTime(
                    (dwEndTime % (SECONDS_PER_DAY) as u32).wrapping_sub(dwBias),
                );
                ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
            }

            bFirst = (FALSE) as i32;
        }
        dwBitNumber = dwBitNumber.wrapping_add(1);
    }

    if bFirst != 0 {
        /* No logon hours */
        PrintMessageString(4434);
        ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
    }
}

unsafe extern "C" fn DisplayUser(lpUserName: LPWSTR) -> DWORD {
    let mut pUserModals: PUSER_MODALS_INFO_0 = core::ptr::null_mut();
    let mut pUserInfo: PUSER_INFO_4 = core::ptr::null_mut();
    let mut pLocalGroupInfo: PLOCALGROUP_USERS_INFO_0 = core::ptr::null_mut();
    let mut pGroupInfo: PGROUP_USERS_INFO_0 = core::ptr::null_mut();
    let mut dwLocalGroupRead: DWORD = 0;
    let mut dwLocalGroupTotal: DWORD = 0;
    let mut dwGroupRead: DWORD = 0;
    let mut dwGroupTotal: DWORD = 0;
    let mut dwLastSet: DWORD = 0;
    let mut i: DWORD = 0;
    let mut szCountry: [WCHAR; 40] = [0; 40];
    let nPaddedLength: DWORD = 36;
    let mut Status: DWORD = 0;

    /* Modify the user */
    Status = NetUserGetInfo(
        core::ptr::null_mut(),
        lpUserName,
        4,
        core::ptr::addr_of_mut!(pUserInfo) as *mut LPBYTE,
    );
    if Status != (0 as DWORD) /* NERR_Success */ {
        return Status;
    }

    'done: {
        Status = NetUserModalsGet(
            core::ptr::null_mut(),
            0,
            core::ptr::addr_of_mut!(pUserModals) as *mut LPBYTE,
        );
        if Status != (0 as DWORD) /* NERR_Success */ {
            break 'done;
        }

        Status = NetUserGetLocalGroups(
            core::ptr::null_mut(),
            lpUserName,
            0,
            0,
            core::ptr::addr_of_mut!(pLocalGroupInfo) as *mut LPBYTE,
            MAX_PREFERRED_LENGTH,
            core::ptr::addr_of_mut!(dwLocalGroupRead),
            core::ptr::addr_of_mut!(dwLocalGroupTotal),
        );
        if Status != (0 as DWORD) /* NERR_Success */ {
            break 'done;
        }

        Status = NetUserGetGroups(
            core::ptr::null_mut(),
            lpUserName,
            0,
            core::ptr::addr_of_mut!(pGroupInfo) as *mut LPBYTE,
            MAX_PREFERRED_LENGTH,
            core::ptr::addr_of_mut!(dwGroupRead),
            core::ptr::addr_of_mut!(dwGroupTotal),
        );
        if Status != (0 as DWORD) /* NERR_Success */ {
            break 'done;
        }

        PrintPaddedMessageString(4411, nPaddedLength);
        ConPrintf(StdOut, RUSTCONV_WSTR_4BA5D7E4F16E.as_ptr(), (*pUserInfo).usri4_name);

        PrintPaddedMessageString(4412, nPaddedLength);
        ConPrintf(StdOut, RUSTCONV_WSTR_4BA5D7E4F16E.as_ptr(), (*pUserInfo).usri4_full_name);

        PrintPaddedMessageString(4413, nPaddedLength);
        ConPrintf(StdOut, RUSTCONV_WSTR_4BA5D7E4F16E.as_ptr(), (*pUserInfo).usri4_comment);

        PrintPaddedMessageString(4414, nPaddedLength);
        ConPrintf(StdOut, RUSTCONV_WSTR_4BA5D7E4F16E.as_ptr(), (*pUserInfo).usri4_usr_comment);

        PrintPaddedMessageString(4416, nPaddedLength);
        GetCountryFromCountryCode(
            (*pUserInfo).usri4_country_code,
            ARRAYSIZE!(szCountry),
            core::ptr::addr_of_mut!(szCountry) as PWSTR,
        );
        ConPrintf(
            StdOut,
            RUSTCONV_WSTR_74DDC21DC0C7.as_ptr(),
            (*pUserInfo).usri4_country_code,
            core::ptr::addr_of_mut!(szCountry) as *mut WCHAR,
        );

        PrintPaddedMessageString(4419, nPaddedLength);
        if ((*pUserInfo).usri4_flags & UF_ACCOUNTDISABLE) != 0 {
            PrintMessageString(4301);
        } else if ((*pUserInfo).usri4_flags & UF_LOCKOUT) != 0 {
            PrintMessageString(4440);
        } else {
            PrintMessageString(4300);
        }
        ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());

        PrintPaddedMessageString(4420, nPaddedLength);
        if (*pUserInfo).usri4_acct_expires == TIMEQ_FOREVER {
            PrintMessageString(4305);
        } else {
            PrintDateTime((*pUserInfo).usri4_acct_expires);
        }
        ConPuts(StdOut, RUSTCONV_WSTR_9D313EA1711F.as_ptr());

        PrintPaddedMessageString(4421, nPaddedLength);
        dwLastSet = GetTimeInSeconds().wrapping_sub((*pUserInfo).usri4_password_age);
        PrintDateTime(dwLastSet);
        ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());

        PrintPaddedMessageString(4422, nPaddedLength);
        if ((*pUserInfo).usri4_flags & UF_DONT_EXPIRE_PASSWD) != 0
            || (*pUserModals).usrmod0_max_passwd_age == TIMEQ_FOREVER
        {
            PrintMessageString(4305);
        } else {
            PrintDateTime(dwLastSet.wrapping_add((*pUserModals).usrmod0_max_passwd_age));
        }
        ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());

        PrintPaddedMessageString(4423, nPaddedLength);
        PrintDateTime(dwLastSet.wrapping_add((*pUserModals).usrmod0_min_passwd_age));
        ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());

        PrintPaddedMessageString(4437, nPaddedLength);
        PrintMessageString(
            (if ((*pUserInfo).usri4_flags & UF_PASSWD_NOTREQD) != 0 {
                4301
            } else {
                4300
            }) as DWORD,
        );
        ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());

        PrintPaddedMessageString(4438, nPaddedLength);
        PrintMessageString(
            (if ((*pUserInfo).usri4_flags & UF_PASSWD_CANT_CHANGE) != 0 {
                4301
            } else {
                4300
            }) as DWORD,
        );
        ConPuts(StdOut, RUSTCONV_WSTR_9D313EA1711F.as_ptr());

        PrintPaddedMessageString(4424, nPaddedLength);
        if (*pUserInfo).usri4_workstations.is_null()
            || wcslen((*pUserInfo).usri4_workstations) == 0
        {
            PrintMessageString(4302);
        } else {
            ConPrintf(StdOut, RUSTCONV_WSTR_FB47CCC40FE6.as_ptr(), (*pUserInfo).usri4_workstations);
        }
        ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());

        PrintPaddedMessageString(4429, nPaddedLength);
        ConPrintf(StdOut, RUSTCONV_WSTR_4BA5D7E4F16E.as_ptr(), (*pUserInfo).usri4_script_path);

        PrintPaddedMessageString(4439, nPaddedLength);
        ConPrintf(StdOut, RUSTCONV_WSTR_4BA5D7E4F16E.as_ptr(), (*pUserInfo).usri4_profile);

        PrintPaddedMessageString(4436, nPaddedLength);
        ConPrintf(StdOut, RUSTCONV_WSTR_4BA5D7E4F16E.as_ptr(), (*pUserInfo).usri4_home_dir);

        PrintPaddedMessageString(4430, nPaddedLength);
        if (*pUserInfo).usri4_last_logon == 0 {
            PrintMessageString(4305);
        } else {
            PrintDateTime((*pUserInfo).usri4_last_logon);
        }
        ConPuts(StdOut, RUSTCONV_WSTR_9D313EA1711F.as_ptr());

        PrintPaddedMessageString(4432, nPaddedLength);
        if (*pUserInfo).usri4_logon_hours.is_null() {
            PrintMessageString(4302);
            ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
        } else {
            PrintLogonHours(
                (*pUserInfo).usri4_units_per_week,
                (*pUserInfo).usri4_logon_hours,
                nPaddedLength,
            );
        }

        ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
        PrintPaddedMessageString(4427, nPaddedLength);
        if dwLocalGroupTotal != 0 && !pLocalGroupInfo.is_null() {
            i = 0;
            while i < dwLocalGroupTotal {
                if i != 0 {
                    PrintPadding(0x0020u16, nPaddedLength);
                }
                ConPrintf(
                    StdOut,
                    RUSTCONV_WSTR_F118FF86CE3B.as_ptr(),
                    (*pLocalGroupInfo.offset(i as isize)).lgrui0_name,
                );
                i = i.wrapping_add(1);
            }
        } else {
            ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
        }

        PrintPaddedMessageString(4431, nPaddedLength);
        if dwGroupTotal != 0 && !pGroupInfo.is_null() {
            i = 0;
            while i < dwGroupTotal {
                if i != 0 {
                    PrintPadding(0x0020u16, nPaddedLength);
                }
                ConPrintf(
                    StdOut,
                    RUSTCONV_WSTR_F118FF86CE3B.as_ptr(),
                    (*pGroupInfo.offset(i as isize)).grui0_name,
                );
                i = i.wrapping_add(1);
            }
        } else {
            ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
        }
    }

    /* done: */
    if !pGroupInfo.is_null() {
        NetApiBufferFree(pGroupInfo as PVOID);
    }

    if !pLocalGroupInfo.is_null() {
        NetApiBufferFree(pLocalGroupInfo as PVOID);
    }

    if !pUserModals.is_null() {
        NetApiBufferFree(pUserModals as PVOID);
    }

    if !pUserInfo.is_null() {
        NetApiBufferFree(pUserInfo as PVOID);
    }

    return 0 as DWORD; /* NERR_Success */
}

unsafe extern "C" fn ReadPassword(lpPassword: *mut LPWSTR, lpAllocated: LPBOOL) {
    let mut szPassword1: [WCHAR; (PWLEN + 1) as _] = [0; (PWLEN + 1) as _];
    let mut szPassword2: [WCHAR; (PWLEN + 1) as _] = [0; (PWLEN + 1) as _];
    let mut ptr: LPWSTR;

    *lpAllocated = (FALSE) as i32;

    loop {
        PrintMessageString(4358);
        ReadFromConsole(core::ptr::addr_of_mut!(szPassword1) as LPWSTR, PWLEN + 1, (FALSE) as BOOL);
        ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());

        PrintMessageString(4361);
        ReadFromConsole(core::ptr::addr_of_mut!(szPassword2) as LPWSTR, PWLEN + 1, (FALSE) as BOOL);
        ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());

        if wcslen(core::ptr::addr_of_mut!(szPassword1) as *mut WCHAR) == wcslen(core::ptr::addr_of_mut!(szPassword2) as *mut WCHAR)
            && wcscmp(core::ptr::addr_of_mut!(szPassword1) as *mut WCHAR, core::ptr::addr_of_mut!(szPassword2) as *mut WCHAR) == 0
        {
            ptr = HeapAlloc(
                GetProcessHeap(),
                0,
                ((wcslen(core::ptr::addr_of_mut!(szPassword1) as *mut WCHAR) as SIZE_T) + (1 as SIZE_T)) * (core::mem::size_of::<WCHAR>() as SIZE_T),
            ) as LPWSTR;
            if !ptr.is_null() {
                wcscpy(ptr as *mut WCHAR, core::ptr::addr_of_mut!(szPassword1) as *mut WCHAR);
                *lpPassword = ptr;
                *lpAllocated = (TRUE) as i32;
                return;
            }
        } else {
            ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
            PrintMessageString(3728);
            *lpPassword = core::ptr::null_mut();
        }
    }
}

unsafe extern "C" fn GenerateRandomPassword(lpPassword: *mut LPWSTR, lpAllocated: LPBOOL) {
    let mut pPassword: LPWSTR = core::ptr::null_mut::<WCHAR>();
    let mut nCharsLen: size_t = 0;
    let mut i: size_t = 0;
    let mut nLength: size_t = 8 as size_t;
    srand(GetTickCount() as core::ffi::c_uint);
    pPassword = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY as DWORD,
        (nLength as SIZE_T)
            .wrapping_add(1 as SIZE_T)
            .wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
    ) as LPWSTR;
    if pPassword.is_null() {
        return;
    }
    nCharsLen = wcslen(core::ptr::addr_of_mut!(szPasswordChars) as *mut WCHAR) as size_t;
    i = 0 as size_t;
    while i < nLength {
        *pPassword.offset(i as isize) =
            szPasswordChars[(rand() as size_t).wrapping_rem(nCharsLen) as usize];
        i = i.wrapping_add(1);
    }
    *lpPassword = pPassword;
    *lpAllocated = TRUE as BOOL;
}

unsafe extern "C" fn BuildWorkstationsList(
    pWorkstationsList: *mut PWSTR,
    pRaw: PWSTR,
) -> DWORD {
    let mut isLastSep: BOOL = 0;
    let mut isSep: BOOL = 0;
    let mut c: WCHAR = 0;
    let mut i: size_t = 0;
    let mut j: size_t = 0;
    let mut nLength: size_t = 0;
    let mut nRawLength: size_t = 0;
    let mut nArgs: UINT = 0;
    let mut pList: PWSTR = core::ptr::null_mut();

    /* Check for invalid characters in the raw string */
    if !wcspbrk(pRaw, RUSTCONV_WSTR_BCE2AEEE903F.as_ptr()).is_null() {
        return 3952;
    }

    /* Count the number of workstations in the list and
     * the required buffer size */
    isLastSep = (FALSE) as i32;
    isSep = (FALSE) as i32;
    nRawLength = wcslen(pRaw) as size_t;
    i = 0;
    while i < nRawLength {
        c = *pRaw.add(i);
        if c == 0x002Cu16 || c == 0x003Bu16 {
            isSep = (TRUE) as i32;
        }

        if (isSep) as u32 == TRUE {
            if ((isLastSep) as u32 == FALSE)
                && (i != 0)
                && (i != nRawLength.wrapping_sub(1))
            {
                nLength = nLength.wrapping_add(1);
            }
        } else {
            nLength = nLength.wrapping_add(1);

            if (isLastSep) as u32 == TRUE
                || ((isLastSep) as u32 == FALSE && i == 0)
            {
                nArgs = nArgs.wrapping_add(1);
            }
        }

        isLastSep = isSep;
        isSep = (FALSE) as i32;
        i = i.wrapping_add(1);
    }

    nLength = nLength.wrapping_add(1);

    /* Leave, if there are no workstations in the list */
    if nArgs == 0 {
        *pWorkstationsList = core::ptr::null_mut();
        return 0 as DWORD; // NERR_Success
    }

    /* Fail if there are more than eight workstations in the list */
    if nArgs > 8 {
        return 3951;
    }

    /* Allocate the buffer for the clean workstation list */
    pList = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        (nLength.wrapping_mul(core::mem::size_of::<WCHAR>())) as SIZE_T,
    ) as PWSTR;
    if pList.is_null() {
        return ERROR_NOT_ENOUGH_MEMORY;
    }

    /* Build the clean workstation list */
    isLastSep = (FALSE) as i32;
    isSep = (FALSE) as i32;
    nRawLength = wcslen(pRaw) as size_t;
    i = 0;
    j = 0;
    while i < nRawLength {
        c = *pRaw.add(i);
        if c == 0x002Cu16 || c == 0x003Bu16 {
            isSep = (TRUE) as i32;
        }

        if (isSep) as u32 == TRUE {
            if ((isLastSep) as u32 == FALSE)
                && (i != 0)
                && (i != nRawLength.wrapping_sub(1))
            {
                *pList.add(j) = 0x002Cu16;
                j = j.wrapping_add(1);
            }
        } else {
            *pList.add(j) = c;
            j = j.wrapping_add(1);

            if (isLastSep) as u32 == TRUE
                || ((isLastSep) as u32 == FALSE && i == 0)
            {
                nArgs = nArgs.wrapping_add(1);
            }
        }

        isLastSep = isSep;
        isSep = (FALSE) as i32;
        i = i.wrapping_add(1);
    }

    *pWorkstationsList = pList;

    return 0 as DWORD; // NERR_Success
}

unsafe extern "C" fn ReadNumber(s: *mut PWSTR, pwValue: PWORD) -> BOOL {
    if iswdigit(**s) == 0 {
        return (FALSE) as BOOL;
    }

    while iswdigit(**s) != 0 {
        *pwValue = ((*pwValue as i32) * 10 + (**s as i32) - (0x0030u16 as i32)) as WORD;
        *s = (*s).offset(1);
    }

    return (TRUE) as BOOL;
}

unsafe extern "C" fn ReadSeparator(s: *mut PWSTR) -> BOOL {
    if **s == 0x002Fu16 || **s == 0x002Eu16
    {
        *s = (*s).add(1);
        return (TRUE) as BOOL;
    }

    return (FALSE) as BOOL;
}

unsafe extern "C" fn ParseDate(s: PWSTR, pSeconds: PULONG) -> BOOL {
    let mut SystemTime: SYSTEMTIME = core::mem::zeroed();
    let mut LocalFileTime: FILETIME = core::mem::zeroed();
    let mut FileTime: FILETIME = core::mem::zeroed();
    let mut Time: LARGE_INTEGER = core::mem::zeroed();
    let mut nDateFormat: INT = 0;
    let mut p: PWSTR = s;

    if *s == 0 {
        return (FALSE) as BOOL;
    }

    // Divergence from the C: it asks for LOCALE_IDATE without LOCALE_RETURN_NUMBER, so it gets
    // the STRING L"0"/L"1"/L"2" (nDateFormat = 0x30..) and every locale fell to the mmddyy
    // default; and it passes sizeof(INT) as cchData, which counts WCHARs (8 bytes into a 4-byte
    // INT). Ask for the number, sized in WCHARs.
    GetLocaleInfoW(
        LOCALE_USER_DEFAULT,
        LOCALE_IDATE | LOCALE_RETURN_NUMBER,
        core::ptr::addr_of_mut!(nDateFormat) as PWSTR,
        (core::mem::size_of::<INT>() / core::mem::size_of::<WCHAR>()) as i32,
    );

    match nDateFormat {
        1 => {
            if ReadNumber(core::ptr::addr_of_mut!(p), core::ptr::addr_of_mut!(SystemTime.wDay)) == 0 {
                return FALSE as BOOL;
            }
            if ReadSeparator(core::ptr::addr_of_mut!(p)) == 0 {
                return FALSE as BOOL;
            }
            if ReadNumber(core::ptr::addr_of_mut!(p), core::ptr::addr_of_mut!(SystemTime.wMonth)) == 0 {
                return FALSE as BOOL;
            }
            if ReadSeparator(core::ptr::addr_of_mut!(p)) == 0 {
                return FALSE as BOOL;
            }
            if ReadNumber(core::ptr::addr_of_mut!(p), core::ptr::addr_of_mut!(SystemTime.wYear)) == 0 {
                return FALSE as BOOL;
            }
        }
        2 => {
            if ReadNumber(core::ptr::addr_of_mut!(p), core::ptr::addr_of_mut!(SystemTime.wYear)) == 0 {
                return FALSE as BOOL;
            }
            if ReadSeparator(core::ptr::addr_of_mut!(p)) == 0 {
                return FALSE as BOOL;
            }
            if ReadNumber(core::ptr::addr_of_mut!(p), core::ptr::addr_of_mut!(SystemTime.wMonth)) == 0 {
                return FALSE as BOOL;
            }
            if ReadSeparator(core::ptr::addr_of_mut!(p)) == 0 {
                return FALSE as BOOL;
            }
            if ReadNumber(core::ptr::addr_of_mut!(p), core::ptr::addr_of_mut!(SystemTime.wDay)) == 0 {
                return FALSE as BOOL;
            }
        }
        // C: `case 0: default:` comes FIRST in the switch; a Rust match takes the first
        // arm that matches, so `0 | _` up there swallowed the dd/mm/yy and yy/mm/dd locales.
        _ => {
            if ReadNumber(core::ptr::addr_of_mut!(p), core::ptr::addr_of_mut!(SystemTime.wMonth)) == 0 {
                return (FALSE) as BOOL;
            }
            if ReadSeparator(core::ptr::addr_of_mut!(p)) == 0 {
                return (FALSE) as BOOL;
            }
            if ReadNumber(core::ptr::addr_of_mut!(p), core::ptr::addr_of_mut!(SystemTime.wDay)) == 0 {
                return (FALSE) as BOOL;
            }
            if ReadSeparator(core::ptr::addr_of_mut!(p)) == 0 {
                return (FALSE) as BOOL;
            }
            if ReadNumber(core::ptr::addr_of_mut!(p), core::ptr::addr_of_mut!(SystemTime.wYear)) == 0 {
                return (FALSE) as BOOL;
            }
        }
    }

    /* if only entered two digits: */
    /*   assume 2000's if value less than 80 */
    /*   assume 1900's if value greater or equal 80 */
    if (SystemTime.wYear as INT) <= 99 {
        if (SystemTime.wYear as INT) >= 80 {
            SystemTime.wYear = ((SystemTime.wYear as INT) + 1900) as WORD;
        } else {
            SystemTime.wYear = ((SystemTime.wYear as INT) + 2000) as WORD;
        }
    }

    if SystemTimeToFileTime(core::ptr::addr_of_mut!(SystemTime), core::ptr::addr_of_mut!(LocalFileTime)) == 0 {
        return FALSE as BOOL;
    }

    if LocalFileTimeToFileTime(core::ptr::addr_of_mut!(LocalFileTime), core::ptr::addr_of_mut!(FileTime)) == 0 {
        return FALSE as BOOL;
    }

    Time.u.LowPart = FileTime.dwLowDateTime as DWORD;
    Time.u.HighPart = FileTime.dwHighDateTime as LONG;

    if RtlTimeToSecondsSince1970(core::ptr::addr_of_mut!(Time), pSeconds) == 0 {
        return FALSE as BOOL;
    }

    return TRUE as BOOL;
}

unsafe extern "C" fn ParseHour(
    pszString: PWSTR,
    AmPmArray: *mut PWSTR,
    plHour: PLONG,
) -> BOOL {
    let mut pChar: PWCHAR = core::ptr::null_mut();
    let mut lHour: LONG = 0;

    if iswdigit(*pszString.offset(0 as isize) as wint_t) == 0 {
        return (FALSE) as BOOL;
    }

    pChar = pszString;
    while iswdigit(*pChar as wint_t) != 0 {
        lHour = lHour * 10 + (*pChar as LONG) - (0x0030u16 as LONG);
        pChar = pChar.offset(1);
    }

    if lHour > 24 {
        return (FALSE) as BOOL;
    }

    if lHour == 24 {
        lHour = 0;
    }

    if (*pChar != UNICODE_NULL)
        && (lHour >= 1)
        && (lHour <= 12)
    {
        if (_wcsicmp(pChar, *AmPmArray.offset(0 as isize)) == 0)
            || (_wcsicmp(pChar, *AmPmArray.offset(1 as isize)) == 0)
        {
            if lHour == 12 {
                lHour = 0;
            }
        } else if (_wcsicmp(pChar, *AmPmArray.offset(2 as isize)) == 0)
            || (_wcsicmp(pChar, *AmPmArray.offset(3 as isize)) == 0)
        {
            if lHour != 12 {
                lHour += 12;
            }
        } else {
            return (FALSE) as BOOL;
        }
    }

    *plHour = lHour;

    return (TRUE) as BOOL;
}

unsafe extern "C" fn ParseDay(
    mut pszString: PWSTR,
    mut ShortDays: *mut PWSTR,
    mut LongDays: *mut PWSTR,
    mut pdwDay: PDWORD,
) -> BOOL {
    let mut i: DWORD;
    i = 0;
    while i < 7 {
        if _wcsicmp(pszString, *ShortDays.offset(i as isize)) == 0
            || _wcsicmp(pszString, *LongDays.offset(i as isize)) == 0
        {
            *pdwDay = i;
            return (TRUE) as BOOL;
        }
        i = i.wrapping_add(1);
    }

    return (FALSE) as BOOL;
}

unsafe extern "C" fn LocalToGmtHour(mut lLocalHour: LONG, mut lBias: LONG) -> DWORD {
    let mut lGmtHour: LONG = 0;
    lGmtHour = lLocalHour + lBias;
    if lGmtHour < 0 as LONG {
        lGmtHour += UNITS_PER_WEEK as LONG;
    } else if lGmtHour > UNITS_PER_WEEK as LONG {
        lGmtHour -= UNITS_PER_WEEK as LONG;
    }
    return lGmtHour as DWORD;
}

unsafe extern "C" fn ParseLogonHours(
    mut pszParams: PWSTR,
    mut ppLogonBitmap: *mut PBYTE,
    mut pdwUnitsPerWeek: PDWORD,
) -> DWORD {
    let mut TimeZoneInformation: TIME_ZONE_INFORMATION = ::core::mem::zeroed();
    let mut pLogonBitmap: PBYTE = core::ptr::null_mut();
    let mut dwError: DWORD = ERROR_SUCCESS;
    let mut szBuffer: [WCHAR; 32] = [0; 32];
    let mut ptr1: PWSTR = core::ptr::null_mut();
    let mut ptr2: PWSTR = core::ptr::null_mut();
    let mut prevSep: WCHAR = 0;
    let mut nextSep: WCHAR = 0;
    let mut dwStartDay: DWORD = 0;
    let mut dwEndDay: DWORD = 0;
    let mut i: DWORD = 0;
    let mut j: DWORD = 0;
    let mut lStartHour: LONG = 0;
    let mut lEndHour: LONG = 0;
    let mut lBias: LONG = 0;
    let mut DayBitmap: BYTE = 0;
    let mut HourBitmap: [BYTE; 6] = [0; 6];
    let mut ShortDays: [LPWSTR; 7] = [core::ptr::null_mut(); 7];
    let mut LongDays: [LPWSTR; 7] = [core::ptr::null_mut(); 7];
    let mut AmPmArray: [LPWSTR; 4] = [core::ptr::null_mut(); 4];

    GetTimeZoneInformation(core::ptr::addr_of_mut!(TimeZoneInformation));
    lBias = TimeZoneInformation.Bias / 60;

    pLogonBitmap = HeapAlloc(GetProcessHeap(),
                             HEAP_ZERO_MEMORY,
                             (UNITS_PER_WEEK / 8) as SIZE_T) as PBYTE;
    if pLogonBitmap.is_null() {
        return ERROR_OUTOFMEMORY;
    }

    'done: {
        if *pszParams == UNICODE_NULL {
            break 'done;
        }

        if _wcsicmp(pszParams, RUSTCONV_WSTR_5295D3F8988C.as_ptr()) == 0 {
            FillMemory!(pLogonBitmap, UNITS_PER_WEEK / 8, 0xFF);
            break 'done;
        }

        i = 0;
        while i < 7 {
            FormatMessageW(FORMAT_MESSAGE_ALLOCATE_BUFFER |
                           FORMAT_MESSAGE_FROM_HMODULE |
                           FORMAT_MESSAGE_IGNORE_INSERTS,
                           hModuleNetMsg as LPCVOID,
                           4314 + i,
                           LANG_USER_DEFAULT,
                           core::ptr::addr_of_mut!(ShortDays[i as usize]) as LPWSTR,
                           0,
                           core::ptr::null_mut());

            FormatMessageW(FORMAT_MESSAGE_ALLOCATE_BUFFER |
                           FORMAT_MESSAGE_FROM_HMODULE |
                           FORMAT_MESSAGE_IGNORE_INSERTS,
                           hModuleNetMsg as LPCVOID,
                           4307 + i,
                           LANG_USER_DEFAULT,
                           core::ptr::addr_of_mut!(LongDays[i as usize]) as LPWSTR,
                           0,
                           core::ptr::null_mut());
            i += 1;
        }

        i = 0;
        while i < 4 {
            FormatMessageW(FORMAT_MESSAGE_ALLOCATE_BUFFER |
                           FORMAT_MESSAGE_FROM_HMODULE |
                           FORMAT_MESSAGE_IGNORE_INSERTS,
                           hModuleNetMsg as LPCVOID,
                           4322 + i,
                           LANG_USER_DEFAULT,
                           core::ptr::addr_of_mut!(AmPmArray[i as usize]) as LPWSTR,
                           0,
                           core::ptr::null_mut());
            i += 1;
        }

        ZeroMemory!(core::ptr::addr_of_mut!(DayBitmap) as *mut core::ffi::c_void, core::mem::size_of::<BYTE>());
        ZeroMemory!(core::ptr::addr_of_mut!(HourBitmap) as *mut core::ffi::c_void, core::mem::size_of::<[BYTE; 6]>());

        ZeroMemory!(core::ptr::addr_of_mut!(szBuffer) as *mut core::ffi::c_void, core::mem::size_of::<[WCHAR; 32]>());
        ptr1 = pszParams;
        ptr2 = core::ptr::addr_of_mut!(szBuffer) as PWSTR;
        prevSep = UNICODE_NULL;
        nextSep = UNICODE_NULL;
        loop {
            if *ptr1 != 0x002Du16 && *ptr1 != 0x002Cu16 && *ptr1 != 0x003Bu16 && *ptr1 != UNICODE_NULL {
                *ptr2 = *ptr1;
                ptr2 = ptr2.offset(1);
            } else {
                prevSep = nextSep;
                nextSep = *ptr1;

                if prevSep != 0x002Du16 {
                    /* Set first value */
                    if iswdigit(szBuffer[0 as usize]) != 0 {
                        /* Parse hour */
                        if ParseHour(core::ptr::addr_of_mut!(szBuffer) as PWSTR,
                                                  core::ptr::addr_of_mut!(AmPmArray) as *mut PWSTR,
                                                  core::ptr::addr_of_mut!(lStartHour)) == 0 {
                            dwError = 3769;
                            break;
                        }

                        SetBitValue(core::ptr::addr_of_mut!(HourBitmap) as PBYTE, LocalToGmtHour(lStartHour, lBias));
                    } else {
                        /* Parse day */
                        if ParseDay(core::ptr::addr_of_mut!(szBuffer) as PWSTR,
                                                  core::ptr::addr_of_mut!(ShortDays) as *mut PWSTR,
                                                  core::ptr::addr_of_mut!(LongDays) as *mut PWSTR,
                                                  core::ptr::addr_of_mut!(dwStartDay)) == 0 {
                            dwError = 3768;
                            break;
                        }

                        SetBitValue(core::ptr::addr_of_mut!(DayBitmap) as PBYTE, dwStartDay);
                    }
                } else {
                    /* Set second value */
                    if iswdigit(szBuffer[0 as usize]) != 0 {
                        /* Parse hour */
                        if ParseHour(core::ptr::addr_of_mut!(szBuffer) as PWSTR,
                                                  core::ptr::addr_of_mut!(AmPmArray) as *mut PWSTR,
                                                  core::ptr::addr_of_mut!(lEndHour)) == 0 {
                            dwError = 3769;
                            break;
                        }

                        if lEndHour <= lStartHour {
                            lEndHour += HOURS_PER_DAY;
                        }

                        i = LocalToGmtHour(lStartHour, lBias);
                        while i < LocalToGmtHour(lEndHour, lBias) {
                            SetBitValue(core::ptr::addr_of_mut!(HourBitmap) as PBYTE, i);
                            i += 1;
                        }
                    } else {
                        /* Parse day */
                        if ParseDay(core::ptr::addr_of_mut!(szBuffer) as PWSTR,
                                                  core::ptr::addr_of_mut!(ShortDays) as *mut PWSTR,
                                                  core::ptr::addr_of_mut!(LongDays) as *mut PWSTR,
                                                  core::ptr::addr_of_mut!(dwEndDay)) == 0 {
                            dwError = 3768;
                            break;
                        }

                        if dwEndDay <= dwStartDay {
                            dwEndDay += (DAYS_PER_WEEK as u32);
                        }

                        i = dwStartDay;
                        while i <= dwEndDay {
                            SetBitValue(core::ptr::addr_of_mut!(DayBitmap) as PBYTE, i % (DAYS_PER_WEEK as u32));
                            i += 1;
                        }
                    }
                }

                if *ptr1 == 0x003Bu16 || *ptr1 == UNICODE_NULL {
                    /* Fill the logon hour bitmap */
                    i = 0;
                    while i < (DAYS_PER_WEEK as u32) {
                        if GetBitValue(core::ptr::addr_of_mut!(DayBitmap) as PBYTE, i) != 0 {
                            j = 0;
                            while j < 48 {
                                if GetBitValue(core::ptr::addr_of_mut!(HourBitmap) as PBYTE, j) != 0 {
                                    SetBitValue(pLogonBitmap, ((i * (HOURS_PER_DAY as u32)) + j) % UNITS_PER_WEEK);
                                }
                                j += 1;
                            }
                        }
                        i += 1;
                    }

                    /* Reset the Bitmaps */
                    ZeroMemory!(core::ptr::addr_of_mut!(DayBitmap) as *mut core::ffi::c_void, core::mem::size_of::<BYTE>());
                    ZeroMemory!(core::ptr::addr_of_mut!(HourBitmap) as *mut core::ffi::c_void, core::mem::size_of::<[BYTE; 6]>());
                }

                if *ptr1 == UNICODE_NULL {
                    break;
                }

                ZeroMemory!(core::ptr::addr_of_mut!(szBuffer) as *mut core::ffi::c_void, core::mem::size_of::<[WCHAR; 32]>());
                ptr2 = core::ptr::addr_of_mut!(szBuffer) as PWSTR;
            }

            ptr1 = ptr1.offset(1);
        }
    }

    i = 0;
    while i < 7 {
        LocalFree(ShortDays[i as usize] as HLOCAL);
        LocalFree(LongDays[i as usize] as HLOCAL);
        i += 1;
    }

    i = 0;
    while i < 4 {
        LocalFree(AmPmArray[i as usize] as HLOCAL);
        i += 1;
    }

    if dwError == ERROR_SUCCESS {
        *ppLogonBitmap = pLogonBitmap;
        *pdwUnitsPerWeek = UNITS_PER_WEEK;
    } else {
        if !pLogonBitmap.is_null() {
            HeapFree(GetProcessHeap(), 0, pLogonBitmap as PVOID);
        }
        *ppLogonBitmap = core::ptr::null_mut();
        *pdwUnitsPerWeek = 0;
    }

    return dwError;
}
#[no_mangle]
pub unsafe extern "C" fn cmdUser(argc: INT, argv: *mut *mut WCHAR) -> INT {
    let mut i: INT = 2;
    let mut j: INT = 0;
    let mut result: INT = 0;
    let mut bAdd: BOOL = (FALSE) as i32;
    let mut bDelete: BOOL = (FALSE) as i32;
    let mut bRandomPassword: BOOL = (FALSE) as i32;
    let mut lpUserName: LPWSTR = core::ptr::null_mut();
    let mut lpPassword: LPWSTR = core::ptr::null_mut();
    let mut pUserInfo: PUSER_INFO_4 = core::ptr::null_mut();
    let mut UserInfo: USER_INFO_4 = core::mem::zeroed();
    let mut pWorkstations: LPWSTR = core::ptr::null_mut();
    let mut p: LPWSTR = core::ptr::null_mut();
    let mut endptr: LPWSTR = core::ptr::null_mut();
    let mut value: DWORD = 0;
    let mut bPasswordAllocated: BOOL = (FALSE) as i32;
    let mut pLogonHours: PBYTE = core::ptr::null_mut();
    let mut dwUnitsPerWeek: DWORD = 0;
    let mut Status: DWORD = 0;

    'done: {
        i = 2;
        if i < argc && *(*argv.offset(i as isize)).offset(0 as isize) != 0x002Fu16 {
            lpUserName = *argv.offset(i as isize);
            i += 1;
        }

        if i < argc && *(*argv.offset(i as isize)).offset(0 as isize) != 0x002Fu16 {
            lpPassword = *argv.offset(i as isize);
            i += 1;
        }

        j = i;
        while j < argc {
            if _wcsicmp(*argv.offset(j as isize), RUSTCONV_WSTR_1A749FECCEE7.as_ptr()) == 0 {
                PrintNetMessage(MSG_USER_HELP);
                return 0;
            } else if _wcsicmp(*argv.offset(j as isize), RUSTCONV_WSTR_5D12D1C1410E.as_ptr()) == 0 {
                bAdd = (TRUE) as i32;
            } else if _wcsicmp(*argv.offset(j as isize), RUSTCONV_WSTR_7E7219CF4FF7.as_ptr()) == 0 {
                bDelete = (TRUE) as i32;
            } else if _wcsicmp(*argv.offset(j as isize), RUSTCONV_WSTR_9BB9C2E30775.as_ptr()) == 0 {
                ConPuts(core::ptr::addr_of_mut!(csStdErr), RUSTCONV_WSTR_582BA4648587.as_ptr());
            } else if _wcsicmp(*argv.offset(j as isize), RUSTCONV_WSTR_0F163FD255A4.as_ptr()) == 0 {
                bRandomPassword = (TRUE) as i32;
                GenerateRandomPassword(core::ptr::addr_of_mut!(lpPassword),
                                       core::ptr::addr_of_mut!(bPasswordAllocated));
            }
            j += 1;
        }

        if lpUserName.is_null() && lpPassword.is_null() {
            Status = EnumerateUsers();
            PrintMessageString(TranslateAppMessage(Status));
            return 0;
        } else if !lpUserName.is_null() && lpPassword.is_null() && argc == 3 {
            Status = DisplayUser(lpUserName);
            PrintMessageString(TranslateAppMessage(Status));
            return 0;
        }

        if bAdd != 0 && bDelete != 0 {
            result = 1;
            break 'done;
        }

        /* Interactive password input */
        if !lpPassword.is_null() && wcscmp(lpPassword, RUSTCONV_WSTR_E8A4B2EE7EDE.as_ptr()) == 0 {
            ReadPassword(core::ptr::addr_of_mut!(lpPassword),
                                   core::ptr::addr_of_mut!(bPasswordAllocated));
        }

        if bAdd == 0 && bDelete == 0 {
            /* Modify the user */
            Status = NetUserGetInfo(core::ptr::null_mut(),
                                            lpUserName,
                                            4,
                                            core::ptr::addr_of_mut!(pUserInfo) as *mut LPBYTE);
            if Status != NERR_Success {
                PrintMessageString(TranslateAppMessage(Status));
                result = 1;
                break 'done;
            }
        } else if bAdd != 0 {
            /* Add the user */
            ZeroMemory!(core::ptr::addr_of_mut!(UserInfo) as *mut core::ffi::c_void, core::mem::size_of::<USER_INFO_4>() as usize);

            UserInfo.usri4_name = lpUserName;
            UserInfo.usri4_password = lpPassword;
            UserInfo.usri4_flags = UF_SCRIPT | UF_NORMAL_ACCOUNT;
            UserInfo.usri4_acct_expires = TIMEQ_FOREVER;
            UserInfo.usri4_primary_group_id = DOMAIN_GROUP_RID_USERS;

            pUserInfo = core::ptr::addr_of_mut!(UserInfo);
        }

        j = i;
        while j < argc {
            if _wcsnicmp(*argv.offset(j as isize), RUSTCONV_WSTR_1A52D437A1CE.as_ptr(), 8) == 0 {
                p = (*argv.offset(i as isize)).offset(8 as isize);
                if _wcsicmp(p, RUSTCONV_WSTR_6D1618CC9381.as_ptr()) == 0 {
                    (*pUserInfo).usri4_flags &= !(UF_ACCOUNTDISABLE);
                } else if _wcsicmp(p, RUSTCONV_WSTR_833AD7EF6992.as_ptr()) == 0 {
                    (*pUserInfo).usri4_flags |= UF_ACCOUNTDISABLE;
                } else {
                    PrintMessageStringV(3952, RUSTCONV_WSTR_ECEFEF436E2E.as_ptr());
                    result = 1;
                    break 'done;
                }
            } else if _wcsnicmp(*argv.offset(j as isize), RUSTCONV_WSTR_00C0DB009DDA.as_ptr(), 9) == 0 {
                (*pUserInfo).usri4_comment = (*argv.offset(j as isize)).offset(9 as isize);
            } else if _wcsnicmp(*argv.offset(j as isize), RUSTCONV_WSTR_D1DAA4B12576.as_ptr(), 13) == 0 {
                p = (*argv.offset(i as isize)).offset(13 as isize);
                value = wcstoul(p, core::ptr::addr_of_mut!(endptr), 10);
                if *endptr != 0 {
                    PrintMessageStringV(3952, RUSTCONV_WSTR_580D24B22F7C.as_ptr());
                    result = 1;
                    break 'done;
                }

                /* Verify the country code */
                if GetCountryFromCountryCode(value, 0, core::ptr::null_mut()) != 0 {
                    (*pUserInfo).usri4_country_code = value;
                }
            } else if _wcsnicmp(*argv.offset(j as isize), RUSTCONV_WSTR_A5AB18086796.as_ptr(), 9) == 0 {
                p = (*argv.offset(i as isize)).offset(9 as isize);
                if _wcsicmp(p, RUSTCONV_WSTR_0BE94F48150F.as_ptr()) == 0 {
                    (*pUserInfo).usri4_acct_expires = TIMEQ_FOREVER;
                } else if ParseDate(p, core::ptr::addr_of_mut!((*pUserInfo).usri4_acct_expires)) == 0 {
                    PrintMessageStringV(3952, RUSTCONV_WSTR_A9A634858003.as_ptr());
                    result = 1;
                    break 'done;
                }
            } else if _wcsnicmp(*argv.offset(j as isize), RUSTCONV_WSTR_5197BE528E07.as_ptr(), 10) == 0 {
                (*pUserInfo).usri4_full_name = (*argv.offset(j as isize)).offset(10 as isize);
            } else if _wcsnicmp(*argv.offset(j as isize), RUSTCONV_WSTR_1B0FB29105DF.as_ptr(), 9) == 0 {
                (*pUserInfo).usri4_home_dir = (*argv.offset(j as isize)).offset(9 as isize);
            } else if _wcsnicmp(*argv.offset(j as isize), RUSTCONV_WSTR_A032BD847C02.as_ptr(), 13) == 0 {
                p = (*argv.offset(i as isize)).offset(13 as isize);
                if _wcsicmp(p, RUSTCONV_WSTR_6D1618CC9381.as_ptr()) == 0 {
                    (*pUserInfo).usri4_flags &= !(UF_PASSWD_CANT_CHANGE);
                } else if _wcsicmp(p, RUSTCONV_WSTR_833AD7EF6992.as_ptr()) == 0 {
                    (*pUserInfo).usri4_flags |= UF_PASSWD_CANT_CHANGE;
                } else {
                    PrintMessageStringV(3952, RUSTCONV_WSTR_C441979A38EC.as_ptr());
                    result = 1;
                    break 'done;
                }
            } else if _wcsnicmp(*argv.offset(j as isize), RUSTCONV_WSTR_17084575BCAC.as_ptr(), 13) == 0 {
                p = (*argv.offset(i as isize)).offset(13 as isize);
                if _wcsicmp(p, RUSTCONV_WSTR_6D1618CC9381.as_ptr()) == 0 {
                    (*pUserInfo).usri4_flags &= !(UF_PASSWD_NOTREQD);
                } else if _wcsicmp(p, RUSTCONV_WSTR_833AD7EF6992.as_ptr()) == 0 {
                    (*pUserInfo).usri4_flags |= UF_PASSWD_NOTREQD;
                } else {
                    PrintMessageStringV(3952, RUSTCONV_WSTR_D90EC374D009.as_ptr());
                    result = 1;
                    break 'done;
                }
            } else if _wcsnicmp(*argv.offset(j as isize), RUSTCONV_WSTR_CBFCEEE03B06.as_ptr(), 13) == 0 {
                (*pUserInfo).usri4_profile = (*argv.offset(j as isize)).offset(13 as isize);
            } else if _wcsnicmp(*argv.offset(j as isize), RUSTCONV_WSTR_329A042C9CFB.as_ptr(), 12) == 0 {
                (*pUserInfo).usri4_script_path = (*argv.offset(j as isize)).offset(12 as isize);
            } else if _wcsnicmp(*argv.offset(j as isize), RUSTCONV_WSTR_0AD474BF6806.as_ptr(), 7) == 0 {
                Status = ParseLogonHours((*argv.offset(j as isize)).offset(7 as isize) as PWSTR,
                                                core::ptr::addr_of_mut!(pLogonHours),
                                                core::ptr::addr_of_mut!(dwUnitsPerWeek));
                if Status == ERROR_SUCCESS {
                    (*pUserInfo).usri4_logon_hours = pLogonHours;
                    (*pUserInfo).usri4_units_per_week = dwUnitsPerWeek;
                } else {
                    PrintMessageString(Status);
                    break 'done;
                }
            } else if _wcsnicmp(*argv.offset(j as isize), RUSTCONV_WSTR_8CE542C39882.as_ptr(), 13) == 0 {
                (*pUserInfo).usri4_usr_comment = (*argv.offset(j as isize)).offset(13 as isize);
            } else if _wcsnicmp(*argv.offset(j as isize), RUSTCONV_WSTR_0DE3843AC6EC.as_ptr(), 14) == 0 {
                p = (*argv.offset(i as isize)).offset(14 as isize);
                if wcscmp(p, RUSTCONV_WSTR_E8A4B2EE7EDE.as_ptr()) == 0 || wcscmp(p, RUSTCONV_WSTR_96A296D224F2.as_ptr()) == 0 {
                    (*pUserInfo).usri4_workstations = core::ptr::null_mut();
                } else {
                    Status = BuildWorkstationsList(core::ptr::addr_of_mut!(pWorkstations), p);
                    if Status == NERR_Success {
                        (*pUserInfo).usri4_workstations = pWorkstations;
                    } else {
                        PrintMessageString(TranslateAppMessage(Status));
                        result = 1;
                        break 'done;
                    }
                }
            }
            j += 1;
        }

        if bAdd == 0 && bDelete == 0 {
            /* Modify the user */
            Status = NetUserSetInfo(core::ptr::null_mut(),
                                            lpUserName,
                                            4,
                                            pUserInfo as LPBYTE,
                                            core::ptr::null_mut());
        } else if bAdd != 0 {
            /* Add the user */
            Status = NetUserAdd(core::ptr::null_mut(),
                                            4,
                                            pUserInfo as LPBYTE,
                                            core::ptr::null_mut());
        } else if bDelete != 0 {
            /* Delete the user */
            Status = NetUserDel(core::ptr::null_mut(),
                                            lpUserName);
        }

        PrintMessageString(TranslateAppMessage(Status));

        if Status == NERR_Success &&
            !lpPassword.is_null() &&
            (bRandomPassword) as u32 == TRUE
        {
            PrintMessageStringV(3968, lpUserName, lpPassword);
        }
    }

    /* done: */
    if !pLogonHours.is_null() {
        HeapFree(GetProcessHeap(), 0, pLogonHours as PVOID);
    }

    if !pWorkstations.is_null() {
        HeapFree(GetProcessHeap(), 0, pWorkstations as PVOID);
    }

    if (bPasswordAllocated) as u32 == TRUE && !lpPassword.is_null() {
        HeapFree(GetProcessHeap(), 0, lpPassword as PVOID);
    }

    if bAdd == 0 && bDelete == 0 && !pUserInfo.is_null() {
        NetApiBufferFree(pUserInfo as PVOID);
    }

    if result != 0 {
        PrintMessageString(4381);
        ConPuts(core::ptr::addr_of_mut!(csStdOut), RUSTCONV_WSTR_075DE2B906DB.as_ptr());
        PrintNetMessage(MSG_USER_SYNTAX);
    }

    return result;
}
