//! HAL disk-access stubs.  Four functions, all pure passthroughs to
//! the corresponding kernel `Io*` calls.  Exists so the disk-driver
//! contract has a HAL-side dispatch slot; the actual logic lives in
//! the kernel.
//!
//! Translated from `hal/halx86/generic/drive.c`.

// Use kernel-side ddk types for the call boundary — passing pointers
// across hal_ddk and ddk would otherwise hit cross-module type mismatch
// even though the C layouts are identical.
use ntkrnl_sys::ddk::{
    IoAssignDriveLetters, IoReadPartitionTable, IoSetPartitionInformation,
    IoWritePartitionTable, BOOLEAN, NTSTATUS, PDEVICE_OBJECT, PDRIVE_LAYOUT_INFORMATION,
    PLOADER_PARAMETER_BLOCK, PSTRING, PUCHAR, ULONG,
};

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpAssignDriveLetters(
    LoaderBlock: PLOADER_PARAMETER_BLOCK,
    NtDeviceName: PSTRING,
    NtSystemPath: PUCHAR,
    NtSystemPathString: PSTRING,
) {
    unsafe { IoAssignDriveLetters(LoaderBlock, NtDeviceName, NtSystemPath, NtSystemPathString); }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpReadPartitionTable(
    DeviceObject: PDEVICE_OBJECT,
    SectorSize: ULONG,
    ReturnRecognizedPartitions: BOOLEAN,
    PartitionBuffer: *mut PDRIVE_LAYOUT_INFORMATION,
) -> NTSTATUS {
    unsafe {
        IoReadPartitionTable(DeviceObject, SectorSize, ReturnRecognizedPartitions, PartitionBuffer)
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpWritePartitionTable(
    DeviceObject: PDEVICE_OBJECT,
    SectorSize: ULONG,
    SectorsPerTrack: ULONG,
    NumberOfHeads: ULONG,
    PartitionBuffer: PDRIVE_LAYOUT_INFORMATION,
) -> NTSTATUS {
    unsafe {
        IoWritePartitionTable(
            DeviceObject, SectorSize, SectorsPerTrack, NumberOfHeads, PartitionBuffer,
        )
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpSetPartitionInformation(
    DeviceObject: PDEVICE_OBJECT,
    SectorSize: ULONG,
    PartitionNumber: ULONG,
    PartitionType: ULONG,
) -> NTSTATUS {
    unsafe { IoSetPartitionInformation(DeviceObject, SectorSize, PartitionNumber, PartitionType) }
}
