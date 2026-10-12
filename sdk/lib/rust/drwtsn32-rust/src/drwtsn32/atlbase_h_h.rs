use crate::prelude::*;
use core::ffi::c_void;











impl CRegKey {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).m_hKey = core::ptr::null_mut();
        {
        }
    }
    pub unsafe fn __ctor_1(&mut self, key: &mut CRegKey) {
        let this_: *mut Self = self;
        (*this_).m_hKey = key.Detach();
        {
        }
    }
    pub unsafe fn __ctor_2(&mut self, hKey: HKEY) {
        let this_: *mut Self = self;
        (*this_).m_hKey = hKey;
        {
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        (*this_).Close();
    }
    pub unsafe fn Close(&mut self) -> LONG {
        let this_: *mut Self = self;
        if (*this_).m_hKey as usize != 0 {
            let hKey: HKEY = (*this_).Detach();
            return RegCloseKey(hKey);
        }
        return (ERROR_SUCCESS) as LONG;
    }
    pub unsafe fn Detach(&mut self) -> HKEY {
        let this_: *mut Self = self;
        let hKey = (*this_).m_hKey;
        (*this_).m_hKey = core::ptr::null_mut();
        return hKey;
    }
    pub unsafe fn Open(&mut self, hKeyParent: HKEY, lpszKeyName: LPCTSTR, samDesired: REGSAM) -> LONG {
        let this_: *mut Self = self;
        ATLASSERT!((hKeyParent as usize != 0) as i32);
        ATLASSERT!((lpszKeyName as usize != 0) as i32);
    
        let mut hKey: HKEY = core::ptr::null_mut();
        let mut lRes: LONG = RegOpenKeyEx(hKeyParent, lpszKeyName, 0, samDesired, core::ptr::addr_of_mut!(hKey));
        if lRes == (ERROR_SUCCESS as i32) {
            (*this_).Close();
            (*this_).m_hKey = hKey;
        }
        return lRes;
    }
    pub unsafe fn QueryValue(&mut self, pszValueName: LPCTSTR, pdwType: *mut DWORD, pData: *mut core::ffi::c_void, pnBytes: *mut ULONG) -> LONG {
        let this_: *mut Self = self;
        ATLASSERT!((*this_).m_hKey);
        return RegQueryValueEx((*this_).m_hKey, pszValueName, core::ptr::null_mut(), pdwType, pData as LPBYTE, pnBytes);
    }
    pub unsafe fn QueryDWORDValue(&mut self, pszValueName: LPCTSTR, dwValue: &mut DWORD) -> LONG {
        let this_: *mut Self = self;
        let mut size: ULONG = core::mem::size_of::<DWORD>() as ULONG;
        let mut type_: DWORD = 0;
        let mut lRet: LONG = (*this_).QueryValue(
            pszValueName,
            core::ptr::addr_of_mut!(type_),
            dwValue as *mut DWORD as *mut core::ffi::c_void,
            core::ptr::addr_of_mut!(size),
        );
    
        if lRet == (ERROR_SUCCESS as i32) && type_ != REG_DWORD {
            lRet = (ERROR_INVALID_DATA as i32);
        }
    
        return lRet;
    }
    pub unsafe fn QueryStringValue(&mut self, pszValueName: LPCTSTR, pszValue: LPTSTR, pnChars: *mut ULONG) -> LONG {
        let this_: *mut Self = self;
        let mut size: ULONG = (*pnChars) * ((core::mem::size_of::<TCHAR>()) as ULONG);
        let mut type_: DWORD = (0) as DWORD;
        let mut lRet: LONG = (*this_).QueryValue(pszValueName, core::ptr::addr_of_mut!(type_), pszValue as *mut core::ffi::c_void, core::ptr::addr_of_mut!(size));
    
        if lRet == ((ERROR_SUCCESS) as LONG) && type_ != ((REG_SZ) as DWORD) && type_ != ((REG_EXPAND_SZ) as DWORD) {
            lRet = (ERROR_INVALID_DATA) as LONG;
        }
    
        *pnChars = size / ((core::mem::size_of::<TCHAR>()) as ULONG);
        return lRet;
    }
}
