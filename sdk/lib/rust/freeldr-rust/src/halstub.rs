//! arch/i386/halstub.c — minimal HAL dispatch-table stubs for rosload.exe.
//!
//! rosload.exe (loaded by freeldr as "scsiport.sys") provides the
//! `HalDispatchTable` / `HalPrivateDispatchTable` globals that the kernel
//! image it loads binds to, plus three trivial HAL routines. i386 + rosload
//! ONLY (see rosload.cmake / lib.rs gate) — nothing else references these
//! symbols. The `xHal*` stubs are referenced solely by the two tables, so
//! they stay private Rust fns (no #[no_mangle], no decoration to match).

use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{
    HAL_DISPATCH, _HAL_PRIVATE_DISPATCH, PDEVICE_OBJECT, PDRIVE_LAYOUT_INFORMATION,
};

// haltypes.h, NTDDI_WS03 (NT 5.2): HAL_DISPATCH_VERSION = 3 (WINXP..<WIN7),
// HAL_PRIVATE_DISPATCH_VERSION = 2 (WINXP..<LONGHORN).
const HAL_DISPATCH_VERSION: ULONG = 3;
const HAL_PRIVATE_DISPATCH_VERSION: ULONG = 2;

extern "system" {
    // IoReadPartitionTable: NTAPI (= stdcall on i386); resolved in the
    // rosload link (same provider that satisfied the C halstub).
    fn IoReadPartitionTable(
        DeviceObject: PDEVICE_OBJECT,
        SectorSize: ULONG,
        ReturnRecognizedPartitions: BOOLEAN,
        PartitionBuffer: *mut PDRIVE_LAYOUT_INFORMATION,
    ) -> NTSTATUS;
}

// FASTCALL — matches pHalIoReadPartitionTable exactly.
unsafe extern "fastcall" fn x_hal_io_read_partition_table(
    device_object: PDEVICE_OBJECT,
    sector_size: ULONG,
    return_recognized_partitions: BOOLEAN,
    partition_buffer: *mut PDRIVE_LAYOUT_INFORMATION,
) -> NTSTATUS {
    IoReadPartitionTable(
        device_object,
        sector_size,
        return_recognized_partitions,
        partition_buffer,
    )
}

// NTAPI — `system` IS that convention per arch (stdcall on i386, C on the 64-bit pair), and
// it is what the HALSTUB slots are spelled now, so these unify without an arch fork.
unsafe extern "system" fn x_hal_vector_to_idt_entry(vector: ULONG) -> UCHAR {
    vector as UCHAR
}

unsafe extern "system" fn x_hal_halt_system() {
    /* Halt execution */
    loop {}
}

// #[used]: the tables are not referenced inside rosload, but the C source
// object always linked them in (the HAL surface for the loaded image).
// #[used] keeps them past --gc-sections in the Rust staticlib.
#[used]
#[no_mangle]
pub static mut HalDispatchTable: HAL_DISPATCH = HAL_DISPATCH {
    Version: HAL_DISPATCH_VERSION,
    HalQuerySystemInformation: None,
    HalSetSystemInformation: None,
    HalQueryBusSlots: None,
    Spare1: 0,
    HalExamineMBR: None,
    HalIoAssignDriveLetters: None,
    HalIoReadPartitionTable: Some(x_hal_io_read_partition_table),
    HalIoSetPartitionInformation: None,
    HalIoWritePartitionTable: None,
    HalReferenceHandlerForBus: None,
    HalReferenceBusHandler: None,
    HalDereferenceBusHandler: None,
    HalInitPnpDriver: None,
    HalInitPowerManagement: None,
    HalGetDmaAdapter: None,
    HalGetInterruptTranslator: None,
    HalStartMirroring: None,
    HalEndMirroring: None,
    HalMirrorPhysicalMemory: None,
    HalEndOfBoot: None,
    HalMirrorVerify: None,
    HalGetCachedAcpiTable: None,
    HalSetPciErrorHandlerCallback: None,
};

#[used]
#[no_mangle]
pub static mut HalPrivateDispatchTable: _HAL_PRIVATE_DISPATCH = _HAL_PRIVATE_DISPATCH {
    Version: HAL_PRIVATE_DISPATCH_VERSION,
    HalHandlerForBus: None,
    HalHandlerForConfigSpace: None,
    HalLocateHiberRanges: None,
    HalRegisterBusHandler: None,
    HalSetWakeEnable: None,
    HalSetWakeAlarm: None,
    HalPciTranslateBusAddress: None,
    HalPciAssignSlotResources: None,
    HalHaltSystem: Some(x_hal_halt_system),
    HalFindBusAddressTranslation: None,
    HalResetDisplay: None,
    HalAllocateMapRegisters: None,
    KdSetupPciDeviceForDebugging: None,
    KdReleasePciDeviceforDebugging: None,
    KdGetAcpiTablePhase0: None,
    KdCheckPowerButton: None,
    HalVectorToIDTEntry: Some(x_hal_vector_to_idt_entry),
    KdMapPhysicalMemory64: None,
    KdUnmapVirtualAddress: None,
    // The NT6 tail ReactOS's haltypes.h declares for every build: NULL, as in the C.
    ..unsafe { core::mem::zeroed() }
};
