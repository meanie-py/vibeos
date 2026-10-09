use crate::console::console::ApplyConsoleInfo;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_8B7803CC0519: [u16; 46] = [
    0x0053u16, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x005C, 0x0043, 0x0075, 0x0072,
    0x0072, 0x0065, 0x006E, 0x0074, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F,
    0x006C, 0x0053, 0x0065, 0x0074, 0x005C, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072,
    0x006F, 0x006C, 0x005C, 0x004E, 0x006C, 0x0073, 0x005C, 0x0043, 0x006F, 0x0064,
    0x0065, 0x0050, 0x0061, 0x0067, 0x0065, 0x0000,
];
pub static RUSTCONV_WSTR_F3C6A8F2AC47: [u16; 4] = [0x0025u16, 0x006C, 0x0075, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const MAX_VALUE_NAME: u32 = 16383;

unsafe extern "C" fn List_GetCount(ListCtl: PLIST_CTL) -> INT {
    return SendMessageW(
        (*ListCtl).hWndList,
        CB_GETCOUNT as UINT,
        0 as WPARAM,
        0 as LPARAM,
    ) as INT;
}

unsafe extern "C" fn List_GetData(ListCtl: PLIST_CTL, Index: INT) -> ULONG_PTR {
    return SendMessageW(
        (*ListCtl).hWndList,
        CB_GETITEMDATA as UINT,
        Index as WPARAM,
        0 as LPARAM,
    ) as ULONG_PTR;
}

unsafe extern "C" fn AddCodePage(ListCtl: PLIST_CTL, CodePage: UINT) {
    let mut iItem: UINT = 0;
    let mut iDupItem: UINT = 0;
    let mut CPInfo: CPINFOEXW = core::mem::zeroed();
    if CodePage == CP_ACP as UINT
        || CodePage == CP_OEMCP as UINT
        || CodePage == CP_MACCP as UINT
        || CodePage == CP_THREAD_ACP as UINT
        || CodePage == CP_SYMBOL as UINT
        || IsValidCodePage(CodePage) == 0
    {
        return;
    }
    if GetCPInfoExW(CodePage, 0 as DWORD, core::ptr::addr_of_mut!(CPInfo)) == 0 {
        StringCchPrintfW!(
            core::ptr::addr_of_mut!(CPInfo.CodePageName) as *mut WCHAR,
            ARRAYSIZE!(CPInfo.CodePageName),
            RUSTCONV_WSTR_F3C6A8F2AC47.as_ptr(),
            CodePage,
        );
    }
    iDupItem = CB_ERR as UINT;
    iItem = BisectListSortedByValue(ListCtl, CodePage as ULONG_PTR, core::ptr::addr_of_mut!(iDupItem), TRUE as BOOL);
    if iItem == CB_ERR as UINT {
        iItem = 0 as UINT;
    }
    if iDupItem != CB_ERR as UINT {
        return;
    }
    iItem = SendMessageW(
        (*ListCtl).hWndList,
        CB_INSERTSTRING as UINT,
        iItem as WPARAM,
        core::ptr::addr_of_mut!(CPInfo.CodePageName) as *mut WCHAR as LPARAM,
    ) as UINT;
    if iItem != CB_ERR as UINT && iItem != CB_ERRSPACE as UINT {
        iItem = SendMessageW(
            (*ListCtl).hWndList,
            CB_SETITEMDATA as UINT,
            iItem as WPARAM,
            CodePage as LPARAM,
        ) as UINT;
    }
}

unsafe extern "C" fn BuildCodePageList(hDlg: HWND, CurrentCodePage: UINT) {
    let mut ListCtl: LIST_CTL = LIST_CTL {
        hWndList: ::core::ptr::null_mut::<HWND__>(),
        GetCount: None,
        GetData: None,
    };
    let mut lResult: LRESULT = 0;
    let mut hKey: HKEY = ::core::ptr::null_mut::<HKEY__>();
    let mut dwIndex: DWORD = 0;
    let mut dwType: DWORD = 0;
    let mut cchValueName: DWORD = 0;
    let mut CodePage: UINT = 0;
    let mut szValueName: [WCHAR; 6] = [0; 6];

    /* Open the Nls\CodePage key */
    if RegOpenKeyExW(
        HKEY_LOCAL_MACHINE as HKEY,
        RUSTCONV_WSTR_8B7803CC0519.as_ptr() as LPCWSTR,
        0 as DWORD,
        KEY_QUERY_VALUE as REGSAM,
        core::ptr::addr_of_mut!(hKey),
    ) != ERROR_SUCCESS as LSTATUS
    {
        return;
    }

    ListCtl.hWndList = GetDlgItem(hDlg, (IDL_CODEPAGE as i32));
    ListCtl.GetCount =
        Some(List_GetCount as unsafe extern "C" fn(PLIST_CTL) -> INT) as PLIST_GETCOUNT;
    ListCtl.GetData =
        Some(List_GetData as unsafe extern "C" fn(PLIST_CTL, INT) -> ULONG_PTR) as PLIST_GETDATA;

    /* Enumerate all the available code pages on the system */
    dwIndex = 0 as DWORD;
    cchValueName = ARRAYSIZE!(szValueName) as DWORD;
    loop {
        lResult = RegEnumValueW(
            hKey,
            dwIndex,
            core::ptr::addr_of_mut!(szValueName) as *mut WCHAR,
            core::ptr::addr_of_mut!(cchValueName),
            ::core::ptr::null_mut::<DWORD>(),
            core::ptr::addr_of_mut!(dwType),
            ::core::ptr::null_mut::<BYTE>(),
            ::core::ptr::null_mut::<DWORD>(),
        ) as LRESULT;
        if !(lResult != ERROR_NO_MORE_ITEMS as LRESULT) {
            break;
        }

        'body: {
            /* Ignore if we failed for another reason, e.g. because
             * the value name is too long (and thus, invalid). */
            if lResult != ERROR_SUCCESS as LRESULT {
                break 'body;
            }

            /* Validate the value name (exclude the unnamed value) */
            if cchValueName == 0
                || (*(core::ptr::addr_of_mut!(szValueName) as *mut WCHAR)
                    == UNICODE_NULL as WCHAR)
            {
                break 'body;
            }
            /* Too large value names have already been handled with ERROR_MORE_DATA */

            /* Validate the value type */
            if dwType != REG_SZ as DWORD {
                break 'body;
            }

            /*
             * Add the code page into the list.
             * If _wtol fails and returns 0, the code page is considered invalid
             * (and indeed this value corresponds to the CP_ACP alias too).
             */
            CodePage = _wtol(core::ptr::addr_of_mut!(szValueName) as *mut WCHAR)
                as UINT;
            if CodePage == 0 as UINT {
                break 'body;
            }
            AddCodePage(core::ptr::addr_of_mut!(ListCtl), CodePage);
        }

        dwIndex = dwIndex.wrapping_add(1);
        cchValueName = ARRAYSIZE!(szValueName) as DWORD;
    }

    RegCloseKey(hKey);

    /* Add the special UTF-7 (CP_UTF7 65000) and UTF-8 (CP_UTF8 65001) code pages */
    AddCodePage(core::ptr::addr_of_mut!(ListCtl), CP_UTF7 as UINT);
    AddCodePage(core::ptr::addr_of_mut!(ListCtl), CP_UTF8 as UINT);

    /* Find and select the current code page in the sorted list */
    if BisectListSortedByValue(
        core::ptr::addr_of_mut!(ListCtl),
        CurrentCodePage as ULONG_PTR,
        core::ptr::addr_of_mut!(CodePage),
        FALSE as BOOL,
    ) as UINT == CB_ERR as UINT
        || CodePage == CB_ERR as UINT
    {
        /* Not found, select the first element */
        CodePage = 0 as UINT;
    }

    SendMessageW(
        ListCtl.hWndList,
        CB_SETCURSEL as UINT,
        CodePage as WPARAM,
        0 as LPARAM,
    );
}

unsafe extern "C" fn UpdateDialogElements(hDlg: HWND, pConInfo: PCONSOLE_STATE_INFO) {
    /* Update the cursor size */
    if (*pConInfo).CursorSize <= 25 {
        /* Small cursor */
        CheckRadioButton(hDlg, (IDC_RADIO_SMALL_CURSOR as i32), (IDC_RADIO_LARGE_CURSOR as i32), (IDC_RADIO_SMALL_CURSOR as i32));
    } else if (*pConInfo).CursorSize <= 50 {
        /* Medium cursor */
        CheckRadioButton(hDlg, (IDC_RADIO_SMALL_CURSOR as i32), (IDC_RADIO_LARGE_CURSOR as i32), (IDC_RADIO_MEDIUM_CURSOR as i32));
    } else {
        /* Large cursor */
        CheckRadioButton(hDlg, (IDC_RADIO_SMALL_CURSOR as i32), (IDC_RADIO_LARGE_CURSOR as i32), (IDC_RADIO_LARGE_CURSOR as i32));
    }

    /* Update the number of history buffers */
    SendDlgItemMessageW(hDlg, (IDC_UPDOWN_NUM_BUFFER as i32), UDM_SETRANGE, 0, MAKELONG!(999, 1) as LPARAM);
    SetDlgItemInt(hDlg, (IDC_EDIT_NUM_BUFFER as i32), (*pConInfo).NumberOfHistoryBuffers as UINT, (FALSE) as BOOL);

    /* Update the history buffer size */
    SendDlgItemMessageW(hDlg, (IDC_UPDOWN_BUFFER_SIZE as i32), UDM_SETRANGE, 0, MAKELONG!(999, 1) as LPARAM);
    SetDlgItemInt(hDlg, (IDC_EDIT_BUFFER_SIZE as i32), (*pConInfo).HistoryBufferSize as UINT, (FALSE) as BOOL);

    /* Update discard duplicates */
    CheckDlgButton(hDlg, (IDC_CHECK_DISCARD_DUPLICATES as i32),
                   (if (*pConInfo).HistoryNoDup != 0 { BST_CHECKED } else { BST_UNCHECKED }) as UINT);

    /* Update full/window screen state */
    if (*pConInfo).FullScreen != 0 {
        CheckRadioButton(hDlg, (IDC_RADIO_DISPLAY_WINDOW as i32), (IDC_RADIO_DISPLAY_FULL as i32), (IDC_RADIO_DISPLAY_FULL as i32));
    } else {
        CheckRadioButton(hDlg, (IDC_RADIO_DISPLAY_WINDOW as i32), (IDC_RADIO_DISPLAY_FULL as i32), (IDC_RADIO_DISPLAY_WINDOW as i32));
    }

    /* Update "Quick-edit" state */
    CheckDlgButton(hDlg, (IDC_CHECK_QUICK_EDIT as i32),
                   (if (*pConInfo).QuickEdit != 0 { BST_CHECKED } else { BST_UNCHECKED }) as UINT);

    /* Update "Insert mode" state */
    CheckDlgButton(hDlg, (IDC_CHECK_INSERT_MODE as i32),
                   (if (*pConInfo).InsertMode != 0 { BST_CHECKED } else { BST_UNCHECKED }) as UINT);
}
#[no_mangle]
pub unsafe extern "system" fn OptionsProc(hDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    'sw: {
        match uMsg {
            WM_INITDIALOG => {
                BuildCodePageList(hDlg, (*ConInfo).CodePage as UINT);
                UpdateDialogElements(hDlg, ConInfo);
                return (TRUE) as INT_PTR;
            }

            WM_NOTIFY => {
                let lppsn: LPPSHNOTIFY = lParam as LPPSHNOTIFY;

                if (*lppsn).hdr.code == (UDN_DELTAPOS) as u32 {
                    let lpnmud: LPNMUPDOWN = lParam as LPNMUPDOWN;

                    if (*lppsn).hdr.idFrom == (IDC_UPDOWN_BUFFER_SIZE as _) {
                        (*lpnmud).iPos = min(max((*lpnmud).iPos + (*lpnmud).iDelta, 1), 999);
                        (*ConInfo).HistoryBufferSize = (*lpnmud).iPos as ULONG;
                        PropSheet_Changed!(GetParent(hDlg), hDlg);
                    } else if (*lppsn).hdr.idFrom == (IDC_UPDOWN_NUM_BUFFER as _) {
                        (*lpnmud).iPos = min(max((*lpnmud).iPos + (*lpnmud).iDelta, 1), 999);
                        (*ConInfo).NumberOfHistoryBuffers = (*lpnmud).iPos as ULONG;
                        PropSheet_Changed!(GetParent(hDlg), hDlg);
                    }
                } else if (*lppsn).hdr.code == (PSN_APPLY) as u32 {
                    ApplyConsoleInfo(hDlg);
                    return (TRUE) as INT_PTR;
                }
            }

            WM_COMMAND => {
                if (HIWORD!(wParam)) as u32 == BN_CLICKED {
                    match (LOWORD!(wParam)) as u32 {
                        IDC_RADIO_SMALL_CURSOR => {
                            (*ConInfo).CursorSize = 25;
                            PropSheet_Changed!(GetParent(hDlg), hDlg);
                        }
                        IDC_RADIO_MEDIUM_CURSOR => {
                            (*ConInfo).CursorSize = 50;
                            PropSheet_Changed!(GetParent(hDlg), hDlg);
                        }
                        IDC_RADIO_LARGE_CURSOR => {
                            (*ConInfo).CursorSize = 100;
                            PropSheet_Changed!(GetParent(hDlg), hDlg);
                        }
                        IDC_RADIO_DISPLAY_WINDOW => {
                            (*ConInfo).FullScreen = (FALSE) as i32;
                            PropSheet_Changed!(GetParent(hDlg), hDlg);
                        }
                        IDC_RADIO_DISPLAY_FULL => {
                            (*ConInfo).FullScreen = (TRUE) as i32;
                            PropSheet_Changed!(GetParent(hDlg), hDlg);
                        }
                        IDC_CHECK_QUICK_EDIT => {
                            (*ConInfo).QuickEdit = (IsDlgButtonChecked(hDlg, (IDC_CHECK_QUICK_EDIT as i32)) == BST_CHECKED) as BOOL;
                            PropSheet_Changed!(GetParent(hDlg), hDlg);
                        }
                        IDC_CHECK_INSERT_MODE => {
                            (*ConInfo).InsertMode = (IsDlgButtonChecked(hDlg, (IDC_CHECK_INSERT_MODE as i32)) == BST_CHECKED) as BOOL;
                            PropSheet_Changed!(GetParent(hDlg), hDlg);
                        }
                        IDC_CHECK_DISCARD_DUPLICATES => {
                            (*ConInfo).HistoryNoDup = (IsDlgButtonChecked(hDlg, (IDC_CHECK_DISCARD_DUPLICATES as i32)) == BST_CHECKED) as BOOL;
                            PropSheet_Changed!(GetParent(hDlg), hDlg);
                        }
                        _ => {}
                    }
                } else if (HIWORD!(wParam)) as u32 == EN_KILLFOCUS {
                    match (LOWORD!(wParam)) as u32 {
                        IDC_EDIT_BUFFER_SIZE => {
                            let mut sizeBuff: DWORD;

                            sizeBuff = GetDlgItemInt(hDlg, (IDC_EDIT_BUFFER_SIZE as i32), core::ptr::null_mut(), (FALSE) as BOOL);
                            sizeBuff = min(max(sizeBuff, 1), 999);

                            (*ConInfo).HistoryBufferSize = sizeBuff as ULONG;
                            PropSheet_Changed!(GetParent(hDlg), hDlg);
                        }
                        IDC_EDIT_NUM_BUFFER => {
                            let mut numBuff: DWORD;

                            numBuff = GetDlgItemInt(hDlg, (IDC_EDIT_NUM_BUFFER as i32), core::ptr::null_mut(), (FALSE) as BOOL);
                            numBuff = min(max(numBuff, 1), 999);

                            (*ConInfo).NumberOfHistoryBuffers = numBuff as ULONG;
                            PropSheet_Changed!(GetParent(hDlg), hDlg);
                        }
                        _ => {}
                    }
                }
                // (HIWORD(wParam) == CBN_KILLFOCUS)
                else if ((HIWORD!(wParam)) as u32 == CBN_SELCHANGE || (HIWORD!(wParam)) as u32 == CBN_SELENDOK)
                    && (LOWORD!(wParam) == (IDL_CODEPAGE as u16))
                {
                    let hWndList: HWND = GetDlgItem(hDlg, (IDL_CODEPAGE as i32));
                    let mut iItem: INT;
                    let mut CodePage: UINT;

                    iItem = SendMessageW(hWndList, CB_GETCURSEL, 0, 0) as INT;
                    if iItem == CB_ERR {
                        break 'sw;
                    }

                    CodePage = SendMessageW(hWndList, CB_GETITEMDATA, iItem as WPARAM, 0) as UINT;
                    if CodePage == (CB_ERR) as u32 {
                        break 'sw;
                    }

                    /* If the user has selected a different code page... */
                    if ((HIWORD!(wParam)) as u32 == CBN_SELENDOK) && (CodePage as ULONG != (*ConInfo).CodePage) {
                        /* ... update the code page and change the property sheet state */
                        (*ConInfo).CodePage = CodePage as ULONG;
                        ResetFontPreview!(core::ptr::addr_of_mut!(FontPreview));
                        PropSheet_Changed!(GetParent(hDlg), hDlg);
                    }
                }

                break 'sw;
            }

            _ => {}
        }
    }

    return (FALSE) as INT_PTR;
}
