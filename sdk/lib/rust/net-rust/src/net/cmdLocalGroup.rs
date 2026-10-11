use crate::net::main::PrintErrorMessage;
use crate::net::main::PrintMessageString;
use crate::net::main::PrintNetMessage;
use crate::net::main::PrintPaddedMessageString;
use crate::net::main::PrintPadding;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_00C0DB009DDA: [u16; 10] = [0x002Fu16, 0x0063, 0x006F, 0x006D, 0x006D, 0x0065, 0x006E, 0x0074, 0x003A, 0x0000];
pub static RUSTCONV_WSTR_075DE2B906DB: [u16; 2] = [0x000Au16, 0x0000];
pub static RUSTCONV_WSTR_1A749FECCEE7: [u16; 6] = [0x002Fu16, 0x0068, 0x0065, 0x006C, 0x0070, 0x0000];
pub static RUSTCONV_WSTR_30D9768121E6: [u16; 19] = [
    0x004Du16, 0x0065, 0x006D, 0x0062, 0x0065, 0x0072, 0x0020, 0x0063, 0x006F, 0x0075,
    0x006E, 0x0074, 0x003A, 0x0020, 0x0025, 0x006C, 0x0075, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_4BA5D7E4F16E: [u16; 4] = [0x0025u16, 0x0073, 0x000A, 0x0000];
pub static RUSTCONV_WSTR_582BA4648587: [u16; 42] = [
    0x0054u16, 0x0068, 0x0065, 0x0020, 0x002F, 0x0044, 0x004F, 0x004D, 0x0041, 0x0049,
    0x004E, 0x0020, 0x006F, 0x0070, 0x0074, 0x0069, 0x006F, 0x006E, 0x0020, 0x0069,
    0x0073, 0x0020, 0x006E, 0x006F, 0x0074, 0x0020, 0x0073, 0x0075, 0x0070, 0x0070,
    0x006F, 0x0072, 0x0074, 0x0065, 0x0064, 0x0020, 0x0079, 0x0065, 0x0074, 0x002E,
    0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_5D12D1C1410E: [u16; 5] = [0x002Fu16, 0x0061, 0x0064, 0x0064, 0x0000];
pub static RUSTCONV_WSTR_7E7219CF4FF7: [u16; 8] = [0x002Fu16, 0x0064, 0x0065, 0x006C, 0x0065, 0x0074, 0x0065, 0x0000];
pub static RUSTCONV_WSTR_9BB9C2E30775: [u16; 8] = [0x002Fu16, 0x0064, 0x006F, 0x006D, 0x0061, 0x0069, 0x006E, 0x0000];
pub static RUSTCONV_WSTR_BE93C907DAAC: [u16; 13] = [
    0x0053u16, 0x0074, 0x0061, 0x0074, 0x0075, 0x0073, 0x003A, 0x0020, 0x0025, 0x006C,
    0x0075, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_F118FF86CE3B: [u16; 5] = [0x002Au16, 0x0025, 0x0073, 0x000A, 0x0000];

use core::ffi::c_void;


unsafe extern "C" fn CompareInfo(
    a: *const core::ffi::c_void,
    b: *const core::ffi::c_void,
) -> core::ffi::c_int {
    return _wcsicmp(
        (*(a as PLOCALGROUP_INFO_0)).lgrpi0_name as *const u16,
        (*(b as PLOCALGROUP_INFO_0)).lgrpi0_name as *const u16,
    );
}

unsafe extern "C" fn EnumerateLocalGroups() -> NET_API_STATUS {
    let mut pBuffer: PLOCALGROUP_INFO_0 = core::ptr::null_mut();
    let mut pServer: PSERVER_INFO_100 = core::ptr::null_mut();
    let mut dwRead: DWORD = 0;
    let mut dwTotal: DWORD = 0;
    let mut i: DWORD;
    let mut ResumeHandle: DWORD_PTR = 0;
    let mut Status: NET_API_STATUS;

    Status = NetServerGetInfo(
        core::ptr::null_mut(),
        100,
        core::ptr::addr_of_mut!(pServer) as *mut LPBYTE,
    );
    if Status != NERR_Success {
        return Status;
    }

    ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
    PrintMessageStringV(4405, (*pServer).sv100_name);
    ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
    PrintPadding((0x002Du16) as WCHAR, 79);
    ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());

    NetApiBufferFree(pServer as PVOID);

    Status = NetLocalGroupEnum(
        core::ptr::null_mut(),
        0,
        core::ptr::addr_of_mut!(pBuffer) as *mut LPBYTE,
        MAX_PREFERRED_LENGTH,
        core::ptr::addr_of_mut!(dwRead),
        core::ptr::addr_of_mut!(dwTotal),
        core::ptr::addr_of_mut!(ResumeHandle),
    );
    if Status != NERR_Success {
        return Status;
    }

    qsort(
        pBuffer as *mut core::ffi::c_void,
        dwRead as usize,
        core::mem::size_of::<PLOCALGROUP_INFO_0>(),
        Some(CompareInfo),
    );

    i = 0;
    while i < dwRead {
        if !(*pBuffer.offset(i as isize)).lgrpi0_name.is_null() {
            ConPrintf(
                StdOut,
                RUSTCONV_WSTR_F118FF86CE3B.as_ptr(),
                (*pBuffer.offset(i as isize)).lgrpi0_name,
            );
        }
        i = i.wrapping_add(1);
    }

    NetApiBufferFree(pBuffer as PVOID);

    return NERR_Success;
}

unsafe extern "C" fn DisplayLocalGroup(lpGroupName: LPWSTR) -> NET_API_STATUS {
    let mut pGroupInfo: PLOCALGROUP_INFO_1 = core::ptr::null_mut();
    let mut pMembers: PLOCALGROUP_MEMBERS_INFO_3 = core::ptr::null_mut();
    let mut pServer: PSERVER_INFO_100 = core::ptr::null_mut();
    let mut pNames: *mut LPWSTR = core::ptr::null_mut();
    let mut dwRead: DWORD = 0;
    let mut dwTotal: DWORD = 0;
    let mut ResumeHandle: DWORD_PTR = 0;
    let mut i: DWORD;
    let mut len: usize;
    let nPaddedLength: DWORD = 18;
    let mut Status: NET_API_STATUS;

    Status = NetLocalGroupGetInfo(
        core::ptr::null(),
        lpGroupName,
        1,
        core::ptr::addr_of_mut!(pGroupInfo) as *mut LPBYTE,
    );
    if Status != NERR_Success {
        return Status;
    }

    'done: {
        Status = NetLocalGroupGetMembers(
            core::ptr::null(),
            lpGroupName,
            3,
            core::ptr::addr_of_mut!(pMembers) as *mut LPBYTE,
            MAX_PREFERRED_LENGTH,
            core::ptr::addr_of_mut!(dwRead),
            core::ptr::addr_of_mut!(dwTotal),
            core::ptr::addr_of_mut!(ResumeHandle),
        );
        if Status != NERR_Success {
            break 'done;
        }

        Status = NetServerGetInfo(
            core::ptr::null_mut(),
            100,
            core::ptr::addr_of_mut!(pServer) as *mut LPBYTE,
        );
        if Status != NERR_Success {
            break 'done;
        }

        pNames = RtlAllocateHeap(
            RtlGetProcessHeap!(),
            HEAP_ZERO_MEMORY,
            (dwRead as SIZE_T).wrapping_mul(core::mem::size_of::<LPWSTR>() as SIZE_T),
        ) as *mut LPWSTR;
        if pNames.is_null() {
            Status = ERROR_OUTOFMEMORY;
            break 'done;
        }

        len = wcslen((*pServer).sv100_name) as usize;
        i = 0;
        while i < dwRead {
            if wcsncmp(
                (*pMembers.offset(i as isize)).lgrmi3_domainandname,
                (*pServer).sv100_name,
                len,
            ) == 0
            {
                *pNames.offset(i as isize) = (*pMembers.offset(i as isize))
                    .lgrmi3_domainandname
                    .add(len + 1 as usize);
            } else {
                *pNames.offset(i as isize) = (*pMembers.offset(i as isize)).lgrmi3_domainandname;
            }
            i += 1;
        }

        PrintPaddedMessageString(4406, nPaddedLength);
        ConPrintf(StdOut, RUSTCONV_WSTR_4BA5D7E4F16E.as_ptr(), (*pGroupInfo).lgrpi1_name);

        PrintPaddedMessageString(4407, nPaddedLength);
        ConPrintf(StdOut, RUSTCONV_WSTR_4BA5D7E4F16E.as_ptr(), (*pGroupInfo).lgrpi1_comment);

        ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());

        PrintMessageString(4408);
        ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());

        PrintPadding(0x002Du16, 79);
        ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());

        i = 0;
        while i < dwRead {
            if !(*pNames.offset(i as isize)).is_null() {
                ConPrintf(StdOut, RUSTCONV_WSTR_4BA5D7E4F16E.as_ptr(), *pNames.offset(i as isize));
            }
            i += 1;
        }
    }

    if !pNames.is_null() {
        RtlFreeHeap(RtlGetProcessHeap!(), 0, pNames as PVOID);
    }

    if !pServer.is_null() {
        NetApiBufferFree(pServer as PVOID);
    }

    if !pMembers.is_null() {
        NetApiBufferFree(pMembers as PVOID);
    }

    if !pGroupInfo.is_null() {
        NetApiBufferFree(pGroupInfo as PVOID);
    }

    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn cmdLocalGroup(mut argc: INT, mut argv: *mut *mut WCHAR) -> INT {
    let mut i: INT = 0;
    let mut j: INT = 0;
    let mut result: INT = 0;
    let mut dwMemberCount: ULONG = 0;
    let mut bAdd: BOOL = (FALSE) as i32;
    let mut bDelete: BOOL = (FALSE) as i32;
    let mut lpGroupName: LPWSTR = ::core::ptr::null_mut::<WCHAR>();
    let mut lpComment: LPWSTR = ::core::ptr::null_mut::<WCHAR>();
    let mut lpMembers: LPLOCALGROUP_MEMBERS_INFO_3 =
        ::core::ptr::null_mut::<_LOCALGROUP_MEMBERS_INFO_3>();
    let mut Info0: LOCALGROUP_INFO_0 = _LOCALGROUP_INFO_0{
        lgrpi0_name: ::core::ptr::null_mut::<WCHAR>(),
    };
    let mut Info1: LOCALGROUP_INFO_1 = _LOCALGROUP_INFO_1 {
        lgrpi1_name: ::core::ptr::null_mut::<WCHAR>(),
        lgrpi1_comment: ::core::ptr::null_mut::<WCHAR>(),
    };
    let mut Info1002: LOCALGROUP_INFO_1002 = _LOCALGROUP_INFO_1002 {
        lgrpi1002_comment: ::core::ptr::null_mut::<WCHAR>(),
    };
    let mut Status: NET_API_STATUS = 0;

    if argc == 2 {
        Status = EnumerateLocalGroups();
        ConPrintf(
            &raw mut csStdOut,
            RUSTCONV_WSTR_BE93C907DAAC.as_ptr(),
            Status,
        );
        return 0;
    } else if argc == 3 {
        Status = DisplayLocalGroup(*argv.offset(2 as isize) as LPWSTR);
        ConPrintf(
            &raw mut csStdOut,
            RUSTCONV_WSTR_BE93C907DAAC.as_ptr(),
            Status,
        );
        return 0;
    }

    i = 2;
    if *(*argv.offset(i as isize)).offset(0 as isize) != 0x002Fu16 {
        lpGroupName = *argv.offset(i as isize) as LPWSTR;
        i += 1;
    }

    j = i;
    while j < argc {
        if *(*argv.offset(j as isize)).offset(0 as isize) == 0x002Fu16 {
            break;
        }
        dwMemberCount = dwMemberCount.wrapping_add(1);
        j += 1;
    }

    ConPrintf(
        &raw mut csStdOut,
        RUSTCONV_WSTR_30D9768121E6.as_ptr(),
        dwMemberCount,
    );

    if dwMemberCount > 0 {
        lpMembers = RtlAllocateHeap(
            RtlGetProcessHeap!(),
            HEAP_ZERO_MEMORY,
            (dwMemberCount as SIZE_T).wrapping_mul(core::mem::size_of::<LPLOCALGROUP_MEMBERS_INFO_3>() as SIZE_T),
        ) as LPLOCALGROUP_MEMBERS_INFO_3;
        if lpMembers.is_null() {
            return 0;
        }
    }

    j = 0;
    while i < argc {
        if *(*argv.offset(i as isize)).offset(0 as isize) == 0x002Fu16 {
            break;
        }
        (*lpMembers.offset(j as isize)).lgrmi3_domainandname = *argv.offset(i as isize) as LPWSTR;
        j += 1;
        i += 1;
    }

    'done: {
        while i < argc {
            if _wcsicmp(*argv.offset(i as isize), RUSTCONV_WSTR_1A749FECCEE7.as_ptr()) == 0 {
                PrintMessageString(4381);
                ConPuts(
                    &raw mut csStdOut,
                    RUSTCONV_WSTR_075DE2B906DB.as_ptr(),
                );
                PrintNetMessage(MSG_LOCALGROUP_SYNTAX);
                PrintNetMessage(MSG_LOCALGROUP_HELP);
                return 0;
            } else if _wcsicmp(*argv.offset(i as isize), RUSTCONV_WSTR_5D12D1C1410E.as_ptr()) == 0 {
                bAdd = (TRUE) as i32;
            } else if _wcsicmp(*argv.offset(i as isize), RUSTCONV_WSTR_7E7219CF4FF7.as_ptr()) == 0 {
                bDelete = (TRUE) as i32;
            } else if _wcsnicmp(
                *argv.offset(i as isize),
                RUSTCONV_WSTR_00C0DB009DDA.as_ptr(),
                9,
            ) == 0 {
                lpComment = (*argv.offset(i as isize)).offset(9 as isize) as LPWSTR;
            } else if _wcsicmp(*argv.offset(i as isize), RUSTCONV_WSTR_9BB9C2E30775.as_ptr()) == 0 {
                ConPuts(
                    &raw mut csStdErr,
                    RUSTCONV_WSTR_582BA4648587.as_ptr(),
                );
            } else {
                PrintErrorMessage(3506);
                result = 1;
                break 'done;
            }
            i += 1;
        }

        if lpGroupName.is_null() {
            result = 1;
            break 'done;
        }

        if bAdd != 0 && bDelete != 0 {
            result = 1;
            break 'done;
        }

        if lpMembers.is_null() {
            if bAdd == 0 && bDelete == 0 && !lpComment.is_null() {
                /* Set group comment */
                Info1002.lgrpi1002_comment = lpComment;
                Status = NetLocalGroupSetInfo(
                    core::ptr::null::<WCHAR>(),
                    lpGroupName as LPCWSTR,
                    1002 as DWORD,
                    &raw mut Info1002 as PBYTE,
                    core::ptr::null_mut::<DWORD>(),
                );
                ConPrintf(
                    &raw mut csStdOut,
                    RUSTCONV_WSTR_BE93C907DAAC.as_ptr(),
                    Status,
                );
            } else if bAdd != 0 && bDelete == 0 {
                /* Add the group */
                if lpComment.is_null() {
                    Info0.lgrpi0_name = lpGroupName;
                } else {
                    Info1.lgrpi1_name = lpGroupName;
                    Info1.lgrpi1_comment = lpComment;
                }

                Status = NetLocalGroupAdd(
                    core::ptr::null::<WCHAR>(),
                    (if lpComment.is_null() {
                        0
                    } else {
                        1
                    }) as DWORD,
                    if lpComment.is_null() {
                        &raw mut Info0 as PBYTE
                    } else {
                        &raw mut Info1 as PBYTE
                    },
                    core::ptr::null_mut::<DWORD>(),
                );
                ConPrintf(
                    &raw mut csStdOut,
                    RUSTCONV_WSTR_BE93C907DAAC.as_ptr(),
                    Status,
                );
            } else if bAdd == 0 && bDelete != 0 && lpComment.is_null() {
                /* Delete the group */
                Status = NetLocalGroupDel(
                    core::ptr::null::<WCHAR>(),
                    lpGroupName as LPCWSTR,
                );
                ConPrintf(
                    &raw mut csStdOut,
                    RUSTCONV_WSTR_BE93C907DAAC.as_ptr(),
                    Status,
                );
            } else {
                result = 1;
            }
        } else {
            if bAdd != 0 && bDelete == 0 && lpComment.is_null() {
                /* Add group members */
                Status = NetLocalGroupAddMembers(
                    core::ptr::null::<WCHAR>(),
                    lpGroupName as LPCWSTR,
                    3 as DWORD,
                    lpMembers as PBYTE,
                    dwMemberCount as DWORD,
                );
                ConPrintf(
                    &raw mut csStdOut,
                    RUSTCONV_WSTR_BE93C907DAAC.as_ptr(),
                    Status,
                );
            } else if bAdd == 0 && bDelete != 0 && lpComment.is_null() {
                /* Delete group members */
                Status = NetLocalGroupDelMembers(
                    core::ptr::null::<WCHAR>(),
                    lpGroupName as LPCWSTR,
                    3 as DWORD,
                    lpMembers as PBYTE,
                    dwMemberCount as DWORD,
                );
                ConPrintf(
                    &raw mut csStdOut,
                    RUSTCONV_WSTR_BE93C907DAAC.as_ptr(),
                    Status,
                );
            } else {
                result = 1;
            }
        }
    }

    // done:
    if !lpMembers.is_null() {
        RtlFreeHeap(
            RtlGetProcessHeap!(),
            0,
            lpMembers as PVOID,
        );
    }

    if result != 0 {
        PrintMessageString(4381);
        ConPuts(
            &raw mut csStdOut,
            RUSTCONV_WSTR_075DE2B906DB.as_ptr(),
        );
        PrintNetMessage(MSG_LOCALGROUP_SYNTAX);
    }

    return result;
}
