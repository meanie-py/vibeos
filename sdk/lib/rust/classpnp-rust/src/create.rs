use crate::types::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{_KWAIT_REASON_Executive, FILE_OBJECT_EXTENSION, IoCopyCurrentIrpStackLocationToNext, IoGetCurrentIrpStackLocation, IoSetCompletionRoutine, KEVENT, KeEnterCriticalRegion, KeInitializeEvent, KeLeaveCriticalRegion, KeWaitForSingleObject, PCSZ, PDEVICE_OBJECT, PFILE_OBJECT, PFILE_OBJECT_EXTENSION, PIO_SECURITY_CONTEXT, PIO_STACK_LOCATION, PIRP, _EVENT_TYPE_SynchronizationEvent, _KWAIT_REASON_UserRequest};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{_CDB as CDB};
use ntkrnl_sys::ddk::{DEVICE_OBJECT, FILE_OBJECT, IRP};
use ntkrnl_sys::driver_ddk::{FILE_REMOVABLE_MEDIA};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{STATUS_INVALID_DEVICE_STATE};

extern "C" {
    fn PAGED_CODE();
    fn NT_ASSERT(condition: i32);
    fn TracePrint(level: u32, flag: u32, format: *const i8, ...);
    fn InitializeStorageRequestBlock( srb: *mut STORAGE_REQUEST_BLOCK, address_type: u32, size: u32, count: u32, data_type: u32, ) -> i32; // NTSTATUS fn SrbSetCdbLength(srb: *mut STORAGE_REQUEST_BLOCK, length: u8);
    fn SrbGetCdb(srb: *mut STORAGE_REQUEST_BLOCK) -> *mut CDB;
    fn ClassEnableMediaChangeDetection(fdoExtension: PFUNCTIONAL_DEVICE_EXTENSION);
    fn AllocateDictionaryEntry( dict: *mut DICTIONARY, key: ULONGLONG, size: ULONG, tag: ULONG, entry: *mut *mut c_void, ) -> NTSTATUS;
    fn FreeDictionaryEntry( dict: *mut DICTIONARY, entry: *mut c_void, );
    fn ClassSignalCompletion( DeviceObject: PDEVICE_OBJECT, Irp: PIRP, Context: *mut c_void, ) -> NTSTATUS;
    fn ClasspAllocateSrb(FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION) -> *mut core::ffi::c_void;
    fn RtlZeroMemory(srb: *mut SCSI_REQUEST_BLOCK, size: usize);
    fn SrbSetCdbLength(srb: *mut SCSI_REQUEST_BLOCK, length: u8);
    fn SrbSetTimeOutValue(srb: *mut SCSI_REQUEST_BLOCK, timeout: u32);
    fn ClassSendSrbSynchronous( deviceObject: PDEVICE_OBJECT, srb: *mut SCSI_REQUEST_BLOCK, arg: *mut core::ffi::c_void, arg1: u32, arg2: BOOLEAN, ) -> NTSTATUS;
    fn ClassFreeOrReuseSrb(FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION, srb: *mut SCSI_REQUEST_BLOCK);
    fn ClassAcquireRemoveLock(DeviceObject: *mut DEVICE_OBJECT, Irp: *mut IRP) -> u32;
    fn ClassReleaseRemoveLock(DeviceObject: *mut DEVICE_OBJECT, Irp: *mut IRP);
    fn ClassCompleteRequest(DeviceObject: *mut DEVICE_OBJECT, Irp: *mut IRP, PriorityBoost: i32);
}

// ---- file-scope preamble (translated) ----
pub const CLASS_INIT_GUID: i32 = 0;

pub static mut BreakOnClose: u32 = 0;

pub static LockTypeStrings: [*const i8; 3] = [
    b"Simple\0".as_ptr() as *const i8,
    b"Secure\0".as_ptr() as *const i8,
    b"Internal\0".as_ptr() as *const i8,
];
// Forward declarations for symbols not in the import list

// Constant definitions
const CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE: usize = 0x30; // typical, adjust as needed
const SRB_TYPE_STORAGE_REQUEST_BLOCK: u32 = 0; // placeholder
const STORAGE_ADDRESS_TYPE_BTL8: u32 = 0; // placeholder
const SrbExDataTypeScsiCdb16: u32 = 0; // placeholder
const TRACE_LEVEL_INFORMATION: u32 = 4; // typical, adjust
const TRACE_FLAG_INIT: u32 = 0; // placeholder
const SCSIOP_MEDIUM_REMOVAL: u8 = 0x1E; // typical SCSI opcode for medium removal

const TRACE_LEVEL_INFORMATION: u32 = 0; // TODO: actual value unknown
const TRACE_FLAG_MCN: u32 = 0; // TODO: actual value unknown

const CLASS_TAG_FILE_OBJECT_EXTENSION: u32 = 0;
const TRACE_LEVEL_INFORMATION: u32 = 2;
const TRACE_FLAG_INIT: u32 = 0x1;
const TRACE_LEVEL_ERROR: u32 = 1;

const NO_REMOVE: u32 = 0;
const STATUS_DEVICE_DOES_NOT_EXIST: i32 = 0xC0000184u32 as i32;
const IRP_MJ_CLEANUP: u8 = 0x12;
#[no_mangle]
pub unsafe extern "C" fn ClassCreateClose(
    device_object: *mut DEVICE_OBJECT,
    irp: *mut IRP,
) -> i32 {
    // PAGED_CODE();
    let common_extension = (*device_object).DeviceExtension as *mut COMMON_DEVICE_EXTENSION;
    let remove_state: u32 = ClassAcquireRemoveLock(device_object, irp);
    let status: i32;
    if (remove_state == NO_REMOVE) ||
       ((*IoGetCurrentIrpStackLocation(irp)).MajorFunction == IRP_MJ_CLEANUP) {
        status = ClasspCreateClose(device_object, irp);
        if status >= 0 {
            let fn_ptr = (*(*common_extension).DevInfo).ClassCreateClose;
            if !fn_ptr.is_null() {
                return fn_ptr(device_object, irp);
            }
        }
    } else {
        status = STATUS_DEVICE_DOES_NOT_EXIST;
    }
    (*irp).IoStatus.__bindgen_anon_1.Status = status;
    ClassReleaseRemoveLock(device_object, irp);
    ClassCompleteRequest(device_object, irp, IO_NO_INCREMENT.into());
    return status;
}
#[no_mangle]
pub unsafe extern "C" fn ClasspCreateClose(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let common_extension = (*DeviceObject).DeviceExtension as *mut COMMON_DEVICE_EXTENSION;
    let irp_stack = IoGetCurrentIrpStackLocation(Irp);
    let file_object = (*irp_stack).FileObject;
    let mut status = ntkrnl_sys::base::STATUS_SUCCESS;

    PAGED_CODE();

    if (*irp_stack).MajorFunction == ntkrnl_sys::constants::IRP_MJ_CREATE {
        let security_context = (*irp_stack).Parameters.Create.SecurityContext;
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
            (*security_context).DesiredAccess,
        );
        TracePrint(
            TRACE_LEVEL_INFORMATION,
            TRACE_FLAG_INIT,
            b"ClasspCREATEClose: file object %p\n\0".as_ptr() as *const i8,
            (*irp_stack).FileObject,
        );

        NT_ASSERT((BreakOnClose == 0) as i32);

        if !(*irp_stack).FileObject.is_null() {
            let mut fs_context: *mut FILE_OBJECT_EXTENSION;
            status = AllocateDictionaryEntry(
                &mut (*common_extension).FileObjectDictionary as *mut DICTIONARY,
                (*irp_stack).FileObject as ULONGLONG,
                core::mem::size_of::<FILE_OBJECT_EXTENSION>() as ULONG,
                CLASS_TAG_FILE_OBJECT_EXTENSION,
                &mut fs_context as *mut *mut FILE_OBJECT_EXTENSION as *mut *mut c_void,
            );

            if ntkrnl_sys::base::NT_SUCCESS(status) {
                ntkrnl_sys::rtl::RtlZeroMemory(
                    fs_context as *mut c_void,
                    core::mem::size_of::<FILE_OBJECT_EXTENSION>(),
                );
                (*fs_context).FileObject = (*irp_stack).FileObject;
                (*fs_context).DeviceObject = DeviceObject;
            } else if status == ntkrnl_sys::constants::STATUS_OBJECT_NAME_COLLISION {
                status = ntkrnl_sys::base::STATUS_SUCCESS;
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
            file_object,
        );

        if !(*irp_stack).FileObject.is_null() {
            let fs_context = ClassGetFsContext(common_extension, (*irp_stack).FileObject);

            TracePrint(
                TRACE_LEVEL_INFORMATION,
                TRACE_FLAG_INIT,
                b"ClasspCreateCLOSE: file extension %p\n\0".as_ptr() as *const i8,
                fs_context,
            );

            if !fs_context.is_null() {
                TracePrint(
                    TRACE_LEVEL_INFORMATION,
                    TRACE_FLAG_INIT,
                    b"ClasspCreateCLOSE: extension is ours - freeing\n\0".as_ptr() as *const i8,
                );
                NT_ASSERT((BreakOnClose == 0) as i32);
                ClasspCleanupProtectedLocks(fs_context);
                ClasspCleanupDisableMcn(fs_context);
                FreeDictionaryEntry(
                    &mut (*common_extension).FileObjectDictionary as *mut DICTIONARY,
                    fs_context as *mut c_void,
                );
            }
        }
    }

    TracePrint(
        TRACE_LEVEL_INFORMATION,
        TRACE_FLAG_INIT,
        b"ClasspCreateClose: %s for devobj %p\n\0".as_ptr() as *const i8,
        if ntkrnl_sys::base::NT_SUCCESS(status) {
            b"Success\0".as_ptr() as *const i8
        } else {
            b"FAILED\0".as_ptr() as *const i8
        },
        DeviceObject,
    );

    if ntkrnl_sys::base::NT_SUCCESS(status) {
        let mut event: ntkrnl_sys::ddk::KEVENT = core::mem::zeroed();
        ntkrnl_sys::ddk::KeInitializeEvent(
            &mut event as *mut ntkrnl_sys::ddk::KEVENT,
            ntkrnl_sys::ddk::_EVENT_TYPE_SynchronizationEvent,
            0,
        );
        ntkrnl_sys::ddk::IoCopyCurrentIrpStackLocationToNext(Irp);
        ntkrnl_sys::ddk::IoSetCompletionRoutine(
            Irp,
            Some(ClassSignalCompletion),
            &mut event as *mut ntkrnl_sys::ddk::KEVENT as *mut c_void,
            1,
            1,
            1,
        );
        status = ntkrnl_sys::driver_ddk::IoCallDriver((*common_extension).LowerDeviceObject, Irp);

        if status == ntkrnl_sys::constants::STATUS_PENDING {
            ntkrnl_sys::ddk::KeWaitForSingleObject(
                &mut event as *mut ntkrnl_sys::ddk::KEVENT,
                ntkrnl_sys::ddk::_KWAIT_REASON_Executive,
                (ntkrnl_sys::ddk::_KWAIT_REASON_Executive as i32).try_into().unwrap(),
                0,
                core::ptr::null(),
            );
            status = (*Irp).IoStatus.__bindgen_anon_1.Status;
        }

        if !ntkrnl_sys::base::NT_SUCCESS(status) {
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
pub unsafe extern "C" fn ClasspCleanupProtectedLocks(
    FsContext: *mut FILE_OBJECT_EXTENSION,
) {
    // PCOMMON_DEVICE_EXTENSION commonExtension = FsContext->DeviceObject->DeviceExtension;
    let commonExtension = (*(*FsContext).DeviceObject).DeviceExtension;
    // PFUNCTIONAL_DEVICE_EXTENSION fdoExtension = commonExtension->PartitionZeroExtension;
    let fdoExtension = (*commonExtension).PartitionZeroExtension;

    // ULONG newDeviceLockCount = 1;
    let mut newDeviceLockCount: u32 = 1;

    PAGED_CODE();

    // TracePrint calls with format strings
    TracePrint(
        TRACE_LEVEL_INFORMATION,
        TRACE_FLAG_INIT,
        b"ClasspCleanupProtectedLocks called for %p\n\0".as_ptr() as *const i8,
        (*FsContext).DeviceObject,
    );
    TracePrint(
        TRACE_LEVEL_INFORMATION,
        TRACE_FLAG_INIT,
        b"ClasspCleanupProtectedLocks - FsContext %p is locked %d times\n\0".as_ptr() as *const i8,
        FsContext,
        (*FsContext).LockCount,
    );

    NT_ASSERT((BreakOnClose == 0) as i32);

    // Synchronize with ejection and ejection control requests.
    ntkrnl_sys::ddk::KeEnterCriticalRegion();
    ntkrnl_sys::ddk::KeWaitForSingleObject(
        core::ptr::addr_of_mut!((*fdoExtension).EjectSynchronizationEvent),
        _KWAIT_REASON_UserRequest, // UserRequest is available from ntkrnl_sys::ddk
        0i8,  // KernelMode is available from ntkrnl_sys::constants
        0,           // FALSE
        core::ptr::null_mut(),
    );

    // For each secure lock on this handle decrement ...
    if (*FsContext).LockCount != 0 {
        loop {
            // InterlockedDecrement on LockCount (volatile cast implied)
            let _ = ntkrnl_sys::interlocked::InterlockedDecrement(
                core::ptr::addr_of_mut!((*FsContext).LockCount) as *mut i32,
            );
            // Decrement ProtectedLockCount and store new value
            newDeviceLockCount = ntkrnl_sys::interlocked::InterlockedDecrement(
                core::ptr::addr_of_mut!((*fdoExtension).ProtectedLockCount) as *mut i32,
            );

            // Condition check after decrement
            if (*FsContext).LockCount <= 0 {
                break;
            }
        }

        // Trace the counts
        TracePrint(
            TRACE_LEVEL_INFORMATION,
            TRACE_FLAG_INIT,
            b"ClasspCleanupProtectedLocks: FDO secured lock count = %d lock count = %d\n\0"
                .as_ptr() as *const i8,
            (*fdoExtension).ProtectedLockCount,
            (*fdoExtension).LockCount,
        );

        // If both counts are zero, issue unlock command
        if newDeviceLockCount == 0 && (*fdoExtension).LockCount == 0 {
            // SCSI_REQUEST_BLOCK srb = {0};
            let mut srb: SCSI_REQUEST_BLOCK = core::mem::zeroed();
            // UCHAR srbExBuffer[CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE] = {0};
            let mut srbExBuffer: [u8; CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE] =
                [0u8; CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE];
            // PSTORAGE_REQUEST_BLOCK srbEx = (PSTORAGE_REQUEST_BLOCK)srbExBuffer;
            let srbEx: *mut STORAGE_REQUEST_BLOCK =
                srbExBuffer.as_mut_ptr() as *mut STORAGE_REQUEST_BLOCK;
            // PCDB cdb = NULL;
            let cdb: *mut CDB;
            // NTSTATUS status;
            let status: i32;
            // PSCSI_REQUEST_BLOCK srbPtr;
            let srbPtr: *mut SCSI_REQUEST_BLOCK;

            TracePrint(
                TRACE_LEVEL_INFORMATION,
                TRACE_FLAG_INIT,
                b"ClasspCleanupProtectedLocks: FDO lock count dropped to zero\n\0".as_ptr()
                    as *const i8,
            );

            // Determine SRB type and set up
            if (*(*fdoExtension).AdapterDescriptor).SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK {
                // Suppress prefast warning – ignore
                status = InitializeStorageRequestBlock(
                    srbEx,
                    STORAGE_ADDRESS_TYPE_BTL8,
                    core::mem::size_of::<[u8; CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE]>() as u32,
                    1,
                    SrbExDataTypeScsiCdb16,
                );
                if ntkrnl_sys::base::NT_SUCCESS(status) {
                    (*srbEx).TimeOutValue = (*fdoExtension).TimeOutValue;
                    SrbSetCdbLength(srbEx, 6);
                    cdb = SrbGetCdb(srbEx);
                    srbPtr = srbEx as *mut SCSI_REQUEST_BLOCK;
                } else {
                    // Should not happen – assert and fallback
                    NT_ASSERT(0); // FALSE = 0
                    srb.TimeOutValue = (*fdoExtension).TimeOutValue;
                    srb.CdbLength = 6;
                    cdb = core::ptr::addr_of_mut!(srb.Cdb) as *mut CDB;
                    srbPtr = core::ptr::addr_of_mut!(srb);
                }
            } else {
                // Legacy SRB path
                srb.TimeOutValue = (*fdoExtension).TimeOutValue;
                srb.CdbLength = 6;
                cdb = core::ptr::addr_of_mut!(srb.Cdb) as *mut CDB;
                srbPtr = core::ptr::addr_of_mut!(srb);
            }

            // Set up SCSI medium removal command
            (*cdb).MEDIA_REMOVAL.OperationCode = SCSIOP_MEDIUM_REMOVAL;
            // FALSE – allow media removal
            (*cdb).MEDIA_REMOVAL.Prevent = 0;

            // Send SRB synchronously
            status = ClassSendSrbSynchronous(
                (*fdoExtension).DeviceObject,
                srbPtr,
                core::ptr::null_mut(),
                0,
                0, // FALSE
            );

            TracePrint(
                TRACE_LEVEL_INFORMATION,
                TRACE_FLAG_INIT,
                b"ClasspCleanupProtectedLocks: unlock request to drive returned status %lx\n\0"
                    .as_ptr() as *const i8,
                status,
            );
        }
    }

    // Signal synchronization event and leave critical region
    ntkrnl_sys::ke::KeSetEvent(
        core::ptr::addr_of_mut!((*fdoExtension).EjectSynchronizationEvent),
        IO_NO_INCREMENT.into(), // from imports: IO_NO_INCREMENT
        0,               // FALSE
    );
    ntkrnl_sys::ddk::KeLeaveCriticalRegion();
    return;
}
#[no_mangle]
pub unsafe extern "C" fn ClasspEjectionControl(
    Fdo: PDEVICE_OBJECT,
    Irp: PIRP,
    LockType: MEDIA_LOCK_TYPE,
    Lock: BOOLEAN,
) -> NTSTATUS {

    let FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION = (*Fdo).DeviceExtension;
    let commonExtension: PCOMMON_DEVICE_EXTENSION = FdoExtension as PCOMMON_DEVICE_EXTENSION;

    let mut fsContext: PFILE_OBJECT_EXTENSION = core::ptr::null_mut();
    let mut srb: *mut SCSI_REQUEST_BLOCK = core::ptr::null_mut();
    let mut countChanged: BOOLEAN = 0u8;
    let mut status: NTSTATUS;

    PAGED_CODE();

    KeEnterCriticalRegion();

    status = KeWaitForSingleObject(
        &mut (*FdoExtension).EjectSynchronizationEvent,
        _KWAIT_REASON_UserRequest,
        0i8,
        0u8, // FALSE
        core::ptr::null_mut(),
    );
    NT_ASSERT(status == STATUS_SUCCESS);

    TracePrint(
        TRACE_LEVEL_INFORMATION,
        TRACE_FLAG_IOCTL,
        b"ClasspEjectionControl: Received request for %s lock type\n\0".as_ptr() as *const i8,
        LockTypeStrings[LockType as usize],
    );

    // SEH2_TRY
    'seh2_try: {
        let mut cdb: *mut CDB = core::ptr::null_mut();

        if LockType == SecureMediaLock {
            let irpStack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
            let fileObject: PFILE_OBJECT = (*irpStack).FileObject;

            if !fileObject.is_null() {
                fsContext = ClassGetFsContext(commonExtension, fileObject);
            }
            if fsContext.is_null() {
                status = STATUS_INVALID_PARAMETER;
                break 'seh2_try;
            }
        }

        if Lock != 0 {
            match LockType {
                SimpleMediaLock => {
                    (*FdoExtension).LockCount += 1;
                    countChanged = 1;
                }
                SecureMediaLock => {
                    (*fsContext).LockCount += 1;
                    (*FdoExtension).ProtectedLockCount += 1;
                    countChanged = 1;
                }
                InternalMediaLock => {
                    (*FdoExtension).InternalLockCount += 1;
                    countChanged = 1;
                }
                _ => {}
            }
        } else {
            match LockType {
                SimpleMediaLock => {
                    if (*FdoExtension).LockCount != 0 {
                        (*FdoExtension).LockCount -= 1;
                        countChanged = 1;
                    }
                }
                SecureMediaLock => {
                    if (*fsContext).LockCount == 0 {
                        status = STATUS_INVALID_DEVICE_STATE;
                        break 'seh2_try;
                    }
                    (*fsContext).LockCount -= 1;
                    (*FdoExtension).ProtectedLockCount -= 1;
                    countChanged = 1;
                }
                InternalMediaLock => {
                    NT_ASSERT((*FdoExtension).InternalLockCount != 0);
                    (*FdoExtension).InternalLockCount -= 1;
                    countChanged = 1;
                }
                _ => {}
            }

            if ((*FdoExtension).ProtectedLockCount != 0)
                || ((*FdoExtension).InternalLockCount != 0)
                || ((*FdoExtension).LockCount != 0)
            {
                status = STATUS_SUCCESS;
                break 'seh2_try;
            }
        }

        status = STATUS_SUCCESS;
        if ((*Fdo).Characteristics & FILE_REMOVABLE_MEDIA) != 0 {
            srb = ClasspAllocateSrb(FdoExtension) as *mut SCSI_REQUEST_BLOCK;
            if srb.is_null() {
                status = STATUS_INSUFFICIENT_RESOURCES;
                break 'seh2_try;
            }

            if (*FdoExtension).AdapterDescriptor.SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK {
                status = InitializeStorageRequestBlock(
                    srb as *mut STORAGE_REQUEST_BLOCK,
                    STORAGE_ADDRESS_TYPE_BTL8,
                    CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE.try_into().unwrap(),
                    1,
                    SrbExDataTypeScsiCdb16,
                );
                if !NT_SUCCESS(status) {
                    NT_ASSERT(false);
                    break 'seh2_try;
                }
            } else {
                RtlZeroMemory(srb, core::mem::size_of::<SCSI_REQUEST_BLOCK>());
            }

            SrbSetCdbLength(srb, 6);
            cdb = SrbGetCdb(srb);
            NT_ASSERT(!cdb.is_null());

            (*cdb).MEDIA_REMOVAL.OperationCode = SCSIOP_MEDIUM_REMOVAL as u8;
            (*cdb).MEDIA_REMOVAL.Prevent = Lock;

            SrbSetTimeOutValue(srb, (*FdoExtension).TimeOutValue);

            status = ClassSendSrbSynchronous(
                (*FdoExtension).DeviceObject,
                srb,
                core::ptr::null_mut(),
                0,
                0u8,
            );
        }
    }

    // SEH2_FINALLY block
    {
        if !NT_SUCCESS(status) {
            TracePrint(
                TRACE_LEVEL_INFORMATION,
                TRACE_FLAG_IOCTL,
                b"ClasspEjectionControl: FAILED status %x -- reverting lock counts\n\0".as_ptr()
                    as *const i8,
                status,
            );
            if countChanged != 0 {
                if Lock != 0 {
                    match LockType {
                        SimpleMediaLock => {
                            (*FdoExtension).LockCount -= 1;
                        }
                        SecureMediaLock => {
                            (*fsContext).LockCount -= 1;
                            (*FdoExtension).ProtectedLockCount -= 1;
                        }
                        InternalMediaLock => {
                            (*FdoExtension).InternalLockCount -= 1;
                        }
                        _ => {}
                    }
                } else {
                    match LockType {
                        SimpleMediaLock => {
                            (*FdoExtension).LockCount += 1;
                        }
                        SecureMediaLock => {
                            (*fsContext).LockCount += 1;
                            (*FdoExtension).ProtectedLockCount += 1;
                        }
                        InternalMediaLock => {
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
                b"ClasspEjectionControl: Succeeded\n\0".as_ptr() as *const i8,
            );
        }

        TracePrint(
            TRACE_LEVEL_INFORMATION,
            TRACE_FLAG_IOCTL,
            b"ClasspEjectionControl: Current Counts: Internal: %x  Secure: %x  Simple: %x\n\0"
                .as_ptr() as *const i8,
            (*FdoExtension).InternalLockCount,
            (*FdoExtension).ProtectedLockCount,
            (*FdoExtension).LockCount,
        );

        KeSetEvent(
            &mut (*FdoExtension).EjectSynchronizationEvent,
            IO_NO_INCREMENT.into(),
            0u8,
        );
        KeLeaveCriticalRegion();

        if !srb.is_null() {
            ClassFreeOrReuseSrb(FdoExtension, srb);
        }
    }

    status
}

unsafe extern "system" {
    fn PAGED_CODE();
    fn GetDictionaryEntry(
        dict: *mut core::ffi::c_void,
        key: u64,
    ) -> *mut FILE_OBJECT_EXTENSION;
}
#[no_mangle]
pub unsafe extern "system" fn ClassGetFsContext(
    CommonExtension: *mut COMMON_DEVICE_EXTENSION,
    FileObject: *mut FILE_OBJECT,
) -> *mut FILE_OBJECT_EXTENSION {
    PAGED_CODE();
    return GetDictionaryEntry(
        &raw mut (*CommonExtension).FileObjectDictionary as *mut core::ffi::c_void,
        FileObject as u64,
    );
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*++
//
// Copyright (C) Microsoft Corporation, 1991 - 2010
//
// Module Name:
//
//     class.c
//
// Abstract:
//
//     SCSI class driver routines
//
// Environment:
//
//     kernel mode only
//
// Notes:
//
//
// Revision History:
//
// --*/
//
// #define CLASS_INIT_GUID 0
// #include "classp.h"
// #include "debug.h"
//
// #ifdef DEBUG_USE_WPP
// #include "create.tmh"
// #endif
//
// ULONG BreakOnClose = 0;
//
// const PCSZ LockTypeStrings[] = {
//     "Simple",
//     "Secure",
//     "Internal"
// };
//
//
// VOID
// ClasspCleanupDisableMcn(
//     IN PFILE_OBJECT_EXTENSION FsContext
//     );
//
// #ifdef ALLOC_PRAGMA
// #pragma alloc_text(PAGE, ClassCreateClose)
// #pragma alloc_text(PAGE, ClasspCreateClose)
// #pragma alloc_text(PAGE, ClasspCleanupProtectedLocks)
// #pragma alloc_text(PAGE, ClasspEjectionControl)
// #pragma alloc_text(PAGE, ClasspCleanupDisableMcn)
// #pragma alloc_text(PAGE, ClassGetFsContext)
// #endif
//
// NTSTATUS
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// ClassCreateClose(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     SCSI class driver create and close routine.  This is called by the I/O system
//     when the device is opened or closed.
//
// Arguments:
//
//     DriverObject - Pointer to driver object created by system.
//
//     Irp - IRP involved.
//
// Return Value:
//
//     Device-specific drivers return value or STATUS_SUCCESS.
//
// --*/
//
// {
//     PCOMMON_DEVICE_EXTENSION commonExtension = DeviceObject->DeviceExtension;
//     ULONG removeState;
//     NTSTATUS status;
//
//     PAGED_CODE();
//
//     //
//     // If we're getting a close request then we know the device object hasn't
//     // been completely destroyed.  Let the driver cleanup if necessary.
//     //
//
//     removeState = ClassAcquireRemoveLock(DeviceObject, Irp);
//
//     //
//     // Invoke the device-specific routine, if one exists. Otherwise complete
//     // with SUCCESS
//     //
//
//     if((removeState == NO_REMOVE) ||
//        IS_CLEANUP_REQUEST(IoGetCurrentIrpStackLocation(Irp)->MajorFunction)) {
//
//         status = ClasspCreateClose(DeviceObject, Irp);
//
//         if((NT_SUCCESS(status)) &&
//            (commonExtension->DevInfo->ClassCreateClose)) {
//
//             return commonExtension->DevInfo->ClassCreateClose(DeviceObject, Irp);
//         }
//
//     } else {
//         status = STATUS_DEVICE_DOES_NOT_EXIST;
//     }
//
//     Irp->IoStatus.Status = status;
//     ClassReleaseRemoveLock(DeviceObject, Irp);
//     ClassCompleteRequest(DeviceObject, Irp, IO_NO_INCREMENT);
//     return status;
// }
//
//
// NTSTATUS
// ClasspCreateClose(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PIRP Irp
//     )
// /*++
//
// Routine Description:
//
//     This routine will handle create/close operations for a given classpnp
//     device if the class driver doesn't supply it's own handler.  If there
//     is a file object supplied for our driver (if it's a FO_DIRECT_DEVICE_OPEN
//     file object) then it will initialize a file extension on create or destroy
//     the extension on a close.
//
// Arguments:
//
//     DeviceObject - the device object being opened or closed.
//
//     Irp - the create/close irp
//
// Return Value:
//
//     status
//
// --*/
// {
//     PCOMMON_DEVICE_EXTENSION commonExtension = DeviceObject->DeviceExtension;
//     PIO_STACK_LOCATION irpStack = IoGetCurrentIrpStackLocation(Irp);
//
//     PFILE_OBJECT fileObject = irpStack->FileObject;
//
//     NTSTATUS status = STATUS_SUCCESS;
//
//     PAGED_CODE();
//
//
//     //
//     // ISSUE-2000/3/28-henrygab - if lower stack fails create/close, we end up
//     // in an inconsistent state.  re-write to verify all args and allocate all
//     // required resources, then pass the irp down, then complete the
//     // transaction.  this is because we also cannot forward the irp, then fail
//     // it after it has succeeded a lower-level driver.
//     //
//
//     if(irpStack->MajorFunction == IRP_MJ_CREATE) {
//
//         PIO_SECURITY_CONTEXT securityContext =
//             irpStack->Parameters.Create.SecurityContext;
//         TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_INIT,
//                     "ClasspCREATEClose: create received for device %p\n",
//                     DeviceObject));
//         TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_INIT,
//                     "ClasspCREATEClose: desired access %lx\n",
//                     securityContext->DesiredAccess));
//         TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_INIT,
//                     "ClasspCREATEClose: file object %p\n",
//                     irpStack->FileObject));
//
//         NT_ASSERT(BreakOnClose == FALSE);
//
//         if(irpStack->FileObject != NULL) {
//
//             PFILE_OBJECT_EXTENSION fsContext;
//
//             //
//             // Allocate our own file object extension for this device object.
//             //
//
//             status = AllocateDictionaryEntry(
//                         &commonExtension->FileObjectDictionary,
//                         (ULONGLONG) irpStack->FileObject,
//                         sizeof(FILE_OBJECT_EXTENSION),
//                         CLASS_TAG_FILE_OBJECT_EXTENSION,
//                         (PVOID *)&fsContext);
//
//             if(NT_SUCCESS(status)) {
//
//                 RtlZeroMemory(fsContext,
//                               sizeof(FILE_OBJECT_EXTENSION));
//
//                 fsContext->FileObject = irpStack->FileObject;
//                 fsContext->DeviceObject = DeviceObject;
//             } else if (status == STATUS_OBJECT_NAME_COLLISION) {
//                 status = STATUS_SUCCESS;
//             }
//         }
//
//     } else {
//
//         TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_INIT,
//                     "ClasspCreateCLOSE: close received for device %p\n",
//                     DeviceObject));
//         TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_INIT,
//                     "ClasspCreateCLOSE: file object %p\n",
//                     fileObject));
//
//         if(irpStack->FileObject != NULL) {
//
//             PFILE_OBJECT_EXTENSION fsContext =
//                 ClassGetFsContext(commonExtension, irpStack->FileObject);
//
//             TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_INIT,
//                         "ClasspCreateCLOSE: file extension %p\n",
//                         fsContext));
//
//             if(fsContext != NULL) {
//
//                 TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_INIT,
//                             "ClasspCreateCLOSE: extension is ours - "
//                             "freeing\n"));
//                 NT_ASSERT(BreakOnClose == FALSE);
//
//                 ClasspCleanupProtectedLocks(fsContext);
//
//                 ClasspCleanupDisableMcn(fsContext);
//
//                 FreeDictionaryEntry(&(commonExtension->FileObjectDictionary),
//                                     fsContext);
//             }
//         }
//     }
//
//     //
//     // Notify the lower levels about the create or close operation - give them
//     // a chance to cleanup too.
//     //
//
//     TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_INIT,
//                 "ClasspCreateClose: %s for devobj %p\n",
//                 (NT_SUCCESS(status) ? "Success" : "FAILED"),
//                 DeviceObject));
//
//
//     if(NT_SUCCESS(status)) {
//
//         KEVENT event;
//
//         //
//         // Set up the event to wait on
//         //
//
//         KeInitializeEvent(&event, SynchronizationEvent, FALSE);
//
//         IoCopyCurrentIrpStackLocationToNext(Irp);
//         IoSetCompletionRoutine( Irp, ClassSignalCompletion, &event,
//                                 TRUE, TRUE, TRUE);
//
//         status = IoCallDriver(commonExtension->LowerDeviceObject, Irp);
//
//         if(status == STATUS_PENDING) {
//             (VOID)KeWaitForSingleObject(&event,
//                                         Executive,
//                                         KernelMode,
//                                         FALSE,
//                                         NULL);
//             status = Irp->IoStatus.Status;
//         }
//
//         if (!NT_SUCCESS(status)) {
//             TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_INIT,
//                         "ClasspCreateClose: Lower driver failed, but we "
//                         "succeeded.  This is a problem, lock counts will be "
//                         "out of sync between levels.\n"));
//         }
//
//     }
//
//
//     return status;
// }
//
//
// VOID
// ClasspCleanupProtectedLocks(
//     IN PFILE_OBJECT_EXTENSION FsContext
//     )
// {
//     PCOMMON_DEVICE_EXTENSION commonExtension =
//         FsContext->DeviceObject->DeviceExtension;
//
//     PFUNCTIONAL_DEVICE_EXTENSION fdoExtension =
//         commonExtension->PartitionZeroExtension;
//
//     ULONG newDeviceLockCount = 1;
//
//     PAGED_CODE();
//
//     TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_INIT,
//                 "ClasspCleanupProtectedLocks called for %p\n",
//                 FsContext->DeviceObject));
//     TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_INIT,
//                 "ClasspCleanupProtectedLocks - FsContext %p is locked "
//                 "%d times\n", FsContext, FsContext->LockCount));
//
//     NT_ASSERT(BreakOnClose == FALSE);
//
//     //
//     // Synchronize with ejection and ejection control requests.
//     //
//
//     KeEnterCriticalRegion();
//     (VOID)KeWaitForSingleObject(&(fdoExtension->EjectSynchronizationEvent),
//                                 UserRequest,
//                                 KernelMode,
//                                 FALSE,
//                           NULL);
//
//     //
//     // For each secure lock on this handle decrement the secured lock count
//     // for the FDO.  Keep track of the new value.
//     //
//
//     if (FsContext->LockCount != 0) {
//
//         do {
//
//             InterlockedDecrement((volatile LONG *)&FsContext->LockCount);
//
//             newDeviceLockCount =
//                 InterlockedDecrement(&fdoExtension->ProtectedLockCount);
//
//         } while (FsContext->LockCount > 0);
//
//         //
//         // If the new lock count has been dropped to zero then issue a lock
//         // command to the device.
//         //
//
//         TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_INIT,
//                     "ClasspCleanupProtectedLocks: FDO secured lock count = %d "
//                     "lock count = %d\n",
//                     fdoExtension->ProtectedLockCount,
//                     fdoExtension->LockCount));
//
//         if ((newDeviceLockCount == 0) && (fdoExtension->LockCount == 0)) {
//
//             SCSI_REQUEST_BLOCK srb = {0};
//             UCHAR srbExBuffer[CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE] = {0};
//             PSTORAGE_REQUEST_BLOCK srbEx = (PSTORAGE_REQUEST_BLOCK)srbExBuffer;
//             PCDB cdb = NULL;
//             NTSTATUS status;
//             PSCSI_REQUEST_BLOCK srbPtr;
//
//             TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_INIT,
//                         "ClasspCleanupProtectedLocks: FDO lock count dropped "
//                         "to zero\n"));
//
//             if (fdoExtension->AdapterDescriptor->SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK) {
// #ifdef _MSC_VER
//                 #pragma prefast(suppress:26015, "InitializeStorageRequestBlock ensures buffer access is bounded")
// #endif
//                 status = InitializeStorageRequestBlock(srbEx,
//                                                        STORAGE_ADDRESS_TYPE_BTL8,
//                                                        sizeof(srbExBuffer),
//                                                        1,
//                                                        SrbExDataTypeScsiCdb16);
//                 if (NT_SUCCESS(status)) {
//                     srbEx->TimeOutValue = fdoExtension->TimeOutValue;
//                     SrbSetCdbLength(srbEx, 6);
//                     cdb = SrbGetCdb(srbEx);
//                     srbPtr = (PSCSI_REQUEST_BLOCK)srbEx;
//                 } else {
//                     //
//                     // Should not happen. Revert to legacy SRB.
//                     //
//                     NT_ASSERT(FALSE);
//                     srb.TimeOutValue = fdoExtension->TimeOutValue;
//                     srb.CdbLength = 6;
//                     cdb = (PCDB) &(srb.Cdb);
//                     srbPtr = &srb;
//                 }
//
//             } else {
//
//                 srb.TimeOutValue = fdoExtension->TimeOutValue;
//                 srb.CdbLength = 6;
//                 cdb = (PCDB) &(srb.Cdb);
//                 srbPtr = &srb;
//
//             }
//
//             cdb->MEDIA_REMOVAL.OperationCode = SCSIOP_MEDIUM_REMOVAL;
//
//             //
//             // TRUE - prevent media removal.
//             // FALSE - allow media removal.
//             //
//
//             cdb->MEDIA_REMOVAL.Prevent = FALSE;
//
//             status = ClassSendSrbSynchronous(fdoExtension->DeviceObject,
//                                              srbPtr,
//                                              NULL,
//                                              0,
//                                              FALSE);
//
//             TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_INIT,
//                         "ClasspCleanupProtectedLocks: unlock request to drive "
//                         "returned status %lx\n", status));
//         }
//     }
//
//     KeSetEvent(&fdoExtension->EjectSynchronizationEvent,
//                IO_NO_INCREMENT,
//                FALSE);
//     KeLeaveCriticalRegion();
//     return;
// }
//
//
// VOID
// ClasspCleanupDisableMcn(
//     IN PFILE_OBJECT_EXTENSION FsContext
//     )
// {
//     PCOMMON_DEVICE_EXTENSION commonExtension =
//         FsContext->DeviceObject->DeviceExtension;
//
//     PFUNCTIONAL_DEVICE_EXTENSION fdoExtension =
//         commonExtension->PartitionZeroExtension;
//
//     PAGED_CODE();
//
//     TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_MCN,
//                 "ClasspCleanupDisableMcn called for %p\n",
//                 FsContext->DeviceObject));
//     TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_MCN,
//                 "ClasspCleanupDisableMcn - FsContext %p is disabled "
//                 "%d times\n", FsContext, FsContext->McnDisableCount));
//
//     //
//     // For each secure lock on this handle decrement the secured lock count
//     // for the FDO.  Keep track of the new value.
//     //
//
//     while(FsContext->McnDisableCount != 0) {
//         FsContext->McnDisableCount--;
//         ClassEnableMediaChangeDetection(fdoExtension);
//     }
//
//     return;
// }
//
//
// /*
//  *  ISSUE: REMOVE this old function implementation as soon as the
//  *                  boottime pagefile problems with the new one (below)
//  *                  are resolved.
//  */
// NTSTATUS
// ClasspEjectionControl(
//     IN PDEVICE_OBJECT Fdo,
//     IN PIRP Irp,
//     IN MEDIA_LOCK_TYPE LockType,
//     IN BOOLEAN Lock
//     )
// {
//     PFUNCTIONAL_DEVICE_EXTENSION FdoExtension = Fdo->DeviceExtension;
//     PCOMMON_DEVICE_EXTENSION commonExtension =
//         (PCOMMON_DEVICE_EXTENSION) FdoExtension;
//
//     PFILE_OBJECT_EXTENSION fsContext = NULL;
//     NTSTATUS status;
//     PSCSI_REQUEST_BLOCK srb = NULL;
//     BOOLEAN countChanged = FALSE;
//
//     PAGED_CODE();
//
//     /*
//      *  Ensure that the user thread is not suspended while we are holding EjectSynchronizationEvent.
//      */
//     KeEnterCriticalRegion();
//
//     status = KeWaitForSingleObject(
//                 &(FdoExtension->EjectSynchronizationEvent),
//                 UserRequest,
//                 KernelMode,
//                 FALSE,
//                 NULL);
//
//     NT_ASSERT(status == STATUS_SUCCESS);
//
//     TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL,
//                 "ClasspEjectionControl: "
//                 "Received request for %s lock type\n",
//                 LockTypeStrings[LockType]
//                 ));
//
//     _SEH2_TRY {
//         PCDB cdb = NULL;
//
//         //
//         // Determine if this is a "secured" request.
//         //
//
//         if (LockType == SecureMediaLock) {
//
//             PIO_STACK_LOCATION irpStack = IoGetCurrentIrpStackLocation(Irp);
//             PFILE_OBJECT fileObject = irpStack->FileObject;
//
//             //
//             // Make sure that the file object we are supplied has a
//             // proper FsContext before we try doing a secured lock.
//             //
//
//             if (fileObject != NULL) {
//                 fsContext = ClassGetFsContext(commonExtension, fileObject);
//             }
//
//             if (fsContext == NULL) {
//
//                 //
//                 // This handle isn't setup correctly.  We can't let the
//                 // operation go.
//                 //
//
//                 status = STATUS_INVALID_PARAMETER;
//                 _SEH2_LEAVE;
//             }
//         }
//
//         if (Lock) {
//
//             //
//             // This is a lock command.  Reissue the command in case bus or
//             // device was reset and the lock was cleared.
//             // note: may need to decrement count if actual lock operation
//             //       failed....
//             //
//
//             switch (LockType) {
//
//                 case SimpleMediaLock: {
//                     FdoExtension->LockCount++;
//                     countChanged = TRUE;
//                     break;
//                 }
//
//                 case SecureMediaLock: {
//                     fsContext->LockCount++;
//                     FdoExtension->ProtectedLockCount++;
//                     countChanged = TRUE;
//                     break;
//                 }
//
//                 case InternalMediaLock: {
//                     FdoExtension->InternalLockCount++;
//                     countChanged = TRUE;
//                     break;
//                 }
//             }
//
//         } else {
//
//             //
//             // This is an unlock command.  If it's a secured one then make sure
//             // the caller has a lock outstanding or return an error.
//             // note: may need to re-increment the count if actual unlock
//             //       operation fails....
//             //
//
//             switch (LockType) {
//
//                 case SimpleMediaLock: {
//                     if(FdoExtension->LockCount != 0) {
//                         FdoExtension->LockCount--;
//                         countChanged = TRUE;
//                     }
//                     break;
//                 }
//
//                 case SecureMediaLock: {
//                     if(fsContext->LockCount == 0) {
//                         status = STATUS_INVALID_DEVICE_STATE;
//                         _SEH2_LEAVE;
//                     }
//                     fsContext->LockCount--;
//                     FdoExtension->ProtectedLockCount--;
//                     countChanged = TRUE;
//                     break;
//                 }
//
//                 case InternalMediaLock: {
//                     NT_ASSERT(FdoExtension->InternalLockCount != 0);
//                     FdoExtension->InternalLockCount--;
//                     countChanged = TRUE;
//                     break;
//                 }
//             }
//
//             //
//             // We only send an unlock command to the drive if both the
//             // secured and unsecured lock counts have dropped to zero.
//             //
//
//             if ((FdoExtension->ProtectedLockCount != 0) ||
//                 (FdoExtension->InternalLockCount != 0) ||
//                 (FdoExtension->LockCount != 0)) {
//
//                 status = STATUS_SUCCESS;
//                 _SEH2_LEAVE;
//             }
//         }
//
//         status = STATUS_SUCCESS;
//         if (TEST_FLAG(Fdo->Characteristics, FILE_REMOVABLE_MEDIA)) {
//
//             srb = (PSCSI_REQUEST_BLOCK)ClasspAllocateSrb(FdoExtension);
//
//             if (srb == NULL) {
//                 status = STATUS_INSUFFICIENT_RESOURCES;
//                 _SEH2_LEAVE;
//             }
//
//             if (FdoExtension->AdapterDescriptor->SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK) {
//
//                 //
//                 // NOTE - this is based on size used in ClasspAllocateSrb
//                 //
//
//                 status = InitializeStorageRequestBlock((PSTORAGE_REQUEST_BLOCK)srb,
//                                                        STORAGE_ADDRESS_TYPE_BTL8,
//                                                        CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE,
//                                                        1,
//                                                        SrbExDataTypeScsiCdb16);
//                 if (!NT_SUCCESS(status)) {
//                     NT_ASSERT(FALSE);
//                     _SEH2_LEAVE;
//                 }
//
//             } else {
//                 RtlZeroMemory(srb, sizeof(SCSI_REQUEST_BLOCK));
//             }
//
//             SrbSetCdbLength(srb, 6);
//             cdb = SrbGetCdb(srb);
//             NT_ASSERT(cdb != NULL);
//
//             cdb->MEDIA_REMOVAL.OperationCode = SCSIOP_MEDIUM_REMOVAL;
//
//             //
//             // TRUE - prevent media removal.
//             // FALSE - allow media removal.
//             //
//
//             cdb->MEDIA_REMOVAL.Prevent = Lock;
//
//             //
//             // Set timeout value.
//             //
//
//             SrbSetTimeOutValue(srb, FdoExtension->TimeOutValue);
//
//             //
//             // The actual lock operation on the device isn't so important
//             // as the internal lock counts.  Ignore failures.
//             //
//
//             status = ClassSendSrbSynchronous(FdoExtension->DeviceObject,
//                                              srb,
//                                              NULL,
//                                              0,
//                                              FALSE);
//         }
//
//     } _SEH2_FINALLY {
//
//         if (!NT_SUCCESS(status)) {
//             TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL,
//                         "ClasspEjectionControl: FAILED status %x -- "
//                         "reverting lock counts\n", status));
//
//             if (countChanged) {
//
//                 //
//                 // have to revert to previous counts if the
//                 // lock/unlock operation actually failed.
//                 //
//
//                 if (Lock) {
//
//                     switch (LockType) {
//
//                         case SimpleMediaLock: {
//                             FdoExtension->LockCount--;
//                             break;
//                         }
//
//                         case SecureMediaLock: {
//                             fsContext->LockCount--;
//                             FdoExtension->ProtectedLockCount--;
//                             break;
//                         }
//
//                         case InternalMediaLock: {
//                             FdoExtension->InternalLockCount--;
//                             break;
//                         }
//                     }
//
//                 } else {
//
//                     switch (LockType) {
//
//                         case SimpleMediaLock: {
//                             FdoExtension->LockCount++;
//                             break;
//                         }
//
//                         case SecureMediaLock: {
//                             fsContext->LockCount++;
//                             FdoExtension->ProtectedLockCount++;
//                             break;
//                         }
//
//                         case InternalMediaLock: {
//                             FdoExtension->InternalLockCount++;
//                             break;
//                         }
//                     }
//                 }
//
//             }
//
//         } else {
//
//             TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL,
//                         "ClasspEjectionControl: Succeeded\n"));
//
//         }
//
//         TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL,
//                     "ClasspEjectionControl: "
//                     "Current Counts: Internal: %x  Secure: %x  Simple: %x\n",
//                     FdoExtension->InternalLockCount,
//                     FdoExtension->ProtectedLockCount,
//                     FdoExtension->LockCount
//                     ));
//
//         KeSetEvent(&(FdoExtension->EjectSynchronizationEvent),
//                    IO_NO_INCREMENT,
//                    FALSE);
//         KeLeaveCriticalRegion();
//
//         if (srb) {
//             ClassFreeOrReuseSrb(FdoExtension, srb);
//         }
//
//     } _SEH2_END;
//     return status;
// }
//
//
// _IRQL_requires_max_(PASSIVE_LEVEL)
// PFILE_OBJECT_EXTENSION
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// ClassGetFsContext(
//      PCOMMON_DEVICE_EXTENSION CommonExtension,
//      PFILE_OBJECT FileObject
//     )
// {
//     PAGED_CODE();
//     return GetDictionaryEntry(&(CommonExtension->FileObjectDictionary),
//                               (ULONGLONG) FileObject);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
