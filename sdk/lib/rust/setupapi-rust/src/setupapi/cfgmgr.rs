// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
use ntdll_sys::seh::rpc_try;
pub static RUSTCONV_WSTR_00A6C7A05271: [u16; 18] = [
    0x0044u16, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x0020, 0x0050, 0x0061, 0x0072,
    0x0061, 0x006D, 0x0065, 0x0074, 0x0065, 0x0072, 0x0073, 0x0000,
];
pub static RUSTCONV_WSTR_036DDE007C31: [u16; 37] = [
    0x0047u16, 0x006C, 0x006F, 0x0062, 0x0061, 0x006C, 0x005C, 0x0050, 0x006E, 0x0050,
    0x005F, 0x004E, 0x006F, 0x005F, 0x0050, 0x0065, 0x006E, 0x0064, 0x0069, 0x006E,
    0x0067, 0x005F, 0x0049, 0x006E, 0x0073, 0x0074, 0x0061, 0x006C, 0x006C, 0x005F,
    0x0045, 0x0076, 0x0065, 0x006E, 0x0074, 0x0073, 0x0000,
];
pub static RUSTCONV_WSTR_05E2B39CEDA8: [u16; 8] = [0x0043u16, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0000];
pub static RUSTCONV_WSTR_5EA47B250AB0: [u16; 43] = [
    0x0053u16, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x005C, 0x0043, 0x0075, 0x0072,
    0x0072, 0x0065, 0x006E, 0x0074, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F,
    0x006C, 0x0053, 0x0065, 0x0074, 0x005C, 0x0048, 0x0061, 0x0072, 0x0064, 0x0077,
    0x0061, 0x0072, 0x0065, 0x0020, 0x0050, 0x0072, 0x006F, 0x0066, 0x0069, 0x006C,
    0x0065, 0x0073, 0x0000,
];
pub static RUSTCONV_WSTR_9C023F1B5915: [u16; 15] = [
    0x0025u16, 0x0073, 0x005C, 0x0025, 0x0030, 0x0034, 0x006C, 0x0075, 0x005C, 0x0025,
    0x0073, 0x005C, 0x0025, 0x0073, 0x0000,
];
pub static RUSTCONV_WSTR_B7FC02907B1F: [u16; 2] = [0x005Cu16, 0x0000];
pub static RUSTCONV_WSTR_BA2342D760A2: [u16; 6] = [0x0025u16, 0x0073, 0x005C, 0x0025, 0x0073, 0x0000];
pub static RUSTCONV_WSTR_BE6FA5D32001: [u16; 30] = [
    0x0053u16, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x005C, 0x0043, 0x0075, 0x0072,
    0x0072, 0x0065, 0x006E, 0x0074, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F,
    0x006C, 0x0053, 0x0065, 0x0074, 0x005C, 0x0045, 0x006E, 0x0075, 0x006D, 0x0000,
];
pub static RUSTCONV_WSTR_D28B95E7C0ED: [u16; 12] = [
    0x0025u16, 0x0073, 0x005C, 0x0025, 0x0073, 0x005C, 0x0025, 0x0073, 0x005C, 0x0025,
    0x0073, 0x0000,
];
pub static RUSTCONV_WSTR_E4F2BEC36683: [u16; 4] = [0x0050u16, 0x004C, 0x0054, 0x0000];
pub static RUSTCONV_WSTR_FC2D9680B5FB: [u16; 39] = [
    0x0053u16, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x005C, 0x0043, 0x0075, 0x0072,
    0x0072, 0x0065, 0x006E, 0x0074, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F,
    0x006C, 0x0053, 0x0065, 0x0074, 0x005C, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072,
    0x006F, 0x006C, 0x005C, 0x0043, 0x006C, 0x0061, 0x0073, 0x0073, 0x0000,
];

use core::ffi::c_void;

extern "C" {
    fn ros_dbg_log( cls: __wine_debug_class, ch: *mut __wine_debug_channel, file: *const core::ffi::c_char, func: *const core::ffi::c_char, line: core::ffi::c_int, fmt: *const core::ffi::c_char, ... );
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut BackslashOpenBrace: [WCHAR; 3] = [
    '\\' as i32 as WCHAR,
    '{' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];
static mut CloseBrace: [WCHAR; 2] = ['}' as i32 as WCHAR, 0 as ::core::ffi::c_int as WCHAR];

static mut Class: [WCHAR; 6] = [
    'C' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    's' as i32 as WCHAR,
    's' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut ControlClass: [WCHAR; 39] = [
    'S' as i32 as WCHAR,
    'y' as i32 as WCHAR,
    's' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'm' as i32 as WCHAR,
    '\\' as i32 as WCHAR,
    'C' as i32 as WCHAR,
    'u' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'C' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    'S' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    't' as i32 as WCHAR,
    '\\' as i32 as WCHAR,
    'C' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    '\\' as i32 as WCHAR,
    'C' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    's' as i32 as WCHAR,
    's' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut DeviceClasses: [WCHAR; 47] = [
    'S' as i32 as WCHAR,
    'y' as i32 as WCHAR,
    's' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'm' as i32 as WCHAR,
    '\\' as i32 as WCHAR,
    'C' as i32 as WCHAR,
    'u' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'C' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    'S' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    't' as i32 as WCHAR,
    '\\' as i32 as WCHAR,
    'C' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    '\\' as i32 as WCHAR,
    'D' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'v' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'c' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'C' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    's' as i32 as WCHAR,
    's' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    's' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];


#[repr(C)]
#[allow(non_camel_case_types, non_snake_case)]
pub struct _MACHINE_INFO {
    pub szMachineName: [WCHAR; (SP_MAX_MACHINENAME_LENGTH) as _],
    pub BindingHandle: RPC_BINDING_HANDLE,
    pub StringTable: HSTRING_TABLE,
    pub bLocal: BOOL,
}
pub type MACHINE_INFO = _MACHINE_INFO;
pub type PMACHINE_INFO = *mut _MACHINE_INFO;

#[repr(C)]
#[allow(non_camel_case_types, non_snake_case)]
pub struct _LOG_CONF_INFO {
    pub ulMagic: ULONG,
    pub dnDevInst: DEVINST,
    pub ulType: ULONG,
    pub ulTag: ULONG,
}
pub type LOG_CONF_INFO = _LOG_CONF_INFO;
pub type PLOG_CONF_INFO = *mut _LOG_CONF_INFO;

pub const LOG_CONF_MAGIC: u32 = 0x464E434C; // "LCNF"

#[repr(C)]
#[allow(non_camel_case_types, non_snake_case)]
pub struct _RES_DES_INFO {
    pub ulMagic: ULONG,
    pub dnDevInst: DEVINST,
    pub ulLogConfType: ULONG,
    pub ulLogConfTag: ULONG,
    pub ulResDesType: ULONG,
    pub ulResDesTag: ULONG,
}
pub type RES_DES_INFO = _RES_DES_INFO;
pub type PRES_DES_INFO = *mut _RES_DES_INFO;

pub const RES_DES_MAGIC: u32 = 0x53445352; // "RSDS"

#[repr(C)]
#[allow(non_camel_case_types, non_snake_case)]
pub struct _NOTIFY_DATA {
    pub ulMagic: ULONG,
    pub hNotifyHandle: PVOID,
}
pub type NOTIFY_DATA = _NOTIFY_DATA;
pub type PNOTIFY_DATA = *mut _NOTIFY_DATA;

pub const NOTIFY_MAGIC: u32 = 0x44556677;

#[repr(C)]
#[allow(non_camel_case_types, non_snake_case)]
pub struct _INTERNAL_RANGE {
    pub ListEntry: _LIST_ENTRY,
    pub pRangeList: *mut _INTERNAL_RANGE_LIST,
    pub ullStart: DWORDLONG,
    pub ullEnd: DWORDLONG,
}
pub type INTERNAL_RANGE = _INTERNAL_RANGE;
pub type PINTERNAL_RANGE = *mut _INTERNAL_RANGE;

#[repr(C)]
#[allow(non_camel_case_types, non_snake_case)]
pub struct _INTERNAL_RANGE_LIST {
    pub ulMagic: ULONG,
    pub hMutex: HANDLE,
    pub ListHead: _LIST_ENTRY,
}
pub type INTERNAL_RANGE_LIST = _INTERNAL_RANGE_LIST;
pub type PINTERNAL_RANGE_LIST = *mut _INTERNAL_RANGE_LIST;

pub const RANGE_LIST_MAGIC: u32 = 0x33445566;

#[repr(C)]
#[allow(non_camel_case_types, non_snake_case)]
pub struct _CONFLICT_DATA {
    pub ulMagic: ULONG,
    pub pConflictList: PPNP_CONFLICT_LIST,
}
pub type CONFLICT_DATA = _CONFLICT_DATA;
pub type PCONFLICT_DATA = *mut _CONFLICT_DATA;

pub const CONFLICT_MAGIC: u32 = 0x11225588;

const __WINE_DBCL_TRACE: u32 = 3;
unsafe extern "C" fn GuidToString(Guid: LPGUID, String: LPWSTR) -> BOOL {
    let mut lpString: LPWSTR = core::ptr::null_mut();
    if UuidToStringW(Guid as *mut UUID, core::ptr::addr_of_mut!(lpString)) != RPC_S_OK {
        return (FALSE) as BOOL;
    }

    lstrcpyW(String.offset(1 as isize) as LPWSTR, lpString as LPCWSTR);

    *String.offset(0 as isize) = '{' as WCHAR;
    *String.offset((MAX_GUID_STRING_LEN - 2) as isize) = '}' as WCHAR;
    *String.offset((MAX_GUID_STRING_LEN - 1) as isize) = UNICODE_NULL;

    RpcStringFreeW(core::ptr::addr_of_mut!(lpString));

    return (TRUE) as BOOL;
}

unsafe extern "C" fn RpcStatusToCmStatus(Status: RPC_STATUS) -> CONFIGRET {
    CR_FAILURE
}

unsafe extern "C" fn GetRegistryPropertyType(mut ulProperty: ULONG) -> ULONG {
    if ulProperty == CM_DRP_DEVICEDESC
        || ulProperty == CM_DRP_SERVICE
        || ulProperty == CM_DRP_CLASS
        || ulProperty == CM_DRP_CLASSGUID
        || ulProperty == CM_DRP_DRIVER
        || ulProperty == CM_DRP_MFG
        || ulProperty == CM_DRP_FRIENDLYNAME
        || ulProperty == CM_DRP_LOCATION_INFORMATION
        || ulProperty == CM_DRP_PHYSICAL_DEVICE_OBJECT_NAME
        || ulProperty == CM_DRP_ENUMERATOR_NAME
        || ulProperty == CM_DRP_SECURITY_SDS
        || ulProperty == CM_DRP_UI_NUMBER_DESC_FORMAT
    {
        return REG_SZ;
    } else if ulProperty == CM_DRP_HARDWAREID
        || ulProperty == CM_DRP_COMPATIBLEIDS
        || ulProperty == CM_DRP_UPPERFILTERS
        || ulProperty == CM_DRP_LOWERFILTERS
    {
        return REG_MULTI_SZ;
    } else if ulProperty == CM_DRP_CONFIGFLAGS
        || ulProperty == CM_DRP_CAPABILITIES
        || ulProperty == CM_DRP_UI_NUMBER
        || ulProperty == CM_DRP_LEGACYBUSTYPE
        || ulProperty == CM_DRP_BUSNUMBER
        || ulProperty == CM_DRP_DEVTYPE
        || ulProperty == CM_DRP_EXCLUSIVE
        || ulProperty == CM_DRP_CHARACTERISTICS
        || ulProperty == CM_DRP_ADDRESS
        || ulProperty == CM_DRP_REMOVAL_POLICY
        || ulProperty == CM_DRP_REMOVAL_POLICY_HW_DEFAULT
        || ulProperty == CM_DRP_REMOVAL_POLICY_OVERRIDE
        || ulProperty == CM_DRP_INSTALL_STATE
    {
        return REG_DWORD;
    } else if ulProperty == CM_DRP_BUSTYPEGUID
        || ulProperty == CM_DRP_SECURITY
        || ulProperty == CM_DRP_DEVICE_POWER_DATA
    {
        return REG_BINARY;
    } else {
        return REG_BINARY;
    }

    return REG_NONE;
}

unsafe extern "C" fn SplitDeviceInstanceId(
    pszDeviceInstanceId: PWSTR,
    pszDeviceId: PWSTR,
    pszInstanceId: PWSTR,
) {
    let mut ptr: PWCHAR = core::ptr::null_mut();

    wcscpy(pszDeviceId, pszDeviceInstanceId);

    ptr = wcschr(pszDeviceId, 0x005Cu16) as PWCHAR;
    if !ptr.is_null() {
        *ptr = UNICODE_NULL;
        ptr = ptr.add(1);

        wcscpy(pszInstanceId, ptr);
    } else {
        *pszInstanceId = UNICODE_NULL;
    }
}

unsafe extern "C" fn GetDeviceInstanceKeyPath(
    BindingHandle: RPC_BINDING_HANDLE,
    pszDeviceInst: PWSTR,
    pszKeyPath: PWSTR,
    pszInstancePath: PWSTR,
    ulHardwareProfile: ULONG,
    ulFlags: ULONG,
) -> CONFIGRET {
    let mut pszBuffer: PWSTR = core::ptr::null_mut();
    let mut ulType: ULONG = 0;
    let mut ulTransferLength: ULONG = 0;
    let mut ulLength: ULONG = 0;
    let mut ret: CONFIGRET = CR_SUCCESS;

    'done: {
        /* Allocate a buffer for the device id */
        pszBuffer = MyMalloc(
            (300 as usize * core::mem::size_of::<WCHAR>()) as DWORD,
        ) as PWSTR;
        if pszBuffer.is_null() {
            ERR!("MyMalloc() failed\n");
            return CR_OUT_OF_MEMORY;
        }

        if (ulFlags & CM_REGISTRY_SOFTWARE) != 0 {
            /* Software Key Path */

            ulTransferLength =
                (300 as usize * core::mem::size_of::<WCHAR>()) as ULONG;
            ulLength = (300 as usize * core::mem::size_of::<WCHAR>()) as ULONG;

            ret = rpc_try(|| PNP_GetDeviceRegProp(
                BindingHandle,
                pszDeviceInst as LPWSTR,
                CM_DRP_DRIVER,
                core::ptr::addr_of_mut!(ulType),
                pszBuffer as PVOID as *mut BYTE,
                core::ptr::addr_of_mut!(ulTransferLength),
                core::ptr::addr_of_mut!(ulLength),
                0,
            )).unwrap_or_else(|code| RpcStatusToCmStatus(code as _) as _) as CONFIGRET;

            if ret != CR_SUCCESS {
                ret = rpc_try(|| PNP_GetClassInstance(
                    BindingHandle,
                    pszDeviceInst as LPWSTR,
                    pszBuffer as PVOID as LPWSTR,
                    300,
                )).unwrap_or_else(|code| RpcStatusToCmStatus(code as _) as _) as CONFIGRET;

                if ret != CR_SUCCESS {
                    break 'done;
                }
            }

            SplitDeviceInstanceId(pszBuffer, pszBuffer, pszInstancePath);

            if (ulFlags & CM_REGISTRY_CONFIG) != 0 {
                if ulHardwareProfile == 0 {
                    wsprintfW(
                        pszKeyPath,
                        RUSTCONV_WSTR_D28B95E7C0ED.as_ptr(),
                        RUSTCONV_WSTR_5EA47B250AB0.as_ptr(),
                        RUSTCONV_WSTR_05E2B39CEDA8.as_ptr(),
                        RUSTCONV_WSTR_FC2D9680B5FB.as_ptr(),
                        pszBuffer,
                    );
                } else {
                    wsprintfW(
                        pszKeyPath,
                        RUSTCONV_WSTR_9C023F1B5915.as_ptr(),
                        RUSTCONV_WSTR_5EA47B250AB0.as_ptr(),
                        ulHardwareProfile,
                        RUSTCONV_WSTR_FC2D9680B5FB.as_ptr(),
                        pszBuffer,
                    );
                }
            } else {
                wsprintfW(
                    pszKeyPath,
                    RUSTCONV_WSTR_BA2342D760A2.as_ptr(),
                    RUSTCONV_WSTR_FC2D9680B5FB.as_ptr(),
                    pszBuffer,
                );
            }
        } else {
            /* Hardware Key Path */

            if (ulFlags & CM_REGISTRY_CONFIG) != 0 {
                SplitDeviceInstanceId(pszDeviceInst, pszBuffer, pszInstancePath);

                if ulHardwareProfile == 0 {
                    wsprintfW(
                        pszKeyPath,
                        RUSTCONV_WSTR_D28B95E7C0ED.as_ptr(),
                        RUSTCONV_WSTR_5EA47B250AB0.as_ptr(),
                        RUSTCONV_WSTR_05E2B39CEDA8.as_ptr(),
                        RUSTCONV_WSTR_BE6FA5D32001.as_ptr(),
                        pszBuffer,
                    );
                } else {
                    wsprintfW(
                        pszKeyPath,
                        RUSTCONV_WSTR_9C023F1B5915.as_ptr(),
                        RUSTCONV_WSTR_5EA47B250AB0.as_ptr(),
                        ulHardwareProfile,
                        RUSTCONV_WSTR_BE6FA5D32001.as_ptr(),
                        pszBuffer,
                    );
                }
            } else if (ulFlags & CM_REGISTRY_USER) != 0 {
                wsprintfW(
                    pszKeyPath,
                    RUSTCONV_WSTR_BA2342D760A2.as_ptr(),
                    RUSTCONV_WSTR_BE6FA5D32001.as_ptr(),
                    pszDeviceInst,
                );

                wcscpy(pszInstancePath, RUSTCONV_WSTR_00A6C7A05271.as_ptr());
            } else {
                SplitDeviceInstanceId(pszDeviceInst, pszBuffer, pszInstancePath);

                wsprintfW(
                    pszKeyPath,
                    RUSTCONV_WSTR_BA2342D760A2.as_ptr(),
                    RUSTCONV_WSTR_BE6FA5D32001.as_ptr(),
                    pszBuffer,
                );
            }
        }
    }

    // done:
    if !pszBuffer.is_null() {
        MyFree(pszBuffer as PVOID);
    }

    return ret;
}
/// Reads a handle's magic the way the C does, under `_SEH2_TRY` with an
/// `_SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)` that answers "not valid": a stale or garbage
/// CM handle from the caller is rejected rather than faulting the process. DUMMY_PSEH
/// compiled the guard out of the C; rpc_try (ntdll's RtlSehTryExcept) makes it real.
unsafe fn magic_is(magic: *const ULONG, expected: ULONG) -> BOOL {
    match rpc_try(|| core::ptr::read_volatile(magic)) {
        Ok(found) if found == expected => (TRUE as i32),
        _ => (FALSE as i32),
    }
}

#[no_mangle]
pub unsafe extern "C" fn IsValidRangeList(pRangeList: PINTERNAL_RANGE_LIST) -> BOOL {
    if pRangeList.is_null() {
        return (FALSE) as BOOL;
    }
    magic_is(core::ptr::addr_of!((*pRangeList).ulMagic), RANGE_LIST_MAGIC as ULONG)
}

#[no_mangle]
pub unsafe extern "C" fn IsValidLogConf(pLogConfInfo: PLOG_CONF_INFO) -> BOOL {
    if pLogConfInfo.is_null() {
        return (FALSE) as BOOL;
    }
    magic_is(core::ptr::addr_of!((*pLogConfInfo).ulMagic), LOG_CONF_MAGIC as ULONG)
}

#[no_mangle]
pub unsafe extern "C" fn IsValidResDes(pResDesInfo: PRES_DES_INFO) -> BOOL {
    if pResDesInfo.is_null() {
        return (FALSE) as BOOL;
    }
    magic_is(core::ptr::addr_of!((*pResDesInfo).ulMagic), RES_DES_MAGIC as ULONG)
}

#[no_mangle]
pub unsafe extern "C" fn IsValidConflictData(pConflictData: PCONFLICT_DATA) -> BOOL {
    if pConflictData.is_null() {
        return (FALSE) as BOOL;
    }
    magic_is(core::ptr::addr_of!((*pConflictData).ulMagic), CONFLICT_MAGIC as ULONG)
}

#[no_mangle]
pub unsafe extern "system" fn CMP_GetBlockedDriverInfo(
    pszNames: LPWSTR,
    pulLength: PULONG,
    ulFlags: ULONG,
    hMachine: HMACHINE,
) -> CONFIGRET {
    let mut BindingHandle: RPC_BINDING_HANDLE = core::ptr::null_mut();
    let mut ulTransferLength: ULONG = 0;
    let mut ret: CONFIGRET = 0;

    if !hMachine.is_null() {
        BindingHandle = (*(hMachine as PMACHINE_INFO)).BindingHandle;
        if BindingHandle.is_null() {
            return CR_FAILURE;
        }
    } else {
        if PnpGetLocalHandles(&raw mut BindingHandle, core::ptr::null_mut()) == 0 {
            return CR_FAILURE;
        }
    }

    ulTransferLength = *pulLength;

    ret = rpc_try(|| PNP_GetBlockedDriverInfo(
        BindingHandle,
        pszNames as PBYTE,
        &raw mut ulTransferLength,
        pulLength,
        ulFlags,
    )).unwrap_or_else(|code| RpcStatusToCmStatus(code as _) as _);


    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CMP_GetServerSideDeviceInstallFlags(
    mut pulSSDIFlags: PULONG,
    mut ulFlags: ULONG,
    mut hMachine: HMACHINE,
) -> CONFIGRET {
    let mut BindingHandle: RPC_BINDING_HANDLE = core::ptr::null_mut();
    let mut ret: CONFIGRET = 0;

    if pulSSDIFlags.is_null() {
        return CR_INVALID_POINTER;
    }

    if ulFlags != 0 {
        return CR_INVALID_FLAG;
    }

    if !hMachine.is_null() {
        BindingHandle = (*(hMachine as PMACHINE_INFO)).BindingHandle;
        if BindingHandle.is_null() {
            return CR_FAILURE;
        }
    } else {
        if PnpGetLocalHandles(&raw mut BindingHandle, core::ptr::null_mut()) == 0 {
            return CR_FAILURE;
        }
    }

    ret = rpc_try(|| PNP_GetServerSideDeviceInstallFlags(
        BindingHandle as handle_t,
        pulSSDIFlags as *mut DWORD,
        ulFlags as DWORD,
    )).unwrap_or_else(|code| RpcStatusToCmStatus(code as _) as _) as CONFIGRET;


    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CMP_Init_Detection(ulMagic: ULONG) -> CONFIGRET {
    let mut BindingHandle: RPC_BINDING_HANDLE = core::ptr::null_mut();
    let mut ret: CONFIGRET;

    TRACE!("CMP_Init_Detection(%lu)\n", ulMagic);

    if ulMagic != CMP_MAGIC {
        return CR_INVALID_DATA;
    }

    if PnpGetLocalHandles(core::ptr::addr_of_mut!(BindingHandle), core::ptr::null_mut()) == 0 {
        return CR_FAILURE;
    }

    ret = rpc_try(|| PNP_InitDetection(BindingHandle as handle_t)).unwrap_or_else(|code| RpcStatusToCmStatus(code as _) as _) as CONFIGRET;

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CMP_RegisterNotification(
    hRecipient: HANDLE,
    lpvNotificationFilter: LPVOID,
    ulFlags: ULONG,
    phDevNotify: PHDEVNOTIFY,
) -> CONFIGRET {
    let mut BindingHandle: RPC_BINDING_HANDLE = core::ptr::null_mut();
    let mut pNotifyData: PNOTIFY_DATA = core::ptr::null_mut();
    let mut szNameBuffer: [WCHAR; 256] = [0; 256];
    let mut nLength: INT;
    let mut ulUnknown9: DWORD = 0 as DWORD;
    let mut dwError: DWORD;
    let mut ret: CONFIGRET = CR_SUCCESS as CONFIGRET;

    if (hRecipient.is_null()) || (lpvNotificationFilter.is_null()) || (phDevNotify.is_null()) {
        return CR_INVALID_POINTER as CONFIGRET;
    }

    if (ulFlags & !0x7 as ULONG) != 0 {
        return CR_INVALID_FLAG as CONFIGRET;
    }

    if ((*(lpvNotificationFilter as PDEV_BROADCAST_HDR)).dbch_size as usize)
        < core::mem::size_of::<DEV_BROADCAST_HDR>() as usize
    {
        return CR_INVALID_DATA as CONFIGRET;
    }

    if PnpGetLocalHandles(&raw mut BindingHandle, core::ptr::null_mut()) == 0 {
        return CR_FAILURE as CONFIGRET;
    }

    pNotifyData = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY as DWORD,
        core::mem::size_of::<NOTIFY_DATA>() as SIZE_T,
    ) as PNOTIFY_DATA;
    if pNotifyData.is_null() {
        return CR_OUT_OF_MEMORY as CONFIGRET;
    }

    (*pNotifyData).ulMagic = NOTIFY_MAGIC as ULONG;
    (*pNotifyData).hNotifyHandle = core::ptr::null_mut();

    ZeroMemory!(
        szNameBuffer.as_mut_ptr() as *mut core::ffi::c_void,
        core::mem::size_of::<[WCHAR; 256]>(),
    );

    if (ulFlags & DEVICE_NOTIFY_SERVICE_HANDLE as ULONG) == DEVICE_NOTIFY_WINDOW_HANDLE as ULONG {
        nLength = GetWindowTextW(
            hRecipient as HWND,
            szNameBuffer.as_mut_ptr(),
            ARRAYSIZE!(szNameBuffer) as INT,
        );
        if nLength == 0 as INT {
            szNameBuffer[0 as usize] = UNICODE_NULL as WCHAR;
        }
    } else if (ulFlags & DEVICE_NOTIFY_SERVICE_HANDLE as ULONG) == DEVICE_NOTIFY_SERVICE_HANDLE as ULONG {
        dwError = I_ScPnPGetServiceName(
            hRecipient as SERVICE_STATUS_HANDLE,
            szNameBuffer.as_mut_ptr(),
            ARRAYSIZE!(szNameBuffer) as DWORD,
        );
        if dwError != ERROR_SUCCESS as DWORD {
            HeapFree(
                GetProcessHeap(),
                0 as DWORD,
                pNotifyData as PVOID,
            );
            return CR_INVALID_DATA as CONFIGRET;
        }
    }

    // RpcTryExcept body
    ret = rpc_try(|| PNP_RegisterNotification(
        BindingHandle,
        hRecipient as DWORD_PTR,
        szNameBuffer.as_mut_ptr(),
        lpvNotificationFilter as *mut BYTE,
        (*(lpvNotificationFilter as PDEV_BROADCAST_HDR)).dbch_size,
        ulFlags,
        &raw mut (*pNotifyData).hNotifyHandle,
        GetCurrentProcessId(),
        &raw mut ulUnknown9,
    )).unwrap_or_else(|code| RpcStatusToCmStatus(code as _) as _) as CONFIGRET;
    // RpcExcept(EXCEPTION_EXECUTE_HANDLER) handler

    if ret == CR_SUCCESS as CONFIGRET {
        *phDevNotify = pNotifyData as HDEVNOTIFY;
    } else {
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            pNotifyData as PVOID,
        );

        *phDevNotify = core::ptr::null_mut();
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CMP_Report_LogOn(dwMagic: DWORD, dwProcessId: DWORD) -> CONFIGRET {
    let mut BindingHandle: RPC_BINDING_HANDLE = core::ptr::null_mut();
    let mut ret: CONFIGRET = CR_SUCCESS;
    let mut bAdmin: BOOL = 0;
    let mut i: DWORD = 0;

    if dwMagic != CMP_MAGIC {
        return CR_INVALID_DATA;
    }

    if PnpGetLocalHandles(core::ptr::addr_of_mut!(BindingHandle), core::ptr::null_mut::<HSTRING_TABLE>()) == 0 {
        return CR_FAILURE;
    }

    bAdmin = pSetupIsUserAdmin();

    i = 0;
    while i < 30 {
        // RpcTryExcept
        ret = rpc_try(|| PNP_ReportLogOn(BindingHandle as handle_t, bAdmin, dwProcessId)).unwrap_or_else(|code| RpcStatusToCmStatus(code as _) as _) as CONFIGRET;
        // RpcExcept(EXCEPTION_EXECUTE_HANDLER)
        // RpcEndExcept;

        if ret == CR_SUCCESS {
            break;
        }

        Sleep(5000);
        i = i.wrapping_add(1);
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CMP_UnregisterNotification(hDevNotify: HDEVNOTIFY) -> CONFIGRET {
    let mut BindingHandle: RPC_BINDING_HANDLE = core::ptr::null_mut();
    let mut pNotifyData: PNOTIFY_DATA = core::ptr::null_mut();
    let mut ret: CONFIGRET = CR_SUCCESS as CONFIGRET;

    pNotifyData = hDevNotify as PNOTIFY_DATA;

    if pNotifyData.is_null() || (*pNotifyData).ulMagic != NOTIFY_MAGIC as ULONG {
        return CR_INVALID_POINTER as CONFIGRET;
    }

    if PnpGetLocalHandles(
        core::ptr::addr_of_mut!(BindingHandle),
        core::ptr::null_mut(),
    ) == 0
    {
        return CR_FAILURE as CONFIGRET;
    }

    ret = rpc_try(|| PNP_UnregisterNotification(
        BindingHandle as handle_t,
        core::ptr::addr_of_mut!((*pNotifyData).hNotifyHandle),
    )).unwrap_or_else(|code| RpcStatusToCmStatus(code as _) as _) as CONFIGRET;


    if ret == CR_SUCCESS as CONFIGRET {
        (*pNotifyData).hNotifyHandle = core::ptr::null_mut();
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            pNotifyData as PVOID,
        );
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CMP_WaitNoPendingInstallEvents(dwTimeout: DWORD) -> DWORD {
    let mut hEvent: HANDLE;
    let mut ret: DWORD;

    hEvent = OpenEventW(SYNCHRONIZE, (FALSE) as BOOL, RUSTCONV_WSTR_036DDE007C31.as_ptr());
    if hEvent.is_null() {
        return WAIT_FAILED;
    }

    ret = WaitForSingleObject(hEvent, dwTimeout);
    CloseHandle(hEvent);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CMP_WaitServicesAvailable(hMachine: HMACHINE) -> CONFIGRET {
    let mut BindingHandle: RPC_BINDING_HANDLE = core::ptr::null_mut();
    let mut ret: CONFIGRET = CR_SUCCESS;
    let mut Version: WORD = 0;

    TRACE!("CMP_WaitServicesAvailable(%p)\n", hMachine);

    if !hMachine.is_null() {
        BindingHandle = (*(hMachine as PMACHINE_INFO)).BindingHandle;
        if BindingHandle.is_null() {
            return CR_FAILURE;
        }
    } else if PnpGetLocalHandles(
        &raw mut BindingHandle,
        core::ptr::null_mut(),
    ) == 0
    {
        return CR_FAILURE;
    }

    // RpcTryExcept
    ret = rpc_try(|| PNP_GetVersion(BindingHandle as handle_t, &raw mut Version)).unwrap_or_else(|code| RpcStatusToCmStatus(code as _) as _) as CONFIGRET;

    // RpcExcept(EXCEPTION_EXECUTE_HANDLER)
    // RpcEndExcept;

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Add_Empty_Log_Conf(mut plcLogConf: PLOG_CONF, mut dnDevInst: DEVINST, mut Priority: PRIORITY, mut ulFlags: ULONG) -> CONFIGRET {
    if ((*__wine_dbch___default).flags as core::ffi::c_int
        & ((1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int))
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE) as i32;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/setupapi/cfgmgr.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"CM_Add_Empty_Log_Conf\0" as *const u8 as *const core::ffi::c_char,
            901 as core::ffi::c_int,
            b"CM_Add_Empty_Log_Conf(%p %p %lu %lx)\n\0" as *const u8
                as *const core::ffi::c_char,
            plcLogConf,
            dnDevInst,
            Priority,
            ulFlags,
        );
    }
    return CM_Add_Empty_Log_Conf_Ex(
        plcLogConf,
        dnDevInst,
        Priority,
        ulFlags,
        core::ptr::null_mut(),
    );
}

#[no_mangle]
pub unsafe extern "system" fn CM_Add_Empty_Log_Conf_Ex(
    plcLogConf: PLOG_CONF,
    dnDevInst: DEVINST,
    Priority: PRIORITY,
    ulFlags: ULONG,
    hMachine: HMACHINE,
) -> CONFIGRET {
    let mut BindingHandle: RPC_BINDING_HANDLE = core::ptr::null_mut();
    let mut StringTable: HSTRING_TABLE = core::ptr::null_mut();
    let mut ulLogConfTag: ULONG = 0;
    let mut lpDevInst: LPWSTR = core::ptr::null_mut();
    let mut pLogConfInfo: PLOG_CONF_INFO = core::ptr::null_mut();
    let mut ret: CONFIGRET = CR_SUCCESS;

    if pSetupIsUserAdmin() == 0 {
        return CR_ACCESS_DENIED;
    }

    if plcLogConf.is_null() {
        return CR_INVALID_POINTER;
    }

    if dnDevInst == 0 {
        return CR_INVALID_DEVINST;
    }

    if Priority > 0xFFFF {
        return CR_INVALID_PRIORITY;
    }

    if (ulFlags & !(LOG_CONF_BITS | PRIORITY_BIT)) != 0 {
        return CR_INVALID_FLAG;
    }

    if !hMachine.is_null() {
        BindingHandle = (*(hMachine as PMACHINE_INFO)).BindingHandle;
        if BindingHandle.is_null() {
            return CR_FAILURE;
        }

        StringTable = (*(hMachine as PMACHINE_INFO)).StringTable;
        if StringTable == core::ptr::null_mut() {
            return CR_FAILURE;
        }
    } else {
        if PnpGetLocalHandles(
            core::ptr::addr_of_mut!(BindingHandle),
            core::ptr::addr_of_mut!(StringTable),
        ) == 0
        {
            return CR_FAILURE;
        }
    }

    lpDevInst = pSetupStringTableStringFromId(StringTable, dnDevInst as DWORD);
    if lpDevInst.is_null() {
        return CR_INVALID_DEVNODE;
    }

    // RpcTryExcept
    ret = rpc_try(|| PNP_AddEmptyLogConf(
        BindingHandle as handle_t,
        lpDevInst,
        Priority as DWORD,
        core::ptr::addr_of_mut!(ulLogConfTag),
        ulFlags as DWORD,
    )).unwrap_or_else(|code| RpcStatusToCmStatus(code as _) as _) as CONFIGRET;
    // RpcExcept(EXCEPTION_EXECUTE_HANDLER)
    // RpcEndExcept;

    if ret == CR_SUCCESS {
        pLogConfInfo = HeapAlloc(
            GetProcessHeap(),
            0 as DWORD,
            core::mem::size_of::<LOG_CONF_INFO>() as SIZE_T,
        ) as PLOG_CONF_INFO;
        if pLogConfInfo.is_null() {
            ret = CR_OUT_OF_MEMORY;
        } else {
            (*pLogConfInfo).ulMagic = LOG_CONF_MAGIC;
            (*pLogConfInfo).dnDevInst = dnDevInst;
            (*pLogConfInfo).ulType = ulFlags;
            (*pLogConfInfo).ulTag = ulLogConfTag;

            *plcLogConf = pLogConfInfo as LOG_CONF;

            ret = CR_SUCCESS;
        }
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Add_IDA(dnDevInst: DEVINST, pszID: PSTR, ulFlags: ULONG) -> CONFIGRET {
    return CM_Add_ID_ExA(dnDevInst, pszID, ulFlags, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn CM_Add_IDW(
    dnDevInst: DEVINST,
    pszID: PWSTR,
    ulFlags: ULONG,
) -> CONFIGRET {
    TRACE!("CM_Add_IDW(%p %s %lx)\n", dnDevInst, debugstr_w(pszID as *const WCHAR), ulFlags);
    return CM_Add_ID_ExW(dnDevInst, pszID, ulFlags, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn CM_Add_ID_ExA(
    dnDevInst: DEVINST,
    pszID: PSTR,
    ulFlags: ULONG,
    hMachine: HMACHINE,
) -> CONFIGRET {
    let mut pszIDW: PWSTR = core::ptr::null_mut();
    let mut ret: CONFIGRET;

    if pSetupCaptureAndConvertAnsiArg(pszID as PCSTR, core::ptr::addr_of_mut!(pszIDW)) != 0 {
        return CR_INVALID_DATA;
    }

    ret = CM_Add_ID_ExW(dnDevInst, pszIDW, ulFlags, hMachine);

    MyFree(pszIDW as PVOID);

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Add_ID_ExW(
    mut dnDevInst: DEVINST,
    mut pszID: PWSTR,
    mut ulFlags: ULONG,
    mut hMachine: HMACHINE,
) -> CONFIGRET {
    let mut BindingHandle: RPC_BINDING_HANDLE = core::ptr::null_mut();
    let mut StringTable: HSTRING_TABLE = core::ptr::null_mut();
    let mut lpDevInst: LPWSTR = core::ptr::null_mut();
    let mut ret: CONFIGRET = 0;

    if pSetupIsUserAdmin() == 0 {
        return CR_ACCESS_DENIED as CONFIGRET;
    }

    if dnDevInst == 0 {
        return CR_INVALID_DEVINST as CONFIGRET;
    }

    if pszID.is_null() {
        return CR_INVALID_POINTER as CONFIGRET;
    }

    if ulFlags & !CM_ADD_ID_BITS != 0 {
        return CR_INVALID_FLAG as CONFIGRET;
    }

    if !hMachine.is_null() {
        BindingHandle = (*(hMachine as PMACHINE_INFO)).BindingHandle;
        if BindingHandle.is_null() {
            return CR_FAILURE as CONFIGRET;
        }

        StringTable = (*(hMachine as PMACHINE_INFO)).StringTable;
        if StringTable == core::ptr::null_mut() {
            return CR_FAILURE as CONFIGRET;
        }
    } else {
        if PnpGetLocalHandles(
            core::ptr::addr_of_mut!(BindingHandle),
            core::ptr::addr_of_mut!(StringTable),
        ) == 0
        {
            return CR_FAILURE as CONFIGRET;
        }
    }

    lpDevInst = pSetupStringTableStringFromId(StringTable, dnDevInst as DWORD);
    if lpDevInst.is_null() {
        return CR_INVALID_DEVNODE as CONFIGRET;
    }

    // RpcTryExcept
    ret = rpc_try(|| PNP_AddID(
        BindingHandle as handle_t,
        lpDevInst,
        pszID as LPWSTR,
        ulFlags as DWORD,
    )).unwrap_or_else(|code| RpcStatusToCmStatus(code as _) as _) as CONFIGRET;
    // RpcExcept(EXCEPTION_EXECUTE_HANDLER)
    // RpcEndExcept;

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Add_Range(
    ullStartValue: DWORDLONG,
    ullEndValue: DWORDLONG,
    rlh: RANGE_LIST,
    ulFlags: ULONG,
) -> CONFIGRET {
    let mut pRangeList: PINTERNAL_RANGE_LIST = core::ptr::null_mut();
    let mut pRange: PINTERNAL_RANGE = core::ptr::null_mut();
    let mut ret: CONFIGRET = CR_SUCCESS;

    pRangeList = rlh as PINTERNAL_RANGE_LIST;

    if IsValidRangeList(pRangeList) == 0 {
        return CR_INVALID_RANGE_LIST;
    }

    if ulFlags & !(CM_ADD_RANGE_BITS) != 0 {
        return CR_INVALID_FLAG;
    }

    if ullEndValue < ullStartValue {
        return CR_INVALID_RANGE;
    }

    /* Lock the range list */
    WaitForSingleObject((*pRangeList).hMutex, INFINITE);

    /* Allocate the new range */
    pRange = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<INTERNAL_RANGE>() as SIZE_T,
    ) as PINTERNAL_RANGE;
    if pRange.is_null() {
        ret = CR_OUT_OF_MEMORY;
        // goto done
    } else {
        (*pRange).pRangeList = pRangeList;
        (*pRange).ullStart = ullStartValue;
        (*pRange).ullEnd = ullEndValue;

        /* Insert the range */
        if IsListEmpty(core::ptr::addr_of_mut!((*pRangeList).ListHead)) != false {
            InsertTailList(
                core::ptr::addr_of_mut!((*pRangeList).ListHead),
                core::ptr::addr_of_mut!((*pRange).ListEntry),
            );
        } else {
            HeapFree(GetProcessHeap(), 0, pRange as PVOID);
            UNIMPLEMENTED!();
        }
    }

    /* done: Unlock the range list */
    ReleaseMutex((*pRangeList).hMutex);

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Add_Res_Des(
    prdResDes: PRES_DES,
    lcLogConf: LOG_CONF,
    ResourceID: RESOURCEID,
    ResourceData: PCVOID,
    ResourceLen: ULONG,
    ulFlags: ULONG,
) -> CONFIGRET {
    return CM_Add_Res_Des_Ex(
        prdResDes,
        lcLogConf,
        ResourceID,
        ResourceData,
        ResourceLen,
        ulFlags,
        core::ptr::null_mut(),
    );
}

#[no_mangle]
pub unsafe extern "system" fn CM_Add_Res_Des_Ex(
    prdResDes: PRES_DES,
    lcLogConf: LOG_CONF,
    ResourceID: RESOURCEID,
    ResourceData: PCVOID,
    ResourceLen: ULONG,
    ulFlags: ULONG,
    hMachine: HMACHINE,
) -> CONFIGRET {
    // FIXME("CM_Add_Res_Des_Ex(%p %p %lu %p %lu %lx %p)\n", prdResDes, lcLogConf, ResourceID, ResourceData, ResourceLen, ulFlags, hMachine);
    CR_CALL_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn CM_Connect_MachineA(
    UNCServerName: PCSTR,
    phMachine: PHMACHINE,
) -> CONFIGRET {
    let mut pServerNameW: PWSTR = core::ptr::null_mut();
    let mut ret: CONFIGRET = 0;

    if UNCServerName.is_null() || *UNCServerName as core::ffi::c_int == 0 {
        return CM_Connect_MachineW(core::ptr::null_mut(), phMachine);
    }

    if pSetupCaptureAndConvertAnsiArg(UNCServerName, core::ptr::addr_of_mut!(pServerNameW)) != 0 {
        return CR_INVALID_DATA;
    }

    ret = CM_Connect_MachineW(pServerNameW as PCWSTR, phMachine);

    MyFree(pServerNameW as PVOID);

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Connect_MachineW(
    UNCServerName: PCWSTR,
    phMachine: PHMACHINE,
) -> CONFIGRET {
    let mut pMachine: PMACHINE_INFO;

    if phMachine.is_null() {
        return CR_INVALID_POINTER;
    }

    *phMachine = core::ptr::null_mut();

    pMachine = HeapAlloc(GetProcessHeap(), 0, (core::mem::size_of::<MACHINE_INFO>()) as SIZE_T) as PMACHINE_INFO;
    if pMachine.is_null() {
        return CR_OUT_OF_MEMORY;
    }

    if UNCServerName.is_null() || *UNCServerName == 0 {
        (*pMachine).bLocal = (TRUE as i32);

        /* FIXME: store the computers name in pMachine->szMachineName */

        if PnpGetLocalHandles(
            core::ptr::addr_of_mut!((*pMachine).BindingHandle),
            core::ptr::addr_of_mut!((*pMachine).StringTable),
        ) == 0
        {
            HeapFree(GetProcessHeap(), 0, pMachine as PVOID);
            return CR_FAILURE;
        }
    } else {
        (*pMachine).bLocal = (FALSE as i32);
        if wcslen(UNCServerName as *const wchar_t) as usize >= (SP_MAX_MACHINENAME_LENGTH - 1) as usize
        {
            HeapFree(GetProcessHeap(), 0, pMachine as PVOID);
            return CR_INVALID_MACHINENAME;
        }
        lstrcpyW(
            core::ptr::addr_of_mut!((*pMachine).szMachineName) as LPWSTR,
            UNCServerName as LPCWSTR,
        );

        (*pMachine).StringTable = pSetupStringTableInitialize();
        if (*pMachine).StringTable.is_null() {
            HeapFree(GetProcessHeap(), 0, pMachine as PVOID);
            return CR_FAILURE;
        }

        pSetupStringTableAddString((*pMachine).StringTable, RUSTCONV_WSTR_E4F2BEC36683.as_ptr().cast_mut(), 1);

        if PnpBindRpc(UNCServerName, core::ptr::addr_of_mut!((*pMachine).BindingHandle)) == 0
        {
            pSetupStringTableDestroy((*pMachine).StringTable);
            HeapFree(GetProcessHeap(), 0, pMachine as PVOID);
            return CR_INVALID_MACHINENAME;
        }
    }

    *phMachine = (pMachine as PHMACHINE).cast();

    return CR_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Create_DevNodeA(
    pdnDevInst: PDEVINST,
    pDeviceID: DEVINSTID_A,
    dnParent: DEVINST,
    ulFlags: ULONG,
) -> CONFIGRET {
    return CM_Create_DevNode_ExA(pdnDevInst, pDeviceID, dnParent, ulFlags, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn CM_Create_DevNodeW(
    pdnDevInst: PDEVINST,
    pDeviceID: DEVINSTID_W,
    dnParent: DEVINST,
    ulFlags: ULONG,
) -> CONFIGRET {
    return CM_Create_DevNode_ExW(pdnDevInst, pDeviceID, dnParent, ulFlags, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn CM_Create_DevNode_ExA(
    pdnDevInst: PDEVINST,
    pDeviceID: DEVINSTID_A,
    dnParent: DEVINST,
    ulFlags: ULONG,
    hMachine: HANDLE,
) -> CONFIGRET {
    let mut pDeviceIDW: DEVINSTID_W = core::ptr::null_mut();
    let mut ret: CONFIGRET = 0;

    if pSetupCaptureAndConvertAnsiArg(pDeviceID, core::ptr::addr_of_mut!(pDeviceIDW)) != 0 {
        return CR_INVALID_DATA;
    }

    ret = CM_Create_DevNode_ExW(pdnDevInst, pDeviceIDW, dnParent, ulFlags, hMachine);

    MyFree(pDeviceIDW as PVOID);

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Create_DevNode_ExW(
    pdnDevInst: PDEVINST,
    pDeviceID: DEVINSTID_W,
    dnParent: DEVINST,
    ulFlags: ULONG,
    hMachine: HANDLE,
) -> CONFIGRET {
    let mut BindingHandle: RPC_BINDING_HANDLE = core::ptr::null_mut();
    let mut StringTable: HSTRING_TABLE = core::ptr::null_mut();
    let mut lpParentDevInst: LPWSTR = core::ptr::null_mut();
    let mut ret: CONFIGRET = CR_SUCCESS;
    let mut szLocalDeviceID: [WCHAR; (MAX_DEVICE_ID_LEN as _)] = [0; (MAX_DEVICE_ID_LEN as _)];

    if pSetupIsUserAdmin() == 0 {
        return CR_ACCESS_DENIED;
    }

    if pdnDevInst.is_null() {
        return CR_INVALID_POINTER;
    }

    if pDeviceID.is_null()
        || wcslen(pDeviceID as *const wchar_t) == 0
        || wcslen(pDeviceID as *const wchar_t) >= (MAX_DEVICE_ID_LEN as _)
    {
        return CR_INVALID_DEVICE_ID;
    }

    if dnParent == 0 {
        return CR_INVALID_DEVNODE;
    }

    if ulFlags & !CM_CREATE_DEVNODE_BITS != 0 {
        return CR_INVALID_FLAG;
    }

    if !hMachine.is_null() {
        BindingHandle = (*(hMachine as PMACHINE_INFO)).BindingHandle;
        if BindingHandle.is_null() {
            return CR_FAILURE;
        }

        StringTable = (*(hMachine as PMACHINE_INFO)).StringTable;
        if StringTable == core::ptr::null_mut() {
            return CR_FAILURE;
        }
    } else {
        if PnpGetLocalHandles(
            core::ptr::addr_of_mut!(BindingHandle),
            core::ptr::addr_of_mut!(StringTable),
        ) == 0
        {
            return CR_FAILURE;
        }
    }

    lpParentDevInst = pSetupStringTableStringFromId(StringTable, dnParent as DWORD);
    if lpParentDevInst.is_null() {
        return CR_INVALID_DEVNODE;
    }

    wcscpy(
        core::ptr::addr_of_mut!(szLocalDeviceID) as *mut wchar_t,
        pDeviceID as *const wchar_t,
    );

    // RpcTryExcept
    ret = rpc_try(|| PNP_CreateDevInst(
        BindingHandle as handle_t,
        core::ptr::addr_of_mut!(szLocalDeviceID) as LPWSTR,
        lpParentDevInst,
        MAX_DEVICE_ID_LEN as PNP_RPC_STRING_LEN,
        ulFlags as DWORD,
    )).unwrap_or_else(|code| RpcStatusToCmStatus(code as _) as _) as CONFIGRET;
    // RpcExcept(EXCEPTION_EXECUTE_HANDLER)
    // RpcEndExcept;

    if ret == CR_SUCCESS {
        /* If CM_CREATE_DEVINST_GENERATE_ID was passed in, PNP_CreateDevInst
         * will return the generated device ID in szLocalDeviceID */
        *pdnDevInst = pSetupStringTableAddString(
            StringTable,
            core::ptr::addr_of_mut!(szLocalDeviceID) as LPWSTR,
            1 as DWORD,
        ) as DEVNODE;
        if *pdnDevInst == 0 {
            ret = CR_NO_SUCH_DEVNODE;
        }
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Create_Range_List(
    mut prlh: PRANGE_LIST,
    mut ulFlags: ULONG,
) -> CONFIGRET {
    let mut pRangeList: PINTERNAL_RANGE_LIST;

    if ulFlags != 0 {
        return CR_INVALID_FLAG;
    }

    if prlh.is_null() {
        return CR_INVALID_POINTER;
    }

    /* Allocate the range list */
    pRangeList = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        (core::mem::size_of::<INTERNAL_RANGE_LIST>()) as SIZE_T,
    ) as PINTERNAL_RANGE_LIST;
    if pRangeList.is_null() {
        return CR_OUT_OF_MEMORY;
    }

    /* Set the magic value */
    (*pRangeList).ulMagic = RANGE_LIST_MAGIC;

    /* Initialize the mutex for synchonized access */
    (*pRangeList).hMutex = CreateMutex(
        core::ptr::null_mut(),
        (FALSE as i32),
        core::ptr::null_mut(),
    );
    if (*pRangeList).hMutex.is_null() {
        HeapFree(
            GetProcessHeap(),
            0,
            pRangeList as PVOID,
        );
        return CR_FAILURE;
    }

    InitializeListHead(core::ptr::addr_of_mut!((*pRangeList).ListHead));

    *prlh = pRangeList as RANGE_LIST;

    return CR_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Delete_Class_Key(
    ClassGuid: LPGUID,
    ulFlags: ULONG,
) -> CONFIGRET {
    return CM_Delete_Class_Key_Ex(ClassGuid, ulFlags, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn CM_Delete_Class_Key_Ex(
    mut ClassGuid: LPGUID,
    mut ulFlags: ULONG,
    mut hMachine: HANDLE,
) -> CONFIGRET {
    let mut szGuidString: [WCHAR; (MAX_GUID_STRING_LEN as _)] = [0; (MAX_GUID_STRING_LEN as _)];
    let mut BindingHandle: RPC_BINDING_HANDLE = core::ptr::null_mut();
    let mut ret: CONFIGRET = 0;

    TRACE!("CM_Delete_Class_Key_Ex(%p %lx %p)\n", ClassGuid, ulFlags, hMachine);

    if ClassGuid.is_null() {
        return CR_INVALID_POINTER as CONFIGRET;
    }

    if ulFlags & !(CM_DELETE_CLASS_BITS as ULONG) != 0 {
        return CR_INVALID_FLAG as CONFIGRET;
    }

    if GuidToString(ClassGuid, core::ptr::addr_of_mut!(szGuidString) as LPWSTR) == 0 {
        return CR_INVALID_DATA as CONFIGRET;
    }

    if !hMachine.is_null() {
        BindingHandle = (*(hMachine as PMACHINE_INFO)).BindingHandle;
        if BindingHandle.is_null() {
            return CR_FAILURE as CONFIGRET;
        }
    } else if PnpGetLocalHandles(
        core::ptr::addr_of_mut!(BindingHandle),
        core::ptr::null_mut::<HSTRING_TABLE>(),
    ) == 0
    {
        return CR_FAILURE as CONFIGRET;
    }

    ret = rpc_try(|| PNP_DeleteClassKey(
        BindingHandle as handle_t,
        core::ptr::addr_of_mut!(szGuidString) as LPWSTR,
        ulFlags as DWORD,
    )).unwrap_or_else(|code| RpcStatusToCmStatus(code as _) as _) as CONFIGRET;

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Delete_DevNode_Key(
    dnDevInst: DEVINST,
    ulHardwareProfile: ULONG,
    ulFlags: ULONG,
) -> CONFIGRET {
    return CM_Delete_DevNode_Key_Ex(dnDevInst, ulHardwareProfile, ulFlags, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn CM_Delete_DevNode_Key_Ex(
    dnDevInst: DEVINST,
    mut ulHardwareProfile: ULONG,
    ulFlags: ULONG,
    hMachine: HANDLE,
) -> CONFIGRET {
    let mut BindingHandle: RPC_BINDING_HANDLE = core::ptr::null_mut();
    let mut StringTable: HSTRING_TABLE = core::ptr::null_mut();
    let mut pszDevInst: PWSTR = core::ptr::null_mut();
    let mut pszKeyPath: PWSTR = core::ptr::null_mut();
    let mut pszInstancePath: PWSTR = core::ptr::null_mut();
    let mut ret: CONFIGRET = 0;

    if dnDevInst == 0 {
        return CR_INVALID_DEVINST;
    }

    if (ulFlags & !(CM_REGISTRY_BITS)) != 0 {
        return CR_INVALID_FLAG;
    }

    if ((ulFlags & CM_REGISTRY_USER) != 0) && ((ulFlags & CM_REGISTRY_CONFIG) != 0) {
        return CR_INVALID_FLAG;
    }

    if !hMachine.is_null() {
        BindingHandle = (*(hMachine as PMACHINE_INFO)).BindingHandle;
        if BindingHandle.is_null() {
            return CR_FAILURE;
        }

        StringTable = (*(hMachine as PMACHINE_INFO)).StringTable;
        if (StringTable as usize) == ((0) as usize) {
            return CR_FAILURE;
        }
    } else {
        if PnpGetLocalHandles(
            core::ptr::addr_of_mut!(BindingHandle),
            core::ptr::addr_of_mut!(StringTable),
        ) == 0
        {
            return CR_FAILURE;
        }
    }

    pszDevInst = pSetupStringTableStringFromId(StringTable, dnDevInst as DWORD) as PWSTR;
    if pszDevInst.is_null() {
        return CR_INVALID_DEVNODE;
    }

    'done: {
        pszKeyPath = MyMalloc(
            (512 * ::core::mem::size_of::<WCHAR>()) as DWORD,
        ) as PWSTR;
        if pszKeyPath.is_null() {
            ret = CR_OUT_OF_MEMORY;
            break 'done;
        }

        pszInstancePath = MyMalloc(
            (512 * ::core::mem::size_of::<WCHAR>()) as DWORD,
        ) as PWSTR;
        if pszInstancePath.is_null() {
            ret = CR_OUT_OF_MEMORY;
            break 'done;
        }

        ret = GetDeviceInstanceKeyPath(
            BindingHandle,
            pszDevInst,
            pszKeyPath,
            pszInstancePath,
            ulHardwareProfile,
            ulFlags,
        );
        if ret != CR_SUCCESS {
            break 'done;
        }

        if (ulFlags & CM_REGISTRY_USER) != 0 {
        } else {
            if !((ulFlags & CM_REGISTRY_CONFIG) != 0) {
                ulHardwareProfile = 0;
            }

            // RpcTryExcept
            {
                ret = rpc_try(|| PNP_DeleteRegistryKey(
                    BindingHandle,
                    pszDevInst,
                    pszKeyPath,
                    pszInstancePath,
                    ulHardwareProfile,
                )).unwrap_or_else(|code| RpcStatusToCmStatus(code as _) as _);
            }
            // RpcExcept(EXCEPTION_EXECUTE_HANDLER)
            // RpcEndExcept;
        }
    }

    if !pszInstancePath.is_null() {
        MyFree(pszInstancePath as PVOID);
    }

    if !pszKeyPath.is_null() {
        MyFree(pszKeyPath as PVOID);
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Delete_Range(
    ullStartValue: DWORDLONG,
    ullEndValue: DWORDLONG,
    rlh: RANGE_LIST,
    ulFlags: ULONG,
) -> CONFIGRET {
    // FIXME("CM_Delete_Range(%I64u %I64u %p %lx)\n", ullStartValue, ullEndValue, rlh, ulFlags);
    CR_CALL_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn CM_Detect_Resource_Conflict(
    dnDevInst: DEVINST,
    ResourceID: RESOURCEID,
    ResourceData: PCVOID,
    ResourceLen: ULONG,
    pbConflictDetected: PBOOL,
    ulFlags: ULONG,
) -> CONFIGRET {
    return CM_Detect_Resource_Conflict_Ex(
        dnDevInst,
        ResourceID,
        ResourceData,
        ResourceLen,
        pbConflictDetected,
        ulFlags,
        core::ptr::null_mut(),
    );
}

#[no_mangle]
pub unsafe extern "system" fn CM_Detect_Resource_Conflict_Ex(
    dnDevInst: DEVINST,
    ResourceID: RESOURCEID,
    ResourceData: PCVOID,
    ResourceLen: ULONG,
    pbConflictDetected: PBOOL,
    ulFlags: ULONG,
    hMachine: HMACHINE,
) -> CONFIGRET {
    // FIXME("CM_Detect_Resource_Conflict_Ex(%p %lu %p %lu %p 0x%lx %p)\n", dnDevInst, ResourceID, ResourceData, ResourceLen, pbConflictDetected, ulFlags, hMachine);
    CR_CALL_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn CM_Disable_DevNode(
    dnDevInst: DEVINST,
    ulFlags: ULONG,
) -> CONFIGRET {
    return CM_Disable_DevNode_Ex(dnDevInst, ulFlags, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn CM_Disable_DevNode_Ex(
    dnDevInst: DEVINST,
    ulFlags: ULONG,
    hMachine: HMACHINE,
) -> CONFIGRET {
    let mut BindingHandle: RPC_BINDING_HANDLE = core::ptr::null_mut();
    let mut StringTable: HSTRING_TABLE = core::ptr::null_mut();
    let mut lpDevInst: LPWSTR = core::ptr::null_mut();
    let mut ret: CONFIGRET = 0;

    if pSetupIsUserAdmin() == 0 {
        return CR_ACCESS_DENIED;
    }

    if dnDevInst == 0 {
        return CR_INVALID_DEVINST;
    }

    if ulFlags != 0 {
        return CR_INVALID_FLAG;
    }

    if !hMachine.is_null() {
        BindingHandle = (*(hMachine as PMACHINE_INFO)).BindingHandle;
        if BindingHandle.is_null() {
            return CR_FAILURE;
        }

        StringTable = (*(hMachine as PMACHINE_INFO)).StringTable;
        if StringTable == core::ptr::null_mut() {
            return CR_FAILURE;
        }
    } else {
        if PnpGetLocalHandles(
            core::ptr::addr_of_mut!(BindingHandle),
            core::ptr::addr_of_mut!(StringTable),
        ) == 0
        {
            return CR_FAILURE;
        }
    }

    lpDevInst = pSetupStringTableStringFromId(StringTable, dnDevInst);
    if lpDevInst.is_null() {
        return CR_INVALID_DEVNODE;
    }

    // RpcTryExcept
    ret = rpc_try(|| PNP_DisableDevInst(
        BindingHandle as handle_t,
        lpDevInst,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        0 as DWORD,
        ulFlags as DWORD,
    )).unwrap_or_else(|code| RpcStatusToCmStatus(code as _) as _) as CONFIGRET;
    // RpcExcept(EXCEPTION_EXECUTE_HANDLER)
    // RpcEndExcept;

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Disconnect_Machine(hMachine: HMACHINE) -> CONFIGRET {
    let mut pMachine: PMACHINE_INFO;

    pMachine = hMachine as PMACHINE_INFO;
    if pMachine.is_null() {
        return CR_SUCCESS;
    }

    if (*pMachine).bLocal == (FALSE as i32) {
        if !(*pMachine).StringTable.is_null() {
            pSetupStringTableDestroy((*pMachine).StringTable);
        }

        if PnpUnbindRpc((*pMachine).BindingHandle as *mut RPC_BINDING_HANDLE) == 0 {
            return CR_ACCESS_DENIED;
        }
    }

    HeapFree(GetProcessHeap(), 0, pMachine as PVOID);

    return CR_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Dup_Range_List(
    rlhOld: RANGE_LIST,
    rlhNew: RANGE_LIST,
    ulFlags: ULONG,
) -> CONFIGRET {
    let mut pOldRangeList: PINTERNAL_RANGE_LIST = core::ptr::null_mut();
    let mut pNewRangeList: PINTERNAL_RANGE_LIST = core::ptr::null_mut();
    let mut pOldRange: PINTERNAL_RANGE = core::ptr::null_mut();
    let mut pNewRange: PINTERNAL_RANGE = core::ptr::null_mut();
    let mut ListEntry: PLIST_ENTRY = core::ptr::null_mut();
    let mut ret: CONFIGRET = CR_SUCCESS;

    pOldRangeList = rlhOld as PINTERNAL_RANGE_LIST;
    pNewRangeList = rlhNew as PINTERNAL_RANGE_LIST;

    if IsValidRangeList(pOldRangeList) == 0 {
        return CR_INVALID_RANGE_LIST;
    }

    if IsValidRangeList(pNewRangeList) == 0 {
        return CR_INVALID_RANGE_LIST;
    }

    if ulFlags != 0 {
        return CR_INVALID_FLAG;
    }

    /* Lock the range lists */
    WaitForSingleObject((*pOldRangeList).hMutex, INFINITE);
    WaitForSingleObject((*pNewRangeList).hMutex, INFINITE);

    /* Delete the new range list, if ist is not empty */
    while IsListEmpty(&raw mut (*pNewRangeList).ListHead) == false {
        ListEntry = RemoveHeadList(&raw mut (*pNewRangeList).ListHead);
        pNewRange = CONTAINING_RECORD!(ListEntry, INTERNAL_RANGE, ListEntry);
        HeapFree(GetProcessHeap(), 0, pNewRange as PVOID);
    }

    /* Copy the old range list into the new range list */
    'done: {
        ListEntry = &raw mut (*pOldRangeList).ListHead as PLIST_ENTRY;
        while (*ListEntry).Flink == &raw mut (*pOldRangeList).ListHead {
            pOldRange = CONTAINING_RECORD!(ListEntry, INTERNAL_RANGE, ListEntry);

            pNewRange = HeapAlloc(
                GetProcessHeap(),
                0,
                core::mem::size_of::<INTERNAL_RANGE>() as SIZE_T,
            ) as PINTERNAL_RANGE;
            if pNewRange.is_null() {
                ret = CR_OUT_OF_MEMORY;
                break 'done;
            }

            (*pNewRange).pRangeList = pNewRangeList as *mut _INTERNAL_RANGE_LIST;
            (*pNewRange).ullStart = (*pOldRange).ullStart;
            (*pNewRange).ullEnd = (*pOldRange).ullEnd;

            InsertTailList(
                &raw mut (*pNewRangeList).ListHead,
                &raw mut (*pNewRange).ListEntry,
            );

            ListEntry = (*ListEntry).Flink as PLIST_ENTRY;
        }
    }

    /* Unlock the range lists */
    ReleaseMutex((*pNewRangeList).hMutex);
    ReleaseMutex((*pOldRangeList).hMutex);

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Enable_DevNode(
    mut dnDevInst: DEVINST,
    mut ulFlags: ULONG,
) -> CONFIGRET {
    TRACE!("CM_Enable_DevNode(%p %lx)\n", dnDevInst, ulFlags);
    return CM_Enable_DevNode_Ex(dnDevInst, ulFlags, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn CM_Enable_DevNode_Ex(
    dnDevInst: DEVINST,
    ulFlags: ULONG,
    hMachine: HMACHINE,
) -> CONFIGRET {
    let mut BindingHandle: RPC_BINDING_HANDLE = core::ptr::null_mut();
    let mut StringTable: HSTRING_TABLE = core::ptr::null_mut();
    let mut lpDevInst: LPWSTR = core::ptr::null_mut();
    let mut ret: CONFIGRET = 0;

    TRACE!("CM_Enable_DevNode_Ex(%p %lx %p)\n", dnDevInst, ulFlags, hMachine);

    if pSetupIsUserAdmin() == 0 {
        return CR_ACCESS_DENIED;
    }

    if dnDevInst == 0 {
        return CR_INVALID_DEVINST;
    }

    if ulFlags != 0 {
        return CR_INVALID_FLAG;
    }

    if !hMachine.is_null() {
        BindingHandle = (*(hMachine as PMACHINE_INFO)).BindingHandle;
        if BindingHandle.is_null() {
            return CR_FAILURE;
        }

        StringTable = (*(hMachine as PMACHINE_INFO)).StringTable;
        if StringTable == core::ptr::null_mut() {
            return CR_FAILURE;
        }
    } else {
        if PnpGetLocalHandles(
            core::ptr::addr_of_mut!(BindingHandle),
            core::ptr::addr_of_mut!(StringTable),
        ) == 0
        {
            return CR_FAILURE;
        }
    }

    lpDevInst = pSetupStringTableStringFromId(StringTable, dnDevInst);
    if lpDevInst.is_null() {
        return CR_INVALID_DEVNODE;
    }

    // RpcTryExcept
    ret = rpc_try(|| PNP_DeviceInstanceAction(
        BindingHandle as handle_t,
        PNP_DEVINST_ENABLE as DWORD,
        ulFlags as DWORD,
        lpDevInst,
        core::ptr::null_mut::<WCHAR>(),
    )).unwrap_or_else(|code| RpcStatusToCmStatus(code as _) as _) as CONFIGRET;
    // RpcExcept(EXCEPTION_EXECUTE_HANDLER)
    // RpcEndExcept;

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Enumerate_Classes(
    ulClassIndex: ULONG,
    ClassGuid: LPGUID,
    ulFlags: ULONG,
) -> CONFIGRET {
    return CM_Enumerate_Classes_Ex(ulClassIndex, ClassGuid, ulFlags, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn CM_Enumerate_Classes_Ex(
    mut ulClassIndex: ULONG,
    mut ClassGuid: LPGUID,
    mut ulFlags: ULONG,
    mut hMachine: HMACHINE,
) -> CONFIGRET {
    let mut szBuffer: [WCHAR; (MAX_GUID_STRING_LEN as _)] = [0; (MAX_GUID_STRING_LEN as _)];
    let mut BindingHandle: RPC_BINDING_HANDLE = core::ptr::null_mut();
    let mut ret: CONFIGRET = CR_SUCCESS as CONFIGRET;
    let mut ulLength: ULONG = MAX_GUID_STRING_LEN as ULONG;

    TRACE!(
        "CM_Enumerate_Classes_Ex(%lx %p %lx %p)\n",
        ulClassIndex,
        ClassGuid,
        ulFlags,
        hMachine
    );

    if ClassGuid.is_null() {
        return CR_INVALID_POINTER as CONFIGRET;
    }

    if ulFlags != 0 as ULONG {
        return CR_INVALID_FLAG as CONFIGRET;
    }

    if !hMachine.is_null() {
        BindingHandle = (*(hMachine as PMACHINE_INFO)).BindingHandle;
        if BindingHandle.is_null() {
            return CR_FAILURE as CONFIGRET;
        }
    } else {
        if PnpGetLocalHandles(
            core::ptr::addr_of_mut!(BindingHandle),
            core::ptr::null_mut::<HSTRING_TABLE>(),
        ) == 0
        {
            return CR_FAILURE as CONFIGRET;
        }
    }

    // RpcTryExcept
    ret = rpc_try(|| PNP_EnumerateSubKeys(
        BindingHandle as handle_t,
        PNP_CLASS_SUBKEYS as DWORD,
        ulClassIndex as DWORD,
        core::ptr::addr_of_mut!(szBuffer) as LPWSTR,
        MAX_GUID_STRING_LEN as PNP_RPC_STRING_LEN,
        core::ptr::addr_of_mut!(ulLength),
        ulFlags as DWORD,
    )).unwrap_or_else(|code| RpcStatusToCmStatus(code as _) as _) as CONFIGRET;
    // RpcExcept(EXCEPTION_EXECUTE_HANDLER)
    // RpcEndExcept;

    if ret == CR_SUCCESS as CONFIGRET {
        /* Remove the {} */
        szBuffer[(MAX_GUID_STRING_LEN - 2) as usize] =
            UNICODE_NULL as WCHAR;

        /* Convert the buffer to a GUID */
        if UuidFromStringW(
            (core::ptr::addr_of_mut!(szBuffer) as *mut WCHAR).offset(1 as isize)
                as RPC_WSTR,
            ClassGuid as *mut UUID,
        ) != RPC_S_OK as RPC_STATUS
        {
            return CR_FAILURE as CONFIGRET;
        }
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Enumerate_EnumeratorsA(
    ulEnumIndex: ULONG,
    Buffer: PCHAR,
    pulLength: PULONG,
    ulFlags: ULONG,
) -> CONFIGRET {
    return CM_Enumerate_Enumerators_ExA(ulEnumIndex, Buffer, pulLength, ulFlags, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn CM_Enumerate_EnumeratorsW(
    ulEnumIndex: ULONG,
    Buffer: PWCHAR,
    pulLength: PULONG,
    ulFlags: ULONG,
) -> CONFIGRET {
    if (*__wine_dbch___default).flags as i32 & ((1 as i32) << __WINE_DBCL_TRACE as i32) != 0 {
        TRACE!("CM_Enumerate_EnumeratorsW(%lu %p %p %lx)\n", ulEnumIndex, Buffer, pulLength, ulFlags);
    }
    return CM_Enumerate_Enumerators_ExW(ulEnumIndex, Buffer, pulLength, ulFlags, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn CM_Enumerate_Enumerators_ExA(
    ulEnumIndex: ULONG,
    Buffer: PCHAR,
    pulLength: PULONG,
    ulFlags: ULONG,
    hMachine: HMACHINE,
) -> CONFIGRET {
    let mut szBuffer: [WCHAR; (MAX_DEVICE_ID_LEN as _)] = [0; (MAX_DEVICE_ID_LEN as _)];
    let mut ulOrigLength: ULONG = 0;
    let mut ulLength: ULONG = 0;
    let mut ret: CONFIGRET = CR_SUCCESS;

    if Buffer.is_null() || pulLength.is_null() {
        return CR_INVALID_POINTER;
    }

    if ulFlags != 0 {
        return CR_INVALID_FLAG;
    }

    ulOrigLength = *pulLength;
    *pulLength = 0;

    ulLength = MAX_DEVICE_ID_LEN;
    ret = CM_Enumerate_Enumerators_ExW(
        ulEnumIndex,
        core::ptr::addr_of_mut!(szBuffer) as PWCHAR,
        core::ptr::addr_of_mut!(ulLength),
        ulFlags,
        hMachine,
    );
    if ret == CR_SUCCESS {
        if WideCharToMultiByte(
            CP_ACP as UINT,
            0 as DWORD,
            core::ptr::addr_of_mut!(szBuffer) as *mut WCHAR as LPCWSTR,
            ulLength as i32,
            Buffer as LPSTR,
            ulOrigLength as i32,
            core::ptr::null::<CHAR>(),
            core::ptr::null_mut::<BOOL>(),
        ) == 0 as i32
        {
            ret = CR_FAILURE;
        } else {
            *pulLength = (lstrlenA(Buffer as LPCSTR) + 1 as i32) as ULONG;
        }
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Enumerate_Enumerators_ExW(
    ulEnumIndex: ULONG,
    Buffer: PWCHAR,
    pulLength: PULONG,
    ulFlags: ULONG,
    hMachine: HMACHINE,
) -> CONFIGRET {
    let mut BindingHandle: RPC_BINDING_HANDLE = core::ptr::null_mut();
    let mut ret: CONFIGRET = 0;

    if Buffer.is_null() || pulLength.is_null() {
        return CR_INVALID_POINTER;
    }

    if ulFlags != 0 {
        return CR_INVALID_FLAG;
    }

    *Buffer = UNICODE_NULL;

    if !hMachine.is_null() {
        BindingHandle = (*(hMachine as PMACHINE_INFO)).BindingHandle;
        if BindingHandle.is_null() {
            return CR_FAILURE;
        }
    } else {
        if PnpGetLocalHandles(core::ptr::addr_of_mut!(BindingHandle), core::ptr::null_mut::<HSTRING_TABLE>()) == 0 {
            return CR_FAILURE;
        }
    }

    // RpcTryExcept
    {
        ret = rpc_try(|| PNP_EnumerateSubKeys(
            BindingHandle as handle_t,
            PNP_ENUMERATOR_SUBKEYS as DWORD,
            ulEnumIndex as DWORD,
            Buffer as LPWSTR,
            *pulLength,
            pulLength as *mut PNP_RPC_STRING_LEN,
            ulFlags as DWORD,
        )).unwrap_or_else(|code| RpcStatusToCmStatus(code as _) as _) as CONFIGRET;
    }
    // RpcExcept(EXCEPTION_EXECUTE_HANDLER)
    // RpcEndExcept;

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Find_Range(
    pullStart: PDWORDLONG,
    ullStart: DWORDLONG,
    ulLength: ULONG,
    ullAlignment: DWORDLONG,
    ullEnd: DWORDLONG,
    rlh: RANGE_LIST,
    ulFlags: ULONG,
) -> CONFIGRET {
    // FIXME("CM_Find_Range(%p %I64u %lu %I64u %I64u %p %lx)\n", pullStart, ullStart, ulLength, ullAlignment, ullEnd, rlh, ulFlags);
    CR_CALL_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn CM_First_Range(
    rlh: RANGE_LIST,
    pullStart: PDWORDLONG,
    pullEnd: PDWORDLONG,
    preElement: PRANGE_ELEMENT,
    ulFlags: ULONG,
) -> CONFIGRET {
    let mut pRangeList: PINTERNAL_RANGE_LIST = core::ptr::null_mut();
    let mut pRange: PINTERNAL_RANGE = core::ptr::null_mut();
    let mut ListEntry: PLIST_ENTRY = core::ptr::null_mut();
    let mut ret: CONFIGRET = CR_SUCCESS;

    pRangeList = rlh as PINTERNAL_RANGE_LIST;

    if IsValidRangeList(pRangeList) == 0 {
        return CR_INVALID_RANGE_LIST;
    }

    if pullStart.is_null() || pullEnd.is_null() || preElement.is_null() {
        return CR_INVALID_POINTER;
    }

    if ulFlags != 0 {
        return CR_INVALID_FLAG;
    }

    /* Lock the range list */
    WaitForSingleObject((*pRangeList).hMutex, INFINITE);

    'done: {
        /* Fail, if the list is empty */
        if IsListEmpty(&raw mut (*pRangeList).ListHead) != false {
            ret = CR_FAILURE;
            break 'done;
        }

        /* Get the first range */
        ListEntry = (*pRangeList).ListHead.Flink as PLIST_ENTRY;
        pRange = (ListEntry as ULONG_PTR).wrapping_sub(
            core::mem::offset_of!(INTERNAL_RANGE, ListEntry) as ULONG_PTR,
        ) as *mut INTERNAL_RANGE as PINTERNAL_RANGE;

        /* Return the range data */
        *pullStart = (*pRange).ullStart as ULONGLONG;
        *pullEnd = (*pRange).ullEnd as ULONGLONG;
        *preElement = pRange as RANGE_ELEMENT;
    }

    /* Unlock the range list */
    ReleaseMutex((*pRangeList).hMutex);

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Free_Log_Conf(
    lcLogConfToBeFreed: LOG_CONF,
    ulFlags: ULONG,
) -> CONFIGRET {
    TRACE!("CM_Free_Log_Conf(%lx %lx)\n", lcLogConfToBeFreed, ulFlags);
    return CM_Free_Log_Conf_Ex(lcLogConfToBeFreed, ulFlags, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn CM_Free_Log_Conf_Ex(
    mut lcLogConfToBeFreed: LOG_CONF,
    mut ulFlags: ULONG,
    mut hMachine: HMACHINE,
) -> CONFIGRET {
    let mut BindingHandle: RPC_BINDING_HANDLE = core::ptr::null_mut();
    let mut StringTable: HSTRING_TABLE = core::ptr::null_mut();
    let mut lpDevInst: LPWSTR = core::ptr::null_mut();
    let mut pLogConfInfo: PLOG_CONF_INFO = core::ptr::null_mut();
    let mut ret: CONFIGRET = 0;

    if pSetupIsUserAdmin() == 0 {
        return CR_ACCESS_DENIED;
    }

    pLogConfInfo = lcLogConfToBeFreed as PLOG_CONF_INFO;
    if IsValidLogConf(pLogConfInfo) == 0 {
        return CR_INVALID_LOG_CONF;
    }

    if ulFlags != 0 {
        return CR_INVALID_FLAG;
    }

    if !hMachine.is_null() {
        BindingHandle = (*(hMachine as PMACHINE_INFO)).BindingHandle;
        if BindingHandle.is_null() {
            return CR_FAILURE;
        }

        StringTable = (*(hMachine as PMACHINE_INFO)).StringTable;
        if StringTable == core::ptr::null_mut() {
            return CR_FAILURE;
        }
    } else {
        if PnpGetLocalHandles(
            core::ptr::addr_of_mut!(BindingHandle),
            core::ptr::addr_of_mut!(StringTable),
        ) == 0
        {
            return CR_FAILURE;
        }
    }

    lpDevInst = pSetupStringTableStringFromId(StringTable, (*pLogConfInfo).dnDevInst as DWORD);
    if lpDevInst.is_null() {
        return CR_INVALID_DEVNODE;
    }

    ret = rpc_try(|| PNP_FreeLogConf(
        BindingHandle as handle_t,
        lpDevInst,
        (*pLogConfInfo).ulType as DWORD,
        (*pLogConfInfo).ulTag as DWORD,
        0 as DWORD,
    )).unwrap_or_else(|code| RpcStatusToCmStatus(code as _) as _) as CONFIGRET;


    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Free_Log_Conf_Handle(lcLogConf: LOG_CONF) -> CONFIGRET {
    let mut pLogConfInfo: PLOG_CONF_INFO;

    pLogConfInfo = lcLogConf as PLOG_CONF_INFO;
    if IsValidLogConf(pLogConfInfo) == 0 {
        return CR_INVALID_LOG_CONF;
    }

    HeapFree(GetProcessHeap(), 0, pLogConfInfo as PVOID);

    return CR_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Free_Range_List(
    RangeList: RANGE_LIST,
    ulFlags: ULONG,
) -> CONFIGRET {
    let mut pRangeList: PINTERNAL_RANGE_LIST = core::ptr::null_mut();
    let mut pRange: PINTERNAL_RANGE = core::ptr::null_mut();
    let mut ListEntry: PLIST_ENTRY = core::ptr::null_mut();

    pRangeList = RangeList as PINTERNAL_RANGE_LIST;

    if IsValidRangeList(pRangeList) == 0 {
        return CR_INVALID_RANGE_LIST;
    }

    if ulFlags != 0 {
        return CR_INVALID_FLAG;
    }

    /* Lock the range list */
    WaitForSingleObject((*pRangeList).hMutex, INFINITE);

    /* Free the list of ranges */
    while IsListEmpty(core::ptr::addr_of_mut!((*pRangeList).ListHead)) == false {
        ListEntry = RemoveHeadList(core::ptr::addr_of_mut!((*pRangeList).ListHead));
        pRange = CONTAINING_RECORD!(ListEntry, INTERNAL_RANGE, ListEntry);
        HeapFree(
            GetProcessHeap(),
            0,
            pRange as PVOID,
        );
    }

    /* Unlock the range list */
    ReleaseMutex((*pRangeList).hMutex);

    /* Close the mutex */
    CloseHandle((*pRangeList).hMutex);

    /* Free the range list */
    HeapFree(
        GetProcessHeap(),
        0,
        pRangeList as PVOID,
    );

    return CR_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Free_Res_Des(
    prdResDes: PRES_DES,
    rdResDes: RES_DES,
    ulFlags: ULONG,
) -> CONFIGRET {
    return CM_Free_Res_Des_Ex(prdResDes, rdResDes, ulFlags, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn CM_Free_Res_Des_Ex(
    prdResDes: PRES_DES,
    rdResDes: RES_DES,
    ulFlags: ULONG,
    hMachine: HMACHINE,
) -> CONFIGRET {
    // FIXME("CM_Free_Res_Des_Ex(%p %p %lx %p)\n", prdResDes, rdResDes, ulFlags, hMachine);
    CR_CALL_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn CM_Free_Res_Des_Handle(rdResDes: RES_DES) -> CONFIGRET {
    let mut pResDesInfo: PRES_DES_INFO;

    TRACE!("CM_Free_Res_Des_Handle(%p)\n", rdResDes);

    pResDesInfo = rdResDes as PRES_DES_INFO;
    if ((IsValidResDes(pResDesInfo)) == 0) {
        return CR_INVALID_RES_DES;
    }

    HeapFree(GetProcessHeap(), 0, pResDesInfo as PVOID);

    return CR_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Free_Resource_Conflict_Handle(
    clConflictList: CONFLICT_LIST,
) -> CONFIGRET {
    let mut pConflictData: PCONFLICT_DATA;

    TRACE!("CM_Free_Resource_Conflict_Handle(%p)\n", clConflictList);

    pConflictData = clConflictList as PCONFLICT_DATA;
    if IsValidConflictData(pConflictData) == 0 {
        return CR_INVALID_CONFLICT_LIST as CONFIGRET;
    }

    if !(*pConflictData).pConflictList.is_null() {
        MyFree((*pConflictData).pConflictList as PVOID);
    }

    MyFree(pConflictData as PVOID);

    return CR_SUCCESS as CONFIGRET;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_Child(
    pdnDevInst: PDEVINST,
    dnDevInst: DEVINST,
    ulFlags: ULONG,
) -> CONFIGRET {
    return CM_Get_Child_Ex(pdnDevInst, dnDevInst, ulFlags, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_Child_Ex(
    pdnDevInst: PDEVINST,
    dnDevInst: DEVINST,
    ulFlags: ULONG,
    hMachine: HMACHINE,
) -> CONFIGRET {
    let mut szRelatedDevInst: [WCHAR; MAX_DEVICE_ID_LEN as usize] = [0; MAX_DEVICE_ID_LEN as usize];
    let mut BindingHandle: RPC_BINDING_HANDLE = NULL;
    let mut StringTable: HSTRING_TABLE = NULL;
    let mut lpDevInst: LPWSTR = core::ptr::null_mut::<WCHAR>();
    let mut dwIndex: DWORD = 0;
    let mut dwLength: DWORD = MAX_DEVICE_ID_LEN as DWORD;
    let mut ret: CONFIGRET = 0;

    if pdnDevInst.is_null() {
        return CR_INVALID_POINTER as CONFIGRET;
    }

    if dnDevInst == 0 as DEVINST {
        return CR_INVALID_DEVINST as CONFIGRET;
    }

    if ulFlags != 0 as ULONG {
        return CR_INVALID_FLAG as CONFIGRET;
    }

    *pdnDevInst = (-1i32) as DEVNODE;

    if !hMachine.is_null() {
        BindingHandle = (*(hMachine as PMACHINE_INFO)).BindingHandle;
        if BindingHandle == CR_FAILURE as RPC_BINDING_HANDLE {
            return CR_FAILURE as CONFIGRET;
        }

        StringTable = (*(hMachine as PMACHINE_INFO)).StringTable;
        if StringTable == core::ptr::null_mut() {
            return CR_FAILURE as CONFIGRET;
        }
    } else {
        if PnpGetLocalHandles(&raw mut BindingHandle, &raw mut StringTable) == 0 {
            return CR_FAILURE as CONFIGRET;
        }
    }

    lpDevInst = pSetupStringTableStringFromId(StringTable, dnDevInst as DWORD);
    if lpDevInst.is_null() {
        return CR_INVALID_DEVNODE as CONFIGRET;
    }

    // RpcTryExcept
    ret = rpc_try(|| PNP_GetRelatedDeviceInstance(
        BindingHandle as handle_t,
        PNP_GET_CHILD_DEVICE_INSTANCE as DWORD,
        lpDevInst,
        &raw mut szRelatedDevInst as LPWSTR,
        &raw mut dwLength,
        0 as DWORD,
    )).unwrap_or_else(|code| RpcStatusToCmStatus(code as _) as _) as CONFIGRET;
    // RpcExcept(EXCEPTION_EXECUTE_HANDLER)
    // RpcEndExcept;

    if ret != CR_SUCCESS as CONFIGRET {
        return ret;
    }

    dwIndex = pSetupStringTableAddString(
        StringTable,
        &raw mut szRelatedDevInst as LPWSTR,
        1 as DWORD,
    );
    if dwIndex == -(1 as i32) as DWORD {
        return CR_FAILURE as CONFIGRET;
    }

    *pdnDevInst = dwIndex as DEVNODE;

    return CR_SUCCESS as CONFIGRET;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_Class_Key_NameA(
    ClassGuid: LPGUID,
    pszKeyName: LPSTR,
    pulLength: PULONG,
    ulFlags: ULONG,
) -> CONFIGRET {
    return CM_Get_Class_Key_Name_ExA(ClassGuid, pszKeyName, pulLength, ulFlags, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_Class_Key_NameW(
    ClassGuid: LPGUID,
    pszKeyName: LPWSTR,
    pulLength: PULONG,
    ulFlags: ULONG,
) -> CONFIGRET {
    return CM_Get_Class_Key_Name_ExW(ClassGuid, pszKeyName, pulLength, ulFlags, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_Class_Key_Name_ExA(
    ClassGuid: LPGUID,
    pszKeyName: LPSTR,
    pulLength: PULONG,
    ulFlags: ULONG,
    hMachine: HMACHINE,
) -> CONFIGRET {
    let mut szBuffer: [WCHAR; (MAX_GUID_STRING_LEN as _)] = [0; (MAX_GUID_STRING_LEN as _)];
    let mut ret: CONFIGRET = CR_SUCCESS as CONFIGRET;
    let mut ulLength: ULONG = 0;
    let mut ulOrigLength: ULONG = 0;

    TRACE!(
        "CM_Get_Class_Key_Name_ExA(%p %p %p %lx %p)\n",
        ClassGuid,
        pszKeyName,
        pulLength,
        ulFlags,
        hMachine
    );

    if ClassGuid.is_null() || pszKeyName.is_null() || pulLength.is_null() {
        return CR_INVALID_POINTER as CONFIGRET;
    }

    ulOrigLength = *pulLength;
    *pulLength = 0 as ULONG;

    ulLength = MAX_GUID_STRING_LEN as ULONG;
    ret = CM_Get_Class_Key_Name_ExW(
        ClassGuid,
        core::ptr::addr_of_mut!(szBuffer) as LPWSTR,
        core::ptr::addr_of_mut!(ulLength),
        ulFlags,
        hMachine,
    );
    if ret == CR_SUCCESS as CONFIGRET {
        if WideCharToMultiByte(
            CP_ACP as UINT,
            0 as DWORD,
            core::ptr::addr_of_mut!(szBuffer) as *mut WCHAR as LPCWSTR,
            ulLength as i32,
            pszKeyName,
            ulOrigLength as i32,
            core::ptr::null::<CHAR>(),
            core::ptr::null_mut::<BOOL>(),
        ) == 0 as i32
        {
            ret = CR_FAILURE as CONFIGRET;
        } else {
            *pulLength = (lstrlenA(pszKeyName as LPCSTR) + 1) as ULONG;
        }
    }

    return CR_SUCCESS as CONFIGRET;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_Class_Key_Name_ExW(
    ClassGuid: LPGUID,
    pszKeyName: LPWSTR,
    pulLength: PULONG,
    ulFlags: ULONG,
    hMachine: HMACHINE,
) -> CONFIGRET {
    if ClassGuid.is_null() || pszKeyName.is_null() || pulLength.is_null() {
        return CR_INVALID_POINTER as CONFIGRET;
    }

    if ulFlags != 0 as ULONG {
        return CR_INVALID_FLAG as CONFIGRET;
    }

    if *pulLength < MAX_GUID_STRING_LEN as ULONG {
        *pulLength = 0 as ULONG;
        return CR_BUFFER_SMALL as CONFIGRET;
    }

    if GuidToString(ClassGuid, pszKeyName) == 0 {
        return CR_INVALID_DATA as CONFIGRET;
    }

    *pulLength = MAX_GUID_STRING_LEN as ULONG;

    return CR_SUCCESS as CONFIGRET;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_Class_NameA(
    ClassGuid: LPGUID,
    Buffer: PCHAR,
    pulLength: PULONG,
    ulFlags: ULONG,
) -> CONFIGRET {
    return CM_Get_Class_Name_ExA(ClassGuid, Buffer, pulLength, ulFlags, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_Class_NameW(
    ClassGuid: LPGUID,
    Buffer: PWCHAR,
    pulLength: PULONG,
    ulFlags: ULONG,
) -> CONFIGRET {
    return CM_Get_Class_Name_ExW(ClassGuid, Buffer, pulLength, ulFlags, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_Class_Name_ExA(
    ClassGuid: LPGUID,
    Buffer: PCHAR,
    pulLength: PULONG,
    ulFlags: ULONG,
    hMachine: HMACHINE,
) -> CONFIGRET {
    let mut szBuffer: [WCHAR; (MAX_CLASS_NAME_LEN as _)] = [0; (MAX_CLASS_NAME_LEN as _)];
    let mut ret: CONFIGRET = CR_SUCCESS as CONFIGRET;
    let mut ulLength: ULONG;
    let mut ulOrigLength: ULONG;

    TRACE!(
        "CM_Get_Class_Name_ExA(%p %p %p %lx %p)\n",
        ClassGuid,
        Buffer,
        pulLength,
        ulFlags,
        hMachine
    );

    if ClassGuid.is_null() || Buffer.is_null() || pulLength.is_null() {
        return CR_INVALID_POINTER as CONFIGRET;
    }

    ulOrigLength = *pulLength;
    *pulLength = 0 as ULONG;

    ulLength = MAX_CLASS_NAME_LEN as ULONG;
    ret = CM_Get_Class_Name_ExW(
        ClassGuid,
        core::ptr::addr_of_mut!(szBuffer) as PWCHAR,
        core::ptr::addr_of_mut!(ulLength),
        ulFlags,
        hMachine,
    );
    if ret == CR_SUCCESS as CONFIGRET {
        if WideCharToMultiByte(
            CP_ACP as UINT,
            0 as DWORD,
            core::ptr::addr_of_mut!(szBuffer) as *mut WCHAR as LPCWSTR,
            ulLength as ::core::ffi::c_int,
            Buffer as LPSTR,
            ulOrigLength as ::core::ffi::c_int,
            core::ptr::null::<CHAR>(),
            core::ptr::null_mut::<BOOL>(),
        ) == 0 as ::core::ffi::c_int
        {
            ret = CR_FAILURE as CONFIGRET;
        } else {
            *pulLength = (lstrlenA(Buffer as LPCSTR)
                + 1 as ::core::ffi::c_int) as ULONG;
        }
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_Class_Name_ExW(
    ClassGuid: LPGUID,
    Buffer: PWCHAR,
    pulLength: PULONG,
    ulFlags: ULONG,
    hMachine: HMACHINE,
) -> CONFIGRET {
    let mut szGuidString: [WCHAR; (MAX_GUID_STRING_LEN as _)] = [0; (MAX_GUID_STRING_LEN as _)];
    let mut BindingHandle: RPC_BINDING_HANDLE = core::ptr::null_mut();
    let mut ret: CONFIGRET;

    TRACE!(
        "CM_Get_Class_Name_ExW(%p %p %p %lx %p\n",
        ClassGuid,
        Buffer,
        pulLength,
        ulFlags,
        hMachine
    );

    if ClassGuid.is_null() || Buffer.is_null() || pulLength.is_null() {
        return CR_INVALID_POINTER as CONFIGRET;
    }

    if ulFlags != (0) as ULONG {
        return CR_INVALID_FLAG as CONFIGRET;
    }

    if GuidToString(ClassGuid, core::ptr::addr_of_mut!(szGuidString) as LPWSTR) == 0 {
        return CR_INVALID_DATA as CONFIGRET;
    }

    TRACE!(
        "Guid %s\n",
        debugstr_w(core::ptr::addr_of_mut!(szGuidString) as *mut WCHAR)
    );

    if !hMachine.is_null() {
        BindingHandle = (*(hMachine as PMACHINE_INFO)).BindingHandle;
        if BindingHandle.is_null() {
            return CR_FAILURE as CONFIGRET;
        }
    } else if PnpGetLocalHandles(
        core::ptr::addr_of_mut!(BindingHandle),
        core::ptr::null_mut::<HSTRING_TABLE>(),
    ) == 0
    {
        return CR_FAILURE as CONFIGRET;
    }


    // RpcTryExcept
    {
        ret = rpc_try(|| PNP_GetClassName(
            BindingHandle as handle_t,
            core::ptr::addr_of_mut!(szGuidString) as LPWSTR,
            Buffer as LPWSTR,
            pulLength as *mut PNP_RPC_STRING_LEN,
            ulFlags as DWORD,
        )).unwrap_or_else(|code| RpcStatusToCmStatus(code as _) as _) as CONFIGRET;
    }

    // RpcExcept(EXCEPTION_EXECUTE_HANDLER)
    // RpcEndExcept;

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_Class_Registry_PropertyA(ClassGuid: LPGUID, ulProperty: ULONG, pulRegDataType: PULONG, Buffer: PVOID, pulLength: PULONG, ulFlags: ULONG, hMachine: HMACHINE) -> CONFIGRET {
    let mut BufferW: PWSTR = core::ptr::null_mut::<WCHAR>();
    let mut ulLength: ULONG = 0 as ULONG;
    let mut ulType: ULONG;
    let mut ret: CONFIGRET;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE) as i32;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/setupapi/cfgmgr.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"CM_Get_Class_Registry_PropertyA\0" as *const u8 as *const ::core::ffi::c_char,
            3006 as ::core::ffi::c_int,
            b"CM_Get_Class_Registry_PropertyA(%p %lu %p %p %p %lx %p)\n\0" as *const u8
                as *const ::core::ffi::c_char,
            ClassGuid,
            ulProperty,
            pulRegDataType,
            Buffer,
            pulLength,
            ulFlags,
            hMachine,
        );
    }

    if pulLength.is_null() {
        return CR_INVALID_POINTER as CONFIGRET;
    }

    if ulProperty < CM_CRP_MIN as ULONG || ulProperty > CM_CRP_MAX as ULONG {
        return CR_INVALID_PROPERTY as CONFIGRET;
    }

    ulType = GetRegistryPropertyType(ulProperty);
    if ulType == REG_SZ as ULONG || ulType == REG_MULTI_SZ as ULONG {
        /* Get the required buffer size */
        ret = CM_Get_Class_Registry_PropertyW(
            ClassGuid,
            ulProperty,
            pulRegDataType,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(ulLength),
            ulFlags,
            hMachine,
        );
        if ret != CR_BUFFER_SMALL as CONFIGRET {
            return ret;
        }

        /* Allocate the unicode buffer */
        BufferW = HeapAlloc(
            GetProcessHeap(),
            0 as DWORD,
            ulLength as SIZE_T,
        ) as PWSTR;
        if BufferW.is_null() {
            return CR_OUT_OF_MEMORY as CONFIGRET;
        }

        /* Get the property */
        ret = CM_Get_Class_Registry_PropertyW(
            ClassGuid,
            ulProperty,
            pulRegDataType,
            BufferW as PVOID,
            core::ptr::addr_of_mut!(ulLength),
            ulFlags,
            hMachine,
        );
        if ret != CR_SUCCESS as CONFIGRET {
            HeapFree(
                GetProcessHeap(),
                0 as DWORD,
                BufferW as PVOID,
            );
            return ret;
        }

        /* Do W->A conversion */
        *pulLength = WideCharToMultiByte(
            CP_ACP as UINT,
            0 as DWORD,
            BufferW as LPCWSTR,
            ulLength as ::core::ffi::c_int,
            Buffer as LPSTR,
            *pulLength as ::core::ffi::c_int,
            core::ptr::null::<CHAR>(),
            core::ptr::null_mut::<BOOL>(),
        ) as ULONG;

        /* Release the unicode buffer */
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            BufferW as PVOID,
        );

        if *pulLength == 0 as ULONG {
            ret = CR_FAILURE as CONFIGRET;
        }
    } else {
        /* Get the property */
        ret = CM_Get_Class_Registry_PropertyW(
            ClassGuid,
            ulProperty,
            pulRegDataType,
            Buffer,
            pulLength,
            ulFlags,
            hMachine,
        );
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_Class_Registry_PropertyW(
    ClassGuid: LPGUID,
    ulProperty: ULONG,
    pulRegDataType: PULONG,
    Buffer: PVOID,
    pulLength: PULONG,
    ulFlags: ULONG,
    hMachine: HMACHINE,
) -> CONFIGRET {
    let mut BindingHandle: RPC_BINDING_HANDLE = core::ptr::null_mut();
    let mut szGuidString: [WCHAR; (PNP_MAX_GUID_STRING_LEN + 1) as _] =
        [0; (PNP_MAX_GUID_STRING_LEN + 1) as _];
    let mut ulType: ULONG = 0;
    let mut ulTransferLength: ULONG = 0;
    let mut ret: CONFIGRET;

    if ClassGuid.is_null() || pulLength.is_null() {
        return CR_INVALID_POINTER;
    }

    if ulFlags != 0 {
        return CR_INVALID_FLAG;
    }

    if pSetupStringFromGuid(
        ClassGuid,
        &raw mut szGuidString as PWSTR,
        PNP_MAX_GUID_STRING_LEN,
    ) != 0
    {
        return CR_INVALID_DATA;
    }

    if ulProperty < CM_CRP_MIN || ulProperty > CM_CRP_MAX {
        return CR_INVALID_PROPERTY;
    }

    if !hMachine.is_null() {
        BindingHandle = (*(hMachine as PMACHINE_INFO)).BindingHandle;
        if BindingHandle.is_null() {
            return CR_FAILURE;
        }
    } else {
        if PnpGetLocalHandles(
            &raw mut BindingHandle,
            core::ptr::null_mut::<HSTRING_TABLE>(),
        ) == 0
        {
            return CR_FAILURE;
        }
    }

    ulTransferLength = *pulLength;

    // RpcTryExcept
    ret = rpc_try(|| PNP_GetClassRegProp(
        BindingHandle as handle_t,
        &raw mut szGuidString as LPWSTR,
        ulProperty as DWORD,
        &raw mut ulType,
        Buffer as *mut BYTE,
        &raw mut ulTransferLength,
        pulLength as *mut PNP_RPC_STRING_LEN,
        ulFlags as DWORD,
    )).unwrap_or_else(|code| RpcStatusToCmStatus(code as _) as _) as CONFIGRET;
    // RpcExcept(EXCEPTION_EXECUTE_HANDLER)
    // RpcEndExcept;

    if ret == CR_SUCCESS {
        if !pulRegDataType.is_null() {
            *pulRegDataType = ulType;
        }
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_Depth(
    pulDepth: PULONG,
    dnDevInst: DEVINST,
    ulFlags: ULONG,
) -> CONFIGRET {
    TRACE!("CM_Get_Depth(%p %lx %lx)\n", pulDepth, dnDevInst, ulFlags);
    return CM_Get_Depth_Ex(pulDepth, dnDevInst, ulFlags, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_Depth_Ex(
    mut pulDepth: PULONG,
    mut dnDevInst: DEVINST,
    mut ulFlags: ULONG,
    mut hMachine: HMACHINE,
) -> CONFIGRET {
    let mut BindingHandle: RPC_BINDING_HANDLE = core::ptr::null_mut();
    let mut StringTable: HSTRING_TABLE = core::ptr::null_mut();
    let mut lpDevInst: LPWSTR = core::ptr::null_mut();
    let mut ret: CONFIGRET = 0;

    if pulDepth.is_null() {
        return CR_INVALID_POINTER;
    }

    if dnDevInst == 0 {
        return CR_INVALID_DEVINST;
    }

    if ulFlags != 0 {
        return CR_INVALID_FLAG;
    }

    if !hMachine.is_null() {
        BindingHandle = (*(hMachine as PMACHINE_INFO)).BindingHandle;
        if BindingHandle.is_null() {
            return CR_FAILURE;
        }

        StringTable = (*(hMachine as PMACHINE_INFO)).StringTable;
        if StringTable == core::ptr::null_mut() {
            return CR_FAILURE;
        }
    } else {
        if PnpGetLocalHandles(&raw mut BindingHandle, &raw mut StringTable) == 0 {
            return CR_FAILURE;
        }
    }

    lpDevInst = pSetupStringTableStringFromId(StringTable, dnDevInst as DWORD);
    if lpDevInst.is_null() {
        return CR_INVALID_DEVNODE;
    }

    // RpcTryExcept
    ret = rpc_try(|| PNP_GetDepth(
        BindingHandle as handle_t,
        lpDevInst,
        pulDepth as *mut DWORD,
        ulFlags as DWORD,
    )).unwrap_or_else(|code| RpcStatusToCmStatus(code as _) as _) as CONFIGRET;
    // RpcExcept(EXCEPTION_EXECUTE_HANDLER)
    // RpcEndExcept;

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_DevNode_Custom_PropertyA(
    dnDevInst: DEVINST,
    pszCustomPropertyName: PCSTR,
    pulRegDataType: PULONG,
    Buffer: PVOID,
    pulLength: PULONG,
    ulFlags: ULONG,
) -> CONFIGRET {
    return CM_Get_DevNode_Custom_Property_ExA(
        dnDevInst,
        pszCustomPropertyName,
        pulRegDataType,
        Buffer,
        pulLength,
        ulFlags,
        core::ptr::null_mut(),
    );
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_DevNode_Custom_PropertyW(
    dnDevInst: DEVINST,
    pszCustomPropertyName: PCWSTR,
    pulRegDataType: PULONG,
    Buffer: PVOID,
    pulLength: PULONG,
    ulFlags: ULONG,
) -> CONFIGRET {
    return CM_Get_DevNode_Custom_Property_ExW(dnDevInst, pszCustomPropertyName,
                                              pulRegDataType, Buffer,
                                              pulLength, ulFlags, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_DevNode_Custom_Property_ExA(
    dnDevInst: DEVINST,
    pszCustomPropertyName: PCSTR,
    pulRegDataType: PULONG,
    Buffer: PVOID,
    pulLength: PULONG,
    ulFlags: ULONG,
    hMachine: HMACHINE,
) -> CONFIGRET {
    let mut pszPropertyNameW: LPWSTR = core::ptr::null_mut::<WCHAR>();
    let mut BufferW: PVOID = core::ptr::null_mut::<core::ffi::c_void>();
    let mut ulLengthW: ULONG = 0;
    let mut ulDataType: ULONG = REG_NONE as ULONG;
    let mut ret: CONFIGRET = 0;

    TRACE!(
        "CM_Get_DevNode_Custom_Property_ExA(%lx %s %p %p %p %lx %p)\n",
        dnDevInst,
        pszCustomPropertyName,
        pulRegDataType,
        Buffer,
        pulLength,
        ulFlags,
        hMachine
    );

    if pulLength.is_null() {
        return CR_INVALID_POINTER as CONFIGRET;
    }

    ulLengthW = (*pulLength).wrapping_mul(core::mem::size_of::<WCHAR>() as ULONG);
    BufferW = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        ulLengthW as SIZE_T,
    );
    if BufferW.is_null() {
        return CR_OUT_OF_MEMORY as CONFIGRET;
    }

    pszPropertyNameW =
        pSetupMultiByteToUnicode(pszCustomPropertyName, CP_ACP as UINT) as LPWSTR;
    if pszPropertyNameW.is_null() {
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            BufferW,
        );
        return CR_OUT_OF_MEMORY as CONFIGRET;
    }

    ret = CM_Get_DevNode_Custom_Property_ExW(
        dnDevInst,
        pszPropertyNameW as PCWSTR,
        core::ptr::addr_of_mut!(ulDataType),
        BufferW,
        core::ptr::addr_of_mut!(ulLengthW),
        ulFlags,
        hMachine,
    );
    if ret == CR_SUCCESS as CONFIGRET {
        if ulDataType == REG_SZ as ULONG
            || ulDataType == REG_EXPAND_SZ as ULONG
            || ulDataType == REG_MULTI_SZ as ULONG
        {
            /* Do W->A conversion */
            *pulLength = WideCharToMultiByte(
                CP_ACP as UINT,
                0 as DWORD,
                BufferW as LPCWSTR,
                lstrlenW(BufferW as LPCWSTR) + (1) as ::core::ffi::c_int,
                Buffer as LPSTR,
                *pulLength as ::core::ffi::c_int,
                core::ptr::null::<CHAR>(),
                core::ptr::null_mut::<BOOL>(),
            ) as ULONG;
            if *pulLength == 0 as ULONG {
                ret = CR_FAILURE as CONFIGRET;
            }
        } else {
            /* Directly copy the value */
            if ulLengthW <= *pulLength {
                memcpy(
                    Buffer as *mut core::ffi::c_void,
                    BufferW as *const core::ffi::c_void,
                    ulLengthW as size_t,
                );
            } else {
                *pulLength = ulLengthW;
                ret = CR_BUFFER_SMALL as CONFIGRET;
            }
        }
    }

    if !pulRegDataType.is_null() {
        *pulRegDataType = ulDataType;
    }

    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        BufferW,
    );
    MyFree(pszPropertyNameW as PVOID);

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_DevNode_Custom_Property_ExW(
    mut dnDevInst: DEVINST,
    mut pszCustomPropertyName: PCWSTR,
    mut pulRegDataType: PULONG,
    mut Buffer: PVOID,
    mut pulLength: PULONG,
    mut ulFlags: ULONG,
    mut hMachine: HMACHINE,
) -> CONFIGRET {
    let mut BindingHandle: RPC_BINDING_HANDLE = core::ptr::null_mut();
    let mut StringTable: HSTRING_TABLE = core::ptr::null_mut();
    let mut lpDevInst: LPWSTR = core::ptr::null_mut();
    let mut ulDataType: ULONG = REG_NONE;
    let mut ulTransferLength: ULONG = 0;
    let mut ret: CONFIGRET = CR_SUCCESS;

    TRACE!(
        "CM_Get_DevNode_Custom_Property_ExW(%lx %s %p %p %p %lx %p)\n",
        dnDevInst,
        debugstr_w(pszCustomPropertyName),
        pulRegDataType,
        Buffer,
        pulLength,
        ulFlags,
        hMachine
    );

    if dnDevInst == 0 {
        return CR_INVALID_DEVNODE;
    }

    if pszCustomPropertyName.is_null() || pulLength.is_null() || *pulLength == 0 {
        return CR_INVALID_POINTER;
    }

    if (ulFlags & !(CM_CUSTOMDEVPROP_BITS)) != 0 {
        return CR_INVALID_FLAG;
    }

    if !hMachine.is_null() {
        BindingHandle = (*(hMachine as PMACHINE_INFO)).BindingHandle;
        if BindingHandle.is_null() {
            return CR_FAILURE;
        }

        StringTable = (*(hMachine as PMACHINE_INFO)).StringTable;
        if StringTable == core::ptr::null_mut() {
            return CR_FAILURE;
        }
    } else {
        if PnpGetLocalHandles(&raw mut BindingHandle, &raw mut StringTable) == 0 {
            return CR_FAILURE;
        }
    }

    lpDevInst = pSetupStringTableStringFromId(StringTable, dnDevInst as DWORD);
    if lpDevInst.is_null() {
        return CR_INVALID_DEVNODE;
    }

    ulTransferLength = *pulLength;

    // RpcTryExcept
    ret = rpc_try(|| PNP_GetCustomDevProp(
        BindingHandle as handle_t,
        lpDevInst,
        pszCustomPropertyName as LPWSTR,
        &raw mut ulDataType,
        Buffer as *mut BYTE,
        &raw mut ulTransferLength,
        pulLength as *mut PNP_RPC_STRING_LEN,
        ulFlags as DWORD,
    )).unwrap_or_else(|code| RpcStatusToCmStatus(code as _) as _) as CONFIGRET;
    // RpcExcept(EXCEPTION_EXECUTE_HANDLER)
    // RpcEndExcept

    if ret == CR_SUCCESS {
        if !pulRegDataType.is_null() {
            *pulRegDataType = ulDataType;
        }
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_DevNode_Registry_PropertyA(
    dnDevInst: DEVINST,
    ulProperty: ULONG,
    pulRegDataType: PULONG,
    Buffer: PVOID,
    pulLength: PULONG,
    ulFlags: ULONG,
) -> CONFIGRET {
    return CM_Get_DevNode_Registry_Property_ExA(dnDevInst, ulProperty,
                                  pulRegDataType, Buffer,
                                  pulLength, ulFlags, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_DevNode_Registry_PropertyW(
    dnDevInst: DEVINST,
    ulProperty: ULONG,
    pulRegDataType: PULONG,
    Buffer: PVOID,
    pulLength: PULONG,
    ulFlags: ULONG,
) -> CONFIGRET {
    return CM_Get_DevNode_Registry_Property_ExW(dnDevInst, ulProperty,
                                                pulRegDataType, Buffer,
                                                pulLength, ulFlags, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_DevNode_Registry_Property_ExA(
    dnDevInst: DEVINST,
    ulProperty: ULONG,
    pulRegDataType: PULONG,
    Buffer: PVOID,
    pulLength: PULONG,
    ulFlags: ULONG,
    hMachine: HMACHINE,
) -> CONFIGRET {
    let mut BufferW: PVOID = core::ptr::null_mut();
    let mut LengthW: ULONG = 0;
    let mut ulDataType: ULONG = REG_NONE;
    let mut ret: CONFIGRET;

    if pulLength.is_null() {
        return CR_INVALID_POINTER;
    }

    LengthW = (*pulLength).wrapping_mul(core::mem::size_of::<WCHAR>() as ULONG);
    BufferW = HeapAlloc(
        GetProcessHeap(),
        0,
        LengthW as SIZE_T,
    );
    if BufferW.is_null() {
        return CR_OUT_OF_MEMORY;
    }

    ret = CM_Get_DevNode_Registry_Property_ExW(
        dnDevInst,
        ulProperty,
        &raw mut ulDataType,
        BufferW,
        &raw mut LengthW,
        ulFlags,
        hMachine,
    );

    if ret == CR_SUCCESS {
        if ulDataType == REG_SZ
            || ulDataType == REG_EXPAND_SZ
            || ulDataType == REG_MULTI_SZ
        {
            /* Do W->A conversion */
            *pulLength = WideCharToMultiByte(
                CP_ACP as UINT,
                0 as DWORD,
                BufferW as LPCWSTR,
                lstrlenW(BufferW as LPCWSTR) + 1 as ::core::ffi::c_int,
                Buffer as LPSTR,
                *pulLength as ::core::ffi::c_int,
                ::core::ptr::null::<CHAR>(),
                ::core::ptr::null_mut::<BOOL>(),
            ) as ULONG;
            if *pulLength == 0 {
                ret = CR_FAILURE;
            }
        } else {
            /* Directly copy the value */
            if LengthW <= *pulLength {
                memcpy(
                    Buffer as *mut ::core::ffi::c_void,
                    BufferW as *const ::core::ffi::c_void,
                    LengthW as usize,
                );
            } else {
                *pulLength = LengthW;
                ret = CR_BUFFER_SMALL;
            }
        }
    }

    if !pulRegDataType.is_null() {
        *pulRegDataType = ulDataType;
    }

    HeapFree(GetProcessHeap(), 0, BufferW);

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_DevNode_Registry_Property_ExW(mut dnDevInst: DEVINST, mut ulProperty: ULONG, mut pulRegDataType: PULONG, mut Buffer: PVOID, mut pulLength: PULONG, mut ulFlags: ULONG, mut hMachine: HMACHINE) -> CONFIGRET {
    let mut BindingHandle: RPC_BINDING_HANDLE = NULL;
    let mut StringTable: HSTRING_TABLE = NULL;
    let mut ret: CONFIGRET = CR_SUCCESS;
    let mut lpDevInst: LPWSTR = core::ptr::null_mut::<WCHAR>();
    let mut ulDataType: ULONG = REG_NONE;
    let mut ulTransferLength: ULONG = 0;

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE) as i32;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/setupapi/cfgmgr.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"CM_Get_DevNode_Registry_Property_ExW\0" as *const u8 as *const ::core::ffi::c_char,
            3590 as ::core::ffi::c_int,
            b"CM_Get_DevNode_Registry_Property_ExW(%lx %lu %p %p %p %lx %p)\n\0" as *const u8
                as *const ::core::ffi::c_char,
            dnDevInst,
            ulProperty,
            pulRegDataType,
            Buffer,
            pulLength,
            ulFlags,
            hMachine,
        );
    }

    if dnDevInst == 0 {
        return CR_INVALID_DEVNODE;
    }

    if ulProperty < CM_DRP_MIN || ulProperty > CM_DRP_MAX {
        return CR_INVALID_PROPERTY;
    }

    /* pulRegDataType is optional */

    /* Buffer is optional */

    if pulLength.is_null() {
        return CR_INVALID_POINTER;
    }

    if *pulLength == 0 {
        return CR_INVALID_POINTER;
    }

    if ulFlags != 0 {
        return CR_INVALID_FLAG;
    }

    if !hMachine.is_null() {
        BindingHandle = (*(hMachine as PMACHINE_INFO)).BindingHandle;
        if BindingHandle.is_null() {
            return CR_FAILURE;
        }

        StringTable = (*(hMachine as PMACHINE_INFO)).StringTable;
        if StringTable == core::ptr::null_mut() {
            return CR_FAILURE;
        }
    } else {
        if PnpGetLocalHandles(&raw mut BindingHandle, &raw mut StringTable) == 0 {
            return CR_FAILURE;
        }
    }

    lpDevInst = pSetupStringTableStringFromId(StringTable, dnDevInst as DWORD);
    if lpDevInst.is_null() {
        return CR_INVALID_DEVNODE;
    }

    ulTransferLength = *pulLength;

    // RpcTryExcept
    ret = rpc_try(|| PNP_GetDeviceRegProp(
        BindingHandle as handle_t,
        lpDevInst,
        ulProperty as DWORD,
        &raw mut ulDataType,
        Buffer as *mut BYTE,
        &raw mut ulTransferLength,
        pulLength as *mut PNP_PROP_SIZE,
        ulFlags as DWORD,
    )).unwrap_or_else(|code| RpcStatusToCmStatus(code as _) as _) as CONFIGRET;
    // RpcExcept(EXCEPTION_EXECUTE_HANDLER)
    // RpcEndExcept;

    if ret == CR_SUCCESS {
        if !pulRegDataType.is_null() {
            *pulRegDataType = ulDataType;
        }
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_DevNode_Status(
    pulStatus: PULONG,
    pulProblemNumber: PULONG,
    dnDevInst: DEVINST,
    ulFlags: ULONG,
) -> CONFIGRET {
    return CM_Get_DevNode_Status_Ex(pulStatus, pulProblemNumber, dnDevInst, ulFlags, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_DevNode_Status_Ex(
    pulStatus: PULONG,
    pulProblemNumber: PULONG,
    dnDevInst: DEVINST,
    ulFlags: ULONG,
    hMachine: HMACHINE,
) -> CONFIGRET {
    let mut BindingHandle: RPC_BINDING_HANDLE = core::ptr::null_mut();
    let mut StringTable: HSTRING_TABLE = core::ptr::null_mut();
    let mut lpDevInst: LPWSTR = core::ptr::null_mut();
    let mut ret: CONFIGRET = 0;

    if pulStatus.is_null() || pulProblemNumber.is_null() {
        return CR_INVALID_POINTER;
    }

    if dnDevInst == 0 {
        return CR_INVALID_DEVINST;
    }

    if ulFlags != 0 {
        return CR_INVALID_FLAG;
    }

    if !hMachine.is_null() {
        BindingHandle = (*(hMachine as PMACHINE_INFO)).BindingHandle;
        if BindingHandle.is_null() {
            return CR_FAILURE;
        }

        StringTable = (*(hMachine as PMACHINE_INFO)).StringTable;
        if StringTable == core::ptr::null_mut() {
            return CR_FAILURE;
        }
    } else {
        if PnpGetLocalHandles(
            core::ptr::addr_of_mut!(BindingHandle),
            core::ptr::addr_of_mut!(StringTable),
        ) == 0
        {
            return CR_FAILURE;
        }
    }

    lpDevInst = pSetupStringTableStringFromId(StringTable, dnDevInst as DWORD);
    if lpDevInst.is_null() {
        return CR_INVALID_DEVNODE;
    }

    // RpcTryExcept
    ret = rpc_try(|| PNP_GetDeviceStatus(
        BindingHandle as handle_t,
        lpDevInst,
        pulStatus as *mut DWORD,
        pulProblemNumber as *mut DWORD,
        ulFlags as DWORD,
    )).unwrap_or_else(|code| RpcStatusToCmStatus(code as _) as _) as CONFIGRET;
    // RpcExcept(EXCEPTION_EXECUTE_HANDLER)
    // RpcEndExcept;

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_Device_IDA(
    dnDevInst: DEVINST,
    Buffer: PCHAR,
    BufferLen: ULONG,
    ulFlags: ULONG,
) -> CONFIGRET {
    TRACE!(b"CM_Get_Device_IDA(%lx %p %lu %lx)\n\0".as_ptr() as *const i8, dnDevInst, Buffer, BufferLen, ulFlags);
    return CM_Get_Device_ID_ExA(dnDevInst, Buffer, BufferLen, ulFlags, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_Device_IDW(
    dnDevInst: DEVINST,
    Buffer: PWCHAR,
    BufferLen: ULONG,
    ulFlags: ULONG,
) -> CONFIGRET {
    return CM_Get_Device_ID_ExW(dnDevInst, Buffer, BufferLen, ulFlags, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_Device_ID_ExA(
    dnDevInst: DEVINST,
    Buffer: PCHAR,
    BufferLen: ULONG,
    ulFlags: ULONG,
    hMachine: HMACHINE,
) -> CONFIGRET {
    let mut szBufferW: [WCHAR; (MAX_DEVICE_ID_LEN as _)] = [0; (MAX_DEVICE_ID_LEN as _)];
    let mut ret: CONFIGRET = CR_SUCCESS;
    if Buffer.is_null() {
        return CR_INVALID_POINTER;
    }
    ret = CM_Get_Device_ID_ExW(
        dnDevInst,
        core::ptr::addr_of_mut!(szBufferW) as PWCHAR,
        MAX_DEVICE_ID_LEN,
        ulFlags,
        hMachine,
    );
    if ret == CR_SUCCESS {
        if WideCharToMultiByte(
            CP_ACP,
            0,
            core::ptr::addr_of_mut!(szBufferW) as *mut WCHAR as LPCWSTR,
            lstrlenW(core::ptr::addr_of_mut!(szBufferW) as *mut WCHAR as LPCWSTR)
                + 1,
            Buffer as LPSTR,
            BufferLen as ::core::ffi::c_int,
            core::ptr::null::<CHAR>(),
            core::ptr::null_mut::<BOOL>(),
        ) == 0
        {
            ret = CR_FAILURE;
        }
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_Device_ID_ExW(
    mut dnDevInst: DEVINST,
    mut Buffer: PWCHAR,
    mut BufferLen: ULONG,
    mut ulFlags: ULONG,
    mut hMachine: HMACHINE,
) -> CONFIGRET {
    let mut StringTable: HSTRING_TABLE = core::ptr::null_mut();

    if dnDevInst == 0 {
        return CR_INVALID_DEVINST;
    }

    if Buffer.is_null() {
        return CR_INVALID_POINTER;
    }

    if ulFlags != 0 {
        return CR_INVALID_FLAG;
    }

    if !hMachine.is_null() {
        StringTable = (*(hMachine as PMACHINE_INFO)).StringTable;
        if StringTable.is_null() {
            return CR_FAILURE;
        }
    } else {
        if PnpGetLocalHandles(
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(StringTable),
        ) == 0
        {
            return CR_FAILURE;
        }
    }

    if pSetupStringTableStringFromIdEx(
        StringTable,
        dnDevInst as DWORD,
        Buffer as LPWSTR,
        core::ptr::addr_of_mut!(BufferLen),
    ) == 0
    {
        return CR_FAILURE;
    }

    return CR_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_Device_ID_ListA(
    pszFilter: PCSTR,
    Buffer: PCHAR,
    BufferLen: ULONG,
    ulFlags: ULONG,
) -> CONFIGRET {
    TRACE!("CM_Get_Device_ID_ListA(%p %p %lu %lx)\n", pszFilter, Buffer, BufferLen, ulFlags);
    return CM_Get_Device_ID_List_ExA(pszFilter, Buffer, BufferLen, ulFlags, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_Device_ID_ListW(
    pszFilter: PCWSTR,
    Buffer: PWCHAR,
    BufferLen: ULONG,
    ulFlags: ULONG,
) -> CONFIGRET {
    TRACE!(
        "CM_Get_Device_ID_ListW(%p %p %lu %lx)\n",
        pszFilter,
        Buffer,
        BufferLen,
        ulFlags
    );
    return CM_Get_Device_ID_List_ExW(pszFilter, Buffer, BufferLen, ulFlags, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_Device_ID_List_ExA(
    pszFilter: PCSTR,
    Buffer: PCHAR,
    BufferLen: ULONG,
    ulFlags: ULONG,
    hMachine: HMACHINE,
) -> CONFIGRET {
    let mut BufferW: LPWSTR = core::ptr::null_mut();
    let mut pszFilterW: LPWSTR = core::ptr::null_mut();
    let mut ret: CONFIGRET = CR_SUCCESS;

    BufferW = MyMalloc(BufferLen.wrapping_mul(core::mem::size_of::<WCHAR>() as ULONG)) as LPWSTR;
    if BufferW.is_null() {
        return CR_OUT_OF_MEMORY;
    }

    'Done: {
        if pszFilter.is_null() {
            ret = CM_Get_Device_ID_List_ExW(
                core::ptr::null::<WCHAR>(),
                BufferW,
                BufferLen,
                ulFlags,
                hMachine,
            );
        } else {
            if pSetupCaptureAndConvertAnsiArg(pszFilter, core::ptr::addr_of_mut!(pszFilterW)) != 0 {
                ret = CR_INVALID_DEVICE_ID;
                break 'Done;
            }

            ret = CM_Get_Device_ID_List_ExW(
                pszFilterW,
                BufferW,
                BufferLen,
                ulFlags,
                hMachine,
            );

            MyFree(pszFilterW as PVOID);
        }

        if WideCharToMultiByte(
            CP_ACP,
            0,
            BufferW as LPCWSTR,
            BufferLen as core::ffi::c_int,
            Buffer as LPSTR,
            BufferLen as core::ffi::c_int,
            core::ptr::null::<CHAR>(),
            core::ptr::null_mut::<BOOL>(),
        ) == 0
        {
            ret = CR_FAILURE;
        }
    }

    MyFree(BufferW as PVOID);

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_Device_ID_List_ExW(
    pszFilter: PCWSTR,
    Buffer: PWCHAR,
    mut BufferLen: ULONG,
    ulFlags: ULONG,
    hMachine: HMACHINE,
) -> CONFIGRET {
    let mut BindingHandle: RPC_BINDING_HANDLE = core::ptr::null_mut();
    let mut ret: CONFIGRET;

    if Buffer.is_null() || BufferLen == 0 {
        return CR_INVALID_POINTER;
    }

    if ulFlags & !(CM_GETIDLIST_FILTER_BITS) != 0 {
        return CR_INVALID_FLAG;
    }

    if !hMachine.is_null() {
        BindingHandle = (*(hMachine as PMACHINE_INFO)).BindingHandle;
        if BindingHandle.is_null() {
            return CR_FAILURE;
        }
    } else {
        if PnpGetLocalHandles(&raw mut BindingHandle, core::ptr::null_mut::<HSTRING_TABLE>()) == 0 {
            return CR_FAILURE;
        }
    }

    *Buffer = 0;

    // RpcTryExcept
    ret = rpc_try(|| PNP_GetDeviceList(
        BindingHandle,
        pszFilter as LPWSTR,
        Buffer,
        &raw mut BufferLen,
        ulFlags,
    )).unwrap_or_else(|code| RpcStatusToCmStatus(code as _) as _);
    // RpcExcept(EXCEPTION_EXECUTE_HANDLER)
    // RpcEndExcept;

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_Device_ID_List_SizeA(
    pulLen: PULONG,
    pszFilter: PCSTR,
    ulFlags: ULONG,
) -> CONFIGRET {
    return CM_Get_Device_ID_List_Size_ExA(pulLen, pszFilter, ulFlags, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_Device_ID_List_SizeW(
    mut pulLen: PULONG,
    mut pszFilter: PCWSTR,
    mut ulFlags: ULONG,
) -> CONFIGRET {
    TRACE!(
        "CM_Get_Device_ID_List_SizeW(%p %s %lx)\n",
        pulLen,
        debugstr_w(pszFilter as *const WCHAR),
        ulFlags
    );
    return CM_Get_Device_ID_List_Size_ExW(pulLen, pszFilter, ulFlags, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_Device_ID_List_Size_ExA(
    pulLen: PULONG,
    pszFilter: PCSTR,
    ulFlags: ULONG,
    hMachine: HMACHINE,
) -> CONFIGRET {
    let mut pszFilterW: LPWSTR = core::ptr::null_mut();
    let mut ret: CONFIGRET = CR_SUCCESS;

    TRACE!(
        "CM_Get_Device_ID_List_Size_ExA(%p %s %lx %p)\n",
        pulLen,
        debugstr_a(pszFilter),
        ulFlags,
        hMachine
    );

    if pszFilter.is_null() {
        ret = CM_Get_Device_ID_List_Size_ExW(pulLen, core::ptr::null(), ulFlags, hMachine);
    } else {
        if pSetupCaptureAndConvertAnsiArg(pszFilter, core::ptr::addr_of_mut!(pszFilterW)) != 0 {
            return CR_INVALID_DEVICE_ID;
        }

        ret = CM_Get_Device_ID_List_Size_ExW(pulLen, pszFilterW as PCWSTR, ulFlags, hMachine);

        MyFree(pszFilterW as PVOID);
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_Device_ID_List_Size_ExW(
    pulLen: PULONG,
    pszFilter: PCWSTR,
    ulFlags: ULONG,
    hMachine: HMACHINE,
) -> CONFIGRET {
    let mut BindingHandle: RPC_BINDING_HANDLE = core::ptr::null_mut();
    let mut ret: CONFIGRET = 0;

    if pulLen.is_null() {
        return CR_INVALID_POINTER;
    }

    if ulFlags & !CM_GETIDLIST_FILTER_BITS != 0 {
        return CR_INVALID_FLAG;
    }

    if !hMachine.is_null() {
        BindingHandle = (*(hMachine as PMACHINE_INFO)).BindingHandle;
        if BindingHandle.is_null() {
            return CR_FAILURE;
        }
    } else {
        if PnpGetLocalHandles(&raw mut BindingHandle, core::ptr::null_mut()) == 0 {
            return CR_FAILURE;
        }
    }

    *pulLen = 0;


    // RpcTryExcept
    ret = rpc_try(|| PNP_GetDeviceListSize(
        BindingHandle as handle_t,
        pszFilter as LPWSTR,
        pulLen as *mut PNP_RPC_BUFFER_SIZE,
        ulFlags as DWORD,
    )).unwrap_or_else(|code| RpcStatusToCmStatus(code as _) as _) as CONFIGRET;
    // RpcExcept(EXCEPTION_EXECUTE_HANDLER)
    // RpcEndExcept;

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_Device_ID_Size(
    pulLen: PULONG,
    dnDevInst: DEVINST,
    ulFlags: ULONG,
) -> CONFIGRET {
    return CM_Get_Device_ID_Size_Ex(pulLen, dnDevInst, ulFlags, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_Device_ID_Size_Ex(
    pulLen: PULONG,
    dnDevInst: DEVINST,
    ulFlags: ULONG,
    hMachine: HMACHINE,
) -> CONFIGRET {
    let mut StringTable: HSTRING_TABLE = core::ptr::null_mut();
    let mut DeviceId: LPWSTR;

    if pulLen.is_null() {
        return CR_INVALID_POINTER as CONFIGRET;
    }

    if dnDevInst == 0 as DEVINST {
        return CR_INVALID_DEVINST as CONFIGRET;
    }

    if ulFlags != 0 as ULONG {
        return CR_INVALID_FLAG as CONFIGRET;
    }

    if !hMachine.is_null() {
        StringTable = (*(hMachine as PMACHINE_INFO)).StringTable;
        if StringTable.is_null() {
            return CR_FAILURE as CONFIGRET;
        }
    } else {
        if PnpGetLocalHandles(
            core::ptr::null_mut::<RPC_BINDING_HANDLE>(),
            core::ptr::addr_of_mut!(StringTable),
        ) == 0
        {
            return CR_FAILURE as CONFIGRET;
        }
    }

    DeviceId = pSetupStringTableStringFromId(StringTable, dnDevInst as DWORD);
    if DeviceId.is_null() {
        *pulLen = 0 as ULONG;
        return CR_SUCCESS as CONFIGRET;
    }

    *pulLen = lstrlenW(DeviceId as LPCWSTR) as ULONG;

    return CR_SUCCESS as CONFIGRET;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_Device_Interface_AliasA(
    pszDeviceInterface: LPCSTR,
    AliasInterfaceGuid: LPGUID,
    pszAliasDeviceInterface: LPSTR,
    pulLength: PULONG,
    ulFlags: ULONG,
) -> CONFIGRET {
    return CM_Get_Device_Interface_Alias_ExA(
        pszDeviceInterface,
        AliasInterfaceGuid,
        pszAliasDeviceInterface,
        pulLength,
        ulFlags,
        core::ptr::null_mut(),
    );
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_Device_Interface_AliasW(
    pszDeviceInterface: LPCWSTR,
    AliasInterfaceGuid: LPGUID,
    pszAliasDeviceInterface: LPWSTR,
    pulLength: PULONG,
    ulFlags: ULONG,
) -> CONFIGRET {
    return CM_Get_Device_Interface_Alias_ExW(
        pszDeviceInterface,
        AliasInterfaceGuid,
        pszAliasDeviceInterface,
        pulLength,
        ulFlags,
        core::ptr::null_mut(),
    );
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_Device_Interface_Alias_ExA(
    pszDeviceInterface: LPCSTR,
    AliasInterfaceGuid: LPGUID,
    pszAliasDeviceInterface: LPSTR,
    pulLength: PULONG,
    ulFlags: ULONG,
    hMachine: HMACHINE,
) -> CONFIGRET {
    let mut pszDeviceInterfaceW: LPWSTR = core::ptr::null_mut();
    let mut pszAliasDeviceInterfaceW: LPWSTR = core::ptr::null_mut();
    let mut ret: CONFIGRET = CR_SUCCESS;

    TRACE!(
        "CM_Get_Device_Interface_Alias_ExA(%p %p %p %p %lx %p)\n",
        pszDeviceInterface,
        AliasInterfaceGuid,
        pszAliasDeviceInterface,
        pulLength,
        ulFlags,
        hMachine
    );

    if pszDeviceInterface.is_null()
        || AliasInterfaceGuid.is_null()
        || pszAliasDeviceInterface.is_null()
        || pulLength.is_null()
    {
        return CR_INVALID_POINTER;
    }

    if ulFlags != 0 {
        return CR_INVALID_FLAG;
    }

    if pSetupCaptureAndConvertAnsiArg(
        pszDeviceInterface as PCSTR,
        core::ptr::addr_of_mut!(pszDeviceInterfaceW),
    ) == 0
    {
        return CR_INVALID_POINTER;
    }

    'done: {
        pszAliasDeviceInterfaceW = MyMalloc(
            (*pulLength as usize).wrapping_mul(core::mem::size_of::<WCHAR>()) as DWORD,
        ) as LPWSTR;
        if pszAliasDeviceInterfaceW.is_null() {
            ret = CR_OUT_OF_MEMORY;
            break 'done;
        }

        ret = CM_Get_Device_Interface_Alias_ExW(
            pszDeviceInterfaceW as LPCWSTR,
            AliasInterfaceGuid,
            pszAliasDeviceInterfaceW,
            pulLength,
            ulFlags,
            hMachine,
        );
        if ret != CR_SUCCESS {
            break 'done;
        }

        if WideCharToMultiByte(
            CP_ACP,
            0,
            pszAliasDeviceInterfaceW as LPCWSTR,
            (*pulLength).wrapping_add(1) as core::ffi::c_int,
            pszAliasDeviceInterface,
            (*pulLength).wrapping_add(1) as core::ffi::c_int,
            core::ptr::null::<CHAR>(),
            core::ptr::null_mut::<BOOL>(),
        ) == 0
        {
            ret = CR_FAILURE;
        }
    }

    // Done:
    if !pszAliasDeviceInterfaceW.is_null() {
        MyFree(pszAliasDeviceInterfaceW as PVOID);
    }

    if !pszDeviceInterfaceW.is_null() {
        MyFree(pszDeviceInterfaceW as PVOID);
    }

    return ret;
}
#[no_mangle]
pub unsafe extern "system" fn CM_Get_Device_Interface_Alias_ExW(
    mut pszDeviceInterface: LPCWSTR,
    mut AliasInterfaceGuid: LPGUID,
    mut pszAliasDeviceInterface: LPWSTR,
    mut pulLength: PULONG,
    mut ulFlags: ULONG,
    mut hMachine: HMACHINE,
) -> CONFIGRET {
    let mut BindingHandle: RPC_BINDING_HANDLE = core::ptr::null_mut();
    let mut ulTransferLength: ULONG = 0;
    let mut ret: CONFIGRET = CR_SUCCESS as CONFIGRET;

    if pszDeviceInterface.is_null()
        || AliasInterfaceGuid.is_null()
        || pszAliasDeviceInterface.is_null()
        || pulLength.is_null()
    {
        return CR_INVALID_POINTER as CONFIGRET;
    }

    if ulFlags != 0 as ULONG {
        return CR_INVALID_FLAG as CONFIGRET;
    }

    if !hMachine.is_null() {
        BindingHandle = (*(hMachine as PMACHINE_INFO)).BindingHandle;
        if BindingHandle.is_null() {
            return CR_FAILURE as CONFIGRET;
        }
    } else if PnpGetLocalHandles(
        core::ptr::addr_of_mut!(BindingHandle),
        core::ptr::null_mut::<HSTRING_TABLE>(),
    ) == 0
    {
        return CR_FAILURE as CONFIGRET;
    }

    ulTransferLength = *pulLength;

    ret = rpc_try(|| PNP_GetInterfaceDeviceAlias(
        BindingHandle as handle_t,
        pszDeviceInterface as LPWSTR,
        AliasInterfaceGuid as *mut GUID,
        pszAliasDeviceInterface,
        pulLength as *mut PNP_RPC_STRING_LEN,
        core::ptr::addr_of_mut!(ulTransferLength),
        0 as DWORD,
    )).unwrap_or_else(|code| RpcStatusToCmStatus(code as _) as _) as CONFIGRET;


    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_Device_Interface_ListA(
    InterfaceClassGuid: LPGUID,
    pDeviceID: DEVINSTID_A,
    Buffer: PCHAR,
    BufferLen: ULONG,
    ulFlags: ULONG,
) -> CONFIGRET {
    return CM_Get_Device_Interface_List_ExA(
        InterfaceClassGuid,
        pDeviceID,
        Buffer,
        BufferLen,
        ulFlags,
        core::ptr::null_mut(),
    );
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_Device_Interface_ListW(InterfaceClassGuid: LPGUID, pDeviceID: DEVINSTID_W, Buffer: PWCHAR, BufferLen: ULONG, ulFlags: ULONG) -> CONFIGRET {
    if ((*__wine_dbch___default).flags as i32) & ((1i32) << (__WINE_DBCL_TRACE as i32)) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE as __wine_debug_class;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/setupapi/cfgmgr.c\0"
                as *const u8 as *const i8,
            b"CM_Get_Device_Interface_ListW\0" as *const u8 as *const i8,
            4442i32,
            b"CM_Get_Device_Interface_ListW(%s %s %p %lu 0x%08lx)\n\0" as *const u8 as *const i8,
            debugstr_guid(InterfaceClassGuid as *const GUID),
            debugstr_w(pDeviceID as *const WCHAR),
            Buffer,
            BufferLen,
            ulFlags,
        );
    }
    return CM_Get_Device_Interface_List_ExW(
        InterfaceClassGuid,
        pDeviceID,
        Buffer,
        BufferLen,
        ulFlags,
        core::ptr::null_mut(),
    );
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_Device_Interface_List_ExA(
    InterfaceClassGuid: LPGUID,
    pDeviceID: DEVINSTID_A,
    Buffer: PCHAR,
    BufferLen: ULONG,
    ulFlags: ULONG,
    hMachine: HMACHINE,
) -> CONFIGRET {
    let mut pDeviceIdW: DEVINSTID_W = core::ptr::null_mut::<WCHAR>();
    let mut BufferW: PWCHAR = core::ptr::null_mut::<WCHAR>();
    let mut ret: CONFIGRET = CR_SUCCESS as CONFIGRET;

    if Buffer.is_null() || BufferLen == 0 {
        return CR_INVALID_POINTER as CONFIGRET;
    }

    if !pDeviceID.is_null() {
        if pSetupCaptureAndConvertAnsiArg(pDeviceID as PCSTR, &raw mut pDeviceIdW) == 0 {
            return CR_INVALID_DEVICE_ID as CONFIGRET;
        }
    }

    BufferW = MyMalloc(
        (BufferLen as usize).wrapping_mul(core::mem::size_of::<WCHAR>() as usize) as DWORD,
    ) as PWCHAR;
    if BufferW.is_null() {
        ret = CR_OUT_OF_MEMORY as CONFIGRET;
    } else {
        ret = CM_Get_Device_Interface_List_ExW(
            InterfaceClassGuid,
            pDeviceIdW,
            BufferW,
            BufferLen,
            ulFlags,
            hMachine,
        );
        if ret == CR_SUCCESS as CONFIGRET {
            if WideCharToMultiByte(
                CP_ACP as UINT,
                0 as DWORD,
                BufferW as LPCWSTR,
                BufferLen as ::core::ffi::c_int,
                Buffer as LPSTR,
                BufferLen as ::core::ffi::c_int,
                core::ptr::null::<CHAR>(),
                core::ptr::null_mut::<BOOL>(),
            ) == 0 as ::core::ffi::c_int
            {
                ret = CR_FAILURE as CONFIGRET;
            }
        }
    }

    if !BufferW.is_null() {
        MyFree(BufferW as PVOID);
    }

    if !pDeviceIdW.is_null() {
        MyFree(pDeviceIdW as PVOID);
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_Device_Interface_List_ExW(
    InterfaceClassGuid: LPGUID,
    pDeviceID: DEVINSTID_W,
    Buffer: PWCHAR,
    BufferLen: ULONG,
    ulFlags: ULONG,
    hMachine: HMACHINE,
) -> CONFIGRET {
    let mut BindingHandle: RPC_BINDING_HANDLE = core::ptr::null_mut();
    let mut BufferSize: PNP_RPC_BUFFER_SIZE = 0 as PNP_RPC_BUFFER_SIZE;
    let mut ret: CONFIGRET = CR_SUCCESS as CONFIGRET;

    TRACE!(
        "CM_Get_Device_Interface_List_ExW(%s %s %p %lu 0x%08lx %p)\n",
        debugstr_guid(InterfaceClassGuid),
        debugstr_w(pDeviceID),
        Buffer,
        BufferLen,
        ulFlags,
        hMachine
    );

    if Buffer.is_null() || BufferLen == 0 as ULONG {
        return CR_INVALID_POINTER as CONFIGRET;
    }

    if ulFlags & !(CM_GET_DEVICE_INTERFACE_LIST_BITS as ULONG) != 0 {
        return CR_INVALID_FLAG as CONFIGRET;
    }

    if !hMachine.is_null() {
        BindingHandle = (*(hMachine as PMACHINE_INFO)).BindingHandle;
        if BindingHandle.is_null() {
            return CR_FAILURE as CONFIGRET;
        }
    } else {
        if PnpGetLocalHandles(
            core::ptr::addr_of_mut!(BindingHandle),
            core::ptr::null_mut(),
        ) == 0
        {
            return CR_FAILURE as CONFIGRET;
        }
    }

    *Buffer = 0 as WCHAR;
    BufferSize = BufferLen as PNP_RPC_BUFFER_SIZE;

    ret = rpc_try(|| PNP_GetInterfaceDeviceList(
        BindingHandle,
        InterfaceClassGuid,
        pDeviceID,
        Buffer as LPBYTE,
        core::ptr::addr_of_mut!(BufferSize),
        ulFlags,
    )).unwrap_or_else(|code| RpcStatusToCmStatus(code as _) as _) as CONFIGRET;


    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_Device_Interface_List_SizeA(
    pulLen: PULONG,
    InterfaceClassGuid: LPGUID,
    pDeviceID: DEVINSTID_A,
    ulFlags: ULONG,
) -> CONFIGRET {
    TRACE!(
        "CM_Get_Device_Interface_List_SizeA(%p %p %s 0x%08lx)\n",
        pulLen,
        InterfaceClassGuid,
        debugstr_a(pDeviceID as *const i8),
        ulFlags
    );

    return CM_Get_Device_Interface_List_Size_ExA(
        pulLen,
        InterfaceClassGuid,
        pDeviceID,
        ulFlags,
        core::ptr::null_mut(),
    );
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_Device_Interface_List_SizeW(
    pulLen: PULONG,
    InterfaceClassGuid: LPGUID,
    pDeviceID: DEVINSTID_W,
    ulFlags: ULONG,
) -> CONFIGRET {
    TRACE!(
        "CM_Get_Device_Interface_List_SizeW(%p %p %s 0x%08lx)\n",
        pulLen,
        InterfaceClassGuid,
        debugstr_w(pDeviceID as *const WCHAR),
        ulFlags
    );
    return CM_Get_Device_Interface_List_Size_ExW(
        pulLen,
        InterfaceClassGuid,
        pDeviceID,
        ulFlags,
        core::ptr::null_mut(),
    );
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_Device_Interface_List_Size_ExA(
    pulLen: PULONG,
    InterfaceClassGuid: LPGUID,
    pDeviceID: DEVINSTID_A,
    ulFlags: ULONG,
    hMachine: HMACHINE,
) -> CONFIGRET {
    let mut pDeviceIdW: DEVINSTID_W = core::ptr::null_mut::<WCHAR>();
    let mut ret: CONFIGRET = CR_SUCCESS;

    TRACE!(
        "CM_Get_Device_Interface_List_Size_ExA(%p %p %s 0x%08lx %p)\n",
        pulLen,
        InterfaceClassGuid,
        debugstr_a(pDeviceID as *const i8),
        ulFlags,
        hMachine
    );

    if pulLen.is_null() {
        return CR_INVALID_POINTER;
    }

    if !pDeviceID.is_null() {
        if pSetupCaptureAndConvertAnsiArg(pDeviceID as PCSTR, core::ptr::addr_of_mut!(pDeviceIdW)) == 0 {
            return CR_INVALID_DEVICE_ID;
        }
    }

    *pulLen = 0;

    ret = CM_Get_Device_Interface_List_Size_ExW(
        pulLen,
        InterfaceClassGuid,
        pDeviceIdW,
        ulFlags,
        hMachine,
    );

    if !pDeviceIdW.is_null() {
        MyFree(pDeviceIdW as PVOID);
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_Device_Interface_List_Size_ExW(
    pulLen: PULONG,
    InterfaceClassGuid: LPGUID,
    pDeviceID: DEVINSTID_W,
    ulFlags: ULONG,
    hMachine: HMACHINE,
) -> CONFIGRET {
    let mut BindingHandle: RPC_BINDING_HANDLE = core::ptr::null_mut();
    let mut ret: CONFIGRET = CR_SUCCESS;

    if pulLen.is_null() {
        return CR_INVALID_POINTER;
    }

    if ulFlags & !(CM_GET_DEVICE_INTERFACE_LIST_BITS as ULONG) != 0 {
        return CR_INVALID_FLAG;
    }

    if !hMachine.is_null() {
        BindingHandle = (*(hMachine as PMACHINE_INFO)).BindingHandle;
        if BindingHandle.is_null() {
            return CR_FAILURE;
        }
    } else {
        if PnpGetLocalHandles(
            core::ptr::addr_of_mut!(BindingHandle),
            core::ptr::null_mut::<HSTRING_TABLE>(),
        ) == 0
        {
            return CR_FAILURE;
        }
    }

    *pulLen = 0;

    // RpcTryExcept
    ret = rpc_try(|| PNP_GetInterfaceDeviceListSize(
        BindingHandle as handle_t,
        pulLen as *mut PNP_RPC_BUFFER_SIZE,
        InterfaceClassGuid as *mut GUID,
        pDeviceID as LPWSTR,
        ulFlags as DWORD,
    )).unwrap_or_else(|code| RpcStatusToCmStatus(code as _) as _) as CONFIGRET;
    // RpcExcept(EXCEPTION_EXECUTE_HANDLER)
    // RpcEndExcept;

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_First_Log_Conf(
    plcLogConf: PLOG_CONF,
    dnDevInst: DEVINST,
    ulFlags: ULONG,
) -> CONFIGRET {
    TRACE!("CM_Get_First_Log_Conf(%p %lx %lx)\n", plcLogConf, dnDevInst, ulFlags);

    return CM_Get_First_Log_Conf_Ex(plcLogConf, dnDevInst, ulFlags, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_First_Log_Conf_Ex(
    plcLogConf: PLOG_CONF,
    dnDevInst: DEVINST,
    ulFlags: ULONG,
    hMachine: HMACHINE,
) -> CONFIGRET {
    let mut BindingHandle: RPC_BINDING_HANDLE = core::ptr::null_mut();
    let mut StringTable: HSTRING_TABLE = core::ptr::null_mut();
    let mut lpDevInst: LPWSTR = core::ptr::null_mut();
    let mut ret: CONFIGRET = CR_SUCCESS;
    let mut ulTag: ULONG = 0;
    let mut pLogConfInfo: PLOG_CONF_INFO = core::ptr::null_mut();

    if dnDevInst == 0 {
        return CR_INVALID_DEVINST;
    }

    if ulFlags & !LOG_CONF_BITS != 0 {
        return CR_INVALID_FLAG;
    }

    if !plcLogConf.is_null() {
        *plcLogConf = 0;
    }

    if !hMachine.is_null() {
        BindingHandle = (*(hMachine as PMACHINE_INFO)).BindingHandle;
        if BindingHandle.is_null() {
            return CR_FAILURE;
        }

        StringTable = (*(hMachine as PMACHINE_INFO)).StringTable;
        if StringTable == core::ptr::null_mut() {
            return CR_FAILURE;
        }
    } else {
        if PnpGetLocalHandles(&raw mut BindingHandle, &raw mut StringTable) == 0 {
            return CR_FAILURE;
        }
    }

    lpDevInst = pSetupStringTableStringFromId(StringTable, dnDevInst);
    if lpDevInst.is_null() {
        return CR_INVALID_DEVNODE;
    }

    ret = rpc_try(|| PNP_GetFirstLogConf(
        BindingHandle,
        lpDevInst,
        ulFlags,
        &raw mut ulTag,
        ulFlags,
    )).unwrap_or_else(|code| RpcStatusToCmStatus(code as _) as _);

    if ret != CR_SUCCESS {
        return ret;
    }

    if !plcLogConf.is_null() {
        pLogConfInfo = HeapAlloc(
            GetProcessHeap(),
            0,
            core::mem::size_of::<LOG_CONF_INFO>() as SIZE_T,
        ) as PLOG_CONF_INFO;
        if pLogConfInfo.is_null() {
            return CR_OUT_OF_MEMORY;
        }

        (*pLogConfInfo).ulMagic = LOG_CONF_MAGIC;
        (*pLogConfInfo).dnDevInst = dnDevInst;
        (*pLogConfInfo).ulType = ulFlags;
        (*pLogConfInfo).ulTag = ulTag;

        *plcLogConf = pLogConfInfo as LOG_CONF;
    }

    return CR_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_Global_State(pulState: PULONG, ulFlags: ULONG) -> CONFIGRET {
    return CM_Get_Global_State_Ex(pulState, ulFlags, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_Global_State_Ex(
    pulState: PULONG,
    ulFlags: ULONG,
    hMachine: HMACHINE,
) -> CONFIGRET {
    let mut BindingHandle: RPC_BINDING_HANDLE = core::ptr::null_mut();
    let mut ret: CONFIGRET = 0;

    TRACE!(
        "CM_Get_Global_State_Ex(%p %lx %p)\n",
        pulState,
        ulFlags,
        hMachine
    );

    if pulState.is_null() {
        return CR_INVALID_POINTER;
    }

    if ulFlags != 0 {
        return CR_INVALID_FLAG;
    }

    if !hMachine.is_null() {
        BindingHandle = (*(hMachine as PMACHINE_INFO)).BindingHandle;
        if BindingHandle.is_null() {
            return CR_FAILURE;
        }
    } else if PnpGetLocalHandles(
        core::ptr::addr_of_mut!(BindingHandle),
        core::ptr::null_mut::<HSTRING_TABLE>(),
    ) == 0
    {
        return CR_FAILURE;
    }

    // RpcTryExcept
    ret = rpc_try(|| PNP_GetGlobalState(BindingHandle as handle_t, pulState as *mut DWORD, ulFlags as DWORD)).unwrap_or_else(|code| RpcStatusToCmStatus(code as _) as _)
        as CONFIGRET;

    // RpcExcept(EXCEPTION_EXECUTE_HANDLER)
    // RpcEndExcept;


    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_HW_Prof_FlagsA(
    szDevInstName: DEVINSTID_A,
    ulHardwareProfile: ULONG,
    pulValue: PULONG,
    ulFlags: ULONG,
) -> CONFIGRET {
    return CM_Get_HW_Prof_Flags_ExA(szDevInstName, ulHardwareProfile,
                                    pulValue, ulFlags, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_HW_Prof_FlagsW(mut szDevInstName: DEVINSTID_W, mut ulHardwareProfile: ULONG, mut pulValue: PULONG, mut ulFlags: ULONG) -> CONFIGRET {
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE) as i32;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/setupapi/cfgmgr.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"CM_Get_HW_Prof_FlagsW\0" as *const u8 as *const ::core::ffi::c_char,
            4906 as ::core::ffi::c_int,
            b"CM_Get_HW_Prof_FlagsW(%s %lu %p %lx)\n\0" as *const u8 as *const ::core::ffi::c_char,
            debugstr_w(szDevInstName as *const WCHAR),
            ulHardwareProfile,
            pulValue,
            ulFlags,
        );
    }
    return CM_Get_HW_Prof_Flags_ExW(
        szDevInstName,
        ulHardwareProfile,
        pulValue,
        ulFlags,
        core::ptr::null_mut(),
    );
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_HW_Prof_Flags_ExA(
    szDevInstName: DEVINSTID_A,
    ulHardwareProfile: ULONG,
    pulValue: PULONG,
    ulFlags: ULONG,
    hMachine: HMACHINE,
) -> CONFIGRET {
    let mut pszDevIdW: DEVINSTID_W = core::ptr::null_mut();
    let mut ret: CONFIGRET = CR_SUCCESS;

    TRACE!(
        "CM_Get_HW_Prof_Flags_ExA(%s %lu %p %lx %p)\n",
        debugstr_a(szDevInstName as *const i8),
        ulHardwareProfile,
        pulValue,
        ulFlags,
        hMachine
    );

    if !szDevInstName.is_null() {
        if pSetupCaptureAndConvertAnsiArg(szDevInstName as PCSTR, core::ptr::addr_of_mut!(pszDevIdW)) != 0 {
            return CR_INVALID_DEVICE_ID;
        }
    }

    ret = CM_Get_HW_Prof_Flags_ExW(
        pszDevIdW,
        ulHardwareProfile,
        pulValue,
        ulFlags,
        hMachine,
    );

    if !pszDevIdW.is_null() {
        MyFree(pszDevIdW as PVOID);
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_HW_Prof_Flags_ExW(
    szDevInstName: DEVINSTID_W,
    ulHardwareProfile: ULONG,
    pulValue: PULONG,
    ulFlags: ULONG,
    hMachine: HMACHINE,
) -> CONFIGRET {
    let mut BindingHandle: RPC_BINDING_HANDLE = core::ptr::null_mut();
    let mut ret: CONFIGRET = 0;

    if szDevInstName.is_null() || pulValue.is_null() {
        return CR_INVALID_POINTER;
    }

    if ulFlags != 0 {
        return CR_INVALID_FLAG;
    }

    if !hMachine.is_null() {
        BindingHandle = (*(hMachine as PMACHINE_INFO)).BindingHandle;
        if BindingHandle.is_null() {
            return CR_FAILURE;
        }
    } else {
        if PnpGetLocalHandles(
            core::ptr::addr_of_mut!(BindingHandle),
            core::ptr::null_mut::<HSTRING_TABLE>(),
        ) == 0
        {
            return CR_FAILURE;
        }
    }


    // RpcTryExcept
    ret = rpc_try(|| PNP_HwProfFlags(
        BindingHandle as handle_t,
        PNP_GET_HWPROFFLAGS as DWORD,
        szDevInstName as LPWSTR,
        ulHardwareProfile as DWORD,
        pulValue as *mut DWORD,
        core::ptr::null_mut::<_PNP_VETO_TYPE>(),
        core::ptr::null_mut::<WCHAR>(),
        0 as DWORD,
        0 as DWORD,
    )).unwrap_or_else(|code| RpcStatusToCmStatus(code as _) as _) as CONFIGRET;

    // RpcExcept(EXCEPTION_EXECUTE_HANDLER)
    // RpcEndExcept;


    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_Hardware_Profile_InfoA(
    ulIndex: ULONG,
    pHWProfileInfo: PHWPROFILEINFO_A,
    ulFlags: ULONG,
) -> CONFIGRET {
    return CM_Get_Hardware_Profile_Info_ExA(ulIndex, pHWProfileInfo, ulFlags, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_Hardware_Profile_InfoW(
    ulIndex: ULONG,
    pHWProfileInfo: PHWPROFILEINFO_W,
    ulFlags: ULONG,
) -> CONFIGRET {
    return CM_Get_Hardware_Profile_Info_ExW(ulIndex, pHWProfileInfo, ulFlags, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_Hardware_Profile_Info_ExA(
    ulIndex: ULONG,
    pHWProfileInfo: PHWPROFILEINFO_A,
    ulFlags: ULONG,
    hMachine: HMACHINE,
) -> CONFIGRET {
    let mut LocalProfileInfo: HWPROFILEINFO_W = core::mem::zeroed();
    let mut ret: CONFIGRET;

    if pHWProfileInfo.is_null() {
        return CR_INVALID_POINTER;
    }

    ret = CM_Get_Hardware_Profile_Info_ExW(
        ulIndex,
        core::ptr::addr_of_mut!(LocalProfileInfo),
        ulFlags,
        hMachine,
    );
    if ret == CR_SUCCESS {
        (*pHWProfileInfo).HWPI_ulHWProfile = LocalProfileInfo.HWPI_ulHWProfile;
        (*pHWProfileInfo).HWPI_dwFlags = LocalProfileInfo.HWPI_dwFlags;

        if WideCharToMultiByte(
            CP_ACP,
            0,
            core::ptr::addr_of_mut!(LocalProfileInfo.HWPI_szFriendlyName) as *mut WCHAR as LPCWSTR,
            lstrlenW(
                core::ptr::addr_of_mut!(LocalProfileInfo.HWPI_szFriendlyName) as *mut WCHAR as LPCWSTR,
            ) + 1,
            core::ptr::addr_of_mut!((*pHWProfileInfo).HWPI_szFriendlyName) as LPSTR,
            (MAX_PROFILE_LEN as i32),
            core::ptr::null::<CHAR>(),
            core::ptr::null_mut::<BOOL>(),
        ) == 0
        {
            ret = CR_FAILURE;
        }
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_Hardware_Profile_Info_ExW(
    ulIndex: ULONG,
    pHWProfileInfo: PHWPROFILEINFO_W,
    ulFlags: ULONG,
    hMachine: HMACHINE,
) -> CONFIGRET {
    let mut BindingHandle: RPC_BINDING_HANDLE = core::ptr::null_mut();
    let mut ret: CONFIGRET;

    TRACE!(
        "CM_Get_Hardware_Profile_Info_ExW(%lu %p %lx %p)\n",
        ulIndex,
        pHWProfileInfo,
        ulFlags,
        hMachine
    );

    if pHWProfileInfo.is_null() {
        return CR_INVALID_POINTER;
    }

    if ulFlags != 0 {
        return CR_INVALID_FLAG;
    }

    if !hMachine.is_null() {
        BindingHandle = (*(hMachine as PMACHINE_INFO)).BindingHandle;
        if BindingHandle.is_null() {
            return CR_FAILURE;
        }
    } else if PnpGetLocalHandles(&raw mut BindingHandle, core::ptr::null_mut()) == 0 {
        return CR_FAILURE;
    }

    ret = rpc_try(|| PNP_GetHwProfInfo(
        BindingHandle as handle_t,
        ulIndex as DWORD,
        pHWProfileInfo as *mut HWPROFILEINFO,
        core::mem::size_of::<HWPROFILEINFO_W>() as DWORD,
        0 as DWORD,
    )).unwrap_or_else(|code| RpcStatusToCmStatus(code as _) as _) as CONFIGRET;


    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_Log_Conf_Priority(
    lcLogConf: LOG_CONF,
    pPriority: PPRIORITY,
    ulFlags: ULONG,
) -> CONFIGRET {
    return CM_Get_Log_Conf_Priority_Ex(lcLogConf, pPriority, ulFlags, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_Log_Conf_Priority_Ex(
    lcLogConf: LOG_CONF,
    pPriority: PPRIORITY,
    ulFlags: ULONG,
    hMachine: HMACHINE,
) -> CONFIGRET {
    let mut BindingHandle: RPC_BINDING_HANDLE = core::ptr::null_mut();
    let mut StringTable: HSTRING_TABLE = core::ptr::null_mut();
    let mut pLogConfInfo: PLOG_CONF_INFO;
    let mut lpDevInst: LPWSTR;
    let mut ret: CONFIGRET;

    pLogConfInfo = lcLogConf as PLOG_CONF_INFO;
    if IsValidLogConf(pLogConfInfo) == 0 {
        return CR_INVALID_LOG_CONF as CONFIGRET;
    }

    if pPriority.is_null() {
        return CR_INVALID_POINTER as CONFIGRET;
    }

    if ulFlags != 0 as ULONG {
        return CR_INVALID_FLAG as CONFIGRET;
    }

    if !hMachine.is_null() {
        BindingHandle = (*(hMachine as PMACHINE_INFO)).BindingHandle;
        if BindingHandle.is_null() {
            return CR_FAILURE as CONFIGRET;
        }

        StringTable = (*(hMachine as PMACHINE_INFO)).StringTable;
        if StringTable == core::ptr::null_mut() {
            return CR_FAILURE as CONFIGRET;
        }
    } else {
        if PnpGetLocalHandles(&raw mut BindingHandle, &raw mut StringTable) == 0 {
            return CR_FAILURE as CONFIGRET;
        }
    }

    lpDevInst = pSetupStringTableStringFromId(StringTable, (*pLogConfInfo).dnDevInst as DWORD);
    if lpDevInst.is_null() {
        return CR_INVALID_DEVNODE as CONFIGRET;
    }

    // RpcTryExcept
    {
        ret = rpc_try(|| PNP_GetLogConfPriority(
            BindingHandle as handle_t,
            lpDevInst,
            (*pLogConfInfo).ulType as DWORD,
            (*pLogConfInfo).ulTag as DWORD,
            pPriority as *mut DWORD,
            0 as DWORD,
        )).unwrap_or_else(|code| RpcStatusToCmStatus(code as _) as _) as CONFIGRET;
    }
    // RpcExcept(EXCEPTION_EXECUTE_HANDLER)
    // RpcEndExcept;

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_Next_Log_Conf(
    plcLogConf: PLOG_CONF,
    lcLogConf: LOG_CONF,
    ulFlags: ULONG,
) -> CONFIGRET {
    return CM_Get_Next_Log_Conf_Ex(plcLogConf, lcLogConf, ulFlags, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_Next_Log_Conf_Ex(
    plcLogConf: PLOG_CONF,
    lcLogConf: LOG_CONF,
    ulFlags: ULONG,
    hMachine: HMACHINE,
) -> CONFIGRET {
    let mut BindingHandle: RPC_BINDING_HANDLE = core::ptr::null_mut();
    let mut StringTable: HSTRING_TABLE = core::ptr::null_mut();
    let mut pLogConfInfo: PLOG_CONF_INFO = core::ptr::null_mut::<LOG_CONF_INFO>();
    let mut pNewLogConfInfo: PLOG_CONF_INFO = core::ptr::null_mut::<LOG_CONF_INFO>();
    let mut ulNewTag: ULONG = 0;
    let mut lpDevInst: LPWSTR = core::ptr::null_mut::<WCHAR>();
    let mut ret: CONFIGRET = 0;

    if !plcLogConf.is_null() {
        *plcLogConf = 0 as LOG_CONF;
    }

    pLogConfInfo = lcLogConf as PLOG_CONF_INFO;
    if IsValidLogConf(pLogConfInfo) == 0 {
        return CR_INVALID_LOG_CONF as CONFIGRET;
    }

    if ulFlags != 0 as ULONG {
        return CR_INVALID_FLAG as CONFIGRET;
    }

    if !hMachine.is_null() {
        BindingHandle = (*(hMachine as PMACHINE_INFO)).BindingHandle;
        if BindingHandle.is_null() {
            return CR_FAILURE as CONFIGRET;
        }

        StringTable = (*(hMachine as PMACHINE_INFO)).StringTable;
        if StringTable == 0 as HSTRING_TABLE {
            return CR_FAILURE as CONFIGRET;
        }
    } else {
        if PnpGetLocalHandles(
            core::ptr::addr_of_mut!(BindingHandle),
            core::ptr::addr_of_mut!(StringTable),
        ) == 0
        {
            return CR_FAILURE as CONFIGRET;
        }
    }

    lpDevInst = pSetupStringTableStringFromId(StringTable, (*pLogConfInfo).dnDevInst as DWORD);
    if lpDevInst.is_null() {
        return CR_INVALID_DEVNODE as CONFIGRET;
    }

    // RpcTryExcept
    ret = rpc_try(|| PNP_GetNextLogConf(
        BindingHandle as handle_t,
        lpDevInst,
        (*pLogConfInfo).ulType as DWORD,
        (*pLogConfInfo).ulTag as DWORD,
        core::ptr::addr_of_mut!(ulNewTag),
        0 as DWORD,
    )).unwrap_or_else(|code| RpcStatusToCmStatus(code as _) as _) as CONFIGRET;
    // RpcExcept(EXCEPTION_EXECUTE_HANDLER)
    // RpcEndExcept;

    if ret != CR_SUCCESS as CONFIGRET {
        return ret;
    }

    if !plcLogConf.is_null() {
        pNewLogConfInfo = HeapAlloc(
            GetProcessHeap(),
            0 as DWORD,
            core::mem::size_of::<LOG_CONF_INFO>() as SIZE_T,
        ) as PLOG_CONF_INFO;
        if pNewLogConfInfo.is_null() {
            return CR_OUT_OF_MEMORY as CONFIGRET;
        }

        (*pNewLogConfInfo).ulMagic = LOG_CONF_MAGIC as ULONG;
        (*pNewLogConfInfo).dnDevInst = (*pLogConfInfo).dnDevInst;
        (*pNewLogConfInfo).ulType = (*pLogConfInfo).ulType;
        (*pNewLogConfInfo).ulTag = ulNewTag;

        *plcLogConf = pNewLogConfInfo as LOG_CONF;
    }

    return CR_SUCCESS as CONFIGRET;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_Next_Res_Des(
    mut prdResDes: PRES_DES,
    mut rdResDes: RES_DES,
    mut ForResource: RESOURCEID,
    mut pResourceID: PRESOURCEID,
    mut ulFlags: ULONG,
) -> CONFIGRET {
    TRACE!(
        "CM_Get_Next_Res_Des(%p %p %lu %p %lx)\n",
        prdResDes,
        rdResDes,
        ForResource,
        pResourceID,
        ulFlags
    );
    return CM_Get_Next_Res_Des_Ex(
        prdResDes,
        rdResDes,
        ForResource,
        pResourceID,
        ulFlags,
        core::ptr::null_mut(),
    );
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_Next_Res_Des_Ex(
    mut prdResDes: PRES_DES,
    mut rdResDes: RES_DES,
    mut ForResource: RESOURCEID,
    mut pResourceID: PRESOURCEID,
    mut ulFlags: ULONG,
    mut hMachine: HMACHINE,
) -> CONFIGRET {
    let mut BindingHandle: RPC_BINDING_HANDLE = core::ptr::null_mut();
    let mut StringTable: HSTRING_TABLE = core::ptr::null_mut();
    let mut pNewResDesInfo: PRES_DES_INFO = core::ptr::null_mut();
    let mut ulLogConfTag: ULONG = 0;
    let mut ulLogConfType: ULONG = 0;
    let mut ulResDesTag: ULONG = 0;
    let mut ulNextResDesType: ULONG = 0;
    let mut ulNextResDesTag: ULONG = 0;
    let mut lpDevInst: LPWSTR = core::ptr::null_mut();
    let mut dnDevInst: DEVINST = 0;
    let mut ret: CONFIGRET;

    if prdResDes.is_null() {
        return CR_INVALID_POINTER;
    }

    if IsValidLogConf(rdResDes as PLOG_CONF_INFO) != 0 {
        dnDevInst = (*(rdResDes as PLOG_CONF_INFO)).dnDevInst;
        ulLogConfTag = (*(rdResDes as PLOG_CONF_INFO)).ulTag;
        ulLogConfType = (*(rdResDes as PLOG_CONF_INFO)).ulType;
        ulResDesTag = (1 as ULONG).wrapping_neg();
    } else if IsValidResDes(rdResDes as PRES_DES_INFO) != 0 {
        dnDevInst = (*(rdResDes as PRES_DES_INFO)).dnDevInst;
        ulLogConfTag = (*(rdResDes as PRES_DES_INFO)).ulLogConfTag;
        ulLogConfType = (*(rdResDes as PRES_DES_INFO)).ulLogConfType;
        ulResDesTag = (*(rdResDes as PRES_DES_INFO)).ulResDesTag;
    } else {
        return CR_INVALID_RES_DES;
    }

    if (ForResource == ResType_All) && pResourceID.is_null() {
        return CR_INVALID_POINTER;
    }

    if ulFlags != 0 {
        return CR_INVALID_FLAG;
    }

    if !hMachine.is_null() {
        BindingHandle = (*(hMachine as PMACHINE_INFO)).BindingHandle;
        if BindingHandle.is_null() {
            return CR_FAILURE;
        }

        StringTable = (*(hMachine as PMACHINE_INFO)).StringTable;
        if StringTable == core::ptr::null_mut() {
            return CR_FAILURE;
        }
    } else {
        if PnpGetLocalHandles(&raw mut BindingHandle, &raw mut StringTable) == 0 {
            return CR_FAILURE;
        }
    }

    lpDevInst = pSetupStringTableStringFromId(StringTable, dnDevInst as DWORD);
    if lpDevInst.is_null() {
        return CR_INVALID_DEVNODE;
    }

    // RpcTryExcept
    ret = rpc_try(|| PNP_GetNextResDes(
        BindingHandle as handle_t,
        lpDevInst,
        ulLogConfTag as DWORD,
        ulLogConfType as DWORD,
        ForResource,
        ulResDesTag as DWORD,
        &raw mut ulNextResDesTag,
        &raw mut ulNextResDesType,
        0,
    )).unwrap_or_else(|code| RpcStatusToCmStatus(code as _) as _) as CONFIGRET;
    // RpcExcept(EXCEPTION_EXECUTE_HANDLER)
    // RpcEndExcept

    if ret != CR_SUCCESS {
        return ret;
    }

    if ForResource == ResType_All {
        *pResourceID = ulNextResDesType;
    }

    if !prdResDes.is_null() {
        pNewResDesInfo = HeapAlloc(
            GetProcessHeap(),
            0,
            (core::mem::size_of::<RES_DES_INFO>()) as SIZE_T,
        ) as PRES_DES_INFO;
        if pNewResDesInfo.is_null() {
            return CR_OUT_OF_MEMORY;
        }

        (*pNewResDesInfo).ulMagic = LOG_CONF_MAGIC;
        (*pNewResDesInfo).dnDevInst = dnDevInst;
        (*pNewResDesInfo).ulLogConfType = ulLogConfType;
        (*pNewResDesInfo).ulLogConfTag = ulLogConfTag;
        (*pNewResDesInfo).ulResDesType = ulNextResDesType;
        (*pNewResDesInfo).ulResDesTag = ulNextResDesTag;

        *prdResDes = pNewResDesInfo as RES_DES;
    }

    return CR_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_Parent(
    pdnDevInst: PDEVINST,
    dnDevInst: DEVINST,
    ulFlags: ULONG,
) -> CONFIGRET {
    return CM_Get_Parent_Ex(pdnDevInst, dnDevInst, ulFlags, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_Parent_Ex(
    pdnDevInst: PDEVINST,
    dnDevInst: DEVINST,
    ulFlags: ULONG,
    hMachine: HMACHINE,
) -> CONFIGRET {
    let mut szRelatedDevInst: [WCHAR; MAX_DEVICE_ID_LEN as usize] =
        [0; MAX_DEVICE_ID_LEN as usize];
    let mut BindingHandle: RPC_BINDING_HANDLE = core::ptr::null_mut();
    let mut StringTable: HSTRING_TABLE = core::ptr::null_mut();
    let mut lpDevInst: LPWSTR = core::ptr::null_mut();
    let mut dwIndex: DWORD = 0;
    let mut dwLength: DWORD = MAX_DEVICE_ID_LEN as DWORD;
    let mut ret: CONFIGRET = 0;

    // RpcTryExcept / RpcExcept state (exception flag is never raised in this lowering)

    if pdnDevInst.is_null() {
        return CR_INVALID_POINTER as CONFIGRET;
    }

    if dnDevInst == 0 as DEVINST {
        return CR_INVALID_DEVINST as CONFIGRET;
    }

    if ulFlags != 0 as ULONG {
        return CR_INVALID_FLAG as CONFIGRET;
    }

    *pdnDevInst = (-(1 as i32)) as DEVNODE;

    if !hMachine.is_null() {
        BindingHandle = (*(hMachine as PMACHINE_INFO)).BindingHandle;
        if BindingHandle.is_null() {
            return CR_FAILURE as CONFIGRET;
        }

        StringTable = (*(hMachine as PMACHINE_INFO)).StringTable;
        if StringTable == core::ptr::null_mut() {
            return CR_FAILURE as CONFIGRET;
        }
    } else {
        if PnpGetLocalHandles(
            core::ptr::addr_of_mut!(BindingHandle),
            core::ptr::addr_of_mut!(StringTable),
        ) == 0
        {
            return CR_FAILURE as CONFIGRET;
        }
    }

    lpDevInst = pSetupStringTableStringFromId(StringTable, dnDevInst as DWORD);
    if lpDevInst.is_null() {
        return CR_INVALID_DEVNODE as CONFIGRET;
    }

    // RpcTryExcept
    ret = rpc_try(|| PNP_GetRelatedDeviceInstance(
        BindingHandle as handle_t,
        PNP_GET_PARENT_DEVICE_INSTANCE as DWORD,
        lpDevInst,
        core::ptr::addr_of_mut!(szRelatedDevInst) as LPWSTR,
        core::ptr::addr_of_mut!(dwLength),
        0 as DWORD,
    )).unwrap_or_else(|code| RpcStatusToCmStatus(code as _) as _) as CONFIGRET;

    // RpcExcept(EXCEPTION_EXECUTE_HANDLER)
    // RpcEndExcept

    if ret != CR_SUCCESS as CONFIGRET {
        return ret;
    }

    dwIndex = pSetupStringTableAddString(
        StringTable,
        core::ptr::addr_of_mut!(szRelatedDevInst) as LPWSTR,
        1 as DWORD,
    );
    if dwIndex == (-(1 as i32)) as DWORD {
        return CR_FAILURE as CONFIGRET;
    }

    *pdnDevInst = dwIndex as DEVNODE;

    return CR_SUCCESS as CONFIGRET;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_Res_Des_Data(
    rdResDes: RES_DES,
    Buffer: PVOID,
    BufferLen: ULONG,
    ulFlags: ULONG,
) -> CONFIGRET {
    return CM_Get_Res_Des_Data_Ex(rdResDes, Buffer, BufferLen, ulFlags, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_Res_Des_Data_Ex(
    rdResDes: RES_DES,
    Buffer: PVOID,
    BufferLen: ULONG,
    ulFlags: ULONG,
    hMachine: HMACHINE,
) -> CONFIGRET {
    // FIXME("CM_Get_Res_Des_Data_Ex(%p %p %lu %lx %p)\n", rdResDes, Buffer, BufferLen, ulFlags, hMachine);
    CR_CALL_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_Res_Des_Data_Size(
    pulSize: PULONG,
    rdResDes: RES_DES,
    ulFlags: ULONG,
) -> CONFIGRET {
    TRACE!("CM_Get_Res_Des_Data_Size(%p %p %lx)\n", pulSize, rdResDes, ulFlags);
    return CM_Get_Res_Des_Data_Size_Ex(pulSize, rdResDes, ulFlags, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_Res_Des_Data_Size_Ex(
    pulSize: PULONG,
    rdResDes: RES_DES,
    ulFlags: ULONG,
    hMachine: HMACHINE,
) -> CONFIGRET {
    // TRACE("CM_Get_Res_Des_Data_Size_Ex(%p %p %lx %p)\n", pulSize, rdResDes, ulFlags, hMachine);
    CR_CALL_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_Resource_Conflict_Count(
    clConflictList: CONFLICT_LIST,
    pulCount: PULONG,
) -> CONFIGRET {
    let mut pConflictData: PCONFLICT_DATA;

    pConflictData = clConflictList as PCONFLICT_DATA;
    if IsValidConflictData(pConflictData) == 0 {
        return CR_INVALID_CONFLICT_LIST;
    }

    if pulCount.is_null() {
        return CR_INVALID_POINTER;
    }

    *pulCount = (*(*pConflictData).pConflictList).ConflictsListed;

    return CR_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_Resource_Conflict_DetailsA(
    clConflictList: CONFLICT_LIST,
    ulIndex: ULONG,
    pConflictDetails: PCONFLICT_DETAILS_A,
) -> CONFIGRET {
    // FIXME("CM_Get_Resource_Conflict_CountA(%p %lu %p)\n", clConflictList, ulIndex, pConflictDetails);
    CR_CALL_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_Resource_Conflict_DetailsW(
    clConflictList: CONFLICT_LIST,
    ulIndex: ULONG,
    pConflictDetails: PCONFLICT_DETAILS_W,
) -> CONFIGRET {
    // FIXME("CM_Get_Resource_Conflict_CountW(%p %lu %p)\n", clConflictList, ulIndex, pConflictDetails);
    CR_CALL_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_Sibling(
    pdnDevInst: PDEVINST,
    dnDevInst: DEVINST,
    ulFlags: ULONG,
) -> CONFIGRET {
    return CM_Get_Sibling_Ex(pdnDevInst, dnDevInst, ulFlags, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_Sibling_Ex(
    pdnDevInst: PDEVINST,
    dnDevInst: DEVINST,
    ulFlags: ULONG,
    hMachine: HMACHINE,
) -> CONFIGRET {
    let mut szRelatedDevInst: [WCHAR; (MAX_DEVICE_ID_LEN as _)] = [0; (MAX_DEVICE_ID_LEN as _)];
    let mut BindingHandle: RPC_BINDING_HANDLE = core::ptr::null_mut();
    let mut StringTable: HSTRING_TABLE = core::ptr::null_mut();
    let mut lpDevInst: LPWSTR = core::ptr::null_mut();
    let mut dwIndex: DWORD = 0;
    let mut dwLength: DWORD = MAX_DEVICE_ID_LEN as DWORD;
    let mut ret: CONFIGRET = 0;

    if pdnDevInst.is_null() {
        return CR_INVALID_POINTER as CONFIGRET;
    }

    if dnDevInst == 0 as DEVINST {
        return CR_INVALID_DEVINST as CONFIGRET;
    }

    if ulFlags != 0 as ULONG {
        return CR_INVALID_FLAG as CONFIGRET;
    }

    *pdnDevInst = -(1 as ::core::ffi::c_int) as DEVNODE;

    if !hMachine.is_null() {
        BindingHandle = (*(hMachine as PMACHINE_INFO)).BindingHandle;
        if BindingHandle.is_null() {
            return CR_FAILURE as CONFIGRET;
        }

        StringTable = (*(hMachine as PMACHINE_INFO)).StringTable;
        if StringTable == 0 as HSTRING_TABLE {
            return CR_FAILURE as CONFIGRET;
        }
    } else {
        if PnpGetLocalHandles(
            core::ptr::addr_of_mut!(BindingHandle),
            core::ptr::addr_of_mut!(StringTable),
        ) == 0
        {
            return CR_FAILURE as CONFIGRET;
        }
    }

    lpDevInst = pSetupStringTableStringFromId(StringTable, dnDevInst as DWORD);
    if lpDevInst.is_null() {
        return CR_INVALID_DEVNODE as CONFIGRET;
    }

    // RpcTryExcept
    ret = rpc_try(|| PNP_GetRelatedDeviceInstance(
        BindingHandle as handle_t,
        PNP_GET_SIBLING_DEVICE_INSTANCE as DWORD,
        lpDevInst,
        core::ptr::addr_of_mut!(szRelatedDevInst) as LPWSTR,
        core::ptr::addr_of_mut!(dwLength),
        0 as DWORD,
    )).unwrap_or_else(|code| RpcStatusToCmStatus(code as _) as _) as CONFIGRET;
    // RpcExcept(EXCEPTION_EXECUTE_HANDLER)
    // RpcEndExcept;

    if ret != CR_SUCCESS as CONFIGRET {
        return ret;
    }

    dwIndex = pSetupStringTableAddString(
        StringTable,
        core::ptr::addr_of_mut!(szRelatedDevInst) as LPWSTR,
        1 as DWORD,
    );
    if dwIndex == -(1 as ::core::ffi::c_int) as DWORD {
        return CR_FAILURE as CONFIGRET;
    }

    *pdnDevInst = dwIndex as DEVNODE;

    return CR_SUCCESS as CONFIGRET;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_Version() -> WORD {
    return CM_Get_Version_Ex(core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_Version_Ex(hMachine: HMACHINE) -> WORD {
    let mut BindingHandle: RPC_BINDING_HANDLE = core::ptr::null_mut();
    let mut Version: WORD = 0;
    let mut ret: CONFIGRET = 0;

    if !hMachine.is_null() {
        BindingHandle = (*(hMachine as PMACHINE_INFO)).BindingHandle;
        if BindingHandle.is_null() {
            return 0;
        }
    } else {
        if PnpGetLocalHandles(
            core::ptr::addr_of_mut!(BindingHandle),
            core::ptr::null_mut(),
        ) == 0
        {
            return (CR_FAILURE) as WORD;
        }
    }

    // RpcTryExcept
    ret = rpc_try(|| PNP_GetVersion(
        BindingHandle as handle_t,
        core::ptr::addr_of_mut!(Version),
    )).unwrap_or_else(|code| RpcStatusToCmStatus(code as _) as _) as CONFIGRET;
    // RpcExcept(EXCEPTION_EXECUTE_HANDLER)
    // RpcEndExcept;

    if ret != CR_SUCCESS {
        return 0;
    }

    return Version;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Intersect_Range_List(
    rlhOld1: RANGE_LIST,
    rlhOld2: RANGE_LIST,
    rlhNew: RANGE_LIST,
    ulFlags: ULONG,
) -> CONFIGRET {
    // FIXME("CM_Intersect_Range_List(%p %p %p %lx)\n", rlhOld1, rlhOld2, rlhNew, ulFlags);
    CR_CALL_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn CM_Invert_Range_List(
    rlhOld: RANGE_LIST,
    rlhNew: RANGE_LIST,
    ullMaxValue: DWORDLONG,
    ulFlags: ULONG,
) -> CONFIGRET {
    // FIXME("CM_Invert_Range_List(%p %p %I64u %lx)\n", rlhOld, rlhNew, ullMaxValue, ulFlags);
    CR_CALL_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn CM_Is_Dock_Station_Present(pbPresent: PBOOL) -> CONFIGRET {
    TRACE!("CM_Is_Dock_Station_Present(%p)\n", pbPresent);
    return CM_Is_Dock_Station_Present_Ex(pbPresent, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn CM_Is_Dock_Station_Present_Ex(
    pbPresent: PBOOL,
    hMachine: HMACHINE,
) -> CONFIGRET {
    let mut BindingHandle: RPC_BINDING_HANDLE = core::ptr::null_mut();
    let mut ret: CONFIGRET = 0;
    if pbPresent.is_null() {
        return CR_INVALID_POINTER;
    }
    *pbPresent = (FALSE as i32);
    if !hMachine.is_null() {
        BindingHandle = (*(hMachine as PMACHINE_INFO)).BindingHandle;
        if BindingHandle.is_null() {
            return CR_FAILURE;
        }
    } else if PnpGetLocalHandles(
        core::ptr::addr_of_mut!(BindingHandle),
        core::ptr::null_mut(),
    ) == 0
    {
        return CR_FAILURE;
    }
    ret = rpc_try(|| PNP_IsDockStationPresent(BindingHandle as handle_t, pbPresent as *mut BOOL)).unwrap_or_else(|code| RpcStatusToCmStatus(code as _) as _) as CONFIGRET;
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Is_Version_Available(wVersion: WORD) -> BOOL {
    TRACE!("CM_Is_Version_Available(%hu)\n", wVersion as core::ffi::c_uint);
    return CM_Is_Version_Available_Ex(wVersion, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn CM_Is_Version_Available_Ex(
    mut wVersion: WORD,
    mut hMachine: HMACHINE,
) -> BOOL {
    let mut BindingHandle: RPC_BINDING_HANDLE = core::ptr::null_mut();
    let mut wServerVersion: WORD = 0;
    let mut ret: CONFIGRET = 0;

    if wVersion as ::core::ffi::c_int <= 0x400 {
        return (TRUE) as BOOL;
    }
    if !hMachine.is_null() {
        BindingHandle = (*(hMachine as PMACHINE_INFO)).BindingHandle;
        if BindingHandle.is_null() {
            return (FALSE) as BOOL;
        }
    } else if PnpGetLocalHandles(
        &raw mut BindingHandle,
        core::ptr::null_mut::<HSTRING_TABLE>(),
    ) == 0
    {
        return (FALSE) as BOOL;
    }

    ret = rpc_try(|| PNP_GetVersion(BindingHandle as handle_t, &raw mut wServerVersion)).unwrap_or_else(|code| RpcStatusToCmStatus(code as _) as _) as CONFIGRET;

    if ret != CR_SUCCESS {
        return (FALSE) as BOOL;
    }

    return (wServerVersion as ::core::ffi::c_int >= wVersion as ::core::ffi::c_int) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Locate_DevNodeA(pdnDevInst: PDEVINST, pDeviceID: DEVINSTID_A, ulFlags: ULONG) -> CONFIGRET {
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE) as i32;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/setupapi/cfgmgr.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"CM_Locate_DevNodeA\0" as *const u8 as *const ::core::ffi::c_char,
            6017 as ::core::ffi::c_int,
            b"CM_Locate_DevNodeA(%p %s %lx)\n\0" as *const u8 as *const ::core::ffi::c_char,
            pdnDevInst,
            debugstr_a(pDeviceID as *const ::core::ffi::c_char),
            ulFlags,
        );
    }
    return CM_Locate_DevNode_ExA(pdnDevInst, pDeviceID, ulFlags, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn CM_Locate_DevNodeW(pdnDevInst: PDEVINST, pDeviceID: DEVINSTID_W, ulFlags: ULONG) -> CONFIGRET {
    if ((*__wine_dbch___default).flags as i32 & ((1i32) << (__WINE_DBCL_TRACE as i32))) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE) as i32;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/setupapi/cfgmgr.c\0".as_ptr() as *const i8,
            b"CM_Locate_DevNodeW\0".as_ptr() as *const i8,
            6034i32,
            b"CM_Locate_DevNodeW(%p %s %lx)\n\0".as_ptr() as *const i8,
            pdnDevInst,
            debugstr_w(pDeviceID as *const WCHAR),
            ulFlags,
        );
    }
    return CM_Locate_DevNode_ExW(pdnDevInst, pDeviceID, ulFlags, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn CM_Locate_DevNode_ExA(
    pdnDevInst: PDEVINST,
    pDeviceID: DEVINSTID_A,
    ulFlags: ULONG,
    hMachine: HMACHINE,
) -> CONFIGRET {
    let mut pDevIdW: DEVINSTID_W = core::ptr::null_mut();
    let mut ret: CONFIGRET = CR_SUCCESS;

    TRACE!(
        "CM_Locate_DevNode_ExA(%p %s %lx %p)\n",
        pdnDevInst,
        debugstr_a(pDeviceID as *const i8),
        ulFlags,
        hMachine
    );

    if !pDeviceID.is_null() {
        if pSetupCaptureAndConvertAnsiArg(pDeviceID as PCSTR, core::ptr::addr_of_mut!(pDevIdW)) != 0 {
            return CR_INVALID_DEVICE_ID;
        }
    }

    ret = CM_Locate_DevNode_ExW(pdnDevInst, pDevIdW, ulFlags, hMachine);

    if !pDevIdW.is_null() {
        MyFree(pDevIdW as PVOID);
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Locate_DevNode_ExW(
    pdnDevInst: PDEVINST,
    pDeviceID: DEVINSTID_W,
    ulFlags: ULONG,
    hMachine: HMACHINE,
) -> CONFIGRET {
    let mut DeviceIdBuffer: [WCHAR; (MAX_DEVICE_ID_LEN as _)] = [0; (MAX_DEVICE_ID_LEN as _)];
    let mut BindingHandle: RPC_BINDING_HANDLE = NULL;
    let mut StringTable: HSTRING_TABLE = NULL;
    let mut ret: CONFIGRET = CR_SUCCESS as CONFIGRET;

    TRACE!(
        "CM_Locate_DevNode_ExW(%p %s %lx %p)\n",
        pdnDevInst,
        debugstr_w(pDeviceID as *const WCHAR),
        ulFlags,
        hMachine
    );

    if pdnDevInst.is_null() {
        return CR_INVALID_POINTER as CONFIGRET;
    }

    if (ulFlags & !(CM_LOCATE_DEVNODE_BITS as ULONG)) != 0 {
        return CR_INVALID_FLAG as CONFIGRET;
    }

    if !hMachine.is_null() {
        BindingHandle = (*(hMachine as PMACHINE_INFO)).BindingHandle;
        if BindingHandle.is_null() {
            return CR_FAILURE as CONFIGRET;
        }

        StringTable = (*(hMachine as PMACHINE_INFO)).StringTable;
        if StringTable == core::ptr::null_mut() {
            return CR_FAILURE as CONFIGRET;
        }
    } else {
        if PnpGetLocalHandles(&raw mut BindingHandle, &raw mut StringTable) == 0 {
            return CR_FAILURE as CONFIGRET;
        }
    }

    if !pDeviceID.is_null()
        && lstrlenW(pDeviceID as LPCWSTR) != (0 as ::core::ffi::c_int)
    {
        lstrcpyW(&raw mut DeviceIdBuffer as LPWSTR, pDeviceID as LPCWSTR);

        /* Validate the device ID */
        ret = rpc_try(|| PNP_ValidateDeviceInstance(
            BindingHandle as handle_t,
            &raw mut DeviceIdBuffer as LPWSTR,
            ulFlags as DWORD,
        )).unwrap_or_else(|code| RpcStatusToCmStatus(code as _) as _) as CONFIGRET;
    } else {
        /* Get the root device ID */
        ret = rpc_try(|| PNP_GetRootDeviceInstance(
            BindingHandle as handle_t,
            &raw mut DeviceIdBuffer as LPWSTR,
            MAX_DEVICE_ID_LEN as PNP_RPC_STRING_LEN,
        )).unwrap_or_else(|code| RpcStatusToCmStatus(code as _) as _) as CONFIGRET;
    }

    TRACE!(
        "DeviceIdBuffer: %s\n",
        debugstr_w(&raw mut DeviceIdBuffer as *mut WCHAR)
    );

    if ret == CR_SUCCESS as CONFIGRET {
        *pdnDevInst = pSetupStringTableAddString(
            StringTable,
            &raw mut DeviceIdBuffer as LPWSTR,
            1 as DWORD,
        ) as DEVNODE;
        if *pdnDevInst == (-1i32) as DEVNODE {
            ret = CR_FAILURE as CONFIGRET;
        }
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Merge_Range_List(
    rlhOld1: RANGE_LIST,
    rlhOld2: RANGE_LIST,
    rlhNew: RANGE_LIST,
    ulFlags: ULONG,
) -> CONFIGRET {
    // FIXME("CM_Merge_Range_List(%p %p %p %lx)\n", rlhOld1, rlhOld2, rlhNew, ulFlags);
    CR_CALL_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn CM_Modify_Res_Des(
    prdResDes: PRES_DES,
    rdResDes: RES_DES,
    ResourceID: RESOURCEID,
    ResourceData: PCVOID,
    ResourceLen: ULONG,
    ulFlags: ULONG,
) -> CONFIGRET {
    TRACE!(
        "CM_Modify_Res_Des(%p %p %lx %p %lu %lx)\n",
        prdResDes,
        rdResDes,
        ResourceID,
        ResourceData,
        ResourceLen,
        ulFlags
    );
    return CM_Modify_Res_Des_Ex(
        prdResDes,
        rdResDes,
        ResourceID,
        ResourceData,
        ResourceLen,
        ulFlags,
        core::ptr::null_mut(),
    );
}

#[no_mangle]
pub unsafe extern "system" fn CM_Modify_Res_Des_Ex(
    prdResDes: PRES_DES,
    rdResDes: RES_DES,
    ResourceID: RESOURCEID,
    ResourceData: PCVOID,
    ResourceLen: ULONG,
    ulFlags: ULONG,
    hMachine: HMACHINE,
) -> CONFIGRET {
    // FIXME("CM_Modify_Res_Des_Ex(%p %p %lx %p %lu %lx %p)\n", prdResDes, rdResDes, ResourceID, ResourceData, ResourceLen, ulFlags, hMachine);
    CR_CALL_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn CM_Move_DevNode(
    dnFromDevInst: DEVINST,
    dnToDevInst: DEVINST,
    ulFlags: ULONG,
) -> CONFIGRET {
    return CM_Move_DevNode_Ex(dnFromDevInst, dnToDevInst, ulFlags, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn CM_Move_DevNode_Ex(mut dnFromDevInst: DEVINST, mut dnToDevInst: DEVINST, mut ulFlags: ULONG, mut hMachine: HMACHINE) -> CONFIGRET {
    let mut BindingHandle: RPC_BINDING_HANDLE = core::ptr::null_mut();
    let mut StringTable: HSTRING_TABLE = core::ptr::null_mut();
    let mut lpFromDevInst: LPWSTR = core::ptr::null_mut::<WCHAR>();
    let mut lpToDevInst: LPWSTR = core::ptr::null_mut::<WCHAR>();
    let mut ret: CONFIGRET = 0;

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/setupapi/cfgmgr.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"CM_Move_DevNode_Ex\0" as *const u8 as *const ::core::ffi::c_char,
            6256 as ::core::ffi::c_int,
            b"CM_Move_DevNode_Ex(%lx %lx %lx %p)\n\0" as *const u8 as *const ::core::ffi::c_char,
            dnFromDevInst,
            dnToDevInst,
            ulFlags,
            hMachine,
        );
    }

    if pSetupIsUserAdmin() == 0 {
        return CR_ACCESS_DENIED;
    }

    if dnFromDevInst == 0 || dnToDevInst == 0 {
        return CR_INVALID_DEVNODE;
    }

    if ulFlags != 0 {
        return CR_INVALID_FLAG;
    }

    if !hMachine.is_null() {
        BindingHandle = (*(hMachine as PMACHINE_INFO)).BindingHandle;
        if BindingHandle.is_null() {
            return CR_FAILURE;
        }

        StringTable = (*(hMachine as PMACHINE_INFO)).StringTable;
        if StringTable == core::ptr::null_mut() {
            return CR_FAILURE;
        }
    } else {
        if PnpGetLocalHandles(
            core::ptr::addr_of_mut!(BindingHandle),
            core::ptr::addr_of_mut!(StringTable),
        ) == 0
        {
            return CR_FAILURE;
        }
    }

    lpFromDevInst = pSetupStringTableStringFromId(StringTable, dnFromDevInst as DWORD);
    if lpFromDevInst.is_null() {
        return CR_INVALID_DEVNODE;
    }

    lpToDevInst = pSetupStringTableStringFromId(StringTable, dnToDevInst as DWORD);
    if lpToDevInst.is_null() {
        return CR_INVALID_DEVNODE;
    }

    ret = rpc_try(|| PNP_DeviceInstanceAction(
        BindingHandle as handle_t,
        PNP_DEVINST_MOVE as DWORD,
        ulFlags as DWORD,
        lpFromDevInst,
        lpToDevInst,
    )).unwrap_or_else(|code| RpcStatusToCmStatus(code as _) as _) as CONFIGRET;


    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Next_Range(
    preElement: PRANGE_ELEMENT,
    pullStart: PDWORDLONG,
    pullEnd: PDWORDLONG,
    ulFlags: ULONG,
) -> CONFIGRET {
    let mut pRangeList: PINTERNAL_RANGE_LIST = core::ptr::null_mut();
    let mut pRange: PINTERNAL_RANGE = core::ptr::null_mut();
    let mut ListEntry: PLIST_ENTRY = core::ptr::null_mut();
    let mut ret: CONFIGRET = CR_SUCCESS;

    pRange = preElement as PINTERNAL_RANGE;

    if pRange.is_null() || (*pRange).pRangeList.is_null() {
        return CR_FAILURE;
    }

    if pullStart.is_null() || pullEnd.is_null() {
        return CR_INVALID_POINTER;
    }

    if ulFlags != 0 {
        return CR_INVALID_FLAG;
    }

    pRangeList = (*pRange).pRangeList as PINTERNAL_RANGE_LIST;

    /* Lock the range list */
    WaitForSingleObject((*pRangeList).hMutex, INFINITE);

    'done: {
        /* Fail, if we reached the end of the list */
        if (*pRange).ListEntry.Flink == core::ptr::addr_of_mut!((*pRangeList).ListHead) {
            ret = CR_FAILURE;
            break 'done;
        }

        /* Get the next range */
        ListEntry = (*pRangeList).ListHead.Flink as PLIST_ENTRY;
        pRange = ((ListEntry as ULONG_PTR).wrapping_sub(
            core::mem::offset_of!(INTERNAL_RANGE, ListEntry) as ULONG_PTR,
        )) as *mut INTERNAL_RANGE as PINTERNAL_RANGE;

        /* Return the range data */
        *pullStart = (*pRange).ullStart;
        *pullEnd = (*pRange).ullEnd;
        *preElement = pRange as RANGE_ELEMENT;
    }

    /* Unlock the range list */
    ReleaseMutex((*pRangeList).hMutex);

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Open_Class_KeyA(
    pClassGuid: LPGUID,
    pszClassName: LPCSTR,
    samDesired: REGSAM,
    Disposition: REGDISPOSITION,
    phkClass: PHKEY,
    ulFlags: ULONG,
) -> CONFIGRET {
    return CM_Open_Class_Key_ExA(pClassGuid, pszClassName, samDesired,
                                 Disposition, phkClass, ulFlags, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn CM_Open_Class_KeyW(
    pClassGuid: LPGUID,
    pszClassName: LPCWSTR,
    samDesired: REGSAM,
    Disposition: REGDISPOSITION,
    phkClass: PHKEY,
    ulFlags: ULONG,
) -> CONFIGRET {
    return CM_Open_Class_Key_ExW(pClassGuid, pszClassName, samDesired,
        Disposition, phkClass, ulFlags, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn CM_Open_Class_Key_ExA(
    pClassGuid: LPGUID,
    pszClassName: LPCSTR,
    samDesired: REGSAM,
    Disposition: REGDISPOSITION,
    phkClass: PHKEY,
    ulFlags: ULONG,
    hMachine: HMACHINE,
) -> CONFIGRET {
    let mut pszClassNameW: LPWSTR = core::ptr::null_mut();
    let mut ret: CONFIGRET = 0;

    if !pszClassName.is_null() {
        if pSetupCaptureAndConvertAnsiArg(pszClassName as PCSTR, core::ptr::addr_of_mut!(pszClassNameW)) != 0 {
            return CR_INVALID_DATA;
        }
    }

    ret = CM_Open_Class_Key_ExW(
        pClassGuid,
        pszClassNameW as LPCWSTR,
        samDesired,
        Disposition,
        phkClass,
        ulFlags,
        hMachine,
    );

    if !pszClassNameW.is_null() {
        MyFree(pszClassNameW as PVOID);
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Open_Class_Key_ExW(
    pClassGuid: LPGUID,
    pszClassName: LPCWSTR,
    samDesired: REGSAM,
    Disposition: REGDISPOSITION,
    phkClass: PHKEY,
    ulFlags: ULONG,
    hMachine: HMACHINE,
) -> CONFIGRET {
    let mut szKeyName: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut lpGuidString: LPWSTR = core::ptr::null_mut();
    let mut dwDisposition: DWORD = 0;
    let mut dwError: DWORD = 0;
    let mut hKey: HKEY = core::ptr::null_mut();

    TRACE!(
        "CM_Open_Class_Key_ExW(%p %s %lx %lx %p %lx %p)\n",
        debugstr_guid(pClassGuid as *const _GUID),
        debugstr_w(pszClassName as *const WCHAR),
        samDesired,
        Disposition,
        phkClass,
        ulFlags,
        hMachine
    );

    /* Check Disposition and ulFlags */
    if (Disposition & !RegDisposition_Bits as REGDISPOSITION) != 0
        || (ulFlags & !CM_OPEN_CLASS_KEY_BITS as ULONG) != 0
    {
        return CR_INVALID_FLAG as CONFIGRET;
    }

    /* Check phkClass */
    if phkClass.is_null() {
        return CR_INVALID_POINTER as CONFIGRET;
    }

    *phkClass = core::ptr::null_mut();

    if ulFlags == CM_OPEN_CLASS_KEY_INTERFACE as ULONG && !pszClassName.is_null() {
        return CR_INVALID_DATA as CONFIGRET;
    }

    if hMachine.is_null() {
        hKey = HKEY_LOCAL_MACHINE as HKEY;
    } else {
        if RegConnectRegistryW(
            &raw mut (*(hMachine as PMACHINE_INFO)).szMachineName as *mut WCHAR as LPCWSTR,
            HKEY_LOCAL_MACHINE as HKEY,
            &raw mut hKey,
        ) != 0
        {
            return CR_REGISTRY_ERROR as CONFIGRET;
        }
    }

    if ulFlags & CM_OPEN_CLASS_KEY_INTERFACE as ULONG != 0 {
        lstrcpyW(
            &raw mut szKeyName as *mut WCHAR as LPWSTR,
            &raw const DeviceClasses as LPCWSTR,
        );
    } else {
        lstrcpyW(
            &raw mut szKeyName as *mut WCHAR as LPWSTR,
            &raw const ControlClass as LPCWSTR,
        );
    }

    if !pClassGuid.is_null() {
        if UuidToStringW(pClassGuid as *mut UUID, &raw mut lpGuidString)
            != RPC_S_OK as RPC_STATUS
        {
            RegCloseKey(hKey);
            return CR_INVALID_DATA as CONFIGRET;
        }

        lstrcatW(
            &raw mut szKeyName as *mut WCHAR as LPWSTR,
            &raw const BackslashOpenBrace as LPCWSTR,
        );
        lstrcatW(
            &raw mut szKeyName as *mut WCHAR as LPWSTR,
            lpGuidString as LPCWSTR,
        );
        lstrcatW(
            &raw mut szKeyName as *mut WCHAR as LPWSTR,
            &raw const CloseBrace as LPCWSTR,
        );
    }

    if Disposition == RegDisposition_OpenAlways as REGDISPOSITION {
        dwError = RegCreateKeyExW(
            hKey,
            &raw mut szKeyName as *mut WCHAR as LPCWSTR,
            0 as DWORD,
            core::ptr::null_mut::<WCHAR>(),
            0 as DWORD,
            samDesired,
            core::ptr::null_mut::<_SECURITY_ATTRIBUTES>(),
            phkClass,
            &raw mut dwDisposition,
        ) as DWORD;
    } else {
        dwError = RegOpenKeyExW(
            hKey,
            &raw mut szKeyName as *mut WCHAR as LPCWSTR,
            0 as DWORD,
            samDesired,
            phkClass,
        ) as DWORD;
    }

    RegCloseKey(hKey);

    if !pClassGuid.is_null() {
        RpcStringFreeW(&raw mut lpGuidString);
    }

    if dwError != ERROR_SUCCESS as DWORD {
        *phkClass = core::ptr::null_mut();
        return CR_NO_SUCH_REGISTRY_KEY as CONFIGRET;
    }

    if !pszClassName.is_null() {
        RegSetValueExW(
            *phkClass,
            &raw const Class as LPCWSTR,
            0 as DWORD,
            REG_SZ as DWORD,
            pszClassName as LPBYTE as *const BYTE,
            ((lstrlenW(pszClassName) + 1) as usize
                * core::mem::size_of::<WCHAR>()) as DWORD,
        );
    }

    return CR_SUCCESS as CONFIGRET;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Open_DevNode_Key(
    dnDevNode: DEVINST,
    samDesired: REGSAM,
    ulHardwareProfile: ULONG,
    Disposition: REGDISPOSITION,
    phkDevice: PHKEY,
    ulFlags: ULONG,
) -> CONFIGRET {
    return CM_Open_DevNode_Key_Ex(dnDevNode, samDesired, ulHardwareProfile,
                                  Disposition, phkDevice, ulFlags, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn CM_Open_DevNode_Key_Ex(
    dnDevNode: DEVINST,
    samDesired: REGSAM,
    ulHardwareProfile: ULONG,
    Disposition: REGDISPOSITION,
    phkDevice: PHKEY,
    ulFlags: ULONG,
    hMachine: HMACHINE,
) -> CONFIGRET {
    let mut BindingHandle: RPC_BINDING_HANDLE = core::ptr::null_mut();
    let mut StringTable: HSTRING_TABLE = core::ptr::null_mut();
    let mut pszDevInst: LPWSTR = core::ptr::null_mut();
    let mut pszKeyPath: LPWSTR = core::ptr::null_mut();
    let mut pszInstancePath: LPWSTR = core::ptr::null_mut();
    let mut lError: LONG;
    let mut dwDisposition: DWORD = 0;
    let mut hRootKey: HKEY = core::ptr::null_mut();
    let mut ret: CONFIGRET = CR_CALL_NOT_IMPLEMENTED;

    if phkDevice.is_null() {
        return CR_INVALID_POINTER;
    }

    *phkDevice = core::ptr::null_mut();

    if dnDevNode == 0 {
        return CR_INVALID_DEVNODE;
    }

    if (ulFlags & !CM_REGISTRY_BITS) != 0 {
        return CR_INVALID_FLAG;
    }

    if (Disposition & !RegDisposition_Bits) != 0 {
        return CR_INVALID_DATA;
    }

    if !hMachine.is_null() {
        BindingHandle = (*(hMachine as PMACHINE_INFO)).BindingHandle;
        if BindingHandle.is_null() {
            return CR_FAILURE;
        }

        StringTable = (*(hMachine as PMACHINE_INFO)).StringTable;
        if StringTable == core::ptr::null_mut() {
            return CR_FAILURE;
        }
    } else {
        if PnpGetLocalHandles(core::ptr::addr_of_mut!(BindingHandle), core::ptr::addr_of_mut!(StringTable)) == 0 {
            return CR_FAILURE;
        }
    }

    pszDevInst = pSetupStringTableStringFromId(StringTable, dnDevNode);
    if pszDevInst.is_null() {
        return CR_INVALID_DEVNODE;
    }

    'done: {
        pszKeyPath = MyMalloc(
            (512 as usize * core::mem::size_of::<u16>()) as DWORD,
        ) as LPWSTR;
        if pszKeyPath.is_null() {
            ret = CR_OUT_OF_MEMORY;
            break 'done;
        }

        pszInstancePath = MyMalloc(
            (512 as usize * core::mem::size_of::<u16>()) as DWORD,
        ) as LPWSTR;
        if pszInstancePath.is_null() {
            ret = CR_OUT_OF_MEMORY;
            break 'done;
        }

        ret = GetDeviceInstanceKeyPath(
            BindingHandle,
            pszDevInst as PWSTR,
            pszKeyPath as PWSTR,
            pszInstancePath as PWSTR,
            ulHardwareProfile,
            ulFlags,
        );
        if ret != CR_SUCCESS {
            break 'done;
        }

        wcscat(pszKeyPath as *mut wchar_t, RUSTCONV_WSTR_B7FC02907B1F.as_ptr());
        wcscat(pszKeyPath as *mut wchar_t, pszInstancePath as *const wchar_t);

        if hMachine.is_null() {
            hRootKey = HKEY_LOCAL_MACHINE;
        } else {
            if RegConnectRegistryW(
                core::ptr::addr_of_mut!((*(hMachine as PMACHINE_INFO)).szMachineName) as *mut WCHAR as LPCWSTR,
                HKEY_LOCAL_MACHINE,
                core::ptr::addr_of_mut!(hRootKey),
            ) != 0
            {
                ret = CR_REGISTRY_ERROR;
                break 'done;
            }
        }

        if Disposition == RegDisposition_OpenAlways {
            lError = RegCreateKeyExW(
                hRootKey,
                pszKeyPath as LPCWSTR,
                0,
                core::ptr::null_mut(),
                0,
                samDesired,
                core::ptr::null_mut(),
                phkDevice,
                core::ptr::addr_of_mut!(dwDisposition),
            );
        } else {
            lError = RegOpenKeyExW(
                hRootKey,
                pszKeyPath as LPCWSTR,
                0,
                samDesired,
                phkDevice,
            );
        }

        if lError != (ERROR_SUCCESS as i32) {
            *phkDevice = core::ptr::null_mut();
            ret = CR_NO_SUCH_REGISTRY_KEY;
        }
    }

    // done:
    if (!hRootKey.is_null()) && (hRootKey != HKEY_LOCAL_MACHINE) {
        RegCloseKey(hRootKey);
    }

    if !pszInstancePath.is_null() {
        MyFree(pszInstancePath as PVOID);
    }

    if !pszKeyPath.is_null() {
        MyFree(pszKeyPath as PVOID);
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Query_And_Remove_SubTreeA(
    dnAncestor: DEVINST,
    pVetoType: PPNP_VETO_TYPE,
    pszVetoName: LPSTR,
    ulNameLength: ULONG,
    ulFlags: ULONG,
) -> CONFIGRET {
    TRACE!(
        "CM_Query_And_Remove_SubTreeA(%lx %p %p %lu %lx)\n",
        dnAncestor,
        pVetoType,
        pszVetoName,
        ulNameLength,
        ulFlags
    );
    return CM_Query_And_Remove_SubTree_ExA(
        dnAncestor,
        pVetoType,
        pszVetoName,
        ulNameLength,
        ulFlags,
        core::ptr::null_mut(),
    );
}

#[no_mangle]
pub unsafe extern "system" fn CM_Query_And_Remove_SubTreeW(
    dnAncestor: DEVINST,
    pVetoType: PPNP_VETO_TYPE,
    pszVetoName: LPWSTR,
    ulNameLength: ULONG,
    ulFlags: ULONG,
) -> CONFIGRET {
    return CM_Query_And_Remove_SubTree_ExW(
        dnAncestor,
        pVetoType,
        pszVetoName,
        ulNameLength,
        ulFlags,
        core::ptr::null_mut(),
    );
}

#[no_mangle]
pub unsafe extern "system" fn CM_Query_And_Remove_SubTree_ExA(
    dnAncestor: DEVINST,
    pVetoType: PPNP_VETO_TYPE,
    pszVetoName: LPSTR,
    ulNameLength: ULONG,
    ulFlags: ULONG,
    hMachine: HMACHINE,
) -> CONFIGRET {
    let mut lpLocalVetoName: LPWSTR = core::ptr::null_mut::<WCHAR>();
    let mut ret: CONFIGRET = 0;

    TRACE!(
        "CM_Query_And_Remove_SubTree_ExA(%lx %p %p %lu %lx %p)\n",
        dnAncestor,
        pVetoType,
        pszVetoName,
        ulNameLength,
        ulFlags,
        hMachine
    );

    if pszVetoName.is_null() && ulNameLength == 0 {
        return CR_INVALID_POINTER;
    }

    lpLocalVetoName = HeapAlloc(
        GetProcessHeap(),
        0,
        (ulNameLength as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
    ) as LPWSTR;
    if lpLocalVetoName.is_null() {
        return CR_OUT_OF_MEMORY;
    }

    ret = CM_Query_And_Remove_SubTree_ExW(
        dnAncestor,
        pVetoType,
        lpLocalVetoName,
        ulNameLength,
        ulFlags,
        hMachine,
    );
    if ret == CR_REMOVE_VETOED {
        if WideCharToMultiByte(
            CP_ACP,
            0,
            lpLocalVetoName as LPCWSTR,
            ulNameLength as ::core::ffi::c_int,
            pszVetoName,
            ulNameLength as ::core::ffi::c_int,
            core::ptr::null::<CHAR>(),
            core::ptr::null_mut::<BOOL>(),
        ) == 0
        {
            ret = CR_FAILURE;
        }
    }

    HeapFree(
        GetProcessHeap(),
        0,
        lpLocalVetoName as PVOID,
    );

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Query_And_Remove_SubTree_ExW(
    mut dnAncestor: DEVINST,
    mut pVetoType: PPNP_VETO_TYPE,
    mut pszVetoName: LPWSTR,
    mut ulNameLength: ULONG,
    mut ulFlags: ULONG,
    mut hMachine: HMACHINE,
) -> CONFIGRET {
    let mut BindingHandle: RPC_BINDING_HANDLE = NULL;
    let mut StringTable: HSTRING_TABLE = NULL;
    let mut lpDevInst: LPWSTR = core::ptr::null_mut();
    let mut ret: CONFIGRET;

    if dnAncestor == 0 {
        return CR_INVALID_DEVNODE;
    }

    if ulFlags & !CM_REMOVE_BITS != 0 {
        return CR_INVALID_FLAG;
    }

    if pszVetoName.is_null() && ulNameLength == 0 {
        return CR_INVALID_POINTER;
    }

    if !hMachine.is_null() {
        BindingHandle = (*(hMachine as PMACHINE_INFO)).BindingHandle;
        if BindingHandle.is_null() {
            return CR_FAILURE;
        }

        StringTable = (*(hMachine as PMACHINE_INFO)).StringTable;
        if StringTable == core::ptr::null_mut() {
            return CR_FAILURE;
        }
    } else {
        if PnpGetLocalHandles(
            core::ptr::addr_of_mut!(BindingHandle),
            core::ptr::addr_of_mut!(StringTable),
        ) == 0
        {
            return CR_FAILURE;
        }
    }

    lpDevInst = pSetupStringTableStringFromId(StringTable, dnAncestor);
    if lpDevInst.is_null() {
        return CR_INVALID_DEVNODE;
    }

    ret = rpc_try(|| PNP_QueryRemove(
        BindingHandle,
        lpDevInst,
        pVetoType,
        pszVetoName,
        ulNameLength,
        ulFlags,
    )).unwrap_or_else(|code| RpcStatusToCmStatus(code as _) as _);

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Query_Arbitrator_Free_Data(
    pData: PVOID,
    DataLen: ULONG,
    dnDevInst: DEVINST,
    ResourceID: RESOURCEID,
    ulFlags: ULONG,
) -> CONFIGRET {
    TRACE!(
        "CM_Query_Arbitrator_Free_Data(%p %lu %lx %lu 0x%08lx)\n",
        pData,
        DataLen,
        dnDevInst,
        ResourceID,
        ulFlags
    );
    return CM_Query_Arbitrator_Free_Data_Ex(pData, DataLen, dnDevInst, ResourceID, ulFlags, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn CM_Query_Arbitrator_Free_Data_Ex(
    pData: PVOID,
    DataLen: ULONG,
    dnDevInst: DEVINST,
    ResourceID: RESOURCEID,
    ulFlags: ULONG,
    hMachine: HMACHINE,
) -> CONFIGRET {
    let mut BindingHandle: RPC_BINDING_HANDLE = core::ptr::null_mut();
    let mut StringTable: HSTRING_TABLE = core::ptr::null_mut();
    let mut lpDevInst: LPWSTR = core::ptr::null_mut();
    let mut ret: CONFIGRET;

    if pData.is_null() || DataLen == 0 {
        return CR_INVALID_POINTER;
    }

    if dnDevInst == 0 {
        return CR_INVALID_DEVINST;
    }

    if ulFlags & !CM_QUERY_ARBITRATOR_BITS != 0 {
        return CR_INVALID_FLAG;
    }

    if !hMachine.is_null() {
        BindingHandle = (*(hMachine as PMACHINE_INFO)).BindingHandle;
        if BindingHandle.is_null() {
            return CR_FAILURE;
        }

        StringTable = (*(hMachine as PMACHINE_INFO)).StringTable;
        if StringTable == core::ptr::null_mut() {
            return CR_FAILURE;
        }
    } else {
        if PnpGetLocalHandles(&raw mut BindingHandle, &raw mut StringTable) == 0 {
            return CR_FAILURE;
        }
    }

    lpDevInst = pSetupStringTableStringFromId(StringTable, dnDevInst as DWORD);
    if lpDevInst.is_null() {
        return CR_INVALID_DEVNODE;
    }

    // RpcTryExcept
    ret = rpc_try(|| PNP_QueryArbitratorFreeData(
        BindingHandle as handle_t,
        pData as *mut BYTE,
        DataLen as DWORD,
        lpDevInst,
        ResourceID,
        ulFlags as DWORD,
    )).unwrap_or_else(|code| RpcStatusToCmStatus(code as _) as _) as CONFIGRET;
    // RpcExcept(EXCEPTION_EXECUTE_HANDLER)
    // RpcEndExcept;

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Query_Arbitrator_Free_Size(
    pulSize: PULONG,
    dnDevInst: DEVINST,
    ResourceID: RESOURCEID,
    ulFlags: ULONG,
) -> CONFIGRET {
    return CM_Query_Arbitrator_Free_Size_Ex(pulSize, dnDevInst, ResourceID, ulFlags, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn CM_Query_Arbitrator_Free_Size_Ex(
    mut pulSize: PULONG,
    mut dnDevInst: DEVINST,
    mut ResourceID: RESOURCEID,
    mut ulFlags: ULONG,
    mut hMachine: HMACHINE,
) -> CONFIGRET {
    let mut BindingHandle: RPC_BINDING_HANDLE = core::ptr::null_mut();
    let mut StringTable: HSTRING_TABLE = core::ptr::null_mut();
    let mut lpDevInst: LPWSTR = core::ptr::null_mut::<WCHAR>();
    let mut ret: CONFIGRET = 0;

    if pulSize.is_null() {
        return CR_INVALID_POINTER as CONFIGRET;
    }

    if dnDevInst == 0 as DEVINST {
        return CR_INVALID_DEVINST as CONFIGRET;
    }

    if ulFlags & !(CM_QUERY_ARBITRATOR_BITS as ULONG) != 0 {
        return CR_INVALID_FLAG as CONFIGRET;
    }

    if !hMachine.is_null() {
        BindingHandle = (*(hMachine as PMACHINE_INFO)).BindingHandle;
        if BindingHandle.is_null() {
            return CR_FAILURE as CONFIGRET;
        }

        StringTable = (*(hMachine as PMACHINE_INFO)).StringTable;
        if StringTable == core::ptr::null_mut() {
            return CR_FAILURE as CONFIGRET;
        }
    } else {
        if PnpGetLocalHandles(
            core::ptr::addr_of_mut!(BindingHandle),
            core::ptr::addr_of_mut!(StringTable),
        ) == 0
        {
            return CR_FAILURE as CONFIGRET;
        }
    }

    lpDevInst = pSetupStringTableStringFromId(StringTable, dnDevInst as DWORD);
    if lpDevInst.is_null() {
        return CR_INVALID_DEVNODE as CONFIGRET;
    }

    // RpcTryExcept
    {
        ret = rpc_try(|| PNP_QueryArbitratorFreeSize(
            BindingHandle as handle_t,
            pulSize as *mut DWORD,
            lpDevInst,
            ResourceID,
            ulFlags as DWORD,
        )).unwrap_or_else(|code| RpcStatusToCmStatus(code as _) as _) as CONFIGRET;
    }
    // RpcExcept(EXCEPTION_EXECUTE_HANDLER)
    // RpcEndExcept;

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Query_Remove_SubTree(
    dnAncestor: DEVINST,
    ulFlags: ULONG,
) -> CONFIGRET {
    // TRACE("CM_Query_Remove_SubTree(%lx %lx)\n", dnAncestor, ulFlags);
    CR_CALL_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn CM_Query_Remove_SubTree_Ex(
    dnAncestor: DEVINST,
    ulFlags: ULONG,
    hMachine: HMACHINE,
) -> CONFIGRET {
    // TRACE("CM_Query_Remove_SubTree_Ex(%lx %lx %p)\n", dnAncestor, ulFlags, hMachine);
    CR_CALL_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn CM_Query_Resource_Conflict_List(
    pclConflictList: PCONFLICT_LIST,
    dnDevInst: DEVINST,
    ResourceID: RESOURCEID,
    ResourceData: PCVOID,
    ResourceLen: ULONG,
    ulFlags: ULONG,
    hMachine: HMACHINE,
) -> CONFIGRET {
    let mut BindingHandle: RPC_BINDING_HANDLE = core::ptr::null_mut();
    let mut StringTable: HSTRING_TABLE = core::ptr::null_mut();
    let mut pConflictBuffer: PPNP_CONFLICT_LIST = core::ptr::null_mut();
    let mut pConflictData: PCONFLICT_DATA = core::ptr::null_mut();
    let mut ulBufferLength: ULONG = 0;
    let mut lpDevInst: LPWSTR = core::ptr::null_mut();
    let mut ret: CONFIGRET = 0;

    if dnDevInst == 0 {
        return CR_INVALID_DEVNODE as CONFIGRET;
    }

    if ulFlags & !(CM_RESDES_WIDTH_BITS as ULONG) != 0 {
        return CR_INVALID_FLAG as CONFIGRET;
    }

    if pclConflictList.is_null()
        || ResourceData.is_null()
        || ResourceLen == 0 as ULONG
    {
        return CR_INVALID_POINTER as CONFIGRET;
    }

    if ResourceID == 0 as RESOURCEID {
        return CR_INVALID_RESOURCEID as CONFIGRET;
    }

    *pclConflictList = 0 as CONFLICT_LIST;

    if !hMachine.is_null() {
        BindingHandle = (*(hMachine as PMACHINE_INFO)).BindingHandle;
        if BindingHandle.is_null() {
            return CR_FAILURE as CONFIGRET;
        }

        StringTable = (*(hMachine as PMACHINE_INFO)).StringTable;
        if StringTable == 0 as HSTRING_TABLE {
            return CR_FAILURE as CONFIGRET;
        }
    } else {
        if PnpGetLocalHandles(&raw mut BindingHandle, &raw mut StringTable) == 0 {
            return CR_FAILURE as CONFIGRET;
        }
    }

    lpDevInst = pSetupStringTableStringFromId(StringTable, dnDevInst as DWORD);
    if lpDevInst.is_null() {
        return CR_INVALID_DEVNODE as CONFIGRET;
    }

    'done: {
        pConflictData = MyMalloc(core::mem::size_of::<CONFLICT_DATA>() as DWORD) as PCONFLICT_DATA;
        if pConflictData.is_null() {
            ret = CR_OUT_OF_MEMORY as CONFIGRET;
            break 'done;
        }

        ulBufferLength = ((::core::mem::size_of::<PNP_CONFLICT_LIST>() as usize)
            + (::core::mem::size_of::<PNP_CONFLICT_STRINGS>() as usize)
            + (::core::mem::size_of::<wchar_t>() as usize) * (200 as usize))
            as ULONG;
        pConflictBuffer = MyMalloc(ulBufferLength as DWORD) as PPNP_CONFLICT_LIST;
        if pConflictBuffer.is_null() {
            ret = CR_OUT_OF_MEMORY as CONFIGRET;
            break 'done;
        }

        // RpcTryExcept
        {
            ret = rpc_try(|| PNP_QueryResConfList(
                BindingHandle,
                lpDevInst,
                ResourceID,
                ResourceData as PBYTE,
                ResourceLen,
                pConflictBuffer as PBYTE,
                ulBufferLength,
                ulFlags,
            )).unwrap_or_else(|code| RpcStatusToCmStatus(code as _) as _) as CONFIGRET;
        }
        // RpcExcept(EXCEPTION_EXECUTE_HANDLER)
        // RpcEndExcept;

        if ret != CR_SUCCESS as CONFIGRET {
            break 'done;
        }

        (*pConflictData).ulMagic = CONFLICT_MAGIC as ULONG;
        (*pConflictData).pConflictList = pConflictBuffer;

        *pclConflictList = pConflictData as CONFLICT_LIST;
    }

    // done:
    if ret != CR_SUCCESS as CONFIGRET {
        if !pConflictBuffer.is_null() {
            MyFree(pConflictBuffer as PVOID);
        }

        if !pConflictData.is_null() {
            MyFree(pConflictData as PVOID);
        }
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Reenumerate_DevNode(
    dnDevInst: DEVINST,
    ulFlags: ULONG,
) -> CONFIGRET {
    return CM_Reenumerate_DevNode_Ex(dnDevInst, ulFlags, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn CM_Reenumerate_DevNode_Ex(
    dnDevInst: DEVINST,
    ulFlags: ULONG,
    hMachine: HMACHINE,
) -> CONFIGRET {
    let mut BindingHandle: RPC_BINDING_HANDLE = core::ptr::null_mut();
    let mut StringTable: HSTRING_TABLE = core::ptr::null_mut();
    let mut lpDevInst: LPWSTR = core::ptr::null_mut::<WCHAR>();
    let mut ret: CONFIGRET = 0;

    if dnDevInst == 0 {
        return CR_INVALID_DEVNODE;
    }

    if ulFlags & !(CM_REENUMERATE_BITS) != 0 {
        return CR_INVALID_FLAG;
    }

    if !hMachine.is_null() {
        BindingHandle = (*(hMachine as PMACHINE_INFO)).BindingHandle;
        if BindingHandle.is_null() {
            return CR_FAILURE;
        }

        StringTable = (*(hMachine as PMACHINE_INFO)).StringTable;
        if StringTable == core::ptr::null_mut() {
            return CR_FAILURE;
        }
    } else {
        if PnpGetLocalHandles(
            core::ptr::addr_of_mut!(BindingHandle),
            core::ptr::addr_of_mut!(StringTable),
        ) == 0
        {
            return CR_FAILURE;
        }
    }

    lpDevInst = pSetupStringTableStringFromId(StringTable, dnDevInst as DWORD);
    if lpDevInst.is_null() {
        return CR_INVALID_DEVNODE;
    }

    ret = rpc_try(|| PNP_DeviceInstanceAction(
        BindingHandle as handle_t,
        PNP_DEVINST_REENUMERATE as DWORD,
        ulFlags as DWORD,
        lpDevInst,
        core::ptr::null_mut::<WCHAR>(),
    )).unwrap_or_else(|code| RpcStatusToCmStatus(code as _) as _) as CONFIGRET;


    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Register_Device_Driver(
    dnDevInst: DEVINST,
    ulFlags: ULONG,
) -> CONFIGRET {
    return CM_Register_Device_Driver_Ex(dnDevInst, ulFlags, core::ptr::null_mut());
}
#[no_mangle]
pub unsafe extern "system" fn CM_Register_Device_Driver_Ex(
    mut dnDevInst: DEVINST,
    mut ulFlags: ULONG,
    mut hMachine: HMACHINE,
) -> CONFIGRET {
    let mut BindingHandle: RPC_BINDING_HANDLE = core::ptr::null_mut();
    let mut StringTable: HSTRING_TABLE = core::ptr::null_mut();
    let mut lpDevInst: LPWSTR = core::ptr::null_mut();
    let mut ret: CONFIGRET = 0;

    if dnDevInst == 0 as DEVINST {
        return CR_INVALID_DEVNODE as CONFIGRET;
    }

    if ulFlags & !(CM_REGISTER_DEVICE_DRIVER_BITS as ULONG) != 0 {
        return CR_INVALID_FLAG as CONFIGRET;
    }

    if !hMachine.is_null() {
        BindingHandle = (*(hMachine as PMACHINE_INFO)).BindingHandle;
        if BindingHandle.is_null() {
            return CR_FAILURE as CONFIGRET;
        }

        StringTable = (*(hMachine as PMACHINE_INFO)).StringTable;
        if StringTable == 0 as HSTRING_TABLE {
            return CR_FAILURE as CONFIGRET;
        }
    } else {
        if PnpGetLocalHandles(
            core::ptr::addr_of_mut!(BindingHandle),
            core::ptr::addr_of_mut!(StringTable),
        ) == 0
        {
            return CR_FAILURE as CONFIGRET;
        }
    }

    lpDevInst = pSetupStringTableStringFromId(StringTable, dnDevInst as DWORD);
    if lpDevInst.is_null() {
        return CR_INVALID_DEVNODE as CONFIGRET;
    }

    // RpcTryExcept
    ret = rpc_try(|| PNP_RegisterDriver(BindingHandle as handle_t, lpDevInst, ulFlags as DWORD)).unwrap_or_else(|code| RpcStatusToCmStatus(code as _) as _) as CONFIGRET;
    // RpcExcept(EXCEPTION_EXECUTE_HANDLER)
    // RpcEndExcept;


    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Register_Device_InterfaceA(
    dnDevInst: DEVINST,
    InterfaceClassGuid: LPGUID,
    pszReference: LPCSTR,
    pszDeviceInterface: LPSTR,
    pulLength: PULONG,
    ulFlags: ULONG,
) -> CONFIGRET {
    return CM_Register_Device_Interface_ExA(
        dnDevInst,
        InterfaceClassGuid,
        pszReference,
        pszDeviceInterface,
        pulLength,
        ulFlags,
        core::ptr::null_mut(),
    );
}

#[no_mangle]
pub unsafe extern "system" fn CM_Register_Device_InterfaceW(
    dnDevInst: DEVINST,
    InterfaceClassGuid: LPGUID,
    pszReference: LPCWSTR,
    pszDeviceInterface: LPWSTR,
    pulLength: PULONG,
    ulFlags: ULONG,
) -> CONFIGRET {
    return CM_Register_Device_Interface_ExW(
        dnDevInst,
        InterfaceClassGuid,
        pszReference,
        pszDeviceInterface,
        pulLength,
        ulFlags,
        core::ptr::null_mut(),
    );
}

#[no_mangle]
pub unsafe extern "system" fn CM_Register_Device_Interface_ExA(
    dnDevInst: DEVINST,
    InterfaceClassGuid: LPGUID,
    pszReference: LPCSTR,
    pszDeviceInterface: LPSTR,
    pulLength: PULONG,
    ulFlags: ULONG,
    hMachine: HMACHINE,
) -> CONFIGRET {
    let mut pszReferenceW: LPWSTR = core::ptr::null_mut();
    let mut pszDeviceInterfaceW: LPWSTR = core::ptr::null_mut();
    let mut ulLength: ULONG = 0;
    let mut ret: CONFIGRET = 0;

    TRACE!(
        "CM_Register_Device_Interface_ExA(%lx %s %s %p %p %lx %p)\n",
        dnDevInst,
        debugstr_guid(InterfaceClassGuid as *const _GUID),
        debugstr_a(pszReference as *const ::core::ffi::c_char),
        pszDeviceInterface,
        pulLength,
        ulFlags,
        hMachine
    );

    if pulLength.is_null() || pszDeviceInterface.is_null() {
        return CR_INVALID_POINTER;
    }

    if !pszReference.is_null() {
        if pSetupCaptureAndConvertAnsiArg(pszReference as PCSTR, core::ptr::addr_of_mut!(pszReferenceW)) != 0 {
            return CR_INVALID_DATA;
        }
    }

    ulLength = *pulLength;

    pszDeviceInterfaceW = HeapAlloc(
        GetProcessHeap(),
        0,
        (ulLength as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
    ) as LPWSTR;

    'done: {
        if pszDeviceInterfaceW.is_null() {
            ret = CR_OUT_OF_MEMORY;
            break 'done;
        }

        ret = CM_Register_Device_Interface_ExW(
            dnDevInst,
            InterfaceClassGuid,
            pszReferenceW as LPCWSTR,
            pszDeviceInterfaceW,
            core::ptr::addr_of_mut!(ulLength),
            ulFlags,
            hMachine,
        );
        if ret == CR_SUCCESS {
            if WideCharToMultiByte(
                CP_ACP,
                0,
                pszDeviceInterfaceW as LPCWSTR,
                ulLength as ::core::ffi::c_int,
                pszDeviceInterface,
                *pulLength as ::core::ffi::c_int,
                core::ptr::null::<CHAR>(),
                core::ptr::null_mut::<BOOL>(),
            ) == 0
            {
                ret = CR_FAILURE;
            }
        }

        *pulLength = ulLength;
    }

    if !pszDeviceInterfaceW.is_null() {
        HeapFree(
            GetProcessHeap(),
            0,
            pszDeviceInterfaceW as PVOID,
        );
    }

    if !pszReferenceW.is_null() {
        MyFree(pszReferenceW as PVOID);
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Register_Device_Interface_ExW(
    mut dnDevInst: DEVINST,
    mut InterfaceClassGuid: LPGUID,
    mut pszReference: LPCWSTR,
    mut pszDeviceInterface: LPWSTR,
    mut pulLength: PULONG,
    mut ulFlags: ULONG,
    mut hMachine: HMACHINE,
) -> CONFIGRET {
    let mut BindingHandle: RPC_BINDING_HANDLE = core::ptr::null_mut();
    let mut StringTable: HSTRING_TABLE = core::ptr::null_mut();
    let mut lpDevInst: LPWSTR = core::ptr::null_mut::<WCHAR>();
    let mut ulTransferLength: ULONG = 0;
    let mut ret: CONFIGRET = 0;

    if dnDevInst == 0 {
        return CR_INVALID_DEVNODE;
    }
    if InterfaceClassGuid.is_null() || pszDeviceInterface.is_null() || pulLength.is_null() {
        return CR_INVALID_POINTER;
    }
    if ulFlags != 0 {
        return CR_INVALID_FLAG;
    }
    if !hMachine.is_null() {
        BindingHandle = (*(hMachine as PMACHINE_INFO)).BindingHandle;
        if BindingHandle.is_null() {
            return CR_FAILURE;
        }
        StringTable = (*(hMachine as PMACHINE_INFO)).StringTable;
        if StringTable == core::ptr::null_mut() {
            return CR_FAILURE;
        }
    } else {
        if PnpGetLocalHandles(
            core::ptr::addr_of_mut!(BindingHandle),
            core::ptr::addr_of_mut!(StringTable),
        ) == 0
        {
            return CR_FAILURE;
        }
    }

    lpDevInst = pSetupStringTableStringFromId(StringTable, dnDevInst as DWORD);
    if lpDevInst.is_null() {
        return CR_INVALID_DEVNODE;
    }

    ulTransferLength = *pulLength;

    // RpcTryExcept body
    ret = rpc_try(|| PNP_RegisterDeviceClassAssociation(
        BindingHandle as handle_t,
        lpDevInst,
        InterfaceClassGuid as *mut GUID,
        pszReference as LPWSTR,
        pszDeviceInterface,
        pulLength as *mut PNP_RPC_STRING_LEN,
        core::ptr::addr_of_mut!(ulTransferLength),
        0,
    )).unwrap_or_else(|code| RpcStatusToCmStatus(code as _) as _) as CONFIGRET;

    // RpcExcept(EXCEPTION_EXECUTE_HANDLER) handler
    // RpcEndExcept;

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Remove_SubTree(
    dnAncestor: DEVINST,
    ulFlags: ULONG,
) -> CONFIGRET {
    // TRACE("CM_Remove_SubTree(%lx %lx)\n", dnAncestor, ulFlags);
    CR_CALL_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn CM_Remove_SubTree_Ex(
    dnAncestor: DEVINST,
    ulFlags: ULONG,
    hMachine: HMACHINE,
) -> CONFIGRET {
    // TRACE("CM_Remove_SubTree_Ex(%lx %lx %p)\n", dnAncestor, ulFlags, hMachine);
    CR_CALL_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn CM_Request_Device_EjectA(
    dnDevInst: DEVINST,
    pVetoType: PPNP_VETO_TYPE,
    pszVetoName: LPSTR,
    ulNameLength: ULONG,
    ulFlags: ULONG,
) -> CONFIGRET {
    return CM_Request_Device_Eject_ExA(dnDevInst, pVetoType, pszVetoName, ulNameLength, ulFlags, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn CM_Request_Device_EjectW(
    dnDevInst: DEVINST,
    pVetoType: PPNP_VETO_TYPE,
    pszVetoName: LPWSTR,
    ulNameLength: ULONG,
    ulFlags: ULONG,
) -> CONFIGRET {
    return CM_Request_Device_Eject_ExW(dnDevInst, pVetoType, pszVetoName,
        ulNameLength, ulFlags, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn CM_Request_Device_Eject_ExA(
    dnDevInst: DEVINST,
    pVetoType: PPNP_VETO_TYPE,
    pszVetoName: LPSTR,
    ulNameLength: ULONG,
    ulFlags: ULONG,
    hMachine: HMACHINE,
) -> CONFIGRET {
    let mut lpLocalVetoName: LPWSTR = core::ptr::null_mut();
    let mut ret: CONFIGRET = 0;

    if ulNameLength != 0 {
        if pszVetoName.is_null() {
            return CR_INVALID_POINTER;
        }

        lpLocalVetoName = HeapAlloc(
            GetProcessHeap(),
            0,
            (ulNameLength.wrapping_mul(core::mem::size_of::<WCHAR>() as ULONG)) as SIZE_T,
        ) as LPWSTR;
        if lpLocalVetoName.is_null() {
            return CR_OUT_OF_MEMORY;
        }
    }

    ret = CM_Request_Device_Eject_ExW(
        dnDevInst,
        pVetoType,
        lpLocalVetoName,
        ulNameLength,
        ulFlags,
        hMachine,
    );
    if ret == CR_REMOVE_VETOED && ulNameLength != 0 {
        if WideCharToMultiByte(
            CP_ACP,
            0,
            lpLocalVetoName as LPCWSTR,
            ulNameLength as core::ffi::c_int,
            pszVetoName,
            ulNameLength as core::ffi::c_int,
            core::ptr::null::<CHAR>(),
            core::ptr::null_mut::<BOOL>(),
        ) == 0
        {
            ret = CR_FAILURE;
        }
    }

    HeapFree(
        GetProcessHeap(),
        0,
        lpLocalVetoName as PVOID,
    );

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Request_Device_Eject_ExW(
    mut dnDevInst: DEVINST,
    mut pVetoType: PPNP_VETO_TYPE,
    mut pszVetoName: LPWSTR,
    mut ulNameLength: ULONG,
    mut ulFlags: ULONG,
    mut hMachine: HMACHINE,
) -> CONFIGRET {
    let mut BindingHandle: RPC_BINDING_HANDLE = core::ptr::null_mut();
    let mut StringTable: HSTRING_TABLE = core::ptr::null_mut();
    let mut lpDevInst: LPWSTR = core::ptr::null_mut();
    let mut ret: CONFIGRET;

    if dnDevInst == 0 {
        return CR_INVALID_DEVNODE;
    }

    if ulFlags != 0 {
        return CR_INVALID_FLAG;
    }

    if pszVetoName.is_null() && ulNameLength != 0 {
        return CR_INVALID_POINTER;
    }

    /* Windows 2003 SP2 ignores pszVetoName when ulNameLength is zero
     * and behaves like when pszVetoName is NULL */
    if ulNameLength == 0 {
        pszVetoName = core::ptr::null_mut();
    }

    if !hMachine.is_null() {
        BindingHandle = (*(hMachine as PMACHINE_INFO)).BindingHandle;
        if BindingHandle.is_null() {
            return CR_FAILURE;
        }

        StringTable = (*(hMachine as PMACHINE_INFO)).StringTable;
        if StringTable == core::ptr::null_mut() {
            return CR_FAILURE;
        }
    } else {
        if PnpGetLocalHandles(&raw mut BindingHandle, &raw mut StringTable) == 0 {
            return CR_FAILURE;
        }
    }

    lpDevInst = pSetupStringTableStringFromId(StringTable, dnDevInst as DWORD);
    if lpDevInst.is_null() {
        return CR_INVALID_DEVNODE;
    }

    // RpcTryExcept body
    ret = rpc_try(|| PNP_RequestDeviceEject(
        BindingHandle as handle_t,
        lpDevInst,
        pVetoType,
        pszVetoName,
        ulNameLength as DWORD,
        ulFlags as DWORD,
    )).unwrap_or_else(|code| RpcStatusToCmStatus(code as _) as _) as CONFIGRET;

    // RpcExcept(EXCEPTION_EXECUTE_HANDLER) handler body
    // RpcEndExcept;

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Request_Eject_PC() -> CONFIGRET {
    TRACE!("CM_Request_Eject_PC()\n");
    return CM_Request_Eject_PC_Ex(core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn CM_Request_Eject_PC_Ex(hMachine: HMACHINE) -> CONFIGRET {
    let mut BindingHandle: RPC_BINDING_HANDLE = core::ptr::null_mut();
    let mut ret: CONFIGRET = 0;

    if !hMachine.is_null() {
        BindingHandle = (*(hMachine as PMACHINE_INFO)).BindingHandle;
        if BindingHandle.is_null() {
            return CR_FAILURE;
        }
    } else {
        if PnpGetLocalHandles(
            core::ptr::addr_of_mut!(BindingHandle),
            core::ptr::null_mut::<HSTRING_TABLE>(),
        ) == 0
        {
            return CR_FAILURE;
        }
    }

    // RpcTryExcept
    ret = rpc_try(|| PNP_RequestEjectPC(BindingHandle as handle_t)).unwrap_or_else(|code| RpcStatusToCmStatus(code as _) as _) as CONFIGRET;
    // RpcExcept(EXCEPTION_EXECUTE_HANDLER)
    // RpcEndExcept;

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Run_Detection(ulFlags: ULONG) -> CONFIGRET {
    return CM_Run_Detection_Ex(ulFlags, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn CM_Run_Detection_Ex(
    ulFlags: ULONG,
    hMachine: HMACHINE,
) -> CONFIGRET {
    let mut BindingHandle: RPC_BINDING_HANDLE = core::ptr::null_mut();
    let mut ret: CONFIGRET = 0;

    if pSetupIsUserAdmin() == 0 {
        return CR_ACCESS_DENIED;
    }

    if ulFlags & !CM_DETECT_BITS != 0 {
        return CR_INVALID_FLAG;
    }

    if !hMachine.is_null() {
        BindingHandle = (*(hMachine as PMACHINE_INFO)).BindingHandle;
        if BindingHandle.is_null() {
            return CR_FAILURE;
        }
    } else if PnpGetLocalHandles(&raw mut BindingHandle, core::ptr::null_mut()) == 0 {
        return CR_FAILURE;
    }

    ret = rpc_try(|| PNP_RunDetection(BindingHandle as handle_t, ulFlags as DWORD)).unwrap_or_else(|code| RpcStatusToCmStatus(code as _) as _) as CONFIGRET;

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Set_Class_Registry_PropertyA(
    ClassGuid: LPGUID,
    ulProperty: ULONG,
    Buffer: PCVOID,
    ulLength: ULONG,
    ulFlags: ULONG,
    hMachine: HMACHINE,
) -> CONFIGRET {
    let mut lpBuffer: LPWSTR = core::ptr::null_mut();
    let mut ulType: ULONG = 0;
    let mut ret: CONFIGRET = 0;

    if ClassGuid.is_null() {
        return CR_INVALID_POINTER as CONFIGRET;
    }

    if Buffer.is_null() && ulLength != 0 as ULONG {
        return CR_INVALID_POINTER as CONFIGRET;
    }

    if ulFlags != 0 as ULONG {
        return CR_INVALID_FLAG as CONFIGRET;
    }

    if Buffer.is_null() {
        ret = CM_Set_Class_Registry_PropertyW(
            ClassGuid,
            ulProperty,
            Buffer,
            ulLength,
            ulFlags,
            hMachine,
        );
    } else {
        /* Get property type */
        ulType = GetRegistryPropertyType(ulProperty);

        /* Allocate buffer if needed */
        if ulType == REG_SZ as ULONG || ulType == REG_MULTI_SZ as ULONG {
            lpBuffer = MyMalloc(
                (ulLength as usize).wrapping_mul(core::mem::size_of::<WCHAR>()) as DWORD,
            ) as LPWSTR;
            if lpBuffer.is_null() {
                ret = CR_OUT_OF_MEMORY as CONFIGRET;
            } else {
                if MultiByteToWideChar(
                    CP_ACP as UINT,
                    0 as DWORD,
                    Buffer as LPCSTR,
                    ulLength as core::ffi::c_int,
                    lpBuffer,
                    ulLength as core::ffi::c_int,
                ) == 0
                {
                    MyFree(lpBuffer as PVOID);
                    ret = CR_FAILURE as CONFIGRET;
                } else {
                    ret = CM_Set_Class_Registry_PropertyW(
                        ClassGuid,
                        ulProperty,
                        lpBuffer as PCVOID,
                        (ulLength as usize).wrapping_mul(core::mem::size_of::<WCHAR>()) as ULONG,
                        ulFlags,
                        hMachine,
                    );
                    MyFree(lpBuffer as PVOID);
                }
            }
        } else {
            ret = CM_Set_Class_Registry_PropertyW(
                ClassGuid,
                ulProperty,
                Buffer,
                ulLength,
                ulFlags,
                hMachine,
            );
        }
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Set_Class_Registry_PropertyW(
    mut ClassGuid: LPGUID,
    mut ulProperty: ULONG,
    mut Buffer: PCVOID,
    mut ulLength: ULONG,
    mut ulFlags: ULONG,
    mut hMachine: HMACHINE,
) -> CONFIGRET {
    let mut BindingHandle: RPC_BINDING_HANDLE = core::ptr::null_mut();
    let mut szGuidString: [WCHAR; (PNP_MAX_GUID_STRING_LEN + 1) as _] =
        [0; (PNP_MAX_GUID_STRING_LEN + 1) as _];
    let mut ulType: ULONG = 0 as ULONG;
    let mut pSecurityDescriptor: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
    let mut SecurityDescriptorSize: ULONG = 0 as ULONG;
    let mut ret: CONFIGRET;

    if ClassGuid.is_null() {
        return CR_INVALID_POINTER as CONFIGRET;
    }

    if Buffer.is_null() && ulLength != 0 as ULONG {
        return CR_INVALID_POINTER as CONFIGRET;
    }

    if ulFlags != 0 as ULONG {
        return CR_INVALID_FLAG as CONFIGRET;
    }

    if pSetupStringFromGuid(
        ClassGuid,
        &raw mut szGuidString as PWSTR,
        PNP_MAX_GUID_STRING_LEN as DWORD,
    ) != 0 as DWORD
    {
        return CR_INVALID_DATA as CONFIGRET;
    }

    if ulProperty < CM_CRP_MIN as ULONG || ulProperty > CM_CRP_MAX as ULONG {
        return CR_INVALID_PROPERTY as CONFIGRET;
    }

    if !hMachine.is_null() {
        BindingHandle = (*(hMachine as PMACHINE_INFO)).BindingHandle;
        if BindingHandle.is_null() {
            return CR_FAILURE as CONFIGRET;
        }
    } else {
        if PnpGetLocalHandles(
            &raw mut BindingHandle,
            core::ptr::null_mut(),
        ) == 0
        {
            return CR_FAILURE as CONFIGRET;
        }
    }

    ulType = GetRegistryPropertyType(ulProperty);
    if ulType == REG_DWORD as ULONG
        && ulLength as usize != core::mem::size_of::<DWORD>() as usize
    {
        return CR_INVALID_DATA as CONFIGRET;
    }

    if ulProperty == CM_CRP_SECURITY_SDS as ULONG {
        if ulLength != 0 as ULONG {
            if ConvertStringSecurityDescriptorToSecurityDescriptorW(
                Buffer as LPCWSTR,
                SDDL_REVISION_1 as DWORD,
                &raw mut pSecurityDescriptor,
                &raw mut SecurityDescriptorSize,
            ) == 0
            {
                ERR!(
                    "ConvertStringSecurityDescriptorToSecurityDescriptorW() failed (Error %lu)\n",
                    GetLastError()
                );
                return CR_INVALID_DATA as CONFIGRET;
            }

            Buffer = pSecurityDescriptor as PCVOID;
            ulLength = SecurityDescriptorSize;
        }

        ulProperty = CM_CRP_SECURITY as ULONG;
        ulType = REG_BINARY as ULONG;
    }

    // RpcTryExcept
    ret = rpc_try(|| PNP_SetClassRegProp(
        BindingHandle as handle_t,
        &raw mut szGuidString as LPWSTR,
        ulProperty as DWORD,
        ulType as DWORD,
        Buffer as LPBYTE,
        ulLength as DWORD,
        ulFlags as DWORD,
    )).unwrap_or_else(|code| RpcStatusToCmStatus(code as _) as _) as CONFIGRET;
    // RpcExcept(EXCEPTION_EXECUTE_HANDLER)
    // RpcEndExcept;

    if !pSecurityDescriptor.is_null() {
        LocalFree(pSecurityDescriptor as HLOCAL);
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Set_DevNode_Problem(
    dnDevInst: DEVINST,
    ulProblem: ULONG,
    ulFlags: ULONG,
) -> CONFIGRET {
    return CM_Set_DevNode_Problem_Ex(dnDevInst, ulProblem, ulFlags, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn CM_Set_DevNode_Problem_Ex(
    mut dnDevInst: DEVINST,
    mut ulProblem: ULONG,
    mut ulFlags: ULONG,
    mut hMachine: HMACHINE,
) -> CONFIGRET {
    let mut BindingHandle: RPC_BINDING_HANDLE = core::ptr::null_mut();
    let mut StringTable: HSTRING_TABLE = core::ptr::null_mut();
    let mut lpDevInst: LPWSTR = core::ptr::null_mut::<WCHAR>();
    let mut ret: CONFIGRET = 0;

    if dnDevInst == 0 {
        return CR_INVALID_DEVNODE;
    }

    if ulFlags & !CM_SET_DEVNODE_PROBLEM_BITS != 0 {
        return CR_INVALID_FLAG;
    }

    if !hMachine.is_null() {
        BindingHandle = (*(hMachine as PMACHINE_INFO)).BindingHandle;
        if BindingHandle.is_null() {
            return CR_FAILURE;
        }

        StringTable = (*(hMachine as PMACHINE_INFO)).StringTable;
        if StringTable == core::ptr::null_mut() {
            return CR_FAILURE;
        }
    } else {
        if PnpGetLocalHandles(&raw mut BindingHandle, &raw mut StringTable) == 0 {
            return CR_FAILURE;
        }
    }

    lpDevInst = pSetupStringTableStringFromId(StringTable, dnDevInst as DWORD);
    if lpDevInst.is_null() {
        return CR_INVALID_DEVNODE;
    }

    // RpcTryExcept
    ret = rpc_try(|| PNP_SetDeviceProblem(
        BindingHandle as handle_t,
        lpDevInst,
        ulProblem as DWORD,
        ulFlags as DWORD,
    )).unwrap_or_else(|code| RpcStatusToCmStatus(code as _) as _) as CONFIGRET;
    // RpcExcept(EXCEPTION_EXECUTE_HANDLER)
    // RpcEndExcept;

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Set_DevNode_Registry_PropertyA(
    dnDevInst: DEVINST,
    ulProperty: ULONG,
    Buffer: PCVOID,
    ulLength: ULONG,
    ulFlags: ULONG,
) -> CONFIGRET {
    return CM_Set_DevNode_Registry_Property_ExA(dnDevInst, ulProperty, Buffer, ulLength, ulFlags, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn CM_Set_DevNode_Registry_PropertyW(
    dnDevInst: DEVINST,
    ulProperty: ULONG,
    Buffer: PCVOID,
    ulLength: ULONG,
    ulFlags: ULONG,
) -> CONFIGRET {
    return CM_Set_DevNode_Registry_Property_ExW(dnDevInst, ulProperty, Buffer, ulLength, ulFlags, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn CM_Set_DevNode_Registry_Property_ExA(
    dnDevInst: DEVINST,
    ulProperty: ULONG,
    Buffer: PCVOID,
    ulLength: ULONG,
    ulFlags: ULONG,
    hMachine: HMACHINE,
) -> CONFIGRET {
    let mut ret: CONFIGRET = CR_SUCCESS as CONFIGRET;
    let mut lpBuffer: LPWSTR = core::ptr::null_mut();
    let mut ulType: ULONG = 0;

    if Buffer.is_null() && ulLength != 0 as ULONG {
        return CR_INVALID_POINTER as CONFIGRET;
    }

    if ulProperty < CM_DRP_MIN as ULONG || ulProperty > CM_DRP_MAX as ULONG {
        return CR_INVALID_PROPERTY as CONFIGRET;
    }

    if Buffer.is_null() {
        ret = CM_Set_DevNode_Registry_Property_ExW(
            dnDevInst,
            ulProperty,
            core::ptr::null(),
            0 as ULONG,
            ulFlags,
            hMachine,
        );
    } else {
        /* Get property type */
        ulType = GetRegistryPropertyType(ulProperty);

        /* Allocate buffer if needed */
        if ulType == REG_SZ as ULONG || ulType == REG_MULTI_SZ as ULONG {
            lpBuffer = MyMalloc(
                (ulLength as usize).wrapping_mul(core::mem::size_of::<WCHAR>()) as DWORD,
            ) as LPWSTR;
            if lpBuffer.is_null() {
                ret = CR_OUT_OF_MEMORY as CONFIGRET;
            } else {
                if MultiByteToWideChar(
                    CP_ACP as UINT,
                    0 as DWORD,
                    Buffer as LPCSTR,
                    ulLength as core::ffi::c_int,
                    lpBuffer,
                    ulLength as core::ffi::c_int,
                ) == 0
                {
                    MyFree(lpBuffer as PVOID);
                    ret = CR_FAILURE as CONFIGRET;
                } else {
                    ret = CM_Set_DevNode_Registry_Property_ExW(
                        dnDevInst,
                        ulProperty,
                        lpBuffer as PCVOID,
                        (ulLength as usize).wrapping_mul(core::mem::size_of::<WCHAR>()) as ULONG,
                        ulFlags,
                        hMachine,
                    );
                    MyFree(lpBuffer as PVOID);
                }
            }
        } else {
            ret = CM_Set_DevNode_Registry_Property_ExW(
                dnDevInst,
                ulProperty,
                Buffer,
                ulLength,
                ulFlags,
                hMachine,
            );
        }

        ret = CR_CALL_NOT_IMPLEMENTED as CONFIGRET;
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Set_DevNode_Registry_Property_ExW(
    mut dnDevInst: DEVINST,
    mut ulProperty: ULONG,
    mut Buffer: PCVOID,
    mut ulLength: ULONG,
    mut ulFlags: ULONG,
    mut hMachine: HMACHINE,
) -> CONFIGRET {
    let mut BindingHandle: RPC_BINDING_HANDLE = core::ptr::null_mut();
    let mut StringTable: HSTRING_TABLE = core::ptr::null_mut();
    let mut lpDevInst: LPWSTR = core::ptr::null_mut();
    let mut ulType: ULONG = 0;
    let mut ret: CONFIGRET = 0;

    if dnDevInst == 0 {
        return CR_INVALID_DEVNODE;
    }

    if ulProperty < CM_DRP_MIN || ulProperty > CM_DRP_MAX {
        return CR_INVALID_PROPERTY;
    }

    if !Buffer.is_null() && ulLength == 0 {
        return CR_INVALID_POINTER;
    }

    if ulFlags != 0 {
        return CR_INVALID_FLAG;
    }

    if !hMachine.is_null() {
        BindingHandle = (*(hMachine as PMACHINE_INFO)).BindingHandle;
        if BindingHandle.is_null() {
            return CR_FAILURE;
        }

        StringTable = (*(hMachine as PMACHINE_INFO)).StringTable;
        if StringTable == core::ptr::null_mut() {
            return CR_FAILURE;
        }
    } else {
        if PnpGetLocalHandles(
            core::ptr::addr_of_mut!(BindingHandle),
            core::ptr::addr_of_mut!(StringTable),
        ) == 0
        {
            return CR_FAILURE;
        }
    }

    lpDevInst = pSetupStringTableStringFromId(StringTable, dnDevInst);
    if lpDevInst.is_null() {
        return CR_INVALID_DEVNODE;
    }

    /* Get property type */
    ulType = GetRegistryPropertyType(ulProperty);

    // RpcTryExcept
    ret = rpc_try(|| PNP_SetDeviceRegProp(
        BindingHandle,
        lpDevInst,
        ulProperty,
        ulType,
        Buffer as *mut BYTE,
        ulLength,
        ulFlags,
    )).unwrap_or_else(|code| RpcStatusToCmStatus(code as _) as _) as CONFIGRET;
    // RpcExcept(EXCEPTION_EXECUTE_HANDLER)
    // RpcEndExcept;

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Set_HW_Prof(ulHardwareProfile: ULONG, ulFlags: ULONG) -> CONFIGRET {
    return CM_Set_HW_Prof_Ex(ulHardwareProfile, ulFlags, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn CM_Set_HW_Prof_Ex(
    mut ulHardwareProfile: ULONG,
    mut ulFlags: ULONG,
    mut hMachine: HMACHINE,
) -> CONFIGRET {
    let mut BindingHandle: RPC_BINDING_HANDLE = core::ptr::null_mut();
    let mut ret: CONFIGRET = 0;

    if pSetupIsUserAdmin() == 0 {
        return CR_ACCESS_DENIED;
    }

    if ulFlags != 0 {
        return CR_INVALID_FLAG;
    }

    if !hMachine.is_null() {
        BindingHandle = (*(hMachine as PMACHINE_INFO)).BindingHandle;
        if BindingHandle.is_null() {
            return CR_FAILURE;
        }
    } else {
        if PnpGetLocalHandles(
            &raw mut BindingHandle,
            core::ptr::null_mut(),
        ) == 0
        {
            return CR_FAILURE;
        }
    }

    // RpcTryExcept
    ret = rpc_try(|| PNP_SetHwProf(
        BindingHandle as handle_t,
        ulHardwareProfile as DWORD,
        ulFlags as DWORD,
    )).unwrap_or_else(|code| RpcStatusToCmStatus(code as _) as _) as CONFIGRET;

    // RpcExcept(EXCEPTION_EXECUTE_HANDLER)
    // RpcEndExcept;

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Set_HW_Prof_FlagsA(
    szDevInstName: DEVINSTID_A,
    ulConfig: ULONG,
    ulValue: ULONG,
    ulFlags: ULONG,
) -> CONFIGRET {
    return CM_Set_HW_Prof_Flags_ExA(szDevInstName, ulConfig, ulValue, ulFlags, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn CM_Set_HW_Prof_FlagsW(
    szDevInstName: DEVINSTID_W,
    ulConfig: ULONG,
    ulValue: ULONG,
    ulFlags: ULONG,
) -> CONFIGRET {
    return CM_Set_HW_Prof_Flags_ExW(szDevInstName, ulConfig, ulValue,
                                    ulFlags, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn CM_Set_HW_Prof_Flags_ExA(
    szDevInstName: DEVINSTID_A,
    ulConfig: ULONG,
    ulValue: ULONG,
    ulFlags: ULONG,
    hMachine: HMACHINE,
) -> CONFIGRET {
    let mut pszDevIdW: DEVINSTID_W = core::ptr::null_mut();
    let mut ret: CONFIGRET = CR_SUCCESS;

    if !szDevInstName.is_null() {
        if pSetupCaptureAndConvertAnsiArg(szDevInstName as PCSTR, core::ptr::addr_of_mut!(pszDevIdW)) != 0 {
            return CR_INVALID_DEVICE_ID;
        }
    }

    ret = CM_Set_HW_Prof_Flags_ExW(pszDevIdW, ulConfig, ulValue, ulFlags, hMachine);

    if !pszDevIdW.is_null() {
        MyFree(pszDevIdW as PVOID);
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Set_HW_Prof_Flags_ExW(
    mut szDevInstName: DEVINSTID_W,
    mut ulConfig: ULONG,
    mut ulValue: ULONG,
    mut ulFlags: ULONG,
    mut hMachine: HMACHINE,
) -> CONFIGRET {
    let mut BindingHandle: RPC_BINDING_HANDLE = core::ptr::null_mut();
    let mut ret: CONFIGRET = 0;
    TRACE!(
        "CM_Set_HW_Prof_Flags_ExW(%s %lu %lu %lx %p)\n",
        debugstr_w(szDevInstName as *const WCHAR),
        ulConfig,
        ulValue,
        ulFlags,
        hMachine
    );
    if szDevInstName.is_null() {
        return CR_INVALID_POINTER;
    }
    if ulFlags & !(CM_SET_HW_PROF_FLAGS_BITS as ULONG) != 0 {
        return CR_INVALID_FLAG;
    }
    if !hMachine.is_null() {
        BindingHandle = (*(hMachine as PMACHINE_INFO)).BindingHandle;
        if BindingHandle.is_null() {
            return CR_FAILURE;
        }
    } else if PnpGetLocalHandles(
        core::ptr::addr_of_mut!(BindingHandle),
        core::ptr::null_mut::<HSTRING_TABLE>(),
    ) == 0
    {
        return CR_FAILURE;
    }
    ret = rpc_try(|| PNP_HwProfFlags(
        BindingHandle as handle_t,
        PNP_SET_HWPROFFLAGS as DWORD,
        szDevInstName as LPWSTR,
        ulConfig as DWORD,
        core::ptr::addr_of_mut!(ulValue),
        core::ptr::null_mut::<_PNP_VETO_TYPE>(),
        core::ptr::null_mut::<WCHAR>(),
        0 as DWORD,
        0 as DWORD,
    )).unwrap_or_else(|code| RpcStatusToCmStatus(code as _) as _) as CONFIGRET;
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Setup_DevNode(dnDevInst: DEVINST, ulFlags: ULONG) -> CONFIGRET {
    TRACE!("CM_Setup_DevNode(%lx %lx)\n", dnDevInst, ulFlags);
    return CM_Setup_DevNode_Ex(dnDevInst, ulFlags, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn CM_Setup_DevNode_Ex(
    dnDevInst: DEVINST,
    ulFlags: ULONG,
    hMachine: HMACHINE,
) -> CONFIGRET {
    let mut BindingHandle: RPC_BINDING_HANDLE = core::ptr::null_mut();
    let mut StringTable: HSTRING_TABLE = core::ptr::null_mut();
    let mut lpDevInst: LPWSTR = core::ptr::null_mut();
    let mut ret: CONFIGRET = 0;

    if ((pSetupIsUserAdmin()) == 0) {
        return CR_ACCESS_DENIED;
    }

    if dnDevInst == 0 {
        return CR_INVALID_DEVNODE;
    }

    if ulFlags & !CM_SETUP_BITS != 0 {
        return CR_INVALID_FLAG;
    }

    if !hMachine.is_null() {
        BindingHandle = (*(hMachine as PMACHINE_INFO)).BindingHandle;
        if BindingHandle.is_null() {
            return CR_FAILURE;
        }

        StringTable = (*(hMachine as PMACHINE_INFO)).StringTable;
        if StringTable == core::ptr::null_mut() {
            return CR_FAILURE;
        }
    } else {
        if PnpGetLocalHandles(
            core::ptr::addr_of_mut!(BindingHandle),
            core::ptr::addr_of_mut!(StringTable),
        ) == 0
        {
            return CR_FAILURE;
        }
    }

    lpDevInst = pSetupStringTableStringFromId(StringTable, dnDevInst as DWORD);
    if lpDevInst.is_null() {
        return CR_INVALID_DEVNODE;
    }

    // RpcTryExcept
    ret = rpc_try(|| PNP_DeviceInstanceAction(
        BindingHandle as handle_t,
        PNP_DEVINST_SETUP as DWORD,
        ulFlags as DWORD,
        lpDevInst,
        core::ptr::null_mut(),
    )).unwrap_or_else(|code| RpcStatusToCmStatus(code as _) as _) as CONFIGRET;
    // RpcExcept(EXCEPTION_EXECUTE_HANDLER)
    // RpcEndExcept;

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Test_Range_Available(
    ullStartValue: DWORDLONG,
    ullEndValue: DWORDLONG,
    rlh: RANGE_LIST,
    ulFlags: ULONG,
) -> CONFIGRET {
    let mut pRangeList: PINTERNAL_RANGE_LIST = core::ptr::null_mut();
    let mut pRange: PINTERNAL_RANGE = core::ptr::null_mut();
    let mut ListEntry: PLIST_ENTRY = core::ptr::null_mut();
    let mut ret: CONFIGRET = CR_SUCCESS;

    pRangeList = rlh as PINTERNAL_RANGE_LIST;

    if IsValidRangeList(pRangeList) == 0 {
        return CR_INVALID_RANGE_LIST;
    }

    if ulFlags != 0 {
        return CR_INVALID_FLAG;
    }

    if ullStartValue > ullEndValue {
        return CR_INVALID_RANGE;
    }

    /* Lock the range list */
    WaitForSingleObject((*pRangeList).hMutex, INFINITE);

    /* Check the ranges */
    ListEntry = core::ptr::addr_of_mut!((*pRangeList).ListHead);
    while (*ListEntry).Flink == core::ptr::addr_of_mut!((*pRangeList).ListHead) {
        pRange = CONTAINING_RECORD!(ListEntry, INTERNAL_RANGE, ListEntry);

        /* Check if the start value is within the current range */
        if (ullStartValue >= (*pRange).ullStart) && (ullStartValue <= (*pRange).ullEnd) {
            ret = CR_FAILURE;
            break;
        }

        /* Check if the end value is within the current range */
        if (ullEndValue >= (*pRange).ullStart) && (ullEndValue <= (*pRange).ullEnd) {
            ret = CR_FAILURE;
            break;
        }

        /* Check if the current range lies inside of the start-end interval */
        if (ullStartValue <= (*pRange).ullStart) && (ullEndValue >= (*pRange).ullEnd) {
            ret = CR_FAILURE;
            break;
        }

        ListEntry = (*ListEntry).Flink;
    }

    /* Unlock the range list */
    ReleaseMutex((*pRangeList).hMutex);

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Uninstall_DevNode(
    dnPhantom: DEVINST,
    ulFlags: ULONG,
) -> CONFIGRET {
    TRACE!("CM_Uninstall_DevNode(%lx %lx)\n", dnPhantom, ulFlags);
    return CM_Uninstall_DevNode_Ex(dnPhantom, ulFlags, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn CM_Uninstall_DevNode_Ex(
    dnPhantom: DEVINST,
    ulFlags: ULONG,
    hMachine: HMACHINE,
) -> CONFIGRET {
    let mut BindingHandle: RPC_BINDING_HANDLE = core::ptr::null_mut();
    let mut StringTable: HSTRING_TABLE = core::ptr::null_mut();
    let mut lpDevInst: LPWSTR = core::ptr::null_mut::<WCHAR>();
    let mut ret: CONFIGRET;

    if dnPhantom == 0 {
        return CR_INVALID_DEVNODE;
    }

    if ulFlags != 0 {
        return CR_INVALID_FLAG;
    }

    if !hMachine.is_null() {
        BindingHandle = (*(hMachine as PMACHINE_INFO)).BindingHandle;
        if BindingHandle.is_null() {
            return CR_FAILURE;
        }

        StringTable = (*(hMachine as PMACHINE_INFO)).StringTable;
        if StringTable == core::ptr::null_mut() {
            return CR_FAILURE;
        }
    } else {
        if PnpGetLocalHandles(
            core::ptr::addr_of_mut!(BindingHandle),
            core::ptr::addr_of_mut!(StringTable),
        ) == 0
        {
            return CR_FAILURE;
        }
    }

    lpDevInst = pSetupStringTableStringFromId(StringTable, dnPhantom as DWORD);
    if lpDevInst.is_null() {
        return CR_INVALID_DEVNODE;
    }

    // RpcTryExcept
    ret = rpc_try(|| PNP_UninstallDevInst(BindingHandle as handle_t, lpDevInst, ulFlags as DWORD)).unwrap_or_else(|code| RpcStatusToCmStatus(code as _) as _) as CONFIGRET;
    // RpcExcept(EXCEPTION_EXECUTE_HANDLER)
    // RpcEndExcept;

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Unregister_Device_InterfaceA(
    pszDeviceInterface: LPCSTR,
    ulFlags: ULONG,
) -> CONFIGRET {
    TRACE!("CM_Unregister_Device_InterfaceA(%s %lx)\n", debugstr_a(pszDeviceInterface), ulFlags);
    return CM_Unregister_Device_Interface_ExA(pszDeviceInterface, ulFlags, core::ptr::null_mut());
}
#[no_mangle]
pub unsafe extern "system" fn CM_Unregister_Device_InterfaceW(pszDeviceInterface: LPCWSTR, ulFlags: ULONG) -> CONFIGRET {
    if (*__wine_dbch___default).flags as ::core::ffi::c_int & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE) as i32;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/setupapi/cfgmgr.c\0" as *const u8 as *const ::core::ffi::c_char,
            b"CM_Unregister_Device_InterfaceW\0" as *const u8 as *const ::core::ffi::c_char,
            8790 as ::core::ffi::c_int,
            b"CM_Unregister_Device_InterfaceW(%s %lx)\n\0" as *const u8 as *const ::core::ffi::c_char,
            debugstr_w(pszDeviceInterface as *const WCHAR),
            ulFlags,
        );
    }
    return CM_Unregister_Device_Interface_ExW(pszDeviceInterface, ulFlags, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn CM_Unregister_Device_Interface_ExA(
    pszDeviceInterface: LPCSTR,
    ulFlags: ULONG,
    hMachine: HMACHINE,
) -> CONFIGRET {
    let mut pszDeviceInterfaceW: LPWSTR = core::ptr::null_mut();
    let mut ret: CONFIGRET;

    if pszDeviceInterface.is_null() {
        return CR_INVALID_POINTER as CONFIGRET;
    }

    if pSetupCaptureAndConvertAnsiArg(pszDeviceInterface as PCSTR, core::ptr::addr_of_mut!(pszDeviceInterfaceW)) != 0 {
        return CR_INVALID_DATA as CONFIGRET;
    }

    ret = CM_Unregister_Device_Interface_ExW(pszDeviceInterfaceW as LPCWSTR, ulFlags, hMachine);

    if !pszDeviceInterfaceW.is_null() {
        MyFree(pszDeviceInterfaceW as PVOID);
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Unregister_Device_Interface_ExW(mut pszDeviceInterface: LPCWSTR, mut ulFlags: ULONG, mut hMachine: HMACHINE) -> CONFIGRET {
    let mut BindingHandle: RPC_BINDING_HANDLE = core::ptr::null_mut();
    let mut ret: CONFIGRET = 0;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE) as i32;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/setupapi/cfgmgr.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"CM_Unregister_Device_Interface_ExW\0" as *const u8 as *const ::core::ffi::c_char,
            8843 as ::core::ffi::c_int,
            b"CM_Unregister_Device_Interface_ExW(%s %lx %p)\n\0" as *const u8
                as *const ::core::ffi::c_char,
            debugstr_w(pszDeviceInterface as *const WCHAR),
            ulFlags,
            hMachine,
        );
    }
    if pszDeviceInterface.is_null() {
        return CR_INVALID_POINTER;
    }
    if ulFlags != 0 {
        return CR_INVALID_FLAG;
    }
    if !hMachine.is_null() {
        BindingHandle = (*(hMachine as PMACHINE_INFO)).BindingHandle;
        if BindingHandle.is_null() {
            return CR_FAILURE;
        }
    } else if PnpGetLocalHandles(
        &raw mut BindingHandle,
        ::core::ptr::null_mut::<HSTRING_TABLE>(),
    ) == 0
    {
        return CR_FAILURE;
    }
    ret = rpc_try(|| PNP_UnregisterDeviceClassAssociation(
        BindingHandle as handle_t,
        pszDeviceInterface as LPWSTR,
        ulFlags as DWORD,
    )).unwrap_or_else(|code| RpcStatusToCmStatus(code as _) as _) as CONFIGRET;
    return ret;
}
