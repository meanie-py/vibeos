use crate::setupapi::cfgmgr::CM_Connect_MachineA;
use crate::setupapi::cfgmgr::CM_Connect_MachineW;
use crate::setupapi::cfgmgr::CM_Disconnect_Machine;
use crate::setupapi::cfgmgr::CM_Get_Class_Registry_PropertyA;
use crate::setupapi::cfgmgr::CM_Get_Class_Registry_PropertyW;
use crate::setupapi::cfgmgr::CM_Get_DevNode_Registry_Property_ExW;
use crate::setupapi::cfgmgr::CM_Get_Device_ID_List_ExW;
use crate::setupapi::cfgmgr::CM_Locate_DevNode_ExW;
use crate::setupapi::cfgmgr::CM_Set_Class_Registry_PropertyA;
use crate::setupapi::cfgmgr::CM_Set_Class_Registry_PropertyW;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut ClassInstall32: [WCHAR; 15] = [
    'C' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    's' as i32 as WCHAR,
    's' as i32 as WCHAR,
    'I' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    's' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    '3' as i32 as WCHAR,
    '2' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut DotServices: [WCHAR; 10] = [
    '.' as i32 as WCHAR,
    'S' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'v' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'c' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    's' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut InterfaceInstall32: [WCHAR; 19] = [
    'I' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'f' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'c' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'I' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    's' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    '3' as i32 as WCHAR,
    '2' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut SetupapiDll: [WCHAR; 13] = [
    's' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'u' as i32 as WCHAR,
    'p' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'p' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    '.' as i32 as WCHAR,
    'd' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut InstallParamsData: [INSTALL_PARAMS_DATA; 2] = unsafe {
    [
        _INSTALL_PARAMS_DATA {
            Function: 18 as DI_FUNCTION,
            UpdateHandler: Some(
                SETUP_PropertyChangeHandler
                    as unsafe extern "C" fn(
                        HDEVINFO,
                        PSP_DEVINFO_DATA,
                        PSP_CLASSINSTALL_HEADER,
                        DWORD,
                    ) -> BOOL,
            ),
            ParamsSize: ::core::mem::size_of::<SP_PROPCHANGE_PARAMS>() as ULONG,
            FieldOffset: core::mem::offset_of!(ClassInstallParams, PropChangeParams) as LONG,
        },
        _INSTALL_PARAMS_DATA {
            Function: 35 as DI_FUNCTION,
            UpdateHandler: Some(
                SETUP_PropertyAddPropertyAdvancedHandler
                    as unsafe extern "C" fn(
                        HDEVINFO,
                        PSP_DEVINFO_DATA,
                        PSP_CLASSINSTALL_HEADER,
                        DWORD,
                    ) -> BOOL,
            ),
            ParamsSize: ::core::mem::size_of::<SP_ADDPROPERTYPAGE_DATA>() as ULONG,
            /* FIELD_OFFSET(struct ClassInstallParams, AddPropertyPageData): c2rust baked amd64's 8; i386 is 4 for the second pointer. */
            FieldOffset: core::mem::offset_of!(ClassInstallParams, AddPropertyPageData) as LONG,
        },
    ]
};

pub static BackSlash: [u16; 2] = [0x005Cu16, 0x0000];
pub static ClassGUID: [u16; 10] = [0x0043u16, 0x006C, 0x0061, 0x0073, 0x0073, 0x0047, 0x0055, 0x0049, 0x0044, 0x0000];

pub type PROPERTY_PAGE_PROVIDER = Option<unsafe extern "system" fn(
    PropPageRequest: PSP_PROPSHEETPAGE_REQUEST,
    fAddFunc: LPFNADDPROPSHEETPAGE,
    lParam: LPARAM,
) -> BOOL>;

pub type UPDATE_CLASS_PARAM_HANDLER = Option<unsafe extern "C" fn(
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
    ClassInstallParams: PSP_CLASSINSTALL_HEADER,
    ClassInstallParamsSize: DWORD,
) -> BOOL>;


pub const UNKNOWN_ICON_INDEX: i32 = 18;

#[repr(C)]
pub struct ClassDevPropertySheetsData {
    pub PropertySheetHeader: LPPROPSHEETHEADERW,
    pub PropertySheetHeaderPageListSize: DWORD,
    pub NumberOfPages: DWORD,
    pub DontCancel: BOOL,
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiDestroyClassImageList(
    ClassImageListData: PSP_CLASSIMAGELIST_DATA,
) -> BOOL {
    let mut list: *mut ClassImageList = core::ptr::null_mut();
    let mut ret: BOOL = (FALSE as i32);

    if ClassImageListData.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
    } else if (*ClassImageListData).cbSize as usize
        != core::mem::size_of::<SP_CLASSIMAGELIST_DATA>() as usize
    {
        SetLastError(ERROR_INVALID_USER_BUFFER as DWORD);
    } else if {
        list = (*ClassImageListData).Reserved as *mut ClassImageList;
        list.is_null()
    } {
        SetLastError(ERROR_INVALID_USER_BUFFER as DWORD);
    } else if (*list).magic != SETUP_CLASS_IMAGE_LIST_MAGIC as DWORD {
        SetLastError(ERROR_INVALID_USER_BUFFER as DWORD);
    } else {
        /* If Reserved wasn't NULL, then this is valid too */
        if !(*ClassImageListData).ImageList.is_null() {
            ImageList_Destroy((*ClassImageListData).ImageList);
            (*ClassImageListData).ImageList = core::ptr::null_mut();
        }

        MyFree(list as PVOID);
        (*ClassImageListData).Reserved = 0;

        ret = (TRUE as i32);
    }

    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn SETUP_CreateDevicesList(
    list: *mut DeviceInfoSet,
    MachineName: PCWSTR,
    Class: *const GUID,
    Enumerator: PCWSTR,
) -> LONG {
    let mut Buffer: PWCHAR = core::ptr::null_mut();
    let mut BufferLength: DWORD = 4096;
    let mut InstancePath: PCWSTR;
    let mut deviceInfo: *mut DeviceInfo = core::ptr::null_mut();
    let mut ClassGuidBuffer: [WCHAR; (MAX_GUID_STRING_LEN as _)] = [0; (MAX_GUID_STRING_LEN as _)];
    let mut ClassGuidBufferSize: DWORD;
    let mut ClassGuid: GUID = core::mem::zeroed();
    let mut dnDevInst: DEVINST = 0;
    let mut cr: CONFIGRET;

    Buffer = HeapAlloc(GetProcessHeap(), 0, BufferLength as SIZE_T) as PWCHAR;
    if Buffer.is_null() {
        return (ERROR_NOT_ENOUGH_MEMORY) as LONG;
    }

    loop {
        cr = CM_Get_Device_ID_List_ExW(
            Enumerator,
            Buffer,
            BufferLength / (core::mem::size_of::<WCHAR>() as DWORD),
            if !Enumerator.is_null() { CM_GETIDLIST_FILTER_ENUMERATOR } else { CM_GETIDLIST_FILTER_NONE },
            (*list).hMachine,
        );
        if cr == CR_BUFFER_SMALL {
            if !Buffer.is_null() {
                HeapFree(GetProcessHeap(), 0, Buffer as LPVOID);
            }
            BufferLength *= 2;
            Buffer = HeapAlloc(GetProcessHeap(), 0, BufferLength as SIZE_T) as PWCHAR;
            if Buffer.is_null() {
                return (ERROR_NOT_ENOUGH_MEMORY) as LONG;
            }
        } else if cr != CR_SUCCESS {
            if !Buffer.is_null() {
                HeapFree(GetProcessHeap(), 0, Buffer as LPVOID);
            }
            return if cr == CR_REGISTRY_ERROR {
                ERROR_INVALID_DATA
            } else {
                GetErrorCodeFromCrCode(cr)
            } as LONG;
        }
        if !(cr != CR_SUCCESS) {
            break;
        }
    }

    InstancePath = Buffer as PCWSTR;
    while *InstancePath != UNICODE_NULL {
        'body: {
            cr = CM_Locate_DevNode_Ex(
                core::ptr::addr_of_mut!(dnDevInst),
                InstancePath as DEVINSTID_W,
                CM_LOCATE_DEVNODE_NORMAL,
                (*list).hMachine,
            );
            if cr != CR_SUCCESS {
                ERR!("CM_Locate_DevNode_ExW('%S') failed with status 0x%x\n", InstancePath, cr);
                break 'body;
            }

            ClassGuidBufferSize = core::mem::size_of::<[WCHAR; (MAX_GUID_STRING_LEN as _)]>() as DWORD;
            cr = CM_Get_DevNode_Registry_Property_ExW(
                dnDevInst,
                CM_DRP_CLASSGUID,
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(ClassGuidBuffer) as *mut WCHAR as PVOID,
                core::ptr::addr_of_mut!(ClassGuidBufferSize),
                0,
                (*list).hMachine,
            );
            if cr == CR_SUCCESS {
                ClassGuidBuffer[(MAX_GUID_STRING_LEN - 2) as usize] = '\0' as WCHAR;
                if UuidFromStringW(
                    (core::ptr::addr_of_mut!(ClassGuidBuffer) as *mut WCHAR).offset(1 as isize) as RPC_WSTR,
                    core::ptr::addr_of_mut!(ClassGuid),
                ) != RPC_S_OK
                {
                    ERR!("Invalid ClassGUID '%S' for device %S\n", core::ptr::addr_of_mut!(ClassGuidBuffer) as *mut WCHAR, InstancePath);
                    break 'body;
                }
            } else {
                memcpy(
                    core::ptr::addr_of_mut!(ClassGuid) as *mut core::ffi::c_void,
                    &GUID_NULL as *const GUID as *const core::ffi::c_void,
                    core::mem::size_of::<GUID>() as size_t,
                );
            }

            if !Class.is_null() && !IsEqualIID(core::ptr::addr_of_mut!(ClassGuid) as *const GUID, Class) {
                break 'body;
            }

            if CreateDeviceInfo(
                list,
                InstancePath,
                core::ptr::addr_of_mut!(ClassGuid),
                core::ptr::addr_of_mut!(deviceInfo),
            ) == 0
            {
                ERR!("Failed to create info for %S\n", InstancePath);
                HeapFree(GetProcessHeap(), 0, Buffer as LPVOID);
                return GetLastError() as LONG;
            }

            InsertTailList(
                core::ptr::addr_of_mut!((*list).ListHead),
                core::ptr::addr_of_mut!((*deviceInfo).ListEntry),
            );
        }
        InstancePath = InstancePath.offset(
            (wcslen(InstancePath as *const wchar_t) as usize + 1 as usize) as isize,
        );
    }

    HeapFree(GetProcessHeap(), 0, Buffer as LPVOID);
    return ERROR_SUCCESS as LONG;
}

unsafe extern "C" fn SETUP_GetIconIndex(hClassKey: HKEY, ImageIndex: PINT) -> BOOL {
    let mut Buffer: LPWSTR = core::ptr::null_mut();
    let mut dwRegType: DWORD = 0;
    let mut dwLength: DWORD = 0;
    let mut rc: LONG;
    let mut ret: BOOL = FALSE as BOOL;

    'cleanup: {
        /* Read icon registry key */
        rc = RegQueryValueExW(hClassKey, REGSTR_VAL_INSICON, core::ptr::null_mut(), &mut dwRegType, core::ptr::null_mut(), &mut dwLength) as LONG;
        if rc != ERROR_SUCCESS as LONG {
            SetLastError(rc as DWORD);
            break 'cleanup;
        } else if dwRegType != REG_SZ as DWORD {
            SetLastError(ERROR_INVALID_INDEX as DWORD);
            break 'cleanup;
        }
        Buffer = MyMalloc(dwLength + core::mem::size_of::<WCHAR>() as DWORD) as LPWSTR;
        if Buffer.is_null() {
            SetLastError(ERROR_NOT_ENOUGH_MEMORY as DWORD);
            break 'cleanup;
        }
        rc = RegQueryValueExW(hClassKey, REGSTR_VAL_INSICON, core::ptr::null_mut(), core::ptr::null_mut(), Buffer as LPBYTE, &mut dwLength) as LONG;
        if rc != ERROR_SUCCESS as LONG {
            SetLastError(rc as DWORD);
            break 'cleanup;
        }
        /* make sure the returned buffer is NULL-terminated */
        *Buffer.offset((dwLength as usize / core::mem::size_of::<WCHAR>()) as isize) = 0 as WCHAR;

        /* Transform icon value to a INT */
        *ImageIndex = atoiW(Buffer);
        ret = TRUE as BOOL;
    }

    MyFree(Buffer as PVOID);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiGetClassImageIndex(
    ClassImageListData: PSP_CLASSIMAGELIST_DATA,
    ClassGuid: *const GUID,
    ImageIndex: PINT,
) -> BOOL {
    let mut list: *mut ClassImageList = core::ptr::null_mut();
    let mut ret: BOOL = (FALSE as i32);

    if ClassImageListData.is_null() || ClassGuid.is_null() || ImageIndex.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
    } else if (*ClassImageListData).cbSize as usize != core::mem::size_of::<SP_CLASSIMAGELIST_DATA>() as usize {
        SetLastError(ERROR_INVALID_USER_BUFFER as DWORD);
    } else {
        list = (*ClassImageListData).Reserved as *mut ClassImageList;
        if list.is_null() {
            SetLastError(ERROR_INVALID_USER_BUFFER as DWORD);
        } else if (*list).magic != SETUP_CLASS_IMAGE_LIST_MAGIC as DWORD {
            SetLastError(ERROR_INVALID_USER_BUFFER as DWORD);
        } else {
            let mut i: DWORD;

            i = 0 as DWORD;
            while i < (*list).NumberOfGuids {
                if IsEqualIID(
                    ClassGuid,
                    (*list).Guids.offset(i as isize) as *const GUID,
                )
                {
                    break;
                }
                i = i.wrapping_add(1);
            }

            if i == (*list).NumberOfGuids
                || *(*list).IconIndexes.offset(i as isize) < 0
            {
                SetLastError(ERROR_FILE_NOT_FOUND as DWORD);
            } else {
                *ImageIndex = *(*list).IconIndexes.offset(i as isize);
                ret = (TRUE as i32);
            }
        }
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiGetClassImageList(
    ClassImageListData: PSP_CLASSIMAGELIST_DATA,
) -> BOOL {
    return SetupDiGetClassImageListExW(ClassImageListData, core::ptr::null_mut(), core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiGetClassImageListExA(
    ClassImageListData: PSP_CLASSIMAGELIST_DATA,
    MachineName: PCSTR,
    Reserved: PVOID,
) -> BOOL {
    let mut MachineNameW: PWSTR = core::ptr::null_mut::<WCHAR>();
    let mut ret: BOOL = 0;
    if !MachineName.is_null() {
        MachineNameW = pSetupMultiByteToUnicode(MachineName, CP_ACP as UINT);
        if MachineNameW.is_null() {
            return (FALSE) as BOOL;
        }
    }
    ret = SetupDiGetClassImageListExW(ClassImageListData, MachineNameW as PCWSTR, Reserved);
    MyFree(MachineNameW as PVOID);
    return ret;
}

unsafe extern "system" fn SETUP_GetClassIconInfo(
    ClassGuid: *const GUID,
    OutIndex: PINT,
    OutDllName: *mut LPWSTR,
) -> BOOL {
    let mut Buffer: LPWSTR = core::ptr::null_mut();
    let mut iconIndex: INT = -UNKNOWN_ICON_INDEX;
    let mut hKey: HKEY = INVALID_HANDLE_VALUE as HKEY;
    let mut ret: BOOL = (FALSE as i32);

    if !ClassGuid.is_null() {
        hKey = SetupDiOpenClassRegKey(ClassGuid, KEY_QUERY_VALUE as REGSAM);
        if hKey != INVALID_HANDLE_VALUE as HKEY {
            SETUP_GetIconIndex(hKey, core::ptr::addr_of_mut!(iconIndex));
        }
    }

    'quit: {
        if iconIndex > 0 {
            /* Look up icon in dll specified by Installer32 or EnumPropPages32 key */
            let mut Comma: PWCHAR = core::ptr::null_mut();
            let mut rc: LONG = 0;
            let mut dwRegType: DWORD = 0;
            let mut dwLength: DWORD = 0;
            rc = RegQueryValueExW(
                hKey,
                REGSTR_VAL_INSTALLER_32,
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(dwRegType),
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(dwLength),
            );
            if rc == (ERROR_SUCCESS as LONG) && dwRegType == REG_SZ {
                Buffer = MyMalloc(
                    (dwLength as usize + core::mem::size_of::<WCHAR>()) as DWORD,
                ) as LPWSTR;
                if Buffer.is_null() {
                    SetLastError(ERROR_NOT_ENOUGH_MEMORY);
                    break 'quit;
                }
                rc = RegQueryValueExW(
                    hKey,
                    REGSTR_VAL_INSTALLER_32,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    Buffer as LPBYTE,
                    core::ptr::addr_of_mut!(dwLength),
                );
                if rc != (ERROR_SUCCESS as LONG) {
                    SetLastError(rc as DWORD);
                    break 'quit;
                }
                /* make sure the returned buffer is NULL-terminated */
                *Buffer.offset((dwLength as usize / core::mem::size_of::<WCHAR>()) as isize) =
                    0;
            } else {
                rc = RegQueryValueExW(
                    hKey,
                    REGSTR_VAL_ENUMPROPPAGES_32,
                    core::ptr::null_mut(),
                    core::ptr::addr_of_mut!(dwRegType),
                    core::ptr::null_mut(),
                    core::ptr::addr_of_mut!(dwLength),
                );
                if ERROR_SUCCESS == (rc) as u32 && dwRegType == REG_SZ {
                    Buffer = MyMalloc(
                        (dwLength as usize + core::mem::size_of::<WCHAR>()) as DWORD,
                    ) as LPWSTR;
                    if Buffer.is_null() {
                        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
                        break 'quit;
                    }
                    rc = RegQueryValueExW(
                        hKey,
                        REGSTR_VAL_ENUMPROPPAGES_32,
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                        Buffer as LPBYTE,
                        core::ptr::addr_of_mut!(dwLength),
                    );
                    if rc != (ERROR_SUCCESS as LONG) {
                        SetLastError(rc as DWORD);
                        break 'quit;
                    }
                    /* make sure the returned buffer is NULL-terminated */
                    *Buffer.offset((dwLength as usize / core::mem::size_of::<WCHAR>()) as isize) =
                        0;
                } else {
                    /* Unable to find where to load the icon */
                    SetLastError(ERROR_FILE_NOT_FOUND);
                    break 'quit;
                }
            }
            Comma = strchrW(Buffer, ',' as WCHAR) as PWCHAR;
            if Comma.is_null() {
                SetLastError(ERROR_GEN_FAILURE);
                break 'quit;
            }
            *Comma = 0 as WCHAR;
            *OutDllName = Buffer;
        } else {
            /* Look up icon in setupapi.dll */
            iconIndex = -iconIndex;
            *OutDllName = core::ptr::null_mut();
        }

        *OutIndex = iconIndex;
        ret = (TRUE as i32);
    }

    if hKey != INVALID_HANDLE_VALUE as HKEY {
        RegCloseKey(hKey);
    }

    if !Buffer.is_null() && ret == 0 {
        MyFree(Buffer as PVOID);
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiGetClassImageListExW(
    ClassImageListData: PSP_CLASSIMAGELIST_DATA,
    MachineName: PCWSTR,
    Reserved: PVOID,
) -> BOOL {
    let mut ret: BOOL = (FALSE as i32);

    if ClassImageListData.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
    } else if (*ClassImageListData).cbSize as usize != core::mem::size_of::<SP_CLASSIMAGELIST_DATA>() as usize {
        SetLastError(ERROR_INVALID_USER_BUFFER);
    } else if !Reserved.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
    } else {
        let mut list: *mut ClassImageList = core::ptr::null_mut();
        let mut hDC: HDC = core::ptr::null_mut();
        let mut RequiredSize: DWORD = 0;
        let mut ilMask: DWORD = 0;
        let mut bkColor: DWORD = 0;
        let mut hIcon: HICON = core::ptr::null_mut();
        let mut size: DWORD = 0;
        let mut i: INT = 0;
        let mut bpp: INT = 0;
        let mut idx: UINT = 0;

        'cleanup: {
            /* Get list of all class GUIDs in given computer */
            ret = SetupDiBuildClassInfoListExW(
                0,
                core::ptr::null_mut(),
                0,
                core::ptr::addr_of_mut!(RequiredSize),
                MachineName,
                core::ptr::null_mut(),
            );
            if ret == 0 && GetLastError() != ERROR_INSUFFICIENT_BUFFER {
                break 'cleanup;
            }

            size = (core::mem::size_of::<ClassImageList>().wrapping_add(
                (core::mem::size_of::<GUID>().wrapping_add(core::mem::size_of::<INT>()))
                    .wrapping_mul(RequiredSize as usize),
            )) as DWORD;
            list = HeapAlloc(
                GetProcessHeap(),
                0,
                size as SIZE_T,
            ) as *mut ClassImageList;
            if list.is_null() {
                SetLastError(ERROR_NOT_ENOUGH_MEMORY);
                break 'cleanup;
            }
            (*list).magic = SETUP_CLASS_IMAGE_LIST_MAGIC;
            (*list).NumberOfGuids = RequiredSize;
            (*list).Guids = list.offset(1 as isize) as *mut GUID;
            (*list).IconIndexes = ((list.offset(1 as isize) as usize)
                .wrapping_add(
                    core::mem::size_of::<GUID>().wrapping_mul(RequiredSize as usize),
                )) as *mut INT;

            ret = SetupDiBuildClassInfoListExW(
                0,
                (*list).Guids,
                (*list).NumberOfGuids,
                core::ptr::addr_of_mut!(RequiredSize),
                MachineName,
                core::ptr::null_mut(),
            );
            if ret == 0 {
                break 'cleanup;
            } else if RequiredSize != (*list).NumberOfGuids {
                /* Hm. Class list changed since last call. Ignore
                 * this case as it should be very rare */
                SetLastError(ERROR_GEN_FAILURE);
                ret = (FALSE as i32);
                break 'cleanup;
            }

            /* Prepare a HIMAGELIST */
            InitCommonControls();

            hDC = GetDC(core::ptr::null_mut());
            if hDC.is_null() {
                break 'cleanup;
            }

            bpp = GetDeviceCaps(hDC, (BITSPIXEL as i32)) as INT;
            ReleaseDC(core::ptr::null_mut(), hDC);

            if bpp <= 4 {
                ilMask = ILC_COLOR4;
            } else if bpp <= 8 {
                ilMask = ILC_COLOR8;
            } else if bpp <= 16 {
                ilMask = ILC_COLOR16;
            } else if bpp <= 24 {
                ilMask = ILC_COLOR24;
            } else if bpp <= 32 {
                ilMask = ILC_COLOR32;
            } else {
                ilMask = ILC_COLOR;
            }

            ilMask |= ILC_MASK;

            (*ClassImageListData).ImageList = ImageList_Create(
                16,
                16,
                ilMask,
                100,
                10,
            );
            if (*ClassImageListData).ImageList.is_null() {
                break 'cleanup;
            }

            (*ClassImageListData).Reserved = list as ULONG_PTR;

            /* For some reason, Windows sets the list background to COLOR_WINDOW */
            bkColor = GetSysColor((COLOR_WINDOW as i32));
            ImageList_SetBkColor((*ClassImageListData).ImageList, bkColor);

            /* Now, we "simply" need to load icons associated with all class guids,
             * and put their index in the image list in the IconIndexes array */
            i = 0 as INT;
            while (i as DWORD) < (*list).NumberOfGuids {
                let mut miniIconIndex: INT = 0;
                let mut DllName: LPWSTR = core::ptr::null_mut();

                if SETUP_GetClassIconInfo(
                    (*list).Guids.offset(i as isize) as *mut GUID,
                    core::ptr::addr_of_mut!(miniIconIndex),
                    core::ptr::addr_of_mut!(DllName),
                ) != 0
                {
                    if !DllName.is_null()
                        && ExtractIconExW(
                            DllName as LPCWSTR,
                            -miniIconIndex,
                            core::ptr::null_mut::<HICON>(),
                            core::ptr::addr_of_mut!(hIcon),
                            1,
                        ) == 1
                    {
                        *(*list).IconIndexes.offset(i as isize) =
                            ImageList_AddIcon((*ClassImageListData).ImageList, hIcon) as INT;
                    } else if DllName.is_null() {
                        hIcon = LoadImage(
                            hInstance,
                            MAKEINTRESOURCEW!(miniIconIndex),
                            IMAGE_ICON,
                            16,
                            16,
                            LR_DEFAULTCOLOR,
                        ) as HICON;
                        *(*list).IconIndexes.offset(i as isize) =
                            ImageList_AddIcon((*ClassImageListData).ImageList, hIcon) as INT;
                    }

                    if !hIcon.is_null() {
                        DestroyIcon(hIcon);
                    } else {
                        *(*list).IconIndexes.offset(i as isize) =
                            -(1 as INT);
                    }

                    if !DllName.is_null() {
                        MyFree(DllName as PVOID);
                    }
                } else {
                    /* Special value to indicate that the icon is unavailable */
                    *(*list).IconIndexes.offset(i as isize) =
                        -(1 as INT);
                }
                i += 1;
            }

            /* Finally, add the overlay icons to the image list */
            i = 0 as INT;
            while i <= 2 as INT {
                hIcon = LoadImage(
                    hInstance,
                    MAKEINTRESOURCEW!(500 + i),
                    IMAGE_ICON,
                    16,
                    16,
                    LR_DEFAULTCOLOR,
                ) as HICON;
                if !hIcon.is_null() {
                    idx = ImageList_AddIcon((*ClassImageListData).ImageList, hIcon) as UINT;
                    if idx != (-(1 as INT)) as UINT {
                        ImageList_SetOverlayImage(
                            (*ClassImageListData).ImageList,
                            idx as INT,
                            i + 1 as INT,
                        );
                    }
                    DestroyIcon(hIcon);
                }
                i += 1;
            }

            ret = (TRUE as i32);
        }

        // cleanup:
        if ret == 0 {
            if (*ClassImageListData).Reserved != 0 {
                SetupDiDestroyClassImageList(ClassImageListData);
            } else if !list.is_null() {
                MyFree(list as PVOID);
            }
        }
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiGetClassInstallParamsA(
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
    ClassInstallParams: PSP_CLASSINSTALL_HEADER,
    ClassInstallParamsSize: DWORD,
    RequiredSize: PDWORD,
) -> BOOL {
    // FIXME("SetupDiGetClassInstallParamsA(%p %p %p %lu %p) Stub\n", DeviceInfoSet, DeviceInfoData, ClassInstallParams, ClassInstallParamsSize, RequiredSize);
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiGetClassInstallParamsW(
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
    ClassInstallParams: PSP_CLASSINSTALL_HEADER,
    ClassInstallParamsSize: DWORD,
    RequiredSize: PDWORD,
) -> BOOL {
    // FIXME("SetupDiGetClassInstallParamsW(%p %p %p %lu %p) Stub\n", DeviceInfoSet, DeviceInfoData, ClassInstallParams, ClassInstallParamsSize, RequiredSize);
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiLoadClassIcon(
    ClassGuid: *const GUID,
    LargeIcon: *mut HICON,
    MiniIconIndex: PINT,
) -> BOOL {
    let mut iconIndex: INT = 0;
    let mut DllName: LPWSTR = core::ptr::null_mut();
    let mut ret: BOOL = (FALSE as i32);
    let mut hIcon: HICON = core::ptr::null_mut();

    'cleanup: {
        if !LargeIcon.is_null() {
            if SETUP_GetClassIconInfo(ClassGuid, &raw mut iconIndex, &raw mut DllName) == 0 {
                break 'cleanup;
            }

            if DllName.is_null()
                || ExtractIconExW(
                    DllName as LPCWSTR,
                    -iconIndex,
                    &raw mut hIcon,
                    core::ptr::null_mut(),
                    1,
                ) != 1
                || hIcon.is_null()
            {
                /* load the default unknown device icon if ExtractIcon failed */
                if !DllName.is_null() {
                    iconIndex = UNKNOWN_ICON_INDEX;
                }

                hIcon = LoadImage(
                    hInstance,
                    MAKEINTRESOURCEW!(iconIndex) as LPCWSTR,
                    IMAGE_ICON,
                    32,
                    32,
                    LR_DEFAULTCOLOR,
                ) as HICON;

                if hIcon.is_null() {
                    break 'cleanup;
                }
            }

            *LargeIcon = hIcon;
        }

        if !MiniIconIndex.is_null() {
            *MiniIconIndex = iconIndex;
        }

        ret = (TRUE as i32);
    }

    if !DllName.is_null() {
        MyFree(DllName as PVOID);
    }

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/setupapi/devclass.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"SetupDiLoadClassIcon\0" as *const u8 as *const ::core::ffi::c_char,
            698 as ::core::ffi::c_int,
            b"Returning %d\n\0" as *const u8 as *const ::core::ffi::c_char,
            ret,
        );
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiInstallClassExW(
    hwndParent: HWND,
    InfFileName: PCWSTR,
    Flags: DWORD,
    FileQueue: HSPFILEQ,
    InterfaceClassGuid: *const GUID,
    Reserved1: PVOID,
    Reserved2: PVOID,
) -> BOOL {
    let mut ret: BOOL = (FALSE as i32);

    DPRINT1!(
        "%p %s 0x%lx %p %s %p %p\n",
        hwndParent,
        debugstr_w(InfFileName),
        Flags,
        FileQueue,
        debugstr_guid(InterfaceClassGuid),
        Reserved1,
        Reserved2
    );

    if InfFileName.is_null() {
        DPRINT1!("Case not implemented: InfFileName NULL\n");
        SetLastError(ERROR_CALL_NOT_IMPLEMENTED as DWORD);
    } else if (Flags & !(DI_NOVCP | DI_NOBROWSE | DI_FORCECOPY | DI_QUIETINSTALL)) != 0 {
        DPRINT1!(
            "Unknown flags: 0x%08lx\n",
            Flags & !(DI_NOVCP | DI_NOBROWSE | DI_FORCECOPY | DI_QUIETINSTALL)
        );
        SetLastError(ERROR_INVALID_FLAGS as DWORD);
    } else if (Flags & DI_NOVCP) != 0 && FileQueue.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
    } else if !Reserved1.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
    } else if !Reserved2.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
    } else {
        let mut hDeviceInfo: HDEVINFO = INVALID_HANDLE_VALUE;
        let mut InstallParams: SP_DEVINSTALL_PARAMS_W = core::mem::zeroed();
        let mut SectionName: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
        let mut hInf: HINF = INVALID_HANDLE_VALUE as HINF;
        let mut hRootKey: HKEY = INVALID_HANDLE_VALUE as HKEY;
        let mut callback_context: PVOID = core::ptr::null_mut();

        'cleanup: {
            hDeviceInfo = SetupDiCreateDeviceInfoList(core::ptr::null(), core::ptr::null_mut());
            if hDeviceInfo == INVALID_HANDLE_VALUE as HDEVINFO {
                break 'cleanup;
            }

            InstallParams.cbSize = core::mem::size_of::<SP_DEVINSTALL_PARAMS>() as DWORD;
            if SetupDiGetDeviceInstallParamsW(hDeviceInfo, core::ptr::null_mut(), core::ptr::addr_of_mut!(InstallParams)) == 0 {
                break 'cleanup;
            }

            InstallParams.Flags &= !(DI_NOVCP | DI_NOBROWSE | DI_QUIETINSTALL);
            InstallParams.Flags |= Flags & (DI_NOVCP | DI_NOBROWSE | DI_QUIETINSTALL);
            if (Flags & DI_NOVCP) != 0 {
                InstallParams.FileQueue = FileQueue;
            }
            if SetupDiSetDeviceInstallParamsW(hDeviceInfo, core::ptr::null_mut(), core::ptr::addr_of_mut!(InstallParams)) == 0 {
                break 'cleanup;
            }

            /* Open the .inf file */
            hInf = SetupOpenInfFileW(
                InfFileName,
                core::ptr::null(),
                INF_STYLE_WIN4 as DWORD,
                core::ptr::null_mut(),
            );
            if hInf == INVALID_HANDLE_VALUE as HINF {
                break 'cleanup;
            }

            /* Try to append a layout file */
            SetupOpenAppendInfFileW(core::ptr::null(), hInf, core::ptr::null_mut());

            if !InterfaceClassGuid.is_null() {
                /* Retrieve the actual section name */
                ret = SetupDiGetActualSectionToInstallW(
                    hInf,
                    core::ptr::addr_of!(InterfaceInstall32) as PCWSTR,
                    core::ptr::addr_of_mut!(SectionName) as *mut WCHAR as PWSTR,
                    MAX_PATH as DWORD,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                );
                if ret == 0 {
                    break 'cleanup;
                }

                /* Open registry key related to this interface */
                /* FIXME: What happens if the key doesn't exist? */
                hRootKey = SetupDiOpenClassRegKeyExW(
                    InterfaceClassGuid,
                    KEY_ENUMERATE_SUB_KEYS as REGSAM,
                    DIOCR_INTERFACE as DWORD,
                    core::ptr::null(),
                    core::ptr::null_mut(),
                );
                if hRootKey == INVALID_HANDLE_VALUE as HKEY {
                    break 'cleanup;
                }

                /* SetupDiCreateDeviceInterface??? */
                SetLastError(ERROR_CALL_NOT_IMPLEMENTED as DWORD);
            } else {
                /* Create or open the class registry key 'HKLM\CurrentControlSet\Class\{GUID}' */
                hRootKey = SETUP_CreateClassKey(hInf);
                if hRootKey == INVALID_HANDLE_VALUE as HKEY {
                    break 'cleanup;
                }

                /* Retrieve the actual section name */
                ret = SetupDiGetActualSectionToInstallW(
                    hInf,
                    core::ptr::addr_of!(ClassInstall32) as PCWSTR,
                    core::ptr::addr_of_mut!(SectionName) as *mut WCHAR as PWSTR,
                    (MAX_PATH as DWORD).wrapping_sub(strlenW(core::ptr::addr_of!(DotServices) as PCWSTR) as DWORD),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                );
                if ret == 0 {
                    break 'cleanup;
                }

                callback_context = SetupInitDefaultQueueCallback(hwndParent);
                if callback_context.is_null() {
                    break 'cleanup;
                }

                ret = SetupInstallFromInfSectionW(
                    hwndParent,
                    hInf,
                    core::ptr::addr_of_mut!(SectionName) as *mut WCHAR as PCWSTR,
                    (SPINST_REGISTRY | SPINST_FILES | SPINST_BITREG | SPINST_INIFILES | SPINST_INI2REG) as UINT,
                    hRootKey,
                    core::ptr::null(), /* FIXME: SourceRootPath */
                    (if (Flags & DI_NOVCP) == 0 && (Flags & DI_FORCECOPY) != 0 {
                        SP_COPY_FORCE_IN_USE
                    } else {
                        0
                    }) as UINT, /* CopyFlags */
                    Some(SetupDefaultQueueCallbackW),
                    callback_context,
                    hDeviceInfo,
                    core::ptr::null_mut(),
                );
                if ret == 0 {
                    break 'cleanup;
                }

                /* OPTIONAL: Install .Services section */
                lstrcatW(
                    core::ptr::addr_of_mut!(SectionName) as *mut WCHAR as LPWSTR,
                    core::ptr::addr_of!(DotServices) as LPCWSTR,
                );
                SetupInstallServicesFromInfSectionExW(
                    hInf,
                    core::ptr::addr_of_mut!(SectionName) as *mut WCHAR as PCWSTR,
                    0 as DWORD,
                    hDeviceInfo,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                );
                ret = TRUE as BOOL;
            }
        }

        if hDeviceInfo != INVALID_HANDLE_VALUE as HDEVINFO {
            SetupDiDestroyDeviceInfoList(hDeviceInfo);
        }
        if hInf != INVALID_HANDLE_VALUE as HINF {
            SetupCloseInfFile(hInf);
        }
        if hRootKey != INVALID_HANDLE_VALUE as HKEY {
            RegCloseKey(hRootKey);
        }
        SetupTermDefaultQueueCallback(callback_context);
    }

    DPRINT1!("Returning %d\n", ret);
    return ret;
}

unsafe extern "C" fn SETUP_PropertyChangeHandler(
    mut DeviceInfoSet: HDEVINFO,
    mut DeviceInfoData: PSP_DEVINFO_DATA,
    mut ClassInstallParams: PSP_CLASSINSTALL_HEADER,
    mut ClassInstallParamsSize: DWORD,
) -> BOOL {
    let mut PropChangeParams: PSP_PROPCHANGE_PARAMS = ClassInstallParams as PSP_PROPCHANGE_PARAMS;
    let mut ret: BOOL = FALSE as BOOL;

    'done: {
        if DeviceInfoData.is_null() {
            SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        } else if ClassInstallParamsSize as usize
            != ::core::mem::size_of::<SP_PROPCHANGE_PARAMS>() as usize
        {
            SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        } else if !PropChangeParams.is_null()
            && (*PropChangeParams).StateChange != DICS_ENABLE as DWORD
            && (*PropChangeParams).StateChange != DICS_DISABLE as DWORD
            && (*PropChangeParams).StateChange != DICS_PROPCHANGE as DWORD
            && (*PropChangeParams).StateChange != DICS_START as DWORD
            && (*PropChangeParams).StateChange != DICS_STOP as DWORD
        {
            SetLastError(ERROR_INVALID_FLAGS as DWORD);
        } else if !PropChangeParams.is_null()
            && (*PropChangeParams).Scope != DICS_FLAG_GLOBAL as DWORD
            && (*PropChangeParams).Scope != DICS_FLAG_CONFIGSPECIFIC as DWORD
        {
            SetLastError(ERROR_INVALID_FLAGS as DWORD);
        } else if !PropChangeParams.is_null()
            && ((*PropChangeParams).StateChange == DICS_START as DWORD
                || (*PropChangeParams).StateChange == DICS_STOP as DWORD)
            && (*PropChangeParams).Scope != DICS_FLAG_CONFIGSPECIFIC as DWORD
        {
            SetLastError(ERROR_INVALID_USER_BUFFER as DWORD);
        } else {
            let mut CurrentPropChangeParams: *mut PSP_PROPCHANGE_PARAMS =
                ::core::ptr::null_mut::<PSP_PROPCHANGE_PARAMS>();
            let mut deviceInfo: *mut DeviceInfo = (*DeviceInfoData).Reserved as *mut DeviceInfo;
            CurrentPropChangeParams = &raw mut (*deviceInfo).ClassInstallParams.PropChangeParams;

            if !(*CurrentPropChangeParams).is_null() {
                MyFree(*CurrentPropChangeParams as PVOID);
                *CurrentPropChangeParams = ::core::ptr::null_mut::<_SP_PROPCHANGE_PARAMS>();
            }
            if !PropChangeParams.is_null() {
                *CurrentPropChangeParams =
                    MyMalloc(ClassInstallParamsSize) as PSP_PROPCHANGE_PARAMS;
                if (*CurrentPropChangeParams).is_null() {
                    SetLastError(ERROR_NOT_ENOUGH_MEMORY as DWORD);
                    break 'done;
                }
                memcpy(
                    *CurrentPropChangeParams as *mut ::core::ffi::c_void,
                    PropChangeParams as *const ::core::ffi::c_void,
                    ClassInstallParamsSize as usize,
                );
            }
            ret = TRUE as BOOL;
        }
    }

    return ret;
}

unsafe extern "C" fn SETUP_PropertyAddPropertyAdvancedHandler(
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
    ClassInstallParams: PSP_CLASSINSTALL_HEADER,
    ClassInstallParamsSize: DWORD,
) -> BOOL {
    let AddPropertyPageData: PSP_ADDPROPERTYPAGE_DATA =
        ClassInstallParams as PSP_ADDPROPERTYPAGE_DATA;
    let mut ret: BOOL = (FALSE as i32);

    'done: {
        if ClassInstallParamsSize as usize != core::mem::size_of::<SP_PROPCHANGE_PARAMS>() {
            SetLastError(ERROR_INVALID_PARAMETER);
        } else if !AddPropertyPageData.is_null()
            && (*AddPropertyPageData).Flags != 0
        {
            SetLastError(ERROR_INVALID_FLAGS);
        } else if !AddPropertyPageData.is_null()
            && (*AddPropertyPageData).NumDynamicPages >= MAX_INSTALLWIZARD_DYNAPAGES
        {
            SetLastError(ERROR_INVALID_USER_BUFFER);
        } else {
            let mut CurrentAddPropertyPageData: *mut PSP_ADDPROPERTYPAGE_DATA =
                core::ptr::null_mut::<PSP_ADDPROPERTYPAGE_DATA>();
            if DeviceInfoData.is_null() {
                let list: *mut DeviceInfoSet = DeviceInfoSet as *mut DeviceInfoSet;
                CurrentAddPropertyPageData =
                    core::ptr::addr_of_mut!((*list).ClassInstallParams.AddPropertyPageData);
            } else {
                let deviceInfo: *mut DeviceInfo =
                    (*DeviceInfoData).Reserved as *mut DeviceInfo;
                CurrentAddPropertyPageData =
                    core::ptr::addr_of_mut!((*deviceInfo).ClassInstallParams.AddPropertyPageData);
            }
            if !(*CurrentAddPropertyPageData).is_null() {
                MyFree(*CurrentAddPropertyPageData as PVOID);
                *CurrentAddPropertyPageData = core::ptr::null_mut();
            }
            if !AddPropertyPageData.is_null() {
                *CurrentAddPropertyPageData =
                    MyMalloc(ClassInstallParamsSize) as PSP_ADDPROPERTYPAGE_DATA;
                if (*CurrentAddPropertyPageData).is_null() {
                    SetLastError(ERROR_NOT_ENOUGH_MEMORY);
                    break 'done;
                }
                memcpy(
                    *CurrentAddPropertyPageData as *mut core::ffi::c_void,
                    AddPropertyPageData as *const core::ffi::c_void,
                    ClassInstallParamsSize as usize,
                );
            }
            ret = (TRUE as i32);
        }
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiSetClassInstallParamsW(
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
    ClassInstallParams: PSP_CLASSINSTALL_HEADER,
    ClassInstallParamsSize: DWORD,
) -> BOOL {
    let mut list: *mut DeviceInfoSet = core::ptr::null_mut();
    let mut ret: BOOL = FALSE as BOOL;

    'done: {
        if DeviceInfoSet.is_null() {
            SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        } else if DeviceInfoSet == (INVALID_HANDLE_VALUE as HDEVINFO) {
            SetLastError(ERROR_INVALID_HANDLE as DWORD);
        } else {
            list = DeviceInfoSet as *mut DeviceInfoSet;
            if (*list).magic != SETUP_DEVICE_INFO_SET_MAGIC as DWORD {
                SetLastError(ERROR_INVALID_HANDLE as DWORD);
            } else if !DeviceInfoData.is_null()
                && (*DeviceInfoData).cbSize as usize != core::mem::size_of::<SP_DEVINFO_DATA>() as usize
            {
                SetLastError(ERROR_INVALID_USER_BUFFER as DWORD);
            } else if !ClassInstallParams.is_null()
                && (*ClassInstallParams).cbSize as usize != core::mem::size_of::<SP_CLASSINSTALL_HEADER>() as usize
            {
                SetLastError(ERROR_INVALID_USER_BUFFER as DWORD);
            } else if !ClassInstallParams.is_null()
                && (ClassInstallParamsSize as usize) < core::mem::size_of::<SP_CLASSINSTALL_HEADER>() as usize
            {
                SetLastError(ERROR_INVALID_PARAMETER as DWORD);
            } else if ClassInstallParams.is_null()
                && ClassInstallParamsSize != 0 as DWORD
            {
                SetLastError(ERROR_INVALID_PARAMETER as DWORD);
            } else {
                let mut InstallParams: SP_DEVINSTALL_PARAMS_W = core::mem::zeroed();
                let mut Result: BOOL = 0;

                InstallParams.cbSize = core::mem::size_of::<SP_DEVINSTALL_PARAMS_W>() as DWORD;
                Result = SetupDiGetDeviceInstallParamsW(
                    DeviceInfoSet,
                    DeviceInfoData,
                    core::ptr::addr_of_mut!(InstallParams),
                );
                if Result == 0 {
                    break 'done;
                }

                if !ClassInstallParams.is_null() {
                    let mut i: DWORD = 0 as DWORD;
                    loop {
                        if !((i as usize)
                            < core::mem::size_of_val(&InstallParamsData)
                                / core::mem::size_of_val(
                                    &InstallParamsData[0 as usize],
                                ))
                        {
                            break;
                        }
                        if InstallParamsData[i as usize].Function
                            == (*ClassInstallParams).InstallFunction
                        {
                            ret = InstallParamsData[i as usize]
                                .UpdateHandler
                                .unwrap_unchecked()(
                                DeviceInfoSet,
                                DeviceInfoData,
                                ClassInstallParams,
                                ClassInstallParamsSize,
                            );
                            if ret != 0 {
                                InstallParams.Flags |= DI_CLASSINSTALLPARAMS as DWORD;
                                ret = SetupDiSetDeviceInstallParamsW(
                                    DeviceInfoSet,
                                    DeviceInfoData,
                                    core::ptr::addr_of_mut!(InstallParams),
                                );
                            }
                            break 'done;
                        }
                        i = i.wrapping_add(1);
                    }
                    ERR!(
                        "InstallFunction %u has no associated update handler\n",
                        (*ClassInstallParams).InstallFunction
                    );
                    SetLastError(ERROR_CALL_NOT_IMPLEMENTED as DWORD);
                    break 'done;
                } else {
                    InstallParams.Flags &= !(DI_CLASSINSTALLPARAMS as DWORD);
                    ret = SetupDiSetDeviceInstallParamsW(
                        DeviceInfoSet,
                        DeviceInfoData,
                        core::ptr::addr_of_mut!(InstallParams),
                    );
                }
            }
        }
    }

    TRACE!("Returning %d\n", ret);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiGetClassDevPropertySheetsA(
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
    PropertySheetHeader: LPPROPSHEETHEADERA,
    PropertySheetHeaderPageListSize: DWORD,
    RequiredSize: PDWORD,
    PropertySheetType: DWORD,
) -> BOOL {
    let mut psh: PROPSHEETHEADERW = core::mem::zeroed();
    let mut ret: BOOL = (FALSE as i32);

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/setupapi/devclass.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"SetupDiGetClassDevPropertySheetsA\0" as *const u8 as *const ::core::ffi::c_char,
            1062 as ::core::ffi::c_int,
            b"%p %p %p 0%lx %p 0x%lx\n\0" as *const u8 as *const ::core::ffi::c_char,
            DeviceInfoSet,
            DeviceInfoData,
            PropertySheetHeader,
            PropertySheetHeaderPageListSize,
            RequiredSize,
            PropertySheetType,
        );
    }

    if !PropertySheetHeader.is_null() {
        psh.dwFlags = (*PropertySheetHeader).dwFlags;
        psh.__bindgen_anon_3.phpage = (*PropertySheetHeader).__bindgen_anon_3.phpage;
        psh.nPages = (*PropertySheetHeader).nPages;
    }

    ret = SetupDiGetClassDevPropertySheetsW(
        DeviceInfoSet,
        DeviceInfoData,
        if !PropertySheetHeader.is_null() {
            core::ptr::addr_of_mut!(psh)
        } else {
            core::ptr::null_mut::<_PROPSHEETHEADERW>()
        },
        PropertySheetHeaderPageListSize,
        RequiredSize,
        PropertySheetType,
    );

    if ret != 0 {
        (*PropertySheetHeader).nPages = psh.nPages;
    }

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl_0,
            __dbch_0,
            b"dll/win32/setupapi/devclass.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"SetupDiGetClassDevPropertySheetsA\0" as *const u8 as *const ::core::ffi::c_char,
            1079 as ::core::ffi::c_int,
            b"Returning %d\n\0" as *const u8 as *const ::core::ffi::c_char,
            ret,
        );
    }

    return ret;
}
unsafe extern "system" fn SETUP_GetClassDevPropertySheetsCallback(
    hPropSheetPage: HPROPSHEETPAGE,
    lParam: LPARAM,
) -> BOOL {
    let mut PropPageData: *mut ClassDevPropertySheetsData =
        core::ptr::null_mut::<ClassDevPropertySheetsData>();

    PropPageData = lParam as *mut ClassDevPropertySheetsData;

    (*PropPageData).NumberOfPages = (*PropPageData).NumberOfPages.wrapping_add(1);

    if ((*(*PropPageData).PropertySheetHeader).nPages as DWORD)
        < (*PropPageData).PropertySheetHeaderPageListSize
    {
        let ref mut fresh16 = *(*(*PropPageData).PropertySheetHeader)
            .__bindgen_anon_3.phpage
            .offset((*(*PropPageData).PropertySheetHeader).nPages as isize);
        *fresh16 = hPropSheetPage;
        (*(*PropPageData).PropertySheetHeader).nPages = (*(*PropPageData).PropertySheetHeader)
            .nPages
            .wrapping_add(1);
        return (TRUE) as BOOL;
    }

    return (*PropPageData).DontCancel;
}

unsafe extern "C" fn SETUP_GetValueString(
    hKey: HKEY,
    lpValueName: LPWSTR,
    lpString: *mut LPWSTR,
) -> DWORD {
    let mut lpBuffer: LPWSTR;
    let mut dwLength: DWORD = 0;
    let mut dwRegType: DWORD = core::mem::zeroed();
    let mut rc: DWORD;

    *lpString = core::ptr::null_mut();

    RegQueryValueExW(
        hKey,
        lpValueName,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(dwRegType),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(dwLength),
    );

    if dwLength == 0 || dwRegType != REG_SZ {
        return ERROR_FILE_NOT_FOUND;
    }

    lpBuffer = HeapAlloc(
        GetProcessHeap(),
        0,
        (dwLength + core::mem::size_of::<WCHAR>() as DWORD) as SIZE_T,
    ) as LPWSTR;
    if lpBuffer.is_null() {
        return ERROR_NOT_ENOUGH_MEMORY;
    }

    rc = (RegQueryValueExW(
        hKey,
        lpValueName,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        lpBuffer as LPBYTE,
        core::ptr::addr_of_mut!(dwLength),
    )) as u32;
    if rc != ERROR_SUCCESS {
        HeapFree(
            GetProcessHeap(),
            0,
            lpBuffer as PVOID,
        );
        return rc;
    }

    *lpBuffer.add((dwLength / core::mem::size_of::<WCHAR>() as DWORD) as usize) =
        UNICODE_NULL as WCHAR;

    *lpString = lpBuffer;

    return ERROR_SUCCESS;
}

unsafe extern "C" fn SETUP_CallInstaller(
    mut InstallFunction: DI_FUNCTION,
    mut DeviceInfoSet: HDEVINFO,
    mut DeviceInfoData: PSP_DEVINFO_DATA,
    mut PageData: PSP_ADDPROPERTYPAGE_DATA,
) -> BOOL {
    let mut pClassInstallParams: PSP_CLASSINSTALL_HEADER = core::ptr::null_mut();
    let mut dwSize: DWORD = 0;
    let mut dwError: DWORD = 0;
    let mut ret: BOOL = (TRUE as i32);

    /* Get the size of the old class install parameters */
    if SetupDiGetClassInstallParams(
        DeviceInfoSet,
        DeviceInfoData,
        core::ptr::null_mut(),
        0,
        core::ptr::addr_of_mut!(dwSize),
    ) == 0
    {
        dwError = GetLastError();
        if dwError != ERROR_INSUFFICIENT_BUFFER {
            ERR!("SetupDiGetClassInstallParams failed (Error %lu)\n", dwError);
            return (FALSE) as BOOL;
        }
    }

    /* Allocate a buffer for the old class install parameters */
    pClassInstallParams = HeapAlloc(
        GetProcessHeap(),
        0,
        dwSize as SIZE_T,
    ) as PSP_CLASSINSTALL_HEADER;
    if pClassInstallParams.is_null() {
        ERR!("Failed to allocate the parameters buffer!\n");
        return (FALSE) as BOOL;
    }

    'done: {
        /* Save the old class install parameters */
        if SetupDiGetClassInstallParams(
            DeviceInfoSet,
            DeviceInfoData,
            pClassInstallParams,
            dwSize,
            core::ptr::addr_of_mut!(dwSize),
        ) == 0
        {
            ERR!(
                "SetupDiGetClassInstallParams failed (Error %lu)\n",
                GetLastError()
            );
            ret = (FALSE as i32);
            break 'done;
        }

        /* Set the new class install parameters */
        if SetupDiSetClassInstallParams(
            DeviceInfoSet,
            DeviceInfoData,
            core::ptr::addr_of_mut!((*PageData).ClassInstallHeader),
            core::mem::size_of::<SP_ADDPROPERTYPAGE_DATA>() as DWORD,
        ) == 0
        {
            ERR!("SetupDiSetClassInstallParams failed (Error %lu)\n", dwError);
            ret = (FALSE as i32);
            break 'done;
        }

        /* Call the installer */
        ret = SetupDiCallClassInstaller(InstallFunction, DeviceInfoSet, DeviceInfoData);
        if ret == (FALSE as BOOL) {
            ERR!("SetupDiCallClassInstaller failed\n");
            break 'done;
        }

        /* Read the new class installer parameters */
        if SetupDiGetClassInstallParams(
            DeviceInfoSet,
            DeviceInfoData,
            core::ptr::addr_of_mut!((*PageData).ClassInstallHeader),
            core::mem::size_of::<SP_ADDPROPERTYPAGE_DATA>() as DWORD,
            core::ptr::null_mut(),
        ) == 0
        {
            ERR!(
                "SetupDiGetClassInstallParams failed (Error %lu)\n",
                GetLastError()
            );
            ret = (FALSE as i32);
            break 'done;
        }
    }

    /* Restore and free the old class install parameters */
    if !pClassInstallParams.is_null() {
        SetupDiSetClassInstallParams(DeviceInfoSet, DeviceInfoData, pClassInstallParams, dwSize);

        HeapFree(
            GetProcessHeap(),
            0,
            pClassInstallParams as PVOID,
        );
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiGetClassDevPropertySheetsW(
    mut DeviceInfoSet: HDEVINFO,
    mut DeviceInfoData: PSP_DEVINFO_DATA,
    mut PropertySheetHeader: LPPROPSHEETHEADERW,
    mut PropertySheetHeaderPageListSize: DWORD,
    mut RequiredSize: PDWORD,
    mut PropertySheetType: DWORD,
) -> BOOL {
    let mut devInfoSet: *mut DeviceInfoSet = ::core::ptr::null_mut::<DeviceInfoSet>();
    let mut devInfo: *mut DeviceInfo = ::core::ptr::null_mut::<DeviceInfo>();
    let mut ret: BOOL = FALSE as BOOL;
    if DeviceInfoSet.is_null() {
        SetLastError(ERROR_INVALID_HANDLE as DWORD);
    } else if (*(DeviceInfoSet as *mut DeviceInfoSet)).magic != SETUP_DEVICE_INFO_SET_MAGIC as DWORD {
        SetLastError(ERROR_INVALID_HANDLE as DWORD);
    } else {
        devInfoSet = DeviceInfoSet as *mut DeviceInfoSet;
        if (*devInfoSet).magic != SETUP_DEVICE_INFO_SET_MAGIC as DWORD {
            SetLastError(ERROR_INVALID_HANDLE as DWORD);
        } else if PropertySheetHeader.is_null() {
            SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        } else if ((*PropertySheetHeader).dwFlags & PSH_PROPSHEETPAGE as DWORD) != 0 {
            SetLastError(ERROR_INVALID_FLAGS as DWORD);
        } else if !DeviceInfoData.is_null()
            && (*DeviceInfoData).cbSize != core::mem::size_of::<SP_DEVINFO_DATA>() as DWORD
        {
            SetLastError(ERROR_INVALID_USER_BUFFER as DWORD);
        } else if DeviceInfoData.is_null()
            && IsEqualIID(
                &raw mut (*devInfoSet).ClassGuid,
                &raw const GUID_NULL,
            )
        {
            SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        } else if PropertySheetType != DIGCDP_FLAG_ADVANCED as DWORD
            && PropertySheetType != DIGCDP_FLAG_BASIC as DWORD
            && PropertySheetType != DIGCDP_FLAG_REMOTE_ADVANCED as DWORD
            && PropertySheetType != DIGCDP_FLAG_REMOTE_BASIC as DWORD
        {
            SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        } else {
            let mut hKey: HKEY = INVALID_HANDLE_VALUE as HKEY;
            let mut Request: SP_PROPSHEETPAGE_REQUEST = _SP_PROPSHEETPAGE_REQUEST {
                cbSize: 0,
                PageRequested: 0,
                DeviceInfoSet: ::core::ptr::null_mut::<::core::ffi::c_void>(),
                DeviceInfoData: ::core::ptr::null_mut::<_SP_DEVINFO_DATA>(),
            };
            let mut PropPageProvider: LPWSTR = ::core::ptr::null_mut::<WCHAR>();
            let mut hModule: HMODULE = ::core::ptr::null_mut::<HINSTANCE__>();
            let mut pPropPageProvider: PROPERTY_PAGE_PROVIDER = None;
            let mut PropPageData: ClassDevPropertySheetsData = ClassDevPropertySheetsData {
                PropertySheetHeader: ::core::ptr::null_mut::<_PROPSHEETHEADERW>(),
                PropertySheetHeaderPageListSize: 0,
                NumberOfPages: 0,
                DontCancel: 0,
            };
            let mut InstallerPropPageData: SP_ADDPROPERTYPAGE_DATA = _SP_NEWDEVICEWIZARD_DATA {
                ClassInstallHeader: _SP_CLASSINSTALL_HEADER {
                    cbSize: 0,
                    InstallFunction: 0,
                },
                Flags: 0,
                DynamicPages: [::core::ptr::null_mut::<HPROPSHEETPAGE__>(); 20],
                NumDynamicPages: 0,
                hwndWizardDlg: ::core::ptr::null_mut::<HWND__>(),
            };
            let mut InitialNumberOfPages: DWORD = 0;
            let mut i: DWORD = 0;
            let mut rc: DWORD = 0;

            'cleanup: {
                if !DeviceInfoData.is_null() {
                    devInfo = (*DeviceInfoData).Reserved as *mut DeviceInfo;
                }

                /* Get the class property page provider */
                if (*devInfoSet).hmodClassPropPageProvider.is_null() {
                    hKey = SetupDiOpenClassRegKeyExW(
                        if !devInfo.is_null() {
                            &raw mut (*devInfo).ClassGuid
                        } else {
                            &raw mut (*devInfoSet).ClassGuid
                        },
                        KEY_QUERY_VALUE as REGSAM,
                        DIOCR_INSTALLER as DWORD,
                        ::core::ptr::null::<WCHAR>(),
                        core::ptr::null_mut(),
                    );
                    if hKey != INVALID_HANDLE_VALUE as HKEY {
                        rc = SETUP_GetValueString(
                            hKey,
                            REGSTR_VAL_ENUMPROPPAGES_32 as LPWSTR,
                            &raw mut PropPageProvider,
                        );
                        if rc == ERROR_SUCCESS as DWORD {
                            rc = GetFunctionPointer(
                                PropPageProvider,
                                &raw mut hModule,
                                &raw mut pPropPageProvider as *mut PVOID,
                            );
                            if rc != ERROR_SUCCESS as DWORD {
                                SetLastError(ERROR_INVALID_PROPPAGE_PROVIDER as DWORD);
                                break 'cleanup;
                            }

                            (*devInfoSet).hmodClassPropPageProvider = hModule;
                            (*devInfoSet).pClassPropPageProvider =
                                ::core::mem::transmute::<PROPERTY_PAGE_PROVIDER, PVOID>(
                                    pPropPageProvider,
                                );

                            HeapFree(
                                GetProcessHeap(),
                                0 as DWORD,
                                PropPageProvider as PVOID,
                            );
                            PropPageProvider = ::core::ptr::null_mut::<WCHAR>();
                        }

                        RegCloseKey(hKey);
                        hKey = INVALID_HANDLE_VALUE as HKEY;
                    }
                }

                /* Get the device property page provider */
                if !devInfo.is_null() && (*devInfo).hmodDevicePropPageProvider.is_null() {
                    hKey = SETUPDI_OpenDrvKey(
                        (*devInfoSet).HKLM,
                        devInfo,
                        KEY_QUERY_VALUE as REGSAM,
                    );
                    if hKey != INVALID_HANDLE_VALUE as HKEY {
                        rc = SETUP_GetValueString(
                            hKey,
                            REGSTR_VAL_ENUMPROPPAGES_32 as LPWSTR,
                            &raw mut PropPageProvider,
                        );
                        if rc == ERROR_SUCCESS as DWORD {
                            rc = GetFunctionPointer(
                                PropPageProvider,
                                &raw mut hModule,
                                &raw mut pPropPageProvider as *mut PVOID,
                            );
                            if rc != ERROR_SUCCESS as DWORD {
                                SetLastError(ERROR_INVALID_PROPPAGE_PROVIDER as DWORD);
                                break 'cleanup;
                            }

                            (*devInfo).hmodDevicePropPageProvider = hModule;
                            (*devInfo).pDevicePropPageProvider =
                                ::core::mem::transmute::<PROPERTY_PAGE_PROVIDER, PVOID>(
                                    pPropPageProvider,
                                );

                            HeapFree(
                                GetProcessHeap(),
                                0 as DWORD,
                                PropPageProvider as PVOID,
                            );
                            PropPageProvider = ::core::ptr::null_mut::<WCHAR>();
                        }

                        RegCloseKey(hKey);
                        hKey = INVALID_HANDLE_VALUE as HKEY;
                    }
                }

                InitialNumberOfPages = (*PropertySheetHeader).nPages as DWORD;

                Request.cbSize = core::mem::size_of::<SP_PROPSHEETPAGE_REQUEST>() as DWORD;
                Request.PageRequested = SPPSR_ENUM_ADV_DEVICE_PROPERTIES as DWORD;
                Request.DeviceInfoSet = DeviceInfoSet;
                Request.DeviceInfoData = DeviceInfoData;

                PropPageData.PropertySheetHeader = PropertySheetHeader;
                PropPageData.PropertySheetHeaderPageListSize = PropertySheetHeaderPageListSize;
                PropPageData.NumberOfPages = 0 as DWORD;
                PropPageData.DontCancel = (if !RequiredSize.is_null() {
                    TRUE
                } else {
                    FALSE
                }) as BOOL;

                /* Call the class property page provider */
                if !(*devInfoSet).pClassPropPageProvider.is_null() {
                    ::core::mem::transmute::<PVOID, PROPERTY_PAGE_PROVIDER>(
                        (*devInfoSet).pClassPropPageProvider,
                    )
                    .unwrap_unchecked()(
                        &raw mut Request,
                        Some(
                            SETUP_GetClassDevPropertySheetsCallback,
                        ),
                        &raw mut PropPageData as LPARAM,
                    );
                }

                /* Call the device property page provider */
                if !devInfo.is_null() && !(*devInfo).pDevicePropPageProvider.is_null() {
                    ::core::mem::transmute::<PVOID, PROPERTY_PAGE_PROVIDER>(
                        (*devInfo).pDevicePropPageProvider,
                    )
                    .unwrap_unchecked()(
                        &raw mut Request,
                        Some(
                            SETUP_GetClassDevPropertySheetsCallback,
                        ),
                        &raw mut PropPageData as LPARAM,
                    );
                }

                /* Call the class installer and add the returned pages */
                ZeroMemory!(
                    &raw mut InstallerPropPageData as *mut ::core::ffi::c_void,
                    core::mem::size_of::<SP_ADDPROPERTYPAGE_DATA>() as size_t,
                );
                InstallerPropPageData.ClassInstallHeader.cbSize =
                    core::mem::size_of::<SP_CLASSINSTALL_HEADER>() as DWORD;
                InstallerPropPageData.ClassInstallHeader.InstallFunction =
                    DIF_ADDPROPERTYPAGE_ADVANCED as DI_FUNCTION;
                InstallerPropPageData.hwndWizardDlg = (*PropertySheetHeader).hwndParent;

                if SETUP_CallInstaller(
                    DIF_ADDPROPERTYPAGE_ADVANCED as DI_FUNCTION,
                    DeviceInfoSet,
                    DeviceInfoData,
                    &raw mut InstallerPropPageData,
                ) != 0
                {
                    i = 0 as DWORD;
                    while i < InstallerPropPageData.NumDynamicPages {
                        if ((*PropPageData.PropertySheetHeader).nPages as DWORD)
                            < PropertySheetHeaderPageListSize
                        {
                            *(*PropPageData.PropertySheetHeader)

                                .__bindgen_anon_3.phpage
                                .offset((*PropPageData.PropertySheetHeader).nPages as isize) =
                                InstallerPropPageData.DynamicPages[i as usize];
                            (*PropPageData.PropertySheetHeader).nPages =
                                (*PropPageData.PropertySheetHeader).nPages.wrapping_add(1);
                        }
                        i = i.wrapping_add(1);
                    }

                    PropPageData.NumberOfPages = PropPageData
                        .NumberOfPages
                        .wrapping_add(InstallerPropPageData.NumDynamicPages);
                }

                if !RequiredSize.is_null() {
                    *RequiredSize = PropPageData.NumberOfPages;
                }

                if InitialNumberOfPages.wrapping_add(PropPageData.NumberOfPages)
                    <= PropertySheetHeaderPageListSize
                {
                    ret = TRUE as BOOL;
                } else {
                    SetLastError(ERROR_INSUFFICIENT_BUFFER as DWORD);
                }
            }

            /* cleanup: */
            if hKey != INVALID_HANDLE_VALUE as HKEY {
                RegCloseKey(hKey);
            }

            if !PropPageProvider.is_null() {
                HeapFree(
                    GetProcessHeap(),
                    0 as DWORD,
                    PropPageProvider as PVOID,
                );
            }
        }
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiGetClassRegistryPropertyA(
    ClassGuid: *const GUID,
    Property: DWORD,
    PropertyRegDataType: PDWORD,
    PropertyBuffer: PBYTE,
    PropertyBufferSize: DWORD,
    RequiredSize: PDWORD,
    MachineName: PCSTR,
    Reserved: PVOID,
) -> BOOL {
    let mut hMachine: HMACHINE = core::ptr::null_mut();
    let mut PropLength: DWORD = 0;
    let mut Error: DWORD = ERROR_SUCCESS;
    let mut cr: CONFIGRET;

    if (*__wine_dbch___default).flags as i32 & ((1i32) << (__WINE_DBCL_TRACE as i32)) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/setupapi/devclass.c\0"
                as *const u8 as *const i8,
            b"SetupDiGetClassRegistryPropertyA\0" as *const u8 as *const i8,
            1444 as i32,
            b"%s %lu %p %p %lu %p %s %p\n\0" as *const u8 as *const i8,
            debugstr_guid(ClassGuid as *const _GUID),
            Property,
            PropertyRegDataType,
            PropertyBuffer,
            PropertyBufferSize,
            RequiredSize,
            MachineName,
            Reserved,
        );
    }

    if !Reserved.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return (FALSE) as BOOL;
    }

    'done: {
        if !MachineName.is_null() {
            cr = CM_Connect_MachineA(MachineName, core::ptr::addr_of_mut!(hMachine));
            if cr != CR_SUCCESS {
                break 'done;
            }
        }

        if Property >= SPCRP_MAXIMUM_PROPERTY {
            cr = CR_INVALID_PROPERTY;
            break 'done;
        }

        PropLength = PropertyBufferSize;
        cr = CM_Get_Class_Registry_PropertyA(
            ClassGuid as LPGUID,
            Property.wrapping_add((CM_DRP_DEVICEDESC - SPDRP_DEVICEDESC) as DWORD),
            PropertyRegDataType,
            (PropertyBuffer).cast(),
            core::ptr::addr_of_mut!(PropLength),
            0,
            hMachine,
        );
        if cr == CR_SUCCESS || cr == CR_BUFFER_SMALL {
            if !RequiredSize.is_null() {
                *RequiredSize = PropLength;
            }
        }
    }

    // done:
    if cr != CR_SUCCESS {
        if cr == CR_INVALID_DEVINST {
            Error = ERROR_NO_SUCH_DEVINST;
        } else if cr == CR_INVALID_PROPERTY {
            Error = ERROR_INVALID_REG_PROPERTY;
        } else if cr == CR_BUFFER_SMALL {
            Error = ERROR_INSUFFICIENT_BUFFER;
        } else {
            Error = GetErrorCodeFromCrCode(cr);
        }
    }

    if !hMachine.is_null() {
        CM_Disconnect_Machine(hMachine);
    }

    SetLastError(Error);
    return (cr == CR_SUCCESS) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiGetClassRegistryPropertyW(
    ClassGuid: *const GUID,
    Property: DWORD,
    PropertyRegDataType: PDWORD,
    PropertyBuffer: PBYTE,
    PropertyBufferSize: DWORD,
    RequiredSize: PDWORD,
    MachineName: PCWSTR,
    Reserved: PVOID,
) -> BOOL {
    let mut hMachine: HMACHINE = core::ptr::null_mut();
    let mut PropLength: DWORD = 0;
    let mut Error: DWORD = ERROR_SUCCESS;
    let mut cr: CONFIGRET = 0;

    TRACE!(
        "%s %lu %p %p %lu %p %s %p\n",
        debugstr_guid(ClassGuid as *const _GUID),
        Property,
        PropertyRegDataType,
        PropertyBuffer,
        PropertyBufferSize,
        RequiredSize,
        debugstr_w(MachineName as *const WCHAR),
        Reserved
    );

    if !Reserved.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return (FALSE) as BOOL;
    }

    'done: {
        if !MachineName.is_null() {
            cr = CM_Connect_MachineW(MachineName, core::ptr::addr_of_mut!(hMachine));
            if cr != CR_SUCCESS {
                break 'done;
            }
        }

        if Property >= SPCRP_MAXIMUM_PROPERTY {
            cr = CR_INVALID_PROPERTY;
            break 'done;
        }

        PropLength = PropertyBufferSize;
        cr = CM_Get_Class_Registry_PropertyW(
            ClassGuid as LPGUID,
            Property.wrapping_add(CM_DRP_DEVICEDESC - SPDRP_DEVICEDESC),
            PropertyRegDataType,
            (PropertyBuffer).cast(),
            core::ptr::addr_of_mut!(PropLength),
            0,
            hMachine,
        );
        if (cr == CR_SUCCESS) || (cr == CR_BUFFER_SMALL) {
            if !RequiredSize.is_null() {
                *RequiredSize = PropLength;
            }
        }
    }

    if cr != CR_SUCCESS {
        if cr == CR_INVALID_DEVINST {
            Error = ERROR_NO_SUCH_DEVINST;
        } else if cr == CR_INVALID_PROPERTY {
            Error = ERROR_INVALID_REG_PROPERTY;
        } else if cr == CR_BUFFER_SMALL {
            Error = ERROR_INSUFFICIENT_BUFFER;
        } else {
            Error = GetErrorCodeFromCrCode(cr);
        }
    }

    if !hMachine.is_null() {
        CM_Disconnect_Machine(hMachine);
    }

    SetLastError(Error);
    return (cr == CR_SUCCESS) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiSetClassRegistryPropertyA(
    ClassGuid: *const GUID,
    Property: DWORD,
    PropertyBuffer: *const BYTE,
    PropertyBufferSize: DWORD,
    MachineName: PCSTR,
    Reserved: PVOID,
) -> BOOL {
    let mut hMachine: HMACHINE = core::ptr::null_mut();
    let mut Error: DWORD = ERROR_SUCCESS;
    let mut cr: CONFIGRET = 0;

    if !Reserved.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return (FALSE) as BOOL;
    }

    'done: {
        if !MachineName.is_null() {
            cr = CM_Connect_MachineA(MachineName, core::ptr::addr_of_mut!(hMachine));
            if cr != CR_SUCCESS {
                break 'done;
            }
        }

        if Property >= SPCRP_MAXIMUM_PROPERTY {
            cr = CR_INVALID_PROPERTY;
            break 'done;
        }

        cr = CM_Set_Class_Registry_PropertyA(
            ClassGuid as LPGUID,
            (Property as ULONG).wrapping_add((CM_DRP_DEVICEDESC - SPDRP_DEVICEDESC) as ULONG),
            PropertyBuffer as PCVOID,
            PropertyBufferSize as ULONG,
            0,
            hMachine,
        );
    }

    if cr != CR_SUCCESS {
        if cr == CR_INVALID_DEVINST {
            Error = ERROR_NO_SUCH_DEVINST;
        } else if cr == CR_INVALID_PROPERTY {
            Error = ERROR_INVALID_REG_PROPERTY;
        } else if cr == CR_BUFFER_SMALL {
            Error = ERROR_INSUFFICIENT_BUFFER;
        } else {
            Error = GetErrorCodeFromCrCode(cr);
        }
    }

    if !hMachine.is_null() {
        CM_Disconnect_Machine(hMachine);
    }

    SetLastError(Error);
    return (cr == CR_SUCCESS) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiSetClassRegistryPropertyW(
    ClassGuid: *const GUID,
    Property: DWORD,
    PropertyBuffer: *const BYTE,
    PropertyBufferSize: DWORD,
    MachineName: PCWSTR,
    Reserved: PVOID,
) -> BOOL {
    let mut hMachine: HMACHINE = core::ptr::null_mut();
    let mut Error: DWORD = ERROR_SUCCESS;
    let mut cr: CONFIGRET;

    if !Reserved.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return (FALSE) as BOOL;
    }

    'done: {
        if !MachineName.is_null() {
            cr = CM_Connect_MachineW(MachineName, core::ptr::addr_of_mut!(hMachine));
            if cr != CR_SUCCESS {
                break 'done;
            }
        }

        if Property >= SPCRP_MAXIMUM_PROPERTY {
            cr = CR_INVALID_PROPERTY;
            break 'done;
        }

        cr = CM_Set_Class_Registry_PropertyW(
            ClassGuid as LPGUID,
            Property.wrapping_add((CM_DRP_DEVICEDESC - SPDRP_DEVICEDESC) as DWORD),
            (PropertyBuffer).cast(),
            PropertyBufferSize,
            0,
            hMachine,
        );
    }

    // done:
    if cr != CR_SUCCESS {
        match cr {
            CR_INVALID_DEVINST => {
                Error = ERROR_NO_SUCH_DEVINST;
            }
            CR_INVALID_PROPERTY => {
                Error = ERROR_INVALID_REG_PROPERTY;
            }
            CR_BUFFER_SMALL => {
                Error = ERROR_INSUFFICIENT_BUFFER;
            }
            _ => {
                Error = GetErrorCodeFromCrCode(cr);
            }
        }
    }

    if !hMachine.is_null() {
        CM_Disconnect_Machine(hMachine);
    }

    SetLastError(Error);
    return (cr == CR_SUCCESS) as BOOL;
}
