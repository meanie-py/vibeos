use crate::prelude::*;
use ntkrnl_sys::ddk::{GUID, IoAllocateWorkItem, IoAttachDeviceToDeviceStack, IoCreateDevice, IoDeleteDevice, IoInitializeTimer, IoQueueWorkItem, IoStartTimer, IoUnregisterPlugPlayNotification, KeInitializeEvent, _EVENT_TYPE_NotificationEvent, PDEVICE_OBJECT, PDRIVER_OBJECT, PIO_STACK_LOCATION, PIRP, PUNICODE_STRING, ZwClose};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut KSCATEGORY_SYSAUDIO: GUID = _GUID {
    Data1: 0xa7c7a5b1 as ::core::ffi::c_ulong as ::core::ffi::c_uint,
    Data2: 0x5af3 as ::core::ffi::c_ushort,
    Data3: 0x11d1 as ::core::ffi::c_ushort,
    Data4: [
        0x9c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xed as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x24 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xbf as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut KSCATEGORY_WDMAUD: GUID = _GUID {
    Data1: 0x3e227e76 as ::core::ffi::c_long as ::core::ffi::c_uint,
    Data2: 0x690d as ::core::ffi::c_ushort,
    Data3: 0x11d2 as ::core::ffi::c_ushort,
    Data4: [
        0x81 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x61 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

// #define NDEBUG: flag-only macro with no value; no constant emitted.

// IO_WORKITEM_ROUTINE WdmAudInitWorkerRoutine; (prototype only, defined with its function body)
// IO_TIMER_ROUTINE WdmAudTimerRoutine; (prototype only, defined with its function body)

const KS_IMPLEMENTED: bool = false;
#[no_mangle]
pub unsafe extern "system" fn WdmAudInitWorkerRoutine(
       DeviceObject: PDEVICE_OBJECT,
       Context: PVOID) {
    let mut Status: NTSTATUS;
    let mut DeviceExtension: PWDMAUD_DEVICE_EXTENSION;
    let mut DeviceCount: ULONG;

    /* get device extension */
    DeviceExtension = (*DeviceObject).DeviceExtension as PWDMAUD_DEVICE_EXTENSION;

    if (*DeviceExtension).FileObject.is_null()
    {
        /* find available sysaudio devices */
        Status = WdmAudOpenSysAudioDevices(DeviceObject, DeviceExtension);
        if !NT_SUCCESS!(Status)
        {
            DPRINT1!("WdmAudOpenSysAudioDevices failed with %x\n", Status);
            return;
        }
    }

    /* get device count */
    DeviceCount = GetSysAudioDeviceCount(DeviceObject);

    DPRINT!("WdmAudInitWorkerRoutine SysAudioDeviceCount %ld\n", DeviceCount);

    /* was a device added / removed */
    if DeviceCount != (*DeviceExtension).SysAudioDeviceCount
    {
        /* init mmixer library */
        Status = WdmAudMixerInitialize(DeviceObject);
        DPRINT!("WdmAudMixerInitialize Status %x WaveIn %lu WaveOut %lu Mixer %lu\n", Status, WdmAudGetWaveInDeviceCount(), WdmAudGetWaveOutDeviceCount(), WdmAudGetMixerDeviceCount());

        /* store sysaudio device count */
        (*DeviceExtension).SysAudioDeviceCount = DeviceCount;
    }

    /* signal completion */
    KeSetEvent(core::ptr::addr_of_mut!((*DeviceExtension).InitializationCompletionEvent), IO_NO_INCREMENT as KPRIORITY, FALSE as BOOLEAN);

    /* reset work item status indicator */
    InterlockedDecrement(core::ptr::addr_of_mut!((*DeviceExtension).WorkItemActive) as *mut _);
}
#[no_mangle]
pub unsafe extern "system" fn WdmAudTimerRoutine(DeviceObject: PDEVICE_OBJECT, Context: PVOID) {
    let mut DeviceExtension: PWDMAUD_DEVICE_EXTENSION = core::ptr::null_mut();
    DeviceExtension = (*DeviceObject).DeviceExtension as PWDMAUD_DEVICE_EXTENSION;
    if InterlockedCompareExchange(
        core::ptr::addr_of_mut!((*DeviceExtension).WorkItemActive) as *mut i32,
        1,
        0,
    ) == 0
    {
        IoQueueWorkItem(
            (*DeviceExtension).WorkItem,
            Some(WdmAudInitWorkerRoutine),
            DelayedWorkQueue,
            DeviceExtension as PVOID,
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn WdmaudAddDevice(
    DriverObject: PDRIVER_OBJECT,
    PhysicalDeviceObject: PDEVICE_OBJECT,
) -> NTSTATUS {
    let mut DeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut Status: NTSTATUS;
    let mut DeviceExtension: PWDMAUD_DEVICE_EXTENSION = core::ptr::null_mut();

    DPRINT!("WdmaudAddDevice called\n");

    Status = IoCreateDevice(
        DriverObject,
        core::mem::size_of::<WDMAUD_DEVICE_EXTENSION>() as ULONG,
        core::ptr::null_mut(),
        FILE_DEVICE_KS as ULONG,
        0 as ULONG,
        FALSE as BOOLEAN,
        core::ptr::addr_of_mut!(DeviceObject),
    );

    if !NT_SUCCESS!(Status) {
        DPRINT1!("IoCreateDevice failed with %x\n", Status);
        return Status;
    }

    /* get device extension */
    DeviceExtension = (*DeviceObject).DeviceExtension as PWDMAUD_DEVICE_EXTENSION;
    RtlZeroMemory!(
        DeviceExtension as *mut core::ffi::c_void,
        core::mem::size_of::<WDMAUD_DEVICE_EXTENSION>() as usize,
    );

    /* allocate work item */
    (*DeviceExtension).WorkItem = IoAllocateWorkItem(DeviceObject);
    if (*DeviceExtension).WorkItem.is_null() {
        /* failed to allocate work item */
        IoDeleteDevice(DeviceObject);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    /* register device interfaces */
    Status = WdmAudRegisterDeviceInterface(PhysicalDeviceObject, DeviceExtension);
    if !NT_SUCCESS!(Status) {
        DPRINT1!("WdmRegisterDeviceInterface failed with %x\n", Status);
        IoDeleteDevice(DeviceObject);
        return Status;
    }

    /* initialize sysaudio device list */
    InitializeListHead(core::ptr::addr_of_mut!((*DeviceExtension).SysAudioDeviceList));

    /* initialize client context device list */
    InitializeListHead(core::ptr::addr_of_mut!((*DeviceExtension).WdmAudClientList));

    /* initialize spinlock */
    KeInitializeSpinLock(core::ptr::addr_of_mut!((*DeviceExtension).Lock));

    /* initialization completion event */
    KeInitializeEvent(
        core::ptr::addr_of_mut!((*DeviceExtension).InitializationCompletionEvent),
        _EVENT_TYPE_NotificationEvent,
        FALSE as BOOLEAN,
    );

    /* initialize timer */
    IoInitializeTimer(
        DeviceObject,
        Some(WdmAudTimerRoutine),
        WdmAudTimerRoutine as PVOID,
    );

    /* allocate ks device header */
    Status = KsAllocateDeviceHeader(
        core::ptr::addr_of_mut!((*DeviceExtension).DeviceHeader),
        0 as ULONG,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS!(Status) {
        DPRINT1!("KsAllocateDeviceHeader failed with %x\n", Status);
        IoDeleteDevice(DeviceObject);
        return Status;
    }

    /* attach to device stack */
    (*DeviceExtension).NextDeviceObject =
        IoAttachDeviceToDeviceStack(DeviceObject, PhysicalDeviceObject);
    KsSetDevicePnpAndBaseObject(
        (*DeviceExtension).DeviceHeader,
        (*DeviceExtension).NextDeviceObject,
        DeviceObject,
    );

    /* start the timer */
    IoStartTimer(DeviceObject);

    (*DeviceObject).Flags |=
        (DO_DIRECT_IO | DO_POWER_PAGABLE) as ULONG;
    (*DeviceObject).Flags &= !(DO_DEVICE_INITIALIZING as ULONG);

    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn WdmAudUnload(driver: PDRIVER_OBJECT) {
    // DPRINT("WdmAudUnload called\n");
}

#[no_mangle]
pub unsafe extern "system" fn WdmAudPnp(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    let IrpStack: PIO_STACK_LOCATION;

    DPRINT!("WdmAudPnp called\n");

    IrpStack = IoGetCurrentIrpStackLocation(Irp);

    if (*IrpStack).MinorFunction as i32 == (IRP_MN_QUERY_PNP_DEVICE_STATE as i32) {
        (*Irp).IoStatus.Information |= PNP_DEVICE_NOT_DISABLEABLE as ULONG_PTR;
        return KsDefaultDispatchPnp(DeviceObject, Irp);
    }
    return KsDefaultDispatchPnp(DeviceObject, Irp);
}

#[no_mangle]
pub unsafe extern "system" fn WdmAudCreate(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut IoStack: PIO_STACK_LOCATION;
    let mut pClient: PWDMAUD_CLIENT = core::mem::zeroed();
    let mut DeviceExtension: PWDMAUD_DEVICE_EXTENSION;

    /* get device extension */
    DeviceExtension = (*DeviceObject).DeviceExtension as PWDMAUD_DEVICE_EXTENSION;

    if KS_IMPLEMENTED {
        Status = KsReferenceSoftwareBusObject((*DeviceObject).DeviceExtension as KSDEVICE_HEADER);
        if !NT_SUCCESS!(Status) {
            DPRINT1!("KsReferenceSoftwareBusObject failed with %x\n", Status);
            return Status;
        }
    }

    if (*DeviceExtension).FileObject.is_null() {
        /* initialize */
        WdmAudInitWorkerRoutine(DeviceObject, core::ptr::null_mut());
    }

    Status = WdmAudOpenSysaudio(DeviceObject, core::ptr::addr_of_mut!(pClient));
    if !NT_SUCCESS!(Status) {
        DPRINT1!("Failed to open sysaudio!\n");

        /* complete and forget */
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_UNSUCCESSFUL;
        IoCompleteRequest(Irp, (IO_NO_INCREMENT as i8));
        /* done */
        return STATUS_UNSUCCESSFUL;
    }

    IoStack = IoGetCurrentIrpStackLocation(Irp);
    ASSERT!(!(*IoStack).FileObject.is_null());

    /* store client context in file object */
    (*(*IoStack).FileObject).FsContext = pClient as PVOID;
    Status = STATUS_SUCCESS;

    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    (*Irp).IoStatus.Information = 0;
    IoCompleteRequest(Irp, (IO_NO_INCREMENT as i8));

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn WdmAudClose(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    /* nothing to do complete request */
    // entry.c's `#if KS_IMPLEMENTED` block (KsDereferenceSoftwareBusObject + unregistering
    // the SysAudio notification) is compiled out, and could not compile: it uses a `Status`
    // and a `DeviceExtension` this function never declares. Nothing to translate.

    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    (*Irp).IoStatus.Information = 0;
    IoCompleteRequest(Irp, (IO_NO_INCREMENT as i8));

    /* done */
    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn WdmAudCleanup(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut IoStack: PIO_STACK_LOCATION;
    let mut pClient: PWDMAUD_CLIENT;
    let mut DeviceExtension: PWDMAUD_DEVICE_EXTENSION;
    let mut Index: ULONG;
    let mut OldIrql: KIRQL = core::mem::zeroed();

    /* get device extension */
    DeviceExtension = (*DeviceObject).DeviceExtension as PWDMAUD_DEVICE_EXTENSION;

    /* get current irp stack location */
    IoStack = IoGetCurrentIrpStackLocation(Irp);

    /* sanity check */
    ASSERT!(!(*IoStack).FileObject.is_null());

    /* get client context struct */
    pClient = (*(*IoStack).FileObject).FsContext as PWDMAUD_CLIENT;

    /* sanity check */
    ASSERT!(!pClient.is_null());

    /* acquire client context list lock */
    KeAcquireSpinLock(&raw mut (*DeviceExtension).Lock, &raw mut OldIrql);

    /* remove entry */
    RemoveEntryList(&raw mut (*pClient).Entry);

    /* release lock */
    KeReleaseSpinLock(&raw mut (*DeviceExtension).Lock, OldIrql);

    /* check if all audio pins have been closed */
    Index = 0;
    while Index < (*pClient).NumPins {
        DPRINT!(
            "Index %u Pin %p Type %x\n",
            Index,
            (*(*pClient).hPins.offset(Index as isize)).Handle,
            (*(*pClient).hPins.offset(Index as isize)).Type
        );
        if !(*(*pClient).hPins.offset(Index as isize)).Handle.is_null()
            && (*(*pClient).hPins.offset(Index as isize)).Type != MIXER_DEVICE_TYPE
        {
            /* found an still open audio pin */
            ZwClose((*(*pClient).hPins.offset(Index as isize)).Handle);
        }
        WdmAudCloseAllMixers(DeviceObject, pClient, Index);
        Index = Index.wrapping_add(1);
    }

    /* free pin array */
    if !(*pClient).hPins.is_null() {
        FreeItem((*pClient).hPins as PVOID);
    }

    /* free client context struct */
    FreeItem(pClient as PVOID);

    /* clear old client pointer */
    (*(*IoStack).FileObject).FsContext = core::ptr::null_mut();

    /* complete request */
    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    (*Irp).IoStatus.Information = 0;
    IoCompleteRequest(Irp, (IO_NO_INCREMENT as i8));

    /* done */
    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn DriverEntry(
    Driver: PDRIVER_OBJECT,
    Registry_path: PUNICODE_STRING,
) -> NTSTATUS {
    DPRINT!("Wdmaud.sys loaded\n");

    (*Driver).DriverUnload =
        Some(WdmAudUnload);

    (*Driver).MajorFunction[IRP_MJ_CREATE as usize] =
        Some(WdmAudCreate);
    (*Driver).MajorFunction[IRP_MJ_CLOSE as usize] =
        Some(WdmAudClose);
    (*Driver).MajorFunction[IRP_MJ_PNP as usize] =
        Some(WdmAudPnp);
    (*Driver).MajorFunction[IRP_MJ_SYSTEM_CONTROL as usize] =
        Some(KsDefaultForwardIrp);
    (*Driver).MajorFunction[IRP_MJ_CLEANUP as usize] =
        Some(WdmAudCleanup);
    (*Driver).MajorFunction[IRP_MJ_DEVICE_CONTROL as usize] =
        Some(WdmAudDeviceControl);
    (*Driver).MajorFunction[IRP_MJ_WRITE as usize] =
        Some(WdmAudReadWrite);
    (*Driver).MajorFunction[IRP_MJ_READ as usize] =
        Some(WdmAudReadWrite);
    (*Driver).MajorFunction[IRP_MJ_POWER as usize] =
        Some(KsDefaultDispatchPower);
    (*(*Driver).DriverExtension).AddDevice =
        Some(WdmaudAddDevice);

    return STATUS_SUCCESS;
}
