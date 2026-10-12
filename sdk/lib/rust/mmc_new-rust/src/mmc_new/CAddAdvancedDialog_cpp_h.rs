use crate::prelude::*;
use core::ffi::c_void;



impl CAddAdvancedDialog {
    pub unsafe fn OnInitDialog(&mut self, nMessage: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        (*this_).__base0.CenterWindow((*(*this_).m_AddDialog).__base0.m_hWnd);
        let state: UINT = if (*(*this_).m_AddDialog).GetAdvanced() != 0 { BST_CHECKED } else { BST_UNCHECKED };
        (*this_).__base0.CheckDlgButton((IDC_ADVANCED_CHECK as i32), state);
        return 0;
    }
    pub unsafe fn OnCommand(&mut self, wNotifyCode: WORD, wID: WORD, hWndCtl: HWND, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        match (wID) as u32 {
            IDOK => {
                {
                    (*(*this_).m_AddDialog).SetAdvanced(if (*this_).__base0.IsDlgButtonChecked((IDC_ADVANCED_CHECK as i32)) == BST_CHECKED { (TRUE as i32) } else { (FALSE as i32) });
                    (*this_).EndDialog((IDOK as _));
                }
                return 0;
            }
            IDCANCEL => {
                {
                    (*this_).EndDialog((IDCANCEL as _));
                }
                return 0;
            }
            _ => {}
        }
    
        return 0;
    }
}
