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

unsafe extern "C" fn CompareInfo(a: *const core::ffi::c_void, b: *const core::ffi::c_void) -> core::ffi::c_int {
    return _wcsicmp(
        (*(a as PGROUP_INFO_0)).grpi0_name as *const u16,
        (*(b as PGROUP_INFO_0)).grpi0_name as *const u16,
    );
}

unsafe extern "C" fn EnumerateGroups() -> DWORD {
    let mut pBuffer: PGROUP_INFO_0 = core::ptr::null_mut::<_GROUP_INFO_0>();
    let mut pServer: PSERVER_INFO_100 = core::ptr::null_mut::<_SERVER_INFO_100>();
    let mut dwRead: DWORD = 0;
    let mut dwTotal: DWORD = 0;
    let mut i: DWORD;
    let mut ResumeHandle: DWORD_PTR = 0;
    let mut Status: DWORD;

    Status = NetServerGetInfo(
        core::ptr::null_mut::<WCHAR>(),
        100,
        &raw mut pServer as *mut LPBYTE,
    );
    if Status != NERR_Success as DWORD {
        return Status;
    }

    ConPuts(&raw mut csStdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
    PrintMessageStringV(4400, (*pServer).sv100_name);
    ConPuts(&raw mut csStdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
    PrintPadding(0x002Du16, 79);
    ConPuts(&raw mut csStdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());

    NetApiBufferFree(pServer as PVOID);

    Status = NetGroupEnum(
        core::ptr::null::<WCHAR>(),
        0,
        &raw mut pBuffer as *mut LPBYTE,
        MAX_PREFERRED_LENGTH,
        &raw mut dwRead,
        &raw mut dwTotal,
        &raw mut ResumeHandle,
    );
    if Status != NERR_Success as DWORD {
        return Status;
    }

    qsort(
        pBuffer as *mut core::ffi::c_void,
        dwRead as size_t,
        core::mem::size_of::<PGROUP_INFO_0>() as size_t,
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
        if !(*pBuffer.offset(i as isize)).grpi0_name.is_null() {
            ConPrintf(
                &raw mut csStdOut,
                RUSTCONV_WSTR_F118FF86CE3B.as_ptr(),
                (*pBuffer.offset(i as isize)).grpi0_name,
            );
        }
        i = i.wrapping_add(1);
    }

    NetApiBufferFree(pBuffer as PVOID);

    return NERR_Success as DWORD;
}

unsafe extern "C" fn DisplayGroup(lpGroupName: LPWSTR) -> NET_API_STATUS {
    let mut pGroupInfo: PGROUP_INFO_1 = core::ptr::null_mut();
    let mut pUsers: PGROUP_USERS_INFO_0 = core::ptr::null_mut();
    let mut pNames: *mut LPWSTR = core::ptr::null_mut();
    let mut dwRead: DWORD = 0;
    let mut dwTotal: DWORD = 0;
    let mut ResumeHandle: DWORD_PTR = 0;
    let mut i: DWORD = 0;
    let nPaddedLength: DWORD = 15;
    let mut Status: NET_API_STATUS = 0;

    Status = NetGroupGetInfo(
        core::ptr::null_mut(),
        lpGroupName,
        1,
        core::ptr::addr_of_mut!(pGroupInfo) as *mut LPBYTE,
    );
    if Status != NERR_Success {
        return Status;
    }

    Status = NetGroupGetUsers(
        core::ptr::null_mut(),
        lpGroupName,
        0,
        core::ptr::addr_of_mut!(pUsers) as *mut LPBYTE,
        MAX_PREFERRED_LENGTH,
        core::ptr::addr_of_mut!(dwRead),
        core::ptr::addr_of_mut!(dwTotal),
        core::ptr::addr_of_mut!(ResumeHandle),
    );

    'done: {
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

        i = 0;
        while i < dwRead {
            *pNames.offset(i as isize) = (*pUsers.offset(i as isize)).grui0_name;
            i = i.wrapping_add(1);
        }

        PrintPaddedMessageString(4401, nPaddedLength);
        ConPrintf(StdOut, RUSTCONV_WSTR_4BA5D7E4F16E.as_ptr(), (*pGroupInfo).grpi1_name);

        PrintPaddedMessageString(4402, nPaddedLength);
        ConPrintf(StdOut, RUSTCONV_WSTR_4BA5D7E4F16E.as_ptr(), (*pGroupInfo).grpi1_comment);

        ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());

        PrintMessageString(4403);
        ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());

        PrintPadding(0x002Du16, 79);
        ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());

        i = 0;
        while i < dwRead {
            if !(*pNames.offset(i as isize)).is_null() {
                ConPrintf(
                    StdOut,
                    RUSTCONV_WSTR_4BA5D7E4F16E.as_ptr(),
                    *pNames.offset(i as isize),
                );
            }
            i = i.wrapping_add(1);
        }
    }

    if !pNames.is_null() {
        RtlFreeHeap(RtlGetProcessHeap!(), 0, pNames as PVOID);
    }

    if !pUsers.is_null() {
        NetApiBufferFree(pUsers as PVOID);
    }

    if !pGroupInfo.is_null() {
        NetApiBufferFree(pGroupInfo as PVOID);
    }

    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn cmdGroup(argc: INT, argv: *mut *mut WCHAR) -> INT {
    let mut i: INT = 0;
    let mut j: INT = 0;
    let mut result: INT = 0;
    let mut dwUserCount: ULONG = 0;
    let mut bAdd: BOOL = (FALSE) as i32;
    let mut bDelete: BOOL = (FALSE) as i32;
    let mut pGroupName: PWSTR = core::ptr::null_mut();
    let mut pComment: PWSTR = core::ptr::null_mut();
    let mut pUsers: *mut PWSTR = core::ptr::null_mut();
    let mut Info0: GROUP_INFO_0 = core::mem::zeroed();
    let mut Info1: GROUP_INFO_1 = core::mem::zeroed();
    let mut Info1002: GROUP_INFO_1002 = core::mem::zeroed();
    let mut Status: NET_API_STATUS = 0;

    if argc == 2 {
        Status = EnumerateGroups();
        ConPrintf(StdOut, RUSTCONV_WSTR_BE93C907DAAC.as_ptr(), Status);
        return 0;
    } else if argc == 3 {
        Status = DisplayGroup(*argv.offset(2 as isize));
        ConPrintf(StdOut, RUSTCONV_WSTR_BE93C907DAAC.as_ptr(), Status);
        return 0;
    }

    i = 2;
    if *(*argv.offset(i as isize)).offset(0 as isize) != 0x002Fu16 {
        pGroupName = *argv.offset(i as isize);
        i += 1;
    }

    j = i;
    while j < argc {
        if *(*argv.offset(j as isize)).offset(0 as isize) == 0x002Fu16 {
            break;
        }
        dwUserCount = dwUserCount.wrapping_add(1);
        j += 1;
    }

    if dwUserCount > 0 as ULONG {
        pUsers = RtlAllocateHeap(
            RtlGetProcessHeap!(),
            HEAP_ZERO_MEMORY as ULONG,
            (dwUserCount as SIZE_T).wrapping_mul(core::mem::size_of::<PGROUP_USERS_INFO_0>() as SIZE_T),
        ) as *mut PWSTR;
        if pUsers.is_null() {
            return 0;
        }
    }

    j = 0;
    while i < argc {
        if *(*argv.offset(i as isize)).offset(0 as isize) == 0x002Fu16 {
            break;
        }
        *pUsers.offset(j as isize) = *argv.offset(i as isize);
        j += 1;
        i += 1;
    }

    'done: {
        while i < argc {
            if _wcsicmp(*argv.offset(i as isize), RUSTCONV_WSTR_1A749FECCEE7.as_ptr()) == 0 {
                PrintMessageString(4381);
                ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
                PrintNetMessage(MSG_GROUP_SYNTAX);
                PrintNetMessage(MSG_GROUP_HELP);
                return 0;
            } else if _wcsicmp(*argv.offset(i as isize), RUSTCONV_WSTR_5D12D1C1410E.as_ptr()) == 0 {
                bAdd = (TRUE) as i32;
            } else if _wcsicmp(*argv.offset(i as isize), RUSTCONV_WSTR_7E7219CF4FF7.as_ptr()) == 0 {
                bDelete = (TRUE) as i32;
            } else if _wcsnicmp(*argv.offset(i as isize), RUSTCONV_WSTR_00C0DB009DDA.as_ptr(), 9 as usize) == 0 {
                pComment = (*argv.offset(i as isize)).offset(9 as isize);
            } else if _wcsicmp(*argv.offset(i as isize), RUSTCONV_WSTR_9BB9C2E30775.as_ptr()) == 0 {
                ConPuts(StdErr, RUSTCONV_WSTR_582BA4648587.as_ptr());
            } else {
                PrintErrorMessage(3506/*, argv[i]*/);
                result = 1;
                break 'done;
            }
            i += 1;
        }

        if pGroupName.is_null() {
            result = 1;
            break 'done;
        }

        if bAdd != 0 && bDelete != 0 {
            result = 1;
            break 'done;
        }

        if pUsers.is_null() {
            if bAdd == 0 && bDelete == 0 && !pComment.is_null() {
                /* Set group comment */
                Info1002.grpi1002_comment = pComment;
                Status = NetGroupSetInfo(
                    core::ptr::null_mut(),
                    pGroupName,
                    1002 as DWORD,
                    core::ptr::addr_of_mut!(Info1002) as LPBYTE,
                    core::ptr::null_mut(),
                );
                ConPrintf(StdOut, RUSTCONV_WSTR_BE93C907DAAC.as_ptr(), Status);
            } else if bAdd != 0 && bDelete == 0 {
                /* Add the group */
                if pComment.is_null() {
                    Info0.grpi0_name = pGroupName;
                } else {
                    Info1.grpi1_name = pGroupName;
                    Info1.grpi1_comment = pComment;
                }

                Status = NetGroupAdd(
                    core::ptr::null_mut(),
                    if pComment.is_null() {
                        0 as DWORD
                    } else {
                        1 as DWORD
                    },
                    if pComment.is_null() {
                        core::ptr::addr_of_mut!(Info0) as LPBYTE
                    } else {
                        core::ptr::addr_of_mut!(Info1) as LPBYTE
                    },
                    core::ptr::null_mut(),
                );
                ConPrintf(StdOut, RUSTCONV_WSTR_BE93C907DAAC.as_ptr(), Status);
            } else if bAdd == 0 && bDelete != 0 && pComment.is_null() {
                /* Delete the group */
                Status = NetGroupDel(
                    core::ptr::null_mut(),
                    pGroupName,
                );
                ConPrintf(StdOut, RUSTCONV_WSTR_BE93C907DAAC.as_ptr(), Status);
            } else {
                result = 1;
            }
        } else {
            if bAdd != 0 && bDelete == 0 && pComment.is_null() {
                /* Add group user */
                i = 0;
                while (i as ULONG) < dwUserCount {
                    Status = NetGroupAddUser(
                        core::ptr::null_mut(),
                        pGroupName,
                        *pUsers.offset(i as isize),
                    );
                    if Status != NERR_Success as NET_API_STATUS {
                        break;
                    }
                    i += 1;
                }
                ConPrintf(StdOut, RUSTCONV_WSTR_BE93C907DAAC.as_ptr(), Status);
            } else if bAdd == 0 && bDelete != 0 && pComment.is_null() {
                /* Delete group members */
                i = 0;
                while (i as ULONG) < dwUserCount {
                    Status = NetGroupDelUser(
                        core::ptr::null_mut(),
                        pGroupName,
                        *pUsers.offset(i as isize),
                    );
                    if Status != NERR_Success as NET_API_STATUS {
                        break;
                    }
                    i += 1;
                }
                ConPrintf(StdOut, RUSTCONV_WSTR_BE93C907DAAC.as_ptr(), Status);
            } else {
                result = 1;
            }
        }
    }

    // done:
    if !pUsers.is_null() {
        RtlFreeHeap(RtlGetProcessHeap!(), 0 as ULONG, pUsers as PVOID);
    }

    if result != 0 {
        PrintMessageString(4381);
        ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
        PrintNetMessage(MSG_GROUP_SYNTAX);
    }

    return result;
}
