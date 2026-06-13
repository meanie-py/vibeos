use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{IoGetCurrentIrpStackLocation, MOUNTMGR_MOUNT_POINT, MOUNTMGR_MOUNT_POINTS, PIO_STACK_LOCATION, PIRP, PMOUNTMGR_MOUNT_POINTS, PUNICODE_STRING, RtlWriteRegistryValue};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use ntkrnl_sys::ddk::{GUID, IO_STACK_LOCATION, IRP, _UNICODE_STRING};

// driver-internal cdecl decls (types_*.rs authority = extern "C").
unsafe extern "C" {
    fn QueryDeviceInformation( _: *mut _UNICODE_STRING, _: *mut _UNICODE_STRING, _: *mut *mut MOUNTDEV_UNIQUE_ID, _: *mut u8, _: *mut u8, _: *mut u8, _: *mut GUID, _: *mut u8, ) -> i32;
    fn GlobalCreateSymbolicLink(_: *mut _UNICODE_STRING, _: *mut _UNICODE_STRING) -> i32;
    fn GlobalDeleteSymbolicLink(_: *mut _UNICODE_STRING) -> i32;
    fn IsDriveLetter(_: *mut _UNICODE_STRING) -> i32;
    fn HasDriveLetter(_: *mut DEVICE_INFORMATION) -> i32;
    fn DeleteRegistryDriveLetter(_: *mut MOUNTDEV_UNIQUE_ID);
    fn DeleteNoDriveLetterEntry(_: *mut MOUNTDEV_UNIQUE_ID);
    fn SendLinkCreated(_: *mut _UNICODE_STRING);
    fn MountMgrNotify(_: *mut DEVICE_EXTENSION);
    fn MountMgrNotifyNameChange(_: *mut DEVICE_EXTENSION, _: *mut _UNICODE_STRING, _: u8);
    fn PostOnlineNotification(_: *mut DEVICE_EXTENSION, _: *mut _UNICODE_STRING);
    fn ReconcileThisDatabaseWithMaster(_: *mut DEVICE_EXTENSION, _: *mut DEVICE_INFORMATION);
}

extern "system" {
    fn AllocatePool(_: u32) -> *mut core::ffi::c_void;
    fn FreePool(_: *mut core::ffi::c_void);
    fn RtlCompareMemory( Source1: *const core::ffi::c_void, Source2: *const core::ffi::c_void, Length: usize, ) -> usize;
    fn RtlEqualUnicodeString( String1: *const _UNICODE_STRING, String2: *const _UNICODE_STRING, CaseInSensitive: BOOLEAN, ) -> BOOLEAN;
}

// ---- file-scope preamble (translated) ----
// NDEBUG is a preprocessor macro; no Rust equivalent declaration is needed.
#[no_mangle]
pub unsafe extern "C" fn MountMgrCreatePointWorker(
    DeviceExtension: *mut DEVICE_EXTENSION,
    SymbolicLinkName: *mut _UNICODE_STRING,
    DeviceName: *mut _UNICODE_STRING,
) -> i32 {
    unsafe {
        let mut Status: i32 = 0;
        let mut DeviceEntry: *mut LIST_ENTRY = core::ptr::null_mut();
        let mut UniqueId: *mut MOUNTDEV_UNIQUE_ID = core::ptr::null_mut();
        let mut SymlinkInformation: *mut SYMLINK_INFORMATION = core::ptr::null_mut();
        let mut SymLink: _UNICODE_STRING = _UNICODE_STRING {
            Length: 0,
            MaximumLength: 0,
            Buffer: core::ptr::null_mut(),
        };
        let mut TargetDeviceName: _UNICODE_STRING = _UNICODE_STRING {
            Length: 0,
            MaximumLength: 0,
            Buffer: core::ptr::null_mut(),
        };
        let mut DeviceInformation: *mut DEVICE_INFORMATION = core::ptr::null_mut();
        let mut DeviceInfo: *mut DEVICE_INFORMATION = core::ptr::null_mut();

        const UNICODE_NULL: u16 = 0;
        const RTL_REGISTRY_ABSOLUTE: u32 = 0;
        const REG_BINARY: u32 = 3;

        Status = QueryDeviceInformation(
            DeviceName,
            &raw mut TargetDeviceName,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        if Status < 0 {
            return Status;
        }

        DeviceEntry = (*DeviceExtension).DeviceListHead.Flink;
        while DeviceEntry != &raw mut (*DeviceExtension).DeviceListHead {
            DeviceInformation = (DeviceEntry as usize)
                .wrapping_sub(core::mem::offset_of!(DEVICE_INFORMATION, DeviceListEntry))
                as *mut DEVICE_INFORMATION;
            if RtlEqualUnicodeString(
                &raw const TargetDeviceName,
                &raw const (*DeviceInformation).DeviceName,
                1u8,
            ) != 0
            {
                break;
            }
            DeviceEntry = (*DeviceEntry).Flink;
        }

        SymLink.Buffer = AllocatePool(
            ((*SymbolicLinkName).Length as usize).wrapping_add(core::mem::size_of::<u16>()) as u32,
        ) as *mut u16;
        if SymLink.Buffer.is_null() {
            FreePool(TargetDeviceName.Buffer as *mut core::ffi::c_void);
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        RtlCopyMemory(
            SymLink.Buffer as *mut core::ffi::c_void,
            (*SymbolicLinkName).Buffer as *const core::ffi::c_void,
            (*SymbolicLinkName).Length as usize,
        );
        *SymLink.Buffer.add(
            ((*SymbolicLinkName).Length as usize) / core::mem::size_of::<u16>(),
        ) = UNICODE_NULL;
        SymLink.Length = (*SymbolicLinkName).Length;
        SymLink.MaximumLength = ((*SymbolicLinkName).Length as usize)
            .wrapping_add(core::mem::size_of::<u16>()) as u16;

        if DeviceEntry == &raw mut (*DeviceExtension).DeviceListHead {
            Status = QueryDeviceInformation(
                SymbolicLinkName,
                core::ptr::null_mut(),
                &raw mut UniqueId,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );
            if Status < 0 {
                FreePool(TargetDeviceName.Buffer as *mut core::ffi::c_void);
                FreePool(SymLink.Buffer as *mut core::ffi::c_void);
                return Status;
            }
            Status = GlobalCreateSymbolicLink(&raw mut SymLink, &raw mut TargetDeviceName);
            if Status < 0 {
                FreePool(UniqueId as *mut core::ffi::c_void);
                FreePool(TargetDeviceName.Buffer as *mut core::ffi::c_void);
                FreePool(SymLink.Buffer as *mut core::ffi::c_void);
                return Status;
            }
            if IsDriveLetter(&raw mut SymLink) != 0 {
                DeleteRegistryDriveLetter(UniqueId);
            }
            Status = RtlWriteRegistryValue(
                RTL_REGISTRY_ABSOLUTE,
                DatabasePath,
                SymLink.Buffer,
                REG_BINARY,
                &raw mut (*UniqueId).UniqueId as *mut u8 as *mut core::ffi::c_void,
                (*UniqueId).UniqueIdLength as u32,
            );
            FreePool(UniqueId as *mut core::ffi::c_void);
            FreePool(TargetDeviceName.Buffer as *mut core::ffi::c_void);
            FreePool(SymLink.Buffer as *mut core::ffi::c_void);
            return Status;
        }

        if IsDriveLetter(&raw mut SymLink) != 0 && HasDriveLetter(DeviceInformation) != 0 {
            FreePool(TargetDeviceName.Buffer as *mut core::ffi::c_void);
            FreePool(SymLink.Buffer as *mut core::ffi::c_void);
            return STATUS_INVALID_PARAMETER;
        }

        Status = GlobalCreateSymbolicLink(&raw mut SymLink, &raw mut TargetDeviceName);
        FreePool(TargetDeviceName.Buffer as *mut core::ffi::c_void);
        if Status < 0 {
            FreePool(SymLink.Buffer as *mut core::ffi::c_void);
            return Status;
        }

        UniqueId = (*DeviceInformation).UniqueId;
        Status = RtlWriteRegistryValue(
            RTL_REGISTRY_ABSOLUTE,
            DatabasePath,
            SymLink.Buffer,
            REG_BINARY,
            &raw mut (*UniqueId).UniqueId as *mut u8 as *mut core::ffi::c_void,
            (*UniqueId).UniqueIdLength as u32,
        );
        if Status < 0 {
            GlobalDeleteSymbolicLink(&raw mut SymLink);
            FreePool(SymLink.Buffer as *mut core::ffi::c_void);
            return Status;
        }

        SymlinkInformation = AllocatePool(core::mem::size_of::<SYMLINK_INFORMATION>() as u32)
            as *mut SYMLINK_INFORMATION;
        if SymlinkInformation.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            GlobalDeleteSymbolicLink(&raw mut SymLink);
            FreePool(SymLink.Buffer as *mut core::ffi::c_void);
            return Status;
        }

        (*SymlinkInformation).Name.Length = SymLink.Length;
        (*SymlinkInformation).Name.MaximumLength =
            (SymLink.Length as usize).wrapping_add(core::mem::size_of::<u16>()) as u16;
        (*SymlinkInformation).Name.Buffer = AllocatePool(
            (*SymlinkInformation).Name.MaximumLength as u32,
        ) as *mut u16;
        if (*SymlinkInformation).Name.Buffer.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            FreePool(SymlinkInformation as *mut core::ffi::c_void);
            GlobalDeleteSymbolicLink(&raw mut SymLink);
            FreePool(SymLink.Buffer as *mut core::ffi::c_void);
            return Status;
        }

        RtlCopyMemory(
            (*SymlinkInformation).Name.Buffer as *mut core::ffi::c_void,
            SymLink.Buffer as *const core::ffi::c_void,
            (*SymlinkInformation).Name.Length as usize,
        );
        *(*SymlinkInformation).Name.Buffer.add(
            ((*SymlinkInformation).Name.Length as usize) / core::mem::size_of::<u16>(),
        ) = UNICODE_NULL;
        (*SymlinkInformation).Online = 1u8;
        InsertTailList(
            &raw mut (*DeviceInformation).SymbolicLinksListHead,
            &raw mut (*SymlinkInformation).SymbolicLinksListEntry,
        );
        SendLinkCreated(&raw mut (*SymlinkInformation).Name);

        if IsDriveLetter(&raw mut SymLink) != 0 {
            DeleteNoDriveLetterEntry(UniqueId);
            if (*DeviceInformation).SkipNotifications == 0 {
                PostOnlineNotification(DeviceExtension, &raw mut (*DeviceInformation).SymbolicName);
            }
        }

        // MOUNTMGR_IS_VOLUME_NAME expanded per c2rust anchor
        if (SymLink.Length as i32 == 96
            || SymLink.Length as i32 == 98
                && *SymLink.Buffer.add(48) as i32 == '\\' as i32)
            && *SymLink.Buffer.add(0) as i32 == '\\' as i32
            && (*SymLink.Buffer.add(1) as i32 == '?' as i32
                || *SymLink.Buffer.add(1) as i32 == '\\' as i32)
            && *SymLink.Buffer.add(2) as i32 == '?' as i32
            && *SymLink.Buffer.add(3) as i32 == '\\' as i32
            && *SymLink.Buffer.add(4) as i32 == 'V' as i32
            && *SymLink.Buffer.add(5) as i32 == 'o' as i32
            && *SymLink.Buffer.add(6) as i32 == 'l' as i32
            && *SymLink.Buffer.add(7) as i32 == 'u' as i32
            && *SymLink.Buffer.add(8) as i32 == 'm' as i32
            && *SymLink.Buffer.add(9) as i32 == 'e' as i32
            && *SymLink.Buffer.add(10) as i32 == '{' as i32
            && *SymLink.Buffer.add(19) as i32 == '-' as i32
            && *SymLink.Buffer.add(24) as i32 == '-' as i32
            && *SymLink.Buffer.add(29) as i32 == '-' as i32
            && *SymLink.Buffer.add(34) as i32 == '-' as i32
            && *SymLink.Buffer.add(47) as i32 == '}' as i32
            && (*DeviceExtension).AutomaticDriveLetter as i32 != 0
        {
            DeviceEntry = (*DeviceExtension).DeviceListHead.Flink;
            while DeviceEntry != &raw mut (*DeviceExtension).DeviceListHead {
                DeviceInfo = (DeviceEntry as usize)
                    .wrapping_sub(core::mem::offset_of!(DEVICE_INFORMATION, DeviceListEntry))
                    as *mut DEVICE_INFORMATION;
                if (*DeviceInfo).NoDatabase == 0 {
                    ReconcileThisDatabaseWithMaster(DeviceExtension, DeviceInfo);
                }
                DeviceEntry = (*DeviceEntry).Flink;
            }
        }

        FreePool(SymLink.Buffer as *mut core::ffi::c_void);
        MountMgrNotify(DeviceExtension);
        if (*DeviceInformation).ManuallyRegistered == 0 {
            MountMgrNotifyNameChange(DeviceExtension, DeviceName, 0u8);
        }

        Status
    }
}

#[no_mangle]
pub unsafe extern "C" fn QueryPointsFromMemory(
    DeviceExtension: *mut DEVICE_EXTENSION,
    Irp: *mut IRP,
    UniqueId: *mut MOUNTDEV_UNIQUE_ID,
    SymbolicName: *mut _UNICODE_STRING,
) -> NTSTATUS {
    // Forward declarations for external functions not in crate

    let mut Status: NTSTATUS;
    let mut Stack: *mut IO_STACK_LOCATION;
    let mut DeviceName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut MountPoints: *mut MOUNTMGR_MOUNT_POINTS;
    let mut DeviceInformation: *mut DEVICE_INFORMATION;
    let mut DeviceEntry: *mut LIST_ENTRY;
    let mut SymlinksEntry: *mut LIST_ENTRY;
    let mut SymlinkInformation: *mut SYMLINK_INFORMATION;
    let mut UniqueIdLength: USHORT;
    let mut DeviceNameLength: USHORT;
    let mut TotalSize: ULONG;
    let mut TotalSymLinks: ULONG;
    let mut UniqueIdOffset: ULONG;
    let mut DeviceNameOffset: ULONG;

    if !SymbolicName.is_null() {
        Status = QueryDeviceInformation(
            SymbolicName,
            &raw mut DeviceName,
            core::ptr::null_mut::<*mut MOUNTDEV_UNIQUE_ID>(),
            core::ptr::null_mut::<BOOLEAN>(),
            core::ptr::null_mut::<BOOLEAN>(),
            core::ptr::null_mut::<BOOLEAN>(),
            core::ptr::null_mut::<GUID>(),
            core::ptr::null_mut(),
        );
        if Status < 0 {
            return Status;
        }
    }

    TotalSize = 0;
    TotalSymLinks = 0;

    let head_ptr = core::ptr::addr_of_mut!((*DeviceExtension).DeviceListHead);
    DeviceEntry = (*DeviceExtension).DeviceListHead.Flink;
    while DeviceEntry != head_ptr {
        DeviceInformation = ((DeviceEntry as usize)
            .wrapping_sub(core::mem::offset_of!(DEVICE_INFORMATION, DeviceListEntry)))
            as *mut DEVICE_INFORMATION;

        if !UniqueId.is_null() {
            if (*UniqueId).UniqueIdLength != (*(*DeviceInformation).UniqueId).UniqueIdLength {
                DeviceEntry = (*DeviceEntry).Flink;
                continue;
            }
            if RtlCompareMemory(
                (*UniqueId).UniqueId.as_ptr() as *const core::ffi::c_void,
                (*(*DeviceInformation).UniqueId).UniqueId.as_ptr() as *const core::ffi::c_void,
                (*UniqueId).UniqueIdLength as usize,
            ) != (*UniqueId).UniqueIdLength as usize
            {
                DeviceEntry = (*DeviceEntry).Flink;
                continue;
            }
        } else if !SymbolicName.is_null() {
            if RtlEqualUnicodeString(
                &raw const DeviceName,
                &raw const (*DeviceInformation).DeviceName,
                1,
            ) == 0
            {
                DeviceEntry = (*DeviceEntry).Flink;
                continue;
            }
        }

        TotalSize = TotalSize.wrapping_add(
            ((*DeviceInformation).DeviceName.Length as usize
                + (*(*DeviceInformation).UniqueId).UniqueIdLength as usize) as ULONG,
        );

        let sym_head_ptr =
            core::ptr::addr_of_mut!((*DeviceInformation).SymbolicLinksListHead);
        SymlinksEntry = (*DeviceInformation).SymbolicLinksListHead.Flink;
        while SymlinksEntry != sym_head_ptr {
            SymlinkInformation = ((SymlinksEntry as usize)
                .wrapping_sub(core::mem::offset_of!(SYMLINK_INFORMATION, SymbolicLinksListEntry)))
                as *mut SYMLINK_INFORMATION;

            TotalSize = TotalSize.wrapping_add((*SymlinkInformation).Name.Length as ULONG);
            TotalSymLinks = TotalSymLinks.wrapping_add(1);
            SymlinksEntry = (*SymlinksEntry).Flink;
        }

        if !UniqueId.is_null() || !SymbolicName.is_null() {
            break;
        }

        DeviceEntry = (*DeviceEntry).Flink;
    }

    if !UniqueId.is_null() || !SymbolicName.is_null() {
        if DeviceEntry == head_ptr {
            if !SymbolicName.is_null() {
                FreePool(DeviceName.Buffer as *mut core::ffi::c_void);
            }
            return STATUS_INVALID_PARAMETER;
        }
    }

    Stack = IoGetCurrentIrpStackLocation(Irp);
    MountPoints = (*Irp).AssociatedIrp.SystemBuffer as *mut MOUNTMGR_MOUNT_POINTS;
    core::ptr::write_bytes(
        MountPoints as *mut u8,
        0,
        (*Stack).Parameters.DeviceIoControl.OutputBufferLength as usize,
    );

    (*MountPoints).Size = (core::mem::size_of::<MOUNTMGR_MOUNT_POINTS>()
        + TotalSymLinks as usize * core::mem::size_of::<MOUNTMGR_MOUNT_POINT>()
        + TotalSize as usize) as ULONG;
    (*MountPoints).NumberOfMountPoints = TotalSymLinks;
    (*Irp).IoStatus.Information = (*MountPoints).Size as _;

    if (*MountPoints).Size > (*Stack).Parameters.DeviceIoControl.OutputBufferLength {
        (*Irp).IoStatus.Information = core::mem::size_of::<MOUNTMGR_MOUNT_POINTS>() as _;
        if !SymbolicName.is_null() {
            FreePool(DeviceName.Buffer as *mut core::ffi::c_void);
        }
        return STATUS_BUFFER_OVERFLOW;
    }

    TotalSize = (core::mem::size_of::<MOUNTMGR_MOUNT_POINTS>()
        + TotalSymLinks as usize * core::mem::size_of::<MOUNTMGR_MOUNT_POINT>()) as ULONG;
    TotalSymLinks = 0;

    DeviceEntry = (*DeviceExtension).DeviceListHead.Flink;
    while DeviceEntry != head_ptr {
        DeviceInformation = ((DeviceEntry as usize)
            .wrapping_sub(core::mem::offset_of!(DEVICE_INFORMATION, DeviceListEntry)))
            as *mut DEVICE_INFORMATION;

        if !UniqueId.is_null() {
            if (*UniqueId).UniqueIdLength != (*(*DeviceInformation).UniqueId).UniqueIdLength {
                DeviceEntry = (*DeviceEntry).Flink;
                continue;
            }
            if RtlCompareMemory(
                (*UniqueId).UniqueId.as_ptr() as *const core::ffi::c_void,
                (*(*DeviceInformation).UniqueId).UniqueId.as_ptr() as *const core::ffi::c_void,
                (*UniqueId).UniqueIdLength as usize,
            ) != (*UniqueId).UniqueIdLength as usize
            {
                DeviceEntry = (*DeviceEntry).Flink;
                continue;
            }
        } else if !SymbolicName.is_null() {
            if RtlEqualUnicodeString(
                &raw const DeviceName,
                &raw const (*DeviceInformation).DeviceName,
                1,
            ) == 0
            {
                DeviceEntry = (*DeviceEntry).Flink;
                continue;
            }
        }

        UniqueIdOffset = TotalSize;
        UniqueIdLength = (*(*DeviceInformation).UniqueId).UniqueIdLength;
        DeviceNameOffset = TotalSize.wrapping_add(UniqueIdLength as ULONG);
        DeviceNameLength = (*DeviceInformation).DeviceName.Length;

        // Set device mount point (this entry will be overwritten by the first symlink)
        let mount_point =
            &mut *((*MountPoints).MountPoints.as_mut_ptr().add(TotalSymLinks as usize));
        mount_point.UniqueIdOffset = UniqueIdOffset;
        mount_point.UniqueIdLength = UniqueIdLength;
        mount_point.DeviceNameOffset = DeviceNameOffset;
        mount_point.DeviceNameLength = DeviceNameLength;

        // Copy unique ID and device name
        core::ptr::copy_nonoverlapping(
            (*(*DeviceInformation).UniqueId).UniqueId.as_ptr(),
            (MountPoints as *mut u8).add(UniqueIdOffset as usize),
            UniqueIdLength as usize,
        );
        core::ptr::copy_nonoverlapping(
            (*DeviceInformation).DeviceName.Buffer as *const u8,
            (MountPoints as *mut u8).add(DeviceNameOffset as usize),
            DeviceNameLength as usize,
        );

        TotalSize = TotalSize.wrapping_add(
            ((*(*DeviceInformation).UniqueId).UniqueIdLength as usize
                + (*DeviceInformation).DeviceName.Length as usize) as ULONG,
        );

        // Populate symlinks
        let sym_head_ptr =
            core::ptr::addr_of_mut!((*DeviceInformation).SymbolicLinksListHead);
        SymlinksEntry = (*DeviceInformation).SymbolicLinksListHead.Flink;
        while SymlinksEntry != sym_head_ptr {
            SymlinkInformation = ((SymlinksEntry as usize)
                .wrapping_sub(core::mem::offset_of!(SYMLINK_INFORMATION, SymbolicLinksListEntry)))
                as *mut SYMLINK_INFORMATION;

            let sym_mount_point =
                &mut *((*MountPoints).MountPoints.as_mut_ptr().add(TotalSymLinks as usize));

            if (*SymlinkInformation).Online != 0 {
                sym_mount_point.UniqueIdOffset = UniqueIdOffset;
                sym_mount_point.UniqueIdLength = UniqueIdLength;
            } else {
                sym_mount_point.UniqueIdOffset = 0;
                sym_mount_point.UniqueIdLength = 0;
            }

            sym_mount_point.DeviceNameOffset = DeviceNameOffset;
            sym_mount_point.DeviceNameLength = DeviceNameLength;
            sym_mount_point.SymbolicLinkNameOffset = TotalSize;
            sym_mount_point.SymbolicLinkNameLength = (*SymlinkInformation).Name.Length;

            core::ptr::copy_nonoverlapping(
                (*SymlinkInformation).Name.Buffer as *const u8,
                (MountPoints as *mut u8).add(sym_mount_point.SymbolicLinkNameOffset as usize),
                (*SymlinkInformation).Name.Length as usize,
            );

            TotalSymLinks = TotalSymLinks.wrapping_add(1);
            TotalSize = TotalSize.wrapping_add((*SymlinkInformation).Name.Length as ULONG);
            SymlinksEntry = (*SymlinksEntry).Flink;
        }

        if !UniqueId.is_null() || !SymbolicName.is_null() {
            break;
        }

        DeviceEntry = (*DeviceEntry).Flink;
    }

    if !SymbolicName.is_null() {
        FreePool(DeviceName.Buffer as *mut core::ffi::c_void);
    }

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn QueryPointsFromSymbolicLinkName(
    DeviceExtension: PDEVICE_EXTENSION,
    SymbolicName: PUNICODE_STRING,
    Irp: PIRP,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut TotalLength: ULONG;
    let mut Stack: PIO_STACK_LOCATION;
    let mut DeviceName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut::<u16>(),
    };
    let mut MountPoints: PMOUNTMGR_MOUNT_POINTS;
    let mut DeviceInformation: PDEVICE_INFORMATION = core::ptr::null_mut::<DEVICE_INFORMATION>();
    let mut DeviceEntry: PLIST_ENTRY;
    let mut SymlinksEntry: PLIST_ENTRY;
    let mut SymlinkInformation: PSYMLINK_INFORMATION = core::ptr::null_mut::<SYMLINK_INFORMATION>();

    // Find device
    Status = QueryDeviceInformation(
        SymbolicName,
        &raw mut DeviceName,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    if Status >= 0 {
        // Look for the device information
        DeviceEntry = (*DeviceExtension).DeviceListHead.Flink as PLIST_ENTRY;
        while DeviceEntry != &raw mut (*DeviceExtension).DeviceListHead {
            DeviceInformation = ((DeviceEntry as usize).wrapping_sub(
                core::mem::offset_of!(DEVICE_INFORMATION, DeviceListEntry),
            )) as *mut DEVICE_INFORMATION;

            if RtlEqualUnicodeString(
                &raw mut DeviceName,
                &raw mut (*DeviceInformation).DeviceName,
                1u8,
            ) != 0
            {
                break;
            }
            DeviceEntry = (*DeviceEntry).Flink as PLIST_ENTRY;
        }

        FreePool(DeviceName.Buffer as *mut core::ffi::c_void);

        if DeviceEntry == &raw mut (*DeviceExtension).DeviceListHead {
            return STATUS_INVALID_PARAMETER;
        }

        // Check for the link
        SymlinksEntry = (*DeviceInformation).SymbolicLinksListHead.Flink as PLIST_ENTRY;
        while SymlinksEntry != &raw mut (*DeviceInformation).SymbolicLinksListHead {
            SymlinkInformation = ((SymlinksEntry as usize).wrapping_sub(
                core::mem::offset_of!(SYMLINK_INFORMATION, SymbolicLinksListEntry),
            )) as *mut SYMLINK_INFORMATION;

            if RtlEqualUnicodeString(
                SymbolicName as *const _UNICODE_STRING,
                &raw mut (*SymlinkInformation).Name,
                1u8,
            ) != 0
            {
                break;
            }
            SymlinksEntry = (*SymlinksEntry).Flink as PLIST_ENTRY;
        }

        if SymlinksEntry == &raw mut (*DeviceInformation).SymbolicLinksListHead {
            return STATUS_INVALID_PARAMETER;
        }
    } else {
        // Browse all the devices to try to find the one that has the given link
        DeviceEntry = (*DeviceExtension).DeviceListHead.Flink as PLIST_ENTRY;
        while DeviceEntry != &raw mut (*DeviceExtension).DeviceListHead {
            DeviceInformation = ((DeviceEntry as usize).wrapping_sub(
                core::mem::offset_of!(DEVICE_INFORMATION, DeviceListEntry),
            )) as *mut DEVICE_INFORMATION;

            SymlinksEntry = (*DeviceInformation).SymbolicLinksListHead.Flink as PLIST_ENTRY;
            while SymlinksEntry != &raw mut (*DeviceInformation).SymbolicLinksListHead {
                SymlinkInformation = ((SymlinksEntry as usize).wrapping_sub(
                    core::mem::offset_of!(SYMLINK_INFORMATION, SymbolicLinksListEntry),
                )) as *mut SYMLINK_INFORMATION;

                if RtlEqualUnicodeString(
                    SymbolicName as *const _UNICODE_STRING,
                    &raw mut (*SymlinkInformation).Name,
                    1u8,
                ) != 0
                {
                    break;
                }
                SymlinksEntry = (*SymlinksEntry).Flink as PLIST_ENTRY;
            }

            if SymlinksEntry != &raw mut (*DeviceInformation).SymbolicLinksListHead {
                break;
            }
            DeviceEntry = (*DeviceEntry).Flink as PLIST_ENTRY;
        }

        // Even that way we didn't find, give up!
        if DeviceEntry == &raw mut (*DeviceExtension).DeviceListHead {
            return STATUS_OBJECT_NAME_NOT_FOUND;
        }
    }

    // Get output buffer
    Stack = IoGetCurrentIrpStackLocation(Irp);
    MountPoints = (*Irp).AssociatedIrp.SystemBuffer as PMOUNTMGR_MOUNT_POINTS;

    // Compute output length
    TotalLength = (*(*DeviceInformation).UniqueId).UniqueIdLength as u32
        + (*SymlinkInformation).Name.Length as u32
        + (*DeviceInformation).DeviceName.Length as u32;

    // Give length to allow reallocation
    (*MountPoints).Size = (core::mem::size_of::<MOUNTMGR_MOUNT_POINTS>() as u32).wrapping_add(TotalLength);
    (*MountPoints).NumberOfMountPoints = 1;
    (*Irp).IoStatus.Information = (core::mem::size_of::<MOUNTMGR_MOUNT_POINTS>() as usize)
        .wrapping_add(TotalLength as usize) as _;

    if (*MountPoints).Size > (*Stack).Parameters.DeviceIoControl.OutputBufferLength {
        (*Irp).IoStatus.Information = core::mem::size_of::<MOUNTMGR_MOUNT_POINTS>() as _;
        return STATUS_BUFFER_OVERFLOW;
    }

    // Write out data
    (*MountPoints).MountPoints[0].SymbolicLinkNameOffset = core::mem::size_of::<MOUNTMGR_MOUNT_POINTS>() as u32;
    (*MountPoints).MountPoints[0].SymbolicLinkNameLength = (*SymlinkInformation).Name.Length;

    // If link is online write it's unique ID, otherwise, forget about it
    if (*SymlinkInformation).Online != 0 {
        (*MountPoints).MountPoints[0].UniqueIdOffset = (core::mem::size_of::<MOUNTMGR_MOUNT_POINTS>() as u32)
            .wrapping_add((*SymlinkInformation).Name.Length as u32);
        (*MountPoints).MountPoints[0].UniqueIdLength = (*(*DeviceInformation).UniqueId).UniqueIdLength;
    } else {
        (*MountPoints).MountPoints[0].UniqueIdOffset = 0;
        (*MountPoints).MountPoints[0].UniqueIdLength = 0;
    }

    (*MountPoints).MountPoints[0].DeviceNameOffset = (core::mem::size_of::<MOUNTMGR_MOUNT_POINTS>() as u32)
        .wrapping_add((*SymlinkInformation).Name.Length as u32)
        .wrapping_add((*(*DeviceInformation).UniqueId).UniqueIdLength as u32);
    (*MountPoints).MountPoints[0].DeviceNameLength = (*DeviceInformation).DeviceName.Length;

    // Copy symbolic link name
    let dst = (MountPoints as *mut u8).add((*MountPoints).MountPoints[0].SymbolicLinkNameOffset as usize);
    core::ptr::copy_nonoverlapping(
        (*SymlinkInformation).Name.Buffer as *const u8,
        dst,
        (*SymlinkInformation).Name.Length as usize,
    );

    if (*SymlinkInformation).Online != 0 {
        let dst = (MountPoints as *mut u8).add((*MountPoints).MountPoints[0].UniqueIdOffset as usize);
        core::ptr::copy_nonoverlapping(
            &raw const (*(*DeviceInformation).UniqueId).UniqueId as *const u8,
            dst,
            (*(*DeviceInformation).UniqueId).UniqueIdLength as usize,
        );
    }

    let dst = (MountPoints as *mut u8).add((*MountPoints).MountPoints[0].DeviceNameOffset as usize);
    core::ptr::copy_nonoverlapping(
        (*DeviceInformation).DeviceName.Buffer as *const u8,
        dst,
        (*DeviceInformation).DeviceName.Length as usize,
    );

    STATUS_SUCCESS
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  *  ReactOS kernel
//  *  Copyright (C) 2011-2012 ReactOS Team
//  *
//  *  This program is free software; you can redistribute it and/or modify
//  *  it under the terms of the GNU General Public License as published by
//  *  the Free Software Foundation; either version 2 of the License, or
//  *  (at your option) any later version.
//  *
//  *  This program is distributed in the hope that it will be useful,
//  *  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  *  GNU General Public License for more details.
//  *
//  *  You should have received a copy of the GNU General Public License
//  *  along with this program; if not, write to the Free Software
//  *  Foundation, Inc., 51 Franklin St, Fifth Floor, Boston, MA 02110-1301, USA.
//  *
//  * COPYRIGHT:        See COPYING in the top level directory
//  * PROJECT:          ReactOS kernel
//  * FILE:             drivers/filesystem/mountmgr/point.c
//  * PURPOSE:          Mount Manager - Mount points
//  * PROGRAMMER:       Pierre Schweitzer (pierre.schweitzer@reactos.org)
//  */
//
// #include "mntmgr.h"
//
// #define NDEBUG
// #include <debug.h>
//
// /*
//  * @implemented
//  */
// NTSTATUS
// MountMgrCreatePointWorker(IN PDEVICE_EXTENSION DeviceExtension,
//                           IN PUNICODE_STRING SymbolicLinkName,
//                           IN PUNICODE_STRING DeviceName)
// {
//     NTSTATUS Status;
//     PLIST_ENTRY DeviceEntry;
//     PMOUNTDEV_UNIQUE_ID UniqueId;
//     PSYMLINK_INFORMATION SymlinkInformation;
//     UNICODE_STRING SymLink, TargetDeviceName;
//     PDEVICE_INFORMATION DeviceInformation = NULL, DeviceInfo;
//
//     /* Get device name */
//     Status = QueryDeviceInformation(DeviceName,
//                                     &TargetDeviceName,
//                                     NULL, NULL, NULL,
//                                     NULL, NULL, NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//
//     /* First of all, try to find device */
//     for (DeviceEntry = DeviceExtension->DeviceListHead.Flink;
//          DeviceEntry != &(DeviceExtension->DeviceListHead);
//          DeviceEntry = DeviceEntry->Flink)
//     {
//         DeviceInformation = CONTAINING_RECORD(DeviceEntry, DEVICE_INFORMATION, DeviceListEntry);
//
//         if (RtlEqualUnicodeString(&TargetDeviceName, &(DeviceInformation->DeviceName), TRUE))
//         {
//             break;
//         }
//     }
//
//     /* Copy symbolic link name and null terminate it */
//     SymLink.Buffer = AllocatePool(SymbolicLinkName->Length + sizeof(UNICODE_NULL));
//     if (!SymLink.Buffer)
//     {
//         FreePool(TargetDeviceName.Buffer);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     RtlCopyMemory(SymLink.Buffer, SymbolicLinkName->Buffer, SymbolicLinkName->Length);
//     SymLink.Buffer[SymbolicLinkName->Length / sizeof(WCHAR)] = UNICODE_NULL;
//     SymLink.Length = SymbolicLinkName->Length;
//     SymLink.MaximumLength = SymbolicLinkName->Length + sizeof(UNICODE_NULL);
//
//     /* If we didn't find device */
//     if (DeviceEntry == &(DeviceExtension->DeviceListHead))
//     {
//         /* Then, try with unique ID */
//         Status = QueryDeviceInformation(SymbolicLinkName,
//                                         NULL, &UniqueId,
//                                         NULL, NULL, NULL,
//                                         NULL, NULL);
//         if (!NT_SUCCESS(Status))
//         {
//             FreePool(TargetDeviceName.Buffer);
//             FreePool(SymLink.Buffer);
//             return Status;
//         }
//
//         /* Create a link to the device */
//         Status = GlobalCreateSymbolicLink(&SymLink, &TargetDeviceName);
//         if (!NT_SUCCESS(Status))
//         {
//             FreePool(UniqueId);
//             FreePool(TargetDeviceName.Buffer);
//             FreePool(SymLink.Buffer);
//             return Status;
//         }
//
//         /* If caller provided driver letter, delete it */
//         if (IsDriveLetter(&SymLink))
//         {
//             DeleteRegistryDriveLetter(UniqueId);
//         }
//
//         /* Device will be identified with its unique ID */
//         Status = RtlWriteRegistryValue(RTL_REGISTRY_ABSOLUTE,
//                                        DatabasePath,
//                                        SymLink.Buffer,
//                                        REG_BINARY,
//                                        UniqueId->UniqueId,
//                                        UniqueId->UniqueIdLength);
//
//         FreePool(UniqueId);
//         FreePool(TargetDeviceName.Buffer);
//         FreePool(SymLink.Buffer);
//         return Status;
//     }
//
//     /* If call provided a driver letter whereas device already has one
//      * fail, this is not doable
//      */
//     if (IsDriveLetter(&SymLink) && HasDriveLetter(DeviceInformation))
//     {
//         FreePool(TargetDeviceName.Buffer);
//         FreePool(SymLink.Buffer);
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Now, create a link */
//     Status = GlobalCreateSymbolicLink(&SymLink, &TargetDeviceName);
//     FreePool(TargetDeviceName.Buffer);
//     if (!NT_SUCCESS(Status))
//     {
//         FreePool(SymLink.Buffer);
//         return Status;
//     }
//
//     /* Associate Unique ID <-> symbolic name */
//     UniqueId = DeviceInformation->UniqueId;
//     Status = RtlWriteRegistryValue(RTL_REGISTRY_ABSOLUTE,
//                                    DatabasePath,
//                                    SymLink.Buffer,
//                                    REG_BINARY,
//                                    UniqueId->UniqueId,
//                                    UniqueId->UniqueIdLength);
//     if (!NT_SUCCESS(Status))
//     {
//         GlobalDeleteSymbolicLink(&SymLink);
//         FreePool(SymLink.Buffer);
//         return Status;
//     }
//
//     /* Now, prepare to save the link with the device */
//     SymlinkInformation = AllocatePool(sizeof(SYMLINK_INFORMATION));
//     if (!SymlinkInformation)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         GlobalDeleteSymbolicLink(&SymLink);
//         FreePool(SymLink.Buffer);
//         return Status;
//     }
//
//     SymlinkInformation->Name.Length = SymLink.Length;
//     SymlinkInformation->Name.MaximumLength = SymLink.Length + sizeof(UNICODE_NULL);
//     SymlinkInformation->Name.Buffer = AllocatePool(SymlinkInformation->Name.MaximumLength);
//     if (!SymlinkInformation->Name.Buffer)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         FreePool(SymlinkInformation);
//         GlobalDeleteSymbolicLink(&SymLink);
//         FreePool(SymLink.Buffer);
//         return Status;
//     }
//
//     /* Save the link and mark it online */
//     RtlCopyMemory(SymlinkInformation->Name.Buffer, SymLink.Buffer, SymlinkInformation->Name.Length);
//     SymlinkInformation->Name.Buffer[SymlinkInformation->Name.Length / sizeof(WCHAR)] = UNICODE_NULL;
//     SymlinkInformation->Online = TRUE;
//     InsertTailList(&DeviceInformation->SymbolicLinksListHead, &SymlinkInformation->SymbolicLinksListEntry);
//     SendLinkCreated(&(SymlinkInformation->Name));
//
//     /* If we have a drive letter */
//     if (IsDriveLetter(&SymLink))
//     {
//         /* Then, delete the no drive letter entry */
//         DeleteNoDriveLetterEntry(UniqueId);
//
//         /* And post online notification if asked */
//         if (!DeviceInformation->SkipNotifications)
//         {
//             PostOnlineNotification(DeviceExtension, &DeviceInformation->SymbolicName);
//         }
//     }
//
//     /* If that's a volume with automatic drive letter, it's now time to resync databases */
//     if (MOUNTMGR_IS_VOLUME_NAME(&SymLink) && DeviceExtension->AutomaticDriveLetter)
//     {
//         for (DeviceEntry = DeviceExtension->DeviceListHead.Flink;
//              DeviceEntry != &(DeviceExtension->DeviceListHead);
//              DeviceEntry = DeviceEntry->Flink)
//         {
//             DeviceInfo = CONTAINING_RECORD(DeviceEntry, DEVICE_INFORMATION, DeviceListEntry);
//
//             /* If there's one, ofc! */
//             if (!DeviceInfo->NoDatabase)
//             {
//                 ReconcileThisDatabaseWithMaster(DeviceExtension, DeviceInfo);
//             }
//         }
//     }
//
//     /* Notify & quit */
//     FreePool(SymLink.Buffer);
//     MountMgrNotify(DeviceExtension);
//
//     if (!DeviceInformation->ManuallyRegistered)
//     {
//         MountMgrNotifyNameChange(DeviceExtension, DeviceName, FALSE);
//     }
//
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// QueryPointsFromMemory(IN PDEVICE_EXTENSION DeviceExtension,
//                       IN PIRP Irp,
//                       IN PMOUNTDEV_UNIQUE_ID UniqueId OPTIONAL,
//                       IN PUNICODE_STRING SymbolicName OPTIONAL)
// {
//     NTSTATUS Status;
//     PIO_STACK_LOCATION Stack;
//     UNICODE_STRING DeviceName;
//     PMOUNTMGR_MOUNT_POINTS MountPoints;
//     PDEVICE_INFORMATION DeviceInformation;
//     PLIST_ENTRY DeviceEntry, SymlinksEntry;
//     PSYMLINK_INFORMATION SymlinkInformation;
//     USHORT UniqueIdLength, DeviceNameLength;
//     ULONG TotalSize, TotalSymLinks, UniqueIdOffset, DeviceNameOffset;
//
//     /* If we got a symbolic link, query device */
//     if (SymbolicName)
//     {
//         Status = QueryDeviceInformation(SymbolicName,
//                                         &DeviceName,
//                                         NULL, NULL,
//                                         NULL, NULL,
//                                         NULL, NULL);
//         if (!NT_SUCCESS(Status))
//         {
//             return Status;
//         }
//     }
//
//     /* Browse all the links to count number of links & size used */
//     TotalSize = 0;
//     TotalSymLinks = 0;
//     for (DeviceEntry = DeviceExtension->DeviceListHead.Flink;
//          DeviceEntry != &(DeviceExtension->DeviceListHead);
//          DeviceEntry = DeviceEntry->Flink)
//     {
//         DeviceInformation = CONTAINING_RECORD(DeviceEntry, DEVICE_INFORMATION, DeviceListEntry);
//
//         /* If we were given an unique ID, it has to match */
//         if (UniqueId)
//         {
//             if (UniqueId->UniqueIdLength != DeviceInformation->UniqueId->UniqueIdLength)
//             {
//                 continue;
//             }
//
//             if (RtlCompareMemory(UniqueId->UniqueId,
//                                  DeviceInformation->UniqueId->UniqueId,
//                                  UniqueId->UniqueIdLength) != UniqueId->UniqueIdLength)
//             {
//                 continue;
//             }
//         }
//         /* Or, if we had a symlink, it has to match */
//         else if (SymbolicName)
//         {
//             if (!RtlEqualUnicodeString(&DeviceName, &(DeviceInformation->DeviceName), TRUE))
//             {
//                 continue;
//             }
//         }
//
//         /* Once here, it matched, save device name & unique ID size */
//         TotalSize += DeviceInformation->DeviceName.Length + DeviceInformation->UniqueId->UniqueIdLength;
//
//         /* And count number of symlinks (and their size) */
//         for (SymlinksEntry = DeviceInformation->SymbolicLinksListHead.Flink;
//              SymlinksEntry != &(DeviceInformation->SymbolicLinksListHead);
//              SymlinksEntry = SymlinksEntry->Flink)
//         {
//             SymlinkInformation = CONTAINING_RECORD(SymlinksEntry, SYMLINK_INFORMATION, SymbolicLinksListEntry);
//
//             TotalSize += SymlinkInformation->Name.Length;
//             TotalSymLinks++;
//         }
//
//         /* We had a specific item to find
//          * if we reach that point, we found it, no need to continue
//          */
//         if (UniqueId || SymbolicName)
//         {
//             break;
//         }
//     }
//
//     /* If we were looking for specific item, ensure we found it */
//     if (UniqueId || SymbolicName)
//     {
//         if (DeviceEntry == &(DeviceExtension->DeviceListHead))
//         {
//             if (SymbolicName)
//             {
//                 FreePool(DeviceName.Buffer);
//             }
//
//             return STATUS_INVALID_PARAMETER;
//         }
//     }
//
//     /* Now, ensure output buffer can hold everything */
//     Stack = IoGetCurrentIrpStackLocation(Irp);
//     MountPoints = (PMOUNTMGR_MOUNT_POINTS)Irp->AssociatedIrp.SystemBuffer;
//     RtlZeroMemory(MountPoints, Stack->Parameters.DeviceIoControl.OutputBufferLength);
//
//     /* Ensure we set output to let user reallocate! */
//     MountPoints->Size = sizeof(MOUNTMGR_MOUNT_POINTS) + TotalSymLinks * sizeof(MOUNTMGR_MOUNT_POINT) + TotalSize;
//     MountPoints->NumberOfMountPoints = TotalSymLinks;
//     Irp->IoStatus.Information = MountPoints->Size;
//
//     if (MountPoints->Size > Stack->Parameters.DeviceIoControl.OutputBufferLength)
//     {
//         Irp->IoStatus.Information = sizeof(MOUNTMGR_MOUNT_POINTS);
//
//         if (SymbolicName)
//         {
//             FreePool(DeviceName.Buffer);
//         }
//
//         return STATUS_BUFFER_OVERFLOW;
//     }
//
//     /* Now, start putting mount points */
//     TotalSize = sizeof(MOUNTMGR_MOUNT_POINTS) + TotalSymLinks * sizeof(MOUNTMGR_MOUNT_POINT);
//     TotalSymLinks = 0;
//     for (DeviceEntry = DeviceExtension->DeviceListHead.Flink;
//          DeviceEntry != &(DeviceExtension->DeviceListHead);
//          DeviceEntry = DeviceEntry->Flink)
//     {
//         DeviceInformation = CONTAINING_RECORD(DeviceEntry, DEVICE_INFORMATION, DeviceListEntry);
//
//         /* Find back correct mount point */
//         if (UniqueId)
//         {
//             if (UniqueId->UniqueIdLength != DeviceInformation->UniqueId->UniqueIdLength)
//             {
//                 continue;
//             }
//
//             if (RtlCompareMemory(UniqueId->UniqueId,
//                                  DeviceInformation->UniqueId->UniqueId,
//                                  UniqueId->UniqueIdLength) != UniqueId->UniqueIdLength)
//             {
//                 continue;
//             }
//         }
//         else if (SymbolicName)
//         {
//             if (!RtlEqualUnicodeString(&DeviceName, &(DeviceInformation->DeviceName), TRUE))
//             {
//                 continue;
//             }
//         }
//
//         /* Save our information about shared data */
//         UniqueIdOffset = TotalSize;
//         UniqueIdLength = DeviceInformation->UniqueId->UniqueIdLength;
//         DeviceNameOffset = TotalSize + UniqueIdLength;
//         DeviceNameLength = DeviceInformation->DeviceName.Length;
//
//         /* Initialize first symlink */
//         MountPoints->MountPoints[TotalSymLinks].UniqueIdOffset = UniqueIdOffset;
//         MountPoints->MountPoints[TotalSymLinks].UniqueIdLength = UniqueIdLength;
//         MountPoints->MountPoints[TotalSymLinks].DeviceNameOffset = DeviceNameOffset;
//         MountPoints->MountPoints[TotalSymLinks].DeviceNameLength = DeviceNameLength;
//
//         /* And copy data */
//         RtlCopyMemory((PWSTR)((ULONG_PTR)MountPoints + UniqueIdOffset),
//                       DeviceInformation->UniqueId->UniqueId, UniqueIdLength);
//         RtlCopyMemory((PWSTR)((ULONG_PTR)MountPoints + DeviceNameOffset),
//                       DeviceInformation->DeviceName.Buffer, DeviceNameLength);
//
//         TotalSize += DeviceInformation->UniqueId->UniqueIdLength + DeviceInformation->DeviceName.Length;
//
//         /* Now we've got it, but all the data */
//         for (SymlinksEntry = DeviceInformation->SymbolicLinksListHead.Flink;
//              SymlinksEntry != &(DeviceInformation->SymbolicLinksListHead);
//              SymlinksEntry = SymlinksEntry->Flink)
//         {
//             SymlinkInformation = CONTAINING_RECORD(SymlinksEntry, SYMLINK_INFORMATION, SymbolicLinksListEntry);
//
//             /* First, set shared data */
//
//             /* Only put UniqueID if online */
//             if (SymlinkInformation->Online)
//             {
//                 MountPoints->MountPoints[TotalSymLinks].UniqueIdOffset = UniqueIdOffset;
//                 MountPoints->MountPoints[TotalSymLinks].UniqueIdLength = UniqueIdLength;
//             }
//             else
//             {
//                 MountPoints->MountPoints[TotalSymLinks].UniqueIdOffset = 0;
//                 MountPoints->MountPoints[TotalSymLinks].UniqueIdLength = 0;
//             }
//
//             MountPoints->MountPoints[TotalSymLinks].DeviceNameOffset = DeviceNameOffset;
//             MountPoints->MountPoints[TotalSymLinks].DeviceNameLength = DeviceNameLength;
//
//             /* And now, copy specific symlink info */
//             MountPoints->MountPoints[TotalSymLinks].SymbolicLinkNameOffset = TotalSize;
//             MountPoints->MountPoints[TotalSymLinks].SymbolicLinkNameLength = SymlinkInformation->Name.Length;
//
//             RtlCopyMemory((PWSTR)((ULONG_PTR)MountPoints + MountPoints->MountPoints[TotalSymLinks].SymbolicLinkNameOffset),
//                           SymlinkInformation->Name.Buffer, SymlinkInformation->Name.Length);
//
//             /* Update counters */
//             TotalSymLinks++;
//             TotalSize += SymlinkInformation->Name.Length;
//         }
//
//         if (UniqueId || SymbolicName)
//         {
//             break;
//         }
//     }
//
//     if (SymbolicName)
//     {
//         FreePool(DeviceName.Buffer);
//     }
//
//     return STATUS_SUCCESS;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// QueryPointsFromSymbolicLinkName(IN PDEVICE_EXTENSION DeviceExtension,
//                                 IN PUNICODE_STRING SymbolicName,
//                                 IN PIRP Irp)
// {
//     NTSTATUS Status;
//     ULONG TotalLength;
//     PIO_STACK_LOCATION Stack;
//     UNICODE_STRING DeviceName;
//     PMOUNTMGR_MOUNT_POINTS MountPoints;
//     PDEVICE_INFORMATION DeviceInformation = NULL;
//     PLIST_ENTRY DeviceEntry, SymlinksEntry;
//     PSYMLINK_INFORMATION SymlinkInformation;
//
//     /* Find device */
//     Status = QueryDeviceInformation(SymbolicName, &DeviceName,
//                                     NULL, NULL, NULL,
//                                     NULL, NULL, NULL);
//     if (NT_SUCCESS(Status))
//     {
//         /* Look for the device information */
//         for (DeviceEntry = DeviceExtension->DeviceListHead.Flink;
//              DeviceEntry != &(DeviceExtension->DeviceListHead);
//              DeviceEntry = DeviceEntry->Flink)
//         {
//             DeviceInformation = CONTAINING_RECORD(DeviceEntry, DEVICE_INFORMATION, DeviceListEntry);
//
//             if (RtlEqualUnicodeString(&DeviceName, &(DeviceInformation->DeviceName), TRUE))
//             {
//                 break;
//             }
//         }
//
//         FreePool(DeviceName.Buffer);
//
//         if (DeviceEntry == &(DeviceExtension->DeviceListHead))
//         {
//             return STATUS_INVALID_PARAMETER;
//         }
//
//         /* Check for the link */
//         for (SymlinksEntry = DeviceInformation->SymbolicLinksListHead.Flink;
//              SymlinksEntry != &(DeviceInformation->SymbolicLinksListHead);
//              SymlinksEntry = SymlinksEntry->Flink)
//         {
//             SymlinkInformation = CONTAINING_RECORD(SymlinksEntry, SYMLINK_INFORMATION, SymbolicLinksListEntry);
//
//             if (RtlEqualUnicodeString(SymbolicName, &SymlinkInformation->Name, TRUE))
//             {
//                 break;
//             }
//         }
//
//         if (SymlinksEntry == &(DeviceInformation->SymbolicLinksListHead))
//         {
//             return STATUS_INVALID_PARAMETER;
//         }
//     }
//     else
//     {
//         /* Browse all the devices to try to find the one
//          * that has the given link...
//          */
//         for (DeviceEntry = DeviceExtension->DeviceListHead.Flink;
//              DeviceEntry != &(DeviceExtension->DeviceListHead);
//              DeviceEntry = DeviceEntry->Flink)
//         {
//             DeviceInformation = CONTAINING_RECORD(DeviceEntry, DEVICE_INFORMATION, DeviceListEntry);
//
//             for (SymlinksEntry = DeviceInformation->SymbolicLinksListHead.Flink;
//                  SymlinksEntry != &(DeviceInformation->SymbolicLinksListHead);
//                  SymlinksEntry = SymlinksEntry->Flink)
//             {
//                 SymlinkInformation = CONTAINING_RECORD(SymlinksEntry, SYMLINK_INFORMATION, SymbolicLinksListEntry);
//
//                 if (RtlEqualUnicodeString(SymbolicName, &SymlinkInformation->Name, TRUE))
//                 {
//                     break;
//                 }
//             }
//
//             if (SymlinksEntry != &(DeviceInformation->SymbolicLinksListHead))
//             {
//                 break;
//             }
//         }
//
//         /* Even that way we didn't find, give up! */
//         if (DeviceEntry == &(DeviceExtension->DeviceListHead))
//         {
//             return STATUS_OBJECT_NAME_NOT_FOUND;
//         }
//     }
//
//     /* Get output buffer */
//     Stack = IoGetCurrentIrpStackLocation(Irp);
//     MountPoints = (PMOUNTMGR_MOUNT_POINTS)Irp->AssociatedIrp.SystemBuffer;
//
//     /* Compute output length */
//     TotalLength = DeviceInformation->UniqueId->UniqueIdLength +
//                   SymlinkInformation->Name.Length + DeviceInformation->DeviceName.Length;
//
//     /* Give length to allow reallocation */
//     MountPoints->Size = sizeof(MOUNTMGR_MOUNT_POINTS) + TotalLength;
//     MountPoints->NumberOfMountPoints = 1;
//     Irp->IoStatus.Information = sizeof(MOUNTMGR_MOUNT_POINTS) + TotalLength;
//
//     if (MountPoints->Size > Stack->Parameters.DeviceIoControl.OutputBufferLength)
//     {
//         Irp->IoStatus.Information = sizeof(MOUNTMGR_MOUNT_POINTS);
//
//         return STATUS_BUFFER_OVERFLOW;
//     }
//
//     /* Write out data */
//     MountPoints->MountPoints[0].SymbolicLinkNameOffset = sizeof(MOUNTMGR_MOUNT_POINTS);
//     MountPoints->MountPoints[0].SymbolicLinkNameLength = SymlinkInformation->Name.Length;
//     /* If link is online write it's unique ID, otherwise, forget about it */
//     if (SymlinkInformation->Online)
//     {
//         MountPoints->MountPoints[0].UniqueIdOffset = sizeof(MOUNTMGR_MOUNT_POINTS) +
//                                                      SymlinkInformation->Name.Length;
//         MountPoints->MountPoints[0].UniqueIdLength = DeviceInformation->UniqueId->UniqueIdLength;
//     }
//     else
//     {
//         MountPoints->MountPoints[0].UniqueIdOffset = 0;
//         MountPoints->MountPoints[0].UniqueIdLength = 0;
//     }
//
//     MountPoints->MountPoints[0].DeviceNameOffset = sizeof(MOUNTMGR_MOUNT_POINTS) +
//                                                    SymlinkInformation->Name.Length +
//                                                    DeviceInformation->UniqueId->UniqueIdLength;
//     MountPoints->MountPoints[0].DeviceNameLength = DeviceInformation->DeviceName.Length;
//
//     RtlCopyMemory((PWSTR)((ULONG_PTR)MountPoints + MountPoints->MountPoints[0].SymbolicLinkNameOffset),
//                   SymlinkInformation->Name.Buffer, SymlinkInformation->Name.Length);
//
//     if (SymlinkInformation->Online)
//     {
//         RtlCopyMemory((PWSTR)((ULONG_PTR)MountPoints + MountPoints->MountPoints[0].UniqueIdOffset),
//                       DeviceInformation->UniqueId->UniqueId, DeviceInformation->UniqueId->UniqueIdLength);
//     }
//
//     RtlCopyMemory((PWSTR)((ULONG_PTR)MountPoints + MountPoints->MountPoints[0].DeviceNameOffset),
//                   DeviceInformation->DeviceName.Buffer, DeviceInformation->DeviceName.Length);
//
//     return STATUS_SUCCESS;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
