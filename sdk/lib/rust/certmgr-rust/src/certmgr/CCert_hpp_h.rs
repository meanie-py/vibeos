use crate::prelude::*;
use core::ffi::c_void;











impl CCert {
    pub unsafe fn GetNameString(&mut self, dwFlags: DWORD, out: &mut CAtlStringW) {
        let this_: *mut Self = self;
        let length: DWORD = CertGetNameStringW((*this_).m_CertContext, CERT_NAME_SIMPLE_DISPLAY_TYPE, dwFlags, core::ptr::null_mut(), core::ptr::null_mut(), 0);
        if length > 1 {
            let buffer: *mut WCHAR = out.GetBuffer(length as i32);
            CertGetNameStringW((*this_).m_CertContext, CERT_NAME_SIMPLE_DISPLAY_TYPE, dwFlags, core::ptr::null_mut(), buffer, length);
            out.ReleaseBuffer();
        }
    }
    pub unsafe fn __ctor_0(&mut self, certContext: PCCERT_CONTEXT) {
        let this_: *mut Self = self;
        (*this_).m_CertContext = core::ptr::null_mut();
        {
            (*this_).m_CertContext = CertDuplicateCertificateContext(certContext);
            (*this_).Init();
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        if !(*this_).m_CertContext.is_null() {
            CertFreeCertificateContext((*this_).m_CertContext);
        }
    }
    pub unsafe fn GetSubjectName(&mut self) -> *const CAtlStringW {
        let this_: *mut Self = self;
        return &(*this_).m_SubjectName as *const CAtlStringW;
    }
    pub unsafe fn GetIssuerName(&mut self) -> *const CAtlStringW {
        let this_: *mut Self = self;
        return &(*this_).m_IssuerName as *const CAtlStringW;
    }
    pub unsafe fn GetNotAfter(&mut self) -> FILETIME {
        let this_: *mut Self = self;
        return if (!(*this_).m_CertContext.is_null() && !(*(*this_).m_CertContext).pCertInfo.is_null()) { (*(*(*this_).m_CertContext).pCertInfo).NotAfter } else { core::mem::zeroed::<FILETIME>() };
    }
    pub unsafe fn GetFriendlyName(&mut self) -> *const CAtlStringW {
        let this_: *mut Self = self;
        return &(*this_).m_FriendlyName as *const CAtlStringW;
    }
    pub unsafe fn GetCertContext(&mut self) -> PCCERT_CONTEXT {
        let this_: *mut Self = self;
        (*this_).m_CertContext
    }
    pub unsafe fn Init(&mut self) {
        let this_: *mut Self = self;
        let s: *mut CAtlStringW = core::ptr::addr_of_mut!((*this_).m_SubjectName);
        (*this_).GetNameString(0, &mut *s);
        let i: *mut CAtlStringW = core::ptr::addr_of_mut!((*this_).m_IssuerName);
        (*this_).GetNameString(CERT_NAME_ISSUER_FLAG, &mut *i);
        let mut length: DWORD = 0;
        if CertGetCertificateContextProperty(
            (*this_).m_CertContext,
            CERT_FRIENDLY_NAME_PROP_ID,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(length),
        ) != 0
            && length > 0
        {
            let buffer = (*this_)
                .m_FriendlyName
                .GetBuffer((length / core::mem::size_of::<WCHAR>() as DWORD) as i32);
            CertGetCertificateContextProperty(
                (*this_).m_CertContext,
                CERT_FRIENDLY_NAME_PROP_ID,
                buffer as *mut core::ffi::c_void,
                core::ptr::addr_of_mut!(length),
            );
            (*this_).m_FriendlyName.ReleaseBuffer();
        }
    }
}
