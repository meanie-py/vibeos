//! Hand-written translation of the device-extension types from
//! `drivers/input/mouclass/mouclass.h`.
//!
//! rustconv only translates the `.c` translation unit; the struct/enum
//! definitions that live in the driver's private header are not emitted by
//! the per-function pipeline. They are reproduced here verbatim from
//! `mouclass.h` (same approach kbdclass-rust / arm64input-rust took).
//! Layout must match the C header exactly — these are `IoCreateDevice`
//! device-extension blobs cast from raw pointers.

#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{
    PDEVICE_OBJECT, PIRP, HANDLE, LPCWSTR, _UNICODE_STRING,
};
use ntkrnl_sys::driver_ddk::{
    ctl_code, FILE_DEVICE_MOUSE, FILE_ANY_ACCESS, METHOD_BUFFERED, METHOD_NEITHER,
};

// #define CLASS_TAG 'CuoM'  (mouclass.h) — pool tag. C multi-char char
// constant is big-endian, so the bytes are 'C','u','o','M'.
pub const CLASS_TAG: u32 = u32::from_be_bytes(*b"CuoM");

// Mouse IOCTLs (ntddmou.h / kbdmou.h) — CTL_CODE values via driver_ddk's
// const-fn composer. Function codes + methods are header-authoritative.
pub const IOCTL_MOUSE_QUERY_ATTRIBUTES: u32 =
    ctl_code(FILE_DEVICE_MOUSE, 0x0000, METHOD_BUFFERED, FILE_ANY_ACCESS);
pub const IOCTL_INTERNAL_MOUSE_CONNECT: u32 =
    ctl_code(FILE_DEVICE_MOUSE, 0x0080, METHOD_NEITHER, FILE_ANY_ACCESS);
pub const IOCTL_INTERNAL_MOUSE_DISCONNECT: u32 =
    ctl_code(FILE_DEVICE_MOUSE, 0x0100, METHOD_NEITHER, FILE_ANY_ACCESS);

// GUID_DEVINTERFACE_MOUSE {378de44c-56ef-11d1-bc8c-00a0c91405dd} (ntddmou.h).
// Registered by ClassAddDevice via IoRegisterDeviceInterface.
//
// NB: the bindgen `_GUID` is mis-emitted with `Data4: [u8; 4]` (12 bytes
// instead of the correct 16). We therefore define a correct 16-byte GUID
// here and the call site casts `addr_of!(GUID_DEVINTERFACE_MOUSE)` to the
// API's pointer type — the kernel reads the full 16 bytes.
#[repr(C)]
pub struct DEVINTERFACE_GUID {
    pub Data1: u32,
    pub Data2: u16,
    pub Data3: u16,
    pub Data4: [u8; 8],
}
pub static GUID_DEVINTERFACE_MOUSE: DEVINTERFACE_GUID = DEVINTERFACE_GUID {
    Data1: 0x378de44c,
    Data2: 0x56ef,
    Data3: 0x11d1,
    Data4: [0xbc, 0x8c, 0x00, 0xa0, 0xc9, 0x14, 0x05, 0xdd],
};

// ntddmou.h driver-facing type (not in ntkrnl-sys bindgen / ntoskrnl.h).
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct MOUSE_INPUT_DATA {
    pub UnitId: USHORT,
    pub Flags: USHORT,
    // union { ULONG Buttons; struct { USHORT ButtonFlags; USHORT ButtonData; } };
    pub ButtonFlags: USHORT,
    pub ButtonData: USHORT,
    pub RawButtons: ULONG,
    pub LastX: LONG,
    pub LastY: LONG,
    pub ExtraInformation: ULONG,
}
pub type PMOUSE_INPUT_DATA = *mut MOUSE_INPUT_DATA;

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct MOUSE_ATTRIBUTES {
    pub MouseIdentifier: USHORT,
    pub NumberOfButtons: USHORT,
    pub SampleRate: USHORT,
    pub InputDataQueueLength: ULONG,
}
pub type PMOUSE_ATTRIBUTES = *mut MOUSE_ATTRIBUTES;

// typedef enum { dsStopped, dsStarted, dsPaused, dsRemoved, dsSurpriseRemoved }
//     PORT_DEVICE_STATE;
pub type PORT_DEVICE_STATE = u32;
pub const dsStopped: PORT_DEVICE_STATE = 0;
pub const dsStarted: PORT_DEVICE_STATE = 1;
pub const dsPaused: PORT_DEVICE_STATE = 2;
pub const dsRemoved: PORT_DEVICE_STATE = 3;
pub const dsSurpriseRemoved: PORT_DEVICE_STATE = 4;

// typedef struct _CLASS_DRIVER_EXTENSION { ... } CLASS_DRIVER_EXTENSION;
#[repr(C)]
pub struct CLASS_DRIVER_EXTENSION {
    pub RegistryPath: _UNICODE_STRING,
    pub ConnectMultiplePorts: ULONG,
    pub DataQueueSize: ULONG,
    pub DeviceBaseName: _UNICODE_STRING,
    pub MainClassDeviceObject: PDEVICE_OBJECT,
}
pub type PCLASS_DRIVER_EXTENSION = *mut CLASS_DRIVER_EXTENSION;

// typedef struct _COMMON_DEVICE_EXTENSION { BOOLEAN IsClassDO; } ...;
#[repr(C)]
pub struct COMMON_DEVICE_EXTENSION {
    pub IsClassDO: BOOLEAN,
}
pub type PCOMMON_DEVICE_EXTENSION = *mut COMMON_DEVICE_EXTENSION;

// typedef struct _PORT_DEVICE_EXTENSION { ... } PORT_DEVICE_EXTENSION;
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

// typedef struct _CLASS_DEVICE_EXTENSION { ... } CLASS_DEVICE_EXTENSION;
#[repr(C)]
pub struct CLASS_DEVICE_EXTENSION {
    pub Common: COMMON_DEVICE_EXTENSION,
    pub DriverExtension: PCLASS_DRIVER_EXTENSION,
    pub ListHead: LIST_ENTRY,
    pub ListSpinLock: usize,
    pub SpinLock: usize,
    pub PendingIrp: PIRP,
    pub InputCount: usize,
    pub PortData: PMOUSE_INPUT_DATA,
    pub DeviceName: LPCWSTR,
}
pub type PCLASS_DEVICE_EXTENSION = *mut CLASS_DEVICE_EXTENSION;

// Stitcher postprocess underscore-prefixes type names; provide aliases so
// translated code that refers to the bindgen-style `_NAME` resolves here.
pub use self::PORT_DEVICE_EXTENSION as _PORT_DEVICE_EXTENSION;
pub use self::CLASS_DEVICE_EXTENSION as _CLASS_DEVICE_EXTENSION;
pub use self::COMMON_DEVICE_EXTENSION as _COMMON_DEVICE_EXTENSION;
pub use self::CLASS_DRIVER_EXTENSION as _CLASS_DRIVER_EXTENSION;
