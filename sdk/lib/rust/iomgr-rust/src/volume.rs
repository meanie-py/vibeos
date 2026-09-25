use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::rtl::*;

use ntkrnl_sys::ddk::{
    _DEVICE_OBJECT, _ERESOURCE, _FS_CHANGE_NOTIFY_ENTRY, _KEVENT, _LIST_ENTRY,
    _IO_STATUS_BLOCK, _MOUNTMGR_VOLUME_PATHS, _OPEN_PACKET, _UNICODE_STRING, _VPB,
    PDEVICE_OBJECT, PFS_CHANGE_NOTIFY_ENTRY, PDRIVER_FS_NOTIFICATION,
    PDRIVER_OBJECT, PFILE_OBJECT, PIO_STACK_LOCATION, PIRP, PKIRQL,
    PMOUNTMGR_VOLUME_PATHS, PNTSTATUS, PVPB, PULONG,
    ExAcquireResourceExclusiveLite, ExAcquireResourceSharedLite,
    ExReleaseResourceLite, IoAllocateIrp, IoBuildDeviceIoControlRequest,
    IoBuildSynchronousFsdRequest, IoGetAttachedDevice,
    IoGetDeviceObjectPointer, IoGetNextIrpStackLocation,
    KeEnterCriticalRegion, KeLeaveCriticalRegion,
    KeInitializeEvent, KeWaitForSingleObject, ZwClose, ZwSetValueKey,
    PsGetCurrentThread, RtlInitUnicodeString, IofCallDriver,
    _EVENT_TYPE_NotificationEvent, _KWAIT_REASON_Executive,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

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


extern "system" {
    fn KeGetPreviousMode() -> i8;
    fn IopUnloadDevice(DeviceObject: PDEVICE_OBJECT);
    fn IopDereferenceDeviceObject(DeviceObject: PDEVICE_OBJECT, UnloadIfUnused: u8);
    fn IopOpenRegistryKeyEx( Handle: *mut HANDLE, ParentHandle: HANDLE, Name: *mut _UNICODE_STRING, DesiredAccess: u32, ) -> NTSTATUS;
    fn IopCreateRegistryKeyEx( Handle: *mut HANDLE, ParentHandle: HANDLE, Name: *mut _UNICODE_STRING, DesiredAccess: u32, CreateOptions: u32, Disposition: *mut u32, ) -> NTSTATUS;
    fn FsRtlIsTotalDeviceFailure(Status: NTSTATUS) -> u8;
    fn PoVolumeDevice(DeviceObject: PDEVICE_OBJECT);
    static mut ExpInitializationPhase: u32;
    static mut Arm64UartKernelBase: usize;
}
// The link_name target IoGetDevObjExtension_shim is defined extern "system" in
// inlines.rs — the decl MUST match the shim's ABI, not IoGetDevObjExtension's.
unsafe extern "system" {
    #[link_name = "IoGetDevObjExtension_shim"]
    fn IoGetDevObjExtension(DeviceObject: PDEVICE_OBJECT) -> *mut core::ffi::c_void;
}

#[inline(always)]
unsafe fn IoCallDriver(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    IofCallDriver(DeviceObject, Irp)
}

// C #define constants not captured by bindgen
const IRP_MJ_SHUTDOWN: u8 = 0x10;
const IRP_MJ_FILE_SYSTEM_CONTROL: u8 = 0x0D;
const IRP_MJ_DEVICE_CONTROL: u8 = 0x0E;
const IRP_MN_MOUNT_VOLUME: u8 = 0x01;
const IRP_MN_VERIFY_VOLUME: u8 = 0x02;
const IRP_MN_LOAD_FILE_SYSTEM: u8 = 0x03;
const IRP_MOUNT_COMPLETION: u32 = 0x00000002;
const IRP_SYNCHRONOUS_PAGING_IO: u32 = 0x00000040;
const SL_ALLOW_RAW_MOUNT: u8 = 0x01;
const FILE_READ_ATTRIBUTES: u32 = 0x00000080;
const FILE_SYNCHRONOUS_IO_ALERT: u32 = 0x00000010;
const IO_TYPE_VPB: i16 = 10;
const IO_NO_INCREMENT: u32 = 0;
const FILE_DEVICE_DISK: u32 = 0x00000007;
const FILE_DEVICE_VIRTUAL_DISK: u32 = 0x00000024;
const FILE_DEVICE_CD_ROM: u32 = 0x00000002;
const FILE_DEVICE_DISK_FILE_SYSTEM: u32 = 0x00000008;
const FILE_DEVICE_NETWORK_FILE_SYSTEM: u32 = 0x00000014;
const FILE_DEVICE_CD_ROM_FILE_SYSTEM: u32 = 0x00000003;
const FILE_DEVICE_TAPE_FILE_SYSTEM: u32 = 0x00000020;
const VPB_MOUNTED: u16 = 0x0001;
const VPB_LOCKED: u16 = 0x0002;
const VPB_PERSISTENT: u16 = 0x0004;
const VPB_REMOVE_PENDING: u16 = 0x0008;
const VPB_RAW_MOUNT: u16 = 0x0010;
const DO_VERIFY_VOLUME: u32 = 0x00000002;
const DO_SYSTEM_BOOT_PARTITION: u32 = 0x00000100;
const DO_LOW_PRIORITY_FILESYSTEM: u32 = 0x00010000;
const DO_DEVICE_INITIALIZING: u32 = 0x00000080;
const DOE_UNLOAD_PENDING: u32 = 0x1;
const DOE_DELETE_PENDING: u32 = 0x2;
const DOE_REMOVE_PENDING: u32 = 0x4;
const KEY_ALL_ACCESS: u32 = 0xF003F;
const REG_OPTION_NON_VOLATILE: u32 = 0;
const REG_SZ: u32 = 1;
const INACCESSIBLE_BOOT_DEVICE: u32 = 0x7B;
const LockQueueIoDatabaseLock: u32 = 10;
const LockQueueIoVpbLock: u32 = 9;
// IOCTL_MOUNTDEV_QUERY_DEVICE_NAME = CTL_CODE('M', 2, METHOD_BUFFERED, FILE_ANY_ACCESS)
const IOCTL_MOUNTDEV_QUERY_DEVICE_NAME: u32 = (0x4D << 16) | (0 << 14) | (2 << 2) | 0;
// IOCTL_MOUNTMGR_QUERY_DOS_VOLUME_PATH = CTL_CODE('m', 12, METHOD_BUFFERED, FILE_ANY_ACCESS)
const IOCTL_MOUNTMGR_QUERY_DOS_VOLUME_PATH: u32 = (0x6D << 16) | (0 << 14) | (12 << 2) | 0;

const STATUS_VERIFY_REQUIRED: NTSTATUS = 0x80000016u32 as i32;
const STATUS_USER_APC: NTSTATUS = 0x000000C0u32 as i32;
const STATUS_WRONG_VOLUME: NTSTATUS = 0xC0000012u32 as i32;
const STATUS_NO_MEDIA_IN_DEVICE: NTSTATUS = 0xC0000013u32 as i32;
const STATUS_UNRECOGNIZED_MEDIA: NTSTATUS = 0xC0000014u32 as i32;
const STATUS_DEVICE_ALREADY_ATTACHED: NTSTATUS = 0xC0000038u32 as i32;
const STATUS_DEVICE_NOT_READY: NTSTATUS = 0xC00000A3u32 as i32;
const STATUS_IO_TIMEOUT: NTSTATUS = 0xC00000B5u32 as i32;
const STATUS_DEVICE_DOES_NOT_EXIST: NTSTATUS = 0xC00000C0u32 as i32;
const STATUS_UNRECOGNIZED_VOLUME: NTSTATUS = 0xC000014Fu32 as i32;
const STATUS_FS_DRIVER_REQUIRED: NTSTATUS = 0xC000019Cu32 as i32;
const STATUS_INVALID_BUFFER_SIZE: NTSTATUS = 0xC0000206u32 as i32;

/// IoIsErrorUserInduced — C macro from iofuncs.h, implemented as Rust inline.
#[inline(always)]
fn IoIsErrorUserInduced(Status: NTSTATUS) -> u8 {
    ((Status == STATUS_DEVICE_NOT_READY)
        || (Status == STATUS_IO_TIMEOUT)
        || (Status == STATUS_MEDIA_WRITE_PROTECTED)
        || (Status == STATUS_NO_MEDIA_IN_DEVICE)
        || (Status == STATUS_VERIFY_REQUIRED)
        || (Status == STATUS_UNRECOGNIZED_MEDIA)
        || (Status == STATUS_WRONG_VOLUME)) as u8
}

type POBJECT = PVOID;

/// Local CONTAINING_RECORD macro supporting dotted field paths.
macro_rules! CONTAINING_RECORD {
    ($ptr:expr, $type:ty, $($field:tt).+) => {
        ($ptr as *const u8).sub(core::mem::offset_of!($type, $($field).+)) as *mut $type
    };
}

macro_rules! FIELD_OFFSET {
    ($type:ty, $($field:tt).+) => {
        core::mem::offset_of!($type, $($field).+)
    };
}

#[cfg(target_arch = "aarch64")]
macro_rules! ARM64_UART_TRACE {
    ($msg:expr) => {
        unsafe {
            let uart = Arm64UartKernelBase as *mut u32;
            if !uart.is_null() {
                for &byte in $msg {
                    *uart = byte as u32;
                }
            }
        }
    };
}

#[cfg(not(target_arch = "aarch64"))]
macro_rules! ARM64_UART_TRACE {
    ($msg:expr) => {};
}

#[no_mangle]
pub static mut IopDatabaseResource: _ERESOURCE = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut IopDiskFileSystemQueueHead: _LIST_ENTRY = _LIST_ENTRY {
    Flink: core::ptr::null_mut(),
    Blink: core::ptr::null_mut(),
};
#[no_mangle]
pub static mut IopNetworkFileSystemQueueHead: _LIST_ENTRY = _LIST_ENTRY {
    Flink: core::ptr::null_mut(),
    Blink: core::ptr::null_mut(),
};
#[no_mangle]
pub static mut IopCdRomFileSystemQueueHead: _LIST_ENTRY = _LIST_ENTRY {
    Flink: core::ptr::null_mut(),
    Blink: core::ptr::null_mut(),
};
#[no_mangle]
pub static mut IopTapeFileSystemQueueHead: _LIST_ENTRY = _LIST_ENTRY {
    Flink: core::ptr::null_mut(),
    Blink: core::ptr::null_mut(),
};
#[no_mangle]
pub static mut IopFsNotifyChangeQueueHead: _LIST_ENTRY = _LIST_ENTRY {
    Flink: core::ptr::null_mut(),
    Blink: core::ptr::null_mut(),
};
static mut IopFsRegistrationOps: ULONG = 0;

const TAG_VPB: u32 = 0x20425056; // nolint: const-contradiction (per-driver pool tag, each verified against its own C)
                                 // tag.h:94 ' BPV'
const TAG_FS_CHANGE_NOTIFY: u32 = 0x4E435346;
const TAG_DEV2DOS: u32 = 0x20643244;
const MOUNTMGR_DEVICE_NAME: &[u16] = &[
    0x5c, 0x44, 0x65, 0x76, 0x69, 0x63, 0x65, 0x5c, 0x4d, 0x6f, 0x75, 0x6e, 0x74, 0x50, 0x6f,
    0x69, 0x6e, 0x74, 0x4d, 0x61, 0x6e, 0x61, 0x67, 0x65, 0x72, 0x00,
];

unsafe fn IopDecrementDeviceObjectRef(DeviceObject: PDEVICE_OBJECT, UnloadIfUnused: u8) {
    let OldIrql = KeAcquireQueuedSpinLock(LockQueueIoDatabaseLock);

    (*DeviceObject).ReferenceCount -= 1;
    if (*DeviceObject).ReferenceCount > 0 {
        KeReleaseQueuedSpinLock(LockQueueIoDatabaseLock, OldIrql);
        return;
    }

    KeReleaseQueuedSpinLock(LockQueueIoDatabaseLock, OldIrql);

    let ext = IoGetDevObjExtension(DeviceObject) as *mut u32;
    if UnloadIfUnused != 0
        || (!ext.is_null()
            && (*ext.add(1) & (DOE_UNLOAD_PENDING | DOE_DELETE_PENDING | DOE_REMOVE_PENDING)) != 0)
    {
        IopUnloadDevice(DeviceObject);
    }
}

#[no_mangle]
pub unsafe extern "system" fn IopDecrementDeviceObjectHandleCount(DeviceObject: PDEVICE_OBJECT) {
    let DeviceObject = opaque_ptr(DeviceObject);
    IopDecrementDeviceObjectRef(DeviceObject, 0u8);
}

#[no_mangle]
pub unsafe extern "system" fn IopCheckVpbMounted(
    OpenPacket: *mut _OPEN_PACKET,
    DeviceObject: PDEVICE_OBJECT,
    RemainingName: *mut _UNICODE_STRING,
    Status: PNTSTATUS,
) -> PVPB {
    let OpenPacket = opaque_ptr(OpenPacket);
    let DeviceObject = opaque_ptr(DeviceObject);
    let RemainingName = opaque_ptr(RemainingName);
    let Status = opaque_ptr(Status);
    let mut OldIrql: KIRQL = 0;
    let mut Vpb: PVPB = core::ptr::null_mut();

    IoAcquireVpbSpinLock(&mut OldIrql);

    let Raw = (*RemainingName).Length == 0 && (*OpenPacket).RelatedFileObject.is_null();
    let Alertable = if (*OpenPacket).CreateOptions & FILE_SYNCHRONOUS_IO_ALERT != 0 {
        1u8
    } else {
        0u8
    };

    loop {
        if (*(*DeviceObject).Vpb).Flags & VPB_MOUNTED == 0 {
            IoReleaseVpbSpinLock(OldIrql);

            *Status = IopMountVolume(
                DeviceObject,
                if Raw { 1u8 } else { 0u8 },
                0u8,
                Alertable,
                &mut Vpb,
            );

            if *Status < 0
                || *Status == STATUS_USER_APC as i32
                || *Status == STATUS_ALERTED as i32
            {
                IopDereferenceDeviceObject(DeviceObject, 0u8);

                if *Status < 0 {
                    return core::ptr::null_mut();
                }

                *Status = STATUS_WRONG_VOLUME as i32;
                return core::ptr::null_mut();
            } else if !Vpb.is_null() {
                return Vpb;
            }

            IoAcquireVpbSpinLock(&mut OldIrql);
        } else {
            break;
        }
    }

    Vpb = (*DeviceObject).Vpb;
    if (*Vpb).Flags & VPB_LOCKED != 0 {
        *Status = STATUS_ACCESS_DENIED as i32;
        Vpb = core::ptr::null_mut();
    } else {
        (*Vpb).ReferenceCount += 1;
    }

    IoReleaseVpbSpinLock(OldIrql);
    Vpb
}

#[no_mangle]
pub unsafe extern "system" fn IopCreateVpb(DeviceObject: PDEVICE_OBJECT) -> NTSTATUS {
    let DeviceObject = opaque_ptr(DeviceObject);
    let Vpb = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<_VPB>(),
        TAG_VPB,
    ) as PVPB;
    if Vpb.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    core::ptr::write_bytes(Vpb as *mut u8, 0, core::mem::size_of::<_VPB>());

    (*Vpb).Type = IO_TYPE_VPB as i16;
    (*Vpb).Size = core::mem::size_of::<_VPB>() as i16;
    (*Vpb).RealDevice = DeviceObject;

    (*DeviceObject).Vpb = Vpb;
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn IopDereferenceVpbAndFree(Vpb: PVPB) {
    let Vpb = opaque_ptr(Vpb);
    let mut OldIrql: KIRQL = 0;

    IoAcquireVpbSpinLock(&mut OldIrql);
    (*Vpb).ReferenceCount -= 1;

    if (*Vpb).ReferenceCount == 0
        && (*(*Vpb).RealDevice).Vpb == Vpb
        && ((*Vpb).Flags & VPB_PERSISTENT) == 0
    {
        IoReleaseVpbSpinLock(OldIrql);
        ExFreePoolWithTag(Vpb as PVOID, TAG_VPB);
    } else {
        IoReleaseVpbSpinLock(OldIrql);
    }
}

#[no_mangle]
pub unsafe extern "system" fn IopReferenceVerifyVpb(
    DeviceObject: PDEVICE_OBJECT,
    FileSystemObject: *mut PDEVICE_OBJECT,
    Vpb: *mut PVPB,
) -> u8 {
    let DeviceObject = opaque_ptr(DeviceObject);
    let FileSystemObject = opaque_ptr(FileSystemObject);
    let Vpb = opaque_ptr(Vpb);
    let mut OldIrql: KIRQL = 0;
    let mut Result = 0u8;

    IoAcquireVpbSpinLock(&mut OldIrql);
    *Vpb = core::ptr::null_mut();
    *FileSystemObject = core::ptr::null_mut();

    let LocalVpb = (*DeviceObject).Vpb;
    if !LocalVpb.is_null() && ((*LocalVpb).Flags & VPB_MOUNTED) != 0 {
        *Vpb = LocalVpb;
        *FileSystemObject = (*LocalVpb).DeviceObject;
        (*LocalVpb).ReferenceCount += 1;
        Result = 1u8;
    }

    IoReleaseVpbSpinLock(OldIrql);
    Result
}

unsafe fn IopMountInitializeVpb(
    DeviceObject: PDEVICE_OBJECT,
    AttachedDeviceObject: PDEVICE_OBJECT,
    Raw: u8,
) -> PVPB {
    let mut OldIrql: KIRQL = 0;

    IoAcquireVpbSpinLock(&mut OldIrql);
    let Vpb = (*DeviceObject).Vpb;

    (*Vpb).Flags |= VPB_MOUNTED | if Raw != 0 { VPB_RAW_MOUNT } else { 0 };

    (*(*Vpb).DeviceObject).StackSize = (*AttachedDeviceObject).StackSize;
    (*(*Vpb).DeviceObject).StackSize += 1;

    let ext = IoGetDevObjExtension((*Vpb).DeviceObject);
    if !ext.is_null() {
        *(ext as *mut PVPB) = Vpb;
    }

    (*Vpb).ReferenceCount += 1;

    IoReleaseVpbSpinLock(OldIrql);
    Vpb
}

unsafe fn IopNotifyFileSystemChange(DeviceObject: PDEVICE_OBJECT, DriverActive: u8) {
    let mut ListEntry = IopFsNotifyChangeQueueHead.Flink;
    while ListEntry != core::ptr::addr_of_mut!(IopFsNotifyChangeQueueHead) {
        let ChangeEntry = CONTAINING_RECORD!(ListEntry, _FS_CHANGE_NOTIFY_ENTRY, FsChangeNotifyList);
        ((*ChangeEntry).FSDNotificationProc.unwrap())(DeviceObject, DriverActive);
        ListEntry = (*ListEntry).Flink;
    }
}

unsafe fn IopInterlockedIncrementUlong(Queue: u32, Ulong: PULONG) -> ULONG {
    let Irql = KeAcquireQueuedSpinLock(Queue);
    let OldValue = *Ulong;
    *Ulong += 1;
    KeReleaseQueuedSpinLock(Queue, Irql);
    OldValue
}

unsafe fn IopInterlockedDecrementUlong(Queue: u32, Ulong: PULONG) -> ULONG {
    let Irql = KeAcquireQueuedSpinLock(Queue);
    let OldValue = *Ulong;
    *Ulong -= 1;
    KeReleaseQueuedSpinLock(Queue, Irql);
    OldValue
}

#[no_mangle]
pub unsafe extern "system" fn IopShutdownBaseFileSystems(ListHead: PLIST_ENTRY) {
    let ListHead = opaque_ptr(ListHead);
    let mut Event: _KEVENT = core::mem::zeroed();
    KeInitializeEvent(&mut Event, _EVENT_TYPE_NotificationEvent, 0u8);

    let mut ListEntry = (*ListHead).Flink;
    while ListEntry != ListHead {
        let DeviceObject = CONTAINING_RECORD!(ListEntry, _DEVICE_OBJECT, Queue);
        ListEntry = (*ListEntry).Flink;

        let DeviceObject = IoGetAttachedDevice(DeviceObject);

        ObReferenceObject(DeviceObject as POBJECT);
        IopInterlockedIncrementUlong(
            LockQueueIoDatabaseLock,
            core::ptr::addr_of_mut!((*DeviceObject).ReferenceCount) as PULONG,
        );

        let mut StatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
        let Irp = IoBuildSynchronousFsdRequest(
            IRP_MJ_SHUTDOWN as u32,
            DeviceObject,
            core::ptr::null_mut(),
            0,
            core::ptr::null_mut(),
            &mut Event,
            &mut StatusBlock,
        );
        if !Irp.is_null() {
            let Status = IoCallDriver(DeviceObject, Irp);
            if Status == STATUS_PENDING as i32 {
                KeWaitForSingleObject(
                    &mut Event as *mut _ as PVOID,
                    _KWAIT_REASON_Executive,
                    0i8,
                    0u8,
                    core::ptr::null_mut(),
                );
            }
        }

        KeClearEvent(&mut Event as *mut _ as PVOID);

        IopDecrementDeviceObjectRef(DeviceObject, 0u8);
        ObDereferenceObject(DeviceObject as POBJECT);
    }
}

unsafe fn IopLoadFileSystemDriver(DeviceObject: PDEVICE_OBJECT) {
    let mut AttachedDeviceObject = DeviceObject;

    while !(*AttachedDeviceObject).AttachedDevice.is_null() {
        AttachedDeviceObject = (*AttachedDeviceObject).AttachedDevice;
    }

    let mut Event: _KEVENT = core::mem::zeroed();
    KeInitializeEvent(&mut Event, _EVENT_TYPE_NotificationEvent, 0u8);
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let Irp = IoBuildDeviceIoControlRequest(
        IRP_MJ_DEVICE_CONTROL as u32,
        AttachedDeviceObject,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
        0,
        0u8,
        &mut Event,
        &mut IoStatusBlock,
    );
    if !Irp.is_null() {
        let StackPtr = IoGetNextIrpStackLocation(Irp);
        (*StackPtr).MajorFunction = IRP_MJ_FILE_SYSTEM_CONTROL;
        (*StackPtr).MinorFunction = IRP_MN_LOAD_FILE_SYSTEM;

        let Status = IoCallDriver(AttachedDeviceObject, Irp);
        if Status == STATUS_PENDING as i32 {
            KeWaitForSingleObject(
                &mut Event as *mut _ as PVOID,
                _KWAIT_REASON_Executive,
                0i8,
                0u8,
                core::ptr::null_mut(),
            );
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn IopMountVolume(
    DeviceObject: PDEVICE_OBJECT,
    AllowRawMount: u8,
    DeviceIsLocked: u8,
    Alertable: u8,
    Vpb: *mut PVPB,
) -> NTSTATUS {
    let DeviceObject = opaque_ptr(DeviceObject);
    let Vpb = opaque_ptr(Vpb);
    let mut Event: _KEVENT = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut AttachedDeviceObject = DeviceObject;
    let mut LocalList: _LIST_ENTRY = core::mem::zeroed();

    #[cfg(target_arch = "aarch64")]
    {
        ARM64_UART_TRACE!(b"[MOUNT] IopMountVolume\r\n");
    }

    if DeviceIsLocked == 0 {
        ARM64_UART_TRACE!(b"[MOUNT] Acquiring DeviceLock\r\n");
        let Status = KeWaitForSingleObject(
            core::ptr::addr_of_mut!((*DeviceObject).DeviceLock) as PVOID,
            _KWAIT_REASON_Executive,
            KeGetPreviousMode(),
            Alertable,
            core::ptr::null_mut(),
        );
        if Status == STATUS_ALERTED as i32 || Status == STATUS_USER_APC as i32 {
            return Status;
        }
        ARM64_UART_TRACE!(b"[MOUNT] DeviceLock acquired\r\n");
    }

    KeEnterCriticalRegion();
    ExAcquireResourceSharedLite(core::ptr::addr_of_mut!(IopDatabaseResource), 1u8);

    let mut Status: NTSTATUS;

    if ((*(*DeviceObject).Vpb).Flags & (VPB_MOUNTED | VPB_REMOVE_PENDING)) == 0 {
        KeInitializeEvent(&mut Event, _EVENT_TYPE_NotificationEvent, 0u8);

        (*DeviceObject).Flags &= !DO_VERIFY_VOLUME;
        while !(*AttachedDeviceObject).AttachedDevice.is_null() {
            AttachedDeviceObject = (*AttachedDeviceObject).AttachedDevice;
        }

        ObReferenceObject(AttachedDeviceObject as POBJECT);

        let FsList = if (*DeviceObject).DeviceType == FILE_DEVICE_DISK
            || (*DeviceObject).DeviceType == FILE_DEVICE_VIRTUAL_DISK
        {
            core::ptr::addr_of_mut!(IopDiskFileSystemQueueHead)
        } else if (*DeviceObject).DeviceType == FILE_DEVICE_CD_ROM {
            core::ptr::addr_of_mut!(IopCdRomFileSystemQueueHead)
        } else {
            core::ptr::addr_of_mut!(IopTapeFileSystemQueueHead)
        };

        Status = STATUS_UNSUCCESSFUL as i32;
        let mut ListEntry = (*FsList).Flink;
        while ListEntry != FsList && Status < 0 {
            if AllowRawMount == 0
                && (*ListEntry).Flink == FsList
                && ListEntry != (*FsList).Flink
            {
                break;
            }

            if ((*(*DeviceObject).Vpb).Flags & VPB_RAW_MOUNT) != 0 && (*ListEntry).Flink != FsList
            {
                ListEntry = (*ListEntry).Flink;
                continue;
            }

            let FileSystemDeviceObject = CONTAINING_RECORD!(ListEntry, _DEVICE_OBJECT, Queue);
            let ParentFsDeviceObject = FileSystemDeviceObject;

            let mut FsStackOverhead = 1u32;
            let mut FsDeviceObject = FileSystemDeviceObject;
            while !(*FsDeviceObject).AttachedDevice.is_null() {
                FsDeviceObject = (*FsDeviceObject).AttachedDevice;
                FsStackOverhead += 1;
            }

            KeClearEvent(&mut Event as *mut _ as PVOID);

            let Irp = IoAllocateIrp(
                (*AttachedDeviceObject).StackSize + FsStackOverhead as i8,
                1u8,
            );
            if Irp.is_null() {
                Status = STATUS_INSUFFICIENT_RESOURCES as i32;
                break;
            }

            (*Irp).UserIosb = &mut IoStatusBlock;
            (*Irp).UserEvent = &mut Event;
            (*Irp).Tail.Overlay.Thread = PsGetCurrentThread();
            (*Irp).Flags = IRP_MOUNT_COMPLETION | IRP_SYNCHRONOUS_PAGING_IO;
            (*Irp).RequestorMode = 0i8;

            let StackPtr = IoGetNextIrpStackLocation(Irp);
            (*StackPtr).MajorFunction = IRP_MJ_FILE_SYSTEM_CONTROL;
            (*StackPtr).MinorFunction = IRP_MN_MOUNT_VOLUME;
            (*StackPtr).Flags = AllowRawMount;
            (*StackPtr).Parameters.MountVolume.Vpb = (*DeviceObject).Vpb;
            (*StackPtr).Parameters.MountVolume.DeviceObject = AttachedDeviceObject;

            let RegistrationOps = IopFsRegistrationOps;

            IopInterlockedIncrementUlong(
                LockQueueIoDatabaseLock,
                core::ptr::addr_of_mut!((*DeviceObject).ReferenceCount) as PULONG,
            );
            ExReleaseResourceLite(core::ptr::addr_of_mut!(IopDatabaseResource));

            ARM64_UART_TRACE!(b"[MOUNT] IoCallDriver(mount IRP)\r\n");
            Status = IoCallDriver(FsDeviceObject, Irp);
            if Status == STATUS_PENDING as i32 {
                ARM64_UART_TRACE!(b"[MOUNT] Waiting for mount completion...\r\n");
                KeWaitForSingleObject(
                    &mut Event as *mut _ as PVOID,
                    _KWAIT_REASON_Executive,
                    0i8,
                    0u8,
                    core::ptr::null_mut(),
                );
                Status = IoStatusBlock.__bindgen_anon_1.Status;
            }

            ExAcquireResourceSharedLite(core::ptr::addr_of_mut!(IopDatabaseResource), 1u8);
            IopInterlockedDecrementUlong(
                LockQueueIoDatabaseLock,
                core::ptr::addr_of_mut!((*DeviceObject).ReferenceCount) as PULONG,
            );

            if Status >= 0 {
                *Vpb = IopMountInitializeVpb(
                    DeviceObject,
                    AttachedDeviceObject,
                    if ((*(*DeviceObject).Vpb).Flags & VPB_RAW_MOUNT) != 0 {
                        1u8
                    } else {
                        0u8
                    },
                );
            } else {
                if IoIsErrorUserInduced(Status) != 0 && IoStatusBlock.Information == 1 {
                    break;
                }

                if RegistrationOps != IopFsRegistrationOps {
                    LocalList.Flink = (*FsList).Flink;
                    ListEntry = &mut LocalList;
                    Status = STATUS_UNRECOGNIZED_VOLUME as i32;
                }

                if Status == STATUS_FS_DRIVER_REQUIRED as i32 {
                    IopInterlockedIncrementUlong(
                        LockQueueIoDatabaseLock,
                        core::ptr::addr_of_mut!((*DeviceObject).ReferenceCount) as PULONG,
                    );
                    ExReleaseResourceLite(core::ptr::addr_of_mut!(IopDatabaseResource));

                    if DeviceIsLocked == 0 {
                        KeSetEvent(
                            core::ptr::addr_of_mut!((*DeviceObject).DeviceLock) as PVOID,
                            0,
                            0u8,
                        );
                    }

                    KeLeaveCriticalRegion();

                    IopLoadFileSystemDriver(ParentFsDeviceObject);

                    if DeviceIsLocked == 0 {
                        let Status2 = KeWaitForSingleObject(
                            core::ptr::addr_of_mut!((*DeviceObject).DeviceLock) as PVOID,
                            _KWAIT_REASON_Executive,
                            KeGetPreviousMode(),
                            Alertable,
                            core::ptr::null_mut(),
                        );
                        if Status2 == STATUS_ALERTED as i32 || Status2 == STATUS_USER_APC as i32 {
                            ObDereferenceObject(AttachedDeviceObject as POBJECT);
                            return Status2;
                        }
                    }

                    KeEnterCriticalRegion();
                    ExAcquireResourceSharedLite(core::ptr::addr_of_mut!(IopDatabaseResource), 1u8);

                    if ((*(*DeviceObject).Vpb).Flags & VPB_MOUNTED) != 0 {
                        Status = STATUS_SUCCESS;
                        break;
                    }

                    Status = STATUS_UNRECOGNIZED_VOLUME as i32;
                    LocalList.Flink = (*FsList).Flink;
                    ListEntry = &mut LocalList;
                }

                if AllowRawMount == 0
                    && Status != STATUS_UNRECOGNIZED_VOLUME as i32
                    && FsRtlIsTotalDeviceFailure(Status) != 0
                {
                    break;
                }
            }

            ListEntry = (*ListEntry).Flink;
        }

        if Status < 0 {
            ObDereferenceObject(AttachedDeviceObject as POBJECT);
        }
    } else if ((*(*DeviceObject).Vpb).Flags & VPB_REMOVE_PENDING) != 0 {
        Status = STATUS_DEVICE_DOES_NOT_EXIST as i32;
    } else {
        Status = STATUS_SUCCESS;
    }

    ExReleaseResourceLite(core::ptr::addr_of_mut!(IopDatabaseResource));
    KeLeaveCriticalRegion();

    if DeviceIsLocked == 0 {
        KeSetEvent(
            core::ptr::addr_of_mut!((*DeviceObject).DeviceLock) as PVOID,
            0,
            0u8,
        );
    }

    if Status < 0
        && ((*DeviceObject).Flags & DO_SYSTEM_BOOT_PARTITION) != 0
        && ExpInitializationPhase < 2
    {
        KeBugCheckEx(
            INACCESSIBLE_BOOT_DEVICE,
            DeviceObject as usize,
            Status as usize,
            0,
            0,
        );
    }

    Status
}

unsafe fn IopNotifyAlreadyRegisteredFileSystems(
    ListHead: PLIST_ENTRY,
    DriverNotificationRoutine: PDRIVER_FS_NOTIFICATION,
    SkipRawFs: u8,
) {
    let mut ListEntry = (*ListHead).Flink;
    while ListEntry != ListHead {
        if (*ListEntry).Flink == ListHead && SkipRawFs != 0 {
            return;
        }

        let DeviceObject = CONTAINING_RECORD!(ListEntry, _DEVICE_OBJECT, Queue);
        (DriverNotificationRoutine.unwrap())(DeviceObject, 1u8);

        ListEntry = (*ListEntry).Flink;
    }
}

#[no_mangle]
pub unsafe extern "system" fn IoEnumerateRegisteredFiltersList(
    DriverObjectList: *mut PDRIVER_OBJECT,
    DriverObjectListSize: ULONG,
    ActualNumberDriverObjects: PULONG,
) -> NTSTATUS {
    let DriverObjectList = opaque_ptr(DriverObjectList);
    let ActualNumberDriverObjects = opaque_ptr(ActualNumberDriverObjects);
    let mut Status = STATUS_SUCCESS;
    let mut ListSize = 0u32;
    let MaximumSize = DriverObjectListSize / core::mem::size_of::<PDRIVER_OBJECT>() as u32;

    KeEnterCriticalRegion();
    ExAcquireResourceExclusiveLite(core::ptr::addr_of_mut!(IopDatabaseResource), 1u8);

    let mut ListEntry = IopFsNotifyChangeQueueHead.Flink;
    while ListEntry != core::ptr::addr_of_mut!(IopFsNotifyChangeQueueHead) {
        let ChangeEntry = CONTAINING_RECORD!(ListEntry, _FS_CHANGE_NOTIFY_ENTRY, FsChangeNotifyList);

        if ListSize < MaximumSize {
            ObReferenceObject((*ChangeEntry).DriverObject as POBJECT);
            *DriverObjectList.add(ListSize as usize) = (*ChangeEntry).DriverObject;
        } else {
            Status = STATUS_BUFFER_TOO_SMALL;
        }

        ListSize += 1;
        ListEntry = (*ListEntry).Flink;
    }

    *ActualNumberDriverObjects = ListSize;

    ExReleaseResourceLite(core::ptr::addr_of_mut!(IopDatabaseResource));
    KeLeaveCriticalRegion();

    Status
}

#[no_mangle]
pub unsafe extern "system" fn IoVerifyVolume(
    DeviceObject: PDEVICE_OBJECT,
    AllowRawMount: u8,
) -> NTSTATUS {
    let DeviceObject = opaque_ptr(DeviceObject);
    let Status = KeWaitForSingleObject(
        core::ptr::addr_of_mut!((*DeviceObject).DeviceLock) as PVOID,
        _KWAIT_REASON_Executive,
        0i8,
        0u8,
        core::ptr::null_mut(),
    );

    let mut FileSystemDeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut Vpb: PVPB = core::ptr::null_mut();

    let mut Status = if IopReferenceVerifyVpb(DeviceObject, &mut FileSystemDeviceObject, &mut Vpb)
        != 0
    {
        let mut Event: _KEVENT = core::mem::zeroed();
        KeInitializeEvent(&mut Event, _EVENT_TYPE_NotificationEvent, 0u8);

        FileSystemDeviceObject = (*(*DeviceObject).Vpb).DeviceObject;
        while !(*FileSystemDeviceObject).AttachedDevice.is_null() {
            FileSystemDeviceObject = (*FileSystemDeviceObject).AttachedDevice;
        }

        let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
        let Irp = IoAllocateIrp((*FileSystemDeviceObject).StackSize, 0u8);
        if Irp.is_null() {
            IopDereferenceVpbAndFree(Vpb);
            STATUS_INSUFFICIENT_RESOURCES as i32
        } else {
            (*Irp).UserIosb = &mut IoStatusBlock;
            (*Irp).UserEvent = &mut Event;
            (*Irp).Tail.Overlay.Thread = PsGetCurrentThread();
            (*Irp).Flags = IRP_MOUNT_COMPLETION | IRP_SYNCHRONOUS_PAGING_IO;
            (*Irp).RequestorMode = 0i8;

            let StackPtr = IoGetNextIrpStackLocation(Irp);
            (*StackPtr).MajorFunction = IRP_MJ_FILE_SYSTEM_CONTROL;
            (*StackPtr).MinorFunction = IRP_MN_VERIFY_VOLUME;
            (*StackPtr).Flags = if AllowRawMount != 0 {
                SL_ALLOW_RAW_MOUNT
            } else {
                0
            };
            (*StackPtr).Parameters.VerifyVolume.Vpb = Vpb;
            (*StackPtr).Parameters.VerifyVolume.DeviceObject = (*(*DeviceObject).Vpb).DeviceObject;

            let mut Status = IoCallDriver(FileSystemDeviceObject, Irp);
            if Status == STATUS_PENDING as i32 {
                KeWaitForSingleObject(
                    &mut Event as *mut _ as PVOID,
                    _KWAIT_REASON_Executive,
                    0i8,
                    0u8,
                    core::ptr::null_mut(),
                );
                Status = IoStatusBlock.__bindgen_anon_1.Status;
            }

            IopDereferenceVpbAndFree(Vpb);
            Status
        }
    } else {
        STATUS_SUCCESS
    };

    if Status == STATUS_WRONG_VOLUME as i32 {
        let VpbStatus = IopCreateVpb(DeviceObject);
        if VpbStatus >= 0 {
            PoVolumeDevice(DeviceObject);

            let mut NewVpb: PVPB = core::ptr::null_mut();
            let VpbStatus = IopMountVolume(DeviceObject, AllowRawMount, 1u8, 0u8, &mut NewVpb);

            if !NewVpb.is_null() {
                IopInterlockedDecrementUlong(
                    LockQueueIoVpbLock,
                    core::ptr::addr_of_mut!((*NewVpb).ReferenceCount),
                );
            }
        }

        if VpbStatus < 0 {
            (*DeviceObject).Flags &= !DO_VERIFY_VOLUME;
        }
    }

    KeSetEvent(
        core::ptr::addr_of_mut!((*DeviceObject).DeviceLock) as PVOID,
        IO_NO_INCREMENT as i32,
        0u8,
    );
    Status
}

#[no_mangle]
pub unsafe extern "system" fn IoRegisterFileSystem(DeviceObject: PDEVICE_OBJECT) {
    let DeviceObject = opaque_ptr(DeviceObject);
    KeEnterCriticalRegion();
    ExAcquireResourceExclusiveLite(core::ptr::addr_of_mut!(IopDatabaseResource), 1u8);

    let FsList = if (*DeviceObject).DeviceType == FILE_DEVICE_DISK_FILE_SYSTEM {
        core::ptr::addr_of_mut!(IopDiskFileSystemQueueHead)
    } else if (*DeviceObject).DeviceType == FILE_DEVICE_NETWORK_FILE_SYSTEM {
        core::ptr::addr_of_mut!(IopNetworkFileSystemQueueHead)
    } else if (*DeviceObject).DeviceType == FILE_DEVICE_CD_ROM_FILE_SYSTEM {
        core::ptr::addr_of_mut!(IopCdRomFileSystemQueueHead)
    } else if (*DeviceObject).DeviceType == FILE_DEVICE_TAPE_FILE_SYSTEM {
        core::ptr::addr_of_mut!(IopTapeFileSystemQueueHead)
    } else {
        core::ptr::null_mut()
    };

    if !FsList.is_null() {
        if ((*DeviceObject).Flags & DO_LOW_PRIORITY_FILESYSTEM) != 0 {
            InsertTailList(
                (*FsList).Blink,
                core::ptr::addr_of_mut!((*DeviceObject).Queue.ListEntry),
            );
        } else {
            InsertHeadList(
                FsList,
                core::ptr::addr_of_mut!((*DeviceObject).Queue.ListEntry),
            );
        }
    }

    IopFsRegistrationOps += 1;

    (*DeviceObject).Flags &= !DO_DEVICE_INITIALIZING;

    IopNotifyFileSystemChange(DeviceObject, 1u8);

    ExReleaseResourceLite(core::ptr::addr_of_mut!(IopDatabaseResource));
    KeLeaveCriticalRegion();

    IopInterlockedIncrementUlong(
        LockQueueIoDatabaseLock,
        core::ptr::addr_of_mut!((*DeviceObject).ReferenceCount) as PULONG,
    );
}

#[no_mangle]
pub unsafe extern "system" fn IoUnregisterFileSystem(DeviceObject: PDEVICE_OBJECT) {
    let DeviceObject = opaque_ptr(DeviceObject);
    KeEnterCriticalRegion();
    ExAcquireResourceExclusiveLite(core::ptr::addr_of_mut!(IopDatabaseResource), 1u8);

    if !(*DeviceObject).Queue.ListEntry.Flink.is_null() {
        RemoveEntryList(core::ptr::addr_of_mut!((*DeviceObject).Queue.ListEntry));
    }

    IopNotifyFileSystemChange(DeviceObject, 0u8);

    IopFsRegistrationOps += 1;

    ExReleaseResourceLite(core::ptr::addr_of_mut!(IopDatabaseResource));
    KeLeaveCriticalRegion();

    IopInterlockedDecrementUlong(
        LockQueueIoDatabaseLock,
        core::ptr::addr_of_mut!((*DeviceObject).ReferenceCount) as PULONG,
    );
}

#[allow(unpredictable_function_pointer_comparisons)] // FNPTR-CMP(audited): intentional identity compare mirroring the C source; revisit -> core::ptr::fn_addr_eq (see docs/engineering/active-lanes/warnings-burndown-plan.md)
#[no_mangle]
pub unsafe extern "system" fn IoRegisterFsRegistrationChange(
    DriverObject: PDRIVER_OBJECT,
    DriverNotificationRoutine: PDRIVER_FS_NOTIFICATION,
) -> NTSTATUS {
    let DriverObject = opaque_ptr(DriverObject);
    KeEnterCriticalRegion();
    ExAcquireResourceExclusiveLite(core::ptr::addr_of_mut!(IopDatabaseResource), 1u8);

    if !IsListEmpty(core::ptr::addr_of_mut!(IopFsNotifyChangeQueueHead)) {
        let Entry = CONTAINING_RECORD!(
            IopFsNotifyChangeQueueHead.Blink,
            _FS_CHANGE_NOTIFY_ENTRY,
            FsChangeNotifyList
        );

        if (*Entry).DriverObject == DriverObject
            && (*Entry).FSDNotificationProc == DriverNotificationRoutine
        {
            ExReleaseResourceLite(core::ptr::addr_of_mut!(IopDatabaseResource));
            return STATUS_DEVICE_ALREADY_ATTACHED;
        }
    }

    let Entry = ExAllocatePoolWithTag(
        PagedPool,
        core::mem::size_of::<_FS_CHANGE_NOTIFY_ENTRY>(),
        TAG_FS_CHANGE_NOTIFY,
    ) as PFS_CHANGE_NOTIFY_ENTRY;
    if Entry.is_null() {
        ExReleaseResourceLite(core::ptr::addr_of_mut!(IopDatabaseResource));
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*Entry).DriverObject = DriverObject;
    (*Entry).FSDNotificationProc = DriverNotificationRoutine;

    InsertTailList(
        core::ptr::addr_of_mut!(IopFsNotifyChangeQueueHead),
        core::ptr::addr_of_mut!((*Entry).FsChangeNotifyList),
    );

    IopNotifyAlreadyRegisteredFileSystems(
        core::ptr::addr_of_mut!(IopNetworkFileSystemQueueHead),
        DriverNotificationRoutine,
        0u8,
    );
    IopNotifyAlreadyRegisteredFileSystems(
        core::ptr::addr_of_mut!(IopCdRomFileSystemQueueHead),
        DriverNotificationRoutine,
        1u8,
    );
    IopNotifyAlreadyRegisteredFileSystems(
        core::ptr::addr_of_mut!(IopDiskFileSystemQueueHead),
        DriverNotificationRoutine,
        1u8,
    );
    IopNotifyAlreadyRegisteredFileSystems(
        core::ptr::addr_of_mut!(IopTapeFileSystemQueueHead),
        DriverNotificationRoutine,
        1u8,
    );

    ExReleaseResourceLite(core::ptr::addr_of_mut!(IopDatabaseResource));
    KeLeaveCriticalRegion();

    ObReferenceObject(DriverObject as POBJECT);
    STATUS_SUCCESS
}

#[allow(unpredictable_function_pointer_comparisons)] // FNPTR-CMP(audited): intentional identity compare mirroring the C source; revisit -> core::ptr::fn_addr_eq (see docs/engineering/active-lanes/warnings-burndown-plan.md)
#[no_mangle]
pub unsafe extern "system" fn IoUnregisterFsRegistrationChange(
    DriverObject: PDRIVER_OBJECT,
    FSDNotificationProc: PDRIVER_FS_NOTIFICATION,
) {
    let DriverObject = opaque_ptr(DriverObject);
    KeEnterCriticalRegion();
    ExAcquireResourceExclusiveLite(core::ptr::addr_of_mut!(IopDatabaseResource), 1u8);

    let mut NextEntry = IopFsNotifyChangeQueueHead.Flink;
    while NextEntry != core::ptr::addr_of_mut!(IopFsNotifyChangeQueueHead) {
        let ChangeEntry = CONTAINING_RECORD!(NextEntry, _FS_CHANGE_NOTIFY_ENTRY, FsChangeNotifyList);

        if (*ChangeEntry).DriverObject == DriverObject
            && (*ChangeEntry).FSDNotificationProc == FSDNotificationProc
        {
            RemoveEntryList(core::ptr::addr_of_mut!((*ChangeEntry).FsChangeNotifyList));
            ExFreePoolWithTag(ChangeEntry as PVOID, TAG_FS_CHANGE_NOTIFY);
            break;
        }

        NextEntry = (*NextEntry).Flink;
    }

    ExReleaseResourceLite(core::ptr::addr_of_mut!(IopDatabaseResource));
    KeLeaveCriticalRegion();

    ObDereferenceObject(DriverObject as POBJECT);
}

#[no_mangle]
pub unsafe extern "system" fn IoAcquireVpbSpinLock(Irql: PKIRQL) {
    *Irql = KeAcquireQueuedSpinLock(LockQueueIoVpbLock);
}

#[no_mangle]
pub unsafe extern "system" fn IoReleaseVpbSpinLock(Irql: KIRQL) {
    KeReleaseQueuedSpinLock(LockQueueIoVpbLock, Irql);
}

#[no_mangle]
pub unsafe extern "system" fn IoSetSystemPartition(VolumeNameString: *mut _UNICODE_STRING) -> NTSTATUS {
    let VolumeNameString = opaque_ptr(VolumeNameString);
    let mut RootHandle: HANDLE = core::ptr::null_mut();
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut HKLMSystem = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut KeyString = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut Buffer = [0u16; 16];

    RtlInitUnicodeString(&mut HKLMSystem, w!("\\REGISTRY\\MACHINE\\SYSTEM").as_ptr() as *const u16);

    let mut Status = IopOpenRegistryKeyEx(&mut RootHandle, core::ptr::null_mut(), &mut HKLMSystem, KEY_ALL_ACCESS);
    if Status < 0 {
        return Status;
    }

    KeyString.Buffer = Buffer.as_mut_ptr();
    KeyString.Length = (core::mem::size_of::<u16>() * 5) as u16;
    KeyString.MaximumLength = (core::mem::size_of::<u16>() * 6) as u16;
    core::ptr::copy_nonoverlapping(b"S\0e\0t\0u\0p\0\0\0".as_ptr(), Buffer.as_mut_ptr() as *mut u8, 12);
    Status = IopCreateRegistryKeyEx(
        &mut KeyHandle,
        RootHandle,
        &mut KeyString,
        KEY_ALL_ACCESS,
        REG_OPTION_NON_VOLATILE,
        core::ptr::null_mut(),
    );
    ZwClose(RootHandle);
    if Status < 0 {
        return Status;
    }

    KeyString.Length = (core::mem::size_of::<u16>() * 15) as u16;
    KeyString.MaximumLength = (core::mem::size_of::<u16>() * 16) as u16;
    core::ptr::copy_nonoverlapping(
        b"S\0y\0s\0t\0e\0m\0P\0a\0r\0t\0i\0t\0i\0o\0n\0\0\0".as_ptr(),
        Buffer.as_mut_ptr() as *mut u8,
        32,
    );
    Status = ZwSetValueKey(
        KeyHandle,
        &mut KeyString,
        0,
        REG_SZ,
        (*VolumeNameString).Buffer as PVOID,
        (*VolumeNameString).Length as u32 + core::mem::size_of::<u16>() as u32,
    );
    ZwClose(KeyHandle);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn IoVolumeDeviceToDosName(
    VolumeDeviceObject: PVOID,
    DosName: *mut _UNICODE_STRING,
) -> NTSTATUS {
    let VolumeDeviceObject = opaque_ptr(VolumeDeviceObject);
    let DosName = opaque_ptr(DosName);
    #[repr(C)]
    struct DeviceNameStruct {
        NameLength: u16,
        DeviceName: [u16; 256],
    }

    let mut DeviceName: DeviceNameStruct = core::mem::zeroed();
    let mut Event: _KEVENT = core::mem::zeroed();
    KeInitializeEvent(&mut Event, _EVENT_TYPE_NotificationEvent, 0u8);
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let Irp = IoBuildDeviceIoControlRequest(
        IOCTL_MOUNTDEV_QUERY_DEVICE_NAME,
        VolumeDeviceObject as PDEVICE_OBJECT,
        core::ptr::null_mut(),
        0,
        &mut DeviceName as *mut _ as PVOID,
        core::mem::size_of::<DeviceNameStruct>() as u32,
        0u8,
        &mut Event,
        &mut IoStatusBlock,
    );
    if Irp.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    let mut Status = IoCallDriver(VolumeDeviceObject as PDEVICE_OBJECT, Irp);
    if Status == STATUS_PENDING as i32 {
        KeWaitForSingleObject(
            &mut Event as *mut _ as PVOID,
            _KWAIT_REASON_Executive,
            0i8,
            0u8,
            core::ptr::null_mut(),
        );
        Status = IoStatusBlock.__bindgen_anon_1.Status;
    }
    if Status < 0 {
        return Status;
    }

    let mut MountMgrDevice = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    RtlInitUnicodeString(&mut MountMgrDevice, MOUNTMGR_DEVICE_NAME.as_ptr());
    let mut FileObject: PFILE_OBJECT = core::ptr::null_mut();
    let mut DeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    Status = IoGetDeviceObjectPointer(
        &mut MountMgrDevice,
        FILE_READ_ATTRIBUTES,
        &mut FileObject,
        &mut DeviceObject,
    );
    if Status < 0 {
        return Status;
    }

    let mut VolumePath: _MOUNTMGR_VOLUME_PATHS = core::mem::zeroed();
    KeInitializeEvent(&mut Event, _EVENT_TYPE_NotificationEvent, 0u8);
    let Irp = IoBuildDeviceIoControlRequest(
        IOCTL_MOUNTMGR_QUERY_DOS_VOLUME_PATH,
        DeviceObject,
        &mut DeviceName as *mut _ as PVOID,
        core::mem::size_of::<DeviceNameStruct>() as u32,
        &mut VolumePath as *mut _ as PVOID,
        core::mem::size_of::<_MOUNTMGR_VOLUME_PATHS>() as u32,
        0u8,
        &mut Event,
        &mut IoStatusBlock,
    );
    if Irp.is_null() {
        Status = STATUS_INSUFFICIENT_RESOURCES as i32;
        ObDereferenceObject(FileObject as POBJECT);
        return Status;
    }

    Status = IoCallDriver(DeviceObject, Irp);
    if Status == STATUS_PENDING as i32 {
        KeWaitForSingleObject(
            &mut Event as *mut _ as PVOID,
            _KWAIT_REASON_Executive,
            0i8,
            0u8,
            core::ptr::null_mut(),
        );
        Status = IoStatusBlock.__bindgen_anon_1.Status;
    }

    if Status < 0 && Status != STATUS_BUFFER_OVERFLOW as i32 {
        ObDereferenceObject(FileObject as POBJECT);
        return Status;
    }

    let Length = FIELD_OFFSET!(_MOUNTMGR_VOLUME_PATHS, MultiSz) + VolumePath.MultiSzLength as usize;
    if Length > 0xFFFF {
        Status = STATUS_INVALID_BUFFER_SIZE as i32;
        ObDereferenceObject(FileObject as POBJECT);
        return Status;
    }

    let VolumePathPtr = ExAllocatePoolWithTag(PagedPool, Length, TAG_DEV2DOS) as PMOUNTMGR_VOLUME_PATHS;
    if VolumePathPtr.is_null() {
        Status = STATUS_INSUFFICIENT_RESOURCES as i32;
        ObDereferenceObject(FileObject as POBJECT);
        return Status;
    }

    KeInitializeEvent(&mut Event, _EVENT_TYPE_NotificationEvent, 0u8);
    let Irp = IoBuildDeviceIoControlRequest(
        IOCTL_MOUNTMGR_QUERY_DOS_VOLUME_PATH,
        DeviceObject,
        &mut DeviceName as *mut _ as PVOID,
        core::mem::size_of::<DeviceNameStruct>() as u32,
        VolumePathPtr as PVOID,
        Length as u32,
        0u8,
        &mut Event,
        &mut IoStatusBlock,
    );
    if Irp.is_null() {
        Status = STATUS_INSUFFICIENT_RESOURCES as i32;
        ExFreePoolWithTag(VolumePathPtr as PVOID, TAG_DEV2DOS);
        ObDereferenceObject(FileObject as POBJECT);
        return Status;
    }

    Status = IoCallDriver(DeviceObject, Irp);
    if Status == STATUS_PENDING as i32 {
        KeWaitForSingleObject(
            &mut Event as *mut _ as PVOID,
            _KWAIT_REASON_Executive,
            0i8,
            0u8,
            core::ptr::null_mut(),
        );
        Status = IoStatusBlock.__bindgen_anon_1.Status;
    }
    if Status < 0 {
        ExFreePoolWithTag(VolumePathPtr as PVOID, TAG_DEV2DOS);
        ObDereferenceObject(FileObject as POBJECT);
        return Status;
    }

    (*DosName).Length = (*VolumePathPtr).MultiSzLength as u16 - 2 * core::mem::size_of::<u16>() as u16;
    (*DosName).MaximumLength = (*DosName).Length + core::mem::size_of::<u16>() as u16;
    (*DosName).Buffer = VolumePathPtr as *mut u16;
    // RtlCopyMemory/RtlMoveMemory count BYTES; copy as u8 (a u16 copy moves 2x).
    core::ptr::copy(
        (*VolumePathPtr).MultiSz.as_ptr() as *const u8,
        (*DosName).Buffer as *mut u8,
        (*DosName).Length as usize,
    );
    *((*DosName).Buffer.add((*DosName).Length as usize / core::mem::size_of::<u16>())) = 0;

    ObDereferenceObject(FileObject as POBJECT);
    STATUS_SUCCESS
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/io/iomgr/volume.c
//  * PURPOSE:         Volume and File System I/O Support
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  *                  Herv� Poussineau (hpoussin@reactos.org)
//  *                  Eric Kohl
//  *                  Pierre Schweitzer (pierre.schweitzer@reactos.org)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS *******************************************************************/
//
// ERESOURCE IopDatabaseResource;
// LIST_ENTRY IopDiskFileSystemQueueHead, IopNetworkFileSystemQueueHead;
// LIST_ENTRY IopCdRomFileSystemQueueHead, IopTapeFileSystemQueueHead;
// LIST_ENTRY IopFsNotifyChangeQueueHead;
// ULONG IopFsRegistrationOps;
//
// /* PRIVATE FUNCTIONS *********************************************************/
//
// /*
//  * @halfplemented
//  */
// VOID
// NTAPI
// IopDecrementDeviceObjectRef(IN PDEVICE_OBJECT DeviceObject,
//                             IN BOOLEAN UnloadIfUnused)
// {
//     KIRQL OldIrql;
//
//     /* Acquire lock */
//     OldIrql = KeAcquireQueuedSpinLock(LockQueueIoDatabaseLock);
//     ASSERT(DeviceObject->ReferenceCount > 0);
//
//     if (--DeviceObject->ReferenceCount > 0)
//     {
//         KeReleaseQueuedSpinLock(LockQueueIoDatabaseLock, OldIrql);
//         return;
//     }
//
//     /* Release lock */
//     KeReleaseQueuedSpinLock(LockQueueIoDatabaseLock, OldIrql);
//
//     /* Here, DO is not referenced any longer, check if we have to unload it */
//     if (UnloadIfUnused || IoGetDevObjExtension(DeviceObject)->ExtensionFlags &
//                           (DOE_UNLOAD_PENDING | DOE_DELETE_PENDING | DOE_REMOVE_PENDING))
//     {
//         /* Unload the driver */
//         IopUnloadDevice(DeviceObject);
//     }
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// IopDecrementDeviceObjectHandleCount(IN PDEVICE_OBJECT DeviceObject)
// {
//     /* Just decrease reference count */
//     IopDecrementDeviceObjectRef(DeviceObject, FALSE);
// }
//
// /*
//  * @implemented
//  */
// PVPB
// NTAPI
// IopCheckVpbMounted(IN POPEN_PACKET OpenPacket,
//                    IN PDEVICE_OBJECT DeviceObject,
//                    IN PUNICODE_STRING RemainingName,
//                    OUT PNTSTATUS Status)
// {
//     BOOLEAN Alertable, Raw;
//     KIRQL OldIrql;
//     PVPB Vpb = NULL;
//
//     /* Lock the VPBs */
//     IoAcquireVpbSpinLock(&OldIrql);
//
//     /* Set VPB mount settings */
//     Raw = !RemainingName->Length && !OpenPacket->RelatedFileObject;
//     Alertable = (OpenPacket->CreateOptions & FILE_SYNCHRONOUS_IO_ALERT) ?
//                 TRUE: FALSE;
//
//     /* Start looping until the VPB is mounted */
//     while (!(DeviceObject->Vpb->Flags & VPB_MOUNTED))
//     {
//         /* Release the lock */
//         IoReleaseVpbSpinLock(OldIrql);
//
//         /* Mount the volume */
//         *Status = IopMountVolume(DeviceObject,
//                                  Raw,
//                                  FALSE,
//                                  Alertable,
//                                  &Vpb);
//
//         /* Check if we failed or if we were alerted */
//         if (!(NT_SUCCESS(*Status)) ||
//             (*Status == STATUS_USER_APC) ||
//             (*Status == STATUS_ALERTED))
//         {
//             /* Dereference the device, since IopParseDevice referenced it */
//             IopDereferenceDeviceObject(DeviceObject, FALSE);
//
//             /* Check if it was a total failure */
//             if (!NT_SUCCESS(*Status)) return NULL;
//
//             /* Otherwise we were alerted */
//             *Status = STATUS_WRONG_VOLUME;
//             return NULL;
//         }
//         /*
//          * In case IopMountVolume returns a valid VPB
//          * Then, the volume is mounted, return it
//          */
//         else if (Vpb != NULL)
//         {
//             return Vpb;
//         }
//
//         /* Re-acquire the lock */
//         IoAcquireVpbSpinLock(&OldIrql);
//     }
//
//     /* Make sure the VPB isn't locked */
//     Vpb = DeviceObject->Vpb;
//     if (Vpb->Flags & VPB_LOCKED)
//     {
//         /* We're locked, so fail */
//         *Status = STATUS_ACCESS_DENIED;
//         Vpb = NULL;
//     }
//     else
//     {
//         /* Success! Reference the VPB */
//         Vpb->ReferenceCount++;
//     }
//
//     /* Release the lock and return the VPB */
//     IoReleaseVpbSpinLock(OldIrql);
//     return Vpb;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// IopCreateVpb(IN PDEVICE_OBJECT DeviceObject)
// {
//     PVPB Vpb;
//
//     /* Allocate the Vpb */
//     Vpb = ExAllocatePoolWithTag(NonPagedPool,
//                                 sizeof(VPB),
//                                 TAG_VPB);
//     if (!Vpb) return STATUS_INSUFFICIENT_RESOURCES;
//
//     /* Clear it so we don't waste time manually */
//     RtlZeroMemory(Vpb, sizeof(VPB));
//
//     /* Set the Header and Device Field */
//     Vpb->Type = IO_TYPE_VPB;
//     Vpb->Size = sizeof(VPB);
//     Vpb->RealDevice = DeviceObject;
//
//     /* Link it to the Device Object */
//     DeviceObject->Vpb = Vpb;
//     return STATUS_SUCCESS;
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// IopDereferenceVpbAndFree(IN PVPB Vpb)
// {
//     KIRQL OldIrql;
//
//     /* Lock the VPBs and decrease references */
//     IoAcquireVpbSpinLock(&OldIrql);
//     Vpb->ReferenceCount--;
//
//     /* Check if we're out of references */
//     if (!Vpb->ReferenceCount && Vpb->RealDevice->Vpb == Vpb &&
//         !(Vpb->Flags & VPB_PERSISTENT))
//     {
//         /* Release VPB lock */
//         IoReleaseVpbSpinLock(OldIrql);
//
//         /* And free VPB */
//         ExFreePoolWithTag(Vpb, TAG_VPB);
//     }
//     else
//     {
//         /* Release VPB lock */
//         IoReleaseVpbSpinLock(OldIrql);
//     }
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// IopReferenceVerifyVpb(IN PDEVICE_OBJECT DeviceObject,
//                       OUT PDEVICE_OBJECT *FileSystemObject,
//                       OUT PVPB *Vpb)
// {
//     KIRQL OldIrql;
//     PVPB LocalVpb;
//     BOOLEAN Result = FALSE;
//
//     /* Lock the VPBs and assume failure */
//     IoAcquireVpbSpinLock(&OldIrql);
//     *Vpb = NULL;
//     *FileSystemObject = NULL;
//
//     /* Get the VPB and make sure it's mounted */
//     LocalVpb = DeviceObject->Vpb;
//     if ((LocalVpb) && (LocalVpb->Flags & VPB_MOUNTED))
//     {
//         /* Return it */
//         *Vpb = LocalVpb;
//         *FileSystemObject = LocalVpb->DeviceObject;
//
//         /* Reference it */
//         LocalVpb->ReferenceCount++;
//         Result = TRUE;
//     }
//
//     /* Release the VPB lock and return status */
//     IoReleaseVpbSpinLock(OldIrql);
//     return Result;
// }
//
// PVPB
// NTAPI
// IopMountInitializeVpb(IN PDEVICE_OBJECT DeviceObject,
//                       IN PDEVICE_OBJECT AttachedDeviceObject,
//                       IN BOOLEAN Raw)
// {
//     KIRQL OldIrql;
//     PVPB Vpb;
//
//     /* Lock the VPBs */
//     IoAcquireVpbSpinLock(&OldIrql);
//     Vpb = DeviceObject->Vpb;
//
//     /* Set the VPB as mounted and possibly raw */
//     Vpb->Flags |= VPB_MOUNTED | (Raw ? VPB_RAW_MOUNT : 0);
//
//     /* Set the stack size */
//     Vpb->DeviceObject->StackSize = AttachedDeviceObject->StackSize;
//
//     /* Add one for the FS Driver */
//     Vpb->DeviceObject->StackSize++;
//
//     /* Set the VPB in the device extension */
//     IoGetDevObjExtension(Vpb->DeviceObject)->Vpb = Vpb;
//
//     /* Reference it */
//     Vpb->ReferenceCount++;
//
//     /* Release the VPB lock and return it */
//     IoReleaseVpbSpinLock(OldIrql);
//     return Vpb;
// }
//
// /*
//  * @implemented
//  */
// FORCEINLINE
// VOID
// IopNotifyFileSystemChange(IN PDEVICE_OBJECT DeviceObject,
//                           IN BOOLEAN DriverActive)
// {
//     PFS_CHANGE_NOTIFY_ENTRY ChangeEntry;
//     PLIST_ENTRY ListEntry;
//
//     /* Loop the list */
//     ListEntry = IopFsNotifyChangeQueueHead.Flink;
//     while (ListEntry != &IopFsNotifyChangeQueueHead)
//     {
//         /* Get the entry */
//         ChangeEntry = CONTAINING_RECORD(ListEntry,
//                                         FS_CHANGE_NOTIFY_ENTRY,
//                                         FsChangeNotifyList);
//
//         /* Call the notification procedure */
//         ChangeEntry->FSDNotificationProc(DeviceObject, DriverActive);
//
//         /* Go to the next entry */
//         ListEntry = ListEntry->Flink;
//     }
// }
//
// /*
//  * @implemented
//  */
// ULONG
// FASTCALL
// IopInterlockedIncrementUlong(IN KSPIN_LOCK_QUEUE_NUMBER Queue,
//                              IN PULONG Ulong)
// {
//     KIRQL Irql;
//     ULONG OldValue;
//
//     Irql = KeAcquireQueuedSpinLock(Queue);
//     OldValue = (*Ulong)++;
//     KeReleaseQueuedSpinLock(Queue, Irql);
//
//     return OldValue;
// }
//
// /*
//  * @implemented
//  */
// ULONG
// FASTCALL
// IopInterlockedDecrementUlong(IN KSPIN_LOCK_QUEUE_NUMBER Queue,
//                              IN PULONG Ulong)
// {
//     KIRQL Irql;
//     ULONG OldValue;
//
//     Irql = KeAcquireQueuedSpinLock(Queue);
//     OldValue = (*Ulong)--;
//     KeReleaseQueuedSpinLock(Queue, Irql);
//
//     return OldValue;
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// IopShutdownBaseFileSystems(IN PLIST_ENTRY ListHead)
// {
//     PLIST_ENTRY ListEntry;
//     PDEVICE_OBJECT DeviceObject;
//     IO_STATUS_BLOCK StatusBlock;
//     PIRP Irp;
//     KEVENT Event;
//     NTSTATUS Status;
//
//     KeInitializeEvent(&Event, NotificationEvent, FALSE);
//
//     /* Get the first entry and start looping */
//     ListEntry = ListHead->Flink;
//     while (ListEntry != ListHead)
//     {
//         /* Get the device object */
//         DeviceObject = CONTAINING_RECORD(ListEntry,
//                                          DEVICE_OBJECT,
//                                          Queue.ListEntry);
//
//         /* Go to the next entry */
//         ListEntry = ListEntry->Flink;
//
//         /* Get the attached device */
//         DeviceObject = IoGetAttachedDevice(DeviceObject);
//
//         ObReferenceObject(DeviceObject);
//         IopInterlockedIncrementUlong(LockQueueIoDatabaseLock, (PULONG)&DeviceObject->ReferenceCount);
//
//         /* Build the shutdown IRP and call the driver */
//         Irp = IoBuildSynchronousFsdRequest(IRP_MJ_SHUTDOWN,
//                                            DeviceObject,
//                                            NULL,
//                                            0,
//                                            NULL,
//                                            &Event,
//                                            &StatusBlock);
//         if (Irp)
//         {
//             Status = IoCallDriver(DeviceObject, Irp);
//             if (Status == STATUS_PENDING)
//             {
//                 /* Wait on the driver */
//                 KeWaitForSingleObject(&Event, Executive, KernelMode, FALSE, NULL);
//             }
//         }
//
//         /* Reset the event */
//         KeClearEvent(&Event);
//
//         IopDecrementDeviceObjectRef(DeviceObject, FALSE);
//         ObDereferenceObject(DeviceObject);
//     }
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// IopLoadFileSystemDriver(IN PDEVICE_OBJECT DeviceObject)
// {
//     IO_STATUS_BLOCK IoStatusBlock;
//     PIO_STACK_LOCATION StackPtr;
//     KEVENT Event;
//     PIRP Irp;
//     NTSTATUS Status;
//     PDEVICE_OBJECT AttachedDeviceObject = DeviceObject;
//     PAGED_CODE();
//
//     /* Loop as long as we're attached */
//     while (AttachedDeviceObject->AttachedDevice)
//     {
//         /* Get the attached device object */
//         AttachedDeviceObject = AttachedDeviceObject->AttachedDevice;
//     }
//
//     /* Initialize the event and build the IRP */
//     KeInitializeEvent(&Event, NotificationEvent, FALSE);
//     Irp = IoBuildDeviceIoControlRequest(IRP_MJ_DEVICE_CONTROL,
//                                         AttachedDeviceObject,
//                                         NULL,
//                                         0,
//                                         NULL,
//                                         0,
//                                         FALSE,
//                                         &Event,
//                                         &IoStatusBlock);
//     if (Irp)
//     {
//         /* Set the major and minor functions */
//         StackPtr = IoGetNextIrpStackLocation(Irp);
//         StackPtr->MajorFunction = IRP_MJ_FILE_SYSTEM_CONTROL;
//         StackPtr->MinorFunction = IRP_MN_LOAD_FILE_SYSTEM;
//
//         /* Call the driver */
//         Status = IoCallDriver(AttachedDeviceObject, Irp);
//         if (Status == STATUS_PENDING)
//         {
//             /* Wait on it */
//             KeWaitForSingleObject(&Event, Executive, KernelMode, FALSE, NULL);
//         }
//     }
//
//     /* Dereference DO - FsRec? - Comment out call, since it breaks up 2nd stage boot, needs more research. */
// //  IopDecrementDeviceObjectRef(AttachedDeviceObject, TRUE);
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// IopMountVolume(IN PDEVICE_OBJECT DeviceObject,
//                IN BOOLEAN AllowRawMount,
//                IN BOOLEAN DeviceIsLocked,
//                IN BOOLEAN Alertable,
//                OUT PVPB *Vpb)
// {
//     KEVENT Event;
//     NTSTATUS Status;
//     IO_STATUS_BLOCK IoStatusBlock;
//     PIRP Irp;
//     PIO_STACK_LOCATION StackPtr;
//     PLIST_ENTRY FsList, ListEntry;
//     LIST_ENTRY LocalList;
//     PDEVICE_OBJECT AttachedDeviceObject = DeviceObject;
//     PDEVICE_OBJECT FileSystemDeviceObject, ParentFsDeviceObject;
//     ULONG FsStackOverhead, RegistrationOps;
//     PAGED_CODE();
//
//
// #if defined(_M_ARM64) && defined(ARM64_UART_TRACE_ENABLE)
//     {
//         volatile ULONG *_u = (volatile ULONG *)Arm64UartKernelBase;
//         char _buf[128];
//         const char *_p;
//         sprintf(_buf, "[MOUNT] IopMountVolume DevType=0x%lx Vpb=%p\r\n",
//                 (unsigned long)DeviceObject->DeviceType, DeviceObject->Vpb);
//         _p = _buf;
//         while (*_p) { *_u = (ULONG)*_p++; }
//     }
// #endif
//
//     /* Check if the device isn't already locked */
//     if (!DeviceIsLocked)
//     {
//         /* Lock it ourselves */
//         ARM64_UART_TRACE!("[MOUNT] Acquiring DeviceLock\r\n");
//         Status = KeWaitForSingleObject(&DeviceObject->DeviceLock,
//                                        Executive,
//                                        KeGetPreviousMode(),
//                                        Alertable,
//                                        NULL);
//         if ((Status == STATUS_ALERTED) || (Status == STATUS_USER_APC))
//         {
//             /* Don't mount if we were interrupted */
//             return Status;
//         }
//         ARM64_UART_TRACE!("[MOUNT] DeviceLock acquired\r\n");
//     }
//
//     /* Acquire the FS Lock*/
//     KeEnterCriticalRegion();
//     ExAcquireResourceSharedLite(&IopDatabaseResource, TRUE);
//
// #if defined(_M_ARM64) && defined(ARM64_UART_TRACE_ENABLE)
//     {
//         volatile ULONG *_u = (volatile ULONG *)Arm64UartKernelBase;
//         char _buf[128];
//         const char *_p;
//         PLIST_ENTRY _fl = &IopDiskFileSystemQueueHead;
//         int _cnt = 0;
//         PLIST_ENTRY _le = _fl->Flink;
//         while (_le != _fl) { _cnt++; _le = _le->Flink; }
//         sprintf(_buf, "[MOUNT] DiskFS list has %d entries, VpbFlags=0x%lx\r\n",
//                 _cnt, DeviceObject->Vpb ? (unsigned long)DeviceObject->Vpb->Flags : 0UL);
//         _p = _buf;
//         while (*_p) { *_u = (ULONG)*_p++; }
//     }
// #endif
//
//     /* Make sure we weren't already mounted */
//     if (!(DeviceObject->Vpb->Flags & (VPB_MOUNTED | VPB_REMOVE_PENDING)))
//     {
//         /* Initialize the event to wait on */
//         KeInitializeEvent(&Event, NotificationEvent, FALSE);
//
//         /* Remove the verify flag and get the actual device to mount */
//         DeviceObject->Flags &= ~DO_VERIFY_VOLUME;
//         while (AttachedDeviceObject->AttachedDevice)
//         {
//             /* Get the next one */
//             AttachedDeviceObject = AttachedDeviceObject->AttachedDevice;
//         }
//
//         /* Reference it */
//         ObReferenceObject(AttachedDeviceObject);
//
//         /* For a mount operation, this can only be a Disk, CD-ROM or tape */
//         if ((DeviceObject->DeviceType == FILE_DEVICE_DISK) ||
//             (DeviceObject->DeviceType == FILE_DEVICE_VIRTUAL_DISK))
//         {
//             /* Use the disk list */
//             FsList = &IopDiskFileSystemQueueHead;
//         }
//         else if (DeviceObject->DeviceType == FILE_DEVICE_CD_ROM)
//         {
//             /* Use the CD-ROM list */
//             FsList = &IopCdRomFileSystemQueueHead;
//         }
//         else
//         {
//             /* It's gotta be a tape... */
//             FsList = &IopTapeFileSystemQueueHead;
//         }
//
//         /* Now loop the fs list until one of the file systems accepts us */
//         Status = STATUS_UNSUCCESSFUL;
//         ListEntry = FsList->Flink;
//         while ((ListEntry != FsList) && !(NT_SUCCESS(Status)))
//         {
//             /*
//              * If we're not allowed to mount this volume and this is our last
//              * (but not only) chance to mount it...
//              */
//             if (!(AllowRawMount) &&
//                 (ListEntry->Flink == FsList) &&
//                 (ListEntry != FsList->Flink))
//             {
//                 /* Then fail this mount request */
//                 break;
//             }
//
//             /*
//              * Also check if this is a raw mount and there are other file
//              * systems on the list.
//              */
//             if ((DeviceObject->Vpb->Flags & VPB_RAW_MOUNT) &&
//                 (ListEntry->Flink != FsList))
//             {
//                 /* Then skip this entry */
//                 ListEntry = ListEntry->Flink;
//                 continue;
//             }
//
//             /* Get the Device Object for this FS */
//             FileSystemDeviceObject = CONTAINING_RECORD(ListEntry,
//                                                        DEVICE_OBJECT,
//                                                        Queue.ListEntry);
//             ParentFsDeviceObject = FileSystemDeviceObject;
//
//             /*
//              * If this file system device is attached to some other device,
//              * then we must make sure to increase the stack size for the IRP.
//              * The default is +1, for the FS device itself.
//              */
//             FsStackOverhead = 1;
//             while (FileSystemDeviceObject->AttachedDevice)
//             {
//                 /* Get the next attached device and increase overhead */
//                 FileSystemDeviceObject = FileSystemDeviceObject->
//                                          AttachedDevice;
//                 FsStackOverhead++;
//             }
//
//             /* Clear the event */
//             KeClearEvent(&Event);
//
//             /* Allocate the IRP */
//             Irp = IoAllocateIrp(AttachedDeviceObject->StackSize +
//                                 (UCHAR)FsStackOverhead,
//                                 TRUE);
//             if (!Irp)
//             {
//                 /* Fail */
//                 Status =  STATUS_INSUFFICIENT_RESOURCES;
//                 break;
//             }
//
//             /* Setup the IRP */
//             Irp->UserIosb = &IoStatusBlock;
//             Irp->UserEvent = &Event;
//             Irp->Tail.Overlay.Thread = PsGetCurrentThread();
//             Irp->Flags = IRP_MOUNT_COMPLETION | IRP_SYNCHRONOUS_PAGING_IO;
//             Irp->RequestorMode = KernelMode;
//
//             /* Get the I/O Stack location and set it up */
//             StackPtr = IoGetNextIrpStackLocation(Irp);
//             StackPtr->MajorFunction = IRP_MJ_FILE_SYSTEM_CONTROL;
//             StackPtr->MinorFunction = IRP_MN_MOUNT_VOLUME;
//             StackPtr->Flags = AllowRawMount;
//             StackPtr->Parameters.MountVolume.Vpb = DeviceObject->Vpb;
//             StackPtr->Parameters.MountVolume.DeviceObject =
//                 AttachedDeviceObject;
//
//             /* Save registration operations */
//             RegistrationOps = IopFsRegistrationOps;
//
//             /* Release locks */
//             IopInterlockedIncrementUlong(LockQueueIoDatabaseLock, (PULONG)&DeviceObject->ReferenceCount);
//             ExReleaseResourceLite(&IopDatabaseResource);
//
//             /* Call the driver */
//             ARM64_UART_TRACE!("[MOUNT] IoCallDriver(mount IRP)\r\n");
//             Status = IoCallDriver(FileSystemDeviceObject, Irp);
// #if defined(_M_ARM64) && defined(ARM64_UART_TRACE_ENABLE)
//             {
//                 volatile ULONG *_u = (volatile ULONG *)Arm64UartKernelBase;
//                 char _buf[64];
//                 const char *_p;
//                 sprintf(_buf, "[MOUNT] IoCallDriver returned 0x%lx\r\n", (unsigned long)Status);
//                 _p = _buf;
//                 while (*_p) { *_u = (ULONG)*_p++; }
//             }
// #endif
//             if (Status == STATUS_PENDING)
//             {
//                 /* Wait on it */
//                 ARM64_UART_TRACE!("[MOUNT] Waiting for mount completion...\r\n");
//                 KeWaitForSingleObject(&Event,
//                                       Executive,
//                                       KernelMode,
//                                       FALSE,
//                                       NULL);
//                 Status = IoStatusBlock.Status;
// #if defined(_M_ARM64) && defined(ARM64_UART_TRACE_ENABLE)
//                 {
//                     volatile ULONG *_u = (volatile ULONG *)Arm64UartKernelBase;
//                     char _buf[64];
//                     const char *_p;
//                     sprintf(_buf, "[MOUNT] Wait done, status=0x%lx\r\n", (unsigned long)Status);
//                     _p = _buf;
//                     while (*_p) { *_u = (ULONG)*_p++; }
//                 }
// #endif
//             }
//
//             ExAcquireResourceSharedLite(&IopDatabaseResource, TRUE);
//             IopInterlockedDecrementUlong(LockQueueIoDatabaseLock, (PULONG)&DeviceObject->ReferenceCount);
//
//             /* Check if mounting was successful */
//             if (NT_SUCCESS(Status))
//             {
//                 /* Mount the VPB */
//                 *Vpb = IopMountInitializeVpb(DeviceObject,
//                                              AttachedDeviceObject,
//                                              (DeviceObject->Vpb->Flags &
//                                               VPB_RAW_MOUNT));
//             }
//             else
//             {
//                 /* Check if we failed because of the user */
//                 if ((IoIsErrorUserInduced(Status)) &&
//                     (IoStatusBlock.Information == 1))
//                 {
//                     /* Break out and fail */
//                     break;
//                 }
//
//                 /* If there were registration operations in the meanwhile */
//                 if (RegistrationOps != IopFsRegistrationOps)
//                 {
//                     /* We need to setup a local list to pickup where we left */
//                     LocalList.Flink = FsList->Flink;
//                     ListEntry = &LocalList;
//
//                     Status = STATUS_UNRECOGNIZED_VOLUME;
//                 }
//
//                 /* Otherwise, check if we need to load the FS driver */
//                 if (Status == STATUS_FS_DRIVER_REQUIRED)
//                 {
//                     /* We need to release the lock */
//                     IopInterlockedIncrementUlong(LockQueueIoDatabaseLock, (PULONG)&DeviceObject->ReferenceCount);
//                     ExReleaseResourceLite(&IopDatabaseResource);
//
//                     /* Release the device lock if we're holding it */
//                     if (!DeviceIsLocked)
//                     {
//                         KeSetEvent(&DeviceObject->DeviceLock, 0, FALSE);
//                     }
//
//                     /* Leave critical section */
//                     KeLeaveCriticalRegion();
//
//                     /* Load the FS */
//                     IopLoadFileSystemDriver(ParentFsDeviceObject);
//
//                     /* Check if the device isn't already locked */
//                     if (!DeviceIsLocked)
//                     {
//                         /* Lock it ourselves */
//                         Status = KeWaitForSingleObject(&DeviceObject->
//                                                        DeviceLock,
//                                                        Executive,
//                                                        KeGetPreviousMode(),
//                                                        Alertable,
//                                                        NULL);
//                         if ((Status == STATUS_ALERTED) ||
//                             (Status == STATUS_USER_APC))
//                         {
//                             /* Don't mount if we were interrupted */
//                             ObDereferenceObject(AttachedDeviceObject);
//                             return Status;
//                         }
//                     }
//
//                     /* Reacquire the lock */
//                     KeEnterCriticalRegion();
//                     ExAcquireResourceSharedLite(&IopDatabaseResource, TRUE);
//
//                     /* When we released the lock, make sure nobody beat us */
//                     if (DeviceObject->Vpb->Flags & VPB_MOUNTED)
//                     {
//                         /* Someone did, break out */
//                         Status = STATUS_SUCCESS;
//                         break;
//                     }
//
//                     /* Start over by setting a failure */
//                     Status = STATUS_UNRECOGNIZED_VOLUME;
//
//                     /* We need to setup a local list to pickup where we left */
//                     LocalList.Flink = FsList->Flink;
//                     ListEntry = &LocalList;
//                 }
//
//                 /*
//                  * Check if we failed with any other error then an unrecognized
//                  * volume, and if this request doesn't allow mounting the raw
//                  * file system.
//                  */
//                 if (!(AllowRawMount) &&
//                     (Status != STATUS_UNRECOGNIZED_VOLUME) &&
//                     (FsRtlIsTotalDeviceFailure(Status)))
//                 {
//                     /* Break out and give up */
//                     break;
//                 }
//             }
//
//             /* Go to the next FS entry */
//             ListEntry = ListEntry->Flink;
//         }
//
//         /* Dereference the device if we failed */
//         if (!NT_SUCCESS(Status)) ObDereferenceObject(AttachedDeviceObject);
//     }
//     else if (DeviceObject->Vpb->Flags & VPB_REMOVE_PENDING)
//     {
//         /* Someone wants to remove us */
//         Status = STATUS_DEVICE_DOES_NOT_EXIST;
//     }
//     else
//     {
//         /* Someone already mounted us */
//         Status = STATUS_SUCCESS;
//     }
//
//     /* Release the FS lock */
//     ExReleaseResourceLite(&IopDatabaseResource);
//     KeLeaveCriticalRegion();
//
//     /* Release the device lock if we're holding it */
//     if (!DeviceIsLocked) KeSetEvent(&DeviceObject->DeviceLock, 0, FALSE);
//
//     /* Check if we failed to mount the boot partition */
//     if ((!NT_SUCCESS(Status)) &&
//         (DeviceObject->Flags & DO_SYSTEM_BOOT_PARTITION) &&
//         ExpInitializationPhase < 2)
//     {
//         /* Bugcheck the system */
//         KeBugCheckEx(INACCESSIBLE_BOOT_DEVICE,
//                      (ULONG_PTR)DeviceObject,
//                      Status,
//                      0,
//                      0);
//     }
//
//     /* Return the mount status */
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// IopNotifyAlreadyRegisteredFileSystems(IN PLIST_ENTRY ListHead,
//                                       IN PDRIVER_FS_NOTIFICATION DriverNotificationRoutine,
//                                       BOOLEAN SkipRawFs)
// {
//     PLIST_ENTRY ListEntry;
//     PDEVICE_OBJECT DeviceObject;
//
//     /* Browse the whole list */
//     ListEntry = ListHead->Flink;
//     while (ListEntry != ListHead)
//     {
//         /* Check if we reached rawfs and if we have to skip it */
//         if (ListEntry->Flink == ListHead && SkipRawFs)
//         {
//             return;
//         }
//
//         /* Otherwise, get DO and notify */
//         DeviceObject = CONTAINING_RECORD(ListEntry,
//                                          DEVICE_OBJECT,
//                                          Queue.ListEntry);
//
//         DriverNotificationRoutine(DeviceObject, TRUE);
//
//         /* Go to the next entry */
//         ListEntry = ListEntry->Flink;
//     }
// }
//
// /* PUBLIC FUNCTIONS **********************************************************/
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// IoEnumerateRegisteredFiltersList(OUT PDRIVER_OBJECT *DriverObjectList,
//                                  IN ULONG DriverObjectListSize,
//                                  OUT PULONG ActualNumberDriverObjects)
// {
//     PLIST_ENTRY ListEntry;
//     NTSTATUS Status = STATUS_SUCCESS;
//     PFS_CHANGE_NOTIFY_ENTRY ChangeEntry;
//     ULONG ListSize = 0, MaximumSize = DriverObjectListSize / sizeof(PDRIVER_OBJECT);
//
//     /* Acquire the FS lock */
//     KeEnterCriticalRegion();
//     ExAcquireResourceExclusiveLite(&IopDatabaseResource, TRUE);
//
//     /* Browse the whole list */
//     ListEntry = IopFsNotifyChangeQueueHead.Flink;
//     while (ListEntry != &IopFsNotifyChangeQueueHead)
//     {
//         ChangeEntry = CONTAINING_RECORD(ListEntry,
//                                         FS_CHANGE_NOTIFY_ENTRY,
//                                         FsChangeNotifyList);
//
//         /* If buffer is still big enough */
//         if (ListSize < MaximumSize)
//         {
//             /* Reference the driver object */
//             ObReferenceObject(ChangeEntry->DriverObject);
//             /* And pass it to the caller */
//             DriverObjectList[ListSize] = ChangeEntry->DriverObject;
//         }
//         else
//         {
//             Status = STATUS_BUFFER_TOO_SMALL;
//         }
//
//         /* Increase size counter */
//         ListSize++;
//
//         /* Go to the next entry */
//         ListEntry = ListEntry->Flink;
//     }
//
//     /* Return list size */
//     *ActualNumberDriverObjects = ListSize;
//
//     /* Release the FS lock */
//     ExReleaseResourceLite(&IopDatabaseResource);
//     KeLeaveCriticalRegion();
//
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// IoVerifyVolume(IN PDEVICE_OBJECT DeviceObject,
//                IN BOOLEAN AllowRawMount)
// {
//     IO_STATUS_BLOCK IoStatusBlock;
//     PIO_STACK_LOCATION StackPtr;
//     KEVENT Event;
//     PIRP Irp;
//     NTSTATUS Status, VpbStatus;
//     PDEVICE_OBJECT FileSystemDeviceObject;
//     PVPB Vpb, NewVpb;
//     //BOOLEAN WasNotMounted = TRUE;
//
//     /* Wait on the device lock */
//     Status = KeWaitForSingleObject(&DeviceObject->DeviceLock,
//                                    Executive,
//                                    KernelMode,
//                                    FALSE,
//                                    NULL);
//     ASSERT(Status == STATUS_SUCCESS);
//
//     /* Reference the VPB */
//     if (IopReferenceVerifyVpb(DeviceObject, &FileSystemDeviceObject, &Vpb))
//     {
//         /* Initialize the event */
//         KeInitializeEvent(&Event, NotificationEvent, FALSE);
//
//         /* Find the actual File System DO */
//         //WasNotMounted = FALSE;
//         FileSystemDeviceObject = DeviceObject->Vpb->DeviceObject;
//         while (FileSystemDeviceObject->AttachedDevice)
//         {
//             /* Go to the next one */
//             FileSystemDeviceObject = FileSystemDeviceObject->AttachedDevice;
//         }
//
//         /* Allocate the IRP */
//         Irp = IoAllocateIrp(FileSystemDeviceObject->StackSize, FALSE);
//         if (!Irp)
//         {
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//             goto Release;
//         }
//
//         /* Set it up */
//         Irp->UserIosb = &IoStatusBlock;
//         Irp->UserEvent = &Event;
//         Irp->Tail.Overlay.Thread = PsGetCurrentThread();
//         Irp->Flags = IRP_MOUNT_COMPLETION | IRP_SYNCHRONOUS_PAGING_IO;
//         Irp->RequestorMode = KernelMode;
//
//         /* Get the I/O Stack location and set it */
//         StackPtr = IoGetNextIrpStackLocation(Irp);
//         StackPtr->MajorFunction = IRP_MJ_FILE_SYSTEM_CONTROL;
//         StackPtr->MinorFunction = IRP_MN_VERIFY_VOLUME;
//         StackPtr->Flags = AllowRawMount ? SL_ALLOW_RAW_MOUNT : 0;
//         StackPtr->Parameters.VerifyVolume.Vpb = Vpb;
//         StackPtr->Parameters.VerifyVolume.DeviceObject =
//             DeviceObject->Vpb->DeviceObject;
//
//         /* Call the driver */
//         Status = IoCallDriver(FileSystemDeviceObject, Irp);
//         if (Status == STATUS_PENDING)
//         {
//             /* Wait on it */
//             KeWaitForSingleObject(&Event, Executive, KernelMode, FALSE, NULL);
//             Status = IoStatusBlock.Status;
//         }
//
//         /* Dereference the VPB */
//         IopDereferenceVpbAndFree(Vpb);
//     }
//
//     /* Check if we had the wrong volume or didn't mount at all */
//     if (Status == STATUS_WRONG_VOLUME)
//     {
//         /* Create a VPB */
//         VpbStatus = IopCreateVpb(DeviceObject);
//         if (NT_SUCCESS(VpbStatus))
//         {
//             PoVolumeDevice(DeviceObject);
//
//             /* Mount it */
//             VpbStatus = IopMountVolume(DeviceObject,
//                                        AllowRawMount,
//                                        TRUE,
//                                        FALSE,
//                                        &NewVpb);
//
//             /* If we got a new VPB, dereference it */
//             if (NewVpb)
//             {
//                 IopInterlockedDecrementUlong(LockQueueIoVpbLock, &NewVpb->ReferenceCount);
//             }
//         }
//
//         /* If we failed, remove the verify flag */
//         if (!NT_SUCCESS(VpbStatus)) DeviceObject->Flags &= ~DO_VERIFY_VOLUME;
//     }
//
// Release:
//     /* Signal the device lock and return */
//     KeSetEvent(&DeviceObject->DeviceLock, IO_NO_INCREMENT, FALSE);
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// IoRegisterFileSystem(IN PDEVICE_OBJECT DeviceObject)
// {
//     PLIST_ENTRY FsList = NULL;
//     PAGED_CODE();
//
//     /* Acquire the FS lock */
//     KeEnterCriticalRegion();
//     ExAcquireResourceExclusiveLite(&IopDatabaseResource, TRUE);
//
//     /* Check what kind of FS this is */
//     if (DeviceObject->DeviceType == FILE_DEVICE_DISK_FILE_SYSTEM)
//     {
//         /* Use the disk list */
//         FsList = &IopDiskFileSystemQueueHead;
//     }
//     else if (DeviceObject->DeviceType == FILE_DEVICE_NETWORK_FILE_SYSTEM)
//     {
//         /* Use the network device list */
//         FsList = &IopNetworkFileSystemQueueHead;
//     }
//     else if (DeviceObject->DeviceType == FILE_DEVICE_CD_ROM_FILE_SYSTEM)
//     {
//         /* Use the CD-ROM list */
//         FsList = &IopCdRomFileSystemQueueHead;
//     }
//     else if (DeviceObject->DeviceType == FILE_DEVICE_TAPE_FILE_SYSTEM)
//     {
//         /* Use the tape list */
//         FsList = &IopTapeFileSystemQueueHead;
//     }
//
//     /* Make sure that we have a valid list */
//     if (FsList)
//     {
//         /* Check if we should insert it at the top or bottom of the list */
//         if (DeviceObject->Flags & DO_LOW_PRIORITY_FILESYSTEM)
//         {
//             /* At the bottom */
//             InsertTailList(FsList->Blink, &DeviceObject->Queue.ListEntry);
//         }
//         else
//         {
//             /* On top */
//             InsertHeadList(FsList, &DeviceObject->Queue.ListEntry);
//         }
//     }
//
//     /* Update operations counter */
//     IopFsRegistrationOps++;
//
//     /* Clear the initializing flag */
//     DeviceObject->Flags &= ~DO_DEVICE_INITIALIZING;
//
//     /* Notify file systems of the addition */
//     IopNotifyFileSystemChange(DeviceObject, TRUE);
//
//     /* Release the FS Lock */
//     ExReleaseResourceLite(&IopDatabaseResource);
//     KeLeaveCriticalRegion();
//
//     /* Ensure driver won't be unloaded */
//     IopInterlockedIncrementUlong(LockQueueIoDatabaseLock, (PULONG)&DeviceObject->ReferenceCount);
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// IoUnregisterFileSystem(IN PDEVICE_OBJECT DeviceObject)
// {
//     PAGED_CODE();
//
//     /* Acquire the FS lock */
//     KeEnterCriticalRegion();
//     ExAcquireResourceExclusiveLite(&IopDatabaseResource, TRUE);
//
//     /* Simply remove the entry - if queued */
//     if (DeviceObject->Queue.ListEntry.Flink)
//     {
//         RemoveEntryList(&DeviceObject->Queue.ListEntry);
//     }
//
//     /* And notify all registered file systems */
//     IopNotifyFileSystemChange(DeviceObject, FALSE);
//
//     /* Update operations counter */
//     IopFsRegistrationOps++;
//
//     /* Then release the lock */
//     ExReleaseResourceLite(&IopDatabaseResource);
//     KeLeaveCriticalRegion();
//
//     /* Decrease reference count to allow unload */
//     IopInterlockedDecrementUlong(LockQueueIoDatabaseLock, (PULONG)&DeviceObject->ReferenceCount);
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// IoRegisterFsRegistrationChange(IN PDRIVER_OBJECT DriverObject,
//                                IN PDRIVER_FS_NOTIFICATION DriverNotificationRoutine)
// {
//     PFS_CHANGE_NOTIFY_ENTRY Entry;
//     PAGED_CODE();
//
//     /* Acquire the list lock */
//     KeEnterCriticalRegion();
//     ExAcquireResourceExclusiveLite(&IopDatabaseResource, TRUE);
//
//     /* Check if that driver is already registered (successive calls)
//      * See MSDN note: https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/ntifs/nf-ntifs-ioregisterfsregistrationchange
//      */
//     if (!IsListEmpty(&IopFsNotifyChangeQueueHead))
//     {
//         Entry = CONTAINING_RECORD(IopFsNotifyChangeQueueHead.Blink,
//                                   FS_CHANGE_NOTIFY_ENTRY,
//                                   FsChangeNotifyList);
//
//         if (Entry->DriverObject == DriverObject &&
//             Entry->FSDNotificationProc == DriverNotificationRoutine)
//         {
//             /* Release the lock */
//             ExReleaseResourceLite(&IopDatabaseResource);
//
//             return STATUS_DEVICE_ALREADY_ATTACHED;
//         }
//     }
//
//     /* Allocate a notification entry */
//     Entry = ExAllocatePoolWithTag(PagedPool,
//                                   sizeof(FS_CHANGE_NOTIFY_ENTRY),
//                                   TAG_FS_CHANGE_NOTIFY);
//     if (!Entry)
//     {
//         /* Release the lock */
//         ExReleaseResourceLite(&IopDatabaseResource);
//
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* Save the driver object and notification routine */
//     Entry->DriverObject = DriverObject;
//     Entry->FSDNotificationProc = DriverNotificationRoutine;
//
//     /* Insert it into the notification list */
//     InsertTailList(&IopFsNotifyChangeQueueHead, &Entry->FsChangeNotifyList);
//
//     /* Start notifying all already present FS */
//     IopNotifyAlreadyRegisteredFileSystems(&IopNetworkFileSystemQueueHead, DriverNotificationRoutine, FALSE);
//     IopNotifyAlreadyRegisteredFileSystems(&IopCdRomFileSystemQueueHead, DriverNotificationRoutine, TRUE);
//     IopNotifyAlreadyRegisteredFileSystems(&IopDiskFileSystemQueueHead, DriverNotificationRoutine, TRUE);
//     IopNotifyAlreadyRegisteredFileSystems(&IopTapeFileSystemQueueHead, DriverNotificationRoutine, TRUE);
//
//     /* Release the lock */
//     ExReleaseResourceLite(&IopDatabaseResource);
//     KeLeaveCriticalRegion();
//
//     /* Reference the driver */
//     ObReferenceObject(DriverObject);
//     return STATUS_SUCCESS;
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// IoUnregisterFsRegistrationChange(IN PDRIVER_OBJECT DriverObject,
//                                  IN PDRIVER_FS_NOTIFICATION FSDNotificationProc)
// {
//     PFS_CHANGE_NOTIFY_ENTRY ChangeEntry;
//     PLIST_ENTRY NextEntry;
//     PAGED_CODE();
//
//     /* Acquire the list lock */
//     KeEnterCriticalRegion();
//     ExAcquireResourceExclusiveLite(&IopDatabaseResource, TRUE);
//
//     /* Loop the list */
//     NextEntry = IopFsNotifyChangeQueueHead.Flink;
//     while (NextEntry != &IopFsNotifyChangeQueueHead)
//     {
//         /* Get the entry */
//         ChangeEntry = CONTAINING_RECORD(NextEntry,
//                                         FS_CHANGE_NOTIFY_ENTRY,
//                                         FsChangeNotifyList);
//
//         /* Check if it matches this de-registration */
//         if ((ChangeEntry->DriverObject == DriverObject) &&
//             (ChangeEntry->FSDNotificationProc == FSDNotificationProc))
//         {
//             /* It does, remove it from the list */
//             RemoveEntryList(&ChangeEntry->FsChangeNotifyList);
//             ExFreePoolWithTag(ChangeEntry, TAG_FS_CHANGE_NOTIFY);
//             break;
//         }
//
//         /* Go to the next entry */
//         NextEntry = NextEntry->Flink;
//     }
//
//     /* Release the lock and dereference the driver */
//     ExReleaseResourceLite(&IopDatabaseResource);
//     KeLeaveCriticalRegion();
//
//     /* Dereference the driver */
//     ObDereferenceObject(DriverObject);
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// IoAcquireVpbSpinLock(OUT PKIRQL Irql)
// {
//     /* Simply acquire the lock */
//     *Irql = KeAcquireQueuedSpinLock(LockQueueIoVpbLock);
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// IoReleaseVpbSpinLock(IN KIRQL Irql)
// {
//     /* Just release the lock */
//     KeReleaseQueuedSpinLock(LockQueueIoVpbLock, Irql);
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// IoSetSystemPartition(IN PUNICODE_STRING VolumeNameString)
// {
//     NTSTATUS Status;
//     HANDLE RootHandle, KeyHandle;
//     UNICODE_STRING HKLMSystem, KeyString;
//     WCHAR Buffer[sizeof(L"SystemPartition") / sizeof(WCHAR)];
//
//     RtlInitUnicodeString(&HKLMSystem, L"\\REGISTRY\\MACHINE\\SYSTEM");
//
//     /* Open registry to save data (HKLM\SYSTEM) */
//     Status = IopOpenRegistryKeyEx(&RootHandle, 0, &HKLMSystem, KEY_ALL_ACCESS);
//     if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//
//     /* Create or open Setup subkey */
//     KeyString.Buffer = Buffer;
//     KeyString.Length = sizeof(L"Setup") - sizeof(UNICODE_NULL);
//     KeyString.MaximumLength = sizeof(L"Setup");
//     RtlCopyMemory(Buffer, L"Setup", sizeof(L"Setup"));
//     Status = IopCreateRegistryKeyEx(&KeyHandle,
//                                     RootHandle,
//                                     &KeyString,
//                                     KEY_ALL_ACCESS,
//                                     REG_OPTION_NON_VOLATILE,
//                                     NULL);
//     ZwClose(RootHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//
//     /* Store caller value */
//     KeyString.Length = sizeof(L"SystemPartition") - sizeof(UNICODE_NULL);
//     KeyString.MaximumLength = sizeof(L"SystemPartition");
//     RtlCopyMemory(Buffer, L"SystemPartition", sizeof(L"SystemPartition"));
//     Status = ZwSetValueKey(KeyHandle,
//                            &KeyString,
//                            0,
//                            REG_SZ,
//                            VolumeNameString->Buffer,
//                            VolumeNameString->Length + sizeof(UNICODE_NULL));
//     ZwClose(KeyHandle);
//
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// IoVolumeDeviceToDosName(
//     _In_ PVOID VolumeDeviceObject,
//     _Out_ _When_(return==0, _At_(DosName->Buffer, __drv_allocatesMem(Mem)))
//         PUNICODE_STRING DosName)
// {
//     NTSTATUS Status;
//     ULONG Length;
//     KEVENT Event;
//     PIRP Irp;
//     PFILE_OBJECT FileObject;
//     PDEVICE_OBJECT DeviceObject;
//     IO_STATUS_BLOCK IoStatusBlock;
//     UNICODE_STRING MountMgrDevice;
//     MOUNTMGR_VOLUME_PATHS VolumePath;
//     PMOUNTMGR_VOLUME_PATHS VolumePathPtr;
//     /*
//      * This variable is used to query the device name.
//      * It's based on MOUNTDEV_NAME (mountmgr.h).
//      * Doing it this way prevents memory allocation.
//      * The device name won't be longer.
//      */
//     struct
//     {
//         USHORT NameLength;
//         WCHAR DeviceName[256];
//     } DeviceName;
//
//     PAGED_CODE();
//
//     /* First, retrieve the corresponding device name */
//     KeInitializeEvent(&Event, NotificationEvent, FALSE);
//     Irp = IoBuildDeviceIoControlRequest(IOCTL_MOUNTDEV_QUERY_DEVICE_NAME,
//                                         VolumeDeviceObject,
//                                         NULL, 0,
//                                         &DeviceName, sizeof(DeviceName),
//                                         FALSE, &Event, &IoStatusBlock);
//     if (!Irp)
//     {
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     Status = IoCallDriver(VolumeDeviceObject, Irp);
//     if (Status == STATUS_PENDING)
//     {
//         KeWaitForSingleObject(&Event, Executive, KernelMode, FALSE, NULL);
//         Status = IoStatusBlock.Status;
//     }
//     if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//
//     /* Retrieve the MountMgr controlling device */
//     RtlInitUnicodeString(&MountMgrDevice, MOUNTMGR_DEVICE_NAME);
//     Status = IoGetDeviceObjectPointer(&MountMgrDevice, FILE_READ_ATTRIBUTES,
//                                       &FileObject, &DeviceObject);
//     if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//
//     /* Now, query the MountMgr for the DOS path */
//     KeInitializeEvent(&Event, NotificationEvent, FALSE);
//     Irp = IoBuildDeviceIoControlRequest(IOCTL_MOUNTMGR_QUERY_DOS_VOLUME_PATH,
//                                         DeviceObject,
//                                         &DeviceName, sizeof(DeviceName),
//                                         &VolumePath, sizeof(VolumePath),
//                                         FALSE, &Event, &IoStatusBlock);
//     if (!Irp)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto Quit;
//     }
//
//     Status = IoCallDriver(DeviceObject, Irp);
//     if (Status == STATUS_PENDING)
//     {
//         KeWaitForSingleObject(&Event, Executive, KernelMode, FALSE, NULL);
//         Status = IoStatusBlock.Status;
//     }
//
//     /* The only tolerated failure here is buffer too small, which is expected */
//     if (!NT_SUCCESS(Status) && (Status != STATUS_BUFFER_OVERFLOW))
//     {
//         goto Quit;
//     }
//
//     /* Compute the needed size to store the DOS path.
//      * Even if MOUNTMGR_VOLUME_PATHS allows bigger name lengths
//      * than MAXUSHORT, we can't use them, because we have to return
//      * this in an UNICODE_STRING that stores length in a USHORT. */
//     Length = FIELD_OFFSET(MOUNTMGR_VOLUME_PATHS, MultiSz) + VolumePath.MultiSzLength;
//     if (Length > MAXUSHORT)
//     {
//         Status = STATUS_INVALID_BUFFER_SIZE;
//         goto Quit;
//     }
//
//     /* Allocate the buffer, even in case of success,
//      * because it is returned to the caller */
//     VolumePathPtr = ExAllocatePoolWithTag(PagedPool, Length, TAG_DEV2DOS);
//     if (!VolumePathPtr)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto Quit;
//     }
//
//     /* Re-query the DOS path with the proper size */
//     KeInitializeEvent(&Event, NotificationEvent, FALSE);
//     Irp = IoBuildDeviceIoControlRequest(IOCTL_MOUNTMGR_QUERY_DOS_VOLUME_PATH,
//                                         DeviceObject,
//                                         &DeviceName, sizeof(DeviceName),
//                                         VolumePathPtr, Length,
//                                         FALSE, &Event, &IoStatusBlock);
//     if (!Irp)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto ReleaseMemory;
//     }
//
//     Status = IoCallDriver(DeviceObject, Irp);
//     if (Status == STATUS_PENDING)
//     {
//         KeWaitForSingleObject(&Event, Executive, KernelMode, FALSE, NULL);
//         Status = IoStatusBlock.Status;
//     }
//     if (!NT_SUCCESS(Status))
//     {
//         goto ReleaseMemory;
//     }
//
//     /* Set the output string. Discount the last two
//      * NUL-terminators from the multi-string length. */
//     DosName->Length = (USHORT)VolumePathPtr->MultiSzLength - 2 * sizeof(UNICODE_NULL);
//     DosName->MaximumLength = DosName->Length + sizeof(UNICODE_NULL);
//     /* Recycle our MOUNTMGR_VOLUME_PATHS as the output buffer
//      * and move the NUL-terminated string to the beginning */
//     DosName->Buffer = (PWSTR)VolumePathPtr;
//     RtlMoveMemory(DosName->Buffer, VolumePathPtr->MultiSz, DosName->Length);
//     DosName->Buffer[DosName->Length / sizeof(WCHAR)] = UNICODE_NULL;
//
//     /* Don't release the buffer, just dereference the FO and return success */
//     Status = STATUS_SUCCESS;
//     goto Quit;
//
// ReleaseMemory:
//     ExFreePoolWithTag(VolumePathPtr, TAG_DEV2DOS);
//
// Quit:
//     ObDereferenceObject(FileObject);
//     return Status;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
