use crate::net::main::PrintErrorMessage;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_047F1ECC6E7F: [u16; 15] = [
    0x0025u16, 0x0073, 0x0009, 0x0009, 0x0009, 0x0025, 0x0073, 0x0009, 0x0009, 0x0009,
    0x0009, 0x0025, 0x0073, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_3E45CCDFE961: [u16; 9] = [0x0050u16, 0x0072, 0x006F, 0x0076, 0x0069, 0x0064, 0x0065, 0x0072, 0x0000];
pub static RUSTCONV_WSTR_5801DD79EE5D: [u16; 11] = [
    0x0050u16, 0x0065, 0x0072, 0x0073, 0x0069, 0x0073, 0x0074, 0x0065, 0x006E, 0x0074,
    0x0000,
];
pub static RUSTCONV_WSTR_76E480513E2C: [u16; 5] = [0x004Eu16, 0x0061, 0x006D, 0x0065, 0x0000];
pub static RUSTCONV_WSTR_841288FC0FB0: [u16; 11] = [
    0x0044u16, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x004E, 0x0061, 0x006D, 0x0065,
    0x0000,
];
pub static RUSTCONV_WSTR_96A296D224F2: [u16; 1] = [0x0000u16];
pub static RUSTCONV_WSTR_B195A53249B1: [u16; 7] = [0x0052u16, 0x0065, 0x006D, 0x006F, 0x0074, 0x0065, 0x0000];
pub static RUSTCONV_WSTR_B22FE3B7C457: [u16; 13] = [
    0x002Fu16, 0x0050, 0x0045, 0x0052, 0x0053, 0x0049, 0x0053, 0x0054, 0x0045, 0x004E,
    0x0054, 0x003A, 0x0000,
];
pub static RUSTCONV_WSTR_C338ABC2AD66: [u16; 4] = [0x000Au16, 0x0025, 0x0073, 0x0000];
pub static RUSTCONV_WSTR_DC40AA356E0B: [u16; 6] = [0x004Cu16, 0x006F, 0x0063, 0x0061, 0x006C, 0x0000];
pub static RUSTCONV_WSTR_F237D6F4AAD5: [u16; 13] = [
    0x0025u16, 0x0073, 0x0009, 0x0009, 0x0009, 0x0025, 0x0073, 0x0009, 0x0009, 0x0025,
    0x0073, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_F3C6A8F2AC47: [u16; 4] = [0x0025u16, 0x006C, 0x0075, 0x0000];
pub static RUSTCONV_WSTR_F3CD7CDBC9E2: [u16; 8] = [0x002Fu16, 0x0044, 0x0045, 0x004C, 0x0045, 0x0054, 0x0045, 0x0000];

use core::ffi::c_void;

unsafe extern "C" fn EnumerateConnections(Local: LPCWSTR) -> DWORD {
    let mut dRet: DWORD;
    let mut hEnum: HANDLE = core::ptr::null_mut();
    let mut lpRes: LPNETRESOURCE;
    let mut dSize: DWORD = 0x1000;
    let mut dCount: DWORD = (-(1 as ::core::ffi::c_int)) as DWORD;
    let mut lpCur: LPNETRESOURCE;

    ConPrintf(
        StdOut,
        RUSTCONV_WSTR_047F1ECC6E7F.as_ptr(),
        RUSTCONV_WSTR_DC40AA356E0B.as_ptr(),
        RUSTCONV_WSTR_B195A53249B1.as_ptr(),
        RUSTCONV_WSTR_3E45CCDFE961.as_ptr(),
    );

    dRet = WNetOpenEnum(
        RESOURCE_CONNECTED,
        RESOURCETYPE_DISK,
        0,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(hEnum),
    );
    if dRet != WN_SUCCESS {
        return 1;
    }

    lpRes = HeapAlloc(
        GetProcessHeap(),
        0,
        dSize as SIZE_T,
    ) as LPNETRESOURCE;
    if lpRes.is_null() {
        WNetCloseEnum(hEnum);
        return 1;
    }

    loop {
        dSize = 0x1000;
        dCount = (-(1 as ::core::ffi::c_int)) as DWORD;

        memset(
            lpRes as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            dSize as usize,
        );
        dRet = WNetEnumResource(
            hEnum,
            core::ptr::addr_of_mut!(dCount),
            lpRes as LPVOID,
            core::ptr::addr_of_mut!(dSize),
        );
        if dRet == WN_SUCCESS || dRet == WN_MORE_DATA {
            lpCur = lpRes;
            while dCount != 0 {
                if Local.is_null()
                    || _wcsicmp((*lpCur).lpLocalName, Local) == 0
                {
                    ConPrintf(
                        StdOut,
                        RUSTCONV_WSTR_F237D6F4AAD5.as_ptr(),
                        (*lpCur).lpLocalName,
                        (*lpCur).lpRemoteName,
                        (*lpCur).lpProvider,
                    );
                }

                lpCur = lpCur.add(1);
                dCount = dCount.wrapping_sub(1);
            }
        }

        if !(dRet != WN_NO_MORE_ENTRIES) {
            break;
        }
    }

    HeapFree(
        GetProcessHeap(),
        0,
        lpRes as PVOID,
    );
    WNetCloseEnum(hEnum);

    return 0;
}

unsafe extern "C" fn PrintError(Status: DWORD) {
    let mut szStatusBuffer: [WCHAR; 16] = [0; 16];
    let mut Buffer: LPWSTR = core::ptr::null_mut();

    _swprintf(core::ptr::addr_of_mut!(szStatusBuffer) as *mut WCHAR, RUSTCONV_WSTR_F3C6A8F2AC47.as_ptr(), Status);
    PrintMessageStringV(3502 as DWORD, core::ptr::addr_of_mut!(szStatusBuffer) as *mut WCHAR);

    if FormatMessageW(
        (FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_IGNORE_INSERTS) as DWORD,
        core::ptr::null(),
        Status,
        LANG_USER_DEFAULT as DWORD,
        core::ptr::addr_of_mut!(Buffer) as LPWSTR,
        0 as DWORD,
        core::ptr::null_mut(),
    ) != 0
    {
        ConPrintf(StdErr, RUSTCONV_WSTR_C338ABC2AD66.as_ptr(), Buffer);
        LocalFree(Buffer as HLOCAL);
    }
}

unsafe extern "C" fn ValidateDeviceName(DevName: PWSTR) -> BOOL {
    return ((*DevName.offset(0 as isize) != 0)
        && (iswalpha(*DevName.offset(0 as isize)) != 0)
        && (*DevName.offset(1 as isize) == 0x003Au16)
        && (*DevName.offset(2 as isize) == 0)) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn cmdUse(argc: INT, argv: *mut *mut WCHAR) -> INT {
    let mut Status: DWORD;
    let mut Delete: DWORD;

    if argc == 2 {
        Status = EnumerateConnections(core::ptr::null());
        if Status == NO_ERROR {
            PrintErrorMessage(ERROR_SUCCESS as DWORD);
        } else {
            PrintError(Status);
        }

        return 0;
    } else if argc == 3 {
        if ValidateDeviceName(*argv.offset(2 as isize)) == 0 {
            PrintMessageStringV(3952 as DWORD, RUSTCONV_WSTR_841288FC0FB0.as_ptr());
            return 1;
        }

        Status = EnumerateConnections(*argv.offset(2 as isize));
        if Status == NO_ERROR {
            PrintErrorMessage(ERROR_SUCCESS as DWORD);
        } else {
            PrintError(Status);
        }

        return 0;
    }

    Delete = 0;
    if _wcsicmp(*argv.offset(2 as isize), RUSTCONV_WSTR_F3CD7CDBC9E2.as_ptr()) == 0 {
        Delete = 3;
    } else {
        if (*(*argv.offset(2 as isize)).offset(0 as isize) != '*' as WCHAR
            && *(*argv.offset(2 as isize)).offset(1 as isize) != 0 as WCHAR)
            && ValidateDeviceName(*argv.offset(2 as isize)) == 0
        {
            PrintMessageStringV(3952 as DWORD, RUSTCONV_WSTR_841288FC0FB0.as_ptr());
            return 1;
        }
    }

    if _wcsicmp(*argv.offset(3 as isize), RUSTCONV_WSTR_F3CD7CDBC9E2.as_ptr()) == 0 {
        Delete = 2;
    }

    if Delete != 0 {
        if ValidateDeviceName(*argv.offset(Delete as isize)) == 0
            || *(*argv.offset(Delete as isize)).offset(0 as isize) == 0x002Au16
        {
            PrintMessageStringV(3952 as DWORD, RUSTCONV_WSTR_841288FC0FB0.as_ptr());
            return 1;
        }

        Status = WNetCancelConnection2(*argv.offset(Delete as isize), CONNECT_UPDATE_PROFILE, (FALSE) as i32);
        if Status != NO_ERROR {
            PrintError(Status);
        }

        return Status as INT;
    } else {
        let mut Persist: BOOL = (FALSE) as i32;
        let mut lpNet: NETRESOURCE = core::mem::zeroed();
        let mut Access: [WCHAR; 256] = [0; 256];
        let mut OutFlags: DWORD = 0;
        let mut Size: DWORD = ARRAYSIZE!(Access) as DWORD;
        let mut Len: usize;

        Len = wcslen(*argv.offset(3 as isize)) as usize;
        if Len < 4 {
            PrintMessageStringV(3952 as DWORD, RUSTCONV_WSTR_76E480513E2C.as_ptr());
            return 1;
        }

        if *(*argv.offset(3 as isize)).offset(0 as isize) != 0x005Cu16
            || *(*argv.offset(3 as isize)).offset(1 as isize) != 0x005Cu16
        {
            PrintMessageStringV(3952 as DWORD, RUSTCONV_WSTR_76E480513E2C.as_ptr());
            return 1;
        }

        if argc > 4 {
            let mut Cpy: LPWSTR = core::ptr::null_mut();
            Len = wcslen(*argv.offset(4 as isize)) as usize;
            if Len > 12 {
                Cpy = HeapAlloc(
                    GetProcessHeap(),
                    0 as DWORD,
                    ((Len + 1) * core::mem::size_of::<WCHAR>()) as SIZE_T,
                ) as LPWSTR;
                if !Cpy.is_null() {
                    let mut i: usize;
                    i = 0;
                    while i < Len {
                        *Cpy.offset(i as isize) = towupper(
                            (*(*argv.offset(4 as isize)).offset(i as isize) as wint_t) as _,
                        ) as WCHAR;
                        i += 1;
                    }

                    if (wcsstr(Cpy, RUSTCONV_WSTR_B22FE3B7C457.as_ptr()) as LPWSTR) == Cpy {
                        let Arg: LPWSTR = Cpy.offset(12 as isize);
                        if Len == 14
                            && *Arg.offset(0 as isize) == 'N' as WCHAR
                            && *Arg.offset(1 as isize) == 'O' as WCHAR
                        {
                            Persist = (FALSE) as i32;
                        } else if Len == 15
                            && *Arg.offset(0 as isize) == 'Y' as WCHAR
                            && *Arg.offset(1 as isize) == 'E' as WCHAR
                            && *Arg.offset(2 as isize) == 'S' as WCHAR
                        {
                            Persist = (TRUE) as i32;
                        } else {
                            HeapFree(GetProcessHeap(), 0 as DWORD, Cpy as PVOID);
                            PrintMessageStringV(3952 as DWORD, RUSTCONV_WSTR_5801DD79EE5D.as_ptr());
                            return 1;
                        }
                    }
                    HeapFree(GetProcessHeap(), 0 as DWORD, Cpy as PVOID);
                }
            }
        }

        lpNet.dwType = RESOURCETYPE_DISK as DWORD;
        lpNet.lpLocalName = if *(*argv.offset(2 as isize)).offset(0 as isize) != 0x002Au16 {
            *argv.offset(2 as isize)
        } else {
            core::ptr::null_mut()
        };
        lpNet.lpRemoteName = *argv.offset(3 as isize);
        lpNet.lpComment = RUSTCONV_WSTR_96A296D224F2.as_ptr() as LPWSTR;
        lpNet.lpProvider = core::ptr::null_mut();

        Status = WNetUseConnection(
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(lpNet),
            core::ptr::null(),
            core::ptr::null(),
            (CONNECT_REDIRECT | (if Persist != 0 { CONNECT_UPDATE_PROFILE } else { 0 })) as DWORD,
            core::ptr::addr_of_mut!(Access) as LPWSTR,
            core::ptr::addr_of_mut!(Size),
            core::ptr::addr_of_mut!(OutFlags),
        );
        if *(*argv.offset(2 as isize)).offset(0 as isize) == 0x002Au16
            && Status == NO_ERROR
            && OutFlags == CONNECT_LOCALDRIVE
        {
            PrintMessageStringV(
                3919 as DWORD,
                *argv.offset(3 as isize),
                core::ptr::addr_of_mut!(Access) as *mut WCHAR,
            );
        } else if Status != NO_ERROR {
            PrintError(Status);
        }

        return Status as INT;
    }
}
