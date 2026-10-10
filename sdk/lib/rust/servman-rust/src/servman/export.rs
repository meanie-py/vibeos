// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_B9B6DE145F6A: [u16; 4] = [0x0074u16, 0x0078, 0x0074, 0x0000];
pub static RUSTCONV_WSTR_E9AC8768D33B: [u16; 22] = [
    0x0045u16, 0x0078, 0x0070, 0x006F, 0x0072, 0x0074, 0x0020, 0x0074, 0x006F, 0x0020,
    0x0066, 0x0069, 0x006C, 0x0065, 0x0020, 0x0066, 0x0061, 0x0069, 0x006C, 0x0065,
    0x0064, 0x0000,
];
pub static RUSTCONV_WSTR_F6D9655F6532: [u16; 71] = [
    0x0054u16, 0x0065, 0x0078, 0x0074, 0x0020, 0x0028, 0x0054, 0x0061, 0x0062, 0x0020,
    0x0044, 0x0065, 0x006C, 0x0069, 0x006D, 0x0069, 0x0074, 0x0065, 0x0064, 0x0029,
    0x0028, 0x002A, 0x002E, 0x0074, 0x0078, 0x0074, 0x0029, 0x0000, 0x002A, 0x002E,
    0x0074, 0x0078, 0x0074, 0x0000, 0x0054, 0x0065, 0x0078, 0x0074, 0x0020, 0x0028,
    0x0043, 0x006F, 0x006D, 0x006D, 0x0061, 0x0020, 0x0044, 0x0065, 0x006C, 0x0069,
    0x006D, 0x0069, 0x0074, 0x0065, 0x0064, 0x0029, 0x0028, 0x002A, 0x002E, 0x0063,
    0x0073, 0x0076, 0x0029, 0x0000, 0x002A, 0x002E, 0x0063, 0x0073, 0x0076, 0x0000,
    0x0000,
];

use core::ffi::c_void;

unsafe extern "C" fn GetTextFromListView(
    Info: PMAIN_WND_INFO,
    Text: LPWSTR,
    row: INT,
    col: INT,
) -> DWORD {
    let mut item: LVITEM = core::mem::zeroed();
    let mut NumChars: DWORD;
    ZeroMemory!(
        core::ptr::addr_of_mut!(item) as *mut core::ffi::c_void,
        core::mem::size_of::<LVITEM>(),
    );
    item.mask = LVIF_TEXT as UINT;
    item.iSubItem = col as core::ffi::c_int;
    item.pszText = Text;
    item.cchTextMax = 500 as core::ffi::c_int;
    NumChars = SendMessageW(
        (*Info).hListView,
        LVM_GETITEMTEXTW as UINT,
        row as WPARAM,
        core::ptr::addr_of_mut!(item) as LPARAM,
    ) as INT as DWORD;
    return NumChars;
}
unsafe extern "C" fn SaveServicesToFile(
    Info: PMAIN_WND_INFO,
    pszFileName: LPCWSTR,
    nFormat: DWORD,
) -> BOOL {
    let mut hFile: HANDLE = core::ptr::null_mut();
    let mut bSuccess: BOOL = (FALSE) as i32;

    if nFormat == 0 || nFormat > 2 {
        return bSuccess;
    }

    hFile = CreateFileW(
        pszFileName,
        GENERIC_WRITE,
        0,
        core::ptr::null_mut(),
        CREATE_ALWAYS,
        FILE_ATTRIBUTE_NORMAL,
        core::ptr::null_mut(),
    );

    if hFile != INVALID_HANDLE_VALUE {
        let mut LVText: [WCHAR; 500] = [0; 500];
        let mut newl: [WCHAR; 2] = [0x000Du16, 0x000A];
        let mut seps: [WCHAR; 2] = [0x0009u16, 0x002C];
        let mut dwTextLength: DWORD = 0;
        let mut dwWritten: DWORD = 0;
        let mut NumListedServ: INT = 0;
        let mut i: INT = 0;
        let mut k: INT = 0;

        NumListedServ = ListView_GetItemCount!((*Info).hListView);

        i = 0;
        while i < NumListedServ {
            k = 0;
            while k < (LVMAX as i32) {
                dwTextLength = GetTextFromListView(
                    Info,
                    core::ptr::addr_of_mut!(LVText) as LPWSTR,
                    i,
                    k,
                );
                if dwTextLength != 0 {
                    WriteFile(
                        hFile,
                        core::ptr::addr_of_mut!(LVText) as *mut WCHAR as LPCVOID,
                        (core::mem::size_of::<WCHAR>() as DWORD).wrapping_mul(dwTextLength),
                        core::ptr::addr_of_mut!(dwWritten),
                        core::ptr::null_mut(),
                    );
                }

                if ((k) as u32) < LVMAX - 1 {
                    /* Do not add separator after the last table cell */
                    WriteFile(
                        hFile,
                        (core::ptr::addr_of_mut!(seps) as *mut WCHAR)
                            .offset((nFormat - 1) as isize)
                            as *mut WCHAR as LPCVOID,
                        core::mem::size_of::<WCHAR>() as DWORD,
                        core::ptr::addr_of_mut!(dwWritten),
                        core::ptr::null_mut(),
                    );
                }
                k += 1;
            }
            WriteFile(
                hFile,
                core::ptr::addr_of_mut!(newl) as *mut WCHAR as LPCVOID,
                core::mem::size_of::<[WCHAR; 2]>() as DWORD,
                core::ptr::addr_of_mut!(dwWritten),
                core::ptr::null_mut(),
            );
            i += 1;
        }

        CloseHandle(hFile);
        bSuccess = (TRUE) as i32;
    }

    return bSuccess;
}
#[no_mangle]
pub unsafe extern "C" fn ExportFile(Info: PMAIN_WND_INFO) {
    let mut ofn: OPENFILENAMEW = core::mem::zeroed();
    let mut szFileName: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];

    ZeroMemory!(&raw mut ofn as *mut core::ffi::c_void, core::mem::size_of::<OPENFILENAMEW>());
    szFileName[0 as usize] = UNICODE_NULL as WCHAR;

    ofn.lStructSize = core::mem::size_of::<OPENFILENAME>() as DWORD;
    ofn.hwndOwner = (*Info).hMainWnd;
    ofn.lpstrFilter = RUSTCONV_WSTR_F6D9655F6532.as_ptr();
    ofn.lpstrFile = &raw mut szFileName as *mut WCHAR as LPWSTR;
    ofn.nMaxFile = MAX_PATH as DWORD;
    ofn.lpstrDefExt = RUSTCONV_WSTR_B9B6DE145F6A.as_ptr();
    ofn.Flags = (OFN_EXPLORER | OFN_PATHMUSTEXIST | OFN_HIDEREADONLY | OFN_OVERWRITEPROMPT) as DWORD;

    if GetSaveFileName(&raw mut ofn) != 0 {
        if SaveServicesToFile(Info, &raw mut szFileName as *mut WCHAR as LPCWSTR, ofn.nFilterIndex) != 0 {
            return;
        }
    }

    if CommDlgExtendedError() != CDERR_GENERALCODES as DWORD {
        MessageBoxW(
            core::ptr::null_mut(),
            RUSTCONV_WSTR_E9AC8768D33B.as_ptr(),
            core::ptr::null(),
            0 as UINT,
        );
    }
}
