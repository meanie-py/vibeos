use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::ddk::{CM_PARTIAL_RESOURCE_DESCRIPTOR, CM_RESOURCE_LIST, DEVICE_CAPABILITIES, DEVICE_RELATIONS, _KWAIT_REASON_Executive, IO_RESOURCE_DESCRIPTOR, IO_RESOURCE_REQUIREMENTS_LIST, _IO_STATUS_BLOCK, IoBuildSynchronousFsdRequest, IoGetAttachedDeviceReference, IoRegisterDeviceInterface, IoSetDeviceInterfaceState, KAFFINITY, KEVENT, KeInitializeEvent, KeWaitForSingleObject, _EVENT_TYPE_NotificationEvent, PCM_PARTIAL_RESOURCE_DESCRIPTOR, PCM_RESOURCE_LIST, PDEVICE_CAPABILITIES, PDEVICE_OBJECT, PDEVICE_RELATIONS, PIO_RESOURCE_DESCRIPTOR, PIO_RESOURCE_REQUIREMENTS_LIST, PIO_STACK_LOCATION, PIRP, PNP_BUS_INFORMATION, PPNP_BUS_INFORMATION};
// IoGet{Current,Next}IrpStackLocation are FORCEINLINE in the DDK (no exported
// symbol) — use the ntkrnl-sys inline impls or the cohab link fails.
use ntkrnl_sys::inline::tier1::{IoGetCurrentIrpStackLocation, IoGetNextIrpStackLocation};

// ---- Enum-variant constants: bindgen prefixes them (`_<ENUM>_<Name>`), so the
// unprefixed spellings the C glue uses don't resolve from ddk. Re-alias here
// (values are bindgen's, verified against the ddk enums; this file only). ----
use ntkrnl_sys::ddk::{
    _INTERFACE_TYPE_Internal as Internal,
    _INTERFACE_TYPE_InternalPowerBus as InternalPowerBus,
    _DEVICE_POWER_STATE_PowerDeviceD0 as PowerDeviceD0,
    _DEVICE_POWER_STATE_PowerDeviceD1 as PowerDeviceD1,
    _DEVICE_POWER_STATE_PowerDeviceD2 as PowerDeviceD2,
    _DEVICE_POWER_STATE_PowerDeviceD3 as PowerDeviceD3,
    _SYSTEM_POWER_STATE_PowerSystemWorking as PowerSystemWorking,
    _SYSTEM_POWER_STATE_PowerSystemSleeping1 as PowerSystemSleeping1,
    _SYSTEM_POWER_STATE_PowerSystemSleeping2 as PowerSystemSleeping2,
    _SYSTEM_POWER_STATE_PowerSystemSleeping3 as PowerSystemSleeping3,
    _CM_SHARE_DISPOSITION_CmResourceShareDeviceExclusive as CmResourceShareDeviceExclusive,
    _CM_SHARE_DISPOSITION_CmResourceShareDriverExclusive as CmResourceShareDriverExclusive,
    _CM_SHARE_DISPOSITION_CmResourceShareShared as CmResourceShareShared,
    _BUS_QUERY_ID_TYPE_BusQueryDeviceID as BusQueryDeviceID,
    _BUS_QUERY_ID_TYPE_BusQueryInstanceID as BusQueryInstanceID,
    _BUS_QUERY_ID_TYPE_BusQueryHardwareIDs as BusQueryHardwareIDs,
    _BUS_QUERY_ID_TYPE_BusQueryCompatibleIDs as BusQueryCompatibleIDs,
};

// ---- ACPICA API (acpica-sys; imported by name — NEVER glob, which collides
// with the ddk glob on shared primitives like BOOLEAN/UINT8). ----
use acpica_sys::{AcpiGetCurrentResources, AcpiGetPossibleResources};

// ---- Cross-module glue: real translations live in bus.rs (acpi.sys). ----
use crate::bus::{acpi_bus_get_device, acpi_bus_power_manageable, acpi_bus_set_power};

// ---- PoSetPowerState: kernel export, NTAPI (stdcall on i386, cdecl on
// amd64/arm64 — `extern "system"` resolves per arch). Returns POWER_STATE by
// value (the C glue ignores it). Signature: pofuncs.h. ----
extern "system" {
    fn PoSetPowerState(
        DeviceObject: PDEVICE_OBJECT,
        Type: ULONG,
        State: POWER_STATE,
    ) -> POWER_STATE;
}

// NT_ASSERT maps to the no-op ASSERT helper (kept from the substrate).
macro_rules! NT_ASSERT {
    ($($t:tt)*) => {};
}

// ---- CRT string helpers (real libc, declared — never the empty-body stubs,
// which silently break ID matching). swprintf is C-variadic. ----
extern "C" {
    fn strstr(s1: *const i8, s2: *const i8) -> *mut i8;
    fn strcmp(s1: *const i8, s2: *const i8) -> i32;
    fn wcsstr(s1: *const u16, s2: *const u16) -> *mut u16;
    fn wcslen(s: *const u16) -> usize;
    fn wcscpy(dest: *mut u16, src: *const u16) -> *mut u16;
    fn swprintf(buffer: *mut u16, format: *const u16, ...) -> i32;
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const ACPI_BUTTON_HID_LID: [::core::ffi::c_char; 8] =
    unsafe { ::core::mem::transmute::<[u8; 8], [::core::ffi::c_char; 8]>(*b"PNP0C0D\0") };

pub const ACPI_FAN_HID: [::core::ffi::c_char; 8] =
    unsafe { ::core::mem::transmute::<[u8; 8], [::core::ffi::c_char; 8]>(*b"PNP0C0B\0") };

pub const ACPI_PROCESSOR_HID: [::core::ffi::c_char; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_char; 10]>(*b"Processor\0") };

pub const ACPI_THERMAL_HID: [::core::ffi::c_char; 12] =
    unsafe { ::core::mem::transmute::<[u8; 12], [::core::ffi::c_char; 12]>(*b"ThermalZone\0") };

pub static mut GUID_DEVICE_BATTERY: GUID = GUID {
    Data1: 0x72631e54 as ::core::ffi::c_long as ::core::ffi::c_uint,
    Data2: 0x78a4 as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0xbc as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xaa as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2a as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut BATTERY_STATUS_WMI_GUID: GUID = GUID {
    Data1: 0xfc4670d1 as ::core::ffi::c_uint,
    Data2: 0xebbf as ::core::ffi::c_ushort,
    Data3: 0x416e as ::core::ffi::c_ushort,
    Data4: [
        0x87 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xce as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x37 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xbc as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x11 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x1a as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut BATTERY_RUNTIME_WMI_GUID: GUID = GUID {
    Data1: 0x535a3767 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x1ac2 as ::core::ffi::c_ushort,
    Data3: 0x49bc as ::core::ffi::c_ushort,
    Data4: [
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x7a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xec as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut BATTERY_TEMPERATURE_WMI_GUID: GUID = GUID {
    Data1: 0x1a52a14d as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xadce as ::core::ffi::c_ushort,
    Data3: 0x4a44 as ::core::ffi::c_ushort,
    Data4: [
        0x9a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut BATTERY_FULL_CHARGED_CAPACITY_WMI_GUID: GUID = GUID {
    Data1: 0x40b40565 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x96f7 as ::core::ffi::c_ushort,
    Data3: 0x4435 as ::core::ffi::c_ushort,
    Data4: [
        0x86 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x94 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x97 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x39 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x59 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut BATTERY_CYCLE_COUNT_WMI_GUID: GUID = GUID {
    Data1: 0xef98db24 as ::core::ffi::c_uint,
    Data2: 0x14 as ::core::ffi::c_ushort,
    Data3: 0x4c25 as ::core::ffi::c_ushort,
    Data4: [
        0xa5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x24 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xae as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x71 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut BATTERY_STATIC_DATA_WMI_GUID: GUID = GUID {
    Data1: 0x5e1e463 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xe4e2 as ::core::ffi::c_ushort,
    Data3: 0x4ea9 as ::core::ffi::c_ushort,
    Data4: [
        0x80 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xcb as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xca as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x55 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut BATTERY_STATUS_CHANGE_WMI_GUID: GUID = GUID {
    Data1: 0xcddfa0c3 as ::core::ffi::c_uint,
    Data2: 0x7c5b as ::core::ffi::c_ushort,
    Data3: 0x4e43 as ::core::ffi::c_ushort,
    Data4: [
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x34 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x43 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x64 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut BATTERY_TAG_CHANGE_WMI_GUID: GUID = GUID {
    Data1: 0x5e1f6e19 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x8786 as ::core::ffi::c_ushort,
    Data3: 0x4d23 as ::core::ffi::c_ushort,
    Data4: [
        0x94 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xfc as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x74 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x88 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_CLASS_INPUT: GUID = GUID {
    Data1: 0x4d1e55b2 as ::core::ffi::c_long as ::core::ffi::c_uint,
    Data2: 0xf16f as ::core::ffi::c_ushort,
    Data3: 0x11cf as ::core::ffi::c_ushort,
    Data4: [
        0x88 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xcb as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x11 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x11 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x30 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_DEVICE_LID: GUID = GUID {
    Data1: 0x4afa3d52 as ::core::ffi::c_long as ::core::ffi::c_uint,
    Data2: 0x74a7 as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0xbe as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x28 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x57 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_DEVICE_MEMORY: GUID = GUID {
    Data1: 0x3fd0f03d as ::core::ffi::c_long as ::core::ffi::c_uint,
    Data2: 0x92e0 as ::core::ffi::c_ushort,
    Data3: 0x45fb as ::core::ffi::c_ushort,
    Data4: [
        0xb7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xff as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x10 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x21 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_DEVICE_MESSAGE_INDICATOR: GUID = GUID {
    Data1: 0xcd48a365 as ::core::ffi::c_ulong as ::core::ffi::c_uint,
    Data2: 0xfa94 as ::core::ffi::c_ushort,
    Data3: 0x4ce2 as ::core::ffi::c_ushort,
    Data4: [
        0xa2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x32 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x64 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_DEVICE_PROCESSOR: GUID = GUID {
    Data1: 0x97fadb10 as ::core::ffi::c_ulong as ::core::ffi::c_uint,
    Data2: 0x4e33 as ::core::ffi::c_ushort,
    Data3: 0x40ae as ::core::ffi::c_ushort,
    Data4: [
        0x35 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xef as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xbd as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_DEVICE_SYS_BUTTON: GUID = GUID {
    Data1: 0x4afa3d53 as ::core::ffi::c_long as ::core::ffi::c_uint,
    Data2: 0x74a7 as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0xbe as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x28 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x57 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_DEVICE_THERMAL_ZONE: GUID = GUID {
    Data1: 0x4afa3d51 as ::core::ffi::c_long as ::core::ffi::c_uint,
    Data2: 0x74a7 as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0xbe as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x28 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x57 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_DEVICE_FAN: GUID = GUID {
    Data1: 0x5ecd13d as ::core::ffi::c_long as ::core::ffi::c_uint,
    Data2: 0x81da as ::core::ffi::c_ushort,
    Data3: 0x4a2a as ::core::ffi::c_ushort,
    Data4: [
        0x8a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x52 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x23 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xdd as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

// Device-extension types come from the hand-verified shared substrate (exact
// acpisys.h layouts). The LLM type-preamble that c2rust emitted here dropped
// fields and used the reserved keyword `Self`; shared.rs replaces it.
// DEVICE_PNP_STATE + NotStarted..UnKnown come from shared::*; the Power*
// states (PowerDeviceD0.., PowerSystemWorking..) come from the ddk imports.
use crate::shared::*;

pub const ACPI_D_STATE_COUNT: ULONG = 4;
pub const ACPI_STATE_D0: ULONG = 0;
pub const ACPI_STATE_D1: ULONG = 1;
pub const ACPI_STATE_D2: ULONG = 2;
pub const ACPI_STATE_D3: ULONG = 3;

// Values match acpica/include/acrestyp.h (LEVEL_SENSITIVE/POS_DECODE/
// READ_ONLY_MEMORY are the 0-valued members of their pairs; TYPE_A/B/F are
// 1/2/3 after ACPI_COMPATIBILITY=0; TRANSFER_8_16 is 1, TRANSFER_16 is 2).
pub const ACPI_PRODUCER: u8 = 0;
pub const ACPI_CONSUMER: ULONG = 1;
pub const ACPI_SHARED: u8 = 1;
pub const ACPI_LEVEL_SENSITIVE: u8 = 0;
pub const ACPI_EDGE_SENSITIVE: u8 = 1;
pub const ACPI_BUS_MASTER: u8 = 1;
pub const ACPI_DECODE_16: u8 = 1;
pub const ACPI_POS_DECODE: u8 = 0;
pub const ACPI_READ_ONLY_MEMORY: u8 = 0;
pub const ACPI_CACHABLE_MEMORY: ULONG = 1;
pub const ACPI_WRITE_COMBINING_MEMORY: ULONG = 2;
pub const ACPI_PREFETCHABLE_MEMORY: ULONG = 3;
pub const ACPI_TYPE_A: ULONG = 1;
pub const ACPI_TYPE_B: ULONG = 2;
pub const ACPI_TYPE_F: ULONG = 3;
pub const ACPI_TRANSFER_8: ULONG = 0;
pub const ACPI_TRANSFER_8_16: ULONG = 1;
pub const ACPI_TRANSFER_16: ULONG = 2;

pub const ACPI_RESOURCE_TYPE_IRQ: ULONG = 0;
pub const ACPI_RESOURCE_TYPE_DMA: ULONG = 1;
pub const ACPI_RESOURCE_TYPE_START_DEPENDENT: ULONG = 2;
pub const ACPI_RESOURCE_TYPE_END_DEPENDENT: ULONG = 3;
pub const ACPI_RESOURCE_TYPE_IO: ULONG = 4;
pub const ACPI_RESOURCE_TYPE_FIXED_IO: ULONG = 5;
pub const ACPI_RESOURCE_TYPE_VENDOR: ULONG = 6;
pub const ACPI_RESOURCE_TYPE_END_TAG: ULONG = 7;
pub const ACPI_RESOURCE_TYPE_MEMORY24: ULONG = 8;
pub const ACPI_RESOURCE_TYPE_MEMORY32: ULONG = 9;
pub const ACPI_RESOURCE_TYPE_FIXED_MEMORY32: ULONG = 10;
pub const ACPI_RESOURCE_TYPE_ADDRESS16: ULONG = 11;
pub const ACPI_RESOURCE_TYPE_ADDRESS32: ULONG = 12;
pub const ACPI_RESOURCE_TYPE_ADDRESS64: ULONG = 13;
pub const ACPI_RESOURCE_TYPE_EXTENDED_ADDRESS64: ULONG = 14;
pub const ACPI_RESOURCE_TYPE_EXTENDED_IRQ: ULONG = 15;
pub const ACPI_RESOURCE_TYPE_GENERIC_REGISTER: ULONG = 16;

// acrestyp.h: MEMORY_RANGE=0, IO_RANGE=1, BUS_NUMBER_RANGE=2 (were swapped).
pub const ACPI_MEMORY_RANGE: u8 = 0;
pub const ACPI_IO_RANGE: u8 = 1;
pub const ACPI_BUS_NUMBER_RANGE: u8 = 2;

// FIXED 2026-07-06: the whole table was a rustconv-invented 0..11 sequence
// (Interrupt=0, Port=2, ...). Real values per sdk/include/xdk/cmtypes.h:224.
// Consequence of the drift: every ACPI-built CM_PARTIAL_RESOURCE_DESCRIPTOR
// was mistyped — IRQ descriptors were CmResourceTypeNull(0), so pnpres
// translation and i8042prt's resource parsing never saw an interrupt →
// IoConnectInterrupt never called → PS/2 keyboard+mouse silently dead on
// i386/amd64 since Rust buspdo.rs integrated (polled detection kept working,
// which masked it). Found by the Doom playable-gate (tests/qemu/test_doom.py);
// independently root-caused in the start-menu lane (field manual §9.37).
pub const CmResourceTypeNull: ULONG = 0;
pub const CmResourceTypePort: ULONG = 1;
pub const CmResourceTypeInterrupt: ULONG = 2;
pub const CmResourceTypeMemory: ULONG = 3;
pub const CmResourceTypeDma: ULONG = 4;
pub const CmResourceTypeDeviceSpecific: ULONG = 5;
pub const CmResourceTypeBusNumber: ULONG = 6;
pub const CmResourceTypeMemoryLarge: ULONG = 7;
pub const CmResourceTypeNonArbitrated: ULONG = 128;
pub const CmResourceTypeConfigData: ULONG = 128;
pub const CmResourceTypeDevicePrivate: ULONG = 129;
pub const CmResourceTypePcCardConfig: ULONG = 130;
pub const CmResourceTypeMfCardConfig: ULONG = 131;

pub const CM_RESOURCE_INTERRUPT_LEVEL_SENSITIVE: ULONG = 0x00;
pub const CM_RESOURCE_INTERRUPT_LATCHED: ULONG = 0x01;
pub const CM_RESOURCE_DMA_TYPE_A: ULONG = 0x10;
pub const CM_RESOURCE_DMA_TYPE_B: ULONG = 0x20;
pub const CM_RESOURCE_DMA_TYPE_F: ULONG = 0x40;
pub const CM_RESOURCE_DMA_BUS_MASTER: ULONG = 8;
pub const CM_RESOURCE_DMA_8: ULONG = 0;
pub const CM_RESOURCE_DMA_16: ULONG = 0x01;
pub const CM_RESOURCE_DMA_8_AND_16: ULONG = 0x04;
pub const CM_RESOURCE_PORT_IO: ULONG = 0x01;
pub const CM_RESOURCE_PORT_16_BIT_DECODE: ULONG = 0x10;
pub const CM_RESOURCE_PORT_10_BIT_DECODE: ULONG = 0x04;
pub const CM_RESOURCE_PORT_POSITIVE_DECODE: ULONG = 0x20;
pub const CM_RESOURCE_MEMORY_READ_ONLY: ULONG = 1;
pub const CM_RESOURCE_MEMORY_READ_WRITE: ULONG = 0x00;
pub const CM_RESOURCE_MEMORY_CACHEABLE: ULONG = 0x20;
pub const CM_RESOURCE_MEMORY_COMBINEDWRITE: ULONG = 8;
pub const CM_RESOURCE_MEMORY_PREFETCHABLE: ULONG = 0x04;
pub const CM_RESOURCE_MEMORY_24: ULONG = 0x10;

pub const GUID_ACPI_INTERFACE_STANDARD: GUID = GUID {
    Data1: 0xE8C9E6C1,
    Data2: 0x4B9C,
    Data3: 0x4B2B,
    Data4: [0x9E, 0x8B, 0x1A, 0x6E, 0x0B, 0x3E, 0x0C, 0xE0],
};

pub const UNICODE_NULL: WCHAR = 0;

// IO_RESOURCE_* option flags + IRP_MN_QUERY_CAPABILITIES — values from
// sdk/include/xdk/iotypes.h (not emitted by the kernel bindgen).
pub const IO_RESOURCE_PREFERRED: ULONG = 0x01;
pub const IO_RESOURCE_DEFAULT: ULONG = 0x02;
pub const IO_RESOURCE_ALTERNATIVE: ULONG = 0x08;
pub const IRP_MN_QUERY_CAPABILITIES: u8 = 0x09;
// The remaining IRP_MN_QUERY_* minor codes the driver_ddk subset omits — without
// these, the `match` arms would silently become catch-all bindings (footgun).
pub const IRP_MN_QUERY_INTERFACE: u8 = 0x08;
pub const IRP_MN_QUERY_RESOURCES: u8 = 0x0A;
pub const IRP_MN_QUERY_RESOURCE_REQUIREMENTS: u8 = 0x0B;
pub const IRP_MN_QUERY_DEVICE_TEXT: u8 = 0x0C;
pub const IRP_MN_QUERY_ID: u8 = 0x13;
pub const IRP_MN_QUERY_BUS_INFORMATION: u8 = 0x15;

// ACPI_HANDLE/ACPI_STATUS, the ACPI_RESOURCE* structs, ACPI_BUFFER, and the
// acpi_device* structs come from the bindgen substrate via `use crate::shared::*`
// (acpica-sys layouts + acpi_bus.h). AE_* codes also come from shared (the local
// AE_BUFFER_OVERFLOW here was 0x3001 — WRONG; correct is 0x000B in shared).
pub fn ACPI_SUCCESS(status: ACPI_STATUS) -> bool {
    status == AE_OK
}

pub fn ACPI_NEXT_RESOURCE(res: *const ACPI_RESOURCE) -> *const ACPI_RESOURCE {
    unsafe {
        let ptr = res as *const u8;
        let offset = (*res).Length as usize;
        (ptr.add(offset)) as *const ACPI_RESOURCE
    }
}


// Faithful to the acpisys.h macros: they mutate through the struct lvalue.
// SET saves the current state into PreviousPnPState, then sets the new one.
pub unsafe fn SET_NEW_PNP_STATE(data: *mut COMMON_DEVICE_DATA, state: DEVICE_PNP_STATE) {
    (*data).PreviousPnPState = (*data).DevicePnPState;
    (*data).DevicePnPState = state;
}

pub unsafe fn RESTORE_PREVIOUS_PNP_STATE(data: *mut COMMON_DEVICE_DATA) {
    (*data).DevicePnPState = (*data).PreviousPnPState;
}

pub fn DbgDeviceRelationString(_type_: ULONG) -> *const u8 {
    b"Unknown\0".as_ptr()
}

pub fn ASSERTMSG(_msg: *const u8, _expr: bool) {
}

pub fn ASSERT(_expr: bool) {
}


pub type PWCHAR = *mut u16;
pub type WCHAR = u16;
pub type ULONG = u32;
pub type ULONGLONG = u64;
pub type BOOLEAN = u8;
pub type PCHAR = *mut u8;
// ULONG_PTR is the kernel's pointer-width unsigned (c_ulong i386 / c_ulonglong
// 64-bit) — must match the IoStatus.Information field type, not `usize`.
pub use ntkrnl_sys::ddk::ULONG_PTR;
pub type PVOID = *mut core::ffi::c_void;
// DEVICE_POWER_STATE is `_DEVICE_POWER_STATE` (c_uint) in the kernel; KAFFINITY
// comes from the ddk import above (don't redefine).
pub type DEVICE_POWER_STATE = core::ffi::c_uint;

// GUID is the kernel _GUID (used by IoRegisterDeviceInterface / PNP_BUS_INFORMATION).
pub use ntkrnl_sys::ddk::GUID;

// POWER_STATE is a C union of {SystemState, DeviceState}; only DeviceState is
// used here. Layout-compatible (single c_uint) with the kernel POWER_STATE for
// the PoSetPowerState call.
#[repr(C)]
pub struct POWER_STATE {
    pub DeviceState: DEVICE_POWER_STATE,
}

// _POWER_STATE_TYPE_DevicePowerState == 1 (the c2rust preamble had 0 — WRONG).
pub const DevicePowerState: ULONG = 1;

// ProcessorIdString / ProcessorNameString / ProcessorHardwareIds are mutable C
// globals defined+populated in main.c (which stays C). The c2rust preamble
// fabricated local consts (and a broken byte-string→*const u16 cast). Declare
// the real externs instead; types from acpisys.h (LPWSTR / UNICODE_STRING).
extern "C" {
    static mut ProcessorIdString: PWCHAR;
    static mut ProcessorNameString: PWCHAR;
    static mut ProcessorHardwareIds: _UNICODE_STRING;
}

use ntkrnl_sys::base::UNICODE_STRING as _UNICODE_STRING;
#[no_mangle]
pub unsafe extern "C" fn Bus_PDO_PnP(
    _DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    IrpStack: PIO_STACK_LOCATION,
    DeviceData: PPDO_DEVICE_DATA,
) -> NTSTATUS {
    let mut status: NTSTATUS;
    let mut state: POWER_STATE = core::mem::zeroed();
    let mut device: *mut acpi_device = core::ptr::null_mut();

    if !(*DeviceData).AcpiHandle.is_null() {
        acpi_bus_get_device((*DeviceData).AcpiHandle, &mut device);
    }

    match (*IrpStack).MinorFunction as u8 {
        IRP_MN_START_DEVICE => {
            if !(*DeviceData).AcpiHandle.is_null()
                && acpi_bus_power_manageable((*DeviceData).AcpiHandle) != 0
                && !ACPI_SUCCESS(acpi_bus_set_power((*DeviceData).AcpiHandle, 0) as ACPI_STATUS)
            {
                status = STATUS_UNSUCCESSFUL;
            } else {
                (*DeviceData).InterfaceName.Length = 0;
                status = STATUS_SUCCESS;

                if device.is_null() {
                    status = IoRegisterDeviceInterface(
                        (*DeviceData).Common.Self_,
                        &raw const GUID_DEVICE_SYS_BUTTON,
                        core::ptr::null_mut(),
                        &mut (*DeviceData).InterfaceName,
                    );
                } else if (*device).flags.hardware_id() != 0
                    && !strstr(
                        (*device).pnp.hardware_id as *const i8,
                        ACPI_THERMAL_HID.as_ptr(),
                    )
                    .is_null()
                {
                    status = IoRegisterDeviceInterface(
                        (*DeviceData).Common.Self_,
                        &raw const GUID_DEVICE_THERMAL_ZONE,
                        core::ptr::null_mut(),
                        &mut (*DeviceData).InterfaceName,
                    );
                } else if (*device).flags.hardware_id() != 0
                    && !strstr(
                        (*device).pnp.hardware_id as *const i8,
                        ACPI_FAN_HID.as_ptr(),
                    )
                    .is_null()
                {
                    status = IoRegisterDeviceInterface(
                        (*DeviceData).Common.Self_,
                        &raw const GUID_DEVICE_FAN,
                        core::ptr::null_mut(),
                        &mut (*DeviceData).InterfaceName,
                    );
                } else if (*device).flags.hardware_id() != 0
                    && !strstr(
                        (*device).pnp.hardware_id as *const i8,
                        ACPI_BUTTON_HID_LID.as_ptr(),
                    )
                    .is_null()
                {
                    status = IoRegisterDeviceInterface(
                        (*DeviceData).Common.Self_,
                        &raw const GUID_DEVICE_LID,
                        core::ptr::null_mut(),
                        &mut (*DeviceData).InterfaceName,
                    );
                } else if (*device).flags.hardware_id() != 0
                    && !strstr(
                        (*device).pnp.hardware_id as *const i8,
                        ACPI_PROCESSOR_HID.as_ptr(),
                    )
                    .is_null()
                {
                    status = IoRegisterDeviceInterface(
                        (*DeviceData).Common.Self_,
                        &raw const GUID_DEVICE_PROCESSOR,
                        core::ptr::null_mut(),
                        &mut (*DeviceData).InterfaceName,
                    );
                }

                if NT_SUCCESS(status) && (*DeviceData).InterfaceName.Length != 0 {
                    IoSetDeviceInterfaceState(&mut (*DeviceData).InterfaceName, 1u8);
                }

                state.DeviceState = PowerDeviceD0;
                PoSetPowerState((*DeviceData).Common.Self_, DevicePowerState, state);
                (*DeviceData).Common.DevicePowerState = PowerDeviceD0;
                SET_NEW_PNP_STATE(&mut (*DeviceData).Common, Started);
                status = STATUS_SUCCESS;
            }
        }

        IRP_MN_STOP_DEVICE => {
            if (*DeviceData).InterfaceName.Length != 0 {
                IoSetDeviceInterfaceState(&mut (*DeviceData).InterfaceName, 0u8);
            }

            if !(*DeviceData).AcpiHandle.is_null()
                && acpi_bus_power_manageable((*DeviceData).AcpiHandle) != 0
                && !ACPI_SUCCESS(acpi_bus_set_power((*DeviceData).AcpiHandle, 3) as ACPI_STATUS)
            {
                status = STATUS_UNSUCCESSFUL;
            } else {
                state.DeviceState = PowerDeviceD3;
                PoSetPowerState((*DeviceData).Common.Self_, DevicePowerState, state);
                (*DeviceData).Common.DevicePowerState = PowerDeviceD3;
                SET_NEW_PNP_STATE(&mut (*DeviceData).Common, Stopped);
                status = STATUS_SUCCESS;
            }
        }

        IRP_MN_QUERY_STOP_DEVICE => {
            SET_NEW_PNP_STATE(&mut (*DeviceData).Common, StopPending);
            status = STATUS_SUCCESS;
        }

        IRP_MN_CANCEL_STOP_DEVICE => {
            if StopPending == (*DeviceData).Common.DevicePnPState {
                RESTORE_PREVIOUS_PNP_STATE(&mut (*DeviceData).Common);
            }
            status = STATUS_SUCCESS;
        }

        IRP_MN_REMOVE_DEVICE => {
            if (*DeviceData).InterfaceName.Length != 0 {
                IoSetDeviceInterfaceState(&mut (*DeviceData).InterfaceName, 0u8);
            }

            if !(*DeviceData).AcpiHandle.is_null()
                && acpi_bus_power_manageable((*DeviceData).AcpiHandle) != 0
                && !ACPI_SUCCESS(acpi_bus_set_power((*DeviceData).AcpiHandle, 3) as ACPI_STATUS)
            {
                state.DeviceState = PowerDeviceD3;
                PoSetPowerState((*DeviceData).Common.Self_, DevicePowerState, state);
                (*DeviceData).Common.DevicePowerState = PowerDeviceD3;
            }

            SET_NEW_PNP_STATE(&mut (*DeviceData).Common, Stopped);
            status = STATUS_SUCCESS;
        }

        IRP_MN_QUERY_REMOVE_DEVICE => {
            SET_NEW_PNP_STATE(&mut (*DeviceData).Common, RemovalPending);
            status = STATUS_SUCCESS;
        }

        IRP_MN_CANCEL_REMOVE_DEVICE => {
            if RemovalPending == (*DeviceData).Common.DevicePnPState {
                RESTORE_PREVIOUS_PNP_STATE(&mut (*DeviceData).Common);
            }
            status = STATUS_SUCCESS;
        }

        IRP_MN_QUERY_CAPABILITIES => {
            status = Bus_PDO_QueryDeviceCaps(DeviceData, Irp);
        }

        IRP_MN_QUERY_ID => {
            status = Bus_PDO_QueryDeviceId(DeviceData, Irp);
        }

        IRP_MN_QUERY_DEVICE_RELATIONS => {
            status = Bus_PDO_QueryDeviceRelations(DeviceData, Irp);
        }

        IRP_MN_QUERY_DEVICE_TEXT => {
            status = Bus_PDO_QueryDeviceText(DeviceData, Irp);
        }

        IRP_MN_QUERY_RESOURCES => {
            status = Bus_PDO_QueryResources(DeviceData, Irp);
        }

        IRP_MN_QUERY_RESOURCE_REQUIREMENTS => {
            status = Bus_PDO_QueryResourceRequirements(DeviceData, Irp);
        }

        IRP_MN_QUERY_BUS_INFORMATION => {
            status = Bus_PDO_QueryBusInformation(DeviceData, Irp);
        }

        IRP_MN_QUERY_INTERFACE => {
            status = Bus_PDO_QueryInterface(DeviceData, Irp);
        }

        _ => {
            status = (*Irp).IoStatus.__bindgen_anon_1.Status;
        }
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = status;
    IoCompleteRequest(Irp, IO_NO_INCREMENT);
    status
}
#[no_mangle]
pub unsafe extern "C" fn Bus_PDO_QueryDeviceCaps(
    DeviceData: PPDO_DEVICE_DATA,
    Irp: PIRP,
) -> NTSTATUS {
    let mut device: *mut acpi_device = core::ptr::null_mut();
    let mut i: ULONG;

    if !(*DeviceData).AcpiHandle.is_null() {
        acpi_bus_get_device((*DeviceData).AcpiHandle, &mut device);
    }

    let stack = IoGetCurrentIrpStackLocation(Irp);
    let deviceCapabilities = (*stack).Parameters.DeviceCapabilities.Capabilities;

    if (*deviceCapabilities).Version != 1 || ((*deviceCapabilities).Size as ULONG) < core::mem::size_of::<DEVICE_CAPABILITIES>() as ULONG {
        return STATUS_UNSUCCESSFUL;
    }

    (*deviceCapabilities).D1Latency = 0;
    (*deviceCapabilities).D2Latency = 0;
    (*deviceCapabilities).D3Latency = 0;

    (*deviceCapabilities).DeviceState[PowerSystemWorking as usize] = PowerDeviceD0;
    (*deviceCapabilities).DeviceState[PowerSystemSleeping1 as usize] = PowerDeviceD3;
    (*deviceCapabilities).DeviceState[PowerSystemSleeping2 as usize] = PowerDeviceD3;
    (*deviceCapabilities).DeviceState[PowerSystemSleeping3 as usize] = PowerDeviceD3;

    i = 0;
    while i < ACPI_D_STATE_COUNT && !device.is_null() {
        if (*device).power.states[i as usize].flags.valid() != 0 {
            match i {
                0 => {
                    (*deviceCapabilities).DeviceState[PowerSystemWorking as usize] = PowerDeviceD0;
                }
                1 => {
                    (*deviceCapabilities).DeviceState[PowerSystemSleeping1 as usize] = PowerDeviceD1;
                    (*deviceCapabilities).D1Latency = (*device).power.states[i as usize].latency as ULONG;
                }
                2 => {
                    (*deviceCapabilities).DeviceState[PowerSystemSleeping2 as usize] = PowerDeviceD2;
                    (*deviceCapabilities).D2Latency = (*device).power.states[i as usize].latency as ULONG;
                }
                3 => {
                    (*deviceCapabilities).DeviceState[PowerSystemSleeping3 as usize] = PowerDeviceD3;
                    (*deviceCapabilities).D3Latency = (*device).power.states[i as usize].latency as ULONG;
                }
                _ => {}
            }
        }
        i = i.wrapping_add(1);
    }

    (*deviceCapabilities).DeviceWake = PowerDeviceD1;

    (*deviceCapabilities).set_DeviceD1(if (*deviceCapabilities).DeviceState[PowerSystemSleeping1 as usize] == PowerDeviceD1 { 1 } else { 0 });
    (*deviceCapabilities).set_DeviceD2(if (*deviceCapabilities).DeviceState[PowerSystemSleeping2 as usize] == PowerDeviceD2 { 1 } else { 0 });

    (*deviceCapabilities).set_WakeFromD0(0);
    (*deviceCapabilities).set_WakeFromD1(1);
    (*deviceCapabilities).set_WakeFromD2(0);
    (*deviceCapabilities).set_WakeFromD3(0);

    if !device.is_null() {
        (*deviceCapabilities).set_LockSupported((*device).flags.lockable() as ULONG);
        (*deviceCapabilities).set_EjectSupported((*device).flags.ejectable() as ULONG);
        (*deviceCapabilities).set_HardwareDisabled(if (*device).status.enabled() == 0 && (*device).status.functional() == 0 { 1 } else { 0 });
        (*deviceCapabilities).set_Removable((*device).flags.removable() as ULONG);
        (*deviceCapabilities).set_SurpriseRemovalOK((*device).flags.surprise_removal_ok() as ULONG);
        (*deviceCapabilities).set_UniqueID((*device).flags.unique_id() as ULONG);
        (*deviceCapabilities).set_NoDisplayInUI(if (*device).status.show_in_ui() == 0 { 1 } else { 0 });
        (*deviceCapabilities).Address = (*device).pnp.bus_address as ULONG;
    }

    if (*DeviceData).DockDevice != 0 {
        (*deviceCapabilities).set_DockDevice(1);
    }

    if device.is_null() ||
        ((*device).flags.hardware_id() != 0 &&
         (!strstr((*device).pnp.hardware_id as *const i8, ACPI_BUTTON_HID_LID.as_ptr()).is_null() ||
          !strstr((*device).pnp.hardware_id as *const i8, ACPI_THERMAL_HID.as_ptr()).is_null() ||
          !strstr((*device).pnp.hardware_id as *const i8, ACPI_PROCESSOR_HID.as_ptr()).is_null()))
    {
        (*deviceCapabilities).set_RawDeviceOK(1);
    }

    (*deviceCapabilities).set_SilentInstall(0);
    (*deviceCapabilities).UINumber = !0u32;

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn Bus_PDO_QueryDeviceId(
    DeviceData: PPDO_DEVICE_DATA,
    Irp: PIRP,
) -> NTSTATUS {
    let stack: PIO_STACK_LOCATION;
    let buffer: PWCHAR;
    let src: PWCHAR;
    let mut temp: [WCHAR; 256] = [0; 256];
    let mut length: ULONG;
    let mut i: ULONG;
    let mut status: NTSTATUS = STATUS_SUCCESS;
    let mut Device: *mut acpi_device = core::ptr::null_mut();

    stack = IoGetCurrentIrpStackLocation(Irp);

    // The C `switch` uses `break` to bail to the trailing `return status`; model
    // that with a labeled block (`break 'query` == the C switch-break).
    'query: {
    match (*stack).Parameters.QueryId.IdType as u32 {
        BusQueryDeviceID => {
            if !(*DeviceData).AcpiHandle.is_null() {
                acpi_bus_get_device((*DeviceData).AcpiHandle, &mut Device);
                if strcmp((*Device).pnp.hardware_id as *const i8, b"Processor\0".as_ptr() as *const i8) == 0 {
                    length = wcslen(ProcessorIdString) as ULONG;
                    wcscpy(temp.as_mut_ptr(), ProcessorIdString);
                } else {
                    length = swprintf(temp.as_mut_ptr(), ntkrnl_sys::w!("ACPI\\%hs").as_ptr(), (*Device).pnp.hardware_id) as ULONG;
                }
            } else {
                length = swprintf(temp.as_mut_ptr(), ntkrnl_sys::w!("ACPI\\FixedButton").as_ptr()) as ULONG;
            }

            let idx = length as usize;
            length = length.wrapping_add(1);
            temp[idx] = UNICODE_NULL;

            NT_ASSERT!((length as usize).wrapping_mul(core::mem::size_of::<WCHAR>()) <= core::mem::size_of::<[WCHAR; 256]>());

            buffer = ExAllocatePoolWithTag(PagedPool, (length as usize).wrapping_mul(core::mem::size_of::<WCHAR>()), u32::from_be_bytes(*b"IpcA") as ULONG) as PWCHAR;
            if buffer.is_null() {
                status = STATUS_INSUFFICIENT_RESOURCES;
                break 'query;
            }

            core::ptr::copy_nonoverlapping(temp.as_ptr(), buffer, length as usize);
            (*Irp).IoStatus.Information = buffer as ULONG_PTR;
        }

        BusQueryInstanceID => {
            if !(*DeviceData).AcpiHandle.is_null() {
                acpi_bus_get_device((*DeviceData).AcpiHandle, &mut Device);
                if (*Device).flags.unique_id() != 0 {
                    // unique_id is a fixed [c_char; 9] array. In C it decays to char*
                    // when passed to the variadic %hs; Rust does NOT decay — passing it
                    // by value misaligns the varargs so %hs reads a stack slot as the
                    // string pointer → strnlen walks off the stack. Pass .as_ptr().
                    length = swprintf(temp.as_mut_ptr(), ntkrnl_sys::w!("%hs").as_ptr(), (*Device).pnp.unique_id.as_ptr()) as ULONG;
                } else {
                    length = swprintf(temp.as_mut_ptr(), ntkrnl_sys::w!("%ls").as_ptr(), ntkrnl_sys::w!("0").as_ptr()) as ULONG;
                }
            } else {
                length = swprintf(temp.as_mut_ptr(), ntkrnl_sys::w!("%ls").as_ptr(), ntkrnl_sys::w!("0").as_ptr()) as ULONG;
            }

            let idx = length as usize;
            length = length.wrapping_add(1);
            temp[idx] = UNICODE_NULL;

            NT_ASSERT!((length as usize).wrapping_mul(core::mem::size_of::<WCHAR>()) <= core::mem::size_of::<[WCHAR; 256]>());

            buffer = ExAllocatePoolWithTag(PagedPool, (length as usize).wrapping_mul(core::mem::size_of::<WCHAR>()), u32::from_be_bytes(*b"IpcA") as ULONG) as PWCHAR;
            if buffer.is_null() {
                status = STATUS_INSUFFICIENT_RESOURCES;
                break 'query;
            }

            core::ptr::copy_nonoverlapping(temp.as_ptr(), buffer, length as usize);
            (*Irp).IoStatus.Information = buffer as ULONG_PTR;
        }

        BusQueryHardwareIDs => {
            length = 0;
            status = STATUS_NOT_SUPPORTED;

            if !(*DeviceData).AcpiHandle.is_null() {
                acpi_bus_get_device((*DeviceData).AcpiHandle, &mut Device);
                if (*Device).flags.hardware_id() == 0 {
                    break 'query;
                }

                if strcmp((*Device).pnp.hardware_id as *const i8, b"Processor\0".as_ptr() as *const i8) == 0 {
                    length = (ProcessorHardwareIds.Length as usize).wrapping_div(core::mem::size_of::<WCHAR>()) as ULONG;
                    src = ProcessorHardwareIds.Buffer as PWCHAR;
                } else {
                    length = length.wrapping_add(swprintf(temp.as_mut_ptr().offset(length as isize), ntkrnl_sys::w!("ACPI\\%hs").as_ptr(), (*Device).pnp.hardware_id) as ULONG);
                    let idx = length as usize;
                    length = length.wrapping_add(1);
                    temp[idx] = UNICODE_NULL;

                    length = length.wrapping_add(swprintf(temp.as_mut_ptr().offset(length as isize), ntkrnl_sys::w!("*%hs").as_ptr(), (*Device).pnp.hardware_id) as ULONG);
                    let idx = length as usize;
                    length = length.wrapping_add(1);
                    temp[idx] = UNICODE_NULL;

                    let idx = length as usize;
                    length = length.wrapping_add(1);
                    temp[idx] = UNICODE_NULL;
                    src = temp.as_mut_ptr();
                }
            } else {
                length = length.wrapping_add(swprintf(temp.as_mut_ptr().offset(length as isize), ntkrnl_sys::w!("ACPI\\FixedButton").as_ptr()) as ULONG);
                let idx = length as usize;
                length = length.wrapping_add(1);
                temp[idx] = UNICODE_NULL;

                length = length.wrapping_add(swprintf(temp.as_mut_ptr().offset(length as isize), ntkrnl_sys::w!("*FixedButton").as_ptr()) as ULONG);
                let idx = length as usize;
                length = length.wrapping_add(1);
                temp[idx] = UNICODE_NULL;

                let idx = length as usize;
                length = length.wrapping_add(1);
                temp[idx] = UNICODE_NULL;
                src = temp.as_mut_ptr();
            }

            NT_ASSERT!((length as usize).wrapping_mul(core::mem::size_of::<WCHAR>()) <= core::mem::size_of::<[WCHAR; 256]>());

            buffer = ExAllocatePoolWithTag(PagedPool, (length as usize).wrapping_mul(core::mem::size_of::<WCHAR>()), u32::from_be_bytes(*b"IpcA") as ULONG) as PWCHAR;
            if buffer.is_null() {
                status = STATUS_INSUFFICIENT_RESOURCES;
                break 'query;
            }

            core::ptr::copy_nonoverlapping(src, buffer, length as usize);
            (*Irp).IoStatus.Information = buffer as ULONG_PTR;
            status = STATUS_SUCCESS;
        }

        BusQueryCompatibleIDs => {
            length = 0;
            status = STATUS_NOT_SUPPORTED;

            if !(*DeviceData).AcpiHandle.is_null() {
                acpi_bus_get_device((*DeviceData).AcpiHandle, &mut Device);
                if (*Device).flags.hardware_id() == 0 {
                    break 'query;
                }

                if strcmp((*Device).pnp.hardware_id as *const i8, b"Processor\0".as_ptr() as *const i8) == 0 {
                    length = length.wrapping_add(swprintf(temp.as_mut_ptr().offset(length as isize), ntkrnl_sys::w!("ACPI\\%hs").as_ptr(), (*Device).pnp.hardware_id) as ULONG);
                    let idx = length as usize;
                    length = length.wrapping_add(1);
                    temp[idx] = UNICODE_NULL;

                    length = length.wrapping_add(swprintf(temp.as_mut_ptr().offset(length as isize), ntkrnl_sys::w!("*%hs").as_ptr(), (*Device).pnp.hardware_id) as ULONG);
                    let idx = length as usize;
                    length = length.wrapping_add(1);
                    temp[idx] = UNICODE_NULL;

                    let idx = length as usize;
                    length = length.wrapping_add(1);
                    temp[idx] = UNICODE_NULL;
                } else if (*Device).flags.compatible_ids() != 0 {
                    i = 0;
                    while i < (*(*Device).pnp.cid_list).Count {
                        length = length.wrapping_add(swprintf(temp.as_mut_ptr().offset(length as isize), ntkrnl_sys::w!("ACPI\\%hs").as_ptr(), (*(*(*Device).pnp.cid_list).Ids.as_ptr().add(i as usize)).String) as ULONG);
                        let idx = length as usize;
                        length = length.wrapping_add(1);
                        temp[idx] = UNICODE_NULL;

                        length = length.wrapping_add(swprintf(temp.as_mut_ptr().offset(length as isize), ntkrnl_sys::w!("*%hs").as_ptr(), (*(*(*Device).pnp.cid_list).Ids.as_ptr().add(i as usize)).String) as ULONG);
                        let idx = length as usize;
                        length = length.wrapping_add(1);
                        temp[idx] = UNICODE_NULL;

                        i = i.wrapping_add(1);
                    }

                    let idx = length as usize;
                    length = length.wrapping_add(1);
                    temp[idx] = UNICODE_NULL;
                } else {
                    break 'query;
                }

                NT_ASSERT!((length as usize).wrapping_mul(core::mem::size_of::<WCHAR>()) <= core::mem::size_of::<[WCHAR; 256]>());

                buffer = ExAllocatePoolWithTag(PagedPool, (length as usize).wrapping_mul(core::mem::size_of::<WCHAR>()), u32::from_be_bytes(*b"IpcA") as ULONG) as PWCHAR;
                if buffer.is_null() {
                    status = STATUS_INSUFFICIENT_RESOURCES;
                    break 'query;
                }

                core::ptr::copy_nonoverlapping(temp.as_ptr(), buffer, length as usize);
                (*Irp).IoStatus.Information = buffer as ULONG_PTR;
                status = STATUS_SUCCESS;
            }
        }

        _ => {
            status = (*Irp).IoStatus.__bindgen_anon_1.Status;
        }
    }
    } // 'query

    return status;
}
#[no_mangle]
pub unsafe extern "C" fn Bus_PDO_QueryDeviceText(
    DeviceData: PPDO_DEVICE_DATA,
    Irp: PIRP,
) -> NTSTATUS {
    let mut Buffer: PWCHAR = core::ptr::null_mut();
    let mut Temp: PWCHAR = core::ptr::null_mut();
    let stack: PIO_STACK_LOCATION;
    let mut status: NTSTATUS = (*Irp).IoStatus.__bindgen_anon_1.Status;

    stack = IoGetCurrentIrpStackLocation(Irp);

    match (*stack).Parameters.QueryDeviceText.DeviceTextType as u32 {
        0 => {
            if (*Irp).IoStatus.Information == 0 {
                if !wcsstr((*DeviceData).HardwareIDs, ntkrnl_sys::w!("PNP000").as_ptr()).is_null() {
                    Temp = ntkrnl_sys::w!("Programmable interrupt controller").as_ptr() as PWCHAR;
                } else if !wcsstr((*DeviceData).HardwareIDs, ntkrnl_sys::w!("PNP010").as_ptr()).is_null() {
                    Temp = ntkrnl_sys::w!("System timer").as_ptr() as PWCHAR;
                } else if !wcsstr((*DeviceData).HardwareIDs, ntkrnl_sys::w!("PNP020").as_ptr()).is_null() {
                    Temp = ntkrnl_sys::w!("DMA controller").as_ptr() as PWCHAR;
                } else if !wcsstr((*DeviceData).HardwareIDs, ntkrnl_sys::w!("PNP03").as_ptr()).is_null() {
                    Temp = ntkrnl_sys::w!("Keyboard").as_ptr() as PWCHAR;
                } else if !wcsstr((*DeviceData).HardwareIDs, ntkrnl_sys::w!("PNP040").as_ptr()).is_null() {
                    Temp = ntkrnl_sys::w!("Parallel port").as_ptr() as PWCHAR;
                } else if !wcsstr((*DeviceData).HardwareIDs, ntkrnl_sys::w!("PNP05").as_ptr()).is_null() {
                    Temp = ntkrnl_sys::w!("Serial port").as_ptr() as PWCHAR;
                } else if !wcsstr((*DeviceData).HardwareIDs, ntkrnl_sys::w!("PNP06").as_ptr()).is_null() {
                    Temp = ntkrnl_sys::w!("Disk controller").as_ptr() as PWCHAR;
                } else if !wcsstr((*DeviceData).HardwareIDs, ntkrnl_sys::w!("PNP07").as_ptr()).is_null() {
                    Temp = ntkrnl_sys::w!("Disk controller").as_ptr() as PWCHAR;
                } else if !wcsstr((*DeviceData).HardwareIDs, ntkrnl_sys::w!("PNP09").as_ptr()).is_null() {
                    Temp = ntkrnl_sys::w!("Display adapter").as_ptr() as PWCHAR;
                } else if !wcsstr((*DeviceData).HardwareIDs, ntkrnl_sys::w!("PNP0A0").as_ptr()).is_null() {
                    Temp = ntkrnl_sys::w!("Bus controller").as_ptr() as PWCHAR;
                } else if !wcsstr((*DeviceData).HardwareIDs, ntkrnl_sys::w!("PNP0E0").as_ptr()).is_null() {
                    Temp = ntkrnl_sys::w!("PCMCIA controller").as_ptr() as PWCHAR;
                } else if !wcsstr((*DeviceData).HardwareIDs, ntkrnl_sys::w!("PNP0F").as_ptr()).is_null() {
                    Temp = ntkrnl_sys::w!("Mouse device").as_ptr() as PWCHAR;
                } else if !wcsstr((*DeviceData).HardwareIDs, ntkrnl_sys::w!("PNP8").as_ptr()).is_null() {
                    Temp = ntkrnl_sys::w!("Network adapter").as_ptr() as PWCHAR;
                } else if !wcsstr((*DeviceData).HardwareIDs, ntkrnl_sys::w!("PNPA0").as_ptr()).is_null() {
                    Temp = ntkrnl_sys::w!("SCSI controller").as_ptr() as PWCHAR;
                } else if !wcsstr((*DeviceData).HardwareIDs, ntkrnl_sys::w!("PNPB0").as_ptr()).is_null() {
                    Temp = ntkrnl_sys::w!("Multimedia device").as_ptr() as PWCHAR;
                } else if !wcsstr((*DeviceData).HardwareIDs, ntkrnl_sys::w!("PNPC00").as_ptr()).is_null() {
                    Temp = ntkrnl_sys::w!("Modem").as_ptr() as PWCHAR;
                } else if !wcsstr((*DeviceData).HardwareIDs, ntkrnl_sys::w!("PNP0C0C").as_ptr()).is_null() {
                    Temp = ntkrnl_sys::w!("Power Button").as_ptr() as PWCHAR;
                } else if !wcsstr((*DeviceData).HardwareIDs, ntkrnl_sys::w!("PNP0C0E").as_ptr()).is_null() {
                    Temp = ntkrnl_sys::w!("Sleep Button").as_ptr() as PWCHAR;
                } else if !wcsstr((*DeviceData).HardwareIDs, ntkrnl_sys::w!("PNP0C0D").as_ptr()).is_null() {
                    Temp = ntkrnl_sys::w!("Lid Switch").as_ptr() as PWCHAR;
                } else if !wcsstr((*DeviceData).HardwareIDs, ntkrnl_sys::w!("PNP0C09").as_ptr()).is_null() {
                    Temp = ntkrnl_sys::w!("ACPI Embedded Controller").as_ptr() as PWCHAR;
                } else if !wcsstr((*DeviceData).HardwareIDs, ntkrnl_sys::w!("PNP0C0B").as_ptr()).is_null() {
                    Temp = ntkrnl_sys::w!("ACPI Fan").as_ptr() as PWCHAR;
                } else if !wcsstr((*DeviceData).HardwareIDs, ntkrnl_sys::w!("PNP0A03").as_ptr()).is_null()
                    || !wcsstr((*DeviceData).HardwareIDs, ntkrnl_sys::w!("PNP0A08").as_ptr()).is_null()
                {
                    Temp = ntkrnl_sys::w!("PCI Root Bridge").as_ptr() as PWCHAR;
                } else if !wcsstr((*DeviceData).HardwareIDs, ntkrnl_sys::w!("PNP0C0A").as_ptr()).is_null() {
                    Temp = ntkrnl_sys::w!("ACPI Battery").as_ptr() as PWCHAR;
                } else if !wcsstr((*DeviceData).HardwareIDs, ntkrnl_sys::w!("PNP0C0F").as_ptr()).is_null() {
                    Temp = ntkrnl_sys::w!("PCI Interrupt Link").as_ptr() as PWCHAR;
                } else if !wcsstr((*DeviceData).HardwareIDs, ntkrnl_sys::w!("ACPI_PWR").as_ptr()).is_null() {
                    Temp = ntkrnl_sys::w!("ACPI Power Resource").as_ptr() as PWCHAR;
                } else if !wcsstr((*DeviceData).HardwareIDs, ntkrnl_sys::w!("Processor").as_ptr()).is_null() {
                    if !ProcessorNameString.is_null() {
                        Temp = ProcessorNameString as PWCHAR;
                    } else {
                        Temp = ntkrnl_sys::w!("Processor").as_ptr() as PWCHAR;
                    }
                } else if !wcsstr((*DeviceData).HardwareIDs, ntkrnl_sys::w!("ThermalZone").as_ptr()).is_null() {
                    Temp = ntkrnl_sys::w!("ACPI Thermal Zone").as_ptr() as PWCHAR;
                } else if !wcsstr((*DeviceData).HardwareIDs, ntkrnl_sys::w!("ACPI0002").as_ptr()).is_null() {
                    Temp = ntkrnl_sys::w!("Smart Battery").as_ptr() as PWCHAR;
                } else if !wcsstr((*DeviceData).HardwareIDs, ntkrnl_sys::w!("ACPI0003").as_ptr()).is_null() {
                    Temp = ntkrnl_sys::w!("AC Adapter").as_ptr() as PWCHAR;
                } else if (*DeviceData).AcpiHandle.is_null() {
                    Temp = ntkrnl_sys::w!("ACPI Fixed Feature Button").as_ptr() as PWCHAR;
                } else {
                    Temp = ntkrnl_sys::w!("Other ACPI device").as_ptr() as PWCHAR;
                }

                Buffer = ExAllocatePoolWithTag(
                    PagedPool,
                    (wcslen(Temp) + 1) * core::mem::size_of::<u16>(),
                    u32::from_be_bytes(*b"IpcA") as u32,
                ) as PWCHAR;

                if Buffer.is_null() {
                    status = STATUS_INSUFFICIENT_RESOURCES;
                } else {
                    core::ptr::copy_nonoverlapping(
                        Temp as *const u8,
                        Buffer as *mut u8,
                        (wcslen(Temp) + 1) * core::mem::size_of::<u16>(),
                    );
                    (*Irp).IoStatus.Information = Buffer as ULONG_PTR;
                    status = STATUS_SUCCESS;
                }
            }
        }
        _ => {}
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn Bus_PDO_QueryResources(
    DeviceData: PPDO_DEVICE_DATA,
    Irp: PIRP,
) -> NTSTATUS {
    let mut NumberOfResources: ULONG = 0;
    let mut ResourceList: PCM_RESOURCE_LIST = core::ptr::null_mut();
    let mut ResourceDescriptor: PCM_PARTIAL_RESOURCE_DESCRIPTOR = core::ptr::null_mut();
    let mut AcpiStatus: ACPI_STATUS = 0;
    let mut Buffer: ACPI_BUFFER = ACPI_BUFFER {
        Length: 0,
        Pointer: core::ptr::null_mut(),
    };
    let mut resource: *mut ACPI_RESOURCE = core::ptr::null_mut();
    let mut ResourceListSize: ULONG = 0;
    let mut i: ULONG = 0;
    let mut BusNumber: ULONGLONG = 0;
    let mut device: *mut acpi_device = core::ptr::null_mut();

    if (*DeviceData).AcpiHandle.is_null() {
        return (*Irp).IoStatus.__bindgen_anon_1.Status;
    }

    if !wcsstr(
        (*DeviceData).HardwareIDs,
        ntkrnl_sys::w!("PNP0A03").as_ptr(),
    )
    .is_null()
        || !wcsstr(
            (*DeviceData).HardwareIDs,
            ntkrnl_sys::w!("PNP0A08").as_ptr(),
        )
        .is_null()
    {
        acpi_bus_get_device((*DeviceData).AcpiHandle, &mut device);

        AcpiStatus = acpi_evaluate_integer(
            (*DeviceData).AcpiHandle,
            b"_BBN\0".as_ptr() as *mut i8,
            core::ptr::null_mut(),
            &mut BusNumber,
        );
        if AcpiStatus != AE_OK {
            BusNumber = 0;
        }

        ResourceListSize = core::mem::size_of::<CM_RESOURCE_LIST>() as ULONG;
        ResourceList = ExAllocatePoolWithTag(
            PagedPool,
            ResourceListSize as usize,
            0x41637052,
        ) as PCM_RESOURCE_LIST;
        if ResourceList.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        (*ResourceList).Count = 1;
        (*ResourceList).List[0].InterfaceType = Internal;
        (*ResourceList).List[0].BusNumber = 0;
        (*ResourceList).List[0].PartialResourceList.Version = 1;
        (*ResourceList).List[0].PartialResourceList.Revision = 1;
        (*ResourceList).List[0].PartialResourceList.Count = 1;
        ResourceDescriptor = &mut (*ResourceList).List[0].PartialResourceList.PartialDescriptors
            as *mut _ as PCM_PARTIAL_RESOURCE_DESCRIPTOR;

        (*ResourceDescriptor).Type = CmResourceTypeBusNumber as u8;
        (*ResourceDescriptor).ShareDisposition = CmResourceShareDeviceExclusive as u8;
        (*ResourceDescriptor).u.BusNumber.Start = BusNumber as ULONG;
        (*ResourceDescriptor).u.BusNumber.Length = 1;

        (*Irp).IoStatus.Information = ResourceList as ULONG_PTR;
        return STATUS_SUCCESS;
    }

    Buffer.Length = 0;
    AcpiStatus = AcpiGetCurrentResources((*DeviceData).AcpiHandle, &mut Buffer);
    if (AcpiStatus != 0 && AcpiStatus != AE_BUFFER_OVERFLOW) || Buffer.Length == 0 {
        return (*Irp).IoStatus.__bindgen_anon_1.Status;
    }

    Buffer.Pointer = ExAllocatePoolWithTag(PagedPool, Buffer.Length as usize, 0x41637042) as *mut core::ffi::c_void;
    if Buffer.Pointer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    AcpiStatus = AcpiGetCurrentResources((*DeviceData).AcpiHandle, &mut Buffer);
    if AcpiStatus != 0 {
        return STATUS_UNSUCCESSFUL;
    }

    resource = Buffer.Pointer as *mut ACPI_RESOURCE;
    while (*resource).Type != ACPI_RESOURCE_TYPE_END_TAG {
        match (*resource).Type {
            15 => {
                let irq_data = &mut (*resource).Data as *mut _ as *mut ACPI_RESOURCE_EXTENDED_IRQ;
                if (*irq_data).ProducerConsumer != ACPI_PRODUCER {
                    NumberOfResources = NumberOfResources.wrapping_add((*irq_data).InterruptCount as u32);
                }
            }
            0 => {
                let irq_data = &mut (*resource).Data as *mut _ as *mut ACPI_RESOURCE_IRQ;
                NumberOfResources = NumberOfResources.wrapping_add((*irq_data).InterruptCount as u32);
            }
            1 => {
                let dma_data = &mut (*resource).Data as *mut _ as *mut ACPI_RESOURCE_DMA;
                NumberOfResources = NumberOfResources.wrapping_add((*dma_data).ChannelCount as u32);
            }
            11 | 12 | 13 | 14 => {
                let addr_res = &mut (*resource).Data as *mut _ as *mut ACPI_RESOURCE_ADDRESS;
                if (*addr_res).ProducerConsumer != ACPI_PRODUCER {
                    NumberOfResources = NumberOfResources.wrapping_add(1);
                }
            }
            8 | 9 | 10 | 5 | 4 => {
                NumberOfResources = NumberOfResources.wrapping_add(1);
            }
            _ => {}
        }
        resource = (resource as *mut u8).offset((*resource).Length as isize) as *mut ACPI_RESOURCE;
    }

    ResourceListSize = (core::mem::size_of::<CM_RESOURCE_LIST>() as usize)
        .wrapping_add(
            (core::mem::size_of::<CM_PARTIAL_RESOURCE_DESCRIPTOR>() as usize)
                .wrapping_mul((NumberOfResources.wrapping_sub(1)) as usize),
        ) as ULONG;
    ResourceList = ExAllocatePoolWithTag(PagedPool, ResourceListSize as usize, 0x41637052) as PCM_RESOURCE_LIST;
    if ResourceList.is_null() {
        ExFreePoolWithTag(Buffer.Pointer, 0x41637042);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*ResourceList).Count = 1;
    (*ResourceList).List[0].InterfaceType = Internal;
    (*ResourceList).List[0].BusNumber = 0;
    (*ResourceList).List[0].PartialResourceList.Version = 1;
    (*ResourceList).List[0].PartialResourceList.Revision = 1;
    (*ResourceList).List[0].PartialResourceList.Count = NumberOfResources;
    ResourceDescriptor = &mut (*ResourceList).List[0].PartialResourceList.PartialDescriptors
        as *mut _ as PCM_PARTIAL_RESOURCE_DESCRIPTOR;

    resource = Buffer.Pointer as *mut ACPI_RESOURCE;
    while (*resource).Type != ACPI_RESOURCE_TYPE_END_TAG {
        match (*resource).Type {
            15 => {
                let irq_data = &mut (*resource).Data as *mut _ as *mut ACPI_RESOURCE_EXTENDED_IRQ;
                if (*irq_data).ProducerConsumer != ACPI_PRODUCER {
                    i = 0;
                    while i < (*irq_data).InterruptCount as u32 {
                        (*ResourceDescriptor).Type = CmResourceTypeInterrupt as u8;
                        (*ResourceDescriptor).ShareDisposition = if (*irq_data).Shareable == ACPI_SHARED {
                            CmResourceShareShared as u8
                        } else {
                            CmResourceShareDeviceExclusive as u8
                        };
                        (*ResourceDescriptor).Flags = if (*irq_data).Triggering == ACPI_LEVEL_SENSITIVE {
                            CM_RESOURCE_INTERRUPT_LEVEL_SENSITIVE
                        } else {
                            CM_RESOURCE_INTERRUPT_LATCHED
                        } as u16;
                        (*ResourceDescriptor).u.Interrupt.Level = core::ptr::read_unaligned((core::ptr::addr_of!((*irq_data).Interrupts) as *const u32).add(i as usize));
                        (*ResourceDescriptor).u.Interrupt.Vector = core::ptr::read_unaligned((core::ptr::addr_of!((*irq_data).Interrupts) as *const u32).add(i as usize));
                        (*ResourceDescriptor).u.Interrupt.Affinity = !0 as KAFFINITY;
                        ResourceDescriptor = ResourceDescriptor.offset(1);
                        i = i.wrapping_add(1);
                    }
                }
            }
            0 => {
                let irq_data = &mut (*resource).Data as *mut _ as *mut ACPI_RESOURCE_IRQ;
                i = 0;
                while i < (*irq_data).InterruptCount as u32 {
                    (*ResourceDescriptor).Type = CmResourceTypeInterrupt as u8;
                    (*ResourceDescriptor).ShareDisposition = if (*irq_data).Shareable == ACPI_SHARED {
                        CmResourceShareShared as u8
                    } else {
                        CmResourceShareDeviceExclusive as u8
                    };
                    (*ResourceDescriptor).Flags = if (*irq_data).Triggering == ACPI_LEVEL_SENSITIVE {
                        CM_RESOURCE_INTERRUPT_LEVEL_SENSITIVE
                    } else {
                        CM_RESOURCE_INTERRUPT_LATCHED
                    } as u16;
                    (*ResourceDescriptor).u.Interrupt.Level = core::ptr::read_unaligned((core::ptr::addr_of!((*irq_data).Interrupts) as *const u8).add(i as usize)) as ULONG;
                    (*ResourceDescriptor).u.Interrupt.Vector = core::ptr::read_unaligned((core::ptr::addr_of!((*irq_data).Interrupts) as *const u8).add(i as usize)) as ULONG;
                    (*ResourceDescriptor).u.Interrupt.Affinity = !0 as KAFFINITY;
                    ResourceDescriptor = ResourceDescriptor.offset(1);
                    i = i.wrapping_add(1);
                }
            }
            1 => {
                let dma_data = &mut (*resource).Data as *mut _ as *mut ACPI_RESOURCE_DMA;
                i = 0;
                while i < (*dma_data).ChannelCount as u32 {
                    (*ResourceDescriptor).Type = CmResourceTypeDma as u8;
                    (*ResourceDescriptor).Flags = 0;
                    match (*dma_data).Type {
                        1 => (*ResourceDescriptor).Flags |= CM_RESOURCE_DMA_TYPE_A as u16,
                        2 => (*ResourceDescriptor).Flags |= CM_RESOURCE_DMA_TYPE_B as u16,
                        3 => (*ResourceDescriptor).Flags |= CM_RESOURCE_DMA_TYPE_F as u16,
                        _ => {}
                    }
                    if (*dma_data).BusMaster == ACPI_BUS_MASTER {
                        (*ResourceDescriptor).Flags |= CM_RESOURCE_DMA_BUS_MASTER as u16;
                    }
                    match (*dma_data).Transfer {
                        0 => (*ResourceDescriptor).Flags |= CM_RESOURCE_DMA_8 as u16,
                        2 => (*ResourceDescriptor).Flags |= CM_RESOURCE_DMA_16 as u16,
                        1 => (*ResourceDescriptor).Flags |= CM_RESOURCE_DMA_8_AND_16 as u16,
                        _ => {}
                    }
                    (*ResourceDescriptor).u.Dma.Channel = *(*dma_data).Channels.as_ptr().add(i as usize) as ULONG;
                    ResourceDescriptor = ResourceDescriptor.offset(1);
                    i = i.wrapping_add(1);
                }
            }
            4 => {
                let io_data = &mut (*resource).Data as *mut _ as *mut ACPI_RESOURCE_IO;
                (*ResourceDescriptor).Type = CmResourceTypePort as u8;
                (*ResourceDescriptor).ShareDisposition = CmResourceShareDriverExclusive as u8;
                (*ResourceDescriptor).Flags = CM_RESOURCE_PORT_IO as u16;
                if (*io_data).IoDecode == ACPI_DECODE_16 {
                    (*ResourceDescriptor).Flags |= CM_RESOURCE_PORT_16_BIT_DECODE as u16;
                } else {
                    (*ResourceDescriptor).Flags |= CM_RESOURCE_PORT_10_BIT_DECODE as u16;
                }
                (*ResourceDescriptor).u.Port.Start.QuadPart = (*io_data).Minimum as i64;
                (*ResourceDescriptor).u.Port.Length = (*io_data).AddressLength as ULONG;
                ResourceDescriptor = ResourceDescriptor.offset(1);
            }
            5 => {
                let io_data = &mut (*resource).Data as *mut _ as *mut ACPI_RESOURCE_FIXED_IO;
                (*ResourceDescriptor).Type = CmResourceTypePort as u8;
                (*ResourceDescriptor).ShareDisposition = CmResourceShareDriverExclusive as u8;
                (*ResourceDescriptor).Flags = CM_RESOURCE_PORT_IO as u16;
                (*ResourceDescriptor).u.Port.Start.QuadPart = (*io_data).Address as i64;
                (*ResourceDescriptor).u.Port.Length = (*io_data).AddressLength as ULONG;
                ResourceDescriptor = ResourceDescriptor.offset(1);
            }
            11 => {
                let addr16_data = &mut (*resource).Data as *mut _ as *mut ACPI_RESOURCE_ADDRESS16;
                if (*addr16_data).ProducerConsumer != ACPI_PRODUCER {
                    if (*addr16_data).ResourceType == ACPI_BUS_NUMBER_RANGE {
                        (*ResourceDescriptor).Type = CmResourceTypeBusNumber as u8;
                        (*ResourceDescriptor).ShareDisposition = CmResourceShareShared as u8;
                        (*ResourceDescriptor).Flags = 0;
                        (*ResourceDescriptor).u.BusNumber.Start = (*addr16_data).Address.Minimum as ULONG;
                        (*ResourceDescriptor).u.BusNumber.Length = (*addr16_data).Address.AddressLength as ULONG;
                    } else if (*addr16_data).ResourceType == ACPI_IO_RANGE {
                        (*ResourceDescriptor).Type = CmResourceTypePort as u8;
                        (*ResourceDescriptor).ShareDisposition = CmResourceShareDeviceExclusive as u8;
                        (*ResourceDescriptor).Flags = CM_RESOURCE_PORT_IO as u16;
                        if (*addr16_data).Decode == ACPI_POS_DECODE {
                            (*ResourceDescriptor).Flags |= CM_RESOURCE_PORT_POSITIVE_DECODE as u16;
                        }
                        (*ResourceDescriptor).u.Port.Start.QuadPart = (*addr16_data).Address.Minimum as i64;
                        (*ResourceDescriptor).u.Port.Length = (*addr16_data).Address.AddressLength as ULONG;
                    } else {
                        (*ResourceDescriptor).Type = CmResourceTypeMemory as u8;
                        (*ResourceDescriptor).ShareDisposition = CmResourceShareDeviceExclusive as u8;
                        (*ResourceDescriptor).Flags = 0;
                        if (*addr16_data).Info.Mem.WriteProtect == ACPI_READ_ONLY_MEMORY {
                            (*ResourceDescriptor).Flags |= CM_RESOURCE_MEMORY_READ_ONLY as u16;
                        } else {
                            (*ResourceDescriptor).Flags |= CM_RESOURCE_MEMORY_READ_WRITE as u16;
                        }
                        match (*addr16_data).Info.Mem.Caching {
                            1 => (*ResourceDescriptor).Flags |= CM_RESOURCE_MEMORY_CACHEABLE as u16,
                            2 => (*ResourceDescriptor).Flags |= CM_RESOURCE_MEMORY_COMBINEDWRITE as u16,
                            3 => (*ResourceDescriptor).Flags |= CM_RESOURCE_MEMORY_PREFETCHABLE as u16,
                            _ => {}
                        }
                        (*ResourceDescriptor).u.Memory.Start.QuadPart = (*addr16_data).Address.Minimum as i64;
                        (*ResourceDescriptor).u.Memory.Length = (*addr16_data).Address.AddressLength as ULONG;
                    }
                    ResourceDescriptor = ResourceDescriptor.offset(1);
                }
            }
            12 => {
                let addr32_data = &mut (*resource).Data as *mut _ as *mut ACPI_RESOURCE_ADDRESS32;
                if (*addr32_data).ProducerConsumer != ACPI_PRODUCER {
                    if (*addr32_data).ResourceType == ACPI_BUS_NUMBER_RANGE {
                        (*ResourceDescriptor).Type = CmResourceTypeBusNumber as u8;
                        (*ResourceDescriptor).ShareDisposition = CmResourceShareShared as u8;
                        (*ResourceDescriptor).Flags = 0;
                        (*ResourceDescriptor).u.BusNumber.Start = (*addr32_data).Address.Minimum as ULONG;
                        (*ResourceDescriptor).u.BusNumber.Length = (*addr32_data).Address.AddressLength as ULONG;
                    } else if (*addr32_data).ResourceType == ACPI_IO_RANGE {
                        (*ResourceDescriptor).Type = CmResourceTypePort as u8;
                        (*ResourceDescriptor).ShareDisposition = CmResourceShareDeviceExclusive as u8;
                        (*ResourceDescriptor).Flags = CM_RESOURCE_PORT_IO as u16;
                        if (*addr32_data).Decode == ACPI_POS_DECODE {
                            (*ResourceDescriptor).Flags |= CM_RESOURCE_PORT_POSITIVE_DECODE as u16;
                        }
                        (*ResourceDescriptor).u.Port.Start.QuadPart = (*addr32_data).Address.Minimum as i64;
                        (*ResourceDescriptor).u.Port.Length = (*addr32_data).Address.AddressLength as ULONG;
                    } else {
                        (*ResourceDescriptor).Type = CmResourceTypeMemory as u8;
                        (*ResourceDescriptor).ShareDisposition = CmResourceShareDeviceExclusive as u8;
                        (*ResourceDescriptor).Flags = 0;
                        if (*addr32_data).Info.Mem.WriteProtect == ACPI_READ_ONLY_MEMORY {
                            (*ResourceDescriptor).Flags |= CM_RESOURCE_MEMORY_READ_ONLY as u16;
                        } else {
                            (*ResourceDescriptor).Flags |= CM_RESOURCE_MEMORY_READ_WRITE as u16;
                        }
                        match (*addr32_data).Info.Mem.Caching {
                            1 => (*ResourceDescriptor).Flags |= CM_RESOURCE_MEMORY_CACHEABLE as u16,
                            2 => (*ResourceDescriptor).Flags |= CM_RESOURCE_MEMORY_COMBINEDWRITE as u16,
                            3 => (*ResourceDescriptor).Flags |= CM_RESOURCE_MEMORY_PREFETCHABLE as u16,
                            _ => {}
                        }
                        (*ResourceDescriptor).u.Memory.Start.QuadPart = (*addr32_data).Address.Minimum as i64;
                        (*ResourceDescriptor).u.Memory.Length = (*addr32_data).Address.AddressLength as ULONG;
                    }
                    ResourceDescriptor = ResourceDescriptor.offset(1);
                }
            }
            13 => {
                let addr64_data = &mut (*resource).Data as *mut _ as *mut ACPI_RESOURCE_ADDRESS64;
                if (*addr64_data).ProducerConsumer != ACPI_PRODUCER {
                    if (*addr64_data).ResourceType == ACPI_BUS_NUMBER_RANGE {
                        (*ResourceDescriptor).Type = CmResourceTypeBusNumber as u8;
                        (*ResourceDescriptor).ShareDisposition = CmResourceShareShared as u8;
                        (*ResourceDescriptor).Flags = 0;
                        (*ResourceDescriptor).u.BusNumber.Start = (*addr64_data).Address.Minimum as ULONG;
                        (*ResourceDescriptor).u.BusNumber.Length = (*addr64_data).Address.AddressLength as ULONG;
                    } else if (*addr64_data).ResourceType == ACPI_IO_RANGE {
                        (*ResourceDescriptor).Type = CmResourceTypePort as u8;
                        (*ResourceDescriptor).ShareDisposition = CmResourceShareDeviceExclusive as u8;
                        (*ResourceDescriptor).Flags = CM_RESOURCE_PORT_IO as u16;
                        if (*addr64_data).Decode == ACPI_POS_DECODE {
                            (*ResourceDescriptor).Flags |= CM_RESOURCE_PORT_POSITIVE_DECODE as u16;
                        }
                        (*ResourceDescriptor).u.Port.Start.QuadPart = (*addr64_data).Address.Minimum as i64;
                        (*ResourceDescriptor).u.Port.Length = (*addr64_data).Address.AddressLength as ULONG;
                    } else {
                        (*ResourceDescriptor).Type = CmResourceTypeMemory as u8;
                        (*ResourceDescriptor).ShareDisposition = CmResourceShareDeviceExclusive as u8;
                        (*ResourceDescriptor).Flags = 0;
                        if (*addr64_data).Info.Mem.WriteProtect == ACPI_READ_ONLY_MEMORY {
                            (*ResourceDescriptor).Flags |= CM_RESOURCE_MEMORY_READ_ONLY as u16;
                        } else {
                            (*ResourceDescriptor).Flags |= CM_RESOURCE_MEMORY_READ_WRITE as u16;
                        }
                        match (*addr64_data).Info.Mem.Caching {
                            1 => (*ResourceDescriptor).Flags |= CM_RESOURCE_MEMORY_CACHEABLE as u16,
                            2 => (*ResourceDescriptor).Flags |= CM_RESOURCE_MEMORY_COMBINEDWRITE as u16,
                            3 => (*ResourceDescriptor).Flags |= CM_RESOURCE_MEMORY_PREFETCHABLE as u16,
                            _ => {}
                        }
                        (*ResourceDescriptor).u.Memory.Start.QuadPart = (*addr64_data).Address.Minimum as i64;
                        (*ResourceDescriptor).u.Memory.Length = (*addr64_data).Address.AddressLength as ULONG;
                    }
                    ResourceDescriptor = ResourceDescriptor.offset(1);
                }
            }
            14 => {
                let addr64_data = &mut (*resource).Data as *mut _ as *mut ACPI_RESOURCE_EXTENDED_ADDRESS64;
                if (*addr64_data).ProducerConsumer != ACPI_PRODUCER {
                    if (*addr64_data).ResourceType == ACPI_BUS_NUMBER_RANGE {
                        (*ResourceDescriptor).Type = CmResourceTypeBusNumber as u8;
                        (*ResourceDescriptor).ShareDisposition = CmResourceShareShared as u8;
                        (*ResourceDescriptor).Flags = 0;
                        (*ResourceDescriptor).u.BusNumber.Start = (*addr64_data).Address.Minimum as ULONG;
                        (*ResourceDescriptor).u.BusNumber.Length = (*addr64_data).Address.AddressLength as ULONG;
                    } else if (*addr64_data).ResourceType == ACPI_IO_RANGE {
                        (*ResourceDescriptor).Type = CmResourceTypePort as u8;
                        (*ResourceDescriptor).ShareDisposition = CmResourceShareDeviceExclusive as u8;
                        (*ResourceDescriptor).Flags = CM_RESOURCE_PORT_IO as u16;
                        if (*addr64_data).Decode == ACPI_POS_DECODE {
                            (*ResourceDescriptor).Flags |= CM_RESOURCE_PORT_POSITIVE_DECODE as u16;
                        }
                        (*ResourceDescriptor).u.Port.Start.QuadPart = (*addr64_data).Address.Minimum as i64;
                        (*ResourceDescriptor).u.Port.Length = (*addr64_data).Address.AddressLength as ULONG;
                    } else {
                        (*ResourceDescriptor).Type = CmResourceTypeMemory as u8;
                        (*ResourceDescriptor).ShareDisposition = CmResourceShareDeviceExclusive as u8;
                        (*ResourceDescriptor).Flags = 0;
                        if (*addr64_data).Info.Mem.WriteProtect == ACPI_READ_ONLY_MEMORY {
                            (*ResourceDescriptor).Flags |= CM_RESOURCE_MEMORY_READ_ONLY as u16;
                        } else {
                            (*ResourceDescriptor).Flags |= CM_RESOURCE_MEMORY_READ_WRITE as u16;
                        }
                        match (*addr64_data).Info.Mem.Caching {
                            1 => (*ResourceDescriptor).Flags |= CM_RESOURCE_MEMORY_CACHEABLE as u16,
                            2 => (*ResourceDescriptor).Flags |= CM_RESOURCE_MEMORY_COMBINEDWRITE as u16,
                            3 => (*ResourceDescriptor).Flags |= CM_RESOURCE_MEMORY_PREFETCHABLE as u16,
                            _ => {}
                        }
                        (*ResourceDescriptor).u.Memory.Start.QuadPart = (*addr64_data).Address.Minimum as i64;
                        (*ResourceDescriptor).u.Memory.Length = (*addr64_data).Address.AddressLength as ULONG;
                    }
                    ResourceDescriptor = ResourceDescriptor.offset(1);
                }
            }
            8 => {
                let mem24_data = &mut (*resource).Data as *mut _ as *mut ACPI_RESOURCE_MEMORY24;
                (*ResourceDescriptor).Type = CmResourceTypeMemory as u8;
                (*ResourceDescriptor).ShareDisposition = CmResourceShareDeviceExclusive as u8;
                (*ResourceDescriptor).Flags = CM_RESOURCE_MEMORY_24 as u16;
                if (*mem24_data).WriteProtect == ACPI_READ_ONLY_MEMORY {
                    (*ResourceDescriptor).Flags |= CM_RESOURCE_MEMORY_READ_ONLY as u16;
                } else {
                    (*ResourceDescriptor).Flags |= CM_RESOURCE_MEMORY_READ_WRITE as u16;
                }
                (*ResourceDescriptor).u.Memory.Start.QuadPart = (*mem24_data).Minimum as i64;
                (*ResourceDescriptor).u.Memory.Length = (*mem24_data).AddressLength as ULONG;
                ResourceDescriptor = ResourceDescriptor.offset(1);
            }
            9 => {
                let mem32_data = &mut (*resource).Data as *mut _ as *mut ACPI_RESOURCE_MEMORY32;
                (*ResourceDescriptor).Type = CmResourceTypeMemory as u8;
                (*ResourceDescriptor).ShareDisposition = CmResourceShareDeviceExclusive as u8;
                (*ResourceDescriptor).Flags = 0;
                if (*mem32_data).WriteProtect == ACPI_READ_ONLY_MEMORY {
                    (*ResourceDescriptor).Flags |= CM_RESOURCE_MEMORY_READ_ONLY as u16;
                } else {
                    (*ResourceDescriptor).Flags |= CM_RESOURCE_MEMORY_READ_WRITE as u16;
                }
                (*ResourceDescriptor).u.Memory.Start.QuadPart = (*mem32_data).Minimum as i64;
                (*ResourceDescriptor).u.Memory.Length = (*mem32_data).AddressLength as ULONG;
                ResourceDescriptor = ResourceDescriptor.offset(1);
            }
            10 => {
                let memfixed32_data = &mut (*resource).Data as *mut _ as *mut ACPI_RESOURCE_FIXED_MEMORY32;
                (*ResourceDescriptor).Type = CmResourceTypeMemory as u8;
                (*ResourceDescriptor).ShareDisposition = CmResourceShareDeviceExclusive as u8;
                (*ResourceDescriptor).Flags = 0;
                if (*memfixed32_data).WriteProtect == ACPI_READ_ONLY_MEMORY {
                    (*ResourceDescriptor).Flags |= CM_RESOURCE_MEMORY_READ_ONLY as u16;
                } else {
                    (*ResourceDescriptor).Flags |= CM_RESOURCE_MEMORY_READ_WRITE as u16;
                }
                (*ResourceDescriptor).u.Memory.Start.QuadPart = (*memfixed32_data).Address as i64;
                (*ResourceDescriptor).u.Memory.Length = (*memfixed32_data).AddressLength as ULONG;
                ResourceDescriptor = ResourceDescriptor.offset(1);
            }
            _ => {}
        }
        resource = (resource as *mut u8).offset((*resource).Length as isize) as *mut ACPI_RESOURCE;
    }

    ExFreePoolWithTag(Buffer.Pointer, 0x41637042);
    (*Irp).IoStatus.Information = ResourceList as ULONG_PTR;
    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn Bus_PDO_QueryResourceRequirements(
    DeviceData: PPDO_DEVICE_DATA,
    Irp: PIRP,
) -> NTSTATUS {
    let mut NumberOfResources: ULONG = 0;
    let mut AcpiStatus: ACPI_STATUS;
    let mut Buffer: ACPI_BUFFER = ACPI_BUFFER {
        Length: 0,
        Pointer: core::ptr::null_mut(),
    };
    let mut resource: *mut ACPI_RESOURCE;
    let mut i: ULONG;
    let RequirementsListSize: ULONG;
    let RequirementsList: PIO_RESOURCE_REQUIREMENTS_LIST;
    let mut RequirementDescriptor: PIO_RESOURCE_DESCRIPTOR;
    let mut CurrentRes: BOOLEAN = 0;
    let mut SeenStartDependent: BOOLEAN;

    if (*DeviceData).AcpiHandle.is_null() {
        return (*Irp).IoStatus.__bindgen_anon_1.Status;
    }

    if !wcsstr(
        (*DeviceData).HardwareIDs,
        ntkrnl_sys::w!("PNP0A03").as_ptr(),
    ).is_null() || !wcsstr(
        (*DeviceData).HardwareIDs,
        ntkrnl_sys::w!("PNP0A08").as_ptr(),
    ).is_null() {
        return (*Irp).IoStatus.__bindgen_anon_1.Status;
    }

    loop {
        Buffer.Length = 0;
        if CurrentRes != 0 {
            AcpiStatus = AcpiGetCurrentResources((*DeviceData).AcpiHandle, &mut Buffer);
        } else {
            AcpiStatus = AcpiGetPossibleResources((*DeviceData).AcpiHandle, &mut Buffer);
        }
        if (ACPI_SUCCESS(AcpiStatus) || AcpiStatus == AE_BUFFER_OVERFLOW) && Buffer.Length != 0 {
            break;
        }
        if CurrentRes == 0 {
            CurrentRes = 1;
        } else {
            return (*Irp).IoStatus.__bindgen_anon_1.Status;
        }
    }

    Buffer.Pointer = ExAllocatePoolWithTag(PagedPool, Buffer.Length as usize, u32::from_be_bytes(*b"BpcA") as ULONG) as *mut core::ffi::c_void;
    if Buffer.Pointer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    if CurrentRes != 0 {
        AcpiStatus = AcpiGetCurrentResources((*DeviceData).AcpiHandle, &mut Buffer);
    } else {
        AcpiStatus = AcpiGetPossibleResources((*DeviceData).AcpiHandle, &mut Buffer);
    }
    if !ACPI_SUCCESS(AcpiStatus) {
        return STATUS_UNSUCCESSFUL;
    }

    SeenStartDependent = 0;
    resource = Buffer.Pointer as *mut ACPI_RESOURCE;
    while (*resource).Type != ACPI_RESOURCE_TYPE_END_TAG && (*resource).Type != ACPI_RESOURCE_TYPE_END_DEPENDENT {
        if (*resource).Type == ACPI_RESOURCE_TYPE_START_DEPENDENT {
            if SeenStartDependent != 0 {
                break;
            }
            SeenStartDependent = 1;
        }
        match (*resource).Type {
            15 => {
                let irq_data = &mut (*resource).Data.ExtendedIrq;
                if irq_data.ProducerConsumer != ACPI_PRODUCER {
                    NumberOfResources = NumberOfResources.wrapping_add(irq_data.InterruptCount as u32);
                }
            }
            0 => {
                let irq_data = &mut (*resource).Data.Irq;
                NumberOfResources = NumberOfResources.wrapping_add(irq_data.InterruptCount as u32);
            }
            1 => {
                let dma_data = &mut (*resource).Data.Dma;
                NumberOfResources = NumberOfResources.wrapping_add(dma_data.ChannelCount as u32);
            }
            11 | 12 | 13 | 14 => {
                let res_addr = &mut (*resource).Data.Address;
                if res_addr.ProducerConsumer != ACPI_PRODUCER {
                    NumberOfResources = NumberOfResources.wrapping_add(1);
                }
            }
            8 | 9 | 10 | 5 | 4 => {
                NumberOfResources = NumberOfResources.wrapping_add(1);
            }
            _ => {}
        }
        resource = (resource as *mut u8).offset((*resource).Length as isize) as *mut ACPI_RESOURCE;
    }

    RequirementsListSize = (core::mem::size_of::<IO_RESOURCE_REQUIREMENTS_LIST>() as usize)
        .wrapping_add((core::mem::size_of::<IO_RESOURCE_DESCRIPTOR>() as usize).wrapping_mul((NumberOfResources.wrapping_sub(1)) as usize)) as ULONG;
    RequirementsList = ExAllocatePoolWithTag(PagedPool, RequirementsListSize as usize, u32::from_be_bytes(*b"RpcA") as ULONG) as PIO_RESOURCE_REQUIREMENTS_LIST;

    if RequirementsList.is_null() {
        ExFreePoolWithTag(Buffer.Pointer, u32::from_be_bytes(*b"BpcA") as ULONG);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*RequirementsList).ListSize = RequirementsListSize;
    (*RequirementsList).InterfaceType = Internal;
    (*RequirementsList).BusNumber = 0;
    (*RequirementsList).SlotNumber = 0;
    (*RequirementsList).AlternativeLists = 1;
    (*RequirementsList).List[0].Version = 1;
    (*RequirementsList).List[0].Revision = 1;
    (*RequirementsList).List[0].Count = NumberOfResources;
    RequirementDescriptor = &mut (*RequirementsList).List[0].Descriptors[0] as *mut IO_RESOURCE_DESCRIPTOR;

    SeenStartDependent = 0;
    resource = Buffer.Pointer as *mut ACPI_RESOURCE;
    while (*resource).Type != ACPI_RESOURCE_TYPE_END_TAG && (*resource).Type != ACPI_RESOURCE_TYPE_END_DEPENDENT {
        if (*resource).Type == ACPI_RESOURCE_TYPE_START_DEPENDENT {
            if SeenStartDependent != 0 {
                break;
            }
            SeenStartDependent = 1;
        }
        match (*resource).Type {
            15 => {
                let irq_data = &mut (*resource).Data.ExtendedIrq;
                if irq_data.ProducerConsumer != ACPI_PRODUCER {
                    i = 0;
                    while i < irq_data.InterruptCount as u32 {
                        (*RequirementDescriptor).Option = if i == 0 { IO_RESOURCE_PREFERRED } else { IO_RESOURCE_ALTERNATIVE } as u8;
                        (*RequirementDescriptor).Type = CmResourceTypeInterrupt as u8;
                        (*RequirementDescriptor).ShareDisposition = if irq_data.Shareable == ACPI_SHARED { CmResourceShareShared } else { CmResourceShareDeviceExclusive } as u8;
                        (*RequirementDescriptor).Flags = if irq_data.Triggering == ACPI_LEVEL_SENSITIVE { CM_RESOURCE_INTERRUPT_LEVEL_SENSITIVE } else { CM_RESOURCE_INTERRUPT_LATCHED } as u16;
                        (*RequirementDescriptor).u.Interrupt.MinimumVector = core::ptr::read_unaligned((core::ptr::addr_of!(irq_data.Interrupts) as *const u32).add(i as usize));
                        (*RequirementDescriptor).u.Interrupt.MaximumVector = core::ptr::read_unaligned((core::ptr::addr_of!(irq_data.Interrupts) as *const u32).add(i as usize));
                        RequirementDescriptor = RequirementDescriptor.offset(1);
                        i = i.wrapping_add(1);
                    }
                }
            }
            0 => {
                let irq_data = &mut (*resource).Data.Irq;
                i = 0;
                while i < irq_data.InterruptCount as u32 {
                    (*RequirementDescriptor).Option = if i == 0 { IO_RESOURCE_PREFERRED } else { IO_RESOURCE_ALTERNATIVE } as u8;
                    (*RequirementDescriptor).Type = CmResourceTypeInterrupt as u8;
                    (*RequirementDescriptor).ShareDisposition = if irq_data.Shareable == ACPI_SHARED { CmResourceShareShared } else { CmResourceShareDeviceExclusive } as u8;
                    (*RequirementDescriptor).Flags = if irq_data.Triggering == ACPI_LEVEL_SENSITIVE { CM_RESOURCE_INTERRUPT_LEVEL_SENSITIVE } else { CM_RESOURCE_INTERRUPT_LATCHED } as u16;
                    (*RequirementDescriptor).u.Interrupt.MinimumVector = core::ptr::read_unaligned((core::ptr::addr_of!(irq_data.Interrupts) as *const u8).add(i as usize)) as ULONG;
                    (*RequirementDescriptor).u.Interrupt.MaximumVector = core::ptr::read_unaligned((core::ptr::addr_of!(irq_data.Interrupts) as *const u8).add(i as usize)) as ULONG;
                    RequirementDescriptor = RequirementDescriptor.offset(1);
                    i = i.wrapping_add(1);
                }
            }
            1 => {
                let dma_data = &mut (*resource).Data.Dma;
                i = 0;
                while i < dma_data.ChannelCount as u32 {
                    (*RequirementDescriptor).Type = CmResourceTypeDma as u8;
                    (*RequirementDescriptor).Flags = 0;
                    match dma_data.Type {
                        1 => { (*RequirementDescriptor).Flags |= CM_RESOURCE_DMA_TYPE_A as u16; }
                        2 => { (*RequirementDescriptor).Flags |= CM_RESOURCE_DMA_TYPE_B as u16; }
                        3 => { (*RequirementDescriptor).Flags |= CM_RESOURCE_DMA_TYPE_F as u16; }
                        _ => {}
                    }
                    if dma_data.BusMaster == ACPI_BUS_MASTER {
                        (*RequirementDescriptor).Flags |= CM_RESOURCE_DMA_BUS_MASTER as u16;
                    }
                    match dma_data.Transfer {
                        0 => { (*RequirementDescriptor).Flags |= CM_RESOURCE_DMA_8 as u16; }
                        2 => { (*RequirementDescriptor).Flags |= CM_RESOURCE_DMA_16 as u16; }
                        1 => { (*RequirementDescriptor).Flags |= CM_RESOURCE_DMA_8_AND_16 as u16; }
                        _ => {}
                    }
                    (*RequirementDescriptor).Option = if i == 0 { IO_RESOURCE_PREFERRED } else { IO_RESOURCE_ALTERNATIVE } as u8;
                    (*RequirementDescriptor).ShareDisposition = CmResourceShareDriverExclusive as u8;
                    (*RequirementDescriptor).u.Dma.MinimumChannel = *dma_data.Channels.as_ptr().add(i as usize) as ULONG;
                    (*RequirementDescriptor).u.Dma.MaximumChannel = *dma_data.Channels.as_ptr().add(i as usize) as ULONG;
                    RequirementDescriptor = RequirementDescriptor.offset(1);
                    i = i.wrapping_add(1);
                }
            }
            4 => {
                let io_data = &mut (*resource).Data.Io;
                (*RequirementDescriptor).Flags = CM_RESOURCE_PORT_IO as u16;
                if io_data.IoDecode == ACPI_DECODE_16 {
                    (*RequirementDescriptor).Flags |= CM_RESOURCE_PORT_16_BIT_DECODE as u16;
                } else {
                    (*RequirementDescriptor).Flags |= CM_RESOURCE_PORT_10_BIT_DECODE as u16;
                }
                (*RequirementDescriptor).u.Port.Length = io_data.AddressLength as ULONG;
                (*RequirementDescriptor).Option = if CurrentRes != 0 { 0 } else { IO_RESOURCE_PREFERRED } as u8;
                (*RequirementDescriptor).Type = CmResourceTypePort as u8;
                (*RequirementDescriptor).ShareDisposition = CmResourceShareDriverExclusive as u8;
                (*RequirementDescriptor).u.Port.Alignment = io_data.Alignment as ULONG;
                (*RequirementDescriptor).u.Port.MinimumAddress.QuadPart = io_data.Minimum as i64;
                (*RequirementDescriptor).u.Port.MaximumAddress.QuadPart = (io_data.Maximum as i64).wrapping_add(io_data.AddressLength as i64).wrapping_sub(1);
                RequirementDescriptor = RequirementDescriptor.offset(1);
            }
            5 => {
                let io_data = &mut (*resource).Data.FixedIo;
                (*RequirementDescriptor).Flags = CM_RESOURCE_PORT_IO as u16;
                (*RequirementDescriptor).u.Port.Length = io_data.AddressLength as ULONG;
                (*RequirementDescriptor).Option = if CurrentRes != 0 { 0 } else { IO_RESOURCE_PREFERRED } as u8;
                (*RequirementDescriptor).Type = CmResourceTypePort as u8;
                (*RequirementDescriptor).ShareDisposition = CmResourceShareDriverExclusive as u8;
                (*RequirementDescriptor).u.Port.Alignment = 1;
                (*RequirementDescriptor).u.Port.MinimumAddress.QuadPart = io_data.Address as i64;
                (*RequirementDescriptor).u.Port.MaximumAddress.QuadPart = (io_data.Address as i64).wrapping_add(io_data.AddressLength as i64).wrapping_sub(1);
                RequirementDescriptor = RequirementDescriptor.offset(1);
            }
            11 => {
                let addr16_data = &mut (*resource).Data.Address16;
                if addr16_data.ProducerConsumer != ACPI_PRODUCER {
                    (*RequirementDescriptor).Option = if CurrentRes != 0 { 0 } else { IO_RESOURCE_PREFERRED } as u8;
                    if addr16_data.ResourceType == ACPI_BUS_NUMBER_RANGE {
                        (*RequirementDescriptor).Type = CmResourceTypeBusNumber as u8;
                        (*RequirementDescriptor).ShareDisposition = CmResourceShareShared as u8;
                        (*RequirementDescriptor).Flags = 0;
                        (*RequirementDescriptor).u.BusNumber.MinBusNumber = addr16_data.Address.Minimum as ULONG;
                        (*RequirementDescriptor).u.BusNumber.MaxBusNumber = (addr16_data.Address.Maximum as ULONG).wrapping_add(addr16_data.Address.AddressLength as ULONG).wrapping_sub(1);
                        (*RequirementDescriptor).u.BusNumber.Length = addr16_data.Address.AddressLength as ULONG;
                    } else if addr16_data.ResourceType == ACPI_IO_RANGE {
                        (*RequirementDescriptor).Type = CmResourceTypePort as u8;
                        (*RequirementDescriptor).ShareDisposition = CmResourceShareDeviceExclusive as u8;
                        (*RequirementDescriptor).Flags = CM_RESOURCE_PORT_IO as u16;
                        if addr16_data.Decode == ACPI_POS_DECODE {
                            (*RequirementDescriptor).Flags |= CM_RESOURCE_PORT_POSITIVE_DECODE as u16;
                        }
                        (*RequirementDescriptor).u.Port.MinimumAddress.QuadPart = addr16_data.Address.Minimum as i64;
                        (*RequirementDescriptor).u.Port.MaximumAddress.QuadPart = (addr16_data.Address.Maximum as i64).wrapping_add(addr16_data.Address.AddressLength as i64).wrapping_sub(1);
                        (*RequirementDescriptor).u.Port.Length = addr16_data.Address.AddressLength as ULONG;
                    } else {
                        (*RequirementDescriptor).Type = CmResourceTypeMemory as u8;
                        (*RequirementDescriptor).ShareDisposition = CmResourceShareDeviceExclusive as u8;
                        (*RequirementDescriptor).Flags = 0;
                        if addr16_data.Info.Mem.WriteProtect == ACPI_READ_ONLY_MEMORY {
                            (*RequirementDescriptor).Flags |= CM_RESOURCE_MEMORY_READ_ONLY as u16;
                        } else {
                            (*RequirementDescriptor).Flags |= CM_RESOURCE_MEMORY_READ_WRITE as u16;
                        }
                        match addr16_data.Info.Mem.Caching {
                            1 => { (*RequirementDescriptor).Flags |= CM_RESOURCE_MEMORY_CACHEABLE as u16; }
                            2 => { (*RequirementDescriptor).Flags |= CM_RESOURCE_MEMORY_COMBINEDWRITE as u16; }
                            3 => { (*RequirementDescriptor).Flags |= CM_RESOURCE_MEMORY_PREFETCHABLE as u16; }
                            _ => {}
                        }
                        (*RequirementDescriptor).u.Memory.MinimumAddress.QuadPart = addr16_data.Address.Minimum as i64;
                        (*RequirementDescriptor).u.Memory.MaximumAddress.QuadPart = (addr16_data.Address.Maximum as i64).wrapping_add(addr16_data.Address.AddressLength as i64).wrapping_sub(1);
                        (*RequirementDescriptor).u.Memory.Length = addr16_data.Address.AddressLength as ULONG;
                    }
                    RequirementDescriptor = RequirementDescriptor.offset(1);
                }
            }
            12 => {
                let addr32_data = &mut (*resource).Data.Address32;
                if addr32_data.ProducerConsumer != ACPI_PRODUCER {
                    (*RequirementDescriptor).Option = if CurrentRes != 0 { 0 } else { IO_RESOURCE_PREFERRED } as u8;
                    if addr32_data.ResourceType == ACPI_BUS_NUMBER_RANGE {
                        (*RequirementDescriptor).Type = CmResourceTypeBusNumber as u8;
                        (*RequirementDescriptor).ShareDisposition = CmResourceShareShared as u8;
                        (*RequirementDescriptor).Flags = 0;
                        (*RequirementDescriptor).u.BusNumber.MinBusNumber = addr32_data.Address.Minimum as ULONG;
                        (*RequirementDescriptor).u.BusNumber.MaxBusNumber = (addr32_data.Address.Maximum as ULONG).wrapping_add(addr32_data.Address.AddressLength as ULONG).wrapping_sub(1);
                        (*RequirementDescriptor).u.BusNumber.Length = addr32_data.Address.AddressLength as ULONG;
                    } else if addr32_data.ResourceType == ACPI_IO_RANGE {
                        (*RequirementDescriptor).Type = CmResourceTypePort as u8;
                        (*RequirementDescriptor).ShareDisposition = CmResourceShareDeviceExclusive as u8;
                        (*RequirementDescriptor).Flags = CM_RESOURCE_PORT_IO as u16;
                        if addr32_data.Decode == ACPI_POS_DECODE {
                            (*RequirementDescriptor).Flags |= CM_RESOURCE_PORT_POSITIVE_DECODE as u16;
                        }
                        (*RequirementDescriptor).u.Port.MinimumAddress.QuadPart = addr32_data.Address.Minimum as i64;
                        (*RequirementDescriptor).u.Port.MaximumAddress.QuadPart = (addr32_data.Address.Maximum as u64).wrapping_add(addr32_data.Address.AddressLength as u64).wrapping_sub(1) as i64;
                        (*RequirementDescriptor).u.Port.Length = addr32_data.Address.AddressLength as ULONG;
                    } else {
                        (*RequirementDescriptor).Type = CmResourceTypeMemory as u8;
                        (*RequirementDescriptor).ShareDisposition = CmResourceShareDeviceExclusive as u8;
                        (*RequirementDescriptor).Flags = 0;
                        if addr32_data.Info.Mem.WriteProtect == ACPI_READ_ONLY_MEMORY {
                            (*RequirementDescriptor).Flags |= CM_RESOURCE_MEMORY_READ_ONLY as u16;
                        } else {
                            (*RequirementDescriptor).Flags |= CM_RESOURCE_MEMORY_READ_WRITE as u16;
                        }
                        match addr32_data.Info.Mem.Caching {
                            1 => { (*RequirementDescriptor).Flags |= CM_RESOURCE_MEMORY_CACHEABLE as u16; }
                            2 => { (*RequirementDescriptor).Flags |= CM_RESOURCE_MEMORY_COMBINEDWRITE as u16; }
                            3 => { (*RequirementDescriptor).Flags |= CM_RESOURCE_MEMORY_PREFETCHABLE as u16; }
                            _ => {}
                        }
                        (*RequirementDescriptor).u.Memory.MinimumAddress.QuadPart = addr32_data.Address.Minimum as i64;
                        (*RequirementDescriptor).u.Memory.MaximumAddress.QuadPart = (addr32_data.Address.Maximum as u64).wrapping_add(addr32_data.Address.AddressLength as u64).wrapping_sub(1) as i64;
                        (*RequirementDescriptor).u.Memory.Length = addr32_data.Address.AddressLength as ULONG;
                    }
                    RequirementDescriptor = RequirementDescriptor.offset(1);
                }
            }
            13 => {
                let addr64_data = &mut (*resource).Data.Address64;
                if addr64_data.ProducerConsumer != ACPI_PRODUCER {
                    (*RequirementDescriptor).Option = if CurrentRes != 0 { 0 } else { IO_RESOURCE_PREFERRED } as u8;
                    if addr64_data.ResourceType == ACPI_BUS_NUMBER_RANGE {
                        (*RequirementDescriptor).Type = CmResourceTypeBusNumber as u8;
                        (*RequirementDescriptor).ShareDisposition = CmResourceShareShared as u8;
                        (*RequirementDescriptor).Flags = 0;
                        (*RequirementDescriptor).u.BusNumber.MinBusNumber = addr64_data.Address.Minimum as ULONG;
                        (*RequirementDescriptor).u.BusNumber.MaxBusNumber = (addr64_data.Address.Maximum as u64).wrapping_add(addr64_data.Address.AddressLength as u64).wrapping_sub(1) as ULONG;
                        (*RequirementDescriptor).u.BusNumber.Length = addr64_data.Address.AddressLength as ULONG;
                    } else if addr64_data.ResourceType == ACPI_IO_RANGE {
                        (*RequirementDescriptor).Type = CmResourceTypePort as u8;
                        (*RequirementDescriptor).ShareDisposition = CmResourceShareDeviceExclusive as u8;
                        (*RequirementDescriptor).Flags = CM_RESOURCE_PORT_IO as u16;
                        if addr64_data.Decode == ACPI_POS_DECODE {
                            (*RequirementDescriptor).Flags |= CM_RESOURCE_PORT_POSITIVE_DECODE as u16;
                        }
                        (*RequirementDescriptor).u.Port.MinimumAddress.QuadPart = addr64_data.Address.Minimum as i64;
                        (*RequirementDescriptor).u.Port.MaximumAddress.QuadPart = (addr64_data.Address.Maximum as u64).wrapping_add(addr64_data.Address.AddressLength as u64).wrapping_sub(1) as i64;
                        (*RequirementDescriptor).u.Port.Length = addr64_data.Address.AddressLength as ULONG;
                    } else {
                        (*RequirementDescriptor).Type = CmResourceTypeMemory as u8;
                        (*RequirementDescriptor).ShareDisposition = CmResourceShareDeviceExclusive as u8;
                        (*RequirementDescriptor).Flags = 0;
                        if addr64_data.Info.Mem.WriteProtect == ACPI_READ_ONLY_MEMORY {
                            (*RequirementDescriptor).Flags |= CM_RESOURCE_MEMORY_READ_ONLY as u16;
                        } else {
                            (*RequirementDescriptor).Flags |= CM_RESOURCE_MEMORY_READ_WRITE as u16;
                        }
                        match addr64_data.Info.Mem.Caching {
                            1 => { (*RequirementDescriptor).Flags |= CM_RESOURCE_MEMORY_CACHEABLE as u16; }
                            2 => { (*RequirementDescriptor).Flags |= CM_RESOURCE_MEMORY_COMBINEDWRITE as u16; }
                            3 => { (*RequirementDescriptor).Flags |= CM_RESOURCE_MEMORY_PREFETCHABLE as u16; }
                            _ => {}
                        }
                        (*RequirementDescriptor).u.Memory.MinimumAddress.QuadPart = addr64_data.Address.Minimum as i64;
                        (*RequirementDescriptor).u.Memory.MaximumAddress.QuadPart = (addr64_data.Address.Maximum as u64).wrapping_add(addr64_data.Address.AddressLength as u64).wrapping_sub(1) as i64;
                        (*RequirementDescriptor).u.Memory.Length = addr64_data.Address.AddressLength as ULONG;
                    }
                    RequirementDescriptor = RequirementDescriptor.offset(1);
                }
            }
            14 => {
                let addr64_data = &mut (*resource).Data.ExtAddress64;
                if addr64_data.ProducerConsumer != ACPI_PRODUCER {
                    (*RequirementDescriptor).Option = if CurrentRes != 0 { 0 } else { IO_RESOURCE_PREFERRED } as u8;
                    if addr64_data.ResourceType == ACPI_BUS_NUMBER_RANGE {
                        (*RequirementDescriptor).Type = CmResourceTypeBusNumber as u8;
                        (*RequirementDescriptor).ShareDisposition = CmResourceShareShared as u8;
                        (*RequirementDescriptor).Flags = 0;
                        (*RequirementDescriptor).u.BusNumber.MinBusNumber = addr64_data.Address.Minimum as ULONG;
                        (*RequirementDescriptor).u.BusNumber.MaxBusNumber = (addr64_data.Address.Maximum as u64).wrapping_add(addr64_data.Address.AddressLength as u64).wrapping_sub(1) as ULONG;
                        (*RequirementDescriptor).u.BusNumber.Length = addr64_data.Address.AddressLength as ULONG;
                    } else if addr64_data.ResourceType == ACPI_IO_RANGE {
                        (*RequirementDescriptor).Type = CmResourceTypePort as u8;
                        (*RequirementDescriptor).ShareDisposition = CmResourceShareDeviceExclusive as u8;
                        (*RequirementDescriptor).Flags = CM_RESOURCE_PORT_IO as u16;
                        if addr64_data.Decode == ACPI_POS_DECODE {
                            (*RequirementDescriptor).Flags |= CM_RESOURCE_PORT_POSITIVE_DECODE as u16;
                        }
                        (*RequirementDescriptor).u.Port.MinimumAddress.QuadPart = addr64_data.Address.Minimum as i64;
                        (*RequirementDescriptor).u.Port.MaximumAddress.QuadPart = (addr64_data.Address.Maximum as u64).wrapping_add(addr64_data.Address.AddressLength as u64).wrapping_sub(1) as i64;
                        (*RequirementDescriptor).u.Port.Length = addr64_data.Address.AddressLength as ULONG;
                    } else {
                        (*RequirementDescriptor).Type = CmResourceTypeMemory as u8;
                        (*RequirementDescriptor).ShareDisposition = CmResourceShareDeviceExclusive as u8;
                        (*RequirementDescriptor).Flags = 0;
                        if addr64_data.Info.Mem.WriteProtect == ACPI_READ_ONLY_MEMORY {
                            (*RequirementDescriptor).Flags |= CM_RESOURCE_MEMORY_READ_ONLY as u16;
                        } else {
                            (*RequirementDescriptor).Flags |= CM_RESOURCE_MEMORY_READ_WRITE as u16;
                        }
                        match addr64_data.Info.Mem.Caching {
                            1 => { (*RequirementDescriptor).Flags |= CM_RESOURCE_MEMORY_CACHEABLE as u16; }
                            2 => { (*RequirementDescriptor).Flags |= CM_RESOURCE_MEMORY_COMBINEDWRITE as u16; }
                            3 => { (*RequirementDescriptor).Flags |= CM_RESOURCE_MEMORY_PREFETCHABLE as u16; }
                            _ => {}
                        }
                        (*RequirementDescriptor).u.Memory.MinimumAddress.QuadPart = addr64_data.Address.Minimum as i64;
                        (*RequirementDescriptor).u.Memory.MaximumAddress.QuadPart = (addr64_data.Address.Maximum as u64).wrapping_add(addr64_data.Address.AddressLength as u64).wrapping_sub(1) as i64;
                        (*RequirementDescriptor).u.Memory.Length = addr64_data.Address.AddressLength as ULONG;
                    }
                    RequirementDescriptor = RequirementDescriptor.offset(1);
                }
            }
            8 => {
                let mem24_data = &mut (*resource).Data.Memory24;
                (*RequirementDescriptor).Option = if CurrentRes != 0 { 0 } else { IO_RESOURCE_PREFERRED } as u8;
                (*RequirementDescriptor).Type = CmResourceTypeMemory as u8;
                (*RequirementDescriptor).ShareDisposition = CmResourceShareDeviceExclusive as u8;
                (*RequirementDescriptor).Flags = CM_RESOURCE_MEMORY_24 as u16;
                if mem24_data.WriteProtect == ACPI_READ_ONLY_MEMORY {
                    (*RequirementDescriptor).Flags |= CM_RESOURCE_MEMORY_READ_ONLY as u16;
                } else {
                    (*RequirementDescriptor).Flags |= CM_RESOURCE_MEMORY_READ_WRITE as u16;
                }
                (*RequirementDescriptor).u.Memory.MinimumAddress.QuadPart = mem24_data.Minimum as i64;
                (*RequirementDescriptor).u.Memory.MaximumAddress.QuadPart = (mem24_data.Maximum as i64).wrapping_add(mem24_data.AddressLength as i64).wrapping_sub(1);
                (*RequirementDescriptor).u.Memory.Length = mem24_data.AddressLength as ULONG;
                RequirementDescriptor = RequirementDescriptor.offset(1);
            }
            9 => {
                let mem32_data = &mut (*resource).Data.Memory32;
                (*RequirementDescriptor).Option = if CurrentRes != 0 { 0 } else { IO_RESOURCE_PREFERRED } as u8;
                (*RequirementDescriptor).Type = CmResourceTypeMemory as u8;
                (*RequirementDescriptor).ShareDisposition = CmResourceShareDeviceExclusive as u8;
                (*RequirementDescriptor).Flags = 0;
                if mem32_data.WriteProtect == ACPI_READ_ONLY_MEMORY {
                    (*RequirementDescriptor).Flags |= CM_RESOURCE_MEMORY_READ_ONLY as u16;
                } else {
                    (*RequirementDescriptor).Flags |= CM_RESOURCE_MEMORY_READ_WRITE as u16;
                }
                (*RequirementDescriptor).u.Memory.MinimumAddress.QuadPart = mem32_data.Minimum as i64;
                (*RequirementDescriptor).u.Memory.MaximumAddress.QuadPart = (mem32_data.Maximum as u64).wrapping_add(mem32_data.AddressLength as u64).wrapping_sub(1) as i64;
                (*RequirementDescriptor).u.Memory.Length = mem32_data.AddressLength as ULONG;
                RequirementDescriptor = RequirementDescriptor.offset(1);
            }
            10 => {
                let fixedmem32_data = &mut (*resource).Data.FixedMemory32;
                (*RequirementDescriptor).Option = if CurrentRes != 0 { 0 } else { IO_RESOURCE_PREFERRED } as u8;
                (*RequirementDescriptor).Type = CmResourceTypeMemory as u8;
                (*RequirementDescriptor).ShareDisposition = CmResourceShareDeviceExclusive as u8;
                (*RequirementDescriptor).Flags = 0;
                if fixedmem32_data.WriteProtect == ACPI_READ_ONLY_MEMORY {
                    (*RequirementDescriptor).Flags |= CM_RESOURCE_MEMORY_READ_ONLY as u16;
                } else {
                    (*RequirementDescriptor).Flags |= CM_RESOURCE_MEMORY_READ_WRITE as u16;
                }
                (*RequirementDescriptor).u.Memory.MinimumAddress.QuadPart = fixedmem32_data.Address as i64;
                (*RequirementDescriptor).u.Memory.MaximumAddress.QuadPart = (fixedmem32_data.Address as u64).wrapping_add(fixedmem32_data.AddressLength as u64).wrapping_sub(1) as i64;
                (*RequirementDescriptor).u.Memory.Length = fixedmem32_data.AddressLength as ULONG;
                RequirementDescriptor = RequirementDescriptor.offset(1);
            }
            _ => {}
        }
        resource = (resource as *mut u8).offset((*resource).Length as isize) as *mut ACPI_RESOURCE;
    }

    ExFreePoolWithTag(Buffer.Pointer, u32::from_be_bytes(*b"BpcA") as ULONG);
    (*Irp).IoStatus.Information = RequirementsList as ULONG_PTR;
    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn Bus_PDO_QueryDeviceRelations(
    DeviceData: PPDO_DEVICE_DATA,
    Irp: PIRP,
) -> NTSTATUS {
    let stack: PIO_STACK_LOCATION;
    let _deviceRelations: PDEVICE_RELATIONS;
    let status: NTSTATUS;

    stack = IoGetCurrentIrpStackLocation(Irp);

    match (*stack).Parameters.QueryDeviceRelations.Type as u32 {
        4 => {
            let deviceRelations = (*Irp).IoStatus.Information as PDEVICE_RELATIONS;
            if !deviceRelations.is_null() {
                ASSERTMSG(
                    b"Someone above is handling TargetDeviceRelation\n\0".as_ptr() as PCHAR,
                    deviceRelations.is_null(),
                );
            }

            let deviceRelations = ExAllocatePoolWithTag(
                PagedPool,
                core::mem::size_of::<DEVICE_RELATIONS>(),
                u32::from_be_bytes(*b"IpcA") as ULONG,
            ) as PDEVICE_RELATIONS;

            if deviceRelations.is_null() {
                status = STATUS_INSUFFICIENT_RESOURCES;
            } else {
                (*deviceRelations).Count = 1;
                (*deviceRelations).Objects[0] = (*DeviceData).Common.Self_;
                ObReferenceObject((*DeviceData).Common.Self_ as *mut core::ffi::c_void);
                status = STATUS_SUCCESS;
                (*Irp).IoStatus.Information = deviceRelations as ULONG_PTR;
            }
        }
        0 | 1 | 3 | _ => {
            status = (*Irp).IoStatus.__bindgen_anon_1.Status;
        }
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn Bus_PDO_QueryBusInformation(
    _DeviceData: PPDO_DEVICE_DATA,
    Irp: PIRP,
) -> NTSTATUS {
    let busInfo = ExAllocatePoolWithTag(
        PagedPool,
        core::mem::size_of::<PNP_BUS_INFORMATION>(),
        u32::from_be_bytes(*b"IpcA") as ULONG,
    ) as PPNP_BUS_INFORMATION;

    if busInfo.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*busInfo).BusTypeGuid = GUID_ACPI_INTERFACE_STANDARD;
    (*busInfo).LegacyBusType = InternalPowerBus;
    (*busInfo).BusNumber = 0;

    (*Irp).IoStatus.Information = busInfo as ULONG_PTR;

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn Bus_GetDeviceCapabilities(
    DeviceObject: PDEVICE_OBJECT,
    DeviceCapabilities: PDEVICE_CAPABILITIES,
) -> NTSTATUS {
    let mut ioStatus: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut pnpEvent: KEVENT = core::mem::zeroed();
    let mut status: NTSTATUS;
    let targetObject: PDEVICE_OBJECT;
    let irpStack: PIO_STACK_LOCATION;
    let pnpIrp: PIRP;

    core::ptr::write_bytes(DeviceCapabilities, 0, 1);
    (*DeviceCapabilities).Size = core::mem::size_of::<DEVICE_CAPABILITIES>() as u16;
    (*DeviceCapabilities).Version = 1;
    (*DeviceCapabilities).Address = !0u32;
    (*DeviceCapabilities).UINumber = !0u32;

    KeInitializeEvent(&mut pnpEvent, _EVENT_TYPE_NotificationEvent, 0u8);

    targetObject = IoGetAttachedDeviceReference(DeviceObject);

    pnpIrp = IoBuildSynchronousFsdRequest(
        IRP_MJ_PNP as u32,
        targetObject,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
        &mut pnpEvent,
        &mut ioStatus,
    );

    if pnpIrp.is_null() {
        status = STATUS_INSUFFICIENT_RESOURCES;
    } else {
        (*pnpIrp).IoStatus.__bindgen_anon_1.Status = STATUS_NOT_SUPPORTED;

        irpStack = IoGetNextIrpStackLocation(pnpIrp);

        core::ptr::write_bytes(irpStack, 0, 1);
        (*irpStack).MajorFunction = IRP_MJ_PNP as u8;
        (*irpStack).MinorFunction = IRP_MN_QUERY_CAPABILITIES as u8;
        (*irpStack).Parameters.DeviceCapabilities.Capabilities = DeviceCapabilities;

        status = IoCallDriver(targetObject, pnpIrp);
        if status == STATUS_PENDING {
            KeWaitForSingleObject(
                &mut pnpEvent as *mut _ as *mut core::ffi::c_void,
                _KWAIT_REASON_Executive,
                0i8,
                0u8,
                core::ptr::null_mut(),
            );
            status = ioStatus.__bindgen_anon_1.Status;
        }
    }

    ObDereferenceObject(targetObject as *mut _);
    status
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// #ifndef UNIT_TEST
// #include "precomp.h"
//
// #include <initguid.h>
// #include <poclass.h>
//
// #define NDEBUG
// #include <debug.h>
//
// #ifdef ALLOC_PRAGMA
// #pragma alloc_text (PAGE, Bus_PDO_PnP)
// #pragma alloc_text (PAGE, Bus_PDO_QueryDeviceCaps)
// #pragma alloc_text (PAGE, Bus_PDO_QueryDeviceId)
// #pragma alloc_text (PAGE, Bus_PDO_QueryDeviceText)
// #pragma alloc_text (PAGE, Bus_PDO_QueryResources)
// #pragma alloc_text (PAGE, Bus_PDO_QueryResourceRequirements)
// #pragma alloc_text (PAGE, Bus_PDO_QueryDeviceRelations)
// #pragma alloc_text (PAGE, Bus_PDO_QueryBusInformation)
// #pragma alloc_text (PAGE, Bus_GetDeviceCapabilities)
// #endif
//
// NTSTATUS
// Bus_PDO_PnP (
//      PDEVICE_OBJECT       DeviceObject,
//      PIRP                 Irp,
//      PIO_STACK_LOCATION   IrpStack,
//      PPDO_DEVICE_DATA     DeviceData
//     )
// {
//     NTSTATUS                status;
//     POWER_STATE             state;
//     struct acpi_device      *device = NULL;
//
//     PAGED_CODE ();
//
//     if (DeviceData->AcpiHandle)
//         acpi_bus_get_device(DeviceData->AcpiHandle, &device);
//
//     //
//     // NB: Because we are a bus enumerator, we have no one to whom we could
//     // defer these irps.  Therefore we do not pass them down but merely
//     // return them.
//     //
//
//     switch (IrpStack->MinorFunction) {
//
//     case IRP_MN_START_DEVICE:
//         //
//         // Here we do what ever initialization and ``turning on'' that is
//         // required to allow others to access this device.
//         // Power up the device.
//         //
//         if (DeviceData->AcpiHandle && acpi_bus_power_manageable(DeviceData->AcpiHandle) &&
//             !ACPI_SUCCESS(acpi_bus_set_power(DeviceData->AcpiHandle, ACPI_STATE_D0)))
//         {
//             DPRINT1("Device %x failed to start!\n", DeviceData->AcpiHandle);
//             status = STATUS_UNSUCCESSFUL;
//             break;
//         }
//
//         DeviceData->InterfaceName.Length = 0;
//         status = STATUS_SUCCESS;
//
//         if (!device)
//         {
//             status = IoRegisterDeviceInterface(DeviceData->Common.Self,
//                                                &GUID_DEVICE_SYS_BUTTON,
//                                                NULL,
//                                                &DeviceData->InterfaceName);
//         }
//         else if (device->flags.hardware_id &&
//                  strstr(device->pnp.hardware_id, ACPI_THERMAL_HID))
//         {
//             status = IoRegisterDeviceInterface(DeviceData->Common.Self,
//                                                &GUID_DEVICE_THERMAL_ZONE,
//                                                NULL,
//                                                &DeviceData->InterfaceName);
//         }
//         else if (device->flags.hardware_id &&
//                  strstr(device->pnp.hardware_id, ACPI_FAN_HID))
//         {
//             status = IoRegisterDeviceInterface(DeviceData->Common.Self,
//                                                &GUID_DEVICE_FAN,
//                                                NULL,
//                                                &DeviceData->InterfaceName);
//         }
//         else if (device->flags.hardware_id &&
//                  strstr(device->pnp.hardware_id, ACPI_BUTTON_HID_LID))
//         {
//             status = IoRegisterDeviceInterface(DeviceData->Common.Self,
//                                                &GUID_DEVICE_LID,
//                                                NULL,
//                                                &DeviceData->InterfaceName);
//         }
//         else if (device->flags.hardware_id &&
//                  strstr(device->pnp.hardware_id, ACPI_PROCESSOR_HID))
//         {
//             status = IoRegisterDeviceInterface(DeviceData->Common.Self,
//                                                &GUID_DEVICE_PROCESSOR,
//                                                NULL,
//                                                &DeviceData->InterfaceName);
//         }
//
//         /* Failure to register an interface is not a fatal failure so don't return a failure status */
//         if (NT_SUCCESS(status) && DeviceData->InterfaceName.Length != 0)
//             IoSetDeviceInterfaceState(&DeviceData->InterfaceName, TRUE);
//
//         state.DeviceState = PowerDeviceD0;
//         PoSetPowerState(DeviceData->Common.Self, DevicePowerState, state);
//         DeviceData->Common.DevicePowerState = PowerDeviceD0;
//         SET_NEW_PNP_STATE(DeviceData->Common, Started);
//         status = STATUS_SUCCESS;
//         break;
//
//     case IRP_MN_STOP_DEVICE:
//
//         if (DeviceData->InterfaceName.Length != 0)
//             IoSetDeviceInterfaceState(&DeviceData->InterfaceName, FALSE);
//
//         //
//         // Here we shut down the device and give up and unmap any resources
//         // we acquired for the device.
//         //
//         if (DeviceData->AcpiHandle && acpi_bus_power_manageable(DeviceData->AcpiHandle) &&
//             !ACPI_SUCCESS(acpi_bus_set_power(DeviceData->AcpiHandle, ACPI_STATE_D3)))
//         {
//             DPRINT1("Device %x failed to stop!\n", DeviceData->AcpiHandle);
//             status = STATUS_UNSUCCESSFUL;
//             break;
//         }
//
//         state.DeviceState = PowerDeviceD3;
//         PoSetPowerState(DeviceData->Common.Self, DevicePowerState, state);
//         DeviceData->Common.DevicePowerState = PowerDeviceD3;
//         SET_NEW_PNP_STATE(DeviceData->Common, Stopped);
//         status = STATUS_SUCCESS;
//         break;
//
//
//     case IRP_MN_QUERY_STOP_DEVICE:
//
//         //
//         // No reason here why we can't stop the device.
//         // If there were a reason we should speak now, because answering success
//         // here may result in a stop device irp.
//         //
//
//         SET_NEW_PNP_STATE(DeviceData->Common, StopPending);
//         status = STATUS_SUCCESS;
//         break;
//
//     case IRP_MN_CANCEL_STOP_DEVICE:
//
//         //
//         // The stop was canceled.  Whatever state we set, or resources we put
//         // on hold in anticipation of the forthcoming STOP device IRP should be
//         // put back to normal.  Someone, in the long list of concerned parties,
//         // has failed the stop device query.
//         //
//
//         //
//         // First check to see whether you have received cancel-stop
//         // without first receiving a query-stop. This could happen if someone
//         // above us fails a query-stop and passes down the subsequent
//         // cancel-stop.
//         //
//
//         if (StopPending == DeviceData->Common.DevicePnPState)
//         {
//             //
//             // We did receive a query-stop, so restore.
//             //
//             RESTORE_PREVIOUS_PNP_STATE(DeviceData->Common);
//         }
//         status = STATUS_SUCCESS;// We must not fail this IRP.
//         break;
//
//     case IRP_MN_REMOVE_DEVICE:
//         //
//         // We handle REMOVE_DEVICE just like STOP_DEVICE. This is because
//         // the device is still physically present (or at least we don't know any better)
//         // so we have to retain the PDO after stopping and removing power from it.
//         //
//         if (DeviceData->InterfaceName.Length != 0)
//             IoSetDeviceInterfaceState(&DeviceData->InterfaceName, FALSE);
//
//         if (DeviceData->AcpiHandle && acpi_bus_power_manageable(DeviceData->AcpiHandle) &&
//             !ACPI_SUCCESS(acpi_bus_set_power(DeviceData->AcpiHandle, ACPI_STATE_D3)))
//         {
//             DPRINT1("Device %x failed to enter D3!\n", DeviceData->AcpiHandle);
//             state.DeviceState = PowerDeviceD3;
//             PoSetPowerState(DeviceData->Common.Self, DevicePowerState, state);
//             DeviceData->Common.DevicePowerState = PowerDeviceD3;
//         }
//
//         SET_NEW_PNP_STATE(DeviceData->Common, Stopped);
//         status = STATUS_SUCCESS;
//         break;
//
//     case IRP_MN_QUERY_REMOVE_DEVICE:
//         SET_NEW_PNP_STATE(DeviceData->Common, RemovalPending);
//         status = STATUS_SUCCESS;
//         break;
//
//     case IRP_MN_CANCEL_REMOVE_DEVICE:
//         if (RemovalPending == DeviceData->Common.DevicePnPState)
//         {
//             RESTORE_PREVIOUS_PNP_STATE(DeviceData->Common);
//         }
//         status = STATUS_SUCCESS;
//         break;
//
//     case IRP_MN_QUERY_CAPABILITIES:
//
//         //
//         // Return the capabilities of a device, such as whether the device
//         // can be locked or ejected..etc
//         //
//
//         status = Bus_PDO_QueryDeviceCaps(DeviceData, Irp);
//
//         break;
//
//     case IRP_MN_QUERY_ID:
//
//         // Query the IDs of the device
//         status = Bus_PDO_QueryDeviceId(DeviceData, Irp);
//
//         break;
//
//     case IRP_MN_QUERY_DEVICE_RELATIONS:
//
//         DPRINT("\tQueryDeviceRelation Type: %s\n",DbgDeviceRelationString(\
//                     IrpStack->Parameters.QueryDeviceRelations.Type));
//
//         status = Bus_PDO_QueryDeviceRelations(DeviceData, Irp);
//
//         break;
//
//     case IRP_MN_QUERY_DEVICE_TEXT:
//
//         status = Bus_PDO_QueryDeviceText(DeviceData, Irp);
//
//         break;
//
//     case IRP_MN_QUERY_RESOURCES:
//
//         status = Bus_PDO_QueryResources(DeviceData, Irp);
//
//         break;
//
//     case IRP_MN_QUERY_RESOURCE_REQUIREMENTS:
//
//         status = Bus_PDO_QueryResourceRequirements(DeviceData, Irp);
//
//         break;
//
//     case IRP_MN_QUERY_BUS_INFORMATION:
//
//         status = Bus_PDO_QueryBusInformation(DeviceData, Irp);
//
//         break;
//
//     case IRP_MN_QUERY_INTERFACE:
//
//         status = Bus_PDO_QueryInterface(DeviceData, Irp);
//
//         break;
//
//
//     case IRP_MN_FILTER_RESOURCE_REQUIREMENTS:
//
//         //
//         // OPTIONAL for bus drivers.
//         // The PnP Manager sends this IRP to a device
//         // stack so filter and function drivers can adjust the
//         // resources required by the device, if appropriate.
//         //
//
//         //break;
//
//     //case IRP_MN_QUERY_PNP_DEVICE_STATE:
//
//         //
//         // OPTIONAL for bus drivers.
//         // The PnP Manager sends this IRP after the drivers for
//         // a device return success from the IRP_MN_START_DEVICE
//         // request. The PnP Manager also sends this IRP when a
//         // driver for the device calls IoInvalidateDeviceState.
//         //
//
//         // break;
//
//     //case IRP_MN_READ_CONFIG:
//     //case IRP_MN_WRITE_CONFIG:
//
//         //
//         // Bus drivers for buses with configuration space must handle
//         // this request for their child devices. Our devices don't
//         // have a config space.
//         //
//
//         // break;
//
//     //case IRP_MN_SET_LOCK:
//
//         // break;
//
//     default:
//
//         //
//         // For PnP requests to the PDO that we do not understand we should
//         // return the IRP WITHOUT setting the status or information fields.
//         // These fields may have already been set by a filter (eg acpi).
//         status = Irp->IoStatus.Status;
//
//         break;
//     }
//
//     Irp->IoStatus.Status = status;
//     IoCompleteRequest (Irp, IO_NO_INCREMENT);
//
//     return status;
// }
//
// NTSTATUS
// Bus_PDO_QueryDeviceCaps(
//      PPDO_DEVICE_DATA     DeviceData,
//       PIRP   Irp )
// {
//
//     PIO_STACK_LOCATION      stack;
//     PDEVICE_CAPABILITIES    deviceCapabilities;
//     struct acpi_device *device = NULL;
//     ULONG i;
//
//     PAGED_CODE ();
//
//     if (DeviceData->AcpiHandle)
//         acpi_bus_get_device(DeviceData->AcpiHandle, &device);
//
//     stack = IoGetCurrentIrpStackLocation (Irp);
//
//     //
//     // Get the packet.
//     //
//     deviceCapabilities=stack->Parameters.DeviceCapabilities.Capabilities;
//
//     //
//     // Set the capabilities.
//     //
//
//     if (deviceCapabilities->Version != 1 ||
//             deviceCapabilities->Size < sizeof(DEVICE_CAPABILITIES))
//     {
//        return STATUS_UNSUCCESSFUL;
//     }
//
//     deviceCapabilities->D1Latency = 0;
//     deviceCapabilities->D2Latency = 0;
//     deviceCapabilities->D3Latency = 0;
//
//     deviceCapabilities->DeviceState[PowerSystemWorking] = PowerDeviceD0;
//     deviceCapabilities->DeviceState[PowerSystemSleeping1] = PowerDeviceD3;
//     deviceCapabilities->DeviceState[PowerSystemSleeping2] = PowerDeviceD3;
//     deviceCapabilities->DeviceState[PowerSystemSleeping3] = PowerDeviceD3;
//
//     for (i = 0; i < ACPI_D_STATE_COUNT && device; i++)
//     {
//         if (!device->power.states[i].flags.valid)
//             continue;
//
//         switch (i)
//         {
//            case ACPI_STATE_D0:
//               deviceCapabilities->DeviceState[PowerSystemWorking] = PowerDeviceD0;
//               break;
//
//            case ACPI_STATE_D1:
//               deviceCapabilities->DeviceState[PowerSystemSleeping1] = PowerDeviceD1;
//               deviceCapabilities->D1Latency = device->power.states[i].latency;
//               break;
//
//            case ACPI_STATE_D2:
//               deviceCapabilities->DeviceState[PowerSystemSleeping2] = PowerDeviceD2;
//               deviceCapabilities->D2Latency = device->power.states[i].latency;
//               break;
//
//            case ACPI_STATE_D3:
//               deviceCapabilities->DeviceState[PowerSystemSleeping3] = PowerDeviceD3;
//               deviceCapabilities->D3Latency = device->power.states[i].latency;
//               break;
//         }
//     }
//
//     // We can wake the system from D1
//     deviceCapabilities->DeviceWake = PowerDeviceD1;
//
//
//     deviceCapabilities->DeviceD1 =
//         (deviceCapabilities->DeviceState[PowerSystemSleeping1] == PowerDeviceD1) ? TRUE : FALSE;
//     deviceCapabilities->DeviceD2 =
//         (deviceCapabilities->DeviceState[PowerSystemSleeping2] == PowerDeviceD2) ? TRUE : FALSE;
//
//     deviceCapabilities->WakeFromD0 = FALSE;
//     deviceCapabilities->WakeFromD1 = TRUE; //Yes we can
//     deviceCapabilities->WakeFromD2 = FALSE;
//     deviceCapabilities->WakeFromD3 = FALSE;
//
//     if (device)
//     {
//        deviceCapabilities->LockSupported = device->flags.lockable;
//        deviceCapabilities->EjectSupported = device->flags.ejectable;
//        deviceCapabilities->HardwareDisabled = !device->status.enabled && !device->status.functional;
//        deviceCapabilities->Removable = device->flags.removable;
//        deviceCapabilities->SurpriseRemovalOK = device->flags.surprise_removal_ok;
//        deviceCapabilities->UniqueID = device->flags.unique_id;
//        deviceCapabilities->NoDisplayInUI = !device->status.show_in_ui;
//        deviceCapabilities->Address = device->pnp.bus_address;
//     }
//
//     if (DeviceData->DockDevice)
//         deviceCapabilities->DockDevice = TRUE;
//     DPRINT("DockDevice: %u\n", deviceCapabilities->DockDevice);
//
//     if (!device ||
//         (device->flags.hardware_id &&
//          (strstr(device->pnp.hardware_id, ACPI_BUTTON_HID_LID) ||
//           strstr(device->pnp.hardware_id, ACPI_THERMAL_HID) ||
//           strstr(device->pnp.hardware_id, ACPI_PROCESSOR_HID))))
//     {
//         /* Allow ACPI to control the device if it is a lid button,
//          * a thermal zone, a processor, or a fixed feature button */
//         deviceCapabilities->RawDeviceOK = TRUE;
//     }
//
//     deviceCapabilities->SilentInstall = FALSE;
//     deviceCapabilities->UINumber = (ULONG)-1;
//
//     return STATUS_SUCCESS;
//
// }
//
// NTSTATUS
// Bus_PDO_QueryDeviceId(
//      PPDO_DEVICE_DATA     DeviceData,
//       PIRP   Irp )
// {
//     PIO_STACK_LOCATION      stack;
//     PWCHAR                  buffer, src;
//     WCHAR                   temp[256];
//     ULONG                   length, i;
//     NTSTATUS                status = STATUS_SUCCESS;
//     struct acpi_device *Device;
//
//     PAGED_CODE ();
//
//     stack = IoGetCurrentIrpStackLocation (Irp);
//
//     switch (stack->Parameters.QueryId.IdType) {
//
//     case BusQueryDeviceID:
//
//         /* This is a REG_SZ value */
//
//         if (DeviceData->AcpiHandle)
//         {
//             acpi_bus_get_device(DeviceData->AcpiHandle, &Device);
//
//             if (strcmp(Device->pnp.hardware_id, "Processor") == 0)
//             {
//                 length = wcslen(ProcessorIdString);
//                 wcscpy(temp, ProcessorIdString);
//             }
//             else
//             {
//                 length = swprintf(temp,
//                                   L"ACPI\\%hs",
//                                   Device->pnp.hardware_id);
//             }
//         }
//         else
//         {
//             /* We know it's a fixed feature button because
//              * these are direct children of the ACPI root device
//              * and therefore have no handle
//              */
//             length = swprintf(temp,
//                               L"ACPI\\FixedButton");
//         }
//
//         temp[length++] = UNICODE_NULL;
//
//         NT_ASSERT!(length * sizeof(WCHAR) <= sizeof(temp));
//
//         buffer = ExAllocatePoolWithTag(PagedPool, length * sizeof(WCHAR), u32::from_be_bytes(*b"IpcA"));
//
//         if (!buffer) {
//            status = STATUS_INSUFFICIENT_RESOURCES;
//            break;
//         }
//
//         RtlCopyMemory (buffer, temp, length * sizeof(WCHAR));
//         Irp->IoStatus.Information = (ULONG_PTR) buffer;
//         DPRINT("BusQueryDeviceID: %ls\n",buffer);
//         break;
//
//     case BusQueryInstanceID:
//
//         /* This is a REG_SZ value */
//
//         /* See comment in BusQueryDeviceID case */
//         if(DeviceData->AcpiHandle)
//         {
//            acpi_bus_get_device(DeviceData->AcpiHandle, &Device);
//
//            if (Device->flags.unique_id)
//               length = swprintf(temp,
//                                 L"%hs",
//                                 Device->pnp.unique_id);
//            else
//               /* FIXME: Generate unique id! */
//               length = swprintf(temp, L"%ls", L"0");
//         }
//         else
//         {
//            /* FIXME: Generate unique id! */
//            length = swprintf(temp, L"%ls", L"0");
//         }
//
//         temp[length++] = UNICODE_NULL;
//
//         NT_ASSERT!(length * sizeof(WCHAR) <= sizeof(temp));
//
//         buffer = ExAllocatePoolWithTag(PagedPool, length * sizeof(WCHAR), u32::from_be_bytes(*b"IpcA"));
//         if (!buffer) {
//            status = STATUS_INSUFFICIENT_RESOURCES;
//            break;
//         }
//
//         RtlCopyMemory (buffer, temp, length * sizeof (WCHAR));
//         DPRINT("BusQueryInstanceID: %ls\n",buffer);
//         Irp->IoStatus.Information = (ULONG_PTR) buffer;
//         break;
//
//     case BusQueryHardwareIDs:
//
//         /* This is a REG_MULTI_SZ value */
//         length = 0;
//         status = STATUS_NOT_SUPPORTED;
//
//         /* See comment in BusQueryDeviceID case */
//         if (DeviceData->AcpiHandle)
//         {
//             acpi_bus_get_device(DeviceData->AcpiHandle, &Device);
//
//             if (!Device->flags.hardware_id)
//             {
//                 /* We don't have the ID to satisfy this request */
//                 break;
//             }
//
//             DPRINT("Device name: %s\n", Device->pnp.device_name);
//             DPRINT("Hardware ID: %s\n", Device->pnp.hardware_id);
//
//             if (strcmp(Device->pnp.hardware_id, "Processor") == 0)
//             {
//                 length = ProcessorHardwareIds.Length / sizeof(WCHAR);
//                 src = ProcessorHardwareIds.Buffer;
//             }
//             else
//             {
//                 length += swprintf(&temp[length],
//                                    L"ACPI\\%hs",
//                                    Device->pnp.hardware_id);
//                 temp[length++] = UNICODE_NULL;
//
//                 length += swprintf(&temp[length],
//                                    L"*%hs",
//                                    Device->pnp.hardware_id);
//                 temp[length++] = UNICODE_NULL;
//                 temp[length++] = UNICODE_NULL;
//                 src = temp;
//             }
//         }
//         else
//         {
//             length += swprintf(&temp[length],
//                                L"ACPI\\FixedButton");
//             temp[length++] = UNICODE_NULL;
//
//             length += swprintf(&temp[length],
//                                L"*FixedButton");
//             temp[length++] = UNICODE_NULL;
//             temp[length++] = UNICODE_NULL;
//             src = temp;
//         }
//
//         NT_ASSERT!(length * sizeof(WCHAR) <= sizeof(temp));
//
//         buffer = ExAllocatePoolWithTag(PagedPool, length * sizeof(WCHAR), u32::from_be_bytes(*b"IpcA"));
//
//         if (!buffer) {
//            status = STATUS_INSUFFICIENT_RESOURCES;
//            break;
//         }
//
//         RtlCopyMemory (buffer, src, length * sizeof(WCHAR));
//         Irp->IoStatus.Information = (ULONG_PTR) buffer;
//         DPRINT("BusQueryHardwareIDs: %ls\n",buffer);
//         status = STATUS_SUCCESS;
//         break;
//
//     case BusQueryCompatibleIDs:
//
//         /* This is a REG_MULTI_SZ value */
//         length = 0;
//         status = STATUS_NOT_SUPPORTED;
//
//         /* See comment in BusQueryDeviceID case */
//         if (DeviceData->AcpiHandle)
//         {
//             acpi_bus_get_device(DeviceData->AcpiHandle, &Device);
//
//             if (!Device->flags.hardware_id)
//             {
//                 /* We don't have the ID to satisfy this request */
//                 break;
//             }
//
//             DPRINT("Device name: %s\n", Device->pnp.device_name);
//             DPRINT("Hardware ID: %s\n", Device->pnp.hardware_id);
//
//             if (strcmp(Device->pnp.hardware_id, "Processor") == 0)
//             {
//                 length += swprintf(&temp[length],
//                                    L"ACPI\\%hs",
//                                    Device->pnp.hardware_id);
//                 temp[length++] = UNICODE_NULL;
//
//                 length += swprintf(&temp[length],
//                                    L"*%hs",
//                                    Device->pnp.hardware_id);
//                 temp[length++] = UNICODE_NULL;
//                 temp[length++] = UNICODE_NULL;
//             }
//             else if (Device->flags.compatible_ids)
//             {
//                 for (i = 0; i < Device->pnp.cid_list->Count; i++)
//                 {
//                     length += swprintf(&temp[length],
//                                    L"ACPI\\%hs",
//                                    Device->pnp.cid_list->Ids[i].String);
//                     temp[length++] = UNICODE_NULL;
//
//                     length += swprintf(&temp[length],
//                                    L"*%hs",
//                                    Device->pnp.cid_list->Ids[i].String);
//                     temp[length++] = UNICODE_NULL;
//                 }
//
//                 temp[length++] = UNICODE_NULL;
//             }
//             else
//             {
//                 /* No compatible IDs */
//                 break;
//             }
//
//             NT_ASSERT!(length * sizeof(WCHAR) <= sizeof(temp));
//
//             buffer = ExAllocatePoolWithTag(PagedPool, length * sizeof(WCHAR), u32::from_be_bytes(*b"IpcA"));
//             if (!buffer)
//             {
//                 status = STATUS_INSUFFICIENT_RESOURCES;
//                 break;
//             }
//
//             RtlCopyMemory (buffer, temp, length * sizeof(WCHAR));
//             Irp->IoStatus.Information = (ULONG_PTR) buffer;
//             DPRINT("BusQueryCompatibleIDs: %ls\n",buffer);
//             status = STATUS_SUCCESS;
//         }
//         break;
//
//     default:
//         status = Irp->IoStatus.Status;
//     }
//     return status;
// }
//
// NTSTATUS
// Bus_PDO_QueryDeviceText(
//      PPDO_DEVICE_DATA     DeviceData,
//       PIRP   Irp )
// {
//     PWCHAR  Buffer, Temp;
//     PIO_STACK_LOCATION   stack;
//     NTSTATUS    status = Irp->IoStatus.Status;
//     PAGED_CODE ();
//
//     stack = IoGetCurrentIrpStackLocation (Irp);
//
//     switch (stack->Parameters.QueryDeviceText.DeviceTextType) {
//
//     case DeviceTextDescription:
//
//         if (!Irp->IoStatus.Information) {
//             if (wcsstr (DeviceData->HardwareIDs, L"PNP000") != 0)
//                 Temp = L"Programmable interrupt controller";
//             else if (wcsstr(DeviceData->HardwareIDs, L"PNP010") != 0)
//                 Temp = L"System timer";
//             else if (wcsstr(DeviceData->HardwareIDs, L"PNP020") != 0)
//                 Temp = L"DMA controller";
//             else if (wcsstr(DeviceData->HardwareIDs, L"PNP03") != 0)
//                 Temp = L"Keyboard";
//             else if (wcsstr(DeviceData->HardwareIDs, L"PNP040") != 0)
//                 Temp = L"Parallel port";
//             else if (wcsstr(DeviceData->HardwareIDs, L"PNP05") != 0)
//                 Temp = L"Serial port";
//             else if (wcsstr(DeviceData->HardwareIDs, L"PNP06") != 0)
//                 Temp = L"Disk controller";
//             else if (wcsstr(DeviceData->HardwareIDs, L"PNP07") != 0)
//                 Temp = L"Disk controller";
//             else if (wcsstr(DeviceData->HardwareIDs, L"PNP09") != 0)
//                 Temp = L"Display adapter";
//             else if (wcsstr(DeviceData->HardwareIDs, L"PNP0A0") != 0)
//                 Temp = L"Bus controller";
//             else if (wcsstr(DeviceData->HardwareIDs, L"PNP0E0") != 0)
//                 Temp = L"PCMCIA controller";
//             else if (wcsstr(DeviceData->HardwareIDs, L"PNP0F") != 0)
//                 Temp = L"Mouse device";
//             else if (wcsstr(DeviceData->HardwareIDs, L"PNP8") != 0)
//                 Temp = L"Network adapter";
//             else if (wcsstr(DeviceData->HardwareIDs, L"PNPA0") != 0)
//                 Temp = L"SCSI controller";
//             else if (wcsstr(DeviceData->HardwareIDs, L"PNPB0") != 0)
//                 Temp = L"Multimedia device";
//             else if (wcsstr(DeviceData->HardwareIDs, L"PNPC00") != 0)
//                 Temp = L"Modem";
//             else if (wcsstr(DeviceData->HardwareIDs, L"PNP0C0C") != 0)
//                 Temp = L"Power Button";
//             else if (wcsstr(DeviceData->HardwareIDs, L"PNP0C0E") != 0)
//                 Temp = L"Sleep Button";
//             else if (wcsstr(DeviceData->HardwareIDs, L"PNP0C0D") != 0)
//                 Temp = L"Lid Switch";
//             else if (wcsstr(DeviceData->HardwareIDs, L"PNP0C09") != 0)
//                 Temp = L"ACPI Embedded Controller";
//             else if (wcsstr(DeviceData->HardwareIDs, L"PNP0C0B") != 0)
//                 Temp = L"ACPI Fan";
//             else if (wcsstr(DeviceData->HardwareIDs, L"PNP0A03") != 0 ||
//                      wcsstr(DeviceData->HardwareIDs, L"PNP0A08") != 0)
//                 Temp = L"PCI Root Bridge";
//             else if (wcsstr(DeviceData->HardwareIDs, L"PNP0C0A") != 0)
//                 Temp = L"ACPI Battery";
//             else if (wcsstr(DeviceData->HardwareIDs, L"PNP0C0F") != 0)
//                 Temp = L"PCI Interrupt Link";
//             else if (wcsstr(DeviceData->HardwareIDs, L"ACPI_PWR") != 0)
//                 Temp = L"ACPI Power Resource";
//             else if (wcsstr(DeviceData->HardwareIDs, L"Processor") != 0)
//             {
//                 if (ProcessorNameString != NULL)
//                     Temp = ProcessorNameString;
//                 else
//                     Temp = L"Processor";
//             }
//             else if (wcsstr(DeviceData->HardwareIDs, L"ThermalZone") != 0)
//                 Temp = L"ACPI Thermal Zone";
//             else if (wcsstr(DeviceData->HardwareIDs, L"ACPI0002") != 0)
//                 Temp = L"Smart Battery";
//             else if (wcsstr(DeviceData->HardwareIDs, L"ACPI0003") != 0)
//                 Temp = L"AC Adapter";
//             /* Simply checking if AcpiHandle is NULL eliminates the need to check
//              * for the 4 different names that ACPI knows the fixed feature button as internally
//              */
//             else if (!DeviceData->AcpiHandle)
//                 Temp = L"ACPI Fixed Feature Button";
//             else
//                 Temp = L"Other ACPI device";
//
//             Buffer = ExAllocatePoolWithTag(PagedPool, (wcslen(Temp) + 1) * sizeof(WCHAR), u32::from_be_bytes(*b"IpcA"));
//
//             if (!Buffer) {
//                 status = STATUS_INSUFFICIENT_RESOURCES;
//                 break;
//             }
//
//             RtlCopyMemory (Buffer, Temp, (wcslen(Temp) + 1) * sizeof(WCHAR));
//
//             DPRINT("\tDeviceTextDescription :%ws\n", Buffer);
//
//             Irp->IoStatus.Information = (ULONG_PTR) Buffer;
//             status = STATUS_SUCCESS;
//         }
//         break;
//
//     default:
//         break;
//     }
//
//     return status;
//
// }
//
// NTSTATUS
// Bus_PDO_QueryResources(
//      PPDO_DEVICE_DATA     DeviceData,
//       PIRP   Irp )
// {
//     ULONG NumberOfResources = 0;
//     PCM_RESOURCE_LIST ResourceList;
//     PCM_PARTIAL_RESOURCE_DESCRIPTOR ResourceDescriptor;
//     ACPI_STATUS AcpiStatus;
//     ACPI_BUFFER Buffer;
//     ACPI_RESOURCE* resource;
//     ULONG ResourceListSize;
//     ULONG i;
//     ULONGLONG BusNumber;
//     struct acpi_device *device;
//
//     if (!DeviceData->AcpiHandle)
//     {
//         return Irp->IoStatus.Status;
//     }
//
//     /* A bus number resource is not included in the list of current resources
//      * for the root PCI bus so we manually query one here and if we find it
//      * we create a resource list and add a bus number descriptor to it */
//     if (wcsstr(DeviceData->HardwareIDs, L"PNP0A03") != 0 ||
//         wcsstr(DeviceData->HardwareIDs, L"PNP0A08") != 0)
//     {
//         acpi_bus_get_device(DeviceData->AcpiHandle, &device);
//
//         AcpiStatus = acpi_evaluate_integer(DeviceData->AcpiHandle, "_BBN", NULL, &BusNumber);
//         if (AcpiStatus != AE_OK)
//         {
//             {
//                 BusNumber = 0;
//                 DPRINT1("Failed to find a bus number\n");
//             }
//         }
//         else
//         {
//             DPRINT("Using _BBN for bus number\n");
//         }
//
//         DPRINT("Found PCI root hub: %d\n", BusNumber);
//
//         ResourceListSize = sizeof(CM_RESOURCE_LIST);
//         ResourceList = ExAllocatePoolWithTag(PagedPool, ResourceListSize as usize, u32::from_be_bytes(*b"RpcA"));
//         if (!ResourceList)
//             return STATUS_INSUFFICIENT_RESOURCES;
//
//         ResourceList->Count = 1;
//         ResourceList->List[0].InterfaceType = Internal;
//         ResourceList->List[0].BusNumber = 0;
//         ResourceList->List[0].PartialResourceList.Version = 1;
//         ResourceList->List[0].PartialResourceList.Revision = 1;
//         ResourceList->List[0].PartialResourceList.Count = 1;
//         ResourceDescriptor = ResourceList->List[0].PartialResourceList.PartialDescriptors;
//
//         ResourceDescriptor->Type = CmResourceTypeBusNumber;
//         ResourceDescriptor->ShareDisposition = CmResourceShareDeviceExclusive;
//         ResourceDescriptor->u.BusNumber.Start = BusNumber;
//         ResourceDescriptor->u.BusNumber.Length = 1;
//
//         Irp->IoStatus.Information = (ULONG_PTR)ResourceList;
//         return STATUS_SUCCESS;
//     }
//
//     /* Get current resources */
//     Buffer.Length = 0;
//     AcpiStatus = AcpiGetCurrentResources(DeviceData->AcpiHandle, &Buffer);
//     if ((!ACPI_SUCCESS(AcpiStatus) && AcpiStatus != AE_BUFFER_OVERFLOW) ||
//         Buffer.Length == 0)
//     {
//       return Irp->IoStatus.Status;
//     }
//
//     Buffer.Pointer = ExAllocatePoolWithTag(PagedPool, Buffer.Length as usize, u32::from_be_bytes(*b"BpcA"));
//     if (!Buffer.Pointer)
//       return STATUS_INSUFFICIENT_RESOURCES;
//
//     AcpiStatus = AcpiGetCurrentResources(DeviceData->AcpiHandle, &Buffer);
//     if (!ACPI_SUCCESS(AcpiStatus))
//     {
//       DPRINT1("AcpiGetCurrentResources #2 failed (0x%x)\n", AcpiStatus);
//       ASSERT(FALSE);
//       return STATUS_UNSUCCESSFUL;
//     }
//
//     resource= Buffer.Pointer;
//     /* Count number of resources */
//     while (resource->Type != ACPI_RESOURCE_TYPE_END_TAG)
//     {
//         switch (resource->Type)
//         {
//             case ACPI_RESOURCE_TYPE_EXTENDED_IRQ:
//             {
//                 ACPI_RESOURCE_EXTENDED_IRQ *irq_data = (ACPI_RESOURCE_EXTENDED_IRQ*) &resource->Data;
//                 if (irq_data->ProducerConsumer == ACPI_PRODUCER)
//                     break;
//                 NumberOfResources += irq_data->InterruptCount;
//                 break;
//             }
//             case ACPI_RESOURCE_TYPE_IRQ:
//             {
//                 ACPI_RESOURCE_IRQ *irq_data = (ACPI_RESOURCE_IRQ*) &resource->Data;
//                 NumberOfResources += irq_data->InterruptCount;
//                 break;
//             }
//             case ACPI_RESOURCE_TYPE_DMA:
//             {
//                 ACPI_RESOURCE_DMA *dma_data = (ACPI_RESOURCE_DMA*) &resource->Data;
//                 NumberOfResources += dma_data->ChannelCount;
//                 break;
//             }
//             case ACPI_RESOURCE_TYPE_ADDRESS16:
//             case ACPI_RESOURCE_TYPE_ADDRESS32:
//             case ACPI_RESOURCE_TYPE_ADDRESS64:
//             case ACPI_RESOURCE_TYPE_EXTENDED_ADDRESS64:
//             {
//                 ACPI_RESOURCE_ADDRESS *addr_res = (ACPI_RESOURCE_ADDRESS*) &resource->Data;
//                 if (addr_res->ProducerConsumer == ACPI_PRODUCER)
//                     break;
//                 NumberOfResources++;
//                 break;
//             }
//             case ACPI_RESOURCE_TYPE_MEMORY24:
//             case ACPI_RESOURCE_TYPE_MEMORY32:
//             case ACPI_RESOURCE_TYPE_FIXED_MEMORY32:
//             case ACPI_RESOURCE_TYPE_FIXED_IO:
//             case ACPI_RESOURCE_TYPE_IO:
//             {
//                 NumberOfResources++;
//                 break;
//             }
//             default:
//             {
//                 DPRINT1("Unknown resource type: %d\n", resource->Type);
//                 break;
//             }
//         }
//         resource = ACPI_NEXT_RESOURCE(resource);
//     }
//
//     /* Allocate memory */
//     ResourceListSize = sizeof(CM_RESOURCE_LIST) + sizeof(CM_PARTIAL_RESOURCE_DESCRIPTOR) * (NumberOfResources - 1);
//     ResourceList = ExAllocatePoolWithTag(PagedPool, ResourceListSize as usize, u32::from_be_bytes(*b"RpcA"));
//
//     if (!ResourceList)
//     {
//         ExFreePoolWithTag(Buffer.Pointer, u32::from_be_bytes(*b"BpcA"));
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//     ResourceList->Count = 1;
//     ResourceList->List[0].InterfaceType = Internal; /* FIXME */
//     ResourceList->List[0].BusNumber = 0; /* We're the only ACPI bus device in the system */
//     ResourceList->List[0].PartialResourceList.Version = 1;
//     ResourceList->List[0].PartialResourceList.Revision = 1;
//     ResourceList->List[0].PartialResourceList.Count = NumberOfResources;
//     ResourceDescriptor = ResourceList->List[0].PartialResourceList.PartialDescriptors;
//
//     /* Fill resources list structure */
//         resource = Buffer.Pointer;
//     while (resource->Type != ACPI_RESOURCE_TYPE_END_TAG)
//     {
//         switch (resource->Type)
//         {
//             case ACPI_RESOURCE_TYPE_EXTENDED_IRQ:
//             {
//                 ACPI_RESOURCE_EXTENDED_IRQ *irq_data = (ACPI_RESOURCE_EXTENDED_IRQ*) &resource->Data;
//                 if (irq_data->ProducerConsumer == ACPI_PRODUCER)
//                     break;
//                 for (i = 0; i < irq_data->InterruptCount; i++)
//                 {
//                     ResourceDescriptor->Type = CmResourceTypeInterrupt;
//
//                     ResourceDescriptor->ShareDisposition =
//                     (irq_data->Shareable == ACPI_SHARED ? CmResourceShareShared : CmResourceShareDeviceExclusive);
//                     ResourceDescriptor->Flags =
//                     (irq_data->Triggering == ACPI_LEVEL_SENSITIVE ? CM_RESOURCE_INTERRUPT_LEVEL_SENSITIVE : CM_RESOURCE_INTERRUPT_LATCHED);
//                     ResourceDescriptor->u.Interrupt.Level =
//                     ResourceDescriptor->u.Interrupt.Vector = irq_data->Interrupts[i];
//                     ResourceDescriptor->u.Interrupt.Affinity = (KAFFINITY)(-1);
//
//                     ResourceDescriptor++;
//                 }
//                 break;
//             }
//             case ACPI_RESOURCE_TYPE_IRQ:
//             {
//                 ACPI_RESOURCE_IRQ *irq_data = (ACPI_RESOURCE_IRQ*) &resource->Data;
//                 for (i = 0; i < irq_data->InterruptCount; i++)
//                 {
//                     ResourceDescriptor->Type = CmResourceTypeInterrupt;
//
//                     ResourceDescriptor->ShareDisposition =
//                     (irq_data->Shareable == ACPI_SHARED ? CmResourceShareShared : CmResourceShareDeviceExclusive);
//                     ResourceDescriptor->Flags =
//                     (irq_data->Triggering == ACPI_LEVEL_SENSITIVE ? CM_RESOURCE_INTERRUPT_LEVEL_SENSITIVE : CM_RESOURCE_INTERRUPT_LATCHED);
//                     ResourceDescriptor->u.Interrupt.Level =
//                     ResourceDescriptor->u.Interrupt.Vector = irq_data->Interrupts[i];
//                     ResourceDescriptor->u.Interrupt.Affinity = (KAFFINITY)(-1);
//
//                     ResourceDescriptor++;
//                 }
//                 break;
//             }
//             case ACPI_RESOURCE_TYPE_DMA:
//             {
//                 ACPI_RESOURCE_DMA *dma_data = (ACPI_RESOURCE_DMA*) &resource->Data;
//                 for (i = 0; i < dma_data->ChannelCount; i++)
//                 {
//                     ResourceDescriptor->Type = CmResourceTypeDma;
//                     ResourceDescriptor->Flags = 0;
//                     switch (dma_data->Type)
//                     {
//                         case ACPI_TYPE_A: ResourceDescriptor->Flags |= CM_RESOURCE_DMA_TYPE_A; break;
//                         case ACPI_TYPE_B: ResourceDescriptor->Flags |= CM_RESOURCE_DMA_TYPE_B; break;
//                         case ACPI_TYPE_F: ResourceDescriptor->Flags |= CM_RESOURCE_DMA_TYPE_F; break;
//                     }
//                     if (dma_data->BusMaster == ACPI_BUS_MASTER)
//                         ResourceDescriptor->Flags |= CM_RESOURCE_DMA_BUS_MASTER;
//                     switch (dma_data->Transfer)
//                     {
//                         case ACPI_TRANSFER_8: ResourceDescriptor->Flags |= CM_RESOURCE_DMA_8; break;
//                         case ACPI_TRANSFER_16: ResourceDescriptor->Flags |= CM_RESOURCE_DMA_16; break;
//                         case ACPI_TRANSFER_8_16: ResourceDescriptor->Flags |= CM_RESOURCE_DMA_8_AND_16; break;
//                     }
//                     ResourceDescriptor->u.Dma.Channel = dma_data->Channels[i];
//
//                     ResourceDescriptor++;
//                 }
//                 break;
//             }
//             case ACPI_RESOURCE_TYPE_IO:
//             {
//                 ACPI_RESOURCE_IO *io_data = (ACPI_RESOURCE_IO*) &resource->Data;
//                 ResourceDescriptor->Type = CmResourceTypePort;
//                 ResourceDescriptor->ShareDisposition = CmResourceShareDriverExclusive;
//                 ResourceDescriptor->Flags = CM_RESOURCE_PORT_IO;
//                 if (io_data->IoDecode == ACPI_DECODE_16)
//                     ResourceDescriptor->Flags |= CM_RESOURCE_PORT_16_BIT_DECODE;
//                 else
//                     ResourceDescriptor->Flags |= CM_RESOURCE_PORT_10_BIT_DECODE;
//                 ResourceDescriptor->u.Port.Start.QuadPart = io_data->Minimum;
//                 ResourceDescriptor->u.Port.Length = io_data->AddressLength;
//
//                 ResourceDescriptor++;
//                 break;
//             }
//             case ACPI_RESOURCE_TYPE_FIXED_IO:
//             {
//                 ACPI_RESOURCE_FIXED_IO *io_data = (ACPI_RESOURCE_FIXED_IO*) &resource->Data;
//                 ResourceDescriptor->Type = CmResourceTypePort;
//                 ResourceDescriptor->ShareDisposition = CmResourceShareDriverExclusive;
//                 ResourceDescriptor->Flags = CM_RESOURCE_PORT_IO;
//                 ResourceDescriptor->u.Port.Start.QuadPart = io_data->Address;
//                 ResourceDescriptor->u.Port.Length = io_data->AddressLength;
//
//                 ResourceDescriptor++;
//                 break;
//             }
//             case ACPI_RESOURCE_TYPE_ADDRESS16:
//             {
//                 ACPI_RESOURCE_ADDRESS16 *addr16_data = (ACPI_RESOURCE_ADDRESS16*) &resource->Data;
//                 if (addr16_data->ProducerConsumer == ACPI_PRODUCER)
//                     break;
//                 if (addr16_data->ResourceType == ACPI_BUS_NUMBER_RANGE)
//                 {
//                     ResourceDescriptor->Type = CmResourceTypeBusNumber;
//                     ResourceDescriptor->ShareDisposition = CmResourceShareShared;
//                     ResourceDescriptor->Flags = 0;
//                     ResourceDescriptor->u.BusNumber.Start = addr16_data->Address.Minimum;
//                     ResourceDescriptor->u.BusNumber.Length = addr16_data->Address.AddressLength;
//                 }
//                 else if (addr16_data->ResourceType == ACPI_IO_RANGE)
//                 {
//                     ResourceDescriptor->Type = CmResourceTypePort;
//                     ResourceDescriptor->ShareDisposition = CmResourceShareDeviceExclusive;
//                     ResourceDescriptor->Flags = CM_RESOURCE_PORT_IO;
//                     if (addr16_data->Decode == ACPI_POS_DECODE)
//                         ResourceDescriptor->Flags |= CM_RESOURCE_PORT_POSITIVE_DECODE;
//                     ResourceDescriptor->u.Port.Start.QuadPart = addr16_data->Address.Minimum;
//                     ResourceDescriptor->u.Port.Length = addr16_data->Address.AddressLength;
//                 }
//                 else
//                 {
//                     ResourceDescriptor->Type = CmResourceTypeMemory;
//                     ResourceDescriptor->ShareDisposition = CmResourceShareDeviceExclusive;
//                     ResourceDescriptor->Flags = 0;
//                     if (addr16_data->Info.Mem.WriteProtect == ACPI_READ_ONLY_MEMORY)
//                         ResourceDescriptor->Flags |= CM_RESOURCE_MEMORY_READ_ONLY;
//                     else
//                         ResourceDescriptor->Flags |= CM_RESOURCE_MEMORY_READ_WRITE;
//                     switch (addr16_data->Info.Mem.Caching)
//                     {
//                         case ACPI_CACHABLE_MEMORY: ResourceDescriptor->Flags |= CM_RESOURCE_MEMORY_CACHEABLE; break;
//                         case ACPI_WRITE_COMBINING_MEMORY: ResourceDescriptor->Flags |= CM_RESOURCE_MEMORY_COMBINEDWRITE; break;
//                         case ACPI_PREFETCHABLE_MEMORY: ResourceDescriptor->Flags |= CM_RESOURCE_MEMORY_PREFETCHABLE; break;
//                     }
//                     ResourceDescriptor->u.Memory.Start.QuadPart = addr16_data->Address.Minimum;
//                     ResourceDescriptor->u.Memory.Length = addr16_data->Address.AddressLength;
//                 }
//                 ResourceDescriptor++;
//                 break;
//             }
//             case ACPI_RESOURCE_TYPE_ADDRESS32:
//             {
//                 ACPI_RESOURCE_ADDRESS32 *addr32_data = (ACPI_RESOURCE_ADDRESS32*) &resource->Data;
//                 if (addr32_data->ProducerConsumer == ACPI_PRODUCER)
//                     break;
//                 if (addr32_data->ResourceType == ACPI_BUS_NUMBER_RANGE)
//                 {
//                     ResourceDescriptor->Type = CmResourceTypeBusNumber;
//                     ResourceDescriptor->ShareDisposition = CmResourceShareShared;
//                     ResourceDescriptor->Flags = 0;
//                     ResourceDescriptor->u.BusNumber.Start = addr32_data->Address.Minimum;
//                     ResourceDescriptor->u.BusNumber.Length = addr32_data->Address.AddressLength;
//                 }
//                 else if (addr32_data->ResourceType == ACPI_IO_RANGE)
//                 {
//                     ResourceDescriptor->Type = CmResourceTypePort;
//                     ResourceDescriptor->ShareDisposition = CmResourceShareDeviceExclusive;
//                     ResourceDescriptor->Flags = CM_RESOURCE_PORT_IO;
//                     if (addr32_data->Decode == ACPI_POS_DECODE)
//                         ResourceDescriptor->Flags |= CM_RESOURCE_PORT_POSITIVE_DECODE;
//                     ResourceDescriptor->u.Port.Start.QuadPart = addr32_data->Address.Minimum;
//                     ResourceDescriptor->u.Port.Length = addr32_data->Address.AddressLength;
//                 }
//                 else
//                 {
//                     ResourceDescriptor->Type = CmResourceTypeMemory;
//                     ResourceDescriptor->ShareDisposition = CmResourceShareDeviceExclusive;
//                     ResourceDescriptor->Flags = 0;
//                     if (addr32_data->Info.Mem.WriteProtect == ACPI_READ_ONLY_MEMORY)
//                         ResourceDescriptor->Flags |= CM_RESOURCE_MEMORY_READ_ONLY;
//                     else
//                         ResourceDescriptor->Flags |= CM_RESOURCE_MEMORY_READ_WRITE;
//                     switch (addr32_data->Info.Mem.Caching)
//                     {
//                         case ACPI_CACHABLE_MEMORY: ResourceDescriptor->Flags |= CM_RESOURCE_MEMORY_CACHEABLE; break;
//                         case ACPI_WRITE_COMBINING_MEMORY: ResourceDescriptor->Flags |= CM_RESOURCE_MEMORY_COMBINEDWRITE; break;
//                         case ACPI_PREFETCHABLE_MEMORY: ResourceDescriptor->Flags |= CM_RESOURCE_MEMORY_PREFETCHABLE; break;
//                     }
//                     ResourceDescriptor->u.Memory.Start.QuadPart = addr32_data->Address.Minimum;
//                     ResourceDescriptor->u.Memory.Length = addr32_data->Address.AddressLength;
//                 }
//                 ResourceDescriptor++;
//                 break;
//             }
//             case ACPI_RESOURCE_TYPE_ADDRESS64:
//             {
//                 ACPI_RESOURCE_ADDRESS64 *addr64_data = (ACPI_RESOURCE_ADDRESS64*) &resource->Data;
//                 if (addr64_data->ProducerConsumer == ACPI_PRODUCER)
//                     break;
//                 if (addr64_data->ResourceType == ACPI_BUS_NUMBER_RANGE)
//                 {
//                     DPRINT1("64-bit bus address is not supported!\n");
//                     ResourceDescriptor->Type = CmResourceTypeBusNumber;
//                     ResourceDescriptor->ShareDisposition = CmResourceShareShared;
//                     ResourceDescriptor->Flags = 0;
//                     ResourceDescriptor->u.BusNumber.Start = (ULONG)addr64_data->Address.Minimum;
//                     ResourceDescriptor->u.BusNumber.Length = addr64_data->Address.AddressLength;
//                 }
//                 else if (addr64_data->ResourceType == ACPI_IO_RANGE)
//                 {
//                     ResourceDescriptor->Type = CmResourceTypePort;
//                     ResourceDescriptor->ShareDisposition = CmResourceShareDeviceExclusive;
//                     ResourceDescriptor->Flags = CM_RESOURCE_PORT_IO;
//                     if (addr64_data->Decode == ACPI_POS_DECODE)
//                         ResourceDescriptor->Flags |= CM_RESOURCE_PORT_POSITIVE_DECODE;
//                     ResourceDescriptor->u.Port.Start.QuadPart = addr64_data->Address.Minimum;
//                     ResourceDescriptor->u.Port.Length = addr64_data->Address.AddressLength;
//                 }
//                 else
//                 {
//                     ResourceDescriptor->Type = CmResourceTypeMemory;
//                     ResourceDescriptor->ShareDisposition = CmResourceShareDeviceExclusive;
//                     ResourceDescriptor->Flags = 0;
//                     if (addr64_data->Info.Mem.WriteProtect == ACPI_READ_ONLY_MEMORY)
//                         ResourceDescriptor->Flags |= CM_RESOURCE_MEMORY_READ_ONLY;
//                     else
//                         ResourceDescriptor->Flags |= CM_RESOURCE_MEMORY_READ_WRITE;
//                     switch (addr64_data->Info.Mem.Caching)
//                     {
//                         case ACPI_CACHABLE_MEMORY: ResourceDescriptor->Flags |= CM_RESOURCE_MEMORY_CACHEABLE; break;
//                         case ACPI_WRITE_COMBINING_MEMORY: ResourceDescriptor->Flags |= CM_RESOURCE_MEMORY_COMBINEDWRITE; break;
//                         case ACPI_PREFETCHABLE_MEMORY: ResourceDescriptor->Flags |= CM_RESOURCE_MEMORY_PREFETCHABLE; break;
//                     }
//                     ResourceDescriptor->u.Memory.Start.QuadPart = addr64_data->Address.Minimum;
//                     ResourceDescriptor->u.Memory.Length = addr64_data->Address.AddressLength;
//                 }
//                 ResourceDescriptor++;
//                 break;
//             }
//             case ACPI_RESOURCE_TYPE_EXTENDED_ADDRESS64:
//             {
//                 ACPI_RESOURCE_EXTENDED_ADDRESS64 *addr64_data = (ACPI_RESOURCE_EXTENDED_ADDRESS64*) &resource->Data;
//                 if (addr64_data->ProducerConsumer == ACPI_PRODUCER)
//                     break;
//                 if (addr64_data->ResourceType == ACPI_BUS_NUMBER_RANGE)
//                 {
//                     DPRINT1("64-bit bus address is not supported!\n");
//                     ResourceDescriptor->Type = CmResourceTypeBusNumber;
//                     ResourceDescriptor->ShareDisposition = CmResourceShareShared;
//                     ResourceDescriptor->Flags = 0;
//                     ResourceDescriptor->u.BusNumber.Start = (ULONG)addr64_data->Address.Minimum;
//                     ResourceDescriptor->u.BusNumber.Length = addr64_data->Address.AddressLength;
//                 }
//                 else if (addr64_data->ResourceType == ACPI_IO_RANGE)
//                 {
//                     ResourceDescriptor->Type = CmResourceTypePort;
//                     ResourceDescriptor->ShareDisposition = CmResourceShareDeviceExclusive;
//                     ResourceDescriptor->Flags = CM_RESOURCE_PORT_IO;
//                     if (addr64_data->Decode == ACPI_POS_DECODE)
//                         ResourceDescriptor->Flags |= CM_RESOURCE_PORT_POSITIVE_DECODE;
//                     ResourceDescriptor->u.Port.Start.QuadPart = addr64_data->Address.Minimum;
//                     ResourceDescriptor->u.Port.Length = addr64_data->Address.AddressLength;
//                 }
//                 else
//                 {
//                     ResourceDescriptor->Type = CmResourceTypeMemory;
//                     ResourceDescriptor->ShareDisposition = CmResourceShareDeviceExclusive;
//                     ResourceDescriptor->Flags = 0;
//                     if (addr64_data->Info.Mem.WriteProtect == ACPI_READ_ONLY_MEMORY)
//                         ResourceDescriptor->Flags |= CM_RESOURCE_MEMORY_READ_ONLY;
//                     else
//                         ResourceDescriptor->Flags |= CM_RESOURCE_MEMORY_READ_WRITE;
//                     switch (addr64_data->Info.Mem.Caching)
//                     {
//                         case ACPI_CACHABLE_MEMORY: ResourceDescriptor->Flags |= CM_RESOURCE_MEMORY_CACHEABLE; break;
//                         case ACPI_WRITE_COMBINING_MEMORY: ResourceDescriptor->Flags |= CM_RESOURCE_MEMORY_COMBINEDWRITE; break;
//                         case ACPI_PREFETCHABLE_MEMORY: ResourceDescriptor->Flags |= CM_RESOURCE_MEMORY_PREFETCHABLE; break;
//                     }
//                     ResourceDescriptor->u.Memory.Start.QuadPart = addr64_data->Address.Minimum;
//                     ResourceDescriptor->u.Memory.Length = addr64_data->Address.AddressLength;
//                 }
//                 ResourceDescriptor++;
//                 break;
//             }
//             case ACPI_RESOURCE_TYPE_MEMORY24:
//             {
//                 ACPI_RESOURCE_MEMORY24 *mem24_data = (ACPI_RESOURCE_MEMORY24*) &resource->Data;
//                 ResourceDescriptor->Type = CmResourceTypeMemory;
//                 ResourceDescriptor->ShareDisposition = CmResourceShareDeviceExclusive;
//                 ResourceDescriptor->Flags = CM_RESOURCE_MEMORY_24;
//                 if (mem24_data->WriteProtect == ACPI_READ_ONLY_MEMORY)
//                     ResourceDescriptor->Flags |= CM_RESOURCE_MEMORY_READ_ONLY;
//                 else
//                     ResourceDescriptor->Flags |= CM_RESOURCE_MEMORY_READ_WRITE;
//                 ResourceDescriptor->u.Memory.Start.QuadPart = mem24_data->Minimum;
//                 ResourceDescriptor->u.Memory.Length = mem24_data->AddressLength;
//
//                 ResourceDescriptor++;
//                 break;
//             }
//             case ACPI_RESOURCE_TYPE_MEMORY32:
//             {
//                 ACPI_RESOURCE_MEMORY32 *mem32_data = (ACPI_RESOURCE_MEMORY32*) &resource->Data;
//                 ResourceDescriptor->Type = CmResourceTypeMemory;
//                 ResourceDescriptor->ShareDisposition = CmResourceShareDeviceExclusive;
//                 ResourceDescriptor->Flags = 0;
//                 if (mem32_data->WriteProtect == ACPI_READ_ONLY_MEMORY)
//                     ResourceDescriptor->Flags |= CM_RESOURCE_MEMORY_READ_ONLY;
//                 else
//                     ResourceDescriptor->Flags |= CM_RESOURCE_MEMORY_READ_WRITE;
//                 ResourceDescriptor->u.Memory.Start.QuadPart = mem32_data->Minimum;
//                 ResourceDescriptor->u.Memory.Length = mem32_data->AddressLength;
//
//                 ResourceDescriptor++;
//                 break;
//             }
//             case ACPI_RESOURCE_TYPE_FIXED_MEMORY32:
//             {
//                 ACPI_RESOURCE_FIXED_MEMORY32 *memfixed32_data = (ACPI_RESOURCE_FIXED_MEMORY32*) &resource->Data;
//                 ResourceDescriptor->Type = CmResourceTypeMemory;
//                 ResourceDescriptor->ShareDisposition = CmResourceShareDeviceExclusive;
//                 ResourceDescriptor->Flags = 0;
//                 if (memfixed32_data->WriteProtect == ACPI_READ_ONLY_MEMORY)
//                     ResourceDescriptor->Flags |= CM_RESOURCE_MEMORY_READ_ONLY;
//                 else
//                     ResourceDescriptor->Flags |= CM_RESOURCE_MEMORY_READ_WRITE;
//                 ResourceDescriptor->u.Memory.Start.QuadPart = memfixed32_data->Address;
//                 ResourceDescriptor->u.Memory.Length = memfixed32_data->AddressLength;
//
//                 ResourceDescriptor++;
//                 break;
//             }
//             default:
//             {
//                 break;
//             }
//         }
//         resource = ACPI_NEXT_RESOURCE(resource);
//     }
//
//     ExFreePoolWithTag(Buffer.Pointer, u32::from_be_bytes(*b"BpcA"));
//     Irp->IoStatus.Information = (ULONG_PTR)ResourceList;
//     return STATUS_SUCCESS;
// }
// #endif /* UNIT_TEST */
//
// NTSTATUS
// Bus_PDO_QueryResourceRequirements(
//      PPDO_DEVICE_DATA     DeviceData,
//       PIRP   Irp )
// {
//     ULONG NumberOfResources = 0;
//     ACPI_STATUS AcpiStatus;
//     ACPI_BUFFER Buffer;
//     ACPI_RESOURCE* resource;
//     ULONG i, RequirementsListSize;
//     PIO_RESOURCE_REQUIREMENTS_LIST RequirementsList;
//     PIO_RESOURCE_DESCRIPTOR RequirementDescriptor;
//     BOOLEAN CurrentRes = FALSE;
//     BOOLEAN SeenStartDependent;
//
//     PAGED_CODE ();
//
//     if (!DeviceData->AcpiHandle)
//     {
//         return Irp->IoStatus.Status;
//     }
//
//     /* Handle the PCI root manually */
//     if (wcsstr(DeviceData->HardwareIDs, L"PNP0A03") != 0 ||
//         wcsstr(DeviceData->HardwareIDs, L"PNP0A08") != 0)
//     {
//         return Irp->IoStatus.Status;
//     }
//
//     /* Get current resources */
//     while (TRUE)
//     {
//         Buffer.Length = 0;
//         if (CurrentRes)
//         AcpiStatus = AcpiGetCurrentResources(DeviceData->AcpiHandle, &Buffer);
//         else
//             AcpiStatus = AcpiGetPossibleResources(DeviceData->AcpiHandle, &Buffer);
//         if ((!ACPI_SUCCESS(AcpiStatus) && AcpiStatus != AE_BUFFER_OVERFLOW) ||
//             Buffer.Length == 0)
//         {
//             if (!CurrentRes)
//                 CurrentRes = TRUE;
//             else
//                 return Irp->IoStatus.Status;
//         }
//         else
//             break;
//     }
//
//     Buffer.Pointer = ExAllocatePoolWithTag(PagedPool, Buffer.Length as usize, u32::from_be_bytes(*b"BpcA"));
//     if (!Buffer.Pointer)
//       return STATUS_INSUFFICIENT_RESOURCES;
//
//     if (CurrentRes)
//         AcpiStatus = AcpiGetCurrentResources(DeviceData->AcpiHandle, &Buffer);
//     else
//         AcpiStatus = AcpiGetPossibleResources(DeviceData->AcpiHandle, &Buffer);
//     if (!ACPI_SUCCESS(AcpiStatus))
//     {
//       DPRINT1("AcpiGetCurrentResources #2 failed (0x%x)\n", AcpiStatus);
//       ASSERT(FALSE);
//       return STATUS_UNSUCCESSFUL;
//     }
//
//     SeenStartDependent = FALSE;
//     resource = Buffer.Pointer;
//     /* Count number of resources */
//     while (resource->Type != ACPI_RESOURCE_TYPE_END_TAG && resource->Type != ACPI_RESOURCE_TYPE_END_DEPENDENT)
//     {
//         if (resource->Type == ACPI_RESOURCE_TYPE_START_DEPENDENT)
//         {
//             if (SeenStartDependent)
//             {
//                 break;
//             }
//             SeenStartDependent = TRUE;
//         }
//         switch (resource->Type)
//         {
//             case ACPI_RESOURCE_TYPE_EXTENDED_IRQ:
//             {
//                 ACPI_RESOURCE_EXTENDED_IRQ *irq_data = (ACPI_RESOURCE_EXTENDED_IRQ*) &resource->Data;
//                 if (irq_data->ProducerConsumer == ACPI_PRODUCER)
//                     break;
//                 NumberOfResources += irq_data->InterruptCount;
//                 break;
//             }
//             case ACPI_RESOURCE_TYPE_IRQ:
//             {
//                 ACPI_RESOURCE_IRQ *irq_data = (ACPI_RESOURCE_IRQ*) &resource->Data;
//                 NumberOfResources += irq_data->InterruptCount;
//                 break;
//             }
//             case ACPI_RESOURCE_TYPE_DMA:
//             {
//                 ACPI_RESOURCE_DMA *dma_data = (ACPI_RESOURCE_DMA*) &resource->Data;
//                 NumberOfResources += dma_data->ChannelCount;
//                 break;
//             }
//             case ACPI_RESOURCE_TYPE_ADDRESS16:
//             case ACPI_RESOURCE_TYPE_ADDRESS32:
//             case ACPI_RESOURCE_TYPE_ADDRESS64:
//             case ACPI_RESOURCE_TYPE_EXTENDED_ADDRESS64:
//             {
//                 ACPI_RESOURCE_ADDRESS *res_addr = (ACPI_RESOURCE_ADDRESS*) &resource->Data;
//                 if (res_addr->ProducerConsumer == ACPI_PRODUCER)
//                     break;
//                 NumberOfResources++;
//                 break;
//             }
//             case ACPI_RESOURCE_TYPE_MEMORY24:
//             case ACPI_RESOURCE_TYPE_MEMORY32:
//             case ACPI_RESOURCE_TYPE_FIXED_MEMORY32:
//             case ACPI_RESOURCE_TYPE_FIXED_IO:
//             case ACPI_RESOURCE_TYPE_IO:
//             {
//                 NumberOfResources++;
//                 break;
//             }
//             default:
//             {
//                 break;
//             }
//         }
//         resource = ACPI_NEXT_RESOURCE(resource);
//     }
//
//     RequirementsListSize = sizeof(IO_RESOURCE_REQUIREMENTS_LIST) + sizeof(IO_RESOURCE_DESCRIPTOR) * (NumberOfResources - 1);
//     RequirementsList = ExAllocatePoolWithTag(PagedPool, RequirementsListSize, u32::from_be_bytes(*b"RpcA"));
//
//     if (!RequirementsList)
//     {
//         ExFreePoolWithTag(Buffer.Pointer, u32::from_be_bytes(*b"BpcA"));
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//     RequirementsList->ListSize = RequirementsListSize;
//     RequirementsList->InterfaceType = Internal;
//     RequirementsList->BusNumber = 0;
//     RequirementsList->SlotNumber = 0; /* Not used by WDM drivers */
//     RequirementsList->AlternativeLists = 1;
//     RequirementsList->List[0].Version = 1;
//     RequirementsList->List[0].Revision = 1;
//     RequirementsList->List[0].Count = NumberOfResources;
//     RequirementDescriptor = RequirementsList->List[0].Descriptors;
//
//     /* Fill resources list structure */
//     SeenStartDependent = FALSE;
//     resource = Buffer.Pointer;
//     while (resource->Type != ACPI_RESOURCE_TYPE_END_TAG && resource->Type != ACPI_RESOURCE_TYPE_END_DEPENDENT)
//     {
//         if (resource->Type == ACPI_RESOURCE_TYPE_START_DEPENDENT)
//         {
//             if (SeenStartDependent)
//             {
//                 break;
//             }
//             SeenStartDependent = TRUE;
//         }
//         switch (resource->Type)
//         {
//             case ACPI_RESOURCE_TYPE_EXTENDED_IRQ:
//             {
//                 ACPI_RESOURCE_EXTENDED_IRQ *irq_data = &resource->Data.ExtendedIrq;
//                 if (irq_data->ProducerConsumer == ACPI_PRODUCER)
//                     break;
//                 for (i = 0; i < irq_data->InterruptCount; i++)
//                 {
//                     RequirementDescriptor->Option = (i == 0) ? IO_RESOURCE_PREFERRED : IO_RESOURCE_ALTERNATIVE;
//                     RequirementDescriptor->Type = CmResourceTypeInterrupt;
//                     RequirementDescriptor->ShareDisposition = (irq_data->Shareable == ACPI_SHARED ? CmResourceShareShared : CmResourceShareDeviceExclusive);
//                     RequirementDescriptor->Flags =(irq_data->Triggering == ACPI_LEVEL_SENSITIVE ? CM_RESOURCE_INTERRUPT_LEVEL_SENSITIVE : CM_RESOURCE_INTERRUPT_LATCHED);
//                     RequirementDescriptor->u.Interrupt.MinimumVector =
//                     RequirementDescriptor->u.Interrupt.MaximumVector = irq_data->Interrupts[i];
//
//                     RequirementDescriptor++;
//                 }
//                 break;
//             }
//             case ACPI_RESOURCE_TYPE_IRQ:
//             {
//                 ACPI_RESOURCE_IRQ *irq_data = &resource->Data.Irq;
//                 for (i = 0; i < irq_data->InterruptCount; i++)
//                 {
//                     RequirementDescriptor->Option = (i == 0) ? IO_RESOURCE_PREFERRED : IO_RESOURCE_ALTERNATIVE;
//                     RequirementDescriptor->Type = CmResourceTypeInterrupt;
//                     RequirementDescriptor->ShareDisposition = (irq_data->Shareable == ACPI_SHARED ? CmResourceShareShared : CmResourceShareDeviceExclusive);
//                     RequirementDescriptor->Flags =(irq_data->Triggering == ACPI_LEVEL_SENSITIVE ? CM_RESOURCE_INTERRUPT_LEVEL_SENSITIVE : CM_RESOURCE_INTERRUPT_LATCHED);
//                     RequirementDescriptor->u.Interrupt.MinimumVector =
//                     RequirementDescriptor->u.Interrupt.MaximumVector = irq_data->Interrupts[i];
//
//                     RequirementDescriptor++;
//                 }
//                 break;
//             }
//             case ACPI_RESOURCE_TYPE_DMA:
//             {
//                 ACPI_RESOURCE_DMA *dma_data = &resource->Data.Dma;
//                 for (i = 0; i < dma_data->ChannelCount; i++)
//                 {
//                     RequirementDescriptor->Type = CmResourceTypeDma;
//                     RequirementDescriptor->Flags = 0;
//                     switch (dma_data->Type)
//                     {
//                         case ACPI_TYPE_A: RequirementDescriptor->Flags |= CM_RESOURCE_DMA_TYPE_A; break;
//                         case ACPI_TYPE_B: RequirementDescriptor->Flags |= CM_RESOURCE_DMA_TYPE_B; break;
//                         case ACPI_TYPE_F: RequirementDescriptor->Flags |= CM_RESOURCE_DMA_TYPE_F; break;
//                     }
//                     if (dma_data->BusMaster == ACPI_BUS_MASTER)
//                         RequirementDescriptor->Flags |= CM_RESOURCE_DMA_BUS_MASTER;
//                     switch (dma_data->Transfer)
//                     {
//                         case ACPI_TRANSFER_8: RequirementDescriptor->Flags |= CM_RESOURCE_DMA_8; break;
//                         case ACPI_TRANSFER_16: RequirementDescriptor->Flags |= CM_RESOURCE_DMA_16; break;
//                         case ACPI_TRANSFER_8_16: RequirementDescriptor->Flags |= CM_RESOURCE_DMA_8_AND_16; break;
//                     }
//
//                     RequirementDescriptor->Option = (i == 0) ? IO_RESOURCE_PREFERRED : IO_RESOURCE_ALTERNATIVE;
//                     RequirementDescriptor->ShareDisposition = CmResourceShareDriverExclusive;
//                     RequirementDescriptor->u.Dma.MinimumChannel =
//                     RequirementDescriptor->u.Dma.MaximumChannel = dma_data->Channels[i];
//                     RequirementDescriptor++;
//                 }
//                 break;
//             }
//             case ACPI_RESOURCE_TYPE_IO:
//             {
//                 ACPI_RESOURCE_IO *io_data = &resource->Data.Io;
//                 RequirementDescriptor->Flags = CM_RESOURCE_PORT_IO;
//                 if (io_data->IoDecode == ACPI_DECODE_16)
//                     RequirementDescriptor->Flags |= CM_RESOURCE_PORT_16_BIT_DECODE;
//                 else
//                     RequirementDescriptor->Flags |= CM_RESOURCE_PORT_10_BIT_DECODE;
//                 RequirementDescriptor->u.Port.Length = io_data->AddressLength;
//                 RequirementDescriptor->Option = CurrentRes ? 0 : IO_RESOURCE_PREFERRED;
//                 RequirementDescriptor->Type = CmResourceTypePort;
//                 RequirementDescriptor->ShareDisposition = CmResourceShareDriverExclusive;
//                 RequirementDescriptor->u.Port.Alignment = io_data->Alignment;
//                 RequirementDescriptor->u.Port.MinimumAddress.QuadPart = io_data->Minimum;
//                 RequirementDescriptor->u.Port.MaximumAddress.QuadPart = io_data->Maximum + io_data->AddressLength - 1;
//
//                 RequirementDescriptor++;
//                 break;
//             }
//             case ACPI_RESOURCE_TYPE_FIXED_IO:
//             {
//                 ACPI_RESOURCE_FIXED_IO *io_data = &resource->Data.FixedIo;
//                 RequirementDescriptor->Flags = CM_RESOURCE_PORT_IO;
//                 RequirementDescriptor->u.Port.Length = io_data->AddressLength;
//                 RequirementDescriptor->Option = CurrentRes ? 0 : IO_RESOURCE_PREFERRED;
//                 RequirementDescriptor->Type = CmResourceTypePort;
//                 RequirementDescriptor->ShareDisposition = CmResourceShareDriverExclusive;
//                 RequirementDescriptor->u.Port.Alignment = 1;
//                 RequirementDescriptor->u.Port.MinimumAddress.QuadPart = io_data->Address;
//                 RequirementDescriptor->u.Port.MaximumAddress.QuadPart = io_data->Address + io_data->AddressLength - 1;
//
//                 RequirementDescriptor++;
//                 break;
//             }
//             case ACPI_RESOURCE_TYPE_ADDRESS16:
//             {
//                 ACPI_RESOURCE_ADDRESS16 *addr16_data = &resource->Data.Address16;
//                 if (addr16_data->ProducerConsumer == ACPI_PRODUCER)
//                     break;
//                 RequirementDescriptor->Option = CurrentRes ? 0 : IO_RESOURCE_PREFERRED;
//                 if (addr16_data->ResourceType == ACPI_BUS_NUMBER_RANGE)
//                 {
//                     RequirementDescriptor->Type = CmResourceTypeBusNumber;
//                     RequirementDescriptor->ShareDisposition = CmResourceShareShared;
//                     RequirementDescriptor->Flags = 0;
//                     RequirementDescriptor->u.BusNumber.MinBusNumber = addr16_data->Address.Minimum;
//                     RequirementDescriptor->u.BusNumber.MaxBusNumber = addr16_data->Address.Maximum + addr16_data->Address.AddressLength - 1;
//                     RequirementDescriptor->u.BusNumber.Length = addr16_data->Address.AddressLength;
//                 }
//                 else if (addr16_data->ResourceType == ACPI_IO_RANGE)
//                 {
//                     RequirementDescriptor->Type = CmResourceTypePort;
//                     RequirementDescriptor->ShareDisposition = CmResourceShareDeviceExclusive;
//                     RequirementDescriptor->Flags = CM_RESOURCE_PORT_IO;
//                     if (addr16_data->Decode == ACPI_POS_DECODE)
//                         RequirementDescriptor->Flags |= CM_RESOURCE_PORT_POSITIVE_DECODE;
//                     RequirementDescriptor->u.Port.MinimumAddress.QuadPart = addr16_data->Address.Minimum;
//                     RequirementDescriptor->u.Port.MaximumAddress.QuadPart = addr16_data->Address.Maximum + addr16_data->Address.AddressLength - 1;
//                     RequirementDescriptor->u.Port.Length = addr16_data->Address.AddressLength;
//                 }
//                 else
//                 {
//                     RequirementDescriptor->Type = CmResourceTypeMemory;
//                     RequirementDescriptor->ShareDisposition = CmResourceShareDeviceExclusive;
//                     RequirementDescriptor->Flags = 0;
//                     if (addr16_data->Info.Mem.WriteProtect == ACPI_READ_ONLY_MEMORY)
//                         RequirementDescriptor->Flags |= CM_RESOURCE_MEMORY_READ_ONLY;
//                     else
//                         RequirementDescriptor->Flags |= CM_RESOURCE_MEMORY_READ_WRITE;
//                     switch (addr16_data->Info.Mem.Caching)
//                     {
//                         case ACPI_CACHABLE_MEMORY: RequirementDescriptor->Flags |= CM_RESOURCE_MEMORY_CACHEABLE; break;
//                         case ACPI_WRITE_COMBINING_MEMORY: RequirementDescriptor->Flags |= CM_RESOURCE_MEMORY_COMBINEDWRITE; break;
//                         case ACPI_PREFETCHABLE_MEMORY: RequirementDescriptor->Flags |= CM_RESOURCE_MEMORY_PREFETCHABLE; break;
//                     }
//                     RequirementDescriptor->u.Memory.MinimumAddress.QuadPart = addr16_data->Address.Minimum;
//                     RequirementDescriptor->u.Memory.MaximumAddress.QuadPart = addr16_data->Address.Maximum + addr16_data->Address.AddressLength - 1;
//                     RequirementDescriptor->u.Memory.Length = addr16_data->Address.AddressLength;
//                 }
//                 RequirementDescriptor++;
//                 break;
//             }
//             case ACPI_RESOURCE_TYPE_ADDRESS32:
//             {
//                 ACPI_RESOURCE_ADDRESS32 *addr32_data = &resource->Data.Address32;
//                 if (addr32_data->ProducerConsumer == ACPI_PRODUCER)
//                     break;
//                 RequirementDescriptor->Option = CurrentRes ? 0 : IO_RESOURCE_PREFERRED;
//                 if (addr32_data->ResourceType == ACPI_BUS_NUMBER_RANGE)
//                 {
//                     RequirementDescriptor->Type = CmResourceTypeBusNumber;
//                     RequirementDescriptor->ShareDisposition = CmResourceShareShared;
//                     RequirementDescriptor->Flags = 0;
//                     RequirementDescriptor->u.BusNumber.MinBusNumber = addr32_data->Address.Minimum;
//                     RequirementDescriptor->u.BusNumber.MaxBusNumber = addr32_data->Address.Maximum + addr32_data->Address.AddressLength - 1;
//                     RequirementDescriptor->u.BusNumber.Length = addr32_data->Address.AddressLength;
//                 }
//                 else if (addr32_data->ResourceType == ACPI_IO_RANGE)
//                 {
//                     RequirementDescriptor->Type = CmResourceTypePort;
//                     RequirementDescriptor->ShareDisposition = CmResourceShareDeviceExclusive;
//                     RequirementDescriptor->Flags = CM_RESOURCE_PORT_IO;
//                     if (addr32_data->Decode == ACPI_POS_DECODE)
//                         RequirementDescriptor->Flags |= CM_RESOURCE_PORT_POSITIVE_DECODE;
//                     RequirementDescriptor->u.Port.MinimumAddress.QuadPart = addr32_data->Address.Minimum;
//                     RequirementDescriptor->u.Port.MaximumAddress.QuadPart = addr32_data->Address.Maximum + addr32_data->Address.AddressLength - 1;
//                     RequirementDescriptor->u.Port.Length = addr32_data->Address.AddressLength;
//                 }
//                 else
//                 {
//                     RequirementDescriptor->Type = CmResourceTypeMemory;
//                     RequirementDescriptor->ShareDisposition = CmResourceShareDeviceExclusive;
//                     RequirementDescriptor->Flags = 0;
//                     if (addr32_data->Info.Mem.WriteProtect == ACPI_READ_ONLY_MEMORY)
//                         RequirementDescriptor->Flags |= CM_RESOURCE_MEMORY_READ_ONLY;
//                     else
//                         RequirementDescriptor->Flags |= CM_RESOURCE_MEMORY_READ_WRITE;
//                     switch (addr32_data->Info.Mem.Caching)
//                     {
//                         case ACPI_CACHABLE_MEMORY: RequirementDescriptor->Flags |= CM_RESOURCE_MEMORY_CACHEABLE; break;
//                         case ACPI_WRITE_COMBINING_MEMORY: RequirementDescriptor->Flags |= CM_RESOURCE_MEMORY_COMBINEDWRITE; break;
//                         case ACPI_PREFETCHABLE_MEMORY: RequirementDescriptor->Flags |= CM_RESOURCE_MEMORY_PREFETCHABLE; break;
//                     }
//                     RequirementDescriptor->u.Memory.MinimumAddress.QuadPart = addr32_data->Address.Minimum;
//                     RequirementDescriptor->u.Memory.MaximumAddress.QuadPart = addr32_data->Address.Maximum + addr32_data->Address.AddressLength - 1;
//                     RequirementDescriptor->u.Memory.Length = addr32_data->Address.AddressLength;
//                 }
//                 RequirementDescriptor++;
//                 break;
//             }
//             case ACPI_RESOURCE_TYPE_ADDRESS64:
//             {
//                 ACPI_RESOURCE_ADDRESS64 *addr64_data = &resource->Data.Address64;
//                 if (addr64_data->ProducerConsumer == ACPI_PRODUCER)
//                     break;
//                 RequirementDescriptor->Option = CurrentRes ? 0 : IO_RESOURCE_PREFERRED;
//                 if (addr64_data->ResourceType == ACPI_BUS_NUMBER_RANGE)
//                 {
//                     DPRINT1("64-bit bus address is not supported!\n");
//                     RequirementDescriptor->Type = CmResourceTypeBusNumber;
//                     RequirementDescriptor->ShareDisposition = CmResourceShareShared;
//                     RequirementDescriptor->Flags = 0;
//                     RequirementDescriptor->u.BusNumber.MinBusNumber = (ULONG)addr64_data->Address.Minimum;
//                     RequirementDescriptor->u.BusNumber.MaxBusNumber = (ULONG)addr64_data->Address.Maximum + addr64_data->Address.AddressLength - 1;
//                     RequirementDescriptor->u.BusNumber.Length = addr64_data->Address.AddressLength;
//                 }
//                 else if (addr64_data->ResourceType == ACPI_IO_RANGE)
//                 {
//                     RequirementDescriptor->Type = CmResourceTypePort;
//                     RequirementDescriptor->ShareDisposition = CmResourceShareDeviceExclusive;
//                     RequirementDescriptor->Flags = CM_RESOURCE_PORT_IO;
//                     if (addr64_data->Decode == ACPI_POS_DECODE)
//                         RequirementDescriptor->Flags |= CM_RESOURCE_PORT_POSITIVE_DECODE;
//                     RequirementDescriptor->u.Port.MinimumAddress.QuadPart = addr64_data->Address.Minimum;
//                     RequirementDescriptor->u.Port.MaximumAddress.QuadPart = addr64_data->Address.Maximum + addr64_data->Address.AddressLength - 1;
//                     RequirementDescriptor->u.Port.Length = addr64_data->Address.AddressLength;
//                 }
//                 else
//                 {
//                     RequirementDescriptor->Type = CmResourceTypeMemory;
//                     RequirementDescriptor->ShareDisposition = CmResourceShareDeviceExclusive;
//                     RequirementDescriptor->Flags = 0;
//                     if (addr64_data->Info.Mem.WriteProtect == ACPI_READ_ONLY_MEMORY)
//                         RequirementDescriptor->Flags |= CM_RESOURCE_MEMORY_READ_ONLY;
//                     else
//                         RequirementDescriptor->Flags |= CM_RESOURCE_MEMORY_READ_WRITE;
//                     switch (addr64_data->Info.Mem.Caching)
//                     {
//                         case ACPI_CACHABLE_MEMORY: RequirementDescriptor->Flags |= CM_RESOURCE_MEMORY_CACHEABLE; break;
//                         case ACPI_WRITE_COMBINING_MEMORY: RequirementDescriptor->Flags |= CM_RESOURCE_MEMORY_COMBINEDWRITE; break;
//                         case ACPI_PREFETCHABLE_MEMORY: RequirementDescriptor->Flags |= CM_RESOURCE_MEMORY_PREFETCHABLE; break;
//                     }
//                     RequirementDescriptor->u.Memory.MinimumAddress.QuadPart = addr64_data->Address.Minimum;
//                     RequirementDescriptor->u.Memory.MaximumAddress.QuadPart = addr64_data->Address.Maximum + addr64_data->Address.AddressLength - 1;
//                     RequirementDescriptor->u.Memory.Length = addr64_data->Address.AddressLength;
//                 }
//                 RequirementDescriptor++;
//                 break;
//             }
//             case ACPI_RESOURCE_TYPE_EXTENDED_ADDRESS64:
//             {
//                 ACPI_RESOURCE_EXTENDED_ADDRESS64 *addr64_data = &resource->Data.ExtAddress64;
//                 if (addr64_data->ProducerConsumer == ACPI_PRODUCER)
//                     break;
//                 RequirementDescriptor->Option = CurrentRes ? 0 : IO_RESOURCE_PREFERRED;
//                 if (addr64_data->ResourceType == ACPI_BUS_NUMBER_RANGE)
//                 {
//                     DPRINT1("64-bit bus address is not supported!\n");
//                     RequirementDescriptor->Type = CmResourceTypeBusNumber;
//                     RequirementDescriptor->ShareDisposition = CmResourceShareShared;
//                     RequirementDescriptor->Flags = 0;
//                     RequirementDescriptor->u.BusNumber.MinBusNumber = (ULONG)addr64_data->Address.Minimum;
//                     RequirementDescriptor->u.BusNumber.MaxBusNumber = (ULONG)addr64_data->Address.Maximum + addr64_data->Address.AddressLength - 1;
//                     RequirementDescriptor->u.BusNumber.Length = addr64_data->Address.AddressLength;
//                 }
//                 else if (addr64_data->ResourceType == ACPI_IO_RANGE)
//                 {
//                     RequirementDescriptor->Type = CmResourceTypePort;
//                     RequirementDescriptor->ShareDisposition = CmResourceShareDeviceExclusive;
//                     RequirementDescriptor->Flags = CM_RESOURCE_PORT_IO;
//                     if (addr64_data->Decode == ACPI_POS_DECODE)
//                         RequirementDescriptor->Flags |= CM_RESOURCE_PORT_POSITIVE_DECODE;
//                     RequirementDescriptor->u.Port.MinimumAddress.QuadPart = addr64_data->Address.Minimum;
//                     RequirementDescriptor->u.Port.MaximumAddress.QuadPart = addr64_data->Address.Maximum + addr64_data->Address.AddressLength - 1;
//                     RequirementDescriptor->u.Port.Length = addr64_data->Address.AddressLength;
//                 }
//                 else
//                 {
//                     RequirementDescriptor->Type = CmResourceTypeMemory;
//                     RequirementDescriptor->ShareDisposition = CmResourceShareDeviceExclusive;
//                     RequirementDescriptor->Flags = 0;
//                     if (addr64_data->Info.Mem.WriteProtect == ACPI_READ_ONLY_MEMORY)
//                         RequirementDescriptor->Flags |= CM_RESOURCE_MEMORY_READ_ONLY;
//                     else
//                         RequirementDescriptor->Flags |= CM_RESOURCE_MEMORY_READ_WRITE;
//                     switch (addr64_data->Info.Mem.Caching)
//                     {
//                         case ACPI_CACHABLE_MEMORY: RequirementDescriptor->Flags |= CM_RESOURCE_MEMORY_CACHEABLE; break;
//                         case ACPI_WRITE_COMBINING_MEMORY: RequirementDescriptor->Flags |= CM_RESOURCE_MEMORY_COMBINEDWRITE; break;
//                         case ACPI_PREFETCHABLE_MEMORY: RequirementDescriptor->Flags |= CM_RESOURCE_MEMORY_PREFETCHABLE; break;
//                     }
//                     RequirementDescriptor->u.Memory.MinimumAddress.QuadPart = addr64_data->Address.Minimum;
//                     RequirementDescriptor->u.Memory.MaximumAddress.QuadPart = addr64_data->Address.Maximum + addr64_data->Address.AddressLength - 1;
//                     RequirementDescriptor->u.Memory.Length = addr64_data->Address.AddressLength;
//                 }
//                 RequirementDescriptor++;
//                 break;
//             }
//             case ACPI_RESOURCE_TYPE_MEMORY24:
//             {
//                 ACPI_RESOURCE_MEMORY24 *mem24_data = &resource->Data.Memory24;
//                 RequirementDescriptor->Option = CurrentRes ? 0 : IO_RESOURCE_PREFERRED;
//                 RequirementDescriptor->Type = CmResourceTypeMemory;
//                 RequirementDescriptor->ShareDisposition = CmResourceShareDeviceExclusive;
//                 RequirementDescriptor->Flags = CM_RESOURCE_MEMORY_24;
//                 if (mem24_data->WriteProtect == ACPI_READ_ONLY_MEMORY)
//                     RequirementDescriptor->Flags |= CM_RESOURCE_MEMORY_READ_ONLY;
//                 else
//                     RequirementDescriptor->Flags |= CM_RESOURCE_MEMORY_READ_WRITE;
//                 RequirementDescriptor->u.Memory.MinimumAddress.QuadPart = mem24_data->Minimum;
//                 RequirementDescriptor->u.Memory.MaximumAddress.QuadPart = mem24_data->Maximum + mem24_data->AddressLength - 1;
//                 RequirementDescriptor->u.Memory.Length = mem24_data->AddressLength;
//
//                 RequirementDescriptor++;
//                 break;
//             }
//             case ACPI_RESOURCE_TYPE_MEMORY32:
//             {
//                 ACPI_RESOURCE_MEMORY32 *mem32_data = &resource->Data.Memory32;
//                 RequirementDescriptor->Option = CurrentRes ? 0 : IO_RESOURCE_PREFERRED;
//                 RequirementDescriptor->Type = CmResourceTypeMemory;
//                 RequirementDescriptor->ShareDisposition = CmResourceShareDeviceExclusive;
//                 RequirementDescriptor->Flags = 0;
//                 if (mem32_data->WriteProtect == ACPI_READ_ONLY_MEMORY)
//                     RequirementDescriptor->Flags |= CM_RESOURCE_MEMORY_READ_ONLY;
//                 else
//                     RequirementDescriptor->Flags |= CM_RESOURCE_MEMORY_READ_WRITE;
//                 RequirementDescriptor->u.Memory.MinimumAddress.QuadPart = mem32_data->Minimum;
//                 RequirementDescriptor->u.Memory.MaximumAddress.QuadPart = mem32_data->Maximum + mem32_data->AddressLength - 1;
//                 RequirementDescriptor->u.Memory.Length = mem32_data->AddressLength;
//
//                 RequirementDescriptor++;
//                 break;
//             }
//             case ACPI_RESOURCE_TYPE_FIXED_MEMORY32:
//             {
//                 ACPI_RESOURCE_FIXED_MEMORY32 *fixedmem32_data = &resource->Data.FixedMemory32;
//                 RequirementDescriptor->Option = CurrentRes ? 0 : IO_RESOURCE_PREFERRED;
//                 RequirementDescriptor->Type = CmResourceTypeMemory;
//                 RequirementDescriptor->ShareDisposition = CmResourceShareDeviceExclusive;
//                 RequirementDescriptor->Flags = 0;
//                 if (fixedmem32_data->WriteProtect == ACPI_READ_ONLY_MEMORY)
//                     RequirementDescriptor->Flags |= CM_RESOURCE_MEMORY_READ_ONLY;
//                 else
//                     RequirementDescriptor->Flags |= CM_RESOURCE_MEMORY_READ_WRITE;
//                 RequirementDescriptor->u.Memory.MinimumAddress.QuadPart = fixedmem32_data->Address;
//                 RequirementDescriptor->u.Memory.MaximumAddress.QuadPart = fixedmem32_data->Address + fixedmem32_data->AddressLength - 1;
//                 RequirementDescriptor->u.Memory.Length = fixedmem32_data->AddressLength;
//
//                 RequirementDescriptor++;
//                 break;
//             }
//             default:
//             {
//                 break;
//             }
//         }
//         resource = ACPI_NEXT_RESOURCE(resource);
//     }
//     ExFreePoolWithTag(Buffer.Pointer, u32::from_be_bytes(*b"BpcA"));
//
//     Irp->IoStatus.Information = (ULONG_PTR)RequirementsList;
//
//     return STATUS_SUCCESS;
// }
//
// #ifndef UNIT_TEST
// NTSTATUS
// Bus_PDO_QueryDeviceRelations(
//      PPDO_DEVICE_DATA     DeviceData,
//       PIRP   Irp )
// /*++
//
// Routine Description:
//
//     The PnP Manager sends this IRP to gather information about
//     devices with a relationship to the specified device.
//     Bus drivers must handle this request for TargetDeviceRelation
//     for their child devices (child PDOs).
//
//     If a driver returns relations in response to this IRP,
//     it allocates a DEVICE_RELATIONS structure from paged
//     memory containing a count and the appropriate number of
//     device object pointers. The PnP Manager frees the structure
//     when it is no longer needed. If a driver replaces a
//     DEVICE_RELATIONS structure allocated by another driver,
//     it must free the previous structure.
//
//     A driver must reference the PDO of any device that it
//     reports in this IRP (ObReferenceObject). The PnP Manager
//     removes the reference when appropriate.
//
// Arguments:
//
//     DeviceData - Pointer to the PDO's device extension.
//     Irp          - Pointer to the irp.
//
// Return Value:
//
//     NT STATUS
//
// --*/
// {
//
//     PIO_STACK_LOCATION   stack;
//     PDEVICE_RELATIONS deviceRelations;
//     NTSTATUS status;
//
//     PAGED_CODE ();
//
//     stack = IoGetCurrentIrpStackLocation (Irp);
//
//     switch (stack->Parameters.QueryDeviceRelations.Type) {
//
//     case TargetDeviceRelation:
//
//         deviceRelations = (PDEVICE_RELATIONS) Irp->IoStatus.Information;
//         if (deviceRelations) {
//             //
//             // Only PDO can handle this request. Somebody above
//             // is not playing by rule.
//             //
//             ASSERTMSG("Someone above is handling TargetDeviceRelation\n", !deviceRelations);
//         }
//
//         deviceRelations = ExAllocatePoolWithTag(PagedPool,
//                                                 sizeof(DEVICE_RELATIONS),
//                                                 u32::from_be_bytes(*b"IpcA"));
//         if (!deviceRelations) {
//                 status = STATUS_INSUFFICIENT_RESOURCES;
//                 break;
//         }
//
//         //
//         // There is only one PDO pointer in the structure
//         // for this relation type. The PnP Manager removes
//         // the reference to the PDO when the driver or application
//         // un-registers for notification on the device.
//         //
//
//         deviceRelations->Count = 1;
//         deviceRelations->Objects[0] = DeviceData->Common.Self;
//         ObReferenceObject(DeviceData->Common.Self);
//
//         status = STATUS_SUCCESS;
//         Irp->IoStatus.Information = (ULONG_PTR) deviceRelations;
//         break;
//
//     case BusRelations: // Not handled by PDO
//     case EjectionRelations: // optional for PDO
//     case RemovalRelations: // // optional for PDO
//     default:
//         status = Irp->IoStatus.Status;
//     }
//
//     return status;
// }
//
// NTSTATUS
// Bus_PDO_QueryBusInformation(
//      PPDO_DEVICE_DATA     DeviceData,
//       PIRP   Irp )
// /*++
//
// Routine Description:
//
//     The PnP Manager uses this IRP to request the type and
//     instance number of a device's parent bus. Bus drivers
//     should handle this request for their child devices (PDOs).
//
// Arguments:
//
//     DeviceData - Pointer to the PDO's device extension.
//     Irp          - Pointer to the irp.
//
// Return Value:
//
//     NT STATUS
//
// --*/
// {
//
//     PPNP_BUS_INFORMATION busInfo;
//
//     PAGED_CODE ();
//
//     busInfo = ExAllocatePoolWithTag(PagedPool,
//                                     sizeof(PNP_BUS_INFORMATION),
//                                     u32::from_be_bytes(*b"IpcA"));
//
//     if (busInfo == NULL) {
//       return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     busInfo->BusTypeGuid = GUID_ACPI_INTERFACE_STANDARD;
//
//     busInfo->LegacyBusType = InternalPowerBus;
//
//     busInfo->BusNumber = 0; //fixme
//
//     Irp->IoStatus.Information = (ULONG_PTR)busInfo;
//
//     return STATUS_SUCCESS;
// }
//
//
// NTSTATUS
// Bus_GetDeviceCapabilities(
//       PDEVICE_OBJECT          DeviceObject,
//       PDEVICE_CAPABILITIES    DeviceCapabilities
//     )
// {
//     IO_STATUS_BLOCK     ioStatus;
//     KEVENT              pnpEvent;
//     NTSTATUS            status;
//     PDEVICE_OBJECT      targetObject;
//     PIO_STACK_LOCATION  irpStack;
//     PIRP                pnpIrp;
//
//     PAGED_CODE();
//
//     //
//     // Initialize the capabilities that we will send down
//     //
//     RtlZeroMemory( DeviceCapabilities, sizeof(DEVICE_CAPABILITIES) );
//     DeviceCapabilities->Size = sizeof(DEVICE_CAPABILITIES);
//     DeviceCapabilities->Version = 1;
//     DeviceCapabilities->Address = -1;
//     DeviceCapabilities->UINumber = -1;
//
//     //
//     // Initialize the event
//     //
//     KeInitializeEvent( &pnpEvent, NotificationEvent, FALSE );
//
//     targetObject = IoGetAttachedDeviceReference( DeviceObject );
//
//     //
//     // Build an Irp
//     //
//     pnpIrp = IoBuildSynchronousFsdRequest(
//         IRP_MJ_PNP,
//         targetObject,
//         NULL,
//         0,
//         NULL,
//         &pnpEvent,
//         &ioStatus
//         );
//     if (pnpIrp == NULL) {
//
//         status = STATUS_INSUFFICIENT_RESOURCES;
//         goto GetDeviceCapabilitiesExit;
//
//     }
//
//     //
//     // Pnp Irps all begin life as STATUS_NOT_SUPPORTED;
//     //
//     pnpIrp->IoStatus.Status = STATUS_NOT_SUPPORTED;
//
//     //
//     // Get the top of stack
//     //
//     irpStack = IoGetNextIrpStackLocation( pnpIrp );
//
//     //
//     // Set the top of stack
//     //
//     RtlZeroMemory( irpStack, sizeof(IO_STACK_LOCATION ) );
//     irpStack->MajorFunction = IRP_MJ_PNP;
//     irpStack->MinorFunction = IRP_MN_QUERY_CAPABILITIES;
//     irpStack->Parameters.DeviceCapabilities.Capabilities = DeviceCapabilities;
//
//     //
//     // Call the driver
//     //
//     status = IoCallDriver( targetObject, pnpIrp );
//     if (status == STATUS_PENDING) {
//
//         //
//         // Block until the irp comes back.
//         // Important thing to note here is when you allocate
//         // the memory for an event in the stack you must do a
//         // KernelMode wait instead of UserMode to prevent
//         // the stack from getting paged out.
//         //
//
//         KeWaitForSingleObject(
//             &pnpEvent,
//             Executive,
//             KernelMode,
//             FALSE,
//             NULL
//             );
//         status = ioStatus.Status;
//
//     }
//
// GetDeviceCapabilitiesExit:
//     //
//     // Done with reference
//     //
//     ObDereferenceObject( targetObject );
//
//     //
//     // Done
//     //
//     return status;
//
// }
// #endif /* UNIT_TEST */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
