use crate::prelude::*;
use crate::fontext::CFontCache_cpp_h::g_FontCache;
use core::ffi::c_void;




impl CFontFolderViewCB {
    pub unsafe fn FilterEvent(&mut self, apidls: *mut PIDLIST_ABSOLUTE, lEvent: LONG) -> BOOL {
        let this_: *mut Self = self;
        let mut lEvent = lEvent;
        lEvent &= (!SHCNE_INTERRUPT) as i32;

        match (lEvent) as u32 {
            SHCNE_CREATE | SHCNE_RENAMEITEM | SHCNE_UPDATEDIR => {
                // Refresh font cache and notify the system about the font change
                if !g_FontCache.is_null() {
                    (*g_FontCache).Read();
                }
            }
            SHCNE_DELETE => {
            }
            _ => {
                return (TRUE) as BOOL; // We don't want this event
            }
        }

        return (FALSE) as BOOL;
    }
    pub unsafe fn Initialize(&mut self, pFontExt: *mut CFontExt, psv: *mut IShellView, pidlParent: LPCITEMIDLIST) {
        let this_: *mut Self = self;
        ATLASSERT!(pFontExt);
        ATLASSERT!(psv);
        ATLASSERT!(pidlParent);
        (*this_).m_pFontExt = pFontExt;
        (*this_).m_pShellView.assign(psv);
        (*this_).m_pidlParent.Attach(ILClone(pidlParent));
        if (*this_).m_pidlParent.is_null() {
            ERR!("m_pidlParent was null\n");
        }
    }
    pub unsafe fn MessageSFVCB(&mut self, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> HRESULT {
        let this_: *mut Self = self;
        if uMsg == SFVM_QUERYFSNOTIFY {
            // Registering change notification
            if (*this_).m_pShellView.is_null() || (*this_).m_pFontExt.is_null() {
                return E_FAIL;
            }
            // Now, we can get the view window
            (*(*this_).m_pShellView.Get()).GetWindow(core::ptr::addr_of_mut!((*this_).m_hwndView));
            (*(*this_).m_pFontExt).SetViewWindow((*this_).m_hwndView);
            return S_OK;
        }
        if uMsg == SFVM_FSNOTIFY {
            // Change notification
            if (*this_).FilterEvent(wParam as *mut PIDLIST_ABSOLUTE, lParam as LONG) != 0 {
                return S_FALSE; // Don't process
            }
            return S_OK;
        }
        return E_NOTIMPL;
    }
}
