use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{IoGetCurrentIrpStackLocation, KeInitializeDpc, KeInitializeTimer, KeInsertQueueDpc, KeSetTimer, LARGE_INTEGER, PDEVICE_OBJECT, PIO_STACK_LOCATION, PSTORAGE_ADAPTER_DESCRIPTOR};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{IRP_MJ_SCSI, SL_OVERRIDE_VERIFY_VOLUME, STATUS_IO_DEVICE_ERROR, STATUS_VERIFY_REQUIRED, TRACE_LEVEL_INFORMATION};
use crate::types::{CLASS_SRB_STORAGE_REQUEST_BLOCK, PCDB, PCLASS_PRIVATE_FDO_DATA, PCOMMON_DEVICE_EXTENSION, PFIXED_SENSE_DATA, PFUNCTIONAL_DEVICE_EXTENSION, PSCSI_REQUEST_BLOCK, PSCSI_SENSE_DESCRIPTOR_INFORMATION, PSTORAGE_REQUEST_BLOCK, PTRANSFER_PACKET, SCSIOP_MEDIUM_REMOVAL, SCSIOP_MODE_SENSE, SCSIOP_MODE_SENSE10, SCSIOP_READ, SCSIOP_READ12, SCSIOP_READ16, SCSIOP_READ6, SCSIOP_READ_CAPACITY, SCSIOP_READ_CAPACITY16, SCSIOP_WRITE, SCSIOP_WRITE12, SCSIOP_WRITE16, SCSIOP_WRITE6, SCSI_ADSENSE_COPY_TARGET_DEVICE_ERROR, SCSI_ADSENSE_LUN_NOT_READY, SCSI_SENSEQ_DATA_UNDERRUN, SCSI_SENSE_ABORTED_COMMAND, SCSI_SENSE_COPY_ABORTED, SCSI_SENSE_DESCRIPTOR_TYPE_INFORMATION, SENSE_DATA_EX, SRB_FLAGS_DISABLE_DISCONNECT, SRB_FLAGS_DISABLE_SYNCH_TRANSFER, SRB_FLAGS_FREE_SENSE_BUFFER, SRB_FLAGS_PORT_DRIVER_ALLOCSENSE, SRB_FLAGS_QUEUE_ACTION_ENABLE, SRB_FUNCTION_STORAGE_REQUEST_BLOCK, SRB_HISTORY, SRB_STATUS_AUTOSENSE_VALID, SRB_STATUS_DATA_OVERRUN, SRB_STATUS_QUEUE_FROZEN, SRB_STATUS_SELECTION_TIMEOUT, _CLASS_PRIVATE_FDO_DATA, _FUNCTIONAL_DEVICE_EXTENSION, _TRANSFER_PACKET};
use ntkrnl_sys::ddk::{PCCHAR, PCSTR};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{SCSI_SENSE_OPTIONS_FIXED_FORMAT_IF_UNKNOWN_FORMAT_INDICATED, STATUS_DEVICE_NOT_READY};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{TRACE_FLAG_GENERAL, TRACE_FLAG_RW};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{SP_UNTAGGED};
use crate::shims::*;
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{ClassInterpretSenseInfo, ClasspConvertToScsiRequestBlock, SetupReadWriteTransferPacket, SubmitTransferPacket};
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{_SCSI_REQUEST_BLOCK};
// resolve_imports.py: explicit kernel/private-type imports
use crate::srblib::{SrbGetSenseInfoBuffer, SrbGetSenseInfoBufferLength};

extern "C" {
    
    fn DbgPrint(format: PCSTR, ...);
    fn __int2c(); // NOT `-> !`: int 0x2c can return (no debugger); a noreturn decl poisons the
                  // shared symbol and folds whole caller functions to `unreachable` (→ ud2 BUGCHECK).
    
    fn SrbSetSrbFlags(srb: PVOID, flags: ULONG);
    fn SrbClearSrbFlags(srb: PVOID, flags: ULONG);
    fn SrbSetRequestTag(srb: PVOID, tag: ULONG);
    fn ClasspIsThinProvisioningError(srb: PSCSI_REQUEST_BLOCK) -> BOOLEAN;
    
    
    fn DbgGetScsiOpStr(srb: PVOID) -> PCCHAR;
    fn SrbGetSrbFlags(srb: PVOID) -> ULONG;
    fn SrbGetSenseInfoBuffer_raw(srb: PVOID) -> PVOID;
    
    
    // Complex classp.h inline functions (FORCEINLINE) - need C shim wrappers at link time.
    fn ClasspTransferPacketGetNumberOfRetriesDone(pkt: PTRANSFER_PACKET, cdb: PCDB, retries_done: *mut ULONG) -> BOOLEAN;
    
    fn ClasspIsOffloadDataTransferCommand(cdb: PCDB) -> BOOLEAN;
    fn ClasspIsTokenOperation(cdb: PCDB) -> BOOLEAN;
    fn IsDescriptorSenseDataFormat(buf: PVOID) -> BOOLEAN;
    fn ScsiGetSenseDescriptor(buf: PVOID, buf_len: UCHAR, desc_buf: *mut PVOID, desc_len: *mut UCHAR) -> BOOLEAN;
    fn ScsiGetNextSenseDescriptorByType(start: PVOID, start_len: UCHAR, types: *mut UCHAR, type_count: ULONG, out_type: *mut UCHAR, out_buf: *mut PVOID, out_len: *mut UCHAR) -> BOOLEAN;
    fn ScsiValidateInformationSenseDescriptor(buf: PVOID, buf_len: u32) -> BOOLEAN;
}

// Inline helpers replacing C FORCEINLINE/macro functions.

/// ClasspTransferPacketGetCdb(Pkt) = SrbGetCdb(Pkt->Srb)
#[inline(always)]
unsafe fn classp_get_cdb(pkt: PTRANSFER_PACKET) -> PCDB {
    SrbGetCdb((*pkt).Srb as PVOID)
}

/// ClasspTransferPacketGetSenseInfoBuffer(Pkt) = SrbGetSenseInfoBuffer(Pkt->Srb)
#[inline(always)]
unsafe fn classp_get_sense_buf(pkt: PTRANSFER_PACKET) -> PVOID {
    crate::srblib::SrbGetSenseInfoBuffer((*pkt).Srb as PVOID)
}

/// ClasspTransferPacketGetSenseInfoBufferLength(Pkt) = SrbGetSenseInfoBufferLength(Pkt->Srb)
#[inline(always)]
unsafe fn classp_get_sense_buf_len(pkt: PTRANSFER_PACKET) -> UCHAR {
    crate::srblib::SrbGetSenseInfoBufferLength((*pkt).Srb as PVOID)
}

/// PORT_ALLOCATED_SENSE_EX(FdoExtension, Srb):
/// TEST_FLAG(SrbGetSrbFlags(Srb), SRB_FLAGS_PORT_DRIVER_ALLOCSENSE)
///   && TEST_FLAG(SrbGetSrbFlags(Srb), SRB_FLAGS_FREE_SENSE_BUFFER)
///   && SrbGetSenseInfoBuffer(Srb) != FdoExtension->SenseData
#[inline(always)]
unsafe fn port_allocated_sense_ex(fdo_extension: PFUNCTIONAL_DEVICE_EXTENSION, srb: PVOID) -> bool {
    let flags = SrbGetSrbFlags(srb);
    (flags & SRB_FLAGS_PORT_DRIVER_ALLOCSENSE) != 0
        && (flags & SRB_FLAGS_FREE_SENSE_BUFFER) != 0
        && crate::srblib::SrbGetSenseInfoBuffer(srb) != (*fdo_extension).SenseData as PVOID
}

/// FREE_PORT_ALLOCATED_SENSE_BUFFER_EX(FdoExtension, Srb):
/// ExFreePool(SrbGetSenseInfoBuffer(Srb));
/// SrbSetSenseInfoBuffer(Srb, FdoExtension->SenseData);
/// SrbSetSenseInfoBufferLength(Srb, GET_FDO_EXTENSION_SENSE_DATA_LENGTH(FdoExtension));
/// SrbClearSrbFlags(Srb, SRB_FLAGS_FREE_SENSE_BUFFER);
#[inline(always)]
unsafe fn free_port_allocated_sense_buffer_ex(fdo_extension: PFUNCTIONAL_DEVICE_EXTENSION, srb: PVOID) {
    ExFreePool(crate::srblib::SrbGetSenseInfoBuffer(srb));
    SrbSetSenseInfoBuffer(srb, (*fdo_extension).SenseData as PVOID);
    // GET_FDO_EXTENSON_SENSE_DATA_LENGTH: use SenseDataLength if non-zero, else SENSE_BUFFER_SIZE
    let sense_len: ULONG = if (*fdo_extension).SenseDataLength > 0 {
        (*fdo_extension).SenseDataLength as ULONG
    } else {
        18u32 // SENSE_BUFFER_SIZE
    };
    SrbSetSenseInfoBufferLength(srb, (sense_len) as _);
    SrbClearSrbFlags(srb, SRB_FLAGS_FREE_SENSE_BUFFER);
}

/// REVERSE_BYTES_QUAD: byte-swap 8 bytes from src into dst.
#[inline(always)]
unsafe fn reverse_bytes_quad(dst: *mut u8, src: *const u8) {
    *dst.add(7) = *src.add(0);
    *dst.add(6) = *src.add(1);
    *dst.add(5) = *src.add(2);
    *dst.add(4) = *src.add(3);
    *dst.add(3) = *src.add(4);
    *dst.add(2) = *src.add(5);
    *dst.add(1) = *src.add(6);
    *dst.add(0) = *src.add(7);
}
#[no_mangle]
pub unsafe extern "C" fn InterpretTransferPacketError(Pkt: PTRANSFER_PACKET) -> BOOLEAN {
    let fdoExtension = (*(*Pkt).Fdo).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let fdoData = (*fdoExtension).PrivateFdoData;
    let mut timesAlreadyRetried: u32 = 0;
    let mut shouldRetry: u8 = 0;
    let pCdb = classp_get_cdb(Pkt);
    (*Pkt).RetryIn100nsUnits = 0 as i64;
    if ClasspTransferPacketGetNumberOfRetriesDone(Pkt, pCdb, &mut timesAlreadyRetried) == 0 {
        TracePrint(
            2u32,
            TRACE_FLAG_GENERAL,
            b"Unhandled SRB Function %xh in error path for packet %p (did miniport change Srb.Cdb.OperationCode ?)\0"
                .as_ptr() as PCCHAR,
            (*pCdb).CDB10.as_ref().OperationCode as u32,
            Pkt,
        );
    }
    if !(*fdoData).InterpretSenseInfo.is_null() {
        let mut tempSrb: _SCSI_REQUEST_BLOCK = ::core::mem::zeroed();
        let mut srbPtr: PSCSI_REQUEST_BLOCK = (*Pkt).Srb as PVOID as PSCSI_REQUEST_BLOCK;
        ntkrnl_sys::ASSERT!(!(*(*fdoData).InterpretSenseInfo).Interpret.is_none());
        if (*((*Pkt).Srb as PVOID as PSCSI_REQUEST_BLOCK)).Function as i32 == (SRB_FUNCTION_STORAGE_REQUEST_BLOCK) as i32
            && ((*(*fdoExtension).__bindgen_anon_1.CommonExtension.as_ref().DriverExtension).SrbSupport
                & CLASS_SRB_STORAGE_REQUEST_BLOCK as u32)
                == 0
        {
            ClasspConvertToScsiRequestBlock(&mut tempSrb, (*Pkt).Srb as PSTORAGE_REQUEST_BLOCK);
            srbPtr = &mut tempSrb as PSCSI_REQUEST_BLOCK;
        }
        shouldRetry = (*(*fdoData).InterpretSenseInfo).Interpret.unwrap()(
            (*Pkt).Fdo,
            (*Pkt).OriginalIrp,
            srbPtr,
            IRP_MJ_SCSI as u8,
            0u32,
            timesAlreadyRetried,
            (*Pkt).RetryHistory as *mut SRB_HISTORY,
            &mut (*(*Pkt).Irp).IoStatus.__bindgen_anon_1.Status,
            &mut (*Pkt).RetryIn100nsUnits,
        );
    } else {
        let senseInfoBuffer = classp_get_sense_buf(Pkt);
        let senseInfoBufferLength = classp_get_sense_buf_len(Pkt);
        let mut validSense: u8 = 0;
        let mut senseKey: u8 = 0;
        let mut additionalSenseCode: u8 = 0;
        let mut additionalSenseQual: u8 = 0;
        ntkrnl_sys::ASSERT!(!senseInfoBuffer.is_null());
        validSense = ScsiGetSenseKeyAndCodes(
            senseInfoBuffer,
            (senseInfoBufferLength) as _,
            SCSI_SENSE_OPTIONS_FIXED_FORMAT_IF_UNKNOWN_FORMAT_INDICATED,
            &mut senseKey,
            &mut additionalSenseCode,
            &mut additionalSenseQual,
        );
        if (*pCdb).MEDIA_REMOVAL.as_ref().OperationCode as u32 == SCSIOP_MEDIUM_REMOVAL {
            let mut retryIntervalSeconds: u32 = 0;
            shouldRetry = ClassInterpretSenseInfo(
                (*Pkt).Fdo,
                (*Pkt).Srb as PSCSI_REQUEST_BLOCK,
                IRP_MJ_SCSI as u8,
                0u32,
                timesAlreadyRetried,
                &mut (*(*Pkt).Irp).IoStatus.__bindgen_anon_1.Status,
                &mut retryIntervalSeconds,
            );
            if shouldRetry != 0 {
                let mut setRetryIntervalSeconds: u8 = 0;
                if validSense != 0 {
                    if (*(*Pkt).Irp).IoStatus.__bindgen_anon_1.Status == STATUS_DEVICE_NOT_READY
                        && additionalSenseCode as i32 == (SCSI_ADSENSE_LUN_NOT_READY) as i32
                    {
                        setRetryIntervalSeconds = 1;
                    }
                }
                if setRetryIntervalSeconds == 0
                    && ((*((*Pkt).Srb as PVOID as PSCSI_REQUEST_BLOCK)).SrbStatus
                        & (!(SRB_STATUS_AUTOSENSE_VALID | SRB_STATUS_QUEUE_FROZEN)) as u8)
                        == (SRB_STATUS_SELECTION_TIMEOUT) as u8
                {
                    setRetryIntervalSeconds = 1;
                }
                if setRetryIntervalSeconds != 0 {
                    retryIntervalSeconds = core::cmp::max(retryIntervalSeconds, 2);
                }
            }
            if shouldRetry != 0 {
                (*Pkt).RetryIn100nsUnits = retryIntervalSeconds as i64;
                (*Pkt).RetryIn100nsUnits *= (1000 * 1000 * 10) as i64;
            }
        } else if (*pCdb).MODE_SENSE.as_ref().OperationCode as u32 == SCSIOP_MODE_SENSE
            || (*pCdb).MODE_SENSE.as_ref().OperationCode as u32 == SCSIOP_MODE_SENSE10
        {
            let mut retryIntervalSeconds_0: u32 = 0;
            shouldRetry = ClassInterpretSenseInfo(
                (*Pkt).Fdo,
                (*Pkt).Srb as PSCSI_REQUEST_BLOCK,
                IRP_MJ_SCSI as u8,
                0u32,
                timesAlreadyRetried,
                &mut (*(*Pkt).Irp).IoStatus.__bindgen_anon_1.Status,
                &mut retryIntervalSeconds_0,
            );
            if shouldRetry != 0 {
                let mut setRetryIntervalSeconds_0: u8 = 0;
                if validSense != 0 {
                    if (*(*Pkt).Irp).IoStatus.__bindgen_anon_1.Status == STATUS_DEVICE_NOT_READY
                        && additionalSenseCode as i32 == (SCSI_ADSENSE_LUN_NOT_READY) as i32
                    {
                        setRetryIntervalSeconds_0 = 1;
                    }
                }
                if setRetryIntervalSeconds_0 == 0
                    && ((*((*Pkt).Srb as PVOID as PSCSI_REQUEST_BLOCK)).SrbStatus
                        & (!(SRB_STATUS_AUTOSENSE_VALID | SRB_STATUS_QUEUE_FROZEN)) as u8)
                        == (SRB_STATUS_SELECTION_TIMEOUT) as u8
                {
                    setRetryIntervalSeconds_0 = 1;
                }
                if setRetryIntervalSeconds_0 != 0 {
                    retryIntervalSeconds_0 = core::cmp::max(retryIntervalSeconds_0, 2);
                }
            }
            if (*(*Pkt).Irp).IoStatus.__bindgen_anon_1.Status == STATUS_VERIFY_REQUIRED {
                shouldRetry = 1;
            } else if ((*((*Pkt).Srb as PVOID as PSCSI_REQUEST_BLOCK)).SrbStatus
                & (!(SRB_STATUS_AUTOSENSE_VALID | SRB_STATUS_QUEUE_FROZEN)) as u8)
                == (SRB_STATUS_DATA_OVERRUN) as u8
            {
                (*(*Pkt).Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
                shouldRetry = 0;
            }
            if shouldRetry != 0 {
                (*Pkt).RetryIn100nsUnits = retryIntervalSeconds_0 as i64;
                (*Pkt).RetryIn100nsUnits *= (1000 * 1000 * 10) as i64;
            }
        } else if (*pCdb).CDB10.as_ref().OperationCode as u32 == SCSIOP_READ_CAPACITY
            || (*pCdb).CDB16.as_ref().OperationCode as u32 == SCSIOP_READ_CAPACITY16
        {
            let mut retryIntervalSeconds_1: u32 = 0;
            shouldRetry = ClassInterpretSenseInfo(
                (*Pkt).Fdo,
                (*Pkt).Srb as PSCSI_REQUEST_BLOCK,
                IRP_MJ_SCSI as u8,
                0u32,
                timesAlreadyRetried,
                &mut (*(*Pkt).Irp).IoStatus.__bindgen_anon_1.Status,
                &mut retryIntervalSeconds_1,
            );
            if (*(*Pkt).Irp).IoStatus.__bindgen_anon_1.Status == STATUS_VERIFY_REQUIRED {
                shouldRetry = 1;
            }
            if shouldRetry != 0 {
                (*Pkt).RetryIn100nsUnits = retryIntervalSeconds_1 as i64;
                (*Pkt).RetryIn100nsUnits *= (1000 * 1000 * 10) as i64;
            }
        } else if (*pCdb).CDB10.as_ref().OperationCode as u32 == SCSIOP_READ6
            || (*pCdb).CDB10.as_ref().OperationCode as u32 == SCSIOP_READ
            || (*pCdb).CDB10.as_ref().OperationCode as u32 == SCSIOP_READ12
            || (*pCdb).CDB10.as_ref().OperationCode as u32 == SCSIOP_READ16
            || (*pCdb).CDB10.as_ref().OperationCode as u32 == SCSIOP_WRITE6
            || (*pCdb).CDB10.as_ref().OperationCode as u32 == SCSIOP_WRITE
            || (*pCdb).CDB10.as_ref().OperationCode as u32 == SCSIOP_WRITE12
            || (*pCdb).CDB10.as_ref().OperationCode as u32 == SCSIOP_WRITE16
        {
            let mut retryIntervalSeconds_2: u32 = 0;
            let origCurrentSp = IoGetCurrentIrpStackLocation((*Pkt).OriginalIrp);
            shouldRetry = ClassInterpretSenseInfo(
                (*Pkt).Fdo,
                (*Pkt).Srb as PSCSI_REQUEST_BLOCK,
                (*origCurrentSp).MajorFunction,
                0u32,
                timesAlreadyRetried,
                &mut (*(*Pkt).Irp).IoStatus.__bindgen_anon_1.Status,
                &mut retryIntervalSeconds_2,
            );
            if (*(*Pkt).Irp).IoStatus.__bindgen_anon_1.Status == STATUS_INSUFFICIENT_RESOURCES {
                shouldRetry = 1;
            } else if TEST_FLAG(((*origCurrentSp).Flags) as u32, SL_OVERRIDE_VERIFY_VOLUME) != 0
                && (*(*Pkt).Irp).IoStatus.__bindgen_anon_1.Status == STATUS_VERIFY_REQUIRED
            {
                (*(*Pkt).Irp).IoStatus.__bindgen_anon_1.Status = STATUS_IO_DEVICE_ERROR;
                shouldRetry = 1;
            }
            if shouldRetry != 0 {
                (*Pkt).RetryIn100nsUnits = retryIntervalSeconds_2 as i64;
                (*Pkt).RetryIn100nsUnits *= (1000 * 1000 * 10) as i64;
            }
        } else if ClasspIsOffloadDataTransferCommand(pCdb) != 0 {
            let mut retryIntervalSeconds_3: u32 = 0;
            (*Pkt).TransferCount = 0u64;
            shouldRetry = ClassInterpretSenseInfo(
                (*Pkt).Fdo,
                (*Pkt).Srb as PSCSI_REQUEST_BLOCK,
                IRP_MJ_SCSI as u8,
                0u32,
                timesAlreadyRetried,
                &mut (*(*Pkt).Irp).IoStatus.__bindgen_anon_1.Status,
                &mut retryIntervalSeconds_3,
            );
            if shouldRetry != 0 {
                (*Pkt).RetryIn100nsUnits = retryIntervalSeconds_3 as i64;
                (*Pkt).RetryIn100nsUnits *= (1000 * 1000 * 10) as i64;
            } else if ClasspIsTokenOperation(pCdb) != 0 {
                let mut isInformationValid: u8 = 0;
                let mut information: u64 = 0;
                if validSense != 0 {
                    if (senseKey as i32 == (SCSI_SENSE_COPY_ABORTED) as i32
                        || senseKey as i32 == (SCSI_SENSE_ABORTED_COMMAND) as i32)
                        && (additionalSenseCode as i32 == (SCSI_ADSENSE_COPY_TARGET_DEVICE_ERROR) as i32
                            && additionalSenseQual as i32 == (SCSI_SENSEQ_DATA_UNDERRUN) as i32)
                    {
                        if IsDescriptorSenseDataFormat(senseInfoBuffer) != 0 {
                            let mut startBuffer: PVOID = core::ptr::null_mut();
                            let mut startBufferLength: u8 = 0;
                            if ScsiGetSenseDescriptor(
                                senseInfoBuffer,
                                SrbGetSenseInfoBufferLength(((*Pkt).Srb).cast()),
                                &mut startBuffer,
                                &mut startBufferLength,
                            ) != 0
                            {
                                let mut outType: u8 = 0;
                                let mut outBuffer: PVOID = core::ptr::null_mut();
                                let mut outBufferLength: u8 = 0;
                                let mut typeList: [u8; 1] =
                                    [SCSI_SENSE_DESCRIPTOR_TYPE_INFORMATION as u8];
                                if ScsiGetNextSenseDescriptorByType(
                                    startBuffer,
                                    startBufferLength,
                                    typeList.as_mut_ptr(),
                                    typeList.len() as u32,
                                    &mut outType,
                                    &mut outBuffer,
                                    &mut outBufferLength,
                                ) != 0
                                {
                                    if outType as i32 == (SCSI_SENSE_DESCRIPTOR_TYPE_INFORMATION) as i32 {
                                        if ScsiValidateInformationSenseDescriptor(
                                            outBuffer,
                                            (outBufferLength) as _,
                                        ) != 0
                                        {
                                            reverse_bytes_quad(
                                                &mut information as *mut u64 as *mut u8,
                                                (*(outBuffer as PSCSI_SENSE_DESCRIPTOR_INFORMATION))
                                                    .Information.as_ptr(),
                                            );
                                            isInformationValid = 1;
                                        }
                                    } else {
                                        ntkrnl_sys::ASSERT!(false);
                                    }
                                }
                            }
                        } else {
                            REVERSE_BYTES(
                                &mut information as *mut u64 as *mut u32,
                                (*(senseInfoBuffer as PFIXED_SENSE_DATA)).Information.as_ptr() as *const u32,
                            );
                            isInformationValid = 1;
                        }
                        if isInformationValid != 0 {
                            (*Pkt).TransferCount = information;
                        }
                    }
                }
            }
        } else {
            TracePrint(
                2,
                TRACE_FLAG_GENERAL,
                b"Unhandled SRB Function %xh in error path for packet %p (did miniport change Srb.Cdb.OperationCode ?)\0"
                    .as_ptr() as PCCHAR,
                (*pCdb).CDB10.as_ref().OperationCode as u32,
                Pkt,
            );
        }
    }
    shouldRetry
}
#[no_mangle]
pub unsafe extern "C" fn RetryTransferPacket(Pkt: PTRANSFER_PACKET) -> BOOLEAN {

    let mut packetDone: BOOLEAN;
    let mut scaleDown: BOOLEAN = 0u8;
    let fdoExtension: *mut _FUNCTIONAL_DEVICE_EXTENSION =
        (*(*Pkt).Fdo).DeviceExtension as *mut _FUNCTIONAL_DEVICE_EXTENSION;
    let fdoData: *mut _CLASS_PRIVATE_FDO_DATA = (*fdoExtension).PrivateFdoData;
    let mut pCdb: PCDB = SrbGetCdb(((*Pkt).Srb).cast());

    if !((*Pkt).NumRetries > 0 || !(*Pkt).RetryHistory.is_null()) {
        DbgPrint(
            b"Assertion failed at %s(%d): %s\n\0".as_ptr() as PCSTR,
            b"drivers/storage/class/classpnp/retry.c\0".as_ptr()
                as PCSTR,
            476i32,
            b"Pkt->NumRetries > 0 || Pkt->RetryHistory\0".as_ptr() as PCSTR,
        );
        __int2c();
    }
    (*Pkt).NumRetries = (*Pkt).NumRetries.wrapping_sub(1);

    TracePrint(
        TRACE_LEVEL_INFORMATION,
        TRACE_FLAG_GENERAL,
        b"retrying failed transfer (pkt=%ph, op=%s)\0".as_ptr() as PCCHAR,
        Pkt,
        DbgGetScsiOpStr(((*Pkt).Srb).cast()),
    );

    if (*fdoData).DisableThrottling == 0 {
        if (*Pkt).NumRetries == 0
            && ClasspIsThinProvisioningError((*Pkt).Srb as PSCSI_REQUEST_BLOCK) == 0u8
        {
            scaleDown = 1u8;
        }
    }

    if scaleDown != 0 {
        SrbSetSrbFlags((*Pkt).Srb as PVOID, SRB_FLAGS_DISABLE_DISCONNECT);
        SrbSetSrbFlags((*Pkt).Srb as PVOID, SRB_FLAGS_DISABLE_SYNCH_TRANSFER);
        SrbClearSrbFlags((*Pkt).Srb as PVOID, SRB_FLAGS_QUEUE_ACTION_ENABLE);
        SrbSetRequestTag((*Pkt).Srb as PVOID, SP_UNTAGGED);
    }

    if (*(*Pkt).Irp).IoStatus.__bindgen_anon_1.Status == STATUS_INSUFFICIENT_RESOURCES {
        let mut cdbOpcode: UCHAR = 0;
        let mut isReadWrite: BOOLEAN = 0u8;
        if !pCdb.is_null() {
            cdbOpcode = (*pCdb).CDB10.as_ref().OperationCode;
            isReadWrite = (cdbOpcode == (SCSIOP_READ6) as u8
                || cdbOpcode == (SCSIOP_READ) as u8
                || cdbOpcode == (SCSIOP_READ12) as u8
                || cdbOpcode == (SCSIOP_READ16) as u8
                || cdbOpcode == (SCSIOP_WRITE6) as u8
                || cdbOpcode == (SCSIOP_WRITE) as u8
                || cdbOpcode == (SCSIOP_WRITE12) as u8
                || cdbOpcode == (SCSIOP_WRITE16) as u8) as BOOLEAN;
        }
        if (*Pkt).DriverUsesStartIO != 0
            && (cdbOpcode == (SCSIOP_WRITE6) as u8
                || cdbOpcode == (SCSIOP_WRITE) as u8
                || cdbOpcode == (SCSIOP_WRITE12) as u8
                || cdbOpcode == (SCSIOP_WRITE16) as u8)
        {
            (*fdoData).TrackingFlags |= 0x1;
            packetDone = 1u8;
        } else if (*Pkt).InLowMemRetry != 0 || isReadWrite == 0u8 {
            (*fdoData).TrackingFlags |= 0x2;
            packetDone = 1u8;
        } else {
            (*fdoData).TrackingFlags |= 0x4;
            InitLowMemRetry(
                Pkt,
                (*Pkt).BufPtrCopy as PVOID,
                (*Pkt).BufLenCopy,
                (*Pkt).TargetLocationCopy,
            );
            StepLowMemRetry(Pkt);
            packetDone = 0u8;
        }
    } else {
        SrbSetDataBuffer((*Pkt).Srb as PVOID, (*Pkt).BufPtrCopy as PVOID);
        SrbSetDataTransferLength((*Pkt).Srb as PVOID, (*Pkt).BufLenCopy);
        TransferPacketQueueRetryDpc(Pkt);
        packetDone = 0u8;
    }

    packetDone
}
#[no_mangle]
pub unsafe extern "C" fn TransferPacketQueueRetryDpc(Pkt: PTRANSFER_PACKET) {
    KeInitializeDpc(
        core::ptr::addr_of_mut!((*Pkt).RetryTimerDPC),
        Some(core::mem::transmute::<unsafe extern "system" fn(PKDPC, PVOID, PVOID, PVOID), _>(TransferPacketRetryTimerDpc)),
        Pkt as PVOID,
    );
    if (*Pkt).RetryIn100nsUnits == 0i64 {
        KeInsertQueueDpc(
            core::ptr::addr_of_mut!((*Pkt).RetryTimerDPC),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
    } else {
        let mut timerPeriod: LARGE_INTEGER = core::mem::zeroed();
        if !((*Pkt).RetryIn100nsUnits < (100i64 * 1000 * 1000 * 10)) {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0".as_ptr() as PCSTR,
                b"drivers/storage/class/classpnp/retry.c\0".as_ptr() as *const ::core::ffi::c_char,
                614i32,
                b"Pkt->RetryIn100nsUnits < 100 * 1000 * 1000 * 10\0".as_ptr() as *const ::core::ffi::c_char,
            );
            __int2c();
        }
        timerPeriod.QuadPart = -(*Pkt).RetryIn100nsUnits;
        KeInitializeTimer(core::ptr::addr_of_mut!((*Pkt).RetryTimer));
        KeSetTimer(
            core::ptr::addr_of_mut!((*Pkt).RetryTimer),
            timerPeriod,
            core::ptr::addr_of_mut!((*Pkt).RetryTimerDPC),
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn TransferPacketRetryTimerDpc(
    Dpc: PKDPC,
    DeferredContext: PVOID,
    SystemArgument1: PVOID,
    SystemArgument2: PVOID,
) {
    let pkt: PTRANSFER_PACKET = DeferredContext as PTRANSFER_PACKET;
    let fdo: PDEVICE_OBJECT = (*pkt).Fdo;
    let fdoExtension: PFUNCTIONAL_DEVICE_EXTENSION = (*fdo).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;

    if port_allocated_sense_ex(fdoExtension, (*pkt).Srb as PVOID) {
        TracePrint(
            TRACE_LEVEL_INFORMATION,
            TRACE_FLAG_RW,
            b"Freeing port-allocated sense buffer for pkt %ph.\0".as_ptr() as PCHAR,
            pkt,
        );
        free_port_allocated_sense_buffer_ex(fdoExtension, (*pkt).Srb as PVOID);
        SrbSetSenseInfoBuffer(
            (*pkt).Srb as PVOID,
            core::ptr::addr_of_mut!((*pkt).SrbErrorSenseData) as PVOID,
        );
        SrbSetSenseInfoBufferLength(
            (*pkt).Srb as PVOID,
            ((core::mem::size_of::<SENSE_DATA_EX>() as UCHAR) as u32) as _,
        );
    } else {
        NT_ASSERT(
            (SrbGetSenseInfoBuffer((*pkt).Srb as PVOID) as *mut core::ffi::c_void
                == core::ptr::addr_of_mut!((*pkt).SrbErrorSenseData) as *mut core::ffi::c_void) as u8,
        );
        NT_ASSERT(
            (SrbGetSenseInfoBufferLength((*pkt).Srb as PVOID) as usize
                <= core::mem::size_of::<SENSE_DATA_EX>()) as u8,
        );
    }

    core::ptr::write_bytes(
        core::ptr::addr_of_mut!((*pkt).SrbErrorSenseData) as *mut u8,
        0,
        core::mem::size_of::<SENSE_DATA_EX>(),
    );

    SubmitTransferPacket(pkt);
}
#[no_mangle]
pub unsafe extern "C" fn InitLowMemRetry(
    Pkt: *mut _TRANSFER_PACKET,
    BufPtr: *mut core::ffi::c_void,
    Len: u32,
    TargetLocation: LARGE_INTEGER,
) {
    NT_ASSERT(if Len > 0 { 1u8 } else { 0u8 });
    NT_ASSERT(if (*Pkt).InLowMemRetry == 0 { 1u8 } else { 0u8 });
    if (*Pkt).DriverUsesStartIO != 0 {
        (*Pkt).InLowMemRetry = 0u8;
    } else {
        (*Pkt).InLowMemRetry = 1u8;
    }
    (*Pkt).LowMemRetry_remainingBufPtr = BufPtr as *mut u8;
    (*Pkt).LowMemRetry_remainingBufLen = Len;
    (*Pkt).LowMemRetry_nextChunkTargetLocation = TargetLocation;
}
#[no_mangle]
pub unsafe extern "C" fn StepLowMemRetry(Pkt: PTRANSFER_PACKET) -> BOOLEAN {
    let mut packetDone: BOOLEAN = 0u8;
    if (*Pkt).LowMemRetry_remainingBufLen == 0u32 {
        packetDone = 1u8;
    } else {
        let mut thisChunkLen: ULONG;
        if (*Pkt).DriverUsesStartIO != 0u8 {
            let fdoExt: PFUNCTIONAL_DEVICE_EXTENSION = (*(*Pkt).Fdo).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
            let fdoData: PCLASS_PRIVATE_FDO_DATA = (*fdoExt).PrivateFdoData;
            let commonExtension: PCOMMON_DEVICE_EXTENSION = (*(*Pkt).Fdo).DeviceExtension as PCOMMON_DEVICE_EXTENSION;
            let adapterDesc: PSTORAGE_ADAPTER_DESCRIPTOR = (*(*commonExtension).PartitionZeroExtension).AdapterDescriptor;

            if (*Pkt).InLowMemRetry != 0u8 {
                DbgPrint(
                    b"Assertion failed at %s(%d): %s\n\0".as_ptr() as PCSTR,
                    b"drivers/storage/class/classpnp/retry.c\0".as_ptr() as PCSTR,
                    740i32,
                    b"!Pkt->InLowMemRetry\0".as_ptr() as PCSTR,
                );
                __int2c();
            }

            let hwMaxXferLen: ULONG;
            if ((*Pkt).LowMemRetry_remainingBufPtr as u64 & ((PAGE_SIZE - 1) as u64)) != 0
                || (*fdoData).HwMaxXferLen > (0xFFFFFFFFu32.wrapping_sub((PAGE_SIZE) as u32))
            {
                hwMaxXferLen = (*fdoData).HwMaxXferLen;
            } else {
                if (PAGE_SIZE % ((*fdoExt).DiskGeometry.BytesPerSector) as usize) != 0 {
                    DbgPrint(
                        b"Assertion failed at %s(%d): %s\n\0".as_ptr() as PCSTR,
                        b"drivers/storage/class/classpnp/retry.c\0".as_ptr() as PCSTR,
                        754i32,
                        b"(PAGE_SIZE%fdoExt->DiskGeometry.BytesPerSector) == 0\0".as_ptr() as PCSTR,
                    );
                    __int2c();
                }
                let sum: ULONG = (*fdoData).HwMaxXferLen.wrapping_add((PAGE_SIZE) as u32);
                hwMaxXferLen = if sum < (*adapterDesc).MaximumTransferLength {
                    sum
                } else {
                    (*adapterDesc).MaximumTransferLength
                };
            }

            thisChunkLen = if (*Pkt).LowMemRetry_remainingBufLen < hwMaxXferLen {
                (*Pkt).LowMemRetry_remainingBufLen
            } else {
                hwMaxXferLen
            };
        } else {
            let bytesToNextPageBoundary: ULONG = PAGE_SIZE.wrapping_sub(
                (((*Pkt).LowMemRetry_remainingBufPtr as u64).wrapping_rem(PAGE_SIZE as u64) as ULONG) as usize,
  ) as u32 ;
            thisChunkLen = if (*Pkt).LowMemRetry_remainingBufLen < bytesToNextPageBoundary {
                (*Pkt).LowMemRetry_remainingBufLen
            } else {
                bytesToNextPageBoundary
            };

            if (*Pkt).InLowMemRetry == 0u8 {
                DbgPrint(
                    b"Assertion failed at %s(%d): %s\n\0".as_ptr() as PCSTR,
                    b"drivers/storage/class/classpnp/retry.c\0".as_ptr() as PCSTR,
                    767i32,
                    b"Pkt->InLowMemRetry\0".as_ptr() as PCSTR,
                );
                __int2c();
            }
        }

        SetupReadWriteTransferPacket(
            Pkt,
            (*Pkt).LowMemRetry_remainingBufPtr as PVOID,
            thisChunkLen,
            (*Pkt).LowMemRetry_nextChunkTargetLocation,
            (*Pkt).OriginalIrp,
        );

        (*Pkt).LowMemRetry_remainingBufPtr = (*Pkt).LowMemRetry_remainingBufPtr.offset(thisChunkLen as isize);
        (*Pkt).LowMemRetry_remainingBufLen = (*Pkt).LowMemRetry_remainingBufLen.wrapping_sub(thisChunkLen);
        (*Pkt).LowMemRetry_nextChunkTargetLocation.QuadPart += thisChunkLen as i64;
        (*Pkt).UsePartialMdl = 1u8;
        TransferPacketQueueRetryDpc(Pkt);
        packetDone = 0u8;
    }
    packetDone
}



// ---- ABI-corrected extern decls (CC matched to in-crate definition) ----
extern "C" {
    
}
extern "system" {
    fn NT_ASSERT(cond: u8);
    fn SrbGetCdb(srb: PVOID) -> PCDB;
    fn SrbSetDataBuffer(srb: PVOID, buf: PVOID);
    fn SrbSetDataTransferLength(srb: PVOID, len: ULONG);
    fn SrbSetSenseInfoBuffer(srb: PVOID, buf: PVOID);
    fn SrbSetSenseInfoBufferLength(srb: PVOID, len: u8);
    fn ScsiGetSenseKeyAndCodes(buf: PVOID, buf_len: u32, opts: u32, key: *mut UCHAR, asc: *mut UCHAR, ascq: *mut UCHAR) -> BOOLEAN;
}


// ---- export decls forced to stdcall (real kernel export CC) ----
extern "system" {
    pub fn ExFreePool(p: PVOID);
}
