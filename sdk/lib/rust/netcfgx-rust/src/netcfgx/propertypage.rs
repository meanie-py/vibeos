use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum _PARAM_TYPE {
    NO_TYPE = 0,
    INT_TYPE = 1,
    LONG_TYPE = 2,
    WORD_TYPE = 3,
    DWORD_TYPE = 4,
    EDIT_TYPE = 5,
    ENUM_TYPE = 6,
}
pub use _PARAM_TYPE as PARAM_TYPE;
// The C enum's members are unqualified names in every TU that sees the typedef;
// a Rust enum scopes them to the type, so re-export them to match the C.
pub use _PARAM_TYPE::*;
pub type PPARAM_TYPE = *mut PARAM_TYPE;

#[repr(C)]
pub struct _ENUM_OPTION {
    pub pszValue: PWSTR,
    pub pszName: PWSTR,
}
pub use _ENUM_OPTION as ENUM_OPTION;
pub type PENUM_OPTION = *mut ENUM_OPTION;

#[repr(C)]
pub struct _PARAMETER {
    pub pszName: PWSTR,
    pub pszDescription: PWSTR,
    pub pszValue: PWSTR,
    pub cchValueLength: DWORD,
    pub pszDefault: PWSTR,
    pub bOptional: BOOL,
    pub bPresent: BOOL,
    pub Type: PARAM_TYPE,
    pub dwEnumOptions: DWORD,
    pub pEnumOptions: PENUM_OPTION,
    pub bUpperCase: BOOL,
    pub iTextLimit: INT,
    pub iBase: INT,
    pub iStep: INT,
    pub u: _PARAMETER__bindgen_ty_1,
}

#[repr(C)]
pub union _PARAMETER__bindgen_ty_1 {
    pub l: _PARAMETER__bindgen_ty_1__bindgen_ty_1,
    pub dw: _PARAMETER__bindgen_ty_1__bindgen_ty_2,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct _PARAMETER__bindgen_ty_1__bindgen_ty_1 {
    pub lMin: LONG,
    pub lMax: LONG,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct _PARAMETER__bindgen_ty_1__bindgen_ty_2 {
    pub dwMin: DWORD,
    pub dwMax: DWORD,
}

pub use _PARAMETER as PARAMETER;
pub type PPARAMETER = *mut PARAMETER;

#[repr(C)]
pub struct _PARAMETER_ARRAY {
    pub DeviceInfoSet: HDEVINFO,
    pub DeviceInfoData: PSP_DEVINFO_DATA,
    pub pCurrentParam: PPARAMETER,
    pub dwCount: DWORD,
    pub Array: [PARAMETER; 0],
}
pub use _PARAMETER_ARRAY as PARAMETER_ARRAY;
pub type PPARAMETER_ARRAY = *mut PARAMETER_ARRAY;
unsafe extern "C" fn FreeParameterArray(
    ParamArray: PPARAMETER_ARRAY) {
    let mut i: INT;
    let mut j: INT;

    if ParamArray.is_null() {
        return;
    }

    i = 0;
    while i < (*ParamArray).dwCount as INT {
        if !(*ParamArray).Array[i as usize].pszName.is_null() {
            HeapFree(GetProcessHeap(), 0, (*ParamArray).Array[i as usize].pszName as *mut _);
        }

        if !(*ParamArray).Array[i as usize].pszDescription.is_null() {
            HeapFree(GetProcessHeap(), 0, (*ParamArray).Array[i as usize].pszDescription as *mut _);
        }

        if !(*ParamArray).Array[i as usize].pszDefault.is_null() {
            HeapFree(GetProcessHeap(), 0, (*ParamArray).Array[i as usize].pszDefault as *mut _);
        }

        if !(*ParamArray).Array[i as usize].pEnumOptions.is_null() {
            j = 0;
            while j < (*ParamArray).Array[i as usize].dwEnumOptions as INT {
                if !(*(*ParamArray).Array[i as usize].pEnumOptions.offset(j as isize)).pszValue.is_null() {
                    HeapFree(GetProcessHeap(), 0, (*(*ParamArray).Array[i as usize].pEnumOptions.offset(j as isize)).pszValue as *mut _);
                }

                if !(*(*ParamArray).Array[i as usize].pEnumOptions.offset(j as isize)).pszName.is_null() {
                    HeapFree(GetProcessHeap(), 0, (*(*ParamArray).Array[i as usize].pEnumOptions.offset(j as isize)).pszName as *mut _);
                }
                j += 1;
            }

            HeapFree(GetProcessHeap(), 0, (*ParamArray).Array[i as usize].pEnumOptions as *mut _);
        }
        i += 1;
    }

    HeapFree(GetProcessHeap(), 0, ParamArray as *mut _);
}
unsafe extern "C" fn GetStringValue(
    hKey: HKEY,
    pValueName: PWSTR,
    pString: *mut PWSTR,
    pdwStringLength: PDWORD,
) -> DWORD {
    let mut pBuffer: PWSTR;
    let mut dwLength: DWORD = 0;
    let mut dwRegType: DWORD = 0;
    let dwError: DWORD;

    *pString = core::ptr::null_mut();

    RegQueryValueExW(hKey, pValueName, core::ptr::null_mut(), &mut dwRegType, core::ptr::null_mut(), &mut dwLength);

    if dwLength == 0 || dwRegType != (REG_SZ) as u32 {
        return ERROR_FILE_NOT_FOUND;
    }

    pBuffer = HeapAlloc(GetProcessHeap(), 0, (dwLength + core::mem::size_of::<u16>() as u32) as SIZE_T) as PWSTR;
    if pBuffer.is_null() {
        return ERROR_NOT_ENOUGH_MEMORY;
    }

    dwError = (RegQueryValueExW(hKey, pValueName, core::ptr::null_mut(), core::ptr::null_mut(), pBuffer as LPBYTE, &mut dwLength)) as u32;
    if dwError != ERROR_SUCCESS {
        HeapFree(GetProcessHeap(), 0, pBuffer as *mut _);
        return dwError;
    }

    *pBuffer.offset((dwLength / core::mem::size_of::<u16>() as u32) as isize) = UNICODE_NULL;

    *pString = pBuffer;
    if !pdwStringLength.is_null() {
        *pdwStringLength = dwLength;
    }

    ERROR_SUCCESS
}
unsafe extern "C" fn GetBooleanValue(
    hKey: HKEY,
    pValueName: PWSTR,
    bDefault: BOOL,
    pValue: PBOOL,
) -> DWORD {
    let mut szBuffer: [u16; 16] = [0; 16];
    let mut dwLength: DWORD = 0;
    let mut dwRegType: DWORD = 0;

    *pValue = bDefault;

    dwLength = core::mem::size_of_val(&szBuffer) as DWORD;
    RegQueryValueExW(
        hKey,
        pValueName,
        core::ptr::null_mut(),
        &mut dwRegType,
        szBuffer.as_mut_ptr() as LPBYTE,
        &mut dwLength,
    );

    if dwRegType == (REG_SZ) as u32 && dwLength >= core::mem::size_of::<u16>() as DWORD {
        if szBuffer[0] == 0x0030 {
            *pValue = (FALSE) as i32;
        } else {
            *pValue = (TRUE) as i32;
        }
    }

    ERROR_SUCCESS
}
unsafe extern "C" fn GetIntValue(
    hKey: HKEY,
    pValueName: PWSTR,
    iDefault: INT,
    pValue: PINT,
) -> DWORD {
    let mut szBuffer: [u16; 24] = [0; 24];
    let mut dwLength: DWORD = 0;
    let mut dwRegType: DWORD = 0;

    *pValue = iDefault;

    dwLength = core::mem::size_of_val(&szBuffer) as DWORD;
    RegQueryValueExW(
        hKey,
        pValueName,
        core::ptr::null_mut(),
        &mut dwRegType,
        szBuffer.as_mut_ptr() as LPBYTE,
        &mut dwLength,
    );

    if dwRegType == (REG_SZ) as u32 && dwLength >= core::mem::size_of::<u16>() as DWORD {
        *pValue = _wtoi(szBuffer.as_ptr());
    }

    return ERROR_SUCCESS;
}
unsafe extern "C" fn GetLongValue(
    hKey: HKEY,
    pValueName: PWSTR,
    lDefault: i32,
    pValue: *mut i32,
) -> u32 {
    let mut szBuffer: [u16; 24] = [0; 24];
    let mut dwLength: u32 = 0;
    let mut dwRegType: u32 = 0;
    let mut ptr: PWSTR = core::ptr::null_mut();

    dwLength = (core::mem::size_of_val(&szBuffer)) as u32;
    RegQueryValueExW(
        hKey,
        pValueName,
        core::ptr::null_mut(),
        &mut dwRegType,
        szBuffer.as_mut_ptr() as LPBYTE,
        &mut dwLength,
    );

    if dwRegType == (REG_SZ) as u32 && dwLength >= core::mem::size_of::<u16>() as u32 {
        *pValue = wcstol(szBuffer.as_ptr(), &mut ptr, 10);
        if *pValue == 0 && !ptr.is_null() {
            *pValue = lDefault;
        }
    } else {
        *pValue = lDefault;
    }

    ERROR_SUCCESS
}
unsafe extern "C" fn GetDWordValue(
    hKey: HKEY,
    pValueName: PWSTR,
    dwDefault: DWORD,
    pValue: PDWORD,
) -> DWORD {
    let mut szBuffer: [u16; 24] = [0; 24];
    let mut dwLength: DWORD = 0;
    let mut dwRegType: DWORD = 0;
    let mut ptr: PWSTR = core::ptr::null_mut();

    dwLength = core::mem::size_of_val(&szBuffer) as DWORD;
    RegQueryValueExW(
        hKey,
        pValueName,
        core::ptr::null_mut(),
        &mut dwRegType,
        szBuffer.as_mut_ptr() as LPBYTE,
        &mut dwLength,
    );

    if dwRegType == (REG_SZ) as u32 && dwLength >= core::mem::size_of::<u16>() as DWORD {
        *pValue = wcstoul(szBuffer.as_ptr(), &mut ptr, 10);
        if *pValue == 0 && !ptr.is_null() {
            *pValue = dwDefault;
        }
    } else {
        *pValue = dwDefault;
    }

    ERROR_SUCCESS
}
unsafe extern "C" fn GetEnumOptions(
    hKey: HKEY,
    pParameter: *mut PARAMETER,
) -> DWORD {
    let mut hEnumKey: HKEY = core::ptr::null_mut();
    let mut pOptions: *mut ENUM_OPTION = core::ptr::null_mut();
    let mut dwValues: DWORD = 0;
    let mut dwMaxValueNameLen: DWORD = 0;
    let mut dwMaxValueLen: DWORD = 0;
    let mut dwValueNameLength: DWORD;
    let mut dwValueLength: DWORD;
    let mut i: DWORD;
    let mut dwError: DWORD;

    dwError = (RegOpenKeyExW(
        hKey,
        w!("enum") as PWSTR,
        0,
        KEY_READ,
        &mut hEnumKey,
    )) as u32;
    if dwError != ERROR_SUCCESS {
        return dwError;
    }

    dwError = (RegQueryInfoKeyW(
        hEnumKey,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &mut dwValues,
        &mut dwMaxValueNameLen,
        &mut dwMaxValueLen,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    )) as u32;
    if dwError != ERROR_SUCCESS {
        'done: loop {
            if !pOptions.is_null() {
                i = 0;
                while i < dwValues {
                    if !(*pOptions.add(i as usize)).pszValue.is_null() {
                        HeapFree(GetProcessHeap(), 0, (*pOptions.add(i as usize)).pszValue as *mut _);
                    }
                    if !(*pOptions.add(i as usize)).pszName.is_null() {
                        HeapFree(GetProcessHeap(), 0, (*pOptions.add(i as usize)).pszName as *mut _);
                    }
                    i = i.wrapping_add(1);
                }
                HeapFree(GetProcessHeap(), 0, pOptions as *mut _);
            }
            if !hEnumKey.is_null() {
                RegCloseKey(hEnumKey);
            }
            break 'done dwError;
        };
        return dwError;
    }

    pOptions = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        (dwValues.wrapping_mul(core::mem::size_of::<ENUM_OPTION>() as DWORD)) as SIZE_T,
    ) as *mut ENUM_OPTION;
    if pOptions.is_null() {
        dwError = ERROR_OUTOFMEMORY;
        'done: loop {
            if !pOptions.is_null() {
                i = 0;
                while i < dwValues {
                    if !(*pOptions.add(i as usize)).pszValue.is_null() {
                        HeapFree(GetProcessHeap(), 0, (*pOptions.add(i as usize)).pszValue as *mut _);
                    }
                    if !(*pOptions.add(i as usize)).pszName.is_null() {
                        HeapFree(GetProcessHeap(), 0, (*pOptions.add(i as usize)).pszName as *mut _);
                    }
                    i = i.wrapping_add(1);
                }
                HeapFree(GetProcessHeap(), 0, pOptions as *mut _);
            }
            if !hEnumKey.is_null() {
                RegCloseKey(hEnumKey);
            }
            break 'done dwError;
        };
        return dwError;
    }

    i = 0;
    while i < dwValues {
        dwValueNameLength = dwMaxValueNameLen.wrapping_add(core::mem::size_of::<u16>() as DWORD);
        (*pOptions.add(i as usize)).pszValue = HeapAlloc(
            GetProcessHeap(),
            0,
            (dwValueNameLength.wrapping_mul(core::mem::size_of::<u16>() as DWORD)) as SIZE_T,
        ) as PWSTR;
        if (*pOptions.add(i as usize)).pszValue.is_null() {
            dwError = ERROR_OUTOFMEMORY;
            'done: loop {
                if !pOptions.is_null() {
                    i = 0;
                    while i < dwValues {
                        if !(*pOptions.add(i as usize)).pszValue.is_null() {
                            HeapFree(GetProcessHeap(), 0, (*pOptions.add(i as usize)).pszValue as *mut _);
                        }
                        if !(*pOptions.add(i as usize)).pszName.is_null() {
                            HeapFree(GetProcessHeap(), 0, (*pOptions.add(i as usize)).pszName as *mut _);
                        }
                        i = i.wrapping_add(1);
                    }
                    HeapFree(GetProcessHeap(), 0, pOptions as *mut _);
                }
                if !hEnumKey.is_null() {
                    RegCloseKey(hEnumKey);
                }
                break 'done dwError;
            };
            return dwError;
        }

        dwValueLength = dwMaxValueLen;
        (*pOptions.add(i as usize)).pszName = HeapAlloc(
            GetProcessHeap(),
            0,
            (dwValueLength) as SIZE_T,
        ) as PWSTR;
        if (*pOptions.add(i as usize)).pszName.is_null() {
            dwError = ERROR_OUTOFMEMORY;
            'done: loop {
                if !pOptions.is_null() {
                    i = 0;
                    while i < dwValues {
                        if !(*pOptions.add(i as usize)).pszValue.is_null() {
                            HeapFree(GetProcessHeap(), 0, (*pOptions.add(i as usize)).pszValue as *mut _);
                        }
                        if !(*pOptions.add(i as usize)).pszName.is_null() {
                            HeapFree(GetProcessHeap(), 0, (*pOptions.add(i as usize)).pszName as *mut _);
                        }
                        i = i.wrapping_add(1);
                    }
                    HeapFree(GetProcessHeap(), 0, pOptions as *mut _);
                }
                if !hEnumKey.is_null() {
                    RegCloseKey(hEnumKey);
                }
                break 'done dwError;
            };
            return dwError;
        }

        dwError = (RegEnumValueW(
            hEnumKey,
            i,
            (*pOptions.add(i as usize)).pszValue,
            &mut dwValueNameLength,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            (*pOptions.add(i as usize)).pszName as LPBYTE,
            &mut dwValueLength,
        )) as u32;
        if dwError == ERROR_NO_MORE_ITEMS {
            dwError = ERROR_SUCCESS;
            'done: loop {
                if !pOptions.is_null() {
                    i = 0;
                    while i < dwValues {
                        if !(*pOptions.add(i as usize)).pszValue.is_null() {
                            HeapFree(GetProcessHeap(), 0, (*pOptions.add(i as usize)).pszValue as *mut _);
                        }
                        if !(*pOptions.add(i as usize)).pszName.is_null() {
                            HeapFree(GetProcessHeap(), 0, (*pOptions.add(i as usize)).pszName as *mut _);
                        }
                        i = i.wrapping_add(1);
                    }
                    HeapFree(GetProcessHeap(), 0, pOptions as *mut _);
                }
                if !hEnumKey.is_null() {
                    RegCloseKey(hEnumKey);
                }
                break 'done dwError;
            };
            return dwError;
        } else if dwError != ERROR_SUCCESS {
            'done: loop {
                if !pOptions.is_null() {
                    i = 0;
                    while i < dwValues {
                        if !(*pOptions.add(i as usize)).pszValue.is_null() {
                            HeapFree(GetProcessHeap(), 0, (*pOptions.add(i as usize)).pszValue as *mut _);
                        }
                        if !(*pOptions.add(i as usize)).pszName.is_null() {
                            HeapFree(GetProcessHeap(), 0, (*pOptions.add(i as usize)).pszName as *mut _);
                        }
                        i = i.wrapping_add(1);
                    }
                    HeapFree(GetProcessHeap(), 0, pOptions as *mut _);
                }
                if !hEnumKey.is_null() {
                    RegCloseKey(hEnumKey);
                }
                break 'done dwError;
            };
            return dwError;
        }

        i = i.wrapping_add(1);
    }

    (*pParameter).pEnumOptions = pOptions;
    (*pParameter).dwEnumOptions = dwValues;
    pOptions = core::ptr::null_mut();

    'done: loop {
        if !pOptions.is_null() {
            i = 0;
            while i < dwValues {
                if !(*pOptions.add(i as usize)).pszValue.is_null() {
                    HeapFree(GetProcessHeap(), 0, (*pOptions.add(i as usize)).pszValue as *mut _);
                }
                if !(*pOptions.add(i as usize)).pszName.is_null() {
                    HeapFree(GetProcessHeap(), 0, (*pOptions.add(i as usize)).pszName as *mut _);
                }
                i = i.wrapping_add(1);
            }
            HeapFree(GetProcessHeap(), 0, pOptions as *mut _);
        }
        if !hEnumKey.is_null() {
            RegCloseKey(hEnumKey);
        }
        break 'done dwError;
    };

    dwError
}
unsafe extern "C" fn FindEnumOption(
    pParameter: *mut PARAMETER,
    pszValue: PWSTR,
) -> INT {
    let mut i: INT;

    if (*pParameter).pEnumOptions == core::ptr::null_mut() ||
        (*pParameter).dwEnumOptions == 0 {
        return -1;
    }

    i = 0;
    while i < (*pParameter).dwEnumOptions as INT {
        if _wcsicmp((*(*pParameter).pEnumOptions.offset(i as isize)).pszValue, pszValue) == 0 {
            return i;
        }
        i += 1;
    }

    return -1;
}
unsafe extern "C" fn BuildParameterArray(
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
    ParameterArray: *mut PPARAMETER_ARRAY,
) -> BOOL {
    let mut hDriverKey: HKEY = (INVALID_HANDLE_VALUE).cast();
    let mut hParamsKey: HKEY = (INVALID_HANDLE_VALUE).cast();
    let mut hParamKey: HKEY = core::ptr::null_mut();
    let mut ParamArray: PPARAMETER_ARRAY = core::ptr::null_mut();
    let mut dwSubKeys: DWORD = 0;
    let mut dwMaxSubKeyLen: DWORD = 0;
    let mut dwKeyLen: DWORD;
    let mut dwIndex: DWORD;
    let mut pszType: PWSTR = core::ptr::null_mut();
    let mut lError: i32;
    // The C declares these uninitialised and assigns in every arm of the Type switch;
    // Rust cannot see that the arms are exhaustive over the reachable values.
    let mut lDefaultMin: i32 = 0;
    let mut lDefaultMax: i32 = 0;
    let mut dwDefaultMin: DWORD = 0;
    let mut dwDefaultMax: DWORD = 0;
    let mut ret: BOOL = (FALSE) as i32;

    hDriverKey = SetupDiOpenDevRegKey(
        DeviceInfoSet,
        DeviceInfoData,
        (DICS_FLAG_GLOBAL) as u32,
        0,
        (DIREG_DRV) as u32,
        KEY_READ,
    );
    if hDriverKey == (INVALID_HANDLE_VALUE).cast() {
        return (FALSE) as i32;
    }

    lError = RegOpenKeyExW(
        hDriverKey,
        w!("Ndi\\Params") as PWSTR,
        0,
        KEY_READ,
        &mut hParamsKey,
    );
    if lError != (ERROR_SUCCESS) as i32 {
        ret = (FALSE) as i32;
        if ret == (FALSE) as i32 && !ParamArray.is_null() {
            FreeParameterArray(ParamArray);
        }
        if hParamsKey != (INVALID_HANDLE_VALUE).cast() {
            RegCloseKey(hParamsKey);
        }
        if hDriverKey != (INVALID_HANDLE_VALUE).cast() {
            RegCloseKey(hDriverKey);
        }
        return ret;
    }

    lError = RegQueryInfoKeyW(
        hParamsKey,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &mut dwSubKeys,
        &mut dwMaxSubKeyLen,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    if lError != (ERROR_SUCCESS) as i32 {
        ret = (FALSE) as i32;
        if ret == (FALSE) as i32 && !ParamArray.is_null() {
            FreeParameterArray(ParamArray);
        }
        if hParamsKey != (INVALID_HANDLE_VALUE).cast() {
            RegCloseKey(hParamsKey);
        }
        if hDriverKey != (INVALID_HANDLE_VALUE).cast() {
            RegCloseKey(hDriverKey);
        }
        return ret;
    }

    if dwSubKeys == 0 {
        ret = (FALSE) as i32;
        if ret == (FALSE) as i32 && !ParamArray.is_null() {
            FreeParameterArray(ParamArray);
        }
        if hParamsKey != (INVALID_HANDLE_VALUE).cast() {
            RegCloseKey(hParamsKey);
        }
        if hDriverKey != (INVALID_HANDLE_VALUE).cast() {
            RegCloseKey(hDriverKey);
        }
        return ret;
    }

    ParamArray = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,(core::mem::size_of::<PARAMETER_ARRAY>() + (dwSubKeys as usize * core::mem::size_of::<PARAMETER>())) as SIZE_T,
    ) as PPARAMETER_ARRAY;
    if ParamArray.is_null() {
        ret = (FALSE) as i32;
        if ret == (FALSE) as i32 && !ParamArray.is_null() {
            FreeParameterArray(ParamArray);
        }
        if hParamsKey != (INVALID_HANDLE_VALUE).cast() {
            RegCloseKey(hParamsKey);
        }
        if hDriverKey != (INVALID_HANDLE_VALUE).cast() {
            RegCloseKey(hDriverKey);
        }
        return ret;
    }

    (*ParamArray).DeviceInfoSet = DeviceInfoSet;
    (*ParamArray).DeviceInfoData = DeviceInfoData;
    (*ParamArray).dwCount = dwSubKeys;

    dwMaxSubKeyLen = dwMaxSubKeyLen + 1;

    dwIndex = 0;
    while dwIndex < dwSubKeys {
        (*ParamArray).Array[dwIndex as usize].pszName = HeapAlloc(
            GetProcessHeap(),
            0,(dwMaxSubKeyLen as usize * core::mem::size_of::<u16>()) as SIZE_T,
        ) as PWSTR;
        if (*ParamArray).Array[dwIndex as usize].pszName.is_null() {
            ret = (FALSE) as i32;
            if ret == (FALSE) as i32 && !ParamArray.is_null() {
                FreeParameterArray(ParamArray);
            }
            if hParamsKey != (INVALID_HANDLE_VALUE).cast() {
                RegCloseKey(hParamsKey);
            }
            if hDriverKey != (INVALID_HANDLE_VALUE).cast() {
                RegCloseKey(hDriverKey);
            }
            return ret;
        }

        dwKeyLen = dwMaxSubKeyLen;
        lError = RegEnumKeyExW(
            hParamsKey,
            dwIndex,
            (*ParamArray).Array[dwIndex as usize].pszName,
            &mut dwKeyLen,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        if lError != (ERROR_SUCCESS) as i32 {
            break;
        }

        lError = RegOpenKeyExW(
            hParamsKey,
            (*ParamArray).Array[dwIndex as usize].pszName,
            0,
            KEY_READ,
            &mut hParamKey,
        );
        if lError == (ERROR_SUCCESS) as i32 {
            GetStringValue(
                hParamKey,
                w!("ParamDesc") as PWSTR,
                &mut (*ParamArray).Array[dwIndex as usize].pszDescription,
                core::ptr::null_mut(),
            );

            GetStringValue(
                hParamKey,
                w!("Type") as PWSTR,
                &mut pszType,
                core::ptr::null_mut(),
            );
            if !pszType.is_null() {
                if _wcsicmp(pszType, w!("int") as PWSTR) == 0 {
                    (*ParamArray).Array[dwIndex as usize].Type = INT_TYPE;
                } else if _wcsicmp(pszType, w!("long") as PWSTR) == 0 {
                    (*ParamArray).Array[dwIndex as usize].Type = LONG_TYPE;
                } else if _wcsicmp(pszType, w!("word") as PWSTR) == 0 {
                    (*ParamArray).Array[dwIndex as usize].Type = WORD_TYPE;
                } else if _wcsicmp(pszType, w!("dword") as PWSTR) == 0 {
                    (*ParamArray).Array[dwIndex as usize].Type = DWORD_TYPE;
                } else if _wcsicmp(pszType, w!("edit") as PWSTR) == 0 {
                    (*ParamArray).Array[dwIndex as usize].Type = EDIT_TYPE;
                } else if _wcsicmp(pszType, w!("enum") as PWSTR) == 0 {
                    (*ParamArray).Array[dwIndex as usize].Type = ENUM_TYPE;
                } else {
                    (*ParamArray).Array[dwIndex as usize].Type = NO_TYPE;
                }

                HeapFree(GetProcessHeap(), 0, pszType as *mut core::ffi::c_void);
                pszType = core::ptr::null_mut();
            }

            GetStringValue(
                hParamKey,
                w!("Default") as PWSTR,
                &mut (*ParamArray).Array[dwIndex as usize].pszDefault,
                core::ptr::null_mut(),
            );

            GetBooleanValue(
                hParamKey,
                w!("Optional") as PWSTR,
                (FALSE) as i32,
                &mut (*ParamArray).Array[dwIndex as usize].bOptional,
            );

            if (*ParamArray).Array[dwIndex as usize].Type == INT_TYPE
                || (*ParamArray).Array[dwIndex as usize].Type == LONG_TYPE
                || (*ParamArray).Array[dwIndex as usize].Type == WORD_TYPE
                || (*ParamArray).Array[dwIndex as usize].Type == DWORD_TYPE
            {
                if (*ParamArray).Array[dwIndex as usize].Type == INT_TYPE {
                    lDefaultMin = -32768;
                    lDefaultMax = 32767;
                } else if (*ParamArray).Array[dwIndex as usize].Type == LONG_TYPE {
                    lDefaultMin = (-2147483647 - 1);
                    lDefaultMax = 2147483647;
                } else if (*ParamArray).Array[dwIndex as usize].Type == WORD_TYPE {
                    dwDefaultMin = 0;
                    dwDefaultMax = 65535;
                } else if (*ParamArray).Array[dwIndex as usize].Type == DWORD_TYPE {
                    dwDefaultMin = 0;
                    dwDefaultMax = 4294967295;
                }

                if (*ParamArray).Array[dwIndex as usize].Type == INT_TYPE
                    || (*ParamArray).Array[dwIndex as usize].Type == LONG_TYPE
                {
                    GetLongValue(
                        hParamKey,
                        w!("Min") as PWSTR,
                        lDefaultMin,
                        &mut (*ParamArray).Array[dwIndex as usize].u.l.lMin,
                    );

                    GetLongValue(
                        hParamKey,
                        w!("Max") as PWSTR,
                        lDefaultMax,
                        &mut (*ParamArray).Array[dwIndex as usize].u.l.lMax,
                    );
                } else if (*ParamArray).Array[dwIndex as usize].Type == WORD_TYPE
                    || (*ParamArray).Array[dwIndex as usize].Type == DWORD_TYPE
                {
                    GetDWordValue(
                        hParamKey,
                        w!("Min") as PWSTR,
                        dwDefaultMin,
                        &mut (*ParamArray).Array[dwIndex as usize].u.dw.dwMin,
                    );

                    GetDWordValue(
                        hParamKey,
                        w!("Max") as PWSTR,
                        dwDefaultMax,
                        &mut (*ParamArray).Array[dwIndex as usize].u.dw.dwMax,
                    );
                }

                GetIntValue(
                    hParamKey,
                    w!("Base") as PWSTR,
                    10,
                    &mut (*ParamArray).Array[dwIndex as usize].iBase,
                );

                GetIntValue(
                    hParamKey,
                    w!("Step") as PWSTR,
                    1,
                    &mut (*ParamArray).Array[dwIndex as usize].iStep,
                );
            } else if (*ParamArray).Array[dwIndex as usize].Type == EDIT_TYPE {
                GetBooleanValue(
                    hParamKey,
                    w!("UpperCase") as PWSTR,
                    (FALSE) as i32,
                    &mut (*ParamArray).Array[dwIndex as usize].bUpperCase,
                );

                GetIntValue(
                    hParamKey,
                    w!("TextLimit") as PWSTR,
                    0,
                    &mut (*ParamArray).Array[dwIndex as usize].iTextLimit,
                );
            } else if (*ParamArray).Array[dwIndex as usize].Type == ENUM_TYPE {
                GetEnumOptions(
                    hParamKey,
                    &mut (*ParamArray).Array[dwIndex as usize],
                );
            }

            RegCloseKey(hParamKey);
        }

        lError = (GetStringValue(
            hDriverKey,
            (*ParamArray).Array[dwIndex as usize].pszName,
            &mut (*ParamArray).Array[dwIndex as usize].pszValue,
            &mut (*ParamArray).Array[dwIndex as usize].cchValueLength,
        )) as i32;
        if (lError == (ERROR_SUCCESS) as i32) || (!(*ParamArray).Array[dwIndex as usize].pszDefault.is_null()) {
            (*ParamArray).Array[dwIndex as usize].bPresent = (TRUE) as i32;
        }

        dwIndex += 1;
    }

    *ParameterArray = ParamArray;
    ret = (TRUE) as i32;

    if ret == (FALSE) as i32 && !ParamArray.is_null() {
        FreeParameterArray(ParamArray);
    }
    if hParamsKey != (INVALID_HANDLE_VALUE).cast() {
        RegCloseKey(hParamsKey);
    }
    if hDriverKey != (INVALID_HANDLE_VALUE).cast() {
        RegCloseKey(hDriverKey);
    }

    ret
}
unsafe extern "C" fn ReadParameterValue(
    hwnd: HWND,
    pParam: PPARAMETER,
) {
    let mut iIndex: INT;
    let mut iLength: INT;

    if (*pParam).Type == ENUM_TYPE {
        iIndex = ComboBox_GetCurSel!(GetDlgItem(hwnd, IDC_PROPERTY_VALUE_LIST));
        if iIndex != CB_ERR && iIndex < (*pParam).dwEnumOptions as INT {
            iLength = (wcslen((*(*pParam).pEnumOptions.offset(iIndex as isize)).pszValue)) as i32;
            if iLength > (*pParam).cchValueLength as INT {
                if (*pParam).pszValue != core::ptr::null_mut() {
                    HeapFree(GetProcessHeap(), 0, (*pParam).pszValue as *mut core::ffi::c_void);
                }

                (*pParam).pszValue = HeapAlloc(GetProcessHeap(), 0,((iLength + 1) as usize * core::mem::size_of::<u16>()) as SIZE_T) as PWSTR;
            }

            if (*pParam).pszValue != core::ptr::null_mut() {
                wcscpy((*pParam).pszValue, (*(*pParam).pEnumOptions.offset(iIndex as isize)).pszValue);
                (*pParam).cchValueLength = iLength as DWORD;
            }
        }
    } else {
        iLength = Edit_GetTextLength!(GetDlgItem(hwnd, IDC_PROPERTY_VALUE_EDIT));
        if iLength > (*pParam).cchValueLength as INT {
            if (*pParam).pszValue != core::ptr::null_mut() {
                HeapFree(GetProcessHeap(), 0, (*pParam).pszValue as *mut core::ffi::c_void);
            }

            (*pParam).pszValue = HeapAlloc(GetProcessHeap(), 0,((iLength + 1) as usize * core::mem::size_of::<u16>()) as SIZE_T) as PWSTR;
        }

        if (*pParam).pszValue != core::ptr::null_mut() {
            Edit_GetText!(GetDlgItem(hwnd, IDC_PROPERTY_VALUE_EDIT),
                         (*pParam).pszValue,
                         iLength + 1);
            (*pParam).cchValueLength = iLength as DWORD;
        }
    }
}
unsafe extern "C" fn WriteParameterArray(
    hwnd: HWND,
    ParamArray: PPARAMETER_ARRAY,
) {
    let mut InstallParams: SP_DEVINSTALL_PARAMS_W = core::mem::zeroed();
    let mut Param: PPARAMETER;
    let mut hDriverKey: HKEY;
    let mut i: INT;

    if ParamArray == core::ptr::null_mut() {
        return;
    }

    hDriverKey = SetupDiOpenDevRegKey(
        (*ParamArray).DeviceInfoSet,
        (*ParamArray).DeviceInfoData,
        (DICS_FLAG_GLOBAL) as u32,
        0,
        (DIREG_DRV) as u32,
        KEY_WRITE,
    );
    if hDriverKey == (INVALID_HANDLE_VALUE).cast() {
        return;
    }

    i = 0;
    while i < (*ParamArray).dwCount as INT {
        Param = core::ptr::addr_of_mut!((*ParamArray).Array).cast::<PARAMETER>().offset(i as isize);

        if Param == (*ParamArray).pCurrentParam {
            ReadParameterValue(hwnd, Param);
        }

        if (*Param).bPresent != 0 {
            RegSetValueExW(
                hDriverKey,
                (*Param).pszName,
                0,
                (REG_SZ) as u32,
                (*Param).pszValue as LPBYTE,
                ((wcslen((*Param).pszValue) + 1) * (core::mem::size_of::<u16>() as u32) as usize) as u32,
            );
        } else {
            RegDeleteValueW(hDriverKey, (*Param).pszName);
        }

        i += 1;
    }

    RegCloseKey(hDriverKey);

    InstallParams.cbSize = core::mem::size_of::<SP_DEVINSTALL_PARAMS_W>() as u32;
    if SetupDiGetDeviceInstallParamsW(
        (*ParamArray).DeviceInfoSet,
        (*ParamArray).DeviceInfoData,
        &mut InstallParams,
    ) != 0
    {
        InstallParams.FlagsEx |= (DI_FLAGSEX_PROPCHANGE_PENDING) as u32;

        SetupDiSetDeviceInstallParamsW(
            (*ParamArray).DeviceInfoSet,
            (*ParamArray).DeviceInfoData,
            &mut InstallParams,
        );
    }
}
unsafe extern "C" fn DisplayParameter(
    hwnd: HWND,
    Parameter: *mut PARAMETER,
) {
    let mut hwndControl: HWND;
    let mut Style: isize;
    let mut idx: i32;
    let mut i: u32;

    ShowWindow(GetDlgItem(hwnd, IDC_PROPERTY_PRESENT), if (*Parameter).bOptional != 0 { (SW_SHOW) as i32 } else { (SW_HIDE) as i32 });
    ShowWindow(GetDlgItem(hwnd, IDC_PROPERTY_NOT_PRESENT), if (*Parameter).bOptional != 0 { (SW_SHOW) as i32 } else { (SW_HIDE) as i32 });
    if (*Parameter).bOptional != 0 {
        if (*Parameter).bPresent != 0 {
            Button_SetCheck!(GetDlgItem(hwnd, IDC_PROPERTY_PRESENT), BST_CHECKED);
        } else {
            Button_SetCheck!(GetDlgItem(hwnd, IDC_PROPERTY_NOT_PRESENT), BST_CHECKED);
        }
    }

    match (*Parameter).Type {
        INT_TYPE | LONG_TYPE | WORD_TYPE | DWORD_TYPE => {
            ShowWindow(GetDlgItem(hwnd, IDC_PROPERTY_VALUE_LIST), (SW_HIDE) as i32);

            hwndControl = GetDlgItem(hwnd, IDC_PROPERTY_VALUE_UPDN);

            if (*Parameter).Type != DWORD_TYPE {
                EnableWindow(hwndControl, (*Parameter).bPresent);
                ShowWindow(hwndControl, (SW_SHOW) as i32);
            }

            if (*Parameter).Type == WORD_TYPE || (*Parameter).Type == DWORD_TYPE {
                SendMessage(hwndControl, (UDM_SETBASE) as u32, (*Parameter).iBase as WPARAM, 0);
            } else {
                SendMessage(hwndControl, (UDM_SETBASE) as u32, 10, 0);
            }

            if (*Parameter).Type == INT_TYPE || (*Parameter).Type == LONG_TYPE {
                SendMessage(hwndControl, (UDM_SETRANGE32) as u32, ((*Parameter).u.l.lMin) as SIZE_T, ((*Parameter).u.l.lMax) as LRESULT);
            } else if (*Parameter).Type == WORD_TYPE {
                SendMessage(hwndControl, (UDM_SETRANGE32) as u32, ((*Parameter).u.dw.dwMin as i32) as SIZE_T, ((*Parameter).u.dw.dwMax as i32) as LRESULT);
            }

            hwndControl = GetDlgItem(hwnd, IDC_PROPERTY_VALUE_EDIT);
            EnableWindow(hwndControl, (*Parameter).bPresent);
            ShowWindow(hwndControl, (SW_SHOW) as i32);

            Style = (GetWindowLongPtr(hwndControl, GWL_STYLE)) as isize;
            Style |= (ES_NUMBER) as isize;
            SetWindowLongPtr(hwndControl, GWL_STYLE, Style as LONG_PTR);

            Edit_LimitText!(hwndControl, 0);

            if !(*Parameter).pszValue.is_null() {
                Edit_SetText!(hwndControl, (*Parameter).pszValue);
            } else if !(*Parameter).pszDefault.is_null() {
                Edit_SetText!(hwndControl, (*Parameter).pszDefault);
            }
        }

        EDIT_TYPE => {
            ShowWindow(GetDlgItem(hwnd, IDC_PROPERTY_VALUE_UPDN), (SW_HIDE) as i32);
            ShowWindow(GetDlgItem(hwnd, IDC_PROPERTY_VALUE_LIST), (SW_HIDE) as i32);

            hwndControl = GetDlgItem(hwnd, IDC_PROPERTY_VALUE_EDIT);
            EnableWindow(hwndControl, (*Parameter).bPresent);
            ShowWindow(hwndControl, (SW_SHOW) as i32);

            Style = (GetWindowLongPtr(hwndControl, GWL_STYLE)) as isize;
            // Invert AFTER widening. `(!ES_NUMBER) as isize` complements the u32 first,
            // so the result ZERO-extends on the 64-bit targets and the mask clears every
            // bit above 31 as well. The C complements an `int` and sign-extends, which
            // leaves the upper half of the LONG_PTR alone. (scan_mask_drift caught this.)
            Style &= !(ES_NUMBER as isize);
            if (*Parameter).bUpperCase != 0 {
                Style |= (ES_UPPERCASE) as isize;
            } else {
                Style &= !(ES_UPPERCASE as isize);
            }
            SetWindowLongPtr(hwndControl, GWL_STYLE, Style as LONG_PTR);

            Edit_LimitText!(hwndControl, (*Parameter).iTextLimit);

            if !(*Parameter).pszValue.is_null() {
                Edit_SetText!(hwndControl, (*Parameter).pszValue);
            } else if !(*Parameter).pszDefault.is_null() {
                Edit_SetText!(hwndControl, (*Parameter).pszDefault);
            }
        }

        ENUM_TYPE => {
            ShowWindow(GetDlgItem(hwnd, IDC_PROPERTY_VALUE_EDIT), (SW_HIDE) as i32);
            ShowWindow(GetDlgItem(hwnd, IDC_PROPERTY_VALUE_UPDN), (SW_HIDE) as i32);

            hwndControl = GetDlgItem(hwnd, IDC_PROPERTY_VALUE_LIST);
            EnableWindow(hwndControl, (*Parameter).bPresent);
            ShowWindow(hwndControl, (SW_SHOW) as i32);

            ComboBox_ResetContent!(hwndControl);

            if !(*Parameter).pEnumOptions.is_null() && (*Parameter).dwEnumOptions != 0 {
                i = 0;
                while i < (*Parameter).dwEnumOptions {
                    ComboBox_AddString!(hwndControl, (*(*Parameter).pEnumOptions.offset(i as isize)).pszName);
                    i += 1;
                }
            }

            if !(*Parameter).pszValue.is_null() {
                idx = FindEnumOption(Parameter, (*Parameter).pszValue);
                if idx != CB_ERR {
                    ComboBox_SetCurSel!(hwndControl, idx);
                }
            } else if !(*Parameter).pszDefault.is_null() {
                idx = FindEnumOption(Parameter, (*Parameter).pszDefault);
                if idx != CB_ERR {
                    ComboBox_SetCurSel!(hwndControl, idx);
                }
            }
        }

        _ => {}
    }
}
unsafe extern "C" fn OnInitDialog(
    hwnd: HWND,
    wParam: WPARAM,
    lParam: LPARAM,
) -> BOOL {
    let pParamArray: PPARAMETER_ARRAY;
    let hwndControl: HWND;
    let mut pszText: PWSTR;
    let mut i: DWORD;
    let idx: INT;

    pParamArray = (*(lParam as *mut PROPSHEETPAGEW)).lParam as PPARAMETER_ARRAY;
    if pParamArray == core::ptr::null_mut() {
        return (FALSE) as i32;
    }

    SetWindowLongPtr(hwnd, DWLP_USER, pParamArray as LONG_PTR);

    hwndControl = GetDlgItem(hwnd, IDC_PROPERTY_NAME);
    if hwndControl != core::ptr::null_mut() {
        i = 0;
        while i < (*pParamArray).dwCount {
            if (*pParamArray).Array[i as usize].pszDescription != core::ptr::null_mut() {
                pszText = (*pParamArray).Array[i as usize].pszDescription;
            } else {
                pszText = (*pParamArray).Array[i as usize].pszName;
            }

            let idx = ListBox_AddString!(hwndControl, pszText);
            if idx != LB_ERR {
                ListBox_SetItemData!(hwndControl, idx, core::ptr::addr_of_mut!((*pParamArray).Array[i as usize]) as LPARAM);
            }
            i = i + 1;
        }

        if (*pParamArray).dwCount > 0 {
            ListBox_SetCurSel!(hwndControl, 0);
            (*pParamArray).pCurrentParam = ListBox_GetItemData!(hwndControl, 0) as PPARAMETER;
            DisplayParameter(hwnd, (*pParamArray).pCurrentParam);
        }
    }

    (TRUE) as i32
}
unsafe extern "C" fn OnCommand(
    hwnd: HWND,
    wParam: WPARAM,
    lParam: LPARAM,
) {
    let pParamArray: *mut PARAMETER_ARRAY;
    let iIndex: i32;

    pParamArray = GetWindowLongPtr(hwnd, DWLP_USER) as *mut PARAMETER_ARRAY;
    if pParamArray.is_null() {
        return;
    }

    if (LOWORD!(wParam) == (IDC_PROPERTY_NAME) as u16) && (HIWORD!(wParam) == (LBN_SELCHANGE) as u16) {
        if !(*pParamArray).pCurrentParam.is_null() {
            ReadParameterValue(hwnd, (*pParamArray).pCurrentParam);
        }

        iIndex = ListBox_GetCurSel!(lParam as HWND);
        if iIndex != LB_ERR && (iIndex as DWORD) < (*pParamArray).dwCount {
            (*pParamArray).pCurrentParam = ListBox_GetItemData!(lParam as HWND, iIndex) as *mut PARAMETER;
            DisplayParameter(hwnd, (*pParamArray).pCurrentParam);
        }
    } else if (LOWORD!(wParam) == (IDC_PROPERTY_PRESENT) as u16) && (HIWORD!(wParam) == (BN_CLICKED) as u16) {
        EnableWindow(GetDlgItem(hwnd, IDC_PROPERTY_VALUE_EDIT), (TRUE) as i32);
        EnableWindow(GetDlgItem(hwnd, IDC_PROPERTY_VALUE_UPDN), (TRUE) as i32);
        EnableWindow(GetDlgItem(hwnd, IDC_PROPERTY_VALUE_LIST), (TRUE) as i32);
        (*(*pParamArray).pCurrentParam).bPresent = (TRUE) as i32;
    } else if (LOWORD!(wParam) == (IDC_PROPERTY_NOT_PRESENT) as u16) && (HIWORD!(wParam) == (BN_CLICKED) as u16) {
        EnableWindow(GetDlgItem(hwnd, IDC_PROPERTY_VALUE_EDIT), (FALSE) as i32);
        EnableWindow(GetDlgItem(hwnd, IDC_PROPERTY_VALUE_UPDN), (FALSE) as i32);
        EnableWindow(GetDlgItem(hwnd, IDC_PROPERTY_VALUE_LIST), (FALSE) as i32);
        (*(*pParamArray).pCurrentParam).bPresent = (FALSE) as i32;
    }
}
unsafe extern "C" fn OnNotify(
    hwnd: HWND,
    wParam: WPARAM,
    lParam: LPARAM,
) {
    let pParamArray: PPARAMETER_ARRAY;

    pParamArray = GetWindowLongPtr(hwnd, DWLP_USER) as PPARAMETER_ARRAY;
    if pParamArray.is_null() {
        return;
    }

    if (*(lParam as LPNMHDR)).code == PSN_APPLY as u32 {
        WriteParameterArray(hwnd, pParamArray);
    } else if (*(lParam as LPNMHDR)).code == UDN_DELTAPOS as u32 {
        let pUpDown = lParam as LPNMUPDOWN;
        // `pParamArray->pCurrentParam->iStep` — a plain C deref, so a plain Rust deref.
        // `.as_ref().unwrap()` would PANIC where the C faults, which is not the same
        // program: a WM_NOTIFY handler that panics unwinds through the Win32 dispatcher.
        (*pUpDown).iDelta *= (*(*pParamArray).pCurrentParam).iStep;
    }
}
unsafe extern "C" fn OnDestroy(
    hwnd: HWND) {
    let mut pParamArray: PPARAMETER_ARRAY;

    pParamArray = GetWindowLongPtr(hwnd, DWLP_USER) as PPARAMETER_ARRAY;
    if pParamArray == core::ptr::null_mut()
    {
        return;
    }

    FreeParameterArray(pParamArray);
    SetWindowLongPtr(hwnd, DWLP_USER, 0 as LONG_PTR);
}
#[cfg(target_arch = "x86")]
unsafe extern "system" fn NetPropertyPageDlgProc(
    hwnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    match uMsg {
        WM_INITDIALOG => {
            return OnInitDialog(hwnd, wParam, lParam);
        }

        WM_COMMAND => {
            OnCommand(hwnd, wParam, lParam);
        }

        WM_NOTIFY => {
            OnNotify(hwnd, wParam, lParam);
        }

        WM_DESTROY => {
            OnDestroy(hwnd);
        }

        _ => {}
    }

    FALSE
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "system" fn NetPropertyPageDlgProc(
    hwnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    match uMsg {
        WM_INITDIALOG => {
            return (OnInitDialog(hwnd, wParam, lParam)) as LRESULT;
        }

        WM_COMMAND => {
            OnCommand(hwnd, wParam, lParam);
        }

        WM_NOTIFY => {
            OnNotify(hwnd, wParam, lParam);
        }

        WM_DESTROY => {
            OnDestroy(hwnd);
        }

        _ => {}
    }

    (FALSE) as LRESULT
}

#[no_mangle]
pub unsafe extern "system" fn NetPropPageProvider(
    lpPropSheetPageRequest: PSP_PROPSHEETPAGE_REQUEST,
    lpfnAddPropSheetPageProc: LPFNADDPROPSHEETPAGE,
    lParam: LPARAM,
) -> BOOL {
    let mut PropSheetPage: PROPSHEETPAGEW = core::mem::zeroed();
    let mut hPropSheetPage: HPROPSHEETPAGE;
    let mut ParameterArray: PPARAMETER_ARRAY = core::ptr::null_mut();

    if BuildParameterArray(
        (*lpPropSheetPageRequest).DeviceInfoSet,
        (*lpPropSheetPageRequest).DeviceInfoData,
        &mut ParameterArray,
    ) == 0
    {
        return (FALSE) as i32;
    }

    if (*lpPropSheetPageRequest).PageRequested == (SPPSR_ENUM_ADV_DEVICE_PROPERTIES) as u32 {
        PropSheetPage.dwSize = core::mem::size_of::<PROPSHEETPAGEW>() as u32;
        PropSheetPage.dwFlags = 0;
        PropSheetPage.hInstance = netcfgx_hInstance;
        *PropSheetPage.u.pszTemplate.as_mut() = MAKEINTRESOURCE!(IDD_NET_PROPERTY_DLG) as *const u16;
        PropSheetPage.pfnDlgProc = Some(NetPropertyPageDlgProc);
        PropSheetPage.lParam = ParameterArray as LPARAM;
        PropSheetPage.pfnCallback = None;

        hPropSheetPage = CreatePropertySheetPageW(&mut PropSheetPage);
        if hPropSheetPage.is_null() {
            return (FALSE) as i32;
        }

        // bindgen renders a C function-pointer parameter as `Option<fn>`; the C calls it
        // unconditionally, so `unwrap_unchecked` is the faithful lowering — `.unwrap()` and
        // `.expect()` both ADD a panic to a live DLL where the C simply faults, which is
        // the same objection crate-repair raises to rustc's `.try_into().unwrap()` fix-it.
        if lpfnAddPropSheetPageProc.unwrap_unchecked()(hPropSheetPage, lParam) == 0 {
            DestroyPropertySheetPage(hPropSheetPage);
            return (FALSE) as i32;
        }
    }

    (TRUE) as i32
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS system libraries
//  * LICENSE:     GPL-2.0+ (https://spdx.org/licenses/GPL-2.0+)
//  * PURPOSE:     Network property page provider
//  * COPYRIGHT:   Copyright 2018 Eric Kohl (eric.kohl@reactos.org)
//  */
//
// #include "precomp.h"
//
// typedef enum _PARAM_TYPE
// {
//     NO_TYPE,
//     INT_TYPE,
//     LONG_TYPE,
//     WORD_TYPE,
//     DWORD_TYPE,
//     EDIT_TYPE,
//     ENUM_TYPE,
// } PARAM_TYPE, *PPARAM_TYPE;
//
// typedef struct _ENUM_OPTION
// {
//     PWSTR pszValue;
//     PWSTR pszName;
// } ENUM_OPTION, *PENUM_OPTION;
//
// typedef struct _PARAMETER
// {
//     PWSTR pszName;
//     PWSTR pszDescription;
//     PWSTR pszValue;
//     DWORD cchValueLength;
//     PWSTR pszDefault;
//     BOOL bOptional;
//     BOOL bPresent;
//     PARAM_TYPE Type;
//
//     DWORD dwEnumOptions;
//     PENUM_OPTION pEnumOptions;
//
//     BOOL bUpperCase;
//     INT iTextLimit;
//
//     INT iBase;
//     INT iStep;
//
//     union
//     {
//         struct
//         {
//             LONG lMin;
//             LONG lMax;
//         } l;
//         struct
//         {
//             DWORD dwMin;
//             DWORD dwMax;
//         } dw;
//     } u;
// } PARAMETER, *PPARAMETER;
//
// typedef struct _PARAMETER_ARRAY
// {
//     HDEVINFO DeviceInfoSet;
//     PSP_DEVINFO_DATA DeviceInfoData;
//     PPARAMETER pCurrentParam;
//     DWORD dwCount;
//     PARAMETER Array[0];
// } PARAMETER_ARRAY, *PPARAMETER_ARRAY;
//
//
// static
// VOID
// FreeParameterArray(
//      PPARAMETER_ARRAY ParamArray)
// {
//     INT i, j;
//
//     if (ParamArray == NULL)
//         return;
//
//     for (i = 0; i < ParamArray->dwCount; i++)
//     {
//         if (ParamArray->Array[i].pszName != NULL)
//             HeapFree(GetProcessHeap(), 0, ParamArray->Array[i].pszName);
//
//         if (ParamArray->Array[i].pszDescription != NULL)
//             HeapFree(GetProcessHeap(), 0, ParamArray->Array[i].pszDescription);
//
//         if (ParamArray->Array[i].pszDefault != NULL)
//             HeapFree(GetProcessHeap(), 0, ParamArray->Array[i].pszDefault);
//
//
//         if (ParamArray->Array[i].pEnumOptions != NULL)
//         {
//             for (j = 0; j < ParamArray->Array[i].dwEnumOptions; j++)
//             {
//                 if (ParamArray->Array[i].pEnumOptions[j].pszValue != NULL)
//                     HeapFree(GetProcessHeap(), 0, ParamArray->Array[i].pEnumOptions[j].pszValue);
//
//                 if (ParamArray->Array[i].pEnumOptions[j].pszName != NULL)
//                     HeapFree(GetProcessHeap(), 0, ParamArray->Array[i].pEnumOptions[j].pszName);
//             }
//
//             HeapFree(GetProcessHeap(), 0, ParamArray->Array[i].pEnumOptions);
//         }
//     }
//
//     HeapFree(GetProcessHeap(), 0, ParamArray);
// }
//
//
// static DWORD
// GetStringValue(
//      HKEY hKey,
//      PWSTR pValueName,
//      PWSTR *pString,
//      PDWORD pdwStringLength)
// {
//     PWSTR pBuffer;
//     DWORD dwLength = 0;
//     DWORD dwRegType;
//     DWORD dwError;
//
//     *pString = NULL;
//
//     RegQueryValueExW(hKey, pValueName, NULL, &dwRegType, NULL, &dwLength);
//
//     if (dwLength == 0 || dwRegType != REG_SZ)
//         return ERROR_FILE_NOT_FOUND;
//
//     pBuffer = HeapAlloc(GetProcessHeap(), 0, dwLength + sizeof(WCHAR));
//     if (pBuffer == NULL)
//         return ERROR_NOT_ENOUGH_MEMORY;
//
//     dwError = RegQueryValueExW(hKey, pValueName, NULL, NULL, (LPBYTE)pBuffer, &dwLength);
//     if (dwError != ERROR_SUCCESS)
//     {
//         HeapFree(GetProcessHeap(), 0, pBuffer);
//         return dwError;
//     }
//
//     pBuffer[dwLength / sizeof(WCHAR)] = UNICODE_NULL;
//
//     *pString = pBuffer;
//     if (pdwStringLength)
//         *pdwStringLength = dwLength;
//
//     return ERROR_SUCCESS;
// }
//
//
// static DWORD
// GetBooleanValue(
//      HKEY hKey,
//      PWSTR pValueName,
//      BOOL bDefault,
//      PBOOL pValue)
// {
//     WCHAR szBuffer[16];
//     DWORD dwLength = 0;
//     DWORD dwRegType;
//
//     *pValue = bDefault;
//
//     dwLength = sizeof(szBuffer);
//     RegQueryValueExW(hKey,
//                      pValueName,
//                      NULL,
//                      &dwRegType,
//                      (LPBYTE)szBuffer,
//                      &dwLength);
//
//     if (dwRegType == REG_SZ && dwLength >= sizeof(WCHAR))
//     {
//         if (szBuffer[0] == L'0')
//             *pValue = FALSE;
//         else
//             *pValue = TRUE;
//     }
//
//     return ERROR_SUCCESS;
// }
//
//
// static DWORD
// GetIntValue(
//      HKEY hKey,
//      PWSTR pValueName,
//      INT iDefault,
//      PINT pValue)
// {
//     WCHAR szBuffer[24];
//     DWORD dwLength = 0;
//     DWORD dwRegType;
//
//     *pValue = iDefault;
//
//     dwLength = sizeof(szBuffer);
//     RegQueryValueExW(hKey,
//                      pValueName,
//                      NULL,
//                      &dwRegType,
//                      (LPBYTE)szBuffer,
//                      &dwLength);
//
//     if (dwRegType == REG_SZ && dwLength >= sizeof(WCHAR))
//     {
//         *pValue = _wtoi(szBuffer);
//     }
//
//     return ERROR_SUCCESS;
// }
//
//
// static DWORD
// GetLongValue(
//      HKEY hKey,
//      PWSTR pValueName,
//      LONG lDefault,
//      PLONG pValue)
// {
//     WCHAR szBuffer[24];
//     DWORD dwLength = 0;
//     DWORD dwRegType;
//     PWSTR ptr = NULL;
//
//     dwLength = sizeof(szBuffer);
//     RegQueryValueExW(hKey,
//                      pValueName,
//                      NULL,
//                      &dwRegType,
//                      (LPBYTE)szBuffer,
//                      &dwLength);
//
//     if (dwRegType == REG_SZ && dwLength >= sizeof(WCHAR))
//     {
//         *pValue = wcstol(szBuffer, &ptr, 10);
//         if (*pValue == 0 && ptr != NULL)
//             *pValue = lDefault;
//     }
//     else
//     {
//         *pValue = lDefault;
//     }
//
//     return ERROR_SUCCESS;
// }
//
//
// static DWORD
// GetDWordValue(
//      HKEY hKey,
//      PWSTR pValueName,
//      DWORD dwDefault,
//      PDWORD pValue)
// {
//     WCHAR szBuffer[24];
//     DWORD dwLength = 0;
//     DWORD dwRegType;
//     PWSTR ptr = NULL;
//
//     dwLength = sizeof(szBuffer);
//     RegQueryValueExW(hKey,
//                      pValueName,
//                      NULL,
//                      &dwRegType,
//                      (LPBYTE)szBuffer,
//                      &dwLength);
//
//     if (dwRegType == REG_SZ && dwLength >= sizeof(WCHAR))
//     {
//         *pValue = wcstoul(szBuffer, &ptr, 10);
//         if (*pValue == 0 && ptr != NULL)
//             *pValue = dwDefault;
//     }
//     else
//     {
//         *pValue = dwDefault;
//     }
//
//     return ERROR_SUCCESS;
// }
//
//
// static
// DWORD
// GetEnumOptions(
//      HKEY hKey,
//      PPARAMETER pParameter)
// {
//     HKEY hEnumKey = NULL;
//     PENUM_OPTION pOptions = NULL;
//     DWORD dwValues, dwMaxValueNameLen, dwMaxValueLen;
//     DWORD dwValueNameLength, dwValueLength;
//     DWORD i;
//     DWORD dwError;
//
//     dwError = RegOpenKeyExW(hKey,
//                             L"enum",
//                             0,
//                             KEY_READ,
//                             &hEnumKey);
//     if (dwError != ERROR_SUCCESS)
//         return dwError;
//
//     dwError = RegQueryInfoKeyW(hEnumKey,
//                                NULL,
//                                NULL,
//                                NULL,
//                                NULL,
//                                NULL,
//                                NULL,
//                                &dwValues,
//                                &dwMaxValueNameLen,
//                                &dwMaxValueLen,
//                                NULL,
//                                NULL);
//     if (dwError != ERROR_SUCCESS)
//     {
//         ERR("RegQueryInfoKeyW failed (Error %lu)\n", dwError);
//         goto done;
//     }
//
//     pOptions = HeapAlloc(GetProcessHeap(),
//                          HEAP_ZERO_MEMORY,
//                          dwValues * sizeof(ENUM_OPTION));
//     if (pOptions == NULL)
//     {
//         dwError = ERROR_OUTOFMEMORY;
//         goto done;
//     }
//
//     for (i = 0; i < dwValues; i++)
//     {
//         dwValueNameLength = dwMaxValueNameLen + sizeof(WCHAR);
//         pOptions[i].pszValue = HeapAlloc(GetProcessHeap(),
//                                         0,
//                                         dwValueNameLength * sizeof(WCHAR));
//         if (pOptions[i].pszValue == NULL)
//         {
//             dwError = ERROR_OUTOFMEMORY;
//             goto done;
//         }
//
//         dwValueLength = dwMaxValueLen;
//         pOptions[i].pszName = HeapAlloc(GetProcessHeap(),
//                                         0,
//                                         dwValueLength);
//         if (pOptions[i].pszName == NULL)
//         {
//             dwError = ERROR_OUTOFMEMORY;
//             goto done;
//         }
//
//         dwError = RegEnumValueW(hEnumKey,
//                                 i,
//                                 pOptions[i].pszValue,
//                                 &dwValueNameLength,
//                                 NULL,
//                                 NULL,
//                                 (PBYTE)pOptions[i].pszName,
//                                 &dwValueLength);
//         if (dwError == ERROR_NO_MORE_ITEMS)
//         {
//             dwError = ERROR_SUCCESS;
//             goto done;
//         }
//         else if (dwError != ERROR_SUCCESS)
//         {
//             goto done;
//         }
//     }
//
//     pParameter->pEnumOptions = pOptions;
//     pParameter->dwEnumOptions = dwValues;
//     pOptions = NULL;
//
// done:
//     if (pOptions != NULL)
//     {
//         for (i = 0; i < dwValues; i++)
//         {
//             if (pOptions[i].pszValue != NULL)
//                 HeapFree(GetProcessHeap(), 0, pOptions[i].pszValue);
//
//             if (pOptions[i].pszName != NULL)
//                 HeapFree(GetProcessHeap(), 0, pOptions[i].pszName);
//         }
//
//         HeapFree(GetProcessHeap(), 0, pOptions);
//     }
//
//     if (hEnumKey != NULL)
//         RegCloseKey(hEnumKey);
//
//     return dwError;
// }
//
//
// static
// INT
// FindEnumOption(
//      PPARAMETER pParameter,
//      PWSTR pszValue)
// {
//     INT i;
//
//     if ((pParameter->pEnumOptions == NULL) ||
//         (pParameter->dwEnumOptions == 0))
//         return -1;
//
//     for (i = 0; i < pParameter->dwEnumOptions; i++)
//     {
//         if (_wcsicmp(pParameter->pEnumOptions[i].pszValue, pszValue) == 0)
//             return i;
//     }
//
//     return -1;
// }
//
//
// static
// BOOL
// BuildParameterArray(
//      HDEVINFO DeviceInfoSet,
//      PSP_DEVINFO_DATA DeviceInfoData,
//      PPARAMETER_ARRAY *ParameterArray)
// {
//     HKEY hDriverKey = INVALID_HANDLE_VALUE;
//     HKEY hParamsKey = INVALID_HANDLE_VALUE;
//     HKEY hParamKey;
//     PPARAMETER_ARRAY ParamArray = NULL;
//     DWORD dwSubKeys, dwMaxSubKeyLen, dwKeyLen, dwIndex;
//     PWSTR pszType = NULL;
//     LONG lError;
//     LONG lDefaultMin, lDefaultMax;
//     DWORD dwDefaultMin, dwDefaultMax;
//     BOOL ret = FALSE;
//
//     hDriverKey = SetupDiOpenDevRegKey(DeviceInfoSet,
//                                       DeviceInfoData,
//                                       DICS_FLAG_GLOBAL,
//                                       0,
//                                       DIREG_DRV,
//                                       KEY_READ);
//     if (hDriverKey == INVALID_HANDLE_VALUE)
//     {
//         ERR("SetupDiOpenDevRegKey() failed\n");
//         return FALSE;
//     }
//
//     lError = RegOpenKeyExW(hDriverKey,
//                            L"Ndi\\Params",
//                            0,
//                            KEY_READ,
//                            &hParamsKey);
//     if (lError != ERROR_SUCCESS)
//     {
//         ERR("RegOpenKeyExW failed (Error %lu)\n", lError);
//         goto done;
//     }
//
//     lError = RegQueryInfoKeyW(hParamsKey,
//                               NULL,
//                               NULL,
//                               NULL,
//                               &dwSubKeys,
//                               &dwMaxSubKeyLen,
//                               NULL,
//                               NULL,
//                               NULL,
//                               NULL,
//                               NULL,
//                               NULL);
//     if (lError != ERROR_SUCCESS)
//     {
//         ERR("RegOpenKeyExW failed (Error %lu)\n", lError);
//         goto done;
//     }
//
//     TRACE("Sub keys: %lu\n", dwSubKeys);
//
//     if (dwSubKeys == 0)
//     {
//         TRACE("No sub keys. Done!\n");
//         goto done;
//     }
//
//     ParamArray = HeapAlloc(GetProcessHeap(),
//                            HEAP_ZERO_MEMORY,
//                            sizeof(PARAMETER_ARRAY) + (dwSubKeys * sizeof(PARAMETER)));
//     if (ParamArray == NULL)
//     {
//         ERR("Parameter array allocation failed!\n");
//         goto done;
//     }
//
//     ParamArray->DeviceInfoSet = DeviceInfoSet;
//     ParamArray->DeviceInfoData = DeviceInfoData;
//     ParamArray->dwCount = dwSubKeys;
//
//     dwMaxSubKeyLen++;
//
//     for (dwIndex = 0; dwIndex < dwSubKeys; dwIndex++)
//     {
//         ParamArray->Array[dwIndex].pszName = HeapAlloc(GetProcessHeap(),
//                                                        0,
//                                                        dwMaxSubKeyLen * sizeof(WCHAR));
//         if (ParamArray->Array[dwIndex].pszName == NULL)
//         {
//             ERR("Parameter array allocation failed!\n");
//             goto done;
//         }
//
//         dwKeyLen = dwMaxSubKeyLen;
//         lError = RegEnumKeyExW(hParamsKey,
//                                dwIndex,
//                                ParamArray->Array[dwIndex].pszName,
//                                &dwKeyLen,
//                                NULL,
//                                NULL,
//                                NULL,
//                                NULL);
//         if (lError != ERROR_SUCCESS)
//             break;
//
//         TRACE("Sub key '%S'\n", ParamArray->Array[dwIndex].pszName);
//
//         lError = RegOpenKeyExW(hParamsKey,
//                                ParamArray->Array[dwIndex].pszName,
//                                0,
//                                KEY_READ,
//                                &hParamKey);
//         if (lError == ERROR_SUCCESS)
//         {
//             GetStringValue(hParamKey,
//                            L"ParamDesc",
//                            &ParamArray->Array[dwIndex].pszDescription,
//                            NULL);
//
//             GetStringValue(hParamKey,
//                            L"Type",
//                            &pszType,
//                            NULL);
//             if (pszType != NULL)
//             {
//                 if (_wcsicmp(pszType, L"int") == 0)
//                     ParamArray->Array[dwIndex].Type = INT_TYPE;
//                 else if (_wcsicmp(pszType, L"long") == 0)
//                     ParamArray->Array[dwIndex].Type = LONG_TYPE;
//                 else if (_wcsicmp(pszType, L"word") == 0)
//                     ParamArray->Array[dwIndex].Type = WORD_TYPE;
//                 else if (_wcsicmp(pszType, L"dword") == 0)
//                     ParamArray->Array[dwIndex].Type = DWORD_TYPE;
//                 else if (_wcsicmp(pszType, L"edit") == 0)
//                     ParamArray->Array[dwIndex].Type = EDIT_TYPE;
//                 else if (_wcsicmp(pszType, L"enum") == 0)
//                     ParamArray->Array[dwIndex].Type = ENUM_TYPE;
//                 else
//                     ParamArray->Array[dwIndex].Type = NO_TYPE;
//
//                 HeapFree(GetProcessHeap(), 0, pszType);
//                 pszType = NULL;
//             }
//
//             GetStringValue(hParamKey,
//                            L"Default",
//                            &ParamArray->Array[dwIndex].pszDefault,
//                            NULL);
//
//             GetBooleanValue(hParamKey,
//                             L"Optional",
//                             FALSE,
//                             &ParamArray->Array[dwIndex].bOptional);
//
//             if (ParamArray->Array[dwIndex].Type == INT_TYPE ||
//                 ParamArray->Array[dwIndex].Type == LONG_TYPE ||
//                 ParamArray->Array[dwIndex].Type == WORD_TYPE ||
//                 ParamArray->Array[dwIndex].Type == DWORD_TYPE)
//             {
//                 if (ParamArray->Array[dwIndex].Type == INT_TYPE)
//                 {
//                     lDefaultMin = -32768L; //MIN_SHORT;
//                     lDefaultMax = 32767L;  //MAX_SHORT;
//                 }
//                 else if (ParamArray->Array[dwIndex].Type == LONG_TYPE)
//                 {
//                     lDefaultMin = (-2147483647L - 1); // MIN_LONG;
//                     lDefaultMax = 2147483647L;        // MAX_LONG;
//                 }
//                 else if (ParamArray->Array[dwIndex].Type == WORD_TYPE)
//                 {
//                     dwDefaultMin = 0UL;
//                     dwDefaultMax = 65535UL; // MAX_WORD;
//                 }
//                 else if (ParamArray->Array[dwIndex].Type == DWORD_TYPE)
//                 {
//                     dwDefaultMin = 0UL;
//                     dwDefaultMax = 4294967295UL; //MAX_DWORD;
//                 }
//
//                 if (ParamArray->Array[dwIndex].Type == INT_TYPE ||
//                     ParamArray->Array[dwIndex].Type == LONG_TYPE)
//                 {
//                     GetLongValue(hParamKey,
//                                  L"Min",
//                                  lDefaultMin,
//                                  &ParamArray->Array[dwIndex].u.l.lMin);
//
//                     GetLongValue(hParamKey,
//                                  L"Max",
//                                  lDefaultMax,
//                                  &ParamArray->Array[dwIndex].u.l.lMax);
//                 }
//                 else if (ParamArray->Array[dwIndex].Type == WORD_TYPE ||
//                          ParamArray->Array[dwIndex].Type == DWORD_TYPE)
//                 {
//                     GetDWordValue(hParamKey,
//                                   L"Min",
//                                   dwDefaultMin,
//                                   &ParamArray->Array[dwIndex].u.dw.dwMin);
//
//                     GetDWordValue(hParamKey,
//                                   L"Max",
//                                   dwDefaultMax,
//                                   &ParamArray->Array[dwIndex].u.dw.dwMax);
//                 }
//
//                 GetIntValue(hParamKey,
//                             L"Base",
//                             10,
//                             &ParamArray->Array[dwIndex].iBase);
//
//                 GetIntValue(hParamKey,
//                             L"Step",
//                             1,
//                             &ParamArray->Array[dwIndex].iStep);
//             }
//             else if (ParamArray->Array[dwIndex].Type == EDIT_TYPE)
//             {
//                 GetBooleanValue(hParamKey,
//                                 L"UpperCase",
//                                 FALSE,
//                                 &ParamArray->Array[dwIndex].bUpperCase);
//
//                 GetIntValue(hParamKey,
//                             L"TextLimit",
//                             0,
//                             &ParamArray->Array[dwIndex].iTextLimit);
//             }
//             else if (ParamArray->Array[dwIndex].Type == ENUM_TYPE)
//             {
//                 GetEnumOptions(hParamKey,
//                                &ParamArray->Array[dwIndex]);
//             }
//
//             RegCloseKey(hParamKey);
//         }
//
//         lError = GetStringValue(hDriverKey,
//                                 ParamArray->Array[dwIndex].pszName,
//                                 &ParamArray->Array[dwIndex].pszValue,
//                                 &ParamArray->Array[dwIndex].cchValueLength);
//         if ((lError == ERROR_SUCCESS) ||
//             (ParamArray->Array[dwIndex].pszDefault != NULL))
//         {
//             ParamArray->Array[dwIndex].bPresent = TRUE;
//         }
//     }
//
//     *ParameterArray = ParamArray;
//     ret = TRUE;
//
// done:
//     if (ret == FALSE && ParamArray != NULL)
//         FreeParameterArray(ParamArray);
//
//     if (hParamsKey != INVALID_HANDLE_VALUE)
//         RegCloseKey(hParamsKey);
//
//     if (hDriverKey != INVALID_HANDLE_VALUE)
//         RegCloseKey(hDriverKey);
//
//     return ret;
// }
//
//
// static
// VOID
// ReadParameterValue(
//      HWND hwnd,
//      PPARAMETER pParam)
// {
//     INT iIndex, iLength;
//
//     if (pParam->Type == ENUM_TYPE)
//     {
//         iIndex = ComboBox_GetCurSel(GetDlgItem(hwnd, IDC_PROPERTY_VALUE_LIST));
//         if (iIndex != CB_ERR && iIndex < pParam->dwEnumOptions)
//         {
//             iLength = wcslen(pParam->pEnumOptions[iIndex].pszValue);
//             if (iLength > pParam->cchValueLength)
//             {
//                 if (pParam->pszValue != NULL)
//                     HeapFree(GetProcessHeap(), 0, pParam->pszValue);
//
//                 pParam->pszValue = HeapAlloc(GetProcessHeap(), 0, (iLength + 1) * sizeof(WCHAR));
//             }
//
//             if (pParam->pszValue != NULL)
//             {
//                 wcscpy(pParam->pszValue,
//                        pParam->pEnumOptions[iIndex].pszValue);
//                 pParam->cchValueLength = iLength;
//             }
//         }
//     }
//     else
//     {
//         iLength = Edit_GetTextLength(GetDlgItem(hwnd, IDC_PROPERTY_VALUE_EDIT));
//         if (iLength > pParam->cchValueLength)
//         {
//             if (pParam->pszValue != NULL)
//                 HeapFree(GetProcessHeap(), 0, pParam->pszValue);
//
//             pParam->pszValue = HeapAlloc(GetProcessHeap(), 0, (iLength + 1) * sizeof(WCHAR));
//         }
//
//         if (pParam->pszValue != NULL)
//         {
//             Edit_GetText(GetDlgItem(hwnd, IDC_PROPERTY_VALUE_EDIT),
//                          pParam->pszValue,
//                          iLength + 1);
//             pParam->cchValueLength = iLength;
//         }
//     }
// }
//
//
// static
// VOID
// WriteParameterArray(
//      HWND hwnd,
//      PPARAMETER_ARRAY ParamArray)
// {
//     SP_DEVINSTALL_PARAMS_W InstallParams;
//     PPARAMETER Param;
//     HKEY hDriverKey;
//     INT i;
//
//     if (ParamArray == NULL)
//         return;
//
//     hDriverKey = SetupDiOpenDevRegKey(ParamArray->DeviceInfoSet,
//                                       ParamArray->DeviceInfoData,
//                                       DICS_FLAG_GLOBAL,
//                                       0,
//                                       DIREG_DRV,
//                                       KEY_WRITE);
//     if (hDriverKey == INVALID_HANDLE_VALUE)
//     {
//         ERR("SetupDiOpenDevRegKey() failed\n");
//         return;
//     }
//
//     for (i = 0; i < ParamArray->dwCount; i++)
//     {
//         Param = &ParamArray->Array[i];
//
//         if (Param == ParamArray->pCurrentParam)
//         {
//             ReadParameterValue(hwnd, Param);
//         }
//
//         if (Param->bPresent)
//         {
//             TRACE("Set '%S' --> '%S'\n", Param->pszName, Param->pszValue);
//             RegSetValueExW(hDriverKey,
//                            Param->pszName,
//                            0,
//                            REG_SZ,
//                            (LPBYTE)Param->pszValue,
//                            (wcslen(Param->pszValue) + 1) * sizeof(WCHAR));
//         }
//         else
//         {
//             TRACE("Delete '%S'\n", Param->pszName);
//             RegDeleteValueW(hDriverKey,
//                             Param->pszName);
//         }
//     }
//
//     RegCloseKey(hDriverKey);
//
//     /* Notify the installer of changes to the properties */
//     InstallParams.cbSize = sizeof(SP_DEVINSTALL_PARAMS_W);
//     if (SetupDiGetDeviceInstallParamsW(ParamArray->DeviceInfoSet,
//                                        ParamArray->DeviceInfoData,
//                                        &InstallParams))
//     {
//         InstallParams.FlagsEx |= DI_FLAGSEX_PROPCHANGE_PENDING;
//
//         SetupDiSetDeviceInstallParamsW(ParamArray->DeviceInfoSet,
//                                        ParamArray->DeviceInfoData,
//                                        &InstallParams);
//     }
// }
//
//
// static
// VOID
// DisplayParameter(
//      HWND hwnd,
//      PPARAMETER Parameter)
// {
//     HWND hwndControl;
//     LONG_PTR Style;
//     INT idx;
//     DWORD i;
//
//     ShowWindow(GetDlgItem(hwnd, IDC_PROPERTY_PRESENT), (Parameter->bOptional) ? SW_SHOW : SW_HIDE);
//     ShowWindow(GetDlgItem(hwnd, IDC_PROPERTY_NOT_PRESENT), (Parameter->bOptional) ? SW_SHOW : SW_HIDE);
//     if (Parameter->bOptional)
//     {
//         if (Parameter->bPresent)
//             Button_SetCheck(GetDlgItem(hwnd, IDC_PROPERTY_PRESENT), BST_CHECKED);
//         else
//             Button_SetCheck(GetDlgItem(hwnd, IDC_PROPERTY_NOT_PRESENT), BST_CHECKED);
//     }
//
//     switch (Parameter->Type)
//     {
//         case INT_TYPE:
//         case LONG_TYPE:
//         case WORD_TYPE:
//         case DWORD_TYPE:
//             ShowWindow(GetDlgItem(hwnd, IDC_PROPERTY_VALUE_LIST), SW_HIDE);
//
//             hwndControl = GetDlgItem(hwnd, IDC_PROPERTY_VALUE_UPDN);
//
//             if (Parameter->Type != DWORD_TYPE)
//             {
//                 EnableWindow(hwndControl, Parameter->bPresent);
//                 ShowWindow(hwndControl, SW_SHOW);
//             }
//
//             if (Parameter->Type == WORD_TYPE || Parameter->Type == DWORD_TYPE)
//                 SendMessage(hwndControl, UDM_SETBASE, Parameter->iBase, 0);
//             else
//                 SendMessage(hwndControl, UDM_SETBASE, 10, 0);
//
//             if (Parameter->Type == INT_TYPE || Parameter->Type == LONG_TYPE)
//             {
//                 TRACE("SetMin %ld  SetMax %ld\n", Parameter->u.l.lMin, Parameter->u.l.lMax);
//                 SendMessage(hwndControl, UDM_SETRANGE32, Parameter->u.l.lMin, Parameter->u.l.lMax);
//             }
//             else if (Parameter->Type == WORD_TYPE)
//             {
//                 TRACE("SetMin %lu  SetMax %lu\n", Parameter->u.dw.dwMin, Parameter->u.dw.dwMax);
//                 SendMessage(hwndControl, UDM_SETRANGE32, (INT)Parameter->u.dw.dwMin, (INT)Parameter->u.dw.dwMax);
//             }
//
//             hwndControl = GetDlgItem(hwnd, IDC_PROPERTY_VALUE_EDIT);
//             EnableWindow(hwndControl, Parameter->bPresent);
//             ShowWindow(hwndControl, SW_SHOW);
//
//             Style = GetWindowLongPtr(hwndControl, GWL_STYLE);
//             Style |= ES_NUMBER;
//             SetWindowLongPtr(hwndControl, GWL_STYLE, Style);
//
//             Edit_LimitText(hwndControl, 0);
//
//             if (Parameter->pszValue)
//                 Edit_SetText(hwndControl, Parameter->pszValue);
//             else if (Parameter->pszDefault)
//                 Edit_SetText(hwndControl, Parameter->pszDefault);
//             break;
//
//         case EDIT_TYPE:
//             ShowWindow(GetDlgItem(hwnd, IDC_PROPERTY_VALUE_UPDN), SW_HIDE);
//             ShowWindow(GetDlgItem(hwnd, IDC_PROPERTY_VALUE_LIST), SW_HIDE);
//
//             hwndControl = GetDlgItem(hwnd, IDC_PROPERTY_VALUE_EDIT);
//             EnableWindow(hwndControl, Parameter->bPresent);
//             ShowWindow(hwndControl, SW_SHOW);
//
//             Style = GetWindowLongPtr(hwndControl, GWL_STYLE);
//             Style &= ~ES_NUMBER;
//             if (Parameter->bUpperCase)
//                 Style |= ES_UPPERCASE;
//             else
//                 Style &= ~ES_UPPERCASE;
//             SetWindowLongPtr(hwndControl, GWL_STYLE, Style);
//
//             Edit_LimitText(hwndControl, Parameter->iTextLimit);
//
//             if (Parameter->pszValue)
//                 Edit_SetText(hwndControl, Parameter->pszValue);
//             else if (Parameter->pszDefault)
//                 Edit_SetText(hwndControl, Parameter->pszDefault);
//             break;
//
//         case ENUM_TYPE:
//             ShowWindow(GetDlgItem(hwnd, IDC_PROPERTY_VALUE_EDIT), SW_HIDE);
//             ShowWindow(GetDlgItem(hwnd, IDC_PROPERTY_VALUE_UPDN), SW_HIDE);
//
//             hwndControl = GetDlgItem(hwnd, IDC_PROPERTY_VALUE_LIST);
//             EnableWindow(hwndControl, Parameter->bPresent);
//             ShowWindow(hwndControl, SW_SHOW);
//
//             ComboBox_ResetContent(hwndControl);
//
//             if (Parameter->pEnumOptions != NULL && Parameter->dwEnumOptions != 0)
//             {
//                 for (i = 0; i < Parameter->dwEnumOptions; i++)
//                 {
//                     ComboBox_AddString(hwndControl, Parameter->pEnumOptions[i].pszName);
//                 }
//             }
//
//             if (Parameter->pszValue)
//             {
//                 idx = FindEnumOption(Parameter, Parameter->pszValue);
//                 if (idx != CB_ERR)
//                     ComboBox_SetCurSel(hwndControl, idx);
//             }
//             else if (Parameter->pszDefault)
//             {
//                 idx = FindEnumOption(Parameter, Parameter->pszDefault);
//                 if (idx != CB_ERR)
//                     ComboBox_SetCurSel(hwndControl, idx);
//             }
//             break;
//
//         default:
//             break;
//     }
// }
//
//
// static
// BOOL
// OnInitDialog(
//     HWND hwnd,
//     WPARAM wParam,
//     LPARAM lParam)
// {
//     PPARAMETER_ARRAY pParamArray;
//     HWND hwndControl;
//     PWSTR pszText;
//     DWORD i;
//     INT idx;
//
//     TRACE("OnInitDialog()\n");
//
//     pParamArray = (PPARAMETER_ARRAY)((LPPROPSHEETPAGEW)lParam)->lParam;
//     if (pParamArray == NULL)
//     {
//         ERR("pParamArray is NULL\n");
//         return FALSE;
//     }
//
//     SetWindowLongPtr(hwnd, DWLP_USER, (LONG_PTR)pParamArray);
//
//     hwndControl = GetDlgItem(hwnd, IDC_PROPERTY_NAME);
//     if (hwndControl)
//     {
//         for (i = 0; i < pParamArray->dwCount; i++)
//         {
//             if (pParamArray->Array[i].pszDescription != NULL)
//                 pszText = pParamArray->Array[i].pszDescription;
//             else
//                 pszText = pParamArray->Array[i].pszName;
//
//             idx = ListBox_AddString(hwndControl, pszText);
//             if (idx != LB_ERR)
//                 ListBox_SetItemData(hwndControl, idx, (LPARAM)&pParamArray->Array[i]);
//         }
//
//         if (pParamArray->dwCount > 0)
//         {
//             ListBox_SetCurSel(hwndControl, 0);
//             pParamArray->pCurrentParam = (PPARAMETER)ListBox_GetItemData(hwndControl, 0);
//             DisplayParameter(hwnd, pParamArray->pCurrentParam);
//         }
//     }
//
//     return TRUE;
// }
//
//
// static
// VOID
// OnCommand(
//     HWND hwnd,
//     WPARAM wParam,
//     LPARAM lParam)
// {
//     PPARAMETER_ARRAY pParamArray;
//     INT iIndex;
//
//     TRACE("OnCommand()\n");
//
//     pParamArray = (PPARAMETER_ARRAY)GetWindowLongPtr(hwnd, DWLP_USER);
//     if (pParamArray == NULL)
//     {
//         ERR("pParamArray is NULL\n");
//         return;
//     }
//
//     if ((LOWORD(wParam) == IDC_PROPERTY_NAME) && (HIWORD(wParam) == LBN_SELCHANGE))
//     {
//         if (pParamArray->pCurrentParam != NULL)
//         {
//             ReadParameterValue(hwnd, pParamArray->pCurrentParam);
//         }
//
//         iIndex = ListBox_GetCurSel((HWND)lParam);
//         if (iIndex != LB_ERR && iIndex < pParamArray->dwCount)
//         {
//             pParamArray->pCurrentParam = (PPARAMETER)ListBox_GetItemData((HWND)lParam, iIndex);
//             DisplayParameter(hwnd, pParamArray->pCurrentParam);
//         }
//     }
//     else if ((LOWORD(wParam) == IDC_PROPERTY_PRESENT) && (HIWORD(wParam) == BN_CLICKED))
//     {
//         EnableWindow(GetDlgItem(hwnd, IDC_PROPERTY_VALUE_EDIT), TRUE);
//         EnableWindow(GetDlgItem(hwnd, IDC_PROPERTY_VALUE_UPDN), TRUE);
//         EnableWindow(GetDlgItem(hwnd, IDC_PROPERTY_VALUE_LIST), TRUE);
//         pParamArray->pCurrentParam->bPresent = TRUE;
//     }
//     else if ((LOWORD(wParam) == IDC_PROPERTY_NOT_PRESENT) && (HIWORD(wParam) == BN_CLICKED))
//     {
//         EnableWindow(GetDlgItem(hwnd, IDC_PROPERTY_VALUE_EDIT), FALSE);
//         EnableWindow(GetDlgItem(hwnd, IDC_PROPERTY_VALUE_UPDN), FALSE);
//         EnableWindow(GetDlgItem(hwnd, IDC_PROPERTY_VALUE_LIST), FALSE);
//         pParamArray->pCurrentParam->bPresent = FALSE;
//     }
// }
//
//
// static
// VOID
// OnNotify(
//     HWND hwnd,
//     WPARAM wParam,
//     LPARAM lParam)
// {
//     PPARAMETER_ARRAY pParamArray;
//
//     TRACE("OnNotify()\n");
//
//     pParamArray = (PPARAMETER_ARRAY)GetWindowLongPtr(hwnd, DWLP_USER);
//     if (pParamArray == NULL)
//     {
//         ERR("pParamArray is NULL\n");
//         return;
//     }
//
//     if (((LPNMHDR)lParam)->code == (UINT)PSN_APPLY)
//     {
//         TRACE("PSN_APPLY!\n");
//         WriteParameterArray(hwnd, pParamArray);
//     }
//     else if (((LPNMHDR)lParam)->code == (UINT)UDN_DELTAPOS)
//     {
//         LPNMUPDOWN pUpDown = (LPNMUPDOWN)lParam;
//         pUpDown->iDelta *= pParamArray->pCurrentParam->iStep;
//     }
// }
//
//
// static
// VOID
// OnDestroy(
//     HWND hwnd)
// {
//     PPARAMETER_ARRAY pParamArray;
//
//     TRACE("OnDestroy()\n");
//
//     pParamArray = (PPARAMETER_ARRAY)GetWindowLongPtr(hwnd, DWLP_USER);
//     if (pParamArray == NULL)
//     {
//         ERR("pParamArray is NULL\n");
//         return;
//     }
//
//     FreeParameterArray(pParamArray);
//     SetWindowLongPtr(hwnd, DWLP_USER, (LONG_PTR)NULL);
// }
//
//
// static
// INT_PTR
// __stdcall
// NetPropertyPageDlgProc(
//     HWND hwnd,
//     UINT uMsg,
//     WPARAM wParam,
//     LPARAM lParam)
// {
//     switch (uMsg)
//     {
//         case WM_INITDIALOG:
//             return OnInitDialog(hwnd, wParam, lParam);
//
//         case WM_COMMAND:
//             OnCommand(hwnd, wParam, lParam);
//             break;
//
//         case WM_NOTIFY:
//             OnNotify(hwnd, wParam, lParam);
//             break;
//
//         case WM_DESTROY:
//             OnDestroy(hwnd);
//             break;
//
//         default:
//             break;
//     }
//
//     return FALSE;
// }
//
//
// BOOL
// WINAPI
// NetPropPageProvider(
//     PSP_PROPSHEETPAGE_REQUEST lpPropSheetPageRequest,
//     LPFNADDPROPSHEETPAGE lpfnAddPropSheetPageProc,
//     LPARAM lParam)
// {
//     PROPSHEETPAGEW PropSheetPage;
//     HPROPSHEETPAGE hPropSheetPage;
//     PPARAMETER_ARRAY ParameterArray = NULL;
//
//     TRACE("NetPropPageProvider(%p %p %lx)\n",
//           lpPropSheetPageRequest, lpfnAddPropSheetPageProc, lParam);
//
//     if (!BuildParameterArray(lpPropSheetPageRequest->DeviceInfoSet,
//                              lpPropSheetPageRequest->DeviceInfoData,
//                              &ParameterArray))
//         return FALSE;
//
//     if (lpPropSheetPageRequest->PageRequested == SPPSR_ENUM_ADV_DEVICE_PROPERTIES)
//     {
//         TRACE("SPPSR_ENUM_ADV_DEVICE_PROPERTIES\n");
//
//         PropSheetPage.dwSize = sizeof(PROPSHEETPAGEW);
//         PropSheetPage.dwFlags = 0;
//         PropSheetPage.hInstance = netcfgx_hInstance;
//         PropSheetPage.u.pszTemplate = MAKEINTRESOURCE(IDD_NET_PROPERTY_DLG);
//         PropSheetPage.pfnDlgProc = NetPropertyPageDlgProc;
//         PropSheetPage.lParam = (LPARAM)ParameterArray;
//         PropSheetPage.pfnCallback = NULL;
//
//         hPropSheetPage = CreatePropertySheetPageW(&PropSheetPage);
//         if (hPropSheetPage == NULL)
//         {
//             ERR("CreatePropertySheetPageW() failed!\n");
//             return FALSE;
//         }
//
//         if (!(*lpfnAddPropSheetPageProc)(hPropSheetPage, lParam))
//         {
//             ERR("lpfnAddPropSheetPageProc() failed!\n");
//             DestroyPropertySheetPage(hPropSheetPage);
//             return FALSE;
//         }
//     }
//
//     TRACE("Done!\n");
//
//     return TRUE;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
