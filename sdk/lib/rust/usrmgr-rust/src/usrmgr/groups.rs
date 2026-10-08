use crate::usrmgr::misc::CheckAccountName;
use crate::usrmgr::usrmgr::GetDlgItemTextAlloc;
use crate::usrmgr::groupprops::GroupProperties;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct GROUP_DATA {
    pub hPopupMenu: HMENU,
    pub iCurrentItem: INT,
}

pub type PGROUP_DATA = *mut GROUP_DATA;

unsafe extern "C" fn SetGroupsListColumns(hwndListView: HWND) {
    let mut column: LV_COLUMN = core::mem::zeroed();
    let mut rect: RECT = core::mem::zeroed();
    let mut szStr: [TCHAR; 32] = [0; 32];

    GetClientRect(hwndListView, core::ptr::addr_of_mut!(rect));

    memset(
        core::ptr::addr_of_mut!(column) as *mut core::ffi::c_void,
        0x00 as core::ffi::c_int,
        core::mem::size_of::<LV_COLUMN>() as usize,
    );
    column.mask = (LVCF_FMT | LVCF_WIDTH | LVCF_SUBITEM | LVCF_TEXT) as u32;
    column.fmt = LVCFMT_LEFT as i32;
    column.cx = ((rect.right - rect.left) as f64 * 0.40) as i32;
    column.iSubItem = 0 as i32;
    LoadString(
        hApplet,
        IDS_NAME as u32,
        core::ptr::addr_of_mut!(szStr) as *mut TCHAR,
        (core::mem::size_of::<[TCHAR; 32]>() / core::mem::size_of_val(&szStr[0])) as i32,
    );
    column.pszText = core::ptr::addr_of_mut!(szStr) as *mut TCHAR;
    let _ = ListView_InsertColumn!(hwndListView, 0 as i32, core::ptr::addr_of_mut!(column));

    column.cx = ((rect.right - rect.left) as f64 * 0.60) as i32;
    column.iSubItem = 1 as i32;
    LoadString(
        hApplet,
        IDS_DESCRIPTION as u32,
        core::ptr::addr_of_mut!(szStr) as *mut TCHAR,
        (core::mem::size_of::<[TCHAR; 32]>() / core::mem::size_of_val(&szStr[0])) as i32,
    );
    column.pszText = core::ptr::addr_of_mut!(szStr) as *mut TCHAR;
    let _ = ListView_InsertColumn!(hwndListView, 1 as i32, core::ptr::addr_of_mut!(column));
}

unsafe extern "C" fn UpdateGroupsList(hwndListView: HWND) {
    let mut netStatus: NET_API_STATUS;
    let mut pBuffer: PLOCALGROUP_INFO_1 = core::ptr::null_mut();
    let mut entriesread: DWORD = core::mem::zeroed();
    let mut totalentries: DWORD = core::mem::zeroed();
    let mut resume_handle: DWORD_PTR = 0;
    let mut i: DWORD;
    let mut lvi: LV_ITEM = core::mem::zeroed();
    let mut iItem: INT;

    loop {
        netStatus = NetLocalGroupEnum(
            core::ptr::null(),
            1,
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
                core::ptr::addr_of_mut!(lvi) as *mut ::core::ffi::c_void,
                0x00 as ::core::ffi::c_int,
                core::mem::size_of::<LV_ITEM>(),
            );
            lvi.mask = (LVIF_TEXT | LVIF_STATE | LVIF_IMAGE) as UINT;
            lvi.pszText = (*pBuffer.add(i as usize)).lgrpi1_name;
            lvi.state = 0 as UINT;
            lvi.iImage = 0 as ::core::ffi::c_int;
            iItem = ListView_InsertItem!(hwndListView, core::ptr::addr_of_mut!(lvi));

            ListView_SetItemText!(
                hwndListView,
                iItem,
                1 as ::core::ffi::c_int,
                (*pBuffer.add(i as usize)).lgrpi1_comment,
            );
            i += 1;
        }

        NetApiBufferFree(pBuffer as PVOID);

        /* No more data left */
        if netStatus != ERROR_MORE_DATA {
            break;
        }
    }
}

unsafe extern "C" fn UpdateGroupProperties(hwndDlg: HWND) {
    let mut szGroupName: [TCHAR; (UNLEN + 1) as _] = [0; (UNLEN + 1) as _];
    let mut iItem: INT;
    let mut hwndLV: HWND;
    let mut pGroupInfo: PLOCALGROUP_INFO_1 = core::ptr::null_mut();

    hwndLV = GetDlgItem(hwndDlg, (IDC_GROUPS_LIST as i32));
    iItem = ListView_GetNextItem!(hwndLV, -1, LVNI_SELECTED) as INT;
    if iItem == -1 {
        return;
    }

    /* Get the group name */
    ListView_GetItemText!(
        hwndLV,
        iItem,
        0,
        szGroupName.as_mut_ptr(),
        UNLEN + 1,
    );

    NetLocalGroupGetInfo(
        core::ptr::null_mut(),
        szGroupName.as_mut_ptr(),
        1,
        core::ptr::addr_of_mut!(pGroupInfo) as *mut LPBYTE,
    );

    ListView_SetItemText!(
        hwndLV,
        iItem,
        1,
        (*pGroupInfo).lgrpi1_comment,
    );

    NetApiBufferFree(pGroupInfo as PVOID);
}
#[no_mangle]
pub unsafe extern "system" fn NewGroupDlgProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut groupInfo: PLOCALGROUP_INFO_1 = core::ptr::null_mut();
    let mut nLength: INT = 0;

    UNREFERENCED_PARAMETER!(wParam);

    groupInfo = GetWindowLongPtr(hwndDlg, DWLP_USER) as PLOCALGROUP_INFO_1;

    match uMsg {
        WM_INITDIALOG => {
            SetWindowLongPtr(hwndDlg, DWLP_USER, lParam);
            groupInfo = lParam as PLOCALGROUP_INFO_1;
            SendDlgItemMessage(hwndDlg, (IDC_GROUP_NEW_NAME as i32), EM_SETLIMITTEXT, 20, 0);
        }

        WM_COMMAND => {
            'inner: {
                match (LOWORD!(wParam) as INT) as u32 {
                    IDC_GROUP_NEW_NAME => {
                        if (HIWORD!(wParam)) as u32 == EN_CHANGE {
                            nLength = SendDlgItemMessage(hwndDlg, (IDC_GROUP_NEW_NAME as i32), WM_GETTEXTLENGTH, 0, 0) as INT;
                            EnableWindow(
                                GetDlgItem(hwndDlg, (IDOK) as i32),
                                (nLength > 0) as INT,
                            );
                        }
                        break 'inner;
                    }

                    IDOK => {
                        if CheckAccountName(hwndDlg, (IDC_GROUP_NEW_NAME) as INT, core::ptr::null_mut()) == 0 {
                            SetFocus(GetDlgItem(hwndDlg, (IDC_GROUP_NEW_NAME as i32)));
                            SendDlgItemMessage(hwndDlg, (IDC_GROUP_NEW_NAME as i32), EM_SETSEL, 0, -1);
                            break 'inner;
                        }

                        /* Get Name */
                        (*groupInfo).lgrpi1_name = GetDlgItemTextAlloc(hwndDlg, (IDC_GROUP_NEW_NAME) as INT) as LPWSTR;

                        /* Get Description */
                        (*groupInfo).lgrpi1_comment = GetDlgItemTextAlloc(hwndDlg, (IDC_GROUP_NEW_DESCRIPTION) as INT) as LPWSTR;

                        EndDialog(hwndDlg, (IDOK) as INT_PTR);
                        break 'inner;
                    }

                    IDCANCEL => {
                        EndDialog(hwndDlg, (IDCANCEL) as INT_PTR);
                        break 'inner;
                    }

                    _ => {}
                }
            }
        }

        _ => {
            return FALSE as INT_PTR;
        }
    }

    return TRUE as INT_PTR;
}

unsafe extern "C" fn GroupNew(hwndDlg: HWND) {
    let mut status: NET_API_STATUS = 0;
    let mut group: LOCALGROUP_INFO_1 = core::mem::zeroed();
    let mut lvi: LV_ITEM = core::mem::zeroed();
    let mut iItem: INT = 0;
    let mut hwndLV: HWND = core::ptr::null_mut();

    ZeroMemory!(
        core::ptr::addr_of_mut!(group) as *mut core::ffi::c_void,
        core::mem::size_of::<LOCALGROUP_INFO_1>(),
    );

    if DialogBoxParam(
        hApplet,
        MAKEINTRESOURCE!(IDD_GROUP_NEW),
        hwndDlg,
        Some(NewGroupDlgProc),
        core::ptr::addr_of_mut!(group) as LPARAM,
    ) == IDOK as INT_PTR
    {
        status = NetLocalGroupAdd(
            core::ptr::null(),
            1,
            core::ptr::addr_of_mut!(group) as LPBYTE,
            core::ptr::null_mut(),
        );
        if status != NERR_Success {
            let mut szText: [TCHAR; 256] = [0; 256];
            let fmtError: [u16; 10] = [
                'E' as u16, 'r' as u16, 'r' as u16, 'o' as u16, 'r' as u16,
                ':' as u16, ' ' as u16, '%' as u16, 'u' as u16, 0,
            ];
            wsprintf(szText.as_mut_ptr(), fmtError.as_ptr(), status);
            let caption: [u16; 11] = [
                'N' as u16, 'e' as u16, 't' as u16, 'U' as u16, 's' as u16,
                'e' as u16, 'r' as u16, 'A' as u16, 'd' as u16, 'd' as u16, 0,
            ];
            MessageBox(
                core::ptr::null_mut(),
                szText.as_ptr() as LPCWSTR,
                caption.as_ptr() as LPCWSTR,
                MB_ICONERROR | MB_OK,
            );
            return;
        }

        hwndLV = GetDlgItem(hwndDlg, (IDC_GROUPS_LIST as i32));

        ZeroMemory!(
            core::ptr::addr_of_mut!(lvi) as *mut core::ffi::c_void,
            core::mem::size_of::<LV_ITEM>(),
        );
        lvi.mask = LVIF_TEXT | LVIF_STATE | LVIF_IMAGE;
        lvi.pszText = group.lgrpi1_name;
        lvi.state = 0;
        lvi.iImage = 0;
        iItem = ListView_InsertItem!(hwndLV, core::ptr::addr_of_mut!(lvi));

        ListView_SetItemText!(
            hwndLV,
            iItem,
            1,
            group.lgrpi1_comment,
        );
    }

    HeapFree(
        GetProcessHeap(),
        0,
        group.lgrpi1_name as PVOID,
    );
    HeapFree(
        GetProcessHeap(),
        0,
        group.lgrpi1_comment as PVOID,
    );
}

unsafe extern "C" fn GroupRename(mut hwndDlg: HWND) {
    let mut nItem: INT;
    let mut hwndLV: HWND;
    hwndLV = GetDlgItem(hwndDlg, (IDC_GROUPS_LIST as i32));
    nItem = ListView_GetNextItem!(hwndLV, -1, LVNI_SELECTED);
    if nItem != -1 {
        let _ = ListView_EditLabel!(hwndLV, nItem);
    }
}

unsafe extern "C" fn GroupDelete(hwndDlg: HWND) -> BOOL {
    let mut szGroupName: [TCHAR; (UNLEN + 1) as _] = [0; (UNLEN + 1) as _];
    let mut szText: [TCHAR; 256] = [0; 256];
    let mut nItem: INT = 0;
    let mut hwndLV: HWND = core::ptr::null_mut();
    let mut status: NET_API_STATUS;

    hwndLV = GetDlgItem(hwndDlg, (IDC_GROUPS_LIST as i32));
    nItem = ListView_GetNextItem!(hwndLV, -1, LVNI_SELECTED);
    if nItem == -1 {
        return (FALSE) as BOOL;
    }

    /* Get the new group name */
    ListView_GetItemText!(
        hwndLV,
        nItem,
        0,
        core::ptr::addr_of_mut!(szGroupName) as *mut TCHAR,
        UNLEN + 1,
    );

    /* Display a warning message, because the delete operation cannot be reverted */
    let fmt_delete: [u16; 51] = [
        'D' as u16, 'o' as u16, 'u' as u16, ' ' as u16, 'y' as u16, 'o' as u16, 'u' as u16, ' ' as u16,
        'r' as u16, 'e' as u16, 'a' as u16, 'l' as u16, 'l' as u16, 'y' as u16, ' ' as u16, 'w' as u16,
        'a' as u16, 'n' as u16, 't' as u16, ' ' as u16, 't' as u16, 'o' as u16, ' ' as u16, 'd' as u16,
        'e' as u16, 'l' as u16, 'e' as u16, 't' as u16, 'e' as u16, ' ' as u16, 't' as u16, 'h' as u16,
        'e' as u16, ' ' as u16, 'u' as u16, 's' as u16, 'e' as u16, 'r' as u16, ' ' as u16, 'g' as u16,
        'r' as u16, 'o' as u16, 'u' as u16, 'p' as u16, ' ' as u16, '"' as u16, '%' as u16, 's' as u16,
        '"' as u16, '?' as u16, 0,
    ];
    wsprintf(
        core::ptr::addr_of_mut!(szText) as *mut TCHAR,
        core::ptr::addr_of!(fmt_delete) as *const TCHAR,
        core::ptr::addr_of_mut!(szGroupName) as *mut TCHAR,
    );
    let title_user_groups: [u16; 12] = [
        'U' as u16, 's' as u16, 'e' as u16, 'r' as u16, ' ' as u16, 'G' as u16,
        'r' as u16, 'o' as u16, 'u' as u16, 'p' as u16, 's' as u16, 0,
    ];
    if (MessageBox(
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(szText) as *mut TCHAR,
        core::ptr::addr_of!(title_user_groups) as *const TCHAR,
        MB_ICONWARNING | MB_YESNO,
    )) as u32 == IDNO
    {
        return (FALSE) as BOOL;
    }

    /* Delete the group */
    status = NetLocalGroupDel(core::ptr::null(), core::ptr::addr_of_mut!(szGroupName) as *mut TCHAR);
    if status != NERR_Success {
        let mut szText: [TCHAR; 256] = [0; 256];
        let fmt_error: [u16; 10] = [
            'E' as u16, 'r' as u16, 'r' as u16, 'o' as u16, 'r' as u16, ':' as u16,
            ' ' as u16, '%' as u16, 'u' as u16, 0,
        ];
        wsprintf(
            core::ptr::addr_of_mut!(szText) as *mut TCHAR,
            core::ptr::addr_of!(fmt_error) as *const TCHAR,
            status,
        );
        let title_netlocal: [u16; 17] = [
            'N' as u16, 'e' as u16, 't' as u16, 'L' as u16, 'o' as u16, 'c' as u16,
            'a' as u16, 'l' as u16, 'G' as u16, 'r' as u16, 'o' as u16, 'u' as u16,
            'p' as u16, 'D' as u16, 'e' as u16, 'l' as u16, 0,
        ];
        MessageBox(
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(szText) as *mut TCHAR,
            core::ptr::addr_of!(title_netlocal) as *const TCHAR,
            MB_ICONERROR | MB_OK,
        );
        return (FALSE) as BOOL;
    }

    /* Delete the group from the list */
    let _ = ListView_DeleteItem!(hwndLV, nItem);

    return (TRUE) as BOOL;
}

unsafe extern "C" fn OnGroupsPageInitDialog(hwndDlg: HWND) {
    let mut hwndListView: HWND = core::ptr::null_mut();
    let mut hImgList: HIMAGELIST = core::ptr::null_mut();
    let mut hIcon: HICON = core::ptr::null_mut();

    /* Create the image list */
    hImgList = ImageList_Create(16, 16, ILC_COLOR32 | ILC_MASK, 5, 5);
    hIcon = LoadImage(hApplet, MAKEINTRESOURCE!(IDI_GROUP) as LPCWSTR, IMAGE_ICON, 16, 16, LR_DEFAULTCOLOR) as HICON;
    ImageList_AddIcon!(hImgList, hIcon);
    DestroyIcon(hIcon);

    hwndListView = GetDlgItem(hwndDlg, (IDC_GROUPS_LIST as i32));

    let _ = ListView_SetImageList!(hwndListView, hImgList, LVSIL_SMALL);

    let _ = ListView_SetExtendedListViewStyle!(hwndListView, LVS_EX_FULLROWSELECT);

    SetGroupsListColumns(hwndListView);

    UpdateGroupsList(hwndListView);
}

unsafe extern "C" fn OnGroupsPageBeginLabelEdit(pnmv: LPNMLVDISPINFOW) -> BOOL {
    let mut hwndEdit: HWND;
    hwndEdit = ListView_GetEditControl!((*pnmv).hdr.hwndFrom) as HWND;
    if hwndEdit.is_null() {
        return (TRUE) as BOOL;
    }
    SendMessage(hwndEdit, EM_SETLIMITTEXT as UINT, 20 as WPARAM, 0 as LPARAM);
    return (FALSE) as BOOL;
}

unsafe extern "C" fn OnGroupsPageEndLabelEdit(pnmv: LPNMLVDISPINFO) -> BOOL {
    let mut szOldGroupName: [TCHAR; (UNLEN + 1) as _] = [0; (UNLEN + 1) as _];
    let mut szNewGroupName: [TCHAR; (UNLEN + 1) as _] = [0; (UNLEN + 1) as _];
    let mut lgrpi0: LOCALGROUP_INFO_0 = _LOCALGROUP_INFO_0 {
        lgrpi0_name: ::core::ptr::null_mut::<WCHAR>(),
    };
    let mut status: NET_API_STATUS = 0;

    /* Leave, if there is no valid listview item */
    if (*pnmv).item.iItem == -1 {
        return (FALSE) as BOOL;
    }

    /* Get the new user name */
    ListView_GetItemText!((*pnmv).hdr.hwndFrom,
                         (*pnmv).item.iItem, 0,
                         &raw mut szOldGroupName as LPTSTR,
                         UNLEN + 1);

    /* Leave, if the user canceled the edit action */
    if (*pnmv).item.pszText.is_null() {
        return (FALSE) as BOOL;
    }

    /* Get the new user name */
    lstrcpy(&raw mut szNewGroupName as LPTSTR, (*pnmv).item.pszText);

    /* Leave, if the user name was not changed */
    if lstrcmp(&raw mut szOldGroupName as LPTSTR, &raw mut szNewGroupName as LPTSTR) == 0 {
        return (FALSE) as BOOL;
    }

    /* Check the group name for illegal characters */
    if CheckAccountName(::core::ptr::null_mut::<HWND__>(), 0, &raw mut szNewGroupName as LPTSTR) == 0 {
        return (FALSE) as BOOL;
    }

    /* Change the user name */
    lgrpi0.lgrpi0_name = &raw mut szNewGroupName as *mut TCHAR as LPWSTR;

    status = NetLocalGroupSetInfo(::core::ptr::null_mut::<WCHAR>(),
                                    &raw mut szOldGroupName as *mut TCHAR as LPCWSTR,
                                    0,
                                    &raw mut lgrpi0 as LPBYTE,
                                    ::core::ptr::null_mut::<DWORD>());
    if status != NERR_Success {
        let mut szText: [TCHAR; 256] = [0; 256];
        wsprintf(&raw mut szText as LPTSTR,
                               (['E' as u16, 'r' as u16, 'r' as u16, 'o' as u16, 'r' as u16, ':' as u16, ' ' as u16, '%' as u16, 'u' as u16, 0].as_ptr() as LPCWSTR),
                               status);
        MessageBox(::core::ptr::null_mut::<HWND__>(),
                               &raw mut szText as *mut TCHAR as LPCWSTR,
                               (['N' as u16, 'e' as u16, 't' as u16, 'L' as u16, 'o' as u16, 'c' as u16, 'a' as u16, 'l' as u16, 'G' as u16, 'r' as u16, 'o' as u16, 'u' as u16, 'p' as u16, 'S' as u16, 'e' as u16, 't' as u16, 'I' as u16, 'n' as u16, 'f' as u16, 'o' as u16, 0].as_ptr() as LPCWSTR),
                               MB_ICONERROR | MB_OK);
        return (FALSE) as BOOL;
    }

    /* Update the listview item */
    ListView_SetItemText!((*pnmv).hdr.hwndFrom,
                         (*pnmv).item.iItem, 0,
                         &raw mut szNewGroupName as LPTSTR);

    return (TRUE) as BOOL;
}

unsafe extern "C" fn OnGroupsPageNotify(hwndDlg: HWND, pGroupData: PGROUP_DATA, phdr: *mut NMHDR) -> BOOL {
    let lpnmlv: LPNMLISTVIEW = phdr as LPNMLISTVIEW;

    match (*phdr).idFrom {
        x if x == IDC_GROUPS_LIST as UINT_PTR => match (*phdr).code {
            NM_CLICK => {
                (*pGroupData).iCurrentItem = (*lpnmlv).iItem as INT;
            }
            NM_DBLCLK => {
                if (*lpnmlv).iItem != -(1 as i32) {
                    let mut uItem: UINT = 0;

                    uItem = GetMenuDefaultItem(
                        GetSubMenu((*pGroupData).hPopupMenu, 1 as i32),
                        (FALSE as BOOL) as UINT,
                        0 as UINT,
                    );
                    if uItem != ((-(1 as i32)) as UINT) {
                        SendMessage(
                            hwndDlg,
                            WM_COMMAND as UINT,
                            MAKEWPARAM!(uItem as WORD, 0 as WORD),
                            0 as LPARAM,
                        );
                    }
                }
            }
            NM_RCLICK => {
                ClientToScreen(
                    GetDlgItem(hwndDlg, IDC_GROUPS_LIST as i32),
                    core::ptr::addr_of_mut!((*lpnmlv).ptAction),
                );
                TrackPopupMenu(
                    GetSubMenu(
                        (*pGroupData).hPopupMenu,
                        if (*lpnmlv).iItem == -(1 as i32) {
                            0 as i32
                        } else {
                            1 as i32
                        },
                    ),
                    TPM_LEFTALIGN as UINT,
                    (*lpnmlv).ptAction.x as i32,
                    (*lpnmlv).ptAction.y as i32,
                    0 as i32,
                    hwndDlg,
                    core::ptr::null::<RECT>(),
                );
            }
            LVN_BEGINLABELEDIT => {
                return OnGroupsPageBeginLabelEdit(phdr as LPNMLVDISPINFO);
            }
            LVN_ENDLABELEDIT => {
                return OnGroupsPageEndLabelEdit(phdr as LPNMLVDISPINFO);
            }
            _ => {}
        },
        _ => {}
    }

    return (FALSE) as BOOL;
}
#[no_mangle]
pub unsafe extern "system" fn GroupsPageProc(hwndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let mut pGroupData: PGROUP_DATA = core::ptr::null_mut();

    UNREFERENCED_PARAMETER!(lParam);
    UNREFERENCED_PARAMETER!(wParam);
    UNREFERENCED_PARAMETER!(hwndDlg);

    pGroupData = GetWindowLongPtr(hwndDlg, DWLP_USER) as PGROUP_DATA;

    match uMsg {
        WM_INITDIALOG => {
            pGroupData = HeapAlloc(
                GetProcessHeap(),
                HEAP_ZERO_MEMORY as DWORD,
                core::mem::size_of::<GROUP_DATA>() as SIZE_T,
            ) as PGROUP_DATA;
            SetWindowLongPtr(
                hwndDlg,
                DWLP_USER as ::core::ffi::c_int,
                pGroupData as LONG_PTR,
            );

            (*pGroupData).hPopupMenu = LoadMenu(
                hApplet,
                MAKEINTRESOURCE!(IDM_POPUP_GROUP),
            );

            OnGroupsPageInitDialog(hwndDlg);
            SetMenuDefaultItem(
                GetSubMenu((*pGroupData).hPopupMenu, 1 as ::core::ffi::c_int),
                IDM_GROUP_PROPERTIES as UINT,
                FALSE as UINT,
            );
            // break;
        }

        WM_COMMAND => {
            match (LOWORD!(wParam) as ::core::ffi::c_int) as u32 {
                IDM_GROUP_NEW | IDC_GROUPS_ADD => {
                    GroupNew(hwndDlg);
                }

                IDM_GROUP_RENAME => {
                    GroupRename(hwndDlg);
                }

                IDM_GROUP_DELETE | IDC_GROUPS_REMOVE => {
                    GroupDelete(hwndDlg);
                }

                IDM_GROUP_PROPERTIES | IDC_GROUPS_PROPERTIES => {
                    if (GroupProperties(hwndDlg)) as u32 == IDOK {
                        UpdateGroupProperties(hwndDlg);
                    }
                }

                _ => {}
            }
            // break;
        }

        WM_NOTIFY => {
            return OnGroupsPageNotify(hwndDlg, pGroupData, lParam as *mut NMHDR) as INT_PTR;
        }

        WM_DESTROY => {
            DestroyMenu((*pGroupData).hPopupMenu);
            HeapFree(
                GetProcessHeap(),
                0 as DWORD,
                pGroupData as PVOID,
            );
            // break;
        }

        _ => {}
    }

    return FALSE as INT_PTR;
}
