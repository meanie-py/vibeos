use crate::prelude::*;
use crate::rapps::settings_cpp_h::FillDefaultSettings;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_96A296D224F2: [WCHAR; 1] = [0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub static mut g_pNewSettingsInfo: *mut SETTINGS_INFO = core::ptr::null_mut();
unsafe extern "system" fn BrowseFolderCallback(hwnd: HWND, uMsg: u32, lParam: LPARAM, lpData: LPARAM) -> i32 {
    match ((uMsg) as i32) as u32 {
        BFFM_INITIALIZED => {
            SendMessageW(hwnd, BFFM_SETSELECTIONW, (TRUE) as WPARAM, lpData);
        }
        __c if __c == BFFM_VALIDATEFAILED as u32 => {
            return (TRUE as i32);
        }
        _ => {}
    }
    return 0;
}

unsafe extern "C" fn ChooseFolder(hwnd: HWND) -> BOOL {
    let mut bRet: BOOL = (FALSE as i32);
    let mut bi: BROWSEINFOW = core::mem::zeroed();
    let mut szChooseFolderText = CStringW::new();

    szChooseFolderText.LoadStringW(IDS_CHOOSE_FOLDER_TEXT);

    ZeroMemory!(
        core::ptr::addr_of_mut!(bi) as *mut core::ffi::c_void,
        core::mem::size_of::<BROWSEINFOW>(),
    );
    bi.hwndOwner = hwnd;
    bi.pidlRoot = core::ptr::null_mut();
    bi.lpszTitle = szChooseFolderText.GetString();
    bi.ulFlags =
        BIF_USENEWUI | BIF_DONTGOBELOWDOMAIN | BIF_RETURNONLYFSDIRS | /* BIF_BROWSEFILEJUNCTIONS | */ BIF_VALIDATE;

    if ((CoInitialize(core::ptr::null_mut())) >= 0)
    {
        let mut szDir: [WCHAR; (MAX_PATH) as _] = [0 as WCHAR; (MAX_PATH) as _];
        if GetWindowTextW(
            GetDlgItem(hwnd, (IDC_DOWNLOAD_DIR_EDIT as i32)),
            szDir.as_mut_ptr(),
            (_countof!(szDir)) as i32,
        ) != 0
        {
            bi.lpfn = Some(BrowseFolderCallback);
            bi.lParam = szDir.as_mut_ptr() as LPARAM;
        }

        let lpItemList: LPITEMIDLIST = SHBrowseForFolderW(core::ptr::addr_of_mut!(bi));
        if !lpItemList.is_null() && SHGetPathFromIDListW(lpItemList, szDir.as_mut_ptr()) != 0
        {
            if szDir[0] != 0 {
                SetDlgItemTextW(hwnd, (IDC_DOWNLOAD_DIR_EDIT as i32), szDir.as_ptr());
                bRet = (TRUE as i32);
            }
        }

        CoTaskMemFree(lpItemList as *mut core::ffi::c_void);
        CoUninitialize();
    }

    bRet
}

unsafe extern "C" fn IsUrlValid(Url: *const u16) -> i32 {
    let mut UrlComponmentInfo: URL_COMPONENTSW = core::mem::zeroed();
    UrlComponmentInfo.dwStructSize = core::mem::size_of::<URL_COMPONENTSW>() as u32;
    UrlComponmentInfo.dwSchemeLength = 1;

    let bSuccess: i32 = InternetCrackUrlW(Url, wcslen(Url) as u32, 0, &mut UrlComponmentInfo);
    if bSuccess == 0 {
        return (FALSE as i32);
    }

    match UrlComponmentInfo.nScheme {
        INTERNET_SCHEME_HTTP | INTERNET_SCHEME_HTTPS | INTERNET_SCHEME_FTP | INTERNET_SCHEME_FILE => {
            // supported
            return (TRUE as i32);
        }
        _ => {
            return (FALSE as i32);
        }
    }
}

unsafe extern "C" fn IsCheckedDlgItem(hDlg: HWND, nIDDlgItem: i32) -> i32 {
    return (SendDlgItemMessageW(hDlg, nIDDlgItem, BM_GETCHECK, 0, 0) == BST_CHECKED as LRESULT) as i32;
}

unsafe extern "C" fn AdjustListViewHeader(hWndList: HWND) {
    ListView_SetColumnWidth!(hWndList, 0, LVSCW_AUTOSIZE_USEHEADER);
}

unsafe extern "C" fn HandleGeneralListItems(hWndList: HWND, Load: PSETTINGS_INFO, Save: PSETTINGS_INFO) {
    let Info: PSETTINGS_INFO = if !Load.is_null() { Load } else { Save };
    let Map: [(WORD, *mut BOOL); 4] = [
        ((IDS_CFG_SAVE_WINDOW_POS as u16), core::ptr::addr_of_mut!((*Info).bSaveWndPos)),
        ((IDS_CFG_UPDATE_AVLIST as u16), core::ptr::addr_of_mut!((*Info).bUpdateAtStart)),
        ((IDS_CFG_LOG_ENABLED as u16), core::ptr::addr_of_mut!((*Info).bLogEnabled)),
        ((IDS_CFG_SMALL_ICONS as u16), core::ptr::addr_of_mut!((*Info).bSmallIcons)),
    ];

    if ListView_GetItemCount!(hWndList) == 0 {
        let ExStyle: UINT = LVS_EX_CHECKBOXES | LVS_EX_LABELTIP;
        ListView_SetExtendedListViewStyleEx!(hWndList, ExStyle, ExStyle);
        let mut lvc: LVCOLUMN = core::mem::zeroed();
        lvc.mask = LVCF_TEXT | LVCF_SUBITEM;
        lvc.iSubItem = 0;
        lvc.pszText = CPP_COM_WSTR_96A296D224F2.as_ptr() as PWSTR;
        ListView_InsertColumn!(hWndList, 0, core::ptr::addr_of_mut!(lvc));

        let mut Name = CStringW::new();
        let mut i: SIZE_T = 0;
        while i < _countof!(Map) as SIZE_T {
            let mut Item: LVITEMW = core::mem::zeroed();
            Item.mask = LVIF_TEXT | LVIF_PARAM;
            Item.iItem = 0x7fff;
            Item.iSubItem = 0;
            Item.lParam = Map[i as usize].0 as LPARAM;
            Name.LoadStringW(Map[i as usize].0 as UINT);
            Item.pszText = Name.GetString() as PWSTR;
            Item.iItem = ListView_InsertItem!(hWndList, core::ptr::addr_of_mut!(Item));
            i += 1;
        }
        ListView_SetItemState!(hWndList, 0, -1, LVIS_FOCUSED | LVIS_SELECTED);
        AdjustListViewHeader(hWndList);
    }

    let mut i: SIZE_T = 0;
    while i < _countof!(Map) as SIZE_T {
        let mut FindInfo: LVFINDINFOW = core::mem::zeroed();
        FindInfo.flags = LVFI_PARAM;
        FindInfo.psz = core::ptr::null_mut();
        FindInfo.lParam = Map[i as usize].0 as LPARAM;
        let Idx: i32 = ListView_FindItem!(hWndList, -1, core::ptr::addr_of_mut!(FindInfo)) as i32;
        if Idx >= 0 {
            if !Load.is_null() {
                ListView_SetCheckState!(hWndList, Idx, *Map[i as usize].1);
            } else {
                *Map[i as usize].1 = ListView_GetCheckState!(hWndList, Idx) as BOOL;
            }
        }
        i += 1;
    }
}

unsafe extern "C" fn InitSettingsControls(hDlg: HWND, Info: PSETTINGS_INFO) {
    HandleGeneralListItems(GetDlgItem(hDlg, (IDC_GENERALLIST as i32)), Info, core::ptr::null_mut());
    SendDlgItemMessageW(hDlg, (IDC_DEL_AFTER_INSTALL as i32), BM_SETCHECK, ((*Info).bDelInstaller) as WPARAM, 0);

    let hCtl: HWND = GetDlgItem(hDlg, (IDC_DOWNLOAD_DIR_EDIT as i32));
    SetWindowTextW(hCtl, (*Info).szDownloadDir.as_ptr());
    SendMessageW(hCtl, EM_LIMITTEXT, (MAX_PATH - 1) as WPARAM, 0);

    CheckRadioButton(hDlg, (IDC_PROXY_DEFAULT as i32), (IDC_USE_PROXY as i32), (IDC_PROXY_DEFAULT + ((*Info).Proxy) as u32) as i32);

    if (*Info).Proxy == 2
    {
        EnableWindow(GetDlgItem(hDlg, (IDC_PROXY_SERVER as i32)), (TRUE) as BOOL);
        EnableWindow(GetDlgItem(hDlg, (IDC_NO_PROXY_FOR as i32)), (TRUE) as BOOL);
    }
    else
    {
        EnableWindow(GetDlgItem(hDlg, (IDC_PROXY_SERVER as i32)), (FALSE) as BOOL);
        EnableWindow(GetDlgItem(hDlg, (IDC_NO_PROXY_FOR as i32)), (FALSE) as BOOL);
    }

    CheckRadioButton(hDlg, (IDC_SOURCE_DEFAULT as i32), (IDC_USE_SOURCE as i32), if (*Info).bUseSource != 0 { (IDC_USE_SOURCE as i32) } else { (IDC_SOURCE_DEFAULT as i32) });

    EnableWindow(GetDlgItem(hDlg, (IDC_SOURCE_URL as i32)), (*Info).bUseSource);

    SetWindowTextW(GetDlgItem(hDlg, (IDC_SOURCE_URL as i32)), (*Info).szSourceURL.as_ptr());
    SetWindowTextW(GetDlgItem(hDlg, (IDC_PROXY_SERVER as i32)), (*Info).szProxyServer.as_ptr());
    SetWindowTextW(GetDlgItem(hDlg, (IDC_NO_PROXY_FOR as i32)), (*Info).szNoProxyFor.as_ptr());
}

unsafe extern "system" fn SettingsDlgProc(hDlg: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let NewSettingsInfo: &mut SETTINGS_INFO = &mut *g_pNewSettingsInfo;

    match Msg {
        WM_INITDIALOG => {
            InitSettingsControls(hDlg, core::ptr::addr_of_mut!(SettingsInfo));
            return (TRUE) as INT_PTR;
        }

        WM_SETTINGCHANGE | WM_THEMECHANGED | WM_SYSCOLORCHANGE => {
            SendMessage(GetDlgItem(hDlg, (IDC_GENERALLIST as i32)), Msg, wParam, lParam);
            AdjustListViewHeader(GetDlgItem(hDlg, (IDC_GENERALLIST as i32)));
        }

        WM_COMMAND => {
            'sw: {
                match (LOWORD!(wParam as DWORD)) as u32 {
                    IDC_CHOOSE => {
                        ChooseFolder(hDlg);
                    }

                    IDC_DEL_AFTER_INSTALL => {
                        NewSettingsInfo.bDelInstaller = IsCheckedDlgItem(hDlg, (IDC_DEL_AFTER_INSTALL as i32));
                    }

                    IDC_SOURCE_DEFAULT => {
                        NewSettingsInfo.bUseSource = (FALSE as i32);
                        EnableWindow(GetDlgItem(hDlg, (IDC_SOURCE_URL as i32)), NewSettingsInfo.bUseSource);
                    }

                    IDC_USE_SOURCE => {
                        NewSettingsInfo.bUseSource = (TRUE as i32);
                        EnableWindow(GetDlgItem(hDlg, (IDC_SOURCE_URL as i32)), NewSettingsInfo.bUseSource);
                    }

                    IDC_PROXY_DEFAULT => {
                        NewSettingsInfo.Proxy = 0;
                        EnableWindow(GetDlgItem(hDlg, (IDC_PROXY_SERVER as i32)), (FALSE) as BOOL);
                        EnableWindow(GetDlgItem(hDlg, (IDC_NO_PROXY_FOR as i32)), (FALSE) as BOOL);
                    }

                    IDC_NO_PROXY => {
                        NewSettingsInfo.Proxy = 1;
                        EnableWindow(GetDlgItem(hDlg, (IDC_PROXY_SERVER as i32)), (FALSE) as BOOL);
                        EnableWindow(GetDlgItem(hDlg, (IDC_NO_PROXY_FOR as i32)), (FALSE) as BOOL);
                    }

                    IDC_USE_PROXY => {
                        NewSettingsInfo.Proxy = 2;
                        EnableWindow(GetDlgItem(hDlg, (IDC_PROXY_SERVER as i32)), (TRUE) as BOOL);
                        EnableWindow(GetDlgItem(hDlg, (IDC_NO_PROXY_FOR as i32)), (TRUE) as BOOL);
                    }

                    IDC_DEFAULT_SETTINGS => {
                        FillDefaultSettings(g_pNewSettingsInfo);
                        InitSettingsControls(hDlg, g_pNewSettingsInfo);
                    }

                    IDOK => {
                        HandleGeneralListItems(GetDlgItem(hDlg, (IDC_GENERALLIST as i32)), core::ptr::null_mut(), g_pNewSettingsInfo);

                        let mut szDir = CStringW::new();
                        let mut szSource = CStringW::new();
                        let mut szProxy = CStringW::new();
                        let mut szNoProxy = CStringW::new();
                        let mut dwAttr: DWORD;

                        GetWindowTextW(GetDlgItem(hDlg, (IDC_DOWNLOAD_DIR_EDIT as i32)), szDir.GetBuffer((MAX_PATH as i32)), (MAX_PATH as i32));
                        szDir.ReleaseBuffer();

                        GetWindowTextW(
                            GetDlgItem(hDlg, (IDC_SOURCE_URL as i32)), szSource.GetBuffer((INTERNET_MAX_URL_LENGTH as i32)),
                            (INTERNET_MAX_URL_LENGTH as i32));
                        szSource.ReleaseBuffer();

                        GetWindowTextW(GetDlgItem(hDlg, (IDC_PROXY_SERVER as i32)), szProxy.GetBuffer((MAX_PATH as i32)), (MAX_PATH as i32));
                        szProxy.ReleaseBuffer();
                        core::ptr::copy_nonoverlapping(szProxy.GetString(), NewSettingsInfo.szProxyServer.as_mut_ptr(), (szProxy.GetLength() + 1) as usize); // CStringW::CopyChars == memcpy

                        GetWindowTextW(GetDlgItem(hDlg, (IDC_NO_PROXY_FOR as i32)), szNoProxy.GetBuffer((MAX_PATH as i32)), (MAX_PATH as i32));
                        szNoProxy.ReleaseBuffer();
                        core::ptr::copy_nonoverlapping(szNoProxy.GetString(), NewSettingsInfo.szNoProxyFor.as_mut_ptr(), (szNoProxy.GetLength() + 1) as usize); // CStringW::CopyChars == memcpy

                        core::ptr::copy_nonoverlapping(szDir.GetString(), NewSettingsInfo.szDownloadDir.as_mut_ptr(), (szDir.GetLength() + 1) as usize); // CStringW::CopyChars == memcpy
                        dwAttr = GetFileAttributesW(szDir.GetString());
                        if dwAttr == INVALID_FILE_ATTRIBUTES || (dwAttr & FILE_ATTRIBUTE_DIRECTORY) == 0
                        {
                            let mut szMsgText = CStringW::new();
                            szMsgText.LoadStringW(IDS_CHOOSE_FOLDER_ERROR);

                            if MessageBoxW(hDlg, szMsgText.GetString(), core::ptr::null_mut(), MB_YESNO) == (IDYES as i32)
                            {
                                if CreateDirectoryW(szDir.GetString(), core::ptr::null_mut()) == 0
                                {
                                    ErrorBox__0(hDlg, GetLastError());
                                    break 'sw;
                                }
                            }
                            else
                            {
                                SetFocus(GetDlgItem(hDlg, (IDC_DOWNLOAD_DIR_EDIT as i32)));
                                break 'sw;
                            }
                        }

                        if NewSettingsInfo.bUseSource != 0 && IsUrlValid(szSource.GetString()) == 0
                        {
                            let mut szMsgText = CStringW::new();
                            szMsgText.LoadStringW(IDS_URL_INVALID);

                            MessageBoxW(hDlg, szMsgText.GetString(), core::ptr::null_mut(), MB_OK);
                            SetFocus(GetDlgItem(hDlg, (IDC_SOURCE_URL as i32)));
                            break 'sw;
                        }
                        else
                        {
                            core::ptr::copy_nonoverlapping(szSource.GetString(), NewSettingsInfo.szSourceURL.as_mut_ptr(), (szSource.GetLength() + 1) as usize); // CStringW::CopyChars == memcpy
                        }

                        if SettingsInfo.bSmallIcons != NewSettingsInfo.bSmallIcons
                        {
                            SendMessageW(hMainWnd, WM_SETTINGCHANGE, (SPI_SETICONMETRICS) as WPARAM, 0); // Note: WM_SETTINGCHANGE cannot be posted
                            PostMessageW(hMainWnd, WM_COMMAND, (ID_REFRESH as _), 0);
                        }
                        core::ptr::copy_nonoverlapping(
                            core::ptr::addr_of!(*NewSettingsInfo),
                            core::ptr::addr_of_mut!(SettingsInfo),
                            1,
                        );
                        SaveSettings(GetParent(hDlg), core::ptr::addr_of_mut!(SettingsInfo));
                        EndDialog(hDlg, LOWORD!(wParam as DWORD) as INT_PTR);
                    }

                    IDCANCEL => {
                        EndDialog(hDlg, LOWORD!(wParam as DWORD) as INT_PTR);
                    }

                    _ => {}
                }
            }
        }

        WM_NOTIFY => {
            let nmia: &mut NMITEMACTIVATE = &mut *(lParam as *mut NMITEMACTIVATE);
            if wParam == (IDC_GENERALLIST as _) && nmia.hdr.code == NM_CLICK
            {
                let mut lvhti: LVHITTESTINFO = core::mem::zeroed();
                lvhti.pt = nmia.ptAction;
                if nmia.iItem != -1 && ListView_HitTest!(nmia.hdr.hwndFrom, core::ptr::addr_of_mut!(lvhti)) != -1
                {
                    if (lvhti.flags & (LVHT_ONITEMICON | LVHT_ONITEMLABEL)) != 0
                    {
                        ListView_SetCheckState!(nmia.hdr.hwndFrom, nmia.iItem,
                            if ListView_GetCheckState!(nmia.hdr.hwndFrom, nmia.iItem) != 0 { 0 } else { 1 });
                    }
                }
            }
        }

        _ => {}
    }

    return (FALSE) as INT_PTR;
}
#[no_mangle]
pub unsafe extern "C" fn CreateSettingsDlg(hwnd: HWND) {
    let mut NewSettingsInfo: SETTINGS_INFO = {
        let mut tmp: SETTINGS_INFO = SETTINGS_INFO__make_0();
        tmp = core::ptr::read(core::ptr::addr_of!(SettingsInfo));
        tmp
    };
    g_pNewSettingsInfo = core::ptr::addr_of_mut!(NewSettingsInfo);

    DialogBoxW!(hInst, MAKEINTRESOURCEW!(IDD_SETTINGS_DIALOG), hwnd,
        // prelude's DLGPROC is spelled over ntdll_sys's HWND__, the island's over types_arch's: same ABI
        Some(core::mem::transmute::<unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> INT_PTR, _>(SettingsDlgProc)));
}
