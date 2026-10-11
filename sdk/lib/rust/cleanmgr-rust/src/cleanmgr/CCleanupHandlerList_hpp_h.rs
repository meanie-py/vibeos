use crate::prelude::*;
use core::ffi::c_void;



impl CCleanupHandlerList {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;}
}

impl CCleanupHandlerList {
    /// CCleanupHandlerList.hpp's member TEMPLATE `template<typename Fn> void ForEach(Fn
    /// callback)`, hand-ported (cppapp does not generate templates): call `callback` with each
    /// handler, in list order. The C++ callers pass lambdas; here any `FnMut` -- a closure, or
    /// `&mut` one.
    pub unsafe fn ForEach<F: FnMut(*mut CCleanupHandler)>(&mut self, mut callback: F) {
        let mut it: POSITION = self.m_Handlers.GetHeadPosition();
        while !it.is_null() {
            let current: *mut CCleanupHandler = *self.m_Handlers.GetAt(it);
            callback(current);
            self.m_Handlers.GetNext(&mut it);
        }
    }
}
