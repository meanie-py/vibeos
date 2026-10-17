// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
pub static RUSTCONV_WSTR_505EA03B4355: [u16; 20] = [
    0x005Cu16, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x005C, 0x0052, 0x0061,
    0x0069, 0x0064, 0x0050, 0x006F, 0x0072, 0x0074, 0x0025, 0x006C, 0x0075, 0x0000,
];

use crate::prelude::*;
use ntkrnl_sys::ddk::{DEVICE_DESCRIPTION, ExFreePool, ExInitializeNPagedLookasideList, IoAllocateDriverObjectExtension, IoAttachDeviceToDeviceStackSafe, IoCreateDevice, IoDeleteDevice, IoGetDmaAdapter, IoGetDriverObjectExtension, KLOCK_QUEUE_HANDLE, KeAcquireInStackQueuedSpinLock, KeAcquireInterruptSpinLock, KeCancelTimer, KeInitializeDpc, KeInitializeTimer, KeInsertQueueDpc, KeReleaseInStackQueuedSpinLock, KeReleaseInterruptSpinLock, KeSetTimer, KeStallExecutionProcessor, LARGE_INTEGER, MmAllocateContiguousMemorySpecifyCache, MmGetPhysicalAddress, MmMapIoSpace, PBUS_INTERFACE_STANDARD, PDEVICE_OBJECT, PDRIVER_OBJECT, PHYSICAL_ADDRESS, PIRP, PLARGE_INTEGER, PRKDPC, PULONGLONG, PUNICODE_STRING, RtlInitUnicodeString, va_list};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut PortNumber: ULONG = 0 as ULONG;

// #define NDEBUG: no value, so nothing to emit.
// Bare STORPORT_API prototype fragments: no declarations to emit; they are
// covered by the per-function translations.
// StorPortQuerySystemTime: function definition, emitted with its function body.


unsafe extern "C" fn PortAddDriverInitData(
    DriverExtension: PDRIVER_OBJECT_EXTENSION,
    HwInitializationData: PHW_INITIALIZATION_DATA,
) -> NTSTATUS {
    let mut InitData: PDRIVER_INIT_DATA;

    DPRINT1!("PortAddDriverInitData()\n");

    InitData = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<DRIVER_INIT_DATA>() as SIZE_T,
        TAG_INIT_DATA,
    ) as PDRIVER_INIT_DATA;
    if InitData.is_null() {
        return STATUS_NO_MEMORY;
    }

    RtlCopyMemory!(
        &raw mut (*InitData).HwInitData as *mut ::core::ffi::c_void,
        HwInitializationData as *const ::core::ffi::c_void,
        core::mem::size_of::<HW_INITIALIZATION_DATA>() as SIZE_T,
    );

    InsertHeadList(
        &raw mut (*DriverExtension).InitDataListHead,
        &raw mut (*InitData).Entry,
    );

    return STATUS_SUCCESS;
}

unsafe extern "C" fn PortDeleteDriverInitData(DriverExtension: PDRIVER_OBJECT_EXTENSION) {
    let mut InitData: PDRIVER_INIT_DATA = core::ptr::null_mut();
    let mut ListEntry: PLIST_ENTRY = core::ptr::null_mut();

    DPRINT1!("PortDeleteDriverInitData()\n");

    ListEntry = (*DriverExtension).InitDataListHead.Flink as PLIST_ENTRY;
    while ListEntry != &raw mut (*DriverExtension).InitDataListHead {
        InitData = (ListEntry as ULONG_PTR).wrapping_sub(
            core::mem::offset_of!(DRIVER_INIT_DATA, Entry) as ULONG_PTR,
        ) as *mut DRIVER_INIT_DATA as PDRIVER_INIT_DATA;

        RemoveEntryList(&raw mut (*InitData).Entry);

        ExFreePoolWithTag(InitData as PVOID, TAG_INIT_DATA as ULONG);

        ListEntry = (*DriverExtension).InitDataListHead.Flink as PLIST_ENTRY;
    }
}
#[no_mangle]
pub unsafe extern "C" fn PortGetDriverInitData(
    DriverExtension: PDRIVER_OBJECT_EXTENSION,
    InterfaceType: INTERFACE_TYPE,
) -> PHW_INITIALIZATION_DATA {
    let mut InitData: PDRIVER_INIT_DATA;
    let mut ListEntry: PLIST_ENTRY;

    DPRINT1!("PortGetDriverInitData()\n");

    ListEntry = (*DriverExtension).InitDataListHead.Flink;
    while ListEntry != &raw mut (*DriverExtension).InitDataListHead {
        InitData = (ListEntry as ULONG_PTR).wrapping_sub(
            core::mem::offset_of!(DRIVER_INIT_DATA, Entry) as ULONG_PTR,
        ) as PDRIVER_INIT_DATA;
        if (*InitData).HwInitData.AdapterInterfaceType as ::core::ffi::c_int
            == InterfaceType as ::core::ffi::c_int
        {
            return &raw mut (*InitData).HwInitData;
        }

        ListEntry = (*ListEntry).Flink;
    }

    return core::ptr::null_mut();
}

unsafe extern "C" fn PortAcquireSpinLock(
    DeviceExtension: PFDO_DEVICE_EXTENSION,
    SpinLock: STOR_SPINLOCK,
    LockContext: PVOID,
    LockHandle: PSTOR_LOCK_HANDLE,
) {
    DPRINT!(
        "PortAcquireSpinLock(%p %lu %p %p)\n",
        DeviceExtension,
        SpinLock,
        LockContext,
        LockHandle
    );

    (*LockHandle).Lock = SpinLock;

    match SpinLock {
        DpcLock => {
            /* 1, */
            DPRINT!("DpcLock\n");
        }
        StartIoLock => {
            /* 2 */
            DPRINT!("StartIoLock\n");
        }
        InterruptLock => {
            /* 3 */
            DPRINT!("InterruptLock\n");
            if (*DeviceExtension).Interrupt.is_null() {
                (*LockHandle).Context.OldIrql = 0;
            } else {
                (*LockHandle).Context.OldIrql =
                    KeAcquireInterruptSpinLock((*DeviceExtension).Interrupt);
            }
        }
        _ => {}
    }
}

unsafe extern "C" fn PortReleaseSpinLock(
    DeviceExtension: PFDO_DEVICE_EXTENSION,
    LockHandle: PSTOR_LOCK_HANDLE,
) {
    DPRINT!("PortReleaseSpinLock(%p %p)\n", DeviceExtension, LockHandle);

    match (*LockHandle).Lock {
        DpcLock => {
            /* 1, */
            DPRINT!("DpcLock\n");
        }

        StartIoLock => {
            /* 2 */
            DPRINT!("StartIoLock\n");
        }

        InterruptLock => {
            /* 3 */
            DPRINT!("InterruptLock\n");
            if !(*DeviceExtension).Interrupt.is_null() {
                KeReleaseInterruptSpinLock(
                    (*DeviceExtension).Interrupt,
                    (*LockHandle).Context.OldIrql,
                );
            }
        }

        _ => {}
    }
}
unsafe extern "system" fn PortAddDevice(
    mut DriverObject: PDRIVER_OBJECT,
    mut PhysicalDeviceObject: PDEVICE_OBJECT,
) -> NTSTATUS {
    let mut DriverObjectExtension: PDRIVER_OBJECT_EXTENSION = core::ptr::null_mut();
    let mut DeviceExtension: PFDO_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut NameBuffer: [WCHAR; 27] = [0; 27];
    let mut DeviceName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut Fdo: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut Status: NTSTATUS = 0;

    DPRINT1!("PortAddDevice(%p %p)\n", DriverObject, PhysicalDeviceObject);

    ASSERT!(!DriverObject.is_null());
    ASSERT!(!PhysicalDeviceObject.is_null());

    Status = RtlStringCbPrintfW!(
        core::ptr::addr_of_mut!(NameBuffer) as *mut WCHAR,
        core::mem::size_of::<[WCHAR; 27]>() as SIZE_T,
        RUSTCONV_WSTR_505EA03B4355.as_ptr(),
        PortNumber,
    );
    ASSERT!(NT_SUCCESS!(Status));
    RtlInitUnicodeString(
        core::ptr::addr_of_mut!(DeviceName),
        core::ptr::addr_of!(NameBuffer) as PCWSTR,
    );
    PortNumber = PortNumber.wrapping_add(1);

    DPRINT1!("Creating device: %wZ\n", core::ptr::addr_of_mut!(DeviceName));

    /* Create the port device */
    Status = IoCreateDevice(
        DriverObject,
        core::mem::size_of::<FDO_DEVICE_EXTENSION>() as ULONG,
        core::ptr::addr_of_mut!(DeviceName),
        FILE_DEVICE_CONTROLLER as ULONG,
        FILE_DEVICE_SECURE_OPEN as ULONG,
        FALSE as BOOLEAN,
        core::ptr::addr_of_mut!(Fdo),
    );
    if !NT_SUCCESS!(Status) {
        DPRINT1!("IoCreateDevice() failed (Status 0x%08lx)\n", Status);
        return Status;
    }

    DPRINT1!("Created device: %wZ (%p)\n", core::ptr::addr_of_mut!(DeviceName), Fdo);

    /* Initialize the device */
    (*Fdo).Flags |= DO_DIRECT_IO as ULONG;
    (*Fdo).Flags |= DO_POWER_PAGABLE as ULONG;

    /* Initialize the device extension */
    DeviceExtension = (*Fdo).DeviceExtension as PFDO_DEVICE_EXTENSION;
    RtlZeroMemory!(
        DeviceExtension as PVOID,
        core::mem::size_of::<FDO_DEVICE_EXTENSION>() as SIZE_T,
    );

    (*DeviceExtension).ExtensionType = EXTENSION_TYPE_FdoExtension;

    (*DeviceExtension).Device = Fdo;
    (*DeviceExtension).PhysicalDevice = PhysicalDeviceObject;

    (*DeviceExtension).PnpState = dsStopped;

    (*DeviceExtension).ScsiPortNumber = -(1 as LONG);

    KeInitializeSpinLock(core::ptr::addr_of_mut!((*DeviceExtension).PdoListLock));
    InitializeListHead(core::ptr::addr_of_mut!((*DeviceExtension).PdoListHead));

    KeInitializeSpinLock(core::ptr::addr_of_mut!((*DeviceExtension).TimerListLock));
    InitializeListHead(core::ptr::addr_of_mut!((*DeviceExtension).TimerListHead));

    /* Initialize simple timer for StorPortNotification */
    KeInitializeTimer(core::ptr::addr_of_mut!((*DeviceExtension).Timer));
    KeInitializeDpc(
        core::ptr::addr_of_mut!((*DeviceExtension).TimerDpc),
        Some(SimpleTimerCallbackDpcRoutine),
        DeviceExtension as PVOID,
    );

    /* Initialize FDO Device DPC for delayed completion */
    KeInitializeDpc(
        core::ptr::addr_of_mut!((*Fdo).Dpc),
        Some(PortCompletionDpc),
        DeviceExtension as PVOID,
    );

    /* Initialize completion list header */
    InitializeSListHead(core::ptr::addr_of_mut!((*DeviceExtension).CompletionList));

    /* Initialize the request reference lookaside list */
    ExInitializeNPagedLookasideList(
        core::ptr::addr_of_mut!((*DeviceExtension).RequestReferenceLookaside),
        None,
        None,
        0 as ULONG,
        core::mem::size_of::<QUEUED_REQUEST_REFERENCE>() as SIZE_T,
        TAG_QUEUED_REQUEST as ULONG,
        0 as USHORT,
    );

    /* Initialize the queue for requests waiting on memory */
    PortInitializeStarvedRequestQueue(core::ptr::addr_of_mut!((*DeviceExtension).StarvedRequests));

    /* Initialize FDO flow control structure */
    KeInitializeSpinLock(core::ptr::addr_of_mut!((*DeviceExtension).FlowControl.Lock));
    (*DeviceExtension).FlowControl.IsBusy = FALSE as BOOLEAN;
    (*DeviceExtension).FlowControl.IsPaused = FALSE as BOOLEAN;
    (*DeviceExtension).FlowControl.RemainingBusyRequests = 0 as ULONG;
    (*DeviceExtension).FlowControl.RemainingPauseTime = 0 as ULONG;
    (*DeviceExtension).FlowControl.OutstandingRequestCount = 0 as ULONG;
    InitializeListHead(core::ptr::addr_of_mut!(
        (*DeviceExtension).FlowControl.FdoBlockedRequestsListHead
    ));

    /* Attach the FDO to the device stack */
    Status = IoAttachDeviceToDeviceStackSafe(
        Fdo,
        PhysicalDeviceObject,
        core::ptr::addr_of_mut!((*DeviceExtension).LowerDevice),
    );
    if !NT_SUCCESS!(Status) {
        DPRINT1!("IoAttachDeviceToDeviceStackSafe() failed (Status 0x%08lx)\n", Status);
        IoDeleteDevice(Fdo);
        return Status;
    }

    /* Insert the FDO to the drivers FDO list */
    DriverObjectExtension = IoGetDriverObjectExtension(DriverObject, DriverEntry as PVOID)
        as PDRIVER_OBJECT_EXTENSION;
    ASSERT!((*DriverObjectExtension).ExtensionType == EXTENSION_TYPE_DriverExtension);

    (*DeviceExtension).DriverExtension = DriverObjectExtension;

    StorpInterlockedInsertHeadListCounted(
        core::ptr::addr_of_mut!((*DriverObjectExtension).AdapterListLock),
        core::ptr::addr_of_mut!((*DriverObjectExtension).AdapterListHead),
        core::ptr::addr_of_mut!((*DeviceExtension).AdapterListEntry),
        core::ptr::addr_of_mut!((*DriverObjectExtension).AdapterCount),
    );

    /* The device has been initialized */
    (*Fdo).Flags &= !(DO_DEVICE_INITIALIZING as ULONG);

    DPRINT1!("PortAddDevice() done (Status 0x%08lx)\n", Status);

    return Status;
}

unsafe extern "system" fn PortUnload(DriverObject: PDRIVER_OBJECT) {
    let mut DriverExtension: PDRIVER_OBJECT_EXTENSION;

    DPRINT1!("PortUnload(%p)\n", DriverObject);

    DriverExtension = IoGetDriverObjectExtension(DriverObject, DriverEntry as usize as PVOID)
        as PDRIVER_OBJECT_EXTENSION;
    if !DriverExtension.is_null() {
        PortDeleteDriverInitData(DriverExtension);
    }
}
unsafe extern "system" fn PortDispatchCreate(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    DPRINT!("PortDispatchCreate(%p %p)\n", DeviceObject, Irp);

    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    (*Irp).IoStatus.Information = FILE_OPENED as ULONG_PTR;

    IoCompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);

    return STATUS_SUCCESS;
}
unsafe extern "system" fn PortDispatchClose(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    DPRINT!("PortDispatchClose(%p %p)\n", DeviceObject, Irp);

    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    (*Irp).IoStatus.Information = 0;

    IoCompleteRequest(Irp, (IO_NO_INCREMENT as i8));

    return STATUS_SUCCESS;
}

unsafe extern "system" fn PortDispatchDeviceControl(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut DeviceExtension: PFDO_DEVICE_EXTENSION = core::ptr::null_mut();

    DPRINT!("PortDispatchDeviceControl({:p} {:p})\n", DeviceObject, Irp);

    DeviceExtension = (*DeviceObject).DeviceExtension as PFDO_DEVICE_EXTENSION;
    DPRINT!("ExtensionType: {}\n", (*DeviceExtension).ExtensionType);

    match (*DeviceExtension).ExtensionType as u32 {
        2 /* EXTENSION_TYPE_FdoExtension */ => {
            return PortFdoDeviceControl(DeviceObject, Irp);
        }
        3 /* EXTENSION_TYPE_PdoExtension */ => {
            return PortPdoDeviceControl(DeviceObject, Irp);
        }
        _ => {
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_UNSUCCESSFUL;
            (*Irp).IoStatus.Information = 0;
            IoCompleteRequest(Irp, (IO_NO_INCREMENT as i8));
            return STATUS_UNSUCCESSFUL;
        }
    }

    #[allow(unreachable_code)]
    {
        return STATUS_SUCCESS;
    }
}
unsafe extern "system" fn PortDispatchScsi(mut DeviceObject: PDEVICE_OBJECT, mut Irp: PIRP) -> NTSTATUS {
    let mut DeviceExtension: PFDO_DEVICE_EXTENSION = core::ptr::null_mut();

    DPRINT!("PortDispatchScsi(%p %p)\n", DeviceObject, Irp);

    DeviceExtension = (*DeviceObject).DeviceExtension as PFDO_DEVICE_EXTENSION;
    DPRINT!("ExtensionType: %u\n", (*DeviceExtension).ExtensionType);

    match (*DeviceExtension).ExtensionType as u32 {
        EXTENSION_TYPE_FdoExtension => {
            return PortFdoScsi(DeviceObject, Irp);
        }
        EXTENSION_TYPE_PdoExtension => {
            return PortPdoScsi(DeviceObject, Irp);
        }
        _ => {
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_UNSUCCESSFUL;
            (*Irp).IoStatus.Information = 0;
            IoCompleteRequest(Irp, (IO_NO_INCREMENT as i8));
            return STATUS_UNSUCCESSFUL;
        }
    }

    return STATUS_SUCCESS;
}

unsafe extern "system" fn PortDispatchSystemControl(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    DPRINT1!("PortDispatchSystemControl(%p %p)\n", DeviceObject, Irp);

    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    (*Irp).IoStatus.Information = 0;

    IoCompleteRequest(Irp, (IO_NO_INCREMENT as i8));

    return STATUS_SUCCESS;
}
unsafe extern "system" fn PortDispatchPnp(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    let mut DeviceExtension: PFDO_DEVICE_EXTENSION = core::ptr::null_mut();

    DPRINT1!("PortDispatchPnp(%p %p)\n", DeviceObject, Irp);

    DeviceExtension = (*DeviceObject).DeviceExtension as PFDO_DEVICE_EXTENSION;
    DPRINT!("ExtensionType: %u\n", (*DeviceExtension).ExtensionType);

    match (*DeviceExtension).ExtensionType as u32 {
        // EXTENSION_TYPE_FdoExtension
        2 => {
            return PortFdoPnp(DeviceObject, Irp);
        }
        // EXTENSION_TYPE_PdoExtension
        3 => {
            return PortPdoPnp(DeviceObject, Irp);
        }
        _ => {
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_UNSUCCESSFUL;
            (*Irp).IoStatus.Information = 0;
            IoCompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);
            return STATUS_UNSUCCESSFUL;
        }
    }
}
unsafe extern "system" fn PortDispatchPower(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    DPRINT1!("PortDispatchPower(%p %p)\n", DeviceObject, Irp);

    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    (*Irp).IoStatus.Information = 0;

    IoCompleteRequest(Irp, (IO_NO_INCREMENT as i8));

    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn DriverEntry(
    DriverObject: PDRIVER_OBJECT,
    RegistryPath: PUNICODE_STRING,
) -> NTSTATUS {
    DPRINT1!("DriverEntry(%p %p)\n", DriverObject, RegistryPath);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn StorPortAllocateRegistryBuffer(
    HwDeviceExtension: PVOID,
    Length: PULONG,
) -> PUCHAR {
    DPRINT1!("StorPortAllocateRegistryBuffer()\n");
    UNIMPLEMENTED!();
    return core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "system" fn StorPortBusy(HwDeviceExtension: PVOID, RequestsToComplete: ULONG) -> BOOLEAN {
    DPRINT1!("StorPortBuzy()\n");
    UNIMPLEMENTED!();
    return FALSE;
}

#[no_mangle]
pub unsafe extern "system" fn StorPortCompleteRequest(
    HwDeviceExtension: PVOID,
    PathId: UCHAR,
    TargetId: UCHAR,
    Lun: UCHAR,
    SrbStatus: UCHAR,
) {
    DPRINT1!("StorPortCompleteRequest()\n");
    UNIMPLEMENTED!();
}

#[no_mangle]
pub unsafe extern "system" fn StorPortConvertPhysicalAddressToUlong(
    Address: STOR_PHYSICAL_ADDRESS,
) -> ULONG {
    DPRINT!("StorPortConvertPhysicalAddressToUlong()\n");

    return Address.u.LowPart;
}

#[no_mangle]
pub unsafe extern "system" fn StorPortConvertUlongToPhysicalAddress(
    UlongAddress: ULONG_PTR,
) -> STOR_PHYSICAL_ADDRESS {
    let mut Address: STOR_PHYSICAL_ADDRESS = core::mem::zeroed();

    DPRINT!("StorPortConvertUlongToPhysicalAddress()\n");

    Address.QuadPart = UlongAddress as LONGLONG;
    return Address;
}

#[no_mangle]
pub unsafe extern "C" fn StorPortDebugPrint_v(
    DebugPrintLevel: ULONG,
    DebugMessage: PCHAR,
    ap_list: *mut ::core::ffi::c_void,
) {
    let mut ap: va_list = *(ap_list as *mut va_list);
    vDbgPrintExWithPrefix(
        b"STORMINI: \0".as_ptr() as PCCH,
        (1 as u32).wrapping_neg(),
        DPFLTR_ERROR_LEVEL,
        DebugMessage,
        ap,
    );
}



#[no_mangle]
pub unsafe extern "system" fn StorPortDeviceBusy(
    HwDeviceExtension: PVOID,
    PathId: UCHAR,
    TargetId: UCHAR,
    Lun: UCHAR,
    RequestsToComplete: ULONG,
) -> BOOLEAN {
    /* TODO: Need coverage test */
    let mut FdoExtension: PFDO_DEVICE_EXTENSION;
    let mut PdoExtension: PPDO_DEVICE_EXTENSION;
    let mut LockHandle: KLOCK_QUEUE_HANDLE = core::mem::zeroed();

    DPRINT1!("StorPortDeviceBusy(%p %d %d %d %d)\n",
            HwDeviceExtension, PathId as core::ffi::c_uint, TargetId as core::ffi::c_uint, Lun as core::ffi::c_uint, RequestsToComplete);

    FdoExtension = StorpGetMiniportFdo(HwDeviceExtension);
    PdoExtension = FdoFindLun(FdoExtension, (PathId) as ULONG, (TargetId) as ULONG, (Lun) as ULONG);

    KeAcquireInStackQueuedSpinLock(core::ptr::addr_of_mut!((*FdoExtension).FlowControl.Lock), core::ptr::addr_of_mut!(LockHandle));
    NT_ASSERT!(!((*PdoExtension).FlowControl.IsBusy != 0));
    (*PdoExtension).FlowControl.IsBusy = TRUE;
    (*PdoExtension).FlowControl.RemainingBusyRequests = RequestsToComplete;
    KeReleaseInStackQueuedSpinLock(core::ptr::addr_of_mut!(LockHandle));

    return FALSE;
}

#[no_mangle]
pub unsafe extern "system" fn StorPortDeviceReady(
    HwDeviceExtension: PVOID,
    PathId: UCHAR,
    TargetId: UCHAR,
    Lun: UCHAR,
) -> BOOLEAN {
    let mut FdoExtension: PFDO_DEVICE_EXTENSION;
    let mut PdoExtension: PPDO_DEVICE_EXTENSION;
    let mut LockHandle: KLOCK_QUEUE_HANDLE = core::mem::zeroed();

    DPRINT1!(
        "StorPortDeviceReady(%p %d %d %d %d)\n",
        HwDeviceExtension,
        PathId as core::ffi::c_uint,
        TargetId as core::ffi::c_uint,
        Lun as core::ffi::c_uint
    );

    FdoExtension = StorpGetMiniportFdo(HwDeviceExtension);
    PdoExtension = FdoFindLun(FdoExtension, (PathId) as ULONG, (TargetId) as ULONG, (Lun) as ULONG);

    KeAcquireInStackQueuedSpinLock(
        core::ptr::addr_of_mut!((*FdoExtension).FlowControl.Lock),
        core::ptr::addr_of_mut!(LockHandle),
    );
    NT_ASSERT!((*PdoExtension).FlowControl.IsBusy == 0);
    (*PdoExtension).FlowControl.IsBusy = FALSE;
    KeReleaseInStackQueuedSpinLock(core::ptr::addr_of_mut!(LockHandle));

    return FALSE;
}

#[no_mangle]
pub unsafe extern "C" fn StorPortExtendedFunction_v(
    FunctionCode: STORPORT_FUNCTION_CODE,
    HwDeviceExtension: PVOID,
    ap_list: *mut ::core::ffi::c_void,
) -> ULONG {
    let mut va: varargs_rust::VaArgs;
    let mut Status: ULONG = STATUS_NOT_IMPLEMENTED as ULONG;
    let mut MiniportExtension: PMINIPORT_DEVICE_EXTENSION =
        ::core::ptr::null_mut::<MINIPORT_DEVICE_EXTENSION>();
    let mut DeviceExtension: PFDO_DEVICE_EXTENSION =
        ::core::ptr::null_mut::<FDO_DEVICE_EXTENSION>();
    let mut AllocatePoolSize: ULONG = 0;
    let mut PoolTag: ULONG = 0;
    let mut AllocatedPoolPointer: *mut PVOID =
        ::core::ptr::null_mut::<PVOID>();
    let mut PoolPointer: PVOID = ::core::ptr::null_mut::<::core::ffi::c_void>();

    DPRINT1!(
        "StorPortExtendedFunction(%d %p ...)\n",
        FunctionCode,
        HwDeviceExtension
    );

    va = varargs_rust::VaArgs::from_ptr(ap_list);

    match FunctionCode {
        ExtFunctionAllocatePool => {
            /* Allocate a non paged pool for miniport */
            AllocatePoolSize = va.arg::<ULONG>();
            PoolTag = va.arg::<ULONG>();
            AllocatedPoolPointer = va.arg::<*mut PVOID>() as *mut PVOID;

            if AllocatedPoolPointer.is_null() {
                Status = STOR_STATUS_INVALID_PARAMETER as ULONG;
            } else {
                PoolPointer = ExAllocatePoolWithTag(
                    NonPagedPool,
                    AllocatePoolSize as SIZE_T,
                    PoolTag,
                );

                if PoolPointer.is_null() {
                    *AllocatedPoolPointer = ::core::ptr::null_mut::<::core::ffi::c_void>() as _;
                    Status = STOR_STATUS_INSUFFICIENT_RESOURCES as ULONG;
                } else {
                    *AllocatedPoolPointer = PoolPointer as _;
                    Status = STOR_STATUS_SUCCESS as ULONG;
                }
            }
        }

        ExtFunctionFreePool => {
            /* Free a pool allocated with ExtFunctionAllocatePool */
            PoolPointer = va.arg::<PVOID>();

            if PoolPointer.is_null() {
                Status = STOR_STATUS_INVALID_PARAMETER as ULONG;
            } else {
                ExFreePool(PoolPointer);
                Status = STOR_STATUS_SUCCESS as ULONG;
            }
        }

        ExtFunctionInitializeTimer => {
            /* Allocate a timer for an HBA */
            let mut TimerEntry: PTIMER_ENTRY = ::core::ptr::null_mut::<TIMER_ENTRY>();
            let mut TimerHandle: *mut PVOID =
                va.arg::<*mut PVOID>() as *mut PVOID;

            if TimerHandle.is_null() {
                Status = STOR_STATUS_INVALID_PARAMETER as ULONG;
            } else {
                TimerEntry = ExAllocatePoolWithTag(
                    NonPagedPool,
                    ::core::mem::size_of::<TIMER_ENTRY>() as SIZE_T,
                    TAG_MINIPORT_DATA,
                ) as PTIMER_ENTRY;
                RtlZeroMemory!(
                    TimerEntry as PVOID,
                    ::core::mem::size_of::<TIMER_ENTRY>() as SIZE_T,
                );

                if TimerEntry.is_null() {
                    Status = STOR_STATUS_INSUFFICIENT_RESOURCES as ULONG;
                } else {
                    /* Initialize internal timer object; Win8.1+ EX_TIMER may be used too if we had it */
                    /* FIXME: IRQL > DISPATCH_LEVEL will bugcheck; queue a DPC to do this */
                    KeInitializeTimer(&raw mut (*TimerEntry).Timer);

                    MiniportExtension = (HwDeviceExtension as ULONG_PTR).wrapping_sub(
                        core::mem::offset_of!(MINIPORT_DEVICE_EXTENSION, HwDeviceExtension) as ULONG_PTR,
                    ) as *mut MINIPORT_DEVICE_EXTENSION
                        as PMINIPORT_DEVICE_EXTENSION;
                    DeviceExtension =
                        (*(*MiniportExtension).Miniport).DeviceExtension as PFDO_DEVICE_EXTENSION;

                    /* Insert into timer list and increment counter */
                    StorpInterlockedInsertHeadListCounted(
                        &raw mut (*DeviceExtension).TimerListLock,
                        &raw mut (*DeviceExtension).TimerListHead,
                        &raw mut (*TimerEntry).ListEntry,
                        &raw mut (*DeviceExtension).TimerCount,
                    );

                    /* Return list entry of the timer as its handle */
                    *TimerHandle = &raw mut (*TimerEntry).ListEntry as PVOID;
                    Status = STOR_STATUS_SUCCESS as ULONG;
                }
            }
        }

        ExtFunctionRequestTimer => {
            /* Schedule a single shot timer event */
            let mut TimerHandle: PVOID = va.arg::<PVOID>();
            let mut TimerCallback: PHW_TIMER_EX =
                ::core::mem::transmute(va.arg::<*mut PHW_TIMER_EX>());
            let mut CallbackContext: PVOID = va.arg::<PVOID>();
            let mut TimerValue: ULONGLONG = va.arg::<ULONGLONG>();
            let mut TolerableDelay: ULONGLONG = va.arg::<ULONGLONG>();
            let mut TimerEntry: PTIMER_ENTRY = ::core::ptr::null_mut::<TIMER_ENTRY>();
            let mut IsTimerAlreadySet: ULONG = 0;
            let mut TimerDueTime: LARGE_INTEGER = LARGE_INTEGER {
                QuadPart: 0,
            };
            let mut MultiplyResult: NTSTATUS = 0;

            /* FIXME: I actually don't know how to deal with tolerable delay */
            UNREFERENCED_PARAMETER!(TolerableDelay);

            if HwDeviceExtension.is_null() || TimerHandle.is_null() || TimerCallback.is_none() {
                Status = STOR_STATUS_INVALID_PARAMETER as ULONG;
            } else {
                TimerEntry = (TimerHandle as ULONG_PTR).wrapping_sub(
                    core::mem::offset_of!(TIMER_ENTRY, ListEntry)
                        as ULONG_PTR,
                ) as *mut TIMER_ENTRY as PTIMER_ENTRY;

                /* Check if timer is already set. If so, don't try to reactivate timer. */
                IsTimerAlreadySet = InterlockedCompareExchange(
                    &raw mut (*TimerEntry).TimerAlreadySet as *mut ::core::ffi::c_long,
                    1 as ::core::ffi::c_long,
                    0 as ::core::ffi::c_long,
                ) as ULONG;
                if IsTimerAlreadySet == 1 as ULONG {
                    Status = STOR_STATUS_BUSY as ULONG;
                } else if TimerValue == 0 as ULONGLONG {
                    /* If timeout is zero, cancel the timer */
                    KeCancelTimer(&raw mut (*TimerEntry).Timer);
                    Status = STOR_STATUS_SUCCESS as ULONG;
                } else {
                    /* This API uses microseconds, KTIMER operate on 100 nanoseconds, multiply by 10
                       to get the correct timer due time */
                    MultiplyResult = RtlULongLongMult(
                        TimerValue,
                        10 as ULONGLONG,
                        &raw mut TimerDueTime.QuadPart as *mut ULONGLONG,
                    );
                    if !NT_SUCCESS!(MultiplyResult) {
                        Status = STOR_STATUS_INVALID_PARAMETER as ULONG;
                    } else {
                        /* Initialize the DPC, context and set timer accordingly */
                        (*TimerEntry).HwDeviceExtension = HwDeviceExtension;
                        (*TimerEntry).TimerCallback = TimerCallback;
                        (*TimerEntry).Context = CallbackContext;
                        KeInitializeDpc(
                            &raw mut (*TimerEntry).TimerCallbackDpc,
                            Some(
                                TimerCallbackDpcRoutine,
                            ),
                            TimerEntry as PVOID,
                        );
                        KeSetTimer(
                            &raw mut (*TimerEntry).Timer,
                            TimerDueTime,
                            &raw mut (*TimerEntry).TimerCallbackDpc,
                        );
                        Status = STOR_STATUS_SUCCESS as ULONG;
                    }
                }
            }
        }

        _ => {
            UNIMPLEMENTED!();
            // __debugbreak();
        }
    }


    return Status;
}



#[no_mangle]
pub unsafe extern "system" fn StorPortFreeDeviceBase(
    HwDeviceExtension: PVOID,
    MappedAddress: PVOID,
) {
    DPRINT1!("StorPortFreeDeviceBase(%p %p)\n", HwDeviceExtension, MappedAddress);
}

#[no_mangle]
pub unsafe extern "system" fn StorPortFreeRegistryBuffer(
    HwDeviceExtension: PVOID,
    Buffer: PUCHAR,
) {
    DPRINT1!("StorPortFreeRegistryBuffer()\n");
    UNIMPLEMENTED!();
}

#[no_mangle]
pub unsafe extern "system" fn StorPortGetBusData(
    DeviceExtension: PVOID,
    BusDataType: ULONG,
    SystemIoBusNumber: ULONG,
    SlotNumber: ULONG,
    Buffer: PVOID,
    Length: ULONG,
) -> ULONG {
    let mut MiniportExtension: PMINIPORT_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut Interface: PBUS_INTERFACE_STANDARD = core::ptr::null_mut();
    let mut ReturnLength: ULONG = 0;

    DPRINT1!(
        "StorPortGetBusData(%p %lu %lu %lu %p %lu)\n",
        DeviceExtension,
        BusDataType,
        SystemIoBusNumber,
        SlotNumber,
        Buffer,
        Length
    );

    /* Get the miniport extension */
    MiniportExtension = (DeviceExtension as ULONG_PTR).wrapping_sub(
        core::mem::offset_of!(MINIPORT_DEVICE_EXTENSION, HwDeviceExtension) as ULONG_PTR,
    ) as PMINIPORT_DEVICE_EXTENSION;
    DPRINT1!(
        "DeviceExtension %p  MiniportExtension %p\n",
        DeviceExtension,
        MiniportExtension
    );

    Interface = core::ptr::addr_of_mut!(
        (*(*(*MiniportExtension).Miniport).DeviceExtension).BusInterface
    ) as PBUS_INTERFACE_STANDARD;

    ReturnLength = ((*Interface).GetBusData.unwrap_unchecked())(
        (*Interface).Context,
        BusDataType,
        Buffer,
        0,
        Length,
    );
    DPRINT1!("ReturnLength: %lu\n", ReturnLength);

    return ReturnLength;
}

#[no_mangle]
pub unsafe extern "system" fn StorPortGetDeviceBase(
    HwDeviceExtension: PVOID,
    BusType: INTERFACE_TYPE,
    SystemIoBusNumber: ULONG,
    IoAddress: STOR_PHYSICAL_ADDRESS,
    NumberOfBytes: ULONG,
    InIoSpace: BOOLEAN,
) -> PVOID {
    let mut MiniportExtension: PMINIPORT_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut TranslatedAddress: PHYSICAL_ADDRESS = core::mem::zeroed();
    let mut MappedAddress: PVOID = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    DPRINT1!(
        "StorPortGetDeviceBase(%p %lu %lu 0x%I64x %lu %u)\n",
        HwDeviceExtension,
        BusType,
        SystemIoBusNumber,
        IoAddress.QuadPart,
        NumberOfBytes,
        InIoSpace as core::ffi::c_uint
    );

    /* Get the miniport extension */
    MiniportExtension = CONTAINING_RECORD!(
        HwDeviceExtension,
        MINIPORT_DEVICE_EXTENSION,
        HwDeviceExtension
    );
    DPRINT1!(
        "HwDeviceExtension %p  MiniportExtension %p\n",
        HwDeviceExtension,
        MiniportExtension
    );

    if TranslateResourceListAddress(
        (*(*MiniportExtension).Miniport).DeviceExtension as PFDO_DEVICE_EXTENSION,
        BusType,
        SystemIoBusNumber,
        IoAddress,
        NumberOfBytes,
        InIoSpace,
        core::ptr::addr_of_mut!(TranslatedAddress),
    ) == 0
    {
        DPRINT1!("Checkpoint!\n");
        return core::ptr::null_mut();
    }

    DPRINT1!(
        "Translated Address: 0x%I64x\n",
        TranslatedAddress.QuadPart
    );

    /* In I/O space */
    if InIoSpace != 0 {
        DPRINT1!(
            "Translated Address: %p\n",
            TranslatedAddress.QuadPart as ULONG_PTR as PVOID
        );
        return TranslatedAddress.QuadPart as ULONG_PTR as PVOID;
    }

    /* In memory space */
    MappedAddress = MmMapIoSpace(
        TranslatedAddress,
        NumberOfBytes as SIZE_T,
        (FALSE) as MEMORY_CACHING_TYPE
    );
    DPRINT1!("Mapped Address: %p\n", MappedAddress);

    Status = AllocateAddressMapping(
        core::ptr::addr_of_mut!(
            (*(*(*MiniportExtension).Miniport).DeviceExtension).MappedAddressList
        ),
        IoAddress,
        MappedAddress,
        NumberOfBytes,
        SystemIoBusNumber,
    );
    if !NT_SUCCESS!(Status) {
        DPRINT1!("Checkpoint!\n");
        MappedAddress = core::ptr::null_mut();
    }

    DPRINT1!("Mapped Address: %p\n", MappedAddress);
    return MappedAddress;
}

#[no_mangle]
pub unsafe extern "system" fn StorPortGetLogicalUnit(
    HwDeviceExtension: PVOID,
    PathId: UCHAR,
    TargetId: UCHAR,
    Lun: UCHAR,
) -> PVOID {
    DPRINT1!("StorPortGetLogicalUnit()\n");
    UNIMPLEMENTED!();
    return core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "system" fn StorPortGetPhysicalAddress(
    HwDeviceExtension: PVOID,
    Srb: PSCSI_REQUEST_BLOCK,
    VirtualAddress: PVOID,
    Length: *mut ULONG,
) -> STOR_PHYSICAL_ADDRESS {
    let mut MiniportExtension: PMINIPORT_DEVICE_EXTENSION;
    let mut DeviceExtension: PFDO_DEVICE_EXTENSION;
    let mut PhysicalAddress: STOR_PHYSICAL_ADDRESS = core::mem::zeroed();
    let mut Offset: ULONG_PTR;

    DPRINT!(
        "StorPortGetPhysicalAddress(%p %p %p %p)\n",
        HwDeviceExtension,
        Srb,
        VirtualAddress,
        Length
    );

    /* Get the miniport extension */
    MiniportExtension = CONTAINING_RECORD!(
        HwDeviceExtension,
        MINIPORT_DEVICE_EXTENSION,
        HwDeviceExtension
    );
    DPRINT!(
        "HwDeviceExtension %p  MiniportExtension %p\n",
        HwDeviceExtension,
        MiniportExtension
    );

    DeviceExtension = (*(*MiniportExtension).Miniport).DeviceExtension as PFDO_DEVICE_EXTENSION;

    /* Inside of the uncached extension? */
    if (VirtualAddress as ULONG_PTR >= (*DeviceExtension).UncachedExtensionVirtualBase as ULONG_PTR)
        && (VirtualAddress as ULONG_PTR
            <= ((*DeviceExtension).UncachedExtensionVirtualBase as ULONG_PTR)
                .wrapping_add((*DeviceExtension).UncachedExtensionSize as ULONG_PTR))
    {
        Offset = (VirtualAddress as ULONG_PTR)
            .wrapping_sub((*DeviceExtension).UncachedExtensionVirtualBase as ULONG_PTR);

        PhysicalAddress.QuadPart = ((*DeviceExtension).UncachedExtensionPhysicalBase.QuadPart
            as ULONG_PTR)
            .wrapping_add(Offset) as LONGLONG;
        *Length =
            ((*DeviceExtension).UncachedExtensionSize as ULONG_PTR).wrapping_sub(Offset) as ULONG;

        return PhysicalAddress;
    }

    // FIXME

    PhysicalAddress = MmGetPhysicalAddress(VirtualAddress);
    *Length = 1;
    //    UNIMPLEMENTED;

    //    *Length = 0;
    //    PhysicalAddress.QuadPart = (LONGLONG)0;

    return PhysicalAddress;
}

#[no_mangle]
pub unsafe extern "system" fn StorPortGetScatterGatherList(
    HwDeviceExtension: PVOID,
    Srb: PSCSI_REQUEST_BLOCK,
) -> PSTOR_SCATTER_GATHER_LIST {
    UNREFERENCED_PARAMETER!(HwDeviceExtension);
    let mut RequestReference: PQUEUED_REQUEST_REFERENCE;

    DPRINT!("StorPortGetScatterGatherList(%p %p)\n", HwDeviceExtension, Srb);
    RequestReference = (*Srb).OriginalRequest as PQUEUED_REQUEST_REFERENCE;

    return (*RequestReference).ScatterGatherList;
}

#[no_mangle]
pub unsafe extern "system" fn StorPortGetSrb(
    DeviceExtension: PVOID,
    PathId: UCHAR,
    TargetId: UCHAR,
    Lun: UCHAR,
    QueueTag: LONG,
) -> PSCSI_REQUEST_BLOCK {
    DPRINT!("StorPortGetSrb()\n");
    return core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "system" fn StorPortGetUncachedExtension(
    HwDeviceExtension: PVOID,
    ConfigInfo: PPORT_CONFIGURATION_INFORMATION,
    NumberOfBytes: ULONG,
) -> PVOID {
    let mut MiniportExtension: PMINIPORT_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut DeviceExtension: PFDO_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut LowestAddress: PHYSICAL_ADDRESS = core::mem::zeroed();
    let mut HighestAddress: PHYSICAL_ADDRESS = core::mem::zeroed();
    let mut Alignment: PHYSICAL_ADDRESS = core::mem::zeroed();
    let mut DeviceDescription: DEVICE_DESCRIPTION = core::mem::zeroed();
    let mut NumberOfMapRegisters: ULONG = 0;

    DPRINT1!(
        "StorPortGetUncachedExtension(%p %p %lu)\n",
        HwDeviceExtension,
        ConfigInfo,
        NumberOfBytes
    );

    /* Get the miniport extension */
    MiniportExtension = (HwDeviceExtension as ULONG_PTR).wrapping_sub(
        core::mem::offset_of!(MINIPORT_DEVICE_EXTENSION, HwDeviceExtension) as ULONG_PTR,
    ) as PMINIPORT_DEVICE_EXTENSION;
    DPRINT1!(
        "HwDeviceExtension %p  MiniportExtension %p\n",
        HwDeviceExtension,
        MiniportExtension
    );

    DeviceExtension = (*(*MiniportExtension).Miniport).DeviceExtension as PFDO_DEVICE_EXTENSION;

    /* Return the uncached extension base address if we already have one */
    if !(*DeviceExtension).UncachedExtensionVirtualBase.is_null() {
        return (*DeviceExtension).UncachedExtensionVirtualBase;
    }

    // FIXME: Set DMA stuff here?
    DeviceDescription.Version = DEVICE_DESCRIPTION_VERSION as ULONG;
    DeviceDescription.Master = (*ConfigInfo).Master;
    DeviceDescription.ScatterGather = (*ConfigInfo).ScatterGather;
    DeviceDescription.DemandMode = (*ConfigInfo).DemandMode;
    DeviceDescription.Dma32BitAddresses = (*ConfigInfo).Dma32BitAddresses;
    DeviceDescription.BusNumber = (*ConfigInfo).SystemIoBusNumber;
    DeviceDescription.DmaChannel = (*ConfigInfo).DmaChannel;
    DeviceDescription.InterfaceType = (*ConfigInfo).AdapterInterfaceType;
    DeviceDescription.DmaWidth = (*ConfigInfo).DmaWidth;
    DeviceDescription.DmaSpeed = (*ConfigInfo).DmaSpeed;
    DeviceDescription.MaximumLength = (*ConfigInfo).MaximumTransferLength;
    DeviceDescription.DmaPort = (*ConfigInfo).DmaPort;

    (*DeviceExtension).DmaAdapter = IoGetDmaAdapter(
        (*DeviceExtension).PhysicalDevice,
        core::ptr::addr_of_mut!(DeviceDescription),
        core::ptr::addr_of_mut!(NumberOfMapRegisters),
    );

    NT_ASSERT!(!(*DeviceExtension).DmaAdapter.is_null());

    /* FIXME: AdapterInterfaceType is exposed here and I think it's convenient to initialize
       HwInitData in FDO extension right here. Needs review. */
    (*DeviceExtension).HwInitData = PortGetDriverInitData(
        (*DeviceExtension).DriverExtension,
        DeviceDescription.InterfaceType,
    );
    NT_ASSERT!(!(*DeviceExtension).HwInitData.is_null());

    /* Allocate the uncached extension */
    Alignment.QuadPart = 0 as LONGLONG;
    LowestAddress.QuadPart = 0 as LONGLONG;
    HighestAddress.QuadPart = 0x00000000FFFFFFFF as LONGLONG;
    (*DeviceExtension).UncachedExtensionVirtualBase = MmAllocateContiguousMemorySpecifyCache(
        NumberOfBytes as SIZE_T,
        LowestAddress,
        HighestAddress,
        Alignment,
        MmCached,
    );
    if (*DeviceExtension).UncachedExtensionVirtualBase.is_null() {
        return core::ptr::null_mut();
    }

    (*DeviceExtension).UncachedExtensionPhysicalBase =
        MmGetPhysicalAddress((*DeviceExtension).UncachedExtensionVirtualBase);
    (*DeviceExtension).UncachedExtensionSize = NumberOfBytes;

    return (*DeviceExtension).UncachedExtensionVirtualBase;
}

#[no_mangle]
pub unsafe extern "system" fn StorPortGetVirtualAddress(
    HwDeviceExtension: PVOID,
    PhysicalAddress: STOR_PHYSICAL_ADDRESS,
) -> PVOID {
    DPRINT1!(
        "StorPortGetVirtualAddress(%p %I64x)\n",
        HwDeviceExtension,
        PhysicalAddress.QuadPart
    );
    UNIMPLEMENTED!();
    return core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "system" fn StorPortInitialize(
    Argument1: PVOID,
    Argument2: PVOID,
    HwInitializationData: *mut _HW_INITIALIZATION_DATA,
    HwContext: PVOID,
) -> NTSTATUS {
    let DriverObject: PDRIVER_OBJECT = Argument1 as PDRIVER_OBJECT;
    let RegistryPath: PUNICODE_STRING = Argument2 as PUNICODE_STRING;
    let mut DriverObjectExtension: PDRIVER_OBJECT_EXTENSION = core::ptr::null_mut();
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    DPRINT1!(
        "StorPortInitialize(%p %p %p %p)\n",
        Argument1,
        Argument2,
        HwInitializationData,
        HwContext
    );

    DPRINT1!(
        "HwInitializationDataSize: %lu\n",
        (*HwInitializationData).HwInitializationDataSize
    );
    DPRINT1!(
        "AdapterInterfaceType: %u\n",
        (*HwInitializationData).AdapterInterfaceType
    );
    DPRINT1!("HwInitialize: %p\n", (*HwInitializationData).HwInitialize);
    DPRINT1!("HwStartIo: %p\n", (*HwInitializationData).HwStartIo);
    DPRINT1!("HwInterrupt: %p\n", (*HwInitializationData).HwInterrupt);
    DPRINT1!("HwFindAdapter: %p\n", (*HwInitializationData).HwFindAdapter);
    DPRINT1!("HwResetBus: %p\n", (*HwInitializationData).HwResetBus);
    DPRINT1!("HwDmaStarted: %p\n", (*HwInitializationData).HwDmaStarted);
    DPRINT1!("HwAdapterState: %p\n", (*HwInitializationData).HwAdapterState);
    DPRINT1!(
        "DeviceExtensionSize: %lu\n",
        (*HwInitializationData).DeviceExtensionSize
    );
    DPRINT1!(
        "SpecificLuExtensionSize: %lu\n",
        (*HwInitializationData).SpecificLuExtensionSize
    );
    DPRINT1!(
        "SrbExtensionSize: %lu\n",
        (*HwInitializationData).SrbExtensionSize
    );
    DPRINT1!(
        "NumberOfAccessRanges: %lu\n",
        (*HwInitializationData).NumberOfAccessRanges
    );

    /* Check parameters */
    if DriverObject.is_null() || RegistryPath.is_null() || HwInitializationData.is_null() {
        DPRINT1!("Invalid parameter!\n");
        return STATUS_INVALID_PARAMETER;
    }

    /* Check initialization data */
    if (*HwInitializationData).HwInitializationDataSize < (core::mem::size_of::<HW_INITIALIZATION_DATA>()) as ULONG
        || (*HwInitializationData).HwInitialize.is_none()
        || (*HwInitializationData).HwStartIo.is_none()
        || (*HwInitializationData).HwFindAdapter.is_none()
        || (*HwInitializationData).HwResetBus.is_none()
    {
        DPRINT1!("Revision mismatch!\n");
        return STATUS_REVISION_MISMATCH;
    }

    DriverObjectExtension = IoGetDriverObjectExtension(
        DriverObject,
        DriverEntry as *const () as PVOID,
    ) as PDRIVER_OBJECT_EXTENSION;
    if DriverObjectExtension.is_null() {
        DPRINT1!("No driver object extension!\n");

        Status = IoAllocateDriverObjectExtension(
            DriverObject,
            DriverEntry as *const () as PVOID,
            (core::mem::size_of::<DRIVER_OBJECT_EXTENSION>()) as ULONG,
            core::ptr::addr_of_mut!(DriverObjectExtension) as *mut PVOID,
        );
        if !NT_SUCCESS!(Status) {
            DPRINT1!(
                "IoAllocateDriverObjectExtension() failed (Status 0x%08lx)\n",
                Status
            );
            return Status;
        }

        DPRINT1!("Driver object extension created!\n");

        /* Initialize the driver object extension */
        RtlZeroMemory!(
            DriverObjectExtension as PVOID,
            (core::mem::size_of::<DRIVER_OBJECT_EXTENSION>()) as usize,
        );

        (*DriverObjectExtension).ExtensionType = EXTENSION_TYPE_DriverExtension;
        (*DriverObjectExtension).DriverObject = DriverObject;

        InitializeListHead(core::ptr::addr_of_mut!(
            (*DriverObjectExtension).AdapterListHead
        ));
        KeInitializeSpinLock(core::ptr::addr_of_mut!(
            (*DriverObjectExtension).AdapterListLock
        ));

        InitializeListHead(core::ptr::addr_of_mut!(
            (*DriverObjectExtension).InitDataListHead
        ));

        /* Set handlers */
        (*(*DriverObject).DriverExtension).AddDevice = Some(PortAddDevice);
//        DriverObject->DriverStartIo = PortStartIo;
        (*DriverObject).DriverUnload =
            Some(PortUnload);
        (*DriverObject).MajorFunction[IRP_MJ_CREATE as usize] = Some(PortDispatchCreate);
        (*DriverObject).MajorFunction[IRP_MJ_CLOSE as usize] = Some(PortDispatchClose);
        (*DriverObject).MajorFunction[IRP_MJ_DEVICE_CONTROL as usize] = Some(PortDispatchDeviceControl);
        (*DriverObject).MajorFunction[IRP_MJ_SCSI as usize] = Some(PortDispatchScsi);
        (*DriverObject).MajorFunction[IRP_MJ_POWER as usize] = Some(PortDispatchPower);
        (*DriverObject).MajorFunction[IRP_MJ_SYSTEM_CONTROL as usize] = Some(PortDispatchSystemControl);
        (*DriverObject).MajorFunction[IRP_MJ_PNP as usize] = Some(PortDispatchPnp);
    }

    /* Add the initialzation data to the driver extension */
    Status = PortAddDriverInitData(
        DriverObjectExtension,
        HwInitializationData as PHW_INITIALIZATION_DATA,
    );

    DPRINT1!("StorPortInitialize() done (Status 0x%08lx)\n", Status);

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn StorPortLogError(
    HwDeviceExtension: PVOID,
    Srb: PSCSI_REQUEST_BLOCK,
    PathId: UCHAR,
    TargetId: UCHAR,
    Lun: UCHAR,
    ErrorCode: ULONG,
    UniqueId: ULONG,
) {
    DPRINT1!("ScsiPortLogError() called\n");
    DPRINT1!(
        "PathId: 0x%02x  TargetId: 0x%02x  Lun: 0x%02x  ErrorCode: 0x%08lx  UniqueId: 0x%08lx\n",
        PathId as core::ffi::c_uint,
        TargetId as core::ffi::c_uint,
        Lun as core::ffi::c_uint,
        ErrorCode,
        UniqueId
    );

    DPRINT1!("ScsiPortLogError() done\n");
}

#[no_mangle]
pub unsafe extern "system" fn StorPortMoveMemory(
    Destination: *mut core::ffi::c_void,
    Source: *const core::ffi::c_void,
    Length: u32,
) {
    core::ptr::copy(Source as *const u8, Destination as *mut u8, Length as usize);
}

#[no_mangle]
pub unsafe extern "C" fn StorPortNotification_v(
    NotificationType: SCSI_NOTIFICATION_TYPE,
    HwDeviceExtension: PVOID,
    ap_list: *mut ::core::ffi::c_void,
) {
    let mut MiniportExtension: PMINIPORT_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut FdoExtension_local: PFDO_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut HwPassiveInitRoutine: PHW_PASSIVE_INITIALIZE_ROUTINE = None;
    let mut ppExtendedFunctions: *mut PSTORPORT_EXTENDED_FUNCTIONS = core::ptr::null_mut();
    let mut RequestReference: PQUEUED_REQUEST_REFERENCE = core::ptr::null_mut();
    let mut Result: PBOOLEAN = core::ptr::null_mut();
    let mut Dpc: PSTOR_DPC = core::ptr::null_mut();
    let mut HwDpcRoutine: PHW_DPC_ROUTINE = None;
    let mut DpcArgument1: PVOID = core::ptr::null_mut();
    let mut DpcArgument2: PVOID = core::ptr::null_mut();

    let mut SpinLock: STOR_SPINLOCK = 0 as STOR_SPINLOCK;
    let mut LockContext: PVOID = core::ptr::null_mut();
    let mut LockHandle: PSTOR_LOCK_HANDLE = core::ptr::null_mut();
    let mut Srb: PSCSI_REQUEST_BLOCK = core::ptr::null_mut();

    let mut ap = varargs_rust::VaArgs::from_ptr(ap_list);

    DPRINT!("StorPortNotification(%x %p)\n", NotificationType, HwDeviceExtension);

    /* Get the miniport extension */
    if !HwDeviceExtension.is_null() {
        MiniportExtension = CONTAINING_RECORD!(
            HwDeviceExtension,
            MINIPORT_DEVICE_EXTENSION,
            HwDeviceExtension
        );
        DPRINT!(
            "HwDeviceExtension %p  MiniportExtension %p\n",
            HwDeviceExtension,
            MiniportExtension
        );

        FdoExtension_local = (*(*MiniportExtension).Miniport).DeviceExtension as PFDO_DEVICE_EXTENSION;
    }



    'sw: {
        match NotificationType {
            RequestComplete => {
                DPRINT!("RequestComplete\n");
                Srb = ap.arg::<PSCSI_REQUEST_BLOCK>();
                DPRINT!(
                    "Complete Notify - (%d:%d:%d) ReqRef %p Srb %p Irp %p Tag %x\n",
                    (*Srb).PathId,
                    (*Srb).TargetId,
                    (*Srb).Lun,
                    (*Srb).OriginalRequest,
                    Srb,
                    ((*Srb).OriginalRequest as PQUEUED_REQUEST_REFERENCE).read().Irp,
                    (*Srb).QueueTag
                );
                NT_ASSERT!(!(*Srb).OriginalRequest.is_null());
                RequestReference = (*Srb).OriginalRequest as PQUEUED_REQUEST_REFERENCE;

                /* Queue for completion */
                InterlockedPushEntrySList(
                    core::ptr::addr_of_mut!((*FdoExtension_local).CompletionList),
                    core::ptr::addr_of_mut!((*RequestReference).CompletionEntry)
                );

                /* Insert DPC for delayed completion */
                KeInsertQueueDpc(
                    core::ptr::addr_of_mut!((*(*FdoExtension_local).Device).Dpc),
                    core::ptr::null_mut(),
                    core::ptr::null_mut()
                );

                break 'sw;
            }

            GetExtendedFunctionTable => {
                DPRINT1!("GetExtendedFunctionTable\n");
                ppExtendedFunctions = ap.arg::<*mut PSTORPORT_EXTENDED_FUNCTIONS>();
                if !ppExtendedFunctions.is_null() {
                    *ppExtendedFunctions = core::ptr::null_mut(); /* FIXME: */
                }
                break 'sw;
            }

            EnablePassiveInitialization => {
                DPRINT1!("EnablePassiveInitialization\n");
                HwPassiveInitRoutine = core::mem::transmute::<*mut u8, PHW_PASSIVE_INITIALIZE_ROUTINE>(
                    ap.arg::<*mut u8>()
                );
                DPRINT1!("HwPassiveInitRoutine %p\n", HwPassiveInitRoutine);
                Result = ap.arg::<PBOOLEAN>();

                // FIXME: It is not guaranteed that enabling passive initialization will fail and return
                // FALSE when not called in driver's HwStorInitialize routine.
                // Docs: Remarks of https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/storport/nf-storport-storportenablepassiveinitialization
                *Result = FALSE;

                if !FdoExtension_local.is_null() && (*FdoExtension_local).HwPassiveInitRoutine.is_none() {
                    (*FdoExtension_local).HwPassiveInitRoutine = HwPassiveInitRoutine;
                    *Result = TRUE;
                }
                break 'sw;
            }

            InitializeDpc => {
                DPRINT!("InitializeDpc\n");
                Dpc = ap.arg::<PSTOR_DPC>();
                DPRINT!("Dpc %p\n", Dpc);
                HwDpcRoutine = core::mem::transmute::<*mut u8, PHW_DPC_ROUTINE>(
                    ap.arg::<*mut u8>()
                );
                DPRINT!("HwDpcRoutine %p\n", HwDpcRoutine);

                KeInitializeDpc(
                    core::ptr::addr_of_mut!((*Dpc).Dpc) as PRKDPC,
                    core::mem::transmute::<PHW_DPC_ROUTINE, PKDEFERRED_ROUTINE>(HwDpcRoutine),
                    core::ptr::addr_of_mut!(
                        (*(*FdoExtension_local).Miniport.MiniportExtension).HwDeviceExtension
                    ) as *mut UCHAR as PVOID
                );
                KeInitializeSpinLock(core::ptr::addr_of_mut!((*Dpc).Lock));
                break 'sw;
            }

            IssueDpc => {
                DPRINT!("IssueDpc\n");
                Dpc = ap.arg::<PSTOR_DPC>();
                DpcArgument1 = ap.arg::<PVOID>();
                DpcArgument2 = ap.arg::<PVOID>();
                DPRINT!("Dpc %p Arg1 %p Arg2 %p\n", Dpc, DpcArgument1, DpcArgument2);
                Result = ap.arg::<PBOOLEAN>();

                *Result = KeInsertQueueDpc(Dpc as PKDPC, DpcArgument1, DpcArgument2) as UCHAR;
                break 'sw;
            }

            AcquireSpinLock => {
                DPRINT!("AcquireSpinLock\n");
                SpinLock = ap.arg::<STOR_SPINLOCK>();
                DPRINT!("SpinLock %lu\n", SpinLock);
                LockContext = ap.arg::<PVOID>();
                DPRINT!("LockContext %p\n", LockContext);
                LockHandle = ap.arg::<PSTOR_LOCK_HANDLE>();
                DPRINT!("LockHandle %p\n", LockHandle);
                PortAcquireSpinLock(FdoExtension_local, SpinLock, LockContext, LockHandle);
                break 'sw;
            }

            ReleaseSpinLock => {
                DPRINT!("ReleaseSpinLock\n");
                LockHandle = ap.arg::<PSTOR_LOCK_HANDLE>();
                DPRINT!("LockHandle %p\n", LockHandle);
                PortReleaseSpinLock(FdoExtension_local, LockHandle);
                break 'sw;
            }

            RequestTimerCall => {
                let mut TimerRoutine: PHW_TIMER = None;
                let mut TimerValue: ULONG = 0;
                let mut TimerDueTime: LARGE_INTEGER = core::mem::zeroed();
                let mut MultiplyResult: NTSTATUS = 0;

                TimerRoutine = core::mem::transmute::<*mut u8, PHW_TIMER>(
                    ap.arg::<*mut u8>()
                );
                TimerValue = ap.arg::<ULONG>();
                NT_ASSERT!(TimerRoutine.is_some());

                /* Set timer if timer value != 0. Logic is exactly same as StorPortRequestTimer */
                if TimerValue != 0 {
                    MultiplyResult = RtlULongLongMult(
                        TimerValue as ULONGLONG,
                        10,
                        core::ptr::addr_of_mut!(TimerDueTime.QuadPart) as PULONGLONG
                    );
                    if !NT_SUCCESS!(MultiplyResult) {
                        break 'sw;
                    }

                    (*FdoExtension_local).TimerCallback = TimerRoutine;
                    KeSetTimer(
                        core::ptr::addr_of_mut!((*FdoExtension_local).Timer),
                        TimerDueTime,
                        core::ptr::addr_of_mut!((*FdoExtension_local).TimerDpc)
                    );
                } else {
                    /* Otherwise cancel the timer. */
                    KeCancelTimer(core::ptr::addr_of_mut!((*FdoExtension_local).Timer));
                }
                break 'sw;
            }

            _ => {
                DPRINT1!("Unsupported Notification %lx\n", NotificationType);
                break 'sw;
            }
        }
    }


}



#[no_mangle]
pub unsafe extern "system" fn StorPortPause(HwDeviceExtension: PVOID, TimeOut: ULONG) -> BOOLEAN {
    DPRINT1!("StorPortPause()\n");
    UNIMPLEMENTED!();
    return FALSE;
}

#[no_mangle]
pub unsafe extern "system" fn StorPortPauseDevice(
    HwDeviceExtension: PVOID,
    PathId: UCHAR,
    TargetId: UCHAR,
    Lun: UCHAR,
    TimeOut: ULONG,
) -> BOOLEAN {
    DPRINT1!("StorPortPauseDevice()\n");
    UNIMPLEMENTED!();
    return FALSE;
}

// storport.c: `#if defined(_M_AMD64)` -- the export exists on amd64 only (storport.spec
// forwards i386's to NTOSKRNL.KeQuerySystemTime). The translation dropped the #if and read
// `SharedSystemTime` as the old crate did (KUSER_SHARED_DATA at its user-mode address).
#[cfg(target_arch = "x86_64")]
#[no_mangle]
pub unsafe extern "system" fn StorPortQuerySystemTime(CurrentTime: PLARGE_INTEGER) {
    DPRINT1!("StorPortQuerySystemTime(%p)\n", CurrentTime);

    ntkrnl_sys::inline::ke::KeQuerySystemTime(CurrentTime as *mut _);
}

#[no_mangle]
pub unsafe extern "system" fn StorPortReady(HwDeviceExtension: PVOID) -> BOOLEAN {
    DPRINT1!("StorPortReady()\n");
    UNIMPLEMENTED!();
    return FALSE;
}

#[no_mangle]
pub unsafe extern "system" fn StorPortRegistryRead(
    HwDeviceExtension: PVOID,
    ValueName: PUCHAR,
    Global: ULONG,
    Type: ULONG,
    Buffer: PUCHAR,
    BufferLength: PULONG,
) -> BOOLEAN {
    DPRINT1!("StorPortRegistryRead()\n");
    UNIMPLEMENTED!();
    return FALSE;
}

#[no_mangle]
pub unsafe extern "system" fn StorPortRegistryWrite(
    HwDeviceExtension: PVOID,
    ValueName: PUCHAR,
    Global: ULONG,
    Type: ULONG,
    Buffer: PUCHAR,
    BufferLength: ULONG,
) -> BOOLEAN {
    DPRINT1!("StorPortRegistryWrite()\n");
    UNIMPLEMENTED!();
    return FALSE;
}

#[no_mangle]
pub unsafe extern "system" fn StorPortResume(HwDeviceExtension: PVOID) -> BOOLEAN {
    DPRINT1!("StorPortResume()\n");
    UNIMPLEMENTED!();
    return FALSE;
}

#[no_mangle]
pub unsafe extern "system" fn StorPortResumeDevice(
    HwDeviceExtension: PVOID,
    PathId: UCHAR,
    TargetId: UCHAR,
    Lun: UCHAR,
) -> BOOLEAN {
    DPRINT1!("StorPortResumeDevice()\n");
    UNIMPLEMENTED!();
    return FALSE;
}

#[no_mangle]
pub unsafe extern "system" fn StorPortSetBusDataByOffset(
    DeviceExtension: PVOID,
    BusDataType: ULONG,
    SystemIoBusNumber: ULONG,
    SlotNumber: ULONG,
    Buffer: PVOID,
    Offset: ULONG,
    Length: ULONG,
) -> ULONG {
    let mut MiniportExtension: PMINIPORT_DEVICE_EXTENSION;
    let mut Interface: PBUS_INTERFACE_STANDARD;
    let mut ReturnLength: ULONG;

    DPRINT1!("StorPortSetBusData(%p %lu %lu %lu %p %lu %lu)\n",
            DeviceExtension, BusDataType, SystemIoBusNumber, SlotNumber, Buffer, Offset, Length);

    MiniportExtension = (DeviceExtension as ULONG_PTR).wrapping_sub(
        core::mem::offset_of!(MINIPORT_DEVICE_EXTENSION, HwDeviceExtension) as ULONG_PTR,
    ) as *mut MINIPORT_DEVICE_EXTENSION as PMINIPORT_DEVICE_EXTENSION;
    DPRINT1!("DeviceExtension %p  MiniportExtension %p\n",
            DeviceExtension, MiniportExtension);

    Interface = core::ptr::addr_of_mut!(
        (*(*(*MiniportExtension).Miniport).DeviceExtension).BusInterface
    ) as PBUS_INTERFACE_STANDARD;

    ReturnLength = (*Interface).SetBusData.unwrap_unchecked()(
        (*Interface).Context,
        BusDataType,
        Buffer,
        Offset,
        Length,
    );
    DPRINT1!("ReturnLength: %lu\n", ReturnLength);

    return ReturnLength;
}

#[no_mangle]
pub unsafe extern "system" fn StorPortSetDeviceQueueDepth(
    HwDeviceExtension: PVOID,
    PathId: UCHAR,
    TargetId: UCHAR,
    Lun: UCHAR,
    Depth: ULONG,
) -> BOOLEAN {
    let mut FdoExtension: PFDO_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut PdoExtension: PPDO_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut LockHandle: KLOCK_QUEUE_HANDLE = core::mem::zeroed();

    DPRINT1!(
        "StorPortSetDeviceQueueDepth(%p %d %d %d %d)\n",
        HwDeviceExtension,
        PathId as core::ffi::c_uint,
        TargetId as core::ffi::c_uint,
        Lun as core::ffi::c_uint,
        Depth
    );

    FdoExtension = StorpGetMiniportFdo(HwDeviceExtension);
    PdoExtension = FdoFindLun(FdoExtension, PathId as ULONG, TargetId as ULONG, Lun as ULONG);

    /* 254 is pre-Windows-8 per LUN queue limit */
    if PdoExtension.is_null() || Depth > 254 {
        return FALSE;
    }

    KeAcquireInStackQueuedSpinLock(&raw mut (*FdoExtension).FlowControl.Lock, &raw mut LockHandle);
    (*PdoExtension).OutstandingRequestMax = Depth;
    KeReleaseInStackQueuedSpinLock(&raw mut LockHandle);

    DPRINT!(
        "Set Path %d Target %d Lun %d Queue depth to %d\n",
        PathId,
        TargetId,
        Lun,
        Depth
    );

    return FALSE;
}

#[no_mangle]
pub unsafe extern "system" fn StorPortStallExecution(Delay: ULONG) {
    KeStallExecutionProcessor(Delay);
}

#[no_mangle]
pub unsafe extern "system" fn StorPortSynchronizeAccess(
    HwDeviceExtension: PVOID,
    SynchronizedAccessRoutine: PSTOR_SYNCHRONIZED_ACCESS,
    Context: PVOID,
) {
    DPRINT1!("StorPortSynchronizeAccess()\n");
    UNIMPLEMENTED!();
}

#[no_mangle]
pub unsafe extern "system" fn StorPortValidateRange(
    HwDeviceExtension: PVOID,
    BusType: INTERFACE_TYPE,
    SystemIoBusNumber: ULONG,
    IoAddress: STOR_PHYSICAL_ADDRESS,
    NumberOfBytes: ULONG,
    InIoSpace: BOOLEAN,
) -> BOOLEAN {
    DPRINT1!("StorPortValidateRange()\n");
    return TRUE;
}
