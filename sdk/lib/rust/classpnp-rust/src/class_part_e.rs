//! class_part_e.rs — final cohab link-gap fns surfaced once the systemic captures(none)/CC
//! fold was eliminated (2026-06-11). Hand-translated from class.c / classp.h (no API spend):
//!   CountOfSetBitsUChar, ClasspIsTokenOperation, ClasspIsOffloadDataTransferCommand,
//!   ClasspTransferPacketGetNumberOfRetriesDone, ClasspSendSynchronousCompletion,
//!   ClasspAllocateReleaseQueueIrp, ClassReleaseQueueCompletion, ClasspRetryDpcTimer.
//! ABI set from each undef's decoration: bare `_X` => extern "C"; `_X@N` => extern "system".
//! Output pointers use write_volatile (captures(none) firewall — DO NOT use `*p = v`).
#![allow(non_snake_case, unused_imports, unused_variables, dead_code, unused_mut)]
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{
    IoInitializeIrp, IoFreeMdl, IoFreeIrp, MmUnlockPages, KeSetEvent, KeSetTimerEx,
    ExAllocatePoolWithTag,
    PDEVICE_OBJECT, PIRP, _IRP, _IO_STACK_LOCATION, STORAGE_HOTPLUG_INFO,
};
use crate::types::*;
use crate::glue::*;
use crate::shims::*;

// classp.h:1195  #define MINIMUM_RETRY_UNITS ((LONGLONG)32)
const MINIMUM_RETRY_UNITS: i64 = 32;
// classpnp.h:81  #define CLASS_TAG_RELEASE_QUEUE 'qLcS'
const CLASS_TAG_RELEASE_QUEUE: u32 = 0x714c6353;

// IoSizeOfIrp(StackSize) — macro: sizeof(IRP) + StackSize*sizeof(IO_STACK_LOCATION)
#[inline(always)]
unsafe fn io_size_of_irp(stack_size: u8) -> u16 {
    (core::mem::size_of::<_IRP>() + (stack_size as usize) * core::mem::size_of::<_IO_STACK_LOCATION>()) as u16
}

#[inline(always)]
fn is_scsiop_read(op: u32) -> bool {
    op == SCSIOP_READ6 || op == SCSIOP_READ || op == SCSIOP_READ12 || op == SCSIOP_READ16
}
#[inline(always)]
fn is_scsiop_write(op: u32) -> bool {
    op == SCSIOP_WRITE6 || op == SCSIOP_WRITE || op == SCSIOP_WRITE12 || op == SCSIOP_WRITE16
}

// classp.h: static inline ULONG CountOfSetBitsUChar(UCHAR _X)
#[no_mangle]
pub unsafe extern "C" fn CountOfSetBitsUChar(mut x: u8) -> u32 {
    let mut i: u32 = 0;
    while x != 0 { x &= x - 1; i += 1; }
    i
}

// classp.h:2044  FORCEINLINE BOOLEAN ClasspIsTokenOperation(PCDB Cdb)
#[no_mangle]
pub unsafe extern "C" fn ClasspIsTokenOperation(Cdb: PCDB) -> BOOLEAN {
    if Cdb.is_null() { return 0; }
    let b = (*Cdb).AsByte.as_ref();
    let op = b[0] as u32;
    let sa = b[1] as u32;
    if (op == SCSIOP_POPULATE_TOKEN && sa == SERVICE_ACTION_POPULATE_TOKEN)
        || (op == SCSIOP_WRITE_USING_TOKEN && sa == SERVICE_ACTION_WRITE_USING_TOKEN) {
        1
    } else {
        0
    }
}

#[inline(always)]
unsafe fn is_receive_token_information(Cdb: PCDB) -> bool {
    if Cdb.is_null() { return false; }
    let b = (*Cdb).AsByte.as_ref();
    (b[0] as u32) == SCSIOP_RECEIVE_ROD_TOKEN_INFORMATION
        && (b[1] as u32) == SERVICE_ACTION_RECEIVE_TOKEN_INFORMATION
}

// classp.h:2087  FORCEINLINE BOOLEAN ClasspIsOffloadDataTransferCommand(PCDB Cdb)
#[no_mangle]
pub unsafe extern "C" fn ClasspIsOffloadDataTransferCommand(Cdb: PCDB) -> BOOLEAN {
    if ClasspIsTokenOperation(Cdb) != 0 || is_receive_token_information(Cdb) { 1 } else { 0 }
}

// classp.h:2459  ClasspTransferPacketGetNumberOfRetriesDone
#[no_mangle]
pub unsafe extern "C" fn ClasspTransferPacketGetNumberOfRetriesDone(
    Pkt: PTRANSFER_PACKET,
    Cdb: PCDB,
    TimesAlreadyRetried: *mut ULONG,
) -> BOOLEAN {
    let fdoExtension = (*(*Pkt).Fdo).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let fdoData = (*fdoExtension).PrivateFdoData;
    let b = (*Cdb).AsByte.as_ref();
    let op = b[0] as u32;
    let sa = b[1] as u32;
    let num_retries = (*Pkt).NumRetries;

    let value: ULONG;
    if op == SCSIOP_MEDIUM_REMOVAL {
        value = NUM_LOCKMEDIAREMOVAL_RETRIES - num_retries;
    } else if op == SCSIOP_MODE_SENSE || op == SCSIOP_MODE_SENSE10 {
        value = NUM_MODESENSE_RETRIES - num_retries;
    } else if op == SCSIOP_READ_CAPACITY || op == SCSIOP_READ_CAPACITY16 {
        value = NUM_DRIVECAPACITY_RETRIES - num_retries;
    } else if is_scsiop_read(op) || is_scsiop_write(op) {
        value = (*fdoData).MaxNumberOfIoRetries as ULONG - num_retries;
    } else if op == SCSIOP_POPULATE_TOKEN && sa == SERVICE_ACTION_POPULATE_TOKEN {
        value = NUM_POPULATE_TOKEN_RETRIES - num_retries;
    } else if op == SCSIOP_WRITE_USING_TOKEN && sa == SERVICE_ACTION_WRITE_USING_TOKEN {
        value = NUM_WRITE_USING_TOKEN_RETRIES - num_retries;
    } else if is_receive_token_information(Cdb) {
        value = NUM_RECEIVE_TOKEN_INFORMATION_RETRIES - num_retries;
    } else {
        core::ptr::write_volatile(TimesAlreadyRetried, 0);
        return 0;
    }
    core::ptr::write_volatile(TimesAlreadyRetried, value);
    1
}

// class.c:10925  ClasspSendSynchronousCompletion (IO completion routine)
//
// NTAPI, like every IO_COMPLETION_ROUTINE (the C says so explicitly). It was
// `extern "C"`: on i386 the I/O manager calls it stdcall and expects it to pop
// its 12 bytes, it did not, and IofCompleteRequest returned through its own
// arguments into pool -- BUGCHECK 0x1E at a "<no module>" EIP beside the disk PDO
// on the first synchronous SRB of any hard disk. 64-bit has one convention,
// which is why only i386 crashed.
#[no_mangle]
pub unsafe extern "system" fn ClasspSendSynchronousCompletion(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Context: PVOID,
) -> NTSTATUS {
    // *(Irp->UserIosb) = Irp->IoStatus;
    core::ptr::copy_nonoverlapping(
        core::ptr::addr_of!((*Irp).IoStatus),
        (*Irp).UserIosb,
        1,
    );
    let mdl = (*Irp).MdlAddress;
    if !mdl.is_null() {
        MmUnlockPages(mdl);
        IoFreeMdl(mdl);
    }
    KeSetEvent((*Irp).UserEvent, IO_NO_INCREMENT as i32, 0);
    IoFreeIrp(Irp);
    STATUS_MORE_PROCESSING_REQUIRED
}

// class.c:11621  ClasspAllocateReleaseQueueIrp
#[no_mangle]
pub unsafe extern "C" fn ClasspAllocateReleaseQueueIrp(
    FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION,
) -> NTSTATUS {
    let fdoData = (*FdoExtension).PrivateFdoData;
    // initial check w/o the spinlock
    if (*fdoData).ReleaseQueueIrpAllocated != 0 {
        return STATUS_SUCCESS;
    }
    let lowerStackSize = (*(*(FdoExtension as PCOMMON_DEVICE_EXTENSION)).LowerDeviceObject).StackSize;

    let irp = ExAllocatePoolWithTag(
        NonPagedPoolNx as _,
        io_size_of_irp(lowerStackSize as u8) as _,
        CLASS_TAG_RELEASE_QUEUE,
    ) as PIRP;
    (*fdoData).ReleaseQueueIrp = irp;

    if irp.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    IoInitializeIrp(irp, io_size_of_irp(lowerStackSize as u8), lowerStackSize as i8);
    (*fdoData).ReleaseQueueIrpAllocated = TRUE as u8;
    STATUS_SUCCESS
}

// class.c:11898  ClassReleaseQueueCompletion (stdcall — undef was _ClassReleaseQueueCompletion@12)
#[no_mangle]
pub unsafe extern "system" fn ClassReleaseQueueCompletion(
    mut DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Context: PVOID,
) -> NTSTATUS {
    if Context.is_null() {
        return STATUS_INVALID_PARAMETER;
    }
    DeviceObject = Context as PDEVICE_OBJECT;
    let fdoExtension = (*DeviceObject).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;

    ClassReleaseRemoveLock(DeviceObject, Irp);

    let mut oldIrql: KIRQL = 0;
    KeAcquireSpinLock(core::ptr::addr_of_mut!((*fdoExtension).ReleaseQueueSpinLock) as *mut usize, &mut oldIrql);
    let releaseQueueNeeded = (*fdoExtension).ReleaseQueueNeeded;
    (*fdoExtension).ReleaseQueueNeeded = FALSE as u8;
    (*fdoExtension).ReleaseQueueInProgress = FALSE as u8;
    KeReleaseSpinLock(core::ptr::addr_of_mut!((*fdoExtension).ReleaseQueueSpinLock) as *mut usize, oldIrql);

    if releaseQueueNeeded != 0 {
        ClasspReleaseQueue(DeviceObject, Irp);
    }
    STATUS_MORE_PROCESSING_REQUIRED
}

// class.c:12457  ClasspRetryDpcTimer
#[no_mangle]
pub unsafe extern "C" fn ClasspRetryDpcTimer(FdoData: PCLASS_PRIVATE_FDO_DATA) {
    let mut fire: LARGE_INTEGER = core::mem::zeroed();
    KeQueryTickCount(&mut fire);
    fire.QuadPart = (*FdoData).Retry.Tick.QuadPart - fire.QuadPart;
    fire.QuadPart *= (*FdoData).Retry.Granularity as i64;

    if fire.QuadPart < MINIMUM_RETRY_UNITS {
        fire.QuadPart = MINIMUM_RETRY_UNITS;
    }
    // negative => relative time
    fire.QuadPart = fire.QuadPart * -1;

    KeSetTimerEx(
        core::ptr::addr_of_mut!((*FdoData).Retry.Timer) as _,
        core::mem::transmute(fire),
        0,
        core::ptr::addr_of_mut!((*FdoData).Retry.Dpc) as _,
    );
}

extern "system" {
    fn KeQueryTickCount(CurrentCount: *mut LARGE_INTEGER);
}
extern "C" {
    fn ClasspReleaseQueue(Fdo: PDEVICE_OBJECT, Irp: PIRP);
}
unsafe extern "system" {
    fn ClassReleaseRemoveLock(DeviceObject: PDEVICE_OBJECT, Irp: PIRP);
}

// ============================================================================
// amd64/arm64 link-gap fns (refs hidden by the i386 fold; live on amd64/arm64).
// class.c StartDevice helpers. ABI extern "C" (all undefs undecorated).
// ============================================================================
const CLASS_TAG_POWER: u32 = 0x574c6353; // 'WLcS'

// class.c:11666  ClasspAllocatePowerProcessIrp — mirror of ClasspAllocateReleaseQueueIrp.
#[no_mangle]
pub unsafe extern "C" fn ClasspAllocatePowerProcessIrp(
    FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION,
) -> NTSTATUS {
    let fdoData = (*FdoExtension).PrivateFdoData;
    let stack_size = (*(*(FdoExtension as PCOMMON_DEVICE_EXTENSION)).LowerDeviceObject).StackSize + 1;
    let irp = ExAllocatePoolWithTag(
        NonPagedPoolNx as _,
        io_size_of_irp(stack_size as u8) as _,
        CLASS_TAG_POWER,
    ) as PIRP;
    (*fdoData).PowerProcessIrp = irp;
    if irp.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    IoInitializeIrp(irp, io_size_of_irp(stack_size as u8), stack_size as i8);
    STATUS_SUCCESS
}

// class.c:12634  ClasspScanForClassHacks — CLEAR_FLAG(Data,INVALID); SET_FLAG(HackFlags,Data).
#[no_mangle]
pub unsafe extern "system" fn ClasspScanForClassHacks(
    FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION,
    Data: usize,
) {
    let fdoData = (*FdoExtension).PrivateFdoData;
    // CLEAR_FLAG(Data, FDO_HACK_INVALID_FLAGS=~FDO_HACK_VALID_FLAGS(0x1F)) => keep valid bits.
    // HackFlags is ULONG_PTR (pointer-sized); cast to *mut usize to work in one int type.
    let p = core::ptr::addr_of_mut!((*fdoData).HackFlags) as *mut usize;
    core::ptr::write_volatile(p, core::ptr::read_volatile(p) | (Data & 0x1F));
}

// class.c:8818  ClasspIsPortable — boot-safe: fixed boot disks are not portable. Full path
// queries IoGetDevicePropertyData(DEVPKEY_Device_InLocalMachineContainer) + BusType; OWED.
#[no_mangle]
pub unsafe extern "C" fn ClasspIsPortable(
    FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION,
    IsPortable: *mut BOOLEAN,
) -> NTSTATUS {
    let _ = FdoExtension;
    core::ptr::write_volatile(IsPortable, FALSE as u8);
    STATUS_SUCCESS
}

// class.c:12651  ClasspScanForSpecialInRegistry — boot-safe no-op (HackFlags default 0 = no
// device quirks). Full path reads Classpnp\HackFlags from the device instance key; OWED.
#[no_mangle]
pub unsafe extern "C" fn ClasspScanForSpecialInRegistry(
    FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION,
) {
    let _ = FdoExtension;
}

// class.c:12507  ClasspInitializeHotplugInfo — boot-safe minimal: zero the struct + set Size.
// Fixed boot disks => non-removable/non-hotplug/no-writecache-override (all zero). Full path
// queries IoGetDeviceProperty(RemovalPolicy) + ClassGetDeviceParameter registry overrides; OWED.
#[no_mangle]
pub unsafe extern "C" fn ClasspInitializeHotplugInfo(
    FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION,
) -> NTSTATUS {
    let fdoData = (*FdoExtension).PrivateFdoData;
    core::ptr::write_bytes(
        core::ptr::addr_of_mut!((*fdoData).HotplugInfo) as *mut u8,
        0,
        core::mem::size_of::<STORAGE_HOTPLUG_INFO>(),
    );
    core::ptr::write_volatile(
        core::ptr::addr_of_mut!((*fdoData).HotplugInfo.Size),
        core::mem::size_of::<STORAGE_HOTPLUG_INFO>() as u32,
    );
    STATUS_SUCCESS
}
