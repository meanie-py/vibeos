use crate::prelude::*;
use core::ffi::c_void;

extern "system" {
    fn ReleaseMutex(hMutex: HANDLE) -> BOOL;
}







impl CicMutex {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).m_hMutex = core::ptr::null_mut();
        (*this_).m_bInit = (FALSE) as i32;
        {
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        (*this_).Uninit();
    }
    pub unsafe fn Init(&mut self, lpSA: LPSECURITY_ATTRIBUTES, pszMutexName: LPCTSTR) {
        let this_: *mut Self = self;
        (*this_).m_hMutex = CreateMutex(lpSA, (FALSE) as i32, pszMutexName);
        (*this_).m_bInit = (TRUE) as i32;
    }
    pub unsafe fn Uninit(&mut self) {
        let this_: *mut Self = self;
        if !(*this_).m_hMutex.is_null() {
            CloseHandle((*this_).m_hMutex);
            (*this_).m_hMutex = core::ptr::null_mut();
        }
        (*this_).m_bInit = (FALSE) as i32;
    }
    pub unsafe fn Enter(&mut self) -> BOOL {
        let this_: *mut Self = self;
        let dwWait: DWORD = WaitForSingleObject((*this_).m_hMutex, 5000);
        return ((dwWait == WAIT_OBJECT_0) || (dwWait == WAIT_ABANDONED)) as BOOL;
    }
    pub unsafe fn Leave(&mut self) {
        let this_: *mut Self = self;
        ReleaseMutex((*this_).m_hMutex);
    }
}
