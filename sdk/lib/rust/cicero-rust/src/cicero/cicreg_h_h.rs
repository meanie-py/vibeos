use crate::prelude::*;
use core::ffi::c_void;
















impl CicRegKey {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).m_hKey = core::ptr::null_mut();
        {}
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        (*this_).Close();
    }
    pub unsafe fn op_to_HKEY(&mut self) -> HKEY {
        let this_: *mut Self = self; return (*this_).m_hKey; }
    pub unsafe fn Open(&mut self, hKey: HKEY, lpSubKey: LPCTSTR, samDesired: REGSAM) -> LSTATUS {
        let this_: *mut Self = self;
        return _cicRegKey_Open(this_, hKey, lpSubKey, samDesired);
    }
    pub unsafe fn Create(&mut self, hKey: HKEY, lpSubKey: LPCTSTR) -> LSTATUS {
        let this_: *mut Self = self;
        return _cicRegKey_Create(this_, hKey, lpSubKey);
    }
    pub unsafe fn Close(&mut self) {
        let this_: *mut Self = self;
        if (*this_).m_hKey.is_null() {
            return;
        }
    
        RegCloseKey((*this_).m_hKey);
        (*this_).m_hKey = core::ptr::null_mut();
    }
    pub unsafe fn QueryDword(&mut self, pszValueName: LPCTSTR, pdwValue: LPDWORD) -> LSTATUS {
        let this_: *mut Self = self;
        let mut cbData: DWORD = core::mem::size_of::<DWORD>() as DWORD;
        return RegQueryValueEx((*this_).m_hKey, pszValueName, core::ptr::null_mut(), core::ptr::null_mut(), pdwValue as LPBYTE, core::ptr::addr_of_mut!(cbData));
    }
    pub unsafe fn SetDword(&mut self, pszValueName: LPCTSTR, mut dwValue: DWORD) -> LSTATUS {
        let this_: *mut Self = self;
        return RegSetValueEx((*this_).m_hKey, pszValueName, 0, REG_DWORD, core::ptr::addr_of_mut!(dwValue) as LPBYTE, core::mem::size_of::<DWORD>() as DWORD);
    }
    pub unsafe fn QuerySz(&mut self, pszValueName: LPCTSTR, pszValue: LPTSTR, cchValueMax: DWORD) -> LSTATUS {
        let this_: *mut Self = self;
        _cicRegKey_QuerySz(&mut *this_, pszValueName, pszValue, cchValueMax)
    }
    pub unsafe fn SetSz(&mut self, pszValueName: LPCTSTR, pszValue: LPCTSTR) -> LSTATUS {
        let this_: *mut Self = self;
        let cbValue: DWORD = ((lstrlen(pszValue) as DWORD) + 1) * (core::mem::size_of::<TCHAR>() as DWORD);
        return RegSetValueEx((*this_).m_hKey, pszValueName, 0, REG_SZ, pszValue as LPBYTE, cbValue);
    }
    pub unsafe fn SetSzW(&mut self, pszValueName: LPCWSTR, pszValue: LPCWSTR) -> LSTATUS {
        let this_: *mut Self = self;
        let cbValue: DWORD = ((lstrlenW(pszValue) as DWORD) + 1) * (core::mem::size_of::<WCHAR>() as DWORD);
        return RegSetValueExW((*this_).m_hKey, pszValueName, 0, REG_SZ, pszValue as LPBYTE, cbValue);
    }
    pub unsafe fn DeleteValue(&mut self, pszValueName: LPCTSTR) -> LSTATUS {
        let this_: *mut Self = self;
        return RegDeleteValueW((*this_).m_hKey, pszValueName);
    }
    pub unsafe fn DeleteSubKey(&mut self, lpSubKey: LPCTSTR) -> LSTATUS {
        let this_: *mut Self = self;
        return RegDeleteKey((*this_).m_hKey, lpSubKey);
    }
    pub unsafe fn RecurseDeleteKey(&mut self, lpSubKey: LPCTSTR) -> LSTATUS {
        let this_: *mut Self = self;
        return _cicRegKey_RecurseDeleteKey(this_, lpSubKey);
    }
    pub unsafe fn EnumValue(&mut self, dwIndex: DWORD, lpValueName: LPTSTR, cchValueName: DWORD) -> LSTATUS {
        let this_: *mut Self = self;
        return _cicRegKey_EnumValue(this_, dwIndex, lpValueName, cchValueName);
    }
}
