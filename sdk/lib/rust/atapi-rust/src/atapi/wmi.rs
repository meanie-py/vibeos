use crate::prelude::*;
use ntkrnl_sys::ddk::{IoWMIRegistrationControl, PDEVICE_OBJECT, PIRP, PUNICODE_STRING};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut MSIde_PortDeviceInfo_GUID: GUID = _GUID {
    Data1: 0x53f5630f as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xb6bf as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0x94 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x1e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xfb as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8b as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

static mut AtapWmiGuidList: [WMIGUIDREGINFO; 1] = unsafe {
    [_WMIGUIDREGINFO {
        Guid: &raw const MSIde_PortDeviceInfo_GUID,
        InstanceCount: 1 as ULONG,
        Flags: 0 as ULONG,
    }]
};
unsafe extern "system" fn AtaQueryWmiRegInfo(mut DeviceObject: PDEVICE_OBJECT, mut RegFlags: PULONG, mut InstanceName: PUNICODE_STRING, mut RegistryPath: *mut PUNICODE_STRING, mut MofResourceName: PUNICODE_STRING, mut Pdo: *mut PDEVICE_OBJECT) -> NTSTATUS {
    let mut DevExt: PATAPORT_DEVICE_EXTENSION =
        (*DeviceObject).DeviceExtension as PATAPORT_DEVICE_EXTENSION;
    if (KeGetCurrentIrql() as ::core::ffi::c_int) as u32 > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as PCSTR,
            KeGetCurrentIrql() as ::core::ffi::c_int,
        );
        if 0 as ::core::ffi::c_int == 0 {
            RtlAssert(
                b"0\0" as *const u8 as *const ::core::ffi::c_char as PVOID,
                b"drivers/storage/ide/atapi/wmi.c\0"
                    as *const u8 as *const ::core::ffi::c_char as PVOID,
                38 as ULONG,
                ::core::ptr::null_mut::<CHAR>(),
            );
        } else {
        };
    }
    *RegFlags = WMIREG_FLAG_INSTANCE_PDO;
    *RegistryPath = &raw mut AtapDriverRegistryPath as PUNICODE_STRING;
    *Pdo = (*DevExt).Common.Self_;
    return STATUS_SUCCESS;
}
unsafe extern "system" fn AtaQueryWmiDataBlock(DeviceObject: PDEVICE_OBJECT, Irp: PIRP, GuidIndex: ULONG, InstanceIndex: ULONG, InstanceCount: ULONG, InstanceLengthArray: PULONG, BufferAvail: ULONG, Buffer: PUCHAR) -> NTSTATUS {
    let mut DevExt: *mut _ATAPORT_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut Status: i32 = 0;
    let mut AtaScsiAddress: ATA_SCSI_ADDRESS = { let mut __lit: ATA_SCSI_ADDRESS = core::mem::zeroed(); __lit.__bindgen_anon_1.Lun = 0; __lit.__bindgen_anon_1.TargetId = 0; __lit.__bindgen_anon_1.PathId = 0; __lit.__bindgen_anon_1.IsValid = 0; __lit };
    let mut DeviceInfo: *mut _MSIde_PortDeviceInfo = core::ptr::null_mut();

    PAGED_CODE!();

    if GuidIndex as usize > RTL_NUMBER_OF!(AtapWmiGuidList) as usize {
        Status = STATUS_WMI_GUID_NOT_FOUND;
        return WmiCompleteRequest(
            DeviceObject,
            Irp,
            Status,
            core::mem::size_of::<_MSIde_PortDeviceInfo>() as u32,
            (IO_NO_INCREMENT) as CCHAR,
        );
    }

    if InstanceIndex != 0 || InstanceCount != 1 {
        Status = STATUS_WMI_INSTANCE_NOT_FOUND;
        return WmiCompleteRequest(
            DeviceObject,
            Irp,
            Status,
            core::mem::size_of::<_MSIde_PortDeviceInfo>() as u32,
            (IO_NO_INCREMENT) as CCHAR,
        );
    }

    if InstanceLengthArray.is_null()
        || (BufferAvail as usize) < core::mem::size_of::<_MSIde_PortDeviceInfo>()
    {
        Status = STATUS_BUFFER_TOO_SMALL;
        return WmiCompleteRequest(
            DeviceObject,
            Irp,
            Status,
            core::mem::size_of::<_MSIde_PortDeviceInfo>() as u32,
            (IO_NO_INCREMENT) as CCHAR,
        );
    }

    DevExt = (*DeviceObject).DeviceExtension as *mut _ATAPORT_DEVICE_EXTENSION;
    AtaScsiAddress = (*DevExt).Device.AtaScsiAddress;

    DeviceInfo = Buffer as *mut _MSIde_PortDeviceInfo;
    (*DeviceInfo).Bus = AtaScsiAddress.PathId;
    (*DeviceInfo).Target = AtaScsiAddress.TargetId;
    (*DeviceInfo).Lun = AtaScsiAddress.Lun;

    *InstanceLengthArray = core::mem::size_of::<_MSIde_PortDeviceInfo>() as u32;

    Status = STATUS_SUCCESS;

    return WmiCompleteRequest(
        DeviceObject,
        Irp,
        Status,
        core::mem::size_of::<_MSIde_PortDeviceInfo>() as u32,
        (IO_NO_INCREMENT) as CCHAR,
    );
}

unsafe extern "C" fn AtaPdoWmi(
    DevExt: PATAPORT_DEVICE_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut Disposition: SYSCTL_IRP_DISPOSITION = core::mem::zeroed();

    PAGED_CODE!();

    Status = IoAcquireRemoveLock(&raw mut (*DevExt).Common.RemoveLock, (Irp).cast());
    if !NT_SUCCESS!(Status) {
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        IoCompleteRequest(Irp, (IO_NO_INCREMENT) as CCHAR);

        return Status;
    }

    Status = WmiSystemControl(
        &raw mut (*DevExt).WmiLibInfo,
        (*DevExt).Common.Self_,
        Irp,
        &raw mut Disposition,
    );
    match Disposition {
        IrpProcessed => {}

        IrpNotCompleted => {
            IoCompleteRequest(Irp, (IO_NO_INCREMENT) as CCHAR);
        }

        IrpForward | IrpNotWmi => {
            Status = STATUS_NOT_SUPPORTED;
            (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
            IoCompleteRequest(Irp, (IO_NO_INCREMENT) as CCHAR);
        }

        // The C's `default:` sits before the last case; a Rust `_` must come last.
        _ => {
            ASSERT!(FALSE);
            Status = STATUS_NOT_SUPPORTED;
            (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
            IoCompleteRequest(Irp, (IO_NO_INCREMENT) as CCHAR);
        }
    }

    IoReleaseRemoveLock(&raw mut (*DevExt).Common.RemoveLock, (Irp).cast());

    return Status;
}

unsafe extern "C" fn AtaFdoWmi(
    ChanExt: PATAPORT_CHANNEL_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    PAGED_CODE!();
    IoSkipCurrentIrpStackLocation(Irp);
    return IoCallDriver((*ChanExt).Common.__bindgen_anon_1.LowerDeviceObject, Irp);
}

#[no_mangle]
pub unsafe extern "system" fn AtaDispatchWmi(DeviceObject: *mut _DEVICE_OBJECT, Irp: *mut _IRP) -> NTSTATUS {
    PAGED_CODE!();

    if IS_FDO!((*DeviceObject).DeviceExtension) != false {
        return AtaFdoWmi(((*DeviceObject).DeviceExtension).cast(), Irp);
    } else {
        return AtaPdoWmi(((*DeviceObject).DeviceExtension).cast(), Irp);
    }
}
#[no_mangle]
pub unsafe extern "C" fn AtaPdoWmiRegistration(
    DevExt: *mut ATAPORT_DEVICE_EXTENSION,
    Register: u8,
) -> NTSTATUS {
    let mut Action: u32;

    PAGED_CODE!();

    if Register != 0 {
        (*DevExt).WmiLibInfo.GuidCount = RTL_NUMBER_OF!(AtapWmiGuidList);
        (*DevExt).WmiLibInfo.GuidList = AtapWmiGuidList.as_mut_ptr();

        (*DevExt).WmiLibInfo.QueryWmiRegInfo = Some(AtaQueryWmiRegInfo);
        (*DevExt).WmiLibInfo.QueryWmiDataBlock = Some(AtaQueryWmiDataBlock);

        Action = WMIREG_ACTION_REGISTER;
    } else {
        Action = WMIREG_ACTION_DEREGISTER;
    }

    return IoWMIRegistrationControl((*DevExt).Common.Self_, Action);
}
