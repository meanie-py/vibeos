use crate::prelude::*;
use crate::shell32::IDefClFImpl::IDefClFImpl;
use crate::shell32::IDefClFImpl::IDefClFImpl_CreateInstance;
use crate::def_view_support::IsEqualCLSID;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_F3C6A8F2AC47: [WCHAR; 4] = [0x0025, 0x006C, 0x0075, 0x0000];

use core::ffi::c_void;
use crate::shell32::IDefClFImpl::IID_IClassFactory;
use crate::shell32::utils_support::LOCALE_USER_DEFAULT;

// ---- file-scope preamble (translated) ----
// shell32.cpp:132 `typedef HRESULT (CALLBACK *LPFNCREATEINSTANCE)(IUnknown* pUnkOuter, REFIID riid,
// LPVOID* ppvObject)`. CALLBACK is stdcall, hence "system". The translation had the outer
// unknown BY VALUE and `LPVOID` for `LPVOID*`, which hands the callee the vtable pointer where
// the object pointer belongs — wrong on every arch.
pub type LPFNCREATEINSTANCE =
    Option<unsafe extern "system" fn(*mut IUnknown, REFIID, *mut LPVOID) -> HRESULT>;

// winnls.h LOCALE_USER_DEFAULT is MAKELCID(LANG_USER_DEFAULT, SORT_DEFAULT) — a macro, so not
// in the island. utils_support.rs already derives it.
// The class factory's own island-typed IID declaration.

/// shlwapi.h `MAKEDLLVERULL(major, minor, build, qfe)` — a macro: four 16-bit fields packed
/// into a ULONGLONG, major in the top word.
#[inline]
fn MAKEDLLVERULL(major: DWORD, minor: DWORD, build: DWORD, qfe: DWORD) -> ULONGLONG {
    ((major as ULONGLONG & 0xFFFF) << 48)
        | ((minor as ULONGLONG & 0xFFFF) << 32)
        | ((build as ULONGLONG & 0xFFFF) << 16)
        | (qfe as ULONGLONG & 0xFFFF)
}

#[no_mangle]
pub unsafe extern "system" fn AddCommasW(lValue: u32, lpNumber: PWSTR) -> PWSTR {
    let mut szValue = [0u16; MAX_PATH as usize];
    let mut szSeparator = [0u16; 8 + 1];
    let mut szGrouping = [0u16; 10 + 1];
    let mut numFormat: NUMBERFMTW = core::mem::zeroed();

    GetLocaleInfoW(
        LOCALE_USER_DEFAULT,
        LOCALE_STHOUSAND,
        szSeparator.as_mut_ptr(),
        szSeparator.len() as INT
    );

    let mut dwGrouping: u32 = 3;
    if GetLocaleInfoW(
        LOCALE_USER_DEFAULT,
        LOCALE_SGROUPING,
        szGrouping.as_mut_ptr(),
        szGrouping.len() as INT
    ) != 0 {
        let mut p: PWSTR = szGrouping.as_mut_ptr();
        dwGrouping = 0;
        while *p != 0 {
            if *p >= 0x0031u16 && *p <= 0x0039u16 {
                dwGrouping = dwGrouping * 10 + ((*p - 0x0030u16) as u32);
            }
            p = p.add(1);
        }
        if p > szGrouping.as_mut_ptr() && *p.sub(1) != 0x0030u16 {
            dwGrouping *= 10;
        }
    }

    numFormat.NumDigits = 0;
    numFormat.LeadingZero = 0;
    numFormat.Grouping = dwGrouping;
    numFormat.lpDecimalSep = szSeparator.as_mut_ptr();
    numFormat.lpThousandSep = szSeparator.as_mut_ptr();
    numFormat.NegativeOrder = 0;

    _swprintf(
        szValue.as_mut_ptr(),
        CPP_COM_WSTR_F3C6A8F2AC47.as_ptr(),
        lValue
    );

    if GetNumberFormatW(
        LOCALE_USER_DEFAULT,
        0,
        szValue.as_ptr(),
        &numFormat,
        lpNumber,
        MAX_PATH as INT
    ) != 0 {
        return lpNumber;
    }

    wcscpy(
        lpNumber,
        szValue.as_ptr()
    );

    return lpNumber;
}
#[no_mangle]
pub unsafe extern "system" fn RegenerateUserEnvironment(
    lpEnvironment: *mut *mut core::ffi::c_void,
    bUpdateSelf: i32,
) -> i32 {
    // Upstream c25e4fd7dec (Wine/Windows behaviour): the right token rights, no handle or
    // block leaks on any path, `=X:` variables preserved, and variables the new block no
    // longer has are deleted from the process.
    let mut hUserToken: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut pEnv: *mut core::ffi::c_void = core::ptr::null_mut();

    if lpEnvironment.is_null() {
        return FALSE;
    }

    if OpenProcessToken(
        GetCurrentProcess(),
        TOKEN_QUERY | TOKEN_DUPLICATE | TOKEN_ASSIGN_PRIMARY,
        &mut hUserToken,
    ) == 0
    {
        return FALSE;
    }

    if CreateEnvironmentBlock(&mut pEnv, hUserToken, TRUE) == 0 {
        CloseHandle(hUserToken);
        return FALSE;
    }

    *lpEnvironment = pEnv;
    let bResult: i32 = TRUE;

    if bUpdateSelf != 0 {
        let mut pszz = pEnv as PWSTR;
        while !pszz.is_null() && *pszz != 0 {
            if *pszz != 0x003D {
                let pchEqual: PWSTR = wcschr(pszz, 0x003Du16);
                if !pchEqual.is_null() {
                    *pchEqual = 0;
                    SetEnvironmentVariableW(pszz, pchEqual.add(1));
                    *pchEqual = 0x003D;
                }
            }
            pszz = pszz.add(wcslen(pszz) + 1);
        }

        let pCurEnv: PWSTR = GetEnvironmentStringsW();
        if !pCurEnv.is_null() {
            let mut pCur = pCurEnv;
            while *pCur != 0 {
                if *pCur != 0x003D {
                    let pchEqual: PWSTR = wcschr(pCur, 0x003Du16);
                    if !pchEqual.is_null() {
                        let name_len = pchEqual.offset_from(pCur) as usize;
                        // The C keeps the new block's names upper-cased in a CSimpleMap; a
                        // case-insensitive match on the name scans the same block.
                        if !env_block_has_name(pEnv as PWSTR, pCur, name_len) {
                            *pchEqual = 0;
                            SetEnvironmentVariableW(pCur, core::ptr::null());
                            *pchEqual = 0x003D;
                        }
                    }
                }
                pCur = pCur.add(wcslen(pCur) + 1);
            }
            FreeEnvironmentStringsW(pCurEnv);
        }

        // We free the environment block now that we have applied it to the process;
        // the caller gets NULL back in *lpEnvironment in this case.
        DestroyEnvironmentBlock(pEnv);
        *lpEnvironment = core::ptr::null_mut();
    }

    CloseHandle(hUserToken);
    bResult
}

/// Whether `block` (a `NAME=value\0...\0` environment) defines `name[..len]`, ignoring case
/// and the `=X:` entries.
unsafe fn env_block_has_name(block: PWSTR, name: PWSTR, len: usize) -> bool {
    let mut p = block;
    while *p != 0 {
        let n = wcslen(p);
        if *p != 0x003D && n > len && *p.add(len) == 0x003D && _wcsnicmp(p, name, len) == 0 {
            return true;
        }
        p = p.add(n + 1);
    }
    false
}
#[no_mangle]
pub unsafe extern "C" fn IDefClF_fnConstructor(
    lpfnCI: LPFNCREATEINSTANCE,
    pcRefDll: *mut LONG,
    riidInst: *const IID,
    theFactory: *mut *mut IClassFactory,
) -> HRESULT {
    IDefClFImpl_CreateInstance(
        lpfnCI,
        pcRefDll,
        riidInst,
        &IID_IClassFactory,
        theFactory as *mut *mut core::ffi::c_void,
    )
}

#[no_mangle]
pub unsafe extern "system" fn SHCreateDefClassObject(
    riid: REFIID,
    ppv: *mut *mut core::ffi::c_void,
    lpfnCI: LPFNCREATEINSTANCE,
    pcRefDll: *mut u32,
    riidInst: REFIID,
) -> HRESULT {
    let mut pcf: *mut IClassFactory = core::ptr::null_mut();
    let hResult: HRESULT;

    if IsEqualCLSID(riid, &IID_IClassFactory) == 0 {
        return E_NOINTERFACE;
    }
    hResult = IDefClF_fnConstructor(
        lpfnCI,
        pcRefDll as *mut i32,
        riidInst,
        &mut pcf as *mut *mut IClassFactory,
    );
    if hResult < 0 {
        return hResult;
    }
    *ppv = pcf as *mut core::ffi::c_void;
    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn DllGetVersion(pdvi: *mut DLLVERSIONINFO) -> HRESULT {
    if (*pdvi).cbSize == core::mem::size_of::<DLLVERSIONINFO>() as u32
        || (*pdvi).cbSize == core::mem::size_of::<DLLVERSIONINFO2>() as u32
    {
        (*pdvi).dwMajorVersion = WINE_FILEVERSION_MAJOR;
        (*pdvi).dwMinorVersion = WINE_FILEVERSION_MINOR;
        (*pdvi).dwBuildNumber = WINE_FILEVERSION_BUILD;
        (*pdvi).dwPlatformID = WINE_FILEVERSION_PLATFORMID;
        if (*pdvi).cbSize == core::mem::size_of::<DLLVERSIONINFO2>() as u32 {
            let pdvi2 = pdvi as *mut DLLVERSIONINFO2;
            (*pdvi2).dwFlags = 0;
            (*pdvi2).ullVersion = MAKEDLLVERULL(
                WINE_FILEVERSION_MAJOR,
                WINE_FILEVERSION_MINOR,
                WINE_FILEVERSION_BUILD,
                WINE_FILEVERSION_PLATFORMID,
            );
        }
        return S_OK;
    } else {
        return E_INVALIDARG;
    }
}

#[no_mangle]
pub unsafe extern "system" fn DllInstall(bInstall: BOOL, cmdline: LPCWSTR) -> HRESULT {
    // FIXME("%s %s: stub\n", bInstall ? "TRUE":"FALSE", debugstr_w(cmdline));
    S_OK
}
