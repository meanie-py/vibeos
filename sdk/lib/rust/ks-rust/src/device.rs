use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ddk::{DelayedWorkQueue, _KWAIT_REASON_Executive, IoAllocateAdapterChannel, IoAllocateWorkItem, IoForwardIrpSynchronously, IoFreeWorkItem, IoGetCurrentIrpStackLocation, IoQueueWorkItem, IoSkipCurrentIrpStackLocation, KeInitializeMutex, KeReleaseMutex, KeWaitForSingleObject, PADAPTER_OBJECT, PCM_RESOURCE_LIST, PDEVICE_OBJECT, PDRIVER_CONTROL, PIO_STACK_LOCATION, PIRP, PRKMUTEX, PowerDeviceD0, PowerSystemWorking, Unknown};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut vt_IKsDevice: IKsDeviceVtbl = unsafe {
    IKsDeviceVtbl {
        QueryInterface: Some(
            IKsDevice_fnQueryInterface
                as unsafe extern "C" fn(*mut IKsDevice, *const IID, *mut PVOID) -> NTSTATUS,
        ),
        AddRef: Some(IKsDevice_fnAddRef as unsafe extern "C" fn(*mut IKsDevice) -> ULONG),
        Release: Some(IKsDevice_fnRelease as unsafe extern "C" fn(*mut IKsDevice) -> ULONG),
        GetStruct: Some(IKsDevice_fnGetStruct as unsafe extern "C" fn(*mut IKsDevice) -> PKSDEVICE),
        InitializeObjectBag: Some(
            IKsDevice_fnInitializeObjectBag
                as unsafe extern "C" fn(*mut IKsDevice, PKSIOBJECT_BAG, PRKMUTEX) -> NTSTATUS,
        ),
        AcquireDevice: Some(
            IKsDevice_fnAcquireDevice as unsafe extern "C" fn(*mut IKsDevice) -> NTSTATUS,
        ),
        ReleaseDevice: Some(
            IKsDevice_fnReleaseDevice as unsafe extern "C" fn(*mut IKsDevice) -> NTSTATUS,
        ),
        GetAdapterObject: Some(
            IKsDevice_fnGetAdapterObject
                as unsafe extern "C" fn(
                    *mut IKsDevice,
                    *mut PADAPTER_OBJECT,
                    PULONG,
                    PULONG,
                ) -> NTSTATUS,
        ),
        AddPowerEntry: Some(
            IKsDevice_fnAddPowerEntry
                as unsafe extern "C" fn(
                    *mut IKsDevice,
                    *mut KSPOWER_ENTRY,
                    *mut IKsPowerNotify,
                ) -> NTSTATUS,
        ),
        RemovePowerEntry: Some(
            IKsDevice_fnRemovePowerEntry
                as unsafe extern "C" fn(*mut IKsDevice, *mut KSPOWER_ENTRY) -> NTSTATUS,
        ),
        PinStateChange: Some(
            IKsDevice_fnPinStateChange
                as unsafe extern "C" fn(*mut IKsDevice, KSPIN, PIRP, KSSTATE, KSSTATE) -> NTSTATUS,
        ),
        ArbitrateAdapterChannel: Some(
            IKsDevice_fnArbitrateAdapterChannel
                as unsafe extern "C" fn(*mut IKsDevice, ULONG, PDRIVER_CONTROL, PVOID) -> NTSTATUS,
        ),
        CheckIoCapability: Some(
            IKsDevice_fnCheckIoCapability
                as unsafe extern "C" fn(*mut IKsDevice, ULONG) -> NTSTATUS,
        ),
    }
};

#[repr(C)]
pub struct IKsDeviceVtbl {
    pub QueryInterface: unsafe extern "system" fn(*mut IKsDevice, *mut GUID, *mut *mut c_void) -> NTSTATUS,
    pub AddRef: unsafe extern "system" fn(*mut IKsDevice) -> ULONG,
    pub Release: unsafe extern "system" fn(*mut IKsDevice) -> ULONG,
    pub GetStruct: unsafe extern "system" fn(*mut IKsDevice) -> *mut c_void,
    pub InitializeObjectBag: unsafe extern "system" fn(*mut IKsDevice, ULONG, ULONG) -> NTSTATUS,
    pub AcquireDevice: unsafe extern "system" fn(*mut IKsDevice) -> NTSTATUS,
    pub ReleaseDevice: unsafe extern "system" fn(*mut IKsDevice) -> NTSTATUS,
    pub GetAdapterObject: unsafe extern "system" fn(*mut IKsDevice) -> PADAPTER_OBJECT,
    pub AddPowerEntry: unsafe extern "system" fn(*mut IKsDevice, *mut KSPOWER_ENTRY) -> NTSTATUS,
    pub RemovePowerEntry: unsafe extern "system" fn(*mut IKsDevice, *mut KSPOWER_ENTRY) -> NTSTATUS,
    pub PinStateChange: unsafe extern "system" fn(*mut IKsDevice, KSPIN_DATAFLOW, KSPIN_STATE, KSPIN_STATE) -> NTSTATUS,
    pub ArbitrateAdapterChannel: unsafe extern "system" fn(*mut IKsDevice, ULONG, ULONG, PDRIVER_CONTROL, *mut c_void) -> NTSTATUS,
    pub CheckIoCapability: unsafe extern "system" fn(*mut IKsDevice, *mut KSPIN_DATAFLOW, *mut KSPIN_STATE, *mut KSPIN_STATE) -> NTSTATUS,
}

pub type PIKSDEVICE = *mut IKsDevice;

#[no_mangle]
pub unsafe extern "system" fn IKsDevice_fnQueryInterface(
    iface: *mut IKsDevice,
    refiid: *const IID,
    Output: *mut PVOID,
) -> NTSTATUS {
    let This: PKSIDEVICE_HEADER = CONTAINING_RECORD(
        iface,
        KSIDEVICE_HEADER,
        BasicHeader.OuterUnknown,
    );

    if IsEqualGUIDAligned(refiid, &IID_IUnknown) {
        *Output = &mut (*This).BasicHeader.OuterUnknown as *mut _ as PVOID;
        _InterlockedIncrement(&mut (*This).ref_0);
        return STATUS_SUCCESS;
    }

    if !(*This).BasicHeader.ClientAggregate.is_null() {
        let Status = (*(*(*This).BasicHeader.ClientAggregate).lpVtbl).QueryInterface.unwrap()(
            (*This).BasicHeader.ClientAggregate as *mut IUnknown,
            refiid,
            Output,
        );

        if NT_SUCCESS(Status) {
            return Status;
        }
    }

    return STATUS_NOT_SUPPORTED;
}

#[no_mangle]
pub unsafe extern "system" fn IKsDevice_fnAddRef(iface: *mut IKsDevice) -> ULONG {
    let This: PKSIDEVICE_HEADER = core::mem::transmute(
        (iface as usize)
            .wrapping_sub(core::mem::offset_of!(KSIDEVICE_HEADER, BasicHeader.OuterUnknown))
            as *mut KSIDEVICE_HEADER,
    );
    InterlockedIncrement(&mut (*This).ref_0) as ULONG
}

#[no_mangle]
pub unsafe extern "system" fn IKsDevice_fnRelease(
    iface: *mut IKsDevice,
) -> ULONG {
    let This: PKSIDEVICE_HEADER = CONTAINING_RECORD(
        iface,
        KSIDEVICE_HEADER,
        BasicHeader.OuterUnknown,
    );
    InterlockedDecrement(&mut (*This).ref_0);
    return (*This).ref_0;
}

#[no_mangle]
pub unsafe extern "system" fn IKsDevice_fnGetStruct(
    iface: *mut IKsDevice,
) -> PKSDEVICE {
    let This: PKSIDEVICE_HEADER = CONTAINING_RECORD(
        iface,
        KSIDEVICE_HEADER,
        BasicHeader.OuterUnknown,
    );
    &mut (*This).KsDevice
}

#[no_mangle]
pub unsafe extern "system" fn IKsDevice_fnInitializeObjectBag(
    iface: *mut IKsDevice,
    Bag: PKSIOBJECT_BAG,
    Mutex: PRKMUTEX,
) -> NTSTATUS {
    let This: PKSIDEVICE_HEADER = CONTAINING_RECORD(
        iface as *mut u8,
        KSIDEVICE_HEADER,
        BasicHeader.OuterUnknown,
    ) as PKSIDEVICE_HEADER;

    let mut Mutex = Mutex;
    if Mutex.is_null() {
        Mutex = core::ptr::addr_of_mut!((*This).BagMutex) as PRKMUTEX;
    }

    (*Bag).BagMutex = Mutex;
    (*Bag).DeviceHeader = This as *mut core::ffi::c_void as PKSIDEVICE_HEADER;
    InitializeListHead(core::ptr::addr_of_mut!((*Bag).ObjectList));
    InsertTailList(
        core::ptr::addr_of_mut!((*This).ObjectBags),
        core::ptr::addr_of_mut!((*Bag).Entry),
    );

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn IKsDevice_fnAcquireDevice(
    iface: *mut IKsDevice,
) -> NTSTATUS {
    let This: PKSIDEVICE_HEADER = core::mem::transmute(
        (iface as usize)
            .wrapping_sub(core::mem::offset_of!(KSIDEVICE_HEADER, BasicHeader.OuterUnknown)),
    );
    KeWaitForSingleObject(
        &mut (*This).DeviceMutex as *mut _ as PVOID,
        _KWAIT_REASON_Executive,
        0i32 as KPROCESSOR_MODE,
        0u8,
        core::ptr::null_mut(),
    )
}

#[no_mangle]
pub unsafe extern "system" fn IKsDevice_fnReleaseDevice(
    iface: *mut IKsDevice,
) -> NTSTATUS {
    let this: PKSIDEVICE_HEADER = core::mem::transmute::<usize, *mut KSIDEVICE_HEADER>(
        (iface as usize)
            .wrapping_sub(
                &raw const (*core::ptr::null_mut::<KSIDEVICE_HEADER>())
                    .BasicHeader
                    .OuterUnknown as usize,
            ),
    );
    KeReleaseMutex(&raw mut (*this).DeviceMutex, 0u8) as NTSTATUS
}

#[no_mangle]
pub unsafe extern "system" fn IKsDevice_fnGetAdapterObject(
    iface: *mut IKsDevice,
    Object: *mut PADAPTER_OBJECT,
    MaxMappingsByteCount: *mut u32,
    MappingTableStride: *mut u32,
) -> NTSTATUS {
    let This: PKSIDEVICE_HEADER = core::mem::transmute(
        (iface as usize)
            .wrapping_sub(core::mem::offset_of!(KSIDEVICE_HEADER, BasicHeader.OuterUnknown))
    );
    *Object = (*This).AdapterObject;
    *MaxMappingsByteCount = (*This).MaxMappingsByteCount;
    *MappingTableStride = (*This).MappingTableStride;
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn IKsDevice_fnAddPowerEntry(
    iface: *mut IKsDevice,
    Entry: *mut KSPOWER_ENTRY,
    Notify: *mut IKsPowerNotify,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn IKsDevice_fnRemovePowerEntry(
    iface: *mut IKsDevice,
    Entry: *mut KSPOWER_ENTRY,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn IKsDevice_fnPinStateChange(
    iface: *mut IKsDevice,
    Pin: KSPIN,
    Irp: PIRP,
    OldState: KSSTATE,
    NewState: KSSTATE,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn IKsDevice_fnArbitrateAdapterChannel(
    iface: *mut IKsDevice,
    NumberOfMapRegisters: ULONG,
    ExecutionRoutine: PDRIVER_CONTROL,
    Context: PVOID,
) -> NTSTATUS {
    let This: PKSIDEVICE_HEADER = CONTAINING_RECORD(iface, KSIDEVICE_HEADER, BasicHeader.OuterUnknown);
    let Status: NTSTATUS;

    Status = IoAllocateAdapterChannel(
        (*This).AdapterObject,
        (*This).KsDevice.FunctionalDeviceObject,
        NumberOfMapRegisters,
        ExecutionRoutine,
        Context,
    );

    Status
}

#[no_mangle]
pub unsafe extern "system" fn IKsDevice_fnCheckIoCapability(
    iface: *mut IKsDevice,
    Unknown: ULONG,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn IKsDevice_PnpPostStart(
    DeviceObject: PDEVICE_OBJECT,
    Context: PVOID,
) {
    let Ctx = Context as PPNP_POSTSTART_CONTEXT;
    let Status = ((*(*(*(*Ctx).DeviceHeader).KsDevice.Descriptor).Dispatch).PostStart)(
        &mut (*(*Ctx).DeviceHeader).KsDevice,
    );

    if !NT_SUCCESS(Status) {
        (*(*Ctx).DeviceHeader).TargetState = KSTARGET_STATE_DISABLED;
    } else {
        (*(*Ctx).DeviceHeader).TargetState = KSTARGET_STATE_ENABLED;
        let _ = KspSetFilterFactoriesState((*Ctx).DeviceHeader, 1u8);
    }

    IoFreeWorkItem((*Ctx).WorkItem);
    FreeItem(Ctx as PVOID);
}

#[no_mangle]
pub unsafe extern "system" fn IKsDevice_PnpStartDevice(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let IoStack: PIO_STACK_LOCATION;
    let DeviceExtension: PDEVICE_EXTENSION;
    let DeviceHeader: PKSIDEVICE_HEADER;
    let mut Ctx: PPNP_POSTSTART_CONTEXT = core::ptr::null_mut();
    let mut Status: NTSTATUS;
    let TranslatedResourceList: PCM_RESOURCE_LIST;
    let UntranslatedResourceList: PCM_RESOURCE_LIST;

    IoStack = IoGetCurrentIrpStackLocation(Irp);
    DeviceExtension = (*DeviceObject).DeviceExtension as PDEVICE_EXTENSION;
    DeviceHeader = (*DeviceExtension).DeviceHeader;

    if IoForwardIrpSynchronously((*DeviceHeader).KsDevice.NextDeviceObject, Irp) == 0 {
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_UNSUCCESSFUL;
    }
    Status = (*Irp).IoStatus.__bindgen_anon_1.Status;

    if !NT_SUCCESS(Status) {
        CompleteRequest(Irp, IO_NO_INCREMENT as i8);
        return Status;
    }

    TranslatedResourceList = (*IoStack).Parameters.StartDevice.AllocatedResourcesTranslated;
    UntranslatedResourceList = (*IoStack).Parameters.StartDevice.AllocatedResources;

    if !(*DeviceHeader).KsDevice.Descriptor.is_null() {
        if !(*(*DeviceHeader).KsDevice.Descriptor).Dispatch.is_null() {
            if (*(*(*DeviceHeader).KsDevice.Descriptor).Dispatch).Start.is_some() {
                Status = (*(*(*DeviceHeader).KsDevice.Descriptor).Dispatch)
                    .Start
                    .expect("non-null function pointer")(
                    &mut (*DeviceHeader).KsDevice,
                    Irp,
                    TranslatedResourceList,
                    UntranslatedResourceList,
                );

                if !NT_SUCCESS(Status) {
                    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
                    CompleteRequest(Irp, IO_NO_INCREMENT as i8);
                    return Status;
                }

                (*DeviceHeader).KsDevice.Started = 1u8;
            }

            if (*(*(*DeviceHeader).KsDevice.Descriptor).Dispatch).PostStart.is_some() {
                Ctx = AllocateItem(
                    NonPagedPool,
                    core::mem::size_of::<PNP_POSTSTART_CONTEXT>() as usize,
                ) as PPNP_POSTSTART_CONTEXT;
                if Ctx.is_null() {
                    Status = STATUS_INSUFFICIENT_RESOURCES;
                } else {
                    (*Ctx).WorkItem = IoAllocateWorkItem(DeviceObject);
                    if (*Ctx).WorkItem.is_null() {
                        FreeItem(Ctx as *mut _);
                        Ctx = core::ptr::null_mut();
                        Status = STATUS_INSUFFICIENT_RESOURCES;
                    } else {
                        (*Ctx).DeviceHeader = DeviceHeader;
                    }
                }
            } else {
                (*DeviceHeader).TargetState = KSTARGET_STATE_ENABLED;
                Status = KspSetFilterFactoriesState(DeviceHeader, 1u8);
            }
        } else {
            (*DeviceHeader).KsDevice.Started = 1u8;
        }
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    CompleteRequest(Irp, IO_NO_INCREMENT as i8);

    if !Ctx.is_null() {
        IoQueueWorkItem(
            (*Ctx).WorkItem,
            Some(IKsDevice_PnpPostStart as unsafe extern "system" fn(PDEVICE_OBJECT, *mut _) -> ()),
            DelayedWorkQueue,
            Ctx as *mut _,
        );
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn IKsDevice_Pnp(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let IoStack: PIO_STACK_LOCATION;
    let DeviceExtension: PDEVICE_EXTENSION;
    let DeviceHeader: PKSIDEVICE_HEADER;
    let Dispatch: PKSDEVICE_DISPATCH;
    let Status: NTSTATUS;

    IoStack = IoGetCurrentIrpStackLocation(Irp);
    DeviceExtension = (*DeviceObject).DeviceExtension as PDEVICE_EXTENSION;
    DeviceHeader = (*DeviceExtension).DeviceHeader;

    if !(*DeviceHeader).KsDevice.Descriptor.is_null() && !(*(*DeviceHeader).KsDevice.Descriptor).Dispatch.is_null() {
        Dispatch = (*(*DeviceHeader).KsDevice.Descriptor).Dispatch as PKSDEVICE_DISPATCH;
    } else {
        Dispatch = core::ptr::null_mut();
    }

    match (*IoStack).MinorFunction as u32 {
        IRP_MN_START_DEVICE => {
            return IKsDevice_PnpStartDevice(DeviceObject, Irp);
        }
        IRP_MN_QUERY_STOP_DEVICE => {
            Status = STATUS_SUCCESS;
            if !Dispatch.is_null() {
                if let Some(QueryStop) = (*Dispatch).QueryStop {
                    Status = QueryStop(&mut (*DeviceHeader).KsDevice, Irp);
                }
            }

            if !NT_SUCCESS(Status) {
                (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
                CompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);
                return Status;
            }

            IoSkipCurrentIrpStackLocation(Irp);
            return IoCallDriver((*DeviceHeader).KsDevice.NextDeviceObject, Irp);
        }
        IRP_MN_REMOVE_DEVICE => {
            if !Dispatch.is_null() {
                if let Some(Remove) = (*Dispatch).Remove {
                    Remove(&mut (*DeviceHeader).KsDevice, Irp);
                }
            }

            IoSkipCurrentIrpStackLocation(Irp);
            return IoCallDriver((*DeviceHeader).KsDevice.NextDeviceObject, Irp);
        }
        IRP_MN_QUERY_INTERFACE => {
            Status = STATUS_UNSUCCESSFUL;
            if !Dispatch.is_null() {
                if let Some(QueryInterface) = (*Dispatch).QueryInterface {
                    Status = QueryInterface(&mut (*DeviceHeader).KsDevice, Irp);
                }
            }

            if NT_SUCCESS(Status) {
                (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
                CompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);
                return Status;
            }

            IoSkipCurrentIrpStackLocation(Irp);
            return IoCallDriver((*DeviceHeader).KsDevice.NextDeviceObject, Irp);
        }
        IRP_MN_QUERY_DEVICE_RELATIONS => {
            IoSkipCurrentIrpStackLocation(Irp);
            return IoCallDriver((*DeviceHeader).KsDevice.NextDeviceObject, Irp);
        }
        IRP_MN_FILTER_RESOURCE_REQUIREMENTS => {
            Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
            CompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);
            return Status;
        }
        IRP_MN_QUERY_RESOURCE_REQUIREMENTS => {
            IoSkipCurrentIrpStackLocation(Irp);
            return IoCallDriver((*DeviceHeader).KsDevice.NextDeviceObject, Irp);
        }
        _ => {
            IoSkipCurrentIrpStackLocation(Irp);
            return IoCallDriver((*DeviceHeader).KsDevice.NextDeviceObject, Irp);
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn IKsDevice_Power(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    (*Irp).IoStatus.Information = 0;
    CompleteRequest(Irp, IO_NO_INCREMENT as i8);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn IKsDevice_Create(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut CreateItemEntry: PCREATE_ITEM_ENTRY = core::ptr::null_mut();
    let IoStack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let DeviceExtension: PDEVICE_EXTENSION = (*DeviceObject).DeviceExtension as PDEVICE_EXTENSION;
    let DeviceHeader: PKSIDEVICE_HEADER = (*DeviceExtension).DeviceHeader;
    let mut ObjectHeader: PKSIOBJECT_HEADER = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    if (*(*IoStack).FileObject).FileName.Buffer.is_null() {
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
        IoCompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);
        return STATUS_SUCCESS;
    }

    IKsDevice_fnAcquireDevice(&mut (*DeviceHeader).BasicHeader.OuterUnknown as *mut IKsDevice);

    if (*(*IoStack).FileObject).RelatedFileObject.is_null() {
        Status = FindMatchingCreateItem(
            &mut (*DeviceHeader).ItemList,
            &mut (*(*IoStack).FileObject).FileName,
            &mut CreateItemEntry,
        );
    } else {
        ObjectHeader = (*(*(*IoStack).FileObject).RelatedFileObject).FsContext2 as PKSIOBJECT_HEADER;
        Status = FindMatchingCreateItem(
            &mut (*ObjectHeader).ItemList,
            &mut (*(*IoStack).FileObject).FileName,
            &mut CreateItemEntry,
        );
    }

    if NT_SUCCESS(Status) {
        *((&mut (*Irp).Tail.Overlay.__bindgen_anon_1.__bindgen_anon_1.DriverContext as *mut PVOID).offset(0) as *mut PKSOBJECT_CREATE_ITEM) = (*CreateItemEntry).CreateItem;
        Status = ((*(*CreateItemEntry).CreateItem).Create.expect("non-null function pointer"))(
            DeviceObject,
            Irp,
        );
        if NT_SUCCESS(Status) {
            InterlockedIncrement(&mut (*CreateItemEntry).ReferenceCount);
        }
    }

    IKsDevice_fnReleaseDevice(&mut (*DeviceHeader).BasicHeader.OuterUnknown as *mut IKsDevice);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn KsInitializeDevice(
    FunctionalDeviceObject: PDEVICE_OBJECT,
    PhysicalDeviceObject: PDEVICE_OBJECT,
    NextDeviceObject: PDEVICE_OBJECT,
    Descriptor: *const KSDEVICE_DESCRIPTOR,
) -> NTSTATUS {
    let DeviceExtension: PDEVICE_EXTENSION;
    let Header: PKSIDEVICE_HEADER;
    let Index: ULONG;
    let Bag: PKSIOBJECT_BAG;
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    DeviceExtension = (*FunctionalDeviceObject).DeviceExtension as PDEVICE_EXTENSION;

    Status = KsAllocateDeviceHeader(
        &mut (*DeviceExtension).DeviceHeader as *mut KSDEVICE_HEADER,
        0,
        core::ptr::null_mut(),
    );

    Header = (*DeviceExtension).DeviceHeader;

    if !NT_SUCCESS(Status) {
        return Status;
    }

    (*Header).BasicHeader.OuterUnknown = &vt_IKsDevice as *const _ as PUNKNOWN;
    (*Header).ref = 1;

    (*Header).KsDevice.Bag = AllocateItem(
        NonPagedPool,
        core::mem::size_of::<KSIOBJECT_BAG>(),
    ) as KSOBJECT_BAG;
    if (*Header).KsDevice.Bag.is_null() {
        KsFreeDeviceHeader(&mut (*DeviceExtension).DeviceHeader as *mut KSDEVICE_HEADER);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    KeInitializeMutex(&mut (*Header).BagMutex, 0);
    KeInitializeMutex(&mut (*Header).DeviceMutex, 0);

    Bag = (*Header).KsDevice.Bag as PKSIOBJECT_BAG;
    (*Bag).BagMutex = &mut (*Header).BagMutex as PRKMUTEX;
    InitializeListHead(&mut (*Header).ObjectBags);
    InitializeListHead(&mut (*Bag).ObjectList);
    (*Bag).DeviceHeader = Header as PVOID;

    InsertTailList(&mut (*Header).ObjectBags, &mut (*Bag).Entry);

    (*Header).KsDevice.FunctionalDeviceObject = FunctionalDeviceObject;
    (*Header).KsDevice.PhysicalDeviceObject = PhysicalDeviceObject;
    (*Header).KsDevice.NextDeviceObject = NextDeviceObject;
    (*Header).KsDevice.Descriptor = Descriptor;
    (*Header).KsDevice.SystemPowerState = PowerSystemWorking;
    (*Header).KsDevice.DevicePowerState = PowerDeviceD0;
    (*Header).KsDevice.Started = 0u8;
    (*Header).KsDevice.Context = core::ptr::null_mut();
    KsSetDevicePnpAndBaseObject(Header, PhysicalDeviceObject, NextDeviceObject);

    if !Descriptor.is_null() {
        let mut Index: ULONG = 0;
        while Index < (*Descriptor).FilterDescriptorsCount {
            Status = KspCreateFilterFactory(
                FunctionalDeviceObject,
                *(*Descriptor).FilterDescriptors.offset(Index as isize),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                0,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );

            if !NT_SUCCESS(Status) {
                return Status;
            }
            Index = Index.wrapping_add(1);
        }

        if !(*Descriptor).Dispatch.is_null() && !(*((*Descriptor).Dispatch)).Add.is_null() {
            Status = ((*((*Descriptor).Dispatch)).Add)(&mut (*Header).KsDevice);
        }

        (*Header).KsDevice.Descriptor = Descriptor;
    }

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn KsReferenceSoftwareBusObject(
    Header: KSDEVICE_HEADER,
) -> NTSTATUS {
    let DeviceHeader = Header as PKSIDEVICE_HEADER;
    let Device = (*DeviceHeader).BasicHeader.OuterUnknown as *mut IKsDevice;

    if !Device.is_null() {
        (*(*Device).lpVtbl)
            .AddRef
            .expect("non-null function pointer")(Device);
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn KsReferenceBusObject(
    Header: KSDEVICE_HEADER,
) -> NTSTATUS {
    let DeviceHeader = Header as PKSIDEVICE_HEADER;
    let Device = (*DeviceHeader).BasicHeader.OuterUnknown as *mut IKsDevice;

    if !Device.is_null() {
        ((*(*Device).lpVtbl).AddRef.expect("non-null function pointer"))(Device);
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn KsDereferenceBusObject(Header: KSDEVICE_HEADER) {
    let DeviceHeader = Header as PKSIDEVICE_HEADER;
    let Device = (*DeviceHeader).BasicHeader.OuterUnknown as *mut IKsDevice;
    if !Device.is_null() {
        ((*(*Device).lpVtbl).Release)(Device);
    }
}

#[no_mangle]
pub unsafe extern "system" fn KsDereferenceSoftwareBusObject(
    Header: KSDEVICE_HEADER,
) {
    let DeviceHeader = Header as PKSIDEVICE_HEADER;
    let Device = (*DeviceHeader).BasicHeader.OuterUnknown as *mut IKsDevice;

    if !Device.is_null() {
        (*(*Device).lpVtbl)
            .Release
            .expect("non-null function pointer")(Device);
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS Kernel Streaming
//  * FILE:            drivers/ksfilter/ks/device.c
//  * PURPOSE:         KS IKsDevice interface functions
//  * PROGRAMMER:      Johannes Anderwald
//  */
//
// #include "precomp.h"
//
// #define NDEBUG
// #include <debug.h>
//
// NTSTATUS
// NTAPI
// IKsDevice_fnQueryInterface(
//     IN IKsDevice * iface,
//     REFIID refiid,
//     PVOID* Output)
// {
//     NTSTATUS Status;
//     PKSIDEVICE_HEADER This = (PKSIDEVICE_HEADER)CONTAINING_RECORD(iface, KSIDEVICE_HEADER, BasicHeader.OuterUnknown);
//
//     if (IsEqualGUIDAligned(refiid, &IID_IUnknown))
//     {
//         *Output = &This->BasicHeader.OuterUnknown;
//         _InterlockedIncrement(&This->ref);
//         return STATUS_SUCCESS;
//     }
//
//     if (This->BasicHeader.ClientAggregate)
//     {
//          /* using client aggregate */
//          Status = This->BasicHeader.ClientAggregate->lpVtbl->QueryInterface(This->BasicHeader.ClientAggregate, refiid, Output);
//
//          if (NT_SUCCESS(Status))
//          {
//              /* client aggregate supports interface */
//              return Status;
//          }
//     }
//
//     DPRINT("IKsDevice_fnQueryInterface no interface\n");
//     return STATUS_NOT_SUPPORTED;
// }
//
// ULONG
// NTAPI
// IKsDevice_fnAddRef(
//     IN IKsDevice * iface)
// {
//     PKSIDEVICE_HEADER This = (PKSIDEVICE_HEADER)CONTAINING_RECORD(iface, KSIDEVICE_HEADER, BasicHeader.OuterUnknown);
//
//     return InterlockedIncrement(&This->ref);
// }
//
// ULONG
// NTAPI
// IKsDevice_fnRelease(
//     IN IKsDevice * iface)
// {
//     PKSIDEVICE_HEADER This = (PKSIDEVICE_HEADER)CONTAINING_RECORD(iface, KSIDEVICE_HEADER, BasicHeader.OuterUnknown);
//
//     InterlockedDecrement(&This->ref);
//
//     return This->ref;
// }
//
//
//
// PKSDEVICE
// NTAPI
// IKsDevice_fnGetStruct(
//     IN IKsDevice * iface)
// {
//     PKSIDEVICE_HEADER This = (PKSIDEVICE_HEADER)CONTAINING_RECORD(iface, KSIDEVICE_HEADER, BasicHeader.OuterUnknown);
//
//     return &This->KsDevice;
// }
//
// NTSTATUS
// NTAPI
// IKsDevice_fnInitializeObjectBag(
//     IN IKsDevice * iface,
//     IN PKSIOBJECT_BAG Bag,
//     IN PRKMUTEX Mutex)
// {
//     PKSIDEVICE_HEADER This = (PKSIDEVICE_HEADER)CONTAINING_RECORD(iface, KSIDEVICE_HEADER, BasicHeader.OuterUnknown);
//
//     if (!Mutex)
//     {
//         /* use device mutex */
//         Mutex = &This->BagMutex;
//     }
//
//     /* initialize object bag */
//     Bag->BagMutex = Mutex;
//     Bag->DeviceHeader = (PKSIDEVICE_HEADER)This;
//     InitializeListHead(&Bag->ObjectList);
//
//     /* insert bag into device list */
//     InsertTailList(&This->ObjectBags, &Bag->Entry);
//
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// IKsDevice_fnAcquireDevice(
//     IN IKsDevice * iface)
// {
//     PKSIDEVICE_HEADER This = (PKSIDEVICE_HEADER)CONTAINING_RECORD(iface, KSIDEVICE_HEADER, BasicHeader.OuterUnknown);
//
//     return KeWaitForSingleObject(&This->DeviceMutex, Executive, KernelMode, FALSE, NULL);
// }
//
// NTSTATUS
// NTAPI
// IKsDevice_fnReleaseDevice(
//     IN IKsDevice * iface)
// {
//     PKSIDEVICE_HEADER This = (PKSIDEVICE_HEADER)CONTAINING_RECORD(iface, KSIDEVICE_HEADER, BasicHeader.OuterUnknown);
//
//     return KeReleaseMutex(&This->DeviceMutex, FALSE);
// }
//
// NTSTATUS
// NTAPI
// IKsDevice_fnGetAdapterObject(
//     IN IKsDevice * iface,
//     IN PADAPTER_OBJECT * Object,
//     IN PULONG MaxMappingsByteCount,
//     IN PULONG MappingTableStride)
// {
//     PKSIDEVICE_HEADER This = (PKSIDEVICE_HEADER)CONTAINING_RECORD(iface, KSIDEVICE_HEADER, BasicHeader.OuterUnknown);
//
//     *Object = This->AdapterObject;
//     *MaxMappingsByteCount = This->MaxMappingsByteCount;
//     *MappingTableStride = This->MappingTableStride;
//
//     return STATUS_SUCCESS;
//
// }
//
// NTSTATUS
// NTAPI
// IKsDevice_fnAddPowerEntry(
//     IN IKsDevice * iface,
//     IN struct KSPOWER_ENTRY * Entry,
//     IN IKsPowerNotify* Notify)
// {
//     //PKSIDEVICE_HEADER This = (PKSIDEVICE_HEADER)CONTAINING_RECORD(iface, KSIDEVICE_HEADER, BasicHeader.OuterUnknown);
//
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// IKsDevice_fnRemovePowerEntry(
//     IN IKsDevice * iface,
//     IN struct KSPOWER_ENTRY * Entry)
// {
//     //PKSIDEVICE_HEADER This = (PKSIDEVICE_HEADER)CONTAINING_RECORD(iface, KSIDEVICE_HEADER, BasicHeader.OuterUnknown);
//
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
//
// }
//
// NTSTATUS
// NTAPI
// IKsDevice_fnPinStateChange(
//     IN IKsDevice * iface,
//     IN KSPIN Pin,
//     IN PIRP Irp,
//     IN KSSTATE OldState,
//     IN KSSTATE NewState)
// {
//     //PKSIDEVICE_HEADER This = (PKSIDEVICE_HEADER)CONTAINING_RECORD(iface, KSIDEVICE_HEADER, BasicHeader.OuterUnknown);
//
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
//
// }
//
// NTSTATUS
// NTAPI
// IKsDevice_fnArbitrateAdapterChannel(
//     IN IKsDevice * iface,
//     IN ULONG NumberOfMapRegisters,
//     IN PDRIVER_CONTROL ExecutionRoutine,
//     IN PVOID Context)
// {
//     PKSIDEVICE_HEADER This = (PKSIDEVICE_HEADER)CONTAINING_RECORD(iface, KSIDEVICE_HEADER, BasicHeader.OuterUnknown);
//     NTSTATUS Status;
//
//     DPRINT("IKsDevice_fnArbitrateAdapterChannel NumberOfMapRegisters %lu ExecutionRoutine %p Context %p Irql %lu\n", NumberOfMapRegisters, ExecutionRoutine, Context, KeGetCurrentIrql());
//
//     /* sanity check */
//     ASSERT(KeGetCurrentIrql() == DISPATCH_LEVEL);
//     ASSERT(This->AdapterObject);
//
//     /* allocate adapter channel */
//     Status = IoAllocateAdapterChannel(This->AdapterObject, This->KsDevice.FunctionalDeviceObject, NumberOfMapRegisters, ExecutionRoutine, Context);
//
//     /* done */
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// IKsDevice_fnCheckIoCapability(
//     IN IKsDevice * iface,
//     IN ULONG Unknown)
// {
//     //PKSIDEVICE_HEADER This = (PKSIDEVICE_HEADER)CONTAINING_RECORD(iface, KSIDEVICE_HEADER, BasicHeader.OuterUnknown);
//
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// static IKsDeviceVtbl vt_IKsDevice =
// {
//     IKsDevice_fnQueryInterface,
//     IKsDevice_fnAddRef,
//     IKsDevice_fnRelease,
//     IKsDevice_fnGetStruct,
//     IKsDevice_fnInitializeObjectBag,
//     IKsDevice_fnAcquireDevice,
//     IKsDevice_fnReleaseDevice,
//     IKsDevice_fnGetAdapterObject,
//     IKsDevice_fnAddPowerEntry,
//     IKsDevice_fnRemovePowerEntry,
//     IKsDevice_fnPinStateChange,
//     IKsDevice_fnArbitrateAdapterChannel,
//     IKsDevice_fnCheckIoCapability
// };
//
//
// VOID
// NTAPI
// IKsDevice_PnpPostStart(
//     IN PDEVICE_OBJECT  DeviceObject,
//     IN PVOID  Context)
// {
//     NTSTATUS Status;
//     PPNP_POSTSTART_CONTEXT Ctx = (PPNP_POSTSTART_CONTEXT)Context;
//
//     /* call driver pnp post routine */
//     Status = Ctx->DeviceHeader->KsDevice.Descriptor->Dispatch->PostStart(&Ctx->DeviceHeader->KsDevice);
//
//     if (!NT_SUCCESS(Status))
//     {
//         /* set state to disabled */
//         Ctx->DeviceHeader->TargetState  = KSTARGET_STATE_DISABLED;
//     }
//     else
//     {
//         /* set state to enabled */
//         Ctx->DeviceHeader->TargetState = KSTARGET_STATE_ENABLED;
//         Status = KspSetFilterFactoriesState(Ctx->DeviceHeader, TRUE);
//     }
//
//     /* free work item */
//     IoFreeWorkItem(Ctx->WorkItem);
//
//     /* free work context */
//     FreeItem(Ctx);
//
//     DPRINT("IKsDevice_PnpPostStart: PostStart Routine returned %x\n", Status);
// }
//
// NTSTATUS
// NTAPI
// IKsDevice_PnpStartDevice(
//     IN PDEVICE_OBJECT  DeviceObject,
//     IN PIRP Irp)
// {
//     PIO_STACK_LOCATION IoStack;
//     PDEVICE_EXTENSION DeviceExtension;
//     PKSIDEVICE_HEADER DeviceHeader;
//     PPNP_POSTSTART_CONTEXT Ctx = NULL;
//     NTSTATUS Status;
//     PCM_RESOURCE_LIST TranslatedResourceList;
//     PCM_RESOURCE_LIST UntranslatedResourceList;
//
//     /* get current stack location */
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//     /* get device extension */
//     DeviceExtension = (PDEVICE_EXTENSION)DeviceObject->DeviceExtension;
//     /* get device header */
//     DeviceHeader = DeviceExtension->DeviceHeader;
//
//     DPRINT("IKsDevice_PnpStartDevice DeviceHeader %p\n", DeviceHeader);
//
//     /* first forward irp to lower device object */
//     if (!IoForwardIrpSynchronously(DeviceHeader->KsDevice.NextDeviceObject, Irp))
//     {
//         Irp->IoStatus.Status = STATUS_UNSUCCESSFUL;
//     }
//     Status = Irp->IoStatus.Status;
//
//     /* check for success */
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("NextDevice object failed to start with %x\n", Status);
//         CompleteRequest(Irp, IO_NO_INCREMENT);
//         return Status;
//     }
//
//     TranslatedResourceList = IoStack->Parameters.StartDevice.AllocatedResourcesTranslated;
//     UntranslatedResourceList = IoStack->Parameters.StartDevice.AllocatedResources;
//
//     ASSERT(DeviceHeader->KsDevice.Descriptor);
//
//     /* do we have a device descriptor */
//     if (DeviceHeader->KsDevice.Descriptor)
//     {
//         /* does the device want pnp notifications */
//         if (DeviceHeader->KsDevice.Descriptor->Dispatch)
//         {
//             /* does the driver care about IRP_MN_START_DEVICE */
//             if (DeviceHeader->KsDevice.Descriptor->Dispatch->Start)
//             {
//                 /* call driver start device routine */
//                 Status = DeviceHeader->KsDevice.Descriptor->Dispatch->Start(&DeviceHeader->KsDevice, Irp,
//                                                                    TranslatedResourceList,
//                                                                    UntranslatedResourceList);
//
//
//                 DPRINT("IKsDevice_PnpStartDevice Start %p, Context %p\n", DeviceHeader->KsDevice.Descriptor->Dispatch->Start, DeviceHeader->KsDevice.Context);
//                 ASSERT(Status != STATUS_PENDING);
//
//                 if (!NT_SUCCESS(Status))
//                 {
//                     DPRINT1("Driver: failed to start %x\n", Status);
//                     Irp->IoStatus.Status = Status;
//                     CompleteRequest(Irp, IO_NO_INCREMENT);
//                     return Status;
//                 }
//
//                 /* set state to run */
//                 DeviceHeader->KsDevice.Started = TRUE;
//
//             }
//
//             /* does the driver need post start routine */
//             if (DeviceHeader->KsDevice.Descriptor->Dispatch->PostStart)
//             {
//                 /* allocate pnp post workitem context */
//                 Ctx = (PPNP_POSTSTART_CONTEXT)AllocateItem(NonPagedPool, sizeof(PNP_POSTSTART_CONTEXT));
//                 if (!Ctx)
//                 {
//                     /* no memory */
//                     Status = STATUS_INSUFFICIENT_RESOURCES;
//                 }
//                 else
//                 {
//                     /* allocate a work item */
//                     Ctx->WorkItem = IoAllocateWorkItem(DeviceObject);
//
//                     if (!Ctx->WorkItem)
//                     {
//                          /* no memory */
//                         FreeItem(Ctx);
//                         Ctx = NULL;
//                         Status = STATUS_INSUFFICIENT_RESOURCES;
//                     }
//                     else
//                     {
//                         /* store device header for post-start pnp processing */
//                         Ctx->DeviceHeader = DeviceHeader;
//                     }
//                 }
//             }
//             else
//             {
//                 /* set state to enabled, IRP_MJ_CREATE request may now succeed */
//                 DeviceHeader->TargetState = KSTARGET_STATE_ENABLED;
//                 Status = KspSetFilterFactoriesState(DeviceHeader, TRUE);
//             }
//         }
//         else
//         {
//             /* set state to run */
//             DeviceHeader->KsDevice.Started = TRUE;
//         }
//     }
//
//     /* store result */
//     Irp->IoStatus.Status = Status;
//     /* complete request */
//     CompleteRequest(Irp, IO_NO_INCREMENT);
//
//     if (Ctx)
//     {
//         /* queue a work item for driver post start routine */
//         IoQueueWorkItem(Ctx->WorkItem, IKsDevice_PnpPostStart, DelayedWorkQueue, (PVOID)Ctx);
//     }
//
//     /* return result */
//     DPRINT("IKsDevice_PnpStartDevice Status %x PostStartRoutine %p\n", Status, Ctx);
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// IKsDevice_Pnp(
//     IN PDEVICE_OBJECT  DeviceObject,
//     IN PIRP Irp)
// {
//     PIO_STACK_LOCATION IoStack;
//     PDEVICE_EXTENSION DeviceExtension;
//     PKSIDEVICE_HEADER DeviceHeader;
//     PKSDEVICE_DISPATCH Dispatch = NULL;
//     NTSTATUS Status;
//
//     /* get current stack location */
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     /* get device extension */
//     DeviceExtension = (PDEVICE_EXTENSION)DeviceObject->DeviceExtension;
//     /* get device header */
//     DeviceHeader = DeviceExtension->DeviceHeader;
//
//     /* do we have a device descriptor */
//     if (DeviceHeader->KsDevice.Descriptor && DeviceHeader->KsDevice.Descriptor->Dispatch)
//     {
//         /* does the device want pnp notifications */
//         Dispatch = (PKSDEVICE_DISPATCH)DeviceHeader->KsDevice.Descriptor->Dispatch;
//     }
//
//     switch (IoStack->MinorFunction)
//     {
//         case IRP_MN_START_DEVICE:
//         {
//             return IKsDevice_PnpStartDevice(DeviceObject, Irp);
//         }
//
//         case IRP_MN_QUERY_STOP_DEVICE:
//         {
//             Status = STATUS_SUCCESS;
//             /* check for pnp notification support */
//             if (Dispatch)
//             {
//                 /* check for query stop support */
//                 if (Dispatch->QueryStop)
//                 {
//                     /* call driver's query stop */
//                     Status = Dispatch->QueryStop(&DeviceHeader->KsDevice, Irp);
//                     ASSERT(Status != STATUS_PENDING);
//                 }
//             }
//
//             if (!NT_SUCCESS(Status))
//             {
//                 DPRINT1("Driver: query stop failed %x\n", Status);
//                 Irp->IoStatus.Status = Status;
//                 CompleteRequest(Irp, IO_NO_INCREMENT);
//                 return Status;
//             }
//
//             /* pass the irp down the driver stack */
//             IoSkipCurrentIrpStackLocation(Irp);
//             return IoCallDriver(DeviceHeader->KsDevice.NextDeviceObject, Irp);
//         }
//
//         case IRP_MN_REMOVE_DEVICE:
//         {
//             /* Clean up */
//             if (Dispatch)
//             {
//                 /* check for remove support */
//                 if (Dispatch->Remove)
//                 {
//                     /* call driver's stop routine */
//                     Dispatch->Remove(&DeviceHeader->KsDevice, Irp);
//                 }
//             }
//
//             /* pass the irp down the driver stack */
//             IoSkipCurrentIrpStackLocation(Irp);
//             Status = IoCallDriver(DeviceHeader->KsDevice.NextDeviceObject, Irp);
//             /* FIXME delete device resources */
//             return Status;
//         }
//         case IRP_MN_QUERY_INTERFACE:
//         {
//             Status = STATUS_UNSUCCESSFUL;
//             /* check for pnp notification support */
//             if (Dispatch)
//             {
//                 /* check for query interface support */
//                 if (Dispatch->QueryInterface)
//                 {
//                     /* call driver's query interface */
//                     Status = Dispatch->QueryInterface(&DeviceHeader->KsDevice, Irp);
//                     ASSERT(Status != STATUS_PENDING);
//                 }
//             }
//
//             if (NT_SUCCESS(Status))
//             {
//                 /* driver supports a private interface */
//                 DPRINT1("IRP_MN_QUERY_INTERFACE Device supports interface\n");
//                 Irp->IoStatus.Status = Status;
//                 CompleteRequest(Irp, IO_NO_INCREMENT);
//                 return Status;
//             }
//
//             /* pass the irp down the driver stack */
//             IoSkipCurrentIrpStackLocation(Irp);
//             return IoCallDriver(DeviceHeader->KsDevice.NextDeviceObject, Irp);
//         }
//         case IRP_MN_QUERY_DEVICE_RELATIONS:
//         {
//             /* pass the irp down the driver stack */
//             IoSkipCurrentIrpStackLocation(Irp);
//             return IoCallDriver(DeviceHeader->KsDevice.NextDeviceObject, Irp);
//         }
//         case IRP_MN_FILTER_RESOURCE_REQUIREMENTS:
//         {
//             /* pass the irp down the driver stack */
//             //Status = KspForwardIrpSynchronous(DeviceObject, Irp);
//             Status = Irp->IoStatus.Status;
//             DPRINT("IRP_MN_FILTER_RESOURCE_REQUIREMENTS Next Device: Status %x\n", Status);
//
//             //Irp->IoStatus.Status = Status;
//             CompleteRequest(Irp, IO_NO_INCREMENT);
//             return Status;
//         }
//        case IRP_MN_QUERY_RESOURCE_REQUIREMENTS:
//        {
//             /* pass the irp down the driver stack */
//             IoSkipCurrentIrpStackLocation(Irp);
//             return IoCallDriver(DeviceHeader->KsDevice.NextDeviceObject, Irp);
//        }
//        default:
//             DPRINT1("unhandled function %u\n", IoStack->MinorFunction);
//             /* pass the irp down the driver stack */
//             IoSkipCurrentIrpStackLocation(Irp);
//             return IoCallDriver(DeviceHeader->KsDevice.NextDeviceObject, Irp);
//     }
// }
//
// NTSTATUS
// NTAPI
// IKsDevice_Power(
//     IN PDEVICE_OBJECT  DeviceObject,
//     IN PIRP Irp)
// {
//     UNIMPLEMENTED;
//
//     /* TODO */
//
//     Irp->IoStatus.Status = STATUS_SUCCESS;
//     Irp->IoStatus.Information = 0;
//     CompleteRequest(Irp, IO_NO_INCREMENT);
//
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// IKsDevice_Create(
//     IN PDEVICE_OBJECT  DeviceObject,
//     IN PIRP Irp)
// {
//     PCREATE_ITEM_ENTRY CreateItemEntry;
//     PIO_STACK_LOCATION IoStack;
//     PDEVICE_EXTENSION DeviceExtension;
//     PKSIDEVICE_HEADER DeviceHeader;
//     PKSIOBJECT_HEADER ObjectHeader;
//     NTSTATUS Status;
//
//     DPRINT("IKsDevice_Create\n");
//     /* get current stack location */
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//     /* get device extension */
//     DeviceExtension = (PDEVICE_EXTENSION)DeviceObject->DeviceExtension;
//     /* get device header */
//     DeviceHeader = DeviceExtension->DeviceHeader;
//
//     if (IoStack->FileObject->FileName.Buffer == NULL)
//     {
//         // ReactOS PnPMgr still sucks
//         ASSERT(IoStack->FileObject->FileName.Length == 0);
//         Irp->IoStatus.Status = STATUS_SUCCESS;
//         IoCompleteRequest(Irp, IO_NO_INCREMENT);
//         DPRINT1("ReactOS PnP hack\n");
//         return STATUS_SUCCESS;
//     }
//
//     /* acquire list lock */
//     IKsDevice_fnAcquireDevice((IKsDevice*)&DeviceHeader->BasicHeader.OuterUnknown);
//
//     /* sanity check */
//     ASSERT(IoStack->FileObject);
//
//     /* check if the request is relative */
//     if (IoStack->FileObject->RelatedFileObject != NULL)
//     {
//         /* request is to instantiate a pin / node / clock / allocator */
//         ObjectHeader = (PKSIOBJECT_HEADER)IoStack->FileObject->RelatedFileObject->FsContext2;
//
//         /* sanity check */
//         ASSERT(ObjectHeader);
//
//         /* find a matching a create item */
//         Status = FindMatchingCreateItem(&ObjectHeader->ItemList,
//                                         &IoStack->FileObject->FileName,
//                                         &CreateItemEntry);
//     }
//     else
//     {
//         /* request to create a filter */
//         Status = FindMatchingCreateItem(&DeviceHeader->ItemList,
//                                         &IoStack->FileObject->FileName,
//                                         &CreateItemEntry);
//     }
//
//     if (NT_SUCCESS(Status))
//     {
//         /* set object create item */
//         KSCREATE_ITEM_IRP_STORAGE(Irp) = CreateItemEntry->CreateItem;
//
//         /* call create function */
//         Status = CreateItemEntry->CreateItem->Create(DeviceObject, Irp);
//
//         if (NT_SUCCESS(Status))
//         {
//             /* increment create item reference count */
//             InterlockedIncrement(&CreateItemEntry->ReferenceCount);
//         }
//     }
//
//     /* release list lock */
//     IKsDevice_fnReleaseDevice((IKsDevice*)&DeviceHeader->BasicHeader.OuterUnknown);
//
//     /* done */
//     return Status;
//
//
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// NTSTATUS
// NTAPI
// KsInitializeDevice(
//     IN PDEVICE_OBJECT FunctionalDeviceObject,
//     IN PDEVICE_OBJECT PhysicalDeviceObject,
//     IN PDEVICE_OBJECT NextDeviceObject,
//     IN const KSDEVICE_DESCRIPTOR* Descriptor OPTIONAL)
// {
//     PDEVICE_EXTENSION DeviceExtension;
//     PKSIDEVICE_HEADER Header;
//     ULONG Index;
//     PKSIOBJECT_BAG Bag;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     DPRINT1("KsInitializeDevice Descriptor %p\n", Descriptor);
//
//     /* get device extension */
//     DeviceExtension = (PDEVICE_EXTENSION)FunctionalDeviceObject->DeviceExtension;
//
//     /* first allocate device header */
//     Status = KsAllocateDeviceHeader((KSDEVICE_HEADER*)&DeviceExtension->DeviceHeader, 0, NULL);
//
//     /* point to allocated header */
//     Header = DeviceExtension->DeviceHeader;
//
//     DPRINT1("DeviceHeader %p\n", DeviceExtension->DeviceHeader);
//
//     if (Descriptor && Descriptor->Dispatch)
//     {
//         DPRINT("Descriptor Add %p\n", Descriptor->Dispatch->Add);
//         DPRINT("Descriptor Start %p\n", Descriptor->Dispatch->Start);
//         DPRINT("Descriptor PostStart %p\n", Descriptor->Dispatch->PostStart);
//         DPRINT("Descriptor QueryStop %p\n", Descriptor->Dispatch->QueryStop);
//         DPRINT("Descriptor CancelStop %p\n", Descriptor->Dispatch->CancelStop);
//         DPRINT("Descriptor Stop %p\n", Descriptor->Dispatch->Stop);
//         DPRINT("Descriptor QueryRemove %p\n", Descriptor->Dispatch->QueryRemove);
//         DPRINT("Descriptor CancelRemove %p\n", Descriptor->Dispatch->CancelRemove);
//         DPRINT("Descriptor Remove %p\n", Descriptor->Dispatch->Remove);
//         DPRINT("Descriptor QueryCapabilities %p\n", Descriptor->Dispatch->QueryCapabilities);
//         DPRINT("Descriptor SurpriseRemoval %p\n", Descriptor->Dispatch->SurpriseRemoval);
//         DPRINT("Descriptor QueryPower %p\n", Descriptor->Dispatch->QueryPower);
//         DPRINT("Descriptor SetPower %p\n", Descriptor->Dispatch->SetPower);
//         DPRINT("Descriptor QueryInterface %p\n", Descriptor->Dispatch->QueryInterface);
//     }
//
//     /* check for success */
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("KsInitializeDevice Failed to allocate device header with %x\n", Status);
//         return Status;
//     }
//
//     /* initialize IKsDevice interface */
//     Header->BasicHeader.OuterUnknown = (PUNKNOWN)&vt_IKsDevice;
//     Header->ref = 1;
//
//     /* allocate object bag */
//     Header->KsDevice.Bag = AllocateItem(NonPagedPool, sizeof(KSIOBJECT_BAG));
//     if (!Header->KsDevice.Bag)
//     {
//         /* no memory */
//         KsFreeDeviceHeader((KSDEVICE_HEADER*)&DeviceExtension->DeviceHeader);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* initialize object bag */
//     KeInitializeMutex(&Header->BagMutex, 0);
//     KeInitializeMutex(&Header->DeviceMutex, 0);
//
//     Bag = (PKSIOBJECT_BAG)Header->KsDevice.Bag;
//     Bag->BagMutex = &Header->BagMutex;
//     InitializeListHead(&Header->ObjectBags);
//     InitializeListHead(&Bag->ObjectList);
//     Bag->DeviceHeader = (PVOID)Header;
//
//     /* insert bag into device list */
//     InsertTailList(&Header->ObjectBags, &Bag->Entry);
//
//     /* initialize device header */
//     Header->KsDevice.FunctionalDeviceObject = FunctionalDeviceObject;
//     Header->KsDevice.PhysicalDeviceObject = PhysicalDeviceObject;
//     Header->KsDevice.NextDeviceObject = NextDeviceObject;
//     Header->KsDevice.Descriptor = Descriptor;
//     Header->KsDevice.SystemPowerState = PowerSystemWorking;
//     Header->KsDevice.DevicePowerState = PowerDeviceD0;
//     Header->KsDevice.Started = FALSE;
//     Header->KsDevice.Context = NULL;
//     KsSetDevicePnpAndBaseObject(Header, PhysicalDeviceObject, NextDeviceObject);
//
//
//
//     if (Descriptor)
//     {
//         /* create a filter factory for each filter descriptor */
//         DPRINT("KsInitializeDevice FilterDescriptorCount %lu\n", Descriptor->FilterDescriptorsCount);
//         for(Index = 0; Index < Descriptor->FilterDescriptorsCount; Index++)
//         {
//             Status = KspCreateFilterFactory(FunctionalDeviceObject, Descriptor->FilterDescriptors[Index], NULL, NULL, 0, NULL, NULL, NULL);
//
//             DPRINT("KsInitializeDevice Index %lu KspCreateFilterFactory Status %lx\n", Index, Status);
//             /* check for success */
//             if (!NT_SUCCESS(Status))
//             {
//                 DPRINT1("KspCreateFilterFactory failed with %x\n", Status);
//                 /* FIXME memory leak */
//                 return Status;
//             }
//         }
//
//         /* does the driver care about the add device */
//         if (Descriptor->Dispatch && Descriptor->Dispatch->Add)
//         {
//             Status = Descriptor->Dispatch->Add(&Header->KsDevice);
//             DPRINT("Driver: AddHandler Status %x\n", Status);
//         }
//
//         Header->KsDevice.Descriptor = Descriptor;
//     }
//
//
//    return Status;
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// NTSTATUS
// NTAPI
// KsReferenceSoftwareBusObject(
//     IN KSDEVICE_HEADER  Header)
// {
//      IKsDevice * Device;
//      PKSIDEVICE_HEADER DeviceHeader = (PKSIDEVICE_HEADER)Header;
//
//      /* get device interface */
//      Device = (IKsDevice*)DeviceHeader->BasicHeader.OuterUnknown;
//
//      if (Device)
//      {
//          /* reference device interface */
//          Device->lpVtbl->AddRef(Device);
//      }
//
//     return STATUS_SUCCESS;
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// NTSTATUS
// NTAPI
// KsReferenceBusObject(
//     IN  KSDEVICE_HEADER Header)
// {
//      IKsDevice * Device;
//      PKSIDEVICE_HEADER DeviceHeader = (PKSIDEVICE_HEADER)Header;
//
//      /* get device interface */
//      Device = (IKsDevice*)DeviceHeader->BasicHeader.OuterUnknown;
//
//      if (Device)
//      {
//          /* reference device interface */
//          Device->lpVtbl->AddRef(Device);
//      }
//
//     return STATUS_SUCCESS;
//
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// VOID
// NTAPI
// KsDereferenceBusObject(
//     IN  KSDEVICE_HEADER Header)
// {
//      IKsDevice * Device;
//      PKSIDEVICE_HEADER DeviceHeader = (PKSIDEVICE_HEADER)Header;
//
//      /* get device interface */
//      Device = (IKsDevice*)DeviceHeader->BasicHeader.OuterUnknown;
//
//      if (Device)
//      {
//          /* release device interface */
//          Device->lpVtbl->Release(Device);
//      }
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// VOID
// NTAPI
// KsDereferenceSoftwareBusObject(
//     IN KSDEVICE_HEADER  Header)
// {
//      IKsDevice * Device;
//      PKSIDEVICE_HEADER DeviceHeader = (PKSIDEVICE_HEADER)Header;
//
//      DPRINT("KsDereferenceSoftwareBusObject DeviceHeader %p\n", Header);
//
//      /* get device interface */
//      Device = (IKsDevice*)DeviceHeader->BasicHeader.OuterUnknown;
//
//      if (Device)
//      {
//          /* release device interface */
//          Device->lpVtbl->Release(Device);
//      }
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
