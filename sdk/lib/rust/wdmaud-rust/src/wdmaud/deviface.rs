// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
pub static RUSTCONV_WSTR_951662ABB9FD: [u16; 24] = [
    0x005Cu16, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x005C, 0x0073, 0x0079,
    0x0073, 0x0061, 0x0075, 0x0064, 0x0069, 0x006F, 0x005C, 0x0047, 0x004C, 0x004F,
    0x0042, 0x0041, 0x004C, 0x0000,
];

use crate::prelude::*;
use ntkrnl_sys::ddk::{DEVICE_INTERFACE_CHANGE_NOTIFICATION, _IO_STATUS_BLOCK, IoCreateFile, IoGetDeviceInterfaces, IoRegisterDeviceInterface, IoRegisterPlugPlayNotification, IoSetDeviceInterfaceState, LPWSTR, _OBJECT_ATTRIBUTES, ObReferenceObjectByHandle, PDEVICE_OBJECT, PFILE_OBJECT, PHANDLE, RtlAppendUnicodeStringToString, RtlFreeUnicodeString, RtlInitUnicodeString, ZwClose};
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn WdmAudOpenSysAudioDevice(
    mut DeviceName: LPWSTR,
    mut Handle: PHANDLE,
) -> NTSTATUS {
    let mut SymbolicLink: _UNICODE_STRING = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Status: NTSTATUS = 0;

    RtlInitUnicodeString(core::ptr::addr_of_mut!(SymbolicLink), DeviceName as *const u16);

    // InitializeObjectAttributes(&ObjectAttributes, &SymbolicLink, OBJ_KERNEL_HANDLE | OBJ_OPENIF, NULL, NULL)
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.ObjectName = core::ptr::addr_of_mut!(SymbolicLink);
    ObjectAttributes.Attributes = (OBJ_OPENIF | OBJ_KERNEL_HANDLE) as ULONG;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = IoCreateFile(
        Handle,
        (SYNCHRONIZE | GENERIC_READ | GENERIC_WRITE) as ACCESS_MASK,
        core::ptr::addr_of_mut!(ObjectAttributes),
        core::ptr::addr_of_mut!(IoStatusBlock),
        core::ptr::null_mut(),
        0 as ULONG,
        0 as ULONG,
        FILE_OPEN as ULONG,
        FILE_SYNCHRONOUS_IO_NONALERT as ULONG,
        core::ptr::null_mut(),
        0 as ULONG,
        CreateFileTypeNone,
        core::ptr::null_mut(),
        (IO_NO_PARAMETER_CHECKING | IO_FORCE_ACCESS_CHECK) as ULONG,
    );

    return Status;
}
#[no_mangle]
pub unsafe extern "system" fn DeviceInterfaceChangeCallback(
    NotificationStructure: PVOID,
    Context: PVOID,
) -> NTSTATUS {
    let Event: *mut DEVICE_INTERFACE_CHANGE_NOTIFICATION =
        NotificationStructure as *mut DEVICE_INTERFACE_CHANGE_NOTIFICATION;

    DPRINT1!("DeviceInterfaceChangeCallback called %p\n", Event);
    DbgBreakPoint();
    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn WdmAudOpenSysAudioDeviceInterfaces(
    mut DeviceExtension: PWDMAUD_DEVICE_EXTENSION,
    mut SymbolicLinkList: LPWSTR,
) -> NTSTATUS {
    let mut Entry: *mut SYSAUDIO_ENTRY;
    let mut Length: ULONG;

    DPRINT1!("WdmAudOpenSysAudioDeviceInterfaces called\n");

    while *SymbolicLinkList != 0 {
        Length = (wcslen(SymbolicLinkList as *const WCHAR) as ULONG) + 1;
        Entry = AllocateItem(
            NonPagedPool,
            (core::mem::size_of::<SYSAUDIO_ENTRY>() + (Length as usize) * core::mem::size_of::<WCHAR>()) as SIZE_T,
        ) as *mut SYSAUDIO_ENTRY;
        if Entry.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        (*Entry).SymbolicLink.Length = ((Length as usize) * core::mem::size_of::<WCHAR>()) as USHORT;
        (*Entry).SymbolicLink.MaximumLength = ((Length as usize) * core::mem::size_of::<WCHAR>()) as USHORT;
        (*Entry).SymbolicLink.Buffer = Entry.add(1 as usize) as LPWSTR;
        wcscpy((*Entry).SymbolicLink.Buffer, SymbolicLinkList);

        InsertTailList(
            core::ptr::addr_of_mut!((*DeviceExtension).SysAudioDeviceList),
            core::ptr::addr_of_mut!((*Entry).Entry),
        );

        (*DeviceExtension).NumSysAudioDevices = (*DeviceExtension).NumSysAudioDevices.wrapping_add(1);
        SymbolicLinkList = SymbolicLinkList.add(Length as usize);
    }
    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn WdmAudOpenSysAudioDevices(mut DeviceObject: PDEVICE_OBJECT, mut DeviceExtension: PWDMAUD_DEVICE_EXTENSION) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut SymbolicLinkList: LPWSTR = core::ptr::null_mut();
    let mut Entry: *mut SYSAUDIO_ENTRY = core::ptr::null_mut();
    let mut Length: ULONG = 0;
    let mut hSysAudio: HANDLE = core::ptr::null_mut();
    let mut FileObject: PFILE_OBJECT = core::ptr::null_mut();
    let mut DeviceName: _UNICODE_STRING = RTL_CONSTANT_STRING!(RUSTCONV_WSTR_951662ABB9FD.as_ptr());

    if (*DeviceExtension).DeviceInterfaceSupport != 0 {
        Status = IoGetDeviceInterfaces(
            core::ptr::addr_of!(KSCATEGORY_SYSAUDIO),
            core::ptr::null_mut(),
            0,
            core::ptr::addr_of_mut!(SymbolicLinkList),
        );

        if NT_SUCCESS!(Status) {
            WdmAudOpenSysAudioDeviceInterfaces(DeviceExtension, SymbolicLinkList);
            FreeItem(SymbolicLinkList as PVOID);
        }

        Status = IoRegisterPlugPlayNotification(
            EventCategoryDeviceInterfaceChange,
            PNPNOTIFY_DEVICE_INTERFACE_INCLUDE_EXISTING_INTERFACES,
            core::ptr::addr_of!(KSCATEGORY_SYSAUDIO) as PVOID,
            (*DeviceObject).DriverObject,
            Some(DeviceInterfaceChangeCallback),
            DeviceExtension as PVOID,
            core::ptr::addr_of_mut!((*DeviceExtension).SysAudioNotification),
        );
    } else {
        Entry = AllocateItem(
            NonPagedPool,
            core::mem::size_of::<SYSAUDIO_ENTRY>() as SIZE_T,
        ) as *mut SYSAUDIO_ENTRY;
        if Entry.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        Length = (wcslen(DeviceName.Buffer as *const WCHAR) as ULONG)
            + 1;
        (*Entry).SymbolicLink.Length = 0;
        (*Entry).SymbolicLink.MaximumLength =
            (Length as usize * core::mem::size_of::<WCHAR>()) as USHORT;
        (*Entry).SymbolicLink.Buffer = AllocateItem(
            NonPagedPool,
            (*Entry).SymbolicLink.MaximumLength as SIZE_T,
        ) as PWCH;

        if (*Entry).SymbolicLink.Buffer.is_null() {
            FreeItem(Entry as PVOID);
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        Status = RtlAppendUnicodeStringToString(
            core::ptr::addr_of_mut!((*Entry).SymbolicLink),
            core::ptr::addr_of_mut!(DeviceName),
        );

        if !NT_SUCCESS!(Status) {
            FreeItem((*Entry).SymbolicLink.Buffer as PVOID);
            FreeItem(Entry as PVOID);
            return Status;
        }

        DPRINT!("Opening device %S\n", (*Entry).SymbolicLink.Buffer);
        Status = WdmAudOpenSysAudioDevice(
            (*Entry).SymbolicLink.Buffer as LPWSTR,
            core::ptr::addr_of_mut!(hSysAudio),
        );
        if !NT_SUCCESS!(Status) {
            DPRINT1!("Failed to open sysaudio %x\n", Status);
            FreeItem((*Entry).SymbolicLink.Buffer as PVOID);
            FreeItem(Entry as PVOID);
            return Status;
        }

        InsertTailList(
            core::ptr::addr_of_mut!((*DeviceExtension).SysAudioDeviceList),
            core::ptr::addr_of_mut!((*Entry).Entry),
        );
        (*DeviceExtension).NumSysAudioDevices =
            (*DeviceExtension).NumSysAudioDevices.wrapping_add(1);

        /* get the file object */
        Status = ObReferenceObjectByHandle(
            hSysAudio,
            (FILE_READ_DATA | FILE_WRITE_DATA) as ACCESS_MASK,
            *IoFileObjectType,
            0i32 as KPROCESSOR_MODE,
            core::ptr::addr_of_mut!(FileObject) as *mut PVOID,
            core::ptr::null_mut(),
        );
        if !NT_SUCCESS!(Status) {
            DPRINT1!("Failed to reference FileObject %x\n", Status);
            ZwClose(hSysAudio);
            return Status;
        }
        DeviceExtension = (*DeviceObject).DeviceExtension as PWDMAUD_DEVICE_EXTENSION;
        (*DeviceExtension).hSysAudio = hSysAudio;
        (*DeviceExtension).FileObject = FileObject;
    }

    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn WdmAudRegisterDeviceInterface(
    PhysicalDeviceObject: PDEVICE_OBJECT,
    DeviceExtension: PWDMAUD_DEVICE_EXTENSION,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut SymbolicLinkName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };

    Status = IoRegisterDeviceInterface(
        PhysicalDeviceObject,
        core::ptr::addr_of!(KSCATEGORY_WDMAUD),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(SymbolicLinkName),
    );
    if NT_SUCCESS!(Status) {
        IoSetDeviceInterfaceState(
            core::ptr::addr_of_mut!(SymbolicLinkName),
            TRUE as BOOLEAN,
        );
        RtlFreeUnicodeString(core::ptr::addr_of_mut!(SymbolicLinkName));
        //DeviceExtension->DeviceInterfaceSupport = TRUE;
        return Status;
    }

    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn WdmAudOpenSysaudio(
    DeviceObject: PDEVICE_OBJECT,
    pClient: *mut PWDMAUD_CLIENT,
) -> NTSTATUS {
    let mut Client: PWDMAUD_CLIENT = core::ptr::null_mut();
    let mut DeviceExtension: PWDMAUD_DEVICE_EXTENSION = core::ptr::null_mut();

    /* get device extension */
    DeviceExtension = (*DeviceObject).DeviceExtension as PWDMAUD_DEVICE_EXTENSION;

    if (*DeviceExtension).NumSysAudioDevices == 0 {
        /* wdmaud failed to open sysaudio */
        return STATUS_UNSUCCESSFUL;
    }

    /* sanity check */
    ASSERT!(!(IsListEmpty(&raw mut (*DeviceExtension).SysAudioDeviceList) != 0));

    /* allocate client context struct */
    Client = AllocateItem(
        NonPagedPool,
        core::mem::size_of::<WDMAUD_CLIENT>() as SIZE_T,
    ) as PWDMAUD_CLIENT;

    /* check for allocation failure */
    if Client.is_null() {
        /* not enough memory */
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    /* zero client context struct */
    RtlZeroMemory!(
        Client as PVOID,
        core::mem::size_of::<WDMAUD_CLIENT>() as SIZE_T,
    );

    /* initialize mixer event list */
    InitializeListHead(&raw mut (*Client).MixerEventList);

    /* store result */
    *pClient = Client;

    /* insert client into list */
    ExInterlockedInsertTailList(
        &raw mut (*DeviceExtension).WdmAudClientList,
        &raw mut (*Client).Entry,
        &raw mut (*DeviceExtension).Lock,
    );

    /* done */
    return STATUS_SUCCESS;
}
