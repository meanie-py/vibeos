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
            if !(*this_).m_hKey.is_null()
            {
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
        ATLASSERT!(hKeyParent as usize);
        ATLASSERT!(lpszKeyName as usize);
    
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
            ATLASSERT!(hKeyParent);
            ATLASSERT!(lpszKeyName);
    
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
    pub unsafe fn DeleteValue(&mut self, lpszValue: LPCTSTR) -> LONG {
        let this_: *mut Self = self;
        ATLASSERT!((*this_).m_hKey);
        return RegDeleteValue((*this_).m_hKey, lpszValue);
    }
    pub unsafe fn RecurseDeleteKey(&mut self, lpszKey: LPCTSTR) -> LONG {
        let this_: *mut Self = self;
        ATLASSERT!((*this_).m_hKey);
        ATLASSERT!(lpszKey);
        return CRegKey::_DoDeleteKeyTree((*this_).m_hKey, lpszKey);
    }
}
