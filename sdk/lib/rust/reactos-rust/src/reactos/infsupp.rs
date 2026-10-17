use crate::reactos::reactos::ConvertNtPathToWin32Path;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut SpInfExports: SPINF_EXPORTS = unsafe {
    _SPINF_EXPORTS {
        SpInfCloseInfFile: Some(SetupCloseInfFile),
        SpInfFindFirstLine: Some(
            SetupFindFirstLineW,
        ),
        SpInfFindNextLine: Some(
            SetupFindNextLine,
        ),
        SpInfGetFieldCount: Some(SetupGetFieldCount),
        SpInfGetBinaryField: Some(
            SetupGetBinaryField,
        ),
        SpInfGetIntField: Some(
            SetupGetIntField,
        ),
        SpInfGetMultiSzField: Some(
            SetupGetMultiSzFieldW,
        ),
        SpInfGetStringField: Some(
            SetupGetStringFieldW,
        ),
        SpInfGetField: Some(pSetupGetField),
        SpInfOpenInfFile: Some(
            SetupOpenInfFileExW,
        ),
    }
};

// #define NDEBUG: flag-only macro with no value, not emitted.
// pSetupGetField prototype: declaration only, defined with its body elsewhere.
#[no_mangle]
pub unsafe extern "system" fn SetupOpenInfFileExW(
    FileName: PCWSTR,
    InfClass: PCWSTR,
    InfStyle: DWORD,
    LocaleId: LCID,
    ErrorLine: PUINT,
) -> HINF {
    let mut InfHandle: HINF;
    let mut OldLocaleId: LCID;
    let mut Win32FileName: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    /*
     * SetupOpenInfFileExW is called within setuplib with NT paths, however
     * the Win32 SetupOpenInfFileW API only takes Win32 paths. We therefore
     * map the NT path to Win32 path and then call the Win32 API.
     */
    if ConvertNtPathToWin32Path(
        core::ptr::addr_of_mut!(SetupData.MappingList),
        core::ptr::addr_of_mut!(Win32FileName) as PWSTR,
        _countof!(Win32FileName),
        FileName,
    ) == 0
    {
        return INVALID_HANDLE_VALUE;
    }

    /*
     * Because SetupAPI's SetupOpenInfFileW() function does not allow the user
     * to specify a given LCID to use to load localized string substitutions,
     * we temporarily change the current thread locale before calling
     * SetupOpenInfFileW(). When we have finished we restore the original
     * thread locale.
     */
    OldLocaleId = GetThreadLocale();
    if OldLocaleId != LocaleId {
        SetThreadLocale(LocaleId);
    }

    /* Load the INF file */
    InfHandle = SetupOpenInfFileW(
        core::ptr::addr_of_mut!(Win32FileName) as *mut WCHAR as PCWSTR,
        InfClass,
        InfStyle,
        ErrorLine,
    );

    /* Restore the original thread locale */
    if OldLocaleId != LocaleId {
        SetThreadLocale(OldLocaleId);
    }

    return InfHandle;
}
