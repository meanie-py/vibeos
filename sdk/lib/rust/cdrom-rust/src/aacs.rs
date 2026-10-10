#[allow(unused_imports)] use crate::{NT_ASSERT};
use crate::types::*;
use kmdf_sys::*;
use ntkrnl_sys::ddk::{IRP, PIRP, IO_STACK_LOCATION, PIO_STACK_LOCATION, GUID, LPGUID, LPCGUID, DEVICE_OBJECT, PDEVICE_OBJECT, MDL, PMDL};
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::rtl::*;
use core::ffi::c_void;

extern "system" {
    fn ScratchBuffer_BeginUse(DeviceExtension: *mut CDROM_DEVICE_EXTENSION);
    fn ScratchBuffer_EndUse(DeviceExtension: *mut CDROM_DEVICE_EXTENSION);
    fn ScratchBuffer_ExecuteCdb( DeviceExtension: *mut CDROM_DEVICE_EXTENSION, Request: WDFREQUEST, TransferSize: u32, Retry: u8, Cdb: *const core::ffi::c_void, CdbLength: u32, ) -> i32;
    fn ScratchBuffer_ResetItems(DeviceExtension: *mut CDROM_DEVICE_EXTENSION, ClearStatus: u8);
}
unsafe extern "C" {
    fn REVERSE_SHORT(p: *mut u16);
}

const IOCTL_AACS_READ_MEDIA_KEY_BLOCK: u32 = 0x003350C4; // FIXED 2026-07-05: translation invented wrong value (verified vs psdk headers)
const IOCTL_AACS_READ_MEDIA_KEY_BLOCK_SIZE: u32 = 0x003350C0; // FIXED 2026-07-05: translation invented wrong value (verified vs psdk headers)
const AACS_MKB_PACK_SIZE: u32 = 0x8000;
const SCSIOP_READ_DVD_STRUCTURE: u8 = 0xAD;
const TRACE_LEVEL_WARNING: i32 = 3; // storswtr.h:26-30
const TRACE_FLAG_INIT: i32 = 8; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)

// WDF_REQUEST_PARAMETERS is assumed defined elsewhere with correct layout
// CDROM_DEVICE_EXTENSION is assumed defined elsewhere
// CDB, READ_DVD_STRUCTURES_HEADER, DVD_DESCRIPTOR_HEADER are assumed defined elsewhere

// Unknown constants from the C code; approximated values
const DVD_END_ALL_SESSIONS: u8 = 0xFF;
const MAX_COPY_PROTECT_AGID: u8 = 4;
const DVD_INVALIDATE_AGID: u8 = 0x3F;

// SCSI opcodes (REPORT KEY / SEND KEY) per SPC/MMC
const SCSIOP_REPORT_KEY: u8 = 0xA4; // typical SCSI REPORT KEY opcode
const SCSIOP_SEND_KEY: u8 = 0xA3;
const STATUS_IO_DEVICE_ERROR: i32 = 0xC0000185u32 as i32; // NTSTATUS code
const TRUE: i32 = 1;
#[no_mangle]
pub unsafe extern "C" fn DeviceHandleAacsReadMediaKeyBlock(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Request: WDFREQUEST,
    RequestParameters: WDF_REQUEST_PARAMETERS,
    DataLength: *mut usize,
) -> i32 {
    let mut status: i32; // STATUS_SUCCESS
    let mut layerNumber: *mut u8 = core::ptr::null_mut();
    let mut outputBuffer: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut transferSize: u32 = core::mem::size_of::<READ_DVD_STRUCTURES_HEADER>() as u32;

    PAGED_CODE!();

    *DataLength = 0;

    status = WdfRequestRetrieveInputBuffer(
        Request,
        RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
        &raw mut layerNumber as *mut *mut core::ffi::c_void,
        core::ptr::null_mut(),
    );

    if status >= 0 {
        status = WdfRequestRetrieveOutputBuffer(
            Request,
            RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength,
            &raw mut outputBuffer as *mut *mut core::ffi::c_void,
            core::ptr::null_mut(),
        );
    }

    if status >= 0 {
        if RequestParameters.Parameters.DeviceIoControl.as_ref().IoControlCode == IOCTL_AACS_READ_MEDIA_KEY_BLOCK {
            transferSize += AACS_MKB_PACK_SIZE;
        }

        if transferSize > (*(DeviceExtension)).ScratchContext.ScratchBufferSize as u32 {
            status = (0xC00000E5u32 as i32); // STATUS_INTERNAL_ERROR
        }
    }

    if status >= 0 {
        let mut rmdBlockNumber: u8 = 0;
        let mut sendChangedCommand = true;
        let mut shouldRetry = true;
        let mut cdb: CDB = core::mem::zeroed();

        ScratchBuffer_BeginUse(DeviceExtension);

        RtlZeroMemory(&raw mut cdb as *mut core::ffi::c_void, core::mem::size_of::<CDB>());
        cdb.READ_DVD_STRUCTURE.as_mut().OperationCode = SCSIOP_READ_DVD_STRUCTURE;
        cdb.READ_DVD_STRUCTURE.as_mut().LayerNumber = *layerNumber;
        cdb.READ_DVD_STRUCTURE.as_mut().Format = 0x83;
        cdb.READ_DVD_STRUCTURE.as_mut().AllocationLength[0] = ((transferSize as u32) >> 8) as u8;
        cdb.READ_DVD_STRUCTURE.as_mut().AllocationLength[1] = (transferSize as u32) as u8;

        while sendChangedCommand {
            cdb.READ_DVD_STRUCTURE.as_mut().RMDBlockNumber[3] = rmdBlockNumber;

            if shouldRetry {
                status = ScratchBuffer_ExecuteCdb(
                    DeviceExtension,
                    Request,
                    transferSize,
                    1u8,
                    &raw const cdb as *const core::ffi::c_void,
                    12,
                );
            }

            #[cfg(feature = "ENABLE_AACS_TESTING")]
            {
                if RequestParameters.Parameters.DeviceIoControl.as_ref().IoControlCode == IOCTL_AACS_READ_MEDIA_KEY_BLOCK_SIZE {
                    const RESULTS: [u8; 4] = [0x80, 0x02, 0x00, 0x02];
                    RtlCopyMemory(
                        (*(DeviceExtension)).ScratchContext.ScratchBuffer as *mut core::ffi::c_void,
                        RESULTS.as_ptr() as *const core::ffi::c_void,
                        RESULTS.len(),
                    );
                    status = 0; // STATUS_SUCCESS
                } else if RequestParameters.Parameters.DeviceIoControl.as_ref().IoControlCode == IOCTL_AACS_READ_MEDIA_KEY_BLOCK {
                    const RESULTS: [u8; 4] = [0x80, 0x02, 0x00, 0x02];
                    const DEFAULT_FILL: u8 = 0x30;
                    core::ptr::write_bytes(
                        (*(DeviceExtension)).ScratchContext.ScratchBuffer as *mut u8,
                        DEFAULT_FILL,
                        0x8004,
                    );
                    RtlCopyMemory(
                        (*(DeviceExtension)).ScratchContext.ScratchBuffer as *mut core::ffi::c_void,
                        RESULTS.as_ptr() as *const core::ffi::c_void,
                        RESULTS.len(),
                    );
                    status = 0; // STATUS_SUCCESS
                }
            }

            if status >= 0 {
                let header: *mut DVD_DESCRIPTOR_HEADER =
                    (*(DeviceExtension)).ScratchContext.ScratchBuffer as *mut DVD_DESCRIPTOR_HEADER;
                let thisPackNumber: u8 = cdb.READ_DVD_STRUCTURE.as_ref().RMDBlockNumber[3];
                let mut otherPacks: u8 = (*header).Reserved[1];

                if otherPacks == 0 {
                    TracePrint!(
                        TRACE_LEVEL_WARNING,
                        TRACE_FLAG_INIT,
                        c"AACS: Device is reporting zero total packs (invalid)\n".as_ptr() as *const i8,
                    );
                    *DataLength = 0;
                    status = STATUS_IO_DEVICE_ERROR;
                } else {
                    otherPacks -= 1;

                    if RequestParameters.Parameters.DeviceIoControl.as_ref().IoControlCode == IOCTL_AACS_READ_MEDIA_KEY_BLOCK_SIZE {
                        if otherPacks != thisPackNumber {
                            RtlZeroMemory(
                                (*(DeviceExtension)).ScratchContext.ScratchBuffer as *mut core::ffi::c_void,
                                core::mem::size_of::<READ_DVD_STRUCTURES_HEADER>(),
                            );

                            rmdBlockNumber = otherPacks;
                            transferSize = core::mem::size_of::<READ_DVD_STRUCTURES_HEADER>() as u32;
                            ScratchBuffer_ResetItems(DeviceExtension, 1u8);
                            sendChangedCommand = true;
                            shouldRetry = true;
                        } else {
                            REVERSE_SHORT(&raw mut (*header).Length as *mut u16);
                            if (*header).Length
                                < (core::mem::size_of::<DVD_DESCRIPTOR_HEADER>() as u16)
                                    - (core::mem::offset_of!(DVD_DESCRIPTOR_HEADER, Length) as u16
                                        + core::mem::size_of::<u16>() as u16)
                            {
                                *DataLength = 0;
                                status = STATUS_IO_DEVICE_ERROR;
                            } else {
                                let mut totalSize: u32 = (*header).Length as u32;
                                totalSize -= core::mem::size_of::<DVD_DESCRIPTOR_HEADER>() as u32
                                    - (core::mem::offset_of!(DVD_DESCRIPTOR_HEADER, Length) as u32
                                        + core::mem::size_of::<u16>() as u32);
                                totalSize += (otherPacks as u32) * AACS_MKB_PACK_SIZE;

                                *(outputBuffer as *mut u32) = totalSize;
                                *DataLength = core::mem::size_of::<u32>();
                                status = 0; // STATUS_SUCCESS
                            }
                            sendChangedCommand = false;
                            shouldRetry = false;
                        }
                    } else if RequestParameters.Parameters.DeviceIoControl.as_ref().IoControlCode == IOCTL_AACS_READ_MEDIA_KEY_BLOCK {
                        REVERSE_SHORT(&raw mut (*header).Length as *mut u16);
                        if (*header).Length
                            < (core::mem::size_of::<DVD_DESCRIPTOR_HEADER>() as u16)
                                - (core::mem::offset_of!(DVD_DESCRIPTOR_HEADER, Length) as u16
                                    + core::mem::size_of::<u16>() as u16)
                        {
                            *DataLength = 0;
                            status = STATUS_IO_DEVICE_ERROR;
                        } else {
                            let mut totalSize: u32 = (*header).Length as u32;
                            totalSize -= core::mem::size_of::<DVD_DESCRIPTOR_HEADER>() as u32
                                - (core::mem::offset_of!(DVD_DESCRIPTOR_HEADER, Length) as u32
                                    + core::mem::size_of::<u16>() as u32);

                            NT_ASSERT!((totalSize == AACS_MKB_PACK_SIZE || thisPackNumber == otherPacks) as i32);

                            let originalBufferSize: u32 =
                                RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength as u32;
                            if originalBufferSize < totalSize + (AACS_MKB_PACK_SIZE * (thisPackNumber as u32)) {
                                *DataLength = (((otherPacks as u32) + 1) * AACS_MKB_PACK_SIZE) as usize;
                                status = -1073741789; // STATUS_BUFFER_TOO_SMALL
                            } else {
                                let whereToCopy = (outputBuffer as *mut u8)
                                    .add((AACS_MKB_PACK_SIZE as usize) * (thisPackNumber as usize));
                                RtlCopyMemory(
                                    whereToCopy as *mut core::ffi::c_void,
                                    (*header).Data.as_ptr() as *const core::ffi::c_void,
                                    totalSize as usize,
                                );
                                *DataLength = totalSize as usize
                                    + (AACS_MKB_PACK_SIZE as usize) * (thisPackNumber as usize);
                                status = 0; // STATUS_SUCCESS

                                if thisPackNumber != otherPacks {
                                    RtlZeroMemory(
                                        (*(DeviceExtension)).ScratchContext.ScratchBuffer as *mut core::ffi::c_void,
                                        core::mem::size_of::<READ_DVD_STRUCTURES_HEADER>(),
                                    );

                                    // Preserve exact C side-effect: post-increment, assign old value to rmdBlockNumber
                                    let old_val = cdb.READ_DVD_STRUCTURE.as_ref().RMDBlockNumber[3];
                                    cdb.READ_DVD_STRUCTURE.as_mut().RMDBlockNumber[3] += 1;
                                    rmdBlockNumber = old_val;

                                    transferSize =
                                        AACS_MKB_PACK_SIZE + core::mem::size_of::<READ_DVD_STRUCTURES_HEADER>() as u32;
                                    ScratchBuffer_ResetItems(DeviceExtension, 0u8);
                                    sendChangedCommand = true;
                                    shouldRetry = true;
                                } else {
                                    sendChangedCommand = false;
                                }
                            }
                        }
                    }
                }
            }

            if status < 0 {
                sendChangedCommand = false;
            }
        }

        ScratchBuffer_EndUse(DeviceExtension);
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn DeviceHandleAacsStartSession(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Request: WDFREQUEST,
    RequestParameters: WDF_REQUEST_PARAMETERS,
    DataLength: *mut usize,
) -> NTSTATUS {
    let mut status: NTSTATUS;
    let mut sessionId: *mut DVD_SESSION_ID = core::ptr::null_mut();
    // PAGED_CODE();

    *DataLength = 0;

    status = WdfRequestRetrieveOutputBuffer(
        Request,
        RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength,
        core::ptr::addr_of_mut!(sessionId) as *mut *mut core::ffi::c_void,
        core::ptr::null_mut(),
    );

    if NT_SUCCESS(status) {
        let mut dataTransferLength: u32 = (core::mem::size_of::<CDVD_KEY_HEADER>() + core::mem::size_of::<CDVD_REPORT_AGID_DATA>()) as u32;
        let mut cdb: CDB = core::mem::zeroed();

        ScratchBuffer_BeginUse(DeviceExtension);

        RtlZeroMemory(core::ptr::addr_of_mut!(cdb) as *mut core::ffi::c_void, core::mem::size_of::<CDB>() as usize);

        cdb.REPORT_KEY.as_mut().OperationCode = SCSIOP_REPORT_KEY;
        cdb.AsByte.as_mut()[7] = 0x02; // AACS key class
        cdb.REPORT_KEY.as_mut().AllocationLength[0] = (dataTransferLength >> 8) as u8;
        cdb.REPORT_KEY.as_mut().AllocationLength[1] = (dataTransferLength & 0xff) as u8;
        cdb.REPORT_KEY.as_mut().set_KeyFormat(0x00); // DVD_REPORT_AGID?

        status = ScratchBuffer_ExecuteCdb(DeviceExtension, Request, dataTransferLength, 1, core::ptr::addr_of_mut!(cdb) as *const core::ffi::c_void, 12);

        #[cfg(feature = "ENABLE_AACS_TESTING")]
        {
            const RESULTS: [u8; 8] = [0x00, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0xC0];
            RtlCopyMemory((*DeviceExtension).ScratchContext.ScratchBuffer as *mut core::ffi::c_void, RESULTS.as_ptr() as *const core::ffi::c_void, RESULTS.len());
            status = STATUS_SUCCESS;
        }

        if NT_SUCCESS(status) {
            let keyHeader: *mut CDVD_KEY_HEADER = (*DeviceExtension).ScratchContext.ScratchBuffer as *mut CDVD_KEY_HEADER;
            let keyData: *mut CDVD_REPORT_AGID_DATA = (*keyHeader).Data.as_ptr() as *mut CDVD_REPORT_AGID_DATA;
            *sessionId = (*keyData).AGID() as DVD_SESSION_ID;
            *DataLength = core::mem::size_of::<DVD_SESSION_ID>();
        }

        ScratchBuffer_EndUse(DeviceExtension);
    }

    return status;
}
#[no_mangle]
pub unsafe extern "C" fn DeviceHandleAacsEndSession(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Request: WDFREQUEST,
    RequestParameters: WDF_REQUEST_PARAMETERS,
    DataLength: *mut usize,
) -> NTSTATUS {
    // PAGED_CODE();
    let mut status: NTSTATUS;
    let mut sessionId: *mut u8 = core::ptr::null_mut(); // DVD_SESSION_ID is u8
    *DataLength = 0;

    status = WdfRequestRetrieveInputBuffer(
        Request,
        RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength as usize,
        &mut sessionId as *mut *mut u8 as *mut *mut core::ffi::c_void,
        core::ptr::null_mut::<usize>(),
    );

    if NT_SUCCESS(status) {
        let mut transferSize: u32 = 0;
        let mut cdb: CDB = core::mem::zeroed();
        let mut currentSession: u8;
        let mut limitSession: u8;

        if *sessionId == DVD_END_ALL_SESSIONS {
            currentSession = 0;
            limitSession = MAX_COPY_PROTECT_AGID - 1;
        } else {
            currentSession = *sessionId;
            limitSession = *sessionId;
        }

        ScratchBuffer_BeginUse(DeviceExtension);

        loop {
            RtlZeroMemory(
                &mut cdb as *mut CDB as *mut core::ffi::c_void,
                core::mem::size_of::<CDB>(),
            );
            // Set up the CDB
            cdb.SEND_KEY.as_mut().OperationCode = SCSIOP_SEND_KEY;
            cdb.AsByte.as_mut()[7] = 0x02; // AACS key class
            cdb.SEND_KEY.as_mut().set_AGID((currentSession) as u8);
            cdb.SEND_KEY.as_mut().set_KeyFormat((DVD_INVALIDATE_AGID) as u8);

            status = ScratchBuffer_ExecuteCdb(
                DeviceExtension,
                Request,
                transferSize,
                0, // FALSE
                &cdb as *const CDB as *const core::ffi::c_void,
                12,
            );

            currentSession += 1;

            if !(currentSession <= limitSession && NT_SUCCESS(status)) {
                break;
            }
        }

        #[cfg(feature = "ENABLE_AACS_TESTING")]
        {
            status = STATUS_SUCCESS;
        }

        ScratchBuffer_EndUse(DeviceExtension);
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn DeviceHandleAacsSendCertificate(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Request: WDFREQUEST,
    RequestParameters: WDF_REQUEST_PARAMETERS,
    DataLength: *mut usize,
) -> NTSTATUS {
    let mut status: NTSTATUS;
    let mut input: *mut AACS_SEND_CERTIFICATE = core::ptr::null_mut();

    PAGED_CODE!();

    *DataLength = 0;

    status = WdfRequestRetrieveInputBuffer(
        Request,
        RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength as usize,
        core::ptr::addr_of_mut!(input) as *mut *mut core::ffi::c_void,
        core::ptr::null_mut::<usize>(),
    );

    if status >= 0 {
        let mut data_transfer_length: u32 = (core::mem::size_of::<CDVD_KEY_HEADER>()
            + core::mem::size_of::<AACS_CERTIFICATE>()) as u32;
        let mut cdb: CDB = core::mem::zeroed();

        ScratchBuffer_BeginUse(DeviceExtension);

        // copy the input buffer to the data buffer for the transfer
        {
            let header = (*DeviceExtension).ScratchContext.ScratchBuffer as *mut CDVD_KEY_HEADER;
            let mut tmp: u32 = data_transfer_length;

            // expand RTL_SIZEOF_THROUGH_FIELD(CDVD_KEY_HEADER, DataLength)
            let size_through_data_length = core::mem::offset_of!(CDVD_KEY_HEADER, DataLength)
                + core::mem::size_of::<[u8; 2]>();
            tmp -= size_through_data_length as u32;

            (*header).DataLength[0] = (tmp >> 8) as u8;
            (*header).DataLength[1] = tmp as u8;

            core::ptr::copy_nonoverlapping(
                core::ptr::addr_of!((*input).Certificate) as *const u8,
                (*header).Data.as_mut_ptr() as *mut u8,
                core::mem::size_of::<AACS_CERTIFICATE>(),
            );
        }

        // RtlZeroMemory(&cdb, sizeof(CDB))
        core::ptr::write_bytes(
            core::ptr::addr_of_mut!(cdb) as *mut u8,
            0u8,
            core::mem::size_of::<CDB>(),
        );

        // Set up the CDB
        cdb.SEND_KEY.as_mut().OperationCode = SCSIOP_SEND_KEY;
        cdb.AsByte.as_mut()[7] = 0x02; // AACS key class
        cdb.SEND_KEY.as_mut().ParameterListLength[0] = (data_transfer_length >> 8) as u8;
        cdb.SEND_KEY.as_mut().ParameterListLength[1] = data_transfer_length as u8;
        cdb.SEND_KEY.as_mut().set_AGID((*input).SessionId as u8);
        cdb.SEND_KEY.as_mut().set_KeyFormat(0x01); // Send Host Challenge Certificate

        status = ScratchBuffer_ExecuteCdb(
            DeviceExtension,
            Request,
            data_transfer_length,
            0u8, // FALSE
            core::ptr::addr_of_mut!(cdb) as *const core::ffi::c_void,
            12u32,
        );

        // #ifdef ENABLE_AACS_TESTING — must NOT mask the real CDB status in
        // production (was unconditionally forcing SUCCESS). Found by semdiff.
        #[cfg(feature = "ENABLE_AACS_TESTING")]
        { status = STATUS_SUCCESS; }

        if status >= 0 {
            *DataLength = 0;
        }

        ScratchBuffer_EndUse(DeviceExtension);
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn DeviceHandleAacsGetCertificate(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Request: WDFREQUEST,
    RequestParameters: WDF_REQUEST_PARAMETERS,
    DataLength: *mut usize,
) -> NTSTATUS {
    let mut status: NTSTATUS;
    let mut input: *mut DVD_SESSION_ID = core::ptr::null_mut();
    let mut outputBuffer: *mut core::ffi::c_void = core::ptr::null_mut();

    PAGED_CODE!();

    *DataLength = 0;

    status = WdfRequestRetrieveInputBuffer(
        Request,
        RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength as usize,
        &mut input as *mut *mut DVD_SESSION_ID as *mut *mut core::ffi::c_void,
        core::ptr::null_mut(),
    );

    if NT_SUCCESS(status) {
        status = WdfRequestRetrieveOutputBuffer(
            Request,
            RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength as usize,
            &mut outputBuffer as *mut *mut core::ffi::c_void as *mut *mut core::ffi::c_void,
            core::ptr::null_mut(),
        );
    }

    if NT_SUCCESS(status) {
        let data_transfer_length: u32 = core::mem::size_of::<CDVD_KEY_HEADER>() as u32
            + core::mem::size_of::<AACS_CERTIFICATE>() as u32;
        let mut cdb: CDB = core::mem::zeroed();

        // RtlZeroMemory(&cdb, sizeof(CDB))
        core::ptr::write_bytes(
            core::ptr::addr_of_mut!(cdb) as *mut u8,
            0u8,
            core::mem::size_of::<CDB>(),
        );

        ScratchBuffer_BeginUse(DeviceExtension);

        cdb.REPORT_KEY.as_mut().OperationCode = SCSIOP_REPORT_KEY;
        cdb.AsByte.as_mut()[7] = 0x02u8;
        cdb.REPORT_KEY.as_mut().AllocationLength[0] = (data_transfer_length >> 8) as u8;
        cdb.REPORT_KEY.as_mut().AllocationLength[1] = (data_transfer_length & 0xFF) as u8;
        cdb.REPORT_KEY.as_mut().set_AGID((*input) as u8);
        cdb.REPORT_KEY.as_mut().set_KeyFormat((0x01u8) as u8);

        status = ScratchBuffer_ExecuteCdb(
            DeviceExtension,
            Request,
            data_transfer_length,
            1, // TRUE
            &cdb as *const CDB as *const core::ffi::c_void,
            12,
        );

        // #ifdef ENABLE_AACS_TESTING (C aacs.c:651) — test-only override, off in production
        #[cfg(feature = "ENABLE_AACS_TESTING")]
        {
            const RESULTS: [u8; 4] = [0x00, 0x72, 0x00, 0x00];
            const DEFAULT_FILL: u8 = 0x31;

            core::ptr::write_bytes(
                (*DeviceExtension).ScratchContext.ScratchBuffer as *mut u8,
                DEFAULT_FILL,
                0x0074usize,
            );
            core::ptr::copy_nonoverlapping(
                &RESULTS as *const [u8; 4] as *const u8,
                (*DeviceExtension).ScratchContext.ScratchBuffer as *mut u8,
                4usize, // SIZEOF_ARRAY(RESULTS)
            );
            status = STATUS_SUCCESS;
        }

        if NT_SUCCESS(status) {
            let header: *mut DVD_DESCRIPTOR_HEADER =
                (*DeviceExtension).ScratchContext.ScratchBuffer as *mut DVD_DESCRIPTOR_HEADER;
            let data_length_to_copy: u32 = core::mem::size_of::<AACS_CERTIFICATE>() as u32;

            REVERSE_SHORT(&mut (*header).Length as *mut u16);

            // RTL_SIZEOF_THROUGH_FIELD(DVD_DESCRIPTOR_HEADER, Length) = core::mem::offset_of!(Length) + size_of::<u16>()
            let min_length: usize = core::mem::size_of::<DVD_DESCRIPTOR_HEADER>()
                - (core::mem::offset_of!(DVD_DESCRIPTOR_HEADER, Length)
                    + core::mem::size_of::<u16>());
            let min_length_u16 = min_length as u16;

            if (*header).Length < min_length_u16 {
                *DataLength = 0;
                status = STATUS_IO_DEVICE_ERROR;
            }

            if NT_SUCCESS(status) {
                (*header).Length -= min_length_u16;

                if (*header).Length != data_length_to_copy as u16 {
                    *DataLength = 0;
                    status = STATUS_IO_DEVICE_ERROR;
                }
            }

            if NT_SUCCESS(status) {
                core::ptr::copy_nonoverlapping(
                    (*header).Data.as_ptr() as *const u8,
                    outputBuffer as *mut u8,
                    data_length_to_copy as usize,
                );
                *DataLength = data_length_to_copy as usize;
            }
        }

        ScratchBuffer_EndUse(DeviceExtension);
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn DeviceHandleAacsGetChallengeKey(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Request: WDFREQUEST,
    RequestParameters: WDF_REQUEST_PARAMETERS,
    DataLength: *mut usize,
) -> i32 {
    // AacsGetChallengeKey

    let mut status: i32;
    let mut input: *mut DVD_SESSION_ID = core::ptr::null_mut();
    let mut outputBuffer: *mut core::ffi::c_void = core::ptr::null_mut();

    // PAGED_CODE();

    *DataLength = 0;

    status = WdfRequestRetrieveInputBuffer(
        Request,
        RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
        &mut input as *mut _ as *mut *mut core::ffi::c_void,
        core::ptr::null_mut(),
    );

    if status >= 0 {
        status = WdfRequestRetrieveOutputBuffer(
            Request,
            RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength,
            &mut outputBuffer as *mut _ as *mut *mut core::ffi::c_void,
            core::ptr::null_mut(),
        );
    }

    if status >= 0 {
        let mut dataTransferLength: u32 =
            (core::mem::size_of::<CDVD_KEY_HEADER>() + core::mem::size_of::<AACS_CHALLENGE_KEY>()) as u32;
        let mut cdb: CDB = core::mem::zeroed();

        ScratchBuffer_BeginUse(DeviceExtension);

        core::ptr::write_bytes(
            (core::ptr::addr_of_mut!(cdb).cast::<u8>()) as *mut u8,
            0,
            core::mem::size_of::<CDB>());

        // Set up the CDB
        cdb.REPORT_KEY.as_mut().OperationCode = SCSIOP_REPORT_KEY;
        cdb.AsByte.as_mut()[7] = 0x02;
        cdb.REPORT_KEY.as_mut().AllocationLength[0] = (dataTransferLength >> 8) as u8;
        cdb.REPORT_KEY.as_mut().AllocationLength[1] = dataTransferLength as u8;
        cdb.REPORT_KEY.as_mut().set_AGID(*input as u8);
        cdb.REPORT_KEY.as_mut().set_KeyFormat(0x02);

        status = ScratchBuffer_ExecuteCdb(
            DeviceExtension,
            Request,
            dataTransferLength,
            1, // TRUE
            &mut cdb as *mut CDB as *const core::ffi::c_void,
            12,
        );

        if cfg!(feature = "ENABLE_AACS_TESTING") {
            const RESULTS: [u8; 4] = [0x00, 0x52, 0x00, 0x00];
            const DEFAULT_FILL: u8 = 0x32;
            // RtlFillMemory inline
            core::ptr::write_bytes(
                ((*DeviceExtension).ScratchContext.ScratchBuffer) as *mut u8,
                DEFAULT_FILL,
                0x0054);
            // RtlCopyMemory
            core::ptr::copy_nonoverlapping(
                RESULTS.as_ptr(),
                (*DeviceExtension).ScratchContext.ScratchBuffer as *mut u8,
                4,
            );
            status = STATUS_SUCCESS;
        }

        if status >= 0 {
            let header = (*DeviceExtension).ScratchContext.ScratchBuffer as *mut DVD_DESCRIPTOR_HEADER;
            let mut dataLengthToCopy: u32 = core::mem::size_of::<AACS_CHALLENGE_KEY>() as u32;

            REVERSE_SHORT(&mut (*header).Length as *mut u16);

            let min_length = core::mem::size_of::<DVD_DESCRIPTOR_HEADER>()
                - (core::mem::offset_of!(DVD_DESCRIPTOR_HEADER, Length)
                    + core::mem::size_of::<u16>());

            // exit if getting invalid data from the drive
            if (*header).Length < min_length as u16 {
                *DataLength = 0;
                status = STATUS_IO_DEVICE_ERROR;
            }

            if status >= 0 {
                // adjust data length to reflect only the addition data
                (*header).Length -= min_length as u16;

                // exit if the drive is returning an unexpected data size
                if (*header).Length != dataLengthToCopy as u16 {
                    *DataLength = 0;
                    status = STATUS_IO_DEVICE_ERROR;
                }
            }

            if status >= 0 {
                // else copy the data to the user's buffer
                core::ptr::copy_nonoverlapping(
                    core::ptr::addr_of!((*header).Data) as *const u8,
                    outputBuffer as *mut u8,
                    dataLengthToCopy as usize,
                );
                *DataLength = dataLengthToCopy as usize;
            }
        }

        ScratchBuffer_EndUse(DeviceExtension);
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn DeviceHandleSendChallengeKey(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Request: WDFREQUEST,
    RequestParameters: WDF_REQUEST_PARAMETERS,
    DataLength: *mut usize,
) -> NTSTATUS {
    // PAGED_CODE();
    let mut status: NTSTATUS;
    let mut input: *mut AACS_SEND_CHALLENGE_KEY = core::ptr::null_mut();

    *DataLength = 0;

    let in_len = RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength as usize;
    status = WdfRequestRetrieveInputBuffer(
        Request,
        in_len,
        core::ptr::addr_of_mut!(input) as *mut *mut core::ffi::c_void,
        core::ptr::null_mut(),
    );

    if NT_SUCCESS(status) {
        let dataTransferLength: u32 = core::mem::size_of::<CDVD_KEY_HEADER>() as u32
            + core::mem::size_of::<AACS_CHALLENGE_KEY>() as u32;
        let mut cdb: CDB = core::mem::zeroed();

        ScratchBuffer_BeginUse(DeviceExtension);

        {
            let header: *mut CDVD_KEY_HEADER =
                (*DeviceExtension).ScratchContext.ScratchBuffer as *mut CDVD_KEY_HEADER;
            let mut tmp: u32 = dataTransferLength;
            tmp -= (core::mem::offset_of!(CDVD_KEY_HEADER, DataLength) as u32 + 2);

            (*header).DataLength[0] = (tmp >> 8) as u8;
            (*header).DataLength[1] = (tmp >> 0) as u8;

            core::ptr::copy_nonoverlapping(
                &(*input).ChallengeKey as *const AACS_CHALLENGE_KEY as *const u8,
                &raw mut (*header).Data as *mut u8,
                core::mem::size_of::<AACS_CHALLENGE_KEY>(),
            );
        }

        RtlZeroMemory(
            core::ptr::addr_of_mut!(cdb) as *mut core::ffi::c_void,
            core::mem::size_of::<CDB>(),
        );

        // Set up the CDB
        cdb.SEND_KEY.as_mut().OperationCode = SCSIOP_SEND_KEY;
        cdb.AsByte.as_mut()[7] = 0x02;
        cdb.SEND_KEY.as_mut().ParameterListLength[0] = (dataTransferLength >> 8) as u8;
        cdb.SEND_KEY.as_mut().ParameterListLength[1] = (dataTransferLength >> 0) as u8;
        cdb.SEND_KEY.as_mut().set_AGID(((*input).SessionId) as u8);
        cdb.SEND_KEY.as_mut().set_KeyFormat(0x02); // Send Host Challenge Certificate

        status = ScratchBuffer_ExecuteCdb(
            DeviceExtension,
            Request,
            dataTransferLength,
            0, // FALSE
            &cdb as *const CDB as *const core::ffi::c_void,
            12u32,
        );

        // #ifdef ENABLE_AACS_TESTING
        #[cfg(feature = "ENABLE_AACS_TESTING")]
        {
            status = STATUS_SUCCESS;
        }

        if NT_SUCCESS(status) {
            *DataLength = 0;
        }

        ScratchBuffer_EndUse(DeviceExtension);
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn DeviceHandleReadVolumeId(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Request: WDFREQUEST,
    RequestParameters: WDF_REQUEST_PARAMETERS,
    DataLength: *mut usize,
) -> NTSTATUS {
    // AacsReadVolumeID
    let mut status: NTSTATUS; // STATUS_SUCCESS
    let mut input: *mut u8 = core::ptr::null_mut();
    let mut output_buffer: *mut core::ffi::c_void = core::ptr::null_mut();

    PAGED_CODE!();

    *DataLength = 0;

    status = WdfRequestRetrieveInputBuffer(
        Request,
        RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength as usize,
        &mut input as *mut *mut u8 as *mut *mut core::ffi::c_void,
        core::ptr::null_mut(),
    );

    if status >= 0 {
        status = WdfRequestRetrieveOutputBuffer(
            Request,
            RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength as usize,
            &mut output_buffer as *mut *mut core::ffi::c_void as *mut *mut core::ffi::c_void,
            core::ptr::null_mut(),
        );
    }

    if status >= 0 {
        let data_transfer_length: u32 = (core::mem::size_of::<CDVD_KEY_HEADER>() + core::mem::size_of::<AACS_VOLUME_ID>()) as u32;
        let mut cdb: [u8; 12] = [0u8; 12];

        ScratchBuffer_BeginUse(DeviceExtension);

        // RtlZeroMemory(&cdb, sizeof(CDB)) is already done by initialization above,
        // but we preserve the call explicitly for exact transliteration.
        core::ptr::write_bytes((cdb.as_mut_ptr()) as *mut u8, 0, core::mem::size_of::<[u8; 12]>());

        // Set up the CDB
        cdb[0] = SCSIOP_READ_DVD_STRUCTURE;
        cdb[1] = 0x80; // Return the AACS volumeID
        cdb[2] = (data_transfer_length >> 8) as u8;
        cdb[3] = (data_transfer_length >> 0) as u8;
        cdb[4] = *input;

        status = ScratchBuffer_ExecuteCdb(
            DeviceExtension,
            Request,
            data_transfer_length,
            TRUE as u8,
            cdb.as_mut_ptr() as *const core::ffi::c_void,
            12,
        );

        #[cfg(feature = "ENABLE_AACS_TESTING")]
        {
            let results: [u8; 4] = [0x00, 0x22, 0x00, 0x00];
            // RtlFillMemory expanded: fill 0x24 bytes with 0x33
            core::ptr::write_bytes(
                (*DeviceExtension).ScratchContext.ScratchBuffer as *mut u8,
                0x33,
                0x0024,
            );
            // RtlCopyMemory: copy `results` into ScratchBuffer
            core::ptr::copy_nonoverlapping(
                results.as_ptr(),
                (*DeviceExtension).ScratchContext.ScratchBuffer as *mut u8,
                core::mem::size_of_val(&results),
            );
            status = 0; // STATUS_SUCCESS
        }

        if status >= 0 {
            let header: *mut DVD_DESCRIPTOR_HEADER =
                (*DeviceExtension).ScratchContext.ScratchBuffer as *mut DVD_DESCRIPTOR_HEADER;
            let data_length_to_copy: u16 = core::mem::size_of::<AACS_VOLUME_ID>() as u16;

            // make length field native byte ordering
            REVERSE_SHORT(&mut (*header).Length);

            // exit if getting invalid data from the drive
            let field_offset_length = core::mem::offset_of!(DVD_DESCRIPTOR_HEADER, Length);
            let size_through_length = field_offset_length + core::mem::size_of::<u16>();
            if (*header).Length < (core::mem::size_of::<DVD_DESCRIPTOR_HEADER>() - size_through_length) as u16 {
                *DataLength = 0;
                status = STATUS_IO_DEVICE_ERROR;
            }

            if status >= 0 {
                // adjust data length to reflect only the addition data
                (*header).Length -= (core::mem::size_of::<DVD_DESCRIPTOR_HEADER>() - size_through_length) as u16;

                // exit if the drive is returning an unexpected data size
                if (*header).Length != data_length_to_copy {
                    *DataLength = 0;
                    status = STATUS_IO_DEVICE_ERROR;
                }
            }

            if status >= 0 {
                // else copy the data to the user's buffer
                core::ptr::copy_nonoverlapping(
                    (*header).Data.as_ptr(),
                    output_buffer as *mut u8,
                    data_length_to_copy as usize,
                );
                *DataLength = data_length_to_copy as usize;
            }
        }

        ScratchBuffer_EndUse(DeviceExtension);
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn DeviceHandleAacsReadSerialNumber(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Request: *mut core::ffi::c_void,
    RequestParameters: WDF_REQUEST_PARAMETERS,
    DataLength: *mut usize,
) -> i32 {
    unsafe {
        let mut status: i32; // STATUS_SUCCESS
        let mut input: *mut u8 = core::ptr::null_mut();
        let mut outputBuffer: *mut u8 = core::ptr::null_mut();

        // PAGED_CODE();

        *DataLength = 0;

        status = WdfRequestRetrieveInputBuffer(
            Request as WDFREQUEST,
            RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
            &raw mut input as *mut *mut core::ffi::c_void,
            core::ptr::null_mut(),
        );

        // NT_SUCCESS is `status >= 0`
        if status >= 0 {
            status = WdfRequestRetrieveOutputBuffer(
                Request as WDFREQUEST,
                RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength,
                &raw mut outputBuffer as *mut *mut core::ffi::c_void,
                core::ptr::null_mut(),
            );
        }

        if status >= 0 {
            let dataTransferLength: u32 =
                (core::mem::size_of::<CDVD_KEY_HEADER>() + core::mem::size_of::<AACS_SERIAL_NUMBER>())
                    as u32;
            let mut cdb: CDB = core::mem::zeroed();

            ScratchBuffer_BeginUse(DeviceExtension);

            // RtlZeroMemory(&cdb, sizeof(CDB));
            core::ptr::write_bytes(
                &raw mut cdb as *mut u8,
                0,
                core::mem::size_of::<CDB>(),
            );

            // Set up the CDB
            cdb.READ_DVD_STRUCTURE.as_mut().OperationCode = SCSIOP_READ_DVD_STRUCTURE;
            cdb.READ_DVD_STRUCTURE.as_mut().Format = 0x81;
            cdb.READ_DVD_STRUCTURE.as_mut().AllocationLength[0] = (dataTransferLength >> 8) as u8;
            cdb.READ_DVD_STRUCTURE.as_mut().AllocationLength[1] = (dataTransferLength >> 0) as u8;
            cdb.READ_DVD_STRUCTURE.as_mut().set_AGID((*input) as u8);

            status = ScratchBuffer_ExecuteCdb(
                DeviceExtension,
                Request as WDFREQUEST,
                dataTransferLength,
                1, // TRUE
                &cdb as *const CDB as *const core::ffi::c_void,
                12,
            );

            #[cfg(feature = "ENABLE_AACS_TESTING")]
            {
                const RESULTS: [u8; 4] = [0x00, 0x22, 0x00, 0x00];
                const DEFAULT_FILL: u8 = 0x34;
                let scratch_ptr = (*DeviceExtension).ScratchContext.ScratchBuffer;
                core::ptr::write_bytes((scratch_ptr) as *mut u8, DEFAULT_FILL, 0x0024);
                core::ptr::copy_nonoverlapping(RESULTS.as_ptr(), scratch_ptr, RESULTS.len());
                status = 0; // STATUS_SUCCESS
            }

            if status >= 0 {
                let header =
                    (*DeviceExtension).ScratchContext.ScratchBuffer as *mut DVD_DESCRIPTOR_HEADER;
                let dataLengthToCopy: u32 = core::mem::size_of::<AACS_SERIAL_NUMBER>() as u32;

                // make length field native byte ordering
                REVERSE_SHORT(&raw mut (*header).Length);

                // RTL_SIZEOF_THROUGH_FIELD(DVD_DESCRIPTOR_HEADER, Length) expansion
                let descriptor_header_size = core::mem::size_of::<DVD_DESCRIPTOR_HEADER>();
                let offset_of_length = core::mem::offset_of!(DVD_DESCRIPTOR_HEADER, Length);
                let size_from_length = descriptor_header_size - offset_of_length;

                // exit if getting invalid data from the drive
                if (*header).Length < size_from_length as u16 {
                    *DataLength = 0;
                    status = STATUS_IO_DEVICE_ERROR;
                }

                if status >= 0 {
                    // adjust data length to reflect only the addition data
                    (*header).Length -= size_from_length as u16;

                    // exit if the drive is returning an unexpected data size
                    if (*header).Length != dataLengthToCopy as u16 {
                        *DataLength = 0;
                        status = STATUS_IO_DEVICE_ERROR;
                    }
                }

                if status >= 0 {
                    // copy the data to the user's buffer
                    core::ptr::copy_nonoverlapping(
                        &raw const (*header).Data as *const u8,
                        outputBuffer,
                        dataLengthToCopy as usize,
                    );
                    *DataLength = dataLengthToCopy as usize;
                }
            }

            ScratchBuffer_EndUse(DeviceExtension);
        }

        status
    }
}
#[no_mangle]
pub unsafe extern "C" fn DeviceHandleAacsReadMediaId(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Request: WDFREQUEST,
    RequestParameters: WDF_REQUEST_PARAMETERS,
    DataLength: *mut usize,
) -> NTSTATUS {
    let mut status: NTSTATUS;
    let mut input: *mut u8 = core::ptr::null_mut();
    let mut outputBuffer: *mut core::ffi::c_void = core::ptr::null_mut();

    // PAGED_CODE();

    *DataLength = 0;

    status = WdfRequestRetrieveInputBuffer(
        Request,
        RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
        core::ptr::addr_of_mut!(input) as *mut *mut core::ffi::c_void,
        core::ptr::null_mut(),
    );

    if NT_SUCCESS(status) {
        status = WdfRequestRetrieveOutputBuffer(
            Request,
            RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength,
            core::ptr::addr_of_mut!(outputBuffer) as *mut *mut core::ffi::c_void,
            core::ptr::null_mut(),
        );
    }

    if NT_SUCCESS(status) {
        let dataTransferLength: u32 =
            (core::mem::size_of::<CDVD_KEY_HEADER>() + core::mem::size_of::<AACS_MEDIA_ID>()) as u32;
        let mut cdb: CDB = core::mem::zeroed();

        ScratchBuffer_BeginUse(DeviceExtension);

        cdb.READ_DVD_STRUCTURE.as_mut().OperationCode = SCSIOP_READ_DVD_STRUCTURE;
        cdb.READ_DVD_STRUCTURE.as_mut().Format = 0x82u8;
        cdb.READ_DVD_STRUCTURE.as_mut().AllocationLength[0] = (dataTransferLength >> 8) as u8;
        cdb.READ_DVD_STRUCTURE.as_mut().AllocationLength[1] = (dataTransferLength >> 0) as u8;
        cdb.READ_DVD_STRUCTURE.as_mut().set_AGID(*input as u8);

        status = ScratchBuffer_ExecuteCdb(
            DeviceExtension,
            Request,
            dataTransferLength,
            1u8,
            &mut cdb as *mut CDB as *const core::ffi::c_void,
            12u32,
        );

        #[cfg(feature = "ENABLE_AACS_TESTING")]
        {
            let results: [u8; 4] = [0x00, 0x22, 0x00, 0x00];
            let defaultFill: u8 = 0x35; // '5'
            core::ptr::write_bytes(
                (*DeviceExtension).ScratchContext.ScratchBuffer as *mut u8,
                defaultFill,
                0x0024,
            );
            core::ptr::copy_nonoverlapping(
                results.as_ptr(),
                (*DeviceExtension).ScratchContext.ScratchBuffer as *mut u8,
                results.len(),
            );
            status = STATUS_SUCCESS;
        }

        if NT_SUCCESS(status) {
            let header: *mut DVD_DESCRIPTOR_HEADER =
                (*DeviceExtension).ScratchContext.ScratchBuffer as *mut DVD_DESCRIPTOR_HEADER;
            let dataLengthToCopy: usize = core::mem::size_of::<AACS_MEDIA_ID>();

            REVERSE_SHORT(&mut (*header).Length as *mut u16);

            let size_through_length =
                core::mem::offset_of!(DVD_DESCRIPTOR_HEADER, Length) + core::mem::size_of::<u16>();
            if ((*header).Length as usize) < core::mem::size_of::<DVD_DESCRIPTOR_HEADER>() - size_through_length {
                *DataLength = 0;
                status = STATUS_IO_DEVICE_ERROR;
            }

            if NT_SUCCESS(status) {
                (*header).Length -=
                    (core::mem::size_of::<DVD_DESCRIPTOR_HEADER>() - size_through_length) as u16;

                if (*header).Length as usize != dataLengthToCopy {
                    *DataLength = 0;
                    status = STATUS_IO_DEVICE_ERROR;
                }
            }

            if NT_SUCCESS(status) {
                core::ptr::copy_nonoverlapping(
                    core::ptr::addr_of!((*header).Data) as *const u8,
                    outputBuffer as *mut u8,
                    dataLengthToCopy,
                );
                *DataLength = dataLengthToCopy;
            }
        }

        ScratchBuffer_EndUse(DeviceExtension);
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn DeviceHandleAacsReadBindingNonce(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Request: WDFREQUEST,
    RequestParameters: WDF_REQUEST_PARAMETERS,
    DataLength: *mut SIZE_T,
) -> NTSTATUS {
    // AacsReadBindingNonce
    let mut status: NTSTATUS;
    let mut input: *mut AACS_READ_BINDING_NONCE = core::ptr::null_mut();
    let mut outputBuffer: *mut c_void = core::ptr::null_mut();

    // PAGED_CODE();

    *DataLength = 0;

    status = WdfRequestRetrieveInputBuffer(
        Request,
        RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
        core::ptr::addr_of_mut!(input) as *mut *mut c_void,
        core::ptr::null_mut(),
    );

    if NT_SUCCESS(status) {
        status = WdfRequestRetrieveOutputBuffer(
            Request,
            RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength,
            core::ptr::addr_of_mut!(outputBuffer) as *mut *mut c_void,
            core::ptr::null_mut(),
        );
    }

    if NT_SUCCESS(status) {
        let dataTransferLength: u32 =
            (core::mem::size_of::<CDVD_KEY_HEADER>() + core::mem::size_of::<AACS_BINDING_NONCE>())
                as u32;
        let mut cdb: CDB = core::mem::zeroed();
        ScratchBuffer_BeginUse(DeviceExtension);

        RtlZeroMemory(
            core::ptr::addr_of_mut!(cdb) as *mut c_void,
            core::mem::size_of::<CDB>(),
        );

        // Set up the CDB
        cdb.REPORT_KEY.as_mut().OperationCode = 0xA4; // SCSIOP_REPORT_KEY
        let start_lba = (*input).StartLba;
        cdb.REPORT_KEY.as_mut().LogicalBlockAddress[0] = (start_lba >> (3 * 8)) as u8;
        cdb.REPORT_KEY.as_mut().LogicalBlockAddress[1] = (start_lba >> (2 * 8)) as u8;
        cdb.REPORT_KEY.as_mut().LogicalBlockAddress[2] = (start_lba >> (1 * 8)) as u8;
        cdb.REPORT_KEY.as_mut().LogicalBlockAddress[3] = (start_lba >> (0 * 8)) as u8;
        cdb.AsByte.as_mut()[6] = (*input).NumberOfSectors as u8;
        cdb.AsByte.as_mut()[7] = 0x02; // AACS key class
        cdb.REPORT_KEY.as_mut().AllocationLength[0] = (dataTransferLength >> (8 * 1)) as u8;
        cdb.REPORT_KEY.as_mut().AllocationLength[1] = (dataTransferLength >> (8 * 0)) as u8;
        cdb.REPORT_KEY.as_mut().set_AGID((*input).SessionId as u8);
        cdb.REPORT_KEY.as_mut().set_KeyFormat(0x21); // Return an existing binding nonce

        status = ScratchBuffer_ExecuteCdb(
            DeviceExtension,
            Request,
            dataTransferLength,
            1, // TRUE
            core::ptr::addr_of!(cdb) as *const CDB as *const core::ffi::c_void,
            12,
        );

        #[cfg(feature = "ENABLE_AACS_TESTING")]
        {
            static results: [u8; 4] = [0x00, 0x22, 0x00, 0x00];
            static defaultFill: u8 = 0x36; // '6'
            core::ptr::write_bytes(
                ((*DeviceExtension).ScratchContext.ScratchBuffer) as *mut u8,
                defaultFill,
                0x0024);
            RtlCopyMemory(
                (*DeviceExtension).ScratchContext.ScratchBuffer as *mut c_void,
                results.as_ptr() as *const c_void,
                (core::mem::size_of_val(&results) / core::mem::size_of::<u8>()) as usize,
            );
            status = STATUS_SUCCESS;
        }

        if NT_SUCCESS(status) {
            let header: *mut DVD_DESCRIPTOR_HEADER =
                (*DeviceExtension).ScratchContext.ScratchBuffer as *mut DVD_DESCRIPTOR_HEADER;
            let dataLengthToCopy: u32 = core::mem::size_of::<AACS_BINDING_NONCE>() as u32;

            // make length field native byte ordering
            REVERSE_SHORT(core::ptr::addr_of_mut!((*header).Length));

            let len_offset = core::mem::offset_of!(DVD_DESCRIPTOR_HEADER, Length);
            let len_field_size = core::mem::size_of::<u16>();
            let size_through_length = len_offset + len_field_size;
            let min_length = core::mem::size_of::<DVD_DESCRIPTOR_HEADER>() - size_through_length;

            // exit if getting invalid data from the drive
            if ((*header).Length as usize) < min_length {
                *DataLength = 0;
                status = STATUS_IO_DEVICE_ERROR;
            }

            if NT_SUCCESS(status) {
                // adjust data length to reflect only the addition data
                let overhead = (core::mem::size_of::<DVD_DESCRIPTOR_HEADER>() - size_through_length) as u16;
                (*header).Length -= overhead;

                // exit if the drive is returning an unexpected data size
                if (*header).Length != dataLengthToCopy as u16 {
                    *DataLength = 0;
                    status = STATUS_IO_DEVICE_ERROR;
                }
            }

            if NT_SUCCESS(status) {
                // else copy the data to the user's buffer
                RtlCopyMemory(
                    outputBuffer,
                    (*header).Data.as_ptr() as *const c_void,
                    dataLengthToCopy as usize,
                );
                *DataLength = dataLengthToCopy as usize;
            }
        }

        ScratchBuffer_EndUse(DeviceExtension);
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn DeviceHandleAacsGenerateBindingNonce(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Request: WDFREQUEST,
    RequestParameters: WDF_REQUEST_PARAMETERS,
    DataLength: *mut usize,
) -> NTSTATUS {
    // AacsGenerateBindingNonce
    unsafe {
        let mut status: NTSTATUS;
        let mut input: *mut AACS_READ_BINDING_NONCE = core::ptr::null_mut();
        let mut outputBuffer: *mut core::ffi::c_void = core::ptr::null_mut();

        PAGED_CODE!();

        *DataLength = 0;

        status = WdfRequestRetrieveInputBuffer(
            Request,
            RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength as usize,
            &mut input as *mut *mut AACS_READ_BINDING_NONCE as *mut *mut core::ffi::c_void,
            core::ptr::null_mut(),
        );

        if NT_SUCCESS(status) {
            status = WdfRequestRetrieveOutputBuffer(
                Request,
                RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength as usize,
                &mut outputBuffer,
                core::ptr::null_mut(),
            );
        }

        if NT_SUCCESS(status) {
            let dataTransferLength = (core::mem::size_of::<CDVD_KEY_HEADER>() + core::mem::size_of::<AACS_BINDING_NONCE>()) as u32;
            let mut cdb: CDB = core::mem::zeroed();
            RtlZeroMemory(&mut cdb as *mut CDB as *mut core::ffi::c_void, core::mem::size_of::<CDB>());

            ScratchBuffer_BeginUse(DeviceExtension);

            status = ScratchBuffer_ExecuteCdb(
                DeviceExtension,
                Request,
                dataTransferLength,
                1, // TRUE
                &mut cdb as *mut CDB as *const core::ffi::c_void,
                12,
            );

            #[cfg(feature = "ENABLE_AACS_TESTING")]
            {
                static RESULTS: [u8; 4] = [0x00, 0x22, 0x00, 0x00];
                const DEFAULT_FILL: u8 = 0x37;
                core::ptr::write_bytes(
                    (*DeviceExtension).ScratchContext.ScratchBuffer as *mut u8,
                    DEFAULT_FILL,
                    0x0024,
                );
                core::ptr::copy_nonoverlapping(
                    RESULTS.as_ptr(),
                    (*DeviceExtension).ScratchContext.ScratchBuffer as *mut u8,
                    RESULTS.len(),
                );
                status = STATUS_SUCCESS;
            }

            if NT_SUCCESS(status) {
                let header: *mut DVD_DESCRIPTOR_HEADER = (*DeviceExtension).ScratchContext.ScratchBuffer as *mut DVD_DESCRIPTOR_HEADER;
                let dataLengthToCopy: u32 = core::mem::size_of::<AACS_BINDING_NONCE>() as u32;

                // make length field native byte ordering
                REVERSE_SHORT(core::ptr::addr_of_mut!((*header).Length));

                let size_through_length = (core::mem::offset_of!(DVD_DESCRIPTOR_HEADER, Length) + core::mem::size_of::<u16>()) as u16;
                let header_descriptor_size = core::mem::size_of::<DVD_DESCRIPTOR_HEADER>() as u16;

                // exit if getting invalid data from the drive
                if (*header).Length < (header_descriptor_size - size_through_length) {
                    *DataLength = 0;
                    status = STATUS_IO_DEVICE_ERROR;
                }

                if NT_SUCCESS(status) {
                    // adjust data length to reflect only the addition data
                    (*header).Length -= header_descriptor_size - size_through_length;

                    // exit if the drive is returning an unexpected data size
                    if (*header).Length != dataLengthToCopy as u16 {
                        *DataLength = 0;
                        status = STATUS_IO_DEVICE_ERROR;
                    }
                }

                if NT_SUCCESS(status) {
                    // else copy the data to the user's buffer
                    RtlCopyMemory(
                        outputBuffer as *mut core::ffi::c_void,
                        (*header).Data.as_ptr() as *const core::ffi::c_void,
                        dataLengthToCopy as usize,
                    );
                    *DataLength = dataLengthToCopy as usize;
                }
            }

            ScratchBuffer_EndUse(DeviceExtension);
        }

        status
    }
}

// Declarations for missing constants and functions

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
//     aacs.c
//
// Abstract:
//
//     The CDROM class driver implementation of handling AACS IOCTLs.
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
// #include "stddef.h"
// #include "string.h"
//
// #include "ntddk.h"
// #include "ntddstor.h"
// #include "cdrom.h"
// #include "ioctl.h"
// #include "scratch.h"
//
// #ifdef DEBUG_USE_WPP
// #include "aacs.tmh"
// #endif
//
// #ifdef ALLOC_PRAGMA
//
// #pragma alloc_text(PAGE, DeviceHandleAacsReadMediaKeyBlock)
// #pragma alloc_text(PAGE, DeviceHandleAacsStartSession)
// #pragma alloc_text(PAGE, DeviceHandleAacsEndSession)
// #pragma alloc_text(PAGE, DeviceHandleAacsSendCertificate)
// #pragma alloc_text(PAGE, DeviceHandleAacsGetCertificate)
// #pragma alloc_text(PAGE, DeviceHandleAacsGetChallengeKey)
// #pragma alloc_text(PAGE, DeviceHandleAacsReadSerialNumber)
// #pragma alloc_text(PAGE, DeviceHandleAacsReadMediaId)
// #pragma alloc_text(PAGE, DeviceHandleAacsReadBindingNonce)
// #pragma alloc_text(PAGE, DeviceHandleAacsGenerateBindingNonce)
// #pragma alloc_text(PAGE, DeviceHandleReadVolumeId)
// #pragma alloc_text(PAGE, DeviceHandleSendChallengeKey)
//
// #endif
//
// _IRQL_requires_max_(APC_LEVEL)
// NTSTATUS
// DeviceHandleAacsReadMediaKeyBlock(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDFREQUEST               Request,
//       WDF_REQUEST_PARAMETERS   RequestParameters,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//     This routine is used to process IOCTLs:
//         IOCTL_AACS_READ_MEDIA_KEY_BLOCK_SIZE
//         IOCTL_AACS_READ_MEDIA_KEY_BLOCK
// Arguments:
//     DeviceExtension - device context
//
//     Request - the request that will be formatted
//
//     RequestParameters - request parameter structur
//
//     DataLength - data transferred length
//
// Return Value:
//     NTSTATUS
//
//   --*/
// {
//     NTSTATUS            status = STATUS_SUCCESS;
//     PAACS_LAYER_NUMBER  layerNumber = NULL;
//     PVOID               outputBuffer = NULL;
//     ULONG               transferSize = sizeof(READ_DVD_STRUCTURES_HEADER);
//
//     PAGED_CODE();
//
//     *DataLength = 0;
//
//     status = WdfRequestRetrieveInputBuffer(Request,
//                                            RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
//                                            (PVOID*)&layerNumber,
//                                            NULL);
//
//     if (NT_SUCCESS(status))
//     {
//         status = WdfRequestRetrieveOutputBuffer(Request,
//                                                 RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength,
//                                                 (PVOID*)&outputBuffer,
//                                                 NULL);
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         if (RequestParameters.Parameters.DeviceIoControl.as_ref().IoControlCode == IOCTL_AACS_READ_MEDIA_KEY_BLOCK)
//         {
//             // maximum size for this transfer is one pack + header
//             transferSize += AACS_MKB_PACK_SIZE;
//         }
//
//         if (transferSize > DeviceExtension->ScratchContext.ScratchBufferSize)
//         {
//             // rare case. normally the size of scratch buffer is 64k.
//             status = STATUS_INTERNAL_ERROR;
//         }
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         UCHAR                       rmdBlockNumber = 0;
//         BOOLEAN                     sendChangedCommand = TRUE;
//         BOOLEAN                     shouldRetry = TRUE;
//         CDB                         cdb;
//
//         ScratchBuffer_BeginUse(DeviceExtension);
//
//         RtlZeroMemory(&cdb, sizeof(CDB));
//         // Set up the CDB
//         cdb.READ_DVD_STRUCTURE.OperationCode = SCSIOP_READ_DVD_STRUCTURE;
//         // cdb->AsByte[1] = 0x01; // AACS sub-command not required for this
//
//         cdb.READ_DVD_STRUCTURE.LayerNumber = (UCHAR)(*layerNumber);
//         cdb.READ_DVD_STRUCTURE.Format = 0x83; // MKB
//         cdb.READ_DVD_STRUCTURE.AllocationLength[0] = (UCHAR)(transferSize >> (8*1));
//         cdb.READ_DVD_STRUCTURE.AllocationLength[1] = (UCHAR)(transferSize >> (8*0));
//
//         while (sendChangedCommand)
//         {
//             // RMDBlockNumber is set to zero....
//             // RMDBlockNumber[3] maybe changed for other blocks.
//             cdb.READ_DVD_STRUCTURE.RMDBlockNumber[3] = rmdBlockNumber;
//
//             if (shouldRetry)
//             {
//                 status = ScratchBuffer_ExecuteCdb(DeviceExtension, Request, transferSize, TRUE, &cdb, 12);
//             }
//
//     #ifdef ENABLE_AACS_TESTING
//             if (RequestParameters.Parameters.DeviceIoControl.as_ref().IoControlCode == IOCTL_AACS_READ_MEDIA_KEY_BLOCK_SIZE)
//             {
//                 static const UCHAR results[] = { 0x80, 0x02, 0x00, 0x02 };
//                 RtlCopyMemory(DeviceExtension->ScratchContext.ScratchBuffer, results, SIZEOF_ARRAY(results));
//                 status = STATUS_SUCCESS;
//             }
//             else if (RequestParameters.Parameters.DeviceIoControl.as_ref().IoControlCode == IOCTL_AACS_READ_MEDIA_KEY_BLOCK)
//             {
//                 static const UCHAR results[] = { 0x80, 0x02, 0x00, 0x02 };
//                 static const UCHAR defaultFill = 0x30; // '0'
//                 RtlFillMemory(DeviceExtension->ScratchContext.ScratchBuffer, 0x8004, defaultFill);
//                 RtlCopyMemory(DeviceExtension->ScratchContext.ScratchBuffer, results, SIZEOF_ARRAY(results));
//                 status = STATUS_SUCCESS;
//             }
//     #endif //ENABLE_AACS_TESTING
//
//             if (NT_SUCCESS(status))
//             {
//                 // command succeeded, process data...
//                 PDVD_DESCRIPTOR_HEADER  header = DeviceExtension->ScratchContext.ScratchBuffer;
//                 UCHAR                   thisPackNumber = cdb.READ_DVD_STRUCTURE.RMDBlockNumber[3];
//                 UCHAR                   otherPacks = header->Reserved[1];
//
//                 // validate and zero-base the otherPacks
//                 if (otherPacks == 0)
//                 {
//                     TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_INIT,
//                                 "AACS: Device is reporting zero total packs (invalid)\n"));
//                     *DataLength = 0;
//                     status = STATUS_IO_DEVICE_ERROR;
//                 }
//                 else
//                 {
//                     otherPacks--;
//
//                     if (RequestParameters.Parameters.DeviceIoControl.as_ref().IoControlCode == IOCTL_AACS_READ_MEDIA_KEY_BLOCK_SIZE)
//                     {
//                         // if not already requested last pack, do so now
//                         if (otherPacks != thisPackNumber)
//                         {
//                             // re-send the command for the other pack number.
//                             // this is safe here because NT_SUCCESS() is TRUE,
//                             // and all the rest of this routine does is SetHardError()
//                             // and release of resources we're still about to use.
//
//                             // re-zero the output buffer
//                             RtlZeroMemory(DeviceExtension->ScratchContext.ScratchBuffer, sizeof(READ_DVD_STRUCTURES_HEADER));
//
//                             // modify the CDB to get the very last pack of the MKB
//                             rmdBlockNumber = otherPacks;
//
//                             transferSize = sizeof(READ_DVD_STRUCTURES_HEADER);
//
//                             // keep items clean
//                             ScratchBuffer_ResetItems(DeviceExtension, TRUE);
//
//                             // make sure the loop will be executed for modified command.
//                             sendChangedCommand = TRUE;
//                             shouldRetry = TRUE;
//                         }
//                         else
//                         {
//                             // this request already got the last pack
//                             // so just interpret the data
//                             REVERSE_SHORT(&header->Length);
//                             if (header->Length < sizeof(DVD_DESCRIPTOR_HEADER) - RTL_SIZEOF_THROUGH_FIELD(DVD_DESCRIPTOR_HEADER, Length))
//                             {
//                                 *DataLength = 0;
//                                 status = STATUS_IO_DEVICE_ERROR;
//                             }
//                             else
//                             {
//                                 ULONG totalSize = header->Length;
//                                 // subtract out any remaining bytes in the header
//                                 // to get the number of usable bytes in this pack
//                                 totalSize -= sizeof(DVD_DESCRIPTOR_HEADER) - RTL_SIZEOF_THROUGH_FIELD(DVD_DESCRIPTOR_HEADER, Length);
//                                 totalSize += otherPacks * AACS_MKB_PACK_SIZE;
//
//                                 // save the result and complete the request
//                                 *((PULONG)outputBuffer) = totalSize;
//                                 *DataLength = sizeof(ULONG);
//                                 status = STATUS_SUCCESS;
//                             }
//                             // This will exit the loop of sendChangedCommand
//                             sendChangedCommand = FALSE;
//                             shouldRetry = FALSE;
//                         }
//                     }
//                     else if (RequestParameters.Parameters.DeviceIoControl.as_ref().IoControlCode == IOCTL_AACS_READ_MEDIA_KEY_BLOCK)
//                     {
//                         // make length field native byte ordering
//                         REVERSE_SHORT(&header->Length);
//
//                         // exit if getting invalid data from the drive
//                         if (header->Length < sizeof(DVD_DESCRIPTOR_HEADER) - RTL_SIZEOF_THROUGH_FIELD(DVD_DESCRIPTOR_HEADER, Length))
//                         {
//                             *DataLength = 0;
//                             status = STATUS_IO_DEVICE_ERROR;
//                         }
//                         else
//                         {
//                             // success, how many bytes to copy for this pack?
//                             ULONG totalSize = header->Length;
//                             size_t originalBufferSize;
//
//                             // subtract out any remaining bytes in the header
//                             // to get the number of usable bytes in this pack
//                             totalSize -= sizeof(DVD_DESCRIPTOR_HEADER) - RTL_SIZEOF_THROUGH_FIELD(DVD_DESCRIPTOR_HEADER, Length);
//
//                             // if not the final pack, this should be a full transfer per spec
//                             NT_ASSERT( (totalSize == AACS_MKB_PACK_SIZE) || (thisPackNumber == otherPacks) );
//
//                             // validate the user's buffer is large enough to accept the full data
//                             originalBufferSize = RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength;
//
//                             if (originalBufferSize < (totalSize + (AACS_MKB_PACK_SIZE*thisPackNumber)))
//                             {
//                                 // just return a slightly bigger-than-normal size
//                                 *DataLength = (otherPacks + 1)*AACS_MKB_PACK_SIZE;
//                                 status = STATUS_BUFFER_TOO_SMALL;
//                             }
//                             else
//                             {
//                                 PUCHAR whereToCopy;
//                                 // determine where to copy to the user's memory
//                                 whereToCopy = outputBuffer;
//                                 whereToCopy += AACS_MKB_PACK_SIZE * thisPackNumber;
//
//                                 RtlCopyMemory(whereToCopy, header->Data, totalSize);
//
//                                 // update the Information field here because we already
//                                 // have calculated the size of the block
//                                 *DataLength = totalSize + (AACS_MKB_PACK_SIZE * thisPackNumber);
//                                 status = STATUS_SUCCESS;
//
//                                 // if there are more packs to get from the device, send it again....
//                                 if (thisPackNumber != otherPacks)
//                                 {
//                                     // re-send the command for the next pack number.
//                                     // this is safe here because NT_SUCCESS() is TRUE,
//                                     // and all the rest of this routine does is SetHardError()
//                                     // and release of resources we're still about to use.
//
//                                     // re-zero the output buffer
//                                     RtlZeroMemory(DeviceExtension->ScratchContext.ScratchBuffer, sizeof(READ_DVD_STRUCTURES_HEADER));
//
//                                     // modify the CDB to get the next pack of the MKB
//                                     rmdBlockNumber = cdb.READ_DVD_STRUCTURE.RMDBlockNumber[3]++;
//
//                                     // modify the SRB to be resent
//                                     //
//                                     transferSize = AACS_MKB_PACK_SIZE + sizeof(READ_DVD_STRUCTURES_HEADER);
//
//                                     // keep items clean
//                                     ScratchBuffer_ResetItems(DeviceExtension, FALSE);
//
//                                     // make sure the loop will be executed for modified command.
//                                     sendChangedCommand = TRUE;
//                                     shouldRetry = TRUE;
//                                 }
//                                 else
//                                 {
//                                     // else, that was the end of the transfer, so just complete the request
//                                     sendChangedCommand = FALSE;
//                                 }
//                             }
//                         }
//
//                     } // end of IOCTL_AACS_READ_MEDIA_KEY_BLOCK
//                 }
//             } // end of NT_SUCCESS(status)
//
//             if (!NT_SUCCESS(status))
//             {
//                 // command failed.
//                 sendChangedCommand = FALSE;
//             }
//         } //end of while (sendChangedCommand)
//
//         ScratchBuffer_EndUse(DeviceExtension);
//     }
//
//     return status;
// }
//
// _IRQL_requires_max_(APC_LEVEL)
// NTSTATUS
// DeviceHandleAacsStartSession(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDFREQUEST               Request,
//       WDF_REQUEST_PARAMETERS   RequestParameters,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//     This routine is used to process IOCTL:
//         IOCTL_AACS_START_SESSION
// Arguments:
//     DeviceExtension - device context
//
//     Request - the request that will be formatted
//
//     RequestParameters - request parameter structur
//
//     DataLength - data transferred length
//
// Return Value:
//     NTSTATUS
//
//   --*/
// {
//     //AacsGetAgid
//
//     NTSTATUS        status = STATUS_SUCCESS;
//     PDVD_SESSION_ID sessionId = NULL;
//
//     PAGED_CODE();
//
//     *DataLength = 0;
//
//     status = WdfRequestRetrieveOutputBuffer(Request,
//                                             RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength,
//                                             (PVOID*)&sessionId,
//                                             NULL);
//
//     if (NT_SUCCESS(status))
//     {
//         ULONG   dataTransferLength = sizeof(CDVD_KEY_HEADER) + sizeof(CDVD_REPORT_AGID_DATA);
//         CDB     cdb;
//
//         ScratchBuffer_BeginUse(DeviceExtension);
//
//         RtlZeroMemory(&cdb, sizeof(CDB));
//         // Set up the CDB
//         cdb.REPORT_KEY.OperationCode = SCSIOP_REPORT_KEY;
//         cdb.AsByte[7] = 0x02; // AACS key class
//         cdb.REPORT_KEY.AllocationLength[0] = (UCHAR)(dataTransferLength >> (8*1));
//         cdb.REPORT_KEY.AllocationLength[1] = (UCHAR)(dataTransferLength >> (8*0));
//         cdb.REPORT_KEY.KeyFormat = 0x00; // DVD_REPORT_AGID?
//
//         status = ScratchBuffer_ExecuteCdb(DeviceExtension, Request, dataTransferLength, TRUE, &cdb, 12);
//
// #ifdef ENABLE_AACS_TESTING
//         static const UCHAR results[] = { 0x00, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0xC0 };
//         RtlCopyMemory(DeviceExtension->ScratchContext.ScratchBuffer, results, SIZEOF_ARRAY(results));
//         status = STATUS_SUCCESS;
// #endif
//         if (NT_SUCCESS(status))
//         {
//             PCDVD_KEY_HEADER        keyHeader = DeviceExtension->ScratchContext.ScratchBuffer;
//             PCDVD_REPORT_AGID_DATA  keyData = (PCDVD_REPORT_AGID_DATA)keyHeader->Data;
//
//             *sessionId = (DVD_SESSION_ID)(keyData->AGID);
//             *DataLength = sizeof(DVD_SESSION_ID);
//         }
//
//         ScratchBuffer_EndUse(DeviceExtension);
//     }
//
//     return status;
// }
//
// _IRQL_requires_max_(APC_LEVEL)
// NTSTATUS
// DeviceHandleAacsEndSession(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDFREQUEST               Request,
//       WDF_REQUEST_PARAMETERS   RequestParameters,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//     This routine is used to process IOCTL:
//         IOCTL_AACS_END_SESSION
// Arguments:
//     DeviceExtension - device context
//
//     Request - the request that will be formatted
//
//     RequestParameters - request parameter structur
//
//     DataLength - data transferred length
//
// Return Value:
//     NTSTATUS
//
//   --*/
// {
//     //AacsReleaseAgid
//
//     NTSTATUS        status = STATUS_SUCCESS;
//     PDVD_SESSION_ID sessionId = NULL;
//
//     PAGED_CODE();
//
//     *DataLength = 0;
//
//     status = WdfRequestRetrieveInputBuffer(Request,
//                                            RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
//                                            (PVOID*)&sessionId,
//                                            NULL);
//
//     if (NT_SUCCESS(status))
//     {
//         ULONG                       transferSize = 0;
//         CDB                         cdb;
//         DVD_SESSION_ID              currentSession = 0;
//         DVD_SESSION_ID              limitSession = 0;
//
//         if(*sessionId == DVD_END_ALL_SESSIONS)
//         {
//             currentSession = 0;
//             limitSession = MAX_COPY_PROTECT_AGID - 1;
//         }
//         else
//         {
//             currentSession = *sessionId;
//             limitSession = *sessionId;
//         }
//
//         ScratchBuffer_BeginUse(DeviceExtension);
//
//         do
//         {
//             RtlZeroMemory(&cdb, sizeof(CDB));
//             // Set up the CDB
//             cdb.SEND_KEY.OperationCode = SCSIOP_SEND_KEY;
//             cdb.AsByte[7] = 0x02; // AACS key class
//             cdb.SEND_KEY.AGID = (UCHAR)(currentSession);
//             cdb.SEND_KEY.KeyFormat = DVD_INVALIDATE_AGID;
//
//             status = ScratchBuffer_ExecuteCdb(DeviceExtension, Request, transferSize, FALSE, &cdb, 12);
//
//             currentSession++;
//         } while ((currentSession <= limitSession) && NT_SUCCESS(status));
//
// #ifdef ENABLE_AACS_TESTING
//         status = STATUS_SUCCESS;
// #endif
//
//         ScratchBuffer_EndUse(DeviceExtension);
//     }
//
//     return status;
// }
//
// _IRQL_requires_max_(APC_LEVEL)
// NTSTATUS
// DeviceHandleAacsSendCertificate(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDFREQUEST               Request,
//       WDF_REQUEST_PARAMETERS   RequestParameters,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//     This routine is used to process IOCTL:
//         IOCTL_AACS_SEND_CERTIFICATE
// Arguments:
//     DeviceExtension - device context
//
//     Request - the request that will be formatted
//
//     RequestParameters - request parameter structur
//
//     DataLength - data transferred length
//
// Return Value:
//     NTSTATUS
//
//   --*/
// {
//     //AacsSendHostCertificate
//
//     NTSTATUS                status = STATUS_SUCCESS;
//     PAACS_SEND_CERTIFICATE  input = NULL;
//
//     PAGED_CODE();
//
//     *DataLength = 0;
//
//     status = WdfRequestRetrieveInputBuffer(Request,
//                                            RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
//                                            (PVOID*)&input,
//                                            NULL);
//
//     if (NT_SUCCESS(status))
//     {
//         ULONG   dataTransferLength = sizeof(CDVD_KEY_HEADER) + sizeof(AACS_CERTIFICATE);
//         CDB     cdb;
//
//         ScratchBuffer_BeginUse(DeviceExtension);
//
//         // copy the input buffer to the data buffer for the transfer
//         {
//             PCDVD_KEY_HEADER header = (PCDVD_KEY_HEADER)DeviceExtension->ScratchContext.ScratchBuffer;
//             ULONG            tmp = dataTransferLength;
//
//             tmp -= RTL_SIZEOF_THROUGH_FIELD(CDVD_KEY_HEADER, DataLength);
//
//             header->DataLength[0] = (UCHAR)(tmp >> (8*1));
//             header->DataLength[1] = (UCHAR)(tmp >> (8*0));
//             RtlCopyMemory(header->Data, &(input->Certificate), sizeof(AACS_CERTIFICATE));
//         }
//
//         RtlZeroMemory(&cdb, sizeof(CDB));
//         // Set up the CDB
//         cdb.SEND_KEY.OperationCode = SCSIOP_SEND_KEY;
//         cdb.AsByte[7] = 0x02; // AACS key class
//         cdb.SEND_KEY.ParameterListLength[0] = (UCHAR)(dataTransferLength >> (8*1));
//         cdb.SEND_KEY.ParameterListLength[1] = (UCHAR)(dataTransferLength >> (8*0));
//         cdb.SEND_KEY.AGID = (UCHAR)( input->SessionId );
//         cdb.SEND_KEY.KeyFormat = 0x01; // Send Host Challenge Certificate
//
//         status = ScratchBuffer_ExecuteCdb(DeviceExtension, Request, dataTransferLength, FALSE, &cdb, 12);
//
// #ifdef ENABLE_AACS_TESTING
//         status = STATUS_SUCCESS;
// #endif
//         if (NT_SUCCESS(status))
//         {
//             *DataLength = 0;
//         }
//
//         ScratchBuffer_EndUse(DeviceExtension);
//     }
//
//     return status;
// }
//
// _IRQL_requires_max_(APC_LEVEL)
// NTSTATUS
// DeviceHandleAacsGetCertificate(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDFREQUEST               Request,
//       WDF_REQUEST_PARAMETERS   RequestParameters,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//     This routine is used to process IOCTL:
//         IOCTL_AACS_GET_CERTIFICATE
// Arguments:
//     DeviceExtension - device context
//
//     Request - the request that will be formatted
//
//     RequestParameters - request parameter structur
//
//     DataLength - data transferred length
//
// Return Value:
//     NTSTATUS
//
//   --*/
// {
//     //AacsGetDriveCertificate
//
//     NTSTATUS        status = STATUS_SUCCESS;
//     PDVD_SESSION_ID input = NULL;
//     PVOID           outputBuffer = NULL;
//
//     PAGED_CODE();
//
//     *DataLength = 0;
//
//     status = WdfRequestRetrieveInputBuffer(Request,
//                                            RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
//                                            (PVOID*)&input,
//                                            NULL);
//
//     if (NT_SUCCESS(status))
//     {
//         status = WdfRequestRetrieveOutputBuffer(Request,
//                                                 RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength,
//                                                 (PVOID*)&outputBuffer,
//                                                 NULL);
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         ULONG   dataTransferLength = sizeof(CDVD_KEY_HEADER) + sizeof(AACS_CERTIFICATE);
//         CDB     cdb;
//
//         ScratchBuffer_BeginUse(DeviceExtension);
//
//         RtlZeroMemory(&cdb, sizeof(CDB));
//         // Set up the CDB
//         cdb.REPORT_KEY.OperationCode = SCSIOP_REPORT_KEY;
//         cdb.AsByte[7] = 0x02; // AACS key class
//         cdb.REPORT_KEY.AllocationLength[0] = (UCHAR)(dataTransferLength >> (8*1));
//         cdb.REPORT_KEY.AllocationLength[1] = (UCHAR)(dataTransferLength >> (8*0));
//         cdb.REPORT_KEY.AGID = (UCHAR)(*input);
//         cdb.REPORT_KEY.KeyFormat = 0x01; // Return a drive certificate challenge
//
//         status = ScratchBuffer_ExecuteCdb(DeviceExtension, Request, dataTransferLength, TRUE, &cdb, 12);
//
// #ifdef ENABLE_AACS_TESTING
//             static const UCHAR results[] = { 0x00, 0x72, 0x00, 0x00 };
//             static const UCHAR defaultFill = 0x31; // '1'
//             RtlFillMemory(DeviceExtension->ScratchContext.ScratchBuffer, 0x0074, defaultFill);
//             RtlCopyMemory(DeviceExtension->ScratchContext.ScratchBuffer, results, SIZEOF_ARRAY(results));
//             status = STATUS_SUCCESS;
// #endif
//         if (NT_SUCCESS(status))
//         {
//             PDVD_DESCRIPTOR_HEADER  header = DeviceExtension->ScratchContext.ScratchBuffer;
//             ULONG                   dataLengthToCopy = sizeof(AACS_CERTIFICATE);
//
//             // make length field native byte ordering
//             REVERSE_SHORT(&header->Length);
//
//             // exit if getting invalid data from the drive
//             if (header->Length < (sizeof(DVD_DESCRIPTOR_HEADER) - RTL_SIZEOF_THROUGH_FIELD(DVD_DESCRIPTOR_HEADER, Length)))
//             {
//                 *DataLength = 0;
//                 status = STATUS_IO_DEVICE_ERROR;
//             }
//
//             if (NT_SUCCESS(status))
//             {
//                 // adjust data length to reflect only the addition data
//                 header->Length -= sizeof(DVD_DESCRIPTOR_HEADER) - RTL_SIZEOF_THROUGH_FIELD(DVD_DESCRIPTOR_HEADER, Length);
//
//                 // exit if the drive is returning an unexpected data size
//                 if (header->Length != dataLengthToCopy)
//                 {
//                     *DataLength = 0;
//                     status = STATUS_IO_DEVICE_ERROR;
//                 }
//             }
//
//             if (NT_SUCCESS(status))
//             {
//                 // else copy the data to the user's buffer
//                 RtlCopyMemory(outputBuffer, header->Data, dataLengthToCopy);
//                 *DataLength = dataLengthToCopy;
//             }
//         }
//
//         ScratchBuffer_EndUse(DeviceExtension);
//     }
//
//     return status;
// }
//
// _IRQL_requires_max_(APC_LEVEL)
// NTSTATUS
// DeviceHandleAacsGetChallengeKey(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDFREQUEST               Request,
//       WDF_REQUEST_PARAMETERS   RequestParameters,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//     This routine is used to process IOCTL:
//         IOCTL_AACS_GET_CHALLENGE_KEY
// Arguments:
//     DeviceExtension - device context
//
//     Request - the request that will be formatted
//
//     RequestParameters - request parameter structur
//
//     DataLength - data transferred length
//
// Return Value:
//     NTSTATUS
//
//   --*/
// {
//     //AacsGetChallengeKey
//
//     NTSTATUS        status = STATUS_SUCCESS;
//     PDVD_SESSION_ID input = NULL;
//     PVOID           outputBuffer = NULL;
//
//     PAGED_CODE();
//
//     *DataLength = 0;
//
//     status = WdfRequestRetrieveInputBuffer(Request,
//                                            RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
//                                            (PVOID*)&input,
//                                            NULL);
//
//     if (NT_SUCCESS(status))
//     {
//         status = WdfRequestRetrieveOutputBuffer(Request,
//                                                 RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength,
//                                                 (PVOID*)&outputBuffer,
//                                                 NULL);
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         ULONG   dataTransferLength = sizeof(CDVD_KEY_HEADER) + sizeof(AACS_CHALLENGE_KEY);
//         CDB     cdb;
//
//         ScratchBuffer_BeginUse(DeviceExtension);
//
//         RtlZeroMemory(&cdb, sizeof(CDB));
//         // Set up the CDB
//         cdb.REPORT_KEY.OperationCode = SCSIOP_REPORT_KEY;
//         cdb.AsByte[7] = 0x02; // AACS key class
//         cdb.REPORT_KEY.AllocationLength[0] = (UCHAR)(dataTransferLength >> (8*1));
//         cdb.REPORT_KEY.AllocationLength[1] = (UCHAR)(dataTransferLength >> (8*0));
//         cdb.REPORT_KEY.AGID = (UCHAR)(*input);
//         cdb.REPORT_KEY.KeyFormat = 0x02; // Return a drive certificate challenge
//
//         status = ScratchBuffer_ExecuteCdb(DeviceExtension, Request, dataTransferLength, TRUE, &cdb, 12);
//
// #ifdef ENABLE_AACS_TESTING
//             static const UCHAR results[] = { 0x00, 0x52, 0x00, 0x00 };
//             static const UCHAR defaultFill = 0x32; // '2'
//             RtlFillMemory(DeviceExtension->ScratchContext.ScratchBuffer, 0x0054, defaultFill);
//             RtlCopyMemory(DeviceExtension->ScratchContext.ScratchBuffer, results, SIZEOF_ARRAY(results));
//             status = STATUS_SUCCESS;
// #endif
//         if (NT_SUCCESS(status))
//         {
//             PDVD_DESCRIPTOR_HEADER  header = DeviceExtension->ScratchContext.ScratchBuffer;
//             ULONG                   dataLengthToCopy = sizeof(AACS_CHALLENGE_KEY);
//
//             // make length field native byte ordering
//             REVERSE_SHORT(&header->Length);
//
//             // exit if getting invalid data from the drive
//             if (header->Length < sizeof(DVD_DESCRIPTOR_HEADER) - RTL_SIZEOF_THROUGH_FIELD(DVD_DESCRIPTOR_HEADER, Length))
//             {
//                 *DataLength = 0;
//                 status = STATUS_IO_DEVICE_ERROR;
//             }
//
//             if (NT_SUCCESS(status))
//             {
//                 // adjust data length to reflect only the addition data
//                 header->Length -= sizeof(DVD_DESCRIPTOR_HEADER) - RTL_SIZEOF_THROUGH_FIELD(DVD_DESCRIPTOR_HEADER, Length);
//
//                 // exit if the drive is returning an unexpected data size
//                 if (header->Length != dataLengthToCopy)
//                 {
//                     *DataLength = 0;
//                     status = STATUS_IO_DEVICE_ERROR;
//                 }
//             }
//
//             if (NT_SUCCESS(status))
//             {
//                 // else copy the data to the user's buffer
//                 RtlCopyMemory(outputBuffer, header->Data, dataLengthToCopy);
//                 *DataLength = dataLengthToCopy;
//             }
//         }
//
//         ScratchBuffer_EndUse(DeviceExtension);
//     }
//
//     return status;
// }
//
// _IRQL_requires_max_(APC_LEVEL)
// NTSTATUS
// DeviceHandleSendChallengeKey(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDFREQUEST               Request,
//       WDF_REQUEST_PARAMETERS   RequestParameters,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//     This routine is used to process IOCTL:
//         IOCTL_AACS_SEND_CHALLENGE_KEY
// Arguments:
//     DeviceExtension - device context
//
//     Request - the request that will be formatted
//
//     RequestParameters - request parameter structur
//
//     DataLength - data transferred length
//
// Return Value:
//     NTSTATUS
//
//   --*/
// {
//     //AacsSendChallengeKey
//
//     NTSTATUS                 status = STATUS_SUCCESS;
//     PAACS_SEND_CHALLENGE_KEY input = NULL;
//
//     PAGED_CODE();
//
//     *DataLength = 0;
//
//     status = WdfRequestRetrieveInputBuffer(Request,
//                                            RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
//                                            (PVOID*)&input,
//                                            NULL);
//
//     if (NT_SUCCESS(status))
//     {
//         ULONG   dataTransferLength = sizeof(CDVD_KEY_HEADER) + sizeof(AACS_CHALLENGE_KEY);
//         CDB     cdb;
//
//         ScratchBuffer_BeginUse(DeviceExtension);
//
//         // copy the input buffer to the data buffer for the transfer
//         {
//             PCDVD_KEY_HEADER header = DeviceExtension->ScratchContext.ScratchBuffer;
//             ULONG tmp = dataTransferLength;
//             tmp -= RTL_SIZEOF_THROUGH_FIELD(CDVD_KEY_HEADER, DataLength);
//
//             header->DataLength[0] = (UCHAR)(tmp >> (8*1));
//             header->DataLength[1] = (UCHAR)(tmp >> (8*0));
//             RtlCopyMemory(header->Data, &(input->ChallengeKey), sizeof(AACS_CHALLENGE_KEY));
//         }
//
//         RtlZeroMemory(&cdb, sizeof(CDB));
//         // Set up the CDB
//         cdb.SEND_KEY.OperationCode = SCSIOP_SEND_KEY;
//         cdb.AsByte[7] = 0x02; // AACS key class
//         cdb.SEND_KEY.ParameterListLength[0] = (UCHAR)(dataTransferLength >> (8*1));
//         cdb.SEND_KEY.ParameterListLength[1] = (UCHAR)(dataTransferLength >> (8*0));
//         cdb.SEND_KEY.AGID = (UCHAR)( input->SessionId );
//         cdb.SEND_KEY.KeyFormat = 0x02; // Send Host Challenge Certificate
//
//         status = ScratchBuffer_ExecuteCdb(DeviceExtension, Request, dataTransferLength, FALSE, &cdb, 12);
//
// #ifdef ENABLE_AACS_TESTING
//             status = STATUS_SUCCESS;
// #endif
//         if (NT_SUCCESS(status))
//         {
//             *DataLength = 0;
//         }
//
//         ScratchBuffer_EndUse(DeviceExtension);
//     }
//
//     return status;
// }
//
// _IRQL_requires_max_(APC_LEVEL)
// NTSTATUS
// DeviceHandleReadVolumeId(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDFREQUEST               Request,
//       WDF_REQUEST_PARAMETERS   RequestParameters,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//     This routine is used to process IOCTL:
//         IOCTL_AACS_READ_VOLUME_ID
// Arguments:
//     DeviceExtension - device context
//
//     Request - the request that will be formatted
//
//     RequestParameters - request parameter structur
//
//     DataLength - data transferred length
//
// Return Value:
//     NTSTATUS
//
//   --*/
// {
//     //AacsReadVolumeID
//
//     NTSTATUS        status = STATUS_SUCCESS;
//     PDVD_SESSION_ID input = NULL;
//     PVOID           outputBuffer = NULL;
//
//     PAGED_CODE();
//
//     *DataLength = 0;
//
//     status = WdfRequestRetrieveInputBuffer(Request,
//                                            RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
//                                            (PVOID*)&input,
//                                            NULL);
//
//     if (NT_SUCCESS(status))
//     {
//         status = WdfRequestRetrieveOutputBuffer(Request,
//                                                 RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength,
//                                                 (PVOID*)&outputBuffer,
//                                                 NULL);
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         ULONG   dataTransferLength = sizeof(CDVD_KEY_HEADER) + sizeof(AACS_VOLUME_ID);
//         CDB     cdb;
//
//         ScratchBuffer_BeginUse(DeviceExtension);
//
//         RtlZeroMemory(&cdb, sizeof(CDB));
//         // Set up the CDB
//         cdb.READ_DVD_STRUCTURE.OperationCode = SCSIOP_READ_DVD_STRUCTURE;
//         cdb.READ_DVD_STRUCTURE.Format = 0x80; // Return the AACS volumeID
//         cdb.READ_DVD_STRUCTURE.AllocationLength[0] = (UCHAR)(dataTransferLength >> (8*1));
//         cdb.READ_DVD_STRUCTURE.AllocationLength[1] = (UCHAR)(dataTransferLength >> (8*0));
//         cdb.READ_DVD_STRUCTURE.AGID = (UCHAR)(*input);
//
//         status = ScratchBuffer_ExecuteCdb(DeviceExtension, Request, dataTransferLength, TRUE, &cdb, 12);
//
// #ifdef ENABLE_AACS_TESTING
//             static const UCHAR results[] = { 0x00, 0x22, 0x00, 0x00 };
//             static const UCHAR defaultFill = 0x33; // '3'
//             RtlFillMemory(DeviceExtension->ScratchContext.ScratchBuffer, 0x0024, defaultFill);
//             RtlCopyMemory(DeviceExtension->ScratchContext.ScratchBuffer, results, SIZEOF_ARRAY(results));
//             status = STATUS_SUCCESS;
// #endif
//         if (NT_SUCCESS(status))
//         {
//             PDVD_DESCRIPTOR_HEADER  header = DeviceExtension->ScratchContext.ScratchBuffer;
//             ULONG                   dataLengthToCopy = sizeof(AACS_VOLUME_ID);
//
//             // make length field native byte ordering
//             REVERSE_SHORT(&header->Length);
//
//             // exit if getting invalid data from the drive
//             if (header->Length < sizeof(DVD_DESCRIPTOR_HEADER) - RTL_SIZEOF_THROUGH_FIELD(DVD_DESCRIPTOR_HEADER, Length))
//             {
//                 *DataLength = 0;
//                 status = STATUS_IO_DEVICE_ERROR;
//             }
//
//             if (NT_SUCCESS(status))
//             {
//                 // adjust data length to reflect only the addition data
//                 header->Length -= sizeof(DVD_DESCRIPTOR_HEADER) - RTL_SIZEOF_THROUGH_FIELD(DVD_DESCRIPTOR_HEADER, Length);
//
//                 // exit if the drive is returning an unexpected data size
//                 if (header->Length != dataLengthToCopy)
//                 {
//                     *DataLength = 0;
//                     status = STATUS_IO_DEVICE_ERROR;
//                 }
//             }
//
//             if (NT_SUCCESS(status))
//             {
//                 // else copy the data to the user's buffer
//                 RtlCopyMemory(outputBuffer, header->Data, dataLengthToCopy);
//                 *DataLength = dataLengthToCopy;
//             }
//         }
//
//         ScratchBuffer_EndUse(DeviceExtension);
//     }
//
//     return status;
// }
//
// _IRQL_requires_max_(APC_LEVEL)
// NTSTATUS
// DeviceHandleAacsReadSerialNumber(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDFREQUEST               Request,
//       WDF_REQUEST_PARAMETERS   RequestParameters,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//     This routine is used to process IOCTL:
//         IOCTL_AACS_READ_SERIAL_NUMBER
// Arguments:
//     DeviceExtension - device context
//
//     Request - the request that will be formatted
//
//     RequestParameters - request parameter structur
//
//     DataLength - data transferred length
//
// Return Value:
//     NTSTATUS
//
//   --*/
// {
//     //AacsReadSerialNumber
//
//     NTSTATUS        status = STATUS_SUCCESS;
//     PDVD_SESSION_ID input = NULL;
//     PVOID           outputBuffer = NULL;
//
//     PAGED_CODE();
//
//     *DataLength = 0;
//
//     status = WdfRequestRetrieveInputBuffer(Request,
//                                            RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
//                                            (PVOID*)&input,
//                                            NULL);
//
//     if (NT_SUCCESS(status))
//     {
//         status = WdfRequestRetrieveOutputBuffer(Request,
//                                                 RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength,
//                                                 (PVOID*)&outputBuffer,
//                                                 NULL);
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         ULONG   dataTransferLength = sizeof(CDVD_KEY_HEADER) + sizeof(AACS_SERIAL_NUMBER);
//         CDB     cdb;
//
//         ScratchBuffer_BeginUse(DeviceExtension);
//
//         RtlZeroMemory(&cdb, sizeof(CDB));
//         // Set up the CDB
//         cdb.READ_DVD_STRUCTURE.OperationCode = SCSIOP_READ_DVD_STRUCTURE;
//         cdb.READ_DVD_STRUCTURE.Format = 0x81; // Return the AACS volumeID
//         cdb.READ_DVD_STRUCTURE.AllocationLength[0] = (UCHAR)(dataTransferLength >> (8*1));
//         cdb.READ_DVD_STRUCTURE.AllocationLength[1] = (UCHAR)(dataTransferLength >> (8*0));
//         cdb.READ_DVD_STRUCTURE.AGID = (UCHAR)(*input);
//
//         status = ScratchBuffer_ExecuteCdb(DeviceExtension, Request, dataTransferLength, TRUE, &cdb, 12);
//
// #ifdef ENABLE_AACS_TESTING
//             static const UCHAR results[] = { 0x00, 0x22, 0x00, 0x00 };
//             static const UCHAR defaultFill = 0x34; // '4'
//             RtlFillMemory(DeviceExtension->ScratchContext.ScratchBuffer, 0x0024, defaultFill);
//             RtlCopyMemory(DeviceExtension->ScratchContext.ScratchBuffer, results, SIZEOF_ARRAY(results));
//             status = STATUS_SUCCESS;
// #endif
//         if (NT_SUCCESS(status))
//         {
//             PDVD_DESCRIPTOR_HEADER  header = DeviceExtension->ScratchContext.ScratchBuffer;
//             ULONG                   dataLengthToCopy = sizeof(AACS_SERIAL_NUMBER);
//
//             // make length field native byte ordering
//             REVERSE_SHORT(&header->Length);
//
//             // exit if getting invalid data from the drive
//             if (header->Length < sizeof(DVD_DESCRIPTOR_HEADER) - RTL_SIZEOF_THROUGH_FIELD(DVD_DESCRIPTOR_HEADER, Length))
//             {
//                 *DataLength = 0;
//                 status = STATUS_IO_DEVICE_ERROR;
//             }
//
//             if (NT_SUCCESS(status))
//             {
//                 // adjust data length to reflect only the addition data
//                 header->Length -= sizeof(DVD_DESCRIPTOR_HEADER) - RTL_SIZEOF_THROUGH_FIELD(DVD_DESCRIPTOR_HEADER, Length);
//
//                 // exit if the drive is returning an unexpected data size
//                 if (header->Length != dataLengthToCopy)
//                 {
//                     *DataLength = 0;
//                     status = STATUS_IO_DEVICE_ERROR;
//                 }
//             }
//
//             if (NT_SUCCESS(status))
//             {
//                 // else copy the data to the user's buffer
//                 RtlCopyMemory(outputBuffer, header->Data, dataLengthToCopy);
//                 *DataLength = dataLengthToCopy;
//             }
//         }
//
//         ScratchBuffer_EndUse(DeviceExtension);
//     }
//
//     return status;
// }
//
// _IRQL_requires_max_(APC_LEVEL)
// NTSTATUS
// DeviceHandleAacsReadMediaId(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDFREQUEST               Request,
//       WDF_REQUEST_PARAMETERS   RequestParameters,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//     This routine is used to process IOCTL:
//         IOCTL_AACS_READ_MEDIA_ID
// Arguments:
//     DeviceExtension - device context
//
//     Request - the request that will be formatted
//
//     RequestParameters - request parameter structur
//
//     DataLength - data transferred length
//
// Return Value:
//     NTSTATUS
//
//   --*/
// {
//     //AacsReadMediaID
//
//     NTSTATUS        status = STATUS_SUCCESS;
//     PDVD_SESSION_ID input = NULL;
//     PVOID           outputBuffer = NULL;
//
//     PAGED_CODE();
//
//     *DataLength = 0;
//
//     status = WdfRequestRetrieveInputBuffer(Request,
//                                            RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
//                                            (PVOID*)&input,
//                                            NULL);
//
//     if (NT_SUCCESS(status))
//     {
//         status = WdfRequestRetrieveOutputBuffer(Request,
//                                                 RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength,
//                                                 (PVOID*)&outputBuffer,
//                                                 NULL);
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         ULONG   dataTransferLength = sizeof(CDVD_KEY_HEADER) + sizeof(AACS_MEDIA_ID);
//         CDB     cdb;
//
//         ScratchBuffer_BeginUse(DeviceExtension);
//
//         RtlZeroMemory(&cdb, sizeof(CDB));
//         // Set up the CDB
//         cdb.READ_DVD_STRUCTURE.OperationCode = SCSIOP_READ_DVD_STRUCTURE;
//         cdb.READ_DVD_STRUCTURE.Format = 0x82; // Return the AACS volumeID
//         cdb.READ_DVD_STRUCTURE.AllocationLength[0] = (UCHAR)(dataTransferLength >> (8*1));
//         cdb.READ_DVD_STRUCTURE.AllocationLength[1] = (UCHAR)(dataTransferLength >> (8*0));
//         cdb.READ_DVD_STRUCTURE.AGID = (UCHAR)(*input);
//
//         status = ScratchBuffer_ExecuteCdb(DeviceExtension, Request, dataTransferLength, TRUE, &cdb, 12);
//
// #ifdef ENABLE_AACS_TESTING
//             static const UCHAR results[] = { 0x00, 0x22, 0x00, 0x00 };
//             static const UCHAR defaultFill = 0x35; // '5'
//             RtlFillMemory(DeviceExtension->ScratchContext.ScratchBuffer, 0x0024, defaultFill);
//             RtlCopyMemory(DeviceExtension->ScratchContext.ScratchBuffer, results, SIZEOF_ARRAY(results));
//             status = STATUS_SUCCESS;
// #endif
//         if (NT_SUCCESS(status))
//         {
//             PDVD_DESCRIPTOR_HEADER  header = DeviceExtension->ScratchContext.ScratchBuffer;
//             ULONG                   dataLengthToCopy = sizeof(AACS_MEDIA_ID);
//
//             // make length field native byte ordering
//             REVERSE_SHORT(&header->Length);
//
//             // exit if getting invalid data from the drive
//             if (header->Length < sizeof(DVD_DESCRIPTOR_HEADER) - RTL_SIZEOF_THROUGH_FIELD(DVD_DESCRIPTOR_HEADER, Length))
//             {
//                 *DataLength = 0;
//                 status = STATUS_IO_DEVICE_ERROR;
//             }
//
//             if (NT_SUCCESS(status))
//             {
//                 // adjust data length to reflect only the addition data
//                 header->Length -= sizeof(DVD_DESCRIPTOR_HEADER) - RTL_SIZEOF_THROUGH_FIELD(DVD_DESCRIPTOR_HEADER, Length);
//
//                 // exit if the drive is returning an unexpected data size
//                 if (header->Length != dataLengthToCopy)
//                 {
//                     *DataLength = 0;
//                     status = STATUS_IO_DEVICE_ERROR;
//                 }
//             }
//
//             if (NT_SUCCESS(status))
//             {
//                 // else copy the data to the user's buffer
//                 RtlCopyMemory(outputBuffer, header->Data, dataLengthToCopy);
//                 *DataLength = dataLengthToCopy;
//             }
//         }
//
//         ScratchBuffer_EndUse(DeviceExtension);
//     }
//
//     return status;
// }
//
// _IRQL_requires_max_(APC_LEVEL)
// NTSTATUS
// DeviceHandleAacsReadBindingNonce(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDFREQUEST               Request,
//       WDF_REQUEST_PARAMETERS   RequestParameters,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//     This routine is used to process IOCTL:
//         IOCTL_AACS_READ_BINDING_NONCE
// Arguments:
//     DeviceExtension - device context
//
//     Request - the request that will be formatted
//
//     RequestParameters - request parameter structur
//
//     DataLength - data transferred length
//
// Return Value:
//     NTSTATUS
//
//   --*/
// {
//     //AacsReadBindingNonce
//
//     NTSTATUS                 status = STATUS_SUCCESS;
//     PAACS_READ_BINDING_NONCE input = NULL;
//     PVOID                    outputBuffer = NULL;
//
//     PAGED_CODE();
//
//     *DataLength = 0;
//
//     status = WdfRequestRetrieveInputBuffer(Request,
//                                            RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
//                                            (PVOID*)&input,
//                                            NULL);
//
//     if (NT_SUCCESS(status))
//     {
//         status = WdfRequestRetrieveOutputBuffer(Request,
//                                                 RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength,
//                                                 (PVOID*)&outputBuffer,
//                                                 NULL);
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         ULONG   dataTransferLength = sizeof(CDVD_KEY_HEADER) + sizeof(AACS_BINDING_NONCE);
//         CDB     cdb;
//
//         ScratchBuffer_BeginUse(DeviceExtension);
//
//         RtlZeroMemory(&cdb, sizeof(CDB));
//         // Set up the CDB
//         cdb.REPORT_KEY.OperationCode = SCSIOP_REPORT_KEY;
//         cdb.REPORT_KEY.LogicalBlockAddress[0] = (UCHAR)( input->StartLba >> (3*8) );
//         cdb.REPORT_KEY.LogicalBlockAddress[1] = (UCHAR)( input->StartLba >> (2*8) );
//         cdb.REPORT_KEY.LogicalBlockAddress[2] = (UCHAR)( input->StartLba >> (1*8) );
//         cdb.REPORT_KEY.LogicalBlockAddress[3] = (UCHAR)( input->StartLba >> (0*8) );
//         cdb.AsByte[6] = (UCHAR)( input->NumberOfSectors );
//         cdb.AsByte[7] = 0x02; // AACS key class
//         cdb.REPORT_KEY.AllocationLength[0] = (UCHAR)(dataTransferLength >> (8*1));
//         cdb.REPORT_KEY.AllocationLength[1] = (UCHAR)(dataTransferLength >> (8*0));
//         cdb.REPORT_KEY.AGID = (UCHAR)( input->SessionId );
//         cdb.REPORT_KEY.KeyFormat = 0x21; // Return an existing binding nonce
//
//         status = ScratchBuffer_ExecuteCdb(DeviceExtension, Request, dataTransferLength, TRUE, &cdb, 12);
//
// #ifdef ENABLE_AACS_TESTING
//             static const UCHAR results[] = { 0x00, 0x22, 0x00, 0x00 };
//             static const UCHAR defaultFill = 0x36; // '6'
//             RtlFillMemory(DeviceExtension->ScratchContext.ScratchBuffer, 0x0024, defaultFill);
//             RtlCopyMemory(DeviceExtension->ScratchContext.ScratchBuffer, results, SIZEOF_ARRAY(results));
//             status = STATUS_SUCCESS;
// #endif
//         if (NT_SUCCESS(status))
//         {
//             PDVD_DESCRIPTOR_HEADER  header = DeviceExtension->ScratchContext.ScratchBuffer;
//             ULONG                   dataLengthToCopy = sizeof(AACS_BINDING_NONCE);
//
//             // make length field native byte ordering
//             REVERSE_SHORT(&header->Length);
//
//             // exit if getting invalid data from the drive
//             if (header->Length < sizeof(DVD_DESCRIPTOR_HEADER) - RTL_SIZEOF_THROUGH_FIELD(DVD_DESCRIPTOR_HEADER, Length))
//             {
//                 *DataLength = 0;
//                 status = STATUS_IO_DEVICE_ERROR;
//             }
//
//             if (NT_SUCCESS(status))
//             {
//                 // adjust data length to reflect only the addition data
//                 header->Length -= sizeof(DVD_DESCRIPTOR_HEADER) - RTL_SIZEOF_THROUGH_FIELD(DVD_DESCRIPTOR_HEADER, Length);
//
//                 // exit if the drive is returning an unexpected data size
//                 if (header->Length != dataLengthToCopy)
//                 {
//                     *DataLength = 0;
//                     status = STATUS_IO_DEVICE_ERROR;
//                 }
//             }
//
//             if (NT_SUCCESS(status))
//             {
//                 // else copy the data to the user's buffer
//                 RtlCopyMemory(outputBuffer, header->Data, dataLengthToCopy);
//                 *DataLength = dataLengthToCopy;
//             }
//         }
//
//         ScratchBuffer_EndUse(DeviceExtension);
//     }
//
//     return status;
// }
//
// _IRQL_requires_max_(APC_LEVEL)
// NTSTATUS
// DeviceHandleAacsGenerateBindingNonce(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDFREQUEST               Request,
//       WDF_REQUEST_PARAMETERS   RequestParameters,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//     This routine is used to process IOCTL:
//         IOCTL_AACS_GENERATE_BINDING_NONCE
// Arguments:
//     DeviceExtension - device context
//
//     Request - the request that will be formatted
//
//     RequestParameters - request parameter structur
//
//     DataLength - data transferred length
//
// Return Value:
//     NTSTATUS
//
//   --*/
// {
//     //AacsGenerateBindingNonce
//
//     NTSTATUS                 status = STATUS_SUCCESS;
//     PAACS_READ_BINDING_NONCE input = NULL;
//     PVOID                    outputBuffer = NULL;
//
//     PAGED_CODE();
//
//     *DataLength = 0;
//
//     status = WdfRequestRetrieveInputBuffer(Request,
//                                            RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
//                                            (PVOID*)&input,
//                                            NULL);
//
//     if (NT_SUCCESS(status))
//     {
//         status = WdfRequestRetrieveOutputBuffer(Request,
//                                                 RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength,
//                                                 (PVOID*)&outputBuffer,
//                                                 NULL);
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         ULONG   dataTransferLength = sizeof(CDVD_KEY_HEADER) + sizeof(AACS_BINDING_NONCE);
//         CDB     cdb;
//
//         ScratchBuffer_BeginUse(DeviceExtension);
//
//         RtlZeroMemory(&cdb, sizeof(CDB));
//         // Set up the CDB
//
//         status = ScratchBuffer_ExecuteCdb(DeviceExtension, Request, dataTransferLength, TRUE, &cdb, 12);
//
// #ifdef ENABLE_AACS_TESTING
//             static const UCHAR results[] = { 0x00, 0x22, 0x00, 0x00 };
//             static const UCHAR defaultFill = 0x37; // '7'
//             RtlFillMemory(DeviceExtension->ScratchContext.ScratchBuffer, 0x0024, defaultFill);
//             RtlCopyMemory(DeviceExtension->ScratchContext.ScratchBuffer, results, SIZEOF_ARRAY(results));
//             status = STATUS_SUCCESS;
// #endif
//         if (NT_SUCCESS(status))
//         {
//             PDVD_DESCRIPTOR_HEADER  header = DeviceExtension->ScratchContext.ScratchBuffer;
//             ULONG                   dataLengthToCopy = sizeof(AACS_BINDING_NONCE);
//
//             // make length field native byte ordering
//             REVERSE_SHORT(&header->Length);
//
//             // exit if getting invalid data from the drive
//             if (header->Length < sizeof(DVD_DESCRIPTOR_HEADER) - RTL_SIZEOF_THROUGH_FIELD(DVD_DESCRIPTOR_HEADER, Length))
//             {
//                 *DataLength = 0;
//                 status = STATUS_IO_DEVICE_ERROR;
//             }
//
//             if (NT_SUCCESS(status))
//             {
//                 // adjust data length to reflect only the addition data
//                 header->Length -= sizeof(DVD_DESCRIPTOR_HEADER) - RTL_SIZEOF_THROUGH_FIELD(DVD_DESCRIPTOR_HEADER, Length);
//
//                 // exit if the drive is returning an unexpected data size
//                 if (header->Length != dataLengthToCopy)
//                 {
//                     *DataLength = 0;
//                     status = STATUS_IO_DEVICE_ERROR;
//                 }
//             }
//
//             if (NT_SUCCESS(status))
//             {
//                 // else copy the data to the user's buffer
//                 RtlCopyMemory(outputBuffer, header->Data, dataLengthToCopy);
//                 *DataLength = dataLengthToCopy;
//             }
//         }
//
//         ScratchBuffer_EndUse(DeviceExtension);
//     }
//
//     return status;
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
