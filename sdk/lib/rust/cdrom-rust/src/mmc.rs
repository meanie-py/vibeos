use crate::types::*;
use kmdf_sys::*;
use ntkrnl_sys::ddk::{IRP, PIRP, IO_STACK_LOCATION, PIO_STACK_LOCATION, GUID, LPGUID, LPCGUID, DEVICE_OBJECT, PDEVICE_OBJECT, MDL, PMDL};
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{ExFreePool, IoAllocateIrp, IoAllocateMdl, IoFreeIrp, IoFreeMdl};
use core::ffi::c_void;

extern "system" {
    fn DeviceGetExtension(Device: WDFDEVICE) -> *mut CDROM_DEVICE_EXTENSION;
    fn DeviceSendSrbSynchronously( Device: WDFDEVICE, Srb: *mut SCSI_REQUEST_BLOCK, Buffer: *mut core::ffi::c_void, BufferSize: u32, ForcePIO: i32, Unknown: *mut core::ffi::c_void, ) -> NTSTATUS;
    fn NT_ASSERT(condition: i32);
}

// WPP tracing constants. These are C *compile-time* constants (`storswtr.h`), NOT linkable
// symbols — the translation wrongly emitted them as `extern "C" static`, which stayed
// latent only because the old no-op `TracePrint` let LLVM fold its arguments away. With
// `TracePrint` now a real extern call (its variadic entry lives in the C shim), the
// arguments are materialised and the bogus externs became `undefined symbol` link errors.
// Declared as `const` here, matching how the sibling modules (pnppower.rs, scratch.rs)
// already do it. Values from sdk/include/ddk/storswtr.h; inert in any case, because
// TracePrint is a no-op in the retail (non-WPP) build.
const TRACE_FLAG_IOCTL: u32 = 4; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)
const TRACE_LEVEL_ERROR: u32 = 2; // storswtr.h:26-30
// C min() is a macro; extern "system" decl = i386 stdcall/cdecl mismatch vs
// ddk_shims' extern "C" def → stack corruption. Local wrapper instead.
#[inline] fn min(a: u32, b: u32) -> u32 { core::cmp::min(a, b) }
extern "system" {
    fn ScratchBuffer_BeginUse(DeviceExtension: *mut CDROM_DEVICE_EXTENSION);
    fn ScratchBuffer_EndUse(DeviceExtension: *mut CDROM_DEVICE_EXTENSION);
    fn ScratchBuffer_ExecuteCdb( DeviceExtension: *mut CDROM_DEVICE_EXTENSION, unknown: *mut core::ffi::c_void, transferSize: u32, flag: i32, cdb: *const CDB, cdbLength: u32, ) -> NTSTATUS;
    fn ScratchBuffer_ExecuteCdbEx( DeviceExtension: *mut CDROM_DEVICE_EXTENSION, Param1: *mut core::ffi::c_void, TransferSize: u32, Flag: u8, /* BOOLEAN TRUE -> 1 */ Cdb: *const CDB, CdbLength: u32, Timeout: u32, ) -> i32; // NTSTATUS
    fn WDF_OBJECT_ATTRIBUTES_INIT_CONTEXT_TYPE( attributes: *mut WDF_OBJECT_ATTRIBUTES, context_type_info: *const core::ffi::c_void, );
}
unsafe extern "C" {
    fn ConvertSectorsPerSecondTo100nsUnitsFor64kWrite(SectorsPerSecond: u32) -> i64;
    fn InterlockedCompareExchange( Destination: *mut i32, Exchange: i32, Comperand: i32, ) -> i32;
}
unsafe extern "C" {
    fn DeviceFindFeaturePage( Buffer: *mut GET_CONFIGURATION_HEADER, Length: u32, Feature: FEATURE_NUMBER, ) -> *mut FEATURE_HEADER;
}

const TRACE_LEVEL_WARNING: u32 = 3; // storswtr.h:26-30
const TRACE_FLAG_INIT: u32 = 8; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)
const SCSIOP_MODE_SENSE10: u8 = 0x5A;
const MODE_PAGE_CAPABILITIES: u8 = 0x2A;

const FeatureDefectManagement: FEATURE_NUMBER = 0x0018;
const FeatureRandomWritable: FEATURE_NUMBER = 0x0020;
const FeatureRandomReadable: FEATURE_NUMBER = 0x0010;
const FeatureRigidRestrictedOverwrite: FEATURE_NUMBER = 0x0025;
const FeatureRestrictedOverwrite: FEATURE_NUMBER = 0x0026;
const FeatureIncrementalStreamingWritable: FEATURE_NUMBER = 0x0021;
const FeatureDvdPlusR: FEATURE_NUMBER = 0x002B;
const FeatureDvdPlusRDualLayer: FEATURE_NUMBER = 0x002D;
const FeatureProfileList: FEATURE_NUMBER = 0x0000;

const ProfileDvdRewritable: FEATURE_PROFILE_TYPE = 0x001A;
const ProfileCdRewritable: FEATURE_PROFILE_TYPE = 0x000A;
const ProfileDvdRecordable: FEATURE_PROFILE_TYPE = 0x0012;
const ProfileCdRecordable: FEATURE_PROFILE_TYPE = 0x0009;

const TRACE_LEVEL_INFORMATION: u32 = 4; // storswtr.h:26-30

// Forward declarations for unknown functions

const CdromMmcUpdateStarted: i32 = 0x2;  // was 0 (LLM-invented); bindgen-correct
const CdromMmcUpdateRequired: i32 = 1;
const CdromMmcUpdateComplete: i32 = 0x0;  // was 2 (LLM-invented); bindgen-correct
const SCSI_GET_CONFIGURATION_REQUEST_TYPE_CURRENT: u32 = 0x1;  // was 2 (LLM-invented); bindgen-correct
const FeatureRealTimeStreaming: FEATURE_NUMBER = 0x0107;
const FeatureDvdCSS: FEATURE_NUMBER = 0x0106;
const MAXLONGLONG: i64 = i64::MAX;
#[no_mangle]
pub unsafe extern "C" fn DeviceDeallocateMmcResources(Device: WDFDEVICE) {
    // _IRQL_requires_max_(APC_LEVEL)
    let device_extension = DeviceGetExtension(Device);
    let cddata = core::ptr::addr_of_mut!((*device_extension).DeviceAdditionalData);
    let mmc_data = core::ptr::addr_of_mut!((*cddata).Mmc);

    // PAGED_CODE();

    if !(*mmc_data).CapabilitiesIrp.is_null() {
        IoFreeIrp((*mmc_data).CapabilitiesIrp);
        (*mmc_data).CapabilitiesIrp = core::ptr::null_mut();
    }
    if !(*mmc_data).CapabilitiesMdl.is_null() {
        IoFreeMdl((*mmc_data).CapabilitiesMdl);
        (*mmc_data).CapabilitiesMdl = core::ptr::null_mut();
    }
    if !(*mmc_data).CapabilitiesBuffer.is_null() {
        ExFreePool((*mmc_data).CapabilitiesBuffer as *mut core::ffi::c_void);
        (*mmc_data).CapabilitiesBuffer = core::ptr::null_mut();
    }
    if !(*mmc_data).CapabilitiesRequest.is_null() {
        WdfObjectDelete((*mmc_data).CapabilitiesRequest as *mut core::ffi::c_void);
        (*mmc_data).CapabilitiesRequest = core::ptr::null_mut();
    }
    (*mmc_data).CapabilitiesBufferSize = 0;
    (*mmc_data).IsMmc = 0; // FALSE
    (*mmc_data).WriteAllowed = 0; // FALSE
}
#[no_mangle]
pub unsafe extern "C" fn DeviceAllocateMmcResources(Device: WDFDEVICE) -> NTSTATUS {
    let mut status: NTSTATUS;
    let deviceExtension: *mut CDROM_DEVICE_EXTENSION = DeviceGetExtension(Device);
    let cddata: *mut CDROM_DATA = &raw mut (*deviceExtension).DeviceAdditionalData;
    let mmcData: *mut CDROM_MMC_EXTENSION = &raw mut (*cddata).Mmc;
    let mut attributes: WDF_OBJECT_ATTRIBUTES = core::mem::zeroed();

    PAGED_CODE!();

    NT_ASSERT(((*mmcData).CapabilitiesBuffer.is_null()) as i32);
    NT_ASSERT(((*mmcData).CapabilitiesBufferSize == 0) as i32);

    status = DeviceGetConfigurationWithAlloc(
        Device,
        &raw mut (*mmcData).CapabilitiesBuffer as *mut *mut GET_CONFIGURATION_HEADER,
        &raw mut (*mmcData).CapabilitiesBufferSize as *mut ULONG,
        FeatureProfileList,
        SCSI_GET_CONFIGURATION_REQUEST_TYPE_ALL,
    );

    if !NT_SUCCESS(status) {
        NT_ASSERT(((*mmcData).CapabilitiesBuffer.is_null()) as i32);
        NT_ASSERT(((*mmcData).CapabilitiesBufferSize == 0) as i32);
        return status;
    }

    NT_ASSERT((!(*mmcData).CapabilitiesBuffer.is_null()) as i32);
    NT_ASSERT(((*mmcData).CapabilitiesBufferSize != 0) as i32);

    (*mmcData).CapabilitiesMdl = IoAllocateMdl(
        (*mmcData).CapabilitiesBuffer as *mut core::ffi::c_void,
        (*mmcData).CapabilitiesBufferSize,
        0, // FALSE
        0, // FALSE
        core::ptr::null_mut(), // NULL
    );

    if (*mmcData).CapabilitiesMdl.is_null() {
        ExFreePool((*mmcData).CapabilitiesBuffer as *mut core::ffi::c_void);
        (*mmcData).CapabilitiesBuffer = core::ptr::null_mut();
        (*mmcData).CapabilitiesBufferSize = 0;
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    // Irp allocation: stack size from DeviceObject->StackSize + 1
    let stackSize: i32 = (*(*deviceExtension).DeviceObject).StackSize as i32 + 1;
    (*mmcData).CapabilitiesIrp = IoAllocateIrp(stackSize as i8, 0); // FALSE -> 0

    if (*mmcData).CapabilitiesIrp.is_null() {
        IoFreeMdl((*mmcData).CapabilitiesMdl);
        (*mmcData).CapabilitiesMdl = core::ptr::null_mut();
        ExFreePool((*mmcData).CapabilitiesBuffer as *mut core::ffi::c_void);
        (*mmcData).CapabilitiesBuffer = core::ptr::null_mut();
        (*mmcData).CapabilitiesBufferSize = 0;
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    // Init WDF object attributes for the request context
    WDF_OBJECT_ATTRIBUTES_INIT_CONTEXT_TYPE(
        &raw mut attributes,
        &raw const _WDF_CDROM_REQUEST_CONTEXT_TYPE_INFO as *const core::ffi::c_void,
    );

    status = WdfRequestCreateFromIrp(
        &raw mut attributes,
        (*mmcData).CapabilitiesIrp,
        0, // FALSE
        &raw mut (*mmcData).CapabilitiesRequest,
    );

    if !NT_SUCCESS(status) {
        return status;
    }

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn DeviceUpdateMmcCapabilities(
    Device: WDFDEVICE,
) -> i32 {
    let mut status: i32; // STATUS_SUCCESS
    let deviceExtension: *mut CDROM_DEVICE_EXTENSION = DeviceGetExtension(Device);
    let cdData: *mut CDROM_DATA = core::ptr::addr_of_mut!((*deviceExtension).DeviceAdditionalData);
    let mmcData: *mut CDROM_MMC_EXTENSION = core::ptr::addr_of_mut!((*cdData).Mmc);
    let mut returnedBytes: u32 = 0;
    let mut updateState: i32;

    updateState = InterlockedCompareExchange(
        core::ptr::addr_of_mut!((*cdData).Mmc.UpdateState) as *mut i32,
        CdromMmcUpdateStarted,
        CdromMmcUpdateRequired,
    );
    if updateState != CdromMmcUpdateRequired {
        return 0; // STATUS_SUCCESS
    }

    (*mmcData).WriteAllowed = 0; // FALSE
    (*mmcData).StreamingReadSupported = 0;
    (*mmcData).StreamingWriteSupported = 0;

    status = DeviceGetConfiguration(
        Device,
        (*mmcData).CapabilitiesBuffer,
        (*mmcData).CapabilitiesBufferSize,
        &mut returnedBytes,
        FeatureProfileList as u16,
        SCSI_GET_CONFIGURATION_REQUEST_TYPE_CURRENT,
    );

    if status >= 0 && ((*mmcData).CapabilitiesBufferSize >= returnedBytes) {
        {
            let mut writeAllowed: u8 = 0;
            let mut validationSchema: FEATURE_NUMBER = 0;
            let mut blockingFactor: u32 = 1;

            DeviceUpdateMmcWriteCapability(
                (*mmcData).CapabilitiesBuffer,
                returnedBytes,
                1, // TRUE
                &mut writeAllowed,
                &mut validationSchema,
                &mut blockingFactor,
            );

            (*mmcData).WriteAllowed = writeAllowed;
            (*mmcData).ValidationSchema = validationSchema;
            (*mmcData).Blocking = blockingFactor;
        }

        {
            let header: *mut FEATURE_HEADER;
            let minAdditionalLength: u32;
            minAdditionalLength = core::mem::offset_of!(FEATURE_DATA_REAL_TIME_STREAMING, Reserved2) as u32
                - core::mem::size_of::<FEATURE_HEADER>() as u32;

            header = MmcDataFindFeaturePage(
                (*mmcData).CapabilitiesBuffer,
                returnedBytes,
                FeatureRealTimeStreaming,
            ) as *mut FEATURE_HEADER;

            if header != core::ptr::null_mut()
                && (*header).Current() != 0
                && (*header).AdditionalLength as u32 >= minAdditionalLength
            {
                let feature: *mut FEATURE_DATA_REAL_TIME_STREAMING =
                    header as *mut FEATURE_DATA_REAL_TIME_STREAMING;
                (*mmcData).StreamingReadSupported = 1; // TRUE
                (*mmcData).StreamingWriteSupported = if (*feature).StreamRecording() == 1 { 1 } else { 0 };
            }
        }

        {
            let header: *mut FEATURE_HEADER;
            header = DeviceFindFeaturePage(
                (*mmcData).CapabilitiesBuffer,
                returnedBytes,
                FeatureDvdCSS,
            );
            (*mmcData).IsCssDvd = (header != core::ptr::null_mut() && (*header).Current() != 0) as u8;
        }

        {
            let profile: FEATURE_PROFILE_TYPE =
                ((((*((*mmcData).CapabilitiesBuffer)).CurrentProfile[0] as u16) << 8) |
                (((*((*mmcData).CapabilitiesBuffer)).CurrentProfile[1] as u16) << 0)) as FEATURE_PROFILE_TYPE;
            let mut t1: i64;
            let mut t2: i64;
            let mut t3: i64;
            let mut t4: i64;
            let mut final_val: i64;

            t1 = DeviceRetryTimeGuessBasedOnProfile(profile);
            t2 = DeviceRetryTimeDetectionBasedOnModePage2A(deviceExtension);
            t3 = DeviceRetryTimeDetectionBasedOnGetPerformance(deviceExtension, 1);
            t4 = DeviceRetryTimeDetectionBasedOnGetPerformance(deviceExtension, 0);

            final_val = MAXLONGLONG;
            if t4 != -1 {
                if t4 < final_val { final_val = t4; }
            }
            if t3 != -1 {
                if t3 < final_val { final_val = t3; }
            }
            if t2 != -1 {
                if t2 < final_val { final_val = t2; }
            }
            if t1 != -1 {
                if t1 < final_val { final_val = t1; }
            }
            if final_val == MAXLONGLONG {
                final_val = WRITE_RETRY_DELAY_CD_4x;
            }

            (*cdData).ReadWriteRetryDelay100nsUnits = final_val;
        }
    } else {
        (*cdData).Mmc.UpdateState = CdromMmcUpdateRequired;
    }

    InterlockedCompareExchange(
        core::ptr::addr_of_mut!((*cdData).Mmc.UpdateState) as *mut i32,
        CdromMmcUpdateComplete,
        CdromMmcUpdateStarted,
    );

    status
}
#[no_mangle]
pub unsafe extern "system" fn DeviceGetConfigurationWithAlloc(
    Device: WDFDEVICE,
    Buffer: *mut *mut GET_CONFIGURATION_HEADER,
    BytesReturned: *mut u32,
    StartingFeature: FEATURE_NUMBER,
    RequestedType: u32,
) -> i32 {
    // Forward declarations for external symbols not in imports
    const STATUS_INTERNAL_ERROR: i32 = (0xC000000Fu32 as i32);
    const CDROM_TAG_FEATURE: u32 = 0x46436353; // 'CDDR' little-endian? typical tag
    // cdrom.h:68: `#define NonPagedPoolNxCacheAligned NonPagedPoolCacheAligned` under
    // `#ifdef __REACTOS__` -- NX pool is NT6.2 and this kernel has none, so the
    // Windows value (0x200/0x204) is not what any TU here compiles.
    const NonPagedPoolNxCacheAligned: u32 = 4;
    const TRACE_LEVEL_ERROR: u32 = 2; // storswtr.h:26-30
    const TRACE_FLAG_IOCTL: u32 = 4; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)

    let mut status: i32;
    let mut header: GET_CONFIGURATION_HEADER = core::mem::zeroed();
    let mut buffer: *mut GET_CONFIGURATION_HEADER = core::ptr::null_mut();
    let mut returned: u32 = 0;
    let mut size: u32;
    let mut i: u32 = 0;

    // PAGED_CODE();
    unsafe {
        *Buffer = core::ptr::null_mut();
        *BytesReturned = 0;

        // send the first request down to just get the header
        status = DeviceGetConfiguration(
            Device,
            &mut header,
            core::mem::size_of::<GET_CONFIGURATION_HEADER>() as u32,
            &mut returned,
            StartingFeature as u16,
            RequestedType,
        );

        // now send command again, using information returned to allocate just enough memory
        if NT_SUCCESS(status) {
            size = ((header.DataLength[0] as u32) << 24)
                | ((header.DataLength[1] as u32) << 16)
                | ((header.DataLength[2] as u32) << 8)
                | ((header.DataLength[3] as u32) << 0);

            // the loop is in case that the retrieved data length is bigger than last time reported.
            i = 0;
            while i < 4 && NT_SUCCESS(status) {
                // the datalength field is the size *following* itself, so adjust accordingly
                size += 4 * 1; // sizeof(UCHAR)

                // make sure the size is reasonable
                if size <= core::mem::size_of::<FEATURE_HEADER>() as u32 {
                    TracePrint!(
                        TRACE_LEVEL_ERROR,
                        TRACE_FLAG_IOCTL,
                        b"DeviceGetConfigurationWithAlloc: drive reports only %x bytes?\n\0"
                            .as_ptr() as *const i8,
                        size,
                    );
                    status = STATUS_UNSUCCESSFUL;
                }

                if NT_SUCCESS(status) {
                    // allocate the memory
                    buffer = ExAllocatePoolWithTag(
                        NonPagedPoolNxCacheAligned,
                        size as usize,
                        CDROM_TAG_FEATURE,
                    ) as *mut GET_CONFIGURATION_HEADER;

                    if buffer.is_null() {
                        status = STATUS_INSUFFICIENT_RESOURCES;
                    }
                }

                if NT_SUCCESS(status) {
                    // send the request down to get the full configuration
                    status = DeviceGetConfiguration(
                        Device,
                        buffer,
                        size,
                        &mut returned,
                        StartingFeature as u16,
                        RequestedType,
                    );

                    if !NT_SUCCESS(status) {
                        ExFreePool(buffer as *mut core::ffi::c_void);
                    } else if returned > size {
                        ExFreePool(buffer as *mut core::ffi::c_void);
                        status = STATUS_INTERNAL_ERROR;
                    }
                }

                // command succeeded.
                if NT_SUCCESS(status) {
                    returned = (((*buffer).DataLength[0] as u32) << 24)
                        | (((*buffer).DataLength[1] as u32) << 16)
                        | (((*buffer).DataLength[2] as u32) << 8)
                        | (((*buffer).DataLength[3] as u32) << 0);
                    returned += 4 * 1; // sizeof(UCHAR)

                    if returned <= size {
                        *Buffer = buffer;
                        *BytesReturned = returned; // amount of 'safe' memory
                        status = STATUS_SUCCESS;
                        break;
                    } else {
                        // the data size is bigger than the buffer size, retry using new size....
                        size = returned;
                        ExFreePool(buffer as *mut core::ffi::c_void);
                        buffer = core::ptr::null_mut();
                    }
                }

                i += 1;
            } // end of while() loop
        }

        if !NT_SUCCESS(status) {
            // it failed after a number of attempts, so just fail.
            TracePrint!(
                TRACE_LEVEL_ERROR,
                TRACE_FLAG_IOCTL,
                b"DeviceGetConfigurationWithAlloc: Failed %d attempts to get all feature information\n\0"
                    .as_ptr() as *const i8,
                i,
            );
        }
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn DeviceGetConfiguration(
    Device: WDFDEVICE,
    Buffer: *mut GET_CONFIGURATION_HEADER,
    BufferSize: u32,
    ValidBytes: *mut u32,
    StartingFeature: u16,
    RequestedType: u32,
) -> NTSTATUS {
    // PAGED_CODE();

    NT_ASSERT((!ValidBytes.is_null()) as i32);

    if Buffer.is_null() || BufferSize < core::mem::size_of::<GET_CONFIGURATION_HEADER>() as u32 {
        return STATUS_BUFFER_TOO_SMALL;
    }

    *ValidBytes = 0;

    let deviceExtension = DeviceGetExtension(Device);
    let mut srb: SCSI_REQUEST_BLOCK = core::mem::zeroed();
    core::ptr::write_bytes(Buffer as *mut u8, 0, BufferSize as usize);

    if ((*deviceExtension).DeviceAdditionalData.HackFlags & CDROM_HACK_BAD_GET_CONFIG_SUPPORT as ntkrnl_sys::ddk::ULONG_PTR) != 0 {
        return STATUS_INVALID_DEVICE_REQUEST;
    }

    if BufferSize > 0xFFFC {
        return DeviceGetConfiguration(Device, Buffer, 0xFFFC_u32, ValidBytes, StartingFeature, RequestedType);
    }

    srb.TimeOutValue = CDROM_GET_CONFIGURATION_TIMEOUT;
    srb.CdbLength = 10;

    let cdb = &raw mut srb.Cdb as *mut CDB;
    (*cdb).GET_CONFIGURATION.as_mut().OperationCode = SCSIOP_GET_CONFIGURATION as u8;
    (*cdb).GET_CONFIGURATION.as_mut().set_RequestType(RequestedType as u8);
    (*cdb).GET_CONFIGURATION.as_mut().StartingFeature[0] = (StartingFeature >> 8) as u8;
    (*cdb).GET_CONFIGURATION.as_mut().StartingFeature[1] = (StartingFeature & 0xff) as u8;
    (*cdb).GET_CONFIGURATION.as_mut().AllocationLength[0] = (BufferSize >> 8) as u8;
    (*cdb).GET_CONFIGURATION.as_mut().AllocationLength[1] = (BufferSize & 0xff) as u8;

    let mut status = DeviceSendSrbSynchronously(
        Device,
        &raw mut srb,
        Buffer as *mut core::ffi::c_void,
        BufferSize,
        0,
        core::ptr::null_mut(),
    );

    TracePrint!(
        TRACE_LEVEL_INFORMATION,
        TRACE_FLAG_IOCTL,
        b"DeviceGetConfiguration: Status was %x\n\0".as_ptr() as *const i8,
        status,
    );

    if ntkrnl_sys::base::NT_SUCCESS(status)
        || status == STATUS_BUFFER_OVERFLOW
        || status == STATUS_DATA_OVERRUN
    {
        let returned: u32 = srb.DataTransferLength;
        let header = Buffer;
        let available = {
            let data_length = &(*header).DataLength;
            let raw = ((data_length[0] as u32) << 24)
                | ((data_length[1] as u32) << 16)
                | ((data_length[2] as u32) << 8)
                | (data_length[3] as u32);
            raw + (core::mem::offset_of!(GET_CONFIGURATION_HEADER, DataLength)
                + core::mem::size_of::<[u8; 4]>()) as u32
        };
        // _Analysis_assume_(returned <= BufferSize);
        let mut returned_val = returned;
        if available < returned_val {
            returned_val = available;
        }
        NT_ASSERT((returned_val <= BufferSize) as i32);
        *ValidBytes = returned_val;
        status = STATUS_SUCCESS;
    } else {
        TracePrint!(
            TRACE_LEVEL_ERROR,
            TRACE_FLAG_IOCTL,
            b"DeviceGetConfiguration: failed %x\n\0".as_ptr() as *const i8,
            status,
        );
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn DeviceUpdateMmcWriteCapability(
    Buffer: *mut GET_CONFIGURATION_HEADER,
    BufferSize: u32,
    CurrentOnly: u8,
    Writable: *mut u8,
    ValidationSchema: *mut FEATURE_NUMBER,
    BlockingFactor: *mut u32,
) {
    PAGED_CODE!();

    *Writable = 0u8;
    *ValidationSchema = 0;
    *BlockingFactor = 1;

    /* if the drive supports hardware defect management and random writes, that's sufficient */
    {
        let defect_header: *mut FEATURE_HEADER;
        let writable_header: *mut FEATURE_HEADER;

        defect_header = MmcDataFindFeaturePage(Buffer, BufferSize, FeatureDefectManagement) as *mut FEATURE_HEADER;
        writable_header = MmcDataFindFeaturePage(Buffer, BufferSize, FeatureRandomWritable) as *mut FEATURE_HEADER;

        if defect_header.is_null() || writable_header.is_null() {
            /* cannot write this way */
        } else if CurrentOnly == 0u8 {
            TracePrint!(
                TRACE_LEVEL_INFORMATION,
                TRACE_FLAG_INIT,
                b"DeviceUpdateMmcWriteCapability => Writes supported (defect management)\n\0".as_ptr() as *const i8,
            );
            *Writable = 1u8;
            return;
        } else if (*defect_header).Current() != 0 && (*writable_header).Current() != 0 {
            TracePrint!(
                TRACE_LEVEL_INFORMATION,
                TRACE_FLAG_INIT,
                b"DeviceUpdateMmcWriteCapability => Writes *allowed* (defect management)\n\0".as_ptr() as *const i8,
            );
            *Writable = 1u8;
            *ValidationSchema = FeatureDefectManagement;
            return;
        }
    }

    /* Certain validation schema require the blocking factor */
    {
        let header: *mut FEATURE_HEADER;
        let additional_length: u32;

        additional_length = ((core::mem::offset_of!(FEATURE_DATA_RANDOM_READABLE, Blocking) + core::mem::size_of::<[u8; 2]>()) - core::mem::size_of::<FEATURE_HEADER>()) as u32;

        header = MmcDataFindFeaturePage(Buffer, BufferSize, FeatureRandomReadable) as *mut FEATURE_HEADER;

        if !header.is_null()
            && (*header).Current() != 0
            && (*header).AdditionalLength as u32 >= additional_length
        {
            let feature = header as *mut FEATURE_DATA_RANDOM_READABLE;
            *BlockingFactor = ((*feature).Blocking[0] as u32) << 8 | (*feature).Blocking[1] as u32;
        }
    }

    /* the majority of features to indicate write capability */
    {
        const FeaturesToAllowWritesWith: [(FEATURE_NUMBER, FEATURE_NUMBER); 4] = [
            (FeatureRandomWritable, FeatureRandomWritable),
            (FeatureRigidRestrictedOverwrite, FeatureRigidRestrictedOverwrite),
            (FeatureRestrictedOverwrite, FeatureRestrictedOverwrite),
            (FeatureIncrementalStreamingWritable, FeatureIncrementalStreamingWritable),
        ];

        let mut count: usize;
        for count in 0..FeaturesToAllowWritesWith.len() {
            let (feature_to_find, validation_schema) = FeaturesToAllowWritesWith[count];
            let header = MmcDataFindFeaturePage(Buffer, BufferSize, feature_to_find) as *mut FEATURE_HEADER;
            if header.is_null() {
                /* cannot write using this method */
            } else if CurrentOnly == 0u8 {
                TracePrint!(
                    TRACE_LEVEL_INFORMATION,
                    TRACE_FLAG_INIT,
                    b"DeviceUpdateMmcWriteCapability => Writes supported (feature %04x)\n\0".as_ptr() as *const i8,
                    feature_to_find,
                );
                *Writable = 1u8;
                return;
            } else if (*header).Current() != 0 {
                TracePrint!(
                    TRACE_LEVEL_INFORMATION,
                    TRACE_FLAG_INIT,
                    b"DeviceUpdateMmcWriteCapability => Writes *allowed* (feature %04x)\n\0".as_ptr() as *const i8,
                    feature_to_find,
                );
                *Writable = 1u8;
                *ValidationSchema = validation_schema;
                return;
            }
        }
    }

    /* DVD+R feature special case */
    {
        let header: *mut FEATURE_DATA_DVD_PLUS_R;
        let additional_length = core::mem::offset_of!(FEATURE_DATA_DVD_PLUS_R, Reserved2) - core::mem::size_of::<FEATURE_HEADER>();

        header = MmcDataFindFeaturePage(Buffer, BufferSize, FeatureDvdPlusR) as *mut FEATURE_DATA_DVD_PLUS_R;

        if header.is_null() || ((*header).Header.AdditionalLength as usize) < additional_length || (*header).Write() == 0 {
            /* cannot write this way */
        } else if CurrentOnly == 0u8 {
            TracePrint!(
                TRACE_LEVEL_INFORMATION,
                TRACE_FLAG_INIT,
                b"DeviceUpdateMmcWriteCapability => Writes supported (feature %04x)\n\0".as_ptr() as *const i8,
                FeatureDvdPlusR,
            );
            *Writable = 1u8;
            return;
        } else if (*header).Header.Current() != 0 {
            TracePrint!(
                TRACE_LEVEL_INFORMATION,
                TRACE_FLAG_INIT,
                b"DeviceUpdateMmcWriteCapability => Writes *allowed* (feature %04x)\n\0".as_ptr() as *const i8,
                FeatureDvdPlusR,
            );
            *Writable = 1u8;
            *ValidationSchema = FeatureIncrementalStreamingWritable;
            return;
        }
    }

    /* DVD+R DL feature special case */
    {
        let header: *mut FEATURE_DATA_DVD_PLUS_R_DUAL_LAYER;
        let additional_length = core::mem::offset_of!(FEATURE_DATA_DVD_PLUS_R_DUAL_LAYER, Reserved2) - core::mem::size_of::<FEATURE_HEADER>();

        header = MmcDataFindFeaturePage(Buffer, BufferSize, FeatureDvdPlusRDualLayer) as *mut FEATURE_DATA_DVD_PLUS_R_DUAL_LAYER;

        if header.is_null() || ((*header).Header.AdditionalLength as usize) < additional_length || (*header).Write() == 0 {
            /* cannot write this way */
        } else if CurrentOnly == 0u8 {
            TracePrint!(
                TRACE_LEVEL_INFORMATION,
                TRACE_FLAG_INIT,
                b"DeviceUpdateMmcWriteCapability => Writes supported (feature %04x)\n\0".as_ptr() as *const i8,
                FeatureDvdPlusRDualLayer,
            );
            *Writable = 1u8;
            return;
        } else if (*header).Header.Current() != 0 {
            TracePrint!(
                TRACE_LEVEL_INFORMATION,
                TRACE_FLAG_INIT,
                b"DeviceUpdateMmcWriteCapability => Writes *allowed* (feature %04x)\n\0".as_ptr() as *const i8,
                FeatureDvdPlusRDualLayer,
            );
            *Writable = 1u8;
            *ValidationSchema = FeatureIncrementalStreamingWritable;
            return;
        }
    }

    /* use profiles also */
    {
        let header: *mut FEATURE_HEADER;
        header = MmcDataFindFeaturePage(Buffer, BufferSize, FeatureProfileList) as *mut FEATURE_HEADER;

        if !header.is_null() && (*header).Current() != 0 {
            let buffer_end = (Buffer as *mut u8).add(BufferSize as usize);
            let header_end = (header as *mut u8).add((*header).AdditionalLength as usize + core::mem::size_of::<FEATURE_HEADER>());

            if buffer_end >= header_end {
                const ProfilesToAllowWritesWith: [(FEATURE_PROFILE_TYPE, FEATURE_NUMBER); 4] = [
                    (ProfileDvdRewritable, FeatureRigidRestrictedOverwrite),
                    (ProfileCdRewritable, FeatureRestrictedOverwrite),
                    (ProfileDvdRecordable, FeatureIncrementalStreamingWritable),
                    (ProfileCdRecordable, FeatureIncrementalStreamingWritable),
                ];

                let mut count: usize;
                for count in 0..ProfilesToAllowWritesWith.len() {
                    let (profile_to_find, validation_schema) = ProfilesToAllowWritesWith[count];
                    let mut exists: u8 = 0u8;
                    MmcDataFindProfileInProfiles(
                        header as *const FEATURE_DATA_PROFILE_LIST,
                        profile_to_find,
                        CurrentOnly,
                        core::ptr::addr_of_mut!(exists),
                    );
                    if exists != 0 {
                        let msg = if CurrentOnly != 0u8 {
                            b"DeviceUpdateMmcWriteCapability => Writes *allowed* (profile %04x)\n\0"
                        } else {
                            b"DeviceUpdateMmcWriteCapability => Writes supported (profile %04x)\n\0"
                        };
                        TracePrint!(
                            TRACE_LEVEL_INFORMATION,
                            TRACE_FLAG_INIT,
                            msg.as_ptr() as *const i8,
                            FeatureDvdPlusR,
                        );
                        *Writable = 1u8;
                        *ValidationSchema = validation_schema;
                        return;
                    }
                }
            }
        }
    }

    return;
}

#[no_mangle]
pub unsafe extern "C" fn MmcDataFindFeaturePage(
    FeatureBuffer: *mut GET_CONFIGURATION_HEADER,
    Length: u32,
    Feature: FEATURE_NUMBER,
) -> *mut core::ffi::c_void {
    let mut buffer: *mut u8;
    let mut limit: *mut u8;
    let mut valid_length: u32;

    // PAGED_CODE() - no-op for translation

    if Length < (core::mem::size_of::<GET_CONFIGURATION_HEADER>() + core::mem::size_of::<FEATURE_HEADER>()) as u32 {
        return core::ptr::null_mut();
    }

    // REVERSE_BYTES(&valid_length, FeatureBuffer->DataLength)
    valid_length = {
        let dl = &(*FeatureBuffer).DataLength;
        ((dl[0] as u32) << 24) | ((dl[1] as u32) << 16) | ((dl[2] as u32) << 8) | (dl[3] as u32)
    };

    // RTL_SIZEOF_THROUGH_FIELD(GET_CONFIGURATION_HEADER, DataLength)
    valid_length = valid_length.wrapping_add(
        (core::mem::offset_of!(GET_CONFIGURATION_HEADER, DataLength) + core::mem::size_of::<u32>()) as u32,
    );

    limit = FeatureBuffer as *mut u8;
    limit = limit.wrapping_add(core::cmp::min(Length, valid_length) as usize);

    buffer = (FeatureBuffer as *mut u8).add(core::mem::offset_of!(GET_CONFIGURATION_HEADER, Data));

    while buffer.add(core::mem::size_of::<FEATURE_HEADER>()) <= limit {
        let header: *mut FEATURE_HEADER = buffer as *mut FEATURE_HEADER;
        let mut this_feature: FEATURE_NUMBER;

        this_feature = (((*header).FeatureCode[0] as FEATURE_NUMBER) << 8)
            | ((*header).FeatureCode[1] as FEATURE_NUMBER);

        if this_feature == Feature {
            let mut temp: *mut u8;

            temp = buffer;
            temp = temp.add(core::mem::size_of::<FEATURE_HEADER>());
            temp = temp.add((*header).AdditionalLength as usize);

            if temp > limit {
                TracePrint!(
                    3, // TRACE_LEVEL_WARNING
                    0x00000010, // TRACE_FLAG_GENERAL
                    b"Feature %x exists, but not safe to access all its data.  returning NULL\n"
                        .as_ptr() as *const i8,
                    Feature as u32,
                );
                return core::ptr::null_mut();
            } else {
                return buffer as *mut core::ffi::c_void;
            }
        }

        if ((*header).AdditionalLength % 4) != 0
            && !(Feature >= 0xff00 && Feature <= 0xffff)
        {
            return core::ptr::null_mut();
        }

        buffer = buffer.add(core::mem::size_of::<FEATURE_HEADER>());
        buffer = buffer.add((*header).AdditionalLength as usize);
    }

    core::ptr::null_mut()
}
#[no_mangle]
pub unsafe extern "C" fn MmcDataFindProfileInProfiles(
    ProfileHeader: *const FEATURE_DATA_PROFILE_LIST,
    ProfileToFind: FEATURE_PROFILE_TYPE,
    CurrentOnly: BOOLEAN,
    Found: PBOOLEAN,
) {
    // initializations
    let mut profile: *const FEATURE_DATA_PROFILE_LIST_EX;
    let mut numberOfProfiles: u32;
    let mut i: u32;

    // initialize output
    *Found = 0; // FALSE

    // sanity check
    if (*ProfileHeader).Header.AdditionalLength % 2 != 0 {
        TracePrint!(
            3 /* TRACE_LEVEL_WARNING */,
            0x40 /* TRACE_FLAG_GENERAL */,
            b"Profile total length %x is not integral multiple of 4\n\0".as_ptr() as *const i8,
            (*ProfileHeader).Header.AdditionalLength as core::ffi::c_uint,
        );
        ntkrnl_sys::ASSERT!(false); // NT_ASSERT(FALSE)
        return;
    }

    // calculate number of profiles
    numberOfProfiles = (*ProfileHeader).Header.AdditionalLength as u32 / 4;
    profile = core::ptr::addr_of!((*ProfileHeader).Profiles) as *const FEATURE_DATA_PROFILE_LIST_EX; // zero-sized array

    // loop through profiles
    for i in 0..numberOfProfiles {
        let currentProfile: FEATURE_PROFILE_TYPE;

        currentProfile = ((((*profile).ProfileNumber[0] as u16) << 8) | ((*profile).ProfileNumber[1] as u16 & 0xff)) as FEATURE_PROFILE_TYPE;

        if currentProfile == ProfileToFind {
            if (*profile).Current() != 0 || CurrentOnly == 0 {
                *Found = 1; // TRUE
            }
        }

        profile = profile.add(1);
    }
}

// C cdrom.h:116-121 (100ns units): CD_4x=1066667 (106.7ms), DVD_1x=473373 (47.3ms),
// HDDVD_1x=143498 (14.3ms). Were 1000/2000/3000 (LLM-invented ~0.1ms placeholders,
// ~1000x too small — a busy-retry storm on any read error).
const WRITE_RETRY_DELAY_CD_4x: i64 = 1_066_667;
const WRITE_RETRY_DELAY_DVD_1x: i64 = 473_373;
const WRITE_RETRY_DELAY_HDDVD_1x: i64 = 143_498;
#[no_mangle]
pub unsafe extern "C" fn DeviceRetryTimeGuessBasedOnProfile(Profile: FEATURE_PROFILE_TYPE) -> i64 {
    // PAGED_CODE();
    let mut result: i64 = -1;
    let profile_num = Profile as u32;
    match profile_num {
        // Invalid, non-removable, removable, MO erasable, write-once, AS-MO, non-standard, and default — no-op
        0x0000 | 0x0001 | 0x0002 | 0x0003 | 0x0004 | 0x0005 | 0xFFFF => { /* NOTHING */ }
        // CD profiles
        0x0008 | 0x0009 | 0x000A | 0x0020 | 0x0021 | 0x0022 => {
            result = WRITE_RETRY_DELAY_CD_4x;
        }
        // DVD (recordable, rewritable, RAM, +RW, +R, layer jump, etc.)
        0x0010 | 0x0011 | 0x0012 | 0x0013 | 0x0014 | 0x0016 | 0x001A | 0x001B => {
            result = WRITE_RETRY_DELAY_DVD_1x;
        }
        // Dual‑layer DVD profiles
        0x0015 | 0x002A | 0x002B => {
            result = WRITE_RETRY_DELAY_DVD_1x;
        }
        // BD profiles
        0x0040 | 0x0041 | 0x0042 | 0x0043 => {
            result = WRITE_RETRY_DELAY_HDDVD_1x;
        }
        // HDDVD profiles
        0x0050 | 0x0051 | 0x0052 => {
            result = WRITE_RETRY_DELAY_HDDVD_1x;
        }
        _ => {} // any other profile number stays at -1 (no idea)
    }
    result
}
#[no_mangle]
pub unsafe extern "C" fn DeviceRetryTimeDetectionBasedOnModePage2A(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
) -> i64 {
    let mut result: i64 = -1;
    let transferSize = min(0xFFF0u32, (*DeviceExtension).ScratchContext.ScratchBufferSize);
    let mut cdb: CDB = core::mem::zeroed();
    PAGED_CODE!();
    ScratchBuffer_BeginUse(DeviceExtension);
    RtlZeroMemory(
        &mut cdb as *mut CDB as *mut core::ffi::c_void,
        core::mem::size_of::<CDB>(),
    );
    let cdb_ptr: *mut CDB = &mut cdb;
    (*cdb_ptr).MODE_SENSE10.as_mut().OperationCode = SCSIOP_MODE_SENSE10;
    (*cdb_ptr).MODE_SENSE10.as_mut().set_Dbd(1);
    (*cdb_ptr).MODE_SENSE10.as_mut().set_PageCode(MODE_PAGE_CAPABILITIES);
    (*cdb_ptr).MODE_SENSE10.as_mut().AllocationLength[0] = (transferSize >> 8) as u8;
    (*cdb_ptr).MODE_SENSE10.as_mut().AllocationLength[1] = (transferSize & 0xFF) as u8;
    let status = ScratchBuffer_ExecuteCdb(
        DeviceExtension,
        core::ptr::null_mut(),
        transferSize,
        1,
        &cdb,
        10u32,
    );
    if NT_SUCCESS(status) {
        let header = (*DeviceExtension).ScratchContext.ScratchBuffer as *const MODE_PARAMETER_HEADER10;
        let mut dataLength: u32 =
            ((*header).ModeDataLength[0] as u32) << 8 | ((*header).ModeDataLength[1] as u32);
        if dataLength != 0 {
            dataLength +=
                (core::mem::offset_of!(MODE_PARAMETER_HEADER10, ModeDataLength) as u32) + 2;
        }
        let mut page: *const CDVD_CAPABILITIES_PAGE = core::ptr::null();
        if dataLength < core::mem::size_of::<MODE_PARAMETER_HEADER10>() as u32 {
            dataLength = 0;
        } else if dataLength > (*DeviceExtension).ScratchContext.ScratchBufferSize {
            dataLength = 0;
        } else if ((*header).BlockDescriptorLength[1] == 0) && ((*header).BlockDescriptorLength[0] == 0) {
            dataLength -= core::mem::size_of::<MODE_PARAMETER_HEADER10>() as u32;
            page = header.wrapping_add(1) as *const CDVD_CAPABILITIES_PAGE;
        } else if ((*header).BlockDescriptorLength[1] == 0)
            && ((*header).BlockDescriptorLength[0] == core::mem::size_of::<MODE_PARAMETER_BLOCK>() as u8)
        {
            dataLength -= core::mem::size_of::<MODE_PARAMETER_HEADER10>() as u32;
            dataLength -= min(dataLength, core::mem::size_of::<MODE_PARAMETER_BLOCK>() as u32);
            page = (header as *const u8)
                .wrapping_add(core::mem::size_of::<MODE_PARAMETER_HEADER10>())
                .wrapping_add(core::mem::size_of::<MODE_PARAMETER_BLOCK>())
                as *const CDVD_CAPABILITIES_PAGE;
        }
        if !page.is_null()
            && dataLength
                >= (core::mem::offset_of!(CDVD_CAPABILITIES_PAGE, PageLength) as u32) + 1
        {
            dataLength = min(dataLength, ((*page).PageLength as u32) + 2);
        }
        if !page.is_null()
            && dataLength
                < (core::mem::offset_of!(CDVD_CAPABILITIES_PAGE, __bindgen_anon_3) as u32) + 2
        {
            TracePrint!(
                TRACE_LEVEL_WARNING,
                TRACE_FLAG_INIT,
                b"ModePage 2Ah was requested, but drive reported only %x bytes (%x needed). Ignoring.\n\0"
                    .as_ptr() as *const i8,
                dataLength,
                (core::mem::offset_of!(CDVD_CAPABILITIES_PAGE, __bindgen_anon_3) as u32) + 2,
            );
            page = core::ptr::null();
        }
        if !page.is_null() && ((*page).PageCode() != MODE_PAGE_CAPABILITIES) {
            TracePrint!(
                TRACE_LEVEL_WARNING,
                TRACE_FLAG_INIT,
                b"ModePage 2Ah was requested, but drive reported page %x\n\0"
                    .as_ptr() as *const i8,
                (*page).PageCode() as core::ffi::c_uint,
            );
            page = core::ptr::null();
        }
        if !page.is_null() {
            let mut temp: u32 =
                ((*page).__bindgen_anon_3.WriteSpeedMaximum.as_ref()[0] as u32) << 8 | ((*page).__bindgen_anon_3.WriteSpeedMaximum.as_ref()[1] as u32);
            temp *= 1000;
            temp /= 2048;
            if temp != 0 {
                result = ConvertSectorsPerSecondTo100nsUnitsFor64kWrite(temp);
            }
        }
    }
    ScratchBuffer_EndUse(DeviceExtension);
    result
}
#[no_mangle]
pub unsafe extern "C" fn DeviceRetryTimeDetectionBasedOnGetPerformance(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    UseLegacyNominalPerformance: u8,
) -> i64 {
    // ---- external declarations ----

    // ---- constants ----
    const SCSIOP_GET_PERFORMANCE: u8 = 0xAC;
    const CDROM_GET_PERFORMANCE_TIMEOUT: u32 = 0x14; // placeholder  // was 15 (LLM-invented); bindgen-correct
    const MAXIMUM_RETRY_FOR_SINGLE_IO_IN_100NS_UNITS: i64 = 0x3B9ACA00; // 1e9 (classpnp.h)

    // ---- local struct types (mirror C definitions exactly) ----
    #[repr(C)]
    struct GET_PERFORMANCE_HEADER {
        TotalDataLength: [u8; 4],
        _bitfield_byte: u8, // bit0=Except, bit1=Write, bits2..7=Reserved0
        Reserved1: [u8; 3],
    }
    const _: () = assert!(core::mem::size_of::<GET_PERFORMANCE_HEADER>() == 8);

    #[repr(C)]
    struct GET_PERFORMANCE_NOMINAL_PERFORMANCE_DESCRIPTOR {
        StartLba: [u8; 4],
        StartPerformance: [u8; 4],
        EndLba: [u8; 4],
        EndPerformance: [u8; 4],
    }
    const _: () = assert!(core::mem::size_of::<GET_PERFORMANCE_NOMINAL_PERFORMANCE_DESCRIPTOR>() == 16);

    #[repr(C)]
    struct GET_PERFORMANCE_WRITE_SPEED_DESCRIPTOR {
        _bitfield0: u8, // MixedReadWrite:1, GuaranteedForWholeMedia:1, Reserved0_RDD:1, WriteRotationControl:2, Reserved1:3
        Reserved2: [u8; 3],
        MediaCapacity: [u8; 4],
        ReadSpeedKilobytesPerSecond: [u8; 4],
        WriteSpeedKilobytesPerSecond: [u8; 4],
    }
    const _: () = assert!(core::mem::size_of::<GET_PERFORMANCE_WRITE_SPEED_DESCRIPTOR>() == 16);

    // Both descriptors are the same size
    const _: () = assert!(
        core::mem::size_of::<GET_PERFORMANCE_WRITE_SPEED_DESCRIPTOR>()
            == core::mem::size_of::<GET_PERFORMANCE_NOMINAL_PERFORMANCE_DESCRIPTOR>()
    );

    // The field of interest is at the same offset and has the same size in both descriptor types
    const _: () = assert!(
        core::mem::offset_of!(GET_PERFORMANCE_WRITE_SPEED_DESCRIPTOR, WriteSpeedKilobytesPerSecond)
            == core::mem::offset_of!(GET_PERFORMANCE_NOMINAL_PERFORMANCE_DESCRIPTOR, EndPerformance)
    );
    const _: () = assert!(
        core::mem::size_of::<[u8; 4]>()
            == core::mem::size_of::<[u8; 4]>()
    );

    // The CDB union and its GET_PERFORMANCE member

    #[repr(C)]
    struct GET_PERFORMANCE_CDB {
        OperationCode: u8,
        _bitfield_byte: u8, // bit0=Except, bit1=Write, bits2..7=Reserved0
        Reserved1: [u8; 3],
        Tolerance: u8,
        MaximumNumberOfDescriptors: [u8; 2],
        Type: u8,
        // remaining bytes are not touched by this code
    }
    const _: () = assert!(core::mem::size_of::<CDB>() == 16);

    // ---- local variables ----
    let mut result: i64 = -1;
    let transferSize: u32;
    let mut validDescriptors: u32 = 0;
    let mut cdb: CDB = unsafe { core::mem::zeroed() };

    unsafe {
        PAGED_CODE!();
        ScratchBuffer_BeginUse(DeviceExtension);

        // RtlZeroMemory(&cdb, sizeof(CDB))
        core::ptr::write_bytes(&mut cdb as *mut CDB as *mut u8, 0, core::mem::size_of::<CDB>());
    }

    // compute maxDescriptors
    let scratchBufSize = unsafe { (*DeviceExtension).ScratchContext.ScratchBufferSize };
    let maxDescriptors = {
        let tmp1 = 200u32;
        let tmp2 = (scratchBufSize - core::mem::size_of::<GET_PERFORMANCE_HEADER>() as u32)
            / core::mem::size_of::<GET_PERFORMANCE_WRITE_SPEED_DESCRIPTOR>() as u32;
        if tmp1 < tmp2 { tmp1 } else { tmp2 }
    };
    transferSize = core::mem::size_of::<GET_PERFORMANCE_HEADER>() as u32
        + (maxDescriptors * core::mem::size_of::<GET_PERFORMANCE_WRITE_SPEED_DESCRIPTOR>() as u32);

    // Set up the CDB
    unsafe {
        if UseLegacyNominalPerformance != 0 {
            cdb.GET_PERFORMANCE.as_mut().OperationCode = SCSIOP_GET_PERFORMANCE;
            // Except = 0, Write = 1, Reserved0 = 0 -> byte = 0b00000010
            cdb.GET_PERFORMANCE.as_mut().set_Write(1);
            cdb.GET_PERFORMANCE.as_mut().set_Tolerance(2);
            cdb.GET_PERFORMANCE.as_mut().MaximumNumberOfDescriptors[1] = maxDescriptors as u8;
            cdb.GET_PERFORMANCE.as_mut().Type = 0;
        } else {
            cdb.GET_PERFORMANCE.as_mut().OperationCode = SCSIOP_GET_PERFORMANCE;
            // Except/Write not set; they stay zero from RtlZeroMemory
            cdb.GET_PERFORMANCE.as_mut().MaximumNumberOfDescriptors[1] = maxDescriptors as u8;
            cdb.GET_PERFORMANCE.as_mut().Type = 3;
        }
    }

    let status = unsafe {
        ScratchBuffer_ExecuteCdbEx(
            DeviceExtension,
            core::ptr::null_mut(),
            transferSize,
            1, // TRUE
            &cdb as *const CDB,
            12,
            CDROM_GET_PERFORMANCE_TIMEOUT,
        )
    };

    // determine how many valid descriptors there actually are
    if status >= 0 {
        unsafe {
            let header = (*DeviceExtension).ScratchContext.ScratchBuffer as *const GET_PERFORMANCE_HEADER;
            let mut temp1: u32 =
                ((*header).TotalDataLength[0] as u32) << (3 * 8)
                    | ((*header).TotalDataLength[1] as u32) << (2 * 8)
                    | ((*header).TotalDataLength[2] as u32) << (1 * 8)
                    | ((*header).TotalDataLength[3] as u32) << (0 * 8);

            // RTL_SIZEOF_THROUGH_FIELD(GET_PERFORMANCE_HEADER, TotalDataLength)
            let sizeThroughField =
                core::mem::offset_of!(GET_PERFORMANCE_HEADER, TotalDataLength) as u32 + 4;

            if temp1.wrapping_add(sizeThroughField) < temp1 {
                temp1 = 0;
            } else if temp1 != 0 {
                temp1 = temp1.wrapping_add(sizeThroughField);
            }

            let adjusted = temp1;
            if adjusted == 0 {
                // no data
            } else if adjusted <= core::mem::size_of::<GET_PERFORMANCE_HEADER>() as u32 {
                // only the header
            } else if UseLegacyNominalPerformance != 0
                && (((*header)._bitfield_byte & (1 << 0)) != 0 /* Except */
                    || ((*header)._bitfield_byte & (1 << 1)) == 0 /* Write not set */)
            {
                // bad data
            } else if UseLegacyNominalPerformance == 0
                && (((*header)._bitfield_byte & (1 << 0)) != 0 /* Except */
                    || ((*header)._bitfield_byte & (1 << 1)) != 0 /* Write set */)
            {
                // returning wrong type
            } else if (adjusted - core::mem::size_of::<GET_PERFORMANCE_HEADER>() as u32)
                % core::mem::size_of::<GET_PERFORMANCE_WRITE_SPEED_DESCRIPTOR>() as u32
                != 0
            {
                // not returning valid data
            } else {
                // take the smaller usable value
                let srb = (*DeviceExtension).ScratchContext.ScratchSrb;
                let dataTransferLength = (*srb).DataTransferLength;
                let usable = if adjusted < dataTransferLength {
                    adjusted
                } else {
                    dataTransferLength
                };
                validDescriptors = (usable - core::mem::size_of::<GET_PERFORMANCE_HEADER>() as u32)
                    / core::mem::size_of::<GET_PERFORMANCE_WRITE_SPEED_DESCRIPTOR>() as u32;
            }
        }
    }

    // verify the data makes sense
    if status >= 0 {
        unsafe {
            let header =
                (*DeviceExtension).ScratchContext.ScratchBuffer as *const GET_PERFORMANCE_HEADER;
            let mut descriptor = header.add(1) as *const GET_PERFORMANCE_WRITE_SPEED_DESCRIPTOR;

            let mut i: u32 = 0;
            while i < validDescriptors {
                let writeKBps: u32 =
                    ((*descriptor).WriteSpeedKilobytesPerSecond[0] as u32) << (3 * 8)
                        | ((*descriptor).WriteSpeedKilobytesPerSecond[1] as u32) << (2 * 8)
                        | ((*descriptor).WriteSpeedKilobytesPerSecond[2] as u32) << (1 * 8)
                        | ((*descriptor).WriteSpeedKilobytesPerSecond[3] as u32) << (0 * 8);

                let sectorsPerSecond: u32 =
                    if writeKBps > 0x00418937 {
                        (writeKBps / 2048) * 1000
                    } else {
                        (writeKBps * 1000) / 2048
                    };

                if sectorsPerSecond <= 0 {
                    break;
                }

                if result == -1 {
                    result = MAXIMUM_RETRY_FOR_SINGLE_IO_IN_100NS_UNITS;
                }

                let converted =
                    ConvertSectorsPerSecondTo100nsUnitsFor64kWrite(sectorsPerSecond);
                if converted < result {
                    result = converted;
                }

                descriptor = descriptor.add(1);
                i += 1;
            }
        }
    }

    unsafe {
        ScratchBuffer_EndUse(DeviceExtension);
    }

    result
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
//     mmc.c
//
// Abstract:
//
//     Include all funtions relate to MMC
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
// #include "mmc.h"
// #include "scratch.h"
//
// #ifdef DEBUG_USE_WPP
// #include "mmc.tmh"
// #endif
//
// #ifdef ALLOC_PRAGMA
//
// #pragma alloc_text(PAGE, DeviceDeallocateMmcResources)
// #pragma alloc_text(PAGE, DeviceAllocateMmcResources)
// #pragma alloc_text(PAGE, DeviceUpdateMmcCapabilities)
// #pragma alloc_text(PAGE, DeviceGetConfigurationWithAlloc)
// #pragma alloc_text(PAGE, DeviceGetConfiguration)
// #pragma alloc_text(PAGE, DeviceUpdateMmcWriteCapability)
// #pragma alloc_text(PAGE, MmcDataFindFeaturePage)
// #pragma alloc_text(PAGE, MmcDataFindProfileInProfiles)
// #pragma alloc_text(PAGE, DeviceRetryTimeGuessBasedOnProfile)
// #pragma alloc_text(PAGE, DeviceRetryTimeDetectionBasedOnModePage2A)
// #pragma alloc_text(PAGE, DeviceRetryTimeDetectionBasedOnGetPerformance)
//
// #endif
//
// #pragma warning(push)
// #pragma warning(disable:4214) // nonstandard extension used : bit field types other than int
//
// _IRQL_requires_max_(APC_LEVEL)
// VOID
// DeviceDeallocateMmcResources(
//      WDFDEVICE Device
//     )
// /*++
//
// Routine Description:
//
//    release MMC resources
//
// Arguments:
//
//     Device - device object
//
// Return Value:
//
//     none
//
// --*/
// {
//     PCDROM_DEVICE_EXTENSION deviceExtension = DeviceGetExtension(Device);
//     PCDROM_DATA             cddata = &(deviceExtension->DeviceAdditionalData);
//     PCDROM_MMC_EXTENSION    mmcData = &cddata->Mmc;
//
//     PAGED_CODE();
//
//     if (mmcData->CapabilitiesIrp)
//     {
//         IoFreeIrp(mmcData->CapabilitiesIrp);
//         mmcData->CapabilitiesIrp = NULL;
//     }
//     if (mmcData->CapabilitiesMdl)
//     {
//         IoFreeMdl(mmcData->CapabilitiesMdl);
//         mmcData->CapabilitiesMdl = NULL;
//     }
//     if (mmcData->CapabilitiesBuffer)
//     {
//         ExFreePool(mmcData->CapabilitiesBuffer);
//         mmcData->CapabilitiesBuffer = NULL;
//     }
//     if (mmcData->CapabilitiesRequest)
//     {
//         WdfObjectDelete(mmcData->CapabilitiesRequest);
//         mmcData->CapabilitiesRequest = NULL;
//     }
//     mmcData->CapabilitiesBufferSize = 0;
//     mmcData->IsMmc = FALSE;
//     mmcData->WriteAllowed = FALSE;
//
//     return;
// }
//
//
// _IRQL_requires_max_(PASSIVE_LEVEL)
// NTSTATUS
// DeviceAllocateMmcResources(
//      WDFDEVICE Device
//     )
// /*++
//
// Routine Description:
//
//    allocate all MMC resources needed
//
// Arguments:
//
//     Device - device object
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS status                         = STATUS_SUCCESS;
//     PCDROM_DEVICE_EXTENSION deviceExtension = DeviceGetExtension(Device);
//     PCDROM_DATA cddata                      = &(deviceExtension->DeviceAdditionalData);
//     PCDROM_MMC_EXTENSION mmcData            = &(cddata->Mmc);
//     WDF_OBJECT_ATTRIBUTES attributes        = {0};
//
//     PAGED_CODE();
//
//     NT_ASSERT(mmcData->CapabilitiesBuffer == NULL);
//     NT_ASSERT(mmcData->CapabilitiesBufferSize == 0);
//
//     // allocate the buffer and set the buffer size.
//     // retrieve drive configuration information.
//     status = DeviceGetConfigurationWithAlloc(Device,
//                                              &mmcData->CapabilitiesBuffer,
//                                              &mmcData->CapabilitiesBufferSize,
//                                              FeatureProfileList,
//                                              SCSI_GET_CONFIGURATION_REQUEST_TYPE_ALL);
//     if (!NT_SUCCESS(status))
//     {
//         NT_ASSERT(mmcData->CapabilitiesBuffer     == NULL);
//         NT_ASSERT(mmcData->CapabilitiesBufferSize == 0);
//         return status;
//     }
//
//     NT_ASSERT(mmcData->CapabilitiesBuffer     != NULL);
//     NT_ASSERT(mmcData->CapabilitiesBufferSize != 0);
//
//     // Create an MDL over the new Buffer (allocated by DeviceGetConfiguration)
//     mmcData->CapabilitiesMdl = IoAllocateMdl(mmcData->CapabilitiesBuffer,
//                                              mmcData->CapabilitiesBufferSize,
//                                              FALSE, FALSE, NULL);
//     if (mmcData->CapabilitiesMdl == NULL)
//     {
//         ExFreePool(mmcData->CapabilitiesBuffer);
//         mmcData->CapabilitiesBuffer = NULL;
//         mmcData->CapabilitiesBufferSize = 0;
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     // Create an IRP from which we will create a WDFREQUEST
//     mmcData->CapabilitiesIrp = IoAllocateIrp(deviceExtension->DeviceObject->StackSize + 1, FALSE);
//     if (mmcData->CapabilitiesIrp == NULL)
//     {
//         IoFreeMdl(mmcData->CapabilitiesMdl);
//         mmcData->CapabilitiesMdl = NULL;
//         ExFreePool(mmcData->CapabilitiesBuffer);
//         mmcData->CapabilitiesBuffer = NULL;
//         mmcData->CapabilitiesBufferSize = 0;
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     // create WDF request object
//     WDF_OBJECT_ATTRIBUTES_INIT_CONTEXT_TYPE(&attributes,
//                                             CDROM_REQUEST_CONTEXT);
//     status = WdfRequestCreateFromIrp(&attributes,
//                                      mmcData->CapabilitiesIrp,
//                                      FALSE,
//                                      &mmcData->CapabilitiesRequest);
//     if (!NT_SUCCESS(status))
//     {
//         return status;
//     }
//
//     return STATUS_SUCCESS;
// }
//
// _IRQL_requires_max_(PASSIVE_LEVEL)
// NTSTATUS
// DeviceUpdateMmcCapabilities(
//      WDFDEVICE Device
//     )
// /*++
//
// Routine Description:
//
//    issue get congiguration command ans save result in device extension
//
// Arguments:
//
//     Device - device object
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS                 status = STATUS_SUCCESS;
//     PCDROM_DEVICE_EXTENSION  deviceExtension = DeviceGetExtension(Device);
//     PCDROM_DATA              cdData = &(deviceExtension->DeviceAdditionalData);
//     PCDROM_MMC_EXTENSION     mmcData = &(cdData->Mmc);
//     ULONG                    returnedBytes = 0;
//     LONG                     updateState;
//
//     PAGED_CODE();
//
//     // first of all, check if we're still in the CdromMmcUpdateRequired state
//     // and, if yes, change it to CdromMmcUpdateStarted.
//     updateState = InterlockedCompareExchange((PLONG)&(cdData->Mmc.UpdateState),
//                                              CdromMmcUpdateStarted,
//                                              CdromMmcUpdateRequired);
//     if (updateState != CdromMmcUpdateRequired) {
//         // Mmc capabilities have been already updated or are in the process of
//         // being updated - just return STATUS_SUCCESS
//         return STATUS_SUCCESS;
//     }
//
//     // default to read-only, no Streaming, non-blank
//     mmcData->WriteAllowed = FALSE;
//     mmcData->StreamingReadSupported = FALSE;
//     mmcData->StreamingWriteSupported = FALSE;
//
//     // Issue command to update the drive capabilities.
//     // The failure of MMC update is not considered critical,
//     // so that we'll continue to process I/O even MMC update fails.
//     status = DeviceGetConfiguration(Device,
//                                     mmcData->CapabilitiesBuffer,
//                                     mmcData->CapabilitiesBufferSize,
//                                     &returnedBytes,
//                                     FeatureProfileList,
//                                     SCSI_GET_CONFIGURATION_REQUEST_TYPE_CURRENT);
//
//     if (NT_SUCCESS(status) &&                               // succeeded.
//         (mmcData->CapabilitiesBufferSize >= returnedBytes)) // not overflow.
//     {
//         // update whether or not writes are allowed
//         // this should be the *ONLY* place writes are set to allowed
//         {
//             BOOLEAN         writeAllowed = FALSE;
//             FEATURE_NUMBER  validationSchema = 0;
//             ULONG           blockingFactor = 1;
//
//             DeviceUpdateMmcWriteCapability(mmcData->CapabilitiesBuffer,
//                                            returnedBytes,
//                                            TRUE,
//                                            &writeAllowed,
//                                            &validationSchema,
//                                            &blockingFactor);
//
//             mmcData->WriteAllowed = writeAllowed;
//             mmcData->ValidationSchema = validationSchema;
//             mmcData->Blocking = blockingFactor;
//         }
//
//         // Check if Streaming reads/writes are supported and cache
//         // this information for later use.
//         {
//             PFEATURE_HEADER header;
//             ULONG           minAdditionalLength;
//
//             minAdditionalLength = FIELD_OFFSET(FEATURE_DATA_REAL_TIME_STREAMING, Reserved2) -
//                                   sizeof(FEATURE_HEADER);
//
//             header = MmcDataFindFeaturePage(mmcData->CapabilitiesBuffer,
//                                             returnedBytes,
//                                             FeatureRealTimeStreaming);
//
//             if ((header != NULL) &&
//                 (header->Current) &&
//                 (header->AdditionalLength >= minAdditionalLength))
//             {
//                 PFEATURE_DATA_REAL_TIME_STREAMING feature = (PFEATURE_DATA_REAL_TIME_STREAMING)header;
//
//                 // If Real-Time feature is current, then Streaming reads are supported for sure.
//                 mmcData->StreamingReadSupported = TRUE;
//
//                 // Streaming writes are supported if an appropriate bit is set in the feature page.
//                 mmcData->StreamingWriteSupported = (feature->StreamRecording == 1);
//             }
//         }
//
//         // update the flag to reflect that if the media is CSS protected DVD or CPPM-protected DVDAudio
//         {
//             PFEATURE_HEADER header;
//
//             header = DeviceFindFeaturePage(mmcData->CapabilitiesBuffer,
//                                            returnedBytes,
//                                            FeatureDvdCSS);
//
//             mmcData->IsCssDvd = (header != NULL) && (header->Current);
//         }
//
//         // Update the guesstimate for the drive's write speed
//         // Use the GetConfig profile first as a quick-guess based
//         // on media "type", then continue with media-specific
//         // queries for older media types, and use GET_PERFORMANCE
//         // for all unknown/future media types.
//         {
//             // pseudo-code:
//             // 1) Determine default based on profile (slowest for media)
//             // 2) Determine default based on MODE PAGE 2Ah
//             // 3) Determine default based on GET PERFORMANCE data
//             // 4) Choose fastest reported speed (-1 == none reported)
//             // 5) If all failed (returned -1), go with very safe (slow) default
//             //
//             // This ensures that the retries do not overload the drive's processor.
//             // Sending at highest possible speed for the media is OK, because the
//             // major downside is drive processor usage.  (bus usage too, but most
//             // storage is becoming a point-to-point link.)
//
//             FEATURE_PROFILE_TYPE const profile =
//                                             mmcData->CapabilitiesBuffer->CurrentProfile[0] << (8*1) |
//                                             mmcData->CapabilitiesBuffer->CurrentProfile[1] << (8*0) ;
//             LONGLONG t1 = (LONGLONG)-1;
//             LONGLONG t2 = (LONGLONG)-1;
//             LONGLONG t3 = (LONGLONG)-1;
//             LONGLONG t4 = (LONGLONG)-1;
//             LONGLONG final;
//
//             t1 = DeviceRetryTimeGuessBasedOnProfile(profile);
//             t2 = DeviceRetryTimeDetectionBasedOnModePage2A(deviceExtension);
//             t3 = DeviceRetryTimeDetectionBasedOnGetPerformance(deviceExtension, TRUE);
//             t4 = DeviceRetryTimeDetectionBasedOnGetPerformance(deviceExtension, FALSE);
//
//             // use the "fastest" value returned
//             final = MAXLONGLONG;
//             if (t4 != -1)
//             {
//                 final = min(final, t4);
//             }
//             if (t3 != -1)
//             {
//                 final = min(final, t3);
//             }
//             if (t2 != -1)
//             {
//                 final = min(final, t2);
//             }
//             if (t1 != -1)
//             {
//                 final = min(final, t1);
//             }
//             if (final == MAXLONGLONG)
//             {
//                 // worst case -- use relatively slow default....
//                 final = WRITE_RETRY_DELAY_CD_4x;
//             }
//
//             cdData->ReadWriteRetryDelay100nsUnits = final;
//         }
//
//     }
//     else
//     {
//         // Rediscovery of MMC capabilities has failed - we'll need to retry
//         cdData->Mmc.UpdateState = CdromMmcUpdateRequired;
//     }
//
//     // Change the state to CdromMmcUpdateComplete if it is CdromMmcUpdateStarted.
//     // If it is not, some error must have happened while this function was executed
//     // and the state is CdromMmcUpdateRequired now. In that case, we want to perform
//     // everything again, so we do not set CdromMmcUpdateComplete.
//     InterlockedCompareExchange((PLONG)&(cdData->Mmc.UpdateState),
//                                CdromMmcUpdateComplete,
//                                CdromMmcUpdateStarted);
//
//     return status;
// }
//
// _IRQL_requires_max_(PASSIVE_LEVEL)
// NTSTATUS
// DeviceGetConfigurationWithAlloc(
//      WDFDEVICE               Device,
//     
//      PGET_CONFIGURATION_HEADER*  Buffer,  // this routine allocates this memory
//      PULONG                 BytesReturned,
//     FEATURE_NUMBER const         StartingFeature,
//     ULONG const                  RequestedType
//     )
// /*++
//
// Routine Description:
//
//     This function will allocates configuration buffer and set the size.
//
// Arguments:
//
//     Device - device object
//     Buffer - to be allocated by this function
//     BytesReturned - size of the buffer
//     StartingFeature - the starting point of the feature list
//     RequestedType -
//
// Return Value:
//
//     NTSTATUS
//
// NOTE: does not handle case where more than 65000 bytes are returned,
//       which requires multiple calls with different starting feature
//       numbers.
//
// --*/
// {
//     NTSTATUS                    status = STATUS_SUCCESS;
//     GET_CONFIGURATION_HEADER    header = {0};  // eight bytes, not a lot
//     PGET_CONFIGURATION_HEADER   buffer = NULL;
//     ULONG                       returned = 0;
//     ULONG                       size = 0;
//     ULONG                       i = 0;
//
//     PAGED_CODE();
//
//     *Buffer = NULL;
//     *BytesReturned = 0;
//
//     // send the first request down to just get the header
//     status = DeviceGetConfiguration(Device,
//                                     &header,
//                                     sizeof(header),
//                                     &returned,
//                                     StartingFeature,
//                                     RequestedType);
//
//     // now send command again, using information returned to allocate just enough memory
//     if (NT_SUCCESS(status))
//     {
//         size = header.DataLength[0] << 24 |
//                header.DataLength[1] << 16 |
//                header.DataLength[2] <<  8 |
//                header.DataLength[3] <<  0 ;
//
//         // the loop is in case that the retrieved data length is bigger than last time reported.
//         for (i = 0; (i < 4) && NT_SUCCESS(status); i++)
//         {
//             // the datalength field is the size *following* itself, so adjust accordingly
//             size += 4*sizeof(UCHAR);
//
//             // make sure the size is reasonable
//             if (size <= sizeof(FEATURE_HEADER))
//             {
//                 TracePrint!((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL,
//                            "DeviceGetConfigurationWithAlloc: drive reports only %x bytes?\n",
//                            size));
//                 status = STATUS_UNSUCCESSFUL;
//             }
//
//             if (NT_SUCCESS(status))
//             {
//                 // allocate the memory
//                 buffer = (PGET_CONFIGURATION_HEADER)ExAllocatePoolWithTag(NonPagedPoolNxCacheAligned,
//                                                                           size,
//                                                                           CDROM_TAG_FEATURE);
//
//                 if (buffer == NULL)
//                 {
//                     status = STATUS_INSUFFICIENT_RESOURCES;
//                 }
//             }
//
//             if (NT_SUCCESS(status))
//             {
//                 // send the first request down to just get the header
//                 status = DeviceGetConfiguration(Device,
//                                                 buffer,
//                                                 size,
//                                                 &returned,
//                                                 StartingFeature,
//                                                 RequestedType);
//
//                 if (!NT_SUCCESS(status))
//                 {
//                     ExFreePool(buffer);
//                 }
//                 else if (returned > size)
//                 {
//                     ExFreePool(buffer);
//                     status = STATUS_INTERNAL_ERROR;
//                 }
//             }
//
//             // command succeeded.
//             if (NT_SUCCESS(status))
//             {
//                 returned = buffer->DataLength[0] << 24 |
//                            buffer->DataLength[1] << 16 |
//                            buffer->DataLength[2] <<  8 |
//                            buffer->DataLength[3] <<  0 ;
//                 returned += 4*sizeof(UCHAR);
//
//                 if (returned <= size)
//                 {
//                     *Buffer = buffer;
//                     *BytesReturned = returned;  // amount of 'safe' memory
//                     // succes, get out of loop.
//                     status = STATUS_SUCCESS;
//                     break;
//                 }
//                 else
//                 {
//                     // the data size is bigger than the buffer size, retry using new size....
//                     size = returned;
//                     ExFreePool(buffer);
//                     buffer = NULL;
//                 }
//             }
//         } // end of for() loop
//     }
//
//     if (!NT_SUCCESS(status))
//     {
//         // it failed after a number of attempts, so just fail.
//         TracePrint!((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL,
//                    "DeviceGetConfigurationWithAlloc: Failed %d attempts to get all feature "
//                    "information\n", i));
//     }
//
//     return status;
// }
//
// _IRQL_requires_max_(PASSIVE_LEVEL)
// NTSTATUS
// DeviceGetConfiguration(
//       WDFDEVICE                 Device,
//     
//           PGET_CONFIGURATION_HEADER Buffer,
//       ULONG const               BufferSize,
//      PULONG                    ValidBytes,
//       FEATURE_NUMBER const      StartingFeature,
//       ULONG const               RequestedType
//     )
// /*++
//
// Routine Description:
//
//     This function is used to get configuration data.
//
// Arguments:
//
//     Device - device object
//     Buffer - buffer address to hold data.
//     BufferSize - size of the buffer
//     ValidBytes - valid data size in buffer
//     StartingFeature - the starting point of the feature list
//     RequestedType -
//
// Return Value:
//
//     NTSTATUS
//
// NOTE: does not handle case where more than 64k bytes are returned,
//       which requires multiple calls with different starting feature
//       numbers.
//
// --*/
// {
//     NTSTATUS                status;
//     PCDROM_DEVICE_EXTENSION deviceExtension = DeviceGetExtension(Device);
//     SCSI_REQUEST_BLOCK      srb;
//     PCDB                    cdb = (PCDB)srb.Cdb;
//
//     PAGED_CODE();
//
//     NT_ASSERT(ValidBytes);
//
//     // when system is low resources we can receive empty buffer
//     if (Buffer == NULL || BufferSize < sizeof(GET_CONFIGURATION_HEADER))
//     {
//         return STATUS_BUFFER_TOO_SMALL;
//     }
//
//     *ValidBytes = 0;
//
//     RtlZeroMemory(&srb, sizeof(SCSI_REQUEST_BLOCK));
//     RtlZeroMemory(Buffer, BufferSize);
//
//     if (TEST_FLAG(deviceExtension->DeviceAdditionalData.HackFlags, CDROM_HACK_BAD_GET_CONFIG_SUPPORT))
//     {
//         return STATUS_INVALID_DEVICE_REQUEST;
//     }
//
// #pragma warning(push)
// #pragma warning(disable: 6386) // OACR will complain buffer overrun: the writable size is 'BufferSize' bytes, but '65532'
//                                // bytes might be written, which is impossible because BufferSize > 0xFFFC.
//
//     if (BufferSize > 0xFFFC)
//     {
//         // cannot request more than 0xFFFC bytes in one request
//         // Eventually will "stitch" together multiple requests if needed
//         // Today, no drive has anywhere close to 4k.....
//         return DeviceGetConfiguration(Device,
//                                       Buffer,
//                                       0xFFFC,
//                                       ValidBytes,
//                                       StartingFeature,
//                                       RequestedType);
//     }
// #pragma warning(pop)
//
//     //Start real work
//     srb.TimeOutValue = CDROM_GET_CONFIGURATION_TIMEOUT;
//     srb.CdbLength = 10;
//
//     cdb->GET_CONFIGURATION.OperationCode = SCSIOP_GET_CONFIGURATION;
//     cdb->GET_CONFIGURATION.RequestType = (UCHAR)RequestedType;
//     cdb->GET_CONFIGURATION.StartingFeature[0] = (UCHAR)(StartingFeature >> 8);
//     cdb->GET_CONFIGURATION.StartingFeature[1] = (UCHAR)(StartingFeature & 0xff);
//     cdb->GET_CONFIGURATION.AllocationLength[0] = (UCHAR)(BufferSize >> 8);
//     cdb->GET_CONFIGURATION.AllocationLength[1] = (UCHAR)(BufferSize & 0xff);
//
//     status = DeviceSendSrbSynchronously(Device,
//                                         &srb,
//                                         Buffer,
//                                         BufferSize,
//                                         FALSE,
//                                         NULL);
//
//     TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL,
//                "DeviceGetConfiguration: Status was %x\n", status));
//
//     if (NT_SUCCESS(status) ||
//         (status == STATUS_BUFFER_OVERFLOW) ||
//         (status == STATUS_DATA_OVERRUN))
//     {
//         ULONG                       returned = srb.DataTransferLength;
//         PGET_CONFIGURATION_HEADER   header = (PGET_CONFIGURATION_HEADER)Buffer;
//         ULONG                       available = (header->DataLength[0] << (8*3)) |
//                                                 (header->DataLength[1] << (8*2)) |
//                                                 (header->DataLength[2] << (8*1)) |
//                                                 (header->DataLength[3] << (8*0)) ;
//
//         available += RTL_SIZEOF_THROUGH_FIELD(GET_CONFIGURATION_HEADER, DataLength);
//
//         _Analysis_assume_(srb.DataTransferLength <= BufferSize);
//
//         // The true usable amount of data returned is the lesser of
//         // * the returned data per the srb.DataTransferLength field
//         // * the total size per the GET_CONFIGURATION_HEADER
//         // This is because ATAPI can't tell how many bytes really
//         // were transferred on success when using DMA.
//         if (available < returned)
//         {
//             returned = available;
//         }
//
//         NT_ASSERT(returned <= BufferSize);
//         *ValidBytes = (ULONG)returned;
//
//         //This is succeed case
//         status = STATUS_SUCCESS;
//     }
//     else
//     {
//         TracePrint!((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL,
//                    "DeviceGetConfiguration: failed %x\n", status));
//     }
//
//     return status;
// }
//
//
// _IRQL_requires_max_(APC_LEVEL)
// VOID
// DeviceUpdateMmcWriteCapability(
//     
//         PGET_CONFIGURATION_HEADER   Buffer,
//     ULONG const                     BufferSize,
//     BOOLEAN const                   CurrentOnly, // TRUE == can drive write now, FALSE == can drive ever write
//      PBOOLEAN                  Writable,
//      PFEATURE_NUMBER           ValidationSchema,
//      PULONG                    BlockingFactor
//     )
// /*++
//
// Routine Description:
//
//     This function will allocates configuration buffer and set the size.
//
// Arguments:
//
//     Buffer -
//     BufferSize - size of the buffer
//     CurrentOnly - valid data size in buffer
//     Writable - the buffer is allocationed in non-paged pool.
//     validationSchema - the starting point of the feature list
//     BlockingFactor -
//
// Return Value:
//
//     NTSTATUS
//
// NOTE: does not handle case where more than 64k bytes are returned,
//       which requires multiple calls with different starting feature
//       numbers.
//
// --*/
// {
//     //
//     // this routine is used to check if the drive can currently (current==TRUE)
//     // or can ever (current==FALSE) write to media with the current CDROM.SYS
//     // driver.  this check parses the GET_CONFIGURATION response data to search
//     // for the appropriate features and/or if they are current.
//     //
//     // this function should not allocate any resources, and thus may safely
//     // return from any point within the function.
//     //
//     PAGED_CODE();
//
//     *Writable = FALSE;
//     *ValidationSchema = 0;
//     *BlockingFactor = 1;
//
//     //
//     // if the drive supports hardware defect management and random writes, that's
//     // sufficient to allow writes.
//     //
//     {
//         PFEATURE_HEADER defectHeader;
//         PFEATURE_HEADER writableHeader;
//
//         defectHeader   = MmcDataFindFeaturePage(Buffer,
//                                                 BufferSize,
//                                                 FeatureDefectManagement);
//         writableHeader = MmcDataFindFeaturePage(Buffer,
//                                                 BufferSize,
//                                                 FeatureRandomWritable);
//
//         if (defectHeader == NULL || writableHeader == NULL)
//         {
//             // cannot write this way
//         }
//         else if (!CurrentOnly)
//         {
//             TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_INIT,
//                         "DeviceUpdateMmcWriteCapability => Writes supported (defect management)\n"));
//             *Writable = TRUE;
//             return;
//         }
//         else if (defectHeader->Current && writableHeader->Current)
//         {
//             TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_INIT,
//                         "DeviceUpdateMmcWriteCapability => Writes *allowed* (defect management)\n"));
//             *Writable = TRUE;
//             *ValidationSchema = FeatureDefectManagement;
//             return;
//         }
//     }
//
//     // Certain validation schema require the blocking factor
//     // This is a best-effort attempt to ensure that illegal
//     // requests do not make it to drive
//     {
//         PFEATURE_HEADER header;
//         ULONG           additionalLength;
//
//         // Certain validation schema require the blocking factor
//         // This is a best-effort attempt to ensure  that illegal
//         // requests do not make it to drive
//         additionalLength = RTL_SIZEOF_THROUGH_FIELD(FEATURE_DATA_RANDOM_READABLE, Blocking) - sizeof(FEATURE_HEADER);
//
//         header = MmcDataFindFeaturePage(Buffer,
//                                         BufferSize,
//                                         FeatureRandomReadable);
//
//         if ((header != NULL) &&
//             (header->Current) &&
//             (header->AdditionalLength >= additionalLength))
//         {
//             PFEATURE_DATA_RANDOM_READABLE feature = (PFEATURE_DATA_RANDOM_READABLE)header;
//             *BlockingFactor = (feature->Blocking[0] << 8) | feature->Blocking[1];
//         }
//     }
//
//     // the majority of features to indicate write capability
//     // indicate this by a single feature existance/current bit.
//     // thus, can use a table-based method for the majority
//     // of the detection....
//     {
//         typedef struct {
//             FEATURE_NUMBER  FeatureToFind;    // the ones allowed
//             FEATURE_NUMBER  ValidationSchema; // and their related schema
//         } FEATURE_TO_WRITE_SCHEMA_MAP;
//
//         static FEATURE_TO_WRITE_SCHEMA_MAP const FeaturesToAllowWritesWith[] = {
//             { FeatureRandomWritable,               FeatureRandomWritable               },
//             { FeatureRigidRestrictedOverwrite,     FeatureRigidRestrictedOverwrite     },
//             { FeatureRestrictedOverwrite,          FeatureRestrictedOverwrite          },
//             { FeatureIncrementalStreamingWritable, FeatureIncrementalStreamingWritable },
//         };
//
//         ULONG count;
//         for (count = 0; count < RTL_NUMBER_OF(FeaturesToAllowWritesWith); count++)
//         {
//             PFEATURE_HEADER header = MmcDataFindFeaturePage(Buffer,
//                                                             BufferSize,
//                                                             FeaturesToAllowWritesWith[count].FeatureToFind);
//             if (header == NULL)
//             {
//                 // cannot write using this method
//             }
//             else if (!CurrentOnly)
//             {
//                 TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_INIT,
//                             "DeviceUpdateMmcWriteCapability => Writes supported (feature %04x)\n",
//                             FeaturesToAllowWritesWith[count].FeatureToFind
//                             ));
//                 *Writable = TRUE;
//                 return;
//             }
//             else if (header->Current)
//             {
//                 TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_INIT,
//                             "DeviceUpdateMmcWriteCapability => Writes *allowed* (feature %04x)\n",
//                             FeaturesToAllowWritesWith[count].FeatureToFind
//                             ));
//                 *Writable = TRUE;
//                 *ValidationSchema = FeaturesToAllowWritesWith[count].ValidationSchema;
//                 return;
//             }
//         } // end count loop
//     }
//
//     // unfortunately, DVD+R media doesn't require IncrementalStreamingWritable feature
//     // to be explicitly set AND it has a seperate bit in the feature to indicate
//     // being able to write to this media type. Thus, use a special case of the above code.
//     {
//         PFEATURE_DATA_DVD_PLUS_R header;
//         ULONG                    additionalLength = FIELD_OFFSET(FEATURE_DATA_DVD_PLUS_R, Reserved2[0]) - sizeof(FEATURE_HEADER);
//         header = MmcDataFindFeaturePage(Buffer,
//                                         BufferSize,
//                                         FeatureDvdPlusR);
//
//         if (header == NULL || (header->Header.AdditionalLength < additionalLength) || (!header->Write))
//         {
//             // cannot write this way
//         }
//         else if (!CurrentOnly)
//         {
//             TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_INIT,
//                         "DeviceUpdateMmcWriteCapability => Writes supported (feature %04x)\n",
//                         FeatureDvdPlusR
//                         ));
//             *Writable = TRUE;
//             return;
//         }
//         else if (header->Header.Current)
//         {
//             TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_INIT,
//                         "DeviceUpdateMmcWriteCapability => Writes *allowed* (feature %04x)\n",
//                         FeatureDvdPlusR
//                         ));
//             *Writable = TRUE;
//             *ValidationSchema = FeatureIncrementalStreamingWritable;
//             return;
//         }
//     }
//
//     // unfortunately, DVD+R DL media doesn't require IncrementalStreamingWritable feature
//     // to be explicitly set AND it has a seperate bit in the feature to indicate
//     // being able to write to this media type. Thus, use a special case of the above code.
//     {
//         PFEATURE_DATA_DVD_PLUS_R_DUAL_LAYER header;
//         ULONG additionalLength = FIELD_OFFSET(FEATURE_DATA_DVD_PLUS_R_DUAL_LAYER, Reserved2[0]) - sizeof(FEATURE_HEADER);
//         header = MmcDataFindFeaturePage(Buffer,
//                                         BufferSize,
//                                         FeatureDvdPlusRDualLayer);
//
//         if (header == NULL || (header->Header.AdditionalLength < additionalLength) || (!header->Write))
//         {
//             // cannot write this way
//         }
//         else if (!CurrentOnly)
//         {
//             TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_INIT,
//                         "DeviceUpdateMmcWriteCapability => Writes supported (feature %04x)\n",
//                         FeatureDvdPlusRDualLayer
//                         ));
//             *Writable = TRUE;
//             return;
//         }
//         else if (header->Header.Current)
//         {
//             TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_INIT,
//                         "DeviceUpdateMmcWriteCapability => Writes *allowed* (feature %04x)\n",
//                         FeatureDvdPlusRDualLayer
//                         ));
//             *Writable = TRUE;
//             *ValidationSchema = FeatureIncrementalStreamingWritable;
//             return;
//         }
//     }
//
//     // There are currently a number of drives on the market
//     // that fail to report:
//     // (a) FeatureIncrementalStreamingWritable as current
//     //     for CD-R / DVD-R profile.
//     // (b) FeatureRestrictedOverwrite as current for CD-RW
//     //     profile
//     // (c) FeatureRigidRestrictedOverwrite as current for
//     //     DVD-RW profile
//     //
//     // Thus, use the profiles also.
//     {
//         PFEATURE_HEADER header;
//         header = MmcDataFindFeaturePage(Buffer,
//                                         BufferSize,
//                                         FeatureProfileList);
//
//         if (header != NULL && header->Current)
//         {
//             // verify buffer bounds -- the below routine presumes full profile list provided
//             PUCHAR bufferEnd = ((PUCHAR)Buffer) + BufferSize;
//             PUCHAR headerEnd = ((PUCHAR)header) + header->AdditionalLength + RTL_SIZEOF_THROUGH_FIELD(FEATURE_HEADER, AdditionalLength);
//             if (bufferEnd >= headerEnd) // this _should_ never occurr, but....
//             {
//                 // Profiles don't contain any data other than current/not current.
//                 // thus, can generically loop through them to see if any of the
//                 // below (in order of preference) are current.
//                 typedef struct {
//                     FEATURE_PROFILE_TYPE ProfileToFind;    // the ones allowed
//                     FEATURE_NUMBER       ValidationSchema; // and their related schema
//                 } PROFILE_TO_WRITE_SCHEMA_MAP;
//
//                 static PROFILE_TO_WRITE_SCHEMA_MAP const ProfilesToAllowWritesWith[] = {
//                     { ProfileDvdRewritable,  FeatureRigidRestrictedOverwrite     },
//                     { ProfileCdRewritable,   FeatureRestrictedOverwrite },
//                     { ProfileDvdRecordable,  FeatureIncrementalStreamingWritable },
//                     { ProfileCdRecordable,   FeatureIncrementalStreamingWritable },
//                 };
//
//                 ULONG count;
//                 for (count = 0; count < RTL_NUMBER_OF(ProfilesToAllowWritesWith); count++)
//                 {
//                     BOOLEAN exists = FALSE;
//                     MmcDataFindProfileInProfiles((PFEATURE_DATA_PROFILE_LIST)header,
//                                                  ProfilesToAllowWritesWith[count].ProfileToFind,
//                                                  CurrentOnly,
//                                                  &exists);
//                     if (exists)
//                     {
//                         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_INIT,
//                                     "DeviceUpdateMmcWriteCapability => Writes %s (profile %04x)\n",
//                                     (CurrentOnly ? "*allowed*" : "supported"),
//                                     FeatureDvdPlusR
//                                     ));
//
//                         *Writable = TRUE;
//                         *ValidationSchema = ProfilesToAllowWritesWith[count].ValidationSchema;
//                         return;
//                     }
//                 } // end count loop
//             } // end if (bufferEnd >= headerEnd)
//
//         } // end if (header != NULL && header->Current)
//     }
//
//     // nothing matched to say it's writable.....
//     return;
// }
//
// _IRQL_requires_max_(APC_LEVEL)
// PVOID
// MmcDataFindFeaturePage(
//     
//         PGET_CONFIGURATION_HEADER   FeatureBuffer,
//     ULONG const                     Length,
//     FEATURE_NUMBER const            Feature
//     )
// /*++
//
// Routine Description:
//
//     search the specific feature from feature list buffer
//
// Arguments:
//
//     FeatureBuffer - buffer of feature list
//     Length - size of the buffer
//     Feature - feature wanted to find
//
// Return Value:
//
//     PVOID - if found, pointer of starting address of the specific feature.
//             otherwise, NULL.
//
// --*/
// {
//     PUCHAR buffer;
//     PUCHAR limit;
//     ULONG  validLength;
//
//     PAGED_CODE();
//
//     if (Length < sizeof(GET_CONFIGURATION_HEADER) + sizeof(FEATURE_HEADER)) {
//         return NULL;
//     }
//
//     // Calculate the length of valid data available in the
//     // capabilities buffer from the DataLength field
//     REVERSE_BYTES(&validLength, FeatureBuffer->DataLength);
//     validLength += RTL_SIZEOF_THROUGH_FIELD(GET_CONFIGURATION_HEADER, DataLength);
//
//     // set limit to point to first illegal address
//     limit  = (PUCHAR)FeatureBuffer;
//     limit += min(Length, validLength);
//
//     // set buffer to point to first page
//     buffer = FeatureBuffer->Data;
//
//     // loop through each page until we find the requested one, or
//     // until it's not safe to access the entire feature header
//     // (if equal, have exactly enough for the feature header)
//     while (buffer + sizeof(FEATURE_HEADER) <= limit)
//     {
//         PFEATURE_HEADER header = (PFEATURE_HEADER)buffer;
//         FEATURE_NUMBER  thisFeature;
//
//         thisFeature  = (header->FeatureCode[0] << 8) |
//                        (header->FeatureCode[1]);
//
//         if (thisFeature == Feature)
//         {
//             PUCHAR temp;
//
//             // if don't have enough memory to safely access all the feature
//             // information, return NULL
//             temp = buffer;
//             temp += sizeof(FEATURE_HEADER);
//             temp += header->AdditionalLength;
//
//             if (temp > limit)
//             {
//                 // this means the transfer was cut-off, an insufficiently
//                 // small buffer was given, or other arbitrary error.  since
//                 // it's not safe to view the amount of data (even though
//                 // the header is safe) in this feature, pretend it wasn't
//                 // transferred at all...
//                 TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_GENERAL,
//                            "Feature %x exists, but not safe to access all its "
//                            "data.  returning NULL\n", Feature));
//                 return NULL;
//             }
//             else
//             {
//                 return buffer;
//             }
//         }
//
//         if ((header->AdditionalLength % 4) &&
//             !(Feature >= 0xff00 && Feature <= 0xffff))
//         {
//             return NULL;
//         }
//
//         buffer += sizeof(FEATURE_HEADER);
//         buffer += header->AdditionalLength;
//
//     }
//     return NULL;
// }
//
// _IRQL_requires_max_(APC_LEVEL)
// VOID
// MmcDataFindProfileInProfiles(
//      FEATURE_DATA_PROFILE_LIST const* ProfileHeader,
//      FEATURE_PROFILE_TYPE const       ProfileToFind,
//      BOOLEAN const                    CurrentOnly,
//      PBOOLEAN                        Found
//     )
// /*++
//
// Routine Description:
//
//     search the specific feature from feature list buffer
//
// Arguments:
//
//     ProfileHeader - buffer of profile list
//     ProfileToFind - profile to be found
//     CurrentOnly -
//
// Return Value:
//
//     Found - found or not
//
// --*/
// {
//     FEATURE_DATA_PROFILE_LIST_EX const * profile;
//     ULONG numberOfProfiles;
//     ULONG i;
//
//     PAGED_CODE();
//
//     // initialize output
//     *Found = FALSE;
//
//     // sanity check
//     if (ProfileHeader->Header.AdditionalLength % 2 != 0)
//     {
//         TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_GENERAL,
//                    "Profile total length %x is not integral multiple of 4\n",
//                    ProfileHeader->Header.AdditionalLength));
//         NT_ASSERT(FALSE);
//         return;
//     }
//
//     // calculate number of profiles
//     numberOfProfiles = ProfileHeader->Header.AdditionalLength / 4;
//     profile = ProfileHeader->Profiles; // zero-sized array
//
//     // loop through profiles
//     for (i = 0; i < numberOfProfiles; i++)
//     {
//         FEATURE_PROFILE_TYPE currentProfile;
//
//         currentProfile = (profile->ProfileNumber[0] << 8) |
//                          (profile->ProfileNumber[1] & 0xff);
//
//         if (currentProfile == ProfileToFind)
//         {
//             if (profile->Current || (!CurrentOnly))
//             {
//                 *Found = TRUE;
//             }
//         }
//
//         profile++;
//     }
//
//     return;
// }
//
// _IRQL_requires_max_(APC_LEVEL)
//
// LONGLONG
// DeviceRetryTimeGuessBasedOnProfile(
//     FEATURE_PROFILE_TYPE const Profile
//     )
// /*++
//
// Routine Description:
//
//     determine the retry time based on profile
//
// Arguments:
//
//     Profile -
//
// Return Value:
//
//     LONGLONG - retry time
//
// --*/
// {
//     LONGLONG result = -1; // this means we have no idea
//
//     PAGED_CODE();
//
//     switch (Profile)
//     {
//     case ProfileInvalid:               // = 0x0000,
//     case ProfileNonRemovableDisk:      // = 0x0001,
//     case ProfileRemovableDisk:         // = 0x0002,
//     case ProfileMOErasable:            // = 0x0003,
//     case ProfileMOWriteOnce:           // = 0x0004,
//     case ProfileAS_MO:                 // = 0x0005,
//     // Reserved                             0x0006 - 0x0007,
//     // Reserved                             0x000b - 0x000f,
//     // Reserved                             0x0017 - 0x0019
//     // Reserved                             0x001C - 001F
//     // Reserved                             0x0023 - 0x0029
//     // Reserved                             0x002C - 0x003F
//     // Reserved                             0x0044 - 0x004F
//     // Reserved                             0x0053 - 0xfffe
//     case ProfileNonStandard:          //  = 0xffff
//     default:
//     {
//         NOTHING; // no default
//         break;
//     }
//
//     case ProfileCdrom:                 // = 0x0008,
//     case ProfileCdRecordable:          // = 0x0009,
//     case ProfileCdRewritable:          // = 0x000a,
//     case ProfileDDCdrom:               // = 0x0020, // obsolete
//     case ProfileDDCdRecordable:        // = 0x0021, // obsolete
//     case ProfileDDCdRewritable:        // = 0x0022, // obsolete
//     {
//         // 4x is ok as all CD drives have
//         // at least 64k*4 (256k) buffer
//         // and this is just a first-pass
//         // guess based only on profile
//         result = WRITE_RETRY_DELAY_CD_4x;
//         break;
//     }
//     case ProfileDvdRom:                // = 0x0010,
//     case ProfileDvdRecordable:         // = 0x0011,
//     case ProfileDvdRam:                // = 0x0012,
//     case ProfileDvdRewritable:         // = 0x0013,  // restricted overwrite
//     case ProfileDvdRWSequential:       // = 0x0014,
//     case ProfileDvdDashRLayerJump:     // = 0x0016,
//     case ProfileDvdPlusRW:             // = 0x001A,
//     case ProfileDvdPlusR:              // = 0x001B,
//     {
//         result = WRITE_RETRY_DELAY_DVD_1x;
//         break;
//     }
//     case ProfileDvdDashRDualLayer:     // = 0x0015,
//     case ProfileDvdPlusRWDualLayer:    // = 0x002A,
//     case ProfileDvdPlusRDualLayer:     // = 0x002B,
//     {
//         result = WRITE_RETRY_DELAY_DVD_1x;
//         break;
//     }
//
//     case ProfileBDRom:                 // = 0x0040,
//     case ProfileBDRSequentialWritable: // = 0x0041,  // BD-R 'SRM'
//     case ProfileBDRRandomWritable:     // = 0x0042,  // BD-R 'RRM'
//     case ProfileBDRewritable:          // = 0x0043,
//     {
//         // I could not find specifications for the
//         // minimal 1x data rate for BD media.  Use
//         // HDDVD values for now, since they are
//         // likely to be similar.  Also, all media
//         // except for CD, DVD, and AS-MO should
//         // already fully support GET_CONFIG, so
//         // this guess is only used if we fail to
//         // get a performance descriptor....
//         result = WRITE_RETRY_DELAY_HDDVD_1x;
//         break;
//     }
//
//     case ProfileHDDVDRom:              // = 0x0050,
//     case ProfileHDDVDRecordable:       // = 0x0051,
//     case ProfileHDDVDRam:              // = 0x0052,
//     {
//         // All HDDVD drives support GET_PERFORMANCE
//         // so this guess is fine at 1x....
//         result = WRITE_RETRY_DELAY_HDDVD_1x;
//         break;
//     }
//
//     // addition of any further profile types is not
//     // technically required as GET PERFORMANCE
//     // should succeed for all future drives.  However,
//     // it is useful in case GET PERFORMANCE does
//     // fail for other reasons (i.e. bus resets, etc)
//
//     } // end switch(Profile)
//
//     return result;
// }
//
// _IRQL_requires_max_(APC_LEVEL)
//
// LONGLONG
// DeviceRetryTimeDetectionBasedOnModePage2A(
//      PCDROM_DEVICE_EXTENSION DeviceExtension
//     )
// /*++
//
// Routine Description:
//
//     determine the retry time based on mode sense data
//
// Arguments:
//
//     DeviceExtension - device context
//
// Return Value:
//
//     LONGLONG - retry time
//
// --*/
// {
//     NTSTATUS    status;
//     ULONG       transferSize = min(0xFFF0, DeviceExtension->ScratchContext.ScratchBufferSize);
//     CDB         cdb;
//     LONGLONG    result = -1;
//
//     PAGED_CODE();
//
//     ScratchBuffer_BeginUse(DeviceExtension);
//
//     RtlZeroMemory(&cdb, sizeof(CDB));
//     // Set up the CDB
//     cdb.MODE_SENSE10.OperationCode = SCSIOP_MODE_SENSE10;
//     cdb.MODE_SENSE10.Dbd           = 1;
//     cdb.MODE_SENSE10.PageCode      = MODE_PAGE_CAPABILITIES;
//     cdb.MODE_SENSE10.AllocationLength[0] = (UCHAR)(transferSize >> 8);
//     cdb.MODE_SENSE10.AllocationLength[1] = (UCHAR)(transferSize & 0xFF);
//
//     status = ScratchBuffer_ExecuteCdb(DeviceExtension, NULL, transferSize, TRUE, &cdb, 10);
//
//     // analyze the data on success....
//     if (NT_SUCCESS(status))
//     {
//         MODE_PARAMETER_HEADER10 const* header = DeviceExtension->ScratchContext.ScratchBuffer;
//         CDVD_CAPABILITIES_PAGE const*  page = NULL;
//         ULONG dataLength = (header->ModeDataLength[0] << (8*1)) |
//                            (header->ModeDataLength[1] << (8*0)) ;
//
//         // no possible overflow
//         if (dataLength != 0)
//         {
//             dataLength += RTL_SIZEOF_THROUGH_FIELD(MODE_PARAMETER_HEADER10, ModeDataLength);
//         }
//
//         // If it's not abundantly clear, we really don't trust the drive
//         // to be returning valid data.  Get the page pointer and usable
//         // size of the page here...
//         if (dataLength < sizeof(MODE_PARAMETER_HEADER10))
//         {
//             dataLength = 0;
//         }
//         else if (dataLength > DeviceExtension->ScratchContext.ScratchBufferSize)
//         {
//             dataLength = 0;
//         }
//         else if ((header->BlockDescriptorLength[1] == 0) &&
//                  (header->BlockDescriptorLength[0] == 0))
//         {
//             dataLength -= sizeof(MODE_PARAMETER_HEADER10);
//             page = (CDVD_CAPABILITIES_PAGE const *)(header + 1);
//         }
//         else if ((header->BlockDescriptorLength[1] == 0) &&
//                  (header->BlockDescriptorLength[0] == sizeof(MODE_PARAMETER_BLOCK)))
//         {
//             dataLength -= sizeof(MODE_PARAMETER_HEADER10);
//             dataLength -= min(dataLength, sizeof(MODE_PARAMETER_BLOCK));
//             page = (CDVD_CAPABILITIES_PAGE const *)
//                                             ( ((PUCHAR)header) +
//                                               sizeof(MODE_PARAMETER_HEADER10) +
//                                               sizeof(MODE_PARAMETER_BLOCK)
//                                               );
//         }
//
//         // Change dataLength from the size available per the header to
//         // the size available per the page itself.
//         if ((page != NULL) &&
//             (dataLength >= RTL_SIZEOF_THROUGH_FIELD(CDVD_CAPABILITIES_PAGE, PageLength))
//             )
//         {
//             dataLength = min(dataLength, ((ULONG)(page->PageLength) + 2));
//         }
//
//         // Ignore the page if the fastest write speed field isn't available.
//         if ((page != NULL) &&
//             (dataLength < RTL_SIZEOF_THROUGH_FIELD(CDVD_CAPABILITIES_PAGE, WriteSpeedMaximum))
//             )
//         {
//             TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_INIT,
//                         "ModePage 2Ah was requested, but drive reported "
//                         "only %x bytes (%x needed). Ignoring.\n",
//                         dataLength,
//                         RTL_SIZEOF_THROUGH_FIELD(CDVD_CAPABILITIES_PAGE, WriteSpeedMaximum)
//                         ));
//             page = NULL;
//         }
//
//         // Verify the page we requested is the one the drive actually provided
//         if ((page != NULL) && (page->PageCode != MODE_PAGE_CAPABILITIES))
//         {
//             TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_INIT,
//                         "ModePage 2Ah was requested, but drive reported "
//                         "page %x\n",
//                         page->PageCode
//                         ));
//             page = NULL;
//         }
//
//         // If _everything_ succeeded, then use the speed value in the page!
//         if (page != NULL)
//         {
//             ULONG temp =
//                 (page->WriteSpeedMaximum[0] << (8*1)) |
//                 (page->WriteSpeedMaximum[1] << (8*0)) ;
//             // stored as 1,000 byte increments...
//             temp *= 1000;
//             // typically stored at 2448 bytes/sector due to CD media
//             // error up to 20% high by presuming it returned 2048 data
//             // and convert to sectors/second
//             temp /= 2048;
//             // currently: sectors/sec
//             // ignore too-small or zero values
//             if (temp != 0)
//             {
//                 result = ConvertSectorsPerSecondTo100nsUnitsFor64kWrite(temp);
//             }
//         }
//     }
//
//     ScratchBuffer_EndUse(DeviceExtension);
//
//     return result;
// }
//
//
// _IRQL_requires_max_(APC_LEVEL)
//
// LONGLONG
// DeviceRetryTimeDetectionBasedOnGetPerformance(
//      PCDROM_DEVICE_EXTENSION DeviceExtension,
//      BOOLEAN                 UseLegacyNominalPerformance
//     )
// /*++
//
// Routine Description:
//
//     determine the retry time based on get performance data
//
// Arguments:
//
//     DeviceExtension - device context
//     UseLegacyNominalPerformance -
//
// Return Value:
//
//     LONGLONG - retry time
//
// --*/
// {
//     typedef struct _GET_PERFORMANCE_HEADER {
//         UCHAR TotalDataLength[4]; // not including this field
//         UCHAR Except : 1;
//         UCHAR Write  : 1;
//         UCHAR Reserved0 : 6;
//         UCHAR Reserved1[3];
//     } GET_PERFORMANCE_HEADER, *PGET_PERFORMANCE_HEADER;
//     C_ASSERT( sizeof(GET_PERFORMANCE_HEADER) == 8);
//
//     typedef struct _GET_PERFORMANCE_NOMINAL_PERFORMANCE_DESCRIPTOR {
//         UCHAR StartLba[4];
//         UCHAR StartPerformance[4];
//         UCHAR EndLba[4];
//         UCHAR EndPerformance[4];
//     } GET_PERFORMANCE_NOMINAL_PERFORMANCE_DESCRIPTOR, *PGET_PERFORMANCE_NOMINAL_PERFORMANCE_DESCRIPTOR;
//     C_ASSERT( sizeof(GET_PERFORMANCE_NOMINAL_PERFORMANCE_DESCRIPTOR) == 16);
//
//
//     typedef struct _GET_PERFORMANCE_WRITE_SPEED_DESCRIPTOR {
//         UCHAR MixedReadWrite          : 1;
//         UCHAR GuaranteedForWholeMedia : 1;
//         UCHAR Reserved0_RDD           : 1;
//         UCHAR WriteRotationControl    : 2;
//         UCHAR Reserved1               : 3;
//         UCHAR Reserved2[3];
//
//         UCHAR MediaCapacity[4];
//         UCHAR ReadSpeedKilobytesPerSecond[4];
//         UCHAR WriteSpeedKilobytesPerSecond[4];
//     } GET_PERFORMANCE_WRITE_SPEED_DESCRIPTOR, *PGET_PERFORMANCE_WRITE_SPEED_DESCRIPTOR;
//     C_ASSERT( sizeof(GET_PERFORMANCE_WRITE_SPEED_DESCRIPTOR) == 16);
//
//     //////
//
//     NTSTATUS    status;
//     LONGLONG    result = -1;
//
//     // transfer size -- descriptors + 8 byte header
//     // Note: this size is identical for both descriptor types
//     C_ASSERT( sizeof(GET_PERFORMANCE_WRITE_SPEED_DESCRIPTOR) == sizeof(GET_PERFORMANCE_NOMINAL_PERFORMANCE_DESCRIPTOR));
//
//     ULONG const maxDescriptors = min(200, (DeviceExtension->ScratchContext.ScratchBufferSize-sizeof(GET_PERFORMANCE_HEADER))/sizeof(GET_PERFORMANCE_WRITE_SPEED_DESCRIPTOR));
//     ULONG       validDescriptors = 0;
//     ULONG       transferSize = sizeof(GET_PERFORMANCE_HEADER) + (maxDescriptors*sizeof(GET_PERFORMANCE_WRITE_SPEED_DESCRIPTOR));
//     CDB         cdb;
//
//     PAGED_CODE();
//
//     ScratchBuffer_BeginUse(DeviceExtension);
//
//     RtlZeroMemory(&cdb, sizeof(CDB));
//     // Set up the CDB
//     if (UseLegacyNominalPerformance)
//     {
//         cdb.GET_PERFORMANCE.OperationCode = SCSIOP_GET_PERFORMANCE;
//         cdb.GET_PERFORMANCE.Except = 0;
//         cdb.GET_PERFORMANCE.Write = 1;
//         cdb.GET_PERFORMANCE.Tolerance = 2; // only defined option
//         cdb.GET_PERFORMANCE.MaximumNumberOfDescriptors[1] = (UCHAR)maxDescriptors;
//         cdb.GET_PERFORMANCE.Type = 0; // legacy nominal descriptors
//     }
//     else
//     {
//         cdb.GET_PERFORMANCE.OperationCode = SCSIOP_GET_PERFORMANCE;
//         cdb.GET_PERFORMANCE.MaximumNumberOfDescriptors[1] = (UCHAR)maxDescriptors;
//         cdb.GET_PERFORMANCE.Type = 3; // write speed
//     }
//
//     status = ScratchBuffer_ExecuteCdbEx(DeviceExtension, NULL, transferSize, TRUE, &cdb, 12, CDROM_GET_PERFORMANCE_TIMEOUT);
//
//     // determine how many valid descriptors there actually are
//     if (NT_SUCCESS(status))
//     {
//         GET_PERFORMANCE_HEADER const* header = (GET_PERFORMANCE_HEADER const*)DeviceExtension->ScratchContext.ScratchBuffer;
//         ULONG temp1 = (header->TotalDataLength[0] << (8*3)) |
//                       (header->TotalDataLength[1] << (8*2)) |
//                       (header->TotalDataLength[2] << (8*1)) |
//                       (header->TotalDataLength[3] << (8*0)) ;
//
//         // adjust data size for header
//         if (temp1 + (ULONG)RTL_SIZEOF_THROUGH_FIELD(GET_PERFORMANCE_HEADER, TotalDataLength) < temp1)
//         {
//             temp1 = 0;
//         }
//         else if (temp1 != 0)
//         {
//             temp1 += RTL_SIZEOF_THROUGH_FIELD(GET_PERFORMANCE_HEADER, TotalDataLength);
//         }
//
//         if (temp1 == 0)
//         {
//             // no data returned
//         }
//         else if (temp1 <= sizeof(GET_PERFORMANCE_HEADER))
//         {
//             // only the header returned, no descriptors
//         }
//         else if (UseLegacyNominalPerformance &&
//                  ((header->Except != 0) || (header->Write == 0))
//                  )
//         {
//             // bad data being returned -- ignore it
//         }
//         else if (!UseLegacyNominalPerformance &&
//                  ((header->Except != 0) || (header->Write != 0))
//                  )
//         {
//             // returning Performance (Type 0) data, not requested Write Speed (Type 3) data
//         }
//         else if ( (temp1 - sizeof(GET_PERFORMANCE_HEADER)) % sizeof(GET_PERFORMANCE_WRITE_SPEED_DESCRIPTOR) != 0)
//         {
//             // Note: this size is identical for both descriptor types
//             C_ASSERT( sizeof(GET_PERFORMANCE_WRITE_SPEED_DESCRIPTOR) == sizeof(GET_PERFORMANCE_NOMINAL_PERFORMANCE_DESCRIPTOR));
//
//             // not returning valid data....
//         }
//         else // save how many are usable
//         {
//             // Note: this size is identical for both descriptor types
//             C_ASSERT( sizeof(GET_PERFORMANCE_WRITE_SPEED_DESCRIPTOR) == sizeof(GET_PERFORMANCE_NOMINAL_PERFORMANCE_DESCRIPTOR));
//
//             // take the smaller usable value
//             temp1 = min(temp1, DeviceExtension->ScratchContext.ScratchSrb->DataTransferLength);
//             // then determine the usable descriptors
//             validDescriptors = (temp1 - sizeof(GET_PERFORMANCE_HEADER)) / sizeof(GET_PERFORMANCE_WRITE_SPEED_DESCRIPTOR);
//         }
//     }
//
//     // The drive likely supports this command.
//     // Verify the data makes sense.
//     if (NT_SUCCESS(status))
//     {
//         ULONG       i;
//         GET_PERFORMANCE_HEADER const*                 header = (GET_PERFORMANCE_HEADER const*)DeviceExtension->ScratchContext.ScratchBuffer;
//         GET_PERFORMANCE_WRITE_SPEED_DESCRIPTOR const* descriptor = (GET_PERFORMANCE_WRITE_SPEED_DESCRIPTOR const*)(header+1); // pointer math
//
//         // NOTE: We could write this loop twice, once for each write descriptor type
//         //       However, the only fields of interest are the writeKBps field (Type 3) and
//         //       the EndPerformance field (Type 0), which both exist in the same exact
//         //       location and have essentially the same meaning.  So, just use the same
//         //       loop/structure pointers for both of the to simplify the readability of
//         //       this code.  The C_ASSERT()s here verify this at compile-time.
//
//         C_ASSERT( sizeof(GET_PERFORMANCE_WRITE_SPEED_DESCRIPTOR) == sizeof(GET_PERFORMANCE_NOMINAL_PERFORMANCE_DESCRIPTOR));
//         C_ASSERT( FIELD_OFFSET(GET_PERFORMANCE_WRITE_SPEED_DESCRIPTOR, WriteSpeedKilobytesPerSecond) ==
//                   FIELD_OFFSET(GET_PERFORMANCE_NOMINAL_PERFORMANCE_DESCRIPTOR, EndPerformance)
//                   );
//         C_ASSERT( RTL_FIELD_SIZE(GET_PERFORMANCE_WRITE_SPEED_DESCRIPTOR, WriteSpeedKilobytesPerSecond) ==
//                   RTL_FIELD_SIZE(GET_PERFORMANCE_NOMINAL_PERFORMANCE_DESCRIPTOR, EndPerformance)
//                   );
//
//         // loop through them all, and find the fastest listed write speed
//         for (i = 0; NT_SUCCESS(status) && (i <validDescriptors); descriptor++, i++)
//         {
//             ULONG const writeKBps =
//                 (descriptor->WriteSpeedKilobytesPerSecond[0] << (8*3)) |
//                 (descriptor->WriteSpeedKilobytesPerSecond[1] << (8*2)) |
//                 (descriptor->WriteSpeedKilobytesPerSecond[2] << (8*1)) |
//                 (descriptor->WriteSpeedKilobytesPerSecond[3] << (8*0)) ;
//
//             // Avoid overflow and still have good estimates
//             // 0x1 0000 0000 / 1000  == 0x00418937 == maximum writeKBps to multiple first
//             ULONG const sectorsPerSecond =
//                 (writeKBps > 0x00418937) ? // would overflow occur by multiplying by 1000?
//                 ((writeKBps / 2048) * 1000) : // must divide first, minimal loss of accuracy
//                 ((writeKBps * 1000) / 2048) ; // must multiply first, avoid loss of accuracy
//
//             if (sectorsPerSecond <= 0)
//             {
//                 break; // out of the loop -- no longer valid data (very defensive programming)
//             }
//
//             // we have at least one valid result, so prevent returning -1 as our result
//             if (result == -1) { result = MAXIMUM_RETRY_FOR_SINGLE_IO_IN_100NS_UNITS; }
//
//             // take the fastest speed (smallest wait time) we've found thus far
//             result = min(result, ConvertSectorsPerSecondTo100nsUnitsFor64kWrite(sectorsPerSecond));
//         }
//     }
//
//     ScratchBuffer_EndUse(DeviceExtension);
//
//     return result;
// }
//
// #pragma warning(pop) // un-sets any local warning changes
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
