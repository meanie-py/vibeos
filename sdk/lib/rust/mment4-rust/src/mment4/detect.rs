// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_00489567B13A: [u16; 42] = [
    0x0049u16, 0x006E, 0x0074, 0x0065, 0x0072, 0x0065, 0x0073, 0x0074, 0x0065, 0x0064,
    0x0020, 0x0069, 0x006E, 0x0020, 0x0064, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065,
    0x0073, 0x0020, 0x0062, 0x0065, 0x0067, 0x0069, 0x006E, 0x006E, 0x0069, 0x006E,
    0x0067, 0x0020, 0x0077, 0x0069, 0x0074, 0x0068, 0x0020, 0x0025, 0x0077, 0x0053,
    0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_0824B282959F: [u16; 42] = [
    0x0046u16, 0x0061, 0x0069, 0x006C, 0x0065, 0x0064, 0x0020, 0x0074, 0x006F, 0x0020,
    0x0071, 0x0075, 0x0065, 0x0072, 0x0079, 0x0020, 0x0072, 0x0065, 0x0067, 0x0069,
    0x0073, 0x0074, 0x0072, 0x0079, 0x0020, 0x006B, 0x0065, 0x0079, 0x0020, 0x0069,
    0x006E, 0x0066, 0x006F, 0x0072, 0x006D, 0x0061, 0x0074, 0x0069, 0x006F, 0x006E,
    0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_189877A0FAAE: [u16; 5] = [0x005Cu16, 0x005C, 0x002E, 0x005C, 0x0000];
pub static RUSTCONV_WSTR_2810CD562234: [u16; 19] = [
    0x0046u16, 0x006F, 0x0075, 0x006E, 0x0064, 0x0020, 0x0064, 0x0065, 0x0076, 0x0069,
    0x0063, 0x0065, 0x003A, 0x0020, 0x0025, 0x0077, 0x0053, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_5694F9FB6CA5: [u16; 6] = [0x0025u16, 0x006C, 0x0073, 0x0025, 0x0064, 0x0000];
pub static RUSTCONV_WSTR_9600E09AB8B3: [u16; 33] = [
    0x0055u16, 0x006E, 0x0061, 0x0062, 0x006C, 0x0065, 0x0020, 0x0074, 0x006F, 0x0020,
    0x006F, 0x0070, 0x0065, 0x006E, 0x0020, 0x0074, 0x0068, 0x0065, 0x0020, 0x0044,
    0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x0073, 0x0020, 0x006B, 0x0065, 0x0079,
    0x0021, 0x000A, 0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const REG_DEVICES_KEY_NAME_U: [::core::ffi::c_ushort; 8] = unsafe {
    ::core::mem::transmute::<[u8; 16], [::core::ffi::c_ushort; 8]>(*b"D\0e\0v\0i\0c\0e\0s\0\0\0")
};
#[no_mangle]
pub unsafe extern "C" fn EnumerateNt4ServiceSoundDevices(
    ServiceName: LPWSTR,
    DeviceType: MMDEVICE_TYPE,
    SoundDeviceDetectedProc: SOUND_DEVICE_DETECTED_PROC,
) -> MMRESULT {
    let mut Key: HKEY = core::ptr::null_mut();
    let mut KeyIndex: DWORD = 0;

    VALIDATE_MMSYS_PARAMETER!(ServiceName);

    /* Device type zero means "all" */
    VALIDATE_MMSYS_PARAMETER!(
        (IsValidSoundDeviceType!(DeviceType) != 0) || DeviceType == 0
    );

    while OpenSoundDeviceRegKey(ServiceName, KeyIndex, core::ptr::addr_of_mut!(Key))
        == MMSYSERR_NOERROR
    {
        let mut DevicesKey: HKEY = core::ptr::null_mut();
        let mut ValueType: DWORD = REG_NONE;
        let mut ValueIndex: DWORD = 0;
        let mut MaxNameLength: DWORD = 0;
        let mut ValueNameLength: DWORD = 0;
        let mut DevicePath: PWSTR = core::ptr::null_mut();
        let mut ValueName: PWSTR = core::ptr::null_mut();
        let mut ValueDataLength: DWORD = core::mem::size_of::<DWORD>() as DWORD;
        let mut ValueData: DWORD = core::mem::zeroed();

        if RegOpenKeyEx(
            Key,
            (REG_DEVICES_KEY_NAME_U).as_ptr(),
            0,
            KEY_READ,
            core::ptr::addr_of_mut!(DevicesKey),
        ) == (ERROR_SUCCESS as i32)
        {
            /* Find out how much memory is needed for the key name */
            if RegQueryInfoKey(
                DevicesKey,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(MaxNameLength),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            ) != (ERROR_SUCCESS as i32)
            {
                SND_ERR!(RUSTCONV_WSTR_0824B282959F.as_ptr());
                RegCloseKey(DevicesKey);
                RegCloseKey(Key);

                return MMSYSERR_ERROR;
            }

            DevicePath = AllocateWideString!(
                MaxNameLength as usize
                    + strlen(b"\\\\.\\\0".as_ptr() as *const i8),
            );

            /* Check that the memory allocation was successful */
            if DevicePath.is_null() {
                /* There's no point in going further */
                RegCloseKey(DevicesKey);
                RegCloseKey(Key);

                return MMSYSERR_NOMEM;
            }

            /* Insert the device path prefix */
            wsprintf(DevicePath, RUSTCONV_WSTR_189877A0FAAE.as_ptr());

            /* The offset of the string following this prefix */
            ValueName = DevicePath.add(
                strlen(b"\\\\.\\\0".as_ptr() as *const i8),
            );

            /* Copy this so that it may be overwritten - include NULL */
            ValueNameLength = MaxNameLength + core::mem::size_of::<WCHAR>() as DWORD;

            SND_TRACE!(RUSTCONV_WSTR_00489567B13A.as_ptr(), DevicePath);

            while RegEnumValue(
                DevicesKey,
                ValueIndex,
                ValueName,
                core::ptr::addr_of_mut!(ValueNameLength),
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(ValueType),
                core::ptr::addr_of_mut!(ValueData) as LPBYTE,
                core::ptr::addr_of_mut!(ValueDataLength),
            ) == (ERROR_SUCCESS as i32)
            {
                /* Device types are stored as DWORDs */
                if (ValueType == REG_DWORD)
                    && (ValueDataLength == core::mem::size_of::<DWORD>() as DWORD)
                {
                    if (DeviceType == 0) || ((DeviceType) as u32 == ValueData) {
                        SND_TRACE!(RUSTCONV_WSTR_2810CD562234.as_ptr(), DevicePath);
                        (SoundDeviceDetectedProc.unwrap_unchecked())((ValueData) as u8, DevicePath);
                    }
                }

                /* Reset variables for the next iteration */
                ValueNameLength = MaxNameLength + core::mem::size_of::<WCHAR>() as DWORD;
                ZeroMemory!(
                    ValueName as *mut core::ffi::c_void,
                    (MaxNameLength + 1) as usize
                        * core::mem::size_of::<WCHAR>(),
                );
                /*ZeroWideString(ValueName);*/
                ValueDataLength = core::mem::size_of::<DWORD>() as DWORD;
                ValueData = 0;
                ValueType = REG_NONE;

                ValueIndex = ValueIndex.wrapping_add(1);
            }

            FreeMemory((DevicePath).cast());

            RegCloseKey(DevicesKey);
        } else {
            SND_WARN!(RUSTCONV_WSTR_9600E09AB8B3.as_ptr());
        }

        KeyIndex = KeyIndex.wrapping_add(1);

        RegCloseKey(Key);
    }

    return MMSYSERR_NOERROR;
}
#[no_mangle]
pub unsafe extern "C" fn DetectNt4SoundDevices(
    DeviceType: MMDEVICE_TYPE,
    BaseDeviceName: PWSTR,
    SoundDeviceDetectedProc: SOUND_DEVICE_DETECTED_PROC,
) -> MMRESULT {
    let mut DeviceNameLength: SIZE_T = 0;
    let mut DeviceName: PWSTR = core::ptr::null_mut();
    let mut Index: ULONG = 0;
    let mut DeviceHandle: HANDLE = core::ptr::null_mut(); // out-parameter of OpenKernelSoundDeviceByName
    let mut DoSearch: BOOLEAN = (TRUE as u8);

    VALIDATE_MMSYS_PARAMETER!(IsValidSoundDeviceType!(DeviceType));

    DeviceNameLength = wcslen(BaseDeviceName) as SIZE_T;
    /* Consider the length of the number */
    DeviceNameLength += GetDigitCount(Index) as SIZE_T;

    DeviceName = AllocateWideString!(DeviceNameLength);

    if DeviceName.is_null() {
        return MMSYSERR_NOMEM;
    }

    while DoSearch != 0 {
        /* Nothing like a nice clean device name */
        ZeroWideString!(DeviceName);
        wsprintf(DeviceName, RUSTCONV_WSTR_5694F9FB6CA5.as_ptr(), BaseDeviceName, Index);

        if OpenKernelSoundDeviceByName(
            DeviceName,
            (TRUE) as BOOLEAN,
            core::ptr::addr_of_mut!(DeviceHandle),
        ) == MMSYSERR_NOERROR
        {
            /* Notify the callback function */
            SND_TRACE!(RUSTCONV_WSTR_2810CD562234.as_ptr(), DeviceName);
            (SoundDeviceDetectedProc.unwrap_unchecked())(DeviceType, DeviceName);

            CloseHandle(DeviceHandle);

            Index += 1;
        } else {
            DoSearch = (FALSE as u8);
        }
    }

    FreeMemory((DeviceName).cast());

    return MMSYSERR_NOERROR;
}
