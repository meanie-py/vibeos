//! power.c → Rust (ACPI Phase 3b glue leaf). Cohab: #[no_mangle] symbols
//! replace the C file in acpi.sys at link time. ACPICA stays C (libacpica.a).
//!
//! Top-level IRP_MJ_POWER dispatch (NOT busmgr/power.c). Bus_Power routes to
//! Bus_FDO_Power / Bus_PDO_Power based on commonData->IsFDO.
//!
//! NOTE: PowerMinorFunctionString / DbgSystemPowerString / DbgDevicePowerString
//! are guarded by `#if !defined(NDEBUG) || defined(_MSC_VER)` in the C. This is
//! a non-MSVC build with NDEBUG defined, so those functions are NOT compiled and
//! NOT linked — they are intentionally OMITTED here.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, unused)]

use crate::shared::*; // crate::types::* (P*_DEVICE_DATA, ACPI_STATUS, AE_OK, acpi_* externs)
// STATUS_* + IO_NO_INCREMENT + NTSTATUS live in base/constants (NOT re-exported
// by crate::types). IRP_MN_* / IRP_MJ_POWER live in driver_ddk. Mirror buspdo.rs.
use ntkrnl_sys::base::*;
// One declaration per external symbol: the variant matching the C prototype.
use crate::pnp::PoSetPowerState;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::{
    IRP_MN_SET_POWER, IRP_MN_QUERY_POWER, IRP_MN_WAIT_WAKE, IRP_MN_POWER_SEQUENCE, IRP_MJ_POWER,
    PoStartNextPowerIrp, PoCallDriver, IoCompleteRequest,
};
use core::ffi::{c_char, c_int, c_void};

use ntkrnl_sys::ddk::{
    PDEVICE_OBJECT, PIRP, PIO_STACK_LOCATION, POWER_STATE, POWER_STATE_TYPE, SYSTEM_POWER_STATE,
    DEVICE_POWER_STATE, ULONG, BOOLEAN, UINT32,
};

// IoGetCurrentIrpStackLocation + IoSkipCurrentIrpStackLocation are FORCEINLINE
// in the DDK (no exported symbol) — use the ntkrnl-sys inline impls.
use ntkrnl_sys::inline::tier1::{IoGetCurrentIrpStackLocation, IoSkipCurrentIrpStackLocation};

// ---- Power-state enum-variant constants: bindgen prefixes them
// (`_<ENUM>_<Name>`), so the unprefixed spellings the C uses don't resolve from
// ddk. Re-alias here (values are bindgen's, verified against the ddk enums). ----
use ntkrnl_sys::ddk::{
    _POWER_STATE_TYPE_SystemPowerState as SystemPowerState,
    _POWER_STATE_TYPE_DevicePowerState as DevicePowerState,
    _SYSTEM_POWER_STATE_PowerSystemSleeping1 as PowerSystemSleeping1,
    _SYSTEM_POWER_STATE_PowerSystemSleeping2 as PowerSystemSleeping2,
    _SYSTEM_POWER_STATE_PowerSystemSleeping3 as PowerSystemSleeping3,
    _SYSTEM_POWER_STATE_PowerSystemHibernate as PowerSystemHibernate,
    _SYSTEM_POWER_STATE_PowerSystemShutdown as PowerSystemShutdown,
    _DEVICE_POWER_STATE_PowerDeviceD0 as PowerDeviceD0,
    _DEVICE_POWER_STATE_PowerDeviceD1 as PowerDeviceD1,
    _DEVICE_POWER_STATE_PowerDeviceD2 as PowerDeviceD2,
    _DEVICE_POWER_STATE_PowerDeviceD3 as PowerDeviceD3,
};

macro_rules! ASSERT {
    ($($t:tt)*) => {};
}

// ---- ACPI sleep/device-state constants (actypes.h). ACPI_STATE_D0..D3 already
// defined in bus.rs; redefine the S-states + UNKNOWN here (file-local; values
// per actypes.h ACPI_STATE_S0..S5 / ACPI_STATE_UNKNOWN). ----
const ACPI_STATE_S1: ULONG = 1; // FIXME(integrator): confirm value (actypes.h ACPI_STATE_S1)
const ACPI_STATE_S2: ULONG = 2; // FIXME(integrator): confirm value (actypes.h ACPI_STATE_S2)
const ACPI_STATE_S3: ULONG = 3; // FIXME(integrator): confirm value (actypes.h ACPI_STATE_S3)
const ACPI_STATE_S4: ULONG = 4; // FIXME(integrator): confirm value (actypes.h ACPI_STATE_S4)
const ACPI_STATE_S5: ULONG = 5; // FIXME(integrator): confirm value (actypes.h ACPI_STATE_S5)
const ACPI_STATE_UNKNOWN: ULONG = 0xFF; // actypes.h ACPI_STATE_UNKNOWN (matches bus.rs)
const ACPI_STATE_D0: ULONG = 0; // actypes.h (matches bus.rs/buspdo.rs)
const ACPI_STATE_D1: ULONG = 1; // FIXME(integrator): confirm value (actypes.h ACPI_STATE_D1)
const ACPI_STATE_D2: ULONG = 2; // FIXME(integrator): confirm value (actypes.h ACPI_STATE_D2)
const ACPI_STATE_D3: ULONG = 3; // actypes.h (matches bus.rs/buspdo.rs)

// ACPI_SUCCESS(s) == (s == AE_OK). ACPI_STATUS success is 0 (AE_OK).
#[inline]
unsafe fn ACPI_SUCCESS(s: ACPI_STATUS) -> bool {
    s == AE_OK
}

// ---- Sibling acpi-rust #[no_mangle] functions this file CALLS.
// acpi_suspend (busmgr/system.c), acpi_bus_power_manageable + acpi_bus_set_power
// (bus.rs) are all extern "C" (Linux-style acpi_*). They are declared in
// crate::types (re-exported through crate::shared::*) with these exact
// signatures, so no local extern block is needed — see notes in report.
//   acpi_suspend(state: UINT32) -> ACPI_STATUS
//   acpi_bus_power_manageable(handle: ACPI_HANDLE) -> BOOLEAN
//   acpi_bus_set_power(handle: ACPI_HANDLE, state: c_int) -> c_int

// ---- PoSetPowerState: kernel export, NTAPI (stdcall on i386, cdecl on
// amd64/arm64 — `extern "system"` resolves per arch). Returns POWER_STATE by
// value (the C glue ignores it). Signature: pofuncs.h. ----
extern "system" {
}

// ---------------------------------------------------------------------------
// Bus_Power — NTAPI top-level IRP_MJ_POWER dispatch → extern "system".
// ---------------------------------------------------------------------------
#[no_mangle]
pub unsafe extern "system" fn Bus_Power(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut status: NTSTATUS;
    let irpStack: PIO_STACK_LOCATION;
    let commonData: PCOMMON_DEVICE_DATA;

    status = STATUS_SUCCESS;
    irpStack = IoGetCurrentIrpStackLocation(Irp);
    ASSERT!(IRP_MJ_POWER == (*irpStack).MajorFunction);

    commonData = (*DeviceObject).DeviceExtension as PCOMMON_DEVICE_DATA;

    if (*commonData).IsFDO != 0 {
        DPRINT!(
            "FDO %s IRP:0x%p %s %s\n",
            PowerMinorFunctionString((*irpStack).MinorFunction),
            Irp,
            DbgSystemPowerString((*commonData).SystemPowerState),
            DbgDevicePowerString((*commonData).DevicePowerState)
        );

        status = Bus_FDO_Power(
            (*DeviceObject).DeviceExtension as PFDO_DEVICE_DATA,
            Irp,
        );
    } else {
        DPRINT!(
            "PDO %s IRP:0x%p %s %s\n",
            PowerMinorFunctionString((*irpStack).MinorFunction),
            Irp,
            DbgSystemPowerString((*commonData).SystemPowerState),
            DbgDevicePowerString((*commonData).DevicePowerState)
        );

        status = Bus_PDO_Power(
            (*DeviceObject).DeviceExtension as PPDO_DEVICE_DATA,
            Irp,
        );
    }

    status
}

// ---------------------------------------------------------------------------
// Bus_FDO_Power — plain C → extern "C".
// ---------------------------------------------------------------------------
#[no_mangle]
pub unsafe extern "C" fn Bus_FDO_Power(
    Data: PFDO_DEVICE_DATA,
    Irp: PIRP,
) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_SUCCESS;
    let powerState: POWER_STATE;
    let powerType: POWER_STATE_TYPE;
    let stack: PIO_STACK_LOCATION;
    let mut AcpiState: ULONG;
    let mut AcpiStatus: ACPI_STATUS;
    let oldPowerState: SYSTEM_POWER_STATE;

    stack = IoGetCurrentIrpStackLocation(Irp);
    // union path: Parameters.Power.{Type,State}. Power arm has Type/State as
    // direct named fields (a leading __bindgen_anon_1 holds SystemContext/
    // SystemPowerStateContext, irrelevant here).
    powerType = (*stack).Parameters.Power.Type;
    powerState = (*stack).Parameters.Power.State;

    if (*stack).MinorFunction == IRP_MN_SET_POWER {
        DPRINT!("\tRequest to set ... state to ...\n");

        if powerType == SystemPowerState {
            // POWER_STATE is a union; .SystemState reads SYSTEM_POWER_STATE.
            AcpiState = match powerState.SystemState {
                x if x == PowerSystemSleeping1 => ACPI_STATE_S1,
                x if x == PowerSystemSleeping2 => ACPI_STATE_S2,
                x if x == PowerSystemSleeping3 => ACPI_STATE_S3,
                x if x == PowerSystemHibernate => ACPI_STATE_S4,
                x if x == PowerSystemShutdown => ACPI_STATE_S5,
                _ => {
                    ASSERT!(false);
                    ACPI_STATE_UNKNOWN
                }
            };

            oldPowerState = (*Data).Common.SystemPowerState;
            (*Data).Common.SystemPowerState = powerState.SystemState;
            AcpiStatus = acpi_suspend(AcpiState as UINT32);
            if !ACPI_SUCCESS(AcpiStatus) {
                DPRINT1!(
                    "Failed to enter sleep state %d (Status 0x%X)\n",
                    AcpiState,
                    AcpiStatus
                );
                (*Data).Common.SystemPowerState = oldPowerState;
                status = STATUS_UNSUCCESSFUL;
            }
        }
    }
    PoStartNextPowerIrp(Irp);
    IoSkipCurrentIrpStackLocation(Irp);
    status = PoCallDriver((*Data).NextLowerDriver, Irp);
    status
}

// ---------------------------------------------------------------------------
// Bus_PDO_Power — plain C → extern "C".
// ---------------------------------------------------------------------------
#[no_mangle]
pub unsafe extern "C" fn Bus_PDO_Power(
    PdoData: PPDO_DEVICE_DATA,
    Irp: PIRP,
) -> NTSTATUS {
    let mut status: NTSTATUS;
    let stack: PIO_STACK_LOCATION;
    let powerState: POWER_STATE;
    let powerType: POWER_STATE_TYPE;
    let mut error: ULONG;

    stack = IoGetCurrentIrpStackLocation(Irp);
    powerType = (*stack).Parameters.Power.Type;
    powerState = (*stack).Parameters.Power.State;

    // NOTE: the C `switch` cases below use IRP_MN_* (u8 in driver_ddk). Rust
    // `match` on those consts is fine since they're distinct constants. Default
    // arm covers IRP_MN_WAIT_WAKE / IRP_MN_POWER_SEQUENCE / anything else.
    if (*stack).MinorFunction == IRP_MN_SET_POWER {
        DPRINT!("\tSetting ... power state to ...\n");

        if powerType == DevicePowerState {
            if (*PdoData).AcpiHandle.is_null()
                || acpi_bus_power_manageable((*PdoData).AcpiHandle) == 0
            {
                PoSetPowerState((*PdoData).Common.Self_, DevicePowerState, powerState);
                (*PdoData).Common.DevicePowerState = powerState.DeviceState;
                status = STATUS_SUCCESS;
            } else {
                error = match powerState.DeviceState {
                    x if x == PowerDeviceD0 => {
                        acpi_bus_set_power((*PdoData).AcpiHandle, ACPI_STATE_D0 as c_int) as ULONG
                    }
                    x if x == PowerDeviceD1 => {
                        acpi_bus_set_power((*PdoData).AcpiHandle, ACPI_STATE_D1 as c_int) as ULONG
                    }
                    x if x == PowerDeviceD2 => {
                        acpi_bus_set_power((*PdoData).AcpiHandle, ACPI_STATE_D2 as c_int) as ULONG
                    }
                    x if x == PowerDeviceD3 => {
                        acpi_bus_set_power((*PdoData).AcpiHandle, ACPI_STATE_D3 as c_int) as ULONG
                    }
                    _ => 0,
                };

                // C: ACPI_SUCCESS(error) where error is the int from
                // acpi_bus_set_power — faithful: pass it through as ACPI_STATUS.
                if ACPI_SUCCESS(error as ACPI_STATUS) {
                    PoSetPowerState((*PdoData).Common.Self_, DevicePowerState, powerState);
                    (*PdoData).Common.DevicePowerState = powerState.DeviceState;
                    status = STATUS_SUCCESS;
                } else {
                    status = STATUS_UNSUCCESSFUL;
                }
            }
        } else if powerType == SystemPowerState {
            (*PdoData).Common.SystemPowerState = powerState.SystemState;
            status = STATUS_SUCCESS;
        } else {
            status = STATUS_NOT_SUPPORTED;
        }
    } else if (*stack).MinorFunction == IRP_MN_QUERY_POWER {
        status = STATUS_SUCCESS;
    } else {
        // IRP_MN_WAIT_WAKE (not supported: root-enumerated, parent PnP manager
        // doesn't support wait-wake), IRP_MN_POWER_SEQUENCE, and default.
        status = STATUS_NOT_SUPPORTED;
    }

    if status != STATUS_NOT_SUPPORTED {
        (*Irp).IoStatus.__bindgen_anon_1.Status = status;
    }

    PoStartNextPowerIrp(Irp);
    status = (*Irp).IoStatus.__bindgen_anon_1.Status;
    IoCompleteRequest(Irp, IO_NO_INCREMENT);

    status
}

// PowerMinorFunctionString / DbgSystemPowerString / DbgDevicePowerString are
// under `#if !defined(NDEBUG) || defined(_MSC_VER)` → NOT compiled in this
// (non-MSVC, NDEBUG) build, and NOT linked. The DPRINT! call sites above are
// no-ops, so these debug-string helpers are intentionally OMITTED.
