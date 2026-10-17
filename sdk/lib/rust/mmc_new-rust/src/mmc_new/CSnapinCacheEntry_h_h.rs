use crate::prelude::*;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_099909F467C7: [WCHAR; 6] = [0x0041, 0x0062, 0x006F, 0x0075, 0x0074, 0x0000];
pub static CPP_COM_WSTR_0C8C69F592A1: [WCHAR; 19] = [0x004E, 0x0061, 0x006D, 0x0065, 0x0053, 0x0074, 0x0072, 0x0069, 0x006E, 0x0067, 0x0049, 0x006E, 0x0064, 0x0069, 0x0072, 0x0065, 0x0063, 0x0074, 0x0000];
pub static CPP_COM_WSTR_8AD1F2B698A5: [WCHAR; 11] = [0x004E, 0x0061, 0x006D, 0x0065, 0x0053, 0x0074, 0x0072, 0x0069, 0x006E, 0x0067, 0x0000];
pub static CPP_COM_WSTR_F6E5A95477AE: [WCHAR; 11] = [0x0053, 0x0074, 0x0061, 0x006E, 0x0064, 0x0041, 0x006C, 0x006F, 0x006E, 0x0065, 0x0000];

use core::ffi::c_void;















impl CSnapinCacheEntry {
    pub unsafe fn GuidString(&mut self) -> &CAtlString {
        let this_: *mut Self = self;
        return &(*this_).m_GuidString;
    }
    pub unsafe fn AboutGuidString(&mut self) -> &CAtlString {
        let this_: *mut Self = self;
        return &(*this_).m_AboutGuidString;
    }
    pub unsafe fn Name(&mut self) -> &CAtlString {
        let this_: *mut Self = self;
        return &(*this_).m_Name;
    }
    pub unsafe fn Provider(&mut self) -> &CAtlString {
        let this_: *mut Self = self;
        return &(*this_).m_Provider;
    }
    pub unsafe fn Description(&mut self) -> &CAtlString {
        let this_: *mut Self = self;
        return &(*this_).m_Description;
    }
    pub unsafe fn Version(&mut self) -> &CAtlString {
        let this_: *mut Self = self;
        return &(*this_).m_Version;
    }
    pub unsafe fn Icon(&mut self) -> HICON {
        let this_: *mut Self = self;
        (*this_).m_hIcon
    }
    pub unsafe fn NormalImageIndex(&mut self) -> i32 {
        let this_: *mut Self = self;
        (*this_).m_ImageNormal
    }
    pub unsafe fn OpenImageIndex(&mut self) -> i32 {
        let this_: *mut Self = self;
        (*this_).m_ImageOpen
    }
    pub unsafe fn Create(SnapinsKey: &mut CRegKey, guidString: PWSTR, hImageList: HIMAGELIST) -> *mut CSnapinCacheEntry {
            DPRINT1!("Create(%S)\n", guidString);
            let mut IndirectBuffer = [0u16; (MAX_PATH) as _];
            let mut Buffer = [0u16; (MAX_PATH) as _];
            let mut dwSize: ULONG;
            let mut ret: LONG;

            let mut SnapinKey = CRegKey__make_0();
            ret = SnapinKey.Open(SnapinsKey.op_to_HKEY(), guidString, KEY_READ);
            if ret != (ERROR_SUCCESS as i32)
            {
                return core::ptr::null_mut();
            }

            let mut NodeTypesKey = CRegKey__make_0();
            ret = NodeTypesKey.Open(SnapinKey.op_to_HKEY(), CPP_COM_WSTR_F6E5A95477AE.as_ptr(), KEY_READ);
            if ret != (ERROR_SUCCESS as i32)
            {
                SnapinKey.Close();
                return core::ptr::null_mut();
            }
            NodeTypesKey.Close();

            let mut Name = CStringW::new();
            dwSize = (MAX_PATH as ULONG) * (core::mem::size_of::<WCHAR>() as ULONG);
            if SnapinKey.QueryStringValue(CPP_COM_WSTR_0C8C69F592A1.as_ptr(), IndirectBuffer.as_mut_ptr(), core::ptr::addr_of_mut!(dwSize)) == (ERROR_SUCCESS as i32)
            {
                if (SHLoadIndirectString(IndirectBuffer.as_ptr(), Buffer.as_mut_ptr(), MAX_PATH, core::ptr::null_mut())) >= 0
                {
                    Name = CStringW::from_ptr(Buffer.as_ptr());
                }
                else
                {
                    Name.Empty();
                }
            }

            if Name.IsEmpty()
            {
                dwSize = (MAX_PATH as ULONG) * (core::mem::size_of::<WCHAR>() as ULONG);
                if SnapinKey.QueryStringValue(CPP_COM_WSTR_8AD1F2B698A5.as_ptr(), Buffer.as_mut_ptr(), core::ptr::addr_of_mut!(dwSize)) == (ERROR_SUCCESS as i32)
                {
                    Name = CStringW::from_ptr(Buffer.as_ptr());
                }
            }

            if Name.IsEmpty()
            {
                SnapinKey.Close();
                return core::ptr::null_mut();
            }

            let mut AboutGuidString = CStringW::new();
            dwSize = (MAX_PATH as ULONG) * (core::mem::size_of::<WCHAR>() as ULONG);
            if SnapinKey.QueryStringValue(CPP_COM_WSTR_099909F467C7.as_ptr(), Buffer.as_mut_ptr(), core::ptr::addr_of_mut!(dwSize)) == (ERROR_SUCCESS as i32)
            {
                AboutGuidString = CStringW::from_ptr(Buffer.as_ptr());
            }

            SnapinKey.Close();

            if AboutGuidString.IsEmpty()
            {
                return core::ptr::null_mut();
            }

            let mut GuidString = CStringW::from_ptr(guidString);

            return CSnapinCacheEntry__new_0(&GuidString, &AboutGuidString, &Name, hImageList);
        }
}
