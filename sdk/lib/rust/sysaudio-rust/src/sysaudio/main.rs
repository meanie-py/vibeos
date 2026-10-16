// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
pub static RUSTCONV_WSTR_A5AD37463E7C: [u16; 21] = [
    0x005Cu16, 0x0044, 0x006F, 0x0073, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065,
    0x0073, 0x005C, 0x0073, 0x0079, 0x0073, 0x0061, 0x0075, 0x0064, 0x0069, 0x006F,
    0x0000,
];
pub static RUSTCONV_WSTR_C5890CA79737: [u16; 17] = [
    0x005Cu16, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x005C, 0x0073, 0x0079,
    0x0073, 0x0061, 0x0075, 0x0064, 0x0069, 0x006F, 0x0000,
];

use crate::prelude::*;
use ntkrnl_sys::ddk::{GUID, IoAttachDeviceToDeviceStack, IoCreateDevice, IoCreateSymbolicLink, IoDeleteDevice, IoDeleteSymbolicLink, IoRegisterShutdownNotification, IoUnregisterPlugPlayNotification, PDEVICE_OBJECT, PDRIVER_OBJECT, PIO_STACK_LOCATION, PIRP, PUNICODE_STRING, RtlFreeUnicodeString, ZwClose};
use core::ffi::c_void;
use crate::types::*;

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
pub static mut KSCATEGORY_AUDIO_DEVICE: GUID = _GUID {
    Data1: 0xfbf6f530 as ::core::ffi::c_ulong as ::core::ffi::c_uint,
    Data2: 0x7b9 as ::core::ffi::c_ushort,
    Data3: 0x11d2 as ::core::ffi::c_ushort,
    Data4: [
        0xa7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x1e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x47 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x88 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut KSCATEGORY_PREFERRED_WAVEOUT_DEVICE: GUID = _GUID {
    Data1: 0xd6c5066e as ::core::ffi::c_ulong as ::core::ffi::c_uint,
    Data2: 0x72c1 as ::core::ffi::c_ushort,
    Data3: 0x11d2 as ::core::ffi::c_ushort,
    Data4: [
        0x97 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x55 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x47 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x88 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut KSCATEGORY_PREFERRED_WAVEIN_DEVICE: GUID = _GUID {
    Data1: 0xd6c50671 as ::core::ffi::c_ulong as ::core::ffi::c_uint,
    Data2: 0x72c1 as ::core::ffi::c_ushort,
    Data3: 0x11d2 as ::core::ffi::c_ushort,
    Data4: [
        0x97 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x55 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x47 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x88 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut KSCATEGORY_PREFERRED_MIDIOUT_DEVICE: GUID = _GUID {
    Data1: 0xd6c50674 as ::core::ffi::c_ulong as ::core::ffi::c_uint,
    Data2: 0x72c1 as ::core::ffi::c_ushort,
    Data3: 0x11d2 as ::core::ffi::c_ushort,
    Data4: [
        0x97 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x55 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x47 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x88 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

// #define NDEBUG  (valueless flag, no Rust declaration emitted)

pub const TAG_SYSAUDIO: u32 = 0x41737953; // 'AsyS' (multi-char constant, MSVC order)
#[no_mangle]
pub unsafe extern "C" fn AllocateItem(PoolType: POOL_TYPE, NumberOfBytes: SIZE_T) -> PVOID {
    return ExAllocatePoolZero(PoolType, (NumberOfBytes) as _, TAG_SYSAUDIO as ULONG);
}
#[no_mangle]
pub unsafe extern "C" fn FreeItem(Item: PVOID) {
    ExFreePoolWithTag(Item, TAG_SYSAUDIO as ULONG);
}

#[no_mangle]
pub unsafe extern "system" fn SysAudio_Unload(DriverObject: PDRIVER_OBJECT) {
    // DPRINT("SysAudio_Unload called\n");
}
#[no_mangle]
pub unsafe extern "system" fn SysAudio_Shutdown(
    mut DeviceObject: PDEVICE_OBJECT,
    mut Irp: PIRP,
) -> NTSTATUS {
    let mut DeviceEntry: PKSAUDIO_DEVICE_ENTRY = core::ptr::null_mut();
    let mut DeviceExtension: PSYSAUDIODEVEXT = core::ptr::null_mut();
    let mut Entry: PLIST_ENTRY = core::ptr::null_mut();

    DPRINT!("SysAudio_Shutdown called\n");

    DeviceExtension = (*DeviceObject).DeviceExtension as PSYSAUDIODEVEXT;

    while IsListEmpty(&raw mut (*DeviceExtension).KsAudioDeviceList) == 0 {
        Entry = RemoveHeadList(&raw mut (*DeviceExtension).KsAudioDeviceList);
        DeviceEntry = CONTAINING_RECORD!(Entry, KSAUDIO_DEVICE_ENTRY, Entry) as PKSAUDIO_DEVICE_ENTRY;

        DPRINT!("Freeing item %wZ\n", &raw mut (*DeviceEntry).DeviceName);

        /* dereference audio device file object */
        ObDereferenceObject((*DeviceEntry).FileObject as PVOID);

        /* close audio device handle */
        ZwClose((*DeviceEntry).Handle);

        /* free device string */
        RtlFreeUnicodeString(&raw mut (*DeviceEntry).DeviceName);

        /* free audio device entry */
        FreeItem(DeviceEntry as PVOID);
    }

    (*Irp).IoStatus.Information = 0;
    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    IoCompleteRequest(Irp, (IO_NO_INCREMENT as i8));
    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "system" fn SysAudio_Pnp(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    let mut IrpStack: PIO_STACK_LOCATION = ::core::ptr::null_mut::<IO_STACK_LOCATION>();
    let mut SymlinkName: _UNICODE_STRING = RTL_CONSTANT_STRING!(RUSTCONV_WSTR_A5AD37463E7C.as_ptr());
    let mut DeviceExtension: *mut SYSAUDIODEVEXT = ::core::ptr::null_mut::<SYSAUDIODEVEXT>();

    /* Get current irp stack */
    IrpStack = IoGetCurrentIrpStackLocation(Irp);

    /* Fetch the device extension */
    DeviceExtension = (*DeviceObject).DeviceExtension as *mut SYSAUDIODEVEXT;
    ASSERT!(!DeviceExtension.is_null());

    if (*IrpStack).MinorFunction as ::core::ffi::c_int == (IRP_MN_REMOVE_DEVICE as i32) {
        /* Unregister the echo cancel hook */
        if !(*DeviceExtension).EchoCancelNotificationEntry.is_null() {
            IoUnregisterPlugPlayNotification((*DeviceExtension).EchoCancelNotificationEntry);
        }

        /* Unregister the ks audio hook */
        if !(*DeviceExtension).KsAudioNotificationEntry.is_null() {
            IoUnregisterPlugPlayNotification((*DeviceExtension).KsAudioNotificationEntry);
        }

        /* Destroy our symbolic link */
        IoDeleteSymbolicLink(&raw mut SymlinkName);
    } else if (*IrpStack).MinorFunction as ::core::ffi::c_int == (IRP_MN_QUERY_PNP_DEVICE_STATE as i32) {
        /* Sysaudio can not be disabled */
        (*Irp).IoStatus.Information |= PNP_DEVICE_NOT_DISABLEABLE as ULONG_PTR;
    }

    /* Perform default pnp actions */
    return KsDefaultDispatchPnp(DeviceObject, Irp);
}

#[no_mangle]
pub unsafe extern "system" fn SysAudio_AddDevice(
    DriverObject: PDRIVER_OBJECT,
    PhysicalDeviceObject: PDEVICE_OBJECT,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut DeviceName: _UNICODE_STRING = RTL_CONSTANT_STRING!(RUSTCONV_WSTR_C5890CA79737.as_ptr());
    let mut SymlinkName: _UNICODE_STRING = RTL_CONSTANT_STRING!(RUSTCONV_WSTR_A5AD37463E7C.as_ptr());
    let mut DeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut NextDeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut DeviceExtension: *mut SYSAUDIODEVEXT = core::ptr::null_mut();

    DPRINT!("SysAudio_AddDevice called\n");

    /* Create the device */
    Status = IoCreateDevice(
        DriverObject,
        core::mem::size_of::<SYSAUDIODEVEXT>() as ULONG,
        core::ptr::addr_of_mut!(DeviceName),
        FILE_DEVICE_KS as ULONG,
        0 as ULONG,
        FALSE as BOOLEAN,
        core::ptr::addr_of_mut!(DeviceObject),
    );

    /* Check for success */
    if !NT_SUCCESS!(Status) {
        DPRINT!("Failed to create \\Device\\sysaudio !\n");
        return Status;
    }

    'cleanup: {
        /* Register device interfaces */
        Status = SysAudioRegisterDeviceInterfaces(PhysicalDeviceObject);
        if !NT_SUCCESS!(Status) {
            /* Failed to register
             * Create a hack interface
             */
            Status = IoCreateSymbolicLink(
                core::ptr::addr_of_mut!(SymlinkName),
                core::ptr::addr_of_mut!(DeviceName),
            );
            if !NT_SUCCESS!(Status) {
                IoDeleteDevice(DeviceObject);
                DPRINT1!("Failed to create sysaudio symlink!\n");
                return Status;
            }
        }
        /* Acquire device extension */
        DeviceExtension = (*DeviceObject).DeviceExtension as *mut SYSAUDIODEVEXT;
        /* Initialize device extension */
        RtlZeroMemory!(
            DeviceExtension as *mut core::ffi::c_void,
            core::mem::size_of::<SYSAUDIODEVEXT>() as usize,
        );

        /* Initialize the mutex */
        KeInitializeSpinLock(core::ptr::addr_of_mut!((*DeviceExtension).Lock));

        /* Initialize the ks audio device list */
        InitializeListHead(core::ptr::addr_of_mut!((*DeviceExtension).KsAudioDeviceList));

        /* Allocate kernel streaming device header */
        Status = SysAudioAllocateDeviceHeader(DeviceExtension);
        if !NT_SUCCESS!(Status) {
            DPRINT1!("KsAllocateDeviceHeader failed with %x\n", Status);
            break 'cleanup;
        }

        /* Register device notification hooks */
        Status = SysAudioRegisterNotifications(DriverObject, DeviceObject);
        if !NT_SUCCESS!(Status) {
            DPRINT1!("Failed to register device notifications\n");
            break 'cleanup;
        }

        /* Load kmixer */
        Status = SysAudioOpenKMixer(DeviceExtension);
        if !NT_SUCCESS!(Status) {
            DPRINT1!("SysAudioOpenKMixer failed with %x\n", Status);
            break 'cleanup;
        }

        /* set io flags */
        (*DeviceObject).Flags |= (DO_DIRECT_IO | DO_POWER_PAGABLE) as ULONG;
        /* clear initializing flag */
        (*DeviceObject).Flags &= !(DO_DEVICE_INITIALIZING as ULONG);

        /* atttach to device stack */
        NextDeviceObject = IoAttachDeviceToDeviceStack(DeviceObject, PhysicalDeviceObject);
        KsSetDevicePnpAndBaseObject((*DeviceExtension).KsDeviceHeader, NextDeviceObject, DeviceObject);

        /* register shutdown notification */
        IoRegisterShutdownNotification(DeviceObject);

        /* Done */
        return STATUS_SUCCESS;
    }

    if !(*DeviceExtension).KsAudioNotificationEntry.is_null() {
        IoUnregisterPlugPlayNotification((*DeviceExtension).KsAudioNotificationEntry);
    }

    if !(*DeviceExtension).EchoCancelNotificationEntry.is_null() {
        IoUnregisterPlugPlayNotification((*DeviceExtension).EchoCancelNotificationEntry);
    }

    IoDeleteSymbolicLink(core::ptr::addr_of_mut!(SymlinkName));
    IoDeleteDevice(DeviceObject);
    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn DriverEntry(
    DriverObject: PDRIVER_OBJECT,
    RegistryPath: PUNICODE_STRING,
) -> NTSTATUS {
    DPRINT!("System audio graph builder (sysaudio) started\n");

    /* Let ks handle these */
    KsSetMajorFunctionHandler(DriverObject, IRP_MJ_CREATE as ULONG);
    KsSetMajorFunctionHandler(DriverObject, IRP_MJ_CLOSE as ULONG);
    KsSetMajorFunctionHandler(DriverObject, IRP_MJ_WRITE as ULONG);
    KsSetMajorFunctionHandler(DriverObject, IRP_MJ_DEVICE_CONTROL as ULONG);

    /* Let ks handle these */
    (*DriverObject).MajorFunction[IRP_MJ_POWER as usize] =
        Some(KsDefaultDispatchPower);
    (*DriverObject).MajorFunction[IRP_MJ_SYSTEM_CONTROL as usize] =
        Some(KsDefaultForwardIrp);

    /* Use provided ks unload function */
    (*DriverObject).DriverUnload =
        Some(KsNullDriverUnload);

    /* Sysaudio needs to do work on pnp, so handle it */
    (*DriverObject).MajorFunction[IRP_MJ_PNP as usize] =
        Some(SysAudio_Pnp);
    (*DriverObject).MajorFunction[IRP_MJ_SHUTDOWN as usize] =
        Some(SysAudio_Shutdown);
    (*(*DriverObject).DriverExtension).AddDevice = Some(SysAudio_AddDevice);

    /* done */
    return STATUS_SUCCESS;
}
