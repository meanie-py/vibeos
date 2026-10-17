// rustconv-assembly: whole-tu
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct _CDevSettings {
    pub lpIDataObjectVtbl: *const IDataObjectVtbl,
    pub ref_: DWORD,

    pub cfExtInterface: CLIPFORMAT,
    pub cfDisplayDevice: CLIPFORMAT,
    pub cfDisplayName: CLIPFORMAT,
    pub cfDisplayId: CLIPFORMAT,
    pub cfMonitorName: CLIPFORMAT,
    pub cfMonitorDevice: CLIPFORMAT,
    pub cfDisplayKey: CLIPFORMAT,
    pub cfDisplayStateFlags: CLIPFORMAT,
    pub cfPruningMode: CLIPFORMAT,

    pub pDisplayDevice: PWSTR,
    pub pDisplayName: PWSTR,
    pub pDisplayKey: PWSTR,
    pub pDisplayId: PWSTR,
    pub pMonitorName: PWSTR,
    pub pMonitorDevice: PWSTR,

    pub ExtInterface: DESK_EXT_INTERFACE,

    pub lpOrigDevMode: PDEVMODEW,
    pub lpCurDevMode: PDEVMODEW,
    pub DevModes: *mut PDEVMODEW,
    pub nDevModes: DWORD,
    pub StateFlags: DWORD,

    pub __bindgen_anon_1: _CDevSettings__bindgen_ty_1,
}

#[repr(C)]
pub union _CDevSettings__bindgen_ty_1 {
    pub Flags: DWORD,
    pub __bindgen_anon_1: _CDevSettings__bindgen_ty_1__bindgen_ty_1,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct _CDevSettings__bindgen_ty_1__bindgen_ty_1 {
    pub _bitfield_1: DWORD,
}

// The C's anonymous union/struct members, which C injects into _CDevSettings's own name
// scope. Bit 0..2 in declaration order — LSB first, as every compiler
// targeting these three ABIs allocates them.
impl _CDevSettings {
    #[inline(always)]
    pub unsafe fn bModesPruned(&self) -> DWORD {
        (self.__bindgen_anon_1.__bindgen_anon_1._bitfield_1 >> 0) & 1
    }
    #[inline(always)]
    pub unsafe fn set_bModesPruned(&mut self, value: DWORD) {
        let bits = &mut self.__bindgen_anon_1.__bindgen_anon_1._bitfield_1;
        *bits = (*bits & !(1 << 0)) | (((value != 0) as DWORD) << 0);
    }
    #[inline(always)]
    pub unsafe fn bKeyIsReadOnly(&self) -> DWORD {
        (self.__bindgen_anon_1.__bindgen_anon_1._bitfield_1 >> 1) & 1
    }
    #[inline(always)]
    pub unsafe fn set_bKeyIsReadOnly(&mut self, value: DWORD) {
        let bits = &mut self.__bindgen_anon_1.__bindgen_anon_1._bitfield_1;
        *bits = (*bits & !(1 << 1)) | (((value != 0) as DWORD) << 1);
    }
    #[inline(always)]
    pub unsafe fn bPruningOn(&self) -> DWORD {
        (self.__bindgen_anon_1.__bindgen_anon_1._bitfield_1 >> 2) & 1
    }
    #[inline(always)]
    pub unsafe fn set_bPruningOn(&mut self, value: DWORD) {
        let bits = &mut self.__bindgen_anon_1.__bindgen_anon_1._bitfield_1;
        *bits = (*bits & !(1 << 2)) | (((value != 0) as DWORD) << 2);
    }
}

pub type CDevSettings = _CDevSettings;
pub type PCDevSettings = *mut _CDevSettings;
unsafe extern "C" fn impl_from_IDataObject(iface: *mut IDataObject) -> PCDevSettings {
    ((iface as ULONG_PTR) - FIELD_OFFSET!(CDevSettings, lpIDataObjectVtbl)) as PCDevSettings
}
unsafe extern "C" fn pCDevSettings_FreeString(psz: *mut PWSTR) {
    if (*psz) != core::ptr::null_mut() {
        LocalFree((*psz) as HLOCAL);
        *psz = core::ptr::null_mut();
    }
}
unsafe extern "C" fn pCDevSettings_AllocAndCopyString(pszSrc: LPCTSTR) -> PWSTR {
    let mut c: usize;
    let mut str: PWSTR;

    c = _tcslen(pszSrc) + 1;
    str = LocalAlloc(LMEM_FIXED, (c * core::mem::size_of::<u16>()) as SIZE_T) as PWSTR;
    if !str.is_null() {
        // desk is built UNICODE (`set_module_type(desk cpl UNICODE)`), so the C takes its
        // `#ifdef UNICODE` branch. This was gated on an undeclared `feature = "unicode"`, which
        // is never on, so it ran the ANSI branch on a wide string: MultiByteToWideChar stopped
        // at the first UTF-16 high byte and every device name, description and key came out
        // one character long.
        StringCbCopyW(str, c * core::mem::size_of::<u16>(), pszSrc);
    }

    str
}
unsafe extern "C" fn pCDevSettings_GetMonitorName(pszDisplayDevice: *const u16) -> *mut u16 {
    let mut dd: _DISPLAY_DEVICEW = core::mem::zeroed();
    let mut dd2: _DISPLAY_DEVICEW = core::mem::zeroed();
    let mut str: *mut u16 = core::ptr::null_mut();

    dd.cb = core::mem::size_of::<_DISPLAY_DEVICEW>() as u32;
    if EnumDisplayDevicesW(
        pszDisplayDevice,
        0,
        &mut dd,
        0,
    ) != 0
    {
        dd2.cb = core::mem::size_of::<_DISPLAY_DEVICEW>() as u32;
        if EnumDisplayDevicesW(
            pszDisplayDevice,
            1,
            &mut dd2,
            0,
        ) != 0
        {
            LoadStringW(
                hApplet,
                IDS_MULTIPLEMONITORS,
                dd.DeviceString.as_mut_ptr(),
                (core::mem::size_of_val(&dd.DeviceString) / core::mem::size_of_val(&dd.DeviceString[0])) as i32,
            );
        }
    } else {
        LoadStringW(
            hApplet,
            IDS_UNKNOWNMONITOR,
            dd.DeviceString.as_mut_ptr(),
            (core::mem::size_of_val(&dd.DeviceString) / core::mem::size_of_val(&dd.DeviceString[0])) as i32,
        );
    }

    str = LocalAlloc(
        LMEM_FIXED,
        ((wcslen(dd.DeviceString.as_ptr()) + 1) * core::mem::size_of::<u16>()) as SIZE_T,
    ) as *mut u16;
    if !str.is_null() {
        wcscpy(str, dd.DeviceString.as_ptr());
    }

    str
}
unsafe extern "C" fn pCDevSettings_GetMonitorDevice(pszDisplayDevice: *const u16) -> *mut u16 {
    let mut dd: DISPLAY_DEVICEW = core::mem::zeroed();
    let mut str: *mut u16 = core::ptr::null_mut();

    dd.cb = core::mem::size_of::<DISPLAY_DEVICEW>() as u32;
    if EnumDisplayDevicesW(pszDisplayDevice,
                            0,
                            &mut dd,
                            0) != 0
    {
        str = LocalAlloc(LMEM_FIXED,
                         ((wcslen(dd.DeviceName.as_ptr()) + 1) * core::mem::size_of::<u16>()) as SIZE_T) as *mut u16;
        if str != core::ptr::null_mut()
        {
            wcscpy(str,
                   dd.DeviceName.as_ptr());
        }
    }

    str
}
unsafe extern "C" fn pCDevSettings_GetDeviceInstanceId(pszDevice: *const WCHAR) -> PWSTR {
    let mut DevInfo: HDEVINFO;
    let mut InfoData: SP_DEVINFO_DATA = core::mem::zeroed();
    let mut BufLen: ULONG = 0;
    let mut lpDevInstId: LPWSTR = core::ptr::null_mut();

    // DPRINT removed

    DevInfo = SetupDiGetClassDevsW(
        core::ptr::null_mut(),
        pszDevice,
        core::ptr::null_mut(),
        DIGCF_ALLCLASSES | DIGCF_PRESENT,
    );
    if DevInfo == INVALID_HANDLE_VALUE {
        // DPRINT removed
        return core::ptr::null_mut();
    }

    ZeroMemory!(
        &mut InfoData as *mut _ as *mut _,
        core::mem::size_of_val(&InfoData),
    );
    InfoData.cbSize = core::mem::size_of::<SP_DEVINFO_DATA>() as DWORD;

    if SetupDiEnumDeviceInfo(DevInfo, 0, &mut InfoData) == 0 {
        // DPRINT removed
        return core::ptr::null_mut();
    }

    if SetupDiGetDeviceInstanceId(
        DevInfo,
        &mut InfoData,
        core::ptr::null_mut(),
        0,
        &mut BufLen,
    ) != 0
        || GetLastError() != ERROR_INSUFFICIENT_BUFFER
    {
        // DPRINT removed
        return core::ptr::null_mut();
    }

    lpDevInstId = (LocalAlloc(
        LMEM_FIXED,
        ((BufLen + 1) * (core::mem::size_of::<WCHAR>()) as u32) as SIZE_T,
    )).cast();

    if lpDevInstId.is_null() {
        // DPRINT removed
        return core::ptr::null_mut();
    }

    if SetupDiGetDeviceInstanceId(
        DevInfo,
        &mut InfoData,
        lpDevInstId,
        BufLen,
        core::ptr::null_mut(),
    ) == 0
    {
        // DPRINT removed
        LocalFree(lpDevInstId as HLOCAL);
        lpDevInstId = core::ptr::null_mut();
    } else {
        // DPRINT removed
    }

    return lpDevInstId;
}
unsafe extern "C" fn pCDevSettings_OpenDeviceKey(
    This: *mut CDevSettings,
    ReadOnly: BOOL,
) -> HKEY {
    static szRegPrefix: [u16; 19] = [
        b'\\' as u16, b'R' as u16, b'e' as u16, b'g' as u16, b'i' as u16, b's' as u16,
        b't' as u16, b'r' as u16, b'y' as u16, b'\\' as u16, b'M' as u16, b'a' as u16,
        b'c' as u16, b'h' as u16, b'i' as u16, b'n' as u16, b'e' as u16, b'\\' as u16,
        0,
    ];
    let mut lpRegKey: PWSTR;
    let mut Access: REGSAM = KEY_READ;
    let mut hKey: HKEY = core::ptr::null_mut();

    lpRegKey = (*This).pDisplayKey;
    if !lpRegKey.is_null() {
        if wcslen(lpRegKey) >= wcslen(szRegPrefix.as_ptr())
            && _wcsnicmp(
                lpRegKey,
                szRegPrefix.as_ptr(),
                wcslen(szRegPrefix.as_ptr()),
            ) == 0
        {
            lpRegKey = lpRegKey.add(wcslen(szRegPrefix.as_ptr()));
        }

        if ReadOnly == 0 {
            Access |= KEY_WRITE;
        }

        if RegOpenKeyEx(
            HKEY_LOCAL_MACHINE,
            lpRegKey,
            0,
            Access,
            &mut hKey,
        ) == (ERROR_SUCCESS) as i32
        {
            return hKey;
        }
    }

    core::ptr::null_mut()
}
#[no_mangle]
pub unsafe extern "system" fn CDevSettings_EnumAllModes(
    Context: PVOID,
    Index: DWORD,
) -> PDEVMODEW {
    let This = impl_from_IDataObject(Context as *mut IDataObject);
    let mut idx = 0;

    // DPRINT removed

    if (*This).DevModes.is_null() {
        return core::ptr::null_mut();
    }

    for i in 0..(*This).nDevModes {
        if (*(*This).DevModes.offset(i as isize)).is_null() {
            continue;
        }

        if idx == Index {
            return *(*This).DevModes.offset(i as isize);
        }

        idx += 1;
    }

    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn CDevSettings_GetCurrentMode(Context: PVOID) -> PDEVMODEW {
    let This = impl_from_IDataObject(Context as *mut IDataObject);

    // DPRINT removed
    (*This).lpCurDevMode
}

#[no_mangle]
pub unsafe extern "system" fn CDevSettings_SetCurrentMode(
    Context: PVOID,
    pDevMode: *mut DEVMODEW,
) -> BOOL {
    let This = impl_from_IDataObject(Context as *mut IDataObject);
    let mut i: DWORD;

    // DPRINT removed

    if (*This).DevModes.is_null() {
        return (FALSE) as i32;
    }

    i = 0;
    while i < (*This).nDevModes {
        if (*(*This).DevModes.offset(i as isize)).is_null() {
            i = i.wrapping_add(1);
            continue;
        }

        if *(*This).DevModes.offset(i as isize) == pDevMode {
            (*This).lpCurDevMode = pDevMode;
            return (TRUE) as i32;
        }
        i = i.wrapping_add(1);
    }

    return (FALSE) as i32;
}
#[no_mangle]
pub unsafe extern "system" fn CDevSettings_GetPruningMode(
    Context: PVOID,
    pbModesPruned: PBOOL,
    pbKeyIsReadOnly: PBOOL,
    pbPruningOn: PBOOL,
) {
    let This = impl_from_IDataObject(Context as *mut IDataObject);

    // DPRINT removed

    *pbModesPruned = ((*This).bModesPruned()) as i32;
    *pbKeyIsReadOnly = ((*This).bKeyIsReadOnly()) as i32;
    *pbPruningOn = ((*This).bPruningOn()) as i32;
}

#[no_mangle]
pub unsafe extern "system" fn CDevSettings_SetPruningMode(
    Context: PVOID,
    PruningOn: BOOL,
) {
    let mut hKey: HKEY;
    let mut dwValue: DWORD;
    let This: PCDevSettings = impl_from_IDataObject(Context as *mut IDataObject);

    // DPRINT removed

    if (*This).bModesPruned() != 0 && (*This).bKeyIsReadOnly() == 0 &&
        (if PruningOn != 0 { 1 } else { 0 }) != (*This).bPruningOn() {
        (*This).set_bPruningOn(if PruningOn != (FALSE) as i32 { 1 } else { 0 });

        hKey = pCDevSettings_OpenDeviceKey(This, (FALSE) as BOOL);
        if hKey != core::ptr::null_mut() {
            dwValue = (*This).bPruningOn();

            RegSetValueEx(
                hKey,
                TEXT(w!("PruningMode") as LPCTSTR),
                0,
                REG_DWORD,
                core::ptr::addr_of!(dwValue) as *const BYTE,
                core::mem::size_of_val(&dwValue) as DWORD,
            );

            RegCloseKey(hKey);
        }
    }
}
unsafe extern "C" fn pCDevSettings_ReadHardwareInfo(
    hKey: HKEY,
    lpValueName: LPCTSTR,
    lpBuffer: LPWSTR,
) {
    let mut type_: DWORD = REG_BINARY;
    let mut size: DWORD = 128 * core::mem::size_of::<WCHAR>() as DWORD;
    RegQueryValueEx(
        hKey,
        lpValueName,
        core::ptr::null_mut(),
        &mut type_,
        lpBuffer as PBYTE,
        &mut size,
    );
}
unsafe extern "C" fn pCDevSettings_InitializeExtInterface(This: *mut CDevSettings) {
    let Interface: *mut DESK_EXT_INTERFACE = core::ptr::addr_of_mut!((*This).ExtInterface);
    let mut hKeyDev: HKEY;

    ZeroMemory!(
        Interface as *mut core::ffi::c_void,
        core::mem::size_of::<DESK_EXT_INTERFACE>(),
    );
    (*Interface).cbSize = core::mem::size_of::<DESK_EXT_INTERFACE>() as u32;

    (*Interface).Context = impl_to_interface!(This, IDataObject) as *mut core::ffi::c_void;
    (*Interface).EnumAllModes = Some(CDevSettings_EnumAllModes);
    (*Interface).SetCurrentMode = Some(CDevSettings_SetCurrentMode);
    (*Interface).GetCurrentMode = Some(CDevSettings_GetCurrentMode);
    (*Interface).SetPruningMode = Some(CDevSettings_SetPruningMode);
    (*Interface).GetPruningMode = Some(CDevSettings_GetPruningMode);

    hKeyDev = pCDevSettings_OpenDeviceKey(This, (TRUE) as BOOL);
    if hKeyDev != core::ptr::null_mut() {
        let mut dwType: DWORD = 0;
        let mut dwMemSize: DWORD = 0;
        let mut dwSize: DWORD = core::mem::size_of::<DWORD>() as DWORD;

        if RegQueryValueEx(
            hKeyDev,
            TEXT(w!("HardwareInformation.MemorySize") as *const u16),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(dwType),
            core::ptr::addr_of_mut!(dwMemSize) as PBYTE,
            core::ptr::addr_of_mut!(dwSize),
        ) == (ERROR_SUCCESS) as i32
            && (dwType == REG_BINARY || dwType == REG_DWORD)
            && dwSize == core::mem::size_of::<DWORD>() as DWORD
        {
            dwMemSize /= 1024;

            if dwMemSize > 1024 {
                dwMemSize /= 1024;
                if dwMemSize > 1024 {
                    wsprintf(
                        (*Interface).MemorySize.as_mut_ptr(),
                        _T(w!("%u GB") as *const u16),
                        dwMemSize / 1024,
                    );
                } else {
                    wsprintf(
                        (*Interface).MemorySize.as_mut_ptr(),
                        _T(w!("%u MB") as *const u16),
                        dwMemSize,
                    );
                }
            } else {
                wsprintf(
                    (*Interface).MemorySize.as_mut_ptr(),
                    _T(w!("%u KB") as *const u16),
                    dwMemSize,
                );
            }
        }

        pCDevSettings_ReadHardwareInfo(
            hKeyDev,
            TEXT(w!("HardwareInformation.ChipType") as *const u16),
            (*Interface).ChipType.as_mut_ptr(),
        );
        pCDevSettings_ReadHardwareInfo(
            hKeyDev,
            TEXT(w!("HardwareInformation.DacType") as *const u16),
            (*Interface).DacType.as_mut_ptr(),
        );
        pCDevSettings_ReadHardwareInfo(
            hKeyDev,
            TEXT(w!("HardwareInformation.AdapterString") as *const u16),
            (*Interface).AdapterString.as_mut_ptr(),
        );
        pCDevSettings_ReadHardwareInfo(
            hKeyDev,
            TEXT(w!("HardwareInformation.BiosString") as *const u16),
            (*Interface).BiosString.as_mut_ptr(),
        );
        RegCloseKey(hKeyDev);
    }
}
unsafe extern "C" fn pCDevSettings_Initialize(
    This: *mut CDevSettings,
    DisplayDeviceInfo: *mut DISPLAY_DEVICE_ENTRY,
) -> HRESULT {
    let mut hKey: HKEY;
    let mut i: DWORD = 0;
    let mut dwSize: DWORD;
    let mut devmode: DEVMODEW = core::mem::zeroed();

    (*This).lpOrigDevMode = core::ptr::null_mut();
    (*This).lpCurDevMode = core::ptr::null_mut();
    (*This).DevModes = core::ptr::null_mut();
    (*This).nDevModes = 0;
    (*This).__bindgen_anon_1.Flags = 0;
    (*This).StateFlags = (*DisplayDeviceInfo).DeviceStateFlags;
    // DPRINT removed

    (*This).cfExtInterface = (RegisterClipboardFormat(DESK_EXT_EXTINTERFACE!())) as u16;
    (*This).cfDisplayDevice = (RegisterClipboardFormat(DESK_EXT_DISPLAYDEVICE!())) as u16;
    (*This).cfDisplayName = (RegisterClipboardFormat(DESK_EXT_DISPLAYNAME!())) as u16;
    (*This).cfDisplayId = (RegisterClipboardFormat(DESK_EXT_DISPLAYID!())) as u16;
    (*This).cfDisplayKey = (RegisterClipboardFormat(DESK_EXT_DISPLAYKEY!())) as u16;
    (*This).cfDisplayStateFlags = (RegisterClipboardFormat(DESK_EXT_DISPLAYSTATEFLAGS!())) as u16;
    (*This).cfMonitorName = (RegisterClipboardFormat(DESK_EXT_MONITORNAME!())) as u16;
    (*This).cfMonitorDevice = (RegisterClipboardFormat(DESK_EXT_MONITORDEVICE!())) as u16;
    (*This).cfPruningMode = (RegisterClipboardFormat(DESK_EXT_PRUNINGMODE!())) as u16;

    (*This).pDisplayDevice = pCDevSettings_AllocAndCopyString(((*DisplayDeviceInfo).DeviceName) as *const u16);
    // DPRINT removed
    (*This).pDisplayName = pCDevSettings_AllocAndCopyString(((*DisplayDeviceInfo).DeviceDescription) as *const u16);
    // DPRINT removed
    (*This).pDisplayKey = pCDevSettings_AllocAndCopyString(((*DisplayDeviceInfo).DeviceKey) as *const u16);
    // DPRINT removed
    (*This).pDisplayId = pCDevSettings_GetDeviceInstanceId(((*DisplayDeviceInfo).DeviceID) as *const u16);
    // DPRINT removed
    (*This).pMonitorName = pCDevSettings_GetMonitorName((*This).pDisplayDevice);
    // DPRINT removed
    (*This).pMonitorDevice = pCDevSettings_GetMonitorDevice((*This).pDisplayDevice);
    // DPRINT removed

    (*This).set_bModesPruned((((*DisplayDeviceInfo).DeviceStateFlags & DISPLAY_DEVICE_MODESPRUNED) != 0) as u32);
    hKey = pCDevSettings_OpenDeviceKey(This, (FALSE) as BOOL);
    if hKey.is_null() {
        hKey = pCDevSettings_OpenDeviceKey(This, (FALSE) as BOOL);
        (*This).set_bKeyIsReadOnly((TRUE) as u32);
    }
    if !hKey.is_null() {
        let mut dw: DWORD = 0;
        let mut dwType: DWORD = 0;

        dwSize = core::mem::size_of_val(&dw) as DWORD;
        if RegQueryValueEx(
            hKey,
            TEXT(w!("PruningMode") as LPCTSTR),
            core::ptr::null_mut(),
            &mut dwType,
            &mut dw as *mut DWORD as PBYTE,
            &mut dwSize,
        ) == (ERROR_SUCCESS) as i32
        {
            if dwType == REG_DWORD && dwSize == core::mem::size_of_val(&dw) as DWORD {
                (*This).set_bPruningOn((dw != 0) as u32);
            }
        }

        RegCloseKey(hKey);
    }

    ZeroMemory!(&mut devmode as *mut DEVMODEW as *mut core::ffi::c_void, core::mem::size_of::<DEVMODEW>());
    devmode.dmSize = core::mem::size_of::<DEVMODEW>() as u16;
    while EnumDisplaySettingsExW((*This).pDisplayDevice, i, &mut devmode, EDS_RAWMODE) != 0 {
        dwSize = devmode.dmSize as DWORD + devmode.dmDriverExtra as DWORD;
        let pDevMode: *mut DEVMODEW = LocalAlloc(LMEM_FIXED, (dwSize) as SIZE_T) as *mut DEVMODEW;
        let mut DevModesNew: *mut *mut DEVMODEW = core::ptr::null_mut();

        if !pDevMode.is_null() {
            CopyMemory!(
                pDevMode as *mut core::ffi::c_void,
                &devmode as *const DEVMODEW as *const core::ffi::c_void,
                dwSize as usize,
            );

            dwSize = ((*This).nDevModes + 1) * core::mem::size_of::<*mut DEVMODEW>() as DWORD;
            DevModesNew = LocalAlloc(LMEM_FIXED, (dwSize) as SIZE_T) as *mut *mut DEVMODEW;
            if !DevModesNew.is_null() {
                if !(*This).DevModes.is_null() {
                    CopyMemory!(
                        DevModesNew as *mut core::ffi::c_void,
                        (*This).DevModes as *const core::ffi::c_void,
                        ((*This).nDevModes * core::mem::size_of::<*mut DEVMODEW>() as DWORD) as usize,
                    );

                    LocalFree((*This).DevModes as HLOCAL);
                }

                (*This).DevModes = DevModesNew;
                *(*This).DevModes.add((*This).nDevModes as usize) = pDevMode;
                (*This).nDevModes += 1;
            } else {
                // DPRINT removed
                return E_OUTOFMEMORY;
            }
        } else {
            // DPRINT removed
            return E_OUTOFMEMORY;
        }

        devmode.dmDriverExtra = 0;
        i += 1;
    }

    ZeroMemory!(&mut devmode as *mut DEVMODEW as *mut core::ffi::c_void, core::mem::size_of::<DEVMODEW>());
    devmode.dmSize = core::mem::size_of::<DEVMODEW>() as u16;
    if EnumDisplaySettingsExW((*This).pDisplayDevice, ENUM_CURRENT_SETTINGS, &mut devmode, 0) != 0 {
        i = 0;
        while i < (*This).nDevModes {
            let CurMode: *mut DEVMODEW = *(*This).DevModes.add(i as usize);

            if CurMode.is_null() {
                i += 1;
                continue;
            }

            if (((*CurMode).dmFields & DM_PELSWIDTH) != 0 && devmode.dmPelsWidth == (*CurMode).dmPelsWidth)
                && (((*CurMode).dmFields & DM_PELSHEIGHT) != 0 && devmode.dmPelsHeight == (*CurMode).dmPelsHeight)
                && (((*CurMode).dmFields & DM_BITSPERPEL) != 0 && devmode.dmBitsPerPel == (*CurMode).dmBitsPerPel)
                && (((*CurMode).dmFields & DM_DISPLAYFREQUENCY) != 0 && devmode.dmDisplayFrequency == (*CurMode).dmDisplayFrequency)
            {
                (*This).lpOrigDevMode = CurMode;
                (*This).lpCurDevMode = CurMode;
                break;
            }
            i += 1;
        }
    }

    pCDevSettings_InitializeExtInterface(This);

    return S_OK;
}
unsafe extern "C" fn pCDevSettings_Free(This: *mut CDevSettings) {
    (*This).lpOrigDevMode = core::ptr::null_mut();
    (*This).lpCurDevMode = core::ptr::null_mut();
    while (*This).nDevModes != 0 {
        (*This).nDevModes -= 1;
        LocalFree(((*(*This).DevModes.offset((*This).nDevModes as isize))).cast());
    }
    LocalFree((*This).DevModes as HLOCAL);
    (*This).DevModes = core::ptr::null_mut();

    pCDevSettings_FreeString(core::ptr::addr_of_mut!((*This).pDisplayDevice));
    pCDevSettings_FreeString(core::ptr::addr_of_mut!((*This).pDisplayName));
    pCDevSettings_FreeString(core::ptr::addr_of_mut!((*This).pDisplayKey));
    pCDevSettings_FreeString(core::ptr::addr_of_mut!((*This).pDisplayId));
    pCDevSettings_FreeString(core::ptr::addr_of_mut!((*This).pMonitorName));
    pCDevSettings_FreeString(core::ptr::addr_of_mut!((*This).pMonitorDevice));
}
unsafe extern "system" fn CDevSettings_QueryInterface(
    iface: *mut IDataObject,
    riid: REFIID,
    ppvObject: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let This = impl_from_IDataObject(iface);

    *ppvObject = core::ptr::null_mut();

    if IsEqualGUID!(riid, &IID_IUnknown) != 0 || IsEqualGUID!(riid, &IID_IDataObject) != 0 {
        *ppvObject = impl_to_interface!(This, IDataObject) as *mut core::ffi::c_void;
        return S_OK;
    } else {
        // DPRINT removed
    }

    E_NOINTERFACE
}
unsafe extern "system" fn CDevSettings_AddRef(iface: *mut IDataObject) -> ULONG {
    let This = impl_from_IDataObject(iface);
    InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_) as PLONG) as ULONG
}
unsafe extern "system" fn CDevSettings_Release(iface: *mut IDataObject) -> ULONG {
    let refs: ULONG;
    let This: PCDevSettings = impl_from_IDataObject(iface);
    refs = InterlockedDecrement(core::ptr::addr_of_mut!((*This).ref_) as PLONG) as ULONG;
    if refs == 0 {
        pCDevSettings_Free(This);
    }

    return refs;
}
unsafe extern "system" fn CDevSettings_GetData(
    iface: *mut IDataObject,
    pformatetcIn: *mut FORMATETC,
    pmedium: *mut STGMEDIUM,
) -> HRESULT {
    static szEmpty: [u16; 1] = [0];
    let mut hr: HRESULT;
    let mut pszRet: PCWSTR = core::ptr::null();
    let mut pszBuf: PWSTR;
    let This: PCDevSettings = impl_from_IDataObject(iface);

    ZeroMemory!(
        pmedium as *mut _,
        core::mem::size_of::<STGMEDIUM>(),
    );

    hr = IDataObject_QueryGetData(iface, pformatetcIn);
    if hr >= 0 {
        if (*pformatetcIn).cfFormat == (*This).cfDisplayDevice {
            pszRet = (*This).pDisplayDevice;
            // DPRINT removed
        } else if (*pformatetcIn).cfFormat == (*This).cfDisplayName {
            pszRet = (*This).pDisplayName;
            // DPRINT removed
        } else if (*pformatetcIn).cfFormat == (*This).cfDisplayKey {
            pszRet = (*This).pDisplayKey;
            // DPRINT removed
        } else if (*pformatetcIn).cfFormat == (*This).cfDisplayId {
            pszRet = (*This).pDisplayId;
            // DPRINT removed
        } else if (*pformatetcIn).cfFormat == (*This).cfMonitorName {
            pszRet = (*This).pMonitorName;
            // DPRINT removed
        } else if (*pformatetcIn).cfFormat == (*This).cfMonitorDevice {
            pszRet = (*This).pMonitorDevice;
            // DPRINT removed
        } else if (*pformatetcIn).cfFormat == (*This).cfExtInterface {
            let pIface: PDESK_EXT_INTERFACE = GlobalAlloc(
                GPTR,
                (core::mem::size_of::<DESK_EXT_INTERFACE>()) as SIZE_T,
            ) as PDESK_EXT_INTERFACE;
            if !pIface.is_null() {
                CopyMemory!(
                    pIface as *mut _,
                    core::ptr::addr_of!((*This).ExtInterface) as *const _,
                    core::mem::size_of_val(&(*This).ExtInterface),
                );

                // DPRINT removed

                (*pmedium).tymed = TYMED_HGLOBAL;
                (*pmedium).__bindgen_anon_1.hGlobal = pIface as *mut _;

                return S_OK;
            } else {
                return E_OUTOFMEMORY;
            }
        } else if (*pformatetcIn).cfFormat == (*This).cfDisplayStateFlags {
            let pdw: PDWORD = GlobalAlloc(
                GPTR,
                (core::mem::size_of::<DWORD>()) as SIZE_T,
            ) as PDWORD;
            if !pdw.is_null() {
                *pdw = (*This).StateFlags;

                // DPRINT removed

                (*pmedium).tymed = TYMED_HGLOBAL;
                (*pmedium).__bindgen_anon_1.hGlobal = pdw as *mut _;

                return S_OK;
            } else {
                return E_OUTOFMEMORY;
            }
        } else if (*pformatetcIn).cfFormat == (*This).cfPruningMode {
            let pb: PBYTE = GlobalAlloc(
                GPTR,
                (core::mem::size_of::<u8>()) as SIZE_T,
            ) as PBYTE;
            if !pb.is_null() {
                *pb = if (*This).bModesPruned() != 0 && (*This).bPruningOn() != 0 {
                    1
                } else {
                    0
                };

                (*pmedium).tymed = TYMED_HGLOBAL;
                (*pmedium).__bindgen_anon_1.hGlobal = pb as *mut _;

                return S_OK;
            } else {
                return E_OUTOFMEMORY;
            }
        }

        if pszRet.is_null() {
            pszRet = szEmpty.as_ptr();
        }

        pszBuf = GlobalAlloc(
            GPTR,
            ((wcslen(pszRet) + 1) * core::mem::size_of::<u16>()) as SIZE_T,
        ) as PWSTR;
        if !pszBuf.is_null() {
            hr = StringCbCopyW(
                pszBuf as *mut _,
                (wcslen(pszRet) + 1) * core::mem::size_of::<u16>(),
                pszRet,
            );
            if hr < 0 {
                GlobalFree(pszBuf as *mut _);
                return hr;
            }

            (*pmedium).tymed = TYMED_HGLOBAL;
            (*pmedium).__bindgen_anon_1.hGlobal = pszBuf as *mut _;

            hr = S_OK;
        } else {
            hr = E_OUTOFMEMORY;
        }
    }

    hr
}
unsafe extern "system" fn CDevSettings_GetDataHere(
    iface: *mut IDataObject,
    pformatetc: *mut FORMATETC,
    pmedium: *mut STGMEDIUM,
) -> HRESULT {
    ZeroMemory!(
        pformatetc as *mut _,
        core::mem::size_of::<FORMATETC>(),
    );
    E_NOTIMPL
}
unsafe extern "system" fn CDevSettings_QueryGetData(
    iface: *mut IDataObject,
    pformatetc: *mut FORMATETC,
) -> HRESULT {
    #[cfg(debug_assertions)]
    let mut szFormatName: [TCHAR; 255] = [0; 255];

    let This = impl_from_IDataObject(iface);

    if (*pformatetc).dwAspect != DVASPECT_CONTENT {
        return DV_E_DVASPECT;
    }

    if (*pformatetc).lindex != -1 {
        return DV_E_LINDEX;
    }

    if ((*pformatetc).tymed & TYMED_HGLOBAL) == 0 {
        return DV_E_TYMED;
    }

    if (*pformatetc).cfFormat == (*This).cfExtInterface
        || (*pformatetc).cfFormat == (*This).cfDisplayDevice
        || (*pformatetc).cfFormat == (*This).cfDisplayName
        || (*pformatetc).cfFormat == (*This).cfDisplayId
        || (*pformatetc).cfFormat == (*This).cfDisplayKey
        || (*pformatetc).cfFormat == (*This).cfDisplayStateFlags
        || (*pformatetc).cfFormat == (*This).cfMonitorDevice
        || (*pformatetc).cfFormat == (*This).cfMonitorName
        || (*pformatetc).cfFormat == (*This).cfPruningMode
    {
        return S_OK;
    }
    #[cfg(debug_assertions)]
    {
        if GetClipboardFormatName(
            (*pformatetc).cfFormat as UINT,
            szFormatName.as_mut_ptr(),
            (core::mem::size_of_val(&szFormatName) / core::mem::size_of::<TCHAR>()) as c_int,
        ) != 0
        {
            // DPRINT removed
        } else {
            // DPRINT removed
        }
    }

    DV_E_FORMATETC
}
unsafe extern "system" fn CDevSettings_GetCanonicalFormatEtc(
    iface: *mut IDataObject,
    pformatectIn: *mut FORMATETC,
    pformatetcOut: *mut FORMATETC,
) -> HRESULT {
    let mut hr: HRESULT;

    // DPRINT removed

    hr = IDataObject_QueryGetData(iface, pformatectIn);
    if hr >= 0 {
        CopyMemory!(
            pformatetcOut as *mut _,
            pformatectIn as *const _,
            core::mem::size_of::<FORMATETC>(),
        );

        if (*pformatectIn).ptd == core::ptr::null_mut() {
            hr = DATA_S_SAMEFORMATETC;
        } else {
            (*pformatetcOut).ptd = core::ptr::null_mut();
            hr = S_OK;
        }
    } else {
        ZeroMemory!(
            pformatetcOut as *mut _,
            core::mem::size_of::<FORMATETC>(),
        );
    }

    hr
}
unsafe extern "system" fn CDevSettings_SetData(
    iface: *mut IDataObject,
    pformatetc: *mut FORMATETC,
    pmedium: *mut STGMEDIUM,
    fRelease: BOOL,
) -> HRESULT {
    // DPRINT1!("CDevSettings::SetData UNIMPLEMENTED\n");
    E_NOTIMPL
}
unsafe extern "C" fn pCDevSettings_FillFormatEtc(
    pFormatEtc: *mut FORMATETC,
    cf: CLIPFORMAT,
) {
    (*pFormatEtc).cfFormat = cf;
    (*pFormatEtc).ptd = core::ptr::null_mut();
    (*pFormatEtc).dwAspect = DVASPECT_CONTENT;
    (*pFormatEtc).lindex = -(1 as i32);
    (*pFormatEtc).tymed = TYMED_HGLOBAL;
}
unsafe extern "system" fn CDevSettings_EnumFormatEtc(
    iface: *mut IDataObject,
    dwDirection: DWORD,
    ppenumFormatEtc: *mut *mut IEnumFORMATETC,
) -> HRESULT {
    let mut hr: HRESULT;
    let mut fetc: [FORMATETC; 9] = core::mem::zeroed();
    let This: PCDevSettings = impl_from_IDataObject(iface);

    *ppenumFormatEtc = core::ptr::null_mut();

    if dwDirection == DATADIR_GET {
        pCDevSettings_FillFormatEtc(
            &mut fetc[0],
            (*This).cfExtInterface,
        );
        pCDevSettings_FillFormatEtc(
            &mut fetc[1],
            (*This).cfDisplayDevice,
        );
        pCDevSettings_FillFormatEtc(
            &mut fetc[2],
            (*This).cfDisplayName,
        );
        pCDevSettings_FillFormatEtc(
            &mut fetc[3],
            (*This).cfDisplayId,
        );
        pCDevSettings_FillFormatEtc(
            &mut fetc[4],
            (*This).cfDisplayKey,
        );
        pCDevSettings_FillFormatEtc(
            &mut fetc[5],
            (*This).cfDisplayStateFlags,
        );
        pCDevSettings_FillFormatEtc(
            &mut fetc[6],
            (*This).cfMonitorName,
        );
        pCDevSettings_FillFormatEtc(
            &mut fetc[7],
            (*This).cfMonitorDevice,
        );
        pCDevSettings_FillFormatEtc(
            &mut fetc[8],
            (*This).cfPruningMode,
        );

        hr = SHCreateStdEnumFmtEtc(
            (core::mem::size_of_val(&fetc) / core::mem::size_of_val(&fetc[0])) as UINT,
            fetc.as_mut_ptr(),
            ppenumFormatEtc,
        );
    } else {
        hr = E_NOTIMPL;
    }

    hr
}
unsafe extern "system" fn CDevSettings_DAdvise(
    iface: *mut IDataObject,
    pformatetc: *mut FORMATETC,
    advf: DWORD,
    pAdvSink: *mut IAdviseSink,
    pdwConnection: *mut DWORD,
) -> HRESULT {
    *pdwConnection = 0;
    OLE_E_ADVISENOTSUPPORTED
}
unsafe extern "system" fn CDevSettings_DUnadvise(
    iface: *mut IDataObject,
    dwConnection: DWORD,
) -> HRESULT {
    OLE_E_ADVISENOTSUPPORTED
}
unsafe extern "system" fn CDevSettings_EnumDAdvise(
    iface: *mut IDataObject,
    ppenumAdvise: *mut *mut IEnumSTATDATA,
) -> HRESULT {
    *ppenumAdvise = core::ptr::null_mut();
    return OLE_E_ADVISENOTSUPPORTED;
}

// devsett.c:1029 `static const struct IDataObjectVtbl vtblIDataObject = { ... }` — the vtable
// itself, which the translation dropped: every slot function came across but the table that
// wires them into a COM object did not, so `CreateDevSettings` referred to a name nothing
// defined. Written as a designated-field initialiser rather than the C's positional list, so
// the slot ORDER is the island's (bindgen read objidl.h) and cannot be miscounted by hand --
// a positional vtable off by one is a silent call to the wrong method.
//
// `static` and not `const`: the C's is one object at one address, and `This->lpIDataObjectVtbl`
// stores its ADDRESS. A `const` would be inlined into a fresh temporary per use.
static vtblIDataObject: IDataObjectVtbl = IDataObjectVtbl {
    QueryInterface: Some(CDevSettings_QueryInterface),
    AddRef: Some(CDevSettings_AddRef),
    Release: Some(CDevSettings_Release),
    GetData: Some(CDevSettings_GetData),
    GetDataHere: Some(CDevSettings_GetDataHere),
    QueryGetData: Some(CDevSettings_QueryGetData),
    GetCanonicalFormatEtc: Some(CDevSettings_GetCanonicalFormatEtc),
    SetData: Some(CDevSettings_SetData),
    EnumFormatEtc: Some(CDevSettings_EnumFormatEtc),
    DAdvise: Some(CDevSettings_DAdvise),
    DUnadvise: Some(CDevSettings_DUnadvise),
    EnumDAdvise: Some(CDevSettings_EnumDAdvise),
};

#[no_mangle]
pub unsafe extern "C" fn CreateDevSettings(
    DisplayDeviceInfo: PDISPLAY_DEVICE_ENTRY,
) -> *mut IDataObject {
    let This: PCDevSettings;

    This = (HeapAlloc(
        GetProcessHeap(),
        0,
        (core::mem::size_of::<CDevSettings>()) as SIZE_T,
    )).cast();
    if !This.is_null() {
        (*This).lpIDataObjectVtbl = &vtblIDataObject;
        (*This).ref_ = 1;

        if pCDevSettings_Initialize(This, DisplayDeviceInfo) >= 0 {
            return impl_to_interface!(This, IDataObject);
        }

        CDevSettings_Release(impl_to_interface!(This, IDataObject));
    }

    core::ptr::null_mut()
}
#[no_mangle]
pub unsafe extern "system" fn DisplaySaveSettings(
    pContext: PVOID,
    hwndPropSheet: HWND,
) -> LONG {
    let This: PCDevSettings = impl_from_IDataObject(pContext as *mut IDataObject);
    let mut rc: LONG = (DISP_CHANGE_SUCCESSFUL) as i32;

    if (*This).lpCurDevMode != (*This).lpOrigDevMode {
        let mut seOrig: SETTINGS_ENTRY = core::mem::zeroed();
        let mut seCur: SETTINGS_ENTRY = core::mem::zeroed();
        let Ret: BOOL;

        seOrig.dmPelsWidth = (*(*This).lpOrigDevMode).dmPelsWidth;
        seOrig.dmPelsHeight = (*(*This).lpOrigDevMode).dmPelsHeight;
        seOrig.dmBitsPerPel = (*(*This).lpOrigDevMode).dmBitsPerPel;
        seOrig.dmDisplayFrequency = (*(*This).lpOrigDevMode).dmDisplayFrequency;

        seCur.dmPelsWidth = (*(*This).lpCurDevMode).dmPelsWidth;
        seCur.dmPelsHeight = (*(*This).lpCurDevMode).dmPelsHeight;
        seCur.dmBitsPerPel = (*(*This).lpCurDevMode).dmBitsPerPel;
        seCur.dmDisplayFrequency = (*(*This).lpCurDevMode).dmDisplayFrequency;

        Ret = (SwitchDisplayMode(
            hwndPropSheet,
            (*This).pDisplayDevice,
            &mut seOrig,
            &mut seCur,
            &mut rc,
        )) as i32;

        if rc == (DISP_CHANGE_SUCCESSFUL) as i32 {
            if Ret != 0 {
                (*This).lpOrigDevMode = (*This).lpCurDevMode;
            } else {
                (*This).lpCurDevMode = (*This).lpOrigDevMode;
            }
        }
    }

    rc
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS Display Control Panel
//  * FILE:            dll/cpl/desk/devsett.c
//  * PURPOSE:         ReactOS Display Control Panel Shell Extension Support
//  */
//
// #include "desk.h"
//
// #include <cfgmgr32.h>
//
// #define NDEBUG
// #include <debug.h>
//
// #define DEBUG_DEVSETTINGS
//
// typedef struct _CDevSettings
// {
//     const struct IDataObjectVtbl *lpIDataObjectVtbl;
//     DWORD ref;
//
//     CLIPFORMAT cfExtInterface;      /* "Desk.cpl extension interface" */
//     CLIPFORMAT cfDisplayDevice;     /* "Display Device" */
//     CLIPFORMAT cfDisplayName;       /* "Display Name" */
//     CLIPFORMAT cfDisplayId;         /* "Display ID" */
//     CLIPFORMAT cfMonitorName;       /* "Monitor Name" */
//     CLIPFORMAT cfMonitorDevice;     /* "Monitor Device" */
//     CLIPFORMAT cfDisplayKey;        /* "Display Key" */
//     CLIPFORMAT cfDisplayStateFlags; /* "Display State Flags" */
//     CLIPFORMAT cfPruningMode;       /* "Pruning Mode" */
//
//     PWSTR pDisplayDevice;
//     PWSTR pDisplayName;
//     PWSTR pDisplayKey;
//     PWSTR pDisplayId;
//     PWSTR pMonitorName;
//     PWSTR pMonitorDevice;
//
//     DESK_EXT_INTERFACE ExtInterface;
//
//     PDEVMODEW lpOrigDevMode;
//     PDEVMODEW lpCurDevMode;
//     PDEVMODEW * DevModes;
//     DWORD nDevModes;
//     DWORD StateFlags;
//
//     union
//     {
//         DWORD Flags;
//         struct
//         {
//             DWORD bModesPruned : 1;
//             DWORD bKeyIsReadOnly : 1;
//             DWORD bPruningOn : 1;
//         };
//     };
// } CDevSettings, *PCDevSettings;
//
// #define impl_to_interface(impl,iface) (struct iface *)(&(impl)->lp##iface##Vtbl)
//
// static __inline PCDevSettings
// impl_from_IDataObject(struct IDataObject *iface)
// {
//     return (PCDevSettings)((ULONG_PTR)iface - FIELD_OFFSET(CDevSettings,
//                                                            lpIDataObjectVtbl));
// }
//
// static __inline VOID
// pCDevSettings_FreeString(PWCHAR *psz)
// {
//     if (*psz != NULL)
//     {
//         LocalFree((HLOCAL)*psz);
//         *psz = NULL;
//     }
// }
//
// static PWSTR
// pCDevSettings_AllocAndCopyString(const TCHAR *pszSrc)
// {
//     SIZE_T c;
//     PWSTR str;
//
//     c = _tcslen(pszSrc) + 1;
//     str = (PWSTR)LocalAlloc(LMEM_FIXED,
//                             c * sizeof(WCHAR));
//     if (str != NULL)
//     {
// #ifdef UNICODE
//         StringCbCopyW(str, c * sizeof(WCHAR),
//                pszSrc);
// #else
//         MultiByteToWideChar(CP_ACP,
//                             0,
//                             pszSrc,
//                             -1,
//                             str,
//                             c);
// #endif
//     }
//
//     return str;
// }
//
// static PWSTR
// pCDevSettings_GetMonitorName(const WCHAR *pszDisplayDevice)
// {
//     DISPLAY_DEVICEW dd, dd2;
//     PWSTR str = NULL;
//
//     dd.cb = sizeof(dd);
//     if (EnumDisplayDevicesW(pszDisplayDevice,
//                             0,
//                             &dd,
//                             0))
//     {
//         dd2.cb = sizeof(dd2);
//         if (EnumDisplayDevicesW(pszDisplayDevice,
//                                 1,
//                                 &dd2,
//                                 0))
//         {
//             /* There's more than one monitor connected... */
//             LoadStringW(hApplet,
//                         IDS_MULTIPLEMONITORS,
//                         dd.DeviceString,
//                         sizeof(dd.DeviceString) / sizeof(dd.DeviceString[0]));
//         }
//     }
//     else
//     {
//         /* We can't enumerate a monitor, make sure this fact is reported
//            to the user! */
//         LoadStringW(hApplet,
//                     IDS_UNKNOWNMONITOR,
//                     dd.DeviceString,
//                     sizeof(dd.DeviceString) / sizeof(dd.DeviceString[0]));
//     }
//
//     str = LocalAlloc(LMEM_FIXED,
//                      (wcslen(dd.DeviceString) + 1) * sizeof(WCHAR));
//     if (str != NULL)
//     {
//         wcscpy(str,
//                dd.DeviceString);
//     }
//
//     return str;
// }
//
// static PWSTR
// pCDevSettings_GetMonitorDevice(const WCHAR *pszDisplayDevice)
// {
//     DISPLAY_DEVICEW dd;
//     PWSTR str = NULL;
//
//     dd.cb = sizeof(dd);
//     if (EnumDisplayDevicesW(pszDisplayDevice,
//                             0,
//                             &dd,
//                             0))
//     {
//         str = LocalAlloc(LMEM_FIXED,
//                          (wcslen(dd.DeviceName) + 1) * sizeof(WCHAR));
//         if (str != NULL)
//         {
//             wcscpy(str,
//                    dd.DeviceName);
//         }
//     }
//
//     return str;
// }
//
// /**
//  * @brief
//  * Converts a Hardware ID (DeviceID from EnumDisplayDevices)
//  * to an unique Device Instance ID.
//  *
//  * @param[in] pszDevice
//  * A pointer to a null-terminated Unicode string
//  * containing a Hardware ID.
//  * e.g. "PCI\VEN_80EE&DEV_BEEF&SUBSYS_00000000&REV_00"
//  *
//  * @return
//  * A pointer to a null-terminated Unicode string
//  * containing an unique Device Instance ID
//  * or NULL in case of error.
//  * e.g. "PCI\VEN_80EE&DEV_BEEF&SUBSYS_00000000&REV_00\3&267A616A&0&10"
//  *
//  * @remarks
//  * The caller must free the returned string with LocalFree.
//  */
// static PWSTR
// pCDevSettings_GetDeviceInstanceId(const WCHAR *pszDevice)
// {
//     HDEVINFO DevInfo;
//     SP_DEVINFO_DATA InfoData;
//     ULONG BufLen;
//     LPWSTR lpDevInstId = NULL;
//
//     DPRINT("CDevSettings::GetDeviceInstanceId(%ws)!\n", pszDevice);
//
//     DevInfo = SetupDiGetClassDevsW(NULL, pszDevice, NULL, DIGCF_ALLCLASSES | DIGCF_PRESENT);
//     if (DevInfo == INVALID_HANDLE_VALUE)
//     {
//         DPRINT1("SetupDiGetClassDevsW(\"%ws\") failed: %d\n", pszDevice, GetLastError());
//         return NULL;
//     }
//
//     ZeroMemory(&InfoData, sizeof(InfoData));
//     InfoData.cbSize = sizeof(InfoData);
//
//     /* Try to enumerate the first matching device */
//     if (!SetupDiEnumDeviceInfo(DevInfo, 0, &InfoData))
//     {
//         DPRINT1("SetupDiEnumDeviceInfo failed: %d\n", GetLastError());
//         return NULL;
//     }
//
//     if (SetupDiGetDeviceInstanceId(DevInfo, &InfoData, NULL, 0, &BufLen) ||
//         GetLastError() != ERROR_INSUFFICIENT_BUFFER)
//     {
//         DPRINT1("SetupDiGetDeviceInstanceId failed: %d\n", GetLastError());
//         return NULL;
//     }
//
//     lpDevInstId = LocalAlloc(LMEM_FIXED,
//                              (BufLen + 1) * sizeof(WCHAR));
//
//     if (lpDevInstId == NULL)
//     {
//         DPRINT1("LocalAlloc failed\n");
//         return NULL;
//     }
//
//     if (!SetupDiGetDeviceInstanceId(DevInfo, &InfoData, lpDevInstId, BufLen, NULL))
//     {
//         DPRINT1("SetupDiGetDeviceInstanceId failed: %d\n", GetLastError());
//         LocalFree((HLOCAL)lpDevInstId);
//         lpDevInstId = NULL;
//     }
//     else
//     {
//         DPRINT("instance id: %ws\n", lpDevInstId);
//     }
//
//     return lpDevInstId;
// }
//
//
// static HKEY
// pCDevSettings_OpenDeviceKey(PCDevSettings This,
//                             BOOL ReadOnly)
// {
//     static const WCHAR szRegPrefix[] = L"\\Registry\\Machine\\";
//     PWSTR lpRegKey;
//     REGSAM Access = KEY_READ;
//     HKEY hKey;
//
//     lpRegKey = This->pDisplayKey;
//     if (lpRegKey != NULL)
//     {
//         if (wcslen(lpRegKey) >= wcslen(szRegPrefix) &&
//             !_wcsnicmp(lpRegKey,
//                       szRegPrefix,
//                       wcslen(szRegPrefix)))
//         {
//             lpRegKey += wcslen(szRegPrefix);
//         }
//
//         if (!ReadOnly)
//             Access |= KEY_WRITE;
//
//         if (RegOpenKeyEx(HKEY_LOCAL_MACHINE,
//                          lpRegKey,
//                          0,
//                          Access,
//                          &hKey) == ERROR_SUCCESS)
//         {
//             return hKey;
//         }
//     }
//
//     return NULL;
// }
//
// PDEVMODEW __stdcall
// CDevSettings_EnumAllModes(PVOID Context,
//                           DWORD Index)
// {
//     PCDevSettings This = impl_from_IDataObject((IDataObject *)Context);
//     DWORD i, idx = 0;
//
//     DPRINT1("CDevSettings::EnumAllModes(%u)\n", Index);
//
//     if (!This->DevModes)
//         return NULL;
//
//     for (i = 0; i < This->nDevModes; i++)
//     {
//         /* FIXME: Add more sanity checks */
//         if (!This->DevModes[i])
//             continue;
//
//         if (idx == Index)
//             return This->DevModes[i];
//
//         idx++;
//     }
//
//     return NULL;
// }
//
// PDEVMODEW __stdcall
// CDevSettings_GetCurrentMode(PVOID Context)
// {
//     PCDevSettings This = impl_from_IDataObject((IDataObject *)Context);
//
//     DPRINT1("CDevSettings::GetCurrentMode\n");
//     return This->lpCurDevMode;
// }
//
// BOOL __stdcall
// CDevSettings_SetCurrentMode(PVOID Context,
//                             DEVMODEW *pDevMode)
// {
//     PCDevSettings This = impl_from_IDataObject((IDataObject *)Context);
//     DWORD i;
//
//     DPRINT1("CDevSettings::SetCurrentMode(0x%p)\n", pDevMode);
//
//     if (!This->DevModes)
//         return FALSE;
//
//     for (i = 0; i < This->nDevModes; i++)
//     {
//         /* FIXME: Add more sanity checks */
//         if (!This->DevModes[i])
//             continue;
//
//         if (This->DevModes[i] == pDevMode)
//         {
//             This->lpCurDevMode = pDevMode;
//             return TRUE;
//         }
//     }
//
//     return FALSE;
// }
//
// VOID __stdcall
// CDevSettings_GetPruningMode(PVOID Context,
//                             PBOOL pbModesPruned,
//                             PBOOL pbKeyIsReadOnly,
//                             PBOOL pbPruningOn)
// {
//     PCDevSettings This = impl_from_IDataObject((IDataObject *)Context);
//
//     DPRINT1("CDevSettings::GetPruningMode(%p,%p,%p)\n", pbModesPruned, pbKeyIsReadOnly, pbPruningOn);
//
//     *pbModesPruned = This->bModesPruned;
//     *pbKeyIsReadOnly = This->bKeyIsReadOnly;
//     *pbPruningOn = This->bPruningOn;
// }
//
// VOID __stdcall
// CDevSettings_SetPruningMode(PVOID Context,
//                             BOOL PruningOn)
// {
//     HKEY hKey;
//     DWORD dwValue;
//     PCDevSettings This = impl_from_IDataObject((IDataObject *)Context);
//
//     DPRINT1("CDevSettings::SetPruningMode(%d)\n", PruningOn);
//
//     if (This->bModesPruned && !This->bKeyIsReadOnly &&
//         PruningOn != This->bPruningOn)
//     {
//         This->bPruningOn = (PruningOn != FALSE);
//
//         hKey = pCDevSettings_OpenDeviceKey(This,
//                                            FALSE);
//         if (hKey != NULL)
//         {
//             dwValue = (DWORD)This->bPruningOn;
//
//             RegSetValueEx(hKey,
//                           TEXT("PruningMode"),
//                           0,
//                           REG_DWORD,
//                           (const BYTE *)&dwValue,
//                           sizeof(dwValue));
//
//             RegCloseKey(hKey);
//         }
//     }
// }
//
// static VOID
// pCDevSettings_ReadHardwareInfo(HKEY hKey,
//                                LPCTSTR lpValueName,
//                                LPWSTR lpBuffer)
// {
//     DWORD type = REG_BINARY;
//     DWORD size = 128 * sizeof(WCHAR);
//     RegQueryValueEx(hKey,
//                     lpValueName,
//                     NULL,
//                     &type,
//                     (PBYTE)lpBuffer,
//                     &size);
// }
//
// static VOID
// pCDevSettings_InitializeExtInterface(PCDevSettings This)
// {
//     PDESK_EXT_INTERFACE Interface = &This->ExtInterface;
//     HKEY hKeyDev;
//
//     ZeroMemory(Interface,
//                sizeof(*Interface));
//     Interface->cbSize = sizeof(*Interface);
//
//     /* Initialize the callback table */
//     Interface->Context = impl_to_interface(This, IDataObject);
//     Interface->EnumAllModes = CDevSettings_EnumAllModes;
//     Interface->SetCurrentMode = CDevSettings_SetCurrentMode;
//     Interface->GetCurrentMode = CDevSettings_GetCurrentMode;
//     Interface->SetPruningMode = CDevSettings_SetPruningMode;
//     Interface->GetPruningMode = CDevSettings_GetPruningMode;
//
//     /* Read the HardwareInformation.* values from the registry key */
//     hKeyDev = pCDevSettings_OpenDeviceKey(This,
//                                           TRUE);
//     if (hKeyDev != NULL)
//     {
//         DWORD dwType, dwMemSize = 0;
//         DWORD dwSize = sizeof(dwMemSize);
//
//         if (RegQueryValueEx(hKeyDev,
//                             TEXT("HardwareInformation.MemorySize"),
//                             NULL,
//                             &dwType,
//                             (PBYTE)&dwMemSize,
//                             &dwSize) == ERROR_SUCCESS &&
//             (dwType == REG_BINARY || dwType == REG_DWORD) &&
//             dwSize == sizeof(dwMemSize))
//         {
//             dwMemSize /= 1024;
//
//             if (dwMemSize > 1024)
//             {
//                 dwMemSize /= 1024;
//                 if (dwMemSize > 1024)
//                 {
//                     wsprintf(Interface->MemorySize,
//                              _T("%u GB"),
//                              dwMemSize / 1024);
//                 }
//                 else
//                 {
//                     wsprintf(Interface->MemorySize,
//                              _T("%u MB"),
//                              dwMemSize);
//                 }
//             }
//             else
//             {
//                 wsprintf(Interface->MemorySize,
//                          _T("%u KB"),
//                          dwMemSize);
//             }
//         }
//
//         pCDevSettings_ReadHardwareInfo(hKeyDev,
//                                        TEXT("HardwareInformation.ChipType"),
//                                        Interface->ChipType);
//         pCDevSettings_ReadHardwareInfo(hKeyDev,
//                                        TEXT("HardwareInformation.DacType"),
//                                        Interface->DacType);
//         pCDevSettings_ReadHardwareInfo(hKeyDev,
//                                        TEXT("HardwareInformation.AdapterString"),
//                                        Interface->AdapterString);
//         pCDevSettings_ReadHardwareInfo(hKeyDev,
//                                        TEXT("HardwareInformation.BiosString"),
//                                        Interface->BiosString);
//         RegCloseKey(hKeyDev);
//     }
// }
//
// static HRESULT
// pCDevSettings_Initialize(PCDevSettings This,
//                          PDISPLAY_DEVICE_ENTRY DisplayDeviceInfo)
// {
//     HKEY hKey;
//     DWORD i = 0, dwSize;
//     DEVMODEW devmode;
//
//     This->lpOrigDevMode = NULL;
//     This->lpCurDevMode = NULL;
//     This->DevModes = NULL;
//     This->nDevModes = 0;
//     This->Flags = 0;
//     This->StateFlags = DisplayDeviceInfo->DeviceStateFlags;
//     DPRINT1("This->StateFlags: %x\n", This->StateFlags);
//
//     /* Register clipboard formats */
//     This->cfExtInterface = RegisterClipboardFormat(DESK_EXT_EXTINTERFACE);
//     This->cfDisplayDevice = RegisterClipboardFormat(DESK_EXT_DISPLAYDEVICE);
//     This->cfDisplayName = RegisterClipboardFormat(DESK_EXT_DISPLAYNAME);
//     This->cfDisplayId = RegisterClipboardFormat(DESK_EXT_DISPLAYID);
//     This->cfDisplayKey = RegisterClipboardFormat(DESK_EXT_DISPLAYKEY);
//     This->cfDisplayStateFlags = RegisterClipboardFormat(DESK_EXT_DISPLAYSTATEFLAGS);
//     This->cfMonitorName = RegisterClipboardFormat(DESK_EXT_MONITORNAME);
//     This->cfMonitorDevice = RegisterClipboardFormat(DESK_EXT_MONITORDEVICE);
//     This->cfPruningMode = RegisterClipboardFormat(DESK_EXT_PRUNINGMODE);
//
//     /* Copy the device name */
//     This->pDisplayDevice = pCDevSettings_AllocAndCopyString(DisplayDeviceInfo->DeviceName);
//     DPRINT1("This->pDisplayDevice: %ws\n", This->pDisplayDevice);
//     This->pDisplayName = pCDevSettings_AllocAndCopyString(DisplayDeviceInfo->DeviceDescription);
//     DPRINT1("This->pDisplayName: %ws\n", This->pDisplayName);
//     This->pDisplayKey = pCDevSettings_AllocAndCopyString(DisplayDeviceInfo->DeviceKey);
//     DPRINT1("This->pDisplayKey: %ws\n", This->pDisplayKey);
//     This->pDisplayId = pCDevSettings_GetDeviceInstanceId(DisplayDeviceInfo->DeviceID);
//     DPRINT1("This->pDisplayId: %ws\n", This->pDisplayId);
//     This->pMonitorName = pCDevSettings_GetMonitorName(This->pDisplayDevice);
//     DPRINT1("This->pMonitorName: %ws\n", This->pMonitorName);
//     This->pMonitorDevice = pCDevSettings_GetMonitorDevice(This->pDisplayDevice);
//     DPRINT1("This->pMonitorDevice: %ws\n", This->pMonitorDevice);
//
//     /* Check pruning mode */
//     This->bModesPruned = ((DisplayDeviceInfo->DeviceStateFlags & DISPLAY_DEVICE_MODESPRUNED) != 0);
//     hKey = pCDevSettings_OpenDeviceKey(This,
//                                        FALSE);
//     if (hKey == NULL)
//     {
//         hKey = pCDevSettings_OpenDeviceKey(This,
//                                            FALSE);
//         This->bKeyIsReadOnly = TRUE;
//     }
//     if (hKey != NULL)
//     {
//         DWORD dw = 0;
//         DWORD dwType;
//
//         dwSize = sizeof(dw);
//         if (RegQueryValueEx(hKey,
//                             TEXT("PruningMode"),
//                             NULL,
//                             &dwType,
//                             (PBYTE)&dw,
//                             &dwSize) == ERROR_SUCCESS)
//         {
//             if (dwType == REG_DWORD && dwSize == sizeof(dw))
//                 This->bPruningOn = (dw != 0);
//         }
//
//         RegCloseKey(hKey);
//     }
//
//     /* Initialize display modes */
//     ZeroMemory(&devmode, sizeof(devmode));
//     devmode.dmSize = (WORD)sizeof(devmode);
//     while (EnumDisplaySettingsExW(This->pDisplayDevice, i, &devmode, EDS_RAWMODE))
//     {
//         dwSize = devmode.dmSize + devmode.dmDriverExtra;
//         PDEVMODEW pDevMode = LocalAlloc(LMEM_FIXED, dwSize);
//         PDEVMODEW * DevModesNew = NULL;
//
//         if (pDevMode)
//         {
//             CopyMemory(pDevMode,
//                        &devmode,
//                        dwSize);
//
//             dwSize = (This->nDevModes + 1) * sizeof(pDevMode);
//             DevModesNew = LocalAlloc(LMEM_FIXED, dwSize);
//             if (DevModesNew)
//             {
//                 if (This->DevModes)
//                 {
//                     CopyMemory(DevModesNew,
//                                This->DevModes,
//                                This->nDevModes * sizeof(pDevMode));
//
//                     LocalFree(This->DevModes);
//                 }
//
//                 This->DevModes = DevModesNew;
//                 This->DevModes[This->nDevModes++] = pDevMode;
//             }
//             else
//             {
//                 DPRINT1("LocalAlloc failed to allocate %d bytes\n", dwSize);
//                 return E_OUTOFMEMORY;
//             }
//         }
//         else
//         {
//             DPRINT1("LocalAlloc failed to allocate %d bytes\n", dwSize);
//             return E_OUTOFMEMORY;
//         }
//
//         devmode.dmDriverExtra = 0;
//         i++;
//     }
//
//     /* FIXME: Detect duplicated modes and mark them.
//      * Enumeration functions should check these marks
//      * and skip corresponding array entries. */
//
//     /* Get current display mode */
//     ZeroMemory(&devmode, sizeof(devmode));
//     devmode.dmSize = (WORD)sizeof(devmode);
//     if (EnumDisplaySettingsExW(This->pDisplayDevice, ENUM_CURRENT_SETTINGS, &devmode, 0))
//     {
//         for (i = 0; i < This->nDevModes; i++)
//         {
//             PDEVMODEW CurMode = This->DevModes[i];
//
//             if (!CurMode)
//                 continue;
//
//             if (((CurMode->dmFields & DM_PELSWIDTH) && devmode.dmPelsWidth == CurMode->dmPelsWidth) &&
//                 ((CurMode->dmFields & DM_PELSHEIGHT) && devmode.dmPelsHeight == CurMode->dmPelsHeight) &&
//                 ((CurMode->dmFields & DM_BITSPERPEL) && devmode.dmBitsPerPel == CurMode->dmBitsPerPel) &&
//                 ((CurMode->dmFields & DM_DISPLAYFREQUENCY) && devmode.dmDisplayFrequency == CurMode->dmDisplayFrequency))
//             {
//                 This->lpOrigDevMode = This->lpCurDevMode = CurMode;
//                 break;
//             }
//         }
//     }
//
//     /* Initialize the shell extension interface */
//     pCDevSettings_InitializeExtInterface(This);
//
//     return S_OK;
// }
//
// static VOID
// pCDevSettings_Free(PCDevSettings This)
// {
//     This->lpOrigDevMode = NULL;
//     This->lpCurDevMode = NULL;
//     while (This->nDevModes)
//     {
//         LocalFree(This->DevModes[--This->nDevModes]);
//     }
//     LocalFree(This->DevModes);
//     This->DevModes = NULL;
//
//     pCDevSettings_FreeString(&This->pDisplayDevice);
//     pCDevSettings_FreeString(&This->pDisplayName);
//     pCDevSettings_FreeString(&This->pDisplayKey);
//     pCDevSettings_FreeString(&This->pDisplayId);
//     pCDevSettings_FreeString(&This->pMonitorName);
//     pCDevSettings_FreeString(&This->pMonitorDevice);
// }
//
// static HRESULT STDMETHODCALLTYPE
// CDevSettings_QueryInterface(IDataObject* iface,
//                             REFIID riid,
//                             void** ppvObject)
// {
//     PCDevSettings This = impl_from_IDataObject(iface);
//
//     *ppvObject = NULL;
//
//     if (IsEqualGUID(riid,
//                     &IID_IUnknown) ||
//         IsEqualGUID(riid,
//                     &IID_IDataObject))
//     {
//         *ppvObject = (PVOID)impl_to_interface(This, IDataObject);
//         return S_OK;
//     }
//     else
//     {
//         DPRINT1("CDevSettings::QueryInterface: Queried unknown interface\n");
//     }
//
//     return E_NOINTERFACE;
// }
//
// static ULONG STDMETHODCALLTYPE
// CDevSettings_AddRef(IDataObject* iface)
// {
//     PCDevSettings This = impl_from_IDataObject(iface);
//     return (ULONG)InterlockedIncrement((PLONG)&This->ref);
// }
//
// static ULONG STDMETHODCALLTYPE
// CDevSettings_Release(IDataObject* iface)
// {
//     ULONG refs;
//     PCDevSettings This = impl_from_IDataObject(iface);
//     refs = (ULONG)InterlockedDecrement((PLONG)&This->ref);
//     if (refs == 0)
//         pCDevSettings_Free(This);
//
//     return refs;
// }
//
// static HRESULT STDMETHODCALLTYPE
// CDevSettings_GetData(IDataObject* iface,
//                      FORMATETC* pformatetcIn,
//                      STGMEDIUM* pmedium)
// {
//     static const WCHAR szEmpty[] = {0};
//     HRESULT hr;
//     PCWSTR pszRet = NULL;
//     PWSTR pszBuf;
//     PCDevSettings This = impl_from_IDataObject(iface);
//
//     ZeroMemory(pmedium,
//                sizeof(STGMEDIUM));
//
//     hr = IDataObject_QueryGetData(iface,
//                                   pformatetcIn);
//     if (SUCCEEDED(hr))
//     {
//         /* Return the requested data back to the shell extension */
//
//         if (pformatetcIn->cfFormat == This->cfDisplayDevice)
//         {
//             pszRet = This->pDisplayDevice;
//             DPRINT1("CDevSettings::GetData returns display device %ws\n", pszRet);
//         }
//         else if (pformatetcIn->cfFormat == This->cfDisplayName)
//         {
//             pszRet = This->pDisplayName;
//             DPRINT1("CDevSettings::GetData returns display name %ws\n", pszRet);
//         }
//         else if (pformatetcIn->cfFormat == This->cfDisplayKey)
//         {
//             pszRet = This->pDisplayKey;
//             DPRINT1("CDevSettings::GetData returns display key %ws\n", pszRet);
//         }
//         else if (pformatetcIn->cfFormat == This->cfDisplayId)
//         {
//             pszRet = This->pDisplayId;
//             DPRINT1("CDevSettings::GetData returns display id %ws\n", pszRet);
//         }
//         else if (pformatetcIn->cfFormat == This->cfMonitorName)
//         {
//             pszRet = This->pMonitorName;
//             DPRINT1("CDevSettings::GetData returns monitor name %ws\n", pszRet);
//         }
//         else if (pformatetcIn->cfFormat == This->cfMonitorDevice)
//         {
//             pszRet = This->pMonitorDevice;
//             DPRINT1("CDevSettings::GetData returns monitor device %ws\n", pszRet);
//         }
//         else if (pformatetcIn->cfFormat == This->cfExtInterface)
//         {
//             PDESK_EXT_INTERFACE pIface;
//
//             pIface = GlobalAlloc(GPTR,
//                                  sizeof(*pIface));
//             if (pIface != NULL)
//             {
//                 CopyMemory(pIface,
//                            &This->ExtInterface,
//                            sizeof(This->ExtInterface));
//
//                 DPRINT1("CDevSettings::GetData returns the desk.cpl extension interface\n");
//
//                 pmedium->tymed = TYMED_HGLOBAL;
//                 pmedium->hGlobal = pIface;
//
//                 return S_OK;
//             }
//             else
//                 return E_OUTOFMEMORY;
//         }
//         else if (pformatetcIn->cfFormat == This->cfDisplayStateFlags)
//         {
//             PDWORD pdw;
//
//             pdw = GlobalAlloc(GPTR,
//                               sizeof(*pdw));
//             if (pdw != NULL)
//             {
//                 *pdw = This->StateFlags;
//
//                 DPRINT1("CDevSettings::GetData returns the display state flags %x\n", This->StateFlags);
//
//                 pmedium->tymed = TYMED_HGLOBAL;
//                 pmedium->hGlobal = pdw;
//
//                 return S_OK;
//             }
//             else
//                 return E_OUTOFMEMORY;
//         }
//         else if (pformatetcIn->cfFormat == This->cfPruningMode)
//         {
//             PBYTE pb;
//
//             pb = GlobalAlloc(GPTR,
//                              sizeof(*pb));
//             if (pb != NULL)
//             {
//                 *pb = (This->bModesPruned && This->bPruningOn);
//
//                 pmedium->tymed = TYMED_HGLOBAL;
//                 pmedium->hGlobal = pb;
//
//                 return S_OK;
//             }
//             else
//                 return E_OUTOFMEMORY;
//         }
//
//         /* NOTE: This only returns null-terminated strings! */
//         if (pszRet == NULL)
//             pszRet = szEmpty;
//
//         pszBuf = GlobalAlloc(GPTR,
//                              (wcslen(pszRet) + 1) * sizeof(WCHAR));
//         if (pszBuf != NULL)
//         {
//             hr = StringCbCopyW(pszBuf, (wcslen(pszRet) + 1) * sizeof(WCHAR), pszRet);
//             if (FAILED(hr))
//             {
//                 GlobalFree(pszBuf);
//                 return hr;
//             }
//
//             pmedium->tymed = TYMED_HGLOBAL;
//             pmedium->hGlobal = pszBuf;
//
//             hr = S_OK;
//         }
//         else
//             hr = E_OUTOFMEMORY;
//     }
//
//     return hr;
// }
//
// static HRESULT STDMETHODCALLTYPE
// CDevSettings_GetDataHere(IDataObject* iface,
//                          FORMATETC* pformatetc,
//                          STGMEDIUM* pmedium)
// {
//     ZeroMemory(pformatetc,
//                sizeof(*pformatetc));
//     return E_NOTIMPL;
// }
//
// static HRESULT STDMETHODCALLTYPE
// CDevSettings_QueryGetData(IDataObject* iface,
//                           FORMATETC* pformatetc)
// {
// #if DEBUG
//     TCHAR szFormatName[255];
// #endif
//     PCDevSettings This = impl_from_IDataObject(iface);
//
//     if (pformatetc->dwAspect != DVASPECT_CONTENT)
//         return DV_E_DVASPECT;
//
//     if (pformatetc->lindex != -1)
//         return DV_E_LINDEX;
//
//     if (!(pformatetc->tymed & TYMED_HGLOBAL))
//         return DV_E_TYMED;
//
//     /* Check if the requested data can be provided */
//     if (pformatetc->cfFormat == This->cfExtInterface ||
//         pformatetc->cfFormat == This->cfDisplayDevice ||
//         pformatetc->cfFormat == This->cfDisplayName ||
//         pformatetc->cfFormat == This->cfDisplayId ||
//         pformatetc->cfFormat == This->cfDisplayKey ||
//         pformatetc->cfFormat == This->cfDisplayStateFlags ||
//         pformatetc->cfFormat == This->cfMonitorDevice ||
//         pformatetc->cfFormat == This->cfMonitorName ||
//         pformatetc->cfFormat == This->cfPruningMode)
//     {
//         return S_OK;
//     }
// #if DEBUG
//     else
//     {
//         if (GetClipboardFormatName(pformatetc->cfFormat,
//                                    szFormatName,
//                                    sizeof(szFormatName) / sizeof(szFormatName[0])))
//         {
//             DPRINT1("CDevSettings::QueryGetData(\"%ws\")\n", szFormatName);
//         }
//         else
//         {
//             DPRINT1("CDevSettings::QueryGetData(Format %u)\n", (unsigned int)pformatetc->cfFormat);
//         }
//     }
// #endif
//
//     return DV_E_FORMATETC;
// }
//
// static HRESULT STDMETHODCALLTYPE
// CDevSettings_GetCanonicalFormatEtc(IDataObject* iface,
//                                    FORMATETC* pformatectIn,
//                                    FORMATETC* pformatetcOut)
// {
//     HRESULT hr;
//
//     DPRINT1("CDevSettings::GetCanonicalFormatEtc\n");
//
//     hr = IDataObject_QueryGetData(iface,
//                                   pformatectIn);
//     if (SUCCEEDED(hr))
//     {
//         CopyMemory(pformatetcOut,
//                    pformatectIn,
//                    sizeof(FORMATETC));
//
//         /* Make sure the data is target device independent */
//         if (pformatectIn->ptd == NULL)
//             hr = DATA_S_SAMEFORMATETC;
//         else
//         {
//             pformatetcOut->ptd = NULL;
//             hr = S_OK;
//         }
//     }
//     else
//     {
//         ZeroMemory(pformatetcOut,
//                    sizeof(FORMATETC));
//     }
//
//     return hr;
// }
//
// static HRESULT STDMETHODCALLTYPE
// CDevSettings_SetData(IDataObject* iface,
//                      FORMATETC* pformatetc,
//                      STGMEDIUM* pmedium,
//                      BOOL fRelease)
// {
//     DPRINT1("CDevSettings::SetData UNIMPLEMENTED\n");
//     return E_NOTIMPL;
// }
//
// static __inline VOID
// pCDevSettings_FillFormatEtc(FORMATETC *pFormatEtc,
//                             CLIPFORMAT cf)
// {
//     pFormatEtc->cfFormat = cf;
//     pFormatEtc->ptd = NULL;
//     pFormatEtc->dwAspect = DVASPECT_CONTENT;
//     pFormatEtc->lindex = -1;
//     pFormatEtc->tymed = TYMED_HGLOBAL;
// }
//
// static HRESULT STDMETHODCALLTYPE
// CDevSettings_EnumFormatEtc(IDataObject* iface,
//                            DWORD dwDirection,
//                            IEnumFORMATETC** ppenumFormatEtc)
// {
//     HRESULT hr;
//     FORMATETC fetc[9];
//     PCDevSettings This = impl_from_IDataObject(iface);
//
//     *ppenumFormatEtc = NULL;
//
//     if (dwDirection == DATADIR_GET)
//     {
//         pCDevSettings_FillFormatEtc(&fetc[0],
//                                     This->cfExtInterface);
//         pCDevSettings_FillFormatEtc(&fetc[1],
//                                     This->cfDisplayDevice);
//         pCDevSettings_FillFormatEtc(&fetc[2],
//                                     This->cfDisplayName);
//         pCDevSettings_FillFormatEtc(&fetc[3],
//                                     This->cfDisplayId);
//         pCDevSettings_FillFormatEtc(&fetc[4],
//                                     This->cfDisplayKey);
//         pCDevSettings_FillFormatEtc(&fetc[5],
//                                     This->cfDisplayStateFlags);
//         pCDevSettings_FillFormatEtc(&fetc[6],
//                                     This->cfMonitorName);
//         pCDevSettings_FillFormatEtc(&fetc[7],
//                                     This->cfMonitorDevice);
//         pCDevSettings_FillFormatEtc(&fetc[8],
//                                     This->cfPruningMode);
//
//         hr = SHCreateStdEnumFmtEtc(sizeof(fetc) / sizeof(fetc[0]),
//                                    fetc,
//                                    ppenumFormatEtc);
//     }
//     else
//         hr = E_NOTIMPL;
//
//     return hr;
// }
//
// static HRESULT STDMETHODCALLTYPE
// CDevSettings_DAdvise(IDataObject* iface,
//                      FORMATETC* pformatetc,
//                      DWORD advf,
//                      IAdviseSink* pAdvSink,
//                      DWORD* pdwConnection)
// {
//     *pdwConnection = 0;
//     return OLE_E_ADVISENOTSUPPORTED;
// }
//
// static HRESULT STDMETHODCALLTYPE
// CDevSettings_DUnadvise(IDataObject* iface,
//                        DWORD dwConnection)
// {
//     return OLE_E_ADVISENOTSUPPORTED;
// }
//
// static HRESULT STDMETHODCALLTYPE
// CDevSettings_EnumDAdvise(IDataObject* iface,
//                          IEnumSTATDATA** ppenumAdvise)
// {
//     *ppenumAdvise = NULL;
//     return OLE_E_ADVISENOTSUPPORTED;
// }
//
// static const struct IDataObjectVtbl vtblIDataObject = {
//     CDevSettings_QueryInterface,
//     CDevSettings_AddRef,
//     CDevSettings_Release,
//     CDevSettings_GetData,
//     CDevSettings_GetDataHere,
//     CDevSettings_QueryGetData,
//     CDevSettings_GetCanonicalFormatEtc,
//     CDevSettings_SetData,
//     CDevSettings_EnumFormatEtc,
//     CDevSettings_DAdvise,
//     CDevSettings_DUnadvise,
//     CDevSettings_EnumDAdvise,
// };
//
// IDataObject *
// CreateDevSettings(PDISPLAY_DEVICE_ENTRY DisplayDeviceInfo)
// {
//     PCDevSettings This;
//
//     This = HeapAlloc(GetProcessHeap(),
//                      0,
//                      sizeof(*This));
//     if (This != NULL)
//     {
//         This->lpIDataObjectVtbl = &vtblIDataObject;
//         This->ref = 1;
//
//         if (SUCCEEDED(pCDevSettings_Initialize(This,
//                                                DisplayDeviceInfo)))
//         {
//             return impl_to_interface(This, IDataObject);
//         }
//
//         CDevSettings_Release(impl_to_interface(This, IDataObject));
//     }
//
//     return NULL;
// }
//
// LONG WINAPI
// DisplaySaveSettings(PVOID pContext,
//                     HWND hwndPropSheet)
// {
//     PCDevSettings This = impl_from_IDataObject((IDataObject *)pContext);
//     LONG rc = DISP_CHANGE_SUCCESSFUL;
//
//     if (This->lpCurDevMode != This->lpOrigDevMode)
//     {
//         SETTINGS_ENTRY seOrig, seCur;
//         BOOL Ret;
//
//         seOrig.dmPelsWidth = This->lpOrigDevMode->dmPelsWidth;
//         seOrig.dmPelsHeight = This->lpOrigDevMode->dmPelsHeight;
//         seOrig.dmBitsPerPel = This->lpOrigDevMode->dmBitsPerPel;
//         seOrig.dmDisplayFrequency = This->lpOrigDevMode->dmDisplayFrequency;
//
//         seCur.dmPelsWidth = This->lpCurDevMode->dmPelsWidth;
//         seCur.dmPelsHeight = This->lpCurDevMode->dmPelsHeight;
//         seCur.dmBitsPerPel = This->lpCurDevMode->dmBitsPerPel;
//         seCur.dmDisplayFrequency = This->lpCurDevMode->dmDisplayFrequency;
//
//         Ret = SwitchDisplayMode(hwndPropSheet,
//                                 This->pDisplayDevice,
//                                 &seOrig,
//                                 &seCur,
//                                 &rc);
//
//         if (rc == DISP_CHANGE_SUCCESSFUL)
//         {
//             if (Ret)
//                 This->lpOrigDevMode = This->lpCurDevMode;
//             else
//                 This->lpCurDevMode = This->lpOrigDevMode;
//         }
//     }
//
//     return rc;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
