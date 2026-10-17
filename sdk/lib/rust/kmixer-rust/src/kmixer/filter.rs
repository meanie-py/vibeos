// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
pub static RUSTCONV_WSTR_46CDC0812373: [u16; 7] = [0x0047u16, 0x004C, 0x004F, 0x0042, 0x0041, 0x004C, 0x0000];

use crate::prelude::*;
use ntkrnl_sys::ddk::{ExAllocatePool, ExFreePool, PDEVICE_OBJECT, PIRP, RtlInitUnicodeString};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static KSSTRING_Filter: [::core::ffi::c_ushort; 39] = unsafe {
    ::core::mem::transmute::<
        [u8; 78],
        [::core::ffi::c_ushort; 39],
    >(
        *b"{\09\0B\x003\x006\x005\08\09\x000\0-\x001\x006\x005\0F\0-\x001\x001\0D\x000\0-\0A\x001\09\x005\0-\x000\x000\x002\x000\0A\0F\0D\x001\x005\x006\0E\x004\0}\0\0\0",
    )
};

pub static KSSTRING_Pin: [::core::ffi::c_ushort; 39] = unsafe {
    ::core::mem::transmute::<
        [u8; 78],
        [::core::ffi::c_ushort; 39],
    >(
        *b"{\x001\x004\x006\0F\x001\0A\08\x000\0-\x004\x007\09\x001\0-\x001\x001\0D\x000\0-\0A\x005\0D\x006\0-\x002\08\0D\0B\x000\x004\0C\x001\x000\x000\x000\x000\0}\0\0\0",
    )
};

pub static KSSTRING_Allocator: [::core::ffi::c_ushort; 39] = unsafe {
    ::core::mem::transmute::<
        [u8; 78],
        [::core::ffi::c_ushort; 39],
    >(
        *b"{\x006\x004\x002\0F\x005\0D\x000\x000\0-\x004\x007\09\x001\0-\x001\x001\0D\x000\0-\0A\x005\0D\x006\0-\x002\08\0D\0B\x000\x004\0C\x001\x000\x000\x000\x000\0}\0\0\0",
    )
};

static mut DispatchTable: KSDISPATCH_TABLE = unsafe {
    KSDISPATCH_TABLE {
        DeviceIoControl: Some(
            Dispatch_fnDeviceIoControl,
        ),
        Read: Some(
            KsDispatchInvalidDeviceRequest,
        ),
        Write: Some(
            KsDispatchInvalidDeviceRequest,
        ),
        Flush: Some(
            KsDispatchInvalidDeviceRequest,
        ),
        Close: Some(Dispatch_fnClose),
        QuerySecurity: Some(
            KsDispatchInvalidDeviceRequest,
        ),
        SetSecurity: Some(
            KsDispatchInvalidDeviceRequest,
        ),
        FastDeviceIoControl: Some(
            KsDispatchFastIoDeviceControlFailure,
        ),
        FastRead: Some(
            KsDispatchFastReadFailure,
        ),
        FastWrite: Some(
            KsDispatchFastReadFailure,
        ),
    }
};

#[no_mangle]
pub unsafe extern "system" fn Dispatch_fnDeviceIoControl(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    UNIMPLEMENTED!();
    DbgBreakPoint();

    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_UNSUCCESSFUL;
    (*Irp).IoStatus.Information = 0;
    IoCompleteRequest(Irp, (IO_NO_INCREMENT as i8));
    return STATUS_UNSUCCESSFUL;
}

#[no_mangle]
pub unsafe extern "system" fn Dispatch_fnClose(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    UNIMPLEMENTED!();

    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    (*Irp).IoStatus.Information = 0;
    IoCompleteRequest(Irp, (IO_NO_INCREMENT as i8));
    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "system" fn DispatchCreateKMixPin(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut Status: NTSTATUS;

    DPRINT!("DispatchCreateKMix entered\n");

    /* create the pin */
    Status = CreatePin(Irp);

    /* save result */
    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    /* complete the request */
    IoCompleteRequest(Irp, (IO_NO_INCREMENT as i8));
    /* done */
    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn DispatchCreateKMixAllocator(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut Status: NTSTATUS;

    /* create the allocator */
    Status = KsCreateDefaultAllocator(Irp);

    /* save result */
    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    /* complete the request */
    IoCompleteRequest(Irp, (IO_NO_INCREMENT as i8));
    /* done */
    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn DispatchCreateKMix(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut ObjectHeader: KSOBJECT_HEADER = core::ptr::null_mut();
    let mut CreateItem: PKSOBJECT_CREATE_ITEM;
    let mut DeviceExtension: PKMIXER_DEVICE_EXT;

    DPRINT!("DispatchCreateKMix entered\n");

    /* check if the request was from usermode */
    if (*Irp).RequestorMode as u32 == (1i32) as u32 {
        /* deny access from usermode */
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_INVALID_DEVICE_REQUEST;
        (*Irp).IoStatus.Information = 0;
        IoCompleteRequest(Irp, (IO_NO_INCREMENT as i8));
        return STATUS_INVALID_DEVICE_REQUEST;
    }

    /* get device extension */
    DeviceExtension = (*DeviceObject).DeviceExtension as PKMIXER_DEVICE_EXT;

    /* allocate create item */
    CreateItem = ExAllocatePool(
        NonPagedPool,
        (core::mem::size_of::<KSOBJECT_CREATE_ITEM>() * 2) as SIZE_T,
    ) as PKSOBJECT_CREATE_ITEM;
    if CreateItem.is_null() {
        /* not enough memory */
        (*Irp).IoStatus.Information = 0;
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_INSUFFICIENT_RESOURCES;
        IoCompleteRequest(Irp, (IO_NO_INCREMENT as i8));
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    /* zero create struct */
    RtlZeroMemory!(
        CreateItem as *mut core::ffi::c_void,
        core::mem::size_of::<KSOBJECT_CREATE_ITEM>() * 2,
    );

    /* initialize pin create item */
    (*CreateItem.offset(0 as isize)).Create = Some(DispatchCreateKMixPin);
    RtlInitUnicodeString(
        core::ptr::addr_of_mut!((*CreateItem.offset(0 as isize)).ObjectClass),
        KSSTRING_Pin.as_ptr() as PCWSTR,
    );
    (*CreateItem.offset(1 as isize)).Create = Some(DispatchCreateKMixAllocator);
    RtlInitUnicodeString(
        core::ptr::addr_of_mut!((*CreateItem.offset(1 as isize)).ObjectClass),
        KSSTRING_Allocator.as_ptr() as PCWSTR,
    );

    /* allocate object header */
    Status = KsAllocateObjectHeader(
        core::ptr::addr_of_mut!(ObjectHeader),
        2,
        CreateItem,
        Irp,
        core::ptr::addr_of_mut!(DispatchTable),
    );

    if !NT_SUCCESS!(Status) {
        /* failed to allocate object header */
        ExFreePool(CreateItem as PVOID);
        KsDereferenceSoftwareBusObject((*DeviceExtension).KsDeviceHeader);
    }

    DPRINT!("KsAllocateObjectHeader result %x\n", Status);
    /* complete the irp */
    (*Irp).IoStatus.Information = 0;
    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    IoCompleteRequest(Irp, (IO_NO_INCREMENT as i8));
    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn KMixAllocateDeviceHeader(DeviceExtension: PKMIXER_DEVICE_EXT) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut CreateItem: PKSOBJECT_CREATE_ITEM;

    /* allocate create item */
    CreateItem = ExAllocatePool(
        NonPagedPool,
        (::core::mem::size_of::<KSOBJECT_CREATE_ITEM>() as SIZE_T).wrapping_mul(2 as SIZE_T),
    ) as PKSOBJECT_CREATE_ITEM;
    if CreateItem.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    /* initialize create item struct */
    RtlZeroMemory!(
        CreateItem as *mut ::core::ffi::c_void,
        (::core::mem::size_of::<KSOBJECT_CREATE_ITEM>() as SIZE_T).wrapping_mul(2 as SIZE_T),
    );
    (*CreateItem.offset(0 as isize)).Create =
        Some(DispatchCreateKMix);
    RtlInitUnicodeString(
        core::ptr::addr_of_mut!((*CreateItem.offset(0 as isize)).ObjectClass),
        RUSTCONV_WSTR_46CDC0812373.as_ptr(),
    );
    (*CreateItem.offset(1 as isize)).Create =
        Some(DispatchCreateKMix);
    RtlInitUnicodeString(
        core::ptr::addr_of_mut!((*CreateItem.offset(1 as isize)).ObjectClass),
        KSSTRING_Filter.as_ptr() as PCWSTR,
    );

    Status = KsAllocateDeviceHeader(
        core::ptr::addr_of_mut!((*DeviceExtension).KsDeviceHeader),
        2,
        CreateItem,
    );
    return Status;
}
