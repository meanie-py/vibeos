use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{_DEVICE_RELATION_TYPE_BusRelations as BusRelations, DEVICE_CAPABILITIES, DEVICE_RELATIONS, ExFreePool, _KWAIT_REASON_Executive, IRP, IoAllocateIrp, IoCreateDevice, IoDeleteDevice, IoDetachDevice, IoForwardIrpSynchronously, IoFreeIrp, IoGetCurrentIrpStackLocation, IoGetNextIrpStackLocation, IoInvalidateDeviceRelations, IoMarkIrpPending, IoSetCompletionRoutine, IoSkipCurrentIrpStackLocation, KEVENT, KeInitializeEvent, KeWaitForSingleObject, LIST_ENTRY, _EVENT_TYPE_NotificationEvent, PDEVICE_CAPABILITIES, PDEVICE_OBJECT, PDEVICE_RELATIONS, PIO_STACK_LOCATION, PIRP, PRKEVENT};
use core::ffi::c_void;

extern "system" {
    fn RtlAssert( FailedAssertion: PVOID, FileName: PVOID, LineNumber: ULONG, Message: *mut CHAR, );
    fn AllocateItem(PoolType: u32, NumberOfBytes: ULONG) -> PVOID;
    fn DbgPrint(Format: *const i8, ...) -> u32;
    fn USBCCGP_SyncUrbRequest(DeviceObject: PDEVICE_OBJECT, Urb: PURB) -> NTSTATUS;
    fn USBCCGP_GetDescriptors(DeviceObject: PDEVICE_OBJECT) -> NTSTATUS;
    fn USBCCGP_SelectConfiguration( DeviceObject: PDEVICE_OBJECT, FDODeviceExtension: PFDO_DEVICE_EXTENSION, ) -> NTSTATUS;
    fn USBCCGP_QueryInterface( DeviceObject: PDEVICE_OBJECT, BusInterface: PVOID, );
    fn USBCCGP_EnumerateFunctions(DeviceObject: PDEVICE_OBJECT) -> NTSTATUS;
    fn DumpFunctionDescriptor( FunctionDescriptor: PVOID, FunctionDescriptorCount: ULONG, );
}
// USBD_CreateConfigurationRequestEx is NTAPI (usbdlib.h) → stdcall on i386; usbd-rust exports
// `_USBD_CreateConfigurationRequestEx@8`. Must be extern "system", NOT the cdecl block above.
extern "system" {
    fn USBD_CreateConfigurationRequestEx( ConfigurationDescriptor: PUSB_CONFIGURATION_DESCRIPTOR, InterfaceList: PUSBD_INTERFACE_LIST_ENTRY, ) -> PURB;
}

// ---- file-scope preamble (translated) ----
// #define NDEBUG

const FILE_DEVICE_USB: u32 = 0x00000022;

// NT_VERIFY(c): evaluates the expression; asserts in checked builds, no-op under NDEBUG.
#[inline(always)]
unsafe fn NT_VERIFY(_cond: i32) {}

#[no_mangle]
pub unsafe extern "system" fn FDO_QueryCapabilitiesCompletionRoutine(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Context: PVOID,
) -> NTSTATUS {
    /* Set event */
    KeSetEvent(Context as PRKEVENT as *mut c_void, 0, 0);

    /* Completion is done in the HidClassFDO_QueryCapabilities routine */
    return STATUS_MORE_PROCESSING_REQUIRED;
}
const STATUS_MORE_PROCESSING_REQUIRED: NTSTATUS = 0xC0000016u32 as NTSTATUS;
#[no_mangle]
pub unsafe extern "C" fn FDO_QueryCapabilities(
    DeviceObject: PDEVICE_OBJECT,
    Capabilities: PDEVICE_CAPABILITIES,
) -> NTSTATUS {
    const IRP_MN_QUERY_CAPABILITIES: u32 = 0x09;

    let Irp: PIRP;
    let mut Event: KEVENT = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let IoStack: PIO_STACK_LOCATION;
    let FDODeviceExtension: PFDO_DEVICE_EXTENSION;

    /* Get device extension */
    FDODeviceExtension = (*DeviceObject).DeviceExtension as PFDO_DEVICE_EXTENSION;
    if (*FDODeviceExtension).Common.IsFDO == 0 {
        RtlAssert(
            b"FDODeviceExtension->Common.IsFDO\0".as_ptr() as *const i8 as PVOID,
            b"/home/curi/Projects/os-playground/vibeos-wt-filesystems/drivers/usb/usbccgp/fdo.c\0".as_ptr() as *const i8 as PVOID,
            44 as ULONG,
            core::ptr::null_mut(),
        );
    } else {
    }

    /* Init event */
    KeInitializeEvent(core::ptr::addr_of_mut!(Event), _EVENT_TYPE_NotificationEvent, 0);

    /* Now allocate the irp */
    Irp = IoAllocateIrp((*DeviceObject).StackSize, 0);
    if Irp.is_null() {
        /* No memory */
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    /* Get next stack location */
    IoStack = IoGetNextIrpStackLocation(Irp);

    /* Init stack location */
    (*IoStack).MajorFunction = IRP_MJ_PNP as UCHAR;
    (*IoStack).MinorFunction = IRP_MN_QUERY_CAPABILITIES as UCHAR;
    (*IoStack).Parameters.DeviceCapabilities.Capabilities = Capabilities;

    /* Set completion routine */
    IoSetCompletionRoutine(
        Irp,
        core::mem::transmute::<
            unsafe extern "system" fn(PDEVICE_OBJECT, PIRP, PVOID) -> NTSTATUS,
            ntkrnl_sys::ddk::PIO_COMPLETION_ROUTINE,
        >(FDO_QueryCapabilitiesCompletionRoutine),
        core::ptr::addr_of_mut!(Event) as PVOID,
        1,
        1,
        1,
    );

    /* Init capabilities */
    RtlZeroMemory(
        Capabilities as PVOID,
        core::mem::size_of::<DEVICE_CAPABILITIES>(),
    );
    (*Capabilities).Size = core::mem::size_of::<DEVICE_CAPABILITIES>() as USHORT;
    (*Capabilities).Version = 1 as USHORT; // FIXME hardcoded constant
    (*Capabilities).Address = MAXULONG as ULONG;
    (*Capabilities).UINumber = MAXULONG as ULONG;

    /* Pnp irps have default completion code */
    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_NOT_SUPPORTED;

    /* Call lower device */
    Status = IoCallDriver((*FDODeviceExtension).NextDeviceObject, Irp);
    if Status == STATUS_PENDING {
        /* Wait for completion */
        KeWaitForSingleObject(
            core::ptr::addr_of_mut!(Event) as PVOID,
            _KWAIT_REASON_Executive,
            0i32 as KPROCESSOR_MODE,
            0,
            core::ptr::null_mut(),
        );
    }

    /* Get status */
    Status = (*Irp).IoStatus.__bindgen_anon_1.Status;

    /* Complete request */
    IoFreeIrp(Irp);

    /* Done */
    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn FDO_DeviceRelations(
    mut DeviceObject: PDEVICE_OBJECT,
    mut Irp: PIRP,
) -> NTSTATUS {
    let mut DeviceCount: ULONG = 0 as ULONG;
    let mut Index: ULONG;
    let mut DeviceRelations: PDEVICE_RELATIONS;
    let mut IoStack: PIO_STACK_LOCATION;
    let mut FDODeviceExtension: PFDO_DEVICE_EXTENSION;

    /* Get device extension */
    FDODeviceExtension = (*DeviceObject).DeviceExtension as PFDO_DEVICE_EXTENSION;

    /* Get current irp stack location */
    IoStack = IoGetCurrentIrpStackLocation(Irp);

    /* Check if relation type is BusRelations */
    if (*IoStack).Parameters.QueryDeviceRelations.Type as u32 != BusRelations as u32 {
        /* FDO always only handles bus relations */
        return STATUS_SUCCESS;
    }

    /* Go through array and count device objects */
    Index = 0 as ULONG;
    while Index < (*FDODeviceExtension).FunctionDescriptorCount {
        if !(*(*FDODeviceExtension).ChildPDO.offset(Index as isize)).is_null() {
            /* Child pdo */
            DeviceCount = DeviceCount.wrapping_add(1);
        }
        Index = Index.wrapping_add(1);
    }

    /* Allocate device relations */
    DeviceRelations = AllocateItem(
        PagedPool as u32,
        (core::mem::size_of::<DEVICE_RELATIONS>()).wrapping_add(if DeviceCount > 1 {
            ((DeviceCount as usize).wrapping_sub(1))
                .wrapping_mul(core::mem::size_of::<PDEVICE_OBJECT>())
        } else {
            0
        }) as ULONG,
    ) as PDEVICE_RELATIONS;
    if DeviceRelations.is_null() {
        /* No memory */
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    /* Add device objects */
    Index = 0 as ULONG;
    while Index < (*FDODeviceExtension).FunctionDescriptorCount {
        if !(*(*FDODeviceExtension).ChildPDO.offset(Index as isize)).is_null() {
            /* Store child pdo */
            let fresh1 = core::ptr::addr_of_mut!((*DeviceRelations).Objects) as *mut PDEVICE_OBJECT;
            *fresh1.offset((*DeviceRelations).Count as isize) =
                *(*FDODeviceExtension).ChildPDO.offset(Index as isize);

            /* Add reference */
            ObReferenceObject(
                *(*FDODeviceExtension).ChildPDO.offset(Index as isize) as PVOID,
            );

            /* Increment count */
            (*DeviceRelations).Count = (*DeviceRelations).Count.wrapping_add(1);
        }
        Index = Index.wrapping_add(1);
    }

    /* Store result */
    (*Irp).IoStatus.Information = DeviceRelations as _;
    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;

    /* Request completed successfully */
    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "C" fn FDO_CreateChildPdo(mut DeviceObject: PDEVICE_OBJECT) -> NTSTATUS {
    let mut Status: NTSTATUS = 0;
    let mut PDODeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut PDODeviceExtension: PPDO_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut FDODeviceExtension: PFDO_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut Index: ULONG;

    /* Get device extension */
    FDODeviceExtension = (*DeviceObject).DeviceExtension as PFDO_DEVICE_EXTENSION;
    if (*FDODeviceExtension).Common.IsFDO == 0 {
        RtlAssert(
            b"FDODeviceExtension->Common.IsFDO\0".as_ptr() as *const i8 as PVOID,
            b"/home/curi/Projects/os-playground/vibeos-wt-filesystems/drivers/usb/usbccgp/fdo.c\0".as_ptr() as *const i8 as PVOID,
            180 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    }

    /* Lets create array for the child PDO */
    (*FDODeviceExtension).ChildPDO = AllocateItem(
        NonPagedPool,
        (core::mem::size_of::<PDEVICE_OBJECT>() as usize)
            .wrapping_mul((*FDODeviceExtension).FunctionDescriptorCount as usize) as ULONG,
    ) as *mut PDEVICE_OBJECT;
    if (*FDODeviceExtension).ChildPDO.is_null() {
        /* No memory */
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    /* Create pdo for each function */
    Index = 0 as ULONG;
    while Index < (*FDODeviceExtension).FunctionDescriptorCount {
        if (*(*FDODeviceExtension).FunctionDescriptor.offset(Index as isize)).NumberOfInterfaces as i32 == 0 {
            // Ignore invalid devices
            if DbgPrint(
                b"(%s:%d) [USBCCGP] Found descriptor with 0 interfaces\n\0".as_ptr() as *const i8,
                b"/home/curi/Projects/os-playground/vibeos-wt-filesystems/drivers/usb/usbccgp/fdo.c\0".as_ptr() as *const i8,
                197 as i32,
            ) != 0
            {
                DbgPrint(
                    b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
                    b"/home/curi/Projects/os-playground/vibeos-wt-filesystems/drivers/usb/usbccgp/fdo.c\0".as_ptr() as *const i8,
                    197 as i32,
                );
            }
        } else {
            /* Create the PDO */
            Status = IoCreateDevice(
                (*FDODeviceExtension).DriverObject,
                core::mem::size_of::<PDO_DEVICE_EXTENSION>() as ULONG,
                core::ptr::null_mut(),
                FILE_DEVICE_USB as ULONG,
                FILE_AUTOGENERATED_DEVICE_NAME as ULONG,
                0 as BOOLEAN,
                core::ptr::addr_of_mut!(PDODeviceObject),
            );
            if !(Status >= 0 as i32) {
                /* Failed to create device object */
                if DbgPrint(
                    b"(%s:%d) IoCreateDevice failed with %x\n\0".as_ptr() as *const i8,
                    b"/home/curi/Projects/os-playground/vibeos-wt-filesystems/drivers/usb/usbccgp/fdo.c\0".as_ptr() as *const i8,
                    212 as i32,
                    Status,
                ) != 0
                {
                    DbgPrint(
                        b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
                        b"/home/curi/Projects/os-playground/vibeos-wt-filesystems/drivers/usb/usbccgp/fdo.c\0".as_ptr() as *const i8,
                        212 as i32,
                    );
                }
                return Status;
            }

            /* Store in array */
            let fresh2 = &mut *(*FDODeviceExtension).ChildPDO.offset(Index as isize);
            *fresh2 = PDODeviceObject;

            /* Get device extension */
            PDODeviceExtension = (*PDODeviceObject).DeviceExtension as PPDO_DEVICE_EXTENSION;
            core::ptr::write_bytes(
                PDODeviceExtension as *mut core::ffi::c_void,
                0,
                core::mem::size_of::<PDO_DEVICE_EXTENSION>(),
            );

            /* Init device extension */
            (*PDODeviceExtension).Common.IsFDO = 0 as BOOLEAN;
            (*PDODeviceExtension).FunctionDescriptor =
                (*FDODeviceExtension).FunctionDescriptor.offset(Index as isize) as PUSBC_FUNCTION_DESCRIPTOR;
            (*PDODeviceExtension).NextDeviceObject = DeviceObject;
            (*PDODeviceExtension).FunctionIndex = Index;
            (*PDODeviceExtension).FDODeviceExtension = FDODeviceExtension;
            (*PDODeviceExtension).InterfaceList = (*FDODeviceExtension).InterfaceList;
            (*PDODeviceExtension).InterfaceListCount = (*FDODeviceExtension).InterfaceListCount;
            (*PDODeviceExtension).ConfigurationHandle = (*FDODeviceExtension).ConfigurationHandle;
            (*PDODeviceExtension).ConfigurationDescriptor = (*FDODeviceExtension).ConfigurationDescriptor;
            core::ptr::copy_nonoverlapping(
                core::ptr::addr_of!((*FDODeviceExtension).Capabilities) as *const core::ffi::c_void,
                core::ptr::addr_of_mut!((*PDODeviceExtension).Capabilities) as *mut core::ffi::c_void,
                core::mem::size_of::<DEVICE_CAPABILITIES>(),
            );
            core::ptr::copy_nonoverlapping(
                (*FDODeviceExtension).DeviceDescriptor as *const core::ffi::c_void,
                core::ptr::addr_of_mut!((*PDODeviceExtension).DeviceDescriptor) as *mut core::ffi::c_void,
                core::mem::size_of::<USB_DEVICE_DESCRIPTOR>(),
            );

            /* Patch the stack size */
            (*PDODeviceObject).StackSize = ((*DeviceObject).StackSize as i32 + 1) as CCHAR;

            /* Set device flags */
            (*PDODeviceObject).Flags |= (DO_DIRECT_IO | DO_MAP_IO_BUFFER) as core::ffi::c_ulong;

            /* Device is initialized */
            (*PDODeviceObject).Flags &= !DO_DEVICE_INITIALIZING as core::ffi::c_ulong;
        }
        Index = Index.wrapping_add(1);
    }

    /* Done */
    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn FDO_StartDevice(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {

    let mut Status: NTSTATUS;
    let FDODeviceExtension: PFDO_DEVICE_EXTENSION;

    /* Get device extension */
    FDODeviceExtension = (*DeviceObject).DeviceExtension as PFDO_DEVICE_EXTENSION;
    /* ASSERT(FDODeviceExtension->Common.IsFDO); */

    /* First start lower device */
    if IoForwardIrpSynchronously((*FDODeviceExtension).NextDeviceObject, Irp) != 0 {
        Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
    } else {
        Status = STATUS_UNSUCCESSFUL;
    }

    if !NT_SUCCESS(Status) {
        /* Failed to start lower device */
        return Status;
    }

    /* Get descriptors */
    Status = USBCCGP_GetDescriptors(DeviceObject);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    /* Get capabilities */
    Status = FDO_QueryCapabilities(
        DeviceObject,
        core::ptr::addr_of_mut!((*FDODeviceExtension).Capabilities),
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    /* Now select the configuration */
    Status = USBCCGP_SelectConfiguration(DeviceObject, FDODeviceExtension);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    /* Query bus interface */
    USBCCGP_QueryInterface(
        (*FDODeviceExtension).NextDeviceObject,
        core::ptr::addr_of_mut!((*FDODeviceExtension).BusInterface) as PVOID,
    );

    /* Now enumerate the functions */
    Status = USBCCGP_EnumerateFunctions(DeviceObject);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    /* Sanity checks */
    /* ASSERT(FDODeviceExtension->FunctionDescriptorCount); */
    /* ASSERT(FDODeviceExtension->FunctionDescriptor); */
    DumpFunctionDescriptor(
        (*FDODeviceExtension).FunctionDescriptor as PVOID,
        (*FDODeviceExtension).FunctionDescriptorCount,
    );

    /* Now create the pdo */
    Status = FDO_CreateChildPdo(DeviceObject);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    /* Inform pnp manager of new device objects */
    IoInvalidateDeviceRelations(
        (*FDODeviceExtension).PhysicalDeviceObject,
        BusRelations,
    );

    /* Done */
    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn FDO_CloseConfiguration(DeviceObject: PDEVICE_OBJECT) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let Urb: PURB;
    let FDODeviceExtension: PFDO_DEVICE_EXTENSION;

    /* Get device extension */
    FDODeviceExtension = (*DeviceObject).DeviceExtension as PFDO_DEVICE_EXTENSION;
    /* ASSERT(FDODeviceExtension->Common.IsFDO); */

    /* Nothing to do if we're not configured */
    if (*FDODeviceExtension).ConfigurationDescriptor.is_null()
        || (*FDODeviceExtension).InterfaceList.is_null()
    {
        return STATUS_SUCCESS;
    }

    /* Now allocate the urb */
    Urb = USBD_CreateConfigurationRequestEx(
        (*FDODeviceExtension).ConfigurationDescriptor,
        (*FDODeviceExtension).InterfaceList,
    );
    if Urb.is_null() {
        /* No memory */
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    /* Clear configuration descriptor to make it an unconfigure request */
    (*(*Urb).__bindgen_anon_1.UrbSelectConfiguration.as_mut()).ConfigurationDescriptor = core::ptr::null_mut();

    /* Submit urb */
    Status = USBCCGP_SyncUrbRequest((*FDODeviceExtension).NextDeviceObject, Urb);
    if !NT_SUCCESS(Status) {
        /* Failed to set configuration */
        /* DPRINT1("USBCCGP_SyncUrbRequest failed to unconfigure device\n", Status); */
    }

    ExFreePool(Urb as *mut core::ffi::c_void);
    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn FDO_HandlePnp(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut IoStack: PIO_STACK_LOCATION;
    let mut Status: NTSTATUS;
    let mut FDODeviceExtension: PFDO_DEVICE_EXTENSION;

    /* Get device extension */
    FDODeviceExtension = (*DeviceObject).DeviceExtension as PFDO_DEVICE_EXTENSION;

    const IRP_MN_QUERY_CAPABILITIES: u8 = 0x09;

    /* Get stack location */
    IoStack = IoGetCurrentIrpStackLocation(Irp);
    match (*IoStack).MinorFunction {
        IRP_MN_REMOVE_DEVICE => {
            // Unconfigure device */
            FDO_CloseConfiguration(DeviceObject);

            /* Send the IRP down the stack */
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
            IoSkipCurrentIrpStackLocation(Irp);
            Status = IoCallDriver((*FDODeviceExtension).NextDeviceObject, Irp);

            /* Detach from the device stack */
            IoDetachDevice((*FDODeviceExtension).NextDeviceObject);

            /* Delete the device object */
            IoDeleteDevice(DeviceObject);

            /* The lower driver completes the IRP for removal */
            return Status;
        }
        IRP_MN_START_DEVICE => {
            /* Start the device */
            Status = FDO_StartDevice(DeviceObject, Irp);
        }
        IRP_MN_QUERY_DEVICE_RELATIONS => {
            /* Handle device relations */
            Status = FDO_DeviceRelations(DeviceObject, Irp);
            if !NT_SUCCESS(Status) {
                /* break */
            } else {
                /* Forward irp to next device object */
                IoSkipCurrentIrpStackLocation(Irp);
                return IoCallDriver((*FDODeviceExtension).NextDeviceObject, Irp);
            }
        }
        IRP_MN_QUERY_CAPABILITIES => {
            /* Copy capabilities */
            RtlCopyMemory(
                (*IoStack).Parameters.DeviceCapabilities.Capabilities as *mut core::ffi::c_void,
                core::ptr::addr_of!((*FDODeviceExtension).Capabilities) as *const core::ffi::c_void,
                core::mem::size_of::<DEVICE_CAPABILITIES>() as usize,
            );
            Status = STATUS_UNSUCCESSFUL;

            if IoForwardIrpSynchronously((*FDODeviceExtension).NextDeviceObject, Irp) != 0 {
                Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
                if NT_SUCCESS(Status) {
                    (*(*IoStack).Parameters.DeviceCapabilities.Capabilities)
                        .set_SurpriseRemovalOK(1 as ULONG);
                }
            }
        }
        IRP_MN_QUERY_REMOVE_DEVICE | IRP_MN_QUERY_STOP_DEVICE => {
            /* Sure */
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;

            /* Forward irp to next device object */
            IoSkipCurrentIrpStackLocation(Irp);
            return IoCallDriver((*FDODeviceExtension).NextDeviceObject, Irp);
        }
        _ => {
            /* Forward irp to next device object */
            IoSkipCurrentIrpStackLocation(Irp);
            return IoCallDriver((*FDODeviceExtension).NextDeviceObject, Irp);
        }
    }

    /* Complete request */
    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn FDO_HandleResetCyclePort(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut IoStack: PIO_STACK_LOCATION;
    let mut Status: NTSTATUS = 0;
    let FDODeviceExtension: PFDO_DEVICE_EXTENSION;
    let ListHead: PLIST_ENTRY;
    let mut Entry: PLIST_ENTRY;
    let mut TempList: LIST_ENTRY = core::mem::zeroed();
    let ResetActive: PUCHAR;
    let mut ListIrp: PIRP;
    let mut OldLevel: KIRQL = 0;

    // CTL_CODE(FILE_DEVICE_USB=0x22, USB_RESET_PORT=1, METHOD_NEITHER=3, FILE_ANY_ACCESS=0)
    const IOCTL_INTERNAL_USB_RESET_PORT: u32 = 0x00220007;

    /* Get device extension */
    FDODeviceExtension = (*DeviceObject).DeviceExtension as PFDO_DEVICE_EXTENSION;
    /* ASSERT(FDODeviceExtension->Common.IsFDO); */

    /* Get stack location */
    IoStack = IoGetCurrentIrpStackLocation(Irp);

    if (*IoStack).Parameters.DeviceIoControl.IoControlCode
        == IOCTL_INTERNAL_USB_RESET_PORT as u32
    {
        /* Use reset port list */
        ListHead = core::ptr::addr_of_mut!((*FDODeviceExtension).ResetPortListHead) as PLIST_ENTRY;
        ResetActive = core::ptr::addr_of_mut!((*FDODeviceExtension).ResetPortActive) as PUCHAR;
    } else {
        /* Use cycle port list */
        ListHead = core::ptr::addr_of_mut!((*FDODeviceExtension).CyclePortListHead) as PLIST_ENTRY;
        ResetActive = core::ptr::addr_of_mut!((*FDODeviceExtension).CyclePortActive) as PUCHAR;
    }

    /* Acquire lock */
    KeAcquireSpinLock(core::ptr::addr_of_mut!((*FDODeviceExtension).Lock) as _, core::ptr::addr_of_mut!(OldLevel) as _);

    if *ResetActive != 0 {
        /* Insert into pending list */
        InsertTailList(ListHead, core::ptr::addr_of_mut!((*Irp).Tail.Overlay.__bindgen_anon_2.ListEntry));

        /* Mark irp pending */
        IoMarkIrpPending(Irp);
        Status = STATUS_PENDING;

        /* Release lock */
        KeReleaseSpinLock(core::ptr::addr_of_mut!((*FDODeviceExtension).Lock) as _, OldLevel as _);
    } else {
        /* Mark reset active */
        *ResetActive = 1 as u8;

        /* Release lock */
        KeReleaseSpinLock(core::ptr::addr_of_mut!((*FDODeviceExtension).Lock) as _, OldLevel as _);

        /* Forward request synchronized */
        if IoForwardIrpSynchronously((*FDODeviceExtension).NextDeviceObject, Irp) == 0 {
            NT_VERIFY(0);
        }

        /* Reacquire lock */
        KeAcquireSpinLock(core::ptr::addr_of_mut!((*FDODeviceExtension).Lock) as _, core::ptr::addr_of_mut!(OldLevel) as _);

        /* Mark reset as completed */
        *ResetActive = 0 as u8;

        /* Move all requests into temporary list */
        InitializeListHead(core::ptr::addr_of_mut!(TempList));
        while !IsListEmpty(ListHead) {
            Entry = RemoveHeadList(ListHead);
            InsertTailList(core::ptr::addr_of_mut!(TempList), Entry);
        }

        /* Release lock */
        KeReleaseSpinLock(core::ptr::addr_of_mut!((*FDODeviceExtension).Lock) as _, OldLevel as _);

        /* Complete pending irps */
        while !IsListEmpty(core::ptr::addr_of_mut!(TempList)) {
            Entry = RemoveHeadList(core::ptr::addr_of_mut!(TempList));
            ListIrp = ((Entry as usize)
                .wrapping_sub(
                    &raw const (*(core::ptr::null::<IRP>())).Tail.Overlay.__bindgen_anon_2.ListEntry as usize,
                ) /* CONTAINING_RECORD(Entry, IRP, Tail.Overlay.ListEntry) */)
                as PIRP;
            /* CONTAINING_RECORD(Entry, IRP, Tail.Overlay.ListEntry) */
            ListIrp = ((Entry as usize)
                .wrapping_sub(
                    &raw const (*(core::ptr::null::<IRP>())).Tail.Overlay.__bindgen_anon_2.ListEntry as usize,
                )) as PIRP;

            /* Complete request with status success */
            (*ListIrp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
            IoCompleteRequest(ListIrp, IO_NO_INCREMENT as i8);
        }

        /* Status success */
        Status = STATUS_SUCCESS;
    }

    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn FDO_HandleInternalDeviceControl(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut IoStack: PIO_STACK_LOCATION;
    let mut Status: NTSTATUS;
    let FDODeviceExtension: PFDO_DEVICE_EXTENSION;

    // CTL_CODE(FILE_DEVICE_USB=0x22, USB_RESET_PORT=1, METHOD_NEITHER=3, FILE_ANY_ACCESS=0)
    const IOCTL_INTERNAL_USB_RESET_PORT: u32 = 0x00220007;
    // CTL_CODE(FILE_DEVICE_USB=0x22, USB_CYCLE_PORT=7, METHOD_NEITHER=3, FILE_ANY_ACCESS=0)
    const IOCTL_INTERNAL_USB_CYCLE_PORT: u32 = 0x0022001F;

    /* Get device extension */
    FDODeviceExtension = (*DeviceObject).DeviceExtension as PFDO_DEVICE_EXTENSION;
    // ASSERT(FDODeviceExtension->Common.IsFDO);

    /* Get stack location */
    IoStack = IoGetCurrentIrpStackLocation(Irp);

    if (*IoStack).Parameters.DeviceIoControl.IoControlCode == IOCTL_INTERNAL_USB_RESET_PORT
        || (*IoStack).Parameters.DeviceIoControl.IoControlCode == IOCTL_INTERNAL_USB_CYCLE_PORT
    {
        /* Handle reset / cycle ports */
        Status = FDO_HandleResetCyclePort(DeviceObject, Irp);
        // DPRINT("FDO_HandleResetCyclePort Status %x\n", Status);
        if Status != STATUS_PENDING {
            /* Complete request */
            (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
            IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
        }
        return Status;
    }

    /* Forward and forget request */
    IoSkipCurrentIrpStackLocation(Irp);
    return IoCallDriver((*FDODeviceExtension).NextDeviceObject, Irp);
}
#[no_mangle]
pub unsafe extern "C" fn FDO_HandleSystemControl(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut FDODeviceExtension: PFDO_DEVICE_EXTENSION;

    /* Get device extension */
    FDODeviceExtension = (*DeviceObject).DeviceExtension as PFDO_DEVICE_EXTENSION;
    if (*FDODeviceExtension).Common.IsFDO == 0 {
        RtlAssert(
            b"FDODeviceExtension->Common.IsFDO\0".as_ptr() as *const i8 as PVOID,
            b"/home/curi/Projects/os-playground/vibeos-wt-filesystems/drivers/usb/usbccgp/fdo.c\0".as_ptr() as *const i8 as PVOID,
            632 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    }

    /* Forward and forget request */
    IoSkipCurrentIrpStackLocation(Irp);
    return IoCallDriver((*FDODeviceExtension).NextDeviceObject, Irp);
}
#[no_mangle]
pub unsafe extern "C" fn FDO_Dispatch(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut IoStack: PIO_STACK_LOCATION;
    let Status: NTSTATUS;
    let FDODeviceExtension: PFDO_DEVICE_EXTENSION;

    /* Get device extension */
    FDODeviceExtension = (*DeviceObject).DeviceExtension as PFDO_DEVICE_EXTENSION;
    // ASSERT(FDODeviceExtension->Common.IsFDO);

    /* Get stack location */
    IoStack = IoGetCurrentIrpStackLocation(Irp);

    match (*IoStack).MajorFunction {
        IRP_MJ_PNP => {
            return FDO_HandlePnp(DeviceObject, Irp);
        }
        IRP_MJ_INTERNAL_DEVICE_CONTROL => {
            return FDO_HandleInternalDeviceControl(DeviceObject, Irp);
        }
        IRP_MJ_POWER => {
            PoStartNextPowerIrp(Irp);
            IoSkipCurrentIrpStackLocation(Irp);
            return PoCallDriver((*FDODeviceExtension).NextDeviceObject, Irp);
        }
        IRP_MJ_SYSTEM_CONTROL => {
            return FDO_HandleSystemControl(DeviceObject, Irp);
        }
        _ => {
            // DPRINT1("FDO_Dispatch Function %x not implemented\n", IoStack->MajorFunction);
            // ASSERT(FALSE);
            Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
            IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
            return Status;
        }
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS Universal Serial Bus Bulk Enhanced Host Controller Interface
//  * LICENSE:     GPL - See COPYING in the top level directory
//  * FILE:        drivers/usb/usbccgp/fdo.c
//  * PURPOSE:     USB  device driver.
//  * PROGRAMMERS:
//  *              Michael Martin (michael.martin@reactos.org)
//  *              Johannes Anderwald (johannes.anderwald@reactos.org)
//  *              Cameron Gutman
//  */
//
// #include "usbccgp.h"
//
// #define NDEBUG
// #include <debug.h>
//
// NTSTATUS
// NTAPI
// FDO_QueryCapabilitiesCompletionRoutine(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PIRP Irp,
//     IN PVOID Context)
// {
//     /* Set event */
//     KeSetEvent((PRKEVENT)Context, 0, FALSE);
//
//     /* Completion is done in the HidClassFDO_QueryCapabilities routine */
//     return STATUS_MORE_PROCESSING_REQUIRED;
// }
//
// NTSTATUS
// FDO_QueryCapabilities(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN OUT PDEVICE_CAPABILITIES Capabilities)
// {
//     PIRP Irp;
//     KEVENT Event;
//     NTSTATUS Status;
//     PIO_STACK_LOCATION IoStack;
//     PFDO_DEVICE_EXTENSION FDODeviceExtension;
//
//     /* Get device extension */
//     FDODeviceExtension = (PFDO_DEVICE_EXTENSION)DeviceObject->DeviceExtension;
//     ASSERT(FDODeviceExtension->Common.IsFDO);
//
//     /* Init event */
//     KeInitializeEvent(&Event, NotificationEvent, FALSE);
//
//     /* Now allocate the irp */
//     Irp = IoAllocateIrp(DeviceObject->StackSize, FALSE);
//     if (!Irp)
//     {
//         /* No memory */
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* Get next stack location */
//     IoStack = IoGetNextIrpStackLocation(Irp);
//
//     /* Init stack location */
//     IoStack->MajorFunction = IRP_MJ_PNP;
//     IoStack->MinorFunction = IRP_MN_QUERY_CAPABILITIES;
//     IoStack->Parameters.DeviceCapabilities.Capabilities = Capabilities;
//
//     /* Set completion routine */
//     IoSetCompletionRoutine(Irp,
//                            FDO_QueryCapabilitiesCompletionRoutine,
//                            (PVOID)&Event,
//                            TRUE,
//                            TRUE,
//                            TRUE);
//
//     /* Init capabilities */
//     RtlZeroMemory(Capabilities, sizeof(DEVICE_CAPABILITIES));
//     Capabilities->Size = sizeof(DEVICE_CAPABILITIES);
//     Capabilities->Version = 1; // FIXME hardcoded constant
//     Capabilities->Address = MAXULONG;
//     Capabilities->UINumber = MAXULONG;
//
//     /* Pnp irps have default completion code */
//     Irp->IoStatus.Status = STATUS_NOT_SUPPORTED;
//
//     /* Call lower device */
//     Status = IoCallDriver(FDODeviceExtension->NextDeviceObject, Irp);
//     if (Status == STATUS_PENDING)
//     {
//         /* Wait for completion */
//         KeWaitForSingleObject(&Event, Executive, KernelMode, FALSE, NULL);
//     }
//
//     /* Get status */
//     Status = Irp->IoStatus.Status;
//
//     /* Complete request */
//     IoFreeIrp(Irp);
//
//     /* Done */
//     return Status;
// }
//
// NTSTATUS
// FDO_DeviceRelations(
//     PDEVICE_OBJECT DeviceObject,
//     PIRP Irp)
// {
//     ULONG DeviceCount = 0;
//     ULONG Index;
//     PDEVICE_RELATIONS DeviceRelations;
//     PIO_STACK_LOCATION IoStack;
//     PFDO_DEVICE_EXTENSION FDODeviceExtension;
//
//     /* Get device extension */
//     FDODeviceExtension = (PFDO_DEVICE_EXTENSION)DeviceObject->DeviceExtension;
//
//     /* Get current irp stack location */
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     /* Check if relation type is BusRelations */
//     if (IoStack->Parameters.QueryDeviceRelations.Type != BusRelations)
//     {
//         /* FDO always only handles bus relations */
//         return STATUS_SUCCESS;
//     }
//
//     /* Go through array and count device objects */
//     for(Index = 0; Index < FDODeviceExtension->FunctionDescriptorCount; Index++)
//     {
//         if (FDODeviceExtension->ChildPDO[Index])
//         {
//             /* Child pdo */
//             DeviceCount++;
//         }
//     }
//
//     /* Allocate device relations */
//     DeviceRelations = (PDEVICE_RELATIONS)AllocateItem(PagedPool,
//                                                       sizeof(DEVICE_RELATIONS) + (DeviceCount > 1 ? (DeviceCount-1) * sizeof(PDEVICE_OBJECT) : 0));
//     if (!DeviceRelations)
//     {
//         /* No memory */
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* Add device objects */
//     for(Index = 0; Index < FDODeviceExtension->FunctionDescriptorCount; Index++)
//     {
//         if (FDODeviceExtension->ChildPDO[Index])
//         {
//             /* Store child pdo */
//             DeviceRelations->Objects[DeviceRelations->Count] = FDODeviceExtension->ChildPDO[Index];
//
//             /* Add reference */
//             ObReferenceObject(FDODeviceExtension->ChildPDO[Index]);
//
//             /* Increment count */
//             DeviceRelations->Count++;
//         }
//     }
//
//     /* Store result */
//     Irp->IoStatus.Information = (ULONG_PTR)DeviceRelations;
//     Irp->IoStatus.Status = STATUS_SUCCESS;
//
//     /* Request completed successfully */
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// FDO_CreateChildPdo(
//     IN PDEVICE_OBJECT DeviceObject)
// {
//     NTSTATUS Status;
//     PDEVICE_OBJECT PDODeviceObject;
//     PPDO_DEVICE_EXTENSION PDODeviceExtension;
//     PFDO_DEVICE_EXTENSION FDODeviceExtension;
//     ULONG Index;
//
//     /* Get device extension */
//     FDODeviceExtension = (PFDO_DEVICE_EXTENSION)DeviceObject->DeviceExtension;
//     ASSERT(FDODeviceExtension->Common.IsFDO);
//
//     /* Lets create array for the child PDO */
//     FDODeviceExtension->ChildPDO = AllocateItem(NonPagedPool,
//                                                 sizeof(PDEVICE_OBJECT) * FDODeviceExtension->FunctionDescriptorCount);
//     if (!FDODeviceExtension->ChildPDO)
//     {
//         /* No memory */
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* Create pdo for each function */
//     for (Index = 0; Index < FDODeviceExtension->FunctionDescriptorCount; Index++)
//     {
//         if (FDODeviceExtension->FunctionDescriptor[Index].NumberOfInterfaces == 0)
//         {
//             // Ignore invalid devices
//             DPRINT1("[USBCCGP] Found descriptor with 0 interfaces\n");
//             continue;
//         }
//
//         /* Create the PDO */
//         Status = IoCreateDevice(FDODeviceExtension->DriverObject,
//                                 sizeof(PDO_DEVICE_EXTENSION),
//                                 NULL,
//                                 FILE_DEVICE_USB,
//                                 FILE_AUTOGENERATED_DEVICE_NAME,
//                                 FALSE,
//                                 &PDODeviceObject);
//         if (!NT_SUCCESS(Status))
//         {
//             /* Failed to create device object */
//             DPRINT1("IoCreateDevice failed with %x\n", Status);
//             return Status;
//         }
//
//         /* Store in array */
//         FDODeviceExtension->ChildPDO[Index] = PDODeviceObject;
//
//         /* Get device extension */
//         PDODeviceExtension = (PPDO_DEVICE_EXTENSION)PDODeviceObject->DeviceExtension;
//         RtlZeroMemory(PDODeviceExtension, sizeof(PDO_DEVICE_EXTENSION));
//
//         /* Init device extension */
//         PDODeviceExtension->Common.IsFDO = FALSE;
//         PDODeviceExtension->FunctionDescriptor = &FDODeviceExtension->FunctionDescriptor[Index];
//         PDODeviceExtension->NextDeviceObject = DeviceObject;
//         PDODeviceExtension->FunctionIndex = Index;
//         PDODeviceExtension->FDODeviceExtension = FDODeviceExtension;
//         PDODeviceExtension->InterfaceList = FDODeviceExtension->InterfaceList;
//         PDODeviceExtension->InterfaceListCount = FDODeviceExtension->InterfaceListCount;
//         PDODeviceExtension->ConfigurationHandle = FDODeviceExtension->ConfigurationHandle;
//         PDODeviceExtension->ConfigurationDescriptor = FDODeviceExtension->ConfigurationDescriptor;
//         RtlCopyMemory(&PDODeviceExtension->Capabilities, &FDODeviceExtension->Capabilities, sizeof(DEVICE_CAPABILITIES));
//         RtlCopyMemory(&PDODeviceExtension->DeviceDescriptor, FDODeviceExtension->DeviceDescriptor, sizeof(USB_DEVICE_DESCRIPTOR));
//
//         /* Patch the stack size */
//         PDODeviceObject->StackSize = DeviceObject->StackSize + 1;
//
//         /* Set device flags */
//         PDODeviceObject->Flags |= DO_DIRECT_IO | DO_MAP_IO_BUFFER;
//
//         /* Device is initialized */
//         PDODeviceObject->Flags &= ~DO_DEVICE_INITIALIZING;
//     }
//
//     /* Done */
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// FDO_StartDevice(
//     PDEVICE_OBJECT DeviceObject,
//     PIRP Irp)
// {
//     NTSTATUS Status;
//     PFDO_DEVICE_EXTENSION FDODeviceExtension;
//
//     /* Get device extension */
//     FDODeviceExtension = (PFDO_DEVICE_EXTENSION)DeviceObject->DeviceExtension;
//     ASSERT(FDODeviceExtension->Common.IsFDO);
//
//     /* First start lower device */
//     if (IoForwardIrpSynchronously(FDODeviceExtension->NextDeviceObject, Irp))
//     {
//         Status = Irp->IoStatus.Status;
//     }
//     else
//     {
//         Status = STATUS_UNSUCCESSFUL;
//     }
//
//     if (!NT_SUCCESS(Status))
//     {
//         /* Failed to start lower device */
//         DPRINT1("FDO_StartDevice lower device failed to start with %x\n", Status);
//         return Status;
//     }
//
//     /* Get descriptors */
//     Status = USBCCGP_GetDescriptors(DeviceObject);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Failed to start lower device */
//         DPRINT1("FDO_StartDevice failed to get descriptors with %x\n", Status);
//         return Status;
//     }
//
//     /* Get capabilities */
//     Status = FDO_QueryCapabilities(DeviceObject,
//                                    &FDODeviceExtension->Capabilities);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Failed to start lower device */
//         DPRINT1("FDO_StartDevice failed to get capabilities with %x\n", Status);
//         return Status;
//     }
//
//     /* Now select the configuration */
//     Status = USBCCGP_SelectConfiguration(DeviceObject, FDODeviceExtension);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Failed to select interface */
//         DPRINT1("FDO_StartDevice failed to get capabilities with %x\n", Status);
//         return Status;
//     }
//
//     /* Query bus interface */
//     USBCCGP_QueryInterface(FDODeviceExtension->NextDeviceObject,
//                            &FDODeviceExtension->BusInterface);
//
//     /* Now enumerate the functions */
//     Status = USBCCGP_EnumerateFunctions(DeviceObject);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Failed to enumerate functions */
//         DPRINT1("Failed to enumerate functions with %x\n", Status);
//         return Status;
//     }
//
//     /* Sanity checks */
//     ASSERT(FDODeviceExtension->FunctionDescriptorCount);
//     ASSERT(FDODeviceExtension->FunctionDescriptor);
//     DumpFunctionDescriptor(FDODeviceExtension->FunctionDescriptor,
//                            FDODeviceExtension->FunctionDescriptorCount);
//
//     /* Now create the pdo */
//     Status = FDO_CreateChildPdo(DeviceObject);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Failed */
//         DPRINT1("FDO_CreateChildPdo failed with %x\n", Status);
//         return Status;
//     }
//
//     /* Inform pnp manager of new device objects */
//     IoInvalidateDeviceRelations(FDODeviceExtension->PhysicalDeviceObject,
//                                 BusRelations);
//
//     /* Done */
//     DPRINT("[USBCCGP] FDO initialized successfully\n");
//     return Status;
// }
//
// NTSTATUS
// FDO_CloseConfiguration(
//     IN PDEVICE_OBJECT DeviceObject)
// {
//     NTSTATUS Status;
//     PURB Urb;
//     PFDO_DEVICE_EXTENSION FDODeviceExtension;
//
//     /* Get device extension */
//     FDODeviceExtension = (PFDO_DEVICE_EXTENSION)DeviceObject->DeviceExtension;
//     ASSERT(FDODeviceExtension->Common.IsFDO);
//
//     /* Nothing to do if we're not configured */
//     if (FDODeviceExtension->ConfigurationDescriptor == NULL ||
//         FDODeviceExtension->InterfaceList == NULL)
//     {
//         return STATUS_SUCCESS;
//     }
//
//     /* Now allocate the urb */
//     Urb = USBD_CreateConfigurationRequestEx(FDODeviceExtension->ConfigurationDescriptor,
//                                             FDODeviceExtension->InterfaceList);
//     if (!Urb)
//     {
//         /* No memory */
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* Clear configuration descriptor to make it an unconfigure request */
//     Urb->UrbSelectConfiguration.ConfigurationDescriptor = NULL;
//
//     /* Submit urb */
//     Status = USBCCGP_SyncUrbRequest(FDODeviceExtension->NextDeviceObject, Urb);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Failed to set configuration */
//         DPRINT1("USBCCGP_SyncUrbRequest failed to unconfigure device\n", Status);
//     }
//
//     ExFreePool(Urb);
//     return Status;
// }
//
//
// NTSTATUS
// FDO_HandlePnp(
//     PDEVICE_OBJECT DeviceObject,
//     PIRP Irp)
// {
//     PIO_STACK_LOCATION IoStack;
//     NTSTATUS Status;
//     PFDO_DEVICE_EXTENSION FDODeviceExtension;
//
//     /* Get device extension */
//     FDODeviceExtension = (PFDO_DEVICE_EXTENSION)DeviceObject->DeviceExtension;
//     ASSERT(FDODeviceExtension->Common.IsFDO);
//
//
//     /* Get stack location */
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//     DPRINT("[USBCCGP] PnP Minor %x\n", IoStack->MinorFunction);
//     switch(IoStack->MinorFunction)
//     {
//         case IRP_MN_REMOVE_DEVICE:
//         {
//             // Unconfigure device */
//             DPRINT1("[USBCCGP] FDO IRP_MN_REMOVE\n");
//             FDO_CloseConfiguration(DeviceObject);
//
//             /* Send the IRP down the stack */
//             Irp->IoStatus.Status = STATUS_SUCCESS;
//             IoSkipCurrentIrpStackLocation(Irp);
//             Status = IoCallDriver(FDODeviceExtension->NextDeviceObject, Irp);
//
//             /* Detach from the device stack */
//             IoDetachDevice(FDODeviceExtension->NextDeviceObject);
//
//             /* Delete the device object */
//             IoDeleteDevice(DeviceObject);
//
//             /* The lower driver completes the IRP for removal */
//             return Status;
//         }
//         case IRP_MN_START_DEVICE:
//         {
//             /* Start the device */
//             Status = FDO_StartDevice(DeviceObject, Irp);
//             break;
//         }
//         case IRP_MN_QUERY_DEVICE_RELATIONS:
//         {
//             /* Handle device relations */
//             Status = FDO_DeviceRelations(DeviceObject, Irp);
//             if (!NT_SUCCESS(Status))
//             {
//                 break;
//             }
//
//             /* Forward irp to next device object */
//             IoSkipCurrentIrpStackLocation(Irp);
//             return IoCallDriver(FDODeviceExtension->NextDeviceObject, Irp);
//         }
//         case IRP_MN_QUERY_CAPABILITIES:
//         {
//             /* Copy capabilities */
//             RtlCopyMemory(IoStack->Parameters.DeviceCapabilities.Capabilities,
//                           &FDODeviceExtension->Capabilities,
//                           sizeof(DEVICE_CAPABILITIES));
//             Status = STATUS_UNSUCCESSFUL;
//
//             if (IoForwardIrpSynchronously(FDODeviceExtension->NextDeviceObject, Irp))
//             {
//                 Status = Irp->IoStatus.Status;
//                 if (NT_SUCCESS(Status))
//                 {
//                     IoStack->Parameters.DeviceCapabilities.Capabilities->SurpriseRemovalOK = TRUE;
//                 }
//             }
//             break;
//        }
//         case IRP_MN_QUERY_REMOVE_DEVICE:
//         case IRP_MN_QUERY_STOP_DEVICE:
//         {
//             /* Sure */
//             Irp->IoStatus.Status = STATUS_SUCCESS;
//
//             /* Forward irp to next device object */
//             IoSkipCurrentIrpStackLocation(Irp);
//             return IoCallDriver(FDODeviceExtension->NextDeviceObject, Irp);
//         }
//        default:
//        {
//             /* Forward irp to next device object */
//             IoSkipCurrentIrpStackLocation(Irp);
//             return IoCallDriver(FDODeviceExtension->NextDeviceObject, Irp);
//        }
//
//     }
//
//     /* Complete request */
//     Irp->IoStatus.Status = Status;
//     IoCompleteRequest(Irp, IO_NO_INCREMENT);
//     return Status;
// }
//
// NTSTATUS
// FDO_HandleResetCyclePort(
//     PDEVICE_OBJECT DeviceObject,
//     PIRP Irp)
// {
//     PIO_STACK_LOCATION IoStack;
//     NTSTATUS Status;
//     PFDO_DEVICE_EXTENSION FDODeviceExtension;
//     PLIST_ENTRY ListHead, Entry;
//     LIST_ENTRY TempList;
//     PUCHAR ResetActive;
//     PIRP ListIrp;
//     KIRQL OldLevel;
//
//     /* Get device extension */
//     FDODeviceExtension = (PFDO_DEVICE_EXTENSION)DeviceObject->DeviceExtension;
//     ASSERT(FDODeviceExtension->Common.IsFDO);
//
//     /* Get stack location */
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//     DPRINT("FDO_HandleResetCyclePort IOCTL %x\n", IoStack->Parameters.DeviceIoControl.IoControlCode);
//
//     if (IoStack->Parameters.DeviceIoControl.IoControlCode == IOCTL_INTERNAL_USB_RESET_PORT)
//     {
//         /* Use reset port list */
//         ListHead = &FDODeviceExtension->ResetPortListHead;
//         ResetActive = &FDODeviceExtension->ResetPortActive;
//     }
//     else
//     {
//         /* Use cycle port list */
//         ListHead = &FDODeviceExtension->CyclePortListHead;
//         ResetActive = &FDODeviceExtension->CyclePortActive;
//     }
//
//     /* Acquire lock */
//     KeAcquireSpinLock(&FDODeviceExtension->Lock, &OldLevel);
//
//     if (*ResetActive)
//     {
//         /* Insert into pending list */
//         InsertTailList(ListHead, &Irp->Tail.Overlay.ListEntry);
//
//         /* Mark irp pending */
//         IoMarkIrpPending(Irp);
//         Status = STATUS_PENDING;
//
//         /* Release lock */
//         KeReleaseSpinLock(&FDODeviceExtension->Lock, OldLevel);
//     }
//     else
//     {
//         /* Mark reset active */
//         *ResetActive = TRUE;
//
//         /* Release lock */
//         KeReleaseSpinLock(&FDODeviceExtension->Lock, OldLevel);
//
//         /* Forward request synchronized */
//         NT_VERIFY(IoForwardIrpSynchronously(FDODeviceExtension->NextDeviceObject, Irp));
//
//         /* Reacquire lock */
//         KeAcquireSpinLock(&FDODeviceExtension->Lock, &OldLevel);
//
//         /* Mark reset as completed */
//         *ResetActive = FALSE;
//
//         /* Move all requests into temporary list */
//         InitializeListHead(&TempList);
//         while(!IsListEmpty(ListHead))
//         {
//             Entry = RemoveHeadList(ListHead);
//             InsertTailList(&TempList, Entry);
//         }
//
//         /* Release lock */
//         KeReleaseSpinLock(&FDODeviceExtension->Lock, OldLevel);
//
//         /* Complete pending irps */
//         while(!IsListEmpty(&TempList))
//         {
//             Entry = RemoveHeadList(&TempList);
//             ListIrp = (PIRP)CONTAINING_RECORD(Entry, IRP, Tail.Overlay.ListEntry);
//
//             /* Complete request with status success */
//             ListIrp->IoStatus.Status = STATUS_SUCCESS;
//             IoCompleteRequest(ListIrp, IO_NO_INCREMENT);
//         }
//
//         /* Status success */
//         Status = STATUS_SUCCESS;
//     }
//
//     return Status;
// }
//
//
//
// NTSTATUS
// FDO_HandleInternalDeviceControl(
//     PDEVICE_OBJECT DeviceObject,
//     PIRP Irp)
// {
//     PIO_STACK_LOCATION IoStack;
//     NTSTATUS Status;
//     PFDO_DEVICE_EXTENSION FDODeviceExtension;
//
//     /* Get device extension */
//     FDODeviceExtension = (PFDO_DEVICE_EXTENSION)DeviceObject->DeviceExtension;
//     ASSERT(FDODeviceExtension->Common.IsFDO);
//
//     /* Get stack location */
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     if (IoStack->Parameters.DeviceIoControl.IoControlCode == IOCTL_INTERNAL_USB_RESET_PORT ||
//         IoStack->Parameters.DeviceIoControl.IoControlCode == IOCTL_INTERNAL_USB_CYCLE_PORT)
//     {
//         /* Handle reset / cycle ports */
//         Status = FDO_HandleResetCyclePort(DeviceObject, Irp);
//         DPRINT("FDO_HandleResetCyclePort Status %x\n", Status);
//         if (Status != STATUS_PENDING)
//         {
//             /* Complete request */
//             Irp->IoStatus.Status = Status;
//             IoCompleteRequest(Irp, IO_NO_INCREMENT);
//         }
//         return Status;
//     }
//
//     /* Forward and forget request */
//     IoSkipCurrentIrpStackLocation(Irp);
//     return IoCallDriver(FDODeviceExtension->NextDeviceObject, Irp);
// }
//
// NTSTATUS
// FDO_HandleSystemControl(
//     PDEVICE_OBJECT DeviceObject,
//     PIRP Irp)
// {
//     PFDO_DEVICE_EXTENSION FDODeviceExtension;
//
//     /* Get device extension */
//     FDODeviceExtension = (PFDO_DEVICE_EXTENSION)DeviceObject->DeviceExtension;
//     ASSERT(FDODeviceExtension->Common.IsFDO);
//
//     /* Forward and forget request */
//     IoSkipCurrentIrpStackLocation(Irp);
//     return IoCallDriver(FDODeviceExtension->NextDeviceObject, Irp);
// }
//
// NTSTATUS
// FDO_Dispatch(
//     PDEVICE_OBJECT DeviceObject,
//     PIRP Irp)
// {
//     PIO_STACK_LOCATION IoStack;
//     NTSTATUS Status;
//     PFDO_DEVICE_EXTENSION FDODeviceExtension;
//
//     /* Get device extension */
//     FDODeviceExtension = DeviceObject->DeviceExtension;
//     ASSERT(FDODeviceExtension->Common.IsFDO);
//
//     /* Get stack location */
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     switch(IoStack->MajorFunction)
//     {
//         case IRP_MJ_PNP:
//             return FDO_HandlePnp(DeviceObject, Irp);
//         case IRP_MJ_INTERNAL_DEVICE_CONTROL:
//             return FDO_HandleInternalDeviceControl(DeviceObject, Irp);
//         case IRP_MJ_POWER:
//             PoStartNextPowerIrp(Irp);
//             IoSkipCurrentIrpStackLocation(Irp);
//             return PoCallDriver(FDODeviceExtension->NextDeviceObject, Irp);
//         case IRP_MJ_SYSTEM_CONTROL:
//             return FDO_HandleSystemControl(DeviceObject, Irp);
//         default:
//             DPRINT1("FDO_Dispatch Function %x not implemented\n", IoStack->MajorFunction);
//             ASSERT(FALSE);
//             Status = Irp->IoStatus.Status;
//             IoCompleteRequest(Irp, IO_NO_INCREMENT);
//             return Status;
//     }
//
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================