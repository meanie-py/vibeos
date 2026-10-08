// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_28C3B008BDBC: [u16; 11] = [
    0x0049u16, 0x0073, 0x0053, 0x0068, 0x006F, 0x0072, 0x0074, 0x0063, 0x0075, 0x0074,
    0x0000,
];
pub static RUSTCONV_WSTR_6D1618CC9381: [u16; 4] = [0x0079u16, 0x0065, 0x0073, 0x0000];
pub static RUSTCONV_WSTR_731D7EC48EEF: [u16; 3] = [0x0020u16, 0x0009, 0x0000];
pub static RUSTCONV_WSTR_8B3F999AF209: [u16; 10] = [0x005Cu16, 0x002F, 0x003A, 0x002A, 0x003F, 0x0022, 0x003C, 0x003E, 0x007C, 0x0000];
pub static RUSTCONV_WSTR_914188520D5C: [u16; 5] = [0x0077u16, 0x0077, 0x0077, 0x002E, 0x0000];
pub static RUSTCONV_WSTR_C186F67FBF2C: [u16; 5] = [0x002Eu16, 0x0075, 0x0072, 0x006C, 0x0000];
pub static RUSTCONV_WSTR_D30E8899E227: [u16; 12] = [
    0x0073u16, 0x0068, 0x0065, 0x006C, 0x006C, 0x0033, 0x0032, 0x002E, 0x0064, 0x006C,
    0x006C, 0x0000,
];
pub static RUSTCONV_WSTR_FE706C396A0C: [u16; 5] = [0x002Eu16, 0x006C, 0x006E, 0x006B, 0x0000];

use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn IsShortcut(hKey: HKEY) -> BOOL {
    let mut Value: [WCHAR; 10] = [0; 10];
    let mut Size: DWORD;
    let mut Type: DWORD = core::mem::zeroed();

    Size = core::mem::size_of::<[WCHAR; 10]>() as DWORD;
    if (RegQueryValueExW(
        hKey,
        RUSTCONV_WSTR_28C3B008BDBC.as_ptr(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(Type),
        core::ptr::addr_of_mut!(Value) as *mut WCHAR as LPBYTE,
        core::ptr::addr_of_mut!(Size),
    )) as u32 != ERROR_SUCCESS
    {
        return (FALSE) as BOOL;
    }

    if Type != REG_SZ {
        return (FALSE) as BOOL;
    }

    return (_wcsicmp(
        core::ptr::addr_of_mut!(Value) as *mut WCHAR,
        RUSTCONV_WSTR_6D1618CC9381.as_ptr(),
    ) == 0) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn IsExtensionAShortcut(lpExtension: LPWSTR) -> BOOL {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut Buffer: [WCHAR; 100] = [0; 100];
    let mut Size: DWORD = 0;
    let mut Type: DWORD = 0;

    if (RegOpenKeyExW(HKEY_CLASSES_ROOT, lpExtension as LPCWSTR, 0 as DWORD, KEY_READ as REGSAM, &raw mut hKey)) as u32 != ERROR_SUCCESS {
        return (FALSE) as BOOL;
    }

    if IsShortcut(hKey) != 0 {
        RegCloseKey(hKey);
        return (TRUE) as BOOL;
    }

    Size = core::mem::size_of::<[WCHAR; 100]>() as DWORD;
    if (RegQueryValueEx(hKey, core::ptr::null::<WCHAR>(), core::ptr::null_mut::<DWORD>(), &raw mut Type, &raw mut Buffer as *mut WCHAR as LPBYTE, &raw mut Size)) as u32 != ERROR_SUCCESS || Type != REG_SZ {
        RegCloseKey(hKey);
        return (FALSE) as BOOL;
    }

    RegCloseKey(hKey);

    if (RegOpenKeyExW(HKEY_CLASSES_ROOT, &raw mut Buffer as *mut WCHAR as LPCWSTR, 0 as DWORD, KEY_READ as REGSAM, &raw mut hKey)) as u32 != ERROR_SUCCESS {
        return (FALSE) as BOOL;
    }

    if IsShortcut(hKey) != 0 {
        RegCloseKey(hKey);
        return (TRUE) as BOOL;
    }

    RegCloseKey(hKey);
    return (FALSE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn CreateShortcut(pContext: PCREATE_LINK_CONTEXT) -> BOOL {
    let mut pShellLink: *mut IShellLinkW = core::ptr::null_mut::<IShellLinkW>();
    let mut pPersistFile: *mut IPersistFile = core::ptr::null_mut::<IPersistFile>();
    let mut hr: HRESULT = 0;

    hr = CoCreateInstance(
        core::ptr::addr_of!(CLSID_ShellLink),
        core::ptr::null_mut::<IUnknown>(),
        CLSCTX_ALL as DWORD,
        core::ptr::addr_of!(IID_IShellLinkW),
        &raw mut pShellLink as *mut LPVOID,
    );
    if hr < (0 as HRESULT) {
        return FALSE as BOOL;
    }

    hr = IUnknown_QueryInterface(
        pShellLink as *mut IUnknown,
        core::ptr::addr_of!(IID_IPersistFile),
        &raw mut pPersistFile as *mut *mut ::core::ffi::c_void,
    );
    if hr < (0 as HRESULT) {
        IUnknown_Release(pShellLink as *mut IUnknown);
        return FALSE as BOOL;
    }

    if IsExtensionAShortcut(PathFindExtensionW(
        &raw mut (*pContext).szTarget as *mut WCHAR as LPCWSTR,
    )) != 0
    {
        hr = (*(*pPersistFile).lpVtbl)
            .Load
            .unwrap_unchecked()(
            pPersistFile,
            &raw mut (*pContext).szTarget as *mut WCHAR as LPCOLESTR,
            STGM_READ as DWORD,
        );
    } else {
        if !(*pContext).pidlTarget.is_null() {
            (*(*pShellLink).lpVtbl)
                .SetIDList
                .unwrap_unchecked()(
                pShellLink,
                (*pContext).pidlTarget as PCIDLIST_ABSOLUTE,
            );
        } else {
            (*(*pShellLink).lpVtbl)
                .SetPath
                .unwrap_unchecked()(
                pShellLink,
                &raw mut (*pContext).szTarget as *mut WCHAR as LPCWSTR,
            );
        }

        if (*pContext).szArguments[0 as usize] != 0 {
            (*(*pShellLink).lpVtbl)
                .SetArguments
                .unwrap_unchecked()(
                pShellLink,
                &raw mut (*pContext).szArguments as *mut WCHAR as LPCWSTR,
            );
        }

        if (*pContext).szDescription[0 as usize] != 0 {
            (*(*pShellLink).lpVtbl)
                .SetDescription
                .unwrap_unchecked()(
                pShellLink,
                &raw mut (*pContext).szDescription as *mut WCHAR as LPCWSTR,
            );
        }
    }

    if hr >= (0 as HRESULT) {
        hr = (*(*pPersistFile).lpVtbl)
            .Save
            .unwrap_unchecked()(
            pPersistFile,
            &raw mut (*pContext).szLinkName as *mut WCHAR as LPCOLESTR,
            TRUE as BOOL,
        );
    }

    IUnknown_Release(pPersistFile as *mut IUnknown);
    IUnknown_Release(pShellLink as *mut IUnknown);
    return (hr >= (0 as HRESULT)) as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn CreateInternetShortcut(pContext: PCREATE_LINK_CONTEXT) -> BOOL {
    let mut pURL: *mut IUniformResourceLocatorW = core::ptr::null_mut();
    let mut pPersistFile: *mut IPersistFile = core::ptr::null_mut();
    let mut hr: HRESULT;
    let mut szPath: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    GetFullPathNameW(
        core::ptr::addr_of_mut!((*pContext).szLinkName) as LPCWSTR,
        _countof!(szPath) as DWORD,
        core::ptr::addr_of_mut!(szPath) as LPWSTR,
        core::ptr::null_mut(),
    );

    hr = CoCreateInstance(
        core::ptr::addr_of!(CLSID_InternetShortcut),
        core::ptr::null_mut(),
        CLSCTX_ALL as DWORD,
        core::ptr::addr_of!(IID_IUniformResourceLocatorW),
        core::ptr::addr_of_mut!(pURL) as *mut LPVOID,
    );
    if hr < 0 as HRESULT {
        return (FALSE) as BOOL;
    }

    hr = IUnknown_QueryInterface(
        pURL,
        core::ptr::addr_of!(IID_IPersistFile),
        core::ptr::addr_of_mut!(pPersistFile) as *mut *mut core::ffi::c_void,
    );
    if hr < 0 as HRESULT {
        IUnknown_Release(pURL);
        return (FALSE) as BOOL;
    }

    (*(*pURL).lpVtbl).SetURL.unwrap_unchecked()(
        pURL,
        core::ptr::addr_of_mut!((*pContext).szTarget) as LPCWSTR,
        0 as DWORD,
    );

    hr = (*(*pPersistFile).lpVtbl).Save.unwrap_unchecked()(
        pPersistFile,
        core::ptr::addr_of_mut!(szPath) as LPCOLESTR,
        TRUE as BOOL,
    );

    IUnknown_Release(pPersistFile as _);
    IUnknown_Release(pURL);

    return (hr >= 0 as HRESULT) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn IsInternetLocation(pszLocation: LPCWSTR) -> BOOL {
    return (PathIsURLW(pszLocation) != 0
        || wcsstr(pszLocation, RUSTCONV_WSTR_914188520D5C.as_ptr()) == pszLocation as *mut u16) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn DoConvertNameForFileSystem(mut szName: LPWSTR) {
    let mut pch1: LPWSTR = core::ptr::null_mut();
    let mut pch2: LPWSTR = core::ptr::null_mut();
    pch2 = szName;
    pch1 = pch2;
    while *pch1 != 0 {
        if wcschr(RUSTCONV_WSTR_8B3F999AF209.as_ptr(), *pch1).is_null() {
            /* *pch1 is not an invalid character */
            *pch2 = *pch1;
            pch2 = pch2.offset(1);
        }
        pch1 = pch1.offset(1);
    }
    *pch2 = 0;
}
#[no_mangle]
pub unsafe extern "C" fn DoValidateShortcutName(pContext: PCREATE_LINK_CONTEXT) -> BOOL {
    let mut cch: SIZE_T;
    let mut pch: LPCWSTR;
    let mut pszName: LPCWSTR = core::ptr::addr_of_mut!((*pContext).szDescription) as *mut WCHAR as LPCWSTR;

    if pszName.is_null() || *pszName.add(0 as usize) == 0 {
        return (FALSE) as BOOL;
    }

    cch = wcslen(core::ptr::addr_of_mut!((*pContext).szOrigin) as *mut WCHAR as LPCWSTR) as SIZE_T
        + wcslen(pszName) as SIZE_T
        + 1 as SIZE_T;
    if cch >= MAX_PATH as SIZE_T {
        return (FALSE) as BOOL;
    }

    pch = pszName;
    pch = pszName;
    while *pch != 0 {
        if !wcschr(RUSTCONV_WSTR_8B3F999AF209.as_ptr(), *pch).is_null() {
            /* *pch is an invalid character */
            return (FALSE) as BOOL;
        }
        pch = pch.add(1);
    }

    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "system" fn WelcomeDlgProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut ppsp: LPPROPSHEETPAGEW = core::ptr::null_mut();
    let mut pContext: PCREATE_LINK_CONTEXT = GetWindowLongPtr(hwndDlg, DWLP_USER) as PCREATE_LINK_CONTEXT;
    let mut lppsn: LPPSHNOTIFY = core::ptr::null_mut();
    let mut szPath: [WCHAR; (MAX_PATH * 2) as _] = [0; (MAX_PATH * 2) as _];
    let mut szDisplayName: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut szDesc: [WCHAR; 100] = [0; 100];
    let mut szTitle: [WCHAR; 100] = [0; 100];
    let mut brws: BROWSEINFOW = core::mem::zeroed();
    let mut pidllist: LPITEMIDLIST = core::ptr::null_mut();
    let mut FileInfo: SHFILEINFOW = core::mem::zeroed();

    if uMsg == WM_INITDIALOG {
        ppsp = lParam as LPPROPSHEETPAGEW;
        pContext = (*ppsp).lParam as PCREATE_LINK_CONTEXT;
        SetWindowLongPtr(hwndDlg, DWLP_USER, pContext as LONG_PTR);
        PropSheet_SetWizButtons!(GetParent(hwndDlg), 0);
        SHAutoComplete(GetDlgItem(hwndDlg, (IDC_LINK_LOCATION) as i32), SHACF_DEFAULT);
    } else if uMsg == WM_COMMAND {
        let cmdId = LOWORD!(wParam);
        if (cmdId) as u32 == IDC_LINK_LOCATION {
            if (HIWORD!(wParam)) as u32 == EN_CHANGE {
                /* The text was changed by user. Invalidate pidlTarget. */
                if !(*pContext).pidlTarget.is_null() {
                    CoTaskMemFree((*pContext).pidlTarget as LPVOID);
                    (*pContext).pidlTarget = core::ptr::null_mut();
                }

                if SendDlgItemMessage(hwndDlg, IDC_LINK_LOCATION as c_int, WM_GETTEXTLENGTH, 0, 0) != 0 {
                    PropSheet_SetWizButtons!(GetParent(hwndDlg), PSWIZB_NEXT);
                } else {
                    PropSheet_SetWizButtons!(GetParent(hwndDlg), 0);
                }
            }
        } else if (cmdId) as u32 == IDC_SHORTCUT_BROWSE {
            'cmd: {
                LoadStringW(hApplet, IDS_BROWSE_FOR_TARGET, addr_of_mut!(szTitle) as LPWSTR, (_countof!(szTitle)) as INT);
                ZeroMemory!(addr_of_mut!(brws) as *mut core::ffi::c_void, core::mem::size_of::<BROWSEINFOW>());
                brws.hwndOwner = hwndDlg;
                brws.pidlRoot = core::ptr::null_mut();
                brws.pszDisplayName = addr_of_mut!(szDisplayName) as LPWSTR;
                brws.lpszTitle = addr_of_mut!(szTitle) as LPCWSTR;
                brws.ulFlags = BIF_BROWSEINCLUDEFILES | BIF_NEWDIALOGSTYLE | BIF_SHAREABLE;
                brws.lpfn = None;
                pidllist = SHBrowseForFolderW(addr_of_mut!(brws));
                if pidllist.is_null() {
                    break 'cmd;
                }

                StringCchCopyW(addr_of_mut!((*pContext).szDescription) as LPWSTR, (_countof!((*pContext).szDescription)) as _, addr_of_mut!(szDisplayName) as LPWSTR);

                SHGetPathFromIDListW(pidllist, addr_of_mut!(szPath) as LPWSTR);

                if PathFileExistsW(addr_of_mut!(szPath) as LPCWSTR) != 0 && PathIsRelativeW(addr_of_mut!(szPath) as LPCWSTR) == 0 {
                    SetDlgItemTextW(hwndDlg, (IDC_LINK_LOCATION) as i32, addr_of_mut!(szPath) as LPCWSTR);
                } else {
                    SetDlgItemTextW(hwndDlg, (IDC_LINK_LOCATION) as i32, addr_of_mut!(szDisplayName) as LPCWSTR);
                }

                SendDlgItemMessageW(hwndDlg, (IDC_LINK_LOCATION) as i32, EM_SETSEL, 0, -1);

                if !(*pContext).pidlTarget.is_null() {
                    CoTaskMemFree((*pContext).pidlTarget as LPVOID);
                }
                (*pContext).pidlTarget = pidllist;
            }
        }
    } else if uMsg == WM_NOTIFY {
        lppsn = lParam as LPPSHNOTIFY;
        if (*lppsn).hdr.code == (PSN_SETACTIVE) as u32 {
            SetDlgItemTextW(hwndDlg, (IDC_LINK_LOCATION) as i32, addr_of_mut!((*pContext).szTarget) as LPCWSTR);
        } else if (*lppsn).hdr.code == (PSN_WIZNEXT) as u32 {
            GetDlgItemTextW(hwndDlg, (IDC_LINK_LOCATION) as i32, addr_of_mut!((*pContext).szTarget) as LPWSTR, (_countof!((*pContext).szTarget)) as i32);
            StrTrimW(addr_of_mut!((*pContext).szTarget) as LPWSTR, RUSTCONV_WSTR_731D7EC48EEF.as_ptr());
            ExpandEnvironmentStringsW(addr_of_mut!((*pContext).szTarget) as LPCWSTR, addr_of_mut!(szPath) as LPWSTR, _countof!(szPath));

            if IsInternetLocation(addr_of_mut!(szPath) as LPCWSTR) != 0 { /* The internet location */
                let mut szName: [WCHAR; 128] = [0; 128];
                LoadStringW(hApplet, IDS_NEW_INTERNET_SHORTCUT, addr_of_mut!(szName) as LPWSTR, (_countof!(szName)) as INT);
                StringCchCopyW(addr_of_mut!((*pContext).szDescription) as LPWSTR, (_countof!((*pContext).szDescription)) as _, addr_of_mut!(szName) as LPWSTR);
                (*pContext).szArguments[0] = 0;
                return (FALSE) as INT_PTR;
            }

            if !(*pContext).pidlTarget.is_null() { /* The result of SHBrowseForFolderW */
                SHGetPathFromIDListW((*pContext).pidlTarget, addr_of_mut!((*pContext).szTarget) as LPWSTR);
                (*pContext).szArguments[0] = 0;
                return (FALSE) as INT_PTR;
            }

            /* Otherwise, the target is a command line or pathname */

            /* Split and build args */
            let mut pszArgs: LPWSTR = PathGetArgsW(addr_of_mut!(szPath) as LPCWSTR);
            if !pszArgs.is_null() && (pszArgs > (addr_of_mut!(szPath) as LPWSTR)) {
                PathRemoveArgsW(addr_of_mut!(szPath) as LPWSTR);
                StringCchCopyW(addr_of_mut!((*pContext).szArguments) as LPWSTR, (_countof!((*pContext).szArguments)) as _, pszArgs);
            } else {
                (*pContext).szArguments[0] = 0;
            }

            /* Find the file */
            if PathFindOnPathExW(addr_of_mut!(szPath) as LPWSTR, core::ptr::null_mut(), WHICH_DEFAULT) == 0
                && PathFileExistsW(addr_of_mut!(szPath) as LPCWSTR) == 0
            {
                /* Not found */
                SendDlgItemMessageW(hwndDlg, (IDC_LINK_LOCATION) as i32, EM_SETSEL, 0, -1);

                LoadStringW(hApplet, IDS_CREATE_SHORTCUT, addr_of_mut!(szDesc) as LPWSTR, (_countof!(szDesc)) as INT);
                LoadStringW(hApplet, IDS_ERROR_NOT_FOUND, addr_of_mut!(szPath) as LPWSTR, (_countof!(szPath)) as INT);

                let mut szError: [WCHAR; (MAX_PATH + 100) as _] = [0; (MAX_PATH + 100) as _];
                StringCchPrintfW!(addr_of_mut!(szError) as LPWSTR, _countof!(szError), addr_of_mut!(szPath) as LPCWSTR, addr_of_mut!((*pContext).szTarget) as LPCWSTR);
                MessageBoxW(hwndDlg, addr_of_mut!(szError) as LPCWSTR, addr_of_mut!(szDesc) as LPCWSTR, MB_ICONERROR);

                /* Prevent the wizard to go next */
                SetWindowLongPtr(hwndDlg, DWLP_MSGRESULT, -1);
                return (TRUE) as INT_PTR;
            }

            /* Rebuild target */
            if PathIsRelativeW(addr_of_mut!(szPath) as LPCWSTR) != 0 {
                GetFullPathNameW(addr_of_mut!(szPath) as LPCWSTR, _countof!((*pContext).szTarget), addr_of_mut!((*pContext).szTarget) as LPWSTR, core::ptr::null_mut());
            } else {
                StringCchCopyW(addr_of_mut!((*pContext).szTarget) as LPWSTR, (_countof!((*pContext).szTarget)) as _, addr_of_mut!(szPath) as LPCWSTR);
            }

            /* Get display name */
            FileInfo.szDisplayName[0] = UNICODE_NULL;
            if SHGetFileInfoW(addr_of_mut!((*pContext).szTarget) as LPCWSTR, 0, addr_of_mut!(FileInfo), core::mem::size_of::<SHFILEINFOW>() as UINT, SHGFI_DISPLAYNAME) != 0 {
                StringCchCopyW(addr_of_mut!((*pContext).szDescription) as LPWSTR, (_countof!((*pContext).szDescription)) as _, addr_of_mut!(FileInfo.szDisplayName) as LPCWSTR);
            }
        } else if (*lppsn).hdr.code == (PSN_RESET) as u32 && (*lppsn).lParam == 0 {
            /* The user has clicked [Cancel] */
            DeleteFileW(addr_of_mut!((*pContext).szOldFile) as LPCWSTR);
            SHChangeNotify((SHCNE_DELETE) as LONG, SHCNF_PATHW, addr_of_mut!((*pContext).szOldFile) as LPCVOID, core::ptr::null());
        }
    }
    return (FALSE) as INT_PTR;
}
#[no_mangle]
pub unsafe extern "system" fn FinishDlgProc(
    mut hwndDlg: HWND,
    mut uMsg: UINT,
    mut wParam: WPARAM,
    mut lParam: LPARAM,
) -> INT_PTR {
    let mut ppsp: LPPROPSHEETPAGEW = core::ptr::null_mut();
    let mut pContext: PCREATE_LINK_CONTEXT = core::ptr::null_mut();
    let mut lppsn: LPPSHNOTIFY = core::ptr::null_mut();
    let mut szText: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut szPath: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut szMessage: [WCHAR; 128] = [0; 128];

    match uMsg {
        WM_INITDIALOG => {
            ppsp = lParam as LPPROPSHEETPAGEW;
            pContext = (*ppsp).lParam as PCREATE_LINK_CONTEXT;
            SetWindowLongPtr(hwndDlg, DWLP_USER, pContext as LONG_PTR);
            PropSheet_SetWizButtons!(
                GetParent(hwndDlg),
                PSWIZB_BACK | PSWIZB_FINISH,
            );
        }
        WM_COMMAND => {
            match (HIWORD!(wParam)) as u32 {
                EN_CHANGE => {
                    if SendDlgItemMessage(
                        hwndDlg,
                        IDC_SHORTCUT_NAME as c_int,
                        WM_GETTEXTLENGTH,
                        0,
                        0,
                    ) != 0
                    {
                        GetDlgItemTextW(
                            hwndDlg,
                            (IDC_SHORTCUT_NAME) as i32,
                            core::ptr::addr_of_mut!(szText) as LPWSTR,
                            (_countof!(szText)) as i32,
                        );
                        StrTrimW(
                            core::ptr::addr_of_mut!(szText) as LPWSTR,
                            RUSTCONV_WSTR_731D7EC48EEF.as_ptr(),
                        );
                        if szText[0] != 0 {
                            PropSheet_SetWizButtons!(
                                GetParent(hwndDlg),
                                PSWIZB_BACK | PSWIZB_FINISH,
                            );
                        } else {
                            PropSheet_SetWizButtons!(
                                GetParent(hwndDlg),
                                PSWIZB_BACK,
                            );
                        }
                    } else {
                        PropSheet_SetWizButtons!(
                            GetParent(hwndDlg),
                            PSWIZB_BACK,
                        );
                    }
                }
                _ => {}
            }
        }
        WM_NOTIFY => {
            lppsn = lParam as LPPSHNOTIFY;
            pContext = GetWindowLongPtr(hwndDlg, DWLP_USER) as PCREATE_LINK_CONTEXT;
            if (*lppsn).hdr.code == (PSN_SETACTIVE) as u32 {
                /* Remove invalid characters */
                DoConvertNameForFileSystem(core::ptr::addr_of_mut!((*pContext).szDescription) as LPWSTR);
                PathCleanupSpec(
                    core::ptr::null_mut(),
                    core::ptr::addr_of_mut!((*pContext).szDescription) as LPWSTR,
                );

                /* Is it empty? (rare case) */
                if (*pContext).szDescription[0] == 0 {
                    let mut hShell32: HMODULE = GetModuleHandleW(RUSTCONV_WSTR_D30E8899E227.as_ptr());
                    LoadStringW(
                        hShell32,
                        IDS_NEWITEMFORMAT,
                        core::ptr::addr_of_mut!(szText) as LPWSTR,
                        (_countof!(szText)) as INT,
                    );
                    LoadStringW(
                        hShell32,
                        IDS_LNK_FILE,
                        core::ptr::addr_of_mut!(szMessage) as LPWSTR,
                        (_countof!(szMessage)) as INT,
                    );
                    StringCchPrintfW!(
                        core::ptr::addr_of_mut!((*pContext).szDescription) as LPWSTR,
                        _countof!((*pContext).szDescription),
                        core::ptr::addr_of_mut!(szText) as LPWSTR,
                        core::ptr::addr_of_mut!(szMessage) as LPWSTR,
                    );
                }

                /* Build a path from szOldFile */
                StringCchCopyW(
                    core::ptr::addr_of_mut!(szText) as LPWSTR,
                    (_countof!(szText)) as _,
                    core::ptr::addr_of_mut!((*pContext).szOldFile) as LPCWSTR,
                );
                PathRemoveFileSpecW(core::ptr::addr_of_mut!(szText) as LPWSTR);

                /* Rename duplicate if necessary */
                PathAddExtensionW(
                    core::ptr::addr_of_mut!((*pContext).szDescription) as LPWSTR,
                    if IsInternetLocation(core::ptr::addr_of_mut!((*pContext).szTarget) as LPCWSTR) != 0 {
                        RUSTCONV_WSTR_C186F67FBF2C.as_ptr()
                    } else {
                        RUSTCONV_WSTR_FE706C396A0C.as_ptr()
                    },
                );
                PathYetAnotherMakeUniqueName(
                    core::ptr::addr_of_mut!(szPath) as LPWSTR,
                    core::ptr::addr_of_mut!(szText) as LPCWSTR,
                    core::ptr::null(),
                    core::ptr::addr_of_mut!((*pContext).szDescription) as LPCWSTR,
                );
                StringCchCopyW(
                    core::ptr::addr_of_mut!((*pContext).szDescription) as LPWSTR,
                    (_countof!((*pContext).szDescription)) as _,
                    PathFindFileNameW(core::ptr::addr_of_mut!(szPath) as LPCWSTR),
                );
                PathRemoveExtensionW(core::ptr::addr_of_mut!((*pContext).szDescription) as LPWSTR);

                SetDlgItemTextW(
                    hwndDlg,
                    (IDC_SHORTCUT_NAME) as i32,
                    core::ptr::addr_of_mut!((*pContext).szDescription) as LPCWSTR,
                );
                SendDlgItemMessageW(
                    hwndDlg,
                    (IDC_SHORTCUT_NAME) as i32,
                    EM_SETSEL,
                    0,
                    -1,
                );
                SetFocus(GetDlgItem(hwndDlg, (IDC_SHORTCUT_NAME) as i32));
            } else if (*lppsn).hdr.code == (PSN_WIZFINISH) as u32 {
                GetDlgItemTextW(
                    hwndDlg,
                    (IDC_SHORTCUT_NAME) as i32,
                    core::ptr::addr_of_mut!((*pContext).szDescription) as LPWSTR,
                    (_countof!((*pContext).szDescription)) as i32,
                );
                StrTrimW(
                    core::ptr::addr_of_mut!((*pContext).szDescription) as LPWSTR,
                    RUSTCONV_WSTR_731D7EC48EEF.as_ptr(),
                );

                if DoValidateShortcutName(pContext) == 0 {
                    SendDlgItemMessageW(
                        hwndDlg,
                        (IDC_SHORTCUT_NAME) as i32,
                        EM_SETSEL,
                        0,
                        -1,
                    );

                    LoadStringW(
                        hApplet,
                        IDS_INVALID_NAME,
                        core::ptr::addr_of_mut!(szMessage) as LPWSTR,
                        (_countof!(szMessage)) as INT,
                    );
                    MessageBoxW(
                        hwndDlg,
                        core::ptr::addr_of_mut!(szMessage) as LPCWSTR,
                        core::ptr::null(),
                        MB_ICONERROR,
                    );

                    /* prevent the wizard to go next */
                    SetWindowLongPtr(hwndDlg, DWLP_MSGRESULT, -1);
                    return (TRUE) as INT_PTR;
                }

                /* if old shortcut file exists, then delete it now */
                DeleteFileW(core::ptr::addr_of_mut!((*pContext).szOldFile) as LPCWSTR);
                SHChangeNotify(
                    (SHCNE_DELETE) as LONG,
                    SHCNF_PATHW,
                    (core::ptr::addr_of_mut!((*pContext).szOldFile) as LPCWSTR).cast(),
                    core::ptr::null(),
                );

                if IsInternetLocation(core::ptr::addr_of_mut!((*pContext).szTarget) as LPCWSTR) != 0 {
                    /* internet */
                    StringCchCopyW(
                        core::ptr::addr_of_mut!((*pContext).szLinkName) as LPWSTR,
                        (_countof!((*pContext).szLinkName)) as _,
                        core::ptr::addr_of_mut!((*pContext).szOrigin) as LPCWSTR,
                    );
                    PathAppendW(
                        core::ptr::addr_of_mut!((*pContext).szLinkName) as LPWSTR,
                        core::ptr::addr_of_mut!((*pContext).szDescription) as LPCWSTR,
                    );

                    /* change extension if any */
                    PathRemoveExtensionW(core::ptr::addr_of_mut!((*pContext).szLinkName) as LPWSTR);
                    PathAddExtensionW(
                        core::ptr::addr_of_mut!((*pContext).szLinkName) as LPWSTR,
                        RUSTCONV_WSTR_C186F67FBF2C.as_ptr(),
                    );

                    if CreateInternetShortcut(pContext) == 0 {
                        LoadStringW(
                            hApplet,
                            IDS_CANTMAKEINETSHORTCUT,
                            core::ptr::addr_of_mut!(szMessage) as LPWSTR,
                            (_countof!(szMessage)) as INT,
                        );
                        MessageBoxW(
                            hwndDlg,
                            core::ptr::addr_of_mut!(szMessage) as LPCWSTR,
                            core::ptr::null(),
                            MB_ICONERROR,
                        );
                    }
                } else {
                    /* file */
                    StringCchCopyW(
                        core::ptr::addr_of_mut!((*pContext).szLinkName) as LPWSTR,
                        (_countof!((*pContext).szLinkName)) as _,
                        core::ptr::addr_of_mut!((*pContext).szOrigin) as LPCWSTR,
                    );
                    PathAppendW(
                        core::ptr::addr_of_mut!((*pContext).szLinkName) as LPWSTR,
                        core::ptr::addr_of_mut!((*pContext).szDescription) as LPCWSTR,
                    );

                    /* change extension if any */
                    PathRemoveExtensionW(core::ptr::addr_of_mut!((*pContext).szLinkName) as LPWSTR);
                    PathAddExtensionW(
                        core::ptr::addr_of_mut!((*pContext).szLinkName) as LPWSTR,
                        RUSTCONV_WSTR_FE706C396A0C.as_ptr(),
                    );

                    /* Don't set a comment */
                    (*pContext).szDescription[0] = UNICODE_NULL;

                    if CreateShortcut(pContext) == 0 {
                        let mut szMessage: [WCHAR; 128] = [0; 128];
                        LoadStringW(
                            hApplet,
                            IDS_CANTMAKESHORTCUT,
                            core::ptr::addr_of_mut!(szMessage) as LPWSTR,
                            (_countof!(szMessage)) as INT,
                        );
                        MessageBoxW(
                            hwndDlg,
                            core::ptr::addr_of_mut!(szMessage) as LPCWSTR,
                            core::ptr::null(),
                            MB_ICONERROR,
                        );
                    }
                }
            } else if (*lppsn).hdr.code == (PSN_RESET) as u32 && (*lppsn).lParam == 0 {
                /* The user has clicked [Cancel] */
                DeleteFileW(core::ptr::addr_of_mut!((*pContext).szOldFile) as LPCWSTR);
                SHChangeNotify(
                    (SHCNE_DELETE) as LONG,
                    SHCNF_PATHW,
                    (core::ptr::addr_of_mut!((*pContext).szOldFile) as LPCWSTR).cast(),
                    core::ptr::null(),
                );
            }

        }
        _ => {}
    }
    return (FALSE) as INT_PTR;
}
unsafe extern "system" fn PropSheetProc(hwndDlg: HWND, uMsg: UINT, lParam: LPARAM) -> i32 {
    let mut hIcon: HICON;
    match uMsg {
        PSCB_INITIALIZED => {
            hIcon = LoadIconW(hApplet, MAKEINTRESOURCEW!(IDI_APPINETICO));
            SendMessageW(hwndDlg, WM_SETICON, ICON_BIG as WPARAM, hIcon as LPARAM);
            // break;
        }
        _ => {}
    }
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn ShowCreateShortcutWizard(hwndCPl: HWND, szPath: LPCWSTR) -> LONG {
    let mut psh: PROPSHEETHEADERW = core::mem::zeroed();
    let mut ahpsp: [HPROPSHEETPAGE; 2] = [core::ptr::null_mut(); 2];
    let mut psp: PROPSHEETPAGEW = core::mem::zeroed();
    let mut nPages: UINT = 0;
    let mut nLength: UINT = 0;
    let mut pContext: PCREATE_LINK_CONTEXT = core::ptr::null_mut();
    let mut szMessage: [WCHAR; 128] = [0; 128];
    let mut pch: LPWSTR = core::ptr::null_mut();

    pContext = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<CREATE_LINK_CONTEXT>() as SIZE_T) as PCREATE_LINK_CONTEXT;
    if pContext.is_null() {
        /* no memory */
        LoadStringW(hApplet, IDS_NO_MEMORY, szMessage.as_mut_ptr(), (_countof!(szMessage)) as INT);
        MessageBoxW(hwndCPl, szMessage.as_ptr(), core::ptr::null(), MB_ICONERROR);
        return (FALSE) as LONG;
    }

    nLength = wcslen(szPath) as UINT;
    if nLength == 0 {
        HeapFree(GetProcessHeap(), 0, pContext as LPVOID);

        /* no directory given */
        LoadStringW(hApplet, IDS_NO_DIRECTORY, szMessage.as_mut_ptr(), (_countof!(szMessage)) as INT);
        MessageBoxW(hwndCPl, szMessage.as_ptr(), core::ptr::null(), MB_ICONERROR);
        return (FALSE) as LONG;
    }

    if PathFileExistsW(szPath) == 0 {
        HeapFree(GetProcessHeap(), 0, pContext as LPVOID);

        /* invalid path */
        LoadStringW(hApplet, IDS_INVALID_PATH, szMessage.as_mut_ptr(), (_countof!(szMessage)) as INT);
        MessageBoxW(hwndCPl, szMessage.as_ptr(), core::ptr::null(), MB_ICONERROR);
        return (FALSE) as LONG;
    }

    /* build the pContext->szOrigin and pContext->szOldFile */
    if PathIsDirectoryW(szPath) != 0 {
        StringCchCopyW((*pContext).szOrigin.as_mut_ptr(), (_countof!((*pContext).szOrigin)) as _, szPath);
        (*pContext).szOldFile[0 as usize] = 0;
    } else {
        StringCchCopyW((*pContext).szOrigin.as_mut_ptr(), (_countof!((*pContext).szOrigin)) as _, szPath);
        pch = PathFindFileNameW((*pContext).szOrigin.as_mut_ptr());
        if !pch.is_null() && *pch != 0 {
            *pch = 0;
        }

        StringCchCopyW((*pContext).szOldFile.as_mut_ptr(), (_countof!((*pContext).szOldFile)) as _, szPath);

        pch = PathFindFileNameW(szPath);
        if !pch.is_null() && *pch != 0 {
            /* build szDescription */
            StringCchCopyW((*pContext).szDescription.as_mut_ptr(), (_countof!((*pContext).szDescription)) as _, pch);
            *pch = 0;

            pch = PathFindExtensionW((*pContext).szDescription.as_mut_ptr());
            *pch = 0;
        }
    }
    PathAddBackslashW((*pContext).szOrigin.as_mut_ptr());

    /* Create the Welcome page */
    psp.dwSize = core::mem::size_of::<PROPSHEETPAGEW>() as DWORD;
    psp.dwFlags = PSP_DEFAULT | PSP_HIDEHEADER;
    psp.hInstance = hApplet;
    psp.pfnDlgProc = Some(WelcomeDlgProc);
    psp.__bindgen_anon_1.pszTemplate = MAKEINTRESOURCEW!(IDD_SHORTCUT_LOCATION);
    psp.lParam = pContext as LPARAM;
    let idx0 = nPages as usize;
    nPages += 1;
    ahpsp[idx0] = CreatePropertySheetPage(core::ptr::addr_of_mut!(psp));

    /* Create the Finish page */
    psp.dwFlags = PSP_DEFAULT | PSP_HIDEHEADER;
    psp.pfnDlgProc = Some(FinishDlgProc);
    psp.__bindgen_anon_1.pszTemplate = MAKEINTRESOURCEW!(IDD_SHORTCUT_FINISH);
    let idx1 = nPages as usize;
    nPages += 1;
    ahpsp[idx1] = CreatePropertySheetPage(core::ptr::addr_of_mut!(psp));

    /* Create the property sheet */
    psh.dwSize = core::mem::size_of::<PROPSHEETHEADERW>() as DWORD;
    psh.dwFlags = PSH_WIZARD97 | PSH_WATERMARK | PSH_USEICONID | PSH_USECALLBACK;
    psh.hInstance = hApplet;
    psh.__bindgen_anon_1.pszIcon = MAKEINTRESOURCEW!(IDI_APPINETICO);
    psh.hwndParent = core::ptr::null_mut();
    psh.nPages = nPages;
    psh.__bindgen_anon_2.nStartPage = 0;
    psh.__bindgen_anon_3.phpage = ahpsp.as_mut_ptr();
    psh.__bindgen_anon_4.pszbmWatermark = MAKEINTRESOURCEW!(IDB_SHORTCUT);
    psh.pfnCallback = Some(PropSheetProc);

    /* Display the wizard */
    PropertySheet(core::ptr::addr_of_mut!(psh));

    CoTaskMemFree((*pContext).pidlTarget as LPVOID);
    HeapFree(GetProcessHeap(), 0, pContext as LPVOID);
    return (TRUE) as LONG;
}

#[no_mangle]
pub unsafe extern "system" fn NewLinkHereW(hwndCPl: HWND, uMsg: UINT, lParam1: LPARAM, lParam2: LPARAM) -> LONG {
    InitCommonControls();
    return ShowCreateShortcutWizard(hwndCPl, lParam1 as LPCWSTR);
}

#[no_mangle]
pub unsafe extern "system" fn NewLinkHereA(hwndCPl: HWND, uMsg: UINT, lParam1: LPARAM, lParam2: LPARAM) -> LONG {
    let mut szFile: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    if MultiByteToWideChar(CP_ACP, 0, lParam1 as LPCSTR, -(1) as i32, core::ptr::addr_of_mut!(szFile) as LPWSTR, _countof!(szFile) as i32) != 0 {
        return NewLinkHereW(hwndCPl, uMsg, core::ptr::addr_of_mut!(szFile) as *mut WCHAR as LPARAM, lParam2);
    }
    return -(1) as LONG;
}
