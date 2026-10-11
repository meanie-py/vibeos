// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use ntdll_sys::nt::*;
pub static RUSTCONV_WSTR_15D33A6D42D7: [u16; 15] = [
    0x004Fu16, 0x0053, 0x0050, 0x0072, 0x006F, 0x0064, 0x0075, 0x0063, 0x0074, 0x0053,
    0x0075, 0x0069, 0x0074, 0x0065, 0x0000,
];
pub static RUSTCONV_WSTR_1F7A38F3D6FA: [u16; 8] = [0x0056u16, 0x0065, 0x0072, 0x0073, 0x0069, 0x006F, 0x006E, 0x0000];
pub static RUSTCONV_WSTR_216E1EFDB8A1: [u16; 13] = [
    0x0041u16, 0x0072, 0x0063, 0x0068, 0x0069, 0x0074, 0x0065, 0x0063, 0x0074, 0x0075,
    0x0072, 0x0065, 0x0000,
];
pub static RUSTCONV_WSTR_2A663D6AB2F2: [u16; 12] = [
    0x0042u16, 0x0075, 0x0069, 0x006C, 0x0064, 0x004E, 0x0075, 0x006D, 0x0062, 0x0065,
    0x0072, 0x0000,
];
pub static RUSTCONV_WSTR_420BAC9386E7: [u16; 36] = [
    0x0053u16, 0x0045, 0x004C, 0x0045, 0x0043, 0x0054, 0x0020, 0x002A, 0x0020, 0x0046,
    0x0052, 0x004F, 0x004D, 0x0020, 0x0057, 0x0069, 0x006E, 0x0033, 0x0032, 0x005F,
    0x004F, 0x0070, 0x0065, 0x0072, 0x0061, 0x0074, 0x0069, 0x006E, 0x0067, 0x0053,
    0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x0000,
];
pub static RUSTCONV_WSTR_60F3492FB3CB: [u16; 11] = [
    0x0052u16, 0x004F, 0x004F, 0x0054, 0x005C, 0x0043, 0x0049, 0x004D, 0x0056, 0x0032,
    0x0000,
];
pub static RUSTCONV_WSTR_925254771579: [u16; 4] = [0x0025u16, 0x0068, 0x0075, 0x0000];
pub static RUSTCONV_WSTR_9A3C13E25FF3: [u16; 7] = [0x004Fu16, 0x0053, 0x0054, 0x0079, 0x0070, 0x0065, 0x0000];
pub static RUSTCONV_WSTR_AC4862C373EB: [u16; 30] = [
    0x0053u16, 0x0045, 0x004C, 0x0045, 0x0043, 0x0054, 0x0020, 0x002A, 0x0020, 0x0046,
    0x0052, 0x004F, 0x004D, 0x0020, 0x0057, 0x0069, 0x006E, 0x0033, 0x0032, 0x005F,
    0x0050, 0x0072, 0x006F, 0x0063, 0x0065, 0x0073, 0x0073, 0x006F, 0x0072, 0x0000,
];
pub static RUSTCONV_WSTR_B1DB089221CB: [u16; 24] = [
    0x0053u16, 0x0065, 0x0072, 0x0076, 0x0069, 0x0063, 0x0065, 0x0050, 0x0061, 0x0063,
    0x006B, 0x004D, 0x0069, 0x006E, 0x006F, 0x0072, 0x0056, 0x0065, 0x0072, 0x0073,
    0x0069, 0x006F, 0x006E, 0x0000,
];
pub static RUSTCONV_WSTR_C6B74772B6A4: [u16; 4] = [0x0057u16, 0x0051, 0x004C, 0x0000];
pub static RUSTCONV_WSTR_D26E7C09CBD9: [u16; 24] = [
    0x0053u16, 0x0065, 0x0072, 0x0076, 0x0069, 0x0063, 0x0065, 0x0050, 0x0061, 0x0063,
    0x006B, 0x004D, 0x0061, 0x006A, 0x006F, 0x0072, 0x0056, 0x0065, 0x0072, 0x0073,
    0x0069, 0x006F, 0x006E, 0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut VersionInfoArchitecture: UINT = 0;

#[no_mangle]
pub static mut VersionInfoOsProductSuite: UINT = 0;

#[no_mangle]
pub static mut VersionInfoOsType: UINT = 0;

#[no_mangle]
pub static mut VersionInfoVersion: [WCHAR; 260] = [0; 260];

#[no_mangle]
pub static mut VersionInfoBuildNumber: [WCHAR; 260] = [0; 260];

#[no_mangle]
pub static mut VersionInfoServicePackMajorVersion: [WCHAR; 260] = [0; 260];

#[no_mangle]
pub static mut VersionInfoServicePackMinorVersion: [WCHAR; 260] = [0; 260];

#[no_mangle]
pub static mut CLSID_WbemLocator: GUID = _GUID {
    Data1: 0x4590f811 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x1d3a as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0x89 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x1f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xaa as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x24 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut IID_IWbemLocator: GUID = _GUID {
    Data1: 0xdc12a687 as ::core::ffi::c_uint,
    Data2: 0x737f as ::core::ffi::c_ushort,
    Data3: 0x11cf as ::core::ffi::c_ushort,
    Data4: [
        0x88 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xaa as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x24 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};
#[no_mangle]
pub unsafe extern "C" fn CheckOsVersion(mut pfnOsVersionCheck: PNS_OSVERSIONCHECK) -> BOOL {
    DPRINT!("CheckOsVersion(%p)\n", pfnOsVersionCheck);

    if pfnOsVersionCheck.is_none() {
        return TRUE;
    }

    return pfnOsVersionCheck(
        VersionInfoOsType,
        VersionInfoOsProductSuite,
        core::ptr::addr_of_mut!(VersionInfoVersion) as *mut WCHAR as LPCWSTR,
        core::ptr::addr_of_mut!(VersionInfoBuildNumber) as *mut WCHAR as LPCWSTR,
        core::ptr::addr_of_mut!(VersionInfoServicePackMajorVersion) as *mut WCHAR as LPCWSTR,
        core::ptr::addr_of_mut!(VersionInfoServicePackMinorVersion) as *mut WCHAR as LPCWSTR,
        VersionInfoArchitecture,
        0,
    );
}

unsafe extern "C" fn QueryOperatingSystemInfo(Services: *mut IWbemServices) -> HRESULT {
    let mut Result: *mut IEnumWbemClassObject = core::ptr::null_mut();
    let mut Object: *mut IWbemClassObject = core::ptr::null_mut();
    let mut Wql: BSTR = core::ptr::null_mut();
    let mut Query: BSTR = core::ptr::null_mut();
    let mut Name: BSTR = core::ptr::null_mut();
    let mut Type: CIMTYPE = 0;
    let mut Count: ULONG = 0;
    let mut Value: VARIANT = core::mem::zeroed();
    let mut hr: HRESULT = 0;

    'done: {
        Wql = SysAllocString(RUSTCONV_WSTR_C6B74772B6A4.as_ptr());
        if Wql.is_null() {
            return E_OUTOFMEMORY;
        }

        Query = SysAllocString(RUSTCONV_WSTR_420BAC9386E7.as_ptr());
        if Query.is_null() {
            hr = E_OUTOFMEMORY;
            break 'done;
        }

        hr = IWbemServices_ExecQuery(
            Services,
            Wql,
            Query,
            0,
            core::ptr::null_mut(),
            &raw mut Result,
        );
        if hr != S_OK {
            DPRINT1!("IWbemServices_ExecQuery failed %08x\n", hr);
            break 'done;
        }

        hr = IEnumWbemClassObject_Next(
            Result,
            10000,
            1,
            &raw mut Object,
            &raw mut Count,
        );
        if hr != S_OK {
            DPRINT1!("IEnumWbemClassObject_Next failed %08x\n", hr);
            break 'done;
        }

        hr = IWbemClassObject_BeginEnumeration(Object, 0);
        if hr != S_OK {
            DPRINT1!("IWbemClassObject_BeginEnumeration returned %08x\n", hr);
            break 'done;
        }

        while IWbemClassObject_Next(
            Object,
            0,
            &raw mut Name,
            &raw mut Value,
            &raw mut Type,
            core::ptr::null_mut(),
        ) == S_OK
        {
            DPRINT!("Name: %S\n", Name);
            if _wcsicmp(Name as *const u16, RUSTCONV_WSTR_2A663D6AB2F2.as_ptr()) == 0 {
                DPRINT!("BuildNumber %S\n", V_BSTR(&raw mut Value));
                StringCbCopyW(
                    &raw mut VersionInfoBuildNumber as *mut WCHAR,
                    core::mem::size_of::<[WCHAR; MAX_PATH]>() as size_t,
                    V_BSTR(&raw mut Value),
                );
            } else if _wcsicmp(Name as *const u16, RUSTCONV_WSTR_15D33A6D42D7.as_ptr()) == 0 {
                DPRINT!("OSProductSuite 0x%x\n", V_UINT(&raw mut Value));
                VersionInfoOsProductSuite = V_UINT(&raw mut Value);
            } else if _wcsicmp(Name as *const u16, RUSTCONV_WSTR_9A3C13E25FF3.as_ptr()) == 0 {
                DPRINT!("OSType %u\n", V_UINT(&raw mut Value));
                VersionInfoOsType = V_UINT(&raw mut Value);
            } else if _wcsicmp(Name as *const u16, RUSTCONV_WSTR_D26E7C09CBD9.as_ptr()) == 0 {
                DPRINT!("ServicePackMajorVersion %hu\n", V_UINT(&raw mut Value));
                _swprintf(
                    &raw mut VersionInfoServicePackMajorVersion as *mut WCHAR,
                    RUSTCONV_WSTR_925254771579.as_ptr(),
                    V_UINT(&raw mut Value),
                );
            } else if _wcsicmp(Name as *const u16, RUSTCONV_WSTR_B1DB089221CB.as_ptr()) == 0 {
                DPRINT!("ServicePackMinorVersion %hu\n", V_UINT(&raw mut Value));
                _swprintf(
                    &raw mut VersionInfoServicePackMinorVersion as *mut WCHAR,
                    RUSTCONV_WSTR_925254771579.as_ptr(),
                    V_UINT(&raw mut Value),
                );
            } else if _wcsicmp(Name as *const u16, RUSTCONV_WSTR_1F7A38F3D6FA.as_ptr()) == 0 {
                DPRINT!("Version %S\n", V_BSTR(&raw mut Value));
                StringCbCopyW(
                    &raw mut VersionInfoVersion as *mut WCHAR,
                    core::mem::size_of::<[WCHAR; MAX_PATH]>() as size_t,
                    V_BSTR(&raw mut Value),
                );
            }

            SysFreeString(Name);
            VariantClear(&raw mut Value);
        }

        hr = IWbemClassObject_EndEnumeration(Object);
        if hr != S_OK {
            DPRINT1!("got %08x\n", hr);
            break 'done;
        }
    }

    if !Object.is_null() {
        IWbemClassObject_Release(Object);
    }

    if !Result.is_null() {
        IEnumWbemClassObject_Release(Result);
    }

    if !Query.is_null() {
        SysFreeString(Query);
    }

    if !Wql.is_null() {
        SysFreeString(Wql);
    }

    return hr;
}

unsafe extern "C" fn QueryProcessorInfo(Services: *mut IWbemServices) -> HRESULT {
    let mut Result: *mut IEnumWbemClassObject = core::ptr::null_mut();
    let mut Object: *mut IWbemClassObject = core::ptr::null_mut();
    let mut Wql: BSTR = core::ptr::null_mut();
    let mut Query: BSTR = core::ptr::null_mut();
    let mut Name: BSTR = core::ptr::null_mut();
    let mut Type: CIMTYPE = 0;
    let mut Count: ULONG = 0;
    let mut Value: VARIANT = core::mem::zeroed();
    let mut hr: HRESULT = 0;

    Wql = SysAllocString(RUSTCONV_WSTR_C6B74772B6A4.as_ptr());
    if Wql.is_null() {
        return E_OUTOFMEMORY;
    }

    Query = SysAllocString(RUSTCONV_WSTR_AC4862C373EB.as_ptr());
    'done: {
        if Query.is_null() {
            hr = E_OUTOFMEMORY;
            break 'done;
        }

        hr = IWbemServices_ExecQuery(
            Services,
            Wql,
            Query,
            0,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(Result),
        );
        if hr != S_OK {
            DPRINT1!("IWbemServices_ExecQuery failed %08x\n", hr);
            break 'done;
        }

        hr = IEnumWbemClassObject_Next(
            Result,
            10000,
            1,
            core::ptr::addr_of_mut!(Object),
            core::ptr::addr_of_mut!(Count),
        );
        if hr != S_OK {
            DPRINT1!("IEnumWbemClassObject_Next failed %08x\n", hr);
            break 'done;
        }

        hr = IWbemClassObject_BeginEnumeration(Object, 0);
        if hr != S_OK {
            DPRINT1!("IWbemClassObject_BeginEnumeration returned %08x\n", hr);
            break 'done;
        }

        while IWbemClassObject_Next(
            Object,
            0,
            core::ptr::addr_of_mut!(Name),
            core::ptr::addr_of_mut!(Value),
            core::ptr::addr_of_mut!(Type),
            core::ptr::null_mut(),
        ) == S_OK
        {
            DPRINT!("Name: %S\n", Name);
            if _wcsicmp(Name, RUSTCONV_WSTR_216E1EFDB8A1.as_ptr()) == 0 {
                DPRINT!("Architecture %u\n", V_UINT(core::ptr::addr_of_mut!(Value)));
                VersionInfoArchitecture = V_UINT(core::ptr::addr_of_mut!(Value));
            }

            SysFreeString(Name);
            VariantClear(core::ptr::addr_of_mut!(Value));
        }

        hr = IWbemClassObject_EndEnumeration(Object);
        if hr != S_OK {
            DPRINT1!("got %08x\n", hr);
            break 'done;
        }
    }

    if !Object.is_null() {
        IWbemClassObject_Release(Object);
    }

    if !Result.is_null() {
        IEnumWbemClassObject_Release(Result);
    }

    if !Query.is_null() {
        SysFreeString(Query);
    }

    if !Wql.is_null() {
        SysFreeString(Wql);
    }

    return hr;
}
#[no_mangle]
pub unsafe extern "C" fn GetWmiVersionInfo() -> HRESULT {
    let mut Path: BSTR = core::ptr::null_mut();
    let mut Locator: *mut IWbemLocator = core::ptr::null_mut();
    let mut Services: *mut IWbemServices = core::ptr::null_mut();
    let mut hr: HRESULT = 0;

    CoInitialize(core::ptr::null_mut());

    CoInitializeSecurity(
        core::ptr::null_mut(),
        -1,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        RPC_C_AUTHN_LEVEL_DEFAULT,
        RPC_C_IMP_LEVEL_IMPERSONATE,
        core::ptr::null_mut(),
        EOAC_NONE,
        core::ptr::null_mut(),
    );
    hr = CoCreateInstance(
        core::ptr::addr_of!(CLSID_WbemLocator),
        core::ptr::null_mut(),
        CLSCTX_INPROC_SERVER,
        core::ptr::addr_of!(IID_IWbemLocator),
        core::ptr::addr_of_mut!(Locator) as *mut *mut core::ffi::c_void,
    );
    if hr != S_OK {
        DPRINT1!("Can't create instance of WbemLocator\n");
        return hr;
    }

    Path = SysAllocString(RUSTCONV_WSTR_60F3492FB3CB.as_ptr());
    'done: {
        if Path.is_null() {
            hr = E_OUTOFMEMORY;
            break 'done;
        }

        hr = IWbemLocator_ConnectServer(
            Locator,
            Path,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            0,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(Services),
        );
        if hr != S_OK {
            DPRINT1!("Failed to get IWbemServices interface %08x\n", hr);
            break 'done;
        }

        hr = CoSetProxyBlanket(
            Services as *mut IUnknown,
            RPC_C_AUTHN_WINNT,
            RPC_C_AUTHZ_NONE,
            core::ptr::null_mut(),
            RPC_C_AUTHN_LEVEL_CALL,
            RPC_C_IMP_LEVEL_IMPERSONATE,
            core::ptr::null_mut(),
            EOAC_NONE,
        );
        if hr != S_OK {
            DPRINT1!("Failed to set proxy blanket %08x\n", hr);
            break 'done;
        }

        hr = QueryOperatingSystemInfo(Services);
        if hr == S_OK {
            hr = QueryProcessorInfo(Services);
        }
    }

    if !Path.is_null() {
        SysFreeString(Path);
    }

    if !Services.is_null() {
        IWbemServices_Release(Services);
    }

    if !Locator.is_null() {
        IWbemLocator_Release(Locator);
    }

    CoUninitialize();

    return hr;
}
