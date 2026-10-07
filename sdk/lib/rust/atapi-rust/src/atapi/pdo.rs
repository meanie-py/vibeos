// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
pub static RUSTCONV_WSTR_0E21877A26E8: [u16; 30] = [
    0x0043u16, 0x0068, 0x0061, 0x006E, 0x006E, 0x0065, 0x006C, 0x0020, 0x0025, 0x0075,
    0x002C, 0x0020, 0x0054, 0x0061, 0x0072, 0x0067, 0x0065, 0x0074, 0x0020, 0x0025,
    0x0075, 0x002C, 0x0020, 0x004C, 0x0075, 0x006E, 0x0020, 0x0025, 0x0075, 0x0000,
];
pub static RUSTCONV_WSTR_245E26C7A284: [u16; 11] = [
    0x0049u16, 0x0044, 0x0045, 0x005C, 0x0025, 0x002E, 0x0034, 0x0038, 0x0068, 0x0073,
    0x0000,
];
pub static RUSTCONV_WSTR_341441D4E8BF: [u16; 14] = [
    0x0049u16, 0x0044, 0x0045, 0x005C, 0x0025, 0x0068, 0x0073, 0x0025, 0x002E, 0x0034,
    0x0038, 0x0068, 0x0073, 0x0000,
];
pub static RUSTCONV_WSTR_34675BEEA782: [u16; 7] = [0x0025u16, 0x002E, 0x0034, 0x0038, 0x0068, 0x0073, 0x0000];
pub static RUSTCONV_WSTR_6BE0C8F6E502: [u16; 38] = [
    0x005Cu16, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x005C, 0x0049, 0x0064,
    0x0065, 0x005C, 0x0049, 0x0064, 0x0065, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063,
    0x0065, 0x0050, 0x0025, 0x006C, 0x0075, 0x0054, 0x0025, 0x006C, 0x0075, 0x004C,
    0x0025, 0x006C, 0x0075, 0x002D, 0x0025, 0x006C, 0x0078, 0x0000,
];
pub static RUSTCONV_WSTR_8D44D4323B7D: [u16; 4] = [0x0025u16, 0x0068, 0x0073, 0x0000];
pub static RUSTCONV_WSTR_A1EB8D485789: [u16; 9] = [0x0025u16, 0x0078, 0x002E, 0x0025, 0x0078, 0x002E, 0x0025, 0x0078, 0x0000];
pub static RUSTCONV_WSTR_FA5668E0AE70: [u16; 14] = [
    0x0049u16, 0x0044, 0x0045, 0x005C, 0x0025, 0x0068, 0x0073, 0x0025, 0x002E, 0x0034,
    0x0030, 0x0068, 0x0073, 0x0000,
];

use crate::prelude::*;
use ntkrnl_sys::ddk::{DEVICE_CAPABILITIES, ExAllocatePoolUninitialized, ExAllocatePoolZero, _KWAIT_REASON_Executive, IoCreateDevice, IoDeleteDevice, KeInitializeEvent, KeInitializeSpinLock, KeWaitForSingleObject, _EVENT_TYPE_NotificationEvent, PCSTR, PDEVICE_CAPABILITIES, PDEVICE_OBJECT, PDEVICE_RELATIONS, PIO_STACK_LOCATION, PIRP, RtlInitUnicodeString};
use core::ffi::c_void;

unsafe extern "C" fn AtaPdoStartDevice(
    mut DevExt: PATAPORT_DEVICE_EXTENSION,
    mut Irp: PIRP,
) -> NTSTATUS {
    PAGED_CODE!();

    if (*DevExt).Device.DeviceFlags & DEVICE_PNP_STARTED == 0 {
        let mut PortData: PATAPORT_PORT_DATA = (*DevExt).Device.PortData as PATAPORT_PORT_DATA;

        AtaPdoWmiRegistration(DevExt, TRUE);

        (*PortData)
            .SetDeviceData
            .expect("non-null function pointer")(
            (*PortData).ChannelContext,
            (*DevExt).Common.Self_,
            &raw mut (*DevExt).__bindgen_anon_1.IdentifyDeviceData,
        );

        if (*DevExt).GtfDataBuffer.is_null() {
            (*DevExt).GtfDataBuffer = (*PortData)
                .GetInitTaskFile
                .expect("non-null function pointer")(
                (*PortData).ChannelContext,
                (*DevExt).Common.Self_,
            );
        }

        (*DevExt).Device.PowerIdleCounter = PoRegisterDeviceForIdleDetection(
            (*DevExt).Common.Self_ as *mut _DEVICE_OBJECT,
            -(1 as i32) as ULONG,
            -(1 as i32) as ULONG,
            PowerDeviceD3,
        );

        KeClearEvent(&raw mut (*DevExt).Worker.ConfigureEvent);
        AtaDeviceQueueEvent(
            (*DevExt).Device.PortData as PATAPORT_PORT_DATA,
            DevExt,
            ACTION_DEVICE_CONFIG,
        );
        KeWaitForSingleObject(
            &raw mut (*DevExt).Worker.ConfigureEvent as PVOID,
            _KWAIT_REASON_Executive,
            (0i32) as KPROCESSOR_MODE,
            FALSE,
            core::ptr::null_mut::<_LARGE_INTEGER>(),
        );

        (*DevExt).Device.DeviceFlags |= DEVICE_PNP_STARTED;
    }

    AtaReqThawQueue(DevExt, QUEUE_FLAG_FROZEN_PNP);
    return STATUS_SUCCESS;
}

unsafe extern "C" fn AtaPdoStopDevice(
    DevExt: PATAPORT_DEVICE_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    PAGED_CODE!();

    AtaReqFreezeQueue(DevExt, QUEUE_FLAG_FROZEN_PNP);
    AtaReqWaitForOutstandingIoToComplete(core::ptr::addr_of_mut!((*DevExt).Device), core::ptr::null_mut());

    return STATUS_SUCCESS;
}

unsafe extern "C" fn AtaPdoRemoveDevice(
    DevExt: PATAPORT_DEVICE_EXTENSION,
    Irp: PIRP,
    FinalRemove: BOOLEAN,
) -> NTSTATUS {
    let ChanExt: PATAPORT_CHANNEL_EXTENSION = ((*DevExt).Common.__bindgen_anon_1.FdoExt).cast();
    let mut Status: NTSTATUS;
    let mut WasRemoved: BOOLEAN = FALSE;

    PAGED_CODE!();

    AtaReqFreezeQueue(DevExt, QUEUE_FLAG_FROZEN_REMOVED);
    AtaReqFlushDeviceQueue(&raw mut (*DevExt).Device);
    AtaPdoWmiRegistration(DevExt, FALSE);

    if FinalRemove != 0 && (*DevExt).ReportedMissing != 0 {
        IoReleaseRemoveLockAndWait(&raw mut (*DevExt).Common.RemoveLock, (Irp).cast());

        AtaFdoDeviceListInsert(ChanExt, DevExt, FALSE);
        AtaPdoFreeDevice((DevExt).cast());
        WasRemoved = TRUE;
    }

    Status = STATUS_SUCCESS;
    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    IoCompleteRequest(Irp, (IO_NO_INCREMENT) as CCHAR);

    if WasRemoved == 0 {
        IoReleaseRemoveLock(&raw mut (*DevExt).Common.RemoveLock, (Irp).cast());
    }
    return Status;
}

unsafe extern "C" fn AtaPdoQueryStopRemoveDevice(
    DevExt: PATAPORT_DEVICE_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    if (*DevExt).Common.PageFiles != 0
        || (*DevExt).Common.HibernateFiles != 0
        || (*DevExt).Common.DumpFiles != 0
    {
        return STATUS_DEVICE_BUSY;
    }
    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn AtaPdoCompletionRoutine(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Context: PVOID,
) -> NTSTATUS {
    UNREFERENCED_PARAMETER!(DeviceObject);

    if (*Irp).PendingReturned != 0 {
        KeSetEvent((Context).cast(), (IO_NO_INCREMENT) as KPRIORITY, FALSE);
    }

    return STATUS_MORE_PROCESSING_REQUIRED;
}

unsafe extern "C" fn AtaPdoQueryCapabilities(
    DevExt: PATAPORT_DEVICE_EXTENSION,
    Irp: PIRP,
    IoStack: PIO_STACK_LOCATION,
) -> NTSTATUS {
    let mut ParentCapabilities: DEVICE_CAPABILITIES = core::mem::zeroed();
    let mut DeviceCapabilities: PDEVICE_CAPABILITIES = core::ptr::null_mut();
    let mut Status: NTSTATUS;
    let mut AtaScsiAddress: ATA_SCSI_ADDRESS;

    PAGED_CODE!();

    RtlZeroMemory!(
        &mut ParentCapabilities as *mut DEVICE_CAPABILITIES as *mut core::ffi::c_void,
        core::mem::size_of::<DEVICE_CAPABILITIES>(),
    );
    ParentCapabilities.Size = core::mem::size_of::<DEVICE_CAPABILITIES>() as u16;
    ParentCapabilities.Version = 1;
    ParentCapabilities.Address = MAXULONG;
    ParentCapabilities.UINumber = MAXULONG;
    Status = AtaPnpRepeatRequest(
        &mut (*DevExt).Common,
        Irp,
        &mut ParentCapabilities,
    );
    if !NT_SUCCESS!(Status) {
        return Status;
    }

    DeviceCapabilities = (*IoStack).Parameters.DeviceCapabilities.Capabilities;
    RtlCopyMemory!(
        DeviceCapabilities as *mut core::ffi::c_void,
        &mut ParentCapabilities as *mut DEVICE_CAPABILITIES as *const core::ffi::c_void,
        core::mem::size_of::<DEVICE_CAPABILITIES>(),
    );

    AtaScsiAddress = (*DevExt).Device.AtaScsiAddress;

    (*DeviceCapabilities).UINumber = AtaScsiAddress.TargetId as u32;
    (*DeviceCapabilities).set_UniqueID((FALSE as u32) as _);
    (*DeviceCapabilities).set_SurpriseRemovalOK((FALSE as u32) as _);
    (*DeviceCapabilities).set_Removable(((((*DevExt).Device.DeviceFlags & DEVICE_IS_PDO_REMOVABLE) != 0) as u32) as _);
    (*DeviceCapabilities).D1Latency = 31 * 10000;
    (*DeviceCapabilities).D2Latency = (*DeviceCapabilities).D1Latency;
    (*DeviceCapabilities).D3Latency = (*DeviceCapabilities).D2Latency;
    (*DeviceCapabilities).Address =
        ((AtaScsiAddress.Lun as u32) << 4) | (AtaScsiAddress.TargetId as u32);

    return STATUS_SUCCESS;
}

unsafe extern "C" fn AtaPdoQueryTargetDeviceRelations(
    DevExt: PATAPORT_DEVICE_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    let mut DeviceRelations: PDEVICE_RELATIONS;

    PAGED_CODE!();

    DeviceRelations = ExAllocatePoolUninitialized(
        PagedPool,
        (core::mem::size_of::<DEVICE_RELATIONS>()) as _,
        ATAPORT_TAG,
    ) as PDEVICE_RELATIONS;
    if DeviceRelations.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*DeviceRelations).Count = 1;
    *(*DeviceRelations).Objects.as_mut_ptr().add(0) = (*DevExt).Common.Self_;
    ObReferenceObject((*DevExt).Common.Self_);

    (*Irp).IoStatus.Information = DeviceRelations as ULONG_PTR;
    return STATUS_SUCCESS;
}

unsafe extern "C" fn AtaPdoQueryId(
    DevExt: PATAPORT_DEVICE_EXTENSION,
    Irp: PIRP,
    IoStack: PIO_STACK_LOCATION,
) -> NTSTATUS {
    let mut LocalBuffer: [CHAR; (ATAPORT_FN_FIELD + ATAPORT_RN_FIELD) as _] = [0; (ATAPORT_FN_FIELD + ATAPORT_RN_FIELD) as _];
    let mut Status: NTSTATUS;
    let mut Buffer: PWCHAR;
    let mut End: PWCHAR;
    let mut DeviceType: PCSTR;
    let mut GenericType: PCSTR;
    let mut CharCount: size_t;
    let mut Remaining: size_t;

    PAGED_CODE!();

    match (*IoStack).Parameters.QueryId.IdType as u32 {
        BusQueryDeviceID => {
            DeviceType = AtaTypeCodeToName(DevExt, GetDeviceType);

            CharCount = (core::mem::size_of::<[u8; 5]>() - 1)
                + strlen(DeviceType)
                + (ATAPORT_FN_FIELD) as _
                + (ATAPORT_RN_FIELD) as _
                + core::mem::size_of_val(&ANSI_NULL);

            Buffer = (ExAllocatePoolUninitialized(
                PagedPool,
                (CharCount * core::mem::size_of::<WCHAR>()) as _,
                ATAPORT_TAG,
            )).cast();
            if Buffer.is_null() {
                return STATUS_INSUFFICIENT_RESOURCES;
            }

            AtaCopyIdStringSafe(
                LocalBuffer.as_mut_ptr(),
                (*DevExt).FriendlyName.as_mut_ptr() as PUCHAR,
                ATAPORT_FN_FIELD,
                '_' as CHAR,
            );

            AtaCopyIdStringSafe(
                LocalBuffer.as_mut_ptr().add((ATAPORT_FN_FIELD) as _),
                (*DevExt).RevisionNumber.as_mut_ptr() as PUCHAR,
                ATAPORT_RN_FIELD,
                '_' as CHAR,
            );

            Status = RtlStringCchPrintfExW!(
                Buffer,
                CharCount,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                0,
                RUSTCONV_WSTR_341441D4E8BF.as_ptr(),
                DeviceType,
                LocalBuffer.as_mut_ptr(),
            );

            INFO!("DeviceID: '%S'\n", Buffer);
        }

        BusQueryHardwareIDs => {
            let mut IdStart: PWCHAR;

            DBG_UNREFERENCED_LOCAL_VARIABLE(IdStart);

            DeviceType = AtaTypeCodeToName(DevExt, GetDeviceType);
            GenericType = AtaTypeCodeToName(DevExt, GetGenericType);

            CharCount = strlen(DeviceType) * 2
                + strlen(GenericType)
                + (core::mem::size_of::<[u8; 5]>() - 1) * 3
                + (ATAPORT_FN_FIELD * 4) as _
                + (ATAPORT_RN_FIELD * 3) as _
                + 5 * core::mem::size_of_val(&ANSI_NULL)
                + core::mem::size_of_val(&ANSI_NULL);

            Buffer = (ExAllocatePoolUninitialized(
                PagedPool,
                (CharCount * core::mem::size_of::<WCHAR>()) as _,
                ATAPORT_TAG,
            )).cast();
            if Buffer.is_null() {
                return STATUS_INSUFFICIENT_RESOURCES;
            }

            AtaCopyIdStringSafe(
                LocalBuffer.as_mut_ptr(),
                (*DevExt).FriendlyName.as_mut_ptr() as PUCHAR,
                ATAPORT_FN_FIELD,
                '_' as CHAR,
            );

            AtaCopyIdStringSafe(
                LocalBuffer.as_mut_ptr().add((ATAPORT_FN_FIELD) as _),
                (*DevExt).RevisionNumber.as_mut_ptr() as PUCHAR,
                ATAPORT_RN_FIELD,
                '_' as CHAR,
            );

            INFO!("HardwareIDs:\n");

            Status = RtlStringCchPrintfExW!(
                Buffer,
                CharCount,
                &mut End,
                &mut Remaining,
                0,
                RUSTCONV_WSTR_341441D4E8BF.as_ptr(),
                DeviceType,
                LocalBuffer.as_mut_ptr(),
            );

            INFO!("  '%S'\n", Buffer);

            End = End.add(1);
            Remaining -= 1;

            IdStart = End;
            Status = RtlStringCchPrintfExW!(
                End,
                Remaining,
                &mut End,
                &mut Remaining,
                0,
                RUSTCONV_WSTR_245E26C7A284.as_ptr(),
                LocalBuffer.as_mut_ptr(),
            );

            INFO!("  '%S'\n", IdStart);

            End = End.add(1);
            Remaining -= 1;

            IdStart = End;
            Status = RtlStringCchPrintfExW!(
                End,
                Remaining,
                &mut End,
                &mut Remaining,
                0,
                RUSTCONV_WSTR_FA5668E0AE70.as_ptr(),
                DeviceType,
                LocalBuffer.as_mut_ptr(),
            );

            INFO!("  '%S'\n", IdStart);

            End = End.add(1);
            Remaining -= 1;

            IdStart = End;
            Status = RtlStringCchPrintfExW!(
                End,
                Remaining,
                &mut End,
                &mut Remaining,
                0,
                RUSTCONV_WSTR_34675BEEA782.as_ptr(),
                LocalBuffer.as_mut_ptr(),
            );

            INFO!("  '%S'\n", IdStart);

            End = End.add(1);
            Remaining -= 1;

            IdStart = End;
            Status = RtlStringCchPrintfExW!(
                End,
                Remaining,
                &mut End,
                &mut Remaining,
                0,
                RUSTCONV_WSTR_8D44D4323B7D.as_ptr(),
                GenericType,
            );

            End = End.add(1);
            *End = UNICODE_NULL;

            INFO!("  '%S'\n", IdStart);
        }

        BusQueryCompatibleIDs => {
            GenericType = AtaTypeCodeToName(DevExt, GetGenericType);

            CharCount = strlen(GenericType)
                + 2 * core::mem::size_of_val(&ANSI_NULL);

            Buffer = (ExAllocatePoolUninitialized(
                PagedPool,
                (CharCount * core::mem::size_of::<WCHAR>()) as _,
                ATAPORT_TAG,
            )).cast();
            if Buffer.is_null() {
                return STATUS_INSUFFICIENT_RESOURCES;
            }

            Status = RtlStringCchPrintfExW!(
                Buffer,
                CharCount,
                &mut End,
                &mut Remaining,
                0,
                RUSTCONV_WSTR_8D44D4323B7D.as_ptr(),
                GenericType,
            );

            End = End.add(1);
            *End = UNICODE_NULL;

            INFO!("CompatibleIDs: '%S'\n", Buffer);
        }

        BusQueryInstanceID => {
            let mut AtaScsiAddress: ATA_SCSI_ADDRESS;

            CharCount = core::mem::size_of::<[u8; 9]>();

            Buffer = (ExAllocatePoolUninitialized(
                PagedPool,
                (CharCount * core::mem::size_of::<WCHAR>()) as _,
                ATAPORT_TAG,
            )).cast();
            if Buffer.is_null() {
                return STATUS_INSUFFICIENT_RESOURCES;
            }

            AtaScsiAddress = (*DevExt).Device.AtaScsiAddress;
            Status = RtlStringCchPrintfExW!(
                Buffer,
                CharCount,
                &mut End,
                &mut Remaining,
                0,
                RUSTCONV_WSTR_A1EB8D485789.as_ptr(),
                AtaScsiAddress.PathId as core::ffi::c_uint,
                AtaScsiAddress.TargetId as core::ffi::c_uint,
                AtaScsiAddress.Lun as core::ffi::c_uint,
            );

            INFO!("InstanceID: '%S'\n", Buffer);
        }

        _ => return (*Irp).IoStatus.__bindgen_anon_1.Status,
    }

    (*Irp).IoStatus.Information = Buffer as ULONG_PTR;
    return STATUS_SUCCESS;
}

unsafe extern "C" fn AtaPdoQueryDeviceText(
    mut DevExt: PATAPORT_DEVICE_EXTENSION,
    mut Irp: PIRP,
    mut IoStack: PIO_STACK_LOCATION,
) -> NTSTATUS {
    let mut Status: NTSTATUS = 0;
    let mut Buffer: PWCHAR = core::ptr::null_mut::<u16>();
    let mut CharCount: usize = 0;

    PAGED_CODE!();

    match (*IoStack).Parameters.QueryDeviceText.DeviceTextType as u32 {
        DeviceTextDescription => {
            CharCount = strlen(((*DevExt).FriendlyName).as_ptr() as *const i8)
                .wrapping_add(core::mem::size_of_val(&ANSI_NULL));

            Buffer = ExAllocatePoolUninitialized(
                PagedPool,
                (CharCount.wrapping_mul(core::mem::size_of::<u16>())) as _,
                ATAPORT_TAG,
            ) as PWCHAR;
            if Buffer.is_null() {
                return STATUS_INSUFFICIENT_RESOURCES;
            }

            Status = RtlStringCchPrintfExW!(
                Buffer,
                CharCount,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                0,
                RUSTCONV_WSTR_8D44D4323B7D.as_ptr(),
                ((*DevExt).FriendlyName).as_ptr() as *const i8,
            );

            INFO!(b"TextDescription: '%S'\n\0".as_ptr() as PCSTR, Buffer);
        }
        DeviceTextLocationInformation => {
            let mut AtaScsiAddress: ATA_SCSI_ADDRESS = { let mut __lit: ATA_SCSI_ADDRESS = core::mem::zeroed(); __lit.__bindgen_anon_1.PathId = 0; __lit.__bindgen_anon_1.TargetId = 0; __lit.__bindgen_anon_1.Lun = 0; __lit };

            CharCount = core::mem::size_of::<[i8; 33]>();

            Buffer = ExAllocatePoolUninitialized(
                PagedPool,
                (CharCount.wrapping_mul(core::mem::size_of::<u16>())) as _,
                ATAPORT_TAG,
            ) as PWCHAR;
            if Buffer.is_null() {
                return STATUS_INSUFFICIENT_RESOURCES;
            }

            AtaScsiAddress = (*DevExt).Device.AtaScsiAddress;
            Status = RtlStringCchPrintfExW!(
                Buffer,
                CharCount,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                0,
                RUSTCONV_WSTR_0E21877A26E8.as_ptr(),
                AtaScsiAddress.PathId as i32,
                AtaScsiAddress.TargetId as i32,
                AtaScsiAddress.Lun as i32,
            );

            INFO!(b"TextLocationInformation: '%S'\n\0".as_ptr() as PCSTR, Buffer);
        }
        _ => return (*Irp).IoStatus.__bindgen_anon_1.Status,
    }

    (*Irp).IoStatus.Information = Buffer as ULONG_PTR;
    return STATUS_SUCCESS;
}

unsafe extern "C" fn AtaPdoPnp(
    DevExt: *mut PATAPORT_DEVICE_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let IoStack: PIO_STACK_LOCATION;

    PAGED_CODE!();

    INFO!(
        b"(%p, %p) Tid.%lu %s\n\0".as_ptr() as *const i8,
        (*DevExt).Common.Self,
        Irp,
        (*DevExt).Device.AtaScsiAddress.TargetId,
        GetIRPMinorFunctionString(IoGetCurrentIrpStackLocation(Irp).MinorFunction),
    );

    Status = IoAcquireRemoveLock(&mut (*(*DevExt)).Common.RemoveLock, (Irp).cast());
    if !NT_SUCCESS!(Status) {
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        IoCompleteRequest(Irp, (IO_NO_INCREMENT) as CCHAR);
        return Status;
    }

    IoStack = IoGetCurrentIrpStackLocation(Irp);

    match (*IoStack).MinorFunction {
        IRP_MN_START_DEVICE => {
            Status = AtaPdoStartDevice((DevExt).cast(), Irp);
        }
        IRP_MN_STOP_DEVICE => {
            Status = AtaPdoStopDevice((DevExt).cast(), Irp);
        }
        IRP_MN_REMOVE_DEVICE | IRP_MN_SURPRISE_REMOVAL => {
            return AtaPdoRemoveDevice(
                (DevExt).cast(),
                Irp,
                ((*IoStack).MinorFunction == IRP_MN_REMOVE_DEVICE) as BOOLEAN,
            );
        }
        IRP_MN_QUERY_STOP_DEVICE | IRP_MN_QUERY_REMOVE_DEVICE => {
            Status = AtaPdoQueryStopRemoveDevice((DevExt).cast(), Irp);
        }
        IRP_MN_CANCEL_REMOVE_DEVICE | IRP_MN_CANCEL_STOP_DEVICE => {
            Status = STATUS_SUCCESS;
        }
        IRP_MN_QUERY_CAPABILITIES => {
            Status = AtaPdoQueryCapabilities((DevExt).cast(), Irp, IoStack);
        }
        IRP_MN_QUERY_PNP_DEVICE_STATE => {
            Status = AtaPnpQueryPnpDeviceState(&mut (*(*DevExt)).Common, Irp);
        }
        IRP_MN_QUERY_ID => {
            Status = AtaPdoQueryId((DevExt).cast(), Irp, IoStack);
        }
        IRP_MN_QUERY_DEVICE_TEXT => {
            Status = AtaPdoQueryDeviceText((DevExt).cast(), Irp, IoStack);
        }
        IRP_MN_DEVICE_USAGE_NOTIFICATION => {
            Status = AtaPnpQueryDeviceUsageNotification(&mut (*(*DevExt)).Common, Irp);
        }
        IRP_MN_QUERY_DEVICE_RELATIONS => {
            if (*IoStack).Parameters.QueryDeviceRelations.Type == (TargetDeviceRelation as DEVICE_RELATION_TYPE) {
                Status = AtaPdoQueryTargetDeviceRelations((DevExt).cast(), Irp);
            } else {
                Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
            }
        }
        _ => {
            Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
        }
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    IoCompleteRequest(Irp, (IO_NO_INCREMENT) as CCHAR);

    IoReleaseRemoveLock(&mut (*(*DevExt)).Common.RemoveLock, (Irp).cast());

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn AtaDispatchPnp(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    PAGED_CODE!();

    if IS_FDO!((*DeviceObject).DeviceExtension) != false {
        return AtaFdoPnp(((*DeviceObject).DeviceExtension).cast(), Irp);
    } else {
        return AtaPdoPnp(((*DeviceObject).DeviceExtension).cast(), Irp);
    }
}
#[no_mangle]
pub unsafe extern "C" fn AtaPdoFreeDevice(DevExt: *mut PATAPORT_DEVICE_EXTENSION) {
    PAGED_CODE!();

    if !(*(*DevExt)).Device.PowerIdleCounter.is_null() {
        PoRegisterDeviceForIdleDetection(
            (*(*DevExt)).Common.Self_,
            0,
            0,
            PowerDeviceD3,
        );
    }

    if !(*(*DevExt)).GtfDataBuffer.is_null() {
        ExFreePoolWithTag((*(*DevExt)).GtfDataBuffer, ATAPORT_TAG);
        (*(*DevExt)).GtfDataBuffer = core::ptr::null_mut();
    }

    if !(*(*DevExt)).Device.Requests.is_null() {
        ExFreePoolWithTag(((*(*DevExt)).Device.Requests).cast(), ATAPORT_TAG);
    }

    IoDeleteDevice((*(*DevExt)).Common.Self_);
}
#[no_mangle]
pub unsafe extern "C" fn AtaPdoCreateDevice(
    ChanExt: *mut PATAPORT_CHANNEL_EXTENSION,
    AtaScsiAddress: ATA_SCSI_ADDRESS,
) -> *mut PATAPORT_DEVICE_EXTENSION {
    let PortData: *mut PATAPORT_PORT_DATA = &mut (*(*ChanExt)).PortData as *mut PATAPORT_PORT_DATA;
    let mut Status: NTSTATUS;
    let mut DeviceName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut DevExt: *mut PATAPORT_DEVICE_EXTENSION;
    let mut Pdo: PDEVICE_OBJECT;
    let mut DeviceNameBuffer: [u16; 36] = [0; 36];
    let mut i: u32;
    static mut PdoFormat: *const u16 = RUSTCONV_WSTR_6BE0C8F6E502.as_ptr();
    static mut AtapPdoNumber: u32 = 0;

    Status = RtlStringCbPrintfW!(
        DeviceNameBuffer.as_mut_ptr(),
        core::mem::size_of::<[u16; 36]>() as u32,
        PdoFormat,
        (*(*ChanExt)).DeviceObjectNumber,
        AtaScsiAddress.TargetId as i32,
        AtaScsiAddress.Lun as i32,
        AtapPdoNumber,
    );
    AtapPdoNumber = AtapPdoNumber.wrapping_add(1);

    RtlInitUnicodeString(&mut DeviceName, DeviceNameBuffer.as_mut_ptr());

    Status = IoCreateDevice(
        (*(*(*ChanExt)).Common.Self_).DriverObject,
        core::mem::size_of::<PATAPORT_DEVICE_EXTENSION>() as u32,
        &mut DeviceName,
        FILE_DEVICE_MASS_STORAGE,
        FILE_DEVICE_SECURE_OPEN,
        FALSE,
        &mut Pdo,
    );
    if !NT_SUCCESS!(Status) {
        return core::ptr::null_mut();
    }

    INFO!(
        b"Created device object %p '%wZ'\n\0".as_ptr() as *const i8,
        Pdo,
        &mut DeviceName,
    );

    (*Pdo).AlignmentRequirement = (*(*(*ChanExt)).Common.Self_).AlignmentRequirement;
    (*Pdo).AlignmentRequirement = max((*Pdo).AlignmentRequirement, ATA_MIN_BUFFER_ALIGNMENT);

    (*Pdo).Flags |= DO_DIRECT_IO;

    DevExt = (*Pdo).DeviceExtension as *mut PATAPORT_DEVICE_EXTENSION;

    RtlZeroMemory!(
        DevExt as *mut core::ffi::c_void,
        core::mem::size_of::<PATAPORT_DEVICE_EXTENSION>(),
    );
    AtaPnpInitializeCommonExtension(
        &mut (*(*DevExt)).Common,
        Pdo,
        (*(*ChanExt)).Common.Flags & !DO_IS_FDO,
    );
    (*(*DevExt)).Common.__bindgen_anon_1.FdoExt = ChanExt as *mut core::ffi::c_void;
    (*(*DevExt)).TransferModeAllowedMask = MAXULONG;
    (*(*DevExt)).Device.__bindgen_anon_1.SectorSize = ATA_MIN_SECTOR_SIZE;
    (*(*DevExt)).Device.PortData = (PortData).cast();
    (*(*DevExt)).Device.LocalBuffer = (*(*PortData)).LocalBuffer;

    (*(*DevExt)).Device.AtaScsiAddress = AtaScsiAddress;
    (*(*DevExt)).Device.__bindgen_anon_1.TransportFlags = AtaScsiAddress.TargetId as u32;
    (*(*DevExt)).Device.__bindgen_anon_1.DeviceSelect = (IDE_DRIVE_SELECT | (AtaScsiAddress.Lun as i32) as u32) as u8;

    if !((*(*PortData)).PortFlags & PORT_FLAG_IS_AHCI != 0) {
        (*(*DevExt)).Device.__bindgen_anon_1.DeviceSelect = ((*(*DevExt)).Device.__bindgen_anon_1.DeviceSelect as i32
            | ((AtaScsiAddress.TargetId as i32 & 1) << 4))
            as u8;
    }

    (*(*DevExt)).Device.DeviceFlags = DEVICE_UNINITIALIZED;

    if (*(*PortData)).PortFlags & PORT_FLAG_IS_EXTERNAL != 0 {
        (*(*DevExt)).Device.DeviceFlags |= DEVICE_IS_PDO_REMOVABLE;
    }

    if (*(*PortData)).PortFlags & PORT_FLAG_PIO_VIA_DMA != 0 {
        (*(*DevExt)).Device.DeviceFlags |= DEVICE_PIO_VIA_DMA;
    }

    if (*(*PortData)).PortFlags & PORT_FLAG_PIO_FOR_LBA48_XFER != 0 {
        (*(*DevExt)).Device.DeviceFlags |= DEVICE_PIO_FOR_LBA48_XFER;
    }

    (*(*DevExt)).Device.DeviceFlags |= DEVICE_PIO_ONLY;

    (*(*DevExt)).Device.FreeRequestsBitmap = NUM_TO_BITMAP!(((*(*PortData)).QueueDepth) as _);
    (*(*DevExt)).Device.MaxRequestsBitmap = (*(*DevExt)).Device.FreeRequestsBitmap;

    (*(*DevExt)).Device.Requests = ExAllocatePoolZero(
        NonPagedPool,
        ((core::mem::size_of::<ATA_DEVICE_REQUEST>() as u32) * (*(*PortData)).QueueDepth) as _,
        ATAPORT_TAG,
    ) as *mut ATA_DEVICE_REQUEST;
    if (*(*DevExt)).Device.Requests.is_null() {
        AtaPdoFreeDevice(DevExt);
        return core::ptr::null_mut();
    }

    i = 0;
    while i < (*(*PortData)).QueueDepth {
        let Request: *mut ATA_DEVICE_REQUEST = (*(*DevExt)).Device.Requests.add(i as usize);

        (*Request).Device = &mut (*(*DevExt)).Device as *mut _ as *mut ATA_IO_CONTEXT_COMMON;
        (*Request).Signature = ATA_DEVICE_REQUEST_SIGNATURE;
        i = i.wrapping_add(1);
    }

    KeInitializeSpinLock(&mut (*(*DevExt)).Device.QueueLock);
    InitializeListHead(&mut (*(*DevExt)).Device.DeviceQueueList);
    InitializeListHead(&mut (*(*DevExt)).PowerIrpQueueList);
    KeInitializeEvent(
        &mut (*(*DevExt)).Device.QueueStoppedEvent,
        _EVENT_TYPE_NotificationEvent,
        FALSE,
    );
    KeInitializeEvent(
        &mut (*(*DevExt)).Worker.EnumerationEvent,
        _EVENT_TYPE_NotificationEvent,
        FALSE,
    );
    KeInitializeEvent(
        &mut (*(*DevExt)).Worker.ConfigureEvent,
        _EVENT_TYPE_NotificationEvent,
        FALSE,
    );

    return DevExt;
}
