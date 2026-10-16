// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
pub static RUSTCONV_WSTR_19006A653697: [u16; 22] = [
    0x005Cu16, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x005C, 0x006B, 0x006D,
    0x0069, 0x0078, 0x0065, 0x0072, 0x005C, 0x0047, 0x004C, 0x004F, 0x0042, 0x0041,
    0x004C, 0x0000,
];
pub static RUSTCONV_WSTR_46CDC0812373: [u16; 7] = [0x0047u16, 0x004C, 0x004F, 0x0042, 0x0041, 0x004C, 0x0000];
pub static RUSTCONV_WSTR_E1E6E71B0243: [u16; 59] = [
    0x005Cu16, 0x0052, 0x0065, 0x0067, 0x0069, 0x0073, 0x0074, 0x0072, 0x0079, 0x005C,
    0x004D, 0x0061, 0x0063, 0x0068, 0x0069, 0x006E, 0x0065, 0x005C, 0x0053, 0x0079,
    0x0073, 0x0074, 0x0065, 0x006D, 0x005C, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065,
    0x006E, 0x0074, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F, 0x006C, 0x0053,
    0x0065, 0x0074, 0x005C, 0x0053, 0x0065, 0x0072, 0x0076, 0x0069, 0x0063, 0x0065,
    0x0073, 0x005C, 0x006B, 0x006D, 0x0069, 0x0078, 0x0065, 0x0072, 0x0000,
];

use crate::prelude::*;
use ntkrnl_sys::ddk::{PDEVICE_OBJECT, PIO_STACK_LOCATION, PIRP, RtlInitUnicodeString, ZwLoadDriver};
use core::ffi::c_void;
use crate::types::*;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static KSSTRING_Pin: [::core::ffi::c_ushort; 39] = unsafe {
    ::core::mem::transmute::<
        [u8; 78],
        [::core::ffi::c_ushort; 39],
    >(
        *b"{\x001\x004\x006\0F\x001\0A\08\x000\0-\x004\x007\09\x001\0-\x001\x001\0D\x000\0-\0A\x005\0D\x006\0-\x002\08\0D\0B\x000\x004\0C\x001\x000\x000\x000\x000\0}\0\0\0",
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

// #define NDEBUG  (flag-only macro, no value to emit)
#[no_mangle]
pub unsafe extern "system" fn Dispatch_fnDeviceIoControl(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut IoStack: PIO_STACK_LOCATION;

    IoStack = IoGetCurrentIrpStackLocation(Irp);
    if (*IoStack).Parameters.DeviceIoControl.IoControlCode == IOCTL_KS_PROPERTY {
        return SysAudioHandleProperty(DeviceObject, Irp);
    }

    /* unsupported request */
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
    DPRINT!("Dispatch_fnClose called DeviceObject %p Irp %p\n", DeviceObject);

    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    (*Irp).IoStatus.Information = 0;
    IoCompleteRequest(Irp, (IO_NO_INCREMENT as i8));
    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "system" fn DispatchCreateSysAudio(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut ObjectHeader: KSOBJECT_HEADER = core::ptr::null_mut();
    let mut CreateItem: PKSOBJECT_CREATE_ITEM;

    DPRINT!("DispatchCreateSysAudio entered\n");

    /* allocate create item */
    CreateItem = AllocateItem(NonPagedPool, core::mem::size_of::<KSOBJECT_CREATE_ITEM>() as SIZE_T) as PKSOBJECT_CREATE_ITEM;
    if CreateItem.is_null() {
        (*Irp).IoStatus.Information = 0 as ULONG_PTR;
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_INSUFFICIENT_RESOURCES;
        IoCompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    /* zero create struct */
    RtlZeroMemory!(CreateItem as *mut core::ffi::c_void, core::mem::size_of::<KSOBJECT_CREATE_ITEM>() as SIZE_T);

    /* setup create context */
    (*CreateItem).Create = Some(DispatchCreateSysAudioPin);
    RtlInitUnicodeString(core::ptr::addr_of_mut!((*CreateItem).ObjectClass), KSSTRING_Pin.as_ptr() as PCWSTR);

    /* allocate object header */
    Status = KsAllocateObjectHeader(core::ptr::addr_of_mut!(ObjectHeader), 1 as ULONG, CreateItem, Irp, core::ptr::addr_of_mut!(DispatchTable));

    DPRINT!("KsAllocateObjectHeader result %x\n", Status);
    /* complete the irp */
    (*Irp).IoStatus.Information = 0 as ULONG_PTR;
    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    IoCompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);
    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn SysAudioAllocateDeviceHeader(
    mut DeviceExtension: *mut SYSAUDIODEVEXT,
) -> NTSTATUS {
    let mut Status: NTSTATUS = 0;
    let mut CreateItem: PKSOBJECT_CREATE_ITEM = core::ptr::null_mut();

    /* allocate create item */
    CreateItem = AllocateItem(
        NonPagedPool,
        core::mem::size_of::<KSOBJECT_CREATE_ITEM>() as SIZE_T,
    ) as PKSOBJECT_CREATE_ITEM;
    if CreateItem.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    /* initialize create item struct */
    RtlZeroMemory!(
        CreateItem as *mut core::ffi::c_void,
        core::mem::size_of::<KSOBJECT_CREATE_ITEM>() as SIZE_T,
    );
    (*CreateItem).Create =
        Some(DispatchCreateSysAudio);

    /* FIXME Sysaudio doesnt need a named create item because it installs itself
     * via the device interface
     */
    RtlInitUnicodeString(&raw mut (*CreateItem).ObjectClass, RUSTCONV_WSTR_46CDC0812373.as_ptr());
    (*CreateItem).Flags = KSCREATE_ITEM_WILDCARD;

    Status = KsAllocateDeviceHeader(
        &raw mut (*DeviceExtension).KsDeviceHeader,
        1,
        CreateItem,
    );
    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn SysAudioOpenKMixer(DeviceExtension: *mut SYSAUDIODEVEXT) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut DeviceInstanceName: _UNICODE_STRING = RTL_CONSTANT_STRING!(RUSTCONV_WSTR_19006A653697.as_ptr());
    let mut DevicePath: _UNICODE_STRING = RTL_CONSTANT_STRING!(RUSTCONV_WSTR_E1E6E71B0243.as_ptr());

    Status = ZwLoadDriver(core::ptr::addr_of_mut!(DevicePath));

    if NT_SUCCESS!(Status) {
        Status = OpenDevice(
            core::ptr::addr_of_mut!(DeviceInstanceName),
            core::ptr::addr_of_mut!((*DeviceExtension).KMixerHandle),
            core::ptr::addr_of_mut!((*DeviceExtension).KMixerFileObject),
        );
        if !NT_SUCCESS!(Status) {
            (*DeviceExtension).KMixerHandle = core::ptr::null_mut();
            (*DeviceExtension).KMixerFileObject = core::ptr::null_mut();
        }
    }

    DPRINT!(
        "Status %lx KMixerHandle %p KMixerFileObject %p\n",
        Status,
        (*DeviceExtension).KMixerHandle,
        (*DeviceExtension).KMixerFileObject
    );
    return STATUS_SUCCESS;
}
