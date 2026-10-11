// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_3DBECEBCF87C: [u16; 3] = [0x005Cu16, 0x005C, 0x0000];
pub static RUSTCONV_WSTR_595FA2D85B6D: [u16; 7] = [0x0043u16, 0x004F, 0x004E, 0x0046, 0x0049, 0x0047, 0x0000];
pub static RUSTCONV_WSTR_B7FC02907B1F: [u16; 2] = [0x005Cu16, 0x0000];
pub static RUSTCONV_WSTR_BF8BAB706FCE: [u16; 10] = [0x0061u16, 0x0063, 0x006C, 0x0075, 0x0069, 0x002E, 0x0064, 0x006C, 0x006C, 0x0000];
pub static RUSTCONV_WSTR_D1EB02738F2B: [u16; 13] = [
    0x0043u16, 0x004C, 0x0041, 0x0053, 0x0053, 0x0045, 0x0053, 0x005F, 0x0052, 0x004F,
    0x004F, 0x0054, 0x0000,
];
pub static RUSTCONV_WSTR_D76AA73CA09D: [u16; 13] = [
    0x0043u16, 0x0055, 0x0052, 0x0052, 0x0045, 0x004E, 0x0054, 0x005F, 0x0055, 0x0053,
    0x0045, 0x0052, 0x0000,
];
pub static RUSTCONV_WSTR_D89386175ED2: [u16; 8] = [0x004Du16, 0x0041, 0x0043, 0x0048, 0x0049, 0x004E, 0x0045, 0x0000];
pub static RUSTCONV_WSTR_FD9080F09EC0: [u16; 6] = [0x0055u16, 0x0053, 0x0045, 0x0052, 0x0053, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut IID_IRegKeySecurity: GUID = _GUID {
    Data1: 0x965fc360 as ::core::ffi::c_uint,
    Data2: 0x16ff as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0x91 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xcb as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xaa as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xbb as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x23 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut IID_IEffectivePermission: GUID = _GUID {
    Data1: 0x3853dc76 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x9f35 as ::core::ffi::c_ushort,
    Data3: 0x407c as ::core::ffi::c_ushort,
    Data4: [
        0x88 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x93 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x44 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x36 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xbc as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut IID_ISecurityObjectTypeInfo: GUID = _GUID {
    Data1: 0xfc3066eb as ::core::ffi::c_uint,
    Data2: 0x79ef as ::core::ffi::c_ushort,
    Data3: 0x444b as ::core::ffi::c_ushort,
    Data4: [
        0x91 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x11 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x75 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xeb as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xfa as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

static mut vtblISecurityInformation: ifaceISecurityInformationVbtl = unsafe {
    ifaceISecurityInformationVbtl {
        QueryInterface: Some(
            ISecurityInformation_fnQueryInterface,
        ),
        AddRef: Some(
            ISecurityInformation_fnAddRef,
        ),
        Release: Some(
            ISecurityInformation_fnRelease,
        ),
        GetObjectInformation: Some(
            ISecurityInformation_fnGetObjectInformation,
        ),
        GetSecurity: Some(
            ISecurityInformation_fnGetSecurity,
        ),
        SetSecurity: Some(
            ISecurityInformation_fnSetSecurity,
        ),
        GetAccessRights: Some(
            ISecurityInformation_fnGetAccessRights,
        ),
        MapGeneric: Some(
            ISecurityInformation_fnMapGeneric,
        ),
        GetInheritTypes: Some(
            ISecurityInformation_fnGetInheritTypes,
        ),
        PropertySheetPageCallback: Some(
            ISecurityInformation_fnPropertySheetPageCallback,
        ),
    }
};

static mut vtblIEffectivePermission: ifaceIEffectivePermissionVbtl = unsafe {
    ifaceIEffectivePermissionVbtl {
        QueryInterface: Some(
            IEffectivePermission_fnQueryInterface,
        ),
        AddRef: Some(
            IEffectivePermission_fnAddRef,
        ),
        Release: Some(
            IEffectivePermission_fnRelease,
        ),
        GetEffectivePermission: Some(
            IEffectivePermission_fnGetEffectivePermission,
        ),
    }
};

static mut vtblISecurityObjectTypeInfo: ifaceISecurityObjectTypeInfoVbtl = unsafe {
    ifaceISecurityObjectTypeInfoVbtl {
        QueryInterface: Some(
            ISecurityObjectTypeInfo_fnQueryInterface,
        ),
        AddRef: Some(
            ISecurityObjectTypeInfo_fnAddRef,
        ),
        Release: Some(
            ISecurityObjectTypeInfo_fnRelease,
        ),
        GetInheritSource: Some(
            ISecurityObjectTypeInfo_fnGetInheritSource,
        ),
    }
};

static mut RegAccess: [SI_ACCESS; 12] = unsafe {
    [
        _SI_ACCESS {
            pguid: &raw const GUID_NULL,
            mask: KEY_ALL_ACCESS as ACCESS_MASK,
            pszName: (31100 as ::core::ffi::c_int as DWORD_PTR & 0xffff as DWORD_PTR) as WORD
                as ULONG_PTR as LPWSTR as LPCWSTR,
            dwFlags: (SI_ACCESS_GENERAL | SI_ACCESS_SPECIFIC) as DWORD,
        },
        _SI_ACCESS {
            pguid: &raw const GUID_NULL,
            mask: KEY_READ as ACCESS_MASK,
            pszName: (31101 as ::core::ffi::c_int as DWORD_PTR & 0xffff as DWORD_PTR) as WORD
                as ULONG_PTR as LPWSTR as LPCWSTR,
            dwFlags: SI_ACCESS_GENERAL as DWORD,
        },
        _SI_ACCESS {
            pguid: &raw const GUID_NULL,
            mask: KEY_QUERY_VALUE as ACCESS_MASK,
            pszName: (31102 as ::core::ffi::c_int as DWORD_PTR & 0xffff as DWORD_PTR) as WORD
                as ULONG_PTR as LPWSTR as LPCWSTR,
            dwFlags: SI_ACCESS_SPECIFIC as DWORD,
        },
        _SI_ACCESS {
            pguid: &raw const GUID_NULL,
            mask: KEY_SET_VALUE as ACCESS_MASK,
            pszName: (31103 as ::core::ffi::c_int as DWORD_PTR & 0xffff as DWORD_PTR) as WORD
                as ULONG_PTR as LPWSTR as LPCWSTR,
            dwFlags: SI_ACCESS_SPECIFIC as DWORD,
        },
        _SI_ACCESS {
            pguid: &raw const GUID_NULL,
            mask: KEY_CREATE_SUB_KEY as ACCESS_MASK,
            pszName: (31104 as ::core::ffi::c_int as DWORD_PTR & 0xffff as DWORD_PTR) as WORD
                as ULONG_PTR as LPWSTR as LPCWSTR,
            dwFlags: SI_ACCESS_SPECIFIC as DWORD,
        },
        _SI_ACCESS {
            pguid: &raw const GUID_NULL,
            mask: KEY_ENUMERATE_SUB_KEYS as ACCESS_MASK,
            pszName: (31105 as ::core::ffi::c_int as DWORD_PTR & 0xffff as DWORD_PTR) as WORD
                as ULONG_PTR as LPWSTR as LPCWSTR,
            dwFlags: SI_ACCESS_SPECIFIC as DWORD,
        },
        _SI_ACCESS {
            pguid: &raw const GUID_NULL,
            mask: KEY_NOTIFY as ACCESS_MASK,
            pszName: (31106 as ::core::ffi::c_int as DWORD_PTR & 0xffff as DWORD_PTR) as WORD
                as ULONG_PTR as LPWSTR as LPCWSTR,
            dwFlags: SI_ACCESS_SPECIFIC as DWORD,
        },
        _SI_ACCESS {
            pguid: &raw const GUID_NULL,
            mask: KEY_CREATE_LINK as ACCESS_MASK,
            pszName: (31107 as ::core::ffi::c_int as DWORD_PTR & 0xffff as DWORD_PTR) as WORD
                as ULONG_PTR as LPWSTR as LPCWSTR,
            dwFlags: SI_ACCESS_SPECIFIC as DWORD,
        },
        _SI_ACCESS {
            pguid: &raw const GUID_NULL,
            mask: DELETE as ACCESS_MASK,
            pszName: (31108 as ::core::ffi::c_int as DWORD_PTR & 0xffff as DWORD_PTR) as WORD
                as ULONG_PTR as LPWSTR as LPCWSTR,
            dwFlags: SI_ACCESS_SPECIFIC as DWORD,
        },
        _SI_ACCESS {
            pguid: &raw const GUID_NULL,
            mask: WRITE_DAC as ACCESS_MASK,
            pszName: (31109 as ::core::ffi::c_int as DWORD_PTR & 0xffff as DWORD_PTR) as WORD
                as ULONG_PTR as LPWSTR as LPCWSTR,
            dwFlags: SI_ACCESS_SPECIFIC as DWORD,
        },
        _SI_ACCESS {
            pguid: &raw const GUID_NULL,
            mask: WRITE_OWNER as ACCESS_MASK,
            pszName: (31110 as ::core::ffi::c_int as DWORD_PTR & 0xffff as DWORD_PTR) as WORD
                as ULONG_PTR as LPWSTR as LPCWSTR,
            dwFlags: SI_ACCESS_SPECIFIC as DWORD,
        },
        _SI_ACCESS {
            pguid: &raw const GUID_NULL,
            mask: READ_CONTROL as ACCESS_MASK,
            pszName: (31111 as ::core::ffi::c_int as DWORD_PTR & 0xffff as DWORD_PTR) as WORD
                as ULONG_PTR as LPWSTR as LPCWSTR,
            dwFlags: SI_ACCESS_SPECIFIC as DWORD,
        },
    ]
};

static mut RegDefaultAccess: DWORD = 1 as DWORD;

static mut RegAccessMasks: _GENERIC_MAPPING = _GENERIC_MAPPING {
    GenericRead: KEY_READ as ACCESS_MASK,
    GenericWrite: KEY_WRITE as ACCESS_MASK,
    GenericExecute: KEY_EXECUTE as ACCESS_MASK,
    GenericAll: KEY_ALL_ACCESS as ACCESS_MASK,
};

static mut RegInheritTypes: [SI_INHERIT_TYPE; 3] = unsafe {
    [
        _SI_INHERIT_TYPE {
            pguid: &raw const GUID_NULL,
            dwFlags: 0 as ULONG,
            pszName: (31200 as ::core::ffi::c_int as DWORD_PTR & 0xffff as DWORD_PTR) as WORD
                as ULONG_PTR as LPWSTR as LPCWSTR,
        },
        _SI_INHERIT_TYPE {
            pguid: &raw const GUID_NULL,
            dwFlags: CONTAINER_INHERIT_ACE as ULONG,
            pszName: (31201 as ::core::ffi::c_int as DWORD_PTR & 0xffff as DWORD_PTR) as WORD
                as ULONG_PTR as LPWSTR as LPCWSTR,
        },
        _SI_INHERIT_TYPE {
            pguid: &raw const GUID_NULL,
            dwFlags: (INHERIT_ONLY_ACE | CONTAINER_INHERIT_ACE) as ULONG,
            pszName: (31202 as ::core::ffi::c_int as DWORD_PTR & 0xffff as DWORD_PTR) as WORD
                as ULONG_PTR as LPWSTR as LPCWSTR,
        },
    ]
};

static mut pfnEditSecurity: PEDITSECURITY = None;

static mut hAclUiDll: HMODULE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

pub const IID_IRegKeySecurity2: GUID = GUID {
    Data1: 0xc3ccfdb4,
    Data2: 0x6f88,
    Data3: 0x11d2,
    Data4: [0xa3, 0xce, 0x00, 0xc0, 0x4f, 0xb1, 0x78, 0x2a],
};

// GUID_NULL (extern const GUID in the Windows SDK; all-zero value)
pub static GUID_NULL: GUID = GUID {
    Data1: 0,
    Data2: 0,
    Data3: 0,
    Data4: [0; 8],
};

// SI_ACCESS_* flags from aclui.h

// A C function pointer is nullable; LoadAclUiDll tests it against NULL.
pub type PEDITSECURITY = Option<
    unsafe extern "system" fn(hwndOwner: HWND, psi: *mut ISecurityInformation) -> BOOL,
>;

#[repr(C)]
pub struct _CHANGE_CONTEXT {
    pub hKey: HKEY,
    pub KeyString: LPWSTR,
}
pub type CHANGE_CONTEXT = _CHANGE_CONTEXT;
pub type PCHANGE_CONTEXT = *mut _CHANGE_CONTEXT;

// Contains raw pointers (*const GUID / LPCWSTR), so it must be `static mut`
// to avoid the Sync requirement; MAKEINTRESOURCEW(n) is lowered to a
// without_provenance pointer. IDS_* are resource IDs from resource.h and must
// be in scope from the caller.
unsafe extern "C" fn impl_from_ISecurityInformation(iface: *mut ISecurityInformation) -> PCRegKeySecurity {
    return ((iface as ULONG_PTR).wrapping_sub(core::mem::offset_of!(CRegKeySecurity, lpISecurityInformationVtbl) as ULONG_PTR))
        as PCRegKeySecurity;
}

unsafe extern "C" fn impl_from_ISecurityObjectTypeInfo(
    iface: *mut ISecurityObjectTypeInfo,
) -> PCRegKeySecurity {
    return ((iface as ULONG_PTR).wrapping_sub(core::mem::offset_of!(CRegKeySecurity, lpISecurityObjectTypeInfoVtbl) as ULONG_PTR)) as PCRegKeySecurity;
}

unsafe extern "C" fn impl_from_IEffectivePermission(
    iface: *mut IEffectivePermission,
) -> PCRegKeySecurity {
    return ((iface as ULONG_PTR)
        .wrapping_sub(core::mem::offset_of!(CRegKeySecurity, lpIEffectivePermissionVtbl) as ULONG_PTR))
        as PCRegKeySecurity;
}

unsafe extern "C" fn CRegKeySecurity_fnAddRef(mut obj: PCRegKeySecurity) -> ULONG {
    return InterlockedIncrement(
        core::ptr::addr_of_mut!((*obj).ref_) as *mut LONG,
    ) as ULONG;
}

unsafe extern "C" fn CRegKeySecurity_fnRelease(obj: PCRegKeySecurity) -> ULONG {
    let mut Ret: ULONG;

    Ret = InterlockedDecrement(core::ptr::addr_of_mut!((*obj).ref_) as *mut LONG) as ULONG;
    if Ret == 0 {
        HeapFree(GetProcessHeap(), 0, obj as PVOID);
    }

    return Ret;
}

unsafe extern "C" fn CRegKeySecurity_fnQueryInterface(
    obj: PCRegKeySecurity,
    iid: REFIID,
    pvObject: *mut PVOID,
) -> HRESULT {
    let mut pvObj: PVOID = core::ptr::null_mut();

    if IsEqualGUID!(iid, core::ptr::addr_of!(IID_IRegKeySecurity)) != false {
        pvObj = impl_to_interface!(obj, ISecurityInformation) as PVOID;
    }
    // REGEDIT_IMPLEMENT_ISECURITYINFORMATION2 is not defined: the
    // ISecurityInformation2 branch is compiled out.
    else if IsEqualGUID!(iid, core::ptr::addr_of!(IID_IEffectivePermission)) != false {
        pvObj = impl_to_interface!(obj, IEffectivePermission) as PVOID;
    } else if IsEqualGUID!(iid, core::ptr::addr_of!(IID_ISecurityObjectTypeInfo)) != false {
        pvObj = impl_to_interface!(obj, ISecurityObjectTypeInfo) as PVOID;
    }

    if pvObj.is_null() {
        return E_NOINTERFACE;
    }

    *pvObject = pvObj;
    CRegKeySecurity_fnAddRef(obj);

    return S_OK;
}
unsafe extern "system" fn ISecurityInformation_fnQueryInterface(
    mut this: *mut ISecurityInformation,
    iid: *const IID,
    mut pvObject: *mut PVOID,
) -> HRESULT {
    if IsEqualGUID!(iid, core::ptr::addr_of!(IID_IUnknown)) != false {
        *pvObject = this as PVOID;
        ISecurityInformation_fnAddRef(this);
        return S_OK;
    }

    return CRegKeySecurity_fnQueryInterface(impl_from_ISecurityInformation(this), iid, pvObject);
}
unsafe extern "system" fn ISecurityInformation_fnAddRef(this: *mut ISecurityInformation) -> ULONG {
    return CRegKeySecurity_fnAddRef(impl_from_ISecurityInformation(this));
}

unsafe extern "system" fn ISecurityInformation_fnRelease(this: *mut ISecurityInformation) -> ULONG {
    return CRegKeySecurity_fnRelease(impl_from_ISecurityInformation(this));
}
unsafe extern "system" fn ISecurityInformation_fnGetObjectInformation(
    mut this: *mut ISecurityInformation,
    mut pObjectInfo: PSI_OBJECT_INFO,
) -> HRESULT {
    let mut obj: PCRegKeySecurity = impl_from_ISecurityInformation(this);

    *pObjectInfo = (*obj).ObjectInfo;
    return S_OK;
}

unsafe extern "system" fn ISecurityInformation_fnGetSecurity(
    this: *mut ISecurityInformation,
    RequestedInformation: SECURITY_INFORMATION,
    ppSecurityDescriptor: *mut PSECURITY_DESCRIPTOR,
    fDefault: BOOL,
) -> HRESULT {
    let obj: PCRegKeySecurity = impl_from_ISecurityInformation(this);
    let mut ErrorCode: LONG;

    ErrorCode = GetNamedSecurityInfoW(
        core::ptr::addr_of_mut!((*obj).szRegKey) as LPWSTR,
        SE_REGISTRY_KEY,
        RequestedInformation,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        ppSecurityDescriptor,
    ) as LONG;

    return HRESULT_FROM_WIN32!(ErrorCode);
}
unsafe extern "system" fn ISecurityInformation_fnSetSecurity(
    this: *mut ISecurityInformation,
    RequestedInformation: SECURITY_INFORMATION,
    pSecurityDescriptor: PSECURITY_DESCRIPTOR,
) -> HRESULT {
    let obj: PCRegKeySecurity = impl_from_ISecurityInformation(this);

    /* FIXME */
    *(*obj).Btn = (TRUE) as i32;
    return S_OK;
}

unsafe extern "system" fn ISecurityInformation_fnGetAccessRights(
    this: *mut ISecurityInformation,
    pguidObjectType: *const GUID,
    dwFlags: DWORD,
    ppAccess: *mut PSI_ACCESS,
    pcAccesses: *mut ULONG,
    piDefaultAccess: *mut ULONG,
) -> HRESULT {
    *ppAccess = core::ptr::addr_of_mut!(RegAccess) as *mut SI_ACCESS as PSI_ACCESS;
    *pcAccesses = ARRAY_SIZE!(RegAccess) as ULONG;
    *piDefaultAccess = RegDefaultAccess as ULONG;
    return S_OK;
}

unsafe extern "system" fn ISecurityInformation_fnMapGeneric(
    this: *mut ISecurityInformation,
    pguidObjectType: *const GUID,
    pAceFlags: *mut UCHAR,
    pMask: *mut ACCESS_MASK,
) -> HRESULT {
    MapGenericMask(pMask as PDWORD, core::ptr::addr_of_mut!(RegAccessMasks));
    *pMask &= !SYNCHRONIZE;
    return S_OK;
}
unsafe extern "system" fn ISecurityInformation_fnGetInheritTypes(
    this: *mut ISecurityInformation,
    ppInheritTypes: *mut PSI_INHERIT_TYPE,
    pcInheritTypes: *mut ULONG,
) -> HRESULT {
    let obj: PCRegKeySecurity = impl_from_ISecurityInformation(this);

    /* FIXME */
    if (*obj).ObjectInfo.dwFlags & SI_CONTAINER != 0 {
        *ppInheritTypes = core::ptr::addr_of_mut!(RegInheritTypes) as *mut SI_INHERIT_TYPE as PSI_INHERIT_TYPE;
        *pcInheritTypes = ARRAY_SIZE!(RegInheritTypes) as ULONG;
        return S_OK;
    }

    return E_NOTIMPL;
}
unsafe extern "system" fn ISecurityInformation_fnPropertySheetPageCallback(
    this: *mut ISecurityInformation,
    hwnd: HWND,
    uMsg: UINT,
    uPage: SI_PAGE_TYPE,
) -> HRESULT {
    S_OK
}
unsafe extern "system" fn IEffectivePermission_fnQueryInterface(
    this: *mut IEffectivePermission,
    iid: REFIID,
    pvObject: *mut PVOID,
) -> HRESULT {
    if IsEqualGUID!(iid, core::ptr::addr_of!(IID_IUnknown)) != false {
        *pvObject = this as PVOID;
        IEffectivePermission_fnAddRef(this);
        return S_OK;
    }

    return CRegKeySecurity_fnQueryInterface(impl_from_IEffectivePermission(this), iid, pvObject);
}
unsafe extern "system" fn IEffectivePermission_fnAddRef(this: *mut IEffectivePermission) -> ULONG {
    return CRegKeySecurity_fnAddRef(impl_from_IEffectivePermission(this));
}
unsafe extern "system" fn IEffectivePermission_fnRelease(this: *mut IEffectivePermission) -> ULONG {
    return CRegKeySecurity_fnRelease(impl_from_IEffectivePermission(this));
}

unsafe extern "system" fn IEffectivePermission_fnGetEffectivePermission(
    this: *mut IEffectivePermission,
    pguidObjectType: *const GUID,
    pUserSid: PSID,
    pszServerName: LPCWSTR,
    pSD: PSECURITY_DESCRIPTOR,
    ppObjectTypeList: *mut POBJECT_TYPE_LIST,
    pcObjectTypeListLength: *mut ULONG,
    ppGrantedAccessList: *mut PACCESS_MASK,
    pcGrantedAccessListLength: *mut ULONG,
) -> HRESULT {
    let mut Dacl: PACL = core::ptr::null_mut();
    let mut DaclPresent: BOOL;
    let mut DaclDefaulted: BOOL;
    let mut GrantedAccessList: PACCESS_MASK;
    let mut ErrorCode: DWORD = ERROR_SUCCESS;
    let mut Trustee: TRUSTEE = core::mem::zeroed();
    static mut DefObjTypeList: OBJECT_TYPE_LIST = OBJECT_TYPE_LIST {
        Level: 0,
        Sbz: 0,
        ObjectType: core::ptr::null_mut(),
    };

    *ppObjectTypeList = core::ptr::addr_of_mut!(DefObjTypeList);
    *pcObjectTypeListLength = 1;

    BuildTrusteeWithSid(core::ptr::addr_of_mut!(Trustee), pUserSid);

    DaclPresent = 0;
    DaclDefaulted = 0;
    let mut fail: bool = false;
    if GetSecurityDescriptorDacl(
        pSD,
        core::ptr::addr_of_mut!(DaclPresent),
        core::ptr::addr_of_mut!(Dacl),
        core::ptr::addr_of_mut!(DaclDefaulted),
    ) != 0
        && DaclPresent != 0
    {
        GrantedAccessList = LocalAlloc(
            LMEM_FIXED as UINT,
            core::mem::size_of::<ACCESS_MASK>() as SIZE_T,
        ) as PACCESS_MASK;
        if GrantedAccessList.is_null() {
            fail = true;
        } else {
            ErrorCode = GetEffectiveRightsFromAcl(
                Dacl,
                core::ptr::addr_of_mut!(Trustee),
                GrantedAccessList,
            );
            if ErrorCode == ERROR_SUCCESS {
                *ppGrantedAccessList = GrantedAccessList;
                *pcGrantedAccessListLength = 1;
            } else {
                LocalFree(GrantedAccessList as HLOCAL);
            }
        }
    } else {
        fail = true;
    }
    if fail {
        ErrorCode = GetLastError();
    }

    return HRESULT_FROM_WIN32!(ErrorCode);
}
unsafe extern "system" fn ISecurityObjectTypeInfo_fnQueryInterface(
    this: *mut ISecurityObjectTypeInfo,
    iid: REFIID,
    pvObject: *mut PVOID,
) -> HRESULT {
    if IsEqualGUID!(iid, core::ptr::addr_of!(IID_IUnknown)) != false {
        *pvObject = this as PVOID;
        ISecurityObjectTypeInfo_fnAddRef(this);
        return S_OK;
    }

    return CRegKeySecurity_fnQueryInterface(
        impl_from_ISecurityObjectTypeInfo(this),
        iid,
        pvObject,
    );
}
unsafe extern "system" fn ISecurityObjectTypeInfo_fnAddRef(this: *mut ISecurityObjectTypeInfo) -> ULONG {
    return CRegKeySecurity_fnAddRef(impl_from_ISecurityObjectTypeInfo(this));
}
unsafe extern "system" fn ISecurityObjectTypeInfo_fnRelease(this: *mut ISecurityObjectTypeInfo) -> ULONG {
    return CRegKeySecurity_fnRelease(impl_from_ISecurityObjectTypeInfo(this));
}
unsafe extern "system" fn ISecurityObjectTypeInfo_fnGetInheritSource(
    this: *mut ISecurityObjectTypeInfo,
    si: SECURITY_INFORMATION,
    pACL: PACL,
    ppInheritArray: *mut PINHERITED_FROM,
) -> HRESULT {
    let obj: PCRegKeySecurity = impl_from_ISecurityObjectTypeInfo(this);
    let mut pif: PINHERITED_FROM = core::ptr::null_mut();
    let mut pif2: PINHERITED_FROM = core::ptr::null_mut();
    let mut pifSize: SIZE_T = 0;
    let mut ErrorCode: DWORD = 0;
    let mut i: DWORD = 0;
    let mut lpBuf: LPWSTR = core::ptr::null_mut();

    pifSize = ((*pACL).AceCount as SIZE_T)
        .wrapping_mul(core::mem::size_of::<INHERITED_FROM>() as SIZE_T);
    pif = HeapAlloc(
        GetProcessHeap(),
        0,
        pifSize,
    ) as PINHERITED_FROM;
    if pif.is_null() {
        return E_OUTOFMEMORY;
    }

    ErrorCode = GetInheritanceSourceW(
        core::ptr::addr_of_mut!((*obj).szRegKey) as LPWSTR,
        SE_REGISTRY_KEY,
        si,
        (((*obj).ObjectInfo.dwFlags & SI_CONTAINER) != 0)
            as ::core::ffi::c_int,
        core::ptr::null_mut(),
        0,
        pACL,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(RegAccessMasks),
        pif as PINHERITED_FROMW,
    );

    'cleanup: {
        if ErrorCode == ERROR_SUCCESS {
            /* Calculate the size of the buffer to return */
            i = 0;
            while i < (*pACL).AceCount as DWORD {
                if !(*pif.offset(i as isize)).AncestorName.is_null() {
                    pifSize = pifSize.wrapping_add(
                        ((wcslen(
                            (*pif.offset(i as isize)).AncestorName as *const WCHAR,
                        ) as SIZE_T)
                            .wrapping_add(1 as SIZE_T))
                            .wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
                    );
                }
                i += 1;
            }

            /* Allocate enough space for the array and the strings */
            pif2 = LocalAlloc(LMEM_FIXED, pifSize) as PINHERITED_FROM;
            if pif2.is_null() {
                ErrorCode = GetLastError();
                break 'cleanup;
            }

            /* copy the array and strings to the buffer */
            lpBuf = ((pif2 as usize)
                + ((*pACL).AceCount as usize)
                    .wrapping_mul(core::mem::size_of::<INHERITED_FROM>()))
                as LPWSTR;
            i = 0;
            while i < (*pACL).AceCount as DWORD {
                (*pif2.offset(i as isize)).GenerationGap =
                    (*pif.offset(i as isize)).GenerationGap;
                if !(*pif.offset(i as isize)).AncestorName.is_null() {
                    (*pif2.offset(i as isize)).AncestorName = lpBuf;
                    wcscpy(
                        lpBuf,
                        (*pif.offset(i as isize)).AncestorName,
                    );
                    lpBuf = lpBuf.offset(
                        (wcslen(
                            (*pif.offset(i as isize)).AncestorName as *const WCHAR,
                        ) as usize
                            + 1 as usize) as isize,
                    );
                } else {
                    (*pif2.offset(i as isize)).AncestorName = core::ptr::null_mut();
                }
                i += 1;
            }

            /* return the newly allocated array */
            *ppInheritArray = pif2;
        }
    }

    FreeInheritedFromArray(pif as PINHERITED_FROMW, (*pACL).AceCount as USHORT, core::ptr::null_mut());
    HeapFree(
        GetProcessHeap(),
        0,
        pif as PVOID,
    );

    return HRESULT_FROM_WIN32!(ErrorCode);
}

unsafe extern "C" fn CRegKeySecurity_fnConstructor(
    mut lpRegKey: LPWSTR,
    mut hRootKey: HKEY,
    mut ObjectInfo: *mut SI_OBJECT_INFO,
    mut Btn: *mut BOOL,
) -> PCRegKeySecurity {
    let mut obj: PCRegKeySecurity;
    obj = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY as DWORD,
        (core::mem::offset_of!(CRegKeySecurity, szRegKey)
            + (wcslen(lpRegKey as *const WCHAR) + 1 as usize)
                * core::mem::size_of::<WCHAR>()) as SIZE_T,
    ) as PCRegKeySecurity;
    if !obj.is_null() {
        (*obj).ref_ = 1 as DWORD;
        (*obj).lpISecurityInformationVtbl = core::ptr::addr_of!(vtblISecurityInformation);
        (*obj).lpIEffectivePermissionVtbl = core::ptr::addr_of!(vtblIEffectivePermission);
        (*obj).lpISecurityObjectTypeInfoVtbl = core::ptr::addr_of!(vtblISecurityObjectTypeInfo);
        (*obj).ObjectInfo = *ObjectInfo;
        (*obj).Btn = Btn;
        (*obj).hRootKey = hRootKey;
        StringCbCopyW(
            core::ptr::addr_of_mut!((*obj).szRegKey) as *mut WCHAR,
            core::mem::size_of_val(&(*obj).szRegKey),
            lpRegKey,
        );
    } else {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY as DWORD);
    }

    return obj;
}
#[no_mangle]
pub unsafe extern "C" fn InitializeAclUiDll() -> BOOL {
    hAclUiDll = LoadLibraryW(RUSTCONV_WSTR_BF8BAB706FCE.as_ptr()) as HMODULE;
    if hAclUiDll.is_null() {
        return (FALSE) as BOOL;
    }

    pfnEditSecurity = ::core::mem::transmute::<FARPROC, PEDITSECURITY>(GetProcAddress(
        hAclUiDll,
        (b"EditSecurity\0".as_ptr() as *const u8).cast(),
    ));
    if ::core::mem::transmute::<PEDITSECURITY, usize>(pfnEditSecurity) == 0 {
        FreeLibrary(hAclUiDll);
        hAclUiDll = core::ptr::null_mut();
        return (FALSE) as BOOL;
    }

    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn UnloadAclUiDll() {
    if !hAclUiDll.is_null() {
        FreeLibrary(hAclUiDll);
    }
}
#[no_mangle]
pub unsafe extern "C" fn RegKeyEditPermissions(hWndOwner: HWND, hKey: HKEY, lpMachine: LPCWSTR, lpKeyName: LPCWSTR) -> BOOL {
    let mut Result: BOOL = FALSE as BOOL;
    let mut lphKey: LPCWSTR = core::ptr::null();
    let mut lpKeyPath: LPWSTR = core::ptr::null_mut();
    let mut RegKeySecurity: PCRegKeySecurity;
    let mut ObjectInfo: SI_OBJECT_INFO = core::mem::zeroed();
    let mut lnMachine: size_t = 0 as size_t;
    let mut lnKeyName: size_t = 0 as size_t;

    if pfnEditSecurity.is_none() {
        return FALSE as BOOL;
    }

    if !lpMachine.is_null() {
        lnMachine = wcslen(lpMachine) as size_t;
    }
    if !lpKeyName.is_null() {
        lnKeyName = wcslen(lpKeyName) as size_t;
    }

    /* build registry path */
    if !lpMachine.is_null()
        && (*lpMachine.offset(0 as isize) == 0x0000u16
            || (*lpMachine.offset(0 as isize) == 0x002Eu16
                && *lpMachine.offset(1 as isize) == 0x002Eu16))
    {
        lnMachine = 0 as size_t;
    }

    'cleanup: {
        if hKey == HKEY_CLASSES_ROOT {
            lphKey = RUSTCONV_WSTR_D1EB02738F2B.as_ptr();
        } else if hKey == HKEY_CURRENT_USER {
            lphKey = RUSTCONV_WSTR_D76AA73CA09D.as_ptr();
        } else if hKey == HKEY_LOCAL_MACHINE {
            lphKey = RUSTCONV_WSTR_D89386175ED2.as_ptr();
        } else if hKey == HKEY_USERS {
            lphKey = RUSTCONV_WSTR_FD9080F09EC0.as_ptr();
        } else if hKey == HKEY_CURRENT_CONFIG {
            lphKey = RUSTCONV_WSTR_595FA2D85B6D.as_ptr();
        } else {
            break 'cleanup;
        }

        lpKeyPath = HeapAlloc(
            GetProcessHeap(),
            0 as DWORD,
            ((2 as SIZE_T
                + lnMachine as SIZE_T
                + 1 as SIZE_T
                + wcslen(lphKey) as SIZE_T
                + 1 as SIZE_T
                + lnKeyName as SIZE_T)
                * core::mem::size_of::<WCHAR>() as SIZE_T),
        ) as LPWSTR;
        if lpKeyPath.is_null() {
            SetLastError(ERROR_NOT_ENOUGH_MEMORY as DWORD);
            break 'cleanup;
        }
        *lpKeyPath.offset(0 as isize) = 0x0000u16;

        if lnMachine != 0 as size_t {
            wcscat(lpKeyPath, RUSTCONV_WSTR_3DBECEBCF87C.as_ptr());
            wcscat(lpKeyPath, lpMachine);
            wcscat(lpKeyPath, RUSTCONV_WSTR_B7FC02907B1F.as_ptr());
        }

        wcscat(lpKeyPath, lphKey);
        if !lpKeyName.is_null()
            && *lpKeyName.offset(0 as isize) != 0x0000u16
        {
            if *lpKeyName.offset(0 as isize) != 0x005Cu16 {
                wcscat(lpKeyPath, RUSTCONV_WSTR_B7FC02907B1F.as_ptr());
            }

            wcscat(lpKeyPath, lpKeyName);
        }

        ObjectInfo.dwFlags = (SI_EDIT_ALL
            | SI_ADVANCED
            | SI_CONTAINER
            | SI_EDIT_EFFECTIVE
            | SI_EDIT_PERMS
            | SI_OWNER_RECURSE
            | SI_RESET_DACL_TREE
            | SI_RESET_SACL_TREE) as DWORD;
        ObjectInfo.hInstance = hInst;
        ObjectInfo.pszServerName = lpMachine as LPWSTR;
        ObjectInfo.pszObjectName = lpKeyName as LPWSTR; /* FIXME */
        ObjectInfo.pszPageTitle = lpKeyName as LPWSTR; /* FIXME */

        RegKeySecurity = CRegKeySecurity_fnConstructor(
            lpKeyPath,
            hKey,
            core::ptr::addr_of_mut!(ObjectInfo),
            core::ptr::addr_of_mut!(Result),
        );
        if RegKeySecurity.is_null() {
            break 'cleanup;
        }

        /* display the security editor dialog */
        pfnEditSecurity.unwrap_unchecked()(
            hWndOwner,
            impl_to_interface!(RegKeySecurity, ISecurityInformation),
        );

        /* dereference the interface, it should be destroyed here */
        CRegKeySecurity_fnRelease(RegKeySecurity);
    }

    /* Cleanup: */
    if !lpKeyPath.is_null() {
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            lpKeyPath as PVOID,
        );
    }

    return Result;
}
