use crate::setupapi::cfgmgr::CM_Get_DevNode_Registry_Property_ExW;
use crate::setupapi::cfgmgr::CM_Get_Device_ID_ExW;
use crate::setupapi::cfgmgr::CM_Open_DevNode_Key_Ex;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_007D169EA923: [u16; 11] = [
    0x0050u16, 0x0072, 0x006F, 0x0070, 0x0065, 0x0072, 0x0074, 0x0069, 0x0065, 0x0073,
    0x0000,
];
pub static RUSTCONV_WSTR_20DD19EBCF98: [u16; 12] = [
    0x0050u16, 0x0072, 0x006F, 0x0070, 0x0065, 0x0072, 0x0074, 0x0069, 0x0065, 0x0073,
    0x005C, 0x0000,
];
pub static RUSTCONV_WSTR_B7FC02907B1F: [u16; 2] = [0x005Cu16, 0x0000];
pub static RUSTCONV_WSTR_CFA00EC2E978: [u16; 6] = [0x005Cu16, 0x0025, 0x0030, 0x0034, 0x0058, 0x0000];
pub static RUSTCONV_WSTR_FF65F7182CB1: [u16; 51] = [
    0x007Bu16, 0x0025, 0x0030, 0x0038, 0x0058, 0x002D, 0x0025, 0x0030, 0x0034, 0x0058,
    0x002D, 0x0025, 0x0030, 0x0034, 0x0058, 0x002D, 0x0025, 0x0030, 0x0032, 0x0058,
    0x0025, 0x0030, 0x0032, 0x0058, 0x002D, 0x0025, 0x0030, 0x0032, 0x0058, 0x0025,
    0x0030, 0x0032, 0x0058, 0x0025, 0x0030, 0x0032, 0x0058, 0x0025, 0x0030, 0x0032,
    0x0058, 0x0025, 0x0030, 0x0032, 0x0058, 0x0025, 0x0030, 0x0032, 0x0058, 0x007D,
    0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static RUSTCONV_C2R_WSTR_069A7D4B45B6: [u16; 15] = [
    0x0044u16, 0x0072, 0x0069, 0x0076, 0x0065, 0x0072, 0x0044, 0x0061, 0x0074, 0x0065,
    0x0044, 0x0061, 0x0074, 0x0061, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_36B976A63B1A: [u16; 14] = [
    0x0044u16, 0x0072, 0x0069, 0x0076, 0x0065, 0x0072, 0x0056, 0x0065, 0x0072, 0x0073,
    0x0069, 0x006F, 0x006E, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_756844D34686: [u16; 8] = [0x0049u16, 0x006E, 0x0066, 0x0050, 0x0061, 0x0074, 0x0068, 0x0000];
pub static RUSTCONV_C2R_WSTR_9EF518E24D40: [u16; 17] = [
    0x004Du16, 0x0061, 0x0074, 0x0063, 0x0068, 0x0069, 0x006E, 0x0067, 0x0044, 0x0065,
    0x0076, 0x0069, 0x0063, 0x0065, 0x0049, 0x0064, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_AB10087FF1D1: [u16; 14] = [
    0x0049u16, 0x006E, 0x0066, 0x0053, 0x0065, 0x0063, 0x0074, 0x0069, 0x006F, 0x006E,
    0x0045, 0x0078, 0x0074, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_BF4CD342C46C: [u16; 11] = [
    0x0044u16, 0x0072, 0x0069, 0x0076, 0x0065, 0x0072, 0x0044, 0x0065, 0x0073, 0x0063,
    0x0000,
];
pub static RUSTCONV_C2R_WSTR_D8F4007C07BF: [u16; 11] = [
    0x0049u16, 0x006E, 0x0066, 0x0053, 0x0065, 0x0063, 0x0074, 0x0069, 0x006F, 0x006E,
    0x0000,
];
pub static RUSTCONV_C2R_WSTR_FCF60029E473: [u16; 13] = [
    0x0050u16, 0x0072, 0x006F, 0x0076, 0x0069, 0x0064, 0x0065, 0x0072, 0x004E, 0x0061,
    0x006D, 0x0065, 0x0000,
];

static mut DpFmtidDevice: GUID = _GUID {
    Data1: 0xa45c254e as ::core::ffi::c_uint,
    Data2: 0xdf1c as ::core::ffi::c_ushort,
    Data3: 0x4efd as ::core::ffi::c_ushort,
    Data4: [
        0x80 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x20 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x67 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x46 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x50 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

static mut DpFmtidInstance: GUID = _GUID {
    Data1: 0x78c34fc8 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x104a as ::core::ffi::c_ushort,
    Data3: 0x4aca as ::core::ffi::c_ushort,
    Data4: [
        0x9e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x52 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x52 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x99 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x57 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

static mut DpFmtidDriver: GUID = _GUID {
    Data1: 0xa8b865dd as ::core::ffi::c_uint,
    Data2: 0x2e3d as ::core::ffi::c_ushort,
    Data3: 0x4094 as ::core::ffi::c_ushort,
    Data4: [
        0xad as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x97 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x93 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x75 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

static mut DpFmtidName: GUID = _GUID {
    Data1: 0xb725f130 as ::core::ffi::c_uint,
    Data2: 0x47ef as ::core::ffi::c_ushort,
    Data3: 0x101a as ::core::ffi::c_ushort,
    Data4: [
        0xa5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x60 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xeb as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xac as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

static mut DpDeviceMap: [DpDeviceMapEntry; 32] = [
    DpDeviceMapEntry {
        pid: 2 as ULONG,
        spdrp: SPDRP_DEVICEDESC as ULONG,
        kind: DPK_STRING,
    },
    DpDeviceMapEntry {
        pid: 3 as ULONG,
        spdrp: SPDRP_HARDWAREID as ULONG,
        kind: DPK_MULTISZ,
    },
    DpDeviceMapEntry {
        pid: 4 as ULONG,
        spdrp: SPDRP_COMPATIBLEIDS as ULONG,
        kind: DPK_MULTISZ,
    },
    DpDeviceMapEntry {
        pid: 6 as ULONG,
        spdrp: SPDRP_SERVICE as ULONG,
        kind: DPK_STRING,
    },
    DpDeviceMapEntry {
        pid: 9 as ULONG,
        spdrp: SPDRP_CLASS as ULONG,
        kind: DPK_STRING,
    },
    DpDeviceMapEntry {
        pid: 10 as ULONG,
        spdrp: SPDRP_CLASSGUID as ULONG,
        kind: DPK_GUID,
    },
    DpDeviceMapEntry {
        pid: 11 as ULONG,
        spdrp: SPDRP_DRIVER as ULONG,
        kind: DPK_STRING,
    },
    DpDeviceMapEntry {
        pid: 12 as ULONG,
        spdrp: SPDRP_CONFIGFLAGS as ULONG,
        kind: DPK_UINT32,
    },
    DpDeviceMapEntry {
        pid: 13 as ULONG,
        spdrp: SPDRP_MFG as ULONG,
        kind: DPK_STRING,
    },
    DpDeviceMapEntry {
        pid: 14 as ULONG,
        spdrp: SPDRP_FRIENDLYNAME as ULONG,
        kind: DPK_STRING,
    },
    DpDeviceMapEntry {
        pid: 15 as ULONG,
        spdrp: SPDRP_LOCATION_INFORMATION as ULONG,
        kind: DPK_STRING,
    },
    DpDeviceMapEntry {
        pid: 16 as ULONG,
        spdrp: SPDRP_PHYSICAL_DEVICE_OBJECT_NAME as ULONG,
        kind: DPK_STRING,
    },
    DpDeviceMapEntry {
        pid: 17 as ULONG,
        spdrp: SPDRP_CAPABILITIES as ULONG,
        kind: DPK_UINT32,
    },
    DpDeviceMapEntry {
        pid: 18 as ULONG,
        spdrp: SPDRP_UI_NUMBER as ULONG,
        kind: DPK_UINT32,
    },
    DpDeviceMapEntry {
        pid: 19 as ULONG,
        spdrp: SPDRP_UPPERFILTERS as ULONG,
        kind: DPK_MULTISZ,
    },
    DpDeviceMapEntry {
        pid: 20 as ULONG,
        spdrp: SPDRP_LOWERFILTERS as ULONG,
        kind: DPK_MULTISZ,
    },
    DpDeviceMapEntry {
        pid: 21 as ULONG,
        spdrp: SPDRP_BUSTYPEGUID as ULONG,
        kind: DPK_GUID,
    },
    DpDeviceMapEntry {
        pid: 22 as ULONG,
        spdrp: SPDRP_LEGACYBUSTYPE as ULONG,
        kind: DPK_UINT32,
    },
    DpDeviceMapEntry {
        pid: 23 as ULONG,
        spdrp: SPDRP_BUSNUMBER as ULONG,
        kind: DPK_UINT32,
    },
    DpDeviceMapEntry {
        pid: 24 as ULONG,
        spdrp: SPDRP_ENUMERATOR_NAME as ULONG,
        kind: DPK_STRING,
    },
    DpDeviceMapEntry {
        pid: 25 as ULONG,
        spdrp: SPDRP_SECURITY as ULONG,
        kind: DPK_SD,
    },
    DpDeviceMapEntry {
        pid: 26 as ULONG,
        spdrp: SPDRP_SECURITY_SDS as ULONG,
        kind: DPK_SDS,
    },
    DpDeviceMapEntry {
        pid: 27 as ULONG,
        spdrp: SPDRP_DEVTYPE as ULONG,
        kind: DPK_UINT32,
    },
    DpDeviceMapEntry {
        pid: 28 as ULONG,
        spdrp: SPDRP_EXCLUSIVE as ULONG,
        kind: DPK_BOOL,
    },
    DpDeviceMapEntry {
        pid: 29 as ULONG,
        spdrp: SPDRP_CHARACTERISTICS as ULONG,
        kind: DPK_UINT32,
    },
    DpDeviceMapEntry {
        pid: 30 as ULONG,
        spdrp: SPDRP_ADDRESS as ULONG,
        kind: DPK_UINT32,
    },
    DpDeviceMapEntry {
        pid: 31 as ULONG,
        spdrp: SPDRP_UI_NUMBER_DESC_FORMAT as ULONG,
        kind: DPK_STRING,
    },
    DpDeviceMapEntry {
        pid: 32 as ULONG,
        spdrp: SPDRP_DEVICE_POWER_DATA as ULONG,
        kind: DPK_BINARY,
    },
    DpDeviceMapEntry {
        pid: 33 as ULONG,
        spdrp: SPDRP_REMOVAL_POLICY as ULONG,
        kind: DPK_UINT32,
    },
    DpDeviceMapEntry {
        pid: 34 as ULONG,
        spdrp: SPDRP_REMOVAL_POLICY_HW_DEFAULT as ULONG,
        kind: DPK_UINT32,
    },
    DpDeviceMapEntry {
        pid: 35 as ULONG,
        spdrp: SPDRP_REMOVAL_POLICY_OVERRIDE as ULONG,
        kind: DPK_UINT32,
    },
    DpDeviceMapEntry {
        pid: 36 as ULONG,
        spdrp: SPDRP_INSTALL_STATE as ULONG,
        kind: DPK_UINT32,
    },
];

static mut DpDriverMap: [DpDriverMapEntry; 8] = unsafe {
    [
        DpDriverMapEntry {
            pid: 2 as ULONG,
            valueName: RUSTCONV_C2R_WSTR_069A7D4B45B6.as_ptr() as PCWSTR,
            kind: DPK_FILETIME,
        },
        DpDriverMapEntry {
            pid: 3 as ULONG,
            valueName: RUSTCONV_C2R_WSTR_36B976A63B1A.as_ptr() as PCWSTR,
            kind: DPK_STRING,
        },
        DpDriverMapEntry {
            pid: 4 as ULONG,
            valueName: RUSTCONV_C2R_WSTR_BF4CD342C46C.as_ptr() as PCWSTR,
            kind: DPK_STRING,
        },
        DpDriverMapEntry {
            pid: 5 as ULONG,
            valueName: RUSTCONV_C2R_WSTR_756844D34686.as_ptr() as PCWSTR,
            kind: DPK_STRING,
        },
        DpDriverMapEntry {
            pid: 6 as ULONG,
            valueName: RUSTCONV_C2R_WSTR_D8F4007C07BF.as_ptr() as PCWSTR,
            kind: DPK_STRING,
        },
        DpDriverMapEntry {
            pid: 7 as ULONG,
            valueName: RUSTCONV_C2R_WSTR_AB10087FF1D1.as_ptr() as PCWSTR,
            kind: DPK_STRING,
        },
        DpDriverMapEntry {
            pid: 8 as ULONG,
            valueName: RUSTCONV_C2R_WSTR_9EF518E24D40.as_ptr() as PCWSTR,
            kind: DPK_STRING,
        },
        DpDriverMapEntry {
            pid: 9 as ULONG,
            valueName: RUSTCONV_C2R_WSTR_FCF60029E473.as_ptr() as PCWSTR,
            kind: DPK_STRING,
        },
    ]
};

pub const DP_PID_INSTANCEID: u32 = 256;
pub const DP_PID_NAME: u32 = 10;

// SETUPAPI device property registry constants (setupapi.h)


// enum DpKind
pub type DpKind = i32;
pub const DPK_STRING: DpKind = 0;     /* REG_SZ            -> DEVPROP_TYPE_STRING */
pub const DPK_MULTISZ: DpKind = 1;    /* REG_MULTI_SZ      -> DEVPROP_TYPE_STRING_LIST */
pub const DPK_UINT32: DpKind = 2;     /* REG_DWORD         -> DEVPROP_TYPE_UINT32 */
pub const DPK_BOOL: DpKind = 3;       /* REG_DWORD         -> DEVPROP_TYPE_BOOLEAN (1 byte, -1 / 0) */
pub const DPK_GUID: DpKind = 4;       /* "{guid}" or 16 raw bytes -> DEVPROP_TYPE_GUID */
pub const DPK_BINARY: DpKind = 5;     /* REG_BINARY        -> DEVPROP_TYPE_BINARY */
pub const DPK_SD: DpKind = 6;         /* REG_BINARY        -> DEVPROP_TYPE_SECURITY_DESCRIPTOR */
pub const DPK_SDS: DpKind = 7;        /* SDDL string       -> DEVPROP_TYPE_SECURITY_DESCRIPTOR_STRING */
pub const DPK_FILETIME: DpKind = 8;   /* 8 bytes binary    -> DEVPROP_TYPE_FILETIME */

#[repr(C)]
pub struct DpDeviceMapEntry {
    pub pid: ULONG,      /* DEVPKEY_Device_xxx pid; always spdrp + 2 (verified against devpkey.h) */
    pub spdrp: ULONG,    /* SPDRP_xxx */
    pub kind: DpKind,
}

#[repr(C)]
pub struct DpDriverMapEntry {
    pub pid: ULONG,              /* DEVPKEY_Device_Driver* pid under DpFmtidDriver */
    pub valueName: PCWSTR,       /* value in the device's driver (software) key */
    pub kind: DpKind,
}

#[repr(C)]
pub struct _DPCTX {
    pub setupdi: BOOL,               /* TRUE: SetupDi entry point, FALSE: CM entry point */
    pub hDevInfo: HDEVINFO,          /* SetupDi path */
    pub pData: PSP_DEVINFO_DATA,     /* SetupDi path */
    pub devInfo: *mut DeviceInfo,    /* SetupDi path */
    pub set: *mut DeviceInfoSet,     /* SetupDi path */
    pub devInst: DEVINST,
    pub hMachine: HMACHINE,
    pub instanceId: PCWSTR,          /* lazily resolved on the CM path */
    pub idBuf: [WCHAR; (MAX_DEVICE_ID_LEN as usize)],
}
pub type DPCTX = _DPCTX;

#[repr(C)]
pub struct _DPKEYS {
    pub k: *mut DEVPROPKEY,
    pub n: DWORD,
    pub cap: DWORD,
    pub err: DWORD,
}
pub type DPKEYS = _DPKEYS;

unsafe extern "C" fn dp_cr_to_error(cr: CONFIGRET) -> DWORD {
    match cr {
        CR_SUCCESS => return ERROR_SUCCESS,
        CR_BUFFER_SMALL => return ERROR_INSUFFICIENT_BUFFER,
        CR_NO_SUCH_VALUE | CR_INVALID_PROPERTY => return ERROR_NOT_FOUND,
        CR_INVALID_DEVNODE => return ERROR_NO_SUCH_DEVINST,
        CR_OUT_OF_MEMORY => return ERROR_NOT_ENOUGH_MEMORY,
        CR_INVALID_POINTER | CR_INVALID_FLAG | CR_INVALID_DATA => return ERROR_INVALID_PARAMETER,
        CR_ACCESS_DENIED => return ERROR_ACCESS_DENIED,
        _ => return ERROR_GEN_FAILURE,
    }
}

unsafe extern "C" fn dp_error_to_cr(err: DWORD) -> CONFIGRET {
    match err {
        ERROR_SUCCESS => { return CR_SUCCESS as CONFIGRET; }
        ERROR_INSUFFICIENT_BUFFER => { return CR_BUFFER_SMALL as CONFIGRET; }
        ERROR_NOT_FOUND => { return CR_NO_SUCH_VALUE as CONFIGRET; }
        ERROR_NO_SUCH_DEVINST => { return CR_INVALID_DEVNODE as CONFIGRET; }
        ERROR_NOT_ENOUGH_MEMORY => { return CR_OUT_OF_MEMORY as CONFIGRET; }
        ERROR_ACCESS_DENIED => { return CR_ACCESS_DENIED as CONFIGRET; }
        _ => { return CR_FAILURE as CONFIGRET; }
    }
}

unsafe extern "C" fn dp_get_instance_id(c: *mut DPCTX, id: *mut PCWSTR) -> DWORD {
    if (*c).instanceId.is_null() {
        let cr: CONFIGRET = CM_Get_Device_ID_ExW(
            (*c).devInst,
            core::ptr::addr_of_mut!((*c).idBuf) as PWCHAR,
            MAX_DEVICE_ID_LEN as ULONG,
            0 as ULONG,
            (*c).hMachine,
        );
        if cr != CR_SUCCESS as CONFIGRET {
            return dp_cr_to_error(cr);
        }
        (*c).instanceId = core::ptr::addr_of_mut!((*c).idBuf) as *mut WCHAR as PCWSTR;
    }
    *id = (*c).instanceId;
    return ERROR_SUCCESS as DWORD;
}

unsafe extern "C" fn dp_normalize_missing(err: DWORD) -> DWORD {
    match err {
        ERROR_FILE_NOT_FOUND | ERROR_PATH_NOT_FOUND | ERROR_INVALID_DATA | ERROR_INVALID_REG_PROPERTY => {
            return ERROR_NOT_FOUND;
        }
        _ => {
            return err;
        }
    }
}

unsafe extern "C" fn dp_read_reg(c: *mut DPCTX, spdrp: ULONG, regType: *mut ULONG, buf: *mut BYTE, size: *mut ULONG) -> DWORD {
    let mut type_0: ULONG = REG_NONE;

    if (*c).setupdi != 0 {
        let mut req: DWORD = 0;
        let mut err: DWORD;

        if SetupDiGetDeviceRegistryPropertyW(
            (*c).hDevInfo,
            (*c).pData,
            spdrp as DWORD,
            core::ptr::addr_of_mut!(type_0),
            buf as PBYTE,
            *size,
            core::ptr::addr_of_mut!(req),
        ) != 0
        {
            *regType = type_0;
            *size = req as ULONG;
            return ERROR_SUCCESS as DWORD;
        }
        err = GetLastError();
        if err == ERROR_INSUFFICIENT_BUFFER as DWORD {
            *regType = type_0;
            *size = req as ULONG;
        }
        return dp_normalize_missing(err);
    } else {
        /* CM_Get_DevNode_Registry_Property_ExW rejects a zero length: callers always pass >= 1. */
        let mut len: ULONG = *size;
        let mut cr: CONFIGRET = CM_Get_DevNode_Registry_Property_ExW(
            (*c).devInst,
            spdrp.wrapping_add((CM_DRP_DEVICEDESC - SPDRP_DEVICEDESC) as ULONG),
            core::ptr::addr_of_mut!(type_0),
            buf as PVOID,
            core::ptr::addr_of_mut!(len),
            0 as ULONG,
            (*c).hMachine,
        );
        if cr == CR_SUCCESS as CONFIGRET || cr == CR_BUFFER_SMALL as CONFIGRET {
            *regType = type_0;
            *size = len;
        }
        return dp_cr_to_error(cr);
    }
}

unsafe extern "C" fn dp_read_reg_alloc(
    c: *mut DPCTX,
    spdrp: ULONG,
    regType: *mut ULONG,
    data: *mut *mut BYTE,
    len: *mut ULONG,
) -> DWORD {
    let mut cap: ULONG = 128 as ULONG;
    let mut i: ULONG = 0 as ULONG;
    let mut p: *mut BYTE = core::ptr::null_mut();
    while i < 4 as ULONG {
        let mut size: ULONG = cap;
        let mut err: DWORD = 0;
        p = HeapAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY as DWORD,
            cap.wrapping_add(4 as ULONG) as SIZE_T,
        ) as *mut BYTE;
        if p.is_null() {
            return ERROR_NOT_ENOUGH_MEMORY as DWORD;
        }
        err = dp_read_reg(c, spdrp, regType, p, core::ptr::addr_of_mut!(size));
        if err == ERROR_SUCCESS as DWORD {
            *data = p;
            *len = size;
            return ERROR_SUCCESS as DWORD;
        }
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            p as PVOID,
        );
        if err != ERROR_INSUFFICIENT_BUFFER as DWORD || size <= cap {
            return if err == ERROR_INSUFFICIENT_BUFFER as DWORD {
                ERROR_GEN_FAILURE as DWORD
            } else {
                err
            };
        }
        cap = size;
        i = i.wrapping_add(1);
    }
    return ERROR_GEN_FAILURE as DWORD;
}

unsafe extern "C" fn dp_emit(type_: DEVPROPTYPE, data: *const core::ffi::c_void, len: DWORD, outType: *mut DEVPROPTYPE, buf: *mut BYTE, bufSize: DWORD, req: *mut DWORD) -> DWORD {
    *outType = type_;
    if !req.is_null() {
        *req = len;
    }
    if len > bufSize {
        return ERROR_INSUFFICIENT_BUFFER;
    }
    if len != 0 {
        memcpy(buf as *mut core::ffi::c_void, data, len as size_t);
    }
    return ERROR_SUCCESS;
}

unsafe extern "C" fn dp_parse_guid(s: *const WCHAR, guid: *mut GUID) -> BOOL {
    let mut tmp: [WCHAR; 37] = [0; 37];

    if lstrlenW(s as LPCWSTR) < 38
        || *s.offset(0 as isize) != '{' as WCHAR
        || *s.offset(37 as isize) != '}' as WCHAR
    {
        return (FALSE) as BOOL;
    }
    memcpy(
        &raw mut tmp as *mut WCHAR as *mut ::core::ffi::c_void,
        s.offset(1 as isize) as *const ::core::ffi::c_void,
        (36 as size_t).wrapping_mul(::core::mem::size_of::<WCHAR>() as size_t),
    );
    tmp[36 as usize] = 0 as WCHAR;
    return (UuidFromStringW(&raw mut tmp as RPC_WSTR, guid as *mut UUID)
        == RPC_S_OK as RPC_STATUS) as ::core::ffi::c_int;
}

unsafe extern "C" fn dp_guid_to_string(guid: *const GUID, out: *mut WCHAR) {
    sprintfW(
        out,
        RUSTCONV_WSTR_FF65F7182CB1.as_ptr(),
        (*guid).Data1,
        (*guid).Data2 as i32,
        (*guid).Data3 as i32,
        (*guid).Data4[0 as usize] as i32,
        (*guid).Data4[1 as usize] as i32,
        (*guid).Data4[2 as usize] as i32,
        (*guid).Data4[3 as usize] as i32,
        (*guid).Data4[4 as usize] as i32,
        (*guid).Data4[5 as usize] as i32,
        (*guid).Data4[6 as usize] as i32,
        (*guid).Data4[7 as usize] as i32,
    );
}

unsafe extern "C" fn dp_convert(
    kind: DpKind,
    regType: ULONG,
    data: *mut BYTE,
    len: ULONG,
    outType: *mut DEVPROPTYPE,
    buf: *mut BYTE,
    bufSize: DWORD,
    req: *mut DWORD,
) -> DWORD {
    match (kind as u32) as i32 {
        DPK_STRING | DPK_SDS => {
            let s: *mut WCHAR = data as *mut WCHAR;
            let mut n: ULONG = len / core::mem::size_of::<WCHAR>() as ULONG;

            if regType != REG_SZ as ULONG && regType != REG_EXPAND_SZ as ULONG {
                return ERROR_INVALID_DATA as DWORD;
            }
            if n == 0 || *s.offset(n.wrapping_sub(1 as ULONG) as isize) != 0 {
                n = n.wrapping_add(1); /* add the NUL the store lacked */
            }
            return dp_emit(
                (if (kind as u32) == (DPK_STRING as u32) {
                    DEVPROP_TYPE_STRING
                } else {
                    DEVPROP_TYPE_SECURITY_DESCRIPTOR_STRING
                }) as DEVPROPTYPE,
                s as *const core::ffi::c_void,
                (n * core::mem::size_of::<WCHAR>() as ULONG) as DWORD,
                outType,
                buf,
                bufSize,
                req,
            );
        }
        DPK_MULTISZ => {
            let s: *mut WCHAR = data as *mut WCHAR;
            let mut n: ULONG = len / core::mem::size_of::<WCHAR>() as ULONG;

            if regType != REG_MULTI_SZ as ULONG {
                return ERROR_INVALID_DATA as DWORD;
            }
            if n == 0 {
                n = 2 as ULONG; /* empty list = two NULs */
            } else {
                if *s.offset(n.wrapping_sub(1 as ULONG) as isize) != 0 {
                    n = n.wrapping_add(1); /* terminate the last string */
                }
                if n < 2 as ULONG
                    || *s.offset(n.wrapping_sub(2 as ULONG) as isize) != 0
                {
                    n = n.wrapping_add(1); /* list terminator */
                }
            }
            return dp_emit(
                DEVPROP_TYPE_STRING_LIST as DEVPROPTYPE,
                s as *const core::ffi::c_void,
                (n * core::mem::size_of::<WCHAR>() as ULONG) as DWORD,
                outType,
                buf,
                bufSize,
                req,
            );
        }
        DPK_UINT32 | DPK_BOOL => {
            let mut v: DWORD = 0;

            if !(regType == REG_DWORD as ULONG || regType == REG_BINARY as ULONG)
                || len < core::mem::size_of::<DWORD>() as ULONG
            {
                return ERROR_INVALID_DATA as DWORD;
            }
            memcpy(
                core::ptr::addr_of_mut!(v) as *mut core::ffi::c_void,
                data as *const core::ffi::c_void,
                core::mem::size_of::<DWORD>() as usize,
            );
            if (kind as u32) == (DPK_UINT32 as u32) {
                return dp_emit(
                    DEVPROP_TYPE_UINT32 as DEVPROPTYPE,
                    core::ptr::addr_of_mut!(v) as *const core::ffi::c_void,
                    core::mem::size_of::<DWORD>() as DWORD,
                    outType,
                    buf,
                    bufSize,
                    req,
                );
            } else {
                let mut b: DEVPROP_BOOLEAN = (if v != 0 {
                    DEVPROP_TRUE
                } else {
                    DEVPROP_FALSE
                }) as DEVPROP_BOOLEAN;
                return dp_emit(
                    DEVPROP_TYPE_BOOLEAN as DEVPROPTYPE,
                    core::ptr::addr_of_mut!(b) as *const core::ffi::c_void,
                    core::mem::size_of::<DEVPROP_BOOLEAN>() as DWORD,
                    outType,
                    buf,
                    bufSize,
                    req,
                );
            }
        }
        DPK_GUID => {
            let mut g: GUID = core::mem::zeroed::<GUID>();

            if (regType == REG_SZ as ULONG || regType == REG_EXPAND_SZ as ULONG)
                && dp_parse_guid(data as *mut WCHAR, core::ptr::addr_of_mut!(g)) != 0
            {
                /* parsed as SDDL-style GUID string */
            } else if regType == REG_BINARY as ULONG && len == core::mem::size_of::<GUID>() as ULONG {
                memcpy(
                    core::ptr::addr_of_mut!(g) as *mut core::ffi::c_void,
                    data as *const core::ffi::c_void,
                    core::mem::size_of::<GUID>() as usize,
                );
            } else {
                return ERROR_INVALID_DATA as DWORD;
            }
            return dp_emit(
                DEVPROP_TYPE_GUID as DEVPROPTYPE,
                core::ptr::addr_of_mut!(g) as *const core::ffi::c_void,
                core::mem::size_of::<GUID>() as DWORD,
                outType,
                buf,
                bufSize,
                req,
            );
        }
        DPK_BINARY => {
            return dp_emit(
                DEVPROP_TYPE_BINARY as DEVPROPTYPE,
                data as *const core::ffi::c_void,
                len as DWORD,
                outType,
                buf,
                bufSize,
                req,
            );
        }
        DPK_SD => {
            return dp_emit(
                DEVPROP_TYPE_SECURITY_DESCRIPTOR as DEVPROPTYPE,
                data as *const core::ffi::c_void,
                len as DWORD,
                outType,
                buf,
                bufSize,
                req,
            );
        }
        DPK_FILETIME => {
            if regType != REG_BINARY as ULONG
                || len != core::mem::size_of::<FILETIME>() as ULONG
            {
                return ERROR_INVALID_DATA as DWORD;
            }
            return dp_emit(
                DEVPROP_TYPE_FILETIME as DEVPROPTYPE,
                data as *const core::ffi::c_void,
                len as DWORD,
                outType,
                buf,
                bufSize,
                req,
            );
        }
        _ => {}
    }
    return ERROR_INVALID_DATA as DWORD;
}

unsafe extern "C" fn dp_get_mapped(c: *mut DPCTX, m: *const DpDeviceMapEntry, type_: *mut DEVPROPTYPE, buf: *mut BYTE, bufSize: DWORD, req: *mut DWORD) -> DWORD {
    let mut regType: ULONG = REG_NONE as ULONG;
    let mut len: ULONG = 0 as ULONG;
    let mut data: *mut BYTE = core::ptr::null_mut::<BYTE>();
    let mut err: DWORD;
    err = dp_read_reg_alloc(c, (*m).spdrp, core::ptr::addr_of_mut!(regType), core::ptr::addr_of_mut!(data), core::ptr::addr_of_mut!(len));
    if err != ERROR_SUCCESS as DWORD {
        return err;
    }
    err = dp_convert((*m).kind, regType, data, len, type_, buf, bufSize, req);
    HeapFree(GetProcessHeap(), 0 as DWORD, data as PVOID);
    return if err == ERROR_INVALID_DATA as DWORD {
        ERROR_NOT_FOUND as DWORD
    } else {
        err
    };
}

unsafe extern "C" fn dp_open_driver_key(c: *mut DPCTX) -> HKEY {
    let mut key: HKEY = core::ptr::null_mut();

    if (*c).setupdi != 0 {
        key = SETUPDI_OpenDrvKey((*(*c).set).HKLM, (*c).devInfo, KEY_QUERY_VALUE as REGSAM);
        return if key == INVALID_HANDLE_VALUE as HKEY {
            core::ptr::null_mut()
        } else {
            key
        };
    }
    if CM_Open_DevNode_Key_Ex(
        (*c).devInst,
        KEY_QUERY_VALUE as REGSAM,
        0 as ULONG,
        RegDisposition_OpenExisting as REGDISPOSITION,
        core::ptr::addr_of_mut!(key),
        CM_REGISTRY_SOFTWARE as ULONG,
        (*c).hMachine,
    ) != CR_SUCCESS as CONFIGRET
    {
        return core::ptr::null_mut();
    }
    return key;
}

unsafe extern "C" fn dp_get_driver(c: *mut DPCTX, m: *const DpDriverMapEntry, type_: *mut DEVPROPTYPE, buf: *mut BYTE, bufSize: DWORD, req: *mut DWORD) -> DWORD {
    let mut key: HKEY = core::ptr::null_mut();
    let mut regType: DWORD = REG_NONE as DWORD;
    let mut size: DWORD = 0 as DWORD;
    let mut err: DWORD;
    let mut data: *mut BYTE;

    /* No driver key (driver not installed) or no such value: not found. */
    key = dp_open_driver_key(c);
    if key.is_null() {
        return ERROR_NOT_FOUND as DWORD;
    }
    err = RegQueryValueExW(
        key,
        (*m).valueName as LPCWSTR,
        core::ptr::null_mut::<DWORD>(),
        core::ptr::addr_of_mut!(regType),
        core::ptr::null_mut::<BYTE>(),
        core::ptr::addr_of_mut!(size),
    ) as DWORD;
    if err != ERROR_SUCCESS as DWORD {
        RegCloseKey(key);
        return dp_normalize_missing(err);
    }
    data = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY as DWORD,
        size.wrapping_add(4 as DWORD) as SIZE_T,
    ) as *mut BYTE;
    if data.is_null() {
        RegCloseKey(key);
        return ERROR_NOT_ENOUGH_MEMORY as DWORD;
    }
    err = RegQueryValueExW(
        key,
        (*m).valueName as LPCWSTR,
        core::ptr::null_mut::<DWORD>(),
        core::ptr::addr_of_mut!(regType),
        data as LPBYTE,
        core::ptr::addr_of_mut!(size),
    ) as DWORD;
    RegCloseKey(key);
    if err == ERROR_SUCCESS as DWORD {
        err = dp_convert(
            (*m).kind,
            regType as ULONG,
            data,
            size as ULONG,
            type_,
            buf,
            bufSize,
            req,
        );
        if err == ERROR_INVALID_DATA as DWORD {
            err = ERROR_NOT_FOUND as DWORD;
        }
    } else {
        err = dp_normalize_missing(err);
    }
    HeapFree(GetProcessHeap(), 0 as DWORD, data as PVOID);
    return err;
}

unsafe extern "C" fn dp_open_enum_key(c: *mut DPCTX, sam: REGSAM, key: *mut HKEY) -> DWORD {
    let mut path: [WCHAR; core::mem::size_of::<[u8; 60]>() / core::mem::size_of::<WCHAR>() + (MAX_DEVICE_ID_LEN as usize) + 2] = [0; core::mem::size_of::<[u8; 60]>() / core::mem::size_of::<WCHAR>() + (MAX_DEVICE_ID_LEN as usize) + 2];
    let mut id: PCWSTR = core::ptr::null::<WCHAR>();
    let mut hklm: HKEY = core::ptr::null_mut();
    let mut err: DWORD;

    if (*c).setupdi != 0 {
        hklm = (*(*c).set).HKLM;
    } else if (*c).hMachine.is_null() {
        hklm = HKEY_LOCAL_MACHINE;
    } else {
        return ERROR_NOT_FOUND; // see header: no remote registry here
    }

    err = dp_get_instance_id(c, &raw mut id);
    if err != 0 {
        return err;
    }
    lstrcpyW(&raw mut path as LPWSTR, REGSTR_PATH_SYSTEMENUM);
    lstrcatW(&raw mut path as LPWSTR, RUSTCONV_WSTR_B7FC02907B1F.as_ptr());
    lstrcatW(&raw mut path as LPWSTR, id as LPCWSTR);
    err = RegOpenKeyExW(hklm, &raw mut path as *mut WCHAR as LPCWSTR, 0, sam, key as PHKEY) as DWORD;
    return if err == ERROR_FILE_NOT_FOUND { ERROR_NO_SUCH_DEVINST } else { err };
}

unsafe extern "C" fn dp_get_registry(
    c: *mut DPCTX,
    key: *const DEVPROPKEY,
    type_0: *mut DEVPROPTYPE,
    buf: *mut BYTE,
    bufSize: DWORD,
    req: *mut DWORD,
) -> DWORD {
    let mut path: [WCHAR; 11 + 38 + 5 + 1] =
        [0; 11 + 38 + 5 + 1];
    let mut dev: HKEY = core::ptr::null_mut();
    let mut prop: HKEY = core::ptr::null_mut();
    let mut size: DWORD = 0;
    let mut valType: DWORD = 0;
    let mut err: DWORD;

    err = dp_open_enum_key(c, KEY_QUERY_VALUE as REGSAM, &raw mut dev);
    if err == ERROR_SUCCESS {
        lstrcpyW(&raw mut path as *mut WCHAR, RUSTCONV_WSTR_20DD19EBCF98.as_ptr());
        dp_guid_to_string(
            &raw const (*key).fmtid,
            (&raw mut path as *mut WCHAR).offset(11 as isize),
        );
        sprintfW(
            (&raw mut path as *mut WCHAR).offset(49 as isize),
            RUSTCONV_WSTR_CFA00EC2E978.as_ptr(),
            (*key).pid,
        );
        err = RegOpenKeyExW(
            dev,
            &raw mut path as *mut WCHAR as LPCWSTR,
            0 as DWORD,
            KEY_QUERY_VALUE as REGSAM,
            &raw mut prop,
        ) as DWORD;
        RegCloseKey(dev);
        if err == ERROR_SUCCESS {
            size = bufSize;
            err = RegQueryValueExW(
                prop,
                core::ptr::null::<WCHAR>(),
                core::ptr::null_mut::<DWORD>(),
                &raw mut valType,
                buf as LPBYTE,
                &raw mut size,
            ) as DWORD;
            RegCloseKey(prop);
        }
    }

    match err {
        ERROR_SUCCESS | ERROR_MORE_DATA => {
            *type_0 = (valType & 0xffff) as DEVPROPTYPE;
            if !req.is_null() {
                *req = size;
            }
            return (if err == ERROR_MORE_DATA || (buf.is_null() && size != 0) {
                ERROR_INSUFFICIENT_BUFFER
            } else {
                ERROR_SUCCESS
            }) as DWORD;
        }
        ERROR_FILE_NOT_FOUND | ERROR_PATH_NOT_FOUND => {
            err = ERROR_NOT_FOUND;
            /* fall through */
            *type_0 = DEVPROP_TYPE_EMPTY as DEVPROPTYPE;
            if !req.is_null() {
                *req = 0 as DWORD;
            }
            return err;
        }
        _ => {
            *type_0 = DEVPROP_TYPE_EMPTY as DEVPROPTYPE;
            if !req.is_null() {
                *req = 0 as DWORD;
            }
            return err;
        }
    }
}

unsafe extern "C" fn dp_find_device_map(mut pid: ULONG) -> *const DpDeviceMapEntry {
    let mut i: ULONG = 0;
    while (i as usize) < (*(&raw const DpDeviceMap)).len() {
        if DpDeviceMap[i as usize].pid == pid {
            return (&raw const DpDeviceMap as *const DpDeviceMapEntry).add(i as usize);
        }
        i = i.wrapping_add(1);
    }
    return core::ptr::null::<DpDeviceMapEntry>();
}

unsafe extern "C" fn dp_find_driver_map(pid: ULONG) -> *const DpDriverMapEntry {
    let mut i: ULONG;
    i = 0;
    while (i as usize) < ((*(&raw const DpDriverMap)).len() as usize) {
        if DpDriverMap[i as usize].pid == pid {
            return &DpDriverMap[i as usize] as *const DpDriverMapEntry;
        }
        i = i.wrapping_add(1);
    }
    return core::ptr::null::<DpDriverMapEntry>();
}

unsafe extern "C" fn dp_get(
    c: *mut DPCTX,
    key: *const DEVPROPKEY,
    type_0: *mut DEVPROPTYPE,
    buf: *mut BYTE,
    bufSize: DWORD,
    req: *mut DWORD,
) -> DWORD {
    let mut err: DWORD = ERROR_NOT_FOUND;

    if IsEqualGUID!(&raw const (*key).fmtid, &raw const DpFmtidInstance) != false
        && (*key).pid == DP_PID_INSTANCEID
    {
        let mut id: PCWSTR = core::ptr::null::<WCHAR>();

        err = dp_get_instance_id(c, &raw mut id);
        if err != 0 {
            return err;
        }
        return dp_emit(
            DEVPROP_TYPE_STRING,
            id as *const core::ffi::c_void,
            ((lstrlenW(id as LPCWSTR) + 1) as usize)
                .wrapping_mul(core::mem::size_of::<WCHAR>() as usize) as DWORD,
            type_0,
            buf,
            bufSize,
            req,
        );
    }
    if IsEqualGUID!(&raw const (*key).fmtid, &raw const DpFmtidName) != false
        && (*key).pid == DP_PID_NAME
    {
        /* DEVPKEY_NAME: friendly name, else the device description. */
        err = dp_get_mapped(
            c,
            dp_find_device_map(12),
            type_0,
            buf,
            bufSize,
            req,
        );
        if err == ERROR_NOT_FOUND {
            err = dp_get_mapped(
                c,
                dp_find_device_map(2),
                type_0,
                buf,
                bufSize,
                req,
            );
        }
    } else if IsEqualGUID!(&raw const (*key).fmtid, &raw const DpFmtidDevice) != false {
        let m: *const DpDeviceMapEntry = dp_find_device_map((*key).pid);

        if !m.is_null() {
            err = dp_get_mapped(c, m, type_0, buf, bufSize, req);
        }
    } else if IsEqualGUID!(&raw const (*key).fmtid, &raw const DpFmtidDriver) != false {
        let m: *const DpDriverMapEntry = dp_find_driver_map((*key).pid);

        if !m.is_null() {
            err = dp_get_driver(c, m, type_0, buf, bufSize, req);
        }
    }

    if err != ERROR_NOT_FOUND {
        return err;
    }
    return dp_get_registry(c, key, type_0, buf, bufSize, req);
}

unsafe extern "C" fn dpk_add(l: *mut DPKEYS, fmtid: *const GUID, pid: DWORD) {
    let mut i: DWORD = 0;
    if (*l).err != 0 {
        return;
    }
    i = 0;
    while i < (*l).n {
        if (*(*l).k.offset(i as isize)).pid == pid
            && IsEqualGUID!(
                core::ptr::addr_of_mut!((*(*l).k.offset(i as isize)).fmtid) as *const GUID,
                fmtid,
            ) != false
        {
            return;
        }
        i = i.wrapping_add(1);
    }
    if (*l).n == (*l).cap {
        let cap: DWORD = if (*l).cap != 0 {
            (*l).cap.wrapping_mul(2)
        } else {
            64
        };
        let p: *mut DEVPROPKEY = (if !(*l).k.is_null() {
            HeapReAlloc(
                GetProcessHeap(),
                0,
                (*l).k as PVOID,
                (cap as SIZE_T).wrapping_mul(core::mem::size_of::<DEVPROPKEY>() as SIZE_T),
            ) as *mut core::ffi::c_void
        } else {
            HeapAlloc(
                GetProcessHeap(),
                0,
                (cap as SIZE_T).wrapping_mul(core::mem::size_of::<DEVPROPKEY>() as SIZE_T),
            ) as *mut core::ffi::c_void
        }) as *mut DEVPROPKEY;
        if p.is_null() {
            (*l).err = ERROR_NOT_ENOUGH_MEMORY;
            return;
        }
        (*l).k = p;
        (*l).cap = cap;
    }
    (*(*l).k.offset((*l).n as isize)).fmtid = *fmtid;
    (*(*l).k.offset((*l).n as isize)).pid = pid;
    (*l).n = (*l).n.wrapping_add(1);
}

unsafe extern "C" fn dp_probe(c: *mut DPCTX, fmtid: *const GUID, pid: DWORD) -> BOOL {
    let mut key: DEVPROPKEY = core::mem::zeroed();
    let mut type_0: DEVPROPTYPE = 0;
    let mut tmp: [BYTE; 8] = [0; 8];
    let mut err: DWORD = 0;
    key.fmtid = *fmtid as DEVPROPGUID;
    key.pid = pid as DEVPROPID;
    err = dp_get(
        c,
        core::ptr::addr_of_mut!(key),
        core::ptr::addr_of_mut!(type_0),
        core::ptr::addr_of_mut!(tmp) as *mut BYTE,
        0 as DWORD,
        core::ptr::null_mut::<DWORD>(),
    );
    return (err == ERROR_SUCCESS || err == ERROR_INSUFFICIENT_BUFFER) as BOOL;
}

unsafe extern "C" fn dp_list_registry_keys(mut c: *mut DPCTX, mut l: *mut DPKEYS) {
    let mut dev: HKEY = core::ptr::null_mut::<HKEY__>();
    let mut props: HKEY = core::ptr::null_mut::<HKEY__>();
    let mut i: DWORD = 0;
    if dp_open_enum_key(c, KEY_READ, core::ptr::addr_of_mut!(dev)) != 0 {
        return;
    }
    if RegOpenKeyExW(
        dev,
        RUSTCONV_WSTR_007D169EA923.as_ptr(),
        0,
        KEY_ENUMERATE_SUB_KEYS,
        core::ptr::addr_of_mut!(props),
    ) != 0
    {
        RegCloseKey(dev);
        return;
    }
    RegCloseKey(dev);

    i = 0;
    while (*l).err == 0 {
        let mut name: [WCHAR; 39] = [0; 39];
        let mut len: DWORD = (core::mem::size_of_val(&name) / core::mem::size_of::<WCHAR>()) as DWORD;
        let mut j: DWORD = 0;
        let mut fmt: HKEY = core::ptr::null_mut::<HKEY__>();
        let mut guid: GUID = GUID {
            Data1: 0,
            Data2: 0,
            Data3: 0,
            Data4: [0; 8],
        };

        if RegEnumKeyExW(
            props,
            i,
            name.as_mut_ptr(),
            core::ptr::addr_of_mut!(len),
            core::ptr::null_mut::<DWORD>(),
            core::ptr::null_mut::<WCHAR>(),
            core::ptr::null_mut::<DWORD>(),
            core::ptr::null_mut::<FILETIME>(),
        ) != 0
        {
            break;
        }
        if dp_parse_guid(name.as_mut_ptr(), core::ptr::addr_of_mut!(guid)) != 0 {
            if RegOpenKeyExW(
                props,
                name.as_mut_ptr(),
                0,
                KEY_ENUMERATE_SUB_KEYS,
                core::ptr::addr_of_mut!(fmt),
            ) == 0
            {
                j = 0;
                while (*l).err == 0 {
                    let mut pidName: [WCHAR; 6] = [0; 6];
                    let mut plen: DWORD =
                        (core::mem::size_of_val(&pidName) / core::mem::size_of::<WCHAR>()) as DWORD;

                    if RegEnumKeyExW(
                        fmt,
                        j,
                        pidName.as_mut_ptr(),
                        core::ptr::addr_of_mut!(plen),
                        core::ptr::null_mut::<DWORD>(),
                        core::ptr::null_mut::<WCHAR>(),
                        core::ptr::null_mut::<DWORD>(),
                        core::ptr::null_mut::<FILETIME>(),
                    ) != 0
                    {
                        break;
                    }
                    dpk_add(
                        l,
                        core::ptr::addr_of_mut!(guid),
                        strtoulW(
                            pidName.as_mut_ptr(),
                            core::ptr::null_mut::<*mut WCHAR>(),
                            16,
                        ),
                    );
                    j = j.wrapping_add(1);
                }
                RegCloseKey(fmt);
            }
        }
        i = i.wrapping_add(1);
    }
    RegCloseKey(props);
}

unsafe extern "C" fn dp_list_keys(c: *mut DPCTX, l: *mut DPKEYS) -> DWORD {
    let mut i: ULONG = 0;
    let mut id: PCWSTR = core::ptr::null::<WCHAR>();
    let mut err: DWORD = 0;

    memset(
        l as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        core::mem::size_of::<DPKEYS>() as size_t,
    );

    /* A bad devnode must fail, not yield an empty list. */
    err = dp_get_instance_id(c, &raw mut id);
    if err != 0 {
        return err;
    }
    dpk_add(
        l,
        &raw const DpFmtidInstance,
        DP_PID_INSTANCEID as DWORD,
    );

    i = 0 as ULONG;
    while (i as usize) < ((*(&raw const DpDeviceMap)).len() as usize) {
        if dp_probe(
            c,
            &raw const DpFmtidDevice,
            DpDeviceMap[i as usize].pid as DWORD,
        ) != 0
        {
            dpk_add(
                l,
                &raw const DpFmtidDevice,
                DpDeviceMap[i as usize].pid as DWORD,
            );
        }
        i = i.wrapping_add(1);
    }

    i = 0 as ULONG;
    while (i as usize) < ((*(&raw const DpDriverMap)).len() as usize) {
        if dp_probe(
            c,
            &raw const DpFmtidDriver,
            DpDriverMap[i as usize].pid as DWORD,
        ) != 0
        {
            dpk_add(
                l,
                &raw const DpFmtidDriver,
                DpDriverMap[i as usize].pid as DWORD,
            );
        }
        i = i.wrapping_add(1);
    }

    dp_list_registry_keys(c, l);

    if (*l).err != 0 {
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            (*l).k as PVOID,
        );
        (*l).k = core::ptr::null_mut();
    }
    return (*l).err;
}

unsafe extern "C" fn dp_get_keys(c: *mut DPCTX, out: *mut DEVPROPKEY, outLen: DWORD, required: *mut DWORD) -> DWORD {
    let mut l: DPKEYS = DPKEYS {
        k: core::ptr::null_mut(),
        n: 0,
        cap: 0,
        err: 0,
    };
    let mut err: DWORD = dp_list_keys(c, core::ptr::addr_of_mut!(l));

    if err != 0 {
        return err;
    }
    if !required.is_null() {
        *required = l.n;
    }
    if l.n > outLen {
        err = ERROR_INSUFFICIENT_BUFFER as DWORD;
    } else if l.n != 0 {
        memcpy(
            out as *mut core::ffi::c_void,
            l.k as *const core::ffi::c_void,
            (l.n as usize).wrapping_mul(core::mem::size_of::<DEVPROPKEY>()),
        );
    }
    HeapFree(GetProcessHeap(), 0 as DWORD, l.k as PVOID);
    return err;
}

unsafe extern "C" fn dp_init_setupdi(c: *mut DPCTX, hDevInfo: HDEVINFO, data: PSP_DEVINFO_DATA) -> BOOL {
    let mut set: *mut DeviceInfoSet = hDevInfo as *mut DeviceInfoSet;
    let mut devInfo: *mut DeviceInfo = core::ptr::null_mut::<DeviceInfo>();
    if hDevInfo.is_null()
        || hDevInfo == INVALID_HANDLE_VALUE
        || (*set).magic != SETUP_DEVICE_INFO_SET_MAGIC as DWORD
    {
        SetLastError(ERROR_INVALID_HANDLE as DWORD);
        return FALSE as BOOL;
    }
    if data.is_null()
        || (*data).cbSize as usize != core::mem::size_of::<SP_DEVINFO_DATA>() as usize
        || (*data).Reserved == 0
    {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return FALSE as BOOL;
    }
    devInfo = (*data).Reserved as *mut DeviceInfo;
    if (*devInfo).set != set {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return FALSE as BOOL;
    }

    memset(
        c as *mut core::ffi::c_void,
        0 as core::ffi::c_int,
        core::mem::size_of::<DPCTX>() as usize,
    );
    (*c).setupdi = TRUE as BOOL;
    (*c).hDevInfo = hDevInfo;
    (*c).pData = data;
    (*c).devInfo = devInfo;
    (*c).set = set;
    (*c).devInst = (*devInfo).dnDevInst;
    (*c).hMachine = (*set).hMachine;
    (*c).instanceId = (*devInfo).instanceId;
    return TRUE as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiGetDevicePropertyW(
    mut DeviceInfoSet: HDEVINFO,
    mut DeviceInfoData: PSP_DEVINFO_DATA,
    mut PropertyKey: *const DEVPROPKEY,
    mut PropertyType: *mut DEVPROPTYPE,
    mut PropertyBuffer: PBYTE,
    mut PropertyBufferSize: DWORD,
    mut RequiredSize: PDWORD,
    mut Flags: DWORD,
) -> BOOL {
    let mut ctx: DPCTX = core::mem::zeroed::<DPCTX>();
    let mut err: DWORD = 0;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/setupapi/devprop.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"SetupDiGetDevicePropertyW\0" as *const u8 as *const ::core::ffi::c_char,
            800 as ::core::ffi::c_int,
            b"%p %p %p %p %p %lu %p %lx\n\0" as *const u8 as *const ::core::ffi::c_char,
            DeviceInfoSet,
            DeviceInfoData,
            PropertyKey,
            PropertyType,
            PropertyBuffer,
            PropertyBufferSize,
            RequiredSize,
            Flags,
        );
    }
    if dp_init_setupdi(&raw mut ctx, DeviceInfoSet, DeviceInfoData) == 0 {
        return (FALSE) as BOOL;
    }
    if PropertyKey.is_null() {
        SetLastError(ERROR_INVALID_DATA);
        return (FALSE) as BOOL;
    }
    if PropertyType.is_null() || PropertyBuffer.is_null() && PropertyBufferSize != 0 {
        SetLastError(ERROR_INVALID_USER_BUFFER);
        return (FALSE) as BOOL;
    }
    if Flags != 0 {
        SetLastError(ERROR_INVALID_FLAGS);
        return (FALSE) as BOOL;
    }

    err = dp_get(
        &raw mut ctx,
        PropertyKey,
        PropertyType,
        PropertyBuffer as *mut BYTE,
        PropertyBufferSize,
        RequiredSize as *mut DWORD,
    );
    SetLastError(err);
    return (err == ERROR_SUCCESS) as ::core::ffi::c_int;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiGetDevicePropertyKeys(
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
    PropertyKeyArray: *mut DEVPROPKEY,
    PropertyKeyCount: DWORD,
    RequiredPropertyKeyCount: PDWORD,
    Flags: DWORD,
) -> BOOL {
    let mut ctx: DPCTX = core::mem::zeroed();
    let mut err: DWORD;

    if Flags != 0 {
        SetLastError(ERROR_INVALID_FLAGS as DWORD);
        return (FALSE) as BOOL;
    }
    if PropertyKeyArray.is_null() && PropertyKeyCount != 0 {
        SetLastError(ERROR_INVALID_USER_BUFFER as DWORD);
        return (FALSE) as BOOL;
    }
    if dp_init_setupdi(core::ptr::addr_of_mut!(ctx), DeviceInfoSet, DeviceInfoData) == 0 {
        return (FALSE) as BOOL;
    }

    err = dp_get_keys(
        core::ptr::addr_of_mut!(ctx),
        PropertyKeyArray,
        PropertyKeyCount,
        RequiredPropertyKeyCount,
    );
    SetLastError(err);
    return (err == ERROR_SUCCESS as DWORD) as BOOL;
}

unsafe extern "C" fn dp_is_valid_type(t: DEVPROPTYPE) -> BOOL {
    let type_0: DWORD = t as DWORD & DEVPROP_MASK_TYPE as DWORD;
    let mod_0: DWORD = t as DWORD & DEVPROP_MASK_TYPEMOD as DWORD;
    if type_0 > MAX_DEVPROP_TYPE as DWORD || mod_0 > MAX_DEVPROP_TYPEMOD as DWORD {
        return (FALSE) as BOOL;
    }
    if mod_0 == DEVPROP_TYPEMOD_ARRAY as DWORD
        && (type_0 == DEVPROP_TYPE_EMPTY as DWORD
            || type_0 == DEVPROP_TYPE_NULL as DWORD
            || type_0 == DEVPROP_TYPE_STRING as DWORD
            || type_0 == DEVPROP_TYPE_SECURITY_DESCRIPTOR_STRING as DWORD)
    {
        return (FALSE) as BOOL;
    }
    if mod_0 == DEVPROP_TYPEMOD_LIST as DWORD
        && !(type_0 == DEVPROP_TYPE_STRING as DWORD || type_0 == DEVPROP_TYPE_SECURITY_DESCRIPTOR_STRING as DWORD)
    {
        return (FALSE) as BOOL;
    }
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiSetDevicePropertyW(
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
    PropertyKey: *const DEVPROPKEY,
    PropertyType: DEVPROPTYPE,
    PropertyBuffer: *const BYTE,
    PropertyBufferSize: DWORD,
    Flags: DWORD,
) -> BOOL {
    let mut ctx: DPCTX = core::mem::zeroed();
    let mut dev: HKEY = core::ptr::null_mut();
    let mut props: HKEY = core::ptr::null_mut();
    let mut prop: HKEY = core::ptr::null_mut();
    let mut path: [WCHAR; 38 + 5 + 1] =
        [0; 38 + 5 + 1];
    let mut err: DWORD = 0;
    let mut empty: BOOL = 0;

    if dp_init_setupdi(core::ptr::addr_of_mut!(ctx), DeviceInfoSet, DeviceInfoData) == 0 {
        return (FALSE) as BOOL;
    }
    if PropertyBufferSize != 0 && PropertyBuffer.is_null() {
        SetLastError(ERROR_INVALID_USER_BUFFER);
        return (FALSE) as BOOL;
    }
    empty = (PropertyType == DEVPROP_TYPE_EMPTY || PropertyType == DEVPROP_TYPE_NULL) as BOOL;
    if PropertyKey.is_null()
        || dp_is_valid_type(PropertyType) == 0
        || (!(!PropertyBuffer.is_null() && PropertyBufferSize != 0) && empty == 0)
        || (!PropertyBuffer.is_null() && PropertyBufferSize != 0 && empty != 0)
    {
        SetLastError(ERROR_INVALID_DATA);
        return (FALSE) as BOOL;
    }
    if Flags != 0 {
        SetLastError(ERROR_INVALID_FLAGS);
        return (FALSE) as BOOL;
    }

    err = dp_open_enum_key(
        core::ptr::addr_of_mut!(ctx),
        KEY_READ | KEY_WRITE,
        core::ptr::addr_of_mut!(dev),
    ) as DWORD;
    if err == ERROR_SUCCESS {
        err = RegCreateKeyExW(
            dev,
            RUSTCONV_WSTR_007D169EA923.as_ptr(),
            0,
            core::ptr::null_mut(),
            0,
            KEY_READ | KEY_WRITE,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(props),
            core::ptr::null_mut(),
        ) as DWORD;
        RegCloseKey(dev);
    }
    if err != 0 {
        SetLastError(err);
        return (FALSE) as BOOL;
    }

    dp_guid_to_string(core::ptr::addr_of!((*PropertyKey).fmtid), path.as_mut_ptr());
    sprintfW(
        path.as_mut_ptr().offset(38 as isize),
        RUSTCONV_WSTR_CFA00EC2E978.as_ptr(),
        (*PropertyKey).pid,
    );

    match PropertyType {
        DEVPROP_TYPE_EMPTY => {
            err = RegDeleteKeyW(props, path.as_ptr()) as DWORD;
        }
        DEVPROP_TYPE_NULL => {
            err = RegOpenKeyExW(
                props,
                path.as_ptr(),
                0,
                KEY_SET_VALUE,
                core::ptr::addr_of_mut!(prop),
            ) as DWORD;
            if err == ERROR_SUCCESS {
                err = RegDeleteValueW(prop, core::ptr::null()) as DWORD;
                RegCloseKey(prop);
            }
        }
        _ => {
            err = RegCreateKeyExW(
                props,
                path.as_ptr(),
                0,
                core::ptr::null_mut(),
                0,
                KEY_READ | KEY_WRITE,
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(prop),
                core::ptr::null_mut(),
            ) as DWORD;
            if err == ERROR_SUCCESS {
                err = RegSetValueExW(
                    prop,
                    core::ptr::null(),
                    0,
                    0xffff0000 | (0xffff & PropertyType),
                    PropertyBuffer,
                    PropertyBufferSize,
                ) as DWORD;
                RegCloseKey(prop);
            }
        }
    }
    RegCloseKey(props);

    if err == ERROR_FILE_NOT_FOUND {
        err = ERROR_NOT_FOUND;
    }
    SetLastError(err);
    return (err == ERROR_SUCCESS) as BOOL;
}

unsafe extern "C" fn dp_init_cm(c: *mut DPCTX, devInst: DEVINST, hMachine: HMACHINE) {
    memset(c as *mut core::ffi::c_void, 0, core::mem::size_of::<DPCTX>());
    (*c).devInst = devInst;
    (*c).hMachine = hMachine;
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_DevNode_Property_ExW(
    dnDevInst: DEVINST,
    PropertyKey: *const DEVPROPKEY,
    PropertyType: *mut DEVPROPTYPE,
    PropertyBuffer: PBYTE,
    PropertyBufferSize: PULONG,
    ulFlags: ULONG,
    hMachine: HMACHINE,
) -> CONFIGRET {
    let mut ctx: DPCTX = core::mem::zeroed();
    let mut err: DWORD;
    let mut req: DWORD = 0;

    if PropertyKey.is_null() || PropertyType.is_null() || PropertyBufferSize.is_null() {
        return CR_INVALID_POINTER;
    }
    if PropertyBuffer.is_null() && *PropertyBufferSize != 0 {
        return CR_INVALID_POINTER;
    }
    if ulFlags != 0 {
        return CR_INVALID_FLAG;
    }
    if dnDevInst == 0 {
        return CR_INVALID_DEVNODE;
    }

    dp_init_cm(core::ptr::addr_of_mut!(ctx), dnDevInst, hMachine);
    err = dp_get(
        core::ptr::addr_of_mut!(ctx),
        PropertyKey,
        PropertyType,
        PropertyBuffer,
        *PropertyBufferSize,
        core::ptr::addr_of_mut!(req),
    );
    if err == ERROR_SUCCESS || err == ERROR_INSUFFICIENT_BUFFER {
        *PropertyBufferSize = req;
    }
    return dp_error_to_cr(err);
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_DevNode_PropertyW(
    dnDevInst: DEVINST,
    PropertyKey: *const DEVPROPKEY,
    PropertyType: *mut DEVPROPTYPE,
    PropertyBuffer: PBYTE,
    PropertyBufferSize: PULONG,
    ulFlags: ULONG,
) -> CONFIGRET {
    return CM_Get_DevNode_Property_ExW(
        dnDevInst,
        PropertyKey,
        PropertyType,
        PropertyBuffer,
        PropertyBufferSize,
        ulFlags,
        core::ptr::null_mut(),
    );
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_DevNode_Property_Keys_Ex(
    mut dnDevInst: DEVINST,
    mut PropertyKeyArray: *mut DEVPROPKEY,
    mut PropertyKeyCount: PULONG,
    mut ulFlags: ULONG,
    mut hMachine: HMACHINE,
) -> CONFIGRET {
    let mut ctx: DPCTX = core::mem::zeroed();
    let mut err: DWORD;
    let mut required: DWORD = 0;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/setupapi/devprop.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"CM_Get_DevNode_Property_Keys_Ex\0" as *const u8 as *const ::core::ffi::c_char,
            1045 as ::core::ffi::c_int,
            b"%lx %p %p %lx %p\n\0" as *const u8 as *const ::core::ffi::c_char,
            dnDevInst,
            PropertyKeyArray,
            PropertyKeyCount,
            ulFlags,
            hMachine,
        );
    }
    if PropertyKeyCount.is_null() {
        return CR_INVALID_POINTER;
    }
    if PropertyKeyArray.is_null() && *PropertyKeyCount != 0 {
        return CR_INVALID_POINTER;
    }
    if ulFlags != 0 {
        return CR_INVALID_FLAG;
    }
    if dnDevInst == 0 {
        return CR_INVALID_DEVNODE;
    }
    dp_init_cm(&raw mut ctx, dnDevInst, hMachine);
    err = dp_get_keys(
        &raw mut ctx,
        PropertyKeyArray,
        *PropertyKeyCount,
        &raw mut required,
    );
    if err == ERROR_SUCCESS || err == ERROR_INSUFFICIENT_BUFFER {
        *PropertyKeyCount = required;
    }
    return dp_error_to_cr(err);
}

#[no_mangle]
pub unsafe extern "system" fn CM_Get_DevNode_Property_Keys(
    dnDevInst: DEVINST,
    PropertyKeyArray: *mut DEVPROPKEY,
    PropertyKeyCount: PULONG,
    ulFlags: ULONG,
) -> CONFIGRET {
    return CM_Get_DevNode_Property_Keys_Ex(
        dnDevInst,
        PropertyKeyArray,
        PropertyKeyCount,
        ulFlags,
        core::ptr::null_mut(),
    );
}
