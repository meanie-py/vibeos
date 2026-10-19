// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_DB0A80B41F41: [u16; 33] = [
    0x0053u16, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x004D,
    0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0041,
    0x0044, 0x0073, 0x005C, 0x0050, 0x0072, 0x006F, 0x0076, 0x0069, 0x0064, 0x0065,
    0x0072, 0x0073, 0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub unsafe extern "system" fn ADsGetObject(path: LPCWSTR, riid: *const IID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let mut hr: HRESULT;

    hr = ADsOpenObject(path, core::ptr::null_mut(), core::ptr::null_mut(), ADS_SECURE_AUTHENTICATION as DWORD, riid, obj);
    if hr != S_OK {
        hr = ADsOpenObject(path, core::ptr::null_mut(), core::ptr::null_mut(), 0 as DWORD, riid, obj);
    }
    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn ADsBuildEnumerator(
    pADsContainer: *mut IADsContainer,
    ppEnumVariant: *mut *mut IEnumVARIANT,
) -> HRESULT {
    // FIXME("(%p)->(%p)!stub\n",pADsContainer, ppEnumVariant);
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn ADsFreeEnumerator(pEnumVariant: *mut IEnumVARIANT) -> HRESULT {
    // FIXME("(%p)!stub\n",pEnumVariant);
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn ADsEnumerateNext(
    pEnumVariant: *mut IEnumVARIANT,
    cElements: ULONG,
    pvar: *mut VARIANT,
    pcElementsFetched: *mut ULONG,
) -> HRESULT {
    // FIXME("(%p)->(%lu, %p, %p)!stub\n",pEnumVariant, cElements, pvar, pcElementsFetched);
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn ADsBuildVarArrayStr(str: *mut LPWSTR, count: DWORD, var: *mut VARIANT) -> HRESULT {
    let mut hr: HRESULT = 0;
    let mut sa: *mut SAFEARRAY;
    let mut idx: LONG;
    let mut end: LONG = count as LONG;

    if var.is_null() {
        return E_ADS_BAD_PARAMETER;
    }

    sa = SafeArrayCreateVector(VT_VARIANT, 0, count);
    if sa.is_null() {
        return E_OUTOFMEMORY;
    }

    VariantInit(var);
    idx = 0;
    'fail: {
        while idx < end {
            let mut item: VARIANT = core::mem::zeroed();

            item.n1.n2.vt = VT_BSTR as VARTYPE;
            item.n1.n2.n3.bstrVal = SysAllocString(*str.offset(idx as isize));
            if item.n1.n2.n3.bstrVal.is_null() {
                hr = E_OUTOFMEMORY;
                break 'fail;
            }

            hr = SafeArrayPutElement(
                sa,
                core::ptr::addr_of_mut!(idx),
                core::ptr::addr_of_mut!(item) as *mut core::ffi::c_void,
            );
            SysFreeString(item.n1.n2.n3.bstrVal);
            if hr != S_OK {
                break 'fail;
            }
            idx += 1;
        }

        (*var).n1.n2.vt = (VT_ARRAY | VT_VARIANT) as VARTYPE;
        (*var).n1.n2.n3.parray = sa;
        return S_OK;
    }

    SafeArrayDestroy(sa);
    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn ADsBuildVarArrayInt(
    lpdwObjectTypes: LPDWORD,
    dwObjectTypes: DWORD,
    pvar: *mut VARIANT,
) -> HRESULT {
    // FIXME("(%p, %ld, %p)!stub\n",lpdwObjectTypes, dwObjectTypes, pvar);
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn ADsOpenObject(
    path: LPCWSTR,
    user: LPCWSTR,
    password: LPCWSTR,
    reserved: DWORD,
    riid: REFIID,
    obj: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let mut hr: HRESULT;
    let mut hkey: HKEY;
    let mut hprov: HKEY;
    let mut provider: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut progid: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut idx: DWORD = 0;

    if path.is_null() || riid.is_null() || obj.is_null() {
        return E_INVALIDARG;
    }

    hr = E_FAIL;

    hkey = core::ptr::null_mut();
    if RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        RUSTCONV_WSTR_DB0A80B41F41.as_ptr(),
        0,
        KEY_READ,
        &raw mut hkey,
    ) != 0
    {
        return hr;
    }

    loop {
        let fresh15 = idx;
        idx = idx.wrapping_add(1);
        if RegEnumKeyW(
            hkey,
            fresh15,
            &raw mut provider as LPWSTR,
            ARRAY_SIZE!(provider) as DWORD,
        ) != 0
        {
            break;
        }

        if wcsnicmp(
            path,
            &raw mut provider as *mut WCHAR as LPCWSTR,
            wcslen(&raw mut provider as *mut WCHAR as LPCWSTR) as usize,
        ) == 0
            && *path.add(wcslen(&raw mut provider as *mut WCHAR as LPCWSTR) as usize)
                == ':' as WCHAR
        {
            let mut size: LONG;

            hprov = core::ptr::null_mut();
            if RegOpenKeyExW(
                hkey,
                &raw mut provider as *mut WCHAR as LPCWSTR,
                0,
                KEY_READ,
                &raw mut hprov,
            ) != 0
            {
                break;
            }

            size = ARRAY_SIZE!(progid) as LONG;
            if RegQueryValueW(
                hprov,
                core::ptr::null(),
                &raw mut progid as LPWSTR,
                &raw mut size,
            ) == 0
            {
                let mut clsid: CLSID = core::mem::zeroed();

                if CLSIDFromProgID(
                    &raw mut progid as *mut WCHAR as LPCOLESTR,
                    &raw mut clsid,
                ) == S_OK
                {
                    let mut adsopen: *mut IADsOpenDSObject = core::ptr::null_mut();
                    let mut disp: *mut IDispatch = core::ptr::null_mut();

                    if CoCreateInstance(
                        &raw mut clsid,
                        core::ptr::null_mut(),
                        CLSCTX_INPROC_SERVER,
                        &raw const IID_IADsOpenDSObject,
                        &raw mut adsopen as *mut LPVOID,
                    ) == S_OK
                    {
                        let mut bpath: BSTR = core::ptr::null_mut();
                        let mut buser: BSTR = core::ptr::null_mut();
                        let mut bpassword: BSTR = core::ptr::null_mut();

                        bpath = SysAllocString(path);
                        buser = SysAllocString(user);
                        bpassword = SysAllocString(password);

                        hr = IADsOpenDSObject_OpenDSObject(
                            adsopen,
                            bpath,
                            buser,
                            bpassword,
                            reserved as LONG,
                            &raw mut disp,
                        );
                        if hr == S_OK {
                            hr = IDispatch_QueryInterface(disp, riid, obj);
                            IDispatch_Release(disp);
                        }

                        SysFreeString(bpath);
                        SysFreeString(buser);
                        SysFreeString(bpassword);

                        IADsOpenDSObject_Release(adsopen);
                    }
                }
            }

            RegCloseKey(hprov);
            break;
        }
    }

    RegCloseKey(hkey);

    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn ADsSetLastError(
    dwErr: DWORD,
    pszError: LPWSTR,
    pszProvider: LPWSTR,
) {
    // FIXME("(%ld,%p,%p)!stub\n", dwErr, pszError, pszProvider);
}

#[no_mangle]
pub unsafe extern "system" fn ADsGetLastError(
    perror: LPDWORD,
    errorbuf: LPWSTR,
    errorbuflen: DWORD,
    namebuf: LPWSTR,
    namebuflen: DWORD,
) -> HRESULT {
    // FIXME("(%p,%p,%ld,%p,%ld)!stub\n", perror, errorbuf, errorbuflen, namebuf, namebuflen);
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn AllocADsMem(cb: DWORD) -> LPVOID {
    malloc((cb) as _)
}

#[no_mangle]
pub unsafe extern "system" fn FreeADsMem(pMem: LPVOID) -> i32 {
    free(pMem as *mut ::core::ffi::c_void);
    return (TRUE as i32);
}

#[no_mangle]
pub unsafe extern "system" fn ReallocADsMem(pOldMem: LPVOID, cbOld: DWORD, cbNew: DWORD) -> LPVOID {
    return realloc(pOldMem as *mut core::ffi::c_void, cbNew as usize);
}

#[no_mangle]
pub unsafe extern "system" fn AllocADsStr(pStr: LPWSTR) -> LPWSTR {
    // TRACE("(%p)\n", pStr);
    wcsdup(pStr)
}

#[no_mangle]
pub unsafe extern "system" fn FreeADsStr(pStr: LPWSTR) -> BOOL {
    // TRACE("(%p)\n", pStr);
    FreeADsMem((pStr).cast())
}

#[no_mangle]
pub unsafe extern "system" fn ReallocADsStr(ppStr: *mut LPWSTR, pStr: LPWSTR) -> BOOL {
    // FIXME("(%p,%p)!stub\n",*ppStr, pStr);
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn ADsEncodeBinaryData(
    pbSrcData: PBYTE,
    dwSrcLen: DWORD,
    ppszDestData: *mut LPWSTR,
) -> HRESULT {
    // FIXME("(%p,%ld,%p)!stub\n", pbSrcData, dwSrcLen, *ppszDestData);
    E_NOTIMPL
}
