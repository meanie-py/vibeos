// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_447438009695: [u16; 6] = [0x0025u16, 0x0073, 0x0020, 0x0025, 0x0073, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut registeredAppsLaunchPath: [WCHAR; 77] = unsafe {
    ::core::mem::transmute::<
        [u8; 154],
        [WCHAR; 77],
    >(
        *b"S\0O\0F\0T\0W\0A\0R\0E\0\\\0M\0i\0c\0r\0o\0s\0o\0f\0t\0\\\0W\0i\0n\0d\0o\0w\0s\0\\\0C\0u\0r\0r\0e\0n\0t\0V\0e\0r\0s\0i\0o\0n\0\\\0S\0t\0i\0l\0l\0I\0m\0a\0g\0e\0\\\0R\0e\0g\0i\0s\0t\0e\0r\0e\0d\0 \0A\0p\0p\0l\0i\0c\0a\0t\0i\0o\0n\0s\0\0\0",
    )
};

#[repr(C)]
pub struct stillimage {
    pub IUnknown_inner: IUnknown,
    pub IStillImageW_iface: IStillImageW,
    pub outer_unk: *mut IUnknown,
    pub r#ref: LONG,
}

static stillimagew_GetHwNotificationState_DUMMY: u8 = 0;

unsafe extern "C" fn impl_from_IStillImageW(iface: *mut IStillImageW) -> *mut stillimage {
    return (iface as *mut u8).wrapping_sub(core::mem::offset_of!(stillimage, IStillImageW_iface)) as *mut stillimage;
}

unsafe extern "system" fn stillimagew_QueryInterface(iface: *mut IStillImageW, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let mut This: *mut stillimage = impl_from_IStillImageW(iface);
    return IUnknown_QueryInterface((*This).outer_unk, riid, ppvObject);
}

unsafe extern "system" fn stillimagew_AddRef(iface: *mut IStillImageW) -> ULONG {
    let mut This: *mut stillimage = impl_from_IStillImageW(iface);
    return IUnknown_AddRef((*This).outer_unk);
}

unsafe extern "system" fn stillimagew_Release(iface: *mut IStillImageW) -> ULONG {
    let mut This: *mut stillimage = impl_from_IStillImageW(iface);
    return IUnknown_Release((*This).outer_unk);
}
unsafe extern "system" fn stillimagew_Initialize(
    mut iface: *mut IStillImageW,
    mut hinst: HINSTANCE,
    mut dwVersion: DWORD,
) -> HRESULT {
    let mut This: *mut stillimage = impl_from_IStillImageW(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/sti/sti.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"stillimagew_Initialize\0" as *const u8 as *const ::core::ffi::c_char,
            76 as ::core::ffi::c_int,
            b"(%p, %p, 0x%lX)\n\0" as *const u8 as *const ::core::ffi::c_char,
            This,
            hinst,
            dwVersion,
        );
    }
    return S_OK;
}
unsafe extern "system" fn stillimagew_GetDeviceList(
    mut iface: *mut IStillImageW,
    mut dwType: DWORD,
    mut dwFlags: DWORD,
    mut pdwItemsReturned: *mut DWORD,
    mut ppBuffer: *mut LPVOID,
) -> HRESULT {
    let mut This: *mut stillimage = impl_from_IStillImageW(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/sti/sti.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"stillimagew_GetDeviceList\0" as *const u8 as *const ::core::ffi::c_char,
            84 as ::core::ffi::c_int,
            b"(%p, %lu, 0x%lX, %p, %p): stub\n\0" as *const u8 as *const ::core::ffi::c_char,
            This,
            dwType,
            dwFlags,
            pdwItemsReturned,
            ppBuffer,
        );
    }
    return E_NOTIMPL;
}

unsafe extern "system" fn stillimagew_GetDeviceInfo(
    iface: *mut IStillImageW,
    pwszDeviceName: LPWSTR,
    ppBuffer: *mut LPVOID,
) -> HRESULT {
    let This: *mut stillimage = impl_from_IStillImageW(iface);
    return E_NOTIMPL;
}

unsafe extern "system" fn stillimagew_CreateDevice(
    mut iface: *mut IStillImageW,
    mut pwszDeviceName: LPWSTR,
    mut dwMode: DWORD,
    mut pDevice: *mut PSTIDEVICEW,
    mut pUnkOuter: LPUNKNOWN,
) -> HRESULT {
    let mut This: *mut stillimage = impl_from_IStillImageW(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/sti/sti.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"stillimagew_CreateDevice\0" as *const u8 as *const ::core::ffi::c_char,
            100 as ::core::ffi::c_int,
            b"(%p, %s, %lu, %p, %p): stub\n\0" as *const u8 as *const ::core::ffi::c_char,
            This,
            debugstr_w(pwszDeviceName as *const WCHAR),
            dwMode,
            pDevice,
            pUnkOuter,
        );
    }
    return E_NOTIMPL;
}

unsafe extern "system" fn stillimagew_GetDeviceValue(
    iface: *mut IStillImageW,
    pwszDeviceName: LPWSTR,
    pValueName: LPWSTR,
    pType: LPDWORD,
    pData: LPBYTE,
    cbData: LPDWORD,
) -> HRESULT {
    let mut This: *mut stillimage = impl_from_IStillImageW(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/sti/sti.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"stillimagew_GetDeviceValue\0" as *const u8 as *const ::core::ffi::c_char,
            109 as ::core::ffi::c_int,
            b"(%p, %s, %s, %p, %p, %p): stub\n\0" as *const u8 as *const ::core::ffi::c_char,
            This,
            debugstr_w(pwszDeviceName as *const WCHAR),
            debugstr_w(pValueName as *const WCHAR),
            pType,
            pData,
            cbData,
        );
    }
    return E_NOTIMPL;
}

unsafe extern "system" fn stillimagew_SetDeviceValue(
    iface: *mut IStillImageW,
    pwszDeviceName: LPWSTR,
    pValueName: LPWSTR,
    type_: DWORD,
    pData: LPBYTE,
    cbData: DWORD,
) -> HRESULT {
    let mut This: *mut stillimage = impl_from_IStillImageW(iface);
    return E_NOTIMPL;
}

unsafe extern "system" fn stillimagew_GetSTILaunchInformation(iface: *mut IStillImageW, pwszDeviceName: LPWSTR, pdwEventCode: *mut DWORD, pwszEventName: LPWSTR) -> HRESULT {
    let mut This: *mut stillimage = impl_from_IStillImageW(iface);
    DPRINT1!("(%p, %p, %p, %p): stub\n", This, pwszDeviceName, pdwEventCode, pwszEventName);
    return E_NOTIMPL;
}

unsafe extern "system" fn stillimagew_RegisterLaunchApplication(iface: *mut IStillImageW, pwszAppName: LPWSTR, pwszCommandLine: LPWSTR) -> HRESULT {
    static commandLineSuffix: [u16; 27] = [
    0x002Fu16, 0x0053, 0x0074, 0x0069, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065,
    0x003A, 0x0025, 0x0031, 0x0020, 0x002F, 0x0053, 0x0074, 0x0069, 0x0045, 0x0076,
    0x0065, 0x006E, 0x0074, 0x003A, 0x0025, 0x0032, 0x0000,
];
    let mut registeredAppsKey: HKEY = core::ptr::null_mut();
    let mut ret: DWORD = 0;
    let mut hr: HRESULT = S_OK;
    let mut This: *mut stillimage = impl_from_IStillImageW(iface);

    ret = (RegCreateKeyW(HKEY_LOCAL_MACHINE, core::ptr::addr_of!(registeredAppsLaunchPath) as LPCWSTR, core::ptr::addr_of_mut!(registeredAppsKey))) as u32;
    if ret == ERROR_SUCCESS {
        let mut len: usize = (lstrlenW(pwszCommandLine as LPCWSTR) + 1 + lstrlenW(core::ptr::addr_of!(commandLineSuffix) as LPCWSTR) + 1) as usize;
        let mut value: *mut WCHAR = HeapAlloc(GetProcessHeap(), 0, (len * core::mem::size_of::<WCHAR>()) as SIZE_T) as *mut WCHAR;
        if !value.is_null() {
            swprintf(value, len, RUSTCONV_WSTR_447438009695.as_ptr(), pwszCommandLine, core::ptr::addr_of!(commandLineSuffix) as *const u16);
            ret = (RegSetValueExW(registeredAppsKey, pwszAppName as LPCWSTR, 0, REG_SZ, value as LPBYTE, (((lstrlenW(value as LPCWSTR) + 1) as usize) * core::mem::size_of::<WCHAR>()) as DWORD)) as u32;
            if ret != ERROR_SUCCESS {
                hr = HRESULT_FROM_WIN32!(ret);
            }
            HeapFree(GetProcessHeap(), 0, value as LPVOID);
        } else {
            hr = E_OUTOFMEMORY;
        }
        RegCloseKey(registeredAppsKey);
    } else {
        hr = HRESULT_FROM_WIN32!(ret);
    }
    return hr;
}

unsafe extern "system" fn stillimagew_UnregisterLaunchApplication(iface: *mut IStillImageW, pwszAppName: LPWSTR) -> HRESULT {
    let mut This: *mut stillimage = impl_from_IStillImageW(iface);
    let mut registeredAppsKey: HKEY = core::ptr::null_mut();
    let mut ret: DWORD;
    let mut hr: HRESULT = S_OK;

    ret = RegCreateKeyW(
        HKEY_LOCAL_MACHINE,
        core::ptr::addr_of!(registeredAppsLaunchPath) as LPCWSTR,
        core::ptr::addr_of_mut!(registeredAppsKey),
    ) as DWORD;
    if ret == ERROR_SUCCESS as DWORD {
        ret = RegDeleteValueW(registeredAppsKey, pwszAppName as LPCWSTR) as DWORD;
        if ret != ERROR_SUCCESS as DWORD {
            hr = HRESULT_FROM_WIN32!(ret);
        }
        RegCloseKey(registeredAppsKey);
    } else {
        hr = HRESULT_FROM_WIN32!(ret);
    }
    return hr;
}
unsafe extern "system" fn stillimagew_EnableHwNotifications(
    iface: *mut IStillImageW,
    pwszDeviceName: LPCWSTR,
    bNewState: BOOL,
) -> HRESULT {
    let mut This: *mut stillimage = impl_from_IStillImageW(iface);
    if ((*__wine_dbch___default).flags as i32) & ((1i32) << (__WINE_DBCL_FIXME as i32)) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/sti/sti.c\0"
                as *const u8 as *const i8,
            b"stillimagew_EnableHwNotifications\0" as *const u8 as *const i8,
            191i32,
            b"(%p, %s, %u): stub\n\0" as *const u8 as *const i8,
            This,
            debugstr_w(pwszDeviceName),
            bNewState,
        );
    }
    return E_NOTIMPL;
}

unsafe extern "system" fn stillimagew_GetHwNotificationState(
    iface: *mut IStillImageW,
    pwszDeviceName: LPCWSTR,
    pbCurrentState: *mut BOOL,
) -> HRESULT {
    let mut This: *mut stillimage = impl_from_IStillImageW(iface);
    if (*__wine_dbch___default).flags as i32 & (1i32) << __WINE_DBCL_FIXME as i32 != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/sti/sti.c\0".as_ptr()
                as *const core::ffi::c_char,
            b"stillimagew_GetHwNotificationState\0".as_ptr() as *const core::ffi::c_char,
            199i32,
            b"(%p, %s, %p): stub\n\0".as_ptr() as *const core::ffi::c_char,
            This,
            debugstr_w(pwszDeviceName as *const WCHAR),
            pbCurrentState,
        );
    }
    return E_NOTIMPL;
}

unsafe extern "system" fn stillimagew_RefreshDeviceBus(
    mut iface: *mut IStillImageW,
    mut pwszDeviceName: LPCWSTR,
) -> HRESULT {
    let mut This: *mut stillimage = impl_from_IStillImageW(iface);
    FIXME!("(%p, %s): stub\n", This, debugstr_w(pwszDeviceName));
    return E_NOTIMPL;
}

unsafe extern "system" fn stillimagew_LaunchApplicationForDevice(
    iface: *mut IStillImageW,
    pwszDeviceName: LPWSTR,
    pwszAppName: LPWSTR,
    pStiNotify: LPSTINOTIFY,
) -> HRESULT {
    let mut This: *mut stillimage = impl_from_IStillImageW(iface);
    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << (__WINE_DBCL_FIXME as i32))) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/sti/sti.c\0"
                as *const u8 as *const i8,
            b"stillimagew_LaunchApplicationForDevice\0" as *const u8 as *const i8,
            215 as i32,
            b"(%p, %s, %s, %p): stub\n\0" as *const u8 as *const i8,
            This,
            debugstr_w(pwszDeviceName as *const u16),
            debugstr_w(pwszAppName as *const u16),
            pStiNotify,
        );
    }
    return E_NOTIMPL;
}

unsafe extern "system" fn stillimagew_SetupDeviceParameters(iface: *mut IStillImageW, pDevInfo: PSTI_DEVICE_INFORMATIONW) -> HRESULT {
    let This: *mut stillimage = impl_from_IStillImageW(iface);
    DPRINT1!("(%p, %p): stub\n", This, pDevInfo);
    return E_NOTIMPL;
}

unsafe extern "system" fn stillimagew_WriteToErrorLog(iface: *mut IStillImageW, dwMessageType: DWORD, pszMessage: LPCWSTR) -> HRESULT {
    let This: *mut stillimage = impl_from_IStillImageW(iface);
    if ((*__wine_dbch___default).flags as i32 & ((1i32) << (__WINE_DBCL_FIXME as i32))) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/sti/sti.c\0" as *const u8 as *const i8,
            b"stillimagew_WriteToErrorLog\0" as *const u8 as *const i8,
            229 as i32,
            b"(%p, %lu, %s): stub\n\0" as *const u8 as *const i8,
            This,
            dwMessageType,
            debugstr_w(pszMessage as *const WCHAR),
        );
    }
    return E_NOTIMPL;
}

unsafe extern "C" fn impl_from_IUnknown(iface: *mut IUnknown) -> *mut stillimage {
    return (iface as *mut u8).sub(core::mem::offset_of!(stillimage, IUnknown_inner)) as *mut stillimage;
}

unsafe extern "system" fn Internal_QueryInterface(iface: *mut IUnknown, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let This: *mut stillimage = impl_from_IUnknown(iface);

    if IsEqualGUID!(riid, core::ptr::addr_of!(IID_IUnknown)) != false {
        *ppvObject = iface as *mut core::ffi::c_void;
    } else if IsEqualGUID!(riid, core::ptr::addr_of!(IID_IStillImageW)) != false {
        *ppvObject = core::ptr::addr_of_mut!((*This).IStillImageW_iface) as *mut core::ffi::c_void;
    } else {
        if IsEqualGUID!(riid, core::ptr::addr_of!(IID_IStillImageA)) != false {
            FIXME!("interface IStillImageA is unsupported on Windows Vista too, please report if it's needed\n");
        } else {
            FIXME!("interface %s not implemented\n", debugstr_guid(riid));
        }
        *ppvObject = core::ptr::null_mut();
        return E_NOINTERFACE;
    }

    IUnknown_AddRef((*ppvObject) as *mut IUnknown);
    return S_OK;
}

unsafe extern "system" fn Internal_AddRef(iface: *mut IUnknown) -> ULONG {
    let This: *mut stillimage = impl_from_IUnknown(iface);
    return InterlockedIncrement(core::ptr::addr_of_mut!((*This).r#ref)) as ULONG;
}

unsafe extern "system" fn Internal_Release(iface: *mut IUnknown) -> ULONG {
    let mut ref_0: ULONG;
    let This: *mut stillimage = impl_from_IUnknown(iface);
    ref_0 = InterlockedDecrement(core::ptr::addr_of_mut!((*This).r#ref) as *mut ::core::ffi::c_long) as ULONG;
    if ref_0 == 0 {
        HeapFree(GetProcessHeap(), 0, This as PVOID);
    }
    return ref_0;
}

#[no_mangle]
pub unsafe extern "system" fn StiCreateInstanceA(
    hinst: HINSTANCE,
    dwVer: DWORD,
    ppSti: *mut PSTIA,
    pUnkOuter: LPUNKNOWN,
) -> HRESULT {
    // FIXME("(%p, %lu, %p, %p): stub, unimplemented on Windows Vista too, please report if it's needed\n", hinst, dwVer, ppSti, pUnkOuter);
    STG_E_UNIMPLEMENTEDFUNCTION
}

#[no_mangle]
pub unsafe extern "system" fn StiCreateInstanceW(
    hinst: HINSTANCE,
    dwVer: DWORD,
    ppSti: *mut PSTIW,
    pUnkOuter: LPUNKNOWN,
) -> HRESULT {
    let mut This: *mut stillimage = core::ptr::null_mut();
    let mut hr: HRESULT;

    This = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY as DWORD,
        core::mem::size_of::<stillimage>() as SIZE_T,
    ) as *mut stillimage;
    if !This.is_null() {
        (*This).IStillImageW_iface.lpVtbl =
            core::ptr::addr_of!(stillimagew_vtbl) as *const IStillImageWVtbl;
        (*This).IUnknown_inner.lpVtbl =
            core::ptr::addr_of!(internal_unk_vtbl) as *const IUnknownVtbl;
        if !pUnkOuter.is_null() {
            (*This).outer_unk = pUnkOuter as *mut IUnknown;
        } else {
            (*This).outer_unk = core::ptr::addr_of_mut!((*This).IUnknown_inner);
        }
        (*This).r#ref = 1 as LONG;

        hr = IStillImage_Initialize!(
            core::ptr::addr_of_mut!((*This).IStillImageW_iface),
            hinst,
            dwVer,
        );
        if hr >= 0 as HRESULT {
            if !pUnkOuter.is_null() {
                *ppSti = core::ptr::addr_of_mut!((*This).IUnknown_inner) as *mut IStillImageW
                    as PSTIW;
            } else {
                *ppSti = core::ptr::addr_of_mut!((*This).IStillImageW_iface) as PSTIW;
            }
        }
    } else {
        hr = E_OUTOFMEMORY as HRESULT;
    }

    return hr;
}

// COM vtable tables — generated from sti.c by vtbl_gen.py
/// sti.c:233 `static const IStillImageWVtbl stillimagew_vtbl`  (slot order: rust:types_amd64.rs)
pub static stillimagew_vtbl: IStillImageWVtbl = IStillImageWVtbl {
    QueryInterface: Some(stillimagew_QueryInterface),
    AddRef: Some(stillimagew_AddRef),
    Release: Some(stillimagew_Release),
    Initialize: Some(stillimagew_Initialize),
    GetDeviceList: Some(stillimagew_GetDeviceList),
    GetDeviceInfo: Some(stillimagew_GetDeviceInfo),
    CreateDevice: Some(stillimagew_CreateDevice),
    GetDeviceValue: Some(stillimagew_GetDeviceValue),
    SetDeviceValue: Some(stillimagew_SetDeviceValue),
    GetSTILaunchInformation: Some(stillimagew_GetSTILaunchInformation),
    RegisterLaunchApplication: Some(stillimagew_RegisterLaunchApplication),
    UnregisterLaunchApplication: Some(stillimagew_UnregisterLaunchApplication),
    EnableHwNotifications: Some(stillimagew_EnableHwNotifications),
    GetHwNotificationState: Some(stillimagew_GetHwNotificationState),
    RefreshDeviceBus: Some(stillimagew_RefreshDeviceBus),
    LaunchApplicationForDevice: Some(stillimagew_LaunchApplicationForDevice),
    SetupDeviceParameters: Some(stillimagew_SetupDeviceParameters),
    WriteToErrorLog: Some(stillimagew_WriteToErrorLog),
};

/// sti.c:301 `static const IUnknownVtbl internal_unk_vtbl`  (slot order: com-sys/bindgen_i386.rs)
pub static internal_unk_vtbl: IUnknownVtbl = IUnknownVtbl {
    QueryInterface: Some(Internal_QueryInterface),
    AddRef: Some(Internal_AddRef),
    Release: Some(Internal_Release),
};
