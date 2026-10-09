// ---- crate-internal glue (private structs, no-op DDK shims, inlines) ----
#[allow(unused_imports)] use crate::types::*;
#[allow(unused_imports)] use crate::ddk_shims::*;
#[allow(unused_imports)] use crate::glue::*;
#[allow(unused_imports)] use crate::inlines::*;
#[allow(unused_imports)] use ntkrnl_sys::ke::KeGetCurrentIrql; // FORCEINLINE on amd64/arm64

use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ddk::{ExFreePool, _KWAIT_REASON_Executive, _IO_STATUS_BLOCK, IoCreateDevice, IoDeleteDevice, IoRegisterFileSystem, IoUnregisterFileSystem, KEVENT, KeEnterCriticalRegion, KeInitializeEvent, KeLeaveCriticalRegion, KeWaitForSingleObject, MmPageEntireDriver, _OBJECT_ATTRIBUTES, PDEVICE_OBJECT, PDRIVER_OBJECT, PIO_STACK_LOCATION, PIRP, PKEVENT, PUNICODE_STRING, RtlInitUnicodeString, _EVENT_TYPE_SynchronizationEvent, ZwClose, ZwCreateFile, ZwLoadDriver};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use ntkrnl_sys::ddk::{ACCESS_MASK, DEVICE_OBJECT, KPRIORITY, KPROCESSOR_MODE, PCSTR, PRKEVENT, RtlAssert, _KEVENT, _LARGE_INTEGER, _UNICODE_STRING};

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static mut FsRecLoadSync: PKEVENT = ::core::ptr::null::<_KEVENT>() as *mut _KEVENT;
#[no_mangle]
pub unsafe extern "system" fn FsRecLoadFileSystem(
    mut DeviceObject: PDEVICE_OBJECT,
    mut DriverServiceName: PWCHAR,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_IMAGE_ALREADY_LOADED;
    let mut DeviceExtension: PDEVICE_EXTENSION = (*DeviceObject).DeviceExtension as PDEVICE_EXTENSION;
    let mut DriverName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: ::core::ptr::null_mut::<WCHAR>(),
    };

    if KeGetCurrentIrql() as ::core::ffi::c_int > (APC_LEVEL) as i32 {
        DbgPrint(
            (b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0").as_ptr() as PCSTR,
            KeGetCurrentIrql() as ::core::ffi::c_int,
        );
        if 0 as ::core::ffi::c_int == 0 {
            RtlAssert(
                (b"0\0").as_ptr() as *const ::core::ffi::c_char as PVOID,
                (b"drivers/filesystems/fs_rec/fs_rec.c\0")
                    .as_ptr() as *const ::core::ffi::c_char as PVOID,
                30 as ULONG,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
            );
        }
    }

    if (*DeviceExtension).State as ::core::ffi::c_uint
        != Loaded as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        KeEnterCriticalRegion();
        KeWaitForSingleObject(
            FsRecLoadSync as PVOID,
            _KWAIT_REASON_Executive,
            0i32 as ::core::ffi::c_int as KPROCESSOR_MODE,
            FALSE as BOOLEAN,
            ::core::ptr::null_mut::<_LARGE_INTEGER>(),
        );
        if (*DeviceExtension).State as ::core::ffi::c_uint
            == Pending as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            RtlInitUnicodeString(&raw mut DriverName, DriverServiceName as PCWSTR);
            Status = ZwLoadDriver(&raw mut DriverName);
            while (*DeviceExtension).State as ::core::ffi::c_uint
                != Unloading as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                (*DeviceExtension).State = Unloading;
                DeviceObject = (*DeviceExtension).Alternate;
                DeviceExtension = (*DeviceObject).DeviceExtension as PDEVICE_EXTENSION;
            }
        }
        if (*DeviceExtension).State as ::core::ffi::c_uint
            != Loaded as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            IoUnregisterFileSystem(DeviceObject);
            (*DeviceExtension).State = Loaded;
        }
        KeSetEvent(
            (FsRecLoadSync as PRKEVENT).cast(),
            IO_NO_INCREMENT as KPRIORITY,
            FALSE as BOOLEAN,
        );
        KeLeaveCriticalRegion();
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn FsRecCreate(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let _ = DeviceObject; // UNREFERENCED_PARAMETER
    let IoStack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    // PAGED_CODE
    if KeGetCurrentIrql() as u32 > APC_LEVEL as u32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            RtlAssert(
                b"0\0".as_ptr() as *mut c_void,
                b"drivers/filesystems/fs_rec/fs_rec.c\0"
                    .as_ptr() as *mut c_void,
                86u32,
                core::ptr::null_mut::<i8>(),
            );
        }
    }
    let Status: NTSTATUS;
    if (*(*IoStack).FileObject).FileName.Length != 0 {
        Status = STATUS_OBJECT_PATH_NOT_FOUND;
    } else {
        Status = STATUS_SUCCESS;
    }
    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    (*Irp).IoStatus.Information = (1usize) as _; // FILE_OPENED
    IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
    Status
}
#[no_mangle]
pub unsafe extern "system" fn FsRecClose(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    // PAGED_CODE() expansion
    if KeGetCurrentIrql() as i32 > APC_LEVEL as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            RtlAssert(
                b"0\0" as *const u8 as *mut core::ffi::c_void,
                b"drivers/filesystems/fs_rec/fs_rec.c\0"
                    as *const u8 as *mut core::ffi::c_void,
                115u32,
                core::ptr::null_mut::<i8>(),
            );
        }
    }
    IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn FsRecFsControl(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let DeviceExtension: PDEVICE_EXTENSION = (*DeviceObject).DeviceExtension as PDEVICE_EXTENSION;
    let mut Status: NTSTATUS;

    if KeGetCurrentIrql() as core::ffi::c_int > (APC_LEVEL) as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as PCSTR,
            KeGetCurrentIrql() as core::ffi::c_int,
        );
        RtlAssert(
            b"0\0" as *const u8 as PVOID,
            b"drivers/filesystems/fs_rec/fs_rec.c\0"
                as *const u8 as PVOID,
            132 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    }

    match (*DeviceExtension).FsType as core::ffi::c_uint {
        1 => {
            Status = FsRecVfatFsControl(DeviceObject, Irp);
        }
        2 => {
            Status = FsRecNtfsFsControl(DeviceObject, Irp);
        }
        3 => {
            Status = FsRecCdfsFsControl(DeviceObject, Irp);
        }
        4 => {
            Status = FsRecUdfsFsControl(DeviceObject, Irp);
        }
        5 => {
            Status = FsRecExtFsControl(DeviceObject, Irp);
        }
        6 => {
            Status = FsRecBtrfsFsControl(DeviceObject, Irp);
        }
        7 => {
            Status = FsRecReiserfsFsControl(DeviceObject, Irp);
        }
        8 => {
            Status = FsRecFfsFsControl(DeviceObject, Irp);
        }
        9 => {
            Status = FsRecFatxFsControl(DeviceObject, Irp);
        }
        _ => {
            Status = STATUS_INVALID_DEVICE_REQUEST;
        }
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    IoCompleteRequest(Irp, IO_NO_INCREMENT);
    Status
}
#[no_mangle]
pub unsafe extern "system" fn FsRecUnload(DriverObject: PDRIVER_OBJECT) {
    if KeGetCurrentIrql() as i32 > (ntkrnl_sys::base::APC_LEVEL) as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as PCSTR,
            KeGetCurrentIrql() as i32,
        );
        if 0 as i32 == 0 {
            RtlAssert(
                b"0\0" as *const u8 as PVOID,
                b"drivers/filesystems/fs_rec/fs_rec.c\0"
                    as *const u8 as PVOID,
                208,
                core::ptr::null_mut::<CHAR>(),
            );
        }
    }
    while !(*DriverObject).DeviceObject.is_null() {
        IoDeleteDevice((*DriverObject).DeviceObject);
    }
    ExFreePool(FsRecLoadSync as PVOID);
}

#[no_mangle]
pub unsafe extern "system" fn FsRecRegisterFs(
    DriverObject: PDRIVER_OBJECT,
    ParentObject: PDEVICE_OBJECT,
    NewDeviceObject: *mut PDEVICE_OBJECT,
    FsName: PCWSTR,
    RecognizerName: PCWSTR,
    FsType: ULONG,
    DeviceType: u32,
    AdditionalFlags: ULONG,
) -> NTSTATUS {
    unsafe {
        let mut ObjectAttributes: _OBJECT_ATTRIBUTES = _OBJECT_ATTRIBUTES {
            Length: 0,
            RootDirectory: core::ptr::null_mut::<core::ffi::c_void>(),
            ObjectName: core::ptr::null_mut::<_UNICODE_STRING>(),
            Attributes: 0,
            SecurityDescriptor: core::ptr::null_mut::<core::ffi::c_void>(),
            SecurityQualityOfService: core::ptr::null_mut::<core::ffi::c_void>(),
        };
        let mut IoStatus: _IO_STATUS_BLOCK = core::mem::zeroed();
        let mut DeviceName: _UNICODE_STRING = _UNICODE_STRING {
            Length: 0,
            MaximumLength: 0,
            Buffer: core::ptr::null_mut::<u16>(),
        };
        let mut FileHandle: HANDLE = core::ptr::null_mut::<core::ffi::c_void>();
        let mut DeviceObject: PDEVICE_OBJECT = core::ptr::null_mut::<DEVICE_OBJECT>();
        let mut Status: NTSTATUS;

        if !NewDeviceObject.is_null() {
            *NewDeviceObject = core::ptr::null_mut::<DEVICE_OBJECT>();
        }

        RtlInitUnicodeString(&raw mut DeviceName, FsName);
        ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
        ObjectAttributes.RootDirectory = core::ptr::null_mut::<core::ffi::c_void>();
        ObjectAttributes.ObjectName = &raw mut DeviceName as PUNICODE_STRING;
        ObjectAttributes.Attributes = (OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE) as ULONG;
        ObjectAttributes.SecurityDescriptor = core::ptr::null_mut::<core::ffi::c_void>();
        ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut::<core::ffi::c_void>();

        Status = ZwCreateFile(
            &raw mut FileHandle,
            SYNCHRONIZE as ACCESS_MASK,
            &raw mut ObjectAttributes,
            &raw mut IoStatus,
            (core::ptr::null_mut::<LARGE_INTEGER>()).cast(),
            0,
            (FILE_SHARE_READ | FILE_SHARE_WRITE) as ULONG,
            FILE_OPEN as ULONG,
            0,
            core::ptr::null_mut::<core::ffi::c_void>(),
            0,
        );
        if NT_SUCCESS(Status) {
            ZwClose(FileHandle);
        } else if Status != STATUS_OBJECT_NAME_NOT_FOUND {
            Status = STATUS_SUCCESS;
        }
        if NT_SUCCESS(Status) {
            return STATUS_IMAGE_ALREADY_LOADED;
        }

        RtlInitUnicodeString(&raw mut DeviceName, RecognizerName);
        Status = IoCreateDevice(
            DriverObject,
            core::mem::size_of::<DEVICE_EXTENSION>() as ULONG,
            &raw mut DeviceName,
            DeviceType,
            0,
            FALSE as BOOLEAN,
            &raw mut DeviceObject,
        );
        if NT_SUCCESS(Status) {
            (*DeviceObject).Flags |= AdditionalFlags as u32;
            let DeviceExtension = (*DeviceObject).DeviceExtension as PDEVICE_EXTENSION;
            (*DeviceExtension).FsType = FsType as FILE_SYSTEM_TYPE;
            (*DeviceExtension).State = Pending;
            if !ParentObject.is_null() {
                (*DeviceExtension).Alternate =
                    (*((*ParentObject).DeviceExtension as PDEVICE_EXTENSION)).Alternate;
                (*((*ParentObject).DeviceExtension as PDEVICE_EXTENSION)).Alternate = DeviceObject;
            } else {
                (*DeviceExtension).Alternate = DeviceObject;
            }
            if !NewDeviceObject.is_null() {
                *NewDeviceObject = DeviceObject;
            }
            IoRegisterFileSystem(DeviceObject);
        }
        Status
    }
}

#[no_mangle]
pub unsafe extern "system" fn DriverEntry(
    DriverObject: PDRIVER_OBJECT,
    RegistryPath: PUNICODE_STRING,
) -> NTSTATUS {
    let mut Status: NTSTATUS = 0;
    let mut DeviceCount: ULONG = 0;
    let mut CdfsObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut UdfsObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut FatObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut ExtObject: PDEVICE_OBJECT = core::ptr::null_mut();

    /* PAGED_CODE!() */
    if (KeGetCurrentIrql() as u32) > (APC_LEVEL) as u32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
            (KeGetCurrentIrql()) as i32,
        );
        RtlAssert(
            b"0\0".as_ptr() as *const i8 as PVOID,
            b"drivers/filesystems/fs_rec/fs_rec.c\0"
                .as_ptr()
                as *const i8 as PVOID,
            334 as ULONG,
            core::ptr::null_mut::<i8>(),
        );
    }

    let _ = RegistryPath;

    /* Page the entire driver */
    MmPageEntireDriver(DriverEntry as PVOID);

    /* Allocate the lock */
    let pool_size = core::mem::size_of::<KEVENT>() as SIZE_T;
    FsRecLoadSync =
        ExAllocatePoolWithTag(NonPagedPool, pool_size, FSREC_TAG as ULONG) as PKEVENT;
    if FsRecLoadSync.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    /* Initialize it */
    KeInitializeEvent(
        FsRecLoadSync as PRKEVENT,
        _EVENT_TYPE_SynchronizationEvent,
        1u8, /* TRUE */
    );

    /* Setup the major functions */
    // Dispatch routines are extern "system" (correct NTAPI per arch); the bindgen
    // MajorFunction / DriverUnload fields are Option<extern "C" fn>. Rust treats the two
    // fn-pointer types as distinct even where the ABI is identical → transmute via
    // *const() (pcix/partmgr idiom).
    use core::mem::transmute as xm;
    (*DriverObject).MajorFunction[IRP_MJ_CREATE as usize] = Some(xm::<*const(), _>(FsRecCreate as *const()));
    (*DriverObject).MajorFunction[IRP_MJ_CLOSE as usize] = Some(xm::<*const(), _>(FsRecClose as *const()));
    (*DriverObject).MajorFunction[IRP_MJ_CLEANUP as usize] = Some(xm::<*const(), _>(FsRecClose as *const()));
    (*DriverObject).MajorFunction[IRP_MJ_FILE_SYSTEM_CONTROL as usize] =
        Some(xm::<*const(), _>(FsRecFsControl as *const()));
    (*DriverObject).DriverUnload = Some(xm::<*const(), _>(FsRecUnload as *const()));

    /* Register CDFS for CDs */
    let w_cdfs: [u16; 6] = [0x005C, 0x0043, 0x0064, 0x0066, 0x0073, 0x0000];
    let w_cdfs_recog: [u16; 27] = [
        0x005C, 0x0046, 0x0069, 0x006C, 0x0065, 0x0053, 0x0079, 0x0073, 0x0074, 0x0065,
        0x006D, 0x005C, 0x0043, 0x0064, 0x0066, 0x0073, 0x0052, 0x0065, 0x0063, 0x006F,
        0x0067, 0x006E, 0x0069, 0x007A, 0x0065, 0x0072, 0x0000,
    ];
    Status = FsRecRegisterFs(
        DriverObject,
        core::ptr::null_mut(),
        &raw mut CdfsObject,
        w_cdfs.as_ptr() as PCWSTR,
        w_cdfs_recog.as_ptr() as PCWSTR,
        FS_TYPE_CDFS as ULONG,
        FILE_DEVICE_CD_ROM_FILE_SYSTEM as ULONG,
        DO_LOW_PRIORITY_FILESYSTEM as ULONG,
    );
    if Status >= 0 {
        DeviceCount += 1;
    }

    /* Register CDFS for HDDs */
    let w_cdfs_hdd: [u16; 9] = [
        0x005C, 0x0043, 0x0064, 0x0066, 0x0073, 0x0048, 0x0064, 0x0064, 0x0000,
    ];
    let w_cdfs_hdd_recog: [u16; 30] = [
        0x005C, 0x0046, 0x0069, 0x006C, 0x0065, 0x0053, 0x0079, 0x0073, 0x0074, 0x0065,
        0x006D, 0x005C, 0x0043, 0x0064, 0x0066, 0x0073, 0x0048, 0x0064, 0x0064, 0x0052,
        0x0065, 0x0063, 0x006F, 0x0067, 0x006E, 0x0069, 0x007A, 0x0065, 0x0072, 0x0000,
    ];
    Status = FsRecRegisterFs(
        DriverObject,
        CdfsObject,
        core::ptr::null_mut(),
        w_cdfs_hdd.as_ptr() as PCWSTR,
        w_cdfs_hdd_recog.as_ptr() as PCWSTR,
        FS_TYPE_CDFS as ULONG,
        FILE_DEVICE_DISK_FILE_SYSTEM as ULONG,
        DO_LOW_PRIORITY_FILESYSTEM as ULONG,
    );
    if Status >= 0 {
        DeviceCount += 1;
    }

    /* Register UDFS for CDs */
    let w_udfs_cd: [u16; 11] = [
        0x005C, 0x0055, 0x0064, 0x0066, 0x0073, 0x0043, 0x0064, 0x0052, 0x006F, 0x006D,
        0x0000,
    ];
    let w_udfs_cd_recog: [u16; 32] = [
        0x005C, 0x0046, 0x0069, 0x006C, 0x0065, 0x0053, 0x0079, 0x0073, 0x0074, 0x0065,
        0x006D, 0x005C, 0x0055, 0x0064, 0x0066, 0x0073, 0x0043, 0x0064, 0x0052, 0x006F,
        0x006D, 0x0052, 0x0065, 0x0063, 0x006F, 0x0067, 0x006E, 0x0069, 0x007A, 0x0065,
        0x0072, 0x0000,
    ];
    Status = FsRecRegisterFs(
        DriverObject,
        core::ptr::null_mut(),
        &raw mut UdfsObject,
        w_udfs_cd.as_ptr() as PCWSTR,
        w_udfs_cd_recog.as_ptr() as PCWSTR,
        FS_TYPE_UDFS as ULONG,
        FILE_DEVICE_CD_ROM_FILE_SYSTEM as ULONG,
        0,
    );
    if Status >= 0 {
        DeviceCount += 1;
    }

    /* Register UDFS for HDDs */
    let w_udfs_disk: [u16; 10] = [
        0x005C, 0x0055, 0x0064, 0x0066, 0x0073, 0x0044, 0x0069, 0x0073, 0x006B, 0x0000,
    ];
    let w_udfs_disk_recog: [u16; 31] = [
        0x005C, 0x0046, 0x0069, 0x006C, 0x0065, 0x0053, 0x0079, 0x0073, 0x0074, 0x0065,
        0x006D, 0x005C, 0x0055, 0x0064, 0x0066, 0x0073, 0x0044, 0x0069, 0x0073, 0x006B,
        0x0052, 0x0065, 0x0063, 0x006F, 0x0067, 0x006E, 0x0069, 0x007A, 0x0065, 0x0072,
        0x0000,
    ];
    Status = FsRecRegisterFs(
        DriverObject,
        UdfsObject,
        core::ptr::null_mut(),
        w_udfs_disk.as_ptr() as PCWSTR,
        w_udfs_disk_recog.as_ptr() as PCWSTR,
        FS_TYPE_UDFS as ULONG,
        FILE_DEVICE_DISK_FILE_SYSTEM as ULONG,
        0,
    );
    if Status >= 0 {
        DeviceCount += 1;
    }

    /* Register FAT */
    let w_fat: [u16; 5] = [0x005C, 0x0046, 0x0061, 0x0074, 0x0000];
    let w_fat_recog: [u16; 26] = [
        0x005C, 0x0046, 0x0069, 0x006C, 0x0065, 0x0053, 0x0079, 0x0073, 0x0074, 0x0065,
        0x006D, 0x005C, 0x0046, 0x0061, 0x0074, 0x0052, 0x0065, 0x0063, 0x006F, 0x0067,
        0x006E, 0x0069, 0x007A, 0x0065, 0x0072, 0x0000,
    ];
    Status = FsRecRegisterFs(
        DriverObject,
        core::ptr::null_mut(),
        &raw mut FatObject,
        w_fat.as_ptr() as PCWSTR,
        w_fat_recog.as_ptr() as PCWSTR,
        FS_TYPE_VFAT as ULONG,
        FILE_DEVICE_DISK_FILE_SYSTEM as ULONG,
        0,
    );
    if Status >= 0 {
        DeviceCount += 1;
    }

    /* Register FAT for CDs */
    let w_fat_cd: [u16; 10] = [
        0x005C, 0x0046, 0x0061, 0x0074, 0x0043, 0x0064, 0x0072, 0x006F, 0x006D, 0x0000,
    ];
    let w_fat_cd_recog: [u16; 31] = [
        0x005C, 0x0046, 0x0069, 0x006C, 0x0065, 0x0053, 0x0079, 0x0073, 0x0074, 0x0065,
        0x006D, 0x005C, 0x0046, 0x0061, 0x0074, 0x0043, 0x0064, 0x0052, 0x006F, 0x006D,
        0x0052, 0x0065, 0x0063, 0x006F, 0x0067, 0x006E, 0x0069, 0x007A, 0x0065, 0x0072,
        0x0000,
    ];
    Status = FsRecRegisterFs(
        DriverObject,
        FatObject,
        core::ptr::null_mut(),
        w_fat_cd.as_ptr() as PCWSTR,
        w_fat_cd_recog.as_ptr() as PCWSTR,
        FS_TYPE_VFAT as ULONG,
        FILE_DEVICE_CD_ROM_FILE_SYSTEM as ULONG,
        0,
    );
    if Status >= 0 {
        DeviceCount += 1;
    }

    /* Register NTFS */
    let w_ntfs: [u16; 6] = [0x005C, 0x004E, 0x0074, 0x0066, 0x0073, 0x0000];
    let w_ntfs_recog: [u16; 27] = [
        0x005C, 0x0046, 0x0069, 0x006C, 0x0065, 0x0053, 0x0079, 0x0073, 0x0074, 0x0065,
        0x006D, 0x005C, 0x004E, 0x0074, 0x0066, 0x0073, 0x0052, 0x0065, 0x0063, 0x006F,
        0x0067, 0x006E, 0x0069, 0x007A, 0x0065, 0x0072, 0x0000,
    ];
    Status = FsRecRegisterFs(
        DriverObject,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        w_ntfs.as_ptr() as PCWSTR,
        w_ntfs_recog.as_ptr() as PCWSTR,
        FS_TYPE_NTFS as ULONG,
        FILE_DEVICE_DISK_FILE_SYSTEM as ULONG,
        0,
    );
    if Status >= 0 {
        DeviceCount += 1;
    }

    /* Register EXT */
    let w_ext: [u16; 7] = [
        0x005C, 0x0045, 0x0078, 0x0074, 0x0066, 0x0073, 0x0000,
    ];
    let w_ext_recog: [u16; 26] = [
        0x005C, 0x0046, 0x0069, 0x006C, 0x0065, 0x0053, 0x0079, 0x0073, 0x0074, 0x0065,
        0x006D, 0x005C, 0x0045, 0x0078, 0x0074, 0x0052, 0x0065, 0x0063, 0x006F, 0x0067,
        0x006E, 0x0069, 0x007A, 0x0065, 0x0072, 0x0000,
    ];
    Status = FsRecRegisterFs(
        DriverObject,
        core::ptr::null_mut(),
        &raw mut ExtObject,
        w_ext.as_ptr() as PCWSTR,
        w_ext_recog.as_ptr() as PCWSTR,
        FS_TYPE_EXT as ULONG,
        FILE_DEVICE_DISK_FILE_SYSTEM as ULONG,
        0,
    );
    if Status >= 0 {
        DeviceCount += 1;
    }

    /* Register EXT for CDs */
    let w_ext_cd: [u16; 12] = [
        0x005C, 0x0045, 0x0078, 0x0074, 0x0066, 0x0073, 0x0043, 0x0064, 0x0072, 0x006F,
        0x006D, 0x0000,
    ];
    let w_ext_cd_recog: [u16; 31] = [
        0x005C, 0x0046, 0x0069, 0x006C, 0x0065, 0x0053, 0x0079, 0x0073, 0x0074, 0x0065,
        0x006D, 0x005C, 0x0045, 0x0078, 0x0074, 0x0043, 0x0064, 0x0052, 0x006F, 0x006D,
        0x0052, 0x0065, 0x0063, 0x006F, 0x0067, 0x006E, 0x0069, 0x007A, 0x0065, 0x0072,
        0x0000,
    ];
    Status = FsRecRegisterFs(
        DriverObject,
        ExtObject,
        core::ptr::null_mut(),
        w_ext_cd.as_ptr() as PCWSTR,
        w_ext_cd_recog.as_ptr() as PCWSTR,
        FS_TYPE_EXT as ULONG,
        FILE_DEVICE_CD_ROM_FILE_SYSTEM as ULONG,
        0,
    );
    if Status >= 0 {
        DeviceCount += 1;
    }

    /* Register BTRFS */
    let w_btrfs: [u16; 7] = [
        0x005C, 0x0042, 0x0074, 0x0072, 0x0066, 0x0073, 0x0000,
    ];
    let w_btrfs_recog: [u16; 28] = [
        0x005C, 0x0046, 0x0069, 0x006C, 0x0065, 0x0053, 0x0079, 0x0073, 0x0074, 0x0065,
        0x006D, 0x005C, 0x0042, 0x0074, 0x0072, 0x0066, 0x0073, 0x0052, 0x0065, 0x0063,
        0x006F, 0x0067, 0x006E, 0x0069, 0x007A, 0x0065, 0x0072, 0x0000,
    ];
    Status = FsRecRegisterFs(
        DriverObject,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        w_btrfs.as_ptr() as PCWSTR,
        w_btrfs_recog.as_ptr() as PCWSTR,
        FS_TYPE_BTRFS as ULONG,
        FILE_DEVICE_DISK_FILE_SYSTEM as ULONG,
        0,
    );
    if Status >= 0 {
        DeviceCount += 1;
    }

    /* Register REISERFS */
    let w_reiser: [u16; 10] = [
        0x005C, 0x0052, 0x0065, 0x0069, 0x0073, 0x0065, 0x0072, 0x0066, 0x0073, 0x0000,
    ];
    let w_reiser_recog: [u16; 31] = [
        0x005C, 0x0046, 0x0069, 0x006C, 0x0065, 0x0053, 0x0079, 0x0073, 0x0074, 0x0065,
        0x006D, 0x005C, 0x0052, 0x0065, 0x0069, 0x0073, 0x0065, 0x0072, 0x0066, 0x0073,
        0x0052, 0x0065, 0x0063, 0x006F, 0x0067, 0x006E, 0x0069, 0x007A, 0x0065, 0x0072,
        0x0000,
    ];
    Status = FsRecRegisterFs(
        DriverObject,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        w_reiser.as_ptr() as PCWSTR,
        w_reiser_recog.as_ptr() as PCWSTR,
        FS_TYPE_REISERFS as ULONG,
        FILE_DEVICE_DISK_FILE_SYSTEM as ULONG,
        0,
    );
    if Status >= 0 {
        DeviceCount += 1;
    }

    /* Register FFS */
    let w_ffs: [u16; 5] = [0x005C, 0x0066, 0x0066, 0x0073, 0x0000];
    let w_ffs_recog: [u16; 26] = [
        0x005C, 0x0046, 0x0069, 0x006C, 0x0065, 0x0053, 0x0079, 0x0073, 0x0074, 0x0065,
        0x006D, 0x005C, 0x0046, 0x0066, 0x0073, 0x0052, 0x0065, 0x0063, 0x006F, 0x0067,
        0x006E, 0x0069, 0x007A, 0x0065, 0x0072, 0x0000,
    ];
    Status = FsRecRegisterFs(
        DriverObject,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        w_ffs.as_ptr() as PCWSTR,
        w_ffs_recog.as_ptr() as PCWSTR,
        FS_TYPE_FFS as ULONG,
        FILE_DEVICE_DISK_FILE_SYSTEM as ULONG,
        0,
    );
    if Status >= 0 {
        DeviceCount += 1;
    }

    /* Register FATX */
    let w_fatx: [u16; 6] = [0x005C, 0x0046, 0x0061, 0x0074, 0x0058, 0x0000];
    let w_fatx_recog: [u16; 27] = [
        0x005C, 0x0046, 0x0069, 0x006C, 0x0065, 0x0053, 0x0079, 0x0073, 0x0074, 0x0065,
        0x006D, 0x005C, 0x0046, 0x0061, 0x0074, 0x0058, 0x0052, 0x0065, 0x0063, 0x006F,
        0x0067, 0x006E, 0x0069, 0x007A, 0x0065, 0x0072, 0x0000,
    ];
    Status = FsRecRegisterFs(
        DriverObject,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        w_fatx.as_ptr() as PCWSTR,
        w_fatx_recog.as_ptr() as PCWSTR,
        FS_TYPE_FATX as ULONG,
        FILE_DEVICE_DISK_FILE_SYSTEM as ULONG,
        0,
    );
    if Status >= 0 {
        DeviceCount += 1;
    }

    /* Return appropriate Status */
    if DeviceCount > 0 {
        STATUS_SUCCESS
    } else {
        STATUS_IMAGE_ALREADY_LOADED
    }
}
