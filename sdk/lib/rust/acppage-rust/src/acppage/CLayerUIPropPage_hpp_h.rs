use crate::prelude::*;
use crate::acppage::ACPPage_cpp_h::g_ModuleRefCnt;
use core::ffi::c_void;






impl CLayerUIPropPage {
    pub unsafe fn AddPages(&mut self, pfnAddPage: LPFNADDPROPSHEETPAGE, lParam: LPARAM) -> HRESULT {
        let this_: *mut Self = self;
        let hPage: HPROPSHEETPAGE = (*this_).Create();
        if !hPage.is_null() && (pfnAddPage.unwrap_unchecked())(hPage, lParam) == 0 {
            DestroyPropertySheetPage(hPage);
        }

        return S_OK;
    }
    pub unsafe fn ReplacePage(&mut self, arg0: UINT,
        arg1: LPFNADDPROPSHEETPAGE,
        arg2: LPARAM,
    ) -> HRESULT {
        let this_: *mut Self = self;
        E_NOTIMPL
    }
    pub unsafe fn OnPageAddRef(&mut self) {
        let this_: *mut Self = self;
        InterlockedIncrement(core::ptr::addr_of_mut!(g_ModuleRefCnt));
    }
    pub unsafe fn OnPageRelease(&mut self) {
        let this_: *mut Self = self;
        InterlockedDecrement(core::ptr::addr_of_mut!(g_ModuleRefCnt));
    }
}
