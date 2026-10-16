// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
pub static RUSTCONV_WSTR_C554B73F8939: [u16; 5] = [0x005Cu16, 0x003F, 0x003F, 0x005C, 0x0000];

use crate::prelude::*;
use ntkrnl_sys::ddk::{DEVICE_INTERFACE_CHANGE_NOTIFICATION, GUID, _IO_STATUS_BLOCK, IoRegisterDeviceInterface, IoRegisterPlugPlayNotification, IoSetDeviceInterfaceState, _OBJECT_ATTRIBUTES, ObReferenceObjectByHandle, PDEVICE_OBJECT, PDRIVER_OBJECT, PFILE_OBJECT, PHANDLE, PUNICODE_STRING, RtlAppendUnicodeStringToString, RtlAppendUnicodeToString, RtlFreeUnicodeString, ZwClose, ZwCreateFile};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut GUID_DEVICE_INTERFACE_ARRIVAL: GUID = _GUID {
    Data1: 0xcb3a4004 as ::core::ffi::c_ulong as ::core::ffi::c_uint,
    Data2: 0x46f0 as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0xb0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x60 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x97 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x13 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3f as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut GUID_DEVICE_INTERFACE_REMOVAL: GUID = _GUID {
    Data1: 0xcb3a4005 as ::core::ffi::c_ulong as ::core::ffi::c_uint,
    Data2: 0x46f0 as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0xb0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x60 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x97 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x13 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3f as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut KS_CATEGORY_AUDIO: GUID = _GUID {
    Data1: 0x6994ad04 as ::core::ffi::c_long as ::core::ffi::c_uint,
    Data2: 0x93ef as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0xa3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xcc as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x22 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x31 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x96 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut KS_CATEGORY_TOPOLOGY: GUID = _GUID {
    Data1: 0xdda54a40 as ::core::ffi::c_uint,
    Data2: 0x1e4c as ::core::ffi::c_ushort,
    Data3: 0x11d1 as ::core::ffi::c_ushort,
    Data4: [
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x50 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x40 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x57 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut DMOCATEGORY_ACOUSTIC_ECHO_CANCEL: GUID = _GUID {
    Data1: 0xbf963d80 as ::core::ffi::c_ulong as ::core::ffi::c_uint,
    Data2: 0xc559 as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0x8a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x25 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};
#[no_mangle]
pub unsafe extern "C" fn OpenDevice(
    DeviceName: PUNICODE_STRING,
    HandleOut: PHANDLE,
    FileObjectOut: *mut PFILE_OBJECT,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut NodeHandle: HANDLE = core::ptr::null_mut();
    let mut FileObject: PFILE_OBJECT = core::ptr::null_mut();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();

    // InitializeObjectAttributes(&ObjectAttributes, DeviceName, OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE, NULL, NULL)
    (*(&raw mut ObjectAttributes)).Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    (*(&raw mut ObjectAttributes)).RootDirectory = core::ptr::null_mut();
    (*(&raw mut ObjectAttributes)).Attributes = (OBJ_KERNEL_HANDLE | OBJ_CASE_INSENSITIVE) as ULONG;
    (*(&raw mut ObjectAttributes)).ObjectName = DeviceName;
    (*(&raw mut ObjectAttributes)).SecurityDescriptor = core::ptr::null_mut();
    (*(&raw mut ObjectAttributes)).SecurityQualityOfService = core::ptr::null_mut();

    Status = ZwCreateFile(
        &raw mut NodeHandle,
        (GENERIC_READ | GENERIC_WRITE | SYNCHRONIZE) as ACCESS_MASK,
        &raw mut ObjectAttributes,
        &raw mut IoStatusBlock,
        core::ptr::null_mut(),
        0 as ULONG,
        0 as ULONG,
        FILE_OPEN as ULONG,
        FILE_SYNCHRONOUS_IO_NONALERT as ULONG,
        core::ptr::null_mut(),
        0 as ULONG,
    );

    if !NT_SUCCESS!(Status) {
        DPRINT!("ZwCreateFile failed with %x %S\n", Status, (*DeviceName).Buffer);
        return Status;
    }

    Status = ObReferenceObjectByHandle(
        NodeHandle,
        (GENERIC_READ | GENERIC_WRITE) as ACCESS_MASK,
        *IoFileObjectType,
        0i32 as KPROCESSOR_MODE,
        &raw mut FileObject as *mut PVOID,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS!(Status) {
        ZwClose(NodeHandle);
        DPRINT!("ObReferenceObjectByHandle failed with %x\n", Status);
        return Status;
    }

    *HandleOut = NodeHandle;
    *FileObjectOut = FileObject;
    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn InsertAudioDevice(
    DeviceObject: PDEVICE_OBJECT,
    DeviceName: PUNICODE_STRING,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut DeviceExtension: PSYSAUDIODEVEXT = core::ptr::null_mut();
    let mut DeviceEntry: PKSAUDIO_DEVICE_ENTRY = core::ptr::null_mut();

    /* a new device has arrived */
    DeviceEntry = AllocateItem(
        NonPagedPool,
        core::mem::size_of::<KSAUDIO_DEVICE_ENTRY>() as SIZE_T,
    ) as PKSAUDIO_DEVICE_ENTRY;
    if DeviceEntry.is_null() {
        /* no memory */
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    'cleanup: {
        /* initialize audio device entry */
        RtlZeroMemory!(
            DeviceEntry as PVOID,
            core::mem::size_of::<KSAUDIO_DEVICE_ENTRY>() as SIZE_T,
        );

        /* set device name */
        (*DeviceEntry).DeviceName.Length = 0 as USHORT;
        (*DeviceEntry).DeviceName.MaximumLength = ((*DeviceName).MaximumLength as usize)
            .wrapping_add(
                (10 as usize).wrapping_mul(core::mem::size_of::<WCHAR>()),
            ) as USHORT;

        (*DeviceEntry).DeviceName.Buffer = AllocateItem(
            NonPagedPool,
            (*DeviceEntry).DeviceName.MaximumLength as SIZE_T,
        ) as PWCH;

        if (*DeviceEntry).DeviceName.Buffer.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'cleanup;
        }

        /* open device */
        Status = OpenDevice(
            DeviceName,
            core::ptr::addr_of_mut!((*DeviceEntry).Handle),
            core::ptr::addr_of_mut!((*DeviceEntry).FileObject),
        );
        if NT_SUCCESS!(Status) {
            /* copy device name */
            RtlAppendUnicodeStringToString(
                core::ptr::addr_of_mut!((*DeviceEntry).DeviceName),
                DeviceName,
            );
        } else {
            /* the device name needs to be prefixed */
            RtlAppendUnicodeToString(
                core::ptr::addr_of_mut!((*DeviceEntry).DeviceName),
                RUSTCONV_WSTR_C554B73F8939.as_ptr(),
            );
            RtlAppendUnicodeStringToString(
                core::ptr::addr_of_mut!((*DeviceEntry).DeviceName),
                DeviceName,
            );

            /* open device */
            Status = OpenDevice(
                core::ptr::addr_of_mut!((*DeviceEntry).DeviceName),
                core::ptr::addr_of_mut!((*DeviceEntry).Handle),
                core::ptr::addr_of_mut!((*DeviceEntry).FileObject),
            );
        }

        if !NT_SUCCESS!(Status) {
            break 'cleanup;
        }

        /* fetch device extension */
        DeviceExtension = (*DeviceObject).DeviceExtension as PSYSAUDIODEVEXT;
        /* insert new audio device */
        ExInterlockedInsertTailList(
            core::ptr::addr_of_mut!((*DeviceExtension).KsAudioDeviceList),
            core::ptr::addr_of_mut!((*DeviceEntry).Entry),
            core::ptr::addr_of_mut!((*DeviceExtension).Lock),
        );
        InterlockedIncrement(
            core::ptr::addr_of_mut!((*DeviceExtension).NumberOfKsAudioDevices) as PLONG,
        );

        DPRINT!(
            "Successfully opened audio device %u Device %S\n",
            (*DeviceExtension).NumberOfKsAudioDevices,
            (*DeviceEntry).DeviceName.Buffer
        );
        return Status;
    }

    /* cleanup: */
    if !DeviceEntry.is_null() {
        if !(*DeviceEntry).DeviceName.Buffer.is_null() {
            FreeItem((*DeviceEntry).DeviceName.Buffer as PVOID);
        }

        FreeItem(DeviceEntry as PVOID);
    }

    return Status;
}
#[no_mangle]
pub unsafe extern "system" fn DeviceInterfaceChangeCallback(
    NotificationStructure: PVOID,
    Context: PVOID,
) -> NTSTATUS {
    let mut Event: *mut DEVICE_INTERFACE_CHANGE_NOTIFICATION = core::ptr::null_mut();
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let DeviceObject: PDEVICE_OBJECT = Context as PDEVICE_OBJECT;

    Event = NotificationStructure as *mut DEVICE_INTERFACE_CHANGE_NOTIFICATION;

    if IsEqualGUIDAligned(
        core::ptr::addr_of_mut!((*Event).Event),
        core::ptr::addr_of!(GUID_DEVICE_INTERFACE_ARRIVAL),
    ) != false
    {
        Status = InsertAudioDevice(DeviceObject, (*Event).SymbolicLinkName);
        return Status;
    } else {
        DPRINT!("Remove interface to audio device!\n");
        UNIMPLEMENTED!();
        return STATUS_SUCCESS;
    }
}
#[no_mangle]
pub unsafe extern "C" fn SysAudioRegisterNotifications(
    DriverObject: PDRIVER_OBJECT,
    DeviceObject: PDEVICE_OBJECT,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut DeviceExtension: PSYSAUDIODEVEXT;

    DeviceExtension = (*DeviceObject).DeviceExtension as PSYSAUDIODEVEXT;

    Status = IoRegisterPlugPlayNotification(
        EventCategoryDeviceInterfaceChange,
        PNPNOTIFY_DEVICE_INTERFACE_INCLUDE_EXISTING_INTERFACES as ULONG,
        &raw const KS_CATEGORY_AUDIO as PVOID,
        DriverObject,
        Some(DeviceInterfaceChangeCallback),
        DeviceObject as PVOID,
        &raw mut (*DeviceExtension).KsAudioNotificationEntry,
    );

    if !NT_SUCCESS!(Status) {
        DPRINT!("IoRegisterPlugPlayNotification failed with %x\n", Status);
    }

    Status = IoRegisterPlugPlayNotification(
        EventCategoryDeviceInterfaceChange,
        PNPNOTIFY_DEVICE_INTERFACE_INCLUDE_EXISTING_INTERFACES as ULONG,
        &raw const DMOCATEGORY_ACOUSTIC_ECHO_CANCEL as PVOID,
        DriverObject,
        Some(DeviceInterfaceChangeCallback),
        DeviceObject as PVOID,
        &raw mut (*DeviceExtension).EchoCancelNotificationEntry,
    );

    if !NT_SUCCESS!(Status) {
        /* ignore failure for now */
        DPRINT!("IoRegisterPlugPlayNotification failed for DMOCATEGORY_ACOUSTIC_ECHO_CANCEL\n", Status);
    }

    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn SysAudioRegisterDeviceInterfaces(
    DeviceObject: PDEVICE_OBJECT,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut SymbolicLink: _UNICODE_STRING = core::mem::zeroed();

    Status = IoRegisterDeviceInterface(DeviceObject, &raw const KSCATEGORY_PREFERRED_MIDIOUT_DEVICE, core::ptr::null_mut(), &raw mut SymbolicLink);
    if NT_SUCCESS!(Status) {
        IoSetDeviceInterfaceState(&raw mut SymbolicLink, TRUE as BOOLEAN);
        RtlFreeUnicodeString(&raw mut SymbolicLink);
    } else {
        DPRINT!("Failed to register KSCATEGORY_PREFERRED_MIDIOUT_DEVICE interface Status %x\n", Status);
        return Status;
    }

    Status = IoRegisterDeviceInterface(DeviceObject, &raw const KSCATEGORY_PREFERRED_WAVEIN_DEVICE, core::ptr::null_mut(), &raw mut SymbolicLink);
    if NT_SUCCESS!(Status) {
        IoSetDeviceInterfaceState(&raw mut SymbolicLink, TRUE as BOOLEAN);
        RtlFreeUnicodeString(&raw mut SymbolicLink);
    } else {
        DPRINT!("Failed to register KSCATEGORY_PREFERRED_WAVEIN_DEVICE interface Status %x\n", Status);
        return Status;
    }

    Status = IoRegisterDeviceInterface(DeviceObject, &raw const KSCATEGORY_PREFERRED_WAVEOUT_DEVICE, core::ptr::null_mut(), &raw mut SymbolicLink);
    if NT_SUCCESS!(Status) {
        IoSetDeviceInterfaceState(&raw mut SymbolicLink, TRUE as BOOLEAN);
        RtlFreeUnicodeString(&raw mut SymbolicLink);
    } else {
        DPRINT!("Failed to register KSCATEGORY_PREFERRED_WAVEOUT_DEVICE interface Status %x\n", Status);
    }

    Status = IoRegisterDeviceInterface(DeviceObject, &raw const KSCATEGORY_SYSAUDIO, core::ptr::null_mut(), &raw mut SymbolicLink);
    if NT_SUCCESS!(Status) {
        IoSetDeviceInterfaceState(&raw mut SymbolicLink, TRUE as BOOLEAN);
        RtlFreeUnicodeString(&raw mut SymbolicLink);
    } else {
        DPRINT!("Failed to register KSCATEGORY_SYSAUDIO interface Status %x\n", Status);
    }

    return Status;
}
