use crate::types::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{IoGetCurrentIrpStackLocation, KeInitializeDpc, KeInitializeTimer, KeInsertQueueDpc, KeSetTimer, LARGE_INTEGER, PDEVICE_OBJECT, PIO_STACK_LOCATION, PSTORAGE_ADAPTER_DESCRIPTOR};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{_CDB as CDB};

extern "C" {
    fn NT_ASSERT(cond: i32)
}
extern "C" {
    fn TracePrint(level: u32, flag: u32, format: *const u8, ...);
    fn ClasspTransferPacketGetCdb(Pkt: *mut TRANSFER_PACKET) -> *mut CDB;
    fn ClasspTransferPacketGetNumberOfRetriesDone(Pkt: *mut TRANSFER_PACKET, pCdb: *mut CDB, timesAlreadyRetried: *mut u32) -> i32;
    fn ScsiGetSenseKeyAndCodes(buffer: *mut core::ffi::c_void, bufferLength: u8, options: u32, senseKey: *mut u8, asc: *mut u8, ascq: *mut u8) -> i32;
    fn ClassInterpretSenseInfo(Fdo: *mut FUNCTIONAL_DEVICE_OBJECT, Srb: *mut SCSI_REQUEST_BLOCK, MajorFunction: u32, RetryCount: u32, timesAlreadyRetried: u32, status: *mut i32, retryIntervalSeconds: *mut u32) -> i32;
    fn SRB_STATUS(srbStatus: u8) -> u32;
    fn MAX(a: u32, b: u32) -> u32;
    fn ClasspSrbTimeOutStatus(Srb: *mut STORAGE_REQUEST_BLOCK) -> i32;
    fn IS_SCSIOP_READWRITE(opcode: u8) -> i32;
    fn TEST_FLAG(flags: u32, bit: u32) -> i32;
    fn ClasspIsOffloadDataTransferCommand(pCdb: *mut CDB) -> i32;
    fn ClasspIsTokenOperation(pCdb: *mut CDB) -> i32;
    fn IsDescriptorSenseDataFormat(senseBuffer: *mut core::ffi::c_void) -> i32;
    fn ScsiGetSenseDescriptor(senseBuffer: *mut core::ffi::c_void, bufferLength: u8, startBuffer: *mut *mut core::ffi::c_void, startBufferLength: *mut u8) -> i32;
    fn SrbGetSenseInfoBufferLength(Srb: *mut STORAGE_REQUEST_BLOCK) -> u8;
    fn ScsiGetNextSenseDescriptorByType(startBuffer: *mut core::ffi::c_void, startBufferLength: u8, typeList: *const u8, typeListLength: u32, outType: *mut u8, outBuffer: *mut *mut core::ffi::c_void, outBufferLength: *mut u8) -> i32;
    fn ScsiValidateInformationSenseDescriptor(buf: *mut core::ffi::c_void, len: u8) -> i32;
    fn REVERSE_BYTES_QUAD(dst: *mut u64, src: *const u8);
    fn REVERSE_BYTES(dst: *mut u32, src: *const u8);
    fn ClasspConvertToScsiRequestBlock(tempSrb: *mut SCSI_REQUEST_BLOCK, srb: *mut STORAGE_REQUEST_BLOCK);
    fn ClasspTransferPacketGetSenseInfoBuffer(Pkt: *mut TRANSFER_PACKET) -> *mut core::ffi::c_void;
    fn ClasspTransferPacketGetSenseInfoBufferLength(Pkt: *mut TRANSFER_PACKET) -> u8;
    fn _Analysis_assume_(b: i32);
    fn PORT_ALLOCATED_SENSE_EX(extension: *mut FUNCTIONAL_DEVICE_EXTENSION, srb: *mut core::ffi::c_void) -> i32;
    fn FREE_PORT_ALLOCATED_SENSE_BUFFER_EX(extension: *mut FUNCTIONAL_DEVICE_EXTENSION, srb: *mut core::ffi::c_void);
    fn SrbGetSenseInfoBuffer(srb: *mut core::ffi::c_void) -> *mut core::ffi::c_void;
    fn SrbSetSenseInfoBuffer(srb: *mut core::ffi::c_void, buffer: *mut core::ffi::c_void);
    fn SrbSetSenseInfoBufferLength(srb: *mut core::ffi::c_void, length: u32);
    fn SubmitTransferPacket(pkt: *mut TRANSFER_PACKET);
    fn min(a: u32, b: u32) -> u32;
    fn MIN(a: u32, b: u32) -> u32;
    fn SetupReadWriteTransferPacket(pkt: *mut TRANSFER_PACKET, buf: *mut u8, len: u32, target: LARGE_INTEGER, irp: *mut core::ffi::c_void);
    fn ClasspIsThinProvisioningError(Srb: *mut SCSI_REQUEST_BLOCK) -> i32;
    fn DBGGETSCSIOPSTR(Srb: *mut SCSI_REQUEST_BLOCK) -> *const i8;
    fn SrbClearSrbFlags(Srb: *mut SCSI_REQUEST_BLOCK, Flags: u32);
    fn SrbGetCdb(Srb: *mut SCSI_REQUEST_BLOCK) -> *mut CDB;
    fn SrbSetDataBuffer(Srb: *mut SCSI_REQUEST_BLOCK, BufPtr: *mut core::ffi::c_void);
    fn SrbSetDataTransferLength(Srb: *mut SCSI_REQUEST_BLOCK, Length: u32);
    fn SrbSetRequestTag(Srb: *mut SCSI_REQUEST_BLOCK, Tag: u8);
    fn SrbSetSrbFlags(Srb: *mut SCSI_REQUEST_BLOCK, Flags: u32);
}

// Extern declarations for functions used by this function (not in provided imports)
#[allow(unused)]

// Constants used (values taken from standard Windows headers where possible, otherwise placeholders)
const SCSIOP_MEDIUM_REMOVAL: u8 = 0x1E;
const SCSIOP_MODE_SENSE: u8 = 0x1A;
const SCSIOP_MODE_SENSE10: u8 = 0x5A;
const SCSIOP_READ_CAPACITY: u8 = 0x25;
const SCSIOP_READ_CAPACITY16: u8 = 0x9E;
const SCSI_SENSE_OPTIONS_FIXED_FORMAT_IF_UNKNOWN_FORMAT_INDICATED: u32 = 0x00000001;
const SCSI_ADSENSE_LUN_NOT_READY: u8 = 0x04;
const SCSI_ADSENSE_COPY_TARGET_DEVICE_ERROR: u8 = 0x00;  // TODO: actual value
const SCSI_SENSEQ_DATA_UNDERRUN: u8 = 0x00;
const SCSI_SENSE_COPY_ABORTED: u8 = 0x0A;  // placeholder
const SCSI_SENSE_ABORTED_COMMAND: u8 = 0x0B;
const SCSI_SENSE_DESCRIPTOR_TYPE_INFORMATION: u8 = 0x00;
const SRB_FUNCTION_STORAGE_REQUEST_BLOCK: u32 = 0x19;
const CLASS_SRB_STORAGE_REQUEST_BLOCK: u32 = 0x100;
const STATUS_DEVICE_NOT_READY: i32 = 0xC00000A2u32 as i32;
const STATUS_VERIFY_REQUIRED: i32 = 0xC0000187u32 as i32;
const STATUS_INSUFFICIENT_RESOURCES: i32 = 0xC000009Au32 as i32;
const STATUS_IO_DEVICE_ERROR: i32 = 0xC0000185u32 as i32;
const STATUS_SUCCESS: i32 = 0;

// Note: SL_OVERRIDE_VERIFY_VOLUME is a bit flag within SL_FLAGS (actual value unknown, used with TEST_FLAG)
const SL_OVERRIDE_VERIFY_VOLUME: u32 = 0x08;  // placeholder

const NUM_THIN_PROVISIONING_RETRIES: u32 = 3;
const TRACE_LEVEL_INFORMATION: u32 = 4;
const TRACE_FLAG_GENERAL: u32 = 0x00000010;
const SRB_FLAGS_DISABLE_DISCONNECT: u32 = 0x00000004;
const SRB_FLAGS_DISABLE_SYNCH_TRANSFER: u32 = 0x00000008;
const SRB_FLAGS_QUEUE_ACTION_ENABLE: u32 = 0x00000002;
const SP_UNTAGGED: u8 = 0;
const TRACKING_FORWARD_PROGRESS_PATH1: u32 = 0x00000001;
const TRACKING_FORWARD_PROGRESS_PATH2: u32 = 0x00000002;
const TRACKING_FORWARD_PROGRESS_PATH3: u32 = 0x00000004;
const SCSIOP_WRITE6: u8 = 0x0A;
const SCSIOP_WRITE: u8 = 0x2A;
const SCSIOP_WRITE12: u8 = 0xAA;
const SCSIOP_WRITE16: u8 = 0x8A;
#[no_mangle]
pub unsafe extern "C" fn InterpretTransferPacketError(Pkt: *mut TRANSFER_PACKET) -> u8 {
    // Mechanical transliteration – statement-for-statement, raw pointers, unsafe block
    let fdoExtension = (*(*Pkt).Fdo).DeviceExtension as *mut FUNCTIONAL_DEVICE_EXTENSION;
    let fdoData = (*fdoExtension).PrivateFdoData as *mut CLASS_PRIVATE_FDO_DATA;
    let mut shouldRetry: u8 = 0;
    let pCdb = ClasspTransferPacketGetCdb(Pkt);

    (*Pkt).RetryIn100nsUnits = 0;

    let mut timesAlreadyRetried: u32 = 0;
    if ClasspTransferPacketGetNumberOfRetriesDone(Pkt, pCdb, core::ptr::addr_of_mut!(timesAlreadyRetried)) == 0 {
        TracePrint(
            0x00000001,  // TRACE_LEVEL_ERROR
            0x00000001,  // TRACE_FLAG_GENERAL
            b"Unhandled SRB Function %xh in error path for packet %p (did miniport change Srb.Cdb.OperationCode ?)\0".as_ptr() as *const i8,
            (*pCdb).CDB10.OperationCode as u32,
            Pkt,
        );
    }

    if !(*fdoData).InterpretSenseInfo.is_null() {
        let mut tempSrb: SCSI_REQUEST_BLOCK = core::mem::zeroed();
        let mut srbPtr = (*Pkt).Srb as *mut SCSI_REQUEST_BLOCK;

        NT_ASSERT(!((*(*(*fdoData).InterpretSenseInfo)).Interpret).is_null() as i32)

        if (u32::from((*(*Pkt).Srb).Function) == SRB_FUNCTION_STORAGE_REQUEST_BLOCK)
            && ((*(*(*fdoExtension).CommonExtension.DriverExtension)).SrbSupport & CLASS_SRB_STORAGE_REQUEST_BLOCK) == 0
        {
            ClasspConvertToScsiRequestBlock(
                &raw mut tempSrb as *mut SCSI_REQUEST_BLOCK,
                (*Pkt).Srb as *mut STORAGE_REQUEST_BLOCK,
            );
            srbPtr = &raw mut tempSrb as *mut SCSI_REQUEST_BLOCK;
        }

        shouldRetry = (*(*fdoData).InterpretSenseInfo).Interpret(
            (*Pkt).Fdo,
            (*Pkt).OriginalIrp,
            srbPtr,
            0,          // IRP_MJ_SCSI
            0,
            timesAlreadyRetried,
            (*Pkt).RetryHistory,
            core::ptr::addr_of_mut!((*(*Pkt).Irp).IoStatus.__bindgen_anon_1.Status),
            core::ptr::addr_of_mut!((*Pkt).RetryIn100nsUnits),
        ) as u8;
    } else {
        let senseInfoBuffer = ClasspTransferPacketGetSenseInfoBuffer(Pkt);
        let senseInfoBufferLength = ClasspTransferPacketGetSenseInfoBufferLength(Pkt);
        let mut validSense: u8 = 0;
        let mut senseKey: u8 = 0;
        let mut additionalSenseCode: u8 = 0;
        let mut additionalSenseQual: u8 = 0;

        NT_ASSERT(!senseInfoBuffer.is_null() as i32);

        validSense = ScsiGetSenseKeyAndCodes(
            senseInfoBuffer,
            senseInfoBufferLength,
            SCSI_SENSE_OPTIONS_FIXED_FORMAT_IF_UNKNOWN_FORMAT_INDICATED,
            core::ptr::addr_of_mut!(senseKey),
            core::ptr::addr_of_mut!(additionalSenseCode),
            core::ptr::addr_of_mut!(additionalSenseQual),
        ) as u8;

        if (*pCdb).MEDIA_REMOVAL.OperationCode == SCSIOP_MEDIUM_REMOVAL {
            let mut retryIntervalSeconds: u32 = 0;

            shouldRetry = ClassInterpretSenseInfo(
                (*Pkt).Fdo,
                (*Pkt).Srb as *mut SCSI_REQUEST_BLOCK,
                0,
                0,
                timesAlreadyRetried,
                core::ptr::addr_of_mut!((*(*Pkt).Irp).IoStatus.__bindgen_anon_1.Status),
                core::ptr::addr_of_mut!(retryIntervalSeconds),
            ) as u8;

            if shouldRetry != 0 {
                let mut setRetryIntervalSeconds: u8 = 0;

                if validSense != 0 {
                    if (*(*Pkt).Irp).IoStatus.__bindgen_anon_1.Status == STATUS_DEVICE_NOT_READY
                        && additionalSenseCode == SCSI_ADSENSE_LUN_NOT_READY
                    {
                        setRetryIntervalSeconds = 1;
                    }
                }

                if setRetryIntervalSeconds == 0 && SRB_STATUS((*(*Pkt).Srb).SrbStatus) == 0x01 /*SRB_STATUS_SELECTION_TIMEOUT*/ {
                    setRetryIntervalSeconds = 1;
                }

                if setRetryIntervalSeconds != 0 {
                    retryIntervalSeconds = MAX(retryIntervalSeconds, 2);
                }
            }

            if shouldRetry != 0 {
                (*Pkt).RetryIn100nsUnits = retryIntervalSeconds;
                (*Pkt).RetryIn100nsUnits *= 1000 * 1000 * 10;
            }
        } else if (*pCdb).MODE_SENSE.OperationCode == SCSIOP_MODE_SENSE
            || (*pCdb).MODE_SENSE.OperationCode == SCSIOP_MODE_SENSE10
        {
            let mut retryIntervalSeconds: u32 = 0;

            shouldRetry = ClassInterpretSenseInfo(
                (*Pkt).Fdo,
                (*Pkt).Srb as *mut SCSI_REQUEST_BLOCK,
                0,
                0,
                timesAlreadyRetried,
                core::ptr::addr_of_mut!((*(*Pkt).Irp).IoStatus.__bindgen_anon_1.Status),
                core::ptr::addr_of_mut!(retryIntervalSeconds),
            ) as u8;

            if shouldRetry != 0 {
                let mut setRetryIntervalSeconds: u8 = 0;

                if validSense != 0 {
                    if (*(*Pkt).Irp).IoStatus.__bindgen_anon_1.Status == STATUS_DEVICE_NOT_READY
                        && additionalSenseCode == SCSI_ADSENSE_LUN_NOT_READY
                    {
                        setRetryIntervalSeconds = 1;
                    }
                }

                if setRetryIntervalSeconds == 0 && SRB_STATUS((*(*Pkt).Srb).SrbStatus) == 0x01 /*SRB_STATUS_SELECTION_TIMEOUT*/ {
                    setRetryIntervalSeconds = 1;
                }

                if setRetryIntervalSeconds != 0 {
                    retryIntervalSeconds = MAX(retryIntervalSeconds, 2);
                }
            }

            if (*(*Pkt).Irp).IoStatus.__bindgen_anon_1.Status == STATUS_VERIFY_REQUIRED {
                shouldRetry = 1;
            } else if SRB_STATUS((*(*Pkt).Srb).SrbStatus) == 3 /*SRB_STATUS_DATA_OVERRUN*/ {
                (*(*Pkt).Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
                shouldRetry = 0;
            }

            if shouldRetry != 0 {
                (*Pkt).RetryIn100nsUnits = retryIntervalSeconds;
                (*Pkt).RetryIn100nsUnits *= 1000 * 1000 * 10;
            }
        } else if (*pCdb).CDB10.OperationCode == SCSIOP_READ_CAPACITY
            || (*pCdb).CDB16.OperationCode == SCSIOP_READ_CAPACITY16
        {
            let mut retryIntervalSeconds: u32 = 0;

            shouldRetry = ClassInterpretSenseInfo(
                (*Pkt).Fdo,
                (*Pkt).Srb as *mut SCSI_REQUEST_BLOCK,
                0,
                0,
                timesAlreadyRetried,
                core::ptr::addr_of_mut!((*(*Pkt).Irp).IoStatus.__bindgen_anon_1.Status),
                core::ptr::addr_of_mut!(retryIntervalSeconds),
            ) as u8;

            if (*(*Pkt).Irp).IoStatus.__bindgen_anon_1.Status == STATUS_VERIFY_REQUIRED {
                shouldRetry = 1;
            }
            // #if (NTDDI_VERSION >= NTDDI_WINBLUE)
            else if ClasspSrbTimeOutStatus((*Pkt).Srb) != 0 {
                (*Pkt).TimedOut = 1;

                if shouldRetry != 0 {
                    if (*Pkt).NumIoTimeoutRetries == 0 {
                        shouldRetry = 0;
                    } else {
                        (*Pkt).NumIoTimeoutRetries -= 1;
                    }
                }
            }

            if shouldRetry != 0 {
                (*Pkt).RetryIn100nsUnits = retryIntervalSeconds;
                (*Pkt).RetryIn100nsUnits *= 1000 * 1000 * 10;
            }
        } else if IS_SCSIOP_READWRITE((*pCdb).CDB10.OperationCode) != 0 {
            let origCurrentSp = IoGetCurrentIrpStackLocation((*Pkt).OriginalIrp);
            let mut retryIntervalSeconds: u32 = 0;

            shouldRetry = ClassInterpretSenseInfo(
                (*Pkt).Fdo,
                (*Pkt).Srb as *mut SCSI_REQUEST_BLOCK,
                (*origCurrentSp).MajorFunction.into(),
                0,
                timesAlreadyRetried,
                core::ptr::addr_of_mut!((*(*Pkt).Irp).IoStatus.__bindgen_anon_1.Status),
                core::ptr::addr_of_mut!(retryIntervalSeconds),
            ) as u8;

            if (*(*Pkt).Irp).IoStatus.__bindgen_anon_1.Status == STATUS_INSUFFICIENT_RESOURCES {
                shouldRetry = 1;
            } else if TEST_FLAG((*origCurrentSp).Flags.into(), SL_OVERRIDE_VERIFY_VOLUME) != 0
                && (*(*Pkt).Irp).IoStatus.__bindgen_anon_1.Status == STATUS_VERIFY_REQUIRED
            {
                (*(*Pkt).Irp).IoStatus.__bindgen_anon_1.Status = STATUS_IO_DEVICE_ERROR;
                shouldRetry = 1;
            }
            // #if (NTDDI_VERSION >= NTDDI_WINBLUE)
            else if ClasspSrbTimeOutStatus((*Pkt).Srb) != 0 {
                (*Pkt).TimedOut = 1;

                if shouldRetry != 0 {
                    if (*Pkt).NumIoTimeoutRetries == 0 {
                        shouldRetry = 0;
                    } else {
                        (*Pkt).NumIoTimeoutRetries -= 1;
                    }
                }
            }

            if shouldRetry != 0 {
                (*Pkt).RetryIn100nsUnits = retryIntervalSeconds;
                (*Pkt).RetryIn100nsUnits *= 1000 * 1000 * 10;
            }
        } else if ClasspIsOffloadDataTransferCommand(pCdb) != 0 {
            let mut retryIntervalSeconds: u32 = 0;

            (*Pkt).TransferCount = 0;

            shouldRetry = ClassInterpretSenseInfo(
                (*Pkt).Fdo,
                (*Pkt).Srb as *mut SCSI_REQUEST_BLOCK,
                0,
                0,
                timesAlreadyRetried,
                core::ptr::addr_of_mut!((*(*Pkt).Irp).IoStatus.__bindgen_anon_1.Status),
                core::ptr::addr_of_mut!(retryIntervalSeconds),
            ) as u8;

            if shouldRetry != 0 {
                (*Pkt).RetryIn100nsUnits = retryIntervalSeconds;
                (*Pkt).RetryIn100nsUnits *= 1000 * 1000 * 10;
            } else {
                if ClasspIsTokenOperation(pCdb) != 0 {
                    let mut isInformationValid: u8 = 0;
                    let mut information: u64 = 0;

                    if validSense != 0 {
                        if (senseKey == SCSI_SENSE_COPY_ABORTED || senseKey == SCSI_SENSE_ABORTED_COMMAND)
                            && (additionalSenseCode == SCSI_ADSENSE_COPY_TARGET_DEVICE_ERROR
                                && additionalSenseQual == SCSI_SENSEQ_DATA_UNDERRUN)
                        {
                            if IsDescriptorSenseDataFormat(senseInfoBuffer) != 0 {
                                let mut startBuffer: *mut core::ffi::c_void = core::ptr::null_mut();
                                let mut startBufferLength: u8 = 0;

                                if ScsiGetSenseDescriptor(
                                    senseInfoBuffer,
                                    SrbGetSenseInfoBufferLength((*Pkt).Srb),
                                    core::ptr::addr_of_mut!(startBuffer),
                                    core::ptr::addr_of_mut!(startBufferLength),
                                ) != 0
                                {
                                    let mut outType: u8 = 0;
                                    let mut outBuffer: *mut core::ffi::c_void = core::ptr::null_mut();
                                    let mut outBufferLength: u8 = 0;
                                    let typeList: [u8; 1] = [SCSI_SENSE_DESCRIPTOR_TYPE_INFORMATION];

                                    if ScsiGetNextSenseDescriptorByType(
                                        startBuffer,
                                        startBufferLength,
                                        typeList.as_ptr(),
                                        1, // ARRAYSIZE(typeList)
                                        core::ptr::addr_of_mut!(outType),
                                        core::ptr::addr_of_mut!(outBuffer),
                                        core::ptr::addr_of_mut!(outBufferLength),
                                    ) != 0
                                    {
                                        if outType == SCSI_SENSE_DESCRIPTOR_TYPE_INFORMATION {
                                            if ScsiValidateInformationSenseDescriptor(outBuffer, outBufferLength) != 0 {
                                                let pDesc = outBuffer as *mut SCSI_SENSE_DESCRIPTOR_INFORMATION;
                                                REVERSE_BYTES_QUAD(
                                                    core::ptr::addr_of_mut!(information),
                                                    core::ptr::addr_of!((*pDesc).Information) as *const u8,
                                                );
                                                isInformationValid = 1;
                                            }
                                        } else {
                                            NT_ASSERT(0 as i32)
                                        }
                                    }
                                }
                            } else {
                                let pFixed = senseInfoBuffer as *mut FIXED_SENSE_DATA;
                                REVERSE_BYTES(
                                    core::ptr::addr_of_mut!(information) as *mut u32,  // REVERSE_BYTES expects u32 dst
                                    core::ptr::addr_of!((*pFixed).Information) as *const u8,
                                );
                                isInformationValid = 1;
                            }

                            if isInformationValid != 0 {
                                (*Pkt).TransferCount = information;
                            }
                        }
                    }
                }
            }
        } else {
            TracePrint(
                0x00000001,
                0x00000001,
                b"Unhandled SRB Function %xh in error path for packet %p (did miniport change Srb.Cdb.OperationCode ?)\0".as_ptr() as *const i8,
                (*pCdb).CDB10.OperationCode as u32,
                Pkt,
            );
        }
    }

    shouldRetry
}
#[no_mangle]
pub unsafe extern "C" fn RetryTransferPacket(Pkt: *mut TRANSFER_PACKET) -> i32 {
    unsafe {
        let mut packetDone: i32;
        let mut scaleDown = 0; // FALSE
        let fdoExtension = (*(*Pkt).Fdo).DeviceExtension as *mut FUNCTIONAL_DEVICE_EXTENSION;
        let fdoData = (*fdoExtension).PrivateFdoData as *mut CLASS_PRIVATE_FDO_DATA;
        let pCdb = SrbGetCdb((*Pkt).Srb);

        #[cfg(not(reactos))]
        #[cfg(ntddi_ge_winblue)]
        {
            if ClasspIsThinProvisioningError((*Pkt).Srb as *mut SCSI_REQUEST_BLOCK) != 0
                && !pCdb.is_null()
                && IS_SCSIOP_READWRITE((*pCdb).CDB10.OperationCode) != 0
            {
                if (*Pkt).NumThinProvisioningRetries >= NUM_THIN_PROVISIONING_RETRIES {
                    return 1; // TRUE
                }
                (*Pkt).NumThinProvisioningRetries += 1;
            }
            else {
                NT_ASSERT(
                    (((*Pkt).NumRetries > 0) || !(*Pkt).RetryHistory.is_null()) as i32
                );
                (*Pkt).NumRetries -= 1;
            }
        }
        #[cfg(not(all(not(reactos), ntddi_ge_winblue)))]
        {
            NT_ASSERT(
                (((*Pkt).NumRetries > 0) || !(*Pkt).RetryHistory.is_null()) as i32
            );
            (*Pkt).NumRetries -= 1;
        }

        TracePrint(
            TRACE_LEVEL_INFORMATION,
            TRACE_FLAG_GENERAL,
            b"retrying failed transfer (pkt=%ph, op=%s)\0".as_ptr() as *const i8,
            Pkt,
            DBGGETSCSIOPSTR((*Pkt).Srb),
        );

        if (*fdoData).DisableThrottling == 0 {
            if ((*Pkt).NumRetries == 0)
                && (ClasspIsThinProvisioningError((*Pkt).Srb as *mut SCSI_REQUEST_BLOCK) == 0)
            {
                scaleDown = 1; // TRUE
            }
            #[cfg(ntddi_ge_winblue)]
            {
                if (*Pkt).TimedOut != 0 && (*Pkt).NumIoTimeoutRetries == 0 {
                    scaleDown = 1;
                }
            }
        }

        if scaleDown != 0 {
            SrbSetSrbFlags((*Pkt).Srb, SRB_FLAGS_DISABLE_DISCONNECT);
            SrbSetSrbFlags((*Pkt).Srb, SRB_FLAGS_DISABLE_SYNCH_TRANSFER);
            SrbClearSrbFlags((*Pkt).Srb, SRB_FLAGS_QUEUE_ACTION_ENABLE);
            SrbSetRequestTag((*Pkt).Srb, SP_UNTAGGED);
        }

        if (*(*Pkt).Irp).IoStatus.__bindgen_anon_1.Status == STATUS_INSUFFICIENT_RESOURCES {
            let mut cdbOpcode: u8 = 0;
            let mut isReadWrite: i32 = 0;

            if !pCdb.is_null() {
                cdbOpcode = (*pCdb).CDB10.OperationCode;
                isReadWrite = IS_SCSIOP_READWRITE(cdbOpcode);
            }

            if ((*Pkt).DriverUsesStartIO != 0)
                && (cdbOpcode == SCSIOP_WRITE6
                    || cdbOpcode == SCSIOP_WRITE
                    || cdbOpcode == SCSIOP_WRITE12
                    || cdbOpcode == SCSIOP_WRITE16)
            {
                (*fdoData).TrackingFlags |= TRACKING_FORWARD_PROGRESS_PATH1;
                packetDone = 1;
            }
            else if (*Pkt).InLowMemRetry != 0 || isReadWrite == 0 {
                (*fdoData).TrackingFlags |= TRACKING_FORWARD_PROGRESS_PATH2;
                packetDone = 1;
            }
            else {
                (*fdoData).TrackingFlags |= TRACKING_FORWARD_PROGRESS_PATH3;
                InitLowMemRetry(
                    Pkt,
                    (*Pkt).BufPtrCopy,
                    (*Pkt).BufLenCopy,
                    (*Pkt).TargetLocationCopy,
                );
                StepLowMemRetry(Pkt);
                packetDone = 0;
            }
        }
        else {
            SrbSetDataBuffer((*Pkt).Srb, (*Pkt).BufPtrCopy);
            SrbSetDataTransferLength((*Pkt).Srb, (*Pkt).BufLenCopy);
            TransferPacketQueueRetryDpc(Pkt);
            packetDone = 0;
        }

        packetDone
    }
}
#[no_mangle]
pub unsafe extern "C" fn TransferPacketQueueRetryDpc(Pkt: *mut TRANSFER_PACKET) {
    KeInitializeDpc(
        core::ptr::addr_of_mut!((*Pkt).RetryTimerDPC),
        TransferPacketRetryTimerDpc as unsafe extern "system" fn(PKDPC, *mut core::ffi::c_void, *mut core::ffi::c_void, *mut core::ffi::c_void),
        Pkt as *mut core::ffi::c_void,
    );
    if (*Pkt).RetryIn100nsUnits == 0 {
        KeInsertQueueDpc(
            core::ptr::addr_of_mut!((*Pkt).RetryTimerDPC),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
    } else {
        let mut timerPeriod: LARGE_INTEGER;
        NT_ASSERT((*Pkt).RetryIn100nsUnits < 100 * 1000 * 1000 * 10 as i32)
        timerPeriod.QuadPart = -((*Pkt).RetryIn100nsUnits as i64);
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
    DeferredContext: *mut core::ffi::c_void,
    SystemArgument1: *mut core::ffi::c_void,
    SystemArgument2: *mut core::ffi::c_void,
) {

    _Analysis_assume_((DeferredContext as usize != 0) as i32);
    let pkt: *mut TRANSFER_PACKET = DeferredContext as *mut TRANSFER_PACKET;
    let fdo: PDEVICE_OBJECT = (*pkt).Fdo;
    let fdoExtension: *mut FUNCTIONAL_DEVICE_EXTENSION = (*fdo).DeviceExtension as *mut FUNCTIONAL_DEVICE_EXTENSION;

    let _ = Dpc;
    let _ = SystemArgument1;
    let _ = SystemArgument2;

    let srb = (*pkt).Srb;

    if PORT_ALLOCATED_SENSE_EX(fdoExtension, srb) != 0 {
        TracePrint(
            TRACE_LEVEL_INFORMATION,
            TRACE_FLAG_RW,
            b"Freeing port-allocated sense buffer for pkt %ph.\0".as_ptr() as *const i8,
            pkt,
        );
        FREE_PORT_ALLOCATED_SENSE_BUFFER_EX(fdoExtension, srb);
        SrbSetSenseInfoBuffer(srb, core::ptr::addr_of!((*pkt).SrbErrorSenseData) as *mut core::ffi::c_void);
        SrbSetSenseInfoBufferLength(srb, core::mem::size_of_val(&(*pkt).SrbErrorSenseData) as u32);
    } else {
        NT_ASSERT((SrbGetSenseInfoBuffer(srb) as *const _ == core::ptr::addr_of!((*pkt).SrbErrorSenseData) as *const _) as i32);
        NT_ASSERT((u32::from(SrbGetSenseInfoBufferLength(srb)) <= core::mem::size_of_val(&(*pkt).SrbErrorSenseData) as u32) as i32);
    }

    ntkrnl_sys::rtl::RtlZeroMemory(
        core::ptr::addr_of!((*pkt).SrbErrorSenseData) as *mut core::ffi::c_void,
        core::mem::size_of_val(&(*pkt).SrbErrorSenseData),
    );

    SubmitTransferPacket(pkt);
}
#[no_mangle]
pub unsafe extern "C" fn InitLowMemRetry(
    Pkt: *mut TRANSFER_PACKET,
    BufPtr: *mut core::ffi::c_void,
    Len: u32,
    TargetLocation: LARGE_INTEGER,
) {
    NT_ASSERT((Len > 0) as i32);
    NT_ASSERT(((*Pkt).InLowMemRetry == 0) as i32);
    if (*Pkt).DriverUsesStartIO != 0 {
        (*Pkt).InLowMemRetry = 0;
    } else {
        (*Pkt).InLowMemRetry = 1;
    }
    (*Pkt).LowMemRetry_remainingBufPtr = BufPtr;
    (*Pkt).LowMemRetry_remainingBufLen = Len;
    (*Pkt).LowMemRetry_nextChunkTargetLocation = TargetLocation;
}
#[no_mangle] 
 pub unsafe extern "C" fn StepLowMemRetry(Pkt: *mut TRANSFER_PACKET) -> i32 {
    let packetDone: i32;
    if (*Pkt).LowMemRetry_remainingBufLen == 0 {
        packetDone = 1;
    } else {
        let thisChunkLen: u32;
        if (*Pkt).DriverUsesStartIO != 0 {
            NT_ASSERT(if (*Pkt).InLowMemRetry == 0 { 1 } else { 0 } as i32)
            let fdoExt = (*(*Pkt).Fdo).DeviceExtension as *mut FUNCTIONAL_DEVICE_EXTENSION;
            let fdoData = (*fdoExt).PrivateFdoData;
            let commonExtension = (*(*Pkt).Fdo).DeviceExtension as *mut COMMON_DEVICE_EXTENSION;
            let adapterDesc = (*(*commonExtension).PartitionZeroExtension).AdapterDescriptor;
            let hwMaxXferLen: u32;
            let offset = (*Pkt).LowMemRetry_remainingBufPtr as usize;
            let not_aligned = (offset & (PAGE_SIZE - 1) as usize) != 0;
            let overflow_check = (*fdoData).HwMaxXferLen > (0xffffffffu32 - PAGE_SIZE);
            if not_aligned || overflow_check {
                hwMaxXferLen = (*fdoData).HwMaxXferLen;
            } else {
                NT_ASSERT(if (PAGE_SIZE % (*fdoExt).DiskGeometry.BytesPerSector) == 0 { 1 } else { 0 } as i32)
                hwMaxXferLen = min((*fdoData).HwMaxXferLen + PAGE_SIZE, (*adapterDesc).MaximumTransferLength);
            }
            thisChunkLen = MIN((*Pkt).LowMemRetry_remainingBufLen, hwMaxXferLen);
        } else {
            let bytesToNextPageBoundary: u32;
            let addr = (*Pkt).LowMemRetry_remainingBufPtr as usize;
            let offset = (addr % PAGE_SIZE as usize) as u32;
            bytesToNextPageBoundary = PAGE_SIZE - offset;
            thisChunkLen = MIN((*Pkt).LowMemRetry_remainingBufLen, bytesToNextPageBoundary);
            NT_ASSERT(if (*Pkt).InLowMemRetry != 0 { 1 } else { 0 } as i32)
        }
        SetupReadWriteTransferPacket(
            Pkt,
            (*Pkt).LowMemRetry_remainingBufPtr as *mut u8,
            thisChunkLen,
            (*Pkt).LowMemRetry_nextChunkTargetLocation,
            (*Pkt).OriginalIrp as *mut core::ffi::c_void,
        );
        (*Pkt).LowMemRetry_remainingBufPtr = ((*Pkt).LowMemRetry_remainingBufPtr as *mut u8).add(thisChunkLen as usize) as *mut core::ffi::c_void;
        (*Pkt).LowMemRetry_remainingBufLen -= thisChunkLen;
        (*Pkt).LowMemRetry_nextChunkTargetLocation.QuadPart += thisChunkLen as i64;
        (*Pkt).UsePartialMdl = 1;
        TransferPacketQueueRetryDpc(Pkt);
        packetDone = 0;
    }
    packetDone
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
//     retry.c
//
// Abstract:
//
//     Packet retry routines for CLASSPNP
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
// #include "classp.h"
// #include "debug.h"
//
// #ifdef DEBUG_USE_WPP
// #include "retry.tmh"
// #endif
//
//
// /*
//  *  InterpretTransferPacketError
//  *
//  *      Interpret the SRB error into a meaningful IRP status.
//  *      ClassInterpretSenseInfo also may modify the SRB for the retry.
//  *
//  *      Return TRUE iff packet should be retried.
//  */
// BOOLEAN InterpretTransferPacketError(PTRANSFER_PACKET Pkt)
// {
//     PFUNCTIONAL_DEVICE_EXTENSION fdoExtension = Pkt->Fdo->DeviceExtension;
//     PCLASS_PRIVATE_FDO_DATA fdoData = fdoExtension->PrivateFdoData;
//     ULONG timesAlreadyRetried;
//     BOOLEAN shouldRetry = FALSE;
//     PCDB pCdb = ClasspTransferPacketGetCdb(Pkt);
//
//     /*
//      *  Interpret the error using the returned sense info first.
//      */
//     Pkt->RetryIn100nsUnits = 0;
//
//
//     /*
//      *  Pre-calculate the number of times the IO has already been
//      *  retried, so that all InterpretSenseInfo routines get the right value.
//      */
//     if (ClasspTransferPacketGetNumberOfRetriesDone(Pkt, pCdb, &timesAlreadyRetried) == FALSE)
//     {
//         TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_GENERAL, "Unhandled SRB Function %xh in error path for packet %p (did miniport change Srb.Cdb.OperationCode ?)", (ULONG)pCdb->CDB10.OperationCode, Pkt));
//     }
//
//     if (fdoData->InterpretSenseInfo != NULL) {
//
//         SCSI_REQUEST_BLOCK tempSrb = { 0 };
//         PSCSI_REQUEST_BLOCK srbPtr = (PSCSI_REQUEST_BLOCK)Pkt->Srb;
//
//         // SAL annotation and ClassInitializeEx() both validate this
//         NT_ASSERT(fdoData->InterpretSenseInfo->Interpret != NULL as i32)
//
//         //
//         // If class driver does not support extended SRB and this is
//         // an extended SRB, convert to legacy SRB and pass to class
//         // driver.
//         //
//         if ((Pkt->Srb->Function == SRB_FUNCTION_STORAGE_REQUEST_BLOCK) &&
//             ((fdoExtension->CommonExtension.DriverExtension->SrbSupport &
//             CLASS_SRB_STORAGE_REQUEST_BLOCK) == 0)) {
//             ClasspConvertToScsiRequestBlock(&tempSrb, (PSTORAGE_REQUEST_BLOCK)Pkt->Srb);
//             srbPtr = &tempSrb;
//         }
//
//         shouldRetry = fdoData->InterpretSenseInfo->Interpret(Pkt->Fdo,
//                                                              Pkt->OriginalIrp,
//                                                              srbPtr,
//                                                              IRP_MJ_SCSI,
//                                                              0,
//                                                              timesAlreadyRetried,
//                                                              Pkt->RetryHistory,
//                                                              &Pkt->Irp->IoStatus.Status,
//                                                              &Pkt->RetryIn100nsUnits);
//
//
//     } else {
//
//         //
//         // In this case, fdoData->InterpretSenseInfo == NULL so we must do our
//         // own error code and sense info processing.
//         //
//
//         PVOID senseInfoBuffer = ClasspTransferPacketGetSenseInfoBuffer(Pkt);
//         UCHAR senseInfoBufferLength = ClasspTransferPacketGetSenseInfoBufferLength(Pkt);
//         BOOLEAN validSense = FALSE;
//         UCHAR senseKey = 0;
//         UCHAR additionalSenseCode = 0;
//         UCHAR additionalSenseQual = 0;
//
//         NT_ASSERT(senseInfoBuffer as i32)
//
//         validSense = ScsiGetSenseKeyAndCodes(senseInfoBuffer,
//                                              senseInfoBufferLength,
//                                              SCSI_SENSE_OPTIONS_FIXED_FORMAT_IF_UNKNOWN_FORMAT_INDICATED,
//                                              &senseKey,
//                                              &additionalSenseCode,
//                                              &additionalSenseQual);
//
//         if (pCdb->MEDIA_REMOVAL.OperationCode == SCSIOP_MEDIUM_REMOVAL) {
//
//             ULONG retryIntervalSeconds = 0;
//             /*
//              *  This is an Ejection Control SRB.  Interpret its sense info specially.
//              */
//             shouldRetry = ClassInterpretSenseInfo(
//                 Pkt->Fdo,
//                 (PSCSI_REQUEST_BLOCK)Pkt->Srb,
//                 IRP_MJ_SCSI,
//                 0,
//                 timesAlreadyRetried,
//                 &Pkt->Irp->IoStatus.Status,
//                 &retryIntervalSeconds);
//
//             if (shouldRetry) {
//                 /*
//                  *  If the device is not ready, wait at least 2 seconds before retrying.
//                  */
//                 BOOLEAN setRetryIntervalSeconds = FALSE;
//
//                 if (validSense) {
//
//                     if ((Pkt->Irp->IoStatus.Status == STATUS_DEVICE_NOT_READY) &&
//                         (additionalSenseCode == SCSI_ADSENSE_LUN_NOT_READY)) {
//                         setRetryIntervalSeconds = TRUE;
//                     }
//                 }
//
//                 if (!setRetryIntervalSeconds && (SRB_STATUS(Pkt->Srb->SrbStatus) == SRB_STATUS_SELECTION_TIMEOUT)) {
//                     setRetryIntervalSeconds = TRUE;
//                 }
//
//                 if (setRetryIntervalSeconds) {
//                     retryIntervalSeconds = MAX(retryIntervalSeconds, 2);
//                 }
//             }
//
//             if (shouldRetry)
//             {
//                 Pkt->RetryIn100nsUnits = retryIntervalSeconds;
//                 Pkt->RetryIn100nsUnits *= 1000 * 1000 * 10;
//             }
//
//         }
//         else if ((pCdb->MODE_SENSE.OperationCode == SCSIOP_MODE_SENSE) ||
//                  (pCdb->MODE_SENSE.OperationCode == SCSIOP_MODE_SENSE10)) {
//
//             ULONG retryIntervalSeconds = 0;
//             /*
//              *  This is an Mode Sense SRB.  Interpret its sense info specially.
//              */
//             shouldRetry = ClassInterpretSenseInfo(
//                 Pkt->Fdo,
//                 (PSCSI_REQUEST_BLOCK)Pkt->Srb,
//                 IRP_MJ_SCSI,
//                 0,
//                 timesAlreadyRetried,
//                 &Pkt->Irp->IoStatus.Status,
//                 &retryIntervalSeconds);
//             if (shouldRetry) {
//                 /*
//                  *  If the device is not ready, wait at least 2 seconds before retrying.
//                  */
//                 BOOLEAN setRetryIntervalSeconds = FALSE;
//
//                 if (validSense) {
//                     if ((Pkt->Irp->IoStatus.Status == STATUS_DEVICE_NOT_READY) &&
//                         (additionalSenseCode == SCSI_ADSENSE_LUN_NOT_READY)) {
//                         setRetryIntervalSeconds = TRUE;
//                     }
//                 }
//
//                 if (!setRetryIntervalSeconds && (SRB_STATUS(Pkt->Srb->SrbStatus) == SRB_STATUS_SELECTION_TIMEOUT)) {
//                     setRetryIntervalSeconds = TRUE;
//                 }
//
//                 if (setRetryIntervalSeconds) {
//                     retryIntervalSeconds = MAX(retryIntervalSeconds, 2);
//                 }
//             }
//
//             /*
//              *  Some special cases for mode sense.
//              */
//             if (Pkt->Irp->IoStatus.Status == STATUS_VERIFY_REQUIRED) {
//                 shouldRetry = TRUE;
//             }
//             else if (SRB_STATUS(Pkt->Srb->SrbStatus) == SRB_STATUS_DATA_OVERRUN) {
//                 /*
//                  *  This is a HACK.
//                  *  Atapi returns SRB_STATUS_DATA_OVERRUN when it really means
//                  *  underrun (i.e. success, and the buffer is longer than needed).
//                  *  So treat this as a success.
//                  *  When the caller of this function sees that the status was changed to success,
//                  *  it will add the transferred length to the original irp.
//                  */
//                 Pkt->Irp->IoStatus.Status = STATUS_SUCCESS;
//                 shouldRetry = FALSE;
//             }
//
//             if (shouldRetry)
//             {
//                 Pkt->RetryIn100nsUnits = retryIntervalSeconds;
//                 Pkt->RetryIn100nsUnits *= 1000 * 1000 * 10;
//             }
//
//         }
//         else if ((pCdb->CDB10.OperationCode == SCSIOP_READ_CAPACITY) ||
//                  (pCdb->CDB16.OperationCode == SCSIOP_READ_CAPACITY16)) {
//
//             ULONG retryIntervalSeconds = 0;
//
//             /*
//              *  This is a Drive Capacity SRB.  Interpret its sense info specially.
//              */
//             shouldRetry = ClassInterpretSenseInfo(
//                 Pkt->Fdo,
//                 (PSCSI_REQUEST_BLOCK)Pkt->Srb,
//                 IRP_MJ_SCSI,
//                 0,
//                 timesAlreadyRetried,
//                 &Pkt->Irp->IoStatus.Status,
//                 &retryIntervalSeconds);
//             if (Pkt->Irp->IoStatus.Status == STATUS_VERIFY_REQUIRED) {
//                 shouldRetry = TRUE;
//             }
// #if (NTDDI_VERSION >= NTDDI_WINBLUE)
//             else if (ClasspSrbTimeOutStatus(Pkt->Srb)) {
//
//                 Pkt->TimedOut = TRUE;
//
//                 if (shouldRetry) {
//                     //
//                     // For requests that have timed-out we may only perform a limited
//                     // number of retries.  This is typically less than the general
//                     // number of retries allowed.
//                     //
//                     if (Pkt->NumIoTimeoutRetries == 0) {
//                         shouldRetry = FALSE;
//                     } else {
//                         Pkt->NumIoTimeoutRetries--;
//                         //
//                         // We expect to be able to retry if there are some general retries remaining.
//                         //
//                     }
//                 }
//             }
// #endif
//
//             if (shouldRetry)
//             {
//                 Pkt->RetryIn100nsUnits = retryIntervalSeconds;
//                 Pkt->RetryIn100nsUnits *= 1000 * 1000 * 10;
//             }
//
//         }
//         else if (IS_SCSIOP_READWRITE(pCdb->CDB10.OperationCode)) {
//
//             ULONG retryIntervalSeconds = 0;
//             /*
//              *  This is a Read/Write Data packet.
//              */
//             PIO_STACK_LOCATION origCurrentSp = IoGetCurrentIrpStackLocation(Pkt->OriginalIrp);
//
//             shouldRetry = ClassInterpretSenseInfo(Pkt->Fdo,
//                                                   (PSCSI_REQUEST_BLOCK)Pkt->Srb,
//                                                   origCurrentSp->MajorFunction,
//                                                   0,
//                                                   timesAlreadyRetried,
//                                                   &Pkt->Irp->IoStatus.Status,
//                                                   &retryIntervalSeconds);
//
//             /*
//              *  Deal with some special cases.
//              */
//             if (Pkt->Irp->IoStatus.Status == STATUS_INSUFFICIENT_RESOURCES) {
//                 /*
//                  *  We are in extreme low-memory stress.
//                  *  We will retry in smaller chunks.
//                  */
//                 shouldRetry = TRUE;
//             }
//             else if (TEST_FLAG(origCurrentSp->Flags, SL_OVERRIDE_VERIFY_VOLUME) &&
//                      (Pkt->Irp->IoStatus.Status == STATUS_VERIFY_REQUIRED)) {
//                 /*
//                  *  We are still verifying a (possibly) reloaded disk/cdrom.
//                  *  So retry the request.
//                  */
//                 Pkt->Irp->IoStatus.Status = STATUS_IO_DEVICE_ERROR;
//                 shouldRetry = TRUE;
//
//             }
// #if (NTDDI_VERSION >= NTDDI_WINBLUE)
//             else if (ClasspSrbTimeOutStatus(Pkt->Srb)) {
//
//                 Pkt->TimedOut = TRUE;
//
//                 if (shouldRetry) {
//                     //
//                     // For requests that have timed-out we may only perform a limited
//                     // number of retries.  This is typically less than the general
//                     // number of retries allowed.
//                     //
//                     if (Pkt->NumIoTimeoutRetries == 0) {
//                         shouldRetry = FALSE;
//                     } else {
//                         Pkt->NumIoTimeoutRetries--;
//                         //
//                         // We expect to be able to retry if there are some general retries remaining.
//                         //
//                     }
//                 }
//             }
// #endif
//
//             if (shouldRetry)
//             {
//                 Pkt->RetryIn100nsUnits = retryIntervalSeconds;
//                 Pkt->RetryIn100nsUnits *= 1000 * 1000 * 10;
//             }
//
//         } else if (ClasspIsOffloadDataTransferCommand(pCdb)) {
//
//             ULONG retryIntervalSeconds = 0;
//
//             Pkt->TransferCount = 0;
//
//             shouldRetry = ClassInterpretSenseInfo(
//                 Pkt->Fdo,
//                 (PSCSI_REQUEST_BLOCK)Pkt->Srb,
//                 IRP_MJ_SCSI,
//                 0,
//                 timesAlreadyRetried,
//                 &Pkt->Irp->IoStatus.Status,
//                 &retryIntervalSeconds);
//
//             if (shouldRetry) {
//
//                 Pkt->RetryIn100nsUnits = retryIntervalSeconds;
//                 Pkt->RetryIn100nsUnits *= 1000 * 1000 * 10;
//
//             } else {
//
//                 if (ClasspIsTokenOperation(pCdb)) {
//
//                     BOOLEAN isInformationValid = FALSE;
//                     ULONGLONG information = 0;
//
//                     if (validSense) {
//
//                         //
//                         // If this is a data underrun condition (i.e. target truncated the offload data transfer),
//                         // the SenseData's Information field may have the TransferCount.
//                         //
//                         if ((senseKey == SCSI_SENSE_COPY_ABORTED || senseKey == SCSI_SENSE_ABORTED_COMMAND) &&
//                             (additionalSenseCode == SCSI_ADSENSE_COPY_TARGET_DEVICE_ERROR && additionalSenseQual == SCSI_SENSEQ_DATA_UNDERRUN)) {
//
//                             //
//                             // Sense data in Descriptor format
//                             //
//                             if (IsDescriptorSenseDataFormat(senseInfoBuffer)) {
//
//                                 PVOID startBuffer = NULL;
//                                 UCHAR startBufferLength = 0;
//
//
//                                 if (ScsiGetSenseDescriptor(senseInfoBuffer,
//                                                             SrbGetSenseInfoBufferLength(Pkt->Srb),
//                                                             &startBuffer,
//                                                             &startBufferLength)) {
//                                     UCHAR outType;
//                                     PVOID outBuffer = NULL;
//                                     UCHAR outBufferLength = 0;
//
//                                     UCHAR typeList[1] = { SCSI_SENSE_DESCRIPTOR_TYPE_INFORMATION };
//
//                                     if (ScsiGetNextSenseDescriptorByType(startBuffer,
//                                         startBufferLength,
//                                         typeList,
//                                         ARRAYSIZE(typeList),
//                                         &outType,
//                                         &outBuffer,
//                                         &outBufferLength)) {
//
//                                         if (outType == SCSI_SENSE_DESCRIPTOR_TYPE_INFORMATION) {
//
//                                             if (ScsiValidateInformationSenseDescriptor(outBuffer, outBufferLength)) {
//                                                 REVERSE_BYTES_QUAD(&information, &(((PSCSI_SENSE_DESCRIPTOR_INFORMATION)outBuffer)->Information));
//                                                 isInformationValid = TRUE;
//                                             }
//
//                                         } else {
//
//                                             //
//                                             // ScsiGetNextDescriptorByType should only return a type that is specified by us.
//                                             //
//                                             NT_ASSERT(FALSE as i32)
//                                         }
//                                     }
//                                 }
//                             } else {
//
//                                 //
//                                 // Sense data in Fixed format
//                                 //
//                                 REVERSE_BYTES(&information, &(((PFIXED_SENSE_DATA)senseInfoBuffer)->Information));
//                                 isInformationValid = TRUE;
//                             }
//
//                             if (isInformationValid) {
//                                 Pkt->TransferCount = information;
//                             }
//                         }
//                     }
//                 }
//             }
//
//         }
//         else {
//             TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_GENERAL, "Unhandled SRB Function %xh in error path for packet %p (did miniport change Srb.Cdb.OperationCode ?)", (ULONG)pCdb->CDB10.OperationCode, Pkt));
//         }
//     }
//
//     return shouldRetry;
// }
//
//
// /*
//  *  RetryTransferPacket
//  *
//  *      Retry sending a TRANSFER_PACKET.
//  *
//  *      Return TRUE iff the packet is complete.
//  *          (if so the status in pkt->irp is the final status).
//  */
// BOOLEAN RetryTransferPacket(PTRANSFER_PACKET Pkt)
// {
//     BOOLEAN packetDone;
//     BOOLEAN scaleDown = FALSE;
//     PFUNCTIONAL_DEVICE_EXTENSION fdoExtension = (PFUNCTIONAL_DEVICE_EXTENSION)Pkt->Fdo->DeviceExtension;
//     PCLASS_PRIVATE_FDO_DATA fdoData = fdoExtension->PrivateFdoData;
//     PCDB pCdb = SrbGetCdb(Pkt->Srb);
//
// #if !defined(__REACTOS__) && NTDDI_VERSION >= NTDDI_WINBLUE
//     if(ClasspIsThinProvisioningError((PSCSI_REQUEST_BLOCK)Pkt->Srb) &&
//         (pCdb != NULL) && IS_SCSIOP_READWRITE(pCdb->CDB10.OperationCode)) {
//
//         if(Pkt->NumThinProvisioningRetries >= NUM_THIN_PROVISIONING_RETRIES) {
//             //We've already retried this the maximum times.  Bail out.
//             return TRUE;
//         }
//         Pkt->NumThinProvisioningRetries++;
//     }
//     else {
//         NT_ASSERT(Pkt->NumRetries > 0 || Pkt->RetryHistory as i32)
//         Pkt->NumRetries--;
//     }
// #else
//     NT_ASSERT(Pkt->NumRetries > 0 || Pkt->RetryHistory as i32)
//     Pkt->NumRetries--;
// #endif
//
//     TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_GENERAL, "retrying failed transfer (pkt=%ph, op=%s)", Pkt, DBGGETSCSIOPSTR(Pkt->Srb)));
//
//     if (!fdoData->DisableThrottling) {
//
//         //
//         // If this is the last retry, then turn off disconnect, sync transfer,
//         // and tagged queuing.  On all other retries, leave the original settings.
//         // Do not apply this for thin provisioning soft threshold errors, since
//         // they should succeed as soon as they're retried on the right IT nexus.
//         //
//         if ((Pkt->NumRetries == 0) && !ClasspIsThinProvisioningError((PSCSI_REQUEST_BLOCK)Pkt->Srb)) {
//             scaleDown = TRUE;
//         }
//
// #if (NTDDI_VERSION >= NTDDI_WINBLUE)
//         //
//         // If this request previously timed-out and there are no more retries left
//         // for timed-out requests, then we should also apply the scale down.
//         //
//         if (Pkt->TimedOut && Pkt->NumIoTimeoutRetries == 0) {
//             scaleDown = TRUE;
//         }
// #endif
//     }
//
//
//     if (scaleDown) {
//         /*
//          *  Tone down performance on the retry.
//          *  This increases the chance for success on the retry.
//          *  We've seen instances of drives that fail consistently but then start working
//          *  once this scale-down is applied.
//          */
//         SrbSetSrbFlags(Pkt->Srb, SRB_FLAGS_DISABLE_DISCONNECT);
//         SrbSetSrbFlags(Pkt->Srb, SRB_FLAGS_DISABLE_SYNCH_TRANSFER);
//         SrbClearSrbFlags(Pkt->Srb, SRB_FLAGS_QUEUE_ACTION_ENABLE);
//         SrbSetRequestTag(Pkt->Srb, SP_UNTAGGED);
//     }
//
//     if (Pkt->Irp->IoStatus.Status == STATUS_INSUFFICIENT_RESOURCES) {
//
//         UCHAR cdbOpcode = 0;
//         BOOLEAN isReadWrite = FALSE;
//
//         if (pCdb) {
//             cdbOpcode = pCdb->CDB10.OperationCode;
//             isReadWrite = IS_SCSIOP_READWRITE(cdbOpcode);
//         }
//
//         if ((Pkt->DriverUsesStartIO) &&
//             ( (cdbOpcode == SCSIOP_WRITE6 ) ||
//               (cdbOpcode == SCSIOP_WRITE  ) ||
//               (cdbOpcode == SCSIOP_WRITE12) ||
//               (cdbOpcode == SCSIOP_WRITE16) )) {
//
//             /* don't retry writes in super-low-memory conditions if the
//              * driver must serialize against StartIO.  This is because
//              * some write methods used in such drivers cannot accept
//              * random-sized writes. (i.e CD-RW in packet writing mode)
//              * Reads, however, are always safe to split up.
//              */
//             SET_FLAG(fdoData->TrackingFlags, TRACKING_FORWARD_PROGRESS_PATH1);
//             packetDone = TRUE;
//         }
//         else if (Pkt->InLowMemRetry || !isReadWrite){
//             /*
//              *  This should never happen under normal circumstances.
//              *  The memory manager guarantees that at least four pages will
//              *  be available to allow forward progress in the port driver.
//              *  So a one-page transfer should never fail with insufficient resources.
//              *
//              *  However, it is possible to get in here with virtual storage
//              *  or thin provisioned storage for example.
//              *  A single sector write can trigger an allocation request and
//              *  presently a forward progress guarantee is not provided.
//              *  VHD also may have some limitations in forward progress guarantee.
//              *  And USB too might also fall into this category.
//              */
//             SET_FLAG(fdoData->TrackingFlags, TRACKING_FORWARD_PROGRESS_PATH2);
//             packetDone = TRUE;
//         }
//         else {
//             /*
//              *  We are in low-memory stress.
//              *  Start the low-memory retry state machine, which tries to
//              *  resend the packet in little one-page chunks.
//              */
//             SET_FLAG(fdoData->TrackingFlags, TRACKING_FORWARD_PROGRESS_PATH3);
//             InitLowMemRetry(Pkt,
//                             Pkt->BufPtrCopy,
//                             Pkt->BufLenCopy,
//                             Pkt->TargetLocationCopy);
//             StepLowMemRetry(Pkt);
//             packetDone = FALSE;
//         }
//     }
//     else {
//         /*
//          *  Retry the packet by simply resending it after a delay.
//          *  Put the packet back in the pending queue and
//          *  schedule a timer to retry the transfer.
//          *
//          *  Do not call SetupReadWriteTransferPacket again because:
//          *  (1)  The minidriver may have set some bits
//          *       in the SRB that it needs again and
//          *  (2)  doing so would reset numRetries.
//          *
//          *  BECAUSE we do not call SetupReadWriteTransferPacket again,
//          *  we have to reset a couple fields in the SRB that
//          *  some miniports overwrite when they fail an SRB.
//          */
//
//         SrbSetDataBuffer(Pkt->Srb, Pkt->BufPtrCopy);
//         SrbSetDataTransferLength(Pkt->Srb, Pkt->BufLenCopy);
//
//         TransferPacketQueueRetryDpc(Pkt);
//
//         packetDone = FALSE;
//     }
//
//     return packetDone;
// }
//
//
// VOID TransferPacketQueueRetryDpc(PTRANSFER_PACKET Pkt)
// {
//     KeInitializeDpc(&Pkt->RetryTimerDPC, TransferPacketRetryTimerDpc, Pkt);
//
//     if (Pkt->RetryIn100nsUnits == 0){
//         KeInsertQueueDpc(&Pkt->RetryTimerDPC, NULL, NULL);
//     }
//     else {
//         LARGE_INTEGER timerPeriod;
//
//         NT_ASSERT(Pkt->RetryIn100nsUnits < 100 * 1000 * 1000 * 10 as i32) // sanity check -- 100 seconds is normally too long
//         timerPeriod.QuadPart = -(Pkt->RetryIn100nsUnits);
//         KeInitializeTimer(&Pkt->RetryTimer);
//         KeSetTimer(&Pkt->RetryTimer, timerPeriod, &Pkt->RetryTimerDPC);
//     }
// }
//
//
// VOID
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// TransferPacketRetryTimerDpc(   IN PKDPC Dpc,
//                                     IN PVOID DeferredContext,
//                                     IN PVOID SystemArgument1,
//                                     IN PVOID SystemArgument2)
// {
//     PTRANSFER_PACKET pkt;
//     PDEVICE_OBJECT fdo;
//     PFUNCTIONAL_DEVICE_EXTENSION fdoExtension;
//
//     _Analysis_assume_(DeferredContext != NULL);
//
//     pkt = (PTRANSFER_PACKET)DeferredContext;
//
//     fdo = pkt->Fdo;
//     fdoExtension = fdo->DeviceExtension;
//
//     UNREFERENCED_PARAMETER(Dpc);
//     UNREFERENCED_PARAMETER(SystemArgument1);
//     UNREFERENCED_PARAMETER(SystemArgument2);
//
//
//     /*
//      *  Sometimes the port driver can allocates a new 'sense' buffer
//      *  to report transfer errors, e.g. when the default sense buffer
//      *  is too small.  If so, it is up to us to free it.
//      *  Now that we're done using the sense info, free it if appropriate.
//      *  Then clear the sense buffer so it doesn't pollute future errors returned in this packet.
//      */
//     if (PORT_ALLOCATED_SENSE_EX(fdoExtension, pkt->Srb)) {
//         TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_RW, "Freeing port-allocated sense buffer for pkt %ph.", pkt));
//         FREE_PORT_ALLOCATED_SENSE_BUFFER_EX(fdoExtension, pkt->Srb);
//         SrbSetSenseInfoBuffer(pkt->Srb, &pkt->SrbErrorSenseData);
//         SrbSetSenseInfoBufferLength(pkt->Srb, sizeof(pkt->SrbErrorSenseData));
//     }
//     else {
//         NT_ASSERT(SrbGetSenseInfoBuffer(pkt->Srb) == &pkt->SrbErrorSenseData as i32)
//         NT_ASSERT(SrbGetSenseInfoBufferLength(pkt->Srb) <= sizeof(pkt->SrbErrorSenseData) as i32)
//     }
//
//     RtlZeroMemory(&pkt->SrbErrorSenseData, sizeof(pkt->SrbErrorSenseData));
//
//     SubmitTransferPacket(pkt);
//
// }
//
//
// VOID InitLowMemRetry(PTRANSFER_PACKET Pkt, PVOID BufPtr, ULONG Len, LARGE_INTEGER TargetLocation)
// {
//     NT_ASSERT(Len > 0 as i32)
//     NT_ASSERT(!Pkt->InLowMemRetry as i32)
//
//     if (Pkt->DriverUsesStartIO)
//     {
//         /*
//          * special case: StartIO-based writing must stay serialized for performance
//          * and proper operations (i.e. sequential writing mode).  If need more than
//          * one transfer to perform this operation, and it's a StartIO-based driver
//          * (such as CDROM), then just use a single packet and use the retry logic
//          * that's already built-in to the packet engine.  Note that low-mem retry
//          * cannot be used directly because some write methods do not work if the
//          * writes are only PAGE_SIZE (i.e. packet writing may corrupt data).
//          */
//         Pkt->InLowMemRetry = FALSE;
//     }
//     else
//     {
//         Pkt->InLowMemRetry = TRUE;
//     }
//     Pkt->LowMemRetry_remainingBufPtr = BufPtr;
//     Pkt->LowMemRetry_remainingBufLen = Len;
//     Pkt->LowMemRetry_nextChunkTargetLocation = TargetLocation;
// }
//
//
// /*
//  *  StepLowMemRetry
//  *
//  *      During extreme low-memory stress, this function retries
//  *      a packet in small one-page chunks, sent serially.
//  *
//  *      Returns TRUE iff the packet is done.
//  */
// BOOLEAN StepLowMemRetry(PTRANSFER_PACKET Pkt)
// {
//     BOOLEAN packetDone;
//
//     if (Pkt->LowMemRetry_remainingBufLen == 0){
//         packetDone = TRUE;
//     }
//     else {
//         ULONG thisChunkLen;
//         if (Pkt->DriverUsesStartIO)
//         {
//             /*
//              * Need the fdoData for the HwMaxXferLen
//              */
//             PFUNCTIONAL_DEVICE_EXTENSION fdoExt = Pkt->Fdo->DeviceExtension;
//             PCLASS_PRIVATE_FDO_DATA fdoData = fdoExt->PrivateFdoData;
//
//             /*
//              * Need the adapterDesc to limit transfers based on byte count
//              */
//             PCOMMON_DEVICE_EXTENSION commonExtension = Pkt->Fdo->DeviceExtension;
//             PSTORAGE_ADAPTER_DESCRIPTOR adapterDesc = commonExtension->PartitionZeroExtension->AdapterDescriptor;
//
//             ULONG hwMaxXferLen;
//
//             /*
//              * special case: StartIO-based writing must stay serialized for performance
//              * and proper operations (i.e. sequential writing mode).  If need more than
//              * one transfer to perform this operation, and it's a StartIO-based driver
//              * (such as CDROM), then just use a single packet and use the retry logic
//              * that's already built-in to the packet engine.  Note that low-mem retry
//              * cannot be used directly because some write methods do not work if the
//              * writes are only PAGE_SIZE (i.e. packet writing may corrupt data).
//              */
//             NT_ASSERT(!Pkt->InLowMemRetry as i32)
//
//             /*
//              *  We precomputed fdoData->HwMaxXferLen using (MaximumPhysicalPages-1).
//              *  If the buffer is page-aligned, that's one less page crossing so we can add the page back in.
//              *  Note: adapters that return MaximumPhysicalPages=0x10 depend on this to
//              *           transfer aligned 64K requests in one piece.
//              *  Also note:  make sure adding PAGE_SIZE back in doesn't wrap to zero.
//              */
//
//             if (((ULONG_PTR)(Pkt->LowMemRetry_remainingBufPtr) & (PAGE_SIZE-1)) || (fdoData->HwMaxXferLen > 0xffffffff-PAGE_SIZE)){
//                 hwMaxXferLen = fdoData->HwMaxXferLen;
//             }
//             else {
//                 NT_ASSERT((PAGE_SIZE%fdoExt->DiskGeometry.BytesPerSector) == 0 as i32)
//                 hwMaxXferLen = min(fdoData->HwMaxXferLen+PAGE_SIZE, adapterDesc->MaximumTransferLength);
//             }
//             thisChunkLen = MIN(Pkt->LowMemRetry_remainingBufLen, hwMaxXferLen);
//         }
//         else {
//             /*
//              *  Make sure the little chunk we send is <= a page length
//              *  AND that it does not cross any page boundaries.
//              */
//             ULONG bytesToNextPageBoundary;
//             bytesToNextPageBoundary = PAGE_SIZE-(ULONG)((ULONG_PTR)Pkt->LowMemRetry_remainingBufPtr%PAGE_SIZE);
//             thisChunkLen = MIN(Pkt->LowMemRetry_remainingBufLen, bytesToNextPageBoundary);
//             NT_ASSERT(Pkt->InLowMemRetry as i32)
//         }
//
//
//         /*
//          *  Set up the transfer packet for the new little chunk.
//          *  This will reset numRetries so that we retry each chunk as required.
//          */
//         SetupReadWriteTransferPacket(Pkt,
//                                      Pkt->LowMemRetry_remainingBufPtr,
//                                      thisChunkLen,
//                                      Pkt->LowMemRetry_nextChunkTargetLocation,
//                                      Pkt->OriginalIrp);
//
//         Pkt->LowMemRetry_remainingBufPtr += thisChunkLen;
//         Pkt->LowMemRetry_remainingBufLen -= thisChunkLen;
//         Pkt->LowMemRetry_nextChunkTargetLocation.QuadPart += thisChunkLen;
//
//         //
//         // When running in low-memory stress, always use a partial MDL.
//         // This allows lower drivers to potentially map a smaller buffer.
//         //
//         Pkt->UsePartialMdl = TRUE;
//
//         TransferPacketQueueRetryDpc(Pkt);
//
//         packetDone = FALSE;
//     }
//
//     return packetDone;
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
