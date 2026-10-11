use crate::net::main::PrintMessageString;
use crate::net::main::PrintNetMessage;
use crate::net::main::PrintPaddedMessageString;
use crate::net::main::PrintPadding;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_075DE2B906DB: [u16; 2] = [0x000Au16, 0x0000];
pub static RUSTCONV_WSTR_149E174B0F80: [u16; 16] = [
    0x0025u16, 0x002D, 0x0031, 0x0032, 0x0073, 0x0020, 0x0025, 0x002D, 0x0033, 0x0031,
    0x0073, 0x0020, 0x0025, 0x0073, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_1A749FECCEE7: [u16; 6] = [0x002Fu16, 0x0068, 0x0065, 0x006C, 0x0070, 0x0000];
pub static RUSTCONV_WSTR_4BA5D7E4F16E: [u16; 4] = [0x0025u16, 0x0073, 0x000A, 0x0000];
pub static RUSTCONV_WSTR_7E7219CF4FF7: [u16; 8] = [0x002Fu16, 0x0064, 0x0065, 0x006C, 0x0065, 0x0074, 0x0065, 0x0000];
pub static RUSTCONV_WSTR_BE93C907DAAC: [u16; 13] = [
    0x0053u16, 0x0074, 0x0061, 0x0074, 0x0075, 0x0073, 0x003A, 0x0020, 0x0025, 0x006C,
    0x0075, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_F3C6A8F2AC47: [u16; 4] = [0x0025u16, 0x006C, 0x0075, 0x0000];

use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn EnumerateShares() -> NET_API_STATUS {
    let mut pBuffer: PSHARE_INFO_2 = core::ptr::null_mut();
    let mut dwRead: DWORD = 0;
    let mut dwTotal: DWORD = 0;
    let mut ResumeHandle: DWORD = 0;
    let mut i: DWORD;
    let mut Status: NET_API_STATUS;

    ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
    PrintMessageString(4730);
    ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
    PrintPadding(0x002Du16, 79);
    ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());

    loop {
        Status = NetShareEnum(
            core::ptr::null_mut(),
            2,
            core::ptr::addr_of_mut!(pBuffer) as *mut LPBYTE,
            MAX_PREFERRED_LENGTH,
            core::ptr::addr_of_mut!(dwRead),
            core::ptr::addr_of_mut!(dwTotal),
            core::ptr::addr_of_mut!(ResumeHandle),
        );
        if (Status != 0 /* NERR_Success */) && (Status != ERROR_MORE_DATA) {
            return Status;
        }

        i = 0;
        while i < dwRead {
            ConPrintf(
                StdOut,
                RUSTCONV_WSTR_149E174B0F80.as_ptr(),
                (*pBuffer.offset(i as isize)).shi2_netname,
                (*pBuffer.offset(i as isize)).shi2_path,
                (*pBuffer.offset(i as isize)).shi2_remark,
            );
            i = i.wrapping_add(1);
        }

        NetApiBufferFree((pBuffer).cast());
        pBuffer = core::ptr::null_mut();

        if !(Status == ERROR_MORE_DATA) {
            break;
        }
    }

    return 0 /* NERR_Success */;
}
#[no_mangle]
pub unsafe extern "C" fn DisplayShare(mut pShareName: PWSTR) -> DWORD {
    let mut pBuffer: PSHARE_INFO_2 = core::ptr::null_mut();
    let nPaddedLength: DWORD = 22;
    let mut Status: DWORD;

    Status = NetShareGetInfo(core::ptr::null_mut(),
                             pShareName,
                             2,
                             core::ptr::addr_of_mut!(pBuffer) as *mut LPBYTE);
    if Status != 0 /* NERR_Success */ {
        return Status;
    }

    PrintPaddedMessageString(4731, nPaddedLength);
    ConPrintf(StdOut, RUSTCONV_WSTR_4BA5D7E4F16E.as_ptr(), (*pBuffer).shi2_netname);

    PrintPaddedMessageString(4339, nPaddedLength);
    ConPrintf(StdOut, RUSTCONV_WSTR_4BA5D7E4F16E.as_ptr(), (*pBuffer).shi2_path);

    PrintPaddedMessageString(4334, nPaddedLength);
    ConPrintf(StdOut, RUSTCONV_WSTR_4BA5D7E4F16E.as_ptr(), (*pBuffer).shi2_remark);

    PrintPaddedMessageString(4735, nPaddedLength);
    if (*pBuffer).shi2_max_uses == (-(1 as i32)) as DWORD {
        PrintMessageString(4736);
    } else {
        ConPrintf(StdOut, RUSTCONV_WSTR_F3C6A8F2AC47.as_ptr(), (*pBuffer).shi2_max_uses);
    }
    ConPrintf(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());

    PrintPaddedMessageString(4737, nPaddedLength);
    if (*pBuffer).shi2_current_uses > 0 {
        ConPrintf(StdOut, RUSTCONV_WSTR_F3C6A8F2AC47.as_ptr(), (*pBuffer).shi2_current_uses);
    }
    ConPrintf(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());

    NetApiBufferFree(pBuffer as PVOID);

    return 0 /* NERR_Success */;
}
#[no_mangle]
pub unsafe extern "C" fn cmdShare(mut argc: INT, mut argv: *mut *mut WCHAR) -> INT {
    let mut ShareInfo: SHARE_INFO_2 = core::mem::zeroed();
    let mut pszShareName: PWSTR = core::ptr::null_mut();
    let mut pszSharePath: PWSTR = core::ptr::null_mut();
    let mut ptr: PWSTR = core::ptr::null_mut();
    let mut bDelete: BOOL = (FALSE) as i32;
    let mut len: usize;
    let mut i: INT;
    let mut result: INT = 0;
    let mut Status: NET_API_STATUS;

    i = 2;
    if argc > 2
        && *(*argv.offset(i as isize)).offset(0 as isize) != 0x002Fu16
    {
        ptr = wcschr(*argv.offset(i as isize), 0x003Du16) as PWSTR;
        if !ptr.is_null() {
            if *ptr.offset(1 as isize) != UNICODE_NULL {
                len = wcslen(ptr.offset(i as isize)) as usize;
                pszSharePath = HeapAlloc(
                    GetProcessHeap(),
                    HEAP_ZERO_MEMORY as DWORD,
                    ((len + 1 as usize) * core::mem::size_of::<WCHAR>()) as SIZE_T,
                ) as PWSTR;
                if pszSharePath.is_null() {
                    // FIXME: Proper error code!
                    return 1;
                }

                wcscpy(pszSharePath, ptr.offset(1 as isize));
            }

            len = ((ptr as UINT_PTR).wrapping_sub(*argv.offset(i as isize) as UINT_PTR)
                / (core::mem::size_of::<WCHAR>() as UINT_PTR)) as usize;
            pszShareName = HeapAlloc(
                GetProcessHeap(),
                HEAP_ZERO_MEMORY as DWORD,
                ((len + 1 as usize) * core::mem::size_of::<WCHAR>()) as SIZE_T,
            ) as PWSTR;
            if pszShareName.is_null() {
                // FIXME: Proper error code!
                return 1;
            }

            wcsncpy(pszShareName, *argv.offset(i as isize), len);
        } else {
            len = wcslen(*argv.offset(i as isize)) as usize;
            pszShareName = HeapAlloc(
                GetProcessHeap(),
                HEAP_ZERO_MEMORY as DWORD,
                ((len + 1 as usize) * core::mem::size_of::<WCHAR>()) as SIZE_T,
            ) as PWSTR;
            if pszShareName.is_null() {
                // FIXME: Proper error code!
                return 1;
            }

            wcscpy(pszShareName, *argv.offset(i as isize));
        }

        i += 1;
    }

    while i < argc {
        if _wcsicmp(*argv.offset(i as isize), RUSTCONV_WSTR_1A749FECCEE7.as_ptr()) == 0 {
            /* Print full help text*/
            PrintMessageString(4381);
            ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
            PrintNetMessage(MSG_SHARE_SYNTAX);
            PrintNetMessage(MSG_SHARE_HELP);
            return 0;
        } else if _wcsicmp(*argv.offset(i as isize), RUSTCONV_WSTR_7E7219CF4FF7.as_ptr()) == 0 {
            bDelete = (TRUE) as i32;
        }
        i += 1;
    }

    printf(
        b"pszShareName: '%S'\n\0" as *const u8 as *const i8,
        pszShareName,
    );
    printf(
        b"pszSharePath: '%S'\n\0" as *const u8 as *const i8,
        pszSharePath,
    );

    if pszShareName.is_null() && pszSharePath.is_null() {
        Status = EnumerateShares();
        ConPrintf(StdOut, RUSTCONV_WSTR_BE93C907DAAC.as_ptr(), Status);
    } else if !pszShareName.is_null() && pszSharePath.is_null() {
        if (bDelete) as u32 == TRUE {
            Status = NetShareDel(
                core::ptr::null_mut(),
                pszShareName as LPWSTR,
                0 as DWORD,
            );
        } else {
            Status = DisplayShare(pszShareName);
        }

        ConPrintf(StdOut, RUSTCONV_WSTR_BE93C907DAAC.as_ptr(), Status);
    } else if !pszShareName.is_null() && !pszSharePath.is_null() {
        ZeroMemory!(
            core::ptr::addr_of_mut!(ShareInfo) as PVOID,
            core::mem::size_of::<SHARE_INFO_2>() as usize,
        );
        ShareInfo.shi2_netname = pszShareName as LPWSTR;
        ShareInfo.shi2_path = pszSharePath as LPWSTR;

        Status = NetShareAdd(
            core::ptr::null_mut(),
            2 as DWORD,
            core::ptr::addr_of_mut!(ShareInfo) as LPBYTE,
            core::ptr::null_mut(),
        );

        ConPrintf(StdOut, RUSTCONV_WSTR_BE93C907DAAC.as_ptr(), Status);
    }

    if !pszSharePath.is_null() {
        HeapFree(GetProcessHeap(), 0 as DWORD, pszSharePath as PVOID);
    }

    if !pszShareName.is_null() {
        HeapFree(GetProcessHeap(), 0 as DWORD, pszShareName as PVOID);
    }

    return result;
}
