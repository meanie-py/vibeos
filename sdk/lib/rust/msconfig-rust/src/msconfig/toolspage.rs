use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut hToolsPage: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

#[no_mangle]
pub static mut hToolsListCtrl: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

#[no_mangle]
pub static mut hToolsDialog: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

#[no_mangle]
pub static mut ListItems_Cmds: [DWORD; 20] = [0; 20];

#[no_mangle]
pub static mut ListItems_Params: [DWORD; 20] = [0; 20];

#[no_mangle]
pub static mut ListItems_Locations: [DWORD; 20] = [0; 20];
#[no_mangle]
pub unsafe extern "C" fn AddItem(name_id: DWORD, descr_id: DWORD, cmd_id: DWORD, param_id: DWORD, csidl: core::ffi::c_int) {
    let mut szTemp: [TCHAR; 256] = [0; 256];
    let mut item: LV_ITEM = core::mem::zeroed();

    LoadString(hInst, name_id, szTemp.as_mut_ptr(), 256);
    memset(core::ptr::addr_of_mut!(item) as *mut core::ffi::c_void, 0, core::mem::size_of::<LV_ITEM>());
    item.mask = LVIF_TEXT;
    item.iImage = 0;
    item.pszText = szTemp.as_mut_ptr();
    item.iItem = ListView_GetItemCount!(hToolsListCtrl);
    item.lParam = 0;
    let _ = ListView_InsertItem!(hToolsListCtrl, core::ptr::addr_of_mut!(item));

    ListItems_Cmds[item.iItem as usize] = cmd_id;
    ListItems_Params[item.iItem as usize] = param_id;
    ListItems_Locations[item.iItem as usize] = csidl as DWORD;

    LoadString(hInst, descr_id, szTemp.as_mut_ptr(), 256);
    item.pszText = szTemp.as_mut_ptr();
    item.iSubItem = 1;
    SendMessage(hToolsListCtrl, (LVM_SETITEMTEXT as u32), item.iItem as WPARAM, core::ptr::addr_of_mut!(item) as LPARAM);
}
#[no_mangle]
pub unsafe extern "C" fn FillListView() {
    AddItem(IDS_TOOLS_CMD_NAME as DWORD, IDS_TOOLS_CMD_DESCR as DWORD, IDS_TOOLS_CMD_CMD as DWORD, IDS_TOOLS_CMD_PARAM as DWORD, (CSIDL_SYSTEM as i32));
    AddItem(IDS_TOOLS_REGEDIT_NAME as DWORD, IDS_TOOLS_REGEDIT_DESCR as DWORD, IDS_TOOLS_REGEDIT_CMD as DWORD, IDS_TOOLS_REGEDIT_PARAM as DWORD, (CSIDL_WINDOWS as i32));
    AddItem(IDS_TOOLS_SYSDM_NAME as DWORD, IDS_TOOLS_SYSDM_DESCR as DWORD, IDS_TOOLS_SYSDM_CMD as DWORD, IDS_TOOLS_SYSDM_PARAM as DWORD, (CSIDL_SYSTEM as i32));
    AddItem(IDS_TOOLS_INFO_NAME as DWORD, IDS_TOOLS_INFO_DESCR as DWORD, IDS_TOOLS_INFO_CMD as DWORD, IDS_TOOLS_INFO_PARAM as DWORD, (CSIDL_SYSTEM as i32));
}
#[no_mangle]
pub unsafe extern "system" fn ToolsPageWndProc(hDlg: HWND, message: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let mut column: LV_COLUMN = core::mem::zeroed();
    let mut szTemp: [TCHAR; (MAX_PATH * 2) as _] = [0; (MAX_PATH * 2) as _];
    let mut szTemp2: [TCHAR; 256] = [0; 256];
    let mut Ptr: *mut TCHAR = core::ptr::null_mut();
    let mut lpnmitem: LPNMITEMACTIVATE = core::ptr::null_mut();
    let mut nmh: LPNMHDR = core::ptr::null_mut();
    let mut dwStyle: DWORD = 0;
    let mut pi: PROCESS_INFORMATION = core::mem::zeroed();
    let mut si: STARTUPINFO = core::mem::zeroed();

    match message {
        WM_INITDIALOG => {
            hToolsListCtrl = GetDlgItem(hDlg, (IDC_TOOLS_LIST as i32));
            hToolsDialog = hDlg;

            dwStyle = SendMessage(hToolsListCtrl, LVM_GETEXTENDEDLISTVIEWSTYLE, 0, 0) as DWORD;
            dwStyle = dwStyle | LVS_EX_FULLROWSELECT;
            SendMessage(hToolsListCtrl, LVM_SETEXTENDEDLISTVIEWSTYLE, 0, (dwStyle) as _);

            SetWindowPos(hDlg, core::ptr::null_mut(), 10, 32, 0, 0, SWP_NOACTIVATE | SWP_NOOWNERZORDER | SWP_NOSIZE | SWP_NOZORDER);

            // Initialize the application page's controls
            column.mask = LVCF_TEXT | LVCF_WIDTH;

            LoadString(hInst, IDS_TOOLS_COLUMN_NAME, szTemp.as_mut_ptr(), 256);
            column.pszText = szTemp.as_mut_ptr();
            column.cx = 150;
            let _ = ListView_InsertColumn!(hToolsListCtrl, 0, core::ptr::addr_of_mut!(column));

            column.mask = LVCF_TEXT | LVCF_WIDTH;
            LoadString(hInst, IDS_TOOLS_COLUMN_DESCR, szTemp.as_mut_ptr(), 256);
            column.pszText = szTemp.as_mut_ptr();
            column.cx = 500;
            let _ = ListView_InsertColumn!(hToolsListCtrl, 1, core::ptr::addr_of_mut!(column));

            FillListView();
            return (TRUE) as INT_PTR;
        }
        WM_COMMAND => {
            match (LOWORD!(wParam)) as u32 {
                IDC_BTN_RUN => {
                    if ListView_GetSelectionMark!(hToolsListCtrl) != -1 {
                        if ((SHGetSpecialFolderPath(core::ptr::null_mut(), szTemp.as_mut_ptr(), ListItems_Locations[ListView_GetSelectionMark!(hToolsListCtrl) as usize] as i32, FALSE as BOOL)) != 0) {
                            Ptr = PathAddBackslash(szTemp.as_mut_ptr());
                        }

                        if Ptr.is_null() {
                            Ptr = szTemp.as_mut_ptr();
                        }

                        szTemp2[0 as usize] = ('\0' as u16);
                        LoadString(hInst, ListItems_Cmds[ListView_GetSelectionMark!(hToolsListCtrl) as usize], Ptr, 256);
                        LoadString(hInst, ListItems_Params[ListView_GetSelectionMark!(hToolsListCtrl) as usize], szTemp2.as_mut_ptr(), 256);
                        if _tcslen(szTemp2.as_mut_ptr()) != 0 {
                            _tcscat(szTemp.as_mut_ptr(), (&[0x0020, 0x0000u16]).as_ptr() /* _T " " */);
                            _tcscat(Ptr, szTemp2.as_mut_ptr());
                        }
                        ZeroMemory!(core::ptr::addr_of_mut!(si), core::mem::size_of::<STARTUPINFO>() as usize);
                        si.cb = core::mem::size_of::<STARTUPINFO>() as DWORD;
                        if CreateProcess(core::ptr::null_mut(), szTemp.as_mut_ptr(), core::ptr::null_mut(), core::ptr::null_mut(), FALSE as BOOL, 0, core::ptr::null_mut(), core::ptr::null_mut(), (core::ptr::addr_of_mut!(si)).cast(), core::ptr::addr_of_mut!(pi)) != 0 {
                            CloseHandle(pi.hThread);
                            CloseHandle(pi.hProcess);
                        }
                    }
                }
                _ => {}
            }
        }
        WM_NOTIFY => {
            nmh = lParam as LPNMHDR;
            if (*nmh).hwndFrom == hToolsListCtrl {
                match (((*nmh).code) as i32) as u32 {
                    __c if __c == LVN_ITEMCHANGED as u32 => {
                        lpnmitem = lParam as LPNMITEMACTIVATE;
                        if (*lpnmitem).iItem > -1 {
                            LoadString(hInst, ListItems_Cmds[(*lpnmitem).iItem as usize], szTemp.as_mut_ptr(), 256);
                            szTemp2[0 as usize] = ('\0' as u16);
                            LoadString(hInst, ListItems_Params[(*lpnmitem).iItem as usize], szTemp2.as_mut_ptr(), 256);
                            _tcscat(szTemp.as_mut_ptr(), (&[0x0020, 0x0000u16]).as_ptr() /* _T " " */);
                            _tcscat(szTemp.as_mut_ptr(), szTemp2.as_mut_ptr());
                            SendDlgItemMessage(hToolsDialog, (IDC_TOOLS_CMDLINE as i32), WM_SETTEXT, 0, szTemp.as_mut_ptr() as LPARAM);
                        }
                    }
                    NM_DBLCLK => {
                        lpnmitem = lParam as LPNMITEMACTIVATE;
                        if (*lpnmitem).iItem > -1 {
                            if ((SHGetSpecialFolderPath(core::ptr::null_mut(), szTemp.as_mut_ptr(), ListItems_Locations[ListView_GetSelectionMark!(hToolsListCtrl) as usize] as i32, FALSE as BOOL)) != 0) {
                                Ptr = PathAddBackslash(szTemp.as_mut_ptr());
                            }

                            if Ptr.is_null() {
                                Ptr = szTemp.as_mut_ptr();
                            }

                            szTemp2[0 as usize] = ('\0' as u16);
                            LoadString(hInst, ListItems_Cmds[(*lpnmitem).iItem as usize], Ptr, 256);
                            LoadString(hInst, ListItems_Params[(*lpnmitem).iItem as usize], szTemp2.as_mut_ptr(), 256);
                            if _tcslen(szTemp2.as_mut_ptr()) != 0 {
                                _tcscat(szTemp.as_mut_ptr(), (&[0x0020, 0x0000u16]).as_ptr() /* _T " " */);
                                _tcscat(Ptr, szTemp2.as_mut_ptr());
                            }
                            ZeroMemory!(core::ptr::addr_of_mut!(si), core::mem::size_of::<STARTUPINFO>() as usize);
                            si.cb = core::mem::size_of::<STARTUPINFO>() as DWORD;
                            if CreateProcess(core::ptr::null_mut(), szTemp.as_mut_ptr(), core::ptr::null_mut(), core::ptr::null_mut(), FALSE as BOOL, 0, core::ptr::null_mut(), core::ptr::null_mut(), (core::ptr::addr_of_mut!(si)).cast(), core::ptr::addr_of_mut!(pi)) != 0 {
                                CloseHandle(pi.hThread);
                                CloseHandle(pi.hProcess);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }
    return 0;
}
