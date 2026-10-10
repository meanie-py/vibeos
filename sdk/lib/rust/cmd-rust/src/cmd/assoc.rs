use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// assoc.c preamble: no file-scope data declarations (no typedefs, #defines, enums, or tables). Only static functions follow, and those are emitted by the function pass.

unsafe extern "C" fn PrintAssociationEx(mut hKeyClasses: HKEY, mut pszExtension: PCTSTR) -> LONG {
    let mut lRet: LONG;
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut dwFileTypeLen: DWORD = 0;
    let mut pszFileType: PTSTR;

    lRet = RegOpenKeyEx(hKeyClasses, pszExtension, 0, KEY_QUERY_VALUE, core::ptr::addr_of_mut!(hKey));
    if (lRet) as u32 != ERROR_SUCCESS {
        if (lRet) as u32 != ERROR_FILE_NOT_FOUND {
            ErrorMessage((lRet) as DWORD, core::ptr::null_mut());
        }
        return lRet;
    }

    /* Obtain the string length */
    lRet = RegQueryValueEx(hKey, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::addr_of_mut!(dwFileTypeLen));

    /* If there is no default value, don't display it */
    if (lRet) as u32 == ERROR_FILE_NOT_FOUND {
        RegCloseKey(hKey);
        return lRet;
    }
    if (lRet) as u32 != ERROR_SUCCESS {
        ErrorMessage((lRet) as DWORD, core::ptr::null_mut());
        RegCloseKey(hKey);
        return lRet;
    }

    dwFileTypeLen = dwFileTypeLen.wrapping_add(1);
    pszFileType = cmd_alloc!((dwFileTypeLen as usize).wrapping_mul(core::mem::size_of::<TCHAR>() as usize)) as PTSTR;
    if pszFileType.is_null() {
        WARN!("Cannot allocate memory for pszFileType!\n\0".as_ptr() as *const core::ffi::c_char);
        RegCloseKey(hKey);
        return (ERROR_NOT_ENOUGH_MEMORY) as LONG;
    }

    /* Obtain the actual file type */
    lRet = RegQueryValueEx(hKey, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), pszFileType as LPBYTE, core::ptr::addr_of_mut!(dwFileTypeLen));
    RegCloseKey(hKey);

    if (lRet) as u32 != ERROR_SUCCESS {
        ErrorMessage((lRet) as DWORD, core::ptr::null_mut());
        cmd_free!(pszFileType);
        return lRet;
    }

    /* If there is a default key, display the relevant information */
    if dwFileTypeLen != 0 {
        let fmt: [u16; 7] = ['%' as u16, 's' as u16, '=' as u16, '%' as u16, 's' as u16, '\n' as u16, 0];
        ConOutPrintf!((fmt.as_ptr()), pszExtension, pszFileType);
    }

    cmd_free!(pszFileType);
    return (ERROR_SUCCESS) as LONG;
}

unsafe extern "C" fn PrintAssociation(pszExtension: PCTSTR) -> LONG {
    let mut lRet: LONG;
    let mut hKeyClasses: HKEY = core::ptr::null_mut();
    let class_name: [u16; 17] = [
        'S' as u16, 'O' as u16, 'F' as u16, 'T' as u16, 'W' as u16, 'A' as u16, 'R' as u16, 'E' as u16,
        '\\' as u16, 'C' as u16, 'l' as u16, 'a' as u16, 's' as u16, 's' as u16, 'e' as u16, 's' as u16,
        0,
    ];

    lRet = RegOpenKeyEx(
        HKEY_LOCAL_MACHINE,
        class_name.as_ptr(),
        0,
        KEY_ENUMERATE_SUB_KEYS,
        core::ptr::addr_of_mut!(hKeyClasses),
    );
    if (lRet) as u32 != ERROR_SUCCESS {
        ErrorMessage((lRet) as DWORD, core::ptr::null_mut());
        return lRet;
    }

    lRet = PrintAssociationEx(hKeyClasses, pszExtension);

    RegCloseKey(hKeyClasses);
    return lRet;
}

unsafe extern "C" fn PrintAllAssociations() -> LONG {
    let mut lRet: LONG;
    let mut hKeyClasses: HKEY = core::ptr::null_mut();
    let mut dwKeyCtr: DWORD;
    let mut dwNumKeys: DWORD = 0;
    let mut dwExtLen: DWORD = 0;
    let mut pszExtName: PTSTR;

    let subkey: [u16; 17] = [
        'S' as u16, 'O' as u16, 'F' as u16, 'T' as u16, 'W' as u16, 'A' as u16,
        'R' as u16, 'E' as u16, '\\' as u16, 'C' as u16, 'l' as u16, 'a' as u16,
        's' as u16, 's' as u16, 'e' as u16, 's' as u16, 0,
    ];

    lRet = RegOpenKeyEx(
        HKEY_LOCAL_MACHINE,
        subkey.as_ptr() as PCTSTR,
        0,
        KEY_QUERY_VALUE | KEY_ENUMERATE_SUB_KEYS,
        core::ptr::addr_of_mut!(hKeyClasses),
    );
    if (lRet) as u32 != ERROR_SUCCESS {
        ErrorMessage(lRet as DWORD, core::ptr::null_mut());
        return lRet;
    }

    lRet = RegQueryInfoKey(
        hKeyClasses,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(dwNumKeys),
        core::ptr::addr_of_mut!(dwExtLen),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    if (lRet) as u32 != ERROR_SUCCESS {
        ErrorMessage(lRet as DWORD, core::ptr::null_mut());
        RegCloseKey(hKeyClasses);
        return lRet;
    }

    dwExtLen = dwExtLen.wrapping_add(1);
    pszExtName = cmd_alloc!(
        (dwExtLen as usize).wrapping_mul(core::mem::size_of::<TCHAR>()),
    ) as PTSTR;
    if pszExtName.is_null() {
        RegCloseKey(hKeyClasses);
        return (ERROR_NOT_ENOUGH_MEMORY) as LONG;
    }

    dwKeyCtr = 0;
    while dwKeyCtr < dwNumKeys {
        let mut dwBufSize: DWORD = dwExtLen;
        lRet = RegEnumKeyEx(
            hKeyClasses,
            dwKeyCtr,
            pszExtName,
            core::ptr::addr_of_mut!(dwBufSize),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );

        if (lRet) as u32 == ERROR_SUCCESS || (lRet) as u32 == ERROR_MORE_DATA {
            if *pszExtName == '.' as u16 {
                PrintAssociationEx(hKeyClasses, pszExtName as PCTSTR);
            }
        } else {
            ErrorMessage(lRet as DWORD, core::ptr::null_mut());
            cmd_free!(pszExtName);
            RegCloseKey(hKeyClasses);
            return lRet;
        }
        dwKeyCtr = dwKeyCtr.wrapping_add(1);
    }

    RegCloseKey(hKeyClasses);

    cmd_free!(pszExtName);
    return (ERROR_SUCCESS) as LONG;
}

unsafe extern "C" fn AddAssociation(pszExtension: PCTSTR, pszType: PCTSTR) -> LONG {
    let mut lRet: LONG;
    let mut hKeyClasses: HKEY = core::ptr::null_mut();
    let mut hKey: HKEY = core::ptr::null_mut();
    let SubKeyClasses: [u16; 17] = ['S' as u16, 'O' as u16, 'F' as u16, 'T' as u16, 'W' as u16, 'A' as u16, 'R' as u16, 'E' as u16, '\\' as u16, 'C' as u16, 'l' as u16, 'a' as u16, 's' as u16, 's' as u16, 'e' as u16, 's' as u16, 0];

    lRet = RegOpenKeyEx(HKEY_LOCAL_MACHINE, SubKeyClasses.as_ptr(), 0,
                        KEY_CREATE_SUB_KEY, &mut hKeyClasses);
    if (lRet) as u32 != ERROR_SUCCESS {
        ErrorMessage((lRet) as DWORD, core::ptr::null_mut());
        return lRet;
    }

    lRet = RegCreateKeyEx(hKeyClasses, pszExtension, 0, core::ptr::null_mut(), REG_OPTION_NON_VOLATILE,
                          KEY_SET_VALUE, core::ptr::null_mut(), &mut hKey, core::ptr::null_mut());
    RegCloseKey(hKeyClasses);

    if (lRet) as u32 != ERROR_SUCCESS {
        ErrorMessage((lRet) as DWORD, core::ptr::null_mut());
        return lRet;
    }

    lRet = RegSetValueEx(hKey, core::ptr::null(), 0, REG_SZ,
                         pszType as LPBYTE, (((_tcslen(pszType) + 1) as DWORD as usize).wrapping_mul(core::mem::size_of::<TCHAR>())) as DWORD);
    RegCloseKey(hKey);

    if (lRet) as u32 != ERROR_SUCCESS {
        ErrorMessage((lRet) as DWORD, core::ptr::null_mut());
        return lRet;
    }

    return (ERROR_SUCCESS) as LONG;
}

unsafe extern "C" fn RemoveAssociation(mut pszExtension: PCTSTR) -> LONG {
    let mut lRet: LONG;
    let mut hKeyClasses: HKEY = core::ptr::null_mut();

    lRet = RegOpenKeyEx(HKEY_LOCAL_MACHINE, (&[0x0053, 0x004f, 0x0046, 0x0054, 0x0057, 0x0041, 0x0052, 0x0045, 0x005c, 0x0043, 0x006c, 0x0061, 0x0073, 0x0073, 0x0065, 0x0073, 0x0000u16]).as_ptr() /* _T "SOFTWARE\\Classes" */, 0,
                        KEY_QUERY_VALUE, core::ptr::addr_of_mut!(hKeyClasses));
    if (lRet) as u32 != ERROR_SUCCESS {
        ErrorMessage((lRet) as DWORD, core::ptr::null_mut());
        return lRet;
    }

    lRet = RegDeleteKey(hKeyClasses, pszExtension);
    RegCloseKey(hKeyClasses);

    if (lRet) as u32 != ERROR_SUCCESS {
        if (lRet) as u32 != ERROR_FILE_NOT_FOUND {
            ErrorMessage((lRet) as DWORD, core::ptr::null_mut());
        }
        return lRet;
    }

    return (ERROR_SUCCESS) as LONG;
}

#[no_mangle]
pub unsafe extern "C" fn CommandAssoc(mut param: LPTSTR) -> INT {
    let mut retval: INT = 0;
    let mut pEqualSign: PTCHAR;
    let slash_q: [u16; 3] = ['/' as u16, '?' as u16, 0];

    'quit: {
        /* Print help */
        if !(_tcsncmp(param, slash_q.as_ptr(), 2) == 0) {
            ConOutResPaging((TRUE) as BOOL, STRING_ASSOC_HELP);
            return 0;
        }

        /* Print all associations if no parameter has been specified */
        if !(*param == 0) {
        } else {
            PrintAllAssociations();
            break 'quit;
        }

        pEqualSign = _tcschr(param, '=' as u16);
        if !pEqualSign.is_null() {
            let mut pszFileType: PTSTR = pEqualSign.offset(1);

            /* NULL-terminate at the equals sign */
            *pEqualSign = 0;

            /* If the equals sign is the last character
             * in the string, delete the association. */
            if *pszFileType == 0 {
                retval = RemoveAssociation(param);
            } else {
                /* Otherwise, add the association and print it out */
                retval = AddAssociation(param, pszFileType);
                PrintAssociation(param);
            }

            if (retval) as u32 != ERROR_SUCCESS {
                if (retval) as u32 != ERROR_FILE_NOT_FOUND {
                    ConErrResPrintf!(STRING_ERROR_WHILE_PROCESSING, param);
                }
                // retval = 1; /* Fixup the error value */
            }
        } else {
            /* No equals sign, print the association */
            retval = PrintAssociation(param);
            if (retval) as u32 != ERROR_SUCCESS {
                ConErrResPrintf!(STRING_ASSOC_ERROR, param);
                retval = 1; /* Fixup the error value */
            }
        }
    }

    // Quit:
    if BatType != CMD_TYPE {
        if retval != 0 {
            nErrorLevel = retval;
        }
    } else {
        nErrorLevel = retval;
    }

    return retval;
}
