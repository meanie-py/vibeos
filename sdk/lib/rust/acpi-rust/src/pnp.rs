//! pnp.c → Rust (ACPI Phase 3b glue leaf). Cohab: #[no_mangle] symbols
//! replace the C file in acpi.sys at link time. ACPICA stays C (libacpica.a).
//!
//! Bus FDO/PDO PnP dispatch: Bus_PnP, Bus_FDO_PnP, Bus_StartFdo,
//! Bus_SendIrpSynchronously, Bus_CompletionRoutine, Bus_DestroyPdo,
//! Bus_InitializePdo.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, unused)]

use crate::shared::*;
use core::ffi::{c_char, c_int, c_void};
use core::ptr::{addr_of_mut, null_mut};

// Kernel types (DEVICE_OBJECT, IRP, IO_STACK_LOCATION, NTSTATUS, LIST_ENTRY,
// PLIST_ENTRY, DEVICE_RELATIONS, PDEVICE_RELATIONS, KEVENT, PDEVICE_OBJECT,
// POWER_STATE, DEVICE_POWER_STATE, ULONG, etc.) via the ddk glob. ACPI_* types
// come from crate::shared (acpica-sys) — ddk has no ACPI_HANDLE, no collision.
use ntkrnl_sys::ddk::*;
// IoCallDriver / IoCompleteRequest live in driver_ddk (inline wrappers), as do
// PoCallDriver etc. (matches power.rs).
use ntkrnl_sys::driver_ddk::{IoCallDriver, IoCompleteRequest};
// IoSkipCurrentIrpStackLocation + IoSetCompletionRoutine are FORCEINLINE — from inline::tier1.
use ntkrnl_sys::inline::tier1::{IoGetCurrentIrpStackLocation, IoGetNextIrpStackLocation,
    IoSkipCurrentIrpStackLocation, IoSetCompletionRoutine, ObReferenceObject};
// InsertTailList is a FORCEINLINE list helper.
use ntkrnl_sys::inline::common::InsertTailList;
// STATUS_* + IO_NO_INCREMENT live in base/constants (not in the ddk glob).
use ntkrnl_sys::base::{STATUS_SUCCESS, STATUS_INSUFFICIENT_RESOURCES, TRUE, FALSE};
use ntkrnl_sys::constants::{STATUS_UNSUCCESSFUL, STATUS_PENDING, IO_NO_INCREMENT};
use ntkrnl_sys::mm::PagedPool;
// IRP_MN_* minor codes (u8) live in driver_ddk, not the ddk bindgen glob.
use ntkrnl_sys::driver_ddk::{IRP_MN_START_DEVICE, IRP_MN_QUERY_STOP_DEVICE,
    IRP_MN_CANCEL_STOP_DEVICE, IRP_MN_QUERY_DEVICE_RELATIONS, DO_DEVICE_INITIALIZING};

// STATUS_MORE_PROCESSING_REQUIRED (ntstatus.h) — not exported by ntkrnl_sys, define locally.
const STATUS_MORE_PROCESSING_REQUIRED: NTSTATUS = 0xC0000016_u32 as i32;

// DEVICE_POWER_STATE constants (bindgen emits _DEVICE_POWER_STATE_<Name>).
use ntkrnl_sys::ddk::{
    _DEVICE_POWER_STATE_PowerDeviceUnspecified as PowerDeviceUnspecified,
    _DEVICE_POWER_STATE_PowerDeviceD0 as PowerDeviceD0,
    _DEVICE_POWER_STATE_PowerDeviceD1 as PowerDeviceD1,
    _DEVICE_POWER_STATE_PowerDeviceD2 as PowerDeviceD2,
    _DEVICE_POWER_STATE_PowerDeviceD3 as PowerDeviceD3,
    _POWER_STATE_TYPE_DevicePowerState as DevicePowerState,
    _DEVICE_RELATION_TYPE_BusRelations as BusRelations,
    _EVENT_TYPE_NotificationEvent as NotificationEvent,
    _KWAIT_REASON_Executive as Executive,
    _MODE_KernelMode as KernelMode,
};

// ACPICA outbound calls (acpica_sys, all extern "C").
use acpica_sys::{AcpiInitializeSubsystem, AcpiInitializeTables, AcpiLoadTables,
    AcpiTerminate, AcpiGetHandle};

// ---- Pool tags. C multichar 'WXYZ' = ('W'<<24)|('X'<<16)|('Y'<<8)|'Z'. ----
const TAG_IpcA: u32 = u32::from_be_bytes([b'I', b'p', b'c', b'A']);
const TAG_DpcA: u32 = u32::from_be_bytes([b'D', b'p', b'c', b'A']);

// ---- ACPI device power states (acpi_bus.h enum acpi_device_power_state). ----
// FIXME(integrator): confirm values — acpi_bus.h: ACPI_STATE_D0=0, D1=1, D2=2, D3=3.
const ACPI_STATE_D0: c_int = 0;
const ACPI_STATE_D1: c_int = 1;
const ACPI_STATE_D2: c_int = 2;
const ACPI_STATE_D3: c_int = 3;

#[inline]
fn NT_SUCCESS(status: NTSTATUS) -> bool {
    status >= 0
}
#[inline]
fn ACPI_SUCCESS(status: ACPI_STATUS) -> bool {
    status == AE_OK
}
#[inline]
fn ACPI_FAILURE(status: ACPI_STATUS) -> bool {
    status != AE_OK
}

// ---- SET_NEW_PNP_STATE / RESTORE_PREVIOUS_PNP_STATE / INITIALIZE_PNP_STATE
//      (acpisys.h macros, expanded inline; plain struct field writes). ----
#[inline]
unsafe fn SET_NEW_PNP_STATE(common: *mut COMMON_DEVICE_DATA, state: DEVICE_PNP_STATE) {
    (*common).PreviousPnPState = (*common).DevicePnPState;
    (*common).DevicePnPState = state;
}
#[inline]
unsafe fn RESTORE_PREVIOUS_PNP_STATE(common: *mut COMMON_DEVICE_DATA) {
    (*common).DevicePnPState = (*common).PreviousPnPState;
    (*common).PreviousPnPState = NotStarted;
}
#[inline]
unsafe fn INITIALIZE_PNP_STATE(common: *mut COMMON_DEVICE_DATA) {
    (*common).DevicePnPState = NotStarted;
    (*common).PreviousPnPState = NotStarted;
}

// ---- CONTAINING_RECORD(addr, PDO_DEVICE_DATA, Link) ----
#[inline]
unsafe fn CONTAINING_RECORD_PDO_Link(entry: *mut LIST_ENTRY) -> *mut PDO_DEVICE_DATA {
    (entry as *mut u8).wrapping_sub(core::mem::offset_of!(PDO_DEVICE_DATA, Link))
        as *mut PDO_DEVICE_DATA
}

// ---- Kernel exports not in the ddk glob ----
// PoSetPowerState: kernel export, NTAPI (stdcall on i386, cdecl on amd64/arm64 —
// `extern "system"` resolves per arch). Returns POWER_STATE by value (matches power.rs).
extern "system" {
    pub fn PoSetPowerState(
        DeviceObject: PDEVICE_OBJECT,
        Type: POWER_STATE_TYPE,
        State: POWER_STATE,
    ) -> POWER_STATE;
}

// RtlCopyMemory is a C macro (→ memcpy), not an export → inline.
#[inline]
unsafe fn RtlCopyMemory(dst: *mut c_void, src: *const c_void, len: usize) {
    core::ptr::copy_nonoverlapping(src as *const u8, dst as *mut u8, len);
}

// IoCopyCurrentIrpStackLocationToNext is a FORCEINLINE (wdm.h), not an export:
// copy the current stack location to the next, up through (not including) the
// CompletionRoutine field, then clear the next location's Control flags.
#[inline]
unsafe fn IoCopyCurrentIrpStackLocationToNext(Irp: PIRP) {
    let cur = IoGetCurrentIrpStackLocation(Irp);
    let next = IoGetNextIrpStackLocation(Irp);
    core::ptr::copy_nonoverlapping(
        cur as *const u8,
        next as *mut u8,
        core::mem::offset_of!(IO_STACK_LOCATION, CompletionRoutine),
    );
    (*next).Control = 0;
}

// ---- Sibling acpi-rust functions (all extern "C"/cdecl; #[no_mangle] elsewhere). ----
// Bus_PlugInDevice is DEFINED in acpienum.c (forward-decl only here).
extern "C" {
    fn Bus_PlugInDevice(Device: *mut acpi_device, FdoData: PFDO_DEVICE_DATA) -> NTSTATUS;
    fn Bus_PDO_PnP(
        dev: PDEVICE_OBJECT,
        irp: PIRP,
        sp: PIO_STACK_LOCATION,
        dd: PPDO_DEVICE_DATA,
    ) -> NTSTATUS;
    fn acpi_create_volatile_registry_tables() -> NTSTATUS;
    fn acpi_init() -> c_int;
    fn ACPIEnumerateDevices(DeviceExtension: PFDO_DEVICE_DATA) -> NTSTATUS;
    fn acpi_bus_get_power(handle: ACPI_HANDLE, state: *mut c_int) -> c_int;
}

// ============================================================================
// Bus_PnP — NTAPI top-level IRP_MJ_PNP dispatch → extern "system".
// ============================================================================
#[no_mangle]
pub unsafe extern "system" fn Bus_PnP(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    let irpStack: PIO_STACK_LOCATION;
    let status: NTSTATUS;
    let commonData: PCOMMON_DEVICE_DATA;

    // PAGED_CODE() — no-op.

    irpStack = IoGetCurrentIrpStackLocation(Irp);
    // ASSERT(IRP_MJ_PNP == irpStack->MajorFunction) — release-build no-op.

    commonData = (*DeviceObject).DeviceExtension as PCOMMON_DEVICE_DATA;

    if (*commonData).IsFDO != 0 {
        // Request is for the bus FDO.
        status = Bus_FDO_PnP(
            DeviceObject,
            Irp,
            irpStack,
            commonData as PFDO_DEVICE_DATA,
        );
    } else {
        // Request is for the child PDO.
        status = Bus_PDO_PnP(
            DeviceObject,
            Irp,
            irpStack,
            commonData as PPDO_DEVICE_DATA,
        );
    }

    status
}

// ============================================================================
// Bus_FDO_PnP — plain C → extern "C".
// ============================================================================
#[no_mangle]
pub unsafe extern "C" fn Bus_FDO_PnP(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    IrpStack: PIO_STACK_LOCATION,
    DeviceData: PFDO_DEVICE_DATA,
) -> NTSTATUS {
    let mut status: NTSTATUS;
    let mut length: ULONG;
    let mut prevcount: ULONG;
    let mut numPdosPresent: ULONG;
    let mut entry: PLIST_ENTRY;
    let mut pdoData: PPDO_DEVICE_DATA;
    let mut relations: PDEVICE_RELATIONS;
    let mut oldRelations: PDEVICE_RELATIONS;

    // PAGED_CODE() — no-op.

    match (*IrpStack).MinorFunction {
        IRP_MN_START_DEVICE => {
            status = Bus_StartFdo(DeviceData, Irp);

            // We must now complete the IRP, since we stopped it in the
            // completion routine with MORE_PROCESSING_REQUIRED.
            (*Irp).IoStatus.__bindgen_anon_1.Status = status;
            IoCompleteRequest(Irp, IO_NO_INCREMENT);

            return status;
        }

        IRP_MN_QUERY_STOP_DEVICE => {
            // PnP manager is trying to stop the device for resource rebalancing.
            SET_NEW_PNP_STATE(addr_of_mut!((*DeviceData).Common), StopPending);
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
        }

        IRP_MN_CANCEL_STOP_DEVICE => {
            // First check we received a query-stop before this cancel-stop.
            if StopPending == (*DeviceData).Common.DevicePnPState {
                RESTORE_PREVIOUS_PNP_STATE(addr_of_mut!((*DeviceData).Common));
                // ASSERT(DevicePnPState == Started) — no-op.
            }
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS; // must not fail.
        }

        IRP_MN_QUERY_DEVICE_RELATIONS => {
            // union path: IO_STACK_LOCATION.Parameters.QueryDeviceRelations.Type
            // (QueryDeviceRelations is a named member, Type is DEVICE_RELATION_TYPE).
            if BusRelations != (*IrpStack).Parameters.QueryDeviceRelations.Type {
                // We don't support any other Device Relations.
                // fall through to IoSkip + IoCallDriver below.
            } else {
                ExAcquireFastMutex(addr_of_mut!((*DeviceData).Mutex));

                // Irp->IoStatus.Information holds the existing PDEVICE_RELATIONS.
                oldRelations =
                    (*Irp).IoStatus.Information as PDEVICE_RELATIONS;
                if !oldRelations.is_null() {
                    prevcount = (*oldRelations).Count;
                    if (*DeviceData).NumPDOs == 0 {
                        // Nothing to add — just IoSkip and IoCall.
                        ExReleaseFastMutex(addr_of_mut!((*DeviceData).Mutex));
                        // break out of the match
                        IoSkipCurrentIrpStackLocation(Irp);
                        status = IoCallDriver((*DeviceData).NextLowerDriver, Irp);
                        return STATUS_SUCCESS;
                    }
                } else {
                    prevcount = 0;
                }

                // Calculate the number of PDOs actually present on the bus.
                numPdosPresent = 0;
                entry = (*DeviceData).ListOfPDOs.Flink;
                while entry != addr_of_mut!((*DeviceData).ListOfPDOs) {
                    pdoData = CONTAINING_RECORD_PDO_Link(entry);
                    numPdosPresent += 1;
                    entry = (*entry).Flink;
                }

                // Allocate a new relations structure with our PDOs.
                length = (core::mem::size_of::<DEVICE_RELATIONS>() as ULONG)
                    + (((numPdosPresent + prevcount) - 1)
                        * core::mem::size_of::<PDEVICE_OBJECT>() as ULONG);

                relations = ExAllocatePoolWithTag(PagedPool, length as SIZE_T, TAG_IpcA)
                    as PDEVICE_RELATIONS;

                if relations.is_null() {
                    // Fail the IRP.
                    ExReleaseFastMutex(addr_of_mut!((*DeviceData).Mutex));
                    status = STATUS_INSUFFICIENT_RESOURCES;
                    (*Irp).IoStatus.__bindgen_anon_1.Status = status;
                    IoCompleteRequest(Irp, IO_NO_INCREMENT);
                    return status;
                }

                // Copy in the device objects so far.
                if prevcount != 0 {
                    RtlCopyMemory(
                        (*relations).Objects.as_mut_ptr() as *mut c_void,
                        (*oldRelations).Objects.as_ptr() as *const c_void,
                        (prevcount as usize) * core::mem::size_of::<PDEVICE_OBJECT>(),
                    );
                }

                (*relations).Count = prevcount + numPdosPresent;

                // For each PDO add a pointer + take a reference. Objects is a
                // flexible array [PDEVICE_OBJECT;1] → pointer arithmetic, NOT [].
                entry = (*DeviceData).ListOfPDOs.Flink;
                while entry != addr_of_mut!((*DeviceData).ListOfPDOs) {
                    pdoData = CONTAINING_RECORD_PDO_Link(entry);
                    let slot = (*relations).Objects.as_mut_ptr().add(prevcount as usize);
                    *slot = (*pdoData).Common.Self_;
                    ObReferenceObject((*pdoData).Common.Self_ as PVOID);
                    prevcount += 1;
                    entry = (*entry).Flink;
                }

                // Replace the relations structure in the IRP with the new one.
                if !oldRelations.is_null() {
                    ExFreePoolWithTag(oldRelations as PVOID, 0);
                }
                (*Irp).IoStatus.Information = relations as ULONG_PTR;

                ExReleaseFastMutex(addr_of_mut!((*DeviceData).Mutex));

                (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
            }
        }

        _ => {
            // Default: merely call the next driver. Must not modify
            // Irp->IoStatus.Status or complete the IRP.
        }
    }

    IoSkipCurrentIrpStackLocation(Irp);
    status = IoCallDriver((*DeviceData).NextLowerDriver, Irp);
    let _ = status;
    STATUS_SUCCESS
}

// ============================================================================
// Bus_StartFdo — plain C → extern "C".
// ============================================================================
#[no_mangle]
pub unsafe extern "C" fn Bus_StartFdo(FdoData: PFDO_DEVICE_DATA, Irp: PIRP) -> NTSTATUS {
    let status: NTSTATUS;
    let mut powerState: POWER_STATE = core::mem::zeroed();
    let mut AcpiStatus: ACPI_STATUS;

    // PAGED_CODE() — no-op.

    (*FdoData).Common.DevicePowerState = PowerDeviceD0;
    powerState.DeviceState = PowerDeviceD0;
    PoSetPowerState((*FdoData).Common.Self_, DevicePowerState, powerState);

    SET_NEW_PNP_STATE(addr_of_mut!((*FdoData).Common), Started);

    AcpiStatus = AcpiInitializeSubsystem();
    if ACPI_FAILURE(AcpiStatus) {
        DPRINT1!("Unable to AcpiInitializeSubsystem\n");
        return STATUS_UNSUCCESSFUL;
    }

    AcpiStatus = AcpiInitializeTables(null_mut(), 16, 0);
    if ACPI_FAILURE(AcpiStatus) {
        DPRINT1!("Unable to AcpiInitializeTables\n");
        return STATUS_UNSUCCESSFUL;
    }

    AcpiStatus = AcpiLoadTables();
    if ACPI_FAILURE(AcpiStatus) {
        DPRINT1!("Unable to AcpiLoadTables\n");
        AcpiTerminate();
        return STATUS_UNSUCCESSFUL;
    }

    let regstatus = acpi_create_volatile_registry_tables();
    if !NT_SUCCESS(regstatus) {
        DPRINT1!("Unable to create ACPI tables in registry\n");
    }

    // Initialize ACPI bus manager.
    AcpiStatus = acpi_init() as _;
    if !ACPI_SUCCESS(AcpiStatus) {
        DPRINT1!("acpi_init() failed\n");
        AcpiTerminate();
        return STATUS_UNSUCCESSFUL;
    }
    status = ACPIEnumerateDevices(FdoData);

    status
}

// ============================================================================
// Bus_SendIrpSynchronously — plain C → extern "C".
// ============================================================================
#[no_mangle]
pub unsafe extern "C" fn Bus_SendIrpSynchronously(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut event: KEVENT = core::mem::zeroed();
    let mut status: NTSTATUS;

    // PAGED_CODE() — no-op.

    KeInitializeEvent(&mut event, NotificationEvent, FALSE as BOOLEAN);

    IoCopyCurrentIrpStackLocationToNext(Irp);

    IoSetCompletionRoutine(
        Irp,
        // extern "system" fn → PIO_COMPLETION_ROUTINE (stdcall/i386, C/64b): transmute
        // (inferred target) bridges the abi-string type mismatch.
        core::mem::transmute(Bus_CompletionRoutine as *const ()),
        addr_of_mut!(event) as PVOID,
        TRUE as BOOLEAN,
        TRUE as BOOLEAN,
        TRUE as BOOLEAN,
    );

    status = IoCallDriver(DeviceObject, Irp);

    // Wait for lower drivers to be done with the Irp. Stack-allocated event →
    // KernelMode wait to keep the stack from paging out.
    if status == STATUS_PENDING {
        KeWaitForSingleObject(
            addr_of_mut!(event) as PVOID,
            Executive,
            KernelMode as KPROCESSOR_MODE,
            FALSE as BOOLEAN,
            null_mut(),
        );
        status = (*Irp).IoStatus.__bindgen_anon_1.Status;
    }

    status
}

// ============================================================================
// Bus_CompletionRoutine — NTAPI → extern "system".
// ============================================================================
#[no_mangle]
pub unsafe extern "system" fn Bus_CompletionRoutine(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Context: PVOID,
) -> NTSTATUS {
    let _ = DeviceObject; // UNREFERENCED_PARAMETER

    // If the lower driver didn't return STATUS_PENDING, don't set the event.
    if (*Irp).PendingReturned != FALSE as BOOLEAN {
        KeSetEvent(Context as PKEVENT, IO_NO_INCREMENT as KPRIORITY, FALSE as BOOLEAN);
    }
    STATUS_MORE_PROCESSING_REQUIRED // Keep this IRP.
}

// ============================================================================
// Bus_DestroyPdo — plain C → extern "C".
// ============================================================================
#[no_mangle]
pub unsafe extern "C" fn Bus_DestroyPdo(
    Device: PDEVICE_OBJECT,
    PdoData: PPDO_DEVICE_DATA,
) -> NTSTATUS {
    // PAGED_CODE() — no-op.

    // BusEnum queues no irps → nothing to do. Free any resources.
    if !(*PdoData).HardwareIDs.is_null() {
        ExFreePoolWithTag((*PdoData).HardwareIDs as PVOID, TAG_DpcA);
        (*PdoData).HardwareIDs = null_mut();
    }

    IoDeleteDevice(Device);
    STATUS_SUCCESS
}

// ============================================================================
// Bus_InitializePdo — plain C → extern "C".
// ============================================================================
#[no_mangle]
pub unsafe extern "C" fn Bus_InitializePdo(Pdo: PDEVICE_OBJECT, FdoData: PFDO_DEVICE_DATA) {
    let pdoData: PPDO_DEVICE_DATA;
    let mut acpistate: c_int = 0;
    let ntState: DEVICE_POWER_STATE;
    let mut handle: ACPI_HANDLE = null_mut();
    let mut status: ACPI_STATUS = 0;

    // PAGED_CODE() — no-op.

    pdoData = (*Pdo).DeviceExtension as PPDO_DEVICE_DATA;

    if !(*pdoData).AcpiHandle.is_null() {
        acpi_bus_get_power((*pdoData).AcpiHandle, addr_of_mut!(acpistate));
    } else {
        acpistate = ACPI_STATE_D0;
    }

    ntState = match acpistate {
        x if x == ACPI_STATE_D0 => PowerDeviceD0,
        x if x == ACPI_STATE_D1 => PowerDeviceD1,
        x if x == ACPI_STATE_D2 => PowerDeviceD2,
        x if x == ACPI_STATE_D3 => PowerDeviceD3,
        _ => {
            DPRINT1!("Unknown power state\n");
            PowerDeviceUnspecified
        }
    };

    // Initialize the rest.
    (*pdoData).Common.IsFDO = FALSE as BOOLEAN;
    (*pdoData).Common.Self_ = Pdo;

    (*pdoData).ParentFdo = (*FdoData).Common.Self_;

    INITIALIZE_PNP_STATE(addr_of_mut!((*pdoData).Common));

    (*pdoData).Common.DevicePowerState = ntState;
    (*pdoData).Common.SystemPowerState = (*FdoData).Common.SystemPowerState;

    // Identify the dock device.
    if !(*pdoData).AcpiHandle.is_null() {
        status = AcpiGetHandle(
            (*pdoData).AcpiHandle,
            c"_DCK".as_ptr() as ACPI_STRING,
            addr_of_mut!(handle),
        );
        if ACPI_SUCCESS(status) {
            DPRINT!("Found _DCK method!");
            (*pdoData).DockDevice = TRUE as BOOLEAN;
        }
    }

    ExAcquireFastMutex(addr_of_mut!((*FdoData).Mutex));
    InsertTailList(addr_of_mut!((*FdoData).ListOfPDOs), addr_of_mut!((*pdoData).Link));
    (*FdoData).NumPDOs += 1;
    ExReleaseFastMutex(addr_of_mut!((*FdoData).Mutex));

    // This should be the last step in initialization.
    (*Pdo).Flags &= !DO_DEVICE_INITIALIZING;
}

// NOTE: PnPMinorFunctionString / DbgDeviceRelationString / DbgDeviceIDString are
// all under `#if DBG` in pnp.c. The driver builds with NDEBUG (DBG off), so they
// are NOT compiled/linked — omitted here intentionally.
