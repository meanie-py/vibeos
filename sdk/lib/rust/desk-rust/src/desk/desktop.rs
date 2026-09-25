// rustconv-assembly: whole-tu
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const FCIDM_DESKBROWSER_REFRESH: u32 = 0xA220;

pub const IDS_TITLE_MYCOMP: u32 = 30386;

pub const IDS_TITLE_MYNET: u32 = 30387;

pub const IDS_TITLE_BIN_1: u32 = 30388;

pub const IDS_TITLE_BIN_0: u32 = 30389;

pub const IDS_PERSONAL: u32 = 9227;

pub static szHideDesktopIcons: [WCHAR; 69] = wide!("Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\HideDesktopIcons\\");

pub static szClassicStartMenu: [WCHAR; 17] = wide!("ClassicStartMenu");

pub static szNewStartPanel: [WCHAR; 14] = wide!("NewStartPanel");

#[repr(C)]
pub struct DesktopIconEntry {
    pub CLSID: LPCTSTR,
    pub Checkbox: u32,
}

pub static DesktopIcons: SyncTable<[DesktopIconEntry; 4]> = SyncTable([
    DesktopIconEntry {
        CLSID: w!("{450D8FBA-AD25-11D0-98A8-0800361B1103}"),
        Checkbox: IDC_ICONS_MYDOCS,
    },
    DesktopIconEntry {
        CLSID: w!("{208D2C60-3AEA-1069-A2D7-08002B30309D}"),
        Checkbox: IDC_ICONS_MYNET,
    },
    DesktopIconEntry {
        CLSID: w!("{20D04FE0-3AEA-1069-A2D8-08002B30309D}"),
        Checkbox: IDC_ICONS_MYCOMP,
    },
    DesktopIconEntry {
        CLSID: w!("{871C5380-42A0-1069-A2EA-08002B30309D}"),
        Checkbox: IDC_ICONS_INTERNET,
    },
]);

pub static szUserClass: [WCHAR; 58] = wide!("Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\CLSID\\");

pub static szSysClass: [WCHAR; 7] = wide!("CLSID\\");

pub static szDefaultIcon: [WCHAR; 13] = wide!("\\DefaultIcon");

pub static szFallbackIcon: [WCHAR; 36] = wide!("%SystemRoot%\\system32\\shell32.dll,0");

#[repr(C)]
pub struct IconChangeEntry {
    pub CLSID: LPCTSTR,
    pub TitleId: u32,
    pub IconName: LPCTSTR,
}

pub static IconChange: SyncTable<[IconChangeEntry; 5]> = SyncTable([
    IconChangeEntry {
        CLSID: w!("{20D04FE0-3AEA-1069-A2D8-08002B30309D}"),
        TitleId: 30386,
        IconName: core::ptr::null(),
    },
    IconChangeEntry {
        CLSID: w!("{450D8FBA-AD25-11D0-98A8-0800361B1103}"),
        TitleId: 9227,
        IconName: core::ptr::null(),
    },
    IconChangeEntry {
        CLSID: w!("{208D2C60-3AEA-1069-A2D7-08002B30309D}"),
        TitleId: 30387,
        IconName: core::ptr::null(),
    },
    IconChangeEntry {
        CLSID: w!("{645FF040-5081-101B-9F08-00AA002F954E}"),
        TitleId: 30388,
        IconName: w!("Full"),
    },
    IconChangeEntry {
        CLSID: w!("{645FF040-5081-101B-9F08-00AA002F954E}"),
        TitleId: 30389,
        IconName: w!("Empty"),
    },
]);
#[no_mangle]
pub unsafe extern "C" fn InitDesktopSettings(pData: *mut DESKTOP_DATA) {
    let mut i: UINT;
    let mut regPath: [TCHAR; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];

    StringCchCopy(regPath.as_mut_ptr(), (_countof!(regPath)) as usize, szHideDesktopIcons.as_ptr());
    StringCchCat(regPath.as_mut_ptr(), (_countof!(regPath)) as usize, szClassicStartMenu.as_ptr());

    i = 0;
    while i < _countof!((*pData).optIcons) {
        (*pData).optIcons[i as usize].bHideClassic = SHRegGetBoolUSValue(regPath.as_ptr(), DesktopIcons[i as usize].CLSID, (FALSE) as i32, (FALSE) as i32);
        i += 1;
    }

    StringCchCopy(regPath.as_mut_ptr(), (_countof!(regPath)) as usize, szHideDesktopIcons.as_ptr());
    StringCchCat(regPath.as_mut_ptr(), (_countof!(regPath)) as usize, szNewStartPanel.as_ptr());

    i = 0;
    while i < _countof!((*pData).optIcons) {
        (*pData).optIcons[i as usize].bHideNewStart = SHRegGetBoolUSValue(regPath.as_ptr(), DesktopIcons[i as usize].CLSID, (FALSE) as i32, (TRUE) as i32);
        i += 1;
    }

    i = 0;
    while i < _countof!(IconChange) {
        let mut cbData: u32;
        let mut dwType: u32 = 0;
        let mut szData: [TCHAR; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];

        StringCchCopy(regPath.as_mut_ptr(), (_countof!(regPath)) as usize, szUserClass.as_ptr());
        StringCchCat(regPath.as_mut_ptr(), (_countof!(regPath)) as usize, IconChange[i as usize].CLSID);
        StringCchCat(regPath.as_mut_ptr(), (_countof!(regPath)) as usize, szDefaultIcon.as_ptr());
        cbData = core::mem::size_of_val(&szData) as u32;

        if SHGetValue(HKEY_CURRENT_USER, regPath.as_ptr(), IconChange[i as usize].IconName, core::ptr::addr_of_mut!(dwType), szData.as_mut_ptr() as *mut core::ffi::c_void, core::ptr::addr_of_mut!(cbData)) == ERROR_SUCCESS
            && (dwType == REG_SZ || dwType == REG_EXPAND_SZ)
        {
            StringCchCopy(((*pData).Icon[i as usize].szPath.as_mut_ptr()).cast(), (_countof!((*pData).Icon[i as usize].szPath)) as usize, szData.as_ptr());
        }

        StringCchCopy(regPath.as_mut_ptr(), (_countof!(regPath)) as usize, szSysClass.as_ptr());
        StringCchCat(regPath.as_mut_ptr(), (_countof!(regPath)) as usize, IconChange[i as usize].CLSID);
        StringCchCat(regPath.as_mut_ptr(), (_countof!(regPath)) as usize, szDefaultIcon.as_ptr());
        cbData = core::mem::size_of_val(&szData) as u32;

        if SHGetValue(HKEY_CLASSES_ROOT, regPath.as_ptr(), IconChange[i as usize].IconName, core::ptr::addr_of_mut!(dwType), szData.as_mut_ptr() as *mut core::ffi::c_void, core::ptr::addr_of_mut!(cbData)) == ERROR_SUCCESS
            && (dwType == REG_SZ || dwType == REG_EXPAND_SZ)
        {
            StringCchCopy(((*pData).DefIcon[i as usize].szPath.as_mut_ptr()).cast(), (_countof!((*pData).DefIcon[i as usize].szPath)) as usize, szData.as_ptr());
        }

        if lstrlen(((*pData).DefIcon[i as usize].szPath.as_ptr()).cast()) == 0 {
            StringCchCopy(((*pData).DefIcon[i as usize].szPath.as_mut_ptr()).cast(), (_countof!((*pData).DefIcon[i as usize].szPath)) as usize, szFallbackIcon.as_ptr());
        }

        if lstrlen(((*pData).Icon[i as usize].szPath.as_ptr()).cast()) == 0 {
            StringCchCopy((*pData).Icon[i as usize].szPath.as_mut_ptr(), (_countof!((*pData).Icon[i as usize].szPath)) as usize, (*pData).DefIcon[i as usize].szPath.as_ptr());
        }

        i += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn SaveDesktopSettings(pData: *mut DESKTOP_DATA) -> BOOL {
    let mut i: UINT;

    if (*pData).bLocalSettingsChanged == 0 {
        return (FALSE) as i32;
    }

    i = 0;
    while i < _countof!(DesktopIcons) {
        if (*pData).bLocalHideChanged[i as usize] == 0 {
            i += 1;
            continue;
        }

        (*pData).optIcons[i as usize].bHideClassic = (*pData).bLocalHideIcon[i as usize];
        (*pData).optIcons[i as usize].bHideNewStart = (*pData).bLocalHideIcon[i as usize];
        (*pData).bHideChanged[i as usize] = (TRUE) as i32;
        i += 1;
    }

    i = 0;
    while i < _countof!(IconChange) {
        if (*pData).bLocalIconChanged[i as usize] == 0 {
            i += 1;
            continue;
        }

        StringCchCopy(
            (*pData).Icon[i as usize].szPath.as_mut_ptr(),
            (_countof!((*pData).Icon[i as usize].szPath)) as usize,
            (*pData).LocalIcon[i as usize].szPath.as_ptr()
        );
        (*pData).bIconChanged[i as usize] = (TRUE) as i32;
        i += 1;
    }

    (*pData).bSettingsChanged = (TRUE) as i32;
    (TRUE) as i32
}
unsafe extern "C" fn GetCurrentValue(i: u32, bNewStart: u8) -> u8 {
    let mut regPath: [TCHAR; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];

    StringCchCopy(regPath.as_mut_ptr(), (_countof!(regPath)) as usize, szHideDesktopIcons.as_ptr());
    StringCchCat(regPath.as_mut_ptr(), (_countof!(regPath)) as usize, if bNewStart != 0 { szNewStartPanel.as_ptr() } else { szClassicStartMenu.as_ptr() });

    (SHRegGetBoolUSValue(regPath.as_ptr(), DesktopIcons[i as usize].CLSID, (FALSE) as i32, (bNewStart) as i32)) as u8
}
unsafe extern "C" fn SetCurrentValue(i: u32, bNewStart: u8, bValue: u8) {
    let mut regPath: [TCHAR; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];

    StringCchCopy(regPath.as_mut_ptr(), (_countof!(regPath)) as usize, szHideDesktopIcons.as_ptr());
    StringCchCat(regPath.as_mut_ptr(), (_countof!(regPath)) as usize, if bNewStart != 0 { szNewStartPanel.as_ptr() } else { szClassicStartMenu.as_ptr() });

    SHSetValue(HKEY_CURRENT_USER, regPath.as_ptr(), DesktopIcons[i as usize].CLSID, REG_DWORD,
               (core::ptr::addr_of!(bValue) as LPBYTE) as *const c_void, core::mem::size_of_val(&bValue) as u32);
}
#[no_mangle]
pub unsafe extern "C" fn SetDesktopSettings(pData: PDESKTOP_DATA) {
    let mut i: u32;

    i = 0;
    while i < _countof!(DesktopIcons) {
        if (*pData).bHideChanged[i as usize] == 0 {
            i = i.wrapping_add(1);
            continue;
        }

        if GetCurrentValue(i, (FALSE) as u8) != ((*pData).optIcons[i as usize].bHideClassic) as u8 {
            SetCurrentValue(i, (FALSE) as u8, ((*pData).optIcons[i as usize].bHideClassic) as u8);
        }

        if GetCurrentValue(i, (TRUE) as u8) != ((*pData).optIcons[i as usize].bHideNewStart) as u8 {
            SetCurrentValue(i, (TRUE) as u8, ((*pData).optIcons[i as usize].bHideNewStart) as u8);
        }

        (*pData).bHideChanged[i as usize] = (FALSE) as i32;
        i = i.wrapping_add(1);
    }

    i = 0;
    while i < _countof!(IconChange) {
        let mut iconPath: [TCHAR; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];
        let dwType: u32 = if (*pData).Icon[i as usize].szPath[0] == '%' as WCHAR {
            REG_EXPAND_SZ
        } else {
            REG_SZ
        };

        if (*pData).bIconChanged[i as usize] == 0 {
            i = i.wrapping_add(1);
            continue;
        }

        StringCchCopy(iconPath.as_mut_ptr(), (_countof!(iconPath)) as usize, szUserClass.as_ptr());
        StringCchCat(iconPath.as_mut_ptr(), (_countof!(iconPath)) as usize, IconChange[i as usize].CLSID);
        StringCchCat(iconPath.as_mut_ptr(), (_countof!(iconPath)) as usize, szDefaultIcon.as_ptr());

        SHSetValue(
            HKEY_CURRENT_USER,
            iconPath.as_ptr(),
            IconChange[i as usize].IconName,
            dwType,
            (*pData).Icon[i as usize].szPath.as_ptr() as *const core::ffi::c_void,
            core::mem::size_of_val(&(*pData).Icon[i as usize].szPath) as u32
        );
        if IconChange[i as usize].TitleId == IDS_TITLE_BIN_0 {
            SHSetValue(
                HKEY_CURRENT_USER,
                iconPath.as_ptr(),
                core::ptr::null(),
                dwType,
                (*pData).Icon[i as usize].szPath.as_ptr() as *const core::ffi::c_void,
                core::mem::size_of_val(&(*pData).Icon[i as usize].szPath) as u32
            );
        }
        (*pData).bIconChanged[i as usize] = (FALSE) as i32;
        i = i.wrapping_add(1);
    }

    (*pData).bSettingsChanged = (FALSE) as i32;

    PostMessage(GetShellWindow(), WM_COMMAND, (FCIDM_DESKBROWSER_REFRESH) as WPARAM, 0);
}
unsafe extern "C" fn GetIconFromLocation(szIconPath: LPTSTR) -> HICON {
    let mut iIndex: i32;
    let mut szPath: [TCHAR; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];

    ExpandEnvironmentStrings(szIconPath, szPath.as_mut_ptr(), _countof!(szPath));
    iIndex = PathParseIconLocation(szPath.as_mut_ptr());
    return ExtractIcon(hApplet, szPath.as_mut_ptr(), (iIndex) as u32);
}
unsafe extern "C" fn DesktopOnInitDialog(hwndDlg: HWND, pData: *mut DESKTOP_DATA) {
    let mut i: u32;
    let mut ss: SHELLSTATE = core::mem::zeroed();
    let hwndList: HWND;

    SHGetSetSettings(&mut ss, SSF_STARTPANELON, (FALSE) as BOOL);

    i = 0;
    while i < _countof!((*pData).optIcons) {
        let bHide: i32;

        if ss.fStartPanelOn() != 0 {
            bHide = (*pData).optIcons[i as usize].bHideNewStart;
        } else {
            bHide = (*pData).optIcons[i as usize].bHideClassic;
        }

        CheckDlgButton(
            hwndDlg,
            (DesktopIcons[i as usize].Checkbox) as i32,
            if bHide != 0 { BST_UNCHECKED } else { BST_CHECKED }
        );

        (*pData).bLocalHideIcon[i as usize] = bHide;
        (*pData).bLocalHideChanged[i as usize] = (FALSE) as i32;

        i += 1;
    }

    (*pData).iLocalCurIcon = 0;
    hwndList = GetDlgItem(hwndDlg, (IDC_ICONS_LISTVIEW) as i32);
    (*pData).hLocalImageList = ImageList_Create(
        GetSystemMetrics((SM_CXICON) as i32),
        GetSystemMetrics((SM_CYICON) as i32),
        ILC_COLOR32 | ILC_MASK,
        1,
        1
    );
    ListView_SetImageList!(hwndList, (*pData).hLocalImageList, LVSIL_NORMAL);

    i = 0;
    while i < _countof!(IconChange) {
        let mut szClassPath: [TCHAR; (MAX_PATH) as usize] = core::mem::zeroed();
        let mut dwType: u32 = 0;
        let mut cbData: u32;
        let mut lvitem: LVITEM = core::mem::zeroed();
        let hIcon: HICON;

        StringCchCopy(
            (*pData).LocalIcon[i as usize].szPath.as_mut_ptr(),
            (_countof!((*pData).LocalIcon[i as usize].szPath)) as usize,
            (*pData).Icon[i as usize].szPath.as_ptr()
        );
        (*pData).bLocalIconChanged[i as usize] = (FALSE) as i32;

        StringCchCopy(szClassPath.as_mut_ptr(), (_countof!(szClassPath)) as usize, szUserClass.as_ptr());
        StringCchCat(szClassPath.as_mut_ptr(), (_countof!(szClassPath)) as usize, IconChange[i as usize].CLSID);
        cbData = core::mem::size_of_val(&(*pData).LocalIcon[i as usize].szTitle) as u32;

        if SHGetValue(
            HKEY_CURRENT_USER,
            szClassPath.as_ptr(),
            IconChange[i as usize].IconName,
            &mut dwType,
            ((*pData).LocalIcon[i as usize].szTitle.as_mut_ptr() as LPBYTE).cast(),
            &mut cbData
        ) != ERROR_SUCCESS || dwType != REG_SZ
        {
            LoadString(
                GetModuleHandle(w!("shell32.dll")),
                IconChange[i as usize].TitleId,
                ((*pData).LocalIcon[i as usize].szTitle.as_mut_ptr()).cast(),
                (_countof!((*pData).LocalIcon[i as usize].szTitle)) as i32
            );
        }

        hIcon = GetIconFromLocation(((*pData).LocalIcon[i as usize].szPath.as_mut_ptr()).cast());

        lvitem.mask = LVIF_TEXT | LVIF_IMAGE | LVIF_PARAM;
        lvitem.iItem = i as i32;
        lvitem.iSubItem = 0;
        lvitem.pszText = ((*pData).LocalIcon[i as usize].szTitle.as_mut_ptr()).cast();
        lvitem.lParam = i as LPARAM;

        if !hIcon.is_null() {
            if !(*pData).hLocalImageList.is_null() {
                lvitem.iImage = ImageList_AddIcon!((*pData).hLocalImageList, hIcon);
            }
            DestroyIcon(hIcon);
        }

        if ListView_InsertItem!(hwndList, &lvitem) < 0 {
            i += 1;
            continue;
        }

        if i > 0 {
            i += 1;
            continue;
        }

        lvitem.state = LVIS_FOCUSED | LVIS_SELECTED;
        lvitem.stateMask = LVIS_FOCUSED | LVIS_SELECTED;
        SendMessage(hwndList, LVM_SETITEMSTATE, 0, &lvitem as *const _ as LPARAM);

        i += 1;
    }

    (*pData).bLocalSettingsChanged = (FALSE) as i32;
}
unsafe extern "C" fn DesktopOnDestroyDialog(hwndDlg: HWND, pData: *mut DESKTOP_DATA) {
    if (*pData).hLocalImageList != core::ptr::null_mut() {
        ListView_SetImageList!(GetDlgItem(hwndDlg, (IDC_ICONS_LISTVIEW) as i32), core::ptr::null_mut::<core::ffi::c_void>(), LVSIL_NORMAL);
        ImageList_Destroy((*pData).hLocalImageList);
    }

    SetWindowLongPtr(hwndDlg, DWLP_USER, 0);
}
#[no_mangle]
pub unsafe extern "system" fn DesktopPageProc(
    hwndDlg: HWND,
    uMsg: u32,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let pData: *mut DESKTOP_DATA;

    pData = GetWindowLongPtr(hwndDlg, DWLP_USER) as *mut DESKTOP_DATA;

    match uMsg {
        WM_INITDIALOG => {
            let ppsp = lParam as *mut PROPSHEETPAGE;
            let pData = (*ppsp).lParam as *mut DESKTOP_DATA;

            SetWindowLongPtr(hwndDlg, DWLP_USER, ((pData as isize) as i32) as LPARAM);
            DesktopOnInitDialog(hwndDlg, pData);
        }

        WM_DESTROY => {
            DesktopOnDestroyDialog(hwndDlg, pData);
        }

        WM_COMMAND => {
            let controlId = LOWORD!(wParam);
            let command = HIWORD!(wParam);

            if command == (BN_CLICKED) as u16 {
                let mut i: u32;
                let mut bUpdateIcon = FALSE;

                i = 0;
                while i < _countof!(DesktopIcons) {
                    if DesktopIcons[i as usize].Checkbox == (controlId) as u32 {
                        (*pData).bLocalHideIcon[i as usize] = if IsDlgButtonChecked(
                            hwndDlg,
                            (DesktopIcons[i as usize].Checkbox) as i32,
                        ) == BST_UNCHECKED
                        {
                            1
                        } else {
                            0
                        };

                        (*pData).bLocalSettingsChanged = (TRUE) as i32;
                        (*pData).bLocalHideChanged[i as usize] = (TRUE) as i32;

                        PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                        break;
                    }
                    i += 1;
                }

                if controlId == (IDC_ICONS_CHANGEICON) as u16 {
                    let mut szPath: [TCHAR; (MAX_PATH) as usize] =
                        [0; (MAX_PATH) as usize];
                    let mut iIndex: i32;
                    i = ((*pData).iLocalCurIcon) as u32;

                    ExpandEnvironmentStrings(
                        ((*pData).LocalIcon[i as usize].szPath.as_mut_ptr()) as *const u16,
                        szPath.as_mut_ptr(),
                        _countof!(szPath),
                    );
                    iIndex = PathParseIconLocation(szPath.as_mut_ptr());

                    if PickIconDlg(
                        hwndDlg,
                        szPath.as_mut_ptr(),
                        _countof!(szPath),
                        &mut iIndex,
                    ) != 0
                    {
                        StringCchCopy(
                            ((*pData).LocalIcon[i as usize].szPath.as_mut_ptr()).cast(),
                            (_countof!((*pData).LocalIcon[i as usize].szPath)) as usize,
                            szPath.as_mut_ptr(),
                        );
                        PathUnExpandEnvStrings(
                            ((*pData).LocalIcon[i as usize].szPath.as_mut_ptr()) as *const u16,
                            szPath.as_mut_ptr(),
                            _countof!(szPath),
                        );

                        StringCchPrintf!(
                            ((*pData).LocalIcon[i as usize].szPath.as_mut_ptr()).cast(),
                            (_countof!((*pData).LocalIcon[i as usize].szPath)) as usize,
                            w!("%s,%d"),
                            szPath.as_ptr(),
                            iIndex,
                        );
                        bUpdateIcon = TRUE;
                    }
                } else if controlId == (IDC_ICONS_SETDEFAULT) as u16 {
                    i = ((*pData).iLocalCurIcon) as u32;

                    StringCchCopy(
                        (*pData).LocalIcon[i as usize].szPath.as_mut_ptr(),
                        (_countof!((*pData).LocalIcon[i as usize].szPath)) as usize,
                        (*pData).DefIcon[i as usize].szPath.as_ptr(),
                    );
                    bUpdateIcon = TRUE;
                }

                if bUpdateIcon != 0 {
                    let hwndList = GetDlgItem(hwndDlg, (IDC_ICONS_LISTVIEW) as i32);
                    let hIcon: HICON;

                    hIcon = GetIconFromLocation(
                        ((*pData).LocalIcon[i as usize].szPath.as_mut_ptr()).cast(),
                    );

                    if !hIcon.is_null() {
                        if !(*pData).hLocalImageList.is_null() {
                            ImageList_ReplaceIcon((*pData).hLocalImageList, i as i32, hIcon);
                        }
                        DestroyIcon(hIcon);
                    }

                    (*pData).bLocalSettingsChanged = (TRUE) as i32;
                    (*pData).bLocalIconChanged[i as usize] = (TRUE) as i32;

                    InvalidateRect(hwndList, core::ptr::null_mut(), (TRUE) as BOOL);
                    SetFocus(hwndList);
                    PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                }
            }
        }

        WM_NOTIFY => {
            let nm = lParam as *mut NMLISTVIEW;

            match (*nm).hdr.code {
                LVN_ITEMCHANGED => {
                    if ((*nm).uNewState & LVIS_SELECTED) == 0 {
                        return (FALSE) as LPARAM;
                    }

                    (*pData).iLocalCurIcon = ((*nm).iItem as u32) as i32;
                }
                _ => {}
            }
        }

        _ => {}
    }

    return (FALSE as INT_PTR) as LPARAM;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS Display Control Panel
//  * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
//  * PURPOSE:     Desktop customization property page
//  * COPYRIGHT:   Copyright 2018-2022 Stanislav Motylkov <x86corez@gmail.com>
//  */
//
// #include "desk.h"
//
// #include <shlwapi.h>
// #include <shellapi.h>
//
// /* From shresdef.h */
// #define FCIDM_DESKBROWSER_REFRESH    0xA220
//
// #define IDS_TITLE_MYCOMP  30386
// #define IDS_TITLE_MYNET   30387
// #define IDS_TITLE_BIN_1   30388
// #define IDS_TITLE_BIN_0   30389
//
// /* Workaround:
//  * There's no special fallback icon title string
//  * for My Documents in shell32.dll, so use IDS_PERSONAL.
//  *
//  * Windows does this in some different way.
//  */
// #define IDS_PERSONAL  9227
//
// static const TCHAR szHideDesktopIcons[] = TEXT("Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\HideDesktopIcons\\");
// static const TCHAR szClassicStartMenu[] = TEXT("ClassicStartMenu");
// static const TCHAR szNewStartPanel[] = TEXT("NewStartPanel");
//
// struct
// {
//     LPCTSTR CLSID;
//     UINT Checkbox;
// } DesktopIcons[NUM_DESKTOP_ICONS] = {
//     {TEXT("{450D8FBA-AD25-11D0-98A8-0800361B1103}"), IDC_ICONS_MYDOCS},   /* My Documents */
//     {TEXT("{208D2C60-3AEA-1069-A2D7-08002B30309D}"), IDC_ICONS_MYNET},    /* My Network Places */
//     {TEXT("{20D04FE0-3AEA-1069-A2D8-08002B30309D}"), IDC_ICONS_MYCOMP},   /* My Computer */
//     {TEXT("{871C5380-42A0-1069-A2EA-08002B30309D}"), IDC_ICONS_INTERNET}, /* Internet Browser */
// };
//
// static const TCHAR szUserClass[] = TEXT("Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\CLSID\\");
// static const TCHAR szSysClass[] = TEXT("CLSID\\");
// static const TCHAR szDefaultIcon[] = TEXT("\\DefaultIcon");
// static const TCHAR szFallbackIcon[] = TEXT("%SystemRoot%\\system32\\shell32.dll,0");
//
// struct
// {
//     LPCTSTR CLSID;
//     UINT TitleId;
//     LPCTSTR IconName;
// } IconChange[NUM_CHANGE_ICONS] = {
//     {TEXT("{20D04FE0-3AEA-1069-A2D8-08002B30309D}"), IDS_TITLE_MYCOMP, NULL},         /* My Computer */
//     {TEXT("{450D8FBA-AD25-11D0-98A8-0800361B1103}"), IDS_PERSONAL, NULL},             /* My Documents */
//     {TEXT("{208D2C60-3AEA-1069-A2D7-08002B30309D}"), IDS_TITLE_MYNET, NULL},          /* My Network Places */
//     {TEXT("{645FF040-5081-101B-9F08-00AA002F954E}"), IDS_TITLE_BIN_1, TEXT("Full")},  /* Recycle Bin (full) */
//     {TEXT("{645FF040-5081-101B-9F08-00AA002F954E}"), IDS_TITLE_BIN_0, TEXT("Empty")}, /* Recycle Bin (empty) */
// };
//
// VOID
// InitDesktopSettings(PDESKTOP_DATA pData)
// {
//     UINT i;
//     TCHAR regPath[MAX_PATH];
//
//     /* Load desktop icon settings from the registry */
//     StringCchCopy(regPath, _countof(regPath), szHideDesktopIcons);
//     StringCchCat(regPath, _countof(regPath), szClassicStartMenu);
//
//     for (i = 0; i < _countof(pData->optIcons); i++)
//     {
//         pData->optIcons[i].bHideClassic = SHRegGetBoolUSValue(regPath, DesktopIcons[i].CLSID, FALSE, FALSE);
//     }
//
//     StringCchCopy(regPath, _countof(regPath), szHideDesktopIcons);
//     StringCchCat(regPath, _countof(regPath), szNewStartPanel);
//
//     for (i = 0; i < _countof(pData->optIcons); i++)
//     {
//         pData->optIcons[i].bHideNewStart = SHRegGetBoolUSValue(regPath, DesktopIcons[i].CLSID, FALSE, TRUE);
//     }
//
//     for (i = 0; i < _countof(IconChange); i++)
//     {
//         DWORD cbData, dwType;
//         TCHAR szData[MAX_PATH];
//
//         /* Current icons */
//         StringCchCopy(regPath, _countof(regPath), szUserClass);
//         StringCchCat(regPath, _countof(regPath), IconChange[i].CLSID);
//         StringCchCat(regPath, _countof(regPath), szDefaultIcon);
//         cbData = sizeof(szData);
//
//         if (SHGetValue(HKEY_CURRENT_USER, regPath, IconChange[i].IconName, &dwType,
//                        &szData, &cbData) == ERROR_SUCCESS &&
//             (dwType == REG_SZ || dwType == REG_EXPAND_SZ))
//         {
//             StringCchCopy(pData->Icon[i].szPath, _countof(pData->Icon[i].szPath), szData);
//         }
//
//         /* Default icons */
//         /* FIXME: Get default icons from theme data, fallback to CLSID data on error. */
//         StringCchCopy(regPath, _countof(regPath), szSysClass);
//         StringCchCat(regPath, _countof(regPath), IconChange[i].CLSID);
//         StringCchCat(regPath, _countof(regPath), szDefaultIcon);
//         cbData = sizeof(szData);
//
//         if (SHGetValue(HKEY_CLASSES_ROOT, regPath, IconChange[i].IconName, &dwType,
//                        &szData, &cbData) == ERROR_SUCCESS &&
//             (dwType == REG_SZ || dwType == REG_EXPAND_SZ))
//         {
//             StringCchCopy(pData->DefIcon[i].szPath, _countof(pData->DefIcon[i].szPath), szData);
//         }
//
//         /* Emergency fallback */
//         if (lstrlen(pData->DefIcon[i].szPath) == 0)
//             StringCchCopy(pData->DefIcon[i].szPath, _countof(pData->DefIcon[i].szPath), szFallbackIcon);
//
//         if (lstrlen(pData->Icon[i].szPath) == 0)
//             StringCchCopy(pData->Icon[i].szPath, _countof(pData->Icon[i].szPath), pData->DefIcon[i].szPath);
//     }
// }
//
// BOOL
// SaveDesktopSettings(PDESKTOP_DATA pData)
// {
//     UINT i;
//
//     if (!pData->bLocalSettingsChanged)
//         return FALSE;
//
//     for (i = 0; i < _countof(DesktopIcons); i++)
//     {
//         if (!pData->bLocalHideChanged[i])
//             continue;
//
//         pData->optIcons[i].bHideClassic =
//         pData->optIcons[i].bHideNewStart = pData->bLocalHideIcon[i];
//         pData->bHideChanged[i] = TRUE;
//     }
//
//     for (i = 0; i < _countof(IconChange); i++)
//     {
//         if (!pData->bLocalIconChanged[i])
//             continue;
//
//         StringCchCopy(pData->Icon[i].szPath, _countof(pData->Icon[i].szPath), pData->LocalIcon[i].szPath);
//         pData->bIconChanged[i] = TRUE;
//     }
//
//     pData->bSettingsChanged = TRUE;
//     return TRUE;
// }
//
// static BOOL
// GetCurrentValue(UINT i, BOOL bNewStart)
// {
//     TCHAR regPath[MAX_PATH];
//
//     StringCchCopy(regPath, _countof(regPath), szHideDesktopIcons);
//     StringCchCat(regPath, _countof(regPath), bNewStart ? szNewStartPanel : szClassicStartMenu);
//
//     return SHRegGetBoolUSValue(regPath, DesktopIcons[i].CLSID, FALSE, bNewStart);
// }
//
// static VOID
// SetCurrentValue(UINT i, BOOL bNewStart, BOOL bValue)
// {
//     TCHAR regPath[MAX_PATH];
//
//     StringCchCopy(regPath, _countof(regPath), szHideDesktopIcons);
//     StringCchCat(regPath, _countof(regPath), bNewStart ? szNewStartPanel : szClassicStartMenu);
//
//     SHSetValue(HKEY_CURRENT_USER, regPath, DesktopIcons[i].CLSID, REG_DWORD,
//                (LPBYTE)&bValue, sizeof(bValue));
// }
//
// VOID
// SetDesktopSettings(PDESKTOP_DATA pData)
// {
//     UINT i;
//
//     for (i = 0; i < _countof(DesktopIcons); i++)
//     {
//         if (!pData->bHideChanged[i])
//             continue;
//
//         if (GetCurrentValue(i, FALSE) != pData->optIcons[i].bHideClassic)
//             SetCurrentValue(i, FALSE, pData->optIcons[i].bHideClassic);
//
//         if (GetCurrentValue(i, TRUE) != pData->optIcons[i].bHideNewStart)
//             SetCurrentValue(i, TRUE, pData->optIcons[i].bHideNewStart);
//
//         pData->bHideChanged[i] = FALSE;
//     }
//
//     for (i = 0; i < _countof(IconChange); i++)
//     {
//         TCHAR iconPath[MAX_PATH];
//         DWORD dwType = (pData->Icon[i].szPath[0] == TEXT('%') ? REG_EXPAND_SZ : REG_SZ);
//
//         if (!pData->bIconChanged[i])
//             continue;
//
//         StringCchCopy(iconPath, _countof(iconPath), szUserClass);
//         StringCchCat(iconPath, _countof(iconPath), IconChange[i].CLSID);
//         StringCchCat(iconPath, _countof(iconPath), szDefaultIcon);
//
//         SHSetValue(HKEY_CURRENT_USER, iconPath, IconChange[i].IconName, dwType,
//                    pData->Icon[i].szPath, sizeof(pData->Icon[i].szPath));
//         if (IconChange[i].TitleId == IDS_TITLE_BIN_0)
//         {
//             /* Also apply to the root value */
//             SHSetValue(HKEY_CURRENT_USER, iconPath, NULL, dwType,
//                        pData->Icon[i].szPath, sizeof(pData->Icon[i].szPath));
//         }
//         pData->bIconChanged[i] = FALSE;
//     }
//
//     pData->bSettingsChanged = FALSE;
//
//     /* Refresh the desktop */
//     PostMessage(GetShellWindow(), WM_COMMAND, FCIDM_DESKBROWSER_REFRESH, 0);
// }
//
// static HICON
// GetIconFromLocation(LPTSTR szIconPath)
// {
//     INT iIndex;
//     TCHAR szPath[MAX_PATH];
//
//     ExpandEnvironmentStrings(szIconPath, szPath, _countof(szPath));
//     iIndex = PathParseIconLocation(szPath);
//     return ExtractIcon(hApplet, szPath, iIndex);
// }
//
// static VOID
// DesktopOnInitDialog( HWND hwndDlg,  PDESKTOP_DATA pData)
// {
//     UINT i;
//     SHELLSTATE ss = {0};
//     HWND hwndList;
//
//     SHGetSetSettings(&ss, SSF_STARTPANELON, FALSE);
//
//     for (i = 0; i < _countof(pData->optIcons); i++)
//     {
//         BOOL bHide;
//
//         if (ss.fStartPanelOn)
//             bHide = pData->optIcons[i].bHideNewStart;
//         else
//             bHide = pData->optIcons[i].bHideClassic;
//
//         CheckDlgButton(hwndDlg,
//                        DesktopIcons[i].Checkbox,
//                        bHide ? BST_UNCHECKED : BST_CHECKED);
//
//         pData->bLocalHideIcon[i] = bHide;
//         pData->bLocalHideChanged[i] = FALSE;
//     }
//
//     pData->iLocalCurIcon = 0;
//     hwndList = GetDlgItem(hwndDlg, IDC_ICONS_LISTVIEW);
//     pData->hLocalImageList = ImageList_Create(GetSystemMetrics(SM_CXICON), GetSystemMetrics(SM_CYICON), ILC_COLOR32 | ILC_MASK, 1, 1);
//     ListView_SetImageList(hwndList, pData->hLocalImageList, LVSIL_NORMAL);
//
//     for (i = 0; i < _countof(IconChange); i++)
//     {
//         TCHAR szClassPath[MAX_PATH];
//         DWORD dwType, cbData;
//         LVITEM lvitem = {0};
//         HICON hIcon;
//
//         StringCchCopy(pData->LocalIcon[i].szPath, _countof(pData->LocalIcon[i].szPath), pData->Icon[i].szPath);
//         pData->bLocalIconChanged[i] = FALSE;
//
//         /* Try loading user-defined desktop icon title */
//         StringCchCopy(szClassPath, _countof(szClassPath), szUserClass);
//         StringCchCat(szClassPath, _countof(szClassPath), IconChange[i].CLSID);
//         cbData = sizeof(pData->LocalIcon[i].szTitle);
//
//         if (SHGetValue(HKEY_CURRENT_USER, szClassPath, IconChange[i].IconName, &dwType,
//                        pData->LocalIcon[i].szTitle, &cbData) != ERROR_SUCCESS || dwType != REG_SZ)
//         {
//             /* Fallback to predefined strings */
//             LoadString(GetModuleHandle(TEXT("shell32.dll")),
//                        IconChange[i].TitleId,
//                        pData->LocalIcon[i].szTitle,
//                        _countof(pData->LocalIcon[i].szTitle));
//         }
//
//         hIcon = GetIconFromLocation(pData->LocalIcon[i].szPath);
//
//         lvitem.mask = LVIF_TEXT | LVIF_IMAGE | LVIF_PARAM;
//         lvitem.iItem = i;
//         lvitem.iSubItem = 0;
//         lvitem.pszText = pData->LocalIcon[i].szTitle;
//         lvitem.lParam = (LPARAM)i;
//
//         if (hIcon)
//         {
//             if (pData->hLocalImageList)
//                 lvitem.iImage = ImageList_AddIcon(pData->hLocalImageList, hIcon);
//             DestroyIcon(hIcon);
//         }
//
//         if (ListView_InsertItem(hwndList, &lvitem) < 0)
//             continue;
//
//         if (i > 0)
//             continue;
//
//         lvitem.state = LVIS_FOCUSED | LVIS_SELECTED;
//         lvitem.stateMask = LVIS_FOCUSED | LVIS_SELECTED;
//         SendMessage(hwndList, LVM_SETITEMSTATE, 0, (LPARAM)&lvitem);
//     }
//
//     pData->bLocalSettingsChanged = FALSE;
// }
//
// static VOID
// DesktopOnDestroyDialog( HWND hwndDlg,  PDESKTOP_DATA pData)
// {
//     if (pData->hLocalImageList)
//     {
//         ListView_SetImageList(GetDlgItem(hwndDlg, IDC_ICONS_LISTVIEW), NULL, LVSIL_NORMAL);
//         ImageList_Destroy(pData->hLocalImageList);
//     }
//
//     SetWindowLongPtr(hwndDlg, DWLP_USER, 0);
// }
//
// /* Property page dialog callback */
// INT_PTR __stdcall
// DesktopPageProc( HWND hwndDlg,  UINT uMsg,  WPARAM wParam,  LPARAM lParam)
// {
//     PDESKTOP_DATA pData;
//
//     pData = (PDESKTOP_DATA)GetWindowLongPtr(hwndDlg, DWLP_USER);
//
//     switch (uMsg)
//     {
//         case WM_INITDIALOG:
//         {
//             LPPROPSHEETPAGE ppsp = (LPPROPSHEETPAGE)lParam;
//             pData = (PDESKTOP_DATA)ppsp->lParam;
//
//             SetWindowLongPtr(hwndDlg, DWLP_USER, (LONG_PTR)pData);
//             DesktopOnInitDialog(hwndDlg, pData);
//             break;
//         }
//
//         case WM_DESTROY:
//         {
//             DesktopOnDestroyDialog(hwndDlg, pData);
//             break;
//         }
//
//         case WM_COMMAND:
//         {
//             DWORD controlId = LOWORD(wParam);
//             DWORD command   = HIWORD(wParam);
//
//             if (command == BN_CLICKED)
//             {
//                 UINT i;
//                 BOOL bUpdateIcon = FALSE;
//
//                 for (i = 0; i < _countof(DesktopIcons); i++)
//                 {
//                     if (DesktopIcons[i].Checkbox == controlId)
//                     {
//                         pData->bLocalHideIcon[i] =
//                             (IsDlgButtonChecked(hwndDlg, DesktopIcons[i].Checkbox) == BST_UNCHECKED);
//
//                         pData->bLocalSettingsChanged =
//                         pData->bLocalHideChanged[i] = TRUE;
//
//                         PropSheet_Changed(GetParent(hwndDlg), hwndDlg);
//                         break;
//                     }
//                 }
//
//                 if (controlId == IDC_ICONS_CHANGEICON)
//                 {
//                     TCHAR szPath[MAX_PATH];
//                     INT iIndex;
//                     i = pData->iLocalCurIcon;
//
//                     ExpandEnvironmentStrings(pData->LocalIcon[i].szPath, szPath, _countof(szPath));
//                     iIndex = PathParseIconLocation(szPath);
//
//                     if (PickIconDlg(hwndDlg, szPath, _countof(szPath), &iIndex))
//                     {
//                         StringCchCopy(pData->LocalIcon[i].szPath, _countof(pData->LocalIcon[i].szPath), szPath);
//                         PathUnExpandEnvStrings(pData->LocalIcon[i].szPath, szPath, _countof(szPath));
//
//                         StringCchPrintf(pData->LocalIcon[i].szPath, _countof(pData->LocalIcon[i].szPath), TEXT("%s,%d"), szPath, iIndex);
//                         bUpdateIcon = TRUE;
//                     }
//                 }
//                 else if (controlId == IDC_ICONS_SETDEFAULT)
//                 {
//                     i = pData->iLocalCurIcon;
//
//                     StringCchCopy(pData->LocalIcon[i].szPath, _countof(pData->LocalIcon[i].szPath), pData->DefIcon[i].szPath);
//                     bUpdateIcon = TRUE;
//                 }
//
//                 if (bUpdateIcon)
//                 {
//                     HWND hwndList = GetDlgItem(hwndDlg, IDC_ICONS_LISTVIEW);
//                     HICON hIcon;
//
//                     hIcon = GetIconFromLocation(pData->LocalIcon[i].szPath);
//
//                     if (hIcon)
//                     {
//                         if (pData->hLocalImageList)
//                             ImageList_ReplaceIcon(pData->hLocalImageList, i, hIcon);
//                         DestroyIcon(hIcon);
//                     }
//
//                     pData->bLocalSettingsChanged =
//                     pData->bLocalIconChanged[i] = TRUE;
//
//                     InvalidateRect(hwndList, NULL, TRUE);
//                     SetFocus(hwndList);
//                     PropSheet_Changed(GetParent(hwndDlg), hwndDlg);
//                 }
//             }
//             break;
//         }
//
//         case WM_NOTIFY:
//         {
//             LPNMLISTVIEW nm = (LPNMLISTVIEW)lParam;
//
//             switch (nm->hdr.code)
//             {
//                 case LVN_ITEMCHANGED:
//                 {
//                     if ((nm->uNewState & LVIS_SELECTED) == 0)
//                         return FALSE;
//
//                     pData->iLocalCurIcon = nm->iItem;
//                     break;
//                 }
//             }
//             break;
//         }
//     }
//
//     return FALSE;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
