// ---- crate-internal glue (private structs, GUID externs, no-op DDK shims) ----
#[allow(unused_imports)] use crate::types::*;
#[allow(unused_imports)] use crate::guids::*;
#[allow(unused_imports)] use crate::ddk_shims::*;
#[allow(unused_imports)] use crate::glue::*;
#[allow(unused_imports)] use crate::inlines::*;
#[allow(unused_imports)] use ntkrnl_sys::ke::KeGetCurrentIrql; // FORCEINLINE on amd64/arm64

use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{DEVICE_RELATIONS, IoCreateDevice, IoCreateSymbolicLink, IoDeleteDevice, IoDeleteSymbolicLink, IoGetDeviceObjectPointer, IoInvalidateDeviceRelations, IoRegisterDeviceInterface, IoSetDeviceInterfaceState, IoSetPartitionInformation, IoSetPartitionInformationEx, LOGICAL, PARTITION_INFORMATION, PARTITION_INFORMATION_EX, PARTITION_INFORMATION_GPT, PARTITION_INFORMATION_MBR, PDEVICE_CAPABILITIES, PDEVICE_OBJECT, PDEVICE_RELATIONS, PFILE_OBJECT, PGET_LENGTH_INFORMATION, PIO_STACK_LOCATION, PIRP, PMOUNTDEV_NAME, PMOUNTMGR_MOUNT_POINT, PMOUNTMGR_MOUNT_POINTS, PPARTITION_INFORMATION, PPARTITION_INFORMATION_EX, PSET_PARTITION_INFORMATION, PSET_PARTITION_INFORMATION_EX, PSTORAGE_DEVICE_NUMBER, PUNICODE_STRING, PVERIFY_INFORMATION, RtlCreateUnicodeString, RtlFreeUnicodeString, RtlInitUnicodeString, UINT32};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use ntkrnl_sys::ddk::{BUS_QUERY_ID_TYPE, IRP, MOUNTMGR_MOUNT_POINT, MOUNTMGR_MOUNT_POINTS, PARTITION_STYLE, PCSTR, PWCH, RtlAssert, _DEVICE_OBJECT, _DPFLTR_TYPE_DPFLTR_DISK_ID as DPFLTR_DISK_ID, _GET_LENGTH_INFORMATION, _LARGE_INTEGER, _PARTITION_INFORMATION, _PARTITION_INFORMATION_EX, _PARTITION_INFORMATION_EX__bindgen_ty_1, _PARTITION_INFORMATION_GPT, _PARTITION_INFORMATION_MBR, _SET_PARTITION_INFORMATION, _SET_PARTITION_INFORMATION_EX, _STORAGE_DEVICE_NUMBER, _UNICODE_STRING, _VERIFY_INFORMATION, wchar_t};

use crate::glue::{IOCTL_DISK_GET_LENGTH_INFO, IOCTL_DISK_GET_PARTITION_INFO, IOCTL_DISK_GET_PARTITION_INFO_EX, IOCTL_DISK_SET_PARTITION_INFO, IOCTL_DISK_SET_PARTITION_INFO_EX, IOCTL_DISK_UPDATE_PROPERTIES, IOCTL_DISK_VERIFY, IOCTL_MOUNTDEV_QUERY_DEVICE_NAME, IOCTL_MOUNTDEV_QUERY_UNIQUE_ID, IOCTL_STORAGE_GET_DEVICE_NUMBER, IOCTL_STORAGE_MEDIA_REMOVAL, IOCTL_VOLUME_GET_GPT_ATTRIBUTES, IOCTL_VOLUME_GET_VOLUME_DISK_EXTENTS, IOCTL_VOLUME_IS_PARTITION, IOCTL_VOLUME_ONLINE, IOCTL_VOLUME_QUERY_VOLUME_NUMBER};
// w! — NUL-terminated UTF-16 array from an ASCII literal (2-aligned 'static;
// a b"...\0" byte string cast to *const u16 is both ANSI-not-wide and
// potentially odd-aligned — see field manual / basesrv init.rs CSDVersion).
macro_rules! w {
    ($s:literal) => {{
        const __W: &[u16] = &{
            const BYTES: &[u8] = $s.as_bytes();
            let mut out = [0u16; { BYTES.len() + 1 }];
            let mut i = 0;
            while i < BYTES.len() {
                out[i] = BYTES[i] as u16;
                i += 1;
            }
            out
        };
        __W
    }};
}


// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const MOUNTMGR_DEVICE_NAME: [::core::ffi::c_ushort; 26] = unsafe {
    ::core::mem::transmute::<[u8; 52], [::core::ffi::c_ushort; 26]>(
        *b"\\\0D\0e\0v\0i\0c\0e\0\\\0M\0o\0u\0n\0t\0P\0o\0i\0n\0t\0M\0a\0n\0a\0g\0e\0r\0\0\0",
    )
};

static mut PartitionSymLinkFormat: [WCHAR; 33] = unsafe {
    ::core::mem::transmute::<
        [u8; 66],
        [WCHAR; 33],
    >(
        *b"\\\0D\0e\0v\0i\0c\0e\0\\\0H\0a\0r\0d\0d\0i\0s\0k\0%\0l\0u\0\\\0P\0a\0r\0t\0i\0t\0i\0o\0n\0%\0l\0u\0\0\0",
    )
};
#[no_mangle]
pub unsafe extern "C" fn PartitionCreateDevice(
    FDObject: PDEVICE_OBJECT,
    PartitionEntry: PPARTITION_INFORMATION_EX,
    PdoNumber: u32,
    PartitionStyle: PARTITION_STYLE,
    PDO: *mut PDEVICE_OBJECT,
) -> i32 {
    // PAGED_CODE() expansion
    const APC_LEVEL: u8 = 1;
    if (KeGetCurrentIrql() as u32 > APC_LEVEL as u32) {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
            KeGetCurrentIrql() as i32,
        );
        RtlAssert(
            b"0\0".as_ptr() as PVOID,
            b"drivers/storage/partmgr/partition.c\0".as_ptr() as PVOID,
            22 as ULONG,
            core::ptr::null_mut::<i8>(),
        );
    }

    const FILE_DEVICE_DISK: u32 = 7;
    // Corrected from the LLM's fabricated values (review: DISK_ID=9, ERROR=0, INFO=3) —
    // diagnostic-only (DbgPrintEx component-ID + level).
    const DPFLTR_DISK_ID: u32 = 9;
    const DPFLTR_ERROR_LEVEL: u32 = 0;
    const DPFLTR_INFO_LEVEL: u32 = 3;

    static mut HarddiskVolumeNextId: u32 = 1;
    let volumeNum: u32;
    unsafe {
        volumeNum = HarddiskVolumeNextId;
        HarddiskVolumeNextId = HarddiskVolumeNextId.wrapping_add(1);
    }

    let mut nameBuf: [u16; 64] = [0; 64];
    let mut deviceName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut::<u16>(),
    };

    swprintf(
        nameBuf.as_mut_ptr(),
        w!("\\Device\\HarddiskVolume%lu").as_ptr(),
        volumeNum,
    );
    RtlCreateUnicodeString(&mut deviceName, nameBuf.as_ptr());

    let mut partitionDevice: PDEVICE_OBJECT = core::ptr::null_mut::<_DEVICE_OBJECT>();
    let mut status: i32 = IoCreateDevice(
        (*FDObject).DriverObject,
        core::mem::size_of::<PARTITION_EXTENSION>() as u32,
        &mut deviceName,
        FILE_DEVICE_DISK,
        FILE_DEVICE_SECURE_OPEN,
        0u8,
        &mut partitionDevice,
    );

    if !(status >= 0) {
        DbgPrintEx(
            DPFLTR_DISK_ID,
            DPFLTR_ERROR_LEVEL,
            b"(%s:%d) Unable to create device object %wZ\n\0".as_ptr() as *const i8,
            b"drivers/storage/partmgr/partition.c\0".as_ptr() as *const i8,
            55,
            &mut deviceName,
        );
        return status;
    }

    DbgPrintEx(
        DPFLTR_DISK_ID,
        DPFLTR_INFO_LEVEL,
        b"(%s:%d) Created device object %p %wZ\n\0".as_ptr() as *const i8,
        b"drivers/storage/partmgr/partition.c\0".as_ptr() as *const i8,
        59,
        partitionDevice,
        &mut deviceName,
    );

    let partExt: PPARTITION_EXTENSION = (*partitionDevice).DeviceExtension as PPARTITION_EXTENSION;
    RtlZeroMemory(
        partExt as *mut core::ffi::c_void,
        core::mem::size_of::<PARTITION_EXTENSION>(),
    );

    (*partExt).DeviceObject = partitionDevice;
    (*partExt).LowerDevice = FDObject;

    // Adjust stack size and flags
    (*partitionDevice).StackSize = (*FDObject).StackSize;
    (*partitionDevice).Flags |= DO_DIRECT_IO as u32;

    // Copy partition information based on style
    if PartitionStyle == PARTITION_STYLE_MBR {
        (*partExt).__bindgen_anon_1.Mbr.as_mut().PartitionType =
            (*PartitionEntry).__bindgen_anon_1.Mbr.PartitionType;
        (*partExt).__bindgen_anon_1.Mbr.as_mut().BootIndicator =
            (*PartitionEntry).__bindgen_anon_1.Mbr.BootIndicator;
        (*partExt).__bindgen_anon_1.Mbr.as_mut().HiddenSectors =
            (*PartitionEntry).__bindgen_anon_1.Mbr.HiddenSectors as u32;
    } else {
        (*partExt).__bindgen_anon_1.Gpt.as_mut().PartitionType =
            (*PartitionEntry).__bindgen_anon_1.Gpt.PartitionType;
        (*partExt).__bindgen_anon_1.Gpt.as_mut().PartitionId =
            (*PartitionEntry).__bindgen_anon_1.Gpt.PartitionId;
        (*partExt).__bindgen_anon_1.Gpt.as_mut().Attributes =
            (*PartitionEntry).__bindgen_anon_1.Gpt.Attributes as u64;
        RtlCopyMemory(
            (*partExt).__bindgen_anon_1.Gpt.as_mut().Name.as_mut_ptr() as *mut core::ffi::c_void,
            (*PartitionEntry).__bindgen_anon_1.Gpt.Name.as_ptr() as *const core::ffi::c_void,
            core::mem::size_of::<[u16; 36]>(),
        );
    }

    (*partExt).DeviceName = deviceName;
    (*partExt).StartingOffset = (*PartitionEntry).StartingOffset.QuadPart as u64;
    (*partExt).PartitionLength = (*PartitionEntry).PartitionLength.QuadPart as u64;
    (*partExt).OnDiskNumber = (*PartitionEntry).PartitionNumber as u32;
    (*partExt).DetectedNumber = PdoNumber;
    (*partExt).VolumeNumber = volumeNum;

    // Clear DO_DEVICE_INITIALIZING
    (*partitionDevice).Flags &= !(DO_DEVICE_INITIALIZING as u32);

    *PDO = partitionDevice;
    status
}
#[no_mangle]
pub unsafe extern "C" fn PartitionHandleStartDevice(
    PartExt: *mut PARTITION_EXTENSION,
    Irp: *mut IRP,
) -> NTSTATUS {
    // PAGED_CODE() – check IRQL
    if KeGetCurrentIrql() as i32 > (APC_LEVEL) as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            RtlAssert(
                b"0\0" as *const u8 as *const i8 as PVOID,
                b"drivers/storage/partmgr/partition.c\0"
                    as *const u8 as *const i8 as PVOID,
                110 as u32,
                core::ptr::null_mut::<i8>(),
            );
        }
    }

    let mut nameBuf: [u16; 64] = [0; 64];
    let mut partitionSymlink: _UNICODE_STRING = core::mem::zeroed();
    let mut interfaceName: _UNICODE_STRING = core::mem::zeroed();

    let fdoExtension: *mut FDO_EXTENSION = (!(*PartExt).LowerDevice.is_null()).then(|| {
        (*(*PartExt).LowerDevice).DeviceExtension as *mut FDO_EXTENSION
    }).unwrap_or(core::ptr::null_mut());

    swprintf(
        nameBuf.as_mut_ptr(),
        (&raw const PartitionSymLinkFormat).cast::<WCHAR>(),
        (*fdoExtension).DiskData.DeviceNumber,
        (*PartExt).DetectedNumber,
    );

    if RtlCreateUnicodeString(
        &raw mut partitionSymlink,
        nameBuf.as_ptr(),
    ) == 0
    {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    let mut status: NTSTATUS = IoCreateSymbolicLink(
        &raw mut partitionSymlink,
        &raw mut (*PartExt).DeviceName,
    );

    if !(status >= 0) {
        return status;
    }

    (*PartExt).SymlinkCreated = 1u8;

    DbgPrintEx(
        DPFLTR_DISK_ID as u32,
        DPFLTR_INFO_LEVEL as u32,
        b"(%s:%d) Symlink created %wZ -> %wZ\n\0" as *const u8 as *const i8,
        b"drivers/storage/partmgr/partition.c\0"
            as *const u8 as *const i8,
        135 as i32,
        &raw mut partitionSymlink,
        &raw mut (*PartExt).DeviceName,
    );

    status = IoRegisterDeviceInterface(
        (*PartExt).DeviceObject,
        &raw const GUID_DEVINTERFACE_PARTITION,
        core::ptr::null_mut::<_UNICODE_STRING>(),
        &raw mut interfaceName,
    );

    if !(status >= 0) {
        return status;
    }

    DbgPrintEx(
        DPFLTR_DISK_ID as u32,
        DPFLTR_INFO_LEVEL as u32,
        b"(%s:%d) Partition interface %wZ\n\0" as *const u8 as *const i8,
        b"drivers/storage/partmgr/partition.c\0"
            as *const u8 as *const i8,
        151 as i32,
        &raw mut interfaceName,
    );

    (*PartExt).PartitionInterfaceName = interfaceName;

    status = IoSetDeviceInterfaceState(&raw mut interfaceName, 1u8);
    if !(status >= 0) {
        RtlFreeUnicodeString(&raw mut interfaceName);
        RtlInitUnicodeString(
            &raw mut (*PartExt).PartitionInterfaceName,
            core::ptr::null(),
        );
        return status;
    }

    status = IoRegisterDeviceInterface(
        (*PartExt).DeviceObject,
        &raw const GUID_DEVINTERFACE_VOLUME,
        core::ptr::null_mut::<_UNICODE_STRING>(),
        &raw mut interfaceName,
    );

    if !(status >= 0) {
        return status;
    }

    DbgPrintEx(
        DPFLTR_DISK_ID as u32,
        DPFLTR_INFO_LEVEL as u32,
        b"(%s:%d) Volume interface %wZ\n\0" as *const u8 as *const i8,
        b"drivers/storage/partmgr/partition.c\0"
            as *const u8 as *const i8,
        170 as i32,
        &raw mut interfaceName,
    );

    (*PartExt).VolumeInterfaceName = interfaceName;

    status = IoSetDeviceInterfaceState(&raw mut interfaceName, 1u8);
    if !(status >= 0) {
        RtlFreeUnicodeString(&raw mut interfaceName);
        RtlInitUnicodeString(
            &raw mut (*PartExt).VolumeInterfaceName,
            core::ptr::null(),
        );
        return status;
    }

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn VolumeDeleteMountPoints(PartExt: *mut PARTITION_EXTENSION) -> NTSTATUS {
    let mut Status: NTSTATUS = 0;
    let mut MountMgr: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut DeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut FileObject: PFILE_OBJECT = core::ptr::null_mut();
    let mut InputBuffer: PMOUNTMGR_MOUNT_POINT = core::ptr::null_mut();
    let mut OutputBuffer: PMOUNTMGR_MOUNT_POINTS = core::ptr::null_mut();

    // PAGED_CODE assertion (c2rust expansion)
    if KeGetCurrentIrql() as u32 > (APC_LEVEL) as u32 {
        // Should never be called at high IRQL
    }

    RtlInitUnicodeString(&raw mut MountMgr, MOUNTMGR_DEVICE_NAME.as_ptr() as PCWSTR);
    Status = IoGetDeviceObjectPointer(
        &raw mut MountMgr,
        FILE_READ_ATTRIBUTES,
        &raw mut FileObject,
        &raw mut DeviceObject,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    let DeviceName = &(*PartExt).DeviceName;
    let InputSize = core::mem::size_of::<MOUNTMGR_MOUNT_POINT>() as ULONG + (DeviceName.Length) as u32;
    InputBuffer = ExAllocatePoolWithTag(
        PagedPool,
        InputSize as SIZE_T,
        TAG_PARTMGR,
    ) as PMOUNTMGR_MOUNT_POINT;

    if InputBuffer.is_null() {
        Status = STATUS_INSUFFICIENT_RESOURCES;
    } else {
        // Zero first field
        core::ptr::write_bytes(
            InputBuffer as *mut u8,
            0,
            core::mem::size_of::<MOUNTMGR_MOUNT_POINT>(),
        );
        (*InputBuffer).DeviceNameOffset = core::mem::size_of::<MOUNTMGR_MOUNT_POINT>() as ULONG;
        (*InputBuffer).DeviceNameLength = DeviceName.Length;
        // Copy the device name into InputBuffer just after the header
        let dst = (InputBuffer as *mut u8).add(core::mem::size_of::<MOUNTMGR_MOUNT_POINT>());
        core::ptr::copy_nonoverlapping(
            DeviceName.Buffer as *const u8,
            dst,
            DeviceName.Length as usize,
        );

        // Determine starting output buffer size
        let mut OutputSize = core::cmp::max(
            PAGE_SIZE as usize,
            core::mem::size_of::<MOUNTMGR_MOUNT_POINTS>(),
        ) as ULONG;

        for _ in 0..2 {
            OutputBuffer = ExAllocatePoolWithTag(
                PagedPool,
                OutputSize as SIZE_T,
                TAG_PARTMGR,
            ) as PMOUNTMGR_MOUNT_POINTS;
            if OutputBuffer.is_null() {
                Status = STATUS_INSUFFICIENT_RESOURCES;
                break;
            }

            Status = IssueSyncIoControlRequest(
                IOCTL_MOUNTMGR_DELETE_POINTS,
                DeviceObject,
                InputBuffer as PVOID,
                InputSize,
                OutputBuffer as PVOID,
                OutputSize,
                FALSE,
            );

            if Status != STATUS_BUFFER_OVERFLOW {
                break;
            }

            OutputSize = (*OutputBuffer).Size;
            ExFreePoolWithTag(OutputBuffer as PVOID, TAG_PARTMGR);
            // Diverges from the C, which leaves the freed pointer here: a SECOND
            // STATUS_BUFFER_OVERFLOW ends the loop and the cleanup below freed it again.
            OutputBuffer = core::ptr::null_mut();
            // Continue loop to reallocate
        }
    }

    // Cleanup
    if !OutputBuffer.is_null() {
        ExFreePoolWithTag(OutputBuffer as PVOID, TAG_PARTMGR);
    }
    if !InputBuffer.is_null() {
        ExFreePoolWithTag(InputBuffer as PVOID, TAG_PARTMGR);
    }
    if !FileObject.is_null() {
        ObDereferenceObject(FileObject as PVOID);
    }

    Status
}
#[no_mangle]
pub unsafe extern "C" fn PartitionHandleRemove(
    PartExt: PPARTITION_EXTENSION,
    FinalRemove: BOOLEAN,
) -> NTSTATUS {
    let mut status: NTSTATUS;

    // PAGED_CODE
    if KeGetCurrentIrql() as i32 > (APC_LEVEL) as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as PCSTR,
            KeGetCurrentIrql() as i32,
        );
        RtlAssert(
            b"0\0".as_ptr() as *const i8 as PVOID,
            b"partition.c\0".as_ptr() as *const i8 as PVOID,
            290u32,
            core::ptr::null_mut::<i8>(),
        );
    }

    if (*PartExt).SymlinkCreated != 0u8 {
        let mut nameBuf: [u16; 64] = [0; 64];
        let fdoExtension = (*(*PartExt).LowerDevice).DeviceExtension as PFDO_EXTENSION;

        swprintf(
            nameBuf.as_mut_ptr(),
            (&raw const PartitionSymLinkFormat).cast::<WCHAR>(),
            (*fdoExtension).DiskData.DeviceNumber,
            (*PartExt).DetectedNumber,
        );

        let mut partitionSymlink: _UNICODE_STRING = _UNICODE_STRING {
            Length: 0,
            MaximumLength: 0,
            Buffer: core::ptr::null_mut(),
        };
        RtlInitUnicodeString(
            core::ptr::addr_of_mut!(partitionSymlink),
            nameBuf.as_ptr() as PCWSTR,
        );

        status = IoDeleteSymbolicLink(core::ptr::addr_of_mut!(partitionSymlink));
        if !NT_SUCCESS(status) {
            return status;
        }
        (*PartExt).SymlinkCreated = 0u8;

        DbgPrintEx(
            DPFLTR_DISK_ID as i32 as ULONG,
            DPFLTR_INFO_LEVEL as ULONG,
            b"(%s:%d) Symlink removed %wZ -> %wZ\n\0".as_ptr() as PCSTR,
            b"partition.c\0".as_ptr() as *const i8,
            312i32,
            core::ptr::addr_of_mut!(partitionSymlink) as *mut _,
            core::ptr::addr_of_mut!((*PartExt).DeviceName) as *mut _,
        );
    }

    if !(*PartExt).PartitionInterfaceName.Buffer.is_null() {
        status = IoSetDeviceInterfaceState(
            core::ptr::addr_of_mut!((*PartExt).PartitionInterfaceName),
            0u8,
        );
        if !NT_SUCCESS(status) {
            return status;
        }
        RtlFreeUnicodeString(core::ptr::addr_of_mut!((*PartExt).PartitionInterfaceName));
        RtlInitUnicodeString(
            core::ptr::addr_of_mut!((*PartExt).PartitionInterfaceName),
            core::ptr::null(),
        );
    }

    if !(*PartExt).VolumeInterfaceName.Buffer.is_null() {
        status = VolumeDeleteMountPoints(PartExt);
        if !NT_SUCCESS(status) {
            DbgPrintEx(
                DPFLTR_DISK_ID as i32 as ULONG,
                DPFLTR_ERROR_LEVEL as ULONG,
                b"(%s:%d) VolumeDeleteMountPoints(%wZ) failed with status 0x%08lx\n\0".as_ptr() as PCSTR,
                b"partition.c\0".as_ptr() as *const i8,
                337i32,
                core::ptr::addr_of_mut!((*PartExt).DeviceName) as *mut _,
                status,
            );
        }
        status = IoSetDeviceInterfaceState(
            core::ptr::addr_of_mut!((*PartExt).VolumeInterfaceName),
            0u8,
        );
        if !NT_SUCCESS(status) {
            return status;
        }
        RtlFreeUnicodeString(core::ptr::addr_of_mut!((*PartExt).VolumeInterfaceName));
        RtlInitUnicodeString(
            core::ptr::addr_of_mut!((*PartExt).VolumeInterfaceName),
            core::ptr::null(),
        );
    }

    if FinalRemove != 0u8 {
        if (*PartExt).DeviceName.Buffer.is_null() {
            RtlAssert(
                b"PartExt->DeviceName.Buffer\0".as_ptr() as *const i8 as PVOID,
                b"partition.c\0".as_ptr() as *const i8 as PVOID,
                353u32,
                core::ptr::null_mut::<i8>(),
            );
        }
        if !(*PartExt).DeviceName.Buffer.is_null() {
            DbgPrintEx(
                DPFLTR_DISK_ID as i32 as ULONG,
                DPFLTR_INFO_LEVEL as ULONG,
                b"(%s:%d) Removed device %wZ\n\0".as_ptr() as PCSTR,
                b"partition.c\0".as_ptr() as *const i8,
                356i32,
                core::ptr::addr_of_mut!((*PartExt).DeviceName) as *mut _,
            );
            RtlFreeUnicodeString(core::ptr::addr_of_mut!((*PartExt).DeviceName));
        }
        IoDeleteDevice((*PartExt).DeviceObject);
    }

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn PartitionHandleDeviceRelations(
    PartExt: PPARTITION_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    let io_stack = IoGetCurrentIrpStackLocation(Irp);
    let relation_type = (*io_stack).Parameters.QueryDeviceRelations.Type;

    if relation_type == TargetDeviceRelation {
        let device_relations = ExAllocatePoolZero(
            PagedPool,
            (core::mem::size_of::<DEVICE_RELATIONS>() as _),
            TAG_PARTMGR,
        ) as PDEVICE_RELATIONS;

        if !device_relations.is_null() {
            (*device_relations).Count = 1;
            (*device_relations).Objects[0] = (*PartExt).DeviceObject;
            ObReferenceObject((*device_relations).Objects[0] as PVOID);
            (*Irp).IoStatus.Information = (device_relations as _);
            return STATUS_SUCCESS;
        } else {
            return STATUS_INSUFFICIENT_RESOURCES;
        }
    } else {
        (*Irp).IoStatus.Information = 0;
        return (*Irp).IoStatus.__bindgen_anon_1.Status;
    }
}
#[no_mangle]
pub unsafe extern "C" fn PartitionHandleQueryId(
    PartExt: PPARTITION_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    let ioStack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let idType: BUS_QUERY_ID_TYPE = (*ioStack).Parameters.QueryId.IdType;
    let mut idString: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut status: NTSTATUS;

    // PAGED_CODE() expansion
    {
        if KeGetCurrentIrql() as i32 > APC_LEVEL as i32 {
            DbgPrint(
                b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as PCSTR,
                KeGetCurrentIrql() as i32,
            );
            if 0i32 == 0 {
                RtlAssert(
                    b"0\0" as *const u8 as *const i8 as PVOID,
                    b"drivers/storage/partmgr/partition.c\0"
                        as *const u8 as *const i8 as PVOID,
                    417u32,
                    core::ptr::null_mut::<i8>(),
                );
            }
        }
    }

    match idType as u32 {
        0 => {
            status = if RtlCreateUnicodeString(
                core::ptr::addr_of_mut!(idString),
                unsafe {
                    core::mem::transmute::<[u8; 36], [u16; 18]>(
                        *b"S\0T\0O\0R\0A\0G\0E\0\\\0P\0a\0r\0t\0i\0t\0i\0o\0n\0\0\0",
                    )
                }.as_ptr() as PCWSTR,
            ) != 0
            {
                STATUS_SUCCESS
            } else {
                STATUS_INSUFFICIENT_RESOURCES
            };
        }
        1 => {
            static mut volumeID: [u16; 16] = unsafe {
                core::mem::transmute::<[u8; 32], [u16; 16]>(
                    *b"S\0T\0O\0R\0A\0G\0E\0\\\0V\0o\0l\0u\0m\0e\0\0\0\0\0",
                )
            };
            idString.Buffer = ExAllocatePoolWithTag(
                PagedPool,
                core::mem::size_of::<[u16; 16]>() as SIZE_T,
                TAG_PARTMGR,
            ) as PWCH;
            core::ptr::copy_nonoverlapping(
                (&raw const volumeID).cast::<u16>() as *const u8,
                idString.Buffer as *mut u8,
                core::mem::size_of::<[u16; 16]>(),
            );
            status = STATUS_SUCCESS;
        }
        3 => {
            let mut string: [u16; 64] = [0u16; 64];
            let fdoExtension: PFDO_EXTENSION =
                (*(*PartExt).LowerDevice).DeviceExtension as PFDO_EXTENSION;

            PartMgrAcquireLayoutLock(fdoExtension);

            if (*fdoExtension).DiskData.PartitionStyle as u32 == PARTITION_STYLE_MBR as u32 {
                swprintf(
                    string.as_mut_ptr() as *mut wchar_t,
                    unsafe {
                        core::mem::transmute::<[u8; 42], [u16; 21]>(
                            *b"S\0%\x000\08\0l\0x\0_\0O\0%\0I\x006\x004\0x\0_\0L\0%\0I\x006\x004\0x\0\0\0",
                        )
                    }
                    .as_ptr() as *const wchar_t,
                    (*fdoExtension).DiskData.__bindgen_anon_1.Mbr.as_ref().Signature,
                    (*PartExt).StartingOffset,
                    (*PartExt).PartitionLength,
                );
            } else {
                swprintf(
                    string.as_mut_ptr() as *mut wchar_t,
                    unsafe {
                        core::mem::transmute::<[u8; 130], [u16; 65]>(
                            *b"S\0%\x000\08\0x\0-\0%\x000\x004\0x\0-\0%\x000\x004\0x\0-\0%\x000\x002\0x\0%\x000\x002\0x\0-\0%\x000\x002\0x\0%\x000\x002\0x\0%\x000\x002\0x\0%\x000\x002\0x\0%\x000\x002\0x\0%\x000\x002\0x\0S\0_\0O\0%\0I\x006\x004\0x\0_\0L\0%\0I\x006\x004\0x\0\0\0",
                        )
                    }
                    .as_ptr() as *const wchar_t,
                    (*fdoExtension).DiskData.__bindgen_anon_1.Gpt.as_ref().DiskId.Data1,
                    (*fdoExtension).DiskData.__bindgen_anon_1.Gpt.as_ref().DiskId.Data2 as i32,
                    (*fdoExtension).DiskData.__bindgen_anon_1.Gpt.as_ref().DiskId.Data3 as i32,
                    (*fdoExtension).DiskData.__bindgen_anon_1.Gpt.as_ref().DiskId.Data4[0usize] as i32,
                    (*fdoExtension).DiskData.__bindgen_anon_1.Gpt.as_ref().DiskId.Data4[1usize] as i32,
                    (*fdoExtension).DiskData.__bindgen_anon_1.Gpt.as_ref().DiskId.Data4[2usize] as i32,
                    (*fdoExtension).DiskData.__bindgen_anon_1.Gpt.as_ref().DiskId.Data4[3usize] as i32,
                    (*fdoExtension).DiskData.__bindgen_anon_1.Gpt.as_ref().DiskId.Data4[4usize] as i32,
                    (*fdoExtension).DiskData.__bindgen_anon_1.Gpt.as_ref().DiskId.Data4[5usize] as i32,
                    (*fdoExtension).DiskData.__bindgen_anon_1.Gpt.as_ref().DiskId.Data4[6usize] as i32,
                    (*fdoExtension).DiskData.__bindgen_anon_1.Gpt.as_ref().DiskId.Data4[7usize] as i32,
                    (*PartExt).StartingOffset,
                    (*PartExt).PartitionLength,
                );
            }
            PartMgrReleaseLayoutLock(fdoExtension);

            status = if RtlCreateUnicodeString(
                core::ptr::addr_of_mut!(idString),
                string.as_mut_ptr() as PCWSTR,
            ) != 0
            {
                STATUS_SUCCESS
            } else {
                STATUS_INSUFFICIENT_RESOURCES
            };
        }
        _ => {
            status = STATUS_NOT_SUPPORTED;
        }
    }

    (*Irp).IoStatus.Information = if NT_SUCCESS(status) {
        idString.Buffer as usize as _
    } else {
        0
    };

    status
}
#[no_mangle]
pub unsafe extern "C" fn PartitionHandleQueryCapabilities(
    PartExt: *mut _PARTITION_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    let ioStack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let devCaps: PDEVICE_CAPABILITIES = (*ioStack).Parameters.DeviceCapabilities.Capabilities;

    // PAGED_CODE()
    if KeGetCurrentIrql() > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            RtlAssert(
                b"0\0".as_ptr() as PVOID,
                b"drivers/storage/partmgr/partition.c\0".as_ptr() as PVOID,
                493 as ULONG,
                core::ptr::null_mut::<i8>(),
            );
        }
    }

    // ASSERT(devCaps)
    if devCaps.is_null() {
        RtlAssert(
            b"devCaps\0".as_ptr() as PVOID,
            b"drivers/storage/partmgr/partition.c\0".as_ptr() as PVOID,
            494 as ULONG,
            core::ptr::null_mut::<i8>(),
        );
    }

    (*devCaps).set_SilentInstall((1u8) as u32); // TRUE
    (*devCaps).set_RawDeviceOK((1u8) as u32);
    (*devCaps).set_NoDisplayInUI((1u8) as u32);
    (*devCaps).Address = (*PartExt).OnDiskNumber as ULONG;
    (*devCaps).set_UniqueID((0u8) as u32); // FALSE

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn PartitionHandlePnp(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let partExt = (*DeviceObject).DeviceExtension as *mut _PARTITION_EXTENSION;
    let ioStack = IoGetCurrentIrpStackLocation(Irp);
    let status: NTSTATUS;

    if KeGetCurrentIrql() > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as PCSTR,
            KeGetCurrentIrql() as i32,
        );
        RtlAssert(
            b"0\0".as_ptr() as PVOID,
            b"drivers/storage/partmgr/partition.c\0".as_ptr() as PVOID,
            515 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    }

    match (*ioStack).MinorFunction {
        IRP_MN_START_DEVICE => {
            status = PartitionHandleStartDevice(partExt, Irp);
        }
        IRP_MN_QUERY_DEVICE_RELATIONS => {
            status = PartitionHandleDeviceRelations(partExt, Irp);
        }
        IRP_MN_QUERY_STOP_DEVICE
        | IRP_MN_QUERY_REMOVE_DEVICE
        | IRP_MN_CANCEL_STOP_DEVICE
        | IRP_MN_CANCEL_REMOVE_DEVICE
        | IRP_MN_STOP_DEVICE => {
            status = STATUS_SUCCESS;
        }
        IRP_MN_SURPRISE_REMOVAL => {
            status = PartitionHandleRemove(partExt, 0u8);
        }
        IRP_MN_REMOVE_DEVICE => {
            status = PartitionHandleRemove(partExt, 1u8);
        }
        IRP_MN_QUERY_ID => {
            status = PartitionHandleQueryId(partExt, Irp);
        }
        IRP_MN_QUERY_CAPABILITIES => {
            status = PartitionHandleQueryCapabilities(partExt, Irp);
        }
        _ => {
            (*Irp).IoStatus.Information = (0 as _);
            status = STATUS_NOT_SUPPORTED;
        }
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = status;
    IoCompleteRequest(Irp, IO_NO_INCREMENT);
    status
}
#[no_mangle]
pub unsafe extern "C" fn PartitionHandleDeviceControl(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    let ioStack = IoGetCurrentIrpStackLocation(Irp);
    let partExt = (*DeviceObject).DeviceExtension as PPARTITION_EXTENSION;
    let fdoExtension = (*(*partExt).LowerDevice).DeviceExtension as PFDO_EXTENSION;
    let mut status: NTSTATUS = 0;

    // Assert that this is a partition, not the FDO (debug only)
    ntkrnl_sys::ASSERT!((*partExt).IsFDO == 0);

    if (*partExt).IsEnumerated == 0 {
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_DEVICE_DOES_NOT_EXIST;
        IoCompleteRequest(Irp, IO_NO_INCREMENT);
        return STATUS_DEVICE_DOES_NOT_EXIST;
    }

    match (*ioStack).Parameters.DeviceIoControl.IoControlCode {
        IOCTL_DISK_GET_PARTITION_INFO => {
            if VerifyIrpOutBufferSize(Irp, (core::mem::size_of::<_PARTITION_INFORMATION>() as SIZE_T)) == 0 {
                status = STATUS_BUFFER_TOO_SMALL;
            } else {
                PartMgrAcquireLayoutLock(fdoExtension);
                if (*fdoExtension).DiskData.PartitionStyle != PARTITION_STYLE_MBR {
                    status = STATUS_INVALID_DEVICE_REQUEST;
                    PartMgrReleaseLayoutLock(fdoExtension);
                } else {
                    let partInfo = (*Irp).AssociatedIrp.SystemBuffer as PPARTITION_INFORMATION;
                    *partInfo = _PARTITION_INFORMATION {
                        StartingOffset: _LARGE_INTEGER { QuadPart: (*partExt).StartingOffset as i64 },
                        PartitionLength: _LARGE_INTEGER { QuadPart: (*partExt).PartitionLength as i64 },
                        HiddenSectors: (*partExt).__bindgen_anon_1.Mbr.as_ref().HiddenSectors as u32,
                        PartitionNumber: (*partExt).DetectedNumber as u32,
                        PartitionType: (*partExt).__bindgen_anon_1.Mbr.as_ref().PartitionType,
                        BootIndicator: (*partExt).__bindgen_anon_1.Mbr.as_ref().BootIndicator,
                        RecognizedPartition: (*partExt).__bindgen_anon_1.Mbr.as_ref().RecognizedPartition,
                        RewritePartition: 0u8,
                    };
                    PartMgrReleaseLayoutLock(fdoExtension);
                    (*Irp).IoStatus.Information = core::mem::size_of::<_PARTITION_INFORMATION>() as _;
                    status = STATUS_SUCCESS;
                }
            }
        }
        IOCTL_DISK_GET_PARTITION_INFO_EX => {
            if VerifyIrpOutBufferSize(Irp, (core::mem::size_of::<_PARTITION_INFORMATION_EX>() as SIZE_T)) == 0 {
                status = STATUS_BUFFER_TOO_SMALL;
            } else {
                let partInfoEx = (*Irp).AssociatedIrp.SystemBuffer as PPARTITION_INFORMATION_EX;
                PartMgrAcquireLayoutLock(fdoExtension);
                *partInfoEx = _PARTITION_INFORMATION_EX {
                    PartitionStyle: (*fdoExtension).DiskData.PartitionStyle,
                    StartingOffset: _LARGE_INTEGER { QuadPart: (*partExt).StartingOffset as i64 },
                    PartitionLength: _LARGE_INTEGER { QuadPart: (*partExt).PartitionLength as i64 },
                    PartitionNumber: (*partExt).DetectedNumber as u32,
                    RewritePartition: 0u8,
                    __bindgen_anon_1: _PARTITION_INFORMATION_EX__bindgen_ty_1 {
                        Mbr: _PARTITION_INFORMATION_MBR {
                            PartitionType: 0,
                            BootIndicator: 0,
                            RecognizedPartition: 0,
                            HiddenSectors: 0,
                        },
                    },
                };
                if (*fdoExtension).DiskData.PartitionStyle == PARTITION_STYLE_MBR {
                    (*partInfoEx).__bindgen_anon_1.Mbr = _PARTITION_INFORMATION_MBR {
                        PartitionType: (*partExt).__bindgen_anon_1.Mbr.as_ref().PartitionType,
                        BootIndicator: (*partExt).__bindgen_anon_1.Mbr.as_ref().BootIndicator,
                        RecognizedPartition: (*partExt).__bindgen_anon_1.Mbr.as_ref().RecognizedPartition,
                        HiddenSectors: (*partExt).__bindgen_anon_1.Mbr.as_ref().HiddenSectors as u32,
                    };
                } else {
                    (*partInfoEx).__bindgen_anon_1.Gpt = _PARTITION_INFORMATION_GPT {
                        PartitionType: (*partExt).__bindgen_anon_1.Gpt.as_ref().PartitionType,
                        PartitionId: (*partExt).__bindgen_anon_1.Gpt.as_ref().PartitionId,
                        Attributes: (*partExt).__bindgen_anon_1.Gpt.as_ref().Attributes as u64,
                        Name: [0u16; 36],
                    };
                    core::ptr::copy_nonoverlapping(
                        (*partExt).__bindgen_anon_1.Gpt.as_ref().Name.as_ptr(),
                        (*partInfoEx).__bindgen_anon_1.Gpt.Name.as_mut_ptr(),
                        36,
                    );
                }
                PartMgrReleaseLayoutLock(fdoExtension);
                (*Irp).IoStatus.Information = core::mem::size_of::<_PARTITION_INFORMATION_EX>() as _;
                status = STATUS_SUCCESS;
            }
        }
        IOCTL_DISK_SET_PARTITION_INFO => {
            let inputBuffer = (*Irp).AssociatedIrp.SystemBuffer as PSET_PARTITION_INFORMATION;
            if VerifyIrpInBufferSize(Irp, (core::mem::size_of::<_SET_PARTITION_INFORMATION>() as SIZE_T)) == 0 {
                status = STATUS_INFO_LENGTH_MISMATCH;
            } else {
                PartMgrAcquireLayoutLock(fdoExtension);
                status = IoSetPartitionInformation(
                    (*fdoExtension).LowerDevice,
                    (*fdoExtension).DiskData.BytesPerSector as u32,
                    (*partExt).OnDiskNumber as u32,
                    (*inputBuffer).PartitionType as u32,
                );
                if NT_SUCCESS(status) {
                    (*partExt).__bindgen_anon_1.Mbr.as_mut().PartitionType = (*inputBuffer).PartitionType;
                }
                PartMgrReleaseLayoutLock(fdoExtension);
                (*Irp).IoStatus.Information = 0;
            }
        }
        IOCTL_DISK_SET_PARTITION_INFO_EX => {
            let inputBuffer = (*Irp).AssociatedIrp.SystemBuffer as PSET_PARTITION_INFORMATION_EX;
            if VerifyIrpInBufferSize(Irp, (core::mem::size_of::<_SET_PARTITION_INFORMATION_EX>() as SIZE_T)) == 0 {
                status = STATUS_INFO_LENGTH_MISMATCH;
            } else {
                PartMgrAcquireLayoutLock(fdoExtension);
                status = IoSetPartitionInformationEx(
                    (*fdoExtension).LowerDevice,
                    (*partExt).OnDiskNumber as u32,
                    inputBuffer as *mut _SET_PARTITION_INFORMATION_EX,
                );
                if NT_SUCCESS(status) {
                    if (*fdoExtension).DiskData.PartitionStyle == PARTITION_STYLE_MBR {
                        (*partExt).__bindgen_anon_1.Mbr.as_mut().PartitionType = (*inputBuffer).__bindgen_anon_1.Mbr.PartitionType;
                    } else {
                        (*partExt).__bindgen_anon_1.Gpt.as_mut().PartitionType = (*inputBuffer).__bindgen_anon_1.Gpt.PartitionType;
                        (*partExt).__bindgen_anon_1.Gpt.as_mut().PartitionId = (*inputBuffer).__bindgen_anon_1.Gpt.PartitionId;
                        (*partExt).__bindgen_anon_1.Gpt.as_mut().Attributes = (*inputBuffer).__bindgen_anon_1.Gpt.Attributes as u64;
                        core::ptr::copy_nonoverlapping(
                            (*inputBuffer).__bindgen_anon_1.Gpt.Name.as_ptr(),
                            (*partExt).__bindgen_anon_1.Gpt.as_mut().Name.as_mut_ptr(),
                            36,
                        );
                    }
                }
                PartMgrReleaseLayoutLock(fdoExtension);
                (*Irp).IoStatus.Information = 0;
            }
        }
        IOCTL_DISK_GET_LENGTH_INFO => {
            let lengthInfo = (*Irp).AssociatedIrp.SystemBuffer as PGET_LENGTH_INFORMATION;
            if VerifyIrpOutBufferSize(Irp, (core::mem::size_of::<_GET_LENGTH_INFORMATION>() as SIZE_T)) == 0 {
                status = STATUS_BUFFER_TOO_SMALL;
            } else {
                PartMgrAcquireLayoutLock(fdoExtension);
                (*lengthInfo).Length.QuadPart = (*partExt).PartitionLength as i64;
                PartMgrReleaseLayoutLock(fdoExtension);
                status = STATUS_SUCCESS;
                (*Irp).IoStatus.Information = core::mem::size_of::<_GET_LENGTH_INFORMATION>() as _;
            }
        }
        IOCTL_DISK_VERIFY => {
            let verifyInfo = (*Irp).AssociatedIrp.SystemBuffer as PVERIFY_INFORMATION;
            if VerifyIrpInBufferSize(Irp, (core::mem::size_of::<_VERIFY_INFORMATION>() as SIZE_T)) == 0 {
                status = STATUS_INFO_LENGTH_MISMATCH;
            } else {
                (*verifyInfo).StartingOffset.QuadPart = ((*verifyInfo).StartingOffset.QuadPart as u64)
                    .wrapping_add((*partExt).StartingOffset as u64) as i64;
                return ForwardIrpAndForget(DeviceObject, Irp);
            }
        }
        IOCTL_DISK_UPDATE_PROPERTIES => {
            (*fdoExtension).LayoutValid = 0u8;
            IoInvalidateDeviceRelations((*fdoExtension).PhysicalDiskDO, BusRelations);
            status = STATUS_SUCCESS;
        }
        IOCTL_STORAGE_GET_DEVICE_NUMBER => {
            let deviceNumber = (*Irp).AssociatedIrp.SystemBuffer as PSTORAGE_DEVICE_NUMBER;
            if VerifyIrpOutBufferSize(Irp, (core::mem::size_of::<_STORAGE_DEVICE_NUMBER>() as SIZE_T)) == 0 {
                status = STATUS_BUFFER_TOO_SMALL;
            } else {
                PartMgrAcquireLayoutLock(fdoExtension);
                (*deviceNumber).DeviceType = (*(*partExt).DeviceObject).DeviceType;
                (*deviceNumber).DeviceNumber = (*fdoExtension).DiskData.DeviceNumber as u32;
                (*deviceNumber).PartitionNumber = (*partExt).DetectedNumber as u32;
                PartMgrReleaseLayoutLock(fdoExtension);
                status = STATUS_SUCCESS;
                (*Irp).IoStatus.Information = core::mem::size_of::<_STORAGE_DEVICE_NUMBER>() as _;
            }
        }
        IOCTL_STORAGE_MEDIA_REMOVAL => {
            return ForwardIrpAndForget(DeviceObject, Irp);
        }
        IOCTL_VOLUME_GET_VOLUME_DISK_EXTENTS => {
            let volExts = (*Irp).AssociatedIrp.SystemBuffer as PVOLUME_DISK_EXTENTS;
            if VerifyIrpOutBufferSize(Irp, (core::mem::size_of::<_VOLUME_DISK_EXTENTS>() as SIZE_T)) == 0 {
                status = STATUS_BUFFER_TOO_SMALL;
            } else {
                PartMgrAcquireLayoutLock(fdoExtension);
                *volExts = _VOLUME_DISK_EXTENTS {
                    NumberOfDiskExtents: 1,
                    Extents: [_DISK_EXTENT {
                        DiskNumber: (*fdoExtension).DiskData.DeviceNumber as u32,
                        StartingOffset: _LARGE_INTEGER { QuadPart: (*partExt).StartingOffset as i64 },
                        ExtentLength: _LARGE_INTEGER { QuadPart: (*partExt).PartitionLength as i64 },
                    }],
                };
                PartMgrReleaseLayoutLock(fdoExtension);
                status = STATUS_SUCCESS;
                (*Irp).IoStatus.Information = core::mem::size_of::<_VOLUME_DISK_EXTENTS>() as _;
            }
        }
        IOCTL_VOLUME_QUERY_VOLUME_NUMBER => {
            let volNum = (*Irp).AssociatedIrp.SystemBuffer as PVOLUME_NUMBER;
            if VerifyIrpOutBufferSize(Irp, (core::mem::size_of::<_VOLUME_NUMBER>() as SIZE_T)) == 0 {
                status = STATUS_BUFFER_TOO_SMALL;
            } else {
                PartMgrAcquireLayoutLock(fdoExtension);
                (*volNum).VolumeNumber = (*partExt).VolumeNumber as u32;
                core::ptr::copy_nonoverlapping(
                    // C: L"VOLMGR  " (8 space-padded WIDE chars) — the old byte
                    // literal was packed ANSI, so the copied u16s were garbage
                    w!("VOLMGR  ").as_ptr(),
                    (*volNum).VolumeManagerName.as_mut_ptr(),
                    8,
                );
                PartMgrReleaseLayoutLock(fdoExtension);
                status = STATUS_SUCCESS;
                (*Irp).IoStatus.Information = core::mem::size_of::<_VOLUME_NUMBER>() as _;
            }
        }
        IOCTL_VOLUME_IS_PARTITION => {
            status = STATUS_SUCCESS;
        }
        IOCTL_VOLUME_ONLINE => {
            status = STATUS_SUCCESS;
        }
        IOCTL_VOLUME_GET_GPT_ATTRIBUTES => {
            let gptAttrs = (*Irp).AssociatedIrp.SystemBuffer as PVOLUME_GET_GPT_ATTRIBUTES_INFORMATION;
            if VerifyIrpOutBufferSize(Irp, (core::mem::size_of::<_VOLUME_GET_GPT_ATTRIBUTES_INFORMATION>() as SIZE_T)) == 0 {
                status = STATUS_BUFFER_TOO_SMALL;
            } else if (*fdoExtension).DiskData.PartitionStyle != PARTITION_STYLE_GPT {
                status = STATUS_INVALID_DEVICE_REQUEST;
            } else {
                (*gptAttrs).GptAttributes = (*partExt).__bindgen_anon_1.Gpt.as_ref().Attributes as u64;
                status = STATUS_SUCCESS;
                (*Irp).IoStatus.Information = core::mem::size_of::<_VOLUME_GET_GPT_ATTRIBUTES_INFORMATION>() as _;
            }
        }
        IOCTL_MOUNTDEV_QUERY_DEVICE_NAME => {
            let name = (*Irp).AssociatedIrp.SystemBuffer as PMOUNTDEV_NAME;
            if VerifyIrpOutBufferSize(Irp, (core::mem::size_of::<u16>() as SIZE_T)) == 0 {
                status = STATUS_BUFFER_TOO_SMALL;
            } else {
                (*name).NameLength = (*partExt).DeviceName.Length;
                let total_size = (core::mem::size_of::<u16>() as usize) + (*name).NameLength as usize;
                if VerifyIrpOutBufferSize(Irp, (total_size as SIZE_T)) == 0 {
                    (*Irp).IoStatus.Information = core::mem::size_of::<u16>() as _;
                    status = STATUS_BUFFER_OVERFLOW;
                } else {
                    core::ptr::copy_nonoverlapping(
                        (*partExt).DeviceName.Buffer as *const u8,
                        (*name).Name.as_mut_ptr() as *mut u8,
                        (*name).NameLength as usize,
                    );
                    status = STATUS_SUCCESS;
                    (*Irp).IoStatus.Information = total_size as _;
                }
            }
        }
        IOCTL_MOUNTDEV_QUERY_UNIQUE_ID => {
            let headerSize: SIZE_T = 2; // FIELD_OFFSET(MOUNTDEV_UNIQUE_ID, UniqueId)
            let uniqueId = (*Irp).AssociatedIrp.SystemBuffer as PMOUNTDEV_UNIQUE_ID;
            let basicVolId = &raw mut (*uniqueId).UniqueId as PBASIC_VOLUME_UNIQUE_ID;
            let mut InterfaceName: PUNICODE_STRING = core::ptr::null_mut::<_UNICODE_STRING>();

            if VerifyIrpOutBufferSize(Irp, (headerSize) as usize) == 0 {
                status = STATUS_BUFFER_TOO_SMALL;
            } else {
                PartMgrAcquireLayoutLock(fdoExtension);
                InterfaceName = &raw mut (*partExt).VolumeInterfaceName as PUNICODE_STRING;
                if (*fdoExtension).IsSuperFloppy != 0 {
                    InterfaceName = &raw mut (*fdoExtension).DiskInterfaceName as PUNICODE_STRING;
                }

                if (*fdoExtension).DiskData.PartitionStyle == PARTITION_STYLE_MBR && (*fdoExtension).IsSuperFloppy == 0 {
                    // C: sizeof(basicVolId->Mbr) — union member size (0x0C). NB: Mbr is a
                    // __BindgenUnionField ZST marker, so size_of_val on it returns 0; size the
                    // member's target type instead.
                    (*uniqueId).UniqueIdLength = core::mem::size_of::<crate::types::_BASIC_VOLUME_UNIQUE_ID__bindgen_ty_1>() as u16;
                } else if (*fdoExtension).DiskData.PartitionStyle == PARTITION_STYLE_GPT {
                    // C: sizeof(basicVolId->Gpt) — union member size (0x18). Same ZST-marker
                    // hazard as Mbr above.
                    (*uniqueId).UniqueIdLength = core::mem::size_of::<crate::types::_BASIC_VOLUME_UNIQUE_ID__bindgen_ty_2>() as u16;
                } else if (*InterfaceName).Buffer.is_null() || (*InterfaceName).Length == 0 {
                    PartMgrReleaseLayoutLock(fdoExtension);
                    status = STATUS_INVALID_PARAMETER;
                } else {
                    (*uniqueId).UniqueIdLength = (*InterfaceName).Length;
                }

                if status != STATUS_INVALID_PARAMETER {
                    let full_size = headerSize.wrapping_add((*uniqueId).UniqueIdLength as SIZE_T);
                    if VerifyIrpOutBufferSize(Irp, (full_size) as usize) == 0 {
                        PartMgrReleaseLayoutLock(fdoExtension);
                        (*Irp).IoStatus.Information = headerSize as _;
                        status = STATUS_BUFFER_OVERFLOW;
                    } else {
                        if (*fdoExtension).DiskData.PartitionStyle == PARTITION_STYLE_MBR && (*fdoExtension).IsSuperFloppy == 0 {
                            (*basicVolId).Mbr.as_mut().Signature = (*fdoExtension).DiskData.__bindgen_anon_1.Mbr.as_ref().Signature as u32;
                            (*basicVolId).Mbr.as_mut().StartingOffset = (*partExt).StartingOffset as u64;
                        } else if (*fdoExtension).DiskData.PartitionStyle == PARTITION_STYLE_GPT {
                            (*basicVolId).Gpt.as_mut().Signature = DMIO_ID_SIGNATURE;
                            (*basicVolId).Gpt.as_mut().PartitionGuid = (*partExt).__bindgen_anon_1.Gpt.as_ref().PartitionId;
                        } else {
                            core::ptr::copy_nonoverlapping(
                                (*InterfaceName).Buffer as *const u8,
                                (*uniqueId).UniqueId.as_mut_ptr() as *mut u8,
                                (*uniqueId).UniqueIdLength as usize,
                            );
                        }
                        PartMgrReleaseLayoutLock(fdoExtension);
                        status = STATUS_SUCCESS;
                        (*Irp).IoStatus.Information = full_size as _;
                    }
                }
            }
        }
        // Handle ignored MountMgr notifications
        // C had only the named cases here; the LLM also emitted CTL_CODE(...,4/5,...,
        // FILE_ANY_ACCESS) arms — those are a DIFFERENT (nonexistent) ioctl value
        // (0x4D0010 vs the real 0x4DC010) AND invalid as patterns. Dropped; the named
        // IOCTL_MOUNTDEV_LINK_CREATED/DELETED above already cover the real values.
        IOCTL_MOUNTDEV_QUERY_SUGGESTED_LINK_NAME
        | IOCTL_MOUNTDEV_LINK_CREATED
        | IOCTL_MOUNTDEV_LINK_DELETED
        | IOCTL_MOUNTDEV_QUERY_STABLE_GUID
        | IOCTL_MOUNTDEV_UNIQUE_ID_CHANGE_NOTIFY => {
            DbgPrint(
                b"Ignored MountMgr notification: 0x%lX\n\0".as_ptr() as *const i8,
                (*ioStack).Parameters.DeviceIoControl.IoControlCode,
            );
            status = STATUS_NOT_IMPLEMENTED;
        }
        _ => {
            return ForwardIrpAndForget(DeviceObject, Irp);
        }
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = status;
    IoCompleteRequest(Irp, IO_NO_INCREMENT);
    status
}
