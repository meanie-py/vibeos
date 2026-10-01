use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::macros::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::ex::*;

use ntkrnl_sys::ddk::{
    _DEVICE_OBJECT, _DRIVER_OBJECT, _DEVICE_NODE, _EXTENDED_DEVOBJ_EXTENSION,
    _IRP, _FILE_OBJECT, _IO_STATUS_BLOCK, _IO_STACK_LOCATION, _SHUTDOWN_ENTRY,
    _VPB, _ERESOURCE, _FILE_OBJECT_EXTENSION, _OBJECT_ATTRIBUTES,
    _UNICODE_STRING, _EVENT_TYPE_NotificationEvent, _EVENT_TYPE_SynchronizationEvent,
    _KWAIT_REASON_Executive,
    PDEVICE_OBJECT, PDRIVER_OBJECT, PFILE_OBJECT, PIO_STATUS_BLOCK, PIRP,
    PVPB, POBJECT_ATTRIBUTES, PSHUTDOWN_ENTRY, PUNICODE_STRING, POBJECT_TYPE,
    ACCESS_MASK, HANDLE, KPROCESSOR_MODE,
    IoAcquireCancelSpinLock, IoReleaseCancelSpinLock, IoAcquireVpbSpinLock, IoReleaseVpbSpinLock,
    IoBuildSynchronousFsdRequest, IofCallDriver,
    ZwOpenFile, ZwClose, KeWaitForSingleObject, KeInitializeEvent, KeInitializeDeviceQueue,
    KeRemoveDeviceQueue, KeRemoveByKeyDeviceQueue, KeInsertDeviceQueue, KeInsertByKeyDeviceQueue,
    ObMakeTemporaryObject, ObCloseHandle, ObDereferenceSecurityDescriptor,
    _DEVOBJ_EXTENSION,
    PACL,
    RtlCreateSecurityDescriptor, RtlSetDaclSecurityDescriptor,
    IopGetDeviceNode, IopFreeDeviceNode, IopCreateVpb, IopRemoveTimerFromTimerList,
    IopSynchronousCall, IopShutdownBaseFileSystems,
    RtlEqualSid, RtlGetAce, ExAcquireResourceExclusiveLite,
    ExFreePool, RtlInitUnicodeString,
};

// ExInterlockedInsertHeadList / ExInterlockedRemoveHeadList: in bindgen on amd64/arm64
// On i386 they are ExfInterlockedInsertHeadList / ExfInterlockedRemoveHeadList (fastcall)
#[cfg(not(target_arch = "x86"))]
use ntkrnl_sys::ddk::{ExInterlockedInsertHeadList, ExInterlockedRemoveHeadList};

#[cfg(target_arch = "x86")]
unsafe fn ExInterlockedInsertHeadList(
    ListHead: PLIST_ENTRY,
    ListEntry: PLIST_ENTRY,
    Lock: *mut KSPIN_LOCK,
) -> PLIST_ENTRY {
    extern "fastcall" {
        fn ExfInterlockedInsertHeadList(
            ListHead: PLIST_ENTRY,
            ListEntry: PLIST_ENTRY,
            Lock: *mut KSPIN_LOCK,
        ) -> PLIST_ENTRY;
    }
    ExfInterlockedInsertHeadList(ListHead, ListEntry, Lock)
}

#[cfg(target_arch = "x86")]
unsafe fn ExInterlockedRemoveHeadList(
    ListHead: PLIST_ENTRY,
    Lock: *mut KSPIN_LOCK,
) -> PLIST_ENTRY {
    extern "fastcall" {
        fn ExfInterlockedRemoveHeadList(
            ListHead: PLIST_ENTRY,
            Lock: *mut KSPIN_LOCK,
        ) -> PLIST_ENTRY;
    }
    ExfInterlockedRemoveHeadList(ListHead, Lock)
}

type DEVICE_TYPE = ULONG;

extern "system" {
    static mut IoDeviceObjectType: POBJECT_TYPE;
    static mut IoFileObjectType: POBJECT_TYPE;
    static SePublicDefaultDacl: PVOID;
    static SePublicDefaultUnrestrictedDacl: PVOID;
    static SePublicOpenDacl: PVOID;
    static SePublicOpenUnrestrictedDacl: PVOID;
    static SeSystemDefaultDacl: PVOID;
    static SeAliasAdminsSid: PVOID;
    static SeWorldSid: PVOID;
}

extern "C" {
    fn CONTAINING_RECORD(ptr: PVOID, ty: usize, field_offset: usize) -> PVOID;
    fn IoGetDevObjExtension(DeviceObject: PDEVICE_OBJECT) -> *mut _EXTENDED_DEVOBJ_EXTENSION;
}
unsafe extern "system" {
    fn KeRaiseIrql(NewIrql: KIRQL, OldIrql: *mut KIRQL);
    fn KeLowerIrql(NewIrql: KIRQL);
}

extern "system" {
    fn PoInitializeDeviceObject(DeviceObjectExtension: *mut _DEVOBJ_EXTENSION);
    fn PoVolumeDevice(DeviceObject: PDEVICE_OBJECT);
    fn PoRemoveVolumeDevice(DeviceObject: PDEVICE_OBJECT);
}

// `ObCreateObject` / `ObInsertObject` / `ObReferenceObjectByHandle` are
// `extern "system"` in `ob/oblife.rs`, `ob/obhandle.rs`, `ob/obref.rs` and
// already work with their original `*mut PVOID` output-parameter shape from
// other call sites (`ex/timer.rs`, `ex/sem.rs`, `ex/mutant.rs`, `ex/evtpair.rs`,
// `lpc/{connect,send,complete}.rs`). The historical `ObCreateObjectAddress_shim`
// / `ObInsertObject_shim` / `ObReferenceObjectByHandle_shim` C trampolines in
// `io_rust_shims.c` repackaged the output as `ULONG_PTR` to dodge an LLVM
// cross-CGU captures(none) attribute propagation bug; the workspace
// `lto = false` + `codegen-units = 256` firewall now keeps the callee body
// in a different CGU than the caller, so the propagation can't happen.
// Re-probed and verified clean 2026-05-06.
extern "system" {
    fn ObCreateObject(
        ProbeMode: KPROCESSOR_MODE, ObjectType: POBJECT_TYPE,
        ObjectAttributes: POBJECT_ATTRIBUTES, OwnershipMode: KPROCESSOR_MODE,
        ParseContext: PVOID, ObjectBodySize: ULONG, PagedPoolCharge: ULONG,
        NonPagedPoolCharge: ULONG, Object: *mut PVOID,
    ) -> NTSTATUS;
    fn ObInsertObject(
        Object: PVOID, AccessState: PVOID, DesiredAccess: ACCESS_MASK,
        ObjectPointerBias: ULONG, NewObject: *mut PVOID, Handle: *mut HANDLE,
    ) -> NTSTATUS;
    fn ObReferenceObjectByHandle(
        Handle: HANDLE, DesiredAccess: ACCESS_MASK, ObjectType: POBJECT_TYPE,
        AccessMode: KPROCESSOR_MODE, Object: *mut PVOID, HandleInformation: PVOID,
    ) -> NTSTATUS;
}

#[inline(always)]
unsafe fn IoCallDriver(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    IofCallDriver(DeviceObject, Irp)
}

use ntkrnl_sys::inline::common::InitializeObjectAttributes;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

const STATUS_NO_SUCH_DEVICE: NTSTATUS = 0xC000000Eu32 as i32;
const STATUS_VOLUME_DISMOUNTED: NTSTATUS = 0xC000026Eu32 as i32;

static mut IopDeviceObjectNumber: ULONG = 0;
#[no_mangle]
pub static mut ShutdownListHead: LIST_ENTRY = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut LastChanceShutdownListHead: LIST_ENTRY = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut ShutdownListLock: KSPIN_LOCK = unsafe { core::mem::zeroed() };

extern "system" {
    static mut IopDiskFileSystemQueueHead: LIST_ENTRY;
    static mut IopCdRomFileSystemQueueHead: LIST_ENTRY;
    static mut IopTapeFileSystemQueueHead: LIST_ENTRY;
    static mut IopDatabaseResource: _ERESOURCE;
}

const DACL_SET: ULONG = 4;

const TAG_SHUTDOWN_ENTRY: ULONG = 0x54554853;
const TAG_IO_TIMER: ULONG = 0x4D544F49;

#[cfg(target_pointer_width = "64")]
const MEMORY_ALLOCATION_ALIGNMENT: usize = 16;
#[cfg(target_pointer_width = "32")]
const MEMORY_ALLOCATION_ALIGNMENT: usize = 8;

const DRVO_INITIALIZED: ULONG = 0x10;
const DRVO_UNLOAD_INVOKED: ULONG = 0x01;
const DRVO_LEGACY_DRIVER: ULONG = 0x02;

const DOE_UNLOAD_PENDING: ULONG = 0x00000001;
const DOE_DELETE_PENDING: ULONG = 0x00000002;
const DOE_REMOVE_PENDING: ULONG = 0x00000004;
const DOE_REMOVE_PROCESSED: ULONG = 0x00000008;
const DOE_START_PENDING: ULONG = 0x00000010;
const DOE_SIO_DEFERRED: ULONG = 0x00000100;
const DOE_SIO_NO_CANCEL: ULONG = 0x00000200;
const DOE_SIO_WITH_KEY: ULONG = 0x40;
const DOE_SIO_NO_KEY: ULONG = 0x20;
const DOE_SIO_CANCELABLE: ULONG = 0x80;

const DO_DEVICE_INITIALIZING: ULONG = 0x00000080;
const DO_EXCLUSIVE: ULONG = 0x00000008;
const DO_DEVICE_HAS_NAME: ULONG = 0x00000040;
const DO_SHUTDOWN_REGISTERED: ULONG = 0x00000800;

const IO_TYPE_DEVICE: USHORT = 3;
const IO_TYPE_DEVICE_OBJECT_EXTENSION: USHORT = 13;

const VPB_MOUNTED: USHORT = 0x0001;

const FO_DIRECT_DEVICE_OPEN: ULONG = 0x0800;
const FO_FILE_OBJECT_HAS_EXTENSION: ULONG = 0x00800000;

const IO_ATTACH_DEVICE_API: ULONG = 0x80000000;

const FILE_READ_DATA: ACCESS_MASK = 0x0001;
const FILE_WRITE_DATA: ACCESS_MASK = 0x0002;
const FILE_READ_ATTRIBUTES: ACCESS_MASK = 0x0080;

const FILE_NON_DIRECTORY_FILE: ULONG = 0x00000040;

const FILE_DEVICE_DISK_FILE_SYSTEM: DEVICE_TYPE = 0x00000008;
const FILE_DEVICE_FILE_SYSTEM: DEVICE_TYPE = 0x00000009;
const FILE_DEVICE_CD_ROM_FILE_SYSTEM: DEVICE_TYPE = 0x00000003;
const FILE_DEVICE_NETWORK_FILE_SYSTEM: DEVICE_TYPE = 0x00000014;
const FILE_DEVICE_TAPE_FILE_SYSTEM: DEVICE_TYPE = 0x00000020;
const FILE_DEVICE_DISK: DEVICE_TYPE = 0x00000007;
const FILE_DEVICE_VIRTUAL_DISK: DEVICE_TYPE = 0x00000024;
const FILE_DEVICE_CD_ROM: DEVICE_TYPE = 0x00000002;
const FILE_DEVICE_TAPE: DEVICE_TYPE = 0x0000001f;
const FILE_DEVICE_DFS_FILE_SYSTEM: DEVICE_TYPE = 0x35;
const FILE_DEVICE_NETWORK: DEVICE_TYPE = 0x00000012;
const FILE_DEVICE_MASS_STORAGE: DEVICE_TYPE = 0x0000002d;

const FILE_FLOPPY_DISKETTE: ULONG = 0x00000004;
const FILE_AUTOGENERATED_DEVICE_NAME: ULONG = 0x00000080;

const SECURITY_DESCRIPTOR_REVISION: ULONG = 1;

const IRP_MJ_SHUTDOWN: UCHAR = 0x10;
const IRP_MJ_PNP: UCHAR = 0x1b;
const IRP_MN_QUERY_DEVICE_RELATIONS: UCHAR = 0x07;

const Executive: i32 = 0;
const NotificationEvent: i32 = 0;
const SynchronizationEvent: i32 = 1;

const LockQueueIoDatabaseLock: ULONG = 0x0000000A;

#[repr(C)]
enum IOP_DEVICE_LIST_OPERATION {
    IopRemove,
    IopAdd,
}

#[repr(C)]
enum SECURITY_DESCRIPTOR_TYPE {
    RestrictedPublic,
    UnrestrictedPublic,
    RestrictedPublicOpen,
    UnrestrictedPublicOpen,
    SystemDefault,
}

#[repr(C)]
struct DEVICE_RELATIONS {
    Count: ULONG,
    Objects: [PDEVICE_OBJECT; 1],
}

type PSECURITY_DESCRIPTOR = PVOID;

#[no_mangle]
pub unsafe extern "system" fn IopReadyDeviceObjects(Driver: PDRIVER_OBJECT) {
    let Driver = opaque_ptr(Driver);
    (*Driver).Flags |= DRVO_INITIALIZED;
    let mut DeviceObject = (*Driver).DeviceObject;
    while !DeviceObject.is_null() {
        (*DeviceObject).Flags &= !DO_DEVICE_INITIALIZING;
        DeviceObject = (*DeviceObject).NextDevice;
    }
}

#[no_mangle]
pub unsafe extern "system" fn IopDeleteDevice(ObjectBody: PVOID) {
    let ObjectBody = opaque_ptr(ObjectBody);
    let DeviceObject = ObjectBody as PDEVICE_OBJECT;
    let DeviceNode = IopGetDeviceNode(DeviceObject);

    if !DeviceNode.is_null() {
        IopFreeDeviceNode(DeviceNode);
    }

    if !(*DeviceObject).DriverObject.is_null() {
        ObDereferenceObject((*DeviceObject).DriverObject as PVOID);
    }
}

unsafe fn IopAttachDeviceToDeviceStackSafe(
    SourceDevice: PDEVICE_OBJECT,
    TargetDevice: PDEVICE_OBJECT,
    AttachedToDeviceObject: *mut PDEVICE_OBJECT,
) -> PDEVICE_OBJECT {
    // The whole attach -- and the *AttachedToDeviceObject store, which is the point of the
    // ...Safe API -- happens under the device database lock (upstream cea19d2e0f6). Without
    // it a filter can get an IRP before its LowerDevice is set and pass it to NULL.
    let OldIrql = KeAcquireQueuedSpinLock(LockQueueIoDatabaseLock);
    let mut AttachedDevice = IoGetAttachedDevice(TargetDevice);
    let SourceDeviceExtension = IoGetDevObjExtension(SourceDevice);

    if ((*AttachedDevice).Flags & DO_DEVICE_INITIALIZING) != 0
        || ((*IoGetDevObjExtension(AttachedDevice)).ExtensionFlags
            & (DOE_UNLOAD_PENDING | DOE_DELETE_PENDING | DOE_REMOVE_PENDING | DOE_REMOVE_PROCESSED))
            != 0
    {
        AttachedDevice = core::ptr::null_mut();
    } else {
        (*AttachedDevice).AttachedDevice = SourceDevice;
        (*AttachedDevice).Spare1 = (*AttachedDevice).Spare1.wrapping_add(1);

        (*SourceDevice).StackSize = (*AttachedDevice).StackSize + 1;
        (*SourceDevice).AlignmentRequirement = (*AttachedDevice).AlignmentRequirement;
        (*SourceDevice).SectorSize = (*AttachedDevice).SectorSize;

        if ((*IoGetDevObjExtension(AttachedDevice)).ExtensionFlags & DOE_START_PENDING) != 0 {
            (*IoGetDevObjExtension(SourceDevice)).ExtensionFlags |= DOE_START_PENDING;
        }

        (*SourceDeviceExtension).AttachedTo = AttachedDevice;
    }

    if !AttachedToDeviceObject.is_null() {
        *AttachedToDeviceObject = AttachedDevice;
    }
    KeReleaseQueuedSpinLock(LockQueueIoDatabaseLock, OldIrql);
    AttachedDevice
}

unsafe fn IoShutdownPnpDevices() {}

#[no_mangle]
pub unsafe extern "system" fn IoShutdownSystem(Phase: ULONG) {
    let mut Event: core::mem::MaybeUninit<_> = core::mem::MaybeUninit::uninit();
    KeInitializeEvent(Event.as_mut_ptr(), _EVENT_TYPE_NotificationEvent, 0u8);
    let Event = Event.assume_init();

    if Phase == 0 {
        IoShutdownPnpDevices();

        let mut ListEntry = ExInterlockedRemoveHeadList(
            core::ptr::addr_of_mut!(ShutdownListHead) as PLIST_ENTRY,
            core::ptr::addr_of_mut!(ShutdownListLock) as *mut _,
        );
        while !ListEntry.is_null() {
            let ShutdownEntry = CONTAINING_RECORD(
                ListEntry as PVOID,
                core::mem::size_of::<_SHUTDOWN_ENTRY>(),
                core::mem::offset_of!(_SHUTDOWN_ENTRY, ShutdownList),
            ) as PSHUTDOWN_ENTRY;

            let DeviceObject = IoGetAttachedDevice((*ShutdownEntry).DeviceObject);

            let mut StatusBlock = core::mem::zeroed::<_IO_STATUS_BLOCK>();
            let Irp = IoBuildSynchronousFsdRequest(
                IRP_MJ_SHUTDOWN as ULONG,
                DeviceObject,
                core::ptr::null_mut(),
                0,
                core::ptr::null_mut(),
                core::ptr::addr_of!(Event) as *mut _,
                core::ptr::addr_of_mut!(StatusBlock),
            );
            if !Irp.is_null() {
                let Status = IoCallDriver(DeviceObject, Irp);
                if Status == STATUS_PENDING {
                    KeWaitForSingleObject(
                        core::ptr::addr_of!(Event) as PVOID,
                        _KWAIT_REASON_Executive,
                        0i32 as i8,
                        0u8,
                        core::ptr::null_mut(),
                    );
                }
            }

            (*(*ShutdownEntry).DeviceObject).Flags &= !DO_SHUTDOWN_REGISTERED;
            ObDereferenceObject((*ShutdownEntry).DeviceObject as PVOID);
            ExFreePoolWithTag(ShutdownEntry as PVOID, TAG_SHUTDOWN_ENTRY);
            KeClearEvent(core::ptr::addr_of!(Event) as *mut _);

            ListEntry = ExInterlockedRemoveHeadList(
                core::ptr::addr_of_mut!(ShutdownListHead) as PLIST_ENTRY,
                core::ptr::addr_of_mut!(ShutdownListLock) as *mut _,
            );
        }
    } else if Phase == 1 {
        ExAcquireResourceExclusiveLite(core::ptr::addr_of_mut!(IopDatabaseResource), 1u8);

        IopShutdownBaseFileSystems(core::ptr::addr_of_mut!(IopDiskFileSystemQueueHead) as PLIST_ENTRY);
        IopShutdownBaseFileSystems(core::ptr::addr_of_mut!(IopCdRomFileSystemQueueHead) as PLIST_ENTRY);
        IopShutdownBaseFileSystems(core::ptr::addr_of_mut!(IopTapeFileSystemQueueHead) as PLIST_ENTRY);

        let mut ListEntry = ExInterlockedRemoveHeadList(
            core::ptr::addr_of_mut!(LastChanceShutdownListHead) as PLIST_ENTRY,
            core::ptr::addr_of_mut!(ShutdownListLock) as *mut _,
        );
        while !ListEntry.is_null() {
            let ShutdownEntry = CONTAINING_RECORD(
                ListEntry as PVOID,
                core::mem::size_of::<_SHUTDOWN_ENTRY>(),
                core::mem::offset_of!(_SHUTDOWN_ENTRY, ShutdownList),
            ) as PSHUTDOWN_ENTRY;

            let DeviceObject = IoGetAttachedDevice((*ShutdownEntry).DeviceObject);

            let mut StatusBlock = core::mem::zeroed::<_IO_STATUS_BLOCK>();
            let Irp = IoBuildSynchronousFsdRequest(
                IRP_MJ_SHUTDOWN as ULONG,
                DeviceObject,
                core::ptr::null_mut(),
                0,
                core::ptr::null_mut(),
                core::ptr::addr_of!(Event) as *mut _,
                core::ptr::addr_of_mut!(StatusBlock),
            );
            if !Irp.is_null() {
                let Status = IoCallDriver(DeviceObject, Irp);
                if Status == STATUS_PENDING {
                    KeWaitForSingleObject(
                        core::ptr::addr_of!(Event) as PVOID,
                        _KWAIT_REASON_Executive,
                        0i32 as i8,
                        0u8,
                        core::ptr::null_mut(),
                    );
                }
            }

            (*(*ShutdownEntry).DeviceObject).Flags &= !DO_SHUTDOWN_REGISTERED;
            ObDereferenceObject((*ShutdownEntry).DeviceObject as PVOID);
            ExFreePoolWithTag(ShutdownEntry as PVOID, TAG_SHUTDOWN_ENTRY);
            KeClearEvent(core::ptr::addr_of!(Event) as *mut _);

            ListEntry = ExInterlockedRemoveHeadList(
                core::ptr::addr_of_mut!(LastChanceShutdownListHead) as PLIST_ENTRY,
                core::ptr::addr_of_mut!(ShutdownListLock) as *mut _,
            );
        }
    }
}

unsafe fn IopGetDeviceObjectPointer(
    ObjectName: *mut _UNICODE_STRING,
    DesiredAccess: ACCESS_MASK,
    FileObject: *mut PFILE_OBJECT,
    DeviceObject: *mut PDEVICE_OBJECT,
    AttachFlag: ULONG,
) -> NTSTATUS {
    let mut ObjectAttributes = core::mem::zeroed::<_OBJECT_ATTRIBUTES>();
    InitializeObjectAttributes(
        core::ptr::addr_of_mut!(ObjectAttributes),
        ObjectName,
        OBJ_KERNEL_HANDLE,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    let mut StatusBlock = core::mem::zeroed::<_IO_STATUS_BLOCK>();
    let mut FileHandle = core::ptr::null_mut();
    let mut Status = ZwOpenFile(
        core::ptr::addr_of_mut!(FileHandle),
        DesiredAccess,
        core::ptr::addr_of_mut!(ObjectAttributes),
        core::ptr::addr_of_mut!(StatusBlock),
        0,
        FILE_NON_DIRECTORY_FILE | AttachFlag,
    );
    if Status < 0 {
        return Status;
    }

    let mut LocalFileObject: PVOID = core::ptr::null_mut();
    Status = ObReferenceObjectByHandle(
        FileHandle,
        0,
        IoFileObjectType,
        0i32 as KPROCESSOR_MODE,
        &mut LocalFileObject,
        core::ptr::null_mut(),
    );
    let LocalFileObject = LocalFileObject as *mut _FILE_OBJECT;
    if Status >= 0 {
        *DeviceObject = IoGetRelatedDeviceObject(LocalFileObject);
        *FileObject = LocalFileObject;
    }

    ZwClose(FileHandle);
    Status
}

unsafe fn IopGetLowestDevice(DeviceObject: PDEVICE_OBJECT) -> PDEVICE_OBJECT {
    let mut LowestDevice = DeviceObject;
    let mut DeviceExtension = IoGetDevObjExtension(LowestDevice);

    while !(*DeviceExtension).AttachedTo.is_null() {
        LowestDevice = (*DeviceExtension).AttachedTo;
        DeviceExtension = IoGetDevObjExtension(LowestDevice);
    }

    LowestDevice
}

unsafe fn IopEditDeviceList(
    DriverObject: PDRIVER_OBJECT,
    DeviceObject: PDEVICE_OBJECT,
    Type: IOP_DEVICE_LIST_OPERATION,
) {
    let OldIrql = KeAcquireQueuedSpinLock(LockQueueIoDatabaseLock);

    match Type {
        IOP_DEVICE_LIST_OPERATION::IopRemove => {
            let mut Previous = (*(*DeviceObject).DriverObject).DeviceObject;
            if Previous == DeviceObject {
                (*(*DeviceObject).DriverObject).DeviceObject = (*DeviceObject).NextDevice;
            } else {
                while (*Previous).NextDevice != DeviceObject {
                    if (*Previous).NextDevice.is_null() {
                        KeReleaseQueuedSpinLock(LockQueueIoDatabaseLock, OldIrql);
                        return;
                    }
                    Previous = (*Previous).NextDevice;
                }
                (*Previous).NextDevice = (*DeviceObject).NextDevice;
            }
        }
        IOP_DEVICE_LIST_OPERATION::IopAdd => {
            (*DeviceObject).NextDevice = (*DriverObject).DeviceObject;
            (*DriverObject).DeviceObject = DeviceObject;
        }
    }

    KeReleaseQueuedSpinLock(LockQueueIoDatabaseLock, OldIrql);
}

#[no_mangle]
pub unsafe extern "system" fn IopUnloadDevice(DeviceObject: PDEVICE_OBJECT) {
    let DeviceObject = opaque_ptr(DeviceObject);
    let DriverObject = (*DeviceObject).DriverObject;
    let ThisExtension = IoGetDevObjExtension(DeviceObject);

    if ((*ThisExtension).ExtensionFlags & DOE_DELETE_PENDING) != 0 {
        if !(*DeviceObject).AttachedDevice.is_null() {
            return;
        }

        if (*DeviceObject).ReferenceCount != 0 {
            return;
        }

        if !(*DeviceObject).SecurityDescriptor.is_null() {
            ObDereferenceSecurityDescriptor((*DeviceObject).SecurityDescriptor, 1);
        }

        IopEditDeviceList(
            (*DeviceObject).DriverObject,
            DeviceObject,
            IOP_DEVICE_LIST_OPERATION::IopRemove,
        );

        ObDereferenceObject(DeviceObject as PVOID);
    }

    if ((*DriverObject).Flags & DRVO_LEGACY_DRIVER) != 0 {
        return;
    }

    if ((*DriverObject).Flags & DRVO_UNLOAD_INVOKED) != 0 {
        return;
    }

    if (*DriverObject).DriverUnload.is_none() {
        return;
    }

    if !(*DriverObject).DeviceObject.is_null() {
        return;
    }

    (*DriverObject).Flags |= DRVO_UNLOAD_INVOKED;

    ((*DriverObject).DriverUnload.unwrap())(DriverObject);

    ObMakeTemporaryObject(DriverObject as PVOID);
}

#[no_mangle]
pub unsafe extern "system" fn IopDereferenceDeviceObject(DeviceObject: PDEVICE_OBJECT, ForceUnload: BOOLEAN) {
    let DeviceObject = opaque_ptr(DeviceObject);
    InterlockedDecrement(core::ptr::addr_of_mut!((*DeviceObject).ReferenceCount));

    if (*DeviceObject).ReferenceCount == 0
        && ((*IoGetDevObjExtension(DeviceObject)).ExtensionFlags & DOE_DELETE_PENDING) != 0
    {
        IopUnloadDevice(DeviceObject);
    }
}

unsafe fn IopStartNextPacketByKey(DeviceObject: PDEVICE_OBJECT, Cancelable: BOOLEAN, Key: ULONG) {
    let mut OldIrql = 0u8;

    if Cancelable != 0 {
        IoAcquireCancelSpinLock(core::ptr::addr_of_mut!(OldIrql));
    }

    (*DeviceObject).CurrentIrp = core::ptr::null_mut();

    let Entry = KeRemoveByKeyDeviceQueue(
        core::ptr::addr_of_mut!((*DeviceObject).DeviceQueue),
        Key,
    );
    if !Entry.is_null() {
        let Irp = CONTAINING_RECORD(
            Entry as PVOID,
            core::mem::size_of::<_IRP>(),
            core::mem::offset_of!(_IRP, Tail),
        ) as PIRP;
        (*DeviceObject).CurrentIrp = Irp;

        if Cancelable != 0 {
            if ((*IoGetDevObjExtension(DeviceObject)).StartIoFlags & DOE_SIO_NO_CANCEL) != 0 {
                (*Irp).CancelRoutine = None;
            }
            IoReleaseCancelSpinLock(OldIrql);
        }

        ((*(*DeviceObject).DriverObject).DriverStartIo.unwrap())(DeviceObject, Irp);
    } else {
        if Cancelable != 0 {
            IoReleaseCancelSpinLock(OldIrql);
        }
    }
}

unsafe fn IopStartNextPacket(DeviceObject: PDEVICE_OBJECT, Cancelable: BOOLEAN) {
    let mut OldIrql = 0u8;

    if Cancelable != 0 {
        IoAcquireCancelSpinLock(core::ptr::addr_of_mut!(OldIrql));
    }

    (*DeviceObject).CurrentIrp = core::ptr::null_mut();

    let Entry = KeRemoveDeviceQueue(core::ptr::addr_of_mut!((*DeviceObject).DeviceQueue));
    if !Entry.is_null() {
        let Irp = CONTAINING_RECORD(
            Entry as PVOID,
            core::mem::size_of::<_IRP>(),
            core::mem::offset_of!(_IRP, Tail),
        ) as PIRP;
        (*DeviceObject).CurrentIrp = Irp;

        if Cancelable != 0 {
            if ((*IoGetDevObjExtension(DeviceObject)).StartIoFlags & DOE_SIO_NO_CANCEL) != 0 {
                (*Irp).CancelRoutine = None;
            }
            IoReleaseCancelSpinLock(OldIrql);
        }

        ((*(*DeviceObject).DriverObject).DriverStartIo.unwrap())(DeviceObject, Irp);
    } else {
        if Cancelable != 0 {
            IoReleaseCancelSpinLock(OldIrql);
        }
    }
}

unsafe fn IopStartNextPacketByKeyEx(DeviceObject: PDEVICE_OBJECT, Key: ULONG, Flags: ULONG) {
    let DeviceExtension = IoGetDevObjExtension(DeviceObject);
    let mut CurrentKey = Key;
    let mut CurrentFlags = Flags;

    loop {
        if InterlockedIncrement(core::ptr::addr_of_mut!((*DeviceExtension).StartIoCount)) > 1 {
            (*DeviceExtension).StartIoFlags |= CurrentFlags;
            (*DeviceExtension).StartIoKey = CurrentKey as i32;
        } else {
            (*DeviceExtension).StartIoFlags &= !(DOE_SIO_WITH_KEY | DOE_SIO_NO_KEY | DOE_SIO_CANCELABLE);
            (*DeviceExtension).StartIoKey = 0;

            if (Flags & DOE_SIO_WITH_KEY) != 0 {
                IopStartNextPacketByKey(
                    DeviceObject,
                    if (Flags & DOE_SIO_CANCELABLE) != 0 { 1u8 } else { 0u8 },
                    CurrentKey,
                );
            } else if (Flags & DOE_SIO_NO_KEY) != 0 {
                IopStartNextPacket(
                    DeviceObject,
                    if (Flags & DOE_SIO_CANCELABLE) != 0 { 1u8 } else { 0u8 },
                );
            }
        }

        if InterlockedDecrement(core::ptr::addr_of_mut!((*DeviceExtension).StartIoCount)) == 0 {
            CurrentKey = (*DeviceExtension).StartIoKey as ULONG;
            CurrentFlags = (*DeviceExtension).StartIoFlags & (DOE_SIO_WITH_KEY | DOE_SIO_NO_KEY | DOE_SIO_CANCELABLE);

            if (CurrentFlags & (DOE_SIO_WITH_KEY | DOE_SIO_NO_KEY)) == 0 {
                break;
            }
        } else {
            break;
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn IopGetRelatedTargetDevice(
    FileObject: PFILE_OBJECT,
    DeviceNode: *mut *mut _DEVICE_NODE,
) -> NTSTATUS {
    let FileObject = opaque_ptr(FileObject);
    let DeviceNode = opaque_ptr(DeviceNode);
    let mut Stack = core::mem::zeroed::<_IO_STACK_LOCATION>();
    let DeviceObject = IoGetRelatedDeviceObject(FileObject);
    if DeviceObject.is_null() {
        return STATUS_NO_SUCH_DEVICE;
    }

    Stack.MajorFunction = IRP_MJ_PNP;
    Stack.MinorFunction = IRP_MN_QUERY_DEVICE_RELATIONS;
    Stack.Parameters.QueryDeviceRelations.Type = 0;
    Stack.FileObject = FileObject;

    let mut DeviceRelations: PVOID = core::ptr::null_mut();
    let Status = IopSynchronousCall(
        DeviceObject,
        &mut Stack,
        &mut DeviceRelations,
    );
    if Status < 0 {
        return Status;
    }

    *DeviceNode = IopGetDeviceNode((*(DeviceRelations as *mut DEVICE_RELATIONS)).Objects[0]);
    let Status = if (*DeviceNode).is_null() {
        STATUS_NO_SUCH_DEVICE
    } else {
        STATUS_SUCCESS
    };

    ExFreePool(DeviceRelations as PVOID);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn IopVerifyDeviceObjectOnStack(
    BaseDeviceObject: PDEVICE_OBJECT,
    TopDeviceObjectHint: PDEVICE_OBJECT,
) -> BOOLEAN {
    let BaseDeviceObject = opaque_ptr(BaseDeviceObject);
    let TopDeviceObjectHint = opaque_ptr(TopDeviceObjectHint);
    let mut Result = 0u8;
    let OldIrql = KeAcquireQueuedSpinLock(LockQueueIoDatabaseLock);
    let mut LoopObject = BaseDeviceObject;
    loop {
        if LoopObject == TopDeviceObjectHint {
            Result = 1u8;
            break;
        }
        if LoopObject.is_null() {
            break;
        }
        LoopObject = (*LoopObject).AttachedDevice;
    }
    KeReleaseQueuedSpinLock(LockQueueIoDatabaseLock, OldIrql);
    Result
}

unsafe fn IopCreateSecurityDescriptorPerType(
    SecurityDescriptor: PSECURITY_DESCRIPTOR,
    Type: SECURITY_DESCRIPTOR_TYPE,
    OutputFlags: *mut ULONG,
) -> NTSTATUS {
    let Dacl = match Type {
        SECURITY_DESCRIPTOR_TYPE::RestrictedPublic => SePublicDefaultDacl,
        SECURITY_DESCRIPTOR_TYPE::UnrestrictedPublic => SePublicDefaultUnrestrictedDacl,
        SECURITY_DESCRIPTOR_TYPE::RestrictedPublicOpen => SePublicOpenDacl,
        SECURITY_DESCRIPTOR_TYPE::UnrestrictedPublicOpen => SePublicOpenUnrestrictedDacl,
        SECURITY_DESCRIPTOR_TYPE::SystemDefault => SeSystemDefaultDacl,
    };

    let Status = RtlCreateSecurityDescriptor(SecurityDescriptor, SECURITY_DESCRIPTOR_REVISION);
    let Status = RtlSetDaclSecurityDescriptor(SecurityDescriptor, 1u8, Dacl as PACL, 0u8);

    if !OutputFlags.is_null() {
        *OutputFlags |= DACL_SET;
    }
    Status
}

unsafe fn IopCreateDefaultDeviceSecurityDescriptor(
    DeviceType: DEVICE_TYPE,
    DeviceCharacteristics: ULONG,
    HasDeviceName: BOOLEAN,
    SecurityDescriptor: PSECURITY_DESCRIPTOR,
    OutputDacl: *mut PACL,
    OutputFlags: *mut ULONG,
) -> PSECURITY_DESCRIPTOR {
    if !OutputFlags.is_null() {
        *OutputFlags = 0;
    }
    *OutputDacl = core::ptr::null_mut();

    let Status = if DeviceType == FILE_DEVICE_TAPE_FILE_SYSTEM
        || DeviceType == FILE_DEVICE_CD_ROM_FILE_SYSTEM
        || DeviceType == FILE_DEVICE_DISK_FILE_SYSTEM
        || DeviceType == FILE_DEVICE_FILE_SYSTEM
    {
        IopCreateSecurityDescriptorPerType(
            SecurityDescriptor,
            SECURITY_DESCRIPTOR_TYPE::UnrestrictedPublic,
            OutputFlags,
        )
    } else if (DeviceType != FILE_DEVICE_VIRTUAL_DISK
        && DeviceType != FILE_DEVICE_MASS_STORAGE
        && DeviceType != FILE_DEVICE_CD_ROM
        && DeviceType != FILE_DEVICE_DISK
        && DeviceType != FILE_DEVICE_DFS_FILE_SYSTEM
        && DeviceType != FILE_DEVICE_NETWORK
        && DeviceType != FILE_DEVICE_NETWORK_FILE_SYSTEM)
        || (HasDeviceName != 0 && (DeviceCharacteristics & FILE_FLOPPY_DISKETTE) != 0)
    {
        IopCreateSecurityDescriptorPerType(
            SecurityDescriptor,
            SECURITY_DESCRIPTOR_TYPE::UnrestrictedPublicOpen,
            OutputFlags,
        )
    } else {
        let acl_size = (*(SePublicDefaultUnrestrictedDacl as *const ntkrnl_sys::ddk::_ACL)).AclSize as usize;
        let Dacl = ExAllocatePoolWithTag(
            PagedPool,
            acl_size,
            TAG_SHUTDOWN_ENTRY,
        ) as PACL;
        if Dacl.is_null() {
            return core::ptr::null_mut();
        }

        core::ptr::copy_nonoverlapping(
            SePublicDefaultUnrestrictedDacl as *const u8,
            Dacl as *mut u8,
            acl_size,
        );

        let mut AceId = 0u32;
        let mut AdminsSet = 0u8;
        let mut WorldSet = 0u8;
        loop {
            let mut Ace = core::ptr::null_mut();
            if RtlGetAce(Dacl as PACL, AceId, core::ptr::addr_of_mut!(Ace)) < 0 {
                break;
            }

            let SidStart = (Ace as usize + 8) as PVOID;
            if RtlEqualSid(SeAliasAdminsSid, SidStart) != 0 {
                let mask_ptr = (Ace as usize + 4) as *mut ACCESS_MASK;
                *mask_ptr |= GENERIC_READ | GENERIC_WRITE | GENERIC_EXECUTE;
                AdminsSet = 1u8;
            }

            if DeviceType == FILE_DEVICE_CD_ROM && RtlEqualSid(SeWorldSid, SidStart) != 0 {
                let mask_ptr = (Ace as usize + 4) as *mut ACCESS_MASK;
                *mask_ptr |= GENERIC_READ;
                WorldSet = 1u8;
            }

            AceId += 1;
        }

        RtlCreateSecurityDescriptor(SecurityDescriptor, SECURITY_DESCRIPTOR_REVISION);
        let status = RtlSetDaclSecurityDescriptor(SecurityDescriptor, 1u8, Dacl as PACL, 0u8);

        if !OutputFlags.is_null() {
            *OutputFlags |= DACL_SET;
        }

        *OutputDacl = Dacl;
        status
    };

    if Status < 0 {
        core::ptr::null_mut()
    } else {
        SecurityDescriptor
    }
}

#[no_mangle]
pub unsafe extern "system" fn IoAttachDevice(
    SourceDevice: PDEVICE_OBJECT,
    TargetDeviceName: *mut _UNICODE_STRING,
    AttachedDevice: *mut PDEVICE_OBJECT,
) -> NTSTATUS {
    let SourceDevice = opaque_ptr(SourceDevice);
    let TargetDeviceName = opaque_ptr(TargetDeviceName);
    let AttachedDevice = opaque_ptr(AttachedDevice);
    let mut FileObject = core::ptr::null_mut();
    let mut TargetDevice = core::ptr::null_mut();

    let Status = IopGetDeviceObjectPointer(
        TargetDeviceName,
        FILE_READ_ATTRIBUTES,
        core::ptr::addr_of_mut!(FileObject),
        core::ptr::addr_of_mut!(TargetDevice),
        IO_ATTACH_DEVICE_API,
    );
    if Status < 0 {
        return Status;
    }

    let Status = IoAttachDeviceToDeviceStackSafe(SourceDevice, TargetDevice, AttachedDevice);

    ObDereferenceObject(FileObject as PVOID);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn IoAttachDeviceByPointer(
    SourceDevice: PDEVICE_OBJECT,
    TargetDevice: PDEVICE_OBJECT,
) -> NTSTATUS {
    let SourceDevice = opaque_ptr(SourceDevice);
    let TargetDevice = opaque_ptr(TargetDevice);
    let AttachedDevice = IoAttachDeviceToDeviceStack(SourceDevice, TargetDevice);
    if AttachedDevice.is_null() {
        STATUS_NO_SUCH_DEVICE
    } else {
        STATUS_SUCCESS
    }
}

#[no_mangle]
pub unsafe extern "system" fn IoAttachDeviceToDeviceStack(
    SourceDevice: PDEVICE_OBJECT,
    TargetDevice: PDEVICE_OBJECT,
) -> PDEVICE_OBJECT {
    let SourceDevice = opaque_ptr(SourceDevice);
    let TargetDevice = opaque_ptr(TargetDevice);
    IopAttachDeviceToDeviceStackSafe(SourceDevice, TargetDevice, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "system" fn IoAttachDeviceToDeviceStackSafe(
    SourceDevice: PDEVICE_OBJECT,
    TargetDevice: PDEVICE_OBJECT,
    AttachedToDeviceObject: *mut PDEVICE_OBJECT,
) -> NTSTATUS {
    let SourceDevice = opaque_ptr(SourceDevice);
    let TargetDevice = opaque_ptr(TargetDevice);
    let AttachedToDeviceObject = opaque_ptr(AttachedToDeviceObject);
    if IopAttachDeviceToDeviceStackSafe(SourceDevice, TargetDevice, AttachedToDeviceObject).is_null() {
        STATUS_NO_SUCH_DEVICE
    } else {
        STATUS_SUCCESS
    }
}

#[inline(never)]
unsafe fn IopFinishCreatedDeviceObject(
    mut CreatedDeviceObject: *mut _DEVICE_OBJECT,
    TotalSize: usize,
    DriverObject: PDRIVER_OBJECT,
    DeviceExtensionSize: ULONG,
    AlignedDeviceExtensionSize: usize,
    DeviceName: PUNICODE_STRING,
    DeviceType: DEVICE_TYPE,
    DeviceCharacteristics: ULONG,
    Exclusive: BOOLEAN,
    DeviceObject: *mut PDEVICE_OBJECT,
) -> NTSTATUS {
    // Keep the post-ObCreateObject path in a separate function: on i386 LLVM
    // otherwise folds the caller's success path into the null-object cleanup.
    if CreatedDeviceObject.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    let mut Status: NTSTATUS;
    core::ptr::write_bytes(CreatedDeviceObject.cast::<u8>(), 0, TotalSize);

    (*CreatedDeviceObject).Type = IO_TYPE_DEVICE as i16;
    (*CreatedDeviceObject).Size =
        (core::mem::size_of::<_DEVICE_OBJECT>() + DeviceExtensionSize as usize) as USHORT;

    let DeviceObjectExtension = CreatedDeviceObject
        .add(1)
        .cast::<u8>()
        .add(AlignedDeviceExtensionSize) as *mut _EXTENDED_DEVOBJ_EXTENSION;

    (*DeviceObjectExtension).Type = IO_TYPE_DEVICE_OBJECT_EXTENSION as i16;
    (*DeviceObjectExtension).Size = 0;
    PoInitializeDeviceObject(DeviceObjectExtension as *mut _);

    (*DeviceObjectExtension).DeviceObject = CreatedDeviceObject;
    (*CreatedDeviceObject).DeviceObjectExtension = DeviceObjectExtension as *mut _;

    (*CreatedDeviceObject).DeviceType = DeviceType;
    (*CreatedDeviceObject).Characteristics = DeviceCharacteristics;
    (*CreatedDeviceObject).DeviceExtension = if DeviceExtensionSize != 0 {
        CreatedDeviceObject.add(1) as PVOID
    } else {
        core::ptr::null_mut()
    };
    (*CreatedDeviceObject).StackSize = 1;
    (*CreatedDeviceObject).AlignmentRequirement = 0;

    (*CreatedDeviceObject).Flags = DO_DEVICE_INITIALIZING;
    if Exclusive != 0 {
        (*CreatedDeviceObject).Flags |= DO_EXCLUSIVE;
    }
    if !DeviceName.is_null() {
        (*CreatedDeviceObject).Flags |= DO_DEVICE_HAS_NAME;
    }

    if (*CreatedDeviceObject).DeviceType == FILE_DEVICE_DISK
        || (*CreatedDeviceObject).DeviceType == FILE_DEVICE_VIRTUAL_DISK
        || (*CreatedDeviceObject).DeviceType == FILE_DEVICE_CD_ROM
        || (*CreatedDeviceObject).DeviceType == FILE_DEVICE_TAPE
    {
        Status = IopCreateVpb(CreatedDeviceObject);
        if Status < 0 {
            ObDereferenceObject(CreatedDeviceObject as PVOID);
            return Status;
        }

        KeInitializeEvent(
            core::ptr::addr_of_mut!((*CreatedDeviceObject).DeviceLock) as *mut _,
            _EVENT_TYPE_SynchronizationEvent,
            1u8,
        );
    }

    match DeviceType {
        FILE_DEVICE_DISK_FILE_SYSTEM | FILE_DEVICE_DISK | FILE_DEVICE_VIRTUAL_DISK => {
            (*CreatedDeviceObject).SectorSize = 512;
        }
        FILE_DEVICE_CD_ROM_FILE_SYSTEM => {
            (*CreatedDeviceObject).SectorSize = 2048;
        }
        _ => {}
    }

    if (*CreatedDeviceObject).DeviceType == FILE_DEVICE_DISK_FILE_SYSTEM
        || (*CreatedDeviceObject).DeviceType == FILE_DEVICE_FILE_SYSTEM
        || (*CreatedDeviceObject).DeviceType == FILE_DEVICE_CD_ROM_FILE_SYSTEM
        || (*CreatedDeviceObject).DeviceType == FILE_DEVICE_NETWORK_FILE_SYSTEM
        || (*CreatedDeviceObject).DeviceType == FILE_DEVICE_TAPE_FILE_SYSTEM
    {
        InitializeListHead(
            core::ptr::addr_of_mut!((*CreatedDeviceObject).Queue.ListEntry) as PLIST_ENTRY,
        );
    } else {
        KeInitializeDeviceQueue(core::ptr::addr_of_mut!((*CreatedDeviceObject).DeviceQueue));
    }

    let mut TempHandle: HANDLE = core::ptr::null_mut();
    let mut InsertedDeviceObject: PVOID = core::ptr::null_mut();
    Status = ObInsertObject(
        CreatedDeviceObject as PVOID,
        core::ptr::null_mut(),
        FILE_READ_DATA | FILE_WRITE_DATA,
        1,
        &mut InsertedDeviceObject,
        &mut TempHandle,
    );
    if Status < 0 {
        return Status;
    }
    CreatedDeviceObject = InsertedDeviceObject as *mut _DEVICE_OBJECT;

    ObReferenceObject(DriverObject as PVOID);
    (*CreatedDeviceObject).DriverObject = DriverObject;
    IopEditDeviceList(DriverObject, CreatedDeviceObject, IOP_DEVICE_LIST_OPERATION::IopAdd);

    if !(*CreatedDeviceObject).Vpb.is_null() {
        PoVolumeDevice(CreatedDeviceObject);
    }

    ObCloseHandle(TempHandle, 0i32 as i8);
    *DeviceObject = CreatedDeviceObject;
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn IoCreateDevice(
    DriverObject: PDRIVER_OBJECT,
    DeviceExtensionSize: ULONG,
    DeviceName: *mut _UNICODE_STRING,
    DeviceType: DEVICE_TYPE,
    DeviceCharacteristics: ULONG,
    Exclusive: BOOLEAN,
    DeviceObject: *mut PDEVICE_OBJECT,
) -> NTSTATUS {
    let DriverObject = opaque_ptr(DriverObject);
    let DeviceName = opaque_ptr(DeviceName);
    let DeviceObject = opaque_ptr(DeviceObject);
    let mut AutoNameBuffer = [0u16; 20];
    let mut AutoName = core::mem::zeroed::<_UNICODE_STRING>();
    let mut DeviceName = DeviceName;

    if (DeviceCharacteristics & FILE_AUTOGENERATED_DEVICE_NAME) != 0 {
        crate::swprintf!(
            AutoNameBuffer.as_mut_ptr(),
            AutoNameBuffer.len(),
            crate::wstr!("\\Device\\%08lx"),
            InterlockedIncrementUL(core::ptr::addr_of_mut!(IopDeviceObjectNumber) as *mut ULONG),
        );
        RtlInitUnicodeString(core::ptr::addr_of_mut!(AutoName), AutoNameBuffer.as_ptr());
        DeviceName = core::ptr::addr_of_mut!(AutoName);
    }

    let mut Dacl = core::ptr::null_mut();
    let mut SecurityDescriptor = [0u8; 256];
    let ReturnedSD = IopCreateDefaultDeviceSecurityDescriptor(
        DeviceType,
        DeviceCharacteristics,
        if !DeviceName.is_null() { 1u8 } else { 0u8 },
        SecurityDescriptor.as_mut_ptr() as PSECURITY_DESCRIPTOR,
        core::ptr::addr_of_mut!(Dacl),
        core::ptr::null_mut(),
    );

    let mut ObjectAttributes = core::mem::zeroed::<_OBJECT_ATTRIBUTES>();
    InitializeObjectAttributes(
        core::ptr::addr_of_mut!(ObjectAttributes),
        DeviceName,
        OBJ_KERNEL_HANDLE,
        core::ptr::null_mut(),
        ReturnedSD,
    );

    if Exclusive != 0 {
        ObjectAttributes.Attributes |= OBJ_EXCLUSIVE;
    }
    if !DeviceName.is_null() {
        ObjectAttributes.Attributes |= OBJ_PERMANENT;
    }

    let AlignedDeviceExtensionSize = ALIGN_UP_BY(DeviceExtensionSize as usize, MEMORY_ALLOCATION_ALIGNMENT);
    let TotalSize = AlignedDeviceExtensionSize
        + core::mem::size_of::<_DEVICE_OBJECT>()
        + core::mem::size_of::<_EXTENDED_DEVOBJ_EXTENSION>();

    *DeviceObject = core::ptr::null_mut();
    let mut CreatedDeviceObject: PVOID = core::ptr::null_mut();
    let mut Status: NTSTATUS = ObCreateObject(
        0i32 as KPROCESSOR_MODE,
        IoDeviceObjectType,
        &mut ObjectAttributes,
        0i32 as KPROCESSOR_MODE,
        core::ptr::null_mut(),
        TotalSize as ULONG,
        0,
        0,
        &mut CreatedDeviceObject,
    );
    let CreatedDeviceObject = CreatedDeviceObject as *mut _DEVICE_OBJECT;
    if Status < 0 {
        if !Dacl.is_null() {
            ExFreePoolWithTag(Dacl as PVOID, TAG_SHUTDOWN_ENTRY);
        }
        return Status;
    }
    Status = IopFinishCreatedDeviceObject(
        CreatedDeviceObject,
        TotalSize,
        DriverObject,
        DeviceExtensionSize,
        AlignedDeviceExtensionSize,
        DeviceName,
        DeviceType,
        DeviceCharacteristics,
        Exclusive,
        DeviceObject,
    );
    if Status < 0 {
        if !Dacl.is_null() {
            ExFreePoolWithTag(Dacl as PVOID, TAG_SHUTDOWN_ENTRY);
        }
        return Status;
    }

    if !Dacl.is_null() {
        ExFreePoolWithTag(Dacl as PVOID, TAG_SHUTDOWN_ENTRY);
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn IoDeleteDevice(DeviceObject: PDEVICE_OBJECT) {
    let DeviceObject = opaque_ptr(DeviceObject);
    if ((*DeviceObject).Flags & DO_SHUTDOWN_REGISTERED) != 0 {
        IoUnregisterShutdownNotification(DeviceObject);
    }

    let Timer = (*DeviceObject).Timer;
    if !Timer.is_null() {
        IopRemoveTimerFromTimerList(Timer);
        ExFreePoolWithTag(Timer as PVOID, TAG_IO_TIMER);
    }

    if ((*DeviceObject).Flags & DO_DEVICE_HAS_NAME) != 0 {
        ObMakeTemporaryObject(DeviceObject as PVOID);
    }

    (*IoGetDevObjExtension(DeviceObject)).ExtensionFlags |= DOE_DELETE_PENDING;

    if !(*DeviceObject).Vpb.is_null() {
        PoRemoveVolumeDevice(DeviceObject);
    }

    if (*DeviceObject).ReferenceCount == 0 {
        IopUnloadDevice(DeviceObject);
    }
}

#[no_mangle]
pub unsafe extern "system" fn IoDetachDevice(TargetDevice: PDEVICE_OBJECT) {
    let TargetDevice = opaque_ptr(TargetDevice);
    let DeviceExtension = IoGetDevObjExtension((*TargetDevice).AttachedDevice);
    (*DeviceExtension).AttachedTo = core::ptr::null_mut();
    (*TargetDevice).AttachedDevice = core::ptr::null_mut();

    if ((*IoGetDevObjExtension(TargetDevice)).ExtensionFlags & DOE_DELETE_PENDING) != 0
        && (*TargetDevice).ReferenceCount == 0
    {
        IopUnloadDevice(TargetDevice);
    }
}

#[no_mangle]
pub unsafe extern "system" fn IoEnumerateDeviceObjectList(
    DriverObject: PDRIVER_OBJECT,
    DeviceObjectList: *mut PDEVICE_OBJECT,
    DeviceObjectListSize: ULONG,
    ActualNumberDeviceObjects: *mut ULONG,
) -> NTSTATUS {
    let DriverObject = opaque_ptr(DriverObject);
    let DeviceObjectList = opaque_ptr(DeviceObjectList);
    let ActualNumberDeviceObjects = opaque_ptr(ActualNumberDeviceObjects);
    let mut ActualDevices = 1u32;
    let mut CurrentDevice = (*DriverObject).DeviceObject;

    let OldIrql = KeAcquireQueuedSpinLock(LockQueueIoDatabaseLock);

    while !(*CurrentDevice).NextDevice.is_null() {
        CurrentDevice = (*CurrentDevice).NextDevice;
        ActualDevices += 1;
    }

    CurrentDevice = (*DriverObject).DeviceObject;

    *ActualNumberDeviceObjects = ActualDevices;

    if (ActualDevices * core::mem::size_of::<PDEVICE_OBJECT>() as u32) > DeviceObjectListSize {
        KeReleaseQueuedSpinLock(LockQueueIoDatabaseLock, OldIrql);
        return STATUS_BUFFER_TOO_SMALL;
    }

    if !DeviceObjectList.is_null() {
        let mut idx = 0usize;
        while ActualDevices > 0 {
            ObReferenceObject(CurrentDevice as PVOID);
            *DeviceObjectList.add(idx) = CurrentDevice;
            CurrentDevice = (*CurrentDevice).NextDevice;
            ActualDevices -= 1;
            idx += 1;
        }
    }

    KeReleaseQueuedSpinLock(LockQueueIoDatabaseLock, OldIrql);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn IoGetAttachedDevice(DeviceObject: PDEVICE_OBJECT) -> PDEVICE_OBJECT {
    let DeviceObject = opaque_ptr(DeviceObject);
    let mut DeviceObject = DeviceObject;
    while !(*DeviceObject).AttachedDevice.is_null() {
        DeviceObject = (*DeviceObject).AttachedDevice;
    }
    DeviceObject
}

#[no_mangle]
pub unsafe extern "system" fn IoGetAttachedDeviceReference(DeviceObject: PDEVICE_OBJECT) -> PDEVICE_OBJECT {
    let DeviceObject = opaque_ptr(DeviceObject);
    // Walk and reference under the device database lock (upstream b364f702d2f), or a
    // concurrent detach can hand back a device that is going away.
    let OldIrql = KeAcquireQueuedSpinLock(LockQueueIoDatabaseLock);
    let DeviceObject = IoGetAttachedDevice(DeviceObject);
    ObReferenceObject(DeviceObject as PVOID);
    KeReleaseQueuedSpinLock(LockQueueIoDatabaseLock, OldIrql);
    DeviceObject
}

#[no_mangle]
pub unsafe extern "system" fn IoGetDeviceAttachmentBaseRef(DeviceObject: PDEVICE_OBJECT) -> PDEVICE_OBJECT {
    let DeviceObject = opaque_ptr(DeviceObject);
    // Walk and reference under the device database lock (upstream b364f702d2f), or a
    // concurrent detach can hand back a device that is going away.
    let OldIrql = KeAcquireQueuedSpinLock(LockQueueIoDatabaseLock);
    let DeviceObject = IopGetLowestDevice(DeviceObject);
    ObReferenceObject(DeviceObject as PVOID);
    KeReleaseQueuedSpinLock(LockQueueIoDatabaseLock, OldIrql);
    DeviceObject
}

#[no_mangle]
pub unsafe extern "system" fn IoGetDeviceObjectPointer(
    ObjectName: *mut _UNICODE_STRING,
    DesiredAccess: ACCESS_MASK,
    FileObject: *mut PFILE_OBJECT,
    DeviceObject: *mut PDEVICE_OBJECT,
) -> NTSTATUS {
    let ObjectName = opaque_ptr(ObjectName);
    let FileObject = opaque_ptr(FileObject);
    let DeviceObject = opaque_ptr(DeviceObject);
    IopGetDeviceObjectPointer(ObjectName, DesiredAccess, FileObject, DeviceObject, 0)
}

#[no_mangle]
pub unsafe extern "system" fn IoGetDiskDeviceObject(
    FileSystemDeviceObject: PDEVICE_OBJECT,
    DiskDeviceObject: *mut PDEVICE_OBJECT,
) -> NTSTATUS {
    let FileSystemDeviceObject = opaque_ptr(FileSystemDeviceObject);
    let DiskDeviceObject = opaque_ptr(DiskDeviceObject);
    if (*FileSystemDeviceObject).Vpb.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    let mut OldIrql = 0u8;
    IoAcquireVpbSpinLock(core::ptr::addr_of_mut!(OldIrql));

    let DeviceExtension = IoGetDevObjExtension(FileSystemDeviceObject);
    let Vpb = (*DeviceExtension).Vpb;

    let Status = if !Vpb.is_null() {
        if (*Vpb).ReferenceCount != 0 && ((*Vpb).Flags & VPB_MOUNTED) != 0 {
            *DiskDeviceObject = (*Vpb).RealDevice;
            ObReferenceObject((*Vpb).RealDevice as PVOID);
            STATUS_SUCCESS
        } else {
            STATUS_VOLUME_DISMOUNTED
        }
    } else {
        STATUS_INVALID_PARAMETER
    };

    IoReleaseVpbSpinLock(OldIrql);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn IoGetLowerDeviceObject(DeviceObject: PDEVICE_OBJECT) -> PDEVICE_OBJECT {
    let DeviceObject = opaque_ptr(DeviceObject);
    let DeviceExtension = IoGetDevObjExtension(DeviceObject);
    if ((*DeviceExtension).ExtensionFlags
        & (DOE_UNLOAD_PENDING | DOE_DELETE_PENDING | DOE_REMOVE_PENDING | DOE_REMOVE_PROCESSED))
        != 0
    {
        return core::ptr::null_mut();
    }

    let LowerDeviceObject = (*DeviceExtension).AttachedTo;

    if !LowerDeviceObject.is_null() {
        ObReferenceObject(LowerDeviceObject as PVOID);
    }

    LowerDeviceObject
}

#[no_mangle]
pub unsafe extern "system" fn IoGetRelatedDeviceObject(FileObject: PFILE_OBJECT) -> PDEVICE_OBJECT {
    let FileObject = opaque_ptr(FileObject);
    let mut DeviceObject = (*FileObject).DeviceObject;

    if !(*FileObject).Vpb.is_null() && !(*(*FileObject).Vpb).DeviceObject.is_null() {
        DeviceObject = (*(*FileObject).Vpb).DeviceObject;
    } else if ((*FileObject).Flags & FO_DIRECT_DEVICE_OPEN) == 0
        && !(*(*FileObject).DeviceObject).Vpb.is_null()
        && !(*(*(*FileObject).DeviceObject).Vpb).DeviceObject.is_null()
    {
        DeviceObject = (*(*(*FileObject).DeviceObject).Vpb).DeviceObject;
    } else {
        DeviceObject = (*FileObject).DeviceObject;
    }

    if !(*DeviceObject).AttachedDevice.is_null() {
        if ((*FileObject).Flags & FO_FILE_OBJECT_HAS_EXTENSION) != 0 {
            if !(*FileObject).FileObjectExtension.is_null() {
                let FileObjectExtension = (*FileObject).FileObjectExtension as *mut _FILE_OBJECT_EXTENSION;

                if !(*FileObjectExtension).TopDeviceObjectHint.is_null()
                    && IopVerifyDeviceObjectOnStack(DeviceObject, (*FileObjectExtension).TopDeviceObjectHint) != 0
                {
                    return (*FileObjectExtension).TopDeviceObjectHint;
                }
            }
        }

        DeviceObject = IoGetAttachedDevice(DeviceObject);
    }

    DeviceObject
}

#[no_mangle]
pub unsafe extern "system" fn IoGetRelatedTargetDevice(
    FileObject: PFILE_OBJECT,
    DeviceObject: *mut PDEVICE_OBJECT,
) -> NTSTATUS {
    let FileObject = opaque_ptr(FileObject);
    let DeviceObject = opaque_ptr(DeviceObject);
    let mut DeviceNode = core::ptr::null_mut();
    let Status = IopGetRelatedTargetDevice(FileObject, core::ptr::addr_of_mut!(DeviceNode));
    if Status >= 0 && !DeviceNode.is_null() {
        *DeviceObject = (*DeviceNode).PhysicalDeviceObject;
    }
    Status
}

#[no_mangle]
pub unsafe extern "system" fn IoGetBaseFileSystemDeviceObject(FileObject: PFILE_OBJECT) -> PDEVICE_OBJECT {
    let FileObject = opaque_ptr(FileObject);
    let DeviceObject = if !(*FileObject).Vpb.is_null() && !(*(*FileObject).Vpb).DeviceObject.is_null() {
        (*(*FileObject).Vpb).DeviceObject
    } else if ((*FileObject).Flags & FO_DIRECT_DEVICE_OPEN) == 0
        && !(*(*FileObject).DeviceObject).Vpb.is_null()
        && !(*(*(*FileObject).DeviceObject).Vpb).DeviceObject.is_null()
    {
        (*(*(*FileObject).DeviceObject).Vpb).DeviceObject
    } else {
        (*FileObject).DeviceObject
    };

    DeviceObject
}

#[no_mangle]
pub unsafe extern "system" fn IoRegisterLastChanceShutdownNotification(DeviceObject: PDEVICE_OBJECT) -> NTSTATUS {
    let DeviceObject = opaque_ptr(DeviceObject);
    let Entry = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<_SHUTDOWN_ENTRY>(),
        TAG_SHUTDOWN_ENTRY,
    ) as PSHUTDOWN_ENTRY;
    if Entry.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*Entry).DeviceObject = DeviceObject;
    ObReferenceObject(DeviceObject as PVOID);

    ExInterlockedInsertHeadList(
        core::ptr::addr_of_mut!(LastChanceShutdownListHead) as PLIST_ENTRY,
        core::ptr::addr_of_mut!((*Entry).ShutdownList) as PLIST_ENTRY,
        core::ptr::addr_of_mut!(ShutdownListLock) as *mut _,
    );

    (*DeviceObject).Flags |= DO_SHUTDOWN_REGISTERED;
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn IoRegisterShutdownNotification(DeviceObject: PDEVICE_OBJECT) -> NTSTATUS {
    let DeviceObject = opaque_ptr(DeviceObject);
    let Entry = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<_SHUTDOWN_ENTRY>(),
        TAG_SHUTDOWN_ENTRY,
    ) as PSHUTDOWN_ENTRY;
    if Entry.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*Entry).DeviceObject = DeviceObject;
    ObReferenceObject(DeviceObject as PVOID);

    ExInterlockedInsertHeadList(
        core::ptr::addr_of_mut!(ShutdownListHead) as PLIST_ENTRY,
        core::ptr::addr_of_mut!((*Entry).ShutdownList) as PLIST_ENTRY,
        core::ptr::addr_of_mut!(ShutdownListLock) as *mut _,
    );

    (*DeviceObject).Flags |= DO_SHUTDOWN_REGISTERED;
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn IoUnregisterShutdownNotification(DeviceObject: PDEVICE_OBJECT) {
    let DeviceObject = opaque_ptr(DeviceObject);
    (*DeviceObject).Flags &= !DO_SHUTDOWN_REGISTERED;

    let mut OldIrql = 0u8;
    // [guard] KeAcquireSpinLock(core::ptr::addr_of_mut!(ShutdownListLock), core::ptr::addr_of_mut!(OldIrql));
    let spin_guard = unsafe { ntkrnl_sys::guards::SpinLockGuard::acquire(core::ptr::addr_of_mut!(ShutdownListLock)) };

    let mut NextEntry = ShutdownListHead.Flink;
    while NextEntry != core::ptr::addr_of_mut!(ShutdownListHead) as PLIST_ENTRY {
        let ShutdownEntry = CONTAINING_RECORD(
            NextEntry as PVOID,
            core::mem::size_of::<_SHUTDOWN_ENTRY>(),
            core::mem::offset_of!(_SHUTDOWN_ENTRY, ShutdownList),
        ) as PSHUTDOWN_ENTRY;

        if (*ShutdownEntry).DeviceObject == DeviceObject {
            RemoveEntryList(NextEntry);
            NextEntry = (*NextEntry).Blink;
            ExFreePoolWithTag(ShutdownEntry as PVOID, TAG_SHUTDOWN_ENTRY);
            ObDereferenceObject(DeviceObject as PVOID);
        }

        NextEntry = (*NextEntry).Flink;
    }

    NextEntry = LastChanceShutdownListHead.Flink;
    while NextEntry != core::ptr::addr_of_mut!(LastChanceShutdownListHead) as PLIST_ENTRY {
        let ShutdownEntry = CONTAINING_RECORD(
            NextEntry as PVOID,
            core::mem::size_of::<_SHUTDOWN_ENTRY>(),
            core::mem::offset_of!(_SHUTDOWN_ENTRY, ShutdownList),
        ) as PSHUTDOWN_ENTRY;

        if (*ShutdownEntry).DeviceObject == DeviceObject {
            RemoveEntryList(NextEntry);
            NextEntry = (*NextEntry).Blink;
            ExFreePoolWithTag(ShutdownEntry as PVOID, TAG_SHUTDOWN_ENTRY);
            ObDereferenceObject(DeviceObject as PVOID);
        }

        NextEntry = (*NextEntry).Flink;
    }

    drop(spin_guard);
}

#[no_mangle]
pub unsafe extern "system" fn IoSetStartIoAttributes(
    DeviceObject: PDEVICE_OBJECT,
    DeferredStartIo: BOOLEAN,
    NonCancelable: BOOLEAN,
) {
    let DeviceObject = opaque_ptr(DeviceObject);
    let DeviceExtension = IoGetDevObjExtension(DeviceObject);

    if DeferredStartIo != 0 {
        (*DeviceExtension).StartIoFlags |= DOE_SIO_DEFERRED;
    }
    if NonCancelable != 0 {
        (*DeviceExtension).StartIoFlags |= DOE_SIO_NO_CANCEL;
    }
}

#[no_mangle]
pub unsafe extern "system" fn IoStartNextPacketByKey(
    DeviceObject: PDEVICE_OBJECT,
    Cancelable: BOOLEAN,
    Key: ULONG,
) {
    let DeviceObject = opaque_ptr(DeviceObject);
    let DeviceExtension = IoGetDevObjExtension(DeviceObject);

    if ((*DeviceExtension).StartIoFlags & DOE_SIO_DEFERRED) != 0 {
        IopStartNextPacketByKeyEx(
            DeviceObject,
            Key,
            DOE_SIO_WITH_KEY | if Cancelable != 0 { DOE_SIO_CANCELABLE } else { 0 },
        );
    } else {
        IopStartNextPacketByKey(DeviceObject, Cancelable, Key);
    }
}

#[no_mangle]
pub unsafe extern "system" fn IoStartNextPacket(DeviceObject: PDEVICE_OBJECT, Cancelable: BOOLEAN) {
    let DeviceObject = opaque_ptr(DeviceObject);
    let DeviceExtension = IoGetDevObjExtension(DeviceObject);

    if ((*DeviceExtension).StartIoFlags & DOE_SIO_DEFERRED) != 0 {
        IopStartNextPacketByKeyEx(
            DeviceObject,
            0,
            DOE_SIO_NO_KEY | if Cancelable != 0 { DOE_SIO_CANCELABLE } else { 0 },
        );
    } else {
        IopStartNextPacket(DeviceObject, Cancelable);
    }
}

#[no_mangle]
pub unsafe extern "system" fn IoStartPacket(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Key: *mut ULONG,
    CancelFunction: ntkrnl_sys::ddk::PDRIVER_CANCEL,
) {
    let DeviceObject = opaque_ptr(DeviceObject);
    let Irp = opaque_ptr(Irp);
    let Key = opaque_ptr(Key);
    let mut OldIrql = 0u8;
    // [guard] KeRaiseIrql(DISPATCH_LEVEL, core::ptr::addr_of_mut!(OldIrql));
    let irql_guard = unsafe { ntkrnl_sys::guards::IrqlGuard::raise(DISPATCH_LEVEL) };

    let mut CancelIrql = 0u8;
    if CancelFunction.is_some() {
        IoAcquireCancelSpinLock(core::ptr::addr_of_mut!(CancelIrql));
        (*Irp).CancelRoutine = CancelFunction;
    }

    let Stat = if !Key.is_null() {
        KeInsertByKeyDeviceQueue(
            core::ptr::addr_of_mut!((*DeviceObject).DeviceQueue),
            core::ptr::addr_of_mut!((*Irp).Tail.Overlay.__bindgen_anon_1.DeviceQueueEntry),
            *Key,
        )
    } else {
        KeInsertDeviceQueue(
            core::ptr::addr_of_mut!((*DeviceObject).DeviceQueue),
            core::ptr::addr_of_mut!((*Irp).Tail.Overlay.__bindgen_anon_1.DeviceQueueEntry),
        )
    };

    if Stat == 0 {
        (*DeviceObject).CurrentIrp = Irp;

        if CancelFunction.is_some() {
            if ((*IoGetDevObjExtension(DeviceObject)).StartIoFlags & DOE_SIO_NO_CANCEL) != 0 {
                (*Irp).CancelRoutine = None;
            }
            IoReleaseCancelSpinLock(CancelIrql);
        }

        ((*(*DeviceObject).DriverObject).DriverStartIo.unwrap())(DeviceObject, Irp);
    } else {
        if CancelFunction.is_some() {
            if (*Irp).Cancel != 0 {
                (*Irp).CancelIrql = CancelIrql;
                (*Irp).CancelRoutine = None;
                (CancelFunction.unwrap())(DeviceObject, Irp);
            } else {
                IoReleaseCancelSpinLock(CancelIrql);
            }
        }
    }

    drop(irql_guard);
}

#[cfg(target_pointer_width = "64")]
#[no_mangle]
pub unsafe extern "system" fn IoWMIDeviceObjectToProviderId(_DeviceObject: PDEVICE_OBJECT) -> ULONG {
    let _DeviceObject = opaque_ptr(_DeviceObject);
    0
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/io/iomgr/device.c
//  * PURPOSE:         Device Object Management, including Notifications and Queues.
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  *                  Filip Navara (navaraf@reactos.org)
//  *                  Hervé Poussineau (hpoussin@reactos.org)
//  *                  Pierre Schweitzer
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS ********************************************************************/
//
// ULONG IopDeviceObjectNumber = 0;
// LIST_ENTRY ShutdownListHead, LastChanceShutdownListHead;
// KSPIN_LOCK ShutdownListLock;
// extern LIST_ENTRY IopDiskFileSystemQueueHead;
// extern LIST_ENTRY IopCdRomFileSystemQueueHead;
// extern LIST_ENTRY IopTapeFileSystemQueueHead;
// extern ERESOURCE IopDatabaseResource;
//
// #define DACL_SET 4
//
// /* PRIVATE FUNCTIONS **********************************************************/
//
// VOID
// NTAPI
// IopReadyDeviceObjects(IN PDRIVER_OBJECT Driver)
// {
//     PDEVICE_OBJECT DeviceObject;
//     PAGED_CODE();
//
//     /* Set the driver as initialized */
//     Driver->Flags |= DRVO_INITIALIZED;
//     DeviceObject = Driver->DeviceObject;
//     while (DeviceObject)
//     {
//         /* Set every device as initialized too */
//         DeviceObject->Flags &= ~DO_DEVICE_INITIALIZING;
//         DeviceObject = DeviceObject->NextDevice;
//     }
// }
//
// VOID
// NTAPI
// IopDeleteDevice(IN PVOID ObjectBody)
// {
//     PDEVICE_OBJECT DeviceObject = ObjectBody;
//     PDEVICE_NODE DeviceNode = IopGetDeviceNode(DeviceObject);
//     PAGED_CODE();
//
//     /* Cleanup and free the device node */
//     if (DeviceNode)
//         IopFreeDeviceNode(DeviceNode);
//
//     /* Dereference the driver object, referenced in IoCreateDevice */
//     if (DeviceObject->DriverObject)
//         ObDereferenceObject(DeviceObject->DriverObject);
// }
//
//
// PDEVICE_OBJECT
// NTAPI
// IopAttachDeviceToDeviceStackSafe(IN PDEVICE_OBJECT SourceDevice,
//                                  IN PDEVICE_OBJECT TargetDevice,
//                                  OUT PDEVICE_OBJECT *AttachedToDeviceObject OPTIONAL)
// {
//     PDEVICE_OBJECT AttachedDevice;
//     PEXTENDED_DEVOBJ_EXTENSION SourceDeviceExtension;
//
//     /* Get the Attached Device and source extension */
//     AttachedDevice = IoGetAttachedDevice(TargetDevice);
//     SourceDeviceExtension = IoGetDevObjExtension(SourceDevice);
//     ASSERT(SourceDeviceExtension->AttachedTo == NULL);
//
//     /* Make sure that it's in a correct state */
//     if ((AttachedDevice->Flags & DO_DEVICE_INITIALIZING) ||
//         (IoGetDevObjExtension(AttachedDevice)->ExtensionFlags &
//          (DOE_UNLOAD_PENDING |
//           DOE_DELETE_PENDING |
//           DOE_REMOVE_PENDING |
//           DOE_REMOVE_PROCESSED)))
//     {
//         /* Device was unloading or being removed */
//         AttachedDevice = NULL;
//     }
//     else
//     {
//         /* Update atached device fields */
//         AttachedDevice->AttachedDevice = SourceDevice;
//         AttachedDevice->Spare1++;
//
//         /* Update the source with the attached data */
//         SourceDevice->StackSize = AttachedDevice->StackSize + 1;
//         SourceDevice->AlignmentRequirement = AttachedDevice->
//                                              AlignmentRequirement;
//         SourceDevice->SectorSize = AttachedDevice->SectorSize;
//
//         /* Check for pending start flag */
//         if (IoGetDevObjExtension(AttachedDevice)->ExtensionFlags &
//             DOE_START_PENDING)
//         {
//             /* Propagate */
//             IoGetDevObjExtension(SourceDevice)->ExtensionFlags |=
//                 DOE_START_PENDING;
//         }
//
//         /* Set the attachment in the device extension */
//         SourceDeviceExtension->AttachedTo = AttachedDevice;
//     }
//
//     /* Return the attached device */
//     if (AttachedToDeviceObject) *AttachedToDeviceObject = AttachedDevice;
//     return AttachedDevice;
// }
//
// VOID
// NTAPI
// IoShutdownPnpDevices(VOID)
// {
//     /* This routine is only used by Driver Verifier to validate shutdown */
//     return;
// }
//
// VOID
// NTAPI
// IoShutdownSystem(IN ULONG Phase)
// {
//     PLIST_ENTRY ListEntry;
//     PDEVICE_OBJECT DeviceObject;
//     PSHUTDOWN_ENTRY ShutdownEntry;
//     IO_STATUS_BLOCK StatusBlock;
//     PIRP Irp;
//     KEVENT Event;
//     NTSTATUS Status;
//
//     /* Initialize an event to wait on */
//     KeInitializeEvent(&Event, NotificationEvent, FALSE);
//
//     /* What phase? */
//     if (Phase == 0)
//     {
//         /* Shutdown PnP */
//         IoShutdownPnpDevices();
//
//         /* Loop first-chance shutdown notifications */
//         ListEntry = ExInterlockedRemoveHeadList(&ShutdownListHead,
//                                                 &ShutdownListLock);
//         while (ListEntry)
//         {
//             /* Get the shutdown entry */
//             ShutdownEntry = CONTAINING_RECORD(ListEntry,
//                                               SHUTDOWN_ENTRY,
//                                               ShutdownList);
//
//             /* Get the attached device */
//             DeviceObject = IoGetAttachedDevice(ShutdownEntry->DeviceObject);
//
//             /* Build the shutdown IRP and call the driver */
//             Irp = IoBuildSynchronousFsdRequest(IRP_MJ_SHUTDOWN,
//                                                DeviceObject,
//                                                NULL,
//                                                0,
//                                                NULL,
//                                                &Event,
//                                                &StatusBlock);
//             if (Irp)
//             {
//                 Status = IoCallDriver(DeviceObject, Irp);
//                 if (Status == STATUS_PENDING)
//                 {
//                     /* Wait on the driver */
//                     KeWaitForSingleObject(&Event, Executive, KernelMode, FALSE, NULL);
//                 }
//             }
//
//             /* Remove the flag */
//             ShutdownEntry->DeviceObject->Flags &= ~DO_SHUTDOWN_REGISTERED;
//
//             /* Get rid of our reference to it */
//             ObDereferenceObject(ShutdownEntry->DeviceObject);
//
//             /* Free the shutdown entry and reset the event */
//             ExFreePoolWithTag(ShutdownEntry, TAG_SHUTDOWN_ENTRY);
//             KeClearEvent(&Event);
//
//             /* Go to the next entry */
//             ListEntry = ExInterlockedRemoveHeadList(&ShutdownListHead,
//                                                     &ShutdownListLock);
//          }
//     }
//     else if (Phase == 1)
//     {
//         /* Acquire resource forever */
//         ExAcquireResourceExclusiveLite(&IopDatabaseResource, TRUE);
//
//         /* Shutdown disk file systems */
//         IopShutdownBaseFileSystems(&IopDiskFileSystemQueueHead);
//
//         /* Shutdown cdrom file systems */
//         IopShutdownBaseFileSystems(&IopCdRomFileSystemQueueHead);
//
//         /* Shutdown tape filesystems */
//         IopShutdownBaseFileSystems(&IopTapeFileSystemQueueHead);
//
//         /* Loop last-chance shutdown notifications */
//         ListEntry = ExInterlockedRemoveHeadList(&LastChanceShutdownListHead,
//                                                 &ShutdownListLock);
//         while (ListEntry)
//         {
//             /* Get the shutdown entry */
//             ShutdownEntry = CONTAINING_RECORD(ListEntry,
//                                               SHUTDOWN_ENTRY,
//                                               ShutdownList);
//
//             /* Get the attached device */
//             DeviceObject = IoGetAttachedDevice(ShutdownEntry->DeviceObject);
//
//             /* Build the shutdown IRP and call the driver */
//             Irp = IoBuildSynchronousFsdRequest(IRP_MJ_SHUTDOWN,
//                                                DeviceObject,
//                                                NULL,
//                                                0,
//                                                NULL,
//                                                &Event,
//                                                &StatusBlock);
//             if (Irp)
//             {
//                 Status = IoCallDriver(DeviceObject, Irp);
//                 if (Status == STATUS_PENDING)
//                 {
//                     /* Wait on the driver */
//                     KeWaitForSingleObject(&Event, Executive, KernelMode, FALSE, NULL);
//                 }
//             }
//
//             /* Remove the flag */
//             ShutdownEntry->DeviceObject->Flags &= ~DO_SHUTDOWN_REGISTERED;
//
//             /* Get rid of our reference to it */
//             ObDereferenceObject(ShutdownEntry->DeviceObject);
//
//             /* Free the shutdown entry and reset the event */
//             ExFreePoolWithTag(ShutdownEntry, TAG_SHUTDOWN_ENTRY);
//             KeClearEvent(&Event);
//
//             /* Go to the next entry */
//             ListEntry = ExInterlockedRemoveHeadList(&LastChanceShutdownListHead,
//                                                     &ShutdownListLock);
//          }
//
//     }
// }
//
// NTSTATUS
// NTAPI
// IopGetDeviceObjectPointer(IN PUNICODE_STRING ObjectName,
//                           IN ACCESS_MASK DesiredAccess,
//                           OUT PFILE_OBJECT *FileObject,
//                           OUT PDEVICE_OBJECT *DeviceObject,
//                           IN ULONG AttachFlag)
// {
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     IO_STATUS_BLOCK StatusBlock;
//     PFILE_OBJECT LocalFileObject;
//     HANDLE FileHandle;
//     NTSTATUS Status;
//
//     /* Open the Device */
//     InitializeObjectAttributes(&ObjectAttributes,
//                                ObjectName,
//                                OBJ_KERNEL_HANDLE,
//                                NULL,
//                                NULL);
//     Status = ZwOpenFile(&FileHandle,
//                         DesiredAccess,
//                         &ObjectAttributes,
//                         &StatusBlock,
//                         0,
//                         FILE_NON_DIRECTORY_FILE | AttachFlag);
//     if (!NT_SUCCESS(Status)) return Status;
//
//     /* Get File Object */
//     Status = ObReferenceObjectByHandle(FileHandle,
//                                        0,
//                                        IoFileObjectType,
//                                        KernelMode,
//                                        (PVOID*)&LocalFileObject,
//                                        NULL);
//     if (NT_SUCCESS(Status))
//     {
//         /* Return the requested data */
//         *DeviceObject = IoGetRelatedDeviceObject(LocalFileObject);
//         *FileObject = LocalFileObject;
//     }
//
//     /* Close the handle */
//     ZwClose(FileHandle);
//
//     return Status;
// }
//
// PDEVICE_OBJECT
// NTAPI
// IopGetLowestDevice(IN PDEVICE_OBJECT DeviceObject)
// {
//     PDEVICE_OBJECT LowestDevice;
//     PEXTENDED_DEVOBJ_EXTENSION DeviceExtension;
//
//     /* Get the current device and its extension */
//     LowestDevice = DeviceObject;
//     DeviceExtension = IoGetDevObjExtension(LowestDevice);
//
//     /* Keep looping as long as we're attached */
//     while (DeviceExtension->AttachedTo)
//     {
//         /* Get the lowest device and its extension */
//         LowestDevice = DeviceExtension->AttachedTo;
//         DeviceExtension = IoGetDevObjExtension(LowestDevice);
//     }
//
//     /* Return the lowest device */
//     return LowestDevice;
// }
//
// VOID
// NTAPI
// IopEditDeviceList(IN PDRIVER_OBJECT DriverObject,
//                   IN PDEVICE_OBJECT DeviceObject,
//                   IN IOP_DEVICE_LIST_OPERATION Type)
// {
//     PDEVICE_OBJECT Previous;
//     KIRQL OldIrql;
//
//     /* Lock the Device list while we edit it */
//     OldIrql = KeAcquireQueuedSpinLock(LockQueueIoDatabaseLock);
//
//     /* Check the type of operation */
//     if (Type == IopRemove)
//     {
//         /* Get the current device and check if it's the current one */
//         Previous = DeviceObject->DriverObject->DeviceObject;
//         if (Previous == DeviceObject)
//         {
//             /* It is, simply unlink this one directly */
//             DeviceObject->DriverObject->DeviceObject =
//                 DeviceObject->NextDevice;
//         }
//         else
//         {
//             /* It's not, so loop until we find the device */
//             while (Previous->NextDevice != DeviceObject)
//             {
//                 /* Not this one, keep moving */
//                 if (!Previous->NextDevice)
//                 {
//                     DPRINT1("Failed to remove PDO %p (not found)\n",
//                             DeviceObject);
//
//                     ASSERT(FALSE);
//                     KeReleaseQueuedSpinLock(LockQueueIoDatabaseLock, OldIrql);
//                     return;
//                 }
//                 Previous = Previous->NextDevice;
//             }
//
//             /* We found it, now unlink us */
//             Previous->NextDevice = DeviceObject->NextDevice;
//         }
//     }
//     else
//     {
//         /* Link the device object and the driver object */
//         DeviceObject->NextDevice = DriverObject->DeviceObject;
//         DriverObject->DeviceObject = DeviceObject;
//     }
//
//     /* Release the device list lock */
//     KeReleaseQueuedSpinLock(LockQueueIoDatabaseLock, OldIrql);
// }
//
// VOID
// NTAPI
// IopUnloadDevice(IN PDEVICE_OBJECT DeviceObject)
// {
//     PDRIVER_OBJECT DriverObject = DeviceObject->DriverObject;
//     PEXTENDED_DEVOBJ_EXTENSION ThisExtension = IoGetDevObjExtension(DeviceObject);
//
//     /* Check if deletion is pending */
//     if (ThisExtension->ExtensionFlags & DOE_DELETE_PENDING)
//     {
//         if (DeviceObject->AttachedDevice)
//         {
//             DPRINT("Device object is in the middle of a device stack\n");
//             return;
//         }
//
//         if (DeviceObject->ReferenceCount)
//         {
//             DPRINT("Device object still has %d references\n", DeviceObject->ReferenceCount);
//             return;
//         }
//
//         /* Check if we have a Security Descriptor */
//         if (DeviceObject->SecurityDescriptor)
//         {
//             /* Dereference it */
//             ObDereferenceSecurityDescriptor(DeviceObject->SecurityDescriptor, 1);
//         }
//
//         /* Remove the device from the list */
//         IopEditDeviceList(DeviceObject->DriverObject, DeviceObject, IopRemove);
//
//         /* Dereference the keep-alive */
//         ObDereferenceObject(DeviceObject);
//     }
//
//     /* We can't unload a non-PnP driver here */
//     if (DriverObject->Flags & DRVO_LEGACY_DRIVER)
//     {
//         DPRINT("Not a PnP driver! '%wZ' will not be unloaded!\n", &DriverObject->DriverName);
//         return;
//     }
//
//     /* Return if we've already called unload (maybe we're in it?) */
//     if (DriverObject->Flags & DRVO_UNLOAD_INVOKED) return;
//
//     /* We can't unload unless there's an unload handler */
//     if (!DriverObject->DriverUnload)
//     {
//         DPRINT1("No DriverUnload function on PnP driver! '%wZ' will not be unloaded!\n", &DriverObject->DriverName);
//         return;
//     }
//
//     /* Bail if there are still devices present */
//     if (DriverObject->DeviceObject)
//     {
//         DPRINT("Devices still present! '%wZ' will not be unloaded!\n", &DriverObject->DriverName);
//         return;
//     }
//
//     DPRINT1("Unloading driver '%wZ' (automatic)\n", &DriverObject->DriverName);
//
//     /* Set the unload invoked flag */
//     DriverObject->Flags |= DRVO_UNLOAD_INVOKED;
//
//     /* Unload it */
//     DriverObject->DriverUnload(DriverObject);
//
//     /* Make object temporary so it can be deleted */
//     ObMakeTemporaryObject(DriverObject);
// }
//
// VOID
// NTAPI
// IopDereferenceDeviceObject(IN PDEVICE_OBJECT DeviceObject,
//                            IN BOOLEAN ForceUnload)
// {
//     /* Sanity check */
//     ASSERT(DeviceObject->ReferenceCount);
//
//     /* Dereference the device */
//     InterlockedDecrement(&DeviceObject->ReferenceCount);
//
//     /*
//      * Check if we can unload it and it's safe to unload (or if we're forcing
//      * an unload, which is OK too).
//      */
//     ASSERT(!ForceUnload);
//     if (!(DeviceObject->ReferenceCount) &&
//         (IoGetDevObjExtension(DeviceObject)->ExtensionFlags & DOE_DELETE_PENDING))
//     {
//         /* Unload it */
//         IopUnloadDevice(DeviceObject);
//     }
// }
//
// VOID
// NTAPI
// IopStartNextPacketByKey(IN PDEVICE_OBJECT DeviceObject,
//                         IN BOOLEAN Cancelable,
//                         IN ULONG Key)
// {
//     PKDEVICE_QUEUE_ENTRY Entry;
//     PIRP Irp;
//     KIRQL OldIrql;
//
//     /* Acquire the cancel lock if this is cancelable */
//     if (Cancelable) IoAcquireCancelSpinLock(&OldIrql);
//
//     /* Clear the current IRP */
//     DeviceObject->CurrentIrp = NULL;
//
//     /* Remove an entry from the queue */
//     Entry = KeRemoveByKeyDeviceQueue(&DeviceObject->DeviceQueue, Key);
//     if (Entry)
//     {
//         /* Get the IRP and set it */
//         Irp = CONTAINING_RECORD(Entry, IRP, Tail.Overlay.DeviceQueueEntry);
//         DeviceObject->CurrentIrp = Irp;
//
//         /* Check if this is a cancelable packet */
//         if (Cancelable)
//         {
//             /* Check if the caller requested no cancellation */
//             if (IoGetDevObjExtension(DeviceObject)->StartIoFlags &
//                 DOE_SIO_NO_CANCEL)
//             {
//                 /* He did, so remove the cancel routine */
//                 Irp->CancelRoutine = NULL;
//             }
//
//             /* Release the cancel lock */
//             IoReleaseCancelSpinLock(OldIrql);
//         }
//
//         /* Call the Start I/O Routine */
//         DeviceObject->DriverObject->DriverStartIo(DeviceObject, Irp);
//     }
//     else
//     {
//         /* Otherwise, release the cancel lock if we had acquired it */
//         if (Cancelable) IoReleaseCancelSpinLock(OldIrql);
//     }
// }
//
// VOID
// NTAPI
// IopStartNextPacket(IN PDEVICE_OBJECT DeviceObject,
//                    IN BOOLEAN Cancelable)
// {
//     PKDEVICE_QUEUE_ENTRY Entry;
//     PIRP Irp;
//     KIRQL OldIrql;
//
//     /* Acquire the cancel lock if this is cancelable */
//     if (Cancelable) IoAcquireCancelSpinLock(&OldIrql);
//
//     /* Clear the current IRP */
//     DeviceObject->CurrentIrp = NULL;
//
//     /* Remove an entry from the queue */
//     Entry = KeRemoveDeviceQueue(&DeviceObject->DeviceQueue);
//     if (Entry)
//     {
//         /* Get the IRP and set it */
//         Irp = CONTAINING_RECORD(Entry, IRP, Tail.Overlay.DeviceQueueEntry);
//         DeviceObject->CurrentIrp = Irp;
//
//         /* Check if this is a cancelable packet */
//         if (Cancelable)
//         {
//             /* Check if the caller requested no cancellation */
//             if (IoGetDevObjExtension(DeviceObject)->StartIoFlags &
//                 DOE_SIO_NO_CANCEL)
//             {
//                 /* He did, so remove the cancel routine */
//                 Irp->CancelRoutine = NULL;
//             }
//
//             /* Release the cancel lock */
//             IoReleaseCancelSpinLock(OldIrql);
//         }
//
//         /* Call the Start I/O Routine */
//         DeviceObject->DriverObject->DriverStartIo(DeviceObject, Irp);
//     }
//     else
//     {
//         /* Otherwise, release the cancel lock if we had acquired it */
//         if (Cancelable) IoReleaseCancelSpinLock(OldIrql);
//     }
// }
//
// VOID
// NTAPI
// IopStartNextPacketByKeyEx(IN PDEVICE_OBJECT DeviceObject,
//                           IN ULONG Key,
//                           IN ULONG Flags)
// {
//     PEXTENDED_DEVOBJ_EXTENSION DeviceExtension;
//     ULONG CurrentKey = Key;
//     ULONG CurrentFlags = Flags;
//
//     /* Get the device extension and start the packet loop */
//     DeviceExtension = IoGetDevObjExtension(DeviceObject);
//     while (TRUE)
//     {
//         /* Increase the count */
//         if (InterlockedIncrement(&DeviceExtension->StartIoCount) > 1)
//         {
//             /*
//              * We've already called the routine once...
//              * All we have to do is save the key and add the new flags
//              */
//             DeviceExtension->StartIoFlags |= CurrentFlags;
//             DeviceExtension->StartIoKey = CurrentKey;
//         }
//         else
//         {
//             /* Mask out the current packet flags and key */
//             DeviceExtension->StartIoFlags &= ~(DOE_SIO_WITH_KEY |
//                                                DOE_SIO_NO_KEY |
//                                                DOE_SIO_CANCELABLE);
//             DeviceExtension->StartIoKey = 0;
//
//             /* Check if this is a packet start with key */
//             if (Flags & DOE_SIO_WITH_KEY)
//             {
//                 /* Start the packet with a key */
//                 IopStartNextPacketByKey(DeviceObject,
//                                         (Flags & DOE_SIO_CANCELABLE) ?
//                                         TRUE : FALSE,
//                                         CurrentKey);
//             }
//             else if (Flags & DOE_SIO_NO_KEY)
//             {
//                 /* Start the packet */
//                 IopStartNextPacket(DeviceObject,
//                                    (Flags & DOE_SIO_CANCELABLE) ?
//                                    TRUE : FALSE);
//             }
//         }
//
//         /* Decrease the Start I/O count and check if it's 0 now */
//         if (!InterlockedDecrement(&DeviceExtension->StartIoCount))
//         {
//             /* Get the current active key and flags */
//             CurrentKey = DeviceExtension->StartIoKey;
//             CurrentFlags = DeviceExtension->StartIoFlags & (DOE_SIO_WITH_KEY |
//                                                             DOE_SIO_NO_KEY |
//                                                             DOE_SIO_CANCELABLE);
//
//             /* Check if we should still loop */
//             if (!(CurrentFlags & (DOE_SIO_WITH_KEY | DOE_SIO_NO_KEY))) break;
//         }
//         else
//         {
//             /* There are still Start I/Os active, so quit this loop */
//             break;
//         }
//     }
// }
//
// NTSTATUS
// NTAPI
// IopGetRelatedTargetDevice(IN PFILE_OBJECT FileObject,
//                           OUT PDEVICE_NODE *DeviceNode)
// {
//     NTSTATUS Status;
//     IO_STACK_LOCATION Stack = {0};
//     PDEVICE_RELATIONS DeviceRelations;
//     PDEVICE_OBJECT DeviceObject = NULL;
//
//     ASSERT(FileObject);
//
//     /* Get DeviceObject related to given FileObject */
//     DeviceObject = IoGetRelatedDeviceObject(FileObject);
//     if (!DeviceObject) return STATUS_NO_SUCH_DEVICE;
//
//     /* Define input parameters */
//     Stack.MajorFunction = IRP_MJ_PNP;
//     Stack.MinorFunction = IRP_MN_QUERY_DEVICE_RELATIONS;
//     Stack.Parameters.QueryDeviceRelations.Type = TargetDeviceRelation;
//     Stack.FileObject = FileObject;
//
//     /* Call the driver to query all relations (IRP_MJ_PNP) */
//     Status = IopSynchronousCall(DeviceObject,
//                                 &Stack,
//                                 (PVOID)&DeviceRelations);
//     if (!NT_SUCCESS(Status)) return Status;
//
//     /* Make sure it's not NULL and contains only one object */
//     ASSERT(DeviceRelations);
//     ASSERT(DeviceRelations->Count == 1);
//
//     /* Finally get the device node */
//     *DeviceNode = IopGetDeviceNode(DeviceRelations->Objects[0]);
//     if (!*DeviceNode) Status = STATUS_NO_SUCH_DEVICE;
//
//     /* Free the DEVICE_RELATIONS structure, it's not needed anymore */
//     ExFreePool(DeviceRelations);
//
//     return Status;
// }
//
// BOOLEAN
// NTAPI
// IopVerifyDeviceObjectOnStack(IN PDEVICE_OBJECT BaseDeviceObject,
//                              IN PDEVICE_OBJECT TopDeviceObjectHint)
// {
//     KIRQL OldIrql;
//     BOOLEAN Result;
//     PDEVICE_OBJECT LoopObject;
//
//     ASSERT(BaseDeviceObject != NULL);
//
//     Result = FALSE;
//     /* Simply loop on the device stack and try to find our hint */
//     OldIrql = KeAcquireQueuedSpinLock(LockQueueIoDatabaseLock);
//     for (LoopObject = BaseDeviceObject; ; LoopObject = LoopObject->AttachedDevice)
//     {
//         /* It was found, it's a success */
//         if (LoopObject == TopDeviceObjectHint)
//         {
//             Result = TRUE;
//             break;
//         }
//
//         /* End of the stack, that's a failure - default */
//         if (LoopObject == NULL)
//         {
//             break;
//         }
//     }
//     KeReleaseQueuedSpinLock(LockQueueIoDatabaseLock, OldIrql);
//
//     return Result;
// }
//
// NTSTATUS
// NTAPI
// IopCreateSecurityDescriptorPerType(IN PSECURITY_DESCRIPTOR SecurityDescriptor,
//                                    IN SECURITY_DESCRIPTOR_TYPE Type,
//                                    OUT PULONG OutputFlags)
// {
//     PACL Dacl;
//     NTSTATUS Status;
//
//     /* Select the DACL the caller wants */
//     switch (Type)
//     {
//         case RestrictedPublic:
//             Dacl = SePublicDefaultDacl;
//             break;
//
//         case UnrestrictedPublic:
//             Dacl = SePublicDefaultUnrestrictedDacl;
//             break;
//
//         case RestrictedPublicOpen:
//             Dacl = SePublicOpenDacl;
//             break;
//
//         case UnrestrictedPublicOpen:
//             Dacl = SePublicOpenUnrestrictedDacl;
//             break;
//
//         case SystemDefault:
//             Dacl = SeSystemDefaultDacl;
//             break;
//
//         default:
//             ASSERT(FALSE);
//             return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Create the SD and set the DACL caller wanted */
//     Status = RtlCreateSecurityDescriptor(SecurityDescriptor, SECURITY_DESCRIPTOR_REVISION);
//     ASSERT(NT_SUCCESS(Status));
//     Status = RtlSetDaclSecurityDescriptor(SecurityDescriptor, TRUE, Dacl, FALSE);
//
//     /* We've set DACL */
//     if (OutputFlags) *OutputFlags |= DACL_SET;
//
//     /* Done */
//     return Status;
// }
//
// PSECURITY_DESCRIPTOR
// NTAPI
// IopCreateDefaultDeviceSecurityDescriptor(IN DEVICE_TYPE DeviceType,
//                                          IN ULONG DeviceCharacteristics,
//                                          IN BOOLEAN HasDeviceName,
//                                          IN PSECURITY_DESCRIPTOR SecurityDescriptor,
//                                          OUT PACL * OutputDacl,
//                                          OUT PULONG OutputFlags)
// {
//     PACL Dacl;
//     ULONG AceId;
//     NTSTATUS Status;
//     PACCESS_ALLOWED_ACE Ace;
//     BOOLEAN AdminsSet, WorldSet;
//
//     PAGED_CODE();
//
//     /* Zero our output vars */
//     if (OutputFlags) *OutputFlags = 0;
//
//     *OutputDacl = NULL;
//
//     /* For FSD, easy use SePublicDefaultUnrestrictedDacl */
//     if (DeviceType == FILE_DEVICE_TAPE_FILE_SYSTEM ||
//         DeviceType == FILE_DEVICE_CD_ROM_FILE_SYSTEM ||
//         DeviceType == FILE_DEVICE_DISK_FILE_SYSTEM ||
//         DeviceType == FILE_DEVICE_FILE_SYSTEM)
//     {
//         Status = IopCreateSecurityDescriptorPerType(SecurityDescriptor,
//                                                     UnrestrictedPublic,
//                                                     OutputFlags);
//         goto Quit;
//     }
//     /* For storage devices with a name and floppy attribute,
//      * use SePublicOpenUnrestrictedDacl
//      */
//     else if ((DeviceType != FILE_DEVICE_VIRTUAL_DISK &&
//               DeviceType != FILE_DEVICE_MASS_STORAGE &&
//               DeviceType != FILE_DEVICE_CD_ROM &&
//               DeviceType != FILE_DEVICE_DISK &&
//               DeviceType != FILE_DEVICE_DFS_FILE_SYSTEM &&
//               DeviceType != FILE_DEVICE_NETWORK &&
//               DeviceType != FILE_DEVICE_NETWORK_FILE_SYSTEM) ||
//               (HasDeviceName && BooleanFlagOn(DeviceCharacteristics, FILE_FLOPPY_DISKETTE)))
//     {
//         Status = IopCreateSecurityDescriptorPerType(SecurityDescriptor,
//                                                     UnrestrictedPublicOpen,
//                                                     OutputFlags);
//         goto Quit;
//     }
//
//     /* The rest...
//      * We will rely on SePublicDefaultUnrestrictedDacl as well
//      */
//     Dacl = ExAllocatePoolWithTag(PagedPool, SePublicDefaultUnrestrictedDacl->AclSize, 'eSoI');
//     if (Dacl == NULL)
//     {
//         return NULL;
//     }
//
//     /* Copy our DACL */
//     RtlCopyMemory(Dacl, SePublicDefaultUnrestrictedDacl, SePublicDefaultUnrestrictedDacl->AclSize);
//
//     /* Now, browse the DACL to make sure we have everything we want in them,
//      * including permissions
//      */
//     AceId = 0;
//     AdminsSet = FALSE;
//     WorldSet = FALSE;
//     while (NT_SUCCESS(RtlGetAce(Dacl, AceId, (PVOID *)&Ace)))
//     {
//         /* Admins must acess and in RWX, set it */
//         if (RtlEqualSid(SeAliasAdminsSid, &Ace->SidStart))
//         {
//             SetFlag(Ace->Mask, (GENERIC_READ | GENERIC_WRITE | GENERIC_EXECUTE));
//             AdminsSet = TRUE;
//         }
//
//         /* World can read a CD_ROM device */
//         if (DeviceType == FILE_DEVICE_CD_ROM && RtlEqualSid(SeWorldSid, &Ace->SidStart))
//         {
//             SetFlag(Ace->Mask, GENERIC_READ);
//             WorldSet = TRUE;
//         }
//
//         ++AceId;
//     }
//
//     /* AdminSid was present and set (otherwise, we have big trouble) */
//     ASSERT(AdminsSet);
//
//     /* If CD_ROM device, we've set world permissions */
//     if (DeviceType == FILE_DEVICE_CD_ROM) ASSERT(WorldSet);
//
//     /* Now our DACL is correct, setup the security descriptor */
//     RtlCreateSecurityDescriptor(SecurityDescriptor, SECURITY_DESCRIPTOR_REVISION);
//     RtlSetDaclSecurityDescriptor(SecurityDescriptor, TRUE, Dacl, FALSE);
//
//     /* We've set DACL */
//     if (OutputFlags) *OutputFlags |= DACL_SET;
//
//     /* Return DACL to allow later freeing */
//     *OutputDacl = Dacl;
//     Status = STATUS_SUCCESS;
//
// Quit:
//     /* Only return SD if we succeed */
//     if (!NT_SUCCESS(Status))
//     {
//         return NULL;
//     }
//
//     return SecurityDescriptor;
// }
//
// /* PUBLIC FUNCTIONS ***********************************************************/
//
// /*
//  * IoAttachDevice
//  *
//  * Layers a device over the highest device in a device stack.
//  *
//  * Parameters
//  *    SourceDevice
//  *       Device to be attached.
//  *
//  *    TargetDevice
//  *       Name of the target device.
//  *
//  *    AttachedDevice
//  *       Caller storage for the device attached to.
//  *
//  * Status
//  *    @implemented
//  */
// NTSTATUS
// NTAPI
// IoAttachDevice(PDEVICE_OBJECT SourceDevice,
//                PUNICODE_STRING TargetDeviceName,
//                PDEVICE_OBJECT *AttachedDevice)
// {
//     NTSTATUS Status;
//     PFILE_OBJECT FileObject = NULL;
//     PDEVICE_OBJECT TargetDevice = NULL;
//
//     /* Call the helper routine for an attach operation */
//     Status = IopGetDeviceObjectPointer(TargetDeviceName,
//                                        FILE_READ_ATTRIBUTES,
//                                        &FileObject,
//                                        &TargetDevice,
//                                        IO_ATTACH_DEVICE_API);
//     if (!NT_SUCCESS(Status)) return Status;
//
//     /* Attach the device */
//     Status = IoAttachDeviceToDeviceStackSafe(SourceDevice,
//                                              TargetDevice,
//                                              AttachedDevice);
//
//     /* Dereference it */
//     ObDereferenceObject(FileObject);
//     return Status;
// }
//
// /*
//  * IoAttachDeviceByPointer
//  *
//  * Status
//  *    @implemented
//  */
// NTSTATUS
// NTAPI
// IoAttachDeviceByPointer(IN PDEVICE_OBJECT SourceDevice,
//                         IN PDEVICE_OBJECT TargetDevice)
// {
//     PDEVICE_OBJECT AttachedDevice;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     /* Do the Attach */
//     AttachedDevice = IoAttachDeviceToDeviceStack(SourceDevice, TargetDevice);
//     if (!AttachedDevice) Status = STATUS_NO_SUCH_DEVICE;
//
//     /* Return the status */
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// PDEVICE_OBJECT
// NTAPI
// IoAttachDeviceToDeviceStack(IN PDEVICE_OBJECT SourceDevice,
//                             IN PDEVICE_OBJECT TargetDevice)
// {
//     /* Attach it safely */
//     return IopAttachDeviceToDeviceStackSafe(SourceDevice,
//                                             TargetDevice,
//                                             NULL);
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// IoAttachDeviceToDeviceStackSafe(IN PDEVICE_OBJECT SourceDevice,
//                                 IN PDEVICE_OBJECT TargetDevice,
//                                 IN OUT PDEVICE_OBJECT *AttachedToDeviceObject)
// {
//     /* Call the internal function */
//     if (!IopAttachDeviceToDeviceStackSafe(SourceDevice,
//                                           TargetDevice,
//                                           AttachedToDeviceObject))
//     {
//         /* Nothing found */
//         return STATUS_NO_SUCH_DEVICE;
//     }
//
//     /* Success! */
//     return STATUS_SUCCESS;
// }
//
// /*
//  * IoCreateDevice
//  *
//  * Allocates memory for and intializes a device object for use for
//  * a driver.
//  *
//  * Parameters
//  *    DriverObject
//  *       Driver object passed by IO Manager when the driver was loaded.
//  *
//  *    DeviceExtensionSize
//  *       Number of bytes for the device extension.
//  *
//  *    DeviceName
//  *       Unicode name of device.
//  *
//  *    DeviceType
//  *       Device type of the new device.
//  *
//  *    DeviceCharacteristics
//  *       Bit mask of device characteristics.
//  *
//  *    Exclusive
//  *       TRUE if only one thread can access the device at a time.
//  *
//  *    DeviceObject
//  *       On successful return this parameter is filled by pointer to
//  *       allocated device object.
//  *
//  * Status
//  *    @implemented
//  */
// NTSTATUS
// NTAPI
// IoCreateDevice(IN PDRIVER_OBJECT DriverObject,
//                IN ULONG DeviceExtensionSize,
//                IN PUNICODE_STRING DeviceName,
//                IN DEVICE_TYPE DeviceType,
//                IN ULONG DeviceCharacteristics,
//                IN BOOLEAN Exclusive,
//                OUT PDEVICE_OBJECT *DeviceObject)
// {
//     WCHAR AutoNameBuffer[20];
//     UNICODE_STRING AutoName;
//     PDEVICE_OBJECT CreatedDeviceObject;
//     PDEVOBJ_EXTENSION DeviceObjectExtension;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     NTSTATUS Status;
//     ULONG AlignedDeviceExtensionSize;
//     ULONG TotalSize;
//     HANDLE TempHandle;
//     PACL Dacl;
//     SECURITY_DESCRIPTOR SecurityDescriptor, *ReturnedSD;
//     PAGED_CODE();
//
//     /* Check if we have to generate a name */
//     if (DeviceCharacteristics & FILE_AUTOGENERATED_DEVICE_NAME)
//     {
//         /* Generate it */
//         swprintf(AutoNameBuffer,
//                  L"\\Device\\%08lx",
//                  InterlockedIncrementUL(&IopDeviceObjectNumber));
//
//         /* Initialize the name */
//         RtlInitUnicodeString(&AutoName, AutoNameBuffer);
//         DeviceName = &AutoName;
//     }
//
//     /* Get the security descriptor */
//     ReturnedSD = IopCreateDefaultDeviceSecurityDescriptor(DeviceType,
//                                                           DeviceCharacteristics,
//                                                           DeviceName != NULL,
//                                                           &SecurityDescriptor,
//                                                           &Dacl,
//                                                           NULL);
//
//     /* Initialize the Object Attributes */
//     InitializeObjectAttributes(&ObjectAttributes,
//                                DeviceName,
//                                OBJ_KERNEL_HANDLE,
//                                NULL,
//                                ReturnedSD);
//
//     /* Honor exclusive flag */
//     if (Exclusive) ObjectAttributes.Attributes |= OBJ_EXCLUSIVE;
//
//     /* Create a permanent object for named devices */
//     if (DeviceName) ObjectAttributes.Attributes |= OBJ_PERMANENT;
//
//     /* Align the Extension Size to 8-bytes */
//     AlignedDeviceExtensionSize = ALIGN_UP_BY(DeviceExtensionSize,
//                                              MEMORY_ALLOCATION_ALIGNMENT);
//
//     /* Total Size */
//     TotalSize = AlignedDeviceExtensionSize +
//                 sizeof(DEVICE_OBJECT) +
//                 sizeof(EXTENDED_DEVOBJ_EXTENSION);
//
//     /* Create the Device Object */
//     *DeviceObject = NULL;
//     Status = ObCreateObject(KernelMode,
//                             IoDeviceObjectType,
//                             &ObjectAttributes,
//                             KernelMode,
//                             NULL,
//                             TotalSize,
//                             0,
//                             0,
//                             (PVOID*)&CreatedDeviceObject);
//     if (!NT_SUCCESS(Status))
//     {
//         if (Dacl != NULL) ExFreePoolWithTag(Dacl, 'eSoI');
//
//         return Status;
//     }
//
//     /* Clear the whole Object and extension so we don't null stuff manually */
//     RtlZeroMemory(CreatedDeviceObject, TotalSize);
//
//     /*
//      * Setup the Type and Size. Note that we don't use the aligned size,
//      * because that's only padding for the DevObjExt and not part of the Object.
//      */
//     CreatedDeviceObject->Type = IO_TYPE_DEVICE;
//     CreatedDeviceObject->Size = sizeof(DEVICE_OBJECT) + (USHORT)DeviceExtensionSize;
//
//     /* The kernel extension is after the driver internal extension */
//     DeviceObjectExtension = (PDEVOBJ_EXTENSION)
//                             ((ULONG_PTR)(CreatedDeviceObject + 1) +
//                              AlignedDeviceExtensionSize);
//
//     /* Set the Type and Size. Question: why is Size 0 on Windows? */
//     DeviceObjectExtension->Type = IO_TYPE_DEVICE_OBJECT_EXTENSION;
//     DeviceObjectExtension->Size = 0;
//
//     /* Initialize with Power Manager */
//     PoInitializeDeviceObject(DeviceObjectExtension as *mut _);
//
//     /* Link the Object and Extension */
//     DeviceObjectExtension->DeviceObject = CreatedDeviceObject;
//     CreatedDeviceObject->DeviceObjectExtension = DeviceObjectExtension;
//
//     /* Set Device Object Data */
//     CreatedDeviceObject->DeviceType = DeviceType;
//     CreatedDeviceObject->Characteristics = DeviceCharacteristics;
//     CreatedDeviceObject->DeviceExtension = DeviceExtensionSize ?
//                                            CreatedDeviceObject + 1 :
//                                            NULL;
//     CreatedDeviceObject->StackSize = 1;
//     CreatedDeviceObject->AlignmentRequirement = 0;
//
//     /* Set the Flags */
//     CreatedDeviceObject->Flags = DO_DEVICE_INITIALIZING;
//     if (Exclusive) CreatedDeviceObject->Flags |= DO_EXCLUSIVE;
//     if (DeviceName) CreatedDeviceObject->Flags |= DO_DEVICE_HAS_NAME;
//
//     /* Attach a Vpb for Disks and Tapes, and create the Device Lock */
//     if ((CreatedDeviceObject->DeviceType == FILE_DEVICE_DISK) ||
//         (CreatedDeviceObject->DeviceType == FILE_DEVICE_VIRTUAL_DISK) ||
//         (CreatedDeviceObject->DeviceType == FILE_DEVICE_CD_ROM) ||
//         (CreatedDeviceObject->DeviceType == FILE_DEVICE_TAPE))
//     {
//         /* Create Vpb */
//         Status = IopCreateVpb(CreatedDeviceObject);
//         if (!NT_SUCCESS(Status))
//         {
//             if (Dacl != NULL) ExFreePoolWithTag(Dacl, 'eSoI');
//
//             /* Dereference the device object and fail */
//             ObDereferenceObject(CreatedDeviceObject);
//             return Status;
//         }
//
//         /* Initialize Lock Event */
//         KeInitializeEvent(&CreatedDeviceObject->DeviceLock,
//                           SynchronizationEvent,
//                           TRUE);
//     }
//
//     /* Set the right Sector Size */
//     switch (DeviceType)
//     {
//         /* All disk systems */
//         case FILE_DEVICE_DISK_FILE_SYSTEM:
//         case FILE_DEVICE_DISK:
//         case FILE_DEVICE_VIRTUAL_DISK:
//
//             /* The default is 512 bytes */
//             CreatedDeviceObject->SectorSize  = 512;
//             break;
//
//         /* CD-ROM file systems */
//         case FILE_DEVICE_CD_ROM_FILE_SYSTEM:
//
//             /* The default is 2048 bytes */
//             CreatedDeviceObject->SectorSize = 2048;
//     }
//
//     /* Create the Device Queue */
//     if ((CreatedDeviceObject->DeviceType == FILE_DEVICE_DISK_FILE_SYSTEM) ||
//         (CreatedDeviceObject->DeviceType == FILE_DEVICE_FILE_SYSTEM) ||
//         (CreatedDeviceObject->DeviceType == FILE_DEVICE_CD_ROM_FILE_SYSTEM) ||
//         (CreatedDeviceObject->DeviceType == FILE_DEVICE_NETWORK_FILE_SYSTEM) ||
//         (CreatedDeviceObject->DeviceType == FILE_DEVICE_TAPE_FILE_SYSTEM))
//     {
//         /* Simple FS Devices, they don't need a real Device Queue */
//         InitializeListHead(&CreatedDeviceObject->Queue.ListEntry);
//     }
//     else
//     {
//         /* An actual Device, initialize its DQ */
//         KeInitializeDeviceQueue(&CreatedDeviceObject->DeviceQueue);
//     }
//
//     /* Insert the Object */
//     Status = ObInsertObject(CreatedDeviceObject,
//                             NULL,
//                             FILE_READ_DATA | FILE_WRITE_DATA,
//                             1,
//                             (PVOID*)&CreatedDeviceObject,
//                             &TempHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         if (Dacl != NULL) ExFreePoolWithTag(Dacl, 'eSoI');
//
//         return Status;
//     }
//
//     /* Now do the final linking */
//     ObReferenceObject(DriverObject);
//     ASSERT((DriverObject->Flags & DRVO_UNLOAD_INVOKED) == 0);
//     CreatedDeviceObject->DriverObject = DriverObject;
//     IopEditDeviceList(DriverObject, CreatedDeviceObject, IopAdd);
//
//     /* Link with the power manager */
//     if (CreatedDeviceObject->Vpb) PoVolumeDevice(CreatedDeviceObject);
//
//     /* Close the temporary handle and return to caller */
//     ObCloseHandle(TempHandle, KernelMode);
//     *DeviceObject = CreatedDeviceObject;
//
//     if (Dacl != NULL) ExFreePoolWithTag(Dacl, 'eSoI');
//
//     return STATUS_SUCCESS;
// }
//
// /*
//  * IoDeleteDevice
//  *
//  * Status
//  *    @implemented
//  */
// VOID
// NTAPI
// IoDeleteDevice(IN PDEVICE_OBJECT DeviceObject)
// {
//     PIO_TIMER Timer;
//
//     /* Check if the device is registered for shutdown notifications */
//     if (DeviceObject->Flags & DO_SHUTDOWN_REGISTERED)
//     {
//         /* Call the shutdown notifications */
//         IoUnregisterShutdownNotification(DeviceObject);
//     }
//
//     /* Check if it has a timer */
//     Timer = DeviceObject->Timer;
//     if (Timer)
//     {
//         /* Remove it and free it */
//         IopRemoveTimerFromTimerList(Timer);
//         ExFreePoolWithTag(Timer, TAG_IO_TIMER);
//     }
//
//     /* Check if the device has a name */
//     if (DeviceObject->Flags & DO_DEVICE_HAS_NAME)
//     {
//         /* It does, make it temporary so we can remove it */
//         ObMakeTemporaryObject(DeviceObject);
//     }
//
//     /* Set the pending delete flag */
//     IoGetDevObjExtension(DeviceObject)->ExtensionFlags |= DOE_DELETE_PENDING;
//
//     /* Unlink with the power manager */
//     if (DeviceObject->Vpb) PoRemoveVolumeDevice(DeviceObject);
//
//     /* Check if the device object can be unloaded */
//     if (!DeviceObject->ReferenceCount) IopUnloadDevice(DeviceObject);
// }
//
// /*
//  * IoDetachDevice
//  *
//  * Status
//  *    @implemented
//  */
// VOID
// NTAPI
// IoDetachDevice(IN PDEVICE_OBJECT TargetDevice)
// {
//     PEXTENDED_DEVOBJ_EXTENSION DeviceExtension;
//
//     /* Sanity check */
//     DeviceExtension = IoGetDevObjExtension(TargetDevice->AttachedDevice);
//     ASSERT(DeviceExtension->AttachedTo == TargetDevice);
//
//     /* Remove the attachment */
//     DeviceExtension->AttachedTo = NULL;
//     TargetDevice->AttachedDevice = NULL;
//
//     /* Check if it's ok to delete this device */
//     if ((IoGetDevObjExtension(TargetDevice)->ExtensionFlags & DOE_DELETE_PENDING) &&
//         !(TargetDevice->ReferenceCount))
//     {
//         /* It is, do it */
//         IopUnloadDevice(TargetDevice);
//     }
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// IoEnumerateDeviceObjectList(IN  PDRIVER_OBJECT DriverObject,
//                             IN  PDEVICE_OBJECT *DeviceObjectList,
//                             IN  ULONG DeviceObjectListSize,
//                             OUT PULONG ActualNumberDeviceObjects)
// {
//     ULONG ActualDevices = 1;
//     PDEVICE_OBJECT CurrentDevice = DriverObject->DeviceObject;
//     KIRQL OldIrql;
//
//     /* Lock the Device list while we enumerate it */
//     OldIrql = KeAcquireQueuedSpinLock(LockQueueIoDatabaseLock);
//
//     /* Find out how many devices we'll enumerate */
//     while ((CurrentDevice = CurrentDevice->NextDevice)) ActualDevices++;
//
//     /* Go back to the first */
//     CurrentDevice = DriverObject->DeviceObject;
//
//     /* Start by at least returning this */
//     *ActualNumberDeviceObjects = ActualDevices;
//
//     /* Check if we can support so many */
//     if ((ActualDevices * sizeof(PDEVICE_OBJECT)) > DeviceObjectListSize)
//     {
//         /* Fail because the buffer was too small */
//         KeReleaseQueuedSpinLock(LockQueueIoDatabaseLock, OldIrql);
//         return STATUS_BUFFER_TOO_SMALL;
//     }
//
//     /* Check if the caller wanted the device list */
//     if (DeviceObjectList)
//     {
//         /* Loop through all the devices */
//         while (ActualDevices)
//         {
//             /* Reference each Device */
//             ObReferenceObject(CurrentDevice);
//
//             /* Add it to the list */
//             *DeviceObjectList = CurrentDevice;
//
//             /* Go to the next one */
//             CurrentDevice = CurrentDevice->NextDevice;
//             ActualDevices--;
//             DeviceObjectList++;
//         }
//     }
//
//     /* Release the device list lock */
//     KeReleaseQueuedSpinLock(LockQueueIoDatabaseLock, OldIrql);
//
//     /* Return the status */
//     return STATUS_SUCCESS;
// }
//
// /*
//  * IoGetAttachedDevice
//  *
//  * Status
//  *    @implemented
//  */
// PDEVICE_OBJECT
// NTAPI
// IoGetAttachedDevice(PDEVICE_OBJECT DeviceObject)
// {
//     /* Get the last attached device */
//     while (DeviceObject->AttachedDevice)
//     {
//         /* Move to the next one */
//         DeviceObject = DeviceObject->AttachedDevice;
//     }
//
//     /* Return it */
//     return DeviceObject;
// }
//
// /*
//  * IoGetAttachedDeviceReference
//  *
//  * Status
//  *    @implemented
//  */
// PDEVICE_OBJECT
// NTAPI
// IoGetAttachedDeviceReference(PDEVICE_OBJECT DeviceObject)
// {
//     /* Reference the Attached Device */
//     DeviceObject = IoGetAttachedDevice(DeviceObject);
//     ObReferenceObject(DeviceObject);
//     return DeviceObject;
// }
//
// /*
//  * @implemented
//  */
// PDEVICE_OBJECT
// NTAPI
// IoGetDeviceAttachmentBaseRef(IN PDEVICE_OBJECT DeviceObject)
// {
//     /* Reference the lowest attached device */
//     DeviceObject = IopGetLowestDevice(DeviceObject);
//     ObReferenceObject(DeviceObject);
//     return DeviceObject;
// }
//
// /*
//  * IoGetDeviceObjectPointer
//  *
//  * Status
//  *    @implemented
//  */
// NTSTATUS
// NTAPI
// IoGetDeviceObjectPointer(IN PUNICODE_STRING ObjectName,
//                          IN ACCESS_MASK DesiredAccess,
//                          OUT PFILE_OBJECT *FileObject,
//                          OUT PDEVICE_OBJECT *DeviceObject)
// {
//     /* Call the helper routine for a normal operation */
//     return IopGetDeviceObjectPointer(ObjectName,
//                                      DesiredAccess,
//                                      FileObject,
//                                      DeviceObject,
//                                      0);
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// IoGetDiskDeviceObject(IN PDEVICE_OBJECT FileSystemDeviceObject,
//                       OUT PDEVICE_OBJECT *DiskDeviceObject)
// {
//     PEXTENDED_DEVOBJ_EXTENSION DeviceExtension;
//     PVPB Vpb;
//     KIRQL OldIrql;
//     NTSTATUS Status;
//
//     /* Make sure there's a VPB */
//     if (!FileSystemDeviceObject->Vpb) return STATUS_INVALID_PARAMETER;
//
//     /* Acquire it */
//     IoAcquireVpbSpinLock(&OldIrql);
//
//     /* Get the Device Extension */
//     DeviceExtension = IoGetDevObjExtension(FileSystemDeviceObject);
//
//     /* Make sure this one has a VPB too */
//     Vpb = DeviceExtension->Vpb;
//     if (Vpb)
//     {
//         /* Make sure that it's mounted */
//         if ((Vpb->ReferenceCount) &&
//             (Vpb->Flags & VPB_MOUNTED))
//         {
//             /* Return the Disk Device Object */
//             *DiskDeviceObject = Vpb->RealDevice;
//
//             /* Reference it and return success */
//             ObReferenceObject(Vpb->RealDevice);
//             Status = STATUS_SUCCESS;
//         }
//         else
//         {
//             /* It's not, so return failure */
//             Status = STATUS_VOLUME_DISMOUNTED;
//         }
//     }
//     else
//     {
//         /* Fail */
//         Status = STATUS_INVALID_PARAMETER;
//     }
//
//     /* Release the lock */
//     IoReleaseVpbSpinLock(OldIrql);
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// PDEVICE_OBJECT
// NTAPI
// IoGetLowerDeviceObject(IN PDEVICE_OBJECT DeviceObject)
// {
//     PEXTENDED_DEVOBJ_EXTENSION DeviceExtension;
//     PDEVICE_OBJECT LowerDeviceObject = NULL;
//
//     /* Make sure it's not getting deleted */
//     DeviceExtension = IoGetDevObjExtension(DeviceObject);
//     if (!(DeviceExtension->ExtensionFlags & (DOE_UNLOAD_PENDING |
//                                            DOE_DELETE_PENDING |
//                                            DOE_REMOVE_PENDING |
//                                            DOE_REMOVE_PROCESSED)))
//     {
//         /* Get the Lower Device Object */
//         LowerDeviceObject = DeviceExtension->AttachedTo;
//
//         /* Check that we got a valid device object */
//         if (LowerDeviceObject)
//         {
//             /* We did so let's reference it */
//             ObReferenceObject(LowerDeviceObject);
//         }
//     }
//
//     /* Return it */
//     return LowerDeviceObject;
// }
//
// /*
//  * @implemented
//  */
// PDEVICE_OBJECT
// NTAPI
// IoGetRelatedDeviceObject(IN PFILE_OBJECT FileObject)
// {
//     PDEVICE_OBJECT DeviceObject = FileObject->DeviceObject;
//
//     /* Check if we have a VPB with a device object */
//     if ((FileObject->Vpb) && (FileObject->Vpb->DeviceObject))
//     {
//         /* Then use the DO from the VPB */
//         ASSERT(!(FileObject->Flags & FO_DIRECT_DEVICE_OPEN));
//         DeviceObject = FileObject->Vpb->DeviceObject;
//     }
//     else if (!(FileObject->Flags & FO_DIRECT_DEVICE_OPEN) &&
//               (FileObject->DeviceObject->Vpb) &&
//               (FileObject->DeviceObject->Vpb->DeviceObject))
//     {
//         /* The disk device actually has a VPB, so get the DO from there */
//         DeviceObject = FileObject->DeviceObject->Vpb->DeviceObject;
//     }
//     else
//     {
//         /* Otherwise, this was a direct open */
//         DeviceObject = FileObject->DeviceObject;
//     }
//
//     /* Sanity check */
//     ASSERT(DeviceObject != NULL);
//
//     /* Check if we were attached */
//     if (DeviceObject->AttachedDevice)
//     {
//         /* Check if the file object has an extension present */
//         if (FileObject->Flags & FO_FILE_OBJECT_HAS_EXTENSION)
//         {
//             /* Sanity check, direct open files can't have this */
//             ASSERT(!(FileObject->Flags & FO_DIRECT_DEVICE_OPEN));
//
//             /* Check if the extension is really present */
//             if (FileObject->FileObjectExtension)
//             {
//                 PFILE_OBJECT_EXTENSION FileObjectExtension;
//
//                 /* Cast the buffer to something we understand */
//                 FileObjectExtension = FileObject->FileObjectExtension;
//
//                 /* Check if have a valid replacement top level device */
//                 if (FileObjectExtension->TopDeviceObjectHint &&
//                     IopVerifyDeviceObjectOnStack(DeviceObject,
//                                                  FileObjectExtension->TopDeviceObjectHint))
//                 {
//                     /* Use this instead of returning the top level device */
//                     return FileObjectExtension->TopDeviceObjectHint;
//                 }
//             }
//         }
//
//         /* Return the highest attached device */
//         DeviceObject = IoGetAttachedDevice(DeviceObject);
//     }
//
//     /* Return the DO we found */
//     return DeviceObject;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// IoGetRelatedTargetDevice(IN PFILE_OBJECT FileObject,
//                          OUT PDEVICE_OBJECT *DeviceObject)
// {
//     NTSTATUS Status;
//     PDEVICE_NODE DeviceNode = NULL;
//
//     /* Call the internal helper function */
//     Status = IopGetRelatedTargetDevice(FileObject, &DeviceNode);
//     if (NT_SUCCESS(Status) && DeviceNode)
//     {
//         *DeviceObject = DeviceNode->PhysicalDeviceObject;
//     }
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// PDEVICE_OBJECT
// NTAPI
// IoGetBaseFileSystemDeviceObject(IN PFILE_OBJECT FileObject)
// {
//     PDEVICE_OBJECT DeviceObject;
//
//     /*
//     * If the FILE_OBJECT's VPB is defined,
//     * get the device from it.
//     */
//     if ((FileObject->Vpb) && (FileObject->Vpb->DeviceObject))
//     {
//         /* Use the VPB's Device Object's */
//         DeviceObject = FileObject->Vpb->DeviceObject;
//     }
//     else if (!(FileObject->Flags & FO_DIRECT_DEVICE_OPEN) &&
//              (FileObject->DeviceObject->Vpb) &&
//              (FileObject->DeviceObject->Vpb->DeviceObject))
//     {
//         /* Use the VPB's File System Object */
//         DeviceObject = FileObject->DeviceObject->Vpb->DeviceObject;
//     }
//     else
//     {
//         /* Use the FO's Device Object */
//         DeviceObject = FileObject->DeviceObject;
//     }
//
//     /* Return the device object we found */
//     ASSERT(DeviceObject != NULL);
//     return DeviceObject;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// IoRegisterLastChanceShutdownNotification(IN PDEVICE_OBJECT DeviceObject)
// {
//     PSHUTDOWN_ENTRY Entry;
//
//     /* Allocate the shutdown entry */
//     Entry = ExAllocatePoolWithTag(NonPagedPool,
//                                   sizeof(SHUTDOWN_ENTRY),
//                                   TAG_SHUTDOWN_ENTRY);
//     if (!Entry) return STATUS_INSUFFICIENT_RESOURCES;
//
//     /* Set the DO */
//     Entry->DeviceObject = DeviceObject;
//
//     /* Reference it so it doesn't go away */
//     ObReferenceObject(DeviceObject);
//
//     /* Insert it into the list */
//     ExInterlockedInsertHeadList(&LastChanceShutdownListHead,
//                                 &Entry->ShutdownList,
//                                 &ShutdownListLock);
//
//     /* Set the shutdown registered flag */
//     DeviceObject->Flags |= DO_SHUTDOWN_REGISTERED;
//     return STATUS_SUCCESS;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// IoRegisterShutdownNotification(PDEVICE_OBJECT DeviceObject)
// {
//     PSHUTDOWN_ENTRY Entry;
//
//     /* Allocate the shutdown entry */
//     Entry = ExAllocatePoolWithTag(NonPagedPool,
//                                   sizeof(SHUTDOWN_ENTRY),
//                                   TAG_SHUTDOWN_ENTRY);
//     if (!Entry) return STATUS_INSUFFICIENT_RESOURCES;
//
//     /* Set the DO */
//     Entry->DeviceObject = DeviceObject;
//
//     /* Reference it so it doesn't go away */
//     ObReferenceObject(DeviceObject);
//
//     /* Insert it into the list */
//     ExInterlockedInsertHeadList(&ShutdownListHead,
//                                 &Entry->ShutdownList,
//                                 &ShutdownListLock);
//
//     /* Set the shutdown registered flag */
//     DeviceObject->Flags |= DO_SHUTDOWN_REGISTERED;
//     return STATUS_SUCCESS;
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// IoUnregisterShutdownNotification(PDEVICE_OBJECT DeviceObject)
// {
//     PSHUTDOWN_ENTRY ShutdownEntry;
//     PLIST_ENTRY NextEntry;
//     KIRQL OldIrql;
//
//     /* Remove the flag */
//     DeviceObject->Flags &= ~DO_SHUTDOWN_REGISTERED;
//
//     /* Acquire the shutdown lock and loop the shutdown list */
//     KeAcquireSpinLock(&ShutdownListLock, &OldIrql);
//     NextEntry = ShutdownListHead.Flink;
//     while (NextEntry != &ShutdownListHead)
//     {
//         /* Get the entry */
//         ShutdownEntry = CONTAINING_RECORD(NextEntry,
//                                           SHUTDOWN_ENTRY,
//                                           ShutdownList);
//
//         /* Get if the DO matches */
//         if (ShutdownEntry->DeviceObject == DeviceObject)
//         {
//             /* Remove it from the list */
//             RemoveEntryList(NextEntry);
//             NextEntry = NextEntry->Blink;
//
//             /* Free the entry */
//             ExFreePoolWithTag(ShutdownEntry, TAG_SHUTDOWN_ENTRY);
//
//             /* Get rid of our reference to it */
//             ObDereferenceObject(DeviceObject);
//         }
//
//         /* Go to the next entry */
//         NextEntry = NextEntry->Flink;
//     }
//
//     /* Now loop the last chance list */
//     NextEntry = LastChanceShutdownListHead.Flink;
//     while (NextEntry != &LastChanceShutdownListHead)
//     {
//         /* Get the entry */
//         ShutdownEntry = CONTAINING_RECORD(NextEntry,
//                                           SHUTDOWN_ENTRY,
//                                           ShutdownList);
//
//         /* Get if the DO matches */
//         if (ShutdownEntry->DeviceObject == DeviceObject)
//         {
//             /* Remove it from the list */
//             RemoveEntryList(NextEntry);
//             NextEntry = NextEntry->Blink;
//
//             /* Free the entry */
//             ExFreePoolWithTag(ShutdownEntry, TAG_SHUTDOWN_ENTRY);
//
//             /* Get rid of our reference to it */
//             ObDereferenceObject(DeviceObject);
//         }
//
//         /* Go to the next entry */
//         NextEntry = NextEntry->Flink;
//     }
//
//     /* Release the shutdown lock */
//     KeReleaseSpinLock(&ShutdownListLock, OldIrql);
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// IoSetStartIoAttributes(IN PDEVICE_OBJECT DeviceObject,
//                        IN BOOLEAN DeferredStartIo,
//                        IN BOOLEAN NonCancelable)
// {
//     PEXTENDED_DEVOBJ_EXTENSION DeviceExtension;
//
//     /* Get the Device Extension */
//     DeviceExtension = IoGetDevObjExtension(DeviceObject);
//
//     /* Set the flags the caller requested */
//     DeviceExtension->StartIoFlags |= (DeferredStartIo) ? DOE_SIO_DEFERRED : 0;
//     DeviceExtension->StartIoFlags |= (NonCancelable) ? DOE_SIO_NO_CANCEL : 0;
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// IoStartNextPacketByKey(IN PDEVICE_OBJECT DeviceObject,
//                        IN BOOLEAN Cancelable,
//                        IN ULONG Key)
// {
//     PEXTENDED_DEVOBJ_EXTENSION DeviceExtension;
//
//     /* Get the Device Extension */
//     DeviceExtension = IoGetDevObjExtension(DeviceObject);
//
//     /* Check if deferred start was requested */
//     if (DeviceExtension->StartIoFlags & DOE_SIO_DEFERRED)
//     {
//         /* Call our internal function to handle the defered case */
//         IopStartNextPacketByKeyEx(DeviceObject,
//                                   Key,
//                                   DOE_SIO_WITH_KEY |
//                                   (Cancelable ? DOE_SIO_CANCELABLE : 0));
//     }
//     else
//     {
//         /* Call the normal routine */
//         IopStartNextPacketByKey(DeviceObject, Cancelable, Key);
//     }
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// IoStartNextPacket(IN PDEVICE_OBJECT DeviceObject,
//                   IN BOOLEAN Cancelable)
// {
//     PEXTENDED_DEVOBJ_EXTENSION DeviceExtension;
//
//     /* Get the Device Extension */
//     DeviceExtension = IoGetDevObjExtension(DeviceObject);
//
//     /* Check if deferred start was requested */
//     if (DeviceExtension->StartIoFlags & DOE_SIO_DEFERRED)
//     {
//         /* Call our internal function to handle the defered case */
//         IopStartNextPacketByKeyEx(DeviceObject,
//                                   0,
//                                   DOE_SIO_NO_KEY |
//                                   (Cancelable ? DOE_SIO_CANCELABLE : 0));
//     }
//     else
//     {
//         /* Call the normal routine */
//         IopStartNextPacket(DeviceObject, Cancelable);
//     }
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// IoStartPacket(IN PDEVICE_OBJECT DeviceObject,
//               IN PIRP Irp,
//               IN PULONG Key,
//               IN PDRIVER_CANCEL CancelFunction)
// {
//     BOOLEAN Stat;
//     KIRQL OldIrql, CancelIrql;
//
//     /* Raise to dispatch level */
//     KeRaiseIrql(DISPATCH_LEVEL, &OldIrql);
//
//     /* Check if we should acquire the cancel lock */
//     if (CancelFunction)
//     {
//         /* Acquire and set it */
//         IoAcquireCancelSpinLock(&CancelIrql);
//         Irp->CancelRoutine = CancelFunction;
//     }
//
//     /* Check if we have a key */
//     if (Key)
//     {
//         /* Insert by key */
//         Stat = KeInsertByKeyDeviceQueue(&DeviceObject->DeviceQueue,
//                                         &Irp->Tail.Overlay.DeviceQueueEntry,
//                                         *Key);
//     }
//     else
//     {
//         /* Insert without a key */
//         Stat = KeInsertDeviceQueue(&DeviceObject->DeviceQueue,
//                                    &Irp->Tail.Overlay.DeviceQueueEntry);
//     }
//
//     /* Check if this was a first insert */
//     if (!Stat)
//     {
//         /* Set the IRP */
//         DeviceObject->CurrentIrp = Irp;
//
//         /* Check if this is a cancelable packet */
//         if (CancelFunction)
//         {
//             /* Check if the caller requested no cancellation */
//             if (IoGetDevObjExtension(DeviceObject)->StartIoFlags &
//                 DOE_SIO_NO_CANCEL)
//             {
//                 /* He did, so remove the cancel routine */
//                 Irp->CancelRoutine = NULL;
//             }
//
//             /* Release the cancel lock */
//             IoReleaseCancelSpinLock(CancelIrql);
//         }
//
//         /* Call the Start I/O function */
//         DeviceObject->DriverObject->DriverStartIo(DeviceObject, Irp);
//     }
//     else
//     {
//         /* The packet was inserted... check if we have a cancel function */
//         if (CancelFunction)
//         {
//             /* Check if the IRP got cancelled */
//             if (Irp->Cancel)
//             {
//                 /*
//                  * Set the cancel IRQL, clear the currnet cancel routine and
//                  * call ours
//                  */
//                 Irp->CancelIrql = CancelIrql;
//                 Irp->CancelRoutine = NULL;
//                 CancelFunction(DeviceObject, Irp);
//             }
//             else
//             {
//                 /* Otherwise, release the lock */
//                 IoReleaseCancelSpinLock(CancelIrql);
//             }
//         }
//     }
//
//     /* Return back to previous IRQL */
//     KeLowerIrql(OldIrql);
// }
//
// #if defined (_WIN64)
// ULONG
// NTAPI
// IoWMIDeviceObjectToProviderId(
//     IN PDEVICE_OBJECT DeviceObject)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
// #endif
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
