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
    let mut hUserToken: *mut core::ffi::c_void = core::ptr::null_mut();
    if OpenProcessToken(GetCurrentProcess(), TOKEN_READ | TOKEN_WRITE, &mut hUserToken) == 0 {
        return FALSE;
    }

    let bResult: i32 = CreateEnvironmentBlock(lpEnvironment, hUserToken, TRUE);
    if bResult == 0 || lpEnvironment.is_null() {
        CloseHandle(hUserToken);
        return FALSE;
    }

    if bUpdateSelf != 0 {
        let mut pszz: PWSTR = (*lpEnvironment) as PWSTR;
        if pszz.is_null() {
            return FALSE;
        }

        while *pszz != 0 {
            let cch: usize = wcslen(pszz);
            let pchEqual: PWSTR = wcschr(pszz, 0x003Du16);
            if !pchEqual.is_null() {
                // C++: `CStringW strName(pszz, pchEqual - pszz)` — a length in CHARACTERS. The
                // translation subtracted the addresses as integers, i.e. BYTES, and then copied
                // that many UTF-16 units into a fixed MAX_PATH buffer: twice the name, so every
                // variable came out as `NAME=value…`, and a long one overran the buffer.
                // `offset_from` counts elements; CStringW has no fixed cap, like the original.
                let name_len = pchEqual.offset_from(pszz) as i32;
                let mut strName = CStringW::new();
                strName.SetString(pszz, name_len);
                SetEnvironmentVariableW(strName.as_ptr(), pchEqual.add(1 as usize));
            }
            pszz = pszz.add(cch + 1 as usize);
        }
    }

    CloseHandle(hUserToken);

    bResult
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
