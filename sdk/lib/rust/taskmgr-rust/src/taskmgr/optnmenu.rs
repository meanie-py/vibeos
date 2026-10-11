use crate::taskmgr::procpage::RefreshProcessPage;
use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn TaskManager_OnOptionsAlwaysOnTop() {
    let mut hMenu: HMENU = core::ptr::null_mut();
    let mut hOptionsMenu: HMENU = core::ptr::null_mut();

    hMenu = GetMenu(hMainWnd);
    hOptionsMenu = GetSubMenu(hMenu, (OPTIONS_MENU_INDEX as i32));

    /*
     * Check or uncheck the always on top menu item
     * and update main window.
     */
    if (GetWindowLongPtrW(hMainWnd, GWL_EXSTYLE) & (WS_EX_TOPMOST) as LONG_PTR) != 0
    {
        CheckMenuItem(hOptionsMenu, ID_OPTIONS_ALWAYSONTOP, MF_BYCOMMAND|MF_UNCHECKED);
        TaskManagerSettings.AlwaysOnTop = (FALSE) as i32;
        SetWindowPos(hMainWnd, HWND_NOTOPMOST, 0, 0, 0, 0, SWP_NOMOVE|SWP_NOSIZE);
    }
    else
    {
        CheckMenuItem(hOptionsMenu, ID_OPTIONS_ALWAYSONTOP, MF_BYCOMMAND|MF_CHECKED);
        TaskManagerSettings.AlwaysOnTop = (TRUE) as i32;
        SetWindowPos(hMainWnd, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOMOVE|SWP_NOSIZE);
    }
}

#[no_mangle]
pub unsafe extern "C" fn TaskManager_OnOptionsMinimizeOnUse() {
    let mut hMenu: HMENU = core::ptr::null_mut();
    let mut hOptionsMenu: HMENU = core::ptr::null_mut();

    hMenu = GetMenu(hMainWnd);
    hOptionsMenu = GetSubMenu(hMenu, (OPTIONS_MENU_INDEX as i32));

    /*
     * Check or uncheck the minimize on use menu item.
     */
    if GetMenuState(hOptionsMenu, ID_OPTIONS_MINIMIZEONUSE, MF_BYCOMMAND) & MF_CHECKED != 0
    {
        CheckMenuItem(hOptionsMenu, ID_OPTIONS_MINIMIZEONUSE, MF_BYCOMMAND | MF_UNCHECKED);
        TaskManagerSettings.MinimizeOnUse = (FALSE) as i32;
    }
    else
    {
        CheckMenuItem(hOptionsMenu, ID_OPTIONS_MINIMIZEONUSE, MF_BYCOMMAND | MF_CHECKED);
        TaskManagerSettings.MinimizeOnUse = (TRUE) as i32;
    }
}
#[no_mangle]
pub unsafe extern "C" fn TaskManager_OnOptionsHideWhenMinimized() {
    let mut hMenu: HMENU = core::ptr::null_mut();
    let mut hOptionsMenu: HMENU = core::ptr::null_mut();

    hMenu = GetMenu(hMainWnd);
    hOptionsMenu = GetSubMenu(hMenu, (OPTIONS_MENU_INDEX as i32));

    /*
     * Check or uncheck the hide when minimized menu item.
     */
    if (GetMenuState(hOptionsMenu, ID_OPTIONS_HIDEWHENMINIMIZED, MF_BYCOMMAND) & MF_CHECKED) != 0
    {
        CheckMenuItem(hOptionsMenu, ID_OPTIONS_HIDEWHENMINIMIZED, MF_BYCOMMAND|MF_UNCHECKED);
        TaskManagerSettings.HideWhenMinimized = (FALSE) as i32;
    }
    else
    {
        CheckMenuItem(hOptionsMenu, ID_OPTIONS_HIDEWHENMINIMIZED, MF_BYCOMMAND|MF_CHECKED);
        TaskManagerSettings.HideWhenMinimized = (TRUE) as i32;
    }
}
#[no_mangle]
pub unsafe extern "C" fn TaskManager_OnOptionsShow16BitTasks() {
    let mut hMenu: HMENU = core::ptr::null_mut();
    let mut hOptionsMenu: HMENU = core::ptr::null_mut();

    hMenu = GetMenu(hMainWnd);
    hOptionsMenu = GetSubMenu(hMenu, (OPTIONS_MENU_INDEX as i32));

    /*
     * FIXME: Currently this is useless because the
     * current implementation doesn't list the 16-bit
     * processes. I believe that would require querying
     * each ntvdm.exe process for it's children.
     */

    /*
     * Check or uncheck the show 16-bit tasks menu item
     */
    if (GetMenuState(hOptionsMenu, ID_OPTIONS_SHOW16BITTASKS, MF_BYCOMMAND) & MF_CHECKED) != 0
    {
        CheckMenuItem(hOptionsMenu, ID_OPTIONS_SHOW16BITTASKS, (MF_BYCOMMAND | MF_UNCHECKED));
        TaskManagerSettings.Show16BitTasks = (FALSE) as i32;
    }
    else
    {
        CheckMenuItem(hOptionsMenu, ID_OPTIONS_SHOW16BITTASKS, (MF_BYCOMMAND | MF_CHECKED));
        TaskManagerSettings.Show16BitTasks = (TRUE) as i32;
    }

    /*
     * Refresh the list of processes.
     */
    RefreshProcessPage();
}
