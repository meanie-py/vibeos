use crate::types::*;
// On i386 there is no separate 32-bit thunk struct — the native SCSI_PASS_THROUGH
// *is* the 32-bit form. These aliases (only used in size_of bounds checks) keep the
// amd64-shared code compiling on i386 where SCSI_PASS_THROUGH32/_EX don't exist.
#[cfg(target_arch = "x86")]
use crate::types::{SCSI_PASS_THROUGH as SCSI_PASS_THROUGH32, SCSI_PASS_THROUGH_EX as SCSI_PASS_THROUGH32_EX};
#[cfg(target_arch = "x86")]
type CompletionInformation = u32;
#[cfg(not(target_arch = "x86"))]
type CompletionInformation = u64;
use kmdf_sys::*;
use ntkrnl_sys::ddk::{IRP, PIRP, IO_STACK_LOCATION, PIO_STACK_LOCATION, GUID, LPGUID, LPCGUID, DEVICE_OBJECT, PDEVICE_OBJECT, MDL, PMDL};
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{DISK_GEOMETRY, DISK_GEOMETRY_EX, DRIVE_LAYOUT_INFORMATION, DRIVE_LAYOUT_INFORMATION_EX, _KWAIT_REASON_Executive, GET_LENGTH_INFORMATION, IoCancelIrp, IoDeleteSymbolicLink, IoGetConfigurationInformation, IoGetNextIrpStackLocation, KeFlushQueuedDpcs, KeInitializeEvent, KeReadStateEvent, KeWaitForSingleObject, NTSTRSAFE_PWSTR, _EVENT_TYPE_NotificationEvent, PARTITION_INFORMATION, PARTITION_INFORMATION_EX, PDRIVER_OBJECT, PKEVENT, PREVENT_MEDIA_REMOVAL, PSTORAGE_PROPERTY_QUERY, PUNICODE_STRING, PsIsThreadTerminating, STORAGE_IDLE_POWERUP_REASON, STORAGE_PROPERTY_QUERY, STORAGE_READ_CAPACITY, VERIFY_INFORMATION, _UNICODE_STRING, PETHREAD, KEVENT};
use ntkrnl_sys::ddk::_DEVICE_RELATION_TYPE_BusRelations as BusRelations;
use ntkrnl_sys::ddk::_STORAGE_QUERY_TYPE_PropertyExistsQuery as PropertyExistsQuery;
use ntkrnl_sys::ddk::_STORAGE_QUERY_TYPE_PropertyStandardQuery as PropertyStandardQuery;
use ntkrnl_sys::ddk::_STORAGE_PROPERTY_ID_StorageAdapterProperty as StorageAdapterProperty;
use ntkrnl_sys::ddk::_STORAGE_PROPERTY_ID_StorageDeviceProperty as StorageDeviceProperty;
use ntkrnl_sys::ddk::_STORAGE_PROPERTY_ID_StorageDeviceUniqueIdProperty as StorageDeviceUniqueIdProperty;
use ntkrnl_sys::ddk::_STORAGE_PROPERTY_ID_StorageDeviceWriteCacheProperty as StorageDeviceWriteCacheProperty;
use ntkrnl_sys::ddk::_STORAGE_POWERUP_REASON_TYPE_StoragePowerupDeviceAttention as StoragePowerupDeviceAttention;
use core::ffi::c_void;
use core::ptr::addr_of_mut;
use kmdf_sys::_WDF_POWER_DEVICE_STATE_WdfPowerDeviceD3 as WdfPowerDeviceD3;
use kmdf_sys::_WDF_POWER_DEVICE_STATE_WdfPowerDeviceD3Final as WdfPowerDeviceD3Final;

// NTSTATUS constants not present in the kernel substrate (well-known NT values).
const STATUS_NO_MEDIA_IN_DEVICE: NTSTATUS = 0xC0000013u32 as i32;
const STATUS_INTERNAL_ERROR: NTSTATUS = 0xC00000E5u32 as i32;
const STATUS_INVALID_LEVEL: NTSTATUS = 0xC0000148u32 as i32;
// WPP trace flags/levels (gate tracing only; distinct nonzero values).
const TRACE_FLAG_POWER: u32 = 2; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)
const TRACE_FLAG_QUEUE: u32 = 5; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)
const TRACE_LEVEL_WARNING: u32 = 3; // storswtr.h:26-30

// These two WDF ioctl handlers are DEFINED `extern "C"` (aacs.rs / ioctl.rs);
// their decls must match that decoration — i386 was declaring them
// `extern "system"` -> undefined `_..@36`. Exposed once the corrected IOCTL
// consts routed dispatch into them. 2026-07-06.
extern "C" {
    fn DeviceHandleAacsReadMediaKeyBlock( dev: *mut CDROM_DEVICE_EXTENSION, req: WDFREQUEST, params: WDF_REQUEST_PARAMETERS, info: *mut usize, ) -> i32;
    fn RequestHandleGetPerformance( dev: *mut CDROM_DEVICE_EXTENSION, req: WDFREQUEST, params: WDF_REQUEST_PARAMETERS, info: *mut usize, ) -> i32;
}

extern "system" {
    fn KeQueryPerformanceCounter(PerformanceFrequency: *mut i64) -> i64;
}
extern "C" {
    fn CdromDbgPrint0(format: *const i8) -> u32;
    fn CdromGapLog();
    fn CdromDbgPrintForce2(format: *const i8, arg1: usize, arg2: usize) -> u32;
    fn CdromDbgPrint1(format: *const i8, arg1: usize) -> u32;
    fn CdromDbgPrint3(format: *const i8, arg1: usize, arg2: usize, arg3: usize) -> u32;
    fn CdromDbgPrint4(format: *const i8, arg1: usize, arg2: usize, arg3: usize, arg4: usize) -> u32;
    static CDROM_REQUEST_CONTEXT_TYPE_INFO: WDF_OBJECT_CONTEXT_TYPE_INFO;
    static FILE_OBJECT_CONTEXT_TYPE_INFO: WDF_OBJECT_CONTEXT_TYPE_INFO;
    static CDROM_DEVICE_EXTENSION_TYPE_INFO: WDF_OBJECT_CONTEXT_TYPE_INFO;
    static CDROM_DRIVER_EXTENSION_TYPE_INFO: WDF_OBJECT_CONTEXT_TYPE_INFO;
}
extern "C" {
    fn DeviceClaimRelease(ext: *mut CDROM_DEVICE_EXTENSION, release: u8) -> NTSTATUS;
    fn DeviceSendRequestSynchronously(device: WDFDEVICE, request: WDFREQUEST, sendFlags: BOOLEAN) -> NTSTATUS;
    fn CdromRequestCompletion(deviceExtension: *mut CDROM_DEVICE_EXTENSION, request: WDFREQUEST, status: NTSTATUS, information: ULONG_PTR);
    fn CdromPrepareRequestContext(request: WDFREQUEST, deviceExtension: *mut CDROM_DEVICE_EXTENSION);
    fn CdromGetReadWriteVerifyState(
        deviceExtension: *mut CDROM_DEVICE_EXTENSION,
        request: WDFREQUEST,
        deviceFlags: *mut ULONG,
        minorFunction: *mut u8,
        irpFlags: *mut u8,
    );
    fn CdromMarkVolumeVerifyChecked(request: WDFREQUEST);
    fn CdromInvokeSyncCallback(
        requestContext: *mut CDROM_REQUEST_CONTEXT,
        device: WDFDEVICE,
        request: WDFREQUEST,
    ) -> NTSTATUS;
    fn CdromDeviceEvtFileClose(fileObject: WDFFILEOBJECT);
    fn CdromTryHandleStorageControl(
        deviceExtension: *mut CDROM_DEVICE_EXTENSION,
        request: WDFREQUEST,
        requestParameters: *mut WDF_REQUEST_PARAMETERS,
    ) -> BOOLEAN;
    fn CdromTryHandleSerializedStorageControl(
        deviceExtension: *mut CDROM_DEVICE_EXTENSION,
        request: WDFREQUEST,
        requestParameters: *mut WDF_REQUEST_PARAMETERS,
    ) -> BOOLEAN;
}
extern "system" {
    fn CdromSequentialQueueEvtIoReadWrite(queue: WDFQUEUE, request: WDFREQUEST, length: usize);
    fn DriverGetExtension(Driver: WDFDRIVER) -> *mut CDROM_DRIVER_EXTENSION;
    fn DeviceGetExtension(Device: WDFDEVICE) -> *mut CDROM_DEVICE_EXTENSION;
    fn IoGetDevicePropertyData( PDO: *mut DEVICE_OBJECT, propertyKey: *const c_void, lcid: u32, flags: u32, size: u32, data: *mut c_void, requiredSize: *mut u32, dataType: *mut u32, ) -> NTSTATUS;
    fn RtlInitUnicodeString(s: *mut _UNICODE_STRING, src: *const u16);
    fn WDF_FILEOBJECT_CONFIG_INIT( cfg: *mut WDF_FILEOBJECT_CONFIG, evtFileCreate: Option<unsafe extern "system" fn(WDFDEVICE, WDFFILEOBJECT) -> ()>, evtFileClose: Option<unsafe extern "system" fn(WDFFILEOBJECT) -> ()>, evtFileCleanup: Option<unsafe extern "system" fn(WDFFILEOBJECT) -> ()>, );
    fn WDF_IO_QUEUE_CONFIG_INIT(cfg: *mut WDF_IO_QUEUE_CONFIG, dispatch: WDF_IO_QUEUE_DISPATCH_TYPE);
    fn WDF_IO_QUEUE_CONFIG_INIT_DEFAULT_QUEUE(cfg: *mut WDF_IO_QUEUE_CONFIG, dispatch: WDF_IO_QUEUE_DISPATCH_TYPE);
    fn WDF_IO_TARGET_OPEN_PARAMS_INIT_EXISTING_DEVICE( params: *mut WDF_IO_TARGET_OPEN_PARAMS, target: PDEVICE_OBJECT, );
    fn WDF_OBJECT_ATTRIBUTES_INIT(attr: *mut WDF_OBJECT_ATTRIBUTES);
    fn WDF_OBJECT_ATTRIBUTES_INIT_CONTEXT_TYPE( attr: *mut WDF_OBJECT_ATTRIBUTES, ctx: *const WDF_OBJECT_CONTEXT_TYPE_INFO, );
    fn WDF_PNPPOWER_EVENT_CALLBACKS_INIT(cbs: *mut WDF_PNPPOWER_EVENT_CALLBACKS);
    fn WDF_REMOVE_LOCK_OPTIONS_INIT(opts: *mut WDF_REMOVE_LOCK_OPTIONS, flags: u32);
    fn WDF_WORKITEM_CONFIG_INIT(cfg: *mut WDF_WORKITEM_CONFIG, routine: unsafe extern "system" fn(WDFWORKITEM) -> ());
    fn DeviceEvtSelfManagedIoInit(Dev: WDFDEVICE) -> NTSTATUS;
    fn RequestProcessSetPower(Dev: WDFDEVICE, Irp: PIRP) -> NTSTATUS;
    fn RequestProcessShutdownFlush(Dev: WDFDEVICE, Irp: PIRP) -> NTSTATUS;
    fn PoQueryWatchdogTime(LowerPdo: *mut core::ffi::c_void, secondsRemaining: *mut u32) -> u8;
    fn SET_FLAG(p: *mut u32, flag: u32);
    fn TEST_FLAG(flags: u32, mask: u32) -> i32;
    fn PagedCode();
    fn FileObjectGetContext(fileObject: WDFFILEOBJECT) -> *mut FILE_OBJECT_CONTEXT;
    fn DeviceSendSrbSynchronously(Device: *mut core::ffi::c_void, Srb: *mut SCSI_REQUEST_BLOCK, Buffer: *mut core::ffi::c_void, Length: u32, WriteToDevice: u8, BytesReturned: *mut u32) -> NTSTATUS;
    fn RequestHandleEventNotification( deviceExtension: PCDROM_DEVICE_EXTENSION, Request: WDFREQUEST, RequestParameters: *const WDF_REQUEST_PARAMETERS, dataLength: *mut usize, ) -> NTSTATUS;
    fn RequestGetCurrentStackLocationFlags(Request: WDFREQUEST) -> UCHAR;
    fn IoInvalidateDeviceRelations( Pdo: *mut core::ffi::c_void, Relations: u32, );
    fn RequestDummyCompletionRoutine( request: WDFREQUEST, target: WDFIOTARGET, params: *mut core::ffi::c_void, context: *mut core::ffi::c_void, );
    fn RequestGetContext(Request: *mut c_void) -> PCDROM_REQUEST_CONTEXT;
    fn WDF_REQUEST_PARAMETERS_INIT(params: *mut WDF_REQUEST_PARAMETERS);
    fn NT_ASSERT(condition: i32);
    fn WDF_DRIVER_CONFIG_INIT( Config: *mut WDF_DRIVER_CONFIG, EvtDeviceAdd: PFN_WDF_DRIVER_DEVICE_ADD, );
    fn KeGetCurrentIrql() -> u8;
    fn RequestHandleVolumeOnline( dev: *mut CDROM_DEVICE_EXTENSION, req: WDFREQUEST, params: WDF_REQUEST_PARAMETERS, info: *mut usize, ) -> i32;
    fn PsGetCurrentThread() -> PETHREAD;
    fn RequestValidateExclusiveAccess( Request: WDFREQUEST, RequestParameters: WDF_REQUEST_PARAMETERS, DataLength: *mut usize, ) -> NTSTATUS;
    fn RequestSetContextFields(Request: WDFREQUEST, Handler: PSYNC_HANDLER) -> NTSTATUS;
    fn RequestHandleUnknownIoctl(Device: WDFDEVICE, Request: WDFREQUEST) -> NTSTATUS;
    fn RequestHandleQueryPropertyRetrieveCachedData( Extension: *mut CDROM_DEVICE_EXTENSION, Request: WDFREQUEST, Params: *const WDF_REQUEST_PARAMETERS, DataLength: *mut usize, ) -> NTSTATUS;
}
unsafe extern "C" {
    fn DeviceDeallocateMmcResources(Device: *mut core::ffi::c_void);
    fn ScratchBuffer_Deallocate(devExt: *mut CDROM_DEVICE_EXTENSION);
    fn RequestHandleGetInquiryData( deviceExtension: PCDROM_DEVICE_EXTENSION, Request: WDFREQUEST, RequestParameters: WDF_REQUEST_PARAMETERS, dataLength: *mut usize, ) -> NTSTATUS;
    fn RequestHandleMountQueryUniqueId( deviceExtension: PCDROM_DEVICE_EXTENSION, Request: WDFREQUEST, RequestParameters: WDF_REQUEST_PARAMETERS, dataLength: *mut usize, ) -> NTSTATUS;
    fn RequestHandleMountQueryDeviceName( deviceExtension: PCDROM_DEVICE_EXTENSION, Request: WDFREQUEST, RequestParameters: WDF_REQUEST_PARAMETERS, dataLength: *mut usize, ) -> NTSTATUS;
    fn RequestHandleMountQuerySuggestedLinkName( deviceExtension: PCDROM_DEVICE_EXTENSION, Request: WDFREQUEST, RequestParameters: WDF_REQUEST_PARAMETERS, dataLength: *mut usize, ) -> NTSTATUS;
    fn RequestHandleGetDeviceNumber( deviceExtension: PCDROM_DEVICE_EXTENSION, Request: WDFREQUEST, RequestParameters: WDF_REQUEST_PARAMETERS, dataLength: *mut usize, ) -> NTSTATUS;
    fn RequestHandleGetHotPlugInfo( deviceExtension: PCDROM_DEVICE_EXTENSION, Request: WDFREQUEST, RequestParameters: WDF_REQUEST_PARAMETERS, dataLength: *mut usize, ) -> NTSTATUS;
    fn RequestHandleSetHotPlugInfo( deviceExtension: PCDROM_DEVICE_EXTENSION, Request: WDFREQUEST, RequestParameters: WDF_REQUEST_PARAMETERS, dataLength: *mut usize, ) -> NTSTATUS;
    fn RequestValidateAacsBindingNonce( deviceExtension: *mut CDROM_DEVICE_EXTENSION, Request: WDFREQUEST, RequestParameters: WDF_REQUEST_PARAMETERS, dataLength: *mut usize, ) -> NTSTATUS;
    fn RequestValidateAacsEndSession( deviceExtension: *mut CDROM_DEVICE_EXTENSION, Request: WDFREQUEST, RequestParameters: WDF_REQUEST_PARAMETERS, dataLength: *mut usize, ) -> NTSTATUS;
    fn RequestValidateAacsGetCertificate( deviceExtension: *mut CDROM_DEVICE_EXTENSION, Request: WDFREQUEST, RequestParameters: WDF_REQUEST_PARAMETERS, dataLength: *mut usize, ) -> NTSTATUS;
    fn RequestValidateAacsGetChallengeKey( deviceExtension: *mut CDROM_DEVICE_EXTENSION, Request: WDFREQUEST, RequestParameters: WDF_REQUEST_PARAMETERS, dataLength: *mut usize, ) -> NTSTATUS;
    fn RequestValidateAacsReadMediaId( deviceExtension: *mut CDROM_DEVICE_EXTENSION, Request: WDFREQUEST, RequestParameters: WDF_REQUEST_PARAMETERS, dataLength: *mut usize, ) -> NTSTATUS;
    fn RequestValidateAacsReadMediaKeyBlock( deviceExtension: *mut CDROM_DEVICE_EXTENSION, Request: WDFREQUEST, RequestParameters: WDF_REQUEST_PARAMETERS, dataLength: *mut usize, ) -> NTSTATUS;
    fn RequestValidateAacsReadSerialNumber( deviceExtension: *mut CDROM_DEVICE_EXTENSION, Request: WDFREQUEST, RequestParameters: WDF_REQUEST_PARAMETERS, dataLength: *mut usize, ) -> NTSTATUS;
    fn RequestValidateAacsReadVolumeId( deviceExtension: *mut CDROM_DEVICE_EXTENSION, Request: WDFREQUEST, RequestParameters: WDF_REQUEST_PARAMETERS, dataLength: *mut usize, ) -> NTSTATUS;
    fn RequestValidateAacsSendCertificate( deviceExtension: *mut CDROM_DEVICE_EXTENSION, Request: WDFREQUEST, RequestParameters: WDF_REQUEST_PARAMETERS, dataLength: *mut usize, ) -> NTSTATUS;
    fn RequestValidateAacsSendChallengeKey( deviceExtension: *mut CDROM_DEVICE_EXTENSION, Request: WDFREQUEST, RequestParameters: WDF_REQUEST_PARAMETERS, dataLength: *mut usize, ) -> NTSTATUS;
    fn RequestValidateAacsStartSession( deviceExtension: *mut CDROM_DEVICE_EXTENSION, RequestParameters: WDF_REQUEST_PARAMETERS, dataLength: *mut usize, ) -> NTSTATUS;
    fn RequestValidateDvdEndSession( deviceExtension: *mut CDROM_DEVICE_EXTENSION, Request: WDFREQUEST, RequestParameters: WDF_REQUEST_PARAMETERS, dataLength: *mut usize, ) -> NTSTATUS;
    fn RequestValidateDvdReadKey( deviceExtension: *mut CDROM_DEVICE_EXTENSION, Request: WDFREQUEST, RequestParameters: WDF_REQUEST_PARAMETERS, dataLength: *mut usize, ) -> NTSTATUS;
    fn RequestValidateDvdReadStructure( deviceExtension: *mut CDROM_DEVICE_EXTENSION, RequestParameters: WDF_REQUEST_PARAMETERS, dataLength: *mut usize, ) -> NTSTATUS;
    fn RequestValidateDvdSendKey( deviceExtension: *mut CDROM_DEVICE_EXTENSION, Request: WDFREQUEST, RequestParameters: WDF_REQUEST_PARAMETERS, dataLength: *mut usize, ) -> NTSTATUS;
    fn RequestValidateDvdStartSession( deviceExtension: *mut CDROM_DEVICE_EXTENSION, RequestParameters: WDF_REQUEST_PARAMETERS, dataLength: *mut usize, ) -> NTSTATUS;
    fn RequestValidateEnableStreaming( Request: WDFREQUEST, RequestParameters: WDF_REQUEST_PARAMETERS, dataLength: *mut usize, ) -> NTSTATUS;
    fn RequestValidateGetConfiguration( deviceExtension: *mut CDROM_DEVICE_EXTENSION, Request: WDFREQUEST, RequestParameters: WDF_REQUEST_PARAMETERS, dataLength: *mut usize, ) -> NTSTATUS;
    fn RequestValidateGetLastSession( deviceExtension: *mut CDROM_DEVICE_EXTENSION, RequestParameters: WDF_REQUEST_PARAMETERS, dataLength: *mut usize, ) -> NTSTATUS;
    fn RequestValidateGetPerformance( Request: WDFREQUEST, RequestParameters: WDF_REQUEST_PARAMETERS, dataLength: *mut usize, ) -> NTSTATUS;
    fn RequestValidatePersistentReserve( deviceExtension: *mut CDROM_DEVICE_EXTENSION, Request: WDFREQUEST, RequestParameters: WDF_REQUEST_PARAMETERS, dataLength: *mut usize, ) -> NTSTATUS;
    fn RequestValidateReadQChannel( Request: WDFREQUEST, RequestParameters: WDF_REQUEST_PARAMETERS, dataLength: *mut usize, ) -> NTSTATUS;
    fn RequestValidateReadToc( deviceExtension: *mut CDROM_DEVICE_EXTENSION, RequestParameters: WDF_REQUEST_PARAMETERS, dataLength: *mut usize, ) -> NTSTATUS;
    fn RequestValidateReadTocEx( deviceExtension: *mut CDROM_DEVICE_EXTENSION, Request: WDFREQUEST, RequestParameters: WDF_REQUEST_PARAMETERS, dataLength: *mut usize, ) -> NTSTATUS;
    fn RequestValidateSendOpcInformation( Request: WDFREQUEST, RequestParameters: WDF_REQUEST_PARAMETERS, dataLength: *mut usize, ) -> NTSTATUS;
    fn RequestValidateSetSpeed( deviceExtension: *mut CDROM_DEVICE_EXTENSION, Request: WDFREQUEST, RequestParameters: WDF_REQUEST_PARAMETERS, dataLength: *mut usize, ) -> NTSTATUS;
    fn RequestHandleReadTOC( dev: *mut CDROM_DEVICE_EXTENSION, req: WDFREQUEST, params: WDF_REQUEST_PARAMETERS, info: *mut usize, ) -> i32;
    fn RequestHandleReadTocEx( dev: *mut CDROM_DEVICE_EXTENSION, req: WDFREQUEST, params: WDF_REQUEST_PARAMETERS, info: *mut usize, ) -> i32;
    fn RequestHandleGetDriveGeometry( dev: *mut CDROM_DEVICE_EXTENSION, req: WDFREQUEST, params: WDF_REQUEST_PARAMETERS, info: *mut usize, ) -> i32;
    fn RequestHandleDiskVerify( dev: *mut CDROM_DEVICE_EXTENSION, req: WDFREQUEST, params: WDF_REQUEST_PARAMETERS, info: *mut usize, ) -> i32;
    fn RequestHandleCheckVerify( dev: *mut CDROM_DEVICE_EXTENSION, req: WDFREQUEST, params: WDF_REQUEST_PARAMETERS, info: *mut usize, ) -> i32;
    fn DeviceHandlePlayAudioMsf( dev: *mut CDROM_DEVICE_EXTENSION, req: WDFREQUEST, params: WDF_REQUEST_PARAMETERS, info: *mut usize, ) -> i32;
    fn DeviceHandleReadQChannel( dev: *mut CDROM_DEVICE_EXTENSION, req: WDFREQUEST, params: WDF_REQUEST_PARAMETERS, info: *mut usize, ) -> i32;
    fn DeviceHandlePauseAudio( dev: *mut CDROM_DEVICE_EXTENSION, req: WDFREQUEST, info: *mut usize, ) -> i32;
    fn DeviceHandleResumeAudio( dev: *mut CDROM_DEVICE_EXTENSION, req: WDFREQUEST, info: *mut usize, ) -> i32;
    fn DeviceHandleSeekAudioMsf( dev: *mut CDROM_DEVICE_EXTENSION, req: WDFREQUEST, params: WDF_REQUEST_PARAMETERS, info: *mut usize, ) -> i32;
    fn DeviceHandleStopAudio( dev: *mut CDROM_DEVICE_EXTENSION, req: WDFREQUEST, info: *mut usize, ) -> i32;
    fn RequestHandleGetDvdRegion( dev: *mut CDROM_DEVICE_EXTENSION, req: WDFREQUEST, info: *mut usize, ) -> i32;
    fn DeviceHandleReadDvdStructure( dev: *mut CDROM_DEVICE_EXTENSION, req: WDFREQUEST, params: WDF_REQUEST_PARAMETERS, info: *mut usize, ) -> i32;
    fn DeviceHandleDvdEndSession( dev: *mut CDROM_DEVICE_EXTENSION, req: WDFREQUEST, params: WDF_REQUEST_PARAMETERS, info: *mut usize, ) -> i32;
    fn DeviceHandleDvdStartSessionReadKey( dev: *mut CDROM_DEVICE_EXTENSION, req: WDFREQUEST, params: WDF_REQUEST_PARAMETERS, info: *mut usize, ) -> i32;
    fn DeviceHandleDvdSendKey( dev: *mut CDROM_DEVICE_EXTENSION, req: WDFREQUEST, params: WDF_REQUEST_PARAMETERS, info: *mut usize, ) -> i32;
    fn DeviceHandleSetReadAhead( dev: *mut CDROM_DEVICE_EXTENSION, req: WDFREQUEST, params: WDF_REQUEST_PARAMETERS, info: *mut usize, ) -> i32;
    fn DeviceHandleAacsStartSession( dev: *mut CDROM_DEVICE_EXTENSION, req: WDFREQUEST, params: WDF_REQUEST_PARAMETERS, info: *mut usize, ) -> i32;
    fn DeviceHandleAacsEndSession( dev: *mut CDROM_DEVICE_EXTENSION, req: WDFREQUEST, params: WDF_REQUEST_PARAMETERS, info: *mut usize, ) -> i32;
    fn DeviceHandleAacsSendCertificate( dev: *mut CDROM_DEVICE_EXTENSION, req: WDFREQUEST, params: WDF_REQUEST_PARAMETERS, info: *mut usize, ) -> i32;
    fn DeviceHandleAacsGetCertificate( dev: *mut CDROM_DEVICE_EXTENSION, req: WDFREQUEST, params: WDF_REQUEST_PARAMETERS, info: *mut usize, ) -> i32;
    fn DeviceHandleAacsGetChallengeKey( dev: *mut CDROM_DEVICE_EXTENSION, req: WDFREQUEST, params: WDF_REQUEST_PARAMETERS, info: *mut usize, ) -> i32;
    fn DeviceHandleSendChallengeKey( dev: *mut CDROM_DEVICE_EXTENSION, req: WDFREQUEST, params: WDF_REQUEST_PARAMETERS, info: *mut usize, ) -> i32;
    fn DeviceHandleReadVolumeId( dev: *mut CDROM_DEVICE_EXTENSION, req: WDFREQUEST, params: WDF_REQUEST_PARAMETERS, info: *mut usize, ) -> i32;
    fn DeviceHandleAacsReadSerialNumber( dev: *mut CDROM_DEVICE_EXTENSION, req: WDFREQUEST, params: WDF_REQUEST_PARAMETERS, info: *mut usize, ) -> i32;
    fn DeviceHandleAacsReadMediaId( dev: *mut CDROM_DEVICE_EXTENSION, req: WDFREQUEST, params: WDF_REQUEST_PARAMETERS, info: *mut usize, ) -> i32;
    fn DeviceHandleAacsGenerateBindingNonce( dev: *mut CDROM_DEVICE_EXTENSION, req: WDFREQUEST, params: WDF_REQUEST_PARAMETERS, info: *mut usize, ) -> i32;
    fn RequestHandleEjectionControl( dev: *mut CDROM_DEVICE_EXTENSION, req: WDFREQUEST, params: WDF_REQUEST_PARAMETERS, info: *mut usize, ) -> i32;
    fn RequestHandleLoadEjectMedia( dev: *mut CDROM_DEVICE_EXTENSION, req: WDFREQUEST, params: WDF_REQUEST_PARAMETERS, info: *mut usize, ) -> i32;
    fn RequestHandleMcnControl( dev: *mut CDROM_DEVICE_EXTENSION, req: WDFREQUEST, info: *mut usize, ) -> i32;
    fn RequestHandleReserveRelease( dev: *mut CDROM_DEVICE_EXTENSION, req: WDFREQUEST, params: WDF_REQUEST_PARAMETERS, info: *mut usize, ) -> i32;
    fn RequestHandlePersistentReserve( dev: *mut CDROM_DEVICE_EXTENSION, req: WDFREQUEST, params: WDF_REQUEST_PARAMETERS, info: *mut usize, ) -> i32;
    fn RequestHandleEnableStreaming( dev: *mut CDROM_DEVICE_EXTENSION, req: WDFREQUEST, info: *mut usize, ) -> i32;
    fn RequestHandleSendOpcInformation( dev: *mut CDROM_DEVICE_EXTENSION, req: WDFREQUEST, info: *mut usize, ) -> i32;
    fn RequestHandleMcnSyncFakeIoctl( dev: *mut CDROM_DEVICE_EXTENSION, info: *mut usize, ) -> i32;
    fn RequestHandleAreVolumesReady( dev: *mut CDROM_DEVICE_EXTENSION, req: WDFREQUEST, params: WDF_REQUEST_PARAMETERS, info: *mut usize, ) -> i32;
    fn IoGetCurrentIrpStackLocation(Irp: PIRP) -> PIO_STACK_LOCATION;
    fn DEVICE_TYPE_FROM_CTL_CODE(io_control_code: u32) -> u32;
}
unsafe extern "C" {
}
unsafe extern "C" {
    fn DeviceCreateWellKnownName(ext: *mut CDROM_DEVICE_EXTENSION) -> NTSTATUS;
}
unsafe extern "C" {
    fn DeviceGetTimeOutValueFromRegistry() -> u32;
}
unsafe extern "C" {
    fn DeviceInitPowerContext(ext: *mut CDROM_DEVICE_EXTENSION) -> NTSTATUS;
}
unsafe extern "C" {
    fn DeviceInitReleaseQueueContext(ext: *mut CDROM_DEVICE_EXTENSION) -> NTSTATUS;
}
unsafe extern "C" {
    fn DeviceRegisterInterface(ext: *mut CDROM_DEVICE_EXTENSION, InterfaceType: i32) -> NTSTATUS;
}
unsafe extern "C" {
    fn UNREFERENCED_PARAMETER(param: usize);
}
unsafe extern "C" {
    fn FREE_POOL(ptr: *mut core::ffi::c_void);
}
unsafe extern "C" {
    fn DeviceReleaseMcnResources(devExt: *mut CDROM_DEVICE_EXTENSION);
}
unsafe extern "C" {
    fn DeviceReleaseZPODDResources(devExt: *mut CDROM_DEVICE_EXTENSION);
}
unsafe extern "C" {
    fn DeviceZPODDGetPowerupReason( DeviceExtension: PCDROM_DEVICE_EXTENSION, PowerupReason: *mut STORAGE_IDLE_POWERUP_REASON, ) -> NTSTATUS;
}
unsafe extern "C" {
    fn DeviceSendIoctlAsynchronously( DeviceExtension: PCDROM_DEVICE_EXTENSION, IoctlCode: u32, DeviceObject: PDEVICE_OBJECT, );
}
unsafe extern "C" {
    fn DeviceDisableMainTimer(extension: *mut CDROM_DEVICE_EXTENSION);
}
unsafe extern "C" {
    fn DeviceSendPowerDownProcessRequest(extension: *mut CDROM_DEVICE_EXTENSION, param2: *mut core::ffi::c_void, param3: *mut core::ffi::c_void) -> i32;
}
unsafe extern "C" {
    fn PowerContextBeginUse(extension: *mut CDROM_DEVICE_EXTENSION) -> i32;
}
unsafe extern "C" {
    fn PowerContextEndUse(extension: *mut CDROM_DEVICE_EXTENSION) -> u32;
}
unsafe extern "C" {
    fn DeviceEnableMediaChangeDetection(DeviceExtension: *mut CDROM_DEVICE_EXTENSION, FileObjectContext: *mut FILE_OBJECT_CONTEXT, Enable: i32);
}
unsafe extern "C" {
    fn RequestSend( device_ext: *mut CDROM_DEVICE_EXTENSION, request: WDFREQUEST, target: WDFIOTARGET, flags: u32, request_sent: *mut u8, ) -> NTSTATUS;
}
unsafe extern "C" {
    fn DeviceUnlockExclusive(deviceExtension: *mut CDROM_DEVICE_EXTENSION, FileObject: WDFFILEOBJECT, Flag: BOOL) -> NTSTATUS;
}
unsafe extern "C" {
    fn EXCLUSIVE_MODE(cdData: *const CDROM_DATA) -> BOOL;
}
unsafe extern "C" {
    fn EXCLUSIVE_OWNER(cdData: *const CDROM_DATA, FileObject: WDFFILEOBJECT) -> u8;
}
unsafe extern "C" {
    fn RequestSetReceivedTime(Request: *mut core::ffi::c_void);
}
extern "C" {
    fn DeviceEnableMainTimer(DeviceExtension: PCDROM_DEVICE_EXTENSION) -> u32;
}
extern "C" {
    fn RequestHandleGetMediaTypeEx( deviceExtension: PCDROM_DEVICE_EXTENSION, Request: WDFREQUEST, dataLength: *mut usize, ) -> NTSTATUS;
    fn RequestValidateRawRead( deviceExtension: *mut CDROM_DEVICE_EXTENSION, Request: WDFREQUEST, RequestParameters: WDF_REQUEST_PARAMETERS, dataLength: *mut usize, ) -> NTSTATUS;
    fn RequestHandleGetConfiguration( dev: *mut CDROM_DEVICE_EXTENSION, req: WDFREQUEST, params: WDF_REQUEST_PARAMETERS, info: *mut usize, ) -> i32;
    fn DeviceHandleRawRead( dev: *mut CDROM_DEVICE_EXTENSION, req: WDFREQUEST, params: WDF_REQUEST_PARAMETERS, info: *mut usize, ) -> i32;
    fn RequestHandleFakePartitionInfo( dev: *mut CDROM_DEVICE_EXTENSION, req: WDFREQUEST, params: WDF_REQUEST_PARAMETERS, info: *mut usize, ) -> i32;
    fn DeviceHandleGetSetVolume( dev: *mut CDROM_DEVICE_EXTENSION, req: WDFREQUEST, params: WDF_REQUEST_PARAMETERS, info: *mut usize, ) -> i32;
    fn DeviceHandleSetSpeed( dev: *mut CDROM_DEVICE_EXTENSION, req: WDFREQUEST, params: WDF_REQUEST_PARAMETERS, info: *mut usize, ) -> i32;
    fn DeviceHandleAacsReadBindingNonce( dev: *mut CDROM_DEVICE_EXTENSION, req: WDFREQUEST, params: WDF_REQUEST_PARAMETERS, info: *mut usize, ) -> i32;
    fn RequestValidateReadWrite( DeviceExtension: *mut CDROM_DEVICE_EXTENSION, Request: WDFREQUEST, Params: *mut WDF_REQUEST_PARAMETERS, ) -> NTSTATUS;
    fn RequestHandleReadWrite( DeviceExtension: *mut CDROM_DEVICE_EXTENSION, Request: WDFREQUEST, Params: *mut WDF_REQUEST_PARAMETERS, ) -> NTSTATUS;
    fn DeviceUpdateMmcCapabilities(Device: WDFDEVICE) -> NTSTATUS;
}

// ---- file-scope preamble (translated) ----
pub const DEBUG_MAIN_SOURCE: i32 = 1;
extern "C" {
    fn BootEnvironmentIsWinPE() -> BOOLEAN;
}

const TRACE_LEVEL_INFORMATION: u32 = 4; // storswtr.h:26-30
const TRACE_FLAG_INIT: u32 = 8; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)

// Forward‑declared constants (unknown values, placeholders)
const TRACE_LEVEL_FATAL: u32 = 1; // storswtr.h:26-30
const TRACE_FLAG_PNP: u32 = 1; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)
const CDROM_TAG_STRINGS: u32 = 0x73436353; // 'CDRM' in LE
const SCSI_CDROM_TIMEOUT: u32 = 0xa;       // seconds, placeholder  // was 1800 (LLM-invented); bindgen-correct
const WRITE_RETRY_DELAY_DVD_1x: u64 = 473_373; // 100ns units (47.3ms) — C cdrom.h:118 ((LONGLONG)473373); was 10_000_000 (LLM-invented 1s placeholder)
const SRB_FLAGS_DISABLE_SYNCH_TRANSFER: u32 = 0x8; // placeholder  // was 0x10 (LLM-invented); bindgen-correct
const INVALID_SESSION: u32 = 0xFFFFFFFF;
const DEVPROP_TYPE_EMPTY: u32 = 0;
const WDF_REMOVE_LOCK_OPTION_ACQUIRE_FOR_IO: u32 = 1; // placeholder
const FILE_DEVICE_CD_ROM: u32 = 0x00000002;
const FILE_REMOVABLE_MEDIA: u32 = 0x01;
const WdfFileObjectWdfCannotUseFsContexts: u32 = 4; // WDF_FILEOBJECT_CLASS
const WdfTrue: u32 = 1;
const WdfSynchronizationScopeNone: u32 = 4;   // WDF_SYNCHRONIZATION_SCOPE (0=Invalid,1=Inherit,2=Device,3=Queue,4=None)
const WdfExecutionLevelPassive: u32 = 2;       // WDF_EXECUTION_LEVEL (0=Invalid,1=Inherit,2=Passive,3=Dispatch)
const WdfExecutionLevelDispatch: u32 = 3;
const WdfFileObjectCanBeOptional: u32 = 0x80000000; // WDF_FILEOBJECT_CLASS flag
const WdfDeviceIoDirect: u32 = 3; // WDF_DEVICE_IO_TYPE (1=Neither,2=Buffered,3=Direct,4=BufferedOrDirect)
const WdfFalse: u32 = 0;
const WdfIoQueueDispatchSequential: u32 = 1; // WDF_IO_QUEUE_DISPATCH_TYPE (0=Invalid,1=Seq,2=Parallel,3=Manual)
const WdfIoQueueDispatchParallel: u32 = 2;
const WdfIoQueueDispatchManual: u32 = 3;
const WdfRequestTypeCreate: u32 = 0; // WDF_REQUEST_TYPE
// CDROM_DEVICE_INTERFACES enum selectors (NOT GUIDs — DeviceRegisterInterface maps
// the selector to the real GUID_DEVINTERFACE_CDROM / MOUNTDEV_MOUNTED_DEVICE_GUID).
const CdRomDeviceInterface: i32 = 0;
const MountedDeviceInterface: i32 = 1;
const DEVPKEY_Device_SessionId: DEVPROPKEY = DEVPROPKEY { fmtid: DEVPROPGUID { Data1: 0, Data2: 0, Data3: 0, Data4: [0;8] }, pid: 10 }; // placeholder

// Context type info statics (generated by WDF_DECLARE_CONTEXT_TYPE)

// Function pointers for callbacks (cast later)

// Forward declare unknown structs referenced in the body
struct CONFIGURATION_INFORMATION;

type NTSTATUS = i32;
type BOOLEAN = u8;
type UCHAR = u8;

// WDF_REQUEST_PARAMETERS assumed defined elsewhere (with a Parameters union containing DeviceIoControl.IoControlCode)
// The layout must match the Windows driver framework definition.

const CDROM_FLAG_WINPE_MODE: u32 = 0x00000001;

type BOOL = i32;

const IOCTL_CDROM_READ_TOC: u32 = 0x24000;
const IOCTL_CDROM_GET_LAST_SESSION: u32 = 0x24038;
const IOCTL_CDROM_READ_TOC_EX: u32 = 0x24054;
const IOCTL_CDROM_GET_CONFIGURATION: u32 = 0x24058;
const IOCTL_CDROM_RAW_READ: u32 = 0x2403E;
const IOCTL_DISK_GET_LENGTH_INFO: u32 = 0x0007405C;
const IOCTL_DISK_GET_DRIVE_GEOMETRY: u32 = 0x00070000;
const IOCTL_DISK_GET_DRIVE_GEOMETRY_EX: u32 = 0x000700A0;
const IOCTL_CDROM_GET_DRIVE_GEOMETRY: u32 = 0x2404C;
const IOCTL_CDROM_GET_DRIVE_GEOMETRY_EX: u32 = 0x24050;
const IOCTL_STORAGE_READ_CAPACITY: u32 = 0x2D5140;
const IOCTL_DISK_VERIFY: u32 = 0x00070014;
const IOCTL_STORAGE_CHECK_VERIFY: u32 = 0x2D4800;
const IOCTL_STORAGE_CHECK_VERIFY2: u32 = 0x2D0800;
const IOCTL_DISK_GET_DRIVE_LAYOUT: u32 = 0x0007400C;
const IOCTL_DISK_GET_DRIVE_LAYOUT_EX: u32 = 0x00070050;
const IOCTL_DISK_GET_PARTITION_INFO: u32 = 0x00074004;
const IOCTL_DISK_GET_PARTITION_INFO_EX: u32 = 0x00070048;
const IOCTL_DISK_IS_WRITABLE: u32 = 0x00070024;
const IOCTL_CDROM_PLAY_AUDIO_MSF: u32 = 0x24018;
const IOCTL_CDROM_READ_Q_CHANNEL: u32 = 0x2402C;
const IOCTL_CDROM_PAUSE_AUDIO: u32 = 0x2400C;
const IOCTL_CDROM_RESUME_AUDIO: u32 = 0x24010;
const IOCTL_CDROM_SEEK_AUDIO_MSF: u32 = 0x24004;
const IOCTL_CDROM_STOP_AUDIO: u32 = 0x24008;
const IOCTL_CDROM_GET_VOLUME: u32 = 0x24014;
const IOCTL_CDROM_SET_VOLUME: u32 = 0x24028;
const IOCTL_DVD_GET_REGION: u32 = 0x00335014; // FIXED 2026-07-05: translation invented wrong value (verified vs psdk headers)
const IOCTL_DVD_READ_STRUCTURE: u32 = 0x00335140; // FIXED 2026-07-05: translation invented wrong value (verified vs psdk headers)
const IOCTL_DVD_END_SESSION: u32 = 0x0033500C; // FIXED 2026-07-05: translation invented wrong value (verified vs psdk headers)
const IOCTL_DVD_START_SESSION: u32 = 0x00335000; // FIXED 2026-07-05: translation invented wrong value (verified vs psdk headers)
const IOCTL_DVD_READ_KEY: u32 = 0x00335004; // FIXED 2026-07-05: translation invented wrong value (verified vs psdk headers)
const IOCTL_DVD_SEND_KEY: u32 = 0x00335008; // FIXED 2026-07-05: translation invented wrong value (verified vs psdk headers)
const IOCTL_DVD_SEND_KEY2: u32 = 0x0033D018; // FIXED 2026-07-05: translation invented wrong value (verified vs psdk headers)
const IOCTL_STORAGE_SET_READ_AHEAD: u32 = 0x002D4400; // FIXED 2026-07-05: translation invented wrong value (verified vs psdk headers)
const IOCTL_CDROM_SET_SPEED: u32 = 0x24060;
const IOCTL_CDROM_EXCLUSIVE_ACCESS: u32 = 0x0002C05C; // ADDED 2026-07-05: was unresolved in RequestDispatchSyncWithSequentialQueue match -> bind-all catch-all swallowed every ioctl
const IOCTL_AACS_READ_MEDIA_KEY_BLOCK_SIZE: u32 = 0x003350C0; // FIXED 2026-07-05: translation invented wrong value (verified vs psdk headers)
const IOCTL_AACS_READ_MEDIA_KEY_BLOCK: u32 = 0x003350C4; // FIXED 2026-07-05: translation invented wrong value (verified vs psdk headers)
const IOCTL_AACS_START_SESSION: u32 = 0x003350C8; // FIXED 2026-07-05: translation invented wrong value (verified vs psdk headers)
const IOCTL_AACS_END_SESSION: u32 = 0x003350CC; // FIXED 2026-07-05: translation invented wrong value (verified vs psdk headers)
const IOCTL_AACS_SEND_CERTIFICATE: u32 = 0x003350D0; // FIXED 2026-07-05: translation invented wrong value (verified vs psdk headers)
const IOCTL_AACS_GET_CERTIFICATE: u32 = 0x003350D4; // FIXED 2026-07-05: translation invented wrong value (verified vs psdk headers)
const IOCTL_AACS_GET_CHALLENGE_KEY: u32 = 0x003350D8; // FIXED 2026-07-05: translation invented wrong value (verified vs psdk headers)
const IOCTL_AACS_SEND_CHALLENGE_KEY: u32 = 0x003350DC; // FIXED 2026-07-05: translation invented wrong value (verified vs psdk headers)
const IOCTL_AACS_READ_VOLUME_ID: u32 = 0x003350E0; // FIXED 2026-07-05: translation invented wrong value (verified vs psdk headers)
const IOCTL_AACS_READ_SERIAL_NUMBER: u32 = 0x003350E4; // FIXED 2026-07-05: translation invented wrong value (verified vs psdk headers)
const IOCTL_AACS_READ_MEDIA_ID: u32 = 0x003350E8; // FIXED 2026-07-05: translation invented wrong value (verified vs psdk headers)
const IOCTL_AACS_READ_BINDING_NONCE: u32 = 0x003350EC; // FIXED 2026-07-05: translation invented wrong value (verified vs psdk headers)
const IOCTL_AACS_GENERATE_BINDING_NONCE: u32 = 0x0033D0F0; // FIXED 2026-07-05: translation invented wrong value (verified vs psdk headers)
const IOCTL_CDROM_ENABLE_STREAMING: u32 = 0x24068;
const IOCTL_CDROM_SEND_OPC_INFORMATION: u32 = 0x2C06C;
const IOCTL_CDROM_GET_PERFORMANCE: u32 = 0x24070;
const IOCTL_MCN_SYNC_FAKE_IOCTL: u32 = 0x0007C0C8; // FIXED 2026-07-05: = IOCTL_DISK_UPDATE_DRIVE_SIZE (cdrom.h); translation invented wrong value
const IOCTL_STORAGE_MEDIA_REMOVAL: u32 = 0x2D4804;
const IOCTL_STORAGE_EJECTION_CONTROL: u32 = 0x2D0940;
const IOCTL_STORAGE_EJECT_MEDIA: u32 = 0x2D4808;
const IOCTL_STORAGE_LOAD_MEDIA: u32 = 0x2D480C;
const IOCTL_STORAGE_LOAD_MEDIA2: u32 = 0x2D080C;
const IOCTL_STORAGE_MCN_CONTROL: u32 = 0x2D0944;
const IOCTL_STORAGE_RESERVE: u32 = 0x2D4810;
const IOCTL_STORAGE_RELEASE: u32 = 0x2D4814;
const IOCTL_STORAGE_FIND_NEW_DEVICES: u32 = 0x2D4818;
const IOCTL_STORAGE_PERSISTENT_RESERVE_IN: u32 = 0x2D5018;
const IOCTL_STORAGE_PERSISTENT_RESERVE_OUT: u32 = 0x2DD01C;
const IOCTL_STORAGE_CHECK_PRIORITY_HINT_SUPPORT: u32 = 0x2D1880;
const IOCTL_DISK_ARE_VOLUMES_READY: u32 = 0x7421C; // CTL_CODE(DISK,0x87,0,READ)
const IOCTL_VOLUME_ONLINE: u32 = 0x56C008;          // CTL_CODE(VOLUME,2,0,ANY)
const IOCTL_VOLUME_POST_ONLINE: u32 = 0x56C064;     // CTL_CODE(VOLUME,25,0,ANY)
const FeatureDefectManagement: u32 = 0x10;

const IO_CD_ROM_INCREMENT: i32 = 1;
// CdromMmcUpdateRequired is assumed to be defined elsewhere; keep as-is.

// Forward declarations for unknown functions and constants used in this function
// Local function (same translation unit) – will be linked later

// Forward declarations for external functions and constants used by this function.
// These will be deduplicated by the caller.

// Constants
const DO_VERIFY_VOLUME: u32 = 0x00000002;           // DO_VERIFY_VOLUME
const SL_OVERRIDE_VERIFY_VOLUME: u8 = 0x02;         // SL_OVERRIDE_VERIFY_VOLUME (Flags field)
const CDROM_VOLUME_VERIFY_CHECKED: u8 = 0x34;       // CDROM_VOLUME_VERIFY_CHECKED (MinorFunction)  // was 0x01 (LLM-invented); bindgen-correct
const WdfRequestTypeRead: u32 = 3;                  // WDF_REQUEST_TYPE WdfRequestTypeRead
const WdfRequestTypeWrite: u32 = 4;                 // WDF_REQUEST_TYPE WdfRequestTypeWrite
const TRACE_LEVEL_VERBOSE: u32 = 5; // storswtr.h:26-30
const TRACE_FLAG_GENERAL: u32 = 0; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)
const STATUS_VERIFY_REQUIRED: i32 = 0x80000016_u32 as i32; // STATUS_VERIFY_REQUIRED
const STATUS_DEVICE_BUSY: i32 = 0x80000011_u32 as i32;     // STATUS_DEVICE_BUSY
const STATUS_ACCESS_DENIED: i32 = 0xC0000022u32 as i32;   // STATUS_ACCESS_DENIED
const STATUS_SUCCESS: i32 = 0;                              // STATUS_SUCCESS
const CdromMmcUpdateRequired: u32 = 0x1;                      // enum value  // was 2 (LLM-invented); bindgen-correct

// External functions

// Forward declarations of unknown externs (caller will dedup these)

const FILE_DEVICE_ACPI: u32 = 0x32;

// Forward declarations for unknown functions from the driver

// Placeholder constants for missing defines
const IOCTL_SCSI_PASS_THROUGH: u32 = 0x0004D004; // actual value from driver headers
const IOCTL_SCSI_PASS_THROUGH_DIRECT: u32 = 0x0004D014;
const IOCTL_SCSI_PASS_THROUGH_EX: u32 = 0x0004D044;
const IOCTL_SCSI_PASS_THROUGH_DIRECT_EX: u32 = 0x0004D048;
const IOCTL_STORAGE_QUERY_PROPERTY: u32 = 0x002D1400;
const STATUS_IO_DEVICE_ERROR: i32 = 0xC0000185_u32 as i32;
// The MCN sync IRP marks itself with the ADDRESS of RequestSetupMcnSyncIrp in
// Arg1/Arg2/Arg4 (autorun.rs stores it; cdrom.c:3037 compares against the function).
// The translation had replaced this with `const … = 0`, so the self-check below never
// matched, the driver's own sync IRP was dispatched as an UNKNOWN ioctl and forwarded
// to the port driver once per timer tick: scsiport logged
// `unknown ioctl code: 0x7C0C8` ~700 times per hung amd64-bios boot, media-change
// detection never ran, and the chatter kept the smoke harness from recognising a
// wedged guest (the "serial still flowing" rule). Compare against the real function.
#[inline(always)]
unsafe fn RequestSetupMcnSyncIrpAddr() -> usize {
    crate::autorun::RequestSetupMcnSyncIrp as *const () as usize
}

const TRACE_LEVEL_ERROR: u32 = 2; // storswtr.h:26-30
const TRACE_FLAG_IOCTL: u32 = 4; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)
const IOCTL_STORAGE_EVENT_NOTIFICATION: u32 = 0x2D1C90;
const WdfRequestTypeDeviceControl: u32 = 14; // WDF_REQUEST_TYPE (0xe)
const WdfRequestTypeDeviceControlInternal: u32 = 15; // WDF_REQUEST_TYPE (0xf)
#[no_mangle]
pub unsafe extern "system" fn DriverEntry(
    DriverObject: ntkrnl_sys::ddk::PDRIVER_OBJECT,
    RegistryPath: ntkrnl_sys::ddk::PUNICODE_STRING,
) -> ntkrnl_sys::base::NTSTATUS {
    let mut status: ntkrnl_sys::base::NTSTATUS;
    let mut config: WDF_DRIVER_CONFIG = core::mem::zeroed();
    let mut attributes: WDF_OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut driverObject: WDFDRIVER = core::ptr::null_mut();

    PagedCode();
    ntkrnl_sys::WPP_INIT_TRACING!(DriverObject, RegistryPath);

    {
        let level = TRACE_LEVEL_INFORMATION;
        let flags = TRACE_FLAG_INIT;
        let fmt = b"CDROM.SYS DriverObject %p loading\n\0".as_ptr() as *const i8;
        TracePrint!(level, flags, fmt, DriverObject as usize);
    }

    WDF_OBJECT_ATTRIBUTES_INIT_CONTEXT_TYPE(
        &raw mut attributes,
        &CDROM_DRIVER_EXTENSION_TYPE_INFO,
    );
    // `extern "system"` is stdcall-ABI on i386 / C-ABI on amd64+arm64 — the same
    // ABI the WDF callback field expects on each arch — but rustc nominally
    // distinguishes it from the field's `"stdcall"`/`"C"` fn-ptr type on i386.
    // Transmute to the inferred field type (zero-cost, ABI-identical).
    (*core::ptr::addr_of_mut!(attributes)).EvtCleanupCallback =
        Some(core::mem::transmute(DriverEvtCleanup as unsafe extern "system" fn(_)));

    WDF_DRIVER_CONFIG_INIT(
        &raw mut config,
        Some(core::mem::transmute(DriverEvtDeviceAdd as unsafe extern "system" fn(_, _) -> _)),
    );

    status = WdfDriverCreate(
        DriverObject,
        RegistryPath,
        &raw mut attributes,
        &raw mut config,
        &raw mut driverObject,
    );

    if !ntkrnl_sys::base::NT_SUCCESS(status) {
        {
            let level = TRACE_LEVEL_FATAL;
            let flags = TRACE_FLAG_INIT;
            let fmt = b"WdfDriverCreate failed. %x\n\0".as_ptr() as *const i8;
            TracePrint!(level, flags, fmt, status as usize);
            ntkrnl_sys::WPP_CLEANUP!(DriverObject);
        }
    } else {
        let driverExtension: *mut CDROM_DRIVER_EXTENSION = DriverGetExtension(driverObject);
        (*driverExtension).Version = 0x01;
        (*driverExtension).DriverObject = DriverObject;
        if BootEnvironmentIsWinPE() != 0 {
            (*driverExtension).Flags |= CDROM_FLAG_WINPE_MODE;
        }
    }

    status
}

// Forward declarations for unknown symbols – deduplicated by the caller.

extern "C" {
    // Constant wide string; replace with actual registry path.
    static WINPE_REG_KEY_NAME: *const u16;
}

#[no_mangle]
pub unsafe extern "system" fn DriverEvtCleanup(Driver: WDFOBJECT) {
    let driver: WDFDRIVER = Driver as WDFDRIVER;

    ntkrnl_sys::ASSERT!(KeGetCurrentIrql() == PASSIVE_LEVEL);

    TracePrint!(
        TRACE_LEVEL_INFORMATION,
        TRACE_FLAG_INIT,
        b"CDROM.SYS DriverObject %p cleanup. Will be unloaded soon\n\0".as_ptr() as *const i8,
        driver,
    );

    ntkrnl_sys::WPP_CLEANUP!(WdfDriverWdmGetDriverObject(driver));

    return;
}

#[inline(never)]
unsafe fn DeviceFinishBasicInitialization(deviceExtension: *mut CDROM_DEVICE_EXTENSION) {
    if (*(*(deviceExtension)).LowerPdo).AlignmentRequirement
        > (*(*(deviceExtension)).DeviceObject).AlignmentRequirement
    {
        (*(*(deviceExtension)).DeviceObject).AlignmentRequirement =
            (*(*(deviceExtension)).LowerPdo).AlignmentRequirement;
    }

    (*(deviceExtension)).StartingOffset.__bindgen_anon_1.LowPart = 0;
    (*(deviceExtension)).StartingOffset.__bindgen_anon_1.HighPart = 0;
    (*(deviceExtension)).DiskGeometry.MediaType = 0;
    (*(deviceExtension)).DiskGeometry.TracksPerCylinder = 0x40;
    (*(deviceExtension)).DiskGeometry.SectorsPerTrack = 0x20;
    (*(deviceExtension)).DeviceAdditionalData.ReadWriteRetryDelay100nsUnits =
        WRITE_RETRY_DELAY_DVD_1x as i64;
    (*(deviceExtension)).SrbFlags = SRB_FLAGS_DISABLE_SYNCH_TRANSFER;
    (*(deviceExtension)).TimeOutValue = DeviceGetTimeOutValueFromRegistry();
    if (*(deviceExtension)).TimeOutValue > 1800 || (*(deviceExtension)).TimeOutValue == 0 {
        (*(deviceExtension)).TimeOutValue = SCSI_CDROM_TIMEOUT;
    }
}

#[no_mangle]
pub unsafe extern "system" fn DriverEvtDeviceAdd(
    Driver: WDFDRIVER,
    mut DeviceInit: *mut WDFDEVICE_INIT,
) -> NTSTATUS {
    let mut status: NTSTATUS; // STATUS_SUCCESS
    let mut driverExtension: *mut CDROM_DRIVER_EXTENSION;
    let mut device: WDFDEVICE = core::ptr::null_mut();
    let mut deviceExtension: *mut CDROM_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut deviceClaimed: i32 = 0; // FALSE

    let mut attributes: WDF_OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut fileObjectConfig: WDF_FILEOBJECT_CONFIG = core::mem::zeroed();
    let mut ioTargetOpenParams: WDF_IO_TARGET_OPEN_PARAMS = core::mem::zeroed();
    let mut queueConfig: WDF_IO_QUEUE_CONFIG = core::mem::zeroed();
    let mut pnpPowerCallbacks: WDF_PNPPOWER_EVENT_CALLBACKS = core::mem::zeroed();
    let mut removeLockOptions: WDF_REMOVE_LOCK_OPTIONS = core::mem::zeroed();
    let mut wideDeviceName: *mut u16 = core::ptr::null_mut();
    let mut unicodeDeviceName: _UNICODE_STRING = core::mem::zeroed();
    let mut lowerPdo: *mut DEVICE_OBJECT;
    let mut deviceNumber: u32 = 0;
    let mut devicePropertySessionId: u32 = INVALID_SESSION;
    let mut devicePropertySize: u32 = 0;
    let mut devicePropertyType: u32 = DEVPROP_TYPE_EMPTY;

    PAGED_CODE!();

    driverExtension = DriverGetExtension(Driver);

    RtlInitUnicodeString(&mut unicodeDeviceName as *mut _UNICODE_STRING, core::ptr::null());

    'Exit: loop {
        // 1. PnP & Power callbacks
        {
            WDF_PNPPOWER_EVENT_CALLBACKS_INIT(&mut pnpPowerCallbacks);
            pnpPowerCallbacks.EvtDeviceSelfManagedIoInit = core::mem::transmute::<unsafe extern "system" fn(WDFDEVICE) -> NTSTATUS, _>(DeviceEvtSelfManagedIoInit);
            pnpPowerCallbacks.EvtDeviceD0Entry = core::mem::transmute::<unsafe extern "system" fn(WDFDEVICE, u32) -> NTSTATUS, _>(DeviceEvtD0Entry);
            pnpPowerCallbacks.EvtDeviceD0Exit = core::mem::transmute::<unsafe extern "system" fn(WDFDEVICE, u32) -> NTSTATUS, _>(DeviceEvtD0Exit);
            pnpPowerCallbacks.EvtDeviceSelfManagedIoCleanup = core::mem::transmute::<unsafe extern "system" fn(*mut core::ffi::c_void), _>(DeviceEvtSelfManagedIoCleanup);
            pnpPowerCallbacks.EvtDeviceSurpriseRemoval = core::mem::transmute::<unsafe extern "system" fn(WDFDEVICE), _>(DeviceEvtSurpriseRemoval);
            WdfDeviceInitSetPnpPowerEventCallbacks(DeviceInit, &mut pnpPowerCallbacks);
        }

        // 2. IoInCallerContext
        WdfDeviceInitSetIoInCallerContextCallback(
            DeviceInit,
            core::mem::transmute::<unsafe extern "system" fn(WDFDEVICE, WDFREQUEST), _>(DeviceEvtIoInCallerContext),
        );

        // 3. Preprocess callbacks
        {
            let mut minorFunctions: [u8; 1] = [0u8; 1];
            minorFunctions[0] = 0x02; // IRP_MN_SET_POWER (wdm.h)
            status = WdfDeviceInitAssignWdmIrpPreprocessCallback(
                DeviceInit,
                core::mem::transmute::<unsafe extern "system" fn(WDFDEVICE, PIRP) -> NTSTATUS, _>(RequestProcessSetPower),
                0x16, // IRP_MJ_POWER
                minorFunctions.as_mut_ptr(),
                (core::mem::size_of_val(&minorFunctions) / core::mem::size_of::<u8>()) as u32,
            );
            if !NT_SUCCESS(status) {
                TracePrint!(
                    TRACE_LEVEL_FATAL,
                    TRACE_FLAG_PNP,
                    b"DriverEvtDeviceAdd: Assign IrpPreprocessCallback for IRP_MJ_POWER failed, status: 0x%X\n\0".as_ptr() as *const i8,
                    status,
                );
                break 'Exit;
            }

            status = WdfDeviceInitAssignWdmIrpPreprocessCallback(
                DeviceInit,
                core::mem::transmute::<unsafe extern "system" fn(WDFDEVICE, PIRP) -> NTSTATUS, _>(RequestProcessShutdownFlush),
                0x09, // IRP_MJ_FLUSH_BUFFERS
                core::ptr::null_mut(),
                0,
            );
            if !NT_SUCCESS(status) {
                TracePrint!(
                    TRACE_LEVEL_FATAL,
                    TRACE_FLAG_PNP,
                    b"DriverEvtDeviceAdd: Assign IrpPreprocessCallback for IRP_MJ_FLUSH_BUFFERS failed, status: 0x%X\n\0".as_ptr() as *const i8,
                    status,
                );
                break 'Exit;
            }

            status = WdfDeviceInitAssignWdmIrpPreprocessCallback(
                DeviceInit,
                core::mem::transmute::<unsafe extern "system" fn(WDFDEVICE, PIRP) -> NTSTATUS, _>(RequestProcessShutdownFlush),
                0x10, // IRP_MJ_SHUTDOWN
                core::ptr::null_mut(),
                0,
            );
            if !NT_SUCCESS(status) {
                TracePrint!(
                    TRACE_LEVEL_FATAL,
                    TRACE_FLAG_PNP,
                    b"DriverEvtDeviceAdd: Assign IrpPreprocessCallback for IRP_MJ_SHUTDOWN failed, status: 0x%X\n\0".as_ptr() as *const i8,
                    status,
                );
                break 'Exit;
            }
        }

        // 4. Request context attributes
        {
            WDF_OBJECT_ATTRIBUTES_INIT_CONTEXT_TYPE(
                &mut attributes,
                &CDROM_REQUEST_CONTEXT_TYPE_INFO,
            );
            attributes.EvtCleanupCallback = core::mem::transmute::<unsafe extern "system" fn(*mut core::ffi::c_void), _>(RequestEvtCleanup);
            WdfDeviceInitSetRequestAttributes(DeviceInit, &mut attributes);
        }

        // 5. File object callbacks
        {
            WDF_OBJECT_ATTRIBUTES_INIT_CONTEXT_TYPE(
                &mut attributes,
                &FILE_OBJECT_CONTEXT_TYPE_INFO,
            );
            WDF_FILEOBJECT_CONFIG_INIT(
                &mut fileObjectConfig,
                core::mem::transmute(0usize), // NULL create callback
                core::mem::transmute::<unsafe extern "system" fn(WDFFILEOBJECT) -> (), _>(DeviceEvtFileClose),
                core::mem::transmute(0usize), // WDF_NO_EVENT_CALLBACK cleanup
            );
            fileObjectConfig.FileObjectClass = WdfFileObjectWdfCannotUseFsContexts;
            fileObjectConfig.AutoForwardCleanupClose = WdfTrue;
            attributes.SynchronizationScope = WdfSynchronizationScopeNone;
            attributes.ExecutionLevel = WdfExecutionLevelPassive;
            fileObjectConfig.FileObjectClass |= WdfFileObjectCanBeOptional;
            WdfDeviceInitSetFileObjectConfig(DeviceInit, &mut fileObjectConfig, &mut attributes);
        }

        // 6. Device‑wide attributes
        {
            WdfDeviceInitSetPowerPolicyOwnership(DeviceInit, 0);
            WdfDeviceInitSetExclusive(DeviceInit, 0);
            WdfDeviceInitSetDeviceType(DeviceInit, FILE_DEVICE_CD_ROM);
            WdfDeviceInitSetCharacteristics(DeviceInit, FILE_REMOVABLE_MEDIA, 0);
            WdfDeviceInitSetIoType(DeviceInit, WdfDeviceIoDirect);
            WdfDeviceInitSetPowerPageable(DeviceInit);
            WDF_REMOVE_LOCK_OPTIONS_INIT(&mut removeLockOptions, WDF_REMOVE_LOCK_OPTION_ACQUIRE_FOR_IO);
            WdfDeviceInitSetRemoveLockOptions(DeviceInit, &mut removeLockOptions);
            lowerPdo = WdfFdoInitWdmGetPhysicalDevice(DeviceInit);
            WDF_OBJECT_ATTRIBUTES_INIT_CONTEXT_TYPE(
                &mut attributes,
                &CDROM_DEVICE_EXTENSION_TYPE_INFO,
            );
            attributes.SynchronizationScope = WdfSynchronizationScopeNone;
            attributes.ExecutionLevel = WdfExecutionLevelDispatch;
            attributes.EvtCleanupCallback = core::mem::transmute::<unsafe extern "system" fn(*mut core::ffi::c_void), _>(DeviceEvtCleanup);
        }

        // 7. Device creation
        {
            wideDeviceName = ExAllocatePoolWithTag(
                ntkrnl_sys::ddk::_POOL_TYPE_NonPagedPool as _, // cdrom.h: #define NonPagedPoolNx NonPagedPool (was 512)
                64 * core::mem::size_of::<u16>(),
                CDROM_TAG_STRINGS,
            ) as *mut u16;
            if wideDeviceName.is_null() {
                status = (0xC000009Au32 as i32); // STATUS_INSUFFICIENT_RESOURCES
                break 'Exit;
            }

            loop {
                status = crate::write_wide_cdrom_name(
                    wideDeviceName,
                    64,
                    b"\\Device\\CdRom",
                    deviceNumber as i32,
                );
                if !NT_SUCCESS(status) {
                    TracePrint!(
                        TRACE_LEVEL_FATAL,
                        TRACE_FLAG_PNP,
                        b"DriverEvtDeviceAdd: Format device name failed with error: 0x%X\n\0".as_ptr() as *const i8,
                        status,
                    );
                    break 'Exit;
                }
                RtlInitUnicodeString(&mut unicodeDeviceName as *mut _UNICODE_STRING, wideDeviceName);
                status = WdfDeviceInitAssignName(DeviceInit, &mut unicodeDeviceName);
                if !NT_SUCCESS(status) {
                    TracePrint!(
                        TRACE_LEVEL_FATAL,
                        TRACE_FLAG_PNP,
                        b"DriverEvtDeviceAdd: WdfDeviceInitAssignName() failed with error: 0x%X\n\0".as_ptr() as *const i8,
                        status,
                    );
                    break 'Exit;
                }
                status = WdfDeviceCreate(
                    core::ptr::addr_of_mut!(DeviceInit),
                    &mut attributes,
                    &mut device,
                );
                deviceNumber += 1;
                if status != (0xC0000035u32 as i32) /* STATUS_OBJECT_NAME_COLLISION */ {
                    break;
                }
            }
            deviceNumber -= 1;

            if !NT_SUCCESS(status) {
                TracePrint!(
                    TRACE_LEVEL_FATAL,
                    TRACE_FLAG_PNP,
                    b"DriverEvtDeviceAdd: Can not create a new device, status: 0x%X\n\0".as_ptr() as *const i8,
                    status,
                );
                break 'Exit;
            }
        }

        // 8. Device extension basic settings & remote I/O target
        {
            deviceExtension = DeviceGetExtension(device);
            (*(deviceExtension)).Version = 0x01;
            (*(deviceExtension)).Size = core::mem::size_of::<CDROM_DEVICE_EXTENSION>() as u32;
            (*(deviceExtension)).DeviceObject = WdfDeviceWdmGetDeviceObject(device);
            (*(deviceExtension)).Device = device;
            (*(deviceExtension)).DriverExtension = driverExtension;
            (*(deviceExtension)).LowerPdo = lowerPdo;
            (*(deviceExtension)).DeviceType = FILE_DEVICE_CD_ROM;
            (*(deviceExtension)).DeviceName = unicodeDeviceName;
            (*(deviceExtension)).DeviceNumber = deviceNumber;
        }
        {
            WDF_OBJECT_ATTRIBUTES_INIT(&mut attributes);
            attributes.ParentObject = (*(deviceExtension)).Device as *mut core::ffi::c_void;
            status = WdfIoTargetCreate(
                (*(deviceExtension)).Device,
                &mut attributes,
                &mut (*(deviceExtension)).IoTarget,
            );
            CdromDbgPrint3(
                b"[CDS] 08 target create st=%x ext=%p io=%p\n\0".as_ptr() as *const i8,
                status as usize,
                deviceExtension as usize,
                (*(deviceExtension)).IoTarget as usize,
            );
            if !NT_SUCCESS(status) {
                TracePrint!(
                    TRACE_LEVEL_FATAL,
                    TRACE_FLAG_PNP,
                    b"DriverEvtDeviceAdd: Can not create a remote I/O target object, status: 0x%X\n\0".as_ptr() as *const i8,
                    status,
                );
                break 'Exit;
            }
            WDF_IO_TARGET_OPEN_PARAMS_INIT_EXISTING_DEVICE(
                &mut ioTargetOpenParams,
                WdfDeviceWdmGetAttachedDevice((*(deviceExtension)).Device),
            );
            status = WdfIoTargetOpen((*(deviceExtension)).IoTarget, &mut ioTargetOpenParams);
            CdromDbgPrint3(
                b"[CDS] 08 target open st=%x ext=%p io=%p\n\0".as_ptr() as *const i8,
                status as usize,
                deviceExtension as usize,
                (*(deviceExtension)).IoTarget as usize,
            );
            if !NT_SUCCESS(status) {
                TracePrint!(
                    TRACE_LEVEL_FATAL,
                    TRACE_FLAG_PNP,
                    b"DriverEvtDeviceAdd: Can not open a remote I/O target for the next-lower device, status: 0x%X\n\0".as_ptr() as *const i8,
                    status,
                );
                WdfObjectDelete((*(deviceExtension)).IoTarget as *mut core::ffi::c_void);
                (*(deviceExtension)).IoTarget = core::ptr::null_mut();
                break 'Exit;
            }
        }

        // 9. Claim the device
        status = DeviceClaimRelease(deviceExtension, 0);
        if !NT_SUCCESS(status) {
            break 'Exit;
        } else {
            deviceClaimed = 1;
        }

        CdromDbgPrint1(b"[CDS] 09 claim st=%x\n\0".as_ptr() as *const i8, status as usize);
        // 10. Set up I/O queues
        {
            WDF_IO_QUEUE_CONFIG_INIT_DEFAULT_QUEUE(&mut queueConfig, WdfIoQueueDispatchSequential);
            queueConfig.PowerManaged = WdfFalse;
            queueConfig.EvtIoRead = core::mem::transmute::<unsafe extern "system" fn(WDFQUEUE, WDFREQUEST, usize), _>(SequentialQueueEvtIoReadWrite); // Option A: async Rust read path
            queueConfig.EvtIoWrite = core::mem::transmute::<unsafe extern "system" fn(WDFQUEUE, WDFREQUEST, usize), _>(SequentialQueueEvtIoReadWrite);
            queueConfig.EvtIoDeviceControl = core::mem::transmute::<unsafe extern "system" fn(WDFQUEUE, WDFREQUEST, usize, usize, u32), _>(SequentialQueueEvtIoDeviceControl);
            queueConfig.EvtIoCanceledOnQueue = core::mem::transmute::<unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void), _>(SequentialQueueEvtCanceledOnQueue);
            status = WdfIoQueueCreate(
                device,
                &mut queueConfig,
                core::ptr::null_mut(), // WDF_NO_OBJECT_ATTRIBUTES
                &mut (*(deviceExtension)).SerialIOQueue,
            );
            if !NT_SUCCESS(status) {
                break 'Exit;
            }

            WDF_IO_QUEUE_CONFIG_INIT(&mut queueConfig, WdfIoQueueDispatchParallel);
            queueConfig.PowerManaged = WdfFalse;
            queueConfig.EvtIoDefault = core::mem::transmute::<unsafe extern "system" fn(WDFQUEUE, WDFREQUEST), _>(CreateQueueEvtIoDefault);

            WDF_OBJECT_ATTRIBUTES_INIT_CONTEXT_TYPE(
                &mut attributes,
                &CDROM_REQUEST_CONTEXT_TYPE_INFO,
            );
            attributes.SynchronizationScope = WdfSynchronizationScopeNone;
            attributes.ExecutionLevel = WdfExecutionLevelPassive;

            status = WdfIoQueueCreate(
                device,
                &mut queueConfig,
                &mut attributes,
                &mut (*(deviceExtension)).CreateQueue,
            );
            if !NT_SUCCESS(status) {
                break 'Exit;
            }

            status = WdfDeviceConfigureRequestDispatching(
                device,
                (*(deviceExtension)).CreateQueue,
                WdfRequestTypeCreate,
            );
            if !NT_SUCCESS(status) {
                break 'Exit;
            }
        }

        CdromDbgPrint1(b"[CDS] 10 queues st=%x\n\0".as_ptr() as *const i8, status as usize);
        DeviceFinishBasicInitialization(deviceExtension);
        // (NTDDI_VERSION >= NTDDI_WIN8) assumed true
        {
            (*(deviceExtension)).IsVolumeOnlinePending = 1;
            WDF_IO_QUEUE_CONFIG_INIT(&mut queueConfig, WdfIoQueueDispatchManual);
            queueConfig.PowerManaged = WdfFalse;
            status = WdfIoQueueCreate(
                device,
                &mut queueConfig,
                core::ptr::null_mut(),
                &mut (*(deviceExtension)).ManualVolumeReadyQueue,
            );
            if !NT_SUCCESS(status) {
                break 'Exit;
            }
        }

        CdromDbgPrint1(b"[CDS] 12 manualq st=%x\n\0".as_ptr() as *const i8, status as usize);
        // 13. Media locking
        WDF_OBJECT_ATTRIBUTES_INIT(&mut attributes);
        attributes.ParentObject = (*(deviceExtension)).Device as *mut core::ffi::c_void;
        status = WdfWaitLockCreate(
            &mut attributes,
            &mut (*(deviceExtension)).EjectSynchronizationLock,
        );
        (*(deviceExtension)).LockCount = 0;

        CdromDbgPrint1(b"[CDS] 13 waitlock st=%x\n\0".as_ptr() as *const i8, status as usize);
        // 14. Asynchronous contexts
        if NT_SUCCESS(status) {
            status = DeviceInitReleaseQueueContext(deviceExtension);
        }
        if NT_SUCCESS(status) {
            status = DeviceInitPowerContext(deviceExtension);
        }

        CdromDbgPrint1(b"[CDS] 14 asyncctx st=%x\n\0".as_ptr() as *const i8, status as usize);
        // 15. External access points
        if NT_SUCCESS(status) {
            status = DeviceCreateWellKnownName(deviceExtension);
        }

        CdromDbgPrint1(b"[CDS] 15 wellknown st=%x\n\0".as_ptr() as *const i8, status as usize);
        // 16. Query session ID
        if NT_SUCCESS(status) {
            status = IoGetDevicePropertyData(
                (*(deviceExtension)).LowerPdo,
                &DEVPKEY_Device_SessionId as *const _ as *const c_void,
                0,
                0,
                core::mem::size_of_val(&devicePropertySessionId) as u32,
                &mut devicePropertySessionId as *mut _ as *mut c_void,
                &mut devicePropertySize,
                &mut devicePropertyType,
            );
            if !NT_SUCCESS(status) {
                devicePropertySessionId = INVALID_SESSION;
                status = 0;
            }
        }

        // 17. Register interfaces
        if NT_SUCCESS(status) {
            status = DeviceRegisterInterface(deviceExtension, CdRomDeviceInterface);
        }
        if NT_SUCCESS(status) {
            if devicePropertySessionId == INVALID_SESSION {
                status = DeviceRegisterInterface(deviceExtension, MountedDeviceInterface);
            }
        }

        CdromDbgPrint1(b"[CDS] 17 regiface st=%x\n\0".as_ptr() as *const i8, status as usize);
        // 18. Shutdown/flush lock
        if NT_SUCCESS(status) {
            WDF_OBJECT_ATTRIBUTES_INIT(&mut attributes);
            attributes.ParentObject = (*(deviceExtension)).Device as *mut core::ffi::c_void;
            status = WdfWaitLockCreate(
                &mut attributes,
                &mut (*(deviceExtension)).ShutdownFlushWaitLock,
            );
            if !NT_SUCCESS(status) {
                TracePrint!(
                    TRACE_LEVEL_FATAL,
                    TRACE_FLAG_PNP,
                    b"DriverEvtDeviceAdd: Cannot create shutdown/flush waitlock, status: 0x%X\n\0".as_ptr() as *const i8,
                    status,
                );
            }
        }

        // 19. Read/write work item
        if NT_SUCCESS(status) {
            let mut workItemConfig: WDF_WORKITEM_CONFIG = core::mem::zeroed();
            let mut workItemAttributes: WDF_OBJECT_ATTRIBUTES = core::mem::zeroed();
            WDF_WORKITEM_CONFIG_INIT(
                &mut workItemConfig,
                core::mem::transmute::<unsafe extern "system" fn(WDFWORKITEM), _>(ReadWriteWorkItemRoutine),
            );
            WDF_OBJECT_ATTRIBUTES_INIT(&mut workItemAttributes);
            workItemAttributes.ParentObject = (*(deviceExtension)).Device as *mut core::ffi::c_void;
            status = WdfWorkItemCreate(
                &mut workItemConfig,
                &mut workItemAttributes,
                &mut (*(deviceExtension)).ReadWriteWorkItem,
            );
            if !NT_SUCCESS(status) {
                TracePrint!(
                    TRACE_LEVEL_FATAL,
                    TRACE_FLAG_INIT,
                    b"DriverEvtDeviceAdd: Cannot create read/write work item, status: 0x%X\n\0".as_ptr() as *const i8,
                    status,
                );
            }
        }

        CdromDbgPrint1(b"[CDS] 19 rwworkitem st=%x\n\0".as_ptr() as *const i8, status as usize);
        // 20. IOCTL work item
        if NT_SUCCESS(status) {
            let mut workItemConfig: WDF_WORKITEM_CONFIG = core::mem::zeroed();
            let mut workItemAttributes: WDF_OBJECT_ATTRIBUTES = core::mem::zeroed();
            WDF_WORKITEM_CONFIG_INIT(
                &mut workItemConfig,
                core::mem::transmute::<unsafe extern "system" fn(WDFWORKITEM), _>(IoctlWorkItemRoutine),
            );
            WDF_OBJECT_ATTRIBUTES_INIT(&mut workItemAttributes);
            workItemAttributes.ParentObject = (*(deviceExtension)).Device as *mut core::ffi::c_void;
            status = WdfWorkItemCreate(
                &mut workItemConfig,
                &mut workItemAttributes,
                &mut (*(deviceExtension)).IoctlWorkItem,
            );
            if !NT_SUCCESS(status) {
                TracePrint!(
                    TRACE_LEVEL_FATAL,
                    TRACE_FLAG_INIT,
                    b"DriverEvtDeviceAdd: Cannot create ioctl work item, status: 0x%X\n\0".as_ptr() as *const i8,
                    status,
                );
            }
        }

        CdromDbgPrint1(b"[CDS] 20 ioctlworkitem st=%x\n\0".as_ptr() as *const i8, status as usize);
        // Fall‑through to cleanup
        break 'Exit;
    }

    // Cleanup
    if !NT_SUCCESS(status) {
        if !wideDeviceName.is_null() {
            FREE_POOL(wideDeviceName as *mut core::ffi::c_void);
        }

        if !deviceExtension.is_null() {
            RtlInitUnicodeString(
                &mut (*(deviceExtension)).DeviceName as *mut _UNICODE_STRING,
                core::ptr::null(),
            );
        }

        if !deviceExtension.is_null() && deviceClaimed != 0 {
            DeviceClaimRelease(deviceExtension, 1);
        }
    }

    status
}
#[no_mangle]
pub unsafe extern "system" fn DeviceEvtCleanup(Device: WDFOBJECT) {
    // PAGED_CODE();
    PAGED_CODE!();

    let device = Device as *mut core::ffi::c_void;
    let format = b"WDFDEVICE %p cleanup: The device is about to be destroyed.\n\0".as_ptr() as *const i8;
    TracePrint!(0, 0, format, device); // TRACE_LEVEL_INFORMATION, TRACE_FLAG_INIT

    let deviceExtension: *mut CDROM_DEVICE_EXTENSION = core::ptr::null_mut();
    let deviceExtension = DeviceGetExtension(device as WDFDEVICE);

    FREE_POOL((*deviceExtension).DeviceName.Buffer as *mut core::ffi::c_void);
    RtlInitUnicodeString(
        core::ptr::addr_of_mut!((*deviceExtension).DeviceName),
        core::ptr::null::<u16>(),
    );

    if !((*deviceExtension).DeviceAdditionalData.WellKnownName.Buffer).is_null() {
        IoDeleteSymbolicLink(core::ptr::addr_of_mut!(
            (*deviceExtension).DeviceAdditionalData.WellKnownName
        ));
    }

    FREE_POOL((*deviceExtension).DeviceAdditionalData.WellKnownName.Buffer as *mut core::ffi::c_void);
    RtlInitUnicodeString(
        core::ptr::addr_of_mut!((*deviceExtension).DeviceAdditionalData.WellKnownName),
        core::ptr::null::<u16>(),
    );
}
#[no_mangle]
pub unsafe extern "system" fn DeviceEvtSelfManagedIoCleanup(Device: *mut core::ffi::c_void) {
    // const values for TracePrint flags
    const TRACE_LEVEL_INFORMATION: u32 = 4; // storswtr.h:26-30
    const TRACE_FLAG_PNP: u32 = 1; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)

    let status: i32; // NTSTATUS mapped to i32
    let deviceExtension: *mut CDROM_DEVICE_EXTENSION; // NULL initially

    // PAGED_CODE(); remains as a call
    PAGED_CODE!();

    // TracePrint with varargs; we pass the format string and pointer
    TracePrint!(
        TRACE_LEVEL_INFORMATION,
        TRACE_FLAG_PNP,
        b"DeviceEvtSelfManagedIoCleanup: WDFDEVICE %p is being stopped.\n\0".as_ptr() as *const i8,
        Device,
    );

    // extract the device extension
    deviceExtension = DeviceGetExtension(Device as WDFDEVICE);

    // Purge queues; WDF_NO_EVENT_CALLBACK and WDF_NO_CONTEXT are both NULL.
    WdfIoQueuePurge(
        (*deviceExtension).SerialIOQueue,
        None,
        core::ptr::null_mut(),
    );
    WdfIoQueuePurge(
        (*deviceExtension).CreateQueue,
        None,
        core::ptr::null_mut(),
    );

    // Close IoTarget if present
    if !(*deviceExtension).IoTarget.is_null() {
        WdfIoTargetClose((*deviceExtension).IoTarget);
    }

    // Release device unless surprise removed
    if (*deviceExtension).SurpriseRemoved == 0 {
        status = DeviceClaimRelease(deviceExtension, 1 /* TRUE */);
        UNREFERENCED_PARAMETER((status) as usize); // defensive coding
    }

    // Flush queued DPCs
    KeFlushQueuedDpcs();

    // Release memory
    DeviceDeallocateMmcResources(Device);
    ScratchBuffer_Deallocate(deviceExtension);

    // RtlZeroMemory the Mmc extension
    RtlZeroMemory(
        addr_of_mut!((*deviceExtension).DeviceAdditionalData.Mmc) as *mut core::ffi::c_void,
        core::mem::size_of::<CDROM_MMC_EXTENSION>(),
    );

    // Free allocated descriptors and sense data
    FREE_POOL((*deviceExtension).DeviceDescriptor as *mut core::ffi::c_void);
    FREE_POOL((*deviceExtension).AdapterDescriptor as *mut core::ffi::c_void);
    FREE_POOL((*deviceExtension).PowerDescriptor as *mut core::ffi::c_void);
    FREE_POOL((*deviceExtension).SenseData as *mut core::ffi::c_void);

    // Cached inquiry data
    if !(*deviceExtension).DeviceAdditionalData.CachedInquiryData.is_null() {
        FREE_POOL((*deviceExtension).DeviceAdditionalData.CachedInquiryData as *mut core::ffi::c_void);
        (*deviceExtension).DeviceAdditionalData.CachedInquiryDataByteCount = 0;
    }

    FREE_POOL((*deviceExtension).PrivateFdoData as *mut core::ffi::c_void);

    // Release MCN and ZPODD resources
    DeviceReleaseMcnResources(deviceExtension);
    DeviceReleaseZPODDResources(deviceExtension);

    // Decrement system-wide CDROM count
    (*IoGetConfigurationInformation()).CdRomCount -= 1;

    // Zero partition length
    (*deviceExtension).PartitionLength.QuadPart = 0;

    // No explicit return needed for void
}
#[no_mangle]
pub unsafe extern "system" fn DeviceEvtD0Entry(
    Device: WDFDEVICE,
    PreviousState: WDF_POWER_DEVICE_STATE,
) -> NTSTATUS {
    let _ = PreviousState;
    CdromDbgPrint0(b"[D0E] enter\n\0".as_ptr() as *const i8);
    let deviceExtension = DeviceGetExtension(Device);
    CdromDbgPrint3(
        b"[D0E] ext=%p io=%p init=%x\n\0".as_ptr() as *const i8,
        deviceExtension as usize,
        (*deviceExtension).IoTarget as usize,
        (*deviceExtension).IsInitialized as usize,
    );
    let mut status: NTSTATUS;
    let mut zpoddInfo: PZERO_POWER_ODD_INFO;
    let mut powerupReason: STORAGE_IDLE_POWERUP_REASON = core::mem::zeroed();

    if (*deviceExtension).IsInitialized != 0 {
        zpoddInfo = (*deviceExtension).ZeroPowerODDInfo;
        if !zpoddInfo.is_null() {
            if (*zpoddInfo).InZeroPowerState != 0 {
                (*zpoddInfo).InZeroPowerState = 0;
                (*zpoddInfo).RetryFirstCommand = 1;
                (*zpoddInfo).BecomingReadyRetryCount = BECOMING_READY_RETRY_COUNT;

                status = DeviceZPODDGetPowerupReason(deviceExtension, &mut powerupReason);

                if ntkrnl_sys::base::NT_SUCCESS(status)
                    && (powerupReason.PowerupReason == StoragePowerupDeviceAttention as _)
                {
                    TracePrint!(
                        TRACE_LEVEL_INFORMATION,
                        TRACE_FLAG_POWER,
                        b"DeviceEvtD0Entry: Device has left zero power state due to eject button pressed\n\0"
                            .as_ptr() as *const i8,
                    );

                    if ((*zpoddInfo).LoadingMechanism as u32 == LOADING_MECHANISM_TRAY)
                        && ((*zpoddInfo).Load == 0)
                    {
                        DeviceSendIoctlAsynchronously(
                            deviceExtension,
                            IOCTL_STORAGE_EJECT_MEDIA,
                            (*deviceExtension).DeviceObject,
                        );
                    }
                } else {
                    TracePrint!(
                        TRACE_LEVEL_INFORMATION,
                        TRACE_FLAG_POWER,
                        b"DeviceEvtD0Entry: Device has left zero power state due to IO received\n\0"
                            .as_ptr() as *const i8,
                    );
                }
            }
        }
        DeviceEnableMainTimer(deviceExtension);
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn DeviceEvtD0Exit(Device: WDFDEVICE, TargetState: WDF_POWER_DEVICE_STATE) -> i32 {
    PagedCode();

    let mut status: i32;
    let mut deviceExtension: *mut CDROM_DEVICE_EXTENSION;
    let mut zpoddInfo: *mut ZERO_POWER_ODD_INFO;

    deviceExtension = DeviceGetExtension(Device);
    zpoddInfo = (*deviceExtension).ZeroPowerODDInfo;

    if (TargetState != WdfPowerDeviceD3) && (TargetState != WdfPowerDeviceD3Final) {
        return ntkrnl_sys::base::STATUS_SUCCESS;
    }

    DeviceDisableMainTimer(deviceExtension);

    if ((*deviceExtension).IsInitialized == 0) || ((*deviceExtension).SurpriseRemoved != 0) {
        return ntkrnl_sys::base::STATUS_SUCCESS;
    }

    {
        // DBG block with WINVER >= 0x0601 (ignored condition)
        let mut secondsRemaining: u32 = 0;
        let mut watchdogTimeSupported: i32 = 0; // BOOL
        PoQueryWatchdogTime((*deviceExtension).LowerPdo as *mut core::ffi::c_void, &mut secondsRemaining as *mut u32);
        UNREFERENCED_PARAMETER(0);
    }

    (*deviceExtension).PowerDownInProgress = 1; // TRUE

    status = PowerContextBeginUse(deviceExtension);

    (*deviceExtension).PowerContext.Options.set_PowerDown(1); // TRUE

    // Step 1. LOCK QUEUE
    if ntkrnl_sys::base::NT_SUCCESS(status) && (TargetState != WdfPowerDeviceD3Final) {
        status = DeviceSendPowerDownProcessRequest(deviceExtension, core::ptr::null_mut(), core::ptr::null_mut());

        if ntkrnl_sys::base::NT_SUCCESS(status) {
            (*deviceExtension).PowerContext.Options.set_LockQueue(1);
        }

        // Ignore failure.
        status = ntkrnl_sys::base::STATUS_SUCCESS;
    }

    (*deviceExtension).PowerContext.PowerChangeState.PowerDown += 1;

    // Step 2. QUIESCE QUEUE
    if ntkrnl_sys::base::NT_SUCCESS(status) && (TargetState != WdfPowerDeviceD3Final) {
        DeviceSendPowerDownProcessRequest(deviceExtension, core::ptr::null_mut(), core::ptr::null_mut());
        UNREFERENCED_PARAMETER(0);
        // We don't care about the status.
        status = ntkrnl_sys::base::STATUS_SUCCESS;
    }

    (*deviceExtension).PowerContext.PowerChangeState.PowerDown += 1;

    // Step 3. SYNC CACHE
    if ntkrnl_sys::base::NT_SUCCESS(status) && (*deviceExtension).DeviceAdditionalData.Mmc.WriteAllowed != 0 {
        DeviceSendPowerDownProcessRequest(deviceExtension, core::ptr::null_mut(), core::ptr::null_mut());
        UNREFERENCED_PARAMETER(0);
        status = ntkrnl_sys::base::STATUS_SUCCESS;
    }

    (*deviceExtension).PowerContext.PowerChangeState.PowerDown += 1;

    // Step 4. STOP UNIT
    if ntkrnl_sys::base::NT_SUCCESS(status) && (TEST_FLAG((*deviceExtension).ScanForSpecialFlags, CDROM_SPECIAL_DISABLE_SPIN_DOWN) == 0) {
        DeviceSendPowerDownProcessRequest(deviceExtension, core::ptr::null_mut(), core::ptr::null_mut());
        UNREFERENCED_PARAMETER(0);
    }

    if TargetState == WdfPowerDeviceD3Final {
        PowerContextEndUse(deviceExtension);
    }

    // Bumping the media change count forces volume verification on resume
    SET_FLAG(core::ptr::addr_of_mut!((*(*deviceExtension).DeviceObject).Flags) as *mut u32, DO_VERIFY_VOLUME);
    ntkrnl_sys::interlocked::InterlockedIncrement(core::ptr::addr_of_mut!((*deviceExtension).MediaChangeCount) as *mut i32);

    // Zero Power ODD handling
    if !zpoddInfo.is_null() {
        (*zpoddInfo).InZeroPowerState = 1; // TRUE

        TracePrint!(TRACE_LEVEL_INFORMATION, TRACE_FLAG_POWER,
                   b"Device has entered zero power state\n\0".as_ptr() as *const i8);
    }

    (*deviceExtension).PowerDownInProgress = 0; // FALSE

    ntkrnl_sys::base::STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "system" fn DeviceEvtSurpriseRemoval(Device: WDFDEVICE) {
    // PAGED_CODE();
    let mut deviceExtension: PCDROM_DEVICE_EXTENSION;
    deviceExtension = DeviceGetExtension(Device);

    (*deviceExtension).SurpriseRemoved = 1; // TRUE

    // Stop the main timer
    DeviceDisableMainTimer(deviceExtension);

    // legacy behavior to set partition length to be 0.
    (*deviceExtension).PartitionLength.QuadPart = 0;

    TracePrint!(
        TRACE_LEVEL_INFORMATION,
        TRACE_FLAG_INIT,
        b"Surprisely remove a WDFDEVICE %p\n\0".as_ptr() as *const i8,
        Device as usize,
    );
}
#[no_mangle]
pub unsafe extern "system" fn CreateQueueEvtIoDefault(
    Queue: WDFQUEUE,
    Request: WDFREQUEST,
) {
    CreateQueueEvtIoDefaultImpl(Queue, Request);
}

unsafe fn CreateQueueEvtIoDefaultImpl(
    Queue: WDFQUEUE,
    Request: WDFREQUEST,
) {
    unsafe {
        let fileObject: WDFFILEOBJECT = WdfRequestGetFileObject(Request);
        let device: WDFDEVICE = WdfIoQueueGetDevice(Queue);
        let mut status: NTSTATUS;
        let deviceExtension: *mut CDROM_DEVICE_EXTENSION = DeviceGetExtension(device);

        // PAGED_CODE(); // no-op

        if fileObject.is_null() {
            // "received a file create request with file object set to NULL"
            CdromRequestCompletion(deviceExtension, Request, STATUS_INTERNAL_ERROR, 0);
            return;
        }

        // Initialize this WDFFILEOBJECT's context: DeviceEvtFileClose's lock /
        // MCN / exclusive-access cleanup reads it. Guarded like the C close
        // shim guards its own lookup.
        let fileObjectContext: *mut FILE_OBJECT_CONTEXT = FileObjectGetContext(fileObject);
        if !fileObjectContext.is_null() {
            (*fileObjectContext).DeviceObject = device;
            (*fileObjectContext).FileObject = fileObject;
            (*fileObjectContext).LockCount = 0;
            (*fileObjectContext).McnDisableCount = 0;
            (*fileObjectContext).EnforceStreamingRead = 0; // FALSE
            (*fileObjectContext).EnforceStreamingWrite = 0; // FALSE
        }

        // send down the create synchronously
        status = DeviceSendRequestSynchronously(device, Request, 0); // FALSE

        // Need to complete the request in this routine, passing on the lower
        // driver's Information (FILE_OPENED...), not 0.
        CdromRequestCompletion(
            deviceExtension,
            Request,
            status,
            WdfRequestGetInformation(Request) as ULONG_PTR,
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn DeviceEvtFileClose(FileObject: WDFFILEOBJECT) {
    CdromDeviceEvtFileClose(FileObject);
}
#[no_mangle]
pub unsafe extern "C" fn DeviceCleanupProtectedLocks(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    FileObjectContext: *mut FILE_OBJECT_CONTEXT,
) -> NTSTATUS {
    let mut status: NTSTATUS = 0; // STATUS_SUCCESS
    PAGED_CODE!();
    TracePrint!(
        4, // TRACE_LEVEL_INFORMATION
        0x1, // TRACE_FLAG_INIT
        b"CleanupProtectedLocks called for WDFDEVICE %p, WDFFILEOBJECT %p, locked %d times.\n\0".as_ptr() as *const i8,
        (*DeviceExtension).Device,
        (*FileObjectContext).FileObject,
        (*FileObjectContext).LockCount,
    );
    WdfWaitLockAcquire((*DeviceExtension).EjectSynchronizationLock, core::ptr::null_mut());
    if (*FileObjectContext).LockCount != 0 {
        (*DeviceExtension).ProtectedLockCount -= (*FileObjectContext).LockCount as i32;
        (*FileObjectContext).LockCount = 0;
        TracePrint!(
            4,
            0x1,
            b"FDO secured lock count = %d lock count = %d\n\0".as_ptr() as *const i8,
            (*DeviceExtension).ProtectedLockCount,
            (*DeviceExtension).LockCount,
        );
        if (*DeviceExtension).ProtectedLockCount == 0 && (*DeviceExtension).LockCount == 0 {
            let mut srb: SCSI_REQUEST_BLOCK = core::mem::zeroed();
            let cdb: *mut CDB = srb.Cdb.as_mut_ptr() as *mut CDB;
            srb.CdbLength = 6;
            (*cdb).MEDIA_REMOVAL.as_mut().OperationCode = 0x1E; // SCSIOP_MEDIUM_REMOVAL
            (*cdb).MEDIA_REMOVAL.as_mut().set_Prevent(0); // FALSE
            srb.TimeOutValue = (*DeviceExtension).TimeOutValue;
            status = DeviceSendSrbSynchronously(
                (*DeviceExtension).Device as *mut core::ffi::c_void,
                &mut srb,
                core::ptr::null_mut(),
                0,
                0,
                core::ptr::null_mut(),
            );
            TracePrint!(
                4,
                0x1,
                b"Allow media removal (unlock) request to drive returned %!STATUS!\n\0".as_ptr() as *const i8,
                status,
            );
        }
    }
    WdfWaitLockRelease((*DeviceExtension).EjectSynchronizationLock);
    status
}
#[no_mangle]
pub unsafe extern "C" fn DeviceCleanupDisableMcn(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    FileObjectContext: *mut FILE_OBJECT_CONTEXT,
) -> i32 {
    // PAGED_CODE(); — ignored (paging not modelled)

    // Forward‑declare unknown externals

    // TRACE_LEVEL_VERBOSE / TRACE_FLAG_INIT are unspecified; use typical values
    let trace_level_verbose: u32 = 5;   // TRACE_LEVEL_VERBOSE
    let trace_flag_init: u32 = 0x1;     // TRACE_FLAG_INIT

    let device = (*DeviceExtension).Device;   // WDFDEVICE (pointer)
    let file_object = (*FileObjectContext).FileObject;
    let mcn_count = (*FileObjectContext).McnDisableCount;

    let format = b"CleanupDisableMcn called for WDFDEVICE %p, WDFFILEOBJECT %p, locked %d times.\n\0"
        .as_ptr() as *const i8;

    TracePrint!(
        trace_level_verbose,
        trace_flag_init,
        format,
        device as *const core::ffi::c_void,
        file_object as *const core::ffi::c_void,
        mcn_count as u32,
    );

    // For each secure lock on this handle decrement the secured lock count
    while (*FileObjectContext).McnDisableCount != 0 {
        DeviceEnableMediaChangeDetection(DeviceExtension, FileObjectContext, 1); // TRUE
    }

    0   // STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn NormalizeIoctl(requestParameters: *mut WDF_REQUEST_PARAMETERS) {
    const IOCTL_SCSI_BASE: u32 = 0x4;  // was 0x00000009 (LLM-invented); bindgen-correct
    const IOCTL_DISK_BASE: u32 = 0x00000007;
    const IOCTL_TAPE_BASE: u32 = 0x0000001f;
    const IOCTL_DVD_BASE: u32 = 0x00000033;
    const IOCTL_CDROM_BASE: u32 = 0x00000002;
    const IOCTL_STORAGE_BASE: u32 = 0x0000002d;
    const TRACE_LEVEL_VERBOSE: u32 = 5; // storswtr.h:26-30
    const TRACE_FLAG_IOCTL: u32 = 4; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)

    let mut ioctlCode: u32;
    let mut baseCode: u32;
    let mut functionCode: u32;

    ioctlCode = (*requestParameters).Parameters.DeviceIoControl.as_ref().IoControlCode;
    baseCode = (ioctlCode >> 16) & 0xFFFF;
    functionCode = (ioctlCode & !0xffffc003u32) >> 2;

    if (baseCode == IOCTL_SCSI_BASE)
        || (baseCode == IOCTL_DISK_BASE)
        || (baseCode == IOCTL_TAPE_BASE)
        || (baseCode == IOCTL_DVD_BASE)
        || (baseCode == IOCTL_CDROM_BASE)
    {
        if (functionCode >= 0x200) && (functionCode <= 0x300) {
            ioctlCode = (ioctlCode & 0x0000ffffu32) | (IOCTL_STORAGE_BASE << 16);

            TracePrint!(
                TRACE_LEVEL_VERBOSE,
                TRACE_FLAG_IOCTL,
                b"IOCTL code recalibrate, New ioctl code is %lx\n\0".as_ptr() as *const i8,
                ioctlCode
            );

            (*requestParameters).Parameters.DeviceIoControl.as_mut().IoControlCode = ioctlCode;
        }
    }
}
#[no_mangle]
pub unsafe extern "system" fn DeviceEvtIoInCallerContext(
    Device: WDFDEVICE,
    Request: WDFREQUEST,
) {
    let mut status: i32;
    let deviceExtension: *mut CDROM_DEVICE_EXTENSION = DeviceGetExtension(Device);
    let mut requestParameters_uninit: core::mem::MaybeUninit<WDF_REQUEST_PARAMETERS> = core::mem::MaybeUninit::uninit();
    WDF_REQUEST_PARAMETERS_INIT(requestParameters_uninit.as_mut_ptr());
    let requestParameters_ptr: *mut WDF_REQUEST_PARAMETERS = requestParameters_uninit.as_mut_ptr();

    CdromPrepareRequestContext(Request, deviceExtension);

    WdfRequestGetParameters(Request, requestParameters_ptr);

    if (*requestParameters_ptr).Type == WdfRequestTypeDeviceControl {
        let mut processed: u8 = 0;
        let cdData: *mut CDROM_DATA = &raw mut (*deviceExtension).DeviceAdditionalData;
        let info: *mut MEDIA_CHANGE_DETECTION_INFO = (*deviceExtension).MediaChangeDetectionInfo;

        if (*requestParameters_ptr).Parameters.DeviceIoControl.as_ref().IoControlCode != IOCTL_MCN_SYNC_FAKE_IOCTL {
            TracePrint!(
                TRACE_LEVEL_INFORMATION,
                TRACE_FLAG_IOCTL,
                b"Receiving IOCTL: %lx\n\0".as_ptr() as *const i8,
                (*requestParameters_ptr).Parameters.DeviceIoControl.as_ref().IoControlCode as u32,
            );
        } else {
            TracePrint!(
                TRACE_LEVEL_VERBOSE,
                TRACE_FLAG_IOCTL,
                b"Receiving IOCTL: %lx\n\0".as_ptr() as *const i8,
                (*requestParameters_ptr).Parameters.DeviceIoControl.as_ref().IoControlCode as u32,
            );
        }

        let fileObject = WdfRequestGetFileObject(Request);
        if !(*cdData).ExclusiveOwner.is_null() && (*cdData).ExclusiveOwner != fileObject {
            let mut isBlocked: u8 = 0;
            status = RequestIsIoctlBlockedByExclusiveAccess(Request, &mut isBlocked as *mut u8);
            UNREFERENCED_PARAMETER((status) as usize);

            if isBlocked != 0 {
                if (*requestParameters_ptr).Parameters.DeviceIoControl.as_ref().IoControlCode == IOCTL_STORAGE_EVENT_NOTIFICATION
                    && !info.is_null()
                    && (*info).AsynchronousNotificationSupported != 0
                {
                    (*info).ANSignalPendingDueToExclusiveLock = 1;
                }

                TracePrint!(
                    TRACE_LEVEL_ERROR,
                    TRACE_FLAG_IOCTL,
                    b"Access Denied! Device in exclusive mode.Failing Ioctl %lx\n\0".as_ptr() as *const i8,
                    (*requestParameters_ptr).Parameters.DeviceIoControl.as_ref().IoControlCode as u32,
                );
                CdromRequestCompletion(deviceExtension, Request, STATUS_ACCESS_DENIED, 0);
                return;
            }
        }

        NormalizeIoctl(requestParameters_ptr);

        if processed == 0 {
            processed = RequestDispatchProcessDirectly(Device as WDFDEVICE, Request as WDFREQUEST, core::ptr::read(requestParameters_ptr));
        }
        if processed == 0 {
            processed = CdromTryHandleStorageControl(deviceExtension, Request, requestParameters_ptr);
        }
        if processed == 0 {
            processed = RequestDispatchToSequentialQueue(Device as WDFDEVICE, Request as WDFREQUEST, core::ptr::read(requestParameters_ptr));
        }
        if processed == 0 {
            processed = RequestDispatchSyncWithSequentialQueue(Device as WDFDEVICE, Request as WDFREQUEST, core::ptr::read(requestParameters_ptr));
        }
        if processed == 0 {
            processed = RequestDispatchSpecialIoctls(Device as WDFDEVICE, Request as WDFREQUEST, core::ptr::read(requestParameters_ptr));
        }
        if processed == 0 {
            processed = RequestDispatchUnknownRequests(Device as WDFDEVICE, Request as WDFREQUEST, core::ptr::read(requestParameters_ptr));
        }

        NT_ASSERT((processed != 0) as i32);
        UNREFERENCED_PARAMETER((processed as i32) as usize);
    } else if (*requestParameters_ptr).Type == WdfRequestTypeDeviceControlInternal {
        RequestProcessInternalDeviceControl(Request as WDFREQUEST, deviceExtension);
    } else {
        status = WdfDeviceEnqueueRequest(Device as WDFDEVICE, Request as WDFREQUEST);
        if !NT_SUCCESS(status) {
            CdromRequestCompletion(deviceExtension, Request, status, WdfRequestGetInformation(Request as WDFREQUEST) as ULONG_PTR);
        }
    }

    return;
}
#[no_mangle]
pub unsafe extern "C" fn RequestDispatchProcessDirectly(
    Device: WDFDEVICE,
    Request: WDFREQUEST,
    RequestParameters: WDF_REQUEST_PARAMETERS,
) -> BOOLEAN {
    const IOCTL_CDROM_GET_INQUIRY_DATA: u32 = 0x00024064;
    const IOCTL_STORAGE_GET_MEDIA_TYPES_EX: u32 = 0x2D0C04;
    const IOCTL_MOUNTDEV_QUERY_UNIQUE_ID: u32 = 0x4D0000;
    const IOCTL_MOUNTDEV_QUERY_DEVICE_NAME: u32 = 0x4D0008;
    const IOCTL_MOUNTDEV_QUERY_SUGGESTED_LINK_NAME: u32 = 0x4D000C;
    const IOCTL_STORAGE_GET_DEVICE_NUMBER: u32 = 0x2D1080;
    const IOCTL_STORAGE_GET_HOTPLUG_INFO: u32 = 0x2D0C14;
    const IOCTL_STORAGE_SET_HOTPLUG_INFO: u32 = 0x2DCC18;
    const IOCTL_STORAGE_EVENT_NOTIFICATION: u32 = 0x2D1C90;
    const IOCTL_VOLUME_ONLINE: u32 = 0x56C008;
    const STATUS_VERIFY_REQUIRED: NTSTATUS = (0x80000016u32 as i32); // ntstatus.h
    const STATUS_IO_DEVICE_ERROR: NTSTATUS = (0xC0000185u32 as i32); // ntstatus.h
    const SL_OVERRIDE_VERIFY_VOLUME: u8 = 0x02; // SL_OVERRIDE_VERIFY_VOLUME

    let mut status: NTSTATUS = 0; // STATUS_SUCCESS
    let mut processed: BOOLEAN = 0; // FALSE
    let mut dataLength: usize = 0;

    let deviceExtension: PCDROM_DEVICE_EXTENSION = DeviceGetExtension(Device);
    let ioctlCode: u32 = RequestParameters.Parameters.DeviceIoControl.as_ref().IoControlCode;
    CdromDbgPrint3(
        b"[DIO] ioctl=%x out=%zx in=%zx\n\0".as_ptr() as *const i8,
        ioctlCode as usize,
        RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength,
        RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
    );

    match ioctlCode {
        IOCTL_CDROM_GET_INQUIRY_DATA => {
            status = RequestHandleGetInquiryData(deviceExtension, Request, core::ptr::read(&RequestParameters), &mut dataLength);
            processed = 1;
        }
        IOCTL_STORAGE_GET_MEDIA_TYPES_EX => {
            status = RequestHandleGetMediaTypeEx(deviceExtension, Request, &mut dataLength);
            processed = 1;
        }
        IOCTL_MOUNTDEV_QUERY_UNIQUE_ID => {
            status = RequestHandleMountQueryUniqueId(deviceExtension, Request, core::ptr::read(&RequestParameters), &mut dataLength);
            processed = 1;
        }
        IOCTL_MOUNTDEV_QUERY_DEVICE_NAME => {
            status = RequestHandleMountQueryDeviceName(deviceExtension, Request, core::ptr::read(&RequestParameters), &mut dataLength);
            processed = 1;
        }
        IOCTL_MOUNTDEV_QUERY_SUGGESTED_LINK_NAME => {
            status = RequestHandleMountQuerySuggestedLinkName(deviceExtension, Request, core::ptr::read(&RequestParameters), &mut dataLength);
            processed = 1;
        }
        IOCTL_STORAGE_GET_DEVICE_NUMBER => {
            status = RequestHandleGetDeviceNumber(deviceExtension, Request, core::ptr::read(&RequestParameters), &mut dataLength);
            processed = 1;
        }
        IOCTL_STORAGE_GET_HOTPLUG_INFO => {
            status = RequestHandleGetHotPlugInfo(deviceExtension, Request, core::ptr::read(&RequestParameters), &mut dataLength);
            processed = 1;
        }
        IOCTL_STORAGE_SET_HOTPLUG_INFO => {
            status = RequestHandleSetHotPlugInfo(deviceExtension, Request, core::ptr::read(&RequestParameters), &mut dataLength);
            processed = 1;
        }
        IOCTL_STORAGE_EVENT_NOTIFICATION => {
            status = RequestHandleEventNotification(deviceExtension, Request, &RequestParameters as *const WDF_REQUEST_PARAMETERS, &mut dataLength);
            processed = 1;
        }
        IOCTL_VOLUME_ONLINE => {
            // originally guarded by #if (NTDDI_VERSION >= NTDDI_WIN8)
            if WdfRequestGetRequestorMode(Request) == 0i8 {
                processed = 1;
            }
        }
        _ => {
            processed = 0;
        }
    }

    if processed != 0 {
        let mut currentStackLocationFlags: u8;
        currentStackLocationFlags = RequestGetCurrentStackLocationFlags(Request);

        if (status == STATUS_VERIFY_REQUIRED) &&
            ((currentStackLocationFlags & SL_OVERRIDE_VERIFY_VOLUME) != 0)
        {
            status = STATUS_IO_DEVICE_ERROR;
            let _ = status; // UNREFERENCED_PARAMETER macro

            processed = RequestDispatchProcessDirectly(Device, Request, RequestParameters);
        } else {
            RequestCompletion(deviceExtension, Request, status, dataLength as CompletionInformation);
        }
    }

    processed
}
#[no_mangle]
pub unsafe extern "C" fn RequestDispatchToSequentialQueue(
    Device: WDFDEVICE,
    Request: WDFREQUEST,
    RequestParameters: WDF_REQUEST_PARAMETERS,
) -> u8 {
    let mut status: NTSTATUS = STATUS_SUCCESS;
    let mut processed: u8; // FALSE
    let mut dataLength: usize = 0;
    let mut inZeroPowerState: u8 = 0; // FALSE

    let deviceExtension: *mut CDROM_DEVICE_EXTENSION = DeviceGetExtension(Device);
    let ioctlCode: u32 = RequestParameters.Parameters.DeviceIoControl.as_ref().IoControlCode;
    let zpoddInfo: *mut ZERO_POWER_ODD_INFO = (*deviceExtension).ZeroPowerODDInfo;

    if (zpoddInfo as *const _ as usize != 0) && ((*zpoddInfo).InZeroPowerState != 0) {
        inZeroPowerState = 1;
    }

    match ioctlCode {
        IOCTL_CDROM_RAW_READ => {
            status = RequestValidateRawRead(
                deviceExtension,
                Request,
                core::ptr::read(&RequestParameters),
                &mut dataLength,
            );
            processed = 1;
        }
        IOCTL_DISK_GET_DRIVE_GEOMETRY_EX | IOCTL_CDROM_GET_DRIVE_GEOMETRY_EX => {
            TracePrint!(
                TRACE_LEVEL_INFORMATION,
                TRACE_FLAG_IOCTL,
                b"RequestDispatchToSequentialQueue: Get drive geometryEx\n\0".as_ptr() as *const i8,
            );
            if RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength
                < core::mem::offset_of!(DISK_GEOMETRY_EX, Data) as usize
            {
                status = STATUS_BUFFER_TOO_SMALL;
                dataLength = core::mem::offset_of!(DISK_GEOMETRY_EX, Data);
            } else if inZeroPowerState != 0 {
                status = STATUS_NO_MEDIA_IN_DEVICE;
            } else {
                status = STATUS_SUCCESS;
            }
            processed = 1;
        }
        IOCTL_DISK_GET_DRIVE_GEOMETRY | IOCTL_CDROM_GET_DRIVE_GEOMETRY => {
            TracePrint!(
                TRACE_LEVEL_INFORMATION,
                TRACE_FLAG_IOCTL,
                b"RequestDispatchToSequentialQueue: Get drive geometry\n\0".as_ptr() as *const i8,
            );
            if RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength
                < core::mem::size_of::<DISK_GEOMETRY>() as usize
            {
                status = STATUS_BUFFER_TOO_SMALL;
                dataLength = core::mem::size_of::<DISK_GEOMETRY>();
            } else if inZeroPowerState != 0 {
                status = STATUS_NO_MEDIA_IN_DEVICE;
            } else {
                status = STATUS_SUCCESS;
            }
            processed = 1;
        }
        IOCTL_CDROM_READ_TOC_EX => {
            status = RequestValidateReadTocEx(
                deviceExtension,
                Request,
                core::ptr::read(&RequestParameters),
                &mut dataLength,
            );
            if inZeroPowerState != 0 {
                status = STATUS_NO_MEDIA_IN_DEVICE;
            }
            processed = 1;
        }
        IOCTL_CDROM_READ_TOC => {
            status = RequestValidateReadToc(deviceExtension, core::ptr::read(&RequestParameters), &mut dataLength);
            if inZeroPowerState != 0 {
                status = STATUS_NO_MEDIA_IN_DEVICE;
            }
            processed = 1;
        }
        IOCTL_CDROM_GET_LAST_SESSION => {
            status = RequestValidateGetLastSession(
                deviceExtension,
                core::ptr::read(&RequestParameters),
                &mut dataLength,
            );
            processed = 1;
        }
        IOCTL_CDROM_PLAY_AUDIO_MSF => {
            TracePrint!(
                TRACE_LEVEL_INFORMATION,
                TRACE_FLAG_IOCTL,
                b"RequestDispatchToSequentialQueue: Play audio MSF\n\0".as_ptr() as *const i8,
            );
            if RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength
                < core::mem::size_of::<CDROM_PLAY_AUDIO_MSF>() as usize
            {
                status = STATUS_INFO_LENGTH_MISMATCH;
            } else {
                status = STATUS_SUCCESS;
            }
            processed = 1;
        }
        IOCTL_CDROM_SEEK_AUDIO_MSF => {
            TracePrint!(
                TRACE_LEVEL_INFORMATION,
                TRACE_FLAG_IOCTL,
                b"RequestDispatchToSequentialQueue: Seek audio MSF\n\0".as_ptr() as *const i8,
            );
            if RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength
                < core::mem::size_of::<CDROM_SEEK_AUDIO_MSF>() as usize
            {
                status = STATUS_INFO_LENGTH_MISMATCH;
            } else {
                status = STATUS_SUCCESS;
            }
            processed = 1;
        }
        IOCTL_CDROM_PAUSE_AUDIO => {
            TracePrint!(
                TRACE_LEVEL_INFORMATION,
                TRACE_FLAG_IOCTL,
                b"RequestDispatchToSequentialQueue: Pause audio\n\0".as_ptr() as *const i8,
            );
            status = STATUS_SUCCESS;
            processed = 1;
        }
        IOCTL_CDROM_RESUME_AUDIO => {
            TracePrint!(
                TRACE_LEVEL_INFORMATION,
                TRACE_FLAG_IOCTL,
                b"RequestDispatchToSequentialQueue: Resume audio\n\0".as_ptr() as *const i8,
            );
            status = STATUS_SUCCESS;
            processed = 1;
        }
        IOCTL_CDROM_READ_Q_CHANNEL => {
            status = RequestValidateReadQChannel(Request, core::ptr::read(&RequestParameters), &mut dataLength);
            processed = 1;
        }
        IOCTL_CDROM_GET_VOLUME => {
            TracePrint!(
                TRACE_LEVEL_INFORMATION,
                TRACE_FLAG_IOCTL,
                b"RequestDispatchToSequentialQueue: Get volume control\n\0".as_ptr() as *const i8,
            );
            if RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength
                < core::mem::size_of::<VOLUME_CONTROL>() as usize
            {
                status = STATUS_BUFFER_TOO_SMALL;
                dataLength = core::mem::size_of::<VOLUME_CONTROL>();
            } else {
                status = STATUS_SUCCESS;
            }
            processed = 1;
        }
        IOCTL_CDROM_SET_VOLUME => {
            TracePrint!(
                TRACE_LEVEL_INFORMATION,
                TRACE_FLAG_IOCTL,
                b"RequestDispatchToSequentialQueue: Set volume control\n\0".as_ptr() as *const i8,
            );
            if RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength
                < core::mem::size_of::<VOLUME_CONTROL>() as usize
            {
                status = STATUS_INFO_LENGTH_MISMATCH;
            } else {
                status = STATUS_SUCCESS;
            }
            processed = 1;
        }
        IOCTL_CDROM_STOP_AUDIO => {
            TracePrint!(
                TRACE_LEVEL_INFORMATION,
                TRACE_FLAG_IOCTL,
                b"RequestDispatchToSequentialQueue: Stop audio\n\0".as_ptr() as *const i8,
            );
            status = STATUS_SUCCESS;
            processed = 1;
        }
        IOCTL_STORAGE_CHECK_VERIFY | IOCTL_STORAGE_CHECK_VERIFY2 => {
            TracePrint!(
                TRACE_LEVEL_INFORMATION,
                TRACE_FLAG_IOCTL,
                b"RequestDispatchToSequentialQueue: [%p] Check Verify\n\0".as_ptr() as *const i8,
                // extra argument Request omitted
            );
            if (RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength > 0)
                && (RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength
                    < core::mem::size_of::<u32>() as usize)
            {
                status = STATUS_BUFFER_TOO_SMALL;
                dataLength = core::mem::size_of::<u32>();
            } else if inZeroPowerState != 0 {
                status = STATUS_NO_MEDIA_IN_DEVICE;
            } else {
                status = STATUS_SUCCESS;
            }
            processed = 1;
        }
        IOCTL_DVD_GET_REGION => {
            status = STATUS_SUCCESS;
            processed = 1;
        }
        IOCTL_DVD_READ_STRUCTURE => {
            TracePrint!(
                TRACE_LEVEL_INFORMATION,
                TRACE_FLAG_IOCTL,
                b"RequestDispatchToSequentialQueue: [%p] IOCTL_DVD_READ_STRUCTURE\n\0".as_ptr()
                    as *const i8,
                // extra argument Request omitted
            );
            status = RequestValidateDvdReadStructure(
                deviceExtension,
                core::ptr::read(&RequestParameters),
                &mut dataLength,
            );
            processed = 1;
        }
        IOCTL_DVD_READ_KEY => {
            TracePrint!(
                TRACE_LEVEL_INFORMATION,
                TRACE_FLAG_IOCTL,
                b"RequestDispatchToSequentialQueue: [%p] IOCTL_DVD_READ_KEY\n\0".as_ptr()
                    as *const i8,
                // extra argument Request omitted
            );
            status = RequestValidateDvdReadKey(
                deviceExtension,
                Request,
                core::ptr::read(&RequestParameters),
                &mut dataLength,
            );
            processed = 1;
        }
        IOCTL_DVD_START_SESSION => {
            TracePrint!(
                TRACE_LEVEL_INFORMATION,
                TRACE_FLAG_IOCTL,
                b"RequestDispatchToSequentialQueue: [%p] IOCTL_DVD_START_SESSION\n\0".as_ptr()
                    as *const i8,
                // extra argument Request omitted
            );
            status = RequestValidateDvdStartSession(
                deviceExtension,
                core::ptr::read(&RequestParameters),
                &mut dataLength,
            );
            processed = 1;
        }
        IOCTL_DVD_SEND_KEY | IOCTL_DVD_SEND_KEY2 => {
            TracePrint!(
                TRACE_LEVEL_INFORMATION,
                TRACE_FLAG_IOCTL,
                b"RequestDispatchToSequentialQueue: [%p] IOCTL_DVD_SEND_KEY\n\0".as_ptr()
                    as *const i8,
                // extra argument Request omitted
            );
            status = RequestValidateDvdSendKey(
                deviceExtension,
                Request,
                core::ptr::read(&RequestParameters),
                &mut dataLength,
            );
            processed = 1;
        }
        IOCTL_STORAGE_SET_READ_AHEAD => {
            TracePrint!(
                TRACE_LEVEL_INFORMATION,
                TRACE_FLAG_IOCTL,
                b"RequestDispatchToSequentialQueue: [%p] SetReadAhead\n\0".as_ptr() as *const i8,
                // extra argument Request omitted
            );
            if RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength
                < core::mem::size_of::<STORAGE_SET_READ_AHEAD>() as usize
            {
                status = STATUS_INVALID_PARAMETER;
            } else {
                status = STATUS_SUCCESS;
            }
            processed = 1;
        }
        IOCTL_DISK_IS_WRITABLE => {
            status = STATUS_SUCCESS;
            processed = 1;
        }
        IOCTL_DISK_GET_DRIVE_LAYOUT => {
            let requiredSize: u32 =
                (core::mem::offset_of!(DRIVE_LAYOUT_INFORMATION, PartitionEntry)
                    + core::mem::size_of::<PARTITION_INFORMATION>()) as u32;
            TracePrint!(
                TRACE_LEVEL_INFORMATION,
                TRACE_FLAG_IOCTL,
                b"RequestDispatchToSequentialQueue: Get drive layout\n\0".as_ptr() as *const i8,
            );
            if RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength < requiredSize as usize {
                status = STATUS_BUFFER_TOO_SMALL;
                dataLength = requiredSize as usize;
            } else {
                status = STATUS_SUCCESS;
            }
            processed = 1;
        }
        IOCTL_DISK_GET_DRIVE_LAYOUT_EX => {
            let requiredSize: u32 =
                (core::mem::offset_of!(DRIVE_LAYOUT_INFORMATION_EX, PartitionEntry)
                    + core::mem::size_of::<PARTITION_INFORMATION_EX>()) as u32;
            TracePrint!(
                TRACE_LEVEL_INFORMATION,
                TRACE_FLAG_IOCTL,
                b"RequestDispatchToSequentialQueue: Get drive layoutEx\n\0".as_ptr() as *const i8,
            );
            if RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength < requiredSize as usize {
                status = STATUS_BUFFER_TOO_SMALL;
                dataLength = requiredSize as usize;
            } else {
                status = STATUS_SUCCESS;
            }
            processed = 1;
        }
        IOCTL_DISK_GET_PARTITION_INFO => {
            TracePrint!(
                TRACE_LEVEL_INFORMATION,
                TRACE_FLAG_IOCTL,
                b"RequestDispatchToSequentialQueue: Get Partition Info\n\0".as_ptr() as *const i8,
            );
            if RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength
                < core::mem::size_of::<PARTITION_INFORMATION>() as usize
            {
                status = STATUS_BUFFER_TOO_SMALL;
                dataLength = core::mem::size_of::<PARTITION_INFORMATION>();
            } else {
                status = STATUS_SUCCESS;
            }
            processed = 1;
        }
        IOCTL_DISK_GET_PARTITION_INFO_EX => {
            TracePrint!(
                TRACE_LEVEL_INFORMATION,
                TRACE_FLAG_IOCTL,
                b"RequestDispatchToSequentialQueue: Get Partition InfoEx\n\0".as_ptr() as *const i8,
            );
            if RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength
                < core::mem::size_of::<PARTITION_INFORMATION_EX>() as usize
            {
                status = STATUS_BUFFER_TOO_SMALL;
                dataLength = core::mem::size_of::<PARTITION_INFORMATION_EX>();
            } else {
                status = STATUS_SUCCESS;
            }
            processed = 1;
        }
        IOCTL_DISK_VERIFY => {
            TracePrint!(
                TRACE_LEVEL_INFORMATION,
                TRACE_FLAG_IOCTL,
                b"RequestDispatchToSequentialQueue: IOCTL_DISK_VERIFY to device %p through request %p\n\0"
                    .as_ptr() as *const i8,
                // extra arguments Device and Request omitted
            );
            if RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength
                < core::mem::size_of::<VERIFY_INFORMATION>() as usize
            {
                status = STATUS_INVALID_PARAMETER;
            } else {
                status = STATUS_SUCCESS;
            }
            processed = 1;
        }
        IOCTL_DISK_GET_LENGTH_INFO => {
            TracePrint!(
                TRACE_LEVEL_INFORMATION,
                TRACE_FLAG_IOCTL,
                b"RequestDispatchToSequentialQueue: Disk Get Length InfoEx\n\0".as_ptr()
                    as *const i8,
            );
            if RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength
                < core::mem::size_of::<GET_LENGTH_INFORMATION>() as usize
            {
                status = STATUS_BUFFER_TOO_SMALL;
                dataLength = core::mem::size_of::<GET_LENGTH_INFORMATION>();
            } else if inZeroPowerState != 0 {
                status = STATUS_NO_MEDIA_IN_DEVICE;
            } else {
                status = STATUS_SUCCESS;
            }
            processed = 1;
        }
        IOCTL_CDROM_GET_CONFIGURATION => {
            TracePrint!(
                TRACE_LEVEL_INFORMATION,
                TRACE_FLAG_IOCTL,
                b"RequestDispatchToSequentialQueue: [%p] IOCTL_CDROM_GET_CONFIGURATION\n\0"
                    .as_ptr() as *const i8,
                // extra argument Request omitted
            );
            status = RequestValidateGetConfiguration(
                deviceExtension,
                Request,
                core::ptr::read(&RequestParameters),
                &mut dataLength,
            );
            processed = 1;
        }
        IOCTL_CDROM_SET_SPEED => {
            TracePrint!(
                TRACE_LEVEL_INFORMATION,
                TRACE_FLAG_IOCTL,
                b"RequestDispatchToSequentialQueue: [%p] IOCTL_CDROM_SET_SPEED\n\0".as_ptr()
                    as *const i8,
                // extra argument Request omitted
            );
            status = RequestValidateSetSpeed(deviceExtension, Request, core::ptr::read(&RequestParameters), &mut dataLength);
            processed = 1;
        }
        IOCTL_DVD_END_SESSION => {
            TracePrint!(
                TRACE_LEVEL_INFORMATION,
                TRACE_FLAG_IOCTL,
                b"RequestDispatchToSequentialQueue: [%p] IOCTL_DVD_END_SESSION\n\0".as_ptr()
                    as *const i8,
                // extra argument Request omitted
            );
            status = RequestValidateDvdEndSession(
                deviceExtension,
                Request,
                core::ptr::read(&RequestParameters),
                &mut dataLength,
            );
            processed = 1;
        }
        IOCTL_AACS_END_SESSION => {
            TracePrint!(
                TRACE_LEVEL_INFORMATION,
                TRACE_FLAG_IOCTL,
                b"RequestDispatchToSequentialQueue: [%p] IOCTL_AACS_END_SESSION\n\0".as_ptr()
                    as *const i8,
                // extra argument Request omitted
            );
            status = RequestValidateAacsEndSession(
                deviceExtension,
                Request,
                core::ptr::read(&RequestParameters),
                &mut dataLength,
            );
            processed = 1;
        }
        IOCTL_AACS_READ_MEDIA_KEY_BLOCK => {
            TracePrint!(
                TRACE_LEVEL_WARNING,
                TRACE_FLAG_IOCTL,
                b"AACS: Querying full MKB with bufferSize of %x bytes\n\0".as_ptr() as *const i8,
                // extra argument (int)RequestParameters.Parameters.DeviceIoControl.OutputBufferLength omitted
            );
            status = RequestValidateAacsReadMediaKeyBlock(
                deviceExtension,
                Request,
                core::ptr::read(&RequestParameters),
                &mut dataLength,
            );
            processed = 1;
        }
        IOCTL_AACS_START_SESSION => {
            TracePrint!(
                TRACE_LEVEL_WARNING,
                TRACE_FLAG_IOCTL,
                b"AACS: Requesting AGID\n\0".as_ptr() as *const i8,
            );
            status = RequestValidateAacsStartSession(
                deviceExtension,
                core::ptr::read(&RequestParameters),
                &mut dataLength,
            );
            processed = 1;
        }
        IOCTL_AACS_SEND_CERTIFICATE => {
            TracePrint!(
                TRACE_LEVEL_WARNING,
                TRACE_FLAG_IOCTL,
                b"AACS: Sending host certificate to drive\n\0".as_ptr() as *const i8,
            );
            status = RequestValidateAacsSendCertificate(
                deviceExtension,
                Request,
                core::ptr::read(&RequestParameters),
                &mut dataLength,
            );
            processed = 1;
        }
        IOCTL_AACS_GET_CERTIFICATE => {
            TracePrint!(
                TRACE_LEVEL_WARNING,
                TRACE_FLAG_IOCTL,
                b"AACS: Querying drive certificate\n\0".as_ptr() as *const i8,
            );
            status = RequestValidateAacsGetCertificate(
                deviceExtension,
                Request,
                core::ptr::read(&RequestParameters),
                &mut dataLength,
            );
            processed = 1;
        }
        IOCTL_AACS_GET_CHALLENGE_KEY => {
            TracePrint!(
                TRACE_LEVEL_WARNING,
                TRACE_FLAG_IOCTL,
                b"AACS: Querying drive challenge key\n\0".as_ptr() as *const i8,
            );
            status = RequestValidateAacsGetChallengeKey(
                deviceExtension,
                Request,
                core::ptr::read(&RequestParameters),
                &mut dataLength,
            );
            processed = 1;
        }
        IOCTL_AACS_SEND_CHALLENGE_KEY => {
            TracePrint!(
                TRACE_LEVEL_WARNING,
                TRACE_FLAG_IOCTL,
                b"AACS: Sending drive challenge key\n\0".as_ptr() as *const i8,
            );
            status = RequestValidateAacsSendChallengeKey(
                deviceExtension,
                Request,
                core::ptr::read(&RequestParameters),
                &mut dataLength,
            );
            processed = 1;
        }
        IOCTL_AACS_READ_VOLUME_ID => {
            TracePrint!(
                TRACE_LEVEL_WARNING,
                TRACE_FLAG_IOCTL,
                b"AACS: Reading volume ID\n\0".as_ptr() as *const i8,
            );
            status = RequestValidateAacsReadVolumeId(
                deviceExtension,
                Request,
                core::ptr::read(&RequestParameters),
                &mut dataLength,
            );
            processed = 1;
        }
        IOCTL_AACS_READ_SERIAL_NUMBER => {
            TracePrint!(
                TRACE_LEVEL_WARNING,
                TRACE_FLAG_IOCTL,
                b"AACS: Reading Serial Number\n\0".as_ptr() as *const i8,
            );
            status = RequestValidateAacsReadSerialNumber(
                deviceExtension,
                Request,
                core::ptr::read(&RequestParameters),
                &mut dataLength,
            );
            processed = 1;
        }
        IOCTL_AACS_READ_MEDIA_ID => {
            TracePrint!(
                TRACE_LEVEL_WARNING,
                TRACE_FLAG_IOCTL,
                b"AACS: Reading media ID\n\0".as_ptr() as *const i8,
            );
            status = RequestValidateAacsReadMediaId(
                deviceExtension,
                Request,
                core::ptr::read(&RequestParameters),
                &mut dataLength,
            );
            processed = 1;
        }
        IOCTL_AACS_READ_BINDING_NONCE | IOCTL_AACS_GENERATE_BINDING_NONCE => {
            if ioctlCode == IOCTL_AACS_GENERATE_BINDING_NONCE {
                TracePrint!(
                    TRACE_LEVEL_WARNING,
                    TRACE_FLAG_IOCTL,
                    b"AACS: Generating new binding nonce\n\0".as_ptr() as *const i8,
                );
            } else {
                TracePrint!(
                    TRACE_LEVEL_WARNING,
                    TRACE_FLAG_IOCTL,
                    b"AACS: Reading existing binding nonce\n\0".as_ptr() as *const i8,
                );
            }
            status = RequestValidateAacsBindingNonce(
                deviceExtension,
                Request,
                core::ptr::read(&RequestParameters),
                &mut dataLength,
            );
            processed = 1;
        }
        IOCTL_CDROM_ENABLE_STREAMING => {
            TracePrint!(
                TRACE_LEVEL_INFORMATION,
                TRACE_FLAG_IOCTL,
                b"RequestDispatchToSequentialQueue: [%p] IOCTL_CDROM_ENABLE_STREAMING\n\0"
                    .as_ptr() as *const i8,
                // extra argument Request omitted
            );
            status = RequestValidateEnableStreaming(Request, core::ptr::read(&RequestParameters), &mut dataLength);
            processed = 1;
        }
        IOCTL_CDROM_SEND_OPC_INFORMATION => {
            TracePrint!(
                TRACE_LEVEL_INFORMATION,
                TRACE_FLAG_IOCTL,
                b"RequestDispatchToSequentialQueue: [%p] IOCTL_CDROM_SEND_OPC_INFORMATION\n\0"
                    .as_ptr() as *const i8,
                // extra argument Request omitted
            );
            status = RequestValidateSendOpcInformation(Request, core::ptr::read(&RequestParameters), &mut dataLength);
            processed = 1;
        }
        IOCTL_CDROM_GET_PERFORMANCE => {
            TracePrint!(
                TRACE_LEVEL_INFORMATION,
                TRACE_FLAG_IOCTL,
                b"RequestDispatchToSequentialQueue: [%p] IOCTL_CDROM_GET_PERFORMANCE\n\0"
                    .as_ptr() as *const i8,
                // extra argument Request omitted
            );
            status = RequestValidateGetPerformance(Request, core::ptr::read(&RequestParameters), &mut dataLength);
            processed = 1;
        }
        IOCTL_STORAGE_MEDIA_REMOVAL | IOCTL_STORAGE_EJECTION_CONTROL => {
            if RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength
                < core::mem::size_of::<PREVENT_MEDIA_REMOVAL>() as usize
            {
                status = STATUS_INFO_LENGTH_MISMATCH;
            } else {
                status = STATUS_SUCCESS;
            }
            processed = 1;
        }
        IOCTL_STORAGE_MCN_CONTROL => {
            if RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength
                < core::mem::size_of::<PREVENT_MEDIA_REMOVAL>() as usize
            {
                status = STATUS_INFO_LENGTH_MISMATCH;
            } else {
                status = STATUS_SUCCESS;
            }
            processed = 1;
        }
        IOCTL_STORAGE_RESERVE | IOCTL_STORAGE_RELEASE => {
            status = STATUS_SUCCESS;
            processed = 1;
        }
        IOCTL_STORAGE_PERSISTENT_RESERVE_IN | IOCTL_STORAGE_PERSISTENT_RESERVE_OUT => {
            status = RequestValidatePersistentReserve(
                deviceExtension,
                Request,
                core::ptr::read(&RequestParameters),
                &mut dataLength,
            );
            processed = 1;
        }
        IOCTL_STORAGE_EJECT_MEDIA | IOCTL_STORAGE_LOAD_MEDIA | IOCTL_STORAGE_LOAD_MEDIA2 => {
            status = STATUS_SUCCESS;
            processed = 1;
        }
        IOCTL_STORAGE_FIND_NEW_DEVICES => {
            IoInvalidateDeviceRelations((*deviceExtension).LowerPdo as *mut core::ffi::c_void, BusRelations);
            status = STATUS_SUCCESS;
            processed = 1;
        }
        IOCTL_STORAGE_READ_CAPACITY => {
            if RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength
                < core::mem::size_of::<STORAGE_READ_CAPACITY>() as usize
            {
                dataLength = core::mem::size_of::<STORAGE_READ_CAPACITY>();
                status = STATUS_BUFFER_TOO_SMALL;
            } else if inZeroPowerState != 0 {
                status = STATUS_NO_MEDIA_IN_DEVICE;
            } else {
                status = STATUS_SUCCESS;
            }
            processed = 1;
        }
        IOCTL_STORAGE_CHECK_PRIORITY_HINT_SUPPORT => {
            status = STATUS_NOT_SUPPORTED;
            processed = 1;
        }
        IOCTL_DISK_ARE_VOLUMES_READY => {
            status = STATUS_SUCCESS;
            processed = 1;
        }
        IOCTL_VOLUME_ONLINE | IOCTL_VOLUME_POST_ONLINE => {
            status = STATUS_SUCCESS;
            processed = 1;
        }
        _ => {
            processed = 0;
        }
    } // end match ioctlCode

    if processed != 0 {
        let mut currentStackLocationFlags: u8;
        currentStackLocationFlags = RequestGetCurrentStackLocationFlags(Request);
        if (status == STATUS_VERIFY_REQUIRED)
            && ((currentStackLocationFlags & SL_OVERRIDE_VERIFY_VOLUME) != 0)
        {
            status = STATUS_IO_DEVICE_ERROR;
            let _ = status; // UNREFERENCED_PARAMETER
            processed = RequestDispatchToSequentialQueue(Device, Request, RequestParameters);
        } else {
            if NT_SUCCESS(status) {
                status = WdfDeviceEnqueueRequest(Device, Request);
            }
            if !NT_SUCCESS(status) {
                RequestCompletion(deviceExtension, Request, status, dataLength as CompletionInformation);
            }
        }
    }

    processed
}
#[no_mangle]
pub unsafe extern "C" fn RequestDispatchSyncWithSequentialQueue(
    Device: WDFDEVICE,
    Request: WDFREQUEST,
    RequestParameters: WDF_REQUEST_PARAMETERS,
) -> BOOLEAN {
    let mut status: NTSTATUS = STATUS_SUCCESS;
    let mut processed: BOOLEAN; // FALSE
    let mut dataLength: usize = 0;

    let deviceExtension: *mut CDROM_DEVICE_EXTENSION = DeviceGetExtension(Device);
    let ioctlCode: u32 = RequestParameters.Parameters.DeviceIoControl.as_ref().IoControlCode;

    match ioctlCode {
        IOCTL_CDROM_EXCLUSIVE_ACCESS => {
            // 1. Validate
            status = RequestValidateExclusiveAccess(
                Request,
                core::ptr::read(&RequestParameters),
                core::ptr::addr_of_mut!(dataLength),
            );

            // 2. keep the request in serialized manner and stay in user's context.
            if NT_SUCCESS(status) {
                let mut exclusiveAccess: *mut CDROM_EXCLUSIVE_ACCESS = core::ptr::null_mut();

                status = WdfRequestRetrieveInputBuffer(
                    Request,
                    RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
                    core::ptr::addr_of_mut!(exclusiveAccess) as *mut *mut core::ffi::c_void,
                    core::ptr::null_mut(),
                );

                if NT_SUCCESS(status) {
                    // do not need to check "status" as it passed validation and cannot fail
                    match (*exclusiveAccess).RequestType {
                        crate::types::_EXCLUSIVE_ACCESS_REQUEST_TYPE_ExclusiveAccessQueryState => {
                            status = RequestSetContextFields(
                                Request,
                                Some(crate::ioctl::RequestHandleExclusiveAccessQueryLockState),
                            );
                        }
                        crate::types::_EXCLUSIVE_ACCESS_REQUEST_TYPE_ExclusiveAccessLockDevice => {
                            status = RequestSetContextFields(
                                Request,
                                Some(crate::ioctl::RequestHandleExclusiveAccessLockDevice),
                            );
                        }
                        crate::types::_EXCLUSIVE_ACCESS_REQUEST_TYPE_ExclusiveAccessUnlockDevice => {
                            status = RequestSetContextFields(
                                Request,
                                Some(crate::ioctl::RequestHandleExclusiveAccessUnlockDevice),
                            );
                        }
                        _ => {
                            // already validated in RequestValidateExclusiveAccess()
                            NT_ASSERT(0); // FALSE
                        }
                    }
                }

                if NT_SUCCESS(status) {
                    // now, put the special synchronization information into the context
                    RequestSynchronizeProcessWithSerialQueue(Device, Request);

                    // "status" is used for debugging in above statement, reset to success
                    status = STATUS_SUCCESS;
                }
            }

            processed = 1; // TRUE
        }
        _ => {
            processed = 0; // FALSE
        }
    }

    // Following process is only valid if the request is not really processed. (failed in validation)
    if (processed != 0) && (!NT_SUCCESS(status)) {
        let currentStackLocationFlags: u8 = RequestGetCurrentStackLocationFlags(Request);

        if (status == STATUS_VERIFY_REQUIRED)
            && (currentStackLocationFlags & SL_OVERRIDE_VERIFY_VOLUME) != 0
        {
            //
            // If the status is verified required and this request
            // should bypass verify required then retry the request.
            //
            status = STATUS_IO_DEVICE_ERROR;
            UNREFERENCED_PARAMETER((status) as usize); // disables prefast warning; defensive coding...

            processed = RequestDispatchSyncWithSequentialQueue(Device, Request, RequestParameters);
        } else {
            // Validation failed / forward failed, complete the request.
            RequestCompletion(
                deviceExtension,
                Request,
                status,
                dataLength as CompletionInformation,
            );
        }
    }

    processed
}
#[no_mangle]
pub unsafe extern "C" fn RequestDispatchSpecialIoctls(
    Device: WDFDEVICE,
    Request: WDFREQUEST,
    RequestParameters: WDF_REQUEST_PARAMETERS,
) -> BOOLEAN {
    let mut status: NTSTATUS = ntkrnl_sys::base::STATUS_SUCCESS;
    let mut processed: BOOLEAN; // FALSE
    let mut dataLength: usize = 0;
    let mut requestCompleted: BOOLEAN = 0; // FALSE

    let deviceExtension: *mut CDROM_DEVICE_EXTENSION = DeviceGetExtension(Device);
    let cdData: *mut CDROM_DATA = &raw mut (*deviceExtension).DeviceAdditionalData;
    let ioctlCode: u32 = RequestParameters.Parameters.DeviceIoControl.as_ref().IoControlCode;

    match ioctlCode {
        IOCTL_SCSI_PASS_THROUGH
        | IOCTL_SCSI_PASS_THROUGH_DIRECT
        | IOCTL_SCSI_PASS_THROUGH_EX
        | IOCTL_SCSI_PASS_THROUGH_DIRECT_EX => {
            // SPTI is considered special case as we need to set the MinorFunction before pass to low level.
            let is_32bit = if cfg!(target_pointer_width = "64") {
                WdfRequestIsFrom32BitProcess(Request) != 0
            } else {
                false
            };

            if is_32bit {
                if (ioctlCode == IOCTL_SCSI_PASS_THROUGH) || (ioctlCode == IOCTL_SCSI_PASS_THROUGH_DIRECT) {
                    if RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength
                        < core::mem::size_of::<SCSI_PASS_THROUGH32>()
                    {
                        status = ntkrnl_sys::constants::STATUS_INVALID_PARAMETER;
                    }
                } else {
                    if RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength
                        < core::mem::size_of::<SCSI_PASS_THROUGH32_EX>()
                    {
                        status = ntkrnl_sys::constants::STATUS_INVALID_PARAMETER;
                    }
                }
            } else {
                if (ioctlCode == IOCTL_SCSI_PASS_THROUGH) || (ioctlCode == IOCTL_SCSI_PASS_THROUGH_DIRECT) {
                    if RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength
                        < core::mem::size_of::<SCSI_PASS_THROUGH>()
                    {
                        status = ntkrnl_sys::constants::STATUS_INVALID_PARAMETER;
                    }
                } else {
                    if RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength
                        < core::mem::size_of::<SCSI_PASS_THROUGH_EX>()
                    {
                        status = ntkrnl_sys::constants::STATUS_INVALID_PARAMETER;
                    }
                }
            }

            if !ntkrnl_sys::base::NT_SUCCESS(status) {
                // validation failed.
                RequestCompletion(deviceExtension, Request, status, dataLength as CompletionInformation);
            } else {
                // keep the request in serialized manner and stay in user's context.
                status = RequestSetContextFields(Request, Some(crate::ioctl::RequestHandleScsiPassThrough));

                if ntkrnl_sys::base::NT_SUCCESS(status) {
                    status = RequestSynchronizeProcessWithSerialQueue(Device, Request);
                } else {
                    RequestCompletion(deviceExtension, Request, status, 0);
                }
            }

            requestCompleted = 1;
            processed = 1;
        } // end SCSI case

        IOCTL_STORAGE_QUERY_PROPERTY => {
            if RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength
                < core::mem::size_of::<STORAGE_PROPERTY_QUERY>()
            {
                status = ntkrnl_sys::constants::STATUS_INFO_LENGTH_MISMATCH;
            } else {
                let mut inputBuffer: *mut STORAGE_PROPERTY_QUERY = core::ptr::null_mut();

                status = WdfRequestRetrieveInputBuffer(
                    Request,
                    RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength as usize,
                    &raw mut inputBuffer as *mut *mut c_void,
                    core::ptr::null_mut(),
                );

                if ntkrnl_sys::base::NT_SUCCESS(status) {
                    let fileObject = WdfRequestGetFileObject(Request);
                    if (*cdData).ExclusiveOwner.is_null()
                        || (*cdData).ExclusiveOwner == fileObject
                        || ((*inputBuffer).QueryType == PropertyExistsQuery)
                    {
                        if (*inputBuffer).PropertyId == StorageDeviceUniqueIdProperty {
                            // previously handled in classpnp
                            // keep the request in serialized manner and stay in user's context.
                            status =
                                RequestSetContextFields(Request, Some(crate::ioctl::RequestHandleQueryPropertyDeviceUniqueId));

                            if ntkrnl_sys::base::NT_SUCCESS(status) {
                                status = RequestSynchronizeProcessWithSerialQueue(Device, Request);
                                // remember that the request has been completed.
                                requestCompleted = 1;
                            }
                        } else if (*inputBuffer).PropertyId == StorageDeviceWriteCacheProperty {
                            // previously handled in classpnp
                            // keep the request in serialized manner and stay in user's context.
                            status =
                                RequestSetContextFields(Request, Some(crate::ioctl::RequestHandleQueryPropertyWriteCache));

                            if ntkrnl_sys::base::NT_SUCCESS(status) {
                                status = RequestSynchronizeProcessWithSerialQueue(Device, Request);
                                // remember that the request has been completed.
                                requestCompleted = 1;
                            }
                        } else {
                            // Pass to port driver for handling
                            RequestDispatchUnknownRequests(
                                Device,
                                Request,
                                core::ptr::read(&RequestParameters),
                            );
                            // remember that the request has been completed.
                            requestCompleted = 1;
                        }
                    } else {
                        // If cached data exists, return cached data. Otherwise, fail the request.
                        if ((*inputBuffer).QueryType == PropertyStandardQuery)
                            && (((*inputBuffer).PropertyId == StorageDeviceProperty)
                                || ((*inputBuffer).PropertyId == StorageAdapterProperty))
                        {
                            status = RequestHandleQueryPropertyRetrieveCachedData(
                                deviceExtension,
                                Request,
                                &raw const RequestParameters,
                                &raw mut dataLength,
                            );
                        } else {
                            status = ntkrnl_sys::constants::STATUS_ACCESS_DENIED;
                        }
                    }
                }
            }

            processed = 1;
        } // end STORAGE_QUERY_PROPERTY

        IOCTL_MCN_SYNC_FAKE_IOCTL => {
            let irp: PIRP = WdfRequestWdmGetIrp(Request);

            if ((*deviceExtension).MediaChangeDetectionInfo).is_null() == false
                && (irp == (*(*deviceExtension).MediaChangeDetectionInfo).MediaChangeSyncIrp)
                && (WdfRequestGetRequestorMode(Request) == 0i8)
                && (RequestParameters.Parameters.Others.as_ref().Arg1 as usize == RequestSetupMcnSyncIrpAddr())
                && (RequestParameters.Parameters.Others.as_ref().Arg2 as usize == RequestSetupMcnSyncIrpAddr())
                && (RequestParameters.Parameters.Others.as_ref().Arg4 as usize == RequestSetupMcnSyncIrpAddr())
            {
                // This is the request we use to sync Media Change Detection with sequential queue.
                status = WdfDeviceEnqueueRequest(Device, Request);

                if !ntkrnl_sys::base::NT_SUCCESS(status) {
                    RequestCompletion(deviceExtension, Request, status, dataLength as CompletionInformation);
                }

                requestCompleted = 1;
                processed = 1;
            } else {
                // process as an unknown request.
                processed = 0;
            }
        } // end MCN case

        _ => {
            processed = 0;
        }
    } // end match ioctlCode

    if processed != 0 && requestCompleted == 0 {
        let currentStackLocationFlags: u8 = RequestGetCurrentStackLocationFlags(Request);

        if (status == STATUS_VERIFY_REQUIRED)
            && (currentStackLocationFlags & SL_OVERRIDE_VERIFY_VOLUME) != 0u8
        {
            // If the status is verified required and this request
            // should bypass verify required then retry the request.
            status = STATUS_IO_DEVICE_ERROR;
            UNREFERENCED_PARAMETER((status) as usize); // disables prefast warning; defensive coding...

            processed = RequestDispatchSpecialIoctls(Device, Request, RequestParameters);
        } else {
            RequestCompletion(deviceExtension, Request, status, dataLength as CompletionInformation);
        }
    }

    processed // return BOOLEAN
}
#[no_mangle]
pub unsafe extern "C" fn RequestDispatchUnknownRequests(
    Device: WDFDEVICE,
    Request: WDFREQUEST,
    RequestParameters: WDF_REQUEST_PARAMETERS,
) -> u8 {
    let mut status: NTSTATUS;
    let deviceExtension = DeviceGetExtension(Device);
    let baseCode = DEVICE_TYPE_FROM_CTL_CODE(
        RequestParameters.Parameters.DeviceIoControl.as_ref().IoControlCode,
    );
    if KeGetCurrentIrql() != PASSIVE_LEVEL || baseCode == FILE_DEVICE_ACPI {
        status = RequestHandleUnknownIoctl(Device, Request);
    } else {
        status = RequestSetContextFields(Request, Some(RequestHandleUnknownIoctl));
        if NT_SUCCESS(status) {
            status = RequestSynchronizeProcessWithSerialQueue(Device, Request);
        } else {
            RequestCompletion(deviceExtension, Request, status, 0);
        }
    }
    UNREFERENCED_PARAMETER((status) as usize);
    1u8
}
#[no_mangle]
pub unsafe extern "C" fn RequestProcessInternalDeviceControl(
    Request: WDFREQUEST,
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
) {
    const IRP_MJ_SCSI: u8 = 0x0f; // IRP_MJ_SCSI
    const IRP_MN_SCSI_CLASS: u8 = 0x01;
    let mut status: NTSTATUS;
    let mut irp: *mut IRP;
    let mut irpStack: *mut IO_STACK_LOCATION;
    let mut nextStack: *mut IO_STACK_LOCATION;
    let mut requestSent: u8 = 0;

    irp = WdfRequestWdmGetIrp(Request);
    irpStack = IoGetCurrentIrpStackLocation(irp);
    nextStack = IoGetNextIrpStackLocation(irp);

    (*nextStack).Parameters.Scsi.Srb = (*irpStack).Parameters.Scsi.Srb;
    (*nextStack).MajorFunction = IRP_MJ_SCSI;
    (*nextStack).MinorFunction = IRP_MN_SCSI_CLASS;

    WdfRequestSetCompletionRoutine(Request, core::mem::transmute::<unsafe extern "system" fn(WDFREQUEST, WDFIOTARGET, *mut core::ffi::c_void, *mut core::ffi::c_void), _>(RequestDummyCompletionRoutine), core::ptr::null_mut());

    status = RequestSend(
        DeviceExtension,
        Request,
        (*DeviceExtension).IoTarget,
        0,
        core::ptr::addr_of_mut!(requestSent),
    );

    if requestSent == 0 {
        RequestCompletion(
            DeviceExtension,
            Request,
            status,
            WdfRequestGetInformation(Request) as CompletionInformation,
        );
    }
}
#[no_mangle]
pub unsafe extern "system" fn SequentialQueueEvtIoReadWrite(
    Queue: WDFQUEUE,
    Request: WDFREQUEST,
    Length: usize,
) {
    let mut status: NTSTATUS;
    let device: WDFDEVICE = WdfIoQueueGetDevice(Queue);
    let deviceExtension: *mut CDROM_DEVICE_EXTENSION = DeviceGetExtension(device);
    let mut requestParameters: WDF_REQUEST_PARAMETERS = core::mem::zeroed();
    let cdData: *mut CDROM_DATA =
        core::ptr::addr_of_mut!((*deviceExtension).DeviceAdditionalData);
    let mut deviceFlags: ULONG = 0;
    let mut minorFunction: u8 = 0;
    let mut irpFlags: u8 = 0;

    CdromGapLog();
    // Get the request parameters
    WDF_REQUEST_PARAMETERS_INIT(&raw mut requestParameters);
    WdfRequestGetParameters(Request, &raw mut requestParameters);

    CdromDbgPrint3(
        b"[RWQ] type=%x len=%zx req=%p\n\0".as_ptr() as *const i8,
        requestParameters.Type as usize,
        Length,
        Request as usize,
    );
    CdromDbgPrint3(
        b"[RWQ2] rdlen=%zx rdoff_lo=%zx wdlen=%zx\n\0".as_ptr() as *const i8,
        requestParameters.Parameters.Read.as_ref().Length,
        requestParameters.Parameters.Read.as_ref().DeviceOffset as usize,
        Length,
    );

    // Check if a verify is required before a READ/WRITE
    CdromGetReadWriteVerifyState(
        deviceExtension,
        Request,
        &mut deviceFlags,
        &mut minorFunction,
        &mut irpFlags,
    );
    let doVerify = (deviceFlags & DO_VERIFY_VOLUME) != 0;
    let overrideVerifyVol = (irpFlags & SL_OVERRIDE_VERIFY_VOLUME) == 0;

    if doVerify
        && (minorFunction != CDROM_VOLUME_VERIFY_CHECKED)
        && overrideVerifyVol
    {
        //  DO_VERIFY_VOLUME is set for the device object,
        //  but this request is not itself a verify request.
        //  So fail this request.
        RequestCompletion(deviceExtension, Request, STATUS_VERIFY_REQUIRED, 0);
    } else {
        //  Since we've bypassed the verify-required tests we don't need to repeat
        //  them with this IRP - in particular we don't want to worry about
        //  hitting them at the partition 0 level if the request has come through
        //  a non-zero partition.
        CdromMarkVolumeVerifyChecked(Request);

        // Fail READ/WRITE requests when music is playing
        if (*deviceExtension).DeviceAdditionalData.PlayActive != 0 {
            RequestCompletion(deviceExtension, Request, STATUS_DEVICE_BUSY, 0);
            return;
        }

        // Fail READ/WRITE requests from non-owners if the drive is locked
        let fileObject = WdfRequestGetFileObject(Request);
        if !(*cdData).ExclusiveOwner.is_null() && (*cdData).ExclusiveOwner != fileObject {
            RequestCompletion(deviceExtension, Request, STATUS_ACCESS_DENIED, 0);
            return;
        }

        // Succeed READ/WRITE requests of length 0
        if Length == 0 {
            //  Several parts of the code turn 0 into 0xffffffff,
            //  so don't process a zero-length request any further.
            RequestCompletion(deviceExtension, Request, STATUS_SUCCESS, Length as CompletionInformation);
            return;
        }

        // If there is an unexpected write request, we want to rediscover MMC capabilities
        if (*deviceExtension).DeviceAdditionalData.Mmc.WriteAllowed == 0
            && (requestParameters.Type == WdfRequestTypeWrite)
        {
            // Schedule MMC capabilities update now, but perform it later in a work item
            (*deviceExtension).DeviceAdditionalData.Mmc.UpdateState = CdromMmcUpdateRequired as i32;
        }

        // If MMC capabilities update is required, we create a separate work item to avoid blocking
        // the current thread; otherwise, we initiate an async read/write in the current thread.
        if DeviceIsMmcUpdateRequired((*deviceExtension).Device) != 0 {
            (*deviceExtension).ReadWriteWorkItemContext.OriginalRequest = Request;
            WdfWorkItemEnqueue((*deviceExtension).ReadWriteWorkItem);
            status = STATUS_SUCCESS;
        } else {
            CdromDbgPrint3(
                b"[RWQ3] preval rdlen=%zx ptr=%p type=%x\n\0".as_ptr() as *const i8,
                requestParameters.Parameters.Read.as_ref().Length,
                core::ptr::addr_of!(requestParameters) as usize,
                requestParameters.Type as usize,
            );
            status = RequestValidateReadWrite(deviceExtension, Request, &mut requestParameters);
            if status >= 0 {
                // NT_SUCCESS
                status = RequestHandleReadWrite(deviceExtension, Request, &mut requestParameters);
            }
        }

        if status < 0 {
            // not NT_SUCCESS
            RequestCompletion(deviceExtension, Request, status, 0);
        }
    }

    return;
}
#[no_mangle]
pub unsafe extern "system" fn ReadWriteWorkItemRoutine(WorkItem: WDFWORKITEM) {
    let mut deviceExtension: *mut CDROM_DEVICE_EXTENSION;
    let mut readWriteRequest: WDFREQUEST;
    let mut readWriteRequestParameters: WDF_REQUEST_PARAMETERS = core::mem::zeroed();
    let mut status: NTSTATUS = STATUS_SUCCESS;

    PAGED_CODE!();

    deviceExtension = WdfObjectGetTypedContext(
        WdfWorkItemGetParentObject(WorkItem),
        &CDROM_DEVICE_EXTENSION_TYPE_INFO,
    ) as *mut CDROM_DEVICE_EXTENSION;
    readWriteRequest = (*deviceExtension).ReadWriteWorkItemContext.OriginalRequest;
    (*deviceExtension).ReadWriteWorkItemContext.OriginalRequest = core::ptr::null_mut();

    WDF_REQUEST_PARAMETERS_INIT(core::ptr::addr_of_mut!(readWriteRequestParameters));
    WdfRequestGetParameters(readWriteRequest, core::ptr::addr_of_mut!(readWriteRequestParameters));

    if DeviceIsMmcUpdateRequired((*deviceExtension).Device) != 0 {
        // Issue command to update the drive capabilities.
        // The failure of MMC update is not considered critical, so we'll
        // continue to process the request even if MMC update fails.
        let _ = DeviceUpdateMmcCapabilities((*deviceExtension).Device);
    }

    // Now verify and process the request
    if NT_SUCCESS(status) {
        status = RequestValidateReadWrite(deviceExtension, readWriteRequest, &mut readWriteRequestParameters);
    }
    if NT_SUCCESS(status) {
        status = RequestHandleReadWrite(deviceExtension, readWriteRequest, &mut readWriteRequestParameters);
    }

    // Complete the request immediately on failure
    if !NT_SUCCESS(status) {
        RequestCompletion(deviceExtension, readWriteRequest, status, 0);
    }
}
#[no_mangle]
pub unsafe extern "system" fn SequentialQueueEvtIoDeviceControl(
    Queue: WDFQUEUE,
    Request: WDFREQUEST,
    OutputBufferLength: usize,
    InputBufferLength: usize,
    IoControlCode: ULONG,
) {
    let mut status: NTSTATUS;
    let device: WDFDEVICE = WdfIoQueueGetDevice(Queue);
    let deviceExtension: PCDROM_DEVICE_EXTENSION = DeviceGetExtension(device);
    let requestContext: PCDROM_REQUEST_CONTEXT = RequestGetContext(Request as *mut core::ffi::c_void);
    let cdData: *mut CDROM_DATA = core::ptr::addr_of_mut!((*deviceExtension).DeviceAdditionalData);
    let mut requestParameters: core::mem::MaybeUninit<WDF_REQUEST_PARAMETERS> = core::mem::MaybeUninit::uninit();

    CdromDbgPrint1(b"[IOCTL-IN] code=%lx\n\0".as_ptr() as *const i8, IoControlCode as usize);

    UNREFERENCED_PARAMETER((OutputBufferLength as i32) as usize);
    UNREFERENCED_PARAMETER((InputBufferLength as i32) as usize);
    UNREFERENCED_PARAMETER((IoControlCode as i32) as usize);

    // get the request parameters
    WDF_REQUEST_PARAMETERS_INIT(requestParameters.as_mut_ptr());
    WdfRequestGetParameters(Request, requestParameters.as_mut_ptr());

    // If the device is in exclusive mode, check whether the request is from
    // the handle that locked the device
    let fileObject: WDFFILEOBJECT = WdfRequestGetFileObject(Request);
    if !(*cdData).ExclusiveOwner.is_null() && (*cdData).ExclusiveOwner != fileObject {
        let mut isBlocked: BOOLEAN = 0; // FALSE

        status = RequestIsIoctlBlockedByExclusiveAccess(Request, &mut isBlocked as *mut BOOLEAN);
        if ntkrnl_sys::base::NT_SUCCESS(status) && isBlocked != 0 {
            if (*requestContext).SyncRequired != 0 {
                // set the following event, so RequestSynchronizeProcessWithSerialQueue() can continue run to process the real request.
                // this function will wait for the request process finishes.
                ntkrnl_sys::ke::KeSetEvent((*requestContext).SyncEvent as *mut c_void, IO_CD_ROM_INCREMENT, 0);
            } else {
                TracePrint!(
                    TRACE_LEVEL_ERROR,
                    TRACE_FLAG_IOCTL,
                    b"DeviceEvtIoInCallerContext: Access Denied! Device in exclusive mode.Failing Ioctl %lx\n\0".as_ptr() as *const i8,
                    (*requestParameters.as_mut_ptr()).Parameters.DeviceIoControl.as_ref().IoControlCode,
                );
                RequestCompletion(deviceExtension, Request, ntkrnl_sys::constants::STATUS_ACCESS_DENIED, 0);
            }
            return;
        }
    }

    if (*cdData).Mmc.WriteAllowed == 0
        && ((*requestParameters.as_mut_ptr()).Parameters.DeviceIoControl.as_ref().IoControlCode == IOCTL_DISK_IS_WRITABLE
            || (*requestParameters.as_mut_ptr()).Parameters.DeviceIoControl.as_ref().IoControlCode == IOCTL_DISK_VERIFY)
    {
        (*cdData).Mmc.UpdateState = CdromMmcUpdateRequired as i32;
    }

    if (*requestContext).SyncRequired == 0
        && CdromTryHandleSerializedStorageControl(deviceExtension, Request, requestParameters.as_mut_ptr()) != 0
    {
        return;
    }

    // check if this is a synchronized ioctl
    if (*requestContext).SyncRequired != 0 {
        // set the following event, so RequestSynchronizeProcessWithSerialQueue() can continue run to process the real request.
        // this function will wait for the request process finishes.
        ntkrnl_sys::ke::KeSetEvent((*requestContext).SyncEvent as *mut c_void, IO_CD_ROM_INCREMENT, 0);
    } else {
        (*deviceExtension).IoctlWorkItemContext.OriginalRequest = Request;

        // all other IOCTL processing is currently processed via a
        // work item running at PASSIVE_LEVEL.
        WdfWorkItemEnqueue((*deviceExtension).IoctlWorkItem);
    }

    return;
}
#[no_mangle]
pub unsafe extern "system" fn IoctlWorkItemRoutine(WorkItem: WDFWORKITEM) {
    let mut deviceExtension: *mut CDROM_DEVICE_EXTENSION;

    if KeGetCurrentIrql() > APC_LEVEL {
        DbgBreakPoint();
    }

    deviceExtension = WdfObjectGetTypedContext(
        WdfWorkItemGetParentObject(WorkItem),
        &CDROM_DEVICE_EXTENSION_TYPE_INFO,
    ) as *mut CDROM_DEVICE_EXTENSION;

    let mut wi_freq: i64 = 0;
    let wi_t0 = KeQueryPerformanceCounter(core::ptr::addr_of_mut!(wi_freq));
    let mmc_required = DeviceIsMmcUpdateRequired((*deviceExtension).Device);
    if mmc_required != 0 {
        DeviceUpdateMmcCapabilities((*deviceExtension).Device);
    }
    let wi_t1 = KeQueryPerformanceCounter(core::ptr::null_mut());

    RequestProcessSerializedIoctl(
        deviceExtension,
        (*deviceExtension).IoctlWorkItemContext.OriginalRequest,
    );
    let wi_t2 = KeQueryPerformanceCounter(core::ptr::null_mut());
    let mmc_us = if wi_freq != 0 { (wi_t1 - wi_t0).wrapping_mul(1_000_000) / wi_freq } else { 0 };
    let ioctl_us = if wi_freq != 0 { (wi_t2 - wi_t1).wrapping_mul(1_000_000) / wi_freq } else { 0 };
    if mmc_us > 200_000 || ioctl_us > 200_000 {
        CdromDbgPrint3(
            b"[IOCTL-WI] mmcReq=%lx mmc_us=%lx ioctl_us=%lx\n\0".as_ptr() as *const i8,
            mmc_required as usize, mmc_us as usize, ioctl_us as usize,
        );
    }
}
#[no_mangle]
 pub unsafe extern "C" fn RequestProcessSerializedIoctl(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Request: WDFREQUEST,
) -> i32 {
    let mut status: i32;
    let mut information: usize = 0;
    let mut requestParameters: WDF_REQUEST_PARAMETERS = core::mem::zeroed();
    let mut completeRequest: i32 = 1;
    PAGED_CODE!();
    WDF_REQUEST_PARAMETERS_INIT(&mut requestParameters);
    WdfRequestGetParameters(Request, &mut requestParameters);
    NormalizeIoctl(&mut requestParameters);
    if CdromTryHandleSerializedStorageControl(DeviceExtension, Request, &mut requestParameters) != 0 {
        return STATUS_SUCCESS;
    }
    let ioctlCode = requestParameters.Parameters.DeviceIoControl.as_ref().IoControlCode;
    match ioctlCode {
        IOCTL_CDROM_READ_TOC | IOCTL_CDROM_GET_LAST_SESSION => {
            status = RequestHandleReadTOC(DeviceExtension, Request, requestParameters, &mut information);
        }
        IOCTL_CDROM_READ_TOC_EX => {
            status = RequestHandleReadTocEx(DeviceExtension, Request, requestParameters, &mut information);
        }
        IOCTL_CDROM_GET_CONFIGURATION => {
            status = RequestHandleGetConfiguration(DeviceExtension, Request, requestParameters, &mut information);
        }
        IOCTL_CDROM_RAW_READ => {
            status = DeviceHandleRawRead(DeviceExtension, Request, requestParameters, &mut information);
        }
        IOCTL_DISK_GET_LENGTH_INFO
        | IOCTL_DISK_GET_DRIVE_GEOMETRY
        | IOCTL_DISK_GET_DRIVE_GEOMETRY_EX
        | IOCTL_CDROM_GET_DRIVE_GEOMETRY
        | IOCTL_CDROM_GET_DRIVE_GEOMETRY_EX
        | IOCTL_STORAGE_READ_CAPACITY => {
            status = RequestHandleGetDriveGeometry(DeviceExtension, Request, requestParameters, &mut information);
        }
        IOCTL_DISK_VERIFY => {
            status = RequestHandleDiskVerify(DeviceExtension, Request, requestParameters, &mut information);
        }
        IOCTL_STORAGE_CHECK_VERIFY | IOCTL_STORAGE_CHECK_VERIFY2 => {
            status = RequestHandleCheckVerify(DeviceExtension, Request, requestParameters, &mut information);
        }
        IOCTL_DISK_GET_DRIVE_LAYOUT
        | IOCTL_DISK_GET_DRIVE_LAYOUT_EX
        | IOCTL_DISK_GET_PARTITION_INFO
        | IOCTL_DISK_GET_PARTITION_INFO_EX => {
            status = RequestHandleFakePartitionInfo(DeviceExtension, Request, requestParameters, &mut information);
        }
        IOCTL_DISK_IS_WRITABLE => {
            if (*(DeviceExtension)).DeviceAdditionalData.Mmc.WriteAllowed != 0
                && (*(DeviceExtension)).DeviceAdditionalData.Mmc.ValidationSchema == FeatureDefectManagement
            {
                status = 0;
            } else {
                status = STATUS_MEDIA_WRITE_PROTECTED as i32;
            }
            information = 0;
        }
        IOCTL_CDROM_PLAY_AUDIO_MSF => {
            status = DeviceHandlePlayAudioMsf(DeviceExtension, Request, requestParameters, &mut information);
        }
        IOCTL_CDROM_READ_Q_CHANNEL => {
            status = DeviceHandleReadQChannel(DeviceExtension, Request, requestParameters, &mut information);
        }
        IOCTL_CDROM_PAUSE_AUDIO => {
            status = DeviceHandlePauseAudio(DeviceExtension, Request, &mut information);
        }
        IOCTL_CDROM_RESUME_AUDIO => {
            status = DeviceHandleResumeAudio(DeviceExtension, Request, &mut information);
        }
        IOCTL_CDROM_SEEK_AUDIO_MSF => {
            status = DeviceHandleSeekAudioMsf(DeviceExtension, Request, requestParameters, &mut information);
        }
        IOCTL_CDROM_STOP_AUDIO => {
            status = DeviceHandleStopAudio(DeviceExtension, Request, &mut information);
        }
        IOCTL_CDROM_GET_VOLUME | IOCTL_CDROM_SET_VOLUME => {
            status = DeviceHandleGetSetVolume(DeviceExtension, Request, requestParameters, &mut information);
        }
        IOCTL_DVD_GET_REGION => {
            status = RequestHandleGetDvdRegion(DeviceExtension, Request, &mut information);
        }
        IOCTL_DVD_READ_STRUCTURE => {
            status = DeviceHandleReadDvdStructure(DeviceExtension, Request, requestParameters, &mut information);
        }
        IOCTL_DVD_END_SESSION => {
            status = DeviceHandleDvdEndSession(DeviceExtension, Request, requestParameters, &mut information);
        }
        IOCTL_DVD_START_SESSION | IOCTL_DVD_READ_KEY => {
            status = DeviceHandleDvdStartSessionReadKey(DeviceExtension, Request, requestParameters, &mut information);
        }
        IOCTL_DVD_SEND_KEY | IOCTL_DVD_SEND_KEY2 => {
            status = DeviceHandleDvdSendKey(DeviceExtension, Request, requestParameters, &mut information);
        }
        IOCTL_STORAGE_SET_READ_AHEAD => {
            status = DeviceHandleSetReadAhead(DeviceExtension, Request, requestParameters, &mut information);
        }
        IOCTL_CDROM_SET_SPEED => {
            status = DeviceHandleSetSpeed(DeviceExtension, Request, requestParameters, &mut information);
        }
        IOCTL_AACS_READ_MEDIA_KEY_BLOCK_SIZE | IOCTL_AACS_READ_MEDIA_KEY_BLOCK => {
            status = DeviceHandleAacsReadMediaKeyBlock(DeviceExtension, Request, requestParameters, &mut information);
        }
        IOCTL_AACS_START_SESSION => {
            status = DeviceHandleAacsStartSession(DeviceExtension, Request, requestParameters, &mut information);
        }
        IOCTL_AACS_END_SESSION => {
            status = DeviceHandleAacsEndSession(DeviceExtension, Request, requestParameters, &mut information);
        }
        IOCTL_AACS_SEND_CERTIFICATE => {
            status = DeviceHandleAacsSendCertificate(DeviceExtension, Request, requestParameters, &mut information);
        }
        IOCTL_AACS_GET_CERTIFICATE => {
            status = DeviceHandleAacsGetCertificate(DeviceExtension, Request, requestParameters, &mut information);
        }
        IOCTL_AACS_GET_CHALLENGE_KEY => {
            status = DeviceHandleAacsGetChallengeKey(DeviceExtension, Request, requestParameters, &mut information);
        }
        IOCTL_AACS_SEND_CHALLENGE_KEY => {
            status = DeviceHandleSendChallengeKey(DeviceExtension, Request, requestParameters, &mut information);
        }
        IOCTL_AACS_READ_VOLUME_ID => {
            status = DeviceHandleReadVolumeId(DeviceExtension, Request, requestParameters, &mut information);
        }
        IOCTL_AACS_READ_SERIAL_NUMBER => {
            status = DeviceHandleAacsReadSerialNumber(DeviceExtension, Request, requestParameters, &mut information);
        }
        IOCTL_AACS_READ_MEDIA_ID => {
            status = DeviceHandleAacsReadMediaId(DeviceExtension, Request, requestParameters, &mut information);
        }
        IOCTL_AACS_READ_BINDING_NONCE => {
            status = DeviceHandleAacsReadBindingNonce(DeviceExtension, Request, requestParameters, &mut information);
        }
        IOCTL_AACS_GENERATE_BINDING_NONCE => {
            status = DeviceHandleAacsGenerateBindingNonce(DeviceExtension, Request, requestParameters, &mut information);
        }
        IOCTL_CDROM_ENABLE_STREAMING => {
            status = RequestHandleEnableStreaming(DeviceExtension, Request, &mut information);
        }
        IOCTL_CDROM_SEND_OPC_INFORMATION => {
            status = RequestHandleSendOpcInformation(DeviceExtension, Request, &mut information);
        }
        IOCTL_CDROM_GET_PERFORMANCE => {
            status = RequestHandleGetPerformance(DeviceExtension, Request, requestParameters, &mut information);
        }
        IOCTL_MCN_SYNC_FAKE_IOCTL => {
            status = RequestHandleMcnSyncFakeIoctl(DeviceExtension, &mut information);
        }
        IOCTL_STORAGE_MEDIA_REMOVAL | IOCTL_STORAGE_EJECTION_CONTROL => {
            status = RequestHandleEjectionControl(DeviceExtension, Request, requestParameters, &mut information);
        }
        IOCTL_STORAGE_EJECT_MEDIA | IOCTL_STORAGE_LOAD_MEDIA | IOCTL_STORAGE_LOAD_MEDIA2 => {
            status = RequestHandleLoadEjectMedia(DeviceExtension, Request, requestParameters, &mut information);
        }
        IOCTL_STORAGE_MCN_CONTROL => {
            status = RequestHandleMcnControl(DeviceExtension, Request, &mut information);
        }
        IOCTL_STORAGE_RESERVE | IOCTL_STORAGE_RELEASE => {
            status = RequestHandleReserveRelease(DeviceExtension, Request, requestParameters, &mut information);
        }
        IOCTL_STORAGE_PERSISTENT_RESERVE_IN | IOCTL_STORAGE_PERSISTENT_RESERVE_OUT => {
            status = RequestHandlePersistentReserve(DeviceExtension, Request, requestParameters, &mut information);
        }
        #[cfg(windows)]
        IOCTL_DISK_ARE_VOLUMES_READY => {
            status = RequestHandleAreVolumesReady(DeviceExtension, Request, requestParameters, &mut information);
            completeRequest = 0;
        }
        #[cfg(windows)]
        IOCTL_VOLUME_ONLINE | IOCTL_VOLUME_POST_ONLINE => {
            status = RequestHandleVolumeOnline(DeviceExtension, Request, requestParameters, &mut information);
        }
        _ => {
            status = STATUS_ACCESS_DENIED as i32;
        }
    }
    if completeRequest != 0 {
        RequestCompletion(DeviceExtension, Request, status, information as CompletionInformation);
    }
    status
}
#[no_mangle]
pub unsafe extern "system" fn SequentialQueueEvtCanceledOnQueue(
    Queue: *mut c_void,
    Request: *mut c_void,
) {
    let requestContext: PCDROM_REQUEST_CONTEXT = RequestGetContext(Request);
    if (*requestContext).SyncRequired != 0 {
        KeSetEvent((*requestContext).SyncEvent as *mut c_void, 1, 0); // IO_CD_ROM_INCREMENT, FALSE
    } else {
        let mut deviceExtension: PCDROM_DEVICE_EXTENSION;
        let device: WDFDEVICE = WdfIoQueueGetDevice(Queue as WDFQUEUE);
        deviceExtension = DeviceGetExtension(device);
        RequestCompletion(
            deviceExtension,
            Request as WDFREQUEST,
            0xC0000120u32 as i32, // STATUS_CANCELLED
            0,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn RequestSynchronizeProcessWithSerialQueue(
    Device: WDFDEVICE,
    Request: WDFREQUEST,
) -> NTSTATUS {
    // Local variables, preserving original names
    let mut status: NTSTATUS;
    let deviceExtension: *mut CDROM_DEVICE_EXTENSION = DeviceGetExtension(Device);
    let requestContext: *mut CDROM_REQUEST_CONTEXT = RequestGetContext(Request as *mut core::ffi::c_void);
    let bufferToFree: *mut KEVENT = (*requestContext).SyncEvent;

    let mut sync_freq: i64 = 0;
    let sync_t0 = KeQueryPerformanceCounter(core::ptr::addr_of_mut!(sync_freq));

    // KeGetCurrentIrql returns KIRQL (u8); DISPATCH_LEVEL is a constant
    if KeGetCurrentIrql() >= DISPATCH_LEVEL {
        TracePrint!(
            2 /* TRACE_LEVEL_ERROR */,
            0 /* TRACE_FLAG_IOCTL */,
            b"RequestSynchronousProcessWithSerialQueue called at DISPATCH_LEVEL or above\0".as_ptr() as *const i8,
        );
        CdromRequestCompletion(deviceExtension, Request, STATUS_INVALID_LEVEL, 0);
        FREE_POOL(bufferToFree as *mut core::ffi::c_void);
        return STATUS_INVALID_LEVEL;
    }

    // Initialize the synchronization event
    KeInitializeEvent((*requestContext).SyncEvent, _EVENT_TYPE_NotificationEvent, 0 /* FALSE */);

    // Enqueue the request via KMDF
    status = WdfDeviceEnqueueRequest(Device, Request);

    if !NT_SUCCESS(status) {
        // Simulate sync event already occurred, then complete
        KeSetEvent((*requestContext).SyncEvent as *mut c_void, IO_CD_ROM_INCREMENT as i32, 0 /* FALSE */);
        CdromRequestCompletion(
            deviceExtension,
            Request,
            status,
            WdfRequestGetInformation(Request) as ULONG_PTR,
        );
        FREE_POOL(bufferToFree as *mut core::ffi::c_void);
        return status;
    } else {
        let mut waitStatus: NTSTATUS = STATUS_UNSUCCESSFUL;
        let cdData: *mut CDROM_DATA = &mut (*deviceExtension).DeviceAdditionalData as *mut _;
        let mut fCallSyncCallback: BOOLEAN;
        let irp: PIRP = WdfRequestWdmGetIrp(Request);

        // wait loop – exact replication of C control flow
        while waitStatus != STATUS_SUCCESS {
            waitStatus = KeWaitForSingleObject(
                (*requestContext).SyncEvent as *mut c_void,
                _KWAIT_REASON_Executive,
                0i8,
                1, // TRUE (Alertable)
                core::ptr::null_mut(),
            );

            if waitStatus == STATUS_SUCCESS {
                // do nothing
            } else if waitStatus != STATUS_ALERTED {
                // non-alerted, non-success error
                TracePrint!(
                    1 /* TRACE_LEVEL_FATAL */,
                    0 /* TRACE_FLAG_IOCTL */,
                    b"Request %p on device object %p had a non-alert, non-success result from wait (%!HRESULT!)\n\0"
                        .as_ptr() as *const i8,
                    Request as *const core::ffi::c_void,
                    Device as *const core::ffi::c_void,
                    waitStatus,
                );
                status = waitStatus;
                CdromRequestCompletion(deviceExtension, Request, status, 0);
                FREE_POOL(bufferToFree as *mut core::ffi::c_void);
                return status;
            } else if PsIsThreadTerminating(PsGetCurrentThread()) != 0 {
                // Thread alerted and terminating – cancel IRP
                if IoCancelIrp(irp) != 0 {
                    TracePrint!(
                        3 /* TRACE_LEVEL_WARNING */,
                        0 /* TRACE_FLAG_IOCTL */,
                        b"Sychronize Ioctl: request %p cancelled from device %p\n\0".as_ptr()
                            as *const i8,
                        Request as *const core::ffi::c_void,
                        Device as *const core::ffi::c_void,
                    );
                } else {
                    TracePrint!(
                        3 /* TRACE_LEVEL_WARNING */,
                        0 /* TRACE_FLAG_IOCTL */,
                        b"Sychronize Ioctl: request %p could not be cancelled from device %p\n\0"
                            .as_ptr() as *const i8,
                        Request as *const core::ffi::c_void,
                        Device as *const core::ffi::c_void,
                    );
                }
            } else {
                TracePrint!(
                    3 /* TRACE_LEVEL_WARNING */,
                    0 /* TRACE_FLAG_IOCTL */,
                    b"SPURIOUS ALERT waiting for Request %p on device %p (%!STATUS!)\n\0".as_ptr()
                        as *const i8,
                    Request as *const core::ffi::c_void,
                    Device as *const core::ffi::c_void,
                    status,
                );
            }
        } // end while

        {
            let sync_t1 = KeQueryPerformanceCounter(core::ptr::null_mut());
            let us = if sync_freq != 0 { (sync_t1 - sync_t0).wrapping_mul(1_000_000) / sync_freq } else { 0 };
            if us > 500_000 {
                let mut rp: WDF_REQUEST_PARAMETERS = core::mem::zeroed();
                WDF_REQUEST_PARAMETERS_INIT(&mut rp);
                WdfRequestGetParameters(Request, &mut rp);
                CdromDbgPrintForce2(
                    b"[SYNCQ] blocked us=%lx ioctl=%lx\n\0".as_ptr() as *const i8,
                    us as usize,
                    rp.Parameters.DeviceIoControl.as_ref().IoControlCode as usize,
                );
            }
        }

        // Check cancellation or exclusive access
        if WdfRequestIsCanceled(Request) != 0 {
            status = STATUS_CANCELLED;
            CdromRequestCompletion(deviceExtension, Request, status, 0);
            FREE_POOL(bufferToFree as *mut core::ffi::c_void);
            return status;
        } else {
            let fileObject = WdfRequestGetFileObject(Request);
            if !(*cdData).ExclusiveOwner.is_null() && (*cdData).ExclusiveOwner != fileObject {
            let mut requestParameters: WDF_REQUEST_PARAMETERS = core::mem::zeroed();
            let mut isBlocked: BOOLEAN = 0; // FALSE

            WDF_REQUEST_PARAMETERS_INIT(&mut requestParameters);
            WdfRequestGetParameters(Request, &mut requestParameters);

            status = RequestIsIoctlBlockedByExclusiveAccess(Request, &mut isBlocked);
            if isBlocked != 0 {
                TracePrint!(
                    2 /* TRACE_LEVEL_ERROR */,
                    0 /* TRACE_FLAG_IOCTL */,
                    b"Access Denied! Device in exclusive mode.Failing Ioctl %lx\n\0".as_ptr()
                        as *const i8,
                    requestParameters.Parameters.DeviceIoControl.as_ref().IoControlCode,
                );
                CdromRequestCompletion(deviceExtension, Request, STATUS_ACCESS_DENIED, 0);
                FREE_POOL(bufferToFree as *mut core::ffi::c_void);
                return STATUS_ACCESS_DENIED;
            } else {
                TracePrint!(
                    4 /* TRACE_LEVEL_INFORMATION */,
                    0 /* TRACE_FLAG_IOCTL */,
                    b"Ioctl %lx not blocked by cdrom being in exclusive mode\n\0".as_ptr()
                        as *const i8,
                    requestParameters.Parameters.DeviceIoControl.as_ref().IoControlCode,
                );
                fCallSyncCallback = 1; // TRUE
            }
            } else {
                fCallSyncCallback = 1; // TRUE
            }
        }

        if fCallSyncCallback != 0 {
            status = CdromInvokeSyncCallback(requestContext, Device, Request);
            if !NT_SUCCESS(status) {
                // The callback completes the request itself in every path (incl.
                // failure); do NOT complete again here (was a double-complete of
                // an already-completed WDF request). Found by rustconv semdiff.
                FREE_POOL(bufferToFree as *mut core::ffi::c_void);
                return status;
            }
        }
    }

    // Free the allocation for the sync event
    FREE_POOL(bufferToFree as *mut core::ffi::c_void);

    status
}
#[no_mangle]
pub unsafe extern "C" fn RequestIsIoctlBlockedByExclusiveAccess(
    Request: WDFREQUEST,
    IsBlocked: PBOOLEAN,
) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_SUCCESS;
    let mut ioctlCode: ULONG;
    let mut baseCode: ULONG;
    let mut requestParameters: WDF_REQUEST_PARAMETERS = core::mem::zeroed();

    // Get the Request parameters
    WDF_REQUEST_PARAMETERS_INIT(&mut requestParameters);
    WdfRequestGetParameters(Request, &mut requestParameters);

    // check and initialize parameter
    if IsBlocked.is_null() {
        // This is an internal function and this parameter must be supplied.
        NT_ASSERT(0);
        return STATUS_INVALID_PARAMETER;
    } else {
        *IsBlocked = 0; // FALSE
    }

    // check if this is an IOCTL
    if requestParameters.Type == WdfRequestTypeDeviceControl
        || requestParameters.Type == WdfRequestTypeDeviceControlInternal
    {
        //
        // Allow minimum set of commands that are required for the disk manager
        // to show the CD device, while in exclusive mode.
        // Note: These commands should not generate any requests to the device,
        //       and thus must be handled directly in StartIO during exclusive
        //       access (except for the exclusive owner, of course).
        //
        ioctlCode = requestParameters.Parameters.DeviceIoControl.as_ref().IoControlCode;
        baseCode = (ioctlCode >> 16) & 0xFFFF; // DEVICE_TYPE_FROM_CTL_CODE

        if ioctlCode == 0x41018 // IOCTL_SCSI_GET_ADDRESS
            || ioctlCode == 0x2D0C14 // IOCTL_STORAGE_GET_HOTPLUG_INFO
            || ioctlCode == 0x2D1080 // IOCTL_STORAGE_GET_DEVICE_NUMBER
            || ioctlCode == 0x2D0C04 // IOCTL_STORAGE_GET_MEDIA_TYPES_EX
            || ioctlCode == 0x2C05C // IOCTL_CDROM_EXCLUSIVE_ACCESS
            || ioctlCode == 0x24064 // IOCTL_CDROM_GET_INQUIRY_DATA
        {
            *IsBlocked = 0;
        }
        //
        // Handle IOCTL_STORAGE_QUERY_PROPERTY special because:
        //  (1) PropertyExistsQuery should not generate device i/o
        //  (2) Queries for StorageDeviceProperty and StorageAdapterDescriptor
        //      will return cache'd data
        else if ioctlCode == 0x2D1400 /* IOCTL_STORAGE_QUERY_PROPERTY */ {
            let mut query: PSTORAGE_PROPERTY_QUERY = core::ptr::null_mut();
            status = WdfRequestRetrieveInputBuffer(
                Request,
                requestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
                (&mut query) as *mut _ as *mut *mut core::ffi::c_void,
                core::ptr::null_mut(),
            );

            if status >= 0 { // NT_SUCCESS(status)
                if !query.is_null() {
                    if (*query).QueryType == PropertyExistsQuery {
                        *IsBlocked = 0;
                    } else if (*query).QueryType == PropertyStandardQuery
                        && ((*query).PropertyId == StorageDeviceProperty
                            || (*query).PropertyId == StorageAdapterProperty)
                    {
                        *IsBlocked = 0;
                    }
                }
            }
        }
        // Return TRUE for unknown IOCTLs with STORAGE bases
        else if baseCode == 0x4 // IOCTL_SCSI_BASE = FILE_DEVICE_CONTROLLER
            || baseCode == 0x7 // IOCTL_DISK_BASE
            || baseCode == 0x2 // IOCTL_CDROM_BASE
            || baseCode == 0x2D // IOCTL_STORAGE_BASE
            || baseCode == 0x33 // IOCTL_DVD_BASE = FILE_DEVICE_DVD
        {
            *IsBlocked = 1; // TRUE
        }
    } else {
        // this should only be called with an IOCTL
        NT_ASSERT(0);
        status = STATUS_INVALID_PARAMETER;
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn DeviceIsMmcUpdateRequired(Device: WDFDEVICE) -> u8 {
    let deviceExtension: *mut CDROM_DEVICE_EXTENSION = DeviceGetExtension(Device);
    let cdData: *mut CDROM_DATA = core::ptr::addr_of_mut!((*deviceExtension).DeviceAdditionalData);
    if ((*cdData).Mmc.IsMmc != 0) && ((*cdData).Mmc.UpdateState == CdromMmcUpdateRequired as i32) {
        return 1;
    } else {
        return 0;
    }
}
#[no_mangle]
pub unsafe extern "system" fn RequestEvtCleanup(request: *mut core::ffi::c_void) {
    // WDFREQUEST request = (WDFREQUEST)Request;  — parameter is already the right type.
    let requestContext = RequestGetContext(request);
    if (*requestContext).SyncRequired != 0 {
        // NT_ASSERT(KeReadStateEvent(requestContext->SyncEvent) != 0)
        ntkrnl_sys::ASSERT!(KeReadStateEvent((*requestContext).SyncEvent as *mut ntkrnl_sys::ddk::_KEVENT) != 0);
    }
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
//     cdrom.c
//
// Abstract:
//
//     The CDROM class driver tranlates IRPs to SRBs with embedded CDBs
//     and sends them to its devices through the port driver.
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
// // this definition is used to link _StorDebugPrint() function.
// #define DEBUG_MAIN_SOURCE   1
//
//
// #include "ntddk.h"
// #include "ntstrsafe.h"
//
// #include "ntddstor.h"
// #include "ntddtape.h"
// #include "wdfcore.h"
// #include "devpkey.h"
//
// #include "cdrom.h"
// #include "ioctl.h"
// #include "mmc.h"
// #include "scratch.h"
//
//
// #ifdef DEBUG_USE_WPP
// #include "cdrom.tmh"
// #endif
//
// BOOLEAN
// BootEnvironmentIsWinPE(
//     VOID
//     );
//
// #ifdef ALLOC_PRAGMA
//
// #pragma alloc_text(INIT, DriverEntry)
// #pragma alloc_text(INIT, BootEnvironmentIsWinPE)
//
// #pragma alloc_text(PAGE, DriverEvtCleanup)
// #pragma alloc_text(PAGE, DriverEvtDeviceAdd)
// #pragma alloc_text(PAGE, DeviceEvtCleanup)
// #pragma alloc_text(PAGE, DeviceEvtSelfManagedIoCleanup)
// #pragma alloc_text(PAGE, DeviceEvtD0Exit)
// #pragma alloc_text(PAGE, CreateQueueEvtIoDefault)
// #pragma alloc_text(PAGE, DeviceEvtFileClose)
// #pragma alloc_text(PAGE, DeviceCleanupProtectedLocks)
// #pragma alloc_text(PAGE, DeviceCleanupDisableMcn)
// #pragma alloc_text(PAGE, RequestProcessSerializedIoctl)
// #pragma alloc_text(PAGE, ReadWriteWorkItemRoutine)
// #pragma alloc_text(PAGE, IoctlWorkItemRoutine)
// #pragma alloc_text(PAGE, DeviceEvtSurpriseRemoval)
//
// #endif
//
// NTSTATUS
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// DriverEntry(
//      PDRIVER_OBJECT  DriverObject,
//      PUNICODE_STRING RegistryPath
//     )
// /*++
//
// Routine Description:
//
//     Installable driver initialization entry point.
//     This entry point is called directly by the I/O system.
//
// Arguments:
//
//     DriverObject - pointer to the driver object
//
//     RegistryPath - pointer to a unicode string representing the path,
//                    to driver-specific key in the registry.
//
// Return Value:
//
//     STATUS_SUCCESS if successful,
//     STATUS_UNSUCCESSFUL otherwise.
//
// --*/
// {
//     NTSTATUS                status;
//     WDF_DRIVER_CONFIG       config;
//     WDF_OBJECT_ATTRIBUTES   attributes;
//     WDFDRIVER               driverObject = NULL;
//
//     PAGED_CODE();
//
//     // Initialize WPP Tracing
//     WPP_INIT_TRACING(DriverObject, RegistryPath);
//
//     TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_INIT,
//                 "CDROM.SYS DriverObject %p loading\n",
//                 DriverObject));
//
//     // Register DeviceAdd and DriverEvtCleanup callback.
//     // WPP_CLEANUP will be called in DriverEvtCleanup
//     WDF_OBJECT_ATTRIBUTES_INIT_CONTEXT_TYPE(&attributes, CDROM_DRIVER_EXTENSION);
//     attributes.EvtCleanupCallback = DriverEvtCleanup;
//
//     WDF_DRIVER_CONFIG_INIT(&config, DriverEvtDeviceAdd);
//
//     status = WdfDriverCreate(DriverObject,
//                              RegistryPath,
//                              &attributes,
//                              &config,
//                              &driverObject);
//
//     if (!NT_SUCCESS(status))
//     {
//         TracePrint!((TRACE_LEVEL_FATAL, TRACE_FLAG_INIT,
//                     "WdfDriverCreate failed. %x\n",
//                     status));
//
//         // Cleanup tracing here because DriverUnload will not be called
//         // as we have failed to create WDFDRIVER object itself.
//         WPP_CLEANUP(DriverObject);
//
//     }
//     else
//     {
//         PCDROM_DRIVER_EXTENSION driverExtension = DriverGetExtension(driverObject);
//
//         // Copy the registry path into the driver extension so we can use it later
//         driverExtension->Version = 0x01;
//         driverExtension->DriverObject = DriverObject;
//
//         if (BootEnvironmentIsWinPE()) {
//
//             SET_FLAG(driverExtension->Flags, CDROM_FLAG_WINPE_MODE);
//         }
//
//     }
//
//     return status;
// }
//
//
// BOOLEAN
// BootEnvironmentIsWinPE(
//     VOID
//     )
// /*++
//
// Routine Description:
//
//     This routine determines if the boot enviroment is WinPE
//
// Arguments:
//
//     None
//
// Return Value:
//
//     BOOLEAN - TRUE if the environment is WinPE; FALSE otherwise
//
// --*/
// {
//     NTSTATUS    status;
//     WDFKEY      registryKey = NULL;
//
//     DECLARE_CONST_UNICODE_STRING(registryKeyName, WINPE_REG_KEY_NAME);
//
//     PAGED_CODE();
//
//     status = WdfRegistryOpenKey(NULL,
//                                 &registryKeyName,
//                                 KEY_READ,
//                                 WDF_NO_OBJECT_ATTRIBUTES,
//                                 &registryKey);
//
//     if (!NT_SUCCESS(status))
//     {
//         return FALSE;
//     }
//
//     WdfRegistryClose(registryKey);
//     return TRUE;
// } // end BootEnvironmentIsWinPE()
//
//
// VOID
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// DriverEvtCleanup(
//      WDFOBJECT Driver
//     )
// /*++
// Routine Description:
//
//     Free all the resources allocated in DriverEntry.
//
// Arguments:
//
//     Driver - handle to a WDF Driver object.
//
// Return Value:
//
//     VOID.
//
// --*/
// {
//     WDFDRIVER driver = (WDFDRIVER)Driver;
//
//     PAGED_CODE ();
//
//     TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_INIT,
//                 "CDROM.SYS DriverObject %p cleanup. Will be unloaded soon\n",
//                 driver));
//
//     // Stop WPP Tracing
//     WPP_CLEANUP( WdfDriverWdmGetDriverObject(driver) );
//
//
//     return;
// }
//
//
// NTSTATUS
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// DriverEvtDeviceAdd(
//         WDFDRIVER        Driver,
//      PWDFDEVICE_INIT  DeviceInit
//     )
// /*++
//
// Routine Description:
//
//     EvtDeviceAdd is called by the framework in response to AddDevice
//     call from the PnP manager.
//
//
// Arguments:
//
//     Driver - Handle to a framework driver object created in DriverEntry
//
//     DeviceInit - Pointer to a framework-allocated WDFDEVICE_INIT structure.
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS                        status = STATUS_SUCCESS;
//     PCDROM_DRIVER_EXTENSION         driverExtension = NULL;
//     WDFDEVICE                       device = NULL;
//     PCDROM_DEVICE_EXTENSION         deviceExtension = NULL;
//     BOOLEAN                         deviceClaimed = FALSE;
//
//     WDF_OBJECT_ATTRIBUTES           attributes;
//     WDF_FILEOBJECT_CONFIG           fileObjectConfig;
//     WDF_IO_TARGET_OPEN_PARAMS       ioTargetOpenParams;
//     WDF_IO_QUEUE_CONFIG             queueConfig;
//     WDF_PNPPOWER_EVENT_CALLBACKS    pnpPowerCallbacks;
//     WDF_REMOVE_LOCK_OPTIONS         removeLockOptions;
//     PWCHAR                          wideDeviceName = NULL;
//     UNICODE_STRING                  unicodeDeviceName;
//     PDEVICE_OBJECT                  lowerPdo = NULL;
//     ULONG                           deviceNumber = 0;
//     ULONG                           devicePropertySessionId = INVALID_SESSION;
//     ULONG                           devicePropertySize = 0;
//     DEVPROPTYPE                     devicePropertyType = DEVPROP_TYPE_EMPTY;
//
//     PAGED_CODE();
//
//     driverExtension = DriverGetExtension(Driver);
//
//     // 0. Initialize the objects that we're going to use
//     RtlInitUnicodeString(&unicodeDeviceName, NULL);
//
//     // 1. Register PnP&Power callbacks for any we are interested in.
//     // If a callback isn't set, Framework will take the default action by itself.
//     {
//         // Zero out the PnpPowerCallbacks structure.
//         WDF_PNPPOWER_EVENT_CALLBACKS_INIT(&pnpPowerCallbacks);
//
//         // Use this callback to init resources that are used by the device and only needs to be called once.
//         pnpPowerCallbacks.EvtDeviceSelfManagedIoInit = DeviceEvtSelfManagedIoInit;
//
//         // Use this callback to prepare device for coming back from a lower power mode to D0.
//         pnpPowerCallbacks.EvtDeviceD0Entry = DeviceEvtD0Entry;
//
//         // Use this callback to prepare device for entering into a lower power mode.
//         pnpPowerCallbacks.EvtDeviceD0Exit = DeviceEvtD0Exit;
//
//         // Use this callback to free any resources used by device and will be called when the device is
//         // powered down.
//         pnpPowerCallbacks.EvtDeviceSelfManagedIoCleanup = DeviceEvtSelfManagedIoCleanup;
//
//         pnpPowerCallbacks.EvtDeviceSurpriseRemoval = DeviceEvtSurpriseRemoval;
//
//         // Register the PnP and power callbacks.
//         WdfDeviceInitSetPnpPowerEventCallbacks(DeviceInit, &pnpPowerCallbacks);
//     }
//
//     // 2. Register the EvtIoInCallerContext to deal with IOCTLs that need to stay in original context.
//     WdfDeviceInitSetIoInCallerContextCallback(DeviceInit,
//                                               DeviceEvtIoInCallerContext);
//
//     // 3. Register PreprocessCallback for IRP_MJ_POWER, IRP_MJ_FLUSH_BUFFERS and IRP_MJ_SHUTDOWN
//     {
//         UCHAR minorFunctions[1];
//
//         minorFunctions[0] = IRP_MN_SET_POWER;
//
//         status = WdfDeviceInitAssignWdmIrpPreprocessCallback(DeviceInit,
//                                                              RequestProcessSetPower,
//                                                              IRP_MJ_POWER,
//                                                              minorFunctions,
//                                                              RTL_NUMBER_OF(minorFunctions));
//         if (!NT_SUCCESS(status))
//         {
//             TracePrint!((TRACE_LEVEL_FATAL, TRACE_FLAG_PNP,
//                         "DriverEvtDeviceAdd: Assign IrpPreprocessCallback for IRP_MJ_POWER failed, "
//                         "status: 0x%X\n", status));
//
//             goto Exit;
//         }
//
//         status = WdfDeviceInitAssignWdmIrpPreprocessCallback(DeviceInit,
//                                                              RequestProcessShutdownFlush,
//                                                              IRP_MJ_FLUSH_BUFFERS,
//                                                              NULL,
//                                                              0);
//         if (!NT_SUCCESS(status))
//         {
//             TracePrint!((TRACE_LEVEL_FATAL, TRACE_FLAG_PNP,
//                         "DriverEvtDeviceAdd: Assign IrpPreprocessCallback for IRP_MJ_FLUSH_BUFFERS failed, "
//                         "status: 0x%X\n", status));
//
//             goto Exit;
//         }
//
//         status = WdfDeviceInitAssignWdmIrpPreprocessCallback(DeviceInit,
//                                                              RequestProcessShutdownFlush,
//                                                              IRP_MJ_SHUTDOWN,
//                                                              NULL,
//                                                              0);
//         if (!NT_SUCCESS(status))
//         {
//             TracePrint!((TRACE_LEVEL_FATAL, TRACE_FLAG_PNP,
//                         "DriverEvtDeviceAdd: Assign IrpPreprocessCallback for IRP_MJ_SHUTDOWN failed, "
//                         "status: 0x%X\n", status));
//
//             goto Exit;
//         }
//     }
//
//     // 4. Set attributes to create Request Context area.
//     {
//         //Reuse this structure.
//         WDF_OBJECT_ATTRIBUTES_INIT_CONTEXT_TYPE(&attributes,
//                                                 CDROM_REQUEST_CONTEXT);
//         attributes.EvtCleanupCallback = RequestEvtCleanup;
//
//         WdfDeviceInitSetRequestAttributes(DeviceInit,
//                                           &attributes);
//     }
//
//     // 5. Register FileObject related callbacks
//     {
//         // Add FILE_OBJECT_EXTENSION as the context to the file object.
//         WDF_OBJECT_ATTRIBUTES_INIT_CONTEXT_TYPE(&attributes, FILE_OBJECT_CONTEXT);
//
//         // Set Entry points for Create and Close..
//
//         // The framework doesn't sync the file create requests with pnp/power
//         // state. Re-direct all the file create requests to a dedicated
//         // queue, which will be purged manually during removal.
//         WDF_FILEOBJECT_CONFIG_INIT(&fileObjectConfig,
//                                    NULL, //CreateQueueEvtIoDefault,
//                                    DeviceEvtFileClose,
//                                    WDF_NO_EVENT_CALLBACK); // No callback for Cleanup
//
//         fileObjectConfig.FileObjectClass = WdfFileObjectWdfCannotUseFsContexts;
//
//         // Since we are registering file events and fowarding create request
//         // ourself, we must also set AutoForwardCleanupClose so that cleanup
//         // and close can also get forwarded.
//         fileObjectConfig.AutoForwardCleanupClose = WdfTrue;
//         attributes.SynchronizationScope = WdfSynchronizationScopeNone;
//         attributes.ExecutionLevel = WdfExecutionLevelPassive;
//
//         // Indicate that file object is optional.
//         fileObjectConfig.FileObjectClass |= WdfFileObjectCanBeOptional;
//
//         WdfDeviceInitSetFileObjectConfig(DeviceInit,
//                                          &fileObjectConfig,
//                                          &attributes);
//     }
//
//     // 6. Initialize device-wide attributes
//     {
//         // Declare ourselves as NOT owning power policy.
//         // The power policy owner in storage stack is port driver.
//         WdfDeviceInitSetPowerPolicyOwnership(DeviceInit, FALSE);
//
//         // Set other DeviceInit attributes.
//         WdfDeviceInitSetExclusive(DeviceInit, FALSE);
//         WdfDeviceInitSetDeviceType(DeviceInit, FILE_DEVICE_CD_ROM);
//         WdfDeviceInitSetCharacteristics(DeviceInit, FILE_REMOVABLE_MEDIA, FALSE);
//         WdfDeviceInitSetIoType(DeviceInit, WdfDeviceIoDirect);
//         WdfDeviceInitSetPowerPageable(DeviceInit);
//
//         // We require the framework to acquire a remove lock before delivering all IRP types
//         WDF_REMOVE_LOCK_OPTIONS_INIT(&removeLockOptions,
//                                      WDF_REMOVE_LOCK_OPTION_ACQUIRE_FOR_IO);
//
//         WdfDeviceInitSetRemoveLockOptions(DeviceInit, &removeLockOptions);
//
//         // save the PDO for later reference
//         lowerPdo = WdfFdoInitWdmGetPhysicalDevice(DeviceInit);
//
//         WDF_OBJECT_ATTRIBUTES_INIT_CONTEXT_TYPE(&attributes,
//                                                 CDROM_DEVICE_EXTENSION);
//
//         // We have a parallel queue, so WdfSynchronizationScopeNone is our only choice.
//         attributes.SynchronizationScope = WdfSynchronizationScopeNone;
//         attributes.ExecutionLevel = WdfExecutionLevelDispatch;
//
//         // Provide a cleanup callback which will release memory allocated with ExAllocatePool*
//         attributes.EvtCleanupCallback = DeviceEvtCleanup;
//     }
//
//     // 7. Now, the device can be created.
//     {
//         wideDeviceName = ExAllocatePoolWithTag(NonPagedPoolNx,
//                                                64 * sizeof(WCHAR),
//                                                CDROM_TAG_STRINGS);
//
//         if (wideDeviceName == NULL)
//         {
//             status = STATUS_INSUFFICIENT_RESOURCES;
//             goto Exit;
//         }
//
//         // Find the lowest device number currently available.
//         do {
//             status = RtlStringCchPrintfW((NTSTRSAFE_PWSTR)wideDeviceName,
//                                          64,
//                                          L"\\Device\\CdRom%d",
//                                          deviceNumber);
//
//             if (!NT_SUCCESS(status)) {
//                 TracePrint!((TRACE_LEVEL_FATAL, TRACE_FLAG_PNP,
//                             "DriverEvtDeviceAdd: Format device name failed with error: 0x%X\n", status));
//
//                 goto Exit;
//             }
//
//             RtlInitUnicodeString(&unicodeDeviceName, wideDeviceName);
//
//             status = WdfDeviceInitAssignName(DeviceInit, &unicodeDeviceName);
//             if (!NT_SUCCESS(status))
//             {
//                 TracePrint!((TRACE_LEVEL_FATAL, TRACE_FLAG_PNP,
//                             "DriverEvtDeviceAdd: WdfDeviceInitAssignName() failed with error: 0x%X\n", status));
//
//                 goto Exit;
//             }
//
//             status = WdfDeviceCreate(&DeviceInit, &attributes, &device);
//
//             deviceNumber++;
//
//         } while (status == STATUS_OBJECT_NAME_COLLISION);
//
//         // When this loop exits the count is inflated by one - fix that.
//         deviceNumber--;
//
//         if (!NT_SUCCESS(status))
//         {
//             TracePrint!((TRACE_LEVEL_FATAL, TRACE_FLAG_PNP,
//                         "DriverEvtDeviceAdd: Can not create a new device, status: 0x%X\n",
//                         status));
//
//             goto Exit;
//         }
//     }
//
//     // 8. Fill up basic Device Extension settings and create a remote I/O target for the next-lower driver.
//     //    The reason why we do not use the local I/O target is because we want to be able to close the
//     //    I/O target on surprise removal.
//     {
//         deviceExtension = DeviceGetExtension(device);
//
//         deviceExtension->Version = 0x01;
//         deviceExtension->Size = sizeof(CDROM_DEVICE_EXTENSION);
//
//         deviceExtension->DeviceObject = WdfDeviceWdmGetDeviceObject(device);
//         deviceExtension->Device = device;
//         deviceExtension->DriverExtension = driverExtension;
//
//         deviceExtension->LowerPdo = lowerPdo;
//
//         deviceExtension->DeviceType = FILE_DEVICE_CD_ROM;   //Always a FILE_DEVICE_CD_ROM for all device it manages.
//         deviceExtension->DeviceName = unicodeDeviceName;
//
//         deviceExtension->DeviceNumber = deviceNumber;
//     }
//     {
//         WDF_OBJECT_ATTRIBUTES_INIT(&attributes);
//         attributes.ParentObject = deviceExtension->Device;
//
//         status = WdfIoTargetCreate(deviceExtension->Device,
//                                    &attributes,
//                                    &deviceExtension->IoTarget);
//
//         if (!NT_SUCCESS(status))
//         {
//             TracePrint!((TRACE_LEVEL_FATAL, TRACE_FLAG_PNP,
//                         "DriverEvtDeviceAdd: Can not create a remote I/O target object, status: 0x%X\n",
//                         status));
//             goto Exit;
//         }
//
//         WDF_IO_TARGET_OPEN_PARAMS_INIT_EXISTING_DEVICE(&ioTargetOpenParams,
//                                                        WdfDeviceWdmGetAttachedDevice(deviceExtension->Device));
//
//         status = WdfIoTargetOpen(deviceExtension->IoTarget,
//                                  &ioTargetOpenParams);
//         if (!NT_SUCCESS(status))
//         {
//             TracePrint!((TRACE_LEVEL_FATAL, TRACE_FLAG_PNP,
//                         "DriverEvtDeviceAdd: Can not open a remote I/O target for the next-lower device, status: 0x%X\n",
//                         status));
//
//             WdfObjectDelete(deviceExtension->IoTarget);
//             deviceExtension->IoTarget = NULL;
//
//             goto Exit;
//         }
//     }
//
//     // 9. Claim the device, so that port driver will only accept the commands from CDROM.SYS for this device.
//     // NOTE: The I/O should be issued after the device is started. But we would like to claim
//     // the device as soon as possible, so this legacy behavior is kept.
//     status = DeviceClaimRelease(deviceExtension, FALSE);
//
//     if (!NT_SUCCESS(status))
//     {
//         // Someone already had this device - we're in trouble
//         goto Exit;
//     }
//     else
//     {
//         deviceClaimed = TRUE;
//     }
//
//     //
//     // CDROM Queueing Structure
//     //
//     // a. EvtIoInCallerContext (prior to queueing):
//     //      This event will be used ONLY to forward down IOCTLs that come in at PASSIVE LEVEL
//     //      and need to be forwarded down the stack in the original context.
//     //
//     // b. Main input queue: serial queue for main dispatching
//     //      This queue will be used to do all dispatching of requests to serialize
//     //      access to the device.  Any request that was previously completed in
//     //      the Dispatch routines will be completed here.  Anything requiring device
//     //      I/O will be sent through the serial I/O queue.
//     //
//     // 10. Set up IO queues after device being created.
//     //
//     {
//         WDF_IO_QUEUE_CONFIG_INIT_DEFAULT_QUEUE(&queueConfig,
//                                                WdfIoQueueDispatchSequential);
//
//         queueConfig.PowerManaged                = WdfFalse;
//
// #pragma prefast(push)
// #pragma prefast(disable: 28155, "a joint handler for read/write cannot be EVT_WDF_IO_QUEUE_IO_READ and EVT_WDF_IO_QUEUE_IO_WRITE simultaneously")
// #pragma prefast(disable: 28023, "a joint handler for read/write cannot be EVT_WDF_IO_QUEUE_IO_READ and EVT_WDF_IO_QUEUE_IO_WRITE simultaneously")
//         queueConfig.EvtIoRead                   = SequentialQueueEvtIoReadWrite;
//         queueConfig.EvtIoWrite                  = SequentialQueueEvtIoReadWrite;
// #pragma prefast(pop)
//
//         queueConfig.EvtIoDeviceControl          = SequentialQueueEvtIoDeviceControl;
//         queueConfig.EvtIoCanceledOnQueue        = SequentialQueueEvtCanceledOnQueue;
//
//         status = WdfIoQueueCreate(device,
//                                   &queueConfig,
//                                   WDF_NO_OBJECT_ATTRIBUTES,
//                                   &(deviceExtension->SerialIOQueue));
//         if (!NT_SUCCESS(status))
//         {
//             goto Exit;
//         }
//
//         // this queue is dedicated for file create requests.
//         WDF_IO_QUEUE_CONFIG_INIT(&queueConfig, WdfIoQueueDispatchParallel);
//
//         queueConfig.PowerManaged = WdfFalse;
//         queueConfig.EvtIoDefault = CreateQueueEvtIoDefault;
//
//         //Reuse this structure.
//         WDF_OBJECT_ATTRIBUTES_INIT_CONTEXT_TYPE(&attributes,
//                                                 CDROM_REQUEST_CONTEXT);
//
//         attributes.SynchronizationScope = WdfSynchronizationScopeNone;
//         attributes.ExecutionLevel = WdfExecutionLevelPassive;
//
//         status = WdfIoQueueCreate(device,
//                                   &queueConfig,
//                                   &attributes,
//                                   &(deviceExtension->CreateQueue));
//
//         if (!NT_SUCCESS(status))
//         {
//             goto Exit;
//         }
//
//         // Configure the device to use driver created queue for dispatching create.
//         status = WdfDeviceConfigureRequestDispatching(device,
//                                                       deviceExtension->CreateQueue,
//                                                       WdfRequestTypeCreate);
//
//         if (!NT_SUCCESS(status))
//         {
//             goto Exit;
//         }
//     }
//
//     // 11. Set the alignment requirements for the device based on the host adapter requirements.
//     //
//     // NOTE: this should have been set when device is attached on device stack,
//     // by keeping this legacy code, we could avoid issue that this value was not correctly set at that time.
//     if (deviceExtension->LowerPdo->AlignmentRequirement > deviceExtension->DeviceObject->AlignmentRequirement)
//     {
//         WdfDeviceSetAlignmentRequirement(deviceExtension->Device,
//                                          deviceExtension->LowerPdo->AlignmentRequirement);
//     }
//
//     // 12. Initialization of miscellaneous internal properties
//
//     // CDROMs are not partitionable so starting offset is 0.
//     deviceExtension->StartingOffset.LowPart = 0;
//     deviceExtension->StartingOffset.HighPart = 0;
//
//     // Set the default geometry for the cdrom to match what NT 4 used.
//     // these values will be used to compute the cylinder count rather
//     // than using it's NT 5.0 defaults.
//     deviceExtension->DiskGeometry.MediaType = RemovableMedia;
//     deviceExtension->DiskGeometry.TracksPerCylinder = 0x40;
//     deviceExtension->DiskGeometry.SectorsPerTrack = 0x20;
//
//     deviceExtension->DeviceAdditionalData.ReadWriteRetryDelay100nsUnits = WRITE_RETRY_DELAY_DVD_1x as i64;
//
//     // Clear the SrbFlags and disable synchronous transfers
//     deviceExtension->SrbFlags = SRB_FLAGS_DISABLE_SYNCH_TRANSFER;
//
//     // Set timeout value in seconds.
//     deviceExtension->TimeOutValue = DeviceGetTimeOutValueFromRegistry();
//     if ((deviceExtension->TimeOutValue > 30 * 60) || // longer than 30 minutes
//         (deviceExtension->TimeOutValue == 0))
//     {
//         deviceExtension->TimeOutValue = SCSI_CDROM_TIMEOUT;
//     }
//
//     TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_INIT,
//                "DriverEvtDeviceAdd: device timeout is set to %x seconds",
//                deviceExtension->TimeOutValue
//                ));
//
// #if (NTDDI_VERSION >= NTDDI_WIN8)
//     deviceExtension->IsVolumeOnlinePending = TRUE;
//
//     WDF_IO_QUEUE_CONFIG_INIT(&queueConfig, WdfIoQueueDispatchManual);
//
//     queueConfig.PowerManaged = WdfFalse;
//
//     status = WdfIoQueueCreate(device,
//                               &queueConfig,
//                               WDF_NO_OBJECT_ATTRIBUTES,
//                               &deviceExtension->ManualVolumeReadyQueue);
//
//     if (!NT_SUCCESS(status))
//     {
//         goto Exit;
//     }
// #endif
//
//     // 13. Initialize the stuff related to media locking
//     WDF_OBJECT_ATTRIBUTES_INIT(&attributes);
//     attributes.ParentObject = deviceExtension->Device;
//     status = WdfWaitLockCreate(&attributes,
//                                &deviceExtension->EjectSynchronizationLock);
//
//     deviceExtension->LockCount = 0;
//
//     // 14. Initialize context structures needed for asynchronous release queue and power requests
//
//     if (NT_SUCCESS(status))
//     {
//         status = DeviceInitReleaseQueueContext(deviceExtension);
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         status = DeviceInitPowerContext(deviceExtension);
//     }
//
//     // 15. Create external access points other than device name.
//     if (NT_SUCCESS(status))
//     {
//         status = DeviceCreateWellKnownName(deviceExtension);
//     }
//
//     // 16. Query session id from the PDO.
//     if (NT_SUCCESS(status))
//     {
//         status = IoGetDevicePropertyData(deviceExtension->LowerPdo,
//                                          &DEVPKEY_Device_SessionId,
//                                          0,
//                                          0,
//                                          sizeof(devicePropertySessionId),
//                                          &devicePropertySessionId,
//                                          &devicePropertySize,
//                                          &devicePropertyType);
//
//         if (!NT_SUCCESS(status))
//         {
//             // The device is global.
//             devicePropertySessionId = INVALID_SESSION;
//             status = STATUS_SUCCESS;
//         }
//     }
//
//     // 17. Register interfaces for this device.
//     if (NT_SUCCESS(status))
//     {
//         status = DeviceRegisterInterface(deviceExtension, CdRomDeviceInterface);
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         // If this is a per-session DO, don't register for mount interface so that
//         // mountmgr will not automatically assign a drive letter.
//         if (devicePropertySessionId == INVALID_SESSION)
//         {
//             status = DeviceRegisterInterface(deviceExtension, MountedDeviceInterface);
//         }
//     }
//
//     // 18. Initialize the shutdown/flush lock
//     if (NT_SUCCESS(status))
//     {
//         WDF_OBJECT_ATTRIBUTES_INIT(&attributes);
//         attributes.ParentObject = deviceExtension->Device;
//
//         status = WdfWaitLockCreate(&attributes, &deviceExtension->ShutdownFlushWaitLock);
//         if (!NT_SUCCESS(status))
//         {
//             TracePrint!((TRACE_LEVEL_FATAL, TRACE_FLAG_PNP,
//                         "DriverEvtDeviceAdd: Cannot create shutdown/flush waitlock, status: 0x%X\n",
//                         status));
//         }
//     }
//
//     // 19. Initialize the work item that is used to initiate asynchronous reads/writes
//     if (NT_SUCCESS(status))
//     {
//         WDF_WORKITEM_CONFIG workItemConfig;
//         WDF_OBJECT_ATTRIBUTES workItemAttributes;
//
//         WDF_WORKITEM_CONFIG_INIT(&workItemConfig,
//                                  ReadWriteWorkItemRoutine
//                                  );
//
//         WDF_OBJECT_ATTRIBUTES_INIT(&workItemAttributes);
//         workItemAttributes.ParentObject = deviceExtension->Device;
//
//         status = WdfWorkItemCreate(&workItemConfig,
//                                    &workItemAttributes,
//                                    &deviceExtension->ReadWriteWorkItem
//                                    );
//
//         if (!NT_SUCCESS(status))
//         {
//             TracePrint!((TRACE_LEVEL_FATAL, TRACE_FLAG_INIT,
//                         "DriverEvtDeviceAdd: Cannot create read/write work item, status: 0x%X\n",
//                         status));
//         }
//     }
//
//     // 20. Initialize the work item that is used to process most IOCTLs at PASSIVE_LEVEL.
//     if (NT_SUCCESS(status))
//     {
//         WDF_WORKITEM_CONFIG workItemConfig;
//         WDF_OBJECT_ATTRIBUTES workItemAttributes;
//
//         WDF_WORKITEM_CONFIG_INIT(&workItemConfig,
//                                  IoctlWorkItemRoutine
//                                  );
//
//         WDF_OBJECT_ATTRIBUTES_INIT(&workItemAttributes);
//         workItemAttributes.ParentObject = deviceExtension->Device;
//
//         status = WdfWorkItemCreate(&workItemConfig,
//                                    &workItemAttributes,
//                                    &deviceExtension->IoctlWorkItem
//                                    );
//
//         if (!NT_SUCCESS(status))
//         {
//             TracePrint!((TRACE_LEVEL_FATAL, TRACE_FLAG_INIT,
//                         "DriverEvtDeviceAdd: Cannot create ioctl work item, status: 0x%X\n",
//                         status));
//         }
//     }
//
//
// Exit:
//
//     if (!NT_SUCCESS(status))
//     {
//         FREE_POOL(wideDeviceName);
//
//         if (deviceExtension != NULL)
//         {
//             RtlInitUnicodeString(&deviceExtension->DeviceName, NULL);
//         }
//
//         // Release the device with the port driver, if it was claimed
//         if ((deviceExtension != NULL) && deviceClaimed)
//         {
//             DeviceClaimRelease(deviceExtension, TRUE);
//         }
//         deviceClaimed = FALSE;
//     }
//
//     return status;
// }
//
//
// VOID
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// DeviceEvtCleanup(
//      WDFOBJECT Device
//     )
// /*++
// Routine Description:
//
//     Free all the resources allocated in DriverEvtDeviceAdd.
//
// Arguments:
//
//     Device - handle to a WDF Device object.
//
// Return Value:
//
//     VOID.
//
// --*/
// {
//     WDFDEVICE                device = (WDFDEVICE)Device;
//     PCDROM_DEVICE_EXTENSION  deviceExtension = NULL;
//
//     PAGED_CODE ();
//
//     TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_INIT,
//                 "WDFDEVICE %p cleanup: The device is about to be destroyed.\n",
//                 device));
//
//     deviceExtension = DeviceGetExtension(device);
//
//     FREE_POOL(deviceExtension->DeviceName.Buffer);
//     RtlInitUnicodeString(&deviceExtension->DeviceName, NULL);
//
//     if (deviceExtension->DeviceAdditionalData.WellKnownName.Buffer != NULL)
//     {
//         IoDeleteSymbolicLink(&deviceExtension->DeviceAdditionalData.WellKnownName);
//     }
//
//     FREE_POOL(deviceExtension->DeviceAdditionalData.WellKnownName.Buffer);
//     RtlInitUnicodeString(&deviceExtension->DeviceAdditionalData.WellKnownName, NULL);
//
//     return;
// }
//
//
// VOID
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// DeviceEvtSelfManagedIoCleanup(
//      WDFDEVICE    Device
//     )
// /*++
//
// Routine Description:
//
//     this function is called when the device is removed.
//     release the ownership of the device, release allocated resources.
//
// Arguments:
//
//     Device - Handle to device object
//
// Return Value:
//
//     None.
//
// --*/
// {
//     NTSTATUS                status;
//     PCDROM_DEVICE_EXTENSION deviceExtension = NULL;
//
//     PAGED_CODE ();
//
//     TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_PNP,
//                 "DeviceEvtSelfManagedIoCleanup: WDFDEVICE %p is being stopped.\n",
//                 Device));
//
//     // extract the device and driver extensions
//     deviceExtension = DeviceGetExtension(Device);
//
//     // Purge unprocessed requests, stop the IO queues.
//     // Incoming request will be completed with STATUS_INVALID_DEVICE_STATE status.
//     WdfIoQueuePurge(deviceExtension->SerialIOQueue, WDF_NO_EVENT_CALLBACK, WDF_NO_CONTEXT);
//     WdfIoQueuePurge(deviceExtension->CreateQueue, WDF_NO_EVENT_CALLBACK, WDF_NO_CONTEXT);
//
//     // Close the IoTarget so that we are sure there are no outstanding I/Os in the stack.
//     if (deviceExtension->IoTarget)
//     {
//         WdfIoTargetClose(deviceExtension->IoTarget);
//     }
//
//     // Release the device
//     if (!deviceExtension->SurpriseRemoved)
//     {
//         status = DeviceClaimRelease(deviceExtension, TRUE);  //status is mainly for debugging. we don't really care.
//         UNREFERENCED_PARAMETER(status);             //defensive coding, avoid PREFAST warning.
//     }
//
//     // Be sure to flush the DPCs as the READ/WRITE timer routine may still be running
//     // during device removal.  This call may take a while to complete.
//     KeFlushQueuedDpcs();
//
//     // Release all the memory that we have allocated.
//
//     DeviceDeallocateMmcResources(Device);
//     ScratchBuffer_Deallocate(deviceExtension);
//     RtlZeroMemory(&(deviceExtension->DeviceAdditionalData.Mmc), sizeof(CDROM_MMC_EXTENSION));
//
//     FREE_POOL(deviceExtension->DeviceDescriptor);
//     FREE_POOL(deviceExtension->AdapterDescriptor);
//     FREE_POOL(deviceExtension->PowerDescriptor);
//     FREE_POOL(deviceExtension->SenseData);
//
//     if (deviceExtension->DeviceAdditionalData.CachedInquiryData != NULL)
//     {
//         FREE_POOL(deviceExtension->DeviceAdditionalData.CachedInquiryData);
//         deviceExtension->DeviceAdditionalData.CachedInquiryDataByteCount = 0;
//     }
//
//     FREE_POOL(deviceExtension->PrivateFdoData);
//
//     DeviceReleaseMcnResources(deviceExtension);
//
//     DeviceReleaseZPODDResources(deviceExtension);
//
//     // Keep the system-wide CDROM count accurate, as programs use this info to
//     // know when they have found all the cdroms in a system.
//     IoGetConfigurationInformation()->CdRomCount--;
//
//     deviceExtension->PartitionLength.QuadPart = 0;
//
//     // All WDF objects related to Device will be automatically released
//     // when the root object is deleted. No need to release them manually.
//
//     return;
// }
//
// NTSTATUS
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// DeviceEvtD0Entry(
//      WDFDEVICE Device,
//      WDF_POWER_DEVICE_STATE PreviousState
//     )
// /*++
//
// Routine Description:
//
//     This function is called when the device is coming back from a lower power state to D0.
//     This function cannot be placed in a pageable section.
//
// Arguments:
//
//     Device - Handle to device object
//     PreviousState - Power state the device was in.
//
// Return Value:
//
//     NTSTATUS: alway STATUS_SUCCESS
//
// --*/
// {
//     PCDROM_DEVICE_EXTENSION     deviceExtension;
//     NTSTATUS                    status = STATUS_SUCCESS;
//     PZERO_POWER_ODD_INFO        zpoddInfo = NULL;
//     STORAGE_IDLE_POWERUP_REASON powerupReason = {0};
//
//     UNREFERENCED_PARAMETER(PreviousState);
//     deviceExtension = DeviceGetExtension(Device);
//
//     // Make certain not to do anything before properly initialized
//     if (deviceExtension->IsInitialized)
//     {
//         zpoddInfo = deviceExtension->ZeroPowerODDInfo;
//
//         if (zpoddInfo != NULL)
//         {
//             if (zpoddInfo->InZeroPowerState != FALSE)
//             {
//                 // We just woke up from Zero Power state
//                 zpoddInfo->InZeroPowerState = FALSE;
//                 zpoddInfo->RetryFirstCommand = TRUE;
//                 zpoddInfo->BecomingReadyRetryCount = BECOMING_READY_RETRY_COUNT;
//
//                 status = DeviceZPODDGetPowerupReason(deviceExtension, &powerupReason);
//
//                 if (NT_SUCCESS(status) &&
//                     (powerupReason.PowerupReason == StoragePowerupDeviceAttention))
//                 {
//                     TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_POWER,
//                                 "DeviceEvtD0Entry: Device has left zero power state due to eject button pressed\n"
//                                 ));
//
//                     // This wake-up is caused by user pressing the eject button.
//                     // In case of drawer type, we need to soft eject the tray to emulate the effect.
//                     // Note that the first command to the device after power resumed will
//                     // be terminated with CHECK CONDITION status with sense code 6/29/00,
//                     // but we already have a retry logic to handle this.
//                     if ((zpoddInfo->LoadingMechanism == LOADING_MECHANISM_TRAY) && (zpoddInfo->Load == 0))    // Drawer
//                     {
//                         DeviceSendIoctlAsynchronously(deviceExtension, IOCTL_STORAGE_EJECT_MEDIA, deviceExtension->DeviceObject);
//                     }
//                 }
//                 else
//                 {
//                     // This wake-up is caused by non-cached CDB received or a 3rd-party driver
//                     TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_POWER,
//                                 "DeviceEvtD0Entry: Device has left zero power state due to IO received\n"
//                                 ));
//
//                 }
//             }
//         }
//
//         DeviceEnableMainTimer(deviceExtension);
//     }
//
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// DeviceEvtD0Exit(
//      WDFDEVICE Device,
//      WDF_POWER_DEVICE_STATE TargetState
//     )
// /*++
//
// Routine Description:
//
//     This function is called when the device is entering lower powe state from D0 or it's removed.
//     We only care about the case of device entering D3.
//     The purpose of this function is to send SYNC CACHE command and STOP UNIT command if it's necessary.
//
// Arguments:
//
//     Device - Handle to device object
//     TargetState - Power state the device is entering.
//
// Return Value:
//
//     NTSTATUS: alway STATUS_SUCCESS
//
// --*/
// {
//     NTSTATUS                status = STATUS_SUCCESS;
//     PCDROM_DEVICE_EXTENSION deviceExtension = NULL;
//     PZERO_POWER_ODD_INFO    zpoddInfo = NULL;
//
//     PAGED_CODE ();
//
//     deviceExtension = DeviceGetExtension(Device);
//     zpoddInfo = deviceExtension->ZeroPowerODDInfo;
//
//     // we only process the situation that the device is going into D3.
//     if ((TargetState != WdfPowerDeviceD3) &&
//         (TargetState != WdfPowerDeviceD3Final))
//     {
//         return STATUS_SUCCESS;
//     }
//
//     // Stop the main timer
//     DeviceDisableMainTimer(deviceExtension);
//
//     // note: do not stop CreateQueue as the create request can be handled by port driver even the device is in D3 status.
//
//     // If initialization was not finished or the device was removed, we cannot interact
//     // with it device, so we have to exit
//     if ((!deviceExtension->IsInitialized) || deviceExtension->SurpriseRemoved)
//     {
//         return STATUS_SUCCESS;
//     }
//
//
// #ifdef DBG
//     #if (WINVER >= 0x0601)
//     // this API is introduced in Windows7
//     {
//         ULONG   secondsRemaining = 0;
//         BOOLEAN watchdogTimeSupported = FALSE;
//
//         watchdogTimeSupported = PoQueryWatchdogTime(deviceExtension->LowerPdo, &secondsRemaining);
//         UNREFERENCED_PARAMETER(watchdogTimeSupported);
//     }
//     #endif
// #endif
//
//     deviceExtension->PowerDownInProgress = TRUE;
//
//     status = PowerContextBeginUse(deviceExtension);
//
//     deviceExtension->PowerContext.Options.PowerDown = TRUE;
//
//     // Step 1. LOCK QUEUE
//     if (NT_SUCCESS(status) &&
//         (TargetState != WdfPowerDeviceD3Final))
//     {
//         status = DeviceSendPowerDownProcessRequest(deviceExtension, NULL, NULL);
//
//         if (NT_SUCCESS(status))
//         {
//             deviceExtension->PowerContext.Options.LockQueue = TRUE;
//         }
//
//         // Ignore failure.
//         status = STATUS_SUCCESS;
//     }
//
//     deviceExtension->PowerContext.PowerChangeState.PowerDown++;
//
//     // Step 2. QUIESCE QUEUE
//     if (NT_SUCCESS(status) &&
//         (TargetState != WdfPowerDeviceD3Final))
//     {
//         status = DeviceSendPowerDownProcessRequest(deviceExtension, NULL, NULL);
//         UNREFERENCED_PARAMETER(status);
//         // We don't care about the status.
//         status = STATUS_SUCCESS;
//     }
//
//     deviceExtension->PowerContext.PowerChangeState.PowerDown++;
//
//     // Step 3. SYNC CACHE command should be sent to drive if the media is currently writable.
//     if (NT_SUCCESS(status) &&
//         deviceExtension->DeviceAdditionalData.Mmc.WriteAllowed)
//     {
//         status = DeviceSendPowerDownProcessRequest(deviceExtension, NULL, NULL);
//         UNREFERENCED_PARAMETER(status);
//         status = STATUS_SUCCESS;
//     }
//
//     deviceExtension->PowerContext.PowerChangeState.PowerDown++;
//
//     // Step 4. STOP UNIT
//     if (NT_SUCCESS(status) &&
//         !TEST_FLAG(deviceExtension->ScanForSpecialFlags, CDROM_SPECIAL_DISABLE_SPIN_DOWN))
//     {
//         status = DeviceSendPowerDownProcessRequest(deviceExtension, NULL, NULL);
//         UNREFERENCED_PARAMETER(status);
//         status = STATUS_SUCCESS;
//     }
//
//     if (TargetState == WdfPowerDeviceD3Final)
//     {
//         // We're done with the power context.
//         PowerContextEndUse(deviceExtension);
//     }
//
//     // Bumping the media  change count  will force the file system to verify the volume when we resume
//     SET_FLAG(deviceExtension->DeviceObject->Flags, DO_VERIFY_VOLUME);
//     InterlockedIncrement((PLONG)&deviceExtension->MediaChangeCount);
//
//     // If this power down is caused by Zero Power ODD, we should mark the device as in ZPODD mode.
//     if (zpoddInfo != NULL)
//     {
//         zpoddInfo->InZeroPowerState = TRUE;
//
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_POWER,
//                     "Device has entered zero power state\n"
//                     ));
//     }
//
//     deviceExtension->PowerDownInProgress = FALSE;
//
//     return STATUS_SUCCESS;
// }
//
//
// VOID
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// DeviceEvtSurpriseRemoval(
//      WDFDEVICE    Device
//     )
// /*++
//
// Routine Description:
//
//     this function is called when the device is surprisely removed.
//     Stop all IO queues so that there will be no more request being sent down.
//
// Arguments:
//
//     Device - Handle to device object
//
// Return Value:
//
//     None.
//
// --*/
// {
//     PCDROM_DEVICE_EXTENSION deviceExtension = NULL;
//
//     PAGED_CODE();
//
//     deviceExtension = DeviceGetExtension(Device);
//
//     deviceExtension->SurpriseRemoved = TRUE;
//
//     // Stop the main timer
//     DeviceDisableMainTimer(deviceExtension);
//
//     // legacy behavior to set partition length to be 0.
//     deviceExtension->PartitionLength.QuadPart = 0;
//
//     TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_INIT,
//                 "Surprisely remove a WDFDEVICE %p\n", Device));
//
//     return;
// }
//
//
// VOID
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// CreateQueueEvtIoDefault(
//      WDFQUEUE Queue,
//      WDFREQUEST Request
//     )
// /*++
//
// Routine Description:
//
//     this function is called when CREATE irp comes.
//     setup FileObject context fields, so it can be used to track MCN or exclusive lock/unlock.
//
// Arguments:
//
//     Queue - Handle to device queue
//
//     Request - the creation request
//
// Return Value:
//
//     None
//
// --*/
// {
//     WDFFILEOBJECT           fileObject = WdfRequestGetFileObject(Request);
//     WDFDEVICE               device = WdfIoQueueGetDevice(Queue);
//     NTSTATUS                status = STATUS_SUCCESS;
//     PCDROM_DEVICE_EXTENSION deviceExtension = DeviceGetExtension(device);
//     PFILE_OBJECT_CONTEXT    fileObjectContext = NULL;
//
//     PAGED_CODE();
//
//     if (fileObject == NULL) {
//
//         TracePrint!((TRACE_LEVEL_ERROR, TRACE_FLAG_QUEUE,
//                     "Error: received a file create request with file object set to NULL\n"));
//
//         RequestCompletion(deviceExtension, Request, STATUS_INTERNAL_ERROR, 0);
//         return;
//     }
//
//     fileObjectContext = FileObjectGetContext(fileObject);
//
//     // Initialize this WDFFILEOBJECT's context
//     fileObjectContext->DeviceObject = device;
//     fileObjectContext->FileObject = fileObject;
//     fileObjectContext->LockCount = 0;
//     fileObjectContext->McnDisableCount = 0;
//     fileObjectContext->EnforceStreamingRead = FALSE;
//     fileObjectContext->EnforceStreamingWrite = FALSE;
//
//     // send down the create synchronously
//     status = DeviceSendRequestSynchronously(device, Request, FALSE);
//
//     // Need to complete the request in this routine.
//     RequestCompletion(deviceExtension, Request, status, WdfRequestGetInformation(Request));
//
//     return;
// }
//
// VOID
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// DeviceEvtFileClose(
//      WDFFILEOBJECT FileObject
//     )
// /*++
//
// Routine Description:
//
//     this function is called when CLOSE irp comes.
//     clean up MCN / Lock if necessary
//
// Arguments:
//
//     FileObject - WDF file object created for the irp.
//
// Return Value:
//
//     None
//
// --*/
// {
//     NTSTATUS                status = STATUS_SUCCESS;
//
//     PAGED_CODE();
//
//     if (FileObject != NULL)
//     {
//         WDFDEVICE               device = WdfFileObjectGetDevice(FileObject);
//         PCDROM_DEVICE_EXTENSION deviceExtension = DeviceGetExtension(device);
//         PCDROM_DATA             cdData = &(deviceExtension->DeviceAdditionalData);
//         PFILE_OBJECT_CONTEXT    fileObjectContext = FileObjectGetContext(FileObject);
//
//         // cleanup locked media tray
//         status = DeviceCleanupProtectedLocks(deviceExtension, fileObjectContext);
//         if (!NT_SUCCESS(status))
//         {
//             TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_INIT,
//                         "Failed to cleanup protected locks for WDFDEVICE %p, %!STATUS!\n", device, status));
//         }
//
//         // cleanup disabled MCN
//         status = DeviceCleanupDisableMcn(deviceExtension, fileObjectContext);
//         if (!NT_SUCCESS(status))
//         {
//             TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_INIT,
//                         "Failed to disable MCN for WDFDEVICE %p, %!STATUS!\n", device, status));
//         }
//
//         // cleanup exclusive access
//         if (EXCLUSIVE_MODE(cdData) && EXCLUSIVE_OWNER(cdData, FileObject))
//         {
//             status = DeviceUnlockExclusive(deviceExtension, FileObject, FALSE);
//             if (!NT_SUCCESS(status))
//             {
//                 TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_INIT,
//                             "Failed to release exclusive lock for WDFDEVICE %p, %!STATUS!\n", device, status));
//             }
//         }
//     }
//
//     return;
// }
//
// _IRQL_requires_max_(PASSIVE_LEVEL)
// NTSTATUS
// DeviceCleanupProtectedLocks(
//      PCDROM_DEVICE_EXTENSION  DeviceExtension,
//      PFILE_OBJECT_CONTEXT     FileObjectContext
//     )
// /*++
//
// Routine Description:
//
//     this function removes protected locks for the handle
//
// Arguments:
//
//     DeviceExtension - device context
//
//     FileObject - WDF file object created for the irp.
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS    status = STATUS_SUCCESS;
//
//     PAGED_CODE();
//
//     TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_INIT,
//                 "CleanupProtectedLocks called for WDFDEVICE %p, WDFFILEOBJECT %p, locked %d times.\n",
//                 DeviceExtension->Device, FileObjectContext->FileObject, FileObjectContext->LockCount));
//
//     // Synchronize with ejection and ejection control requests.
//     WdfWaitLockAcquire(DeviceExtension->EjectSynchronizationLock, NULL);
//
//     // For each secure lock on this handle decrement the secured lock count
//     // for the FDO.  Keep track of the new value.
//     if (FileObjectContext->LockCount != 0)
//     {
//         DeviceExtension->ProtectedLockCount -= FileObjectContext->LockCount;
//         FileObjectContext->LockCount = 0;
//
//         // If the new lock count has been dropped to zero then issue a lock
//         // command to the device.
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_INIT,
//                     "FDO secured lock count = %d "
//                     "lock count = %d\n",
//                     DeviceExtension->ProtectedLockCount,
//                     DeviceExtension->LockCount));
//
//         if ((DeviceExtension->ProtectedLockCount == 0) && (DeviceExtension->LockCount == 0))
//         {
//             SCSI_REQUEST_BLOCK  srb = {0};
//             PCDB                cdb = (PCDB) &(srb.Cdb);
//
//             srb.CdbLength = 6;
//
//             cdb->MEDIA_REMOVAL.OperationCode = SCSIOP_MEDIUM_REMOVAL;
//
//             // TRUE - prevent media removal.
//             // FALSE - allow media removal.
//             cdb->MEDIA_REMOVAL.Prevent = FALSE;
//
//             // Set timeout value.
//             srb.TimeOutValue = DeviceExtension->TimeOutValue;
//             status = DeviceSendSrbSynchronously(DeviceExtension->Device,
//                                                 &srb,
//                                                 NULL,
//                                                 0,
//                                                 FALSE,
//                                                 NULL);
//
//             TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_INIT,
//                         "Allow media removal (unlock) request to drive returned %!STATUS!\n",
//                         status));
//         }
//     }
//
//     WdfWaitLockRelease(DeviceExtension->EjectSynchronizationLock);
//
//     return status;
// }
//
//
// _IRQL_requires_max_(APC_LEVEL)
// NTSTATUS
// DeviceCleanupDisableMcn(
//      PCDROM_DEVICE_EXTENSION  DeviceExtension,
//      PFILE_OBJECT_CONTEXT     FileObjectContext
//     )
// /*++
//
// Routine Description:
//
//     cleanup the MCN disable count for the handle
//
// Arguments:
//
//     DeviceExtension - device context
//
//     FileObject - WDF file object created for the irp.
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     PAGED_CODE();
//
//     TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_INIT,
//                 "CleanupDisableMcn called for WDFDEVICE %p, WDFFILEOBJECT %p, locked %d times.\n",
//                 DeviceExtension->Device, FileObjectContext->FileObject, FileObjectContext->McnDisableCount));
//
//     // For each secure lock on this handle decrement the secured lock count
//     // for the FDO.  Keep track of the new value.
//     while (FileObjectContext->McnDisableCount != 0)
//     {
//         DeviceEnableMediaChangeDetection(DeviceExtension, FileObjectContext, TRUE);
//     }
//
//     return STATUS_SUCCESS;
// }
//
// VOID
// NormalizeIoctl(
//      PWDF_REQUEST_PARAMETERS  requestParameters
//     )
// {
//     ULONG ioctlCode;
//     ULONG baseCode;
//     ULONG functionCode;
//
//     // if this is a class driver ioctl then we need to change the base code
//     // to IOCTL_STORAGE_BASE so that the switch statement can handle it.
//     //
//     // WARNING - currently the scsi class ioctl function codes are between
//     // 0x200 & 0x300.  this routine depends on that fact
//     ioctlCode = requestParameters->Parameters.DeviceIoControl.IoControlCode;
//     baseCode = DEVICE_TYPE_FROM_CTL_CODE(ioctlCode);
//     functionCode = (ioctlCode & (~0xffffc003)) >> 2;
//
//     if ((baseCode == IOCTL_SCSI_BASE) ||
//         (baseCode == IOCTL_DISK_BASE) ||
//         (baseCode == IOCTL_TAPE_BASE) ||
//         (baseCode == IOCTL_DVD_BASE) ||
//         (baseCode == IOCTL_CDROM_BASE))
//         //IOCTL_STORAGE_BASE does not need to be converted.
//     {
//         if((functionCode >= 0x200) && (functionCode <= 0x300))
//         {
//             ioctlCode = (ioctlCode & 0x0000ffff) | CTL_CODE(IOCTL_STORAGE_BASE, 0, 0, 0);
//
//             TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_IOCTL,
//                         "IOCTL code recalibrate, New ioctl code is %lx\n",
//                         ioctlCode));
//
//             // Set the code into request parameters, then "requestParameters" needs to be used for dispatch functions.
//             requestParameters->Parameters.DeviceIoControl.IoControlCode = ioctlCode;
//         }
//     }
// }
//
// VOID
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// DeviceEvtIoInCallerContext(
//      WDFDEVICE    Device,
//      WDFREQUEST   Request
//     )
// /*++
// Routine Description:
//
//     Responds to EvtIoInCallerContext events from KMDF
//     It calls different functions to process different type of IOCTLs.
//
// Arguments:
//
//     Device - handle to a WDF Device object
//
//     Request - handle to the incoming WDF Request object
//
// Return Value:
//
//     VOID.
//
// --*/
// {
//     NTSTATUS                status  = STATUS_SUCCESS;
//     PCDROM_DEVICE_EXTENSION deviceExtension = DeviceGetExtension(Device);
//     PCDROM_REQUEST_CONTEXT  requestContext = RequestGetContext(Request);
//     WDF_REQUEST_PARAMETERS  requestParameters;
//
//     requestContext->DeviceExtension = deviceExtension;
//
//     // set the received time
//     RequestSetReceivedTime(Request);
//
//     // get the request parameters
//     WDF_REQUEST_PARAMETERS_INIT(&requestParameters);
//     WdfRequestGetParameters(Request, &requestParameters);
//
//     if (requestParameters.Type == WdfRequestTypeDeviceControl)
//     {
//         BOOLEAN                         processed = FALSE;
//         PCDROM_DATA                     cdData = &(deviceExtension->DeviceAdditionalData);
//         PMEDIA_CHANGE_DETECTION_INFO    info = deviceExtension->MediaChangeDetectionInfo;
//
//         if (requestParameters.Parameters.DeviceIoControl.IoControlCode != IOCTL_MCN_SYNC_FAKE_IOCTL)
//         {
//             TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL,
//                         "Receiving IOCTL: %lx\n",
//                         requestParameters.Parameters.DeviceIoControl.IoControlCode));
//         }
//         else
//         {
//             TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_IOCTL,
//                         "Receiving IOCTL: %lx\n",
//                         requestParameters.Parameters.DeviceIoControl.IoControlCode));
//         }
//
//         // If the device is in exclusive mode, check whether the request is from
//         // the handle that locked the device.
//         if (EXCLUSIVE_MODE(cdData) && !EXCLUSIVE_OWNER(cdData, WdfRequestGetFileObject(Request)))
//         {
//             BOOLEAN isBlocked = FALSE;
//
//             status = RequestIsIoctlBlockedByExclusiveAccess(Request, &isBlocked);
//             UNREFERENCED_PARAMETER(status); //defensive coding, avoid PREFAST warning.
//
//             if (isBlocked)
//             {
//                 if ((requestParameters.Parameters.DeviceIoControl.IoControlCode == IOCTL_STORAGE_EVENT_NOTIFICATION) &&
//                     (info != NULL) && (info->AsynchronousNotificationSupported != FALSE))
//                 {
//                     // If AN is supported and we receive a signal but we can't send down GESN
//                     // due to exclusive lock, we should save it and fire a GESN when it's unlocked.
//                     // We just need true/false here and don't need count because we will keep sending
//                     // GESN until we deplete all events.
//                     info->ANSignalPendingDueToExclusiveLock = TRUE;
//                 }
//
//                 TracePrint!((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL,
//                             "Access Denied! Device in exclusive mode.Failing Ioctl %lx\n",
//                             requestParameters.Parameters.DeviceIoControl.IoControlCode));
//                 RequestCompletion(deviceExtension, Request, STATUS_ACCESS_DENIED, 0);
//
//                 return;
//             }
//         }
//
//         NormalizeIoctl(&requestParameters);
//
//         // 1. All requests that don't need to access device can be processed immediately
//         if (!processed)
//         {
//             processed = RequestDispatchProcessDirectly(Device, Request, requestParameters);
//         }
//
//         // 2. Requests that should be put in sequential queue.
//         if (!processed)
//         {
//             processed = RequestDispatchToSequentialQueue(Device, Request, requestParameters);
//         }
//
//         // 3. Requests that need to be processed sequentially and in caller's context.
//         if (!processed)
//         {
//             processed = RequestDispatchSyncWithSequentialQueue(Device, Request, requestParameters);
//         }
//
//         // 4. Special requests that needs different process in different cases.
//         if (!processed)
//         {
//             processed = RequestDispatchSpecialIoctls(Device, Request, requestParameters);
//         }
//
//         // 5. This is default behavior for unknown IOCTLs. To pass it to lower level.
//         if (!processed)
//         {
//             processed = RequestDispatchUnknownRequests(Device, Request, requestParameters);
//         }
//
//         // All requests should be processed already.
//         NT_ASSERT(processed);
//         UNREFERENCED_PARAMETER(processed); //defensive coding, avoid PREFAST warning.
//     }
//     else if (requestParameters.Type == WdfRequestTypeDeviceControlInternal)
//     {
//         RequestProcessInternalDeviceControl(Request, deviceExtension);
//     }
//     else
//     {
//         // Requests other than IOCTLs will be forwarded to default queue.
//         status = WdfDeviceEnqueueRequest(Device, Request);
//         if (!NT_SUCCESS(status))
//         {
//             RequestCompletion(deviceExtension, Request, status, WdfRequestGetInformation(Request));
//         }
//     }
//
//     return;
// }
//
//
// BOOLEAN
// RequestDispatchProcessDirectly(
//      WDFDEVICE              Device,
//      WDFREQUEST             Request,
//      WDF_REQUEST_PARAMETERS RequestParameters
//     )
// /*++
// Routine Description:
//
//     These requests can be processed in a non-serialized manner, most of them don't need to access device.
//
// Arguments:
//
//     Device - handle to a WDF Device object
//
//     Request - handle to the incoming WDF Request object
//
//     RequestParameters - request parameters
//
// Return Value:
//
//     BOOLEAN - TRUE (request processed); FALSE (request is not processed in this function).
//
// --*/
// {
//     NTSTATUS                status = STATUS_SUCCESS;
//     BOOLEAN                 processed = FALSE;
//     size_t                  dataLength = 0;
//
//     PCDROM_DEVICE_EXTENSION deviceExtension = DeviceGetExtension(Device);
//     ULONG                   ioctlCode = RequestParameters.Parameters.DeviceIoControl.IoControlCode;
//
//     switch (ioctlCode)
//     {
//
//     case IOCTL_CDROM_GET_INQUIRY_DATA:
//     {
//         status = RequestHandleGetInquiryData(deviceExtension, Request, RequestParameters, &dataLength);
//
//         processed = TRUE;
//         break; // complete the irp
//     }
//
//     case IOCTL_STORAGE_GET_MEDIA_TYPES_EX:
//     {
//         status = RequestHandleGetMediaTypeEx(deviceExtension, Request, &dataLength);
//
//         processed = TRUE;
//         break; // complete the irp
//     }
//
//     case IOCTL_MOUNTDEV_QUERY_UNIQUE_ID:
//     {
//         status = RequestHandleMountQueryUniqueId(deviceExtension, Request, RequestParameters, &dataLength);
//
//         processed = TRUE;
//         break; // complete the irp
//     }
//
//     case IOCTL_MOUNTDEV_QUERY_DEVICE_NAME:
//     {
//         status = RequestHandleMountQueryDeviceName(deviceExtension, Request, RequestParameters, &dataLength);
//
//         processed = TRUE;
//         break; // complete the irp
//     }
//
//     case IOCTL_MOUNTDEV_QUERY_SUGGESTED_LINK_NAME:
//     {
//         status = RequestHandleMountQuerySuggestedLinkName(deviceExtension, Request, RequestParameters, &dataLength);
//
//         processed = TRUE;
//         break; // complete the irp
//     }
//
//     case IOCTL_STORAGE_GET_DEVICE_NUMBER:
//     {
//         status = RequestHandleGetDeviceNumber(deviceExtension, Request, RequestParameters, &dataLength);
//
//         processed = TRUE;
//         break; // complete the irp
//     }
//
//     case IOCTL_STORAGE_GET_HOTPLUG_INFO:
//     {
//         status = RequestHandleGetHotPlugInfo(deviceExtension, Request, RequestParameters, &dataLength);
//
//         processed = TRUE;
//         break; // complete the irp
//     }
//
//     case IOCTL_STORAGE_SET_HOTPLUG_INFO:
//     {
//         status = RequestHandleSetHotPlugInfo(deviceExtension, Request, RequestParameters, &dataLength);
//
//         processed = TRUE;
//         break; // complete the irp
//     }
//
//     case IOCTL_STORAGE_EVENT_NOTIFICATION:
//     {
//         status = RequestHandleEventNotification(deviceExtension, Request, &RequestParameters, &dataLength);
//
//         processed = TRUE;
//         break; // complete the irp
//     }
//
// #if (NTDDI_VERSION >= NTDDI_WIN8)
//     case IOCTL_VOLUME_ONLINE:
//     {
//         //
//         // Mount manager and volume manager will
//         // follow this online with a post online
//         // but other callers may not. In those
//         // cases, we process this request right
//         // away. We approximate that these other
//         // callers are from user mode
//         //
//
//         if (WdfRequestGetRequestorMode(Request) == KernelMode)
//         {
//             processed = TRUE;
//         }
//         break;
//     }
// #endif
//
//     default:
//     {
//         processed = FALSE;
//         break;
//     }
//
//     } //end of switch (ioctlCode)
//
//     if (processed)
//     {
//         UCHAR currentStackLocationFlags = 0;
//         currentStackLocationFlags = RequestGetCurrentStackLocationFlags(Request);
//
//         if ((status == STATUS_VERIFY_REQUIRED) &&
//             (currentStackLocationFlags & SL_OVERRIDE_VERIFY_VOLUME))
//         {
//             // If the status is verified required and this request
//             // should bypass verify required then retry the request.
//             status = STATUS_IO_DEVICE_ERROR;
//             UNREFERENCED_PARAMETER(status); // disables prefast warning; defensive coding...
//
//             processed = RequestDispatchProcessDirectly(Device, Request, RequestParameters);
//         }
//         else
//         {
//             // Complete the request after processing it.
//             RequestCompletion(deviceExtension, Request, status, dataLength);
//         }
//     }
//
//     return processed;
// }
//
//
// BOOLEAN
// RequestDispatchToSequentialQueue(
//      WDFDEVICE              Device,
//      WDFREQUEST             Request,
//      WDF_REQUEST_PARAMETERS RequestParameters
//     )
// /*++
// Routine Description:
//
//     These requests can be processed in a non-serialized manner, most of them don't need to access device.
//
// Arguments:
//
//     Device - handle to a WDF Device object
//
//     Request - handle to the incoming WDF Request object
//
//     RequestParameters - request parameters
//
// Return Value:
//
//     BOOLEAN - TRUE (request processed); FALSE (request is not processed in this function).
//
// --*/
// {
//     NTSTATUS                status = STATUS_SUCCESS;
//     BOOLEAN                 processed = FALSE;
//     size_t                  dataLength = 0;
//     BOOLEAN                 inZeroPowerState = FALSE;
//
//     PCDROM_DEVICE_EXTENSION deviceExtension = DeviceGetExtension(Device);
//     ULONG                   ioctlCode = RequestParameters.Parameters.DeviceIoControl.IoControlCode;
//     PZERO_POWER_ODD_INFO    zpoddInfo = deviceExtension->ZeroPowerODDInfo;
//
//     if ((zpoddInfo != NULL) &&
//         (zpoddInfo->InZeroPowerState != FALSE))
//     {
//         inZeroPowerState = TRUE;
//     }
//
//     switch (ioctlCode)
//     {
//
//     case IOCTL_CDROM_RAW_READ:
//     {
//         status = RequestValidateRawRead(deviceExtension, Request, RequestParameters, &dataLength);
//
//         processed = TRUE;
//         break;
//     }
//
//     case IOCTL_DISK_GET_DRIVE_GEOMETRY_EX:
//     case IOCTL_CDROM_GET_DRIVE_GEOMETRY_EX:
//     {
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL,
//                     "RequestDispatchToSequentialQueue: Get drive geometryEx\n"));
//         if ( RequestParameters.Parameters.DeviceIoControl.OutputBufferLength <
//              (ULONG)FIELD_OFFSET(DISK_GEOMETRY_EX, Data))
//         {
//             status = STATUS_BUFFER_TOO_SMALL;
//             dataLength = FIELD_OFFSET(DISK_GEOMETRY_EX, Data);
//         }
//         else if (inZeroPowerState != FALSE)
//         {
//             status = STATUS_NO_MEDIA_IN_DEVICE;
//         }
//         else
//         {
//             status = STATUS_SUCCESS;
//         }
//
//         processed = TRUE;
//         break;
//     }
//
//     case IOCTL_DISK_GET_DRIVE_GEOMETRY:
//     case IOCTL_CDROM_GET_DRIVE_GEOMETRY:
//     {
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL,
//                     "RequestDispatchToSequentialQueue: Get drive geometry\n"));
//         if (RequestParameters.Parameters.DeviceIoControl.OutputBufferLength <
//             sizeof(DISK_GEOMETRY))
//         {
//             status = STATUS_BUFFER_TOO_SMALL;
//             dataLength = sizeof(DISK_GEOMETRY);
//         }
//         else if (inZeroPowerState != FALSE)
//         {
//             status = STATUS_NO_MEDIA_IN_DEVICE;
//         }
//         else
//         {
//             status = STATUS_SUCCESS;
//         }
//
//         processed = TRUE;
//         break;
//     }
//
//     case IOCTL_CDROM_READ_TOC_EX:
//     {
//         status = RequestValidateReadTocEx(deviceExtension, Request, RequestParameters, &dataLength);
//
//         if (inZeroPowerState != FALSE)
//         {
//             status = STATUS_NO_MEDIA_IN_DEVICE;
//         }
//
//         processed = TRUE;
//         break;
//     }
//
//     case IOCTL_CDROM_READ_TOC:
//     {
//         status = RequestValidateReadToc(deviceExtension, RequestParameters, &dataLength);
//
//         if (inZeroPowerState != FALSE)
//         {
//             status = STATUS_NO_MEDIA_IN_DEVICE;
//         }
//
//         processed = TRUE;
//         break;
//     }
//
//     case IOCTL_CDROM_GET_LAST_SESSION:
//     {
//         status = RequestValidateGetLastSession(deviceExtension, RequestParameters, &dataLength);
//
//         processed = TRUE;
//         break;
//     }
//
//     case IOCTL_CDROM_PLAY_AUDIO_MSF:
//     {
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL,
//                     "RequestDispatchToSequentialQueue: Play audio MSF\n"));
//
//         if (RequestParameters.Parameters.DeviceIoControl.InputBufferLength <
//             sizeof(CDROM_PLAY_AUDIO_MSF))
//         {
//             status = STATUS_INFO_LENGTH_MISMATCH;
//         }
//         else
//         {
//             status = STATUS_SUCCESS;
//         }
//
//         processed = TRUE;
//         break;
//     }
//
//     case IOCTL_CDROM_SEEK_AUDIO_MSF:
//     {
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL,
//                     "RequestDispatchToSequentialQueue: Seek audio MSF\n"));
//
//         if (RequestParameters.Parameters.DeviceIoControl.InputBufferLength <
//             sizeof(CDROM_SEEK_AUDIO_MSF))
//         {
//             status = STATUS_INFO_LENGTH_MISMATCH;
//         }
//         else
//         {
//             status = STATUS_SUCCESS;
//         }
//
//         processed = TRUE;
//         break;
//     }
//
//     case IOCTL_CDROM_PAUSE_AUDIO:
//     {
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL,
//                     "RequestDispatchToSequentialQueue: Pause audio\n"));
//
//         status = STATUS_SUCCESS;
//         processed = TRUE;
//         break;
//     }
//
//     case IOCTL_CDROM_RESUME_AUDIO:
//     {
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL,
//                     "RequestDispatchToSequentialQueue: Resume audio\n"));
//
//         status = STATUS_SUCCESS;
//         processed = TRUE;
//         break;
//     }
//
//     case IOCTL_CDROM_READ_Q_CHANNEL:
//     {
//         status = RequestValidateReadQChannel(Request, RequestParameters, &dataLength);
//
//         processed = TRUE;
//         break;
//     }
//
//     case IOCTL_CDROM_GET_VOLUME:
//     {
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL,
//                     "RequestDispatchToSequentialQueue: Get volume control\n"));
//
//         if (RequestParameters.Parameters.DeviceIoControl.OutputBufferLength <
//             sizeof(VOLUME_CONTROL))
//         {
//             status = STATUS_BUFFER_TOO_SMALL;
//             dataLength = sizeof(VOLUME_CONTROL);
//         }
//         else
//         {
//             status = STATUS_SUCCESS;
//         }
//
//         processed = TRUE;
//         break;
//     }
//
//     case IOCTL_CDROM_SET_VOLUME:
//     {
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL,
//                     "RequestDispatchToSequentialQueue: Set volume control\n"));
//
//         if (RequestParameters.Parameters.DeviceIoControl.InputBufferLength <
//             sizeof(VOLUME_CONTROL))
//         {
//             status = STATUS_INFO_LENGTH_MISMATCH;
//         }
//         else
//         {
//             status = STATUS_SUCCESS;
//         }
//
//         processed = TRUE;
//         break;
//     }
//
//     case IOCTL_CDROM_STOP_AUDIO:
//     {
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL,
//                     "RequestDispatchToSequentialQueue: Stop audio\n"));
//
//         status = STATUS_SUCCESS;
//         processed = TRUE;
//         break;
//     }
//
//     case IOCTL_STORAGE_CHECK_VERIFY:
//     case IOCTL_STORAGE_CHECK_VERIFY2:
//     {
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL,
//                     "RequestDispatchToSequentialQueue: [%p] Check Verify\n", Request));
//
//         // Following check will let the condition "OutputBufferLength == 0" pass.
//         // Since it's legacy behavior in classpnp, we need to keep it.
//         if ((RequestParameters.Parameters.DeviceIoControl.OutputBufferLength > 0) &&
//             (RequestParameters.Parameters.DeviceIoControl.OutputBufferLength < sizeof(ULONG)))
//         {
//             status = STATUS_BUFFER_TOO_SMALL;
//             dataLength = sizeof(ULONG);
//         }
//         else if (inZeroPowerState != FALSE)
//         {
//             status = STATUS_NO_MEDIA_IN_DEVICE;
//         }
//         else
//         {
//             status = STATUS_SUCCESS;
//         }
//
//         processed = TRUE;
//         break;
//     }
//
//     case IOCTL_DVD_GET_REGION:
//     {
//         // validation will be done when process it.
//         status = STATUS_SUCCESS;
//         processed = TRUE;
//         break;
//     }
//
//     case IOCTL_DVD_READ_STRUCTURE:
//     {
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL,
//                     "RequestDispatchToSequentialQueue: [%p] IOCTL_DVD_READ_STRUCTURE\n", Request));
//
//         status = RequestValidateDvdReadStructure(deviceExtension, RequestParameters, &dataLength);
//
//         processed = TRUE;
//         break;
//     }
//
//     case IOCTL_DVD_READ_KEY:
//     {
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL,
//                     "RequestDispatchToSequentialQueue: [%p] IOCTL_DVD_READ_KEY\n", Request));
//
//         status = RequestValidateDvdReadKey(deviceExtension, Request, RequestParameters, &dataLength);
//
//         processed = TRUE;
//         break;
//     }
//
//     case IOCTL_DVD_START_SESSION:
//     {
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL,
//                     "RequestDispatchToSequentialQueue: [%p] IOCTL_DVD_START_SESSION\n", Request));
//
//         status = RequestValidateDvdStartSession(deviceExtension, RequestParameters, &dataLength);
//
//         processed = TRUE;
//         break;
//     }
//
//     case IOCTL_DVD_SEND_KEY:
//     case IOCTL_DVD_SEND_KEY2:
//     {
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL,
//                     "RequestDispatchToSequentialQueue: [%p] IOCTL_DVD_SEND_KEY\n", Request));
//
//         status = RequestValidateDvdSendKey(deviceExtension, Request, RequestParameters, &dataLength);
//
//         processed = TRUE;
//         break;
//     }
//
//     case IOCTL_STORAGE_SET_READ_AHEAD:
//     {
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL,
//                     "RequestDispatchToSequentialQueue: [%p] SetReadAhead\n", Request));
//
//         if (RequestParameters.Parameters.DeviceIoControl.InputBufferLength <
//             sizeof(STORAGE_SET_READ_AHEAD))
//         {
//             status = STATUS_INVALID_PARAMETER;
//         }
//         else
//         {
//             status = STATUS_SUCCESS;
//         }
//
//         processed = TRUE;
//         break;
//     }
//
//     case IOCTL_DISK_IS_WRITABLE:
//     {
//         status = STATUS_SUCCESS;
//
//         processed = TRUE;
//         break;
//     }
//
//     case IOCTL_DISK_GET_DRIVE_LAYOUT:
//     {
//         ULONG requiredSize = 0;
//
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL,
//                     "RequestDispatchToSequentialQueue: Get drive layout\n"));
//
//         requiredSize = FIELD_OFFSET(DRIVE_LAYOUT_INFORMATION, PartitionEntry[1]);
//
//         if (RequestParameters.Parameters.DeviceIoControl.OutputBufferLength <
//             requiredSize)
//         {
//             status = STATUS_BUFFER_TOO_SMALL;
//             dataLength = requiredSize;
//         }
//         else
//         {
//             status = STATUS_SUCCESS;
//         }
//
//         processed = TRUE;
//         break;
//     }
//
//     case IOCTL_DISK_GET_DRIVE_LAYOUT_EX:
//     {
//         ULONG requiredSize = 0;
//
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL,
//                     "RequestDispatchToSequentialQueue: Get drive layoutEx\n"));
//
//         requiredSize = FIELD_OFFSET(DRIVE_LAYOUT_INFORMATION_EX, PartitionEntry[1]);
//
//         if (RequestParameters.Parameters.DeviceIoControl.OutputBufferLength <
//             requiredSize)
//         {
//             status = STATUS_BUFFER_TOO_SMALL;
//             dataLength = requiredSize;
//         }
//         else
//         {
//             status = STATUS_SUCCESS;
//         }
//
//         processed = TRUE;
//         break;
//     }
//
//     case IOCTL_DISK_GET_PARTITION_INFO:
//     {
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL,
//                     "RequestDispatchToSequentialQueue: Get Partition Info\n"));
//
//         if (RequestParameters.Parameters.DeviceIoControl.OutputBufferLength <
//             sizeof(PARTITION_INFORMATION))
//         {
//             status = STATUS_BUFFER_TOO_SMALL;
//             dataLength = sizeof(PARTITION_INFORMATION);
//         }
//         else
//         {
//             status = STATUS_SUCCESS;
//         }
//
//         processed = TRUE;
//         break;
//     }
//
//     case IOCTL_DISK_GET_PARTITION_INFO_EX:
//     {
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL,
//                     "RequestDispatchToSequentialQueue: Get Partition InfoEx\n"));
//
//         if (RequestParameters.Parameters.DeviceIoControl.OutputBufferLength <
//             sizeof(PARTITION_INFORMATION_EX))
//         {
//             status = STATUS_BUFFER_TOO_SMALL;
//             dataLength = sizeof(PARTITION_INFORMATION_EX);
//         }
//         else
//         {
//             status = STATUS_SUCCESS;
//         }
//
//         processed = TRUE;
//         break;
//     }
//
//     case IOCTL_DISK_VERIFY:
//     {
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL,
//                     "RequestDispatchToSequentialQueue: IOCTL_DISK_VERIFY to device %p through request %p\n",
//                     Device,
//                     Request));
//
//         if (RequestParameters.Parameters.DeviceIoControl.InputBufferLength <
//             sizeof(VERIFY_INFORMATION))
//         {
//             status = STATUS_INVALID_PARAMETER;
//         }
//         else
//         {
//             status = STATUS_SUCCESS;
//         }
//
//         processed = TRUE;
//         break;
//     }
//
//     case IOCTL_DISK_GET_LENGTH_INFO:
//     {
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL,
//                     "RequestDispatchToSequentialQueue: Disk Get Length InfoEx\n"));
//
//         if (RequestParameters.Parameters.DeviceIoControl.OutputBufferLength <
//             sizeof(GET_LENGTH_INFORMATION))
//         {
//             status = STATUS_BUFFER_TOO_SMALL;
//             dataLength = sizeof(GET_LENGTH_INFORMATION);
//         }
//         else if (inZeroPowerState != FALSE)
//         {
//             status = STATUS_NO_MEDIA_IN_DEVICE;
//         }
//         else
//         {
//             status = STATUS_SUCCESS;
//         }
//
//         processed = TRUE;
//         break;
//     }
//
//     case IOCTL_CDROM_GET_CONFIGURATION:
//     {
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL,
//                     "RequestDispatchToSequentialQueue: [%p] IOCTL_CDROM_GET_CONFIGURATION\n", Request));
//
//         status = RequestValidateGetConfiguration(deviceExtension, Request, RequestParameters, &dataLength);
//
//         processed = TRUE;
//         break;
//     }
//
//     case IOCTL_CDROM_SET_SPEED:
//     {
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL,
//                     "RequestDispatchToSequentialQueue: [%p] IOCTL_CDROM_SET_SPEED\n", Request));
//
//         status = RequestValidateSetSpeed(deviceExtension, Request, RequestParameters, &dataLength);
//
//         processed = TRUE;
//         break;
//     }
//
//     case IOCTL_DVD_END_SESSION:
//     {
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL,
//                     "RequestDispatchToSequentialQueue: [%p] IOCTL_DVD_END_SESSION\n", Request));
//
//         status = RequestValidateDvdEndSession(deviceExtension, Request, RequestParameters, &dataLength);
//
//         processed = TRUE;
//         break;
//     }
//
//     case IOCTL_AACS_END_SESSION:
//     {
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL,
//                     "RequestDispatchToSequentialQueue: [%p] IOCTL_AACS_END_SESSION\n", Request));
//
//         status = RequestValidateAacsEndSession(deviceExtension, Request, RequestParameters, &dataLength);
//
//         processed = TRUE;
//         break;
//     }
//
//     case IOCTL_AACS_READ_MEDIA_KEY_BLOCK:
//     {
//         TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_IOCTL,
//                     "AACS: Querying full MKB with bufferSize of %x bytes\n",
//                     (int)RequestParameters.Parameters.DeviceIoControl.OutputBufferLength
//                     ));
//
//         status = RequestValidateAacsReadMediaKeyBlock(deviceExtension, Request, RequestParameters, &dataLength);
//
//         processed = TRUE;
//         break;
//     }
//
//     case IOCTL_AACS_START_SESSION:
//     {
//         TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_IOCTL,
//                     "AACS: Requesting AGID\n"
//                     ));
//
//         status = RequestValidateAacsStartSession(deviceExtension, RequestParameters, &dataLength);
//
//         processed = TRUE;
//         break;
//     }
//
//     case IOCTL_AACS_SEND_CERTIFICATE:
//     {
//         TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_IOCTL,
//                     "AACS: Sending host certificate to drive\n"
//                     ));
//
//         status = RequestValidateAacsSendCertificate(deviceExtension, Request, RequestParameters, &dataLength);
//
//         processed = TRUE;
//         break;
//     }
//
//     case IOCTL_AACS_GET_CERTIFICATE:
//     {
//         TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_IOCTL,
//                     "AACS: Querying drive certificate\n"
//                     ));
//
//         status = RequestValidateAacsGetCertificate(deviceExtension, Request, RequestParameters, &dataLength);
//
//         processed = TRUE;
//         break;
//     }
//
//     case IOCTL_AACS_GET_CHALLENGE_KEY:
//     {
//         TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_IOCTL,
//                     "AACS: Querying drive challenge key\n"
//                     ));
//
//         status = RequestValidateAacsGetChallengeKey(deviceExtension, Request, RequestParameters, &dataLength);
//
//         processed = TRUE;
//         break;
//     }
//
//     case IOCTL_AACS_SEND_CHALLENGE_KEY:
//     {
//         TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_IOCTL,
//                     "AACS: Sending drive challenge key\n"
//                     ));
//
//         status = RequestValidateAacsSendChallengeKey(deviceExtension, Request, RequestParameters, &dataLength);
//
//         processed = TRUE;
//         break;
//     }
//
//     case IOCTL_AACS_READ_VOLUME_ID:
//     {
//         TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_IOCTL,
//                     "AACS: Reading volume ID\n"
//                     ));
//
//         status = RequestValidateAacsReadVolumeId(deviceExtension, Request, RequestParameters, &dataLength);
//
//         processed = TRUE;
//         break;
//     }
//
//     case IOCTL_AACS_READ_SERIAL_NUMBER:
//     {
//         TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_IOCTL,
//                     "AACS: Reading Serial Number\n"
//                     ));
//
//         status = RequestValidateAacsReadSerialNumber(deviceExtension, Request, RequestParameters, &dataLength);
//
//         processed = TRUE;
//         break;
//     }
//
//     case IOCTL_AACS_READ_MEDIA_ID:
//     {
//         TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_IOCTL,
//                     "AACS: Reading media ID\n"
//                     ));
//
//         status = RequestValidateAacsReadMediaId(deviceExtension, Request, RequestParameters, &dataLength);
//
//         processed = TRUE;
//         break;
//     }
//
//     case IOCTL_AACS_READ_BINDING_NONCE:
//     case IOCTL_AACS_GENERATE_BINDING_NONCE:
//     {
//         if (ioctlCode == IOCTL_AACS_GENERATE_BINDING_NONCE)
//         {
//             TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_IOCTL,
//                         "AACS: Generating new binding nonce\n"
//                         ));
//         }
//         else
//         {
//             TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_IOCTL,
//                         "AACS: Reading existing binding nonce\n"
//                         ));
//         }
//
//         status = RequestValidateAacsBindingNonce(deviceExtension, Request, RequestParameters, &dataLength);
//
//         processed = TRUE;
//         break;
//     }
//
//     case IOCTL_CDROM_ENABLE_STREAMING:
//     {
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL,
//                     "RequestDispatchToSequentialQueue: [%p] IOCTL_CDROM_ENABLE_STREAMING\n", Request));
//
//         status = RequestValidateEnableStreaming(Request, RequestParameters, &dataLength);
//
//         processed = TRUE;
//         break;
//     }
//
//     case IOCTL_CDROM_SEND_OPC_INFORMATION:
//     {
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL,
//                     "RequestDispatchToSequentialQueue: [%p] IOCTL_CDROM_SEND_OPC_INFORMATION\n", Request));
//
//         status = RequestValidateSendOpcInformation(Request, RequestParameters, &dataLength);
//
//         processed = TRUE;
//         break;
//     }
//
//     case IOCTL_CDROM_GET_PERFORMANCE:
//     {
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL,
//                     "RequestDispatchToSequentialQueue: [%p] IOCTL_CDROM_GET_PERFORMANCE\n", Request));
//
//         status = RequestValidateGetPerformance(Request, RequestParameters, &dataLength);
//
//         processed = TRUE;
//         break;
//     }
//
//     case IOCTL_STORAGE_MEDIA_REMOVAL:
//     case IOCTL_STORAGE_EJECTION_CONTROL:
//     {
//         if(RequestParameters.Parameters.DeviceIoControl.InputBufferLength <
//            sizeof(PREVENT_MEDIA_REMOVAL))
//         {
//             status = STATUS_INFO_LENGTH_MISMATCH;
//         }
//         else
//         {
//             status = STATUS_SUCCESS;
//         }
//
//         processed = TRUE;
//         break; // complete the irp
//     }
//
//     case IOCTL_STORAGE_MCN_CONTROL:
//     {
//         if(RequestParameters.Parameters.DeviceIoControl.InputBufferLength <
//             sizeof(PREVENT_MEDIA_REMOVAL))
//         {
//             status = STATUS_INFO_LENGTH_MISMATCH;
//         }
//         else
//         {
//             status = STATUS_SUCCESS;
//         }
//
//         processed = TRUE;
//         break; // complete the irp
//     }
//
//     case IOCTL_STORAGE_RESERVE:
//     case IOCTL_STORAGE_RELEASE:
//     {
//         // there is no validate check currently.
//         status = STATUS_SUCCESS;
//         processed = TRUE;
//         break;
//     }
//
//     case IOCTL_STORAGE_PERSISTENT_RESERVE_IN:
//     case IOCTL_STORAGE_PERSISTENT_RESERVE_OUT:
//     {
//         status = RequestValidatePersistentReserve(deviceExtension, Request, RequestParameters, &dataLength);
//
//         processed = TRUE;
//         break;
//     }
//
//     case IOCTL_STORAGE_EJECT_MEDIA:
//     case IOCTL_STORAGE_LOAD_MEDIA:
//     case IOCTL_STORAGE_LOAD_MEDIA2:
//     {
//         status = STATUS_SUCCESS;
//
//         processed = TRUE;
//         break; // complete the irp
//     }
//
//     case IOCTL_STORAGE_FIND_NEW_DEVICES:
//     {
//         // process it.
//         IoInvalidateDeviceRelations(deviceExtension->LowerPdo, BusRelations);
//
//         status = STATUS_SUCCESS;
//
//         processed = TRUE;
//         break; // complete the irp
//     }
//
//     case IOCTL_STORAGE_READ_CAPACITY:
//     {
//         if (RequestParameters.Parameters.DeviceIoControl.OutputBufferLength < sizeof(STORAGE_READ_CAPACITY))
//         {
//             dataLength = sizeof(STORAGE_READ_CAPACITY);
//             status = STATUS_BUFFER_TOO_SMALL;
//         }
//         else if (inZeroPowerState != FALSE)
//         {
//             status = STATUS_NO_MEDIA_IN_DEVICE;
//         }
//         else
//         {
//             status = STATUS_SUCCESS;
//         }
//
//         processed = TRUE;
//         break; // complete the irp
//     }
//
//     case IOCTL_STORAGE_CHECK_PRIORITY_HINT_SUPPORT:
//     {
//         // for disk.sys only in original classpnp
//         status = STATUS_NOT_SUPPORTED;
//
//         processed = TRUE;
//         break; // complete the irp
//     }
//
// #if (NTDDI_VERSION >= NTDDI_WIN8)
//     case IOCTL_DISK_ARE_VOLUMES_READY:
//     {
//         // this request doesn't access device at all, so seemingly it can be processed
//         // directly; however, in case volume online is not received, we will need to
//         // park these requests in a queue, and the only way a request can be queued is
//         // if the request came out of another queue.
//         status = STATUS_SUCCESS;
//
//         processed = TRUE;
//         break;
//     }
//
//     case IOCTL_VOLUME_ONLINE:
//     case IOCTL_VOLUME_POST_ONLINE:
//     {
//         status = STATUS_SUCCESS;
//
//         processed = TRUE;
//         break;
//     }
// #endif
//
//     default:
//     {
//         processed = FALSE;
//         break;
//     }
//     } //end of switch (ioctlCode)
//
//     if (processed)
//     {
//         UCHAR currentStackLocationFlags = 0;
//         currentStackLocationFlags = RequestGetCurrentStackLocationFlags(Request);
//
//         if ((status == STATUS_VERIFY_REQUIRED) &&
//             (currentStackLocationFlags & SL_OVERRIDE_VERIFY_VOLUME))
//         {
//             // If the status is verified required and this request
//             // should bypass verify required then retry the request.
//             status = STATUS_IO_DEVICE_ERROR;
//             UNREFERENCED_PARAMETER(status); // disables prefast warning; defensive coding...
//
//             processed = RequestDispatchToSequentialQueue(Device, Request, RequestParameters);
//         }
//         else
//         {
//             if (NT_SUCCESS(status))
//             {
//                 // Forward the request to serialized queue.
//                 status = WdfDeviceEnqueueRequest(Device, Request);
//             }
//
//             if (!NT_SUCCESS(status))
//             {
//                 // Validation failed / forward failed, complete the request.
//                 RequestCompletion(deviceExtension, Request, status, dataLength);
//             }
//         }
//     }
//
//     return processed;
// }
//
//
// BOOLEAN
// RequestDispatchSyncWithSequentialQueue(
//      WDFDEVICE              Device,
//      WDFREQUEST             Request,
//      WDF_REQUEST_PARAMETERS RequestParameters
//     )
// /*++
// Routine Description:
//
//     These requests need to stay in caller's context and be processed in serialized manner.
//
// Arguments:
//
//     Device - handle to a WDF Device object
//
//     Request - handle to the incoming WDF Request object
//
//     RequestParameters - request parameters
//
// Return Value:
//
//     BOOLEAN - TRUE (request processed); FALSE (request is not processed in this function).
//
// --*/
// {
//     NTSTATUS                status = STATUS_SUCCESS;
//     BOOLEAN                 processed = FALSE;
//     size_t                  dataLength = 0;
//
//     PCDROM_DEVICE_EXTENSION deviceExtension = DeviceGetExtension(Device);
//     ULONG                   ioctlCode = RequestParameters.Parameters.DeviceIoControl.IoControlCode;
//
//     switch (ioctlCode)
//     {
//
//     case IOCTL_CDROM_EXCLUSIVE_ACCESS:
//     {
//         //1. Validate
//         status =  RequestValidateExclusiveAccess(Request, RequestParameters, &dataLength);
//
//         //2. keep the request in serialized manner and stay in user's context.
//         if (NT_SUCCESS(status))
//         {
//             PCDROM_EXCLUSIVE_ACCESS exclusiveAccess = NULL;
//
//             status = WdfRequestRetrieveInputBuffer(Request,
//                                                    RequestParameters.Parameters.DeviceIoControl.InputBufferLength,
//                                                    &exclusiveAccess,
//                                                    NULL);
//
//             if (NT_SUCCESS(status))
//             {
//                 // do not need to check "status" as it passed validation and cannot fail in WdfRequestRetrieveInputBuffer()
//                 switch (exclusiveAccess->RequestType)
//                 {
//
//                     case ExclusiveAccessQueryState:
//                     {
//                         status = RequestSetContextFields(Request, RequestHandleExclusiveAccessQueryLockState);
//                         break;
//                     }
//
//                     case ExclusiveAccessLockDevice:
//                     {
//                         status = RequestSetContextFields(Request, RequestHandleExclusiveAccessLockDevice);
//                         break;
//                     }
//
//                     case ExclusiveAccessUnlockDevice:
//                     {
//                         status = RequestSetContextFields(Request, RequestHandleExclusiveAccessUnlockDevice);
//                         break;
//                     }
//                     default:
//                     {
//                         // already valicated in RequestValidateExclusiveAccess()
//                         NT_ASSERT(FALSE);
//                         break;
//                     }
//                 }
//             }
//
//             if (NT_SUCCESS(status))
//             {
//                 // now, put the special synchronization information into the context
//                 status = RequestSynchronizeProcessWithSerialQueue(Device, Request);
//
//                 // "status" is used for debugging in above statement, reset to success to avoid further work in this function.
//                 status = STATUS_SUCCESS;
//             }
//         }
//
//         processed = TRUE;
//         break; // complete the irp
//     }
//
//     default:
//     {
//         processed = FALSE;
//         break;
//     }
//     } //end of switch (ioctlCode)
//
//     // Following process is only valid if the request is not really processed. (failed in validation)
//     if (processed && !NT_SUCCESS(status))
//     {
//         UCHAR currentStackLocationFlags = 0;
//         currentStackLocationFlags = RequestGetCurrentStackLocationFlags(Request);
//
//         if ((status == STATUS_VERIFY_REQUIRED) &&
//             (currentStackLocationFlags & SL_OVERRIDE_VERIFY_VOLUME))
//         {
//             //
//             // If the status is verified required and this request
//             // should bypass verify required then retry the request.
//             //
//             status = STATUS_IO_DEVICE_ERROR;
//             UNREFERENCED_PARAMETER(status); // disables prefast warning; defensive coding...
//
//             processed = RequestDispatchSyncWithSequentialQueue(Device, Request, RequestParameters);
//         }
//         else
//         {
//             // Validation failed / forward failed, complete the request.
//             RequestCompletion(deviceExtension, Request, status, dataLength);
//         }
//     }
//
//     return processed;
// }
//
//
// BOOLEAN
// RequestDispatchSpecialIoctls(
//      WDFDEVICE              Device,
//      WDFREQUEST             Request,
//      WDF_REQUEST_PARAMETERS RequestParameters
//     )
// /*++
// Routine Description:
//
//     These requests need to be processed in different manner according to input parameters
//
// Arguments:
//
//     Device - handle to a WDF Device object
//
//     Request - handle to the incoming WDF Request object
//
//     RequestParameters - request parameters
//
// Return Value:
//
//     BOOLEAN - TRUE (request processed); FALSE (request is not processed in this function).
//
// --*/
// {
//     NTSTATUS                status = STATUS_SUCCESS;
//     BOOLEAN                 processed = FALSE;
//     size_t                  dataLength = 0;
//     BOOLEAN                 requestCompleted = FALSE;
//
//     PCDROM_DEVICE_EXTENSION deviceExtension = DeviceGetExtension(Device);
//     PCDROM_DATA             cdData = &(deviceExtension->DeviceAdditionalData);
//     ULONG                   ioctlCode = RequestParameters.Parameters.DeviceIoControl.IoControlCode;
//
//     switch (ioctlCode)
//     {
//     case IOCTL_SCSI_PASS_THROUGH:
//     case IOCTL_SCSI_PASS_THROUGH_DIRECT:
//     case IOCTL_SCSI_PASS_THROUGH_EX:
//     case IOCTL_SCSI_PASS_THROUGH_DIRECT_EX:
//     {
//         // SPTI is considered special case as we need to set the MinorFunction before pass to low level.
//
// #if defined (_WIN64)
//         if (WdfRequestIsFrom32BitProcess(Request))
//         {
//             if ((ioctlCode == IOCTL_SCSI_PASS_THROUGH) || (ioctlCode == IOCTL_SCSI_PASS_THROUGH_DIRECT))
//             {
//                 if (RequestParameters.Parameters.DeviceIoControl.InputBufferLength < sizeof(SCSI_PASS_THROUGH32))
//                 {
//                     status = STATUS_INVALID_PARAMETER;
//                 }
//             }
//             else
//             {
//                 if (RequestParameters.Parameters.DeviceIoControl.InputBufferLength < sizeof(SCSI_PASS_THROUGH32_EX))
//                 {
//                     status = STATUS_INVALID_PARAMETER;
//                 }
//             }
//         }
//         else
// #endif
//         {
//             if ((ioctlCode == IOCTL_SCSI_PASS_THROUGH) || (ioctlCode == IOCTL_SCSI_PASS_THROUGH_DIRECT))
//             {
//                 if (RequestParameters.Parameters.DeviceIoControl.InputBufferLength < sizeof(SCSI_PASS_THROUGH))
//                 {
//                     status = STATUS_INVALID_PARAMETER;
//                 }
//             }
//             else
//             {
//                 if (RequestParameters.Parameters.DeviceIoControl.InputBufferLength < sizeof(SCSI_PASS_THROUGH_EX))
//                 {
//                     status = STATUS_INVALID_PARAMETER;
//                 }
//             }
//         }
//
//         if (!NT_SUCCESS(status))
//         {
//             // validation failed.
//             RequestCompletion(deviceExtension, Request, status, dataLength);
//         }
//         else
//         {
//             // keep the request in serialized manner and stay in user's context.
//             status = RequestSetContextFields(Request, Some(crate::ioctl::RequestHandleScsiPassThrough));
//
//             if (NT_SUCCESS(status))
//             {
//                 status = RequestSynchronizeProcessWithSerialQueue(Device, Request);
//             }
//             else
//             {
//                 RequestCompletion(deviceExtension, Request, status, 0);
//             }
//         }
//
//         requestCompleted = TRUE;
//         processed = TRUE;
//         break;
//     }
//
//     case IOCTL_STORAGE_QUERY_PROPERTY:
//     {
//         if (RequestParameters.Parameters.DeviceIoControl.InputBufferLength < sizeof(STORAGE_PROPERTY_QUERY))
//         {
//             status = STATUS_INFO_LENGTH_MISMATCH;
//         }
//         else
//         {
//             PSTORAGE_PROPERTY_QUERY inputBuffer = NULL;
//
//             status = WdfRequestRetrieveInputBuffer(Request,
//                                                    RequestParameters.Parameters.DeviceIoControl.InputBufferLength,
//                                                    &inputBuffer,
//                                                    NULL);
//
//             if (NT_SUCCESS(status))
//             {
//                 if (!EXCLUSIVE_MODE(cdData) ||                                     // not locked
//                     EXCLUSIVE_OWNER(cdData, WdfRequestGetFileObject(Request)) ||   // request is from lock owner
//                     (inputBuffer->QueryType == PropertyExistsQuery))               // request not access device
//                 {
//                     if (inputBuffer->PropertyId == StorageDeviceUniqueIdProperty)
//                     {
//                         // previously handled in classpnp
//                         // keep the request in serialized manner and stay in user's context.
//                         status = RequestSetContextFields(Request, Some(crate::ioctl::RequestHandleQueryPropertyDeviceUniqueId));
//
//                         if (NT_SUCCESS(status))
//                         {
//                             status = RequestSynchronizeProcessWithSerialQueue(Device, Request);
//                             // remeber that the request has been completed.
//                             requestCompleted = TRUE;
//                         }
//                     }
//                     else if (inputBuffer->PropertyId == StorageDeviceWriteCacheProperty)
//                     {
//                         // previously handled in classpnp
//                         // keep the request in serialized manner and stay in user's context.
//                         status = RequestSetContextFields(Request, Some(crate::ioctl::RequestHandleQueryPropertyWriteCache));
//
//                         if (NT_SUCCESS(status))
//                         {
//                             status = RequestSynchronizeProcessWithSerialQueue(Device, Request);
//                             // remeber that the request has been completed.
//                             requestCompleted = TRUE;
//                         }
//                     }
//                     else
//                     {
//                         // Pass to port driver for handling
//                         RequestDispatchUnknownRequests(Device, Request, RequestParameters);
//
//                         // remeber that the request has been completed.
//                         requestCompleted = TRUE;
//                     }
//                 }
//                 else
//                 {
//                     // If cached data exists, return cached data. Otherwise, fail the request.
//                     if ((inputBuffer->QueryType == PropertyStandardQuery) &&
//                         ((inputBuffer->PropertyId == StorageDeviceProperty) || (inputBuffer->PropertyId == StorageAdapterProperty)) )
//                     {
//                         status = RequestHandleQueryPropertyRetrieveCachedData(deviceExtension, Request, RequestParameters, &dataLength);
//                     }
//                     else
//                     {
//                         status = STATUS_ACCESS_DENIED;
//                     }
//                 }
//             }
//         }
//
//         processed = TRUE;
//         break;
//     }
//
//     // this IOCTL is a fake one, used for MCN process sync-ed with serial queue.
//     case IOCTL_MCN_SYNC_FAKE_IOCTL:
//     {
//         PIRP irp = WdfRequestWdmGetIrp(Request);
//
//         if ((deviceExtension->MediaChangeDetectionInfo != NULL) &&
//             (irp == deviceExtension->MediaChangeDetectionInfo->MediaChangeSyncIrp) &&
//             (WdfRequestGetRequestorMode(Request) == KernelMode) &&
//             (RequestParameters.Parameters.Others.Arg1 == RequestSetupMcnSyncIrp) &&
//             (RequestParameters.Parameters.Others.Arg2 == RequestSetupMcnSyncIrp) &&
//             (RequestParameters.Parameters.Others.Arg4 == RequestSetupMcnSyncIrp))
//         {
//             // This is the requset we use to sync Media Change Detection with sequential queue.
//             status = WdfDeviceEnqueueRequest(Device, Request);
//
//             if (!NT_SUCCESS(status))
//             {
//                 RequestCompletion(deviceExtension, Request, status, dataLength);
//             }
//
//             requestCompleted = TRUE;
//             processed = TRUE;
//         }
//         else
//         {
//             // process as an unknown request.
//             processed = FALSE;
//         }
//         break;
//     }
//
//     default:
//     {
//         processed = FALSE;
//         break;
//     }
//     } //end of switch (ioctlCode)
//
//     if (processed && !requestCompleted)
//     {
//         UCHAR currentStackLocationFlags = 0;
//         currentStackLocationFlags = RequestGetCurrentStackLocationFlags(Request);
//
//         if ((status == STATUS_VERIFY_REQUIRED) &&
//             (currentStackLocationFlags & SL_OVERRIDE_VERIFY_VOLUME))
//         {
//             // If the status is verified required and this request
//             // should bypass verify required then retry the request.
//             status = STATUS_IO_DEVICE_ERROR;
//             UNREFERENCED_PARAMETER(status); // disables prefast warning; defensive coding...
//
//             processed = RequestDispatchSpecialIoctls(Device, Request, RequestParameters);
//         }
//         else
//         {
//             RequestCompletion(deviceExtension, Request, status, dataLength);
//         }
//     }
//
//     return processed;
// }
//
//
// BOOLEAN
// RequestDispatchUnknownRequests(
//      WDFDEVICE              Device,
//      WDFREQUEST             Request,
//      WDF_REQUEST_PARAMETERS RequestParameters
//     )
// /*++
// Routine Description:
//
//     All unknown requests will be pass to lower level driver.
//     If IRQL is PASSIVE_LEVEL, the request will be serialized;
//     Otherwise, it'll be sent and forget.
//
// Arguments:
//
//     Device - handle to a WDF Device object
//
//     Request - handle to the incoming WDF Request object
//
//     RequestParameters - request parameters
//
// Return Value:
//
//     BOOLEAN - TRUE (request processed); FALSE (request is not processed in this function).
//
// --*/
// {
//     NTSTATUS                status = STATUS_UNSUCCESSFUL;
//     PCDROM_DEVICE_EXTENSION deviceExtension = DeviceGetExtension(Device);
//
//     ULONG       baseCode = DEVICE_TYPE_FROM_CTL_CODE(RequestParameters.Parameters.DeviceIoControl.IoControlCode);
//
//     if ((KeGetCurrentIrql() != PASSIVE_LEVEL) ||
//         (baseCode == FILE_DEVICE_ACPI))
//     {
//         // 1. When IRQL is higher than PASSIVE_LEVEL,
//         // 2. ataport sends IOCTL_ACPI_ASYNC_EVAL_METHOD before queue starts,
//         // send request directly to lower driver.
//         status = RequestHandleUnknownIoctl(Device, Request);
//     }
//     else
//     {
//         // keep the request in serialized manner and stay in user's context.
//         status = RequestSetContextFields(Request, Some(RequestHandleUnknownIoctl));
//
//         if (NT_SUCCESS(status))
//         {
//             status = RequestSynchronizeProcessWithSerialQueue(Device, Request);
//         }
//         else
//         {
//             RequestCompletion(deviceExtension, Request, status, 0);
//         }
//     }
//
//     UNREFERENCED_PARAMETER(status); //defensive coding, avoid PREFAST warning.
//
//     // All unknown IOCTLs are processed in this function.
//     return TRUE; //processed
// }
//
// VOID
// RequestProcessInternalDeviceControl(
//      WDFREQUEST              Request,
//      PCDROM_DEVICE_EXTENSION DeviceExtension
//     )
// /*++
// Routine Description:
//
//     all internal IOCTL will be send to lower driver asynchronously.
//
// Arguments:
//
//     Request - handle to the incoming WDF Request object
//     DeviceExtension - device extension structure
//
// Return Value:
//
//     None
//
// --*/
// {
//     NTSTATUS                 status = STATUS_SUCCESS;
//     PIRP                     irp = NULL;
//     PIO_STACK_LOCATION       irpStack = NULL;
//     PIO_STACK_LOCATION       nextStack = NULL;
//     BOOLEAN                  requestSent = FALSE;
//
//     irp = WdfRequestWdmGetIrp(Request);
//     irpStack = IoGetCurrentIrpStackLocation(irp);
//     nextStack = IoGetNextIrpStackLocation(irp);
//
//     // Set the parameters in the next stack location.
//     nextStack->Parameters.Scsi.Srb = irpStack->Parameters.Scsi.Srb;
//     nextStack->MajorFunction = IRP_MJ_SCSI;
//     nextStack->MinorFunction = IRP_MN_SCSI_CLASS;
//
//     WdfRequestSetCompletionRoutine(Request, RequestDummyCompletionRoutine, NULL);
//
//     status = RequestSend(DeviceExtension,
//                          Request,
//                          DeviceExtension->IoTarget,
//                          0,
//                          &requestSent);
//
//     // send the request straight down (asynchronously)
//     if (!requestSent)
//     {
//         // fail the request
//         RequestCompletion(DeviceExtension, Request, status, WdfRequestGetInformation(Request));
//     }
//
//     return;
// }
//
//
//
// //
// // Serial I/O Queue Event callbacks
// //
//
// VOID
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// SequentialQueueEvtIoReadWrite(
//      WDFQUEUE    Queue,
//      WDFREQUEST  Request,
//      size_t      Length
//     )
// /*++
// Routine Description:
//
//     validate and process read/write request.
//
// Arguments:
//
//     Queue - parallel queue itself
//
//     Request - handle to the incoming WDF Request object
//
//     Length - read / write lenght
//
// Return Value:
//
//     None
//
// --*/
// {
//     NTSTATUS                status = STATUS_SUCCESS;
//     WDFDEVICE               device = WdfIoQueueGetDevice(Queue);
//     PCDROM_DEVICE_EXTENSION deviceExtension = DeviceGetExtension(device);
//     WDF_REQUEST_PARAMETERS  requestParameters;
//     PIRP                    wdmIrp = WdfRequestWdmGetIrp(Request);
//     PIO_STACK_LOCATION      currentIrpStack = IoGetCurrentIrpStackLocation(wdmIrp);
//     PCDROM_DATA             cdData = &(deviceExtension->DeviceAdditionalData);
//
//     // Get the request parameters
//     WDF_REQUEST_PARAMETERS_INIT(&requestParameters);
//     WdfRequestGetParameters(Request, &requestParameters);
//
//     if (requestParameters.Type ==  WdfRequestTypeRead)
//     {
//         TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_GENERAL,
//                     "Receiving READ, Length %Ix\n", (ULONG) Length));
//     }
//     else
//     {
//         TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_GENERAL,
//                     "Receiving WRITE, Length %Ix\n", (ULONG) Length));
//     }
//
//     // Check if a verify is required before a READ/WRITE
//     if (TEST_FLAG(deviceExtension->DeviceObject->Flags, DO_VERIFY_VOLUME) &&
//         (requestParameters.MinorFunction != CDROM_VOLUME_VERIFY_CHECKED) &&
//         !TEST_FLAG(currentIrpStack->Flags, SL_OVERRIDE_VERIFY_VOLUME))
//     {
//         //  DO_VERIFY_VOLUME is set for the device object,
//         //  but this request is not itself a verify request.
//         //  So fail this request.
//         RequestCompletion(deviceExtension, Request, STATUS_VERIFY_REQUIRED, 0);
//     }
//     else
//     {
//         //  Since we've bypassed the verify-required tests we don't need to repeat
//         //  them with this IRP - in particular we don't want to worry about
//         //  hitting them at the partition 0 level if the request has come through
//         //  a non-zero partition.
//         currentIrpStack->MinorFunction = CDROM_VOLUME_VERIFY_CHECKED;
//
//         // Fail READ/WRITE requests when music is playing
//         if (deviceExtension->DeviceAdditionalData.PlayActive)
//         {
//             RequestCompletion(deviceExtension, Request, STATUS_DEVICE_BUSY, 0);
//
//             return;
//         }
//
//         // Fail READ/WRITE requests from non-owners if the drive is locked
//         if (EXCLUSIVE_MODE(cdData) && !EXCLUSIVE_OWNER(cdData, WdfRequestGetFileObject(Request)))
//         {
//             RequestCompletion(deviceExtension, Request, STATUS_ACCESS_DENIED, 0);
//
//             return;
//         }
//
//         // Succeed READ/WRITE requests of length 0
//         if (Length == 0)
//         {
//             //  Several parts of the code turn 0 into 0xffffffff,
//             //  so don't process a zero-length request any further.
//             RequestCompletion(deviceExtension, Request, STATUS_SUCCESS, Length);
//
//             return;
//         }
//
//         // If there is an unexpected write request, we want to rediscover MMC capabilities
//         if (!deviceExtension->DeviceAdditionalData.Mmc.WriteAllowed &&
//             (requestParameters.Type == WdfRequestTypeWrite))
//         {
//             // Schedule MMC capabilities update now, but perform it later in a work item
//             deviceExtension->DeviceAdditionalData.Mmc.UpdateState = CdromMmcUpdateRequired;
//         }
//
//         // If MMC capabilities update is required, we create a separate work item to avoid blocking
//         // the current thread; otherwise, we initiate an async read/write in the current thread.
//         if (DeviceIsMmcUpdateRequired(deviceExtension->Device))
//         {
//             deviceExtension->ReadWriteWorkItemContext.OriginalRequest = Request;
//             WdfWorkItemEnqueue(deviceExtension->ReadWriteWorkItem);
//
//             status = STATUS_SUCCESS;
//         }
//         else
//         {
//             status = RequestValidateReadWrite(deviceExtension, Request, requestParameters);
//
//             if (NT_SUCCESS(status))
//             {
//                 status = RequestHandleReadWrite(deviceExtension, Request, requestParameters);
//             }
//         }
//
//         if (!NT_SUCCESS(status))
//         {
//             RequestCompletion(deviceExtension, Request, status, 0);
//         }
//     }
//
//     return;
// }
//
//
// VOID
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// ReadWriteWorkItemRoutine(
//      WDFWORKITEM  WorkItem
//     )
// /*++
//
// Routine Description:
//
//     Work item routine for validating and initiating read and write requests.
//     The reason why we do that from a work item is because we may need to update MMC
//     capabilities before validating a read/write request and that is a sync operation.
//
// Arguments:
//
//     WorkItem - WDF work item
//
// Return Value:
//
//     none
//
// --*/
// {
//     PCDROM_DEVICE_EXTENSION             deviceExtension = NULL;
//     WDFREQUEST                          readWriteRequest = NULL;
//     WDF_REQUEST_PARAMETERS              readWriteRequestParameters;
//     NTSTATUS                            status = STATUS_SUCCESS;
//
//     PAGED_CODE ();
//
//     deviceExtension = WdfObjectGetTypedContext(WdfWorkItemGetParentObject(WorkItem), CDROM_DEVICE_EXTENSION);
//     readWriteRequest = deviceExtension->ReadWriteWorkItemContext.OriginalRequest;
//     deviceExtension->ReadWriteWorkItemContext.OriginalRequest = NULL;
//
//     WDF_REQUEST_PARAMETERS_INIT(&readWriteRequestParameters);
//     WdfRequestGetParameters(readWriteRequest, &readWriteRequestParameters);
//
//     if (DeviceIsMmcUpdateRequired(deviceExtension->Device))
//     {
//         // Issue command to update the drive capabilities.
//         // The failure of MMC update is not considered critical, so we'll
//         // continue to process the request even if MMC update fails.
//         (VOID) DeviceUpdateMmcCapabilities(deviceExtension->Device);
//     }
//
//     // Now verify and process the request
//     if (NT_SUCCESS(status))
//     {
//         status = RequestValidateReadWrite(deviceExtension, readWriteRequest, readWriteRequestParameters);
//     }
//     if (NT_SUCCESS(status))
//     {
//         status = RequestHandleReadWrite(deviceExtension, readWriteRequest, readWriteRequestParameters);
//     }
//
//     // Complete the request immediately on failure
//     if (!NT_SUCCESS(status))
//     {
//         RequestCompletion(deviceExtension, readWriteRequest, status, 0);
//     }
// }
//
//
// VOID
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// SequentialQueueEvtIoDeviceControl(
//      WDFQUEUE   Queue,
//      WDFREQUEST Request,
//      size_t     OutputBufferLength,
//      size_t     InputBufferLength,
//      ULONG      IoControlCode
//     )
// /*++
// Routine Description:
//
//     validate and process IOCTL request.
//
// Arguments:
//
//     Queue - sequential queue
//
//     Request - handle to the incoming WDF Request object
//
// Return Value:
//
//     None
//
// --*/
// {
//     NTSTATUS                status = STATUS_SUCCESS;
//     WDFDEVICE               device = WdfIoQueueGetDevice(Queue);
//     PCDROM_DEVICE_EXTENSION deviceExtension = DeviceGetExtension(device);
//     PCDROM_REQUEST_CONTEXT  requestContext = RequestGetContext(Request);
//     PCDROM_DATA             cdData = &(deviceExtension->DeviceAdditionalData);
//     WDF_REQUEST_PARAMETERS  requestParameters;
//
//     UNREFERENCED_PARAMETER(OutputBufferLength);
//     UNREFERENCED_PARAMETER(InputBufferLength);
//     UNREFERENCED_PARAMETER(IoControlCode);
//
//     // get the request parameters
//     WDF_REQUEST_PARAMETERS_INIT(&requestParameters);
//     WdfRequestGetParameters(Request, &requestParameters);
//
//     // If the device is in exclusive mode, check whether the request is from
//     // the handle that locked the device
//     if (EXCLUSIVE_MODE(cdData) && !EXCLUSIVE_OWNER(cdData, WdfRequestGetFileObject(Request)))
//     {
//         BOOLEAN isBlocked = FALSE;
//
//         status = RequestIsIoctlBlockedByExclusiveAccess(Request, &isBlocked);
//         if (NT_SUCCESS(status) && isBlocked)
//         {
//             if (requestContext->SyncRequired)
//             {
//                 // set the following event, so RequestSynchronizeProcessWithSerialQueue() can contintue run to process the real request.
//                 // this function will wait for the request process finishes.
//                 KeSetEvent(requestContext->SyncEvent, IO_CD_ROM_INCREMENT, FALSE);
//             }
//             else
//             {
//                 TracePrint!((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL,
//                             "DeviceEvtIoInCallerContext: Access Denied! Device in exclusive mode.Failing Ioctl %lx\n",
//                             requestParameters.Parameters.DeviceIoControl.IoControlCode));
//                 RequestCompletion(deviceExtension, Request, STATUS_ACCESS_DENIED, 0);
//             }
//
//             return;
//         }
//     }
//
//     if (!cdData->Mmc.WriteAllowed &&
//         ((requestParameters.Parameters.DeviceIoControl.IoControlCode == IOCTL_DISK_IS_WRITABLE) ||
//          (requestParameters.Parameters.DeviceIoControl.IoControlCode == IOCTL_DISK_VERIFY)))
//     {
//         cdData->Mmc.UpdateState = CdromMmcUpdateRequired;
//     }
//
//     // check if this is a synchronized ioctl
//     if (requestContext->SyncRequired)
//     {
//         // set the following event, so RequestSynchronizeProcessWithSerialQueue() can contintue run to process the real request.
//         // this function will wait for the request process finishes.
//         KeSetEvent(requestContext->SyncEvent, IO_CD_ROM_INCREMENT, FALSE);
//     }
//     else
//     {
//         deviceExtension->IoctlWorkItemContext.OriginalRequest = Request;
//
//         // all other IOCTL processing is currently processed via a
//         // work item running at PASSIVE_LEVEL.
//         WdfWorkItemEnqueue(deviceExtension->IoctlWorkItem);
//     }
//
//     return;
// }
//
//
// VOID
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// IoctlWorkItemRoutine(
//      WDFWORKITEM  WorkItem
//     )
// /*++
//
// Routine Description:
//
//     Work item routine for processing ioctl requests.
//     This is needed because event callbacks are called at DISPATCH_LEVEL and ioctl
//     requests are currently processed synchronously and not asynchronously.
//
// Arguments:
//
//     WorkItem - WDF work item
//
// Return Value:
//
//     none
//
// --*/
// {
//     PCDROM_DEVICE_EXTENSION             deviceExtension = NULL;
//
//     PAGED_CODE ();
//
//     deviceExtension = WdfObjectGetTypedContext(WdfWorkItemGetParentObject(WorkItem), CDROM_DEVICE_EXTENSION);
//
//     if (DeviceIsMmcUpdateRequired(deviceExtension->Device))
//     {
//         // Issue command to update the drive capabilities.
//         // The failure of MMC update is not considered critical,
//         // so that we'll continue to process I/O even MMC update fails.
//         DeviceUpdateMmcCapabilities(deviceExtension->Device);
//     }
//
//     RequestProcessSerializedIoctl(deviceExtension, deviceExtension->IoctlWorkItemContext.OriginalRequest);
// }
//
//
// _IRQL_requires_max_(PASSIVE_LEVEL)
// NTSTATUS
// RequestProcessSerializedIoctl(
//      PCDROM_DEVICE_EXTENSION  DeviceExtension,
//      WDFREQUEST               Request
//     )
// /*++
// Routine Description:
//
//     a dispatch routine for all functions to process IOCTLs.
//
// Arguments:
//
//     DeviceExtension - device context
//
//     Request - handle to the incoming WDF Request object
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS                status = STATUS_SUCCESS;
//     size_t                  information = 0;
//     WDF_REQUEST_PARAMETERS  requestParameters;
//     BOOLEAN                 completeRequest = TRUE;
//
//     PAGED_CODE ();
//
//     // Get the Request parameters
//     WDF_REQUEST_PARAMETERS_INIT(&requestParameters);
//     WdfRequestGetParameters(Request, &requestParameters);
//
//     NormalizeIoctl(&requestParameters);
//
//     // process IOCTLs
//     switch (requestParameters.Parameters.DeviceIoControl.IoControlCode)
//     {
//     case IOCTL_CDROM_READ_TOC:
//     case IOCTL_CDROM_GET_LAST_SESSION:
//         status = RequestHandleReadTOC(DeviceExtension, Request, requestParameters, &information);
//         break;
//
//     case IOCTL_CDROM_READ_TOC_EX:
//         status = RequestHandleReadTocEx(DeviceExtension, Request, requestParameters, &information);
//         break;
//
//     case IOCTL_CDROM_GET_CONFIGURATION:
//         status = RequestHandleGetConfiguration(DeviceExtension, Request, requestParameters, &information);
//         break;
//
//     case IOCTL_CDROM_RAW_READ:
//         status = DeviceHandleRawRead(DeviceExtension, Request, requestParameters, &information);
//         break;
//
//     case IOCTL_DISK_GET_LENGTH_INFO:
//     case IOCTL_DISK_GET_DRIVE_GEOMETRY:
//     case IOCTL_DISK_GET_DRIVE_GEOMETRY_EX:
//     case IOCTL_CDROM_GET_DRIVE_GEOMETRY:
//     case IOCTL_CDROM_GET_DRIVE_GEOMETRY_EX:
//     case IOCTL_STORAGE_READ_CAPACITY:
//         status = RequestHandleGetDriveGeometry(DeviceExtension, Request, requestParameters, &information);
//         break;
//
//     case IOCTL_DISK_VERIFY:
//         status = RequestHandleDiskVerify(DeviceExtension, Request, requestParameters, &information);
//         break;
//
//     case IOCTL_STORAGE_CHECK_VERIFY:
//         // IOCTL_STORAGE_CHECK_VERIFY2 was processed including send a Test Unit Read
//         // with srb flag SRB_CLASS_FLAGS_LOW_PRIORITY to port driver asynchronizelly.
//         // The original request was completed after TUR finishes.
//         // As CDROM.SYS serializes IOs need accessing device, it's not a big difference from above behavior to
//         // just process it in serialized manner. So I put it here and treat it as same as IOCTL_STORAGE_CHECK_VERIFY.
//     case IOCTL_STORAGE_CHECK_VERIFY2:
//         status = RequestHandleCheckVerify(DeviceExtension, Request, requestParameters, &information);
//         break;
//
//     case IOCTL_DISK_GET_DRIVE_LAYOUT:
//     case IOCTL_DISK_GET_DRIVE_LAYOUT_EX:
//     case IOCTL_DISK_GET_PARTITION_INFO:
//     case IOCTL_DISK_GET_PARTITION_INFO_EX:
//         status = RequestHandleFakePartitionInfo(DeviceExtension, Request, requestParameters, &information);
//         break;
//
//     case IOCTL_DISK_IS_WRITABLE:
//         //
//         // Even though this media is writable, the requester of this IOCTL really
//         // wants to know if thw media behaves like any other disk or not. This is
//         // so if FeatureDefectManagement and FeatureRandomWritable are current on
//         // the drive-represented by the FeatureDefectManagement validation schema
//         //
//         if (DeviceExtension->DeviceAdditionalData.Mmc.WriteAllowed &&
//             (DeviceExtension->DeviceAdditionalData.Mmc.ValidationSchema == FeatureDefectManagement))
//         {
//             status = STATUS_SUCCESS;
//         }
//         else
//         {
//             status = STATUS_MEDIA_WRITE_PROTECTED;
//         }
//         information = 0;
//         break;
//
//     case IOCTL_CDROM_PLAY_AUDIO_MSF:
//         status = DeviceHandlePlayAudioMsf(DeviceExtension, Request, requestParameters, &information);
//         break;
//
//     case IOCTL_CDROM_READ_Q_CHANNEL:
//         status = DeviceHandleReadQChannel(DeviceExtension, Request, requestParameters, &information);
//         break;
//
//     case IOCTL_CDROM_PAUSE_AUDIO:
//         status = DeviceHandlePauseAudio(DeviceExtension, Request, &information);
//         break;
//
//     case IOCTL_CDROM_RESUME_AUDIO:
//         status = DeviceHandleResumeAudio(DeviceExtension, Request, &information);
//         break;
//
//     case IOCTL_CDROM_SEEK_AUDIO_MSF:
//         status = DeviceHandleSeekAudioMsf(DeviceExtension, Request, requestParameters, &information);
//         break;
//
//     case IOCTL_CDROM_STOP_AUDIO:
//         status = DeviceHandleStopAudio(DeviceExtension, Request, &information);
//         break;
//
//     case IOCTL_CDROM_GET_VOLUME:
//     case IOCTL_CDROM_SET_VOLUME:
//         status = DeviceHandleGetSetVolume(DeviceExtension, Request, requestParameters, &information);
//         break;
//
//     case IOCTL_DVD_GET_REGION:
//         status = RequestHandleGetDvdRegion(DeviceExtension, Request, &information);
//         break;
//
//     case IOCTL_DVD_READ_STRUCTURE:
//         status = DeviceHandleReadDvdStructure(DeviceExtension, Request, requestParameters, &information);
//         break;
//
//     case IOCTL_DVD_END_SESSION:
//         status = DeviceHandleDvdEndSession(DeviceExtension, Request, requestParameters, &information);
//         break;
//
//     case IOCTL_DVD_START_SESSION:
//     case IOCTL_DVD_READ_KEY:
//         status = DeviceHandleDvdStartSessionReadKey(DeviceExtension, Request, requestParameters, &information);
//         break;
//
//     case IOCTL_DVD_SEND_KEY:
//     case IOCTL_DVD_SEND_KEY2:
//         status = DeviceHandleDvdSendKey(DeviceExtension, Request, requestParameters, &information);
//         break;
//
//     case IOCTL_STORAGE_SET_READ_AHEAD:
//         status = DeviceHandleSetReadAhead(DeviceExtension, Request, requestParameters, &information);
//         break;
//
//     case IOCTL_CDROM_SET_SPEED:
//         status = DeviceHandleSetSpeed(DeviceExtension, Request, requestParameters, &information);
//         break;
//
//     case IOCTL_AACS_READ_MEDIA_KEY_BLOCK_SIZE:
//     case IOCTL_AACS_READ_MEDIA_KEY_BLOCK:
//         status = DeviceHandleAacsReadMediaKeyBlock(DeviceExtension, Request, requestParameters, &information);
//         break;
//
//     case IOCTL_AACS_START_SESSION:
//         status = DeviceHandleAacsStartSession(DeviceExtension, Request, requestParameters, &information);
//         break;
//
//     case IOCTL_AACS_END_SESSION:
//         status = DeviceHandleAacsEndSession(DeviceExtension, Request, requestParameters, &information);
//         break;
//
//     case IOCTL_AACS_SEND_CERTIFICATE:
//         status = DeviceHandleAacsSendCertificate(DeviceExtension, Request, requestParameters, &information);
//         break;
//
//     case IOCTL_AACS_GET_CERTIFICATE:
//         status = DeviceHandleAacsGetCertificate(DeviceExtension, Request, requestParameters, &information);
//         break;
//
//     case IOCTL_AACS_GET_CHALLENGE_KEY:
//         status = DeviceHandleAacsGetChallengeKey(DeviceExtension, Request, requestParameters, &information);
//         break;
//
//     case IOCTL_AACS_SEND_CHALLENGE_KEY:
//         status = DeviceHandleSendChallengeKey(DeviceExtension, Request, requestParameters, &information);
//         break;
//
//     case IOCTL_AACS_READ_VOLUME_ID:
//         status = DeviceHandleReadVolumeId(DeviceExtension, Request, requestParameters, &information);
//         break;
//
//     case IOCTL_AACS_READ_SERIAL_NUMBER:
//         status = DeviceHandleAacsReadSerialNumber(DeviceExtension, Request, requestParameters, &information);
//         break;
//
//     case IOCTL_AACS_READ_MEDIA_ID:
//         status = DeviceHandleAacsReadMediaId(DeviceExtension, Request, requestParameters, &information);
//         break;
//
//     case IOCTL_AACS_READ_BINDING_NONCE:
//         status = DeviceHandleAacsReadBindingNonce(DeviceExtension, Request, requestParameters, &information);
//         break;
//
//     case IOCTL_AACS_GENERATE_BINDING_NONCE:
//         status = DeviceHandleAacsGenerateBindingNonce(DeviceExtension, Request, requestParameters, &information);
//         break;
//
//     case IOCTL_CDROM_ENABLE_STREAMING:
//         status = RequestHandleEnableStreaming(DeviceExtension, Request, &information);
//         break;
//
//     case IOCTL_CDROM_SEND_OPC_INFORMATION:
//         status = RequestHandleSendOpcInformation(DeviceExtension, Request, &information);
//         break;
//
//     case IOCTL_CDROM_GET_PERFORMANCE:
//         status = RequestHandleGetPerformance(DeviceExtension, Request, requestParameters, &information);
//         break;
//
//     // This IOCTL is a fake one, used for MCN process sync-ed with serial queue.
//     case IOCTL_MCN_SYNC_FAKE_IOCTL:
//         status = RequestHandleMcnSyncFakeIoctl(DeviceExtension, &information);
//         break;
//
//     case IOCTL_STORAGE_MEDIA_REMOVAL:
//     case IOCTL_STORAGE_EJECTION_CONTROL:
//     {
//         status = RequestHandleEjectionControl(DeviceExtension, Request, requestParameters, &information);
//
//         break;
//     }
//
//     case IOCTL_STORAGE_EJECT_MEDIA:
//     case IOCTL_STORAGE_LOAD_MEDIA:
//     case IOCTL_STORAGE_LOAD_MEDIA2:
//     {
//         status = RequestHandleLoadEjectMedia(DeviceExtension, Request, requestParameters, &information);
//
//         break;
//     }
//
//     case IOCTL_STORAGE_MCN_CONTROL:
//     {
//         status = RequestHandleMcnControl(DeviceExtension, Request, &information);
//
//         break;
//     }
//
//     case IOCTL_STORAGE_RESERVE:
//     case IOCTL_STORAGE_RELEASE:
//     {
//         status = RequestHandleReserveRelease(DeviceExtension, Request, requestParameters, &information);
//
//         break;
//     }
//
//     case IOCTL_STORAGE_PERSISTENT_RESERVE_IN:
//     case IOCTL_STORAGE_PERSISTENT_RESERVE_OUT:
//     {
//         status = RequestHandlePersistentReserve(DeviceExtension, Request, requestParameters, &information);
//
//         break;
//     }
//
// #if (NTDDI_VERSION >= NTDDI_WIN8)
//     case IOCTL_DISK_ARE_VOLUMES_READY:
//     {
//         status = RequestHandleAreVolumesReady(DeviceExtension, Request, requestParameters, &information);
//
//         completeRequest = FALSE;
//
//         break;
//     }
//
//     case IOCTL_VOLUME_ONLINE:
//     case IOCTL_VOLUME_POST_ONLINE:
//     {
//         status = RequestHandleVolumeOnline(DeviceExtension, Request, requestParameters, &information);
//
//         break;
//     }
// #endif
//
//     default:
//     {
//         status = STATUS_ACCESS_DENIED;
//         break;
//     }
//     } // end of switch(ioctl)
//
//     if (completeRequest)
//     {
//         RequestCompletion(DeviceExtension, Request, status, information);
//     }
//
//     return status;
// }
//
// VOID
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// SequentialQueueEvtCanceledOnQueue(
//      WDFQUEUE   Queue,
//      WDFREQUEST Request
//     )
// /*++
// Routine Description:
//
//     Perform cancellation when request is still in queue.
//
//     If request is sych-ed in another thread, signal the event to let that thread be able to complete the request.
//     Otherwise, complete the request.
//
// Arguments:
//
//     Queue - serial queue
//     Request - handle to the incoming WDF Request object
//
// Return Value:
//
//     None
//
// --*/
// {
//     PCDROM_REQUEST_CONTEXT  requestContext = RequestGetContext(Request);
//
//     if (requestContext->SyncRequired)
//     {
//         KeSetEvent(requestContext->SyncEvent, IO_CD_ROM_INCREMENT, FALSE);
//     }
//     else
//     {
//         PCDROM_DEVICE_EXTENSION deviceExtension = NULL;
//         WDFDEVICE               device = WdfIoQueueGetDevice(Queue);
//
//         deviceExtension = DeviceGetExtension(device);
//
//         RequestCompletion(deviceExtension, Request, STATUS_CANCELLED, 0);
//
//     }
//
//     return;
// }
//
//
// NTSTATUS
// RequestSynchronizeProcessWithSerialQueue(
//      WDFDEVICE Device,
//      WDFREQUEST Request
//     )
// /*++
// Routine Description:
//
//     This is the mechanism to sync a request process in original thread with serialize queue.
//     initialize a EVENT and put the request inot serialize queue;
//     waiting for serialize queue processes to this request and signal the EVENT;
//     call the request handler to process this request.
//
// Arguments:
//
//     DeviceExtension - device context
//
//     Request - handle to the incoming WDF Request object
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS                status = STATUS_SUCCESS;
//     PCDROM_DEVICE_EXTENSION deviceExtension = DeviceGetExtension(Device);
//     PCDROM_REQUEST_CONTEXT  requestContext = RequestGetContext(Request);
//     PKEVENT                 bufferToFree = requestContext->SyncEvent;
//
//     if (KeGetCurrentIrql() >= DISPATCH_LEVEL) {
//         // cannot block at or above DISPATCH_LEVEL
//         TracePrint!((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL,
//             "RequestSynchronousProcessWithSerialQueue called at DISPATCH_LEVEL or above"));
//         NT_ASSERT(FALSE);
//         RequestCompletion(deviceExtension, Request, STATUS_INVALID_LEVEL, 0);
//         return STATUS_INVALID_LEVEL;
//     }
//
//     // init the synchronization event
//     KeInitializeEvent(requestContext->SyncEvent, NotificationEvent, FALSE);
//
//     // do we still need to do something like this?
//     // SET_FLAG(nextStack->Flags, SL_OVERRIDE_VERIFY_VOLUME);
//
//     // NOTE: this mechanism relies on that KMDF will not complete request by itself.
//     // Doing that will cause the syncEvent not fired thus this thread will stuck.
//     // This should not really happen: our EvtCanceledOnQueue callbacks should be
//     // called even if queues are purged for some reason. The only case when these
//     // callbacks are not called is when a request is owned by the driver (i.e. has
//     // already been passed to one of the registered handlers). In this case, it is
//     // our responsibility to cancel such requests properly.
//     status = WdfDeviceEnqueueRequest(Device, Request);
//
//     if (!NT_SUCCESS(status))
//     {
//         // Failed to forward request! Pretend the sync event already occured, otherwise we'll hit
//         // an assert in RequestEvtCleanup.
//         KeSetEvent(requestContext->SyncEvent, IO_CD_ROM_INCREMENT, FALSE);
//         RequestCompletion(deviceExtension, Request, status, WdfRequestGetInformation(Request));
//     }
//     else
//     {
//         NTSTATUS                waitStatus = STATUS_UNSUCCESSFUL;
//         PCDROM_DATA             cdData = &(deviceExtension->DeviceAdditionalData);
//         BOOLEAN                 fCallSyncCallback = FALSE;
//         PIRP                    irp = WdfRequestWdmGetIrp(Request);
//
//         // ok, now wait on the event
//         while (waitStatus != STATUS_SUCCESS)
//         {
//             waitStatus = KeWaitForSingleObject(requestContext->SyncEvent, Executive, KernelMode, TRUE, NULL);
//             if (waitStatus == STATUS_SUCCESS) // must check equality -- STATUS_ALERTED is success code
//             {
//                 // do nothing
//             }
//             else if (waitStatus != STATUS_ALERTED)
//             {
//                 // do nothing
//                 TracePrint!((TRACE_LEVEL_FATAL, TRACE_FLAG_IOCTL,
//                             "Request %p on device object %p had a non-alert, non-success result from wait (%!HRESULT!)\n",
//                             Request, Device, waitStatus));
//                 NT_ASSERT(FALSE);
//             }
//             else if (PsIsThreadTerminating(PsGetCurrentThread()))
//             {
//                 // the thread was alerted and is terminating, so cancel the irp
//                 // this will cause EvtIoCanceledOnQueue to be called, which will signal the event,
//                 // so we will get out of the while loop and eventually complete the request.
//                 if (IoCancelIrp(irp))
//                 {
//                     // cancellation routine was called
//                     TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_IOCTL,
//                                 "Sychronize Ioctl: request %p cancelled from device %p\n",
//                                 Request, Device));
//                 }
//                 else
//                 {
//                     TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_IOCTL,
//                                 "Sychronize Ioctl: request %p could not be cancelled from device %p\n",
//                                 Request, Device));
//                 }
//             }
//             else
//             {
//                 TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_IOCTL,
//                             "SPURIOUS ALERT waiting for Request %p on device %p (%!STATUS!)\n",
//                             Request, Device, status));
//             }
//         } // end of wait loop on the event
//
//         // because we've waited an unknown amount of time, should check
//         // the cancelled flag to immediately fail the irp as appropriate
//         if (WdfRequestIsCanceled(Request))
//         {
//             // the request was cancelled, thus we should always stop
//             // processing here if possible.
//             status = STATUS_CANCELLED;
//             RequestCompletion(deviceExtension, Request, status, 0);
//         }
//         else if (EXCLUSIVE_MODE(cdData) && !EXCLUSIVE_OWNER(cdData, WdfRequestGetFileObject(Request)))
//         {
//             WDF_REQUEST_PARAMETERS  requestParameters;
//             BOOLEAN                 isBlocked = FALSE;
//
//             // get the request parameters
//             WDF_REQUEST_PARAMETERS_INIT(&requestParameters);
//             WdfRequestGetParameters(Request, &requestParameters);
//
//             status = RequestIsIoctlBlockedByExclusiveAccess(Request, &isBlocked);
//             if (isBlocked)
//             {
//                 TracePrint!((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL,
//                             "Access Denied! Device in exclusive mode.Failing Ioctl %lx\n",
//                             requestParameters.Parameters.DeviceIoControl.IoControlCode));
//                 RequestCompletion(deviceExtension, Request, STATUS_ACCESS_DENIED, 0);
//             }
//             else
//             {
//                 TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL,
//                             "Ioctl %lx not blocked by cdrom being in exclusive mode\n",
//                             requestParameters.Parameters.DeviceIoControl.IoControlCode));
//                 fCallSyncCallback = TRUE;
//             }
//         }
//         else
//         {
//             fCallSyncCallback = TRUE;
//         }
//
//         if (fCallSyncCallback)
//         {
//             // Synchronization completed successfully.  Call the requested routine
//             status = requestContext->SyncCallback(Device, Request);
//         }
//     }
//
//     // The next SequentialQueue evt routine will not be triggered until the current request is completed.
//
//     // clean up the request context setting.
//     FREE_POOL(bufferToFree);
//
//     return status;
// }
//
// NTSTATUS
// RequestIsIoctlBlockedByExclusiveAccess(
//       WDFREQUEST  Request,
//      PBOOLEAN    IsBlocked
//     )
// /*++
// Routine Description:
//
//     Check if the IOCTL request should be blocked or not according to
//     the exclusive lock stat.
//
// Arguments:
//
//     Request - handle to the incoming WDF Request object
//
// Return Value:
//
//     NTSTATUS
//
//     IsBlocked - TRUE (be blocked); FALSE (not blocked)
//
// --*/
// {
//     NTSTATUS                status = STATUS_SUCCESS;
//     ULONG                   ioctlCode = 0;
//     ULONG                   baseCode = 0;
//     WDF_REQUEST_PARAMETERS  requestParameters;
//
//     // Get the Request parameters
//     WDF_REQUEST_PARAMETERS_INIT(&requestParameters);
//     WdfRequestGetParameters(Request, &requestParameters);
//
//     // check and initialize parameter
//     if (IsBlocked == NULL)
//     {
//         //This is an internal function and this parameter must be supplied.
//         NT_ASSERT(FALSE);
//
//         return STATUS_INVALID_PARAMETER;
//     }
//     else
//     {
//         *IsBlocked = FALSE;
//     }
//
//     // check if this is an IOCTL
//     if ((requestParameters.Type == WdfRequestTypeDeviceControl) ||
//         (requestParameters.Type == WdfRequestTypeDeviceControlInternal))
//     {
//         //
//         // Allow minimum set of commands that are required for the disk manager
//         // to show the CD device, while in exclusive mode.
//         // Note: These commands should not generate any requests to the device,
//         //       and thus must be handled directly in StartIO during exclusive
//         //       access (except for the exclusive owner, of course).
//         //
//         ioctlCode = requestParameters.Parameters.DeviceIoControl.IoControlCode;
//         baseCode = DEVICE_TYPE_FROM_CTL_CODE(ioctlCode);
//
//         if (ioctlCode == IOCTL_SCSI_GET_ADDRESS           ||
//             ioctlCode == IOCTL_STORAGE_GET_HOTPLUG_INFO   ||
//             ioctlCode == IOCTL_STORAGE_GET_DEVICE_NUMBER  ||
//             ioctlCode == IOCTL_STORAGE_GET_MEDIA_TYPES_EX ||
//             ioctlCode == IOCTL_CDROM_EXCLUSIVE_ACCESS     ||
//             ioctlCode == IOCTL_CDROM_GET_INQUIRY_DATA
//             )
//         {
//             *IsBlocked = FALSE;
//         }
//
//         //
//         // Handle IOCTL_STORAGE_QUERY_PROPERTY special because:
//         //  (1) PropertyExistsQuery should not generate device i/o
//         //  (2) Queries for StorageDeviceProperty and StorageAdapterDescriptor
//         //      will return cache'd data
//     else if (ioctlCode == IOCTL_STORAGE_QUERY_PROPERTY)
//         {
//             PSTORAGE_PROPERTY_QUERY query = NULL;
//             status = WdfRequestRetrieveInputBuffer(Request,
//                                                    requestParameters.Parameters.DeviceIoControl.InputBufferLength,
//                                                    (PVOID*)&query,
//                                                    NULL);
//
//             if (NT_SUCCESS(status))
//             {
//                 if (query != NULL)
//                 {
//                     if (query->QueryType == PropertyExistsQuery)
//                     {
//                         *IsBlocked = FALSE;
//                     }
//                     else if ((query->QueryType == PropertyStandardQuery) &&
//                              ((query->PropertyId == StorageDeviceProperty) ||
//                               (query->PropertyId == StorageAdapterProperty)))
//                     {
//                         *IsBlocked = FALSE;
//                     }
//                 }
//             }
//         }
//
//         // Return TRUE for unknown IOCTLs with STORAGE bases
//     else if (baseCode == IOCTL_SCSI_BASE    ||
//             baseCode == IOCTL_DISK_BASE    ||
//             baseCode == IOCTL_CDROM_BASE   ||
//             baseCode == IOCTL_STORAGE_BASE ||
//             baseCode == IOCTL_DVD_BASE     )
//         {
//             *IsBlocked = TRUE;
//         }
//     }
//     else
//     {
//         // this should only be called with an IOCTL
//         NT_ASSERT(FALSE);
//
//         status = STATUS_INVALID_PARAMETER;
//     }
//
//     return status;
// }
//
// BOOLEAN
// DeviceIsMmcUpdateRequired(
//      WDFDEVICE    Device
//     )
// /*++
// Routine Description:
//
//     Check if the device needs to update its MMC information.
//
// Arguments:
//
//     Device - device to be checked.
//
// Return Value:
//
//     TRUE (require update); FALSE (not require update)
//
// --*/
// {
//     PCDROM_DEVICE_EXTENSION deviceExtension = DeviceGetExtension(Device);
//     PCDROM_DATA             cdData = &(deviceExtension->DeviceAdditionalData);
//
//     if ((cdData->Mmc.IsMmc) &&
//         (cdData->Mmc.UpdateState == CdromMmcUpdateRequired as i32))
//     {
//         return TRUE;
//     }
//     else
//     {
//         // no update required: just proceed
//         return FALSE;
//     }
// }
//
// VOID
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// RequestEvtCleanup(
//      WDFOBJECT Request
//     )
// /*++
// Routine Description:
//
//     Request cleanup callback.
//
// Arguments:
//
//     Request - request to clean up.
//
// Return Value:
//
//     None
//
// --*/
// {
//     WDFREQUEST              request = (WDFREQUEST)Request;
//     PCDROM_REQUEST_CONTEXT  requestContext = RequestGetContext(request);
//
//     if (requestContext->SyncRequired)
//     {
//         // the event should have been signaled, just check that
//         NT_ASSERT(KeReadStateEvent(requestContext->SyncEvent) != 0);
//     }
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
