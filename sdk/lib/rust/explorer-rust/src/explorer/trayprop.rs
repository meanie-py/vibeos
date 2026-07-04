use core::ffi::c_void;
use explorer_sys::*;
use explorer_sys::com::*;
use explorer_sys::externs::*;
use explorer_sys::globals::*;
use explorer_sys::types::*;
use winsrv_sys::consts::*;
use winsrv_sys::w;
use winsrv_sys::winsrv::*;

extern "system" {
    fn ShowCustomizeNotifyIcons(hInstance: HINSTANCE, hwnd: HWND);
}

// ---- file-scope preamble (translated) ----
pub static wImageIdLookupTable: [[[[u16; 2]; 2]; 2]; 2] = [
    [
        [
            [IDB_SYSTRAYPROP_SHOW_NOCLOCK_NODESK, IDB_SYSTRAYPROP_SHOW_NOCLOCK_DESK],
            [IDB_SYSTRAYPROP_HIDE_NOCLOCK_NODESK, IDB_SYSTRAYPROP_HIDE_NOCLOCK_DESK]
        ],
        [
            [IDB_SYSTRAYPROP_SHOW_NOCLOCK_NODESK, IDB_SYSTRAYPROP_SHOW_NOCLOCK_DESK],
            [IDB_SYSTRAYPROP_HIDE_NOCLOCK_NODESK, IDB_SYSTRAYPROP_HIDE_NOCLOCK_DESK]
        ]
    ],
    [
        [
            [IDB_SYSTRAYPROP_SHOW_CLOCK_NODESK, IDB_SYSTRAYPROP_SHOW_CLOCK_DESK],
            [IDB_SYSTRAYPROP_HIDE_CLOCK_NODESK, IDB_SYSTRAYPROP_HIDE_CLOCK_DESK]
        ],
        [
            [IDB_SYSTRAYPROP_SHOW_SECONDS_NODESK, IDB_SYSTRAYPROP_SHOW_SECONDS_DESK],
            [IDB_SYSTRAYPROP_HIDE_SECONDS_NODESK, IDB_SYSTRAYPROP_HIDE_SECONDS_DESK]
        ]
    ]
];

unsafe fn SetBitmap(hwnd: HWND, hbmp: *mut HBITMAP, uImageId: UINT) {
    if !(*hbmp).is_null() {
        DeleteObject(*hbmp as *mut core::ffi::c_void);
    }

    *hbmp = LoadImageW(
        hExplorerInstance,
        MAKEINTRESOURCEW(uImageId),
        IMAGE_BITMAP,
        0,
        0,
        LR_DEFAULTCOLOR,
    ) as HBITMAP;

    if !(*hbmp).is_null() && !hwnd.is_null() {
        let mut bm: BITMAP = core::mem::zeroed();
        GetObject(
            *hbmp as *mut core::ffi::c_void,
            core::mem::size_of::<BITMAP>() as i32,
            &mut bm as *mut BITMAP as *mut core::ffi::c_void,
        );
        SetWindowPos(
            hwnd,
            core::ptr::null_mut(),
            0,
            0,
            bm.bmWidth + 2,
            bm.bmHeight + 2,
            SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOZORDER,
        );
        SendMessageW(hwnd, STM_SETIMAGE, IMAGE_BITMAP as WPARAM, *hbmp as LPARAM);
    }
}

#[no_mangle]
pub unsafe extern "C" fn CTaskBarSettingsPage__UpdateDialog(this: *mut CTaskBarSettingsPage) {
    let bLock = IsDlgButtonChecked((*this).m_hWnd, IDC_TASKBARPROP_LOCK);
    let bHide = IsDlgButtonChecked((*this).m_hWnd, IDC_TASKBARPROP_HIDE);
    let bGroup = IsDlgButtonChecked((*this).m_hWnd, IDC_TASKBARPROP_GROUP);
    let bShowQL = IsDlgButtonChecked((*this).m_hWnd, IDC_TASKBARPROP_SHOWQL);
    let uImageId: UINT;

    let hwndTaskbarBitmap = GetDlgItem((*this).m_hWnd, IDC_TASKBARPROP_TASKBARBITMAP);

    if bHide != 0 {
        uImageId = IDB_TASKBARPROP_AUTOHIDE;
    } else if bLock != 0 && bGroup != 0 && bShowQL != 0 {
        uImageId = IDB_TASKBARPROP_LOCK_GROUP_QL;
    } else if bLock != 0 && bGroup == 0 && bShowQL == 0 {
        uImageId = IDB_TASKBARPROP_LOCK_NOGROUP_NOQL;
    } else if bLock != 0 && bGroup != 0 && bShowQL == 0 {
        uImageId = IDB_TASKBARPROP_LOCK_GROUP_NOQL;
    } else if bLock != 0 && bGroup == 0 && bShowQL != 0 {
        uImageId = IDB_TASKBARPROP_LOCK_NOGROUP_QL;
    } else if bLock == 0 && bGroup == 0 && bShowQL == 0 {
        uImageId = IDB_TASKBARPROP_NOLOCK_NOGROUP_NOQL;
    } else if bLock == 0 && bGroup != 0 && bShowQL == 0 {
        uImageId = IDB_TASKBARPROP_NOLOCK_GROUP_NOQL;
    } else if bLock == 0 && bGroup == 0 && bShowQL != 0 {
        uImageId = IDB_TASKBARPROP_NOLOCK_NOGROUP_QL;
    } else {
        uImageId = IDB_TASKBARPROP_NOLOCK_GROUP_QL;
    }

    SetBitmap(hwndTaskbarBitmap, core::ptr::addr_of_mut!((*this).m_hbmpTaskbar), uImageId);
}

#[no_mangle]
pub unsafe extern "C" fn CTaskBarSettingsPage_ctor(
    this: *mut CTaskBarSettingsPage,
    hwnd: HWND,
) {
    (*this).m_hbmpTaskbar = core::ptr::null_mut();
    (*this).m_hwndTaskbar = hwnd;
}

#[no_mangle]
pub unsafe extern "C" fn CTaskBarSettingsPage_dtor(this: *mut CTaskBarSettingsPage) {
    if !(*this).m_hbmpTaskbar.is_null() {
        DeleteObject((*this).m_hbmpTaskbar);
    }
}

#[no_mangle]
pub unsafe extern "C" fn CTaskBarSettingsPage_OnInitDialog(
    this: *mut CTaskBarSettingsPage,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
    bHandled: *mut BOOL,
) -> LRESULT {
    CheckDlgButton(
        IDC_TASKBARPROP_LOCK,
        if g_TaskbarSettings.bLock != 0 {
            BST_CHECKED
        } else {
            BST_UNCHECKED
        },
    );
    CheckDlgButton(
        IDC_TASKBARPROP_HIDE,
        if g_TaskbarSettings.sr.AutoHide != 0 {
            BST_CHECKED
        } else {
            BST_UNCHECKED
        },
    );
    CheckDlgButton(
        IDC_TASKBARPROP_ONTOP,
        if g_TaskbarSettings.sr.AlwaysOnTop != 0 {
            BST_CHECKED
        } else {
            BST_UNCHECKED
        },
    );
    CheckDlgButton(
        IDC_TASKBARPROP_GROUP,
        if g_TaskbarSettings.bGroupButtons != 0 {
            BST_CHECKED
        } else {
            BST_UNCHECKED
        },
    );
    CheckDlgButton(
        IDC_TASKBARPROP_SMALLICONS,
        if g_TaskbarSettings.bSmallIcons != 0 {
            BST_CHECKED
        } else {
            BST_UNCHECKED
        },
    );

    CTaskBarSettingsPage__UpdateDialog(this);
    1
}

#[no_mangle]
pub unsafe extern "C" fn CTaskBarSettingsPage_OnCtrlCommand(
    this: *mut CTaskBarSettingsPage,
    wNotifyCode: u16,
    wID: u16,
    hWndCtl: HWND,
    bHandled: *mut u8,
) -> LRESULT {
    CTaskBarSettingsPage__UpdateDialog(this);
    CTaskBarSettingsPage_SetModified(this, 1u8);
    0
}

#[no_mangle]
pub unsafe extern "C" fn CTaskBarSettingsPage_OnApply(this: *mut CTaskBarSettingsPage) -> i32 {
    let mut newSettings = g_TaskbarSettings;

    newSettings.bLock = IsDlgButtonChecked((*this).m_hWnd, IDC_TASKBARPROP_LOCK);
    newSettings.sr.AutoHide = IsDlgButtonChecked((*this).m_hWnd, IDC_TASKBARPROP_HIDE);
    newSettings.sr.AlwaysOnTop = IsDlgButtonChecked((*this).m_hWnd, IDC_TASKBARPROP_ONTOP);
    newSettings.bGroupButtons = IsDlgButtonChecked((*this).m_hWnd, IDC_TASKBARPROP_GROUP);
    newSettings.bSmallIcons = IsDlgButtonChecked((*this).m_hWnd, IDC_TASKBARPROP_SMALLICONS);

    SendMessageW((*this).m_hwndTaskbar, TWM_SETTINGSCHANGED, 0, core::ptr::addr_of_mut!(newSettings) as LPARAM);

    PSNRET_NOERROR
}

#[no_mangle]
pub unsafe extern "C" fn CStartMenuSettingsPage__UpdateDialog(this: *mut CStartMenuSettingsPage) {
    let hwndCustomizeClassic = GetDlgItem((*this).m_hWnd, IDC_TASKBARPROP_STARTMENUCLASSICCUST);
    let hwndCustomizeModern = GetDlgItem((*this).m_hWnd, IDC_TASKBARPROP_STARTMENUCUST);
    let hwndStartBitmap = GetDlgItem((*this).m_hWnd, IDC_TASKBARPROP_STARTMENU_BITMAP);
    let hwndModernRadioBtn = GetDlgItem((*this).m_hWnd, IDC_TASKBARPROP_STARTMENU);
    let hwndModernText = GetDlgItem((*this).m_hWnd, IDC_TASKBARPROP_STARTMENUMODERNTEXT);
    let policyNoSimpleStartMenu = SHRestricted(REST_NOSTARTPANEL) != 0;
    let mut bModern: i32 = 0;

    if policyNoSimpleStartMenu != 0 {
        CheckDlgButton((*this).m_hWnd, IDC_TASKBARPROP_STARTMENUCLASSIC, BST_CHECKED);
        EnableWindow(hwndModernRadioBtn, 0);
        ShowWindow(hwndModernRadioBtn, SW_HIDE);
        ShowWindow(hwndModernText, SW_HIDE);
        ShowWindow(hwndCustomizeModern, SW_HIDE);
    } else {
        bModern = IsDlgButtonChecked((*this).m_hWnd, IDC_TASKBARPROP_STARTMENU);
    }

    EnableWindow(hwndCustomizeModern, bModern);
    EnableWindow(hwndCustomizeClassic, if bModern != 0 { 0 } else { 1 });

    let uImageId = if bModern != 0 { IDB_STARTPREVIEW } else { IDB_STARTPREVIEW_CLASSIC };
    SetBitmap(hwndStartBitmap, core::ptr::addr_of_mut!((*this).m_hbmpStartBitmap), uImageId);
}

#[no_mangle]
pub unsafe extern "C" fn CStartMenuSettingsPage_ctor(this: *mut CStartMenuSettingsPage) {
    (*this).m_hbmpStartBitmap = core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "C" fn CStartMenuSettingsPage_dtor_2(this: *mut CStartMenuSettingsPage) {
    if !(*this).m_hbmpStartBitmap.is_null() {
        DeleteObject((*this).m_hbmpStartBitmap);
    }
}

#[no_mangle]
pub unsafe extern "C" fn CStartMenuSettingsPage_OnInitDialog(
    this: *mut CStartMenuSettingsPage,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
    bHandled: *mut BOOL,
) -> LRESULT {
    let modern: BOOL = SHELL_GetSetting(SSF_STARTPANELON, fStartPanelOn);
    CheckDlgButton(
        (*this).m_hWnd,
        if modern != 0 { IDC_TASKBARPROP_STARTMENU } else { IDC_TASKBARPROP_STARTMENUCLASSIC },
        BST_CHECKED,
    );
    CStartMenuSettingsPage__UpdateDialog(this);
    1
}

#[no_mangle]
pub unsafe extern "C" fn CStartMenuSettingsPage_OnStartMenuCustomize(
    this: *mut CStartMenuSettingsPage,
    wNotifyCode: WORD,
    wID: WORD,
    hWndCtl: HWND,
    bHandled: *mut BOOL,
) -> LRESULT {
    ShowCustomizeClassic(hExplorerInstance, (*this).m_hWnd);
    0
}

#[no_mangle]
pub unsafe extern "C" fn CStartMenuSettingsPage_OnApply(this: *mut CStartMenuSettingsPage) -> i32 {
    let mut ss: SHELLSTATE = core::mem::zeroed();
    ss.fStartPanelOn = if IsDlgButtonChecked((*this).m_hWnd, IDC_TASKBARPROP_STARTMENUCLASSIC) != 0 { 0 } else { 1 };
    SHGetSetSettings(core::ptr::addr_of_mut!(ss), SSF_STARTPANELON, 1);
    PSNRET_NOERROR
}

#[no_mangle]
pub unsafe extern "C" fn CNotifySettingsPage__UpdateDialog(this: *mut CNotifySettingsPage) {
    let bShowClock = IsDlgButtonChecked((*this).m_hWnd, IDC_TASKBARPROP_CLOCK);
    let bShowSeconds = IsDlgButtonChecked((*this).m_hWnd, IDC_TASKBARPROP_SECONDS);
    let bHideInactive = IsDlgButtonChecked((*this).m_hWnd, IDC_TASKBARPROP_HIDEICONS);
    let bShowDesktopButton = IsDlgButtonChecked((*this).m_hWnd, IDC_TASKBARPROP_DESKTOP);

    let hwndCustomizeNotifyButton = GetDlgItem((*this).m_hWnd, IDC_TASKBARPROP_ICONCUST);
    let hwndSeconds = GetDlgItem((*this).m_hWnd, IDC_TASKBARPROP_SECONDS);
    let hwndTrayBitmap = GetDlgItem((*this).m_hWnd, IDC_TASKBARPROP_NOTIFICATIONBITMAP);

    EnableWindow(hwndCustomizeNotifyButton, bHideInactive);
    EnableWindow(hwndSeconds, bShowClock);
    if bShowSeconds == 0 {
        CheckDlgButton((*this).m_hWnd, IDC_TASKBARPROP_SECONDS, BST_UNCHECKED);
    }
    let uImageId = wImageIdLookupTable[bShowClock as usize][bShowSeconds as usize][bHideInactive as usize][bShowDesktopButton as usize];

    SetBitmap(hwndTrayBitmap, core::ptr::addr_of_mut!((*this).m_hbmpTray), uImageId);
}

#[no_mangle]
pub unsafe extern "C" fn CNotifySettingsPage_ctor(
    this: *mut CNotifySettingsPage,
    hwnd: HWND,
) {
    (*this).m_hbmpTray = core::ptr::null_mut();
    (*this).m_hwndTaskbar = hwnd;
}

#[no_mangle]
pub unsafe extern "C" fn CNotifySettingsPage_dtor(this: *mut CNotifySettingsPage) {
    if !(*this).m_hbmpTray.is_null() {
        DeleteObject((*this).m_hbmpTray);
    }
}

#[no_mangle]
pub unsafe extern "C" fn CNotifySettingsPage_OnInitDialog(
    this: *mut CNotifySettingsPage,
    uMsg: u32,
    wParam: WPARAM,
    lParam: LPARAM,
    bHandled: *mut u8,
) -> LRESULT {
    EnableWindow(
        GetDlgItem((*this).m_hWnd, IDC_TASKBARPROP_CLOCK),
        if SHRestricted(REST_HIDECLOCK) != 0 { 0 } else { 1 },
    );
    CheckDlgButton(
        (*this).m_hWnd,
        IDC_TASKBARPROP_CLOCK,
        if GetHideClock() == 0 { BST_CHECKED } else { BST_UNCHECKED },
    );
    CheckDlgButton(
        (*this).m_hWnd,
        IDC_TASKBARPROP_SECONDS,
        if g_TaskbarSettings.bShowSeconds != 0 { BST_CHECKED } else { BST_UNCHECKED },
    );
    CheckDlgButton(
        (*this).m_hWnd,
        IDC_TASKBARPROP_HIDEICONS,
        if g_TaskbarSettings.bHideInactiveIcons != 0 { BST_CHECKED } else { BST_UNCHECKED },
    );
    CheckDlgButton(
        (*this).m_hWnd,
        IDC_TASKBARPROP_DESKTOP,
        if g_TaskbarSettings.bShowDesktopButton != 0 { BST_CHECKED } else { BST_UNCHECKED },
    );

    CheckDlgButton((*this).m_hWnd, IDC_TASKBARPROP_HIDEICONS, BST_UNCHECKED);
    EnableWindow(GetDlgItem((*this).m_hWnd, IDC_TASKBARPROP_HIDEICONS), 0);

    CNotifySettingsPage__UpdateDialog(this);
    1
}

#[no_mangle]
pub unsafe extern "C" fn CNotifySettingsPage_OnCustomizeTrayIcons(
    this: *mut CNotifySettingsPage,
    wNotifyCode: WORD,
    wID: WORD,
    hWndCtl: HWND,
    bHandled: *mut BOOL,
) -> LRESULT {
    
    ShowCustomizeNotifyIcons(hExplorerInstance, (*this).m_hWnd);
    0
}

#[no_mangle]
pub unsafe extern "C" fn CNotifySettingsPage_OnCtrlCommand(
    this: *mut CNotifySettingsPage,
    wNotifyCode: u16,
    wID: u16,
    hWndCtl: HWND,
    bHandled: *mut u8,
) -> LRESULT {
    CNotifySettingsPage__UpdateDialog(this);
    CNotifySettingsPage_SetModified(this, 1u8);
    0
}

#[no_mangle]
pub unsafe extern "C" fn CNotifySettingsPage_OnApply(this: *mut CNotifySettingsPage) -> i32 {
    let mut newSettings: TaskbarSettings = g_TaskbarSettings;

    newSettings.sr.HideClock = (IsDlgButtonChecked((*this).m_hWnd, IDC_TASKBARPROP_CLOCK) == 0) as u8;
    newSettings.bShowSeconds = (IsDlgButtonChecked((*this).m_hWnd, IDC_TASKBARPROP_SECONDS) != 0) as u8;
    newSettings.bHideInactiveIcons = (IsDlgButtonChecked((*this).m_hWnd, IDC_TASKBARPROP_HIDEICONS) != 0) as u8;
    newSettings.bShowDesktopButton = (IsDlgButtonChecked((*this).m_hWnd, IDC_TASKBARPROP_DESKTOP) != 0) as u8;

    SendMessageW(
        (*this).m_hwndTaskbar,
        TWM_SETTINGSCHANGED,
        0,
        core::ptr::addr_of_mut!(newSettings) as LPARAM,
    );

    PSNRET_NOERROR
}
unsafe fn PropSheetProc(
    hwndDlg: HWND,
    uMsg: UINT,
    lParam: LPARAM,
) -> i32 {
    let mut hIcon: HICON;
    match uMsg {
        PSCB_INITIALIZED => {
            hIcon = LoadIconW(hExplorerInstance, MAKEINTRESOURCEW(IDI_STARTMENU));
            SendMessageW(hwndDlg, WM_SETICON, ICON_BIG as WPARAM, hIcon as LPARAM);
        }
        _ => {}
    }
    return 0;
}
#[no_mangle]
pub unsafe extern "C" fn DisplayTrayProperties(
    hwndOwner: HWND,
    hwndTaskbar: HWND,
) {
    let mut psh: PROPSHEETHEADER = core::mem::zeroed();
    let mut hpsp: CSimpleArray<HPROPSHEETPAGE> = CSimpleArray::new();
    let mut tbSettingsPage = CTaskBarSettingsPage::new(hwndTaskbar);
    let mut smSettingsPage = CStartMenuSettingsPage::new();
    let mut naSettingsPage = CNotifySettingsPage::new(hwndTaskbar);
    let mut caption = CStringW::new();

    caption.LoadStringW(IDS_TASKBAR_STARTMENU_PROP_CAPTION);

    hpsp.Add(tbSettingsPage.Create());
    hpsp.Add(smSettingsPage.Create());
    hpsp.Add(naSettingsPage.Create());

    core::ptr::write_bytes(&mut psh as *mut PROPSHEETHEADER, 0, 1);
    psh.dwSize = core::mem::size_of::<PROPSHEETHEADER>() as u32;
    psh.dwFlags = PSH_PROPTITLE | PSH_USEICONID | PSH_USECALLBACK;
    psh.hwndParent = hwndOwner;
    psh.hInstance = hExplorerInstance;
    psh.pszIcon = MAKEINTRESOURCEW(IDI_STARTMENU);
    psh.pszCaption = caption.GetString();
    psh.nPages = hpsp.GetSize();
    psh.nStartPage = 0;
    psh.phpage = hpsp.GetData();
    psh.pfnCallback = Some(PropSheetProc);

    PropertySheet(&mut psh);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * ReactOS Explorer
//  *
//  * Copyright 2006 - 2007 Thomas Weidenmueller <w3seek@reactos.org>
//  *                  2015 Robert Naumann <gonzomdx@gmail.com>
//  *
//  * This library is free software; you can redistribute it and/or
//  * modify it under the terms of the GNU Lesser General Public
//  * License as published by the Free Software Foundation; either
//  * version 2.1 of the License, or (at your option) any later version.
//  *
//  * This library is distributed in the hope that it will be useful,
//  * but WITHOUT ANY WARRANTY; without even the implied warranty of
//  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
//  * Lesser General Public License for more details.
//  *
//  * You should have received a copy of the GNU Lesser General Public
//  * License along with this library; if not, write to the Free Software
//  * Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA  02110-1301  USA
//  */
//
// #include "precomp.h"
//
// static void SetBitmap(HWND hwnd, HBITMAP* hbmp, UINT uImageId)
// {
//     if (*hbmp)
//         DeleteObject(*hbmp);
//
//     *hbmp = (HBITMAP)LoadImageW(hExplorerInstance,
//                                 MAKEINTRESOURCEW(uImageId),
//                                 IMAGE_BITMAP,
//                                 0,
//                                 0,
//                                 LR_DEFAULTCOLOR);
//
//     if (*hbmp && hwnd)
//     {
//         BITMAP bm;
//         GetObject(*hbmp, sizeof(bm), &bm);
//         ::SetWindowPos(hwnd, NULL, 0, 0, bm.bmWidth + 2, bm.bmHeight + 2,
//                        SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOZORDER);
//         SendMessage(hwnd, STM_SETIMAGE, IMAGE_BITMAP, (LPARAM)*hbmp);
//     }
//
// }
//
// class CTaskBarSettingsPage : public CPropertyPageImpl<CTaskBarSettingsPage>
// {
// private:
//     HBITMAP m_hbmpTaskbar;
//     HWND m_hwndTaskbar;
//
//     void _UpdateDialog()
//     {
//         BOOL bLock = IsDlgButtonChecked(IDC_TASKBARPROP_LOCK);
//         BOOL bHide = IsDlgButtonChecked(IDC_TASKBARPROP_HIDE);
//         BOOL bGroup = IsDlgButtonChecked(IDC_TASKBARPROP_GROUP);
//         BOOL bShowQL = IsDlgButtonChecked(IDC_TASKBARPROP_SHOWQL);
//         UINT uImageId;
//
//         HWND hwndTaskbarBitmap = GetDlgItem(IDC_TASKBARPROP_TASKBARBITMAP);
//
//         if (bHide)
//             uImageId = IDB_TASKBARPROP_AUTOHIDE;
//         else if (bLock  && bGroup  && bShowQL)
//             uImageId = IDB_TASKBARPROP_LOCK_GROUP_QL;
//         else if (bLock  && !bGroup && !bShowQL)
//             uImageId = IDB_TASKBARPROP_LOCK_NOGROUP_NOQL;
//         else if (bLock  && bGroup  && !bShowQL)
//             uImageId = IDB_TASKBARPROP_LOCK_GROUP_NOQL;
//         else if (bLock  && !bGroup && bShowQL)
//             uImageId = IDB_TASKBARPROP_LOCK_NOGROUP_QL;
//         else if (!bLock && !bGroup && !bShowQL)
//             uImageId = IDB_TASKBARPROP_NOLOCK_NOGROUP_NOQL;
//         else if (!bLock && bGroup  && !bShowQL)
//             uImageId = IDB_TASKBARPROP_NOLOCK_GROUP_NOQL;
//         else if (!bLock && !bGroup && bShowQL)
//             uImageId = IDB_TASKBARPROP_NOLOCK_NOGROUP_QL;
//         else if (!bLock && bGroup  && bShowQL)
//             uImageId = IDB_TASKBARPROP_NOLOCK_GROUP_QL;
//
//         SetBitmap(hwndTaskbarBitmap, &m_hbmpTaskbar, uImageId);
//     }
//
// public:
//     enum { IDD = IDD_TASKBARPROP_TASKBAR };
//
//     BEGIN_MSG_MAP(CTaskBarSettingsPage)
//         MESSAGE_HANDLER(WM_INITDIALOG, OnInitDialog)
//         COMMAND_RANGE_HANDLER(IDC_TASKBARPROP_FIRST_CMD, IDC_TASKBARPROP_LAST_CMD, OnCtrlCommand)
//         CHAIN_MSG_MAP(CPropertyPageImpl<CTaskBarSettingsPage>)
//     END_MSG_MAP()
//
//     CTaskBarSettingsPage(HWND hwnd):
//         m_hbmpTaskbar(NULL),
//         m_hwndTaskbar(hwnd)
//     {
//     }
//
//     ~CTaskBarSettingsPage()
//     {
//         if (m_hbmpTaskbar)
//             DeleteObject(m_hbmpTaskbar);
//     }
//
//     LRESULT OnInitDialog(UINT uMsg, WPARAM wParam, LPARAM lParam, BOOL &bHandled)
//     {
//         CheckDlgButton(IDC_TASKBARPROP_LOCK, g_TaskbarSettings.bLock ? BST_CHECKED : BST_UNCHECKED);
//         CheckDlgButton(IDC_TASKBARPROP_HIDE, g_TaskbarSettings.sr.AutoHide ? BST_CHECKED : BST_UNCHECKED);
//         CheckDlgButton(IDC_TASKBARPROP_ONTOP, g_TaskbarSettings.sr.AlwaysOnTop ? BST_CHECKED : BST_UNCHECKED);
//         CheckDlgButton(IDC_TASKBARPROP_GROUP, g_TaskbarSettings.bGroupButtons ? BST_CHECKED : BST_UNCHECKED);
//         //CheckDlgButton(IDC_TASKBARPROP_SHOWQL, g_TaskbarSettings.bShowQuickLaunch ? BST_CHECKED : BST_UNCHECKED);
//         CheckDlgButton(IDC_TASKBARPROP_SMALLICONS, g_TaskbarSettings.bSmallIcons ? BST_CHECKED : BST_UNCHECKED);
//
//         _UpdateDialog();
//         return TRUE;
//     }
//
//     LRESULT OnCtrlCommand(WORD wNotifyCode, WORD wID, HWND hWndCtl, BOOL &bHandled)
//     {
//         _UpdateDialog();
//         SetModified(TRUE);
//         return 0;
//     }
//
//     int OnApply()
//     {
//         TaskbarSettings newSettings = g_TaskbarSettings;
//
//         newSettings.bLock = IsDlgButtonChecked(IDC_TASKBARPROP_LOCK);
//         newSettings.sr.AutoHide = IsDlgButtonChecked(IDC_TASKBARPROP_HIDE);
//         newSettings.sr.AlwaysOnTop = IsDlgButtonChecked(IDC_TASKBARPROP_ONTOP);
//         newSettings.bGroupButtons = IsDlgButtonChecked(IDC_TASKBARPROP_GROUP);
//         //newSettings.bShowQuickLaunch = IsDlgButtonChecked(IDC_TASKBARPROP_SHOWQL);
//         newSettings.bSmallIcons = IsDlgButtonChecked(IDC_TASKBARPROP_SMALLICONS);
//
//         SendMessage(m_hwndTaskbar, TWM_SETTINGSCHANGED, 0, (LPARAM)&newSettings);
//
//         return PSNRET_NOERROR;
//     }
// };
//
// class CStartMenuSettingsPage : public CPropertyPageImpl<CStartMenuSettingsPage>
// {
// private:
//     HBITMAP m_hbmpStartBitmap;
//
//     void _UpdateDialog()
//     {
//         HWND hwndCustomizeClassic = GetDlgItem(IDC_TASKBARPROP_STARTMENUCLASSICCUST);
//         HWND hwndCustomizeModern = GetDlgItem(IDC_TASKBARPROP_STARTMENUCUST);
//         HWND hwndStartBitmap = GetDlgItem(IDC_TASKBARPROP_STARTMENU_BITMAP);
//         HWND hwndModernRadioBtn = GetDlgItem(IDC_TASKBARPROP_STARTMENU);
//         HWND hwndModernText = GetDlgItem(IDC_TASKBARPROP_STARTMENUMODERNTEXT);
//         BOOL policyNoSimpleStartMenu = SHRestricted(REST_NOSTARTPANEL) != 0;
//         BOOL bModern = FALSE;
//
//         /* If NoSimpleStartMenu, disable ability to use Modern Start Menu */
//         if (policyNoSimpleStartMenu)
//         {
//             /* Switch to classic */
//             CheckDlgButton(IDC_TASKBARPROP_STARTMENUCLASSIC, BST_CHECKED);
//
//             /* Disable radio button */
//             ::EnableWindow(hwndModernRadioBtn, FALSE);
//
//             /* Hide controls related to modern menu */
//             ::ShowWindow(hwndModernRadioBtn, SW_HIDE);
//             ::ShowWindow(hwndModernText, SW_HIDE);
//             ::ShowWindow(hwndCustomizeModern, SW_HIDE);
//         }
//         /* If no restrictions, then get bModern from dialog */
//         else
//         {
//             bModern = IsDlgButtonChecked(IDC_TASKBARPROP_STARTMENU);
//         }
//
//         ::EnableWindow(hwndCustomizeModern, bModern);
//         ::EnableWindow(hwndCustomizeClassic, !bModern);
//
//         UINT uImageId = bModern ? IDB_STARTPREVIEW : IDB_STARTPREVIEW_CLASSIC;
//         SetBitmap(hwndStartBitmap, &m_hbmpStartBitmap, uImageId);
//     }
//
// public:
//     enum { IDD = IDD_TASKBARPROP_STARTMENU };
//
//     BEGIN_MSG_MAP(CTaskBarSettingsPage)
//         MESSAGE_HANDLER(WM_INITDIALOG, OnInitDialog)
//         COMMAND_ID_HANDLER(IDC_TASKBARPROP_STARTMENUCLASSICCUST, OnStartMenuCustomize)
//         CHAIN_MSG_MAP(CPropertyPageImpl<CStartMenuSettingsPage>)
//     END_MSG_MAP()
//
//     CStartMenuSettingsPage():
//         m_hbmpStartBitmap(NULL)
//     {
//     }
//
//     ~CStartMenuSettingsPage()
//     {
//         if (m_hbmpStartBitmap)
//             DeleteObject(m_hbmpStartBitmap);
//     }
//
//     LRESULT OnInitDialog(UINT uMsg, WPARAM wParam, LPARAM lParam, BOOL &bHandled)
//     {
//         BOOL modern = SHELL_GetSetting(SSF_STARTPANELON, fStartPanelOn);
//         CheckDlgButton(modern ? IDC_TASKBARPROP_STARTMENU : IDC_TASKBARPROP_STARTMENUCLASSIC, BST_CHECKED);
//         _UpdateDialog();
//         return TRUE;
//     }
//
//     LRESULT OnStartMenuCustomize(WORD wNotifyCode, WORD wID, HWND hWndCtl, BOOL &bHandled)
//     {
//         ShowCustomizeClassic(hExplorerInstance, m_hWnd);
//         return 0;
//     }
//
//     int OnApply()
//     {
//         SHELLSTATE ss;
//         ss.fStartPanelOn = !IsDlgButtonChecked(IDC_TASKBARPROP_STARTMENUCLASSIC);
//         SHGetSetSettings(&ss, SSF_STARTPANELON, TRUE);
//         return PSNRET_NOERROR;
//     }
// };
//
// class CNotifySettingsPage : public CPropertyPageImpl<CNotifySettingsPage>
// {
// private:
//     HBITMAP m_hbmpTray;
//     HWND m_hwndTaskbar;
//     static const WORD wImageIdLookupTable[2][2][2][2];
//
//     void _UpdateDialog()
//     {
//         BOOL bShowClock = IsDlgButtonChecked(IDC_TASKBARPROP_CLOCK);
//         BOOL bShowSeconds = IsDlgButtonChecked(IDC_TASKBARPROP_SECONDS);
//         BOOL bHideInactive = IsDlgButtonChecked(IDC_TASKBARPROP_HIDEICONS);
//         BOOL bShowDesktopButton = IsDlgButtonChecked(IDC_TASKBARPROP_DESKTOP);
//
//         HWND hwndCustomizeNotifyButton = GetDlgItem(IDC_TASKBARPROP_ICONCUST);
//         HWND hwndSeconds = GetDlgItem(IDC_TASKBARPROP_SECONDS);
//         HWND hwndTrayBitmap = GetDlgItem(IDC_TASKBARPROP_NOTIFICATIONBITMAP);
//
//         ::EnableWindow(hwndCustomizeNotifyButton, bHideInactive);
//         ::EnableWindow(hwndSeconds, bShowClock);
//         if (!bShowSeconds)
//             CheckDlgButton(IDC_TASKBARPROP_SECONDS, BST_UNCHECKED);
//         UINT uImageId = wImageIdLookupTable[bShowClock][bShowSeconds][bHideInactive][bShowDesktopButton];
//
//         SetBitmap(hwndTrayBitmap, &m_hbmpTray, uImageId);
//     }
//
// public:
//     enum { IDD = IDD_TASKBARPROP_NOTIFY };
//
//     BEGIN_MSG_MAP(CNotifySettingsPage)
//         MESSAGE_HANDLER(WM_INITDIALOG, OnInitDialog)
//         COMMAND_ID_HANDLER(IDC_TASKBARPROP_ICONCUST, OnCustomizeTrayIcons)
//         COMMAND_RANGE_HANDLER(IDC_TASKBARPROP_NOTIFY_FIRST_CMD, IDC_TASKBARPROP_NOTIFY_LAST_CMD, OnCtrlCommand)
//         CHAIN_MSG_MAP(CPropertyPageImpl<CNotifySettingsPage>)
//     END_MSG_MAP()
//
//     CNotifySettingsPage(HWND hwnd):
//         m_hbmpTray(NULL),
//         m_hwndTaskbar(hwnd)
//     {
//     }
//
//     ~CNotifySettingsPage()
//     {
//         if (m_hbmpTray)
//             DeleteObject(m_hbmpTray);
//     }
//
//     LRESULT OnInitDialog(UINT uMsg, WPARAM wParam, LPARAM lParam, BOOL &bHandled)
//     {
//         ::EnableWindow(GetDlgItem(IDC_TASKBARPROP_CLOCK), !SHRestricted(REST_HIDECLOCK));
//         CheckDlgButton(IDC_TASKBARPROP_CLOCK, (!GetHideClock()) ? BST_CHECKED : BST_UNCHECKED);
//         CheckDlgButton(IDC_TASKBARPROP_SECONDS, g_TaskbarSettings.bShowSeconds ? BST_CHECKED : BST_UNCHECKED);
//         CheckDlgButton(IDC_TASKBARPROP_HIDEICONS, g_TaskbarSettings.bHideInactiveIcons ? BST_CHECKED : BST_UNCHECKED);
//         CheckDlgButton(IDC_TASKBARPROP_DESKTOP, g_TaskbarSettings.bShowDesktopButton ? BST_CHECKED : BST_UNCHECKED);
//
//         // TODO: bHideInactiveIcons is not implemented yet, just disable for now
//         CheckDlgButton(IDC_TASKBARPROP_HIDEICONS, BST_UNCHECKED);
//         ::EnableWindow(GetDlgItem(IDC_TASKBARPROP_HIDEICONS), FALSE);
//
//         _UpdateDialog();
//         return TRUE;
//     }
//
//     LRESULT OnCustomizeTrayIcons(WORD wNotifyCode, WORD wID, HWND hWndCtl, BOOL &bHandled)
//     {
//         ShowCustomizeNotifyIcons(hExplorerInstance, m_hWnd);
//         return 0;
//     }
//
//     LRESULT OnCtrlCommand(WORD wNotifyCode, WORD wID, HWND hWndCtl, BOOL &bHandled)
//     {
//         _UpdateDialog();
//         SetModified(TRUE);
//         return 0;
//     }
//
//     int OnApply()
//     {
//         TaskbarSettings newSettings = g_TaskbarSettings;
//
//         newSettings.sr.HideClock = !IsDlgButtonChecked(IDC_TASKBARPROP_CLOCK);
//         newSettings.bShowSeconds = IsDlgButtonChecked(IDC_TASKBARPROP_SECONDS);
//         newSettings.bHideInactiveIcons = IsDlgButtonChecked(IDC_TASKBARPROP_HIDEICONS);
//         newSettings.bShowDesktopButton = IsDlgButtonChecked(IDC_TASKBARPROP_DESKTOP);
//
//         SendMessage(m_hwndTaskbar, TWM_SETTINGSCHANGED, 0, (LPARAM)&newSettings);
//
//         return PSNRET_NOERROR;
//     }
// };
//
// const WORD CNotifySettingsPage::wImageIdLookupTable[2][2][2][2] =
// {
//     {
//         {
//             {IDB_SYSTRAYPROP_SHOW_NOCLOCK_NODESK, IDB_SYSTRAYPROP_SHOW_NOCLOCK_DESK},
//             {IDB_SYSTRAYPROP_HIDE_NOCLOCK_NODESK, IDB_SYSTRAYPROP_HIDE_NOCLOCK_DESK}
//         },
//         {
//             {IDB_SYSTRAYPROP_SHOW_NOCLOCK_NODESK, IDB_SYSTRAYPROP_SHOW_NOCLOCK_DESK},
//             {IDB_SYSTRAYPROP_HIDE_NOCLOCK_NODESK, IDB_SYSTRAYPROP_HIDE_NOCLOCK_DESK}
//         }
//     },
//     {
//         {
//             {IDB_SYSTRAYPROP_SHOW_CLOCK_NODESK, IDB_SYSTRAYPROP_SHOW_CLOCK_DESK},
//             {IDB_SYSTRAYPROP_HIDE_CLOCK_NODESK, IDB_SYSTRAYPROP_HIDE_CLOCK_DESK}
//         },
//         {
//             {IDB_SYSTRAYPROP_SHOW_SECONDS_NODESK, IDB_SYSTRAYPROP_SHOW_SECONDS_DESK},
//             {IDB_SYSTRAYPROP_HIDE_SECONDS_NODESK, IDB_SYSTRAYPROP_HIDE_SECONDS_DESK}
//         }
//     }
// };
//
// static int __stdcall
// PropSheetProc(HWND hwndDlg, UINT uMsg, LPARAM lParam)
// {
//     // NOTE: This callback is needed to set large icon correctly.
//     HICON hIcon;
//     switch (uMsg)
//     {
//         case PSCB_INITIALIZED:
//         {
//             hIcon = LoadIconW(hExplorerInstance, MAKEINTRESOURCEW(IDI_STARTMENU));
//             SendMessageW(hwndDlg, WM_SETICON, ICON_BIG, (LPARAM)hIcon);
//             break;
//         }
//     }
//     return 0;
// }
//
// VOID
// DisplayTrayProperties(IN HWND hwndOwner, IN HWND hwndTaskbar)
// {
//     PROPSHEETHEADER psh;
//     CSimpleArray<HPROPSHEETPAGE> hpsp;
//     CTaskBarSettingsPage tbSettingsPage(hwndTaskbar);
//     CStartMenuSettingsPage smSettingsPage;
//     CNotifySettingsPage naSettingsPage(hwndTaskbar);
//     CStringW caption;
//
//     caption.LoadStringW(IDS_TASKBAR_STARTMENU_PROP_CAPTION);
//
//     hpsp.Add(tbSettingsPage.Create());
//     hpsp.Add(smSettingsPage.Create());
//     hpsp.Add(naSettingsPage.Create());
//
//     ZeroMemory(&psh, sizeof(psh));
//     psh.dwSize = sizeof(psh);
//     psh.dwFlags =  PSH_PROPTITLE | PSH_USEICONID | PSH_USECALLBACK;
//     psh.hwndParent = hwndOwner;
//     psh.hInstance = hExplorerInstance;
//     psh.pszIcon = MAKEINTRESOURCEW(IDI_STARTMENU);
//     psh.pszCaption = caption.GetString();
//     psh.nPages = hpsp.GetSize();
//     psh.nStartPage = 0;
//     psh.phpage = hpsp.GetData();
//     psh.pfnCallback = PropSheetProc;
//
//     PropertySheet(&psh);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
