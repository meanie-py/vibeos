use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// DEFINE_ENUM_FLAG_OPERATORS(UninstallCommandFlags) is a function-like C++ macro that generates bitwise operator overloads. It is skipped here, as the preamble rules require. The repeated invocations are not emitted.






#[no_mangle]
pub unsafe extern "C" fn IsKnownLicenseType(x: i32) -> i32 {
    return if (x) as u32 > LICENSE_NONE && ((x) as u32) <= LICENSE_MAX { 1 } else { 0 };
}
#[no_mangle]
pub unsafe extern "C" fn IsAvailableEnum(x: INT) -> BOOL {
    C_ASSERT!(ENUM_CAT_AUDIO == 1 && ENUM_CAT_THEMES == 15 && ENUM_CAT_OTHER == 16);
    C_ASSERT!(ENUM_LASTCATEGORY >= ENUM_CAT_OTHER);
    C_ASSERT!(ENUM_LASTCATEGORY < ENUM_CAT_SELECTED);
    C_ASSERT!(ENUM_LASTCATEGORY < ENUM_INSTALLED_MIN);
    C_ASSERT!(ENUM_CAT_SELECTED < ENUM_INSTALLED_MIN);

    return ((x) as u32 >= ENUM_AVAILABLE_MIN && ((x) as u32) <= ENUM_AVAILABLE_MAX) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn IsInstalledEnum(x: i32) -> i32 {
    if (x) as u32 >= ENUM_INSTALLED_MIN && ((x) as u32) <= ENUM_INSTALLED_MAX {
        return 1;
    } else {
        return 0;
    }
}

impl CAppInfo {
    pub unsafe fn GetInstallerType__body(&mut self, NestedType: bool,
    ) -> InstallerType {
        let this_: *mut Self = self;
        INSTALLER_UNKNOWN
    }
    pub unsafe fn GetInstallerInfo__body(&mut self, SilentParameters: &mut CStringW) -> InstallerType {
        let this_: *mut Self = self;
        return (*this_).GetInstallerType(false);
    }
}

impl CAvailableApplicationInfo {
    pub unsafe fn GetConfigParser(&mut self) -> *mut CConfigParser {
        let this_: *mut Self = self;
        (*this_).m_Parser
    }
}

impl CInstalledApplicationInfo {
    pub unsafe fn GetRegKey(&mut self) -> &'static mut CRegKey {
        let this_: *mut Self = self;
        return &mut (*this_).m_hKey;
    }
}
