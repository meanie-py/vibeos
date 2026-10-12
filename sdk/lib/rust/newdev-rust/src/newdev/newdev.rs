// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_068BF9CAD306: [u16; 5] = [0x002Eu16, 0x0069, 0x006E, 0x0066, 0x0000];
pub static RUSTCONV_WSTR_77ECA80E282A: [u16; 3] = [0x002Eu16, 0x002E, 0x0000];
pub static RUSTCONV_WSTR_96A296D224F2: [u16; 1] = [0x0000u16];
pub static RUSTCONV_WSTR_B7FC02907B1F: [u16; 2] = [0x005Cu16, 0x0000];
pub static RUSTCONV_WSTR_C245AFF91259: [u16; 2] = [0x002Eu16, 0x0000];
pub static RUSTCONV_WSTR_E8A4B2EE7EDE: [u16; 2] = [0x002Au16, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut hDllInstance: HINSTANCE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

// `static BOOL SearchDriver(...)` is a prototype only; its definition is emitted with the function bodies.

#[no_mangle]
pub unsafe extern "system" fn UpdateDriverForPlugAndPlayDevicesW(
    hwndParent: HWND,
    HardwareId: LPCWSTR,
    FullInfPath: LPCWSTR,
    InstallFlags: DWORD,
    bRebootRequired: PBOOL,
) -> BOOL {
    let mut DevInstData: DEVINSTDATA = core::mem::zeroed();
    let mut InstallParams: SP_DEVINSTALL_PARAMS = core::mem::zeroed();
    let mut i: DWORD = 0;
    let mut Buffer: LPWSTR = core::ptr::null_mut();
    let mut BufferSize: DWORD = 0;
    let mut CurrentHardwareId: LPCWSTR = core::ptr::null();
    let mut Property: DWORD = 0;
    let mut FoundHardwareId: BOOL = 0;
    let mut FoundAtLeastOneDevice: BOOL = (FALSE as i32);
    let mut RebootRequired: BOOL = (FALSE as i32);
    let mut ret: BOOL = (FALSE as i32);

    DevInstData.hDevInfo = INVALID_HANDLE_VALUE;

    TRACE!(
        "UpdateDriverForPlugAndPlayDevicesW(%p %s %s 0x%x %p)\n",
        hwndParent,
        debugstr_w(HardwareId),
        debugstr_w(FullInfPath),
        InstallFlags,
        bRebootRequired
    );

    'cleanup: {
        /* Check flags */
        if (InstallFlags & !(INSTALLFLAG_FORCE | INSTALLFLAG_READONLY | INSTALLFLAG_NONINTERACTIVE)) != 0 {
            TRACE!(
                "Unknown flags: 0x%08lx\n",
                InstallFlags & !(INSTALLFLAG_FORCE | INSTALLFLAG_READONLY | INSTALLFLAG_NONINTERACTIVE)
            );
            SetLastError(ERROR_INVALID_FLAGS);
            break 'cleanup;
        }

        /* Enumerate all devices of the system */
        DevInstData.hDevInfo = SetupDiGetClassDevsW(
            core::ptr::null(),
            core::ptr::null(),
            hwndParent,
            DIGCF_ALLCLASSES | DIGCF_PRESENT,
        );
        if DevInstData.hDevInfo == INVALID_HANDLE_VALUE {
            break 'cleanup;
        }
        DevInstData.devInfoData.cbSize = (core::mem::size_of::<SP_DEVINFO_DATA>()) as _;
        i = 0;
        'enum_devices: loop {
            if SetupDiEnumDeviceInfo(
                DevInstData.hDevInfo,
                i,
                core::ptr::addr_of_mut!(DevInstData.devInfoData),
            ) == 0
            {
                if GetLastError() != ERROR_NO_MORE_ITEMS {
                    TRACE!(
                        "SetupDiEnumDeviceInfo() failed with error 0x%x\n",
                        GetLastError()
                    );
                    break 'cleanup;
                }
                /* This error was expected */
                break 'enum_devices;
            }

            'body: {
                /* Match Hardware ID */
                FoundHardwareId = (FALSE as i32);
                Property = SPDRP_HARDWAREID;
                loop {
                    /* Get IDs data */
                    Buffer = core::ptr::null_mut();
                    BufferSize = 0;
                    while SetupDiGetDeviceRegistryPropertyW(
                        DevInstData.hDevInfo,
                        core::ptr::addr_of_mut!(DevInstData.devInfoData),
                        Property,
                        core::ptr::null_mut(),
                        Buffer as PBYTE,
                        BufferSize,
                        core::ptr::addr_of_mut!(BufferSize),
                    ) == 0
                    {
                        if GetLastError() == ERROR_FILE_NOT_FOUND {
                            break;
                        } else if GetLastError() != ERROR_INSUFFICIENT_BUFFER {
                            TRACE!(
                                "SetupDiGetDeviceRegistryPropertyW() failed with error 0x%x\n",
                                GetLastError()
                            );
                            break 'cleanup;
                        }
                        /* This error was expected */
                        HeapFree(GetProcessHeap(), 0, Buffer as PVOID);
                        Buffer = HeapAlloc(
                            GetProcessHeap(),
                            0,
                            BufferSize as SIZE_T,
                        ) as LPWSTR;
                        if Buffer.is_null() {
                            TRACE!("HeapAlloc() failed\n");
                            SetLastError(ERROR_NOT_ENOUGH_MEMORY);
                            break 'cleanup;
                        }
                    }
                    if !Buffer.is_null() {
                        /* Check if we match the given hardware ID */
                        CurrentHardwareId = Buffer as LPCWSTR;
                        while *CurrentHardwareId != UNICODE_NULL {
                            if _wcsicmp(CurrentHardwareId, HardwareId) == 0 {
                                FoundHardwareId = (TRUE as i32);
                                break;
                            }
                            CurrentHardwareId = CurrentHardwareId.offset(
                                (wcslen(CurrentHardwareId) + 1) as isize,
                            );
                        }
                    }
                    if FoundHardwareId != 0 || Property == SPDRP_COMPATIBLEIDS {
                        break;
                    }
                    Property = SPDRP_COMPATIBLEIDS;
                }
                if FoundHardwareId == 0 {
                    break 'body;
                }

                /* We need to try to update the driver of this device */

                /* Get Instance ID */
                HeapFree(GetProcessHeap(), 0, Buffer as PVOID);
                Buffer = core::ptr::null_mut();
                if SetupDiGetDeviceInstanceIdW(
                    DevInstData.hDevInfo,
                    core::ptr::addr_of_mut!(DevInstData.devInfoData),
                    core::ptr::null_mut(),
                    0,
                    core::ptr::addr_of_mut!(BufferSize),
                ) != 0
                {
                    /* Error, as the output buffer should be too small */
                    SetLastError(ERROR_GEN_FAILURE);
                    break 'cleanup;
                } else if GetLastError() != ERROR_INSUFFICIENT_BUFFER {
                    TRACE!(
                        "SetupDiGetDeviceInstanceIdW() failed with error 0x%x\n",
                        GetLastError()
                    );
                    break 'cleanup;
                } else {
                    Buffer = HeapAlloc(
                        GetProcessHeap(),
                        0,
                        BufferSize.wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD) as SIZE_T,
                    ) as LPWSTR;
                    if Buffer.is_null() {
                        /* Error, as the output buffer should be too small */
                        TRACE!("HeapAlloc() failed\n");
                        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
                        break 'cleanup;
                    } else if SetupDiGetDeviceInstanceIdW(
                        DevInstData.hDevInfo,
                        core::ptr::addr_of_mut!(DevInstData.devInfoData),
                        Buffer,
                        BufferSize,
                        core::ptr::null_mut(),
                    ) == 0
                    {
                        TRACE!(
                            "SetupDiGetDeviceInstanceIdW() failed with error 0x%x\n",
                            GetLastError()
                        );
                        break 'cleanup;
                    }

                    /* Search driver in the specified .inf file */
                    TRACE!("Trying to update the driver of %s\n", debugstr_w(Buffer));
                    if SearchDriver(
                        core::ptr::addr_of_mut!(DevInstData),
                        core::ptr::null(),
                        FullInfPath,
                    ) == 0
                    {
                        TRACE!("SearchDriver() failed with error 0x%x\n", GetLastError());
                        break 'body;
                    }

                    /* FIXME: HACK! We shouldn't check of ERROR_PRIVILEGE_NOT_HELD */
                    if InstallCurrentDriver(core::ptr::addr_of_mut!(DevInstData)) == 0
                        && GetLastError() != ERROR_PRIVILEGE_NOT_HELD
                    {
                        TRACE!(
                            "InstallCurrentDriver() failed with error 0x%x\n",
                            GetLastError()
                        );
                        break 'body;
                    }

                    /* Do we need to reboot? */
                    InstallParams.cbSize = core::mem::size_of::<SP_DEVINSTALL_PARAMS>() as DWORD;
                    if SetupDiGetDeviceInstallParams(
                        DevInstData.hDevInfo,
                        core::ptr::addr_of_mut!(DevInstData.devInfoData),
                        core::ptr::addr_of_mut!(InstallParams),
                    ) != 0
                    {
                        if (InstallParams.Flags & (DI_NEEDRESTART | DI_NEEDREBOOT)) != 0 {
                            RebootRequired = (TRUE as i32);
                        }
                    }

                    FoundAtLeastOneDevice = (TRUE as i32);
                }
            }
            i = i.wrapping_add(1);
        }

        if FoundAtLeastOneDevice != 0 {
            if !bRebootRequired.is_null() {
                *bRebootRequired = RebootRequired;
            }
            SetLastError(NO_ERROR);
            ret = (TRUE as i32);
        } else {
            TRACE!(
                "No device found with HardwareID %s\n",
                debugstr_w(HardwareId)
            );
            SetLastError(ERROR_NO_SUCH_DEVINST);
        }
    }

    if DevInstData.hDevInfo != INVALID_HANDLE_VALUE {
        SetupDiDestroyDeviceInfoList(DevInstData.hDevInfo);
    }
    HeapFree(GetProcessHeap(), 0, Buffer as PVOID);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn UpdateDriverForPlugAndPlayDevicesA(
    hwndParent: HWND,
    HardwareId: LPCSTR,
    FullInfPath: LPCSTR,
    InstallFlags: DWORD,
    bRebootRequired: PBOOL,
) -> BOOL {
    let mut Result: BOOL = 0;
    let mut HardwareIdW: LPWSTR = core::ptr::null_mut();
    let mut FullInfPathW: LPWSTR = core::ptr::null_mut();

    let mut len: core::ffi::c_int = MultiByteToWideChar(
        CP_ACP as UINT,
        0 as DWORD,
        HardwareId,
        -(1 as core::ffi::c_int),
        core::ptr::null_mut(),
        0 as core::ffi::c_int,
    );
    HardwareIdW = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        (len as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
    ) as LPWSTR;
    if HardwareIdW.is_null() {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY as DWORD);
        return FALSE as BOOL;
    }
    MultiByteToWideChar(
        CP_ACP as UINT,
        0 as DWORD,
        HardwareId,
        -(1 as core::ffi::c_int),
        HardwareIdW,
        len,
    );

    len = MultiByteToWideChar(
        CP_ACP as UINT,
        0 as DWORD,
        FullInfPath,
        -(1 as core::ffi::c_int),
        core::ptr::null_mut(),
        0 as core::ffi::c_int,
    );
    FullInfPathW = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        (len as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
    ) as LPWSTR;
    if FullInfPathW.is_null() {
        HeapFree(GetProcessHeap(), 0 as DWORD, HardwareIdW as PVOID);
        SetLastError(ERROR_NOT_ENOUGH_MEMORY as DWORD);
        return FALSE as BOOL;
    }
    MultiByteToWideChar(
        CP_ACP as UINT,
        0 as DWORD,
        FullInfPath,
        -(1 as core::ffi::c_int),
        FullInfPathW,
        len,
    );

    Result = UpdateDriverForPlugAndPlayDevicesW(
        hwndParent,
        HardwareIdW as LPCWSTR,
        FullInfPathW as LPCWSTR,
        InstallFlags,
        bRebootRequired,
    );

    HeapFree(GetProcessHeap(), 0 as DWORD, HardwareIdW as PVOID);
    HeapFree(GetProcessHeap(), 0 as DWORD, FullInfPathW as PVOID);

    return Result;
}

unsafe extern "C" fn SearchDriver(
    DevInstData: PDEVINSTDATA,
    Directory: LPCWSTR,
    InfFile: LPCWSTR,
) -> BOOL {
    let mut DevInstallParams: SP_DEVINSTALL_PARAMS_W = SP_DEVINSTALL_PARAMS_W {
        cbSize: 0,
        ..core::mem::zeroed()
    };
    let mut ret: BOOL = 0;

    DevInstallParams.cbSize = core::mem::size_of::<SP_DEVINSTALL_PARAMS_W>() as DWORD;
    if SetupDiGetDeviceInstallParamsW(
        (*DevInstData).hDevInfo,
        core::ptr::addr_of_mut!((*DevInstData).devInfoData),
        core::ptr::addr_of_mut!(DevInstallParams),
    ) == 0
    {
        return (FALSE) as BOOL;
    }
    DevInstallParams.FlagsEx |= DI_FLAGSEX_ALLOWEXCLUDEDDRVS as DWORD;

    if !InfFile.is_null() {
        DevInstallParams.Flags |= DI_ENUMSINGLEINF as DWORD;
        wcsncpy(
            core::ptr::addr_of_mut!(DevInstallParams.DriverPath) as *mut WCHAR,
            InfFile as *const WCHAR,
            MAX_PATH as size_t,
        );
    } else if !Directory.is_null() {
        DevInstallParams.Flags &= !(DI_ENUMSINGLEINF as DWORD);
        wcsncpy(
            core::ptr::addr_of_mut!(DevInstallParams.DriverPath) as *mut WCHAR,
            Directory as *const WCHAR,
            MAX_PATH as size_t,
        );
    } else {
        DevInstallParams.Flags &= !(DI_ENUMSINGLEINF as DWORD);
        *(core::ptr::addr_of_mut!(DevInstallParams.DriverPath) as *mut WCHAR) = 0 as WCHAR;
    }

    ret = SetupDiSetDeviceInstallParamsW(
        (*DevInstData).hDevInfo,
        core::ptr::addr_of_mut!((*DevInstData).devInfoData),
        core::ptr::addr_of_mut!(DevInstallParams),
    );
    if ret == 0 {
        return (FALSE) as BOOL;
    }

    ret = SetupDiBuildDriverInfoList(
        (*DevInstData).hDevInfo,
        core::ptr::addr_of_mut!((*DevInstData).devInfoData),
        SPDIT_COMPATDRIVER,
    );
    if ret == 0 {
        return (FALSE) as BOOL;
    }

    (*DevInstData).drvInfoData.cbSize = core::mem::size_of::<SP_DRVINFO_DATA>() as DWORD;
    ret = SetupDiEnumDriverInfoW(
        (*DevInstData).hDevInfo,
        core::ptr::addr_of_mut!((*DevInstData).devInfoData),
        SPDIT_COMPATDRIVER,
        0,
        core::ptr::addr_of_mut!((*DevInstData).drvInfoData),
    );
    if ret == 0 {
        if GetLastError() == ERROR_NO_MORE_ITEMS {
            return (FALSE) as BOOL;
        }
        return (FALSE) as BOOL;
    }

    return (TRUE) as BOOL;
}

unsafe extern "C" fn IsDots(str: LPCWSTR) -> BOOL {
    if wcscmp(str, RUSTCONV_WSTR_C245AFF91259.as_ptr()) != 0 && wcscmp(str, RUSTCONV_WSTR_77ECA80E282A.as_ptr()) != 0 {
        return (FALSE) as BOOL;
    }
    return (TRUE) as BOOL;
}
unsafe extern "C" fn GetFileExt(FileName: LPWSTR) -> LPCWSTR {
    let mut Dot: LPCWSTR;

    Dot = wcsrchr(FileName as *const u16, '.' as u16) as LPCWSTR;
    if Dot.is_null() {
        return RUSTCONV_WSTR_96A296D224F2.as_ptr() as LPCWSTR;
    }

    return Dot;
}

unsafe extern "C" fn SearchDriverRecursive(
    DevInstData: PDEVINSTDATA,
    Path: LPCWSTR,
) -> BOOL {
    let mut wfd: WIN32_FIND_DATAW = core::mem::zeroed();
    let mut DirPath: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut FileName: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut FullPath: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut LastDirPath: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut PathWithPattern: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut ok: BOOL = (TRUE as i32);
    let mut retval: BOOL = (FALSE as i32);
    let mut hFindFile: HANDLE = INVALID_HANDLE_VALUE;

    wcscpy(addr_of_mut!(DirPath) as *mut WCHAR, Path);

    if DirPath[(wcslen(addr_of_mut!(DirPath) as *mut WCHAR)
        - 1 as usize)] != '\\' as u16
    {
        wcscat(addr_of_mut!(DirPath) as *mut WCHAR, RUSTCONV_WSTR_B7FC02907B1F.as_ptr());
    }

    wcscpy(
        addr_of_mut!(PathWithPattern) as *mut WCHAR,
        addr_of_mut!(DirPath) as *mut WCHAR,
    );
    wcscat(addr_of_mut!(PathWithPattern) as *mut WCHAR, RUSTCONV_WSTR_E8A4B2EE7EDE.as_ptr());

    hFindFile = FindFirstFileW(
        addr_of_mut!(PathWithPattern) as *mut WCHAR as LPCWSTR,
        addr_of_mut!(wfd),
    );
    while ok != 0 && hFindFile != INVALID_HANDLE_VALUE {
        'body: {
            wcscpy(
                addr_of_mut!(FileName) as *mut WCHAR,
                addr_of_mut!(wfd.cFileName) as *mut WCHAR,
            );
            if IsDots(addr_of_mut!(FileName) as *mut WCHAR as LPCWSTR) != 0 {
                break 'body;
            }

            if (wfd.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) != 0 {
                /* Recursive search */
                wcscpy(
                    addr_of_mut!(FullPath) as *mut WCHAR,
                    addr_of_mut!(DirPath) as *mut WCHAR,
                );
                wcscat(
                    addr_of_mut!(FullPath) as *mut WCHAR,
                    addr_of_mut!(FileName) as *mut WCHAR,
                );
                if SearchDriverRecursive(
                    DevInstData,
                    addr_of_mut!(FullPath) as *mut WCHAR as LPCWSTR,
                ) != 0
                {
                    retval = (TRUE as i32);
                    /* We continue the search for a better driver */
                }
            } else {
                let pszExtension: LPCWSTR =
                    GetFileExt(addr_of_mut!(FileName) as *mut WCHAR as LPWSTR);

                if (_wcsicmp(pszExtension, RUSTCONV_WSTR_068BF9CAD306.as_ptr())
                    == 0)
                    && (wcscmp(
                        addr_of_mut!(LastDirPath) as *mut WCHAR,
                        addr_of_mut!(DirPath) as *mut WCHAR,
                    ) != 0)
                {
                    wcscpy(
                        addr_of_mut!(LastDirPath) as *mut WCHAR,
                        addr_of_mut!(DirPath) as *mut WCHAR,
                    );

                    if wcslen(addr_of_mut!(DirPath) as *mut WCHAR)
                        > MAX_PATH as usize
                    {
                        /* Path is too long to be searched */
                        break 'body;
                    }

                    if SearchDriver(
                        DevInstData,
                        addr_of_mut!(DirPath) as *mut WCHAR as LPCWSTR,
                        core::ptr::null::<WCHAR>(),
                    ) != 0
                    {
                        retval = (TRUE as i32);
                        /* We continue the search for a better driver */
                    }
                }
            }
        }
        ok = FindNextFileW(hFindFile, addr_of_mut!(wfd));
    }

    if hFindFile != INVALID_HANDLE_VALUE {
        FindClose(hFindFile);
    }
    return retval;
}

#[no_mangle]
pub unsafe extern "C" fn CheckBestDriver(DevInstData: PDEVINSTDATA, pszDir: PCWSTR) -> BOOL {
    SearchDriverRecursive(DevInstData, pszDir)
}
#[no_mangle]
pub unsafe extern "C" fn ScanFoldersForDriver(DevInstData: PDEVINSTDATA) -> BOOL {
    let mut result: BOOL;

    /* Search in default location */
    result = SearchDriver(DevInstData, core::ptr::null_mut(), core::ptr::null_mut());

    if !(*DevInstData).CustomSearchPath.is_null() {
        /* Search only in specified paths */
        /* We need to check all specified directories to be
         * sure to find the best driver for the device.
         */
        let mut Path: LPCWSTR;
        Path = (*DevInstData).CustomSearchPath;
        while *Path != 0 {
            if wcslen(Path) == 2
                && *Path.offset(1 as isize) == ':' as WCHAR
            {
                if SearchDriverRecursive(DevInstData, Path) != 0 {
                    result = (TRUE as i32);
                }
            } else {
                if SearchDriver(DevInstData, Path, core::ptr::null_mut()) != 0 {
                    result = (TRUE as i32);
                }
            }
            Path = Path.offset(
                (wcslen(Path) + 1) as isize,
            );
        }
    }

    return result;
}
#[no_mangle]
pub unsafe extern "C" fn PrepareFoldersToScan(mut DevInstData: PDEVINSTDATA, mut IncludeRemovableDevices: BOOL, mut IncludeCustomPath: BOOL, mut hwndCombo: HWND) -> BOOL {
    let mut drive: [u16; 3] = [0x003Fu16, 0x003A, 0x0000];
    let mut dwDrives: DWORD = 0;
    let mut i: DWORD = 0;
    let mut nType: UINT = 0;
    let mut CustomTextLength: DWORD = 0;
    let mut LengthNeeded: DWORD = 0;
    let mut Buffer: LPWSTR = core::ptr::null_mut();
    let mut idx: INT = SendMessageW(
        hwndCombo,
        CB_GETCURSEL as UINT,
        0 as WPARAM,
        0 as LPARAM,
    ) as INT;

    /* Calculate length needed to store the search paths */
    if IncludeRemovableDevices != 0 {
        dwDrives = GetLogicalDrives();
        drive[0 as usize] = 'A' as u16;
        i = 1;
        while (drive[0 as usize] as u32) <= ('Z' as u32) {
            if (dwDrives & i) != 0 {
                nType = GetDriveTypeW(
                    core::ptr::addr_of_mut!(drive) as *mut WCHAR as LPCWSTR,
                );
                if nType == DRIVE_REMOVABLE as UINT || nType == DRIVE_CDROM as UINT {
                    LengthNeeded = LengthNeeded.wrapping_add(3);
                }
            }
            drive[0 as usize] =
                drive[0 as usize].wrapping_add(1);
            i <<= 1;
        }
    }
    if IncludeCustomPath != 0 {
        CustomTextLength = (1 as INT
            + (if idx != CB_ERR as INT {
                SendMessageW(
                    hwndCombo,
                    CB_GETLBTEXTLEN as UINT,
                    idx as WPARAM,
                    0 as LPARAM,
                ) as INT
            } else {
                ComboBox_GetTextLength!(hwndCombo) as INT
            })) as DWORD;
        LengthNeeded = LengthNeeded.wrapping_add(CustomTextLength);
    }

    /* Allocate space for search paths */
    HeapFree(
        GetProcessHeap(),
        0,
        (*DevInstData).CustomSearchPath as PVOID,
    );
    Buffer = HeapAlloc(
        GetProcessHeap(),
        0,
        (LengthNeeded.wrapping_add(1) as SIZE_T)
            .wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
    ) as LPWSTR;
    (*DevInstData).CustomSearchPath = Buffer;
    if Buffer.is_null() {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY as DWORD);
        return (FALSE) as BOOL;
    }

    /* Fill search paths */
    if IncludeRemovableDevices != 0 {
        drive[0 as usize] = 'A' as u16;
        i = 1;
        while (drive[0 as usize] as u32) <= ('Z' as u32) {
            if (dwDrives & i) != 0 {
                nType = GetDriveTypeW(
                    core::ptr::addr_of_mut!(drive) as *mut WCHAR as LPCWSTR,
                );
                if nType == DRIVE_REMOVABLE as UINT || nType == DRIVE_CDROM as UINT {
                    Buffer = Buffer.offset(
                        (1
                            + _swprintf(
                                Buffer,
                                core::ptr::addr_of_mut!(drive) as *mut WCHAR,
                            )) as isize,
                    );
                }
            }
            drive[0 as usize] =
                drive[0 as usize].wrapping_add(1);
            i <<= 1;
        }
    }
    if IncludeCustomPath != 0 {
        Buffer = Buffer.offset(
            (1 as LRESULT
                + (if idx != CB_ERR as INT {
                    SendMessageW(
                        hwndCombo,
                        CB_GETLBTEXT as UINT,
                        idx as WPARAM,
                        Buffer as LPARAM,
                    ) as LRESULT
                } else {
                    GetWindowTextW(hwndCombo, Buffer, CustomTextLength as INT) as LRESULT
                })) as isize,
        );
    }
    *Buffer = '\0' as WCHAR;

    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn InstallCurrentDriver(mut DevInstData: PDEVINSTDATA) -> BOOL {
    let mut ret: BOOL;

    ret = SetupDiCallClassInstaller(
        DIF_SELECTBESTCOMPATDRV,
        (*DevInstData).hDevInfo,
        core::ptr::addr_of_mut!((*DevInstData).devInfoData),
    );
    if ret == 0 {
        return (FALSE) as BOOL;
    }

    ret = SetupDiCallClassInstaller(
        DIF_ALLOW_INSTALL,
        (*DevInstData).hDevInfo,
        core::ptr::addr_of_mut!((*DevInstData).devInfoData),
    );
    if ret == 0 {
        return (FALSE) as BOOL;
    }

    ret = SetupDiCallClassInstaller(
        DIF_NEWDEVICEWIZARD_PREANALYZE,
        (*DevInstData).hDevInfo,
        core::ptr::addr_of_mut!((*DevInstData).devInfoData),
    );
    if ret == 0 {
        return (FALSE) as BOOL;
    }

    ret = SetupDiCallClassInstaller(
        DIF_NEWDEVICEWIZARD_POSTANALYZE,
        (*DevInstData).hDevInfo,
        core::ptr::addr_of_mut!((*DevInstData).devInfoData),
    );
    if ret == 0 {
        return (FALSE) as BOOL;
    }

    ret = SetupDiCallClassInstaller(
        DIF_INSTALLDEVICEFILES,
        (*DevInstData).hDevInfo,
        core::ptr::addr_of_mut!((*DevInstData).devInfoData),
    );
    if ret == 0 {
        return (FALSE) as BOOL;
    }

    ret = SetupDiCallClassInstaller(
        DIF_REGISTER_COINSTALLERS,
        (*DevInstData).hDevInfo,
        core::ptr::addr_of_mut!((*DevInstData).devInfoData),
    );
    if ret == 0 {
        return (FALSE) as BOOL;
    }

    ret = SetupDiCallClassInstaller(
        DIF_INSTALLINTERFACES,
        (*DevInstData).hDevInfo,
        core::ptr::addr_of_mut!((*DevInstData).devInfoData),
    );
    if ret == 0 {
        return (FALSE) as BOOL;
    }

    ret = SetupDiCallClassInstaller(
        DIF_INSTALLDEVICE,
        (*DevInstData).hDevInfo,
        core::ptr::addr_of_mut!((*DevInstData).devInfoData),
    );
    if ret == 0 {
        return (FALSE) as BOOL;
    }

    ret = SetupDiCallClassInstaller(
        DIF_NEWDEVICEWIZARD_FINISHINSTALL,
        (*DevInstData).hDevInfo,
        core::ptr::addr_of_mut!((*DevInstData).devInfoData),
    );
    if ret == 0 {
        return (FALSE) as BOOL;
    }

    ret = SetupDiCallClassInstaller(
        DIF_DESTROYPRIVATEDATA,
        (*DevInstData).hDevInfo,
        core::ptr::addr_of_mut!((*DevInstData).devInfoData),
    );
    if ret == 0 {
        return (FALSE) as BOOL;
    }

    return TRUE as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn DevInstallW(hWndParent: HWND, hInstance: HINSTANCE, InstanceId: LPCWSTR, Show: INT) -> BOOL {
    let mut DevInstData: PDEVINSTDATA = core::ptr::null_mut();
    let mut ret: BOOL;
    let mut config_flags: DWORD = core::mem::zeroed();
    let mut retval: BOOL = (FALSE as i32);

    if IsUserAdmin() == 0 {
        /* XP kills the process... */
        ExitProcess(ERROR_ACCESS_DENIED);
    }

    DevInstData = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<DEVINSTDATA>() as SIZE_T,
    ) as PDEVINSTDATA;
    if DevInstData.is_null() {
        /* HeapAlloc failed */
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return retval;
    }

    /* Clear devinst data */
    ZeroMemory!(
        DevInstData as LPVOID,
        core::mem::size_of::<DEVINSTDATA>() as SIZE_T,
    );
    (*DevInstData).devInfoData.cbSize = 0; /* Tell if the devInfoData is valid */

    /* Fill devinst data */
    (*DevInstData).hDevInfo = SetupDiCreateDeviceInfoListExW(
        core::ptr::null(),
        core::ptr::null_mut(),
        core::ptr::null(),
        core::ptr::null_mut(),
    );
    if (*DevInstData).hDevInfo == INVALID_HANDLE_VALUE {
        return retval;
    }

    (*DevInstData).devInfoData.cbSize = core::mem::size_of::<SP_DEVINFO_DATA>() as DWORD;
    ret = SetupDiOpenDeviceInfoW(
        (*DevInstData).hDevInfo,
        InstanceId,
        core::ptr::null_mut(),
        0, /* Open flags */
        core::ptr::addr_of_mut!((*DevInstData).devInfoData),
    );
    if ret == 0 {
        (*DevInstData).devInfoData.cbSize = 0;
        return retval;
    }

    SetLastError(ERROR_GEN_FAILURE);
    ret = SetupDiGetDeviceRegistryProperty(
        (*DevInstData).hDevInfo,
        core::ptr::addr_of_mut!((*DevInstData).devInfoData),
        SPDRP_DEVICEDESC,
        core::ptr::addr_of_mut!((*DevInstData).regDataType),
        core::ptr::null_mut(),
        0,
        core::ptr::addr_of_mut!((*DevInstData).requiredSize),
    );

    if ret == 0
        && GetLastError() == ERROR_INSUFFICIENT_BUFFER
        && (*DevInstData).regDataType == REG_SZ
    {
        (*DevInstData).buffer = HeapAlloc(
            GetProcessHeap(),
            0,
            (*DevInstData).requiredSize as SIZE_T,
        ) as PBYTE;
        if (*DevInstData).buffer.is_null() {
            /* HeapAlloc failed */
            SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        } else {
            ret = SetupDiGetDeviceRegistryPropertyW(
                (*DevInstData).hDevInfo,
                core::ptr::addr_of_mut!((*DevInstData).devInfoData),
                SPDRP_DEVICEDESC,
                core::ptr::addr_of_mut!((*DevInstData).regDataType),
                (*DevInstData).buffer,
                (*DevInstData).requiredSize,
                core::ptr::addr_of_mut!((*DevInstData).requiredSize),
            );
        }
    }
    if ret == 0 {
        return retval;
    }

    if SetupDiGetDeviceRegistryPropertyW(
        (*DevInstData).hDevInfo,
        core::ptr::addr_of_mut!((*DevInstData).devInfoData),
        SPDRP_CONFIGFLAGS,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(config_flags) as PBYTE,
        core::mem::size_of::<DWORD>() as DWORD,
        core::ptr::null_mut(),
    ) != 0
    {
        if (config_flags & (CONFIGFLAG_FAILEDINSTALL as DWORD)) != 0 {
            /* The device is disabled */
            retval = (TRUE as i32);
            return retval;
        }
    }

    /* Search driver in default location and removable devices */
    if PrepareFoldersToScan(
        DevInstData,
        (FALSE) as BOOL,
        (FALSE) as BOOL,
        core::ptr::null_mut(),
    ) == 0
    {
        return retval;
    }
    if ScanFoldersForDriver(DevInstData) != 0 {
        /* Driver found ; install it */
        retval = InstallCurrentDriver(DevInstData);
        if retval != 0 && Show != (SW_HIDE as i32) {
            /* Should we display the 'Need to reboot' page? */
            let mut installParams: SP_DEVINSTALL_PARAMS = core::mem::zeroed();
            installParams.cbSize = core::mem::size_of::<SP_DEVINSTALL_PARAMS>() as DWORD;
            if SetupDiGetDeviceInstallParams(
                (*DevInstData).hDevInfo,
                core::ptr::addr_of_mut!((*DevInstData).devInfoData),
                core::ptr::addr_of_mut!(installParams),
            ) != 0
            {
                if (installParams.Flags
                    & ((DI_NEEDRESTART | DI_NEEDREBOOT) as DWORD))
                    != 0
                {
                    retval = DisplayWizard(
                        DevInstData,
                        hWndParent,
                        IDD_NEEDREBOOT,
                    );
                }
            }
        }
        /* goto cleanup */
    } else if Show == (SW_HIDE as i32) {
        /* We can't show the wizard. Fail the install */
    } else {
        /* Prepare the wizard, and display it */
        retval = DisplayWizard(DevInstData, hWndParent, IDD_WELCOMEPAGE);
    }

    /* cleanup: */
    if !DevInstData.is_null() {
        if (*DevInstData).devInfoData.cbSize != 0 {
            if SetupDiDestroyDriverInfoList(
                (*DevInstData).hDevInfo,
                core::ptr::addr_of_mut!((*DevInstData).devInfoData),
                SPDIT_COMPATDRIVER,
            ) == 0
            {
                TRACE!(
                    "SetupDiDestroyDriverInfoList() failed with error 0x%lx\n",
                    GetLastError()
                );
            }
        }
        if (*DevInstData).hDevInfo != INVALID_HANDLE_VALUE {
            if SetupDiDestroyDeviceInfoList((*DevInstData).hDevInfo) == 0 {
                TRACE!(
                    "SetupDiDestroyDeviceInfoList() failed with error 0x%lx\n",
                    GetLastError()
                );
            }
        }
        HeapFree(
            GetProcessHeap(),
            0,
            (*DevInstData).buffer as PVOID,
        );
        HeapFree(
            GetProcessHeap(),
            0,
            DevInstData as PVOID,
        );
    }

    return retval;
}

#[no_mangle]
pub unsafe extern "system" fn InstallDevInstEx(
    hWndParent: HWND,
    InstanceId: LPCWSTR,
    bUpdate: BOOL,
    lpReboot: LPDWORD,
    Unknown_var: DWORD,
) -> BOOL {
    let mut DevInstData: PDEVINSTDATA = core::ptr::null_mut();
    let mut ret: BOOL = 0;
    let mut retval: BOOL = (FALSE as i32);

    'cleanup: {
        DevInstData = HeapAlloc(GetProcessHeap(), 0, (core::mem::size_of::<DEVINSTDATA>()) as SIZE_T) as PDEVINSTDATA;
        if DevInstData.is_null() {
            SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            break 'cleanup;
        }

        /* Clear devinst data */
        ZeroMemory!(DevInstData as PVOID, core::mem::size_of::<DEVINSTDATA>());
        (*DevInstData).devInfoData.cbSize = 0; /* Tell if the devInfoData is valid */
        (*DevInstData).bUpdate = bUpdate;

        /* Fill devinst data */
        (*DevInstData).hDevInfo = SetupDiCreateDeviceInfoListExW(core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut());
        if (*DevInstData).hDevInfo == INVALID_HANDLE_VALUE {
            break 'cleanup;
        }

        (*DevInstData).devInfoData.cbSize = core::mem::size_of::<SP_DEVINFO_DATA>() as DWORD;
        ret = SetupDiOpenDeviceInfoW(
            (*DevInstData).hDevInfo,
            InstanceId,
            core::ptr::null_mut(),
            0, /* Open flags */
            core::ptr::addr_of_mut!((*DevInstData).devInfoData),
        );
        if ret == 0 {
            (*DevInstData).devInfoData.cbSize = 0;
            break 'cleanup;
        }

        SetLastError(ERROR_GEN_FAILURE);
        ret = SetupDiGetDeviceRegistryProperty(
            (*DevInstData).hDevInfo,
            core::ptr::addr_of_mut!((*DevInstData).devInfoData),
            SPDRP_DEVICEDESC,
            core::ptr::addr_of_mut!((*DevInstData).regDataType),
            core::ptr::null_mut(),
            0,
            core::ptr::addr_of_mut!((*DevInstData).requiredSize),
        );

        if ret == 0 && GetLastError() == ERROR_INSUFFICIENT_BUFFER && (*DevInstData).regDataType == REG_SZ {
            (*DevInstData).buffer = HeapAlloc(GetProcessHeap(), 0, (*DevInstData).requiredSize as SIZE_T) as PBYTE;
            if (*DevInstData).buffer.is_null() {
                SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            } else {
                ret = SetupDiGetDeviceRegistryPropertyW(
                    (*DevInstData).hDevInfo,
                    core::ptr::addr_of_mut!((*DevInstData).devInfoData),
                    SPDRP_DEVICEDESC,
                    core::ptr::addr_of_mut!((*DevInstData).regDataType),
                    (*DevInstData).buffer,
                    (*DevInstData).requiredSize,
                    core::ptr::addr_of_mut!((*DevInstData).requiredSize),
                );
            }
        }

        if ret == 0 {
            break 'cleanup;
        }

        /* Prepare the wizard, and display it */
        retval = DisplayWizard(DevInstData, hWndParent, IDD_WELCOMEPAGE);
    }

    /* cleanup: */
    if !DevInstData.is_null() {
        if (*DevInstData).devInfoData.cbSize != 0 {
            if SetupDiDestroyDriverInfoList((*DevInstData).hDevInfo, core::ptr::addr_of_mut!((*DevInstData).devInfoData), SPDIT_COMPATDRIVER) == 0 {
                TRACE!("SetupDiDestroyDriverInfoList() failed with error 0x%lx\n", GetLastError());
            }
        }
        if (*DevInstData).hDevInfo != INVALID_HANDLE_VALUE {
            if SetupDiDestroyDeviceInfoList((*DevInstData).hDevInfo) == 0 {
                TRACE!("SetupDiDestroyDeviceInfoList() failed with error 0x%lx\n", GetLastError());
            }
        }
        HeapFree(GetProcessHeap(), 0, (*DevInstData).buffer as PVOID);
        HeapFree(GetProcessHeap(), 0, DevInstData as PVOID);
    }

    return retval;
}

#[no_mangle]
pub unsafe extern "system" fn InstallDevInst(
    hWndParent: HWND,
    InstanceId: LPCWSTR,
    bUpdate: BOOL,
    lpReboot: LPDWORD,
) -> BOOL {
    return InstallDevInstEx(hWndParent, InstanceId, bUpdate, lpReboot, 0);
}

#[no_mangle]
pub unsafe extern "system" fn ClientSideInstallW(
    hWndOwner: HWND,
    hInstance: HINSTANCE,
    lpNamedPipeName: LPWSTR,
    Show: INT,
) -> BOOL {
    let mut ReturnValue: BOOL = (FALSE as i32);
    let mut ShowWizard: BOOL = 0;
    let mut BytesRead: DWORD = 0;
    let mut Value: DWORD = 0;
    let mut hPipe: HANDLE = INVALID_HANDLE_VALUE;
    let mut DeviceInstance: PWSTR = core::ptr::null_mut();
    let mut InstallEventName: PWSTR = core::ptr::null_mut();
    let mut hInstallEvent: HANDLE = core::ptr::null_mut();

    'cleanup: {
        /* Open the pipe */
        hPipe = CreateFileW(
            lpNamedPipeName,
            GENERIC_READ,
            0,
            core::ptr::null_mut(),
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            core::ptr::null_mut(),
        );

        if hPipe == INVALID_HANDLE_VALUE {
            ERR!("CreateFileW failed with error %u\n", GetLastError());
            break 'cleanup;
        }

        /* Read the data. Some is just included for compatibility with Windows right now and not yet used by ReactOS.
           See umpnpmgr for more details. */
        if ReadFile(
            hPipe,
            core::ptr::addr_of_mut!(Value) as LPVOID,
            core::mem::size_of::<DWORD>() as DWORD,
            core::ptr::addr_of_mut!(BytesRead),
            core::ptr::null_mut(),
        ) == 0
        {
            ERR!("ReadFile failed with error %u\n", GetLastError());
            break 'cleanup;
        }

        InstallEventName = HeapAlloc(
            GetProcessHeap(),
            0,
            Value as SIZE_T,
        ) as PWSTR;

        if ReadFile(
            hPipe,
            InstallEventName as LPVOID,
            Value,
            core::ptr::addr_of_mut!(BytesRead),
            core::ptr::null_mut(),
        ) == 0
        {
            ERR!("ReadFile failed with error %u\n", GetLastError());
            break 'cleanup;
        }

        /* I couldn't figure out what the following value means under Windows XP.
           Therefore I used it in umpnpmgr to pass the ShowWizard variable. */
        if ReadFile(
            hPipe,
            core::ptr::addr_of_mut!(ShowWizard) as LPVOID,
            core::mem::size_of::<BOOL>() as DWORD,
            core::ptr::addr_of_mut!(BytesRead),
            core::ptr::null_mut(),
        ) == 0
        {
            ERR!("ReadFile failed with error %u\n", GetLastError());
            break 'cleanup;
        }

        /* Next one is again size in bytes of the following string */
        if ReadFile(
            hPipe,
            core::ptr::addr_of_mut!(Value) as LPVOID,
            core::mem::size_of::<DWORD>() as DWORD,
            core::ptr::addr_of_mut!(BytesRead),
            core::ptr::null_mut(),
        ) == 0
        {
            ERR!("ReadFile failed with error %u\n", GetLastError());
            break 'cleanup;
        }

        DeviceInstance = HeapAlloc(
            GetProcessHeap(),
            0,
            Value as SIZE_T,
        ) as PWSTR;

        if ReadFile(
            hPipe,
            DeviceInstance as LPVOID,
            Value,
            core::ptr::addr_of_mut!(BytesRead),
            core::ptr::null_mut(),
        ) == 0
        {
            ERR!("ReadFile failed with error %u\n", GetLastError());
            break 'cleanup;
        }

        ReturnValue = DevInstallW(
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            DeviceInstance as LPCWSTR,
            if ShowWizard != 0 {
                (SW_SHOWNOACTIVATE as i32)
            } else {
                (SW_HIDE as i32)
            },
        );
        if ReturnValue == 0 {
            ERR!("DevInstallW failed with error %lu\n", GetLastError());
            break 'cleanup;
        }

        hInstallEvent = CreateEventW(
            core::ptr::null_mut(),
            (TRUE) as BOOL,
            (FALSE) as BOOL,
            InstallEventName as LPCWSTR,
        );
        if hInstallEvent.is_null() {
            TRACE!(
                "CreateEventW('%ls') failed with error %lu\n",
                InstallEventName,
                GetLastError()
            );
            break 'cleanup;
        }

        SetEvent(hInstallEvent);
        CloseHandle(hInstallEvent);
    }

    if hPipe != INVALID_HANDLE_VALUE {
        CloseHandle(hPipe);
    }

    if !InstallEventName.is_null() {
        HeapFree(
            GetProcessHeap(),
            0,
            InstallEventName as PVOID,
        );
    }

    if !DeviceInstance.is_null() {
        HeapFree(
            GetProcessHeap(),
            0,
            DeviceInstance as PVOID,
        );
    }

    return ReturnValue;
}
#[no_mangle]
pub unsafe extern "system" fn DllMain(
       hInstance: HINSTANCE,
       dwReason: DWORD,
       lpReserved: LPVOID) -> BOOL {
    if dwReason == DLL_PROCESS_ATTACH as DWORD
    {
        let mut InitControls: INITCOMMONCONTROLSEX = core::mem::zeroed();

        DisableThreadLibraryCalls(hInstance as HMODULE);

        InitControls.dwSize = core::mem::size_of::<INITCOMMONCONTROLSEX>() as DWORD;
        InitControls.dwICC = ICC_PROGRESS_CLASS as DWORD;
        InitCommonControlsEx(core::ptr::addr_of_mut!(InitControls));
        hDllInstance = hInstance;
    }

    return TRUE as BOOL;
}
