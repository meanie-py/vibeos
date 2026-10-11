//! interface.c → Rust (ACPI Phase 3b glue leaf). Cohab: #[no_mangle] symbols
//! replace the C file in acpi.sys at link time. ACPICA stays C (libacpica.a).
//!
//! This file is the ACPI_INTERFACE_STANDARD provider: Bus_PDO_QueryInterface
//! fills the caller's ACPI_INTERFACE_STANDARD with the (mostly UNIMPLEMENTED)
//! AcpiInterface* GPE/notification stubs.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, unused)]

use crate::shared::*; // crate::types::* (PPDO_DEVICE_DATA etc.)
// STATUS_* constants + NTSTATUS live in base/constants (NOT re-exported by
// crate::types). Mirror buspdo.rs: glob base+constants, import ddk by name.
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use core::ffi::{c_char, c_int, c_void};

// Kernel + DDK types. ACPI_INTERFACE_STANDARD / PACPI_INTERFACE_STANDARD,
// IO_STACK_LOCATION + the QueryInterface union arm, GUID, KINTERRUPT_MODE,
// PGPE_SERVICE_ROUTINE, PDEVICE_NOTIFY_CALLBACK, RtlCompareMemory all live here.
use ntkrnl_sys::ddk::{
    GUID, KINTERRUPT_MODE, PDEVICE_NOTIFY_CALLBACK, PDEVICE_OBJECT, PGPE_SERVICE_ROUTINE, PIRP,
    PIO_STACK_LOCATION, ACPI_INTERFACE_STANDARD, PACPI_INTERFACE_STANDARD, RtlCompareMemory,
    BOOLEAN, ULONG, SIZE_T,
};
// IoGetCurrentIrpStackLocation is FORCEINLINE in the DDK (no exported symbol) —
// must come from the ntkrnl-sys inline impl or the cohab link fails.
use ntkrnl_sys::inline::tier1::IoGetCurrentIrpStackLocation;

// `UNIMPLEMENTED` is a no-op marker macro in the ReactOS DDK.
macro_rules! UNIMPLEMENTED {
    () => {};
}

// GUID_ACPI_INTERFACE_STANDARD lives in wdmguid (a static GUID exported to the
// driver). It is a plain C global → extern "C".
extern "C" {
    static GUID_ACPI_INTERFACE_STANDARD: GUID;
}

// ---------------------------------------------------------------------------
// AcpiInterface* — NTAPI stubs (extern "system"). All but
// AcpiInterfaceNotificationsRegister return STATUS_NOT_IMPLEMENTED; the VOID
// ones simply return. Mirrors interface.c exactly (UNIMPLEMENTED is a no-op).
// ---------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "system" fn AcpiInterfaceReference(Context: *mut c_void) {
    UNIMPLEMENTED!();
}

#[no_mangle]
pub unsafe extern "system" fn AcpiInterfaceDereference(Context: *mut c_void) {
    UNIMPLEMENTED!();
}

#[no_mangle]
pub unsafe extern "system" fn AcpiInterfaceConnectVector(
    Context: PDEVICE_OBJECT,
    GpeNumber: ULONG,
    Mode: KINTERRUPT_MODE,
    Shareable: BOOLEAN,
    ServiceRoutine: PGPE_SERVICE_ROUTINE,
    ServiceContext: *mut c_void,
    ObjectContext: *mut c_void,
) -> NTSTATUS {
    UNIMPLEMENTED!();

    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn AcpiInterfaceDisconnectVector(
    ObjectContext: *mut c_void,
) -> NTSTATUS {
    UNIMPLEMENTED!();

    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn AcpiInterfaceEnableEvent(
    Context: PDEVICE_OBJECT,
    ObjectContext: *mut c_void,
) -> NTSTATUS {
    UNIMPLEMENTED!();

    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn AcpiInterfaceDisableEvent(
    Context: PDEVICE_OBJECT,
    ObjectContext: *mut c_void,
) -> NTSTATUS {
    UNIMPLEMENTED!();

    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn AcpiInterfaceClearStatus(
    Context: PDEVICE_OBJECT,
    ObjectContext: *mut c_void,
) -> NTSTATUS {
    UNIMPLEMENTED!();

    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn AcpiInterfaceNotificationsRegister(
    Context: PDEVICE_OBJECT,
    NotificationHandler: PDEVICE_NOTIFY_CALLBACK,
    NotificationContext: *mut c_void,
) -> NTSTATUS {
    UNIMPLEMENTED!();

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn AcpiInterfaceNotificationsUnregister(
    Context: PDEVICE_OBJECT,
    NotificationHandler: PDEVICE_NOTIFY_CALLBACK,
) {
    UNIMPLEMENTED!();
}

// ---------------------------------------------------------------------------
// Bus_PDO_QueryInterface — plain C (no NTAPI per acpisys.h) → extern "C".
// Fills the caller's ACPI_INTERFACE_STANDARD with the stubs above when the
// requested interface GUID matches GUID_ACPI_INTERFACE_STANDARD.
// ---------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "C" fn Bus_PDO_QueryInterface(
    DeviceData: PPDO_DEVICE_DATA,
    Irp: PIRP,
) -> NTSTATUS {
    let IrpSp: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);

    // union path: IO_STACK_LOCATION.Parameters.QueryInterface.{Version,
    // InterfaceType,Size,Interface}. The QueryInterface arm is a named union
    // member in bindgen (not __bindgen_anon), so direct field access works.
    if (*IrpSp).Parameters.QueryInterface.Version != 1 {
        DPRINT1!(
            "Invalid version number: %d\n",
            (*IrpSp).Parameters.QueryInterface.Version as core::ffi::c_uint
        );
        return STATUS_INVALID_PARAMETER;
    }

    // VibeOS: PCI interrupt routing for a root bridge (pciroute.rs).
    if RtlCompareMemory(
        (*IrpSp).Parameters.QueryInterface.InterfaceType as *const c_void,
        core::ptr::addr_of!(crate::pciroute::GUID_VIBEOS_PCI_IRQ_ROUTING) as *const c_void,
        core::mem::size_of::<GUID>() as SIZE_T,
    ) == core::mem::size_of::<GUID>() as SIZE_T
    {
        return crate::pciroute::query_routing_interface(
            (*DeviceData).AcpiHandle,
            (*IrpSp).Parameters.QueryInterface.Interface as *mut c_void,
            (*IrpSp).Parameters.QueryInterface.Size,
        );
    }

    // VibeOS: config-space access for a PCI root bridge, which pcix.sys needs (pcibus.rs).
    if RtlCompareMemory(
        (*IrpSp).Parameters.QueryInterface.InterfaceType as *const c_void,
        core::ptr::addr_of!(crate::pcibus::GUID_PCI_BUS_INTERFACE_STANDARD) as *const c_void,
        core::mem::size_of::<GUID>() as SIZE_T,
    ) == core::mem::size_of::<GUID>() as SIZE_T
        && crate::pcibus::is_pci_root((*DeviceData).HardwareIDs)
    {
        return crate::pcibus::query_pci_bus_interface(
            DeviceData as *mut c_void,
            (*IrpSp).Parameters.QueryInterface.Interface as *mut c_void,
            (*IrpSp).Parameters.QueryInterface.Size,
        );
    }

    if RtlCompareMemory(
        (*IrpSp).Parameters.QueryInterface.InterfaceType as *const c_void,
        core::ptr::addr_of!(GUID_ACPI_INTERFACE_STANDARD) as *const c_void,
        core::mem::size_of::<GUID>() as SIZE_T,
    ) == core::mem::size_of::<GUID>() as SIZE_T
    {
        DPRINT!("GUID_ACPI_INTERFACE_STANDARD\n");

        if (((*IrpSp).Parameters.QueryInterface.Size as usize)
            < core::mem::size_of::<ACPI_INTERFACE_STANDARD>())
        {
            DPRINT1!(
                "Buffer too small! (%d)\n",
                (*IrpSp).Parameters.QueryInterface.Size as core::ffi::c_uint
            );
            return STATUS_BUFFER_TOO_SMALL;
        }

        let AcpiInterface: PACPI_INTERFACE_STANDARD =
            (*IrpSp).Parameters.QueryInterface.Interface as PACPI_INTERFACE_STANDARD;

        // The dispatch-table fields are bindgen `Option<unsafe extern "stdcall"
        // fn ...>` (NTAPI → stdcall on i386, cdecl on amd64/arm64; the alias
        // resolves per arch). `Some(fn as _)` coerces our extern "system" fns
        // to the field's fn-pointer type.
        // The dispatch-table fields are bindgen `Option<extern "stdcall" fn>` on i386,
        // `extern "C"` on amd64/arm64. Our fns are `extern "system"` (== the same ABI per
        // arch) but rustc types the "system"/"stdcall" abi-strings as distinct fn-pointer
        // types, so coerce via transmute — the target type is inferred from each field.
        (*AcpiInterface).InterfaceReference =
            core::mem::transmute(AcpiInterfaceReference as *const ());
        (*AcpiInterface).InterfaceDereference =
            core::mem::transmute(AcpiInterfaceDereference as *const ());
        (*AcpiInterface).GpeConnectVector =
            core::mem::transmute(AcpiInterfaceConnectVector as *const ());
        (*AcpiInterface).GpeDisconnectVector =
            core::mem::transmute(AcpiInterfaceDisconnectVector as *const ());
        (*AcpiInterface).GpeEnableEvent =
            core::mem::transmute(AcpiInterfaceEnableEvent as *const ());
        (*AcpiInterface).GpeDisableEvent =
            core::mem::transmute(AcpiInterfaceDisableEvent as *const ());
        (*AcpiInterface).GpeClearStatus =
            core::mem::transmute(AcpiInterfaceClearStatus as *const ());
        (*AcpiInterface).RegisterForDeviceNotifications =
            core::mem::transmute(AcpiInterfaceNotificationsRegister as *const ());
        (*AcpiInterface).UnregisterForDeviceNotifications =
            core::mem::transmute(AcpiInterfaceNotificationsUnregister as *const ());

        STATUS_SUCCESS
    } else {
        DPRINT1!("Invalid GUID\n");
        STATUS_NOT_SUPPORTED
    }
}
