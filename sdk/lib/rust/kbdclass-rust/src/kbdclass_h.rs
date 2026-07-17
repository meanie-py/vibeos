//! Hand-written translation of the device-extension types from
//! `drivers/input/kbdclass/kbdclass.h`.
//!
//! rustconv only translates the `.c` translation unit; the struct/enum
//! definitions that live in the driver's private header are not emitted by
//! the per-function pipeline. They are reproduced here verbatim from
//! `kbdclass.h` (same approach arm64input-rust took for its in-module
//! `VIRTIO_*` structs). Layout must match the C header exactly — these are
//! `IoCreateDevice` device-extension blobs cast from raw pointers.

#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{PDEVICE_OBJECT, PIRP, PKEYBOARD_INPUT_DATA, HANDLE, LPCWSTR, _UNICODE_STRING};
use ntkrnl_sys::driver_ddk::{
    ctl_code, FILE_DEVICE_KEYBOARD, FILE_ANY_ACCESS, METHOD_BUFFERED, METHOD_NEITHER,
};

// #define CLASS_TAG 'CdbK'  (kbdclass.h) — pool tag. C multi-char char
// constant is big-endian, so the bytes are 'C','d','b','K'.
pub const CLASS_TAG: u32 = u32::from_be_bytes(*b"CdbK");

// Keyboard IOCTLs (ntddkbd.h / kbdmou.h) — CTL_CODE values via driver_ddk's
// const-fn composer. Function codes + methods are header-authoritative.
pub const IOCTL_KEYBOARD_QUERY_ATTRIBUTES: u32 =
    ctl_code(FILE_DEVICE_KEYBOARD, 0x0000, METHOD_BUFFERED, FILE_ANY_ACCESS);
pub const IOCTL_KEYBOARD_QUERY_INDICATOR_TRANSLATION: u32 =
    ctl_code(FILE_DEVICE_KEYBOARD, 0x0002, METHOD_BUFFERED, FILE_ANY_ACCESS);
pub const IOCTL_KEYBOARD_QUERY_TYPEMATIC: u32 =
    ctl_code(FILE_DEVICE_KEYBOARD, 0x0008, METHOD_BUFFERED, FILE_ANY_ACCESS);
pub const IOCTL_KEYBOARD_QUERY_INDICATORS: u32 =
    ctl_code(FILE_DEVICE_KEYBOARD, 0x0010, METHOD_BUFFERED, FILE_ANY_ACCESS);
pub const IOCTL_KEYBOARD_SET_INDICATORS: u32 =
    ctl_code(FILE_DEVICE_KEYBOARD, 0x0020, METHOD_BUFFERED, FILE_ANY_ACCESS);
pub const IOCTL_KEYBOARD_SET_TYPEMATIC: u32 =
    ctl_code(FILE_DEVICE_KEYBOARD, 0x0040, METHOD_BUFFERED, FILE_ANY_ACCESS);
pub const IOCTL_INTERNAL_KEYBOARD_CONNECT: u32 =
    ctl_code(FILE_DEVICE_KEYBOARD, 0x0080, METHOD_NEITHER, FILE_ANY_ACCESS);
pub const IOCTL_INTERNAL_KEYBOARD_DISCONNECT: u32 =
    ctl_code(FILE_DEVICE_KEYBOARD, 0x0100, METHOD_NEITHER, FILE_ANY_ACCESS);

// GUID_DEVINTERFACE_KEYBOARD {884b96c3-56ef-11d1-bc8c-00a0c91405dd}
// (ntddkbd.h). Registered by ClassAddDevice via IoRegisterDeviceInterface.
//
// NB: the bindgen `_GUID` is mis-emitted with `Data4: [u8; 4]` (12 bytes
// instead of the correct 16). We therefore define a correct 16-byte GUID
// here and the call site casts `&GUID_DEVINTERFACE_KEYBOARD as *const _`
// to the API's pointer type — the kernel reads the full 16 bytes.
#[repr(C)]
pub struct DEVINTERFACE_GUID {
    pub Data1: u32,
    pub Data2: u16,
    pub Data3: u16,
    pub Data4: [u8; 8],
}
pub static GUID_DEVINTERFACE_KEYBOARD: DEVINTERFACE_GUID = DEVINTERFACE_GUID {
    Data1: 0x884b96c3,
    Data2: 0x56ef,
    Data3: 0x11d1,
    Data4: [0xbc, 0x8c, 0x00, 0xa0, 0xc9, 0x14, 0x05, 0xdd],
};

// typedef enum { dsStopped, dsStarted, dsPaused, dsRemoved, dsSurpriseRemoved }
//     PORT_DEVICE_STATE;
// Represented as a plain u32 (C enum default ABI) + module-level constants so
// the existing `match`/assignment sites resolve against named values.
pub type PORT_DEVICE_STATE = u32;
pub const dsStopped: PORT_DEVICE_STATE = 0;
pub const dsStarted: PORT_DEVICE_STATE = 1;
pub const dsPaused: PORT_DEVICE_STATE = 2;
pub const dsRemoved: PORT_DEVICE_STATE = 3;
pub const dsSurpriseRemoved: PORT_DEVICE_STATE = 4;

// typedef struct _CLASS_DRIVER_EXTENSION {
//     UNICODE_STRING RegistryPath;
//     ULONG ConnectMultiplePorts;
//     ULONG DataQueueSize;
//     UNICODE_STRING DeviceBaseName;
//     PDEVICE_OBJECT MainClassDeviceObject;
// } CLASS_DRIVER_EXTENSION, *PCLASS_DRIVER_EXTENSION;
#[repr(C)]
pub struct CLASS_DRIVER_EXTENSION {
    pub RegistryPath: _UNICODE_STRING,
    pub ConnectMultiplePorts: ULONG,
    pub DataQueueSize: ULONG,
    pub DeviceBaseName: _UNICODE_STRING,
    pub MainClassDeviceObject: PDEVICE_OBJECT,
}
pub type PCLASS_DRIVER_EXTENSION = *mut CLASS_DRIVER_EXTENSION;

// typedef struct _COMMON_DEVICE_EXTENSION {
//     BOOLEAN IsClassDO;
// } COMMON_DEVICE_EXTENSION, *PCOMMON_DEVICE_EXTENSION;
#[repr(C)]
pub struct COMMON_DEVICE_EXTENSION {
    pub IsClassDO: BOOLEAN,
}
pub type PCOMMON_DEVICE_EXTENSION = *mut COMMON_DEVICE_EXTENSION;

// typedef struct _PORT_DEVICE_EXTENSION {
//     COMMON_DEVICE_EXTENSION Common;
//     LIST_ENTRY ListEntry;
//     PDEVICE_OBJECT DeviceObject;
//     PORT_DEVICE_STATE PnpState;
//     PDEVICE_OBJECT LowerDevice;
//     PDEVICE_OBJECT ClassDO;
//     HANDLE FileHandle;
//     UNICODE_STRING InterfaceName;
// } PORT_DEVICE_EXTENSION, *PPORT_DEVICE_EXTENSION;
#[repr(C)]
pub struct PORT_DEVICE_EXTENSION {
    pub Common: COMMON_DEVICE_EXTENSION,
    pub ListEntry: LIST_ENTRY,
    pub DeviceObject: PDEVICE_OBJECT,
    pub PnpState: PORT_DEVICE_STATE,
    pub LowerDevice: PDEVICE_OBJECT,
    pub ClassDO: PDEVICE_OBJECT,
    pub FileHandle: HANDLE,
    pub InterfaceName: _UNICODE_STRING,
}
pub type PPORT_DEVICE_EXTENSION = *mut PORT_DEVICE_EXTENSION;

// typedef struct _CLASS_DEVICE_EXTENSION {
//     COMMON_DEVICE_EXTENSION Common;
//     PCLASS_DRIVER_EXTENSION DriverExtension;
//     LIST_ENTRY ListHead;
//     KSPIN_LOCK ListSpinLock;
//     KSPIN_LOCK SpinLock;
//     PIRP PendingIrp;
//     SIZE_T InputCount;
//     PKEYBOARD_INPUT_DATA PortData;
//     LPCWSTR DeviceName;
// } CLASS_DEVICE_EXTENSION, *PCLASS_DEVICE_EXTENSION;
#[repr(C)]
pub struct CLASS_DEVICE_EXTENSION {
    pub Common: COMMON_DEVICE_EXTENSION,
    pub DriverExtension: PCLASS_DRIVER_EXTENSION,
    pub ListHead: LIST_ENTRY,
    pub ListSpinLock: usize,
    pub SpinLock: usize,
    pub PendingIrp: PIRP,
    pub InputCount: SIZE_T,
    pub PortData: PKEYBOARD_INPUT_DATA,
    pub DeviceName: LPCWSTR,
}
pub type PCLASS_DEVICE_EXTENSION = *mut CLASS_DEVICE_EXTENSION;
