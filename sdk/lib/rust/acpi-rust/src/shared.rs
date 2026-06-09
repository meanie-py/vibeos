//! Friendly aliases over the bindgen-generated driver-private types (`crate::types`).
//!
//! `types.rs` (gen_driver_types.sh acpi) is the layout source of truth — the exact
//! acpisys.h device extensions (COMMON/PDO/FDO_DEVICE_DATA) + acpi_bus.h Linux-style
//! `acpi_device*` structs, with every ACPI_* type resolved to `acpica-sys` and every
//! kernel type to `ntkrnl-sys` (single identity, no duplicate layouts). This module
//! re-exports all of it and adds the friendly `DEVICE_PNP_STATE` constant names the
//! translated glue uses — bindgen emits them as `_DEVICE_PNP_STATE_<Name>`.

#![allow(non_upper_case_globals)]

pub use crate::types::*;

pub const NotStarted: DEVICE_PNP_STATE = _DEVICE_PNP_STATE_NotStarted;
pub const Started: DEVICE_PNP_STATE = _DEVICE_PNP_STATE_Started;
pub const StopPending: DEVICE_PNP_STATE = _DEVICE_PNP_STATE_StopPending;
pub const Stopped: DEVICE_PNP_STATE = _DEVICE_PNP_STATE_Stopped;
pub const RemovalPending: DEVICE_PNP_STATE = _DEVICE_PNP_STATE_RemovalPending;
pub const UnKnown: DEVICE_PNP_STATE = _DEVICE_PNP_STATE_UnKnown;

// ---- ACPICA glue constants bindgen can't emit (cast-macros / #defines) ----
// ACPICA exception codes (acexcep.h): EXCEP_ENV(c)=c|0x0000, EXCEP_PGM(c)=c|0x1000.
// These are the exact values — the c2rust/LLM preamble had AE_BUFFER_OVERFLOW wrong
// (0x3001 vs correct 0x000B), so define them centrally here, once, correctly.
pub const AE_OK: ACPI_STATUS = 0x0000;
pub const AE_NOT_FOUND: ACPI_STATUS = 0x0005; // EXCEP_ENV(0x0005)
pub const AE_BUFFER_OVERFLOW: ACPI_STATUS = 0x000B; // EXCEP_ENV(0x000B)
pub const AE_BAD_PARAMETER: ACPI_STATUS = 0x1001; // EXCEP_PGM(0x0001)

pub const ACPI_BUS_COMPONENT: u32 = 0x00010000;
// ACPI_ROOT_OBJECT = (ACPI_HANDLE) ACPI_MAX_PTR (all-ones pointer, width per arch).
pub const ACPI_ROOT_OBJECT: ACPI_HANDLE = usize::MAX as ACPI_HANDLE;

// The glue uses the uppercase LIST_HEAD spelling; bindgen emits `list_head`.
pub type LIST_HEAD = list_head;
