use crate::usrmgr::usrmgr::GetDlgItemTextAlloc;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct _GENERAL_GROUP_DATA {
    pub szGroupName: [TCHAR; 1],
}

pub type GENERAL_GROUP_DATA = _GENERAL_GROUP_DATA;
pub type PGENERAL_GROUP_DATA = *mut _GENERAL_GROUP_DATA;

unsafe extern "C" fn GetTextSid(pSid: PSID, pTextSid: LPTSTR) {
    let mut psia: PSID_IDENTIFIER_AUTHORITY;
    let mut dwSubAuthorities: DWORD;
    let mut dwSidRev: DWORD = SID_REVISION;
    let mut dwCounter: DWORD;
    let mut dwSidSize: DWORD;

    static FMT_SID: [u16; 7] = ['S' as u16, '-' as u16, '%' as u16, 'l' as u16, 'u' as u16, '-' as u16, 0];
    static FMT_HEX: [u16; 33] = [
        '0' as u16, 'x' as u16,
        '%' as u16, '0' as u16, '2' as u16, 'h' as u16, 'x' as u16,
        '%' as u16, '0' as u16, '2' as u16, 'h' as u16, 'x' as u16,
        '%' as u16, '0' as u16, '2' as u16, 'h' as u16, 'x' as u16,
        '%' as u16, '0' as u16, '2' as u16, 'h' as u16, 'x' as u16,
        '%' as u16, '0' as u16, '2' as u16, 'h' as u16, 'x' as u16,
        '%' as u16, '0' as u16, '2' as u16, 'h' as u16, 'x' as u16,
        0
    ];
    static FMT_ULONG: [u16; 4] = ['%' as u16, 'l' as u16, 'u' as u16, 0];
    static FMT_SUB: [u16; 5] = ['-' as u16, '%' as u16, 'l' as u16, 'u' as u16, 0];

    psia = GetSidIdentifierAuthority(pSid);

    dwSubAuthorities = (*GetSidSubAuthorityCount(pSid)) as _;

    dwSidSize = wsprintf(pTextSid, FMT_SID.as_ptr(), dwSidRev) as DWORD;

    if ((*psia).Value[0] != 0) || ((*psia).Value[1] != 0)
    {
        dwSidSize = dwSidSize.wrapping_add(
            wsprintf(
                pTextSid.offset(lstrlen(pTextSid) as isize),
                FMT_HEX.as_ptr(),
                (*psia).Value[0] as USHORT as i32,
                (*psia).Value[1] as USHORT as i32,
                (*psia).Value[2] as USHORT as i32,
                (*psia).Value[3] as USHORT as i32,
                (*psia).Value[4] as USHORT as i32,
                (*psia).Value[5] as USHORT as i32,
            ) as DWORD,
        );
    }
    else
    {
        dwSidSize = dwSidSize.wrapping_add(
            wsprintf(
                pTextSid.offset(lstrlen(pTextSid) as isize),
                FMT_ULONG.as_ptr(),
                ((*psia).Value[5] as ULONG)
                    .wrapping_add(((*psia).Value[4] as ULONG) << 8)
                    .wrapping_add(((*psia).Value[3] as ULONG) << 16)
                    .wrapping_add(((*psia).Value[2] as ULONG) << 24),
            ) as DWORD,
        );
    }

    dwCounter = 0;
    while dwCounter < dwSubAuthorities
    {
        dwSidSize = dwSidSize.wrapping_add(
            wsprintf(
                pTextSid.offset(dwSidSize as isize),
                FMT_SUB.as_ptr(),
                *GetSidSubAuthority(pSid, dwCounter),
            ) as DWORD,
        );
        dwCounter = dwCounter.wrapping_add(1);
    }
}

unsafe extern "C" fn InitGroupMembersList(hwndDlg: HWND, pGroupData: PGENERAL_GROUP_DATA) {
    let mut hwndLV: HWND = core::ptr::null_mut();
    let mut column: LV_COLUMN = core::mem::zeroed();
    let mut rect: RECT = core::mem::zeroed();
    let mut szStr: [TCHAR; 32] = [0; 32];
    let mut hImgList: HIMAGELIST = core::ptr::null_mut();
    let mut hIcon: HICON = core::ptr::null_mut();

    let mut netStatus: NET_API_STATUS = 0;
    let mut pUserBuffer: PUSER_INFO_20 = core::ptr::null_mut();
    let mut entriesread: DWORD = 0;
    let mut totalentries: DWORD = 0;
    let mut resume_handle: DWORD = 0;
    let mut i: DWORD = 0;
    let mut lvi: LV_ITEM = core::mem::zeroed();
    let mut iItem: INT = 0;

    hwndLV = GetDlgItem(hwndDlg, (IDC_USER_ADD_MEMBERSHIP_LIST as i32));
    GetClientRect(hwndLV, core::ptr::addr_of_mut!(rect));

    hImgList = ImageList_Create(16, 16, ILC_COLOR32 | ILC_MASK, 5, 5);
    hIcon = LoadImage(hApplet, MAKEINTRESOURCE!(IDI_GROUP), IMAGE_ICON, 16, 16, LR_DEFAULTCOLOR) as HICON;
    ImageList_AddIcon!(hImgList, hIcon);
    DestroyIcon(hIcon);
    hIcon = LoadImage(hApplet, MAKEINTRESOURCE!(IDI_USER), IMAGE_ICON, 16, 16, LR_DEFAULTCOLOR) as HICON;
    ImageList_AddIcon!(hImgList, hIcon);
    DestroyIcon(hIcon);
    hIcon = LoadImage(hApplet, MAKEINTRESOURCE!(IDI_LOCKED_USER), IMAGE_ICON, 16, 16, LR_DEFAULTCOLOR) as HICON;
    ImageList_AddIcon!(hImgList, hIcon);
    DestroyIcon(hIcon);

    let _ = ListView_SetImageList!(hwndLV, hImgList, LVSIL_SMALL);
    let _ = ListView_SetExtendedListViewStyle!(hwndLV, LVS_EX_FULLROWSELECT);

    memset(core::ptr::addr_of_mut!(column) as *mut core::ffi::c_void, 0x00, core::mem::size_of::<LV_COLUMN>());
    column.mask = LVCF_FMT | LVCF_WIDTH | LVCF_SUBITEM | LVCF_TEXT;
    column.fmt = (LVCFMT_LEFT as i32);
    column.cx = ((rect.right - rect.left) as f64 * 0.40) as INT;
    column.iSubItem = 0;
    LoadString(hApplet, IDS_NAME, szStr.as_mut_ptr(), (core::mem::size_of::<[TCHAR; 32]>() / core::mem::size_of_val(&szStr[0])) as INT);
    column.pszText = szStr.as_mut_ptr();
    let _ = ListView_InsertColumn!(hwndLV, 0, core::ptr::addr_of_mut!(column));

    column.cx = ((rect.right - rect.left) as f64 * 0.60) as INT;
    column.iSubItem = 1;
    LoadString(hApplet, IDS_DESCRIPTION, szStr.as_mut_ptr(), (core::mem::size_of::<[TCHAR; 32]>() / core::mem::size_of_val(&szStr[0])) as INT);
    column.pszText = szStr.as_mut_ptr();
    let _ = ListView_InsertColumn!(hwndLV, 1, core::ptr::addr_of_mut!(column));

    /* TODO: Enumerate global groups and add them to the list! */

    loop {
        netStatus = NetUserEnum(core::ptr::null_mut(), 20, FILTER_NORMAL_ACCOUNT,
                                core::ptr::addr_of_mut!(pUserBuffer) as *mut LPBYTE,
                                1024, core::ptr::addr_of_mut!(entriesread),
                                core::ptr::addr_of_mut!(totalentries), core::ptr::addr_of_mut!(resume_handle));
        if netStatus != NERR_Success && netStatus != ERROR_MORE_DATA {
            break;
        }

        i = 0;
        while i < entriesread {
            memset(core::ptr::addr_of_mut!(lvi) as *mut core::ffi::c_void, 0x00, core::mem::size_of::<LV_ITEM>());
            lvi.mask = LVIF_TEXT | LVIF_STATE | LVIF_IMAGE;
            lvi.pszText = (*pUserBuffer.offset(i as isize)).usri20_name;
            lvi.state = 0;
            lvi.iImage = if ((*pUserBuffer.offset(i as isize)).usri20_flags & UF_ACCOUNTDISABLE) != 0 {
                2
            } else {
                1
            };
            iItem = ListView_InsertItem!(hwndLV, core::ptr::addr_of_mut!(lvi));

            let _ = ListView_SetItemText!(hwndLV, iItem, 1,
                                (*pUserBuffer.offset(i as isize)).usri20_full_name);

            let _ = ListView_SetItemText!(hwndLV, iItem, 2,
                                (*pUserBuffer.offset(i as isize)).usri20_comment);

            i = i.wrapping_add(1);
        }

        NetApiBufferFree(pUserBuffer as PVOID);

        /* No more data left */
        if netStatus != ERROR_MORE_DATA {
            break;
        }
    }
}

unsafe extern "C" fn AddSelectedUsersToGroup(hwndDlg: HWND, pGroupData: PGENERAL_GROUP_DATA) -> BOOL {
    let mut hwndLV: HWND;
    let mut nSelectedItems: INT;
    let mut nItem: INT;
    let mut szUserName: [TCHAR; (UNLEN + 1) as _] =
        [0; (UNLEN + 1) as _];
    let mut bResult: BOOL = (FALSE) as i32;
    let mut memberInfo: LOCALGROUP_MEMBERS_INFO_3 = LOCALGROUP_MEMBERS_INFO_3 {
        lgrmi3_domainandname: core::ptr::null_mut(),
    };
    let mut status: DWORD;

    hwndLV = GetDlgItem(hwndDlg, (IDC_USER_ADD_MEMBERSHIP_LIST as i32));

    nSelectedItems = ListView_GetSelectedCount!(hwndLV) as INT;
    if nSelectedItems > 0 {
        nItem = ListView_GetNextItem!(hwndLV, -1, LVNI_SELECTED) as INT;
        while nItem != -1 {
            /* Get the new user name */
            ListView_GetItemText!(
                hwndLV,
                nItem,
                0,
                szUserName.as_mut_ptr(),
                UNLEN + 1,
            );

            memberInfo.lgrmi3_domainandname = szUserName.as_mut_ptr();

            status = NetLocalGroupAddMembers(
                core::ptr::null_mut(),
                (*pGroupData).szGroupName.as_mut_ptr(),
                3,
                core::ptr::addr_of_mut!(memberInfo) as LPBYTE,
                1,
            );
            if status != NERR_Success && status != ERROR_MEMBER_IN_ALIAS {
                let mut szText: [TCHAR; 256] = [0; 256];
                let fmt: [u16; 10] = [
                    'E' as u16, 'r' as u16, 'r' as u16, 'o' as u16, 'r' as u16,
                    ':' as u16, ' ' as u16, '%' as u16, 'u' as u16, 0,
                ];
                wsprintf(szText.as_mut_ptr(), fmt.as_ptr(), status);
                let caption: [u16; 24] = [
                    'N' as u16, 'e' as u16, 't' as u16, 'L' as u16, 'o' as u16, 'c' as u16,
                    'a' as u16, 'l' as u16, 'G' as u16, 'r' as u16, 'o' as u16, 'u' as u16,
                    'p' as u16, 'A' as u16, 'd' as u16, 'd' as u16, 'M' as u16, 'e' as u16,
                    'm' as u16, 'b' as u16, 'e' as u16, 'r' as u16, 's' as u16, 0,
                ];
                MessageBox(
                    core::ptr::null_mut(),
                    szText.as_ptr(),
                    caption.as_ptr(),
                    MB_ICONERROR | MB_OK,
                );
            } else {
                bResult = (TRUE) as i32;
            }

            nItem = ListView_GetNextItem!(hwndLV, nItem, LVNI_SELECTED) as INT;
        }
    }

    return bResult;
}
#[no_mangle]
pub unsafe extern "system" fn AddUsersToGroupDlgProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut pGroupData: PGENERAL_GROUP_DATA = core::ptr::null_mut();

    UNREFERENCED_PARAMETER!(wParam);

    pGroupData = GetWindowLongPtr(hwndDlg, DWLP_USER) as PGENERAL_GROUP_DATA;

    match (uMsg as i32) as u32 {
        WM_INITDIALOG => {
            pGroupData = lParam as PGENERAL_GROUP_DATA;
            SetWindowLongPtr(hwndDlg, DWLP_USER, pGroupData as INT_PTR);
            InitGroupMembersList(hwndDlg, pGroupData);
        }
        WM_COMMAND => {
            match (LOWORD!(wParam) as ::core::ffi::c_int) as u32 {
                IDOK => {
                    if AddSelectedUsersToGroup(hwndDlg, pGroupData) != 0 {
                        EndDialog(hwndDlg, (IDOK) as INT_PTR);
                    } else {
                        EndDialog(hwndDlg, (IDCANCEL) as INT_PTR);
                    }
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

unsafe extern "C" fn AddUsersToGroup(hwndDlg: HWND, pGroupData: PGENERAL_GROUP_DATA) {
    let mut hwndLV: HWND = core::ptr::null_mut();
    let mut membersInfo: PLOCALGROUP_MEMBERS_INFO_1 = core::ptr::null_mut();
    let mut dwRead: DWORD = 0;
    let mut dwTotal: DWORD = 0;
    let mut resumeHandle: DWORD_PTR = 0;
    let mut i: DWORD = 0;
    let mut lvi: LV_ITEM = core::mem::zeroed();
    let mut szGroupName: [TCHAR; 256] = [0; 256];

    if DialogBoxParam(
        hApplet,
        MAKEINTRESOURCE!(IDD_USER_ADD_MEMBERSHIP) as LPCWSTR,
        hwndDlg,
        Some(AddUsersToGroupDlgProc),
        pGroupData as LPARAM,
    ) == IDOK as INT_PTR
    {
        hwndLV = GetDlgItem(hwndDlg, (IDC_GROUP_GENERAL_MEMBERS as i32));

        let _ = ListView_DeleteAllItems!(hwndLV);

        /* Set group members */
        NetLocalGroupGetMembers(
            core::ptr::null::<WCHAR>(),
            core::ptr::addr_of_mut!((*pGroupData).szGroupName) as LPCWSTR,
            1 as DWORD,
            core::ptr::addr_of_mut!(membersInfo) as *mut LPBYTE,
            MAX_PREFERRED_LENGTH as DWORD,
            core::ptr::addr_of_mut!(dwRead),
            core::ptr::addr_of_mut!(dwTotal),
            core::ptr::addr_of_mut!(resumeHandle),
        );

        i = 0 as DWORD;
        while i < dwRead {
            ZeroMemory!(
                core::ptr::addr_of_mut!(lvi) as *mut core::ffi::c_void,
                core::mem::size_of::<LV_ITEM>(),
            );
            lvi.mask = (LVIF_TEXT | LVIF_STATE | LVIF_IMAGE) as UINT;
            lvi.pszText = (*membersInfo.offset(i as isize)).lgrmi1_name;
            lvi.state = 0 as UINT;
            lvi.iImage = if (*membersInfo.offset(i as isize)).lgrmi1_sidusage as core::ffi::c_uint
                == SidTypeGroup as core::ffi::c_uint
                || (*membersInfo.offset(i as isize)).lgrmi1_sidusage as core::ffi::c_uint
                    == SidTypeWellKnownGroup as core::ffi::c_uint
            {
                1
            } else {
                0
            };

            if (*membersInfo.offset(i as isize)).lgrmi1_sidusage as core::ffi::c_uint
                == SidTypeWellKnownGroup as core::ffi::c_uint
            {
                let mut szSid: [TCHAR; 256] = [0; 256];

                GetTextSid(
                    (*membersInfo.offset(i as isize)).lgrmi1_sid,
                    core::ptr::addr_of_mut!(szSid) as LPTSTR,
                );

                // (&[0x0025, 0x0073, 0x0020, 0x0028, 0x0025, 0x0073, 0x0029, 0x0000u16]).as_ptr() /* (&[0x0025, 0x0073, 0x0020, 0x0028, 0x0025, 0x0073, 0x0029, 0x0000u16]).as_ptr() /* (&[0x0025, 0x0073, 0x0020, 0x0028, 0x0025, 0x0073, 0x0029, 0x0000u16]).as_ptr() /* (&[0x0025, 0x0073, 0x0020, 0x0028, 0x0025, 0x0073, 0x0029, 0x0000u16]).as_ptr() /* ("%s (%s)") */ */ */ */ as an explicit NUL-terminated wide array
                let fmt: [u16; 8] = [
                    '%' as u16,
                    's' as u16,
                    ' ' as u16,
                    '(' as u16,
                    '%' as u16,
                    's' as u16,
                    ')' as u16,
                    0,
                ];
                wsprintf(
                    core::ptr::addr_of_mut!(szGroupName) as LPTSTR,
                    fmt.as_ptr() as LPCWSTR,
                    (*membersInfo.offset(i as isize)).lgrmi1_name,
                    core::ptr::addr_of_mut!(szSid) as LPTSTR,
                );

                lvi.pszText = core::ptr::addr_of_mut!(szGroupName) as LPWSTR;
            }

            let _ = ListView_InsertItem!(hwndLV, core::ptr::addr_of_mut!(lvi));
            i = i.wrapping_add(1);
        }

        NetApiBufferFree(membersInfo as PVOID);
    }
}

unsafe extern "C" fn RemoveUserFromGroup(mut hwndDlg: HWND, mut pGroupData: PGENERAL_GROUP_DATA) {
    let mut szUserName: [TCHAR; (UNLEN + 1) as _] = [0; (UNLEN + 1) as _];
    let mut szText: [TCHAR; 256] = [0; 256];
    let mut memberInfo: LOCALGROUP_MEMBERS_INFO_3 = _LOCALGROUP_MEMBERS_INFO_3 {
        lgrmi3_domainandname: ::core::ptr::null_mut::<WCHAR>(),
    };
    let mut hwndLV: HWND = ::core::ptr::null_mut::<HWND__>();
    let mut nItem: INT = 0;
    let mut status: DWORD = 0;

    hwndLV = GetDlgItem(hwndDlg, (IDC_GROUP_GENERAL_MEMBERS as i32));
    nItem = ListView_GetNextItem!(hwndLV, -1, LVNI_SELECTED);
    if nItem == -1 {
        return;
    }

    /* Get the new user name */
    ListView_GetItemText!(
        hwndLV,
        nItem,
        0,
        &raw mut szUserName as *mut TCHAR,
        UNLEN + 1,
    );

    /* Display a warning message because the remove operation cannot be reverted */
    let fmt1 = [
        'D' as u16, 'o' as u16, ' ' as u16, 'y' as u16, 'o' as u16, 'u' as u16, ' ' as u16,
        'r' as u16, 'e' as u16, 'a' as u16, 'l' as u16, 'l' as u16, 'y' as u16, ' ' as u16,
        'w' as u16, 'a' as u16, 'n' as u16, 't' as u16, ' ' as u16, 't' as u16, 'o' as u16, ' ' as u16,
        'r' as u16, 'e' as u16, 'm' as u16, 'o' as u16, 'v' as u16, 'e' as u16, ' ' as u16,
        't' as u16, 'h' as u16, 'e' as u16, ' ' as u16, 'u' as u16, 's' as u16, 'e' as u16, 'r' as u16, ' ' as u16,
        '"' as u16, '%' as u16, 's' as u16, '"' as u16, ' ' as u16,
        'f' as u16, 'r' as u16, 'o' as u16, 'm' as u16, ' ' as u16,
        't' as u16, 'h' as u16, 'e' as u16, ' ' as u16,
        'g' as u16, 'r' as u16, 'o' as u16, 'u' as u16, 'p' as u16, ' ' as u16,
        '"' as u16, '%' as u16, 's' as u16, '"' as u16, '?' as u16, 0u16,
    ];
    wsprintf(
        &raw mut szText as *mut TCHAR,
        fmt1.as_ptr() as LPCWSTR,
        &raw mut szUserName as *mut TCHAR,
        &raw mut (*pGroupData).szGroupName as *mut TCHAR,
    );
    let caption1 = [
        'U' as u16, 's' as u16, 'e' as u16, 'r' as u16, ' ' as u16,
        'A' as u16, 'c' as u16, 'c' as u16, 'o' as u16, 'u' as u16, 'n' as u16, 't' as u16, 's' as u16, 0u16,
    ];
    if (MessageBox(
        ::core::ptr::null_mut::<HWND__>(),
        &raw mut szText as *mut TCHAR as LPCWSTR,
        caption1.as_ptr() as LPCWSTR,
        (MB_ICONWARNING | MB_YESNO) as UINT,
    )) as u32 == IDNO
    {
        return;
    }

    memberInfo.lgrmi3_domainandname = &raw mut szUserName as *mut TCHAR as LPWSTR;

    status = NetLocalGroupDelMembers(
        ::core::ptr::null::<WCHAR>(),
        &raw mut (*pGroupData).szGroupName as *mut TCHAR as LPCWSTR,
        3 as DWORD,
        &raw mut memberInfo as LPBYTE,
        1 as DWORD,
    );
    if status != NERR_Success as DWORD {
        let mut szText: [TCHAR; 256] = [0; 256];
        let fmt2 = [
            'E' as u16, 'r' as u16, 'r' as u16, 'o' as u16, 'r' as u16, ':' as u16, ' ' as u16,
            '%' as u16, 'u' as u16, 0u16,
        ];
        wsprintf(
            &raw mut szText as *mut TCHAR,
            fmt2.as_ptr() as LPCWSTR,
            status,
        );
        let caption2 = [
            'N' as u16, 'e' as u16, 't' as u16, 'L' as u16, 'o' as u16, 'c' as u16, 'a' as u16, 'l' as u16,
            'G' as u16, 'r' as u16, 'o' as u16, 'u' as u16, 'p' as u16,
            'D' as u16, 'e' as u16, 'l' as u16, 'M' as u16, 'e' as u16, 'm' as u16, 'b' as u16, 'e' as u16,
            'r' as u16, 's' as u16, 0u16,
        ];
        MessageBox(
            ::core::ptr::null_mut::<HWND__>(),
            &raw mut szText as *mut TCHAR as LPCWSTR,
            caption2.as_ptr() as LPCWSTR,
            (MB_ICONERROR | MB_OK) as UINT,
        );
        return;
    }

    let _ = ListView_DeleteItem!(hwndLV, nItem);

    if ListView_GetItemCount!(hwndLV) == 0 {
        EnableWindow(
            GetDlgItem(hwndDlg, (IDC_GROUP_GENERAL_REMOVE as i32)),
            (FALSE) as BOOL,
        );
    }
}

unsafe extern "C" fn OnGroupPropSheetNotify(
    mut hwndDlg: HWND,
    mut pGroupData: PGENERAL_GROUP_DATA,
    mut lParam: LPARAM,
) -> BOOL {
    let mut lpnmlv: LPNMLISTVIEW = lParam as LPNMLISTVIEW;
    match (*(lParam as LPNMHDR)).idFrom {
        x if x == (IDC_GROUP_GENERAL_MEMBERS as UINT_PTR) => {
            match (((*(lParam as LPNMHDR)).code) as i32) as u32 {
                NM_CLICK => {
                    EnableWindow(
                        GetDlgItem(hwndDlg, (IDC_GROUP_GENERAL_REMOVE as i32)),
                        ((*lpnmlv).iItem != -1) as ::core::ffi::c_int,
                    );
                }
                x if x == (LVN_KEYDOWN as u32) => {
                    if ((*(lParam as LPNMLVKEYDOWN)).wVKey as ::core::ffi::c_int) as u32
                        == VK_DELETE
                    {
                        RemoveUserFromGroup(hwndDlg, pGroupData);
                    }
                }
                _ => {}
            }
        }
        _ => {}
    }

    return (FALSE) as BOOL;
}

unsafe extern "C" fn GetGeneralGroupData(hwndDlg: HWND, pGroupData: PGENERAL_GROUP_DATA) {
    let mut groupInfo: PLOCALGROUP_INFO_1 = core::ptr::null_mut();
    let mut membersInfo: PLOCALGROUP_MEMBERS_INFO_2 = core::ptr::null_mut();
    let mut dwRead: DWORD = core::mem::zeroed();
    let mut dwTotal: DWORD = core::mem::zeroed();
    let mut resumeHandle: DWORD_PTR = 0;
    let mut i: DWORD;
    let mut lvi: LV_ITEM = core::mem::zeroed();
    let mut hwndLV: HWND;
    let mut column: LV_COLUMN = core::mem::zeroed();
    let mut rect: RECT = core::mem::zeroed();
    let mut hImgList: HIMAGELIST;
    let mut hIcon: HICON;
    let mut szGroupName: [TCHAR; 256] = core::mem::zeroed();

    hwndLV = GetDlgItem(hwndDlg, (IDC_GROUP_GENERAL_MEMBERS as i32));

    /* Create the image list */
    hImgList = ImageList_Create(16, 16, ILC_COLOR32 | ILC_MASK, 5, 5);
    hIcon = (LoadImage(hApplet, MAKEINTRESOURCE!(IDI_GROUP), IMAGE_ICON, 16, 16, LR_DEFAULTCOLOR)).cast();
    ImageList_AddIcon!(hImgList, hIcon);
    DestroyIcon(hIcon);
    hIcon = (LoadImage(hApplet, MAKEINTRESOURCE!(IDI_USER), IMAGE_ICON, 16, 16, LR_DEFAULTCOLOR)).cast();
    ImageList_AddIcon!(hImgList, hIcon);
    DestroyIcon(hIcon);

    let _ = ListView_SetImageList!(hwndLV, hImgList, LVSIL_SMALL as usize);

    /* Set the list column */
    GetClientRect(hwndLV, core::ptr::addr_of_mut!(rect));

    memset(core::ptr::addr_of_mut!(column) as *mut core::ffi::c_void, 0x00, core::mem::size_of::<LV_COLUMN>());
    column.mask = LVCF_FMT | LVCF_WIDTH | LVCF_SUBITEM;
    column.fmt = (LVCFMT_LEFT as i32);
    column.cx = (rect.right - rect.left) as INT;
    column.iSubItem = 0;
    let _ = ListView_InsertColumn!(hwndLV, 0, core::ptr::addr_of!(column));

    /* Set group name */
    SetDlgItemText(hwndDlg, (IDC_GROUP_GENERAL_NAME as i32), core::ptr::addr_of_mut!((*pGroupData).szGroupName) as *mut TCHAR);

    /* Set group description */
    NetLocalGroupGetInfo(core::ptr::null_mut(), core::ptr::addr_of_mut!((*pGroupData).szGroupName) as *mut TCHAR, 1, core::ptr::addr_of_mut!(groupInfo) as *mut LPBYTE);
    SetDlgItemText(hwndDlg, (IDC_GROUP_GENERAL_DESCRIPTION as i32), (*groupInfo).lgrpi1_comment);
    NetApiBufferFree((groupInfo).cast());

    /* Set group members */
    NetLocalGroupGetMembers(core::ptr::null_mut(), core::ptr::addr_of_mut!((*pGroupData).szGroupName) as *mut TCHAR, 2, core::ptr::addr_of_mut!(membersInfo) as *mut LPBYTE,
                            MAX_PREFERRED_LENGTH, core::ptr::addr_of_mut!(dwRead), core::ptr::addr_of_mut!(dwTotal),
                            core::ptr::addr_of_mut!(resumeHandle));

    i = 0;
    while i < dwRead {
        ZeroMemory!(core::ptr::addr_of_mut!(lvi) as *mut core::ffi::c_void, core::mem::size_of::<LV_ITEM>());
        lvi.mask = LVIF_TEXT | LVIF_STATE | LVIF_IMAGE;
        lvi.state = 0;
        if (*membersInfo.offset(i as isize)).lgrmi2_sidusage == SidTypeGroup ||
            (*membersInfo.offset(i as isize)).lgrmi2_sidusage == SidTypeWellKnownGroup
        {
            lvi.iImage = 0;
        }
        else if (*membersInfo.offset(i as isize)).lgrmi2_sidusage == SidTypeUser
        {
            /* FIXME: handle locked user properly! */
            lvi.iImage = 1;
        }

        if (*membersInfo.offset(i as isize)).lgrmi2_sidusage == SidTypeWellKnownGroup
        {
            let mut szSid: [TCHAR; 256] = core::mem::zeroed();

            GetTextSid((*membersInfo.offset(i as isize)).lgrmi2_sid, core::ptr::addr_of_mut!(szSid) as *mut TCHAR);

            static TEXT_SID_FMT: [u16; 8] = ['%' as u16, 's' as u16, ' ' as u16, '(' as u16, '%' as u16, 's' as u16, ')' as u16, 0];
            wsprintf(core::ptr::addr_of_mut!(szGroupName) as *mut TCHAR,
                     core::ptr::addr_of!(TEXT_SID_FMT) as *const u16,
                     (*membersInfo.offset(i as isize)).lgrmi2_domainandname,
                     core::ptr::addr_of_mut!(szSid) as *mut TCHAR);

            lvi.pszText = core::ptr::addr_of_mut!(szGroupName) as *mut TCHAR;
        }
        else
        {
            let mut ptr: LPWSTR;

            ptr = wcschr((*membersInfo.offset(i as isize)).lgrmi2_domainandname, 0x005Cu16) as LPWSTR;
            if !ptr.is_null()
            {
                ptr = ptr.offset(1);
                lvi.pszText = ptr;
            }
            else
            {
                lvi.pszText = (*membersInfo.offset(i as isize)).lgrmi2_domainandname;
            }
        }

        let _ = ListView_InsertItem!(hwndLV, core::ptr::addr_of!(lvi));
        i += 1;
    }

    NetApiBufferFree((membersInfo).cast());
}

unsafe extern "C" fn SetGeneralGroupData(
    mut hwndDlg: HWND,
    mut pGroupData: PGENERAL_GROUP_DATA,
) -> BOOL {
    let mut groupInfo: LOCALGROUP_INFO_1 = _LOCALGROUP_INFO_1 {
        lgrpi1_name: ::core::ptr::null_mut::<WCHAR>(),
        lgrpi1_comment: ::core::ptr::null_mut::<WCHAR>(),
    };
    let mut status: DWORD = 0;
    let mut dwIndex: DWORD = 0;
    groupInfo.lgrpi1_comment =
        GetDlgItemTextAlloc(hwndDlg, (IDC_GROUP_GENERAL_DESCRIPTION) as INT) as LPWSTR;
    status = NetLocalGroupSetInfo(
        ::core::ptr::null::<WCHAR>(),
        core::ptr::addr_of_mut!((*pGroupData).szGroupName) as *mut TCHAR as LPCWSTR,
        1 as DWORD,
        core::ptr::addr_of_mut!(groupInfo) as PBYTE,
        core::ptr::addr_of_mut!(dwIndex),
    );
    if status != NERR_Success as DWORD {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/cpl/usrmgr/groupprops.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"SetGeneralGroupData\0" as *const u8 as *const ::core::ffi::c_char,
                506 as ::core::ffi::c_int,
                b"NetLocalGroupSetInfo failed. Status: %lu  Index: %lu\0" as *const u8
                    as *const ::core::ffi::c_char,
                status,
                dwIndex,
            );
        }
    }
    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        groupInfo.lgrpi1_comment as PVOID,
    );
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "system" fn GroupGeneralPageProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut pGroupData: PGENERAL_GROUP_DATA;

    UNREFERENCED_PARAMETER!(lParam);
    UNREFERENCED_PARAMETER!(wParam);
    UNREFERENCED_PARAMETER!(hwndDlg);

    pGroupData = GetWindowLongPtr(hwndDlg, DWLP_USER) as PGENERAL_GROUP_DATA;

    match uMsg {
        WM_INITDIALOG => {
            pGroupData = HeapAlloc(
                GetProcessHeap(),
                HEAP_ZERO_MEMORY,
                (core::mem::size_of::<GENERAL_GROUP_DATA>()
                    + (lstrlen((*(lParam as *mut PROPSHEETPAGE)).lParam as LPTSTR) as usize)
                        * core::mem::size_of::<TCHAR>()) as SIZE_T,
            ) as PGENERAL_GROUP_DATA;
            lstrcpy(
                core::ptr::addr_of_mut!((*pGroupData).szGroupName) as LPTSTR,
                (*(lParam as *mut PROPSHEETPAGE)).lParam as LPTSTR,
            );

            SetWindowLongPtr(hwndDlg, DWLP_USER, pGroupData as INT_PTR);

            GetGeneralGroupData(hwndDlg, pGroupData);
        }

        WM_COMMAND => {
            match (LOWORD!(wParam)) as u32 {
                IDC_GROUP_GENERAL_DESCRIPTION => {
                    if (HIWORD!(wParam)) as u32 == EN_CHANGE {
                        PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                    }
                }

                IDC_GROUP_GENERAL_ADD => {
                    AddUsersToGroup(hwndDlg, pGroupData);
                }

                IDC_GROUP_GENERAL_REMOVE => {
                    RemoveUserFromGroup(hwndDlg, pGroupData);
                }

                _ => {}
            }
        }

        WM_NOTIFY => {
            if (*(lParam as LPPSHNOTIFY)).hdr.code == (PSN_APPLY) as u32 {
                SetGeneralGroupData(hwndDlg, pGroupData);
                return TRUE as INT_PTR;
            } else {
                return OnGroupPropSheetNotify(hwndDlg, pGroupData, lParam) as INT_PTR;
            }
        }

        WM_DESTROY => {
            HeapFree(GetProcessHeap(), 0, pGroupData as PVOID);
        }

        _ => {}
    }

    return FALSE as INT_PTR;
}

unsafe extern "C" fn InitGroupPropSheetPage(psp: *mut PROPSHEETPAGE, idDlg: WORD, DlgProc: DLGPROC, pszGroup: LPTSTR) {
    ZeroMemory!(psp as *mut core::ffi::c_void, core::mem::size_of::<PROPSHEETPAGE>());
    (*psp).dwSize = core::mem::size_of::<PROPSHEETPAGE>() as DWORD;
    (*psp).dwFlags = PSP_DEFAULT as DWORD;
    (*psp).hInstance = hApplet;
    (*psp).__bindgen_anon_1.pszTemplate = MAKEINTRESOURCE!(idDlg) as LPCWSTR;
    (*psp).pfnDlgProc = DlgProc;
    (*psp).lParam = pszGroup as LPARAM;
}
#[no_mangle]
pub unsafe extern "C" fn GroupProperties(hwndDlg: HWND) -> BOOL {
    let mut psp: [PROPSHEETPAGE; 1] = core::mem::zeroed();
    let mut psh: PROPSHEETHEADER = core::mem::zeroed();
    let mut szGroupName: [TCHAR; (UNLEN + 1) as _] = [0 as TCHAR; (UNLEN + 1) as _];
    let mut nItem: INT;
    let mut hwndLV: HWND;

    hwndLV = GetDlgItem(hwndDlg, IDC_GROUPS_LIST as ::core::ffi::c_int);
    nItem = ListView_GetNextItem!(hwndLV, -(1 as INT), LVNI_SELECTED as UINT) as INT;
    if nItem == -(1 as INT) {
        return FALSE as BOOL;
    }

    /* Get the new user name */
    ListView_GetItemText!(hwndLV,
                         nItem, 0 as INT,
                         szGroupName.as_mut_ptr(),
                         (UNLEN + 1) as INT);

    ZeroMemory!(&raw mut psh as *mut ::core::ffi::c_void, core::mem::size_of::<PROPSHEETHEADER>() as usize);
    psh.dwSize = core::mem::size_of::<PROPSHEETHEADER>() as DWORD;
    psh.dwFlags = (PSH_PROPSHEETPAGE | PSH_PROPTITLE) as DWORD;
    psh.hwndParent = hwndDlg;
    psh.hInstance = hApplet;
    psh.__bindgen_anon_1.hIcon = core::ptr::null_mut();
    psh.pszCaption = szGroupName.as_mut_ptr() as LPCWSTR;
    psh.nPages = (core::mem::size_of::<[PROPSHEETPAGE; 1]>()
        / core::mem::size_of::<PROPSHEETPAGE>() as usize) as UINT;
    psh.__bindgen_anon_2.nStartPage = 0 as UINT;
    psh.__bindgen_anon_3.ppsp = psp.as_mut_ptr() as LPCPROPSHEETPAGE;

    InitGroupPropSheetPage(
        psp.as_mut_ptr().add(0 as usize),
        IDD_GROUP_GENERAL as WORD,
        Some(GroupGeneralPageProc),
        szGroupName.as_mut_ptr(),
    );

    return (PropertySheet(&raw mut psh as LPCPROPSHEETHEADER) == IDOK as INT_PTR) as BOOL;
}
