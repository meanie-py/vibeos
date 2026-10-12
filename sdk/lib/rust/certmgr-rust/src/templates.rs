//! The two `template <typename Fn> void ForEach(Fn callback)` member templates of CStore and
//! CStoreList. Member templates are not lowered by the C++ pipeline (no class-model entry per
//! instantiation), so they are written here by hand, one generic method each, with the C++
//! bodies' order of operations (CStore::ForEach expands first; CStoreList::ForEach loads
//! lazily) and the `for (POSITION it = head; it; list.GetNext(it))` walk. The lambda
//! receives the stored element pointer, as in the header.
use crate::prelude::*;

impl CStore {
    pub unsafe fn ForEach<F: FnMut(*mut CCert)>(&mut self, mut callback: F) {
        let this_: *mut Self = self;
        (*this_).Expand();
        let mut it: POSITION = (*this_).m_Certificates.GetHeadPosition();
        while !it.is_null() {
            let current: *mut CCert = *(*this_).m_Certificates.GetAt(it);
            callback(current);
            (*this_).m_Certificates.GetNext(&mut it);
        }
    }
}

impl CStoreList {
    pub unsafe fn ForEach<F: FnMut(*mut CStore)>(&mut self, mut callback: F) {
        let this_: *mut Self = self;
        if (*this_).m_Stores.IsEmpty() {
            (*this_).LoadStores();
        }
        let mut it: POSITION = (*this_).m_Stores.GetHeadPosition();
        while !it.is_null() {
            let current: *mut CStore = *(*this_).m_Stores.GetAt(it);
            callback(current);
            (*this_).m_Stores.GetNext(&mut it);
        }
    }
}
