use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ex::*;

use ntkrnl_sys::ddk::{
    _DEVICE_OBJECT, _VPB,
    _DISK_GEOMETRY, _PARTITION_INFORMATION, _KEVENT, _IO_STATUS_BLOCK,
    KMUTEX, _SHARE_ACCESS, CSHORT, ULONG_PTR,
    PDEVICE_OBJECT, PDRIVER_OBJECT, PFILE_OBJECT, PIRP, PIO_STACK_LOCATION,
    PVPB, PUNICODE_STRING,
    ACCESS_MASK,
    IoAcquireVpbSpinLock, IoReleaseVpbSpinLock, IoCreateDevice, IoDeleteDevice,
    IoGetCurrentIrpStackLocation, IoGetNextIrpStackLocation, IoMarkIrpPending,
    IofCompleteRequest, IofCallDriver,
    IoSetCompletionRoutine, IoCheckShareAccess, IoSetShareAccess, IoRemoveShareAccess,
    IoRegisterFileSystem, IoGetRelatedDeviceObject,
    IoCreateStreamFileObjectLite, IoBuildDeviceIoControlRequest,
    KeInitializeEvent, KeInitializeMutex, KeWaitForSingleObject,
    KeReleaseMutex, KeClearEvent, KeBugCheck,
    RtlInitUnicodeString,
    ExFreePool,
    PFILE_FS_VOLUME_INFORMATION, PFILE_FS_SIZE_INFORMATION,
    PFILE_FS_DEVICE_INFORMATION, PFILE_FS_ATTRIBUTE_INFORMATION,
    PFILE_POSITION_INFORMATION,
    _KWAIT_REASON_Executive, _EVENT_TYPE_NotificationEvent,
};

extern "C" {
    #[link_name = "IoCopyCurrentIrpStackLocationToNext_shim"]
    fn IoCopyCurrentIrpStackLocationToNext(Irp: PIRP);
}

#[repr(C)]
struct VCB {
    NodeTypeCode: USHORT,
    NodeByteSize: USHORT,
    TargetDeviceObject: PDEVICE_OBJECT,
    Vpb: PVPB,
    LocalVpb: PVPB,
    VcbState: ULONG,
    Mutex: KMUTEX,
    OpenCount: LONG,
    ShareAccess: _SHARE_ACCESS,
    BytesPerSector: ULONG,
    SectorsOnDisk: LARGE_INTEGER,
}

type PVCB = *mut VCB;

#[repr(C)]
struct VOLUME_DEVICE_OBJECT {
    DeviceObject: _DEVICE_OBJECT,
    Vcb: VCB,
}

type PVOLUME_DEVICE_OBJECT = *mut VOLUME_DEVICE_OBJECT;

const VCB_STATE_LOCKED: ULONG = 0x00000001;
const VCB_STATE_DISMOUNTED: ULONG = 0x00000002;

static mut RawDiskDeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
static mut RawCdromDeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
static mut RawTapeDeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();

use ntkrnl_sys::inline::common::{FsRtlEnterFileSystem, FsRtlExitFileSystem};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

extern "system" {
    fn FsRtlNotifyVolumeEvent(FileObject: PFILE_OBJECT, EventCode: ULONG) -> NTSTATUS;
}

#[inline]
unsafe fn IoCompleteRequest(Irp: PIRP, PriorityBoost: i8) {
    IofCompleteRequest(Irp, PriorityBoost);
}

#[inline]
unsafe fn IoCallDriver(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    IofCallDriver(DeviceObject, Irp)
}

const FSRTL_VOLUME_MOUNT: ULONG = 0x06;
const FSRTL_VOLUME_LOCK: ULONG = 0x3;
const FSRTL_VOLUME_LOCK_FAILED: ULONG = 0x4;
const FSRTL_VOLUME_UNLOCK: ULONG = 0x05;
const FSRTL_VOLUME_DISMOUNT: ULONG = 0x1;
const FSRTL_VOLUME_DISMOUNT_FAILED: ULONG = 0x2;

const IO_DISK_INCREMENT: i8 = 1;

const IRP_MJ_CREATE: u8 = 0x00;
const IRP_MJ_CLOSE: u8 = 0x02;
const IRP_MJ_READ: u8 = 0x03;
const IRP_MJ_WRITE: u8 = 0x04;
const IRP_MJ_QUERY_INFORMATION: u8 = 0x05;
const IRP_MJ_SET_INFORMATION: u8 = 0x06;
const IRP_MJ_QUERY_VOLUME_INFORMATION: u8 = 0x0a;
const IRP_MJ_DEVICE_CONTROL: u8 = 0x0e;
const IRP_MJ_SHUTDOWN: u8 = 0x10;
const IRP_MJ_CLEANUP: u8 = 0x12;
const IRP_MJ_FILE_SYSTEM_CONTROL: u8 = 0x0d;

const IRP_MN_USER_FS_REQUEST: u8 = 0x00;
const IRP_MN_MOUNT_VOLUME: u8 = 0x01;
const IRP_MN_VERIFY_VOLUME: u8 = 0x02;

const FILE_OPEN: ULONG = 0x00000001;
const FILE_DIRECTORY_FILE: ULONG = 0x00000001;
const FILE_OPENED: ULONG = 1;

const FO_SYNCHRONOUS_IO: ULONG = 0x00000002;
const FO_NO_INTERMEDIATE_BUFFERING: ULONG = 0x00000008;
const FO_STREAM_FILE: ULONG = 0x0100;

const DO_VERIFY_VOLUME: ULONG = 0x00000002;
const DO_DIRECT_IO: ULONG = 0x00000010;
const DO_DEVICE_INITIALIZING: ULONG = 0x00000080;

const FILE_FLOPPY_DISKETTE: ULONG = 0x00000004;

const SL_OVERRIDE_VERIFY_VOLUME: ULONG = 0x02;

const VPB_MOUNTED: USHORT = 0x0001;
const VPB_PERSISTENT: USHORT = 0x04;
const VPB_REMOVE_PENDING: USHORT = 0x08;

const IO_TYPE_VPB: USHORT = 10;

const FILE_DEVICE_DISK_FILE_SYSTEM: ULONG = 0x00000008;
const FILE_DEVICE_CD_ROM_FILE_SYSTEM: ULONG = 0x00000003;
const FILE_DEVICE_TAPE_FILE_SYSTEM: ULONG = 0x20;
const FILE_DEVICE_DISK: ULONG = 0x00000007;

const IOCTL_DISK_GET_DRIVE_GEOMETRY: ULONG = 0x00070000;
const IOCTL_DISK_GET_PARTITION_INFO: ULONG = 0x00074004;

const FSCTL_REQUEST_OPLOCK_LEVEL_1: ULONG = 0x00090000;
const FSCTL_REQUEST_OPLOCK_LEVEL_2: ULONG = 0x00090004;
const FSCTL_OPLOCK_BREAK_ACKNOWLEDGE: ULONG = 0x0009000C;
const FSCTL_OPLOCK_BREAK_NOTIFY: ULONG = 0x00090014;
const FSCTL_LOCK_VOLUME: ULONG = 0x00090018;
const FSCTL_UNLOCK_VOLUME: ULONG = 0x0009001C;
const FSCTL_DISMOUNT_VOLUME: ULONG = 0x00090020;

const FILE_SYSTEM: ULONG = 0x22;

const STATUS_VOLUME_DISMOUNTED: NTSTATUS = 0xC000026Eu32 as i32;
const STATUS_NOT_LOCKED: NTSTATUS = 0xC000002Au32 as i32;
const STATUS_WRONG_VOLUME: NTSTATUS = 0xC0000012u32 as i32;

const FilePositionInformation: u32 = 14;
const FileFsVolumeInformation: u32 = 1;
const FileFsSizeInformation: u32 = 3;
const FileFsDeviceInformation: u32 = 4;
const FileFsAttributeInformation: u32 = 5;

#[inline]
unsafe fn CONTAINING_RECORD<T, U>(address: *const U, field_offset: usize) -> *mut T {
    (address as usize - field_offset) as *mut T
}

#[inline]
unsafe fn FlagOn(flags: ULONG, bit: ULONG) -> bool {
    (flags & bit) != 0
}

#[inline]
unsafe fn max(a: ULONG, b: ULONG) -> ULONG {
    if a > b { a } else { b }
}

unsafe fn RawInitializeVcb(
    Vcb: PVCB,
    TargetDeviceObject: PDEVICE_OBJECT,
    Vpb: PVPB,
) -> NTSTATUS {
    let mut Status = STATUS_SUCCESS;

    RtlZeroMemory(Vcb as PVOID, core::mem::size_of::<VCB>());

    (*Vcb).TargetDeviceObject = TargetDeviceObject;
    (*Vcb).Vpb = Vpb;

    KeInitializeMutex(core::ptr::addr_of_mut!((*Vcb).Mutex), 0);

    (*Vcb).LocalVpb = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<_VPB>(),
        u32::from_le_bytes(*b" waR"),
    ) as PVPB;
    if (*Vcb).LocalVpb.is_null() {
        Status = STATUS_INSUFFICIENT_RESOURCES;
    }

    Status
}

unsafe fn RawCheckForDismount(Vcb: PVCB, CreateOperation: BOOLEAN) -> BOOLEAN {
    let mut OldIrql: KIRQL = 0;
    let Vpb: PVPB;
    let Delete: BOOLEAN;

    IoAcquireVpbSpinLock(core::ptr::addr_of_mut!(OldIrql));

    Vpb = (*Vcb).Vpb;
    if (*(*Vcb).Vpb).ReferenceCount != CreateOperation as ULONG {
        if !(*Vcb).LocalVpb.is_null() {
            RtlZeroMemory((*Vcb).LocalVpb as PVOID, core::mem::size_of::<_VPB>());
            (*(*Vcb).LocalVpb).Type = IO_TYPE_VPB as CSHORT;
            (*(*Vcb).LocalVpb).Size = core::mem::size_of::<_VPB>() as CSHORT;
            (*(*Vcb).LocalVpb).RealDevice = (*(*Vcb).Vpb).RealDevice;
            (*(*Vcb).LocalVpb).DeviceObject = core::ptr::null_mut();
            (*(*Vcb).LocalVpb).Flags = (*(*Vcb).Vpb).Flags & VPB_REMOVE_PENDING;
            (*(*(*Vcb).Vpb).RealDevice).Vpb = (*Vcb).LocalVpb;
            (*Vcb).LocalVpb = core::ptr::null_mut();
            (*(*Vcb).Vpb).Flags |= VPB_PERSISTENT;
        }

        Delete = 0u8;
    } else {
        Delete = 1u8;

        if (*(*Vpb).RealDevice).Vpb == Vpb {
            (*Vpb).DeviceObject = core::ptr::null_mut();
            (*Vpb).Flags &= !VPB_MOUNTED;
        }
    }

    IoReleaseVpbSpinLock(OldIrql);

    if Delete != 0 {
        KeReleaseMutex(core::ptr::addr_of_mut!((*Vcb).Mutex), 0u8);

        if !(*Vcb).LocalVpb.is_null() {
            ExFreePool((*Vcb).LocalVpb as PVOID);
        } else if (*(*Vcb).Vpb).ReferenceCount == 0 {
            ExFreePool((*Vcb).Vpb as PVOID);
        }

        ObDereferenceObject((*Vcb).TargetDeviceObject as PVOID);
        IoDeleteDevice(CONTAINING_RECORD::<VOLUME_DEVICE_OBJECT, VCB>(
            Vcb,
            core::mem::offset_of!(VOLUME_DEVICE_OBJECT, Vcb),
        ) as PDEVICE_OBJECT);
    }

    Delete
}

#[no_mangle]
pub unsafe extern "system" fn RawCompletionRoutine(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Context: PVOID,
) -> NTSTATUS {
    let DeviceObject = opaque_ptr(DeviceObject);
    let Irp = opaque_ptr(Irp);
    let Context = opaque_ptr(Context);
    let IoStackLocation = IoGetCurrentIrpStackLocation(Irp);

    if (((*IoStackLocation).MajorFunction == IRP_MJ_READ)
        || ((*IoStackLocation).MajorFunction == IRP_MJ_WRITE))
        && !(*IoStackLocation).FileObject.is_null()
        && FlagOn((*(*IoStackLocation).FileObject).Flags, FO_SYNCHRONOUS_IO)
        && (*Irp).IoStatus.__bindgen_anon_1.Status >= 0
    {
        (*(*IoStackLocation).FileObject).CurrentByteOffset.QuadPart +=
            (*Irp).IoStatus.Information as i64;
    }

    if (*Irp).PendingReturned != 0 {
        IoMarkIrpPending(Irp);
    }
    STATUS_SUCCESS
}

unsafe fn RawClose(Vcb: PVCB, Irp: PIRP, IoStackLocation: PIO_STACK_LOCATION) -> NTSTATUS {
    let Status: NTSTATUS;

    if FlagOn((*(*IoStackLocation).FileObject).Flags, FO_STREAM_FILE) {
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
        IoCompleteRequest(Irp, IO_DISK_INCREMENT);
        return STATUS_SUCCESS;
    }

    Status = KeWaitForSingleObject(
        core::ptr::addr_of_mut!((*Vcb).Mutex) as PVOID,
        _KWAIT_REASON_Executive,
        0i8,
        0u8,
        core::ptr::null_mut(),
    );

    (*Vcb).OpenCount -= 1;
    if (*Vcb).OpenCount != 0 || RawCheckForDismount(Vcb, 0u8) == 0 {
        KeReleaseMutex(core::ptr::addr_of_mut!((*Vcb).Mutex), 0u8);
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    IoCompleteRequest(Irp, IO_DISK_INCREMENT);
    STATUS_SUCCESS
}

unsafe fn RawCreate(Vcb: PVCB, Irp: PIRP, IoStackLocation: PIO_STACK_LOCATION) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let ShareAccess: ULONG;
    let DesiredAccess: ACCESS_MASK;
    let mut Deleted = 0u8;

    Status = KeWaitForSingleObject(
        core::ptr::addr_of_mut!((*Vcb).Mutex) as PVOID,
        _KWAIT_REASON_Executive,
        0i8,
        0u8,
        core::ptr::null_mut(),
    );

    if ((*IoStackLocation).FileObject.is_null()
        || (*(*IoStackLocation).FileObject).FileName.Length == 0)
        && (((*IoStackLocation).Parameters.Create.Options >> 24) == FILE_OPEN)
        && !FlagOn(
            (*IoStackLocation).Parameters.Create.Options,
            FILE_DIRECTORY_FILE,
        )
    {
        if FlagOn((*Vcb).VcbState, VCB_STATE_LOCKED) {
            Status = STATUS_ACCESS_DENIED;
            (*Irp).IoStatus.Information = 0;
        } else if FlagOn((*Vcb).VcbState, VCB_STATE_DISMOUNTED) {
            Status = STATUS_VOLUME_DISMOUNTED;
            (*Irp).IoStatus.Information = 0;
        } else {
            ShareAccess = (*IoStackLocation).Parameters.Create.ShareAccess as ULONG;
            DesiredAccess = (*(*IoStackLocation).Parameters.Create.SecurityContext).DesiredAccess;

            if (*Vcb).OpenCount > 0 {
                Status = IoCheckShareAccess(
                    DesiredAccess,
                    ShareAccess,
                    (*IoStackLocation).FileObject,
                    core::ptr::addr_of_mut!((*Vcb).ShareAccess),
                    1u8,
                );
                if Status < 0 {
                    (*Irp).IoStatus.Information = 0;
                }
            }

            if Status >= 0 {
                if (*Vcb).OpenCount == 0 {
                    IoSetShareAccess(
                        DesiredAccess,
                        ShareAccess,
                        (*IoStackLocation).FileObject,
                        core::ptr::addr_of_mut!((*Vcb).ShareAccess),
                    );
                }

                (*Vcb).OpenCount += 1;
                (*(*IoStackLocation).FileObject).Vpb = (*Vcb).Vpb;

                Status = STATUS_SUCCESS;
                (*Irp).IoStatus.Information = FILE_OPENED as ULONG_PTR;
                (*(*IoStackLocation).FileObject).Flags |= FO_NO_INTERMEDIATE_BUFFERING;
            }
        }
    } else {
        Status = STATUS_INVALID_PARAMETER;
        (*Irp).IoStatus.Information = 0;
    }

    if Status < 0 && (*Vcb).OpenCount == 0 {
        Deleted = RawCheckForDismount(Vcb, 1u8);
    }

    if Deleted == 0 {
        KeReleaseMutex(core::ptr::addr_of_mut!((*Vcb).Mutex), 0u8);
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    IoCompleteRequest(Irp, IO_DISK_INCREMENT);
    Status
}

unsafe fn RawReadWriteDeviceControl(
    Vcb: PVCB,
    Irp: PIRP,
    IoStackLocation: PIO_STACK_LOCATION,
) -> NTSTATUS {
    let Status: NTSTATUS;

    if (((*IoStackLocation).MajorFunction == IRP_MJ_READ)
        || ((*IoStackLocation).MajorFunction == IRP_MJ_WRITE))
        && (*IoStackLocation).Parameters.Read.Length == 0
    {
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
        IoCompleteRequest(Irp, IO_DISK_INCREMENT);
        return STATUS_SUCCESS;
    }

    IoCopyCurrentIrpStackLocationToNext(Irp);

    (*IoGetNextIrpStackLocation(Irp)).Flags |= SL_OVERRIDE_VERIFY_VOLUME as u8;

    IoSetCompletionRoutine(Irp, core::mem::transmute(RawCompletionRoutine as usize), core::ptr::null_mut(), 1u8, 1u8, 1u8);

    Status = IoCallDriver((*Vcb).TargetDeviceObject, Irp);
    Status
}

unsafe fn RawMountVolume(IoStackLocation: PIO_STACK_LOCATION) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let DeviceObject: PDEVICE_OBJECT;
    let mut Volume: PVOLUME_DEVICE_OBJECT = core::ptr::null_mut();
    let mut FileObject: PFILE_OBJECT = core::ptr::null_mut();

    DeviceObject = (*IoStackLocation).Parameters.MountVolume.DeviceObject;

    Status = IoCreateDevice(
        (*RawDiskDeviceObject).DriverObject,
        (core::mem::size_of::<VOLUME_DEVICE_OBJECT>() - core::mem::size_of::<_DEVICE_OBJECT>())
            as ULONG,
        core::ptr::null_mut(),
        FILE_DEVICE_DISK_FILE_SYSTEM,
        0,
        0u8,
        core::ptr::addr_of_mut!(Volume) as *mut PDEVICE_OBJECT,
    );
    if Status < 0 {
        return Status;
    }

    (*Volume).DeviceObject.AlignmentRequirement = max(
        (*DeviceObject).AlignmentRequirement,
        (*Volume).DeviceObject.AlignmentRequirement,
    );

    Status = RawInitializeVcb(
        core::ptr::addr_of_mut!((*Volume).Vcb),
        (*IoStackLocation).Parameters.MountVolume.DeviceObject,
        (*IoStackLocation).Parameters.MountVolume.Vpb,
    );
    if Status < 0 {
        IoDeleteDevice(Volume as PDEVICE_OBJECT);
        return Status;
    }

    (*(*Volume).Vcb.Vpb).SerialNumber = 0xFFFFFFFF;
    (*(*Volume).Vcb.Vpb).VolumeLabelLength = 0;

    (*(*Volume).Vcb.Vpb).DeviceObject = core::ptr::addr_of_mut!((*Volume).DeviceObject);
    (*Volume).DeviceObject.StackSize = (*DeviceObject).StackSize + 1;
    (*Volume).DeviceObject.SectorSize = (*DeviceObject).SectorSize;
    (*Volume).DeviceObject.Flags |= DO_DIRECT_IO;
    (*Volume).DeviceObject.Flags &= !DO_DEVICE_INITIALIZING;

    FileObject = IoCreateStreamFileObjectLite(
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!((*Volume).DeviceObject),
    );
    if FileObject.is_null() {
        Status = STATUS_INSUFFICIENT_RESOURCES;
    }

    if Status < 0 {
        IoDeleteDevice(Volume as PDEVICE_OBJECT);
        return Status;
    }

    (*Volume).Vcb.OpenCount += 2;
    FsRtlNotifyVolumeEvent(FileObject, FSRTL_VOLUME_MOUNT);
    ObDereferenceObject(FileObject as PVOID);
    (*Volume).Vcb.OpenCount -= 2;

    Status
}

unsafe fn RawUserFsCtrl(IoStackLocation: PIO_STACK_LOCATION, Vcb: PVCB) -> NTSTATUS {
    let mut Status: NTSTATUS;

    Status = KeWaitForSingleObject(
        core::ptr::addr_of_mut!((*Vcb).Mutex) as PVOID,
        _KWAIT_REASON_Executive,
        0i8,
        0u8,
        core::ptr::null_mut(),
    );

    match (*IoStackLocation).Parameters.FileSystemControl.FsControlCode {
        FSCTL_REQUEST_OPLOCK_LEVEL_1
        | FSCTL_REQUEST_OPLOCK_LEVEL_2
        | FSCTL_OPLOCK_BREAK_ACKNOWLEDGE
        | FSCTL_OPLOCK_BREAK_NOTIFY => {
            Status = STATUS_NOT_IMPLEMENTED;
        }
        FSCTL_LOCK_VOLUME => {
            if !FlagOn((*Vcb).VcbState, VCB_STATE_LOCKED) && (*Vcb).OpenCount == 1 {
                (*Vcb).VcbState |= VCB_STATE_LOCKED;
                Status = STATUS_SUCCESS;
            } else {
                Status = STATUS_ACCESS_DENIED;
            }
        }
        FSCTL_UNLOCK_VOLUME => {
            if !FlagOn((*Vcb).VcbState, VCB_STATE_LOCKED) {
                Status = STATUS_NOT_LOCKED;
            } else {
                (*Vcb).VcbState &= !VCB_STATE_LOCKED;
                Status = STATUS_SUCCESS;
            }
        }
        FSCTL_DISMOUNT_VOLUME => {
            if FlagOn((*Vcb).VcbState, VCB_STATE_LOCKED) {
                Status = STATUS_SUCCESS;
            } else {
                Status = STATUS_ACCESS_DENIED;
            }
        }
        _ => {
            Status = STATUS_INVALID_PARAMETER;
        }
    }

    KeReleaseMutex(core::ptr::addr_of_mut!((*Vcb).Mutex), 0u8);

    match (*IoStackLocation).Parameters.FileSystemControl.FsControlCode {
        FSCTL_LOCK_VOLUME => {
            FsRtlNotifyVolumeEvent(
                (*IoStackLocation).FileObject,
                if Status >= 0 {
                    FSRTL_VOLUME_LOCK
                } else {
                    FSRTL_VOLUME_LOCK_FAILED
                },
            );
        }
        FSCTL_UNLOCK_VOLUME => {
            if Status >= 0 {
                FsRtlNotifyVolumeEvent((*IoStackLocation).FileObject, FSRTL_VOLUME_UNLOCK);
            }
        }
        FSCTL_DISMOUNT_VOLUME => {
            FsRtlNotifyVolumeEvent(
                (*IoStackLocation).FileObject,
                if Status >= 0 {
                    FSRTL_VOLUME_DISMOUNT
                } else {
                    FSRTL_VOLUME_DISMOUNT_FAILED
                },
            );
        }
        _ => {}
    }

    Status
}

unsafe fn RawFileSystemControl(
    Vcb: PVCB,
    Irp: PIRP,
    IoStackLocation: PIO_STACK_LOCATION,
) -> NTSTATUS {
    let mut Status: NTSTATUS;

    match (*IoStackLocation).MinorFunction {
        IRP_MN_USER_FS_REQUEST => {
            Status = RawUserFsCtrl(IoStackLocation, Vcb);
        }
        IRP_MN_MOUNT_VOLUME => {
            Status = RawMountVolume(IoStackLocation);
        }
        IRP_MN_VERIFY_VOLUME => {
            Status = KeWaitForSingleObject(
                core::ptr::addr_of_mut!((*Vcb).Mutex) as PVOID,
                _KWAIT_REASON_Executive,
                0i8,
                0u8,
                core::ptr::null_mut(),
            );

            Status = STATUS_WRONG_VOLUME;
            (*(*(*Vcb).Vpb).RealDevice).Flags &= !DO_VERIFY_VOLUME;

            if (*Vcb).OpenCount != 0 || RawCheckForDismount(Vcb, 0u8) == 0 {
                KeReleaseMutex(core::ptr::addr_of_mut!((*Vcb).Mutex), 0u8);
            }
        }
        _ => {
            Status = STATUS_INVALID_DEVICE_REQUEST;
        }
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    IoCompleteRequest(Irp, IO_DISK_INCREMENT);
    Status
}

unsafe fn RawQueryInformation(
    Vcb: PVCB,
    Irp: PIRP,
    IoStackLocation: PIO_STACK_LOCATION,
) -> NTSTATUS {
    let mut Status = STATUS_INVALID_DEVICE_REQUEST;
    let Length: *mut ULONG;
    let Buffer: PFILE_POSITION_INFORMATION;

    Length = core::ptr::addr_of_mut!((*IoStackLocation).Parameters.QueryFile.Length);
    Buffer = (*Irp).AssociatedIrp.SystemBuffer as PFILE_POSITION_INFORMATION;

    if (*IoStackLocation).Parameters.QueryFile.FileInformationClass == FilePositionInformation {
        if *Length < core::mem::size_of::<ntkrnl_sys::ddk::_FILE_POSITION_INFORMATION>() as ULONG {
            (*Irp).IoStatus.Information = 0;
            Status = STATUS_BUFFER_OVERFLOW;
        } else {
            (*Buffer).CurrentByteOffset = (*(*IoStackLocation).FileObject).CurrentByteOffset;
            *Length -= core::mem::size_of::<ntkrnl_sys::ddk::_FILE_POSITION_INFORMATION>() as ULONG;

            (*Irp).IoStatus.Information =
                core::mem::size_of::<ntkrnl_sys::ddk::_FILE_POSITION_INFORMATION>() as ULONG_PTR;
            Status = STATUS_SUCCESS;
        }
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    IoCompleteRequest(Irp, IO_DISK_INCREMENT);
    Status
}

unsafe fn RawSetInformation(
    Vcb: PVCB,
    Irp: PIRP,
    IoStackLocation: PIO_STACK_LOCATION,
) -> NTSTATUS {
    let mut Status = STATUS_INVALID_DEVICE_REQUEST;
    let Buffer: PFILE_POSITION_INFORMATION;
    let DeviceObject: PDEVICE_OBJECT;

    Buffer = (*Irp).AssociatedIrp.SystemBuffer as PFILE_POSITION_INFORMATION;

    if (*IoStackLocation).Parameters.QueryFile.FileInformationClass == FilePositionInformation {
        DeviceObject = IoGetRelatedDeviceObject((*IoStackLocation).FileObject);

        if ((*Buffer).CurrentByteOffset.QuadPart as ULONG
            & (*DeviceObject).AlignmentRequirement)
            != 0
        {
            Status = STATUS_INVALID_PARAMETER;
        } else {
            (*(*IoStackLocation).FileObject).CurrentByteOffset =
                (*Buffer).CurrentByteOffset;

            Status = STATUS_SUCCESS;
        }
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    IoCompleteRequest(Irp, IO_DISK_INCREMENT);
    Status
}

unsafe fn RawQueryFsVolumeInfo(
    Vcb: PVCB,
    Buffer: PFILE_FS_VOLUME_INFORMATION,
    Length: *mut ULONG,
) -> NTSTATUS {
    RtlZeroMemory(
        Buffer as PVOID,
        core::mem::size_of::<ntkrnl_sys::ddk::_FILE_FS_VOLUME_INFORMATION>(),
    );
    (*Buffer).VolumeSerialNumber = (*(*Vcb).Vpb).SerialNumber;
    (*Buffer).SupportsObjects = 0u8;
    (*Buffer).VolumeLabelLength = 0;

    *Length -= core::mem::offset_of!(ntkrnl_sys::ddk::_FILE_FS_VOLUME_INFORMATION, VolumeLabel)
        as ULONG;
    STATUS_SUCCESS
}

unsafe fn RawQueryFsSizeInfo(
    Vcb: PVCB,
    Buffer: PFILE_FS_SIZE_INFORMATION,
    Length: *mut ULONG,
) -> NTSTATUS {
    let mut Irp: PIRP;
    let mut Event: _KEVENT = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let RealDevice: PDEVICE_OBJECT;
    let mut DiskGeometry: _DISK_GEOMETRY = core::mem::zeroed();
    let mut PartitionInformation: _PARTITION_INFORMATION = core::mem::zeroed();
    let mut DiskHasPartitions: BOOLEAN;

    if *Length < core::mem::size_of::<ntkrnl_sys::ddk::_FILE_FS_SIZE_INFORMATION>() as ULONG {
        return STATUS_BUFFER_OVERFLOW;
    }

    RtlZeroMemory(
        Buffer as PVOID,
        core::mem::size_of::<ntkrnl_sys::ddk::_FILE_FS_SIZE_INFORMATION>(),
    );
    KeInitializeEvent(core::ptr::addr_of_mut!(Event), _EVENT_TYPE_NotificationEvent, 0u8);
    RealDevice = (*(*Vcb).Vpb).RealDevice;

    Irp = IoBuildDeviceIoControlRequest(
        IOCTL_DISK_GET_DRIVE_GEOMETRY,
        RealDevice,
        core::ptr::null_mut(),
        0,
        core::ptr::addr_of_mut!(DiskGeometry) as PVOID,
        core::mem::size_of::<_DISK_GEOMETRY>() as ULONG,
        0u8,
        core::ptr::addr_of_mut!(Event),
        core::ptr::addr_of_mut!(IoStatusBlock),
    );
    if Irp.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Status = IoCallDriver(RealDevice, Irp);
    if Status == STATUS_PENDING {
        KeWaitForSingleObject(
            core::ptr::addr_of_mut!(Event) as PVOID,
            _KWAIT_REASON_Executive,
            0i8,
            0u8,
            core::ptr::null_mut(),
        );
        Status = IoStatusBlock.__bindgen_anon_1.Status;
    }

    if Status < 0 {
        *Length = 0;
        return Status;
    }

    if FlagOn((*RealDevice).Characteristics, FILE_FLOPPY_DISKETTE) {
        DiskHasPartitions = 0u8;
    } else {
        KeClearEvent(core::ptr::addr_of_mut!(Event));
        Irp = IoBuildDeviceIoControlRequest(
            IOCTL_DISK_GET_PARTITION_INFO,
            RealDevice,
            core::ptr::null_mut(),
            0,
            core::ptr::addr_of_mut!(PartitionInformation) as PVOID,
            core::mem::size_of::<_PARTITION_INFORMATION>() as ULONG,
            0u8,
            core::ptr::addr_of_mut!(Event),
            core::ptr::addr_of_mut!(IoStatusBlock),
        );
        if Irp.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        Status = IoCallDriver(RealDevice, Irp);
        if Status == STATUS_PENDING {
            KeWaitForSingleObject(
                core::ptr::addr_of_mut!(Event) as PVOID,
                _KWAIT_REASON_Executive,
                0i8,
                0u8,
                core::ptr::null_mut(),
            );
            Status = IoStatusBlock.__bindgen_anon_1.Status;
        }

        if Status == STATUS_INVALID_DEVICE_REQUEST {
            DiskHasPartitions = 0u8;
        } else {
            DiskHasPartitions = 1u8;
        }
    }

    (*Buffer).BytesPerSector = DiskGeometry.BytesPerSector;
    (*Buffer).SectorsPerAllocationUnit = 1;

    if DiskHasPartitions != 0 {
        let mut total: ntkrnl_sys::ddk::_LARGE_INTEGER = core::mem::zeroed();
        total.QuadPart = PartitionInformation.PartitionLength.QuadPart
            / DiskGeometry.BytesPerSector as i64;
        (*Buffer).TotalAllocationUnits = total;
    } else {
        let mut total: ntkrnl_sys::ddk::_LARGE_INTEGER = core::mem::zeroed();
        total.QuadPart = DiskGeometry.Cylinders.QuadPart
            * (DiskGeometry.TracksPerCylinder * DiskGeometry.SectorsPerTrack) as i64;
        (*Buffer).TotalAllocationUnits = total;
    }

    (*Buffer).AvailableAllocationUnits = (*Buffer).TotalAllocationUnits;

    *Length -= core::mem::size_of::<ntkrnl_sys::ddk::_FILE_FS_SIZE_INFORMATION>() as ULONG;
    STATUS_SUCCESS
}

unsafe fn RawQueryFsDeviceInfo(
    Vcb: PVCB,
    Buffer: PFILE_FS_DEVICE_INFORMATION,
    Length: *mut ULONG,
) -> NTSTATUS {
    if *Length < core::mem::size_of::<ntkrnl_sys::ddk::_FILE_FS_DEVICE_INFORMATION>() as ULONG {
        return STATUS_BUFFER_OVERFLOW;
    }

    RtlZeroMemory(
        Buffer as PVOID,
        core::mem::size_of::<ntkrnl_sys::ddk::_FILE_FS_DEVICE_INFORMATION>(),
    );
    (*Buffer).DeviceType = FILE_DEVICE_DISK;
    (*Buffer).Characteristics = (*(*Vcb).TargetDeviceObject).Characteristics;

    *Length -= core::mem::size_of::<ntkrnl_sys::ddk::_FILE_FS_DEVICE_INFORMATION>() as ULONG;
    STATUS_SUCCESS
}

unsafe fn RawQueryFsAttributeInfo(
    Vcb: PVCB,
    Buffer: PFILE_FS_ATTRIBUTE_INFORMATION,
    Length: *mut ULONG,
) -> NTSTATUS {
    static SZ_RAW_FS_NAME: [u16; 4] = [b'R' as u16, b'A' as u16, b'W' as u16, 0];
    let ReturnLength: ULONG;

    ReturnLength = core::mem::offset_of!(
        ntkrnl_sys::ddk::_FILE_FS_ATTRIBUTE_INFORMATION,
        FileSystemName
    ) as ULONG
        + (SZ_RAW_FS_NAME.len() * core::mem::size_of::<u16>()) as ULONG;
    if *Length < ReturnLength {
        return STATUS_BUFFER_OVERFLOW;
    }

    (*Buffer).FileSystemAttributes = 0;
    (*Buffer).MaximumComponentNameLength = 0;
    (*Buffer).FileSystemNameLength = 6;
    core::ptr::copy_nonoverlapping(
        SZ_RAW_FS_NAME.as_ptr() as *const u8,
        core::ptr::addr_of_mut!((*Buffer).FileSystemName) as *mut u8,
        SZ_RAW_FS_NAME.len() * core::mem::size_of::<u16>(),
    );

    *Length -= ReturnLength;
    STATUS_SUCCESS
}

unsafe fn RawQueryVolumeInformation(
    Vcb: PVCB,
    Irp: PIRP,
    IoStackLocation: PIO_STACK_LOCATION,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut Length: ULONG;
    let Buffer: PVOID;

    Length = (*IoStackLocation).Parameters.QueryVolume.Length;
    Buffer = (*Irp).AssociatedIrp.SystemBuffer;

    match (*IoStackLocation).Parameters.QueryVolume.FsInformationClass {
        FileFsVolumeInformation => {
            Status = RawQueryFsVolumeInfo(Vcb, Buffer as PFILE_FS_VOLUME_INFORMATION, &mut Length);
        }
        FileFsSizeInformation => {
            Status = RawQueryFsSizeInfo(Vcb, Buffer as PFILE_FS_SIZE_INFORMATION, &mut Length);
        }
        FileFsDeviceInformation => {
            Status = RawQueryFsDeviceInfo(Vcb, Buffer as PFILE_FS_DEVICE_INFORMATION, &mut Length);
        }
        FileFsAttributeInformation => {
            Status = RawQueryFsAttributeInfo(
                Vcb,
                Buffer as PFILE_FS_ATTRIBUTE_INFORMATION,
                &mut Length,
            );
        }
        _ => {
            Status = STATUS_INVALID_PARAMETER;
        }
    }

    (*Irp).IoStatus.Information = ((*IoStackLocation).Parameters.QueryVolume.Length - Length) as ULONG_PTR;
    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    IoCompleteRequest(Irp, IO_DISK_INCREMENT);
    Status
}

unsafe fn RawCleanup(Vcb: PVCB, Irp: PIRP, IoStackLocation: PIO_STACK_LOCATION) -> NTSTATUS {
    let Status: NTSTATUS;

    Status = KeWaitForSingleObject(
        core::ptr::addr_of_mut!((*Vcb).Mutex) as PVOID,
        _KWAIT_REASON_Executive,
        0i8,
        0u8,
        core::ptr::null_mut(),
    );

    IoRemoveShareAccess(
        (*IoStackLocation).FileObject,
        core::ptr::addr_of_mut!((*Vcb).ShareAccess),
    );

    if FlagOn((*Vcb).VcbState, VCB_STATE_DISMOUNTED) {
        RawCheckForDismount(Vcb, 0u8);
    }

    KeReleaseMutex(core::ptr::addr_of_mut!((*Vcb).Mutex), 0u8);
    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    IoCompleteRequest(Irp, IO_DISK_INCREMENT);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn RawDispatch(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    let DeviceObject = opaque_ptr(DeviceObject);
    let Irp = opaque_ptr(Irp);
    let VolumeDeviceObject = DeviceObject as PVOLUME_DEVICE_OBJECT;
    let mut Status = STATUS_INVALID_DEVICE_REQUEST;
    let IoStackLocation: PIO_STACK_LOCATION;
    let Vcb: PVCB;

    IoStackLocation = IoGetCurrentIrpStackLocation(Irp);

    if ((*DeviceObject).Size == core::mem::size_of::<_DEVICE_OBJECT>() as USHORT)
        && !(((*IoStackLocation).MajorFunction == IRP_MJ_FILE_SYSTEM_CONTROL)
            && ((*IoStackLocation).MinorFunction == IRP_MN_MOUNT_VOLUME))
    {
        if ((*IoStackLocation).MajorFunction == IRP_MJ_CREATE)
            || ((*IoStackLocation).MajorFunction == IRP_MJ_CLEANUP)
            || ((*IoStackLocation).MajorFunction == IRP_MJ_CLOSE)
        {
            Status = STATUS_SUCCESS;
        } else {
            Status = STATUS_INVALID_DEVICE_REQUEST;
        }

        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        IoCompleteRequest(Irp, IO_DISK_INCREMENT);
        return Status;
    }

    FsRtlEnterFileSystem();
    Vcb = core::ptr::addr_of_mut!((*VolumeDeviceObject).Vcb);

    match (*IoStackLocation).MajorFunction {
        IRP_MJ_CLEANUP => {
            Status = RawCleanup(Vcb, Irp, IoStackLocation);
        }
        IRP_MJ_CLOSE => {
            Status = RawClose(Vcb, Irp, IoStackLocation);
        }
        IRP_MJ_CREATE => {
            Status = RawCreate(Vcb, Irp, IoStackLocation);
        }
        IRP_MJ_FILE_SYSTEM_CONTROL => {
            Status = RawFileSystemControl(Vcb, Irp, IoStackLocation);
        }
        IRP_MJ_READ | IRP_MJ_WRITE | IRP_MJ_DEVICE_CONTROL => {
            Status = RawReadWriteDeviceControl(Vcb, Irp, IoStackLocation);
        }
        IRP_MJ_QUERY_INFORMATION => {
            Status = RawQueryInformation(Vcb, Irp, IoStackLocation);
        }
        IRP_MJ_SET_INFORMATION => {
            Status = RawSetInformation(Vcb, Irp, IoStackLocation);
        }
        IRP_MJ_QUERY_VOLUME_INFORMATION => {
            Status = RawQueryVolumeInformation(Vcb, Irp, IoStackLocation);
        }
        _ => {
            KeBugCheck(FILE_SYSTEM);
        }
    }

    FsRtlExitFileSystem();
    Status
}

#[no_mangle]
pub unsafe extern "system" fn RawShutdown(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    let DeviceObject = opaque_ptr(DeviceObject);
    let Irp = opaque_ptr(Irp);
    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    IoCompleteRequest(Irp, IO_DISK_INCREMENT);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn RawUnload(DriverObject: PDRIVER_OBJECT) {}
#[no_mangle]
pub unsafe extern "system" fn RawFsDriverEntry(
    DriverObject: PDRIVER_OBJECT,
    RegistryPath: PUNICODE_STRING,
) -> NTSTATUS {
    let DriverObject = opaque_ptr(DriverObject);
    let RegistryPath = opaque_ptr(RegistryPath);
    let mut Status: NTSTATUS;
    let mut DeviceName: ntkrnl_sys::ddk::_UNICODE_STRING = core::mem::zeroed();

    RtlInitUnicodeString(
        core::ptr::addr_of_mut!(DeviceName),
        [
            b'\\' as u16,
            b'D' as u16,
            b'e' as u16,
            b'v' as u16,
            b'i' as u16,
            b'c' as u16,
            b'e' as u16,
            b'\\' as u16,
            b'R' as u16,
            b'a' as u16,
            b'w' as u16,
            b'D' as u16,
            b'i' as u16,
            b's' as u16,
            b'k' as u16,
            0,
        ]
        .as_ptr(),
    );
    Status = IoCreateDevice(
        DriverObject,
        0,
        core::ptr::addr_of_mut!(DeviceName),
        FILE_DEVICE_DISK_FILE_SYSTEM,
        0,
        0u8,
        core::ptr::addr_of_mut!(RawDiskDeviceObject),
    );
    if Status < 0 {
        return Status;
    }

    RtlInitUnicodeString(
        core::ptr::addr_of_mut!(DeviceName),
        [
            b'\\' as u16,
            b'D' as u16,
            b'e' as u16,
            b'v' as u16,
            b'i' as u16,
            b'c' as u16,
            b'e' as u16,
            b'\\' as u16,
            b'R' as u16,
            b'a' as u16,
            b'w' as u16,
            b'C' as u16,
            b'd' as u16,
            b'R' as u16,
            b'o' as u16,
            b'm' as u16,
            0,
        ]
        .as_ptr(),
    );
    Status = IoCreateDevice(
        DriverObject,
        0,
        core::ptr::addr_of_mut!(DeviceName),
        FILE_DEVICE_CD_ROM_FILE_SYSTEM,
        0,
        0u8,
        core::ptr::addr_of_mut!(RawCdromDeviceObject),
    );
    if Status < 0 {
        IoDeleteDevice(RawDiskDeviceObject);
        return Status;
    }

    RtlInitUnicodeString(
        core::ptr::addr_of_mut!(DeviceName),
        [
            b'\\' as u16,
            b'D' as u16,
            b'e' as u16,
            b'v' as u16,
            b'i' as u16,
            b'c' as u16,
            b'e' as u16,
            b'\\' as u16,
            b'R' as u16,
            b'a' as u16,
            b'w' as u16,
            b'T' as u16,
            b'a' as u16,
            b'p' as u16,
            b'e' as u16,
            0,
        ]
        .as_ptr(),
    );
    Status = IoCreateDevice(
        DriverObject,
        0,
        core::ptr::addr_of_mut!(DeviceName),
        FILE_DEVICE_TAPE_FILE_SYSTEM,
        0,
        0u8,
        core::ptr::addr_of_mut!(RawTapeDeviceObject),
    );
    if Status < 0 {
        IoDeleteDevice(RawDiskDeviceObject);
        IoDeleteDevice(RawCdromDeviceObject);
        return Status;
    }

    (*RawDiskDeviceObject).Flags |= DO_DIRECT_IO;
    (*RawCdromDeviceObject).Flags |= DO_DIRECT_IO;
    (*RawTapeDeviceObject).Flags |= DO_DIRECT_IO;

    (*DriverObject).MajorFunction[IRP_MJ_CREATE as usize] = core::mem::transmute(RawDispatch as usize);
    (*DriverObject).MajorFunction[IRP_MJ_CLEANUP as usize] = core::mem::transmute(RawDispatch as usize);
    (*DriverObject).MajorFunction[IRP_MJ_CLOSE as usize] = core::mem::transmute(RawDispatch as usize);
    (*DriverObject).MajorFunction[IRP_MJ_READ as usize] = core::mem::transmute(RawDispatch as usize);
    (*DriverObject).MajorFunction[IRP_MJ_WRITE as usize] = core::mem::transmute(RawDispatch as usize);
    (*DriverObject).MajorFunction[IRP_MJ_QUERY_INFORMATION as usize] = core::mem::transmute(RawDispatch as usize);
    (*DriverObject).MajorFunction[IRP_MJ_SET_INFORMATION as usize] = core::mem::transmute(RawDispatch as usize);
    (*DriverObject).MajorFunction[IRP_MJ_QUERY_VOLUME_INFORMATION as usize] = core::mem::transmute(RawDispatch as usize);
    (*DriverObject).MajorFunction[IRP_MJ_FILE_SYSTEM_CONTROL as usize] = core::mem::transmute(RawDispatch as usize);
    (*DriverObject).MajorFunction[IRP_MJ_DEVICE_CONTROL as usize] = core::mem::transmute(RawDispatch as usize);

    (*DriverObject).MajorFunction[IRP_MJ_SHUTDOWN as usize] = core::mem::transmute(RawShutdown as usize);
    (*DriverObject).DriverUnload = core::mem::transmute(RawUnload as usize);

    IoRegisterFileSystem(RawDiskDeviceObject);
    IoRegisterFileSystem(RawCdromDeviceObject);
    IoRegisterFileSystem(RawTapeDeviceObject);

    STATUS_SUCCESS
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
// * PROJECT:         ReactOS Kernel
// * LICENSE:         GPL - See COPYING in the top level directory
// * FILE:            ntoskrnl/io/iomgr/rawfs.c
// * PURPOSE:         Raw File System Driver
// * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
// */
//
// /* INCLUDES *****************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* TYPES *******************************************************************/
//
// typedef struct _VCB
// {
//     USHORT NodeTypeCode;
//     USHORT NodeByteSize;
//     PDEVICE_OBJECT TargetDeviceObject;
//     PVPB Vpb;
//     PVPB LocalVpb;
//     ULONG VcbState;
//     KMUTEX Mutex;
//     CLONG OpenCount;
//     SHARE_ACCESS ShareAccess;
//     ULONG BytesPerSector;
//     LARGE_INTEGER SectorsOnDisk;
// } VCB, *PVCB;
//
// typedef struct _VOLUME_DEVICE_OBJECT
// {
//     DEVICE_OBJECT DeviceObject;
//     VCB Vcb;
// } VOLUME_DEVICE_OBJECT, *PVOLUME_DEVICE_OBJECT;
//
// #define VCB_STATE_LOCKED     0x00000001
// #define VCB_STATE_DISMOUNTED 0x00000002
//
// /* GLOBALS *******************************************************************/
//
// PDEVICE_OBJECT RawDiskDeviceObject, RawCdromDeviceObject, RawTapeDeviceObject;
//
// /* FUNCTIONS *****************************************************************/
//
// NTSTATUS
// NTAPI
// RawInitializeVcb(IN OUT PVCB Vcb,
//                  IN PDEVICE_OBJECT TargetDeviceObject,
//                  IN PVPB Vpb)
// {
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     PAGED_CODE();
//
//     DPRINT("RawInitializeVcb(%p, %p, %p)\n", Vcb, TargetDeviceObject, Vpb);
//
//     /* Clear it */
//     RtlZeroMemory(Vcb, sizeof(VCB));
//
//     /* Associate to system objects */
//     Vcb->TargetDeviceObject = TargetDeviceObject;
//     Vcb->Vpb = Vpb;
//
//     /* Initialize the lock */
//     KeInitializeMutex(&Vcb->Mutex, 0);
//
//     Vcb->LocalVpb = ExAllocatePoolWithTag(NonPagedPool, sizeof(VPB), ' waR');
//     if (Vcb->LocalVpb == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     return Status;
// }
//
// BOOLEAN
// NTAPI
// RawCheckForDismount(IN PVCB Vcb,
//                     IN BOOLEAN CreateOperation)
// {
//     KIRQL OldIrql;
//     PVPB Vpb;
//     BOOLEAN Delete;
//
//     DPRINT("RawCheckForDismount(%p, %lu)\n", Vcb, CreateOperation);
//
//     ASSERT(KeReadStateMutant(&Vcb->Mutex) == 0);
//
//     /* Lock VPB */
//     IoAcquireVpbSpinLock(&OldIrql);
//
//     /* Reference it and check if a create is being done */
//     Vpb = Vcb->Vpb;
//     if (Vcb->Vpb->ReferenceCount != CreateOperation)
//     {
//         /* Copy the VPB to our local own to prepare later dismount */
//         if (Vcb->LocalVpb != NULL)
//         {
//             RtlZeroMemory(Vcb->LocalVpb, sizeof(VPB));
//             Vcb->LocalVpb->Type = IO_TYPE_VPB;
//             Vcb->LocalVpb->Size = sizeof(VPB);
//             Vcb->LocalVpb->RealDevice = Vcb->Vpb->RealDevice;
//             Vcb->LocalVpb->DeviceObject = NULL;
//             Vcb->LocalVpb->Flags = Vcb->Vpb->Flags & VPB_REMOVE_PENDING;
//             Vcb->Vpb->RealDevice->Vpb = Vcb->LocalVpb;
//             Vcb->LocalVpb = NULL;
//             Vcb->Vpb->Flags |= VPB_PERSISTENT;
//         }
//
//         /* Don't do anything */
//         Delete = FALSE;
//     }
//     else
//     {
//         /* Otherwise, delete the volume */
//         Delete = TRUE;
//
//         /* Check if it has a VPB and unmount it */
//         if (Vpb->RealDevice->Vpb == Vpb)
//         {
//             Vpb->DeviceObject = NULL;
//             Vpb->Flags &= ~VPB_MOUNTED;
//         }
//     }
//
//     /* Release the VPB lock */
//     IoReleaseVpbSpinLock(OldIrql);
//
//     /* If we were to delete, delete the volume */
//     if (Delete)
//     {
//         /* Release our Vcb lock to be able delete us */
//         KeReleaseMutex(&Vcb->Mutex, FALSE);
//
//         /* If we have a local VPB, we'll have to delete it
//          * but we won't dismount us - something went bad before
//          */
//         if (Vcb->LocalVpb)
//         {
//             ExFreePool(Vcb->LocalVpb);
//         }
//         /* Otherwise, delete any of the available VPB if its reference count is zero */
//         else if (Vcb->Vpb->ReferenceCount == 0)
//         {
//             ExFreePool(Vcb->Vpb);
//         }
//
//         /* Dismount our device if possible */
//         ObDereferenceObject(Vcb->TargetDeviceObject);
//         IoDeleteDevice((PDEVICE_OBJECT)CONTAINING_RECORD(Vcb,
//                                                          VOLUME_DEVICE_OBJECT,
//                                                          Vcb));
//     }
//
//     return Delete;
// }
//
// NTSTATUS
// NTAPI
// RawCompletionRoutine(IN PDEVICE_OBJECT DeviceObject,
//                      IN PIRP Irp,
//                      IN PVOID Context)
// {
//     PIO_STACK_LOCATION IoStackLocation = IoGetCurrentIrpStackLocation(Irp);
//
//     DPRINT("RawCompletionRoutine(%p, %p, %p)\n", DeviceObject, Irp, Context);
//
//     /* Check if this was a valid sync R/W request */
//     if (((IoStackLocation->MajorFunction == IRP_MJ_READ) ||
//          (IoStackLocation->MajorFunction == IRP_MJ_WRITE)) &&
//         ((IoStackLocation->FileObject)) &&
//          (FlagOn(IoStackLocation->FileObject->Flags, FO_SYNCHRONOUS_IO)) &&
//          (NT_SUCCESS(Irp->IoStatus.Status)))
//     {
//         /* Update byte offset */
//         IoStackLocation->FileObject->CurrentByteOffset.QuadPart +=
//             Irp->IoStatus.Information;
//     }
//
//     /* Mark the IRP Pending if it was */
//     if (Irp->PendingReturned) IoMarkIrpPending(Irp);
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// RawClose(IN PVCB Vcb,
//          IN PIRP Irp,
//          IN PIO_STACK_LOCATION IoStackLocation)
// {
//     NTSTATUS Status;
//     PAGED_CODE();
//
//     DPRINT("RawClose(%p, %p, %p)\n", Vcb, Irp, IoStackLocation);
//
//     /* If its a stream, not much to do */
//     if (IoStackLocation->FileObject->Flags & FO_STREAM_FILE)
//     {
//         Irp->IoStatus.Status = STATUS_SUCCESS;
//         IoCompleteRequest(Irp, IO_DISK_INCREMENT);
//         return STATUS_SUCCESS;
//     }
//
//     /* Make sure we can clean up */
//     Status = KeWaitForSingleObject(&Vcb->Mutex,
//                                    Executive,
//                                    KernelMode,
//                                    FALSE,
//                                    NULL);
//     ASSERT(NT_SUCCESS(Status));
//
//     /* Decrease the open count and check if this is a dismount */
//     Vcb->OpenCount--;
//     if (Vcb->OpenCount != 0 || !RawCheckForDismount(Vcb, FALSE))
//     {
//         KeReleaseMutex(&Vcb->Mutex, FALSE);
//     }
//
//     /* Complete the request */
//     Irp->IoStatus.Status = STATUS_SUCCESS;
//     IoCompleteRequest(Irp, IO_DISK_INCREMENT);
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// RawCreate(IN PVCB Vcb,
//           IN PIRP Irp,
//           IN PIO_STACK_LOCATION IoStackLocation)
// {
//     NTSTATUS Status;
//     USHORT ShareAccess;
//     ACCESS_MASK DesiredAccess;
//     BOOLEAN Deleted = FALSE;
//     PAGED_CODE();
//
//     DPRINT("RawCreate(%p, %p, %p)\n", Vcb, Irp, IoStackLocation);
//
//     /* Make sure we can clean up */
//     Status = KeWaitForSingleObject(&Vcb->Mutex,
//                                    Executive,
//                                    KernelMode,
//                                    FALSE,
//                                    NULL);
//     ASSERT(NT_SUCCESS(Status));
//
//     /* Check if this is a valid non-directory file open */
//     if ((!(IoStackLocation->FileObject) ||
//          !(IoStackLocation->FileObject->FileName.Length)) &&
//         ((IoStackLocation->Parameters.Create.Options >> 24) == FILE_OPEN) &&
//          (!(IoStackLocation->Parameters.Create.Options & FILE_DIRECTORY_FILE)))
//     {
//         /* Make sure the VCB isn't locked */
//         if (Vcb->VcbState & VCB_STATE_LOCKED)
//         {
//             /* Refuse the operation */
//             Status = STATUS_ACCESS_DENIED;
//             Irp->IoStatus.Information = 0;
//         }
//         else if (Vcb->VcbState & VCB_STATE_DISMOUNTED)
//         {
//             /* Refuse the operation */
//             Status = STATUS_VOLUME_DISMOUNTED;
//             Irp->IoStatus.Information = 0;
//         }
//         else
//         {
//             /* Setup share access */
//             ShareAccess = IoStackLocation->Parameters.Create.ShareAccess;
//             DesiredAccess = IoStackLocation->Parameters.Create.
//                             SecurityContext->DesiredAccess;
//
//             /* Check if this VCB was already opened */
//             if (Vcb->OpenCount > 0)
//             {
//                 /* Try to see if we have access to it */
//                 Status = IoCheckShareAccess(DesiredAccess,
//                                             ShareAccess,
//                                             IoStackLocation->FileObject,
//                                             &Vcb->ShareAccess,
//                                             TRUE);
//                 if (!NT_SUCCESS(Status)) Irp->IoStatus.Information = 0;
//             }
//
//             /* Make sure we have access */
//             if (NT_SUCCESS(Status))
//             {
//                 /* Check if this is the first open */
//                 if (!Vcb->OpenCount)
//                 {
//                     /* Set the share access */
//                     IoSetShareAccess(DesiredAccess,
//                                      ShareAccess,
//                                      IoStackLocation->FileObject,
//                                      &Vcb->ShareAccess);
//                 }
//
//                 /* Increase the open count and set the VPB */
//                 Vcb->OpenCount++;
//                 IoStackLocation->FileObject->Vpb = Vcb->Vpb;
//
//                 /* Set IRP status and disable intermediate buffering */
//                 Status = STATUS_SUCCESS;
//                 Irp->IoStatus.Information = FILE_OPENED;
//                 IoStackLocation->FileObject->Flags |=
//                     FO_NO_INTERMEDIATE_BUFFERING;
//             }
//         }
//     }
//     else
//     {
//         /* Invalid create request */
//         Status = STATUS_INVALID_PARAMETER;
//         Irp->IoStatus.Information = 0;
//     }
//
//     /* Check if the request failed */
//     if (!NT_SUCCESS(Status) && !Vcb->OpenCount)
//     {
//         /* Check if we can dismount the device */
//         Deleted = RawCheckForDismount(Vcb, TRUE);
//     }
//
//     /* In case of deletion, the mutex is already released */
//     if (!Deleted)
//     {
//         KeReleaseMutex(&Vcb->Mutex, FALSE);
//     }
//
//     /* Complete the request */
//     Irp->IoStatus.Status = Status;
//     IoCompleteRequest(Irp, IO_DISK_INCREMENT);
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// RawReadWriteDeviceControl(IN PVCB Vcb,
//                           IN PIRP Irp,
//                           IN PIO_STACK_LOCATION IoStackLocation)
// {
//     NTSTATUS Status;
//     PAGED_CODE();
//
//     DPRINT("RawReadWriteDeviceControl(%p, %p, %p)\n", Vcb, Irp, IoStackLocation);
//
//     /* Don't do anything if the request was 0 bytes */
//     if (((IoStackLocation->MajorFunction == IRP_MJ_READ) ||
//          (IoStackLocation->MajorFunction == IRP_MJ_WRITE)) &&
//         !(IoStackLocation->Parameters.Read.Length))
//     {
//         /* Complete it */
//         Irp->IoStatus.Status = STATUS_SUCCESS;
//         IoCompleteRequest(Irp, IO_DISK_INCREMENT);
//         return STATUS_SUCCESS;
//     }
//
//     /* Copy the IRP stack location */
//     IoCopyCurrentIrpStackLocationToNext(Irp);
//
//     /* Disable verifies */
//     IoGetNextIrpStackLocation(Irp)->Flags |= SL_OVERRIDE_VERIFY_VOLUME;
//
//     /* Setup a completion routine */
//     IoSetCompletionRoutine(Irp,
//                            RawCompletionRoutine,
//                            NULL,
//                            TRUE,
//                            TRUE,
//                            TRUE);
//
//     /* Call the next driver and exit */
//     Status = IoCallDriver(Vcb->TargetDeviceObject, Irp);
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// RawMountVolume(IN PIO_STACK_LOCATION IoStackLocation)
// {
//     NTSTATUS Status;
//     PDEVICE_OBJECT DeviceObject;
//     PVOLUME_DEVICE_OBJECT Volume;
//     PFILE_OBJECT FileObject = NULL;
//     PAGED_CODE();
//
//     DPRINT("RawMountVolume(%p)\n", IoStackLocation);
//
//     /* Remember our owner */
//     DeviceObject = IoStackLocation->Parameters.MountVolume.DeviceObject;
//
//     /* Create the volume */
//     Status = IoCreateDevice(RawDiskDeviceObject->DriverObject,
//                             sizeof(VOLUME_DEVICE_OBJECT) -
//                             sizeof(DEVICE_OBJECT),
//                             NULL,
//                             FILE_DEVICE_DISK_FILE_SYSTEM,
//                             0,
//                             FALSE,
//                             (PDEVICE_OBJECT*)&Volume);
//     if (!NT_SUCCESS(Status)) return Status;
//
//     /* Use highest alignment requirement */
//     Volume->DeviceObject.AlignmentRequirement = max(DeviceObject->
//                                                     AlignmentRequirement,
//                                                     Volume->DeviceObject.
//                                                     AlignmentRequirement);
//
//     /* Setup the VCB */
//     Status = RawInitializeVcb(&Volume->Vcb,
//                               IoStackLocation->Parameters.MountVolume.DeviceObject,
//                               IoStackLocation->Parameters.MountVolume.Vpb);
//     if (!NT_SUCCESS(Status))
//     {
//         IoDeleteDevice((PDEVICE_OBJECT)Volume);
//         return Status;
//     }
//
//     /* Set dummy label and serial number */
//     Volume->Vcb.Vpb->SerialNumber = 0xFFFFFFFF;
//     Volume->Vcb.Vpb->VolumeLabelLength = 0;
//
//     /* Setup the DO */
//     Volume->Vcb.Vpb->DeviceObject = &Volume->DeviceObject;
//     Volume->DeviceObject.StackSize = DeviceObject->StackSize + 1;
//     Volume->DeviceObject.SectorSize = DeviceObject->SectorSize;
//     Volume->DeviceObject.Flags |= DO_DIRECT_IO;
//     Volume->DeviceObject.Flags &= ~DO_DEVICE_INITIALIZING;
//
//     /* Try to get associated FO (for notification) */
//     _SEH2_TRY
//     {
//         FileObject = IoCreateStreamFileObjectLite(NULL,
//                                                   &(Volume->DeviceObject));
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         /* Get the exception code */
//         Status = _SEH2_GetExceptionCode();
//     }
//     _SEH2_END;
//
//     /* If failed, delete devive */
//     if (!NT_SUCCESS(Status))
//     {
//         IoDeleteDevice((PDEVICE_OBJECT)Volume);
//         return Status;
//     }
//
//     /* Increment OpenCount by two to avoid dismount when RawClose() will be called on ObDereferenceObject() */
//     Volume->Vcb.OpenCount += 2;
//     /* Notify for successful mount */
//     FsRtlNotifyVolumeEvent(FileObject, FSRTL_VOLUME_MOUNT);
//     /* It's not opened anymore, decrease the reference count to 0 to make FileObject being released */
//     ObDereferenceObject(FileObject);
//     Volume->Vcb.OpenCount -= 2;
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// RawUserFsCtrl(IN PIO_STACK_LOCATION IoStackLocation,
//               IN PVCB Vcb)
// {
//     NTSTATUS Status;
//     PAGED_CODE();
//
//     DPRINT("RawUserFsCtrl(%p, %p)\n", IoStackLocation, Vcb);
//
//     /* Lock the device */
//     Status = KeWaitForSingleObject(&Vcb->Mutex,
//                                    Executive,
//                                    KernelMode,
//                                    FALSE,
//                                    NULL);
//     ASSERT(NT_SUCCESS(Status));
//
//     /* Check what kind of request this is */
//     switch (IoStackLocation->Parameters.FileSystemControl.FsControlCode)
//     {
//         /* Oplock requests */
//         case FSCTL_REQUEST_OPLOCK_LEVEL_1:
//         case FSCTL_REQUEST_OPLOCK_LEVEL_2:
//         case FSCTL_OPLOCK_BREAK_ACKNOWLEDGE:
//         case FSCTL_OPLOCK_BREAK_NOTIFY:
//
//             /* We don't handle them */
//             Status = STATUS_NOT_IMPLEMENTED;
//             break;
//
//         /* Lock request */
//         case FSCTL_LOCK_VOLUME:
//
//             /* Make sure we're not locked, and that we're alone */
//             if (!(Vcb->VcbState & VCB_STATE_LOCKED) && (Vcb->OpenCount == 1))
//             {
//                 /* Lock the VCB */
//                 Vcb->VcbState |= VCB_STATE_LOCKED;
//                 Status = STATUS_SUCCESS;
//             }
//             else
//             {
//                 /* Otherwise, we can't do this */
//                 Status = STATUS_ACCESS_DENIED;
//             }
//             break;
//
//         /* Unlock request */
//         case FSCTL_UNLOCK_VOLUME:
//
//             /* Make sure we're locked */
//             if (!(Vcb->VcbState & VCB_STATE_LOCKED))
//             {
//                 /* Let caller know we're not */
//                 Status = STATUS_NOT_LOCKED;
//             }
//             else
//             {
//                 /* Unlock the VCB */
//                 Vcb->VcbState &= ~VCB_STATE_LOCKED;
//                 Status = STATUS_SUCCESS;
//             }
//             break;
//
//         /* Dismount request */
//         case FSCTL_DISMOUNT_VOLUME:
//
//             /* Make sure we're locked */
//             if (Vcb->VcbState & VCB_STATE_LOCKED)
//             {
//                 /* Do nothing, just return success */
//                 Status = STATUS_SUCCESS;
//             }
//             else
//             {
//                 /* We can't dismount, device not locked */
//                 Status = STATUS_ACCESS_DENIED;
//             }
//             break;
//
//         /* Unknown request */
//         default:
//
//             /* Fail */
//             Status = STATUS_INVALID_PARAMETER;
//             break;
//     }
//
//     /* Unlock device */
//     KeReleaseMutex(&Vcb->Mutex, FALSE);
//
//     /* In case of status change, notify */
//     switch (IoStackLocation->Parameters.FileSystemControl.FsControlCode)
//     {
//         case FSCTL_LOCK_VOLUME:
//             FsRtlNotifyVolumeEvent(IoStackLocation->FileObject, (NT_SUCCESS(Status) ? FSRTL_VOLUME_LOCK : FSRTL_VOLUME_LOCK_FAILED));
//             break;
//         case FSCTL_UNLOCK_VOLUME:
//             if (NT_SUCCESS(Status))
//             {
//                 FsRtlNotifyVolumeEvent(IoStackLocation->FileObject, FSRTL_VOLUME_UNLOCK);
//             }
//             break;
//         case FSCTL_DISMOUNT_VOLUME:
//             FsRtlNotifyVolumeEvent(IoStackLocation->FileObject, (NT_SUCCESS(Status) ? FSRTL_VOLUME_DISMOUNT : FSRTL_VOLUME_DISMOUNT_FAILED));
//             break;
//     }
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// RawFileSystemControl(IN PVCB Vcb,
//                      IN PIRP Irp,
//                      IN PIO_STACK_LOCATION IoStackLocation)
// {
//     NTSTATUS Status;
//     PAGED_CODE();
//
//     DPRINT("RawFileSystemControl(%p, %p, %p)\n", Vcb, Irp, IoStackLocation);
//
//     /* Check the kinds of FSCTLs that we support */
//     switch (IoStackLocation->MinorFunction)
//     {
//         /* User-mode request */
//         case IRP_MN_USER_FS_REQUEST:
//
//             /* Handle it */
//             Status = RawUserFsCtrl(IoStackLocation, Vcb);
//             break;
//
//         /* Mount request */
//         case IRP_MN_MOUNT_VOLUME:
//
//             /* Mount the volume */
//             Status = RawMountVolume(IoStackLocation);
//             break;
//
//         case IRP_MN_VERIFY_VOLUME:
//
//             /* Lock the device */
//             Status = KeWaitForSingleObject(&Vcb->Mutex,
//                                            Executive,
//                                            KernelMode,
//                                            FALSE,
//                                            NULL);
//             ASSERT(NT_SUCCESS(Status));
//
//             /* We don't do verifies */
//             Status = STATUS_WRONG_VOLUME;
//             Vcb->Vpb->RealDevice->Flags &= ~DO_VERIFY_VOLUME;
//
//             /* Check if we should delete the device */
//             if (Vcb->OpenCount != 0 || !RawCheckForDismount(Vcb, FALSE))
//             {
//                 /* In case of deletion, the mutex is already released */
//                 KeReleaseMutex(&Vcb->Mutex, FALSE);
//             }
//
//             /* We're done */
//             break;
//
//         /* Invalid request */
//         default:
//
//             /* Fail it */
//             Status = STATUS_INVALID_DEVICE_REQUEST;
//             break;
//     }
//
//     /* Complete the request */
//     Irp->IoStatus.Status = Status;
//     IoCompleteRequest(Irp, IO_DISK_INCREMENT);
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// RawQueryInformation(IN PVCB Vcb,
//                     IN PIRP Irp,
//                     IN PIO_STACK_LOCATION IoStackLocation)
// {
//     NTSTATUS Status = STATUS_INVALID_DEVICE_REQUEST;
//     PULONG Length;
//     PFILE_POSITION_INFORMATION Buffer;
//     PAGED_CODE();
//
//     DPRINT("RawQueryInformation(%p, %p, %p)\n", Vcb, Irp, IoStackLocation);
//
//     /* Get information from the IRP */
//     Length = &IoStackLocation->Parameters.QueryFile.Length;
//     Buffer = Irp->AssociatedIrp.SystemBuffer;
//
//     /* We only handle this request */
//     if (IoStackLocation->Parameters.QueryFile.FileInformationClass ==
//         FilePositionInformation)
//     {
//         /* Validate buffer size */
//         if (*Length < sizeof(FILE_POSITION_INFORMATION))
//         {
//             /* Invalid, fail */
//             Irp->IoStatus.Information = 0;
//             Status = STATUS_BUFFER_OVERFLOW;
//         }
//         else
//         {
//             /* Get offset and update length */
//             Buffer->CurrentByteOffset = IoStackLocation->FileObject->
//                                         CurrentByteOffset;
//             *Length -= sizeof(FILE_POSITION_INFORMATION);
//
//             /* Set IRP Status information */
//             Irp->IoStatus.Information = sizeof(FILE_POSITION_INFORMATION);
//             Status = STATUS_SUCCESS;
//         }
//     }
//
//     /* Complete it */
//     Irp->IoStatus.Status = Status;
//     IoCompleteRequest(Irp, IO_DISK_INCREMENT);
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// RawSetInformation(IN PVCB Vcb,
//                   IN PIRP Irp,
//                   IN PIO_STACK_LOCATION IoStackLocation)
// {
//     NTSTATUS Status = STATUS_INVALID_DEVICE_REQUEST;
//     PFILE_POSITION_INFORMATION Buffer;
//     PDEVICE_OBJECT DeviceObject;
//     PAGED_CODE();
//
//     DPRINT("RawSetInformation(%p, %p, %p)\n", Vcb, Irp, IoStackLocation);
//
//     /* Get information from the IRP */
//     Buffer = Irp->AssociatedIrp.SystemBuffer;
//
//     /* We only handle this request */
//     if (IoStackLocation->Parameters.QueryFile.FileInformationClass ==
//         FilePositionInformation)
//     {
//         /* Get the DO */
//         DeviceObject = IoGetRelatedDeviceObject(IoStackLocation->FileObject);
//
//         /* Make sure the offset is aligned */
//         if ((Buffer->CurrentByteOffset.LowPart &
//             DeviceObject->AlignmentRequirement))
//         {
//             /* It's not, fail */
//             Status = STATUS_INVALID_PARAMETER;
//         }
//         else
//         {
//             /* Otherwise, set offset */
//             IoStackLocation->FileObject->CurrentByteOffset = Buffer->
//                                                              CurrentByteOffset;
//
//             /* Set IRP Status information */
//             Status = STATUS_SUCCESS;
//         }
//     }
//
//     /* Complete it */
//     Irp->IoStatus.Status = Status;
//     IoCompleteRequest(Irp, IO_DISK_INCREMENT);
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// RawQueryFsVolumeInfo(IN PVCB Vcb,
//                      IN PFILE_FS_VOLUME_INFORMATION Buffer,
//                      IN OUT PULONG Length)
// {
//     PAGED_CODE();
//
//     DPRINT("RawQueryFsVolumeInfo(%p, %p, %p)\n", Vcb, Buffer, Length);
//
//     /* Clear the buffer and stub it out */
//     RtlZeroMemory(Buffer, sizeof(FILE_FS_VOLUME_INFORMATION));
//     Buffer->VolumeSerialNumber = Vcb->Vpb->SerialNumber;
//     Buffer->SupportsObjects = FALSE;
//     Buffer->VolumeLabelLength = 0;
//
//     /* Return length and success */
//     *Length -= FIELD_OFFSET(FILE_FS_VOLUME_INFORMATION, VolumeLabel[0]);
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// RawQueryFsSizeInfo(IN PVCB Vcb,
//                    IN PFILE_FS_SIZE_INFORMATION Buffer,
//                    IN OUT PULONG Length)
// {
//     PIRP Irp;
//     KEVENT Event;
//     NTSTATUS Status;
//     IO_STATUS_BLOCK IoStatusBlock;
//     PDEVICE_OBJECT RealDevice;
//     DISK_GEOMETRY DiskGeometry;
//     PARTITION_INFORMATION PartitionInformation;
//     BOOLEAN DiskHasPartitions;
//     PAGED_CODE();
//
//     DPRINT("RawQueryFsSizeInfo(%p, %p, %p)\n", Vcb, Buffer, Length);
//
//     /* Validate the buffer */
//     if (*Length < sizeof(FILE_FS_SIZE_INFORMATION))
//     {
//         /* Fail */
//         return STATUS_BUFFER_OVERFLOW;
//     }
//
//     /* Clear the buffer, initialize the event and set the DO */
//     RtlZeroMemory(Buffer, sizeof(FILE_FS_SIZE_INFORMATION));
//     KeInitializeEvent(&Event, NotificationEvent, FALSE);
//     RealDevice = Vcb->Vpb->RealDevice;
//
//     /* Build query IRP */
//     Irp = IoBuildDeviceIoControlRequest(IOCTL_DISK_GET_DRIVE_GEOMETRY,
//                                         RealDevice,
//                                         NULL,
//                                         0,
//                                         &DiskGeometry,
//                                         sizeof(DISK_GEOMETRY),
//                                         FALSE,
//                                         &Event,
//                                         &IoStatusBlock);
//     if (!Irp) return STATUS_INSUFFICIENT_RESOURCES;
//
//     /* Call driver and check if we're pending */
//     Status = IoCallDriver(RealDevice, Irp);
//     if (Status == STATUS_PENDING)
//     {
//         /* Wait on driver to finish */
//         KeWaitForSingleObject(&Event,
//                               Executive,
//                               KernelMode,
//                               FALSE,
//                               NULL);
//         Status = IoStatusBlock.Status;
//     }
//
//     /* Fail if we couldn't get CHS data */
//     if (!NT_SUCCESS(Status))
//     {
//         *Length = 0;
//         return Status;
//     }
//
//     /* Check if this is a floppy */
//     if (FlagOn(RealDevice->Characteristics, FILE_FLOPPY_DISKETTE))
//     {
//         /* Floppies don't have partitions */
//         DiskHasPartitions = FALSE;
//     }
//     else
//     {
//         /* Setup query IRP */
//         KeClearEvent(&Event);
//         Irp = IoBuildDeviceIoControlRequest(IOCTL_DISK_GET_PARTITION_INFO,
//                                             RealDevice,
//                                             NULL,
//                                             0,
//                                             &PartitionInformation,
//                                             sizeof(PARTITION_INFORMATION),
//                                             FALSE,
//                                             &Event,
//                                             &IoStatusBlock);
//         if (!Irp) return STATUS_INSUFFICIENT_RESOURCES;
//
//         /* Call driver and check if we're pending */
//         Status = IoCallDriver(RealDevice, Irp);
//         if (Status == STATUS_PENDING)
//         {
//             /* Wait on driver to finish */
//             KeWaitForSingleObject(&Event,
//                                   Executive,
//                                   KernelMode,
//                                   FALSE,
//                                   NULL);
//             Status = IoStatusBlock.Status;
//         }
//
//         /* If this was an invalid request, then the disk is not partitioned */
//         if (Status == STATUS_INVALID_DEVICE_REQUEST)
//         {
//             DiskHasPartitions = FALSE;
//         }
//         else
//         {
//             /* Otherwise, it must be */
//             ASSERT(NT_SUCCESS(Status));
//             DiskHasPartitions = TRUE;
//         }
//     }
//
//     /* Set sector data */
//     Buffer->BytesPerSector = DiskGeometry.BytesPerSector;
//     Buffer->SectorsPerAllocationUnit = 1;
//
//     /* Calculate allocation units */
//     if (DiskHasPartitions)
//     {
//         /* Use partition data */
//         Buffer->TotalAllocationUnits =
//             RtlExtendedLargeIntegerDivide(PartitionInformation.PartitionLength,
//                                           DiskGeometry.BytesPerSector,
//                                           NULL);
//     }
//     else
//     {
//         /* Use CHS */
//         Buffer->TotalAllocationUnits =
//             RtlExtendedIntegerMultiply(DiskGeometry.Cylinders,
//                                        DiskGeometry.TracksPerCylinder *
//                                        DiskGeometry.SectorsPerTrack);
//     }
//
//     /* Set available units */
//     Buffer->AvailableAllocationUnits = Buffer->TotalAllocationUnits;
//
//     /* Return length and success */
//     *Length -= sizeof(FILE_FS_SIZE_INFORMATION);
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// RawQueryFsDeviceInfo(IN PVCB Vcb,
//                      IN PFILE_FS_DEVICE_INFORMATION Buffer,
//                      IN OUT PULONG Length)
// {
//     PAGED_CODE();
//
//     DPRINT("RawQueryFsDeviceInfo(%p, %p, %p)\n", Vcb, Buffer, Length);
//
//     /* Validate buffer */
//     if (*Length < sizeof(FILE_FS_DEVICE_INFORMATION))
//     {
//         /* Fail */
//         return STATUS_BUFFER_OVERFLOW;
//     }
//
//     /* Clear buffer and write information */
//     RtlZeroMemory(Buffer, sizeof(FILE_FS_DEVICE_INFORMATION));
//     Buffer->DeviceType = FILE_DEVICE_DISK;
//     Buffer->Characteristics = Vcb->TargetDeviceObject->Characteristics;
//
//     /* Return length and success */
//     *Length -= sizeof(FILE_FS_DEVICE_INFORMATION);
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// RawQueryFsAttributeInfo(IN PVCB Vcb,
//                         IN PFILE_FS_ATTRIBUTE_INFORMATION Buffer,
//                         IN OUT PULONG Length)
// {
//     const WCHAR szRawFSName[] = L"RAW";
//     ULONG ReturnLength;
//     PAGED_CODE();
//
//     DPRINT("RawQueryFsAttributeInfo(%p, %p, %p)\n", Vcb, Buffer, Length);
//
//     /* Check if the buffer is large enough for our name ("RAW") */
//     ReturnLength = FIELD_OFFSET(FILE_FS_ATTRIBUTE_INFORMATION,
//                                 FileSystemName[sizeof(szRawFSName) / sizeof(szRawFSName[0])]);
//     if (*Length < ReturnLength) return STATUS_BUFFER_OVERFLOW;
//
//     /* Output the data */
//     Buffer->FileSystemAttributes = 0;
//     Buffer->MaximumComponentNameLength = 0;
//     Buffer->FileSystemNameLength = 6;
//     RtlCopyMemory(&Buffer->FileSystemName[0], szRawFSName, sizeof(szRawFSName));
//
//     /* Return length and success */
//     *Length -= ReturnLength;
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// RawQueryVolumeInformation(IN PVCB Vcb,
//                           IN PIRP Irp,
//                           IN PIO_STACK_LOCATION IoStackLocation)
// {
//     NTSTATUS Status;
//     ULONG Length;
//     PVOID Buffer;
//     PAGED_CODE();
//
//     DPRINT("RawQueryVolumeInformation(%p, %p, %p)\n", Vcb, Irp, IoStackLocation);
//
//     /* Get IRP Data */
//     Length = IoStackLocation->Parameters.QueryVolume.Length;
//     Buffer = Irp->AssociatedIrp.SystemBuffer;
//
//     /* Check the kind of request */
//     switch (IoStackLocation->Parameters.QueryVolume.FsInformationClass)
//     {
//         /* Volume information request */
//         case FileFsVolumeInformation:
//
//             Status = RawQueryFsVolumeInfo(Vcb, Buffer, &Length);
//             break;
//
//         /* File system size invormation */
//         case FileFsSizeInformation:
//
//             Status = RawQueryFsSizeInfo(Vcb, Buffer, &Length);
//             break;
//
//         /* Device information */
//         case FileFsDeviceInformation:
//
//             Status = RawQueryFsDeviceInfo(Vcb, Buffer, &Length);
//             break;
//
//         /* Attribute information */
//         case FileFsAttributeInformation:
//
//             Status = RawQueryFsAttributeInfo(Vcb, Buffer, &Length);
//             break;
//
//         /* Invalid request */
//         default:
//
//             /* Fail it */
//             Status = STATUS_INVALID_PARAMETER;
//             break;
//     }
//
//     /* Set status and complete the request */
//     Irp->IoStatus.Information = IoStackLocation->
//                                 Parameters.QueryVolume.Length - Length;
//     Irp->IoStatus.Status = Status;
//     IoCompleteRequest(Irp, IO_DISK_INCREMENT);
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// RawCleanup(IN PVCB Vcb,
//            IN PIRP Irp,
//            IN PIO_STACK_LOCATION IoStackLocation)
// {
//     NTSTATUS Status;
//     PAGED_CODE();
//
//     DPRINT("RawCleanup(%p, %p, %p)\n", Vcb, Irp, IoStackLocation);
//
//     /* Make sure we can clean up */
//     Status = KeWaitForSingleObject(&Vcb->Mutex,
//                                    Executive,
//                                    KernelMode,
//                                    FALSE,
//                                    NULL);
//     ASSERT(NT_SUCCESS(Status));
//
//     /* Remove shared access */
//     IoRemoveShareAccess(IoStackLocation->FileObject, &Vcb->ShareAccess);
//
//     /* Check if we're to dismount */
//     if (Vcb->VcbState & VCB_STATE_DISMOUNTED)
//     {
//         ASSERT(Vcb->OpenCount == 1);
//         RawCheckForDismount(Vcb, FALSE);
//     }
//
//     KeReleaseMutex(&Vcb->Mutex, FALSE);
//     Irp->IoStatus.Status = STATUS_SUCCESS;
//     IoCompleteRequest(Irp, IO_DISK_INCREMENT);
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// RawDispatch(IN PDEVICE_OBJECT DeviceObject,
//             IN PIRP Irp)
// {
//     PVOLUME_DEVICE_OBJECT VolumeDeviceObject = (PVOLUME_DEVICE_OBJECT)DeviceObject;
//     NTSTATUS Status = STATUS_INVALID_DEVICE_REQUEST;
//     PIO_STACK_LOCATION IoStackLocation;
//     PVCB Vcb;
//     PAGED_CODE();
//
//     DPRINT("RawDispatch(%p, %p)\n", DeviceObject, Irp);
//
//     /* Get the stack location */
//     IoStackLocation = IoGetCurrentIrpStackLocation(Irp);
//
//     /* Differentiate between Volume DO and FS DO */
//     if ((DeviceObject->Size == sizeof(DEVICE_OBJECT)) &&
//         !((IoStackLocation->MajorFunction == IRP_MJ_FILE_SYSTEM_CONTROL) &&
//           (IoStackLocation->MinorFunction == IRP_MN_MOUNT_VOLUME)))
//     {
//         /* This is an FS DO. Stub out the common calls */
//         if ((IoStackLocation->MajorFunction == IRP_MJ_CREATE) ||
//             (IoStackLocation->MajorFunction == IRP_MJ_CLEANUP) ||
//             (IoStackLocation->MajorFunction == IRP_MJ_CLOSE))
//         {
//             /* Return success for them */
//             Status = STATUS_SUCCESS;
//         }
//         else
//         {
//             /* Anything else, we don't support */
//             Status = STATUS_INVALID_DEVICE_REQUEST;
//         }
//
//         /* Complete the request */
//         Irp->IoStatus.Status = Status;
//         IoCompleteRequest(Irp, IO_DISK_INCREMENT);
//         return Status;
//     }
//
//     /* Otherwise, get our VCB and start handling the IRP */
//     FsRtlEnterFileSystem();
//     Vcb = &VolumeDeviceObject->Vcb;
//
//     /* Check what kind of IRP this is */
//     switch (IoStackLocation->MajorFunction)
//     {
//         /* Cleanup request */
//         case IRP_MJ_CLEANUP:
//
//             Status = RawCleanup(Vcb, Irp, IoStackLocation);
//             break;
//
//         /* Close request */
//         case IRP_MJ_CLOSE:
//
//             Status = RawClose(Vcb, Irp, IoStackLocation);
//             break;
//
//         /* Create request */
//         case IRP_MJ_CREATE:
//
//             Status = RawCreate(Vcb, Irp, IoStackLocation);
//             break;
//
//         /* FSCTL request */
//         case IRP_MJ_FILE_SYSTEM_CONTROL:
//
//             Status = RawFileSystemControl(Vcb, Irp, IoStackLocation);
//             break;
//
//         /* R/W or IOCTL request */
//         case IRP_MJ_READ:
//         case IRP_MJ_WRITE:
//         case IRP_MJ_DEVICE_CONTROL:
//
//             Status = RawReadWriteDeviceControl(Vcb, Irp, IoStackLocation);
//             break;
//
//         /* Information query request */
//         case IRP_MJ_QUERY_INFORMATION:
//
//             Status = RawQueryInformation(Vcb, Irp, IoStackLocation);
//             break;
//
//         /* Information set request */
//         case IRP_MJ_SET_INFORMATION:
//
//             Status = RawSetInformation(Vcb, Irp, IoStackLocation);
//             break;
//
//         /* Volume information request */
//         case IRP_MJ_QUERY_VOLUME_INFORMATION:
//
//             Status = RawQueryVolumeInformation(Vcb, Irp, IoStackLocation);
//             break;
//
//         /* Unexpected request */
//         default:
//
//             /* Anything else is pretty bad */
//             KeBugCheck(FILE_SYSTEM);
//     }
//
//     /* Return the status */
//     FsRtlExitFileSystem();
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// RawShutdown(IN PDEVICE_OBJECT DeviceObject,
//             IN PIRP Irp)
// {
//     /* Unregister file systems */
// #if 0 // FIXME: This freezes ROS at shutdown. PnP Problem?
//     IoUnregisterFileSystem(RawDiskDeviceObject);
//     IoUnregisterFileSystem(RawCdromDeviceObject);
//     IoUnregisterFileSystem(RawTapeDeviceObject);
//
//     /* Delete the devices */
//     IoDeleteDevice(RawDiskDeviceObject);
//     IoDeleteDevice(RawCdromDeviceObject);
//     IoDeleteDevice(RawTapeDeviceObject);
// #endif
//
//     /* Complete the request */
//     Irp->IoStatus.Status = STATUS_SUCCESS;
//     IoCompleteRequest(Irp, IO_DISK_INCREMENT);
//     return STATUS_SUCCESS;
// }
//
// VOID
// NTAPI
// RawUnload(IN PDRIVER_OBJECT DriverObject)
// {
// #if 0 // FIXME: DriverUnload is never called
//     /* Dereference device objects */
//     ObDereferenceObject(RawDiskDeviceObject);
//     ObDereferenceObject(RawCdromDeviceObject);
//     ObDereferenceObject(RawTapeDeviceObject);
// #endif
// }
//
// CODE_SEG("INIT")
// NTSTATUS
// NTAPI
// RawFsDriverEntry(IN PDRIVER_OBJECT DriverObject,
//                  IN PUNICODE_STRING RegistryPath)
// {
//     NTSTATUS Status;
//     UNICODE_STRING DeviceName;
//
//     UNREFERENCED_PARAMETER(RegistryPath);
//
//     /* Create the raw disk device */
//     RtlInitUnicodeString(&DeviceName, L"\\Device\\RawDisk");
//     Status = IoCreateDevice(DriverObject,
//                             0,
//                             &DeviceName,
//                             FILE_DEVICE_DISK_FILE_SYSTEM,
//                             0,
//                             FALSE,
//                             &RawDiskDeviceObject);
//     if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//
//     /* Create the raw CDROM device */
//     RtlInitUnicodeString(&DeviceName, L"\\Device\\RawCdRom");
//     Status = IoCreateDevice(DriverObject,
//                             0,
//                             &DeviceName,
//                             FILE_DEVICE_CD_ROM_FILE_SYSTEM,
//                             0,
//                             FALSE,
//                             &RawCdromDeviceObject);
//     if (!NT_SUCCESS(Status))
//     {
//         IoDeleteDevice(RawDiskDeviceObject);
//         return Status;
//     }
//
//     /* Create the raw tape device */
//     RtlInitUnicodeString(&DeviceName, L"\\Device\\RawTape");
//     Status = IoCreateDevice(DriverObject,
//                             0,
//                             &DeviceName,
//                             FILE_DEVICE_TAPE_FILE_SYSTEM,
//                             0,
//                             FALSE,
//                             &RawTapeDeviceObject);
//     if (!NT_SUCCESS(Status))
//     {
//         IoDeleteDevice(RawDiskDeviceObject);
//         IoDeleteDevice(RawCdromDeviceObject);
//         return Status;
//     }
//
//     /* Set Direct I/O for all devices */
//     RawDiskDeviceObject->Flags |= DO_DIRECT_IO;
//     RawCdromDeviceObject->Flags |= DO_DIRECT_IO;
//     RawTapeDeviceObject->Flags |= DO_DIRECT_IO;
//
//     /* Set generic stubs */
//     DriverObject->MajorFunction[IRP_MJ_CREATE] =
//     DriverObject->MajorFunction[IRP_MJ_CLEANUP] =
//     DriverObject->MajorFunction[IRP_MJ_CLOSE] =
//     DriverObject->MajorFunction[IRP_MJ_READ] =
//     DriverObject->MajorFunction[IRP_MJ_WRITE] =
//     DriverObject->MajorFunction[IRP_MJ_QUERY_INFORMATION] =
//     DriverObject->MajorFunction[IRP_MJ_SET_INFORMATION] =
//     DriverObject->MajorFunction[IRP_MJ_QUERY_VOLUME_INFORMATION] =
//     DriverObject->MajorFunction[IRP_MJ_FILE_SYSTEM_CONTROL] =
//     DriverObject->MajorFunction[IRP_MJ_DEVICE_CONTROL] = RawDispatch;
//
//     /* Shutdown and unload */
//     DriverObject->MajorFunction[IRP_MJ_SHUTDOWN] = RawShutdown;
//     DriverObject->DriverUnload = RawUnload;
//
//     /* Register the file systems */
//     IoRegisterFileSystem(RawDiskDeviceObject);
//     IoRegisterFileSystem(RawCdromDeviceObject);
//     IoRegisterFileSystem(RawTapeDeviceObject);
//
// #if 0 // FIXME: DriverUnload is never called
//     /* Reference device objects */
//     ObReferenceObject(RawDiskDeviceObject);
//     ObReferenceObject(RawCdromDeviceObject);
//     ObReferenceObject(RawTapeDeviceObject);
// #endif
//     return STATUS_SUCCESS;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
