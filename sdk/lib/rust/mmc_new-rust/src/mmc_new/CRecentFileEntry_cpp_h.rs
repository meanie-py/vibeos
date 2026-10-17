use crate::prelude::*;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_59A543594151: [WCHAR; 4] = [0x002E, 0x002E, 0x002E, 0x0000];

use core::ffi::c_void;




impl CRecentFileEntry {
    pub unsafe fn BuildDisplayName(&mut self) {
        let this_: *mut Self = self;
        let mut SystemDirectory: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
        let mut Length: UINT;

        Length = GetSystemDirectoryW(SystemDirectory.as_mut_ptr(), MAX_PATH);

        if _wcsnicmp((*this_).m_FileName.GetString(), SystemDirectory.as_ptr(), Length as usize) == 0 {
            (*this_).m_DisplayName = (*this_).m_FileName.Mid((Length + 1) as i32);
        } else {
            let mut pBackslash1: PWSTR = core::ptr::null_mut();
            let mut pBackslash2: PWSTR = core::ptr::null_mut();
            let mut pBackslash3: PWSTR = core::ptr::null_mut();

            pBackslash1 = wcschr((*this_).m_FileName.GetString(), 0x005Cu16) as PWSTR;

            if !pBackslash1.is_null() {
                pBackslash2 = wcschr(pBackslash1.add(1 as usize), 0x005Cu16) as PWSTR;
            }

            if !pBackslash2.is_null() {
                pBackslash3 = wcsrchr((*this_).m_FileName.GetString(), 0x005Cu16) as PWSTR;
            }

            if !pBackslash1.is_null() && !pBackslash2.is_null() && !pBackslash3.is_null() && (pBackslash2 != pBackslash3) {
                let prefixLength: i32 = pBackslash2.offset_from((*this_).m_FileName.GetString()) as i32;
                let mut str: CAtlString = (*this_).m_FileName.Left(prefixLength + 1);
                str.Append(CPP_COM_WSTR_59A543594151.as_ptr());
                str.Append(pBackslash3 as *const WCHAR);
                (*this_).m_DisplayName = str;
            } else {
                (*this_).m_DisplayName = (*this_).m_FileName.Mid(0);
            }
        }
    }
    pub unsafe fn __ctor_0(&mut self, FileName: &CAtlString) {
        let this_: *mut Self = self;
        {
            (*this_).m_FileName = FileName.clone();
            (*this_).BuildDisplayName();
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        cpp_delete(core::ptr::addr_of_mut!((*this_).m_FileName));
        cpp_delete(core::ptr::addr_of_mut!((*this_).m_DisplayName));
    }
}
