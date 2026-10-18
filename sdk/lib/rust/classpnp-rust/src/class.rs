#[allow(unused_imports)] use crate::{NT_ASSERTMSG};
#[allow(unused_imports)] use crate::{DBGLOGFLUSHINFO};
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{DEVICE_DATA_SET_RANGE, DEVICE_DSM_OFFLOAD_READ_PARAMETERS, DEVICE_DSM_OFFLOAD_WRITE_PARAMETERS, DEVICE_MANAGE_DATA_SET_ATTRIBUTES, DEVICE_RELATIONS, DISK_GEOMETRY, ExFreePool, ExInitializeRundownProtectionCacheAware, ExReInitializeRundownProtectionCacheAware, ExSizeOfRundownProtectionCacheAware, ExWaitForRundownProtectionReleaseCacheAware, _KWAIT_REASON_Executive, GUID, IO_ERROR_LOG_PACKET, _IO_STATUS_BLOCK, IRP, IoAllocateDriverObjectExtension, IoAllocateErrorLogEntry, IoAllocateIrp, IoAllocateMdl, IoAllocateWorkItem, IoBuildDeviceIoControlRequest, IoCopyCurrentIrpStackLocationToNext, IoCreateDevice, IoDeleteDevice, IoDetachDevice, IoFreeIrp, IoFreeWorkItem, IoGetAttachedDeviceReference, IoGetDeviceProperty, IoGetDriverObjectExtension, IoGetNextIrpStackLocation, IoGetPagingIoPriority, IoInitializeIrp, IoInvalidateDeviceRelations, IoOpenDeviceRegistryKey, IoQueueWorkItem, IoRegisterDeviceInterface, IoSetCompletionRoutine, IoSetDeviceInterfaceState, IoSetNextIrpStackLocation, IoSetStartIoAttributes, IoStartPacket, IoWMIRegistrationControl, IoWriteErrorLogEntry, KEVENT, KLOCK_QUEUE_HANDLE, KeAcquireInStackQueuedSpinLock, KeBugCheck, KeDelayExecutionThread, KeEnterCriticalRegion, KeGetRecommendedSharedDataAlignment, KeInitializeDpc, KeInitializeEvent, KeInitializeSpinLock, KeInitializeTimer, KeLeaveCriticalRegion, KeQueryTimeIncrement, KeReleaseInStackQueuedSpinLock, KeSetTimerEx, KeWaitForSingleObject, LARGE_INTEGER, LIST_ENTRY, LPGUID, MOUNTDEV_NAME, MmBuildMdlForNonPagedPool, MmProbeAndLockPages, MmUnlockPages, NTSTRSAFE_PSTR, _EVENT_TYPE_NotificationEvent, _OBJECT_ATTRIBUTES, ObReferenceObjectByPointer, PCCHAR, PCSZ, PDEVICE_CAPABILITIES, PDEVICE_DATA_SET_RANGE, PDEVICE_DSM_OFFLOAD_READ_PARAMETERS, PDEVICE_DSM_OFFLOAD_WRITE_PARAMETERS, PDEVICE_MANAGE_DATA_SET_ATTRIBUTES, PDEVICE_OBJECT, PDEVICE_POWER_DESCRIPTOR, PDEVICE_RELATIONS, PDRIVER_OBJECT, PEX_RUNDOWN_REF_CACHE_AWARE, PIO_COMPLETION_ROUTINE, PIO_ERROR_LOG_PACKET, PIO_STACK_LOCATION, PIO_STATUS_BLOCK, PIO_WORKITEM, PIRP, PKEVENT, PMDL, PMOUNTDEV_NAME, PPREVENT_MEDIA_REMOVAL, PREVENT_MEDIA_REMOVAL, PRTL_QUERY_REGISTRY_TABLE, PSTORAGE_ADAPTER_DESCRIPTOR, PSTORAGE_DESCRIPTOR_HEADER, PSTORAGE_DEVICE_NUMBER, PSTORAGE_HOTPLUG_INFO, PSTORAGE_OFFLOAD_READ_OUTPUT, PSTORAGE_OFFLOAD_WRITE_OUTPUT, PSTORAGE_PROPERTY_ID, PSTORAGE_PROPERTY_QUERY, PSTORAGE_READ_CAPACITY, PULONGLONG, PUNICODE_STRING, PVPB, PsGetCurrentThread, RTL_QUERY_REGISTRY_TABLE, RtlAnsiStringToUnicodeString, RtlCopyUnicodeString, RtlDeleteRegistryValue, RtlFreeUnicodeString, RtlInitString, RtlInitUnicodeString, RtlQueryRegistryValues, STORAGE_DEVICE_NUMBER, STORAGE_HOTPLUG_INFO, STORAGE_OFFLOAD_READ_OUTPUT, STORAGE_OFFLOAD_WRITE_OUTPUT, STORAGE_PROPERTY_QUERY, STORAGE_READ_CAPACITY, STRING, UINT32, _KWAIT_REASON_UserRequest, ZwClose, ZwOpenKey, ZwSetValueKey};
// Local aliases for enum variants not exported by ntkrnl_sys::ddk under short names
use ntkrnl_sys::ddk::{
    _DEVICE_RELATION_TYPE_BusRelations as BusRelations,
    _DEVICE_RELATION_TYPE_TargetDeviceRelation as TargetDeviceRelation,
    _WORK_QUEUE_TYPE_DelayedWorkQueue as DelayedWorkQueue,
    _DEVICE_POWER_STATE_PowerDeviceD0 as PowerDeviceD0,
    _DEVICE_POWER_STATE_PowerDeviceD3 as PowerDeviceD3,
    _MEDIA_TYPE_RemovableMedia as RemovableMedia,
    _MEDIA_TYPE_FixedMedia as FixedMedia,
    _DEVICE_REMOVAL_POLICY_RemovalPolicyExpectOrderlyRemoval as RemovalPolicyExpectOrderlyRemoval,
    _DEVICE_REMOVAL_POLICY_RemovalPolicyExpectSurpriseRemoval as RemovalPolicyExpectSurpriseRemoval,
    _DEVICE_USAGE_NOTIFICATION_TYPE_DeviceUsageTypePaging as DeviceUsageTypePaging,
    _DEVICE_USAGE_NOTIFICATION_TYPE_DeviceUsageTypeHibernation as DeviceUsageTypeHibernation,
    _DEVICE_USAGE_NOTIFICATION_TYPE_DeviceUsageTypeDumpFile as DeviceUsageTypeDumpFile,
    _DEVICE_USAGE_NOTIFICATION_TYPE_DeviceUsageTypeBoot as DeviceUsageTypeBoot,
    _IO_PAGING_PRIORITY_IoPagingPriorityHigh as IoPagingPriorityHigh,
    _IO_PAGING_PRIORITY_IoPagingPriorityInvalid as IoPagingPriorityInvalid,
    _LOCK_OPERATION_IoReadAccess as IoReadAccess,
    _LOCK_OPERATION_IoWriteAccess as IoWriteAccess,
    _STORAGE_QUERY_TYPE_PropertyStandardQuery as PropertyStandardQuery,
    _STORAGE_PROPERTY_ID_StorageAdapterProperty as StorageAdapterProperty,
    _STORAGE_PROPERTY_ID_StorageDeviceProperty as StorageDeviceProperty,
    _STORAGE_PROPERTY_ID_StorageDeviceUniqueIdProperty as StorageDeviceUniqueIdProperty,
    _STORAGE_PROPERTY_ID_StorageDeviceWriteCacheProperty as StorageDeviceWriteCacheProperty,
    _STORAGE_PROPERTY_ID_StorageMiniportProperty as StorageMiniportProperty,
    _STORAGE_PROPERTY_ID_StorageAccessAlignmentProperty as StorageAccessAlignmentProperty,
    _STORAGE_PROPERTY_ID_StorageDeviceSeekPenaltyProperty as StorageDeviceSeekPenaltyProperty,
    _STORAGE_PROPERTY_ID_StorageDeviceTrimProperty as StorageDeviceTrimProperty,
    _STORAGE_PROPERTY_ID_StorageDeviceLBProvisioningProperty as StorageDeviceLBProvisioningProperty,
    _STORAGE_PROPERTY_ID_StorageDevicePowerProperty as StorageDevicePowerProperty,
    _STORAGE_PROPERTY_ID_StorageDeviceCopyOffloadProperty as StorageDeviceCopyOffloadProperty,
    _STORAGE_PROPERTY_ID_StorageDeviceMediumProductType as StorageDeviceMediumProductType,
    _STORAGE_PORT_CODE_SET_StoragePortCodeSetStorport as StoragePortCodeSetStorport,
    _STORAGE_PORT_CODE_SET_StoragePortCodeSetSDport as StoragePortCodeSetSDport,
    _STORAGE_PORT_CODE_SET_StoragePortCodeSetUSBport as StoragePortCodeSetUSBport,
    _STORAGE_BUS_TYPE_BusType1394 as BusType1394,
    _STORAGE_BUS_TYPE_BusTypeFibre as BusTypeFibre,
    _STORAGE_BUS_TYPE_BusTypeUsb as BusTypeUsb,
    _STORAGE_BUS_TYPE_BusTypeRAID as BusTypeRAID,
    _STORAGE_BUS_TYPE_BusTypeiScsi as BusTypeiScsi,
    _STORAGE_BUS_TYPE_BusTypeFileBackedVirtual as BusTypeFileBackedVirtual,
    DEVICE_REGISTRY_PROPERTY_DevicePropertyRemovalPolicy as DevicePropertyRemovalPolicy,
    _DPFLTR_TYPE_DPFLTR_CLASSPNP_ID as DPFLTR_CLASSPNP_ID,
    _EVENT_TYPE_SynchronizationEvent,
};
// IO_COMPLETION_ROUTINE is a type alias for PIO_COMPLETION_ROUTINE in ntkrnl_sys::ddk
use ntkrnl_sys::ddk::PIO_COMPLETION_ROUTINE as IO_COMPLETION_ROUTINE;
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{DO_VERIFY_VOLUME, FILE_DEVICE_DISK, FILE_PORTABLE_DEVICE, IRP_MJ_SCSI, SL_OVERRIDE_VERIFY_VOLUME, SL_WRITE_THROUGH, STATUS_DEVICE_BUSY, STATUS_DEVICE_DOES_NOT_EXIST, STATUS_VERIFY_REQUIRED, TRACE_LEVEL_ERROR, TRACE_LEVEL_INFORMATION, TRACE_LEVEL_WARNING};
use crate::types::{CLASSP_REG_ACCESS_ALIGNMENT_NOT_SUPPORTED, CLASSP_REG_DISBALE_IDLE_POWER_NAME, CLASSP_REG_PERF_RESTORE_VALUE_NAME, CLASSP_REG_QERR_OVERRIDE_MODE, CLASSP_REG_REMOVAL_POLICY_VALUE_NAME, CLASSP_REG_SUBKEY_NAME, CLASSP_VOLUME_VERIFY_CHECKED, CLASS_DRIVER_EXTENSION, CLASS_INIT_DATA, CLASS_INTERPRET_SENSE_INFO2, CLASS_INTERPRET_SENSE_INFO2_MAXIMUM_HISTORY_COUNT, CLASS_MAX_INTERLEAVE_PER_CRITICAL_IO, CLASS_PERF_RESTORE_MINIMUM, CLASS_QUERY_WMI_REGINFO_EX_LIST, CLASS_SRB_SCSI_REQUEST_BLOCK, CLASS_SRB_STORAGE_REQUEST_BLOCK, CLASS_WORKING_SET, CLASS_WORKING_SET_MAXIMUM, COMMON_DEVICE_EXTENSION, COMPLETION_CONTEXT, ClassGlobalDispatch, ClassMinimalPowerHandler, ClasspPowerSettingCallback, ClasspRetryRequestDpc, ClasspScanForClassHacks, ClasspStartIo, ClasspUpdateDiskProperties, DEV_USE_16BYTE_CDB, FDO_HACK_NO_SYNC_CACHE, FUNCTIONAL_DEVICE_EXTENSION, MAX_TOKEN_LIST_IDENTIFIERS, NUM_IO_RETRIES, PCOMMON_DEVICE_EXTENSION, PFUNCTIONAL_DEVICE_EXTENSION, PREAD_CAPACITY_DATA_EX, PSCSI_REQUEST_BLOCK, PTRANSFER_PACKET, QERR_SET_ZERO_ALWAYS, QERR_SET_ZERO_ODX_OR_TP_ONLY, READ_CAPACITY_DATA, READ_CAPACITY_DATA_EX, REG_DISK_CLASS_CONTROL, REMOVE_PENDING, SCSIOP_START_STOP_UNIT, SCSI_REQUEST_BLOCK, SRB_FLAGS_DISABLE_AUTOSENSE, SRB_FLAGS_DISABLE_SYNCH_TRANSFER, SRB_FLAGS_FREE_SENSE_BUFFER, SRB_FLAGS_NO_DATA_TRANSFER, SRB_FUNCTION_EXECUTE_SCSI, SRB_FUNCTION_STORAGE_REQUEST_BLOCK, SRB_STATUS_QUEUE_FROZEN, SRB_TYPE_STORAGE_REQUEST_BLOCK, START_UNIT_TIMEOUT, STORAGE_ADDRESS_TYPE_BTL8, STORAGE_REQUEST_BLOCK, TRANSFER_PACKET, _CLASS_DEV_INFO, _CLASS_DRIVER_EXTENSION, _CLASS_FUNCTION_SUPPORT_INFO, _CLASS_INIT_DATA, _CLASS_PRIVATE_FDO_DATA, _COMMON_DEVICE_EXTENSION, _COMPLETION_CONTEXT, _FUNCTIONAL_DEVICE_EXTENSION, _SRBEXDATATYPE_SrbExDataTypeScsiCdb16 as SrbExDataTypeScsiCdb16, _STORAGE_REQUEST_BLOCK};
use ntkrnl_sys::ddk::{DRIVER_OBJECT, STORAGE_PROPERTY_ID, _DEVICE_OBJECT, _DEVICE_POWER_DESCRIPTOR, _DRIVER_OBJECT, _IO_PAGING_PRIORITY, _IO_PAGING_PRIORITY_IoPagingPriorityHigh, _IO_PAGING_PRIORITY_IoPagingPriorityInvalid, _IRP, _UNICODE_STRING};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{DPFLTR_INFO_LEVEL, FILE_DEVICE_CD_ROM, FILE_FLOPPY_DISKETTE, IO_WARNING_DISK_SURPRISE_REMOVED, IRP_PAGING_IO, SL_KEY_SPECIFIED, STATUS_DEVICE_NOT_READY, STATUS_FT_READ_FROM_COPY, STATUS_INTERNAL_ERROR, STATUS_MORE_PROCESSING_REQUIRED, WMIREG_ACTION_REGISTER};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{TRACE_FLAG_INIT, TRACE_FLAG_PNP, TRACE_FLAG_RW};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{NonPagedPoolNx};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{CLASS_TAG_PRIVATE_DATA, CLASS_TAG_SENSE2, CLASS_TAG_WORKING_SET, FailurePredictionNone, IOCTL_STORAGE_PERSISTENT_RESERVE_IN, IOCTL_STORAGE_PERSISTENT_RESERVE_OUT, SCSI_DISK_DRIVER_INTERNAL};
use ntkrnl_sys::ddk::{_LARGE_INTEGER__bindgen_ty_1 as LARGE_INTEGER__bindgen_ty_1};
use crate::shims::*;
// resolve_imports.py: explicit kernel/private-type imports
use crate::debug::{ClasspInitializeDebugGlobals, SnapDiskStartup};
use crate::types::{BuildDeviceInputMdl, ClassDetermineTokenOperationCommandSupport, ClassForwardIrpSynchronous, ClassGetDescriptor, ClassGetDeviceParameter, ClassGetPdoId, ClassInitializeDispatchTables, ClassPnpQueryFdoRelations, ClassQueryPnpCapabilities, ClassRemoveDevice, ClassScanForSpecial, ClassSetDeviceParameter, ClasspAllocatePowerProcessIrp, ClasspAllocateReleaseQueueIrp, ClasspCalculateLogicalSectorSize, ClasspDeleteTimer, ClasspDisableTimer, ClasspEjectionControl, ClasspEnableIdlePower, ClasspEnableTimer, ClasspEnqueueIdleRequest, ClasspGetCopyOffloadMaxDuration, ClasspGetInquiryVpdSupportInfo, ClasspGetLBProvisioningInfo, ClasspInitializeHotplugInfo, ClasspInitializeIdleTimer, ClasspInitializeTimer, ClasspIsPortable, ClasspRegisterMountedDeviceInterface, ClasspScanForSpecialInRegistry, ClasspValidateOffloadSupported, ClasspZeroQERR, DequeueFreeTransferPacket, EnqueueDeferredClientIrp, EnqueueFreeTransferPacket, FreeDeviceInputMdl, InitLowMemRetry, InitializeStorageRequestBlock, InitializeTransferPackets, SetupDriveCapacityTransferPacket, SetupReadWriteTransferPacket, StepLowMemRetry, SubmitTransferPacket, TransferPacketQueueRetryDpc};
#[cfg(target_arch = "x86")]
use ntkrnl_sys::ddk::{KeQuerySystemTime};
#[cfg(not(target_arch = "x86"))]
use ntkrnl_sys::inline::ke::KeQuerySystemTime;
// resolve_imports.py: explicit kernel/private-type imports
use crate::guid::{GUID_CONSOLE_DISPLAY_STATE};
use crate::types::{AllFdosList, ClassBadItems, ClassGuidQueryRegInfoEx, ClassGuidSenseInfo2, ClassGuidSrbSupport, ClassGuidWorkingSet, MEDIA_LOCK_TYPE_InternalMediaLock};
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{_SCSI_REQUEST_BLOCK, _CDB__START_STOP};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{min};

extern "C" {
    fn IoGetCurrentIrpStackLocation(Irp: PIRP) -> PIO_STACK_LOCATION;
    fn InterpretSenseInfoWithoutHistory( Fdo: PDEVICE_OBJECT, Irp: PIRP, Srb: PSCSI_REQUEST_BLOCK, MajorFunctionCode: UCHAR, IoDeviceCode: ULONG, RetryCount: ULONG, Status: *mut NTSTATUS, RetryInterval: *mut LONGLONG, ) -> BOOLEAN;
    fn TracePrint(Args: ...);
    
    fn PORT_ALLOCATED_SENSE( FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION, Srb: PSCSI_REQUEST_BLOCK, ) -> BOOLEAN;
    fn FREE_PORT_ALLOCATED_SENSE_BUFFER( FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION, Srb: PSCSI_REQUEST_BLOCK, );
    fn RetryRequest( Fdo: PDEVICE_OBJECT, Irp: PIRP, Srb: PSCSI_REQUEST_BLOCK, Flush: BOOLEAN, RetryInterval: LONGLONG, );
    fn SrbClearSrbFlags(Srb: PSCSI_REQUEST_BLOCK, Flags: ULONG);
    fn ClassFreeOrReuseSrb( FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION, Srb: PSCSI_REQUEST_BLOCK, );
    fn FREE_POOL(Ptr: PVOID);
    fn ClasspPerfIncrementSuccessfulIo( FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION, );
    fn IoIsErrorUserInduced(Status: NTSTATUS) -> BOOLEAN;
    fn ClasspLogSystemEventWithDeviceNumber(Fdo: PDEVICE_OBJECT, EventId: ULONG) -> u32;
    fn IoMarkIrpPending(Irp: PIRP);
    
    fn ClassAcquireRemoveLock(DeviceObject: *mut _DEVICE_OBJECT, Irp: *mut _IRP) -> ULONG;
}

// CLASS_DRIVER_EXTENSION_KEY is #define CLASS_DRIVER_EXTENSION_KEY ((PVOID)ClassInitialize)
// MUST be exactly C's key (the ClassInitialize fn address): several translated call
// sites spell the lookup as `ClassInitialize as PVOID` directly, so a private static
// address here split-brains the registration from those lookups (NULL extension ->
// AV in ClassAddDevice on the first USB disk). One key, both spellings.
#[inline(always)]
fn CLASS_DRIVER_EXTENSION_KEY() -> *mut core::ffi::c_void {
    ClassInitialize as *mut core::ffi::c_void
}

extern "C" {
    fn DbgPrintEx(ComponentId: ULONG, Level: ULONG, Format: *const i8, ...);
    fn IoAdjustPagingPathCount(Count: *mut i32, Increment: BOOLEAN);
    fn SET_FLAG(Flags: *mut ULONG, Flag: ULONG);
    fn LongToPtr(l: LONG) -> PVOID;
    fn RtlEqualMemory(Source1: *const c_void, Source2: *const c_void, Length: usize) -> BOOLEAN;
    fn MmGetMdlVirtualAddress(Mdl: PMDL) -> PVOID;
    fn SimpleInitSlistHdr(Hdr: *mut SINGLE_LIST_ENTRY);
    fn SimplePushSlist(Hdr: *mut SINGLE_LIST_ENTRY, Entry: *mut SINGLE_LIST_ENTRY);
    fn SimplePopSlist(Hdr: *mut SINGLE_LIST_ENTRY) -> *mut SINGLE_LIST_ENTRY;
    fn SimpleIsSlistEmpty(Hdr: *mut SINGLE_LIST_ENTRY) -> BOOLEAN;
    fn IsKeyReadCopyNumber(Key: ULONG) -> BOOLEAN;
    fn ClasspIsObsoletePortDriver(FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION) -> BOOLEAN;
    fn ClasspIsThinProvisioned(FunctionSupportInfo: *mut _CLASS_FUNCTION_SUPPORT_INFO) -> BOOLEAN;
    fn ClasspIsIdleRequestSupported(FdoData: *mut _CLASS_PRIVATE_FDO_DATA, Irp: PIRP) -> BOOLEAN;
    fn ClasspMarkIrpAsIdle(Irp: PIRP, IsIdle: BOOLEAN);
    fn ClasspGetCurrentTime() -> LARGE_INTEGER;
}
unsafe extern "system" {
    fn PoUnregisterPowerSettingCallback(Handle: PVOID) -> u32;
    fn EtwRegister(ProviderId: *const GUID, EnableCallback: *mut c_void, CallbackContext: *mut c_void, RegHandle: *mut u64) -> NTSTATUS;
    fn EtwUnregister(RegHandle: u64) -> NTSTATUS;
    fn IsEqualGUID(Guid1: *const GUID, Guid2: *const GUID) -> BOOLEAN;
    fn PoRegisterPowerSettingCallback(DeviceObject: PDEVICE_OBJECT, SettingGuid: *const GUID, Callback: unsafe extern "system" fn(*const GUID, PVOID, ULONG, PVOID) -> NTSTATUS, Context: PVOID, Handle: *mut PVOID) -> NTSTATUS;
}

// ---- file-scope preamble (translated) ----
pub const CLASS_INIT_GUID: u32 = 1; // nolint: const-contradiction (class.c:24 defines it to 1 so classp.h EMITS the GUID storage in exactly this TU)
pub const DEBUG_MAIN_SOURCE: u32 = 1;
// ClassCheckVerifyComplete: real definition below (was a bogus `static mut ... = None`
// stub that linked only while the CHECK_VERIFY ioctl arm was dead under the bind-all).
pub static mut ClassPnpAllowUnload: ULONG = 1u32;
// ClassMaxInterleavePerCriticalIo / IdlePowerFDOList / IdlePowerFDOListMutex /
// PowerSettingNotificationHandle / ScreenStateNotificationHandle / DiskIdleTimeoutInMS:
// the single no_mangle definitions live in globals.rs; local twins here caused a
// split-brain (ClassInitialize initialized the local list, power.rs walked the shared
// zeroed one). Imported via crate::types externs below.
use crate::types::{DiskIdleTimeoutInMS, IdlePowerFDOList, IdlePowerFDOListMutex,
    PowerSettingNotificationHandle, ScreenStateNotificationHandle,
    ClassMaxInterleavePerCriticalIo};
pub const Magic10000: LARGE_INTEGER = LARGE_INTEGER {
    __bindgen_anon_1: LARGE_INTEGER__bindgen_ty_1 {
        LowPart: 0xe219652c,
        HighPart: 0xd1b71758u32 as i32,
    }
};
// WDI_STORAGE_PREDICT_FAILURE_DPS_GUID — defined in C header, not in ddk; zero-init until substrate provides it.
// GUID: {C55095C8-0FBA-4035-9A5E-A5A75C0C4C0D}
pub static mut StoragePredictFailureDPSGuid: GUID = GUID {
    Data1: 0xC55095C8u32,
    Data2: 0x0FBAu16,
    Data3: 0x4035u16,
    Data4: [0x9A, 0x5E, 0xA5, 0xA7, 0x5C, 0x0C, 0x4C, 0x0D],
};
pub const FirstDriveLetter: u8 = b'C';
pub const LastDriveLetter: u8 = b'Z';
pub static mut UseQPCTime: BOOLEAN = 0u8;
pub static mut InitSecurityCookie: BOOLEAN = 0u8;
#[no_mangle]
pub static mut MaxTokenOperationListIdentifier: ULONG = MAX_TOKEN_LIST_IDENTIFIERS;
pub static mut TokenOperationListIdentifier: ULONG = 0xFFFFFFFFu32;
// (IdlePowerFDOList family: see globals.rs — imported above, do not redefine here.)
/// class.c ClassCheckVerifyComplete — completion routine for the internal
/// CHECK_VERIFY ioctl the FDO forwards to the port driver: copy the media change
/// count + status into the ORIGINAL (master) irp stashed in Others.Argument1,
/// complete it, free the forwarded irp. (Was a bogus `static mut = None` stub;
/// linked only while the ClassDeviceControl CHECK_VERIFY arm was bind-all-dead.)
#[no_mangle]
pub unsafe extern "system" fn ClassCheckVerifyComplete(
    Fdo: *mut _DEVICE_OBJECT,
    Irp: *mut _IRP,
    _Context: *mut c_void,
) -> NTSTATUS {
    let irp_stack = IoGetCurrentIrpStackLocation(Irp);
    let fdo_extension = (*Fdo).DeviceExtension as *mut _FUNCTIONAL_DEVICE_EXTENSION;

    let original_irp = (*irp_stack).Parameters.Others.Argument1 as *mut _IRP;

    // Copy the media change count and status
    *((*original_irp).AssociatedIrp.SystemBuffer as *mut ULONG) =
        (*fdo_extension).MediaChangeCount;

    (*original_irp).IoStatus.__bindgen_anon_1.Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
    (*original_irp).IoStatus.Information = core::mem::size_of::<ULONG>() as _;

    ClassReleaseRemoveLock(Fdo, original_irp);
    ClassCompleteRequest(Fdo, original_irp, (ntkrnl_sys::constants::IO_DISK_INCREMENT) as _);

    IoFreeIrp(Irp);

    STATUS_MORE_PROCESSING_REQUIRED
}

#[no_mangle]
pub unsafe extern "system" fn DllUnload() -> i32 {
    DbgPrintEx(DPFLTR_CLASSPNP_ID, DPFLTR_INFO_LEVEL, b"classpnp.sys is now unloading\n\0".as_ptr() as *const i8);

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
pub unsafe extern "system" fn DriverEntry(
    DriverObject: *mut _DRIVER_OBJECT,
    RegistryPath: *mut _UNICODE_STRING,
) -> NTSTATUS {
    let _ = DriverObject;
    let _ = RegistryPath;
    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "system" fn ClassInitialize(
    Argument1: *mut core::ffi::c_void,
    Argument2: *mut core::ffi::c_void,
    InitializationData: *mut CLASS_INIT_DATA,
) -> NTSTATUS {
    let DriverObject = Argument1 as *mut DRIVER_OBJECT;
    let RegistryPath = Argument2 as *mut _UNICODE_STRING;

    let mut driverExtension: *mut CLASS_DRIVER_EXTENSION = core::ptr::null_mut();
    let mut status: NTSTATUS;

    // PAGED_CODE(); - assumed no-op

    // Security cookie initialization - disabled for ReactOS, ignore.

    TracePrint(TRACE_LEVEL_INFORMATION, TRACE_FLAG_INIT, b"\n\nSCSI Class Driver\n\0".as_ptr() as *const i8);

    ClasspInitializeDebugGlobals();

    if (*InitializationData).InitializationDataSize != core::mem::size_of::<CLASS_INIT_DATA>() as u32 {
        TracePrint(TRACE_LEVEL_ERROR, TRACE_FLAG_INIT, b"ClassInitialize: Class driver wrong version\n\0".as_ptr() as *const i8);
        return STATUS_REVISION_MISMATCH;
    }

    if (*InitializationData).FdoData.ClassDeviceControl.is_none() ||
        ((*InitializationData).FdoData.ClassReadWriteVerification.is_none() &&
         (*InitializationData).ClassStartIo.is_none()) ||
        (*InitializationData).ClassAddDevice.is_none() ||
        (*InitializationData).FdoData.ClassStartDevice.is_none()
    {
        TracePrint(TRACE_LEVEL_ERROR, TRACE_FLAG_INIT,
            b"ClassInitialize: Class device-specific driver missing required FDO entry\n\0".as_ptr() as *const i8);
        return STATUS_REVISION_MISMATCH;
    }

    if !(*InitializationData).ClassEnumerateDevice.is_none() &&
        ((*InitializationData).PdoData.ClassDeviceControl.is_none() ||
         (*InitializationData).PdoData.ClassStartDevice.is_none() ||
         ((*InitializationData).PdoData.ClassReadWriteVerification.is_none() &&
          (*InitializationData).ClassStartIo.is_none()))
    {
        TracePrint(TRACE_LEVEL_ERROR, TRACE_FLAG_INIT,
            b"ClassInitialize: Class device-specific missing required PDO entry\n\0".as_ptr() as *const i8);
        return STATUS_REVISION_MISMATCH;
    }

    if (*InitializationData).FdoData.ClassStopDevice.is_none() ||
        (!(*InitializationData).ClassEnumerateDevice.is_none() &&
         (*InitializationData).PdoData.ClassStopDevice.is_none())
    {
        TracePrint(TRACE_LEVEL_ERROR, TRACE_FLAG_INIT,
            b"ClassInitialize: Class device-specific missing required PDO entry\n\0".as_ptr() as *const i8);
        // NT_ASSERT(FALSE); - debug only
        return STATUS_REVISION_MISMATCH;
    }

    if (*InitializationData).FdoData.ClassPowerDevice.is_none() {
        (*InitializationData).FdoData.ClassPowerDevice = Some(ClassMinimalPowerHandler);
    }

    if (*InitializationData).ClassEnumerateDevice.is_some() &&
       (*InitializationData).PdoData.ClassPowerDevice.is_none()
    {
        (*InitializationData).PdoData.ClassPowerDevice = Some(ClassMinimalPowerHandler);
    }

    if (*InitializationData).ClassUnload.is_none() {
        TracePrint(TRACE_LEVEL_WARNING, TRACE_FLAG_INIT,
            b"ClassInitialize: driver does not support unload %wZ\n\0".as_ptr() as *const i8,
            RegistryPath);
    }

    let mut driverExtension_ptr: *mut core::ffi::c_void = core::ptr::null_mut();
    status = IoAllocateDriverObjectExtension(
        DriverObject,
        CLASS_DRIVER_EXTENSION_KEY(),
        core::mem::size_of::<CLASS_DRIVER_EXTENSION>() as u32,
        &mut driverExtension_ptr as *mut *mut core::ffi::c_void,
    );
    driverExtension = driverExtension_ptr as *mut CLASS_DRIVER_EXTENSION;

    if NT_SUCCESS(status) {
        (*driverExtension).RegistryPath.Length = (*RegistryPath).Length;
        (*driverExtension).RegistryPath.MaximumLength = (*RegistryPath).MaximumLength;

        let buffer = ExAllocatePoolWithTag(
            PagedPool,
            ((*RegistryPath).MaximumLength as u32) as usize,
            u32::from_le_bytes(*b"1CcS"),
        ) as *mut u16;

        if buffer.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        (*driverExtension).RegistryPath.Buffer = buffer;

        RtlCopyUnicodeString(
            &raw mut (*driverExtension).RegistryPath,
            RegistryPath,
        );

        RtlCopyMemory(
            &raw mut (*driverExtension).InitData as *mut core::ffi::c_void,
            InitializationData as *const core::ffi::c_void,
            core::mem::size_of::<CLASS_INIT_DATA>(),
        );

        (*driverExtension).DeviceCount = 0;

        ClassInitializeDispatchTables(driverExtension);
    } else if status == STATUS_OBJECT_NAME_COLLISION {
        driverExtension = IoGetDriverObjectExtension(DriverObject, CLASS_DRIVER_EXTENSION_KEY()) as *mut CLASS_DRIVER_EXTENSION;
        // NT_ASSERT(driverExtension != NULL);
    } else {
        TracePrint(TRACE_LEVEL_ERROR, TRACE_FLAG_INIT,
            b"ClassInitialize: Class driver extension could not be allocated %lx\n\0".as_ptr() as *const i8,
            status);
        return status;
    }

    (*DriverObject).MajorFunction[IRP_MJ_CREATE as usize] = Some(ClassGlobalDispatch);
    (*DriverObject).MajorFunction[IRP_MJ_CLOSE as usize] = Some(ClassGlobalDispatch);
    (*DriverObject).MajorFunction[IRP_MJ_READ as usize] = Some(ClassGlobalDispatch);
    (*DriverObject).MajorFunction[IRP_MJ_WRITE as usize] = Some(ClassGlobalDispatch);
    (*DriverObject).MajorFunction[IRP_MJ_SCSI as usize] = Some(ClassGlobalDispatch);
    (*DriverObject).MajorFunction[IRP_MJ_DEVICE_CONTROL as usize] = Some(ClassGlobalDispatch);
    (*DriverObject).MajorFunction[IRP_MJ_SHUTDOWN as usize] = Some(ClassGlobalDispatch);
    (*DriverObject).MajorFunction[IRP_MJ_FLUSH_BUFFERS as usize] = Some(ClassGlobalDispatch);
    (*DriverObject).MajorFunction[IRP_MJ_PNP as usize] = Some(ClassGlobalDispatch);
    (*DriverObject).MajorFunction[IRP_MJ_POWER as usize] = Some(ClassGlobalDispatch);
    (*DriverObject).MajorFunction[IRP_MJ_SYSTEM_CONTROL as usize] = Some(ClassGlobalDispatch);

    if !(*InitializationData).ClassStartIo.is_none() {
        (*DriverObject).DriverStartIo = Some(ClasspStartIo);
    }

    if !(*InitializationData).ClassUnload.is_none() && ClassPnpAllowUnload == (1u8) as u32 {
        (*DriverObject).DriverUnload = Some(core::mem::transmute::<
            unsafe extern "system" fn(*mut _DRIVER_OBJECT), _>(ClassUnload));
    } else {
        (*DriverObject).DriverUnload = None;
    }

    (*(*DriverObject).DriverExtension).AddDevice = Some(core::mem::transmute::<
        unsafe extern "system" fn(*mut _DRIVER_OBJECT, *mut _DEVICE_OBJECT) -> NTSTATUS,
        _,
    >(ClassAddDevice));

    // EtwHandle is a REGHANDLE (u64, align-8 on i386). A plain `(*driverExtension).EtwHandle`
    // read/write creates an align-8 access whose panic_misaligned_pointer_dereference check
    // drives the captures(none) whole-function fold (=> ud2). Use raw read/write_volatile.
    if core::ptr::read_volatile(core::ptr::addr_of!((*driverExtension).EtwHandle)) == 0 {
        status = EtwRegister(
            &raw const StoragePredictFailureDPSGuid,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!((*driverExtension).EtwHandle),
        );
        if !NT_SUCCESS(status) {
            core::ptr::write_volatile(core::ptr::addr_of_mut!((*driverExtension).EtwHandle), 0);
        }
        ntkrnl_sys::WPP_INIT_TRACING!(DriverObject, RegistryPath);
    }

    if IdlePowerFDOList.Flink.is_null() {
        InitializeListHead(&raw mut IdlePowerFDOList);
        KeInitializeGuardedMutex((&raw mut IdlePowerFDOListMutex).cast());
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn ClassInitializeEx(
    DriverObject: PDRIVER_OBJECT,
    Guid: *const GUID,
    Data: *mut core::ffi::c_void,
) -> NTSTATUS {
    // PAGED_CODE();

    let driverExtension = IoGetDriverObjectExtension(
        DriverObject,
        CLASS_DRIVER_EXTENSION_KEY(),
    ) as *mut CLASS_DRIVER_EXTENSION;

    if driverExtension.is_null() {
        NT_ASSERT!(0u8);
        return STATUS_UNSUCCESSFUL;
    }

    let mut status;

    if IsEqualGUID(Guid, core::ptr::addr_of!(ClassGuidQueryRegInfoEx)) != 0 {
        let List = Data as *mut CLASS_QUERY_WMI_REGINFO_EX_LIST;
        if (*List).Size == core::mem::size_of::<CLASS_QUERY_WMI_REGINFO_EX_LIST>() as u32 {
            (*driverExtension).ClassFdoQueryWmiRegInfoEx = (*List).ClassFdoQueryWmiRegInfoEx;
            (*driverExtension).ClassPdoQueryWmiRegInfoEx = (*List).ClassPdoQueryWmiRegInfoEx;
            status = STATUS_SUCCESS;
        } else {
            status = STATUS_INVALID_PARAMETER;
        }
    } else if IsEqualGUID(Guid, core::ptr::addr_of!(ClassGuidWorkingSet)) != 0 {
        let infoOriginal = Data as *mut CLASS_WORKING_SET;
        let mut info: *mut CLASS_WORKING_SET = core::ptr::null_mut();

        if (*infoOriginal).Size != core::mem::size_of::<CLASS_WORKING_SET>() as u32 {
            status = STATUS_INVALID_PARAMETER;
        } else {
            info = (ExAllocatePoolWithTag(
                NonPagedPoolNx,
                core::mem::size_of::<CLASS_WORKING_SET>(),
                CLASS_TAG_WORKING_SET,
            )).cast();
            if info.is_null() {
                status = STATUS_INSUFFICIENT_RESOURCES;
            } else {
                RtlCopyMemory(
                    info as *mut core::ffi::c_void,
                    infoOriginal as *const core::ffi::c_void,
                    core::mem::size_of::<CLASS_WORKING_SET>(),
                );
                status = STATUS_SUCCESS;
            }
        }

        if status >= 0 {
            if (*info).Size != core::mem::size_of::<CLASS_WORKING_SET>() as u32 {
                status = STATUS_INVALID_PARAMETER;
            } else if (*info).XferPacketsWorkingSetMaximum > CLASS_WORKING_SET_MAXIMUM {
                status = STATUS_INVALID_PARAMETER;
            } else if (*info).XferPacketsWorkingSetMinimum > CLASS_WORKING_SET_MAXIMUM {
                status = STATUS_INVALID_PARAMETER;
            } else if (*driverExtension).InitData.FdoData.DeviceType != FILE_DEVICE_CD_ROM as u32 {
                status = STATUS_INVALID_DEVICE_REQUEST;
            } else if !(*driverExtension).WorkingSet.is_null() {
                status = STATUS_INVALID_PARAMETER;
                NT_ASSERT!(0u8);
            }
        }

        if status >= 0 {
            (*driverExtension).WorkingSet = info;
            info = core::ptr::null_mut();
        } else {
            FREE_POOL(info as *mut core::ffi::c_void);
        }
    } else if IsEqualGUID(Guid, core::ptr::addr_of!(ClassGuidSenseInfo2)) != 0 {
        let infoOriginal = Data as *mut CLASS_INTERPRET_SENSE_INFO2;
        let mut info: *mut CLASS_INTERPRET_SENSE_INFO2 = core::ptr::null_mut();

        if (*infoOriginal).Size != core::mem::size_of::<CLASS_INTERPRET_SENSE_INFO2>() as u32 {
            status = STATUS_INVALID_PARAMETER;
        } else {
            info = (ExAllocatePoolWithTag(
                NonPagedPoolNx,
                core::mem::size_of::<CLASS_INTERPRET_SENSE_INFO2>(),
                CLASS_TAG_SENSE2,
            )).cast();
            if info.is_null() {
                status = STATUS_INSUFFICIENT_RESOURCES;
            } else {
                RtlCopyMemory(
                    info as *mut core::ffi::c_void,
                    infoOriginal as *const core::ffi::c_void,
                    core::mem::size_of::<CLASS_INTERPRET_SENSE_INFO2>(),
                );
                status = STATUS_SUCCESS;
            }
        }

        if status >= 0 {
            if (*info).Size != core::mem::size_of::<CLASS_INTERPRET_SENSE_INFO2>() as u32 {
                status = STATUS_INVALID_PARAMETER;
            } else if (*info).HistoryCount > CLASS_INTERPRET_SENSE_INFO2_MAXIMUM_HISTORY_COUNT {
                status = STATUS_INVALID_PARAMETER;
            } else if (*info).Compress.is_none() {
                status = STATUS_INVALID_PARAMETER;
            } else if (*info).HistoryCount == 0 {
                status = STATUS_INVALID_PARAMETER;
            } else if (*info).Interpret.is_none() {
                status = STATUS_INVALID_PARAMETER;
            } else if (*driverExtension).InitData.FdoData.DeviceType != FILE_DEVICE_CD_ROM as u32 {
                status = STATUS_INVALID_DEVICE_REQUEST;
            } else if !(*driverExtension).InterpretSenseInfo.is_null() {
                status = STATUS_INVALID_PARAMETER;
                NT_ASSERT!(0u8);
            }
        }

        if status >= 0 {
            (*driverExtension).InterpretSenseInfo = info;
            info = core::ptr::null_mut();
        } else {
            FREE_POOL(info as *mut core::ffi::c_void);
        }
    } else if IsEqualGUID(Guid, core::ptr::addr_of!(ClassGuidSrbSupport)) != 0 {
        let srbSupport = *(Data as *const ULONG);
        if (srbSupport & (CLASS_SRB_SCSI_REQUEST_BLOCK | CLASS_SRB_STORAGE_REQUEST_BLOCK)) != 0 {
            (*driverExtension).SrbSupport = srbSupport;
            status = STATUS_SUCCESS;
            if ((*driverExtension).SrbSupport & CLASS_SRB_SCSI_REQUEST_BLOCK) == 0 {
                NT_ASSERT!(0u8);
            }
        } else {
            status = STATUS_INVALID_PARAMETER;
        }
    } else {
        status = STATUS_NOT_SUPPORTED;
    }

    status
}
#[no_mangle]
pub unsafe extern "system" fn ClassUnload(DriverObject: PDRIVER_OBJECT) {
    PAGED_CODE!();
    NT_ASSERT!(((*DriverObject).DeviceObject.is_null() as i32) as u8);
    let driverExtension = IoGetDriverObjectExtension(DriverObject, CLASS_DRIVER_EXTENSION_KEY()) as *mut CLASS_DRIVER_EXTENSION;
    if driverExtension.is_null() {
        NT_ASSERT!((0i32) as u8);
        return;
    }
    NT_ASSERT!((!(*driverExtension).RegistryPath.Buffer.is_null() as i32) as u8);
    NT_ASSERT!(((!(*driverExtension).InitData.ClassUnload.is_none()) as i32) as u8);
    TracePrint(
        TRACE_LEVEL_INFORMATION,
        TRACE_FLAG_INIT,
        b"ClassUnload: driver unloading %wZ\n\0".as_ptr() as *const u8,
        core::ptr::addr_of_mut!((*driverExtension).RegistryPath) as *mut _UNICODE_STRING,
    );
    if let Some(unload_fn) = (*driverExtension).InitData.ClassUnload {
        unload_fn(DriverObject);
    }
    FREE_POOL((*driverExtension).WorkingSet as *mut core::ffi::c_void);
    FREE_POOL((*driverExtension).InterpretSenseInfo as *mut core::ffi::c_void);
    FREE_POOL((*driverExtension).RegistryPath.Buffer as *mut core::ffi::c_void);
    (*driverExtension).RegistryPath.Length = 0;
    (*driverExtension).RegistryPath.MaximumLength = 0;
    if (*driverExtension).EtwHandle != 0 {
        EtwUnregister((*driverExtension).EtwHandle);
        (*driverExtension).EtwHandle = 0;
        ntkrnl_sys::WPP_CLEANUP!(DriverObject);
    }
}

#[no_mangle]
pub unsafe extern "system" fn ClassAddDevice(
    DriverObject: PDRIVER_OBJECT,
    PhysicalDeviceObject: PDEVICE_OBJECT,
) -> NTSTATUS {
    // PAGED_CODE() — a DBG-only KdPrint + ASSERT in C (NOP_FUNCTION in retail).
    // The translation escalated it to an unconditional retail BUGCHECK with an
    // INVENTED stop code: 0x35 is NO_MORE_IRP_STACK_LOCATIONS, which has nothing
    // to do with IRQL (the §9.113 class). It evaded scan_bugcheck_codes.py for
    // six weeks because the literal carried a Rust type suffix (`0x35u32`) and
    // the scanner's regex required the digits to be followed directly by `,`.
    if KeGetCurrentIrql() > APC_LEVEL {
        DbgPrintEx(
            DPFLTR_CLASSPNP_ID,
            DPFLTR_INFO_LEVEL,
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
            KeGetCurrentIrql() as u32,
        );
    }
    // Key MUST match ClassInitialize's IoAllocateDriverObjectExtension registration —
    // a literal `1` here returned NULL and AV'd on the first-ever ClassAddDevice call
    // (USB disk bring-up; no prior boot exercised a disk AddDevice).
    let driver_extension = IoGetDriverObjectExtension(DriverObject, CLASS_DRIVER_EXTENSION_KEY()) as *mut _CLASS_DRIVER_EXTENSION;
    let status = (*driver_extension).InitData.ClassAddDevice.expect("ClassAddDevice")(DriverObject, PhysicalDeviceObject);
    status
}
#[no_mangle]
pub unsafe extern "system" fn ClassDispatchPnp(
    DeviceObject: *mut _DEVICE_OBJECT,
    Irp: *mut IRP,
) -> NTSTATUS {
    let common_extension = (*DeviceObject).DeviceExtension as *mut COMMON_DEVICE_EXTENSION;
    let is_fdo = (*common_extension).__bindgen_anon_1.IsFdo();
    let mut status = (*Irp).IoStatus.__bindgen_anon_1.Status;
    let mut complete_request = 1u8;
    let mut lock_released = 0u8;
    let irp_stack = IoGetCurrentIrpStackLocation(Irp);

    let driver_extension = IoGetDriverObjectExtension(
        (*DeviceObject).DriverObject,
        CLASS_DRIVER_EXTENSION_KEY(),
    ) as *mut CLASS_DRIVER_EXTENSION;

    if !driver_extension.is_null() {
        let init_data = &mut (*driver_extension).InitData;
        let dev_info = if is_fdo != 0 {
            &mut (*init_data).FdoData
        } else {
            &mut (*init_data).PdoData
        };

        ClassAcquireRemoveLock(DeviceObject, Irp);

        TracePrint(
            TRACE_LEVEL_INFORMATION,
            TRACE_FLAG_PNP,
            b"ClassDispatchPnp (%p,%p): minor code %#x for %s %p\n\0".as_ptr() as *const i8,
            DeviceObject,
            Irp,
            ((*irp_stack).MinorFunction) as i32,
            if is_fdo != 0 { b"fdo\0".as_ptr() } else { b"pdo\0".as_ptr() },
            DeviceObject,
        );
        TracePrint(
            TRACE_LEVEL_INFORMATION,
            TRACE_FLAG_PNP,
            b"ClassDispatchPnp (%p,%p): previous %#x, current %#x\n\0".as_ptr() as *const i8,
            DeviceObject,
            Irp,
            ((*common_extension).PreviousState) as i32,
            ((*common_extension).CurrentState) as i32,
        );

        match (*irp_stack).MinorFunction {
            IRP_MN_START_DEVICE => {
                if is_fdo != 0 {
                    status = ClassForwardIrpSynchronous(common_extension, Irp);
                } else {
                    status = STATUS_SUCCESS;
                }
                if NT_SUCCESS(status) {
                    status = ClassPnpStartDevice(DeviceObject);
                    (*Irp).IoStatus.__bindgen_anon_1.Status = status;
                }
            }

            IRP_MN_QUERY_DEVICE_RELATIONS => {
                let type_ = (*irp_stack).Parameters.QueryDeviceRelations.Type;

                if is_fdo == 0 {
                    if type_ == TargetDeviceRelation {
                        status = STATUS_INSUFFICIENT_RESOURCES;
                        let device_relations = ExAllocatePoolWithTag(
                            PagedPool,
                            (core::mem::size_of::<DEVICE_RELATIONS>() as u32) as usize,
                            u32::from_le_bytes(*b"ScC2"),
                        ) as *mut DEVICE_RELATIONS;

                        if !device_relations.is_null() {
                            RtlZeroMemory(
                                device_relations as *mut core::ffi::c_void,
                                (core::mem::size_of::<DEVICE_RELATIONS>() as u32) as usize,
                            );
                            (*Irp).IoStatus.Information = ((((device_relations as ULONG_PTR) as u32) as u64) as u32) as _;
                            (*device_relations).Count = 1;
                            (*device_relations).Objects[0] = DeviceObject;
                            ObReferenceObject((DeviceObject).cast());
                            status = STATUS_SUCCESS;
                        }
                    } else {
                        status = (*Irp).IoStatus.__bindgen_anon_1.Status;
                    }
                } else if type_ == BusRelations {
                    NT_ASSERT!(((*common_extension).__bindgen_anon_1.IsInitialized() != 0) as u8);

                    if !(*init_data).ClassEnumerateDevice.is_none() {
                        status = ClassPnpQueryFdoRelations(DeviceObject, Irp);
                        if !NT_SUCCESS(status) {
                            complete_request = 1u8;
                        }
                    }
                }

                if is_fdo != 0 {
                    IoCopyCurrentIrpStackLocationToNext(Irp);
                    ClassReleaseRemoveLock(DeviceObject, Irp);
                    status = IoCallDriver((*common_extension).LowerDeviceObject, Irp);
                    complete_request = 0u8;
                }
            }

            IRP_MN_QUERY_ID => {
                let id_type = (*irp_stack).Parameters.QueryId.IdType;
                let mut unicode_string: _UNICODE_STRING = core::mem::zeroed();

                if is_fdo != 0 {
                    IoCopyCurrentIrpStackLocationToNext(Irp);
                    ClassReleaseRemoveLock(DeviceObject, Irp);
                    status = IoCallDriver((*common_extension).LowerDeviceObject, Irp);
                    complete_request = 0u8;
                } else {
                    RtlInitUnicodeString(core::ptr::addr_of_mut!(unicode_string), core::ptr::null());
                    status = ClassGetPdoId(DeviceObject, id_type, core::ptr::addr_of_mut!(unicode_string));

                    if status == STATUS_NOT_IMPLEMENTED {
                        status = (*Irp).IoStatus.__bindgen_anon_1.Status;
                    } else if NT_SUCCESS(status) {
                        (*Irp).IoStatus.Information = ((((unicode_string.Buffer as ULONG_PTR) as u32) as u64) as u32) as _;
                    } else {
                        (*Irp).IoStatus.Information = 0;
                    }
                }
            }

            IRP_MN_QUERY_STOP_DEVICE | IRP_MN_QUERY_REMOVE_DEVICE => {
                TracePrint(
                    TRACE_LEVEL_INFORMATION,
                    TRACE_FLAG_PNP,
                    b"ClassDispatchPnp (%p,%p): Processing QUERY_%s irp\n\0".as_ptr() as *const i8,
                    DeviceObject,
                    Irp,
                    if (*irp_stack).MinorFunction == IRP_MN_QUERY_STOP_DEVICE {
                        b"STOP\0".as_ptr()
                    } else {
                        b"REMOVE\0".as_ptr()
                    },
                );

                if (*common_extension).PagingPathCount != 0 {
                    TracePrint(
                        TRACE_LEVEL_WARNING,
                        TRACE_FLAG_PNP,
                        b"ClassDispatchPnp (%p,%p): device is in paging path and cannot be removed\n\0".as_ptr() as *const i8,
                        DeviceObject,
                        Irp,
                    );
                    status = STATUS_DEVICE_BUSY;
                } else {
                    if (*irp_stack).MinorFunction == IRP_MN_QUERY_STOP_DEVICE {
                        status = (*dev_info).ClassStopDevice.expect("ClassStopDevice")(DeviceObject, (*irp_stack).MinorFunction);
                    } else {
                        status = (*dev_info).ClassRemoveDevice.expect("ClassRemoveDevice")(DeviceObject, (*irp_stack).MinorFunction);
                    }

                    if NT_SUCCESS(status) {
                        NT_ASSERT!(((*common_extension).CurrentState != (*irp_stack).MinorFunction) as u8);
                        (*common_extension).PreviousState = (*common_extension).CurrentState;
                        (*common_extension).CurrentState = (*irp_stack).MinorFunction;

                        if is_fdo != 0 {
                            TracePrint(
                                TRACE_LEVEL_INFORMATION,
                                TRACE_FLAG_PNP,
                                b"ClassDispatchPnp (%p,%p): Forwarding QUERY_%s irp\n\0".as_ptr() as *const i8,
                                DeviceObject,
                                Irp,
                                if (*irp_stack).MinorFunction == IRP_MN_QUERY_STOP_DEVICE {
                                    b"STOP\0".as_ptr()
                                } else {
                                    b"REMOVE\0".as_ptr()
                                },
                            );
                            status = ClassForwardIrpSynchronous(common_extension, Irp);
                        }
                    }
                }

                TracePrint(
                    TRACE_LEVEL_INFORMATION,
                    TRACE_FLAG_PNP,
                    b"ClassDispatchPnp (%p,%p): Final status == %x\n\0".as_ptr() as *const i8,
                    DeviceObject,
                    Irp,
                    status,
                );
            }

            IRP_MN_CANCEL_STOP_DEVICE | IRP_MN_CANCEL_REMOVE_DEVICE => {
                if (*irp_stack).MinorFunction == IRP_MN_CANCEL_STOP_DEVICE {
                    status = (*dev_info).ClassStopDevice.expect("ClassStopDevice")(DeviceObject, (*irp_stack).MinorFunction);
                    NT_ASSERTMSG!(b"ClassDispatchPnp !! CANCEL_STOP_DEVICE should never be failed\n\0".as_ptr() as *const i8, NT_SUCCESS(status) as BOOLEAN);
                } else {
                    status = (*dev_info).ClassRemoveDevice.expect("ClassRemoveDevice")(DeviceObject, (*irp_stack).MinorFunction);
                    NT_ASSERTMSG!(b"ClassDispatchPnp !! CANCEL_REMOVE_DEVICE should never be failed\n\0".as_ptr() as *const i8, NT_SUCCESS(status) as BOOLEAN);
                }

                (*Irp).IoStatus.__bindgen_anon_1.Status = status;

                if ((*irp_stack).MinorFunction == IRP_MN_CANCEL_STOP_DEVICE
                        && (*common_extension).CurrentState == IRP_MN_QUERY_STOP_DEVICE)
                    || ((*irp_stack).MinorFunction == IRP_MN_CANCEL_REMOVE_DEVICE
                        && (*common_extension).CurrentState == IRP_MN_QUERY_REMOVE_DEVICE)
                {
                    (*common_extension).CurrentState = (*common_extension).PreviousState;
                    (*common_extension).PreviousState = 0xff;
                }

                if is_fdo != 0 {
                    IoCopyCurrentIrpStackLocationToNext(Irp);
                    ClassReleaseRemoveLock(DeviceObject, Irp);
                    status = IoCallDriver((*common_extension).LowerDeviceObject, Irp);
                    complete_request = 0u8;
                } else {
                    status = STATUS_SUCCESS;
                }
            }

            IRP_MN_STOP_DEVICE => {
                TracePrint(
                    TRACE_LEVEL_INFORMATION,
                    TRACE_FLAG_PNP,
                    b"ClassDispatchPnp (%p,%p): got stop request for %s\n\0".as_ptr() as *const i8,
                    DeviceObject,
                    Irp,
                    if is_fdo != 0 { b"fdo\0".as_ptr() } else { b"pdo\0".as_ptr() },
                );

                NT_ASSERT!(((*common_extension).PagingPathCount == 0) as u8);

                ClasspDisableTimer(common_extension as *mut FUNCTIONAL_DEVICE_EXTENSION);

                status = (*dev_info).ClassStopDevice.expect("ClassStopDevice")(DeviceObject, IRP_MN_STOP_DEVICE);
                NT_ASSERTMSG!(b"ClassDispatchPnp !! STOP_DEVICE should never be failed\n\0".as_ptr() as *const i8, NT_SUCCESS(status) as BOOLEAN);

                if is_fdo != 0 {
                    status = ClassForwardIrpSynchronous(common_extension, Irp);
                }

                if NT_SUCCESS(status) {
                    (*common_extension).CurrentState = (*irp_stack).MinorFunction;
                    (*common_extension).PreviousState = 0xff;
                }
            }

            IRP_MN_REMOVE_DEVICE | IRP_MN_SURPRISE_REMOVAL => {
                let remove_type = (*irp_stack).MinorFunction;

                if is_fdo != 0 && remove_type == IRP_MN_SURPRISE_REMOVAL {
                    let fdo_extension = DeviceObject as *mut FUNCTIONAL_DEVICE_EXTENSION;
                    let fdo_data = (*fdo_extension).PrivateFdoData;
                    let mut log_surprise_remove = 1u8;
                    let bus_type = (*(*fdo_extension).DeviceDescriptor).BusType;

                    if bus_type == BusTypeFileBackedVirtual {
                        log_surprise_remove = 0u8;
                    } else if (*fdo_data).HotplugInfo.MediaRemovable != 0 {
                        log_surprise_remove = 0u8;
                    } else if (*fdo_data).HotplugInfo.DeviceHotplug != 0
                        && (bus_type == BusTypeUsb || bus_type == BusType1394)
                    {
                        log_surprise_remove = 0u8;
                    }

                    if log_surprise_remove != 0 {
                        ClasspLogSystemEventWithDeviceNumber(DeviceObject, (IO_WARNING_DISK_SURPRISE_REMOVED) as u32);
                    }
                }

                if (*common_extension).PagingPathCount != 0 {
                    TracePrint(
                        TRACE_LEVEL_WARNING,
                        TRACE_FLAG_PNP,
                        b"ClassDispatchPnp (%p,%p): paging device is getting removed!\n\0".as_ptr() as *const i8,
                        DeviceObject,
                        Irp,
                    );
                }

                ClassReleaseRemoveLock(DeviceObject, Irp);
                lock_released = 1u8;
                (*common_extension).IsRemoved = REMOVE_PENDING;
                ClasspDisableTimer(common_extension as *mut FUNCTIONAL_DEVICE_EXTENSION);

                if is_fdo != 0 {
                    IoCopyCurrentIrpStackLocationToNext(Irp);
                    status = IoCallDriver((*common_extension).LowerDeviceObject, Irp);
                    NT_ASSERT!(NT_SUCCESS(status) as u8);
                    complete_request = 0u8;
                } else {
                    status = STATUS_SUCCESS;
                }

                (*common_extension).PreviousState = (*common_extension).CurrentState;
                (*common_extension).CurrentState = remove_type;
                ClassRemoveDevice(DeviceObject, remove_type);
            }

            IRP_MN_DEVICE_USAGE_NOTIFICATION => {
                let usage_type = (*irp_stack).Parameters.UsageNotification.Type;

                match usage_type {
                    DeviceUsageTypePaging => {
                        if (*irp_stack).Parameters.UsageNotification.InPath != 0
                            && (*common_extension).CurrentState != IRP_MN_START_DEVICE
                        {
                            status = STATUS_DEVICE_NOT_READY;
                        } else {
                            NT_ASSERT!(((*common_extension).__bindgen_anon_1.IsInitialized() != 0) as u8);
                            KeEnterCriticalRegion();
                            KeWaitForSingleObject(
                                core::ptr::addr_of_mut!((*common_extension).PathCountEvent) as *mut core::ffi::c_void,
                                _KWAIT_REASON_Executive,
                                (0i32) as i8,
                                0u8,
                                core::ptr::null_mut(),
                            );
                            status = STATUS_SUCCESS;

                            if (*common_extension).__bindgen_anon_1.IsFdo() != 0 {
                                status = ClasspEjectionControl(
                                    DeviceObject,
                                    Irp,
                                    MEDIA_LOCK_TYPE_InternalMediaLock,
                                    (*irp_stack).Parameters.UsageNotification.InPath as u8,
                                );
                            }

                            if !NT_SUCCESS(status) {
                                KeSetEvent(
                                    core::ptr::addr_of_mut!((*common_extension).PathCountEvent) as PVOID,
                                    IO_NO_INCREMENT as i32,
                                    0u8,
                                );
                                KeLeaveCriticalRegion();
                            } else {
                                let mut set_pagable = 0u8;
                                if (*irp_stack).Parameters.UsageNotification.InPath == 0
                                    && (*common_extension).PagingPathCount == 1
                                {
                                    if (*DeviceObject).Flags & DO_POWER_INRUSH != 0 {
                                        TracePrint(
                                            TRACE_LEVEL_INFORMATION,
                                            TRACE_FLAG_PNP,
                                            b"ClassDispatchPnp (%p,%p): Last paging file removed, but DO_POWER_INRUSH was set, so NOT setting DO_POWER_PAGABLE\n\0".as_ptr() as *const i8,
                                            DeviceObject,
                                            Irp,
                                        );
                                    } else if (*common_extension).HibernationPathCount == 0
                                        && (*common_extension).DumpPathCount == 0
                                    {
                                        TracePrint(
                                            TRACE_LEVEL_INFORMATION,
                                            TRACE_FLAG_PNP,
                                            b"ClassDispatchPnp (%p,%p): Last paging file removed, setting DO_POWER_PAGABLE\n\0".as_ptr() as *const i8,
                                            DeviceObject,
                                            Irp,
                                        );
                                        (*DeviceObject).Flags |= DO_POWER_PAGABLE;
                                        set_pagable = 1u8;
                                    }
                                }

                                status = ClassForwardIrpSynchronous(common_extension, Irp);

                                if NT_SUCCESS(status) {
                                    IoAdjustPagingPathCount(
                                        core::ptr::addr_of_mut!((*common_extension).PagingPathCount) as *mut i32,
                                        (*irp_stack).Parameters.UsageNotification.InPath as u8,
                                    );

                                    if (*irp_stack).Parameters.UsageNotification.InPath != 0 {
                                        if (*common_extension).PagingPathCount == 1 {
                                            TracePrint(
                                                TRACE_LEVEL_INFORMATION,
                                                TRACE_FLAG_PNP,
                                                b"ClassDispatchPnp (%p,%p): Clearing PAGABLE bit\n\0".as_ptr() as *const i8,
                                                DeviceObject,
                                                Irp,
                                            );
                                            (*DeviceObject).Flags &= !DO_POWER_PAGABLE;
                                        }
                                    }
                                } else {
                                    if set_pagable != 0 {
                                        TracePrint(
                                            TRACE_LEVEL_INFORMATION,
                                            TRACE_FLAG_PNP,
                                            b"ClassDispatchPnp (%p,%p): Unsetting PAGABLE bit due to irp failure\n\0".as_ptr() as *const i8,
                                            DeviceObject,
                                            Irp,
                                        );
                                        (*DeviceObject).Flags &= !DO_POWER_PAGABLE;
                                    }
                                    if (*common_extension).__bindgen_anon_1.IsFdo() != 0 {
                                        ClasspEjectionControl(
                                            DeviceObject,
                                            Irp,
                                            MEDIA_LOCK_TYPE_InternalMediaLock,
                                            (1 - (*irp_stack).Parameters.UsageNotification.InPath) as u8,
                                        );
                                    }
                                }

                                KeSetEvent(
                                    core::ptr::addr_of_mut!((*common_extension).PathCountEvent) as PVOID,
                                    IO_NO_INCREMENT as i32,
                                    0u8,
                                );
                                KeLeaveCriticalRegion();
                            }
                        }
                    }

                    DeviceUsageTypeHibernation => {
                        let mut set_pagable = 0u8;
                        if (*irp_stack).Parameters.UsageNotification.InPath == 0
                            && (*common_extension).HibernationPathCount == 1
                        {
                            if (*DeviceObject).Flags & DO_POWER_INRUSH != 0 {
                                TracePrint(
                                    TRACE_LEVEL_INFORMATION,
                                    TRACE_FLAG_PNP,
                                    b"ClassDispatchPnp (%p,%p): Last hiber file removed, but DO_POWER_INRUSH was set, so NOT setting DO_POWER_PAGABLE\n\0".as_ptr() as *const i8,
                                    DeviceObject,
                                    Irp,
                                );
                            } else if (*common_extension).PagingPathCount == 0
                                && (*common_extension).DumpPathCount == 0
                            {
                                TracePrint(
                                    TRACE_LEVEL_INFORMATION,
                                    TRACE_FLAG_PNP,
                                    b"ClassDispatchPnp (%p,%p): Last hiber file removed, setting DO_POWER_PAGABLE\n\0".as_ptr() as *const i8,
                                    DeviceObject,
                                    Irp,
                                );
                                (*DeviceObject).Flags |= DO_POWER_PAGABLE;
                                set_pagable = 1u8;
                            }
                        }

                        status = ClassForwardIrpSynchronous(common_extension, Irp);
                        if !NT_SUCCESS(status) {
                            if set_pagable != 0 {
                                TracePrint(
                                    TRACE_LEVEL_INFORMATION,
                                    TRACE_FLAG_PNP,
                                    b"ClassDispatchPnp (%p,%p): Unsetting PAGABLE bit due to irp failure\n\0".as_ptr() as *const i8,
                                    DeviceObject,
                                    Irp,
                                );
                                (*DeviceObject).Flags &= !DO_POWER_PAGABLE;
                            }
                        } else {
                            IoAdjustPagingPathCount(
                                core::ptr::addr_of_mut!((*common_extension).HibernationPathCount) as *mut i32,
                                (*irp_stack).Parameters.UsageNotification.InPath as u8,
                            );
                            if (*irp_stack).Parameters.UsageNotification.InPath != 0
                                && (*common_extension).HibernationPathCount == 1
                            {
                                TracePrint(
                                    TRACE_LEVEL_INFORMATION,
                                    TRACE_FLAG_PNP,
                                    b"ClassDispatchPnp (%p,%p): Clearing PAGABLE bit\n\0".as_ptr() as *const i8,
                                    DeviceObject,
                                    Irp,
                                );
                                (*DeviceObject).Flags &= !DO_POWER_PAGABLE;
                            }
                        }
                    }

                    DeviceUsageTypeDumpFile => {
                        let mut set_pagable = 0u8;
                        if (*irp_stack).Parameters.UsageNotification.InPath == 0
                            && (*common_extension).DumpPathCount == 1
                        {
                            if (*DeviceObject).Flags & DO_POWER_INRUSH != 0 {
                                TracePrint(
                                    TRACE_LEVEL_INFORMATION,
                                    TRACE_FLAG_PNP,
                                    b"ClassDispatchPnp (%p,%p): Last dump file removed, but DO_POWER_INRUSH was set, so NOT setting DO_POWER_PAGABLE\n\0".as_ptr() as *const i8,
                                    DeviceObject,
                                    Irp,
                                );
                            } else if (*common_extension).PagingPathCount == 0
                                && (*common_extension).HibernationPathCount == 0
                            {
                                TracePrint(
                                    TRACE_LEVEL_INFORMATION,
                                    TRACE_FLAG_PNP,
                                    b"ClassDispatchPnp (%p,%p): Last dump file removed, setting DO_POWER_PAGABLE\n\0".as_ptr() as *const i8,
                                    DeviceObject,
                                    Irp,
                                );
                                (*DeviceObject).Flags |= DO_POWER_PAGABLE;
                                set_pagable = 1u8;
                            }
                        }

                        status = ClassForwardIrpSynchronous(common_extension, Irp);
                        if !NT_SUCCESS(status) {
                            if set_pagable != 0 {
                                TracePrint(
                                    TRACE_LEVEL_INFORMATION,
                                    TRACE_FLAG_PNP,
                                    b"ClassDispatchPnp (%p,%p): Unsetting PAGABLE bit due to irp failure\n\0".as_ptr() as *const i8,
                                    DeviceObject,
                                    Irp,
                                );
                                (*DeviceObject).Flags &= !DO_POWER_PAGABLE;
                            }
                        } else {
                            IoAdjustPagingPathCount(
                                core::ptr::addr_of_mut!((*common_extension).DumpPathCount) as *mut i32,
                                (*irp_stack).Parameters.UsageNotification.InPath as u8,
                            );
                            if (*irp_stack).Parameters.UsageNotification.InPath != 0
                                && (*common_extension).DumpPathCount == 1
                            {
                                TracePrint(
                                    TRACE_LEVEL_INFORMATION,
                                    TRACE_FLAG_PNP,
                                    b"ClassDispatchPnp (%p,%p): Clearing PAGABLE bit\n\0".as_ptr() as *const i8,
                                    DeviceObject,
                                    Irp,
                                );
                                (*DeviceObject).Flags &= !DO_POWER_PAGABLE;
                            }
                        }
                    }

                    DeviceUsageTypeBoot => {
                        if is_fdo != 0 {
                            let fdo_data = (*(DeviceObject as *mut FUNCTIONAL_DEVICE_EXTENSION)).PrivateFdoData;
                            if !fdo_data.is_null()
                                && (*fdo_data).HotplugInfo.DeviceHotplug != 0
                            {
                                (*fdo_data).HotplugInfo.DeviceHotplug = 0u8;
                                (*fdo_data).HotplugInfo.MediaRemovable = 0u8;
                                ClassSetDeviceParameter(
                                    DeviceObject as *mut FUNCTIONAL_DEVICE_EXTENSION,
                                    crate::glue::regw::CLASSP_REG_SUBKEY_NAME.as_ptr() as PWSTR,
                                    crate::glue::regw::CLASSP_REG_REMOVAL_POLICY_VALUE_NAME.as_ptr() as PWSTR,
                                    RemovalPolicyExpectOrderlyRemoval,
                                );
                            }
                        }
                        status = ClassForwardIrpSynchronous(common_extension, Irp);
                    }

                    _ => {
                        status = STATUS_INVALID_PARAMETER;
                    }
                }
            }

            IRP_MN_QUERY_CAPABILITIES => {
                TracePrint(
                    TRACE_LEVEL_INFORMATION,
                    TRACE_FLAG_PNP,
                    b"ClassDispatchPnp (%p,%p): QueryCapabilities\n\0".as_ptr() as *const i8,
                    DeviceObject,
                    Irp,
                );

                if is_fdo == 0 {
                    status = ClassQueryPnpCapabilities(
                        DeviceObject,
                        (*irp_stack).Parameters.DeviceCapabilities.Capabilities,
                    );
                } else {
                    let device_capabilities = (*irp_stack).Parameters.DeviceCapabilities.Capabilities;
                    let fdo_extension = DeviceObject as *mut FUNCTIONAL_DEVICE_EXTENSION;
                    let fdo_data = (*fdo_extension).PrivateFdoData;

                    status = ClassForwardIrpSynchronous(common_extension, Irp);
                    if NT_SUCCESS(status) {
                        if !fdo_data.is_null()
                            && (*fdo_data).HotplugInfo.WriteCacheEnableOverride != 0
                        {
                            if (*device_capabilities).SurpriseRemovalOK() != 0 {
                                TracePrint(
                                    TRACE_LEVEL_INFORMATION,
                                    TRACE_FLAG_PNP,
                                    b"Classpnp: Clearing SR-OK bit in device capabilities due to hotplug device or media\n\0".as_ptr() as *const i8,
                                );
                            }
                            (*device_capabilities).set_SurpriseRemovalOK((0u8) as u32);
                        }
                    }
                }
            }

            _ => {
                if is_fdo != 0 {
                    IoCopyCurrentIrpStackLocationToNext(Irp);
                    ClassReleaseRemoveLock(DeviceObject, Irp);
                    status = IoCallDriver((*common_extension).LowerDeviceObject, Irp);
                    complete_request = 0u8;
                }
            }
        }
    } else {
        NT_ASSERT!((!driver_extension.is_null()) as u8);
        status = STATUS_INTERNAL_ERROR;
    }

    if complete_request != 0 {
        (*Irp).IoStatus.__bindgen_anon_1.Status = status;
        if lock_released == 0 {
            ClassReleaseRemoveLock(DeviceObject, Irp);
        }
        ClassCompleteRequest(DeviceObject, Irp, (IO_NO_INCREMENT) as _);
        TracePrint(
            TRACE_LEVEL_INFORMATION,
            TRACE_FLAG_PNP,
            b"ClassDispatchPnp (%p,%p): leaving with previous %#x, current %#x.\n\0".as_ptr() as *const i8,
            DeviceObject,
            Irp,
            (if !driver_extension.is_null() { (*common_extension).PreviousState } else { 0 }) as i32,
            (if !driver_extension.is_null() { (*common_extension).CurrentState } else { 0 }) as i32,
        );
    } else {
        TracePrint(
            TRACE_LEVEL_INFORMATION,
            TRACE_FLAG_PNP,
            b"ClassDispatchPnp (%p,%p): leaving.\n\0".as_ptr() as *const i8,
            DeviceObject,
            Irp,
        );
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn ClassPnpStartDevice(device_object: *mut _DEVICE_OBJECT) -> NTSTATUS {
    let common_extension = (*device_object).DeviceExtension as *mut _COMMON_DEVICE_EXTENSION;
    let fdo_extension = (*device_object).DeviceExtension as *mut _FUNCTIONAL_DEVICE_EXTENSION;
    let is_fdo: BOOLEAN = (*common_extension).__bindgen_anon_1.IsFdo();
    let mut is_mounted_device: BOOLEAN = 1u8;
    let mut is_portable: BOOLEAN = 0u8;
    let mut status: NTSTATUS = STATUS_SUCCESS;
    let mut power_descriptor: *mut _DEVICE_POWER_DESCRIPTOR = core::ptr::null_mut();

    // PAGED_CODE(); omitted in Rust – assumed handled by caller

    let driver_extension = IoGetDriverObjectExtension(
        (*device_object).DriverObject,
        CLASS_DRIVER_EXTENSION_KEY(),
    ) as *mut _CLASS_DRIVER_EXTENSION;
    let init_data: *mut _CLASS_INIT_DATA = &raw mut (*driver_extension).InitData;
    let dev_info: *mut _CLASS_DEV_INFO;

    if is_fdo != 0 {
        dev_info = &raw mut (*init_data).FdoData;
    } else {
        dev_info = &raw mut (*init_data).PdoData;
    }

    NT_ASSERT!((*dev_info).ClassInitDevice.is_some() as u8);
    NT_ASSERT!((*dev_info).ClassStartDevice.is_some() as u8);

    if (*common_extension).__bindgen_anon_1.IsInitialized() == 0 {
        if is_fdo != 0 {
            let mut property_id: STORAGE_PROPERTY_ID;

            if (*fdo_extension).PrivateFdoData == core::ptr::null_mut() {
                (*fdo_extension).PrivateFdoData = ExAllocatePoolWithTag(
                    NonPagedPoolNx,
                    (size_of::<_CLASS_PRIVATE_FDO_DATA>() as u32) as usize,
                    CLASS_TAG_PRIVATE_DATA,
                ) as *mut _CLASS_PRIVATE_FDO_DATA;
            }
            if (*fdo_extension).PrivateFdoData == core::ptr::null_mut() {
                TracePrint(TRACE_LEVEL_ERROR, TRACE_FLAG_PNP, b"ClassPnpStartDevice: Cannot allocate for private fdo data\n\0".as_ptr() as PCHAR);
                return STATUS_INSUFFICIENT_RESOURCES;
            }
            core::ptr::write_bytes(
                (*fdo_extension).PrivateFdoData as *mut u8,
                0u8,
                size_of::<_CLASS_PRIVATE_FDO_DATA>(),
            );

            #[cfg(ntddi_ge_ntddi_winthreshold)]
            {
                if (*fdo_extension).AdditionalFdoData == core::ptr::null_mut() {
                    (*fdo_extension).AdditionalFdoData = ExAllocatePoolWithTag(
                        NonPagedPoolNx,
                        size_of::<_ADDITIONAL_FDO_DATA>() as u32,
                        CLASSPNP_POOL_TAG_ADDITIONAL_DATA,
                    ) as *mut _ADDITIONAL_FDO_DATA;
                }
                if (*fdo_extension).AdditionalFdoData == core::ptr::null_mut() {
                    TracePrint(TRACE_LEVEL_ERROR, TRACE_FLAG_PNP, b"ClassPnpStartDevice: Cannot allocate memory for the additional data structure.\n\0".as_ptr() as PCHAR);
                    return STATUS_INSUFFICIENT_RESOURCES;
                }
                core::ptr::write_bytes(
                    (*fdo_extension).AdditionalFdoData as *mut u8,
                    0u8,
                    size_of::<_ADDITIONAL_FDO_DATA>(),
                );
            }

            status = ClasspInitializeTimer(fdo_extension);
            if !NT_SUCCESS(status) {
                FREE_POOL((*fdo_extension).PrivateFdoData as *mut c_void);
                #[cfg(ntddi_ge_ntddi_winthreshold)]
                FREE_POOL((*fdo_extension).AdditionalFdoData as *mut c_void);
                TracePrint(TRACE_LEVEL_ERROR, TRACE_FLAG_PNP, b"ClassPnpStartDevice: Failed to initialize tick timer\n\0".as_ptr() as PCHAR);
                return STATUS_INSUFFICIENT_RESOURCES;
            }

            if (*fdo_extension).FunctionSupportInfo == core::ptr::null_mut() {
                (*fdo_extension).FunctionSupportInfo = ExAllocatePoolWithTag(
                    NonPagedPoolNx,
                    (size_of::<_CLASS_FUNCTION_SUPPORT_INFO>() as u32) as usize,
                    u32::from_be_bytes([b'3', b'B', b'c', b'S']),
                ) as *mut _CLASS_FUNCTION_SUPPORT_INFO;
            }
            if (*fdo_extension).FunctionSupportInfo == core::ptr::null_mut() {
                ClasspDeleteTimer(fdo_extension);
                FREE_POOL((*fdo_extension).PrivateFdoData as *mut c_void);
                #[cfg(ntddi_ge_ntddi_winthreshold)]
                FREE_POOL((*fdo_extension).AdditionalFdoData as *mut c_void);
                TracePrint(TRACE_LEVEL_ERROR, TRACE_FLAG_PNP, b"ClassPnpStartDevice: Cannot allocate for FunctionSupportInfo\n\0".as_ptr() as PCHAR);
                return STATUS_INSUFFICIENT_RESOURCES;
            }

            core::ptr::write_bytes(
                (*fdo_extension).FunctionSupportInfo as *mut u8,
                0u8,
                size_of::<_CLASS_FUNCTION_SUPPORT_INFO>(),
            );
            KeInitializeSpinLock(&raw mut (*(*fdo_extension).FunctionSupportInfo).SyncLock);

            (*(*fdo_extension).FunctionSupportInfo).BlockLimitsData.CommandStatus = -1;
            (*(*fdo_extension).FunctionSupportInfo).DeviceCharacteristicsData.CommandStatus = -1;
            (*(*fdo_extension).FunctionSupportInfo).LBProvisioningData.CommandStatus = -1;
            (*(*fdo_extension).FunctionSupportInfo).ReadCapacity16Data.CommandStatus = -1;
            (*(*fdo_extension).FunctionSupportInfo).BlockDeviceRODLimitsData.CommandStatus = -1;

            KeInitializeTimer(&raw mut (*(*fdo_extension).PrivateFdoData).Retry.Timer);
            KeInitializeDpc(
                &raw mut (*(*fdo_extension).PrivateFdoData).Retry.Dpc,
                Some(ClasspRetryRequestDpc),
                device_object as PVOID,
            );
            KeInitializeSpinLock(&raw mut (*(*fdo_extension).PrivateFdoData).Retry.Lock);
            (*(*fdo_extension).PrivateFdoData).Retry.Granularity = KeQueryTimeIncrement();
            (*common_extension).Reserved4 = ((((u32::from_be_bytes([b' ', b'G', b'P', b'H']) as usize as ULONG_PTR) as u32) as u64) as u32) as _; // debug aid
            InitializeListHead(&raw mut (*(*fdo_extension).PrivateFdoData).DeferredClientIrpList);
            KeInitializeSpinLock(&raw mut (*(*fdo_extension).PrivateFdoData).SpinLock);

            (*(*fdo_extension).PrivateFdoData).InterpretSenseInfo = (*driver_extension).InterpretSenseInfo;
            (*(*fdo_extension).PrivateFdoData).MaxNumberOfIoRetries = (NUM_IO_RETRIES) as u8;

            // NB: sizeof the whole union, NOT size_of_val(&...ReleaseQueueSrbBuffer) —
            // that field is a bindgen __BindgenUnionField<[UCHAR;144]> ZST marker, so
            // size_of_val on it returns 0 and InitializeStorageRequestBlock rejects the
            // "0-byte" buffer (START then fails 0xC000000D; cost a USB-disk debug hunt).
            status = InitializeStorageRequestBlock(
                (*(*fdo_extension).PrivateFdoData).ReleaseQueueSrb.SrbEx.as_mut() as *mut _,
                STORAGE_ADDRESS_TYPE_BTL8 as u16,
                size_of::<crate::types::_CLASS_PRIVATE_FDO_DATA__bindgen_ty_2>() as u32,
                0,
            );
            if !NT_SUCCESS(status) {
                {
                    unsafe extern "C" { fn DbgPrint(Format: *const i8, ...) -> u32; }
                    DbgPrint(
                        c"[CLASSPNP] InitSRB dbg: status=0x%x addrtype=%u bytesize=%u sizeof(SRB)=%u sizeof(BTL8)=%u\n".as_ptr(),
                        status as u32,
                        STORAGE_ADDRESS_TYPE_BTL8 as u32,
                        size_of_val(&(*(*fdo_extension).PrivateFdoData).ReleaseQueueSrb.ReleaseQueueSrbBuffer) as u32,
                        size_of::<crate::types::STORAGE_REQUEST_BLOCK>() as u32,
                        size_of::<crate::types::STOR_ADDR_BTL8>() as u32,
                    );
                }
                NT_ASSERT!(0u8);
                TracePrint(TRACE_LEVEL_ERROR, TRACE_FLAG_PNP,
                    b"ClassPnpStartDevice: fail to initialize release queue extended SRB 0x%x\n\0".as_ptr() as PCHAR, status);
                return status;
            }

            InsertTailList(
                &raw mut AllFdosList,
                &raw mut (*(*fdo_extension).PrivateFdoData).AllFdosListEntry,
            );

            status = ClasspAllocateReleaseQueueIrp(fdo_extension);
            if !NT_SUCCESS(status) {
                TracePrint(TRACE_LEVEL_ERROR, TRACE_FLAG_PNP, b"ClassPnpStartDevice: Cannot allocate the private release queue irp\n\0".as_ptr() as PCHAR);
                return status;
            }

            status = ClasspAllocatePowerProcessIrp(fdo_extension);
            if !NT_SUCCESS(status) {
                TracePrint(TRACE_LEVEL_ERROR, TRACE_FLAG_PNP, b"ClassPnpStartDevice: Cannot allocate the power process irp\n\0".as_ptr() as PCHAR);
                return status;
            }

            if (*device_object).DeviceType == FILE_DEVICE_DISK &&
                (TEST_FLAG((*device_object).Characteristics, FILE_FLOPPY_DISKETTE) == 0)
            {
                property_id = StorageMiniportProperty;
                status = ClassGetDescriptor(
                    (*fdo_extension).__bindgen_anon_1.CommonExtension.as_ref().LowerDeviceObject,
                    &raw mut property_id,
                    &raw mut (*fdo_extension).MiniportDescriptor as *mut PVOID,
                );

                if NT_SUCCESS(status) &&
                    ((*(*fdo_extension).MiniportDescriptor).Portdriver != StoragePortCodeSetStorport &&
                     (*(*fdo_extension).MiniportDescriptor).Portdriver != StoragePortCodeSetUSBport)
                {
                    // IoTimeoutValue field not present in this bindgen layout; skip
                }
            }

            property_id = StorageAdapterProperty;
            status = ClassGetDescriptor(
                (*common_extension).LowerDeviceObject,
                &raw mut property_id,
                &raw mut (*fdo_extension).AdapterDescriptor as *mut PVOID,
            );
            if !NT_SUCCESS(status) {
                TracePrint(TRACE_LEVEL_ERROR, TRACE_FLAG_PNP, b"ClassPnpStartDevice: ClassGetDescriptor [ADAPTER] failed %lx\n\0".as_ptr() as PCHAR, status);
                return status;
            }

            property_id = StorageDeviceProperty;
            status = ClassGetDescriptor(
                (*common_extension).LowerDeviceObject,
                &raw mut property_id,
                &raw mut (*fdo_extension).DeviceDescriptor as *mut PVOID,
            );
            if NT_SUCCESS(status) {
                ClasspScanForSpecialInRegistry(fdo_extension);
                ClassScanForSpecial(fdo_extension, ClassBadItems.as_mut_ptr(), Some(ClasspScanForClassHacks));

                let mut t: ULONG = CLASS_PERF_RESTORE_MINIMUM;
                ClassGetDeviceParameter(
                    fdo_extension,
                    crate::glue::regw::CLASSP_REG_SUBKEY_NAME.as_ptr() as PWSTR,
                    crate::glue::regw::CLASSP_REG_PERF_RESTORE_VALUE_NAME.as_ptr() as PWSTR,
                    &raw mut t,
                );
                if t >= CLASS_PERF_RESTORE_MINIMUM {
                    (*(*fdo_extension).PrivateFdoData).Perf.ReEnableThreshhold = t;
                }

                if (*(*(*fdo_extension).__bindgen_anon_1.CommonExtension.as_ref()).DeviceObject).DeviceType != FILE_DEVICE_DISK {
                    SET_FLAG((&raw mut (*(*fdo_extension).PrivateFdoData).HackFlags).cast(), FDO_HACK_NO_SYNC_CACHE);
                }

                status = ClasspIsPortable(fdo_extension, &mut is_portable);
                if NT_SUCCESS(status) && (is_portable != 0) {
                    (*device_object).Characteristics |= FILE_PORTABLE_DEVICE;
                }

                status = ClasspInitializeHotplugInfo(fdo_extension);
                if NT_SUCCESS(status) {
                    status = InitializeTransferPackets(device_object);
                } else {
                    TracePrint(TRACE_LEVEL_WARNING, TRACE_FLAG_PNP, b"ClassPnpStartDevice: Could not initialize hotplug information %lx\n\0".as_ptr() as PCHAR, status);
                }
            } else {
                TracePrint(TRACE_LEVEL_ERROR, TRACE_FLAG_PNP, b"ClassPnpStartDevice: ClassGetDescriptor [DEVICE] failed %lx\n\0".as_ptr() as PCHAR, status);
                return status;
            }

            if NT_SUCCESS(status) {
                property_id = StorageDevicePowerProperty;
                status = ClassGetDescriptor(
                    (*fdo_extension).__bindgen_anon_1.CommonExtension.as_ref().LowerDeviceObject,
                    &raw mut property_id,
                    &raw mut power_descriptor as *mut PVOID,
                );
                if NT_SUCCESS(status) && !power_descriptor.is_null() {
                    (*(*fdo_extension).FunctionSupportInfo).AsynchronousNotificationSupported = (*power_descriptor).AsynchronousNotificationSupported;
                    (*(*fdo_extension).FunctionSupportInfo).IdlePower.set_D3ColdSupported(((*power_descriptor).D3ColdSupported) as u32);
                    (*(*fdo_extension).FunctionSupportInfo).IdlePower.set_NoVerifyDuringIdlePower(((*power_descriptor).NoVerifyDuringIdlePower) as u32);
                    ExFreePool(power_descriptor as *mut c_void);
                } else {
                    TracePrint(TRACE_LEVEL_ERROR, TRACE_FLAG_PNP, b"ClassPnpStartDevice: ClassGetDescriptor [DevicePower] failed %lx\n\0".as_ptr() as PCHAR, status);
                    status = STATUS_SUCCESS;
                }
            }
        }

        if NT_SUCCESS(status) {
            status = (*dev_info).ClassInitDevice.expect("ClassInitDevice")(device_object);
            {
                unsafe extern "C" { fn DbgPrint(Format: *const i8, ...) -> u32; }
                DbgPrint(
                    c"[CLASSPNP] ClassInitDevice(%p) -> 0x%x\n".as_ptr(),
                    device_object,
                    status as u32,
                );
            }
        }

        if (*common_extension).__bindgen_anon_1.IsFdo() != 0 {
            (*(*fdo_extension).PrivateFdoData).Perf.OriginalSrbFlags = (*fdo_extension).SrbFlags;

            if (*device_object).DeviceType == FILE_DEVICE_DISK &&
                TEST_FLAG((*device_object).Characteristics, FILE_FLOPPY_DISKETTE) == 0
            {
                let mut access_alignment_not_supported: ULONG = 0;
                let mut qerr_override_mode: ULONG = QERR_SET_ZERO_ODX_OR_TP_ONLY;
                let mut legacy_error_handling: BOOLEAN = FALSE;

                TracePrint(TRACE_LEVEL_INFORMATION, TRACE_FLAG_PNP,
                    b"ClassPnpStartDevice: Enabling idle timer for %p\n\0".as_ptr() as PCHAR, device_object);
                ClasspInitializeIdleTimer(fdo_extension);

                if ClasspIsObsoletePortDriver(fdo_extension) == FALSE {
                    ClasspGetInquiryVpdSupportInfo(fdo_extension);
                    ClasspGetLBProvisioningInfo(fdo_extension);

                    if (*(*fdo_extension).FunctionSupportInfo).ValidInquiryPages.BlockDeviceRODLimits() != 0 {
                        ClassDetermineTokenOperationCommandSupport(device_object);
                    }

                    ClassGetDeviceParameter(fdo_extension, crate::glue::regw::CLASSP_REG_SUBKEY_NAME.as_ptr() as PWSTR, crate::glue::regw::CLASSP_REG_QERR_OVERRIDE_MODE.as_ptr() as PWSTR, &raw mut qerr_override_mode);

                    if qerr_override_mode == QERR_SET_ZERO_ALWAYS ||
                        (qerr_override_mode == QERR_SET_ZERO_ODX_OR_TP_ONLY &&
                         (ClasspIsThinProvisioned((*fdo_extension).FunctionSupportInfo) != 0 ||
                          NT_SUCCESS(ClasspValidateOffloadSupported(device_object, core::ptr::null_mut()))))
                    {
                        ClasspZeroQERR(device_object);
                    }
                } else {
                    (*(*fdo_extension).FunctionSupportInfo).LBProvisioningData.CommandStatus = STATUS_UNSUCCESSFUL;
                    (*(*fdo_extension).FunctionSupportInfo).BlockLimitsData.CommandStatus = STATUS_UNSUCCESSFUL;
                    (*(*fdo_extension).FunctionSupportInfo).BlockDeviceRODLimitsData.CommandStatus = STATUS_UNSUCCESSFUL;
                }

                ClassGetDeviceParameter(fdo_extension, crate::glue::regw::CLASSP_REG_SUBKEY_NAME.as_ptr() as PWSTR, crate::glue::regw::CLASSP_REG_ACCESS_ALIGNMENT_NOT_SUPPORTED.as_ptr() as PWSTR, &raw mut access_alignment_not_supported);
                if access_alignment_not_supported > 0 {
                    (*(*fdo_extension).FunctionSupportInfo).RegAccessAlignmentQueryNotSupported = TRUE;
                }

                #[cfg(ntddi_ge_ntddi_winblue)]
                {
                    ClassGetDeviceParameter(fdo_extension, CLASSP_REG_SUBKEY_NAME, CLASSP_REG_LEGACY_ERROR_HANDLING, &raw mut legacy_error_handling);
                    if legacy_error_handling != 0 {
                        (*(*fdo_extension).PrivateFdoData).MaxNumberOfIoRetries = LEGACY_NUM_IO_RETRIES;
                        (*(*fdo_extension).PrivateFdoData).LegacyErrorHandling = TRUE;
                    }
                }

                ClasspGetCopyOffloadMaxDuration(
                    device_object,
                    REG_DISK_CLASS_CONTROL.as_ptr() as *const u16 as *mut u16,
                    &raw mut (*(*fdo_extension).PrivateFdoData).CopyOffloadMaxTargetDuration,
                );
            }
        }
    }

    if !NT_SUCCESS(status) {
        return status;
    } else {
        (*common_extension).__bindgen_anon_1.set_IsInitialized(TRUE);
    }

    if is_fdo != 0 &&
        ((*init_data).ClassTick.is_some() ||
         ((*fdo_extension).MediaChangeDetectionInfo != core::ptr::null_mut() &&
          (*fdo_extension).FunctionSupportInfo != core::ptr::null_mut() &&
          (*(*fdo_extension).FunctionSupportInfo).AsynchronousNotificationSupported == FALSE) ||
         ((*fdo_extension).FailurePredictionInfo != core::ptr::null_mut() &&
          (*(*fdo_extension).FailurePredictionInfo).Method != FailurePredictionNone))
    {
        ClasspEnableTimer(fdo_extension);
        if ScreenStateNotificationHandle == core::ptr::null_mut() {
            PoRegisterPowerSettingCallback(
                device_object,
                &raw const GUID_CONSOLE_DISPLAY_STATE,
                abi_cast!(ClasspPowerSettingCallback),
                core::ptr::null_mut(),
                &raw mut ScreenStateNotificationHandle,
            );
        }
    }

    status = (*dev_info).ClassStartDevice.expect("ClassStartDevice")(device_object);
    {
        unsafe extern "C" { fn DbgPrint(Format: *const i8, ...) -> u32; }
        DbgPrint(
            c"[CLASSPNP] ClassStartDevice(%p) -> 0x%x\n".as_ptr(),
            device_object,
            status as u32,
        );
    }
    if NT_SUCCESS(status) {
        (*common_extension).CurrentState = IRP_MN_START_DEVICE;

        if is_fdo != 0 && (*init_data).ClassEnumerateDevice.is_some() {
            is_mounted_device = FALSE;
        }

        if (*device_object).DeviceType != FILE_DEVICE_CD_ROM {
            is_mounted_device = FALSE;
        }

        if (*device_object).DeviceType == FILE_DEVICE_DISK &&
            TEST_FLAG((*device_object).Characteristics, FILE_FLOPPY_DISKETTE) != 0
        {
            is_mounted_device = TRUE;
        }

        if is_mounted_device != 0 {
            ClasspRegisterMountedDeviceInterface(device_object);
        }

        if (*common_extension).__bindgen_anon_1.IsFdo() != 0 {
            IoWMIRegistrationControl(device_object, WMIREG_ACTION_REGISTER);
            if (*fdo_extension).FunctionSupportInfo != core::ptr::null_mut() &&
                (*(*fdo_extension).FunctionSupportInfo).IdlePower.IdlePowerEnabled() == (FALSE) as u32 &&
                (*fdo_extension).MiniportDescriptor != core::ptr::null_mut() &&
                ((*(*fdo_extension).MiniportDescriptor).Portdriver == StoragePortCodeSetStorport ||
                 (*(*fdo_extension).MiniportDescriptor).Portdriver == StoragePortCodeSetSDport ||
                 (*(*fdo_extension).MiniportDescriptor).Portdriver == StoragePortCodeSetUSBport)
            {
                let mut disable_idle_power: ULONG = 0;
                ClassGetDeviceParameter(
                    fdo_extension,
                    crate::glue::regw::CLASSP_REG_SUBKEY_NAME.as_ptr() as PWSTR,
                    crate::glue::regw::CLASSP_REG_DISBALE_IDLE_POWER_NAME.as_ptr() as PWSTR,
                    &raw mut disable_idle_power,
                );
                if disable_idle_power == 0 {
                    ClasspEnableIdlePower(device_object);
                }
            }
        }
    } else {
        ClasspDisableTimer(fdo_extension);
    }

    status
}

#[no_mangle]
pub unsafe extern "system" fn ClassReadWrite(DeviceObject: *mut _DEVICE_OBJECT, Irp: *mut _IRP) -> NTSTATUS {
    let commonExtension = (*DeviceObject).DeviceExtension as *mut _COMMON_DEVICE_EXTENSION;
    let lowerDeviceObject = (*commonExtension).LowerDeviceObject;
    let currentIrpStack = IoGetCurrentIrpStackLocation(Irp);
    let transferByteCount = (*currentIrpStack).Parameters.Read.Length;
    let isRemoved: u32;
    let mut status: NTSTATUS;

    isRemoved = ClassAcquireRemoveLock(DeviceObject, (Irp as *mut c_void).cast());
    if isRemoved != 0 {
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_DEVICE_DOES_NOT_EXIST;
        (*Irp).IoStatus.Information = 0;
        ClassReleaseRemoveLock(DeviceObject, (Irp as *mut c_void).cast());
        ClassCompleteRequest(DeviceObject, Irp, (IO_NO_INCREMENT) as _);
        status = STATUS_DEVICE_DOES_NOT_EXIST;
    } else if ((*DeviceObject).Flags & DO_VERIFY_VOLUME) != 0
        && (*currentIrpStack).MinorFunction != (CLASSP_VOLUME_VERIFY_CHECKED) as u8
        && ((*currentIrpStack).Flags & (SL_OVERRIDE_VERIFY_VOLUME) as u8) == 0
    {
        if (*Irp).Tail.Overlay.Thread != core::ptr::null_mut() {
            IoSetHardErrorOrVerifyDevice(Irp, DeviceObject);
        }
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_VERIFY_REQUIRED;
        (*Irp).IoStatus.Information = 0;
        ClassReleaseRemoveLock(DeviceObject, (Irp as *mut c_void).cast());
        ClassCompleteRequest(DeviceObject, Irp, 0);
        status = STATUS_VERIFY_REQUIRED;
    } else {
        (*currentIrpStack).MinorFunction = (CLASSP_VOLUME_VERIFY_CHECKED) as u8;
        NT_ASSERT!((*(*commonExtension).DevInfo).ClassReadWriteVerification.is_some() as u8);
        status = (*(*commonExtension).DevInfo).ClassReadWriteVerification.expect("ClassReadWriteVerification")(DeviceObject, Irp);

        if !NT_SUCCESS(status) {
            NT_ASSERT!(((*Irp).IoStatus.__bindgen_anon_1.Status == status) as u8);
            (*Irp).IoStatus.Information = 0;
            ClassReleaseRemoveLock(DeviceObject, (Irp as *mut c_void).cast());
            ClassCompleteRequest(DeviceObject, Irp, (IO_NO_INCREMENT) as _);
        } else if status == STATUS_PENDING {
            // queued, nothing to do
        } else {
            status = STATUS_UNSUCCESSFUL;
            'exit: {
                if transferByteCount == 0 {
                    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
                    (*Irp).IoStatus.Information = 0;
                    ClassReleaseRemoveLock(DeviceObject, (Irp as *mut c_void).cast());
                    ClassCompleteRequest(DeviceObject, Irp, (IO_NO_INCREMENT) as _);
                    status = STATUS_SUCCESS;
                    break 'exit;
                }

                if (*(*commonExtension).DriverExtension).InitData.ClassStartIo.is_some() {
                    IoMarkIrpPending(Irp);
                    IoStartPacket(DeviceObject, Irp, core::ptr::null_mut(), None);
                    status = STATUS_PENDING;
                    break 'exit;
                }

                (*currentIrpStack).Parameters.Read.ByteOffset.QuadPart += (*commonExtension).StartingOffset.QuadPart;

                if (*commonExtension).__bindgen_anon_1.IsFdo() != 0 {
                    let fdoExtension = (*DeviceObject).DeviceExtension as *mut _FUNCTIONAL_DEVICE_EXTENSION;
                    let fdoData = (*fdoExtension).PrivateFdoData;

                    (*currentIrpStack).Parameters.Read.ByteOffset.QuadPart +=
                        ((*(*commonExtension).PartitionZeroExtension).DMByteSkew) as i64;

                    if ((*fdoExtension).DeviceFlags & (DEV_USE_16BYTE_CDB) as u16) == 0 {
                        let mut startingLba: LARGE_INTEGER = core::mem::zeroed();
                        startingLba.QuadPart = (*currentIrpStack).Parameters.Read.ByteOffset.QuadPart
                            >> (*fdoExtension).SectorShift;

                        if startingLba.QuadPart > MAXULONG as i64 {
                            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_INVALID_PARAMETER;
                            (*Irp).IoStatus.Information = 0;
                            ClassReleaseRemoveLock(DeviceObject, (Irp as *mut c_void).cast());
                            ClassCompleteRequest(DeviceObject, Irp, (IO_NO_INCREMENT) as _);
                            status = STATUS_INVALID_PARAMETER;
                            break 'exit;
                        }
                    }

                    #[cfg(debug_assertions)]
                    {
                        if (*fdoExtension).DevicePowerState == PowerDeviceD3
                            && ((*currentIrpStack).MajorFunction == IRP_MJ_READ
                                || ((*currentIrpStack).Flags & (SL_WRITE_THROUGH) as u8) != 0)
                        {
                            SnapDiskStartup();
                        }
                    }

                    if ClasspIsIdleRequestSupported(fdoData, Irp) != 0 {
                        ClasspMarkIrpAsIdle(Irp, 1u8);
                        status = ClasspEnqueueIdleRequest(DeviceObject, Irp);
                    } else {
                        let mut uniqueAddr: u8 = 0u8;

                        ClassAcquireRemoveLock(
                            DeviceObject,
                            (core::ptr::addr_of_mut!(uniqueAddr) as *mut c_void).cast(),
                        );
                        ClasspMarkIrpAsIdle(Irp, 0u8);
                        status = ServiceTransferRequest(DeviceObject, Irp, 0u8);

                        if (*fdoData).IdlePrioritySupported == 1u8 {
                            (*fdoData).LastNonIdleIoTime = ClasspGetCurrentTime();
                        }

                        ClassReleaseRemoveLock(
                            DeviceObject,
                            (core::ptr::addr_of_mut!(uniqueAddr) as *mut c_void).cast(),
                        );
                    }
                } else {
                    IoCopyCurrentIrpStackLocationToNext(Irp);
                    ClassReleaseRemoveLock(DeviceObject, (Irp as *mut c_void).cast());
                    status = IoCallDriver(lowerDeviceObject, Irp);
                }
            }
        }
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn InterpretCapacityData(Fdo: PDEVICE_OBJECT, ReadCapacityData: PREAD_CAPACITY_DATA_EX) {
    let fdoExt = (*Fdo).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let bytesPerSector: ULONG = ClasspCalculateLogicalSectorSize(Fdo, (*ReadCapacityData).BytesPerBlock);

    // WHICH_BIT: compute SectorShift = log2(bytesPerSector)
    {
        let mut v = bytesPerSector;
        let mut s: u32 = 0;
        while v >> 1 != 0 {
            v >>= 1;
            s += 1;
        }
        (*fdoExt).SectorShift = (s) as u8;
    }

    (*fdoExt).DiskGeometry.BytesPerSector = bytesPerSector;

    let mut lastSector: LARGE_INTEGER = core::mem::zeroed();
    // REVERSE_BYTES_QUAD: reverse bytes of LogicalBlockAddress (big-endian -> little-endian)
    lastSector.QuadPart = (*ReadCapacityData).LogicalBlockAddress.QuadPart.swap_bytes();

    if (*fdoExt).DMActive != 0u8 {
        TracePrint(TRACE_LEVEL_WARNING, TRACE_FLAG_INIT, b"ClassReadDriveCapacity: reducing number of sectors by %d\n\0".as_ptr() as PCHAR, (*fdoExt).DMSkew);
        lastSector.QuadPart -= (*fdoExt).DMSkew as i64;
    }

    let mut cylinderSize = (*fdoExt).DiskGeometry.TracksPerCylinder as u32 * (*fdoExt).DiskGeometry.SectorsPerTrack as u32;
    if cylinderSize == 0 {
        (*fdoExt).DiskGeometry.TracksPerCylinder = 0xff;
        (*fdoExt).DiskGeometry.SectorsPerTrack = 0x3f;
        cylinderSize = (*fdoExt).DiskGeometry.TracksPerCylinder as u32 * (*fdoExt).DiskGeometry.SectorsPerTrack as u32;
    }

    (*fdoExt).DiskGeometry.Cylinders.QuadPart = (lastSector.QuadPart + 1) / cylinderSize as i64;
    if (*fdoExt).DiskGeometry.Cylinders.QuadPart == 0i64 {
        (*fdoExt).DiskGeometry.SectorsPerTrack = 1;
        (*fdoExt).DiskGeometry.TracksPerCylinder = 1;
        (*fdoExt).DiskGeometry.Cylinders.QuadPart = lastSector.QuadPart + 1;
    }

    (*fdoExt).__bindgen_anon_1.CommonExtension.as_mut().PartitionLength.QuadPart = (lastSector.QuadPart + 1) << (*fdoExt).SectorShift as i64;

    if (*Fdo).Characteristics & FILE_REMOVABLE_MEDIA != 0 {
        (*fdoExt).DiskGeometry.MediaType = RemovableMedia;
    } else {
        (*fdoExt).DiskGeometry.MediaType = FixedMedia;
    }
}

#[no_mangle]
pub unsafe extern "system" fn ClassReadDriveCapacity(Fdo: PDEVICE_OBJECT) -> NTSTATUS {
    let fdoExt = (*Fdo).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let commonExtension = (*Fdo).DeviceExtension as PCOMMON_DEVICE_EXTENSION;
    let fdoData = (*fdoExt).PrivateFdoData;
    let mut readCapacityData: READ_CAPACITY_DATA_EX = core::mem::zeroed();
    let mut pkt: PTRANSFER_PACKET;
    let mut status: NTSTATUS;
    let mut driveCapMdl: PMDL = core::ptr::null_mut();
    let mut event: KEVENT = core::mem::zeroed();
    let mut pseudoIrp: IRP = core::mem::zeroed();
    let mut readCapacityDataSize: ULONG;
    let mut use16ByteCdb: BOOLEAN = TEST_FLAG(((*fdoExt).DeviceFlags) as u32, DEV_USE_16BYTE_CDB);
    let mut match_val: BOOLEAN = 1u8;

    'cleanup: loop {
        'retry: loop {
            if use16ByteCdb != 0 {
                readCapacityDataSize = core::mem::size_of::<READ_CAPACITY_DATA_EX>() as ULONG;
            } else {
                readCapacityDataSize = core::mem::size_of::<READ_CAPACITY_DATA>() as ULONG;
            }

            if !driveCapMdl.is_null() {
                FreeDeviceInputMdl(driveCapMdl);
                driveCapMdl = core::ptr::null_mut();
            }

            driveCapMdl = BuildDeviceInputMdl(
                &mut readCapacityData as *mut _ as *mut core::ffi::c_void,
                readCapacityDataSize,
            );
            if driveCapMdl.is_null() {
                status = STATUS_INSUFFICIENT_RESOURCES;
                break 'cleanup;
            }

            pkt = DequeueFreeTransferPacket(Fdo, 1u8);
            if pkt.is_null() {
                status = STATUS_INSUFFICIENT_RESOURCES;
                break 'cleanup;
            }

            pseudoIrp.Tail.Overlay.__bindgen_anon_1.__bindgen_anon_1.DriverContext[0] = LongToPtr(1);
            pseudoIrp.IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
            pseudoIrp.IoStatus.Information = 0;
            pseudoIrp.MdlAddress = driveCapMdl;

            KeInitializeEvent(&mut event, _EVENT_TYPE_SynchronizationEvent, 0u8);
            SetupDriveCapacityTransferPacket(
                pkt,
                &mut readCapacityData as *mut _ as *mut c_void,
                readCapacityDataSize,
                &mut event,
                &mut pseudoIrp,
                use16ByteCdb,
            );
            SubmitTransferPacket(pkt);
            _ = KeWaitForSingleObject(&mut event as *mut _ as PVOID, _KWAIT_REASON_Executive, 0i8, 0u8, core::ptr::null_mut());
            status = pseudoIrp.IoStatus.__bindgen_anon_1.Status;

            if NT_SUCCESS(status) && pseudoIrp.IoStatus.Information < (((readCapacityDataSize) as u64) as u32) as _ {
                TracePrint(
                    TRACE_LEVEL_ERROR,
                    TRACE_FLAG_INIT,
                    b"ClassReadDriveCapacity: read len (%xh) < %xh, retrying ...\0".as_ptr() as PCHAR,
                    pseudoIrp.IoStatus.Information as ULONG,
                    readCapacityDataSize,
                );
                pkt = DequeueFreeTransferPacket(Fdo, 1u8);
                if !pkt.is_null() {
                    pseudoIrp.Tail.Overlay.__bindgen_anon_1.__bindgen_anon_1.DriverContext[0] = LongToPtr(1);
                    pseudoIrp.IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
                    pseudoIrp.IoStatus.Information = 0;
                    KeInitializeEvent(&mut event, _EVENT_TYPE_SynchronizationEvent, 0u8);
                    SetupDriveCapacityTransferPacket(
                        pkt,
                        &mut readCapacityData as *mut _ as *mut c_void,
                        readCapacityDataSize,
                        &mut event,
                        &mut pseudoIrp,
                        use16ByteCdb,
                    );
                    SubmitTransferPacket(pkt);
                    _ = KeWaitForSingleObject(&mut event as *mut _ as PVOID, _KWAIT_REASON_Executive, 0i8, 0u8, core::ptr::null_mut());
                    status = pseudoIrp.IoStatus.__bindgen_anon_1.Status;
                    if pseudoIrp.IoStatus.Information < (((readCapacityDataSize) as u64) as u32) as _ {
                        status = STATUS_DEVICE_BUSY;
                    }
                } else {
                    status = STATUS_INSUFFICIENT_RESOURCES;
                }
            }

            break 'retry;
        }

        if NT_SUCCESS(status) {
            if use16ByteCdb == 0 {
                let readCapacity = &mut *((&mut readCapacityData) as *mut READ_CAPACITY_DATA_EX as *mut READ_CAPACITY_DATA);
                if (*readCapacity).LogicalBlockAddress == 0xFFFFFFFF {
                    use16ByteCdb = 1u8;
                    continue 'cleanup;
                } else {
                    core::ptr::copy_nonoverlapping(
                        readCapacity as *mut _ as *const u8,
                        (core::ptr::addr_of_mut!(readCapacityData) as *mut u8)
                            .add(core::mem::size_of::<ULONG>()),
                        core::mem::size_of::<READ_CAPACITY_DATA>(),
                    );
                    core::ptr::write_bytes(
                        core::ptr::addr_of_mut!(readCapacityData) as *mut u8,
                        0,
                        core::mem::size_of::<ULONG>(),
                    );
                }
            } else {
                (*fdoExt).DeviceFlags |= DEV_USE_16BYTE_CDB as u16;
            }

            InterpretCapacityData(Fdo, &mut readCapacityData);

            if (*fdoData).IsCachedDriveCapDataValid != 0 {
                match_val = RtlEqualMemory(
                    &(*fdoData).LastKnownDriveCapacityData as *const _ as *const core::ffi::c_void,
                    &readCapacityData as *const _ as *const core::ffi::c_void,
                    core::mem::size_of::<READ_CAPACITY_DATA_EX>(),
                );
            }

            (*fdoData).LastKnownDriveCapacityData = readCapacityData;
            (*fdoData).IsCachedDriveCapDataValid = 1u8;

            if match_val == 0 {
                if (*commonExtension).CurrentState != IRP_MN_START_DEVICE {
                } else {
                    if InterlockedCompareExchange(
                        &(*fdoData).UpdateDiskPropertiesWorkItemActive as *const _ as *mut LONG,
                        1,
                        0,
                    ) == 0
                    {
                        let workItem = IoAllocateWorkItem(Fdo);
                        if !workItem.is_null() {
                            IoQueueWorkItem(
                                workItem,
                                Some(ClasspUpdateDiskProperties),
                                DelayedWorkQueue,
                                workItem as *mut core::ffi::c_void,
                            );
                        } else {
                            InterlockedExchange(
                                &(*fdoData).UpdateDiskPropertiesWorkItemActive as *const _ as *mut LONG,
                                0,
                            );
                        }
                    }
                }
            }
        } else {
            TracePrint(
                TRACE_LEVEL_WARNING,
                TRACE_FLAG_INIT,
                b"ClassReadDriveCapacity on Fdo %p failed with status %ul.\0".as_ptr() as PCHAR,
                Fdo,
                status,
            );

            core::ptr::write_bytes(
                &mut (*fdoExt).DiskGeometry as *mut _ as *mut u8,
                0,
                core::mem::size_of::<DISK_GEOMETRY>(),
            );
            (*fdoExt).DiskGeometry.BytesPerSector = 512;
            (*fdoExt).SectorShift = 9;
            (*fdoExt).__bindgen_anon_1.CommonExtension.as_mut().PartitionLength.QuadPart = 0;

            if TEST_FLAG((*Fdo).Characteristics, FILE_REMOVABLE_MEDIA) != 0 {
                (*fdoExt).DiskGeometry.MediaType = RemovableMedia;
            } else {
                (*fdoExt).DiskGeometry.MediaType = FixedMedia;
            }
        }

        break 'cleanup;
    }

    if TEST_FLAG(((*fdoExt).DeviceFlags) as u32, DEV_USE_16BYTE_CDB) == 0 {
        let cdb10MaxBlocks = (u16::MAX as ULONG) << (*fdoExt).SectorShift;
        (*fdoData).HwMaxXferLen = min(cdb10MaxBlocks, (*fdoData).HwMaxXferLen);
    }

    if !driveCapMdl.is_null() {
        FreeDeviceInputMdl(driveCapMdl);
    }

    if !NT_SUCCESS(status) && (*fdoExt).DiskGeometry.MediaType == RemovableMedia {
        (*fdoData).IsCachedDriveCapDataValid = 0;
    }

    if status == STATUS_INSUFFICIENT_RESOURCES
        && (*fdoData).IsCachedDriveCapDataValid != 0
        && (*fdoExt).DiskGeometry.MediaType == FixedMedia
    {
        TracePrint(
            TRACE_LEVEL_WARNING,
            TRACE_FLAG_INIT,
            b"ClassReadDriveCapacity: defaulting to cached DriveCapacity data\0".as_ptr() as PCHAR,
        );
        InterpretCapacityData(Fdo, &raw mut (*fdoData).LastKnownDriveCapacityData);
        status = STATUS_SUCCESS;
    }

    status
}
#[no_mangle]
pub unsafe extern "system" fn ClassSendStartUnit(Fdo: *mut _DEVICE_OBJECT) {
    unsafe {
        let fdo_extension = (*Fdo).DeviceExtension as *mut _FUNCTIONAL_DEVICE_EXTENSION;
        let context = ExAllocatePoolWithTag(
            NonPagedPoolNx,
            core::mem::size_of::<COMPLETION_CONTEXT>(),
            0x36634353, // '6CcS'
        ) as *mut COMPLETION_CONTEXT;

        if context.is_null() {
            KeBugCheck(SCSI_DISK_DRIVER_INTERNAL);
        }

        (*context).DeviceObject = Fdo;

        let srb: *mut SCSI_REQUEST_BLOCK = (*context).Srb.Srb.as_mut() as *mut _;

        if (*((*fdo_extension).AdapterDescriptor as *mut crate::types::_STORAGE_ADAPTER_DESCRIPTOR)).SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK as u8 {
            let srbEx: *mut STORAGE_REQUEST_BLOCK = (*context).Srb.SrbEx.as_mut() as *mut _;
            // NB: SrbExBuffer is a __BindgenUnionField ZST marker — size_of_val on it is 0
            // (same class as the ReleaseQueueSrb bug in ClassPnpStartDevice). Size the whole
            // union instead (== the SrbExBuffer byte array, its largest member).
            let srbExBufSize = core::mem::size_of::<crate::types::_COMPLETION_CONTEXT__bindgen_ty_1>() as u32;

            let status = InitializeStorageRequestBlock(
                srbEx,
                STORAGE_ADDRESS_TYPE_BTL8 as u16,
                srbExBufSize,
                1,
                SrbExDataTypeScsiCdb16,
            );

            if !NT_SUCCESS(status) {
                FREE_POOL(context as *mut c_void);
                NT_ASSERT!(0u8);
                return;
            }

            (*srbEx).SrbFunction = SRB_FUNCTION_EXECUTE_SCSI;
        } else {
            core::ptr::write_bytes(srb as *mut u8, 0u8, core::mem::size_of::<SCSI_REQUEST_BLOCK>());
            (*srb).Length = (core::mem::size_of::<SCSI_REQUEST_BLOCK>() as u8) as u16;
            (*srb).Function = (SRB_FUNCTION_EXECUTE_SCSI) as u8;
        }

        SrbSetTimeOutValue((srb).cast(), START_UNIT_TIMEOUT);

        SrbAssignSrbFlags(
            (srb).cast(),
            SRB_FLAGS_NO_DATA_TRANSFER | SRB_FLAGS_DISABLE_AUTOSENSE | SRB_FLAGS_DISABLE_SYNCH_TRANSFER,
        );

        SrbSetCdbLength((srb).cast(), 6);
        let cdb = SrbGetCdb((srb).cast());

        {
            let ss = cdb as *mut _CDB__START_STOP;
            (*ss).OperationCode = SCSIOP_START_STOP_UNIT as u8;
            _CDB__START_STOP::set_Start_raw(ss, 1);
            _CDB__START_STOP::set_Immediate_raw(ss, 0);
            _CDB__START_STOP::set_LogicalUnitNumber_raw(ss, (*srb).Lun);
        }

        let irp = IoAllocateIrp((*Fdo).StackSize as i8, 0u8);
        if irp.is_null() {
            KeBugCheck(SCSI_DISK_DRIVER_INTERNAL);
        }

        ClassAcquireRemoveLock(Fdo, irp);

        IoSetCompletionRoutine(
            irp,
            Some(core::mem::transmute::<unsafe extern "system" fn(*mut _DEVICE_OBJECT, *mut _IRP, *mut c_void) -> i32, _>(ClassAsynchronousCompletion)),
            context as *mut c_void,
            1u8,
            1u8,
            1u8,
        );

        let irpStack = IoGetNextIrpStackLocation(irp);
        (*irpStack).MajorFunction = (IRP_MJ_SCSI) as u8;
        SrbSetOriginalRequest((srb).cast(), irp);
        (*irpStack).Parameters.Scsi.Srb = srb as *mut _;

        IoCallDriver((*fdo_extension).__bindgen_anon_1.CommonExtension.as_ref().LowerDeviceObject, irp);
    }
}
#[no_mangle]
pub unsafe extern "system" fn ClassAsynchronousCompletion(
    mut DeviceObject: *mut _DEVICE_OBJECT,
    Irp: *mut _IRP,
    Context: *mut core::ffi::c_void,
) -> i32 {
    let context = Context as *mut _COMPLETION_CONTEXT;

    if context.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    if DeviceObject.is_null() {
        DeviceObject = (*context).DeviceObject;
    }

    let srb: *mut _SCSI_REQUEST_BLOCK = (*context).Srb.Srb.as_mut() as *mut _;
    let srbFunction: u32;
    let srbFlags: u32;

    if (*srb).Function == (SRB_FUNCTION_STORAGE_REQUEST_BLOCK) as u8 {
        let storage_srb = srb as *mut _STORAGE_REQUEST_BLOCK;
        srbFunction = (*storage_srb).SrbFunction;
        srbFlags = (*storage_srb).SrbFlags;
    } else {
        srbFunction = ((*srb).Function) as u32;
        srbFlags = (*srb).SrbFlags;
    }

    if srbFunction == SRB_FUNCTION_EXECUTE_SCSI {
        if ((*srb).SrbStatus & (SRB_STATUS_QUEUE_FROZEN) as u8) != 0 {
            ClassReleaseQueue((*context).DeviceObject);
        }
    }

    {
        let commonExtension = (*DeviceObject).DeviceExtension as *mut _COMMON_DEVICE_EXTENSION;
        if (*commonExtension).__bindgen_anon_1.IsFdo() != 0 {
            let fdoExtension = (*DeviceObject).DeviceExtension as *mut _FUNCTIONAL_DEVICE_EXTENSION;
            if PORT_ALLOCATED_SENSE(fdoExtension, srb) != 0 {
                FREE_PORT_ALLOCATED_SENSE_BUFFER(fdoExtension, srb);
            }
        } else {
            ntkrnl_sys::ASSERT!(TEST_FLAG(srbFlags, SRB_FLAGS_FREE_SENSE_BUFFER) == 0);
        }
    }

    if !(*Irp).MdlAddress.is_null() {
        MmUnlockPages((*Irp).MdlAddress);
        IoFreeMdl((*Irp).MdlAddress);
        (*Irp).MdlAddress = core::ptr::null_mut();
    }

    ClassReleaseRemoveLock(DeviceObject, Irp);
    FREE_POOL((context).cast());
    IoFreeIrp(Irp);

    STATUS_MORE_PROCESSING_REQUIRED
}
#[no_mangle]
pub unsafe extern "C" fn ServiceTransferRequest(Fdo: *mut _DEVICE_OBJECT, Irp: *mut _IRP, PostToDpc: u8) -> NTSTATUS {
    let fdoExt = (*Fdo).DeviceExtension as *mut _FUNCTIONAL_DEVICE_EXTENSION;
    let commonExtension = (*Fdo).DeviceExtension as *mut _COMMON_DEVICE_EXTENSION;
    let adapterDesc = (*(*commonExtension).PartitionZeroExtension).AdapterDescriptor;
    let fdoData = (*fdoExt).PrivateFdoData;
    let priority: _IO_PAGING_PRIORITY = if ((*Irp).Flags & IRP_PAGING_IO) != 0 {
        IoGetPagingIoPriority(Irp)
    } else {
        _IO_PAGING_PRIORITY_IoPagingPriorityInvalid
    };
    let mut deferClientIrp: u8 = 0;
    let driverUsesStartIO = (*(*commonExtension).DriverExtension).InitData.ClassStartIo.is_some();
    let mut oldIrql: KIRQL = 0;
    let mut status: NTSTATUS = STATUS_UNSUCCESSFUL;
    let currentIrpStack = IoGetCurrentIrpStackLocation(Irp);

    if (*currentIrpStack).MajorFunction == IRP_MJ_READ
        && ((*currentIrpStack).Flags & (SL_KEY_SPECIFIED) as u8) != 0
        && IsKeyReadCopyNumber((*currentIrpStack).Parameters.Read.Key) != 0
    {
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_FT_READ_FROM_COPY;
    } else {
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    }

    if priority == _IO_PAGING_PRIORITY_IoPagingPriorityHigh {
        KeAcquireSpinLock((core::ptr::addr_of_mut!((*fdoData).SpinLock)).cast(), &mut oldIrql);
        if (*fdoData).NumHighPriorityPagingIo == 0 {
            KeQuerySystemTime(core::ptr::addr_of_mut!((*fdoData).ThrottleStartTime));
        }
        (*fdoData).NumHighPriorityPagingIo += 1;
        (*fdoData).MaxInterleavedNormalIo += ClassMaxInterleavePerCriticalIo;
        KeReleaseSpinLock((core::ptr::addr_of_mut!((*fdoData).SpinLock)).cast(), oldIrql);
    } else {
        if (*fdoData).NumHighPriorityPagingIo != 0 {
            KeAcquireSpinLock((core::ptr::addr_of_mut!((*fdoData).SpinLock)).cast(), &mut oldIrql);
            if (*fdoData).NumHighPriorityPagingIo != 0 {
                if (*fdoData).MaxInterleavedNormalIo == 0 {
                    deferClientIrp = 1;
                } else {
                    (*fdoData).MaxInterleavedNormalIo -= 1;
                }
            }
            KeReleaseSpinLock((core::ptr::addr_of_mut!((*fdoData).SpinLock)).cast(), oldIrql);
        }
    }

    if deferClientIrp == 0 {
        let currentSp = IoGetCurrentIrpStackLocation(Irp);
        let mut entireXferLen = (*currentSp).Parameters.Read.Length;
        let mut bufPtr = MmGetMdlVirtualAddress((*Irp).MdlAddress) as *mut u8;
        let mut targetLocation = (*currentSp).Parameters.Read.ByteOffset;
        let mut pkt: *mut TRANSFER_PACKET;
        let mut pktList: SINGLE_LIST_ENTRY = core::mem::zeroed();
        let mut hwMaxXferLen: u32;
        let mut numPackets: u32;
        let mut i: u32;

        if ((bufPtr as usize) & (PAGE_SIZE - 1)) != 0
            || (*fdoData).HwMaxXferLen > 0xffffffffu32 - PAGE_SIZE as u32
        {
            hwMaxXferLen = (*fdoData).HwMaxXferLen;
        } else {
            NT_ASSERT!(((PAGE_SIZE % ((*fdoExt).DiskGeometry.BytesPerSector) as usize) == 0) as u8);
            hwMaxXferLen = (*fdoData).HwMaxXferLen + PAGE_SIZE as u32;
            if hwMaxXferLen > (*adapterDesc).MaximumTransferLength {
                hwMaxXferLen = (*adapterDesc).MaximumTransferLength;
            }
        }

        NT_ASSERT!((hwMaxXferLen >= PAGE_SIZE as u32) as u8);
        numPackets = entireXferLen / hwMaxXferLen;
        if entireXferLen % hwMaxXferLen != 0 {
            numPackets += 1;
        }

        SimpleInitSlistHdr(core::ptr::addr_of_mut!(pktList));

        if driverUsesStartIO {
            pkt = DequeueFreeTransferPacket(Fdo, 1u8);
            if !pkt.is_null() {
                SimplePushSlist(
                    core::ptr::addr_of_mut!(pktList) as *mut _,
                    core::ptr::addr_of_mut!((*pkt).SlistEntry) as *mut _,
                );
                i = 1;
            } else {
                i = 0;
            }
        } else {
            i = 0;
            while i < numPackets {
                pkt = DequeueFreeTransferPacket(Fdo, 1u8);
                if !pkt.is_null() {
                    SimplePushSlist(
                        core::ptr::addr_of_mut!(pktList) as *mut _,
                        core::ptr::addr_of_mut!((*pkt).SlistEntry) as *mut _,
                    );
                } else {
                    break;
                }
                i += 1;
            }
        }

        if i == numPackets && !driverUsesStartIO {
            (*Irp).IoStatus.Information = 0;
            (*Irp).Tail.Overlay.__bindgen_anon_1.__bindgen_anon_1.DriverContext[0] = numPackets as *mut core::ffi::c_void;

            if numPackets > 1 {
                IoMarkIrpPending(Irp);
                status = STATUS_PENDING;
            } else {
                status = STATUS_SUCCESS;
            }

            while entireXferLen > 0 {
                let thisPieceLen = if hwMaxXferLen < entireXferLen {
                    hwMaxXferLen
                } else {
                    entireXferLen
                };

                let slistEntry = SimplePopSlist(core::ptr::addr_of_mut!(pktList));
                NT_ASSERT!((!slistEntry.is_null()) as u8);
                pkt = (slistEntry as *mut u8).wrapping_offset(
                    -(core::mem::offset_of!(TRANSFER_PACKET, SlistEntry) as isize),
                ) as *mut TRANSFER_PACKET;

                SetupReadWriteTransferPacket(pkt, bufPtr as *mut c_void, thisPieceLen, targetLocation, Irp);

                if numPackets > 1 {
                    (*pkt).UsePartialMdl = 1u8;
                }

                if PostToDpc != 0 {
                    (*pkt).RetryIn100nsUnits = 0;
                    TransferPacketQueueRetryDpc(pkt);
                    status = STATUS_PENDING;
                } else {
                    let pktStat = SubmitTransferPacket(pkt);
                    if pktStat != STATUS_SUCCESS {
                        status = STATUS_PENDING;
                    }
                }

                entireXferLen -= thisPieceLen;
                bufPtr = bufPtr.add(thisPieceLen as usize);
                targetLocation.QuadPart += thisPieceLen as i64;
            }
            NT_ASSERT!(SimpleIsSlistEmpty(core::ptr::addr_of_mut!(pktList)));
        } else if i >= 1 {
            while i > 1 {
                let slistEntry = SimplePopSlist(core::ptr::addr_of_mut!(pktList));
                NT_ASSERT!((!slistEntry.is_null()) as u8);
                pkt = (slistEntry as *mut u8).wrapping_offset(
                    -(core::mem::offset_of!(TRANSFER_PACKET, SlistEntry) as isize),
                ) as *mut TRANSFER_PACKET;
                EnqueueFreeTransferPacket(Fdo, pkt);
                i -= 1;
            }
            let slistEntry = SimplePopSlist(core::ptr::addr_of_mut!(pktList));
            NT_ASSERT!((!slistEntry.is_null()) as u8);
            NT_ASSERT!(SimpleIsSlistEmpty(core::ptr::addr_of_mut!(pktList)));
            pkt = (slistEntry as *mut u8).wrapping_offset(
                -(core::mem::offset_of!(TRANSFER_PACKET, SlistEntry) as isize),
            ) as *mut TRANSFER_PACKET;

            if !driverUsesStartIO {
                TracePrint(
                    TRACE_LEVEL_WARNING,
                    TRACE_FLAG_RW,
                    b"Insufficient packets available in ServiceTransferRequest - entering lowMemRetry with pkt=%p.\0"
                        .as_ptr() as *const i8,
                    pkt,
                );
            }

            (*Irp).IoStatus.Information = 0;
            (*Irp).Tail.Overlay.__bindgen_anon_1.__bindgen_anon_1.DriverContext[0] = 1usize as *mut core::ffi::c_void;
            IoMarkIrpPending(Irp);

            SetupReadWriteTransferPacket(pkt, bufPtr as *mut c_void, entireXferLen, targetLocation, Irp);
            InitLowMemRetry(pkt, bufPtr as *mut c_void, entireXferLen, targetLocation);
            StepLowMemRetry(pkt);
            status = STATUS_PENDING;
        } else {
            TracePrint(
                TRACE_LEVEL_WARNING,
                TRACE_FLAG_RW,
                b"No packets available in ServiceTransferRequest - deferring transfer (Irp=%p)...\0"
                    .as_ptr() as *const i8,
                Irp,
            );

            if priority == _IO_PAGING_PRIORITY_IoPagingPriorityHigh {
                KeAcquireSpinLock((core::ptr::addr_of_mut!((*fdoData).SpinLock)).cast(), &mut oldIrql);
                if (*fdoData).MaxInterleavedNormalIo < ClassMaxInterleavePerCriticalIo {
                    (*fdoData).MaxInterleavedNormalIo = 0;
                } else {
                    (*fdoData).MaxInterleavedNormalIo -= ClassMaxInterleavePerCriticalIo;
                }
                (*fdoData).NumHighPriorityPagingIo -= 1;
                if (*fdoData).NumHighPriorityPagingIo == 0 {
                    KeQuerySystemTime(core::ptr::addr_of_mut!((*fdoData).ThrottleStopTime));
                    let mut period: LARGE_INTEGER = core::mem::zeroed();
                    period.QuadPart =
                        (*fdoData).ThrottleStopTime.QuadPart - (*fdoData).ThrottleStartTime.QuadPart;
                    (*fdoData).LongestThrottlePeriod.QuadPart = core::cmp::max(
                        (*fdoData).LongestThrottlePeriod.QuadPart,
                        period.QuadPart,
                    );
                }
                KeReleaseSpinLock((core::ptr::addr_of_mut!((*fdoData).SpinLock)).cast(), oldIrql);
            }
            deferClientIrp = 1;
        }
    }

    if deferClientIrp != 0 {
        IoMarkIrpPending(Irp);
        EnqueueDeferredClientIrp(Fdo, Irp);
        status = STATUS_PENDING;
    }

    NT_ASSERT!((status != STATUS_UNSUCCESSFUL) as u8);
    status
}

#[no_mangle]
pub unsafe extern "system" fn ClassIoComplete(
    Fdo: PDEVICE_OBJECT,
    Irp: PIRP,
    Context: PVOID,
) -> NTSTATUS {
    // Forward declarations for unknown functions/macros

    // Constants
    const SRB_FUNCTION_STORAGE_REQUEST_BLOCK: UCHAR = 0x28; // FIXED 2026-07-05: was LLM-guessed "typical" value; verified vs C header (srb.h)
    const SRB_STATUS_SUCCESS: UCHAR = 0x01;
    const SRB_STATUS_QUEUE_FROZEN: UCHAR = 0x40;
    const SRB_CLASS_FLAGS_FREE_MDL: ULONG = 0x80000000;
    const SRB_CLASS_FLAGS_PERSISTANT: ULONG = 0x20000000;
    const SRB_FLAGS_DONT_START_NEXT_PACKET: ULONG = 0x00800000;
    const SL_OVERRIDE_VERIFY_VOLUME: ULONG = 0x00000002; // guessed
    const IRP_PAGING_IO: ULONG = 0x00000002; // guessed
    const IOCTL_STORAGE_FIRMWARE_ACTIVATE: ULONG = 0x002DDC08; // typical
    const IO_WARNING_DISK_FIRMWARE_UPDATED: ULONG = 0x4004009F;  // ntiologc.h
    const STATUS_MORE_PROCESSING_REQUIRED: NTSTATUS = 0xC0000016u32 as i32;
    const STATUS_SUCCESS: NTSTATUS = 0;
    const STATUS_DATA_OVERRUN: NTSTATUS = 0xC000003C_u32 as i32;
    const STATUS_IO_DEVICE_ERROR: NTSTATUS = 0xC0000185u32 as i32;
    const STATUS_VERIFY_REQUIRED: NTSTATUS = 0x80000016_u32 as i32;
    const MAXIMUM_RETRIES: u32 = 4; // FIXED 2026-07-05: was LLM-guessed "typical" value; verified vs C header (classpnp.h)
    const FALSE: BOOLEAN = 0;
    const TRUE: BOOLEAN = 1;
    const DISPATCH_LEVEL: KIRQL = 2;

    let irpStack = IoGetCurrentIrpStackLocation(Irp);
    let srb = Context as PSCSI_REQUEST_BLOCK;
    let fdoExtension = (*Fdo).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let fdoData = (*fdoExtension).PrivateFdoData;
    let mut status: NTSTATUS = STATUS_SUCCESS;
    let mut retry: BOOLEAN;
    let mut callStartNextPacket: BOOLEAN;
    let srbFlags: ULONG;
    let srbFunction: ULONG;

    ntkrnl_sys::ASSERT!((*fdoExtension).__bindgen_anon_1.CommonExtension.as_ref().__bindgen_anon_1.IsFdo() != 0);

    if (*srb).Function == SRB_FUNCTION_STORAGE_REQUEST_BLOCK {
        srbFlags = (*(srb as *mut STORAGE_REQUEST_BLOCK)).SrbFlags;
        srbFunction = (*(srb as *mut STORAGE_REQUEST_BLOCK)).SrbFunction;
    } else {
        srbFlags = (*srb).SrbFlags;
        srbFunction = ((*srb).Function) as u32;
    }

    #[cfg(debug_assertions)]
    {
        const SRB_FUNCTION_FLUSH: ULONG = 0x08; // typical
        if srbFunction == SRB_FUNCTION_FLUSH {
            DBGLOGFLUSHINFO!(fdoData, FALSE, FALSE, TRUE);
        }
    }

    // Check SRB status for success
    if ((*srb).SrbStatus & 0x0F) != SRB_STATUS_SUCCESS {
        let mut retryInterval: LONGLONG = 0;
        TracePrint(4, 0x00000001, b"ClassIoComplete: IRP %p, SRB %p\n\0" as *const u8 as *const i8, Irp, srb);

        if (*srb).SrbStatus & SRB_STATUS_QUEUE_FROZEN != 0 {
            ClassReleaseQueue(Fdo);
        }

        let ioControlCode = if (*irpStack).MajorFunction == IRP_MJ_DEVICE_CONTROL {
            (*irpStack).Parameters.DeviceIoControl.IoControlCode
        } else {
            0
        };

        let remainingRetries = MAXIMUM_RETRIES - ((*irpStack).Parameters.Others.Argument4 as usize as u32);

        retry = InterpretSenseInfoWithoutHistory(
            Fdo,
            Irp,
            srb,
            (*irpStack).MajorFunction,
            ioControlCode,
            remainingRetries,
            &mut status,
            &mut retryInterval,
        );

        // For Persistent Reserve requests, ensure partial data is returned
        if (*irpStack).MajorFunction == IRP_MJ_DEVICE_CONTROL
            && ((*irpStack).Parameters.DeviceIoControl.IoControlCode == IOCTL_STORAGE_PERSISTENT_RESERVE_IN
                || (*irpStack).Parameters.DeviceIoControl.IoControlCode == IOCTL_STORAGE_PERSISTENT_RESERVE_OUT)
            && status == STATUS_DATA_OVERRUN
        {
            status = STATUS_SUCCESS;
            retry = FALSE;
        }

        // If override verify volume flag set and status is VERIFY_REQUIRED, retry
        if TEST_FLAG(((*irpStack).Flags) as u32, SL_OVERRIDE_VERIFY_VOLUME) != 0
            && status == STATUS_VERIFY_REQUIRED
        {
            status = STATUS_IO_DEVICE_ERROR;
            retry = TRUE;
        }

        // Retry logic
        if retry != 0 {
            let arg4_ptr = core::ptr::addr_of_mut!((*irpStack).Parameters.Others.Argument4);
            let count = *arg4_ptr as usize as u32;
            if count > 0 {
                *arg4_ptr = (count - 1) as PVOID;

                TracePrint(4, 0x00000001, b"Retry request %p\n\0" as *const u8 as *const i8, Irp);

                if PORT_ALLOCATED_SENSE(fdoExtension, srb) != 0 {
                    FREE_PORT_ALLOCATED_SENSE_BUFFER(fdoExtension, srb);
                }

                RetryRequest(Fdo, Irp, srb, FALSE, retryInterval);
                return STATUS_MORE_PROCESSING_REQUIRED;
            }
        }
    } else {
        // Successful request
        (*fdoData).LoggedTURFailureSinceLastIO = FALSE;
        ClasspPerfIncrementSuccessfulIo(fdoExtension);
        status = STATUS_SUCCESS;
    }

    // Paging I/O sanity checks
    if NT_SUCCESS(status) && TEST_FLAG((*Irp).Flags, IRP_PAGING_IO) != 0 {
        ntkrnl_sys::ASSERT!((*Irp).IoStatus.Information != 0);
        ntkrnl_sys::ASSERT!((*irpStack).Parameters.Read.Length == ((*Irp).IoStatus.Information) as u32);
    }

    // Determine whether to call IoStartNextPacket
    callStartNextPacket = if TEST_FLAG(srbFlags, SRB_FLAGS_DONT_START_NEXT_PACKET) == 0 {
        TRUE
    } else {
        FALSE
    };
    if (*irpStack).MajorFunction == IRP_MJ_DEVICE_CONTROL {
        callStartNextPacket = FALSE;
    }

    // Free MDL if allocated
    if TEST_FLAG(srbFlags, SRB_CLASS_FLAGS_FREE_MDL) != 0 {
        SrbClearSrbFlags(srb, SRB_CLASS_FLAGS_FREE_MDL);
        IoFreeMdl((*Irp).MdlAddress);
        (*Irp).MdlAddress = core::ptr::null_mut();
    }

    // Free the SRB
    if TEST_FLAG(srbFlags, SRB_CLASS_FLAGS_PERSISTANT) == 0 {
        if PORT_ALLOCATED_SENSE(fdoExtension, srb) != 0 {
            FREE_PORT_ALLOCATED_SENSE_BUFFER(fdoExtension, srb);
        }

        if (*fdoExtension).__bindgen_anon_1.CommonExtension.as_ref().__bindgen_anon_1.IsSrbLookasideListInitialized() != 0 {
            ClassFreeOrReuseSrb(fdoExtension, srb);
        } else {
            TracePrint(4, 0x00000001, b"ClassIoComplete is freeing an SRB (possibly) on behalf of another driver.\0" as *const u8 as *const i8);
            FREE_POOL(srb as PVOID);
        }
    } else {
        TracePrint(4, 0x00000001,
            b"ClassIoComplete: Not Freeing srb @ %p because SRB_CLASS_FLAGS_PERSISTANT set\n\0" as *const u8 as *const i8,
            srb);
        if PORT_ALLOCATED_SENSE(fdoExtension, srb) != 0 {
            TracePrint(4, 0x00000001,
                b"ClassIoComplete: Not Freeing sensebuffer @ %p because SRB_CLASS_FLAGS_PERSISTANT set\n\0" as *const u8 as *const i8,
                (*srb).SenseInfoBuffer);
        }
    }

    // Set status in completing IRP
    (*Irp).IoStatus.__bindgen_anon_1.Status = status;

    // Set hard error if necessary
    if !NT_SUCCESS(status)
        && IoIsErrorUserInduced(status) != 0
        && !(*Irp).Tail.Overlay.Thread.is_null()
    {
        IoSetHardErrorOrVerifyDevice(Irp, Fdo);
        (*Irp).IoStatus.Information = 0;
    }

    // Log firmware update success
    if NT_SUCCESS(status)
        && (*irpStack).MajorFunction == IRP_MJ_DEVICE_CONTROL
        && (*irpStack).Parameters.DeviceIoControl.IoControlCode == IOCTL_STORAGE_FIRMWARE_ACTIVATE
    {
        ClasspLogSystemEventWithDeviceNumber(Fdo, IO_WARNING_DISK_FIRMWARE_UPDATED);
    }

    // If pending returned, mark pending
    if (*Irp).PendingReturned != 0 {
        IoMarkIrpPending(Irp);
    }

    if !(*fdoExtension).__bindgen_anon_1.CommonExtension.as_ref().DriverExtension.is_null()
        && !(*(*fdoExtension).__bindgen_anon_1.CommonExtension.as_ref().DriverExtension).InitData.ClassStartIo.is_none()
    {
        if callStartNextPacket != 0 {
            let mut oldIrql: KIRQL = 0;
            KeRaiseIrql(DISPATCH_LEVEL, &mut oldIrql);
            IoStartNextPacket(Fdo, TRUE);
            KeLowerIrql(oldIrql);
        }
    }

    ClassReleaseRemoveLock(Fdo, Irp);

    status
}



// ---- ABI-corrected extern decls (CC matched to in-crate definition) ----
extern "C" {
    
    
    

}
unsafe extern "system" {
    pub fn KeRaiseIrql(NewIrql: KIRQL, OldIrql: *mut KIRQL);
    pub fn KeLowerIrql(OldIrql: KIRQL);
}
// Interlocked* are FASTCALL on i386 per ntoskrnl.spec; plain C elsewhere
#[cfg(target_arch = "x86")]
extern "fastcall" {
    pub fn InterlockedCompareExchange(Destination: *mut LONG, ExChange: LONG, Comparand: LONG) -> LONG;
    pub fn InterlockedExchange(Destination: *mut LONG, Value: LONG) -> LONG;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    pub fn InterlockedCompareExchange(Destination: *mut LONG, ExChange: LONG, Comparand: LONG) -> LONG;
    pub fn InterlockedExchange(Destination: *mut LONG, Value: LONG) -> LONG;
}
extern "system" {
    fn ClassReleaseQueue(Fdo: PDEVICE_OBJECT);
    fn ClassReleaseRemoveLock(Fdo: PDEVICE_OBJECT, Irp: PIRP);
}


// ---- export decls forced to stdcall (real kernel export CC) ----
extern "system" {
    pub fn IoFreeMdl(Mdl: PMDL);
    pub fn IoSetHardErrorOrVerifyDevice(Irp: PIRP, Fdo: PDEVICE_OBJECT);
    pub fn IoStartNextPacket(Fdo: PDEVICE_OBJECT, Cancellable: BOOLEAN);
    pub fn KeQueryTickCount(CurrentCount: *mut i64);
}
