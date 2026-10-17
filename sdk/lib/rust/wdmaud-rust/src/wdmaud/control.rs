use crate::prelude::*;
use ntkrnl_sys::ddk::{GUID, IoGetRelatedDeviceObject, LPWSTR, MmUnlockPages, ObReferenceObjectByHandle, PDEVICE_OBJECT, PFILE_OBJECT, PIO_STACK_LOCATION, PIRP, PMDL, ZwClose};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut KSPROPSETID_Sysaudio: GUID = _GUID {
    Data1: 0xcbe3faa0 as ::core::ffi::c_ulong as ::core::ffi::c_uint,
    Data2: 0xcc75 as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0xb4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x65 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x1a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x18 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x18 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};
#[no_mangle]
pub unsafe extern "C" fn WdmAudControlOpen(
    mut DeviceObject: PDEVICE_OBJECT,
    mut Irp: PIRP,
    mut DeviceInfo: PWDMAUD_DEVICE_INFO,
    mut ClientInfo: PWDMAUD_CLIENT,
) -> NTSTATUS {
    if (*DeviceInfo).DeviceType as ::core::ffi::c_uint
        == MIXER_DEVICE_TYPE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return WdmAudControlOpenMixer(DeviceObject, Irp, DeviceInfo, ClientInfo);
    }

    if (*DeviceInfo).DeviceType as ::core::ffi::c_uint
        == WAVE_OUT_DEVICE_TYPE as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*DeviceInfo).DeviceType as ::core::ffi::c_uint
            == WAVE_IN_DEVICE_TYPE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return WdmAudControlOpenWave(DeviceObject, Irp, DeviceInfo, ClientInfo);
    }

    if (*DeviceInfo).DeviceType as ::core::ffi::c_uint
        == MIDI_OUT_DEVICE_TYPE as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*DeviceInfo).DeviceType as ::core::ffi::c_uint
            == MIDI_IN_DEVICE_TYPE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return WdmAudControlOpenMidi(DeviceObject, Irp, DeviceInfo, ClientInfo);
    }

    return SetIrpIoStatus(
        Irp,
        STATUS_NOT_SUPPORTED,
        ::core::mem::size_of::<WDMAUD_DEVICE_INFO>() as ULONG,
    );
}
#[no_mangle]
pub unsafe extern "C" fn WdmAudControlDeviceType(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    DeviceInfo: PWDMAUD_DEVICE_INFO,
    ClientInfo: PWDMAUD_CLIENT,
) -> NTSTATUS {
    let mut Result: ULONG = 0;

    if (*DeviceInfo).DeviceType == MIXER_DEVICE_TYPE {
        Result = WdmAudGetMixerDeviceCount();
    } else if (*DeviceInfo).DeviceType == WAVE_OUT_DEVICE_TYPE {
        Result = WdmAudGetWaveOutDeviceCount();
    } else if (*DeviceInfo).DeviceType == WAVE_IN_DEVICE_TYPE {
        Result = WdmAudGetWaveInDeviceCount();
    } else if (*DeviceInfo).DeviceType == MIDI_IN_DEVICE_TYPE {
        Result = WdmAudGetMidiInDeviceCount();
    } else if (*DeviceInfo).DeviceType == MIDI_OUT_DEVICE_TYPE {
        Result = WdmAudGetMidiOutDeviceCount();
    }

    /* store result count */
    (*DeviceInfo).DeviceCount = Result;

    DPRINT!("WdmAudControlDeviceType Devices %u\n", (*DeviceInfo).DeviceCount);
    return SetIrpIoStatus(
        Irp,
        STATUS_SUCCESS,
        core::mem::size_of::<WDMAUD_DEVICE_INFO>() as ULONG,
    );
}
#[no_mangle]
pub unsafe extern "C" fn WdmAudControlDeviceState(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    DeviceInfo: PWDMAUD_DEVICE_INFO,
    ClientInfo: PWDMAUD_CLIENT,
) -> NTSTATUS {
    let mut Property: KSPROPERTY = core::mem::zeroed();
    let mut State: KSSTATE = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut BytesReturned: ULONG = core::mem::zeroed();
    let mut FileObject: PFILE_OBJECT = core::ptr::null_mut();

    DPRINT!("WdmAudControlDeviceState\n");

    Status = ObReferenceObjectByHandle(
        (*DeviceInfo).hDevice,
        GENERIC_READ | GENERIC_WRITE,
        *IoFileObjectType,
        (0i32) as KPROCESSOR_MODE,
        core::ptr::addr_of_mut!(FileObject) as *mut PVOID,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS!(Status) {
        DPRINT1!(
            "Error: invalid device handle provided %p Type %x\n",
            (*DeviceInfo).hDevice,
            (*DeviceInfo).DeviceType
        );
        return SetIrpIoStatus(Irp, STATUS_UNSUCCESSFUL, 0);
    }

    Property.Set = KSPROPSETID_Connection;
    Property.Id = KSPROPERTY_CONNECTION_STATE;
    Property.Flags = KSPROPERTY_TYPE_SET;

    State = (*DeviceInfo).u.State;

    Status = KsSynchronousIoControlDevice(
        FileObject,
        (0i32) as KPROCESSOR_MODE,
        IOCTL_KS_PROPERTY,
        core::ptr::addr_of_mut!(Property) as PVOID,
        core::mem::size_of::<KSPROPERTY>() as ULONG,
        core::ptr::addr_of_mut!(State) as PVOID,
        core::mem::size_of::<KSSTATE>() as ULONG,
        core::ptr::addr_of_mut!(BytesReturned),
    );

    ObDereferenceObject((FileObject).cast());

    DPRINT!("WdmAudControlDeviceState Status %x\n", Status);
    return SetIrpIoStatus(
        Irp,
        Status,
        core::mem::size_of::<WDMAUD_DEVICE_INFO>() as ULONG,
    );
}
#[no_mangle]
pub unsafe extern "C" fn WdmAudCapabilities(
    mut DeviceObject: PDEVICE_OBJECT,
    mut Irp: PIRP,
    mut DeviceInfo: PWDMAUD_DEVICE_INFO,
    mut ClientInfo: PWDMAUD_CLIENT,
) -> NTSTATUS {
    let mut DeviceExtension: PWDMAUD_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut Status: NTSTATUS = STATUS_UNSUCCESSFUL;

    DPRINT!("WdmAudCapabilities entered\n");

    DeviceExtension = (*DeviceObject).DeviceExtension as PWDMAUD_DEVICE_EXTENSION;

    if (*DeviceInfo).DeviceType as ::core::ffi::c_uint == MIXER_DEVICE_TYPE as ::core::ffi::c_uint
    {
        Status = WdmAudMixerCapabilities(DeviceObject, DeviceInfo, ClientInfo, DeviceExtension);
    } else if (*DeviceInfo).DeviceType as ::core::ffi::c_uint
        == WAVE_IN_DEVICE_TYPE as ::core::ffi::c_uint
        || (*DeviceInfo).DeviceType as ::core::ffi::c_uint
            == WAVE_OUT_DEVICE_TYPE as ::core::ffi::c_uint
    {
        Status = WdmAudWaveCapabilities(DeviceObject, DeviceInfo, ClientInfo, DeviceExtension);
    } else if (*DeviceInfo).DeviceType as ::core::ffi::c_uint
        == MIDI_IN_DEVICE_TYPE as ::core::ffi::c_uint
        || (*DeviceInfo).DeviceType as ::core::ffi::c_uint
            == MIDI_OUT_DEVICE_TYPE as ::core::ffi::c_uint
    {
        Status = WdmAudMidiCapabilities(DeviceObject, DeviceInfo, ClientInfo, DeviceExtension);
    }

    return SetIrpIoStatus(
        Irp,
        Status,
        core::mem::size_of::<WDMAUD_DEVICE_INFO>() as ULONG,
    );
}
#[no_mangle]
pub unsafe extern "system" fn WdmAudIoctlClose(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    DeviceInfo: PWDMAUD_DEVICE_INFO,
    ClientInfo: PWDMAUD_CLIENT,
) -> NTSTATUS {
    let mut Index: ULONG;

    Index = 0;
    while Index < (*ClientInfo).NumPins {
        if (*(*ClientInfo).hPins.offset(Index as isize)).Handle == (*DeviceInfo).hDevice
            && (*(*ClientInfo).hPins.offset(Index as isize)).Type != MIXER_DEVICE_TYPE
        {
            DPRINT1!("Closing device %p\n", (*DeviceInfo).hDevice);
            ZwClose((*DeviceInfo).hDevice);
            (*(*ClientInfo).hPins.offset(Index as isize)).Handle = core::ptr::null_mut();
            SetIrpIoStatus(Irp, STATUS_SUCCESS, core::mem::size_of::<WDMAUD_DEVICE_INFO>() as ULONG);
            return STATUS_SUCCESS;
        } else if (*(*ClientInfo).hPins.offset(Index as isize)).Handle == (*DeviceInfo).hDevice
            && (*(*ClientInfo).hPins.offset(Index as isize)).Type == MIXER_DEVICE_TYPE
        {
            DPRINT1!("Closing mixer %p\n", (*DeviceInfo).hDevice);
            return WdmAudControlCloseMixer(DeviceObject, Irp, DeviceInfo, ClientInfo, Index);
        }
        Index = Index.wrapping_add(1);
    }

    SetIrpIoStatus(Irp, STATUS_INVALID_PARAMETER, core::mem::size_of::<WDMAUD_DEVICE_INFO>() as ULONG);
    return STATUS_INVALID_PARAMETER;
}
#[no_mangle]
pub unsafe extern "system" fn WdmAudFrameSize(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    DeviceInfo: PWDMAUD_DEVICE_INFO,
    ClientInfo: PWDMAUD_CLIENT,
) -> NTSTATUS {
    let mut FileObject: PFILE_OBJECT = core::ptr::null_mut();
    let mut Property: KSPROPERTY = core::mem::zeroed();
    let mut BytesReturned: ULONG = 0;
    let mut Framing: KSALLOCATOR_FRAMING = core::mem::zeroed();
    let mut Status: NTSTATUS;

    /* Get sysaudio pin file object */
    Status = ObReferenceObjectByHandle(
        (*DeviceInfo).hDevice,
        GENERIC_WRITE as ACCESS_MASK,
        *IoFileObjectType,
        0i32 as KPROCESSOR_MODE,
        core::ptr::addr_of_mut!(FileObject) as *mut PVOID,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS!(Status) {
        DPRINT1!("Invalid buffer handle %p\n", (*DeviceInfo).hDevice);
        return SetIrpIoStatus(Irp, Status, 0);
    }

    /* Setup get framing request */
    Property.Id = KSPROPERTY_CONNECTION_ALLOCATORFRAMING;
    Property.Flags = KSPROPERTY_TYPE_GET;
    Property.Set = KSPROPSETID_Connection;

    Status = KsSynchronousIoControlDevice(
        FileObject,
        0i32 as KPROCESSOR_MODE,
        IOCTL_KS_PROPERTY as ULONG,
        core::ptr::addr_of_mut!(Property) as PVOID,
        core::mem::size_of::<KSPROPERTY>() as ULONG,
        core::ptr::addr_of_mut!(Framing) as PVOID,
        core::mem::size_of::<KSALLOCATOR_FRAMING>() as ULONG,
        core::ptr::addr_of_mut!(BytesReturned),
    );
    /* Did we succeed */
    if NT_SUCCESS!(Status) {
        /* Store framesize */
        (*DeviceInfo).u.FrameSize = Framing.FrameSize;
    }

    /* Release file object */
    ObDereferenceObject((FileObject).cast());

    return SetIrpIoStatus(
        Irp,
        Status,
        core::mem::size_of::<WDMAUD_DEVICE_INFO>() as ULONG,
    );
}
#[no_mangle]
pub unsafe extern "system" fn WdmAudGetDeviceInterface(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    DeviceInfo: PWDMAUD_DEVICE_INFO,
) -> NTSTATUS {
    let mut Status: NTSTATUS = 0;
    let mut Device: LPWSTR = core::ptr::null_mut();
    let mut Size: ULONG = 0;
    let mut Length: ULONG = 0;
    let mut _SEH2_Volatile0_local: i32 = 0;
    let mut _SEH2_VolatileExceptionCode_local: NTSTATUS = 0;

    /* get device interface string input length */
    Size = (*DeviceInfo).u.Interface.DeviceInterfaceStringSize;

    /* get mixer info */
    Status = WdmAudGetPnpNameByIndexAndType(
        (*DeviceInfo).DeviceIndex as ULONG,
        (*DeviceInfo).DeviceType,
        core::ptr::addr_of_mut!(Device),
    );

    /* check for success */
    if !NT_SUCCESS!(Status) {
        /* invalid device id */
        return SetIrpIoStatus(
            Irp,
            Status,
            core::mem::size_of::<WDMAUD_DEVICE_INFO>() as ULONG,
        );
    }

    /* calculate length */
    Length = ((wcslen(Device as *const WCHAR) as ULONG)
        + (1 as ULONG))
        * (core::mem::size_of::<WCHAR>() as ULONG);

    if Size == 0 {
        /* store device interface size */
        (*DeviceInfo).u.Interface.DeviceInterfaceStringSize = Length;
    } else if Size < Length {
        /* buffer too small */
        (*DeviceInfo).u.Interface.DeviceInterfaceStringSize = Length;
        FreeItem(Device as PVOID);
        return SetIrpIoStatus(
            Irp,
            STATUS_BUFFER_OVERFLOW,
            core::mem::size_of::<WDMAUD_DEVICE_INFO>() as ULONG,
        );
    } else {
        // __try { ProbeForWrite; copy } __except (EXCEPTION_EXECUTE_HANDLER): the user
        // buffer is probed under a real handler (ntkrnl_sys::seh), not the dead PSEH arm.
        let caught = ntkrnl_sys::seh::try_except(
            &mut (),
            |_| {
                ProbeForWrite(
                    (*DeviceInfo).u.Interface.DeviceInterfaceString as PVOID,
                    Length as SIZE_T,
                    core::mem::size_of::<WCHAR>() as ULONG,
                );
                core::ptr::copy(
                    Device as *const u8,
                    (*DeviceInfo).u.Interface.DeviceInterfaceString as *mut u8,
                    Length as usize,
                );
            },
            |_, _| ntkrnl_sys::seh::EXCEPTION_EXECUTE_HANDLER,
        );
        if let Err(code) = caught {
            FreeItem(Device as PVOID);
            return SetIrpIoStatus(Irp, code, 0 as ULONG);
        }
    }

    FreeItem(Device as PVOID);
    return SetIrpIoStatus(
        Irp,
        STATUS_SUCCESS,
        core::mem::size_of::<WDMAUD_DEVICE_INFO>() as ULONG,
    );
}
#[no_mangle]
pub unsafe extern "system" fn WdmAudResetStream(
        DeviceObject: PDEVICE_OBJECT,
        Irp: PIRP,
        DeviceInfo: PWDMAUD_DEVICE_INFO) -> NTSTATUS {
    let mut ResetStream: KSRESET = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut BytesReturned: ULONG = 0;
    let mut FileObject: PFILE_OBJECT = core::ptr::null_mut();

    DPRINT!("WdmAudResetStream\n");

    Status = ObReferenceObjectByHandle((*DeviceInfo).hDevice, GENERIC_READ | GENERIC_WRITE, *IoFileObjectType, (0i32) as KPROCESSOR_MODE, core::ptr::addr_of_mut!(FileObject) as *mut PVOID, core::ptr::null_mut());
    if !(NT_SUCCESS!(Status)) {
        DPRINT1!("Error: invalid device handle provided %p Type %x\n", (*DeviceInfo).hDevice, (*DeviceInfo).DeviceType);
        return SetIrpIoStatus(Irp, STATUS_UNSUCCESSFUL, 0);
    }

    ResetStream = (*DeviceInfo).u.ResetStream;
    ASSERT!(ResetStream == KSRESET_BEGIN || ResetStream == KSRESET_END);

    Status = KsSynchronousIoControlDevice(FileObject, (0i32) as KPROCESSOR_MODE, IOCTL_KS_RESET_STATE, core::ptr::addr_of_mut!(ResetStream) as PVOID, core::mem::size_of::<KSRESET>() as ULONG, core::ptr::null_mut(), 0, core::ptr::addr_of_mut!(BytesReturned));

    ObDereferenceObject(FileObject as PVOID);

    DPRINT!("WdmAudResetStream Status %x\n", Status);
    return SetIrpIoStatus(Irp, Status, core::mem::size_of::<WDMAUD_DEVICE_INFO>() as ULONG);
}
#[no_mangle]
pub unsafe extern "system" fn WdmAudDeviceControl(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    let mut IoStack: PIO_STACK_LOCATION = core::ptr::null_mut();
    let mut DeviceInfo: PWDMAUD_DEVICE_INFO = core::ptr::null_mut();
    let mut ClientInfo: PWDMAUD_CLIENT = core::ptr::null_mut();

    IoStack = IoGetCurrentIrpStackLocation(Irp);

    DPRINT!("WdmAudDeviceControl entered\n");

    if ((*IoStack).Parameters.DeviceIoControl.InputBufferLength as usize)
        < core::mem::size_of::<WDMAUD_DEVICE_INFO>()
    {
        /* invalid parameter */
        DPRINT1!(
            "Input buffer too small size %u expected %u\n",
            (*IoStack).Parameters.DeviceIoControl.InputBufferLength,
            core::mem::size_of::<WDMAUD_DEVICE_INFO>()
        );
        return SetIrpIoStatus(Irp, STATUS_INVALID_PARAMETER, 0);
    }

    DeviceInfo = (*Irp).AssociatedIrp.SystemBuffer as PWDMAUD_DEVICE_INFO;

    if ((*DeviceInfo).DeviceType as u32) < MIN_SOUND_DEVICE_TYPE
        || (*DeviceInfo).DeviceType as u32 > MAX_SOUND_DEVICE_TYPE
    {
        /* invalid parameter */
        DPRINT1!("Error: device type not set\n");
        return SetIrpIoStatus(Irp, STATUS_INVALID_PARAMETER, 0);
    }

    if (*IoStack).FileObject.is_null() || (*(*IoStack).FileObject).FsContext.is_null() {
        /* file object parameter */
        DPRINT1!("Error: file object is not attached\n");
        return SetIrpIoStatus(Irp, STATUS_UNSUCCESSFUL, 0);
    }
    ClientInfo = (*(*IoStack).FileObject).FsContext as PWDMAUD_CLIENT;

    DPRINT!("WdmAudDeviceControl entered\n");

    match (*IoStack).Parameters.DeviceIoControl.IoControlCode {
        IOCTL_OPEN_WDMAUD => {
            return WdmAudControlOpen(DeviceObject, Irp, DeviceInfo, ClientInfo);
        }
        IOCTL_GETNUMDEVS_TYPE => {
            return WdmAudControlDeviceType(DeviceObject, Irp, DeviceInfo, ClientInfo);
        }
        IOCTL_SETDEVICE_STATE => {
            return WdmAudControlDeviceState(DeviceObject, Irp, DeviceInfo, ClientInfo);
        }
        IOCTL_GETCAPABILITIES => {
            return WdmAudCapabilities(DeviceObject, Irp, DeviceInfo, ClientInfo);
        }
        IOCTL_CLOSE_WDMAUD => {
            return WdmAudIoctlClose(DeviceObject, Irp, DeviceInfo, ClientInfo);
        }
        IOCTL_GETFRAMESIZE => {
            return WdmAudFrameSize(DeviceObject, Irp, DeviceInfo, ClientInfo);
        }
        IOCTL_GETLINEINFO => {
            return WdmAudGetLineInfo(DeviceObject, Irp, DeviceInfo, ClientInfo);
        }
        IOCTL_GETLINECONTROLS => {
            return WdmAudGetLineControls(DeviceObject, Irp, DeviceInfo, ClientInfo);
        }
        IOCTL_SETCONTROLDETAILS => {
            return WdmAudSetControlDetails(DeviceObject, Irp, DeviceInfo, ClientInfo);
        }
        IOCTL_GETCONTROLDETAILS => {
            return WdmAudGetControlDetails(DeviceObject, Irp, DeviceInfo, ClientInfo);
        }
        IOCTL_QUERYDEVICEINTERFACESTRING => {
            return WdmAudGetDeviceInterface(DeviceObject, Irp, DeviceInfo);
        }
        IOCTL_GET_MIXER_EVENT => {
            return WdmAudGetMixerEvent(DeviceObject, Irp, DeviceInfo, ClientInfo);
        }
        IOCTL_RESET_STREAM => {
            return WdmAudResetStream(DeviceObject, Irp, DeviceInfo);
        }
        IOCTL_GETPOS => {
            return WdmAudGetPosition(DeviceObject, Irp, DeviceInfo);
        }
        IOCTL_GETDEVID | IOCTL_GETVOLUME | IOCTL_SETVOLUME => {
            DPRINT1!(
                "Unhandled %x\n",
                (*IoStack).Parameters.DeviceIoControl.IoControlCode
            );
        }
        _ => {}
    }

    return SetIrpIoStatus(Irp, STATUS_NOT_IMPLEMENTED, 0);
}
#[no_mangle]
pub unsafe extern "system" fn IoCompletion(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Ctx: PVOID,
) -> NTSTATUS {
    let mut Header: PKSSTREAM_HEADER;
    let mut Mdl: PMDL;
    let mut NextMdl: PMDL;
    let Context: PWDMAUD_COMPLETION_CONTEXT = Ctx as PWDMAUD_COMPLETION_CONTEXT;

    /* get stream header */
    Header = (*Irp).AssociatedIrp.SystemBuffer as PKSSTREAM_HEADER;

    /* sanity check */
    ASSERT!(!Header.is_null());

    /* time to free all allocated mdls */
    Mdl = (*Irp).MdlAddress as PMDL;

    while !Mdl.is_null() {
        /* get next mdl */
        NextMdl = (*Mdl).Next as PMDL;

        /* unlock pages */
        MmUnlockPages(Mdl);

        /* grab next mdl */
        Mdl = NextMdl;
    }
    //IoFreeMdl(Mdl);
    /* clear mdl list */
    (*Irp).MdlAddress = (*Context).Mdl as *mut _MDL;

    DPRINT!(
        "IoCompletion Irp %p IoStatus %lx Information %lx\n",
        Irp,
        (*Irp).IoStatus.Status,
        (*Irp).IoStatus.Information
    );

    if !NT_SUCCESS!((*Irp).IoStatus.__bindgen_anon_1.Status) {
        /* failed */
        (*Irp).IoStatus.Information = 0;
    }

    /* dereference file object */
    ObDereferenceObject((*Context).FileObject as PVOID);

    /* free context */
    FreeItem(Context as PVOID);

    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "system" fn WdmAudReadWrite(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut DeviceInfo: PWDMAUD_DEVICE_INFO;
    let mut FileObject: PFILE_OBJECT = core::ptr::null_mut();
    let mut IoStack: PIO_STACK_LOCATION;
    let mut Length: ULONG;
    let mut Mdl: PMDL;
    let mut Read: BOOLEAN = TRUE;
    let mut Context: PWDMAUD_COMPLETION_CONTEXT;

    /* allocate completion context */
    Context = AllocateItem(NonPagedPool, core::mem::size_of::<WDMAUD_COMPLETION_CONTEXT>() as SIZE_T) as PWDMAUD_COMPLETION_CONTEXT;

    if Context.is_null() {
        /* not enough memory */
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_INSUFFICIENT_RESOURCES;
        IoCompleteRequest(Irp, (IO_NO_INCREMENT as i8));

        /* done */
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    /* get current irp stack location */
    IoStack = IoGetCurrentIrpStackLocation(Irp);

    /* store the input buffer in UserBuffer - as KsProbeStreamIrp operates on IRP_MJ_DEVICE_CONTROL */
    (*Irp).UserBuffer = MmGetMdlVirtualAddress((*Irp).MdlAddress);

    /* sanity check */
    ASSERT!((*Irp).UserBuffer);

    /* get the length of the request length */
    Length = (*IoStack).Parameters.Write.Length;

    /* store outputbuffer length */
    (*IoStack).Parameters.DeviceIoControl.OutputBufferLength = Length;

    /* setup context */
    (*Context).Length = Length;
    (*Context).Function = (if (*IoStack).MajorFunction as ::core::ffi::c_int == (IRP_MJ_WRITE as i32) {
        IOCTL_KS_WRITE_STREAM
    } else {
        IOCTL_KS_READ_STREAM
    }) as ULONG;
    (*Context).Mdl = (*Irp).MdlAddress;

    /* store mdl address */
    Mdl = (*Irp).MdlAddress;

    /* remove mdladdress as KsProbeStreamIrp will interpret it as an already probed audio buffer */
    (*Irp).MdlAddress = core::ptr::null_mut();

    if (*IoStack).MajorFunction as ::core::ffi::c_int == (IRP_MJ_WRITE as i32) {
        /* probe the write stream irp */
        Read = FALSE;
        Status = KsProbeStreamIrp(Irp, KSPROBE_STREAMWRITE | KSPROBE_ALLOCATEMDL | KSPROBE_PROBEANDLOCK, Length);
    } else {
        /* probe the read stream irp */
        Status = KsProbeStreamIrp(Irp, KSPROBE_STREAMREAD | KSPROBE_ALLOCATEMDL | KSPROBE_PROBEANDLOCK, Length);
    }

    if !NT_SUCCESS!(Status) {
        DPRINT1!("KsProbeStreamIrp failed with Status %x Cancel %u\n", Status, (*Irp).Cancel as core::ffi::c_uint);
        (*Irp).MdlAddress = Mdl;
        FreeItem((Context).cast());
        return SetIrpIoStatus(Irp, Status, 0);
    }

    /* get device info */
    DeviceInfo = (*Irp).AssociatedIrp.SystemBuffer as PWDMAUD_DEVICE_INFO;
    ASSERT!(DeviceInfo);

    /* now get sysaudio file object */
    Status = ObReferenceObjectByHandle(
        (*DeviceInfo).hDevice,
        GENERIC_WRITE,
        *IoFileObjectType,
        (0i32) as KPROCESSOR_MODE,
        core::ptr::addr_of_mut!(FileObject) as *mut PVOID,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS!(Status) {
        DPRINT1!("Invalid pin handle %p\n", (*DeviceInfo).hDevice);
        (*Irp).MdlAddress = Mdl;
        FreeItem((Context).cast());
        return SetIrpIoStatus(Irp, Status, 0);
    }

    /* store file object whose reference is released in the completion callback */
    (*Context).FileObject = FileObject;

    /* skip current irp stack location */
    IoSkipCurrentIrpStackLocation(Irp);

    /* get next stack location */
    IoStack = IoGetNextIrpStackLocation(Irp);

    /* prepare stack location */
    (*IoStack).FileObject = FileObject;
    (*IoStack).Parameters.Write.Length = Length;
    (*IoStack).MajorFunction = (IRP_MJ_WRITE as u8);
    (*IoStack).Parameters.DeviceIoControl.IoControlCode = (if Read as ::core::ffi::c_int != 0 {
        IOCTL_KS_READ_STREAM
    } else {
        IOCTL_KS_WRITE_STREAM
    }) as ULONG;
    IoSetCompletionRoutine(
        Irp,
        Some(IoCompletion),
        Context as PVOID,
        TRUE,
        TRUE,
        TRUE,
    );

    /* mark irp as pending */
//    IoMarkIrpPending(Irp);
    /* call the driver */
    Status = IoCallDriver(IoGetRelatedDeviceObject(FileObject), Irp);
    return Status;
}
