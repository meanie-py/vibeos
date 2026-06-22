use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ddk::{CCHAR, GUID, IoCreateDevice, IoDeleteDevice, PCSTR, PDEVICE_OBJECT, PDRIVER_OBJECT, PIO_STACK_LOCATION, PIRP, PUNICODE_STRING, _DEVICE_OBJECT, _UNICODE_STRING};
// IoGetCurrentIrpStackLocation / IoSkipCurrentIrpStackLocation are INLINE DDK
// helpers (not ntoskrnl exports) — use the ntkrnl-sys inline impls (as beep does).
use ntkrnl_sys::inline::tier1::{IoGetCurrentIrpStackLocation, IoSkipCurrentIrpStackLocation};
use core::ffi::c_void;

extern "C" {
    fn DbgPrint(format: *const i8, ...) -> i32;
}
// NTAPI imports — stdcall (`_Name@N`) on i386, cdecl on amd64/arm64. The Ks*
// bus-enum entry points are ks.sys exports (KSDDKAPI=NTAPI); RtlAssert is an
// ntoskrnl export (NTAPI). extern "system" gives the correct per-arch decoration.
extern "system" {
    fn KsIsBusEnumChildDevice(DeviceObject: PDEVICE_OBJECT, ChildDevice: *mut u8) -> NTSTATUS;
    fn KsGetBusEnumPnpDeviceObject(DeviceObject: PDEVICE_OBJECT, PnpDeviceObject: *mut PDEVICE_OBJECT) -> NTSTATUS;
    fn KsServiceBusEnumPnpRequest(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS;
    fn RtlAssert(Assertion: PVOID, FileName: PVOID, LineNumber: ULONG, Format: *mut core::ffi::c_char);
    fn KsInstallBusEnumInterface(Irp: PIRP) -> NTSTATUS;
    fn KsRemoveBusEnumInterface(Irp: PIRP) -> NTSTATUS;
    fn KsGetBusEnumIdentifier(Irp: PIRP) -> NTSTATUS;
    fn KsServiceBusEnumCreateRequest(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS;
    fn KsCreateBusEnumObject( Name: *const u16, Fdo: PDEVICE_OBJECT, Pdo: PDEVICE_OBJECT, Reserved: PDEVICE_OBJECT, MediumSetId: *const GUID, FilterListName: *const u16, ) -> NTSTATUS;
}

// ---- file-scope preamble (translated) ----
pub static KSMEDIUMSETID_Standard: GUID = GUID {
    Data1: 0x4747B320,
    Data2: 0x62CE,
    Data3: 0x11CF,
    Data4: [0xA5, 0xD6, 0x28, 0xDB, 0x04, 0xC1, 0x00, 0x00],
};

const IRP_MN_REMOVE_DEVICE: u8 = 0x02;
const IRP_MN_QUERY_RESOURCES: u8 = 0x0A;
const IRP_MN_QUERY_RESOURCE_REQUIREMENTS: u8 = 0x0B;
const IRP_MN_QUERY_PNP_DEVICE_STATE: u8 = 0x14;
const PNP_DEVICE_NOT_DISABLEABLE: u32 = 0x20;

// CTL_CODE(FILE_DEVICE_BUS_EXTENDER=0x2a, fn, method, access) — sdk/include/ddk/swenum.h
// (FILE_DEVICE_BUS_EXTENDER = 0x2a per xdk/iotypes.h; 0x2b is FILE_DEVICE_MODEM).
// CTL_CODE = (DeviceType<<16)|(Access<<14)|(Function<<2)|Method.
const IOCTL_SWENUM_INSTALL_INTERFACE: u32 = 0x002a_0000; // fn 0x000, METHOD_BUFFERED, FILE_ANY_ACCESS
const IOCTL_SWENUM_REMOVE_INTERFACE: u32 = 0x002a_0004; // fn 0x001, METHOD_BUFFERED, FILE_ANY_ACCESS
const IOCTL_SWENUM_GET_BUS_ID: u32 = 0x002a_400b; // fn 0x002, METHOD_NEITHER, FILE_READ_ACCESS

// KSDEVICE_HEADER is typedef PVOID (sdk/include/psdk/ks.h:1007)
type KSDEVICE_HEADER = *mut core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn SwDispatchPower(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut Status: NTSTATUS = 0;
    let mut PnpStatus: NTSTATUS = 0;
    let mut ChildDevice: u8 = 0;
    let mut IoStack: PIO_STACK_LOCATION = core::ptr::null_mut();
    let mut PnpDeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    IoStack = IoGetCurrentIrpStackLocation(Irp);
    Status = KsIsBusEnumChildDevice(DeviceObject, &raw mut ChildDevice);
    PnpStatus = KsGetBusEnumPnpDeviceObject(DeviceObject, &raw mut PnpDeviceObject);
    if Status < 0 || PnpStatus < 0 {
        PoStartNextPowerIrp(Irp);
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
        IoCompleteRequest(Irp, IO_NO_INCREMENT);
        return STATUS_SUCCESS;
    }
    if (*IoStack).MinorFunction == IRP_MN_SET_POWER
        || (*IoStack).MinorFunction == IRP_MN_QUERY_POWER
    {
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    }
    if ChildDevice == 0 {
        PoStartNextPowerIrp(Irp);
        IoSkipCurrentIrpStackLocation(Irp);
        return PoCallDriver(PnpDeviceObject, Irp);
    }
    PoStartNextPowerIrp(Irp);
    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    IoCompleteRequest(Irp, IO_NO_INCREMENT);
    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn SwDispatchPnp(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut Status: NTSTATUS = 0;
    let mut ChildDevice: BOOLEAN = 0;
    let mut IoStack: PIO_STACK_LOCATION = core::ptr::null_mut();
    let mut PnpDeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();

    IoStack = IoGetCurrentIrpStackLocation(Irp);

    Status = KsIsBusEnumChildDevice(DeviceObject, &raw mut ChildDevice);

    if !(Status >= 0) {
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        IoCompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);
        return Status;
    }

    if DbgPrint(
        b"(%s:%d) SwDispatchPnp ChildDevice %u Request %x\n\0".as_ptr() as PCSTR,
        b"drivers/ksfilter/swenum/swenum.c\0".as_ptr()
            as *const core::ffi::c_char,
        107i32,
        ChildDevice as i32,
        (*IoStack).MinorFunction as i32,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
            b"drivers/ksfilter/swenum/swenum.c\0".as_ptr()
                as *const core::ffi::c_char,
            107i32,
        );
    }

    Status = KsServiceBusEnumPnpRequest(DeviceObject, Irp);

    if ChildDevice != 0 {
        if Status != STATUS_NOT_SUPPORTED {
            (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        }
        IoCompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);
        return Status;
    }

    if DbgPrint(
        b"(%s:%d) SwDispatchPnp KsServiceBusEnumPnpRequest Status %x\n\0".as_ptr() as PCSTR,
        b"drivers/ksfilter/swenum/swenum.c\0".as_ptr()
            as *const core::ffi::c_char,
        128i32,
        Status,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
            b"drivers/ksfilter/swenum/swenum.c\0".as_ptr()
                as *const core::ffi::c_char,
            128i32,
        );
    }

    if Status >= 0 {
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        IoCompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);
        return Status;
    }

    Status = KsGetBusEnumPnpDeviceObject(DeviceObject, &raw mut PnpDeviceObject);

    if DbgPrint(
        b"(%s:%d) SwDispatchPnp KsGetBusEnumPnpDeviceObject Status %x\n\0".as_ptr() as PCSTR,
        b"drivers/ksfilter/swenum/swenum.c\0".as_ptr()
            as *const core::ffi::c_char,
        141i32,
        Status,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
            b"drivers/ksfilter/swenum/swenum.c\0".as_ptr()
                as *const core::ffi::c_char,
            141i32,
        );
    }

    if !(Status >= 0) {
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_NOT_SUPPORTED;
        IoCompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);
        return STATUS_NOT_SUPPORTED;
    }

    if PnpDeviceObject.is_null() {
        RtlAssert(
            b"PnpDeviceObject\0".as_ptr() as PVOID,
            b"drivers/ksfilter/swenum/swenum.c\0".as_ptr()
                as PVOID,
            153u32,
            core::ptr::null_mut(),
        );
    }

    IoStack = IoGetCurrentIrpStackLocation(Irp);

    if (*IoStack).MinorFunction as i32 == IRP_MN_REMOVE_DEVICE as i32 {
        IoDeleteDevice(DeviceObject);
    } else {
        if (*IoStack).MinorFunction as i32 == IRP_MN_QUERY_RESOURCES as i32
            || (*IoStack).MinorFunction as i32 == IRP_MN_QUERY_RESOURCE_REQUIREMENTS as i32
        {
            (*Irp).IoStatus.Information = 0;
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
            IoSkipCurrentIrpStackLocation(Irp);
            return IoCallDriver(PnpDeviceObject, Irp);
        }

        if (*IoStack).MajorFunction as i32 == IRP_MN_QUERY_PNP_DEVICE_STATE as i32 {
            (*Irp).IoStatus.Information |= PNP_DEVICE_NOT_DISABLEABLE as ntkrnl_sys::ddk::ULONG_PTR;
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
            IoSkipCurrentIrpStackLocation(Irp);
            return IoCallDriver(PnpDeviceObject, Irp);
        }

        if Status == STATUS_NOT_SUPPORTED {
            IoSkipCurrentIrpStackLocation(Irp);
            return IoCallDriver(PnpDeviceObject, Irp);
        }
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    IoCompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn SwDispatchSystemControl(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut Status: NTSTATUS = 0;
    let mut ChildDevice: BOOLEAN = 0;
    let mut PnpDeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    Status = KsIsBusEnumChildDevice(DeviceObject, &raw mut ChildDevice);
    if Status >= 0 as ::core::ffi::c_long {
        if ChildDevice == 0 {
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
            IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
            return STATUS_SUCCESS;
        }
        Status = KsGetBusEnumPnpDeviceObject(DeviceObject, &raw mut PnpDeviceObject);
        if Status >= 0 as ::core::ffi::c_long {
            IoSkipCurrentIrpStackLocation(Irp);
            return IoCallDriver(PnpDeviceObject, Irp);
        }
    }
    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn SwDispatchDeviceControl(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    unsafe {
        let mut IoStack: PIO_STACK_LOCATION = core::ptr::null_mut();
        let mut Status: NTSTATUS = 0;
        IoStack = IoGetCurrentIrpStackLocation(Irp);
        if (*IoStack).Parameters.DeviceIoControl.IoControlCode
            == IOCTL_SWENUM_INSTALL_INTERFACE as u32
        {
            Status = KsInstallBusEnumInterface(Irp);
            if DbgPrint(
                b"(%s:%d) SwDispatchDeviceControl IOCTL_SWENUM_INSTALL_INTERFACE %x\n\0" as *const u8
                    as *const i8,
                b"drivers/ksfilter/swenum/swenum.c\0"
                    as *const u8 as *const i8,
                270 as i32,
                Status,
            ) != 0
            {
                DbgPrint(
                    b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const i8,
                    b"drivers/ksfilter/swenum/swenum.c\0"
                        as *const u8 as *const i8,
                    270 as i32,
                );
            }
        } else if (*IoStack).Parameters.DeviceIoControl.IoControlCode
            == IOCTL_SWENUM_REMOVE_INTERFACE as u32
        {
            Status = KsRemoveBusEnumInterface(Irp);
            if DbgPrint(
                b"(%s:%d) SwDispatchDeviceControl IOCTL_SWENUM_REMOVE_INTERFACE %x\n\0" as *const u8
                    as *const i8,
                b"drivers/ksfilter/swenum/swenum.c\0"
                    as *const u8 as *const i8,
                276 as i32,
                Status,
            ) != 0
            {
                DbgPrint(
                    b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const i8,
                    b"drivers/ksfilter/swenum/swenum.c\0"
                        as *const u8 as *const i8,
                    276 as i32,
                );
            }
        } else if (*IoStack).Parameters.DeviceIoControl.IoControlCode == IOCTL_SWENUM_GET_BUS_ID as u32
        {
            Status = KsGetBusEnumIdentifier(Irp);
            if DbgPrint(
                b"(%s:%d) SwDispatchDeviceControl IOCTL_SWENUM_GET_BUS_ID %x\n\0" as *const u8
                    as *const i8,
                b"drivers/ksfilter/swenum/swenum.c\0"
                    as *const u8 as *const i8,
                283 as i32,
                Status,
            ) != 0
            {
                DbgPrint(
                    b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const i8,
                    b"drivers/ksfilter/swenum/swenum.c\0"
                        as *const u8 as *const i8,
                    283 as i32,
                );
            }
        } else {
            if DbgPrint(
                b"(%s:%d) SwDispatchDeviceControl Unknown IOCTL %x\n\0" as *const u8 as *const i8,
                b"drivers/ksfilter/swenum/swenum.c\0"
                    as *const u8 as *const i8,
                287 as i32,
                (*IoStack).Parameters.DeviceIoControl.IoControlCode,
            ) != 0
            {
                DbgPrint(
                    b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const i8,
                    b"drivers/ksfilter/swenum/swenum.c\0"
                        as *const u8 as *const i8,
                    287 as i32,
                );
            }
            Status = STATUS_INVALID_PARAMETER;
        }
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
        return Status;
    }
}

#[no_mangle]
pub unsafe extern "system" fn SwDispatchCreate(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut Status: NTSTATUS = 0;
    let mut ChildDevice: BOOLEAN = 0;
    Status = KsIsBusEnumChildDevice(DeviceObject, &raw mut ChildDevice);
    if DbgPrint(
        b"(%s:%d) SwDispatchCreate %x\n\0".as_ptr() as PCSTR,
        b"drivers/ksfilter/swenum/swenum.c\0".as_ptr()
            as *const core::ffi::c_char,
        314i32,
        Status,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
            b"drivers/ksfilter/swenum/swenum.c\0".as_ptr()
                as *const core::ffi::c_char,
            314i32,
        );
    }
    if NT_SUCCESS(Status) {
        if ChildDevice != 0 {
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_OBJECT_NAME_NOT_FOUND;
            IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
            return STATUS_OBJECT_NAME_NOT_FOUND;
        }
        Status = KsServiceBusEnumCreateRequest(DeviceObject, Irp);
        if DbgPrint(
            b"(%s:%d) SwDispatchCreate %x\n\0".as_ptr() as PCSTR,
            b"drivers/ksfilter/swenum/swenum.c\0".as_ptr()
                as *const core::ffi::c_char,
            328i32,
            Status,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
                b"drivers/ksfilter/swenum/swenum.c\0".as_ptr()
                    as *const core::ffi::c_char,
                328i32,
            );
        }
    }
    if Status != STATUS_PENDING {
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
    }
    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn SwDispatchClose(
    mut DeviceObject: PDEVICE_OBJECT,
    mut Irp: PIRP,
) -> NTSTATUS {
    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn SwAddDevice(
    DriverObject: PDRIVER_OBJECT,
    PhysicalDeviceObject: PDEVICE_OBJECT,
) -> NTSTATUS {
    let mut Status: NTSTATUS = 0;
    let mut FunctionalDeviceObject: PDEVICE_OBJECT = core::ptr::null_mut::<_DEVICE_OBJECT>();

    // DPRINT expansion (from c2rust anchor)
    if DbgPrint(
        b"(%s:%d) SWENUM AddDevice\n\0".as_ptr() as PCSTR,
        b"drivers/ksfilter/swenum/swenum.c\0"
            as *const u8
            as *const ::core::ffi::c_char,
        369 as i32,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
            b"drivers/ksfilter/swenum/swenum.c\0"
                as *const u8
                as *const ::core::ffi::c_char,
            369 as i32,
        );
    }

    Status = IoCreateDevice(
        DriverObject,
        core::mem::size_of::<KSDEVICE_HEADER>() as u32,
        core::ptr::null_mut::<_UNICODE_STRING>(),
        0x0000002a /* FILE_DEVICE_BUS_EXTENDER */ as u32,
        0 as u32,
        0u8, // FALSE as BOOLEAN
        &raw mut FunctionalDeviceObject,
    );

    if !(Status >= 0) {
        return Status;
    }

    let sw_name: [u16; 3] = ::core::mem::transmute::<[u8; 6], [u16; 3]>(*b"S\0W\0\0\0");
    let devices_name: [u16; 8] =
        ::core::mem::transmute::<[u8; 16], [u16; 8]>(*b"D\0e\0v\0i\0c\0e\0s\0\0\0");

    Status = KsCreateBusEnumObject(
        &raw const sw_name as *const u16, // L"SW"
        FunctionalDeviceObject,
        PhysicalDeviceObject,
        core::ptr::null_mut::<_DEVICE_OBJECT>(),
        &raw const KSMEDIUMSETID_Standard,
        &raw const devices_name as *const u16, // L"Devices"
    );

    if Status >= 0 {
        (*FunctionalDeviceObject).Flags |= DO_POWER_PAGABLE as u32;
        (*FunctionalDeviceObject).Flags &= !(DO_DEVICE_INITIALIZING as u32);
    } else {
        IoDeleteDevice(FunctionalDeviceObject);
    }

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn SwUnload(DriverObject: PDRIVER_OBJECT) {}

#[no_mangle]
pub unsafe extern "system" fn DriverEntry(
    DriverObject: PDRIVER_OBJECT,
    RegistryPathName: PUNICODE_STRING,
) -> NTSTATUS {
    const IRP_MJ_CREATE: u32 = 0x00;
    const IRP_MJ_CLOSE: u32 = 0x02;
    const IRP_MJ_PNP: u32 = 0x1b;
    const IRP_MJ_POWER: u32 = 0x16;
    const IRP_MJ_DEVICE_CONTROL: u32 = 0x0e;
    const IRP_MJ_SYSTEM_CONTROL: u32 = 0x17;

    let add_device: unsafe extern "system" fn(PDRIVER_OBJECT, PDEVICE_OBJECT) -> NTSTATUS = SwAddDevice;
    (*(*DriverObject).DriverExtension).AddDevice = unsafe { core::mem::transmute(Some(add_device)) };
    let unload: unsafe extern "system" fn(PDRIVER_OBJECT) = SwUnload;
    (*DriverObject).DriverUnload = unsafe { core::mem::transmute(Some(unload)) };
    let dispatch_create: unsafe extern "system" fn(PDEVICE_OBJECT, PIRP) -> NTSTATUS = SwDispatchCreate;
    (*DriverObject).MajorFunction[IRP_MJ_CREATE as usize] = unsafe { core::mem::transmute(Some(dispatch_create)) };
    let dispatch_close: unsafe extern "system" fn(PDEVICE_OBJECT, PIRP) -> NTSTATUS = SwDispatchClose;
    (*DriverObject).MajorFunction[IRP_MJ_CLOSE as usize] = unsafe { core::mem::transmute(Some(dispatch_close)) };
    let dispatch_pnp: unsafe extern "system" fn(PDEVICE_OBJECT, PIRP) -> NTSTATUS = SwDispatchPnp;
    (*DriverObject).MajorFunction[IRP_MJ_PNP as usize] = unsafe { core::mem::transmute(Some(dispatch_pnp)) };
    let dispatch_power: unsafe extern "system" fn(PDEVICE_OBJECT, PIRP) -> NTSTATUS = SwDispatchPower;
    (*DriverObject).MajorFunction[IRP_MJ_POWER as usize] = unsafe { core::mem::transmute(Some(dispatch_power)) };
    let dispatch_devctl: unsafe extern "system" fn(PDEVICE_OBJECT, PIRP) -> NTSTATUS = SwDispatchDeviceControl;
    (*DriverObject).MajorFunction[IRP_MJ_DEVICE_CONTROL as usize] = unsafe { core::mem::transmute(Some(dispatch_devctl)) };
    let dispatch_sysctl: unsafe extern "system" fn(PDEVICE_OBJECT, PIRP) -> NTSTATUS = SwDispatchSystemControl;
    (*DriverObject).MajorFunction[IRP_MJ_SYSTEM_CONTROL as usize] = unsafe { core::mem::transmute(Some(dispatch_sysctl)) };

    if DbgPrint(
        b"(%s:%d) SWENUM loaded\n\0" as *const u8 as *const i8,
        b"drivers/ksfilter/swenum/swenum.c\0"
            as *const u8 as *const i8,
        428 as i32,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const i8,
            b"drivers/ksfilter/swenum/swenum.c\0"
                as *const u8 as *const i8,
            428 as i32,
        );
    }

    return STATUS_SUCCESS;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS Kernel Streaming
//  * FILE:            drivers/ksfilter/swenum/swenum.c
//  * PURPOSE:         KS Allocator functions
//  * PROGRAMMER:      Johannes Anderwald
//  */
//
//
// #include "precomp.h"
//
// const GUID KSMEDIUMSETID_Standard = {0x4747B320L, 0x62CE, 0x11CF, {0xA5, 0xD6, 0x28, 0xDB, 0x04, 0xC1, 0x00, 0x00}};
//
// NTSTATUS
// NTAPI
// SwDispatchPower(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PIRP Irp)
// {
//     NTSTATUS Status, PnpStatus;
//     BOOLEAN ChildDevice;
//     PIO_STACK_LOCATION IoStack;
//     PDEVICE_OBJECT PnpDeviceObject = NULL;
//
//     /* get current stack location */
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     /* check if the device object is a child device */
//     Status = KsIsBusEnumChildDevice(DeviceObject, &ChildDevice);
//
//     /* get bus enum pnp object */
//     PnpStatus = KsGetBusEnumPnpDeviceObject(DeviceObject, &PnpDeviceObject);
//
//     /* check for success */
//     if (!NT_SUCCESS(Status) || !NT_SUCCESS(PnpStatus))
//     {
//         /* start next power irp */
//         PoStartNextPowerIrp(Irp);
//
//         /* just complete the irp */
//         Irp->IoStatus.Status = STATUS_SUCCESS;
//
//         /* complete the irp */
//         IoCompleteRequest(Irp, IO_NO_INCREMENT);
//
//         /* done */
//         return STATUS_SUCCESS;
//     }
//
//     if (IoStack->MinorFunction == IRP_MN_SET_POWER || IoStack->MinorFunction == IRP_MN_QUERY_POWER)
//     {
//         /* fake success */
//         Irp->IoStatus.Status = STATUS_SUCCESS;
//     }
//
//     if (!ChildDevice)
//     {
//         /* forward to pnp device object */
//         PoStartNextPowerIrp(Irp);
//
//         /* skip current location */
//         IoSkipCurrentIrpStackLocation(Irp);
//
//         /* done */
//         return PoCallDriver(PnpDeviceObject, Irp);
//     }
//
//     /* start next power irp */
//     PoStartNextPowerIrp(Irp);
//
//     /* just complete the irp */
//     Irp->IoStatus.Status = STATUS_SUCCESS;
//
//     /* complete the irp */
//     IoCompleteRequest(Irp, IO_NO_INCREMENT);
//
//     /* done */
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// SwDispatchPnp(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PIRP Irp)
// {
//     NTSTATUS Status;
//     BOOLEAN ChildDevice;
//     PIO_STACK_LOCATION IoStack;
//     PDEVICE_OBJECT PnpDeviceObject = NULL;
//
//     /* get current stack location */
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     /* check if the device object is a child device */
//     Status = KsIsBusEnumChildDevice(DeviceObject, &ChildDevice);
//
//     /* check for success */
//     if (!NT_SUCCESS(Status))
//     {
//         /* failed */
//         Irp->IoStatus.Status = Status;
//         IoCompleteRequest(Irp, IO_NO_INCREMENT);
//         return Status;
//     }
//
//     DPRINT("SwDispatchPnp ChildDevice %u Request %x\n", ChildDevice, IoStack->MinorFunction);
//
//     /* let ks handle it */
//     Status = KsServiceBusEnumPnpRequest(DeviceObject, Irp);
//
//     /* check if the request was for a pdo */
//     if (ChildDevice)
//     {
//         if (Status != STATUS_NOT_SUPPORTED)
//         {
//             /* store result */
//             Irp->IoStatus.Status = Status;
//         }
//
//         /* complete request */
//         IoCompleteRequest(Irp, IO_NO_INCREMENT);
//
//         /* done */
//         return Status;
//     }
//
//     DPRINT("SwDispatchPnp KsServiceBusEnumPnpRequest Status %x\n", Status);
//
//     if (NT_SUCCESS(Status))
//     {
//         /* invalid request or not supported */
//         Irp->IoStatus.Status = Status;
//         IoCompleteRequest(Irp, IO_NO_INCREMENT);
//         return Status;
//     }
//
//     /* get bus enum pnp object */
//     Status = KsGetBusEnumPnpDeviceObject(DeviceObject, &PnpDeviceObject);
//
//     DPRINT("SwDispatchPnp KsGetBusEnumPnpDeviceObject Status %x\n", Status);
//
//     /* check for success */
//     if (!NT_SUCCESS(Status))
//     {
//         /* failed to get pnp object */
//         Irp->IoStatus.Status = STATUS_NOT_SUPPORTED;
//         IoCompleteRequest(Irp, IO_NO_INCREMENT);
//         return STATUS_NOT_SUPPORTED;
//     }
//
//     /* sanity check */
//     ASSERT(PnpDeviceObject);
//
//     /* get current stack location */
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     if (IoStack->MinorFunction == IRP_MN_REMOVE_DEVICE)
//     {
//         /* delete the device */
//         IoDeleteDevice(DeviceObject);
//     }
//     else
//     {
//         if (IoStack->MinorFunction == IRP_MN_QUERY_RESOURCES || IoStack->MinorFunction == IRP_MN_QUERY_RESOURCE_REQUIREMENTS)
//         {
//             /* no resources required */
//             Irp->IoStatus.Information = 0;
//             Irp->IoStatus.Status = STATUS_SUCCESS;
//
//             /* skip current location */
//             IoSkipCurrentIrpStackLocation(Irp);
//
//             /* call the pnp device object */
//             return IoCallDriver(PnpDeviceObject, Irp);
//         }
//
//         if (IoStack->MajorFunction == IRP_MN_QUERY_PNP_DEVICE_STATE)
//         {
//             /* device cannot be disabled */
//             Irp->IoStatus.Information |= PNP_DEVICE_NOT_DISABLEABLE;
//             Irp->IoStatus.Status = STATUS_SUCCESS;
//
//             /* skip current location */
//             IoSkipCurrentIrpStackLocation(Irp);
//
//             /* call the pnp device object */
//             return IoCallDriver(PnpDeviceObject, Irp);
//         }
//
//         if (Status == STATUS_NOT_SUPPORTED)
//         {
//             /* skip current location */
//             IoSkipCurrentIrpStackLocation(Irp);
//
//             /* call the pnp device object */
//             return IoCallDriver(PnpDeviceObject, Irp);
//         }
//     }
//
//     /* complete the request */
//     Irp->IoStatus.Status = Status;
//     IoCompleteRequest(Irp, IO_NO_INCREMENT);
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// SwDispatchSystemControl(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PIRP Irp)
// {
//     NTSTATUS Status;
//     BOOLEAN ChildDevice;
//     PDEVICE_OBJECT PnpDeviceObject;
//
//     /* check if the device object is a child device */
//     Status = KsIsBusEnumChildDevice(DeviceObject, &ChildDevice);
//
//     /* check for success */
//     if (NT_SUCCESS(Status))
//     {
//         if (!ChildDevice)
//         {
//             /* bus devices dont support internal requests */
//             Irp->IoStatus.Status = STATUS_SUCCESS;
//             IoCompleteRequest(Irp, IO_NO_INCREMENT);
//             return STATUS_SUCCESS;
//         }
//
//         /* get bus enum pnp object */
//         Status = KsGetBusEnumPnpDeviceObject(DeviceObject, &PnpDeviceObject);
//
//         /* check for success */
//         if (NT_SUCCESS(Status))
//         {
//             /* skip current location */
//             IoSkipCurrentIrpStackLocation(Irp);
//             /* call the pnp device object */
//             return IoCallDriver(PnpDeviceObject, Irp);
//         }
//
//     }
//
//     /* complete the request */
//     Irp->IoStatus.Status = Status;
//     IoCompleteRequest(Irp, IO_NO_INCREMENT);
//
//     return Status;
//
// }
//
// NTSTATUS
// NTAPI
// SwDispatchDeviceControl(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PIRP Irp)
// {
//     PIO_STACK_LOCATION IoStack;
//     NTSTATUS Status;
//
//     /* get current stack location */
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     if (IoStack->Parameters.DeviceIoControl.IoControlCode == IOCTL_SWENUM_INSTALL_INTERFACE)
//     {
//         /* install interface */
//         Status = KsInstallBusEnumInterface(Irp);
//         DPRINT("SwDispatchDeviceControl IOCTL_SWENUM_INSTALL_INTERFACE %x\n", Status);
//     }
//     else if (IoStack->Parameters.DeviceIoControl.IoControlCode == IOCTL_SWENUM_REMOVE_INTERFACE)
//     {
//         /* remove interface */
//         Status = KsRemoveBusEnumInterface(Irp);
//         DPRINT("SwDispatchDeviceControl IOCTL_SWENUM_REMOVE_INTERFACE %x\n", Status);
//
//     }
//     else if (IoStack->Parameters.DeviceIoControl.IoControlCode == IOCTL_SWENUM_GET_BUS_ID)
//     {
//         /* get bus id */
//         Status = KsGetBusEnumIdentifier(Irp);
//         DPRINT("SwDispatchDeviceControl IOCTL_SWENUM_GET_BUS_ID %x\n", Status);
//     }
//     else
//     {
//         DPRINT("SwDispatchDeviceControl Unknown IOCTL %x\n", IoStack->Parameters.DeviceIoControl.IoControlCode);
//         Status = STATUS_INVALID_PARAMETER;
//     }
//
//     /* store result */
//     Irp->IoStatus.Status = Status;
//
//     /* complete irp */
//     IoCompleteRequest(Irp, IO_NO_INCREMENT);
//
//     /* done */
//     return Status;
// }
//
//
// NTSTATUS
// NTAPI
// SwDispatchCreate(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PIRP Irp)
// {
//     NTSTATUS Status;
//     BOOLEAN ChildDevice;
//
//     /* check if the device object is a child device */
//     Status = KsIsBusEnumChildDevice(DeviceObject, &ChildDevice);
//
//     DPRINT("SwDispatchCreate %x\n", Status);
//
//     /* check for success */
//     if (NT_SUCCESS(Status))
//     {
//         if (ChildDevice)
//         {
//             /* child devices cant create devices */
//             Irp->IoStatus.Status = STATUS_OBJECT_NAME_NOT_FOUND;
//             IoCompleteRequest(Irp, IO_NO_INCREMENT);
//             return STATUS_OBJECT_NAME_NOT_FOUND;
//         }
//         /* perform the create request */
//         Status = KsServiceBusEnumCreateRequest(DeviceObject, Irp);
//         DPRINT("SwDispatchCreate %x\n", Status);
//     }
//
//     /* check the irp is pending */
//     if (Status != STATUS_PENDING)
//     {
//         /* irp is ok to complete */
//         Irp->IoStatus.Status = Status;
//         IoCompleteRequest(Irp, IO_NO_INCREMENT);
//     }
//
//     return Status;
// }
//
//
// NTSTATUS
// NTAPI
// SwDispatchClose(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PIRP Irp)
// {
//     /* just complete the irp */
//     Irp->IoStatus.Status = STATUS_SUCCESS;
//
//     /* complete the irp */
//     IoCompleteRequest(Irp, IO_NO_INCREMENT);
//
//     /* done */
//     return STATUS_SUCCESS;
//
// }
//
// NTSTATUS
// NTAPI
// SwAddDevice(
//     IN PDRIVER_OBJECT DriverObject,
//     IN PDEVICE_OBJECT PhysicalDeviceObject)
// {
//     NTSTATUS Status;
//     PDEVICE_OBJECT FunctionalDeviceObject;
//
//     DPRINT("SWENUM AddDevice\n");
//     /* create the device */
//     Status = IoCreateDevice(DriverObject, sizeof(KSDEVICE_HEADER), NULL, FILE_DEVICE_BUS_EXTENDER, 0, FALSE, &FunctionalDeviceObject);
//
//     if (!NT_SUCCESS(Status))
//     {
//         /* failed */
//         return Status;
//     }
//
//     /* create the bus enum object */
//     Status = KsCreateBusEnumObject(L"SW", FunctionalDeviceObject, PhysicalDeviceObject, NULL, &KSMEDIUMSETID_Standard, L"Devices");
//
//     /* check for success */
//     if (NT_SUCCESS(Status))
//     {
//         /* set device flags */
//         FunctionalDeviceObject->Flags |= DO_POWER_PAGABLE;
//         FunctionalDeviceObject->Flags &= ~ DO_DEVICE_INITIALIZING;
//     }
//     else
//     {
//         /* failed to create bus enum object */
//         IoDeleteDevice(FunctionalDeviceObject);
//     }
//
//     /* done */
//     return Status;
// }
//
// VOID
// NTAPI
// SwUnload(
//     IN  PDRIVER_OBJECT DriverObject)
// {
//     /* nop */
// }
//
// NTSTATUS
// NTAPI
// DriverEntry(
//     IN PDRIVER_OBJECT DriverObject,
//     IN PUNICODE_STRING RegistryPathName)
// {
//
//     /* setup add device routine */
//     DriverObject->DriverExtension->AddDevice = SwAddDevice;
//
//     /* setup unload routine */
//     DriverObject->DriverUnload = SwUnload;
//
//     /* misc irp handling routines */
//     DriverObject->MajorFunction[IRP_MJ_CREATE] = SwDispatchCreate;
//     DriverObject->MajorFunction[IRP_MJ_CLOSE] = SwDispatchClose;
//     DriverObject->MajorFunction[IRP_MJ_PNP] = SwDispatchPnp;
//     DriverObject->MajorFunction[IRP_MJ_POWER] = SwDispatchPower;
//     DriverObject->MajorFunction[IRP_MJ_DEVICE_CONTROL] = SwDispatchDeviceControl;
//     DriverObject->MajorFunction[IRP_MJ_SYSTEM_CONTROL] = SwDispatchSystemControl;
//
//     DPRINT("SWENUM loaded\n");
//     return STATUS_SUCCESS;
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
