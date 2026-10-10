use crate::types::*;
use kmdf_sys::*;
use ntkrnl_sys::ddk::{IRP, PIRP, IO_STACK_LOCATION, PIO_STACK_LOCATION, GUID, LPGUID, LPCGUID, DEVICE_OBJECT, PDEVICE_OBJECT, MDL, PMDL, PIO_COMPLETION_ROUTINE};
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{IoCopyCurrentIrpStackLocationToNext, IoGetCurrentIrpStackLocation, IoGetNextIrpStackLocation, IoMarkIrpPending, IoSetCompletionRoutine, IoSkipCurrentIrpStackLocation};
use ntkrnl_sys::ddk::_POWER_STATE_TYPE_DevicePowerState as DevicePowerState;
use ntkrnl_sys::ddk::_DEVICE_POWER_STATE_PowerDeviceD0 as PowerDeviceD0;
use ntkrnl_sys::ddk::_MONITOR_DISPLAY_STATE_PowerMonitorOff as PowerMonitorOff;
use core::ffi::c_void;

extern "system" {
    fn ScratchBuffer_BeginUse(DeviceExtension: *mut CDROM_DEVICE_EXTENSION);
    fn ScratchBuffer_EndUse(DeviceExtension: *mut CDROM_DEVICE_EXTENSION);
    fn ScratchBuffer_ExecuteCdbEx(DeviceExtension: *mut CDROM_DEVICE_EXTENSION, OriginalRequest: *mut core::ffi::c_void, TransferSize: u32, GetDataFromDevice: u8, Cdb: *mut CDB, OprationLength: u8, TimeoutValue: u32,) -> i32;
    fn ScratchBuffer_ExecuteCdb(DeviceExtension: PCDROM_DEVICE_EXTENSION, OriginalRequest: *mut core::ffi::c_void, TransferSize: u32, GetDataFromDevice: u8, Cdb: PCDB, OprationLength: u8) -> NTSTATUS;
    fn NT_ASSERT(condition: i32);
    fn DeviceGetExtension(Device: WDFDEVICE) -> *mut CDROM_DEVICE_EXTENSION;
    // DeviceSendPowerDownProcessRequest resolves from crate::types (canonical
    // signature: (PCDROM_DEVICE_EXTENSION, PFN_WDF_REQUEST_COMPLETION_ROUTINE, WDFCONTEXT) -> NTSTATUS).
}
unsafe extern "C" {
    fn TimeOutValueGetCapValue(timeOutValue: u32, value: u32) -> u32;
}
unsafe extern "system" {
    fn KeQueryTickCount(TickCount: *mut ntkrnl_sys::ddk::LARGE_INTEGER);
}
unsafe extern "C" {
    pub fn RequestClearSendTime(Request: WDFREQUEST);
}
extern "C" {
    fn ScratchBuffer_SetupSrb(DeviceExtension: *mut CDROM_DEVICE_EXTENSION, OriginalRequest: *mut core::ffi::c_void, MaximumTransferLength: u32, GetDataFromDevice: u8);
    fn ScratchBuffer_SendSrb(DeviceExtension: *mut CDROM_DEVICE_EXTENSION, SynchronousSrb: u8, SrbHistoryItem: *mut *mut SRB_HISTORY_ITEM,) -> i32;
    fn RequestSenseInfoInterpretForScratchBuffer( DeviceExtension: *mut CDROM_DEVICE_EXTENSION, TimesRetried: u32, Status: *mut NTSTATUS, RetryIn100nsUnits: *mut i64, ) -> i32;
    fn DeviceMarkActive(DeviceExtension: PCDROM_DEVICE_EXTENSION, IsActive: BOOLEAN, SetIdleTimeout: BOOLEAN,);
}
unsafe extern "system" {
    fn WDF_REQUEST_REUSE_PARAMS_INIT(params: *mut WDF_REQUEST_REUSE_PARAMS, flags: u32, status: NTSTATUS);
}

const SCSIOP_MEDIUM_REMOVAL: u8 = 0x1E;

const SP_UNTAGGED: u8 = 0xFF;        // (UCHAR)~0 per srb.h
// SRB_SIMPLE_TAG_REQUEST / SRB_FUNCTION_SHUTDOWN / SRB_FUNCTION_FLUSH resolve
// from crate::types (correct bindgen values).

const WDF_REQUEST_REUSE_NO_FLAGS: u32 = 0;
const IOCTL_SCSI_EXECUTE_IN: u32 = 0x1b0011; // unknown  // was 0 (LLM-invented); bindgen-correct
const IRP_MJ_SCSI: u8 = 0x0F;
const SENSE_BUFFER_SIZE: u32 = 18;
const TRACE_LEVEL_ERROR: u32 = 2; // storswtr.h:26-30
const TRACE_FLAG_GENERAL: u32 = 0; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)
#[no_mangle]
pub unsafe extern "system" fn RequestProcessShutdownFlush(device: WDFDEVICE, irp: PIRP) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_SUCCESS;
    let mut currentStack: *mut IO_STACK_LOCATION;
    let deviceExtension: *mut CDROM_DEVICE_EXTENSION = DeviceGetExtension(device);

    WdfWaitLockAcquire((*deviceExtension).ShutdownFlushWaitLock, core::ptr::null_mut());
    currentStack = IoGetCurrentIrpStackLocation(irp);
    WdfIoQueueStopSynchronously((*deviceExtension).SerialIOQueue);

    if NT_SUCCESS(status) {
        status = DeviceScratchSyncCache(deviceExtension);
    }
    if NT_SUCCESS(status) {
        if (*currentStack).MajorFunction == IRP_MJ_SHUTDOWN {
            status = DeviceScratchPreventMediaRemoval(deviceExtension, 0);
        }
    }
    if NT_SUCCESS(status) {
        status = RequestIssueShutdownFlush(deviceExtension, irp);
    }
    WdfIoQueueStart((*deviceExtension).SerialIOQueue);
    WdfWaitLockRelease((*deviceExtension).ShutdownFlushWaitLock);

    (*irp).IoStatus.__bindgen_anon_1.Status = status;
    IoCompleteRequest(irp, 0);
    status
}
#[no_mangle]
pub unsafe extern "system" fn RequestProcessSetPower(
    Device: WDFDEVICE,
    Irp: PIRP,
) -> NTSTATUS {
    let deviceExtension: PCDROM_DEVICE_EXTENSION = DeviceGetExtension(Device);
    let currentStack = IoGetCurrentIrpStackLocation(Irp);
    let mut status: NTSTATUS;
    let mut IrpMarkedPending: BOOLEAN = 0;

    if (*currentStack).Parameters.Power.Type == DevicePowerState
        && (*currentStack).Parameters.Power.State.DeviceState != PowerDeviceD0
    {
        IoCopyCurrentIrpStackLocationToNext(Irp);
        IoSetCompletionRoutine(
            Irp,
            core::mem::transmute::<_, PIO_COMPLETION_ROUTINE>(
                RequestProcessPowerIrpCompletion
                    as unsafe extern "system" fn(PDEVICE_OBJECT, PIRP, *mut core::ffi::c_void) -> NTSTATUS,
            ),
            deviceExtension as *mut core::ffi::c_void,
            1,
            1,
            1,
        );
        IoMarkIrpPending(Irp);
        IrpMarkedPending = 1;
    } else {
        IoSkipCurrentIrpStackLocation(Irp);
    }

    status = WdfDeviceWdmDispatchPreprocessedIrp(Device, Irp);

    if IrpMarkedPending != 0 {
        let _ = status;
        return STATUS_PENDING;
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn DeviceScratchSyncCache(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
) -> i32 {
    let mut status: i32;
    let mut transferSize: u32 = 0;
    let mut cdb: CDB = core::mem::zeroed();

    ScratchBuffer_BeginUse(DeviceExtension);

    // RtlZeroMemory(&cdb, sizeof(CDB)); already zeroed

    // Set up the CDB
    // srb->QueueTag = SP_UNTAGGED;
    // srb->QueueAction = SRB_SIMPLE_TAG_REQUEST;
    unsafe {
        cdb.SYNCHRONIZE_CACHE10.as_mut().OperationCode = 0x35; // SCSIOP_SYNCHRONIZE_CACHE
    }

    status = ScratchBuffer_ExecuteCdbEx(
        DeviceExtension,
        core::ptr::null_mut(),
        transferSize,
        0, // FALSE
        &mut cdb,
        10,
        TimeOutValueGetCapValue((*DeviceExtension).TimeOutValue, 4),
    );

    ScratchBuffer_EndUse(DeviceExtension);

    status
}
#[no_mangle]
pub unsafe extern "C" fn DeviceScratchPreventMediaRemoval(
    DeviceExtension: PCDROM_DEVICE_EXTENSION,
    Prevent: u8,
) -> i32 {
    let mut status: i32;
    let mut transferSize: u32 = 0;

    let mut cdb: CDB = core::mem::zeroed();
    core::ptr::write_bytes((core::ptr::addr_of_mut!(cdb)) as *mut u8, 0, core::mem::size_of::<CDB>());

    ScratchBuffer_BeginUse(DeviceExtension);

    // Set up the CDB
    let cdb_ptr: *mut CDB = core::ptr::addr_of_mut!(cdb);
    (*cdb_ptr).MEDIA_REMOVAL.as_mut().OperationCode = SCSIOP_MEDIUM_REMOVAL;
    (*cdb_ptr).MEDIA_REMOVAL.as_mut().set_Prevent(Prevent);

    status = ScratchBuffer_ExecuteCdb(
        DeviceExtension,
        core::ptr::null_mut(),
        transferSize,
        0u8,
        cdb_ptr,
        6u8,
    );

    ScratchBuffer_EndUse(DeviceExtension);

    status
}
#[no_mangle]
pub unsafe extern "C" fn RequestIssueShutdownFlush(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_SUCCESS;
    let srb: *mut SCSI_REQUEST_BLOCK = (*DeviceExtension).ScratchContext.ScratchSrb;
    let currentStack: *mut IO_STACK_LOCATION;

    let mut transferSize: u32 = 0;
    let mut shouldRetry: i32;
    let mut timesAlreadyRetried: u32 = 0;
    let mut retryIn100nsUnits: i64 = 0;

    currentStack = IoGetCurrentIrpStackLocation(Irp);

    ScratchBuffer_BeginUse(DeviceExtension);

    {
        ScratchBuffer_SetupSrb(DeviceExtension, core::ptr::null_mut(), transferSize, 0);

        (*srb).QueueTag = SP_UNTAGGED;
        (*srb).QueueAction = SRB_SIMPLE_TAG_REQUEST as u8;
        (*srb).TimeOutValue = TimeOutValueGetCapValue(
            (*DeviceExtension).TimeOutValue,
            4,
        );
        (*srb).CdbLength = 0;

        if (*currentStack).MajorFunction == IRP_MJ_SHUTDOWN {
            (*srb).Function = SRB_FUNCTION_SHUTDOWN as u8;
        } else {
            (*srb).Function = SRB_FUNCTION_FLUSH as u8;
        }

        ScratchBuffer_SendSrb(DeviceExtension, 1, core::ptr::null_mut());

        shouldRetry = RequestSenseInfoInterpretForScratchBuffer(
            DeviceExtension,
            timesAlreadyRetried,
            &mut status,
            &mut retryIn100nsUnits,
        );

        let _ = shouldRetry; // UNREFERENCED_PARAMETER(shouldRetry)
        let _ = status;      // UNREFERENCED_PARAMETER(status)

        status = WdfRequestGetStatus((*DeviceExtension).ScratchContext.ScratchRequest);
    }

    ScratchBuffer_EndUse(DeviceExtension);

    return status;
}

#[no_mangle]
pub unsafe extern "system" fn RequestUnlockQueueCompletion(
    request: WDFREQUEST,
    target: WDFIOTARGET,
    params: PWDF_REQUEST_COMPLETION_PARAMS,
    context: WDFCONTEXT,
) {
    // Translate: PIRP Irp = Context;
    let irp: ntkrnl_sys::ddk::PIRP = context as ntkrnl_sys::ddk::PIRP;
    // Translate: WDFDEVICE device = WdfIoTargetGetDevice(Target);
    let device: WDFDEVICE = WdfIoTargetGetDevice(target);
    // Translate: PCDROM_DEVICE_EXTENSION deviceExtension = DeviceGetExtension(device);
    let device_extension: *mut CDROM_DEVICE_EXTENSION = DeviceGetExtension(device);

    // UNREFERENCED_PARAMETER(Request);
    let _ = &request;
    // UNREFERENCED_PARAMETER(Params);
    let _ = &params;

    // deviceExtension->PowerContext.Options.LockQueue = FALSE;
    (*device_extension).PowerContext.Options.set_LockQueue(0); // FALSE

    // PowerContextEndUse(deviceExtension);
    PowerContextEndUse(device_extension);

    // Complete the original power irp
    ntkrnl_sys::driver_ddk::IoCompleteRequest(
        irp,
        ntkrnl_sys::constants::IO_NO_INCREMENT,
    );
}

#[no_mangle]
pub unsafe extern "system" fn RequestProcessPowerIrpCompletion(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Context: *mut core::ffi::c_void,
) -> NTSTATUS {
    // Forward declarations for unknown externals

    const STATUS_MORE_PROCESSING_REQUIRED: NTSTATUS = 0xC0000016u32 as NTSTATUS;
    const STATUS_CONTINUE_COMPLETION: NTSTATUS = 0x00000000;

    let _ = DeviceObject;
    let deviceExtension: *mut CDROM_DEVICE_EXTENSION = Context as *mut CDROM_DEVICE_EXTENSION;

    if (*Irp).PendingReturned != 0 {
        IoMarkIrpPending(Irp);
    }

    let currentStack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);

    NT_ASSERT(
        if (*currentStack).Parameters.Power.Type == DevicePowerState { 1 } else { 0 },
    );
    NT_ASSERT(
        if (*currentStack).Parameters.Power.State.DeviceState != PowerDeviceD0 { 1 } else { 0 },
    );

    // _Analysis_assume_ removed: SAL no-op annotation; argument must never be evaluated (0xB8 class)

    (*deviceExtension).PowerContext.PowerChangeState.PowerDown += 1;

    if (*deviceExtension).PowerContext.Options.LockQueue() != 0 {
        DeviceSendPowerDownProcessRequest(
            deviceExtension,
            core::mem::transmute::<_, PFN_WDF_REQUEST_COMPLETION_ROUTINE>(
                RequestUnlockQueueCompletion
                    as unsafe extern "system" fn(WDFREQUEST, WDFIOTARGET, PWDF_REQUEST_COMPLETION_PARAMS, WDFCONTEXT),
            ),
            Irp as WDFCONTEXT,
        );
        return STATUS_MORE_PROCESSING_REQUIRED;
    }

    if (*deviceExtension).PowerContext.InUse != 0 {
        PowerContextEndUse(deviceExtension);
    }

    return STATUS_CONTINUE_COMPLETION;
}
#[no_mangle]
pub unsafe extern "C" fn PowerContextReuseRequest(DeviceExtension: *mut CDROM_DEVICE_EXTENSION) -> NTSTATUS {
    let mut status: NTSTATUS;
    let mut reuseParams: core::mem::MaybeUninit<WDF_REQUEST_REUSE_PARAMS> = core::mem::MaybeUninit::uninit();
    let mut irp: PIRP;

    // Zero SenseData
    {
        let sense_data_size = core::mem::size_of_val(&(*DeviceExtension).PowerContext.SenseData);
        core::ptr::write_bytes(
            core::ptr::addr_of_mut!((*DeviceExtension).PowerContext.SenseData) as *mut u8,
            0,
            sense_data_size,
        );
    }
    // Zero Srb
    {
        let srb_size = core::mem::size_of_val(&(*DeviceExtension).PowerContext.Srb);
        core::ptr::write_bytes(
            core::ptr::addr_of_mut!((*DeviceExtension).PowerContext.Srb) as *mut u8,
            0,
            srb_size,
        );
    }

    irp = WdfRequestWdmGetIrp((*DeviceExtension).PowerContext.PowerRequest);

    WDF_REQUEST_REUSE_PARAMS_INIT(
        reuseParams.as_mut_ptr(),
        WDF_REQUEST_REUSE_NO_FLAGS,
        ntkrnl_sys::constants::STATUS_NOT_SUPPORTED,
    );
    status = WdfRequestReuse((*DeviceExtension).PowerContext.PowerRequest, reuseParams.as_mut_ptr());

    if ntkrnl_sys::base::NT_SUCCESS(status) {
        status = WdfIoTargetFormatRequestForInternalIoctlOthers(
            (*DeviceExtension).IoTarget,
            (*DeviceExtension).PowerContext.PowerRequest,
            IOCTL_SCSI_EXECUTE_IN,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );

        if !ntkrnl_sys::base::NT_SUCCESS(status) {
            let msg = b"PowerContextReuseRequest: WdfIoTargetFormatRequestForInternalIoctlOthers failed, %!STATUS!\n\0".as_ptr() as *const i8;
            TracePrint!(TRACE_LEVEL_ERROR, TRACE_FLAG_GENERAL, msg, status);
        }
    }

    if ntkrnl_sys::base::NT_SUCCESS(status) {
        let nextStack: *mut IO_STACK_LOCATION;
        nextStack = IoGetNextIrpStackLocation(irp);
        (*(nextStack)).MajorFunction = IRP_MJ_SCSI;
        (*(nextStack)).Parameters.Scsi.Srb = core::ptr::addr_of_mut!((*DeviceExtension).PowerContext.Srb) as *mut ntkrnl_sys::ddk::_SCSI_REQUEST_BLOCK;
        (*DeviceExtension).PowerContext.Srb.Length = core::mem::size_of::<SCSI_REQUEST_BLOCK>() as u16;
        (*DeviceExtension).PowerContext.Srb.OriginalRequest = irp as *mut core::ffi::c_void;
        (*DeviceExtension).PowerContext.Srb.SenseInfoBuffer = core::ptr::addr_of_mut!((*DeviceExtension).PowerContext.SenseData) as *mut core::ffi::c_void;
        (*DeviceExtension).PowerContext.Srb.SenseInfoBufferLength = SENSE_BUFFER_SIZE as u8;
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn PowerContextBeginUse(DeviceExtension: *mut CDROM_DEVICE_EXTENSION) -> NTSTATUS {
    let mut status: NTSTATUS;

    NT_ASSERT((!((*DeviceExtension).PowerContext.InUse)) as i32);

    (*DeviceExtension).PowerContext.InUse = 1; // TRUE
    (*DeviceExtension).PowerContext.RetryCount = 3; /* MAXIMUM_RETRIES */
    (*DeviceExtension).PowerContext.RetryIntervalIn100ns = 0i64; // LONGLONG

    KeQueryTickCount(core::ptr::addr_of_mut!((*DeviceExtension).PowerContext.StartTime));

    RtlZeroMemory(
        core::ptr::addr_of_mut!((*DeviceExtension).PowerContext.Options) as *mut core::ffi::c_void,
        core::mem::size_of_val(&(*DeviceExtension).PowerContext.Options)
    );
    RtlZeroMemory(
        core::ptr::addr_of_mut!((*DeviceExtension).PowerContext.PowerChangeState) as *mut core::ffi::c_void,
        core::mem::size_of_val(&(*DeviceExtension).PowerContext.PowerChangeState)
    );

    status = PowerContextReuseRequest(DeviceExtension);

    RequestClearSendTime(((*DeviceExtension).PowerContext.PowerRequest as *mut core::ffi::c_void) as _);

    return status;
}
#[no_mangle]
pub unsafe extern "C" fn PowerContextEndUse(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
) -> NTSTATUS {
    NT_ASSERT((*DeviceExtension).PowerContext.InUse as i32);
    (*DeviceExtension).PowerContext.InUse = 0;
    KeQueryTickCount(core::ptr::addr_of_mut!(
        (*DeviceExtension).PowerContext.CompleteTime
    ));
    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn DevicePowerSettingCallback(
    SettingGuid: *const GUID,
    Value: *mut core::ffi::c_void,
    ValueLength: u32,
    Context: *mut core::ffi::c_void,
) -> i32 {
    // PAGED_CODE(); // macro – no‑op in transliteration
    let DeviceExtension = Context as *mut CDROM_DEVICE_EXTENSION;
    let guid_console_display_state = GUID {
        Data1: 0x6FE69556,
        Data2: 0x704A,
        Data3: 0x47A0,
        Data4: [0x8F, 0x24, 0xC2, 0x8D, 0x93, 0x6F, 0xDA, 0x47],
    };
    const SRB_FLAGS_NO_KEEP_AWAKE: u32 = 0x100000;  // was 0x40 (LLM-invented); bindgen-correct

    // IsEqualGUID(a, b): RtlEqualMemory(a, b, sizeof(GUID)) per the DDK macro.
    let is_equal_guid = |a: *const GUID, b: *const GUID| -> bool {
        !a.is_null()
            && !b.is_null()
            && (*a).Data1 == (*b).Data1
            && (*a).Data2 == (*b).Data2
            && (*a).Data3 == (*b).Data3
            && (*a).Data4 == (*b).Data4
    };
    if is_equal_guid(SettingGuid, &guid_console_display_state as *const GUID) {
        if (ValueLength == core::mem::size_of::<u32>() as u32) && (!Value.is_null()) {
            let DisplayState = *(Value as *const u32);
            // _Analysis_assume_(DeviceExtension != NULL); // SAL annotation – skipped
            if (*DeviceExtension).IsInitialized != 0 {
                let media = (*DeviceExtension).MediaChangeDetectionInfo;
                if DisplayState == PowerMonitorOff {
                    DeviceMarkActive(DeviceExtension, 0u8, 1u8);
                    let srb_flags = &mut (*media).SrbFlags;
                    *srb_flags |= SRB_FLAGS_NO_KEEP_AWAKE;
                } else {
                    let srb_flags = &mut (*media).SrbFlags;
                    *srb_flags &= !SRB_FLAGS_NO_KEEP_AWAKE;
                    DeviceMarkActive(DeviceExtension, 1u8, 1u8);
                }
            }
        }
    }

    STATUS_SUCCESS
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*--
//
// Copyright (C) Microsoft Corporation. All rights reserved.
//
// Module Name:
//
//     pnppower.c
//
// Abstract:
//
//     Functions to handle PnP and Power IRPs.
//
// Environment:
//
//     kernel mode only
//
// Notes:
//     optical devices do not need to issue SPIN UP when power up.
//     The device itself should SPIN UP to process commands.
//
//
// Revision History:
//
// --*/
//
//
// #include "ntddk.h"
// #include "wdfcore.h"
//
// #include "cdrom.h"
// #include "ioctl.h"
// #include "scratch.h"
// #include "mmc.h"
//
// #ifdef DEBUG_USE_WPP
// #include "pnppower.tmh"
// #endif
//
//
// NTSTATUS
// DeviceScratchSyncCache(
//      PCDROM_DEVICE_EXTENSION DeviceExtension
//     );
//
// NTSTATUS
// DeviceScratchPreventMediaRemoval(
//      PCDROM_DEVICE_EXTENSION DeviceExtension,
//      BOOLEAN                 Prevent
//     );
//
// NTSTATUS
// RequestIssueShutdownFlush(
//      PCDROM_DEVICE_EXTENSION DeviceExtension,
//      PIRP                    Irp
//     );
//
// IO_COMPLETION_ROUTINE RequestProcessPowerIrpCompletion;
//
// EVT_WDF_REQUEST_COMPLETION_ROUTINE RequestUnlockQueueCompletion;
//
// #ifdef ALLOC_PRAGMA
//
// #pragma alloc_text(PAGE, DevicePowerSettingCallback)
//
// #endif
//
// #pragma warning(push)
// #pragma warning(disable:4152) // nonstandard extension, function/data pointer conversion in expression
//
// #pragma warning(disable:28118)  // Dispatch routines for IRP_MJ_SHUTDOWN, IRP_MJ_FLUSH_BUFFERS occur at PASSIVE_LEVEL.
//                                 // WDF defines EVT_WDFDEVICE_WDM_IRP_PREPROCESS with _IRQL_requires_max_(DISPATCH_LEVEL),
//                                 // triggering a false positive for this warning.
//
// NTSTATUS
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// RequestProcessShutdownFlush(
//     WDFDEVICE  Device,
//     PIRP       Irp
//     )
// /*++
//
// Routine Description:
//
//     process IRP: IRP_MJ_SHUTDOWN, IRP_MJ_FLUSH_BUFFERS
//
// Arguments:
//
//     Device - device object
//     Irp - the irp
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS                status = STATUS_SUCCESS;
//     PIO_STACK_LOCATION      currentStack = NULL;
//     PCDROM_DEVICE_EXTENSION deviceExtension = DeviceGetExtension(Device);
//
//     //add trace info
//
//     // acquire the shutdown/flush lock
//     WdfWaitLockAcquire(deviceExtension->ShutdownFlushWaitLock, NULL);
//
//     currentStack = IoGetCurrentIrpStackLocation(Irp);
//
//     // finish all current requests
//     WdfIoQueueStopSynchronously(deviceExtension->SerialIOQueue);
//
//     // sync cache
//     if (NT_SUCCESS(status))
//     {
//         // safe to use scratch srb to send the request.
//         status = DeviceScratchSyncCache(deviceExtension);
//     }
//
//     // For SHUTDOWN, allow media removal.
//     if (NT_SUCCESS(status))
//     {
//         if (currentStack->MajorFunction == IRP_MJ_SHUTDOWN)
//         {
//             // safe to use scratch srb to send the request.
//             status = DeviceScratchPreventMediaRemoval(deviceExtension, FALSE);
//         }
//     }
//
//     // Use original IRP, send SRB_FUNCTION_SHUTDOWN or SRB_FUNCTION_FLUSH (no retry)
//     if (NT_SUCCESS(status))
//     {
//         status = RequestIssueShutdownFlush(deviceExtension, Irp);
//     }
//
//     // restart queue to allow processing further requests.
//     WdfIoQueueStart(deviceExtension->SerialIOQueue);
//
//     // release the shutdown/flush lock
//     WdfWaitLockRelease(deviceExtension->ShutdownFlushWaitLock);
//
//     // 6. complete the irp
//     Irp->IoStatus.Status = status;
//     IoCompleteRequest(Irp, 0);
//
//     return status;
// }
//
// NTSTATUS
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// RequestProcessSetPower(
//     WDFDEVICE  Device,
//     PIRP       Irp
//     )
// /*++
//
// Routine Description:
//
//     process IRP: IRP_MJ_POWER
//
// Arguments:
//
//     Device - device object
//     Irp - the irp
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     PCDROM_DEVICE_EXTENSION deviceExtension = DeviceGetExtension(Device);
//     PIO_STACK_LOCATION      currentStack;
//     NTSTATUS                status;
//     BOOLEAN                 IrpMarkedPending = FALSE;
//
//     currentStack = IoGetCurrentIrpStackLocation(Irp);
//
//     if ((currentStack->Parameters.Power.Type == DevicePowerState) &&
//         (currentStack->Parameters.Power.State.DeviceState != PowerDeviceD0))
//     {
//         // We need to unlock the device queue in D3 postprocessing.
//         IoCopyCurrentIrpStackLocationToNext(Irp);
//         IoSetCompletionRoutine(Irp,
//                                RequestProcessPowerIrpCompletion,
//                                deviceExtension,
//                                TRUE,
//                                TRUE,
//                                TRUE);
//
//         // Mark the Irp pending as we'll defer the I/O completion.
//         IoMarkIrpPending(Irp);
//         IrpMarkedPending = TRUE;
//     }
//     else {
//
//         IoSkipCurrentIrpStackLocation(Irp);
//     }
//
// #pragma warning(push)
// #pragma warning(disable: 28193) // OACR will complain that the status variable is not examined.
//
//     //
//     // Deliver the IRP back to the framework.
//     //
//
//     status = WdfDeviceWdmDispatchPreprocessedIrp(Device, Irp);
//
//     if (IrpMarkedPending)
//     {
//         UNREFERENCED_PARAMETER(status);
//         return STATUS_PENDING;
//     }
//
// #pragma warning(pop)
//
//     return status;
// }
//
// // use scratch SRB to issue SYNC CACHE command.
// NTSTATUS
// DeviceScratchSyncCache(
//      PCDROM_DEVICE_EXTENSION DeviceExtension
//     )
// /*++
//
// Routine Description:
//
//     use scratch buffer to send SYNC CACHE command
//
// Arguments:
//
//     DeviceExtension - device context
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS    status = STATUS_SUCCESS;
//     ULONG       transferSize = 0;
//     CDB         cdb;
//
//     ScratchBuffer_BeginUse(DeviceExtension);
//
//     RtlZeroMemory(&cdb, sizeof(CDB));
//     // Set up the CDB
//     cdb.SYNCHRONIZE_CACHE10.OperationCode = SCSIOP_SYNCHRONIZE_CACHE;
//     //srb->QueueTag = SP_UNTAGGED;
//     //srb->QueueAction = SRB_SIMPLE_TAG_REQUEST;
//
//     status = ScratchBuffer_ExecuteCdbEx(DeviceExtension, NULL, transferSize, FALSE, &cdb, 10, TimeOutValueGetCapValue(DeviceExtension->TimeOutValue, 4));
//
//     ScratchBuffer_EndUse(DeviceExtension);
//
//     return status;
// }
//
// NTSTATUS
// DeviceScratchPreventMediaRemoval(
//      PCDROM_DEVICE_EXTENSION DeviceExtension,
//      BOOLEAN                 Prevent
//     )
// /*++
//
// Routine Description:
//
//     use scratch SRB to issue ALLOW/PREVENT MEDIA REMOVAL command.
//
// Arguments:
//
//     DeviceExtension - device context
//     Prevent - TRUE (prevent media removal); FALSE (allow media removal)
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS    status = STATUS_SUCCESS;
//     ULONG       transferSize = 0;
//     CDB         cdb;
//
//     ScratchBuffer_BeginUse(DeviceExtension);
//
//     RtlZeroMemory(&cdb, sizeof(CDB));
//     // Set up the CDB
//     cdb.MEDIA_REMOVAL.OperationCode = SCSIOP_MEDIUM_REMOVAL;
//     cdb.MEDIA_REMOVAL.Prevent = Prevent;
//     //srb->QueueTag = SP_UNTAGGED;
//     //srb->QueueAction = SRB_SIMPLE_TAG_REQUEST;
//
//     status = ScratchBuffer_ExecuteCdb(DeviceExtension, NULL, transferSize, FALSE, &cdb, 6);
//
//     ScratchBuffer_EndUse(DeviceExtension);
//
//     return status;
// }
//
// NTSTATUS
// RequestIssueShutdownFlush(
//      PCDROM_DEVICE_EXTENSION DeviceExtension,
//      PIRP                    Irp
//     )
// /*++
//
// Routine Description:
//
//     issue SRB function Flush/Shutdown command.
//
// Arguments:
//
//     DeviceExtension - device context
//     Irp - the irp
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS            status = STATUS_SUCCESS;
//     PSCSI_REQUEST_BLOCK srb = DeviceExtension->ScratchContext.ScratchSrb;
//     PIO_STACK_LOCATION  currentStack = NULL;
//
//     ULONG       transferSize = 0;
//     BOOLEAN     shouldRetry = TRUE;
//     ULONG       timesAlreadyRetried = 0;
//     LONGLONG    retryIn100nsUnits = 0;
//
//
//     currentStack = IoGetCurrentIrpStackLocation(Irp);
//
//
//     ScratchBuffer_BeginUse(DeviceExtension);
//
//     // no retry needed.
//     {
//         ScratchBuffer_SetupSrb(DeviceExtension, NULL, transferSize, FALSE);
//
//         // Set up the SRB/CDB
//         srb->QueueTag = SP_UNTAGGED;
//         srb->QueueAction = SRB_SIMPLE_TAG_REQUEST;
//         srb->TimeOutValue = TimeOutValueGetCapValue(DeviceExtension->TimeOutValue, 4);
//         srb->CdbLength = 0;
//
//         if (currentStack->MajorFunction == IRP_MJ_SHUTDOWN)
//         {
//             srb->Function = SRB_FUNCTION_SHUTDOWN;
//         }
//         else
//         {
//             srb->Function = SRB_FUNCTION_FLUSH;
//         }
//
//         ScratchBuffer_SendSrb(DeviceExtension, TRUE, NULL);
//
//         shouldRetry = RequestSenseInfoInterpretForScratchBuffer(DeviceExtension,
//                                                                 timesAlreadyRetried,
//                                                                 &status,
//                                                                 &retryIn100nsUnits);
//         UNREFERENCED_PARAMETER(shouldRetry); //defensive coding, avoid PREFAST warning.
//         UNREFERENCED_PARAMETER(status); //defensive coding, avoid PREFAST warning.
//
//         // retrieve the real status from the request.
//         status = WdfRequestGetStatus(DeviceExtension->ScratchContext.ScratchRequest);
//     }
//
//     ScratchBuffer_EndUse(DeviceExtension);
//
//
//     return status;
// }
//
// VOID
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// RequestUnlockQueueCompletion (
//      WDFREQUEST Request,
//      WDFIOTARGET Target,
//      PWDF_REQUEST_COMPLETION_PARAMS Params,
//      WDFCONTEXT Context
//     )
// {
//     PIRP                    Irp = Context;
//     WDFDEVICE               device = WdfIoTargetGetDevice(Target);
//     PCDROM_DEVICE_EXTENSION deviceExtension = DeviceGetExtension(device);
//
//     UNREFERENCED_PARAMETER(Request);
//     UNREFERENCED_PARAMETER(Params);
//
//     deviceExtension->PowerContext.Options.LockQueue = FALSE;
//
//     PowerContextEndUse(deviceExtension);
//
//     // Complete the original power irp
//     IoCompleteRequest( Irp, IO_NO_INCREMENT );
// }
//
// NTSTATUS
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// RequestProcessPowerIrpCompletion(
//      PDEVICE_OBJECT DeviceObject,
//      PIRP Irp,
//     () PVOID Context
//     )
// /*++
//
// Routine Description:
//
//     Free the Irp.
//
// Arguments:
//
//     DeviceObject - device that the completion routine fires on.
//
//     Irp - The irp to be completed.
//
//     Context - IRP context
//
// Return Value:
//     NTSTATUS
//
// --*/
// {
//     PCDROM_DEVICE_EXTENSION deviceExtension = Context;
//     PIO_STACK_LOCATION      currentStack;
//
//     UNREFERENCED_PARAMETER(DeviceObject);
//
//     if (Irp->PendingReturned)
//     {
//         IoMarkIrpPending( Irp );
//     }
//
//     currentStack = IoGetCurrentIrpStackLocation(Irp);
//
//     NT_ASSERT(currentStack->Parameters.Power.Type == DevicePowerState);
//     NT_ASSERT(currentStack->Parameters.Power.State.DeviceState != PowerDeviceD0);
//
//     _Analysis_assume_(deviceExtension != NULL);
//
//     deviceExtension->PowerContext.PowerChangeState.PowerDown++;
//
//     // Step 5. UNLOCK QUEUE
//     if (deviceExtension->PowerContext.Options.LockQueue)
//     {
//         (VOID)DeviceSendPowerDownProcessRequest(deviceExtension,
//                                                    RequestUnlockQueueCompletion,
//                                                    Irp);
//
//         // Let the completion routine complete the Irp
//         return STATUS_MORE_PROCESSING_REQUIRED;
//     }
//
//     // Release the power context if it wasn't already done as part of D0Exit handling
//     if (deviceExtension->PowerContext.InUse)
//     {
//         PowerContextEndUse(deviceExtension);
//     }
//
//     return STATUS_CONTINUE_COMPLETION;
// }
//
//
// _IRQL_requires_max_(DISPATCH_LEVEL)
// NTSTATUS
// PowerContextReuseRequest(
//      PCDROM_DEVICE_EXTENSION DeviceExtension
//     )
// /*++
//
// Routine Description:
//
//     reset fields for the request.
//
// Arguments:
//
//     DeviceExtension - device context
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS                 status = STATUS_SUCCESS;
//     WDF_REQUEST_REUSE_PARAMS reuseParams;
//     PIRP                     irp = NULL;
//
//     RtlZeroMemory(&(DeviceExtension->PowerContext.SenseData), sizeof(DeviceExtension->PowerContext.SenseData));
//     RtlZeroMemory(&(DeviceExtension->PowerContext.Srb), sizeof(DeviceExtension->PowerContext.Srb));
//
//     irp = WdfRequestWdmGetIrp(DeviceExtension->PowerContext.PowerRequest);
//
//     // Re-use the previously created PowerRequest object and format it
//     WDF_REQUEST_REUSE_PARAMS_INIT(&reuseParams, WDF_REQUEST_REUSE_NO_FLAGS, STATUS_NOT_SUPPORTED);
//     status = WdfRequestReuse(DeviceExtension->PowerContext.PowerRequest, &reuseParams);
//     if (NT_SUCCESS(status))
//     {
//         // This request was preformated during initialization so this call should never fail.
//         status = WdfIoTargetFormatRequestForInternalIoctlOthers(DeviceExtension->IoTarget,
//                                                                 DeviceExtension->PowerContext.PowerRequest,
//                                                                 IOCTL_SCSI_EXECUTE_IN,
//                                                                 NULL, NULL,
//                                                                 NULL, NULL,
//                                                                 NULL, NULL);
//
//         if (!NT_SUCCESS(status))
//         {
//             TracePrint!((TRACE_LEVEL_ERROR, TRACE_FLAG_GENERAL,
//                        "PowerContextReuseRequest: WdfIoTargetFormatRequestForInternalIoctlOthers failed, %!STATUS!\n",
//                        status));
//         }
//     }
//
//     // Do some basic initialization of the PowerRequest, the rest will be done by the caller
//     // of this function
//     if (NT_SUCCESS(status))
//     {
//         PIO_STACK_LOCATION  nextStack = NULL;
//
//         nextStack = IoGetNextIrpStackLocation(irp);
//
//         nextStack->MajorFunction = IRP_MJ_SCSI;
//         nextStack->Parameters.Scsi.Srb = &(DeviceExtension->PowerContext.Srb);
//
//         DeviceExtension->PowerContext.Srb.Length = sizeof(SCSI_REQUEST_BLOCK);
//         DeviceExtension->PowerContext.Srb.OriginalRequest = irp;
//
//         DeviceExtension->PowerContext.Srb.SenseInfoBuffer = &(DeviceExtension->PowerContext.SenseData);
//         DeviceExtension->PowerContext.Srb.SenseInfoBufferLength = SENSE_BUFFER_SIZE;
//     }
//
//     return status;
// }
//
// _IRQL_requires_max_(DISPATCH_LEVEL)
// NTSTATUS
// PowerContextBeginUse(
//      PCDROM_DEVICE_EXTENSION DeviceExtension
//     )
// /*++
//
// Routine Description:
//
//     initialize fields in power context
//
// Arguments:
//
//     DeviceExtension - device context
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS status = STATUS_SUCCESS;
//
//     NT_ASSERT(!DeviceExtension->PowerContext.InUse);
//
//     DeviceExtension->PowerContext.InUse = TRUE;
//     DeviceExtension->PowerContext.RetryCount = MAXIMUM_RETRIES;
//     DeviceExtension->PowerContext.RetryIntervalIn100ns = 0;
//
//     KeQueryTickCount(&DeviceExtension->PowerContext.StartTime);
//
//     RtlZeroMemory(&(DeviceExtension->PowerContext.Options), sizeof(DeviceExtension->PowerContext.Options));
//     RtlZeroMemory(&(DeviceExtension->PowerContext.PowerChangeState), sizeof(DeviceExtension->PowerContext.PowerChangeState));
//
//     status = PowerContextReuseRequest(DeviceExtension);
//
//     RequestClearSendTime(DeviceExtension->PowerContext.PowerRequest);
//
//     return status;
// }
//
// _IRQL_requires_max_(DISPATCH_LEVEL)
// NTSTATUS
// PowerContextEndUse(
//      PCDROM_DEVICE_EXTENSION DeviceExtension
//     )
// /*++
//
// Routine Description:
//
//     inidate that power context using is finished.
//
// Arguments:
//
//     DeviceExtension - device context
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NT_ASSERT(DeviceExtension->PowerContext.InUse);
//
//     DeviceExtension->PowerContext.InUse = FALSE;
//
//     KeQueryTickCount(&DeviceExtension->PowerContext.CompleteTime);
//
//     return STATUS_SUCCESS;
// }
//
//
// _Function_class_(POWER_SETTING_CALLBACK)
// _IRQL_requires_same_
// NTSTATUS
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// DevicePowerSettingCallback(
//      LPCGUID SettingGuid,
//      PVOID Value,
//      ULONG ValueLength,
//      PVOID Context
//     )
// /*++
// Description:
//
//     This function is the callback for power setting notifications (registered
//     when ClasspGetD3IdleTimeout() is called for the first time).
//
//     Currently, this function is used to get the disk idle timeout value from
//     the system power settings.
//
//     This function is guaranteed to be called at PASSIVE_LEVEL.
//
// Arguments:
//
//     SettingGuid - The power setting GUID.
//     Value - Pointer to the power setting value.
//     ValueLength - Size of the Value buffer.
//     Context - The FDO's device extension.
//
// Return Value:
//
//     STATUS_SUCCESS
//
// --*/
// {
//     PCDROM_DEVICE_EXTENSION DeviceExtension = Context;
//     MONITOR_DISPLAY_STATE DisplayState;
//
//     PAGED_CODE();
//
//     if (IsEqualGUID(SettingGuid, &GUID_CONSOLE_DISPLAY_STATE)) {
//
//         if ((ValueLength == sizeof(ULONG)) && (Value != NULL)) {
//
//             DisplayState = *((PULONG)Value);
//
//             _Analysis_assume_(DeviceExtension != NULL);
//
//             //
//             // Power setting callbacks are asynchronous so make sure the device
//             // is completely initialized before taking any actions.
//             //
//             if (DeviceExtension->IsInitialized) {
//
//                 //
//                 // If monitor is off, change media change requests to not keep device active.
//                 // This allows the devices to go to sleep if there are no other active requests.
//                 //
//
//                 if (DisplayState == PowerMonitorOff) {
//
//                     //
//                     // Mark the device inactive so that it can enter a low power state.
//                     //
//
//                     DeviceMarkActive(DeviceExtension, FALSE, TRUE);
//                     SET_FLAG(DeviceExtension->MediaChangeDetectionInfo->SrbFlags, SRB_FLAGS_NO_KEEP_AWAKE);
//                 }
//                 else
//                 {
//                     CLEAR_FLAG(DeviceExtension->MediaChangeDetectionInfo->SrbFlags, SRB_FLAGS_NO_KEEP_AWAKE);
//                     DeviceMarkActive(DeviceExtension, TRUE, TRUE);
//                 }
//             }
//         }
//     }
//
//     return STATUS_SUCCESS;
// }
//
// #pragma warning(pop) // un-sets any local warning changes
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
