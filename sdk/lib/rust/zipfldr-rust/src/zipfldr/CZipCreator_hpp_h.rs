use crate::prelude::*;
use core::ffi::c_void;


impl CZipCreator {
    pub unsafe fn SetNotifyPidl(&mut self, pidl: PCIDLIST_ABSOLUTE) {
        let this_: *mut Self = self;
            if !pidl.is_null() {
                (*this_).m_pidlNotify.Attach(ILClone(pidl));
            }
        }
}
