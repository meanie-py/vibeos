use crate::prelude::*;
use core::ffi::c_void;






impl CicArrayBase {
    pub unsafe fn __ctor_0(&mut self, cbItem: size_t) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        {
            (*this_).m_cbItem = cbItem;
            (*this_).m_pb = core::ptr::null_mut();
            (*this_).m_cCapacity = 0;
            (*this_).m_cItems = 0;
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        cicMemFree(((*this_).m_pb).cast());
    }
    pub unsafe fn Insert(&mut self, iItem: size_t, cGrow: size_t) -> BOOL {
        let this_: *mut Self = self;
        let mut cNewCapacity: size_t = (*this_).m_cItems + cGrow;
        if (*this_).m_cCapacity < cNewCapacity {
            if cNewCapacity <= (*this_).m_cItems + (*this_).m_cItems / 2 {
                cNewCapacity = (*this_).m_cItems + (*this_).m_cItems / 2;
            }

            let pbNew: LPBYTE = cicMemReAlloc((*this_).m_pb as LPVOID, cNewCapacity * (*this_).m_cbItem) as LPBYTE;
            if pbNew.is_null() {
                return (FALSE) as BOOL;
            }

            (*this_).m_pb = pbNew;
            (*this_).m_cCapacity = cNewCapacity;
        }

        if iItem < (*this_).m_cItems {
            MoveMemory!(
                (*this_).m_pb.add((cGrow + iItem) * (*this_).m_cbItem) as LPVOID,
                (*this_).m_pb.add(iItem * (*this_).m_cbItem) as LPVOID,
                ((*this_).m_cItems - iItem) * (*this_).m_cbItem,
            );
        }

        (*this_).m_cItems += cGrow;
        return (TRUE) as BOOL;
    }
    pub unsafe fn Append(&mut self, cGrow: size_t) -> LPVOID {
        let this_: *mut Self = self;
        if (*this_).Insert((*this_).m_cItems, cGrow) == 0 {
            return core::ptr::null_mut();
        }
        return (*this_).m_pb.add(((*this_).m_cItems - cGrow) * (*this_).m_cbItem) as LPVOID;
    }
    pub unsafe fn Remove(&mut self, iItem: size_t, cRemove: size_t) {
        let this_: *mut Self = self;
        if iItem + cRemove < (*this_).m_cItems {
            MoveMemory!(
                (*this_).m_pb.add(iItem * (*this_).m_cbItem) as LPVOID,
                (*this_).m_pb.add((iItem + cRemove) * (*this_).m_cbItem) as *const core::ffi::c_void,
                ((*this_).m_cItems - iItem - cRemove) * (*this_).m_cbItem,
            );
        }

        (*this_).m_cItems -= cRemove;

        let cHalfCapacity: size_t = (*this_).m_cCapacity / 2;
        if cHalfCapacity <= (*this_).m_cItems {
            return;
        }

        let pb: LPBYTE = cicMemReAlloc((*this_).m_pb as LPVOID, cHalfCapacity * (*this_).m_cbItem) as LPBYTE;
        if !pb.is_null() {
            (*this_).m_pb = pb;
            (*this_).m_cCapacity = cHalfCapacity;
        }
    }
}
