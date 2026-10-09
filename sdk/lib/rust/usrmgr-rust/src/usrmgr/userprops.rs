use crate::usrmgr::usrmgr::GetComboBoxLBTextAlloc;
use crate::usrmgr::usrmgr::GetDlgItemTextAlloc;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct _GENERAL_USER_DATA {
    pub dwFlags: DWORD,
    pub dwPasswordExpired: DWORD,
    pub szUserName: [TCHAR; 1],
}
pub type GENERAL_USER_DATA = _GENERAL_USER_DATA;
pub type PGENERAL_USER_DATA = *mut GENERAL_USER_DATA;

// VALID_GENERAL_FLAGS = UF_PASSWD_CANT_CHANGE | UF_DONT_EXPIRE_PASSWD | UF_ACCOUNTDISABLE | UF_LOCKOUT
// = 0x40 | 0x10000 | 0x2 | 0x200
pub const VALID_GENERAL_FLAGS: DWORD = 0x10052 as DWORD;

#[repr(C)]
pub struct _PROFILE_USER_DATA {
    pub szUserName: [TCHAR; 1],
}
pub type PROFILE_USER_DATA = _PROFILE_USER_DATA;
pub type PPROFILE_USER_DATA = *mut PROFILE_USER_DATA;

#[repr(C)]
pub struct _MEMBERSHIP_USER_DATA {
    pub pGroupData: PLOCALGROUP_USERS_INFO_0,
    pub dwGroupCount: DWORD,
    pub szUserName: [TCHAR; 1],
}
pub type MEMBERSHIP_USER_DATA = _MEMBERSHIP_USER_DATA;
pub type PMEMBERSHIP_USER_DATA = *mut MEMBERSHIP_USER_DATA;

unsafe extern "C" fn GetUserProfileData(hwndDlg: HWND, pUserData: PPROFILE_USER_DATA) {
    let mut userInfo: PUSER_INFO_3 = core::ptr::null_mut();
    let mut status: NET_API_STATUS;
    let mut bLocal: BOOL;
    let mut szDrive: [TCHAR; 8] = [0; 8];
    let mut i: INT;
    let mut nSel: INT = core::mem::zeroed();

    status = NetUserGetInfo(
        core::ptr::null(),
        &raw mut (*pUserData).szUserName as *mut TCHAR as LPCWSTR,
        3,
        core::ptr::addr_of_mut!(userInfo) as *mut LPBYTE,
    );
    if status != NERR_Success as NET_API_STATUS {
        return;
    }

    SetDlgItemText(hwndDlg, (IDC_USER_PROFILE_PATH as i32), (*userInfo).usri3_profile as LPCWSTR);
    SetDlgItemText(hwndDlg, (IDC_USER_PROFILE_SCRIPT as i32), (*userInfo).usri3_script_path as LPCWSTR);

    bLocal = ((*userInfo).usri3_home_dir_drive.is_null()
        || _tcslen((*userInfo).usri3_home_dir_drive) == (0 as usize) as _)
        as BOOL;
    CheckRadioButton(
        hwndDlg,
        (IDC_USER_PROFILE_LOCAL as i32),
        (IDC_USER_PROFILE_REMOTE as i32),
        if bLocal != 0 { (IDC_USER_PROFILE_LOCAL as i32) } else { (IDC_USER_PROFILE_REMOTE as i32) },
    );

    i = 0 as INT;
    while i < 26 as INT {
        wsprintf(
            &raw mut szDrive as *mut TCHAR,
            (&[0x0025, 0x0063, 0x003a, 0x0000u16]).as_ptr() /* _T "%c:" */,
            ('A' as INT + i) as TCHAR as core::ffi::c_uint,
        );
        nSel = SendMessage(
            GetDlgItem(hwndDlg, (IDC_USER_PROFILE_DRIVE as i32)),
            CB_INSERTSTRING as UINT,
            -(1 as INT) as WPARAM,
            &raw mut szDrive as *mut TCHAR as LPARAM,
        ) as INT;
        i += 1;
    }

    if bLocal != 0 {
        SetDlgItemText(
            hwndDlg,
            (IDC_USER_PROFILE_LOCAL_PATH as i32),
            (*userInfo).usri3_home_dir as LPCWSTR,
        );
        EnableWindow(
            GetDlgItem(hwndDlg, (IDC_USER_PROFILE_DRIVE as i32)),
            (FALSE) as BOOL,
        );
        EnableWindow(
            GetDlgItem(hwndDlg, (IDC_USER_PROFILE_REMOTE_PATH as i32)),
            (FALSE) as BOOL,
        );
    } else {
        SetDlgItemText(
            hwndDlg,
            (IDC_USER_PROFILE_REMOTE_PATH as i32),
            (*userInfo).usri3_home_dir as LPCWSTR,
        );
        nSel = SendMessage(
            GetDlgItem(hwndDlg, (IDC_USER_PROFILE_DRIVE as i32)),
            CB_FINDSTRINGEXACT as UINT,
            -(1 as INT) as WPARAM,
            (*userInfo).usri3_home_dir_drive as LPARAM,
        ) as INT;
    }

    SendMessage(
        GetDlgItem(hwndDlg, (IDC_USER_PROFILE_DRIVE as i32)),
        CB_SETCURSEL as UINT,
        nSel as WPARAM,
        0 as LPARAM,
    );

    NetApiBufferFree(userInfo as PVOID);
}

unsafe extern "C" fn SetUserProfileData(hwndDlg: HWND, pUserData: PPROFILE_USER_DATA) -> BOOL {
    let mut pUserInfo: PUSER_INFO_3 = core::ptr::null_mut();
    let mut status: DWORD = 0;
    let mut dwIndex: DWORD = 0;
    let mut nIndex: INT = 0;

    NetUserGetInfo(
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!((*pUserData).szUserName) as *mut TCHAR as LPCWSTR,
        3,
        core::ptr::addr_of_mut!(pUserInfo) as *mut LPBYTE,
    );

    /* Get the profile path */
    (*pUserInfo).usri3_profile = GetDlgItemTextAlloc(hwndDlg, (IDC_USER_PROFILE_PATH) as INT) as LPWSTR;

    /* Get the script path */
    (*pUserInfo).usri3_script_path = GetDlgItemTextAlloc(hwndDlg, (IDC_USER_PROFILE_SCRIPT) as INT) as LPWSTR;

    if IsDlgButtonChecked(hwndDlg, (IDC_USER_PROFILE_LOCAL as i32)) == BST_CHECKED {
        /* Local home directory */
        (*pUserInfo).usri3_home_dir = GetDlgItemTextAlloc(hwndDlg, (IDC_USER_PROFILE_LOCAL_PATH) as INT) as LPWSTR;
    } else {
        /* Remote home directory */
        (*pUserInfo).usri3_home_dir = GetDlgItemTextAlloc(hwndDlg, (IDC_USER_PROFILE_REMOTE_PATH) as INT) as LPWSTR;

        nIndex = SendMessage(
            GetDlgItem(hwndDlg, (IDC_USER_PROFILE_DRIVE as i32)),
            CB_GETCURSEL,
            0,
            0,
        ) as INT;
        if nIndex != CB_ERR {
            (*pUserInfo).usri3_home_dir_drive =
                GetComboBoxLBTextAlloc(hwndDlg, (IDC_USER_PROFILE_DRIVE) as INT, nIndex) as LPWSTR;
        } else {
            (*pUserInfo).usri3_home_dir_drive = HeapAlloc(
                GetProcessHeap(),
                HEAP_ZERO_MEMORY,
                core::mem::size_of::<TCHAR>() as SIZE_T,
            ) as LPWSTR;
        }
    }

    status = NetUserSetInfo(
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!((*pUserData).szUserName) as *mut TCHAR as LPCWSTR,
        3,
        pUserInfo as LPBYTE,
        core::ptr::addr_of_mut!(dwIndex),
    );
    if status != NERR_Success {
    }

    HeapFree(
        GetProcessHeap(),
        0,
        (*pUserInfo).usri3_profile as PVOID,
    );
    HeapFree(
        GetProcessHeap(),
        0,
        (*pUserInfo).usri3_script_path as PVOID,
    );
    HeapFree(
        GetProcessHeap(),
        0,
        (*pUserInfo).usri3_home_dir as PVOID,
    );
    HeapFree(
        GetProcessHeap(),
        0,
        (*pUserInfo).usri3_home_dir_drive as PVOID,
    );

    NetApiBufferFree(pUserInfo as PVOID);

    return (status == NERR_Success) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn UserProfilePageProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut pUserData: PPROFILE_USER_DATA;

    UNREFERENCED_PARAMETER!(lParam);
    UNREFERENCED_PARAMETER!(wParam);
    UNREFERENCED_PARAMETER!(hwndDlg);

    pUserData = GetWindowLongPtr(hwndDlg, DWLP_USER) as PPROFILE_USER_DATA;

    match uMsg {
        WM_INITDIALOG => {
            pUserData = HeapAlloc(
                GetProcessHeap(),
                HEAP_ZERO_MEMORY,
                (core::mem::size_of::<PROFILE_USER_DATA>()
                    + (lstrlen((*(lParam as *mut PROPSHEETPAGE)).lParam as LPTSTR) as usize)
                        * core::mem::size_of::<TCHAR>()) as SIZE_T,
            ) as PPROFILE_USER_DATA;
            lstrcpy(
                core::ptr::addr_of_mut!((*pUserData).szUserName) as LPTSTR,
                (*(lParam as *mut PROPSHEETPAGE)).lParam as LPTSTR,
            );

            SetWindowLongPtr(hwndDlg, DWLP_USER, pUserData as INT_PTR);

            GetUserProfileData(hwndDlg, pUserData);
        }

        WM_COMMAND => {
            match (LOWORD!(wParam)) as u32 {
                IDC_USER_PROFILE_PATH | IDC_USER_PROFILE_SCRIPT => {
                    if (HIWORD!(wParam)) as u32 == EN_CHANGE {
                        PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                    }
                }

                IDC_USER_PROFILE_LOCAL => {
                    EnableWindow(
                        GetDlgItem(hwndDlg, (IDC_USER_PROFILE_LOCAL_PATH as i32)),
                        (TRUE) as BOOL,
                    );
                    EnableWindow(
                        GetDlgItem(hwndDlg, (IDC_USER_PROFILE_DRIVE as i32)),
                        (FALSE) as BOOL,
                    );
                    EnableWindow(
                        GetDlgItem(hwndDlg, (IDC_USER_PROFILE_REMOTE_PATH as i32)),
                        (FALSE) as BOOL,
                    );
                }

                IDC_USER_PROFILE_REMOTE => {
                    EnableWindow(
                        GetDlgItem(hwndDlg, (IDC_USER_PROFILE_LOCAL_PATH as i32)),
                        (FALSE) as BOOL,
                    );
                    EnableWindow(
                        GetDlgItem(hwndDlg, (IDC_USER_PROFILE_DRIVE as i32)),
                        (TRUE) as BOOL,
                    );
                    EnableWindow(
                        GetDlgItem(hwndDlg, (IDC_USER_PROFILE_REMOTE_PATH as i32)),
                        (TRUE) as BOOL,
                    );
                }

                _ => {}
            }
        }

        WM_DESTROY => {
            HeapFree(
                GetProcessHeap(),
                0,
                pUserData as PVOID,
            );
        }

        WM_NOTIFY => {
            if (*(lParam as LPPSHNOTIFY)).hdr.code == (PSN_APPLY) as u32 {
                SetUserProfileData(hwndDlg, pUserData);
                return TRUE as INT_PTR;
            }
        }

        _ => {}
    }

    return FALSE as INT_PTR;
}

unsafe extern "C" fn GetUserMembershipData(hwndDlg: HWND, pUserData: PMEMBERSHIP_USER_DATA) {
    let mut status: NET_API_STATUS = 0;
    let mut dwTotal: DWORD = 0;
    let mut i: DWORD = 0;
    let mut hImgList: HIMAGELIST = core::ptr::null_mut();
    let mut hIcon: HICON = core::ptr::null_mut();
    let mut lvi: LV_ITEM = core::mem::zeroed();
    let mut hwndLV: HWND = core::ptr::null_mut();
    let mut column: LV_COLUMN = core::mem::zeroed();
    let mut rect: RECT = core::mem::zeroed();

    hwndLV = GetDlgItem(hwndDlg, (IDC_USER_MEMBERSHIP_LIST as i32));

    /* Create the image list */
    hImgList = ImageList_Create(16, 16, ILC_COLOR32 | ILC_MASK, 5, 5);
    hIcon = LoadImage(hApplet, MAKEINTRESOURCE!(IDI_GROUP), IMAGE_ICON, 16, 16, LR_DEFAULTCOLOR) as HICON;
    ImageList_AddIcon!(hImgList, hIcon);
    DestroyIcon(hIcon);
    let _ = ListView_SetImageList!(hwndLV, hImgList, LVSIL_SMALL);

    /* Set the list column */
    GetClientRect(hwndLV, core::ptr::addr_of_mut!(rect));

    memset(core::ptr::addr_of_mut!(column) as *mut core::ffi::c_void, 0x00, core::mem::size_of::<LV_COLUMN>());
    column.mask = LVCF_FMT | LVCF_WIDTH | LVCF_SUBITEM;
    column.fmt = (LVCFMT_LEFT as i32);
    column.cx = (rect.right - rect.left) as INT;
    column.iSubItem = 0;
    let _ = ListView_InsertColumn!(hwndLV, 0, core::ptr::addr_of_mut!(column));

    status = NetUserGetLocalGroups(
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!((*pUserData).szUserName) as LPCWSTR,
        0,
        0,
        core::ptr::addr_of_mut!((*pUserData).pGroupData) as *mut LPBYTE,
        MAX_PREFERRED_LENGTH,
        core::ptr::addr_of_mut!((*pUserData).dwGroupCount),
        core::ptr::addr_of_mut!(dwTotal),
    );
    if status != NERR_Success {
        return;
    }

    i = 0;
    while i < (*pUserData).dwGroupCount {
        ZeroMemory!(core::ptr::addr_of_mut!(lvi) as *mut core::ffi::c_void, core::mem::size_of::<LV_ITEM>());
        lvi.mask = LVIF_TEXT | LVIF_STATE | LVIF_IMAGE;
        lvi.pszText = (*(*pUserData).pGroupData.offset(i as isize)).lgrui0_name;
        lvi.state = 0;
        lvi.iImage = 0;

        let _ = ListView_InsertItem!(hwndLV, core::ptr::addr_of_mut!(lvi));
        i += 1;
    }
}

unsafe extern "C" fn RemoveGroupFromUser(hwndDlg: HWND, pUserData: PMEMBERSHIP_USER_DATA) {
    let mut szGroupName: [TCHAR; (UNLEN + 1) as _] = [0; (UNLEN + 1) as _];
    let mut szText: [TCHAR; 256] = [0; 256];
    let mut memberInfo: LOCALGROUP_MEMBERS_INFO_3 = LOCALGROUP_MEMBERS_INFO_3 {
        lgrmi3_domainandname: core::ptr::null_mut(),
    };
    let mut hwndLV: HWND = core::ptr::null_mut();
    let mut nItem: INT = 0;
    let mut status: NET_API_STATUS = 0;

    let fmt_remove: [u16; 64] = [
        'D' as u16, 'o' as u16, ' ' as u16, 'y' as u16, 'o' as u16, 'u' as u16, ' ' as u16,
        'r' as u16, 'e' as u16, 'a' as u16, 'l' as u16, 'l' as u16, 'y' as u16, ' ' as u16,
        'w' as u16, 'a' as u16, 'n' as u16, 't' as u16, ' ' as u16,
        't' as u16, 'o' as u16, ' ' as u16,
        'r' as u16, 'e' as u16, 'm' as u16, 'o' as u16, 'v' as u16, 'e' as u16, ' ' as u16,
        't' as u16, 'h' as u16, 'e' as u16, ' ' as u16,
        'u' as u16, 's' as u16, 'e' as u16, 'r' as u16, ' ' as u16,
        '"' as u16, '%' as u16, 's' as u16, '"' as u16, ' ' as u16,
        'f' as u16, 'r' as u16, 'o' as u16, 'm' as u16, ' ' as u16,
        't' as u16, 'h' as u16, 'e' as u16, ' ' as u16,
        'g' as u16, 'r' as u16, 'o' as u16, 'u' as u16, 'p' as u16, ' ' as u16,
        '"' as u16, '%' as u16, 's' as u16, '"' as u16, '?' as u16, 0,
    ];
    let s_user_accounts: [u16; 14] = [
        'U' as u16, 's' as u16, 'e' as u16, 'r' as u16, ' ' as u16,
        'A' as u16, 'c' as u16, 'c' as u16, 'o' as u16, 'u' as u16, 'n' as u16, 't' as u16, 's' as u16,
        0,
    ];
    let fmt_error: [u16; 10] = [
        'E' as u16, 'r' as u16, 'r' as u16, 'o' as u16, 'r' as u16, ':' as u16, ' ' as u16,
        '%' as u16, 'u' as u16, 0,
    ];
    let s_netlocal: [u16; 24] = [
        'N' as u16, 'e' as u16, 't' as u16, 'L' as u16, 'o' as u16, 'c' as u16, 'a' as u16, 'l' as u16,
        'G' as u16, 'r' as u16, 'o' as u16, 'u' as u16, 'p' as u16,
        'D' as u16, 'e' as u16, 'l' as u16, 'M' as u16, 'e' as u16, 'm' as u16, 'b' as u16, 'e' as u16, 'r' as u16, 's' as u16,
        0,
    ];

    hwndLV = GetDlgItem(hwndDlg, (IDC_USER_MEMBERSHIP_LIST as i32));
    nItem = ListView_GetNextItem!(hwndLV, -1, LVNI_SELECTED);
    if nItem == -1 {
        return;
    }

    /* Get the new user name */
    ListView_GetItemText!(
        hwndLV,
        nItem,
        0,
        core::ptr::addr_of_mut!(szGroupName) as *mut TCHAR,
        UNLEN + 1,
    );

    /* Display a warning message because the remove operation cannot be reverted */
    wsprintf(
        core::ptr::addr_of_mut!(szText) as *mut TCHAR,
        core::ptr::addr_of!(fmt_remove) as *const TCHAR,
        core::ptr::addr_of_mut!((*pUserData).szUserName) as *mut TCHAR,
        core::ptr::addr_of_mut!(szGroupName) as *mut TCHAR,
    );
    if (MessageBox(
        core::ptr::null_mut(),
        core::ptr::addr_of!(szText) as *const TCHAR,
        core::ptr::addr_of!(s_user_accounts) as *const TCHAR,
        MB_ICONWARNING | MB_YESNO,
    )) as u32 == IDNO
    {
        return;
    }

    memberInfo.lgrmi3_domainandname = core::ptr::addr_of_mut!((*pUserData).szUserName) as *mut TCHAR as LPWSTR;

    status = NetLocalGroupDelMembers(
        core::ptr::null(),
        core::ptr::addr_of!(szGroupName) as *const TCHAR,
        3,
        core::ptr::addr_of_mut!(memberInfo) as LPBYTE,
        1,
    );
    if status != NERR_Success {
        let mut szText_0: [TCHAR; 256] = [0; 256];
        wsprintf(
            core::ptr::addr_of_mut!(szText_0) as *mut TCHAR,
            core::ptr::addr_of!(fmt_error) as *const TCHAR,
            status,
        );
        MessageBox(
            core::ptr::null_mut(),
            core::ptr::addr_of!(szText_0) as *const TCHAR,
            core::ptr::addr_of!(s_netlocal) as *const TCHAR,
            MB_ICONERROR | MB_OK,
        );
        return;
    }

    let _ = ListView_DeleteItem!(hwndLV, nItem);

    if ListView_GetItemCount!(hwndLV) == 0 {
        EnableWindow(
            GetDlgItem(hwndDlg, (IDC_USER_MEMBERSHIP_REMOVE as i32)),
            (FALSE) as BOOL,
        );
    }
}

unsafe extern "C" fn InitUserGroupsList(hwndDlg: HWND) {
    let mut hwndLV: HWND = core::ptr::null_mut();
    let mut column: LV_COLUMN = core::mem::zeroed();
    let mut rect: RECT = core::mem::zeroed();
    let mut szStr: [TCHAR; 32] = core::mem::zeroed();

    let mut netStatus: NET_API_STATUS;
    let mut pBuffer: PLOCALGROUP_INFO_1 = core::ptr::null_mut();
    let mut entriesread: DWORD = 0;
    let mut totalentries: DWORD = 0;
    let mut resume_handle: DWORD_PTR = 0;
    let mut i: DWORD;
    let mut lvi: LV_ITEM = core::mem::zeroed();
    let mut iItem: INT;

    let mut hImgList: HIMAGELIST;
    let mut hIcon: HICON;

    hwndLV = GetDlgItem(hwndDlg, (IDC_USER_ADD_MEMBERSHIP_LIST as i32));
    GetClientRect(hwndLV, core::ptr::addr_of_mut!(rect));

    hImgList = ImageList_Create(16, 16, ILC_COLOR32 | ILC_MASK, 5, 5);
    hIcon = LoadImage(hApplet, MAKEINTRESOURCE!(IDI_GROUP), IMAGE_ICON, 16, 16, LR_DEFAULTCOLOR) as HICON;
    ImageList_AddIcon!(hImgList, hIcon);
    DestroyIcon(hIcon);

    ListView_SetImageList!(hwndLV, hImgList, LVSIL_SMALL);
    ListView_SetExtendedListViewStyle!(hwndLV, LVS_EX_FULLROWSELECT);

    memset(core::ptr::addr_of_mut!(column) as *mut core::ffi::c_void, 0x00, core::mem::size_of::<LV_COLUMN>());
    column.mask = LVCF_FMT | LVCF_WIDTH | LVCF_SUBITEM | LVCF_TEXT;
    column.fmt = (LVCFMT_LEFT as i32);
    column.cx = ((rect.right - rect.left) as f64 * 0.40) as INT;
    column.iSubItem = 0;
    LoadString(hApplet, IDS_NAME, core::ptr::addr_of_mut!(szStr) as *mut TCHAR, (core::mem::size_of_val(&szStr) / core::mem::size_of_val(&szStr[0])) as INT);
    column.pszText = core::ptr::addr_of_mut!(szStr) as *mut TCHAR;
    ListView_InsertColumn!(hwndLV, 0, core::ptr::addr_of_mut!(column));

    column.cx = ((rect.right - rect.left) as f64 * 0.60) as INT;
    column.iSubItem = 1;
    LoadString(hApplet, IDS_DESCRIPTION, core::ptr::addr_of_mut!(szStr) as *mut TCHAR, (core::mem::size_of_val(&szStr) / core::mem::size_of_val(&szStr[0])) as INT);
    column.pszText = core::ptr::addr_of_mut!(szStr) as *mut TCHAR;
    ListView_InsertColumn!(hwndLV, 1, core::ptr::addr_of_mut!(column));

    loop {
        netStatus = NetLocalGroupEnum(core::ptr::null(), 1, core::ptr::addr_of_mut!(pBuffer) as *mut LPBYTE,
                                      1024, core::ptr::addr_of_mut!(entriesread),
                                      core::ptr::addr_of_mut!(totalentries), core::ptr::addr_of_mut!(resume_handle));
        if netStatus != NERR_Success && netStatus != ERROR_MORE_DATA {
            break;
        }

        i = 0;
        while i < entriesread {
            memset(core::ptr::addr_of_mut!(lvi) as *mut core::ffi::c_void, 0x00, core::mem::size_of::<LV_ITEM>());
            lvi.mask = LVIF_TEXT | LVIF_STATE | LVIF_IMAGE;
            lvi.pszText = (*pBuffer.offset(i as isize)).lgrpi1_name;
            lvi.state = 0;
            lvi.iImage = 0;
            iItem = ListView_InsertItem!(hwndLV, core::ptr::addr_of_mut!(lvi));

            ListView_SetItemText!(hwndLV, iItem, 1,
                                 (*pBuffer.offset(i as isize)).lgrpi1_comment);
            i += 1;
        }

        NetApiBufferFree((pBuffer).cast());

        /* No more data left */
        if netStatus != ERROR_MORE_DATA {
            break;
        }
    }
}

unsafe extern "C" fn AddSelectedGroupsToUser(
    hwndDlg: HWND,
    pUserData: PMEMBERSHIP_USER_DATA,
) -> BOOL {
    let mut hwndLV: HWND;
    let mut nItem: INT;
    let mut szGroupName: [TCHAR; (UNLEN + 1) as _] =
        [0; (UNLEN + 1) as _];
    let mut bResult: BOOL = (FALSE) as i32;
    let mut bFound: BOOL;
    let mut i: DWORD;
    let mut memberInfo: LOCALGROUP_MEMBERS_INFO_3 = core::mem::zeroed();
    let mut status: NET_API_STATUS;

    hwndLV = GetDlgItem(hwndDlg, (IDC_USER_ADD_MEMBERSHIP_LIST as i32));

    if ListView_GetSelectedCount!(hwndLV) > 0 {
        nItem = ListView_GetNextItem!(hwndLV, -1, LVNI_SELECTED);
        while nItem != -1 {
            /* Get the new user name */
            ListView_GetItemText!(
                hwndLV,
                nItem,
                0,
                szGroupName.as_mut_ptr(),
                UNLEN + 1,
            );

            bFound = (FALSE) as i32;
            i = 0;
            while i < (*pUserData).dwGroupCount {
                if _tcscmp(
                    (*(*pUserData).pGroupData.add(i as usize)).lgrui0_name,
                    szGroupName.as_mut_ptr(),
                ) == 0
                {
                    bFound = (TRUE) as i32;
                }
                i += 1;
            }

            if bFound == 0 {
                memberInfo.lgrmi3_domainandname = (*pUserData).szUserName.as_mut_ptr();

                status = NetLocalGroupAddMembers(
                    core::ptr::null_mut(),
                    szGroupName.as_mut_ptr(),
                    3,
                    core::ptr::addr_of_mut!(memberInfo) as LPBYTE,
                    1,
                );
                if status == NERR_Success {
                    bResult = (TRUE) as i32;
                } else {
                    let mut szText: [TCHAR; 256] =
                        [0; 256];
                    static ERROR_FMT: [u16; 11] = [
                        'E' as u16, 'r' as u16, 'r' as u16, 'o' as u16, 'r' as u16,
                        ':' as u16, ' ' as u16, '%' as u16, 'u' as u16, 0,
                        0,
                    ];
                    static API_NAME: [u16; 24] = [
                        'N' as u16, 'e' as u16, 't' as u16, 'L' as u16, 'o' as u16,
                        'c' as u16, 'a' as u16, 'l' as u16, 'G' as u16, 'r' as u16,
                        'o' as u16, 'u' as u16, 'p' as u16, 'A' as u16, 'd' as u16,
                        'd' as u16, 'M' as u16, 'e' as u16, 'm' as u16, 'b' as u16,
                        'e' as u16, 'r' as u16, 's' as u16, 0,
                    ];
                    wsprintf(
                        szText.as_mut_ptr(),
                        (ERROR_FMT.as_ptr()),
                        status,
                    );
                    MessageBox(
                        core::ptr::null_mut(),
                        szText.as_mut_ptr(),
                        (API_NAME.as_ptr()),
                        MB_ICONERROR | MB_OK,
                    );
                }
            }

            nItem = ListView_GetNextItem!(hwndLV, nItem, LVNI_SELECTED);
        }
    }

    return bResult;
}

#[no_mangle]
pub unsafe extern "system" fn AddGroupToUserDlgProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut pUserData: PMEMBERSHIP_USER_DATA = core::ptr::null_mut();

    UNREFERENCED_PARAMETER!(wParam);

    pUserData = GetWindowLongPtr(hwndDlg, DWLP_USER as ::core::ffi::c_int) as PMEMBERSHIP_USER_DATA;

    match uMsg {
        WM_INITDIALOG => {
            pUserData = lParam as PMEMBERSHIP_USER_DATA;
            SetWindowLongPtr(hwndDlg, DWLP_USER as ::core::ffi::c_int, pUserData as LONG_PTR);
            InitUserGroupsList(hwndDlg);
        }

        WM_COMMAND => {
            match (LOWORD!(wParam) as ::core::ffi::c_int) as u32 {
                IDOK => {
                    if AddSelectedGroupsToUser(hwndDlg, pUserData) != 0 {
                        EndDialog(hwndDlg, IDOK as INT_PTR);
                    } else {
                        EndDialog(hwndDlg, IDCANCEL as INT_PTR);
                    }
                }

                IDCANCEL => {
                    EndDialog(hwndDlg, IDCANCEL as INT_PTR);
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

unsafe extern "C" fn AddGroupToUser(hwndDlg: HWND, pUserData: PMEMBERSHIP_USER_DATA) {
    let mut hwndLV: HWND;
    let mut status: NET_API_STATUS;
    let mut i: DWORD;
    let mut dwTotal: DWORD = core::mem::zeroed();
    let mut lvi: LV_ITEM = core::mem::zeroed();

    if (DialogBoxParam(hApplet,
                       MAKEINTRESOURCE!(IDD_USER_ADD_MEMBERSHIP),
                       hwndDlg,
                       Some(AddGroupToUserDlgProc),
                       pUserData as LPARAM)) as u32 == (IDOK) as _
    {
        // TODO: Update Membership list!
        hwndLV = GetDlgItem(hwndDlg, (IDC_USER_MEMBERSHIP_LIST as i32));

        if !(*pUserData).pGroupData.is_null() {
            NetApiBufferFree((*pUserData).pGroupData as PVOID);
        }

        ListView_DeleteAllItems!(hwndLV);

        status = NetUserGetLocalGroups(core::ptr::null_mut(),
                                       core::ptr::addr_of_mut!((*pUserData).szUserName) as LPWSTR,
                                       0, 0,
                                       core::ptr::addr_of_mut!((*pUserData).pGroupData) as *mut LPBYTE,
                                       MAX_PREFERRED_LENGTH,
                                       core::ptr::addr_of_mut!((*pUserData).dwGroupCount),
                                       core::ptr::addr_of_mut!(dwTotal));
        if status != NERR_Success {
            return;
        }

        i = 0;
        while i < (*pUserData).dwGroupCount {
            ZeroMemory!(core::ptr::addr_of_mut!(lvi) as *mut core::ffi::c_void,
                       core::mem::size_of::<LV_ITEM>());
            lvi.mask = LVIF_TEXT | LVIF_STATE | LVIF_IMAGE;
            lvi.pszText = (*(*pUserData).pGroupData.add(i as usize)).lgrui0_name;
            lvi.state = 0;
            lvi.iImage = 0;

            ListView_InsertItem!(hwndLV, core::ptr::addr_of_mut!(lvi));

            i += 1;
        }
    }
}

static OnUserPropSheetNotify: unsafe extern "C" fn(HWND, PMEMBERSHIP_USER_DATA, LPARAM) -> BOOL = {
    unsafe extern "C" fn OnUserPropSheetNotify(
        mut hwndDlg: HWND,
        mut pUserData: PMEMBERSHIP_USER_DATA,
        mut lParam: LPARAM,
    ) -> BOOL {
        let mut lpnmlv: LPNMLISTVIEW = lParam as LPNMLISTVIEW;
        match (*(lParam as LPNMHDR)).idFrom {
            // C converts the `case` label to idFrom's UINT_PTR; Rust needs that in a guard.
            __c if __c == IDC_USER_MEMBERSHIP_LIST as UINT_PTR => match (((*(lParam as LPNMHDR)).code) as i32) as u32 {
                NM_CLICK => {
                    EnableWindow(
                        GetDlgItem(hwndDlg, (IDC_USER_MEMBERSHIP_REMOVE as i32)),
                        ((*lpnmlv).iItem != -1) as ::core::ffi::c_int,
                    );
                }
                __c if __c == LVN_KEYDOWN as u32 => {
                    if ((*(lParam as LPNMLVKEYDOWN)).wVKey as ::core::ffi::c_int) as u32
                        == VK_DELETE
                    {
                        RemoveGroupFromUser(hwndDlg, pUserData);
                    }
                }
                _ => {}
            },
            _ => {}
        }
        return (FALSE) as BOOL;
    }
    OnUserPropSheetNotify
};

#[no_mangle]
pub unsafe extern "system" fn UserMembershipPageProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut pUserData: PMEMBERSHIP_USER_DATA;

    UNREFERENCED_PARAMETER!(lParam);
    UNREFERENCED_PARAMETER!(wParam);
    UNREFERENCED_PARAMETER!(hwndDlg);

    pUserData = GetWindowLongPtr(hwndDlg, DWLP_USER as ::core::ffi::c_int) as PMEMBERSHIP_USER_DATA;

    match uMsg {
        WM_INITDIALOG => {
            pUserData = HeapAlloc(
                GetProcessHeap(),
                HEAP_ZERO_MEMORY as DWORD,
                (::core::mem::size_of::<MEMBERSHIP_USER_DATA>() as SIZE_T).wrapping_add(
                    (lstrlen((*(lParam as *mut PROPSHEETPAGE)).lParam as LPTSTR) as SIZE_T)
                        .wrapping_mul(::core::mem::size_of::<TCHAR>() as SIZE_T),
                ),
            ) as PMEMBERSHIP_USER_DATA;
            lstrcpy(
                core::ptr::addr_of_mut!((*pUserData).szUserName) as LPTSTR,
                (*(lParam as *mut PROPSHEETPAGE)).lParam as LPTSTR,
            );

            SetWindowLongPtr(hwndDlg, DWLP_USER as ::core::ffi::c_int, pUserData as INT_PTR);

            GetUserMembershipData(hwndDlg, pUserData);
        }

        WM_COMMAND => {
            match (LOWORD!(wParam)) as u32 {
                IDC_USER_MEMBERSHIP_ADD => {
                    AddGroupToUser(hwndDlg, pUserData);
                }

                IDC_USER_MEMBERSHIP_REMOVE => {
                    RemoveGroupFromUser(hwndDlg, pUserData);
                }

                _ => {}
            }
        }

        WM_NOTIFY => {
            if (*(lParam as LPPSHNOTIFY)).hdr.code == (PSN_APPLY) as u32 {
                return TRUE as INT_PTR;
            } else {
                return OnUserPropSheetNotify(hwndDlg, pUserData, lParam) as INT_PTR;
            }
        }

        WM_DESTROY => {
            if !(*pUserData).pGroupData.is_null() {
                NetApiBufferFree((*pUserData).pGroupData as PVOID);
            }

            HeapFree(
                GetProcessHeap(),
                0 as DWORD,
                pUserData as PVOID,
            );
        }

        _ => {}
    }

    return FALSE as INT_PTR;
}

unsafe extern "C" fn UpdateUserOptions(
    hwndDlg: HWND,
    pUserData: PGENERAL_USER_DATA,
    bInit: BOOL,
) {
    EnableWindow(
        GetDlgItem(hwndDlg, (IDC_USER_GENERAL_CANNOT_CHANGE as i32)),
        ((*pUserData).dwPasswordExpired == 0) as ::core::ffi::c_int,
    );
    EnableWindow(
        GetDlgItem(hwndDlg, (IDC_USER_GENERAL_NEVER_EXPIRES as i32)),
        ((*pUserData).dwPasswordExpired == 0) as ::core::ffi::c_int,
    );
    EnableWindow(
        GetDlgItem(hwndDlg, (IDC_USER_GENERAL_FORCE_CHANGE as i32)),
        (((*pUserData).dwFlags & (UF_PASSWD_CANT_CHANGE | UF_DONT_EXPIRE_PASSWD)) == 0)
            as ::core::ffi::c_int,
    );

    EnableWindow(
        GetDlgItem(hwndDlg, (IDC_USER_GENERAL_LOCKED as i32)),
        (((*pUserData).dwFlags & UF_LOCKOUT) != 0) as ::core::ffi::c_int,
    );

    if bInit != 0 {
        CheckDlgButton(
            hwndDlg,
            (IDC_USER_GENERAL_FORCE_CHANGE as i32),
            (if (*pUserData).dwPasswordExpired != 0 {
                BST_CHECKED
            } else {
                BST_UNCHECKED
            }) as UINT,
        );

        CheckDlgButton(
            hwndDlg,
            (IDC_USER_GENERAL_CANNOT_CHANGE as i32),
            (if (*pUserData).dwFlags & UF_PASSWD_CANT_CHANGE != 0 {
                BST_CHECKED
            } else {
                BST_UNCHECKED
            }) as UINT,
        );

        CheckDlgButton(
            hwndDlg,
            (IDC_USER_GENERAL_NEVER_EXPIRES as i32),
            (if (*pUserData).dwFlags & UF_DONT_EXPIRE_PASSWD != 0 {
                BST_CHECKED
            } else {
                BST_UNCHECKED
            }) as UINT,
        );

        CheckDlgButton(
            hwndDlg,
            (IDC_USER_GENERAL_DISABLED as i32),
            (if (*pUserData).dwFlags & UF_ACCOUNTDISABLE != 0 {
                BST_CHECKED
            } else {
                BST_UNCHECKED
            }) as UINT,
        );

        CheckDlgButton(
            hwndDlg,
            (IDC_USER_GENERAL_LOCKED as i32),
            (if (*pUserData).dwFlags & UF_LOCKOUT != 0 {
                BST_CHECKED
            } else {
                BST_UNCHECKED
            }) as UINT,
        );
    }
}

unsafe extern "C" fn GetUserGeneralData(hwndDlg: HWND, pUserData: PGENERAL_USER_DATA) {
    let mut pUserInfo: PUSER_INFO_3 = core::ptr::null_mut();

    SetDlgItemText(hwndDlg, (IDC_USER_GENERAL_NAME as i32), core::ptr::addr_of_mut!((*pUserData).szUserName) as *mut TCHAR as LPCWSTR);

    NetUserGetInfo(core::ptr::null::<WCHAR>(), core::ptr::addr_of_mut!((*pUserData).szUserName) as *mut TCHAR as LPCWSTR, 3, core::ptr::addr_of_mut!(pUserInfo) as *mut LPBYTE);

    SetDlgItemText(hwndDlg, (IDC_USER_GENERAL_FULL_NAME as i32), (*pUserInfo).usri3_full_name as LPCWSTR);
    SetDlgItemText(hwndDlg, (IDC_USER_GENERAL_DESCRIPTION as i32), (*pUserInfo).usri3_comment as LPCWSTR);

    (*pUserData).dwFlags = (*pUserInfo).usri3_flags;
    (*pUserData).dwPasswordExpired = (*pUserInfo).usri3_password_expired;

    NetApiBufferFree(pUserInfo as PVOID);

    UpdateUserOptions(hwndDlg, pUserData, (TRUE) as BOOL);
}

unsafe extern "C" fn SetUserGeneralData(hwndDlg: HWND, pUserData: PGENERAL_USER_DATA) -> BOOL {
    let mut pUserInfo: PUSER_INFO_3 = core::ptr::null_mut();
    let mut status: DWORD = 0;
    let mut dwIndex: DWORD = 0;

    NetUserGetInfo(
        core::ptr::null(),
        core::ptr::addr_of_mut!((*pUserData).szUserName) as *mut TCHAR as LPCWSTR,
        3,
        core::ptr::addr_of_mut!(pUserInfo) as *mut LPBYTE,
    );

    (*pUserInfo).usri3_flags = ((*pUserData).dwFlags & VALID_GENERAL_FLAGS)
        | ((*pUserInfo).usri3_flags & !VALID_GENERAL_FLAGS);

    (*pUserInfo).usri3_password_expired = (*pUserData).dwPasswordExpired;

    /* Get full name */
    (*pUserInfo).usri3_full_name =
        GetDlgItemTextAlloc(hwndDlg, (IDC_USER_GENERAL_FULL_NAME) as INT) as LPWSTR;

    /* Get desciption */
    (*pUserInfo).usri3_comment =
        GetDlgItemTextAlloc(hwndDlg, (IDC_USER_GENERAL_DESCRIPTION) as INT) as LPWSTR;

    status = NetUserSetInfo(
        core::ptr::null(),
        core::ptr::addr_of_mut!((*pUserData).szUserName) as *mut TCHAR as LPCWSTR,
        3,
        pUserInfo as LPBYTE,
        core::ptr::addr_of_mut!(dwIndex),
    );
    if status != NERR_Success {
    }

    HeapFree(
        GetProcessHeap(),
        0,
        (*pUserInfo).usri3_full_name as PVOID,
    );
    HeapFree(
        GetProcessHeap(),
        0,
        (*pUserInfo).usri3_comment as PVOID,
    );

    NetApiBufferFree(pUserInfo as PVOID);

    return (status == NERR_Success) as BOOL;
}
#[no_mangle]
pub unsafe extern "system" fn UserGeneralPageProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut pUserData: PGENERAL_USER_DATA;

    UNREFERENCED_PARAMETER!(lParam);
    UNREFERENCED_PARAMETER!(wParam);
    UNREFERENCED_PARAMETER!(hwndDlg);

    pUserData = GetWindowLongPtr(hwndDlg, DWLP_USER) as PGENERAL_USER_DATA;

    match uMsg {
        WM_INITDIALOG => {
            pUserData = HeapAlloc(
                GetProcessHeap(),
                HEAP_ZERO_MEMORY as DWORD,
                (core::mem::size_of::<GENERAL_USER_DATA>() as SIZE_T).wrapping_add(
                    (lstrlen((*(lParam as *mut PROPSHEETPAGE)).lParam as LPTSTR)
                        as SIZE_T)
                        .wrapping_mul((core::mem::size_of::<TCHAR>()) as SIZE_T),
                ),
            ) as PGENERAL_USER_DATA;
            lstrcpy(
                &raw mut (*pUserData).szUserName as LPTSTR,
                (*(lParam as *mut PROPSHEETPAGE)).lParam as LPTSTR,
            );

            SetWindowLongPtr(hwndDlg, DWLP_USER, pUserData as INT_PTR);

            GetUserGeneralData(hwndDlg, pUserData);
            // break
        }

        WM_COMMAND => {
            match (LOWORD!(wParam)) as u32 {
                IDC_USER_GENERAL_FULL_NAME | IDC_USER_GENERAL_DESCRIPTION => {
                    if (HIWORD!(wParam)) as u32 == EN_CHANGE {
                        PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                    }
                    // break
                }

                IDC_USER_GENERAL_FORCE_CHANGE => {
                    (*pUserData).dwPasswordExpired =
                        ((*pUserData).dwPasswordExpired == 0) as ::core::ffi::c_int as DWORD;
                    UpdateUserOptions(hwndDlg, pUserData, (FALSE) as BOOL);
                    PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                    // break
                }

                IDC_USER_GENERAL_CANNOT_CHANGE => {
                    (*pUserData).dwFlags ^= UF_PASSWD_CANT_CHANGE;
                    UpdateUserOptions(hwndDlg, pUserData, (FALSE) as BOOL);
                    PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                    // break
                }

                IDC_USER_GENERAL_NEVER_EXPIRES => {
                    (*pUserData).dwFlags ^= UF_DONT_EXPIRE_PASSWD;
                    UpdateUserOptions(hwndDlg, pUserData, (FALSE) as BOOL);
                    PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                    // break
                }

                IDC_USER_GENERAL_DISABLED => {
                    (*pUserData).dwFlags ^= UF_ACCOUNTDISABLE;
                    PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                    // break
                }

                IDC_USER_GENERAL_LOCKED => {
                    (*pUserData).dwFlags ^= UF_LOCKOUT;
                    PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                    // break
                }

                _ => {}
            }
            // break
        }

        WM_NOTIFY => {
            if (*(lParam as LPPSHNOTIFY)).hdr.code == (PSN_APPLY) as u32 {
                SetUserGeneralData(hwndDlg, pUserData);
                return TRUE as INT_PTR;
            }
            // break
        }

        WM_DESTROY => {
            HeapFree(
                GetProcessHeap(),
                0 as DWORD,
                pUserData as PVOID,
            );
            // break
        }

        _ => {}
    }

    return FALSE as INT_PTR;
}

unsafe extern "C" fn InitUserPropSheetPage(
    psp: *mut PROPSHEETPAGEW,
    idDlg: WORD,
    DlgProc: DLGPROC,
    pszUser: LPTSTR,
) {
    ZeroMemory!(psp as *mut core::ffi::c_void, core::mem::size_of::<PROPSHEETPAGE>() as usize);
    (*psp).dwSize = core::mem::size_of::<PROPSHEETPAGE>() as DWORD;
    (*psp).dwFlags = PSP_DEFAULT as DWORD;
    (*psp).hInstance = hApplet;
    (*psp).__bindgen_anon_1.pszTemplate = MAKEINTRESOURCE!(idDlg) as LPCWSTR;
    (*psp).pfnDlgProc = DlgProc;
    (*psp).lParam = pszUser as LPARAM;
}
#[no_mangle]
pub unsafe extern "C" fn UserProperties(hwndDlg: HWND) -> BOOL {
    let mut psp: [PROPSHEETPAGE; 3] = core::mem::zeroed();
    let mut psh: PROPSHEETHEADER = core::mem::zeroed();
    let mut szUserName: [TCHAR; (UNLEN + 1) as _] = core::mem::zeroed();
    let mut nItem: INT;
    let mut hwndLV: HWND;

    hwndLV = GetDlgItem(hwndDlg, IDC_USERS_LIST as INT);
    nItem = ListView_GetNextItem!(hwndLV, (-(1)) as INT, LVNI_SELECTED as UINT) as INT;
    if nItem == (-(1)) as INT {
        return FALSE as BOOL;
    }

    /* Get the new user name */
    ListView_GetItemText!(
        hwndLV,
        nItem,
        0 as INT,
        szUserName.as_mut_ptr(),
        (UNLEN + 1) as INT,
    );

    ZeroMemory!(
        core::ptr::addr_of_mut!(psh) as *mut core::ffi::c_void,
        core::mem::size_of::<PROPSHEETHEADER>(),
    );
    psh.dwSize = core::mem::size_of::<PROPSHEETHEADER>() as DWORD;
    psh.dwFlags = (PSH_PROPSHEETPAGE | PSH_PROPTITLE) as DWORD;
    psh.hwndParent = hwndDlg;
    psh.hInstance = hApplet;
    psh.__bindgen_anon_1.hIcon = core::ptr::null_mut();
    psh.pszCaption = szUserName.as_mut_ptr();
    psh.nPages = (core::mem::size_of::<[PROPSHEETPAGE; 3]>()
        / core::mem::size_of::<PROPSHEETPAGE>()) as UINT;
    psh.__bindgen_anon_2.nStartPage = 0 as UINT;
    psh.__bindgen_anon_3.ppsp = psp.as_mut_ptr() as LPCPROPSHEETPAGE;

    InitUserPropSheetPage(
        core::ptr::addr_of_mut!(psp[0 as usize]),
        IDD_USER_GENERAL as WORD,
        Some(UserGeneralPageProc),
        szUserName.as_mut_ptr(),
    );
    InitUserPropSheetPage(
        core::ptr::addr_of_mut!(psp[1 as usize]),
        IDD_USER_MEMBERSHIP as WORD,
        Some(UserMembershipPageProc),
        szUserName.as_mut_ptr(),
    );
    InitUserPropSheetPage(
        core::ptr::addr_of_mut!(psp[2 as usize]),
        IDD_USER_PROFILE as WORD,
        Some(UserProfilePageProc),
        szUserName.as_mut_ptr(),
    );

    return (PropertySheet(core::ptr::addr_of_mut!(psh))
        == IDOK as INT_PTR) as BOOL;
}
