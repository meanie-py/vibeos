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
    pub unsafe fn Attach(&mut self, hKey: HKEY) {
        let this_: *mut Self = self;
        (*this_).m_hKey = hKey;
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
        let hKey: HKEY = (*this_).m_hKey;
        (*this_).m_hKey = core::ptr::null_mut();
        return hKey;
    }
    pub unsafe fn Open(&mut self, hKeyParent: HKEY, lpszKeyName: LPCTSTR, samDesired: REGSAM) -> LONG {
        let this_: *mut Self = self;
            ATLASSERT!(hKeyParent);
            ATLASSERT!(lpszKeyName);
    
            let mut hKey: HKEY = core::ptr::null_mut();
            let mut lRes: LONG = RegOpenKeyEx(hKeyParent, lpszKeyName, 0, samDesired, core::ptr::addr_of_mut!(hKey));
            if (lRes) as u32 == ERROR_SUCCESS {
                (*this_).Close();
                (*this_).m_hKey = hKey;
            }
            return lRes;
        }
    pub unsafe fn Create(&mut self, hKeyParent: HKEY, lpszKeyName: LPCTSTR, lpszClass: LPTSTR, dwOptions: DWORD, samDesired: REGSAM, lpSecAttr: LPSECURITY_ATTRIBUTES, lpdwDisposition: LPDWORD) -> LONG {
        let this_: *mut Self = self;
            ATLASSERT!(hKeyParent as usize != 0);
            ATLASSERT!(lpszKeyName as usize != 0);
    
            let mut hKey: HKEY = core::ptr::null_mut();
            let mut lRes: LONG = RegCreateKeyEx(hKeyParent, lpszKeyName, 0, lpszClass,
                                         dwOptions, samDesired, lpSecAttr, core::ptr::addr_of_mut!(hKey),
                                         lpdwDisposition);
            if (lRes) as u32 == ERROR_SUCCESS
            {
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
    
        if (lRet) as u32 == ERROR_SUCCESS && type_ != REG_DWORD {
            lRet = (ERROR_INVALID_DATA) as i32;
        }
    
        return lRet;
    }
    pub unsafe fn QueryBinaryValue(&mut self, pszValueName: LPCTSTR, pValue: *mut core::ffi::c_void, pnBytes: *mut ULONG) -> LONG {
        let this_: *mut Self = self;
        let mut type_: DWORD = 0;
        let mut lRet: LONG = (*this_).QueryValue(pszValueName, core::ptr::addr_of_mut!(type_), pValue, pnBytes);
    
        if (lRet) as u32 == ERROR_SUCCESS && type_ != REG_BINARY {
            lRet = (ERROR_INVALID_DATA) as i32;
        }
    
        return lRet;
    }
    pub unsafe fn QueryStringValue(&mut self, pszValueName: LPCTSTR, pszValue: LPTSTR, pnChars: *mut ULONG) -> LONG {
        let this_: *mut Self = self;
        let mut size: ULONG = (*pnChars) * core::mem::size_of::<TCHAR>() as ULONG;
        let mut r#type: DWORD = 0;
        let mut lRet: LONG = (*this_).QueryValue(pszValueName, core::ptr::addr_of_mut!(r#type), pszValue as *mut core::ffi::c_void, core::ptr::addr_of_mut!(size));
    
        if (lRet) as u32 == ERROR_SUCCESS && r#type != REG_SZ && r#type != REG_EXPAND_SZ {
            lRet = (ERROR_INVALID_DATA) as i32;
        }
    
        *pnChars = size / core::mem::size_of::<TCHAR>() as ULONG;
        return lRet;
    }
    pub unsafe fn SetValue__0(&mut self, pszValueName: LPCTSTR, dwType: DWORD, pValue: *const core::ffi::c_void, nBytes: ULONG) -> LONG {
        let this_: *mut Self = self;
        ATLASSERT!((*this_).m_hKey);
        return RegSetValueEx((*this_).m_hKey, pszValueName, 0, dwType, pValue as *const u8, nBytes);
    }
    pub unsafe fn SetDWORDValue(&mut self, pszValueName: LPCTSTR, dwValue: DWORD) -> LONG {
        let this_: *mut Self = self;
        return (*this_).SetValue__0(pszValueName, REG_DWORD, core::ptr::addr_of!(dwValue) as *const core::ffi::c_void, core::mem::size_of::<DWORD>() as ULONG);
    }
    pub unsafe fn SetStringValue(&mut self, pszValueName: LPCTSTR, pszValue: LPCTSTR, dwType: DWORD) -> LONG {
        let this_: *mut Self = self;
        let mut length: SIZE_T = 0;
        match dwType {
            REG_SZ | REG_EXPAND_SZ => {
                length = ((_tcslen(pszValue) + 1) * ((core::mem::size_of::<TCHAR>() as SIZE_T)) as _) as _;
                return (*this_).SetValue__0(pszValueName, dwType, pszValue as *const core::ffi::c_void, length as ULONG);
            }
            REG_MULTI_SZ => {
                return (*this_).SetMultiStringValue(pszValueName, pszValue);
            }
            _ => {
                return ERROR_INVALID_DATA as LONG;
            }
        }
    }
    pub unsafe fn SetBinaryValue(&mut self, pszValueName: LPCTSTR, pValue: *const core::ffi::c_void, nBytes: ULONG) -> LONG {
        let this_: *mut Self = self;
        return (*this_).SetValue__0(pszValueName, REG_BINARY, pValue, nBytes);
    }
    pub unsafe fn SetMultiStringValue(&mut self, pszValueName: LPCTSTR, pszValue: LPCTSTR) -> LONG {
        let this_: *mut Self = self;
        let dwSize: ULONG = CRegKey::_GetMultiStringSize(pszValue);
        return (*this_).SetValue__0(pszValueName, REG_MULTI_SZ, pszValue as *const core::ffi::c_void, dwSize);
    }
    pub unsafe fn op_to_HKEY(&mut self) -> HKEY {
        let this_: *mut Self = self;
        return (*this_).m_hKey;
    }
}
