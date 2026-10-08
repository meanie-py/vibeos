use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{_KWAIT_REASON_Executive, FILE_OBJECT_EXTENSION, IoCopyCurrentIrpStackLocationToNext, IoGetCurrentIrpStackLocation, IoSetCompletionRoutine, KEVENT, KeEnterCriticalRegion, KeInitializeEvent, KeLeaveCriticalRegion, KeWaitForSingleObject, PCSZ, PDEVICE_OBJECT, PFILE_OBJECT, PFILE_OBJECT_EXTENSION, PIO_SECURITY_CONTEXT, PIO_STACK_LOCATION, PIRP, _EVENT_TYPE_SynchronizationEvent, _KWAIT_REASON_UserRequest};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{STATUS_DEVICE_DOES_NOT_EXIST, STATUS_INVALID_DEVICE_STATE, TRACE_LEVEL_ERROR, TRACE_LEVEL_INFORMATION};
use crate::types::{ClassSignalCompletion, NO_REMOVE, PCDB, PCOMMON_DEVICE_EXTENSION, PFUNCTIONAL_DEVICE_EXTENSION, PSCSI_REQUEST_BLOCK, PSTORAGE_REQUEST_BLOCK, SCSIOP_MEDIUM_REMOVAL, SCSI_REQUEST_BLOCK, SRB_TYPE_STORAGE_REQUEST_BLOCK, STORAGE_ADDRESS_TYPE_BTL8, _COMMON_DEVICE_EXTENSION, _FUNCTIONAL_DEVICE_EXTENSION, _SRBEXDATATYPE_SrbExDataTypeScsiCdb16 as SrbExDataTypeScsiCdb16, _STORAGE_REQUEST_BLOCK};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{CLASS_TAG_FILE_OBJECT_EXTENSION};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{TRACE_FLAG_INIT, TRACE_FLAG_IOCTL};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE};
use crate::types::{MEDIA_LOCK_TYPE};
use crate::shims::*;
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{AllocateDictionaryEntry, ClassEnableMediaChangeDetection, ClassSendSrbSynchronous, FreeDictionaryEntry, GetDictionaryEntry, InitializeStorageRequestBlock};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{_MEDIA_LOCK_TYPE_SecureMediaLock};
use crate::types::{MEDIA_LOCK_TYPE_SimpleMediaLock as _MEDIA_LOCK_TYPE_SimpleMediaLock, MEDIA_LOCK_TYPE_InternalMediaLock as _MEDIA_LOCK_TYPE_InternalMediaLock};

// Local shadow struct for classpnp's FILE_OBJECT_EXTENSION (sdk/include/ddk/classpnp.h).
// The ntkrnl_sys bindgen version has only TopDeviceObjectHint/FilterContext; classpnp
// defines its own with FileObject/DeviceObject/LockCount/McnDisableCount.
#[repr(C)]
#[allow(non_snake_case, non_camel_case_types, dead_code)]
struct ClasspnpFileObjExt {
    FileObject: PFILE_OBJECT,
    DeviceObject: PDEVICE_OBJECT,
    LockCount: ULONG,
    McnDisableCount: ULONG,
}
#[inline(always)]
unsafe fn foe(p: PFILE_OBJECT_EXTENSION) -> *mut ClasspnpFileObjExt {
    p as *mut ClasspnpFileObjExt
}

// Local shadow struct for _STORAGE_ADAPTER_DESCRIPTOR (SDK extended fields SrbType/AddressType).
#[repr(C)]
#[allow(non_snake_case, non_camel_case_types, dead_code)]
struct LocalStorageAdapterDescriptor {
    Version: u32,
    Size: u32,
    MaximumTransferLength: u32,
    MaximumPhysicalPages: u32,
    AlignmentMask: u32,
    AdapterUsesPio: u8,
    AdapterScansDown: u8,
    CommandQueueing: u8,
    AcceleratedTransfer: u8,
    BusType: u8,
    BusMajorVersion: u16,
    BusMinorVersion: u16,
    SrbType: u8,
    AddressType: u8,
}
#[inline(always)]
unsafe fn adapter_desc_ext(p: *mut core::ffi::c_void) -> *mut LocalStorageAdapterDescriptor {
    p as *mut LocalStorageAdapterDescriptor
}

extern "system" {
    fn ClassAcquireRemoveLockEx(DeviceObject: PDEVICE_OBJECT, Tag: PVOID, File: *const u8, Line: ULONG) -> ULONG;
}

// ---- file-scope preamble (translated) ----
pub const CLASS_INIT_GUID: i32 = 0; // nolint: const-contradiction (create.c:24 defines it to 0 so classp.h only DECLARES the GUIDs here)
pub static mut BreakOnClose: ULONG = 0;
// LockTypeStrings: raw pointer array. Wrapping in a Sync newtype to satisfy static requirements.
struct SyncPcszArr([PCSZ; 3]);
unsafe impl Sync for SyncPcszArr {}
static LockTypeStrings: SyncPcszArr = SyncPcszArr([
    b"Simple\0".as_ptr() as PCSZ,
    b"Secure\0".as_ptr() as PCSZ,
    b"Internal\0".as_ptr() as PCSZ,
]);
#[no_mangle]
pub unsafe extern "system" fn ClassCreateClose(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let commonExtension: *mut _COMMON_DEVICE_EXTENSION =
        (*DeviceObject).DeviceExtension as *mut _COMMON_DEVICE_EXTENSION;
    let removeState: u32 = ClassAcquireRemoveLockEx(DeviceObject, Irp as PVOID, b"\0".as_ptr(), 0);
    let status: NTSTATUS;

    if removeState == NO_REMOVE
        || {
            let _mj = (*(IoGetCurrentIrpStackLocation(Irp))).MajorFunction;
            _mj == IRP_MJ_CLOSE || _mj == IRP_MJ_CLEANUP || _mj == IRP_MJ_SHUTDOWN
        }
    {
        status = ClasspCreateClose(DeviceObject, Irp);

        if NT_SUCCESS(status) {
            let devInfo = (*commonExtension).DevInfo;
            if !devInfo.is_null() {
                let classCreateClosePtr = (*devInfo).ClassCreateClose;
                if !classCreateClosePtr.is_none() {
                    let classCreateCloseFn: unsafe extern "system" fn(
                        PDEVICE_OBJECT,
                        PIRP,
                    ) -> NTSTATUS = core::mem::transmute(classCreateClosePtr);
                    return classCreateCloseFn(DeviceObject, Irp);
                }
            }
        }
    } else {
        status = STATUS_DEVICE_DOES_NOT_EXIST;
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = status;
    ClassReleaseRemoveLock(DeviceObject, Irp);
    ClassCompleteRequest(DeviceObject, Irp, (IO_NO_INCREMENT) as _);
    status
}
#[no_mangle]
pub unsafe extern "C" fn ClasspCreateClose(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    let commonExtension = (*DeviceObject).DeviceExtension as *mut _COMMON_DEVICE_EXTENSION;
    let irpStack = IoGetCurrentIrpStackLocation(Irp);
    let fileObject = (*irpStack).FileObject;
    let mut status: NTSTATUS = STATUS_SUCCESS;

    // PAGED_CODE()

    if (*irpStack).MajorFunction == IRP_MJ_CREATE {
        let securityContext = (*irpStack).Parameters.Create.SecurityContext;

        TracePrint(
            TRACE_LEVEL_INFORMATION,
            TRACE_FLAG_INIT,
            b"ClasspCREATEClose: create received for device %p\n\0".as_ptr() as *const i8,
            DeviceObject,
        );
        TracePrint(
            TRACE_LEVEL_INFORMATION,
            TRACE_FLAG_INIT,
            b"ClasspCREATEClose: desired access %lx\n\0".as_ptr() as *const i8,
            (*securityContext).DesiredAccess,
        );
        TracePrint(
            TRACE_LEVEL_INFORMATION,
            TRACE_FLAG_INIT,
            b"ClasspCREATEClose: file object %p\n\0".as_ptr() as *const i8,
            fileObject,
        );

        NT_ASSERT((BreakOnClose == 0) as u8);

        if !fileObject.is_null() {
            let mut fsContext: PFILE_OBJECT_EXTENSION = core::ptr::null_mut();

            status = AllocateDictionaryEntry(
                core::ptr::addr_of_mut!((*commonExtension).FileObjectDictionary),
                fileObject as u64,
                (core::mem::size_of::<ClasspnpFileObjExt>()) as u32,
                CLASS_TAG_FILE_OBJECT_EXTENSION,
                &mut fsContext as *mut _ as *mut *mut core::ffi::c_void,
            );

            if NT_SUCCESS(status) {
                core::ptr::write_bytes(
                    fsContext as *mut u8,
                    0,
                    core::mem::size_of::<ClasspnpFileObjExt>(),
                );
                (*foe(fsContext)).FileObject = fileObject;
                (*foe(fsContext)).DeviceObject = DeviceObject;
            } else if status == STATUS_OBJECT_NAME_COLLISION {
                status = STATUS_SUCCESS;
            }
        }
    } else {
        TracePrint(
            TRACE_LEVEL_INFORMATION,
            TRACE_FLAG_INIT,
            b"ClasspCreateCLOSE: close received for device %p\n\0".as_ptr() as *const i8,
            DeviceObject,
        );
        TracePrint(
            TRACE_LEVEL_INFORMATION,
            TRACE_FLAG_INIT,
            b"ClasspCreateCLOSE: file object %p\n\0".as_ptr() as *const i8,
            fileObject,
        );

        if !fileObject.is_null() {
            let fsContext = ClassGetFsContext(commonExtension, fileObject);

            TracePrint(
                TRACE_LEVEL_INFORMATION,
                TRACE_FLAG_INIT,
                b"ClasspCreateCLOSE: file extension %p\n\0".as_ptr() as *const i8,
                fsContext,
            );

            if !fsContext.is_null() {
                TracePrint(
                    TRACE_LEVEL_INFORMATION,
                    TRACE_FLAG_INIT,
                    b"ClasspCreateCLOSE: extension is ours - freeing\n\0".as_ptr() as *const i8,
                );
                NT_ASSERT((BreakOnClose == 0) as u8);

                ClasspCleanupProtectedLocks(fsContext);
                ClasspCleanupDisableMcn(fsContext);

                FreeDictionaryEntry(
                    core::ptr::addr_of_mut!((*commonExtension).FileObjectDictionary),
                    (fsContext).cast(),
                );
            }
        }
    }

    TracePrint(
        TRACE_LEVEL_INFORMATION,
        TRACE_FLAG_INIT,
        b"ClasspCreateClose: %s for devobj %p\n\0".as_ptr() as *const i8,
        if NT_SUCCESS(status) {
            b"Success\0".as_ptr() as *const i8
        } else {
            b"FAILED\0".as_ptr() as *const i8
        },
        DeviceObject,
    );

    if NT_SUCCESS(status) {
        let mut event: KEVENT = core::mem::zeroed();
        KeInitializeEvent(&mut event, _EVENT_TYPE_SynchronizationEvent, 0u8);

        IoCopyCurrentIrpStackLocationToNext(Irp);
        IoSetCompletionRoutine(
            Irp,
            Some(ClassSignalCompletion),
            &mut event as *mut _ as *mut core::ffi::c_void,
            1u8,
            1u8,
            1u8,
        );

        status = IoCallDriver((*commonExtension).LowerDeviceObject, Irp);

        if status == STATUS_PENDING {
            KeWaitForSingleObject(
                &mut event as *mut _ as PVOID,
                _KWAIT_REASON_Executive,
                0i8,
                0u8,
                core::ptr::null_mut(),
            );
            status = (*Irp).IoStatus.__bindgen_anon_1.Status;
        }

        if !NT_SUCCESS(status) {
            TracePrint(
                TRACE_LEVEL_ERROR,
                TRACE_FLAG_INIT,
                b"ClasspCreateClose: Lower driver failed, but we succeeded.  This is a problem, lock counts will be out of sync between levels.\n\0".as_ptr() as *const i8,
            );
        }
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn ClasspCleanupProtectedLocks(FsContext: PFILE_OBJECT_EXTENSION) {
    let commonExtension = (*(*foe(FsContext)).DeviceObject).DeviceExtension as PCOMMON_DEVICE_EXTENSION;
    let fdoExtension = (*commonExtension).PartitionZeroExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let mut newDeviceLockCount: u32 = 1;

    TracePrint(TRACE_LEVEL_INFORMATION, TRACE_FLAG_INIT,
               b"ClasspCleanupProtectedLocks called for %p\n\0".as_ptr() as PCSZ,
               (*foe(FsContext)).DeviceObject as PVOID);
    TracePrint(TRACE_LEVEL_INFORMATION, TRACE_FLAG_INIT,
               b"ClasspCleanupProtectedLocks - FsContext %p is locked %d times\n\0".as_ptr() as PCSZ,
               FsContext as PVOID,
               (*foe(FsContext)).LockCount as i32);

    NT_ASSERT((BreakOnClose == 0) as u8);

    KeEnterCriticalRegion();
    KeWaitForSingleObject(
        core::ptr::addr_of_mut!((*fdoExtension).EjectSynchronizationEvent) as PVOID,
        _KWAIT_REASON_UserRequest,
        0i8,
        0u8,
        core::ptr::null_mut(),
    );

    if (*foe(FsContext)).LockCount != 0 {
        let lock_count_ptr = core::ptr::addr_of_mut!((*foe(FsContext)).LockCount);
        loop {
            InterlockedDecrement(lock_count_ptr as *mut i32);
            newDeviceLockCount = InterlockedDecrement(
                core::ptr::addr_of_mut!((*fdoExtension).ProtectedLockCount) as *mut i32,
            ) as u32;
            if *lock_count_ptr <= 0 {
                break;
            }
        }

        TracePrint(TRACE_LEVEL_INFORMATION, TRACE_FLAG_INIT,
                   b"ClasspCleanupProtectedLocks: FDO secured lock count = %d lock count = %d\n\0"
                       .as_ptr() as PCSZ,
                   (*fdoExtension).ProtectedLockCount as i32,
                   (*fdoExtension).LockCount as i32);

        if newDeviceLockCount == 0 && (*fdoExtension).LockCount == 0 {
            let mut srb: SCSI_REQUEST_BLOCK = core::mem::zeroed();
            let mut srbExBuffer: [u8; CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE] =
                [0u8; CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE];
            let srbEx = srbExBuffer.as_mut_ptr() as PSTORAGE_REQUEST_BLOCK;
            let mut cdb: PCDB = core::ptr::null_mut();
            let srbPtr: PSCSI_REQUEST_BLOCK;

            TracePrint(TRACE_LEVEL_INFORMATION, TRACE_FLAG_INIT,
                       b"ClasspCleanupProtectedLocks: FDO lock count dropped to zero\n\0"
                           .as_ptr() as PCSZ);

            if (*adapter_desc_ext((*fdoExtension).AdapterDescriptor as *mut core::ffi::c_void)).SrbType as u32 == SRB_TYPE_STORAGE_REQUEST_BLOCK {
                let status = InitializeStorageRequestBlock(
                    srbEx,
                    STORAGE_ADDRESS_TYPE_BTL8 as u16,
                    CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE as u32,
                    1,
                    SrbExDataTypeScsiCdb16,
                );
                if NT_SUCCESS(status) {
                    (*srbEx).TimeOutValue = (*fdoExtension).TimeOutValue;
                    SrbSetCdbLength((srbEx).cast(), 6);
                    cdb = SrbGetCdb((srbEx).cast());
                    srbPtr = srbEx as PSCSI_REQUEST_BLOCK;
                } else {
                    NT_ASSERT(0);
                    srb.TimeOutValue = (*fdoExtension).TimeOutValue;
                    srb.CdbLength = 6;
                    srbPtr = &mut srb as PSCSI_REQUEST_BLOCK;
                    cdb = core::ptr::addr_of_mut!((*srbPtr).Cdb) as PCDB;
                }
            } else {
                srb.TimeOutValue = (*fdoExtension).TimeOutValue;
                srb.CdbLength = 6;
                srbPtr = &mut srb as PSCSI_REQUEST_BLOCK;
                cdb = core::ptr::addr_of_mut!((*srbPtr).Cdb) as PCDB;
            }

            (*cdb).MEDIA_REMOVAL.as_mut().OperationCode = (SCSIOP_MEDIUM_REMOVAL) as u8;
            (*cdb).MEDIA_REMOVAL.as_mut().set_Prevent(0u8);

            let status = ClassSendSrbSynchronous(
                (*fdoExtension).__bindgen_anon_1.__bindgen_anon_1.as_ref().DeviceObject,
                srbPtr,
                core::ptr::null_mut(),
                0,
                0u8,
            );

            TracePrint(TRACE_LEVEL_INFORMATION, TRACE_FLAG_INIT,
                       b"ClasspCleanupProtectedLocks: unlock request to drive returned status %lx\n\0"
                           .as_ptr() as PCSZ,
                       status as i32);
        }
    }

    KeSetEvent(
        core::ptr::addr_of_mut!((*fdoExtension).EjectSynchronizationEvent) as *mut core::ffi::c_void,
        (IO_NO_INCREMENT) as i32,
        0u8,
    );
    KeLeaveCriticalRegion();
}
#[no_mangle]
pub unsafe extern "C" fn ClasspCleanupDisableMcn(FsContext: PFILE_OBJECT_EXTENSION) {
    let common_extension = (*(*foe(FsContext)).DeviceObject).DeviceExtension as *mut _COMMON_DEVICE_EXTENSION;
    let fdo_extension = (*common_extension).PartitionZeroExtension;

    while (*foe(FsContext)).McnDisableCount != 0 {
        (*foe(FsContext)).McnDisableCount -= 1;
        ClassEnableMediaChangeDetection(fdo_extension);
    }
}
#[no_mangle]
pub unsafe extern "C" fn ClasspEjectionControl(
    Fdo: PDEVICE_OBJECT,
    Irp: PIRP,
    LockType: MEDIA_LOCK_TYPE,
    Lock: BOOLEAN,
) -> NTSTATUS {
    let FdoExtension = (*Fdo).DeviceExtension as *mut _FUNCTIONAL_DEVICE_EXTENSION;
    let commonExtension = FdoExtension as PCOMMON_DEVICE_EXTENSION;
    let mut fsContext: PFILE_OBJECT_EXTENSION = core::ptr::null_mut();
    let mut status: NTSTATUS;
    let mut srb: PSCSI_REQUEST_BLOCK = core::ptr::null_mut();
    let mut countChanged: BOOLEAN = 0u8;

    KeEnterCriticalRegion();
    status = KeWaitForSingleObject(
        core::ptr::addr_of_mut!((*FdoExtension).EjectSynchronizationEvent) as PVOID,
        _KWAIT_REASON_UserRequest,
        0i8,
        0u8,
        core::ptr::null_mut(),
    );
    if status != STATUS_SUCCESS {
        // NT_ASSERT disabled per rules
    }
    TracePrint(
        TRACE_LEVEL_INFORMATION,
        TRACE_FLAG_IOCTL,
        b"ClasspEjectionControl: Received request for %s lock type\n\0"
            .as_ptr() as PCHAR,
        LockTypeStrings.0[LockType as usize],
    );

    'sehtry: {
        let cdb: PCDB;

        if LockType == _MEDIA_LOCK_TYPE_SecureMediaLock {
            let irpStack = IoGetCurrentIrpStackLocation(Irp);
            let fileObject = (*irpStack).FileObject;
            if fileObject != core::ptr::null_mut() {
                fsContext = ClassGetFsContext(commonExtension, fileObject);
            }
            if fsContext == core::ptr::null_mut() {
                status = STATUS_INVALID_PARAMETER;
                break 'sehtry;
            }
        }

        if Lock != 0u8 {
            match LockType {
                _MEDIA_LOCK_TYPE_SimpleMediaLock => {
                    (*FdoExtension).LockCount += 1;
                    countChanged = 1u8;
                }
                _MEDIA_LOCK_TYPE_SecureMediaLock => {
                    (*foe(fsContext)).LockCount += 1;
                    (*FdoExtension).ProtectedLockCount += 1;
                    countChanged = 1u8;
                }
                _MEDIA_LOCK_TYPE_InternalMediaLock => {
                    (*FdoExtension).InternalLockCount += 1;
                    countChanged = 1u8;
                }
                _ => {}
            }
        } else {
            match LockType {
                _MEDIA_LOCK_TYPE_SimpleMediaLock => {
                    if (*FdoExtension).LockCount != 0 {
                        (*FdoExtension).LockCount -= 1;
                        countChanged = 1u8;
                    }
                }
                _MEDIA_LOCK_TYPE_SecureMediaLock => {
                    if (*foe(fsContext)).LockCount == 0 {
                        status = STATUS_INVALID_DEVICE_STATE;
                        break 'sehtry;
                    }
                    (*foe(fsContext)).LockCount -= 1;
                    (*FdoExtension).ProtectedLockCount -= 1;
                    countChanged = 1u8;
                }
                _MEDIA_LOCK_TYPE_InternalMediaLock => {
                    // NT_ASSERT(FdoExtension->InternalLockCount != 0);
                    (*FdoExtension).InternalLockCount -= 1;
                    countChanged = 1u8;
                }
                _ => {}
            }

            if (*FdoExtension).ProtectedLockCount != 0
                || (*FdoExtension).InternalLockCount != 0
                || (*FdoExtension).LockCount != 0
            {
                status = STATUS_SUCCESS;
                break 'sehtry;
            }
        }

        status = STATUS_SUCCESS;
        if ((*Fdo).Characteristics & FILE_REMOVABLE_MEDIA) != 0 {
            srb = ntkrnl_sys::inline::ex::ExAllocateFromNPagedLookasideList(
                core::ptr::addr_of_mut!((*FdoExtension).__bindgen_anon_1.CommonExtension.as_mut().SrbLookasideList).cast(),
            ) as PSCSI_REQUEST_BLOCK;
            if srb == core::ptr::null_mut() {
                status = STATUS_INSUFFICIENT_RESOURCES;
                break 'sehtry;
            }

            if (*adapter_desc_ext((*FdoExtension).AdapterDescriptor as *mut core::ffi::c_void)).SrbType as u32 == SRB_TYPE_STORAGE_REQUEST_BLOCK {
                status = InitializeStorageRequestBlock(
                    srb as *mut _STORAGE_REQUEST_BLOCK,
                    STORAGE_ADDRESS_TYPE_BTL8 as u16,
                    CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE as u32,
                    1,
                    SrbExDataTypeScsiCdb16,
                );
                if !NT_SUCCESS(status) {
                    // NT_ASSERT(FALSE);
                    break 'sehtry;
                }
            } else {
                core::ptr::write_bytes(
                    srb as *mut u8,
                    0,
                    core::mem::size_of::<SCSI_REQUEST_BLOCK>(),
                );
            }

            SrbSetCdbLength((srb).cast(), 6);
            cdb = SrbGetCdb((srb).cast());
            // NT_ASSERT(cdb != NULL);
            (*cdb).MEDIA_REMOVAL.as_mut().OperationCode = (SCSIOP_MEDIUM_REMOVAL) as u8;
            (*cdb).MEDIA_REMOVAL.as_mut().set_Prevent(Lock);
            SrbSetTimeOutValue((srb).cast(), (*FdoExtension).TimeOutValue);

            status = ClassSendSrbSynchronous(
                (*FdoExtension).__bindgen_anon_1.__bindgen_anon_1.as_ref().DeviceObject,
                srb,
                core::ptr::null_mut(),
                0,
                0u8,
            );
        }
    }

    // _SEH2_FINALLY block
    if !NT_SUCCESS(status) {
        TracePrint(
            TRACE_LEVEL_INFORMATION,
            TRACE_FLAG_IOCTL,
            b"ClasspEjectionControl: FAILED status %x -- reverting lock counts\n\0"
                .as_ptr() as PCHAR,
            status,
        );

        if countChanged != 0u8 {
            if Lock != 0u8 {
                match LockType {
                    _MEDIA_LOCK_TYPE_SimpleMediaLock => {
                        (*FdoExtension).LockCount -= 1;
                    }
                    _MEDIA_LOCK_TYPE_SecureMediaLock => {
                        (*foe(fsContext)).LockCount -= 1;
                        (*FdoExtension).ProtectedLockCount -= 1;
                    }
                    _MEDIA_LOCK_TYPE_InternalMediaLock => {
                        (*FdoExtension).InternalLockCount -= 1;
                    }
                    _ => {}
                }
            } else {
                match LockType {
                    _MEDIA_LOCK_TYPE_SimpleMediaLock => {
                        (*FdoExtension).LockCount += 1;
                    }
                    _MEDIA_LOCK_TYPE_SecureMediaLock => {
                        (*foe(fsContext)).LockCount += 1;
                        (*FdoExtension).ProtectedLockCount += 1;
                    }
                    _MEDIA_LOCK_TYPE_InternalMediaLock => {
                        (*FdoExtension).InternalLockCount += 1;
                    }
                    _ => {}
                }
            }
        }
    } else {
        TracePrint(
            TRACE_LEVEL_INFORMATION,
            TRACE_FLAG_IOCTL,
            b"ClasspEjectionControl: Succeeded\n\0".as_ptr() as PCHAR,
        );
    }

    TracePrint(
        TRACE_LEVEL_INFORMATION,
        TRACE_FLAG_IOCTL,
        b"ClasspEjectionControl: Current Counts: Internal: %x  Secure: %x  Simple: %x\n\0"
            .as_ptr() as PCHAR,
        (*FdoExtension).InternalLockCount,
        (*FdoExtension).ProtectedLockCount,
        (*FdoExtension).LockCount,
    );

    KeSetEvent(
        core::ptr::addr_of_mut!((*FdoExtension).EjectSynchronizationEvent) as *mut core::ffi::c_void,
        IO_NO_INCREMENT as i32,
        0u8,
    );
    KeLeaveCriticalRegion();

    if srb != core::ptr::null_mut() {
        ClassFreeOrReuseSrb(FdoExtension, srb);
    }

    status
}
#[no_mangle]
pub unsafe extern "system" fn ClassGetFsContext(
    CommonExtension: *mut _COMMON_DEVICE_EXTENSION,
    FileObject: PFILE_OBJECT
) -> PFILE_OBJECT_EXTENSION {
    (GetDictionaryEntry(
        core::ptr::addr_of_mut!((*CommonExtension).FileObjectDictionary),
        FileObject as u64
    )).cast()
}
