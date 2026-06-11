use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{BusRelations, DPFLTR_CLASSPNP_ID, ExFreePool, _KWAIT_REASON_Executive, GUID, IO_COMPLETION_ROUTINE, _IO_STATUS_BLOCK, IRP, IoFreeIrp, IoGetDriverObjectExtension, IoGetNextIrpStackLocation, IoMarkIrpPending, IoRegisterDeviceInterface, IoSetCompletionRoutine, IoSetDeviceInterfaceState, KEVENT, KLOCK_QUEUE_HANDLE, KeAcquireInStackQueuedSpinLock, KeGetRecommendedSharedDataAlignment, KeInitializeEvent, KeInitializeSpinLock, KeReleaseInStackQueuedSpinLock, KeWaitForSingleObject, LARGE_INTEGER, LIST_ENTRY, PDEVICE_OBJECT, PDEVICE_RELATIONS, PIO_STACK_LOCATION, PIO_WORKITEM, PIRP, PMDL, PUNICODE_STRING, RtlFreeUnicodeString, RtlInitUnicodeString, _EVENT_TYPE_SynchronizationEvent};
use core::ffi::c_void;

extern "C" {
    fn ASSERT_PDO(Pdo: PDEVICE_OBJECT);
    fn NT_ASSERT(condition: bool);
}
extern "C" {
    fn DbgPrintEx(ComponentId: ULONG, Level: ULONG, Format: PCSTR, ...) -> NTSTATUS;
    fn PoUnregisterPowerSettingCallback(Handle: PVOID) -> NTSTATUS;
    fn BuildDeviceInputMdl(buffer: PCHAR, length: ULONG) -> PMDL;
    fn FreeDeviceInputMdl(mdl: PMDL);
    fn DequeueFreeTransferPacket(fdo: PDEVICE_OBJECT, allocate: BOOLEAN) -> *mut TRANSFER_PACKET;
    fn SetupModeSelectTransferPacket( pkt: *mut TRANSFER_PACKET, event: *mut KEVENT, buffer: PCHAR, length: UCHAR, save_pages: BOOLEAN, irp: PIRP );
    fn SubmitTransferPacket(pkt: *mut TRANSFER_PACKET);
    fn LongToPtr(val: LONG) -> PVOID;
    fn TracePrint(level: ULONG, flags: ULONG, format: PCSTR, ...);
    fn ClassAcquireChildLock(Parent: PFUNCTIONAL_DEVICE_EXTENSION);
    fn ClassReleaseChildLock(Parent: PFUNCTIONAL_DEVICE_EXTENSION);
    fn ClasspRetryDpcTimer(fdoData: *mut CLASS_PRIVATE_FDO_DATA);
    fn FREE_POOL(ptr: PVOID);
}

// ---- file-scope preamble (translated) ----
pub static ClassPnpAllowUnload: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(1);

pub static ClassMaxInterleavePerCriticalIo: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(CLASS_MAX_INTERLEAVE_PER_CRITICAL_IO);

pub static Magic10000: LARGE_INTEGER = LARGE_INTEGER {
    QuadPart: 0xd1b71758e219652c,
};

pub static mut StoragePredictFailureDPSGuid: GUID = GUID {
    Data1: 0,
    Data2: 0,
    Data3: 0,
    Data4: [0; 8],
};

pub const FirstDriveLetter: u8 = b'C';
pub const LastDriveLetter: u8 = b'Z';

pub static UseQPCTime: core::sync::atomic::AtomicU8 = core::sync::atomic::AtomicU8::new(0);
pub static InitSecurityCookie: core::sync::atomic::AtomicU8 = core::sync::atomic::AtomicU8::new(0);

pub static MaxTokenOperationListIdentifier: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(MAX_TOKEN_LIST_IDENTIFIERS);

pub static TokenOperationListIdentifier: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0xFFFFFFFF);

pub static mut IdlePowerFDOList: LIST_ENTRY = LIST_ENTRY {
    Flink: core::ptr::null_mut(),
    Blink: core::ptr::null_mut(),
};

pub static mut IdlePowerFDOListMutex: KGUARDED_MUTEX = KGUARDED_MUTEX {
    Count: 0,
    Owner: core::ptr::null_mut(),
    Contention: 0,
    Gate: KGATE {
        Header: DISPATCHER_HEADER {
            __bindgen_anon_1: DISPATCHER_HEADER__bindgen_ty_1 {
                __bindgen_anon_1: DISPATCHER_HEADER__bindgen_ty_1__bindgen_ty_1 {
                    Type: 0,
                    Signalling: 0,
                    Size: 0,
                    Reserved1: 0,
                },
            },
            SignalState: 0,
            WaitListHead: LIST_ENTRY {
                Flink: core::ptr::null_mut(),
                Blink: core::ptr::null_mut(),
            },
        },
    },
};

pub static mut PowerSettingNotificationHandle: PVOID = core::ptr::null_mut();
pub static mut ScreenStateNotificationHandle: PVOID = core::ptr::null_mut();

pub static DiskIdleTimeoutInMS: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0xFFFFFFFF);
#[no_mangle]
pub unsafe extern "C" fn DllUnload() -> NTSTATUS {

    DbgPrintEx(
        DPFLTR_CLASSPNP_ID,
        3,
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
pub unsafe extern "C" fn ClasspModeSelect(
    Fdo: PDEVICE_OBJECT,
    ModeSelectBuffer: PCHAR,
    Length: ULONG,
    SavePages: BOOLEAN
) -> NTSTATUS {

    #[repr(C)]
    struct TRANSFER_PACKET {
        _opaque: [u8; 0],
    }

    let mut status: NTSTATUS = STATUS_UNSUCCESSFUL;

    let senseBufferMdl = BuildDeviceInputMdl(ModeSelectBuffer, Length);
    if !senseBufferMdl.is_null() {
        let pkt = DequeueFreeTransferPacket(Fdo, 1u8);
        if !pkt.is_null() {
            let mut event: KEVENT = core::mem::zeroed();
            let mut pseudoIrp: IRP = core::mem::zeroed();

            pseudoIrp.Tail.Overlay.DriverContext[0] = LongToPtr(1);
            pseudoIrp.IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
            pseudoIrp.IoStatus.Information = 0;
            pseudoIrp.MdlAddress = senseBufferMdl;

            KeInitializeEvent(&mut event, _EVENT_TYPE_SynchronizationEvent, 0);
            SetupModeSelectTransferPacket(
                pkt,
                &mut event,
                ModeSelectBuffer,
                Length as UCHAR,
                SavePages,
                &mut pseudoIrp
            );
            SubmitTransferPacket(pkt);
            KeWaitForSingleObject(
                &mut event as *mut _ as *mut core::ffi::c_void,
                _KWAIT_REASON_Executive,
                0i32 as i8,
                0,
                core::ptr::null_mut()
            );

            if !NT_SUCCESS(pseudoIrp.IoStatus.__bindgen_anon_1.Status) {
                TracePrint(
                    TRACE_LEVEL_WARNING,
                    TRACE_FLAG_GENERAL,
                    b"ClassModeSelect on Fdo %ph failed with status %xh.\0".as_ptr() as PCSTR,
                    Fdo,
                    pseudoIrp.IoStatus.__bindgen_anon_1.Status
                );
            }

            status = pseudoIrp.IoStatus.__bindgen_anon_1.Status;
        }

        FreeDeviceInputMdl(senseBufferMdl);
    } else {
        status = STATUS_INSUFFICIENT_RESOURCES;
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn ClasspModeSense(
    Fdo: PDEVICE_OBJECT,
    ModeSenseBuffer: PCHAR,
    Length: ULONG,
    PageMode: UCHAR,
    PageControl: UCHAR
) -> ULONG {
    let mut lengthTransferred: ULONG = 0;
    let senseBufferMdl: PMDL;

    senseBufferMdl = BuildDeviceInputMdl(ModeSenseBuffer, Length);
    if !senseBufferMdl.is_null() {
        let pkt = DequeueFreeTransferPacket(Fdo, 1u8);
        if !pkt.is_null() {
            let mut event: KEVENT = core::mem::zeroed();
            let mut pseudoIrp: IRP = core::mem::zeroed();

            pseudoIrp.Tail.Overlay.DriverContext[0] = LongToPtr(1);
            pseudoIrp.IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
            pseudoIrp.IoStatus.Information = 0;
            pseudoIrp.MdlAddress = senseBufferMdl;

            NT_ASSERT(Length <= 0x0ff);
            KeInitializeEvent(
                core::ptr::addr_of_mut!(event),
                _EVENT_TYPE_SynchronizationEvent,
                0i32
            );
            SetupModeSenseTransferPacket(
                pkt,
                core::ptr::addr_of_mut!(event),
                ModeSenseBuffer,
                Length as UCHAR,
                PageMode,
                0,
                core::ptr::addr_of_mut!(pseudoIrp),
                PageControl
            );
            SubmitTransferPacket(pkt);
            KeWaitForSingleObject(
                core::ptr::addr_of_mut!(event) as *mut core::ffi::c_void,
                _KWAIT_REASON_Executive,
                0i32 as i8,
                0u8,
                core::ptr::null_mut()
            );

            if NT_SUCCESS(pseudoIrp.IoStatus.__bindgen_anon_1.Status) {
                lengthTransferred = pseudoIrp.IoStatus.Information as ULONG;
            } else {
                TracePrint((
                    TRACE_LEVEL_WARNING,
                    TRACE_FLAG_GENERAL,
                    b"ClasspModeSense on Fdo %ph failed with status %xh.\0".as_ptr() as PCHAR,
                    Fdo,
                    pseudoIrp.IoStatus.__bindgen_anon_1.Status
                ));
            }
        }

        FreeDeviceInputMdl(senseBufferMdl);
    }

    lengthTransferred
}
#[no_mangle]
pub unsafe extern "C" fn ClassAddChild(
    Parent: PFUNCTIONAL_DEVICE_EXTENSION,
    Child: PPHYSICAL_DEVICE_EXTENSION,
    AcquireLock: BOOLEAN
) {
    
    if AcquireLock != 0 {
        ClassAcquireChildLock(Parent);
    }

    #[cfg(debug_assertions)]
    {
        let mut testChild = (*Parent).CommonExtension.ChildList;
        while !testChild.is_null() {
            debug_assert!(testChild != Child);
            testChild = (*testChild).CommonExtension.ChildList;
        }
    }

    (*Child).CommonExtension.ChildList = (*Parent).CommonExtension.ChildList;
    (*Parent).CommonExtension.ChildList = Child;

    if AcquireLock != 0 {
        ClassReleaseChildLock(Parent);
    }
}
#[no_mangle]
pub unsafe extern "C" fn ClassRemoveChild(
    Parent: PFUNCTIONAL_DEVICE_EXTENSION,
    Child: PPHYSICAL_DEVICE_EXTENSION,
    AcquireLock: BOOLEAN,
) -> PPHYSICAL_DEVICE_EXTENSION {
    let mut Child = Child;
    
    if AcquireLock != 0 {
        ClassAcquireChildLock(Parent);
    }

    'try_block: {
        let mut previousChild: PCOMMON_DEVICE_EXTENSION = &mut (*Parent).CommonExtension;

        if (*Parent).CommonExtension.ChildList.is_null() {
            Child = core::ptr::null_mut();
            break 'try_block;
        }

        if !Child.is_null() {
            loop {
                NT_ASSERT(previousChild != &mut (*Child).CommonExtension as *mut _);

                if (*previousChild).ChildList == Child {
                    break;
                }

                previousChild = &mut (*(*previousChild).ChildList).CommonExtension;
                
                if previousChild.is_null() {
                    break;
                }
            }

            if previousChild.is_null() {
                Child = core::ptr::null_mut();
                break 'try_block;
            }
        }

        Child = (*previousChild).ChildList;
        (*previousChild).ChildList = (*Child).CommonExtension.ChildList;
        (*Child).CommonExtension.ChildList = core::ptr::null_mut();
    }

    if AcquireLock != 0 {
        ClassReleaseChildLock(Parent);
    }
    
    Child
}
#[no_mangle]
pub unsafe extern "C" fn ClassPnpQueryFdoRelations(
    Fdo: PDEVICE_OBJECT,
    Irp: PIRP
) -> NTSTATUS {
    let fdoExtension = (*Fdo).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let driverExtension = IoGetDriverObjectExtension(
        (*Fdo).DriverObject,
        CLASS_DRIVER_EXTENSION_KEY as PVOID
    ) as PCLASS_DRIVER_EXTENSION;

    if InterlockedIncrement(core::ptr::addr_of_mut!((*fdoExtension).EnumerationInterlock) as *mut LONG) == 1 {
        ((*driverExtension).InitData.ClassEnumerateDevice)(Fdo);
    }

    (*Irp).IoStatus.Anonymous.__bindgen_anon_1.Status = ClassRetrieveDeviceRelations(
        Fdo,
        BusRelations,
        core::ptr::addr_of_mut!((*Irp).IoStatus.Information) as *mut PDEVICE_RELATIONS
    );
    
    InterlockedDecrement(core::ptr::addr_of_mut!((*fdoExtension).EnumerationInterlock) as *mut LONG);

    (*Irp).IoStatus.Anonymous.__bindgen_anon_1.Status
}
#[no_mangle]
pub unsafe extern "C" fn ClassGetPdoId(
    Pdo: PDEVICE_OBJECT,
    IdType: BUS_QUERY_ID_TYPE,
    IdString: PUNICODE_STRING
) -> NTSTATUS {

    let driverExtension = IoGetDriverObjectExtension(
        (*Pdo).DriverObject,
        CLASS_DRIVER_EXTENSION_KEY as *mut core::ffi::c_void
    ) as PCLASS_DRIVER_EXTENSION;

    ASSERT_PDO(Pdo);
    NT_ASSERT((*driverExtension).InitData.ClassQueryId.is_some());

    (*driverExtension).InitData.ClassQueryId.unwrap()(Pdo, IdType, IdString)
}
#[no_mangle]
pub unsafe extern "C" fn ClassRetryRequest(
    SelfDeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    TimeDelta100ns: i64
) {
    let fdoExtension: *mut FUNCTIONAL_DEVICE_EXTENSION;
    let fdoData: *mut CLASS_PRIVATE_FDO_DATA;
    let retryInfo: *mut CLASS_RETRY_INFO;
    let mut delta: i64;
    let mut irql: KIRQL = 0;
    let mut TimeDelta100ns = TimeDelta100ns;

    NT_ASSERT((core::mem::size_of::<CLASS_RETRY_INFO>() <= (4 * core::mem::size_of::<*mut core::ffi::c_void>())) as u32);

    fdoExtension = (*SelfDeviceObject).DeviceExtension as *mut FUNCTIONAL_DEVICE_EXTENSION;

    if (*fdoExtension).CommonExtension.IsFdo == 0 {
        TracePrint(TRACE_LEVEL_ERROR, TRACE_FLAG_GENERAL, b"ClassRetryRequestEx: LOST IRP %p\n\0".as_ptr(), Irp);
        NT_ASSERT(0);
        return;
    }

    fdoData = (*fdoExtension).PrivateFdoData;

    if TimeDelta100ns < 0 {
        NT_ASSERT(0);
        TimeDelta100ns *= -1;
    }

    IoMarkIrpPending(Irp);

    retryInfo = core::ptr::addr_of_mut!((*Irp).Tail.Overlay.__bindgen_anon_2.__bindgen_anon_1.DriverContext) as *mut CLASS_RETRY_INFO;
    core::ptr::write_bytes(retryInfo, 0, 1);

    delta = TimeDelta100ns / (*fdoData).Retry.Granularity;
    if TimeDelta100ns % (*fdoData).Retry.Granularity != 0 {
        delta += 1;
    }
    if delta == 0 {
        delta = MINIMUM_RETRY_UNITS;
    }

    KeAcquireSpinLock(core::ptr::addr_of_mut!((*fdoData).Retry.Lock), core::ptr::addr_of_mut!(irql));

    (*retryInfo).Next = (*fdoData).Retry.ListHead;
    (*fdoData).Retry.ListHead = retryInfo;

    if (*fdoData).Retry.Delta == 0 {
        TracePrint(TRACE_LEVEL_INFORMATION, TRACE_FLAG_GENERAL, b"ClassRetry: +++ %p\n\0".as_ptr(), Irp);

        NT_ASSERT(((*fdoData).Retry.ListHead != core::ptr::null_mut()) as u32);
        NT_ASSERT(((*(*fdoData).Retry.ListHead).Next == core::ptr::null_mut()) as u32);

        KeQueryTickCount(core::ptr::addr_of_mut!((*fdoData).Retry.Tick));
        (*fdoData).Retry.Tick += delta;
        (*fdoData).Retry.Delta = delta;
        ClasspRetryDpcTimer(fdoData);

    } else if delta > (*fdoData).Retry.Delta {
        TracePrint(TRACE_LEVEL_INFORMATION, TRACE_FLAG_GENERAL, b"ClassRetry:  ++ %p\n\0".as_ptr(), Irp);

        NT_ASSERT(((*fdoData).Retry.ListHead != core::ptr::null_mut()) as u32);
        NT_ASSERT(((*(*fdoData).Retry.ListHead).Next != core::ptr::null_mut()) as u32);

        (*fdoData).Retry.Tick -= (*fdoData).Retry.Delta;
        (*fdoData).Retry.Tick += delta;

        (*fdoData).Retry.Delta = delta;

    } else {
        TracePrint(TRACE_LEVEL_INFORMATION, TRACE_FLAG_GENERAL, b"ClassRetry:  ++ %p\n\0".as_ptr(), Irp);
    }

    KeReleaseSpinLock(core::ptr::addr_of_mut!((*fdoData).Retry.Lock), irql);
}
#[no_mangle]
pub unsafe extern "C" fn ClasspReleaseQueue(
    Fdo: PDEVICE_OBJECT,
    ReleaseQueueIrp: PIRP
) {
    let mut irpStack: PIO_STACK_LOCATION;
    let mut irp: PIRP;
    let fdoExtension = Fdo as *mut FUNCTIONAL_DEVICE_EXTENSION;
    let lowerDevice: PDEVICE_OBJECT;
    let mut srb: PSTORAGE_REQUEST_BLOCK_HEADER;
    let mut currentIrql: KIRQL = 0;
    let function: ULONG;

    lowerDevice = (*fdoExtension).CommonExtension.LowerDeviceObject;

    KeRaiseIrql(DISPATCH_LEVEL, &mut currentIrql);

    KeAcquireSpinLockAtDpcLevel(core::ptr::addr_of_mut!((*fdoExtension).ReleaseQueueSpinLock));

    NT_ASSERT(ReleaseQueueIrp.is_null() || 
           ReleaseQueueIrp == (*(*fdoExtension).PrivateFdoData).ReleaseQueueIrp);

    NT_ASSERT((*(*fdoExtension).PrivateFdoData).ReleaseQueueIrpAllocated != 0);
    if (*(*fdoExtension).PrivateFdoData).ReleaseQueueIrpAllocated == 0 {
        ClasspAllocateReleaseQueueIrp(fdoExtension);
    }
    if (*(*fdoExtension).PrivateFdoData).ReleaseQueueIrpAllocated == 0 {
        KeBugCheckEx(0x52, 0x12, Fdo as ULONG_PTR, 0x0, 0x0);
    }

    if ((*fdoExtension).ReleaseQueueInProgress != 0) && ReleaseQueueIrp.is_null() {

        (*fdoExtension).ReleaseQueueNeeded = 1u8;
        KeReleaseSpinLockFromDpcLevel(core::ptr::addr_of_mut!((*fdoExtension).ReleaseQueueSpinLock));
        KeLowerIrql(currentIrql);
        return;

    }

    (*fdoExtension).ReleaseQueueInProgress = 1u8;
    if !ReleaseQueueIrp.is_null() {
        irp = ReleaseQueueIrp;
    } else {
        irp = (*(*fdoExtension).PrivateFdoData).ReleaseQueueIrp;
    }

    if (*(*fdoExtension).AdapterDescriptor).SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK {
        srb = core::ptr::addr_of_mut!((*(*fdoExtension).PrivateFdoData).ReleaseQueueSrb.SrbEx) as PSTORAGE_REQUEST_BLOCK_HEADER;
    } else {
        srb = core::ptr::addr_of_mut!((*fdoExtension).ReleaseQueueSrb) as PSTORAGE_REQUEST_BLOCK_HEADER;
    }

    KeReleaseSpinLockFromDpcLevel(core::ptr::addr_of_mut!((*fdoExtension).ReleaseQueueSpinLock));

    NT_ASSERT(!irp.is_null());

    irpStack = IoGetNextIrpStackLocation(irp);

    (*irpStack).MajorFunction = IRP_MJ_SCSI as u8;

    SrbSetOriginalRequest(srb, irp);

    (*irpStack).Parameters.Scsi.Srb = srb as PSCSI_REQUEST_BLOCK;

    if TEST_FLAG((*Fdo).Characteristics, FILE_REMOVABLE_MEDIA) != 0 {
       function = SRB_FUNCTION_FLUSH_QUEUE;
    } else {
       function = SRB_FUNCTION_RELEASE_QUEUE;
    }

    if (*(*fdoExtension).AdapterDescriptor).SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK {
        (*(srb as PSTORAGE_REQUEST_BLOCK)).SrbFunction = function;
    } else {
        (*srb).Function = function as UCHAR;
    }

    ClassAcquireRemoveLock(Fdo, irp);

    IoSetCompletionRoutine(irp,
                           Some(ClassReleaseQueueCompletion),
                           Fdo as PVOID,
                           1u8,
                           1u8,
                           1u8);

    IoCallDriver(lowerDevice, irp);

    KeLowerIrql(currentIrql);

    return;
}
#[no_mangle]
pub unsafe extern "C" fn ClasspAllocateReleaseRequest(
    Fdo: PDEVICE_OBJECT
) -> NTSTATUS {
    let fdoExtension = Fdo as PFUNCTIONAL_DEVICE_EXTENSION;

    KeInitializeSpinLock(core::ptr::addr_of_mut!((*fdoExtension).ReleaseQueueSpinLock));

    (*fdoExtension).ReleaseQueueNeeded = 0u8;
    (*fdoExtension).ReleaseQueueInProgress = 0u8;
    (*fdoExtension).ReleaseQueueIrpFromPool = 0u8;

    (*fdoExtension).ReleaseQueueIrp = core::ptr::null_mut();

    (*fdoExtension).ReleaseQueueSrb.Length = core::mem::size_of::<SCSI_REQUEST_BLOCK>() as u16;

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn ClasspFreeReleaseRequest(
    Fdo: PDEVICE_OBJECT
) {
    let fdoExtension: PFUNCTIONAL_DEVICE_EXTENSION = (*Fdo).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;

    NT_ASSERT(if (*fdoExtension).CommonExtension.IsRemoved != NO_REMOVE { 1 } else { 0 });

    if !(*fdoExtension).ReleaseQueueIrp.is_null() {
        if (*fdoExtension).ReleaseQueueIrpFromPool != 0 {
            FREE_POOL((*fdoExtension).ReleaseQueueIrp as PVOID);
        } else {
            IoFreeIrp((*fdoExtension).ReleaseQueueIrp);
        }
        (*fdoExtension).ReleaseQueueIrp = core::ptr::null_mut();
    }

    if !(*fdoExtension).PrivateFdoData.is_null() &&
       (*(*fdoExtension).PrivateFdoData).ReleaseQueueIrpAllocated != 0 {

        FREE_POOL((*(*fdoExtension).PrivateFdoData).ReleaseQueueIrp as PVOID);
        (*(*fdoExtension).PrivateFdoData).ReleaseQueueIrpAllocated = 0;
    }
}
#[no_mangle]
pub unsafe extern "C" fn ClasspGetInquiryVpdSupportInfo(
    FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION
) {
    let mut status: NTSTATUS;
    let commonExtension: PCOMMON_DEVICE_EXTENSION = FdoExtension as PCOMMON_DEVICE_EXTENSION;
    let mut srb: SCSI_REQUEST_BLOCK = core::mem::zeroed();
    let mut cdb: PCDB;
    let mut supportedPages: PVPD_SUPPORTED_PAGES_PAGE = core::ptr::null_mut();
    let bufferLength: UCHAR = VPD_MAX_BUFFER_SIZE;
    let mut allocationBufferLength: ULONG = bufferLength as ULONG;
    let mut srbExBuffer: [UCHAR; CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE] = [0; CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE];
    let mut srbHeader: PSTORAGE_REQUEST_BLOCK_HEADER;

    #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
    {
        allocationBufferLength = ALIGN_UP_BY(allocationBufferLength, KeGetRecommendedSharedDataAlignment());
        supportedPages = ExAllocatePoolWithTag(
            NonPagedPoolNxCacheAligned,
            allocationBufferLength as SIZE_T,
            0x33436353
        ) as PVPD_SUPPORTED_PAGES_PAGE;
    }

    #[cfg(not(any(target_arch = "arm", target_arch = "aarch64")))]
    {
        supportedPages = ExAllocatePoolWithTag(
            NonPagedPoolNx,
            bufferLength as SIZE_T,
            0x33436353
        ) as PVPD_SUPPORTED_PAGES_PAGE;
    }

    if supportedPages.is_null() {
        return;
    }

    RtlZeroMemory(supportedPages as PVOID, allocationBufferLength as SIZE_T);

    if (*(*FdoExtension).AdapterDescriptor).SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK {
        status = InitializeStorageRequestBlock(
            srbExBuffer.as_mut_ptr() as PSTORAGE_REQUEST_BLOCK,
            STORAGE_ADDRESS_TYPE_BTL8,
            CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE as ULONG,
            1,
            SrbExDataTypeScsiCdb16
        );

        if !NT_SUCCESS(status) {
            NT_ASSERT(0);
            srb.Length = SCSI_REQUEST_BLOCK_SIZE as USHORT;
            srb.Function = SRB_FUNCTION_EXECUTE_SCSI;
            srbHeader = core::ptr::addr_of_mut!(srb) as PSTORAGE_REQUEST_BLOCK_HEADER;
        } else {
            (*(srbExBuffer.as_mut_ptr() as PSTORAGE_REQUEST_BLOCK)).SrbFunction = SRB_FUNCTION_EXECUTE_SCSI;
            srbHeader = srbExBuffer.as_mut_ptr() as PSTORAGE_REQUEST_BLOCK_HEADER;
        }
    } else {
        srb.Length = SCSI_REQUEST_BLOCK_SIZE as USHORT;
        srb.Function = SRB_FUNCTION_EXECUTE_SCSI;
        srbHeader = core::ptr::addr_of_mut!(srb) as PSTORAGE_REQUEST_BLOCK_HEADER;
    }

    SrbSetTimeOutValue(srbHeader, (*FdoExtension).TimeOutValue);
    SrbSetRequestTag(srbHeader, SP_UNTAGGED);
    SrbSetRequestAttribute(srbHeader, SRB_SIMPLE_TAG_REQUEST);
    SrbAssignSrbFlags(srbHeader, (*FdoExtension).SrbFlags);
    SrbSetCdbLength(srbHeader, 6);

    cdb = SrbGetCdb(srbHeader);
    (*cdb).CDB6INQUIRY3.OperationCode = SCSIOP_INQUIRY;
    (*cdb).CDB6INQUIRY3.EnableVitalProductData = 1;
    (*cdb).CDB6INQUIRY3.PageCode = VPD_SUPPORTED_PAGES;
    (*cdb).CDB6INQUIRY3.AllocationLength = bufferLength;

    status = ClassSendSrbSynchronous(
        (*commonExtension).DeviceObject,
        srbHeader as PSCSI_REQUEST_BLOCK,
        supportedPages as PVOID,
        allocationBufferLength,
        0u8
    );

    if status == STATUS_DATA_OVERRUN && SrbGetDataTransferLength(srbHeader) < bufferLength as ULONG {
        status = STATUS_SUCCESS;
    }

    if NT_SUCCESS(status) &&
       (*supportedPages).PageLength > 0 &&
       (*supportedPages).SupportedPageList[0] > 0 {
        status = STATUS_NOT_SUPPORTED;
    }

    if NT_SUCCESS(status) {
        let mut i: i32 = 0;

        while i < (*supportedPages).PageLength as i32 {
            if i > 0 && (*supportedPages).SupportedPageList[i as usize] <= (*supportedPages).SupportedPageList[(i - 1) as usize] {
                (*(*(*FdoExtension).FunctionSupportInfo).ValidInquiryPages).BlockDeviceRODLimits = 0u8;
                (*(*(*FdoExtension).FunctionSupportInfo).ValidInquiryPages).BlockLimits = 0u8;
                (*(*(*FdoExtension).FunctionSupportInfo).ValidInquiryPages).BlockDeviceCharacteristics = 0u8;
                (*(*(*FdoExtension).FunctionSupportInfo).ValidInquiryPages).LBProvisioning = 0u8;
                break;
            }

            match (*supportedPages).SupportedPageList[i as usize] {
                VPD_THIRD_PARTY_COPY => {
                    (*(*(*FdoExtension).FunctionSupportInfo).ValidInquiryPages).BlockDeviceRODLimits = 1u8;
                }
                VPD_BLOCK_LIMITS => {
                    (*(*(*FdoExtension).FunctionSupportInfo).ValidInquiryPages).BlockLimits = 1u8;
                }
                VPD_BLOCK_DEVICE_CHARACTERISTICS => {
                    (*(*(*FdoExtension).FunctionSupportInfo).ValidInquiryPages).BlockDeviceCharacteristics = 1u8;
                }
                VPD_LOGICAL_BLOCK_PROVISIONING => {
                    (*(*(*FdoExtension).FunctionSupportInfo).ValidInquiryPages).LBProvisioning = 1u8;
                }
                _ => {}
            }

            i += 1;
        }
    }

    ExFreePool(supportedPages as PVOID);
}
#[no_mangle]
pub unsafe extern "C" fn ClasspGetLBProvisioningInfo(
    FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION
) -> NTSTATUS {
    let mut srb: PSCSI_REQUEST_BLOCK = core::ptr::null_mut();
    let mut srbSize: ULONG = 0;

    if FdoExtension.is_null() ||
        (*FdoExtension).FunctionSupportInfo.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    if ((*(*FdoExtension).FunctionSupportInfo).ValidInquiryPages.LBProvisioning == 1u8) ||
        ((*(*FdoExtension).FunctionSupportInfo).ValidInquiryPages.BlockLimits == 1u8) {

        if (!(*FdoExtension).AdapterDescriptor.is_null()) &&
            ((*(*FdoExtension).AdapterDescriptor).SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK) {
            srbSize = CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE;
        } else {
            srbSize = core::mem::size_of::<SCSI_REQUEST_BLOCK>() as ULONG;
        }

        srb = ExAllocatePoolWithTag(NonPagedPoolNx, srbSize as usize, u32::from_le_bytes(*b"0DcS")) as PSCSI_REQUEST_BLOCK;

        if srb.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }
    }

    ClasspDeviceGetLBProvisioningVPDPage((*FdoExtension).DeviceObject, srb);

    ClasspDeviceGetBlockLimitsVPDPage(FdoExtension,
                                      srb,
                                      srbSize,
                                      core::ptr::addr_of_mut!((*(*FdoExtension).FunctionSupportInfo).BlockLimitsData));

    FREE_POOL(srb);

    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn ClasspBlockLimitsDataSnapshot(
    FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION,
    ForceQuery: BOOLEAN,
    BlockLimitsData: PCLASS_VPD_B0_DATA,
    GenerationCount: *mut ULONG
) -> NTSTATUS {
    let status: NTSTATUS;
    let mut lockHandle: KLOCK_QUEUE_HANDLE = core::mem::zeroed();

    status = ClasspRefreshFunctionSupportInfo(FdoExtension, ForceQuery);

    KeAcquireInStackQueuedSpinLock(
        core::ptr::addr_of_mut!((*(*FdoExtension).FunctionSupportInfo).SyncLock),
        &mut lockHandle
    );
    *BlockLimitsData = (*(*FdoExtension).FunctionSupportInfo).BlockLimitsData;
    *GenerationCount = (*(*FdoExtension).FunctionSupportInfo).GenerationCount;
    KeReleaseInStackQueuedSpinLock(&mut lockHandle);

    status
}
#[no_mangle]
pub unsafe extern "C" fn ClasspRegisterMountedDeviceInterface(
    DeviceObject: PDEVICE_OBJECT
) {
    let commonExtension: *mut COMMON_DEVICE_EXTENSION = (*DeviceObject).DeviceExtension as *mut COMMON_DEVICE_EXTENSION;
    let isFdo: u8 = (*commonExtension).IsFdo;
    let pdo: PDEVICE_OBJECT;
    let mut interfaceName: _UNICODE_STRING = core::mem::zeroed();
    let status: NTSTATUS;

    if isFdo != 0 {
        let functionalExtension: *mut FUNCTIONAL_DEVICE_EXTENSION = commonExtension as *mut FUNCTIONAL_DEVICE_EXTENSION;
        pdo = (*functionalExtension).LowerPdo;
    } else {
        pdo = DeviceObject;
    }

    status = IoRegisterDeviceInterface(
        pdo,
        &MOUNTDEV_MOUNTED_DEVICE_GUID,
        core::ptr::null_mut(),
        &mut interfaceName
    );

    if NT_SUCCESS(status) {
        (*commonExtension).MountedDeviceInterfaceName = interfaceName;
        let status2 = IoSetDeviceInterfaceState(&mut interfaceName, 1u8);

        if !NT_SUCCESS(status2) {
            RtlFreeUnicodeString(&mut interfaceName);
        }
    }

    if !NT_SUCCESS(status) {
        RtlInitUnicodeString(
            &mut (*commonExtension).MountedDeviceInterfaceName,
            core::ptr::null_mut()
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn ClassDetermineTokenOperationCommandSupport(
    DeviceObject: PDEVICE_OBJECT
) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_SUCCESS;

    TracePrint((TRACE_LEVEL_VERBOSE,
                TRACE_FLAG_PNP,
                b"ClassDetermineTokenOperationCommandSupport (%p): Entering function.\n\0".as_ptr() as PCHAR,
                DeviceObject));

    status = ClasspGetBlockDeviceTokenLimitsInfo(DeviceObject);

    if NT_SUCCESS(status) {

        let mut maxListIdentifier: ULONG = MaxTokenOperationListIdentifier;

        if NT_SUCCESS(ClasspGetMaximumTokenListIdentifier(DeviceObject, REG_DISK_CLASS_CONTROL, &mut maxListIdentifier)) {
            if maxListIdentifier >= MIN_TOKEN_LIST_IDENTIFIERS {

                NT_ASSERT(maxListIdentifier <= MAX_TOKEN_LIST_IDENTIFIERS);
                MaxTokenOperationListIdentifier = maxListIdentifier;
            }
        }

    }

    TracePrint((TRACE_LEVEL_VERBOSE,
                TRACE_FLAG_PNP,
                b"ClassDetermineTokenOperationCommandSupport (%p): Exiting function with status %x.\n\0".as_ptr() as PCHAR,
                DeviceObject,
                status));

    return status;
}
