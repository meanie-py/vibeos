use crate::prelude::*;
use core::ffi::c_void;








impl CStore {
    pub unsafe fn StoreHandle(&mut self) -> HCERTSTORE {
        let this_: *mut Self = self;
        if (*this_).m_hStore.is_null() {
            (*this_).m_hStore = CertOpenStore(
                CERT_STORE_PROV_SYSTEM_W, PKCS_7_ASN_ENCODING | X509_ASN_ENCODING, 0,
                CERT_STORE_SET_LOCALIZED_NAME_FLAG | (*this_).m_dwFlags | CERT_STORE_MAXIMUM_ALLOWED_FLAG, (*this_).m_CryptName.as_ptr() as *const c_void);
        }
        return (*this_).m_hStore;
    }
    pub unsafe fn __ctor_0(&mut self, cryptName: LPCWSTR, dwFlags: DWORD) {
        let this_: *mut Self = self;
        (*this_).m_CryptName = CStringW::from_ptr(cryptName);
        (*this_).m_dwFlags = dwFlags;
        (*this_).m_hStore = core::ptr::null_mut();
        (*this_).m_Expanded = false;
        {
            let localized = CryptFindLocalizedName(cryptName);
            if !localized.is_null() {
                (*this_).m_StoreName = CStringW::from_ptr(localized);
            } else {
                (*this_).m_StoreName = CStringW::from_ptr(cryptName);
            }
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        if !(*this_).m_hStore.is_null() {
            CertCloseStore((*this_).m_hStore, 0);
        }
    }
    pub unsafe fn GetStoreName(&mut self) -> LPCWSTR {
        let this_: *mut Self = self;
        (*this_).m_StoreName.as_ptr()
    }
    pub unsafe fn Expand(&mut self) {
        let this_: *mut Self = self;
            if (*this_).m_Expanded {
                return;
            }
            (*this_).m_Expanded = true;
            // EnumCertificateRevocationLists()
            // EnumCertificateTrustLists()
            (*this_).EnumCertificates();
        }
    pub unsafe fn EnumCertificates(&mut self) {
        let this_: *mut Self = self;
        let handle = (*this_).StoreHandle();
        if handle.is_null() {
            return;
        }
        let mut pContext: PCCERT_CONTEXT = core::ptr::null_mut();
        loop {
            pContext = CertEnumCertificatesInStore(handle, pContext);
            if pContext.is_null() {
                break;
            }
            let newCert: *mut CCert = CCert__new_0(pContext);
            (*this_).m_Certificates.AddTail(newCert);
        }
    }
}
