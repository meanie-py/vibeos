//! Driver-private types shared across the ACPI driver glue modules — the exact
//! layouts from drivers/bus/acpi/include/acpisys.h. Hand-written (not taken from
//! the rustconv output, whose LLM-generated type preamble dropped/reordered
//! struct fields) so the layouts match the C the not-yet-flipped glue still uses.
//!
//! Field types come from ntkrnl-sys (kernel) + acpica-sys (ACPI), keeping a
//! single coherent type universe with the functions the glue calls.

#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

use ntkrnl_sys::ddk::{
    PDEVICE_OBJECT, BOOLEAN, ULONG, PWCHAR, LIST_ENTRY, FAST_MUTEX, UNICODE_STRING,
    SYSTEM_POWER_STATE, DEVICE_POWER_STATE,
};
use acpica_sys::ACPI_HANDLE;

// _DEVICE_PNP_STATE — C enum (4 bytes). Use a 4-byte alias + consts so the
// containing structs lay out identically.
pub type DEVICE_PNP_STATE = i32;
pub const NotStarted: DEVICE_PNP_STATE = 0;
pub const Started: DEVICE_PNP_STATE = 1;
pub const StopPending: DEVICE_PNP_STATE = 2;
pub const Stopped: DEVICE_PNP_STATE = 3;
pub const RemovalPending: DEVICE_PNP_STATE = 4;
pub const UnKnown: DEVICE_PNP_STATE = 5;

/// Common header for the PDO and FDO device extensions.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct _COMMON_DEVICE_DATA {
    pub Self_: PDEVICE_OBJECT,
    pub IsFDO: BOOLEAN,
    pub DevicePnPState: DEVICE_PNP_STATE,
    pub PreviousPnPState: DEVICE_PNP_STATE,
    pub SystemPowerState: SYSTEM_POWER_STATE,
    pub DevicePowerState: DEVICE_POWER_STATE,
}
pub type COMMON_DEVICE_DATA = _COMMON_DEVICE_DATA;
pub type PCOMMON_DEVICE_DATA = *mut _COMMON_DEVICE_DATA;

/// PDO device extension (a child ACPI device).
#[repr(C)]
#[derive(Copy, Clone)]
pub struct _PDO_DEVICE_DATA {
    pub Common: COMMON_DEVICE_DATA,
    pub AcpiHandle: ACPI_HANDLE,
    pub ParentFdo: PDEVICE_OBJECT,
    pub HardwareIDs: PWCHAR,
    pub DockDevice: BOOLEAN,
    pub Link: LIST_ENTRY,
    pub InterfaceRefCount: ULONG,
    pub InterfaceName: UNICODE_STRING,
}
pub type PDO_DEVICE_DATA = _PDO_DEVICE_DATA;
pub type PPDO_DEVICE_DATA = *mut _PDO_DEVICE_DATA;

/// FDO device extension (the ACPI bus itself).
#[repr(C)]
#[derive(Copy, Clone)]
pub struct _FDO_DEVICE_DATA {
    pub Common: COMMON_DEVICE_DATA,
    pub UnderlyingPDO: PDEVICE_OBJECT,
    pub NextLowerDriver: PDEVICE_OBJECT,
    pub ListOfPDOs: LIST_ENTRY,
    pub NumPDOs: ULONG,
    pub Mutex: FAST_MUTEX,
}
pub type FDO_DEVICE_DATA = _FDO_DEVICE_DATA;
pub type PFDO_DEVICE_DATA = *mut _FDO_DEVICE_DATA;
