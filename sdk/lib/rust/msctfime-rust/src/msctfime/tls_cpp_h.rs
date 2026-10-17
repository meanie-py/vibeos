use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// `DWORD TLS::s_dwTlsIndex = (DWORD)-1;` (tls.cpp): "no slot yet". The generator zeroed
// static members, and 0 is a VALID TLS index -- TLS::Uninitialize before a successful
// Initialize would TlsFree someone else's slot. (cpp_app._static_member_init now carries it.)
pub static mut TLS__s_dwTlsIndex: DWORD = u32::MAX;








impl TLS {
    pub unsafe fn Initialize() -> BOOL {
        TLS__s_dwTlsIndex = TlsAlloc();
        return (TLS__s_dwTlsIndex != (0 as DWORD).wrapping_sub(1 as DWORD)) as BOOL;
    }
    pub unsafe fn Uninitialize() {
        if TLS__s_dwTlsIndex != 0u32.wrapping_sub(1) {
            TlsFree(TLS__s_dwTlsIndex);
            TLS__s_dwTlsIndex = 0u32.wrapping_sub(1);
        }
    }
    pub unsafe fn GetTLS() -> *mut TLS {
        if TLS__s_dwTlsIndex == (-(1 as i32)) as DWORD {
            return core::ptr::null_mut();
        }

        return TLS::InternalAllocateTLS();
    }
    pub unsafe fn PeekTLS() -> *mut TLS {
        return TlsGetValue(TLS__s_dwTlsIndex) as *mut TLS;
    }
    pub unsafe fn InternalAllocateTLS() -> *mut TLS {
        let mut pTLS: *mut TLS = TLS::PeekTLS();
        if !pTLS.is_null() {
            return pTLS;
        }

        if DllShutdownInProgress() != 0 {
            return core::ptr::null_mut();
        }

        pTLS = cicMemAllocClear((core::mem::size_of::<TLS>()) as _) as *mut TLS;
        if pTLS.is_null() {
            return core::ptr::null_mut();
        }

        if TlsSetValue(TLS__s_dwTlsIndex, pTLS as *mut core::ffi::c_void) == 0 {
            cicMemFree(pTLS as *mut core::ffi::c_void);
            return core::ptr::null_mut();
        }

        (*pTLS).m_dwFlags1 |= 1;
        (*pTLS).m_dwUnknown2 |= 1;
        pTLS
    }
    pub unsafe fn InternalDestroyTLS() -> BOOL {
        let mut pTLS: *mut TLS = TLS::PeekTLS();
        if pTLS.is_null() {
            return (FALSE) as BOOL;
        }

        if !(*pTLS).m_pBridge.is_null() {
            (*(*pTLS).m_pBridge).Release();
        }
        if !(*pTLS).m_pProfile.is_null() {
            (*(*pTLS).m_pProfile).Release();
        }
        if !(*pTLS).m_pThreadMgr.is_null() {
            (*(*pTLS).m_pThreadMgr).Release();
        }

        cicMemFree((pTLS).cast());
        TlsSetValue(TLS__s_dwTlsIndex, core::ptr::null_mut());
        return (TRUE) as BOOL;
    }
    pub unsafe fn NonEACompositionEnabled(&mut self) -> BOOL {
        let this_: *mut Self = self;
        if (*this_).m_NonEAComposition == 0 {
            let mut dwValue: DWORD = 1;

            let mut regKey: CicRegKey = CicRegKey__zeroed();
            let mut error: LSTATUS = regKey.Open(HKEY_CURRENT_USER, (&[0x0053, 0x004f, 0x0046, 0x0054, 0x0057, 0x0041, 0x0052, 0x0045, 0x005c, 0x004d, 0x0069, 0x0063, 0x0072, 0x006f, 0x0073, 0x006f, 0x0066, 0x0074, 0x005c, 0x0043, 0x0054, 0x0046, 0x005c, 0x0043, 0x0055, 0x0041, 0x0053, 0x0000u16]).as_ptr() /* TEXT "SOFTWARE\\Microsoft\\CTF\\CUAS" */, KEY_READ);
            if (error) as u32 == ERROR_SUCCESS {
                error = regKey.QueryDword((&[0x004e, 0x006f, 0x006e, 0x0045, 0x0041, 0x0043, 0x006f, 0x006d, 0x0070, 0x006f, 0x0073, 0x0069, 0x0074, 0x0069, 0x006f, 0x006e, 0x0000u16]).as_ptr() /* TEXT "NonEAComposition" */, core::ptr::addr_of_mut!(dwValue));
                if (error) as u32 != ERROR_SUCCESS {
                    dwValue = 1;
                }
            }

            (*this_).m_NonEAComposition = dwValue;
        }

        return ((*this_).m_NonEAComposition == 2) as BOOL;
    }
}
