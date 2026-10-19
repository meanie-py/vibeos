use crate::prelude::*;
use core::ffi::c_void;


#[no_mangle]
pub unsafe extern "C" fn _CFolderViewCB_CreateInstance(riid: *const IID, ppvOut: *mut LPVOID) -> HRESULT {
    return ShellObjectCreatorIid!(CFolderViewCB, riid, ppvOut);
}

impl CFolderViewCB {
    pub unsafe fn MessageSFVCB(&mut self, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> HRESULT {
        let this_: *mut Self = self;
        let _ = this_;
        let _ = wParam;
        /* TODO: Handle SFVM_GET_WEBVIEW_CONTENT to add tasks */
        match uMsg {
            SFVM_DEFVIEWMODE => {
                let pViewMode: *mut FOLDERVIEWMODE = lParam as *mut FOLDERVIEWMODE;
                *pViewMode = (FVM_DETAILS as i32);
                return S_OK;
            }
            SFVM_COLUMNCLICK => {
                return S_FALSE;
            }
            SFVM_BACKGROUNDENUM => {
                return S_OK;
            }
            _ => {}
        }

        return E_NOTIMPL;
    }
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
    }
}
