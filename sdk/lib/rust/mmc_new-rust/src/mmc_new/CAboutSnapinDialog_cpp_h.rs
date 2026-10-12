use crate::prelude::*;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_80E09B2F645F: [WCHAR; 3] = [0x000D, 0x000A, 0x0000];

use core::ffi::c_void;




impl CAboutSnapinDialog {
    pub unsafe fn OnInitDialog(&mut self, nMessage: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        (*this_).__base0.CenterWindow((*(*this_).m_MainWnd).__base0.m_hWnd);
    
        (*this_).m_SnapinIcon.Attach((*this_).__base0.GetDlgItem((IDC_ABOUT_ICON as i32)));
        (*this_).m_SnapinInfo.Attach((*this_).__base0.GetDlgItem((IDC_ABOUT_INFO as i32)));
        (*this_).m_Description.Attach((*this_).__base0.GetDlgItem((IDC_ABOUT_DESCRIPTION as i32)));
    
        (*this_).m_SnapinIcon.SendMessageW__0(STM_SETIMAGE, (IMAGE_ICON) as WPARAM, (*(*(*this_).m_Snapin).CacheEntry()).Icon() as LPARAM);
    
        let mut Info: CStringW = (*(*this_).m_Snapin).Name().clone();
        Info.Append(CPP_COM_WSTR_80E09B2F645F.as_ptr());
        Info.Append((*(*(*this_).m_Snapin).CacheEntry()).Provider().GetString());
        if !(*(*(*this_).m_Snapin).CacheEntry()).Version().IsEmpty() {
            let mut VersionString: CStringW = CStringW::from_resource(IDS_ABOUT_VERSION);
    
            Info.Append(CPP_COM_WSTR_80E09B2F645F.as_ptr());
            Info.Append(VersionString.GetString()/*L"Version: "*/);
            Info.Append((*(*(*this_).m_Snapin).CacheEntry()).Version().GetString());
        }
        (*this_).m_SnapinInfo.SetWindowTextW(Info.GetString());
    
        (*this_).m_Description.SetWindowTextW((*(*(*this_).m_Snapin).CacheEntry()).Description().GetString() as LPWSTR);
    
        return 0;
    }
    pub unsafe fn OnClose(&mut self, nMessage: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        (*this_).EndDialog((IDOK as _));
        return 0;
    }
    pub unsafe fn OnCommand(&mut self, wNotifyCode: WORD, wID: WORD, hWndCtl: HWND, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        match (wID) as u32 {
            IDOK => {
                (*this_).EndDialog((IDOK as _));
                return 0;
            }
            _ => {}
        }
    
        return 0;
    }
}
