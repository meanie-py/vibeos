use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::macros::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ps::PsGetCurrentProcess;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::probe::*;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

use ntkrnl_sys::ddk::{
    _EPROCESS,
    _ETHREAD,
    _OBJECT_HEADER,
    _OBJECT_TYPE,
    _DEVICE_OBJECT,
    _FILE_OBJECT,
    _DRIVER_OBJECT,
    _VPB,
    _IRP,
    _IO_STACK_LOCATION,
    _DUMMY_FILE_OBJECT,
    _FILE_BASIC_INFORMATION,
    _FILE_NETWORK_OPEN_INFORMATION,
    _FILE_NAME_INFORMATION,
    _REPARSE_DATA_BUFFER,
    _OPEN_PACKET,
    _OBJECT_ATTRIBUTES,
    _IO_STATUS_BLOCK,
    _IO_SECURITY_CONTEXT,
    _ACCESS_STATE,
    _SECURITY_QUALITY_OF_SERVICE,
    _SECURITY_DESCRIPTOR,
    _OBJECT_NAME_INFORMATION,
    _GENERIC_MAPPING,
    _PRIVILEGE_SET,
    _NAMED_PIPE_CREATE_PARAMETERS,
    _MAILSLOT_CREATE_PARAMETERS,
    _FILE_OBJECT_EXTENSION,
    _SHARE_ACCESS,
    _ERESOURCE,
    _UNICODE_STRING,
    _KEVENT,
    _LUID,
    PEPROCESS,
    PETHREAD,
    POBJECT_HEADER,
    POBJECT_TYPE,
    PDEVICE_OBJECT,
    PFILE_OBJECT,
    PDRIVER_OBJECT,
    PVPB,
    PIRP,
    PIO_STACK_LOCATION,
    PDUMMY_FILE_OBJECT,
    PFILE_BASIC_INFORMATION,
    PFILE_NETWORK_OPEN_INFORMATION,
    PFILE_NAME_INFORMATION,
    PREPARSE_DATA_BUFFER,
    POPEN_PACKET,
    POBJECT_ATTRIBUTES,
    PIO_STATUS_BLOCK,
    PIO_SECURITY_CONTEXT,
    PACCESS_STATE,
    PSECURITY_QUALITY_OF_SERVICE,
    PSECURITY_DESCRIPTOR,
    POBJECT_NAME_INFORMATION,
    PGENERIC_MAPPING,
    PPRIVILEGE_SET,
    PNAMED_PIPE_CREATE_PARAMETERS,
    PMAILSLOT_CREATE_PARAMETERS,
    PFILE_OBJECT_EXTENSION,
    PSHARE_ACCESS,
    PERESOURCE,
    PUNICODE_STRING,
    ACCESS_MASK,
    KPROCESSOR_MODE,
    HANDLE,
    OB_OPEN_REASON,
    SECURITY_OPERATION_CODE,
    FILE_INFORMATION_CLASS,
    CREATE_FILE_TYPE,
    ExAcquireRundownProtection,
    ExReleaseRundownProtection,
    ExAcquireResourceExclusiveLite,
    ExReleaseResourceLite,
    ExAcquireResourceSharedLite,
    ExGetPreviousMode,
    IoGetAttachedDevice,
    IoGetRelatedDeviceObject,
    IoAllocateIrp,
    IoFreeIrp,
    IoGetNextIrpStackLocation,
    KeInitializeEvent,
    KeWaitForSingleObject,
    KeClearEvent,
    KeSetEvent,
    KeAcquireQueuedSpinLock,
    KeReleaseQueuedSpinLock,
    ObCreateObject,
    ObInsertObject,
    ObOpenObjectByName,
    ObCloseHandle,
    ObReferenceSecurityDescriptor,
    ObDereferenceSecurityDescriptor,
    ObLogSecurityDescriptor,
    ObFreeObjectCreateInfoBuffer,
    ObReferenceObjectByHandle,
    ObQueryNameString,
    SeAccessCheck,
    SeAppendPrivileges,
    SeFreePrivileges,
    SePrivilegeCheck,
    SeLockSubjectContext,
    SeUnlockSubjectContext,
    SeOpenObjectAuditAlarm,
    SeFastTraverseCheck,
    SeSetSecurityDescriptorInfo,
    SeQuerySecurityDescriptorInfo,
    SeSetWorldSecurityDescriptor,
    SeSetAccessStateGenericMapping,
    PsGetCurrentThread,
    IopCheckVpbMounted,
    IopDereferenceDeviceObject,
    IopDereferenceVpbAndFree,
    IopVerifyDeviceObjectOnStack,
    IopAllocateIrpMustSucceed,
    IoVolumeDeviceToDosName,
    IoQueryFileInformation,
    _EVENT_TYPE_SynchronizationEvent,
    _EVENT_TYPE_NotificationEvent,
    _KWAIT_REASON_Executive,
    _KWAIT_REASON_UserRequest,
    _SECURITY_OPERATION_CODE_QuerySecurityDescriptor,
    _SECURITY_OPERATION_CODE_SetSecurityDescriptor,
    _SECURITY_OPERATION_CODE_DeleteSecurityDescriptor,
    _SECURITY_OPERATION_CODE_AssignSecurityDescriptor,
};

use core::mem::{zeroed, size_of};
use core::ptr::{null, null_mut, addr_of_mut, write_bytes, copy_nonoverlapping, copy};
use core::ffi::c_void;

extern "system" {
    static mut IopSecurityResource: _ERESOURCE;
    static mut SeBackupPrivilege: _LUID;
    static mut SeRestorePrivilege: _LUID;
    static mut IoFileObjectType: *mut _OBJECT_TYPE;
    static mut IopFileMapping: _GENERIC_MAPPING;
    fn ProbeForRead(Address: *const core::ffi::c_void, Length: usize, Alignment: usize);
    fn ProbeForWrite(Address: *mut core::ffi::c_void, Length: usize, Alignment: usize);
    fn RtlMapGenericMask(AccessMask: *mut ACCESS_MASK, GenericMapping: PGENERIC_MAPPING);
    fn RtlRaiseStatus(Status: NTSTATUS);
    fn RtlCopyUnicodeString(DestinationString: PUNICODE_STRING, SourceString: *const _UNICODE_STRING);
    fn IoCheckEaBufferValidity(EaBuffer: *mut core::ffi::c_void, EaLength: ULONG, ErrorOffset: *mut ULONG) -> NTSTATUS;
    fn IoCallDriver(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS;
    fn IoCancelIrp(Irp: PIRP) -> BOOLEAN;
    fn KeEnterCriticalRegion();
    fn KeLeaveCriticalRegion();
    fn KeFlushWriteBuffer();
    fn KeDelayExecutionThread(WaitMode: KPROCESSOR_MODE, Alertable: BOOLEAN, Interval: *const LARGE_INTEGER) -> NTSTATUS;
    fn ExFreePool(P: PVOID);
    fn ZwClose(Handle: HANDLE) -> NTSTATUS;
    #[link_name = "IopValidateOpenPacket_shim"]
    fn IopValidateOpenPacket(OpenPacket: POPEN_PACKET) -> BOOLEAN;
    #[link_name = "IopUpdateOperationCount_shim"]
    fn IopUpdateOperationCount(TransferType: ULONG);
    #[link_name = "IopLockFileObject_shim"]
    fn IopLockFileObject(FileObject: PFILE_OBJECT, PreviousMode: KPROCESSOR_MODE) -> NTSTATUS;
    #[link_name = "IopUnlockFileObject_shim"]
    fn IopUnlockFileObject(FileObject: PFILE_OBJECT);
    fn IopGetFileInformation(FileObject: PFILE_OBJECT, Length: ULONG, FileInformationClass: FILE_INFORMATION_CLASS, FileInformation: PVOID, ReturnedLength: *mut ULONG) -> NTSTATUS;
    fn IopCleanupFailedIrp(FileObject: PFILE_OBJECT, EventObject: *mut _KEVENT, LocalEvent: *mut _KEVENT) -> NTSTATUS;
    fn FsRtlPTeardownPerFileObjectContexts(FileObject: PFILE_OBJECT);
    #[link_name = "IopQueueIrpToThread_shim"]
    fn IopQueueIrpToThread(Irp: PIRP);
    #[link_name = "IopUnQueueIrpFromThread_shim"]
    fn IopUnQueueIrpFromThread(Irp: PIRP);
    static mut SeDefaultObjectMethod: *const core::ffi::c_void;
    static mut PsInitialSystemProcess: PVOID;
}
// The link_name target IoGetDevObjExtension_shim is defined extern "system" in
// inlines.rs — the decl MUST match the shim's ABI, not IoGetDevObjExtension's.
unsafe extern "system" {
    #[link_name = "IoGetDevObjExtension_shim"]
    fn IoGetDevObjExtension(DeviceObject: PDEVICE_OBJECT) -> *mut ntkrnl_sys::ddk::_DEVOBJ_EXTENSION;
}

extern "system" {
    fn ExAllocatePoolWithTag(PoolType: ULONG, NumberOfBytes: usize, Tag: ULONG) -> PVOID;
    fn ExFreePoolWithTag(P: PVOID, Tag: ULONG);
    fn ObDereferenceObject(Object: PVOID);
    fn ExRaiseStatus(Status: NTSTATUS);
}
// These are extern "C" in shim_exports.rs — must match.
extern "C" {
    fn ProbeForReadLargeInteger(Address: *const LARGE_INTEGER) -> LARGE_INTEGER;
    fn ProbeForWriteHandle(Address: *mut HANDLE);
    fn ProbeForWriteIoStatusBlock(IoStatusBlock: PIO_STATUS_BLOCK);
    fn ObReferenceObject(Object: PVOID);
}
unsafe extern "system" {
    fn KeRaiseIrql(NewIrql: KIRQL, OldIrql: *mut KIRQL);
    fn KeLowerIrql(NewIrql: KIRQL);
}

// InterlockedExchange is fastcall on i386, cdecl on others
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn InterlockedExchange(Target: *mut LONG, Value: LONG) -> LONG;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn InterlockedExchange(Target: *mut LONG, Value: LONG) -> LONG;
}

const TAG_IO: ULONG = 0x20206F49;      // '  OI' — generic IO pool tag
const TAG_IO_NAME: ULONG = 0x6D4E6F49;  // 'N OI' — IO name pool tag
const TAG_EA: ULONG = 0x61456F49;        // '  AE' — EA pool tag

const IOP_MAX_REPARSE_TRAVERSAL: ULONG = 0x20;
const IopOtherTransfer: ULONG = 0x00000002;

const IOP_CREATE_FILE_OBJECT_EXTENSION: ULONG = 0x02;
const IOP_USE_TOP_LEVEL_DEVICE_HINT: ULONG = 0x01;

const FILE_GENERIC_READ: ACCESS_MASK = 0x00120089;
const FILE_GENERIC_WRITE: ACCESS_MASK = 0x00120116;
const FILE_TRAVERSE: ACCESS_MASK = 0x00000020;
const FILE_ADD_FILE: ACCESS_MASK = 0x00000002;
const FILE_ADD_SUBDIRECTORY: ACCESS_MASK = 0x00000004;
const FILE_SHARE_DELETE: ULONG = 0x00000004;

const STATUS_NO_SUCH_DEVICE: NTSTATUS = 0xC000000E_u32 as i32;
const STATUS_IO_REPARSE_DATA_INVALID: NTSTATUS = 0xC0000278_u32 as i32;
const STATUS_MOUNT_POINT_NOT_RESOLVED: NTSTATUS = 0xC0000368_u32 as i32;
const STATUS_INVALID_DEVICE_OBJECT_PARAMETER: NTSTATUS = 0xC0000369_u32 as i32;
const STATUS_INVALID_BUFFER_SIZE: NTSTATUS = 0xC0000206_u32 as i32;

const IO_TYPE_OPEN_PACKET: USHORT = 8;  // from iotypes.h
const IO_TYPE_FILE: USHORT = 0x05;
const IO_TYPE_DEVICE: USHORT = 0x03;

const SE_BACKUP_PRIVILEGES_CHECKED: ULONG = 0x0100;
const TOKEN_HAS_BACKUP_PRIVILEGE: ULONG = 0x02;
const TOKEN_HAS_RESTORE_PRIVILEGE: ULONG = 0x04;
const TOKEN_HAS_TRAVERSE_PRIVILEGE: ULONG = 0x01;
const TOKEN_IS_RESTRICTED: ULONG = 0x10;

const IO_REPARSE: ULONG = 0x0;
const IO_REMOUNT: ULONG = 0x1;
const IO_REPARSE_TAG_MOUNT_POINT: ULONG = 0xA0000003;

const REPARSE_DATA_BUFFER_HEADER_SIZE: ULONG = 8;
const MAXIMUM_REPARSE_DATA_BUFFER_SIZE: ULONG = 16384;

const FILE_OPEN: ULONG = 0x00000001;
const FILE_CREATE: ULONG = 0x00000002;
const FILE_OPEN_IF: ULONG = 0x00000003;
const FILE_MAXIMUM_DISPOSITION: ULONG = 0x00000005;

const FILE_DIRECTORY_FILE: ULONG = 0x00000001;
const FILE_NON_DIRECTORY_FILE: ULONG = 0x00000040;
const FILE_WRITE_THROUGH: ULONG = 0x00000002;
const FILE_SEQUENTIAL_ONLY: ULONG = 0x00000004;
const FILE_NO_INTERMEDIATE_BUFFERING: ULONG = 0x00000008;
const FILE_SYNCHRONOUS_IO_ALERT: ULONG = 0x00000010;
const FILE_SYNCHRONOUS_IO_NONALERT: ULONG = 0x00000020;
const FILE_RANDOM_ACCESS: ULONG = 0x00000800;
const FILE_DELETE_ON_CLOSE: ULONG = 0x00001000;
const FILE_OPEN_BY_FILE_ID: ULONG = 0x00002000;
const FILE_OPEN_FOR_BACKUP_INTENT: ULONG = 0x00004000;
const FILE_NO_COMPRESSION: ULONG = 0x00008000;
const FILE_OPEN_REPARSE_POINT: ULONG = 0x00200000;
const FILE_COMPLETE_IF_OPLOCKED: ULONG = 0x00000100;
const FILE_RESERVE_OPFILTER: ULONG = 0x00100000;
const FILE_OPEN_FOR_FREE_SPACE_QUERY: ULONG = 0x00800000;

const FILE_VALID_OPTION_FLAGS: ULONG = 0x00FFFFFF;
const FILE_VALID_PIPE_OPTION_FLAGS: ULONG = 0x00000032;
const FILE_VALID_MAILSLOT_OPTION_FLAGS: ULONG = 0x00000032;

const FILE_ATTRIBUTE_VALID_FLAGS: ULONG = 0x00007FB7;
const FILE_SHARE_VALID_FLAGS: ULONG = 0x00000007;

const FILE_PIPE_MESSAGE_TYPE: ULONG = 0x00000001;
const FILE_PIPE_MESSAGE_MODE: ULONG = 0x00000001;
const FILE_PIPE_COMPLETE_OPERATION: ULONG = 0x00000001;

const IRP_CREATE_OPERATION: ULONG = 0x00000080;
const IRP_SYNCHRONOUS_API: ULONG = 0x04;
const IRP_DEFER_IO_COMPLETION: ULONG = 0x800;
const IRP_CLOSE_OPERATION: ULONG = 0x00000400;
const IRP_BUFFERED_IO: ULONG = 0x10;
const IRP_DEALLOCATE_BUFFER: ULONG = 0x20;

const FO_SYNCHRONOUS_IO: ULONG = 0x00000002;
const FO_ALERTABLE_IO: ULONG = 0x00000004;
const FO_NO_INTERMEDIATE_BUFFERING: ULONG = 0x00000008;
const FO_WRITE_THROUGH: ULONG = 0x00000010;
const FO_SEQUENTIAL_ONLY: ULONG = 0x00000020;
const FO_RANDOM_ACCESS: ULONG = 0x00100000;
const FO_FILE_OBJECT_HAS_EXTENSION: ULONG = 0x00800000;
const FO_HANDLE_CREATED: ULONG = 0x00040000;
const FO_DIRECT_DEVICE_OPEN: ULONG = 0x0800;
const FO_VOLUME_OPEN: ULONG = 0x00400000;
const FO_STREAM_FILE: ULONG = 0x0100;
const FO_OPENED_CASE_SENSITIVE: ULONG = 0x00020000;
const FO_REMOTE_ORIGIN: ULONG = 0x01000000;
const FO_FILE_OPEN_CANCELLED: ULONG = 0x00200000;

const DO_DEVICE_INITIALIZING: ULONG = 0x00000080;
const DO_EXCLUSIVE: ULONG = 0x00000008;
const DO_NEVER_LAST_DEVICE: ULONG = 0x0400;
const DO_BUS_ENUMERATED_DEVICE: ULONG = 0x00001000;

const DOE_UNLOAD_PENDING: ULONG = 0x01;
const DOE_DELETE_PENDING: ULONG = 0x02;
const DOE_REMOVE_PENDING: ULONG = 0x04;
const DOE_REMOVE_PROCESSED: ULONG = 0x08;

const SL_CASE_SENSITIVE: UCHAR = 0x80;

const IRP_MJ_CREATE: UCHAR = 0x00;
const IRP_MJ_CLOSE: UCHAR = 0x02;
const IRP_MJ_CLEANUP: UCHAR = 0x12;
const IRP_MJ_CREATE_NAMED_PIPE: UCHAR = 0x01;
const IRP_MJ_CREATE_MAILSLOT: UCHAR = 0x13;
const IRP_MJ_QUERY_SECURITY: UCHAR = 0x14;
const IRP_MJ_SET_SECURITY: UCHAR = 0x15;
const IRP_MJ_LOCK_CONTROL: UCHAR = 0x11;

const IRP_MN_UNLOCK_ALL: UCHAR = 0x03;

const IO_NO_PARAMETER_CHECKING: ULONG = 0x100;
const IO_CHECK_CREATE_PARAMETERS: ULONG = 0x200;
const IO_ATTACH_DEVICE: ULONG = 0x400;
const IO_ATTACH_DEVICE_API: ULONG = 0x80000000;
const IO_FORCE_ACCESS_CHECK: ULONG = 0x001;
const IO_IGNORE_SHARE_ACCESS_CHECK: ULONG = 0x800;

const FILE_DEVICE_DISK_FILE_SYSTEM: ULONG = 0x00000008;
const FILE_DEVICE_CD_ROM_FILE_SYSTEM: ULONG = 0x00000003;
const FILE_DEVICE_TAPE_FILE_SYSTEM: ULONG = 0x20;
const FILE_DEVICE_NETWORK_FILE_SYSTEM: ULONG = 0x00000014;
const FILE_DEVICE_DFS_FILE_SYSTEM: ULONG = 0x00000035;
const FILE_DEVICE_DISK: ULONG = 0x00000007;
const FILE_DEVICE_CD_ROM: ULONG = 0x00000002;
const FILE_DEVICE_TAPE: ULONG = 0x0000001F;
const FILE_DEVICE_VIRTUAL_DISK: ULONG = 0x00000024;
const FILE_DEVICE_FILE_SYSTEM: ULONG = 0x00000009;

const FILE_DEVICE_SECURE_OPEN: ULONG = 0x00000100;

const CreateFileTypeNone: CREATE_FILE_TYPE = 0;
const CreateFileTypeNamedPipe: CREATE_FILE_TYPE = 1;
const CreateFileTypeMailslot: CREATE_FILE_TYPE = 2;

const FileBasicInformation: FILE_INFORMATION_CLASS = 4;
const FileNetworkOpenInformation: FILE_INFORMATION_CLASS = 34;
const FileNameInformation: FILE_INFORMATION_CLASS = 9;

const QuerySecurityDescriptor: SECURITY_OPERATION_CODE = 1;
const SetSecurityDescriptor: SECURITY_OPERATION_CODE = 0x00000000;
const DeleteSecurityDescriptor: SECURITY_OPERATION_CODE = 0x00000002;
const AssignSecurityDescriptor: SECURITY_OPERATION_CODE = 0x00000003;

const PRIVILEGE_SET_ALL_NECESSARY: ULONG = 1;

const MUTANT_INCREMENT: LONG = 1;

const LockQueueIoDatabaseLock: ULONG = 0x0000000A;

const SynchronizationEvent: ULONG = 0x01;
const NotificationEvent: ULONG = 0x00;

const Executive: ULONG = 0;
const UserRequest: ULONG = 6;

const MAXUSHORT: USHORT = 0xFFFF;

const OBJ_NAME_PATH_SEPARATOR: u16 = b'\\' as u16;

#[inline]
unsafe fn InitializeObjectAttributes(
    p: POBJECT_ATTRIBUTES,
    n: *mut _UNICODE_STRING,
    a: ULONG,
    r: HANDLE,
    s: PSECURITY_DESCRIPTOR,
) {
    write_bytes(p, 0, 1);
    (*p).Length = size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    (*p).ObjectName = n as *mut _;
    (*p).Attributes = a;
    (*p).RootDirectory = r;
    (*p).SecurityDescriptor = s as *mut _;
}

unsafe fn IopCheckBackupRestorePrivilege(
    AccessState: PACCESS_STATE,
    CreateOptions: *mut ULONG,
    PreviousMode: KPROCESSOR_MODE,
    Disposition: ULONG,
) {
    if (*AccessState).Flags & SE_BACKUP_PRIVILEGES_CHECKED != 0 {
        return;
    }

    if *CreateOptions & FILE_OPEN_FOR_BACKUP_INTENT != 0 {
        (*AccessState).Flags |= SE_BACKUP_PRIVILEGES_CHECKED;

        let ReadAccess = READ_CONTROL
            | ACCESS_SYSTEM_SECURITY
            | FILE_GENERIC_READ
            | FILE_TRAVERSE;
        let WriteAccess = WRITE_DAC
            | WRITE_OWNER
            | ACCESS_SYSTEM_SECURITY
            | FILE_GENERIC_WRITE
            | FILE_ADD_FILE
            | FILE_ADD_SUBDIRECTORY
            | DELETE;
        let mut DesiredAccess = (*AccessState).RemainingDesiredAccess;

        if DesiredAccess & MAXIMUM_ALLOWED != 0 {
            DesiredAccess |= ReadAccess | WriteAccess;
        }

        let mut HaveBackupPriv = 0u8;
        let CheckRestore = if Disposition & FILE_OPEN != 0 {
            if ReadAccess & DesiredAccess != 0 {
                let mut Privileges: _PRIVILEGE_SET = zeroed();
                Privileges.PrivilegeCount = 1;
                Privileges.Control = PRIVILEGE_SET_ALL_NECESSARY;
                Privileges.Privilege[0].Luid = SeBackupPrivilege;
                Privileges.Privilege[0].Attributes = 0;

                let AccessGranted = SePrivilegeCheck(
                    &mut Privileges,
                    &mut (*AccessState).SubjectSecurityContext,
                    PreviousMode,
                );
                if AccessGranted != 0 {
                    HaveBackupPriv = 1;
                    SeAppendPrivileges(AccessState, &mut Privileges);
                    (*AccessState).PreviouslyGrantedAccess |= DesiredAccess & ReadAccess;
                    (*AccessState).RemainingDesiredAccess &= !ReadAccess;
                    DesiredAccess &= !ReadAccess;
                    (*AccessState).Flags |= TOKEN_HAS_BACKUP_PRIVILEGE;
                }
            }
            1u8  // CheckRestore = TRUE when Disposition & FILE_OPEN
        } else {
            0u8  // CheckRestore = FALSE otherwise
        };

        if (WriteAccess & DesiredAccess != 0) || (CheckRestore != 0) {
            let mut Privileges: _PRIVILEGE_SET = zeroed();
            Privileges.PrivilegeCount = 1;
            Privileges.Control = PRIVILEGE_SET_ALL_NECESSARY;
            Privileges.Privilege[0].Luid = SeRestorePrivilege;
            Privileges.Privilege[0].Attributes = 0;

            let AccessGranted = SePrivilegeCheck(
                &mut Privileges,
                &mut (*AccessState).SubjectSecurityContext,
                PreviousMode,
            );
            if AccessGranted != 0 {
                HaveBackupPriv = 1;
                SeAppendPrivileges(AccessState, &mut Privileges);
                (*AccessState).PreviouslyGrantedAccess |= DesiredAccess & WriteAccess;
                (*AccessState).RemainingDesiredAccess &= !WriteAccess;
                (*AccessState).Flags |= TOKEN_HAS_RESTORE_PRIVILEGE;
            }
        }

        if HaveBackupPriv == 0 {
            *CreateOptions &= !FILE_OPEN_FOR_BACKUP_INTENT;
        }
    }
}

unsafe fn IopCheckDeviceAndDriver(
    OpenPacket: POPEN_PACKET,
    DeviceObject: PDEVICE_OBJECT,
) -> NTSTATUS {
    let ext_flags = (*(IoGetDevObjExtension(DeviceObject) as *mut ntkrnl_sys::ddk::_EXTENDED_DEVOBJ_EXTENSION)).ExtensionFlags;
    if (ext_flags
        & (DOE_UNLOAD_PENDING | DOE_DELETE_PENDING | DOE_REMOVE_PENDING | DOE_REMOVE_PROCESSED)
        != 0)
        || ((*DeviceObject).Flags & DO_DEVICE_INITIALIZING != 0)
    {
        return STATUS_NO_SUCH_DEVICE;
    } else if ((*DeviceObject).Flags & DO_EXCLUSIVE != 0)
        && ((*DeviceObject).ReferenceCount != 0)
        && ((*OpenPacket).RelatedFileObject.is_null())
        && ((*OpenPacket).Options & IO_ATTACH_DEVICE == 0)
    {
        return STATUS_ACCESS_DENIED;
    }

    InterlockedIncrement(&mut (*DeviceObject).ReferenceCount);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn IopDoNameTransmogrify(
    Irp: PIRP,
    FileObject: PFILE_OBJECT,
    DataBuffer: PREPARSE_DATA_BUFFER,
) {
    let Irp = opaque_ptr(Irp);
    let FileObject = opaque_ptr(FileObject);
    if (*Irp).IoStatus.__bindgen_anon_1.Status != STATUS_REPARSE
        || (*Irp).IoStatus.Information as u32 != IO_REPARSE_TAG_MOUNT_POINT
        || (*Irp).Tail.Overlay.AuxiliaryBuffer.is_null()
        || (*DataBuffer).ReparseTag != IO_REPARSE_TAG_MOUNT_POINT
        || ((*DataBuffer).ReparseDataLength as ULONG) >= MAXIMUM_REPARSE_DATA_BUFFER_SIZE
        || ((*DataBuffer).Reserved as ULONG) >= MAXIMUM_REPARSE_DATA_BUFFER_SIZE
    {
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_IO_REPARSE_DATA_INVALID;
        ExFreePool(DataBuffer as PVOID);
        return;
    }

    if ((*DataBuffer).ReparseDataLength as ULONG) < REPARSE_DATA_BUFFER_HEADER_SIZE
        || ((*DataBuffer)
            .__bindgen_anon_1
            .SymbolicLinkReparseBuffer
            .PrintNameLength
            + (*DataBuffer)
                .__bindgen_anon_1
                .SymbolicLinkReparseBuffer
                .SubstituteNameLength
            + 16) as ULONG  // FIELD_OFFSET(REPARSE_DATA_BUFFER, MountPointReparseBuffer.PathBuffer[0])
            > MAXIMUM_REPARSE_DATA_BUFFER_SIZE
    {
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_IO_REPARSE_DATA_INVALID;
        ExFreePool(DataBuffer as PVOID);
        return;
    }

    let Buffer = ((*DataBuffer)
        .__bindgen_anon_1
        .MountPointReparseBuffer
        .PathBuffer
        .as_ptr() as ULONG_PTR
        + (*DataBuffer)
            .__bindgen_anon_1
            .MountPointReparseBuffer
            .SubstituteNameOffset as ULONG_PTR) as *mut u16;
    let Length = (*DataBuffer)
        .__bindgen_anon_1
        .MountPointReparseBuffer
        .SubstituteNameLength;

    if (MAXUSHORT as ULONG - (*DataBuffer).Reserved as ULONG) <= (Length as ULONG + 2)  {
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_IO_REPARSE_DATA_INVALID;
        ExFreePool(DataBuffer as PVOID);
        return;
    }

    let RequiredLength = (*DataBuffer).Reserved as USHORT + Length + 2;

    let NewBuffer = if (*FileObject).FileName.MaximumLength >= RequiredLength {
        (*FileObject).FileName.Buffer
    } else {
        let nb = ExAllocatePoolWithTag(PagedPool as u32, RequiredLength as usize, TAG_IO_NAME)
            as *mut u16;
        if nb.is_null() {
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_INSUFFICIENT_RESOURCES;
            ExFreePool(DataBuffer as PVOID);
            return;
        }
        nb
    };

    if (*DataBuffer).Reserved != 0 {
        copy(
            ((*FileObject).FileName.Buffer as ULONG_PTR
                + (*FileObject).FileName.Length as ULONG_PTR
                - (*DataBuffer).Reserved as ULONG_PTR) as *const u8,
            (NewBuffer as ULONG_PTR + Length as ULONG_PTR) as *mut u8,
            (*DataBuffer).Reserved as usize,
        );
    }

    if Length != 0 {
        copy_nonoverlapping(Buffer as *const u8, NewBuffer as *mut u8, Length as usize);
    }

    (*FileObject).FileName.Length = RequiredLength - 2;
    if NewBuffer != (*FileObject).FileName.Buffer {
        if !(*FileObject).FileName.Buffer.is_null() {
            ExFreePoolWithTag((*FileObject).FileName.Buffer as PVOID, 0);
        }
        (*FileObject).FileName.Buffer = NewBuffer;
        (*FileObject).FileName.MaximumLength = RequiredLength;
        *NewBuffer.add((RequiredLength / 2 - 1) as usize) = 0;
    }

    ExFreePool(DataBuffer as PVOID);
}

unsafe fn IopCheckTopDeviceHint(
    DeviceObject: *mut PDEVICE_OBJECT,
    OpenPacket: POPEN_PACKET,
    DirectOpen: BOOLEAN,
) -> NTSTATUS {
    let LocalDevice = *DeviceObject;

    if DirectOpen != 0 {
        return STATUS_INVALID_PARAMETER;
    }

    let DeviceType = (*LocalDevice).DeviceType;
    if DeviceType != FILE_DEVICE_DISK_FILE_SYSTEM
        && DeviceType != FILE_DEVICE_CD_ROM_FILE_SYSTEM
        && DeviceType != FILE_DEVICE_TAPE_FILE_SYSTEM
        && DeviceType != FILE_DEVICE_NETWORK_FILE_SYSTEM
        && DeviceType != FILE_DEVICE_DFS_FILE_SYSTEM
    {
        return STATUS_INVALID_PARAMETER;
    }

    if IopVerifyDeviceObjectOnStack(LocalDevice, (*OpenPacket).TopDeviceObjectHint) != 0 {
        *DeviceObject = (*OpenPacket).TopDeviceObjectHint;
        return STATUS_SUCCESS;
    }

    if (*OpenPacket).TraversedMountPoint != 0 {
        (*OpenPacket).TraversedMountPoint = 0;
        return STATUS_MOUNT_POINT_NOT_RESOLVED;
    }

    STATUS_INVALID_DEVICE_OBJECT_PARAMETER
}

#[no_mangle]
pub unsafe extern "system" fn IopParseDevice(
    ParseObject: PVOID,
    ObjectType: PVOID,
    AccessState: PACCESS_STATE,
    AccessMode: KPROCESSOR_MODE,
    Attributes: ULONG,
    CompleteName: *mut _UNICODE_STRING,
    RemainingName: *mut _UNICODE_STRING,
    Context: PVOID,
    SecurityQos: PSECURITY_QUALITY_OF_SERVICE,
    Object: *mut PVOID,
) -> NTSTATUS {
    let ParseObject = opaque_ptr(ParseObject);
    let ObjectType = opaque_ptr(ObjectType);
    let AccessState = opaque_ptr(AccessState);
    let CompleteName = opaque_ptr(CompleteName);
    let RemainingName = opaque_ptr(RemainingName);
    let Context = opaque_ptr(Context);
    let Object = opaque_ptr(Object);

    let OpenPacket = Context as POPEN_PACKET;
    let mut OriginalDeviceObject = ParseObject as PDEVICE_OBJECT;
    let mut DeviceObject: PDEVICE_OBJECT = null_mut();
    let mut Vpb: PVPB = null_mut();
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut FileObject: PFILE_OBJECT = null_mut();
    let mut Irp: PIRP;
    let mut StackLoc: PIO_STACK_LOCATION;
    let mut SecurityContext: _IO_SECURITY_CONTEXT = zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = zeroed();
    let mut DirectOpen = 0u8;
    let mut UseDummyFile: BOOLEAN = 0;
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = zeroed();
    let mut OldIrql: KIRQL = 0;
    let mut LocalFileObject: PDUMMY_FILE_OBJECT;
    let mut FileBasicInfo: PFILE_BASIC_INFORMATION;
    let mut ReturnLength: ULONG = 0;
    let mut CheckMode: KPROCESSOR_MODE;
    let mut VolumeOpen = 0u8;
    let mut DesiredAccess: ACCESS_MASK;
    let mut GrantedAccess: ACCESS_MASK = 0;
    let mut AccessGranted: BOOLEAN;
    let mut LockHeld = 0u8;
    let mut Privileges: PPRIVILEGE_SET = null_mut();
    let mut FileString: _UNICODE_STRING = zeroed();
    let mut Attempt: ULONG = 0;

    while Attempt < IOP_MAX_REPARSE_TRAVERSAL {
        *Object = null_mut();

        if IopValidateOpenPacket(OpenPacket) == 0 {
            return STATUS_OBJECT_TYPE_MISMATCH;
        }

        if (*OpenPacket).TraversedMountPoint != 0 {
            if (*OriginalDeviceObject).DeviceType != FILE_DEVICE_DISK
                && (*OriginalDeviceObject).DeviceType != FILE_DEVICE_CD_ROM
                && (*OriginalDeviceObject).DeviceType != FILE_DEVICE_VIRTUAL_DISK
                && (*OriginalDeviceObject).DeviceType != FILE_DEVICE_TAPE
            {
                (*OpenPacket).FinalStatus = STATUS_IO_REPARSE_DATA_INVALID;
                return STATUS_IO_REPARSE_DATA_INVALID;
            }
        }

        if !(*OpenPacket).RelatedFileObject.is_null() {
            OriginalDeviceObject = (*(*OpenPacket).RelatedFileObject).DeviceObject;
        }
        let CurrentDeviceObject = OriginalDeviceObject;

        Status = IopCheckDeviceAndDriver(OpenPacket, CurrentDeviceObject);
        if Status < 0 {
            (*OpenPacket).FinalStatus = Status;
            return Status;
        }

        RtlMapGenericMask(
            &mut (*AccessState).RemainingDesiredAccess,
            &(*IoFileObjectType).TypeInfo.GenericMapping as *const _ as PGENERIC_MAPPING,
        );
        RtlMapGenericMask(
            &mut (*AccessState).OriginalDesiredAccess,
            &(*IoFileObjectType).TypeInfo.GenericMapping as *const _ as PGENERIC_MAPPING,
        );
        SeSetAccessStateGenericMapping(
            AccessState,
            &(*IoFileObjectType).TypeInfo.GenericMapping as *const _ as PGENERIC_MAPPING,
        );
        DesiredAccess = (*AccessState).RemainingDesiredAccess;

        CheckMode = if (AccessMode != 0i8) || ((*OpenPacket).Options & IO_FORCE_ACCESS_CHECK != 0) {
            1i8
        } else {
            0i8
        };

        IopCheckBackupRestorePrivilege(
            AccessState,
            &mut (*OpenPacket).CreateOptions,
            CheckMode,
            (*OpenPacket).Disposition,
        );

        if (((*OpenPacket).Override != 0) && ((*RemainingName).Length == 0))
            || ((*AccessState).Flags & SE_BACKUP_PRIVILEGES_CHECKED != 0)
        {
            DesiredAccess |= (*AccessState).PreviouslyGrantedAccess;
        }

        if !(*OpenPacket).RelatedFileObject.is_null()
            && ((*(*OpenPacket).RelatedFileObject).Flags & FO_VOLUME_OPEN != 0)
            && ((*RemainingName).Length == 0)
        {
            VolumeOpen = 1;
        }

        if ((AccessMode != 0i8) || ((*OpenPacket).Options & IO_FORCE_ACCESS_CHECK != 0))
            && ((*OpenPacket).RelatedFileObject.is_null() || VolumeOpen != 0)
            && ((*OpenPacket).Override == 0)
        {
            KeEnterCriticalRegion();
            ExAcquireResourceSharedLite(&raw mut IopSecurityResource, 1);

            if (*RemainingName).Length == 0 {
                SeLockSubjectContext(&mut (*AccessState).SubjectSecurityContext);
                LockHeld = 1;

                AccessGranted = SeAccessCheck(
                    (*CurrentDeviceObject).SecurityDescriptor,
                    &mut (*AccessState).SubjectSecurityContext,
                    LockHeld,
                    DesiredAccess,
                    0,
                    &mut Privileges,
                    &(*IoFileObjectType).TypeInfo.GenericMapping as *const _ as PGENERIC_MAPPING,
                    1i8,
                    &mut GrantedAccess,
                    &mut Status,
                );
                if !Privileges.is_null() {
                    SeAppendPrivileges(AccessState, Privileges);
                    SeFreePrivileges(Privileges);
                }

                if AccessGranted != 0 {
                    (*AccessState).PreviouslyGrantedAccess |= GrantedAccess;
                    (*AccessState).RemainingDesiredAccess &= !(GrantedAccess | MAXIMUM_ALLOWED);
                    (*OpenPacket).Override = 1;
                }

                static FILE_STR: [u16; 5] = [
                    b'F' as u16,
                    b'i' as u16,
                    b'l' as u16,
                    b'e' as u16,
                    0,
                ];
                FileString.Length = 8;
                FileString.MaximumLength = 8;
                FileString.Buffer = FILE_STR.as_ptr() as *mut u16;

                SeOpenObjectAuditAlarm(
                    &mut FileString,
                    CurrentDeviceObject as PVOID,
                    CompleteName,
                    (*CurrentDeviceObject).SecurityDescriptor,
                    AccessState,
                    0u8,
                    AccessGranted,
                    1i8,
                    &mut (*AccessState).GenerateOnClose,
                );
            } else {
                if ((*AccessState).Flags & TOKEN_HAS_TRAVERSE_PRIVILEGE == 0)
                    || (((*CurrentDeviceObject).DeviceType == FILE_DEVICE_DISK)
                        || ((*CurrentDeviceObject).DeviceType == FILE_DEVICE_CD_ROM))
                {
                    AccessGranted = if (*AccessState).Flags & TOKEN_IS_RESTRICTED == 0 {
                        SeFastTraverseCheck(
                            (*CurrentDeviceObject).SecurityDescriptor,
                            AccessState,
                            FILE_TRAVERSE,
                            1i8,
                        )
                    } else {
                        0u8
                    };

                    if AccessGranted == 0 {
                        SeLockSubjectContext(&mut (*AccessState).SubjectSecurityContext);
                        LockHeld = 1;

                        AccessGranted = SeAccessCheck(
                            (*CurrentDeviceObject).SecurityDescriptor,
                            &mut (*AccessState).SubjectSecurityContext,
                            LockHeld,
                            FILE_TRAVERSE,
                            0,
                            &mut Privileges,
                            &(*IoFileObjectType).TypeInfo.GenericMapping as *const _
                                as PGENERIC_MAPPING,
                            1i8,
                            &mut GrantedAccess,
                            &mut Status,
                        );
                        if !Privileges.is_null() {
                            SeAppendPrivileges(AccessState, Privileges);
                            SeFreePrivileges(Privileges);
                        }
                    }
                } else {
                    AccessGranted = 1;
                }
            }

            ExReleaseResourceLite(&raw mut IopSecurityResource);
            KeLeaveCriticalRegion();

            if LockHeld != 0 {
                SeUnlockSubjectContext(&mut (*AccessState).SubjectSecurityContext);
            }

            if AccessGranted == 0 {
                IopDereferenceDeviceObject(CurrentDeviceObject, 0u8);
                return STATUS_ACCESS_DENIED;
            }
        }

        UseDummyFile = if ((*OpenPacket).QueryOnly != 0) || ((*OpenPacket).DeleteOnly != 0) {
            1
        } else {
            0
        };

        if ((*RemainingName).Length == 0)
            && (*OpenPacket).RelatedFileObject.is_null()
            && ((DesiredAccess
                & !(SYNCHRONIZE
                    | FILE_READ_ATTRIBUTES
                    | READ_CONTROL
                    | ACCESS_SYSTEM_SECURITY
                    | WRITE_OWNER
                    | WRITE_DAC))
                == 0)
            && (UseDummyFile == 0)
        {
            DirectOpen = 1;
        }

        if !(*OpenPacket).RelatedFileObject.is_null()
            && ((*(*OpenPacket).RelatedFileObject).Flags & FO_DIRECT_DEVICE_OPEN == 0)
        {
            DeviceObject = ParseObject as PDEVICE_OBJECT;
            if !(*(*OpenPacket).RelatedFileObject).Vpb.is_null() {
                Vpb = (*(*OpenPacket).RelatedFileObject).Vpb;
                InterlockedIncrement(&raw mut (*Vpb).ReferenceCount as *mut LONG);
                if (*OpenPacket).InternalFlags & IOP_USE_TOP_LEVEL_DEVICE_HINT != 0 {
                    DeviceObject = (*Vpb).DeviceObject;
                }
            }
        } else {
            if !(*OriginalDeviceObject).Vpb.is_null() && DirectOpen == 0 {
                Vpb = IopCheckVpbMounted(
                    OpenPacket,
                    OriginalDeviceObject,
                    RemainingName,
                    &mut Status,
                );
                if Vpb.is_null() {
                    return Status;
                }
                DeviceObject = (*Vpb).DeviceObject;
            } else {
                DeviceObject = OriginalDeviceObject;
            }

            if ((*OpenPacket).InternalFlags & IOP_USE_TOP_LEVEL_DEVICE_HINT == 0)
                && !(*DeviceObject).AttachedDevice.is_null()
            {
                DeviceObject = IoGetAttachedDevice(DeviceObject);
            }
        }

        if (*OpenPacket).InternalFlags & IOP_USE_TOP_LEVEL_DEVICE_HINT != 0 {
            Status = IopCheckTopDeviceHint(&mut DeviceObject, OpenPacket, DirectOpen);
            if Status < 0 {
                IopDereferenceDeviceObject(CurrentDeviceObject, 0u8);
                if !Vpb.is_null() {
                    IopDereferenceVpbAndFree(Vpb);
                }
                return Status;
            }
        }

        if (*OpenPacket).TraversedMountPoint != 0 {
            (*OpenPacket).TraversedMountPoint = 0;
        }

        if ((*DeviceObject).Characteristics & FILE_DEVICE_SECURE_OPEN != 0)
            && ((!(*OpenPacket).RelatedFileObject.is_null() || (*RemainingName).Length != 0))
            && (VolumeOpen == 0)
        {
            Privileges = null_mut();
            GrantedAccess = 0;

            KeEnterCriticalRegion();
            ExAcquireResourceSharedLite(&raw mut IopSecurityResource, 1);

            SeLockSubjectContext(&mut (*AccessState).SubjectSecurityContext);

            AccessGranted = SeAccessCheck(
                (*CurrentDeviceObject).SecurityDescriptor,
                &mut (*AccessState).SubjectSecurityContext,
                1,
                DesiredAccess,
                0,
                &mut Privileges,
                &(*IoFileObjectType).TypeInfo.GenericMapping as *const _ as PGENERIC_MAPPING,
                1i8,
                &mut GrantedAccess,
                &mut Status,
            );
            if !Privileges.is_null() {
                SeAppendPrivileges(AccessState, Privileges);
                SeFreePrivileges(Privileges);
            }

            if GrantedAccess != 0 {
                (*AccessState).PreviouslyGrantedAccess |= GrantedAccess;
                (*AccessState).RemainingDesiredAccess &= !(GrantedAccess | MAXIMUM_ALLOWED);
            }

            static FILE_STR2: [u16; 5] = [
                b'F' as u16,
                b'i' as u16,
                b'l' as u16,
                b'e' as u16,
                0,
            ];
            FileString.Length = 8;
            FileString.MaximumLength = 8;
            FileString.Buffer = FILE_STR2.as_ptr() as *mut u16;

            SeOpenObjectAuditAlarm(
                &mut FileString,
                DeviceObject as PVOID,
                CompleteName,
                (*CurrentDeviceObject).SecurityDescriptor,
                AccessState,
                0u8,
                AccessGranted,
                1i8,
                &mut (*AccessState).GenerateOnClose,
            );

            SeUnlockSubjectContext(&mut (*AccessState).SubjectSecurityContext);

            ExReleaseResourceLite(&raw mut IopSecurityResource);
            KeLeaveCriticalRegion();

            if AccessGranted == 0 {
                IopDereferenceDeviceObject(CurrentDeviceObject, 0u8);
                if !Vpb.is_null() {
                    IopDereferenceVpbAndFree(Vpb);
                }
                return STATUS_ACCESS_DENIED;
            }
        }

        Irp = IoAllocateIrp((*DeviceObject).StackSize as i8, 1);
        if Irp.is_null() {
            IopDereferenceDeviceObject(CurrentDeviceObject, 0u8);
            if !Vpb.is_null() {
                IopDereferenceVpbAndFree(Vpb);
            }
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        (*Irp).RequestorMode = AccessMode;
        (*Irp).Flags = IRP_CREATE_OPERATION | IRP_SYNCHRONOUS_API | IRP_DEFER_IO_COMPLETION;
        (*Irp).Tail.Overlay.Thread = PsGetCurrentThread() as PETHREAD;
        (*Irp).UserIosb = core::ptr::addr_of_mut!(IoStatusBlock) as PIO_STATUS_BLOCK;
        (*Irp).MdlAddress = null_mut();
        (*Irp).PendingReturned = 0;
        (*Irp).UserEvent = null_mut();
        (*Irp).Cancel = 0;
        (*Irp).CancelRoutine = None;
        (*Irp).Tail.Overlay.AuxiliaryBuffer = null_mut();

        SecurityContext.SecurityQos = SecurityQos;
        SecurityContext.AccessState = AccessState;
        SecurityContext.DesiredAccess = (*AccessState).RemainingDesiredAccess;
        SecurityContext.FullCreateOptions = (*OpenPacket).CreateOptions;

        StackLoc = IoGetNextIrpStackLocation(Irp);
        (*StackLoc).Control = 0;

        match (*OpenPacket).CreateFileType {
            CreateFileTypeNone => {
                (*StackLoc).MajorFunction = IRP_MJ_CREATE as u8;
                (*StackLoc).Parameters.Create.EaLength = (*OpenPacket).EaLength;
                (*StackLoc).Flags = (*OpenPacket).Options as u8;
                (*StackLoc).Flags |= if Attributes & OBJ_CASE_INSENSITIVE == 0 {
                    SL_CASE_SENSITIVE
                } else {
                    0
                };
            }
            CreateFileTypeNamedPipe => {
                (*StackLoc).MajorFunction = IRP_MJ_CREATE_NAMED_PIPE as u8;
                (*StackLoc).Parameters.CreatePipe.Parameters = (*OpenPacket).ExtraCreateParameters as PNAMED_PIPE_CREATE_PARAMETERS;
            }
            CreateFileTypeMailslot => {
                (*StackLoc).MajorFunction = IRP_MJ_CREATE_MAILSLOT as u8;
                (*StackLoc).Parameters.CreateMailslot.Parameters =
                    (*OpenPacket).ExtraCreateParameters as PMAILSLOT_CREATE_PARAMETERS;
            }
            _ => {}
        }

        (*Irp).Overlay.AllocationSize = (*OpenPacket).AllocationSize;
        (*Irp).AssociatedIrp.SystemBuffer = (*OpenPacket).EaBuffer;
        (*StackLoc).Parameters.Create.Options =
            ((*OpenPacket).Disposition << 24) | ((*OpenPacket).CreateOptions & 0xFFFFFF);
        (*StackLoc).Parameters.Create.FileAttributes = (*OpenPacket).FileAttributes as u16;
        (*StackLoc).Parameters.Create.ShareAccess = (*OpenPacket).ShareAccess as u16;
        (*StackLoc).Parameters.Create.SecurityContext = core::ptr::addr_of_mut!(SecurityContext);

        if UseDummyFile == 0 {
            let mut ObjectSize = size_of::<_FILE_OBJECT>();
            if (*OpenPacket).InternalFlags & IOP_CREATE_FILE_OBJECT_EXTENSION != 0 {
                ObjectSize += size_of::<_FILE_OBJECT_EXTENSION>();
            }

            InitializeObjectAttributes(&mut ObjectAttributes, null_mut(), Attributes, 0 as _, null_mut());
            Status = ObCreateObject(
                0i8,
                IoFileObjectType,
                &mut ObjectAttributes,
                AccessMode,
                null_mut(),
                ObjectSize as u32,
                0,
                0,
                &mut FileObject as *mut _ as *mut PVOID,
            );
            if Status < 0 {
                IoFreeIrp(Irp);
                IopDereferenceDeviceObject(CurrentDeviceObject, 0u8);
                if !Vpb.is_null() {
                    IopDereferenceVpbAndFree(Vpb);
                }
                (*OpenPacket).FinalStatus = Status;
                return Status;
            }

            write_bytes(FileObject as *mut u8, 0, ObjectSize);

            if (*OpenPacket).CreateOptions
                & (FILE_SYNCHRONOUS_IO_ALERT | FILE_SYNCHRONOUS_IO_NONALERT)
                != 0
            {
                (*FileObject).Flags |= FO_SYNCHRONOUS_IO;
                if (*OpenPacket).CreateOptions & FILE_SYNCHRONOUS_IO_ALERT != 0 {
                    (*FileObject).Flags |= FO_ALERTABLE_IO;
                }
            }

            if (*FileObject).Flags & FO_SYNCHRONOUS_IO != 0 {
                KeInitializeEvent(
                    &mut (*FileObject).Lock,
                    1,  // SynchronizationEvent
                    0,
                );
            }

            if (*OpenPacket).CreateOptions & FILE_NO_INTERMEDIATE_BUFFERING != 0 {
                (*FileObject).Flags |= FO_NO_INTERMEDIATE_BUFFERING;
            }
            if (*OpenPacket).CreateOptions & FILE_WRITE_THROUGH != 0 {
                (*FileObject).Flags |= FO_WRITE_THROUGH;
            }
            if (*OpenPacket).CreateOptions & FILE_SEQUENTIAL_ONLY != 0 {
                (*FileObject).Flags |= FO_SEQUENTIAL_ONLY;
            }
            if (*OpenPacket).CreateOptions & FILE_RANDOM_ACCESS != 0 {
                (*FileObject).Flags |= FO_RANDOM_ACCESS;
            }

            if (*OpenPacket).InternalFlags & IOP_CREATE_FILE_OBJECT_EXTENSION != 0 {
                (*FileObject).Flags |= FO_FILE_OBJECT_HAS_EXTENSION;
                let FileObjectExtension =
                    (FileObject as ULONG_PTR + size_of::<_FILE_OBJECT>()) as PFILE_OBJECT_EXTENSION;
                (*FileObject).FileObjectExtension = FileObjectExtension as *mut c_void;

                if (*OpenPacket).InternalFlags & IOP_USE_TOP_LEVEL_DEVICE_HINT != 0 {
                    (*FileObjectExtension).TopDeviceObjectHint = DeviceObject;
                }
            }
        } else {
            LocalFileObject = (*OpenPacket).LocalFileObject;
            write_bytes(LocalFileObject as *mut u8, 0, size_of::<_DUMMY_FILE_OBJECT>());
            FileObject = &mut (*LocalFileObject).ObjectHeader.Body as *mut _ as PFILE_OBJECT;
            (*LocalFileObject).ObjectHeader.Type = IoFileObjectType;
            (*LocalFileObject).ObjectHeader.PointerCount = 1;
        }

        (*FileObject).Type = IO_TYPE_FILE as i16;
        (*FileObject).Size = size_of::<_FILE_OBJECT>() as i16;
        (*FileObject).RelatedFileObject = (*OpenPacket).RelatedFileObject;
        (*FileObject).DeviceObject = OriginalDeviceObject;

        if DirectOpen != 0 {
            (*FileObject).Flags |= FO_DIRECT_DEVICE_OPEN;
        }
        if Attributes & OBJ_CASE_INSENSITIVE == 0 {
            (*FileObject).Flags |= FO_OPENED_CASE_SENSITIVE;
        }

        (*Irp).Tail.Overlay.OriginalFileObject = FileObject;
        (*StackLoc).FileObject = FileObject;

        if (*RemainingName).Length != 0 {
            (*FileObject).FileName.MaximumLength = (*RemainingName).Length + 2;
            (*FileObject).FileName.Buffer = ExAllocatePoolWithTag(
                PagedPool as u32,
                (*FileObject).FileName.MaximumLength as usize,
                TAG_IO_NAME,
            ) as *mut u16;
            if (*FileObject).FileName.Buffer.is_null() {
                IoFreeIrp(Irp);
                IopDereferenceDeviceObject(CurrentDeviceObject, 0u8);
                if !Vpb.is_null() {
                    IopDereferenceVpbAndFree(Vpb);
                }
                (*FileObject).DeviceObject = null_mut();
                if UseDummyFile == 0 {
                    ObDereferenceObject(FileObject as PVOID);
                }
                return STATUS_INSUFFICIENT_RESOURCES;
            }
        }

        RtlCopyUnicodeString(&mut (*FileObject).FileName, RemainingName);

        KeInitializeEvent(&mut (*FileObject).Event, 1, 0);
        (*OpenPacket).FileObject = FileObject;

        IopQueueIrpToThread(Irp);
        Status = core::hint::black_box(IoCallDriver(DeviceObject, Irp));
        if Status == STATUS_PENDING {
            KeWaitForSingleObject(
                core::ptr::addr_of_mut!((*FileObject).Event) as PVOID,
                0,
                0i8,
                0u8,
                null_mut(),
            );
            Status = core::ptr::read_volatile(&IoStatusBlock.__bindgen_anon_1.Status);
        } else {
            if Status == STATUS_REPARSE {
                if (*Irp).IoStatus.Information as u32 == IO_REPARSE_TAG_MOUNT_POINT {
                    let ReparseData = (*Irp).Tail.Overlay.AuxiliaryBuffer as PREPARSE_DATA_BUFFER;
                    IopDoNameTransmogrify(Irp, FileObject, ReparseData);
                }
            }

            // [guard] KeRaiseIrql(2, &mut OldIrql);
            let irql_guard = unsafe { ntkrnl_sys::guards::IrqlGuard::raise(2) };
            IoStatusBlock = core::ptr::read_volatile(&(*Irp).IoStatus);
            Status = IoStatusBlock.__bindgen_anon_1.Status;
            core::ptr::write_volatile(&mut (*FileObject).Event.Header.SignalState, 1);
            IopUnQueueIrpFromThread(Irp);

            if ((*Irp).Flags & IRP_BUFFERED_IO != 0) && ((*Irp).Flags & IRP_DEALLOCATE_BUFFER != 0)
            {
                ExFreePool((*Irp).AssociatedIrp.SystemBuffer);
            }

            IoFreeIrp(Irp);
            drop(irql_guard);
        }

        (*OpenPacket).Information = IoStatusBlock.Information;

        if Status < 0 {
            if (*FileObject).FileName.Length != 0 {
                ExFreePoolWithTag((*FileObject).FileName.Buffer as PVOID, 0);
                (*FileObject).FileName.Buffer = null_mut();
                (*FileObject).FileName.Length = 0;
            }
            (*FileObject).DeviceObject = null_mut();
            let OpenCancelled = (*FileObject).Flags & FO_FILE_OPEN_CANCELLED != 0;
            (*OpenPacket).FileObject = null_mut();
            if UseDummyFile == 0 {
                ObDereferenceObject(FileObject as PVOID);
            }
            IopDereferenceDeviceObject(CurrentDeviceObject, 0u8);
            if !OpenCancelled && !Vpb.is_null() {
                IopDereferenceVpbAndFree(Vpb);
            }
            (*OpenPacket).FinalStatus = Status;
            return Status;
        } else if Status == STATUS_REPARSE {
            if (*OpenPacket).Information as u32 == IO_REPARSE
                || (*OpenPacket).Information as u32 == IO_REPARSE_TAG_MOUNT_POINT
            {
                if (*CompleteName).MaximumLength < (*FileObject).FileName.Length {
                    let NewCompleteName = ExAllocatePoolWithTag(
                        PagedPool as u32,
                        (*FileObject).FileName.Length as usize,
                        TAG_IO_NAME,
                    ) as *mut u16;
                    if NewCompleteName.is_null() {
                        (*OpenPacket).FinalStatus = STATUS_INSUFFICIENT_RESOURCES;
                        return STATUS_INSUFFICIENT_RESOURCES;
                    }
                    if !(*CompleteName).Buffer.is_null() {
                        ExFreePoolWithTag((*CompleteName).Buffer as PVOID, 0);
                    }
                    (*CompleteName).Buffer = NewCompleteName;
                    (*CompleteName).MaximumLength = (*FileObject).FileName.Length;
                }
                RtlCopyUnicodeString(CompleteName, &(*FileObject).FileName);

                if (*OpenPacket).Information as u32 == IO_REPARSE_TAG_MOUNT_POINT {
                    (*OpenPacket).RelatedFileObject = null_mut();
                }
            }

            if (*FileObject).FileName.Length != 0 {
                ExFreePoolWithTag((*FileObject).FileName.Buffer as PVOID, 0);
                (*FileObject).FileName.Buffer = null_mut();
                (*FileObject).FileName.Length = 0;
            }
            (*FileObject).DeviceObject = null_mut();
            (*OpenPacket).FileObject = null_mut();
            if UseDummyFile == 0 {
                ObDereferenceObject(FileObject as PVOID);
            }
            IopDereferenceDeviceObject(CurrentDeviceObject, 0u8);
            if !Vpb.is_null() {
                IopDereferenceVpbAndFree(Vpb);
            }

            if (*OpenPacket).Information as u32 != IO_REMOUNT {
                (*OpenPacket).RelatedFileObject = null_mut();
                if (*OpenPacket).Information as u32 == IO_REPARSE_TAG_MOUNT_POINT {
                    (*OpenPacket).TraversedMountPoint = 1;
                }
                if (*OpenPacket).Override != 0 {
                    KeBugCheckEx(
                        ntkrnl_sys::bugcodes::DRIVER_RETURNED_STATUS_REPARSE_FOR_VOLUME_OPEN,
                        CurrentDeviceObject as ULONG_PTR,
                        DeviceObject as ULONG_PTR,
                        CompleteName as ULONG_PTR,
                        (*OpenPacket).Information as ULONG_PTR,
                    );
                }
                return STATUS_REPARSE;
            }
            Attempt += 1;
            continue;
        }

        break;
    }

    if Attempt == IOP_MAX_REPARSE_TRAVERSAL {
        return STATUS_UNSUCCESSFUL;
    }

    let OwnerDevice = IoGetRelatedDeviceObject(FileObject);
    if OwnerDevice != DeviceObject {
        if !Vpb.is_null() {
            IopDereferenceVpbAndFree(Vpb);
        }
        Vpb = (*FileObject).Vpb;
        if !Vpb.is_null() {
            InterlockedIncrement(&raw mut (*Vpb).ReferenceCount as *mut LONG);
        }
    }

    if UseDummyFile == 0 {
        if ((*FileObject).RelatedFileObject.is_null()
            || ((*(*FileObject).RelatedFileObject).Flags & FO_VOLUME_OPEN != 0))
            && ((*FileObject).FileName.Length == 0)
        {
            let dt = (*OwnerDevice).DeviceType;
            if dt == FILE_DEVICE_DISK_FILE_SYSTEM
                || dt == FILE_DEVICE_CD_ROM_FILE_SYSTEM
                || dt == FILE_DEVICE_TAPE_FILE_SYSTEM
                || dt == FILE_DEVICE_FILE_SYSTEM
            {
                (*FileObject).Flags |= FO_VOLUME_OPEN;
            }
        }

        ObReferenceObject(FileObject as PVOID);
        core::ptr::write_volatile(Object, FileObject as PVOID);
        core::ptr::write_volatile(&mut (*OpenPacket).FinalStatus, IoStatusBlock.__bindgen_anon_1.Status);
        core::ptr::write_volatile(&mut (*OpenPacket).ParseCheck, 1);
        return (*OpenPacket).FinalStatus;
    } else {
        if (*OpenPacket).QueryOnly != 0 {
            if (*OpenPacket).FullAttributes == 0 {
                FileBasicInfo = ExAllocatePoolWithTag(
                    NonPagedPool as u32,
                    size_of::<_FILE_BASIC_INFORMATION>(),
                    TAG_IO,
                ) as PFILE_BASIC_INFORMATION;
                if !FileBasicInfo.is_null() {
                    Status = IoQueryFileInformation(
                        FileObject,
                        FileBasicInformation,
                        size_of::<_FILE_BASIC_INFORMATION>() as u32,
                        FileBasicInfo as PVOID,
                        &mut ReturnLength,
                    );
                    if Status >= 0 {
                        copy_nonoverlapping(
                            FileBasicInfo as *const u8,
                            (*OpenPacket).BasicInformation as *mut u8,
                            ReturnLength as usize,
                        );
                    }
                    ExFreePoolWithTag(FileBasicInfo as PVOID, TAG_IO);
                } else {
                    Status = STATUS_INSUFFICIENT_RESOURCES;
                }
            } else {
                Status = IoQueryFileInformation(
                    FileObject,
                    FileNetworkOpenInformation,
                    size_of::<_FILE_NETWORK_OPEN_INFORMATION>() as u32,
                    (*OpenPacket).NetworkInformation as PVOID,
                    &mut ReturnLength,
                );
            }
        }

        IopDeleteFile(FileObject as PVOID);
        (*OpenPacket).FileObject = null_mut();
        (*OpenPacket).FinalStatus = Status;
        (*OpenPacket).ParseCheck = 1;
        return Status;
    }
}

#[no_mangle]
pub unsafe extern "system" fn IopParseFile(
    ParseObject: PVOID,
    ObjectType: PVOID,
    AccessState: PACCESS_STATE,
    AccessMode: KPROCESSOR_MODE,
    Attributes: ULONG,
    CompleteName: *mut _UNICODE_STRING,
    RemainingName: *mut _UNICODE_STRING,
    Context: PVOID,
    SecurityQos: PSECURITY_QUALITY_OF_SERVICE,
    Object: *mut PVOID,
) -> NTSTATUS {
    let ParseObject = opaque_ptr(ParseObject);
    let ObjectType = opaque_ptr(ObjectType);
    let AccessState = opaque_ptr(AccessState);
    let CompleteName = opaque_ptr(CompleteName);
    let RemainingName = opaque_ptr(RemainingName);
    let Context = opaque_ptr(Context);
    let Object = opaque_ptr(Object);
    let OpenPacket = Context as POPEN_PACKET;
    if IopValidateOpenPacket(OpenPacket) == 0 {
        return STATUS_OBJECT_TYPE_MISMATCH;
    }
    let DeviceObject = IoGetRelatedDeviceObject(ParseObject as PFILE_OBJECT);
    (*OpenPacket).RelatedFileObject = ParseObject as PFILE_OBJECT;
    IopParseDevice(
        DeviceObject as PVOID,
        ObjectType,
        AccessState,
        AccessMode,
        Attributes,
        CompleteName,
        RemainingName,
        OpenPacket as PVOID,
        SecurityQos,
        Object,
    )
}

#[no_mangle]
pub unsafe extern "system" fn IopDeleteFile(ObjectBody: PVOID) {
    let ObjectBody = opaque_ptr(ObjectBody);

    let FileObject = ObjectBody as PFILE_OBJECT;
    if (*FileObject).DeviceObject.is_null() {
        return;
    }

    let DeviceObject = if (*FileObject).Flags & FO_DIRECT_DEVICE_OPEN != 0 {
        IoGetAttachedDevice((*FileObject).DeviceObject)
    } else {
        IoGetRelatedDeviceObject(FileObject)
    };

    if (*FileObject).Flags & FO_HANDLE_CREATED == 0  {
        IopCloseFile(null_mut(), ObjectBody, 0, 1, 1);
    }

    KeClearEvent(&mut (*FileObject).Event);
    let mut Event: _KEVENT = zeroed();
    KeInitializeEvent(&mut Event, 1, 0);  // SynchronizationEvent

    let Irp = IopAllocateIrpMustSucceed((*DeviceObject).StackSize as i8);
    (*Irp).UserEvent = core::ptr::addr_of_mut!(Event) as *mut _KEVENT;
    (*Irp).UserIosb = addr_of_mut!((*Irp).IoStatus) as PIO_STATUS_BLOCK;
    (*Irp).Tail.Overlay.Thread = PsGetCurrentThread() as PETHREAD;
    (*Irp).Tail.Overlay.OriginalFileObject = FileObject;
    (*Irp).Flags = IRP_CLOSE_OPERATION | IRP_SYNCHRONOUS_API;

    let StackPtr = IoGetNextIrpStackLocation(Irp);
    (*StackPtr).MajorFunction = IRP_MJ_CLOSE as u8;
    (*StackPtr).FileObject = FileObject;

    IopQueueIrpToThread(Irp);

    let Vpb = (*FileObject).Vpb;
    if !Vpb.is_null() && ((*FileObject).Flags & FO_DIRECT_DEVICE_OPEN == 0) {
        InterlockedDecrement(&raw mut (*Vpb).ReferenceCount as *mut LONG);
    }

    let DereferenceDone = if (*(*FileObject).DeviceObject).Flags & DO_NEVER_LAST_DEVICE != 0 {
        InterlockedDecrement(&raw mut (*(*FileObject).DeviceObject).ReferenceCount as *mut LONG);
        1u8
    } else {
        0u8
    };

    let Status = core::hint::black_box(IoCallDriver(DeviceObject, Irp));
    if Status == STATUS_PENDING {
        KeWaitForSingleObject(core::ptr::addr_of_mut!(Event) as PVOID, 0, 0i8, 0, null_mut());
    }

    let mut OldIrql: KIRQL = 0;
    // [guard] KeRaiseIrql(2, &mut OldIrql);
    let irql_guard = unsafe { ntkrnl_sys::guards::IrqlGuard::raise(2) };
    IopUnQueueIrpFromThread(Irp);
    drop(irql_guard);

    IoFreeIrp(Irp);

    if !(*FileObject).FileName.Buffer.is_null() {
        ExFreePoolWithTag((*FileObject).FileName.Buffer as PVOID, 0);
        (*FileObject).FileName.Buffer = null_mut();
    }

    if !(*FileObject).CompletionContext.is_null() {
        ObDereferenceObject((*(*FileObject).CompletionContext).Port);
        ExFreePool((*FileObject).CompletionContext as PVOID);
    }

    if (*FileObject).Flags & FO_FILE_OBJECT_HAS_EXTENSION != 0 {
        FsRtlPTeardownPerFileObjectContexts(FileObject);
    }

    if DereferenceDone == 0 {
        IopDereferenceDeviceObject((*FileObject).DeviceObject, 0u8);
    }
}

#[no_mangle]
pub unsafe extern "system" fn IopGetDeviceAttachmentBase(DeviceObject: PDEVICE_OBJECT) -> PDEVICE_OBJECT {
    let DeviceObject = opaque_ptr(DeviceObject);
    let mut PDO = DeviceObject;
    while !(*((*PDO).DeviceObjectExtension as *mut ntkrnl_sys::ddk::_EXTENDED_DEVOBJ_EXTENSION))
        .AttachedTo
        .is_null()
    {
        PDO = (*((*PDO).DeviceObjectExtension as *mut ntkrnl_sys::ddk::_EXTENDED_DEVOBJ_EXTENSION)).AttachedTo;
    }
    PDO
}

unsafe fn IopGetDevicePDO(DeviceObject: PDEVICE_OBJECT) -> PDEVICE_OBJECT {
    let OldIrql = KeAcquireQueuedSpinLock(LockQueueIoDatabaseLock as _);
    let mut PDO = IopGetDeviceAttachmentBase(DeviceObject);
    if (*PDO).Flags & DO_BUS_ENUMERATED_DEVICE != DO_BUS_ENUMERATED_DEVICE {
        PDO = null_mut();
    } else {
        ObReferenceObject(PDO as PVOID);
    }
    KeReleaseQueuedSpinLock(LockQueueIoDatabaseLock as _, OldIrql);
    PDO
}

unsafe fn IopSetDeviceSecurityDescriptor(
    DeviceObject: PDEVICE_OBJECT,
    SecurityInformation: *mut u32,
    SecurityDescriptor: PSECURITY_DESCRIPTOR,
    PoolType: u32,
    GenericMapping: PGENERIC_MAPPING,
) -> NTSTATUS {
    loop {
        KeEnterCriticalRegion();
        ExAcquireResourceSharedLite(&raw mut IopSecurityResource, 1);

        let OldSecurityDescriptor = (*DeviceObject).SecurityDescriptor;
        if !OldSecurityDescriptor.is_null() {
            ObReferenceSecurityDescriptor(OldSecurityDescriptor, 1);
        }

        ExReleaseResourceLite(&raw mut IopSecurityResource);
        KeLeaveCriticalRegion();

        let mut NewSecurityDescriptor = OldSecurityDescriptor;
        let Status = SeSetSecurityDescriptorInfo(
            null_mut(),
            SecurityInformation,
            SecurityDescriptor,
            &mut NewSecurityDescriptor,
            PoolType,
            GenericMapping,
        );

        if Status < 0 {
            if !OldSecurityDescriptor.is_null() {
                ObDereferenceSecurityDescriptor(OldSecurityDescriptor, 1);
            }
            return Status;
        }

        let mut CachedSecurityDescriptor: PSECURITY_DESCRIPTOR = null_mut();
        let Status = ObLogSecurityDescriptor(NewSecurityDescriptor, &mut CachedSecurityDescriptor, 1);
        ExFreePool(NewSecurityDescriptor as PVOID);
        if Status < 0 {
            ObDereferenceSecurityDescriptor(OldSecurityDescriptor, 1);
            return Status;
        }

        KeEnterCriticalRegion();
        ExAcquireResourceExclusiveLite(&raw mut IopSecurityResource, 1);
        if (*DeviceObject).SecurityDescriptor == OldSecurityDescriptor {
            (*DeviceObject).SecurityDescriptor = CachedSecurityDescriptor;
            ExReleaseResourceLite(&raw mut IopSecurityResource);
            KeLeaveCriticalRegion();
            ObDereferenceSecurityDescriptor(OldSecurityDescriptor, 2);
            return Status;
        }
        ExReleaseResourceLite(&raw mut IopSecurityResource);
        KeLeaveCriticalRegion();

        ObDereferenceSecurityDescriptor(OldSecurityDescriptor, 1);
        ObDereferenceSecurityDescriptor(CachedSecurityDescriptor, 1);
    }
}

unsafe fn IopSetDeviceSecurityDescriptors(
    UpperDeviceObject: PDEVICE_OBJECT,
    PhysicalDeviceObject: PDEVICE_OBJECT,
    SecurityInformation: *mut u32,
    SecurityDescriptor: PSECURITY_DESCRIPTOR,
    PoolType: u32,
    GenericMapping: PGENERIC_MAPPING,
) -> NTSTATUS {
    let mut CurrentDO = PhysicalDeviceObject;
    let mut Status = STATUS_SUCCESS;
    ObReferenceObject(CurrentDO as PVOID);

    loop {
        let TmpStatus = IopSetDeviceSecurityDescriptor(
            CurrentDO,
            SecurityInformation,
            SecurityDescriptor,
            PoolType,
            GenericMapping,
        );
        if CurrentDO == UpperDeviceObject {
            Status = TmpStatus;
        }

        let NextDevice = (*CurrentDO).AttachedDevice;
        if !NextDevice.is_null() {
            ObReferenceObject(NextDevice as PVOID);
        }
        ObDereferenceObject(CurrentDO as PVOID);
        CurrentDO = NextDevice;
        if CurrentDO.is_null() {
            break;
        }
    }
    Status
}

#[no_mangle]
pub unsafe extern "system" fn IopGetSetSecurityObject(
    ObjectBody: PVOID,
    OperationCode: u32,
    SecurityInformation: *mut u32,
    SecurityDescriptor: PSECURITY_DESCRIPTOR,
    BufferLength: *mut ULONG,
    OldSecurityDescriptor: *mut PSECURITY_DESCRIPTOR,
    PoolType: u32,
    GenericMapping: PGENERIC_MAPPING,
    _AccessMode: KPROCESSOR_MODE,
) -> NTSTATUS {
    let ObjectBody = opaque_ptr(ObjectBody);
    let SecurityInformation = opaque_ptr(SecurityInformation);
    let SecurityDescriptor = opaque_ptr(SecurityDescriptor);
    let BufferLength = opaque_ptr(BufferLength);
    let OldSecurityDescriptor = opaque_ptr(OldSecurityDescriptor);
    let GenericMapping = opaque_ptr(GenericMapping);

    let DeviceObject: PDEVICE_OBJECT;
    let FileObject: PFILE_OBJECT;

    if (*(ObjectBody as PFILE_OBJECT)).Type == IO_TYPE_FILE as i16 {
        FileObject = ObjectBody as PFILE_OBJECT;
        DeviceObject = if (*FileObject).Flags & FO_DIRECT_DEVICE_OPEN != 0 {
            IoGetAttachedDevice((*FileObject).DeviceObject)
        } else {
            (*FileObject).DeviceObject
        };
    } else {
        DeviceObject = ObjectBody as PDEVICE_OBJECT;
        FileObject = null_mut();
    }

    if FileObject.is_null()
        || (((*FileObject).FileName.Length == 0) && (*FileObject).RelatedFileObject.is_null())
        || ((*FileObject).Flags & FO_DIRECT_DEVICE_OPEN != 0)
    {
        if OperationCode == _SECURITY_OPERATION_CODE_QuerySecurityDescriptor {
            return SeQuerySecurityDescriptorInfo(
                SecurityInformation,
                SecurityDescriptor,
                BufferLength,
                &mut (*DeviceObject).SecurityDescriptor,
            );
        } else if OperationCode == _SECURITY_OPERATION_CODE_DeleteSecurityDescriptor {
            return STATUS_SUCCESS;
        } else if OperationCode == _SECURITY_OPERATION_CODE_AssignSecurityDescriptor {
            let mut Status = STATUS_SUCCESS;
            if FileObject.is_null() || ((*FileObject).Flags & FO_STREAM_FILE == 0) {
                let mut CachedSecurityDescriptor: PSECURITY_DESCRIPTOR = null_mut();
                Status = ObLogSecurityDescriptor(SecurityDescriptor, &mut CachedSecurityDescriptor, 1);
                if Status >= 0 {
                    KeEnterCriticalRegion();
                    ExAcquireResourceExclusiveLite(&raw mut IopSecurityResource, 1);
                    (*DeviceObject).SecurityDescriptor = CachedSecurityDescriptor;
                    ExReleaseResourceLite(&raw mut IopSecurityResource);
                    KeLeaveCriticalRegion();
                }
            }
            return Status;
        } else if OperationCode == _SECURITY_OPERATION_CODE_SetSecurityDescriptor {
            let PDO = IopGetDevicePDO(DeviceObject);
            let Status = if !PDO.is_null() {
                let s = IopSetDeviceSecurityDescriptors(
                    DeviceObject,
                    PDO,
                    SecurityInformation,
                    SecurityDescriptor,
                    PoolType,
                    GenericMapping,
                );
                ObDereferenceObject(PDO as PVOID);
                s
            } else {
                IopSetDeviceSecurityDescriptor(
                    DeviceObject,
                    SecurityInformation,
                    SecurityDescriptor,
                    PoolType,
                    GenericMapping,
                )
            };
            return Status;
        }
        return STATUS_SUCCESS;
    } else if OperationCode == _SECURITY_OPERATION_CODE_DeleteSecurityDescriptor {
        return STATUS_SUCCESS;
    }

    ObReferenceObject(FileObject as PVOID);
    let mut Status: NTSTATUS;

    if (*FileObject).Flags & FO_SYNCHRONOUS_IO != 0 {
        Status = IopLockFileObject(FileObject, KeGetPreviousMode());
        if Status != STATUS_SUCCESS {
            ObDereferenceObject(FileObject as PVOID);
            return Status;
        }
    }

    let LocalEvent = (*FileObject).Flags & FO_SYNCHRONOUS_IO == 0;
    let mut Event: _KEVENT = zeroed();
    if LocalEvent {
        KeInitializeEvent(&mut Event, 1, 0);  // SynchronizationEvent
    }

    KeClearEvent(&mut (*FileObject).Event);
    let RelatedDevice = IoGetRelatedDeviceObject(FileObject);
    let Irp = IoAllocateIrp((*RelatedDevice).StackSize as i8, 0);
    if Irp.is_null() {
        return IopCleanupFailedIrp(FileObject, null_mut(), null_mut());
    }

    let mut IoStatusBlock: _IO_STATUS_BLOCK = zeroed();
    (*Irp).Tail.Overlay.OriginalFileObject = FileObject;
    (*Irp).Tail.Overlay.Thread = PsGetCurrentThread() as PETHREAD;
    (*Irp).RequestorMode = KeGetPreviousMode();
    (*Irp).UserIosb = core::ptr::addr_of_mut!(IoStatusBlock) as PIO_STATUS_BLOCK;
    (*Irp).UserEvent = if LocalEvent {
        core::ptr::addr_of_mut!(Event) as *mut _KEVENT
    } else {
        null_mut()
    };
    (*Irp).Flags = if LocalEvent { IRP_SYNCHRONOUS_API } else { 0 };
    (*Irp).Overlay.AsynchronousParameters.__bindgen_anon_1.UserApcRoutine = None;

    let StackPtr = IoGetNextIrpStackLocation(Irp);
    (*StackPtr).FileObject = FileObject;

    if OperationCode == _SECURITY_OPERATION_CODE_QuerySecurityDescriptor {
        (*StackPtr).MajorFunction = IRP_MJ_QUERY_SECURITY as u8;
        (*StackPtr).Parameters.QuerySecurity.SecurityInformation = *SecurityInformation;
        (*StackPtr).Parameters.QuerySecurity.Length = *BufferLength;
        (*Irp).UserBuffer = SecurityDescriptor as PVOID;
    } else {
        (*StackPtr).MajorFunction = IRP_MJ_SET_SECURITY as u8;
        (*StackPtr).Parameters.SetSecurity.SecurityInformation = *SecurityInformation;
        (*StackPtr).Parameters.SetSecurity.SecurityDescriptor = SecurityDescriptor;
    }

    IopQueueIrpToThread(Irp);
    IopUpdateOperationCount(IopOtherTransfer);
    Status = core::hint::black_box(IoCallDriver(RelatedDevice, Irp));

    if LocalEvent {
        if Status == STATUS_PENDING {
            KeWaitForSingleObject(core::ptr::addr_of_mut!(Event) as PVOID, 0, 0i8, 0, null_mut());
            Status = core::ptr::read_volatile(&IoStatusBlock.__bindgen_anon_1.Status);
        }
    } else {
        if Status == STATUS_PENDING {
            KeWaitForSingleObject(
                core::ptr::addr_of_mut!((*FileObject).Event) as PVOID,
                0,
                0i8,
                0u8,
                null_mut(),
            );
            Status = core::ptr::read_volatile(&(*FileObject).FinalStatus);
        }
        IopUnlockFileObject(FileObject);
    }

    if Status == STATUS_INVALID_DEVICE_REQUEST {
        if OperationCode == _SECURITY_OPERATION_CODE_QuerySecurityDescriptor {
            Status = SeSetWorldSecurityDescriptor(*SecurityInformation, SecurityDescriptor as *mut _SECURITY_DESCRIPTOR, BufferLength);
        } else {
            Status = STATUS_SUCCESS;
        }
    } else if OperationCode == _SECURITY_OPERATION_CODE_QuerySecurityDescriptor {
        if Status == STATUS_BUFFER_OVERFLOW {
            Status = STATUS_BUFFER_TOO_SMALL;
        }
        *BufferLength = IoStatusBlock.Information as ULONG;
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn IopQueryName(
    ObjectBody: PVOID,
    HasName: BOOLEAN,
    ObjectNameInfo: POBJECT_NAME_INFORMATION,
    Length: ULONG,
    ReturnLength: *mut ULONG,
    PreviousMode: KPROCESSOR_MODE,
) -> NTSTATUS {
    let ObjectBody = opaque_ptr(ObjectBody);
    let ObjectNameInfo = opaque_ptr(ObjectNameInfo);
    let ReturnLength = opaque_ptr(ReturnLength);
    IopQueryNameInternal(ObjectBody, HasName, 0u8, ObjectNameInfo, Length, ReturnLength, PreviousMode)
}

// FIELD_OFFSET(FILE_NAME_INFORMATION, FileName): 4 on every arch. This function had it as a
// literal 8 (sizeof the padded struct), so every name built here came out 4 bytes, two
// characters, short: process image names read "svchost.e" and "setup.e" in Toolhelp and
// Task Manager.
const FNI_NAME_OFFSET: u32 = core::mem::offset_of!(_FILE_NAME_INFORMATION, FileName) as u32;

unsafe fn IopQueryNameInternal(
    ObjectBody: PVOID,
    HasName: BOOLEAN,
    QueryDosName: BOOLEAN,
    ObjectNameInfo: POBJECT_NAME_INFORMATION,
    Length: ULONG,
    ReturnLength: *mut ULONG,
    PreviousMode: KPROCESSOR_MODE,
) -> NTSTATUS {

    let FileObject = ObjectBody as PFILE_OBJECT;

    if Length < size_of::<_OBJECT_NAME_INFORMATION>() as u32 {
        *ReturnLength = size_of::<_OBJECT_NAME_INFORMATION>() as u32;
        return STATUS_INFO_LENGTH_MISMATCH;
    }

    let LocalInfo = ExAllocatePoolWithTag(PagedPool as u32, Length as usize, TAG_IO)
        as POBJECT_NAME_INFORMATION;
    if LocalInfo.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    let mut Status: NTSTATUS;
    let mut LocalReturnLength: ULONG = 0;
    let NoObCall: BOOLEAN;

    if QueryDosName != 0 {
        let DeviceObject = (*FileObject).DeviceObject;
        if (*DeviceObject).DeviceType == FILE_DEVICE_NETWORK_FILE_SYSTEM {
            LocalReturnLength = size_of::<_OBJECT_NAME_INFORMATION>() as u32 + 4;
            if Length < LocalReturnLength {
                Status = STATUS_BUFFER_OVERFLOW;
            } else {
                (*LocalInfo).Name.Length = 2;
                (*LocalInfo).Name.MaximumLength = 2;
                (*LocalInfo).Name.Buffer =
                    (LocalInfo as ULONG_PTR + size_of::<_OBJECT_NAME_INFORMATION>()) as *mut u16;
                *(*LocalInfo).Name.Buffer = OBJ_NAME_PATH_SEPARATOR;
                Status = STATUS_SUCCESS;
            }
            NoObCall = 1;
        } else {
            Status = IoVolumeDeviceToDosName(DeviceObject as PVOID, &mut (*LocalInfo).Name);
            LocalReturnLength = (*LocalInfo).Name.Length as u32
                + size_of::<_OBJECT_NAME_INFORMATION>() as u32
                + 2;
            NoObCall = 1;
        }
    } else {
        Status = ObQueryNameString(
            (*FileObject).DeviceObject as PVOID,
            LocalInfo,
            Length,
            &mut LocalReturnLength,
        );
        NoObCall = 0;
    }

    if Status < 0 && Status != STATUS_INFO_LENGTH_MISMATCH {
        ExFreePoolWithTag(LocalInfo as PVOID, TAG_IO);
        return Status;
    }

    let mut p = (ObjectNameInfo as ULONG_PTR + size_of::<_OBJECT_NAME_INFORMATION>()) as *mut u16;
    let mut LengthMismatch = 0u8;

    if QueryDosName != 0 && NoObCall != 0 {
        copy_nonoverlapping(
            LocalInfo as *const u8,
            ObjectNameInfo as *mut u8,
            if Length >= LocalReturnLength {
                size_of::<_OBJECT_NAME_INFORMATION>()
            } else {
                Length as usize
            },
        );
        copy_nonoverlapping(
            (*LocalInfo).Name.Buffer as *const u8,
            p as *mut u8,
            if Length >= LocalReturnLength {
                (*LocalInfo).Name.Length as usize
            } else {
                (Length - size_of::<_OBJECT_NAME_INFORMATION>() as u32) as usize
            },
        );

        if (*FileObject).DeviceObject as PDEVICE_OBJECT
            != (null_mut() as PDEVICE_OBJECT)
            && (*(*FileObject).DeviceObject).DeviceType != FILE_DEVICE_NETWORK_FILE_SYSTEM
        {
            ExFreePool((*LocalInfo).Name.Buffer as PVOID);
        }
    } else {
        copy_nonoverlapping(
            LocalInfo as *const u8,
            ObjectNameInfo as *mut u8,
            if LocalReturnLength > Length {
                Length as usize
            } else {
                LocalReturnLength as usize
            },
        );
    }

    (*ObjectNameInfo).Name.Buffer = p;
    p = p.add(((*LocalInfo).Name.Length / 2) as usize);

    if LocalReturnLength > Length {
        LengthMismatch = 1;
        *ReturnLength = LocalReturnLength;
    }

    let LocalFileInfo = LocalInfo as PFILE_NAME_INFORMATION;
    let FileLength = Length - LocalReturnLength + FNI_NAME_OFFSET;

    if PreviousMode == 0i8 && ((*FileObject).Flags & FO_SYNCHRONOUS_IO != 0) {
        Status = IopGetFileInformation(
            FileObject,
            if LengthMismatch != 0 { Length } else { FileLength },
            9,
            LocalFileInfo as PVOID,
            &mut LocalReturnLength,
        );
    } else {
        Status = IoQueryFileInformation(
            FileObject,
            9,
            if LengthMismatch != 0 { Length } else { FileLength },
            LocalFileInfo as PVOID,
            &mut LocalReturnLength,
        );
    }

    if Status < 0 && ((Status as u32) >> 30) == 3 {  // NT_ERROR
        if Status != STATUS_INVALID_PARAMETER
            && Status != STATUS_INVALID_DEVICE_REQUEST
            && Status != STATUS_NOT_IMPLEMENTED
            && Status != STATUS_INVALID_INFO_CLASS
        {
            ExFreePoolWithTag(LocalInfo as PVOID, TAG_IO);
            return Status;
        }
        LocalReturnLength = FNI_NAME_OFFSET;
        (*LocalFileInfo).FileNameLength = 0;
        (*LocalFileInfo).FileName[0] = OBJ_NAME_PATH_SEPARATOR;
        Status = STATUS_SUCCESS;
    } else if LocalReturnLength < FNI_NAME_OFFSET {
        LocalReturnLength = FNI_NAME_OFFSET;
    }

    if LengthMismatch != 0 {
        *ReturnLength += (*LocalFileInfo).FileNameLength;
        ExFreePoolWithTag(LocalInfo as PVOID, TAG_IO);
        return STATUS_BUFFER_OVERFLOW;
    }

    let FileLength = LocalReturnLength - FNI_NAME_OFFSET;
    LocalReturnLength = (p as ULONG_PTR - ObjectNameInfo as ULONG_PTR) as u32
        + (*LocalFileInfo).FileNameLength;

    if (*LocalFileInfo).FileName[0] != OBJ_NAME_PATH_SEPARATOR {
        ExFreePoolWithTag(LocalInfo as PVOID, TAG_IO);
        return STATUS_OBJECT_PATH_INVALID;
    }

    copy_nonoverlapping(
        (*LocalFileInfo).FileName.as_ptr() as *const u8,
        p as *mut u8,
        FileLength as usize,
    );
    p = p.add((FileLength / 2) as usize);
    *p = 0;
    LocalReturnLength += 2;

    *ReturnLength = LocalReturnLength;

    let FileLength = (p as ULONG_PTR - ObjectNameInfo as ULONG_PTR) as u32;
    (*ObjectNameInfo).Name.Length = (FileLength - size_of::<_OBJECT_NAME_INFORMATION>() as u32) as u16;
    (*ObjectNameInfo).Name.MaximumLength = (*ObjectNameInfo).Name.Length + 2;

    ExFreePoolWithTag(LocalInfo as PVOID, TAG_IO);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn IopCloseFile(
    Process: PEPROCESS,
    ObjectBody: PVOID,
    GrantedAccess: ACCESS_MASK,
    HandleCount: ULONG_PTR,
    SystemHandleCount: ULONG_PTR,
) {
    let Process = opaque_ptr(Process);
    let ObjectBody = opaque_ptr(ObjectBody);

    let FileObject = ObjectBody as PFILE_OBJECT;

    if HandleCount != 1 {
        return;
    }

    if ((*FileObject).LockOperation != 0) && (SystemHandleCount != 1) {
        let DeviceObject = if (*FileObject).Flags & FO_DIRECT_DEVICE_OPEN != 0 {
            IoGetAttachedDevice((*FileObject).DeviceObject)
        } else {
            IoGetRelatedDeviceObject(FileObject)
        };

        if (*FileObject).Flags & FO_SYNCHRONOUS_IO != 0 {
            IopLockFileObject(FileObject, 0i8);
        }

        let mut IoStatusBlock: _IO_STATUS_BLOCK = zeroed();
        if (*DeviceObject).DriverObject.is_null()
            || (*(*DeviceObject).DriverObject).FastIoDispatch.is_null()
            || (*(*(*DeviceObject).DriverObject).FastIoDispatch)
                .FastIoUnlockAll
                .is_none()
            || (*(*(*DeviceObject).DriverObject).FastIoDispatch)
                .FastIoUnlockAll
                .unwrap()(FileObject, PsGetCurrentProcess() as _, &mut IoStatusBlock, DeviceObject)
                == 0
        {
            KeClearEvent(&mut (*FileObject).Event);
            let mut Event: _KEVENT = zeroed();
            KeInitializeEvent(&mut Event, 1, 0);  // SynchronizationEvent
            let Irp = IopAllocateIrpMustSucceed((*DeviceObject).StackSize as i8);
            (*Irp).UserEvent = core::ptr::addr_of_mut!(Event) as *mut _KEVENT;
            (*Irp).UserIosb = addr_of_mut!((*Irp).IoStatus) as PIO_STATUS_BLOCK;
            (*Irp).Tail.Overlay.Thread = PsGetCurrentThread() as PETHREAD;
            (*Irp).Tail.Overlay.OriginalFileObject = FileObject;
            (*Irp).RequestorMode = 0i8;
            (*Irp).Flags = IRP_SYNCHRONOUS_API;
            (*Irp).Overlay.AsynchronousParameters.__bindgen_anon_1.UserApcRoutine = None;
            ObReferenceObject(FileObject as PVOID);
            let StackPtr = IoGetNextIrpStackLocation(Irp);
            (*StackPtr).MajorFunction = IRP_MJ_LOCK_CONTROL as u8;
            (*StackPtr).MinorFunction = IRP_MN_UNLOCK_ALL as u8;
            (*StackPtr).FileObject = FileObject;
            IopQueueIrpToThread(Irp);
            let Status = core::hint::black_box(IoCallDriver(DeviceObject, Irp));
            if Status == STATUS_PENDING {
                KeWaitForSingleObject(core::ptr::addr_of_mut!(Event) as PVOID, 1, 0i8, 0, null_mut());
            }
        }

        if (*FileObject).Flags & FO_SYNCHRONOUS_IO != 0 {
            IopUnlockFileObject(FileObject);
        }
    }

    if SystemHandleCount != 1 {
        return;
    }

    let DeviceObject = if (*FileObject).Flags & FO_DIRECT_DEVICE_OPEN != 0 {
        IoGetAttachedDevice((*FileObject).DeviceObject)
    } else {
        IoGetRelatedDeviceObject(FileObject)
    };

    (*FileObject).Flags |= FO_HANDLE_CREATED;

    if !Process.is_null() && ((*FileObject).Flags & FO_SYNCHRONOUS_IO != 0) {
        IopLockFileObject(FileObject, 0i8);
    }

    KeClearEvent(&mut (*FileObject).Event);
    let mut Event: _KEVENT = zeroed();
    KeInitializeEvent(&mut Event, 1, 0);  // SynchronizationEvent
    let Irp = IopAllocateIrpMustSucceed((*DeviceObject).StackSize as i8);
    (*Irp).UserEvent = core::ptr::addr_of_mut!(Event) as *mut _KEVENT;
    (*Irp).UserIosb = addr_of_mut!((*Irp).IoStatus) as PIO_STATUS_BLOCK;
    (*Irp).Tail.Overlay.Thread = PsGetCurrentThread() as PETHREAD;
    (*Irp).Tail.Overlay.OriginalFileObject = FileObject;
    (*Irp).Overlay.AsynchronousParameters.__bindgen_anon_1.UserApcRoutine = None;
    (*Irp).Flags = IRP_CLOSE_OPERATION | IRP_SYNCHRONOUS_API;
    let StackPtr = IoGetNextIrpStackLocation(Irp);
    (*StackPtr).MajorFunction = IRP_MJ_CLEANUP as u8;
    (*StackPtr).FileObject = FileObject;
    IopQueueIrpToThread(Irp);
    IopUpdateOperationCount(IopOtherTransfer);
    let Status = core::hint::black_box(IoCallDriver(DeviceObject, Irp));
    if Status == STATUS_PENDING {
        KeWaitForSingleObject(core::ptr::addr_of_mut!(Event) as PVOID, 1, 0i8, 0, null_mut());
    }
    let mut OldIrql: KIRQL = 0;
    // [guard] KeRaiseIrql(2, &mut OldIrql);
    let irql_guard = unsafe { ntkrnl_sys::guards::IrqlGuard::raise(2) };
    IopUnQueueIrpFromThread(Irp);
    drop(irql_guard);
    IoFreeIrp(Irp);

    if !Process.is_null() && ((*FileObject).Flags & FO_SYNCHRONOUS_IO != 0) {
        IopUnlockFileObject(FileObject);
    }
}

unsafe fn IopQueryAttributesFile(
    ObjectAttributes: POBJECT_ATTRIBUTES,
    FileInformationClass: u32,
    FileInformationSize: ULONG,
    FileInformation: PVOID,
) -> NTSTATUS {
    let AccessMode = KeGetPreviousMode();

    if AccessMode != 0i8 {
        ProbeForWrite(FileInformation, FileInformationSize as usize, 4);
    }

    let IsBasic = FileInformationSize == size_of::<_FILE_BASIC_INFORMATION>() as u32;
    let mut OpenPacket: _OPEN_PACKET = zeroed();
    let mut LocalFileObject: _DUMMY_FILE_OBJECT = zeroed();
    let mut NetworkOpenInfo: _FILE_NETWORK_OPEN_INFORMATION = zeroed();

    OpenPacket.Type = IO_TYPE_OPEN_PACKET as i16;
    OpenPacket.Size = size_of::<_OPEN_PACKET>() as i16;
    OpenPacket.CreateOptions = FILE_OPEN_REPARSE_POINT;
    OpenPacket.ShareAccess = (FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE) as u16;
    OpenPacket.Disposition = FILE_OPEN;
    OpenPacket.BasicInformation = if IsBasic {
        FileInformation as _
    } else {
        null_mut()
    };
    OpenPacket.NetworkInformation = if IsBasic {
        core::ptr::addr_of_mut!(NetworkOpenInfo)
    } else if AccessMode != 0i8 {
        core::ptr::addr_of_mut!(NetworkOpenInfo)
    } else {
        FileInformation as _
    };
    OpenPacket.QueryOnly = 1;
    OpenPacket.FullAttributes = if IsBasic { 0 } else { 1 };
    OpenPacket.LocalFileObject = core::ptr::addr_of_mut!(LocalFileObject);

    IopUpdateOperationCount(IopOtherTransfer);

    let mut Handle: HANDLE = 0 as _;
    let mut Status = ObOpenObjectByName(
        ObjectAttributes,
        null_mut(),
        AccessMode,
        null_mut(),
        FILE_READ_ATTRIBUTES,
        core::ptr::addr_of_mut!(OpenPacket) as PVOID,
        &mut Handle,
    );
    if OpenPacket.ParseCheck == 0 {
        return Status;
    }

    Status = OpenPacket.FinalStatus;

    if Status >= 0 && AccessMode != 0i8 && !IsBasic {
        copy_nonoverlapping(
            &NetworkOpenInfo as *const _ as *const u8,
            FileInformation as *mut u8,
            FileInformationSize as usize,
        );
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn IopAcquireFileObjectLock(
    FileObject: PFILE_OBJECT,
    WaitMode: KPROCESSOR_MODE,
    Alertable: BOOLEAN,
    LockFailed: PBOOLEAN,
) -> NTSTATUS {
    let FileObject = opaque_ptr(FileObject);
    let LockFailed = opaque_ptr(LockFailed);

    InterlockedIncrement(&raw mut (*FileObject).Waiters as *mut LONG);

    let mut Status = STATUS_SUCCESS;
    loop {
        if InterlockedExchange(&raw mut (*FileObject).Busy as *mut LONG, 1) == 0 {
            break;
        }
        Status = KeWaitForSingleObject(
            core::ptr::addr_of_mut!((*FileObject).Lock) as PVOID,
            0,
            WaitMode,
            Alertable,
            null_mut(),
        );
        if Status != STATUS_SUCCESS {
            break;
        }
    }

    InterlockedDecrement(&raw mut (*FileObject).Waiters as *mut LONG);
    if Status == STATUS_SUCCESS {
        ObReferenceObject(FileObject as PVOID);
        *LockFailed = 0;
    } else {
        if (*FileObject).Busy == 0 && (*FileObject).Waiters != 0 {
            KeSetEvent(&mut (*FileObject).Lock, 0, 0);
        }
        *LockFailed = 1;
    }
    Status
}

#[no_mangle]
pub unsafe extern "system" fn IoGetFileObjectFilterContext(FileObject: PFILE_OBJECT) -> PVOID {
    let FileObject = opaque_ptr(FileObject);
    if (*FileObject).Flags & FO_FILE_OBJECT_HAS_EXTENSION != 0 {
        let FileObjectExtension = (*FileObject).FileObjectExtension as PFILE_OBJECT_EXTENSION;
        return (*FileObjectExtension).FilterContext;
    }
    null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn IoChangeFileObjectFilterContext(
    FileObject: PFILE_OBJECT,
    FilterContext: PVOID,
    Define: BOOLEAN,
) -> NTSTATUS {
    let FileObject = opaque_ptr(FileObject);
    let FilterContext = opaque_ptr(FilterContext);
    if (*FileObject).Flags & FO_FILE_OBJECT_HAS_EXTENSION == 0 {
        return STATUS_INVALID_PARAMETER;
    }

    let FileObjectExtension = (*FileObject).FileObjectExtension as PFILE_OBJECT_EXTENSION;
    let Success = if Define != 0 {
        InterlockedCompareExchangePointer(
            &mut (*FileObjectExtension).FilterContext,
            FilterContext,
            null_mut(),
        ) as ULONG_PTR
    } else {
        InterlockedCompareExchangePointer(
            &mut (*FileObjectExtension).FilterContext,
            null_mut(),
            FilterContext,
        ) as ULONG_PTR
            - FilterContext as ULONG_PTR
    };

    if Success != 0 {
        STATUS_ALREADY_COMMITTED
    } else {
        STATUS_SUCCESS
    }
}

unsafe fn IopCreateFile(
    FileHandle: *mut HANDLE,
    DesiredAccess: ACCESS_MASK,
    ObjectAttributes: POBJECT_ATTRIBUTES,
    IoStatusBlock: PIO_STATUS_BLOCK,
    AllocationSize: *mut LARGE_INTEGER,
    FileAttributes: ULONG,
    ShareAccess: ULONG,
    Disposition: ULONG,
    mut CreateOptions: ULONG,
    EaBuffer: PVOID,
    EaLength: ULONG,
    CreateFileType: u32,
    ExtraCreateParameters: PVOID,
    mut Options: ULONG,
    Flags: ULONG,
    DeviceObject: PDEVICE_OBJECT,
) -> NTSTATUS {

    let AccessMode = if Options & IO_NO_PARAMETER_CHECKING != 0 {
        0i8
    } else {
        KeGetPreviousMode()
    };

    if (AccessMode != 0i8) || (Options & IO_CHECK_CREATE_PARAMETERS != 0) {
        if FileAttributes & !FILE_ATTRIBUTE_VALID_FLAGS != 0 {
            return STATUS_INVALID_PARAMETER;
        }
        if ShareAccess & !FILE_SHARE_VALID_FLAGS != 0 {
            return STATUS_INVALID_PARAMETER;
        }
        if Disposition > FILE_MAXIMUM_DISPOSITION {
            return STATUS_INVALID_PARAMETER;
        }
        if CreateOptions & !FILE_VALID_OPTION_FLAGS != 0 {
            return STATUS_INVALID_PARAMETER;
        }
        if (CreateOptions & (FILE_SYNCHRONOUS_IO_ALERT | FILE_SYNCHRONOUS_IO_NONALERT) != 0)
            && (DesiredAccess & SYNCHRONIZE == 0)
        {
            return STATUS_INVALID_PARAMETER;
        }
        if (CreateOptions & FILE_DELETE_ON_CLOSE != 0) && (DesiredAccess & DELETE == 0) {
            return STATUS_INVALID_PARAMETER;
        }
        if (CreateOptions & (FILE_SYNCHRONOUS_IO_NONALERT | FILE_SYNCHRONOUS_IO_ALERT))
            == (FILE_SYNCHRONOUS_IO_NONALERT | FILE_SYNCHRONOUS_IO_ALERT)
        {
            return STATUS_INVALID_PARAMETER;
        }
        if (CreateOptions & FILE_DIRECTORY_FILE != 0)
            && (CreateOptions & FILE_NON_DIRECTORY_FILE == 0)
            && (CreateOptions
                & !(FILE_DIRECTORY_FILE
                    | FILE_SYNCHRONOUS_IO_ALERT
                    | FILE_SYNCHRONOUS_IO_NONALERT
                    | FILE_WRITE_THROUGH
                    | FILE_COMPLETE_IF_OPLOCKED
                    | FILE_OPEN_FOR_BACKUP_INTENT
                    | FILE_DELETE_ON_CLOSE
                    | FILE_OPEN_FOR_FREE_SPACE_QUERY
                    | FILE_OPEN_BY_FILE_ID
                    | FILE_NO_COMPRESSION
                    | FILE_OPEN_REPARSE_POINT)
                != 0)
        {
            return STATUS_INVALID_PARAMETER;
        }
        if (CreateOptions & FILE_DIRECTORY_FILE != 0)
            && (CreateOptions & FILE_NON_DIRECTORY_FILE == 0)
            && (Disposition != FILE_CREATE)
            && (Disposition != FILE_OPEN)
            && (Disposition != FILE_OPEN_IF)
        {
            return STATUS_INVALID_PARAMETER;
        }
        if (CreateOptions & FILE_COMPLETE_IF_OPLOCKED != 0)
            && (CreateOptions & FILE_RESERVE_OPFILTER != 0)
        {
            return STATUS_INVALID_PARAMETER;
        }
        if (CreateOptions & FILE_NO_INTERMEDIATE_BUFFERING != 0)
            && (DesiredAccess & FILE_APPEND_DATA != 0)
        {
            return STATUS_INVALID_PARAMETER;
        }

        if CreateFileType == CreateFileTypeNamedPipe {
            if ExtraCreateParameters.is_null() {
                return STATUS_INVALID_PARAMETER;
            }
            let NamedPipeCreateParameters = ExtraCreateParameters as PNAMED_PIPE_CREATE_PARAMETERS;
            if ((*NamedPipeCreateParameters).NamedPipeType > FILE_PIPE_MESSAGE_TYPE)
                || ((*NamedPipeCreateParameters).ReadMode > FILE_PIPE_MESSAGE_MODE)
                || ((*NamedPipeCreateParameters).CompletionMode > FILE_PIPE_COMPLETE_OPERATION)
                || (ShareAccess & FILE_SHARE_DELETE != 0)
                || ((Disposition < FILE_OPEN) || (Disposition > FILE_OPEN_IF))
                || (CreateOptions & !FILE_VALID_PIPE_OPTION_FLAGS != 0)
            {
                return STATUS_INVALID_PARAMETER;
            }
        } else if CreateFileType == CreateFileTypeMailslot {
            if ExtraCreateParameters.is_null() {
                return STATUS_INVALID_PARAMETER;
            }
            if (ShareAccess & FILE_SHARE_DELETE != 0)
                || (ShareAccess & !FILE_SHARE_WRITE == 0)
                || (Disposition != FILE_CREATE)
                || (CreateOptions & !FILE_VALID_MAILSLOT_OPTION_FLAGS != 0)
            {
                return STATUS_INVALID_PARAMETER;
            }
        }
    }

    let OpenPacket = ExAllocatePoolWithTag(
        NonPagedPool as u32,
        size_of::<_OPEN_PACKET>(),
        u32::from_le_bytes(*b"pOoI"),
    ) as POPEN_PACKET;
    if OpenPacket.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    write_bytes(OpenPacket as *mut u8, 0, size_of::<_OPEN_PACKET>());

    let mut SafeAllocationSize: LARGE_INTEGER = zeroed();
    let mut Status = STATUS_SUCCESS;

    if AccessMode != 0i8 {
        ProbeForWriteHandle(FileHandle);
        ProbeForWriteIoStatusBlock(IoStatusBlock);

        if !AllocationSize.is_null() {
            SafeAllocationSize = ProbeForReadLargeInteger(AllocationSize);
            if SafeAllocationSize.QuadPart < 0 {
                ExFreePool(OpenPacket as PVOID);
                return STATUS_INVALID_PARAMETER;
            }
        }

        if !EaBuffer.is_null() && EaLength != 0 {
            ProbeForRead(EaBuffer, EaLength as usize, 4);
            (*OpenPacket).EaBuffer = ExAllocatePoolWithTag(NonPagedPool as u32, EaLength as usize, TAG_EA);
            (*OpenPacket).EaLength = EaLength;
            copy_nonoverlapping(EaBuffer as *const u8, (*OpenPacket).EaBuffer as *mut u8, EaLength as usize);

            let mut EaErrorOffset: ULONG = 0;
            Status = IoCheckEaBufferValidity((*OpenPacket).EaBuffer, EaLength, &mut EaErrorOffset);
            if Status < 0 {
                (*IoStatusBlock).__bindgen_anon_1.Status = Status;
                (*IoStatusBlock).Information = EaErrorOffset as _;
                if !(*OpenPacket).EaBuffer.is_null() {
                    ExFreePool((*OpenPacket).EaBuffer);
                }
                ExFreePool(OpenPacket as PVOID);
                return Status;
            }
        }
    } else {
        if CreateOptions & IO_ATTACH_DEVICE_API != 0 {
            Options |= IO_ATTACH_DEVICE;
            CreateOptions &= !IO_ATTACH_DEVICE_API;
        }

        if !AllocationSize.is_null() {
            SafeAllocationSize = *AllocationSize;
        }

        if !EaBuffer.is_null() && EaLength != 0 {
            (*OpenPacket).EaBuffer = ExAllocatePoolWithTag(NonPagedPool as u32, EaLength as usize, TAG_EA);
            if (*OpenPacket).EaBuffer.is_null() {
                ExFreePool(OpenPacket as PVOID);
                return STATUS_INSUFFICIENT_RESOURCES;
            }
            (*OpenPacket).EaLength = EaLength;
            copy_nonoverlapping(EaBuffer as *const u8, (*OpenPacket).EaBuffer as *mut u8, EaLength as usize);

            let mut EaErrorOffset: ULONG = 0;
            Status = IoCheckEaBufferValidity((*OpenPacket).EaBuffer, EaLength, &mut EaErrorOffset);
            if Status < 0 {
                ExFreePool((*OpenPacket).EaBuffer);
                (*IoStatusBlock).__bindgen_anon_1.Status = Status;
                (*IoStatusBlock).Information = EaErrorOffset as _;
                ExFreePool(OpenPacket as PVOID);
                return Status;
            }
        }
    }

    (*OpenPacket).Type = IO_TYPE_OPEN_PACKET as i16;
    (*OpenPacket).Size = size_of::<_OPEN_PACKET>() as i16;
    (*OpenPacket).AllocationSize = core::mem::transmute(SafeAllocationSize);
    (*OpenPacket).CreateOptions = CreateOptions;
    (*OpenPacket).FileAttributes = FileAttributes as u16;
    (*OpenPacket).ShareAccess = ShareAccess as u16;
    (*OpenPacket).Options = Options;
    (*OpenPacket).Disposition = Disposition;
    (*OpenPacket).CreateFileType = CreateFileType;
    (*OpenPacket).ExtraCreateParameters = ExtraCreateParameters;
    (*OpenPacket).InternalFlags = Flags;
    (*OpenPacket).TopDeviceObjectHint = DeviceObject;

    IopUpdateOperationCount(IopOtherTransfer);

    let mut LocalHandle: HANDLE = 0 as _;
    Status = ObOpenObjectByName(
        ObjectAttributes,
        null_mut(),
        AccessMode,
        null_mut(),
        DesiredAccess,
        OpenPacket as PVOID,
        &mut LocalHandle,
    );

    // Trace: ObOpenObjectByName returned Status, ParseCheck value

    if !(*OpenPacket).EaBuffer.is_null() {
        ExFreePool((*OpenPacket).EaBuffer);
    }

    if Status < 0 || (*OpenPacket).ParseCheck == 0 {
        if Status >= 0 {
            ObCloseHandle(LocalHandle, 0i8);
            Status = STATUS_OBJECT_TYPE_MISMATCH;
        }

        if (*OpenPacket).FinalStatus < 0 {
            Status = (*OpenPacket).FinalStatus;
            if ((Status as u32) >> 30) == 2 {  // NT_WARNING
                (*IoStatusBlock).Information = (*OpenPacket).Information;
                (*IoStatusBlock).__bindgen_anon_1.Status = (*OpenPacket).FinalStatus;
            }
        } else if !(*OpenPacket).FileObject.is_null() && (*OpenPacket).ParseCheck == 0 {
            if (*(*OpenPacket).FileObject).FileName.Length != 0 {
                ExFreePoolWithTag((*(*OpenPacket).FileObject).FileName.Buffer as PVOID, TAG_IO_NAME);
            }
            (*(*OpenPacket).FileObject).DeviceObject = null_mut();
            ObDereferenceObject((*OpenPacket).FileObject as PVOID);
        }
    } else {
        (*(*OpenPacket).FileObject).Flags |= FO_HANDLE_CREATED;
        *FileHandle = LocalHandle;
        (*IoStatusBlock).Information = (*OpenPacket).Information;
        (*IoStatusBlock).__bindgen_anon_1.Status = (*OpenPacket).FinalStatus;
        Status = (*OpenPacket).FinalStatus;
    }

    if (*OpenPacket).ParseCheck != 0 && !(*OpenPacket).FileObject.is_null() {
        ObDereferenceObject((*OpenPacket).FileObject as PVOID);
    }

    ExFreePool(OpenPacket as PVOID);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn IoCheckQuerySetFileInformation(
    FileInformationClass: u32,
    Length: ULONG,
    SetOperation: BOOLEAN,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn IoCheckQuotaBufferValidity(
    QuotaBuffer: *mut c_void,
    QuotaLength: ULONG,
    ErrorOffset: *mut ULONG,
) -> NTSTATUS {
    let QuotaBuffer = opaque_ptr(QuotaBuffer);
    let ErrorOffset = opaque_ptr(ErrorOffset);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn IoCreateFile(
    FileHandle: *mut HANDLE,
    DesiredAccess: ACCESS_MASK,
    ObjectAttributes: POBJECT_ATTRIBUTES,
    IoStatusBlock: PIO_STATUS_BLOCK,
    AllocationSize: *mut LARGE_INTEGER,
    FileAttributes: ULONG,
    ShareAccess: ULONG,
    Disposition: ULONG,
    CreateOptions: ULONG,
    EaBuffer: PVOID,
    EaLength: ULONG,
    CreateFileType: u32,
    ExtraCreateParameters: PVOID,
    Options: ULONG,
) -> NTSTATUS {
    let FileHandle = opaque_ptr(FileHandle);
    let ObjectAttributes = opaque_ptr(ObjectAttributes);
    let IoStatusBlock = opaque_ptr(IoStatusBlock);
    let AllocationSize = opaque_ptr(AllocationSize);
    let EaBuffer = opaque_ptr(EaBuffer);
    let ExtraCreateParameters = opaque_ptr(ExtraCreateParameters);
    IopCreateFile(
        FileHandle,
        DesiredAccess,
        ObjectAttributes,
        IoStatusBlock,
        AllocationSize,
        FileAttributes,
        ShareAccess,
        Disposition,
        CreateOptions,
        EaBuffer,
        EaLength,
        CreateFileType,
        ExtraCreateParameters,
        Options,
        0,
        null_mut(),
    )
}

#[no_mangle]
pub unsafe extern "system" fn IoCreateFileSpecifyDeviceObjectHint(
    FileHandle: *mut HANDLE,
    DesiredAccess: ACCESS_MASK,
    ObjectAttributes: POBJECT_ATTRIBUTES,
    IoStatusBlock: PIO_STATUS_BLOCK,
    AllocationSize: *mut LARGE_INTEGER,
    FileAttributes: ULONG,
    ShareAccess: ULONG,
    Disposition: ULONG,
    CreateOptions: ULONG,
    EaBuffer: PVOID,
    EaLength: ULONG,
    CreateFileType: u32,
    ExtraCreateParameters: PVOID,
    Options: ULONG,
    DeviceObject: PVOID,
) -> NTSTATUS {
    let FileHandle = opaque_ptr(FileHandle);
    let ObjectAttributes = opaque_ptr(ObjectAttributes);
    let IoStatusBlock = opaque_ptr(IoStatusBlock);
    let AllocationSize = opaque_ptr(AllocationSize);
    let EaBuffer = opaque_ptr(EaBuffer);
    let ExtraCreateParameters = opaque_ptr(ExtraCreateParameters);
    let DeviceObject = opaque_ptr(DeviceObject);
    let Flags = if !DeviceObject.is_null() {
        IOP_CREATE_FILE_OBJECT_EXTENSION | IOP_USE_TOP_LEVEL_DEVICE_HINT
    } else {
        0
    };

    IopCreateFile(
        FileHandle,
        DesiredAccess,
        ObjectAttributes,
        IoStatusBlock,
        AllocationSize,
        FileAttributes,
        ShareAccess,
        Disposition,
        CreateOptions,
        EaBuffer,
        EaLength,
        CreateFileType,
        ExtraCreateParameters,
        Options | IO_NO_PARAMETER_CHECKING,
        Flags,
        DeviceObject as PDEVICE_OBJECT,
    )
}

#[no_mangle]
pub unsafe extern "system" fn IoCreateStreamFileObjectEx(
    FileObject: PFILE_OBJECT,
    DeviceObject: PDEVICE_OBJECT,
    FileObjectHandle: *mut HANDLE,
) -> PFILE_OBJECT {
    let FileObject = opaque_ptr(FileObject);
    let DeviceObject = opaque_ptr(DeviceObject);
    let FileObjectHandle = opaque_ptr(FileObjectHandle);
    let TargetDevice = if !FileObject.is_null() {
        (*FileObject).DeviceObject
    } else {
        DeviceObject
    };

    InterlockedIncrement(&mut (*TargetDevice).ReferenceCount);
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = zeroed();
    InitializeObjectAttributes(&mut ObjectAttributes, null_mut(), 0, 0 as _, null_mut());

    let mut CreatedFileObject: PFILE_OBJECT = null_mut();
    let Status = ObCreateObject(
        0i8,
        IoFileObjectType,
        &mut ObjectAttributes,
        0i8,
        null_mut(),
        size_of::<_FILE_OBJECT>() as u32,
        size_of::<_FILE_OBJECT>() as u32,
        0,
        &mut CreatedFileObject as *mut _ as *mut PVOID,
    );
    if Status < 0 {
        IopDereferenceDeviceObject(TargetDevice, 0u8);
        ExRaiseStatus(Status);
    }

    write_bytes(CreatedFileObject as *mut u8, 0, size_of::<_FILE_OBJECT>());
    (*CreatedFileObject).DeviceObject = TargetDevice;
    (*CreatedFileObject).Type = IO_TYPE_FILE as i16;
    (*CreatedFileObject).Size = size_of::<_FILE_OBJECT>() as i16;
    (*CreatedFileObject).Flags = FO_STREAM_FILE;

    KeInitializeEvent(&mut (*CreatedFileObject).Event, 1, 0);

    let mut FileHandle: HANDLE = 0 as _;
    let Status = ObInsertObject(
        CreatedFileObject as PVOID,
        null_mut(),
        FILE_READ_DATA,
        1,
        &mut CreatedFileObject as *mut _ as *mut PVOID,
        &mut FileHandle,
    );
    if Status < 0 {
        ExRaiseStatus(Status);
    }

    (*CreatedFileObject).Flags |= FO_HANDLE_CREATED;

    if !(*TargetDevice).Vpb.is_null() {
        InterlockedIncrement(&raw mut (*(*TargetDevice).Vpb).ReferenceCount as *mut LONG);
    }

    if !FileObjectHandle.is_null() {
        *FileObjectHandle = FileHandle;
        ObDereferenceObject(CreatedFileObject as PVOID);
    } else {
        ObCloseHandle(FileHandle, 0i8);
    }

    CreatedFileObject
}

#[no_mangle]
pub unsafe extern "system" fn IoCreateStreamFileObject(
    FileObject: PFILE_OBJECT,
    DeviceObject: PDEVICE_OBJECT,
) -> PFILE_OBJECT {
    let FileObject = opaque_ptr(FileObject);
    let DeviceObject = opaque_ptr(DeviceObject);
    IoCreateStreamFileObjectEx(FileObject, DeviceObject, null_mut())
}

#[no_mangle]
pub unsafe extern "system" fn IoCreateStreamFileObjectLite(
    FileObject: PFILE_OBJECT,
    DeviceObject: PDEVICE_OBJECT,
) -> PFILE_OBJECT {
    let FileObject = opaque_ptr(FileObject);
    let DeviceObject = opaque_ptr(DeviceObject);
    let TargetDevice = if !FileObject.is_null() {
        (*FileObject).DeviceObject
    } else {
        DeviceObject
    };

    InterlockedIncrement(&mut (*TargetDevice).ReferenceCount);
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = zeroed();
    InitializeObjectAttributes(&mut ObjectAttributes, null_mut(), 0, 0 as _, null_mut());

    let mut CreatedFileObject: PFILE_OBJECT = null_mut();
    let Status = ObCreateObject(
        0i8,
        IoFileObjectType,
        &mut ObjectAttributes,
        0i8,
        null_mut(),
        size_of::<_FILE_OBJECT>() as u32,
        size_of::<_FILE_OBJECT>() as u32,
        0,
        &mut CreatedFileObject as *mut _ as *mut PVOID,
    );
    if Status < 0 {
        IopDereferenceDeviceObject(TargetDevice, 0u8);
        ExRaiseStatus(Status);
    }

    write_bytes(CreatedFileObject as *mut u8, 0, size_of::<_FILE_OBJECT>());
    (*CreatedFileObject).DeviceObject = TargetDevice;
    (*CreatedFileObject).Type = IO_TYPE_FILE as i16;
    (*CreatedFileObject).Size = size_of::<_FILE_OBJECT>() as i16;
    (*CreatedFileObject).Flags = FO_STREAM_FILE;

    KeInitializeEvent(&mut (*CreatedFileObject).Event, 1, 0);

    ObFreeObjectCreateInfoBuffer(
        (*(OBJECT_TO_OBJECT_HEADER(CreatedFileObject as PVOID) as *mut _OBJECT_HEADER))
            .__bindgen_anon_2.ObjectCreateInfo,
    );
    (*(OBJECT_TO_OBJECT_HEADER(CreatedFileObject as PVOID) as *mut _OBJECT_HEADER)).__bindgen_anon_2.ObjectCreateInfo =
        null_mut();

    (*CreatedFileObject).Flags |= FO_HANDLE_CREATED;

    if !(*TargetDevice).Vpb.is_null() {
        InterlockedIncrement(&raw mut (*(*TargetDevice).Vpb).ReferenceCount as *mut LONG);
    }

    CreatedFileObject
}

#[no_mangle]
pub unsafe extern "system" fn IoGetFileObjectGenericMapping() -> PGENERIC_MAPPING {
    &raw const IopFileMapping as *const _ as PGENERIC_MAPPING
}

#[no_mangle]
pub unsafe extern "system" fn IoIsFileOriginRemote(FileObject: PFILE_OBJECT) -> BOOLEAN {
    let FileObject = opaque_ptr(FileObject);
    if (*FileObject).Flags & FO_REMOTE_ORIGIN != 0 {
        1
    } else {
        0
    }
}

#[no_mangle]
pub unsafe extern "system" fn IoFastQueryNetworkAttributes(
    ObjectAttributes: POBJECT_ATTRIBUTES,
    DesiredAccess: ACCESS_MASK,
    OpenOptions: ULONG,
    IoStatus: PIO_STATUS_BLOCK,
    Buffer: *mut _FILE_NETWORK_OPEN_INFORMATION,
) -> BOOLEAN {
    let ObjectAttributes = opaque_ptr(ObjectAttributes);
    let IoStatus = opaque_ptr(IoStatus);
    let Buffer = opaque_ptr(Buffer);
    let mut OpenPacket: _OPEN_PACKET = zeroed();
    let mut LocalFileObject: _DUMMY_FILE_OBJECT = zeroed();

    OpenPacket.Type = IO_TYPE_OPEN_PACKET as i16;
    OpenPacket.Size = size_of::<_OPEN_PACKET>() as i16;
    OpenPacket.CreateOptions = OpenOptions | FILE_OPEN_REPARSE_POINT;
    OpenPacket.ShareAccess = (FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE) as u16;
    OpenPacket.Options = IO_FORCE_ACCESS_CHECK;
    OpenPacket.Disposition = FILE_OPEN;
    OpenPacket.NetworkInformation = Buffer;
    OpenPacket.QueryOnly = 1;
    OpenPacket.FullAttributes = 1;
    OpenPacket.LocalFileObject = core::ptr::addr_of_mut!(LocalFileObject);

    let mut Handle: HANDLE = 0 as _;
    let Status = ObOpenObjectByName(
        ObjectAttributes,
        null_mut(),
        0i8,
        null_mut(),
        DesiredAccess,
        core::ptr::addr_of_mut!(OpenPacket) as PVOID,
        &mut Handle,
    );
    if OpenPacket.ParseCheck == 0 {
        (*IoStatus).__bindgen_anon_1.Status = Status;
    } else {
        (*IoStatus).__bindgen_anon_1.Status = OpenPacket.FinalStatus;
        (*IoStatus).Information = OpenPacket.Information;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn IoUpdateShareAccess(
    FileObject: PFILE_OBJECT,
    ShareAccess: PSHARE_ACCESS,
) {
    let FileObject = opaque_ptr(FileObject);
    if (*FileObject).Flags & FO_FILE_OBJECT_HAS_EXTENSION != 0 {
        return;
    }

    if ((*FileObject).ReadAccess != 0)
        || ((*FileObject).WriteAccess != 0)
        || ((*FileObject).DeleteAccess != 0)
    {
        (*ShareAccess).OpenCount += 1;
        (*ShareAccess).Readers += (*FileObject).ReadAccess as u32;
        (*ShareAccess).Writers += (*FileObject).WriteAccess as u32;
        (*ShareAccess).Deleters += (*FileObject).DeleteAccess as u32;
        (*ShareAccess).SharedRead += (*FileObject).SharedRead as u32;
        (*ShareAccess).SharedWrite += (*FileObject).SharedWrite as u32;
        (*ShareAccess).SharedDelete += (*FileObject).SharedDelete as u32;
    }
}

#[no_mangle]
pub unsafe extern "system" fn IoCheckShareAccess(
    DesiredAccess: ACCESS_MASK,
    DesiredShareAccess: ULONG,
    FileObject: PFILE_OBJECT,
    ShareAccess: PSHARE_ACCESS,
    Update: BOOLEAN,
) -> NTSTATUS {
    let FileObject = opaque_ptr(FileObject);
    let ReadAccess = (DesiredAccess & (FILE_READ_DATA | FILE_EXECUTE)) != 0;
    let WriteAccess = (DesiredAccess & (FILE_WRITE_DATA | FILE_APPEND_DATA)) != 0;
    let DeleteAccess = (DesiredAccess & DELETE) != 0;

    (*FileObject).ReadAccess = if ReadAccess { 1 } else { 0 };
    (*FileObject).WriteAccess = if WriteAccess { 1 } else { 0 };
    (*FileObject).DeleteAccess = if DeleteAccess { 1 } else { 0 };

    if (*FileObject).Flags & FO_FILE_OBJECT_HAS_EXTENSION != 0 {
        return STATUS_SUCCESS;
    }

    if ReadAccess || WriteAccess || DeleteAccess {
        let SharedRead = (DesiredShareAccess & FILE_SHARE_READ) != 0;
        let SharedWrite = (DesiredShareAccess & FILE_SHARE_WRITE) != 0;
        let SharedDelete = (DesiredShareAccess & FILE_SHARE_DELETE) != 0;

        if (ReadAccess && ((*ShareAccess).SharedRead < (*ShareAccess).OpenCount))
            || (WriteAccess && ((*ShareAccess).SharedWrite < (*ShareAccess).OpenCount))
            || (DeleteAccess && ((*ShareAccess).SharedDelete < (*ShareAccess).OpenCount))
            || (((*ShareAccess).Readers != 0) && !SharedRead)
            || (((*ShareAccess).Writers != 0) && !SharedWrite)
            || (((*ShareAccess).Deleters != 0) && !SharedDelete)
        {
            return STATUS_SHARING_VIOLATION;
        }

        (*FileObject).SharedRead = if SharedRead { 1 } else { 0 };
        (*FileObject).SharedWrite = if SharedWrite { 1 } else { 0 };
        (*FileObject).SharedDelete = if SharedDelete { 1 } else { 0 };

        if Update != 0 {
            (*ShareAccess).OpenCount += 1;
            (*ShareAccess).Readers += if ReadAccess { 1 } else { 0 };
            (*ShareAccess).Writers += if WriteAccess { 1 } else { 0 };
            (*ShareAccess).Deleters += if DeleteAccess { 1 } else { 0 };
            (*ShareAccess).SharedRead += if SharedRead { 1 } else { 0 };
            (*ShareAccess).SharedWrite += if SharedWrite { 1 } else { 0 };
            (*ShareAccess).SharedDelete += if SharedDelete { 1 } else { 0 };
        }
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn IoRemoveShareAccess(
    FileObject: PFILE_OBJECT,
    ShareAccess: PSHARE_ACCESS,
) {
    let FileObject = opaque_ptr(FileObject);
    if (*FileObject).Flags & FO_FILE_OBJECT_HAS_EXTENSION != 0 {
        return;
    }

    if ((*FileObject).ReadAccess != 0)
        || ((*FileObject).WriteAccess != 0)
        || ((*FileObject).DeleteAccess != 0)
    {
        (*ShareAccess).OpenCount -= 1;
        (*ShareAccess).Readers -= (*FileObject).ReadAccess as u32;
        (*ShareAccess).Writers -= (*FileObject).WriteAccess as u32;
        (*ShareAccess).Deleters -= (*FileObject).DeleteAccess as u32;
        (*ShareAccess).SharedRead -= (*FileObject).SharedRead as u32;
        (*ShareAccess).SharedWrite -= (*FileObject).SharedWrite as u32;
        (*ShareAccess).SharedDelete -= (*FileObject).SharedDelete as u32;
    }
}

#[no_mangle]
pub unsafe extern "system" fn IoSetShareAccess(
    DesiredAccess: ACCESS_MASK,
    DesiredShareAccess: ULONG,
    FileObject: PFILE_OBJECT,
    ShareAccess: PSHARE_ACCESS,
) {
    let FileObject = opaque_ptr(FileObject);
    let ReadAccess = (DesiredAccess & (FILE_READ_DATA | FILE_EXECUTE)) != 0;
    let WriteAccess = (DesiredAccess & (FILE_WRITE_DATA | FILE_APPEND_DATA)) != 0;
    let DeleteAccess = (DesiredAccess & DELETE) != 0;

    let mut Update = 1u8;
    if (*FileObject).Flags & FO_FILE_OBJECT_HAS_EXTENSION != 0 {
        Update = 0;
    }

    (*FileObject).ReadAccess = if ReadAccess { 1 } else { 0 };
    (*FileObject).WriteAccess = if WriteAccess { 1 } else { 0 };
    (*FileObject).DeleteAccess = if DeleteAccess { 1 } else { 0 };

    if !ReadAccess && !WriteAccess && !DeleteAccess {
        if Update == 0 {
            return;
        }
        (*ShareAccess).OpenCount = 0;
        (*ShareAccess).Readers = 0;
        (*ShareAccess).Writers = 0;
        (*ShareAccess).Deleters = 0;
        (*ShareAccess).SharedRead = 0;
        (*ShareAccess).SharedWrite = 0;
        (*ShareAccess).SharedDelete = 0;
    } else {
        let SharedRead = (DesiredShareAccess & FILE_SHARE_READ) != 0;
        let SharedWrite = (DesiredShareAccess & FILE_SHARE_WRITE) != 0;
        let SharedDelete = (DesiredShareAccess & FILE_SHARE_DELETE) != 0;

        (*FileObject).SharedRead = if SharedRead { 1 } else { 0 };
        (*FileObject).SharedWrite = if SharedWrite { 1 } else { 0 };
        (*FileObject).SharedDelete = if SharedDelete { 1 } else { 0 };

        if Update == 0 {
            return;
        }

        (*ShareAccess).OpenCount = 1;
        (*ShareAccess).Readers = if ReadAccess { 1 } else { 0 };
        (*ShareAccess).Writers = if WriteAccess { 1 } else { 0 };
        (*ShareAccess).Deleters = if DeleteAccess { 1 } else { 0 };
        (*ShareAccess).SharedRead = if SharedRead { 1 } else { 0 };
        (*ShareAccess).SharedWrite = if SharedWrite { 1 } else { 0 };
        (*ShareAccess).SharedDelete = if SharedDelete { 1 } else { 0 };
    }
}

#[no_mangle]
pub unsafe extern "system" fn IoCancelFileOpen(
    DeviceObject: PDEVICE_OBJECT,
    FileObject: PFILE_OBJECT,
) {
    let DeviceObject = opaque_ptr(DeviceObject);
    let FileObject = opaque_ptr(FileObject);
    if (*FileObject).Flags & FO_HANDLE_CREATED != 0 {
        KeBugCheckEx(
            ntkrnl_sys::bugcodes::INVALID_CANCEL_OF_FILE_OPEN,
            FileObject as ULONG_PTR,
            DeviceObject as ULONG_PTR,
            0,
            0,
        );
    }

    let mut Event: _KEVENT = zeroed();
    KeInitializeEvent(&mut Event, 1, 0);  // SynchronizationEvent
    KeClearEvent(&mut (*FileObject).Event);

    let Irp = IopAllocateIrpMustSucceed((*DeviceObject).StackSize as i8);
    (*Irp).Tail.Overlay.Thread = PsGetCurrentThread() as PETHREAD;
    (*Irp).UserEvent = core::ptr::addr_of_mut!(Event) as *mut _KEVENT;
    (*Irp).UserIosb = addr_of_mut!((*Irp).IoStatus) as PIO_STATUS_BLOCK;
    (*Irp).Overlay.AsynchronousParameters.__bindgen_anon_1.UserApcRoutine = None;
    (*Irp).Tail.Overlay.OriginalFileObject = FileObject;
    (*Irp).RequestorMode = 0i8;
    (*Irp).Flags = IRP_CLOSE_OPERATION | IRP_SYNCHRONOUS_API;

    let Stack = IoGetNextIrpStackLocation(Irp);
    (*Stack).MajorFunction = IRP_MJ_CLEANUP as u8;
    (*Stack).FileObject = FileObject;

    IopQueueIrpToThread(Irp);

    let Status = core::hint::black_box(IoCallDriver(DeviceObject, Irp));
    if Status == STATUS_PENDING {
        KeWaitForSingleObject(core::ptr::addr_of_mut!(Event) as PVOID, 1, 0i8, 0, null_mut());
    }

    let mut OldIrql: KIRQL = 0;
    // [guard] KeRaiseIrql(2, &mut OldIrql);
    let irql_guard = unsafe { ntkrnl_sys::guards::IrqlGuard::raise(2) };
    IopUnQueueIrpFromThread(Irp);
    drop(irql_guard);

    IoFreeIrp(Irp);

    KeClearEvent(&mut (*FileObject).Event);
    (*FileObject).Flags |= FO_FILE_OPEN_CANCELLED;
}

#[no_mangle]
pub unsafe extern "system" fn IoQueryFileDosDeviceName(
    FileObject: PFILE_OBJECT,
    _OBJECT_INFORMATION_CLASS_ObjectNameInformation: *mut POBJECT_NAME_INFORMATION,
) -> NTSTATUS {
    let FileObject = opaque_ptr(FileObject);
    let _OBJECT_INFORMATION_CLASS_ObjectNameInformation = opaque_ptr(_OBJECT_INFORMATION_CLASS_ObjectNameInformation);
    let mut ReturnLength: ULONG = 200;
    let mut Length = 200;

    loop {
        let LocalInfo = ExAllocatePoolWithTag(PagedPool as u32, Length as usize, u32::from_le_bytes(*b"nDoI"))
            as POBJECT_NAME_INFORMATION;
        if LocalInfo.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        let Status = IopQueryNameInternal(
            FileObject as PVOID,
            1,
            1,
            LocalInfo,
            Length,
            &mut ReturnLength,
            0i8,
        );
        if Status == STATUS_SUCCESS {
            *_OBJECT_INFORMATION_CLASS_ObjectNameInformation = LocalInfo;
            return STATUS_SUCCESS;
        }

        ExFreePoolWithTag(LocalInfo as PVOID, u32::from_le_bytes(*b"nDoI"));

        if Status != STATUS_BUFFER_OVERFLOW {
            return Status;
        }

        Length = ReturnLength;
    }
}

#[no_mangle]
pub unsafe extern "system" fn IoSetFileOrigin(
    FileObject: PFILE_OBJECT,
    Remote: BOOLEAN,
) -> NTSTATUS {
    let FileObject = opaque_ptr(FileObject);
    let FlagSet = if (*FileObject).Flags & FO_REMOTE_ORIGIN != 0 {
        1
    } else {
        0
    };

    if Remote != 0 && FlagSet == 0 {
        (*FileObject).Flags |= FO_REMOTE_ORIGIN;
    } else if Remote == 0 && FlagSet != 0 {
        (*FileObject).Flags &= !FO_REMOTE_ORIGIN;
    } else {
        return STATUS_INVALID_PARAMETER_MIX;
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn NtCreateFile(
    FileHandle: *mut HANDLE,
    DesiredAccess: ACCESS_MASK,
    ObjectAttributes: POBJECT_ATTRIBUTES,
    IoStatusBlock: PIO_STATUS_BLOCK,
    AllocateSize: *mut LARGE_INTEGER,
    FileAttributes: ULONG,
    ShareAccess: ULONG,
    CreateDisposition: ULONG,
    CreateOptions: ULONG,
    EaBuffer: PVOID,
    EaLength: ULONG,
) -> NTSTATUS {
    let FileHandle = opaque_ptr(FileHandle);
    let ObjectAttributes = opaque_ptr(ObjectAttributes);
    let IoStatusBlock = opaque_ptr(IoStatusBlock);
    let AllocateSize = opaque_ptr(AllocateSize);
    let EaBuffer = opaque_ptr(EaBuffer);
    IoCreateFile(
        FileHandle,
        DesiredAccess,
        ObjectAttributes,
        IoStatusBlock,
        AllocateSize,
        FileAttributes,
        ShareAccess,
        CreateDisposition,
        CreateOptions,
        EaBuffer,
        EaLength,
        CreateFileTypeNone,
        null_mut(),
        0,
    )
}

#[no_mangle]
pub unsafe extern "system" fn NtCreateMailslotFile(
    FileHandle: *mut HANDLE,
    DesiredAccess: ACCESS_MASK,
    ObjectAttributes: POBJECT_ATTRIBUTES,
    IoStatusBlock: PIO_STATUS_BLOCK,
    CreateOptions: ULONG,
    MailslotQuota: ULONG,
    MaxMessageSize: ULONG,
    TimeOut: *mut LARGE_INTEGER,
) -> NTSTATUS {
    let FileHandle = opaque_ptr(FileHandle);
    let ObjectAttributes = opaque_ptr(ObjectAttributes);
    let IoStatusBlock = opaque_ptr(IoStatusBlock);
    let TimeOut = opaque_ptr(TimeOut);
    let mut Buffer: _MAILSLOT_CREATE_PARAMETERS = core::mem::zeroed();

    if !TimeOut.is_null() {
        if KeGetPreviousMode() != KernelMode {
            Buffer.ReadTimeout = core::mem::transmute(*TimeOut);
        } else {
            Buffer.ReadTimeout = core::mem::transmute(*TimeOut);
        }
        Buffer.TimeoutSpecified = 1u8;
    } else {
        Buffer.TimeoutSpecified = 0u8;
    }

    Buffer.MailslotQuota = MailslotQuota;
    Buffer.MaximumMessageSize = MaxMessageSize;

    IoCreateFile(
        FileHandle,
        DesiredAccess,
        ObjectAttributes,
        IoStatusBlock,
        core::ptr::null_mut(),
        0,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        FILE_CREATE,
        CreateOptions,
        core::ptr::null_mut(),
        0,
        CreateFileTypeMailslot,
        &mut Buffer as *mut _ as PVOID,
        0,
    )
}

#[no_mangle]
pub unsafe extern "system" fn NtCreateNamedPipeFile(
    FileHandle: *mut HANDLE,
    DesiredAccess: ACCESS_MASK,
    ObjectAttributes: POBJECT_ATTRIBUTES,
    IoStatusBlock: PIO_STATUS_BLOCK,
    ShareAccess: ULONG,
    CreateDisposition: ULONG,
    CreateOptions: ULONG,
    NamedPipeType: ULONG,
    ReadMode: ULONG,
    CompletionMode: ULONG,
    MaximumInstances: ULONG,
    InboundQuota: ULONG,
    OutboundQuota: ULONG,
    DefaultTimeout: *mut LARGE_INTEGER,
) -> NTSTATUS {
    let FileHandle = opaque_ptr(FileHandle);
    let ObjectAttributes = opaque_ptr(ObjectAttributes);
    let IoStatusBlock = opaque_ptr(IoStatusBlock);
    let DefaultTimeout = opaque_ptr(DefaultTimeout);
    let mut Buffer: _NAMED_PIPE_CREATE_PARAMETERS = core::mem::zeroed();

    if !DefaultTimeout.is_null() {
        if KeGetPreviousMode() != KernelMode {
            Buffer.DefaultTimeout = core::mem::transmute(*DefaultTimeout);
        } else {
            Buffer.DefaultTimeout = core::mem::transmute(*DefaultTimeout);
        }
        Buffer.TimeoutSpecified = 1u8;
    } else {
        Buffer.TimeoutSpecified = 0u8;
    }

    Buffer.NamedPipeType = NamedPipeType;
    Buffer.ReadMode = ReadMode;
    Buffer.CompletionMode = CompletionMode;
    Buffer.MaximumInstances = MaximumInstances;
    Buffer.InboundQuota = InboundQuota;
    Buffer.OutboundQuota = OutboundQuota;

    IoCreateFile(
        FileHandle,
        DesiredAccess,
        ObjectAttributes,
        IoStatusBlock,
        core::ptr::null_mut(),
        0,
        ShareAccess,
        CreateDisposition,
        CreateOptions,
        core::ptr::null_mut(),
        0,
        CreateFileTypeNamedPipe,
        &mut Buffer as *mut _ as PVOID,
        0,
    )
}

#[no_mangle]
pub unsafe extern "system" fn NtFlushWriteBuffer() -> NTSTATUS {
    KeFlushWriteBuffer();
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn NtOpenFile(
    FileHandle: *mut HANDLE,
    DesiredAccess: ACCESS_MASK,
    ObjectAttributes: POBJECT_ATTRIBUTES,
    IoStatusBlock: PIO_STATUS_BLOCK,
    ShareAccess: ULONG,
    OpenOptions: ULONG,
) -> NTSTATUS {
    let FileHandle = opaque_ptr(FileHandle);
    let ObjectAttributes = opaque_ptr(ObjectAttributes);
    let IoStatusBlock = opaque_ptr(IoStatusBlock);
    IoCreateFile(
        FileHandle,
        DesiredAccess,
        ObjectAttributes,
        IoStatusBlock,
        core::ptr::null_mut(),
        0,
        ShareAccess,
        FILE_OPEN,
        OpenOptions,
        core::ptr::null_mut(),
        0,
        CreateFileTypeNone,
        core::ptr::null_mut(),
        0,
    )
}

#[no_mangle]
pub unsafe extern "system" fn NtQueryAttributesFile(
    ObjectAttributes: POBJECT_ATTRIBUTES,
    FileInformation: PVOID,
) -> NTSTATUS {
    let ObjectAttributes = opaque_ptr(ObjectAttributes);
    let FileInformation = opaque_ptr(FileInformation);
    IopQueryAttributesFile(
        ObjectAttributes,
        FileBasicInformation,
        core::mem::size_of::<_FILE_BASIC_INFORMATION>() as ULONG,
        FileInformation,
    )
}

#[no_mangle]
pub unsafe extern "system" fn NtQueryFullAttributesFile(
    ObjectAttributes: POBJECT_ATTRIBUTES,
    FileInformation: PVOID,
) -> NTSTATUS {
    let ObjectAttributes = opaque_ptr(ObjectAttributes);
    let FileInformation = opaque_ptr(FileInformation);
    IopQueryAttributesFile(
        ObjectAttributes,
        FileNetworkOpenInformation,
        core::mem::size_of::<_FILE_NETWORK_OPEN_INFORMATION>() as ULONG,
        FileInformation,
    )
}

#[no_mangle]
pub unsafe extern "system" fn NtCancelIoFile(
    FileHandle: HANDLE,
    IoStatusBlock: PIO_STATUS_BLOCK,
) -> NTSTATUS {
    let FileHandle = opaque_ptr(FileHandle);
    let IoStatusBlock = opaque_ptr(IoStatusBlock);
    let mut FileObject: PFILE_OBJECT = core::ptr::null_mut();
    let Thread: PETHREAD;
    let mut Irp: PIRP;
    let mut OldIrql: KIRQL = 0;
    let mut OurIrpsInList: BOOLEAN = 0u8;
    let mut Interval: LARGE_INTEGER = core::mem::zeroed();
    let PreviousMode: KPROCESSOR_MODE = KeGetPreviousMode();
    let mut Status: NTSTATUS;
    let ListHead: PLIST_ENTRY;
    let mut NextEntry: PLIST_ENTRY;

    if PreviousMode != KernelMode {
        probe_user_buffer(
            IoStatusBlock as *const core::ffi::c_void,
            core::mem::size_of::<_IO_STATUS_BLOCK>(),
            core::mem::align_of::<_IO_STATUS_BLOCK>(),
            PreviousMode,
        ).map_err(|e| return e).ok();
    }

    Status = ObReferenceObjectByHandle(
        FileHandle,
        0,
        IoFileObjectType,
        PreviousMode,
        &mut FileObject as *mut _ as *mut PVOID,
        core::ptr::null_mut(),
    );
    if Status < 0 { return Status; }

    // [guard] KeRaiseIrql(APC_LEVEL as u8, &mut OldIrql);
    let irql_guard_0 = unsafe { ntkrnl_sys::guards::IrqlGuard::raise(APC_LEVEL as u8) };
    Thread = PsGetCurrentThread();
    IopUpdateOperationCount(IopOtherTransfer);

    ListHead = core::ptr::addr_of_mut!((*(Thread as *mut _ETHREAD)).IrpList);
    NextEntry = (*ListHead).Flink;
    while NextEntry != ListHead {
        Irp = (NextEntry as usize - core::mem::offset_of!(_IRP, ThreadListEntry)) as PIRP;
        if (*Irp).Tail.Overlay.OriginalFileObject == FileObject {
            IoCancelIrp(Irp);
            OurIrpsInList = 1u8;
        }
        NextEntry = (*NextEntry).Flink;
    }

    drop(irql_guard_0);

    if OurIrpsInList != 0 {
        Interval.QuadPart = -100000i64;
        while OurIrpsInList != 0 {
            KeDelayExecutionThread(KernelMode, 0u8, &mut Interval);
            OurIrpsInList = 0u8;

            // [guard] KeRaiseIrql(APC_LEVEL as u8, &mut OldIrql);
            let irql_guard_1 = unsafe { ntkrnl_sys::guards::IrqlGuard::raise(APC_LEVEL as u8) };
            NextEntry = (*ListHead).Flink;
            while NextEntry != ListHead {
                Irp = (NextEntry as usize - core::mem::offset_of!(_IRP, ThreadListEntry)) as PIRP;
                if (*Irp).Tail.Overlay.OriginalFileObject == FileObject {
                    OurIrpsInList = 1u8;
                    break;
                }
                NextEntry = (*NextEntry).Flink;
            }
            drop(irql_guard_1);
        }
    }

    (*IoStatusBlock).__bindgen_anon_1.Status = STATUS_SUCCESS;
    (*IoStatusBlock).Information = 0;

    ObDereferenceObject(FileObject as PVOID);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn NtDeleteFile(
    ObjectAttributes: POBJECT_ATTRIBUTES,
) -> NTSTATUS {
    let ObjectAttributes = opaque_ptr(ObjectAttributes);
    let mut LocalFileObject: _DUMMY_FILE_OBJECT = core::mem::zeroed();
    let mut Handle: HANDLE = core::ptr::null_mut();
    let AccessMode: KPROCESSOR_MODE = KeGetPreviousMode();
    let mut OpenPacket: _OPEN_PACKET = core::mem::zeroed();

    OpenPacket.Type = IO_TYPE_OPEN_PACKET as i16;
    OpenPacket.Size = core::mem::size_of::<_OPEN_PACKET>() as i16;
    OpenPacket.CreateOptions = FILE_DELETE_ON_CLOSE;
    OpenPacket.ShareAccess = (FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE) as u16;
    OpenPacket.Disposition = FILE_OPEN;
    OpenPacket.DeleteOnly = 1u8;
    OpenPacket.LocalFileObject = core::ptr::addr_of_mut!(LocalFileObject);

    IopUpdateOperationCount(IopOtherTransfer);

    let Status = ObOpenObjectByName(
        ObjectAttributes,
        core::ptr::null_mut(),
        AccessMode,
        core::ptr::null_mut(),
        DELETE,
        core::ptr::addr_of_mut!(OpenPacket) as PVOID,
        &mut Handle,
    );
    if OpenPacket.ParseCheck == 0 { return Status; }

    OpenPacket.FinalStatus
}
