// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_714123A8AF14: [u16; 9] = [0x0025u16, 0x0073, 0x005C, 0x0025, 0x0073, 0x005C, 0x0025, 0x0073, 0x0000];
pub static RUSTCONV_WSTR_75C0EF326CD8: [u16; 29] = [
    0x0046u16, 0x0061, 0x0069, 0x006C, 0x0065, 0x0064, 0x0020, 0x0074, 0x006F, 0x0020,
    0x006F, 0x0070, 0x0065, 0x006E, 0x0020, 0x0072, 0x0065, 0x0067, 0x0020, 0x006B,
    0x0065, 0x0079, 0x003A, 0x0020, 0x0025, 0x0077, 0x0053, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_932C12C2C66C: [u16; 18] = [
    0x0025u16, 0x006C, 0x0073, 0x005C, 0x0025, 0x006C, 0x0073, 0x005C, 0x0025, 0x006C,
    0x0073, 0x005C, 0x0025, 0x006C, 0x0073, 0x0025, 0x0064, 0x0000,
];
pub static RUSTCONV_WSTR_DBB0DCE6A22B: [u16; 22] = [
    0x004Fu16, 0x0070, 0x0065, 0x006E, 0x0069, 0x006E, 0x0067, 0x0020, 0x0072, 0x0065,
    0x0067, 0x0020, 0x006B, 0x0065, 0x0079, 0x003A, 0x0020, 0x0025, 0x0077, 0x0053,
    0x000A, 0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static REG_SERVICES_KEY_NAME_U: [::core::ffi::c_ushort; 34] = unsafe {
    ::core::mem::transmute::<
        [u8; 68],
        [::core::ffi::c_ushort; 34],
    >(
        *b"S\0y\0s\0t\0e\0m\0\\\0C\0u\0r\0r\0e\0n\0t\0C\0o\0n\0t\0r\0o\0l\0S\0e\0t\0\\\0S\0e\0r\0v\0i\0c\0e\0s\0\0\0",
    )
};

pub static REG_PARAMETERS_KEY_NAME_U: [::core::ffi::c_ushort; 11] = unsafe {
    ::core::mem::transmute::<[u8; 22], [::core::ffi::c_ushort; 11]>(
        *b"P\0a\0r\0a\0m\0e\0t\0e\0r\0s\0\0\0",
    )
};

pub static REG_DEVICE_KEY_NAME_U: [::core::ffi::c_ushort; 7] = unsafe {
    ::core::mem::transmute::<[u8; 14], [::core::ffi::c_ushort; 7]>(*b"D\0e\0v\0i\0c\0e\0\0\0")
};
#[no_mangle]
pub unsafe extern "C" fn OpenSoundDriverParametersRegKey(
    ServiceName: LPWSTR,
    KeyHandle: PHKEY,
) -> MMRESULT {
    let mut KeyLength: SIZE_T;
    let mut ParametersKeyName: PWCHAR;

    VALIDATE_MMSYS_PARAMETER!(ServiceName);
    VALIDATE_MMSYS_PARAMETER!(KeyHandle);

    /* Work out how long the string will be */
    KeyLength = (wcslen(REG_SERVICES_KEY_NAME_U.as_ptr() as *const u16) + 1
        + wcslen(ServiceName as *const u16) + 1
        + wcslen((REG_PARAMETERS_KEY_NAME_U).as_ptr())) as _;

    /* Allocate memory for the string */
    ParametersKeyName = AllocateWideString!(KeyLength);

    if ParametersKeyName.is_null() {
        return MMSYSERR_NOMEM;
    }

    /* Construct the registry path */
    wsprintf(
        ParametersKeyName as LPWSTR,
        RUSTCONV_WSTR_714123A8AF14.as_ptr(),
        REG_SERVICES_KEY_NAME_U,
        ServiceName,
        REG_PARAMETERS_KEY_NAME_U,
    );

    SND_TRACE!(RUSTCONV_WSTR_DBB0DCE6A22B.as_ptr(), ParametersKeyName);

    /* Perform the open */
    if RegOpenKeyEx(
        HKEY_LOCAL_MACHINE,
        ParametersKeyName as *const u16,
        0,
        KEY_READ,
        KeyHandle,
    ) != (ERROR_SUCCESS as i32)
    {
        /* Couldn't open the key */
        SND_ERR!(RUSTCONV_WSTR_75C0EF326CD8.as_ptr(), ParametersKeyName);
        FreeMemory(ParametersKeyName as PVOID);
        return MMSYSERR_ERROR;
    }

    FreeMemory(ParametersKeyName as PVOID);

    return MMSYSERR_NOERROR;
}
#[no_mangle]
pub unsafe extern "C" fn OpenSoundDeviceRegKey(
    ServiceName: LPWSTR,
    DeviceIndex: DWORD,
    KeyHandle: PHKEY,
) -> MMRESULT {
    let mut PathLength: SIZE_T;
    let mut RegPath: PWCHAR;

    VALIDATE_MMSYS_PARAMETER!(ServiceName);
    VALIDATE_MMSYS_PARAMETER!(KeyHandle);

    PathLength = (wcslen(REG_SERVICES_KEY_NAME_U.as_ptr()) + 1
        + wcslen(ServiceName) + 1
        + wcslen(REG_PARAMETERS_KEY_NAME_U.as_ptr()) + 1
        + wcslen(REG_DEVICE_KEY_NAME_U.as_ptr())) as SIZE_T
        + GetDigitCount(DeviceIndex) as SIZE_T;

    /* Allocate storage for the string */
    RegPath = AllocateWideString!(PathLength);

    if RegPath.is_null() {
        return MMSYSERR_NOMEM;
    }

    /* Write the path */
    wsprintf(
        RegPath,
        RUSTCONV_WSTR_932C12C2C66C.as_ptr(),
        REG_SERVICES_KEY_NAME_U,
        ServiceName,
        REG_PARAMETERS_KEY_NAME_U,
        REG_DEVICE_KEY_NAME_U,
        DeviceIndex,
    );

    SND_TRACE!(RUSTCONV_WSTR_DBB0DCE6A22B.as_ptr(), RegPath);

    /* Perform the open */
    if RegOpenKeyEx(
        HKEY_LOCAL_MACHINE,
        RegPath,
        0,
        KEY_READ,
        KeyHandle,
    ) != (ERROR_SUCCESS as i32)
    {
        /* Couldn't open the key */
        SND_ERR!(RUSTCONV_WSTR_75C0EF326CD8.as_ptr(), RegPath);
        FreeMemory((RegPath).cast());
        return MMSYSERR_ERROR;
    }

    FreeMemory((RegPath).cast());

    return MMSYSERR_NOERROR;
}
