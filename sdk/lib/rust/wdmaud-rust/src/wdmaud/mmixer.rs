use crate::wdmaud::control::KSPROPSETID_Sysaudio;
use crate::prelude::*;
use ntkrnl_sys::ddk::{ACCESS_MASK, GUID, IoOpenDeviceInterfaceRegistryKey, KEVENT, KeInitializeEvent, LPWSTR, _EVENT_TYPE_NotificationEvent, _OBJECT_ATTRIBUTES, ObReferenceObjectByHandle, PDEVICE_OBJECT, PFILE_OBJECT, PHANDLE, PIRP, PKEVENT, PKEY_VALUE_PARTIAL_INFORMATION, RtlInitUnicodeString, ZwClose, ZwOpenKey, ZwQueryValueKey};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut MixerContext: MIXER_CONTEXT = unsafe {
    MIXER_CONTEXT {
        SizeOfStruct: ::core::mem::size_of::<MIXER_CONTEXT>() as ULONG,
        MixerContext: NULL,
        Alloc: Some(Alloc as unsafe extern "C" fn(ULONG) -> PVOID),
        Control: Some(
            Control
                as unsafe extern "C" fn(
                    HANDLE,
                    ULONG,
                    PVOID,
                    ULONG,
                    PVOID,
                    ULONG,
                    PULONG,
                ) -> MIXER_STATUS,
        ),
        Free: Some(Free as unsafe extern "C" fn(PVOID) -> ()),
        Open: Some(Open as unsafe extern "C" fn(LPWSTR, PHANDLE) -> MIXER_STATUS),
        Close: Some(Close as unsafe extern "C" fn(HANDLE) -> MIXER_STATUS),
        Copy: Some(Copy as unsafe extern "C" fn(PVOID, PVOID, ULONG) -> ()),
        OpenKey: Some(
            OpenKey as unsafe extern "C" fn(HANDLE, LPWSTR, ULONG, PHANDLE) -> MIXER_STATUS,
        ),
        QueryKeyValue: Some(
            QueryKeyValue
                as unsafe extern "C" fn(HANDLE, LPWSTR, *mut PVOID, PULONG, PULONG) -> MIXER_STATUS,
        ),
        CloseKey: Some(CloseKey as unsafe extern "C" fn(HANDLE) -> MIXER_STATUS),
        AllocEventData: Some(AllocEventData as unsafe extern "C" fn(ULONG) -> PVOID),
        FreeEventData: Some(FreeEventData as unsafe extern "C" fn(PVOID) -> ()),
    }
};

#[no_mangle]
pub static mut CategoryGuid: GUID = _GUID {
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
pub unsafe extern "C" fn QueryKeyValue(
    hKey: HANDLE,
    lpKeyName: LPWSTR,
    ResultBuffer: *mut PVOID,
    ResultLength: PULONG,
    KeyType: PULONG,
) -> MIXER_STATUS {
    let mut Status: NTSTATUS;
    let mut KeyName: _UNICODE_STRING = core::mem::zeroed();
    let mut Length: ULONG = core::mem::zeroed();
    let mut PartialInformation: PKEY_VALUE_PARTIAL_INFORMATION;

    /* initialize key name */
    RtlInitUnicodeString(core::ptr::addr_of_mut!(KeyName), lpKeyName);

    /* now query MatchingDeviceId key */
    Status = ZwQueryValueKey(
        hKey,
        core::ptr::addr_of_mut!(KeyName),
        KeyValuePartialInformation,
        core::ptr::null_mut(),
        0,
        core::ptr::addr_of_mut!(Length),
    );

    /* check for success */
    if Status != STATUS_BUFFER_TOO_SMALL {
        return MM_STATUS_UNSUCCESSFUL;
    }

    /* allocate a buffer for key data */
    PartialInformation = AllocateItem(NonPagedPool, (Length) as SIZE_T) as PKEY_VALUE_PARTIAL_INFORMATION;

    if PartialInformation.is_null() {
        return MM_STATUS_NO_MEMORY;
    }

    /* now query MatchingDeviceId key */
    Status = ZwQueryValueKey(
        hKey,
        core::ptr::addr_of_mut!(KeyName),
        KeyValuePartialInformation,
        PartialInformation as PVOID,
        Length,
        core::ptr::addr_of_mut!(Length),
    );

    /* check for success */
    if !NT_SUCCESS!(Status) {
        FreeItem(PartialInformation as PVOID);
        return MM_STATUS_UNSUCCESSFUL;
    }

    if !KeyType.is_null() {
        /* return key type */
        *KeyType = (*PartialInformation).Type as ULONG;
    }

    if !ResultLength.is_null() {
        /* return data length */
        *ResultLength = (*PartialInformation).DataLength as ULONG;
    }

    *ResultBuffer = AllocateItem(NonPagedPool, (*PartialInformation).DataLength as SIZE_T);
    if (*ResultBuffer).is_null() {
        /* not enough memory */
        FreeItem(PartialInformation as PVOID);
        return MM_STATUS_NO_MEMORY;
    }

    /* copy key value */
    RtlMoveMemory!(
        *ResultBuffer,
        core::ptr::addr_of_mut!((*PartialInformation).Data) as *const core::ffi::c_void,
        (*PartialInformation).DataLength as usize,
    );

    /* free key info */
    FreeItem(PartialInformation as PVOID);

    return MM_STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn OpenKey(
    hKey: HANDLE,
    lpSubKeyName: LPWSTR,
    DesiredAccess: ULONG,
    OutKey: PHANDLE,
) -> MIXER_STATUS {
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut SubKeyName: _UNICODE_STRING = core::mem::zeroed();
    let mut Status: NTSTATUS = 0;

    /* initialize sub key name */
    RtlInitUnicodeString(&raw mut SubKeyName, lpSubKeyName as PCWSTR);

    /* initialize key attributes */
    core::ptr::write_bytes(&raw mut ObjectAttributes, 0u8, 1);
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = hKey;
    ObjectAttributes.ObjectName = &raw mut SubKeyName as PUNICODE_STRING;
    ObjectAttributes.Attributes = (OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE | OBJ_OPENIF) as ULONG;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut() as PVOID;
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut() as PVOID;

    /* open the key */
    Status = ZwOpenKey(OutKey, DesiredAccess as ACCESS_MASK, &raw mut ObjectAttributes);

    if NT_SUCCESS!(Status) {
        return MM_STATUS_SUCCESS;
    } else {
        return MM_STATUS_UNSUCCESSFUL;
    }
}
#[no_mangle]
pub unsafe extern "C" fn CloseKey(hKey: HANDLE) -> MIXER_STATUS {
    if ZwClose(hKey) == STATUS_SUCCESS {
        return MM_STATUS_SUCCESS;
    } else {
        return MM_STATUS_UNSUCCESSFUL;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Alloc(NumBytes: ULONG) -> PVOID {
    return AllocateItem(NonPagedPool, (NumBytes) as SIZE_T);
}
#[no_mangle]
pub unsafe extern "C" fn Close(hDevice: HANDLE) -> MIXER_STATUS {
    if ZwClose(hDevice) == STATUS_SUCCESS {
        return MM_STATUS_SUCCESS;
    } else {
        return MM_STATUS_UNSUCCESSFUL;
    }
}

#[no_mangle]
pub unsafe extern "C" fn Free(Block: PVOID) {
    FreeItem(Block);
}

#[no_mangle]
pub unsafe extern "C" fn Copy(Src: PVOID, Dst: PVOID, NumBytes: ULONG) {
    RtlMoveMemory!(Src, Dst, NumBytes);
}
#[no_mangle]
pub unsafe extern "C" fn Open(DevicePath: LPWSTR, hDevice: PHANDLE) -> MIXER_STATUS {
    if WdmAudOpenSysAudioDevice(DevicePath, hDevice) == STATUS_SUCCESS {
        return MM_STATUS_SUCCESS;
    } else {
        return MM_STATUS_UNSUCCESSFUL;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Control(
    mut hMixer: HANDLE,
    mut dwIoControlCode: ULONG,
    mut lpInBuffer: PVOID,
    mut nInBufferSize: ULONG,
    mut lpOutBuffer: PVOID,
    mut nOutBufferSize: ULONG,
    mut lpBytesReturned: PULONG,
) -> MIXER_STATUS {
    let mut Status: NTSTATUS = 0;
    let mut FileObject: PFILE_OBJECT = core::ptr::null_mut();

    /* get file object */
    Status = ObReferenceObjectByHandle(
        hMixer,
        (GENERIC_READ | GENERIC_WRITE) as ACCESS_MASK,
        *IoFileObjectType,
        0i32 as KPROCESSOR_MODE,
        core::ptr::addr_of_mut!(FileObject) as *mut PVOID,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS!(Status) {
        DPRINT!("failed to reference %p with %lx\n", hMixer, Status);
        return MM_STATUS_UNSUCCESSFUL;
    }

    /* perform request */
    Status = KsSynchronousIoControlDevice(
        FileObject,
        0i32 as KPROCESSOR_MODE,
        dwIoControlCode,
        lpInBuffer,
        nInBufferSize,
        lpOutBuffer,
        nOutBufferSize,
        lpBytesReturned,
    );

    /* release object reference */
    ObDereferenceObject(FileObject as PVOID);

    if Status == STATUS_MORE_ENTRIES
        || Status == STATUS_BUFFER_OVERFLOW
        || Status == STATUS_BUFFER_TOO_SMALL
    {
        /* more data is available */
        return MM_STATUS_MORE_ENTRIES;
    } else if Status == STATUS_SUCCESS {
        /* operation succeeded */
        return MM_STATUS_SUCCESS;
    } else {
        DPRINT!("Failed with %lx\n", Status);
        return MM_STATUS_UNSUCCESSFUL;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Enum(
    EnumContext: PVOID,
    DeviceIndex: ULONG,
    DeviceName: *mut LPWSTR,
    OutHandle: PHANDLE,
    OutKey: PHANDLE,
) -> MIXER_STATUS {
    let mut DeviceObject: PDEVICE_OBJECT;
    let mut DeviceCount: ULONG;
    let mut Status: NTSTATUS;
    let mut KeyName: _UNICODE_STRING = core::mem::zeroed();

    /* get enumeration context */
    DeviceObject = EnumContext as PDEVICE_OBJECT;

    /* get device count */
    DeviceCount = GetSysAudioDeviceCount(DeviceObject);

    if DeviceIndex >= DeviceCount {
        /* no more devices */
        return MM_STATUS_NO_MORE_DEVICES;
    }

    /* get device name */
    Status = GetSysAudioDevicePnpName(DeviceObject, DeviceIndex, DeviceName);

    if !NT_SUCCESS!(Status) {
        /* failed to retrieve device name */
        return MM_STATUS_UNSUCCESSFUL;
    }

    /* initialize key name */
    RtlInitUnicodeString(&raw mut KeyName, *DeviceName as PCWSTR);

    /* open device interface key */
    Status = IoOpenDeviceInterfaceRegistryKey(
        &raw mut KeyName,
        (GENERIC_READ | GENERIC_WRITE) as ACCESS_MASK,
        OutKey,
    );

    if !NT_SUCCESS!(Status) {
        *OutKey = core::ptr::null_mut();
    }

    /* open device handle */
    Status = OpenDevice(*DeviceName, OutHandle, core::ptr::null_mut());
    if !NT_SUCCESS!(Status) {
        /* failed to open device */
        return MM_STATUS_UNSUCCESSFUL;
    }

    return MM_STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn AllocEventData(ExtraSize: ULONG) -> PVOID {
    let Data: PKSEVENTDATA = AllocateItem(
        NonPagedPool,
        (core::mem::size_of::<KSEVENTDATA>() as SIZE_T).wrapping_add(ExtraSize as SIZE_T),
    ) as PKSEVENTDATA;
    if Data.is_null() {
        return core::ptr::null_mut();
    }
    (*Data).__bindgen_anon_1.EventObject.Event = AllocateItem(
        NonPagedPool,
        core::mem::size_of::<KEVENT>() as SIZE_T,
    );
    if (*Data).__bindgen_anon_1.EventHandle.Event.is_null() {
        FreeItem(Data as PVOID);
        return core::ptr::null_mut();
    }
    KeInitializeEvent(
        (*Data).__bindgen_anon_1.EventObject.Event as PRKEVENT,
        _EVENT_TYPE_NotificationEvent,
        FALSE as BOOLEAN,
    );
    (*Data).NotificationType = KSEVENTF_EVENT_HANDLE as ULONG;
    return Data as PVOID;
}
#[no_mangle]
pub unsafe extern "C" fn FreeEventData(EventData: PVOID) {
    let Data: PKSEVENTDATA = EventData as PKSEVENTDATA;

    FreeItem((*Data).__bindgen_anon_1.EventHandle.Event as PVOID);
    FreeItem(Data as PVOID);
}
#[no_mangle]
pub unsafe extern "system" fn EventCallback(
    MixerEventContext: PVOID,
    hMixer: HANDLE,
    NotificationType: ULONG,
    Value: ULONG,
) {
    let mut ClientInfo: PWDMAUD_CLIENT = core::ptr::null_mut();
    let mut Entry: PEVENT_ENTRY = core::ptr::null_mut();
    let mut Index: ULONG;

    /* get client context */
    ClientInfo = MixerEventContext as PWDMAUD_CLIENT;

    /* now search for the mixer which originated the request */
    Index = 0;
    while Index < (*ClientInfo).NumPins {
        if (*(*ClientInfo).hPins.offset(Index as isize)).Handle == hMixer
            && (*(*ClientInfo).hPins.offset(Index as isize)).Type == MIXER_DEVICE_TYPE
        {
            if !(*(*ClientInfo).hPins.offset(Index as isize)).NotifyEvent.is_null() {
                /* allocate event entry */
                Entry = AllocateItem(
                    NonPagedPool,
                    core::mem::size_of::<EVENT_ENTRY>() as SIZE_T,
                ) as PEVENT_ENTRY;
                if Entry.is_null() {
                    /* no memory */
                    break;
                }

                /* setup event entry */
                (*Entry).NotificationType = NotificationType;
                (*Entry).Value = Value;
                (*Entry).hMixer = hMixer;

                /* insert entry */
                InsertTailList(
                    &raw mut (*ClientInfo).MixerEventList,
                    &raw mut (*Entry).Entry,
                );

                /* now notify the client */
                KeSetEvent(
                    (*(*ClientInfo).hPins.offset(Index as isize)).NotifyEvent,
                    0 as KPRIORITY,
                    FALSE as BOOLEAN,
                );
            }
            /* done */
            break;
        }
        Index = Index.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn WdmAudMixerInitialize(DeviceObject: PDEVICE_OBJECT) -> NTSTATUS {
    let mut Status: MIXER_STATUS;

    /* initialize the mixer library */
    Status = MMixerInitialize(
        core::ptr::addr_of_mut!(MixerContext),
        Some(
            Enum as unsafe extern "C" fn(
                PVOID,
                ULONG,
                *mut LPWSTR,
                PHANDLE,
                PHANDLE,
            ) -> MIXER_STATUS,
        ),
        DeviceObject as PVOID,
    );

    if Status != MM_STATUS_SUCCESS {
        /* failed to initialize mmixer library */
        DPRINT!("MMixerInitialize failed with %lx\n", Status);
    }

    return Status as NTSTATUS;
}
#[no_mangle]
pub unsafe extern "C" fn WdmAudMixerCapabilities(
    mut DeviceObject: PDEVICE_OBJECT,
    mut DeviceInfo: PWDMAUD_DEVICE_INFO,
    mut ClientInfo: PWDMAUD_CLIENT,
    mut DeviceExtension: PWDMAUD_DEVICE_EXTENSION,
) -> NTSTATUS {
    if MMixerGetCapabilities(
        core::ptr::addr_of_mut!(MixerContext),
        (*DeviceInfo).DeviceIndex as ULONG,
        core::ptr::addr_of_mut!((*DeviceInfo).u.MixCaps),
    ) == MM_STATUS_SUCCESS
    {
        return STATUS_SUCCESS;
    }

    return STATUS_INVALID_PARAMETER;
}
#[no_mangle]
pub unsafe extern "C" fn WdmAudControlOpenMixer(DeviceObject: PDEVICE_OBJECT, Irp: PIRP, DeviceInfo: PWDMAUD_DEVICE_INFO, ClientInfo: PWDMAUD_CLIENT) -> NTSTATUS {
    let mut hMixer: HANDLE = core::ptr::null_mut();
    let mut Handles: PWDMAUD_HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;
    let mut EventObject: PKEVENT = core::ptr::null_mut();

    DPRINT!("WdmAudControlOpenMixer\n");

    if !(*DeviceInfo).u.hNotifyEvent.is_null() {
        Status = ObReferenceObjectByHandle(
            (*DeviceInfo).u.hNotifyEvent,
            EVENT_MODIFY_STATE,
            *ExEventObjectType,
            (1i32) as KPROCESSOR_MODE,
            core::ptr::addr_of_mut!(EventObject) as *mut PVOID,
            core::ptr::null_mut(),
        );

        if !NT_SUCCESS!(Status) {
            DPRINT1!(
                "Invalid notify event passed %p from client %p\n",
                (*DeviceInfo).u.hNotifyEvent,
                ClientInfo
            );
            DbgBreakPoint();
            return SetIrpIoStatus(Irp, STATUS_UNSUCCESSFUL, 0);
        }
    }

    if MMixerOpen(
        core::ptr::addr_of_mut!(MixerContext),
        ((*DeviceInfo).DeviceIndex) as ULONG,
        ClientInfo as PVOID,
        Some(EventCallback),
        core::ptr::addr_of_mut!(hMixer),
    ) != MM_STATUS_SUCCESS
    {
        ObDereferenceObject(EventObject as PVOID);
        DPRINT1!("Failed to open mixer\n");
        return SetIrpIoStatus(Irp, STATUS_UNSUCCESSFUL, 0);
    }

    Handles = AllocateItem(
        NonPagedPool,
        (core::mem::size_of::<WDMAUD_HANDLE>() as SIZE_T)
            .wrapping_mul(((*ClientInfo).NumPins.wrapping_add(1)) as SIZE_T),
    ) as PWDMAUD_HANDLE;

    if !Handles.is_null() {
        if (*ClientInfo).NumPins != 0 {
            RtlMoveMemory!(
                Handles,
                (*ClientInfo).hPins,
                core::mem::size_of::<WDMAUD_HANDLE>().wrapping_mul((*ClientInfo).NumPins as usize),
            );
            FreeItem((*ClientInfo).hPins as PVOID);
        }

        (*ClientInfo).hPins = Handles;
        (*(*ClientInfo).hPins.offset((*ClientInfo).NumPins as isize)).Handle = hMixer;
        (*(*ClientInfo).hPins.offset((*ClientInfo).NumPins as isize)).Type = MIXER_DEVICE_TYPE;
        (*(*ClientInfo).hPins.offset((*ClientInfo).NumPins as isize)).NotifyEvent = EventObject;
        (*ClientInfo).NumPins = (*ClientInfo).NumPins.wrapping_add(1);
    } else {
        ObDereferenceObject(EventObject as PVOID);
        return SetIrpIoStatus(
            Irp,
            STATUS_UNSUCCESSFUL,
            core::mem::size_of::<WDMAUD_DEVICE_INFO>() as ULONG,
        );
    }

    (*DeviceInfo).hDevice = hMixer;

    return SetIrpIoStatus(
        Irp,
        STATUS_SUCCESS,
        core::mem::size_of::<WDMAUD_DEVICE_INFO>() as ULONG,
    );
}
#[no_mangle]
pub unsafe extern "C" fn WdmAudControlCloseMixer(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    DeviceInfo: PWDMAUD_DEVICE_INFO,
    ClientInfo: PWDMAUD_CLIENT,
    Index: ULONG,
) -> NTSTATUS {
    /* Remove event associated to this client */
    if MMixerClose(
        core::ptr::addr_of_mut!(MixerContext),
        (*DeviceInfo).DeviceIndex as ULONG,
        ClientInfo as PVOID,
        Some(EventCallback),
    ) != MM_STATUS_SUCCESS
    {
        DPRINT1!("Failed to close mixer\n");
        return SetIrpIoStatus(
            Irp,
            STATUS_UNSUCCESSFUL,
            core::mem::size_of::<WDMAUD_DEVICE_INFO>() as ULONG,
        );
    }

    /* Dereference event */
    if !(*(*ClientInfo).hPins.offset(Index as isize))
        .NotifyEvent
        .is_null()
    {
        ObDereferenceObject(
            (*(*ClientInfo).hPins.offset(Index as isize)).NotifyEvent as PVOID,
        );
        (*(*ClientInfo).hPins.offset(Index as isize)).NotifyEvent = core::ptr::null_mut();
    }

    /* FIXME: do we need to free ClientInfo->hPins ? */
    return SetIrpIoStatus(
        Irp,
        STATUS_SUCCESS,
        core::mem::size_of::<WDMAUD_DEVICE_INFO>() as ULONG,
    );
}
#[no_mangle]
pub unsafe extern "C" fn WdmAudCloseAllMixers(
    DeviceObject: PDEVICE_OBJECT,
    ClientInfo: PWDMAUD_CLIENT,
    Index: ULONG,
) {
    let mut DeviceCount: ULONG;
    let mut DeviceIndex: ULONG;

    /* Get all mixers */
    DeviceCount = GetSysAudioDeviceCount(DeviceObject);

    /* Close every mixer attached to the device */
    DeviceIndex = 0;
    while DeviceIndex < DeviceCount {
        if MMixerClose(
            core::ptr::addr_of_mut!(MixerContext),
            DeviceIndex,
            ClientInfo as PVOID,
            Some(EventCallback),
        ) != MM_STATUS_SUCCESS
        {
            DPRINT1!("Failed to close mixer for device %lu\n", DeviceIndex);
        }
        DeviceIndex = DeviceIndex.wrapping_add(1);
    }

    /* Dereference event */
    if !(*(*ClientInfo).hPins.offset(Index as isize))
        .NotifyEvent
        .is_null()
    {
        ObDereferenceObject((*(*ClientInfo).hPins.offset(Index as isize)).NotifyEvent as PVOID);
        let fresh18 = &mut (*(*ClientInfo).hPins.offset(Index as isize)).NotifyEvent;
        *fresh18 = core::ptr::null_mut();
    }
}

#[no_mangle]
pub unsafe extern "system" fn WdmAudGetControlDetails(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    DeviceInfo: PWDMAUD_DEVICE_INFO,
    ClientInfo: PWDMAUD_CLIENT,
) -> NTSTATUS {
    let mut Status: MIXER_STATUS;

    /* clear hmixer type flag */
    (*DeviceInfo).Flags &= !(MIXER_OBJECTF_HMIXER as ULONG);

    /* query mmixer library */
    Status = MMixerGetControlDetails(
        core::ptr::addr_of_mut!(MixerContext),
        (*DeviceInfo).hDevice,
        (*DeviceInfo).DeviceIndex as ULONG,
        (*DeviceInfo).Flags,
        core::ptr::addr_of_mut!((*DeviceInfo).u.MixDetails),
    );

    if Status == MM_STATUS_SUCCESS {
        return SetIrpIoStatus(
            Irp,
            STATUS_SUCCESS,
            core::mem::size_of::<WDMAUD_DEVICE_INFO>() as ULONG,
        );
    } else {
        return SetIrpIoStatus(
            Irp,
            STATUS_UNSUCCESSFUL,
            core::mem::size_of::<WDMAUD_DEVICE_INFO>() as ULONG,
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn WdmAudGetLineInfo(
        DeviceObject: PDEVICE_OBJECT,
        Irp: PIRP,
        DeviceInfo: PWDMAUD_DEVICE_INFO,
        ClientInfo: PWDMAUD_CLIENT) -> NTSTATUS {
    let mut Status: MIXER_STATUS;

    /* clear hmixer type flag */
    (*DeviceInfo).Flags &= !MIXER_OBJECTF_HMIXER as ULONG;

    /* query mixer library */
    Status = MMixerGetLineInfo(
        core::ptr::addr_of_mut!(MixerContext),
        (*DeviceInfo).hDevice,
        (*DeviceInfo).DeviceIndex as ULONG,
        (*DeviceInfo).Flags,
        core::ptr::addr_of_mut!((*DeviceInfo).u.MixLine),
    );

    if Status as ::core::ffi::c_uint == MM_STATUS_SUCCESS as ::core::ffi::c_uint {
        return SetIrpIoStatus(
            Irp,
            STATUS_SUCCESS,
            core::mem::size_of::<WDMAUD_DEVICE_INFO>() as ULONG,
        );
    } else {
        return SetIrpIoStatus(
            Irp,
            STATUS_UNSUCCESSFUL,
            core::mem::size_of::<WDMAUD_DEVICE_INFO>() as ULONG,
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn WdmAudGetLineControls(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    DeviceInfo: PWDMAUD_DEVICE_INFO,
    ClientInfo: PWDMAUD_CLIENT,
) -> NTSTATUS {
    let mut Status: MIXER_STATUS = MM_STATUS_SUCCESS;

    /* clear hmixer type flag */
    (*DeviceInfo).Flags &= !MIXER_OBJECTF_HMIXER as ULONG;

    /* query mixer library */
    Status = MMixerGetLineControls(
        core::ptr::addr_of_mut!(MixerContext),
        (*DeviceInfo).hDevice,
        (*DeviceInfo).DeviceIndex as ULONG,
        (*DeviceInfo).Flags,
        core::ptr::addr_of_mut!((*DeviceInfo).u.MixControls),
    );

    if Status == MM_STATUS_SUCCESS {
        return SetIrpIoStatus(
            Irp,
            STATUS_SUCCESS,
            core::mem::size_of::<WDMAUD_DEVICE_INFO>() as ULONG,
        );
    } else {
        return SetIrpIoStatus(
            Irp,
            STATUS_UNSUCCESSFUL,
            core::mem::size_of::<WDMAUD_DEVICE_INFO>() as ULONG,
        );
    }
}
#[no_mangle]
pub unsafe extern "system" fn WdmAudSetControlDetails(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    DeviceInfo: PWDMAUD_DEVICE_INFO,
    ClientInfo: PWDMAUD_CLIENT,
) -> NTSTATUS {
    let mut Status: MIXER_STATUS;

    /* clear hmixer type flag */
    (*DeviceInfo).Flags &= !(MIXER_OBJECTF_HMIXER as ULONG);

    /* query mixer library */
    Status = MMixerSetControlDetails(
        core::ptr::addr_of_mut!(MixerContext),
        (*DeviceInfo).hDevice,
        (*DeviceInfo).DeviceIndex as ULONG,
        (*DeviceInfo).Flags,
        core::ptr::addr_of_mut!((*DeviceInfo).u.MixDetails),
    );

    if Status == MM_STATUS_SUCCESS {
        return SetIrpIoStatus(
            Irp,
            STATUS_SUCCESS,
            core::mem::size_of::<WDMAUD_DEVICE_INFO>() as ULONG,
        );
    } else {
        return SetIrpIoStatus(
            Irp,
            STATUS_UNSUCCESSFUL,
            core::mem::size_of::<WDMAUD_DEVICE_INFO>() as ULONG,
        );
    }
}
#[no_mangle]
pub unsafe extern "system" fn WdmAudGetMixerEvent(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    DeviceInfo: PWDMAUD_DEVICE_INFO,
    ClientInfo: PWDMAUD_CLIENT,
) -> NTSTATUS {
    let mut Entry: PLIST_ENTRY;
    let mut EventEntry: PEVENT_ENTRY;

    /* enumerate event list and check if there is a new event */
    Entry = (*ClientInfo).MixerEventList.Flink;

    while Entry != core::ptr::addr_of_mut!((*ClientInfo).MixerEventList) {
        /* grab event entry */
        EventEntry = CONTAINING_RECORD!(Entry, EVENT_ENTRY, Entry) as PEVENT_ENTRY;

        if (*EventEntry).hMixer == (*DeviceInfo).hDevice {
            /* found an entry */
            (*DeviceInfo).u.MixerEvent.hMixer = (*EventEntry).hMixer;
            (*DeviceInfo).u.MixerEvent.NotificationType = (*EventEntry).NotificationType;
            (*DeviceInfo).u.MixerEvent.Value = (*EventEntry).Value;

            /* remove entry from list */
            RemoveEntryList(core::ptr::addr_of_mut!((*EventEntry).Entry));

            /* free event entry */
            FreeItem(EventEntry as PVOID);

            /* done */
            return SetIrpIoStatus(
                Irp,
                STATUS_SUCCESS,
                core::mem::size_of::<WDMAUD_DEVICE_INFO>() as ULONG,
            );
        }

        /* move to next */
        Entry = (*Entry).Flink;
    }

    /* no event entry available */
    return SetIrpIoStatus(
        Irp,
        STATUS_UNSUCCESSFUL,
        core::mem::size_of::<WDMAUD_DEVICE_INFO>() as ULONG,
    );
}
#[no_mangle]
pub unsafe extern "C" fn WdmAudGetMixerDeviceCount() -> ULONG {
    return MMixerGetCount(core::ptr::addr_of_mut!(MixerContext));
}
#[no_mangle]
pub unsafe extern "C" fn WdmAudGetWaveInDeviceCount() -> ULONG {
    return MMixerGetWaveInCount(core::ptr::addr_of_mut!(MixerContext));
}
#[no_mangle]
pub unsafe extern "C" fn WdmAudGetWaveOutDeviceCount() -> ULONG {
    return MMixerGetWaveOutCount(core::ptr::addr_of_mut!(MixerContext));
}
#[no_mangle]
pub unsafe extern "C" fn WdmAudGetMidiInDeviceCount() -> ULONG {
    return MMixerGetMidiInCount(core::ptr::addr_of_mut!(MixerContext));
}
#[no_mangle]
pub unsafe extern "C" fn WdmAudGetMidiOutDeviceCount() -> ULONG {
    return MMixerGetWaveOutCount(core::ptr::addr_of_mut!(MixerContext));
}
#[no_mangle]
pub unsafe extern "C" fn WdmAudGetPnpNameByIndexAndType(
    DeviceIndex: ULONG,
    DeviceType: SOUND_DEVICE_TYPE,
    DevicePath: *mut LPWSTR,
) -> NTSTATUS {
    if DeviceType == WAVE_IN_DEVICE_TYPE || DeviceType == WAVE_OUT_DEVICE_TYPE {
        if MMixerGetWaveDevicePath(
            core::ptr::addr_of_mut!(MixerContext),
            (DeviceType == WAVE_IN_DEVICE_TYPE) as ULONG,
            DeviceIndex,
            DevicePath,
        ) == MM_STATUS_SUCCESS {
            return STATUS_SUCCESS;
        } else {
            return STATUS_UNSUCCESSFUL;
        }
    } else if DeviceType == MIDI_IN_DEVICE_TYPE || DeviceType == MIDI_OUT_DEVICE_TYPE {
        if MMixerGetMidiDevicePath(
            core::ptr::addr_of_mut!(MixerContext),
            (DeviceType == MIDI_IN_DEVICE_TYPE) as ULONG,
            DeviceIndex,
            DevicePath,
        ) == MM_STATUS_SUCCESS {
            return STATUS_SUCCESS;
        } else {
            return STATUS_UNSUCCESSFUL;
        }
    } else if DeviceType == MIXER_DEVICE_TYPE {
        UNIMPLEMENTED!();
    }

    return STATUS_UNSUCCESSFUL;
}
#[no_mangle]
pub unsafe extern "C" fn WdmAudWaveCapabilities(
    DeviceObject: PDEVICE_OBJECT,
    DeviceInfo: PWDMAUD_DEVICE_INFO,
    ClientInfo: PWDMAUD_CLIENT,
    DeviceExtension: PWDMAUD_DEVICE_EXTENSION,
) -> NTSTATUS {
    let mut Status: MIXER_STATUS = MM_STATUS_UNSUCCESSFUL;

    if (*DeviceInfo).DeviceType == WAVE_IN_DEVICE_TYPE {
        /* get capabilities */
        Status = MMixerWaveInCapabilities(
            core::ptr::addr_of_mut!(MixerContext),
            ((*DeviceInfo).DeviceIndex) as ULONG,
            core::ptr::addr_of_mut!((*DeviceInfo).u.WaveInCaps),
        );
    } else if (*DeviceInfo).DeviceType == WAVE_OUT_DEVICE_TYPE {
        /* get capabilities */
        Status = MMixerWaveOutCapabilities(
            core::ptr::addr_of_mut!(MixerContext),
            ((*DeviceInfo).DeviceIndex) as ULONG,
            core::ptr::addr_of_mut!((*DeviceInfo).u.WaveOutCaps),
        );
    }

    if Status == MM_STATUS_SUCCESS {
        return STATUS_SUCCESS;
    } else {
        return Status as NTSTATUS;
    }
}
#[no_mangle]
pub unsafe extern "C" fn WdmAudMidiCapabilities(
    mut DeviceObject: PDEVICE_OBJECT,
    mut DeviceInfo: PWDMAUD_DEVICE_INFO,
    mut ClientInfo: PWDMAUD_CLIENT,
    mut DeviceExtension: PWDMAUD_DEVICE_EXTENSION,
) -> NTSTATUS {
    let mut Status: MIXER_STATUS = MM_STATUS_UNSUCCESSFUL;
    if (*DeviceInfo).DeviceType as ::core::ffi::c_uint
        == MIDI_IN_DEVICE_TYPE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        Status = MMixerMidiInCapabilities(
            &raw mut MixerContext,
            (*DeviceInfo).DeviceIndex as ULONG,
            &raw mut (*DeviceInfo).u.MidiInCaps,
        );
    } else if (*DeviceInfo).DeviceType as ::core::ffi::c_uint
        == WAVE_OUT_DEVICE_TYPE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        Status = MMixerMidiOutCapabilities(
            &raw mut MixerContext,
            (*DeviceInfo).DeviceIndex as ULONG,
            &raw mut (*DeviceInfo).u.MidiOutCaps,
        );
    }
    if Status as ::core::ffi::c_uint
        == MM_STATUS_SUCCESS as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return STATUS_SUCCESS;
    } else {
        return STATUS_UNSUCCESSFUL;
    };
}

#[no_mangle]
pub unsafe extern "system" fn WdmAudGetPosition(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    DeviceInfo: PWDMAUD_DEVICE_INFO,
) -> NTSTATUS {
    let mut Status: MIXER_STATUS = MM_STATUS_SUCCESS;
    let mut Position: ULONG = 0;

    /* Get position */
    Status = MMixerGetWavePosition(
        core::ptr::addr_of_mut!(MixerContext),
        (*DeviceInfo).hDevice,
        core::ptr::addr_of_mut!(Position),
    );

    if Status == MM_STATUS_SUCCESS {
        (*DeviceInfo).u.Position = Position as ULONGLONG;
        return SetIrpIoStatus(
            Irp,
            STATUS_SUCCESS,
            core::mem::size_of::<WDMAUD_DEVICE_INFO>() as ULONG,
        );
    } else {
        return SetIrpIoStatus(
            Irp,
            STATUS_UNSUCCESSFUL,
            core::mem::size_of::<WDMAUD_DEVICE_INFO>() as ULONG,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn CreatePinCallback(
    Ctx: PVOID,
    VirtualDeviceId: ULONG,
    PinId: ULONG,
    hFilter: HANDLE,
    PinConnect: PKSPIN_CONNECT,
    DesiredAccess: ACCESS_MASK,
    PinHandle: PHANDLE,
) -> MIXER_STATUS {
    let mut BytesReturned: ULONG = 0;
    let mut InstanceInfo: SYSAUDIO_INSTANCE_INFO = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut FreeIndex: ULONG;
    let mut Context: PPIN_CREATE_CONTEXT = Ctx as PPIN_CREATE_CONTEXT;

    /* setup property request */
    InstanceInfo.Property.Set = KSPROPSETID_Sysaudio;
    InstanceInfo.Property.Id = KSPROPERTY_SYSAUDIO_INSTANCE_INFO;
    InstanceInfo.Property.Flags = KSPROPERTY_TYPE_SET;
    InstanceInfo.Flags = 0;
    InstanceInfo.DeviceNumber = VirtualDeviceId;

    /* attach to virtual device */
    Status = KsSynchronousIoControlDevice(
        (*(*Context).DeviceExtension).FileObject,
        0i32 as KPROCESSOR_MODE,
        IOCTL_KS_PROPERTY,
        core::ptr::addr_of_mut!(InstanceInfo) as PVOID,
        core::mem::size_of::<SYSAUDIO_INSTANCE_INFO>() as ULONG,
        core::ptr::null_mut(),
        0,
        core::ptr::addr_of_mut!(BytesReturned),
    );

    if !NT_SUCCESS!(Status) {
        return MM_STATUS_UNSUCCESSFUL;
    }

    /* close existing pin */
    FreeIndex = ClosePin(
        (*Context).ClientInfo,
        VirtualDeviceId,
        PinId,
        (*Context).DeviceType,
    );

    /* now create the pin */
    Status = KsCreatePin(
        (*(*Context).DeviceExtension).hSysAudio,
        PinConnect,
        DesiredAccess,
        PinHandle,
    );

    /* check for success */
    if !NT_SUCCESS!(Status) {
        return MM_STATUS_UNSUCCESSFUL;
    }

    /* store the handle */
    Status = InsertPinHandle(
        (*Context).ClientInfo,
        VirtualDeviceId,
        PinId,
        (*Context).DeviceType,
        *PinHandle,
        FreeIndex,
    );
    if !NT_SUCCESS!(Status) {
        /* failed to insert handle */
        ZwClose(*PinHandle);
        return MM_STATUS_UNSUCCESSFUL;
    }

    return MM_STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn WdmAudControlOpenWave(
    mut DeviceObject: PDEVICE_OBJECT,
    mut Irp: PIRP,
    mut DeviceInfo: PWDMAUD_DEVICE_INFO,
    mut ClientInfo: PWDMAUD_CLIENT,
) -> NTSTATUS {
    let mut Status: MIXER_STATUS;
    let mut Context: PIN_CREATE_CONTEXT = core::mem::zeroed();

    Context.ClientInfo = ClientInfo;
    Context.DeviceExtension = (*DeviceObject).DeviceExtension as PWDMAUD_DEVICE_EXTENSION;
    Context.DeviceType = (*DeviceInfo).DeviceType;

    Status = MMixerOpenWave(
        &raw mut MixerContext,
        (*DeviceInfo).DeviceIndex as ULONG,
        (((*DeviceInfo).DeviceType == WAVE_IN_DEVICE_TYPE) as ::core::ffi::c_int) as ULONG,
        &raw mut (*DeviceInfo).u.WaveFormatEx,
        Some(
            CreatePinCallback
                as unsafe extern "C" fn(
                    PVOID,
                    ULONG,
                    ULONG,
                    HANDLE,
                    PKSPIN_CONNECT,
                    ACCESS_MASK,
                    PHANDLE,
                ) -> MIXER_STATUS,
        ),
        &raw mut Context as PVOID,
        &raw mut (*DeviceInfo).hDevice,
    );

    if Status == MM_STATUS_SUCCESS {
        return SetIrpIoStatus(
            Irp,
            STATUS_SUCCESS,
            ::core::mem::size_of::<WDMAUD_DEVICE_INFO>() as ULONG,
        );
    } else {
        return SetIrpIoStatus(
            Irp,
            STATUS_NOT_SUPPORTED,
            ::core::mem::size_of::<WDMAUD_DEVICE_INFO>() as ULONG,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn WdmAudControlOpenMidi(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    DeviceInfo: PWDMAUD_DEVICE_INFO,
    ClientInfo: PWDMAUD_CLIENT,
) -> NTSTATUS {
    let mut Status: MIXER_STATUS;
    let mut Context: PIN_CREATE_CONTEXT = core::mem::zeroed();

    Context.ClientInfo = ClientInfo;
    Context.DeviceExtension = (*DeviceObject).DeviceExtension as PWDMAUD_DEVICE_EXTENSION;
    Context.DeviceType = (*DeviceInfo).DeviceType;

    Status = MMixerOpenMidi(
        core::ptr::addr_of_mut!(MixerContext),
        (*DeviceInfo).DeviceIndex as ULONG,
        ((*DeviceInfo).DeviceType == MIDI_IN_DEVICE_TYPE) as ULONG,
        Some(CreatePinCallback),
        core::ptr::addr_of_mut!(Context) as PVOID,
        core::ptr::addr_of_mut!((*DeviceInfo).hDevice),
    );

    if Status == MM_STATUS_SUCCESS {
        return SetIrpIoStatus(
            Irp,
            STATUS_SUCCESS,
            core::mem::size_of::<WDMAUD_DEVICE_INFO>() as ULONG,
        );
    } else {
        return SetIrpIoStatus(
            Irp,
            STATUS_NOT_SUPPORTED,
            core::mem::size_of::<WDMAUD_DEVICE_INFO>() as ULONG,
        );
    }
}
