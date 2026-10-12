use crate::prelude::*;
use core::ffi::c_void;




impl CCustomizeDialog {
    pub unsafe fn __ctor_0(&mut self, MainWnd: *mut CMainWnd, console: *mut CConsoleWnd) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0(core::ptr::null_mut());
        (*this_).__init_vtables();
        {
            (*this_).m_MainWnd = MainWnd;
            (*this_).m_Console = console;
        }
    }
    pub unsafe fn OnInitDialog(&mut self, nMessage: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        (*this_).__base0.CenterWindow((*(*this_).m_Console).__base0.m_hWnd);
    
        (*this_).__base0.CheckDlgButton((IDC_CUSTOMIZE_SCOPEPANE as i32), if (((*(*this_).m_Console).IsTreeViewVisible()) != 0) { BST_CHECKED } else { BST_UNCHECKED });
        (*this_).__base0.CheckDlgButton((IDC_CUSTOMIZE_MENUS as i32), if (((*(*this_).m_MainWnd).AreStandardMenusVisible()) != 0) { BST_CHECKED } else { BST_UNCHECKED });
        (*this_).__base0.CheckDlgButton((IDC_CUSTOMIZE_TOOLBAR as i32), if (((*(*this_).m_MainWnd).IsToolBarVisible()) != 0) { BST_CHECKED } else { BST_UNCHECKED });
        (*this_).__base0.CheckDlgButton((IDC_CUSTOMIZE_STATUSBAR as i32), if (((*(*this_).m_Console).IsStatusBarVisible()) != 0) { BST_CHECKED } else { BST_UNCHECKED });
        (*this_).__base0.CheckDlgButton((IDC_CUSTOMIZE_DESCRIPTIONBAR as i32), if (((*(*this_).m_Console).IsDescriptionBarVisible()) != 0) { BST_CHECKED } else { BST_UNCHECKED });
        (*this_).__base0.CheckDlgButton((IDC_CUSTOMIZE_ACTIONSPANE as i32), if (((*(*this_).m_Console).IsActionsPaneVisible()) != 0) { BST_CHECKED } else { BST_UNCHECKED });
    
        EnableWindow((*this_).__base0.GetDlgItem((IDC_CUSTOMIZE_MENUS as i32)), (FALSE as i32));
        EnableWindow((*this_).__base0.GetDlgItem((IDC_CUSTOMIZE_TABS as i32)), (FALSE as i32));
        EnableWindow((*this_).__base0.GetDlgItem((IDC_CUSTOMIZE_SNAPIN_MENUS as i32)), (FALSE as i32));
        EnableWindow((*this_).__base0.GetDlgItem((IDC_CUSTOMIZE_SNAPIN_TOOLBARS as i32)), (FALSE as i32));
    
        return 0;
    }
    pub unsafe fn OnCommand(&mut self, wNotifyCode: WORD, wID: WORD, hWndCtl: HWND, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        match (wID) as u32 {
            IDC_CUSTOMIZE_SCOPEPANE => {
                (*(*this_).m_Console).SetTreeViewVisible(if (*this_).__base0.IsDlgButtonChecked((IDC_CUSTOMIZE_SCOPEPANE as i32)) == BST_CHECKED { (TRUE as i32) } else { (FALSE as i32) });
                return 0;
            }
    
            IDC_CUSTOMIZE_MENUS => {
                (*(*this_).m_MainWnd).SetStandardMenusVisible(if (*this_).__base0.IsDlgButtonChecked((IDC_CUSTOMIZE_MENUS as i32)) == BST_CHECKED { (TRUE as i32) } else { (FALSE as i32) });
                return 0;
            }
    
            IDC_CUSTOMIZE_TOOLBAR => {
                (*(*this_).m_MainWnd).SetToolBarVisible(if (*this_).__base0.IsDlgButtonChecked((IDC_CUSTOMIZE_TOOLBAR as i32)) == BST_CHECKED { (TRUE as i32) } else { (FALSE as i32) });
                return 0;
            }
    
            IDC_CUSTOMIZE_STATUSBAR => {
                (*(*this_).m_Console).SetStatusBarVisible(if (*this_).__base0.IsDlgButtonChecked((IDC_CUSTOMIZE_STATUSBAR as i32)) == BST_CHECKED { (TRUE as i32) } else { (FALSE as i32) });
                return 0;
            }
    
            IDC_CUSTOMIZE_DESCRIPTIONBAR => {
                (*(*this_).m_Console).SetDescriptionBarVisible(if (*this_).__base0.IsDlgButtonChecked((IDC_CUSTOMIZE_DESCRIPTIONBAR as i32)) == BST_CHECKED { (TRUE as i32) } else { (FALSE as i32) });
                return 0;
            }
    
            IDC_CUSTOMIZE_ACTIONSPANE => {
                (*(*this_).m_Console).SetActionsPaneVisible(if (*this_).__base0.IsDlgButtonChecked((IDC_CUSTOMIZE_ACTIONSPANE as i32)) == BST_CHECKED { (TRUE as i32) } else { (FALSE as i32) });
                return 0;
            }
    
            IDOK => {
                (*this_).EndDialog((IDOK as _));
                return 0;
            }
    
            _ => {}
        }
    
        return 0;
    }
}
