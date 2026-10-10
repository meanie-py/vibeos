use crate::prelude::*;
use ntkrnl_sys::ddk::{DEVICE_RELATIONS, _KWAIT_REASON_Executive, KeWaitForSingleObject, LARGE_INTEGER, PDEVICE_RELATIONS, PIRP};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const DD_ATA_REG_ATA_DEVICE_TYPE: [::core::ffi::c_ushort; 11] = unsafe {
    ::core::mem::transmute::<[u8; 22], [::core::ffi::c_ushort; 11]>(
        *b"D\0e\0v\0i\0c\0e\0T\0y\0p\0e\0\0\0",
    )
};

pub const DD_ATA_REG_SCSI_DEVICE_TYPE: [::core::ffi::c_ushort; 15] = unsafe {
    ::core::mem::transmute::<[u8; 30], [::core::ffi::c_ushort; 15]>(
        *b"S\0c\0s\0i\0D\0e\0v\0i\0c\0e\0T\0y\0p\0e\0\0\0",
    )
};

pub const DD_ATA_REG_XFER_MODE_ALLOWED: [::core::ffi::c_ushort; 22] = unsafe {
    ::core::mem::transmute::<[u8; 44], [::core::ffi::c_ushort; 22]>(
        *b"U\0s\0e\0r\0T\0i\0m\0i\0n\0g\0M\0o\0d\0e\0A\0l\0l\0o\0w\0e\0d\0\0\0",
    )
};

pub const DD_ATA_REG_XFER_MODE_SUPPORTED: [::core::ffi::c_ushort; 26] = unsafe {
    ::core::mem::transmute::<[u8; 52], [::core::ffi::c_ushort; 26]>(
        *b"D\0e\0v\0i\0c\0e\0T\0i\0m\0i\0n\0g\0M\0o\0d\0e\0S\0u\0p\0p\0o\0r\0t\0e\0d\0\0\0",
    )
};

pub const DD_ATA_REG_XFER_MODE_SELECTED: [::core::ffi::c_ushort; 17] = unsafe {
    ::core::mem::transmute::<[u8; 34], [::core::ffi::c_ushort; 17]>(
        *b"D\0e\0v\0i\0c\0e\0T\0i\0m\0i\0n\0g\0M\0o\0d\0e\0\0\0",
    )
};
#[no_mangle]
pub unsafe extern "C" fn AtaCopyIdStringUnsafe(
    Destination: *mut u8,
    Source: *mut u8,
    Length: u32,
) -> *mut u8 {
    let mut i: u32;

    ASSUME!(
        (Length >= core::mem::size_of::<USHORT>() as u32)
            && (Length % core::mem::size_of::<USHORT>() as u32)
                == 0,
    );

    i = 0;
    while i < Length {
        *Destination.offset(i as isize) = *Source.offset((i + 1) as isize);
        *Destination.offset((i + 1) as isize) = *Source.offset(i as isize);
        i += core::mem::size_of::<USHORT>() as u32;
    }

    return Destination.offset((i - 1) as isize);
}
#[no_mangle]
pub unsafe extern "C" fn AtaCopyIdStringSafe(
    mut Destination: *mut i8,
    mut Source: *mut u8,
    mut MaxLength: u32,
    mut DefaultCharacter: i8,
) -> *mut i8 {
    let mut Dest: *mut i8 = Destination;

    while MaxLength != 0 {
        let Char: u8 = *Source;

        if Char > ' ' as u8 && Char <= '~' as u8 && Char != ',' as u8 {
            *Dest = Char as i8;
        } else {
            *Dest = DefaultCharacter;
        }

        Source = Source.offset(1);
        Dest = Dest.offset(1);
        MaxLength = MaxLength.wrapping_sub(1);
    }

    return Dest;
}
#[no_mangle]
pub unsafe extern "C" fn AtaSwapIdString(
    Buffer: *mut core::ffi::c_void,
    WordCount: u32,
) {
    let mut Word: *mut u16 = Buffer as *mut u16;

    let mut count = WordCount;
    while count != 0 {
        count = count.wrapping_sub(1);
        *Word = RtlUshortByteSwap(*Word);
        Word = Word.add(1);
    }
}

unsafe extern "C" fn AtaTrimIdString(mut Start: PCHAR, mut End: PCHAR) -> PCHAR {
    let mut Current: PCHAR = End.offset(-(1 as isize));

    if (KeGetCurrentIrql() as ::core::ffi::c_int) as u32 > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as PCSTR,
            KeGetCurrentIrql() as ::core::ffi::c_int,
        );
        if 0 as ::core::ffi::c_int == 0 {
            RtlAssert(
                b"0\0" as *const u8 as *const ::core::ffi::c_char as PVOID,
                b"drivers/storage/ide/atapi/enum.c\0"
                    as *const u8 as *const ::core::ffi::c_char as PVOID,
                89 as ULONG,
                ::core::ptr::null_mut::<CHAR>(),
            );
        } else {
        };
    }

    while Current >= Start && *Current as ::core::ffi::c_int == ' ' as i32 {
        Current = Current.offset(-1);
    }

    *Current.offset(1 as isize) = ANSI_NULL;

    return Current.offset(1 as isize);
}

unsafe extern "C" fn AtaPdoFillIdentificationStrings(mut DevExt: PATAPORT_DEVICE_EXTENSION) {
    let mut IdentifyData: PIDENTIFY_DEVICE_DATA = &raw mut (*DevExt).__bindgen_anon_1.IdentifyDeviceData;
    let mut End: PCHAR = core::ptr::null_mut();
    let mut i: ULONG = 0;
    let mut Status: NTSTATUS = 0;
    let mut Remaining: size_t = 0;

    PAGED_CODE!();

    if IS_ATAPI!(&raw mut (*DevExt).Device) != false {
        End = AtaCopyIdStringSafe(
            &raw mut (*DevExt).FriendlyName as PCHAR,
            &raw mut (*DevExt).InquiryData.VendorId as PUCHAR,
            RTL_FIELD_SIZE!(INQUIRYDATA, VendorId) as ULONG,
            ' ' as i32 as CHAR,
        );
        End = AtaTrimIdString(&raw mut (*DevExt).FriendlyName as PCHAR, End);
        let fresh = End;
        End = End.offset(1);
        *fresh = ' ' as i32 as CHAR;
        End = AtaCopyIdStringSafe(
            End,
            &raw mut (*DevExt).InquiryData.ProductId as PUCHAR,
            RTL_FIELD_SIZE!(INQUIRYDATA, ProductId) as ULONG,
            ' ' as i32 as CHAR,
        );
        AtaTrimIdString(&raw mut (*DevExt).FriendlyName as PCHAR, End);

        End = AtaCopyIdStringSafe(
            &raw mut (*DevExt).RevisionNumber as PCHAR,
            &raw mut (*DevExt).InquiryData.ProductRevisionLevel as PUCHAR,
            RTL_FIELD_SIZE!(INQUIRYDATA, ProductRevisionLevel) as ULONG,
            ' ' as i32 as CHAR,
        );
        AtaTrimIdString(&raw mut (*DevExt).RevisionNumber as PCHAR, End);
    } else {
        End = AtaCopyIdStringSafe(
            &raw mut (*DevExt).FriendlyName as PCHAR,
            &raw mut (*IdentifyData).ModelNumber as PUCHAR,
            ATAPORT_FN_FIELD,
            ' ' as i32 as CHAR,
        );
        AtaSwapIdString(
            &raw mut (*DevExt).FriendlyName as *mut CHAR as PVOID,
            (ATAPORT_FN_FIELD / 2) as ULONG,
        );
        AtaTrimIdString(&raw mut (*DevExt).FriendlyName as PCHAR, End);

        End = AtaCopyIdStringSafe(
            &raw mut (*DevExt).RevisionNumber as PCHAR,
            &raw mut (*IdentifyData).FirmwareRevision as PUCHAR,
            ATAPORT_RN_FIELD,
            ' ' as i32 as CHAR,
        );
        AtaSwapIdString(
            &raw mut (*DevExt).RevisionNumber as *mut CHAR as PVOID,
            (ATAPORT_RN_FIELD / 2) as ULONG,
        );
        AtaTrimIdString(&raw mut (*DevExt).RevisionNumber as PCHAR, End);
    }

    End = &raw mut (*DevExt).SerialNumber as *mut CHAR as PCHAR;
    Remaining = core::mem::size_of::<[CHAR; 41]>() as size_t;

    i = 0;
    while (i as usize) < core::mem::size_of::<[UCHAR; 20]>() as usize {
        Status = RtlStringCchPrintfExA!(
            End,
            Remaining,
            &raw mut End,
            &raw mut Remaining,
            0,
            b"%2x\0" as *const u8 as *const i8,
            (*IdentifyData).SerialNumber[i as usize] as i32,
        );
        i = i.wrapping_add(1);
    }

    INFO!(b"FriendlyName: '%s'\n\0" as *const u8 as *const i8, (*DevExt).FriendlyName);
    INFO!(b"RevisionNumber: '%s'\n\0" as *const u8 as *const i8, (*DevExt).RevisionNumber);
    INFO!(b"SerialNumber: '%s'\n\0" as *const u8 as *const i8, (*DevExt).SerialNumber);
}
#[no_mangle]
pub unsafe extern "C" fn AtaDeviceSetAddressingMode(DevExt: PATAPORT_DEVICE_EXTENSION) {
    let IdentifyData: PIDENTIFY_DEVICE_DATA = &raw mut (*DevExt).__bindgen_anon_1.IdentifyDeviceData;
    let mut TotalSectors: ULONG64;

    (*DevExt).Device.DeviceFlags &= !(DEVICE_LBA_MODE | DEVICE_LBA48 | DEVICE_HAS_FUA);

    if AtaDevHasLbaTranslation(IdentifyData) != 0 {
        (*DevExt).Device.DeviceFlags |= DEVICE_LBA_MODE;

        if AtaDevHas48BitAddressFeature(IdentifyData) != 0 {
            TotalSectors = AtaDevUserAddressableSectors48Bit(IdentifyData);

            (*DevExt).Device.DeviceFlags |= DEVICE_LBA48;

            if AtaDevHasForceUnitAccessCommands(IdentifyData) != 0 {
                (*DevExt).Device.DeviceFlags |= DEVICE_HAS_FUA;
            }
        } else {
            TotalSectors = (AtaDevUserAddressableSectors28Bit(IdentifyData)) as _;
        }
    } else {
        let mut Cylinders: USHORT = core::mem::zeroed();
        let mut Heads: USHORT = core::mem::zeroed();
        let mut SectorsPerTrack: USHORT = core::mem::zeroed();

        if AtaDevIsCurrentGeometryValid(IdentifyData) != 0 {
            AtaDevCurrentChsTranslation(IdentifyData, &raw mut Cylinders, &raw mut Heads, &raw mut SectorsPerTrack);
        } else {
            AtaDevDefaultChsTranslation(IdentifyData, &raw mut Cylinders, &raw mut Heads, &raw mut SectorsPerTrack);
        }
        (*DevExt).Device.Cylinders = Cylinders;
        (*DevExt).Device.Heads = Heads;
        (*DevExt).Device.SectorsPerTrack = SectorsPerTrack;

        TotalSectors = (Cylinders as ULONG64)
            .wrapping_mul(Heads as ULONG64)
            .wrapping_mul(SectorsPerTrack as ULONG64);
    }

    if TotalSectors == 0 {
        ERR!(b"Unknown geometry\n\0".as_ptr() as *const i8);

        TotalSectors = 1;

        (*DevExt).Device.SectorsPerTrack = 1;
        (*DevExt).Device.Heads = 1;
    }

    (*DevExt).Device.TotalSectors = TotalSectors;

    (*DevExt).Device.__bindgen_anon_1.SectorSize = AtaDevBytesPerLogicalSector(IdentifyData);
    (*DevExt).Device.__bindgen_anon_1.SectorSize = max((*DevExt).Device.__bindgen_anon_1.SectorSize, ATA_MIN_SECTOR_SIZE);

    INFO!(
        b"Total sectors %I64u of size %lu, CHS %u:%u:%u\n\0".as_ptr() as *const i8,
        (*DevExt).Device.TotalSectors,
        (*DevExt).Device.SectorSize,
        (*DevExt).Device.Cylinders,
        (*DevExt).Device.Heads,
        (*DevExt).Device.SectorsPerTrack,
    );
}

unsafe extern "C" fn AtaDeviceEnableQueuedCommands(mut DevExt: PATAPORT_DEVICE_EXTENSION) {
    let mut DeviceQueueDepth: ULONG = 0;
    PAGED_CODE!();
    DeviceQueueDepth = AtaDevQueueDepth(&raw mut (*DevExt).__bindgen_anon_1.IdentifyDeviceData);
    if DeviceQueueDepth == 0 {
        return;
    }
    DeviceQueueDepth = min(DeviceQueueDepth, (*(*DevExt).Device.PortData).QueueDepth);
    (*DevExt).Device.__bindgen_anon_1.TransportFlags |= DeviceQueueDepth << DEVICE_QUEUE_DEPTH_SHIFT;
    (*DevExt).Device.DeviceFlags |= DEVICE_NCQ;
    INFO!(
        b"NCQ enabled, queue depth %lu/%lu\n\0" as *const u8 as *const i8,
        DeviceQueueDepth,
        (*(*DevExt).Device.PortData).QueueDepth,
    );
}

unsafe extern "C" fn AtaDeviceIsSuperFloppy(DevExt: PATAPORT_DEVICE_EXTENSION) -> BOOLEAN {
    PAGED_CODE!();

    if ((*DevExt).InquiryData.DeviceType()) as u32 == DIRECT_ACCESS_DEVICE {
        return ((!strstr(
            (*DevExt).FriendlyName.as_mut_ptr(),
            b" LS-120\0".as_ptr() as *const i8,
        )
        .is_null()
            || !strstr(
                (*DevExt).FriendlyName.as_mut_ptr(),
                b" LS-240\0".as_ptr() as *const i8,
            )
            .is_null()) as i32) as u8;
    }

    return FALSE;
}

unsafe extern "C" fn AtaPdoInit(
    mut ChanExt: PATAPORT_CHANNEL_EXTENSION,
    mut DevExt: PATAPORT_DEVICE_EXTENSION,
) {
    PAGED_CODE!();

    AtaPdoFillIdentificationStrings(DevExt);

    if IS_ATAPI!(&raw mut (*DevExt).Device) != false {
        if AtaDeviceIsSuperFloppy(DevExt) != 0 {
            (*DevExt).Device.DeviceFlags |= DEVICE_IS_SUPER_FLOPPY;
        }
        if AtaDevIsTape(&raw mut (*DevExt).__bindgen_anon_1.IdentifyPacketData) != 0 {
            INFO!(
                b"Tape drive detected '%s'\n\0" as *const u8 as *const i8,
                (*DevExt).FriendlyName,
            );
        }
    } else {
        if (*ChanExt).PortData.PortFlags & PORT_FLAG_NCQ != 0 {
            AtaDeviceEnableQueuedCommands(DevExt);
        }
        AtaDeviceSetAddressingMode(DevExt);
        AtaCreateStandardInquiryData(DevExt);
    }

    if (*DevExt).Device.AtaScsiAddress.__bindgen_anon_1.Lun as i32 == 0 as i32 {
        AtaSetRegistryKey(
            ChanExt,
            (*DevExt).Device.AtaScsiAddress.__bindgen_anon_1.TargetId,
            (DD_ATA_REG_ATA_DEVICE_TYPE).as_ptr(),
            (*DevExt).DeviceType,
        );
        AtaSetRegistryKey(
            ChanExt,
            (*DevExt).Device.AtaScsiAddress.__bindgen_anon_1.TargetId,
            (DD_ATA_REG_SCSI_DEVICE_TYPE).as_ptr(),
            (*DevExt).InquiryData.DeviceType() as u32,
        );
        AtaGetRegistryKey(
            ChanExt,
            (*DevExt).Device.AtaScsiAddress.__bindgen_anon_1.TargetId,
            (DD_ATA_REG_XFER_MODE_ALLOWED).as_ptr(),
            &raw mut (*DevExt).TransferModeUserAllowedMask,
            MAXULONG,
        );
    }

    AtaReqFreezeQueue(DevExt, QUEUE_FLAG_FROZEN_PNP);

    if AtaDevIsSsd(&raw mut (*DevExt).__bindgen_anon_1.IdentifyDeviceData) == 0 {
        (*(*DevExt).Common.Self_).Flags |= DO_POWER_INRUSH;
    }
    if (*DevExt).InquiryData.RemovableMedia() != 0 {
        (*(*DevExt).Common.Self_).Characteristics |= FILE_REMOVABLE_MEDIA;
    }
    (*(*DevExt).Common.Self_).Flags &= !DO_DEVICE_INITIALIZING;

    (*DevExt).Device.DeviceFlags &= !DEVICE_UNINITIALIZED;
}

unsafe extern "C" fn AtaFdoQueryDeviceCount(mut PortData: PATAPORT_PORT_DATA) -> ULONG {
    PAGED_CODE!();
    KeClearEvent(&raw mut (*PortData).Worker.EnumerationEvent);
    AtaDeviceQueueEvent(
        PortData,
        core::ptr::null_mut(),
        ACTION_ENUM_PORT,
    );
    KeWaitForSingleObject(
        (&raw mut (*PortData).Worker.EnumerationEvent).cast(),
        _KWAIT_REASON_Executive,
        (0i32) as KPROCESSOR_MODE,
        FALSE,
        core::ptr::null_mut(),
    );
    return (*PortData).Worker.DeviceCount;
}

unsafe extern "C" fn AtaFdoQueryDeviceStatus(
    PortData: PATAPORT_PORT_DATA,
    DevExt: PATAPORT_DEVICE_EXTENSION,
    Action: ATA_PORT_ACTION,
) -> ATA_DEVICE_STATUS {
    PAGED_CODE!();

    KeClearEvent(&raw mut (*DevExt).Worker.EnumerationEvent);
    AtaDeviceQueueEvent(PortData, DevExt, Action);
    KeWaitForSingleObject(
        (&raw mut (*DevExt).Worker.EnumerationEvent).cast(),
        _KWAIT_REASON_Executive,
        (0i32) as KPROCESSOR_MODE,
        FALSE,
        core::ptr::null_mut(),
    );
    return (*DevExt).Worker.EnumStatus as ATA_DEVICE_STATUS;
}

unsafe extern "C" fn AtaFdoEnumeratePort(mut ChanExt: PATAPORT_CHANNEL_EXTENSION) {
    let mut PortData: PATAPORT_PORT_DATA = &raw mut (*ChanExt).PortData;
    let mut i: ULONG = 0;
    let mut DeviceCount: ULONG = 0;
    let mut DiscoveredNewDevice: BOOLEAN = FALSE;
    PAGED_CODE!();
    DeviceCount = AtaFdoQueryDeviceCount(PortData);
    i = 0;
    while i < DeviceCount {
        let mut DevExt: PATAPORT_DEVICE_EXTENSION = core::ptr::null_mut();
        let mut AtaScsiAddress: ATA_SCSI_ADDRESS = { let mut __lit: ATA_SCSI_ADDRESS = core::mem::zeroed(); __lit.__bindgen_anon_1.Lun = 0; __lit.__bindgen_anon_1.TargetId = 0; __lit.__bindgen_anon_1.PathId = 0; __lit.__bindgen_anon_1.IsValid = 0; __lit };
        let mut EnumStatus: ATA_DEVICE_STATUS = 0;
        AtaScsiAddress = AtaMarshallScsiAddress((*PortData).PortNumber, i, 0);
        DevExt = (AtaFdoFindDeviceByPath((ChanExt).cast(), AtaScsiAddress, core::ptr::null_mut())).cast();
        if DevExt.is_null() {
            DevExt = (AtaPdoCreateDevice((ChanExt).cast(), AtaScsiAddress)).cast();
            if DevExt.is_null() {
                ERR!(b"Failed to allocate PDO extension\n\0".as_ptr() as *const i8);
                // C `continue` in a `for` still runs `++i`; without it this retried index i forever.
                i = i.wrapping_add(1);
                continue;
            }
            AtaGetRegistryKey(
                ChanExt,
                (i) as UCHAR,
                (DD_ATA_REG_ATA_DEVICE_TYPE).as_ptr(),
                &raw mut (*DevExt).DeviceType,
                DEV_NONE,
            );
            core::ptr::write_volatile(
                &mut (*PortData).Worker.EnumDevExt as *mut PATAPORT_DEVICE_EXTENSION,
                DevExt,
            );
            EnumStatus = AtaFdoQueryDeviceStatus(PortData, DevExt, ACTION_ENUM_DEVICE_NEW);
        } else {
            EnumStatus = AtaFdoQueryDeviceStatus(PortData, DevExt, ACTION_ENUM_DEVICE);
        }
        match EnumStatus {
            DEV_STATUS_FAILED | DEV_STATUS_NO_DEVICE => {
                if (*DevExt).Device.DeviceFlags & DEVICE_UNINITIALIZED != 0 {
                    AtaPdoFreeDevice((DevExt).cast());
                }
                i = i.wrapping_add(1);
                continue;
            }
            DEV_STATUS_SAME_DEVICE => {
                (*DevExt).NotPresent = FALSE;
                i = i.wrapping_add(1);
                continue;
            }
            DEV_STATUS_NEW_DEVICE => {
                if (*DevExt).Device.DeviceFlags & DEVICE_UNINITIALIZED == 0 {
                    i = i.wrapping_sub(1);
                    i = i.wrapping_add(1);
                    continue;
                }
            }
            _ => {
                ASSERT!(FALSE);
                UNREACHABLE!();
            }
        }
        AtaPdoInit(ChanExt, DevExt);
        AtaFdoDeviceListInsert(ChanExt, DevExt, TRUE);
        DiscoveredNewDevice = TRUE;
        i = i.wrapping_add(1);
    }
    if DiscoveredNewDevice != 0 {
        AtaDeviceQueueEvent(PortData, core::ptr::null_mut(), ACTION_PORT_TIMING);
    }
}

unsafe extern "C" fn AtaFdoInitializeDeviceRelations(
    mut ChanExt: PATAPORT_CHANNEL_EXTENSION,
    mut DeviceRelations: PDEVICE_RELATIONS,
) {
    let mut DevExt: PATAPORT_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut AtaScsiAddress: ATA_SCSI_ADDRESS = ATA_SCSI_ADDRESS {
        AsULONG: 0,
    };

    PAGED_CODE!();

    (*DeviceRelations).Count = 0;

    AtaScsiAddress.AsULONG = 0;
    loop {
        DevExt = AtaFdoFindNextDeviceByPath(ChanExt, &raw mut AtaScsiAddress, TRUE, core::ptr::null_mut());
        if DevExt.is_null() {
            break;
        }

        if (*DevExt).NotPresent != 0 {
            if AtaScsiAddress.Lun == 0 {
                AtaSetRegistryKey(
                    ChanExt,
                    AtaScsiAddress.TargetId,
                    (DD_ATA_REG_ATA_DEVICE_TYPE).as_ptr(),
                    DEV_NONE,
                );
            }

            (*DevExt).ReportedMissing = TRUE;
            continue;
        }

        if AtaScsiAddress.Lun == 0 {
            AtaSetRegistryKey(
                ChanExt,
                AtaScsiAddress.TargetId,
                (DD_ATA_REG_XFER_MODE_SUPPORTED).as_ptr(),
                (*DevExt).TransferModeSupportedBitmap,
            );

            AtaSetRegistryKey(
                ChanExt,
                AtaScsiAddress.TargetId,
                (DD_ATA_REG_XFER_MODE_SELECTED).as_ptr(),
                (*DevExt).TransferModeSelectedBitmap,
            );
        }

        let fresh = (*DeviceRelations).Count;
        (*DeviceRelations).Count = (*DeviceRelations).Count.wrapping_add(1);
        *((*DeviceRelations).Objects.as_mut_ptr().offset(fresh as isize)) = (*DevExt).Common.Self_;
        ObReferenceObject(((*DevExt).Common.Self_).cast());
    }
}
#[no_mangle]
pub unsafe extern "C" fn AtaFdoQueryBusRelations(
    mut ChanExt: PATAPORT_CHANNEL_EXTENSION,
    mut Irp: PIRP,
) -> NTSTATUS {
    let mut DeviceRelations: PDEVICE_RELATIONS = core::ptr::null_mut();
    let mut Size: u32 = 0;
    let mut PdoCount: u32 = 0;
    let mut AtaScsiAddress: ATA_SCSI_ADDRESS = core::mem::zeroed();
    let mut TimeStart: LARGE_INTEGER = core::mem::zeroed();
    let mut TimeFinish: LARGE_INTEGER = core::mem::zeroed();
    let mut EnumTimeMs: u32 = 0;

    PAGED_CODE!();

    KeQuerySystemTime(core::ptr::addr_of_mut!(TimeStart));

    AtaScsiAddress.AsULONG = 0;
    loop {
        let mut DevExt: PATAPORT_DEVICE_EXTENSION = core::ptr::null_mut();
        DevExt = AtaFdoFindNextDeviceByPath(
            ChanExt,
            core::ptr::addr_of_mut!(AtaScsiAddress),
            TRUE,
            core::ptr::null_mut(),
        );
        if DevExt.is_null() {
            break;
        }
        (*DevExt).NotPresent = TRUE;
        if (*DevExt).Device.AtaScsiAddress.__bindgen_anon_1.Lun == 0 {
            AtaGetRegistryKey(
                ChanExt,
                (*DevExt).Device.AtaScsiAddress.__bindgen_anon_1.TargetId,
                (DD_ATA_REG_XFER_MODE_ALLOWED).as_ptr(),
                core::ptr::addr_of_mut!((*DevExt).TransferModeUserAllowedMask),
                MAXULONG,
            );
        }
    }

    AtaFdoEnumeratePort(ChanExt);

    PdoCount = 0;
    AtaScsiAddress.AsULONG = 0;
    loop {
        let mut DevExt: PATAPORT_DEVICE_EXTENSION = core::ptr::null_mut();
        DevExt = AtaFdoFindNextDeviceByPath(
            ChanExt,
            core::ptr::addr_of_mut!(AtaScsiAddress),
            TRUE,
            core::ptr::null_mut(),
        );
        if DevExt.is_null() {
            break;
        }
        if (*DevExt).NotPresent == 0 {
            PdoCount = PdoCount.wrapping_add(1);
        }
    }

    Size = FIELD_OFFSET!(DEVICE_RELATIONS, Objects[PdoCount]) as u32;
    DeviceRelations = ExAllocatePoolUninitialized(
        PagedPool,
        (Size as usize) as _,
        ATAPORT_TAG,
    ) as PDEVICE_RELATIONS;

    'cleanup: {
        if DeviceRelations.is_null() {
            ERR!(b"Failed to allocate device relations\n\0".as_ptr() as *const i8);
            break 'cleanup;
        }

        AtaFdoInitializeDeviceRelations(ChanExt, DeviceRelations);

        KeQuerySystemTime(core::ptr::addr_of_mut!(TimeFinish));
        EnumTimeMs = ((TimeFinish.QuadPart - TimeStart.QuadPart) / 10000) as u32;
        if EnumTimeMs >= 5000 {
            WARN!(
                b"%lu: QBR request took %lu ms, %lu devices\n\0".as_ptr() as *const i8,
                (*ChanExt).ScsiPortNumber,
                EnumTimeMs,
                (*DeviceRelations).Count,
            );
        } else {
            INFO!(
                b"%lu: QBR request took %lu ms, %lu devices\n\0".as_ptr() as *const i8,
                (*ChanExt).ScsiPortNumber,
                EnumTimeMs,
                (*DeviceRelations).Count,
            );
        }

        (*Irp).IoStatus.Information = DeviceRelations as ULONG_PTR;
        return STATUS_SUCCESS;
    }

    if !DeviceRelations.is_null() {
        ExFreePoolWithTag((DeviceRelations).cast(), ATAPORT_TAG);
    }

    return STATUS_INSUFFICIENT_RESOURCES;
}
