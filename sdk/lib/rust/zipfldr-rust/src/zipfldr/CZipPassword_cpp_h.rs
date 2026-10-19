use crate::prelude::*;
use core::ffi::c_void;



#[no_mangle]
pub unsafe extern "C" fn _CZipAskPassword(hDlg: HWND, mut filename: PCWSTR, Password: &mut CStringA) -> eZipPasswordResponse {
    if !filename.is_null() {
        filename = PathFindFileNameW(filename);
    }
    let mut password = CZipPassword__make_0(filename, Password as *mut CStringA);
    let Result: INT_PTR = password.DoModal(hDlg, 0);
    match Result {
        __c if __c == IDOK as _ => return eAccept,
        __c if __c == IDSKIP as _ => return eSkip,
        __c if __c == IDCANCEL as _ => return eAbort,
        _ => return eAbort,
    }
}

impl CZipPassword {
    pub unsafe fn __ctor_0(&mut self, filename: PCWSTR, Password: *mut CStringA) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0(core::ptr::null_mut());
        (*this_).__init_vtables();
        (*this_).m_pPassword = Password;
        {
            if !filename.is_null() {
                (*this_).m_Filename = CStringW::from_ptr(filename);
            }
        }
    }
    pub unsafe fn OnInitDialog(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        (*this_).__base0.CenterWindow((*this_).__base0.GetParent().op_to_HWND());

        /* No filename, so this is the question before starting to extract */
        if (*this_).m_Filename.IsEmpty() {
            let mut message = CStringW::from_resource(IDS_PASSWORD_ZIP_TEXT);
            SetDlgItemTextW((*this_).__base0.m_hWnd, (IDC_MESSAGE as i32), message.GetString());
            ShowWindow((*this_).__base0.GetDlgItem((IDSKIP as i32)), (SW_HIDE as i32));
        } else {
            let mut message = CStringW::new();
            message.FormatMessage(IDS_PASSWORD_FILE_TEXT, (*this_).m_Filename.GetString()); // %1 insert
            SetDlgItemTextW((*this_).__base0.m_hWnd, (IDC_MESSAGE as i32), message.GetString());
        }
        return (TRUE) as LRESULT;
    }
    pub unsafe fn OnButton(&mut self, wNotifyCode: WORD, wID: WORD, hWndCtl: HWND, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        if wID == (IDOK as WORD) {
            let item: HWND = (*this_).__base0.GetDlgItem((IDC_PASSWORD as i32));
            let mut len: i32 = GetWindowTextLengthA(item);
            len = (GetDlgItemTextA((*this_).__base0.m_hWnd, (IDC_PASSWORD as i32), (*(*this_).m_pPassword).GetBuffer(len + 1), len + 1)) as i32;
            (*(*this_).m_pPassword).ReleaseBufferSetLength(len);
        }
        (*this_).EndDialog((wID) as _);
        return 0;
    }
}
