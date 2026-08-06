// rustconv-assembly: whole-tu
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct _PROFILEDATA {
    pub dwRefCount: u32,
    pub dwState: u32,
    pub bUnknownProfile: i32,
    pub pszFullName: PWSTR,
    pub pszProfilePath: PWSTR,
}

pub type PROFILEDATA = _PROFILEDATA;
pub type PPROFILEDATA = *mut _PROFILEDATA;
unsafe extern "C" fn OnProfileTypeInit(
    hwndDlg: HWND,
    pProfileData: *mut PROFILEDATA,
) -> BOOL {
    let mut pszRawBuffer: PWSTR = core::ptr::null_mut();
    let mut pszCookedBuffer: PWSTR = core::ptr::null_mut();
    let mut nLength: i32;

    nLength = LoadStringW(hApplet, IDS_USERPROFILE_TYPE_TEXT, &mut pszRawBuffer as *mut _ as PWSTR, 0);
    pszRawBuffer = core::ptr::null_mut();
    if nLength == 0 {
        return FALSE;
    }

    pszRawBuffer = (HeapAlloc(GetProcessHeap(), 0, ((nLength + 1) * core::mem::size_of::<u16>() as i32) as SIZE_T)).cast();
    if pszRawBuffer.is_null() {
        return FALSE;
    }

    LoadStringW(hApplet, IDS_USERPROFILE_TYPE_TEXT, pszRawBuffer, nLength + 1);

    pszCookedBuffer = (HeapAlloc(GetProcessHeap(), 0, ((nLength + (wcslen((*pProfileData).pszFullName)) as i32 + 1) * core::mem::size_of::<u16>() as i32) as SIZE_T)).cast();
    if pszCookedBuffer.is_null() {
        'done: loop {
            if !pszCookedBuffer.is_null() {
                HeapFree(GetProcessHeap(), 0, pszCookedBuffer as LPVOID);
            }

            if !pszRawBuffer.is_null() {
                HeapFree(GetProcessHeap(), 0, pszRawBuffer as LPVOID);
            }

            return TRUE;
        }
    }

    swprintf(pszCookedBuffer, pszRawBuffer, (*pProfileData).pszFullName);

    SetDlgItemText(hwndDlg, (IDC_USERPROFILE_TYPE_TEXT) as i32, pszCookedBuffer);

    EnableWindow(GetDlgItem(hwndDlg, (IDC_USERPROFILE_TYPE_ROAMING) as i32), FALSE);
    Button_SetCheck(GetDlgItem(hwndDlg, (IDC_USERPROFILE_TYPE_LOCAL) as i32), BST_CHECKED);
    EnableWindow(GetDlgItem(hwndDlg, (IDOK) as i32), FALSE);

    if !pszCookedBuffer.is_null() {
        HeapFree(GetProcessHeap(), 0, pszCookedBuffer as LPVOID);
    }

    if !pszRawBuffer.is_null() {
        HeapFree(GetProcessHeap(), 0, pszRawBuffer as LPVOID);
    }

    TRUE
}
#[cfg(target_arch = "x86")]
unsafe extern "stdcall" fn UserProfileTypeDlgProc(
    hwndDlg: HWND,
    uMsg: u32,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    match uMsg {
        WM_INITDIALOG => {
            OnProfileTypeInit(hwndDlg, lParam as *mut PROFILEDATA);
            return TRUE;
        }

        WM_DESTROY => {}

        WM_COMMAND => {
            match LOWORD(wParam) {
                IDOK | IDCANCEL => {
                    EndDialog(hwndDlg, LOWORD(wParam));
                    return TRUE;
                }
                _ => {}
            }
        }

        _ => {}
    }

    FALSE
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn UserProfileTypeDlgProc(
    hwndDlg: HWND,
    uMsg: u32,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    match uMsg {
        WM_INITDIALOG => {
            OnProfileTypeInit(hwndDlg, lParam as *mut PROFILEDATA);
            return TRUE;
        }

        WM_DESTROY => {}

        WM_COMMAND => {
            match LOWORD(wParam) {
                IDOK | IDCANCEL => {
                    EndDialog(hwndDlg, LOWORD(wParam));
                    return TRUE;
                }
                _ => {}
            }
        }

        _ => {}
    }

    FALSE
}
unsafe extern "C" fn ChangeUserProfileType(
    hwndDlg: HWND) -> BOOL {
    let hwndListView: HWND;
    let mut Item: LVITEM = core::mem::zeroed();
    let iSelected: i32;

    // DPRINT removed

    hwndListView = GetDlgItem(hwndDlg, (IDC_USERPROFILE_LIST) as i32);
    if hwndListView == core::ptr::null_mut() {
        return FALSE;
    }

    iSelected = ListView_GetNextItem(hwndListView, -1, LVNI_SELECTED);
    if iSelected == -1 {
        return FALSE;
    }

    ZeroMemory(core::ptr::addr_of_mut!(Item) as LPVOID, core::mem::size_of::<LVITEM>());
    Item.mask = LVIF_PARAM;
    Item.iItem = iSelected;
    if ListView_GetItem(hwndListView, core::ptr::addr_of_mut!(Item)) == 0 {
        return FALSE;
    }

    if Item.lParam == 0 {
        return FALSE;
    }

    if DialogBoxParam(hApplet,
                       MAKEINTRESOURCE(IDD_USERPROFILE_TYPE),
                       hwndDlg,
                       UserProfileTypeDlgProc,
                       Item.lParam as LPARAM) == IDOK
    {
        return TRUE;
    }

    return FALSE;
}
unsafe extern "C" fn DeleteUserProfile(hwndDlg: HWND) -> BOOL {
    let hwndListView: HWND;
    let mut Item: LVITEM = core::mem::zeroed();
    let iSelected: i32;
    let pProfileData: *mut PROFILEDATA;

    hwndListView = GetDlgItem(hwndDlg, (IDC_USERPROFILE_LIST) as i32);
    if hwndListView.is_null() {
        return FALSE;
    }

    iSelected = ListView_GetNextItem(hwndListView, -1, LVNI_SELECTED);
    if iSelected == -1 {
        return FALSE;
    }

    ZeroMemory(
        core::ptr::addr_of_mut!(Item) as *mut core::ffi::c_void,
        core::mem::size_of::<LVITEM>(),
    );
    Item.mask = LVIF_PARAM;
    Item.iItem = iSelected;
    if ListView_GetItem(hwndListView, core::ptr::addr_of_mut!(Item)) == 0 {
        return FALSE;
    }

    if Item.lParam == 0 {
        return FALSE;
    }

    pProfileData = Item.lParam as *mut PROFILEDATA;
    if (*pProfileData).dwRefCount != 0 {
        return FALSE;
    }

    if ResourceMessageBox(
        hApplet,
        hwndDlg,
        MB_ICONQUESTION | MB_YESNO,
        IDS_USERPROFILE_CONFIRM_DELETE_TITLE,
        IDS_USERPROFILE_CONFIRM_DELETE,
        (*pProfileData).pszFullName,
    ) == (IDYES) as i32
    {
        return TRUE;
    }

    return FALSE;
}
#[cfg(target_arch = "x86")]
unsafe extern "stdcall" fn CopyUserProfileDlgProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    match uMsg {
        WM_INITDIALOG => {
            return TRUE;
        }

        WM_DESTROY => {}

        WM_COMMAND => {
            match LOWORD(wParam) {
                IDOK | IDCANCEL => {
                    EndDialog(hwndDlg, LOWORD(wParam));
                    return TRUE;
                }
                _ => {}
            }
        }

        _ => {}
    }

    FALSE
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn CopyUserProfileDlgProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    match uMsg {
        WM_INITDIALOG => {
            return TRUE;
        }

        WM_DESTROY => {}

        WM_COMMAND => {
            match LOWORD(wParam) {
                IDOK | IDCANCEL => {
                    EndDialog(hwndDlg, LOWORD(wParam));
                    return TRUE;
                }
                _ => {}
            }
        }

        _ => {}
    }

    FALSE
}
unsafe extern "C" fn CopyUserProfile(
    hwndDlg: HWND
) -> BOOL {
    let mut hwndListView: HWND;
    let mut Item: LVITEM = core::mem::zeroed();
    let mut iSelected: i32;

    // DPRINT removed

    hwndListView = GetDlgItem(hwndDlg, (IDC_USERPROFILE_LIST) as i32);
    if hwndListView.is_null() {
        return FALSE;
    }

    iSelected = ListView_GetNextItem(hwndListView, -1, LVNI_SELECTED);
    if iSelected == -1 {
        return FALSE;
    }

    ZeroMemory(
        core::ptr::addr_of_mut!(Item) as *mut core::ffi::c_void,
        core::mem::size_of::<LVITEM>()
    );
    Item.mask = LVIF_PARAM;
    Item.iItem = iSelected;
    if ListView_GetItem(hwndListView, core::ptr::addr_of_mut!(Item)) == 0 {
        return FALSE;
    }

    if Item.lParam == 0 {
        return FALSE;
    }

    if DialogBoxParam(
        hApplet,
        MAKEINTRESOURCE(IDD_USERPROFILE_COPY),
        hwndDlg,
        CopyUserProfileDlgProc,
        Item.lParam as LPARAM
    ) == IDOK {
        return TRUE;
    }

    return FALSE;
}
unsafe extern "C" fn SetListViewColumns(hwndListView: HWND) {
    let mut column: LV_COLUMN = core::mem::zeroed();
    let mut rect: RECT = core::mem::zeroed();
    let mut szStr: [TCHAR; 32] = core::mem::zeroed();

    GetClientRect(hwndListView, &mut rect);

    SendMessage(hwndListView, LVM_SETEXTENDEDLISTVIEWSTYLE, 0, LVS_EX_FULLROWSELECT);

    memset(&mut column as *mut _ as *mut core::ffi::c_void, 0x00, core::mem::size_of::<LV_COLUMN>());
    column.mask = LVCF_FMT | LVCF_WIDTH | LVCF_SUBITEM | LVCF_TEXT;
    column.fmt = (LVCFMT_LEFT) as i32;
    column.cx = ((rect.right - rect.left) * 0.40) as INT;
    column.iSubItem = 0;
    LoadString(hApplet, IDS_USERPROFILE_NAME, szStr.as_mut_ptr(), ARRAYSIZE(szStr.as_ptr()));
    column.pszText = szStr.as_mut_ptr();
    ListView_InsertColumn(hwndListView, 0, &mut column);

    column.fmt = (LVCFMT_RIGHT) as i32;
    column.cx = ((rect.right - rect.left) * 0.15) as INT;
    column.iSubItem = 1;
    LoadString(hApplet, IDS_USERPROFILE_SIZE, szStr.as_mut_ptr(), ARRAYSIZE(szStr.as_ptr()));
    column.pszText = szStr.as_mut_ptr();
    ListView_InsertColumn(hwndListView, 1, &mut column);

    column.fmt = (LVCFMT_LEFT) as i32;
    column.cx = ((rect.right - rect.left) * 0.15) as INT;
    column.iSubItem = 2;
    LoadString(hApplet, IDS_USERPROFILE_TYPE, szStr.as_mut_ptr(), ARRAYSIZE(szStr.as_ptr()));
    column.pszText = szStr.as_mut_ptr();
    ListView_InsertColumn(hwndListView, 2, &mut column);

    column.fmt = (LVCFMT_LEFT) as i32;
    column.cx = ((rect.right - rect.left) * 0.15) as INT;
    column.iSubItem = 3;
    LoadString(hApplet, IDS_USERPROFILE_STATUS, szStr.as_mut_ptr(), ARRAYSIZE(szStr.as_ptr()));
    column.pszText = szStr.as_mut_ptr();
    ListView_InsertColumn(hwndListView, 3, &mut column);

    column.fmt = (LVCFMT_LEFT) as i32;
    column.cx = ((rect.right - rect.left) * 0.15) as INT - GetSystemMetrics((SM_CYHSCROLL) as i32);
    column.iSubItem = 4;
    LoadString(hApplet, IDS_USERPROFILE_MODIFIED, szStr.as_mut_ptr(), ARRAYSIZE(szStr.as_ptr()));
    column.pszText = szStr.as_mut_ptr();
    ListView_InsertColumn(hwndListView, 4, &mut column);
}
unsafe extern "C" fn GetProfileSize(
    pszProfilePath: PWSTR,
    pullProfileSize: PULONGLONG,
) -> BOOL {
    let mut hFile = INVALID_HANDLE_VALUE;
    let mut FindData: WIN32_FIND_DATA = core::mem::zeroed();
    let dwProfilePathLength: u32;
    let mut Size: ULARGE_INTEGER = core::mem::zeroed();
    let mut bResult = TRUE;

    dwProfilePathLength = (wcslen(pszProfilePath)) as u32;

    wcscat(pszProfilePath, b"\\*.*\0".as_ptr() as *const u16);

    hFile = FindFirstFileW(pszProfilePath, &mut FindData);
    if hFile == INVALID_HANDLE_VALUE {
        if (GetLastError() != ERROR_FILE_NOT_FOUND) &&
            (GetLastError() != ERROR_PATH_NOT_FOUND) {
            bResult = FALSE;
        }

        *pszProfilePath.offset(dwProfilePathLength as isize) = UNICODE_NULL;

        if hFile != INVALID_HANDLE_VALUE {
            FindClose(hFile);
        }

        return bResult;
    }

    loop {
        if FindData.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY != 0 {
            if (_wcsicmp(FindData.cFileName.as_ptr(), b".\0".as_ptr() as *const u16) == 0) ||
                (_wcsicmp(FindData.cFileName.as_ptr(), b"..\0".as_ptr() as *const u16) == 0) {
                if FindNextFile(hFile, &mut FindData) == 0 {
                    break;
                }
                continue;
            }

            *pszProfilePath.offset((dwProfilePathLength + 1) as isize) = UNICODE_NULL;
            wcscat(pszProfilePath, FindData.cFileName.as_ptr());

            if GetProfileSize(pszProfilePath, pullProfileSize) == 0 {
                bResult = FALSE;

                *pszProfilePath.offset(dwProfilePathLength as isize) = UNICODE_NULL;

                if hFile != INVALID_HANDLE_VALUE {
                    FindClose(hFile);
                }

                return bResult;
            }
        } else {
            (*Size.u).__bindgen_anon_1.LowPart = FindData.nFileSizeLow;
            (*Size.u).__bindgen_anon_1.HighPart = FindData.nFileSizeHigh;
            *pullProfileSize = (*pullProfileSize).wrapping_add(*Size.QuadPart);
        }

        if FindNextFile(hFile, &mut FindData) == 0 {
            break;
        }
    }

    *pszProfilePath.offset(dwProfilePathLength as isize) = UNICODE_NULL;

    if hFile != INVALID_HANDLE_VALUE {
        FindClose(hFile);
    }

    bResult
}
unsafe extern "C" fn GetProfileName(
    pProfileSid: PSID,
    dwNameBufferSize: u32,
    pszNameBuffer: PWSTR,
    pbUnknownProfile: PBOOL,
) -> i32 {
    let mut szAccountName: [u16; 128] = [0; 128];
    let mut szDomainName: [u16; 128] = [0; 128];
    let mut dwAccountNameSize: u32;
    let mut dwDomainNameSize: u32;
    let mut Use: u32 = 0;

    dwAccountNameSize = ARRAYSIZE(szAccountName);
    dwDomainNameSize = ARRAYSIZE(szDomainName);
    if LookupAccountSidW(
        core::ptr::null_mut(),
        pProfileSid,
        szAccountName.as_mut_ptr(),
        &mut dwAccountNameSize,
        szDomainName.as_mut_ptr(),
        &mut dwDomainNameSize,
        &mut Use,
    ) == 0
    {
        LoadStringW(hApplet, IDS_USERPROFILE_ACCOUNT_UNKNOWN, pszNameBuffer, (dwNameBufferSize) as i32);
        *pbUnknownProfile = TRUE;
    } else {
        if Use != SidTypeUser {
            return FALSE;
        }

        if szAccountName[0] == UNICODE_NULL {
            LoadStringW(hApplet, IDS_USERPROFILE_ACCOUNT_DELETED, pszNameBuffer, (dwNameBufferSize) as i32);
        } else {
            wsprintf(pszNameBuffer, (b"L\"%s\\%s\"\0".as_ptr()).cast(), szDomainName.as_ptr(), szAccountName.as_ptr());
        }
        *pbUnknownProfile = FALSE;
    }

    TRUE
}
unsafe extern "C" fn AddUserProfile(
    hwndListView: HWND,
    pProfileSid: PSID,
    hProfileKey: HKEY,
) {
    let mut szTempProfilePath: [u16; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];
    let mut szProfilePath: [u16; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];
    let mut szNameBuffer: [u16; 256] = [0; 256];
    let mut pProfileData: *mut PROFILEDATA = core::ptr::null_mut();
    let mut dwProfileData: u32;
    let mut dwSize: u32;
    let mut dwType: u32;
    let mut dwState: u32 = 0;
    let mut dwRefCount: u32 = 0;
    let mut dwProfilePathLength: u32;
    let mut ptr: PWSTR;
    let mut nId: i32;
    let mut iItem: i32;
    let mut lvi: _LVITEMW = core::mem::zeroed();
    let mut FindData: WIN32_FIND_DATA = core::mem::zeroed();
    let mut hFile: *mut core::ffi::c_void;
    let mut SystemTime: SYSTEMTIME = core::mem::zeroed();
    let mut ullProfileSize: u64;
    let mut bUnknownProfile: u8;
    let mut dwError: u32;

    dwSize = MAX_PATH * core::mem::size_of::<u16>() as u32;
    dwError = (RegQueryValueExW(
        hProfileKey,
        ['P' as u16, 'r' as u16, 'o' as u16, 'f' as u16, 'i' as u16, 'l' as u16, 'e' as u16, 'I' as u16, 'm' as u16, 'a' as u16, 'g' as u16, 'e' as u16, 'P' as u16, 'a' as u16, 't' as u16, 'h' as u16, 0].as_ptr(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(dwType),
        szTempProfilePath.as_mut_ptr() as LPBYTE,
        core::ptr::addr_of_mut!(dwSize),
    )) as u32;
    if dwError != ERROR_SUCCESS {
        return;
    }

    ExpandEnvironmentStringsW(
        szTempProfilePath.as_ptr(),
        szProfilePath.as_mut_ptr(),
        MAX_PATH,
    );

    hFile = FindFirstFileW(szProfilePath.as_ptr(), core::ptr::addr_of_mut!(FindData));
    if hFile == INVALID_HANDLE_VALUE {
        return;
    }

    FindClose(hFile);

    dwProfilePathLength = wcslen(szProfilePath.as_ptr()) as u32;

    wcscat(
        szProfilePath.as_mut_ptr(),
        ['\\' as u16, 'n' as u16, 't' as u16, 'u' as u16, 's' as u16, 'e' as u16, 'r' as u16, '.' as u16, 'd' as u16, 'a' as u16, 't' as u16, 0].as_ptr(),
    );
    hFile = FindFirstFileW(szProfilePath.as_ptr(), core::ptr::addr_of_mut!(FindData));
    if hFile == INVALID_HANDLE_VALUE {
        return;
    }

    FindClose(hFile);
    szProfilePath[dwProfilePathLength as usize] = UNICODE_NULL;

    ullProfileSize = 0ULL;
    GetProfileSize(szProfilePath.as_mut_ptr(), core::ptr::addr_of_mut!(ullProfileSize));

    if GetProfileName(
        pProfileSid,
        ARRAYSIZE(szNameBuffer.as_ptr()),
        szNameBuffer.as_mut_ptr(),
        (core::ptr::addr_of_mut!(bUnknownProfile)).cast(),
    ) == 0
    {
        return;
    }

    dwSize = core::mem::size_of::<u32>() as u32;
    if RegQueryValueExW(
        hProfileKey,
        ['S' as u16, 't' as u16, 'a' as u16, 't' as u16, 'e' as u16, 0].as_ptr(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(dwType),
        core::ptr::addr_of_mut!(dwState) as LPBYTE,
        core::ptr::addr_of_mut!(dwSize),
    ) != (ERROR_SUCCESS) as i32
    {
        dwState = 0;
    }

    dwSize = core::mem::size_of::<u32>() as u32;
    if RegQueryValueExW(
        hProfileKey,
        ['R' as u16, 'e' as u16, 'f' as u16, 'C' as u16, 'o' as u16, 'u' as u16, 'n' as u16, 't' as u16, 0].as_ptr(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(dwType),
        core::ptr::addr_of_mut!(dwRefCount) as LPBYTE,
        core::ptr::addr_of_mut!(dwSize),
    ) != (ERROR_SUCCESS) as i32
    {
        dwRefCount = 0;
    }

    dwProfileData = core::mem::size_of::<PROFILEDATA>() as u32
        + ((wcslen(szNameBuffer.as_ptr()) as u32 + 1) * core::mem::size_of::<u16>() as u32)
        + ((wcslen(szProfilePath.as_ptr()) as u32 + 1) * core::mem::size_of::<u16>() as u32);
    pProfileData = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        (dwProfileData) as SIZE_T,
    ) as *mut PROFILEDATA;
    if pProfileData.is_null() {
        return;
    }

    (*pProfileData).dwRefCount = dwRefCount;
    (*pProfileData).dwState = dwState;
    (*pProfileData).bUnknownProfile = (bUnknownProfile) as i32;

    ptr = ((pProfileData as usize) + core::mem::size_of::<PROFILEDATA>()) as PWSTR;
    (*pProfileData).pszFullName = ptr;

    wcscpy((*pProfileData).pszFullName, szNameBuffer.as_ptr());

    ptr = ((ptr as usize) + ((wcslen((*pProfileData).pszFullName) as u32 + 1) * (core::mem::size_of::<u16>()) as u32) as usize) as PWSTR;
    (*pProfileData).pszProfilePath = ptr;
    wcscpy((*pProfileData).pszProfilePath, szProfilePath.as_ptr());

    memset(core::ptr::addr_of_mut!(lvi) as LPVOID, 0x00, core::mem::size_of::<_LVITEMW>());
    lvi.mask = LVIF_TEXT | LVIF_STATE | LVIF_PARAM;
    lvi.pszText = (*pProfileData).pszFullName;
    lvi.state = 0;
    lvi.lParam = pProfileData as LPARAM;
    iItem = ListView_InsertItem(hwndListView, core::ptr::addr_of_mut!(lvi));

    StrFormatByteSizeW((ullProfileSize) as LONGLONG, szNameBuffer.as_mut_ptr(), ARRAYSIZE(szNameBuffer.as_ptr()) - 1);
    ListView_SetItemText(hwndListView, iItem, 1, szNameBuffer.as_mut_ptr());

    if dwState & 0x0010 != 0 {
        nId = (IDS_USERPROFILE_ROAMING) as i32;
    } else {
        nId = (IDS_USERPROFILE_LOCAL) as i32;
    }

    LoadStringW(hApplet, nId as u32, szNameBuffer.as_mut_ptr(), ARRAYSIZE(szNameBuffer.as_ptr()));

    ListView_SetItemText(hwndListView, iItem, 2, szNameBuffer.as_mut_ptr());

    if dwState & 0x0001 != 0 {
        nId = (IDS_USERPROFILE_MANDATORY) as i32;
    } else if dwState & 0x0010 != 0 {
        nId = (IDS_USERPROFILE_ROAMING) as i32;
    } else {
        nId = (IDS_USERPROFILE_LOCAL) as i32;
    }

    LoadStringW(hApplet, nId as u32, szNameBuffer.as_mut_ptr(), ARRAYSIZE(szNameBuffer.as_ptr()));

    ListView_SetItemText(hwndListView, iItem, 3, szNameBuffer.as_mut_ptr());

    FileTimeToSystemTime(
        core::ptr::addr_of_mut!(FindData.ftLastWriteTime),
        core::ptr::addr_of_mut!(SystemTime),
    );

    GetDateFormatW(
        LOCALE_USER_DEFAULT,
        DATE_SHORTDATE,
        core::ptr::addr_of_mut!(SystemTime),
        core::ptr::null_mut(),
        szNameBuffer.as_mut_ptr(),
        ARRAYSIZE(szNameBuffer.as_ptr()),
    );

    ListView_SetItemText(hwndListView, iItem, 4, szNameBuffer.as_mut_ptr());
}
unsafe extern "C" fn UpdateButtonState(
    hwndDlg: HWND,
    hwndListView: HWND) {
    let mut Item: LVITEM = core::mem::zeroed();
    let mut iSelected: i32;
    let mut bChange: i32 = FALSE;
    let mut bCopy: i32 = FALSE;
    let mut bDelete: i32 = FALSE;
    let pProfileData: *mut PROFILEDATA;

    if ListView_GetSelectedCount(hwndListView) != 0
    {
        iSelected = ListView_GetNextItem(hwndListView, -1, LVNI_SELECTED);
        if iSelected != -1
        {
            ZeroMemory(&mut Item as *mut _ as *mut core::ffi::c_void, core::mem::size_of::<LVITEM>());
            Item.mask = LVIF_PARAM;
            Item.iItem = iSelected;
            if ListView_GetItem(hwndListView, &mut Item) != 0
            {
                if Item.lParam != 0
                {
                    pProfileData = Item.lParam as *mut PROFILEDATA;

                    if (*pProfileData).bUnknownProfile != 0
                    {
                        bDelete = TRUE;
                        bCopy = FALSE;
                    }
                    else
                    {
                        bDelete = ((*pProfileData).dwRefCount == 0) as i32;
                        bCopy = ((*pProfileData).dwRefCount == 0) as i32;
                    }
                }
            }

            bChange = TRUE;
        }
    }

    EnableWindow(GetDlgItem(hwndDlg, (IDC_USERPROFILE_CHANGE) as i32), bChange);
    EnableWindow(GetDlgItem(hwndDlg, (IDC_USERPROFILE_DELETE) as i32), bDelete);
    EnableWindow(GetDlgItem(hwndDlg, (IDC_USERPROFILE_COPY) as i32), bCopy);
}
unsafe extern "C" fn AddUserProfiles(
    hwndDlg: HWND,
    hwndListView: HWND,
    bAdmin: BOOL,
) {
    let mut hKeyUserProfiles: HKEY = (INVALID_HANDLE_VALUE).cast();
    let mut hProfileKey: HKEY = core::ptr::null_mut();
    let mut dwIndex: u32;
    let mut szProfileSid: [u16; 64] = [0; 64];
    let mut dwSidLength: u32;
    let mut ftLastWrite: FILETIME = core::mem::zeroed();
    let mut dwSize: u32 = 0;
    let mut hToken: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut pTokenUser: PTOKEN_USER = core::ptr::null_mut();
    let mut pProfileSid: PSID;
    let mut pszProfileSid: PWSTR;

    if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut hToken) == 0 {
        return;
    }

    'done: {
        if GetTokenInformation(hToken, TokenUser, core::ptr::null_mut(), 0, &mut dwSize) != 0 ||
            GetLastError() != ERROR_INSUFFICIENT_BUFFER
        {
            break 'done;
        }

        pTokenUser = HeapAlloc(GetProcessHeap(), 0, (dwSize) as SIZE_T) as PTOKEN_USER;
        if pTokenUser.is_null() {
            break 'done;
        }

        if GetTokenInformation(hToken, TokenUser, pTokenUser as LPVOID, dwSize, &mut dwSize) == 0 {
            break 'done;
        }

        if RegOpenKeyExW(
            HKEY_LOCAL_MACHINE,
            b"Software\\Microsoft\\Windows NT\\CurrentVersion\\ProfileList\0".as_ptr() as *const u16,
            0,
            KEY_READ,
            &mut hKeyUserProfiles,
        ) != 0
        {
            break 'done;
        }

        if bAdmin != 0 {
            dwIndex = 0;
            loop {
                dwSidLength = ARRAYSIZE(szProfileSid.as_ptr());
                if RegEnumKeyExW(
                    hKeyUserProfiles,
                    dwIndex,
                    szProfileSid.as_mut_ptr(),
                    &mut dwSidLength,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    &mut ftLastWrite,
                ) != 0
                {
                    break;
                }

                if RegOpenKeyExW(
                    hKeyUserProfiles,
                    szProfileSid.as_ptr(),
                    0,
                    KEY_READ,
                    &mut hProfileKey,
                ) == (ERROR_SUCCESS) as i32
                {
                    if ConvertStringSidToSid(szProfileSid.as_ptr(), &mut pProfileSid) != 0 {
                        AddUserProfile(hwndListView, pProfileSid, hProfileKey);
                        LocalFree(pProfileSid);
                    }

                    RegCloseKey(hProfileKey);
                }

                dwIndex += 1;
            }
        } else {
            if ConvertSidToStringSidW((*pTokenUser).User.Sid, &mut pszProfileSid) != 0 {
                if RegOpenKeyExW(
                    hKeyUserProfiles,
                    pszProfileSid,
                    0,
                    KEY_READ,
                    &mut hProfileKey,
                ) == (ERROR_SUCCESS) as i32
                {
                    AddUserProfile(hwndListView, (*pTokenUser).User.Sid, hProfileKey);
                    RegCloseKey(hProfileKey);
                }

                LocalFree((pszProfileSid).cast());
            }
        }

        if ListView_GetItemCount(hwndListView) != 0 {
            ListView_SetItemState(
                hwndListView,
                0,
                LVIS_SELECTED | LVIS_FOCUSED,
                LVIS_SELECTED | LVIS_FOCUSED,
            );
        }

        UpdateButtonState(hwndDlg, hwndListView);
    }

    if hKeyUserProfiles != (INVALID_HANDLE_VALUE).cast() {
        RegCloseKey(hKeyUserProfiles);
    }

    if !pTokenUser.is_null() {
        HeapFree(GetProcessHeap(), 0, pTokenUser as LPVOID);
    }

    if !hToken.is_null() {
        CloseHandle(hToken);
    }
}
unsafe extern "C" fn OnInitUserProfileDialog(hwndDlg: HWND) {
    let bAdmin: BOOL;

    bAdmin = IsUserAdmin();

    SetListViewColumns(GetDlgItem(hwndDlg, (IDC_USERPROFILE_LIST) as i32));

    ShowWindow(GetDlgItem(hwndDlg, (IDC_USERPROFILE_DELETE) as i32), if bAdmin != 0 { (SW_SHOW) as i32 } else { (SW_HIDE) as i32 });
    ShowWindow(GetDlgItem(hwndDlg, (IDC_USERPROFILE_COPY) as i32), if bAdmin != 0 { (SW_SHOW) as i32 } else { (SW_HIDE) as i32 });

    AddUserProfiles(hwndDlg, GetDlgItem(hwndDlg, (IDC_USERPROFILE_LIST) as i32), bAdmin);
}
unsafe extern "C" fn OnNotify(
    hwndDlg: HWND,
    nmhdr: *mut NMHDR) {
    let mut pNMLV: LPNMLISTVIEW;

    if (*nmhdr).idFrom == IDC_USERACCOUNT_LINK && (*nmhdr).code == NM_CLICK {
        ShellExecuteW(hwndDlg, core::ptr::null_mut(), b"usrmgr.cpl\0".as_ptr() as PWSTR, core::ptr::null_mut(), core::ptr::null_mut(), 0);
    } else if (*nmhdr).idFrom == IDC_USERPROFILE_LIST {
        match ((*nmhdr).code) as i32 {
            LVN_ITEMCHANGED => {
                UpdateButtonState(hwndDlg, (*nmhdr).hwndFrom);
            }
            NM_DBLCLK => {
                ChangeUserProfileType(hwndDlg);
            }
            LVN_DELETEITEM => {
                pNMLV = nmhdr as LPNMLISTVIEW;
                if (*pNMLV).lParam != 0 {
                    HeapFree(GetProcessHeap(), 0, (*pNMLV).lParam as LPVOID);
                }
            }
            _ => {}
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn UserProfileDlgProc(
    hwndDlg: HWND,
    uMsg: u32,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    match uMsg {
        WM_INITDIALOG => {
            OnInitUserProfileDialog(hwndDlg);
            return TRUE;
        }

        WM_COMMAND => {
            match LOWORD(wParam) {
                IDOK | IDCANCEL => {
                    EndDialog(hwndDlg, LOWORD(wParam));
                    return TRUE;
                }

                IDC_USERPROFILE_CHANGE => {
                    ChangeUserProfileType(hwndDlg);
                }

                IDC_USERPROFILE_DELETE => {
                    DeleteUserProfile(hwndDlg);
                }

                IDC_USERPROFILE_COPY => {
                    CopyUserProfile(hwndDlg);
                }

                _ => {}
            }
        }

        WM_NOTIFY => {
            OnNotify(hwndDlg, lParam as *mut NMHDR);
        }

        _ => {}
    }

    FALSE
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS System Control Panel Applet
//  * LICENSE:     GPL - See COPYING in the top level directory
//  * FILE:        dll/cpl/sysdm/userprofile.c
//  * PURPOSE:     Computer settings for networking
//  * COPYRIGHT:   Copyright Thomas Weidenmueller <w3seek@reactos.org>
//  *              Copyright 2006 Ged Murphy <gedmurphy@gmail.com>
//  *
//  */
//
// #include "precomp.h"
// #include <sddl.h>
// #include <winnls.h>
//
// #include <debug.h>
//
// typedef struct _PROFILEDATA
// {
//     DWORD dwRefCount;
//     DWORD dwState;
//     BOOL bUnknownProfile;
//     PWSTR pszFullName;
//     PWSTR pszProfilePath;
// } PROFILEDATA, *PPROFILEDATA;
//
//
// static
// BOOL
// OnProfileTypeInit(
//      HWND hwndDlg,
//      PPROFILEDATA pProfileData)
// {
//     PWSTR pszRawBuffer = NULL, pszCookedBuffer = NULL;
//     INT nLength;
//
//     nLength = LoadStringW(hApplet, IDS_USERPROFILE_TYPE_TEXT, (PWSTR)&pszRawBuffer, 0);
//     pszRawBuffer = NULL;
//     if (nLength == 0)
//         return FALSE;
//
//     pszRawBuffer = HeapAlloc(GetProcessHeap(), 0, (nLength + 1) * sizeof(WCHAR));
//     if (pszRawBuffer == NULL)
//         return FALSE;
//
//     LoadStringW(hApplet, IDS_USERPROFILE_TYPE_TEXT, pszRawBuffer, nLength + 1);
//
//     pszCookedBuffer = HeapAlloc(GetProcessHeap(), 0, (nLength + wcslen(pProfileData->pszFullName) + 1) * sizeof(WCHAR));
//     if (pszCookedBuffer == NULL)
//         goto done;
//
//     swprintf(pszCookedBuffer, pszRawBuffer, pProfileData->pszFullName);
//
//     /* Set the full text */
//     SetDlgItemText(hwndDlg, IDC_USERPROFILE_TYPE_TEXT, pszCookedBuffer);
//
//     /* FIXME: Right now, we support local user profiles only! */
//     EnableWindow(GetDlgItem(hwndDlg, IDC_USERPROFILE_TYPE_ROAMING), FALSE);
//     Button_SetCheck(GetDlgItem(hwndDlg, IDC_USERPROFILE_TYPE_LOCAL), BST_CHECKED);
//     EnableWindow(GetDlgItem(hwndDlg, IDOK), FALSE);
//
// done:
//     if (pszCookedBuffer != NULL)
//         HeapFree(GetProcessHeap(), 0, pszCookedBuffer);
//
//     if (pszRawBuffer != NULL)
//         HeapFree(GetProcessHeap(), 0, pszRawBuffer);
//
//     return TRUE;
// }
//
//
// static
// INT_PTR
// __stdcall
// UserProfileTypeDlgProc(
//      HWND hwndDlg,
//      UINT uMsg,
//      WPARAM wParam,
//      LPARAM lParam)
// {
//     switch (uMsg)
//     {
//         case WM_INITDIALOG:
//             OnProfileTypeInit(hwndDlg, (PPROFILEDATA)lParam);
//             return TRUE;
//
//         case WM_DESTROY:
//             break;
//
//         case WM_COMMAND:
//             switch (LOWORD(wParam))
//             {
//                 case IDOK:
//                 case IDCANCEL:
//                     EndDialog(hwndDlg,
//                               LOWORD(wParam));
//                     return TRUE;
//             }
//             break;
//     }
//
//     return FALSE;
// }
//
//
// static
// BOOL
// ChangeUserProfileType(
//      HWND hwndDlg)
// {
//     HWND hwndListView;
//     LVITEM Item;
//     INT iSelected;
//
//     DPRINT("ChangeUserProfileType(%p)\n", hwndDlg);
//
//     hwndListView = GetDlgItem(hwndDlg, IDC_USERPROFILE_LIST);
//     if (hwndListView == NULL)
//         return FALSE;
//
//     iSelected = ListView_GetNextItem(hwndListView, -1, LVNI_SELECTED);
//     if (iSelected == -1)
//         return FALSE;
//
//     ZeroMemory(&Item, sizeof(LVITEM));
//     Item.mask = LVIF_PARAM;
//     Item.iItem = iSelected;
//     if (!ListView_GetItem(hwndListView, &Item))
//         return FALSE;
//
//     if (Item.lParam == 0)
//         return FALSE;
//
//     if (DialogBoxParam(hApplet,
//                        MAKEINTRESOURCE(IDD_USERPROFILE_TYPE),
//                        hwndDlg,
//                        UserProfileTypeDlgProc,
//                        (LPARAM)Item.lParam) == IDOK)
//     {
//         /* FIXME: Update the profile list view */
//         return TRUE;
//     }
//
//     return FALSE;
// }
//
//
// static
// BOOL
// DeleteUserProfile(
//      HWND hwndDlg)
// {
//     HWND hwndListView;
//     LVITEM Item;
//     INT iSelected;
//     PPROFILEDATA pProfileData;
//
//     DPRINT("DeleteUserProfile()\n");
//
//     hwndListView = GetDlgItem(hwndDlg, IDC_USERPROFILE_LIST);
//     if (hwndListView == NULL)
//         return FALSE;
//
//     iSelected = ListView_GetNextItem(hwndListView, -1, LVNI_SELECTED);
//     if (iSelected == -1)
//         return FALSE;
//
//     ZeroMemory(&Item, sizeof(LVITEM));
//     Item.mask = LVIF_PARAM;
//     Item.iItem = iSelected;
//     if (!ListView_GetItem(hwndListView, &Item))
//         return FALSE;
//
//     if (Item.lParam == 0)
//         return FALSE;
//
//     pProfileData = (PPROFILEDATA)Item.lParam;
//     if (pProfileData->dwRefCount != 0)
//         return FALSE;
//
//     if (ResourceMessageBox(hApplet,
//                            hwndDlg,
//                            MB_ICONQUESTION | MB_YESNO,
//                            IDS_USERPROFILE_CONFIRM_DELETE_TITLE,
//                            IDS_USERPROFILE_CONFIRM_DELETE,
//                            pProfileData->pszFullName) == IDYES)
//     {
//         /* FIXME: Delete the profile here! */
//         return TRUE;
//     }
//
//     return FALSE;
// }
//
//
// static
// INT_PTR
// __stdcall
// CopyUserProfileDlgProc(
//      HWND hwndDlg,
//      UINT uMsg,
//      WPARAM wParam,
//      LPARAM lParam)
// {
//     switch (uMsg)
//     {
//         case WM_INITDIALOG:
//             return TRUE;
//
//         case WM_DESTROY:
//             break;
//
//         case WM_COMMAND:
//             switch (LOWORD(wParam))
//             {
//                 case IDOK:
//                 case IDCANCEL:
//                     EndDialog(hwndDlg,
//                               LOWORD(wParam));
//                     return TRUE;
//             }
//             break;
//     }
//
//     return FALSE;
// }
//
//
// static
// BOOL
// CopyUserProfile(
//      HWND hwndDlg)
// {
//     HWND hwndListView;
//     LVITEM Item;
//     INT iSelected;
//
//     DPRINT("CopyUserProfile()\n");
//
//     hwndListView = GetDlgItem(hwndDlg, IDC_USERPROFILE_LIST);
//     if (hwndListView == NULL)
//         return FALSE;
//
//     iSelected = ListView_GetNextItem(hwndListView, -1, LVNI_SELECTED);
//     if (iSelected == -1)
//         return FALSE;
//
//     ZeroMemory(&Item, sizeof(LVITEM));
//     Item.mask = LVIF_PARAM;
//     Item.iItem = iSelected;
//     if (!ListView_GetItem(hwndListView, &Item))
//         return FALSE;
//
//     if (Item.lParam == 0)
//         return FALSE;
//
//     if (DialogBoxParam(hApplet,
//                        MAKEINTRESOURCE(IDD_USERPROFILE_COPY),
//                        hwndDlg,
//                        CopyUserProfileDlgProc,
//                        (LPARAM)Item.lParam) == IDOK)
//     {
//         /* FIXME: Update the profile list view */
//         return TRUE;
//     }
//
//     return FALSE;
// }
//
//
// static VOID
// SetListViewColumns(
//      HWND hwndListView)
// {
//     LV_COLUMN column;
//     RECT rect;
//     TCHAR szStr[32];
//
//     GetClientRect(hwndListView, &rect);
//
//     SendMessage(hwndListView, LVM_SETEXTENDEDLISTVIEWSTYLE, 0, LVS_EX_FULLROWSELECT);
//
//     memset(&column, 0x00, sizeof(column));
//     column.mask = LVCF_FMT | LVCF_WIDTH | LVCF_SUBITEM | LVCF_TEXT;
//     column.fmt = LVCFMT_LEFT;
//     column.cx = (INT)((rect.right - rect.left) * 0.40);
//     column.iSubItem = 0;
//     LoadString(hApplet, IDS_USERPROFILE_NAME, szStr, ARRAYSIZE(szStr));
//     column.pszText = szStr;
//     (void)ListView_InsertColumn(hwndListView, 0, &column);
//
//     column.fmt = LVCFMT_RIGHT;
//     column.cx = (INT)((rect.right - rect.left) * 0.15);
//     column.iSubItem = 1;
//     LoadString(hApplet, IDS_USERPROFILE_SIZE, szStr, ARRAYSIZE(szStr));
//     column.pszText = szStr;
//     (void)ListView_InsertColumn(hwndListView, 1, &column);
//
//     column.fmt = LVCFMT_LEFT;
//     column.cx = (INT)((rect.right - rect.left) * 0.15);
//     column.iSubItem = 2;
//     LoadString(hApplet, IDS_USERPROFILE_TYPE, szStr, ARRAYSIZE(szStr));
//     column.pszText = szStr;
//     (void)ListView_InsertColumn(hwndListView, 2, &column);
//
//     column.fmt = LVCFMT_LEFT;
//     column.cx = (INT)((rect.right - rect.left) * 0.15);
//     column.iSubItem = 3;
//     LoadString(hApplet, IDS_USERPROFILE_STATUS, szStr, ARRAYSIZE(szStr));
//     column.pszText = szStr;
//     (void)ListView_InsertColumn(hwndListView, 3, &column);
//
//     column.fmt = LVCFMT_LEFT;
//     column.cx = (INT)((rect.right - rect.left) * 0.15) - GetSystemMetrics(SM_CYHSCROLL);
//     column.iSubItem = 4;
//     LoadString(hApplet, IDS_USERPROFILE_MODIFIED, szStr, ARRAYSIZE(szStr));
//     column.pszText = szStr;
//     (void)ListView_InsertColumn(hwndListView, 4, &column);
// }
//
//
// static
// BOOL
// GetProfileSize(
//      PWSTR pszProfilePath,
//      PULONGLONG pullProfileSize)
// {
//     HANDLE hFile = INVALID_HANDLE_VALUE;
//     WIN32_FIND_DATA FindData;
//     DWORD dwProfilePathLength;
//     ULARGE_INTEGER Size;
//     BOOL bResult = TRUE;
//
//     dwProfilePathLength = wcslen(pszProfilePath);
//
//     wcscat(pszProfilePath, L"\\*.*");
//
//     hFile = FindFirstFileW(pszProfilePath, &FindData);
//     if (hFile == INVALID_HANDLE_VALUE)
//     {
//         if ((GetLastError() != ERROR_FILE_NOT_FOUND) &&
//             (GetLastError() != ERROR_PATH_NOT_FOUND))
//             bResult = FALSE;
//
//         goto done;
//     }
//
//     do
//     {
//         if (FindData.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY)
//         {
//             if ((_wcsicmp(FindData.cFileName, L".") == 0) ||
//                 (_wcsicmp(FindData.cFileName, L"..") == 0))
//                 continue;
//
//             pszProfilePath[dwProfilePathLength + 1] = UNICODE_NULL;
//             wcscat(pszProfilePath, FindData.cFileName);
//
//             if (!GetProfileSize(pszProfilePath, pullProfileSize))
//             {
//                 bResult = FALSE;
//                 goto done;
//             }
//         }
//         else
//         {
//             Size.u.LowPart = FindData.nFileSizeLow;
//             Size.u.HighPart = FindData.nFileSizeHigh;
//             *pullProfileSize += Size.QuadPart;
//         }
//     }
//     while (FindNextFile(hFile, &FindData));
//
// done:
//     pszProfilePath[dwProfilePathLength] = UNICODE_NULL;
//
//     if (hFile != INVALID_HANDLE_VALUE)
//         FindClose(hFile);
//
//     return bResult;
// }
//
//
// static
// BOOL
// GetProfileName(
//      PSID pProfileSid,
//      DWORD dwNameBufferSize,
//      PWSTR pszNameBuffer,
//      PBOOL pbUnknownProfile)
// {
//     WCHAR szAccountName[128], szDomainName[128];
//     DWORD dwAccountNameSize, dwDomainNameSize;
//     SID_NAME_USE Use;
//
//     dwAccountNameSize = ARRAYSIZE(szAccountName);
//     dwDomainNameSize = ARRAYSIZE(szDomainName);
//     if (!LookupAccountSidW(NULL,
//                            pProfileSid,
//                            szAccountName,
//                            &dwAccountNameSize,
//                            szDomainName,
//                            &dwDomainNameSize,
//                            &Use))
//     {
//         /* Unknown account */
//         LoadStringW(hApplet, IDS_USERPROFILE_ACCOUNT_UNKNOWN, pszNameBuffer, dwNameBufferSize);
//         *pbUnknownProfile = TRUE;
//     }
//     else
//     {
//         /* Show only the user accounts */
//         if (Use != SidTypeUser)
//             return FALSE;
//
//         if (szAccountName[0] == UNICODE_NULL)
//         {
//             /* Deleted account */
//             LoadStringW(hApplet, IDS_USERPROFILE_ACCOUNT_DELETED, pszNameBuffer, dwNameBufferSize);
//         }
//         else
//         {
//             /* Normal account */
//             wsprintf(pszNameBuffer, L"%s\\%s", szDomainName, szAccountName);
//         }
//         *pbUnknownProfile = FALSE;
//     }
//
//     return TRUE;
// }
//
//
// static VOID
// AddUserProfile(
//      HWND hwndListView,
//      PSID pProfileSid,
//      HKEY hProfileKey)
// {
//     WCHAR szTempProfilePath[MAX_PATH], szProfilePath[MAX_PATH];
//     WCHAR szNameBuffer[256];
//     PPROFILEDATA pProfileData = NULL;
//     DWORD dwProfileData, dwSize, dwType, dwState = 0, dwRefCount = 0;
//     DWORD dwProfilePathLength;
//     PWSTR ptr;
//     INT nId, iItem;
//     LV_ITEM lvi;
//     WIN32_FIND_DATA FindData;
//     HANDLE hFile;
//     SYSTEMTIME SystemTime;
//     ULONGLONG ullProfileSize;
//     BOOL bUnknownProfile;
//     DWORD dwError;
//
//     /* Get the profile path */
//     dwSize = MAX_PATH * sizeof(WCHAR);
//     dwError = RegQueryValueExW(hProfileKey,
//                                L"ProfileImagePath",
//                                NULL,
//                                &dwType,
//                                (LPBYTE)szTempProfilePath,
//                                &dwSize);
//     if (dwError != ERROR_SUCCESS)
//         return;
//
//     /* Expand it */
//     ExpandEnvironmentStringsW(szTempProfilePath,
//                               szProfilePath,
//                               MAX_PATH);
//
//     /* Check if the profile path exists */
//     hFile = FindFirstFileW(szProfilePath, &FindData);
//     if (hFile == INVALID_HANDLE_VALUE)
//         return;
//
//     FindClose(hFile);
//
//     /* Get the length of the profile path */
//     dwProfilePathLength = wcslen(szProfilePath);
//
//     /* Check for the ntuser.dat file */
//     wcscat(szProfilePath, L"\\ntuser.dat");
//     hFile = FindFirstFileW(szProfilePath, &FindData);
//     if (hFile == INVALID_HANDLE_VALUE)
//         return;
//
//     FindClose(hFile);
//     szProfilePath[dwProfilePathLength] = UNICODE_NULL;
//
//     /* Get the profile size */
//     ullProfileSize = 0ULL;
//     GetProfileSize(szProfilePath, &ullProfileSize);
//
//     /* Get the profile name */
//     if (!GetProfileName(pProfileSid,
//                         ARRAYSIZE(szNameBuffer),
//                         szNameBuffer,
//                         &bUnknownProfile))
//         return;
//
//     /* Get the profile state value */
//     dwSize = sizeof(dwState);
//     if (RegQueryValueExW(hProfileKey,
//                          L"State",
//                          NULL,
//                          &dwType,
//                          (LPBYTE)&dwState,
//                          &dwSize) != ERROR_SUCCESS)
//     {
//         dwState = 0;
//     }
//
//     /* Get the profile reference counter */
//     dwSize = sizeof(dwRefCount);
//     if (RegQueryValueExW(hProfileKey,
//                          L"RefCount",
//                          NULL,
//                          &dwType,
//                          (LPBYTE)&dwRefCount,
//                          &dwSize) != ERROR_SUCCESS)
//     {
//         dwRefCount = 0;
//     }
//
//     /* Create and fill the profile data entry */
//     dwProfileData = sizeof(PROFILEDATA) +
//                     ((wcslen(szNameBuffer) + 1) * sizeof(WCHAR)) +
//                     ((wcslen(szProfilePath) + 1) * sizeof(WCHAR));
//     pProfileData = HeapAlloc(GetProcessHeap(),
//                              HEAP_ZERO_MEMORY,
//                              dwProfileData);
//     if (pProfileData == NULL)
//         return;
//
//     pProfileData->dwRefCount = dwRefCount;
//     pProfileData->dwState = dwState;
//     pProfileData->bUnknownProfile = bUnknownProfile;
//
//     ptr = (PWSTR)((ULONG_PTR)pProfileData + sizeof(PROFILEDATA));
//     pProfileData->pszFullName = ptr;
//
//     wcscpy(pProfileData->pszFullName, szNameBuffer);
//
//     ptr = (PWSTR)((ULONG_PTR)ptr + ((wcslen(pProfileData->pszFullName) + 1) * sizeof(WCHAR)));
//     pProfileData->pszProfilePath = ptr;
//     wcscpy(pProfileData->pszProfilePath, szProfilePath);
//
//     /* Add the profile and set its name */
//     memset(&lvi, 0x00, sizeof(lvi));
//     lvi.mask = LVIF_TEXT | LVIF_STATE | LVIF_PARAM;
//     lvi.pszText = pProfileData->pszFullName;
//     lvi.state = 0;
//     lvi.lParam = (LPARAM)pProfileData;
//     iItem = ListView_InsertItem(hwndListView, &lvi);
//
//     /* Set the profile size */
//     StrFormatByteSizeW(ullProfileSize, szNameBuffer, ARRAYSIZE(szNameBuffer) - 1);
//     ListView_SetItemText(hwndListView, iItem, 1, szNameBuffer);
//
//     /* Set the profile type */
//     if (dwState & 0x0010) // PROFILE_UPDATE_CENTRAL
//         nId = IDS_USERPROFILE_ROAMING;
//     else
//         nId = IDS_USERPROFILE_LOCAL;
//
//     LoadStringW(hApplet, nId, szNameBuffer, ARRAYSIZE(szNameBuffer));
//
//     ListView_SetItemText(hwndListView, iItem, 2, szNameBuffer);
//
//     /* FIXME: Set the profile status */
//     if (dwState & 0x0001) // PROFILE_MANDATORY
//         nId = IDS_USERPROFILE_MANDATORY;
//     else if (dwState & 0x0010) // PROFILE_UPDATE_CENTRAL
//         nId = IDS_USERPROFILE_ROAMING;
//     else
//         nId = IDS_USERPROFILE_LOCAL;
//
//     LoadStringW(hApplet, nId, szNameBuffer, ARRAYSIZE(szNameBuffer));
//
//     ListView_SetItemText(hwndListView, iItem, 3, szNameBuffer);
//
//     /* Set the profile modified time */
//     FileTimeToSystemTime(&FindData.ftLastWriteTime,
//                          &SystemTime);
//
//     GetDateFormatW(LOCALE_USER_DEFAULT,
//                    DATE_SHORTDATE,
//                    &SystemTime,
//                    NULL,
//                    szNameBuffer,
//                    ARRAYSIZE(szNameBuffer));
//
//     ListView_SetItemText(hwndListView, iItem, 4, szNameBuffer);
// }
//
//
// static VOID
// UpdateButtonState(
//      HWND hwndDlg,
//      HWND hwndListView)
// {
//     LVITEM Item;
//     INT iSelected;
//     BOOL bChange = FALSE;
//     BOOL bCopy = FALSE;
//     BOOL bDelete = FALSE;
//     PPROFILEDATA pProfileData;
//
//     if (ListView_GetSelectedCount(hwndListView) != 0)
//     {
//         iSelected = ListView_GetNextItem(hwndListView, -1, LVNI_SELECTED);
//         if (iSelected != -1)
//         {
//             ZeroMemory(&Item, sizeof(LVITEM));
//             Item.mask = LVIF_PARAM;
//             Item.iItem = iSelected;
//             if (ListView_GetItem(hwndListView, &Item))
//             {
//                 if (Item.lParam != 0)
//                 {
//                     pProfileData = (PPROFILEDATA)Item.lParam;
//
//                     if (pProfileData->bUnknownProfile)
//                     {
//                         bDelete = TRUE;
//                         bCopy = FALSE;
//                     }
//                     else
//                     {
//                         bDelete = (pProfileData->dwRefCount == 0);
//                         bCopy = (pProfileData->dwRefCount == 0);
//                     }
//                 }
//             }
//
//             bChange = TRUE;
//         }
//     }
//
//     EnableWindow(GetDlgItem(hwndDlg, IDC_USERPROFILE_CHANGE), bChange);
//     EnableWindow(GetDlgItem(hwndDlg, IDC_USERPROFILE_DELETE), bDelete);
//     EnableWindow(GetDlgItem(hwndDlg, IDC_USERPROFILE_COPY), bCopy);
// }
//
//
// static VOID
// AddUserProfiles(
//      HWND hwndDlg,
//      HWND hwndListView,
//      BOOL bAdmin)
// {
//     HKEY hKeyUserProfiles = INVALID_HANDLE_VALUE;
//     HKEY hProfileKey;
//     DWORD dwIndex;
//     WCHAR szProfileSid[64];
//     DWORD dwSidLength;
//     FILETIME ftLastWrite;
//     DWORD dwSize;
//     HANDLE hToken = NULL;
//     PTOKEN_USER pTokenUser = NULL;
//     PSID pProfileSid;
//     PWSTR pszProfileSid;
//
//     if (!OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &hToken))
//         return;
//
//     if (GetTokenInformation(hToken, TokenUser, NULL, 0, &dwSize) ||
//         GetLastError() != ERROR_INSUFFICIENT_BUFFER)
//     {
//         goto done;
//     }
//
//     pTokenUser = HeapAlloc(GetProcessHeap(), 0, dwSize);
//     if (pTokenUser == NULL)
//         goto done;
//
//     if (!GetTokenInformation(hToken, TokenUser, pTokenUser, dwSize, &dwSize))
//         goto done;
//
//     if (RegOpenKeyExW(HKEY_LOCAL_MACHINE,
//                       L"Software\\Microsoft\\Windows NT\\CurrentVersion\\ProfileList",
//                       0,
//                       KEY_READ,
//                       &hKeyUserProfiles))
//         goto done;
//
//     if (bAdmin)
//     {
//         for (dwIndex = 0; ; dwIndex++)
//         {
//             dwSidLength = ARRAYSIZE(szProfileSid);
//             if (RegEnumKeyExW(hKeyUserProfiles,
//                               dwIndex,
//                               szProfileSid,
//                               &dwSidLength,
//                               NULL,
//                               NULL,
//                               NULL,
//                               &ftLastWrite))
//                 break;
//
//             if (RegOpenKeyExW(hKeyUserProfiles,
//                               szProfileSid,
//                               0,
//                               KEY_READ,
//                               &hProfileKey) == ERROR_SUCCESS)
//             {
//                 if (ConvertStringSidToSid(szProfileSid, &pProfileSid))
//                 {
//                     AddUserProfile(hwndListView,
//                                    pProfileSid,
//                                    hProfileKey);
//                     LocalFree(pProfileSid);
//                 }
//
//                 RegCloseKey(hProfileKey);
//             }
//         }
//     }
//     else
//     {
//         if (ConvertSidToStringSidW(pTokenUser->User.Sid, &pszProfileSid))
//         {
//             if (RegOpenKeyExW(hKeyUserProfiles,
//                               pszProfileSid,
//                               0,
//                               KEY_READ,
//                               &hProfileKey) == ERROR_SUCCESS)
//             {
//                 AddUserProfile(hwndListView,
//                                pTokenUser->User.Sid,
//                                hProfileKey);
//                 RegCloseKey(hProfileKey);
//             }
//
//             LocalFree(pszProfileSid);
//         }
//     }
//
//     if (ListView_GetItemCount(hwndListView) != 0)
//         ListView_SetItemState(hwndListView, 0, LVIS_SELECTED | LVIS_FOCUSED, LVIS_SELECTED | LVIS_FOCUSED);
//
//     UpdateButtonState(hwndDlg, hwndListView);
//
// done:
//     if (hKeyUserProfiles != INVALID_HANDLE_VALUE)
//         RegCloseKey(hKeyUserProfiles);
//
//     if (pTokenUser != NULL)
//         HeapFree(GetProcessHeap(), 0, pTokenUser);
//
//     if (hToken != NULL)
//         CloseHandle(hToken);
// }
//
//
// static VOID
// OnInitUserProfileDialog(HWND hwndDlg)
// {
//     BOOL bAdmin;
//
//     bAdmin = IsUserAdmin();
//
//     /* Initialize the list view control */
//     SetListViewColumns(GetDlgItem(hwndDlg, IDC_USERPROFILE_LIST));
//
//     /* Hide the delete and copy buttons for non-admins */
//     ShowWindow(GetDlgItem(hwndDlg, IDC_USERPROFILE_DELETE), bAdmin ? SW_SHOW : SW_HIDE);
//     ShowWindow(GetDlgItem(hwndDlg, IDC_USERPROFILE_COPY), bAdmin ? SW_SHOW : SW_HIDE);
//
//     /* Add the profiles to the list view */
//     AddUserProfiles(hwndDlg, GetDlgItem(hwndDlg, IDC_USERPROFILE_LIST), bAdmin);
// }
//
// static
// VOID
// OnNotify(
//      HWND hwndDlg,
//      NMHDR *nmhdr)
// {
//     LPNMLISTVIEW pNMLV;
//
//     if (nmhdr->idFrom == IDC_USERACCOUNT_LINK && nmhdr->code == NM_CLICK)
//     {
//         ShellExecuteW(hwndDlg, NULL, L"usrmgr.cpl", NULL, NULL, 0);
//     }
//     else if (nmhdr->idFrom == IDC_USERPROFILE_LIST)
//     {
//         switch(nmhdr->code)
//         {
//             case LVN_ITEMCHANGED:
//                 UpdateButtonState(hwndDlg, nmhdr->hwndFrom);
//                 break;
//
//             case NM_DBLCLK:
//                 ChangeUserProfileType(hwndDlg);
//                 break;
//
//             case LVN_DELETEITEM:
//                 pNMLV = (LPNMLISTVIEW)nmhdr;
//                 if (pNMLV->lParam != 0)
//                     HeapFree(GetProcessHeap(), 0, (LPVOID)pNMLV->lParam);
//                 break;
//         }
//     }
// }
//
//
// /* Property page dialog callback */
// INT_PTR __stdcall
// UserProfileDlgProc(HWND hwndDlg,
//                    UINT uMsg,
//                    WPARAM wParam,
//                    LPARAM lParam)
// {
//     switch (uMsg)
//     {
//         case WM_INITDIALOG:
//             OnInitUserProfileDialog(hwndDlg);
//             return TRUE;
//
//         case WM_COMMAND:
//             switch (LOWORD(wParam))
//             {
//                 case IDOK:
//                 case IDCANCEL:
//                     EndDialog(hwndDlg,
//                               LOWORD(wParam));
//                     return TRUE;
//
//                 case IDC_USERPROFILE_CHANGE:
//                     ChangeUserProfileType(hwndDlg);
//                     break;
//
//                 case IDC_USERPROFILE_DELETE:
//                     DeleteUserProfile(hwndDlg);
//                     break;
//
//                 case IDC_USERPROFILE_COPY:
//                     CopyUserProfile(hwndDlg);
//                     break;
//             }
//             break;
//
//         case WM_NOTIFY:
//             OnNotify(hwndDlg, (NMHDR *)lParam);
//             break;
//     }
//
//     return FALSE;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
