use crate::usrmgr::misc::CheckAccountName;
use crate::usrmgr::usrmgr::GetDlgItemTextAlloc;
use crate::usrmgr::userprops::UserProperties;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_16ED7B7F1D78: [u16; 15] = [
    0x0041u16, 0x0075, 0x0074, 0x006F, 0x0041, 0x0064, 0x006D, 0x0069, 0x006E, 0x004C,
    0x006F, 0x0067, 0x006F, 0x006E, 0x0000,
];
pub static RUSTCONV_WSTR_4F8320D91E97: [u16; 2] = [0x0030u16, 0x0000];
pub static RUSTCONV_WSTR_60864AAE2645: [u16; 2] = [0x0031u16, 0x0000];
pub static RUSTCONV_WSTR_AA2E5A31D547: [u16; 54] = [
    0x0053u16, 0x004F, 0x0046, 0x0054, 0x0057, 0x0041, 0x0052, 0x0045, 0x005C, 0x004D,
    0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057,
    0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x0020, 0x004E, 0x0054, 0x005C,
    0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0056, 0x0065, 0x0072,
    0x0073, 0x0069, 0x006F, 0x006E, 0x005C, 0x0057, 0x0069, 0x006E, 0x006C, 0x006F,
    0x0067, 0x006F, 0x006E, 0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct USER_DATA {
    pub hPopupMenu: HMENU,
    pub iCurrentItem: INT,
}

pub type PUSER_DATA = *mut USER_DATA;

unsafe extern "C" fn CheckPasswords(hwndDlg: HWND, nIdDlgItem1: INT, nIdDlgItem2: INT) -> BOOL {
    let mut szPassword1: [TCHAR; (PWLEN) as _] = [0; (PWLEN) as _];
    let mut szPassword2: [TCHAR; (PWLEN) as _] = [0; (PWLEN) as _];
    let mut uLen1: UINT = 0;
    let mut uLen2: UINT = 0;

    uLen1 = GetDlgItemText(hwndDlg, nIdDlgItem1, szPassword1.as_mut_ptr(), (PWLEN as i32));
    uLen2 = GetDlgItemText(hwndDlg, nIdDlgItem2, szPassword2.as_mut_ptr(), (PWLEN as i32));

    /* Check the passwords */
    if uLen1 != uLen2 || _tcscmp(szPassword1.as_ptr(), szPassword2.as_ptr()) != 0
    {
        static MSG_TEXT: [u16; 44] = [
            'T' as u16, 'h' as u16, 'e' as u16, ' ' as u16,
            'p' as u16, 'a' as u16, 's' as u16, 's' as u16, 'w' as u16, 'o' as u16, 'r' as u16, 'd' as u16, 's' as u16, ' ' as u16,
            'y' as u16, 'o' as u16, 'u' as u16, ' ' as u16,
            'e' as u16, 'n' as u16, 't' as u16, 'e' as u16, 'r' as u16, 'e' as u16, 'd' as u16, ' ' as u16,
            'a' as u16, 'r' as u16, 'e' as u16, ' ' as u16,
            'n' as u16, 'o' as u16, 't' as u16, ' ' as u16,
            't' as u16, 'h' as u16, 'e' as u16, ' ' as u16,
            's' as u16, 'a' as u16, 'm' as u16, 'e' as u16, '!' as u16,
            0,
        ];
        static CAP_TEXT: [u16; 6] = ['E' as u16, 'R' as u16, 'R' as u16, 'O' as u16, 'R' as u16, 0];

        MessageBox(
            hwndDlg,
            MSG_TEXT.as_ptr(),
            CAP_TEXT.as_ptr(),
            MB_OK | MB_ICONERROR,
        );
        return (FALSE) as BOOL;
    }

    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn ChangePasswordDlgProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut userInfo: PUSER_INFO_1003;

    UNREFERENCED_PARAMETER!(wParam);

    userInfo = GetWindowLongPtr(hwndDlg, DWLP_USER) as PUSER_INFO_1003;

    match (uMsg as ::core::ffi::c_int) as u32 {
        WM_INITDIALOG => {
            userInfo = lParam as PUSER_INFO_1003;
            SetWindowLongPtr(hwndDlg, DWLP_USER, lParam);
        }

        WM_COMMAND => match (LOWORD!(wParam) as ::core::ffi::c_int) as u32 {
            IDOK => {
                if CheckPasswords(hwndDlg, (IDC_EDIT_PASSWORD1) as INT, (IDC_EDIT_PASSWORD2) as INT) != 0 {
                    /* Store the password */
                    (*userInfo).usri1003_password =
                        GetDlgItemTextAlloc(hwndDlg, (IDC_EDIT_PASSWORD1) as INT) as LPWSTR;

                    EndDialog(hwndDlg, (IDOK) as INT_PTR);
                }
            }

            IDCANCEL => {
                EndDialog(hwndDlg, (IDCANCEL) as INT_PTR);
            }

            _ => {}
        },

        _ => return FALSE as INT_PTR,
    }

    return TRUE as INT_PTR;
}

unsafe extern "C" fn UserChangePassword(hwndDlg: HWND) {
    let mut szUserName: [TCHAR; (UNLEN + 1) as _] = [0; (UNLEN + 1) as _];
    let mut user: USER_INFO_1003 = core::mem::zeroed();
    let mut nItem: INT = 0;
    let mut hwndLV: HWND = core::ptr::null_mut();
    let mut status: DWORD = 0;

    ZeroMemory!(core::ptr::addr_of_mut!(user) as *mut core::ffi::c_void, core::mem::size_of::<USER_INFO_1003>() as usize);

    hwndLV = GetDlgItem(hwndDlg, IDC_USERS_LIST as INT);
    nItem = ListView_GetNextItem!(hwndLV, -(1 as INT), LVNI_SELECTED as UINT) as INT;
    if nItem == -(1 as INT) {
        return;
    }

    /* Get the new user name */
    ListView_GetItemText!(
        hwndLV,
        nItem,
        0 as INT,
        core::ptr::addr_of_mut!(szUserName) as *mut TCHAR,
        (UNLEN + 1) as INT,
    );

    if DialogBoxParam(
        hApplet,
        MAKEINTRESOURCE!(IDD_CHANGE_PASSWORD),
        hwndDlg,
        Some(ChangePasswordDlgProc),
        core::ptr::addr_of_mut!(user) as LPARAM,
    ) == IDOK as INT_PTR
    {
        status = NetUserSetInfo(
            core::ptr::null(),
            core::ptr::addr_of_mut!(szUserName) as *mut TCHAR as LPCWSTR,
            1003 as DWORD,
            core::ptr::addr_of_mut!(user) as LPBYTE,
            core::ptr::null_mut(),
        );
        if status != NERR_Success as DWORD {
            let mut szText: [TCHAR; 256] = [0; 256];
            let fmtError: [u16; 10] = ['E' as u16, 'r' as u16, 'r' as u16, 'o' as u16, 'r' as u16, ':' as u16, ' ' as u16, '%' as u16, 'u' as u16, 0];
            wsprintf(
                core::ptr::addr_of_mut!(szText) as *mut TCHAR,
                fmtError.as_ptr(),
                status,
            );
            let capNetUserSetInfo: [u16; 15] = ['N' as u16, 'e' as u16, 't' as u16, 'U' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'S' as u16, 'e' as u16, 't' as u16, 'I' as u16, 'n' as u16, 'f' as u16, 'o' as u16, 0];
            MessageBox(
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(szText) as *mut TCHAR as LPCWSTR,
                capNetUserSetInfo.as_ptr(),
                (MB_ICONERROR | MB_OK) as UINT,
            );
        }
    }

    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        user.usri1003_password as PVOID,
    );
}

unsafe extern "C" fn UpdateNewUserOptions(hwndDlg: HWND, userInfo: PUSER_INFO_3, bInit: BOOL) {
    EnableWindow(
        GetDlgItem(hwndDlg, (IDC_USER_NEW_CANNOT_CHANGE as i32)),
        ((*userInfo).usri3_password_expired == 0) as i32,
    );
    EnableWindow(
        GetDlgItem(hwndDlg, (IDC_USER_NEW_NEVER_EXPIRES as i32)),
        ((*userInfo).usri3_password_expired == 0) as i32,
    );

    EnableWindow(
        GetDlgItem(hwndDlg, (IDC_USER_NEW_FORCE_CHANGE as i32)),
        (((*userInfo).usri3_flags & (UF_PASSWD_CANT_CHANGE | UF_DONT_EXPIRE_PASSWD))
            == 0) as i32,
    );

    if bInit != 0 {
        CheckDlgButton(
            hwndDlg,
            (IDC_USER_NEW_FORCE_CHANGE as i32),
            (if (*userInfo).usri3_password_expired != 0 {
                BST_CHECKED
            } else {
                BST_UNCHECKED
            }) as UINT,
        );

        CheckDlgButton(
            hwndDlg,
            (IDC_USER_NEW_CANNOT_CHANGE as i32),
            (if ((*userInfo).usri3_flags & UF_PASSWD_CANT_CHANGE) != 0 {
                BST_CHECKED
            } else {
                BST_UNCHECKED
            }) as UINT,
        );

        CheckDlgButton(
            hwndDlg,
            (IDC_USER_NEW_NEVER_EXPIRES as i32),
            (if ((*userInfo).usri3_flags & UF_DONT_EXPIRE_PASSWD) != 0 {
                BST_CHECKED
            } else {
                BST_UNCHECKED
            }) as UINT,
        );

        CheckDlgButton(
            hwndDlg,
            (IDC_USER_NEW_DISABLED as i32),
            (if ((*userInfo).usri3_flags & UF_ACCOUNTDISABLE) != 0 {
                BST_CHECKED
            } else {
                BST_UNCHECKED
            }) as UINT,
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn NewUserDlgProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut userInfo: PUSER_INFO_3 = core::ptr::null_mut();
    let mut nLength: INT = 0;

    UNREFERENCED_PARAMETER!(wParam);

    userInfo = GetWindowLongPtr(hwndDlg, DWLP_USER) as PUSER_INFO_3;

    match uMsg {
        WM_INITDIALOG => {
            userInfo = lParam as PUSER_INFO_3;
            SetWindowLongPtr(hwndDlg, DWLP_USER, lParam);
            SendDlgItemMessage(hwndDlg, (IDC_USER_NEW_NAME as i32), EM_SETLIMITTEXT, 20, 0);
            UpdateNewUserOptions(hwndDlg, userInfo, (TRUE) as BOOL);
        }

        WM_COMMAND => {
            match (LOWORD!(wParam) as INT) as u32 {
                IDC_USER_NEW_NAME => {
                    if ((HIWORD!(wParam) as INT)) as u32 == EN_CHANGE {
                        nLength = SendDlgItemMessage(hwndDlg, (IDC_USER_NEW_NAME as i32), WM_GETTEXTLENGTH, 0, 0) as INT;
                        EnableWindow(GetDlgItem(hwndDlg, (IDOK) as i32), (nLength > 0) as INT);
                    }
                }

                IDC_USER_NEW_FORCE_CHANGE => {
                    (*userInfo).usri3_password_expired = ((*userInfo).usri3_password_expired == 0) as INT as DWORD;
                    UpdateNewUserOptions(hwndDlg, userInfo, (FALSE) as BOOL);
                }

                IDC_USER_NEW_CANNOT_CHANGE => {
                    (*userInfo).usri3_flags ^= UF_PASSWD_CANT_CHANGE as DWORD;
                    UpdateNewUserOptions(hwndDlg, userInfo, (FALSE) as BOOL);
                }

                IDC_USER_NEW_NEVER_EXPIRES => {
                    (*userInfo).usri3_flags ^= UF_DONT_EXPIRE_PASSWD as DWORD;
                    UpdateNewUserOptions(hwndDlg, userInfo, (FALSE) as BOOL);
                }

                IDC_USER_NEW_DISABLED => {
                    (*userInfo).usri3_flags ^= UF_ACCOUNTDISABLE as DWORD;
                }

                IDOK => 'block: {
                    if CheckAccountName(hwndDlg, (IDC_USER_NEW_NAME) as INT, core::ptr::null_mut()) == 0 {
                        SetFocus(GetDlgItem(hwndDlg, (IDC_USER_NEW_NAME as i32)));
                        SendDlgItemMessage(hwndDlg, (IDC_USER_NEW_NAME as i32), EM_SETSEL, 0, (-1) as LPARAM);
                        break 'block;
                    }

                    if CheckPasswords(hwndDlg, (IDC_USER_NEW_PASSWORD1) as INT, (IDC_USER_NEW_PASSWORD2) as INT) == 0 {
                        SetDlgItemText(hwndDlg, (IDC_USER_NEW_PASSWORD1 as i32), (&[0x0000u16]).as_ptr() /* (&[0x0000u16]).as_ptr() /* (&[0x0000u16]).as_ptr() /* (&[0x0000u16]).as_ptr() /* ("") */ */ */ */);
                        SetDlgItemText(hwndDlg, (IDC_USER_NEW_PASSWORD2 as i32), (&[0x0000u16]).as_ptr() /* (&[0x0000u16]).as_ptr() /* (&[0x0000u16]).as_ptr() /* (&[0x0000u16]).as_ptr() /* ("") */ */ */ */);
                        break 'block;
                    }

                    /* Store the user name */
                    (*userInfo).usri3_name = GetDlgItemTextAlloc(hwndDlg, (IDC_USER_NEW_NAME) as INT) as LPWSTR;

                    /* Store the full user name */
                    (*userInfo).usri3_full_name = GetDlgItemTextAlloc(hwndDlg, (IDC_USER_NEW_FULL_NAME) as INT) as LPWSTR;

                    /* Store the description */
                    (*userInfo).usri3_comment = GetDlgItemTextAlloc(hwndDlg, (IDC_USER_NEW_DESCRIPTION) as INT) as LPWSTR;

                    /* Store the password */
                    (*userInfo).usri3_password = GetDlgItemTextAlloc(hwndDlg, (IDC_USER_NEW_PASSWORD1) as INT) as LPWSTR;

                    EndDialog(hwndDlg, (IDOK) as INT_PTR);
                }

                IDCANCEL => {
                    EndDialog(hwndDlg, (IDCANCEL) as INT_PTR);
                }

                _ => {}
            }
        }

        _ => {
            return FALSE as INT_PTR;
        }
    }

    return TRUE as INT_PTR;
}

unsafe extern "C" fn UserNew(hwndDlg: HWND) {
    let mut user: USER_INFO_3 = core::mem::zeroed();
    let mut status: DWORD = 0;
    let mut lvi: LV_ITEM = core::mem::zeroed();
    let mut iItem: INT = 0;
    let mut hwndLV: HWND = core::ptr::null_mut();

    ZeroMemory!(&raw mut user as *mut core::ffi::c_void, core::mem::size_of::<USER_INFO_3>() as usize);

    user.usri3_priv = USER_PRIV_USER as DWORD;
    user.usri3_flags = UF_SCRIPT as DWORD;
    user.usri3_acct_expires = TIMEQ_FOREVER as DWORD;
    user.usri3_max_storage = USER_MAXSTORAGE_UNLIMITED as DWORD;
    user.usri3_primary_group_id = DOMAIN_GROUP_RID_USERS as DWORD;

    user.usri3_password_expired = TRUE as DWORD;

    if DialogBoxParam(
        hApplet,
        MAKEINTRESOURCE!(IDD_USER_NEW as WORD) as LPCWSTR,
        hwndDlg,
        Some(NewUserDlgProc as unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> INT_PTR),
        &raw mut user as LPARAM,
    ) == IDOK as INT_PTR
    {
        status = NetUserAdd(
            core::ptr::null_mut(),
            3 as DWORD,
            &raw mut user as LPBYTE,
            core::ptr::null_mut(),
        );
        if status != NERR_Success as DWORD {
            let mut szText: [TCHAR; 256 as usize] = [0 as TCHAR; 256 as usize];
            let fmtError: [u16; 10] = ['E' as u16, 'r' as u16, 'r' as u16, 'o' as u16, 'r' as u16, ':' as u16, ' ' as u16, '%' as u16, 'u' as u16, 0];
            let captionNetUserAdd: [u16; 11] = ['N' as u16, 'e' as u16, 't' as u16, 'U' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'A' as u16, 'd' as u16, 'd' as u16, 0];
            wsprintf(szText.as_mut_ptr(), fmtError.as_ptr(), status);
            MessageBox(
                core::ptr::null_mut(),
                szText.as_ptr(),
                captionNetUserAdd.as_ptr(),
                (MB_ICONERROR | MB_OK) as UINT,
            );
            return;
        }

        hwndLV = GetDlgItem(hwndDlg, IDC_USERS_LIST as INT);

        ZeroMemory!(&raw mut lvi as *mut core::ffi::c_void, core::mem::size_of::<LV_ITEM>());
        lvi.mask = (LVIF_TEXT | LVIF_STATE | LVIF_IMAGE) as UINT;
        lvi.pszText = user.usri3_name;
        lvi.state = 0 as UINT;
        lvi.iImage = if (user.usri3_flags & (UF_ACCOUNTDISABLE as DWORD)) != 0 {
            1 as INT
        } else {
            0 as INT
        };
        iItem = ListView_InsertItem!(hwndLV, &raw mut lvi) as INT;

        ListView_SetItemText!(hwndLV, iItem, 1 as INT,
                             user.usri3_full_name);

        ListView_SetItemText!(hwndLV, iItem, 2 as INT,
                             user.usri3_comment);
    }

    HeapFree(GetProcessHeap(), 0 as DWORD, user.usri3_name as PVOID);
    HeapFree(GetProcessHeap(), 0 as DWORD, user.usri3_full_name as PVOID);
    HeapFree(GetProcessHeap(), 0 as DWORD, user.usri3_comment as PVOID);
    HeapFree(GetProcessHeap(), 0 as DWORD, user.usri3_password as PVOID);
}

unsafe extern "C" fn UserRename(hwndDlg: HWND) {
    let mut hwndLV: HWND;
    let mut nItem: INT;
    hwndLV = GetDlgItem(hwndDlg, (IDC_USERS_LIST as i32));
    if hwndLV.is_null() {
        return;
    }
    nItem = ListView_GetNextItem!(hwndLV, -1, LVNI_SELECTED);
    if nItem != -1 {
        let _ = ListView_EditLabel!(hwndLV, nItem);
    }
}

unsafe extern "C" fn UserDelete(hwndDlg: HWND) -> BOOL {
    let mut szUserName: [TCHAR; (UNLEN + 1) as _] = [0; (UNLEN + 1) as _];
    let mut szText: [TCHAR; 256] = [0; 256];
    let mut nItem: INT;
    let mut hwndLV: HWND;
    let mut status: NET_API_STATUS;

    hwndLV = GetDlgItem(hwndDlg, (IDC_USERS_LIST as i32));
    nItem = ListView_GetNextItem!(hwndLV, -1, LVNI_SELECTED);
    if nItem == -1 {
        return (FALSE) as BOOL;
    }

    /* Get the new user name */
    ListView_GetItemText!(
        hwndLV,
        nItem,
        0,
        szUserName.as_mut_ptr(),
        UNLEN + 1,
    );

    /* Display a warning message because the delete operation cannot be reverted */
    let fmt_delete: [u16; 44] = [
        'D' as u16, 'o' as u16, ' ' as u16, 'y' as u16, 'o' as u16, 'u' as u16, ' ' as u16,
        'r' as u16, 'e' as u16, 'a' as u16, 'l' as u16, 'l' as u16, 'y' as u16, ' ' as u16,
        'w' as u16, 'a' as u16, 'n' as u16, 't' as u16, ' ' as u16,
        't' as u16, 'o' as u16, ' ' as u16,
        'd' as u16, 'e' as u16, 'l' as u16, 'e' as u16, 't' as u16, 'e' as u16, ' ' as u16,
        't' as u16, 'h' as u16, 'e' as u16, ' ' as u16,
        'u' as u16, 's' as u16, 'e' as u16, 'r' as u16, ' ' as u16,
        '"' as u16, '%' as u16, 's' as u16, '"' as u16, '?' as u16,
        0,
    ];
    wsprintf(szText.as_mut_ptr(), fmt_delete.as_ptr(), szUserName.as_mut_ptr());

    let title_accounts: [u16; 14] = [
        'U' as u16, 's' as u16, 'e' as u16, 'r' as u16, ' ' as u16,
        'A' as u16, 'c' as u16, 'c' as u16, 'o' as u16, 'u' as u16, 'n' as u16, 't' as u16, 's' as u16,
        0,
    ];
    if (MessageBox(
        core::ptr::null_mut(),
        szText.as_mut_ptr(),
        title_accounts.as_ptr(),
        (MB_ICONWARNING | MB_YESNO) as UINT,
    )) as u32 == IDNO
    {
        return (FALSE) as BOOL;
    }

    /* Delete the user */
    status = NetUserDel(core::ptr::null(), szUserName.as_ptr());
    if status != NERR_Success {
        let mut szText: [TCHAR; 256] = [0; 256];
        let fmt_error: [u16; 10] = [
            'E' as u16, 'r' as u16, 'r' as u16, 'o' as u16, 'r' as u16, ':' as u16, ' ' as u16,
            '%' as u16, 'u' as u16,
            0,
        ];
        wsprintf(szText.as_mut_ptr(), fmt_error.as_ptr(), status);

        let title_netuser: [u16; 11] = [
            'N' as u16, 'e' as u16, 't' as u16, 'U' as u16, 's' as u16, 'e' as u16, 'r' as u16,
            'D' as u16, 'e' as u16, 'l' as u16,
            0,
        ];
        MessageBox(
            core::ptr::null_mut(),
            szText.as_mut_ptr(),
            title_netuser.as_ptr(),
            (MB_ICONERROR | MB_OK) as UINT,
        );
        return (FALSE) as BOOL;
    }

    /* Delete the user from the list */
    let _ = ListView_DeleteItem!(hwndLV, nItem);

    return (TRUE) as BOOL;
}

unsafe extern "C" fn SetUsersListColumns(hwndListView: HWND) {
    let mut column: LV_COLUMN = core::mem::zeroed();
    let mut rect: RECT = core::mem::zeroed();
    let mut szStr: [TCHAR; 32] = [0; 32];

    GetClientRect(hwndListView, core::ptr::addr_of_mut!(rect));

    memset(
        core::ptr::addr_of_mut!(column) as *mut core::ffi::c_void,
        0x00 as core::ffi::c_int,
        core::mem::size_of::<LV_COLUMN>() as usize,
    );
    column.mask = (LVCF_FMT | LVCF_WIDTH | LVCF_SUBITEM | LVCF_TEXT) as UINT;
    column.fmt = LVCFMT_LEFT as core::ffi::c_int;
    column.cx = (((rect.right - rect.left) as f64) * (0.25 as f64)) as INT;
    column.iSubItem = 0 as core::ffi::c_int;
    LoadString(
        hApplet,
        IDS_NAME as UINT,
        core::ptr::addr_of_mut!(szStr) as *mut TCHAR,
        (core::mem::size_of::<[TCHAR; 32]>() / core::mem::size_of_val(&szStr[0])) as core::ffi::c_int,
    );
    column.pszText = core::ptr::addr_of_mut!(szStr) as *mut TCHAR;
    let _ = ListView_InsertColumn!(hwndListView, 0 as WPARAM, core::ptr::addr_of_mut!(column) as LPARAM);

    column.cx = (((rect.right - rect.left) as f64) * (0.50 as f64)) as INT;
    column.iSubItem = 1 as core::ffi::c_int;
    LoadString(
        hApplet,
        IDS_FULLNAME as UINT,
        core::ptr::addr_of_mut!(szStr) as *mut TCHAR,
        (core::mem::size_of::<[TCHAR; 32]>() / core::mem::size_of_val(&szStr[0])) as core::ffi::c_int,
    );
    column.pszText = core::ptr::addr_of_mut!(szStr) as *mut TCHAR;
    let _ = ListView_InsertColumn!(hwndListView, 1 as WPARAM, core::ptr::addr_of_mut!(column) as LPARAM);

    column.cx = (((rect.right - rect.left) as f64) * (0.25 as f64)) as INT;
    column.iSubItem = 2 as core::ffi::c_int;
    LoadString(
        hApplet,
        IDS_DESCRIPTION as UINT,
        core::ptr::addr_of_mut!(szStr) as *mut TCHAR,
        (core::mem::size_of::<[TCHAR; 32]>() / core::mem::size_of_val(&szStr[0])) as core::ffi::c_int,
    );
    column.pszText = core::ptr::addr_of_mut!(szStr) as *mut TCHAR;
    let _ = ListView_InsertColumn!(hwndListView, 2 as WPARAM, core::ptr::addr_of_mut!(column) as LPARAM);
}

unsafe extern "C" fn UpdateUsersList(hwndListView: HWND) {
    let mut netStatus: DWORD;
    let mut pBuffer: PUSER_INFO_20 = core::ptr::null_mut();
    let mut entriesread: DWORD = 0;
    let mut totalentries: DWORD = 0;
    let mut resume_handle: DWORD = 0;
    let mut i: DWORD;
    let mut lvi: LV_ITEM = core::mem::zeroed();
    let mut iItem: INT;

    loop {
        netStatus = NetUserEnum(
            core::ptr::null(),
            20,
            FILTER_NORMAL_ACCOUNT,
            core::ptr::addr_of_mut!(pBuffer) as *mut LPBYTE,
            1024,
            core::ptr::addr_of_mut!(entriesread),
            core::ptr::addr_of_mut!(totalentries),
            core::ptr::addr_of_mut!(resume_handle),
        );
        if netStatus != NERR_Success && netStatus != ERROR_MORE_DATA {
            break;
        }

        i = 0;
        while i < entriesread {
            memset(
                core::ptr::addr_of_mut!(lvi) as *mut core::ffi::c_void,
                0x00,
                core::mem::size_of::<LV_ITEM>() as usize,
            );
            lvi.mask = LVIF_TEXT | LVIF_STATE | LVIF_IMAGE;
            lvi.pszText = (*pBuffer.offset(i as isize)).usri20_name;
            lvi.state = 0;
            lvi.iImage = if ((*pBuffer.offset(i as isize)).usri20_flags & UF_ACCOUNTDISABLE) != 0 {
                1
            } else {
                0
            };
            iItem = ListView_InsertItem!(hwndListView, core::ptr::addr_of_mut!(lvi));

            if !(*pBuffer.offset(i as isize)).usri20_full_name.is_null() {
                ListView_SetItemText!(
                    hwndListView,
                    iItem,
                    1,
                    (*pBuffer.offset(i as isize)).usri20_full_name,
                );
            }

            if !(*pBuffer.offset(i as isize)).usri20_comment.is_null() {
                ListView_SetItemText!(
                    hwndListView,
                    iItem,
                    2,
                    (*pBuffer.offset(i as isize)).usri20_comment,
                );
            }

            i = i.wrapping_add(1);
        }

        NetApiBufferFree(pBuffer as PVOID);

        /* No more data left */
        if netStatus != ERROR_MORE_DATA {
            break;
        }
    }
}

unsafe extern "C" fn OnInitDialog(hwndDlg: HWND) {
    let mut hwndListView: HWND = core::ptr::null_mut();
    let mut hImgList: HIMAGELIST = core::ptr::null_mut();
    let mut hIcon: HICON = core::ptr::null_mut();

    /* Create the image list */
    hImgList = ImageList_Create(16, 16, ILC_COLOR32 | ILC_MASK, 5, 5);
    hIcon = (LoadImage(hApplet, MAKEINTRESOURCE!(IDI_USER), IMAGE_ICON, 16, 16, LR_DEFAULTCOLOR)).cast();
    ImageList_AddIcon!(hImgList, hIcon);
    hIcon = (LoadImage(hApplet, MAKEINTRESOURCE!(IDI_LOCKED_USER), IMAGE_ICON, 16, 16, LR_DEFAULTCOLOR)).cast();
    ImageList_AddIcon!(hImgList, hIcon);
    DestroyIcon(hIcon);

    hwndListView = GetDlgItem(hwndDlg, (IDC_USERS_LIST as i32));

    ListView_SetImageList!(hwndListView, hImgList, LVSIL_SMALL);

    ListView_SetExtendedListViewStyle!(hwndListView, LVS_EX_FULLROWSELECT);

    SetUsersListColumns(hwndListView);

    UpdateUsersList(hwndListView);
}

unsafe extern "C" fn OnBeginLabelEdit(pnmv: LPNMLVDISPINFO) -> BOOL {
    let mut hwndEdit: HWND = core::ptr::null_mut();

    hwndEdit = ListView_GetEditControl!((*pnmv).hdr.hwndFrom);
    if hwndEdit.is_null() {
        return (TRUE) as BOOL;
    }

    SendMessage(hwndEdit, EM_SETLIMITTEXT, 20, 0);

    return (FALSE) as BOOL;
}

unsafe extern "C" fn OnEndLabelEdit(pnmv: LPNMLVDISPINFO) -> BOOL {
    let mut szOldUserName: [TCHAR; (UNLEN + 1) as _] = [0; (UNLEN + 1) as _];
    let mut szNewUserName: [TCHAR; (UNLEN + 1) as _] = [0; (UNLEN + 1) as _];
    let mut useri0: USER_INFO_0 = USER_INFO_0 {
        usri0_name: core::ptr::null_mut(),
    };
    let mut status: NET_API_STATUS = 0;

    /* Leave, if there is no valid listview item */
    if (*pnmv).item.iItem == -1 {
        return (FALSE) as BOOL;
    }

    /* Get the new user name */
    ListView_GetItemText!(
        (*pnmv).hdr.hwndFrom,
        (*pnmv).item.iItem,
        0,
        &raw mut szOldUserName as *mut TCHAR,
        UNLEN + 1,
    );

    /* Leave, if the user canceled the edit action */
    if (*pnmv).item.pszText.is_null() {
        return (FALSE) as BOOL;
    }

    /* Get the new user name */
    lstrcpy(
        &raw mut szNewUserName as *mut TCHAR,
        (*pnmv).item.pszText,
    );

    /* Leave, if the user name was not changed */
    if lstrcmp(
        &raw mut szOldUserName as *mut TCHAR,
        &raw mut szNewUserName as *mut TCHAR,
    ) == 0
    {
        return (FALSE) as BOOL;
    }

    /* Check the user name for illegal characters */
    if CheckAccountName(
        core::ptr::null_mut(),
        0,
        &raw mut szNewUserName as *mut TCHAR,
    ) == 0
    {
        return (FALSE) as BOOL;
    }

    /* Change the user name */
    useri0.usri0_name = &raw mut szNewUserName as *mut TCHAR as LPWSTR;

    status = NetUserSetInfo(
        core::ptr::null(),
        &raw mut szOldUserName as *mut TCHAR as LPCWSTR,
        0,
        &raw mut useri0 as LPBYTE,
        core::ptr::null_mut(),
    );
    if status != NERR_Success {
        let mut szText: [TCHAR; 256] = [0; 256];
        wsprintf(
            &raw mut szText as *mut TCHAR,
            ['E' as u16, 'r' as u16, 'r' as u16, 'o' as u16, 'r' as u16, ':' as u16, ' ' as u16, '%' as u16, 'u' as u16, 0u16].as_ptr(),
            status,
        );
        MessageBox(
            core::ptr::null_mut(),
            &raw mut szText as *mut TCHAR,
            ['N' as u16, 'e' as u16, 't' as u16, 'U' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'S' as u16, 'e' as u16, 't' as u16, 'I' as u16, 'n' as u16, 'f' as u16, 'o' as u16, 0u16].as_ptr(),
            MB_ICONERROR | MB_OK,
        );
        return (FALSE) as BOOL;
    }

    /* Update the listview item */
    ListView_SetItemText!(
        (*pnmv).hdr.hwndFrom,
        (*pnmv).item.iItem,
        0,
        &raw mut szNewUserName as *mut TCHAR,
    );

    return (TRUE) as BOOL;
}

unsafe extern "C" fn OnNotify(hwndDlg: HWND, pUserData: PUSER_DATA, phdr: *mut NMHDR) -> BOOL {
    let lpnmlv: LPNMLISTVIEW = phdr as LPNMLISTVIEW;

    match (*phdr).idFrom {
        idfrom if idfrom == (IDC_USERS_LIST as _) => {
            match (*phdr).code {
                NM_CLICK => {
                    (*pUserData).iCurrentItem = (*lpnmlv).iItem as INT;
                }

                NM_DBLCLK => {
                    if (*lpnmlv).iItem != -(1 as INT) {
                        let mut uItem: UINT = 0;

                        uItem = GetMenuDefaultItem(
                            GetSubMenu((*pUserData).hPopupMenu, 1 as INT),
                            FALSE as UINT,
                            0 as UINT,
                        );
                        if uItem != (-(1 as i32)) as UINT {
                            SendMessage(
                                hwndDlg,
                                WM_COMMAND as UINT,
                                MAKEWPARAM!(uItem as WORD, 0 as WORD) as WPARAM,
                                0 as LPARAM,
                            );
                        }
                    }
                }

                NM_RCLICK => {
                    ClientToScreen(
                        GetDlgItem(hwndDlg, IDC_USERS_LIST as INT),
                        core::ptr::addr_of_mut!((*lpnmlv).ptAction),
                    );
                    TrackPopupMenu(
                        GetSubMenu(
                            (*pUserData).hPopupMenu,
                            if (*lpnmlv).iItem == -(1 as INT) {
                                0 as INT
                            } else {
                                1 as INT
                            },
                        ),
                        TPM_LEFTALIGN as UINT,
                        (*lpnmlv).ptAction.x as i32,
                        (*lpnmlv).ptAction.y as i32,
                        0 as i32,
                        hwndDlg,
                        core::ptr::null_mut(),
                    );
                }

                LVN_BEGINLABELEDIT => {
                    return OnBeginLabelEdit(phdr as LPNMLVDISPINFO);
                }

                LVN_ENDLABELEDIT => {
                    return OnEndLabelEdit(phdr as LPNMLVDISPINFO);
                }

                _ => {}
            }
        }
        _ => {}
    }

    return (FALSE) as BOOL;
}

unsafe extern "C" fn UpdateUserProperties(hwndDlg: HWND) {
    let mut szUserName: [TCHAR; (UNLEN + 1) as _] =
        [0; (UNLEN + 1) as _];
    let mut iItem: INT = 0;
    let mut hwndLV: HWND = core::ptr::null_mut();
    let mut pUserInfo: PUSER_INFO_2 = core::ptr::null_mut();
    let mut lvi: LV_ITEM = core::mem::zeroed();

    hwndLV = GetDlgItem(hwndDlg, (IDC_USERS_LIST as i32));
    iItem = ListView_GetNextItem!(
        hwndLV,
        -(1 as INT) as WPARAM,
        LVNI_SELECTED as LPARAM,
    );
    if iItem == -(1 as INT) {
        return;
    }

    /* Get the new user name */
    ListView_GetItemText!(
        hwndLV,
        iItem,
        0,
        szUserName.as_mut_ptr(),
        UNLEN + 1,
    );

    NetUserGetInfo(
        core::ptr::null(),
        szUserName.as_mut_ptr(),
        2,
        core::ptr::addr_of_mut!(pUserInfo) as *mut LPBYTE,
    );

    memset(
        core::ptr::addr_of_mut!(lvi) as *mut core::ffi::c_void,
        0x00,
        core::mem::size_of::<LV_ITEM>(),
    );
    lvi.iItem = iItem;
    lvi.iSubItem = 0;
    lvi.mask = LVIF_IMAGE;
    lvi.iImage = if ((*pUserInfo).usri2_flags & UF_ACCOUNTDISABLE) != 0 {
        1
    } else {
        0
    };
    let _ = ListView_SetItem!(hwndLV, core::ptr::addr_of_mut!(lvi));

    ListView_SetItemText!(
        hwndLV,
        iItem,
        1,
        (*pUserInfo).usri2_full_name,
    );

    ListView_SetItemText!(
        hwndLV,
        iItem,
        2,
        (*pUserInfo).usri2_comment,
    );

    NetApiBufferFree(pUserInfo as *mut core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn OnToggleRequireLogon(hwndDlg: HWND) {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut lResult: LONG = 0;
    let mut bIsChecked: BOOL = 0;
    let mut pszAutoAdminLogonValue: PCWSTR = core::ptr::null();

    lResult = RegCreateKeyExW(
        HKEY_LOCAL_MACHINE,
        RUSTCONV_WSTR_AA2E5A31D547.as_ptr(),
        0,
        core::ptr::null_mut(),
        REG_OPTION_NON_VOLATILE,
        KEY_SET_VALUE,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(hKey),
        core::ptr::null_mut(),
    );
    if (lResult) as u32 != ERROR_SUCCESS {
        return;
    }

    bIsChecked = (IsDlgButtonChecked(hwndDlg, (IDC_USERS_STARTUP_REQUIRE as i32)) == BST_CHECKED) as BOOL;
    pszAutoAdminLogonValue = if bIsChecked != 0 {
        RUSTCONV_WSTR_4F8320D91E97.as_ptr()
    } else {
        RUSTCONV_WSTR_60864AAE2645.as_ptr()
    };

    lResult = RegSetValueExW(
        hKey,
        RUSTCONV_WSTR_16ED7B7F1D78.as_ptr(),
        0,
        REG_SZ,
        pszAutoAdminLogonValue as *const u8,
        (2 as usize * core::mem::size_of::<WCHAR>()) as DWORD,
    );
    if (lResult) as u32 != ERROR_SUCCESS {
    }

    RegCloseKey(hKey);
}

#[no_mangle]
pub unsafe extern "system" fn UsersPageProc(hwndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let mut pUserData: PUSER_DATA = core::ptr::null_mut();

    pUserData = GetWindowLongPtr(hwndDlg, DWLP_USER) as PUSER_DATA;

    match uMsg {
        WM_INITDIALOG => {
            'case1: {
                let mut lResult: LONG = 0;
                let mut hKey: HKEY = core::ptr::null_mut();
                let mut szAutoAdminLogonValue: [WCHAR; 2] =
                    [0; 2];
                let mut dwType: DWORD = 0;
                let mut dwSize: DWORD =
                    core::mem::size_of::<[WCHAR; 2]>() as DWORD;
                let mut bRequireLogon: BOOL = (TRUE) as i32;

                pUserData = HeapAlloc(
                    GetProcessHeap(),
                    HEAP_ZERO_MEMORY,
                    (core::mem::size_of::<USER_DATA>()) as SIZE_T,
                ) as PUSER_DATA;
                SetWindowLongPtr(hwndDlg, DWLP_USER, pUserData as LONG_PTR);

                (*pUserData).hPopupMenu =
                    LoadMenu(hApplet, MAKEINTRESOURCE!(IDM_POPUP_USER));

                OnInitDialog(hwndDlg);
                lResult = RegOpenKeyExW(
                    HKEY_LOCAL_MACHINE,
                    RUSTCONV_WSTR_AA2E5A31D547.as_ptr(),
                    0,
                    KEY_READ,
                    core::ptr::addr_of_mut!(hKey),
                );
                if (lResult) as u32 != ERROR_SUCCESS {
                    CheckDlgButton(
                        hwndDlg,
                        (IDC_USERS_STARTUP_REQUIRE as i32),
                        BST_CHECKED,
                    );
                    break 'case1;
                }

                lResult = RegQueryValueExW(
                    hKey,
                    RUSTCONV_WSTR_16ED7B7F1D78.as_ptr(),
                    core::ptr::null_mut(),
                    core::ptr::addr_of_mut!(dwType),
                    core::ptr::addr_of_mut!(szAutoAdminLogonValue) as LPBYTE,
                    core::ptr::addr_of_mut!(dwSize),
                );
                RegCloseKey(hKey);

                if (lResult) as u32 == ERROR_SUCCESS
                    && dwType == REG_SZ
                    && wcscmp(
                        addr_of_mut!(szAutoAdminLogonValue) as *mut WCHAR,
                        RUSTCONV_WSTR_60864AAE2645.as_ptr(),
                    ) == 0
                {
                    bRequireLogon = (FALSE) as i32;
                }

                CheckDlgButton(
                    hwndDlg,
                    (IDC_USERS_STARTUP_REQUIRE as i32),
                    if bRequireLogon != 0 {
                        BST_CHECKED
                    } else {
                        BST_UNCHECKED
                    },
                );

                SetMenuDefaultItem(
                    GetSubMenu((*pUserData).hPopupMenu, 1),
                    IDM_USER_PROPERTIES,
                    FALSE,
                );
            }
        }

        WM_COMMAND => {
            match (LOWORD!(wParam)) as u32 {
                IDM_USER_CHANGE_PASSWORD => {
                    UserChangePassword(hwndDlg);
                }

                IDM_USER_RENAME => {
                    UserRename(hwndDlg);
                }

                IDM_USER_NEW | IDC_USERS_ADD => {
                    UserNew(hwndDlg);
                }

                IDM_USER_DELETE | IDC_USERS_REMOVE => {
                    UserDelete(hwndDlg);
                }

                IDM_USER_PROPERTIES | IDC_USERS_PROPERTIES => {
                    if UserProperties(hwndDlg) != 0 {
                        UpdateUserProperties(hwndDlg);
                    }
                }
                IDC_USERS_STARTUP_REQUIRE => {
                    OnToggleRequireLogon(hwndDlg);
                }
                _ => {}
            }
        }

        WM_NOTIFY => {
            return OnNotify(hwndDlg, pUserData, lParam as *mut NMHDR) as INT_PTR;
        }

        WM_DESTROY => {
            DestroyMenu((*pUserData).hPopupMenu);
            HeapFree(
                GetProcessHeap(),
                0,
                pUserData as PVOID,
            );
        }

        _ => {}
    }

    return (FALSE) as INT_PTR;
}
