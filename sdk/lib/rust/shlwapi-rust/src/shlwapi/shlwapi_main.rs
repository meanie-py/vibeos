use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut shlwapi_hInstance: HINSTANCE =
    ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

#[no_mangle]
pub static mut SHLWAPI_ThreadRef_index: DWORD = TLS_OUT_OF_INDEXES;

// #define NO_SHLWAPI_REG (flag-only macro, no value; used only under #ifdef)
// #define NO_SHLWAPI_STREAM (flag-only macro, no value; used only under #ifdef)

// WINE_DEFAULT_DEBUG_CHANNEL(shell)



// #ifdef __REACTOS__
// extern CRITICAL_SECTION g_csBagCacheLock;  (defined in the propbag translation unit)
// VOID FreeViewStatePropertyBagCache(VOID);  (prototype only; provided by the function pass)

#[no_mangle]
pub unsafe extern "system" fn DllMain(hinstDLL: HINSTANCE, fdwReason: DWORD, fImpLoad: LPVOID) -> BOOL {
    match fdwReason {
        DLL_PROCESS_ATTACH => {
            DisableThreadLibraryCalls(hinstDLL as HMODULE);
            shlwapi_hInstance = hinstDLL;
            SHLWAPI_ThreadRef_index = TlsAlloc();
            InitializeCriticalSection(core::ptr::addr_of_mut!(g_csBagCacheLock));
        }
        DLL_PROCESS_DETACH => {
            if !fImpLoad.is_null() {
                // break
            } else {
                FreeViewStatePropertyBagCache();
                DeleteCriticalSection(core::ptr::addr_of_mut!(g_csBagCacheLock));
                if SHLWAPI_ThreadRef_index != TLS_OUT_OF_INDEXES {
                    TlsFree(SHLWAPI_ThreadRef_index);
                }
            }
        }
        _ => {}
    }
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn DllGetVersion(pdvi: *mut DLLVERSIONINFO) -> HRESULT {
    let mut pdvi2: *mut DLLVERSIONINFO2 = pdvi as *mut DLLVERSIONINFO2;

    if pdvi.is_null() {
        return E_INVALIDARG;
    }
    's_65: {
        let cb: DWORD = (*pdvi2).info1.cbSize;
        if cb == core::mem::size_of::<DLLVERSIONINFO2>() as DWORD {
            (*pdvi2).dwFlags = 0;
            (*pdvi2).ullVersion = MAKEDLLVERULL!(
                6,
                0,
                2800,
                1612,
            );
            /* Fall through */
        } else if cb == core::mem::size_of::<DLLVERSIONINFO>() as DWORD {
            /* case sizeof(core::mem::size_of::<DLLVERSIONINFO>()): no extra work */
        } else {
            break 's_65;
        }
        (*pdvi2).info1.dwMajorVersion = 6;
        (*pdvi2).info1.dwMinorVersion = 0;
        (*pdvi2).info1.dwBuildNumber = 2800;
        (*pdvi2).info1.dwPlatformID = DLLVER_PLATFORM_WINDOWS;
        return S_OK;
    }

    WARN!("pdvi->cbSize = %d, unhandled\n", (*pdvi2).info1.cbSize);
    return E_INVALIDARG;
}
