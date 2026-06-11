use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::ddk::{DPFLTR_CLASSPNP_ID, _KWAIT_REASON_Executive, GUID, IO_COMPLETION_ROUTINE, _IO_STATUS_BLOCK, IRP, IoBuildDeviceIoControlRequest, IoFreeWorkItem, IoGetAttachedDeviceReference, IoGetCurrentIrpStackLocation, IoSetNextIrpStackLocation, IoStartNextPacket, KEVENT, KeInitializeEvent, KeWaitForSingleObject, LARGE_INTEGER, LIST_ENTRY, PDEVICE_OBJECT, PIO_STACK_LOCATION, PIO_WORKITEM, PIRP, _EVENT_TYPE_SynchronizationEvent};
use core::ffi::c_void;

extern "C" {
    fn DbgPrintEx(ComponentId: ULONG, Level: ULONG, Format: PCSTR, ...) -> NTSTATUS;
    fn PoUnregisterPowerSettingCallback(Handle: PVOID) -> NTSTATUS;
    fn ClassAcquireRemoveLock(DeviceObject: PDEVICE_OBJECT, Tag: PIRP) -> NTSTATUS;
    fn ClassReleaseRemoveLock(DeviceObject: PDEVICE_OBJECT, Tag: PIRP);
    fn ClassCompleteRequest(DeviceObject: PDEVICE_OBJECT, Irp: PIRP, PriorityBoost: i8);
    fn ClasspRetryDpcTimer(fdoData: *mut CLASS_PRIVATE_FDO_DATA);
    fn NT_ASSERT(condition: u8);
    fn TracePrint(level: u32, flags: u32, format: PCHAR, ...);
    fn ULongToPtr(val: u32) -> PVOID;
    fn UNREFERENCED_PARAMETER(param: PVOID);
    fn _Analysis_assume_(expr: BOOLEAN);
}

// ---- file-scope preamble (translated) ----
pub const CLASS_INIT_GUID: i32 = 1;
pub const DEBUG_MAIN_SOURCE: i32 = 1;

pub static mut ClassPnpAllowUnload: u32 = 1;
pub static mut ClassMaxInterleavePerCriticalIo: u32 = CLASS_MAX_INTERLEAVE_PER_CRITICAL_IO;
pub static Magic10000: LARGE_INTEGER = LARGE_INTEGER {
    u: core::mem::ManuallyDrop::new(_LARGE_INTEGER__bindgen_ty_1 {
        __bindgen_anon_1: core::mem::ManuallyDrop::new(_LARGE_INTEGER__bindgen_ty_1__bindgen_ty_1 {
            LowPart: 0xe219652c,
            HighPart: 0xd1b71758,
        }),
    }),
};
pub static mut StoragePredictFailureDPSGuid: GUID = WDI_STORAGE_PREDICT_FAILURE_DPS_GUID;

pub const FirstDriveLetter: u8 = b'C';
pub const LastDriveLetter: u8 = b'Z';

pub static mut UseQPCTime: u8 = 0;
pub static mut InitSecurityCookie: u8 = 0;
pub static mut MaxTokenOperationListIdentifier: u32 = MAX_TOKEN_LIST_IDENTIFIERS;
pub static mut TokenOperationListIdentifier: u32 = 0xFFFFFFFF;
pub static mut IdlePowerFDOList: LIST_ENTRY = LIST_ENTRY {
    Flink: core::ptr::null_mut(),
    Blink: core::ptr::null_mut(),
};
pub static mut IdlePowerFDOListMutex: KGUARDED_MUTEX = KGUARDED_MUTEX {
    Count: 0,
    Owner: core::ptr::null_mut(),
    Contention: 0,
    Gate: _KGATE {
        Header: _DISPATCHER_HEADER {
            __bindgen_anon_1: core::mem::ManuallyDrop::new(_DISPATCHER_HEADER__bindgen_ty_1 {
                Lock: 0,
            }),
            SignalState: 0,
            WaitListHead: LIST_ENTRY {
                Flink: core::ptr::null_mut(),
                Blink: core::ptr::null_mut(),
            },
        },
    },
    __bindgen_anon_1: core::mem::ManuallyDrop::new(_KGUARDED_MUTEX__bindgen_ty_1 {
        KernelApcDisable: 0,
    }),
};
pub static mut PowerSettingNotificationHandle: PVOID = core::ptr::null_mut();
pub static mut ScreenStateNotificationHandle: PVOID = core::ptr::null_mut();
pub static mut DiskIdleTimeoutInMS: u32 = 0xFFFFFFFF;
#[no_mangle]
pub unsafe extern "system" fn ClasspRetryRequestDpc(
    Dpc: PKDPC,
    DeferredContext: PVOID,
    Arg1: PVOID,
    Arg2: PVOID,
) {

    let fdoExtension: PFUNCTIONAL_DEVICE_EXTENSION;
    let commonExtension: PCOMMON_DEVICE_EXTENSION;
    let fdoData: *mut CLASS_PRIVATE_FDO_DATA;
    let mut retryList: *mut CLASS_RETRY_INFO;
    let mut irql: KIRQL = 0;
    let DeviceObject = DeferredContext as PDEVICE_OBJECT;

    UNREFERENCED_PARAMETER(Dpc as PVOID);
    UNREFERENCED_PARAMETER(Arg1);
    UNREFERENCED_PARAMETER(Arg2);

    if DeferredContext.is_null() {
        NT_ASSERT(0);
        return;
    }

    commonExtension = (*DeviceObject).DeviceExtension as PCOMMON_DEVICE_EXTENSION;
    NT_ASSERT((*commonExtension).IsFdo);
    fdoExtension = (*DeviceObject).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    fdoData = (*fdoExtension).PrivateFdoData;

    KeAcquireSpinLock(core::ptr::addr_of_mut!((*fdoData).Retry.Lock), core::ptr::addr_of_mut!(irql));
    {
        let mut now: LARGE_INTEGER = core::mem::zeroed();
        KeQueryTickCount(core::ptr::addr_of_mut!(now) as *mut i64);

        if (*core::ptr::addr_of!(now.QuadPart)) < (*core::ptr::addr_of!((*fdoData).Retry.Tick.QuadPart)) {
            ClasspRetryDpcTimer(fdoData);
            retryList = core::ptr::null_mut();
        } else {
            retryList = (*fdoData).Retry.ListHead;
            (*fdoData).Retry.ListHead = core::ptr::null_mut();
            (*fdoData).Retry.Delta.QuadPart = 0i64;
            (*fdoData).Retry.Tick.QuadPart = 0i64;
        }
    }
    KeReleaseSpinLock(core::ptr::addr_of_mut!((*fdoData).Retry.Lock), irql);

    while !retryList.is_null() {
        let irp: PIRP;

        irp = (retryList as *const u8).offset(-(core::mem::offset_of!(IRP, Tail.Overlay.DriverContext) as isize)) as PIRP;
        TracePrint(TRACE_LEVEL_INFORMATION, TRACE_FLAG_GENERAL, b"ClassRetry:  -- %p\n\0".as_ptr() as PCHAR, irp);
        retryList = (*retryList).Next;
#[no_mangle]
pub unsafe extern "C" fn DllUnload() -> NTSTATUS {

    DbgPrintEx(
        DPFLTR_CLASSPNP_ID,
        DPFLTR_INFO_LEVEL,
        b"classpnp.sys is now unloading\n\0".as_ptr() as PCSTR,
    );

    if !PowerSettingNotificationHandle.is_null() {
        PoUnregisterPowerSettingCallback(PowerSettingNotificationHandle);
        PowerSettingNotificationHandle = core::ptr::null_mut();
    }

    if !ScreenStateNotificationHandle.is_null() {
        PoUnregisterPowerSettingCallback(ScreenStateNotificationHandle);
        ScreenStateNotificationHandle = core::ptr::null_mut();
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn ClasspStartIo(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) {

    let commonExtension = (*DeviceObject).DeviceExtension as PCOMMON_DEVICE_EXTENSION;

    if (*commonExtension).IsRemoved != 0 {
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_DEVICE_DOES_NOT_EXIST;

        ClassAcquireRemoveLock(DeviceObject, ClasspStartIo as PIRP);

        ClassReleaseRemoveLock(DeviceObject, Irp);
        ClassCompleteRequest(DeviceObject, Irp, IO_DISK_INCREMENT);
        IoStartNextPacket(DeviceObject, 1u8);

        ClassReleaseRemoveLock(DeviceObject, ClasspStartIo as PIRP);

        return;
    }

    ((*(*commonExtension).DriverExtension).InitData.ClassStartIo)(DeviceObject, Irp);
}

#[cfg(debug_assertions)]
        {
            (*irp).Tail.Overlay.DriverContext[0] = ULongToPtr(0xdddddddd);
            (*irp).Tail.Overlay.DriverContext[1] = ULongToPtr(0xdddddddd);
            (*irp).Tail.Overlay.DriverContext[2] = ULongToPtr(0xdddddddd);
            (*irp).Tail.Overlay.DriverContext[3] = ULongToPtr(0xdddddddd);
        }

        if NO_REMOVE == core::intrinsics::atomic_cxchg_acqrel_acquire(
            core::ptr::addr_of_mut!((*commonExtension).IsRemoved) as *mut i32,
            REMOVE_PENDING,
            REMOVE_PENDING,
        ).0 {
            IoCallDriver((*commonExtension).LowerDeviceObject, irp);
        } else {
            let irpStack: PIO_STACK_LOCATION;

            IoSetNextIrpStackLocation(irp);

            irpStack = IoGetCurrentIrpStackLocation(irp);

            if (*irpStack).MajorFunction == IRP_MJ_SCSI {
                let srb = (*irpStack).Parameters.Scsi.Srb;

                if !srb.is_null() {
                    (*srb).SrbStatus = SRB_STATUS_NO_DEVICE;
                }
            }

            (*irpStack).Parameters.Others.Argument4 = core::ptr::null_mut();

            (*irp).IoStatus.__bindgen_anon_1.Status = STATUS_NO_SUCH_DEVICE;
            (*irp).IoStatus.Information = 0;
            IoCompleteRequest(irp, IO_NO_INCREMENT);
        }
    }
}
#[no_mangle]
pub unsafe extern "system" fn ClasspUpdateDiskProperties(
    Fdo: PDEVICE_OBJECT,
    Context: PVOID
) {
    let mut topOfStack: PDEVICE_OBJECT;
    let mut ioStatus: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut status: NTSTATUS = STATUS_SUCCESS;
    let mut event: KEVENT = core::mem::zeroed();
    let mut irp: PIRP = core::ptr::null_mut();
    let fdoExt: PFUNCTIONAL_DEVICE_EXTENSION = (*Fdo).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let fdoData: PCLASS_PRIVATE_FDO_DATA = (*fdoExt).PrivateFdoData;
    let WorkItem: PIO_WORKITEM = Context as PIO_WORKITEM;

    KeInitializeEvent(&mut event, _EVENT_TYPE_SynchronizationEvent, 0u8);

    topOfStack = IoGetAttachedDeviceReference(Fdo);

    irp = IoBuildDeviceIoControlRequest(
        IOCTL_DISK_UPDATE_PROPERTIES,
        topOfStack,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
        0,
        0u8,
        &mut event,
        &mut ioStatus
    );

    if !irp.is_null() {
        status = IoCallDriver(topOfStack, irp);
        if status == STATUS_PENDING {
            KeWaitForSingleObject(
                &mut event as *mut _ as *mut _,
                _KWAIT_REASON_Executive,
                0i32 as i8,
                0u8,
                core::ptr::null_mut()
            );
            status = ioStatus.__bindgen_anon_1.Status;
        }
    } else {
        status = STATUS_INSUFFICIENT_RESOURCES;
    }

    core::arch::asm!(
        "lock xchg [{0}], {1}",
        in(reg) core::ptr::addr_of_mut!((*fdoData).UpdateDiskPropertiesWorkItemActive),
        in(reg) 0i32,
        options(nostack)
    );

    if !NT_SUCCESS(status) {
        TracePrint((
            TRACE_LEVEL_WARNING,
            TRACE_FLAG_INIT,
            b"ClasspUpdateDiskProperties: Disk property update for fdo %p failed with status 0x%X.\n\0".as_ptr() as PCHAR,
            Fdo,
            status
        ));
    } else {
        TracePrint((
            TRACE_LEVEL_INFORMATION,
            TRACE_FLAG_INIT,
            b"ClasspUpdateDiskProperties: Drive capacity has changed for %p.\n\0".as_ptr() as PCHAR,
            Fdo
        ));
    }
    
    ObDereferenceObject(topOfStack as *mut _);

    if !WorkItem.is_null() {
        IoFreeWorkItem(WorkItem);
    }
}

#[no_mangle]
pub unsafe extern "system" fn ClassDeviceControlDispatch(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP
) -> NTSTATUS {
    let commonExtension = (*DeviceObject).DeviceExtension as *mut COMMON_DEVICE_EXTENSION;
    let isRemoved: ULONG;

    isRemoved = ClassAcquireRemoveLock(DeviceObject, Irp);
    _Analysis_assume_(isRemoved as BOOLEAN);
    if isRemoved != 0 {
        ClassReleaseRemoveLock(DeviceObject, Irp);

        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_DEVICE_DOES_NOT_EXIST;
        ClassCompleteRequest(DeviceObject, Irp, IO_NO_INCREMENT as i8);
        return STATUS_DEVICE_DOES_NOT_EXIST;
    }

    NT_ASSERT(((*(*commonExtension).DevInfo).ClassDeviceControl.is_some()) as BOOLEAN);

    return (*(*commonExtension).DevInfo).ClassDeviceControl.unwrap()(DeviceObject, Irp);
}
