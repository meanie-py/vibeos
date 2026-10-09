//! shims.rs — union of FFI extern decls harvested from all modules (dedup by name),
//! glob-imported by every module. Local extern decls shadow these (no conflict).
#![allow(non_snake_case, dead_code)]
use crate::types::*;
use ntkrnl_sys::ddk::*;
use core::ffi::c_void;

extern "C" {
    pub fn ClassAcquireRemoveLock(fdo: *mut DEVICE_OBJECT, irp: *mut IRP) -> u32;
    
    pub fn ClassFreeOrReuseSrb( fdo_extension: *mut FUNCTIONAL_DEVICE_EXTENSION, srb: *mut SCSI_REQUEST_BLOCK, );
    
    
    pub fn ClasspIsThinProvisioningError(srb: PSCSI_REQUEST_BLOCK) -> BOOLEAN;
    pub fn ClasspLogSystemEventWithDeviceNumber(Fdo: PDEVICE_OBJECT, EventId: ULONG) -> u32;
    pub fn ClasspPerfIncrementSuccessfulIo( FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION, );
    pub fn DbgGetScsiOpStr(srb: PVOID) -> PCCHAR;
    pub fn FREE_POOL(Ptr: PVOID);
    pub fn FREE_PORT_ALLOCATED_SENSE_BUFFER( fdo_extension: *mut FUNCTIONAL_DEVICE_EXTENSION, srb: *mut SCSI_REQUEST_BLOCK, );
    pub fn InterpretSenseInfoWithoutHistory( fdo: *mut DEVICE_OBJECT, irp: *mut IRP, srb: *mut SCSI_REQUEST_BLOCK, major_function: u8, control_code_or_zero: u32, retry_count: u32, status: *mut i32, retry_interval: *mut i64, ) -> u8;
    pub fn IoGetCurrentIrpStackLocation(Irp: PIRP) -> PIO_STACK_LOCATION;
    pub fn IoIsErrorUserInduced(status: i32) -> u8;
    pub fn IoMarkIrpPending(Irp: PIRP);
    pub fn PORT_ALLOCATED_SENSE( fdo_extension: *mut FUNCTIONAL_DEVICE_EXTENSION, srb: *mut SCSI_REQUEST_BLOCK, ) -> u8;
    pub fn REVERSE_BYTES(dest: *mut u32, src: *const u32);
    pub fn RetryRequest( fdo: *mut DEVICE_OBJECT, irp: *mut IRP, srb: *mut SCSI_REQUEST_BLOCK, associated: u8, time_delta_100ns: i64, );
    pub fn SRB_STATUS(status: u8) -> u8;
    pub fn SrbClearSrbFlags(srb: PVOID, flags: ULONG);
    
    
    
    pub fn SrbSetRequestTag(srb: PVOID, tag: ULONG);
    pub fn SrbSetSrbFlags(srb: PVOID, flags: ULONG);
    pub fn TEST_FLAG(Flags: ULONG, Flag: ULONG) -> BOOLEAN;
    pub fn TracePrint(level: u32, flags: u32, format: *const i8, ...);
}
unsafe extern "system" {
    pub fn KeSetCoalescableTimer(Timer: *mut KTIMER, DueTime: *mut core::ffi::c_void, Period: ULONG, TolerableDelay: ULONG, Dpc: PKDPC) -> u8;
}

extern "system" {
    pub fn NT_ASSERT(b: u8);
    pub fn SrbAssignSrbFlags(srb: *mut core::ffi::c_void, flags: u32);
    pub fn SrbSetCdbLength(srb: *mut core::ffi::c_void, length: u8);
    pub fn SrbSetOriginalRequest(srb: *mut core::ffi::c_void, irp: PIRP);
    pub fn SrbSetRequestAttribute(srb: *mut core::ffi::c_void, attribute: u8);
    pub fn SrbSetSenseInfoBuffer(srb: *mut core::ffi::c_void, sense_info: *mut core::ffi::c_void);
    pub fn SrbSetSenseInfoBufferLength(srb: *mut core::ffi::c_void, length: u8);
    pub fn SrbSetTimeOutValue(srb: *mut core::ffi::c_void, timeout: u32);
}

#[cfg(any(target_arch="x86",target_arch="x86_64"))]
#[no_mangle] pub unsafe extern "C" fn __int2c() { core::arch::asm!("int 0x2c", options(nomem, nostack)); }
#[cfg(not(any(target_arch="x86",target_arch="x86_64")))]
#[no_mangle] pub unsafe extern "C" fn __int2c() { core::arch::asm!("brk #0", options(nomem, nostack)); }



// ---- ABI-corrected extern decls (CC matched to in-crate definition) ----
extern "C" {
    
    
    
}
unsafe extern "system" {
    pub fn KeLowerIrql(OldIrql: KIRQL);
    pub fn KeRaiseIrql(NewIrql: KIRQL, OldIrql: *mut KIRQL);
}
extern "system" {
    pub fn ClassCompleteRequest(DeviceObject: *mut DEVICE_OBJECT, Irp: *mut IRP, PriorityBoost: u8);
    pub fn ClassReleaseQueue(fdo: *mut DEVICE_OBJECT);
    pub fn ClassReleaseRemoveLock(DeviceObject: *mut DEVICE_OBJECT, Irp: *mut IRP);
    pub fn SrbGetCdb(srb: PVOID) -> PCDB;
    pub fn SrbSetDataBuffer(srb: PVOID, buf: PVOID);
    pub fn SrbSetDataTransferLength(srb: PVOID, len: ULONG);
}


// ---- export decls forced to stdcall (real kernel export CC) ----
extern "system" {
    pub fn IoFreeMdl(Mdl: PMDL);
    pub fn IoSetHardErrorOrVerifyDevice(Irp: PIRP, Fdo: PDEVICE_OBJECT);
    pub fn IoStartNextPacket(Fdo: PDEVICE_OBJECT, Cancellable: BOOLEAN);
}
