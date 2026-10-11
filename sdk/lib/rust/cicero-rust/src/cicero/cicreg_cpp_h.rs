use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn _cicRegKey_Open(this_: *mut CicRegKey, hKey: HKEY, lpSubKey: LPCTSTR, samDesired: REGSAM) -> LSTATUS {
    let mut hNewKey: HKEY = core::mem::zeroed();
    let mut error: LSTATUS = RegOpenKeyEx(hKey, lpSubKey, 0, samDesired, core::ptr::addr_of_mut!(hNewKey));
    if (error) as u32 != ERROR_SUCCESS {
        return error;
    }

    (*this_).Close();
    (*this_).m_hKey = hNewKey;
    return error;
}
#[no_mangle]
pub unsafe extern "C" fn _cicRegKey_Create(this_: *mut CicRegKey, hKey: HKEY, lpSubKey: LPCTSTR) -> LSTATUS {
    let mut hNewKey: HKEY = core::mem::zeroed();
    let mut error: LSTATUS = RegCreateKeyEx(hKey, lpSubKey, 0, core::ptr::null_mut(), REG_OPTION_NON_VOLATILE,
                                     KEY_ALL_ACCESS, core::ptr::null_mut(), core::ptr::addr_of_mut!(hNewKey), core::ptr::null_mut());
    if (error) as u32 != ERROR_SUCCESS {
        return error;
    }

    (*this_).Close();
    (*this_).m_hKey = hNewKey;
    return error;
}
#[no_mangle]
pub unsafe extern "C" fn _cicRegKey_EnumValue(this_: *mut CicRegKey, dwIndex: DWORD, lpValueName: LPTSTR, mut cchValueName: DWORD) -> LSTATUS {
    let mut dwSaveLen: DWORD = cchValueName;
    let mut error: LSTATUS = RegEnumValue((*this_).m_hKey, dwIndex, lpValueName, core::ptr::addr_of_mut!(cchValueName),
                                   core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut());
    if dwSaveLen != 0 {
        *lpValueName.add((if (error) as u32 == ERROR_SUCCESS { dwSaveLen - 1 } else { 0 }) as usize) = 0;
    }
    return error;
}
#[no_mangle]
pub unsafe extern "C" fn _cicRegKey_QuerySz(self_: &mut CicRegKey, pszValueName: LPCTSTR, pszValue: LPTSTR, cchValueMax: DWORD) -> LSTATUS {
    let mut cbValueMax: DWORD = cchValueMax * (core::mem::size_of::<TCHAR>() as DWORD);
    let mut error: LSTATUS = RegQueryValueEx(self_.m_hKey, pszValueName, core::ptr::null_mut(), core::ptr::null_mut(), pszValue as LPBYTE, core::ptr::addr_of_mut!(cbValueMax));
    if cchValueMax > 0 {
        *pszValue.offset(((if (error) as u32 == ERROR_SUCCESS { cchValueMax - 1 } else { 0 }) as isize)) = UNICODE_NULL;
    }
    return error;
}
#[no_mangle]
pub unsafe extern "C" fn _cicRegKey_RecurseDeleteKey(this_: *mut CicRegKey, lpSubKey: LPCTSTR) -> LSTATUS {
    let mut regKey: CicRegKey = CicRegKey__make_0();
    let mut error: LSTATUS = regKey.Open((*this_).m_hKey, lpSubKey, KEY_READ | KEY_WRITE);
    if (error) as u32 != ERROR_SUCCESS {
        return error;
    }

    let mut szName: [TCHAR; (MAX_PATH) as _] = [0 as TCHAR; (MAX_PATH) as _];
    let mut cchName: DWORD = 0;
    loop {
        cchName = _countof!(szName);
        error = RegEnumKeyEx((regKey).op_to_HKEY(), 0, szName.as_mut_ptr(), core::ptr::addr_of_mut!(cchName), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut());
        if (error) as u32 != ERROR_SUCCESS {
            break;
        }

        szName[(_countof!(szName) - 1) as usize] = UNICODE_NULL;
        error = regKey.RecurseDeleteKey(szName.as_ptr());
        if !((error) as u32 == ERROR_SUCCESS) {
            break;
        }
    }

    regKey.Close();

    return (*this_).DeleteSubKey(lpSubKey);
}
