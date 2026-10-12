use crate::prelude::*;
use core::ffi::c_void;


impl CAddAdvancedDialog {
    pub unsafe fn __ctor_0(&mut self, AddDialog: *mut CAddDialog) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0(core::ptr::null_mut());
        (*this_).__init_vtables();
        {
            (*this_).m_AddDialog = AddDialog;
        }
    }
}
