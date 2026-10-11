use crate::prelude::*;
use core::ffi::c_void;






impl CStoreList {
    pub unsafe extern "system" fn s_StoreCallback(pvSystemStore: *const core::ffi::c_void, dwFlags: DWORD, pStoreInfo: PCERT_SYSTEM_STORE_INFO, pvReserved: *mut core::ffi::c_void, pvArg: *mut core::ffi::c_void) -> BOOL {
        let storeList: *mut CStoreList = pvArg as *mut CStoreList;
        let storeName: *const wchar_t = pvSystemStore as *const wchar_t;
        let newStore: *mut CStore = CStore__new_0(storeName, (*storeList).StoreTypeFlags());
        (*storeList).m_Stores.AddTail(newStore);
        return (TRUE) as BOOL; // Continue enumeration
    }
    pub unsafe fn __ctor_0(&mut self, r#type: StoreType) {
        let this_: *mut Self = self;
        (*this_).m_Type = r#type;
        {
        }
    }
    pub unsafe fn LoadStores(&mut self) {
        let this_: *mut Self = self;
        let _ = CertEnumSystemStore((*this_).StoreTypeFlags(), core::ptr::null_mut(), this_ as *mut core::ffi::c_void, Some(CStoreList::s_StoreCallback));
    }
    pub unsafe fn StoreTypeFlags(&mut self) -> DWORD {
        let this_: *mut Self = self;
        match (*this_).m_Type {
            User => {
                return CERT_SYSTEM_STORE_CURRENT_USER;
            }
            Service => {
                return CERT_SYSTEM_STORE_SERVICES;
            }
            Computer => {
                return CERT_SYSTEM_STORE_LOCAL_MACHINE;
            }
            _ => {
                return 0;
            }
        }
    }
}
