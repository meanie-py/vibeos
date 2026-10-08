use crate::setuplib::volutil::DismountVolume;
use crate::setuplib::fsrec::FileSystemToMBRPartitionType;
use crate::setuplib::registry::GetRootKeyByPredefKey;
use crate::setuplib::volutil::MountVolume;
use crate::setuplib::devutils::pOpenDevice;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_0C22BC20BFAC: [u16; 7] = [0x0025u16, 0x0073, 0x005C, 0x0025, 0x006C, 0x0075, 0x0000];
pub static RUSTCONV_WSTR_18CA6AF5ACA2: [u16; 33] = [
    0x005Cu16, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x005C, 0x0048, 0x0061,
    0x0072, 0x0064, 0x0064, 0x0069, 0x0073, 0x006B, 0x0025, 0x006C, 0x0075, 0x005C,
    0x0050, 0x0061, 0x0072, 0x0074, 0x0069, 0x0074, 0x0069, 0x006F, 0x006E, 0x0025,
    0x006C, 0x0075, 0x0000,
];
pub static RUSTCONV_WSTR_27B3B5E195F7: [u16; 31] = [
    0x005Cu16, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x005C, 0x0048, 0x0061,
    0x0072, 0x0064, 0x0064, 0x0069, 0x0073, 0x006B, 0x0025, 0x006C, 0x0075, 0x005C,
    0x0050, 0x0061, 0x0072, 0x0074, 0x0069, 0x0074, 0x0069, 0x006F, 0x006E, 0x0030,
    0x0000,
];
pub static RUSTCONV_WSTR_344F2CEBFA6E: [u16; 11] = [
    0x0049u16, 0x0064, 0x0065, 0x006E, 0x0074, 0x0069, 0x0066, 0x0069, 0x0065, 0x0072,
    0x0000,
];
pub static RUSTCONV_WSTR_717EA973176B: [u16; 20] = [
    0x005Cu16, 0x0053, 0x0063, 0x0073, 0x0069, 0x005C, 0x0053, 0x0063, 0x0073, 0x0069,
    0x0020, 0x0050, 0x006F, 0x0072, 0x0074, 0x0020, 0x0025, 0x0068, 0x0075, 0x0000,
];
pub static RUSTCONV_WSTR_903B52F937BA: [u16; 4] = [0x0052u16, 0x0041, 0x0057, 0x0000];
pub static RUSTCONV_WSTR_9FD9A3B3A66D: [u16; 19] = [
    0x0043u16, 0x006F, 0x006E, 0x0066, 0x0069, 0x0067, 0x0075, 0x0072, 0x0061, 0x0074,
    0x0069, 0x006F, 0x006E, 0x0020, 0x0044, 0x0061, 0x0074, 0x0061, 0x0000,
];
pub static RUSTCONV_WSTR_A4F854A2734C: [u16; 45] = [
    0x0025u16, 0x0073, 0x005C, 0x0025, 0x006C, 0x0075, 0x005C, 0x0044, 0x0069, 0x0073,
    0x006B, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F, 0x006C, 0x006C, 0x0065,
    0x0072, 0x005C, 0x0025, 0x006C, 0x0075, 0x005C, 0x0044, 0x0069, 0x0073, 0x006B,
    0x0050, 0x0065, 0x0072, 0x0069, 0x0070, 0x0068, 0x0065, 0x0072, 0x0061, 0x006C,
    0x005C, 0x0025, 0x006C, 0x0075, 0x0000,
];
pub static RUSTCONV_WSTR_A603842E44A2: [u16; 13] = [
    0x0025u16, 0x0030, 0x0038, 0x0078, 0x002D, 0x0025, 0x0030, 0x0038, 0x0078, 0x002D,
    0x0025, 0x0063, 0x0000,
];
pub static RUSTCONV_WSTR_A953DDDD78B1: [u16; 4] = [0x0046u16, 0x0041, 0x0054, 0x0000];
pub static RUSTCONV_WSTR_BA3A1CFAAA14: [u16; 22] = [
    0x0025u16, 0x0073, 0x005C, 0x0025, 0x006C, 0x0075, 0x005C, 0x0044, 0x0069, 0x0073,
    0x006B, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F, 0x006C, 0x006C, 0x0065,
    0x0072, 0x0000,
];
pub static RUSTCONV_WSTR_BB53EE05898F: [u16; 6] = [0x0046u16, 0x0041, 0x0054, 0x0033, 0x0032, 0x0000];
pub static RUSTCONV_WSTR_BE0DEB469A7B: [u16; 6] = [0x0042u16, 0x0054, 0x0052, 0x0046, 0x0053, 0x0000];
pub static RUSTCONV_WSTR_BFE4472B9CA3: [u16; 22] = [
    0x0053u16, 0x0059, 0x0053, 0x0054, 0x0045, 0x004D, 0x005C, 0x004D, 0x006F, 0x0075,
    0x006E, 0x0074, 0x0065, 0x0064, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065,
    0x0073, 0x0000,
];
pub static RUSTCONV_WSTR_CE946429FAFD: [u16; 46] = [
    0x005Cu16, 0x0052, 0x0065, 0x0067, 0x0069, 0x0073, 0x0074, 0x0072, 0x0079, 0x005C,
    0x004D, 0x0061, 0x0063, 0x0068, 0x0069, 0x006E, 0x0065, 0x005C, 0x0048, 0x0041,
    0x0052, 0x0044, 0x0057, 0x0041, 0x0052, 0x0045, 0x005C, 0x0044, 0x0045, 0x0053,
    0x0043, 0x0052, 0x0049, 0x0050, 0x0054, 0x0049, 0x004F, 0x004E, 0x005C, 0x0053,
    0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x0000,
];
pub static RUSTCONV_WSTR_DC1628A3C209: [u16; 26] = [
    0x0025u16, 0x0073, 0x005C, 0x0025, 0x006C, 0x0075, 0x005C, 0x0044, 0x0069, 0x0073,
    0x006B, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F, 0x006C, 0x006C, 0x0065,
    0x0072, 0x005C, 0x0025, 0x006C, 0x0075, 0x0000,
];
pub static RUSTCONV_WSTR_DC70E3A7FF9A: [u16; 16] = [
    0x005Cu16, 0x0044, 0x006F, 0x0073, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065,
    0x0073, 0x005C, 0x0025, 0x0063, 0x003A, 0x0000,
];
pub static RUSTCONV_WSTR_F00B6B101025: [u16; 41] = [
    0x0025u16, 0x0073, 0x005C, 0x0025, 0x006C, 0x0075, 0x005C, 0x0044, 0x0069, 0x0073,
    0x006B, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F, 0x006C, 0x006C, 0x0065,
    0x0072, 0x005C, 0x0025, 0x006C, 0x0075, 0x005C, 0x0044, 0x0069, 0x0073, 0x006B,
    0x0050, 0x0065, 0x0072, 0x0069, 0x0070, 0x0068, 0x0065, 0x0072, 0x0061, 0x006C,
    0x0000,
];
pub static RUSTCONV_WSTR_F3D8D400CF21: [u16; 7] = [0x0044u16, 0x0072, 0x0069, 0x0076, 0x0065, 0x0072, 0x0000];

use core::ffi::c_void;

extern "system" {
    fn RtlAssert( FailedAssertion: *mut core::ffi::c_void, FileName: *mut core::ffi::c_void, LineNumber: u32, Message: *mut core::ffi::c_char, );
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const ROOT_NAME: [::core::ffi::c_ushort; 67] = unsafe {
    ::core::mem::transmute::<
        [u8; 134],
        [::core::ffi::c_ushort; 67],
    >(
        *b"\\\0R\0e\0g\0i\0s\0t\0r\0y\0\\\0M\0a\0c\0h\0i\0n\0e\0\\\0H\0A\0R\0D\0W\0A\0R\0E\0\\\0D\0E\0S\0C\0R\0I\0P\0T\0I\0O\0N\0\\\0S\0y\0s\0t\0e\0m\0\\\0M\0u\0l\0t\0i\0f\0u\0n\0c\0t\0i\0o\0n\0A\0d\0a\0p\0t\0e\0r\0\0\0",
    )
};

// partlist.c packs it (pshpack1): the C writes a 12-byte MountedDevices value (signature +
// starting offset, mountmgr's basic-disk unique ID). Unpacked it was 16 bytes.
#[repr(C, packed)]
pub struct _REG_DISK_MOUNT_INFO {
    pub Signature: u32,
    pub StartingOffset: u64,
}
const _: () = assert!(core::mem::size_of::<_REG_DISK_MOUNT_INFO>() == 12);
pub type REG_DISK_MOUNT_INFO = _REG_DISK_MOUNT_INFO;
pub type PREG_DISK_MOUNT_INFO = *mut _REG_DISK_MOUNT_INFO;
#[no_mangle]
pub unsafe extern "C" fn AlignDown(
    Value: u64,
    Alignment: u32,
) -> u64 {
    let Temp: u64;

    Temp = Value / Alignment as u64;

    return Temp * Alignment as u64;
}
#[no_mangle]
pub unsafe extern "C" fn AlignUp(mut Value: u64, mut Alignment: u32) -> u64 {
    let mut Temp: u64 = 0;
    let mut Result: u64 = 0;
    Temp = Value.wrapping_div(Alignment as u64);
    Result = Temp.wrapping_mul(Alignment as u64);
    if Value.wrapping_rem(Alignment as u64) != 0 {
        Result = Result.wrapping_add(Alignment as u64);
    }
    return Result;
}

#[no_mangle]
pub unsafe extern "C" fn RoundingDivide(
    Dividend: ULONGLONG,
    Divisor: ULONGLONG,
) -> ULONGLONG {
    return Dividend
        .wrapping_add(Divisor.wrapping_div(2))
        .wrapping_div(Divisor);
}

unsafe extern "C" fn GetDriverName(mut DiskEntry: PDISKENTRY) {
    let mut QueryTable: [RTL_QUERY_REGISTRY_TABLE; 2] = [RTL_QUERY_REGISTRY_TABLE {
        QueryRoutine: None,
        Flags: 0,
        Name: core::ptr::null(),
        EntryContext: core::ptr::null_mut(),
        DefaultType: 0,
        DefaultData: core::ptr::null_mut(),
        DefaultLength: 0,
    }; 2];
    let mut KeyName: [u16; 32] = [0; 32];
    let mut Status: NTSTATUS = 0;

    RtlInitUnicodeString(&raw mut (*DiskEntry).DriverName, core::ptr::null());

    RtlStringCchPrintfW!(
        &raw mut KeyName as *mut u16,
        ARRAYSIZE!(KeyName),
        RUSTCONV_WSTR_717EA973176B.as_ptr(),
        (*DiskEntry).Port as i32,
    );

    RtlZeroMemory!(
        &raw mut QueryTable as *mut core::ffi::c_void,
        core::mem::size_of::<[RTL_QUERY_REGISTRY_TABLE; 2]>(),
    );

    QueryTable[0].Name = RUSTCONV_WSTR_F3D8D400CF21.as_ptr();
    QueryTable[0].Flags = RTL_QUERY_REGISTRY_DIRECT;
    QueryTable[0].EntryContext = &raw mut (*DiskEntry).DriverName as *mut core::ffi::c_void;

    Status = RtlQueryRegistryValues(
        RTL_REGISTRY_DEVICEMAP,
        &raw mut KeyName as *mut u16,
        &raw mut QueryTable as *mut RTL_QUERY_REGISTRY_TABLE,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT1!("RtlQueryRegistryValues() failed (Status %lx)\n", Status);
    }
}

pub unsafe extern "C" fn AssignDriveLetters(mut List: PPARTLIST) {
    let mut DiskEntry: PDISKENTRY = core::ptr::null_mut();
    let mut PartEntry: PPARTENTRY = core::ptr::null_mut();
    let mut Entry1: PLIST_ENTRY = core::ptr::null_mut();
    let mut Entry2: PLIST_ENTRY = core::ptr::null_mut();
    let mut Letter: u16 = 0x0043u16;

    Entry1 = (*List).DiskListHead.Flink;
    while Entry1 != &raw mut (*List).DiskListHead {
        DiskEntry = CONTAINING_RECORD!(Entry1, DISKENTRY, ListEntry);

        Entry2 = (*DiskEntry).PrimaryPartListHead.Flink;
        while Entry2 != &raw mut (*DiskEntry).PrimaryPartListHead {
            PartEntry = CONTAINING_RECORD!(Entry2, PARTENTRY, ListEntry);

            if !(*PartEntry).Volume.is_null() {
                (*(*PartEntry).Volume).Info.DriveLetter = UNICODE_NULL;

                if (*PartEntry).IsPartitioned != 0
                    && !IsContainerPartition!((*PartEntry).PartitionType)
                    && (IsRecognizedPartition!((*PartEntry).PartitionType)
                        || (*PartEntry).SectorCount.QuadPart != 0)
                {
                    if Letter <= 0x005Au16 {
                        let fresh = Letter;
                        Letter = Letter.wrapping_add(1);
                        (*(*PartEntry).Volume).Info.DriveLetter = fresh;
                    }
                }
            }

            Entry2 = (*Entry2).Flink;
        }

        Entry1 = (*Entry1).Flink;
    }

    Entry1 = (*List).DiskListHead.Flink;
    while Entry1 != &raw mut (*List).DiskListHead {
        DiskEntry = CONTAINING_RECORD!(Entry1, DISKENTRY, ListEntry);

        Entry2 = (*DiskEntry).LogicalPartListHead.Flink;
        while Entry2 != &raw mut (*DiskEntry).LogicalPartListHead {
            PartEntry = CONTAINING_RECORD!(Entry2, PARTENTRY, ListEntry);

            if !(*PartEntry).Volume.is_null() {
                (*(*PartEntry).Volume).Info.DriveLetter = UNICODE_NULL;

                if (*PartEntry).IsPartitioned != 0
                    && (IsRecognizedPartition!((*PartEntry).PartitionType)
                        || (*PartEntry).SectorCount.QuadPart != 0)
                {
                    if Letter <= 0x005Au16 {
                        let fresh = Letter;
                        Letter = Letter.wrapping_add(1);
                        (*(*PartEntry).Volume).Info.DriveLetter = fresh;
                    }
                }
            }

            Entry2 = (*Entry2).Flink;
        }

        Entry1 = (*Entry1).Flink;
    }
}
unsafe extern "system" fn DiskIdentifierQueryRoutine(
    mut ValueName: PWSTR,
    mut ValueType: ULONG,
    mut ValueData: PVOID,
    mut ValueLength: ULONG,
    mut Context: PVOID,
    mut EntryContext: PVOID,
) -> NTSTATUS {
    let mut BiosDiskEntry: PBIOSDISKENTRY = Context as PBIOSDISKENTRY;
    let mut NameU: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    if ValueType == REG_SZ
        && ValueLength as usize
            == (20 as usize)
                .wrapping_mul(core::mem::size_of::<u16>() as usize)
        && *(ValueData as *mut u16).offset(8 as isize) as i32
            == 0x002Du16 as i32
    {
        NameU.Buffer = ValueData as *mut u16;
        NameU.MaximumLength = (8 as usize)
            .wrapping_mul(core::mem::size_of::<u16>() as usize) as u16;
        NameU.Length = NameU.MaximumLength;
        RtlUnicodeStringToInteger(
            &mut NameU,
            16,
            &mut (*BiosDiskEntry).Checksum,
        );
        NameU.Buffer = (ValueData as *mut u16).offset(9 as isize);
        RtlUnicodeStringToInteger(
            &mut NameU,
            16,
            &mut (*BiosDiskEntry).Signature,
        );
        return STATUS_SUCCESS;
    }
    return STATUS_UNSUCCESSFUL;
}
unsafe extern "system" fn DiskConfigurationDataQueryRoutine(
    ValueName: PWSTR,
    ValueType: ULONG,
    ValueData: PVOID,
    ValueLength: ULONG,
    Context: PVOID,
    EntryContext: PVOID,
) -> NTSTATUS {
    let mut BiosDiskEntry: PBIOSDISKENTRY = Context as PBIOSDISKENTRY;
    let mut FullResourceDescriptor: PCM_FULL_RESOURCE_DESCRIPTOR =
        core::ptr::null_mut::<CM_FULL_RESOURCE_DESCRIPTOR>();
    let mut DiskGeometry: PCM_DISK_GEOMETRY_DEVICE_DATA =
        core::ptr::null_mut::<CM_DISK_GEOMETRY_DEVICE_DATA>();
    let mut i: ULONG = 0;

    if ValueType != REG_FULL_RESOURCE_DESCRIPTOR
        || (ValueLength as usize) < core::mem::size_of::<CM_FULL_RESOURCE_DESCRIPTOR>() as usize
    {
        return STATUS_UNSUCCESSFUL;
    }

    FullResourceDescriptor = ValueData as PCM_FULL_RESOURCE_DESCRIPTOR;

    i = 0;
    while i < (*FullResourceDescriptor).PartialResourceList.Count {
        if ((*(&raw mut (*FullResourceDescriptor)
            .PartialResourceList
            .PartialDescriptors as *mut CM_PARTIAL_RESOURCE_DESCRIPTOR)
            .offset(i as isize))
        .Type as core::ffi::c_int) as u32
            != CmResourceTypeDeviceSpecific
            || (*(&raw mut (*FullResourceDescriptor)
                .PartialResourceList
                .PartialDescriptors as *mut CM_PARTIAL_RESOURCE_DESCRIPTOR)
                .offset(i as isize))
            .u
            .DeviceSpecificData
            .DataSize as usize
                != core::mem::size_of::<CM_DISK_GEOMETRY_DEVICE_DATA>() as usize
        {
            i = i.wrapping_add(1);
        } else {
            DiskGeometry = (&raw mut (*FullResourceDescriptor)
                .PartialResourceList
                .PartialDescriptors
                as *mut CM_PARTIAL_RESOURCE_DESCRIPTOR)
                .offset(i.wrapping_add(1) as isize)
                as *mut CM_PARTIAL_RESOURCE_DESCRIPTOR
                as PCM_DISK_GEOMETRY_DEVICE_DATA;
            (*BiosDiskEntry).DiskGeometry = *DiskGeometry as CM_DISK_GEOMETRY_DEVICE_DATA;
            return STATUS_SUCCESS;
        }
    }

    return STATUS_UNSUCCESSFUL;
}
unsafe extern "system" fn SystemConfigurationDataQueryRoutine(
    mut ValueName: PWSTR,
    mut ValueType: ULONG,
    mut ValueData: PVOID,
    mut ValueLength: ULONG,
    mut Context: PVOID,
    mut EntryContext: PVOID,
) -> NTSTATUS {
    let mut FullResourceDescriptor: PCM_FULL_RESOURCE_DESCRIPTOR = core::ptr::null_mut();
    let mut Int13Drives: *mut PCM_INT13_DRIVE_PARAMETER = Context as *mut PCM_INT13_DRIVE_PARAMETER;
    let mut i: ULONG = 0;

    if ValueType != REG_FULL_RESOURCE_DESCRIPTOR ||
        ValueLength < core::mem::size_of::<CM_FULL_RESOURCE_DESCRIPTOR>() as ULONG
    {
        return STATUS_UNSUCCESSFUL;
    }

    FullResourceDescriptor = ValueData as PCM_FULL_RESOURCE_DESCRIPTOR;

    i = 0;
    while i < (*FullResourceDescriptor).PartialResourceList.Count {
        if ((*((*FullResourceDescriptor).PartialResourceList.PartialDescriptors.as_mut_ptr()).offset(i as isize)).Type) as u32 != CmResourceTypeDeviceSpecific ||
            (*((*FullResourceDescriptor).PartialResourceList.PartialDescriptors.as_mut_ptr()).offset(i as isize)).u.DeviceSpecificData.DataSize % core::mem::size_of::<CM_INT13_DRIVE_PARAMETER>() as ULONG != 0
        {
            i = i.wrapping_add(1);
            continue;
        }

        *Int13Drives = RtlAllocateHeap(
            ProcessHeap,
            0,
            ((*((*FullResourceDescriptor).PartialResourceList.PartialDescriptors.as_mut_ptr()).offset(i as isize)).u.DeviceSpecificData.DataSize) as SIZE_T,
        ) as PCM_INT13_DRIVE_PARAMETER;

        if (*Int13Drives).is_null() {
            return STATUS_NO_MEMORY;
        }

        memcpy(
            *Int13Drives as *mut core::ffi::c_void,
            ((*FullResourceDescriptor).PartialResourceList.PartialDescriptors.as_mut_ptr()).offset((i + 1) as isize) as *const core::ffi::c_void,
            ((*((*FullResourceDescriptor).PartialResourceList.PartialDescriptors.as_mut_ptr()).offset(i as isize)).u.DeviceSpecificData.DataSize) as _,
        );

        return STATUS_SUCCESS;
    }

    return STATUS_UNSUCCESSFUL;
}

unsafe extern "C" fn EnumerateBiosDiskEntries(mut PartList: PPARTLIST) {
    let mut QueryTable: [RTL_QUERY_REGISTRY_TABLE; 3] = [RTL_QUERY_REGISTRY_TABLE {
        QueryRoutine: None,
        Flags: 0,
        Name: core::ptr::null(),
        EntryContext: core::ptr::null_mut(),
        DefaultType: 0,
        DefaultData: core::ptr::null_mut(),
        DefaultLength: 0,
    }; 3];
    let mut Name: [u16; 120] = [0; 120];
    let mut AdapterCount: u32;
    let mut ControllerCount: u32;
    let mut DiskCount: u32;
    let mut Status: i32;
    let mut Int13Drives: PCM_INT13_DRIVE_PARAMETER = core::ptr::null_mut();
    let mut BiosDiskEntry: PBIOSDISKENTRY;

    memset(
        QueryTable.as_mut_ptr() as *mut core::ffi::c_void,
        0,
        core::mem::size_of::<[RTL_QUERY_REGISTRY_TABLE; 3]>(),
    );

    QueryTable[1].Name = RUSTCONV_WSTR_9FD9A3B3A66D.as_ptr();
    QueryTable[1].QueryRoutine = Some(SystemConfigurationDataQueryRoutine);
    Int13Drives = core::ptr::null_mut();
    Status = RtlQueryRegistryValues(
        RTL_REGISTRY_ABSOLUTE,
        RUSTCONV_WSTR_CE946429FAFD.as_ptr(),
        QueryTable.as_mut_ptr().add(1),
        &mut Int13Drives as *mut PCM_INT13_DRIVE_PARAMETER as *mut core::ffi::c_void,
        core::ptr::null_mut(),
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT1!(
            b"Unable to query the 'Configuration Data' key in '\\Registry\\Machine\\HARDWARE\\DESCRIPTION\\System', status=%lx\n\0"
                .as_ptr() as *const i8,
            Status,
        );
        return;
    }

    AdapterCount = 0;
    loop {
        RtlStringCchPrintfW!(
            Name.as_mut_ptr(),
            ARRAYSIZE!(Name),
            RUSTCONV_WSTR_0C22BC20BFAC.as_ptr(),
            ROOT_NAME,
            AdapterCount,
        );
        Status = RtlQueryRegistryValues(
            RTL_REGISTRY_ABSOLUTE,
            Name.as_ptr(),
            QueryTable.as_mut_ptr().add(2),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        if ((NT_SUCCESS!(Status)) == 0) {
            break;
        }

        RtlStringCchPrintfW!(
            Name.as_mut_ptr(),
            ARRAYSIZE!(Name),
            RUSTCONV_WSTR_BA3A1CFAAA14.as_ptr(),
            ROOT_NAME,
            AdapterCount,
        );
        Status = RtlQueryRegistryValues(
            RTL_REGISTRY_ABSOLUTE,
            Name.as_ptr(),
            QueryTable.as_mut_ptr().add(2),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        if NT_SUCCESS!(Status) != 0 {
            ControllerCount = 0;
            loop {
                RtlStringCchPrintfW!(
                    Name.as_mut_ptr(),
                    ARRAYSIZE!(Name),
                    RUSTCONV_WSTR_DC1628A3C209.as_ptr(),
                    ROOT_NAME,
                    AdapterCount,
                    ControllerCount,
                );
                Status = RtlQueryRegistryValues(
                    RTL_REGISTRY_ABSOLUTE,
                    Name.as_ptr(),
                    QueryTable.as_mut_ptr().add(2),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                );
                if ((NT_SUCCESS!(Status)) == 0) {
                    RtlFreeHeap(ProcessHeap, 0, Int13Drives as *mut core::ffi::c_void);
                    return;
                }

                RtlStringCchPrintfW!(
                    Name.as_mut_ptr(),
                    ARRAYSIZE!(Name),
                    RUSTCONV_WSTR_F00B6B101025.as_ptr(),
                    ROOT_NAME,
                    AdapterCount,
                    ControllerCount,
                );
                Status = RtlQueryRegistryValues(
                    RTL_REGISTRY_ABSOLUTE,
                    Name.as_ptr(),
                    QueryTable.as_mut_ptr().add(2),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                );
                if NT_SUCCESS!(Status) != 0 {
                    QueryTable[0].Name = RUSTCONV_WSTR_344F2CEBFA6E.as_ptr();
                    QueryTable[0].QueryRoutine = Some(DiskIdentifierQueryRoutine);
                    QueryTable[1].Name = RUSTCONV_WSTR_9FD9A3B3A66D.as_ptr();
                    QueryTable[1].QueryRoutine = Some(DiskConfigurationDataQueryRoutine);

                    DiskCount = 0;
                    loop {
                        BiosDiskEntry = RtlAllocateHeap(
                            ProcessHeap,
                            HEAP_ZERO_MEMORY,
                            (core::mem::size_of::<BIOSDISKENTRY>()) as SIZE_T,
                        ) as PBIOSDISKENTRY;
                        if BiosDiskEntry.is_null() {
                            RtlFreeHeap(ProcessHeap, 0, Int13Drives as *mut core::ffi::c_void);
                            return;
                        }

                        RtlStringCchPrintfW!(
                            Name.as_mut_ptr(),
                            ARRAYSIZE!(Name),
                            RUSTCONV_WSTR_A4F854A2734C.as_ptr(),
                            ROOT_NAME,
                            AdapterCount,
                            ControllerCount,
                            DiskCount,
                        );
                        Status = RtlQueryRegistryValues(
                            RTL_REGISTRY_ABSOLUTE,
                            Name.as_ptr(),
                            QueryTable.as_mut_ptr(),
                            BiosDiskEntry as *mut core::ffi::c_void,
                            core::ptr::null_mut(),
                        );
                        if ((NT_SUCCESS!(Status)) == 0) {
                            RtlFreeHeap(ProcessHeap, 0, BiosDiskEntry as *mut core::ffi::c_void);
                            RtlFreeHeap(ProcessHeap, 0, Int13Drives as *mut core::ffi::c_void);
                            return;
                        }

                        (*BiosDiskEntry).AdapterNumber = 0;
                        (*BiosDiskEntry).ControllerNumber = ControllerCount;
                        (*BiosDiskEntry).DiskNumber = DiskCount;
                        (*BiosDiskEntry).DiskEntry = core::ptr::null_mut();

                        if DiskCount < ((*Int13Drives.add(0)).NumberDrives) as _ {
                            (*BiosDiskEntry).Int13DiskData = *Int13Drives.add((DiskCount) as _);
                        } else {
                            DPRINT1!(b"Didn't find Int13 drive data for disk %u\n\0".as_ptr() as *const i8, DiskCount);
                        }

                        InsertTailList(
                            &mut (*PartList).BiosDiskListHead,
                            &mut (*BiosDiskEntry).ListEntry,
                        );

                        DPRINT!(b"--->\n\0".as_ptr() as *const i8);
                        DPRINT!(b"AdapterNumber:     %lu\n\0".as_ptr() as *const i8, (*BiosDiskEntry).AdapterNumber);
                        DPRINT!(b"ControllerNumber:  %lu\n\0".as_ptr() as *const i8, (*BiosDiskEntry).ControllerNumber);
                        DPRINT!(b"DiskNumber:        %lu\n\0".as_ptr() as *const i8, (*BiosDiskEntry).DiskNumber);
                        DPRINT!(b"Signature:         %08lx\n\0".as_ptr() as *const i8, (*BiosDiskEntry).Signature);
                        DPRINT!(b"Checksum:          %08lx\n\0".as_ptr() as *const i8, (*BiosDiskEntry).Checksum);
                        DPRINT!(b"BytesPerSector:    %lu\n\0".as_ptr() as *const i8, (*BiosDiskEntry).DiskGeometry.BytesPerSector);
                        DPRINT!(b"NumberOfCylinders: %lu\n\0".as_ptr() as *const i8, (*BiosDiskEntry).DiskGeometry.NumberOfCylinders);
                        DPRINT!(b"NumberOfHeads:     %lu\n\0".as_ptr() as *const i8, (*BiosDiskEntry).DiskGeometry.NumberOfHeads);
                        DPRINT!(b"DriveSelect:       %02x\n\0".as_ptr() as *const i8, (*BiosDiskEntry).Int13DiskData.DriveSelect);
                        DPRINT!(b"MaxCylinders:      %lu\n\0".as_ptr() as *const i8, (*BiosDiskEntry).Int13DiskData.MaxCylinders);
                        DPRINT!(b"SectorsPerTrack:   %d\n\0".as_ptr() as *const i8, (*BiosDiskEntry).Int13DiskData.SectorsPerTrack);
                        DPRINT!(b"MaxHeads:          %d\n\0".as_ptr() as *const i8, (*BiosDiskEntry).Int13DiskData.MaxHeads);
                        DPRINT!(b"NumberDrives:      %d\n\0".as_ptr() as *const i8, (*BiosDiskEntry).Int13DiskData.NumberDrives);
                        DPRINT!(b"<---\n\0".as_ptr() as *const i8);

                        DiskCount = DiskCount.wrapping_add(1);
                    }
                }
                ControllerCount = ControllerCount.wrapping_add(1);
            }
        }
        AdapterCount = AdapterCount.wrapping_add(1);
    }

    RtlFreeHeap(ProcessHeap, 0, Int13Drives as *mut core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn IsDiskSuperFloppy2(
    DiskInfo: *const DISK_PARTITION_INFO,
    DiskSize: *const u64,
    PartitionInfo: *const PARTITION_INFORMATION,
) -> i32 {
    if (*DiskInfo).SizeOfPartitionInfo < RTL_SIZEOF_THROUGH_FIELD!(DISK_PARTITION_INFO, __bindgen_anon_1.Mbr) as u32 {
        return (FALSE) as i32;
    }
    if (*DiskInfo).PartitionStyle != PARTITION_STYLE_MBR {
        return (FALSE) as i32;
    }
    if !((*PartitionInfo).StartingOffset.QuadPart == 0
        && (*PartitionInfo).HiddenSectors == 0)
    {
        return (FALSE) as i32;
    }
    if (*DiskInfo).__bindgen_anon_1.Mbr.Signature != 1 {
        DbgPrint(
            b"Super-Floppy signature %08x != 1\n\0".as_ptr() as PCSTR,
            (*DiskInfo).__bindgen_anon_1.Mbr.Signature,
        );
    }
    if ((*PartitionInfo).RecognizedPartition) as u32 != TRUE
        || ((*PartitionInfo).PartitionType) as u32 != PARTITION_FAT_16
        || ((*PartitionInfo).BootIndicator) as u32 != FALSE
    {
        DbgPrint(
            b"Super-Floppy does not return default settings:\n    RecognizedPartition = %s, expected TRUE\n    PartitionType = 0x%02x, expected 0x04 (PARTITION_FAT_16)\n    BootIndicator = %s, expected FALSE\n\0".as_ptr() as PCSTR,
            if (*PartitionInfo).RecognizedPartition != 0 {
                b"TRUE\0".as_ptr() as *const i8
            } else {
                b"FALSE\0".as_ptr() as *const i8
            },
            (*PartitionInfo).PartitionType as core::ffi::c_uint,
            if (*PartitionInfo).BootIndicator != 0 {
                b"TRUE\0".as_ptr() as *const i8
            } else {
                b"FALSE\0".as_ptr() as *const i8
            },
        );
    }
    if !DiskSize.is_null() && ((*PartitionInfo).PartitionLength.QuadPart as ULONGLONG) != *DiskSize {
        DbgPrint(
            b"PartitionLength = %I64u is different from DiskSize = %I64u\n\0".as_ptr() as PCSTR,
            (*PartitionInfo).PartitionLength.QuadPart,
            *DiskSize,
        );
    }
    return (TRUE) as i32;
}
#[no_mangle]
pub unsafe extern "C" fn IsDiskSuperFloppy(
    Layout: *const DRIVE_LAYOUT_INFORMATION,
    DiskSize: *const u64,
) -> i32 {
    let mut DiskInfo: DISK_PARTITION_INFO = core::mem::zeroed();

    if (*Layout).PartitionCount != 1 {
        return (FALSE) as i32;
    }

    DiskInfo.SizeOfPartitionInfo = RTL_SIZEOF_THROUGH_FIELD!(DISK_PARTITION_INFO, __bindgen_anon_1.Mbr) as u32;
    DiskInfo.PartitionStyle = PARTITION_STYLE_MBR;
    DiskInfo.__bindgen_anon_1.Mbr.Signature = (*Layout).Signature;
    DiskInfo.__bindgen_anon_1.Mbr.CheckSum = 0;

    return IsDiskSuperFloppy2(
        core::ptr::addr_of_mut!(DiskInfo),
        DiskSize,
        core::ptr::addr_of!((*Layout).PartitionEntry) as *const PARTITION_INFORMATION,
    );
}
#[no_mangle]
pub unsafe extern "C" fn IsDiskSuperFloppyEx(
    LayoutEx: *const DRIVE_LAYOUT_INFORMATION_EX,
    DiskSize: *const u64,
) -> i32 {
    let mut DiskInfo: DISK_PARTITION_INFO = core::mem::zeroed();
    let mut PartitionInfoEx: *const PARTITION_INFORMATION_EX = core::ptr::null();
    let mut PartitionInfo: PARTITION_INFORMATION = core::mem::zeroed();

    if (*LayoutEx).PartitionStyle != PARTITION_STYLE_MBR {
        return (FALSE) as i32;
    }
    if (*LayoutEx).PartitionCount != 1 {
        return (FALSE) as i32;
    }

    DiskInfo.SizeOfPartitionInfo = RTL_SIZEOF_THROUGH_FIELD!(DISK_PARTITION_INFO, __bindgen_anon_1.Mbr) as u32;
    DiskInfo.PartitionStyle = PARTITION_STYLE_MBR;
    DiskInfo.__bindgen_anon_1.Mbr.Signature = (*LayoutEx).__bindgen_anon_1.Mbr.Signature;
    DiskInfo.__bindgen_anon_1.Mbr.CheckSum = 0;

    PartitionInfoEx = core::ptr::addr_of!((*LayoutEx).PartitionEntry) as *const PARTITION_INFORMATION_EX;

    PartitionInfo.StartingOffset = (*PartitionInfoEx).StartingOffset;
    PartitionInfo.PartitionLength = (*PartitionInfoEx).PartitionLength;
    PartitionInfo.HiddenSectors = (*PartitionInfoEx).__bindgen_anon_1.Mbr.HiddenSectors;
    PartitionInfo.PartitionNumber = (*PartitionInfoEx).PartitionNumber;
    PartitionInfo.PartitionType = (*PartitionInfoEx).__bindgen_anon_1.Mbr.PartitionType;
    PartitionInfo.BootIndicator = (*PartitionInfoEx).__bindgen_anon_1.Mbr.BootIndicator;
    PartitionInfo.RecognizedPartition = (*PartitionInfoEx).__bindgen_anon_1.Mbr.RecognizedPartition;
    PartitionInfo.RewritePartition = (*PartitionInfoEx).RewritePartition;

    return IsDiskSuperFloppy2(
        core::ptr::addr_of_mut!(DiskInfo),
        DiskSize,
        core::ptr::addr_of_mut!(PartitionInfo),
    );
}
#[no_mangle]
pub unsafe extern "C" fn IsSuperFloppy(
    DiskEntry: *mut DISKENTRY,
) -> i32 {
    let mut DiskSize: u64;

    if (*DiskEntry).LayoutBuffer.is_null() {
        return (FALSE) as i32;
    }

    if (*DiskEntry).DiskStyle != PARTITION_STYLE_MBR {
        return (FALSE) as i32;
    }

    DiskSize = (*DiskEntry).SectorCount.QuadPart * (*DiskEntry).BytesPerSector as u64;
    return IsDiskSuperFloppy((*DiskEntry).LayoutBuffer, &mut DiskSize);
}

unsafe extern "C" fn InsertDiskRegion(
    mut DiskEntry: PDISKENTRY,
    mut PartEntry: PPARTENTRY,
    mut LogicalPartition: BOOLEAN,
) -> BOOLEAN {
    let mut List: PLIST_ENTRY = core::ptr::null_mut();
    let mut Entry: PLIST_ENTRY = core::ptr::null_mut();
    let mut PartEntry2: PPARTENTRY = core::ptr::null_mut();

    if LogicalPartition != 0 {
        List = core::ptr::addr_of_mut!((*DiskEntry).LogicalPartListHead);
    } else {
        List = core::ptr::addr_of_mut!((*DiskEntry).PrimaryPartListHead);
    }

    Entry = (*List).Flink;
    while Entry != List {
        PartEntry2 = CONTAINING_RECORD!(Entry, PARTENTRY, ListEntry);

        if !(((*PartEntry2).PartitionType) as u32 == PARTITION_ENTRY_UNUSED
            && (*PartEntry2).StartSector.QuadPart == 0
            || (*PartEntry2).SectorCount.QuadPart == 0)
        {
            if !((*PartEntry2)
                .StartSector
                .QuadPart
                .wrapping_add((*PartEntry2).SectorCount.QuadPart)
                .wrapping_sub(1)
                < (*PartEntry).StartSector.QuadPart)
            {
                if max(
                    (*PartEntry).StartSector.QuadPart,
                    (*PartEntry2).StartSector.QuadPart,
                ) <= min(
                    (*PartEntry)
                        .StartSector
                        .QuadPart
                        .wrapping_add((*PartEntry).SectorCount.QuadPart)
                        .wrapping_sub(1),
                    (*PartEntry2)
                        .StartSector
                        .QuadPart
                        .wrapping_add((*PartEntry2).SectorCount.QuadPart)
                        .wrapping_sub(1),
                ) {
                    DPRINT1!(
                        b"Disk region overlap problem, stopping there!\nPartition to be inserted:\n    StartSector = %I64u ; EndSector = %I64u\nExisting disk region:\n    StartSector = %I64u ; EndSector = %I64u\n\0"
                            .as_ptr() as PCSTR,
                        (*PartEntry).StartSector.QuadPart,
                        (*PartEntry)
                            .StartSector
                            .QuadPart
                            .wrapping_add((*PartEntry).SectorCount.QuadPart)
                            .wrapping_sub(1),
                        (*PartEntry2).StartSector.QuadPart,
                        (*PartEntry2)
                            .StartSector
                            .QuadPart
                            .wrapping_add((*PartEntry2).SectorCount.QuadPart)
                            .wrapping_sub(1),
                    );
                    return (FALSE) as u8;
                }
                break;
            }
        }
        Entry = (*Entry).Flink;
    }

    InsertTailList(Entry, core::ptr::addr_of_mut!((*PartEntry).ListEntry));
    return (TRUE) as u8;
}

unsafe extern "C" fn CreateInsertBlankRegion(
    DiskEntry: PDISKENTRY,
    ListHead: PLIST_ENTRY,
    StartSector: ULONGLONG,
    SectorCount: ULONGLONG,
    LogicalSpace: BOOLEAN,
) -> PPARTENTRY {
    let mut NewPartEntry: PPARTENTRY = core::ptr::null_mut();

    NewPartEntry = RtlAllocateHeap(
        ProcessHeap,
        HEAP_ZERO_MEMORY,
        (core::mem::size_of::<PARTENTRY>()) as SIZE_T,
    ) as PPARTENTRY;

    if NewPartEntry.is_null() {
        return core::ptr::null_mut();
    }

    (*NewPartEntry).DiskEntry = DiskEntry;
    (*NewPartEntry).StartSector.QuadPart = StartSector;
    (*NewPartEntry).SectorCount.QuadPart = SectorCount;
    (*NewPartEntry).LogicalPartition = LogicalSpace;
    (*NewPartEntry).IsPartitioned = (FALSE) as u8;
    (*NewPartEntry).PartitionType = (PARTITION_ENTRY_UNUSED) as u8;
    (*NewPartEntry).Volume = core::ptr::null_mut();

    DPRINT1!(
        b"First Sector : %I64u\n\0".as_ptr() as PCSTR,
        (*NewPartEntry).StartSector.QuadPart,
    );
    DPRINT1!(
        b"Last Sector  : %I64u\n\0".as_ptr() as PCSTR,
        (*NewPartEntry)
            .StartSector
            .QuadPart
            .wrapping_add((*NewPartEntry).SectorCount.QuadPart)
            .wrapping_sub(1),
    );
    DPRINT1!(
        b"Total Sectors: %I64u\n\0".as_ptr() as PCSTR,
        (*NewPartEntry).SectorCount.QuadPart,
    );

    InsertTailList(ListHead, &raw mut (*NewPartEntry).ListEntry);

    NewPartEntry
}

pub unsafe extern "C" fn DestroyRegion(mut PartEntry: PPARTENTRY) {
    if !(*PartEntry).Volume.is_null() {
        RtlFreeHeap(ProcessHeap, 0, ((*PartEntry).Volume).cast());
    }
    RtlFreeHeap(ProcessHeap, 0, (PartEntry).cast());
}

unsafe extern "C" fn AddLogicalDiskSpace(mut DiskEntry: PDISKENTRY) {
    let mut StartSector: ULONGLONG = 0;
    let mut SectorCount: ULONGLONG = 0;
    let mut NewPartEntry: PPARTENTRY = core::ptr::null_mut();

    DPRINT!(b"AddLogicalDiskSpace()\n\0".as_ptr() as PCSTR);

    StartSector = (*(*DiskEntry).ExtendedPartition)
        .StartSector
        .QuadPart
        .wrapping_add((*DiskEntry).SectorAlignment as ULONGLONG);
    SectorCount = (*(*DiskEntry).ExtendedPartition)
        .SectorCount
        .QuadPart
        .wrapping_sub((*DiskEntry).SectorAlignment as ULONGLONG);

    NewPartEntry = CreateInsertBlankRegion(
        DiskEntry,
        core::ptr::addr_of_mut!((*DiskEntry).LogicalPartListHead),
        StartSector,
        SectorCount,
        (TRUE) as BOOLEAN,
    );
    if NewPartEntry.is_null() {
        DPRINT1!(b"Failed to create a new empty region for full extended partition space!\n\0".as_ptr() as PCSTR);
    }
}

pub unsafe extern "C" fn InitializePartitionEntry(
    mut PartEntry: PPARTENTRY,
    mut SizeBytes: ULONGLONG,
    mut PartitionInfo: ULONG_PTR,
) -> BOOLEAN {
    let mut DiskEntry: PDISKENTRY = (*PartEntry).DiskEntry;
    let mut SectorCount: ULONGLONG;
    let mut isContainer: BOOLEAN = (IsContainerPartition!(PartitionInfo as UCHAR)) as u8;

    DPRINT1!(
        b"Current entry sector count: %I64u\n\0".as_ptr() as PCSTR,
        (*PartEntry).SectorCount.QuadPart,
    );

    if SizeBytes == 0
        || SizeBytes
            == (*PartEntry)
                .SectorCount
                .QuadPart
                .wrapping_mul((*(*PartEntry).DiskEntry).BytesPerSector as ULONGLONG)
    {
        SectorCount = (*PartEntry).SectorCount.QuadPart;
    } else {
        SectorCount = SizeBytes.wrapping_div((*DiskEntry).BytesPerSector as ULONGLONG);
        if SectorCount == 0 {
            DPRINT1!(b"Partition size %I64u too small\n\0".as_ptr() as PCSTR, SizeBytes);
            return (FALSE) as u8;
        }
    }
    DPRINT1!(b"    New sector count: %I64u\n\0".as_ptr() as PCSTR, SectorCount);

    if SectorCount > (*PartEntry).SectorCount.QuadPart {
        return (FALSE) as u8;
    }

    if SectorCount == 0
        || AlignDown(
            (*PartEntry).StartSector.QuadPart.wrapping_add(SectorCount),
            (*DiskEntry).SectorAlignment,
        )
        .wrapping_sub((*PartEntry).StartSector.QuadPart)
            == (*PartEntry).SectorCount.QuadPart
    {
    } else {
        let mut StartSector: ULONGLONG;
        let mut SectorCount2: ULONGLONG;
        let mut NewPartEntry: PPARTENTRY;

        StartSector = AlignDown(
            (*PartEntry).StartSector.QuadPart.wrapping_add(SectorCount),
            (*DiskEntry).SectorAlignment,
        );
        SectorCount2 = (*PartEntry)
            .StartSector
            .QuadPart
            .wrapping_add((*PartEntry).SectorCount.QuadPart)
            .wrapping_sub(StartSector);

        NewPartEntry = CreateInsertBlankRegion(
            DiskEntry,
            (*PartEntry).ListEntry.Flink,
            StartSector,
            SectorCount2,
            (*PartEntry).LogicalPartition,
        );
        if NewPartEntry.is_null() {
            DPRINT1!(
                b"Failed to create a new empty region for disk space!\n\0".as_ptr() as PCSTR,
            );
            return (FALSE) as u8;
        }

        (*PartEntry).SectorCount.QuadPart =
            StartSector.wrapping_sub((*PartEntry).StartSector.QuadPart);
    }

    (*PartEntry).New = (TRUE) as u8;
    (*PartEntry).IsPartitioned = (TRUE) as u8;

    (*PartEntry).BootIndicator = (FALSE) as u8;
    if PartitionInfo != 0 {
        if isContainer == 0 {
            (*PartEntry).PartitionType = PartitionInfo as UCHAR;
        } else {
            if (*PartEntry)
                .StartSector
                .QuadPart
                .wrapping_add((*PartEntry).SectorCount.QuadPart)
                .wrapping_sub(1)
                .wrapping_div(
                    (*DiskEntry)
                        .TracksPerCylinder
                        .wrapping_mul((*DiskEntry).SectorsPerTrack) as ULONGLONG,
                )
                < 1024
            {
                (*PartEntry).PartitionType = (PARTITION_EXTENDED) as u8;
            } else {
                (*PartEntry).PartitionType = (PARTITION_XINT13_EXTENDED) as u8;
            }
        }
    } else {
        (*PartEntry).PartitionType = FileSystemToMBRPartitionType(
            RUSTCONV_WSTR_903B52F937BA.as_ptr(),
            (*PartEntry).StartSector.QuadPart,
            (*PartEntry).SectorCount.QuadPart,
        );
    }

    if isContainer != 0 {
        (*DiskEntry).ExtendedPartition = PartEntry;
        AddLogicalDiskSpace(DiskEntry);
    }

    DPRINT1!(
        b"First Sector : %I64u\n\0".as_ptr() as PCSTR,
        (*PartEntry).StartSector.QuadPart,
    );
    DPRINT1!(
        b"Last Sector  : %I64u\n\0".as_ptr() as PCSTR,
        (*PartEntry)
            .StartSector
            .QuadPart
            .wrapping_add((*PartEntry).SectorCount.QuadPart)
            .wrapping_sub(1),
    );
    DPRINT1!(
        b"Total Sectors: %I64u\n\0".as_ptr() as PCSTR,
        (*PartEntry).SectorCount.QuadPart,
    );

    return (TRUE) as u8;
}

pub unsafe extern "C" fn InitPartitionDeviceName(mut PartEntry: PPARTENTRY) {
    let mut Status: NTSTATUS = 0;

    if IsContainerPartition!((*PartEntry).PartitionType) != false {
        return;
    }

    Status = RtlStringCchPrintfW!(
        (*PartEntry).DeviceName.as_mut_ptr(),
        (core::mem::size_of::<[u16; 260]>() / core::mem::size_of::<u16>()) as usize,
        RUSTCONV_WSTR_18CA6AF5ACA2.as_ptr(),
        (*(*PartEntry).DiskEntry).DiskNumber,
        (*PartEntry).PartitionNumber,
    );
}

pub unsafe extern "C" fn InitVolumeDeviceName(
    mut Volume: PVOLENTRY,
    mut AltDeviceName: PCWSTR,
) -> NTSTATUS {
    let mut Status: NTSTATUS = 0;
    let mut IoStatusBlock: _IO_STATUS_BLOCK = { let mut __lit: _IO_STATUS_BLOCK = core::mem::zeroed(); __lit.__bindgen_anon_1.Status = 0; __lit.Information = 0; __lit };
    let mut VolumeHandle: HANDLE = core::ptr::null_mut();
    // partlist.c declares this as an anonymous function-local struct (MOUNTDEV_NAME with a
    // fixed 256-WCHAR buffer, since the device name won't be longer).
    #[repr(C)]
    struct DeviceNameStruct {
        NameLength: USHORT,
        Name: [WCHAR; 256],
    }
    let mut DeviceName: DeviceNameStruct = DeviceNameStruct {
        NameLength: 0,
        Name: [0; 256],
    };

    if (*Volume).Info.DeviceName[0] != 0 {
        return STATUS_SUCCESS;
    }

    if AltDeviceName.is_null() {
        let mut PartEntry: PPARTENTRY = (*Volume).PartEntry;
        AltDeviceName = core::ptr::addr_of_mut!((*PartEntry).DeviceName) as PCWSTR;
    }

    Status = RtlStringCchCopyW(
        (core::ptr::addr_of_mut!((*Volume).Info.DeviceName)).cast(),
        (_countof!((*Volume).Info.DeviceName)) as _,
        AltDeviceName,
    );

    Status = pOpenDevice(
        (*Volume).Info.DeviceName.as_ptr(),
        &mut VolumeHandle,
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT1!("pOpenDevice() failed, Status 0x%08lx\n", Status);
        return Status;
    }

    Status = NtDeviceIoControlFile(
        VolumeHandle,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        &mut IoStatusBlock,
        IOCTL_MOUNTDEV_QUERY_DEVICE_NAME,
        core::ptr::null_mut(),
        0,
        &mut DeviceName as *mut DeviceNameStruct as *mut core::ffi::c_void,
        core::mem::size_of::<DeviceNameStruct>() as u32,
    );
    NtClose(VolumeHandle);

    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT1!("IOCTL_MOUNTDEV_QUERY_DEVICE_NAME failed, Status 0x%08lx\n", Status);
        return Status;
    }

    Status = RtlStringCchCopyNW(
        (core::ptr::addr_of_mut!((*Volume).Info.DeviceName)).cast(),
        (_countof!((*Volume).Info.DeviceName)) as _,
        DeviceName.Name.as_ptr(),
        (DeviceName.NameLength as usize) / core::mem::size_of::<u16>(),
    );

    return STATUS_SUCCESS;
}

pub unsafe extern "C" fn InitVolume(
    List: PPARTLIST,
    PartEntry: PPARTENTRY,
) -> PVOLENTRY {
    let mut Volume: PVOLENTRY;

    Volume = (RtlAllocateHeap(
        ProcessHeap,
        HEAP_ZERO_MEMORY,
        (core::mem::size_of::<VOLENTRY>()) as SIZE_T,
    )).cast();
    if Volume.is_null() {
        return core::ptr::null_mut();
    }

    (*Volume).Info.DeviceName[0] = UNICODE_NULL;

    (*Volume).Info.DriveLetter = UNICODE_NULL;
    (*Volume).Info.VolumeLabel[0] = UNICODE_NULL;

    (*Volume).Info.FileSystem[0] = UNICODE_NULL;
    (*Volume).FormatState = Unformatted;
    (*Volume).NeedsCheck = (FALSE) as u8;
    (*Volume).New = (TRUE) as u8;

    if !PartEntry.is_null() {
        (*Volume).PartEntry = PartEntry;
    }
    InsertTailList(&raw mut (*List).VolumesList, &raw mut (*Volume).ListEntry);
    return Volume;
}

unsafe extern "C" fn AddPartitionToDisk(
    mut DiskNumber: ULONG,
    mut DiskEntry: PDISKENTRY,
    mut PartitionIndex: ULONG,
    mut LogicalPartition: BOOLEAN,
) {
    // PARTITION_* / VALID_NTFT come from the bindgen island (sdk/include/psdk/ntdddisk.h)
    // and PARTITION_EISA..PARTITION_IBM from base/setup/lib/utils/partlist.h. A local
    // re-declaration here once carried PARTITION_OLD_LINUX = 0x82 (C: 0x43) and
    // PARTITION_ISO9660 = 0x90 (C: 0x96), silently un-recognizing those partitions.

    let mut PartitionInfo: PPARTITION_INFORMATION = core::ptr::null_mut();
    let mut PartEntry: PPARTENTRY = core::ptr::null_mut();

    PartitionInfo = (&mut (*(*DiskEntry).LayoutBuffer).PartitionEntry as *mut PARTITION_INFORMATION)
        .offset(PartitionIndex as isize);

    if (*PartitionInfo).PartitionType as i32 == PARTITION_ENTRY_UNUSED as i32 {
        return;
    }

    // Assertion from anchor (macro-expanded)
    if LogicalPartition as i32 != 0
        && ((*PartitionInfo).PartitionType as i32 == PARTITION_EXTENDED as i32
            || (*PartitionInfo).PartitionType as i32 == PARTITION_XINT13_EXTENDED as i32)
    {
        RtlAssert(
            b"!(LogicalPartition && IsContainerPartition(PartitionInfo->PartitionType))\0"
                .as_ptr() as *const i8 as PVOID,
            b"base/setup/lib/utils/partlist.c\0"
                .as_ptr() as *const i8 as PVOID,
            1055,
            core::ptr::null_mut(),
        );
    }

    PartEntry = RtlAllocateHeap(
        ProcessHeap as PVOID,
        HEAP_ZERO_MEMORY,
        (core::mem::size_of::<PARTENTRY>()) as SIZE_T,
    ) as PPARTENTRY;
    if PartEntry.is_null() {
        return;
    }

    (*PartEntry).DiskEntry = DiskEntry;
    (*PartEntry).StartSector.QuadPart = ((*PartitionInfo).StartingOffset.QuadPart as u64)
        .wrapping_div((*DiskEntry).BytesPerSector as u64);
    (*PartEntry).SectorCount.QuadPart = ((*PartitionInfo).PartitionLength.QuadPart as u64)
        .wrapping_div((*DiskEntry).BytesPerSector as u64);
    (*PartEntry).BootIndicator = (*PartitionInfo).BootIndicator;
    (*PartEntry).PartitionType = (*PartitionInfo).PartitionType as u8;
    (*PartEntry).LogicalPartition = LogicalPartition;
    (*PartEntry).IsPartitioned = (TRUE) as u8;
    (*PartEntry).OnDiskPartitionNumber = (*PartitionInfo).PartitionNumber as u32;
    (*PartEntry).PartitionNumber = (*PartitionInfo).PartitionNumber as u32;
    (*PartEntry).PartitionIndex = PartitionIndex;
    InitPartitionDeviceName(PartEntry);

    (*PartEntry).Volume = core::ptr::null_mut();

    if (*PartEntry).PartitionType as i32 == PARTITION_EXTENDED as i32
        || (*PartEntry).PartitionType as i32 == PARTITION_XINT13_EXTENDED as i32
    {
        if LogicalPartition == 0 && (*DiskEntry).ExtendedPartition.is_null() {
            (*DiskEntry).ExtendedPartition = PartEntry;
        }
    } else if ((*PartEntry).PartitionType as i32 & PARTITION_NTFT as i32 != 0
            && ((*PartEntry).PartitionType as i32 & !VALID_NTFT as i32 == PARTITION_HUGE as i32
                || (*PartEntry).PartitionType as i32 & !VALID_NTFT as i32 == PARTITION_IFS as i32
                || (*PartEntry).PartitionType as i32 & !VALID_NTFT as i32 == PARTITION_FAT32 as i32
                || (*PartEntry).PartitionType as i32 & !VALID_NTFT as i32 == PARTITION_FAT32_XINT13 as i32))
        || (*PartEntry).PartitionType as i32 == PARTITION_FAT_12 as i32
        || (*PartEntry).PartitionType as i32 == PARTITION_FAT_16 as i32
        || (*PartEntry).PartitionType as i32 == PARTITION_HUGE as i32
        || (*PartEntry).PartitionType as i32 == PARTITION_IFS as i32
        || (*PartEntry).PartitionType as i32 == PARTITION_FAT32 as i32
        || (*PartEntry).PartitionType as i32 == PARTITION_FAT32_XINT13 as i32
        || (*PartEntry).PartitionType as i32 == PARTITION_XINT13 as i32
        || (*PartEntry).PartitionType as i32 == PARTITION_LINUX as i32
        || (*PartEntry).PartitionType as i32 == PARTITION_OLD_LINUX as i32
        || (*PartEntry).PartitionType as i32 == PARTITION_ISO9660 as i32
        || (*PartEntry).PartitionType as i32 == PARTITION_FREEBSD as i32
        || (*PartEntry).PartitionType as i32 == PARTITION_OPENBSD as i32
        || (*PartEntry).PartitionType as i32 == PARTITION_NETBSD as i32
        || ((*PartEntry).PartitionType as i32 == PARTITION_EISA as i32
            || (*PartEntry).PartitionType as i32 == PARTITION_HIBERNATION as i32
            || (*PartEntry).PartitionType as i32 == PARTITION_DIAGNOSTIC as i32
            || (*PartEntry).PartitionType as i32 == PARTITION_DELL as i32
            || (*PartEntry).PartitionType as i32 == PARTITION_IBM as i32)
        || (*PartEntry).PartitionType as i32 == PARTITION_EISA as i32
        || (*PartEntry).PartitionType as i32 == PARTITION_HIBERNATION as i32
        || (*PartEntry).PartitionType as i32 == PARTITION_DIAGNOSTIC as i32
        || (*PartEntry).PartitionType as i32 == PARTITION_DELL as i32
        || (*PartEntry).PartitionType as i32 == PARTITION_IBM as i32
    {
        let mut Volume: PVOLENTRY = core::ptr::null_mut();
        let mut Status: NTSTATUS = 0;

        if !((*PartEntry).PartitionNumber != 0) {
            RtlAssert(
                b"PartEntry->PartitionNumber != 0\0".as_ptr() as *const i8 as PVOID,
                b"base/setup/lib/utils/partlist.c\0"
                    .as_ptr() as *const i8 as PVOID,
                1092,
                core::ptr::null_mut(),
            );
        }

        Volume = InitVolume((*DiskEntry).PartList, PartEntry);
        if Volume.is_null() {
            // DPRINT1 expansion: Couldn't allocate...
            if DbgPrint(
                b"(%s:%d) Couldn't allocate a volume for device '%S'\n\0".as_ptr() as PCSTR,
                b"base/setup/lib/utils/partlist.c\0".as_ptr() as *const i8,
                1100,
                (*PartEntry).DeviceName.as_ptr(),
            ) != 0 {
                DbgPrint(
                    b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
                    b"base/setup/lib/utils/partlist.c\0".as_ptr() as *const i8,
                    1100,
                );
            }
        } else {
            (*PartEntry).Volume = Volume;
            InitVolumeDeviceName(Volume, core::ptr::null());
            (*Volume).New = (FALSE) as u8;
            Status = MountVolume(&mut (*Volume).Info as *mut _, (*PartEntry).PartitionType);
            if !(Status >= 0) {
                // DPRINT1 expansion: Failed to mount...
                if DbgPrint(
                    b"(%s:%d) Failed to mount volume '%S', Status 0x%08lx\n\0".as_ptr() as PCSTR,
                    b"base/setup/lib/utils/partlist.c\0".as_ptr() as *const i8,
                    1112,
                    (*Volume).Info.DeviceName.as_ptr(),
                    Status,
                ) != 0 {
                    DbgPrint(
                        b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
                        b"base/setup/lib/utils/partlist.c\0".as_ptr() as *const i8,
                        1112,
                    );
                }
            }

            if (*Volume).Info.FileSystem[0 as usize] as i32 == 0 {
                (*Volume).FormatState = UnknownFormat;
            } else if (*Volume).Info.FileSystem[0 as usize] as i32 == 'R' as i32
                && (*Volume).Info.FileSystem[1 as usize] as i32 == 'A' as i32
                && (*Volume).Info.FileSystem[2 as usize] as i32 == 'W' as i32
                && (*Volume).Info.FileSystem[3 as usize] as i32
                    == 0 as i32
            {
                (*Volume).FormatState = Unformatted;
            } else {
                (*Volume).FormatState = Formatted;
            }
        }
    } else {
        // DPRINT1 expansion: not recognized
        if DbgPrint(
            b"(%s:%d) Disk %lu Partition %lu is not recognized (Type 0x%02x)\n\0".as_ptr() as PCSTR,
            b"base/setup/lib/utils/partlist.c\0".as_ptr() as *const i8,
            1142,
            (*DiskEntry).DiskNumber,
            (*PartEntry).PartitionNumber,
            (*PartEntry).PartitionType as i32,
        ) != 0 {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
                b"base/setup/lib/utils/partlist.c\0".as_ptr() as *const i8,
                1142,
            );
        }
    }

    InsertDiskRegion(DiskEntry, PartEntry, LogicalPartition);
}

unsafe extern "C" fn ScanForUnpartitionedDiskSpace(DiskEntry: PDISKENTRY) {
    let mut StartSector: ULONGLONG;
    let mut SectorCount: ULONGLONG;
    let mut LastStartSector: ULONGLONG;
    let mut LastSectorCount: ULONGLONG;
    let mut LastUnusedSectorCount: ULONGLONG;
    let mut PartEntry: PPARTENTRY;
    let mut NewPartEntry: PPARTENTRY;
    let mut Entry: PLIST_ENTRY;

    DPRINT!("ScanForUnpartitionedDiskSpace()\n");

    if IsListEmpty(&mut (*DiskEntry).PrimaryPartListHead) != false {
        DPRINT1!("No primary partition!\n");

        if (*DiskEntry).SectorAlignment < 2048 {
            StartSector = 2048;
        } else {
            StartSector = (*DiskEntry).SectorAlignment as ULONGLONG;
        }
        SectorCount = AlignDown((*DiskEntry).SectorCount.QuadPart, (*DiskEntry).SectorAlignment).wrapping_sub(StartSector);

        NewPartEntry = CreateInsertBlankRegion(
            DiskEntry,
            &mut (*DiskEntry).PrimaryPartListHead,
            StartSector,
            SectorCount,
            (FALSE) as BOOLEAN,
        );
        if NewPartEntry.is_null() {
            DPRINT1!("Failed to create a new empty region for full disk space!\n");
        }

        return;
    }

    if (*DiskEntry).SectorAlignment < 2048 {
        LastStartSector = 2048;
    } else {
        LastStartSector = (*DiskEntry).SectorAlignment as ULONGLONG;
    }
    LastSectorCount = 0;
    LastUnusedSectorCount = 0;

    Entry = (*DiskEntry).PrimaryPartListHead.Flink;
    while Entry != &mut (*DiskEntry).PrimaryPartListHead {
        PartEntry = CONTAINING_RECORD!(Entry, PARTENTRY, ListEntry);

        if ((*PartEntry).PartitionType) as u32 != PARTITION_ENTRY_UNUSED ||
            (*PartEntry).SectorCount.QuadPart != 0
        {
            LastUnusedSectorCount =
                (*PartEntry).StartSector.QuadPart.wrapping_sub(LastStartSector.wrapping_add(LastSectorCount));

            if (*PartEntry).StartSector.QuadPart > LastStartSector.wrapping_add(LastSectorCount) &&
                LastUnusedSectorCount >= (*DiskEntry).SectorAlignment as ULONGLONG
            {
                DPRINT!("Unpartitioned disk space %I64u sectors\n", LastUnusedSectorCount);

                StartSector = LastStartSector.wrapping_add(LastSectorCount);
                SectorCount = AlignDown(StartSector.wrapping_add(LastUnusedSectorCount), (*DiskEntry).SectorAlignment).wrapping_sub(StartSector);

                NewPartEntry = CreateInsertBlankRegion(
                    DiskEntry,
                    &mut (*PartEntry).ListEntry,
                    StartSector,
                    SectorCount,
                    (FALSE) as BOOLEAN,
                );
                if NewPartEntry.is_null() {
                    DPRINT1!("Failed to create a new empty region for disk space!\n");
                    return;
                }
            }

            LastStartSector = (*PartEntry).StartSector.QuadPart;
            LastSectorCount = (*PartEntry).SectorCount.QuadPart;
        }

        Entry = (*Entry).Flink;
    }

    if LastStartSector.wrapping_add(LastSectorCount) < (*DiskEntry).SectorCount.QuadPart {
        LastUnusedSectorCount = AlignDown(
            (*DiskEntry).SectorCount.QuadPart.wrapping_sub(LastStartSector.wrapping_add(LastSectorCount)),
            (*DiskEntry).SectorAlignment,
        );

        if LastUnusedSectorCount >= (*DiskEntry).SectorAlignment as ULONGLONG {
            DPRINT!("Unpartitioned disk space: %I64u sectors\n", LastUnusedSectorCount);

            StartSector = LastStartSector.wrapping_add(LastSectorCount);
            SectorCount = AlignDown(StartSector.wrapping_add(LastUnusedSectorCount), (*DiskEntry).SectorAlignment).wrapping_sub(StartSector);

            NewPartEntry = CreateInsertBlankRegion(
                DiskEntry,
                &mut (*DiskEntry).PrimaryPartListHead,
                StartSector,
                SectorCount,
                (FALSE) as BOOLEAN,
            );
            if NewPartEntry.is_null() {
                DPRINT1!("Failed to create a new empty region for trailing disk space!\n");
                return;
            }
        }
    }

    if !(*DiskEntry).ExtendedPartition.is_null() {
        if IsListEmpty(&mut (*DiskEntry).LogicalPartListHead) != false {
            DPRINT1!("No logical partition!\n");

            AddLogicalDiskSpace(DiskEntry);
            return;
        }

        LastStartSector = (*(*DiskEntry).ExtendedPartition).StartSector.QuadPart.wrapping_add((*DiskEntry).SectorAlignment as ULONGLONG);
        LastSectorCount = 0;
        LastUnusedSectorCount = 0;

        Entry = (*DiskEntry).LogicalPartListHead.Flink;
        while Entry != &mut (*DiskEntry).LogicalPartListHead {
            PartEntry = CONTAINING_RECORD!(Entry, PARTENTRY, ListEntry);

            if ((*PartEntry).PartitionType) as u32 != PARTITION_ENTRY_UNUSED ||
                (*PartEntry).SectorCount.QuadPart != 0
            {
                LastUnusedSectorCount =
                    (*PartEntry).StartSector.QuadPart
                        .wrapping_sub((*DiskEntry).SectorAlignment as ULONGLONG)
                        .wrapping_sub(LastStartSector.wrapping_add(LastSectorCount));

                if (*PartEntry).StartSector.QuadPart.wrapping_sub((*DiskEntry).SectorAlignment as ULONGLONG) > LastStartSector.wrapping_add(LastSectorCount) &&
                    LastUnusedSectorCount >= (*DiskEntry).SectorAlignment as ULONGLONG
                {
                    DPRINT!("Unpartitioned disk space %I64u sectors\n", LastUnusedSectorCount);

                    StartSector = LastStartSector.wrapping_add(LastSectorCount);
                    SectorCount = AlignDown(StartSector.wrapping_add(LastUnusedSectorCount), (*DiskEntry).SectorAlignment).wrapping_sub(StartSector);

                    NewPartEntry = CreateInsertBlankRegion(
                        DiskEntry,
                        &mut (*PartEntry).ListEntry,
                        StartSector,
                        SectorCount,
                        (TRUE) as BOOLEAN,
                    );
                    if NewPartEntry.is_null() {
                        DPRINT1!("Failed to create a new empty region for extended partition space!\n");
                        return;
                    }
                }

                LastStartSector = (*PartEntry).StartSector.QuadPart;
                LastSectorCount = (*PartEntry).SectorCount.QuadPart;
            }

            Entry = (*Entry).Flink;
        }

        if LastStartSector.wrapping_add(LastSectorCount) <
            (*(*DiskEntry).ExtendedPartition).StartSector.QuadPart.wrapping_add((*(*DiskEntry).ExtendedPartition).SectorCount.QuadPart)
        {
            LastUnusedSectorCount = AlignDown(
                (*(*DiskEntry).ExtendedPartition).StartSector.QuadPart
                    .wrapping_add((*(*DiskEntry).ExtendedPartition).SectorCount.QuadPart)
                    .wrapping_sub(LastStartSector.wrapping_add(LastSectorCount)),
                (*DiskEntry).SectorAlignment,
            );

            if LastUnusedSectorCount >= (*DiskEntry).SectorAlignment as ULONGLONG {
                DPRINT!("Unpartitioned disk space: %I64u sectors\n", LastUnusedSectorCount);

                StartSector = LastStartSector.wrapping_add(LastSectorCount);
                SectorCount = AlignDown(StartSector.wrapping_add(LastUnusedSectorCount), (*DiskEntry).SectorAlignment).wrapping_sub(StartSector);

                NewPartEntry = CreateInsertBlankRegion(
                    DiskEntry,
                    &mut (*DiskEntry).LogicalPartListHead,
                    StartSector,
                    SectorCount,
                    (TRUE) as BOOLEAN,
                );
                if NewPartEntry.is_null() {
                    DPRINT1!("Failed to create a new empty region for extended partition space!\n");
                    return;
                }
            }
        }
    }

    DPRINT!("ScanForUnpartitionedDiskSpace() done\n");
}

unsafe extern "C" fn SetDiskSignature(
    mut List: PPARTLIST,
    mut DiskEntry: PDISKENTRY,
) {
    let mut SystemTime: LARGE_INTEGER = LARGE_INTEGER {
        QuadPart: 0,
    };
    let mut TimeFields: TIME_FIELDS = TIME_FIELDS {
        Year: 0,
        Month: 0,
        Day: 0,
        Hour: 0,
        Minute: 0,
        Second: 0,
        Milliseconds: 0,
        Weekday: 0,
    };
    let mut Entry2: PLIST_ENTRY = core::ptr::null_mut();
    let mut DiskEntry2: PDISKENTRY = core::ptr::null_mut();
    let mut Buffer: PUCHAR = core::ptr::null_mut();

    if (*DiskEntry).DiskStyle as u32 == PARTITION_STYLE_GPT {
        DPRINT!("GPT-partitioned disk detected, not currently supported by SETUP!\n");
        return;
    }

    Buffer = core::ptr::addr_of_mut!((*(*DiskEntry).LayoutBuffer).Signature) as PUCHAR;

    // C is `while (TRUE) { ... if (Signature == 0) continue; ... if (Entry2 == head) break; }`,
    // not a do-while: its `continue` re-rolls the signature from the top, exactly as here.
    loop { // nolint: dowhile-continue (C is while (TRUE); continue goes to the top)
        NtQuerySystemTime(core::ptr::addr_of_mut!(SystemTime));
        RtlTimeToTimeFields(
            core::ptr::addr_of_mut!(SystemTime),
            core::ptr::addr_of_mut!(TimeFields),
        );

        *Buffer.offset(0 as isize) =
            ((TimeFields.Year as i32 & 0xFF) as u8 as i32
                + (TimeFields.Hour as i32 & 0xFF) as u8 as i32) as u8;
        *Buffer.offset(1 as isize) =
            ((TimeFields.Year as i32 >> 8) as u8 as i32
                + (TimeFields.Minute as i32 & 0xFF) as u8 as i32) as u8;
        *Buffer.offset(2 as isize) =
            ((TimeFields.Month as i32 & 0xFF) as u8 as i32
                + (TimeFields.Second as i32 & 0xFF) as u8 as i32) as u8;
        *Buffer.offset(3 as isize) =
            ((TimeFields.Day as i32 & 0xFF) as u8 as i32
                + (TimeFields.Milliseconds as i32 & 0xFF) as u8 as i32) as u8;

        if (*(*DiskEntry).LayoutBuffer).Signature == 0 {
            continue;
        }

        Entry2 = (*List).DiskListHead.Flink;
        while Entry2 != core::ptr::addr_of_mut!((*List).DiskListHead) {
            DiskEntry2 = CONTAINING_RECORD!(Entry2, DISKENTRY, ListEntry);

            if (*DiskEntry2).DiskStyle as u32 == PARTITION_STYLE_GPT {
                DPRINT!("GPT-partitioned disk detected, not currently supported by SETUP!\n");
                Entry2 = (*Entry2).Flink;
                continue;
            }

            if DiskEntry != DiskEntry2
                && (*(*DiskEntry).LayoutBuffer).Signature
                    == (*(*DiskEntry2).LayoutBuffer).Signature
            {
                break;
            }

            Entry2 = (*Entry2).Flink;
        }

        if Entry2 == core::ptr::addr_of_mut!((*List).DiskListHead) {
            break;
        }
    }
}

unsafe extern "C" fn UpdateDiskSignatures(mut List: PPARTLIST) {
    let mut Entry: PLIST_ENTRY = core::ptr::null_mut();
    let mut DiskEntry: PDISKENTRY = core::ptr::null_mut();

    Entry = (*List).DiskListHead.Flink;
    while Entry != core::ptr::addr_of_mut!((*List).DiskListHead) {
        DiskEntry = CONTAINING_RECORD!(Entry, DISKENTRY, ListEntry);

        if (*DiskEntry).DiskStyle == PARTITION_STYLE_GPT {
            DPRINT!("GPT-partitioned disk detected, not currently supported by SETUP!\n");
            Entry = (*Entry).Flink;
            continue;
        }

        if !(*DiskEntry).LayoutBuffer.is_null()
            && (*(*DiskEntry).LayoutBuffer).Signature == 0
        {
            SetDiskSignature(List, DiskEntry);
            (*(*DiskEntry).LayoutBuffer).PartitionEntry[0].RewritePartition = (TRUE) as u8;
        }

        Entry = (*Entry).Flink;
    }
}

unsafe extern "C" fn UpdateHwDiskNumbers(mut List: PPARTLIST) {
    let mut ListEntry: PLIST_ENTRY = core::ptr::null_mut();
    let mut BiosDiskEntry: PBIOSDISKENTRY = core::ptr::null_mut();
    let mut DiskEntry: PDISKENTRY = core::ptr::null_mut();
    let mut HwAdapterNumber: u32 = 0;
    let mut HwControllerNumber: u32 = 0;
    let mut RemovableDiskCount: u32 = 0;

    ListEntry = (*List).BiosDiskListHead.Flink;
    while ListEntry != core::ptr::addr_of_mut!((*List).BiosDiskListHead) {
        BiosDiskEntry = CONTAINING_RECORD!(ListEntry, BIOSDISKENTRY, ListEntry);
        DiskEntry = (*BiosDiskEntry).DiskEntry;

        if HwAdapterNumber != (*BiosDiskEntry).AdapterNumber
            || HwControllerNumber != (*BiosDiskEntry).ControllerNumber
        {
            HwAdapterNumber = (*BiosDiskEntry).AdapterNumber;
            HwControllerNumber = (*BiosDiskEntry).ControllerNumber;
            RemovableDiskCount = 0;
        }

        if !DiskEntry.is_null() {
            if (*DiskEntry).MediaType == RemovableMedia {
                RemovableDiskCount = RemovableDiskCount.wrapping_add(1);
                (*DiskEntry).HwFixedDiskNumber = 0;
            } else {
                (*DiskEntry).HwFixedDiskNumber = (*BiosDiskEntry)
                    .DiskNumber
                    .wrapping_sub(RemovableDiskCount);
            }
        } else {
            DbgPrint(
                b"BIOS disk %lu is not recognized by NTOS!\n\0".as_ptr() as PCSTR,
                (*BiosDiskEntry).DiskNumber,
            );
        }

        ListEntry = (*ListEntry).Flink;
    }
}

unsafe extern "C" fn AddDiskToList(
    mut FileHandle: HANDLE,
    mut DiskNumber: ULONG,
    mut List: PPARTLIST,
) {
    let mut DiskGeometry: DISK_GEOMETRY = core::mem::zeroed();
    let mut ScsiAddress: SCSI_ADDRESS = core::mem::zeroed();
    let mut DiskEntry: PDISKENTRY = core::ptr::null_mut();
    let mut Iosb: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Status: NTSTATUS = 0;
    let mut Mbr: PPARTITION_SECTOR = core::ptr::null_mut();
    let mut Buffer: PULONG = core::ptr::null_mut();
    let mut FileOffset: LARGE_INTEGER = core::mem::zeroed();
    let mut Identifier: [u16; 20] = [0; 20];
    let mut Checksum: ULONG = 0;
    let mut Signature: ULONG = 0;
    let mut i: ULONG = 0;
    let mut ListEntry: PLIST_ENTRY = core::ptr::null_mut();
    let mut BiosDiskEntry: PBIOSDISKENTRY = core::ptr::null_mut();
    let mut LayoutBufferSize: ULONG = 0;
    let mut NewLayoutBuffer: PDRIVE_LAYOUT_INFORMATION = core::ptr::null_mut();

    Status = NtDeviceIoControlFile(
        FileHandle,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        &mut Iosb,
        IOCTL_DISK_GET_DRIVE_GEOMETRY,
        core::ptr::null_mut(),
        0,
        &mut DiskGeometry as *mut _ as PVOID,
        core::mem::size_of::<DISK_GEOMETRY>() as ULONG,
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        return;
    }

    if DiskGeometry.MediaType != FixedMedia &&
        DiskGeometry.MediaType != RemovableMedia
    {
        return;
    }

    Status = NtDeviceIoControlFile(
        FileHandle,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        &mut Iosb,
        IOCTL_SCSI_GET_ADDRESS,
        core::ptr::null_mut(),
        0,
        &mut ScsiAddress as *mut _ as PVOID,
        core::mem::size_of::<SCSI_ADDRESS>() as ULONG,
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        return;
    }

    Mbr = RtlAllocateHeap(
        ProcessHeap,
        0,
        DiskGeometry.BytesPerSector as SIZE_T,
    ) as PPARTITION_SECTOR;
    if Mbr.is_null() {
        return;
    }

    FileOffset.QuadPart = 0;
    Status = NtReadFile(
        FileHandle,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        &mut Iosb,
        Mbr as PVOID,
        DiskGeometry.BytesPerSector as ULONG,
        &mut FileOffset,
        core::ptr::null_mut(),
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        RtlFreeHeap(ProcessHeap, 0, Mbr as PVOID);
        DPRINT1!("NtReadFile failed, status=%x\n", Status);
        return;
    }
    Signature = (*Mbr).Signature;

    Checksum = 0;
    Buffer = Mbr as PULONG;
    i = 0;
    while i < 128 {
        Checksum = Checksum.wrapping_add(*Buffer.offset(i as isize));
        i = i.wrapping_add(1);
    }
    Checksum = (!Checksum).wrapping_add(1);

    RtlStringCchPrintfW!(
        Identifier.as_mut_ptr(),
        ARRAYSIZE!(Identifier),
        RUSTCONV_WSTR_A603842E44A2.as_ptr(),
        Checksum,
        Signature,
        if ((*Mbr).Magic) as u32 == PARTITION_MAGIC { 0x0041u16 } else { 0x0058u16 } as core::ffi::c_uint,
    );
    DPRINT!("Identifier: %S\n", Identifier.as_ptr());

    DiskEntry = RtlAllocateHeap(
        ProcessHeap,
        HEAP_ZERO_MEMORY,
        core::mem::size_of::<DISKENTRY>() as SIZE_T,
    ) as PDISKENTRY;
    if DiskEntry.is_null() {
        RtlFreeHeap(ProcessHeap, 0, Mbr as PVOID);
        DPRINT1!("Failed to allocate a new disk entry.\n");
        return;
    }

    (*DiskEntry).PartList = List;

    (*DiskEntry).MediaType = DiskGeometry.MediaType;
    if (*DiskEntry).MediaType == RemovableMedia {
        DPRINT1!("Disk %lu of identifier '%S' is removable\n", DiskNumber, Identifier.as_ptr());
    } else {
        DPRINT1!("Disk %lu of identifier '%S' is fixed\n", DiskNumber, Identifier.as_ptr());
    }

    (*DiskEntry).BiosFound = (FALSE) as u8;

    if ((*Mbr).Magic) as u32 != PARTITION_MAGIC {
        (*DiskEntry).DiskStyle = PARTITION_STYLE_RAW;
    } else if ((*Mbr).Partition[0].PartitionType) as u32 == EFI_PMBR_OSTYPE_EFI &&
        (*Mbr).Partition[1].PartitionType == 0 &&
        (*Mbr).Partition[2].PartitionType == 0 &&
        (*Mbr).Partition[3].PartitionType == 0
    {
        (*DiskEntry).DiskStyle = PARTITION_STYLE_GPT;
    } else {
        (*DiskEntry).DiskStyle = PARTITION_STYLE_MBR;
    }

    RtlFreeHeap(ProcessHeap, 0, Mbr as PVOID);

    ListEntry = (*List).BiosDiskListHead.Flink;
    while ListEntry != &mut (*List).BiosDiskListHead {
        BiosDiskEntry = CONTAINING_RECORD!(ListEntry, BIOSDISKENTRY, ListEntry);
        if (*BiosDiskEntry).Signature == Signature &&
            (*BiosDiskEntry).Checksum == Checksum &&
            (*BiosDiskEntry).DiskEntry.is_null()
        {
            if (*DiskEntry).BiosFound == 0 {
                (*DiskEntry).HwAdapterNumber = (*BiosDiskEntry).AdapterNumber;
                (*DiskEntry).HwControllerNumber = (*BiosDiskEntry).ControllerNumber;
                (*DiskEntry).HwDiskNumber = (*BiosDiskEntry).DiskNumber;

                if (*DiskEntry).MediaType == RemovableMedia {
                    (*DiskEntry).HwFixedDiskNumber = 0;
                } else {
                    (*DiskEntry).HwFixedDiskNumber = (*BiosDiskEntry).DiskNumber;
                }

                (*DiskEntry).BiosFound = (TRUE) as u8;
                (*BiosDiskEntry).DiskEntry = DiskEntry;
                break;
            } else {
                DPRINT1!("Disk %lu of identifier '%S' has already been found?!\n", DiskNumber, Identifier.as_ptr());
            }
        }
        ListEntry = (*ListEntry).Flink;
    }

    if (*DiskEntry).BiosFound == 0 {
        DPRINT1!("WARNING: Setup could not find a matching BIOS disk entry. Disk %lu may not be bootable by the BIOS!\n", DiskNumber);
    }

    (*DiskEntry).Cylinders = (DiskGeometry.Cylinders.QuadPart) as u64;
    (*DiskEntry).TracksPerCylinder = DiskGeometry.TracksPerCylinder;
    (*DiskEntry).SectorsPerTrack = DiskGeometry.SectorsPerTrack;
    (*DiskEntry).BytesPerSector = DiskGeometry.BytesPerSector;

    DPRINT!("Cylinders %I64u\n", (*DiskEntry).Cylinders);
    DPRINT!("TracksPerCylinder %lu\n", (*DiskEntry).TracksPerCylinder);
    DPRINT!("SectorsPerTrack %lu\n", (*DiskEntry).SectorsPerTrack);
    DPRINT!("BytesPerSector %lu\n", (*DiskEntry).BytesPerSector);

    (*DiskEntry).SectorCount.QuadPart = (DiskGeometry.Cylinders.QuadPart *
        (DiskGeometry.TracksPerCylinder as ULONGLONG) as i64 *
        (DiskGeometry.SectorsPerTrack as ULONGLONG) as i64) as u64;

    (*DiskEntry).SectorAlignment = DiskGeometry.SectorsPerTrack;
    (*DiskEntry).CylinderAlignment = DiskGeometry.TracksPerCylinder *
        DiskGeometry.SectorsPerTrack;

    DPRINT!("SectorCount %I64u\n", (*DiskEntry).SectorCount.QuadPart);
    DPRINT!("SectorAlignment %lu\n", (*DiskEntry).SectorAlignment);

    (*DiskEntry).DiskNumber = DiskNumber;
    (*DiskEntry).Port = (ScsiAddress.PortNumber) as _;
    (*DiskEntry).Bus = (ScsiAddress.PathId) as _;
    (*DiskEntry).Id = (ScsiAddress.TargetId) as _;

    GetDriverName(DiskEntry);

    InitializeListHead(&mut (*DiskEntry).PrimaryPartListHead);
    InitializeListHead(&mut (*DiskEntry).LogicalPartListHead);

    {
        let mut current: PLIST_ENTRY = (*List).DiskListHead.Flink;
        while current != &mut (*List).DiskListHead {
            if (*CONTAINING_RECORD!(current, DISKENTRY, ListEntry)).DiskNumber >= (*DiskEntry).DiskNumber {
                break;
            }
            current = (*current).Flink;
        }
        InsertTailList(current, &mut (*DiskEntry).ListEntry);
    }

    if (*DiskEntry).DiskStyle == PARTITION_STYLE_GPT {
        DPRINT1!("GPT-partitioned disk detected, not currently supported by SETUP!\n");
        return;
    }

    LayoutBufferSize = core::mem::size_of::<DRIVE_LAYOUT_INFORMATION>() as ULONG +
        ((4 - ANYSIZE_ARRAY) * core::mem::size_of::<PARTITION_INFORMATION>() as ULONG);
    (*DiskEntry).LayoutBuffer = RtlAllocateHeap(
        ProcessHeap,
        HEAP_ZERO_MEMORY,
        LayoutBufferSize as SIZE_T,
    ) as PDRIVE_LAYOUT_INFORMATION;
    if (*DiskEntry).LayoutBuffer.is_null() {
        DPRINT1!("Failed to allocate the disk layout buffer!\n");
        return;
    }

    loop {
        DPRINT1!("Buffer size: %lu\n", LayoutBufferSize);
        Status = NtDeviceIoControlFile(
            FileHandle,
            core::ptr::null_mut(),
            None,
            core::ptr::null_mut(),
            &mut Iosb,
            (IOCTL_DISK_GET_DRIVE_LAYOUT) as ULONG,
            core::ptr::null_mut(),
            0,
            (*DiskEntry).LayoutBuffer as PVOID,
            LayoutBufferSize,
        );
        if NT_SUCCESS!(Status) != 0 {
            break;
        }

        if Status != STATUS_BUFFER_TOO_SMALL {
            DPRINT1!("NtDeviceIoControlFile() failed (Status: 0x%08lx)\n", Status);
            return;
        }

        LayoutBufferSize += 4 * core::mem::size_of::<PARTITION_INFORMATION>() as ULONG;
        NewLayoutBuffer = RtlReAllocateHeap(
            ProcessHeap,
            HEAP_ZERO_MEMORY,
            (*DiskEntry).LayoutBuffer as PVOID,
            LayoutBufferSize as SIZE_T,
        ) as PDRIVE_LAYOUT_INFORMATION;
        if NewLayoutBuffer.is_null() {
            DPRINT1!("Failed to reallocate the disk layout buffer!\n");
            return;
        }

        (*DiskEntry).LayoutBuffer = NewLayoutBuffer;
    }

    DPRINT1!("PartitionCount: %lu\n", (*(*DiskEntry).LayoutBuffer).PartitionCount);

    if IsSuperFloppy(DiskEntry) != 0 {
        DPRINT1!("Disk %lu is a super-floppy\n", DiskNumber);
    }

    if (*(*DiskEntry).LayoutBuffer).PartitionEntry[0].StartingOffset.QuadPart != 0 &&
        (*(*DiskEntry).LayoutBuffer).PartitionEntry[0].PartitionLength.QuadPart != 0 &&
        ((*(*DiskEntry).LayoutBuffer).PartitionEntry[0].PartitionType) as u32 != PARTITION_ENTRY_UNUSED
    {
        if ((*(*DiskEntry).LayoutBuffer).PartitionEntry[0].StartingOffset.QuadPart / (*DiskEntry).BytesPerSector as LONGLONG) % (*DiskEntry).SectorsPerTrack as LONGLONG == 0 {
            DPRINT!("Use %lu Sector alignment!\n", (*DiskEntry).SectorsPerTrack);
        } else if (*(*DiskEntry).LayoutBuffer).PartitionEntry[0].StartingOffset.QuadPart % (1024 * 1024) as LONGLONG == 0 {
            DPRINT1!("Use megabyte (%lu Sectors) alignment!\n", (1024 * 1024) / (*DiskEntry).BytesPerSector);
        } else {
            DPRINT1!("No matching alignment found! Partition 1 starts at %I64u\n", (*(*DiskEntry).LayoutBuffer).PartitionEntry[0].StartingOffset.QuadPart);
        }
    } else {
        DPRINT1!("No valid partition table found! Use megabyte (%lu Sectors) alignment!\n", (1024 * 1024) / (*DiskEntry).BytesPerSector);
    }

    if (*(*DiskEntry).LayoutBuffer).PartitionCount == 0 {
        (*DiskEntry).NewDisk = (TRUE) as u8;
        (*(*DiskEntry).LayoutBuffer).PartitionCount = 4;

        i = 0;
        while i < 4 {
            (*{ let flex: *mut [_; 1] = core::ptr::addr_of_mut!((*(*DiskEntry).LayoutBuffer).PartitionEntry); core::ptr::addr_of_mut!((*flex)[0]) }.add(i as usize)).RewritePartition = (TRUE) as u8;
            i = i.wrapping_add(1);
        }
    } else {
        i = 0;
        while i < 4 {
            AddPartitionToDisk(DiskNumber, DiskEntry, i, (FALSE) as BOOLEAN);
            i = i.wrapping_add(1);
        }

        i = 4;
        while i < (*(*DiskEntry).LayoutBuffer).PartitionCount {
            AddPartitionToDisk(DiskNumber, DiskEntry, i, (TRUE) as BOOLEAN);
            i += 4;
        }
    }

    ScanForUnpartitionedDiskSpace(DiskEntry);
}

pub unsafe extern "C" fn GetSystemDisk(List: PPARTLIST) -> PDISKENTRY {
    let mut Entry: PLIST_ENTRY;
    let mut DiskEntry: PDISKENTRY = core::mem::zeroed();

    if IsListEmpty(&raw mut (*List).DiskListHead) != false {
        return core::ptr::null_mut();
    }

    if !(*List).SystemPartition.is_null() {
        return (*(*List).SystemPartition).DiskEntry;
    }

    Entry = (*List).DiskListHead.Flink;
    while Entry != &raw mut (*List).DiskListHead {
        DiskEntry = CONTAINING_RECORD!(Entry, DISKENTRY, ListEntry);

        if (*DiskEntry).MediaType == FixedMedia && (*DiskEntry).BiosFound != 0 {
            break;
        }
        Entry = (*Entry).Flink;
    }
    if Entry == &raw mut (*List).DiskListHead {
        return core::ptr::null_mut();
    }

    if (*DiskEntry).DiskStyle == PARTITION_STYLE_GPT {
        DPRINT1!("System disk -- GPT-partitioned disk detected, not currently supported by SETUP!\n");
    }

    return DiskEntry;
}

#[no_mangle]
pub unsafe extern "C" fn IsPartitionActive(PartEntry: PPARTENTRY) -> BOOLEAN {
    if IsContainerPartition!((*PartEntry).PartitionType) != false {
        return (FALSE) as u8;
    }
    if (*PartEntry).IsPartitioned != 0 && (*PartEntry).BootIndicator != 0 {
        return (TRUE) as u8;
    }
    return (FALSE) as u8;
}

pub unsafe extern "C" fn GetActiveDiskPartition(DiskEntry: PDISKENTRY) -> PPARTENTRY {
    let mut ListEntry: PLIST_ENTRY;
    let mut PartEntry: PPARTENTRY;
    let mut ActivePartition: PPARTENTRY = core::ptr::null_mut();

    if DiskEntry.is_null() {
        return core::ptr::null_mut();
    }

    if IsListEmpty(&raw mut (*DiskEntry).PrimaryPartListHead) != false {
        return core::ptr::null_mut();
    }

    if (*DiskEntry).DiskStyle == PARTITION_STYLE_GPT {
        DPRINT1!("GPT-partitioned disk detected, not currently supported by SETUP!\n");
        return core::ptr::null_mut();
    }

    ListEntry = (*DiskEntry).PrimaryPartListHead.Flink;
    while ListEntry != &raw mut (*DiskEntry).PrimaryPartListHead {
        PartEntry = CONTAINING_RECORD!(ListEntry, PARTENTRY, ListEntry);
        if IsPartitionActive(PartEntry) != 0 {
            ActivePartition = PartEntry;

            DPRINT1!(
                "Found active system partition %lu in disk %lu, drive letter %C\n",
                (*PartEntry).PartitionNumber,
                (*DiskEntry).DiskNumber,
                (if (*(*PartEntry).Volume).Info.DriveLetter == 0 {
                    0x002Du16
                } else {
                    (*(*PartEntry).Volume).Info.DriveLetter
                }) as ::core::ffi::c_int,
            );
            break;
        }
        ListEntry = (*ListEntry).Flink;
    }

    if (*DiskEntry).NewDisk != 0 && !ActivePartition.is_null() {
        DPRINT1!("NewDisk TRUE but already existing active partition?\n");
    }

    return ActivePartition;
}

#[no_mangle]
pub unsafe extern "system" fn CreatePartitionList() -> *mut PARTLIST {
    let mut List: *mut PARTLIST = core::ptr::null_mut();
    let mut SystemDisk: *mut DISKENTRY = core::ptr::null_mut();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut Sdi: SYSTEM_DEVICE_INFORMATION = core::mem::zeroed();
    let mut Iosb: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut ReturnSize: u32 = 0;
    let mut Status: i32 = 0;
    let mut DiskNumber: u32 = 0;
    let mut FileHandle: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut Name: _UNICODE_STRING = core::mem::zeroed();
    let mut Buffer: [u16; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    List = RtlAllocateHeap(
        ProcessHeap,
        0,
        (core::mem::size_of::<PARTLIST>()) as SIZE_T,
    ) as *mut PARTLIST;
    if List.is_null() {
        return core::ptr::null_mut();
    }

    (*List).SystemPartition = core::ptr::null_mut();

    InitializeListHead(&raw mut (*List).DiskListHead);
    InitializeListHead(&raw mut (*List).BiosDiskListHead);
    InitializeListHead(&raw mut (*List).VolumesList);
    InitializeListHead(&raw mut (*List).PendingUnmountVolumesList);

    EnumerateBiosDiskEntries(List);

    Status = NtQuerySystemInformation(
        SystemDeviceInformation,
        &raw mut Sdi as *mut _ as *mut core::ffi::c_void,
        core::mem::size_of::<SYSTEM_DEVICE_INFORMATION>() as u32,
        &raw mut ReturnSize,
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT1!(
            b"NtQuerySystemInformation() failed, Status 0x%08lx\n\0".as_ptr() as *const i8,
            Status,
        );
        RtlFreeHeap(ProcessHeap, 0, List as *mut core::ffi::c_void);
        return core::ptr::null_mut();
    }

    DiskNumber = 0;
    while DiskNumber < Sdi.NumberOfDisks {
        RtlStringCchPrintfW!(
            Buffer.as_mut_ptr(),
            ARRAYSIZE!(Buffer),
            RUSTCONV_WSTR_27B3B5E195F7.as_ptr(),
            DiskNumber,
        );
        RtlInitUnicodeString(&raw mut Name, Buffer.as_ptr());

        ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
        ObjectAttributes.RootDirectory = core::ptr::null_mut();
        ObjectAttributes.ObjectName = &raw mut Name;
        ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
        ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
        ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

        Status = NtOpenFile(
            &raw mut FileHandle,
            FILE_READ_DATA | FILE_READ_ATTRIBUTES | SYNCHRONIZE,
            &raw mut ObjectAttributes,
            &raw mut Iosb,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            FILE_SYNCHRONOUS_IO_NONALERT,
        );
        if NT_SUCCESS!(Status) != 0 {
            AddDiskToList(FileHandle, DiskNumber, List);
            NtClose(FileHandle);
        }

        DiskNumber = DiskNumber.wrapping_add(1);
    }

    UpdateDiskSignatures(List);
    UpdateHwDiskNumbers(List);
    AssignDriveLetters(List);

    SystemDisk = GetSystemDisk(List);
    (*List).SystemPartition = if !SystemDisk.is_null() {
        GetActiveDiskPartition(SystemDisk)
    } else {
        core::ptr::null_mut()
    };

    List
}

#[no_mangle]
pub unsafe extern "system" fn DestroyPartitionList(List: *mut PARTLIST) {
    let mut DiskEntry: *mut DISKENTRY;
    let mut BiosDiskEntry: *mut BIOSDISKENTRY;
    let mut PartEntry: *mut PARTENTRY;
    let mut VolumeEntry: *mut VOLENTRY;
    let mut Entry: PLIST_ENTRY;

    while !IsListEmpty(&mut (*List).DiskListHead) {
        Entry = RemoveHeadList(&mut (*List).DiskListHead);
        DiskEntry = CONTAINING_RECORD!(Entry, DISKENTRY, ListEntry);

        RtlFreeUnicodeString(&mut (*DiskEntry).DriverName);

        while !IsListEmpty(&mut (*DiskEntry).PrimaryPartListHead) {
            Entry = RemoveHeadList(&mut (*DiskEntry).PrimaryPartListHead);
            PartEntry = CONTAINING_RECORD!(Entry, PARTENTRY, ListEntry);
            DestroyRegion(PartEntry);
        }

        while !IsListEmpty(&mut (*DiskEntry).LogicalPartListHead) {
            Entry = RemoveHeadList(&mut (*DiskEntry).LogicalPartListHead);
            PartEntry = CONTAINING_RECORD!(Entry, PARTENTRY, ListEntry);
            DestroyRegion(PartEntry);
        }

        if (*DiskEntry).LayoutBuffer != core::ptr::null_mut() {
            RtlFreeHeap(ProcessHeap, 0, ((*DiskEntry).LayoutBuffer).cast());
        }

        RtlFreeHeap(ProcessHeap, 0, (DiskEntry).cast());
    }

    while !IsListEmpty(&mut (*List).BiosDiskListHead) {
        Entry = RemoveHeadList(&mut (*List).BiosDiskListHead);
        BiosDiskEntry = CONTAINING_RECORD!(Entry, BIOSDISKENTRY, ListEntry);
        RtlFreeHeap(ProcessHeap, 0, (BiosDiskEntry).cast());
    }

    while !IsListEmpty(&mut (*List).PendingUnmountVolumesList) {
        Entry = RemoveHeadList(&mut (*List).PendingUnmountVolumesList);
        VolumeEntry = CONTAINING_RECORD!(Entry, VOLENTRY, ListEntry);
        RtlFreeHeap(ProcessHeap, 0, (VolumeEntry).cast());
    }

    RtlFreeHeap(ProcessHeap, 0, (List).cast());
}
#[no_mangle]
pub unsafe extern "C" fn GetDiskByBiosNumber(
    List: *mut PARTLIST,
    HwDiskNumber: u32,
) -> *mut DISKENTRY {
    let mut DiskEntry: *mut DISKENTRY;
    let mut Entry: *mut LIST_ENTRY;

    Entry = (*List).DiskListHead.Flink;
    while Entry != core::ptr::addr_of_mut!((*List).DiskListHead) {
        DiskEntry = CONTAINING_RECORD!(Entry, DISKENTRY, ListEntry);

        if (*DiskEntry).HwDiskNumber == HwDiskNumber {
            return DiskEntry;
        }
        Entry = (*Entry).Flink;
    }

    return core::ptr::null_mut();
}
#[no_mangle]
pub unsafe extern "C" fn GetDiskByNumber(
    List: *mut PARTLIST,
    DiskNumber: u32,
) -> *mut DISKENTRY {
    let mut DiskEntry: *mut DISKENTRY;
    let mut Entry: *mut LIST_ENTRY;

    Entry = (*List).DiskListHead.Flink;
    while Entry != core::ptr::addr_of_mut!((*List).DiskListHead) {
        DiskEntry = CONTAINING_RECORD!(Entry, DISKENTRY, ListEntry);

        if (*DiskEntry).DiskNumber == DiskNumber {
            return DiskEntry;
        }
        Entry = (*Entry).Flink;
    }

    return core::ptr::null_mut();
}
#[no_mangle]
pub unsafe extern "C" fn GetDiskBySCSI(
    List: *mut PARTLIST,
    Port: u16,
    Bus: u16,
    Id: u16,
) -> *mut DISKENTRY {
    let mut DiskEntry: *mut DISKENTRY;
    let mut Entry: *mut LIST_ENTRY;

    Entry = (*List).DiskListHead.Flink;
    while Entry != core::ptr::addr_of_mut!((*List).DiskListHead) {
        DiskEntry = CONTAINING_RECORD!(Entry, DISKENTRY, ListEntry);

        if (*DiskEntry).Port == Port &&
            (*DiskEntry).Bus == Bus &&
            (*DiskEntry).Id == Id
        {
            return DiskEntry;
        }

        Entry = (*Entry).Flink;
    }

    return core::ptr::null_mut();
}
#[no_mangle]
pub unsafe extern "C" fn GetDiskBySignature(
    List: *mut PARTLIST,
    Signature: u32,
) -> *mut DISKENTRY {
    let mut DiskEntry: *mut DISKENTRY;
    let mut Entry: *mut LIST_ENTRY;

    Entry = (*List).DiskListHead.Flink;
    while Entry != core::ptr::addr_of_mut!((*List).DiskListHead) {
        DiskEntry = CONTAINING_RECORD!(Entry, DISKENTRY, ListEntry);

        if (*(*DiskEntry).LayoutBuffer).Signature == Signature {
            return DiskEntry;
        }

        Entry = (*Entry).Flink;
    }

    return core::ptr::null_mut();
}
#[no_mangle]
pub unsafe extern "C" fn GetPartition(
    DiskEntry: PDISKENTRY,
    PartitionNumber: ULONG,
) -> PPARTENTRY {
    let mut PartEntry: PPARTENTRY = core::ptr::null_mut();
    let mut Entry: PLIST_ENTRY;

    if PartitionNumber == 0 {
        return core::ptr::null_mut();
    }

    Entry = (*DiskEntry).PrimaryPartListHead.Flink;
    while Entry != &raw mut (*DiskEntry).PrimaryPartListHead {
        PartEntry = CONTAINING_RECORD!(Entry, PARTENTRY, ListEntry);

        if (*PartEntry).PartitionNumber == PartitionNumber {
            return PartEntry;
        }
        Entry = (*Entry).Flink;
    }

    if (*DiskEntry).DiskStyle == PARTITION_STYLE_GPT {
        return core::ptr::null_mut();
    }

    Entry = (*DiskEntry).LogicalPartListHead.Flink;
    while Entry != &raw mut (*DiskEntry).LogicalPartListHead {
        PartEntry = CONTAINING_RECORD!(Entry, PARTENTRY, ListEntry);

        if (*PartEntry).PartitionNumber == PartitionNumber {
            return PartEntry;
        }
        Entry = (*Entry).Flink;
    }

    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn SelectPartition(
    List: PPARTLIST,
    DiskNumber: ULONG,
    PartitionNumber: ULONG,
) -> PPARTENTRY {
    let DiskEntry: PDISKENTRY;
    let PartEntry: PPARTENTRY;

    DiskEntry = GetDiskByNumber(List, DiskNumber);
    if DiskEntry.is_null() {
        return core::ptr::null_mut();
    }

    PartEntry = GetPartition(DiskEntry, PartitionNumber);
    if PartEntry.is_null() {
        return core::ptr::null_mut();
    }

    return PartEntry;
}

#[no_mangle]
pub unsafe extern "system" fn GetNextPartition(
    List: *mut PARTLIST,
    mut CurrentPart: *mut PARTENTRY,
) -> *mut PARTENTRY {
    let mut DiskListEntry: PLIST_ENTRY;
    let mut PartListEntry: PLIST_ENTRY;
    let mut CurrentDisk: PDISKENTRY = core::mem::zeroed();

    if IsListEmpty(&mut (*List).DiskListHead) != false {
        return core::ptr::null_mut();
    }

    if !CurrentPart.is_null() {
        CurrentDisk = (*CurrentPart).DiskEntry;

        if (*CurrentPart).LogicalPartition != 0 {
            PartListEntry = (*CurrentPart).ListEntry.Flink;
            if PartListEntry != &mut (*CurrentDisk).LogicalPartListHead {
                CurrentPart = CONTAINING_RECORD!(PartListEntry, PARTENTRY, ListEntry);
                return CurrentPart;
            } else {
                PartListEntry = (*(*CurrentDisk).ExtendedPartition).ListEntry.Flink;
                if PartListEntry != &mut (*CurrentDisk).PrimaryPartListHead {
                    CurrentPart = CONTAINING_RECORD!(PartListEntry, PARTENTRY, ListEntry);
                    return CurrentPart;
                }
            }
        } else {
            if (*CurrentPart).IsPartitioned != 0
                && IsContainerPartition!((*CurrentPart).PartitionType)
            {
                PartListEntry = (*CurrentDisk).LogicalPartListHead.Flink;
                if PartListEntry != &mut (*CurrentDisk).LogicalPartListHead {
                    CurrentPart = CONTAINING_RECORD!(PartListEntry, PARTENTRY, ListEntry);
                    return CurrentPart;
                }
            } else {
                PartListEntry = (*CurrentPart).ListEntry.Flink;
                if PartListEntry != &mut (*CurrentDisk).PrimaryPartListHead {
                    CurrentPart = CONTAINING_RECORD!(PartListEntry, PARTENTRY, ListEntry);
                    return CurrentPart;
                }
            }
        }
    }

    DiskListEntry = if !CurrentPart.is_null() {
        (*CurrentDisk).ListEntry.Flink
    } else {
        (*List).DiskListHead.Flink
    };

    while DiskListEntry != &mut (*List).DiskListHead {
        CurrentDisk = CONTAINING_RECORD!(DiskListEntry, DISKENTRY, ListEntry);

        if (*CurrentDisk).DiskStyle == PARTITION_STYLE_GPT {
            DPRINT!("GPT-partitioned disk detected, not currently supported by SETUP!\n");
            DiskListEntry = (*DiskListEntry).Flink;
            continue;
        }

        PartListEntry = (*CurrentDisk).PrimaryPartListHead.Flink;
        if PartListEntry != &mut (*CurrentDisk).PrimaryPartListHead {
            CurrentPart = CONTAINING_RECORD!(PartListEntry, PARTENTRY, ListEntry);
            return CurrentPart;
        }

        DiskListEntry = (*DiskListEntry).Flink;
    }

    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn GetPrevPartition(
    List: *mut PARTLIST,
    mut CurrentPart: *mut PARTENTRY,
) -> *mut PARTENTRY {
    let mut DiskListEntry: *mut LIST_ENTRY;
    let mut PartListEntry: *mut LIST_ENTRY;
    let mut CurrentDisk: *mut DISKENTRY = core::mem::zeroed();

    if IsListEmpty(&mut (*List).DiskListHead) != false {
        return core::ptr::null_mut();
    }

    if !CurrentPart.is_null() {
        CurrentDisk = (*CurrentPart).DiskEntry;

        if (*CurrentPart).LogicalPartition != 0 {
            PartListEntry = (*CurrentPart).ListEntry.Blink;
            if PartListEntry != &mut (*CurrentDisk).LogicalPartListHead {
                CurrentPart = CONTAINING_RECORD!(PartListEntry, PARTENTRY, ListEntry);
            } else {
                CurrentPart = (*CurrentDisk).ExtendedPartition;
            }
            return CurrentPart;
        } else {
            PartListEntry = (*CurrentPart).ListEntry.Blink;
            if PartListEntry != &mut (*CurrentDisk).PrimaryPartListHead {
                CurrentPart = CONTAINING_RECORD!(PartListEntry, PARTENTRY, ListEntry);

                if (*CurrentPart).IsPartitioned != 0
                    && IsContainerPartition!((*CurrentPart).PartitionType)
                {
                    PartListEntry = (*CurrentDisk).LogicalPartListHead.Blink;
                    CurrentPart = CONTAINING_RECORD!(PartListEntry, PARTENTRY, ListEntry);
                }

                return CurrentPart;
            }
        }
    }

    DiskListEntry = if !CurrentPart.is_null() {
        (*CurrentDisk).ListEntry.Blink
    } else {
        (*List).DiskListHead.Blink
    };

    while DiskListEntry != &mut (*List).DiskListHead {
        CurrentDisk = CONTAINING_RECORD!(DiskListEntry, DISKENTRY, ListEntry);

        if (*CurrentDisk).DiskStyle == PARTITION_STYLE_GPT {
            DPRINT!("GPT-partitioned disk detected, not currently supported by SETUP!\n");
            DiskListEntry = (*DiskListEntry).Blink;
            continue;
        }

        PartListEntry = (*CurrentDisk).PrimaryPartListHead.Blink;
        if PartListEntry != &mut (*CurrentDisk).PrimaryPartListHead {
            CurrentPart = CONTAINING_RECORD!(PartListEntry, PARTENTRY, ListEntry);

            if (*CurrentPart).IsPartitioned != 0
                && IsContainerPartition!((*CurrentPart).PartitionType)
            {
                PartListEntry = (*CurrentDisk).LogicalPartListHead.Blink;
                if PartListEntry != &mut (*CurrentDisk).LogicalPartListHead {
                    CurrentPart = CONTAINING_RECORD!(PartListEntry, PARTENTRY, ListEntry);
                    return CurrentPart;
                }
            } else {
                return CurrentPart;
            }
        }

        DiskListEntry = (*DiskListEntry).Blink;
    }

    core::ptr::null_mut()
}

unsafe extern "C" fn IsEmptyLayoutEntry(PartitionInfo: PPARTITION_INFORMATION) -> BOOLEAN {
    return (((*PartitionInfo).StartingOffset.QuadPart == 0
        && (*PartitionInfo).PartitionLength.QuadPart == 0) as i32) as u8;
}

unsafe extern "C" fn IsSamePrimaryLayoutEntry(
    PartitionInfo: PPARTITION_INFORMATION,
    PartEntry: PPARTENTRY,
) -> BOOLEAN {
    return ((*PartitionInfo).StartingOffset.QuadPart as ULONGLONG
        == (*PartEntry)
            .StartSector
            .QuadPart
            .wrapping_mul((*(*PartEntry).DiskEntry).BytesPerSector as ULONGLONG)
        && (*PartitionInfo).PartitionLength.QuadPart as ULONGLONG
            == (*PartEntry)
                .SectorCount
                .QuadPart
                .wrapping_mul((*(*PartEntry).DiskEntry).BytesPerSector as ULONGLONG))
        as ::core::ffi::c_int as BOOLEAN;
}

pub unsafe extern "C" fn GetPartitionCount(PartListHead: PLIST_ENTRY) -> u32 {
    let mut Entry: PLIST_ENTRY;
    let mut PartEntry: *mut PARTENTRY;
    let mut Count: u32 = 0;

    Entry = (*PartListHead).Flink;
    while Entry != PartListHead {
        PartEntry = CONTAINING_RECORD!(Entry, PARTENTRY, ListEntry);
        if (*PartEntry).IsPartitioned != 0 {
            Count = Count.wrapping_add(1);
        }
        Entry = (*Entry).Flink;
    }

    return Count;
}

unsafe extern "C" fn ReAllocateLayoutBuffer(mut DiskEntry: PDISKENTRY) -> BOOLEAN {
    let mut NewLayoutBuffer: PDRIVE_LAYOUT_INFORMATION = core::ptr::null_mut();
    let mut NewPartitionCount: ULONG = 0;
    let mut CurrentPartitionCount: ULONG = 0;
    let mut LayoutBufferSize: ULONG = 0;
    let mut i: ULONG = 0;

    DPRINT1!("ReAllocateLayoutBuffer()\n");

    NewPartitionCount = 4 + (if (*DiskEntry).DiskStyle == PARTITION_STYLE_MBR {
        GetPartitionCount(&raw mut (*DiskEntry).LogicalPartListHead)
    } else {
        0
    }) * 4;

    if !(*DiskEntry).LayoutBuffer.is_null() {
        CurrentPartitionCount = (*(*DiskEntry).LayoutBuffer).PartitionCount;
    }

    DPRINT1!("CurrentPartitionCount: %lu ; NewPartitionCount: %lu\n",
            CurrentPartitionCount, NewPartitionCount);

    if CurrentPartitionCount == NewPartitionCount {
        return (TRUE) as u8;
    }

    LayoutBufferSize = core::mem::size_of::<DRIVE_LAYOUT_INFORMATION>() as ULONG
        + (NewPartitionCount - ANYSIZE_ARRAY) * core::mem::size_of::<PARTITION_INFORMATION>() as ULONG;
    NewLayoutBuffer = (RtlReAllocateHeap(
        ProcessHeap,
        HEAP_ZERO_MEMORY,
        ((*DiskEntry).LayoutBuffer).cast(),
        (LayoutBufferSize) as SIZE_T,
    )).cast();
    if NewLayoutBuffer.is_null() {
        DPRINT1!("Failed to allocate the new layout buffer (size: %lu)\n", LayoutBufferSize);
        return (FALSE) as u8;
    }

    (*NewLayoutBuffer).PartitionCount = NewPartitionCount;

    if NewPartitionCount > CurrentPartitionCount {
        i = CurrentPartitionCount;
        while i < NewPartitionCount {
            (*(&raw mut (*NewLayoutBuffer).PartitionEntry as *mut PARTITION_INFORMATION)
                .offset(i as isize))
            .RewritePartition = (TRUE) as u8;
            i = i.wrapping_add(1);
        }
    }

    (*DiskEntry).LayoutBuffer = NewLayoutBuffer;

    return (TRUE) as u8;
}

unsafe extern "C" fn UpdateDiskLayout(mut DiskEntry: PDISKENTRY) {
    let mut PartitionInfo: PPARTITION_INFORMATION = core::ptr::null_mut();
    let mut LinkInfo: PPARTITION_INFORMATION = core::ptr::null_mut();
    let mut ListEntry: PLIST_ENTRY;
    let mut PartEntry: PPARTENTRY;
    let mut HiddenSectors64: LARGE_INTEGER = LARGE_INTEGER { QuadPart: 0 };
    let mut Index: u32;
    let mut PartitionNumber: u32 = 1;

    DbgPrint(b"UpdateDiskLayout()\n\0".as_ptr() as PCSTR);

    if (*DiskEntry).DiskStyle == PARTITION_STYLE_GPT {
        DbgPrint(b"GPT-partitioned disk detected, not currently supported by SETUP!\n\0".as_ptr() as PCSTR);
        return;
    }

    if ReAllocateLayoutBuffer(DiskEntry) == 0 {
        DbgPrint(b"ReAllocateLayoutBuffer() failed.\n\0".as_ptr() as PCSTR);
        return;
    }

    Index = 0;
    ListEntry = (*DiskEntry).PrimaryPartListHead.Flink;
    while ListEntry != core::ptr::addr_of_mut!((*DiskEntry).PrimaryPartListHead) {
        PartEntry = CONTAINING_RECORD!(ListEntry, PARTENTRY, ListEntry);

        if (*PartEntry).IsPartitioned != 0 {
            PartitionInfo = (*(*DiskEntry).LayoutBuffer).PartitionEntry.as_mut_ptr().add(Index as usize);
            (*PartEntry).PartitionIndex = Index;

            if (*PartEntry).New != 0 {
                (*PartEntry).PartitionNumber = 0;
            }

            (*PartEntry).OnDiskPartitionNumber = if !IsContainerPartition!((*PartEntry).PartitionType) {
                PartitionNumber
            } else {
                0
            };

            if IsSamePrimaryLayoutEntry(PartitionInfo, PartEntry) == 0 {
                DbgPrint(b"Updating primary partition entry %lu\n\0".as_ptr() as PCSTR, Index);

                (*PartitionInfo).StartingOffset.QuadPart = ((*PartEntry).StartSector.QuadPart * ((*(*PartEntry).DiskEntry).BytesPerSector as i64) as u64) as i64;
                (*PartitionInfo).PartitionLength.QuadPart = ((*PartEntry).SectorCount.QuadPart * ((*(*PartEntry).DiskEntry).BytesPerSector as i64) as u64) as i64;
                (*PartitionInfo).HiddenSectors = (*PartEntry).StartSector.__bindgen_anon_1.LowPart;
                (*PartitionInfo).PartitionNumber = (*PartEntry).PartitionNumber;
                (*PartitionInfo).PartitionType = (*PartEntry).PartitionType;
                (*PartitionInfo).BootIndicator = (*PartEntry).BootIndicator;
                (*PartitionInfo).RecognizedPartition = IsRecognizedPartition!((*PartEntry).PartitionType) as BOOLEAN;
                (*PartitionInfo).RewritePartition = (TRUE) as u8;
            }

            if !IsContainerPartition!((*PartEntry).PartitionType) {
                PartitionNumber += 1;
            }

            Index += 1;
        }

        ListEntry = (*ListEntry).Flink;
    }

    LinkInfo = core::ptr::null_mut();
    Index = 4;
    ListEntry = (*DiskEntry).LogicalPartListHead.Flink;
    while ListEntry != core::ptr::addr_of_mut!((*DiskEntry).LogicalPartListHead) {
        PartEntry = CONTAINING_RECORD!(ListEntry, PARTENTRY, ListEntry);

        if (*PartEntry).IsPartitioned != 0 {
            PartitionInfo = (*(*DiskEntry).LayoutBuffer).PartitionEntry.as_mut_ptr().add(Index as usize);
            (*PartEntry).PartitionIndex = Index;

            if (*PartEntry).New != 0 {
                (*PartEntry).PartitionNumber = 0;
            }

            (*PartEntry).OnDiskPartitionNumber = PartitionNumber;

            DbgPrint(b"Updating logical partition entry %lu\n\0".as_ptr() as PCSTR, Index);

            (*PartitionInfo).StartingOffset.QuadPart = ((*PartEntry).StartSector.QuadPart * ((*(*PartEntry).DiskEntry).BytesPerSector as i64) as u64) as i64;
            (*PartitionInfo).PartitionLength.QuadPart = ((*PartEntry).SectorCount.QuadPart * ((*(*PartEntry).DiskEntry).BytesPerSector as i64) as u64) as i64;
            (*PartitionInfo).HiddenSectors = (*DiskEntry).SectorAlignment;
            (*PartitionInfo).PartitionNumber = (*PartEntry).PartitionNumber;
            (*PartitionInfo).PartitionType = (*PartEntry).PartitionType;
            (*PartitionInfo).BootIndicator = (FALSE) as u8;
            (*PartitionInfo).RecognizedPartition = IsRecognizedPartition!((*PartEntry).PartitionType) as BOOLEAN;
            (*PartitionInfo).RewritePartition = (TRUE) as u8;

            if !LinkInfo.is_null() {
                (*LinkInfo).StartingOffset.QuadPart = (((*PartEntry).StartSector.QuadPart - ((*DiskEntry).SectorAlignment as i64) as u64) * ((*DiskEntry).BytesPerSector as i64) as u64) as i64;
                (*LinkInfo).PartitionLength.QuadPart = (((*PartEntry).StartSector.QuadPart + ((*DiskEntry).SectorAlignment as i64) as u64) * ((*DiskEntry).BytesPerSector as i64) as u64) as i64;
                HiddenSectors64.QuadPart = ((*PartEntry).StartSector.QuadPart - ((*DiskEntry).SectorAlignment as i64) as u64 - (*(*DiskEntry).ExtendedPartition).StartSector.QuadPart) as i64;
                (*LinkInfo).HiddenSectors = HiddenSectors64.__bindgen_anon_1.LowPart;
                (*LinkInfo).PartitionNumber = 0;
                (*LinkInfo).PartitionType = (PARTITION_EXTENDED) as u8;
                (*LinkInfo).BootIndicator = (FALSE) as u8;
                (*LinkInfo).RecognizedPartition = (FALSE) as u8;
                (*LinkInfo).RewritePartition = (TRUE) as u8;
            }

            LinkInfo = (*(*DiskEntry).LayoutBuffer).PartitionEntry.as_mut_ptr().add((Index + 1) as usize);

            PartitionNumber += 1;
            Index += 4;
        }

        ListEntry = (*ListEntry).Flink;
    }

    Index = GetPartitionCount(core::ptr::addr_of_mut!((*DiskEntry).PrimaryPartListHead));
    while Index < 4 {
        DbgPrint(b"Primary partition entry %lu\n\0".as_ptr() as PCSTR, Index);

        PartitionInfo = (*(*DiskEntry).LayoutBuffer).PartitionEntry.as_mut_ptr().add(Index as usize);

        if IsEmptyLayoutEntry(PartitionInfo) == 0 {
            DbgPrint(b"Wiping primary partition entry %lu\n\0".as_ptr() as PCSTR, Index);

            (*PartitionInfo).StartingOffset.QuadPart = 0;
            (*PartitionInfo).PartitionLength.QuadPart = 0;
            (*PartitionInfo).HiddenSectors = 0;
            (*PartitionInfo).PartitionNumber = 0;
            (*PartitionInfo).PartitionType = (PARTITION_ENTRY_UNUSED) as u8;
            (*PartitionInfo).BootIndicator = (FALSE) as u8;
            (*PartitionInfo).RecognizedPartition = (FALSE) as u8;
            (*PartitionInfo).RewritePartition = (TRUE) as u8;
        }

        Index += 1;
    }

    Index = 4;
    while Index < (*(*DiskEntry).LayoutBuffer).PartitionCount {
        if Index % 4 >= 2 {
            DbgPrint(b"Logical partition entry %lu\n\0".as_ptr() as PCSTR, Index);

            PartitionInfo = (*(*DiskEntry).LayoutBuffer).PartitionEntry.as_mut_ptr().add(Index as usize);

            if IsEmptyLayoutEntry(PartitionInfo) == 0 {
                DbgPrint(b"Wiping partition entry %lu\n\0".as_ptr() as PCSTR, Index);

                (*PartitionInfo).StartingOffset.QuadPart = 0;
                (*PartitionInfo).PartitionLength.QuadPart = 0;
                (*PartitionInfo).HiddenSectors = 0;
                (*PartitionInfo).PartitionNumber = 0;
                (*PartitionInfo).PartitionType = (PARTITION_ENTRY_UNUSED) as u8;
                (*PartitionInfo).BootIndicator = (FALSE) as u8;
                (*PartitionInfo).RecognizedPartition = (FALSE) as u8;
                (*PartitionInfo).RewritePartition = (TRUE) as u8;
            }
        }

        Index += 1;
    }

    (*DiskEntry).DiskStyle = PARTITION_STYLE_MBR;
    (*DiskEntry).Dirty = (TRUE) as u8;

    // C: `DumpPartitionTable(DiskEntry)` under `#ifdef DUMP_PARTITION_TABLE`, never defined.
}

#[no_mangle]
pub unsafe extern "system" fn GetAdjUnpartitionedEntry(
    mut PartEntry: PPARTENTRY,
    Direction: BOOLEAN,
) -> PPARTENTRY {
    let DiskEntry: PDISKENTRY = (*PartEntry).DiskEntry as PDISKENTRY;
    let mut ListHead: PLIST_ENTRY = core::ptr::null_mut();
    let mut AdjEntry: PLIST_ENTRY = core::ptr::null_mut();

    if (*DiskEntry).DiskStyle == PARTITION_STYLE_MBR && (*PartEntry).LogicalPartition != 0 {
        ListHead = &raw mut (*DiskEntry).LogicalPartListHead as PLIST_ENTRY;
    } else {
        ListHead = &raw mut (*DiskEntry).PrimaryPartListHead as PLIST_ENTRY;
    }

    if Direction != 0 {
        AdjEntry = (*PartEntry).ListEntry.Flink as PLIST_ENTRY;
    } else {
        AdjEntry = (*PartEntry).ListEntry.Blink as PLIST_ENTRY;
    }

    if AdjEntry != ListHead {
        PartEntry = CONTAINING_RECORD!(AdjEntry, PARTENTRY, ListEntry);
        if (*PartEntry).IsPartitioned == 0 {
            if (*PartEntry).PartitionType as core::ffi::c_int != 0 {
                RtlAssert(
                    b"PartEntry->PartitionType == PARTITION_ENTRY_UNUSED\0".as_ptr()
                        as *const core::ffi::c_char
                        as *mut core::ffi::c_void,
                    b"base/setup/lib/utils/partlist.c\0"
                        .as_ptr()
                        as *const core::ffi::c_char
                        as *mut core::ffi::c_void,
                    2884u32,
                    core::ptr::null_mut::<core::ffi::c_char>(),
                );
            }
            return PartEntry;
        }
    }

    return core::ptr::null_mut();
}

unsafe extern "C" fn MBRPartitionCreateChecks(
    mut PartEntry: PPARTENTRY,
    mut SizeBytes: ULONGLONG,
    mut PartitionInfo: ULONG_PTR,
) -> ERROR_NUMBER {
    let mut DiskEntry: PDISKENTRY = (*PartEntry).DiskEntry;
    let mut isContainer: BOOLEAN = (IsContainerPartition!(PartitionInfo as UCHAR)) as u8;

    if isContainer != 0 {
        if (*PartEntry).LogicalPartition != 0 {
            return ERROR_ONLY_ONE_EXTENDED;
        }
        if !(*DiskEntry).ExtendedPartition.is_null() {
            return ERROR_ONLY_ONE_EXTENDED;
        }
    }
    if (*PartEntry).LogicalPartition == 0 || isContainer != 0 {
        if IsSuperFloppy(DiskEntry) != 0 {
            return ERROR_PARTITION_TABLE_FULL;
        }
        if GetPrimaryPartitionCount!(DiskEntry) >= 4 {
            return ERROR_PARTITION_TABLE_FULL;
        }
    } else if IsSuperFloppy(DiskEntry) != 0 {
        return ERROR_PARTITION_TABLE_FULL;
    }
    return ERROR_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn PartitionCreateChecks(
    PartEntry: PPARTENTRY,
    SizeBytes: ULONGLONG,
    PartitionInfo: ULONG_PTR,
) -> ERROR_NUMBER {
    if (*PartEntry).IsPartitioned != 0 {
        return ERROR_NEW_PARTITION;
    }
    return MBRPartitionCreateChecks(PartEntry, SizeBytes, PartitionInfo);
}

#[no_mangle]
pub unsafe extern "system" fn CreatePartition(
    List: PPARTLIST,
    PartEntry: PPARTENTRY,
    SizeBytes: ULONGLONG,
    PartitionInfo: ULONG_PTR,
) -> BOOLEAN {
    let mut Error: ERROR_NUMBER;
    let isContainer: BOOLEAN = (IsContainerPartition!(PartitionInfo as UCHAR)) as u8;
    let mut DiskEntry: PDISKENTRY;
    let mut mainType: PCSTR = b"Primary\0".as_ptr() as PCSTR;

    if isContainer != 0 {
        mainType = b"Extended\0".as_ptr() as PCSTR;
    } else if !PartEntry.is_null() && (*PartEntry).LogicalPartition != 0 {
        mainType = b"Logical\0".as_ptr() as PCSTR;
    }

    DPRINT1!(b"CreatePartition(%s, %I64u bytes)\n\0".as_ptr() as PCSTR, mainType, SizeBytes);

    if List.is_null()
        || PartEntry.is_null()
        || (*PartEntry).DiskEntry.is_null()
        || (*PartEntry).IsPartitioned != 0
    {
        return (FALSE) as u8;
    }

    Error = PartitionCreateChecks(PartEntry, SizeBytes, PartitionInfo);
    if Error != NOT_AN_ERROR {
        DPRINT1!(b"PartitionCreateChecks(%s) failed with error %lu\n\0".as_ptr() as PCSTR, mainType, Error);
        return (FALSE) as u8;
    }

    if InitializePartitionEntry(PartEntry, SizeBytes, PartitionInfo) == 0 {
        return (FALSE) as u8;
    }

    DiskEntry = (*PartEntry).DiskEntry;
    UpdateDiskLayout(DiskEntry);

    if isContainer == 0 {
        (*PartEntry).Volume = InitVolume((*DiskEntry).PartList, PartEntry);
    }

    AssignDriveLetters(List);

    return (TRUE) as u8;
}

unsafe extern "C" fn DismountPartition(
    List: PPARTLIST,
    PartEntry: PPARTENTRY,
) -> NTSTATUS {
    let mut Volume: PVOLENTRY = (*PartEntry).Volume;

    if !Volume.is_null() {
        (*PartEntry).Volume = core::ptr::null_mut();
        (*Volume).PartEntry = core::ptr::null_mut();
        RemoveEntryList(&raw mut (*Volume).ListEntry);

        if (*Volume).New != 0 || (*Volume).FormatState == UnknownFormat {
            RtlFreeHeap(ProcessHeap, 0, (Volume).cast());
            Volume = core::ptr::null_mut();
        }
    }
    if Volume.is_null() {
        return STATUS_SUCCESS;
    }

    InsertTailList(
        &raw mut (*List).PendingUnmountVolumesList,
        &raw mut (*Volume).ListEntry,
    );
    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn DeletePartition(
    List: PPARTLIST,
    PartEntry: PPARTENTRY,
    FreeRegion: *mut PPARTENTRY,
) -> BOOLEAN {
    let mut DiskEntry: PDISKENTRY;
    let mut PrevPartEntry: PPARTENTRY;
    let mut NextPartEntry: PPARTENTRY;
    let mut LogicalPartEntry: PPARTENTRY;
    let mut Entry: PLIST_ENTRY;

    if List.is_null()
        || PartEntry.is_null()
        || (*PartEntry).DiskEntry.is_null()
        || (*PartEntry).IsPartitioned == 0
    {
        return (FALSE) as u8;
    }

    if (*List).SystemPartition == PartEntry {
        (*List).SystemPartition = core::ptr::null_mut();
    }

    DiskEntry = (*PartEntry).DiskEntry;

    if (*DiskEntry).ExtendedPartition == PartEntry {
        while IsListEmpty(&mut (*DiskEntry).LogicalPartListHead) == false {
            Entry = RemoveHeadList(&mut (*DiskEntry).LogicalPartListHead);
            LogicalPartEntry = CONTAINING_RECORD!(Entry, PARTENTRY, ListEntry);

            DismountPartition(List, LogicalPartEntry);
            DestroyRegion(LogicalPartEntry);
        }

        (*DiskEntry).ExtendedPartition = core::ptr::null_mut();
    } else {
        DismountPartition(List, PartEntry);
    }

    PrevPartEntry = GetAdjUnpartitionedEntry(PartEntry, (FALSE) as BOOLEAN);
    NextPartEntry = GetAdjUnpartitionedEntry(PartEntry, (TRUE) as BOOLEAN);

    if !PrevPartEntry.is_null() && !NextPartEntry.is_null() {
        (*PrevPartEntry).SectorCount.QuadPart += (*PartEntry).SectorCount.QuadPart
            + (*NextPartEntry).SectorCount.QuadPart;

        RemoveEntryList(&mut (*PartEntry).ListEntry);
        DestroyRegion(PartEntry);
        RemoveEntryList(&mut (*NextPartEntry).ListEntry);
        DestroyRegion(NextPartEntry);

        if !FreeRegion.is_null() {
            *FreeRegion = PrevPartEntry;
        }
    } else if !PrevPartEntry.is_null() && NextPartEntry.is_null() {
        (*PrevPartEntry).SectorCount.QuadPart += (*PartEntry).SectorCount.QuadPart;

        RemoveEntryList(&mut (*PartEntry).ListEntry);
        DestroyRegion(PartEntry);

        if !FreeRegion.is_null() {
            *FreeRegion = PrevPartEntry;
        }
    } else if PrevPartEntry.is_null() && !NextPartEntry.is_null() {
        (*NextPartEntry).StartSector.QuadPart = (*PartEntry).StartSector.QuadPart;
        (*NextPartEntry).SectorCount.QuadPart += (*PartEntry).SectorCount.QuadPart;

        RemoveEntryList(&mut (*PartEntry).ListEntry);
        DestroyRegion(PartEntry);

        if !FreeRegion.is_null() {
            *FreeRegion = NextPartEntry;
        }
    } else {
        (*PartEntry).New = (FALSE) as u8;
        (*PartEntry).IsPartitioned = (FALSE) as u8;
        (*PartEntry).PartitionType = (PARTITION_ENTRY_UNUSED) as u8;
        (*PartEntry).OnDiskPartitionNumber = 0;
        (*PartEntry).PartitionNumber = 0;
        (*PartEntry).BootIndicator = (FALSE) as u8;
        (*PartEntry).DeviceName[0] = UNICODE_NULL;

        if !(*PartEntry).Volume.is_null() {
            RemoveEntryList(&mut (*(*PartEntry).Volume).ListEntry);
            RtlFreeHeap(ProcessHeap, 0, ((*PartEntry).Volume).cast());
        }
        (*PartEntry).Volume = core::ptr::null_mut();

        if !FreeRegion.is_null() {
            *FreeRegion = PartEntry;
        }
    }

    UpdateDiskLayout(DiskEntry);
    AssignDriveLetters(List);

    return (TRUE) as u8;
}

unsafe extern "C" fn IsSupportedActivePartition(mut PartEntry: PPARTENTRY) -> BOOLEAN {
    let mut Volume: PVOLENTRY = core::ptr::null_mut::<_VOLENTRY>();

    if IsContainerPartition!((*PartEntry).PartitionType) {
        DPRINT1!(
            b"System partition %lu in disk %lu is an extended partition container?!\n\0"
                .as_ptr() as PCSTR,
            (*PartEntry).PartitionNumber,
            (*(*PartEntry).DiskEntry).DiskNumber,
        );
        return (FALSE) as u8;
    }

    Volume = (*PartEntry).Volume;
    if Volume.is_null() {
        return (FALSE) as u8;
    }

    if (*Volume).FormatState as u32 == Unformatted as i32 as u32 {
        return (TRUE) as u8;
    } else if (*Volume).FormatState as u32 == Formatted as i32 as u32 {
        if _wcsicmp(
            &raw mut (*Volume).Info.FileSystem as *mut WCHAR,
            RUSTCONV_WSTR_A953DDDD78B1.as_ptr(),
        ) == 0
            || _wcsicmp(
                &raw mut (*Volume).Info.FileSystem as *mut WCHAR,
                RUSTCONV_WSTR_BB53EE05898F.as_ptr(),
            ) == 0
            || _wcsicmp(
                &raw mut (*Volume).Info.FileSystem as *mut WCHAR,
                RUSTCONV_WSTR_BE0DEB469A7B.as_ptr(),
            ) == 0
        {
            return (TRUE) as u8;
        } else {
            DPRINT1!(
                b"Recognized file system '%S' that doesn't have write support yet!\n\0"
                    .as_ptr() as PCSTR,
                &raw mut (*Volume).Info.FileSystem as *mut WCHAR,
            );
            return (FALSE) as u8;
        }
    } else {
        DPRINT1!(
            b"System partition %lu in disk %lu with no or unknown FS?!\n\0"
                .as_ptr() as PCSTR,
            (*PartEntry).PartitionNumber,
            (*(*PartEntry).DiskEntry).DiskNumber,
        );
        return (FALSE) as u8;
    }

    if ((*PartEntry).PartitionType) as u32 == PARTITION_IFS {
        DPRINT1!(
            b"Recognized file system '%S' that doesn't have write support yet!\n\0"
                .as_ptr() as PCSTR,
            &raw mut (*Volume).Info.FileSystem as *mut WCHAR,
        );
        return (FALSE) as u8;
    }

    return (TRUE) as u8;
}
#[no_mangle]
pub unsafe extern "C" fn FindSupportedSystemPartition(
    mut List: PPARTLIST,
    mut ForceSelect: BOOLEAN,
    mut AlternativeDisk: PDISKENTRY,
    mut AlternativePart: PPARTENTRY,
) -> PPARTENTRY {
    let mut current_block: u32;
    let mut ListEntry: PLIST_ENTRY = core::ptr::null_mut();
    let mut DiskEntry: PDISKENTRY = core::ptr::null_mut();
    let mut PartEntry: PPARTENTRY = core::ptr::null_mut();
    let mut ActivePartition: PPARTENTRY = core::ptr::null_mut();
    let mut CandidatePartition: PPARTENTRY = core::ptr::null_mut();

    current_block = 0;
    loop {
        match current_block {
            0 => {
                if IsListEmpty(&raw mut (*List).DiskListHead) != false {
                    ASSERT!((*List).SystemPartition.is_null());
                    current_block = 3;
                    continue;
                }

                if AlternativeDisk.is_null() && !AlternativePart.is_null() {
                    AlternativeDisk = (*AlternativePart).DiskEntry;
                }

                if !AlternativePart.is_null() {
                    ASSERT!(!AlternativeDisk.is_null() && (*AlternativePart).DiskEntry == AlternativeDisk);
                }

                if !AlternativeDisk.is_null() {
                    ASSERT!((*AlternativeDisk).PartList == List);
                }

                CandidatePartition = core::ptr::null_mut();

                DiskEntry = GetSystemDisk(List);
                if DiskEntry.is_null() {
                    current_block = 1;
                    continue;
                }

                if (*DiskEntry).DiskStyle == PARTITION_STYLE_GPT {
                    DPRINT1!(b"System disk -- GPT-partitioned disk detected, not currently supported by SETUP!\n\0".as_ptr() as PCSTR);
                    current_block = 1;
                    continue;
                }

                ActivePartition = (*List).SystemPartition;
                if !ActivePartition.is_null() && IsSupportedActivePartition(ActivePartition) != 0 {
                    CandidatePartition = ActivePartition;

                    DPRINT1!(b"Use the current system partition %lu in disk %lu, drive letter %C\n\0".as_ptr() as PCSTR,
                            (*CandidatePartition).PartitionNumber,
                            (*(*CandidatePartition).DiskEntry).DiskNumber,
                            (if (*(*CandidatePartition).Volume).Info.DriveLetter == 0 {
                                0x002Du16
                            } else {
                                (*(*CandidatePartition).Volume).Info.DriveLetter
                            }) as ::core::ffi::c_int);

                    return CandidatePartition;
                }

                if DiskEntry != AlternativeDisk {
                    ListEntry = (*DiskEntry).PrimaryPartListHead.Flink;
                    while ListEntry != &raw mut (*DiskEntry).PrimaryPartListHead {
                        PartEntry = CONTAINING_RECORD!(ListEntry, PARTENTRY, ListEntry);

                        if PartEntry != ActivePartition {
                            if (*PartEntry).IsPartitioned != 0 &&
                               !IsContainerPartition!((*PartEntry).PartitionType) {
                                if IsSupportedActivePartition(PartEntry) != 0 {
                                    CandidatePartition = PartEntry;
                                    current_block = 4;
                                    break;
                                }
                            }
                        }

                        ListEntry = (*ListEntry).Flink;
                    }

                    if current_block == 4 {
                        continue;
                    }

                    if GetPartitionCount(&raw mut (*DiskEntry).PrimaryPartListHead) < 4 {
                        ListEntry = (*DiskEntry).PrimaryPartListHead.Flink;
                        while ListEntry != &raw mut (*DiskEntry).PrimaryPartListHead {
                            PartEntry = CONTAINING_RECORD!(ListEntry, PARTENTRY, ListEntry);

                            if PartEntry != ActivePartition {
                                if (*PartEntry).IsPartitioned == 0 {
                                    CandidatePartition = PartEntry;
                                    current_block = 4;
                                    break;
                                }
                            }

                            ListEntry = (*ListEntry).Flink;
                        }

                        if current_block == 4 {
                            continue;
                        }
                    }
                }

                current_block = 1;
                continue;
            }

            1 => {
                if AlternativeDisk.is_null() || (ForceSelect == 0 && DiskEntry != AlternativeDisk) {
                    current_block = 3;
                    continue;
                }

                if (*AlternativeDisk).DiskStyle == PARTITION_STYLE_GPT {
                    DPRINT1!(b"Alternative disk -- GPT-partitioned disk detected, not currently supported by SETUP!\n\0".as_ptr() as PCSTR);
                    current_block = 3;
                    continue;
                }

                if DiskEntry != AlternativeDisk {
                    DiskEntry = AlternativeDisk;

                    ActivePartition = GetActiveDiskPartition(DiskEntry);
                    if !ActivePartition.is_null() && IsSupportedActivePartition(ActivePartition) != 0 {
                        CandidatePartition = ActivePartition;
                        current_block = 4;
                        continue;
                    }
                }

                PartEntry = CONTAINING_RECORD!((*DiskEntry).PrimaryPartListHead.Flink,
                                              PARTENTRY, ListEntry);
                CandidatePartition = PartEntry;

                if (*DiskEntry).NewDisk != 0 {
                    if (*CandidatePartition).IsPartitioned == 0 || (*CandidatePartition).BootIndicator == 0 {
                        DPRINT1!(b"Use new first active system partition %lu in disk %lu, drive letter %C\n\0".as_ptr() as PCSTR,
                                (*CandidatePartition).PartitionNumber,
                                (*(*CandidatePartition).DiskEntry).DiskNumber,
                                (if (*CandidatePartition).Volume.is_null() ||
                                   (*(*CandidatePartition).Volume).Info.DriveLetter == 0 {
                                    0x002Du16
                                } else {
                                    (*(*CandidatePartition).Volume).Info.DriveLetter
                                }) as ::core::ffi::c_int);

                        return CandidatePartition;
                    }

                    DPRINT1!(b"NewDisk TRUE but first partition is used?\n\0".as_ptr() as PCSTR);
                }

                ListEntry = (*DiskEntry).PrimaryPartListHead.Flink;
                while ListEntry != &raw mut (*DiskEntry).PrimaryPartListHead {
                    PartEntry = CONTAINING_RECORD!(ListEntry, PARTENTRY, ListEntry);

                    if ((*PartEntry).PartitionType) as u32 != PARTITION_ENTRY_UNUSED ||
                       (*PartEntry).BootIndicator != 0 {
                        break;
                    }

                    ListEntry = (*ListEntry).Flink;
                }

                if ListEntry == &raw mut (*DiskEntry).PrimaryPartListHead {
                    DPRINT1!(b"Use first active system partition %lu in disk %lu, drive letter %C\n\0".as_ptr() as PCSTR,
                            (*CandidatePartition).PartitionNumber,
                            (*(*CandidatePartition).DiskEntry).DiskNumber,
                            (if (*CandidatePartition).Volume.is_null() ||
                               (*(*CandidatePartition).Volume).Info.DriveLetter == 0 {
                                0x002Du16
                            } else {
                                (*(*CandidatePartition).Volume).Info.DriveLetter
                            }) as ::core::ffi::c_int);

                    return CandidatePartition;
                }

                if !AlternativePart.is_null() {
                    DPRINT1!(b"No valid or supported system partition has been found, use the alternative partition!\n\0".as_ptr() as PCSTR);
                    CandidatePartition = AlternativePart;
                    current_block = 4;
                    continue;
                } else {
                    current_block = 3;
                    continue;
                }
            }

            3 => {
                DPRINT1!(b"No valid or supported system partition has been found on this system!\n\0".as_ptr() as PCSTR);
                return core::ptr::null_mut();
            }

            4 => {
                ASSERT!(!CandidatePartition.is_null());

                DPRINT1!(b"Use alternative active system partition %lu in disk %lu, drive letter %C\n\0".as_ptr() as PCSTR,
                        (*CandidatePartition).PartitionNumber,
                        (*(*CandidatePartition).DiskEntry).DiskNumber,
                        (if (*CandidatePartition).Volume.is_null() ||
                           (*(*CandidatePartition).Volume).Info.DriveLetter == 0 {
                            0x002Du16
                        } else {
                            (*(*CandidatePartition).Volume).Info.DriveLetter
                        }) as ::core::ffi::c_int);

                return CandidatePartition;
            }

            _ => return core::ptr::null_mut(),
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn SetActivePartition(
    List: PPARTLIST,
    PartEntry: PPARTENTRY,
    mut OldActivePart: PPARTENTRY,
) -> BOOLEAN {
    if IsListEmpty(&raw mut (*List).DiskListHead) != false {
        return (FALSE) as u8;
    }
    if PartEntry.is_null() {
        return (FALSE) as u8;
    }
    if PartEntry == (*List).SystemPartition
        || (PartEntry == OldActivePart && IsPartitionActive(OldActivePart) != 0)
    {
        return (TRUE) as u8;
    }
    if !(!OldActivePart.is_null()
        && IsPartitionActive(OldActivePart) != 0
        && (*OldActivePart).DiskEntry == (*PartEntry).DiskEntry)
    {
        OldActivePart = GetActiveDiskPartition((*PartEntry).DiskEntry);
    }
    if !OldActivePart.is_null() {
        (*OldActivePart).BootIndicator = (FALSE) as u8;
        (*(&raw mut (*(*(*OldActivePart).DiskEntry).LayoutBuffer).PartitionEntry
            as *mut PARTITION_INFORMATION)
            .offset((*OldActivePart).PartitionIndex as isize))
        .BootIndicator = (FALSE) as u8;
        (*(&raw mut (*(*(*OldActivePart).DiskEntry).LayoutBuffer).PartitionEntry
            as *mut PARTITION_INFORMATION)
            .offset((*OldActivePart).PartitionIndex as isize))
        .RewritePartition = (TRUE) as u8;
        (*(*OldActivePart).DiskEntry).Dirty = (TRUE) as u8;
    }
    if (*PartEntry).DiskEntry == GetSystemDisk(List) {
        (*List).SystemPartition = PartEntry;
    }
    (*PartEntry).BootIndicator = (TRUE) as u8;
    (*(&raw mut (*(*(*PartEntry).DiskEntry).LayoutBuffer).PartitionEntry
        as *mut PARTITION_INFORMATION)
        .offset((*PartEntry).PartitionIndex as isize))
    .BootIndicator = (TRUE) as u8;
    (*(&raw mut (*(*(*PartEntry).DiskEntry).LayoutBuffer).PartitionEntry
        as *mut PARTITION_INFORMATION)
        .offset((*PartEntry).PartitionIndex as isize))
    .RewritePartition = (TRUE) as u8;
    (*(*PartEntry).DiskEntry).Dirty = (TRUE) as u8;
    return (TRUE) as u8;
}
#[no_mangle]
pub unsafe extern "C" fn WritePartitions(DiskEntry: PDISKENTRY) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut Name: _UNICODE_STRING = core::mem::zeroed();
    let mut FileHandle: HANDLE = core::ptr::null_mut();
    let mut Iosb: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut BufferSize: ULONG;
    let mut PartitionInfo: PPARTITION_INFORMATION;
    let mut PartitionCount: ULONG;
    let mut ListEntry: PLIST_ENTRY;
    let mut PartEntry: PPARTENTRY;
    let mut DstPath: [u16; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    DPRINT!("WritePartitions() Disk: %lu\n", (*DiskEntry).DiskNumber);

    if (*DiskEntry).Dirty == 0 {
        return STATUS_SUCCESS;
    }

    RtlStringCchPrintfW!(
        DstPath.as_mut_ptr(),
        ARRAYSIZE!(DstPath),
        RUSTCONV_WSTR_27B3B5E195F7.as_ptr(),
        (*DiskEntry).DiskNumber,
    );
    RtlInitUnicodeString(&mut Name, DstPath.as_mut_ptr());

    InitializeObjectAttributes!(
        &mut ObjectAttributes,
        &mut Name,
        OBJ_CASE_INSENSITIVE,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    Status = NtOpenFile(
        &mut FileHandle,
        GENERIC_READ | GENERIC_WRITE | SYNCHRONIZE,
        &mut ObjectAttributes,
        &mut Iosb,
        0,
        FILE_SYNCHRONOUS_IO_NONALERT,
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT1!("NtOpenFile() failed (Status %lx)\n", Status);
        return Status;
    }

    PartitionCount = (*(*DiskEntry).LayoutBuffer).PartitionCount;

    BufferSize = (core::mem::size_of::<DRIVE_LAYOUT_INFORMATION>()
        + ((PartitionCount - 1) as usize * core::mem::size_of::<PARTITION_INFORMATION>())) as ULONG;
    Status = NtDeviceIoControlFile(
        FileHandle,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        &mut Iosb,
        (IOCTL_DISK_SET_DRIVE_LAYOUT) as ULONG,
        (*DiskEntry).LayoutBuffer as *mut _,
        BufferSize,
        (*DiskEntry).LayoutBuffer as *mut _,
        BufferSize,
    );
    NtClose(FileHandle);

    (*(*DiskEntry).LayoutBuffer).PartitionCount = PartitionCount;

    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT1!("IOCTL_DISK_SET_DRIVE_LAYOUT failed (Status 0x%08lx)\n", Status);
        return Status;
    }

    ListEntry = (*DiskEntry).PrimaryPartListHead.Flink;
    while ListEntry != &mut (*DiskEntry).PrimaryPartListHead {
        PartEntry = CONTAINING_RECORD!(ListEntry, PARTENTRY, ListEntry);
        if (*PartEntry).IsPartitioned == 0 {
            ListEntry = (*ListEntry).Flink;
            continue;
        }

        if (*PartEntry).New != 0 {
            PartitionInfo = &mut (*{ let flex: *mut [_; 1] = core::ptr::addr_of_mut!((*(*DiskEntry).LayoutBuffer).PartitionEntry); core::ptr::addr_of_mut!((*flex)[0]) }.add((*PartEntry).PartitionIndex as usize));
            (*PartEntry).PartitionNumber = (*PartitionInfo).PartitionNumber;
            InitPartitionDeviceName(PartEntry);
        }
        (*PartEntry).New = (FALSE) as u8;

        ListEntry = (*ListEntry).Flink;
    }

    ListEntry = (*DiskEntry).LogicalPartListHead.Flink;
    while ListEntry != &mut (*DiskEntry).LogicalPartListHead {
        PartEntry = CONTAINING_RECORD!(ListEntry, PARTENTRY, ListEntry);
        if (*PartEntry).IsPartitioned == 0 {
            ListEntry = (*ListEntry).Flink;
            continue;
        }

        if (*PartEntry).New != 0 {
            PartitionInfo = &mut (*{ let flex: *mut [_; 1] = core::ptr::addr_of_mut!((*(*DiskEntry).LayoutBuffer).PartitionEntry); core::ptr::addr_of_mut!((*flex)[0]) }.add((*PartEntry).PartitionIndex as usize));
            (*PartEntry).PartitionNumber = (*PartitionInfo).PartitionNumber;
            InitPartitionDeviceName(PartEntry);
        }
        (*PartEntry).New = (FALSE) as u8;

        ListEntry = (*ListEntry).Flink;
    }

    (*DiskEntry).DiskStyle = PARTITION_STYLE_MBR;
    (*DiskEntry).Dirty = (FALSE) as u8;

    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn WritePartitionsToDisk(List: *mut PARTLIST) -> i32 {
    let mut Status: i32 = 0;
    let mut Entry: PLIST_ENTRY = core::ptr::null_mut();
    let mut DiskEntry: *mut DISKENTRY = core::ptr::null_mut();
    let mut Volume: *mut VOLENTRY = core::ptr::null_mut();

    if List.is_null() {
        return (TRUE) as i32;
    }

    Status = STATUS_SUCCESS;
    while !IsListEmpty(&mut (*List).PendingUnmountVolumesList) {
        let mut UnmountStatus: i32 = 0;
        Entry = RemoveHeadList(&mut (*List).PendingUnmountVolumesList);
        Volume = CONTAINING_RECORD!(Entry, VOLENTRY, ListEntry);
        UnmountStatus = DismountVolume(&mut (*Volume).Info, (TRUE) as BOOLEAN);
        if ((NT_SUCCESS!(UnmountStatus)) == 0) {
            Status = UnmountStatus;
        }
        RtlFreeHeap(ProcessHeap, 0, Volume as *mut core::ffi::c_void);
    }
    if ((NT_SUCCESS!(Status)) == 0) {
        return (FALSE) as i32;
    }

    Entry = (*List).DiskListHead.Flink;
    while Entry != &mut (*List).DiskListHead {
        DiskEntry = CONTAINING_RECORD!(Entry, DISKENTRY, ListEntry);

        if (*DiskEntry).DiskStyle == PARTITION_STYLE_GPT {
            DPRINT!(b"GPT-partitioned disk detected, not currently supported by SETUP!\n\0".as_ptr() as PCSTR);
            Entry = (*Entry).Flink;
            continue;
        }

        if ((*DiskEntry).Dirty) as u32 != FALSE {
            Status = WritePartitions(DiskEntry);
            if ((NT_SUCCESS!(Status)) == 0) {
                DPRINT1!(
                    b"WritePartitionsToDisk() failed to update disk %lu, Status 0x%08lx\n\0".as_ptr() as PCSTR,
                    (*DiskEntry).DiskNumber,
                    Status,
                );
            }
        }
        Entry = (*Entry).Flink;
    }

    Entry = (*List).VolumesList.Flink;
    while Entry != &mut (*List).VolumesList {
        Volume = CONTAINING_RECORD!(Entry, VOLENTRY, ListEntry);
        InitVolumeDeviceName(Volume, core::ptr::null_mut());
        Entry = (*Entry).Flink;
    }

    return (TRUE) as i32;
}

unsafe extern "C" fn SetMountedDeviceValue(mut Volume: PVOLENTRY) -> BOOLEAN {
    let mut PartEntry: PPARTENTRY = (*Volume).PartEntry as PPARTENTRY;
    let mut Letter: WCHAR = (*Volume).Info.DriveLetter;
    let mut Status: NTSTATUS = 0;
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = _OBJECT_ATTRIBUTES {
        Length: 0,
        RootDirectory: core::ptr::null_mut(),
        ObjectName: core::ptr::null_mut(),
        Attributes: 0,
        SecurityDescriptor: core::ptr::null_mut(),
        SecurityQualityOfService: core::ptr::null_mut(),
    };
    let mut KeyName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: RUSTCONV_WSTR_BFE4472B9CA3.as_ptr() as PWSTR,
    };
    let mut ValueName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut Buffer: [WCHAR; 16] = [0; 16];
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut MountInfo: REG_DISK_MOUNT_INFO = REG_DISK_MOUNT_INFO {
        Signature: 0,
        StartingOffset: 0,
    };
    if Letter == 0 {
        return (TRUE) as u8;
    }
    RtlStringCchPrintfW!(
        Buffer.as_mut_ptr(),
        _countof!(Buffer),
        RUSTCONV_WSTR_DC70E3A7FF9A.as_ptr(),
        Letter as i32,
    );
    RtlInitUnicodeString(&mut ValueName, Buffer.as_mut_ptr());
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = GetRootKeyByPredefKey((HKEY_LOCAL_MACHINE).cast(), core::ptr::null_mut());
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.ObjectName = &mut KeyName;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();
    Status = NtOpenKey(
        &mut KeyHandle,
        KEY_ALL_ACCESS,
        &mut ObjectAttributes,
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        Status = NtCreateKey(
            &mut KeyHandle,
            KEY_ALL_ACCESS,
            &mut ObjectAttributes,
            0,
            core::ptr::null_mut(),
            REG_OPTION_NON_VOLATILE,
            core::ptr::null_mut(),
        );
    }
    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT1!("NtCreateKey() failed (Status %lx)\n", Status);
        return (FALSE) as u8;
    }
    MountInfo.Signature = (*(*(*PartEntry).DiskEntry).LayoutBuffer).Signature;
    MountInfo.StartingOffset = (*PartEntry)
        .StartSector
        .QuadPart
        .wrapping_mul((*(*PartEntry).DiskEntry).BytesPerSector as u64);
    Status = NtSetValueKey(
        KeyHandle,
        &mut ValueName,
        0,
        REG_BINARY,
        &mut MountInfo as *mut REG_DISK_MOUNT_INFO as *mut core::ffi::c_void,
        core::mem::size_of::<REG_DISK_MOUNT_INFO>() as u32,
    );
    NtClose(KeyHandle);
    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT1!("NtSetValueKey() failed (Status %lx)\n", Status);
        return (FALSE) as u8;
    }
    return (TRUE) as u8;
}
#[no_mangle]
pub unsafe extern "C" fn SetMountedDeviceValues(
    List: *mut PARTLIST,
) -> i32 {
    let mut Entry: *mut LIST_ENTRY;
    let mut Volume: *mut VOLENTRY;

    if List.is_null() {
        return (FALSE) as i32;
    }

    Entry = (*List).VolumesList.Flink;
    while Entry != core::ptr::addr_of_mut!((*List).VolumesList) {
        Volume = CONTAINING_RECORD!(Entry, VOLENTRY, ListEntry);

        if SetMountedDeviceValue(Volume) == 0 {
            return (FALSE) as i32;
        }

        Entry = (*Entry).Flink;
    }

    return (TRUE) as i32;
}
#[no_mangle]
pub unsafe extern "C" fn SetMBRPartitionType(
    PartEntry: PPARTENTRY,
    PartitionType: u8,
) {
    let DiskEntry: PDISKENTRY = (*PartEntry).DiskEntry;

    if PartitionType as i32 == (*PartEntry).PartitionType as i32 {
        return;
    }

    (*PartEntry).PartitionType = PartitionType;

    (*DiskEntry).Dirty = (TRUE) as u8;
    (*(&raw mut (*(*DiskEntry).LayoutBuffer).PartitionEntry as *mut PARTITION_INFORMATION)
        .offset((*PartEntry).PartitionIndex as isize))
    .PartitionType = PartitionType;
    (*(&raw mut (*(*DiskEntry).LayoutBuffer).PartitionEntry as *mut PARTITION_INFORMATION)
        .offset((*PartEntry).PartitionIndex as isize))
    .RecognizedPartition = IsRecognizedPartition!(PartitionType) as BOOLEAN;
    (*(&raw mut (*(*DiskEntry).LayoutBuffer).PartitionEntry as *mut PARTITION_INFORMATION)
        .offset((*PartEntry).PartitionIndex as isize))
    .RewritePartition = (TRUE) as u8;
}
