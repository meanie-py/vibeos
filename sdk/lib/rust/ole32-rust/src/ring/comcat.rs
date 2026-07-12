use crate::prelude::*;
use core::ffi::c_void;

extern "C" {

    static IID_IUnknown: GUID;
    static IID_ICatRegister: GUID;
    static IID_ICatInformation: GUID;
    fn open_classes_key(hKey: HKEY, lpSubKey: LPCWSTR, samDesired: u32, phkResult: *mut HKEY) -> LSTATUS;
    static IID_IEnumCATEGORYINFO: GUID;
}
extern "C" {
    fn RegCloseKey(hKey: HKEY) -> i32;
    fn StringFromGUID2(rguid: *const GUID, lpsz: *mut u16, cchMax: i32) -> i32;
    fn create_classes_key(hkey: HKEY, name: LPCWSTR, access: u32, retkey: *mut HKEY) -> LSTATUS;
    fn RegDeleteKeyW(hKey: HKEY, lpSubKey: *const u16) -> LSTATUS;
    fn RegQueryValueExW( hKey: HKEY, lpValueName: *const u16, lpReserved: *mut u32, lpType: *mut u32, lpData: LPBYTE, lpcbData: *mut u32, ) -> LSTATUS;
    fn wsprintfW(output: *mut u16, format: *const u16, ...) -> i32;
    fn RegSetValueExW( hkey: HKEY, lpValueName: LPCWSTR, Reserved: u32, dwType: u32, lpData: *const u8, cbData: u32, ) -> i32;
    fn lstrcatW(lpString1: *mut u16, lpString2: *const u16) -> *mut u16;
    fn RegEnumKeyExW( hKey: HKEY, dwIndex: u32, lpName: *mut u16, lpcchName: *mut u32, lpReserved: *mut u32, lpClass: *mut u16, lpcchClass: *mut u32, lpftLastWriteTime: LPVOID, ) -> LSTATUS;
    fn CLSIDFromString(lpsz: LPCWSTR, pclsid: *mut GUID) -> HRESULT;
}

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct ComCatMgrImpl {
    pub ICatRegister_iface: ICatRegister,
    pub ICatInformation_iface: ICatInformation,
}

pub static mut COMCAT_ComCatMgr: ComCatMgrImpl = ComCatMgrImpl {
    ICatRegister_iface: ICatRegister {
        lpVtbl: &COMCAT_ICatRegister_Vtbl as *const ICatRegisterVtbl,
    },
    ICatInformation_iface: ICatInformation {
        lpVtbl: &COMCAT_ICatInformation_Vtbl as *const ICatInformationVtbl,
    },
};

#[repr(C)]
pub struct class_categories {
    pub size: u32,
    pub impl_offset: u32,
    pub req_offset: u32,
}

pub static comcat_keyname: [u16; 21] = [
    'C' as u16, 'o' as u16, 'm' as u16, 'p' as u16, 'o' as u16, 'n' as u16, 'e' as u16, 'n' as u16,
    't' as u16, ' ' as u16, 'C' as u16, 'a' as u16, 't' as u16, 'e' as u16, 'g' as u16, 'o' as u16,
    'r' as u16, 'i' as u16, 'e' as u16, 's' as u16, 0,
];

pub static impl_keyname: [u16; 24] = [
    'I' as u16, 'm' as u16, 'p' as u16, 'l' as u16, 'e' as u16, 'm' as u16, 'e' as u16, 'n' as u16,
    't' as u16, 'e' as u16, 'd' as u16, ' ' as u16, 'C' as u16, 'a' as u16, 't' as u16, 'e' as u16,
    'g' as u16, 'o' as u16, 'r' as u16, 'i' as u16, 'e' as u16, 's' as u16, 0,
];

pub static req_keyname: [u16; 21] = [
    'R' as u16, 'e' as u16, 'q' as u16, 'u' as u16, 'i' as u16, 'r' as u16, 'e' as u16, 'd' as u16,
    ' ' as u16, 'C' as u16, 'a' as u16, 't' as u16, 'e' as u16, 'g' as u16, 'o' as u16, 'r' as u16,
    'i' as u16, 'e' as u16, 's' as u16, 0,
];

pub static clsid_keyname: [u16; 6] = [
    'C' as u16, 'L' as u16, 'S' as u16, 'I' as u16, 'D' as u16, 0,
];

pub static COMCAT_ICatRegister_Vtbl: ICatRegisterVtbl = ICatRegisterVtbl {
    QueryInterface: COMCAT_ICatRegister_QueryInterface,
    AddRef: COMCAT_ICatRegister_AddRef,
    Release: COMCAT_ICatRegister_Release,
    RegisterCategories: COMCAT_ICatRegister_RegisterCategories,
    UnRegisterCategories: COMCAT_ICatRegister_UnRegisterCategories,
    RegisterClassImplCategories: COMCAT_ICatRegister_RegisterClassImplCategories,
    UnRegisterClassImplCategories: COMCAT_ICatRegister_UnRegisterClassImplCategories,
    RegisterClassReqCategories: COMCAT_ICatRegister_RegisterClassReqCategories,
    UnRegisterClassReqCategories: COMCAT_ICatRegister_UnRegisterClassReqCategories,
};

pub static COMCAT_ICatInformation_Vtbl: ICatInformationVtbl = ICatInformationVtbl {
    QueryInterface: COMCAT_ICatInformation_QueryInterface,
    AddRef: COMCAT_ICatInformation_AddRef,
    Release: COMCAT_ICatInformation_Release,
    EnumCategories: COMCAT_ICatInformation_EnumCategories,
    GetCategoryDesc: COMCAT_ICatInformation_GetCategoryDesc,
    EnumClassesOfCategories: COMCAT_ICatInformation_EnumClassesOfCategories,
    IsClassOfCategories: COMCAT_ICatInformation_IsClassOfCategories,
    EnumImplCategoriesOfClass: COMCAT_ICatInformation_EnumImplCategoriesOfClass,
    EnumReqCategoriesOfClass: COMCAT_ICatInformation_EnumReqCategoriesOfClass,
};

#[repr(C)]
pub struct IEnumCATEGORYINFOImpl {
    pub IEnumCATEGORYINFO_iface: IEnumCATEGORYINFO,
    pub ref_: i32,
    pub lcid: LCID,
    pub key: HKEY,
    pub next_index: u32,
}

pub static COMCAT_IEnumCATEGORYINFO_Vtbl: IEnumCATEGORYINFOVtbl = IEnumCATEGORYINFOVtbl {
    QueryInterface: COMCAT_IEnumCATEGORYINFO_QueryInterface,
    AddRef: COMCAT_IEnumCATEGORYINFO_AddRef,
    Release: COMCAT_IEnumCATEGORYINFO_Release,
    Next: COMCAT_IEnumCATEGORYINFO_Next,
    Skip: COMCAT_IEnumCATEGORYINFO_Skip,
    Reset: COMCAT_IEnumCATEGORYINFO_Reset,
    Clone: COMCAT_IEnumCATEGORYINFO_Clone,
};

#[repr(C)]
pub struct CLSID_IEnumGUIDImpl {
    pub IEnumGUID_iface: IEnumGUID,
    pub ref_: i32,
    pub categories: *mut class_categories,
    pub key: HKEY,
    pub next_index: u32,
}

pub static CLSIDEnumGUIDVtbl: IEnumGUIDVtbl = IEnumGUIDVtbl {
    QueryInterface: CLSIDEnumGUID_QueryInterface,
    AddRef: CLSIDEnumGUID_AddRef,
    Release: CLSIDEnumGUID_Release,
    Next: CLSIDEnumGUID_Next,
    Skip: CLSIDEnumGUID_Skip,
    Reset: CLSIDEnumGUID_Reset,
    Clone: CLSIDEnumGUID_Clone,
};

#[repr(C)]
pub struct CATID_IEnumGUIDImpl {
    pub IEnumGUID_iface: IEnumGUID,
    pub ref_: i32,
    pub keyname: [u16; 68],
    pub key: HKEY,
    pub next_index: u32,
}

pub static CATIDEnumGUIDVtbl: IEnumGUIDVtbl = IEnumGUIDVtbl {
    QueryInterface: CATIDEnumGUID_QueryInterface,
    AddRef: CATIDEnumGUID_AddRef,
    Release: CATIDEnumGUID_Release,
    Next: CATIDEnumGUID_Next,
    Skip: CATIDEnumGUID_Skip,
    Reset: CATIDEnumGUID_Reset,
    Clone: CATIDEnumGUID_Clone,
};
unsafe fn COMCAT_RegisterClassCategories(
    rclsid: *const GUID,
    type_: LPCWSTR,
    cCategories: u32,
    rgcatid: *const GUID,
) -> HRESULT {
    let mut keyname: [u16; 39] = [0; 39];
    let mut res: HRESULT;
    let mut clsid_key: HKEY = core::ptr::null_mut();
    let mut class_key: HKEY = core::ptr::null_mut();
    let mut type_key: HKEY = core::ptr::null_mut();

    if cCategories != 0 && rgcatid.is_null() {
        return 0x80004003u32 as HRESULT;
    }

    res = StringFromGUID2(rclsid, keyname.as_mut_ptr(), 39);
    if FAILED(res) {
        return res;
    }

    res = create_classes_key(
        0x80000000u32 as HKEY,
        clsid_keyname.as_ptr(),
        KEY_READ | KEY_WRITE,
        &mut clsid_key,
    );
    if res != ERROR_SUCCESS as HRESULT {
        return 0x80004005u32 as HRESULT;
    }

    res = create_classes_key(clsid_key, keyname.as_ptr(), KEY_READ | KEY_WRITE, &mut class_key);
    if res == ERROR_SUCCESS as HRESULT {
        res = create_classes_key(class_key, type_, KEY_READ | KEY_WRITE, &mut type_key);
        if res == ERROR_SUCCESS as HRESULT {
            let mut cCategories_local = cCategories;
            let mut rgcatid_local = rgcatid;
            while cCategories_local != 0 {
                let mut key: HKEY = core::ptr::null_mut();

                res = StringFromGUID2(rgcatid_local, keyname.as_mut_ptr(), 39);
                if !FAILED(res) {
                    res = create_classes_key(type_key, keyname.as_ptr(), KEY_READ | KEY_WRITE, &mut key);
                    if res == ERROR_SUCCESS as HRESULT {
                        RegCloseKey(key);
                    }
                }

                cCategories_local = cCategories_local.wrapping_sub(1);
                rgcatid_local = rgcatid_local.offset(1);
            }
            res = 0x00000000u32 as HRESULT;
        } else {
            res = 0x80004005u32 as HRESULT;
        }
        RegCloseKey(class_key);
    } else {
        res = 0x80004005u32 as HRESULT;
    }
    RegCloseKey(clsid_key);

    res
}

unsafe fn COMCAT_UnRegisterClassCategories(
    rclsid: *const GUID,
    type_: LPCWSTR,
    cCategories: u32,
    rgcatid: *const GUID,
) -> HRESULT {

    let mut keyname: [u16; 68] = [0; 68];
    keyname[0] = 'C' as u16;
    keyname[1] = 'L' as u16;
    keyname[2] = 'S' as u16;
    keyname[3] = 'I' as u16;
    keyname[4] = 'D' as u16;
    keyname[5] = '\\' as u16;

    let mut res: HRESULT;
    let mut type_key: HKEY = core::ptr::null_mut();
    let mut categories = cCategories;
    let mut catid_ptr = rgcatid;

    if cCategories != 0 && rgcatid.is_null() {
        return 0x80004003u32 as HRESULT; // E_POINTER
    }

    res = StringFromGUID2(rclsid, keyname.as_mut_ptr().add(6), 39) as HRESULT;
    if FAILED(res) != 0 {
        return res;
    }
    keyname[44] = '\\' as u16;
    lstrcpyW(keyname.as_mut_ptr().add(45), type_);

    res = open_classes_key(
        0x80000000u32 as HKEY, // HKEY_CLASSES_ROOT
        keyname.as_ptr(),
        KEY_READ | KEY_WRITE,
        &mut type_key,
    ) as HRESULT;
    if res != ERROR_SUCCESS as HRESULT {
        return 0x80004005u32 as HRESULT; // E_FAIL
    }

    while categories > 0 {
        res = StringFromGUID2(catid_ptr, keyname.as_mut_ptr(), 39) as HRESULT;
        if FAILED(res) == 0 {
            RegDeleteKeyW(type_key, keyname.as_ptr());
        }
        categories -= 1;
        catid_ptr = catid_ptr.add(1);
    }
    RegCloseKey(type_key);

    0x00000000u32 as HRESULT // S_OK
}

unsafe fn COMCAT_GetCategoryDesc(
    key: HKEY,
    lcid: LCID,
    pszDesc: *mut u16,
    buf_wchars: u32,
) -> HRESULT {

    static FMT: [u16; 4] = [b'%' as u16, b'l' as u16, b'X' as u16, 0];
    let mut valname: [u16; 5] = [0; 5];
    let mut res: HRESULT;
    let mut type_: u32 = 0;
    let mut size: u32 = (buf_wchars - 1) * core::mem::size_of::<u16>() as u32;

    if pszDesc.is_null() {
        return 0x80070057u32 as HRESULT; // E_INVALIDARG
    }

    wsprintfW(valname.as_mut_ptr(), FMT.as_ptr(), lcid);
    res = RegQueryValueExW(
        key,
        valname.as_ptr(),
        core::ptr::null_mut(),
        &mut type_,
        pszDesc as LPBYTE,
        &mut size,
    ) as HRESULT;
    if res != ERROR_SUCCESS as HRESULT || type_ != REG_SZ {
        FIXME!(b"Simplified lcid comparison\n\0".as_ptr() as *const i8);
        return 0x80040301u32 as HRESULT; // CAT_E_NODESCRIPTION
    }
    *pszDesc.offset((size / core::mem::size_of::<u16>() as u32) as isize) = 0;

    0 // S_OK
}

unsafe fn COMCAT_PrepareClassCategories(
    impl_count: ULONG,
    impl_catids: *const GUID,
    req_count: ULONG,
    req_catids: *const GUID,
) -> *mut class_categories {
    let mut size: ULONG;

    size = core::mem::size_of::<class_categories>() as ULONG
        + ((impl_count + req_count) * CHARS_IN_GUID + 2) * core::mem::size_of::<WCHAR>() as ULONG;
    let categories = HeapAlloc(GetProcessHeap(), 0, size as usize) as *mut class_categories;
    if categories.is_null() {
        return categories;
    }

    (*categories).size = size;
    (*categories).impl_offset = core::mem::size_of::<class_categories>() as ULONG;
    (*categories).req_offset = (*categories).impl_offset
        + (impl_count * CHARS_IN_GUID + 1) * core::mem::size_of::<WCHAR>() as ULONG;

    let mut strings = categories.offset(1) as *mut WCHAR;
    let mut impl_catids_ptr = impl_catids;
    let mut impl_count_mut = impl_count;
    while impl_count_mut > 0 {
        StringFromGUID2(impl_catids_ptr, strings, CHARS_IN_GUID as i32);
        strings = strings.offset(CHARS_IN_GUID as isize);
        impl_catids_ptr = impl_catids_ptr.offset(1);
        impl_count_mut -= 1;
    }
    *strings = 0;
    strings = strings.offset(1);

    let mut req_catids_ptr = req_catids;
    let mut req_count_mut = req_count;
    while req_count_mut > 0 {
        StringFromGUID2(req_catids_ptr, strings, CHARS_IN_GUID as i32);
        strings = strings.offset(CHARS_IN_GUID as isize);
        req_catids_ptr = req_catids_ptr.offset(1);
        req_count_mut -= 1;
    }
    *strings = 0;

    categories
}

unsafe fn COMCAT_IsClassOfCategories(
    key: HKEY,
    categories: *const class_categories,
) -> HRESULT {
    let impl_strings: LPCWSTR;
    let req_strings: LPCWSTR;
    let mut subkey: HKEY = core::ptr::null_mut();
    let mut res: LSTATUS;
    let mut index: u32;
    let mut string: LPCWSTR;

    impl_strings = ((categories as *const u8).add((*categories).impl_offset as usize)) as LPCWSTR;
    req_strings = ((categories as *const u8).add((*categories).req_offset as usize)) as LPCWSTR;

    if *impl_strings != 0 {
        res = open_classes_key(key, impl_keyname.as_ptr(), KEY_READ, &mut subkey);
        if res != ERROR_SUCCESS {
            return 0x00000001; // S_FALSE
        }
        string = impl_strings;
        while *string != 0 {
            let mut catkey: HKEY = core::ptr::null_mut();
            res = open_classes_key(subkey, string, READ_CONTROL, &mut catkey);
            if res != ERROR_SUCCESS {
                RegCloseKey(subkey);
                return 0x00000001; // S_FALSE
            }
            RegCloseKey(catkey);
            string = string.add(39); // CHARS_IN_GUID
        }
        RegCloseKey(subkey);
    }

    res = open_classes_key(key, req_keyname.as_ptr(), KEY_READ, &mut subkey);
    if res == ERROR_SUCCESS {
        index = 0;
        loop {
            let mut keyname: [u16; 39] = [0; 39]; // CHARS_IN_GUID
            let mut size: u32 = 39; // CHARS_IN_GUID

            res = RegEnumKeyExW(
                subkey,
                index,
                keyname.as_mut_ptr(),
                &mut size,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );
            if res != ERROR_SUCCESS && res != ERROR_MORE_DATA {
                break;
            }
            if size != 38 {
                index += 1;
                continue;
            }
            string = req_strings;
            while *string != 0 {
                if wcsicmp(string, keyname.as_ptr()) == 0 {
                    break;
                }
                string = string.add(39); // CHARS_IN_GUID
            }
            if *string == 0 {
                RegCloseKey(subkey);
                return 0x00000001; // S_FALSE
            }
            index += 1;
        }
        RegCloseKey(subkey);
    }

    0x00000000 // S_OK
}

unsafe fn COMCAT_ICatRegister_QueryInterface(
    iface: LPCATREGISTER,
    riid: REFIID,
    ppvObj: *mut LPVOID,
) -> HRESULT {

    const E_POINTER: HRESULT = 0x80004003u32 as HRESULT;
    const E_NOINTERFACE: HRESULT = 0x80004002u32 as HRESULT;
    const S_OK: HRESULT = 0;

    TRACE!(b"%s\n\0".as_ptr(), debugstr_guid(riid));

    if ppvObj.is_null() {
        return E_POINTER;
    }

    if IsEqualGUID(riid, &IID_IUnknown) != 0 || IsEqualGUID(riid, &IID_ICatRegister) != 0 {
        *ppvObj = iface as LPVOID;
        ICatRegister_AddRef(iface);
        return S_OK;
    }

    if IsEqualGUID(riid, &IID_ICatInformation) != 0 {
        *ppvObj = core::ptr::addr_of_mut!(COMCAT_ComCatMgr.ICatInformation_iface) as LPVOID;
        ICatRegister_AddRef(iface);
        return S_OK;
    }

    E_NOINTERFACE
}

unsafe fn COMCAT_ICatRegister_AddRef(iface: LPCATREGISTER) -> ULONG {
    2
}

unsafe fn COMCAT_ICatRegister_Release(iface: LPCATREGISTER) -> u32 {
    return 1;
}
unsafe fn COMCAT_ICatRegister_RegisterCategories(
    iface: LPCATREGISTER,
    mut cCategories: u32,
    mut rgci: *mut CATEGORYINFO,
) -> HRESULT {

    TRACE!(b"\n\0".as_ptr());

    if cCategories != 0 && rgci.is_null() {
        return 0x80004003u32 as HRESULT;
    }

    let mut comcat_key: HKEY = core::ptr::null_mut();
    let res = create_classes_key(
        0x80000000u32 as HKEY,
        b"C\0o\0m\0p\0o\0n\0e\0n\0t\0 \0C\0a\0t\0e\0g\0o\0r\0i\0e\0s\0\0\0"
            .as_ptr() as LPCWSTR,
        KEY_READ | KEY_WRITE,
        &mut comcat_key,
    );
    if res != ERROR_SUCCESS {
        return 0x80004005u32 as HRESULT;
    }

    while cCategories != 0 {
        let mut keyname: [u16; 39] = [0; 39];
        let mut valname: [u16; 9] = [0; 9];
        let mut cat_key: HKEY = core::ptr::null_mut();

        if StringFromGUID2(
            &(*rgci).catid,
            keyname.as_mut_ptr(),
            39,
        ) == 0
        {
            cCategories = cCategories.wrapping_sub(1);
            rgci = rgci.offset(1);
            continue;
        }

        let res = create_classes_key(
            comcat_key,
            keyname.as_ptr(),
            KEY_READ | KEY_WRITE,
            &mut cat_key,
        );
        if res != ERROR_SUCCESS {
            cCategories = cCategories.wrapping_sub(1);
            rgci = rgci.offset(1);
            continue;
        }

        wsprintfW(
            valname.as_mut_ptr(),
            b"%\0l\0X\0\0\0".as_ptr() as LPCWSTR,
            (*rgci).lcid,
        );
        RegSetValueExW(
            cat_key,
            valname.as_ptr(),
            0,
            REG_SZ,
            (*rgci).szDescription.as_ptr() as *const u8,
            ((lstrlenW((*rgci).szDescription.as_ptr()) + 1) * 2) as u32,
        );

        RegCloseKey(cat_key);

        cCategories = cCategories.wrapping_sub(1);
        rgci = rgci.offset(1);
    }

    RegCloseKey(comcat_key);
    0i32 as HRESULT
}

unsafe fn COMCAT_ICatRegister_UnRegisterCategories(
    iface: LPCATREGISTER,
    mut cCategories: ULONG,
    mut rgcatid: *mut CATID,
) -> HRESULT {
    let mut comcat_key: HKEY = core::ptr::null_mut();
    let res: HRESULT;

    TRACE!(b"\n\0".as_ptr() as *const i8);

    if cCategories != 0 && rgcatid.is_null() {
        return E_POINTER;
    }

    res = open_classes_key(
        HKEY_CLASSES_ROOT,
        comcat_keyname.as_ptr(),
        KEY_READ | KEY_WRITE,
        &mut comcat_key,
    );
    if res != ERROR_SUCCESS {
        return E_FAIL;
    }

    while cCategories != 0 {
        let mut keyname: [u16; CHARS_IN_GUID] = [0; CHARS_IN_GUID];

        if StringFromGUID2(rgcatid, keyname.as_mut_ptr(), CHARS_IN_GUID as i32) != 0 {
            RegDeleteKeyW(comcat_key, keyname.as_ptr());
        }

        cCategories -= 1;
        rgcatid = rgcatid.add(1);
    }

    RegCloseKey(comcat_key);
    S_OK
}

unsafe fn COMCAT_ICatRegister_RegisterClassImplCategories(
    iface: *mut ICatRegister,
    rclsid: *const GUID,
    cCategories: u32,
    rgcatid: *mut GUID,
) -> HRESULT {
    COMCAT_RegisterClassCategories(
        rclsid,
        impl_keyname.as_ptr(),
        cCategories,
        rgcatid,
    )
}

unsafe fn COMCAT_ICatRegister_UnRegisterClassImplCategories(
    iface: *mut ICatRegister,
    rclsid: *const GUID,
    cCategories: u32,
    rgcatid: *mut GUID,
) -> HRESULT {
    COMCAT_UnRegisterClassCategories(
        rclsid,
        impl_keyname.as_ptr(),
        cCategories,
        rgcatid,
    )
}

unsafe fn COMCAT_ICatRegister_RegisterClassReqCategories(
    iface: *mut ICatRegister,
    rclsid: *const GUID,
    cCategories: u32,
    rgcatid: *mut GUID,
) -> HRESULT {
    COMCAT_RegisterClassCategories(
        rclsid,
        req_keyname.as_ptr(),
        cCategories,
        rgcatid,
    )
}

unsafe fn COMCAT_ICatRegister_UnRegisterClassReqCategories(
    iface: *mut ICatRegister,
    rclsid: *const GUID,
    cCategories: u32,
    rgcatid: *mut GUID,
) -> HRESULT {
    COMCAT_UnRegisterClassCategories(
        rclsid,
        req_keyname.as_ptr(),
        cCategories,
        rgcatid,
    )
}

unsafe fn COMCAT_ICatInformation_QueryInterface(
    iface: LPCATINFORMATION,
    riid: REFIID,
    ppvObj: *mut LPVOID,
) -> HRESULT {

    ICatRegister_QueryInterface(
        &mut COMCAT_ComCatMgr.ICatRegister_iface as *mut ICatRegister,
        riid,
        ppvObj,
    )
}

unsafe fn COMCAT_ICatInformation_AddRef(iface: LPCATINFORMATION) -> ULONG {
    ICatRegister_AddRef(core::ptr::addr_of_mut!(COMCAT_ComCatMgr.ICatRegister_iface))
}

unsafe fn COMCAT_ICatInformation_Release(iface: LPCATINFORMATION) -> ULONG {
    ICatRegister_Release(&mut COMCAT_ComCatMgr.ICatRegister_iface)
}

unsafe fn COMCAT_ICatInformation_EnumCategories(
    iface: LPCATINFORMATION,
    lcid: LCID,
    ppenumCatInfo: *mut *mut IEnumCATEGORYINFO,
) -> HRESULT {
    if ppenumCatInfo == core::ptr::null_mut() {
        return E_POINTER;
    }

    EnumCATEGORYINFO_Construct(lcid, ppenumCatInfo)
}

unsafe fn COMCAT_ICatInformation_GetCategoryDesc(
    iface: LPCATINFORMATION,
    rcatid: REFCATID,
    lcid: LCID,
    ppszDesc: *mut *mut u16,
) -> HRESULT {
    let mut keyname: [u16; 60] = [
        'C' as u16, 'o' as u16, 'm' as u16, 'p' as u16, 'o' as u16, 'n' as u16, 'e' as u16, 'n' as u16,
        't' as u16, ' ' as u16, 'C' as u16, 'a' as u16, 't' as u16, 'e' as u16, 'g' as u16, 'o' as u16,
        'r' as u16, 'i' as u16, 'e' as u16, 's' as u16, '\\' as u16, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
    ];
    let mut key: HKEY = core::ptr::null_mut();
    let mut res: HRESULT;

    TRACE!(b"CATID: %s LCID: %x\n\0".as_ptr() as *const i8, debugstr_guid(rcatid), lcid);

    if rcatid.is_null() || ppszDesc.is_null() {
        return E_INVALIDARG;
    }

    if StringFromGUID2(rcatid, keyname.as_mut_ptr().add(21), CHARS_IN_GUID) == 0 {
        return E_FAIL;
    }
    res = open_classes_key(HKEY_CLASSES_ROOT, keyname.as_ptr(), KEY_READ, &mut key);
    if res != ERROR_SUCCESS as HRESULT {
        return CAT_E_CATIDNOEXIST;
    }

    *ppszDesc = CoTaskMemAlloc(128 * core::mem::size_of::<u16>()) as *mut u16;
    if (*ppszDesc).is_null() {
        RegCloseKey(key);
        return E_OUTOFMEMORY;
    }

    res = COMCAT_GetCategoryDesc(key, lcid, *ppszDesc, 128);
    RegCloseKey(key);
    if FAILED(res) != 0 {
        CoTaskMemFree(*ppszDesc as LPVOID);
        return res;
    }

    S_OK
}
unsafe fn COMCAT_ICatInformation_EnumClassesOfCategories(
    iface: LPCATINFORMATION,
    mut cImplemented: ULONG,
    rgcatidImpl: *mut CATID,
    mut cRequired: ULONG,
    rgcatidReq: *mut CATID,
    ppenumCLSID: *mut LPENUMCLSID,
) -> HRESULT {
    let mut categories: *mut class_categories;
    let hr: HRESULT;

    TRACE!(b"\n\0".as_ptr() as *const i8);

    if cImplemented == ULONG::MAX {
        cImplemented = 0;
    }
    if cRequired == ULONG::MAX {
        cRequired = 0;
    }

    if ppenumCLSID.is_null()
        || (cImplemented != 0 && rgcatidImpl.is_null())
        || (cRequired != 0 && rgcatidReq.is_null())
    {
        return E_POINTER;
    }

    categories = COMCAT_PrepareClassCategories(
        cImplemented,
        rgcatidImpl,
        cRequired,
        rgcatidReq,
    );
    if categories.is_null() {
        return E_OUTOFMEMORY;
    }

    hr = CLSIDEnumGUID_Construct(categories, ppenumCLSID as *mut *mut IEnumGUID);
    if FAILED(hr) != 0 {
        HeapFree(GetProcessHeap(), 0, categories as LPVOID);
        return hr;
    }

    hr
}

unsafe fn COMCAT_ICatInformation_IsClassOfCategories(
    iface: LPCATINFORMATION,
    rclsid: REFCLSID,
    cImplemented: ULONG,
    rgcatidImpl: *mut CATID,
    cRequired: ULONG,
    rgcatidReq: *mut CATID,
) -> HRESULT {
    let mut keyname: [u16; 45] = [0; 45];
    keyname[0] = 'C' as u16;
    keyname[1] = 'L' as u16;
    keyname[2] = 'S' as u16;
    keyname[3] = 'I' as u16;
    keyname[4] = 'D' as u16;
    keyname[5] = '\\' as u16;

    let mut res: HRESULT;
    let categories: *mut class_categories;
    let mut key: HKEY = core::ptr::null_mut();

    if TRACE_ON(ole) != 0 {
        let mut count: ULONG;
        TRACE!(b"CLSID: %s Implemented %u\n\0".as_ptr() as *const i8, debugstr_guid(rclsid), cImplemented);
        count = 0;
        while count < cImplemented {
            TRACE!(b"    %s\n\0".as_ptr() as *const i8, debugstr_guid(&*rgcatidImpl.add(count as usize)));
            count += 1;
        }
        TRACE!(b"Required %u\n\0".as_ptr() as *const i8, cRequired);
        count = 0;
        while count < cRequired {
            TRACE!(b"    %s\n\0".as_ptr() as *const i8, debugstr_guid(&*rgcatidReq.add(count as usize)));
            count += 1;
        }
    }

    if (cImplemented != 0 && rgcatidImpl.is_null()) || (cRequired != 0 && rgcatidReq.is_null()) {
        return E_POINTER;
    }

    res = StringFromGUID2(rclsid, keyname.as_mut_ptr().add(6), CHARS_IN_GUID);
    if FAILED(res) != 0 {
        return res;
    }

    categories = COMCAT_PrepareClassCategories(cImplemented, rgcatidImpl, cRequired, rgcatidReq);
    if categories.is_null() {
        return E_OUTOFMEMORY;
    }

    res = open_classes_key(HKEY_CLASSES_ROOT, keyname.as_ptr(), KEY_READ, &mut key);
    if res == ERROR_SUCCESS {
        res = COMCAT_IsClassOfCategories(key, categories);
        RegCloseKey(key);
    } else {
        res = S_FALSE;
    }

    HeapFree(GetProcessHeap(), 0, categories as LPVOID);

    res
}

unsafe fn COMCAT_ICatInformation_EnumImplCategoriesOfClass(
    iface: *mut ICatInformation,
    rclsid: *const GUID,
    ppenumCATID: *mut *mut IEnumCATID
) -> HRESULT {
    static POSTFIX: [u16; 24] = [
        '\\' as u16, 'I' as u16, 'm' as u16, 'p' as u16, 'l' as u16, 'e' as u16, 'm' as u16, 'e' as u16,
        'n' as u16, 't' as u16, 'e' as u16, 'd' as u16, ' ' as u16, 'C' as u16, 'a' as u16, 't' as u16,
        'e' as u16, 'g' as u16, 'o' as u16, 'r' as u16, 'i' as u16, 'e' as u16, 's' as u16, 0
    ];

    TRACE!(b"%s\n\0".as_ptr(), debugstr_guid(rclsid));

    if rclsid.is_null() || ppenumCATID.is_null() {
        return E_POINTER;
    }

    CATIDEnumGUID_Construct(rclsid, POSTFIX.as_ptr(), ppenumCATID as *mut *mut IEnumGUID)
}

unsafe fn COMCAT_ICatInformation_EnumReqCategoriesOfClass(
    iface: *mut ICatInformation,
    rclsid: *const GUID,
    ppenumCATID: *mut *mut IEnumGUID,
) -> HRESULT {

    static POSTFIX: [u16; 21] = [
        '\\' as u16, 'R' as u16, 'e' as u16, 'q' as u16, 'u' as u16, 'i' as u16, 'r' as u16, 'e' as u16,
        'd' as u16, ' ' as u16, 'C' as u16, 'a' as u16, 't' as u16, 'e' as u16, 'g' as u16, 'o' as u16,
        'r' as u16, 'i' as u16, 'e' as u16, 's' as u16, 0
    ];

    if rclsid.is_null() || ppenumCATID.is_null() {
        return E_POINTER;
    }

    CATIDEnumGUID_Construct(rclsid, POSTFIX.as_ptr(), ppenumCATID)
}

#[no_mangle]
pub unsafe extern "system" fn ComCat_CreateInstance(
    iface: *mut IClassFactory,
    pUnkOuter: *mut IUnknown,
    riid: *const GUID,
    ppvObj: *mut LPVOID,
) -> HRESULT {

    TRACE!(b"%s\n\0".as_ptr(), debugstr_guid(riid));

    if ppvObj.is_null() {
        return 0x80004003u32 as HRESULT;
    }

    if !pUnkOuter.is_null() {
        return 0x80040110u32 as HRESULT;
    }

    let res = ICatRegister_QueryInterface(
        core::ptr::addr_of_mut!(COMCAT_ComCatMgr.ICatRegister_iface),
        riid,
        ppvObj,
    );
    if SUCCEEDED(res) != 0 {
        return res;
    }

    0x80040111u32 as HRESULT
}

unsafe fn impl_from_IEnumCATEGORYINFO(iface: *mut IEnumCATEGORYINFO) -> *mut IEnumCATEGORYINFOImpl {
    (iface as *mut u8).offset(-(core::mem::offset_of!(IEnumCATEGORYINFOImpl, IEnumCATEGORYINFO_iface) as isize)) as *mut IEnumCATEGORYINFOImpl
}

unsafe fn COMCAT_IEnumCATEGORYINFO_AddRef(iface: *mut IEnumCATEGORYINFO) -> ULONG {
    let This = impl_from_IEnumCATEGORYINFO(iface);

    TRACE!(b"\n\0".as_ptr() as LPCWSTR);

    InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_)) as ULONG
}

unsafe fn COMCAT_IEnumCATEGORYINFO_QueryInterface(
    iface: *mut IEnumCATEGORYINFO,
    riid: *const GUID,
    ppvObj: *mut LPVOID,
) -> HRESULT {

    TRACE!(b"%s\n\0".as_ptr(), debugstr_guid(riid));

    if ppvObj.is_null() {
        return 0x80004003u32 as HRESULT;
    }

    if IsEqualGUID(riid, core::ptr::addr_of!(IID_IUnknown)) != 0
        || IsEqualGUID(riid, core::ptr::addr_of!(IID_IEnumCATEGORYINFO)) != 0
    {
        *ppvObj = iface as LPVOID;
        COMCAT_IEnumCATEGORYINFO_AddRef(iface);
        return 0u32 as HRESULT;
    }

    return 0x80004002u32 as HRESULT;
}

unsafe fn COMCAT_IEnumCATEGORYINFO_Release(iface: *mut IEnumCATEGORYINFO) -> ULONG {

    let This = impl_from_IEnumCATEGORYINFO(iface);
    let ref_count = InterlockedDecrement(&mut (*This).ref_);

    if ref_count == 0 {
        if (*This).key != core::ptr::null_mut() {
            RegCloseKey((*This).key);
        }
        HeapFree(GetProcessHeap(), 0, This as LPVOID);
        return 0;
    }
    ref_count as ULONG
}

unsafe fn COMCAT_IEnumCATEGORYINFO_Next(
    iface: *mut IEnumCATEGORYINFO,
    celt: u32,
    rgelt: *mut CATEGORYINFO,
    pceltFetched: *mut u32,
) -> HRESULT {

    let This = impl_from_IEnumCATEGORYINFO(iface);
    let mut fetched: u32 = 0;

    TRACE!(b"\n\0".as_ptr() as *const i8);

    if rgelt.is_null() {
        return 0x80004003u32 as HRESULT;
    }

    if !(*This).key.is_null() {
        let mut rgelt_ptr = rgelt;
        while fetched < celt {
            let mut catid: [u16; 39] = [0; 39];
            let mut cName: u32 = 39;
            let mut subkey: HKEY = core::ptr::null_mut();

            let res = RegEnumKeyExW(
                (*This).key,
                (*This).next_index,
                catid.as_mut_ptr(),
                core::ptr::addr_of_mut!(cName),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );
            if res != ERROR_SUCCESS && res != ERROR_MORE_DATA {
                break;
            }
            (*This).next_index += 1;

            let hr = CLSIDFromString(
                catid.as_ptr(),
                core::ptr::addr_of_mut!((*rgelt_ptr).catid),
            );
            if FAILED(hr) != 0 {
                continue;
            }

            let res = open_classes_key((*This).key, catid.as_ptr(), KEY_READ, core::ptr::addr_of_mut!(subkey));
            if res != ERROR_SUCCESS {
                continue;
            }

            let hr = COMCAT_GetCategoryDesc(
                subkey,
                (*This).lcid,
                (*rgelt_ptr).szDescription.as_mut_ptr(),
                128,
            );
            RegCloseKey(subkey);
            if FAILED(hr) != 0 {
                continue;
            }

            (*rgelt_ptr).lcid = (*This).lcid;
            fetched += 1;
            rgelt_ptr = rgelt_ptr.add(1);
        }
    }

    if !pceltFetched.is_null() {
        *pceltFetched = fetched;
    }
    if fetched == celt { 0 } else { 1 }
}
unsafe fn COMCAT_IEnumCATEGORYINFO_Skip(
    iface: *mut IEnumCATEGORYINFO,
    celt: ULONG,
) -> HRESULT {
    let This = impl_from_IEnumCATEGORYINFO(iface);

    TRACE!(b"\n\0".as_ptr() as *const i8);

    (*This).next_index = (*This).next_index.wrapping_add(celt);
    S_OK
}
unsafe fn COMCAT_IEnumCATEGORYINFO_Reset(iface: *mut IEnumCATEGORYINFO) -> HRESULT {
    let This = impl_from_IEnumCATEGORYINFO(iface);

    TRACE!(b"\n\0".as_ptr() as LPCWSTR);

    (*This).next_index = 0;
    S_OK
}
unsafe fn COMCAT_IEnumCATEGORYINFO_Clone(
    iface: *mut IEnumCATEGORYINFO,
    ppenum: *mut *mut IEnumCATEGORYINFO,
) -> HRESULT {

    let This = impl_from_IEnumCATEGORYINFO(iface);
    static keyname: [u16; 21] = [
        'C' as u16, 'o' as u16, 'm' as u16, 'p' as u16, 'o' as u16, 'n' as u16, 'e' as u16, 'n' as u16,
        't' as u16, ' ' as u16, 'C' as u16, 'a' as u16, 't' as u16, 'e' as u16, 'g' as u16, 'o' as u16,
        'r' as u16, 'i' as u16, 'e' as u16, 's' as u16, 0,
    ];

    if ppenum.is_null() {
        return E_POINTER;
    }

    let new_this = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        core::mem::size_of::<IEnumCATEGORYINFOImpl>(),
    ) as *mut IEnumCATEGORYINFOImpl;

    if new_this.is_null() {
        return E_OUTOFMEMORY;
    }

    (*new_this).IEnumCATEGORYINFO_iface = (*This).IEnumCATEGORYINFO_iface;
    (*new_this).ref_ = 1;
    (*new_this).lcid = (*This).lcid;
    open_classes_key(
        HKEY_CLASSES_ROOT,
        keyname.as_ptr(),
        KEY_READ,
        core::ptr::addr_of_mut!((*new_this).key),
    );
    (*new_this).next_index = (*This).next_index;

    *ppenum = core::ptr::addr_of_mut!((*new_this).IEnumCATEGORYINFO_iface);
    S_OK
}

unsafe fn EnumCATEGORYINFO_Construct(lcid: LCID, ret: *mut *mut IEnumCATEGORYINFO) -> HRESULT {
    static KEYNAME: &[u16] = &[
        'C' as u16, 'o' as u16, 'm' as u16, 'p' as u16, 'o' as u16, 'n' as u16, 'e' as u16, 'n' as u16, 't' as u16, ' ' as u16,
        'C' as u16, 'a' as u16, 't' as u16, 'e' as u16, 'g' as u16, 'o' as u16, 'r' as u16, 'i' as u16, 'e' as u16, 's' as u16, 0
    ];


    *ret = core::ptr::null_mut();

    let This = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<IEnumCATEGORYINFOImpl>()) as *mut IEnumCATEGORYINFOImpl;
    if This.is_null() {
        return E_OUTOFMEMORY;
    }

    (*This).IEnumCATEGORYINFO_iface.lpVtbl = &COMCAT_IEnumCATEGORYINFO_Vtbl;
    (*This).ref_ = 1;
    (*This).lcid = lcid;
    open_classes_key(HKEY_CLASSES_ROOT, KEYNAME.as_ptr(), KEY_READ, core::ptr::addr_of_mut!((*This).key));

    *ret = core::ptr::addr_of_mut!((*This).IEnumCATEGORYINFO_iface);
    S_OK
}

unsafe fn impl_from_IEnumCLSID(iface: *mut IEnumGUID) -> *mut CLSID_IEnumGUIDImpl {
    (iface as *mut u8).offset(-(core::mem::offset_of!(CLSID_IEnumGUIDImpl, IEnumGUID_iface) as isize)) as *mut CLSID_IEnumGUIDImpl
}
unsafe fn CLSIDEnumGUID_QueryInterface(
    iface: *mut IEnumGUID,
    riid: *const GUID,
    ppvObj: *mut LPVOID,
) -> HRESULT {

    TRACE!(b"%s\n\0".as_ptr(), debugstr_guid(riid));

    if ppvObj.is_null() {
        return E_POINTER;
    }

    if IsEqualGUID(riid, &IID_IUnknown) != 0 ||
       IsEqualGUID(riid, &IID_IEnumGUID) != 0
    {
        *ppvObj = iface as LPVOID;
        IEnumGUID_AddRef(iface);
        return S_OK;
    }

    E_NOINTERFACE
}

unsafe fn CLSIDEnumGUID_AddRef(iface: *mut IEnumGUID) -> ULONG {
    let This = impl_from_IEnumCLSID(iface);
    TRACE!(b"\n\0".as_ptr() as *const i8);

    InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_)) as ULONG
}

unsafe fn CLSIDEnumGUID_Release(iface: *mut IEnumGUID) -> u32 {
    let This = impl_from_IEnumCLSID(iface);
    let ref_count: u32;

    TRACE!(b"\n\0".as_ptr() as *const i8);

    ref_count = InterlockedDecrement(&mut (*This).ref_) as u32;
    if ref_count == 0 {
        if !(*This).key.is_null() {
            RegCloseKey((*This).key);
        }
        HeapFree(GetProcessHeap(), 0, (*This).categories as LPVOID);
        HeapFree(GetProcessHeap(), 0, This as LPVOID);
        return 0;
    }
    return ref_count;
}

unsafe fn CLSIDEnumGUID_Next(
    iface: *mut IEnumGUID,
    celt: ULONG,
    rgelt: *mut GUID,
    pceltFetched: *mut ULONG,
) -> HRESULT {
    let This = impl_from_IEnumCLSID(iface);
    let mut fetched: ULONG = 0;

    TRACE!(b"\0".as_ptr());

    if rgelt.is_null() {
        return E_POINTER;
    }

    if !(*This).key.is_null() {
        while fetched < celt {
            let mut clsid: [u16; 39] = [0; 39];
            let mut cName: DWORD = 39;
            let mut subkey: HKEY = core::ptr::null_mut();

            let res = RegEnumKeyExW(
                (*This).key,
                (*This).next_index,
                clsid.as_mut_ptr(),
                core::ptr::addr_of_mut!(cName),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );
            if res != ERROR_SUCCESS && res != ERROR_MORE_DATA {
                break;
            }
            (*This).next_index += 1;

            let hr = CLSIDFromString(clsid.as_ptr(), rgelt.add(fetched as usize));
            if FAILED(hr) {
                continue;
            }

            let res = open_classes_key((*This).key, clsid.as_ptr(), KEY_READ, core::ptr::addr_of_mut!(subkey));
            if res != ERROR_SUCCESS {
                continue;
            }

            let hr = COMCAT_IsClassOfCategories(subkey, (*This).categories);
            RegCloseKey(subkey);
            if hr != S_OK {
                continue;
            }

            fetched += 1;
        }
    }

    if !pceltFetched.is_null() {
        *pceltFetched = fetched;
    }

    if fetched == celt { S_OK } else { S_FALSE }
}

unsafe fn CLSIDEnumGUID_Skip(
    iface: *mut IEnumGUID,
    celt: u32,
) -> HRESULT {
    let This = impl_from_IEnumCLSID(iface);

    TRACE!(b"\n\0".as_ptr() as *const i8);

    (*This).next_index = (*This).next_index.wrapping_add(celt);
    FIXME!(b"Never returns S_FALSE\n\0".as_ptr() as *const i8);
    S_OK
}

unsafe fn CLSIDEnumGUID_Reset(iface: *mut IEnumGUID) -> HRESULT {
    let This = impl_from_IEnumCLSID(iface);

    TRACE!(b"\n\0".as_ptr() as PCHAR);

    (*This).next_index = 0;
    S_OK
}

unsafe fn CLSIDEnumGUID_Clone(
    iface: *mut IEnumGUID,
    ppenum: *mut *mut IEnumGUID
) -> HRESULT {
    static keynameW: [u16; 6] = [b'C' as u16, b'L' as u16, b'S' as u16, b'I' as u16, b'D' as u16, 0];
    let This = impl_from_IEnumCLSID(iface);

    TRACE!(b"(%p)->(%p)\n\0".as_ptr() as *const i8, This, ppenum);

    if ppenum == core::ptr::null_mut() {
        return E_POINTER;
    }

    *ppenum = core::ptr::null_mut();

    let cloned = HeapAlloc(GetProcessHeap(), 0, core::mem::size_of::<CLSID_IEnumGUIDImpl>()) as *mut CLSID_IEnumGUIDImpl;
    if cloned == core::ptr::null_mut() {
        return E_OUTOFMEMORY;
    }

    (*cloned).IEnumGUID_iface.lpVtbl = (*This).IEnumGUID_iface.lpVtbl;
    (*cloned).ref_ = 1;

    (*cloned).categories = HeapAlloc(GetProcessHeap(), 0, (*(*This).categories).size as usize) as *mut class_categories;
    if (*cloned).categories == core::ptr::null_mut() {
        HeapFree(GetProcessHeap(), 0, cloned as LPVOID);
        return E_OUTOFMEMORY;
    }
    core::ptr::copy_nonoverlapping(
        (*This).categories as *const u8,
        (*cloned).categories as *mut u8,
        (*(*This).categories).size as usize
    );

    (*cloned).key = core::ptr::null_mut();
    open_classes_key(HKEY_CLASSES_ROOT, keynameW.as_ptr(), KEY_READ, core::ptr::addr_of_mut!((*cloned).key));
    (*cloned).next_index = (*This).next_index;

    *ppenum = core::ptr::addr_of_mut!((*cloned).IEnumGUID_iface);
    S_OK
}

unsafe fn CLSIDEnumGUID_Construct(categories: *mut class_categories, ret: *mut *mut IEnumGUID) -> HRESULT {

    *ret = core::ptr::null_mut();

    let This = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<CLSID_IEnumGUIDImpl>()) as *mut CLSID_IEnumGUIDImpl;
    if This.is_null() {
        return E_OUTOFMEMORY;
    }

    (*This).IEnumGUID_iface.lpVtbl = &CLSIDEnumGUIDVtbl;
    (*This).ref_ = 1;
    (*This).categories = categories;
    open_classes_key(HKEY_CLASSES_ROOT, clsid_keyname.as_ptr(), KEY_READ, core::ptr::addr_of_mut!((*This).key));

    *ret = core::ptr::addr_of_mut!((*This).IEnumGUID_iface);

    S_OK
}

unsafe fn impl_from_IEnumCATID(iface: *mut IEnumGUID) -> *mut CATID_IEnumGUIDImpl {
    (iface as *mut u8).offset(-(core::mem::offset_of!(CATID_IEnumGUIDImpl, IEnumGUID_iface) as isize)) as *mut CATID_IEnumGUIDImpl
}

unsafe fn CATIDEnumGUID_QueryInterface(
    iface: *mut IEnumGUID,
    riid: *const GUID,
    ppvObj: *mut LPVOID,
) -> HRESULT {

    TRACE!(b"%s\n\0".as_ptr(), debugstr_guid(riid));

    if ppvObj.is_null() {
        return E_POINTER;
    }

    if IsEqualGUID(riid, &IID_IUnknown) != 0 || IsEqualGUID(riid, &IID_IEnumGUID) != 0 {
        *ppvObj = iface as LPVOID;
        IEnumGUID_AddRef(iface);
        return S_OK;
    }

    E_NOINTERFACE
}

unsafe fn CATIDEnumGUID_AddRef(iface: *mut IEnumGUID) -> u32 {
    let This = impl_from_IEnumCATID(iface);
    InterlockedIncrement(&mut (*This).ref_) as u32
}
unsafe fn CATIDEnumGUID_Release(iface: *mut IEnumGUID) -> u32 {
    let This = impl_from_IEnumCATID(iface);
    let ref_count: u32;

    TRACE!(b"\n\0".as_ptr() as *const i8);

    ref_count = InterlockedDecrement(&mut (*This).ref_) as u32;
    if ref_count == 0 {
        if !(*This).key.is_null() {
            RegCloseKey((*This).key);
        }
        HeapFree(GetProcessHeap(), 0, This as LPVOID);
        return 0;
    }
    return ref_count;
}

unsafe fn CATIDEnumGUID_Next(
    iface: *mut IEnumGUID,
    celt: u32,
    rgelt: *mut GUID,
    pceltFetched: *mut u32,
) -> HRESULT {
    let This = impl_from_IEnumCATID(iface);
    let mut fetched: u32 = 0;

    TRACE!(b"\n\0".as_ptr() as *const i8);

    if rgelt.is_null() {
        return E_POINTER;
    }

    if !(*This).key.is_null() {
        while fetched < celt {
            let mut res: LSTATUS;
            let hr: HRESULT;
            let mut catid: [u16; 39] = [0; 39];
            let mut cName: u32 = 39;

            res = RegEnumKeyExW(
                (*This).key,
                (*This).next_index,
                catid.as_mut_ptr(),
                &mut cName,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );
            if res != ERROR_SUCCESS && res != ERROR_MORE_DATA {
                break;
            }
            (*This).next_index += 1;

            hr = CLSIDFromString(catid.as_ptr(), rgelt.add(fetched as usize));
            if FAILED(hr) != 0 {
                continue;
            }

            fetched += 1;
        }
    }

    if !pceltFetched.is_null() {
        *pceltFetched = fetched;
    }

    if fetched == celt {
        S_OK
    } else {
        S_FALSE
    }
}

unsafe fn CATIDEnumGUID_Skip(
    iface: *mut IEnumGUID,
    celt: ULONG,
) -> HRESULT {
    let This = impl_from_IEnumCATID(iface);

    TRACE!(b"\n\0".as_ptr() as LPCSTR);

    (*This).next_index = (*This).next_index.wrapping_add(celt);
    FIXME!(b"Never returns S_FALSE\n\0".as_ptr() as LPCSTR);
    S_OK
}

unsafe fn CATIDEnumGUID_Reset(iface: *mut IEnumGUID) -> HRESULT {
    let This = impl_from_IEnumCATID(iface);
    TRACE!(b"\n\0".as_ptr() as *const i8);
    (*This).next_index = 0;
    S_OK
}

unsafe fn CATIDEnumGUID_Clone(
    iface: *mut IEnumGUID,
    ppenum: *mut *mut IEnumGUID
) -> HRESULT {
    let This = impl_from_IEnumCATID(iface);
    let new_this: *mut CATID_IEnumGUIDImpl;

    TRACE!(b"\n\0".as_ptr());

    if ppenum == core::ptr::null_mut() {
        return 0x80004003u32 as HRESULT;
    }

    new_this = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<CATID_IEnumGUIDImpl>()) as *mut CATID_IEnumGUIDImpl;
    if new_this == core::ptr::null_mut() {
        return 0x8007000Eu32 as HRESULT;
    }

    (*new_this).IEnumGUID_iface.lpVtbl = (*This).IEnumGUID_iface.lpVtbl;
    (*new_this).ref_ = 1;
    lstrcpyW((*new_this).keyname.as_mut_ptr(), (*This).keyname.as_ptr());
    open_classes_key(0x80000000u32 as HKEY, (*new_this).keyname.as_ptr(), KEY_READ, core::ptr::addr_of_mut!((*new_this).key));
    (*new_this).next_index = (*This).next_index;

    *ppenum = core::ptr::addr_of_mut!((*new_this).IEnumGUID_iface);
    0
}

unsafe fn CATIDEnumGUID_Construct(rclsid: *const GUID, postfix: LPCWSTR, ret: *mut *mut IEnumGUID) -> HRESULT {

    const CHARS_IN_GUID: i32 = 39;
    static PREFIX_W: [u16; 7] = [b'C' as u16, b'L' as u16, b'S' as u16, b'I' as u16, b'D' as u16, b'\\' as u16, 0];

    let mut keyname: [u16; 100] = [0; 100];
    let mut clsidW: [u16; CHARS_IN_GUID as usize] = [0; CHARS_IN_GUID as usize];

    *ret = core::ptr::null_mut();

    let This = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        core::mem::size_of::<CATID_IEnumGUIDImpl>()
    ) as *mut CATID_IEnumGUIDImpl;
    if This.is_null() {
        return 0x8007000E; // E_OUTOFMEMORY
    }

    StringFromGUID2(rclsid, clsidW.as_mut_ptr(), CHARS_IN_GUID);

    (*This).IEnumGUID_iface.lpVtbl = &CATIDEnumGUIDVtbl;
    (*This).ref_ = 1;
    lstrcpyW(keyname.as_mut_ptr(), PREFIX_W.as_ptr());
    lstrcatW(keyname.as_mut_ptr(), clsidW.as_ptr());
    lstrcatW(keyname.as_mut_ptr(), postfix);

    open_classes_key(
        3 as HKEY, // HKEY_CLASSES_ROOT
        keyname.as_ptr(),
        KEY_READ,
        core::ptr::addr_of_mut!((*This).key)
    );

    *ret = core::ptr::addr_of_mut!((*This).IEnumGUID_iface);
    0 // S_OK
}
