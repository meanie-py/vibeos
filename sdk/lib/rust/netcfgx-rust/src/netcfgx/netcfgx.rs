use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// precomp.h carries `extern HINSTANCE netcfgx_hInstance;`, so the island emits that
// declaration and every other module binds to it through the prelude glob. This is the
// one definition — it needs the C symbol name, exactly as netcfgx.c's did, or the
// declaration resolves to nothing at link time.
#[no_mangle]
pub static mut netcfgx_hInstance: HINSTANCE = core::ptr::null_mut();

pub const CLSID_TcpipConfigNotifyObject: GUID = GUID {
    Data1: 0xA907657F,
    Data2: 0x6FDF,
    Data3: 0x11D0,
    Data4: [0x8E, 0xFB, 0x00, 0xC0, 0x4F, 0xD9, 0x12, 0xB2],
};

pub static mut InterfaceTable: [INTERFACE_TABLE; 3] = [
    INTERFACE_TABLE {
        riid: unsafe { &CLSID_CNetCfg },
        lpfnCI: Some(INetCfg_Constructor),
    },
    INTERFACE_TABLE {
        riid: unsafe { &CLSID_TcpipConfigNotifyObject },
        lpfnCI: Some(TcpipConfigNotify_Constructor),
    },
    INTERFACE_TABLE {
        riid: core::ptr::null(),
        lpfnCI: None,
    },
];

#[no_mangle]
pub unsafe extern "system" fn DllMain(
    hinstDLL: HINSTANCE,
    fdwReason: u32,
    fImpLoad: LPVOID,
) -> i32 {
    match fdwReason {
        DLL_PROCESS_ATTACH => {
            netcfgx_hInstance = hinstDLL;
            DisableThreadLibraryCalls(netcfgx_hInstance);
            InitCommonControls();
        }
        _ => {}
    }

    (TRUE) as i32
}

#[no_mangle]
pub unsafe extern "system" fn DllCanUnloadNow() -> HRESULT {
    S_FALSE
}

#[no_mangle]
pub unsafe extern "system" fn DllRegisterServer() -> HRESULT {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut hSubKey: HKEY = core::ptr::null_mut();
    let mut pStr: *mut u16 = core::ptr::null_mut();
    let mut szName: [u16; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];

    let clsid_str = [
        'C' as u16, 'L' as u16, 'S' as u16, 'I' as u16, 'D' as u16, '\\' as u16, 0
    ];
    for i in 0..6 {
        szName[i] = clsid_str[i];
    }

    if StringFromCLSID(
        &CLSID_CNetCfg as *const GUID,
        &mut pStr as *mut *mut u16
    ) < 0
    {
        return SELFREG_E_CLASS;
    }

    wcscpy(szName.as_mut_ptr().add(6), pStr);
    CoTaskMemFree(pStr as LPVOID);

    if RegCreateKeyExW(
        HKEY_CLASSES_ROOT,
        szName.as_ptr(),
        0,
        core::ptr::null_mut(),
        0,
        KEY_WRITE,
        core::ptr::null_mut(),
        &mut hKey,
        core::ptr::null_mut()
    ) != (ERROR_SUCCESS) as i32
    {
        return SELFREG_E_CLASS;
    }

    if RegCreateKeyExW(
        hKey,
        ['I' as u16, 'n' as u16, 'P' as u16, 'r' as u16, 'o' as u16, 'c' as u16,
         'S' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'e' as u16, 'r' as u16,
         '3' as u16, '2' as u16, 0].as_ptr(),
        0,
        core::ptr::null_mut(),
        0,
        KEY_WRITE,
        core::ptr::null_mut(),
        &mut hSubKey,
        core::ptr::null_mut()
    ) == (ERROR_SUCCESS) as i32
    {
        if GetModuleFileNameW(
            netcfgx_hInstance,
            szName.as_mut_ptr(),
            (core::mem::size_of_val(&szName) / core::mem::size_of::<u16>()) as u32
        ) == 0
        {
            RegCloseKey(hSubKey);
            RegCloseKey(hKey);
            return SELFREG_E_CLASS;
        }
        szName[(core::mem::size_of_val(&szName) / core::mem::size_of::<u16>()) - 1] = 0;
        RegSetValueW(
            hSubKey,
            core::ptr::null_mut(),
            (REG_SZ) as u32,
            szName.as_ptr(),
            ((wcslen(szName.as_ptr()) + 1) * core::mem::size_of::<u16>()) as u32
        );
        RegSetValueExW(
            hSubKey,
            ['T' as u16, 'h' as u16, 'r' as u16, 'e' as u16, 'a' as u16, 'd' as u16,
             'i' as u16, 'n' as u16, 'g' as u16, 'M' as u16, 'o' as u16, 'd' as u16,
             'e' as u16, 'l' as u16, 0].as_ptr(),
            0,
            (REG_SZ) as u32,
            ['B' as u16, 'o' as u16, 't' as u16, 'h' as u16, 0].as_ptr() as LPBYTE,
            10
        );
        RegCloseKey(hSubKey);
    }

    RegCloseKey(hKey);
    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn DllUnregisterServer() -> HRESULT {
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn DllGetClassObject(
    rclsid: *const GUID,
    riid: *const GUID,
    ppv: *mut LPVOID,
) -> HRESULT {
    let mut i: u32;
    let mut hres: HRESULT = E_OUTOFMEMORY;
    let mut pcf: *mut IClassFactory = core::ptr::null_mut();

    if ppv.is_null() {
        return E_INVALIDARG;
    }

    *ppv = core::ptr::null_mut();

    i = 0;
    while !(*InterfaceTable.as_ptr().add(i as usize)).riid.is_null() {
        if IsEqualIID!((*InterfaceTable.as_ptr().add(i as usize)).riid, rclsid) {
            pcf = IClassFactory_fnConstructor((*InterfaceTable.as_ptr().add(i as usize)).lpfnCI, core::ptr::null_mut(), core::ptr::null_mut());
            break;
        }
        i += 1;
    }

    if pcf.is_null() {
        return CLASS_E_CLASSNOTAVAILABLE;
    }

    hres = IClassFactory_QueryInterface(pcf, riid, ppv);
    IClassFactory_Release(pcf);

    hres
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS Configuration of network devices
//  * FILE:            dll/win32/netcfgx/netcfgx.c
//  * PURPOSE:         Network devices installer
//  *
//  * PROGRAMMERS:     Herv� Poussineau (hpoussin@reactos.org)
//  */
//
// #include "precomp.h"
//
// #include <olectl.h>
//
//
// HINSTANCE netcfgx_hInstance;
// const GUID CLSID_TcpipConfigNotifyObject      = {0xA907657F, 0x6FDF, 0x11D0, {0x8E, 0xFB, 0x00, 0xC0, 0x4F, 0xD9, 0x12, 0xB2}};
//
// static INTERFACE_TABLE InterfaceTable[] =
// {
//     {
//         &CLSID_CNetCfg,
//         INetCfg_Constructor
//     },
//     {
//         &CLSID_TcpipConfigNotifyObject,
//         TcpipConfigNotify_Constructor
//     },
//     {
//         NULL,
//         NULL
//     }
// };
//
// BOOL
// WINAPI
// DllMain(HINSTANCE hinstDLL, DWORD fdwReason, LPVOID fImpLoad)
// {
//     switch (fdwReason)
//     {
//         case DLL_PROCESS_ATTACH:
//             netcfgx_hInstance = hinstDLL;
//             DisableThreadLibraryCalls(netcfgx_hInstance);
//             InitCommonControls();
//             break;
//
//         default:
//             break;
//     }
//
//     return TRUE;
// }
//
// HRESULT
// WINAPI
// DllCanUnloadNow(void)
// {
//     return S_FALSE;
// }
//
// extern HRESULT __stdcall
// DllRegisterServer(void)
// {
//     HKEY hKey, hSubKey;
//     LPOLESTR pStr;
//     WCHAR szName[MAX_PATH] = L"CLSID\\";
//
//     if (FAILED(StringFromCLSID(&CLSID_CNetCfg, &pStr)))
//         return SELFREG_E_CLASS;
//
//     wcscpy(&szName[6], pStr);
//     CoTaskMemFree(pStr);
//
//     if (RegCreateKeyExW(HKEY_CLASSES_ROOT, szName, 0, NULL, 0, KEY_WRITE, NULL, &hKey, NULL) != ERROR_SUCCESS)
//         return SELFREG_E_CLASS;
//
//     if (RegCreateKeyExW(hKey, L"InProcServer32", 0, NULL, 0, KEY_WRITE, NULL, &hSubKey, NULL) == ERROR_SUCCESS)
//     {
//         if (!GetModuleFileNameW(netcfgx_hInstance, szName, sizeof(szName)/sizeof(WCHAR)))
//         {
//             RegCloseKey(hSubKey);
//             RegCloseKey(hKey);
//             return SELFREG_E_CLASS;
//         }
//         szName[(sizeof(szName)/sizeof(WCHAR))-1] = L'\0';
//         RegSetValueW(hSubKey, NULL, REG_SZ, szName, (wcslen(szName)+1) * sizeof(WCHAR));
//         RegSetValueExW(hSubKey, L"ThreadingModel", 0, REG_SZ, (LPBYTE)L"Both", 10);
//         RegCloseKey(hSubKey);
//     }
//
//     RegCloseKey(hKey);
//     return S_OK;
// }
//
// extern HRESULT __stdcall
// DllUnregisterServer(void)
// {
//     //FIXME
//     // implement unregistering services
//     //
//     return S_OK;
// }
//
// extern HRESULT __stdcall
// DllGetClassObject(
//     REFCLSID rclsid,
//     REFIID riid,
//     LPVOID* ppv)
// {
//     UINT i;
//     HRESULT hres = E_OUTOFMEMORY;
//     IClassFactory * pcf = NULL;
//
//     if (!ppv)
//         return E_INVALIDARG;
//
//     *ppv = NULL;
//
//     for (i = 0; InterfaceTable[i].riid; i++)
//     {
//         if (IsEqualIID(InterfaceTable[i].riid, rclsid))
//         {
//             pcf = IClassFactory_fnConstructor(InterfaceTable[i].lpfnCI, NULL, NULL);
//             break;
//         }
//     }
//
//     if (!pcf)
//     {
//         return CLASS_E_CLASSNOTAVAILABLE;
//     }
//
//     hres = IClassFactory_QueryInterface(pcf, riid, ppv);
//     IClassFactory_Release(pcf);
//
//     return hres;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
