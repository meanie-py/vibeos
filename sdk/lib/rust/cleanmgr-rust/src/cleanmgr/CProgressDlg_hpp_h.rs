use crate::prelude::*;
use core::ffi::c_void;







impl CProgressDlg {
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        (*this_).Stop();
    }
    pub unsafe fn Start(&mut self, dwTotalSteps: DWORD, Title: LPCWSTR, Text: LPCWSTR) {
        let this_: *mut Self = self;
        let hr: HRESULT = CoCreateInstance(&CLSID_ProgressDialog, core::ptr::null_mut(), CLSCTX_INPROC, core::ptr::addr_of!(IID_IProgressDialog), core::ptr::addr_of_mut!((*this_).m_spProgress) as *mut *mut core::ffi::c_void);
        if FAILED_UNEXPECTEDLY!(hr) {
            return;
        }
    
        (*this_).m_dwTotal = dwTotalSteps;
    
        (*(*this_).m_spProgress.p).SetTitle(Title);
        (*(*this_).m_spProgress.p).SetLine(2, Text, (TRUE) as BOOL, core::ptr::null_mut());
        (*(*this_).m_spProgress.p).StartProgressDialog(core::ptr::null_mut(), core::ptr::null_mut(), PROGDLG_NOMINIMIZE, core::ptr::null_mut());
        (*(*this_).m_spProgress.p).SetProgress(0, (*this_).m_dwTotal);
    }
    pub unsafe fn Step(&mut self, dwProgress: DWORD, Text: LPCWSTR) {
        let this_: *mut Self = self;
            let _ = (*(*this_).m_spProgress.p).SetProgress(dwProgress, (*this_).m_dwTotal);
            let _ = (*(*this_).m_spProgress.p).SetLine(1, Text, (TRUE) as BOOL, core::ptr::null_mut());
        }
    pub unsafe fn Stop(&mut self) {
        let this_: *mut Self = self;
        if !(*this_).m_spProgress.p.is_null() {
            (*(*this_).m_spProgress.p).StopProgressDialog();
            (*this_).m_spProgress.Release();
        }
    }
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).m_dwTotal = 0;
    }
}
