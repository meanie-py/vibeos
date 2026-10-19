use crate::prelude::*;
use core::ffi::c_void;



#[no_mangle]
pub unsafe extern "C" fn _CZipAskReplace(hDlg: HWND, FullPath: PCWSTR) -> eZipConfirmResponse {
    let Filename: PCWSTR = PathFindFileNameW(FullPath);
    let mut confirm: CConfirmReplace = CConfirmReplace__make_0(Filename);
    let Result: INT_PTR = confirm.DoModal(hDlg, 0);
    match Result {
        __c if __c == IDYES as _ => return eYes,
        __c if __c == IDYESALL as _ => return eYesToAll,
        __c if __c == IDNO as _ => return eNo,
        __c if __c == IDCANCEL as _ => return eCancel,
        _ => return eNo,
    }
}

impl CConfirmReplace {
    pub unsafe fn __ctor_0(&mut self, filename: PCWSTR) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0(core::ptr::null_mut());
        (*this_).__init_vtables();
        (*this_).m_Filename = CStringW::from_ptr(filename);
        {
        }
    }
    pub unsafe fn OnInitDialog(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        (*this_).__base0.CenterWindow((*this_).__base0.GetParent().op_to_HWND());

        let hIcon: HICON = LoadIcon(core::ptr::null_mut(), IDI_EXCLAMATION);
        (*this_).__base0.SendDlgItemMessageW((IDC_EXCLAMATION_ICON as i32), STM_SETICON, hIcon as WPARAM, 0);

        let mut message = CStringW::new();
        message.FormatMessage(IDS_OVERWRITEFILE_TEXT, (*this_).m_Filename.GetString());
        SetDlgItemTextW((*this_).__base0.m_hWnd, (IDC_MESSAGE as i32), message.GetString());

        return (TRUE) as LRESULT;
    }
    pub unsafe fn OnButton(&mut self, wNotifyCode: WORD, wID: WORD, hWndCtl: HWND, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
            (*this_).EndDialog(wID as INT_PTR);
            return 0;
        }
}
