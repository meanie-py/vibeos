use crate::prelude::*;
use core::ffi::c_void;















impl tagCTFIMECONTEXT {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;}
}

impl CIC_IMC_LOCK {
    pub unsafe fn __ctor_0(&mut self, hIMC: HIMC) {
        let this_: *mut Self = self;
        {
            (*this_).m_pIC = core::ptr::null_mut();
            (*this_).m_hIMC = hIMC;
            (*this_).m_hr = S_OK;
            (*this_).m_dw3 = 0;
        }
    }
    pub unsafe fn Invalid(&mut self) -> BOOL {
        let this_: *mut Self = self;
        return ((*this_).m_pIC.is_null() || (*this_).m_hr != S_OK) as BOOL;
    }
}

impl CicIMCLock {
    pub unsafe fn __ctor_0(&mut self, hIMC: HIMC) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0(hIMC);
        {
            (*this_).__base0.m_hr = (*this_)._LockIMC(hIMC, core::ptr::addr_of_mut!((*this_).__base0.m_pIC));
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        (*this_).unlock();
    }
    pub unsafe fn unlock(&mut self) {
        let this_: *mut Self = self;
        if !(*this_).__base0.m_pIC.is_null() {
            (*this_)._UnlockIMC((*this_).__base0.m_hIMC);
            (*this_).__base0.m_pIC = core::ptr::null_mut();
        }
    }
    pub unsafe fn InitContext(&mut self) {
        let this_: *mut Self = self;
        if ((*(*this_).__base0.m_pIC).fdwInit & INIT_COMPFORM) == 0 {
            (*(*this_).__base0.m_pIC).cfCompForm.dwStyle = 0;
        }
        let mut i: UINT = 0;
        while i < 4 {
            (*(*this_).__base0.m_pIC).cfCandForm[i as usize].dwStyle = 0;
            i += 1;
        }
    }
    pub unsafe fn ValidCompositionString(&mut self) -> BOOL {
        let this_: *mut Self = self;
        if (ImmGetIMCCSize((*(*this_).__base0.m_pIC).hCompStr) as usize) < core::mem::size_of::<COMPOSITIONSTRING>() {
            return (FALSE) as BOOL;
        }

        let mut imccLock: CicIMCCLock<COMPOSITIONSTRING> = core::mem::zeroed();
        imccLock.__ctor_0((*(*this_).__base0.m_pIC).hCompStr);
        if !imccLock.operator_bool() {
            return (FALSE) as BOOL;
        }

        return (imccLock.get().dwCompStrLen > 0) as BOOL;
    }
    pub unsafe fn UseVerticalCompWindow(&mut self) -> BOOL {
        let this_: *mut Self = self;
        return if (*(*this_).__base0.m_pIC).cfCompForm.dwStyle != 0 && (((*(*this_).__base0.m_pIC).lfFont.A.lfEscapement / 900) % 4 == 3) { 1 } else { 0 };
    }
    pub unsafe fn op_to_INPUTCONTEXTDX(&mut self) -> LPINPUTCONTEXTDX {
        let this_: *mut Self = self;
        return (*this_).__base0.m_pIC;
    }
    pub unsafe fn get(&mut self) -> &'static mut INPUTCONTEXTDX {
        let this_: *mut Self = self;
        return &mut *(*this_).__base0.m_pIC;
    }
    pub unsafe fn _LockIMC(&mut self, hIMC: HIMC, ppIC: *mut LPINPUTCONTEXTDX) -> HRESULT {
        let this_: *mut Self = self;
        if hIMC.is_null() {
            return E_INVALIDARG;
        }

        let pIC: LPINPUTCONTEXTDX = ImmLockIMC(hIMC) as LPINPUTCONTEXTDX;
        *ppIC = pIC;
        return if !pIC.is_null() { S_OK } else { E_FAIL };
    }
    pub unsafe fn _UnlockIMC(&mut self, hIMC: HIMC) -> HRESULT {
        let this_: *mut Self = self;
        if ImmUnlockIMC(hIMC) != 0 { S_OK } else { E_FAIL }
    }
}
